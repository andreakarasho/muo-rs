//! The Dispel Field spell.

use muo_content::spells::hooks::hooks;
use muo_content::spells::magery::{FIFTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_server::World;
use muo_server::effects::send_location_particles;
use muo_server::ids::EntityId;
use muo_server::item::{base, effect_item, ops};
use muo_server::mobile::part2::messages as mm;
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{item_of, turn_to_entity};

/// `DispelFieldSpell`: An Grav.
pub static DISPEL_FIELD_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fifth.DispelFieldSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::field_range),
    info: SpellInfo::new(
        "Dispel Field",
        "An Grav",
        &[
            reagent::BLACK_PEARL,
            reagent::SPIDERS_SILK,
            reagent::SULFUROUS_ASH,
            reagent::GARLIC,
        ],
    )
    .action(206)
    .hand_effect(9002),
    on_cast,
    family: Some(&FIFTH_CIRCLE),
    ..MAGERY_SPELL
};

/// The classes marked `[DispellableField]` (exact class: the attribute is read with `inherit: false`).
pub const DISPELLABLE_FIELDS: &[&str] = &[
    "Server.Spells.Third.WallOfStone",
    "Server.Spells.Fourth.FireFieldItem",
    "Server.Spells.Fifth.PoisonField",
    "Server.Spells.Sixth.ParalyzeField",
    "Server.Spells.Seventh.EnergyField",
    "Server.Spells.Seventh.GateTravelMoongate",
    "Server.Items.Moongate",
];

// `[DispellableField]` is read with `inherit: false`: exact classes only.
fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(item) = item_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let name = w.item(item).map_or("", |i| i.ty.full_name);
    if !DISPELLABLE_FIELDS.contains(&name) {
        mm::send_localized_message(w, caster, 1005049, "", 0x3B2); // That cannot be dispelled.
    } else if hooks(w)
        .and_then(|h| h.moongate_not_dispellable)
        .is_some_and(|f| f(w, item))
    {
        mm::send_localized_message(w, caster, 1005047, "", 0x3B2); // That magic is too chaotic
    } else if spell::check_sequence(w, sp) {
        turn_to_entity(w, caster, EntityId::Item(item));

        let (loc, map) = w
            .item(item)
            .map(|i| (i.location, i.map))
            .unwrap_or_default();
        let e = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
        send_location_particles(w, e, 0x376A, 9, 20, 5042);
        let wl = ops::get_world_location(w, item);
        muo_server::effects::play_sound(w, wl, map, 0x201);

        base::delete(w, item);
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
