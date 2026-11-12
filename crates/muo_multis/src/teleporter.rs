//! `HouseTeleporter`.

use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::effects;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, effect_item, ops};
use muo_server::mobile::AccessLevel;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

use crate::house::SecureLevel;
use crate::house::access::{self, msg};

/// What `HouseTeleporter` adds.
#[derive(Clone, Debug)]
pub struct TeleporterData {
    pub level: SecureLevel,
    pub target: Option<ItemId>,
}

fn data(w: &World, id: ItemId) -> TeleporterData {
    w.item(id)
        .and_then(|i| i.ext.get::<TeleporterData>())
        .cloned()
        .unwrap_or(TeleporterData {
            level: SecureLevel::Anyone,
            target: None,
        })
}

/// `ISecurable.Level` of a house teleporter, for the security entry.
pub fn securable_level(w: &World, id: ItemId) -> Option<u8> {
    w.item(id).filter(|i| i.ty.is(&HOUSE_TELEPORTER))?;
    Some(data(w, id).level as u8)
}

/// `securable.Level = level` of a house teleporter; false for any other item.
pub fn set_securable_level(w: &mut World, id: ItemId, level: u8) -> bool {
    if !w.item(id).is_some_and(|i| i.ty.is(&HOUSE_TELEPORTER)) {
        return false;
    }
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<TeleporterData>())
    {
        d.level = SecureLevel::from_u8(level);
    }
    true
}

/// `new HouseTeleporter(itemID, target)`.
pub fn new_house_teleporter(w: &mut World, item_id: i32, target: Option<ItemId>) -> ItemId {
    let id = ops::new_item_with_id(w, &HOUSE_TELEPORTER, item_id);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(TeleporterData {
            level: SecureLevel::Anyone,
            target,
        });
    }
    id
}

/// `tp.Target = target`.
pub fn set_target(w: &mut World, id: ItemId, target: Option<ItemId>) {
    if let Some(d) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<TeleporterData>())
    {
        d.target = target;
    }
}

fn live_target(w: &World, id: ItemId) -> Option<ItemId> {
    data(w, id)
        .target
        .filter(|&t| w.item(t).is_some_and(|i| !i.deleted()))
}

/// `CheckAccess(m)`.
pub fn check_access(w: &World, id: ItemId, m: MobileId) -> bool {
    let house = access::find_house_at_item(w, id);
    let ok = match house {
        None => true,
        Some(h) => {
            (crate::house::d(w, h).public && !access::is_banned(w, h, m))
                || access::has_access(w, h, m)
        }
    };
    ok && house.is_some_and(|h| access::has_secure_access(w, h, m, data(w, id).level))
}

fn effect_after(
    w: &mut World,
    p: muo_primitives::Point3D,
    map: Option<muo_server::map::MapId>,
    effect_id: i32,
    sound: i32,
    delay: TimeSpan,
) {
    timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        let e = effect_item::create_targeted(w, p, map, effect_item::DEFAULT_DURATION);
        effects::send_location_particles(w, e, 0x3728, 10, 10, effect_id);
        if sound != -1 {
            effects::play_sound(w, p, map, sound);
        }
    });
}

fn on_move_over(w: &mut World, id: ItemId, m: MobileId) -> bool {
    if live_target(w, id).is_none() {
        return true;
    }
    if !check_access(w, id, m) {
        msg(w, m, 1061637); // You are not allowed to access this.
        return false;
    }
    let (hidden, level) = w
        .mobile(m)
        .map_or((false, AccessLevel::Player), |x| (x.hidden, x.access_level));
    let (loc, map) = w.item(id).map(|i| (i.location, i.map)).unwrap_or_default();
    if !hidden || level == AccessLevel::Player {
        effect_after(w, loc, map, 2023, 0x1F0, TimeSpan::from_millis(400));
    }
    timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
        delay_tick(w, id, m)
    });
    true
}

fn delay_tick(w: &mut World, tp: ItemId, m: MobileId) {
    let Some(target) = live_target(w, tp) else {
        return;
    };
    let (tloc, tmap) = w.item(tp).map(|i| (i.location, i.map)).unwrap_or_default();
    let Some(mob) = w.mobile(m) else { return };
    if mob.location != tloc || mob.map != tmap {
        return;
    }
    let (hidden, level) = (mob.hidden, mob.access_level);
    let p = ops::get_world_top(w, target);
    let map = w.item(target).and_then(|i| i.map);
    if let Some(map) = map {
        muo_content::creature::pets::teleport_pets(w, m, p, map, false);
        muo_server::mobile::base::move_to_world(w, m, p, map);
    }
    if hidden && level != AccessLevel::Player {
        return;
    }
    let (tgt_loc, tgt_map) = w
        .item(target)
        .map(|i| (i.location, i.map))
        .unwrap_or_default();
    effects::play_sound(w, tgt_loc, tgt_map, 0x1FE);
    let e = effect_item::create_targeted(w, tloc, tmap, effect_item::DEFAULT_DURATION);
    effects::send_location_particles(w, e, 0x3728, 10, 10, 2023);
    let e = effect_item::create_targeted(w, tgt_loc, tgt_map, effect_item::DEFAULT_DURATION);
    effects::send_location_particles(w, e, 0x3728, 10, 10, 5023);
    effect_after(w, tgt_loc, tgt_map, 2023, -1, TimeSpan::from_millis(400));
}

/// `Server.Items.HouseTeleporter`.
pub static HOUSE_TELEPORTER: ItemType =
    ItemType {
        full_name: "Server.Items.HouseTeleporter",
        parent: Some(&ITEM_BASE),
        construct: None,
        on_move_over,
        get_context_menu_entries:
            |w: &mut World, id: ItemId, from: MobileId, list: &mut Vec<ContextMenuEntry>| {
                (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
                if let Some(e) = crate::secure_entry::add_to(w, from, id) {
                    list.push(e);
                }
            },
        serialize: |w, id, out| {
            (ITEM_BASE.serialize)(w, id, out);
            out.write_i32(2); // version
            let d = data(w, id);
            out.write_enum(4, d.level as i64);
            out.write_item(w, d.target);
        },
        deserialize: |w, id, r| {
            (ITEM_BASE.deserialize)(w, id, r)?;
            let version = r.read_i32()?;
            // older versions: the level as a plain int, then the target
            let level = if version < 2 {
                r.read_i32()?
            } else {
                r.read_enum(4)? as i32
            };
            let target = r.read_item(w)?;
            if let Some(i) = w.item_mut(id) {
                i.ext.insert(TeleporterData {
                    level: SecureLevel::from_u8(level as u8),
                    target,
                });
            }
            Ok(())
        },
        ..ITEM_BASE
    };

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&HOUSE_TELEPORTER, &[]);
}
