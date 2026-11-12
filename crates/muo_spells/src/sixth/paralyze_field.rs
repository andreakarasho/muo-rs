//! The Paralyze Field spell and the `ParalyzeField` item.

use muo_content::core;
use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::magery::{MAGERY_SPELL, SIXTH_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_data::expansion::Expansion;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, effect_item, ops};
use muo_server::light_type::LightType;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::effects;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use crate::common::{creature_on_harmful_spell, reveal_caster_if, surface_top};

/// `ParalyzeFieldSpell`: In Ex Grav.
pub static PARALYZE_FIELD_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Sixth.ParalyzeFieldSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::field_range),
    info: SpellInfo::new(
        "Paralyze Field",
        "In Ex Grav",
        &[
            reagent::BLACK_PEARL,
            reagent::GINSENG,
            reagent::SPIDERS_SILK,
        ],
    )
    .action(230)
    .hand_effect(9012)
    .allow_town(false),
    on_cast,
    family: Some(&SIXTH_CIRCLE),
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

        let item_id = if east_to_west { 0x3967 } else { 0x3979 };

        let exp = core::expansion(w);
        let magery = skills::value(w, caster, SkillName::Magery);
        let duration = if exp == Expansion::None {
            TimeSpan::from_secs(20)
        } else if exp < Expansion::LBR {
            TimeSpan::from_secs_f64(15.0 + magery / 3.0)
        } else {
            TimeSpan::from_secs_f64(3.0 + magery / 3.0)
        };

        for i in -2..=2 {
            let mut target_loc = Point3D::new(
                if east_to_west { loc.x + i } else { loc.x },
                if east_to_west { loc.y } else { loc.y + i },
                loc.z,
            );
            if !helper::adjust_field(w, &mut target_loc, map, 12, false) {
                continue;
            }

            let item = new_paralyze_field(w, caster, item_id, target_loc, map, duration);
            if !ops::is_deleted(w, item) {
                base::process_delta(w, item);
            }

            let e = effect_item::create_targeted(w, target_loc, map, effect_item::DEFAULT_DURATION);
            muo_server::effects::send_location_particles(w, e, 0x376A, 9, 10, 5048);
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

/// What `ParalyzeField` adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct ParalyzeData {
    pub caster: Option<MobileId>,
    pub end: DateTime,
    pub timer: TimerHandle,
}

fn data(w: &World, id: ItemId) -> Option<&ParalyzeData> {
    w.item(id)?.ext.get::<ParalyzeData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut ParalyzeData> {
    w.item_mut(id)?.ext.get_mut::<ParalyzeData>()
}

/// `new ParalyzeField(caster, itemID, loc, map, duration)`.
pub fn new_paralyze_field(
    w: &mut World,
    caster: MobileId,
    item_id: i32,
    loc: Point3D,
    map: Option<muo_server::map::MapId>,
    duration: TimeSpan,
) -> ItemId {
    let id = ops::new_item_with_id(w, &PARALYZE_FIELD, item_id);
    ops::set_visible(w, id, false);
    ops::set_movable(w, id, false);
    ops::set_light(w, id, LightType::Circle300);
    ops::move_to_world_map(w, id, loc, map);

    if deps::in_los_item(w, caster, id) {
        ops::set_visible(w, id, true);
    } else {
        base::delete(w, id);
    }

    if ops::is_deleted(w, id) {
        return id;
    }

    let end = w.now + duration;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ParalyzeData {
            caster: Some(caster),
            end,
            timer: TimerHandle::default(),
        });
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

fn on_after_delete(w: &mut World, id: ItemId) {
    base::on_after_delete_base(w, id);
    if let Some(d) = data_mut(w, id) {
        let mut t = d.timer;
        t.cancel(w);
    }
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
        reveal_caster_if(w, m, caster);

        mb::do_harmful(w, caster, m, false);

        let mut duration: f64;
        if core::aos(w) {
            let eval = (skills::value(w, caster, SkillName::EvalInt) / 10.0) as i32;
            let resist = (skills::value(w, m, SkillName::MagicResist) / 10.0) as i32;
            duration = (2.0 + f64::from(eval - resist)).max(0.0);

            if !w.mobile(m).is_some_and(|x| x.player) {
                duration *= 3.0;
            }
        } else {
            duration = 7.0 + skills::value(w, caster, SkillName::Magery) / 5.0;
        }

        mb::paralyze(w, m, TimeSpan::from_secs_f64(duration));

        effects::play_sound(w, m, 0x204);
        effects::fixed_effect(w, m, 0x376A, 10, 16);

        creature_on_harmful_spell(w, m, caster);
    }
    true
}

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(1); // SerializationVersion
    let d = data(w, id).cloned().unwrap_or_default();
    out.write_mobile(w, d.caster);
    out.write_anchored_time_ticks(d.end.ticks());
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let now = w.now;
    let caster = r.read_mobile(w)?;
    let end = if version == 0 {
        r.read_delta_time_ticks(now.ticks())?
    } else {
        r.read_anchored_time_ticks()?
    };
    let end = DateTime::try_from_ticks(end).unwrap_or(DateTime::MIN);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ParalyzeData {
            caster,
            end,
            timer: TimerHandle::default(),
        });
    }
    // AfterDeserialization: Timer.StartTimer(_end - Core.Now, Delete)
    start_timer(w, id, end - now);
    Ok(())
}

/// `Server.Spells.Sixth.ParalyzeField`: `[DispellableField]`.
pub static PARALYZE_FIELD: ItemType = ItemType {
    full_name: "Server.Spells.Sixth.ParalyzeField",
    parent: Some(&ITEM_BASE),
    construct: None,
    blocks_fit: |_, _| true,
    on_move_over,
    on_after_delete,
    serialize,
    deserialize,
    ..ITEM_BASE
};
