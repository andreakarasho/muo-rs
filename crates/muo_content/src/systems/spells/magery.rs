//! Magery spells: the eight circles of Magery.
//!
//! `MagerySpell` is a [`SpellType`] ([`MAGERY_SPELL`]) with a [`MageryType`] family: the circle, and the
//! `CheckResisted`/`GetResistPercent` virtuals. A circle's spell class is
//! `SpellType { family: Some(&FIRST_CIRCLE), ..MAGERY_SPELL }` (see `muo_spells`).

use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::stats;
use muo_server::skills::{self, SkillName};

use super::hooks::hooks;
use super::spell::{
    self, SPELL_BASE, SpellId, SpellType, base_consume_reagents, base_get_cast_delay, caster,
    scroll_is_wand, spell_type,
};
use super::types::SpellCircle;
use crate::core;

const MANA_TABLE: [i32; 8] = [4, 6, 9, 11, 14, 20, 40, 50];

/// Starts at circle -2 to account for scrolls. Mana requirements formula: (14 * (circle - 1)) + 2 = 50% probability.
/// Add or subtract 20 for max or min limits.
const REQUIRED_SKILL_ML: [f64; 10] = [
    -46.0, -32.0, -18.0, -4.0, 10.0, 24.0, 38.0, 52.0, 66.0, 80.0,
];
const REQUIRED_SKILL: [f64; 10] = [-50.0, -30.0, 0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0];

/// `MagerySpell`'s own virtuals and the abstract `Circle`.
pub struct MageryType {
    pub circle: SpellCircle,
    /// `CheckResisted(target)`.
    pub check_resisted: fn(&mut World, SpellId, MobileId) -> bool,
    /// `GetResistPercentForCircle(target, circle)`.
    pub get_resist_percent_for_circle: fn(&mut World, SpellId, MobileId, SpellCircle) -> f64,
    /// `GetResistPercent(target)`.
    pub get_resist_percent: fn(&mut World, SpellId, MobileId) -> f64,
}

/// The circle of a Magery spell.
pub fn circle(w: &World, id: SpellId) -> Option<SpellCircle> {
    spell_type(w, id)?.family::<MageryType>().map(|m| m.circle)
}

fn circle_of(w: &World, id: SpellId) -> SpellCircle {
    circle(w, id).unwrap_or_default()
}

fn cast_delay_base(w: &World, id: SpellId) -> TimeSpan {
    let c = circle_of(w, id) as i32;
    let per_tick = spell_type(w, id).map_or(0.25, |t| t.cast_delay_seconds_per_tick);
    TimeSpan::from_secs_f64(f64::from(3 + c) * per_tick)
}

/// `ConsumeReagents() => base.ConsumeReagents() || ArcaneGem.ConsumeCharges(Caster, Core.SE ? 1 : 1 + Circle)`.
fn consume_reagents(w: &mut World, id: SpellId) -> bool {
    if base_consume_reagents(w, id) {
        return true;
    }
    let Some(c) = caster(w, id) else { return false };
    let n = if core::se(w) {
        1
    } else {
        1 + circle_of(w, id) as i32
    };
    hooks(w)
        .and_then(|h| h.arcane_gem_consume_charges)
        .is_some_and(|f| f(w, c, n))
}

fn get_cast_skills(w: &mut World, id: SpellId) -> (f64, f64) {
    // Correct algorithm according to OSI for UOR/UOML
    // TODO: Verify this algorithm on OSI for latest expansion.
    let table = if core::ml(w) {
        &REQUIRED_SKILL_ML
    } else {
        &REQUIRED_SKILL
    };
    let circle = circle_of(w, id) as usize;
    let index = if spell::scroll(w, id).is_none() {
        circle + 2
    } else {
        circle
    };
    let min = table[index.min(9)];
    (min, min + 40.0)
}

fn get_mana(w: &World, id: SpellId) -> i32 {
    if scroll_is_wand(w, id) {
        0
    } else {
        MANA_TABLE[circle_of(w, id) as usize]
    }
}

fn get_resist_skill(w: &mut World, id: SpellId, m: MobileId) -> f64 {
    let circle = circle_of(w, id) as i32;
    let max_skill = 1 + circle * 10 + (1 + circle / 6) * 25;

    if skills::value(w, m, SkillName::MagicResist) < f64::from(max_skill) {
        let cap = skills::cap(w, m, SkillName::MagicResist);
        stats::check_skill(w, m, SkillName::MagicResist, 0.0, cap);
    }

    skills::value(w, m, SkillName::MagicResist)
}

