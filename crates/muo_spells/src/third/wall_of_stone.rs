//! The Wall Of Stone spell and the `WallOfStone` item.

use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::magery::{MAGERY_SPELL, THIRD_CIRCLE};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::notoriety;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use crate::common::surface_top;

/// `WallOfStoneSpell`: In Sanct Ylem.
pub static WALL_OF_STONE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Third.WallOfStoneSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::field_range),
    info: SpellInfo::new(
        "Wall of Stone",
        "In Sanct Ylem",
        &[reagent::BLOODMOSS, reagent::GARLIC],
    )
    .action(227)
    .hand_effect(9011)
    .allow_town(false),
    on_cast,
    family: Some(&THIRD_CIRCLE),
    ..MAGERY_SPELL
};

/// `TargetRange => Core.T2A ? 15 : 18`.
fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if helper::check_town(w, Point3dRef::from(targeted), caster) && spell::check_sequence(w, sp) {
        helper::turn(w, caster, Some(Point3dRef::from(targeted)));
        let p = surface_top(w, targeted);
        let from = w.mobile(caster).map(|c| c.location).unwrap_or_default();
        let map = w.mobile(caster).and_then(|c| c.map);

        let east_to_west = helper::get_east_to_west(from, p);

        muo_server::effects::play_sound(w, p, map, 0x1F6);

        for i in -1..=1 {
            let mut target_loc = Point3D::new(
                if east_to_west { p.x + i } else { p.x },
                if east_to_west { p.y } else { p.y + i },
                p.z,
            );
            let can_fit = helper::adjust_field(w, &mut target_loc, map, 22, true);
            if !can_fit {
                continue;
            }
            let item = new_wall_of_stone(w, target_loc, map, caster);
            if !ops::is_deleted(w, item) {
                muo_server::effects::send_location_particles(w, item, 0x376A, 9, 10, 5025);
            }
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

/// What `WallOfStone` adds to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct WallData {
    pub caster: Option<MobileId>,
    pub end: DateTime,
    pub timer: TimerHandle,
}

fn data(w: &World, id: ItemId) -> Option<&WallData> {
    w.item(id)?.ext.get::<WallData>()
}

/// `new WallOfStone(loc, map, caster)`.
pub fn new_wall_of_stone(
    w: &mut World,
    loc: Point3D,
    map: Option<muo_server::map::MapId>,
    caster: MobileId,
) -> ItemId {
    let id = ops::new_item_with_id(w, &WALL_OF_STONE, 0x82);
    ops::set_visible(w, id, false);
    ops::set_movable(w, id, false);
    ops::move_to_world_map(w, id, loc, map);

    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(WallData {
            caster: Some(caster),
            ..Default::default()
        });
    }

    if deps::in_los_item(w, caster, id) {
        ops::set_visible(w, id, true);
    } else {
        base::delete(w, id);
    }

    if ops::is_deleted(w, id) {
        return id;
    }

    let duration = TimeSpan::from_secs(10);
    start_timer(w, id, duration);
    let end = w.now + duration;
    if let Some(Some(d)) = w.item_mut(id).map(|i| i.ext.get_mut::<WallData>()) {
        d.end = end;
    }
    id
}

fn start_timer(w: &mut World, id: ItemId, delay: TimeSpan) {
    let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| base::delete(w, id));
    if let Some(Some(d)) = w.item_mut(id).map(|i| i.ext.get_mut::<WallData>()) {
        d.timer = h;
    }
}

fn on_move_over(w: &mut World, id: ItemId, m: MobileId) -> bool {
    if w.mobile(m)
        .is_some_and(|x| x.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
        && let Some(caster) = data(w, id).and_then(|d| d.caster)
    {
        let noto = notoriety::compute(w, caster, m);
        if noto == notoriety::ENEMY || noto == notoriety::ALLY {
            return false;
        }
    }
    base::on_move_over_base(w, id, m)
}

fn on_after_delete(w: &mut World, id: ItemId) {
    base::on_after_delete_base(w, id);
    if let Some(Some(d)) = w.item_mut(id).map(|i| i.ext.get_mut::<WallData>()) {
        let mut t = d.timer;
        t.cancel(w);
    }
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
        // V0Content: ReadDeltaTime
        r.read_delta_time_ticks(now.ticks())?
    } else {
        r.read_anchored_time_ticks()?
    };
    let end = DateTime::try_from_ticks(end).unwrap_or(DateTime::MIN);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(WallData {
            caster,
            end,
            timer: TimerHandle::default(),
        });
    }
    // AfterDeserialization: Timer.StartTimer(_end - Core.Now, Delete)
    start_timer(w, id, end - now);
    Ok(())
}

/// `Server.Spells.Third.WallOfStone`: `[DispellableField]`.
pub static WALL_OF_STONE: ItemType = ItemType {
    full_name: "Server.Spells.Third.WallOfStone",
    parent: Some(&ITEM_BASE),
    construct: None,
    blocks_fit: |_, _| true,
    on_move_over,
    on_after_delete,
    serialize,
    deserialize,
    ..ITEM_BASE
};
