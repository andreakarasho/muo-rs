//! The Fire Field spell and the `FireFieldItem` item.

use muo_content::core;
use muo_content::misc::aos_helpers::{self, DamageOptions, DamageSplit};
use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::magery::{FOURTH_CIRCLE, MAGERY_SPELL};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_data::expansion::Expansion;
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm, stats};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};

use crate::common::{creature_on_harmful_spell, reveal_caster_if, surface_top};
use crate::fields::{self, FieldClass, data};

/// `FireFieldSpell`: In Flam Grav.
pub static FIRE_FIELD_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fourth.FireFieldSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::field_range),
    info: SpellInfo::new(
        "Fire Field",
        "In Flam Grav",
        &[
            reagent::BLACK_PEARL,
            reagent::SPIDERS_SILK,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(215)
    .hand_effect(9041)
    .allow_town(false),
    on_cast,
    family: Some(&FOURTH_CIRCLE),
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

        muo_server::effects::play_sound(w, loc, map, 0x20C);

        let item_id = if east_to_west { 0x398C } else { 0x3996 };

        let exp = core::expansion(w);
        let magery = skills::value(w, caster, SkillName::Magery);
        let duration = if exp == Expansion::None {
            TimeSpan::from_secs(20)
        } else if exp < Expansion::LBR {
            TimeSpan::from_secs_f64(magery)
        } else {
            TimeSpan::from_secs_f64(4.0 + magery * 0.5)
        };

        for i in -2..=2 {
            let target_loc = Point3D::new(
                if east_to_west { loc.x + i } else { loc.x },
                if east_to_west { loc.y } else { loc.y + i },
                loc.z,
            );
            new_fire_field_item(w, item_id, target_loc, caster, map, duration, i, 2);
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

/// `new FireFieldItem(itemID, loc, caster, map, duration, val, damage)`.
#[allow(clippy::too_many_arguments)]
pub fn new_fire_field_item(
    w: &mut World,
    item_id: i32,
    loc: Point3D,
    caster: MobileId,
    map: Option<muo_server::map::MapId>,
    duration: TimeSpan,
    val: i32,
    damage: i32,
) -> ItemId {
    fields::new_field(
        w,
        &FIRE_FIELD_CLASS,
        item_id,
        loc,
        caster,
        map,
        duration,
        val,
        damage,
    )
}

/// Burns `m` (the body of `OnMoveOver` and of the timer tick).
fn burn(w: &mut World, item: ItemId, caster: MobileId, m: MobileId) {
    reveal_caster_if(w, m, caster);
    mb::do_harmful(w, caster, m, false);

    let mut damage = data(w, item).map_or(0, |d| d.damage);
    if !core::aos(w) && stats::check_skill(w, m, SkillName::MagicResist, 0.0, 30.0) {
        damage = 1;
        mm::send_localized_message(w, m, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
    }

    aos_helpers::damage(
        w,
        m,
        Some(caster),
        damage,
        DamageSplit {
            fire: 100,
            ..Default::default()
        },
        DamageOptions::default(),
    );
    effects::play_sound(w, m, 0x208);
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
        burn(w, id, caster, m);
    }
    true
}

fn on_tick(w: &mut World, id: ItemId, caster: MobileId) {
    let z = w.item(id).map_or(0, |i| i.location.z);
    let queue: Vec<MobileId> = {
        let (map, loc) = match w.item(id) {
            Some(i) => (i.map, i.location),
            None => return,
        };
        muo_server::map::enumerate::mobiles_at(w, map, loc.x, loc.y)
            .filter(|&m| {
                w.mobile(m)
                    .is_some_and(|x| x.location.z + 16 > z && z + 12 > x.location.z)
            })
            .collect()
    };
    for m in queue {
        if (!core::aos(w) || m != caster)
            && helper::valid_indirect_target(w, caster, m)
            && mb::can_be_harmful_msg(w, caster, m, false)
        {
            burn(w, id, caster, m);
        }
    }
}

static FIRE_FIELD_CLASS: FieldClass = FieldClass {
    ty: &FIRE_FIELD_ITEM,
    interval: TimeSpan::from_secs(1),
    effect: 5029,
    has_damage: true,
    on_tick,
};

/// `Server.Spells.Fourth.FireFieldItem`: `[DispellableField]`.
pub static FIRE_FIELD_ITEM: ItemType = ItemType {
    full_name: "Server.Spells.Fourth.FireFieldItem",
    parent: Some(&ITEM_BASE),
    construct: None,
    blocks_fit: |_, _| true,
    on_move_over,
    on_after_delete: fields::on_after_delete,
    serialize: |w, id, out| fields::serialize(w, id, out, true),
    deserialize: |w, id, r| fields::deserialize(w, id, r, &FIRE_FIELD_CLASS),
    ..ITEM_BASE
};
