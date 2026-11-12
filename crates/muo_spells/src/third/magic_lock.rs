//! The Magic Lock spell.

use muo_content::items::cont::{self, LOCKABLE_CONTAINER};
use muo_content::spells::hooks::hooks;
use muo_content::spells::magery::{MAGERY_SPELL, THIRD_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::MessageType;
use muo_server::World;
use muo_server::ids::EntityId;
use muo_server::item::effect_item;
use muo_server::mobile::part2::messages as mm;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{item_is, item_of, turn_to_entity};

/// `ILockpickable.CannotPick`.
pub const CANNOT_PICK: i32 = 0;
/// `ILockpickable.MagicLock`.
pub const MAGIC_LOCK: i32 = -255;

/// `MagicLockSpell`: An Por.
pub static MAGIC_LOCK_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Third.MagicLockSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Magic Lock",
        "An Por",
        &[reagent::GARLIC, reagent::BLOODMOSS, reagent::SULFUROUS_ASH],
    )
    .action(215)
    .hand_effect(9001),
    on_cast,
    family: Some(&THIRD_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(item) = item_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let lockable = w.item(item).is_some_and(|i| i.ty.is(&LOCKABLE_CONTAINER));
    if !lockable {
        mm::send_localized_message(w, caster, 501762, "", 0x3B2); // Target must be an unlocked chest.
    } else if hooks(w)
        .and_then(|h| h.house_locked_down_or_secured)
        .is_some_and(|f| f(w, item))
    {
        // You cannot cast this on a locked down item.
        mm::local_overhead_message_number(w, caster, MessageType::REGULAR, 0x22, 501761, "");
    } else if cont::locked(w, item)
        || cont::ext(w, item).is_some_and(|e| e.lock_level == CANNOT_PICK)
        || item_is(w, item, "Server.Items.ParagonChest")
    {
        mm::send_localized_message(w, caster, 501762, "", 0x3B2); // Target must be an unlocked chest.
    } else if spell::check_sequence(w, sp) {
        turn_to_entity(w, caster, EntityId::Item(item));

        let loc = muo_server::item::ops::get_world_location(w, item);
        let map = w.item(item).and_then(|i| i.map);
        let e = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
        muo_server::effects::send_location_particles(w, e, 0x376A, 9, 32, 5020);
        muo_server::effects::play_sound(w, loc, map, 0x1FA);

        // The chest is now locked!
        mm::local_overhead_message_number(w, caster, MessageType::REGULAR, 0x3B2, 501763, "");

        if let Some(e) = cont::ext_mut(w, item) {
            e.lock_level = MAGIC_LOCK; // signal magic lock
        }
        cont::set_locked(w, item, true);
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
