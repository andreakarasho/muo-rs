//! The bulk order deed gumps: `SmallBODGump`, `SmallBODAcceptGump`, `LargeBODGump`, `LargeBODAcceptGump`.
//!
//! A layout has no world, so a gump holds the deed's numbers as they were when it was sent (`View`).

use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, RelayInfo, send_gump_to_mobile,
};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{deps, ops};
use muo_server::net_state::NetStateId;

use super::{begin_combine, data, in_pack, is_large, mat, material_number};

/// What the layouts read of a deed.
#[derive(Clone, Debug, Default)]
struct View {
    amount_max: i32,
    amount_cur: i32,
    number: i32,
    graphic: i32,
    require_exceptional: bool,
    material: i32,
    /// `(Details.Number, Amount)` of a large deed's entries.
    entries: Vec<(i32, i32)>,
}

fn view(w: &World, deed: ItemId) -> View {
    let d = data(w, deed).cloned().unwrap_or_default();
    View {
        amount_max: d.amount_max,
        amount_cur: d.amount_cur,
        number: d.number,
        graphic: d.graphic,
        require_exceptional: d.require_exceptional,
        material: d.material,
        entries: d
            .entries
            .iter()
            .map(|e| (e.details.number, e.amount))
            .collect(),
    }
}

fn mobile_of(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_states.get(&ns).and_then(|n| n.mobile)
}

fn hl(
    b: &mut DynamicGumpBuilder,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    number: i32,
    color: i32,
) {
    b.add_html_localized_color(x, y, width, height, number, color, false, false);
}

const WHITE: i32 = 0x7FFF;

/// `SmallBODGump(deed)` / `LargeBODGump(deed)` for the deed's kind.
pub fn send_bod_gump(w: &mut World, to: MobileId, deed: ItemId) {
    let v = view(w, deed);
    if is_large(w, deed) {
        send_gump_to_mobile(
            w,
            to,
            Box::new(DynamicGumpObj::new(25, 25, LargeBodGump { deed, v })),
            false,
        );
    } else {
        send_gump_to_mobile(
            w,
            to,
            Box::new(DynamicGumpObj::new(25, 25, SmallBodGump { deed, v })),
            false,
        );
    }
}

/// `new SmallBODAcceptGump(bod)` / `new LargeBODAcceptGump(bod)`.
pub fn send_accept_gump(w: &mut World, to: MobileId, deed: ItemId) {
    let v = view(w, deed);
    if is_large(w, deed) {
        send_gump_to_mobile(
            w,
            to,
            Box::new(DynamicGumpObj::new(50, 50, LargeAcceptGump { deed, v })),
            false,
        );
    } else {
        send_gump_to_mobile(
            w,
            to,
            Box::new(DynamicGumpObj::new(50, 50, SmallAcceptGump { deed, v })),
            false,
        );
    }
}

// ---- SmallBODGump ----

struct SmallBodGump {
    deed: ItemId,
    v: View,
}

impl DynamicGumpLogic for SmallBodGump {
    const TYPE_NAME: &'static str = "Server.Engines.BulkOrders.SmallBODGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let v = &self.v;
        b.add_page(0);
        b.add_background(50, 10, 455, 260, 5054);
        b.add_image_tiled(58, 20, 438, 241, 2624);
        b.add_alpha_region(58, 20, 438, 241);
        b.add_image(45, 5, 10460, 0);
        b.add_image(480, 5, 10460, 0);
        b.add_image(45, 245, 10460, 0);
        b.add_image(480, 245, 10460, 0);

        hl(b, 225, 25, 120, 20, 1045133, WHITE); // A bulk order
        hl(b, 75, 48, 250, 20, 1045138, WHITE); // Amount to make:
        b.add_label(275, 48, 1152, &v.amount_max.to_string());
        hl(b, 275, 76, 200, 20, 1045153, WHITE); // Amount finished:
        hl(b, 75, 72, 120, 20, 1045136, WHITE); // Item requested:
        b.add_item(410, 72, v.graphic, 0);
        hl(b, 75, 96, 210, 20, v.number, WHITE);
        b.add_label(275, 96, 0x480, &v.amount_cur.to_string());

        if v.require_exceptional || v.material != mat::NONE {
            hl(b, 75, 120, 200, 20, 1045140, WHITE); // Special requirements to meet:
        }
        if v.require_exceptional {
            hl(b, 75, 144, 300, 20, 1045141, WHITE); // All items must be exceptional.
        }
        if v.material != mat::NONE {
            hl(
                b,
                75,
                if v.require_exceptional { 168 } else { 144 },
                300,
                20,
                material_number(v.material),
                WHITE,
            );
        }

