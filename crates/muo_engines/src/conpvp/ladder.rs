//! The dueling ladder and preferences: the `LadderController` and `PreferencesController` items and the
//! tables they hold, and the `PreferencesGump`.
//!
//! `Ladder.Find` has an inverted test (`if (m_Table.TryGetValue(mob, out var entry))` makes a new entry when one
//! exists and returns null when none does), so no entry is ever created for a mobile that has none and every `Find` of
//! one that has replaces it with a fresh one (appended to `Entries`, experience 0). That is kept.

use std::collections::HashMap;

use muo_io::{GenericReader, GenericWriter};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

use super::DuelState;

// ---- Ladder -------------------------------------------------------------------------------------------------------

/// `LadderEntry`.
#[derive(Clone, Debug)]
pub struct LadderEntry {
    pub mobile: MobileId,
    pub experience: i32,
    pub wins: i32,
    pub losses: i32,
    /// Where the entry is in `Entries` now (`Index`).
    pub index: usize,
    /// Which entry it is, whatever its place (an entry is identified by it, not by position).
    pub uid: u32,
}

/// `Ladder`.
#[derive(Default, Debug)]
pub struct Ladder {
    pub entries: Vec<LadderEntry>,
    /// `m_Table`: the `uid` of the mobile's current entry.
    table: HashMap<MobileId, u32>,
    next_uid: u32,
}

const SHORT_LEVELS: [i32; 25] = [
    1, 2, 3, 3, 4, 4, 5, 5, 5, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 8, 9, 9, 9, 9, 9,
];
const BASE_XP: [i32; 10] = [0, 100, 200, 400, 600, 900, 1200, 1600, 2000, 2500];
const LOSS_FACTORS: [i32; 9] = [10, 11, 11, 25, 25, 43, 43, 67, 67];
/// `{ win, los }` for a level difference of -6..=+6.
const OFFSET_SCALAR: [[i32; 2]; 13] = [
    [175, 25],
    [165, 35],
    [155, 45],
    [145, 55],
    [130, 70],
    [115, 85],
    [100, 100],
    [90, 110],
    [80, 120],
    [70, 130],
    [60, 140],
    [50, 150],
    [40, 160],
];

/// `Ladder.GetLevel(xp)`.
pub fn get_level(xp: i32) -> i32 {
    if xp >= 22500 {
        return 50;
    }
    if xp >= 2500 {
        return 10 + (xp - 2500) / 500;
    }
    SHORT_LEVELS[(xp.max(0) / 100) as usize]
}

/// `Ladder.GetLevelInfo(level, out xpBase, out xpAdvance)`.
pub fn get_level_info(level: i32) -> (i32, i32) {
    if level >= 10 {
        (2500 + (level - 10) * 500, 500)
    } else {
        let b = BASE_XP[(level - 1) as usize];
        (b, BASE_XP[level as usize] - b)
    }
}

/// `Ladder.GetLossFactor(level)`.
pub fn get_loss_factor(level: i32) -> i32 {
    if level >= 10 {
        100
    } else {
        LOSS_FACTORS[(level - 1) as usize]
    }
}

/// `Ladder.GetOffsetScalar(ourLevel, theirLevel, win)`.
pub fn get_offset_scalar(our_level: i32, their_level: i32, win: bool) -> i32 {
    let x = our_level - their_level;
    if !(-6..=6).contains(&x) {
        return 0;
    }
    OFFSET_SCALAR[(x + 6) as usize][if win { 0 } else { 1 }]
}

/// `Ladder.GetExperienceGain(us, them, weWon)`.
pub fn get_experience_gain(
    us: Option<&LadderEntry>,
    them: Option<&LadderEntry>,
    we_won: bool,
) -> i32 {
    let (Some(us), Some(them)) = (us, them) else {
        return 0;
    };
    let our_level = get_level(us.experience);
    let their_level = get_level(them.experience);
    let scalar = get_offset_scalar(our_level, their_level, we_won);
    if scalar == 0 {
        return 0;
    }
    let mut xp = 25 * scalar;
    if !we_won {
        xp = xp * get_loss_factor(our_level) / 100;
    }
    xp /= 100;
    if xp <= 0 {
        xp = 1;
    }
    xp * if we_won { 1 } else { -1 }
}

