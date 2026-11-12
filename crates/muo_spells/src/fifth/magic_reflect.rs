//! The Magic Reflect spell.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::magery::{FIFTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::{SpellInfo, reagent};
use muo_io::text::TextDefinition;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::combat;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::mobile::{ResistanceType, base as mb};
use muo_server::mods::{ModId, ResistanceMod};
use muo_server::skills::{self, SkillName};

use crate::common::is_player_mobile;

/// `MagicReflectSpell`: In Jux Sanct.
pub static MAGIC_REFLECT_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fifth.MagicReflectSpell",
    parent: Some(&MAGERY_SPELL),
    info: SpellInfo::new(
        "Magic Reflection",
        "In Jux Sanct",
        &[
            reagent::GARLIC,
            reagent::MANDRAKE_ROOT,
            reagent::SPIDERS_SILK,
        ],
    )
    .action(242)
    .hand_effect(9012),
    on_cast,
    check_cast,
    family: Some(&FIFTH_CIRCLE),
    ..MAGERY_SPELL
};

#[derive(Default)]
struct Table(IndexMap<MobileId, Vec<ModId>>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, Vec<ModId>> {
    &mut w.resources.get_mut::<Table>().0
}

/// The `PlayerDeletedEvent` handler.
pub fn init(w: &mut World) {
    muo_content::player_mobile::subscribe_deleted(w, end_reflect);
}

fn absorb(w: &World, m: MobileId) -> i32 {
    w.mobile(m).map_or(0, |x| x.magic_damage_absorb)
}

fn set_absorb(w: &mut World, m: MobileId, v: i32) {
    if let Some(x) = w.mobile_mut(m) {
        x.magic_damage_absorb = v;
    }
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    if core::aos(w) {
        return true;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if absorb(w, caster) > 0 {
        mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
        return false;
    }
    if !core::uor(w) {
        return true;
    }
    if !combat::can_begin_action(w, caster, "DefensiveSpell") {
        mm::send_localized_message(w, caster, 1005385, "", 0x3B2); // The spell will not adhere to you at this time.
        return false;
    }
    true
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if core::aos(w) {
        /* The magic reflection spell decreases the caster's physical resistance, while increasing the caster's
         * elemental resistances. Physical decrease = 25 - (Inscription/20), elemental resistance = +10. It has an
         * indefinite duration, becoming active when cast, and deactivated when re-cast.
         */
        if spell::check_sequence(w, sp) {
            if let Some(mods) = table(w).shift_remove(&caster) {
                effects::play_sound(w, caster, 0x1ED);
                effects::fixed_particles(w, caster, 0x375A, 10, 15, 5037, EffectLayer::Waist);
                for m in mods {
                    mb::remove_resistance_mod(w, caster, m);
                }
                if is_player_mobile(w, caster) {
                    remove_buff(w, caster, BuffIcon::MagicReflection);
                }
            } else {
                effects::play_sound(w, caster, 0x1E9);
                effects::fixed_particles(w, caster, 0x375A, 10, 15, 5037, EffectLayer::Waist);

                let physi_mod = -25 + (skills::value(w, caster, SkillName::Inscribe) / 20.0) as i32;
                const OTHER_MOD: i32 = 10;
                let defs = [
                    (
                        ResistanceType::Physical,
                        "PhysicalResistMagicResist",
                        physi_mod,
                    ),
                    (ResistanceType::Fire, "FireResistMagicResist", OTHER_MOD),
                    (ResistanceType::Cold, "ColdResistMagicResist", OTHER_MOD),
                    (ResistanceType::Poison, "PoisonResistMagicResist", OTHER_MOD),
                    (ResistanceType::Energy, "EnergyResistMagicResist", OTHER_MOD),
                ];
                let mut ids = Vec::new();
                for (ty, name, offset) in defs {
                    let m = ResistanceMod::new(w, ty, Some(name), offset, None);
                    ids.push(m.id);
                    mb::add_resistance_mod(w, caster, m);
                }
                table(w).insert(caster, ids);

                if is_player_mobile(w, caster) {
                    let fmt = format!(
                        "{physi_mod}\t+{OTHER_MOD}\t+{OTHER_MOD}\t+{OTHER_MOD}\t+{OTHER_MOD}"
                    );
                    add_buff(
                        w,
                        caster,
                        BuffInfo::new(BuffIcon::MagicReflection, 1075817)
                            .args(TextDefinition::of_text(fmt))
                            .retain_through_death(),
                    );
                }
            }
        }
    } else if !core::uor(w) {
        if absorb(w, caster) > 0 {
            mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
        } else if spell::check_sequence(w, sp) {
            set_absorb(w, caster, 1);
            effects::fixed_particles(w, caster, 0x375A, 10, 15, 5037, EffectLayer::Waist);
            effects::play_sound(w, caster, 0x1E9);
        }
    } else if absorb(w, caster) > 0 {
        mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
    } else if !combat::can_begin_action(w, caster, "DefensiveSpell") {
        mm::send_localized_message(w, caster, 1005385, "", 0x3B2); // The spell will not adhere to you at this time.
    } else if spell::check_sequence(w, sp) {
        if combat::begin_action(w, caster, "DefensiveSpell") {
            let value = (skills::value(w, caster, SkillName::Magery)
                + skills::value(w, caster, SkillName::Inscribe)) as i32;
            let value = (8.0 + f64::from(value) / 200.0 * 7.0) as i32; // absorb from 8 to 15 "circles"
            set_absorb(w, caster, value);
            effects::fixed_particles(w, caster, 0x375A, 10, 15, 5037, EffectLayer::Waist);
            effects::play_sound(w, caster, 0x1E9);
        } else {
            mm::send_localized_message(w, caster, 1005385, "", 0x3B2); // The spell will not adhere to you at this time.
        }
    }

    spell::finish_sequence(w, sp);
}

/// `EndReflect(m)` on `PlayerDeletedEvent`.
pub fn end_reflect(w: &mut World, m: MobileId) {
    let Some(mods) = table(w).shift_remove(&m) else {
        return;
    };
    for id in mods {
        mb::remove_resistance_mod(w, m, id);
    }
    remove_buff(w, m, BuffIcon::MagicReflection);
}
