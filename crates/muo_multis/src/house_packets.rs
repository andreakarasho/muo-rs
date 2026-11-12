//! House packets: the 0xBF house customization sub-packets and the 0xD8 design state.

use muo_data::client::multi_data::MultiComponentList;
use muo_primitives::Serial;
use muo_server::World;
use muo_server::net_state::NetStateId;

fn customization(w: &mut World, ns: NetStateId, house: Serial, command: u8) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let mut b = Vec::with_capacity(17);
    b.push(0xBF);
    b.extend_from_slice(&17u16.to_be_bytes());
    b.extend_from_slice(&0x20i16.to_be_bytes()); // Sub-packet
    b.extend_from_slice(&house.value().to_be_bytes());
    b.push(command);
    b.extend_from_slice(&0u16.to_be_bytes());
    b.extend_from_slice(&0xFFFFu16.to_be_bytes());
    b.extend_from_slice(&0xFFFFu16.to_be_bytes());
    b.push(0xFF);
    w.send(ns, &b);
}

/// `SendBeginHouseCustomization(house)`.
pub fn send_begin_house_customization(w: &mut World, ns: NetStateId, house: Serial) {
    customization(w, ns, house, 0x04);
}

/// `SendEndHouseCustomization(house)`.
pub fn send_end_house_customization(w: &mut World, ns: NetStateId, house: Serial) {
    customization(w, ns, house, 0x05);
}

/// `SendDesignStateGeneral(house, revision)`.
pub fn send_design_state_general(w: &mut World, ns: NetStateId, house: Serial, revision: i32) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let mut b = Vec::with_capacity(13);
    b.push(0xBF);
    b.extend_from_slice(&13u16.to_be_bytes());
    b.extend_from_slice(&0x1Di16.to_be_bytes()); // Sub-packet
    b.extend_from_slice(&house.value().to_be_bytes());
    b.extend_from_slice(&revision.to_be_bytes());
    w.send(ns, &b);
}

const PLANE_COUNT: usize = 9;
const MAX_PLANE_LENGTH: usize = 0x400;
const MAX_PER_PLANE_OFFSET_BUFFER: usize = 750;

/// `WritePacked(source, ref writer, out length)`: size byte, a nibble of the source size with the high nibble of the
/// packed length, then the zlib stream. The stream is flate2's, valid for the client but not libdeflate's bytes.
fn write_packed(source: &[u8], out: &mut Vec<u8>) -> usize {
    let size = source.len();
    let mut dest = vec![0u8; muo_io::compression::Level::compress_bound(size).max(16)];
    let length = muo_io::compression::STANDARD.pack(&mut dest, source);
    if length == 0 {
        tracing::warn!("House packet compression failed.");
    }
    out.push(length as u8);
    out.push((((size >> 4) & 0xF0) | ((length >> 8) & 0xF)) as u8);
    out.extend_from_slice(&dest[..length]);
    length
}

/// `CreateHouseDesignStateDetailed(serial, revision, components)`.
pub fn create_house_design_state_detailed(
    w: &World,
    serial: Serial,
    revision: i32,
    components: &MultiComponentList,
) -> Vec<u8> {
    let (x_min, y_min) = (components.min().x, components.min().y);
    let (x_max, y_max) = (components.max().x, components.max().y);
    let width = x_max - x_min + 1;
    let height = y_max - y_min + 1;
    let tiles = &components.list;

    let plane_length = (width * height * 2).max(0) as usize;
    let mut plane_offsets: Vec<u8> = Vec::new();
    let mut planes = vec![0u8; plane_length * PLANE_COUNT];
    let mut planes_used = [false; PLANE_COUNT];
    let mut total_plane_offsets = 0usize;
    let mut total_planes = 0usize;

    for mte in tiles {
        let mut x = i32::from(mte.offset_x) - x_min;
        let mut y = i32::from(mte.offset_y) - y_min;
        let z = i32::from(mte.offset_z);
        let floor = w.maps.tile_data.item(usize::from(mte.item_id)).height == 0; // `Height <= 0` on a byte

        let mut plane: i32 = match z {
            0 => 0,
            7 => 1,
            27 => 2,
            47 => 3,
            67 => 4,
            _ => -1,
        };

        if plane > -1 {
            let size;
            if plane == 0 {
                size = height;
            } else if floor {
                size = height - 2;
                x -= 1;
                y -= 1;
            } else {
                size = height - 1;
                plane += 4;
            }
            let index = (x * size + y) * 2;
            if x >= 0 && y >= 0 && y < size && ((index + 1) as usize) < MAX_PLANE_LENGTH {
                let p = plane as usize;
                let base = plane_length * p;
                if !planes_used[p] {
                    planes_used[p] = true;
                    total_planes += 1;
                    planes[base..base + plane_length].fill(0);
                }
                let at = base + index as usize;
                if at + 2 > planes.len() {
                    planes.resize(at + 2, 0); // the writer grows
                }
                planes[at..at + 2].copy_from_slice(&mte.item_id.to_be_bytes()); // SpanWriter: big endian
                continue;
            }
        }

        plane_offsets.extend_from_slice(&mte.item_id.to_be_bytes());
        plane_offsets.push(mte.offset_x as u8);
        plane_offsets.push(mte.offset_y as u8);
        plane_offsets.push(mte.offset_z as u8);
        total_plane_offsets += 1;
    }

    let mut out = Vec::with_capacity(0x1000);
    out.push(0xD8);
    out.extend_from_slice(&[0, 0]); // Length
    out.push(0x03); // Compression Type
    out.push(0x00); // Unknown
    out.extend_from_slice(&serial.value().to_be_bytes());
    out.extend_from_slice(&revision.to_be_bytes());
    out.extend_from_slice(&(tiles.len() as i16).to_be_bytes());
    out.extend_from_slice(&[0, 0, 0]); // Buffer Length, Plane Count

    let mut total_length: usize = 1; // includes plane count

    for (i, &used) in planes_used.iter().enumerate().take(PLANE_COUNT) {
        if !used {
            continue;
        }
        let size = match i {
            0 => plane_length,
            1..=4 => ((width - 1) * (height - 2) * 2).max(0) as usize,
            _ => (width * (height - 1) * 2).max(0) as usize,
        };
        let base = plane_length * i;
        let end = (base + size).min(planes.len());
        out.push((0x20 | i) as u8);
        out.push(size as u8);
        let dest_length = write_packed(&planes[base..end], &mut out);
        total_length += 4 + dest_length;
    }

    let mut index = 0usize;
    let mut remaining = total_plane_offsets;
    while remaining > 0 {
        let count = MAX_PER_PLANE_OFFSET_BUFFER.min(remaining);
        remaining -= count;
        let size = count * 5;
        out.push((9 + index) as u8);
        out.push(size as u8);
        // Slices at `index * 5` of the (larger, zero filled) buffer, not at the chunk's start; kept, clients expect it.
        let mut chunk = vec![0u8; size];
        let start = (index * 5).min(plane_offsets.len());
        let end = (index * 5 + size).min(plane_offsets.len());
        chunk[..end - start].copy_from_slice(&plane_offsets[start..end]);
        let dest_length = write_packed(&chunk, &mut out);
        index += 1;
        total_length += 4 + dest_length;
        total_planes += 1;
    }

    out[15..17].copy_from_slice(&(total_length as i16).to_be_bytes());
    out[17] = total_planes as u8;
    let len = out.len() as u16;
    out[1..3].copy_from_slice(&len.to_be_bytes());
    out
}
