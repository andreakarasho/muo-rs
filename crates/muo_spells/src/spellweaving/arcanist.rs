//! `ArcanistSpell`, `ArcaneForm` and `ArcaneSummon`: the abstract bases of the
//! Spellweaving spells. A class sets `family` to its [`ArcaneType`].

use indexmap::IndexMap;
use muo_content::spells::helper::{self, TransformationInfo};
use muo_content::spells::hooks::hooks;
use muo_content::spells::spell::{
    self, SPELL_BASE, SpellId, SpellType, base_check_cast, base_on_begin_cast, base_on_disturb,
};
use muo_content::spells::{DisturbType, SpellInfo};
use muo_primitives::utility;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};

use super::items::{ARCANE_FOCUS, strength_bonus};
use crate::common::is_player_mobile;

/// What a Spellweaving spell class provides.
pub struct ArcaneType {
    /// `RequiredSkill`.
    pub required_skill: f64,
    /// `RequiredMana`.
    pub required_mana: i32,
    /// For an `ArcaneForm`: the `ITransformationSpell` members, by the spell's focus level.
    pub transformation: Option<fn(&World, i32) -> &'static TransformationInfo>,
}

/// The default [`ArcaneType`] to copy with struct update.
pub const ARCANE_BASE: ArcaneType = ArcaneType {
    required_skill: 0.0,
    required_mana: 0,
    transformation: None,
};

/// `m_CastTimeFocusLevel`, per spell; and the last one per caster (for the `ITransformationSpell` callbacks, which
/// get no spell).
#[derive(Default)]
struct Focus {
    by_spell: IndexMap<SpellId, i32>,
    by_caster: IndexMap<MobileId, i32>,
}

fn focus_state(w: &mut World) -> &mut Focus {
    w.resources.get_mut::<Focus>()
}

/// `FocusLevel`: the level read when the cast began.
pub fn focus_level(w: &mut World, sp: SpellId) -> i32 {
    focus_state(w).by_spell.get(&sp).copied().unwrap_or(0)
}

/// The level of the caster's latest cast.
pub fn caster_focus_level(w: &mut World, caster: MobileId) -> i32 {
    focus_state(w).by_caster.get(&caster).copied().unwrap_or(0)
}

fn family(w: &World, sp: SpellId) -> Option<&'static ArcaneType> {
    spell::spell_type(w, sp)?.family::<ArcaneType>()
}

/// `FindArcaneFocus(from)`: `from.Holding as ArcaneFocus ?? from.Backpack?.FindItemByType<ArcaneFocus>()`.
pub fn find_arcane_focus(w: &mut World, from: MobileId) -> Option<ItemId> {
    if let Some(h) = w.mobile(from).and_then(|m| m.holding)
        && w.item(h).is_some_and(|i| i.ty.is(&ARCANE_FOCUS))
    {
        return Some(h);
    }
    let pack = props_b::backpack(w, from)?;
    muo_server::item::container::find_item_by_type(w, pack, &ARCANE_FOCUS, true)
}

/// `GetFocusLevel(from)`.
pub fn get_focus_level(w: &mut World, from: MobileId) -> i32 {
    match find_arcane_focus(w, from) {
        Some(f) if !muo_server::item::ops::is_deleted(w, f) => strength_bonus(w, f),
        _ => 0,
    }
}

/// `CheckExpansion(from)`.
pub fn check_expansion(w: &World, from: MobileId) -> bool {
    // `NetState.SupportsExpansion(Expansion.ML)`. Unset: supported.
    !is_player_mobile(w, from)
        || hooks(w)
            .and_then(|h| h.net_supports_ml)
            .is_none_or(|f| f(w, from))
}