impl Ladder {
    fn swap(&mut self, idx: usize, new_idx: usize) -> usize {
        self.entries.swap(idx, new_idx);
        self.entries[idx].index = idx;
        self.entries[new_idx].index = new_idx;
        new_idx
    }

    /// `CompareTo`: `(l?.m_Experience ?? 0) - m_Experience`.
    fn compare(a: &LadderEntry, b: &LadderEntry) -> i32 {
        b.experience - a.experience
    }

    /// The place in `entries` of the entry the table holds for `mob`.
    fn position(&self, mob: MobileId) -> Option<usize> {
        let uid = *self.table.get(&mob)?;
        self.entries.iter().position(|e| e.uid == uid)
    }

    fn push(&mut self, mobile: MobileId, experience: i32, wins: i32, losses: i32) {
        let index = self.entries.len();
        self.next_uid += 1;
        let uid = self.next_uid;
        self.table.insert(mobile, uid);
        self.entries.push(LadderEntry {
            mobile,
            experience,
            wins,
            losses,
            index,
            uid,
        });
    }

    /// `UpdateEntry(entry)`: the entry (by its `Index`) moves to where its experience puts it.
    pub fn update_entry(&mut self, entry_index: usize) {
        let mut index = entry_index;
        if index < self.entries.len() {
            while index >= 1 && Self::compare(&self.entries[index], &self.entries[index - 1]) < 0 {
                index = self.swap(index, index - 1);
            }
            while index + 1 < self.entries.len()
                && Self::compare(&self.entries[index], &self.entries[index + 1]) > 0
            {
                index = self.swap(index, index + 1);
            }
        }
    }

    /// `Ladder(IGenericReader)`.
    pub fn read(w: &World, r: &mut dyn GenericReader) -> muo_io::Result<Ladder> {
        let version = r.read_encoded_int()?;
        let mut l = Ladder::default();
        if version == 0 || version == 1 {
            let count = r.read_encoded_int()?;
            for _ in 0..count {
                let mobile = r.read_mobile(w)?;
                let experience = r.read_encoded_int()?;
                let wins = r.read_encoded_int()?;
                let losses = r.read_encoded_int()?;
                if let Some(mobile) = mobile {
                    l.push(mobile, experience, wins, losses);
                }
            }
            if version == 0 {
                // `Entries.Sort()`: by experience, high first.
                muo_primitives::dotnet::net_sort(&mut l.entries, |a, b| {
                    b.experience.cmp(&a.experience)
                });
                for (i, e) in l.entries.iter_mut().enumerate() {
                    e.index = i;
                }
            }
        }
        Ok(l)
    }

    /// `Serialize(writer)`.
    pub fn write(&self, w: &World, out: &mut dyn GenericWriter) {
        out.write_encoded_int(1); // version
        out.write_encoded_int(self.entries.len() as i32);
        for e in &self.entries {
            out.write_mobile(w, Some(e.mobile));
            out.write_encoded_int(e.experience);
            out.write_encoded_int(e.wins);
            out.write_encoded_int(e.losses);
        }
    }
}

/// The ladder `Ladder.Instance`: the controller item holding it.
pub fn instance(w: &World) -> Option<ItemId> {
    w.resources.get::<Singletons>()?.ladder
}

#[derive(Default)]
pub struct Singletons {
    pub ladder: Option<ItemId>,
    pub preferences: Option<ItemId>,
}

fn singletons(w: &mut World) -> &mut Singletons {
    w.resources.get_mut::<Singletons>()
}

