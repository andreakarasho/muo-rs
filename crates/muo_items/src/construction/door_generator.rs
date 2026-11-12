//! The `DoorGen` command (Developer) that finds door frames in the map's
//! statics and places `DarkWoodDoor`s in them. The frame tables and regions are fixed data.

use muo_primitives::{Point3D, Rectangle2D};
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::ItemId;
use muo_server::item::ops;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::type_registry::{Phase, TypeRegistry};

use super::doors::{DARK_WOOD_DOOR, facing_door, set_link};
use muo_content::items::house_deps::DoorFacing;

static SOUTH_FRAMES: &[i32] = &[
    0x0006, 0x0008, 0x000B, 0x001A, 0x001B, 0x001F, 0x0038, 0x0057, 0x0059, 0x005B, 0x005D, 0x0080,
    0x0081, 0x0082, 0x0084, 0x0090, 0x0091, 0x0094, 0x0096, 0x0099, 0x00A6, 0x00A7, 0x00AA, 0x00AE,
    0x00B0, 0x00B3, 0x00C7, 0x00C9, 0x00F8, 0x00FA, 0x00FD, 0x00FE, 0x0100, 0x0103, 0x0104, 0x0106,
    0x0109, 0x0127, 0x0129, 0x012B, 0x012D, 0x012F, 0x0131, 0x0132, 0x0134, 0x0135, 0x0137, 0x0139,
    0x013B, 0x014C, 0x014E, 0x014F, 0x0151, 0x0153, 0x0155, 0x0157, 0x0158, 0x015A, 0x015D, 0x015E,
    0x015F, 0x0162, 0x01CF, 0x01D1, 0x01D4, 0x01FF, 0x0204, 0x0206, 0x0208, 0x020A,
];

static NORTH_FRAMES: &[i32] = &[
    0x0006, 0x0008, 0x000D, 0x001A, 0x001B, 0x0020, 0x003A, 0x0057, 0x0059, 0x005B, 0x005D, 0x0080,
    0x0081, 0x0082, 0x0084, 0x0090, 0x0091, 0x0094, 0x0096, 0x0099, 0x00A6, 0x00A7, 0x00AC, 0x00AE,
    0x00B0, 0x00C7, 0x00C9, 0x00F8, 0x00FA, 0x00FD, 0x00FE, 0x0100, 0x0103, 0x0104, 0x0106, 0x0109,
    0x0127, 0x0129, 0x012B, 0x012D, 0x012F, 0x0131, 0x0132, 0x0134, 0x0135, 0x0137, 0x0139, 0x013B,
    0x014C, 0x014E, 0x014F, 0x0151, 0x0153, 0x0155, 0x0157, 0x0158, 0x015A, 0x015D, 0x015E, 0x015F,
    0x0162, 0x01CF, 0x01D1, 0x01D4, 0x01FF, 0x0201, 0x0204, 0x0208, 0x020A,
];

static EAST_FRAMES: &[i32] = &[
    0x0007, 0x000A, 0x001A, 0x001C, 0x001E, 0x0037, 0x0058, 0x0059, 0x005C, 0x005E, 0x0080, 0x0081,
    0x0082, 0x0084, 0x0090, 0x0092, 0x0095, 0x0097, 0x0098, 0x00A6, 0x00A8, 0x00AB, 0x00AE, 0x00AF,
    0x00B2, 0x00C7, 0x00C8, 0x00EA, 0x00F8, 0x00F9, 0x00FC, 0x00FE, 0x00FF, 0x0102, 0x0104, 0x0105,
    0x0108, 0x0127, 0x0128, 0x012B, 0x012C, 0x012E, 0x0130, 0x0132, 0x0133, 0x0135, 0x0136, 0x0138,
    0x013A, 0x014C, 0x014D, 0x014F, 0x0150, 0x0152, 0x0154, 0x0156, 0x0158, 0x0159, 0x015C, 0x015E,
    0x0160, 0x0163, 0x01CF, 0x01D0, 0x01D3, 0x01FF, 0x0203, 0x0205, 0x0207, 0x0209,
];

