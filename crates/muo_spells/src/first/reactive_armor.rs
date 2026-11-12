//! The Reactive Armor spell.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::creature;
use muo_content::items::weapon_hooks::WeaponHooks;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::helper;
use muo_content::spells::magery::{FIRST_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_io::text::TextDefinition;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::part1::combat;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::mobile::{ResistanceType, base as mb};
use muo_server::mods::{ModId, ResistanceMod};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

/// `ReactiveArmorSpell`: Flam Sanct.
pub static REACTIVE_ARMOR_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.First.ReactiveArmorSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Reactive Armor",
        "Flam Sanct",
        &[
            reagent::GARLIC,
            reagent::SPIDERS_SILK,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(236)
    .hand_effect(9011),
    on_cast,
    check_cast,
    family: Some(&FIRST_CIRCLE),
    ..MAGERY_SPELL
};

/// `_table` and `_t2aTable`.
#[derive(Default)]
struct Tables {
    mods: IndexMap<MobileId, Vec<ModId>>,
    t2a: IndexMap<MobileId, TimerHandle>,
}

fn tables(w: &mut World) -> &mut Tables {
    w.resources.get_mut::<Tables>()
}

/// Sets the weapon hook and the `PlayerDeletedEvent` handler.
pub fn init(w: &mut World) {
    w.resources
        .get_mut::<WeaponHooks>()
        .reactive_armor_handle_melee_hit = Some(handle_melee_hit);
    muo_content::player_mobile::subscribe_deleted(w, end_armor);
}

/// `HasEffect(m)`.
pub fn has_effect(w: &mut World, m: MobileId) -> bool {
    tables(w).t2a.contains_key(&m)
}

/// `RemoveEffect(m)`.
pub fn remove_effect(w: &mut World, m: MobileId) {
    if let Some(mut t) = tables(w).t2a.shift_remove(&m) {
        t.cancel(w);
    }
}

/// `HandleMeleeHit(attacker, defender, ref damage)`: the damage left.
pub fn handle_melee_hit(w: &mut World, attacker: MobileId, defender: MobileId, damage: i32) -> i32 {
    let mut damage = damage;
    if !core::uor(w) {
        // T2A: percentage-based melee reflection
        if !has_effect(w, defender) {
            return damage;
        }

        // Only reflect adjacent melee hits; guards are exempt
        let attacker_loc = w.mobile(attacker).map(|a| a.location);
        let in_range = attacker_loc.is_some_and(|l| mb::in_range_point3d(w, defender, l, 1));
        if !in_range || creature::type_name(w, attacker).ends_with("Guard") {
            return damage;
        }

        let magery = skills::value(w, defender, SkillName::Magery);
        let reflect_pct = (10.0 + magery / 4.0) as i32; // 10-35%
        let reflected = damage * reflect_pct / 100;

        if reflected > 0 {
            damage -= reflected;

            // Sourceless damage; does not trigger attacker's own combat events
            mb::damage(w, attacker, reflected, None, true, false);
        }

        effects::play_sound(w, attacker, 0x1F1);
        effects::fixed_effect(w, attacker, 0x374A, 10, 16);
    } else {
        // UOR: damage absorption pool
        let absorb = w.mobile(defender).map_or(0, |d| d.melee_damage_absorb);
        if absorb <= 0 {
            return damage;
        }

        if absorb > damage {
            let react = (damage / 5).max(1);

            if let Some(d) = w.mobile_mut(defender) {
                d.melee_damage_absorb -= damage;
            }
            damage = 0;

            mb::damage(w, attacker, react, Some(defender), true, false);

            effects::play_sound(w, attacker, 0x1F1);
            effects::fixed_effect(w, attacker, 0x374A, 10, 16);
        } else {
            if let Some(d) = w.mobile_mut(defender) {
                d.melee_damage_absorb = 0;
            }
            mm::send_localized_message(w, defender, 1005556, "", 0x3B2); // Your reactive armor spell has been nullified.
            helper::defensive_spell_nullify(w, defender);
        }
    }
    damage
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    if core::aos(w) || !core::uor(w) {
        return true;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if w.mobile(caster).is_some_and(|c| c.melee_damage_absorb > 0) {
        mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
        return false;
    }
    if !combat::can_begin_action(w, caster, "DefensiveSpell") {
        mm::send_localized_message(w, caster, 1005385, "", 0x3B2); // The spell will not adhere to you at this time.
        return false;
    }
    true
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Targeted::Entity(EntityId::Mobile(m)) = targeted else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if !mb::can_be_beneficial(w, caster, m) {
        return;
    }
    if has_effect(w, m) {
        // This target already has Reactive Armor
        mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
        return;
    }
    if spell::check_b_sequence(w, sp, m, false) {
        mb::do_beneficial(w, caster, m);
        helper::turn(
            w,
            caster,
            Some(helper::Point3dRef::Entity(EntityId::Mobile(m))),
        );
        effects::fixed_particles(w, m, 0x376A, 9, 32, 5008, EffectLayer::Waist);
        effects::play_sound(w, m, 0x1F2);

        let duration =
            TimeSpan::from_secs_f64(25.0 + skills::value(w, caster, SkillName::Magery) / 2.0); // 25-75s
        let token = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
            tables(w).t2a.shift_remove(&m);
        });
        // `_t2aTable[m] = token`: an earlier token is left running.
        tables(w).t2a.insert(m, token);
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if core::aos(w) {
        /* The reactive armor spell increases the caster's physical resistance, while lowering the caster's
         * elemental resistances: 15 + (Inscription/20) physical, -5 elemental; indefinite, deactivated when re-cast.
         */
        if spell::check_sequence(w, sp) {
            if let Some(mods) = tables(w).mods.shift_remove(&caster) {
                effects::play_sound(w, caster, 0x1ED);
                effects::fixed_particles(w, caster, 0x376A, 9, 32, 5008, EffectLayer::Waist);
                for m in mods {
                    mb::remove_resistance_mod(w, caster, m);
                }
                if is_player(w, caster) {
                    remove_buff(w, caster, BuffIcon::ReactiveArmor);
                }
            } else {
                effects::play_sound(w, caster, 0x1E9);
                effects::fixed_particles(w, caster, 0x376A, 9, 32, 5008, EffectLayer::Waist);

                let phys = 15 + (skills::value(w, caster, SkillName::Inscribe) / 20.0) as i32;
                let defs = [
                    (
                        ResistanceType::Physical,
                        "PhysicalResistReactiveArmorSpell",
                        phys,
                    ),
                    (ResistanceType::Fire, "FireResistReactiveArmorSpell", -5),
                    (ResistanceType::Cold, "ColdResistReactiveArmorSpell", -5),
                    (ResistanceType::Poison, "PoisonResistReactiveArmorSpell", -5),
                    (ResistanceType::Energy, "EnergyResistReactiveArmorSpell", -5),
                ];
                let mut ids = Vec::new();
                for (ty, name, offset) in defs {
                    let m = ResistanceMod::new(w, ty, Some(name), offset, None);
                    ids.push(m.id);
                    mb::add_resistance_mod(w, caster, m);
                }
                tables(w).mods.insert(caster, ids);

                if is_player(w, caster) {
                    let args = format!("{phys}\t{}\t{}\t{}\t{}", 5, 5, 5, 5);
                    add_buff(
                        w,
                        caster,
                        BuffInfo::with_secondary(BuffIcon::ReactiveArmor, 1075812, 1075813)
                            .args(TextDefinition::of_text(args)),
                    );
                }
            }
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
        if w.mobile(caster).is_some_and(|c| c.melee_damage_absorb > 0) {
            mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
        } else if !combat::can_begin_action(w, caster, "DefensiveSpell") {
            mm::send_localized_message(w, caster, 1005385, "", 0x3B2); // The spell will not adhere to you at this time.
        } else if spell::check_sequence(w, sp) {
            if combat::begin_action(w, caster, "DefensiveSpell") {
                let sum = skills::value(w, caster, SkillName::Magery)
                    + skills::value(w, caster, SkillName::Meditation)
                    + skills::value(w, caster, SkillName::Inscribe);
                let value = ((sum as i32) / 3).clamp(1, 75);
                if let Some(c) = w.mobile_mut(caster) {
                    c.melee_damage_absorb = value;
                }
                effects::fixed_particles(w, caster, 0x376A, 9, 32, 5008, EffectLayer::Waist);
                effects::play_sound(w, caster, 0x1F2);
            } else {
                mm::send_localized_message(w, caster, 1005385, "", 0x3B2); // The spell will not adhere to you at this time.
            }
        }
        spell::finish_sequence(w, sp);
    }
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
}

/// `EndArmor(m)` on `PlayerDeletedEvent`.
pub fn end_armor(w: &mut World, m: MobileId) {
    remove_effect(w, m);
    let Some(mods) = tables(w).mods.shift_remove(&m) else {
        return;
    };
    for id in mods {
        mb::remove_resistance_mod(w, m, id);
    }
    remove_buff(w, m, BuffIcon::ReactiveArmor);
}