fn ladder_of(w: &World, controller: ItemId) -> Option<&Ladder> {
    w.item(controller)?.ext.get::<Ladder>()
}

fn ladder_mut(w: &mut World, controller: ItemId) -> Option<&mut Ladder> {
    w.item_mut(controller)?.ext.get_mut::<Ladder>()
}

/// `Ladder.Find(mob)` (see the module comment): the fresh entry when `mob` had one.
pub fn find(w: &mut World, controller: ItemId, mob: MobileId) -> Option<LadderEntry> {
    let l = ladder_mut(w, controller)?;
    l.table.get(&mob)?;
    l.push(mob, 0, 0, 0);
    l.entries.last().cloned()
}

/// `Ladder.FindNoCreate(mob)`.
pub fn find_no_create(w: &World, controller: ItemId, mob: MobileId) -> Option<&LadderEntry> {
    let l = ladder_of(w, controller)?;
    l.entries.get(l.position(mob)?)
}

/// The entry `Find`/`FindNoCreate` returns for `mob` now, to change it.
pub fn entry_mut(w: &mut World, controller: ItemId, mob: MobileId) -> Option<&mut LadderEntry> {
    let l = ladder_mut(w, controller)?;
    let i = l.position(mob)?;
    l.entries.get_mut(i)
}

/// `ladder.UpdateEntry(entry)` for `mob`'s entry.
pub fn update_entry(w: &mut World, controller: ItemId, mob: MobileId) {
    if let Some(l) = ladder_mut(w, controller)
        && let Some(i) = l.position(mob)
    {
        l.update_entry(i);
    }
}