static WEST_FRAMES: &[i32] = &[
    0x0007, 0x000C, 0x001A, 0x001C, 0x0021, 0x0039, 0x0058, 0x0059, 0x005C, 0x005E, 0x0080, 0x0081,
    0x0082, 0x0084, 0x0090, 0x0092, 0x0095, 0x0097, 0x0098, 0x00A6, 0x00A8, 0x00AD, 0x00AE, 0x00AF,
    0x00B5, 0x00C7, 0x00C8, 0x00EA, 0x00F8, 0x00F9, 0x00FC, 0x00FE, 0x00FF, 0x0102, 0x0104, 0x0105,
    0x0108, 0x0127, 0x0128, 0x012C, 0x012E, 0x0130, 0x0132, 0x0133, 0x0135, 0x0136, 0x0138, 0x013A,
    0x014C, 0x014D, 0x014F, 0x0150, 0x0152, 0x0154, 0x0156, 0x0158, 0x0159, 0x015C, 0x015E, 0x0160,
    0x0163, 0x01CF, 0x01D0, 0x01D3, 0x01FF, 0x0200, 0x0203, 0x0207, 0x0209,
];

/// `_britRegions` as (x, y, width, height).
static BRIT_REGIONS: &[(i32, i32, i32, i32)] = &[
    (250, 750, 525, 580),
    (525, 2095, 400, 335),
    (1025, 2155, 240, 155),
    (1635, 2430, 70, 78),
    (1775, 2605, 390, 370),
    (1055, 3520, 515, 555),
    (2860, 3310, 260, 320),
    (2470, 1855, 1480, 1190),
    (3425, 990, 475, 465),
    (4175, 735, 665, 865),
    (2375, 330, 725, 715),
    (2100, 1090, 210, 360),
    (1495, 1400, 55, 75),
    (1085, 1520, 330, 390),
    (1410, 1500, 335, 295),
    (5120, 2300, 1023, 1795),
];

/// `_ilshRegions` as (x, y, width, height).
static ILSH_REGIONS: &[(i32, i32, i32, i32)] = &[(0, 0, 2304, 1600)];

/// `_malasRegions` as (x, y, width, height).
static MALAS_REGIONS: &[(i32, i32, i32, i32)] = &[(0, 0, 2560, 2048)];

/// `DoorGenerator.IsFrame(id, list)`.
pub fn is_frame(id: i32, list: &[i32]) -> bool {
    if id > list[list.len() - 1] {
        return false;
    }
    for &f in list {
        let delta = id - f;
        if delta < 0 {
            return false;
        }
        if delta == 0 {
            return true;
        }
    }
    false
}

pub fn is_north_frame(id: i32) -> bool {
    is_frame(id, NORTH_FRAMES)
}

pub fn is_south_frame(id: i32) -> bool {
    is_frame(id, SOUTH_FRAMES)
}

pub fn is_west_frame(id: i32) -> bool {
    is_frame(id, WEST_FRAMES)
}

pub fn is_east_frame(id: i32) -> bool {
    is_frame(id, EAST_FRAMES)
}

/// The map being generated and the number of doors made in it (`m_Map`, `m_Count`).
struct Run {
    map: MapId,
    count: i32,
}

/// `IsEastFrame(x, y, z, out newZ)` / `IsSouthFrame(x, y, z, out newZ)`.
fn frame_at(w: &mut World, run: &Run, x: i32, y: i32, z: i32, east: bool) -> Option<i32> {
    for tile in muo_server::map::tiles::static_tiles(w, run.map, x, y) {
        let ok = if east {
            is_east_frame(i32::from(tile.id))
        } else {
            is_south_frame(i32::from(tile.id))
        };
        if !ok {
            continue;
        }
        let delta = i32::from(tile.z) - z;
        if (-1..=1).contains(&delta) {
            return Some(i32::from(tile.z));
        }
    }
    None
}

/// `AddDoor(x, y, z, newZ, facing)`.
fn add_door(
    w: &mut World,
    run: &mut Run,
    x: i32,
    y: i32,
    z: i32,
    new_z: i32,
    facing: DoorFacing,
) -> Option<ItemId> {
    let min_z = z.min(new_z);
    if !muo_server::map::fit::can_fit(w, run.map, x, y, min_z, 16, false, false, true) {
        return None;
    }
    if y == 1743 && (1343..=1344).contains(&x) {
        return None;
    }
    if y == 1679 && (1392..=1393).contains(&x) {
        return None;
    }
    if x == 1320 && (1618..=1640).contains(&y) {
        return None;
    }
    if x == 1383 && (1642..=1643).contains(&y) {
        return None;
    }
    // Ilshenar ruins
    if run.map == MapId::ILSHENAR && (644..=670).contains(&x) && (925..=941).contains(&y)
        || x == 985 && y == 994
    {
        return None;
    }
    if z != new_z {
        tracing::warn!(
            "Door at {x}, {y}, {z} ({:?}) has mismatched Z levels: {z} != {new_z}",
            run.map
        );
    }
    let door = facing_door(w, &DARK_WOOD_DOOR, facing, 0x6A5, 0xEA, 0xF1);
    ops::move_to_world_map(w, door, Point3D::new(x, y, min_z), Some(run.map));
    run.count += 1;
    Some(door)
}

