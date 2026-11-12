//! `HarrowerGate` and `StarRoomGate` (both `Moongate`).

use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::PropType;
use muo_items::misc::moongates::{MOONGATE, moongate_new, set_dispellable};
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ItemType, ops};
use muo_server::light_type::LightType;
use muo_server::map::MapId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

// ---- HarrowerGate ----

/// `HarrowerGate._harrower`.
#[derive(Clone, Copy, Debug, Default)]
pub struct HarrowerGateData(pub Option<MobileId>);

fn harrower_of(w: &World, id: ItemId) -> Option<MobileId> {
    w.item(id)?.ext.get::<HarrowerGateData>()?.0
}

/// `HarrowerGate(harrower, loc, map, targLoc, targMap)`.
pub fn harrower_gate_new(
    w: &mut World,
    harrower: MobileId,
    loc: Point3D,
    map: Option<MapId>,
    targ_loc: Point3D,
    targ_map: MapId,
) -> ItemId {
    let id = moongate_new(w, &HARROWER_GATE, targ_loc, Some(targ_map), true);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(HarrowerGateData(Some(harrower)));
    }
    set_dispellable(w, id, false);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, 0x1FD4);
    }
    ops::set_light(w, id, LightType::Circle300);
    ops::move_to_world_map(w, id, loc, map);
    id
}

/// `[SerializationGenerator(0, false)] HarrowerGate`.
pub static HARROWER_GATE: ItemType = ItemType {
    full_name: "Server.Items.HarrowerGate",
    parent: Some(&MOONGATE),
    construct: None,
    label_number: |_, _| 1049498, // dark moongate
    serialize: |w, id, out| {
        (MOONGATE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_mobile(w, harrower_of(w, id));
    },
    deserialize: |w, id, r| {
        (MOONGATE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let harrower = r.read_mobile(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(HarrowerGateData(harrower));
        }
        // `Timer.DelayCall(AfterDeserialization)`: a gate without its harrower goes.
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            if harrower_of(w, id).is_none() {
                delete(w, id);
            }
        });
        Ok(())
    },
    ..MOONGATE
};

// ---- StarRoomGate ----

const GATE_DURATION: TimeSpan = TimeSpan::from_mins(2);

/// `StarRoomGate._decays`, `_decayTime` and the timer.
#[derive(Clone, Copy, Debug, Default)]
pub struct StarRoomGateData {
    pub decays: bool,
    pub decay_time: DateTime,
    timer: timer::TimerHandle,
}

fn star(w: &World, id: ItemId) -> StarRoomGateData {
    w.item(id)
        .and_then(|i| i.ext.get::<StarRoomGateData>())
        .copied()
        .unwrap_or_default()
}

fn star_edit(w: &mut World, id: ItemId, f: impl FnOnce(&mut StarRoomGateData)) {
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<StarRoomGateData>())
    {
        f(d);
    }
}

/// `StarRoomGate(decays)`: a gate to the star room (5143, 1774, 0) in Felucca.
pub fn star_room_gate_new(w: &mut World, decays: bool) -> ItemId {
    let id = moongate_new(
        w,
        &STAR_ROOM_GATE,
        Point3D::new(5143, 1774, 0),
        Some(MapId::FELUCCA),
        true,
    );
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(StarRoomGateData::default());
    }
    set_dispellable(w, id, false);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.set_item_id)(w, id, 0x1FD4);
    }
    if decays {
        let decay_time = w.now + GATE_DURATION;
        let h = timer::start(w, GATE_DURATION, TimeSpan::ZERO, 1, move |w| delete(w, id));
        star_edit(w, id, |d| {
            d.decays = true;
            d.decay_time = decay_time;
            d.timer = h;
        });
    }
    id
}

/// `StarRoomGate(loc, map, decays)`.
pub fn star_room_gate_at(w: &mut World, loc: Point3D, map: Option<MapId>, decays: bool) -> ItemId {
    let id = star_room_gate_new(w, decays);
    ops::move_to_world_map(w, id, loc, map);
    muo_server::effects::play_sound(w, loc, map, 0x20E);
    id
}

