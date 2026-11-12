//! The Protection spell.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::helper;
use muo_content::spells::hooks::hooks_mut;
use muo_content::spells::magery::{MAGERY_SPELL, SECOND_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_io::text::TextDefinition;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::{combat, props_a};
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::mobile::{ResistanceType, base as mb};
use muo_server::mods::{ModId, ResistanceMod, SkillMod};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use crate::common::{is_player_mobile, mobile_of, turn_to};

/// `ProtectionSpell`: Uus Sanct.
pub static PROTECTION_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Second.ProtectionSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Protection",
        "Uus Sanct",
        &[reagent::GARLIC, reagent::GINSENG, reagent::SULFUROUS_ASH],
    )
    .action(236)
    .hand_effect(9011),
    on_cast,
    check_cast,
    family: Some(&SECOND_CIRCLE),
    ..MAGERY_SPELL
};

/// `_table`, `_t2aTable`, `Registry` and the timers of the old-era forms.
#[derive(Default)]
pub struct State {
    table: IndexMap<MobileId, (ModId, ModId)>,
    t2a: IndexMap<MobileId, i32>,
    /// `Registry`: the percentage (x10) of disruption the mobile resists.
    registry: IndexMap<MobileId, i32>,
    timers: IndexMap<MobileId, TimerHandle>,
}

fn state(w: &mut World) -> &mut State {
    w.resources.get_mut::<State>()
}

/// Sets the `ProtectionSpell.Registry` hook and the `PlayerDeletedEvent` handler.
pub fn init(w: &mut World) {
    hooks_mut(w).protection = Some(|w, m| w.resources.get::<State>()?.registry.get(&m).copied());
    muo_content::player_mobile::subscribe_deleted(w, end_protection);
}

/// `HasT2AProtection(m)`.
pub fn has_t2a_protection(w: &mut World, m: MobileId) -> bool {
    state(w).t2a.contains_key(&m)
}

/// `RemoveT2AProtection(m)`.
pub fn remove_t2a_protection(w: &mut World, m: MobileId) {
    if let Some(bonus) = state(w).t2a.shift_remove(&m) {
        let cur = w.mobile(m).map_or(0, |x| x.virtual_armor_mod);
        props_a::set_virtual_armor_mod(w, m, cur - bonus.min(cur));
    }
}

/// `ProtectionSpell.Registry.Remove(m)` (DuelContext.Debuff).
pub fn registry_remove(w: &mut World, m: MobileId) {
    state(w).registry.shift_remove(&m);
}

/// `ApplyT2AProtection(caster, target)`.
pub fn apply_t2a_protection(w: &mut World, caster: MobileId, target: MobileId) {
    if has_t2a_protection(w, target) {
        return;
    }
    let magery = skills::value(w, caster, SkillName::Magery);
    let bonus = (magery / 10.0) as i32;
    let duration = TimeSpan::from_secs_f64(6.0 * magery / 5.0);

    state(w).t2a.insert(target, bonus);
    let cur = w.mobile(target).map_or(0, |x| x.virtual_armor_mod);
    props_a::set_virtual_armor_mod(w, target, cur + bonus);
    effects::fixed_particles(w, target, 0x375A, 9, 20, 5016, EffectLayer::Waist);

    start_timer(w, target, duration);
}

/// `new InternalTimer(mobile, duration).Start()`.
fn start_timer(w: &mut World, m: MobileId, duration: TimeSpan) {
    let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
        if !core::uor(w) {
            remove_t2a_protection(w, m);
        } else {
            state(w).registry.shift_remove(&m);
            helper::defensive_spell_nullify(w, m);
        }
    });
    state(w).timers.insert(m, h);
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    if core::aos(w) || !core::uor(w) {
        return true;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if state(w).registry.contains_key(&caster) {
        mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
        return false;
    }
    if combat::can_begin_action(w, caster, "DefensiveSpell") {
        return true;
    }
    mm::send_localized_message(w, caster, 1005385, "", 0x3B2); // The spell will not adhere to you at this time.
    false
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if !mb::can_be_beneficial(w, caster, m) {
        return;
    }
    if has_t2a_protection(w, m) {
        mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
        return;
    }
    if spell::check_b_sequence(w, sp, m, false) {
        mb::do_beneficial(w, caster, m);
        turn_to(w, caster, m);
        apply_t2a_protection(w, caster, m);
        effects::play_sound(w, m, 0x1ED);
    }
}

