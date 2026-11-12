//! The Magic Trap spell.

use muo_content::core;
use muo_content::items::cont::{TRAPPABLE_CONTAINER, TrapType, ext, ext_mut};
use muo_content::spells::magery::{MAGERY_SPELL, SECOND_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::{Point3D, utility};
use muo_server::World;
use muo_server::effects::send_location_particles;
use muo_server::ids::EntityId;
use muo_server::item::effect_item;
use muo_server::mobile::part2::messages as mm;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{item_of, turn_to_entity};

/// `MagicTrapSpell`: In Jux.
pub static MAGIC_TRAP_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Second.MagicTrapSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Magic Trap",
        "In Jux",
        &[
            reagent::GARLIC,
            reagent::SPIDERS_SILK,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(212)
    .hand_effect(9001),
    on_cast,
    family: Some(&SECOND_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(item) = item_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let is_trappable = w.item(item).is_some_and(|i| i.ty.is(&TRAPPABLE_CONTAINER));
    if !is_trappable {
        mm::send_localized_message(w, caster, 502942, "", 0x3B2); // You can't trap this!
        return;
    }
    let trap = ext(w, item).map_or(TrapType::None, |e| e.trap_type);
    if trap != TrapType::None && trap != TrapType::MagicTrap {
        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        (ty.do_fizzle)(w, sp);
    } else if spell::check_sequence(w, sp) {
        turn_to_entity(w, caster, EntityId::Item(item));

        let power = if core::aos(w) {
            utility::random_min_max(&mut w.rng, 10, 50)
        } else {
            1
        };
        if let Some(e) = ext_mut(w, item) {
            e.trap_type = TrapType::MagicTrap;
            e.trap_power = power;
            e.trap_level = 0;
        }

        let loc = muo_server::item::ops::get_world_location(w, item);
        let map = w.item(item).and_then(|i| i.map);
        for (dx, dy) in [(1, 0), (0, -1), (-1, 0), (0, 1)] {
            let p = Point3D::new(loc.x + dx, loc.y + dy, loc.z);
            let e = effect_item::create_targeted(w, p, map, effect_item::DEFAULT_DURATION);
            send_location_particles(w, e, 0x376A, 9, 10, 9502);
        }
        let e = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
        send_location_particles(w, e, 0, 0, 0, 5014);
        muo_server::effects::play_sound(w, loc, map, 0x1EF);
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    begin_spell_target(
        w,
        sp,
        false,
        TargetFlags::NONE,
        false,
        SpellTargetKind::Item,
        target,
    );
}
