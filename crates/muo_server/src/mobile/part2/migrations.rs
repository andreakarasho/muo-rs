//! Values read from old saves that moved to other systems. The
//! dictionaries are in [`Part2Statics`](super::statics::Part2Statics) and stay `None` until something is added.

use indexmap::{IndexMap, IndexSet};

use super::statics::Part2Statics;
use crate::ids::MobileId;
use crate::world::World;

/// `AddToStabledMigration(m, stabled)`: moves the `Stabled` set to `PlayerMobile` once it loads.
pub fn add_to_stabled_migration(w: &mut World, m: MobileId, stabled: IndexSet<MobileId>) {
    if !stabled.is_empty() {
        w.resources
            .get_mut::<Part2Statics>()
            .stable_migrations
            .get_or_insert_with(IndexMap::new)
            .insert(m, stabled);
    }
}

/// `AddToMurderMigration(m, shortTermMurders)`: to the murder system.
pub fn add_to_murder_migration(w: &mut World, m: MobileId, short_term_murders: i32) {
    if short_term_murders > 0 {
        w.resources
            .get_mut::<Part2Statics>()
            .murder_migrations
            .get_or_insert_with(IndexMap::new)
            .insert(m, short_term_murders);
    }
}

/// `AddToVirtueMigration(m, values)`: to the virtue system.
pub fn add_to_virtue_migration(w: &mut World, m: MobileId, values: [i32; 8]) {
    w.resources
        .get_mut::<Part2Statics>()
        .virtue_migrations
        .get_or_insert_with(IndexMap::new)
        .insert(m, values);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::MOBILE_BASE;

    #[test]
    fn only_meaningful_values_are_recorded() {
        let mut w = World::new();
        let a = w.new_mobile(&MOBILE_BASE);
        let b = w.new_mobile(&MOBILE_BASE);
        add_to_stabled_migration(&mut w, a, IndexSet::new());
        add_to_murder_migration(&mut w, a, 0);
        let s = w.resources.get_mut::<Part2Statics>();
        assert!(s.stable_migrations.is_none() && s.murder_migrations.is_none());

        add_to_stabled_migration(&mut w, a, IndexSet::from([b]));
        add_to_murder_migration(&mut w, a, 3);
        add_to_virtue_migration(&mut w, a, [1; 8]);
        let s = w.resources.get::<Part2Statics>().unwrap();
        assert_eq!(s.stable_migrations.as_ref().unwrap()[&a].len(), 1);
        assert_eq!(s.murder_migrations.as_ref().unwrap()[&a], 3);
        assert_eq!(s.virtue_migrations.as_ref().unwrap()[&a], [1; 8]);
    }
}
