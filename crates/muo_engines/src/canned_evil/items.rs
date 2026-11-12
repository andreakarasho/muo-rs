//! `ChampionPlatform`, `ChampionAltar`, `ChampionSkull`, `ChampionSkullBrazier` and
//! `ChampionSkullPlatform`. `HarrowerGate` and `StarRoomGate` are in `gates`; the property tables in `engines_b_props`.

use muo_content::items::addon::{self, AddonClass, Comp};
use muo_content::props::{Ctor, Kind, Param, Value};
use muo_content::utilities::types::PropType;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_items::addons::leaves::PENTAGRAM_ADDON;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::light_type::LightType;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

use super::info::ChampionSkullType;
use crate::util::hue;

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.delete)(w, id);
    }
}

fn is_deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

/// The `_spawn` of the platform and the altar (and `HarrowerGate._harrower`).
#[derive(Clone, Copy, Default)]
pub struct SpawnRef {
    pub spawn: Option<ItemId>,
}

pub fn spawn_ref(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.ext.get::<SpawnRef>()?.spawn
}

fn write_spawn_ref(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
    out.write_item(w, spawn_ref(w, id));
}

fn read_spawn_ref(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    let _version = r.read_i32()?;
    let spawn = r.read_item(w)?;
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(SpawnRef { spawn });
    }
    // `[AfterDeserialization(false)]`: no spawn, no platform/altar.
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        if spawn_ref(w, id).is_none() {
            delete_item(w, id);
        }
    });
    Ok(())
}

// ---- ChampionPlatform ----

/// `AddComponent(int id, x, y, z)`: an `AddonComponent` with `Hue = 0x497`.
fn platform_component(w: &mut World, platform: ItemId, id: i32, x: i32, y: i32, z: i32) {
    let c = addon::new_component(w, &addon::ADDON_COMPONENT, id);
    hue(w, c, 0x497);
    addon::add_component(w, platform, c, x, y, z);
}

/// `ChampionPlatform(spawn)`.
pub fn new_platform(w: &mut World, spawn: ItemId) -> ItemId {
    let id = addon::new_addon(w, &CHAMPION_PLATFORM);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(SpawnRef { spawn: Some(spawn) });
    }
    for x in -2..=2 {
        for y in -2..=2 {
            platform_component(w, id, 0x750, x, y, -5);
        }
    }
    for x in -1..=1 {
        for y in -1..=1 {
            platform_component(w, id, 0x750, x, y, 0);
        }
    }
    for i in -1..=1 {
        platform_component(w, id, 0x751, i, 2, 0);
        platform_component(w, id, 0x752, 2, i, 0);
        platform_component(w, id, 0x753, i, -2, 0);
        platform_component(w, id, 0x754, -2, i, 0);
    }
    platform_component(w, id, 0x759, -2, -2, 0);
    platform_component(w, id, 0x75A, 2, 2, 0);
    platform_component(w, id, 0x75B, -2, 2, 0);
    platform_component(w, id, 0x75C, 2, -2, 0);
    id
}

static CHAMPION_PLATFORM_CLASS: AddonClass = AddonClass {
    ..addon::BASE_ADDON_CLASS
};

/// `[SerializationGenerator(0, false)] ChampionPlatform`.
pub static CHAMPION_PLATFORM: ItemType = ItemType {
    full_name: "Server.Engines.CannedEvil.ChampionPlatform",
    parent: Some(&addon::BASE_ADDON),
    // `ChampionPlatform(ChampionSpawn)`: no parameterless constructor.
    construct: None,
    family: Some(&CHAMPION_PLATFORM_CLASS),
    on_after_delete: |w, id| {
        (addon::BASE_ADDON.on_after_delete)(w, id);
        if let Some(s) = spawn_ref(w, id) {
            delete_item(w, s);
        }
    },
    serialize: |w, id, out| {
        (addon::BASE_ADDON.serialize)(w, id, out);
        write_spawn_ref(w, id, out);
    },
    deserialize: |w, id, r| {
        (addon::BASE_ADDON.deserialize)(w, id, r)?;
        read_spawn_ref(w, id, r)
    },
    ..addon::BASE_ADDON
};

