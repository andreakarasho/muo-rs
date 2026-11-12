//! The `ArenasMoongate` item and the `ArenaGump` list of arenas it opens.

use muo_content::gumps::{DynamicGumpBuilder, DynamicGumpLogic, RelayInfo};
use muo_primitives::{MessageType, html};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages as mm;
use muo_server::net_state::NetStateId;
use muo_server::type_registry::TypeRegistry;

use super::flow::name;
use super::gumps::{center, mobile_of, ns_of, send_dynamic};
use super::{arena, flow, ladder};

/// `ArenasMoongate`.
pub static ARENAS_MOONGATE: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.ArenasMoongate",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &ARENAS_MOONGATE, 0x1FD4);
        ops::set_movable(w, id, false);
        ops::set_light(w, id, muo_server::light_type::LightType::Circle300);
        id
    }),
    default_name: |_, _| Some("arena moongate".to_owned()),
    on_double_click: |w, id, from| {
        let loc = ops::get_world_location(w, id);
        if muo_content::mx::in_range_point(w, from, loc, 1) {
            use_gate(w, id, from);
        } else {
            mm::local_overhead_message_number(w, from, MessageType::REGULAR, 0x3B2, 1019045, ""); // I can't reach that
        }
    },
    on_move_over: |w, id, m| !muo_content::mx::player(w, m) || use_gate(w, id, m),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `UseGate(from)`.
fn use_gate(w: &mut World, gate: ItemId, from: MobileId) -> bool {
    if flow::check_combat(w, from) {
        mm::send_message_hue(
            w,
            from,
            0x22,
            "You have recently been in combat with another player and cannot use this moongate.",
        );
        return false;
    }
    if w.mobile(from).is_some_and(|m| m.spell.is_some()) {
        mm::send_localized_message(w, from, 1049616, "", 0x3B2); // You are too busy to do that at the moment.
        return false;
    }
    display_arenas(w, from, gate);
    let (hidden, level, loc, map) = w.mobile(from).map_or(
        (false, AccessLevel::Player, Default::default(), None),
        |m| (m.hidden, m.access_level, m.location, m.map),
    );
    if !hidden || level == AccessLevel::Player {
        muo_server::effects::play_sound(w, loc, map, 0x20E);
    }
    true
}

/// `ArenaGump`: the arenas as they were when it was sent.
pub struct ArenaGump {
    gate: ItemId,
    arenas: Vec<ItemId>,
    rows: Vec<Row>,
}

struct Row {
    name: String,
    occupied: bool,
    players: String,
    spectators: i32,
    /// `Ladder.Instance == null` while players are inside skips the row's remaining cells (`continue`).
    skipped: bool,
}

fn display_arenas(w: &mut World, from: MobileId, gate: ItemId) {
    if ns_of(w, from).is_none() {
        return;
    }
    let arenas = arena::arenas(w);
    let lad = ladder::instance(w);
    let mut rows = Vec::new();
    for &a in &arenas {
        let d = arena::data(w, a).cloned().unwrap_or_default();
        let mut players = String::new();
        let mut skipped = false;
        if !d.players.is_empty() {
            match lad {
                None => skipped = true,
                Some(l) => {
                    // The four lowest ladder indexes, in order.
                    let mut entries = Vec::new();
                    for &m in &d.players {
                        if let Some(e) = ladder::find(w, l, m) {
                            entries.push(e);
                        }
                    }
                    let (mut p1, mut p2, mut p3, mut p4): (
                        Option<_>,
                        Option<_>,
                        Option<_>,
                        Option<_>,
                    ) = (None, None, None, None);
                    for c in entries {
                        if p1
                            .as_ref()
                            .is_none_or(|p: &ladder::LadderEntry| c.index < p.index)
                        {
                            (p4, p3, p2, p1) = (p3, p2, p1, Some(c));
                        } else if p2.as_ref().is_none_or(|p| c.index < p.index) {
                            (p4, p3, p2) = (p3, p2, Some(c));
                        } else if p3.as_ref().is_none_or(|p| c.index < p.index) {
                            (p4, p3) = (p3, Some(c));
                        } else if p4.as_ref().is_none_or(|p| c.index < p.index) {
                            p4 = Some(c);
                        }
                    }
                    for e in [p1, p2, p3, p4].into_iter().flatten() {
                        if !players.is_empty() {
                            players.push_str(", ");
                        }
                        players.push_str(&name(w, e.mobile));
                    }
                    if d.players.len() > 4 {
                        players.push_str(", ...");
                    }
                }
            }
        } else {
            players.push_str("Empty");
        }
        let spectators = d.region.map_or(0, |r| {
            (muo_server::region::get_player_count(w, r) as i32 - d.players.len() as i32).max(0)
        });
        rows.push(Row {
            name: d.name.clone().unwrap_or_else(|| "(no name)".to_owned()),
            occupied: !d.players.is_empty(),
            players,
            spectators,
            skipped,
        });
    }
    send_dynamic(w, from, 50, 50, ArenaGump { gate, arenas, rows }, true);
}