fn ml_quest_flag(w: &World, m: MobileId, flag: &str) -> bool {
    hooks(w)
        .and_then(|h| h.ml_quest_flag)
        .is_none_or(|f| f(w, m, flag))
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    if !base_check_cast(w, sp) {
        return false;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    let required_mana = family(w, sp).map_or(0, |f| f.required_mana);
    let required_skill = family(w, sp).map_or(0.0, |f| f.required_skill);

    if !check_expansion(w, caster) {
        // You must upgrade to the Mondain's Legacy Expansion Pack before using that ability
        mm::send_localized_message(w, caster, 1072176, "", 0x3B2);
        return false;
    }

    if is_player_mobile(w, caster) && !ml_quest_flag(w, caster, "Spellweaving") {
        // You must have completed the epic arcanist quest to use this ability.
        mm::send_localized_message(w, caster, 1073220, "", 0x3B2);
        return false;
    }

    let ty = spell::spell_type(w, sp).expect("the spell exists");
    let mana = (ty.scale_mana)(w, sp, required_mana);
    if w.mobile(caster).map_or(0, |c| c.mana) < mana {
        // You must have at least ~1_MANA_REQUIREMENT~ Mana to use this ability.
        mm::send_localized_message(w, caster, 1060174, &mana.to_string(), 0x3B2);
        return false;
    }

    if skills::value(w, caster, SkillName::Spellweaving) < required_skill {
        // You need at least ~1_SKILL_REQUIREMENT~ ~2_SKILL_NAME~ skill to use that ability.
        mm::send_localized_message(
            w,
            caster,
            1063013,
            &format!("{required_skill:.1}\t#1044114"),
            0x3B2,
        );
        return false;
    }

    true
}

fn get_cast_skills(w: &mut World, sp: SpellId) -> (f64, f64) {
    let r = family(w, sp).map_or(0.0, |f| f.required_skill);
    (r - 12.5, r + 37.5) // per 5 on Friday, 2/16/07
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
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    send_cast_effect(w, sp);
    let level = get_focus_level(w, caster);
    focus_state(w).by_spell.insert(sp, level);
    focus_state(w).by_caster.insert(caster, level);
}

fn send_cast_effect(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let ty = spell::spell_type(w, sp).expect("the spell exists");
    let secs = (ty.get_cast_delay)(w, sp).total_seconds();
    effects::fixed_effect_hue(w, caster, 0x37C4, 10, (secs * 28.0) as i32, 4, 3);
}

/// `CheckResisted(m)`.
pub fn check_resisted(w: &mut World, sp: SpellId, m: MobileId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    let ty = spell::spell_type(w, sp).expect("the spell exists");
    // TODO: According to the guide this is it.. but.. is it correct per OSI?
    let percent = (50.0
        + 2.0 * ((ty.get_resist_skill)(w, sp, m) - (ty.get_damage_skill)(w, sp, caster)))
        / 100.0;

    if percent <= 0.0 {
        false
    } else if percent >= 1.0 {
        true
    } else {
        percent >= utility::random_double(&mut w.rng)
    }
}

/// `ArcanistSpell`, abstract.
pub static ARCANIST_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.ArcanistSpell",
    parent: Some(&SPELL_BASE),
    info: SpellInfo::new("", "", &[]),
    cast_skill: SkillName::Spellweaving,
    damage_skill: SkillName::Spellweaving,
    clear_hands_on_cast: false,
    check_cast,
    get_cast_skills,
    get_mana: |w, sp| family(w, sp).map_or(0, |f| f.required_mana),
    do_fizzle,
    do_hurt_fizzle,
    on_disturb,
    on_begin_cast,
    family: Some(&ARCANE_BASE),
    ..SPELL_BASE
};

fn form_info(w: &mut World, sp: SpellId) -> Option<&'static TransformationInfo> {
    let f = family(w, sp)?.transformation?;
    let level = focus_level(w, sp);
    Some(f(w, level))
}

fn arcane_form_check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    helper::transformation_check_cast(w, caster) && (ARCANIST_SPELL.check_cast)(w, sp)
}

fn arcane_form_on_cast(w: &mut World, sp: SpellId) {
    if let (Some(caster), Some(info)) = (spell::caster(w, sp), form_info(w, sp)) {
        helper::transformation_on_cast(w, caster, sp, info);
    }
    spell::finish_sequence(w, sp);
}

/// `ArcaneForm`, abstract.
pub static ARCANE_FORM: SpellType = SpellType {
    full_name: "Server.Spells.Spellweaving.ArcaneForm",
    parent: Some(&ARCANIST_SPELL),
    check_cast: arcane_form_check_cast,
    on_cast: arcane_form_on_cast,
    ..ARCANIST_SPELL
};
