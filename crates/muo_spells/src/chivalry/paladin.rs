//! `PaladinSpell`: the abstract base of the Chivalry spells. A concrete class sets `family`
//! to its [`PaladinType`] (the abstract `RequiredSkill`, `RequiredMana`, `RequiredTithing`, `MantraNumber`, and the
//! virtual `SendCastEffect`).

use muo_content::core;
use muo_content::spells::spell::{
    self, SPELL_BASE, SpellId, SpellType, base_check_cast, base_check_fizzle, base_get_cast_delay,
    base_on_begin_cast, base_on_disturb,
};
use muo_content::spells::{DisturbType, SpellInfo};
use muo_primitives::{MessageType, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};

/// What a Chivalry spell class provides.
pub struct PaladinType {
    /// `RequiredSkill`.
    pub required_skill: f64,
    /// `RequiredMana`.
    pub required_mana: i32,
    /// `RequiredTithing`.
    pub required_tithing: i32,
    /// `MantraNumber`.
    pub mantra_number: i32,
    /// `SendCastEffect()`.
    pub send_cast_effect: fn(&mut World, SpellId),
}

/// `PaladinSpell.SendCastEffect()`: `Caster.FixedEffect(0x37C4, 10, (int)(GetCastDelay().TotalSeconds * 28), 4, 3)`.
pub fn base_send_cast_effect(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let ty = spell::spell_type(w, sp).expect("the spell exists");
    let secs = (ty.get_cast_delay)(w, sp).total_seconds();
    effects::fixed_effect_hue(w, caster, 0x37C4, 10, (secs * 28.0) as i32, 4, 3);
}

/// The default [`PaladinType`] to copy with struct update.
pub const PALADIN_BASE: PaladinType = PaladinType {
    required_skill: 0.0,
    required_mana: 0,
    required_tithing: 0,
    mantra_number: 0,
    send_cast_effect: base_send_cast_effect,
};

fn family(w: &World, sp: SpellId) -> Option<&'static PaladinType> {
    spell::spell_type(w, sp)?.family::<PaladinType>()
}

fn required_mana(w: &World, sp: SpellId) -> i32 {
    family(w, sp).map_or(0, |f| f.required_mana)
}

fn required_tithing(w: &World, sp: SpellId) -> i32 {
    family(w, sp).map_or(0, |f| f.required_tithing)
}

fn scale_mana_of(w: &mut World, sp: SpellId, mana: i32) -> i32 {
    match spell::spell_type(w, sp) {
        Some(ty) => (ty.scale_mana)(w, sp, mana),
        None => mana,
    }
}

fn tithing(w: &World, m: MobileId) -> i32 {
    w.mobile(m).map_or(0, |x| x.tithing_points)
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    let mana = scale_mana_of(w, sp, required_mana(w, sp));
    if !base_check_cast(w, sp) {
        return false;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };

    if tithing(w, caster) < required_tithing(w, sp) {
        // You must have at least ~1_TITHE_REQUIREMENT~ Tithing Points to use this ability,
        mm::send_localized_message(
            w,
            caster,
            1060173,
            &required_tithing(w, sp).to_string(),
            0x3B2,
        );
        return false;
    }

    if w.mobile(caster).map_or(0, |c| c.mana) < mana {
        // You must have at least ~1_MANA_REQUIREMENT~ Mana to use this ability.
        mm::send_localized_message(w, caster, 1060174, &mana.to_string(), 0x3B2);
        return false;
    }

    true
}

fn check_fizzle(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    let mut required = required_tithing(w, sp);

    let lrc = muo_content::misc::aos_helpers::attribute_value(
        w,
        caster,
        muo_content::aos::AosAttribute::LowerRegCost,
    );
    if lrc > muo_primitives::utility::random(&mut w.rng, 100) {
        required = 0;
    }

    let mana = scale_mana_of(w, sp, required_mana(w, sp));

    if tithing(w, caster) < required {
        // You must have at least ~1_TITHE_REQUIREMENT~ Tithing Points to use this ability,
        mm::send_localized_message(
            w,
            caster,
            1060173,
            &required_tithing(w, sp).to_string(),
            0x3B2,
        );
        return false;
    }

    if w.mobile(caster).map_or(0, |c| c.mana) < mana {
        // You must have at least ~1_MANA_REQUIREMENT~ Mana to use this ability.
        mm::send_localized_message(w, caster, 1060174, &mana.to_string(), 0x3B2);
        return false;
    }

    let points = w.mobile(caster).map_or(0, |c| c.tithing_points);
    muo_server::mobile::part1::props_a::set_tithing_points(w, caster, points - required);

    if !base_check_fizzle(w, sp) {
        return false;
    }

    let cur = w.mobile(caster).map_or(0, |c| c.mana);
    props_b::set_mana(w, caster, cur - mana);

    true
}

