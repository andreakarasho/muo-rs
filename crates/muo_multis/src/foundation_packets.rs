//! The client side of `HouseFoundation`: the encoded (0xD7) design commands, the extended 0x1E query, the commit
//! confirmation gump.

use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::{DynamicGumpLogic, DynamicGumpObj, RelayInfo, send_gump_to_mobile};
use muo_content::network::extended_commands::register_extended;
use muo_content::network::register_encoded;
use muo_content::vendor::banker;
use muo_data::tile_data::TileData;
use muo_net::net_state::NetState;
use muo_primitives::{Point3D, Serial};
use muo_server::World;
use muo_server::encoded_reader::EncodedReader;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;

use crate::component_verification::{is_item_valid, is_multi_valid};
use crate::foundation::*;
use crate::house;

/// `ValidPiece(itemID, roof)`.
pub fn valid_piece(w: &mut World, item_id: i32, roof: bool) -> bool {
    let id = item_id & TileData::MAX_ITEM_VALUE as i32;
    roof == w.maps.tile_data.item(id as usize).roof() && is_item_valid(w, id)
}

/// `TraceValidity(state, itemID)`.
fn trace_validity(w: &World, ns: NetStateId, m: MobileId, item_id: i32) {
    use std::io::Write;
    // `{0}	{1}`: NetState.ToString() (the address) and Mobile.ToString() (`{Serial} "{Name}"`)
    let state = w
        .net_states
        .get(&ns)
        .map(|n| n.description.clone())
        .unwrap_or_default();
    let mobile = w
        .mobile(m)
        .map(|x| format!("{} \"{}\"", x.serial, (x.ty.name)(w, m).unwrap_or_default()))
        .unwrap_or_default();
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("comp_val.log")
    {
        let _ = writeln!(f, "{state}	{mobile}	Invalid ItemID 0x{item_id:04X}");
    }
}

/// What every designer handler starts with: the connection, its mobile and that mobile's context.
fn begin(w: &mut World, ns: &NetState) -> Option<(NetStateId, MobileId, DesignContext)> {
    let id = muo_server::net_glue::ensure_net_state(w, ns);
    let from = w.net_state(id).and_then(|d| d.mobile)?;
    let ctx = find_context(w, from)?;
    Some((id, from, ctx))
}

fn send_info(w: &mut World, house: ItemId, ns: NetStateId) {
    if let Some(ty) = w.item(house).map(|i| i.ty) {
        (ty.send_info_to)(w, house, ns, &[]);
    }
}

fn designer_sync(w: &mut World, ns: &mut NetState, _e: u32, _b: &[u8]) {
    let Some((id, _, ctx)) = begin(w, ns) else {
        return;
    };
    send_detailed(w, ctx.foundation, id, true); // Resend full house state
}

fn designer_clear(w: &mut World, ns: &mut NetState, _e: u32, _b: &[u8]) {
    let Some((id, _, ctx)) = begin(w, ns) else {
        return;
    };
    let f = ctx.foundation;
    // Restore empty foundation, update revision, update the client
    let empty = get_empty_foundation(w, f);
    edit(w, f, |d, _| {
        let mut s = DesignState::new(empty);
        d.revise(&mut s);
        d.design = Some(s);
    });
    send_info(w, f, id);
    send_detailed(w, f, id, true);
}

fn designer_restore(w: &mut World, ns: &mut NetState, _e: u32, _b: &[u8]) {
    let Some((id, _, ctx)) = begin(w, ns) else {
        return;
    };
    let f = ctx.foundation;
    // Restore backup: a new design from the backup state
    edit(w, f, |d, _| {
        let mut s = DesignState::copy_of(d.backup.as_ref().unwrap_or(&DesignState::default()));
        d.revise(&mut s);
        d.design = Some(s);
    });
    send_info(w, f, id);
    send_detailed(w, f, id, true);
}

fn designer_backup(w: &mut World, ns: &mut NetState, _e: u32, _b: &[u8]) {
    let Some((_, _, ctx)) = begin(w, ns) else {
        return;
    };
    edit(w, ctx.foundation, |d, _| {
        d.backup = Some(DesignState::copy_of(
            d.design.as_ref().unwrap_or(&DesignState::default()),
        ))
    });
}

fn designer_revert(w: &mut World, ns: &mut NetState, _e: u32, _b: &[u8]) {
    let Some((id, _, ctx)) = begin(w, ns) else {
        return;
    };
    let f = ctx.foundation;
    // Revert design state to the currently visible state, fixtures frozen into it
    edit(w, f, |d, td| {
        let mut s = DesignState::copy_of(d.current.as_ref().unwrap_or(&DesignState::default()));
        freeze_fixtures(&mut s, &mut d.last_revision, td);
        d.revise(&mut s);
        d.design = Some(s);
    });
    check_signpost(w, f);
    send_info(w, f, id);
    send_detailed(w, f, id, true);
}

