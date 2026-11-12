//! The game loop (`Core`: tick calculation, `Now`/`TickCount`, `RunEventLoop`, idle sleep health, kill/closing)
//! and the loop's task queue (`EventLoopContext`).
//!
//! `Core.TickCount`/`Core.Now` are `World::tick_count`/`World::now`, set at the top of every iteration; everything
//! else `Core` kept in statics is [`CoreLoop`] (`World::core`). What the loop calls into but another layer owns (delta
//! queues, `NetState.Slice`, world snapshots) is behind [`LoopHooks`].
//!
//! Bootstrap (`setup`, `run`). The network start and `HandleClosed`'s network/ban shutdown are `muo`'s wrapper of
//! the hooks. Assembly loading is the content crate's registration; `server.eventLoopIdleWaitMs` and the late-wake
//! settings are plain fields. `EventLoopProfiler` is skipped (diagnostics). `EventLoopContext.Send`
//! and the `SynchronizationContext` role are dropped with async. `Timer.CheckTimerPool` is the skipped pool.

use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use muo_primitives::DateTime;

use crate::events::{self, ServerCrashedEventArgs};
use crate::timer::{self, TICK_RATE};
use crate::world::World;

// ---- Event loop tasks ------------------------------------------------------------------------------------------

pub type LoopTask = Box<dyn FnOnce(&mut World) + Send>;

#[derive(Default)]
struct Queues {
    normal: Mutex<VecDeque<LoopTask>>,
    priority: Mutex<VecDeque<LoopTask>>,
    waker: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

/// `EventLoopContext`: work posted from any thread that runs on the game loop. Clone it to hand to other threads.
#[derive(Clone)]
pub struct EventLoopContext {
    queues: Arc<Queues>,
    main_thread: ThreadId,
    max_per_frame: usize,
}

impl EventLoopContext {
    /// Created on the loop thread; `max_per_frame` bounds the normal queue per `execute_tasks` (usually 128).
    pub fn new(max_per_frame: usize) -> Self {
        Self {
            queues: Arc::default(),
            main_thread: thread::current().id(),
            max_per_frame,
        }
    }

    /// What `NetState.Wake()` does: nudge a loop asleep in network I/O.
    pub fn set_waker(&self, wake: impl Fn() + Send + Sync + 'static) {
        *self.queues.waker.lock().unwrap_or_else(|e| e.into_inner()) = Some(Arc::new(wake));
    }

    /// True when no callbacks are waiting. `execute_tasks` drains a bounded count, so the loop checks this before
    /// sleeping and a backlog keeps it running.
    pub fn is_empty(&self) -> bool {
        let lock =
            |q: &Mutex<VecDeque<LoopTask>>| q.lock().unwrap_or_else(|e| e.into_inner()).is_empty();
        lock(&self.queues.normal) && lock(&self.queues.priority)
    }

    pub fn post(&self, task: impl FnOnce(&mut World) + Send + 'static) {
        self.push(&self.queues.normal, Box::new(task));
    }

    /// `Post(d, Priority.High)`: every queued priority task runs each frame, ahead of the normal queue.
    pub fn post_priority(&self, task: impl FnOnce(&mut World) + Send + 'static) {
        self.push(&self.queues.priority, Box::new(task));
    }

    fn push(&self, q: &Mutex<VecDeque<LoopTask>>, task: LoopTask) {
        q.lock().unwrap_or_else(|e| e.into_inner()).push_back(task);
        self.wake();
    }

    /// A post from the loop thread cannot need a wake: the loop is executing this very call.
    pub fn wake(&self) {
        if thread::current().id() == self.main_thread {
            return;
        }
        let waker = self
            .queues
            .waker
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if let Some(wake) = waker {
            wake();
        }
    }

