//! Password hashing off the game loop. An Argon2
//! verify costs ~9 ms of frozen world per login attempt, successful or not.
//!
//! Exactly one worker thread, which is load-bearing three times over: it cannot cost the loop more than an inline
//! verify, it caps live hashing arenas at one, and writes apply in dispatch order only because a single thread
//! drains FIFO. The worker reads no game state and writes none; results return through the loop context in order.
//!
//! A login that is handed over is `Deferred` (the packet handler sends nothing until the verdict lands); a full
//! queue rejects the login rather than verifying on the loop.
//!
//! The save gate reads the world's state mirror ([`muo_server::World::state_mirror`]), a copy of `World.WorldState`.
//! shortcut: the skip of a job whose connection dropped while it waited is done when the result lands (the worker
//! reads no net state); it costs a hash, not correctness.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::thread::JoinHandle;
use std::time::Duration;

use muo_server::World;
use muo_server::iaccount::AccountId;
use muo_server::net_state::NetStateId;
use muo_server::world::WorldState;
use tracing::error;

use super::security::{self, PasswordProtectionAlgorithm};

/// Backstop, not a flood defense: connections are capped at 4096 and `SentFirstPacket` holds a connection to one
/// pending verify, so this can only trip if that invariant breaks.
pub const MAX_PENDING: usize = 4096;

/// How long the worker waits before it re-checks the save gate.
const SAVE_GATE_POLL: Duration = Duration::from_millis(50);

/// `PasswordJob`: work for the password thread, which reads no game state.
#[derive(Clone, Debug)]
pub struct PasswordJob {
    pub account: AccountId,
    /// Ties the job to a connection; `None` when the work is not gated on one.
    pub ns: Option<NetStateId>,
    /// Hash to verify against, with `verify_phrase`.
    pub stored_hash: Option<String>,
    /// Algorithm `stored_hash` was written with (resolved on the loop; the configured one is mutable state).
    pub stored_algorithm: PasswordProtectionAlgorithm,
    /// Phrase to verify, or `None` to skip verification.
    pub verify_phrase: Option<String>,
    /// Phrase to hash, or `None` when nothing needs writing.
    pub hash_phrase: Option<String>,
    pub target_algorithm: PasswordProtectionAlgorithm,
}

/// `PasswordOutcome`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PasswordOutcome {
    /// True when no verification was asked for, or it succeeded.
    pub verified: bool,
    /// The derived hash, or `None` when nothing was hashed or verification failed.
    pub hash: Option<String>,
}

/// `Compute(job)`: the work itself, with no state.
pub fn compute(job: &PasswordJob) -> PasswordOutcome {
    if let Some(phrase) = &job.verify_phrase
        && !security::validate_password(
            job.stored_algorithm,
            job.stored_hash.as_deref().unwrap_or_default(),
            phrase,
        )
    {
        return PasswordOutcome {
            verified: false,
            hash: None,
        };
    }
    PasswordOutcome {
        verified: true,
        hash: job
            .hash_phrase
            .as_deref()
            .and_then(|p| security::encrypt_password(job.target_algorithm, p)),
    }
}

/// Runs on the game loop with the result, free to touch game state.
pub type OnComplete = Box<dyn FnOnce(&mut World, &PasswordJob, &PasswordOutcome)>;

/// `Action<bool> onDone` of `SetPassword`.
pub type OnDone = Box<dyn FnOnce(&mut World, bool)>;

