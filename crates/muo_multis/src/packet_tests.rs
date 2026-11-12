//! House and boat packet tests. They compare the span-built packet with the legacy `Packet` classes, written out
//! here as the oracle.

use muo_data::TileFlag;
use muo_data::client::multi_data::{MultiComponentList, MultiTileEntry};
use muo_primitives::{Point3D, Serial};
use muo_server::World;
use muo_server::ids::{EntityId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::MOBILE_BASE;
use muo_server::net_state::{NetOut, NetStateData, NetStateId};

use crate::house_packets::*;

const MAX_ITEMS_PER_STAIR_BUFFER: usize = 750;

fn world() -> (World, NetStateId) {
    let mut w = muo_server::map::test_support::world();
    let ns = NetStateId(1);
    w.net_states.insert(ns, NetStateData::new(0));
    (w, ns)
}

/// `ns.SendBuffer.GetReadSpan()`.
fn sent(w: &mut World) -> Vec<u8> {
    let mut all = Vec::new();
    for o in std::mem::take(&mut w.net_out) {
        if let NetOut::Send(_, b) = o {
            all.extend_from_slice(&b);
        }
    }
    all
}

fn legacy_customization(house: u32, command: u8) -> Vec<u8> {
    let mut b = vec![0xBF, 0, 17];
    b.extend_from_slice(&0x20i16.to_be_bytes());
    b.extend_from_slice(&house.to_be_bytes());
    b.push(command);
    b.extend_from_slice(&0u16.to_be_bytes());
    b.extend_from_slice(&0xFFFFu16.to_be_bytes());
    b.extend_from_slice(&0xFFFFu16.to_be_bytes());
    b.push(0xFF);
    b
}

#[test]
fn begin_house_customization() {
    {
        let serial = 0x1001u32;
        let (mut w, ns) = world();
        send_begin_house_customization(&mut w, ns, Serial::new(serial));
        assert_eq!(sent(&mut w), legacy_customization(serial, 0x04));
    }
}

#[test]
fn end_house_customization() {
    {
        let serial = 0x1001u32;
        let (mut w, ns) = world();
        send_end_house_customization(&mut w, ns, Serial::new(serial));
        assert_eq!(sent(&mut w), legacy_customization(serial, 0x05));
    }
}

#[test]
fn design_state_general() {
    for (serial, revision) in [(0x1001u32, 0i32), (0x1001, 100)] {
        let (mut w, ns) = world();
        let mut expected = vec![0xBF, 0, 13];
        expected.extend_from_slice(&0x1Di16.to_be_bytes());
        expected.extend_from_slice(&serial.to_be_bytes());
        expected.extend_from_slice(&revision.to_be_bytes());

        send_design_state_general(&mut w, ns, Serial::new(serial), revision);
        assert_eq!(sent(&mut w), expected, "revision {revision}");
    }
}

/// `DesignStateDetailed` (the legacy oracle).
fn legacy_design_state_detailed(
    serial: u32,
    revision: i32,
    (x_min, y_min, x_max, y_max): (i32, i32, i32, i32),
    tiles: &[MultiTileEntry],
    floor: impl Fn(u16) -> bool,
) -> Vec<u8> {
    let pack = |src: &[u8]| -> Vec<u8> {
        let mut dest = vec![0u8; muo_io::compression::Level::compress_bound(src.len()).max(16)];
        let n = muo_io::compression::STANDARD.pack(&mut dest, src);
        assert_ne!(n, 0, "Compression error");
        dest.truncate(n);
        dest
    };

    let mut out = vec![0xD8, 0, 0];
    out.push(0x03); // Compression Type
    out.push(0x00); // Unknown
    out.extend_from_slice(&serial.to_be_bytes());
    out.extend_from_slice(&revision.to_be_bytes());
    out.extend_from_slice(&(tiles.len() as i16).to_be_bytes());
    out.extend_from_slice(&0i16.to_be_bytes()); // Buffer length : reserved
    out.push(0); // Plane count : reserved

    let mut total_length = 1usize; // includes plane count
    let width = x_max - x_min + 1;
    let height = y_max - y_min + 1;

    let mut plane_buffers = vec![vec![0u8; 0x400]; 9];
    let mut stair_buffers = vec![vec![0u8; MAX_ITEMS_PER_STAIR_BUFFER * 5]; 6];
    let mut plane_used = [false; 9];
    let mut total_stairs_used = 0usize;

    let mut add_stair = |stairs: &mut Vec<Vec<u8>>, mte: &MultiTileEntry| {
        let buffer = &mut stairs[total_stairs_used / MAX_ITEMS_PER_STAIR_BUFFER];
        let mut at = total_stairs_used % MAX_ITEMS_PER_STAIR_BUFFER * 5;
        buffer[at] = (mte.item_id >> 8) as u8;
        at += 1;
        buffer[at] = mte.item_id as u8;
        at += 1;
        buffer[at] = mte.offset_x as u8;
        at += 1;
        buffer[at] = mte.offset_y as u8;
        at += 1;
        buffer[at] = mte.offset_z as u8;
        total_stairs_used += 1;
    };

    for mte in tiles {
        let mut x = i32::from(mte.offset_x) - x_min;
        let mut y = i32::from(mte.offset_y) - y_min;
        let z = i32::from(mte.offset_z);
        let is_floor = floor(mte.item_id);
        let mut plane = match z {
            0 => 0,
            7 => 1,
            27 => 2,
            47 => 3,
            67 => 4,
            _ => {
                add_stair(&mut stair_buffers, mte);
                continue;
            }
        };
        let size;
        if plane == 0 {
            size = height;
        } else if is_floor {
            size = height - 2;
            x -= 1;
            y -= 1;
        } else {
            size = height - 1;
            plane += 4;
        }
        let index = (x * size + y) * 2;
        if x < 0 || y < 0 || y >= size || index + 1 >= 0x400 {
            add_stair(&mut stair_buffers, mte);
        } else {
            plane_used[plane] = true;
            plane_buffers[plane][index as usize] = (mte.item_id >> 8) as u8;
            plane_buffers[plane][index as usize + 1] = mte.item_id as u8;
        }
    }

    let mut plane_count = 0u8;
    for (i, buffer) in plane_buffers.iter().enumerate() {
        if !plane_used[i] {
            continue;
        }
        plane_count += 1;
        let size = match i {
            0 => width * height * 2,
            1..=4 => (width - 1) * (height - 2) * 2,
            _ => width * (height - 1) * 2,
        } as usize;
        let deflated = pack(&buffer[..size]);
        out.push((0x20 | i) as u8);
        out.push(size as u8);
        out.push(deflated.len() as u8);
        out.push((((size >> 4) & 0xF0) | ((deflated.len() >> 8) & 0xF)) as u8);
        out.extend_from_slice(&deflated);
        total_length += 4 + deflated.len();
    }

    let used_buffers = total_stairs_used.div_ceil(MAX_ITEMS_PER_STAIR_BUFFER);
    for (i, buffer) in stair_buffers.iter().enumerate().take(used_buffers) {
        plane_count += 1;
        let count =
            MAX_ITEMS_PER_STAIR_BUFFER.min(total_stairs_used - i * MAX_ITEMS_PER_STAIR_BUFFER);
        let size = count * 5;
        let deflated = pack(&buffer[..size]);
        out.push((9 + i) as u8);
        out.push(size as u8);
        out.push(deflated.len() as u8);
        out.push((((size >> 4) & 0xF0) | ((deflated.len() >> 8) & 0xF)) as u8);
        out.extend_from_slice(&deflated);
        total_length += 4 + deflated.len();
    }

    out[15..17].copy_from_slice(&(total_length as i16).to_be_bytes()); // Buffer length
    out[17] = plane_count; // Plane count
    let len = out.len() as u16;
    out[1..3].copy_from_slice(&len.to_be_bytes());
    out
}

#[test]
fn house_design_state_detailed() {
    let (w, _) = world();
    let serial = 0x4000_0001u32;
    let revision = 10;
    let tiles: Vec<MultiTileEntry> = (0..250i32)
        .map(|i| {
            MultiTileEntry::new(
                i as u16,
                i as u8 as i16,
                i as u8 as i16,
                (i / 50) as i16,
                TileFlag::empty(),
            )
        })
        .collect();
    let mcl = MultiComponentList::from_entries(&tiles);

    let expected = legacy_design_state_detailed(
        serial,
        revision,
        (mcl.min().x, mcl.min().y, mcl.max().x, mcl.max().y),
        &tiles,
        |id| w.maps.tile_data.item(usize::from(id)).height == 0,
    );

    let actual = create_house_design_state_detailed(&w, Serial::new(serial), revision, &mcl);

    assert_eq!(actual, expected);
}

// ---- BoatPacketTests ----

fn item_at(w: &mut World, serial: u32, loc: Point3D) -> muo_server::ids::ItemId {
    use muo_server::item::{ITEM_BASE, Item};
    let id = w.add_item(Item::new_for_load(Serial::new(serial), &ITEM_BASE));
    w.item_mut(id).unwrap().item_id = 0x13B9;
    muo_server::item::ops::move_to_world_map(w, id, loc, Some(MapId::FELUCCA));
    id
}

/// The beholder `MockedMobile((Serial)0x100)` at (10, 20, 15) on Felucca.
fn beholder(w: &mut World) -> MobileId {
    let m = w.add_mobile(muo_server::mobile::Mobile::new_for_load(
        Serial::new(0x100),
        &MOBILE_BASE,
        0,
    ));
    muo_server::mobile::part2::stats::default_mobile_init(w, m);
    muo_server::mobile::part1::lifecycle::move_to_world(
        w,
        m,
        Point3D::new(10, 20, 15),
        MapId::FELUCCA,
    );
    m
}

/// `TestBoat`: empty components over a fixed 4x3 rectangle at (9, 19), and `Contains(x, y)` true everywhere except
/// at (100, 200), where item2 lies.
static TEST_BOAT: muo_server::item::ItemType = muo_server::item::ItemType {
    full_name: "Server.Multis.TestBoat",
    parent: Some(&crate::boats::items::SMALL_BOAT),
    multi_contains: |_, _, x, y| !(x == 100 && y == 200),
    components: |w, id| {
        w.item(id)
            .and_then(|i| i.ext.get::<MultiComponentList>())
            .unwrap_or_else(|| MultiComponentList::empty())
    },
    ..crate::boats::items::SMALL_BOAT
};

/// A TestBoat at (10, 20, 15) on Felucca, facing `Direction.Right`.
fn boat(w: &mut World) -> muo_server::ids::ItemId {
    let b = crate::boats::items::new_small_boat(w);
    w.item_mut(b).unwrap().ty = &TEST_BOAT;
    // components spanning offsets (-1, -1) to (2, 1): the rectangle (9, 19, 4, 3) around the boat at (10, 20)
    let entries = [
        MultiTileEntry::new(0x13B9, -1, -1, 0, TileFlag::from_bits_retain(1)),
        MultiTileEntry::new(0x13B9, 2, 1, 0, TileFlag::from_bits_retain(1)),
    ];
    w.item_mut(b)
        .unwrap()
        .ext
        .insert(MultiComponentList::from_entries(&entries));
    crate::boats::set_facing(w, b, 2);
    muo_server::item::ops::move_to_world_map(w, b, Point3D::new(10, 20, 15), Some(MapId::FELUCCA));
    b
}

#[test]
fn move_boat_hs() {
    // [InlineData(Direction.West, 10, 100, 200)]
    let (d, speed, x_offset, y_offset) = (6, 10, 100, 200);
    let (mut w, ns) = world();
    let item1 = item_at(&mut w, 0x1000, Point3D::new(11, 21, 16));
    let _item2 = item_at(&mut w, 0x2000, Point3D::new(100, 200, 16));
    let who = beholder(&mut w);
    let b = boat(&mut w);
    let list = [EntityId::Mobile(who), EntityId::Item(item1)];

    // MoveBoatHS (legacy)
    let bt = w.item(b).unwrap();
    let mut expected = vec![0xF6, 0, 0];
    expected.extend_from_slice(&bt.serial.value().to_be_bytes());
    expected.push(speed as u8);
    expected.push(d as u8);
    expected.push(crate::boats::get_facing(&w, b).bits());
    expected.extend_from_slice(&((bt.location.x + x_offset) as i16).to_be_bytes());
    expected.extend_from_slice(&((bt.location.y + y_offset) as i16).to_be_bytes());
    expected.extend_from_slice(&(bt.location.z as i16).to_be_bytes());
    expected.extend_from_slice(&(list.len() as i16).to_be_bytes());
    for (serial, loc) in [
        (
            w.mobile(who).unwrap().serial,
            w.mobile(who).unwrap().location,
        ),
        (
            w.item(item1).unwrap().serial,
            w.item(item1).unwrap().location,
        ),
    ] {
        expected.extend_from_slice(&serial.value().to_be_bytes());
        expected.extend_from_slice(&((loc.x + x_offset) as i16).to_be_bytes());
        expected.extend_from_slice(&((loc.y + y_offset) as i16).to_be_bytes());
        expected.extend_from_slice(&(loc.z as i16).to_be_bytes());
    }
    let len = expected.len() as u16;
    expected[1..3].copy_from_slice(&len.to_be_bytes());

    w.net_states.get_mut(&ns).unwrap().caps.protocol_changes =
        muo_net::protocol_changes::ProtocolChanges::HIGH_SEAS;
    crate::boats::packets::send_move_boat_hs(&mut w, ns, b, &list, d, speed, x_offset, y_offset);

    assert_eq!(sent(&mut w), expected);
}

#[test]
fn display_boat_hs() {
    let (mut w, ns) = world();
    let _item1 = item_at(&mut w, 0x1000, Point3D::new(11, 21, 16));
    let _item2 = item_at(&mut w, 0x2000, Point3D::new(100, 200, 16));
    let who = beholder(&mut w);
    let b = boat(&mut w);
    w.net_states.get_mut(&ns).unwrap().caps.protocol_changes =
        muo_net::protocol_changes::ProtocolChanges::HIGH_SEAS;

    // DisplayBoatHS (legacy): a 0xF7 of embedded 0xF3 world entities the beholder can see.
    let mut body = Vec::new();
    let mut count = 0i16;
    for e in crate::boats::get_moving_entities(&w, b, true) {
        let sees = match e {
            EntityId::Item(i) => muo_server::mobile::base::can_see_item(&mut w, who, i),
            EntityId::Mobile(m) => muo_server::mobile::base::can_see_mobile(&mut w, who, m),
        };
        if !sees {
            continue;
        }
        body.extend_from_slice(&[0xF3, 0, 1]);
        match e {
            EntityId::Item(i) => {
                let it = w.item(i).unwrap();
                body.push(if it.ty.is(&muo_server::item::multi::BASE_MULTI) {
                    0x02
                } else {
                    0x00
                });
                body.extend_from_slice(&it.serial.value().to_be_bytes());
                body.extend_from_slice(&(it.item_id as u16).to_be_bytes());
                body.push(0);
                body.extend_from_slice(&(it.amount as i16).to_be_bytes());
                body.extend_from_slice(&(it.amount as i16).to_be_bytes());
                body.extend_from_slice(&((it.location.x & 0x7FFF) as i16).to_be_bytes());
                body.extend_from_slice(&((it.location.y & 0x3FFF) as i16).to_be_bytes());
                body.push(it.location.z as u8);
                body.push(it.direction.bits());
                body.extend_from_slice(&(it.hue as i16).to_be_bytes());
                body.push((it.ty.get_packet_flags)(&w, i) as u8);
            }
            EntityId::Mobile(m) => {
                let mb = w.mobile(m).unwrap();
                body.push(0x01);
                body.extend_from_slice(&mb.serial.value().to_be_bytes());
                body.extend_from_slice(&(mb.body.0 as i16).to_be_bytes());
                body.push(0);
                body.extend_from_slice(&1i16.to_be_bytes());
                body.extend_from_slice(&1i16.to_be_bytes());
                body.extend_from_slice(&((mb.location.x & 0x7FFF) as i16).to_be_bytes());
                body.extend_from_slice(&((mb.location.y & 0x3FFF) as i16).to_be_bytes());
                body.push(mb.location.z as u8);
                body.push(mb.direction.bits());
                body.extend_from_slice(&(mb.hue as i16).to_be_bytes());
                body.push((mb.ty.get_packet_flags)(&w, m, true) as u8);
            }
        }
        body.extend_from_slice(&0i16.to_be_bytes());
        count += 1;
    }
    let mut expected = vec![0xF7, 0, 0];
    expected.extend_from_slice(&count.to_be_bytes());
    expected.extend_from_slice(&body);
    let len = expected.len() as u16;
    expected[1..3].copy_from_slice(&len.to_be_bytes());
    // `new Item(serial)` is not Visible (only MoveBoatHS' items say so), so the deck holds the beholder alone.
    assert_eq!(count, 1);

    crate::boats::packets::send_display_boat_hs(&mut w, ns, who, b);

    assert_eq!(sent(&mut w), expected);
}
