//! Events (`[GeneratedEvent]` / `[OnEvent]`): a list of function pointers per
//! argument type, held in the world's resources, called in subscription order. Subscribe in a `Configure` or
//! `Initialize` function; the content crates do not need closures, so these are plain `fn`s.

use muo_server::World;

type Ranked<A> = (u32, fn(&mut World, &mut A));

pub struct Handlers<A: 'static>(Vec<Ranked<A>>);

impl<A: 'static> Default for Handlers<A> {
    fn default() -> Self {
        Handlers(Vec::new())
    }
}

/// `[OnEvent(nameof(X))]`.
pub fn subscribe<A: 'static>(w: &mut World, f: fn(&mut World, &mut A)) {
    subscribe_ranked(w, u32::MAX, f);
}

/// `[OnEvent]` with the handler's rank in the event's handler list (handlers run in a fixed order that the crates'
/// initialize order does not follow): lower ranks run first, equal ranks in subscription order.
pub fn subscribe_ranked<A: 'static>(w: &mut World, rank: u32, f: fn(&mut World, &mut A)) {
    let list = &mut w.resources.get_mut::<Handlers<A>>().0;
    let at = list.partition_point(|e| e.0 <= rank);
    list.insert(at, (rank, f));
}

/// How many `[OnEvent]` handlers the event with argument type `A` has (the orphan tests count them).
pub fn count<A: 'static>(w: &World) -> usize {
    w.resources.get::<Handlers<A>>().map_or(0, |h| h.0.len())
}

/// The generated `X(args)` method: calls every subscriber.
pub fn invoke<A: 'static>(w: &mut World, args: &mut A) {
    let list: Vec<_> = w
        .resources
        .get_mut::<Handlers<A>>()
        .0
        .iter()
        .map(|e| e.1)
        .collect();
    for f in list {
        f(w, args);
    }
}

/// `PlayerMobile.PlayerLoginEvent(pm)` (declared by PlayerMobile; the login completes with it). Subscribe with
/// `events::subscribe::<PlayerLogin>(w, f)`, raise with [`player_login_event`].
pub struct PlayerLogin(pub muo_server::ids::MobileId);

pub fn player_login_event(w: &mut World, pm: muo_server::ids::MobileId) {
    invoke(w, &mut PlayerLogin(pm));
}
