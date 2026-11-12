//! The Energy Field spell and the `EnergyField` item.

use muo_content::core;
use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::magery::{MAGERY_SPELL, SEVENTH_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, effect_item, ops};
use muo_server::light_type::LightType;
use muo_server::notoriety;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use crate::common::{is_player_mobile, surface_top};

/// `EnergyFieldSpell`: In Sanct Grav.
pub static ENERGY_FIELD_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Seventh.EnergyFieldSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::field_range),
    info: SpellInfo::new(
        "Energy Field",
        "In Sanct Grav",
        &[
            reagent::BLACK_PEARL,
            reagent::MANDRAKE_ROOT,
            reagent::SPIDERS_SILK,
            reagent::SULFUROUS_ASH,
        ],
    )
    .action(221)
    .hand_effect(9022)
    .allow_town(false),
    on_cast,
    family: Some(&SEVENTH_CIRCLE),
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

        let magery = skills::value(w, caster, SkillName::Magery);
        let duration = if core::aos(w) {
            TimeSpan::from_secs_f64((15.0 + magery * 2.0) / 7.0)
        } else {
            TimeSpan::from_secs_f64(magery * 0.28 + 2.0)
        };

        let item_id = if east_to_west { 0x3946 } else { 0x3956 };

        for i in -2..=2 {
            let mut target_loc = Point3D::new(
                if east_to_west { loc.x + i } else { loc.x },
                if east_to_west { loc.y } else { loc.y + i },
                loc.z,
            );
            let can_fit = helper::adjust_field(w, &mut target_loc, map, 12, false);
            if !can_fit {
                continue;
            }

            let item = new_energy_field(w, target_loc, map, duration, item_id, caster);
            if !ops::is_deleted(w, item) {
                base::process_delta(w, item);
            }

            let e = effect_item::create_targeted(w, target_loc, map, effect_item::DEFAULT_DURATION);
            muo_server::effects::send_location_particles(w, e, 0x376A, 9, 10, 5051);
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

/// What `EnergyField` adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct EnergyData {
    pub caster: Option<MobileId>,
    pub end: DateTime,
    pub timer: TimerHandle,
}

fn data(w: &World, id: ItemId) -> Option<&EnergyData> {
    w.item(id)?.ext.get::<EnergyData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut EnergyData> {
    w.item_mut(id)?.ext.get_mut::<EnergyData>()
}

/// `new EnergyField(loc, map, duration, itemID, caster)`.
pub fn new_energy_field(
    w: &mut World,
    loc: Point3D,
    map: Option<muo_server::map::MapId>,
    duration: TimeSpan,
    item_id: i32,
    caster: MobileId,
) -> ItemId {
    let id = ops::new_item_with_id(w, &ENERGY_FIELD, item_id);
    ops::set_visible(w, id, false);
    ops::set_movable(w, id, false);
    ops::set_light(w, id, LightType::Circle300);
    ops::move_to_world_map(w, id, loc, map);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(EnergyData {
            caster: Some(caster),
            ..Default::default()
        });
    }

    if deps::in_los_item(w, caster, id) {
        ops::set_visible(w, id, true);
    } else {
        base::delete(w, id);
        return id;
    }

    start_timer(w, id, duration);
    id
}

fn start_timer(w: &mut World, id: ItemId, delay: TimeSpan) {
    let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| base::delete(w, id));
    if let Some(d) = data_mut(w, id) {
        d.timer = h;
    }
}

fn on_move_over(w: &mut World, id: ItemId, m: MobileId) -> bool {
    if !is_player_mobile(w, m) {
        return base::on_move_over_base(w, id, m);
    }

    let Some(caster) = data(w, id).and_then(|d| d.caster) else {
        return base::on_move_over_base(w, id, m);
    };
    let noto = notoriety::compute(w, caster, m);
    noto != notoriety::ENEMY && noto != notoriety::ALLY && base::on_move_over_base(w, id, m)
}

fn on_after_delete(w: &mut World, id: ItemId) {
    base::on_after_delete_base(w, id);
    if let Some(d) = data_mut(w, id) {
        let mut t = d.timer;
        t.cancel(w);
    }
}

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(2); // SerializationVersion
    let d = data(w, id).cloned().unwrap_or_default();
    out.write_mobile(w, d.caster);
    out.write_anchored_time_ticks(d.end.ticks());
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let now = w.now;
    let mut caster = None;
    let mut end = DateTime::MIN;
    if version == 1 {
        // V1Content: caster, delta time
        caster = r.read_mobile(w)?;
        end = DateTime::try_from_ticks(r.read_delta_time_ticks(now.ticks())?)
            .unwrap_or(DateTime::MIN);
    } else if version >= 2 {
        caster = r.read_mobile(w)?;
        end = DateTime::try_from_ticks(r.read_anchored_time_ticks()?).unwrap_or(DateTime::MIN);
    }
    // version 0: "Do nothing, no caster"
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(EnergyData {
            caster,
            end,
            timer: TimerHandle::default(),
        });
    }
    // AfterDeserialization
    let duration = if end > DateTime::MIN {
        end - now
    } else {
        TimeSpan::from_secs(5)
    };
    start_timer(w, id, duration);
    Ok(())
}

/// `Server.Spells.Seventh.EnergyField`: `[DispellableField]`.
pub static ENERGY_FIELD: ItemType = ItemType {
    full_name: "Server.Spells.Seventh.EnergyField",
    parent: Some(&ITEM_BASE),
    construct: None,
    blocks_fit: |_, _| true,
    on_move_over,
    on_after_delete,
    serialize,
    deserialize,
    ..ITEM_BASE
};