// ---- ChampionAltar ----

/// `ChampionAltar(spawn)`: `PentagramAddon()`'s nine components.
pub fn new_altar(w: &mut World, spawn: ItemId) -> ItemId {
    let id = addon::new_addon(w, &CHAMPION_ALTAR);
    addon::add_components(
        w,
        id,
        &[
            (Comp::Plain(4071), -1, -1, 0),
            (Comp::Plain(4072), 0, -1, 0),
            (Comp::Plain(4075), 1, -1, 0),
            (Comp::Plain(4070), -1, 0, 0),
            (Comp::Plain(4074), 0, 0, 0),
            (Comp::Plain(4078), 1, 0, 0),
            (Comp::Plain(4073), -1, 1, 0),
            (Comp::Plain(4076), 0, 1, 0),
            (Comp::Plain(4077), 1, 1, 0),
        ],
    );
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(SpawnRef { spawn: Some(spawn) });
    }
    id
}

/// `[SerializationGenerator(0, false)] ChampionAltar : PentagramAddon`.
pub static CHAMPION_ALTAR: ItemType = ItemType {
    full_name: "Server.Engines.CannedEvil.ChampionAltar",
    parent: Some(&PENTAGRAM_ADDON),
    construct: None,
    on_after_delete: |w, id| {
        (PENTAGRAM_ADDON.on_after_delete)(w, id);
        if let Some(s) = spawn_ref(w, id) {
            delete_item(w, s);
        }
    },
    serialize: |w, id, out| {
        (PENTAGRAM_ADDON.serialize)(w, id, out);
        write_spawn_ref(w, id, out);
    },
    deserialize: |w, id, r| {
        (PENTAGRAM_ADDON.deserialize)(w, id, r)?;
        read_spawn_ref(w, id, r)
    },
    ..PENTAGRAM_ADDON
};

// ---- ChampionSkull ----

/// `ChampionSkull._type`.
#[derive(Clone, Copy, Default)]
pub struct SkullData {
    pub ty: ChampionSkullType,
}

pub fn skull_type(w: &World, id: ItemId) -> ChampionSkullType {
    w.item(id)
        .and_then(|i| i.ext.get::<SkullData>())
        .map_or(ChampionSkullType::None, |d| d.ty)
}

/// `ChampionSkull(type)`: `base(0x1AE1)`, `LootType = Cursed`, and the hue by type.
pub fn skull_new(w: &mut World, ty: ChampionSkullType) -> ItemId {
    let id = ops::new_item_with_id(w, &CHAMPION_SKULL, 0x1AE1);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(SkullData { ty });
    }
    ops::set_loot_type(w, id, LootType::Cursed);
    // TODO: all hue values
    let h = match ty {
        ChampionSkullType::Power => Some(0x159),
        ChampionSkullType::Venom => Some(0x172),
        ChampionSkullType::Greed => Some(0x1EE),
        ChampionSkullType::Death => Some(0x025),
        ChampionSkullType::Pain => Some(0x035),
        _ => None,
    };
    if let Some(h) = h {
        hue(w, id, h);
    }
    id
}

