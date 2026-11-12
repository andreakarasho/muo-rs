//! Honorable Execution, Lightning Strike and Momentum Strike: the three Bushido special moves.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::items::weapon;
use muo_content::items::weapon_combat;
use muo_content::items::weapon_hooks::{self, WeaponHooks};
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff};
use muo_content::spells::special_move::{
    AbilityMessage, SpecialMoveType, SpecialMoves, check_mana, clear_current_move, set_context,
    validate,
};
use muo_io::text::TextDefinition;
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::mobile::{MobileDelta, ResistanceType, base as mb};
use muo_server::mods::{ResistanceMod, SkillMod};
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use super::samurai::{SAMURAI_MOVE, samurai_check_gain};
use crate::common::{is_player_mobile, mobiles_in_range};

// ---- HonorableExecution ----

/// `HonorableExecution`.
pub static HONORABLE_EXECUTION: SpecialMoveType = SpecialMoveType {
    full_name: "Server.Spells.Bushido.HonorableExecution",
    base_mana: 0,
    required_skill: 25.0,
    // You better kill your enemy with your next hit or you'll be rather sorry...
    ability_message: AbilityMessage::Number(1063122),
    get_damage_scalar: |w, attacker, _| {
        // TODO: 20 -> Perfection
        1.0 + skills::value_ref(w, attacker, SkillName::Bushido) * 20.0 / 10000.0
    },
    on_hit: he_on_hit,
    ..SAMURAI_MOVE
};

struct HeEntry {
    timer: TimerHandle,
    swing_bonus: i32,
    penalty: bool,
}

#[derive(Default)]
struct HeTable(IndexMap<MobileId, HeEntry>);

fn he_table(w: &mut World) -> &mut IndexMap<MobileId, HeEntry> {
    &mut w.resources.get_mut::<HeTable>().0
}

const HE_RESISTS: [(ResistanceType, &str); 5] = [
    (ResistanceType::Physical, "PhysicalResistHonorableExecution"),
    (ResistanceType::Fire, "FireResistHonorableExecution"),
    (ResistanceType::Cold, "ColdResistHonorableExecution"),
    (ResistanceType::Poison, "PoisonResistHonorableExecution"),
    (ResistanceType::Energy, "EnergyResistHonorableExecution"),
];

fn he_on_hit(w: &mut World, attacker: MobileId, defender: MobileId, _damage: i32) {
    if !validate(w, &HONORABLE_EXECUTION, attacker)
        || !check_mana(w, &HONORABLE_EXECUTION, attacker, true)
    {
        return;
    }

    clear_current_move(w, attacker);

    remove_penalty(w, attacker);

    let timer_entry;

    if !super::super::common::alive(w, defender) {
        effects::fixed_particles(w, attacker, 0x373A, 1, 17, 0x7E2, EffectLayer::Waist);

        let mut bushido = skills::value(w, attacker, SkillName::Bushido);
        bushido *= bushido;

        let hits = w.mobile(attacker).map_or(0, |a| a.hits);
        props_b::set_hits(w, attacker, hits + 20 + (bushido / 480.0) as i32);

        let swing_bonus = 1.max((bushido / 720.0) as i32);

        let h = start_he_timer(w, attacker, TimeSpan::from_secs(20));
        timer_entry = HeEntry {
            timer: h,
            swing_bonus,
            penalty: false,
        };

        if is_player_mobile(w, attacker) {
            add_buff(
                w,
                attacker,
                BuffInfo::with_secondary(BuffIcon::HonorableExecution, 1060595, 1153807)
                    .duration(TimeSpan::from_secs(20))
                    .args(TextDefinition::of_text(swing_bonus.to_string())),
            );
        }
    } else {
        for (ty, name) in HE_RESISTS {
            let m = ResistanceMod::new(w, ty, Some(name), -40, None);
            mb::add_resistance_mod(w, attacker, m);
        }

        let res_spells = skills::value(w, attacker, SkillName::MagicResist);
        if res_spells > 0.0 {
            let sm = SkillMod::new_default(
                w,
                SkillName::MagicResist,
                Some("MagicResistHonorableExecution"),
                true,
                -res_spells,
            );
            mb::add_skill_mod(w, attacker, sm);
        }

        let h = start_he_timer(w, attacker, TimeSpan::from_secs(7));
        timer_entry = HeEntry {
            timer: h,
            swing_bonus: 0,
            penalty: true,
        };

        if is_player_mobile(w, attacker) {
            if core::hs(w) {
                add_buff(
                    w,
                    attacker,
                    BuffInfo::with_secondary(BuffIcon::HonorableExecution, 1060595, 1153808)
                        .duration(TimeSpan::from_secs(7))
                        .args(TextDefinition::of_text(format!(
                            "{res_spells}\t{}\t{}\t{}\t{}\t{}",
                            40, 40, 40, 40, 40
                        ))),
                );
            } else {
                add_buff(
                    w,
                    attacker,
                    BuffInfo::new(BuffIcon::HonorableExecution, 1060595)
                        .duration(TimeSpan::from_secs(7)),
                );
            }
        }
    }

    he_table(w).insert(attacker, timer_entry);

    mb::delta(w, attacker, MobileDelta::WEAPON_DAMAGE);
    samurai_check_gain(w, &HONORABLE_EXECUTION, attacker);
}

