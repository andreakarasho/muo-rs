//! `MysticSpell`: the abstract base of the Mysticism spells. A class sets `family` to its
//! [`MysticType`].

use muo_content::spells::spell::{
    self, SPELL_BASE, SpellId, SpellType, base_check_cast, base_get_cast_delay, base_on_begin_cast,
    scroll_is_wand,
};
use muo_content::spells::{SpellCircle, SpellInfo};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::{effects, messages as mm, stats};
use muo_server::skills::{self, SkillName};

const MANA_TABLE: [i32; 8] = [4, 6, 9, 11, 14, 20, 40, 50];
const REQUIRED_SKILL: [f64; 8] = [0.0, 8.0, 20.0, 33.0, 45.0, 58.0, 70.0, 83.0];

/// What a Mysticism spell class provides.
pub struct MysticType {
    /// `Circle`.
    pub circle: SpellCircle,
    /// `GetMana()` when it is not the circle's (Hail Storm: 50).
    pub mana: Option<i32>,
}

/// The circle of a Mysticism spell.
pub fn circle(w: &World, sp: SpellId) -> SpellCircle {
    spell::spell_type(w, sp)
        .and_then(|t| t.family::<MysticType>())
        .map_or(SpellCircle::First, |f| f.circle)
}

fn cast_delay_base(w: &World, sp: SpellId) -> TimeSpan {
    TimeSpan::from_secs_f64(0.5 + 0.25 * f64::from(circle(w, sp) as i32))
}

/// `RequiredSkill`.
pub fn required_skill(w: &World, sp: SpellId) -> f64 {
    let mut c = circle(w, sp) as i32;
    if spell::scroll(w, sp).is_some() {
        c -= 2;
    }
    REQUIRED_SKILL[c.clamp(0, 7) as usize]
}

/// `GetBaseSkill(m)`.
pub fn get_base_skill(w: &mut World, m: MobileId) -> f64 {
    skills::value(w, m, SkillName::Mysticism)
}

fn get_damage_skill(w: &mut World, _sp: SpellId, m: MobileId) -> f64 {
    skills::value(w, m, SkillName::Imbuing).max(skills::value(w, m, SkillName::Focus))
}

fn get_damage_fixed(w: &mut World, _sp: SpellId, m: MobileId) -> i32 {
    skills::fixed(w, m, SkillName::Imbuing).max(skills::fixed(w, m, SkillName::Focus))
}

fn get_cast_skills(w: &mut World, sp: SpellId) -> (f64, f64) {
    let r = required_skill(w, sp);
    // As per Mysticism page at the UO Herald Playguide
    // This means that we have 25% success chance at min Required Skill
    (r - 12.5, r + 37.5)
}

fn get_mana(w: &World, sp: SpellId) -> i32 {
    if scroll_is_wand(w, sp) {
        return 0;
    }
    spell::spell_type(w, sp)
        .and_then(|t| t.family::<MysticType>())
        .map_or(0, |f| f.mana.unwrap_or(MANA_TABLE[f.circle as usize]))
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    if !base_check_cast(w, sp) {
        return false;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };

    let ty = spell::spell_type(w, sp).expect("the spell exists");
    let mana = (ty.scale_mana)(w, sp, (ty.get_mana)(w, sp));
    if w.mobile(caster).map_or(0, |c| c.mana) < mana {
        // You must have at least ~1_MANA_REQUIREMENT~ Mana to use this ability.
        mm::send_localized_message(w, caster, 1060174, &mana.to_string(), 0x3B2);
        return false;
    }

    let required = required_skill(w, sp);
    if skills::value(w, caster, SkillName::Mysticism) < required {
        // You need at least ~1_SKILL_REQUIREMENT~ ~2_SKILL_NAME~ skill to use that ability.
        mm::send_localized_message(
            w,
            caster,
            1063013,
            &format!("{required:.1}\t{:?}\t ", SkillName::Mysticism),
            0x3B2,
        );
        return false;
    }

    true
}

fn on_begin_cast(w: &mut World, sp: SpellId) {
    base_on_begin_cast(w, sp);
    send_cast_effect(w, sp);
}

/// `SendCastEffect()`.
pub fn send_cast_effect(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let ty = spell::spell_type(w, sp).expect("the spell exists");
    let secs = (ty.get_cast_delay)(w, sp).total_seconds();
    effects::fixed_effect_hue(w, caster, 0x37C4, 10, (secs * 28.0) as i32, 0x66C, 3);
}

/// `GetResistPercentForCircle(target, circle)`.
pub fn get_resist_percent_for_circle(
    w: &mut World,
    sp: SpellId,
    target: MobileId,
    c: SpellCircle,
) -> f64 {
    let magic_resist = skills::value(w, target, SkillName::MagicResist);
    let first_percent = magic_resist / 5.0;
    let caster_skill =
        spell::caster(w, sp).map_or(0.0, |c| skills::value(w, c, SkillName::Mysticism));
    let second_percent =
        magic_resist - ((caster_skill - 20.0) / 5.0 + f64::from(1 + c as i32) * 5.0);

    // Seems should be about half of what stratics says.
    (if first_percent > second_percent {
        first_percent
    } else {
        second_percent
    }) / 2.0
}

/// `GetResistPercent(target)`.
pub fn get_resist_percent(w: &mut World, sp: SpellId, target: MobileId) -> f64 {
    get_resist_percent_for_circle(w, sp, target, circle(w, sp))
}

/// `CheckResisted(target)`.
pub fn check_resisted(w: &mut World, sp: SpellId, target: MobileId) -> bool {
    let n = get_resist_percent(w, sp, target) / 100.0;

    if n <= 0.0 {
        return false;
    }

    if n >= 1.0 {
        return true;
    }

    let c = circle(w, sp) as i32;
    let mut max_skill = (1 + c) * 10;
    max_skill += (1 + c / 6) * 25;

    if skills::value(w, target, SkillName::MagicResist) < f64::from(max_skill) {
        let cap = skills::cap(w, target, SkillName::MagicResist);
        stats::check_skill(w, target, SkillName::MagicResist, 0.0, cap);
    }

    n >= utility::random_double(&mut w.rng)
}

/// `MysticSpell`, abstract.
pub static MYSTIC_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Mysticism.MysticSpell",
    parent: Some(&SPELL_BASE),
    info: SpellInfo::new("", "", &[]),
    cast_skill: SkillName::Mysticism,
    cast_delay_base,
    get_damage_skill,
    get_damage_fixed,
    get_cast_skills,
    get_mana,
    check_cast,
    on_begin_cast,
    get_cast_delay: base_get_cast_delay,
    family: Some(&MysticType {
        circle: SpellCircle::First,
        mana: None,
    }),
    ..SPELL_BASE
};

/// A `MysticType` for `circle`.
pub const fn mystic(circle: SpellCircle) -> MysticType {
    MysticType { circle, mana: None }
}