/// `CheckResisted(target)`.
pub fn check_resisted(w: &mut World, id: SpellId, target: MobileId) -> bool {
    let Some(fam) = spell_type(w, id).and_then(|t| t.family::<MageryType>()) else {
        return false;
    };
    let n = (fam.get_resist_percent)(w, id, target) / 100.0;

    if n <= 0.0 {
        return false;
    }

    if n >= 1.0 {
        return true;
    }

    // Even though this calculation matches AOS+, we don't combine with GetResistSkills because of an assumption
    // about how it is used.
    let circle = circle_of(w, id) as i32;
    let max_skill = (1 + circle) * 10 + (1 + circle / 6) * 25;

    if skills::value(w, target, SkillName::MagicResist) < f64::from(max_skill) {
        let cap = skills::cap(w, target, SkillName::MagicResist);
        stats::check_skill(w, target, SkillName::MagicResist, 0.0, cap);
    }

    n >= utility::random_double(&mut w.rng)
}

/// `GetResistPercentForCircle(target, circle)`.
pub fn get_resist_percent_for_circle(
    w: &mut World,
    id: SpellId,
    target: MobileId,
    circle: SpellCircle,
) -> f64 {
    let magic_resist = skills::value(w, target, SkillName::MagicResist);
    let first_percent = magic_resist / 5.0;
    let cast_skill = spell_type(w, id).map_or(SkillName::Magery, |t| t.cast_skill);
    let caster_skill = caster(w, id).map_or(0.0, |c| skills::value(w, c, cast_skill));
    let second_percent =
        magic_resist - ((caster_skill - 20.0) / 5.0 + f64::from(1 + circle as i32) * 5.0);

    // Seems should be about half of what stratics says.
    (if first_percent > second_percent {
        first_percent
    } else {
        second_percent
    }) / 2.0
}

/// `GetResistPercent(target) => GetResistPercentForCircle(target, Circle)`.
pub fn get_resist_percent(w: &mut World, id: SpellId, target: MobileId) -> f64 {
    let circle = circle_of(w, id);
    match spell_type(w, id).and_then(|t| t.family::<MageryType>()) {
        Some(f) => (f.get_resist_percent_for_circle)(w, id, target, circle),
        None => 0.0,
    }
}

fn get_cast_delay(w: &mut World, id: SpellId) -> TimeSpan {
    if !core::ml(w) && scroll_is_wand(w, id) {
        return TimeSpan::ZERO;
    }

    if !core::aos(w) {
        return TimeSpan::from_secs_f64(0.5 + 0.25 * f64::from(circle_of(w, id) as i32));
    }

    base_get_cast_delay(w, id)
}

/// The default family of a circle: copy it with the circle changed, e.g. `MageryType { circle: Third, ..BASE }`.
pub static MAGERY_BASE: MageryType = MageryType {
    circle: SpellCircle::First,
    check_resisted,
    get_resist_percent_for_circle,
    get_resist_percent,
};

/// One family per circle, for the spell classes to point at.
pub static FIRST_CIRCLE: MageryType = MageryType {
    circle: SpellCircle::First,
    ..MAGERY_BASE
};
pub static SECOND_CIRCLE: MageryType = MageryType {
    circle: SpellCircle::Second,
    ..MAGERY_BASE
};
pub static THIRD_CIRCLE: MageryType = MageryType {
    circle: SpellCircle::Third,
    ..MAGERY_BASE
};
pub static FOURTH_CIRCLE: MageryType = MageryType {
    circle: SpellCircle::Fourth,
    ..MAGERY_BASE
};
pub static FIFTH_CIRCLE: MageryType = MageryType {
    circle: SpellCircle::Fifth,
    ..MAGERY_BASE
};
pub static SIXTH_CIRCLE: MageryType = MageryType {
    circle: SpellCircle::Sixth,
    ..MAGERY_BASE
};
pub static SEVENTH_CIRCLE: MageryType = MageryType {
    circle: SpellCircle::Seventh,
    ..MAGERY_BASE
};
pub static EIGHTH_CIRCLE: MageryType = MageryType {
    circle: SpellCircle::Eighth,
    ..MAGERY_BASE
};

/// `MagerySpell`, abstract.
pub static MAGERY_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.MagerySpell",
    parent: Some(&SPELL_BASE),
    cast_delay_base,
    get_mana,
    consume_reagents,
    get_cast_skills,
    get_resist_skill,
    get_cast_delay,
    family: Some(&MAGERY_BASE),
    ..SPELL_BASE
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mana_and_skill_tables() {
        assert_eq!(MANA_TABLE, [4, 6, 9, 11, 14, 20, 40, 50]);
        // A spell cast from a book (no scroll) of the first circle needs the table entry at circle + 2.
        assert_eq!(REQUIRED_SKILL[SpellCircle::First as usize + 2], 0.0);
        assert_eq!(
            REQUIRED_SKILL_ML[SpellCircle::Eighth as usize + 2],
            80.0 - 14.0 * 0.0 + 0.0
        );
    }
}