fn start_he_timer(w: &mut World, from: MobileId, duration: TimeSpan) -> TimerHandle {
    timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
        mb::delta(w, from, MobileDelta::WEAPON_DAMAGE);
        remove_penalty(w, from);
    })
}

/// `GetSwingBonus(target)`.
pub fn get_swing_bonus(w: &World, target: MobileId) -> i32 {
    w.resources
        .get::<HeTable>()
        .and_then(|t| t.0.get(&target))
        .map_or(0, |e| e.swing_bonus)
}

/// `IsUnderPenalty(target)`.
pub fn is_under_penalty(w: &World, target: MobileId) -> bool {
    w.resources
        .get::<HeTable>()
        .and_then(|t| t.0.get(&target))
        .is_some_and(|e| e.penalty)
}

/// `RemovePenalty(target)`.
pub fn remove_penalty(w: &mut World, target: MobileId) {
    if let Some(mut e) = he_table(w).shift_remove(&target) {
        // `timer.Clear()`
        e.timer.cancel(w);
        for (_, name) in HE_RESISTS {
            mb::remove_resistance_mod_by_name(w, target, name);
        }
        mb::remove_skill_mod_by_name(w, target, "MagicResistHonorableExecution");
    }
}

// ---- LightningStrike ----

/// `LightningStrike`.
pub static LIGHTNING_STRIKE: SpecialMoveType = SpecialMoveType {
    full_name: "Server.Spells.Bushido.LightningStrike",
    base_mana: 5,
    required_skill: 50.0,
    ability_message: AbilityMessage::Number(1063167), // You prepare to strike quickly.
    delayed_context: true,
    validates_during_hit: false,
    get_accuracy_bonus: |_, _| 50,
    ignore_armor: |w, attacker| {
        let bushido = skills::value(w, attacker, SkillName::Bushido);
        let critical_chance = bushido * bushido / 72000.0;
        critical_chance >= utility::random_double(&mut w.rng)
    },
    on_before_swing: |w, attacker, _| {
        /* no mana drain before actual hit */
        let _enough_mana = check_mana(w, &LIGHTNING_STRIKE, attacker, false);
        ls_validate(w, attacker)
    },
    on_hit: |w, attacker, defender, _| {
        clear_current_move(w, attacker);

        if check_mana(w, &LIGHTNING_STRIKE, attacker, true) {
            mm::send_localized_message(w, attacker, 1063168, "", 0x3B2); // You attack with lightning precision!
            mm::send_localized_message(w, defender, 1063169, "", 0x3B2); // Your opponent's quick strike causes extra damage!

            effects::fixed_particles_hue(
                w,
                defender,
                0x3818,
                1,
                11,
                0x13A8,
                0,
                0,
                EffectLayer::Waist,
            );
            effects::play_sound(w, defender, 0x51D);

            samurai_check_gain(w, &LIGHTNING_STRIKE, attacker);
            set_context(w, &LIGHTNING_STRIKE, attacker);
        }
    },
    // `Validate` sets `ExecutesLightningStrike` on success, and `SetCurrentMove` validates before `OnUse`.
    on_use: ls_set_executes,
    on_clear_move: |w, attacker| {
        if is_player_mobile(w, attacker)
            && let Some(d) = muo_content::player_mobile::data_mut(w, attacker)
        {
            d.executes_lightning_strike = 0;
        }
    },
    ..SAMURAI_MOVE
};

