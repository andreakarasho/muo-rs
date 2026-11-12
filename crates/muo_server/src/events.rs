//! Server events (`EventSink`, `AccountLoginEvent`, `AggressiveActionEvent`, `MovementEvent`, `ServerCrashedEvent`,
//! `SocketConnectionEvent`, `SpeechEvent`).
//!
//! `EventSink`'s static `event Action<T>`s are [`Event<A>`] fields of `World.events`: a list of callbacks called in
//! registration order with `&mut World` and `&mut A` (the event's argument, or its `EventArgs` object, which handlers
//! mutate: `Handled`, `Blocked`, `Accepted`...). The `Invoke*` methods keep their `EventSink` names in snake_case.
//! The `Create`/`Free` object pools of `MovementEventArgs` and `AggressiveActionEventArgs` are dropped.

use std::net::IpAddr;

use muo_primitives::{Direction, MessageType};

use crate::ids::MobileId;
use crate::world::World;

type Handler<A> = std::rc::Rc<dyn Fn(&mut World, &mut A)>;

/// Identifies one subscription (the delegate `-=` removes).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Subscription(u64);

struct Entry<A> {
    id: u64,
    handler: Handler<A>,
}

/// A multicast event: a raise runs a snapshot of the list, so a handler may re-raise its own event, and one
/// unsubscribed (or added) by a handler still runs (or not) in the raise in progress.
pub struct Event<A> {
    entries: Vec<Entry<A>>,
    next_id: u64,
}

impl<A> Default for Event<A> {
    fn default() -> Self {
        Event {
            entries: Vec::new(),
            next_id: 0,
        }
    }
}

