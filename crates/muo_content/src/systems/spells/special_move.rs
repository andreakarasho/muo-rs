//! `SpecialMove`: the Bushido and Ninjitsu special moves (Honorable Execution, Backstab..).
//!
//! A move is a `static` [`SpecialMoveType`] built from [`SPECIAL_MOVE_BASE`]. The current move of each mobile is kept
//! in [`SpecialMoves`].

use indexmap::IndexMap;
use muo_io::text::TextDefinition;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::{messages as mm, stats};
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use super::hooks::hooks;
use super::registry::SpellRegistry;
use crate::core;
use crate::misc::aos_helpers;

/// `AbilityMessage`.
#[derive(Copy, Clone, Debug)]
pub enum AbilityMessage {
    Empty,
    Number(i32),
    Text(&'static str),
}

impl AbilityMessage {
    pub fn definition(self) -> TextDefinition {
        match self {
            Self::Empty => TextDefinition::EMPTY,
            Self::Number(n) => TextDefinition::of_number(n),
            Self::Text(t) => TextDefinition::of_text(t),
        }
    }
}

/// One `static` per special move class.
pub struct SpecialMoveType {
    pub full_name: &'static str,
    pub base_mana: i32,
    /// `Core.ML ? a : b`: the base mana and required skill before ML.
    pub base_mana_pre_ml: Option<i32>,
    pub required_skill_pre_ml: Option<f64>,
    pub move_skill: SkillName,
    pub required_skill: f64,
    pub ability_message: AbilityMessage,
    pub blocked_by_animal_form: bool,
    pub delayed_context: bool,
    pub validates_during_hit: bool,

    /// The checks of a subclass's `Validate(from)` that run before `base.Validate(from)` (and send their own
    /// message): false refuses the move.
    pub pre_validate: fn(&mut World, MobileId) -> bool,
    pub get_accuracy_bonus: fn(&mut World, MobileId) -> i32,
    pub get_damage_scalar: fn(&mut World, MobileId, MobileId) -> f64,
    /// Called before swinging, to make sure the accuracy scalar is to be computed.
    pub on_before_swing: fn(&mut World, MobileId, MobileId) -> bool,
    /// Called when a hit connects, but before damage is calculated.
    pub on_before_damage: fn(&mut World, MobileId, MobileId) -> bool,
    /// Called as soon as the ability is used.
    pub on_use: fn(&mut World, MobileId),
    /// Called when a hit connects, at the end of the `weapon.OnHit()` method.
    pub on_hit: fn(&mut World, MobileId, MobileId, i32),
    /// Called when a hit misses.
    pub on_miss: fn(&mut World, MobileId, MobileId),
    /// Called when the move is cleared.
    pub on_clear_move: fn(&mut World, MobileId),
    pub ignore_armor: fn(&mut World, MobileId) -> bool,
    pub get_property_bonus: fn(&mut World, MobileId) -> f64,
}

impl SpecialMoveType {
    /// `BaseMana` for the era.
    pub fn base_mana_in(&self, ml: bool) -> i32 {
        self.base_mana_pre_ml
            .filter(|_| !ml)
            .unwrap_or(self.base_mana)
    }

    /// `RequiredSkill` for the era.
    pub fn required_skill_in(&self, ml: bool) -> f64 {
        self.required_skill_pre_ml
            .filter(|_| !ml)
            .unwrap_or(self.required_skill)
    }