        b.add_button(125, 192, 4005, 4007, 2);
        hl(b, 160, 192, 300, 20, 1045154, WHITE); // Combine this deed with the item requested.
        b.add_button(125, 216, 4005, 4007, 1);
        hl(b, 160, 216, 120, 20, 1011441, WHITE); // EXIT
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if ops::is_deleted(w, self.deed) || !in_pack(w, self.deed, from) {
            return;
        }
        if info.button_id == 2 {
            // Combine
            send_bod_gump(w, from, self.deed);
            begin_combine(w, self.deed, from);
        }
    }
}

// ---- LargeBODGump ----

struct LargeBodGump {
    deed: ItemId,
    v: View,
}

impl DynamicGumpLogic for LargeBodGump {
    const TYPE_NAME: &'static str = "Server.Engines.BulkOrders.LargeBODGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let v = &self.v;
        let n = v.entries.len() as i32;
        b.add_page(0);
        b.add_background(50, 10, 455, 236 + n * 24, 5054);
        b.add_image_tiled(58, 20, 438, 217 + n * 24, 2624);
        b.add_alpha_region(58, 20, 438, 217 + n * 24);
        b.add_image(45, 5, 10460, 0);
        b.add_image(480, 5, 10460, 0);
        b.add_image(45, 221 + n * 24, 10460, 0);
        b.add_image(480, 221 + n * 24, 10460, 0);

        hl(b, 225, 25, 120, 20, 1045134, WHITE); // A large bulk order
        hl(b, 75, 48, 250, 20, 1045138, WHITE); // Amount to make:
        b.add_label(275, 48, 1152, &v.amount_max.to_string());
        hl(b, 75, 72, 120, 20, 1045137, WHITE); // Items requested:
        hl(b, 275, 76, 200, 20, 1045153, WHITE); // Amount finished:

        let mut y = 96;
        for (number, amount) in &v.entries {
            hl(b, 75, y, 210, 20, *number, WHITE);
            b.add_label(275, y, 0x480, &amount.to_string());
            y += 24;
        }
        if v.require_exceptional || v.material != mat::NONE {
            hl(b, 75, y, 200, 20, 1045140, WHITE); // Special requirements to meet:
            y += 24;
        }
        if v.require_exceptional {
            hl(b, 75, y, 300, 20, 1045141, WHITE); // All items must be exceptional.
            y += 24;
        }
        if v.material != mat::NONE {
            hl(b, 75, y, 300, 20, material_number(v.material), WHITE);
        }

        b.add_button(125, 168 + n * 24, 4005, 4007, 2);
        hl(b, 160, 168 + n * 24, 300, 20, 1045155, WHITE); // Combine this deed with another deed.
        b.add_button(125, 192 + n * 24, 4005, 4007, 1);
        hl(b, 160, 192 + n * 24, 120, 20, 1011441, WHITE); // EXIT
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if ops::is_deleted(w, self.deed) || !in_pack(w, self.deed, from) {
            return;
        }
        if info.button_id == 2 {
            // Combine
            send_bod_gump(w, from, self.deed);
            begin_combine(w, self.deed, from);
        }
    }
}

// ---- the accept gumps ----

/// The response both accept gumps share.
fn accept_response(w: &mut World, deed: ItemId, sender: NetStateId, info: &RelayInfo) {
    let Some(from) = mobile_of(w, sender) else {
        return;
    };
    if info.button_id == 1 {
        // Ok
        if muo_server::mobile::part2::items::place_in_backpack(w, from, deed) {
            deps::send_localized_message(w, from, 1045152); // The bulk order deed has been placed in your backpack.
        } else {
            deps::send_localized_message(w, from, 1045150); // There is not enough room in your backpack for the deed.
            delete(w, deed);
        }
    } else {
        delete(w, deed);
    }
}

fn delete(w: &mut World, deed: ItemId) {
    if let Some(ty) = w.item(deed).filter(|i| !i.deleted()).map(|i| i.ty) {
        (ty.delete)(w, deed);
    }
}

struct SmallAcceptGump {
    deed: ItemId,
    v: View,
}