fn designer_commit(w: &mut World, ns: &mut NetState, _e: u32, _b: &[u8]) {
    let Some((_, from, ctx)) = begin(w, ns) else {
        return;
    };
    let f = ctx.foundation;
    let old_price = house::d(w, f).price;
    let dsn = design_state(w, f);
    let cur = current_state(w, f);
    let fixtures = fd(w, f).map_or(0, |x| x.fixtures.len() as i32);
    let new_price = old_price
        + customization_cost(w)
        + (dsn.components.list.len() as i32 - (cur.components.list.len() as i32 + fixtures)) * 500;
    let bank_balance = banker::get_balance(w, from);
    let gump = ConfirmCommitGump {
        foundation: f,
        bank_balance,
        old_price,
        new_price,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(50, 50, gump)), false);
}

fn designer_delete(w: &mut World, ns: &mut NetState, _e: u32, body: &[u8]) {
    let Some((id, _, ctx)) = begin(w, ns) else {
        return;
    };
    let f = ctx.foundation;
    let mut r = EncodedReader::new(body);
    let (Ok(item_id), Ok(x), Ok(y), Ok(z)) =
        (r.read_i32(), r.read_i32(), r.read_i32(), r.read_i32())
    else {
        return;
    };
    let resend = edit(w, f, |d, td| {
        let mut design = d.design.take().unwrap_or_default();
        let mcl = &mut design.components;
        let (ax, ay) = (x + mcl.center().x, y + mcl.center().y);
        let mut result = None; // Some(fix_state)
        if z == 0 && ax >= 0 && ax < mcl.width() && ay >= 0 && ay < mcl.height() - 1 {
            // Component is not deletable: resend the design state
            d.design = Some(design);
            return true;
        }
        let mut fix_state = false;
        if ALLOW_STAIR_SECTIONING {
            if delete_stairs(mcl, item_id, x, y, z, td) {
                fix_state = true; // The client removes the entire set of stairs locally, resend state
            }
            mcl.remove(item_id, x, y, z);
        } else if !delete_stairs(mcl, item_id, x, y, z, td) {
            mcl.remove(item_id, x, y, z);
        }
        // If needed, replace removed component with a dirt tile
        if ax >= 1 && ax < mcl.width() && ay >= 1 && ay < mcl.height() - 1 {
            let has_base_floor = mcl.tiles[ax as usize][ay as usize]
                .iter()
                .any(|t| t.z == 7 && t.id != 1);
            if !has_base_floor {
                mcl.add(0x31F4, x, y, 7, td);
            }
        }
        d.revise(&mut design);
        d.design = Some(design);
        result.get_or_insert(fix_state);
        fix_state
    });
    if resend == Some(true) {
        send_detailed(w, f, id, true);
    }
}

fn designer_stairs(w: &mut World, ns: &mut NetState, _e: u32, body: &[u8]) {
    let Some((id, _, ctx)) = begin(w, ns) else {
        return;
    };
    let f = ctx.foundation;
    let mut r = EncodedReader::new(body);
    let (Ok(item_id), Ok(x), Ok(y)) = (r.read_i32(), r.read_i32(), r.read_i32()) else {
        return;
    };
    if !is_multi_valid(w, item_id) {
        // Specified multi ID is not a stair: resend the design state
        let from = ctx_mobile(w, id);
        if let Some(m) = from {
            trace_validity(w, id, m, item_id);
        }
        send_detailed(w, f, id, true);
        return;
    }
    let stairs = w.maps.multi_data.get_components(item_id).clone();
    let z = get_level_z(w, ctx.level, f);
    edit(w, f, |d, td| {
        let mut design = d.design.take().unwrap_or_default();
        for e in &stairs.list {
            if e.item_id != 1 {
                design.components.add(
                    i32::from(e.item_id),
                    x + i32::from(e.offset_x),
                    y + i32::from(e.offset_y),
                    z + i32::from(e.offset_z),
                    td,
                );
            }
        }
        d.revise(&mut design);
        d.design = Some(design);
    });
}

fn ctx_mobile(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_state(ns).and_then(|d| d.mobile)
}