/// `Toggle(caster, target)`: AOS+ only.
pub fn toggle(w: &mut World, caster: MobileId, target: MobileId) {
    /* Players under the protection spell effect can no longer have their spells "disrupted" when hit. They have
     * decreased physical resistance, a decreased "resisting spells" skill value and a slower casting speed. The spell
     * has an indefinite duration, becoming active when cast, and deactivated when re-cast.
     */
    if let Some((phys, resist)) = state(w).table.shift_remove(&target) {
        effects::play_sound(w, target, 0x1ED);
        effects::fixed_particles(w, target, 0x375A, 9, 20, 5016, EffectLayer::Waist);
        state(w).registry.shift_remove(&target);
        mb::remove_resistance_mod(w, target, phys);
        mb::remove_skill_mod(w, target, resist);
        if is_player_mobile(w, target) {
            remove_buff(w, target, BuffIcon::Protection);
        }
    } else {
        effects::play_sound(w, target, 0x1E9);
        effects::fixed_particles(w, target, 0x375A, 9, 20, 5016, EffectLayer::Waist);

        let inscribe = skills::value(w, caster, SkillName::Inscribe);
        let phys_loss = -15 + (inscribe / 20.0) as i32;
        let resist_loss = -35 + (inscribe / 20.0) as i32;
        let phys_mod = ResistanceMod::new(
            w,
            ResistanceType::Physical,
            Some("PhysicalResistProtectionSpell"),
            phys_loss,
            None,
        );
        let resist_mod = SkillMod::new_default(
            w,
            SkillName::MagicResist,
            Some("MagicResistProtectionSpell"),
            true,
            f64::from(resist_loss),
        );
        state(w).table.insert(target, (phys_mod.id, resist_mod.id));
        state(w).registry.insert(target, 1000); // 100.0% protection from disruption
        mb::add_resistance_mod(w, target, phys_mod);
        mb::add_skill_mod(w, target, resist_mod);

        if is_player_mobile(w, target) {
            let args = format!("{phys_loss}\t{resist_loss}");
            add_buff(
                w,
                target,
                BuffInfo::with_secondary(BuffIcon::Protection, 1075814, 1075815)
                    .args(TextDefinition::of_text(args)),
            );
        }
    }
}

/// `EndProtection(m)` on `PlayerDeletedEvent`.
pub fn end_protection(w: &mut World, m: MobileId) {
    remove_t2a_protection(w, m);
    let Some((phys, resist)) = state(w).table.shift_remove(&m) else {
        return;
    };
    state(w).registry.shift_remove(&m);
    mb::remove_resistance_mod(w, m, phys);
    mb::remove_skill_mod(w, m, resist);
    remove_buff(w, m, BuffIcon::Protection);
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if core::aos(w) {
        if spell::check_sequence(w, sp) {
            toggle(w, caster, caster);
        }
        spell::finish_sequence(w, sp);
    } else if !core::uor(w) {
        begin_spell_target(
            w,
            sp,
            false,
            TargetFlags::BENEFICIAL,
            false,
            SpellTargetKind::Mobile,
            target,
        );
    } else {
        if state(w).registry.contains_key(&caster) {
            mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
        } else if !combat::can_begin_action(w, caster, "DefensiveSpell") {
            mm::send_localized_message(w, caster, 1005385, "", 0x3B2); // The spell will not adhere to you at this time.
        } else if spell::check_sequence(w, sp) {
            if combat::begin_action(w, caster, "DefensiveSpell") {
                let value = (skills::value(w, caster, SkillName::EvalInt)
                    + skills::value(w, caster, SkillName::Meditation)
                    + skills::value(w, caster, SkillName::Inscribe))
                    * 10.0
                    / 4.0;
                // `Registry.Add` throws on a duplicate; the check above makes that unreachable.
                state(w)
                    .registry
                    .insert(caster, (value as i32).clamp(0, 750)); // 75.0% protection from disruption
                let secs = (skills::value(w, caster, SkillName::Magery) * 2.0).clamp(15.0, 240.0);
                start_timer(w, caster, TimeSpan::from_secs_f64(secs));
                effects::fixed_particles(w, caster, 0x375A, 9, 20, 5016, EffectLayer::Waist);
                effects::play_sound(w, caster, 0x1ED);
            } else {
                mm::send_localized_message(w, caster, 1005385, "", 0x3B2); // The spell will not adhere to you at this time.
            }
        }
        spell::finish_sequence(w, sp);
    }
}