/// `Validate(from)` of Lightning Strike: the base, then the player's `ExecutesLightningStrike = BaseMana`.
fn ls_validate(w: &mut World, from: MobileId) -> bool {
    let is_valid = validate(w, &LIGHTNING_STRIKE, from);
    if is_valid {
        ls_set_executes(w, from);
    }
    is_valid
}

fn ls_set_executes(w: &mut World, from: MobileId) {
    if is_player_mobile(w, from)
        && let Some(d) = muo_content::player_mobile::data_mut(w, from)
    {
        d.executes_lightning_strike = LIGHTNING_STRIKE.base_mana;
    }
}

// ---- MomentumStrike ----

/// `MomentumStrike`.
pub static MOMENTUM_STRIKE: SpecialMoveType = SpecialMoveType {
    full_name: "Server.Spells.Bushido.MomentumStrike",
    base_mana: 10,
    required_skill: 70.0,
    ability_message: AbilityMessage::Number(1070757), // You prepare to strike two enemies with one blow.
    on_hit: ms_on_hit,
    ..SAMURAI_MOVE
};

fn ms_on_hit(w: &mut World, attacker: MobileId, defender: MobileId, _damage: i32) {
    if !validate(w, &MOMENTUM_STRIKE, attacker) || !check_mana(w, &MOMENTUM_STRIKE, attacker, false)
    {
        return;
    }

    clear_current_move(w, attacker);

    let wp = weapon_hooks::weapon_item(w, attacker);
    let range = wp.map_or(1, |i| weapon::max_range(w, i));

    let (loc, map) = w
        .mobile(attacker)
        .map(|a| (a.location, a.map))
        .unwrap_or_default();
    let mut queue = Vec::new();
    for m in mobiles_in_range(w, map, loc, range) {
        if m != defender && w.mobile(m).is_some_and(|x| x.combatant == Some(attacker)) {
            queue.push(m);
        }
    }

    if queue.is_empty() {
        mm::send_localized_message(w, attacker, 1063123, "", 0x3B2); // There are no valid targets to attack!
        return;
    }

    if !check_mana(w, &MOMENTUM_STRIKE, attacker, true) {
        return;
    }

    let target = queue[utility::random(&mut w.rng, queue.len() as i32) as usize];

    let mut damage_bonus = skills::value(w, attacker, SkillName::Bushido) / 100.0;

    if !super::super::common::alive(w, defender) {
        damage_bonus *= 1.5;
    }

    mm::send_localized_message(w, attacker, 1063171, "", 0x3B2); // You transfer the momentum of your weapon into another enemy!
    mm::send_localized_message(w, target, 1063172, "", 0x3B2); // You were hit by the momentum of a Samurai's weapon!

    effects::fixed_particles_hue(w, target, 0x37B9, 1, 4, 0x251D, 0, 0, EffectLayer::Waist);
    effects::play_sound(w, attacker, 0x510);

    if let Some(wp) = wp {
        weapon_combat::swing(w, wp, attacker, target, damage_bonus);
    }

    // MomentumStrike.CheckGain: `m.CheckSkill(MoveSkill, RequiredSkill, 120.0)`
    muo_server::mobile::part2::stats::check_skill(
        w,
        attacker,
        SkillName::Bushido,
        MOMENTUM_STRIKE.required_skill,
        120.0,
    );
}

/// Sets the Honorable Execution hooks.
pub fn init(w: &mut World) {
    let h = w.resources.get_mut::<WeaponHooks>();
    h.honorable_execution_swing_bonus = Some(get_swing_bonus);
    h.honorable_execution_remove_penalty = Some(remove_penalty);
    h.honorable_execution_is_under_penalty = Some(is_under_penalty);
    w.resources
        .get_mut::<SpecialMoves>()
        .honorable_execution_penalty = Some(is_under_penalty);
}