struct Thread {
    tx: Sender<(u64, PasswordJob)>,
    pending: Arc<AtomicUsize>,
    exit: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

/// The worker and the continuations of the jobs it holds, in the world's resources.
#[derive(Default)]
pub struct PasswordWorker {
    thread: Option<Thread>,
    continuations: HashMap<u64, OnComplete>,
    next_id: u64,
    /// `Enabled` overridden (tests).
    forced: Option<bool>,
}

/// `PasswordWorker.Enabled`: needs a spare core to move work to, which a 1-2 core host does not have. Off in debug
/// builds, where logins are rare and the inline path is easier to follow.
pub fn enabled(w: &World) -> bool {
    if let Some(f) = w.resources.get::<PasswordWorker>().and_then(|p| p.forced) {
        return f;
    }
    !cfg!(debug_assertions) && std::thread::available_parallelism().is_ok_and(|n| n.get() >= 4)
}

/// Forces `Enabled` on or off (tests).
pub fn set_enabled_for_testing(w: &mut World, on: bool) {
    w.resources.get_mut::<PasswordWorker>().forced = Some(on);
}

/// `CanRunNow()`, read from the worker thread: the world's state mirror.
fn can_run_now(state: &AtomicU8) -> bool {
    let s = state.load(Ordering::SeqCst);
    s == WorldState::Running as u8 || s == WorldState::WritingSave as u8
}

fn spawn(w: &mut World) -> Thread {
    let (tx, rx) = channel();
    let pending = Arc::new(AtomicUsize::new(0));
    let gate = w.state_mirror.clone();
    let exit = Arc::new(AtomicBool::new(false));
    let ctx = w.core.loop_context.clone();
    let (p, g, e) = (pending.clone(), gate, exit.clone());
    let handle = std::thread::Builder::new()
        .name("Password Worker".into())
        .spawn(move || execute(rx, ctx, p, g, e))
        .ok();
    // Stop at shutdown; HandleClosed skips InvokeShutdown when the server crashed, so the crash path needs its own.
    w.events.shutdown.subscribe(|w, _| stop(w));
    w.events.server_crashed.subscribe(|w, _| stop(w));
    Thread {
        tx,
        pending,
        exit,
        handle,
    }
}

fn execute(
    rx: Receiver<(u64, PasswordJob)>,
    ctx: muo_server::core_loop::EventLoopContext,
    pending: Arc<AtomicUsize>,
    gate: Arc<AtomicU8>,
    exit: Arc<AtomicBool>,
) {
    let mut queue: VecDeque<(u64, PasswordJob)> = VecDeque::new();
    while !exit.load(Ordering::SeqCst) {
        // A kernel block at zero CPU while there is nothing to do.
        let timeout = if queue.is_empty() {
            Duration::from_millis(250)
        } else {
            Duration::ZERO
        };
        match rx.recv_timeout(timeout) {
            Ok(item) => queue.push_back(item),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        while let Ok(item) = rx.try_recv() {
            queue.push_back(item);
        }
        if queue.is_empty() {
            continue;
        }
        if !can_run_now(&gate) {
            std::thread::sleep(SAVE_GATE_POLL);
            continue;
        }
        let Some((id, job)) = queue.pop_front() else {
            continue;
        };
        pending.fetch_sub(1, Ordering::SeqCst);
        let outcome = std::panic::catch_unwind(|| compute(&job)).unwrap_or_else(|_| {
            // A verdict must still come back, or the connection never gets a reply.
            error!("Password work failed for account {:?}", job.account);
            PasswordOutcome {
                verified: false,
                hash: None,
            }
        });
        ctx.post(move |w| apply(w, id, job, outcome));
    }
}

/// `Apply(job, outcome)`, on the loop.
fn apply(w: &mut World, id: u64, job: PasswordJob, outcome: PasswordOutcome) {
    let pw = w.resources.get_mut::<PasswordWorker>();
    let Some(done) = pw.continuations.remove(&id) else {
        return;
    };
    // Re-checked: a connection can drop while the result sits in the loop queue.
    if let Some(ns) = job.ns
        && w.net_state(ns).is_none_or(|d| d.cannot_send)
    {
        return;
    }
    if outcome.verified
        && let Some(hash) = &outcome.hash
        && let Some(a) = super::account_mut(w, job.account)
    {
        // ApplyPasswordWrite: last dispatched is last applied, as one worker drains FIFO.
        a.password_algorithm = job.target_algorithm;
        a.password = Some(hash.clone());
    }
    done(w, &job, &outcome);
}

/// `TryEnqueue(job)`: false when full, and the caller must then reject without verifying.
pub fn try_enqueue(w: &mut World, job: PasswordJob, on_complete: OnComplete) -> bool {
    if w.resources.get_mut::<PasswordWorker>().thread.is_none() {
        let t = spawn(w);
        w.resources.get_mut::<PasswordWorker>().thread = Some(t);
    }
    let pw = w.resources.get_mut::<PasswordWorker>();
    let Some(t) = &pw.thread else { return false };
    if t.pending.load(Ordering::SeqCst) >= MAX_PENDING {
        return false;
    }
    let id = pw.next_id;
    pw.next_id += 1;
    t.pending.fetch_add(1, Ordering::SeqCst);
    if t.tx.send((id, job)).is_err() {
        t.pending.fetch_sub(1, Ordering::SeqCst);
        return false;
    }
    pw.continuations.insert(id, on_complete);
    true
}

/// `SetPassword(account, plainPassword, onDone)`: hashes off the loop; `on_done(true)` once the write has landed.
/// A saturated queue pays the hash on the loop, since a password change must not be dropped.
pub fn set_password(
    w: &mut World,
    account: AccountId,
    plain_password: &str,
    on_done: Option<OnDone>,
) {
    let inline = |w: &mut World, on_done: Option<OnDone>| {
        super::set_password(w, account, plain_password);
        if let Some(d) = on_done {
            d(w, true);
        }
    };
    if !enabled(w) {
        return inline(w, on_done);
    }
    let target = super::handler::current_algorithm(w);
    let Some(a) = super::account(w, account) else {
        return;
    };
    let job = PasswordJob {
        account,
        ns: None,
        stored_hash: None,
        stored_algorithm: PasswordProtectionAlgorithm::None,
        verify_phrase: None,
        hash_phrase: Some(security::derive_phrase(target, &a.username, plain_password)),
        target_algorithm: target,
    };
    // The callback is moved into the job; if the queue is full the closure comes back through `cell`.
    let cell = std::rc::Rc::new(std::cell::RefCell::new(on_done));
    let c = cell.clone();
    let queued = try_enqueue(
        w,
        job,
        Box::new(move |w, _, outcome| {
            if let Some(d) = c.borrow_mut().take() {
                d(w, outcome.hash.is_some());
            }
        }),
    );
    if !queued {
        let d = cell.borrow_mut().take();
        inline(w, d);
    }
}

/// `Stop()`: asks the thread to exit and waits for it a few seconds.
pub fn stop(w: &mut World) {
    let Some(t) = w.resources.get_mut::<PasswordWorker>().thread.take() else {
        return;
    };
    t.exit.store(true, Ordering::SeqCst);
    drop(t.tx);
    if let Some(h) = t.handle {
        let end = std::time::Instant::now() + Duration::from_secs(5);
        while !h.is_finished() && std::time::Instant::now() < end {
            std::thread::sleep(Duration::from_millis(10));
        }
        if h.is_finished() {
            let _ = h.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(verify: Option<&str>, hash: Option<&str>, stored: Option<String>) -> PasswordJob {
        PasswordJob {
            account: AccountId(1),
            ns: None,
            stored_hash: stored,
            stored_algorithm: PasswordProtectionAlgorithm::Sha2,
            verify_phrase: verify.map(str::to_owned),
            hash_phrase: hash.map(str::to_owned),
            target_algorithm: PasswordProtectionAlgorithm::Sha2,
        }
    }

    #[test]
    fn compute_verifies_then_hashes() {
        let stored = security::encrypt_password(PasswordProtectionAlgorithm::Sha2, "uname-pass");
        let ok = compute(&job(Some("uname-pass"), Some("uname-new"), stored.clone()));
        assert!(ok.verified);
        assert_eq!(
            ok.hash,
            security::encrypt_password(PasswordProtectionAlgorithm::Sha2, "uname-new")
        );
        let bad = compute(&job(Some("wrong"), Some("uname-new"), stored.clone()));
        assert_eq!(
            bad,
            PasswordOutcome {
                verified: false,
                hash: None
            }
        );
        // no verification asked for: verified, and nothing hashed without a phrase
        let none = compute(&job(None, None, None));
        assert_eq!(
            none,
            PasswordOutcome {
                verified: true,
                hash: None
            }
        );
    }

    #[test]
    fn a_job_runs_off_the_loop_and_lands_in_order() {
        let mut w = World::new();
        set_enabled_for_testing(&mut w, true);
        let results = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let stored = security::encrypt_password(PasswordProtectionAlgorithm::Sha2, "u-pw");
        for (i, phrase) in ["u-pw", "u-bad", "u-pw"].into_iter().enumerate() {
            let r = results.clone();
            assert!(try_enqueue(
                &mut w,
                job(Some(phrase), None, stored.clone()),
                Box::new(move |_, _, o| r.borrow_mut().push((i, o.verified))),
            ));
        }
        let ctx = w.core.loop_context.clone();
        let end = std::time::Instant::now() + Duration::from_secs(10);
        while results.borrow().len() < 3 && std::time::Instant::now() < end {
            ctx.execute_tasks(&mut w);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(*results.borrow(), [(0, true), (1, false), (2, true)]);
        stop(&mut w);
        assert!(w.resources.get_mut::<PasswordWorker>().thread.is_none());
    }

    #[test]
    fn a_job_queued_in_a_save_runs_once_the_save_ends() {
        let mut w = World::new();
        set_enabled_for_testing(&mut w, true);
        w.set_state(WorldState::Saving);
        let done = std::rc::Rc::new(std::cell::Cell::new(false));
        let d = done.clone();
        assert!(try_enqueue(
            &mut w,
            job(None, None, None),
            Box::new(move |_, _, _| d.set(true))
        ));
        let ctx = w.core.loop_context.clone();
        std::thread::sleep(Duration::from_millis(150));
        ctx.execute_tasks(&mut w);
        assert!(!done.get(), "ran during the freeze");
        // the state flips back with no other enqueue or completion in between
        w.set_state(WorldState::Running);
        let end = std::time::Instant::now() + Duration::from_secs(10);
        while !done.get() && std::time::Instant::now() < end {
            ctx.execute_tasks(&mut w);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(done.get());
        stop(&mut w);
    }
}