    /// Same class (move objects are singletons).
    pub fn same(&'static self, other: &'static SpecialMoveType) -> bool {
        std::ptr::eq(self, other)
    }
}

fn nop_m(_: &mut World, _: MobileId) {}

pub static SPECIAL_MOVE_BASE: SpecialMoveType = SpecialMoveType {
    full_name: "Server.Spells.SpecialMove",
    base_mana: 0,
    base_mana_pre_ml: None,
    required_skill_pre_ml: None,
    move_skill: SkillName::Bushido,
    required_skill: 0.0,
    ability_message: AbilityMessage::Empty,
    blocked_by_animal_form: true,
    delayed_context: false,
    validates_during_hit: true,
    pre_validate: |_, _| true,
    get_accuracy_bonus: |_, _| 0,
    get_damage_scalar: |_, _, _| 1.0,
    on_before_swing: |_, _, _| true,
    on_before_damage: |_, _, _| true,
    on_use: nop_m,
    on_hit: |_, _, _, _| {},
    on_miss: |_, _, _| {},
    on_clear_move: nop_m,
    ignore_armor: |_, _| false,
    get_property_bonus: |_, _| 1.0,
};

type DuelAllowSpecialMoveFn = fn(&mut World, MobileId, &'static str, Option<bool>) -> bool;
/// `Table` (the current move per mobile) and `_playersTable` (the contexts that limit how often a move is used).
#[derive(Default)]
pub struct SpecialMoves {
    table: IndexMap<MobileId, &'static SpecialMoveType>,
    contexts: IndexMap<MobileId, SpecialMoveContext>,
    /// `HonorableExecution.IsUnderPenalty(from)`, `AnimalForm.UnderTransformation(from)`: set by those classes.
    pub honorable_execution_penalty: Option<fn(&World, MobileId) -> bool>,
    /// `DuelContext.AllowSpecialMove(from, option, move)`.
    /// `(from, option, Some(is NinjaMove))`: the school is `NinjaMove` or `SamuraiMove`.
    pub duel_allow_special_move: Option<DuelAllowSpecialMoveFn>,
}

/// `SpecialMoveContext`.
#[derive(Copy, Clone)]
pub struct SpecialMoveContext {
    pub timer: TimerHandle,
    pub ty: &'static str,
}

fn state(w: &mut World) -> &mut SpecialMoves {
    w.resources.get_mut::<SpecialMoves>()
}

/// `MindRotSpell.GetMindRotScalar` and `AosAttribute.LowerManaCost` scaling of a move's mana.
pub fn scale_mana(w: &mut World, ty: &'static SpecialMoveType, m: MobileId, mana: i32) -> i32 {
    let mut scalar = hooks(w)
        .and_then(|h| h.mind_rot_scalar)
        .and_then(|f| f(w, m))
        .unwrap_or(1.0);

    // Lower Mana Cost = 40%
    let lmc = aos_helpers::attribute_value(w, m, crate::aos::AosAttribute::LowerManaCost).min(40);
    scalar -= f64::from(lmc) / 100.0;

    let mut total = (f64::from(mana) * scalar) as i32;

    if skills::value(w, m, ty.move_skill) < 50.0 && get_context(w, m).is_some() {
        total *= 2;
    }

    total
}

/// `CheckSkills(m)`.
pub fn check_skills(w: &mut World, ty: &'static SpecialMoveType, m: MobileId) -> bool {
    if skills::value(w, m, ty.move_skill) < ty.required_skill_in(core::ml(w)) {
        // You need at least ~1_SKILL_REQUIREMENT~ ~2_SKILL_NAME~ skill to use that ability.
        let args = format!(
            "{:.1}\t{:?}\t ",
            ty.required_skill_in(core::ml(w)),
            ty.move_skill
        );
        mm::send_localized_message(w, m, 1063013, &args, 0x3B2);
        return false;
    }
    true
}

/// `CheckMana(from, consume)`.
pub fn check_mana(
    w: &mut World,
    ty: &'static SpecialMoveType,
    from: MobileId,
    consume: bool,
) -> bool {
    let mana = scale_mana(w, ty, from, ty.base_mana_in(core::ml(w)));

    if w.mobile(from).map_or(0, |m| m.mana) < mana {
        // You need ~1_MANA_REQUIREMENT~ mana to perform that attack
        mm::send_localized_message(w, from, 1060181, &mana.to_string(), 0x3B2);
        return false;
    }

    if consume {
        if !ty.delayed_context {
            set_context(w, ty, from);
        }
        let cur = w.mobile(from).map_or(0, |m| m.mana);
        muo_server::mobile::part1::props_b::set_mana(w, from, cur - mana); // the Mana setter: delta, regen timer
    }

    true
}

/// `SetContext(from)`.
pub fn set_context(w: &mut World, ty: &'static SpecialMoveType, from: MobileId) {
    if get_context(w, from).is_none()
        && (ty.delayed_context || skills::value(w, from, ty.move_skill) < 50.0)
    {
        let h = timer::start(w, TimeSpan::from_secs(3), TimeSpan::ZERO, 1, move |w| {
            remove_context(w, from)
        });
        state(w).contexts.insert(
            from,
            SpecialMoveContext {
                timer: h,
                ty: ty.full_name,
            },
        );
    }
}

fn remove_context(w: &mut World, m: MobileId) {
    if let Some(mut c) = state(w).contexts.shift_remove(&m) {
        c.timer.cancel(w);
    }
}

fn get_context(w: &World, m: MobileId) -> Option<SpecialMoveContext> {
    w.resources.get::<SpecialMoves>()?.contexts.get(&m).copied()
}

/// `Validate(from)`.
pub fn validate(w: &mut World, ty: &'static SpecialMoveType, from: MobileId) -> bool {
    if !(ty.pre_validate)(w, from) {
        return false;
    }

    if !w.mobile(from).is_some_and(|m| m.player) {
        return true;
    }

    let penalty = w
        .resources
        .get::<SpecialMoves>()
        .and_then(|s| s.honorable_execution_penalty);
    if penalty.is_some_and(|f| f(w, from)) {
        mm::send_localized_message(w, from, 1063024, "", 0x3B2); // You cannot perform this special move right now.
        return false;
    }

    if hooks(w)
        .and_then(|h| h.animal_form_under)
        .is_some_and(|f| f(w, from))
    {
        mm::send_localized_message(w, from, 1063024, "", 0x3B2); // You cannot perform this special move right now.
        return false;
    }

    // The `this switch` of `Validate`: the option name and whether the move is a `NinjaMove` (else `SamuraiMove`).
    let option = match ty.full_name {
        "Server.Spells.Ninjitsu.Backstab" => Some(("Backstab", true)),
        "Server.Spells.Ninjitsu.DeathStrike" => Some(("Death Strike", true)),
        "Server.Spells.Ninjitsu.FocusAttack" => Some(("Focus Attack", true)),
        "Server.Spells.Ninjitsu.KiAttack" => Some(("Ki Attack", true)),
        "Server.Spells.Ninjitsu.SurpriseAttack" => Some(("Surprise Attack", true)),
        "Server.Spells.Bushido.HonorableExecution" => Some(("Honorable Execution", false)),
        "Server.Spells.Bushido.LightningStrike" => Some(("Lightning Strike", false)),
        "Server.Spells.Bushido.MomentumStrike" => Some(("Momentum Strike", false)),
        _ => None,
    };
    if let Some((option, ninja)) = option {
        let allow = w
            .resources
            .get::<SpecialMoves>()
            .and_then(|s| s.duel_allow_special_move);
        if allow.is_some_and(|f| !f(w, from, option, Some(ninja))) {
            return false;
        }
    }

    check_skills(w, ty, from) && check_mana(w, ty, from, false)
}

/// `CheckGain(m)`.
pub fn check_gain(w: &mut World, ty: &'static SpecialMoveType, m: MobileId) {
    stats::check_skill(
        w,
        m,
        ty.move_skill,
        ty.required_skill_in(core::ml(w)),
        ty.required_skill_in(core::ml(w)) + 37.5,
    );
}

/// The `SendToggleSpecialAbility` packet.
pub fn toggle_special_ability_packet(ability_id: i32, active: bool) -> [u8; 8] {
    let mut p = [0u8; 8];
    p[0] = 0xBF; // Packet ID
    p[1..3].copy_from_slice(&8u16.to_be_bytes());
    p[3..5].copy_from_slice(&0x25i16.to_be_bytes());
    p[5..7].copy_from_slice(&(ability_id as i16).to_be_bytes());
    p[7] = u8::from(active);
    p
}

fn send_toggle(w: &mut World, m: MobileId, ability_id: i32, active: bool) {
    if let Some(ns) = w.mobile(m).and_then(|m| m.net_state) {
        w.send(ns, &toggle_special_ability_packet(ability_id, active));
    }
}

/// `ClearAllMoves(m)`.
pub fn clear_all_moves(w: &mut World, m: MobileId) {
    let ids: Vec<i32> = w.resources.get_mut::<SpellRegistry>().special_move_ids();
    for move_id in ids {
        if move_id != -1 {
            send_toggle(w, m, move_id + 1, false);
        }
    }
}

/// The move in the table, as the combat code reads it after `GetCurrentMove` validated it.
pub fn current_move_unchecked(w: &World, m: MobileId) -> Option<&'static SpecialMoveType> {
    w.resources.get::<SpecialMoves>()?.table.get(&m).copied()
}

/// `GetCurrentMove(m)`.
pub fn get_current_move(w: &mut World, m: MobileId) -> Option<&'static SpecialMoveType> {
    if !core::se(w) {
        clear_current_move(w, m);
        return None;
    }