/// Two doors side by side that open together, or neither.
fn add_pair(
    w: &mut World,
    run: &mut Run,
    a: (i32, i32, DoorFacing),
    b: (i32, i32, DoorFacing),
    z: i32,
    new_z: i32,
) {
    let first = add_door(w, run, a.0, a.1, z, new_z, a.2);
    let second = add_door(w, run, b.0, b.1, z, new_z, b.2);
    if let (Some(f), Some(s)) = (first, second) {
        set_link(w, f, Some(s));
        set_link(w, s, Some(f));
    } else {
        for d in [first, second].into_iter().flatten() {
            if let Some(ty) = w.item(d).map(|i| i.ty) {
                (ty.delete)(w, d);
            }
        }
    }
}

/// `Generate(region)`.
fn generate_region(w: &mut World, run: &mut Run, region: Rectangle2D) {
    let (rx0, ry0) = (region.start.x, region.start.y);
    for rx in 0..region.width() {
        for ry in 0..region.height() {
            let (vx, vy) = (rx + rx0, ry + ry0);
            for tile in muo_server::map::tiles::static_tiles(w, run.map, vx, vy) {
                let (id, z) = (i32::from(tile.id), i32::from(tile.z));
                if is_west_frame(id) {
                    if let Some(nz) = frame_at(w, run, vx + 2, vy, z, true) {
                        add_door(w, run, vx + 1, vy, z, nz, DoorFacing::WestCW);
                    } else if let Some(nz) = frame_at(w, run, vx + 3, vy, z, true) {
                        add_pair(
                            w,
                            run,
                            (vx + 1, vy, DoorFacing::WestCW),
                            (vx + 2, vy, DoorFacing::EastCCW),
                            z,
                            nz,
                        );
                    }
                } else if is_north_frame(id) {
                    if let Some(nz) = frame_at(w, run, vx, vy + 2, z, false) {
                        add_door(w, run, vx, vy + 1, z, nz, DoorFacing::SouthCW);
                    } else if let Some(nz) = frame_at(w, run, vx, vy + 3, z, false) {
                        add_pair(
                            w,
                            run,
                            (vx, vy + 1, DoorFacing::NorthCCW),
                            (vx, vy + 2, DoorFacing::SouthCW),
                            z,
                            nz,
                        );
                    }
                }
            }
        }
    }
}

fn rect(r: &(i32, i32, i32, i32)) -> Rectangle2D {
    Rectangle2D::new(r.0, r.1, r.2, r.3)
}

/// `DoorGenerator.Generate()`.
pub fn generate(w: &mut World) {
    w.broadcast(0x35, true, "Generating doors, please wait.");
    let mut counts = [0; 4];
    for (i, (map, regions)) in [
        (MapId::TRAMMEL, BRIT_REGIONS),
        (MapId::FELUCCA, BRIT_REGIONS),
        (MapId::ILSHENAR, ILSH_REGIONS),
        (MapId::MALAS, MALAS_REGIONS),
    ]
    .into_iter()
    .enumerate()
    {
        let mut run = Run { map, count: 0 };
        for r in regions {
            generate_region(w, &mut run, rect(r));
        }
        counts[i] = run.count;
    }
    w.broadcast(
        0x35,
        true,
        &format!(
            "Door generation complete. Trammel: {}; Felucca: {}; Ilshenar: {}; Malas: {};",
            counts[0], counts[1], counts[2], counts[3]
        ),
    );
}

fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "DoorGen",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(|w: &mut World, _e: &mut CommandEventArgs| {
            // Usage: DoorGen; Description: Generates doors by analyzing the map. Slow.
            generate(w);
        }),
    );
}

/// Registers the command.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        Phase::Configure,
        "Server.DoorGenerator.Configure",
        configure,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_lookup_stops_at_the_first_larger_entry() {
        assert!(is_south_frame(0x0006));
        assert!(!is_south_frame(0x0007));
        assert!(is_west_frame(0x0007));
        assert!(!is_north_frame(0x0300), "past the last entry");
        assert!(is_east_frame(0x0209));
    }
}