/// `[SerializationGenerator(2, false)] ChampionSkull`.
pub static CHAMPION_SKULL: ItemType = ItemType {
    full_name: "Server.Items.ChampionSkull",
    parent: Some(&ITEM_BASE),
    // `ChampionSkull(ChampionSkullType type)`: see the constructor table.
    construct: None,
    label_number: |w, id| 1049479 + skull_type(w, id) as i32,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(2);
        out.write_enum(4, skull_type(w, id) as i64);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let ty = if version < 2 {
            r.read_i32()?
        } else {
            r.read_enum(4)? as i32
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(SkullData {
                ty: ChampionSkullType::from_i32(ty),
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

const SKULL_NAMES: &[(&str, i64)] = &[
    ("Power", 0),
    ("Enlightenment", 1),
    ("Venom", 2),
    ("Pain", 3),
    ("Greed", 4),
    ("Death", 5),
    ("None", 6),
];
static SKULL_PARAMS: [Param; 1] = [Param {
    name: "type",
    kind: Kind::Plain(PropType::Enum(SKULL_NAMES)),
    default: None,
    ty: "SkullType",
}];
pub static SKULL_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &SKULL_PARAMS,
    make: |w, v: &[Value]| EntityId::Item(skull_new(w, ChampionSkullType::from_i32(v[0].i32()))),
}];

// ---- ChampionSkullBrazier ----

/// `ChampionSkullBrazier`'s fields.
#[derive(Clone, Copy)]
pub struct BrazierData {
    pub ty: ChampionSkullType,
    pub platform: Option<ItemId>,
    pub skull: Option<ItemId>,
}

fn brazier(w: &World, id: ItemId) -> Option<&BrazierData> {
    w.item(id)?.ext.get::<BrazierData>()
}

/// `Skull = value` (`OnSkullChanged`: `_platform?.Validate()`).
pub fn set_skull(w: &mut World, id: ItemId, value: Option<ItemId>) {
    let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<BrazierData>()) else {
        return;
    };
    if d.skull == value {
        return;
    }
    d.skull = value;
    let platform = d.platform;
    if let Some(p) = platform {
        validate(w, p);
    }
}

/// `ChampionSkullBrazier(platform, type)`: `base(0x19BB)`, `Hue = 0x455`, `Light = Circle300`.
pub fn brazier_new(w: &mut World, platform: ItemId, ty: ChampionSkullType) -> ItemId {
    let id = addon::new_component(w, &CHAMPION_SKULL_BRAZIER, 0x19BB);
    hue(w, id, 0x455);
    ops::set_light(w, id, LightType::Circle300);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(BrazierData {
            ty,
            platform: Some(platform),
            skull: None,
        });
    }
    id
}

fn world_location(w: &World, id: ItemId) -> muo_primitives::Point3D {
    ops::get_world_location(w, id)
}

/// `BeginSacrifice(from)`.
fn begin_sacrifice(w: &mut World, id: ItemId, from: MobileId) {
    if is_deleted(w, id) {
        return;
    }
    if brazier(w, id)
        .and_then(|d| d.skull)
        .is_some_and(|s| is_deleted(w, s))
    {
        set_skull(w, id, None);
    }
    let loc = world_location(w, id);
    let same_map = w.mobile(from).map(|m| m.map) == w.item(id).map(|i| i.map);
    if !same_map || !muo_content::mx::in_range_point(w, from, loc, 3) {
        muo_content::mx::msg(w, from, 500446); // That is too far away.
    } else if !harrower_can_spawn(w) {
        muo_content::mx::send_message(w, from, "The harrower has already been spawned.");
    } else if brazier(w, id).and_then(|d| d.skull).is_none() {
        muo_content::mx::msg(w, from, 1049485); // What would you like to sacrifice?
        muo_server::target::begin_target(
            w,
            from,
            12,
            false,
            TargetFlags::NONE,
            move |w, from, t| {
                let skull = match t {
                    Targeted::Entity(EntityId::Item(i))
                        if w.item(i).is_some_and(|x| x.ty.is(&CHAMPION_SKULL)) =>
                    {
                        Some(i)
                    }
                    _ => None,
                };
                end_sacrifice(w, id, from, skull);
            },
        );
    } else {
        muo_server::item::messages::send_localized_message_to(w, id, from, 1049487, ""); // I already have my champions awakening skull!
    }
}

fn harrower_can_spawn(w: &World) -> bool {
    w.resources
        .get::<super::CannedEvilHooks>()
        .and_then(|h| h.harrower_can_spawn)
        .is_none_or(|f| f(w))
}