    pub fn execute_tasks(&self, w: &mut World) {
        debug_assert_eq!(
            thread::current().id(),
            self.main_thread,
            "EventLoop.ExecuteTasks on incorrect thread"
        );
        let pop =
            |q: &Mutex<VecDeque<LoopTask>>| q.lock().unwrap_or_else(|e| e.into_inner()).pop_front();
        let len = |q: &Mutex<VecDeque<LoopTask>>| q.lock().unwrap_or_else(|e| e.into_inner()).len();

        for _ in 0..len(&self.queues.priority) {
            if let Some(task) = pop(&self.queues.priority) {
                task(w);
            }
        }
        for _ in 0..len(&self.queues.normal).min(self.max_per_frame) {
            if let Some(task) = pop(&self.queues.normal) {
                task(w);
            }
        }
    }
}

// ---- Core ------------------------------------------------------------------------------------------------------

const HEALTH_SAMPLE_INTERVAL_MS: i64 = 1000;
// Doubling: a fixed suspension oscillates forever on a persistently bad host, while doubling converges on "stop
// sleeping" yet still recovers from a transient.
const BACKOFF_BASE_MS: i64 = 5000;
const BACKOFF_MAX_MS: i64 = 120_000;
const BACKOFF_MAX_SHIFT: u32 = 5;
const BACKOFF_RESET_AFTER_CLEAN_MS: i64 = 60_000;
// Below this a backoff is still recoverable and not actionable, so it only logs at debug.
const WARN_AFTER_CONSECUTIVE_BACKOFFS: i32 = 3;

/// The parts of `Core` other threads touch: `Closing`, `Kill`, `RequestSnapshot`.
#[derive(Default)]
pub struct Signals {
    // shared: the console pump reads it as `Core.Closing`
    pub closing: Arc<AtomicBool>,
    perform_process_kill: AtomicBool,
    restart_on_kill: AtomicBool,
    crashed: AtomicBool,
    snapshot_path: Mutex<Option<PathBuf>>,
}

impl Signals {
    /// The flags of `Core.Kill(restart)`, for a signal handler holding no `World` (it then wakes the loop with a
    /// clone of `loop_context`).
    pub fn kill(&self, restart: bool) {
        self.restart_on_kill.store(restart, Ordering::SeqCst);
        self.perform_process_kill.store(true, Ordering::SeqCst);
    }
}

/// How [`run_event_loop`] ended; `DoKill` exits the process with it (`muo::main` exits the process with the code `run_with` returns).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LoopExit {
    pub restart: bool,
    pub crashed: bool,
}

/// What the loop calls into that lives in other layers. Defaults do nothing, so a loop with no network still runs.
pub trait LoopHooks {
    /// `Mobile.ProcessDeltaQueue()`.
    fn mobile_deltas(&mut self, _w: &mut World) {}
    /// `Item.ProcessDeltaQueue()`.
    fn item_deltas(&mut self, _w: &mut World) {}
    /// `NetState.Slice()`.
    fn network_slice(&mut self, _w: &mut World) {}
    /// `World.Snapshot(path)`.
    fn world_snapshot(&mut self, _w: &mut World, _path: &std::path::Path) {}
    /// `World.WaitForWriteCompletion()`.
    fn wait_for_write_completion(&mut self, _w: &mut World) {}
    /// `!Mobile.HasQueuedDeltas && !Item.HasQueuedDeltas && NetState.IsIdle`.
    fn is_idle(&mut self, _w: &mut World) -> bool {
        true
    }
    /// `NetState.WaitForCompletion(ms)`: blocks up to `ms` for network work.
    fn wait_for_completion(&mut self, _w: &mut World, ms: i32) {
        thread::sleep(Duration::from_millis(ms as u64));
    }
    /// `HandleClosed`'s `PingServer.Shutdown()`, `NetState.Shutdown()`, `BanChannel.Stop()`, `ConnectionFilters.Stop()`.
    fn shutdown(&mut self, _w: &mut World) {}
}

