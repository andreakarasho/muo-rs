//! Creature events: the two generated events (`CreatureDeathEvent`, `CreatureDeletedEvent`) as
//! lists of handlers. `[OnEvent(...)]` attributes become `subscribe_*` calls in a `register`.

use muo_server::ids::MobileId;
use muo_server::world::World;

type Handler = fn(&mut World, MobileId);

#[derive(Default)]
pub struct CreatureEvents {
    death: Vec<Handler>,
    deleted: Vec<Handler>,
}

pub fn subscribe_death(w: &mut World, h: Handler) {
    w.resources.get_mut::<CreatureEvents>().death.push(h);
}

pub fn subscribe_deleted(w: &mut World, h: Handler) {
    w.resources.get_mut::<CreatureEvents>().deleted.push(h);
}

/// (death, deleted) subscriber counts, for the orphan tests.
pub fn counts(w: &World) -> (usize, usize) {
    w.resources
        .get::<CreatureEvents>()
        .map_or((0, 0), |e| (e.death.len(), e.deleted.len()))
}

/// `CreatureEvents.CreatureDeathEvent(bc)`.
pub fn creature_death_event(w: &mut World, bc: MobileId) {
    let hs = w
        .resources
        .get::<CreatureEvents>()
        .map(|e| e.death.clone())
        .unwrap_or_default();
    for h in hs {
        h(w, bc);
    }
}

/// `CreatureEvents.CreatureDeletedEvent(bc)`.
pub fn creature_deleted_event(w: &mut World, bc: MobileId) {
    let hs = w
        .resources
        .get::<CreatureEvents>()
        .map(|e| e.deleted.clone())
        .unwrap_or_default();
    for h in hs {
        h(w, bc);
    }
}
