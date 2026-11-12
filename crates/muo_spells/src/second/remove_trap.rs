//! The Remove Trap spell.

use muo_content::items::cont::{TRAPPABLE_CONTAINER, TrapType, ext, ext_mut};
use muo_content::spells::magery::{MAGERY_SPELL, SECOND_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_server::World;
use muo_server::effects::send_location_particles;
use muo_server::ids::EntityId;
use muo_server::item::effect_item;
use muo_server::mobile::part2::messages as mm;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{item_of, turn_to_entity};

/// `RemoveTrapSpell`: An Jux.
pub static REMOVE_TRAP_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Second.RemoveTrapSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Remove Trap",
        "An Jux",
        &[reagent::BLOODMOSS, reagent::SULFUROUS_ASH],
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
        mm::send_localized_message(w, caster, 502373, "", 0x3B2); // That doesn't appear to be trapped
        return;
    }
    let trap = ext(w, item).map_or(TrapType::None, |e| e.trap_type);
    if trap != TrapType::None && trap != TrapType::MagicTrap {
        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        (ty.do_fizzle)(w, sp);
    } else if spell::check_sequence(w, sp) {
        turn_to_entity(w, caster, EntityId::Item(item));

        let loc = muo_server::item::ops::get_world_location(w, item);
        let map = w.item(item).and_then(|i| i.map);
        let e = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
        send_location_particles(w, e, 0x376A, 9, 32, 5015);
        muo_server::effects::play_sound(w, loc, map, 0x1F0);

        if let Some(e) = ext_mut(w, item) {
            e.trap_type = TrapType::None;
            e.trap_power = 0;
            e.trap_level = 0;
        }
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
    if let Some(caster) = spell::caster(w, sp) {
        mm::send_localized_message(w, caster, 502368, "", 0x3B2);
    }
}