    let mv = state(w).table.get(&m).copied();
    if let Some(mv) = mv
        && mv.validates_during_hit
        && !validate(w, mv, m)
    {
        clear_current_move(w, m);
        return None;
    }
    mv
}

/// `SetCurrentMove(m, move)`.
pub fn set_current_move(w: &mut World, m: MobileId, mv: Option<&'static SpecialMoveType>) -> bool {
    if !core::se(w) {
        clear_current_move(w, m);
        return false;
    }

    if let Some(mv) = mv
        && !validate(w, mv, m)
    {
        clear_current_move(w, m);
        return false;
    }

    let same_move = match (mv, get_current_move(w, m)) {
        (Some(a), Some(b)) => std::ptr::eq(a, b),
        (None, None) => true,
        _ => false,
    };

    clear_current_move(w, m);

    if same_move {
        return true;
    }

    if let Some(mv) = mv {
        if let Some(f) = hooks(w).and_then(|h| h.clear_current_weapon_ability) {
            f(w, m);
        }

        state(w).table.insert(m, mv);

        (mv.on_use)(w, m);

        let move_id = w
            .resources
            .get_mut::<SpellRegistry>()
            .registry_number_of_special(mv);
        if move_id > 0 {
            send_toggle(w, m, move_id + 1, true);
        }

        crate::text::send_message_to(w, Some(&mv.ability_message.definition()), m);
    }

    true
}

/// `ClearCurrentMove(m)`.
pub fn clear_current_move(w: &mut World, m: MobileId) {
    if let Some(mv) = state(w).table.shift_remove(&m) {
        (mv.on_clear_move)(w, m);

        let move_id = w
            .resources
            .get_mut::<SpellRegistry>()
            .registry_number_of_special(mv);
        if move_id > 0 {
            send_toggle(w, m, move_id + 1, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_packet_bytes() {
        assert_eq!(
            toggle_special_ability_packet(3, true),
            [0xBF, 0, 8, 0, 0x25, 0, 3, 1]
        );
        assert_eq!(
            toggle_special_ability_packet(258, false),
            [0xBF, 0, 8, 0, 0x25, 1, 2, 0]
        );
    }
}
