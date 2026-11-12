//! House doors, keys, the sign and the trash barrel (`AddEastDoor`.. `AddDoor(itemID)`, `CreateKeys`,
//! `MakeDoor`, `SetSign`, `AddTrashBarrel`, `ChangeLocks`, `RemoveLocks`).

use muo_content::items::gold::{self, KeyType};
use muo_content::items::house_deps::{DoorFacing, HouseDoorKind, deps};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{LootType, ops};

use super::access::{is_co_owner, msg};
use super::{data, data_mut, deleted, is_active, is_aos_rules, location, map_of};

/// `Key.RandomValue()` then, for pre-AOS houses, the key in the pack and the bank box. `CreateKeys(m)`.
pub fn create_keys(w: &mut World, house: ItemId, m: MobileId) -> u32 {
    let value = gold::random_value(w);
    if !is_aos_rules(w, house) {
        let pack_key = gold::new_key_of(w, KeyType::Gold, 0, None);
        let bank_key = gold::new_key_of(w, KeyType::Gold, 0, None);
        gold::set_key_value(w, pack_key, value);
        gold::set_key_value(w, bank_key, value);
        ops::set_loot_type(w, pack_key, LootType::Newbied);
        ops::set_loot_type(w, bank_key, LootType::Newbied);

        let bank = muo_server::mobile::part1::props_b::bank_box(w, m);
        let dropped = bank.is_some_and(|b| {
            muo_server::item::container::try_drop_item(w, b, Some(m), bank_key, false)
        });
        if !dropped {
            delete_item(w, bank_key);
        }
        muo_server::mobile::part2::items::add_to_backpack(w, m, pack_key);
    }
    value
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// `MakeDoor(wood, facing)`.
pub fn make_door(w: &mut World, wood: bool, facing: DoorFacing) -> Option<ItemId> {
    let f = deps(w)?.make_door?;
    f(
        w,
        if wood {
            HouseDoorKind::DarkWood(facing)
        } else {
            HouseDoorKind::Metal(facing)
        },
    )
}

fn lock_door(w: &mut World, door: ItemId, k: u32) {
    if let Some(f) = deps(w).and_then(|x| x.door_set_locked) {
        f(w, door, true);
    }
    if let Some(f) = deps(w).and_then(|x| x.door_set_key_value) {
        f(w, door, k);
    }
}

/// `AddDoor(door, xoff, yoff, zoff)`: `door.MoveToWorld(..); Doors.Add(door)`.
pub fn add_door(w: &mut World, house: ItemId, door: ItemId, xoff: i32, yoff: i32, zoff: i32) {
    let l = location(w, house);
    let map = map_of(w, house);
    ops::move_to_world_map(
        w,
        door,
        Point3D::new(xoff + l.x, yoff + l.y, zoff + l.z),
        map,
    );
    if let Some(h) = data_mut(w, house) {
        h.doors.push(door);
    }
}

/// `AddEastDoor(wood, x, y, z)`.
pub fn add_east_door(
    w: &mut World,
    house: ItemId,
    wood: bool,
    x: i32,
    y: i32,
    z: i32,
) -> Option<ItemId> {
    let door = make_door(w, wood, DoorFacing::SouthCW)?;
    add_door(w, house, door, x, y, z);
    Some(door)
}

/// `AddSouthDoor(wood, x, y, z)`.
pub fn add_south_door(
    w: &mut World,
    house: ItemId,
    wood: bool,
    x: i32,
    y: i32,
    z: i32,
) -> Option<ItemId> {
    let door = make_door(w, wood, DoorFacing::WestCW)?;
    add_door(w, house, door, x, y, z);
    Some(door)
}

/// `AddEastDoor(wood, x, y, z, k)`.
pub fn add_east_door_k(
    w: &mut World,
    house: ItemId,
    wood: bool,
    x: i32,
    y: i32,
    z: i32,
    k: u32,
) -> Option<ItemId> {
    let door = make_door(w, wood, DoorFacing::SouthCW)?;
    lock_door(w, door, k);
    add_door(w, house, door, x, y, z);
    Some(door)
}

/// `AddSouthDoor(wood, x, y, z, k)`.
pub fn add_south_door_k(
    w: &mut World,
    house: ItemId,
    wood: bool,
    x: i32,
    y: i32,
    z: i32,
    k: u32,
) -> Option<ItemId> {
    let door = make_door(w, wood, DoorFacing::WestCW)?;
    lock_door(w, door, k);
    add_door(w, house, door, x, y, z);
    Some(door)
}

fn link(w: &mut World, a: ItemId, b: ItemId) {
    if let Some(f) = deps(w).and_then(|x| x.door_set_link) {
        f(w, a, b);
        f(w, b, a);
    }
}

/// `AddSouthDoors(wood, x, y, z, k)`: a locked pair.
pub fn add_south_doors_k(w: &mut World, house: ItemId, wood: bool, x: i32, y: i32, z: i32, k: u32) {
    let west = make_door(w, wood, DoorFacing::WestCW);
    let east = make_door(w, wood, DoorFacing::EastCCW);
    let (Some(west), Some(east)) = (west, east) else {
        return;
    };
    lock_door(w, west, k);
    lock_door(w, east, k);
    link(w, west, east);
    add_door(w, house, west, x, y, z);
    add_door(w, house, east, x + 1, y, z);
}

/// `AddSouthDoors(wood, x, y, z, inv)`: an unlocked pair.
pub fn add_south_doors(
    w: &mut World,
    house: ItemId,
    wood: bool,
    x: i32,
    y: i32,
    z: i32,
    inv: bool,
) {
    let west = make_door(
        w,
        wood,
        if inv {
            DoorFacing::WestCCW
        } else {
            DoorFacing::WestCW
        },
    );
    let east = make_door(
        w,
        wood,
        if inv {
            DoorFacing::EastCW
        } else {
            DoorFacing::EastCCW
        },
    );
    let (Some(west), Some(east)) = (west, east) else {
        return;
    };
    link(w, west, east);
    add_door(w, house, west, x, y, z);
    add_door(w, house, east, x + 1, y, z);
}

/// `GetSADoorFacing(offset)`.
fn sa_door_facing(offset: i32) -> DoorFacing {
    DoorFacing::from_i32((offset / 2 + 2 * (1 + offset / 4)) % 8)
}

fn generic(facing: DoorFacing, base: i32, open: i32, close: i32, auto: bool) -> HouseDoorKind {
    HouseDoorKind::Generic {
        facing,
        base_item_id: base,
        open_sound: open,
        close_sound: close,
        auto_adjust: auto,
    }
}

/// The door class `AddDoor(itemID, ..)` builds for a door graphic of a house template; None is "not supported".
pub fn door_for_item_id(item_id: i32) -> Option<HouseDoorKind> {
    let f = DoorFacing::from_i32;
    let k = if (0x675..0x6F5).contains(&item_id) {
        let ty = (item_id - 0x675) / 16;
        let facing = f((item_id - 0x675) / 2 % 8);
        match ty {
            0 => generic(facing, 0x675, 0xEC, 0xF3, true),
            1 => generic(facing, 0x685, 0xEC, 0xF3, true),
            2 => generic(facing, 0x695, 0xEB, 0xF2, true),
            3 => generic(facing, 0x6A5, 0xEA, 0xF1, true),
            4 => generic(facing, 0x6B5, 0xEA, 0xF1, true),
            5 => generic(facing, 0x6C5, 0xEC, 0xF3, true),
            6 => generic(facing, 0x6D5, 0xEA, 0xF1, true),
            _ => generic(facing, 0x6E5, 0xEA, 0xF1, true),
        }
    } else if (0x314..0x364).contains(&item_id) {
        let ty = (item_id - 0x314) / 16;
        generic(
            f((item_id - 0x314) / 2 % 8),
            0x314 + ty * 16,
            0xED,
            0xF4,
            true,
        )
    } else if (0x824..0x834).contains(&item_id) {
        generic(f((item_id - 0x824) / 2 % 8), 0x824, 0xEC, 0xF3, true)
    } else if (0x839..0x849).contains(&item_id) {
        generic(f((item_id - 0x839) / 2 % 8), 0x839, 0xEB, 0xF2, true)
    } else if (0x84C..0x85C).contains(&item_id) {
        generic(f((item_id - 0x84C) / 2 % 8), 0x84C, 0xEC, 0xF3, true)
    } else if (0x866..0x876).contains(&item_id) {
        generic(f((item_id - 0x866) / 2 % 8), 0x866, 0xEB, 0xF2, true)
    } else if (0xE8..0xF8).contains(&item_id) {
        generic(f((item_id - 0xE8) / 2 % 8), 0xE8, 0xED, 0xF4, true)
    } else if (0x1FED..0x1FFD).contains(&item_id) {
        generic(f((item_id - 0x1FED) / 2 % 8), 0x1FED, 0xEC, 0xF3, true)
    } else if (0x241F..0x2421).contains(&item_id) {
        generic(DoorFacing::NorthCCW, 0x2415, -1, -1, true)
    } else if (0x2423..0x2425).contains(&item_id) {
        // OSI had the ItemID pattern discombobulated for these
        generic(DoorFacing::WestCW, 0x2423, -1, -1, true)
    } else if (0x2A05..0x2A1D).contains(&item_id) {
        let facing = f((item_id - 0x2A05) / 2 % 4 + 8);
        let sound = if (0x2A0D..0x2a15).contains(&item_id) {
            0x539
        } else {
            -1
        };
        generic(
            facing,
            0x29F5 + 8 * ((item_id - 0x2A05) / 8),
            sound,
            sound,
            true,
        )
    } else if item_id == 0x2D46 {
        generic(DoorFacing::NorthCW, 0x2D46, 0xEA, 0xF1, false)
    } else if item_id == 0x2D48 || item_id == 0x2FE2 {
        generic(DoorFacing::SouthCCW, item_id, 0xEA, 0xF1, false)
    } else if (0x2D63..0x2D70).contains(&item_id) {
        let m = (item_id - 0x2D63) / 2 % 2;
        let facing = if m == 0 {
            DoorFacing::SouthCCW
        } else {
            DoorFacing::WestCCW
        };
        let ty = (item_id - 0x2D63) / 4;
        generic(facing, 0x2D63 + 4 * ty + m * 2, 0xEA, 0xF1, false)
    } else if item_id == 0x2FE4 || item_id == 0x31AE {
        generic(DoorFacing::WestCCW, item_id, 0xEA, 0xF1, false)
    } else if (0x319C..0x31AE).contains(&item_id) {
        // special case for 0x31aa <-> 0x31a8 (a9)
        let m = (item_id - 0x319C) / 2 % 2;
        let facing = match item_id {
            0x31AA | 0x31A8 => {
                if m == 0 {
                    DoorFacing::NorthCW
                } else {
                    DoorFacing::EastCW
                }
            }
            _ => {
                if m == 0 {
                    DoorFacing::EastCW
                } else {
                    DoorFacing::NorthCW
                }
            }
        };
        let ty = (item_id - 0x319C) / 4;
        generic(facing, 0x319C + 4 * ty + m * 2, 0xEA, 0xF1, false)
    } else if (0x367B..0x369B).contains(&item_id) {
        let ty = (item_id - 0x367B) / 16;
        let facing = f((item_id - 0x367B) / 2 % 8);
        if ty == 0 {
            generic(facing, 0x367B, 0xED, 0xF4, true)
        } else {
            generic(facing, 0x368B, 0xEC, 0x3E7, true)
        }
    } else if (0x409B..0x40A3).contains(&item_id) {
        generic(sa_door_facing(item_id - 0x409B), item_id, 0xEA, 0xF1, false)
    } else if (0x410C..0x4114).contains(&item_id) {
        generic(sa_door_facing(item_id - 0x410C), item_id, 0xEA, 0xF1, false)
    } else if (0x41C2..0x41CA).contains(&item_id) {
        generic(sa_door_facing(item_id - 0x41C2), item_id, 0xEA, 0xF1, false)
    } else if (0x41CF..0x41D7).contains(&item_id) {
        generic(sa_door_facing(item_id - 0x41CF), item_id, 0xEA, 0xF1, false)
    } else if (0x436E..0x437E).contains(&item_id) {
        // Offset 0 2 4 6 8 10 12 14 -> DoorFacing 2 3 2 3 6 7 6 7
        let offset = item_id - 0x436E;
        let facing = f((offset / 2 + 2 * ((1 + offset / 4) % 2)) % 8);
        generic(facing, item_id, 0xEA, 0xF1, false)
    } else if (0x46DD..0x46E5).contains(&item_id) {
        generic(sa_door_facing(item_id - 0x46DD), item_id, 0xEB, 0xF2, false)
    } else if (0x4D22..0x4D2A).contains(&item_id) {
        generic(sa_door_facing(item_id - 0x4D22), item_id, 0xEA, 0xF1, false)
    } else if (0x50C8..0x50D0).contains(&item_id) {
        generic(sa_door_facing(item_id - 0x50C8), item_id, 0xEA, 0xF1, false)
    } else if (0x50D0..0x50D8).contains(&item_id) {
        generic(sa_door_facing(item_id - 0x50D0), item_id, 0xEA, 0xF1, false)
    } else if (0x5142..0x514A).contains(&item_id) {
        generic(sa_door_facing(item_id - 0x5142), item_id, 0xF0, 0xEF, false)
    } else if (0x9AD7..=0x9AE6).contains(&item_id) {
        // this heuristic is known to be broken
        let ty = (item_id - 0x9AD7) / 16;
        generic(
            f((item_id - 0x9AD7) / 2 % 8),
            0x9AD7 + ty * 16,
            0xED,
            0xF4,
            true,
        )
    } else if (0x9B3C..=0x9B4B).contains(&item_id) {
        let ty = (item_id - 0x9B3C) / 16;
        generic(
            f((item_id - 0x9B3C) / 2 % 8),
            0x9B3C + ty * 16,
            0xED,
            0xF4,
            true,
        )
    } else {
        return None;
    };
    Some(k)
}

/// `AddDoor(from, itemID, xOffset, yOffset, zOffset)`.
pub fn add_door_item_id(
    w: &mut World,
    house: ItemId,
    from: Option<MobileId>,
    item_id: i32,
    x: i32,
    y: i32,
    z: i32,
) -> Option<ItemId> {
    let Some(kind) = door_for_item_id(item_id) else {
        tracing::warn!("BaseHouse: Door ItemID {item_id} not supported.");
        return None;
    };
    let door = deps(w).and_then(|d| d.make_door).and_then(|f| f(w, kind))?;
    if let Some(m) = from {
        let k = create_keys(w, house, m);
        if let Some(f) = deps(w).and_then(|d| d.door_set_key_value) {
            f(w, door, k);
        }
    }
    add_door(w, house, door, x, y, z);
    Some(door)
}

/// `SetSign(xoff, yoff, zoff)`.
pub fn set_sign(w: &mut World, house: ItemId, xoff: i32, yoff: i32, zoff: i32) {
    let sign = crate::sign::new_house_sign(w, house);
    if let Some(h) = data_mut(w, house) {
        h.sign = Some(sign);
    }
    let l = location(w, house);
    let map = map_of(w, house);
    ops::move_to_world_map(
        w,
        sign,
        Point3D::new(l.x + xoff, l.y + yoff, l.z + zoff),
        map,
    );
}

/// `ChangeSignType(itemID)`.
pub fn change_sign_type(w: &mut World, house: ItemId, item_id: i32) {
    if let Some(s) = data(w, house).and_then(|h| h.sign) {
        let ty = w.item(s).map(|i| i.ty);
        if let Some(ty) = ty {
            (ty.set_item_id)(w, s, item_id);
        }
    }
}

/// `door.Open ? door.Location - door.Offset : door.Location` for the doors the strong box and trash barrel keep
/// away from.
fn door_closed_location(w: &World, door: ItemId) -> Point3D {
    let p = location(w, door);
    match deps(w)
        .and_then(|x| x.door_open_offset)
        .and_then(|f| f(w, door))
    {
        Some(o) => Point3D::new(p.x - o.x, p.y - o.y, p.z - o.z),
        None => p,
    }
}

/// Is a door within one tile of `from` at about the same height; the check of `AddTrashBarrel`/`AddStrongBox`.
pub fn door_near(w: &World, house: ItemId, from: MobileId) -> bool {
    let Some(m) = w.mobile(from) else {
        return false;
    };
    let doors = data(w, house).map(|h| h.doors.clone()).unwrap_or_default();
    for door in doors {
        let p = door_closed_location(w, door);
        if m.location.z + 16 >= p.z
            && p.z + 16 >= m.location.z
            && (m.location.x - p.x).abs().max((m.location.y - p.y).abs()) <= 1
        {
            return true;
        }
    }
    false
}

/// `AddTrashBarrel(from)`.
pub fn add_trash_barrel(w: &mut World, house: ItemId, from: MobileId) {
    if !is_active(w, house) {
        return;
    }
    if door_near(w, house, from) {
        msg(w, from, 502120); // You cannot place a trash barrel near a door or near steps.
        return;
    }
    let trash = data(w, house).and_then(|h| h.trash);
    if trash.is_none_or(|t| deleted(w, t)) {
        let Some(t) = muo_content::factory::make_item(w, "Server.Items.TrashBarrel") else {
            return;
        };
        ops::set_movable(w, t, false);
        if let Some(h) = data_mut(w, house) {
            h.trash = Some(t);
        }
        let (loc, map) = w
            .mobile(from)
            .map(|m| (m.location, m.map))
            .unwrap_or((Point3D::ZERO, None));
        ops::move_to_world_map(w, t, loc, map);
        /* You have a new trash barrel. Three minutes after you put something in the barrel, the trash will be
         * emptied. Be forewarned, this is permanent! */
        msg(w, from, 502121);
    } else {
        msg(w, from, 502117); // You already have a trash barrel!
    }
}

/// `ChangeLocks(m)`.
pub fn change_locks(w: &mut World, house: ItemId, m: MobileId) {
    let key = create_keys(w, house, m);
    let doors = data(w, house).map(|h| h.doors.clone()).unwrap_or_default();
    if let Some(f) = deps(w).and_then(|d| d.door_set_key_value) {
        for door in doors {
            f(w, door, key);
        }
    }
}

/// `RemoveLocks()`.
pub fn remove_locks(w: &mut World, house: ItemId) {
    let doors = data(w, house).map(|h| h.doors.clone()).unwrap_or_default();
    for door in doors {
        if let Some(f) = deps(w).and_then(|d| d.door_set_key_value) {
            f(w, door, 0);
        }
        if let Some(f) = deps(w).and_then(|d| d.door_set_locked) {
            f(w, door, false);
        }
    }
}

/// `RemoveKeys(m)`: the keys of the first door that has one.
pub fn remove_keys(w: &mut World, house: ItemId, m: MobileId) {
    let doors = data(w, house).map(|h| h.doors.clone()).unwrap_or_default();
    let get = deps(w).and_then(|d| d.door_key_value);
    let mut key = 0;
    for door in doors {
        if key != 0 {
            break;
        }
        key = get.map_or(0, |f| f(w, door));
    }
    gold::remove_keys_mobile(w, m, key);
}

/// Co-owner shortcut used by the targets.
pub fn co_owner(w: &World, house: ItemId, m: MobileId) -> bool {
    is_co_owner(w, house, m)
}
