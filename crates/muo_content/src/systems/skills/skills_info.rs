//! Skill info: the skill table (`Data/skills.json`) and the random skill pickers.

use muo_data::expansion::Expansion;
use muo_primitives::utility;
use muo_server::World;
use muo_server::skills::{SkillName, SkillTable};

use crate::core;
use crate::misc::aos_helpers;

/// `CombatSkills`.
pub const COMBAT_SKILLS: [SkillName; 11] = [
    SkillName::Parry,
    SkillName::Tactics,
    SkillName::Archery,
    SkillName::Swords,
    SkillName::Macing,
    SkillName::Fencing,
    SkillName::Wrestling,
    // AOS
    SkillName::Focus,
    // SE
    SkillName::Bushido,
    SkillName::Ninjitsu,
    // SA
    SkillName::Throwing,
];

/// `CraftSkills`.
pub const CRAFT_SKILLS: [SkillName; 10] = [
    SkillName::Alchemy,
    SkillName::Blacksmith,
    SkillName::Fletching,
    SkillName::Carpentry,
    SkillName::Cartography,
    SkillName::Cooking,
    SkillName::Inscribe,
    SkillName::Tailoring,
    SkillName::Tinkering,
    // SA
    SkillName::Imbuing,
];

/// `MagicSkills`.
pub const MAGIC_SKILLS: [SkillName; 8] = [
    SkillName::EvalInt,
    SkillName::Magery,
    SkillName::MagicResist,
    SkillName::Meditation,
    // AOS
    SkillName::Necromancy,
    SkillName::Chivalry,
    // ML
    SkillName::Spellweaving,
    // SA
    SkillName::Mysticism,
];

/// `GatheringSkills`.
pub const GATHERING_SKILLS: [SkillName; 3] = [
    SkillName::Fishing,
    SkillName::Mining,
    SkillName::Lumberjacking,
];

/// `BardSkills`.
pub const BARD_SKILLS: [SkillName; 4] = [
    SkillName::Peacemaking,
    SkillName::Discordance,
    SkillName::Provocation,
    SkillName::Musicianship,
];

fn expansion(w: &World) -> Expansion {
    core::expansion(w)
}

/// `RandomSkill()`.
pub fn random_skill(w: &mut World) -> SkillName {
    let e = expansion(w);
    let exclusive_max_index = if e >= Expansion::SA {
        58 // <= Throwing
    } else if e >= Expansion::ML {
        55 // <= Spellweaving
    } else if e == Expansion::SE {
        54 // <= Ninjitsu
    } else if e == Expansion::AOS {
        52 // <= Chivalry
    } else {
        49 // <= RemoveTrap
    };
    SkillName::from_i32(utility::random(&mut w.rng, exclusive_max_index)).unwrap_or_default()
}

/// `RandomCombatSkill()`.
pub fn random_combat_skill(w: &mut World) -> SkillName {
    let e = expansion(w);
    let n = if e >= Expansion::SA {
        11
    } else if e >= Expansion::SE {
        10
    } else if e == Expansion::AOS {
        8
    } else {
        7
    };
    COMBAT_SKILLS[utility::random(&mut w.rng, n) as usize]
}

/// `RandomCraftSkill()`.
pub fn random_craft_skill(w: &mut World) -> SkillName {
    let n = if expansion(w) >= Expansion::SA { 10 } else { 9 };
    CRAFT_SKILLS[utility::random(&mut w.rng, n) as usize]
}

/// `RandomMagicSkill()`.
pub fn random_magic_skill(w: &mut World) -> SkillName {
    let e = expansion(w);
    let n = if e >= Expansion::SA {
        8
    } else if e >= Expansion::ML {
        7
    } else if e == Expansion::AOS {
        6
    } else {
        4
    };
    MAGIC_SKILLS[utility::random(&mut w.rng, n) as usize]
}

/// `RandomGatheringSkill()`.
pub fn random_gathering_skill(w: &mut World) -> SkillName {
    *utility::random_element(&mut w.rng, &GATHERING_SKILLS).expect("not empty")
}

/// `RandomBardSkill()`.
pub fn random_bard_skill(w: &mut World) -> SkillName {
    *utility::random_element(&mut w.rng, &BARD_SKILLS).expect("not empty")
}

/// `SkillsInfo.Configure()`: loads `Data/skills.json`; AOS turns the stat influences off.
pub fn configure(w: &mut World) {
    let base = w.core.base_directory.clone();
    match muo_data::skill_info::SkillInfo::load_table(&base) {
        Ok(table) => {
            let len = table.len();
            let t = w.resources.get_mut::<SkillTable>();
            t.info = table;
            t.callbacks.resize(len.max(t.callbacks.len()), None);
        }
        Err(e) => tracing::error!("Failed to read Data/skills.json: {e}"),
    }

    if core::aos(w) {
        aos_helpers::disable_stat_influences(w);
    }
}

/// `SkillInfo.Table[skill].Callback = callback`.
pub fn set_callback(
    w: &mut World,
    skill: SkillName,
    callback: muo_server::skills::SkillUseCallback,
) {
    let t = w.resources.get_mut::<SkillTable>();
    let i = skill as usize;
    if t.callbacks.len() <= i {
        t.callbacks.resize(i + 1, None);
    }
    t.callbacks[i] = Some(callback);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_skills_stay_inside_the_expansion() {
        let mut w = World::new();
        w.core.expansion = Some(Expansion::T2A);
        for _ in 0..200 {
            assert!((random_skill(&mut w) as i32) < 49);
            assert!(!matches!(
                random_combat_skill(&mut w),
                SkillName::Focus | SkillName::Bushido | SkillName::Ninjitsu | SkillName::Throwing
            ));
            assert_ne!(random_craft_skill(&mut w), SkillName::Imbuing);
            assert!(matches!(
                random_magic_skill(&mut w),
                SkillName::EvalInt
                    | SkillName::Magery
                    | SkillName::MagicResist
                    | SkillName::Meditation
            ));
        }
        w.core.expansion = Some(Expansion::SA);
        let mut seen_throwing = false;
        for _ in 0..2000 {
            seen_throwing |= random_combat_skill(&mut w) == SkillName::Throwing;
        }
        assert!(seen_throwing);
    }

    #[test]
    fn callbacks_grow_the_table() {
        let mut w = World::new();
        set_callback(&mut w, SkillName::Hiding, |_, _| {
            muo_primitives::TimeSpan::from_secs(1)
        });
        let t = w.resources.get_mut::<SkillTable>();
        assert!(t.callback(SkillName::Hiding as usize).is_some());
        assert!(t.callback(SkillName::Alchemy as usize).is_none());
    }
}
