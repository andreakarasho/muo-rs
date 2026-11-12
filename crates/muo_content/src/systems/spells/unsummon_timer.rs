//! The unsummon timer: deletes a summoned creature after a delay, one timer per creature.

use indexmap::IndexMap;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::timer::{self, TimerHandle};

/// `_timers`: tracked since some of them are really long and might hold references to long dead/deleted mobs.
#[derive(Default)]
pub struct UnsummonTimers {
    timers: IndexMap<MobileId, TimerHandle>,
}

/// `UnsummonTimer.StopTimer(creature)`.
pub fn stop_timer(w: &mut World, creature: MobileId) {
    if let Some(mut t) = w
        .resources
        .get_mut::<UnsummonTimers>()
        .timers
        .shift_remove(&creature)
    {
        t.cancel(w);
    }
}

/// `new UnsummonTimer(creature, delay, onUnsummon).Start()`: replaces the creature's earlier timer.
pub fn start(
    w: &mut World,
    creature: MobileId,
    delay: TimeSpan,
    on_unsummon: Option<fn(&mut World)>,
) {
    if let Some(mut old) = w
        .resources
        .get_mut::<UnsummonTimers>()
        .timers
        .shift_remove(&creature)
    {
        old.cancel(w);
    }
    let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        // BaseCreature.OnAfterDelete will remove the creature from the timers table
        if let Some(ty) = w.mobile(creature).map(|m| m.ty) {
            (ty.delete)(w, creature);
        }
        w.resources
            .get_mut::<UnsummonTimers>()
            .timers
            .shift_remove(&creature);
        if let Some(f) = on_unsummon {
            f(w);
        }
    });
    w.resources
        .get_mut::<UnsummonTimers>()
        .timers
        .insert(creature, h);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_timer_per_creature_and_stop_cancels() {
        let mut w = World::new();
        let c = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        start(&mut w, c, TimeSpan::from_secs(10), None);
        let first = *w
            .resources
            .get_mut::<UnsummonTimers>()
            .timers
            .get(&c)
            .unwrap();
        assert!(first.running(&w));
        start(&mut w, c, TimeSpan::from_secs(20), None);
        assert!(!first.running(&w), "the earlier timer is stopped");
        assert_eq!(w.resources.get_mut::<UnsummonTimers>().timers.len(), 1);
        stop_timer(&mut w, c);
        assert!(w.resources.get_mut::<UnsummonTimers>().timers.is_empty());
    }
}
