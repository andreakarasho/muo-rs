//! The Poison Field spell and the `PoisonField` item.

use muo_content::core;
use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::magery::{FIFTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_data::expansion::Expansion;
use muo_primitives::{Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::mobile::ApplyPoisonResult;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::effects;
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{creature_on_harmful_spell, poison_by_level, reveal_caster_if, surface_top};
use crate::fields::{self, FieldClass, data};

/// `PoisonFieldSpell`: In Nox Grav.
pub static POISON_FIELD_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fifth.PoisonFieldSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::field_range),
    info: SpellInfo::new(
        "Poison Field",
        "In Nox Grav",
        &[
            reagent::BLACK_PEARL,
            reagent::NIGHTSHADE,
            reagent::SPIDERS_SILK,
        ],
    )
    .action(230)
    .hand_effect(9052)
    .allow_town(false),
    on_cast,
    family: Some(&FIFTH_CIRCLE),
    ..MAGERY_SPELL
};

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if helper::check_town(w, Point3dRef::from(targeted), caster) && spell::check_sequence(w, sp) {
        helper::turn(w, caster, Some(Point3dRef::from(targeted)));
        let loc = surface_top(w, targeted);
        let from = w.mobile(caster).map(|c| c.location).unwrap_or_default();
        let map = w.mobile(caster).and_then(|c| c.map);
        let east_to_west = helper::get_east_to_west(from, loc);

        muo_server::effects::play_sound(w, loc, map, 0x20B);

        let item_id = if east_to_west { 0x3915 } else { 0x3922 };

        let exp = core::expansion(w);
        let magery = skills::value(w, caster, SkillName::Magery);
        let duration = if exp == Expansion::None {
            TimeSpan::from_secs(20)
        } else if exp < Expansion::LBR {
            TimeSpan::from_secs_f64(15.0 + magery * 0.4)
        } else {
            TimeSpan::from_secs_f64(3.0 + magery * 0.4)
        };

        for i in -2..=2 {
            let target_loc = Point3D::new(
                if east_to_west { loc.x + i } else { loc.x },
                if east_to_west { loc.y } else { loc.y + i },
                loc.z,
            );
            fields::new_field(
                w,
                &POISON_FIELD_CLASS,
                item_id,
                target_loc,
                caster,
                map,
                duration,
                i,
                0,
            );
        }
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    begin_spell_target(
        w,
        sp,
        true,
        TargetFlags::NONE,
        false,
        SpellTargetKind::Point,
        target,
    );
}

/// `ApplyPoisonTo(m)`.
fn apply_poison_to(w: &mut World, item: ItemId, m: MobileId) {
    let Some(caster) = data(w, item).and_then(|d| d.caster) else {
        return;
    };

    let level = if core::aos(w) {
        let total = skills::value(w, caster, SkillName::Magery)
            + skills::value(w, caster, SkillName::Poisoning);
        if total > 199.8 {
            3
        } else if total > 170.2 {
            2
        } else if total > 130.2 {
            1
        } else {
            0
        }
    } else {
        1
    };

    if let Some(p) = poison_by_level(w, level) {
        let r = mb::apply_poison(w, m, Some(caster), p);
        if matches!(
            r,
            ApplyPoisonResult::Poisoned | ApplyPoisonResult::HigherPoisonActive
        ) {
            reveal_caster_if(w, m, caster);
        }
    }

    creature_on_harmful_spell(w, m, caster);
}

fn on_move_over(w: &mut World, id: ItemId, m: MobileId) -> bool {
    let visible = w.item(id).is_some_and(|i| i.visible());
    let caster = data(w, id).and_then(|d| d.caster);
    if visible
        && let Some(caster) = caster
        && (!core::aos(w) || m != caster)
        && helper::valid_indirect_target(w, caster, m)
        && mb::can_be_harmful_msg(w, caster, m, false)
    {
        mb::do_harmful(w, caster, m, false);

        apply_poison_to(w, id, m);
        effects::play_sound(w, m, 0x474);
    }
    true
}

fn on_tick(w: &mut World, id: ItemId, caster: MobileId) {
    let Some((map, loc, item_id)) = w.item(id).map(|i| (i.map, i.location, i.item_id)) else {
        return;
    };
    let east_to_west = item_id == 0x3915;
    let bounds = Rectangle2D::new(
        loc.x - if east_to_west { 0 } else { 1 },
        loc.y - if east_to_west { 1 } else { 0 },
        if east_to_west { 1 } else { 2 },
        if east_to_west { 2 } else { 1 },
    );
    let queue: Vec<MobileId> = muo_server::map::enumerate::mobiles_in_bounds(w, map, bounds, false)
        .filter(|&m| {
            w.mobile(m)
                .is_some_and(|x| x.location.z + 16 > loc.z && loc.z + 12 > x.location.z)
        })
        .collect();
    for m in queue {
        if (!core::aos(w) || m != caster)
            && helper::valid_indirect_target(w, caster, m)
            && mb::can_be_harmful_msg(w, caster, m, false)
        {
            mb::do_harmful(w, caster, m, false);
            apply_poison_to(w, id, m);
            effects::play_sound(w, m, 0x474);
        }
    }
}

static POISON_FIELD_CLASS: FieldClass = FieldClass {
    ty: &POISON_FIELD,
    interval: TimeSpan::from_millis(1500),
    effect: 5040,
    has_damage: false,
    on_tick,
};

/// `Server.Spells.Fifth.PoisonField`: `[DispellableField]`.
pub static POISON_FIELD: ItemType = ItemType {
    full_name: "Server.Spells.Fifth.PoisonField",
    parent: Some(&ITEM_BASE),
    construct: None,
    blocks_fit: |_, _| true,
    on_move_over,
    on_after_delete: fields::on_after_delete,
    serialize: |w, id, out| fields::serialize(w, id, out, false),
    deserialize: |w, id, r| fields::deserialize(w, id, r, &POISON_FIELD_CLASS),
    ..ITEM_BASE
};