/// `Core`'s statics. `World::core`.
pub struct CoreLoop {
    epoch: Instant,
    first_tick: i64,
    pub signals: Arc<Signals>,
    pub loop_context: EventLoopContext,
    /// `server.eventLoopIdleWaitMs`; 0 disables idle sleeping.
    pub event_loop_idle_wait_ms: i32,
    /// `Core.BaseDirectory`: the folder of the executable; relative paths in the configuration resolve here.
    pub base_directory: PathBuf,
    /// `Core.Headless`: stdin is not a terminal.
    pub headless: bool,
    /// Test hook (golden_saves): the clock `init_clock` starts at, so anchored times load unshifted.
    pub fixed_start_now: Option<DateTime>,
    /// `server.restartExitCode`: 0 relaunches in-process.
    pub restart_exit_code: i32,
    /// A `Configure` function's thrown exception (a bad `Data/names.json`, `Configuration/bans.json`): startup stops with it.
    pub startup_error: Option<String>,
    /// `Core.Expansion`.
    pub expansion: Option<muo_data::expansion::Expansion>,
    /// `ObjectPropertyList.Enabled` (`opl.enable`, default `Core.AOS`; set by `setup`).
    pub opl_enabled: bool,
    /// `ServerFeatureFlags`.
    pub features: ServerFeatureFlags,
    /// `Core.ScriptItems` / `ScriptMobiles`: registered concrete classes, tallied by `verify_serialization`.
    pub script_items: usize,
    pub script_mobiles: usize,
    pub idle_sleep_unsupported: bool,
    pub late_wake_threshold: i32,
    pub late_wake_percent: i32,
    late_wakes: i32,
    sleep_attempts: i32,
    next_health_sample: i64,
    idle_sleep_suspended_until: i64,
    idle_sleep_backoffs: i64,
    consecutive_bad_samples: i32,
    consecutive_backoffs: i32,
    current_backoff_ms: i64,
    last_backoff_at: i64,
    logged_backoff_ceiling: bool,
}

impl Default for CoreLoop {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
            first_tick: 0,
            signals: Arc::default(),
            loop_context: EventLoopContext::new(128),
            event_loop_idle_wait_ms: 2,
            base_directory: PathBuf::new(),
            headless: false,
            fixed_start_now: None,
            restart_exit_code: 0,
            startup_error: None,
            expansion: None,
            opl_enabled: false,
            features: ServerFeatureFlags::default(),
            script_items: 0,
            script_mobiles: 0,
            idle_sleep_unsupported: false,
            late_wake_threshold: 1,
            late_wake_percent: 10,
            late_wakes: 0,
            sleep_attempts: 0,
            next_health_sample: 0,
            idle_sleep_suspended_until: 0,
            idle_sleep_backoffs: 0,
            consecutive_bad_samples: 0,
            consecutive_backoffs: 0,
            current_backoff_ms: BACKOFF_BASE_MS,
            last_backoff_at: 0,
            logged_backoff_ceiling: false,
        }
    }
}

/// `ServerFeatureFlags`.
#[derive(Copy, Clone, Debug)]
pub struct ServerFeatureFlags {
    pub player_trading: bool,
    pub pvp_combat: bool,
    pub bank_access: bool,
    pub speedhack_detection: bool,
    pub insurance_enabled: bool,
}

impl Default for ServerFeatureFlags {
    fn default() -> Self {
        Self {
            player_trading: true,
            pvp_combat: true,
            bank_access: true,
            speedhack_detection: false,
            insurance_enabled: false,
        }
    }
}

impl World {
    /// `Core.Expansion`: the one source of the expansion (`core.expansion`). ML before the configuration prompts
    /// ran.
    pub fn expansion(&self) -> muo_data::expansion::Expansion {
        self.core
            .expansion
            .unwrap_or(muo_data::expansion::Expansion::ML)
    }
    /// `Core.AOS`.
    pub fn core_aos(&self) -> bool {
        self.expansion() >= muo_data::expansion::Expansion::AOS
    }
    /// `Core.SE`.
    pub fn core_se(&self) -> bool {
        self.expansion() >= muo_data::expansion::Expansion::SE
    }
    /// `Core.ML`.
    pub fn core_ml(&self) -> bool {
        self.expansion() >= muo_data::expansion::Expansion::ML
    }
    /// `Core.TOL`.
    pub fn core_tol(&self) -> bool {
        self.expansion() >= muo_data::expansion::Expansion::TOL
    }
    /// `ExpansionInfo.Table`, stored by `setup`.
    pub fn expansion_table(&self) -> Option<&muo_data::expansion::ExpansionTable> {
        self.resources.get::<muo_data::expansion::ExpansionTable>()
    }
    /// `ExpansionInfo.CoreExpansion`.
    pub fn core_expansion_info(&self) -> Option<&muo_data::expansion::ExpansionInfo> {
        self.expansion_table()
            .map(|t| t.get_info(self.expansion().index()))
    }
}

