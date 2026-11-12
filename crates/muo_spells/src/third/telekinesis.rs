//! The Telekinesis spell.

use muo_content::items::cont::{self, TRAPPABLE_CONTAINER};
use muo_content::items::corpse;
use muo_content::spells::hooks::hooks;
use muo_content::spells::magery::{MAGERY_SPELL, THIRD_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_server::World;
use muo_server::effects::send_location_particles;
use muo_server::ids::EntityId;
use muo_server::item::effect_item;
use muo_server::mobile::part2::messages as mm;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{item_is, item_of, turn_to_entity};

/// `TelekinesisSpell`: Ort Por Ylem.
pub static TELEKINESIS_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Third.TelekinesisSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Telekinesis",
        "Ort Por Ylem",
        &[reagent::BLOODMOSS, reagent::MANDRAKE_ROOT],
    )
    .action(203)
    .hand_effect(9031),
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

    // `item as ITelekinesisable`: TrappableContainer, and what the hook knows (BaseDoor, Dice).
    let trappable = w.item(item).is_some_and(|i| i.ty.is(&TRAPPABLE_CONTAINER));
    let hook = hooks(w).and_then(|h| h.item_on_telekinesis);
    let telekinesisable = trappable
        || hooks(w)
            .and_then(|h| h.item_is_telekinesisable)
            .is_some_and(|f| f(w, item));
    let is_container =
        cont::is_base_container(w, item) || item_is(w, item, "Server.Items.Container");

    if !telekinesisable && !is_container {
        mm::send_localized_message(w, caster, 501857, "", 0x3B2); // This spell won't work on that!
        return;
    }

    if spell::check_sequence(w, sp) {
        turn_to_entity(w, caster, EntityId::Item(item));

        if trappable {
            (cont::class(w, item).on_telekinesis)(w, item, caster);
        } else if hook.is_some_and(|f| f(w, item, caster)) {
            // OnTelekinesis ran
        } else if is_container {
            let ty = w.item(item).expect("the target is alive").ty;
            let root = muo_server::item::ops::root_parent(w, item);
            if !(ty.is_accessible_to)(w, item, caster) {
                (ty.on_double_click_not_accessible)(w, item, caster);
            } else if !(ty.check_item_use)(w, item, caster, item) {
                // not usable
            } else if matches!(root, Some(EntityId::Mobile(r)) if r != caster) {
                (ty.on_snoop)(w, item, caster);
            } else if item_is(w, item, "Server.Items.Corpse")
                && !corpse::check_loot(w, item, caster)
            {
                // may not loot
            } else {
                let region = w.mobile(caster).map(|c| c.region);
                let allowed = region.is_none_or(|r| {
                    w.region(r)
                        .map(|x| x.ty)
                        .is_none_or(|t| (t.on_double_click)(w, r, caster, &targeted))
                });
                if allowed {
                    let loc = w.item(item).map(|i| i.location).unwrap_or_default();
                    let map = w.item(item).and_then(|i| i.map);
                    let e =
                        effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
                    send_location_particles(w, e, 0x376A, 9, 32, 5022);
                    muo_server::effects::play_sound(w, loc, map, 0x1F5);
                    (ty.on_item_used)(w, item, caster, item);
                }
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
        SpellTargetKind::Item,
        target,
    );
}