/// `LadderController`.
pub static LADDER_CONTROLLER: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.LadderController",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &LADDER_CONTROLLER, 0x1B7A);
        ops::set_visible(w, id, false);
        ops::set_movable(w, id, false);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(Ladder::default());
        }
        if singletons(w).ladder.is_none() {
            singletons(w).ladder = Some(id); // `Ladder.Instance ??= Ladder`
        }
        id
    }),
    default_name: |_, _| Some("ladder controller".to_owned()),
    delete: |w, id| {
        if instance(w) == Some(id) {
            singletons(w).ladder = None;
        }
        (ITEM_BASE.delete)(w, id)
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(1);
        if let Some(l) = ladder_of(w, id) {
            l.write(w, out);
        } else {
            Ladder::default().write(w, out);
        }
        out.write_bool(instance(w) == Some(id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        if version == 0 || version == 1 {
            let l = Ladder::read(w, r)?;
            if let Some(i) = w.item_mut(id) {
                i.ext.insert(l);
            }
            if version < 1 || r.read_bool()? {
                singletons(w).ladder = Some(id);
            }
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Preferences ----------------------------------------------------------------------------------------------------

/// `PreferencesEntry`.
#[derive(Clone, Debug, Default)]
pub struct PreferencesEntry {
    pub disliked: Vec<Option<String>>,
}

/// `Preferences`.
#[derive(Default, Debug)]
pub struct Preferences {
    /// Insertion order, which is also the save order.
    table: indexmap::IndexMap<MobileId, PreferencesEntry>,
}

impl Preferences {
    /// `Find(mob)`: created when missing.
    pub fn find(&mut self, mob: MobileId) -> &mut PreferencesEntry {
        self.table.entry(mob).or_default()
    }
}

/// `Preferences.Instance`: the controller item holding it.
pub fn preferences_instance(w: &World) -> Option<ItemId> {
    w.resources.get::<Singletons>()?.preferences
}

/// `Preferences.Instance.Find(mob)` as a copy of the disliked names.
pub fn disliked(w: &mut World, controller: ItemId, mob: MobileId) -> Vec<Option<String>> {
    w.item_mut(controller)
        .and_then(|i| i.ext.get_mut::<Preferences>())
        .map(|p| p.find(mob).disliked.clone())
        .unwrap_or_default()
}

/// `PreferencesController`.
pub static PREFERENCES_CONTROLLER: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.PreferencesController",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &PREFERENCES_CONTROLLER, 0x1B7A);
        ops::set_visible(w, id, false);
        ops::set_movable(w, id, false);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(Preferences::default());
        }
        if singletons(w).preferences.is_none() {
            singletons(w).preferences = Some(id);
        } else if let Some(ty) = w.item(id).map(|i| i.ty) {
            // `Delete()` only deletes when it is not the instance.
            (ty.delete)(w, id);
        }
        id
    }),
    default_name: |_, _| Some("preferences controller".to_owned()),
    delete: |w, id| {
        if preferences_instance(w) != Some(id) {
            (ITEM_BASE.delete)(w, id)
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_encoded_int(0); // Preferences.Serialize: version
        let table = w.item(id).and_then(|i| i.ext.get::<Preferences>());
        out.write_encoded_int(table.map_or(0, |p| p.table.len() as i32));
        if let Some(p) = table {
            for (m, e) in &p.table {
                out.write_mobile(w, Some(*m));
                out.write_encoded_int(e.disliked.len() as i32);
                for d in &e.disliked {
                    out.write_opt_string(d.as_deref());
                }
            }
        }
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        if version == 0 {
            let inner = r.read_encoded_int()?;
            let mut p = Preferences::default();
            if inner == 0 {
                let count = r.read_encoded_int()?;
                for _ in 0..count {
                    let mobile = r.read_mobile(w)?;
                    let n = r.read_encoded_int()?;
                    let mut e = PreferencesEntry::default();
                    for _ in 0..n {
                        e.disliked.push(r.read_string()?);
                    }
                    if let Some(m) = mobile {
                        p.table.insert(m, e);
                    }
                }
            }
            if let Some(i) = w.item_mut(id) {
                i.ext.insert(p);
            }
            singletons(w).preferences = Some(id);
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// Registers the item classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&LADDER_CONTROLLER, &[]);
    types.register_item(&PREFERENCES_CONTROLLER, &[]);
}

#[allow(dead_code)]
fn _unused(_: &DuelState) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pvp_ladder_levels_and_gain() {
        assert_eq!(get_level(0), 1);
        assert_eq!(get_level(250), 3);
        assert_eq!(get_level(2500), 10);
        assert_eq!(get_level(22500), 50);
        assert_eq!(get_level_info(10), (2500, 500));
        assert_eq!(get_level_info(3), (200, 200));
        assert_eq!(get_loss_factor(10), 100);
        let e = |xp| LadderEntry {
            mobile: MobileId(muo_primitives::Serial::new(0)),
            experience: xp,
            wins: 0,
            losses: 0,
            index: 0,
            uid: 0,
        };
        // Equal levels: 25 for a win; a loss is 25*100*10/100/100 = 2 in integer math (level 1 loss factor 10).
        assert_eq!(get_experience_gain(Some(&e(0)), Some(&e(0)), true), 25);
        assert_eq!(get_experience_gain(Some(&e(0)), Some(&e(0)), false), -2);
        assert_eq!(get_experience_gain(None, Some(&e(0)), true), 0);
        assert_eq!(
            get_experience_gain(Some(&e(0)), Some(&e(25000)), true),
            0,
            "levels too far apart"
        );
    }

    #[test]
    fn pvp_ladder_update_entry_sorts() {
        let mut l = Ladder::default();
        for (i, xp) in [500, 300, 100].into_iter().enumerate() {
            l.entries.push(LadderEntry {
                mobile: MobileId(muo_primitives::Serial::new(0)),
                experience: xp,
                wins: 0,
                losses: 0,
                index: i,
                uid: i as u32,
            });
        }
        l.entries[2].experience = 900;
        l.update_entry(2);
        assert_eq!(l.entries[0].experience, 900);
        assert_eq!(l.entries[0].index, 0);
        assert_eq!(l.entries[2].index, 2);
    }
}