impl CoreLoop {
    /// `Core.GetTimestamp()`: milliseconds on a monotonic clock. (The clock starts near zero, not at the OS counter;
    /// this one starts near zero. Callers still compare by subtraction.)
    pub fn get_timestamp(&self) -> i64 {
        self.epoch.elapsed().as_millis() as i64
    }

    /// `Core.Closing`.
    pub fn closing(&self) -> bool {
        self.signals.closing.load(Ordering::SeqCst)
    }

    /// `Core.Kill(restart)`: asks the loop to stop after this iteration. Callable from any thread through `signals`.
    pub fn kill(&self, restart: bool) {
        self.signals.kill(restart);
        self.loop_context.wake();
    }

    /// `Core.RequestSnapshot(path)`.
    pub fn request_snapshot(&self, path: PathBuf) {
        *self
            .signals
            .snapshot_path
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(path);
        self.loop_context.wake();
    }

    /// `Core.Uptime`.
    pub fn uptime(&self, tick_count: i64) -> i64 {
        tick_count.wrapping_sub(self.first_tick)
    }

    /// `Core.IdleSleepSuspended`.
    pub fn idle_sleep_suspended(&self, tick_count: i64) -> bool {
        tick_count.wrapping_sub(self.idle_sleep_suspended_until) < 0
    }
}

/// The clock/timer seeding of `Core.Setup` that precedes `Configure`: `Now`, `TickCount`, the first tick, health
/// deadlines seeded from a real tick, and `Timer.Init`.
pub fn init_clock(w: &mut World) {
    w.now = w.core.fixed_start_now.unwrap_or_else(DateTime::utc_now);
    let tick = w.core.get_timestamp();
    w.tick_count = tick;
    w.core.first_tick = tick;
    w.core.next_health_sample = tick.wrapping_add(HEALTH_SAMPLE_INTERVAL_MS);
    w.core.idle_sleep_suspended_until = tick;
    w.timers.init(tick);
}

/// Once a second, suspends idle sleeping (with escalating duration) if the host keeps returning idle waits a full
/// tick or more late.
fn check_scheduler_health(w: &mut World) {
    let tick = w.tick_count;
    let c = &mut w.core;
    if tick.wrapping_sub(c.next_health_sample) < 0 {
        return;
    }
    c.next_health_sample = tick.wrapping_add(HEALTH_SAMPLE_INTERVAL_MS);

    let (late, sleeps) = (c.late_wakes, c.sleep_attempts);
    c.late_wakes = 0;
    c.sleep_attempts = 0;

    // A clean streak resets the escalation. Gated on the count rather than a sentinel: ticks may be negative.
    if c.consecutive_backoffs > 0
        && tick.wrapping_sub(c.last_backoff_at) > BACKOFF_RESET_AFTER_CLEAN_MS
    {
        if c.consecutive_backoffs >= WARN_AFTER_CONSECUTIVE_BACKOFFS {
            tracing::info!(
                "This host has returned idle waits on time for {BACKOFF_RESET_AFTER_CLEAN_MS}ms; idle sleeping is back to normal"
            );
        }
        c.consecutive_backoffs = 0;
        c.logged_backoff_ceiling = false;
    }

    if late <= c.late_wake_threshold {
        c.consecutive_bad_samples = 0;
        return;
    }

    // Lateness is a rate: an idle loop sleeps hundreds of times a second, so a few outliers are normal.
    if late * 100 < sleeps * c.late_wake_percent {
        c.consecutive_bad_samples = 0;
        return;
    }

    // Require persistence: any host can drop one sample to unrelated load, but an oversubscribed one stays bad.
    c.consecutive_bad_samples += 1;
    if c.consecutive_bad_samples < 2 {
        return;
    }

    if c.event_loop_idle_wait_ms <= 0 {
        return;
    }

    c.current_backoff_ms = (BACKOFF_BASE_MS
        << (c.consecutive_backoffs as u32).min(BACKOFF_MAX_SHIFT))
    .min(BACKOFF_MAX_MS);
    c.consecutive_backoffs += 1;
    c.last_backoff_at = tick;
    c.idle_sleep_suspended_until = tick.wrapping_add(c.current_backoff_ms);
    c.idle_sleep_backoffs += 1;

    let (wait, backoff_ms) = (c.event_loop_idle_wait_ms, c.current_backoff_ms);
    if backoff_ms >= BACKOFF_MAX_MS {
        // Escalation has run out of room; say so once.
        if !c.logged_backoff_ceiling {
            c.logged_backoff_ceiling = true;
            tracing::error!(
                "This host keeps returning idle waits late and sleeping has backed off {} times. The process is not \
                 being scheduled promptly, which is typical of shared or burstable vCPUs. Set \
                 server.eventLoopIdleWaitMs to 0 to disable sleeping permanently and trade a full core for latency.",
                c.idle_sleep_backoffs
            );
        }
        return;
    }

    if c.consecutive_backoffs < WARN_AFTER_CONSECUTIVE_BACKOFFS {
        tracing::debug!(
            "This host returned a {wait}ms idle wait at least {TICK_RATE}ms late {late} of {sleeps} time(s) in the \
             last second; idle sleeping suspended for {backoff_ms}ms"
        );
        return;
    }

    tracing::warn!(
        "This host returned a {wait}ms idle wait at least {TICK_RATE}ms late {late} of {sleeps} time(s) in the last \
         second, for the {}th time running; idle sleeping suspended for {backoff_ms}ms",
        c.consecutive_backoffs
    );
}

