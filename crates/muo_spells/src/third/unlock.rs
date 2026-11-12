//! The Unlock spell.

use muo_content::items::cont::{self, LOCKABLE_CONTAINER};
use muo_content::spells::helper::{self, Point3dRef};
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
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

use super::magic_lock::{CANNOT_PICK, MAGIC_LOCK};
use crate::common::item_is;

/// `UnlockSpell`: Ex Por.
pub static UNLOCK_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Third.UnlockSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Unlock Spell",
        "Ex Por",
        &[reagent::BLOODMOSS, reagent::SULFUROUS_ASH],
    )
    .action(215)
    .hand_effect(9001),
    on_cast,
    family: Some(&THIRD_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        let p = Point3dRef::from(targeted);
        helper::turn(w, caster, Some(p));

        let loc = helper::point_of(w, p);
        let map = w.mobile(caster).and_then(|c| c.map);
        let e = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
        muo_server::effects::send_location_particles(w, e, 0x376A, 9, 32, 5024);
        muo_server::effects::play_sound(w, loc, map, 0x1FF);

        let item = match targeted {
            Targeted::Entity(EntityId::Item(i)) => Some(i),
            _ => None,
        };
        if matches!(targeted, Targeted::Entity(EntityId::Mobile(_))) {
            // That did not need to be unlocked.
            mm::local_overhead_message_number(w, caster, MessageType::REGULAR, 0x3B2, 503101, "");
            return;
        }
        let Some(cont_id) =
            item.filter(|&i| w.item(i).is_some_and(|x| x.ty.is(&LOCKABLE_CONTAINER)))
        else {
            mm::send_localized_message(w, caster, 501666, "", 0x3B2); // You can't unlock that!
            return;
        };

        let lock_level = cont::ext(w, cont_id).map_or(0, |e| e.lock_level);
        if hooks(w)
            .and_then(|h| h.house_secured)
            .is_some_and(|f| f(w, cont_id))
        {
            mm::send_localized_message(w, caster, 503098, "", 0x3B2); // You cannot cast this on a secure item.
        } else if !cont::locked(w, cont_id) {
            // That did not need to be unlocked.
            mm::local_overhead_message_number(w, caster, MessageType::REGULAR, 0x3B2, 503101, "");
        } else if lock_level == CANNOT_PICK {
            mm::send_localized_message(w, caster, 501666, "", 0x3B2); // You can't unlock that!
        } else {
            let level = (skills::value(w, caster, SkillName::Magery) * 0.8) as i32 - 4;
            let required = cont::ext(w, cont_id).map_or(0, |e| e.required_skill);
            // `cont is TreasureMapChest chest && chest.Level > 2`
            let big_treasure_map_chest = item_is(w, cont_id, "Server.Items.TreasureMapChest")
                && hooks(w)
                    .and_then(|h| h.treasure_map_chest_level)
                    .is_some_and(|f| f(w, cont_id) > 2);

            if level >= required && !big_treasure_map_chest {
                cont::set_locked(w, cont_id, false);

                if lock_level == MAGIC_LOCK
                    && let Some(e) = cont::ext_mut(w, cont_id)
                {
                    e.lock_level = required - 10;
                }
            } else {
                // My spell does not seem to have an effect on that lock.
                mm::local_overhead_message_number(
                    w,
                    caster,
                    MessageType::REGULAR,
                    0x3B2,
                    503099,
                    "",
                );
            }
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
        SpellTargetKind::Point,
        target,
    );
}