/// `[SerializationGenerator(2, false)] StarRoomGate`.
pub static STAR_ROOM_GATE: ItemType = ItemType {
    full_name: "Server.Items.StarRoomGate",
    parent: Some(&MOONGATE),
    construct: Some(|w| star_room_gate_new(w, false)),
    label_number: |_, _| 1049498, // dark moongate
    on_after_delete: |w, id| {
        let mut h = star(w, id).timer;
        h.cancel(w);
        (MOONGATE.on_after_delete)(w, id);
    },
    serialize: |w, id, out| {
        (MOONGATE.serialize)(w, id, out);
        let d = star(w, id);
        out.write_i32(2);
        out.write_bool(d.decays);
        out.write_anchored_time_ticks(d.decay_time.ticks());
    },
    deserialize: |w, id, r| {
        (MOONGATE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut d = StarRoomGateData::default();
        if version == 1 {
            // V1Content: both fields, the time a delta
            d.decays = r.read_bool()?;
            d.decay_time =
                DateTime::from_ticks(r.read_delta_time_ticks(DateTime::utc_now().ticks())?);
        } else if version < 2 {
            d.decays = r.read_bool()?;
            if d.decays {
                d.decay_time =
                    DateTime::from_ticks(r.read_delta_time_ticks(DateTime::utc_now().ticks())?);
            }
        } else {
            d.decays = r.read_bool()?;
            d.decay_time = DateTime::from_ticks(r.read_anchored_time_ticks()?);
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(d);
        }
        // [AfterDeserialization]
        if d.decays {
            let h = timer::start(w, d.decay_time - w.now, TimeSpan::ZERO, 1, move |w| {
                delete(w, id)
            });
            star_edit(w, id, |d| d.timer = h);
        }
        Ok(())
    },
    ..MOONGATE
};

static BOOL_PARAMS: [Param; 1] = [Param {
    name: "decays",
    kind: Kind::Plain(PropType::Bool),
    default: Some(Value::Bool(false)),
    ty: "",
}];

static LOC_MAP_PARAMS: [Param; 3] = [
    Param {
        name: "loc",
        kind: Kind::Plain(PropType::IPoint3D),
        default: None,
        ty: "",
    },
    Param {
        name: "map",
        kind: Kind::Map,
        default: None,
        ty: "",
    },
    Param {
        name: "decays",
        kind: Kind::Plain(PropType::Bool),
        default: None,
        ty: "",
    },
];

/// `StarRoomGate(Point3D loc, Map map, bool decays)` and `StarRoomGate(bool decays = false)`.
static STAR_ROOM_GATE_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &LOC_MAP_PARAMS,
        make: |w, v| {
            let loc = match &v[0] {
                Value::Point3D(p) => *p,
                _ => Point3D::default(),
            };
            EntityId::Item(star_room_gate_at(w, loc, v[1].map(), v[2].i64() != 0))
        },
    },
    Ctor {
        access: None,
        params: &BOOL_PARAMS,
        make: |w, v| EntityId::Item(star_room_gate_new(w, v[0].i64() != 0)),
    },
];

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&HARROWER_GATE, &[]);
    types.register_item(&STAR_ROOM_GATE, &[]);
    types.register_phase(
        Phase::Configure,
        "CannedEvil.Gates",
        DEFAULT_CALL_PRIORITY,
        |w| {
            add_ctors(w, "Server.Items.StarRoomGate", &STAR_ROOM_GATE_CTORS);
            // `new StarRoomGate(loc, map, true)` of the champion spawn, `new HarrowerGate(..)` of the Harrower.
            w.resources
                .get_mut::<super::CannedEvilHooks>()
                .star_room_gate = Some(|w, loc, map| {
                star_room_gate_at(w, loc, map, true);
            });
            w.resources
                .get_mut::<muo_mobiles::npcs::special::hooks::SpecialHooks>()
                .harrower_gate = Some(|w, harrower, loc, map, targ_loc, targ_map| {
                Some(harrower_gate_new(w, harrower, loc, map, targ_loc, targ_map))
            });
            w.resources
                .get_mut::<super::CannedEvilHooks>()
                .create_random_sot = Some(|w, min, max| {
                Some(muo_items::special::scrolls::transcendence_create_random(
                    w, min, max,
                ))
            });
        },
    );
}
