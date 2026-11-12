//! `NinjaSpell` and `NinjaMove`: the abstract bases of the Ninjitsu spells
//! (Animal Form, Mirror Image, Shadowjump) and moves. A spell class sets `family` to its [`NinjaType`].

use muo_content::items::weapon_hooks;
use muo_content::spells::SpellInfo;
use muo_content::spells::special_move::{SPECIAL_MOVE_BASE, SpecialMoveType};
use muo_content::spells::spell::{
    self, SPELL_BASE, SpellId, SpellType, base_check_cast, base_check_fizzle,
};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{messages as mm, stats};
use muo_server::skills::{self, SkillName};

use crate::common::is_player_mobile;

/// What a Ninjitsu spell class provides.
pub struct NinjaType {
    /// `RequiredSkill`.
    pub required_skill: f64,
    /// `RequiredMana`.
    pub required_mana: i32,
    /// `Core.ML ? a : b`: the values before ML.
    pub required_skill_pre_ml: Option<f64>,
    pub required_mana_pre_ml: Option<i32>,
}

/// `RequiredSkill` of a spell.
pub fn required_skill(w: &World, sp: SpellId) -> f64 {
    family(w, sp).map_or(0.0, |f| {
        f.required_skill_pre_ml
            .filter(|_| !muo_content::core::ml(w))
            .unwrap_or(f.required_skill)
    })
}

fn family(w: &World, sp: SpellId) -> Option<&'static NinjaType> {
    spell::spell_type(w, sp)?.family::<NinjaType>()
}

/// `RequiredMana` of a spell.
pub fn required_mana(w: &World, sp: SpellId) -> i32 {
    family(w, sp).map_or(0, |f| {
        f.required_mana_pre_ml
            .filter(|_| !muo_content::core::ml(w))
            .unwrap_or(f.required_mana)
    })
}

/// `ScaleMana(mana)`.
pub fn scale_mana_of(w: &mut World, sp: SpellId, mana: i32) -> i32 {
    match spell::spell_type(w, sp) {
        Some(ty) => (ty.scale_mana)(w, sp, mana),
        None => mana,
    }
}

/// `CheckExpansion(from)`: a player whose client supports Samurai Empire.
pub fn check_expansion(w: &World, from: MobileId) -> bool {
    is_player_mobile(w, from) && weapon_hooks::net_supports_se(w, from)
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    let required_mana = required_mana(w, sp);
    let required_skill = required_skill(w, sp);
    let mana = scale_mana_of(w, sp, required_mana);

    if !base_check_cast(w, sp) {
        return false;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };

    if !check_expansion(w, caster) {
        mm::send_localized_message(w, caster, 1063456, "", 0x3B2); // You must upgrade to Samurai Empire in order to use that ability.
        return false;
    }

    if skills::value(w, caster, SkillName::Ninjitsu) < required_skill {
        // You need at least ~1_SKILL_REQUIREMENT~ ~2_SKILL_NAME~ skill to use that ability.
        mm::send_localized_message(
            w,
            caster,
            1063013,
            &format!("{required_skill:.1}\t{:?}\t ", SkillName::Ninjitsu),
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
    let required_mana = required_mana(w, sp);
    let required_skill = required_skill(w, sp);
    let mana = scale_mana_of(w, sp, required_mana);
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };

    if skills::value(w, caster, SkillName::Ninjitsu) < required_skill {
        // You need ~1_SKILL_REQUIREMENT~ Ninjitsu skill to perform that attack!
        mm::send_localized_message(w, caster, 1063352, &format!("{required_skill:.1}"), 0x3B2);
        return false;
    }

    if w.mobile(caster).map_or(0, |c| c.mana) < mana {
        // You must have at least ~1_MANA_REQUIREMENT~ Mana to use this ability.
        mm::send_localized_message(w, caster, 1060174, &mana.to_string(), 0x3B2);
        return false;
    }

    if !base_check_fizzle(w, sp) {
        return false;
    }

    let cur = w.mobile(caster).map_or(0, |c| c.mana);
    props_b::set_mana(w, caster, cur - mana);

    true
}

fn get_cast_skills(w: &mut World, sp: SpellId) -> (f64, f64) {
    let r = required_skill(w, sp);
    (r - 12.5, r + 37.5) // Per 5 on friday 2/16/07
}

/// `NinjaSpell`, abstract.
pub static NINJA_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Ninjitsu.NinjaSpell",
    parent: Some(&SPELL_BASE),
    info: SpellInfo::new("", "", &[]),
    cast_skill: SkillName::Ninjitsu,
    damage_skill: SkillName::Ninjitsu,
    reveal_on_cast: false,
    clear_hands_on_cast: false,
    show_hand_movement: false,
    blocks_movement: |_, _| false,
    cast_recovery_base: 7,
    cast_delay_base: |_, _| TimeSpan::from_secs(1),
    check_cast,
    check_fizzle,
    get_cast_skills,
    get_mana: |_, _| 0,
    ..SPELL_BASE
};

/// `NinjaMove.CheckGain(m)`.
pub fn ninja_check_gain(w: &mut World, ty: &'static SpecialMoveType, m: MobileId) {
    stats::check_skill(
        w,
        m,
        SkillName::Ninjitsu,
        ty.required_skill_in(muo_content::core::ml(w)) - 12.5,
        ty.required_skill_in(muo_content::core::ml(w)) + 37.5, // Per five on friday 02/16/07
    );
}

/// `NinjaMove`, abstract.
pub static NINJA_MOVE: SpecialMoveType = SpecialMoveType {
    full_name: "Server.Spells.NinjaMove",
    move_skill: SkillName::Ninjitsu,
    ..SPECIAL_MOVE_BASE
};