/// `EndSacrifice(from, skull)`.
fn end_sacrifice(w: &mut World, id: ItemId, from: MobileId, skull: Option<ItemId>) {
    if is_deleted(w, id) {
        return;
    }
    if brazier(w, id)
        .and_then(|d| d.skull)
        .is_some_and(|s| is_deleted(w, s))
    {
        set_skull(w, id, None);
    }
    let loc = world_location(w, id);
    let same_map = w.mobile(from).map(|m| m.map) == w.item(id).map(|i| i.map);
    let btype = brazier(w, id).map_or(ChampionSkullType::None, |d| d.ty);
    if !same_map || !muo_content::mx::in_range_point(w, from, loc, 3) {
        muo_content::mx::msg(w, from, 500446); // That is too far away.
    } else if !harrower_can_spawn(w) {
        muo_content::mx::send_message(w, from, "The harrower has already been spawned.");
    } else if skull.is_none() {
        muo_server::item::messages::send_localized_message_to(w, id, from, 1049488, ""); // That is not my champions awakening skull!
    } else if brazier(w, id).and_then(|d| d.skull).is_some() {
        muo_server::item::messages::send_localized_message_to(w, id, from, 1049487, ""); // I already have my champions awakening skull!
    } else if let Some(skull) = skull {
        let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
        if !ops::is_child_of(w, skull, pack) {
            muo_content::mx::msg(w, from, 1049486); // You can only sacrifice items that are in your backpack!
        } else if skull_type(w, skull) == btype {
            ops::set_movable(w, skull, false);
            let top = ops::get_world_top(w, id);
            let map = w.item(id).and_then(|i| i.map);
            ops::move_to_world_map(w, skull, top, map);
            set_skull(w, id, Some(skull));
        } else {
            muo_server::item::messages::send_localized_message_to(w, id, from, 1049488, ""); // That is not my champions awakening skull!
        }
    }
}

/// `[SerializationGenerator(1, false)] ChampionSkullBrazier : AddonComponent`.
pub static CHAMPION_SKULL_BRAZIER: ItemType = ItemType {
    full_name: "Server.Engines.CannedEvil.ChampionSkullBrazier",
    parent: Some(&addon::ADDON_COMPONENT),
    construct: None,
    label_number: |w, id| 1049489 + brazier(w, id).map_or(6, |d| d.ty as i32),
    on_double_click: |w, id, from| {
        if let Some(p) = brazier(w, id).and_then(|d| d.platform) {
            validate(w, p);
        }
        begin_sacrifice(w, id, from);
    },
    serialize: |w, id, out| {
        (addon::ADDON_COMPONENT.serialize)(w, id, out);
        let d = brazier(w, id).copied().unwrap_or(BrazierData {
            ty: ChampionSkullType::None,
            platform: None,
            skull: None,
        });
        out.write_i32(1);
        out.write_enum(4, d.ty as i64);
        out.write_item(w, d.platform);
        out.write_item(w, d.skull);
    },
    deserialize: |w, id, r| {
        (addon::ADDON_COMPONENT.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let ty = if version < 1 {
            r.read_i32()?
        } else {
            r.read_enum(4)? as i32
        };
        let platform = r.read_item(w)?;
        let skull = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(BrazierData {
                ty: ChampionSkullType::from_i32(ty),
                platform,
                skull,
            });
        }
        // `[AfterDeserialization(false)]`: no platform, no brazier.
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            if brazier(w, id).and_then(|d| d.platform).is_none() {
                delete_item(w, id);
            }
        });
        Ok(())
    },
    ..addon::ADDON_COMPONENT
};

// ---- ChampionSkullPlatform ----

/// The six braziers.
#[derive(Clone, Copy, Default)]
pub struct SkullPlatformData {
    pub braziers: [Option<ItemId>; 6],
}

fn platform_braziers(w: &World, id: ItemId) -> [Option<ItemId>; 6] {
    w.item(id)
        .and_then(|i| i.ext.get::<SkullPlatformData>())
        .map_or([None; 6], |d| d.braziers)
}

/// `Validate(brazier)`: `brazier is { Skull: { Deleted: false } }`.
fn validate_brazier(w: &World, b: Option<ItemId>) -> bool {
    b.and_then(|b| brazier(w, b))
        .and_then(|d| d.skull)
        .is_some_and(|s| !is_deleted(w, s))
}

