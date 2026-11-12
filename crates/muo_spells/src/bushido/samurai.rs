//! `SamuraiSpell` and `SamuraiMove`: the abstract bases of the Bushido spells
//! (Confidence, Counter Attack, Evasion) and moves. A spell class sets `family` to its [`SamuraiType`].

use muo_content::items::weapon_hooks;
use muo_content::spells::SpellInfo;
use muo_content::spells::registry::registry;
use muo_content::spells::special_move::{
    SPECIAL_MOVE_BASE, SpecialMoveType, toggle_special_ability_packet,
};
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

/// What a Bushido spell class provides.
pub struct SamuraiType {
    /// `RequiredSkill`.
    pub required_skill: f64,
    /// `RequiredMana`.
    pub required_mana: i32,
}

fn family(w: &World, sp: SpellId) -> Option<&'static SamuraiType> {
    spell::spell_type(w, sp)?.family::<SamuraiType>()
}

fn scale_mana_of(w: &mut World, sp: SpellId, mana: i32) -> i32 {
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
    let required_mana = family(w, sp).map_or(0, |f| f.required_mana);
    let required_skill = family(w, sp).map_or(0.0, |f| f.required_skill);
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

    if skills::value(w, caster, SkillName::Bushido) < required_skill {
        // You need at least ~1_SKILL_REQUIREMENT~ ~2_SKILL_NAME~ skill to use that ability.
        mm::send_localized_message(
            w,
            caster,
            1063013,
            &format!("{}\t{:?}\t ", format_g(required_skill), SkillName::Bushido),
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

/// `{x:0.#}`.
fn format_g(x: f64) -> String {
    let s = format!("{x:.1}");
    s.strip_suffix(".0").map_or(s.clone(), str::to_owned)
}

fn check_fizzle(w: &mut World, sp: SpellId) -> bool {
    let required_mana = family(w, sp).map_or(0, |f| f.required_mana);
    let required_skill = family(w, sp).map_or(0.0, |f| f.required_skill);
    let mana = scale_mana_of(w, sp, required_mana);
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };

    if skills::value(w, caster, SkillName::Bushido) < required_skill {
        // You need ~1_SKILL_REQUIREMENT~ Bushido skill to perform that attack!
        mm::send_localized_message(w, caster, 1070768, &format!("{required_skill:.1}"), 0x3B2);
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
    let r = family(w, sp).map_or(0.0, |f| f.required_skill);
    (r - 12.5, r + 37.5) // per 5 on friday, 2/16/07
}

/// `SamuraiSpell`, abstract.
pub static SAMURAI_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Bushido.SamuraiSpell",
    parent: Some(&SPELL_BASE),
    info: SpellInfo::new("", "", &[]),
    cast_skill: SkillName::Bushido,
    damage_skill: SkillName::Bushido,
    clear_hands_on_cast: false,
    show_hand_movement: false,
    blocks_movement: |_, _| false,
    cast_delay_fast_scalar: 0.0,
    cast_recovery_base: 7,
    cast_delay_base: |_, _| TimeSpan::from_millis(250),
    check_cast,
    check_fizzle,
    get_cast_skills,
    get_mana: |_, _| 0,
    ..SPELL_BASE
};

fn send_toggle(w: &mut World, m: MobileId, ability_id: i32, active: bool) {
    if let Some(ns) = w.mobile(m).and_then(|x| x.net_state) {
        w.send(ns, &toggle_special_ability_packet(ability_id, active));
    }
}

/// `OnCastSuccessful(caster)`: ends the other effects and shows this one's icon.
pub fn on_cast_successful(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    super::evasion::end_evasion(w, caster);
    super::confidence::end_confidence(w, caster);
    super::counter_attack::stop_countering(w, caster);

    if let Some(ty) = spell::spell_type(w, sp) {
        let id = registry(w).registry_number(ty.full_name);
        if id > 0 {
            send_toggle(w, caster, id + 1, true);
        }
    }
}

/// `OnEffectEnd(caster, type)`.
pub fn on_effect_end(w: &mut World, caster: MobileId, type_name: &str) {
    let id = registry(w).registry_number(type_name);
    if id > 0 {
        send_toggle(w, caster, id + 1, false);
    }
}

// ---- SamuraiMove ----

/// `SamuraiMove.CheckGain(m)`.
pub fn samurai_check_gain(w: &mut World, ty: &'static SpecialMoveType, m: MobileId) {
    stats::check_skill(
        w,
        m,
        SkillName::Bushido,
        ty.required_skill - 12.5,
        ty.required_skill + 37.5, // Per five on friday 02/16/07
    );
}

/// `SamuraiMove`, abstract.
pub static SAMURAI_MOVE: SpecialMoveType = SpecialMoveType {
    full_name: "Server.Spells.SamuraiMove",
    move_skill: SkillName::Bushido,
    ..SPECIAL_MOVE_BASE
};