fn designer_build(w: &mut World, ns: &mut NetState, _e: u32, body: &[u8]) {
    let Some((id, from, ctx)) = begin(w, ns) else {
        return;
    };
    let f = ctx.foundation;
    let mut r = EncodedReader::new(body);
    let (Ok(item_id), Ok(x), Ok(y)) = (r.read_i32(), r.read_i32(), r.read_i32()) else {
        return;
    };
    let gm = w
        .mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    if !gm && !valid_piece(w, item_id, false) {
        trace_validity(w, id, from, item_id);
        send_detailed(w, f, id, true);
        return;
    }
    let mut z = get_level_z(w, ctx.level, f);
    edit(w, f, |d, td| {
        let mut design = d.design.take().unwrap_or_default();
        if y + design.components.center().y == design.components.height() - 1 {
            z = 0; // Tiles placed on the far-south of the house are at 0 Z
        }
        design.components.add(item_id, x, y, z, td);
        d.revise(&mut design);
        d.design = Some(design);
    });
}

fn designer_close(w: &mut World, ns: &mut NetState, _e: u32, _b: &[u8]) {
    let Some((id, from, ctx)) = begin(w, ns) else {
        return;
    };
    let f = ctx.foundation;
    // Remove design context, tell the client customization ended, refresh the visible state
    remove_context(w, from);
    crate::house_packets::send_end_house_customization(w, id, serial_of(w, f));
    send_info(w, f, id);
    send_detailed(w, f, id, false);
    check_signpost(w, f);
    eject_all(w, f, from);
}

fn designer_level(w: &mut World, ns: &mut NetState, _e: u32, body: &[u8]) {
    let Some((id, from, ctx)) = begin(w, ns) else {
        return;
    };
    let f = ctx.foundation;
    let Ok(mut level) = EncodedReader::new(body).read_i32() else {
        return;
    };
    if level < 1 || level > max_levels(w, f) {
        level = 1;
    }
    if let Some(c) = w.resources.get_mut::<DesignContexts>().0.get_mut(&from) {
        c.level = level;
    }
    let (p, fz) = (
        w.mobile(from).map_or(Point3D::ZERO, |m| m.location),
        house::location(w, f).z,
    );
    let z = fz + get_level_z(w, level, f);
    muo_server::mobile::part1::lifecycle::set_location(w, from, Point3D::new(p.x, p.y, z));
    send_info(w, f, id);
}

fn designer_roof(w: &mut World, ns: &mut NetState, _e: u32, body: &[u8]) {
    let Some((id, from, ctx)) = begin(w, ns) else {
        return;
    };
    let f = ctx.foundation;
    let gm = w
        .mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
    if !muo_content::core::se(w) && !gm {
        return;
    }
    let mut r = EncodedReader::new(body);
    let (Ok(item_id), Ok(x), Ok(y), Ok(mut z)) =
        (r.read_i32(), r.read_i32(), r.read_i32(), r.read_i32())
    else {
        return;
    };
    if !gm && !valid_piece(w, item_id, true) {
        trace_validity(w, id, from, item_id);
        send_detailed(w, f, id, true);
        return;
    }
    if !(-3..=12).contains(&z) || z % 3 != 0 {
        z = -3;
    }
    z += get_level_z(w, ctx.level, f);
    let levels: Vec<(i32, i32)> = {
        let d = design_state(w, f);
        d.components
            .list
            .iter()
            .map(|m| (i32::from(m.offset_z), i32::from(m.item_id)))
            .collect()
    };
    let level_of: Vec<i32> = levels
        .iter()
        .map(|(zz, _)| get_z_level(w, *zz, f))
        .collect();
    let _ = levels;
    edit(w, f, |d, td| {
        let mut design = d.design.take().unwrap_or_default();
        let list = design.components.list.clone();
        for (i, mte) in list.iter().enumerate() {
            if i32::from(mte.offset_x) == x
                && i32::from(mte.offset_y) == y
                && level_of.get(i).copied() == Some(ctx.level)
                && td.item(usize::from(mte.item_id)).roof()
            {
                design
                    .components
                    .remove(i32::from(mte.item_id), x, y, i32::from(mte.offset_z));
            }
        }
        design.components.add(item_id, x, y, z, td);
        d.revise(&mut design);
        d.design = Some(design);
    });
}

fn designer_roof_delete(w: &mut World, ns: &mut NetState, _e: u32, body: &[u8]) {
    let Some((id, _, ctx)) = begin(w, ns) else {
        return;
    };
    let f = ctx.foundation;
    let mut r = EncodedReader::new(body);
    let (Ok(item_id), Ok(x), Ok(y), Ok(z)) =
        (r.read_i32(), r.read_i32(), r.read_i32(), r.read_i32())
    else {
        return;
    };
    if !w
        .maps
        .tile_data
        .item((item_id & TileData::MAX_ITEM_VALUE as i32) as usize)
        .roof()
    {
        send_detailed(w, f, id, true);
        return;
    }
    edit(w, f, |d, _| {
        let mut design = d.design.take().unwrap_or_default();
        design.components.remove(item_id, x, y, z);
        d.revise(&mut design);
        d.design = Some(design);
    });
}