impl DynamicGumpLogic for SmallAcceptGump {
    const TYPE_NAME: &'static str = "Server.Engines.BulkOrders.SmallBODAcceptGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let v = &self.v;
        b.add_page(0);
        b.add_background(25, 10, 430, 264, 5054);
        b.add_image_tiled(33, 20, 413, 245, 2624);
        b.add_alpha_region(33, 20, 413, 245);
        b.add_image(20, 5, 10460, 0);
        b.add_image(430, 5, 10460, 0);
        b.add_image(20, 249, 10460, 0);
        b.add_image(430, 249, 10460, 0);

        hl(b, 190, 25, 120, 20, 1045133, WHITE); // A bulk order
        hl(b, 40, 48, 350, 20, 1045135, WHITE); // Ah!  Thanks for the goods!  Would you help me out?
        hl(b, 40, 72, 210, 20, 1045138, WHITE); // Amount to make:
        b.add_label(250, 72, 1152, &v.amount_max.to_string());
        hl(b, 40, 96, 120, 20, 1045136, WHITE); // Item requested:
        b.add_item(385, 96, v.graphic, 0);
        hl(b, 40, 120, 210, 20, v.number, WHITE);

        if v.require_exceptional || v.material != mat::NONE {
            hl(b, 40, 144, 210, 20, 1045140, WHITE); // Special requirements to meet:
            if v.require_exceptional {
                hl(b, 40, 168, 350, 20, 1045141, WHITE); // All items must be exceptional.
            }
            if v.material != mat::NONE {
                hl(
                    b,
                    40,
                    if v.require_exceptional { 192 } else { 168 },
                    350,
                    20,
                    material_number(v.material),
                    WHITE,
                );
            }
        }

        hl(b, 40, 216, 350, 20, 1045139, WHITE); // Do you want to accept this order?
        b.add_button(100, 240, 4005, 4007, 1);
        hl(b, 135, 240, 120, 20, 1006044, WHITE); // Ok
        b.add_button(275, 240, 4005, 4007, 0);
        hl(b, 310, 240, 120, 20, 1011012, WHITE); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        accept_response(w, self.deed, sender, info);
    }

    fn on_server_close(&mut self, w: &mut World, _owner: NetStateId) {
        delete(w, self.deed);
    }
}

struct LargeAcceptGump {
    deed: ItemId,
    v: View,
}

impl DynamicGumpLogic for LargeAcceptGump {
    const TYPE_NAME: &'static str = "Server.Engines.BulkOrders.LargeBODAcceptGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let v = &self.v;
        let n = v.entries.len() as i32;
        b.add_page(0);
        b.add_background(25, 10, 430, 240 + n * 24, 5054);
        b.add_image_tiled(33, 20, 413, 221 + n * 24, 2624);
        b.add_alpha_region(33, 20, 413, 221 + n * 24);
        b.add_image(20, 5, 10460, 0);
        b.add_image(430, 5, 10460, 0);
        b.add_image(20, 225 + n * 24, 10460, 0);
        b.add_image(430, 225 + n * 24, 10460, 0);

        hl(b, 180, 25, 120, 20, 1045134, WHITE); // A large bulk order
        hl(b, 40, 48, 350, 20, 1045135, WHITE); // Ah!  Thanks for the goods!  Would you help me out?
        hl(b, 40, 72, 210, 20, 1045138, WHITE); // Amount to make:
        b.add_label(250, 72, 1152, &v.amount_max.to_string());
        hl(b, 40, 96, 120, 20, 1045137, WHITE); // Items requested:

        let mut y = 120;
        for (number, _) in &v.entries {
            hl(b, 40, y, 210, 20, *number, WHITE);
            y += 24;
        }
        if v.require_exceptional || v.material != mat::NONE {
            hl(b, 40, y, 210, 20, 1045140, WHITE); // Special requirements to meet:
            y += 24;
            if v.require_exceptional {
                hl(b, 40, y, 350, 20, 1045141, WHITE); // All items must be exceptional.
                y += 24;
            }
            if v.material != mat::NONE {
                hl(b, 40, y, 350, 20, material_number(v.material), WHITE);
            }
        }

        hl(b, 40, 192 + n * 24, 350, 20, 1045139, WHITE); // Do you want to accept this order?
        b.add_button(100, 216 + n * 24, 4005, 4007, 1);
        hl(b, 135, 216 + n * 24, 120, 20, 1006044, WHITE); // Ok
        b.add_button(275, 216 + n * 24, 4005, 4007, 0);
        hl(b, 310, 216 + n * 24, 120, 20, 1011012, WHITE); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        accept_response(w, self.deed, sender, info);
    }

    fn on_server_close(&mut self, w: &mut World, _owner: NetStateId) {
        delete(w, self.deed);
    }
}