/// One pass of `RunEventLoop`'s body with `w.tick_count`/`w.now` already set (tests set them). Returns true when a
/// kill was requested and the loop must stop.
pub fn run_iteration(w: &mut World, hooks: &mut dyn LoopHooks) -> bool {
    hooks.mobile_deltas(w);
    hooks.item_deltas(w);

    timer::slice(w);

    hooks.network_slice(w); // Handle networking

    // Execute captured post-await methods (like Timer.Pause)
    let ctx = w.core.loop_context.clone();
    ctx.execute_tasks(w);

    let snapshot = w
        .core
        .signals
        .snapshot_path
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();
    if let Some(path) = snapshot {
        hooks.world_snapshot(w, &path);
    }

    if w.core.signals.perform_process_kill.load(Ordering::SeqCst) {
        hooks.wait_for_write_completion(w);
        return true;
    }

    check_scheduler_health(w);

    let tick = w.tick_count;
    if w.core.event_loop_idle_wait_ms > 0
        && tick.wrapping_sub(w.core.idle_sleep_suspended_until) >= 0
        && !crate::mobile::part2::stats::has_queued_deltas(w)
        && !crate::item::ops::has_queued_deltas(w)
        && hooks.is_idle(w)
        && ctx.is_empty()
    {
        // Re-read the clock: a stale timestamp overstates the time to the next tick and sleeps straight past it.
        let start = w.core.get_timestamp();
        let due = w.timers.milliseconds_until_next_tick(start);
        if due > 0 {
            let requested = due.min(i64::from(w.core.event_loop_idle_wait_ms)) as i32;
            hooks.wait_for_completion(w, requested);
            let elapsed = w.core.get_timestamp().wrapping_sub(start);
            w.core.sleep_attempts += 1;
            if elapsed - i64::from(requested) >= TICK_RATE {
                w.core.late_wakes += 1;
            }
        }
    }
    false
}

/// `Core.RunEventLoop()`: reads the clock each iteration, then runs [`run_iteration`] until closing or killed.
/// A panic is an unhandled crash: `ServerCrashed` is raised, then the loop ends.
pub fn run_event_loop(w: &mut World, hooks: &mut dyn LoopHooks) -> LoopExit {
    let result = catch_unwind(AssertUnwindSafe(|| {
        while !w.core.closing() {
            w.tick_count = w.core.get_timestamp();
            w.now = DateTime::utc_now();
            if run_iteration(w, hooks) {
                break;
            }
        }
    }));

    if let Err(payload) = result {
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "panic".to_string());
        handle_unhandled_exception(w, hooks, message);
        return LoopExit {
            restart: w.core.signals.restart_on_kill.load(Ordering::SeqCst),
            crashed: true,
        };
    }

    let restart = w.core.signals.restart_on_kill.load(Ordering::SeqCst);
    do_kill(w, hooks);
    LoopExit {
        restart,
        crashed: false,
    }
}