impl<A> Event<A> {
    /// `EventSink.X += handler`.
    pub fn subscribe(&mut self, handler: impl Fn(&mut World, &mut A) + 'static) -> Subscription {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(Entry {
            id,
            handler: std::rc::Rc::new(handler),
        });
        Subscription(id)
    }

    /// `EventSink.X -= handler`.
    pub fn unsubscribe(&mut self, s: Subscription) {
        self.entries.retain(|e| e.id != s.0);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Raises the event `sel` picks out of `w.events`; handlers run in registration order.
pub fn invoke<A>(w: &mut World, sel: fn(&mut Events) -> &mut Event<A>, args: &mut A) {
    let snapshot: Vec<Handler<A>> = sel(&mut w.events)
        .entries
        .iter()
        .map(|e| e.handler.clone())
        .collect();
    for h in snapshot {
        h(w, args);
    }
}

/// `SpeechEventArgs`.
#[derive(Clone, Debug)]
pub struct SpeechEventArgs {
    pub mobile: MobileId,
    pub speech: String,
    pub message_type: MessageType,
    pub hue: i32,
    pub keywords: Vec<i32>,
    pub handled: bool,
    pub blocked: bool,
}

impl SpeechEventArgs {
    pub fn new(
        mobile: MobileId,
        speech: String,
        message_type: MessageType,
        hue: i32,
        keywords: Vec<i32>,
    ) -> Self {
        Self {
            mobile,
            speech,
            message_type,
            hue,
            keywords,
            handled: false,
            blocked: false,
        }
    }

    pub fn has_keyword(&self, keyword: i32) -> bool {
        self.keywords.contains(&keyword)
    }
}

/// `AccountLoginEventArgs`. // shortcut: `State` is the `NetState` id and `RejectReason` the raw `ALRReason` byte;
/// the handlers read and write the same values, so no byte or message differs.
#[derive(Clone, Debug)]
pub struct AccountLoginEventArgs {
    pub net_state: usize,
    pub username: String,
    pub password: String,
    pub accepted: bool,
    pub reject_reason: u8,
    /// No verdict yet: a subscriber moved the password check off the game loop and replies itself once it lands.
    /// The packet handler must send neither accept nor reject while this is set.
    pub deferred: bool,
}

impl AccountLoginEventArgs {
    pub fn new(net_state: usize, username: String, password: String) -> Self {
        Self {
            net_state,
            username,
            password,
            accepted: false,
            reject_reason: 0,
            deferred: false,
        }
    }
}

/// `AggressiveActionEventArgs`.
#[derive(Copy, Clone, Debug)]
pub struct AggressiveActionEventArgs {
    pub aggressed: MobileId,
    pub aggressor: MobileId,
    pub criminal: bool,
}

/// `MovementEventArgs`.
#[derive(Copy, Clone, Debug)]
pub struct MovementEventArgs {
    pub mobile: MobileId,
    pub direction: Direction,
    pub blocked: bool,
}

impl MovementEventArgs {
    pub fn new(mobile: MobileId, direction: Direction) -> Self {
        Self {
            mobile,
            direction,
            blocked: false,
        }
    }
}

/// `ServerCrashedEventArgs`: the exception is its message.
#[derive(Clone, Debug)]
pub struct ServerCrashedEventArgs {
    pub exception: String,
    pub close: bool,
}

/// `SocketConnectEventArgs`.
#[derive(Copy, Clone, Debug)]
pub struct SocketConnectEventArgs {
    pub address: IpAddr,
    pub allow_connection: bool,
}

impl SocketConnectEventArgs {
    pub fn new(address: IpAddr) -> Self {
        Self {
            address,
            allow_connection: true,
        }
    }
}

/// `EventSink`, grouped by area.
#[derive(Default)]
pub struct Events {
    pub shutdown: Event<()>,
    pub logout: Event<MobileId>,
    pub connected: Event<MobileId>,
    pub before_disconnected: Event<MobileId>,
    pub disconnected: Event<MobileId>,
    /// (beholder, beheld)
    pub paperdoll_request: Event<(MobileId, MobileId)>,
    pub server_started: Event<()>,
    pub account_login: Event<AccountLoginEventArgs>,
    pub aggressive_action: Event<AggressiveActionEventArgs>,
    pub movement: Event<MovementEventArgs>,
    pub server_crashed: Event<ServerCrashedEventArgs>,
    pub socket_connect: Event<SocketConnectEventArgs>,
    pub speech: Event<SpeechEventArgs>,
    /// Commands
    pub command: Event<crate::commands::CommandEventArgs>,
    /// World events
    pub world_load: Event<()>,
    pub world_save: Event<()>,
    pub world_save_post_snapshot: Event<WorldSavePostSnapshotEventArgs>,
}

/// `WorldSavePostSnapshotEventArgs`: the previous save folder and the staged new one.
#[derive(Clone, Debug)]
pub struct WorldSavePostSnapshotEventArgs {
    pub old_save_path: std::path::PathBuf,
    pub new_save_path: std::path::PathBuf,
}

macro_rules! invokers {
    ($($(#[$m:meta])* $name:ident => $field:ident($ty:ty)),* $(,)?) => {$(
        $(#[$m])*
        pub fn $name(w: &mut World, args: &mut $ty) {
            invoke(w, |e| &mut e.$field, args);
        }
    )*};
}

invokers! {
    invoke_shutdown => shutdown(()),
    invoke_logout => logout(MobileId),
    invoke_connected => connected(MobileId),
    invoke_before_disconnected => before_disconnected(MobileId),
    invoke_disconnected => disconnected(MobileId),
    /// `InvokePaperdollRequest(beholder, beheld)`.
    invoke_paperdoll_request => paperdoll_request((MobileId, MobileId)),
    invoke_server_started => server_started(()),
    invoke_account_login => account_login(AccountLoginEventArgs),
    invoke_aggressive_action => aggressive_action(AggressiveActionEventArgs),
    invoke_movement => movement(MovementEventArgs),
    invoke_server_crashed => server_crashed(ServerCrashedEventArgs),
    invoke_socket_connect => socket_connect(SocketConnectEventArgs),
    invoke_speech => speech(SpeechEventArgs),
    invoke_command => command(crate::commands::CommandEventArgs),
    invoke_world_load => world_load(()),
    invoke_world_save => world_save(()),
    /// `InvokeWorldSavePostSnapshot(oldSavePath, newSavePath)`.
    invoke_world_save_post_snapshot => world_save_post_snapshot(WorldSavePostSnapshotEventArgs),
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use muo_primitives::Serial;

    use super::*;

    #[test]
    fn handlers_run_in_registration_order_and_can_mutate_args() {
        let mut w = World::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        for n in 0..3 {
            let l = log.clone();
            w.events.movement.subscribe(move |_, e| {
                l.borrow_mut().push(n);
                if n == 1 {
                    e.blocked = true;
                }
            });
        }
        let mut e = MovementEventArgs::new(MobileId(Serial::new(1)), Direction::NORTH);
        invoke_movement(&mut w, &mut e);
        assert_eq!(*log.borrow(), vec![0, 1, 2]);
        assert!(e.blocked);
    }

    #[test]
    fn unsubscribe_and_subscribe_during_a_raise() {
        let mut w = World::new();
        let count = Rc::new(RefCell::new(0));
        let c = count.clone();
        let first = w.events.shutdown.subscribe(|w, _| {
            w.events
                .shutdown
                .subscribe(|_, _| panic!("added during the raise"));
        });
        w.events
            .shutdown
            .subscribe(move |_, _| *c.borrow_mut() += 1);
        invoke_shutdown(&mut w, &mut ());
        assert_eq!(*count.borrow(), 1);
        w.events.shutdown.unsubscribe(first);
        assert_eq!(w.events.shutdown.len(), 2);
    }
}