/// `Validate()`: all six skulls in place calls the harrower.
pub fn validate(w: &mut World, id: ItemId) {
    let bs = platform_braziers(w, id);
    if bs.iter().all(|b| validate_brazier(w, *b)) {
        let (loc, map) = match w.item(id) {
            Some(i) => (i.location, i.map),
            None => return,
        };
        let harrower = w
            .resources
            .get::<super::CannedEvilHooks>()
            .and_then(|h| h.harrower_spawn)
            .and_then(|f| {
                f(
                    w,
                    muo_primitives::Point3D::new(loc.x, loc.y, loc.z + 6),
                    map,
                )
            });
        if harrower.is_none() {
            return;
        }
        for b in bs {
            clear(w, b);
        }
    }
}

/// `Clear(brazier)`.
fn clear(w: &mut World, b: Option<ItemId>) {
    let Some(b) = b else { return };
    muo_server::effects::send_bolt_effect(w, EntityId::Item(b), true, 0);
    if let Some(s) = brazier(w, b).and_then(|d| d.skull) {
        delete_item(w, s);
    }
}

/// `ChampionSkullPlatform()`.
pub fn skull_platform_new(w: &mut World) -> ItemId {
    let id = addon::new_addon(w, &CHAMPION_SKULL_PLATFORM);
    let plain = |w: &mut World, id: ItemId, g: i32, x: i32, y: i32, z: i32| {
        let c = addon::new_component(w, &addon::ADDON_COMPONENT, g);
        addon::add_component(w, id, c, x, y, z);
    };
    plain(w, id, 0x71A, -1, -1, -1);
    for (x, y) in [(0, -1), (1, -1), (-1, 0), (0, 0), (1, 0), (-1, 1), (0, 1)] {
        plain(w, id, 0x709, x, y, -1);
    }
    plain(w, id, 0x71B, 1, 1, -1);

    let mut bs = [None; 6];
    for (n, (ty, x, y)) in [
        (ChampionSkullType::Power, 0, -1),
        (ChampionSkullType::Enlightenment, 1, -1),
        (ChampionSkullType::Venom, -1, 0),
        (ChampionSkullType::Pain, 1, 0),
        (ChampionSkullType::Greed, -1, 1),
        (ChampionSkullType::Death, 0, 1),
    ]
    .into_iter()
    .enumerate()
    {
        plain(w, id, 0x50F, x, y, 4);
        let b = brazier_new(w, id, ty);
        addon::add_component(w, id, b, x, y, 5);
        bs[n] = Some(b);
    }
    for (g, label, x, y, z) in [
        (0x20D2, 1049495, 0, 0, 5),
        (0x0BCF, 1049496, 0, 2, -7),
        (0x0BD0, 1049497, 2, 0, -7),
    ] {
        let c = addon::new_localized_component(w, &addon::LOCALIZED_ADDON_COMPONENT, g, label);
        hue(w, c, 0x482);
        addon::add_component(w, id, c, x, y, z);
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(SkullPlatformData { braziers: bs });
    }
    id
}

/// `[SerializationGenerator(0, false)] ChampionSkullPlatform`.
pub static CHAMPION_SKULL_PLATFORM: ItemType = ItemType {
    full_name: "Server.Engines.CannedEvil.ChampionSkullPlatform",
    parent: Some(&addon::BASE_ADDON),
    construct: Some(skull_platform_new),
    family: Some(&CHAMPION_PLATFORM_CLASS),
    serialize: |w, id, out| {
        (addon::BASE_ADDON.serialize)(w, id, out);
        out.write_i32(0);
        for b in platform_braziers(w, id) {
            out.write_item(w, b);
        }
    },
    deserialize: |w, id, r| {
        (addon::BASE_ADDON.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let mut bs = [None; 6];
        for b in &mut bs {
            *b = r.read_item(w)?;
        }
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(SkullPlatformData { braziers: bs });
        }
        Ok(())
    },
    ..addon::BASE_ADDON
};

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &CHAMPION_PLATFORM,
        &CHAMPION_ALTAR,
        &CHAMPION_SKULL,
        &CHAMPION_SKULL_BRAZIER,
        &CHAMPION_SKULL_PLATFORM,
    ] {
        types.register_item(t, &[]);
    }
}

/// The fields of a `ChampionSkullBrazier`.
pub fn brazier_data(w: &World, id: ItemId) -> Option<BrazierData> {
    brazier(w, id).copied()
}