/// `CurrentDomain_UnhandledException` for a terminating exception.
fn handle_unhandled_exception(w: &mut World, hooks: &mut dyn LoopHooks, message: String) {
    eprintln!("Error:\n{message}");
    w.core.signals.crashed.store(true, Ordering::SeqCst);
    let mut args = ServerCrashedEventArgs {
        exception: message,
        close: false,
    };
    // A panic in a crash handler is swallowed.
    let _ = catch_unwind(AssertUnwindSafe(|| {
        events::invoke_server_crashed(w, &mut args)
    }));
    if !args.close && !w.core.headless {
        println!("This exception is fatal, press return to exit");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
    do_kill(w, hooks);
}

/// `Core.DoKill`'s part before the process exits.
fn do_kill(w: &mut World, hooks: &mut dyn LoopHooks) {
    if w.core.closing() {
        return;
    }
    handle_closed(w, hooks);
}

/// `HandleClosed`.
pub fn handle_closed(w: &mut World, hooks: &mut dyn LoopHooks) {
    w.core.signals.closing.store(true, Ordering::SeqCst);
    tracing::info!("Shutting down");
    w.wait_for_write_completion();
    // A threaded writer would stage and publish the snapshot before releasing `_diskWriteHandle`; here
    // the publish is posted to the loop, so run what the writer just posted or the last save stays in `Saves.next`
    // (recovered, with `Saves` set aside, on the next boot).
    let ctx = w.core.loop_context.clone();
    ctx.execute_tasks(w);
    hooks.shutdown(w);
    if !w.core.signals.crashed.load(Ordering::SeqCst) {
        events::invoke_shutdown(w, &mut ());
    }
}

// ---- Bootstrap -------------------------------------------------------------------------------------------------

use crate::persistence::{LoadPrompt, PersistError, StdioPrompt};
use crate::type_registry::{Phase, TypeRegistry, invoke_phase};

/// `Core.VerifySerialization`: tallies `ScriptItems`/`ScriptMobiles`. The reflection checks (serialization
/// constructor, `Serialize`/`Deserialize` methods) hold by construction of `ItemType`/`MobileType`.
pub fn verify_serialization(w: &mut World) {
    let (items, mobiles) = w.types.counts();
    w.core.script_items = items;
    w.core.script_mobiles = mobiles;
}

/// What `muo` loops with: the pieces of the loop that live in other layers.
pub struct ServerHooks;

impl LoopHooks for ServerHooks {
    fn mobile_deltas(&mut self, w: &mut World) {
        crate::mobile::part2::stats::process_delta_queue(w);
    }
    fn item_deltas(&mut self, w: &mut World) {
        crate::item::ops::process_delta_queue(w);
    }
    fn world_snapshot(&mut self, w: &mut World, path: &std::path::Path) {
        w.snapshot(path);
    }
    fn wait_for_completion(&mut self, w: &mut World, ms: i32) {
        // shortcut: these hooks own no network; `muo`'s wrapper calls `Network::wait_for_completion` (wakes on socket
        // activity) and only reaches here without one, where sleeping is all there is to wake for.
        let _ = w;
        thread::sleep(Duration::from_millis(ms as u64));
    }
    fn wait_for_write_completion(&mut self, w: &mut World) {
        w.wait_for_write_completion();
    }
}

/// `Core.Setup` up to the event loop. `register` is the content crate's registration into the type registry
/// (classes and `ConfigurePrompts`/`Configure`/`Initialize` functions); it replaces loading `assemblies.json`.
/// An `Err` is an exception escaping `Main`.
pub fn setup(
    w: &mut World,
    base_dir: PathBuf,
    register: impl FnOnce(&mut TypeRegistry),
    prompt: &mut dyn LoadPrompt,
) -> Result<(), PersistError> {
    setup_with(w, base_dir, register, prompt, |_| {})
}

/// [`setup`] with `start_network`, which starts `BanChannel`/`NetState`/`PingServer` (after `Initialize`,
/// before `ServerStarted`). The network is owned beside the world, so the caller builds it there.
pub fn setup_with(
    w: &mut World,
    base_dir: PathBuf,
    register: impl FnOnce(&mut TypeRegistry),
    prompt: &mut dyn LoadPrompt,
    start_network: impl FnOnce(&mut World),
) -> Result<(), PersistError> {
    w.core.base_directory = base_dir.clone();
    // Headless = stdin is not a TTY
    w.core.headless = {
        use std::io::IsTerminal;
        !std::io::stdin().is_terminal()
    };
    if w.core.headless {
        tracing::info!(
            "Headless mode detected (stdin is not a TTY); interactive console input is disabled."
        );
    }

    w.config = muo_data::configuration::ServerConfiguration::load(&base_dir, false)
        .map_err(|e| PersistError::Failed(e.to_string()))?;

    // 0 disables idle sleeping entirely (full-core spin, zero scheduling overhead).
    let idle = w.config.get_setting("server.eventLoopIdleWaitMs", 2_i32);
    if idle < 0 {
        tracing::warn!(
            "server.eventLoopIdleWaitMs {idle} is negative; using 0 (idle sleeping disabled)"
        );
    }
    w.core.event_loop_idle_wait_ms = idle.max(0);

    let threshold = w.config.get_setting("server.lateWakeThreshold", 1_i32);
    if threshold < 0 {
        tracing::warn!("server.lateWakeThreshold {threshold} is negative; using 0");
    }
    w.core.late_wake_threshold = threshold.max(0);

    let percent = w.config.get_setting("server.lateWakePercent", 10_i32);
    if !(0..=100).contains(&percent) {
        tracing::warn!(
            "server.lateWakePercent {percent} is outside 0-100; using {}",
            percent.clamp(0, 100)
        );
    }
    w.core.late_wake_percent = percent.clamp(0, 100);

    // Unix truncates exit codes to a byte, so anything outside 1-255 cannot be told apart reliably.
    let mut code = w.config.get_setting("server.restartExitCode", 0_i32);
    if !(0..=255).contains(&code) {
        tracing::warn!(
            "server.restartExitCode {code} is outside 1-255; restarts will relaunch the server directly"
        );
        code = 0;
    }
    w.core.restart_exit_code = code;

    // assemblies.json / LoadAssemblies: content is linked in; it registers itself here.
    crate::server_setup::register(&mut w.types);
    register(&mut w.types);

    // First-boot interactive setup, before the first log line of a normal boot.
    let mut io = muo_data::configuration::StdioPrompt;
    let mut expansions = muo_data::expansion::ExpansionTable::load(&base_dir)
        .map_err(|e| PersistError::Failed(e.to_string()))?;
    let mut client = muo_data::client::UoClient::default();
    w.core.expansion = w
        .config
        .configure_prompts(&mut io, &base_dir, &mut expansions, &mut client)
        .map_err(|e| PersistError::Failed(e.to_string()))?
        .or(w.core.expansion);
    w.resources.insert(expansions);
    w.resources.insert(client);
    // ExpansionConfiguration.Configure: `ObjectPropertyList.Enabled = GetSetting("opl.enable", Core.AOS)`
    let aos = w.core_aos();
    w.core.opl_enabled = w.config.get_setting("opl.enable", aos);
    invoke_phase(w, Phase::ConfigurePrompts);

    verify_serialization(w);
    init_clock(w);

    w.configure_paths();
    invoke_phase(w, Phase::Configure);
    if let Some(e) = w.core.startup_error.take() {
        return Err(PersistError::Failed(e));
    }

    crate::server_setup::load_body_table(w);
    crate::server_setup::load_tile_matrix(w);
    let expansion = w.expansion();
    crate::region_json::load_regions(w, &base_dir, expansion).map_err(PersistError::Failed)?;

    w.load(prompt)?;

    invoke_phase(w, Phase::Initialize);

    start_network(w);
    events::invoke_server_started(w, &mut ());
    Ok(())
}

/// `Application.Main` after `Core.Setup`: runs the loop and returns the process exit code.
pub fn run(w: &mut World) -> i32 {
    run_with(w, &mut ServerHooks)
}

/// [`run`] with the caller's hooks (the network lives beside the world, so `muo` wraps [`ServerHooks`]).
pub fn run_with(w: &mut World, hooks: &mut dyn LoopHooks) -> i32 {
    let exit = run_event_loop(w, hooks);
    w.save_worker.wait_idle();
    if exit.restart && w.core.restart_exit_code != 0 {
        tracing::info!(
            "Restarting (exiting with code {} for the supervisor)",
            w.core.restart_exit_code
        );
        return w.core.restart_exit_code;
    }
    if exit.restart {
        tracing::info!("Restarting");
        match std::env::current_exe().and_then(|exe| std::process::Command::new(exe).spawn()) {
            Ok(_) => tracing::info!("Restart done"),
            Err(e) => tracing::error!("Restart failed: {e}"),
        }
    }
    0 // DoKill ends with Environment.Exit(0), a crash included
}

/// The default boot prompt: stdin/stdout.
pub fn default_prompt() -> StdioPrompt {
    StdioPrompt::new()
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::atomic::AtomicUsize;

    use muo_primitives::TimeSpan;

    use super::*;

    struct Order(Rc<std::cell::RefCell<Vec<&'static str>>>);

    impl LoopHooks for Order {
        fn mobile_deltas(&mut self, _: &mut World) {
            self.0.borrow_mut().push("mobiles");
        }
        fn item_deltas(&mut self, _: &mut World) {
            self.0.borrow_mut().push("items");
        }
        fn network_slice(&mut self, _: &mut World) {
            self.0.borrow_mut().push("net");
        }
    }

    #[test]
    fn slice_order_deltas_timers_network_tasks() {
        let mut w = World::new();
        w.core.event_loop_idle_wait_ms = 0;
        let log = Rc::new(std::cell::RefCell::new(Vec::new()));
        let l = log.clone();
        timer::start(
            &mut w,
            TimeSpan::from_millis(8),
            TimeSpan::ZERO,
            1,
            move |_| l.borrow_mut().push("timer"),
        );
        let done = Arc::new(AtomicUsize::new(0));
        let d = done.clone();
        w.core.loop_context.post(move |_| {
            d.fetch_add(1, Ordering::SeqCst);
        });
        w.tick_count = 8;
        assert!(!run_iteration(&mut w, &mut Order(log.clone())));
        assert_eq!(*log.borrow(), vec!["mobiles", "items", "timer", "net"]);
        assert_eq!(done.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn kill_stops_the_loop_and_shutdown_fires_once() {
        let mut w = World::new();
        w.core.event_loop_idle_wait_ms = 0;
        let shutdowns = Rc::new(Cell::new(0));
        let s = shutdowns.clone();
        w.events.shutdown.subscribe(move |_, _| s.set(s.get() + 1));
        w.core.kill(true);
        let exit = run_event_loop(&mut w, &mut Order(Rc::default()));
        assert_eq!(
            exit,
            LoopExit {
                restart: true,
                crashed: false
            }
        );
        assert!(w.core.closing());
        assert_eq!(shutdowns.get(), 1);
    }

    #[test]
    fn tick_count_wraps() {
        let mut w = World::new();
        w.timers.init(i64::MAX - 7);
        let fired = Rc::new(Cell::new(false));
        let f = fired.clone();
        timer::start(
            &mut w,
            TimeSpan::from_millis(16),
            TimeSpan::ZERO,
            1,
            move |_| f.set(true),
        );
        w.tick_count = i64::MAX.wrapping_add(17);
        timer::slice(&mut w);
        assert!(fired.get());
    }

    #[test]
    fn priority_tasks_run_before_normal_and_normal_is_bounded() {
        let ctx = EventLoopContext::new(2);
        let mut w = World::new();
        let log = Arc::new(Mutex::new(Vec::new()));
        for n in 0..3 {
            let l = log.clone();
            ctx.post(move |_| l.lock().unwrap().push(n));
        }
        let l = log.clone();
        ctx.post_priority(move |_| l.lock().unwrap().push(99));
        ctx.execute_tasks(&mut w);
        assert_eq!(*log.lock().unwrap(), vec![99, 0, 1]);
        assert!(!ctx.is_empty());
        ctx.execute_tasks(&mut w);
        assert!(ctx.is_empty());
    }
}