fn colored(text: &str, color: i32) -> String {
    html::color_rgb(text, color)
}

fn column_header(b: &mut DynamicGumpBuilder, column_x: &mut i32, width: i32, name: Option<&str>) {
    b.add_background(*column_x, 12, width, 20, 0x242C);
    b.add_image_tiled(*column_x + 2, 14, width - 4, 16, 0x2430);
    if let Some(n) = name {
        b.add_html(
            *column_x,
            13,
            width,
            20,
            &colored(&center(n), 0xFFFFFF),
            false,
            false,
        );
    }
    *column_x += width;
}

impl DynamicGumpLogic for ArenaGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.ArenaGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let mut column_x = 12;
        b.add_page(0);
        let height = 12 + 20 + self.rows.len() as i32 * 31 + 24 + 12;
        b.add_background(0, 0, 499 + 40, height, 0x2436);
        let mut i = 1;
        while i < self.rows.len() as i32 {
            b.add_image_tiled(12, 32 + i * 31, 475 + 40, 30, 0x2430);
            i += 2;
        }
        b.add_alpha_region(10, 10, 479 + 40, height - 20);

        column_header(b, &mut column_x, 35, None);
        column_header(b, &mut column_x, 115, Some("Arena"));
        column_header(b, &mut column_x, 325, Some("Participants"));
        column_header(b, &mut column_x, 40, Some("Obs"));

        b.add_button(499 + 40 - 12 - 63 - 4 - 63, height - 12 - 24, 247, 248, 1);
        b.add_button(499 + 40 - 12 - 63, height - 12 - 24, 241, 242, 2);

        for (i, row) in self.rows.iter().enumerate() {
            let mut x = 12;
            let y = 32 + i as i32 * 31;
            let color = if row.occupied { 0xCCFFCC } else { 0xCCCCCC };
            b.add_radio(x + 3, y + 1, 9727, 9730, false, i as i32);
            x += 35;
            b.add_html(
                x + 5,
                y + 5,
                115 - 5,
                20,
                &colored(&row.name, color),
                false,
                false,
            );
            x += 115;
            if row.skipped {
                continue;
            }
            b.add_html(
                x + 5,
                y + 5,
                325 - 5,
                20,
                &colored(&row.players, color),
                false,
                false,
            );
            x += 325;
            b.add_html(
                x,
                y + 5,
                40,
                20,
                &colored(&center(&row.spectators.to_string()), color),
                false,
                false,
            );
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id != 1 {
            return;
        }
        let Some(&opt) = info.switches.first() else {
            return;
        };
        if opt < 0 || opt as usize >= self.arenas.len() {
            return;
        }
        let a = self.arenas[opt as usize];
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let d = arena::data(w, a).cloned().unwrap_or_default();
        let gate_loc = ops::get_world_location(w, self.gate);
        let gate_map = w.item(self.gate).map(|i| i.map);

        if !muo_content::mx::in_range_point(w, from, gate_loc, 1)
            || Some(muo_content::mx::map(w, from)) != gate_map
        {
            mm::send_localized_message(w, from, 1019002, "", 0x3B2); // You are too far away to use the gate.
        } else if flow::check_combat(w, from) {
            mm::send_message_hue(
                w,
                from,
                0x22,
                "You have recently been in combat with another player and cannot use this moongate.",
            );
        } else if w.mobile(from).is_some_and(|m| m.spell.is_some()) {
            mm::send_localized_message(w, from, 1049616, "", 0x3B2); // You are too busy to do that at the moment.
        } else if muo_content::mx::map(w, from) == d.facet && {
            let l = muo_content::mx::location(w, from);
            d.zone.contains(muo_primitives::Point2D::new(l.x, l.y))
        } {
            mm::send_localized_message(w, from, 1019003, "", 0x3B2); // You are already there.
        } else {
            if let Some(map) = d.facet {
                muo_content::creature::pets::teleport_pets(w, from, d.gate_in, map, false);
            }
            muo_content::mx::set_combatant(w, from, None);
            muo_content::mx::set_warmode(w, from, false);
            muo_server::mobile::part1::props_b::set_hidden(w, from, true);
            if let Some(map) = d.facet {
                muo_server::mobile::base::move_to_world(w, from, d.gate_in, map);
            }
            muo_server::effects::play_sound(w, d.gate_in, d.facet, 0x1FE);
        }
    }
}

/// Registers the item class.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ARENAS_MOONGATE, &[]);
}