/// `QueryDesignDetails` (extended 0x1E).
fn query_design_details(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let id = muo_server::net_glue::ensure_net_state(w, ns);
    let Some(from) = w.net_state(id).and_then(|d| d.mobile) else {
        return;
    };
    let Ok(raw) = muo_io::SpanReader::new(body).read_u32() else {
        return;
    };
    let Some(f) = w
        .find_item(Serial::new(raw))
        .filter(|&i| is_foundation(w, i))
    else {
        return;
    };
    let wl = muo_server::item::ops::get_world_location(w, f);
    if w.mobile(from).and_then(|m| m.map) == house::map_of(w, f)
        && muo_server::mobile::base::in_range_point3d(w, from, wl, 24)
        && muo_server::mobile::base::can_see_item(w, from, f)
    {
        let designing = find_context(w, from).is_some_and(|c| c.foundation == f);
        send_detailed(w, f, id, designing);
    }
}

/// `HouseFoundation.Initialize`: the packet registrations.
pub fn configure(w: &mut World) {
    register_extended(w, 0x1E, true, query_design_details);
    register_encoded(w, 0x02, true, designer_backup);
    register_encoded(w, 0x03, true, designer_restore);
    register_encoded(w, 0x04, true, designer_commit);
    register_encoded(w, 0x05, true, designer_delete);
    register_encoded(w, 0x06, true, designer_build);
    register_encoded(w, 0x0C, true, designer_close);
    register_encoded(w, 0x0D, true, designer_stairs);
    register_encoded(w, 0x0E, true, designer_sync);
    register_encoded(w, 0x10, true, designer_clear);
    register_encoded(w, 0x12, true, designer_level);
    register_encoded(w, 0x13, true, designer_roof); // Samurai Empire roof
    register_encoded(w, 0x14, true, designer_roof_delete); // Samurai Empire roof
    register_encoded(w, 0x1A, true, designer_revert);
}

// ---- ConfirmCommitGump ----

struct ConfirmCommitGump {
    foundation: ItemId,
    bank_balance: i32,
    old_price: i32,
    new_price: i32,
}

impl DynamicGumpLogic for ConfirmCommitGump {
    const TYPE_NAME: &'static str = "Server.Multis.ConfirmCommitGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let (bank, old, new) = (self.bank_balance, self.old_price, self.new_price);
        b.add_page(0);
        b.add_background(0, 0, 320, 320, 5054);
        b.add_image_tiled(10, 10, 300, 20, 2624);
        b.add_image_tiled(10, 40, 300, 240, 2624);
        b.add_image_tiled(10, 290, 300, 20, 2624);
        b.add_alpha_region(10, 10, 300, 300);
        b.add_html_localized_color(10, 10, 300, 20, 1062060, 32736, false, false); // <CENTER>COMMIT DESIGN</CENTER>
        b.add_html_localized_color(
            10,
            40,
            300,
            140,
            if new - old <= bank { 1061898 } else { 1061903 },
            1023,
            false,
            true,
        );
        b.add_html_localized_color(10, 190, 150, 20, 1061902, 32736, false, false); // Bank Balance:
        b.add_label(170, 190, 55, &bank.to_string());
        b.add_html_localized_color(10, 215, 150, 20, 1061899, 1023, false, false); // Old Value:
        b.add_label(170, 215, 90, &old.to_string());
        b.add_html_localized_color(10, 235, 150, 20, 1061900, 1023, false, false); // Cost To Commit:
        b.add_label(170, 235, 90, &new.to_string());
        if new - old < 0 {
            b.add_html_localized_color(10, 260, 150, 20, 1062059, 992, false, false); // Your Refund:
            b.add_label(170, 260, 70, &(old - new).to_string());
        } else {
            b.add_html_localized_color(10, 260, 150, 20, 1061901, 31744, false, false); // Your Cost:
            b.add_label(170, 260, 40, &(new - old).to_string());
        }
        b.add_button(10, 290, 4005, 4007, 1);
        b.add_html_localized_color(45, 290, 55, 20, 1011036, 32767, false, false); // OKAY
        b.add_button(170, 290, 4005, 4007, 0);
        b.add_html_localized_color(195, 290, 55, 20, 1011012, 32767, false, false); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 1
            && let Some(m) = w.net_mobile(sender)
        {
            end_confirm_commit(w, self.foundation, m);
        }
    }
}