/// The `CheckCast` of a Paladin spell that first refuses in sudden death.
pub fn check_cast_sudden_death(w: &mut World, sp: SpellId) -> bool {
    !crate::common::sudden_death_blocks(w, sp) && check_cast(w, sp)
}

fn say_mantra(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let n = family(w, sp).map_or(0, |f| f.mantra_number);
    mm::public_overhead_message_number(w, caster, MessageType::REGULAR, 0x3B2, n, "", false);
}

fn do_fizzle(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    effects::play_sound(w, caster, 0x1D6);
    let tick = w.tick_count;
    if let Some(c) = w.mobile_mut(caster) {
        c.next_spell_time = tick;
    }
}

fn do_hurt_fizzle(w: &mut World, sp: SpellId) {
    if let Some(caster) = spell::caster(w, sp) {
        effects::play_sound(w, caster, 0x1D6);
    }
}

fn on_disturb(w: &mut World, sp: SpellId, t: DisturbType, message: bool) {
    base_on_disturb(w, sp, t, message);

    if message && let Some(caster) = spell::caster(w, sp) {
        effects::play_sound(w, caster, 0x1D6);
    }
}

fn on_begin_cast(w: &mut World, sp: SpellId) {
    base_on_begin_cast(w, sp);
    send_cast_effect(w, sp);
}

fn send_cast_effect(w: &mut World, sp: SpellId) {
    if let Some(f) = family(w, sp) {
        (f.send_cast_effect)(w, sp);
    }
}

fn get_cast_skills(w: &mut World, sp: SpellId) -> (f64, f64) {
    let r = family(w, sp).map_or(0.0, |f| f.required_skill);
    (r, r + 50.0)
}

/// `CastDelayMinimum => TimeSpan.FromSeconds(Core.SA ? 0.5 : 0.25)`: the type's field holds 0.25.
fn get_cast_delay(w: &mut World, sp: SpellId) -> TimeSpan {
    let d = base_get_cast_delay(w, sp);
    let min = TimeSpan::from_millis(500);
    if core::sa(w) && d < min { min } else { d }
}

/// `ComputePowerValue(from, div)`.
pub fn compute_power_value(w: &mut World, from: MobileId, div: i32) -> i32 {
    let karma = w.mobile(from).map_or(0, |m| m.karma);
    let chiv = skills::value(w, from, SkillName::Chivalry);
    let v = (f64::from(karma) + 20000.0 + chiv * 100.0).sqrt() as i32;
    v / div
}

/// `ComputePowerValue(div)` of a spell.
pub fn power(w: &mut World, sp: SpellId, div: i32) -> i32 {
    match spell::caster(w, sp) {
        Some(c) => compute_power_value(w, c, div),
        None => 0,
    }
}

/// `PaladinSpell`, abstract.
pub static PALADIN_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Chivalry.PaladinSpell",
    parent: Some(&SPELL_BASE),
    info: SpellInfo::new("", "", &[]),
    cast_skill: SkillName::Chivalry,
    damage_skill: SkillName::Chivalry,
    clear_hands_on_cast: false,
    cast_recovery_base: 7,
    cast_delay_minimum: TimeSpan::from_millis(250),
    check_cast,
    check_fizzle,
    say_mantra,
    do_fizzle,
    do_hurt_fizzle,
    on_disturb,
    on_begin_cast,
    get_cast_skills,
    get_cast_delay,
    get_mana: |_, _| 0,
    family: Some(&PALADIN_BASE),
    ..SPELL_BASE
};
