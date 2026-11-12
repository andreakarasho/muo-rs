//! SlayerName, SlayerEntry and SlayerGroup. The groups' type lists are in
//! `slayer_data.rs` (class names with their namespaces); a creature is
//! matched by walking its `MobileType` chain for the full name, like `Type.IsAssignableFrom`.
//!
//! The abyss group's contents depend on `Core.AOS` when the groups are built; `Core.AOS` does not
//! change after start-up, so both variants are tables and the era picks one.

use muo_server::World;
use muo_server::ids::MobileId;

use super::slayer_data::{GROUPS_AOS, GROUPS_PRE};
use crate::core;

int_enum! {
    /// `SlayerName`.
    pub enum SlayerName {
        None = 0, Silver = 1, OrcSlaying = 2, TrollSlaughter = 3, OgreTrashing = 4, Repond = 5, DragonSlaying = 6,
        Terathan = 7, SnakesBane = 8, LizardmanSlaughter = 9, ReptilianDeath = 10, DaemonDismissal = 11,
        GargoylesFoe = 12, BalronDamnation = 13, Exorcism = 14, Ophidian = 15, SpidersDeath = 16,
        ScorpionsBane = 17, ArachnidDoom = 18, FlameDousing = 19, WaterDissipation = 20, Vacuum = 21,
        ElementalHealth = 22, EarthShatter = 23, BloodDrinking = 24, SummerWind = 25, ElementalBan = 26, Fey = 27,
    }
}

/// `SlayerEntry`: a name and the creature classes it slays.
#[derive(Debug)]
pub struct SlayerEntry {
    pub name: SlayerName,
    pub types: &'static [&'static str],
}

/// `SlayerGroup`: the entries and the groups (by index into the group array) that oppose it.
#[derive(Debug)]
pub struct SlayerGroupData {
    pub opposition: &'static [usize],
    pub found_on: &'static [&'static str],
    pub super_: SlayerEntry,
    pub entries: &'static [SlayerEntry],
}

static AOS_TITLES: [i32; 27] = [
    1060479, 1060470, 1060480, 1060468, 1060472, 1060462, 1060478, 1060475, 1060467, 1060473,
    1060460, 1060466, 1017396, 1060461, 1060469, 1060477, 1060474, 1060458, 1060465, 1060481,
    1060457, 1060471, 1060463, 1060459, 1060476, 1060464, 1070855,
];

static OLD_TITLES: [i32; 27] = [
    1017384, 1017385, 1017386, 1017387, 1017388, 1017389, 1017390, 1017391, 1017392, 1017393,
    1017394, 1017395, 1017396, 1017397, 1017398, 1017399, 1017400, 1017401, 1017402, 1017403,
    1017404, 1017405, 1017406, 1017407, 1017408, 1017409, 1070855,
];

fn groups(w: &World) -> &'static [&'static SlayerGroupData; 7] {
    if core::aos(w) {
        &GROUPS_AOS
    } else {
        &GROUPS_PRE
    }
}

/// `SlayerGroup.GetEntryByName(name)` with the entry's group.
pub fn entry_by_name(
    w: &World,
    name: SlayerName,
) -> Option<(&'static SlayerEntry, &'static SlayerGroupData)> {
    if name == SlayerName::None {
        return None;
    }
    for g in groups(w) {
        if g.super_.name == name {
            return Some((&g.super_, g));
        }
        if let Some(e) = g.entries.iter().find(|e| e.name == name) {
            return Some((e, g));
        }
    }
    None
}

/// `SlayerEntry.Title`.
pub fn title(w: &World, entry: &SlayerEntry) -> i32 {
    let titles = if core::aos(w) {
        &AOS_TITLES
    } else {
        &OLD_TITLES
    };
    titles[entry.name as usize - 1]
}

/// `SlayerEntry.SlayerText(out articleAn)`: whether the title's English text starts with a vowel.
// shortcut: a fixed list instead of reading `Localization.GetText(Title)`; the shipped cliloc titles are fixed, so the
// answer can only differ for a modified cliloc file.
pub fn slayer_article_an(name: SlayerName) -> bool {
    use SlayerName::*;
    matches!(
        name,
        OrcSlaying
            | OgreTrashing
            | Exorcism
            | Ophidian
            | ArachnidDoom
            | ElementalBan
            | ElementalHealth
            | EarthShatter
    )
}

/// `m.GetType()` is `full_name` or a subclass of it.
fn mobile_is(w: &World, m: MobileId, full_name: &str) -> bool {
    let mut t = w.mobile(m).map(|x| x.ty);
    while let Some(c) = t {
        if c.full_name == full_name {
            return true;
        }
        t = c.parent;
    }
    false
}

/// `SlayerEntry.Slays(m)`.
pub fn slays(w: &World, entry: &SlayerEntry, m: MobileId) -> bool {
    entry.types.iter().any(|n| mobile_is(w, m, n))
}

/// `SlayerGroup.OppositionSuperSlays(m)` for the group `g`.
pub fn opposition_super_slays(w: &World, g: &SlayerGroupData, m: MobileId) -> bool {
    let all = groups(w);
    g.opposition.iter().any(|&i| slays(w, &all[i].super_, m))
}

/// `SlayerGroup.GetLootSlayerType(type)`: `type` is the creature's full type name.
pub fn loot_slayer_type(w: &mut World, full_name: &str) -> SlayerName {
    let all = groups(w);
    for g in all {
        if g.found_on.contains(&full_name) {
            let index = muo_primitives::utility::random(&mut w.rng, 1 + g.entries.len() as i32);
            return if index == 0 {
                g.super_.name
            } else {
                g.entries[index as usize - 1].name
            };
        }
    }
    SlayerName::Silver
}

/// `SlayerGroup.Groups[Utility.Random(SlayerGroup.Groups.Length - 1)].Super.Name`: a random super slayer, not Fey.
pub fn random_super_slayer(w: &mut World) -> SlayerName {
    let all = groups(w);
    let i = muo_primitives::utility::random(&mut w.rng, all.len() as i32 - 1);
    all[i as usize].super_.name
}

/// `BaseRunicTool.GetRandomSlayer()`.
pub fn random_slayer(w: &mut World) -> SlayerName {
    let all = groups(w);
    // 10% chance of a super - skip fey
    if muo_primitives::utility::random_double(&mut w.rng) < 0.10 {
        let i = muo_primitives::utility::random(&mut w.rng, all.len() as i32 - 1);
        return all[i as usize].super_.name;
    }
    // Minor slayer - skip fey and undead
    let g = muo_primitives::utility::random(&mut w.rng, all.len() as i32 - 2);
    let entries = all[g as usize].entries;
    let e = muo_primitives::utility::random(&mut w.rng, entries.len() as i32);
    entries[e as usize].name
}
