//! The `LadderItem` (the 1v1 leaderboard) and the gump it shows.

use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, RelayInfo, send_gump_to_mobile,
};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::html;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::net_state::NetStateId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};

use super::flow::name;
use super::ladder;
use super::speech::{n0, rank};

/// A row of the gump, read when it is laid out.
#[derive(Clone, Debug)]
struct LadderRow {
    experience: i32,
    guild: Option<String>,
    name: String,
    wins: i32,
    losses: i32,
}

/// `LadderGump`.
pub struct LadderGump {
    controller: ItemId,
    page: i32,
    /// `_ladder.Entries.Count`, and the first 150 entries' rows.
    total: usize,
    rows: Vec<LadderRow>,
}

fn snapshot(w: &World, controller: ItemId) -> (usize, Vec<LadderRow>) {
    let Some(l) = w
        .item(controller)
        .and_then(|i| i.ext.get::<ladder::Ladder>())
    else {
        return (0, Vec::new());
    };
    let rows = l
        .entries
        .iter()
        .take(150)
        .map(|e| LadderRow {
            experience: e.experience,
            guild: w
                .mobile(e.mobile)
                .and_then(|m| m.guild)
                .and_then(|g| muo_server::guild::guild(w, g))
                .map(|g| g.abbreviation.clone()),
            name: name(w, e.mobile),
            wins: e.wins,
            losses: e.losses,
        })
        .collect();
    (l.entries.len(), rows)
}

/// `LadderGump.DisplayTo(from, ladder, page)`.
pub fn display_to(w: &mut World, from: MobileId, controller: ItemId, page: i32) {
    if w.mobile(from).and_then(|m| m.net_state).is_none() {
        return;
    }
    let (total, rows) = snapshot(w, controller);
    let g = LadderGump {
        controller,
        page,
        total,
        rows,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(50, 50, g)), true);
}

fn center(text: &str) -> String {
    format!("<CENTER>{text}</CENTER>")
}

impl DynamicGumpLogic for LadderGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.LadderGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let mut column_x = 12;
        b.add_page(0);

        let lc = self.rows.len().min(150) as i32;
        let start = self.page * 15;
        let mut end = start + 15;
        if end > lc {
            end = lc;
        }
        let ct = end - start;
        let height = 12 + 20 + ct * 20 + 23 + 12;
        b.add_background(0, 0, 499, height, 0x2436);

        let mut i = start + 1;
        while i < end {
            b.add_image_tiled(12, 32 + (i - start) * 20, 475, 20, 0x2430);
            i += 2;
        }
        b.add_alpha_region(10, 10, 479, height - 20);

        if self.page > 0 {
            b.add_button(446, height - 12 - 2 - 16, 0x15E3, 0x15E7, 1);
        } else {
            b.add_image(446, height - 12 - 2 - 16, 0x2626, 0);
        }
        if (self.page + 1) * 15 < lc {
            b.add_button(466, height - 12 - 2 - 16, 0x15E1, 0x15E5, 2);
        } else {
            b.add_image(466, height - 12 - 2 - 16, 0x2622, 0);
        }

        let summary = format!(
            "Top {lc} of {} duelists, page {} of {}",
            n0(self.total as i32),
            self.page + 1,
            (lc + 14) / 15
        );
        b.add_html(
            16,
            height - 12 - 2 - 18,
            400,
            20,
            &html::color_rgb(&summary, 0xFFC000),
            false,
            false,
        );

        let mut header = |b: &mut DynamicGumpBuilder, width: i32, text: &str| {
            b.add_background(column_x, 12, width, 20, 0x242C);
            b.add_image_tiled(column_x + 2, 14, width - 4, 16, 0x2430);
            b.add_html(
                column_x,
                13,
                width,
                20,
                &html::color_rgb(&center(text), 0xFFFFFF),
                false,
                false,
            );
            column_x += width;
        };
        header(b, 75, "Rank");
        header(b, 115, "Level");
        header(b, 50, "Guild");
        header(b, 115, "Name");
        header(b, 60, "Wins");
        header(b, 60, "Losses");

        let colored = |text: &str| html::color_rgb(text, 0xFFFFFF);
        let mut i = start;
        while i < end && i < lc {
            let e = &self.rows[i as usize];
            let y = 32 + (i - start) * 20;
            let mut x = 12;

            b.add_html(x, y, 75, 20, &colored(&center(&rank(i + 1))), false, false);
            x += 75;

            b.add_image(x + 3, y + 4, 0x805, 0);
            let xp = e.experience;
            let level = ladder::get_level(xp);
            let (xp_base, xp_advance) = ladder::get_level_info(level);
            let xp_offset = xp - xp_base;
            let width = if xp_offset >= xp_advance {
                109 // level 50
            } else {
                (109 * xp_offset + xp_advance / 2) / (xp_advance - 1)
            };
            b.add_image_tiled(x + 3, y + 4, width, 11, 0x806);
            b.add_html(
                x,
                y,
                115,
                20,
                &colored(&center(&level.to_string())),
                false,
                false,
            );
            x += 115;

            if let Some(g) = &e.guild {
                b.add_html(x, y, 50, 20, &colored(&center(g)), false, false);
            }
            x += 50;

            b.add_html(x + 5, y, 115 - 5, 20, &colored(&e.name), false, false);
            x += 115;
            b.add_html(
                x,
                y,
                60,
                20,
                &colored(&center(&e.wins.to_string())),
                false,
                false,
            );
            x += 60;
            b.add_html(
                x,
                y,
                60,
                20,
                &colored(&center(&e.losses.to_string())),
                false,
                false,
            );
            i += 1;
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let (total, _) = snapshot(w, self.controller);
        let mut page = self.page;
        if info.button_id == 1 && page > 0 {
            page -= 1;
        } else if info.button_id == 2 && (page + 1) * 15 < total.min(150) as i32 {
            page += 1;
        }
        display_to(w, from, self.controller, page); // refresh-via-this
    }
}

// ---- LadderItem ---------------------------------------------------------------------------------------------------

/// The field of `LadderItem`: the `LadderController` it shows.
#[derive(Default)]
pub struct LadderItemData {
    pub ladder: Option<ItemId>,
}

fn double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    if w.mobile(from)
        .is_some_and(|m| (m.ty.in_range_point3d)(w, from, loc, 2))
    {
        let own = w
            .item(id)
            .and_then(|i| i.ext.get::<LadderItemData>())
            .and_then(|d| d.ladder);
        // `ConPVP.Ladder.Instance ?? Ladder.Ladder`
        if let Some(l) = ladder::instance(w).or(own) {
            display_to(w, from, l, 0);
        }
    } else {
        muo_server::mobile::part2::messages::local_overhead_message_number(
            w,
            from,
            muo_primitives::MessageType::REGULAR,
            0x3B2,
            1019045,
            "",
        ); // I can't reach that
    }
}

/// `LadderItem`.
pub static LADDER_ITEM: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.LadderItem",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &LADDER_ITEM, 0x117F);
        ops::set_movable(w, id, false);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(LadderItemData::default());
        }
        id
    }),
    default_name: |_, _| Some("1v1 leaderboard".to_owned()),
    on_double_click: double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let l = w
            .item(id)
            .and_then(|i| i.ext.get::<LadderItemData>())
            .and_then(|d| d.ladder);
        out.write_item(w, l);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let l = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(LadderItemData { ladder: l });
        }
        Ok(())
    },
    ..ITEM_BASE
};

#[allow(dead_code)]
fn _unused(_: &dyn GenericReader, _: &dyn GenericWriter) {}
