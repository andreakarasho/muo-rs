//! `WhoGump`.

use muo_primitives::dotnet::InvariantCase;
use std::rc::Rc;

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;

use super::common::{mobile_of, name_of, send};
use super::hooks::hooks;
use muo_content::gumps::props_config::*;
use muo_content::gumps::props_layout::*;

const ENTRY_WIDTH: i32 = 180;
const ENTRY_COUNT: usize = 15;

const TOTAL_WIDTH: i32 = OFFSET_SIZE + ENTRY_WIDTH + OFFSET_SIZE + SET_WIDTH + OFFSET_SIZE;

/// What a row shows of its mobile.
struct Row {
    name: String,
    /// `!m.Deleted && m.NetState != null`
    has_button: bool,
    hue: i32,
}

pub struct WhoGump {
    mobiles: Rc<Vec<MobileId>>,
    page: usize,
    rows: Vec<Row>,
}

/// `new WhoGump(owner, filter)`.
pub fn new_for(w: &World, owner: MobileId, filter: &str) -> DynamicGumpObj<WhoGump> {
    new(w, Rc::new(build_list(w, owner, filter)), 0)
}

/// `new WhoGump(list, page)`.
pub fn new(w: &World, list: Rc<Vec<MobileId>>, page: usize) -> DynamicGumpObj<WhoGump> {
    let count = list
        .len()
        .saturating_sub(page * ENTRY_COUNT)
        .min(ENTRY_COUNT);
    let rows = list
        .iter()
        .skip(page * ENTRY_COUNT)
        .take(count)
        .map(|&m| {
            let deleted = w.mobile(m).is_none_or(|mm| mm.deleted);
            let online = w.mobile(m).is_some_and(|mm| mm.net_state.is_some());
            Row {
                name: if deleted {
                    "(deleted)".to_owned()
                } else {
                    name_of(w, m)
                },
                has_button: online && !deleted,
                hue: get_hue_for(w, m),
            }
        })
        .collect();
    DynamicGumpObj::new(
        GUMP_OFFSET_X,
        GUMP_OFFSET_Y,
        WhoGump {
            mobiles: list,
            page,
            rows,
        },
    )
}

/// `WhoGump.Configure`.
pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "WhoList",
        AccessLevel::Counselor,
        &["Who"],
        Rc::new(who_list_on_command),
    );
}

/// `WhoList_OnCommand`.
fn who_list_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let g = new_for(w, e.mobile, &e.arg_string);
    send(w, e.mobile, g);
}

/// `m.Name.InsensitiveContains(filter)`.
fn insensitive_contains(haystack: &str, needle: &str) -> bool {
    haystack.upper_inv().contains(&needle.upper_inv())
}

/// `WhoGump.BuildList(owner, rawFilter)`.
pub fn build_list(w: &World, owner: MobileId, raw_filter: &str) -> Vec<MobileId> {
    let filter = raw_filter.trim();

    let owner_access = muo_content::mx::access_level(w, owner);
    let mut list = Vec::new();

    for ns in w.net_states.values() {
        let Some(m) = ns.mobile else {
            continue;
        };
        let Some(mm) = w.mobile(m) else {
            continue;
        };

        if m != owner
            && mm.hidden
            && owner_access < mm.access_level
            && !muo_content::player_mobile::data(w, m)
                .is_some_and(|d| d.visibility_list.contains(&owner))
        {
            continue;
        }

        if !filter.is_empty()
            && !insensitive_contains(&(mm.ty.name)(w, m).unwrap_or_default(), filter)
        {
            continue;
        }

        list.push(m);
    }

    // InternalComparer: higher access level first, then by name.
    list.sort_by(|&x, &y| {
        let (ax, ay) = (
            muo_content::mx::access_level(w, x),
            muo_content::mx::access_level(w, y),
        );
        ay.cmp(&ax)
            .then_with(|| muo_primitives::dotnet::cmp_ignore_case(&name_of(w, x), &name_of(w, y)))
    });

    list
}

/// `GetHueFor(m)`.
fn get_hue_for(w: &World, m: MobileId) -> i32 {
    match muo_content::mx::access_level(w, m) {
        a if a >= AccessLevel::Administrator => 0x516,
        AccessLevel::Seer => 0x144,
        AccessLevel::GameMaster => 0x21,
        AccessLevel::Counselor => 0x2,
        _ if muo_server::mobile::base::murderer(w, m) => 0x21,
        _ if muo_server::mobile::base::criminal(w, m) => 0x3B1,
        _ => 0x58,
    }
}

impl DynamicGumpLogic for WhoGump {
    const TYPE_NAME: &'static str = "Server.Gumps.WhoGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let total = self.mobiles.len();
        let count = self.rows.len();

        b.add_page(0);

        let (mut x, mut y) = add_props_frame(b, TOTAL_WIDTH, count as i32 + 1);

        const EMPTY_WIDTH: i32 = TOTAL_WIDTH - PREV_WIDTH - NEXT_WIDTH - OFFSET_SIZE * 4;

        b.add_image_tiled(x, y, EMPTY_WIDTH, ENTRY_HEIGHT, ENTRY_GUMP_ID);

        b.add_label(
            x + TEXT_OFFSET_X,
            y,
            TEXT_HUE,
            &format!(
                "Page {} of {} ({})",
                self.page + 1,
                total.div_ceil(ENTRY_COUNT),
                total
            ),
        );

        x += EMPTY_WIDTH + OFFSET_SIZE;

        b.add_image_tiled(x, y, PREV_WIDTH, ENTRY_HEIGHT, HEADER_GUMP_ID);

        if self.page > 0 {
            b.add_button(
                x + PREV_OFFSET_X,
                y + PREV_OFFSET_Y,
                PREV_BUTTON_ID1,
                PREV_BUTTON_ID2,
                1,
            );
        }

        x += PREV_WIDTH + OFFSET_SIZE;

        b.add_image_tiled(x, y, NEXT_WIDTH, ENTRY_HEIGHT, HEADER_GUMP_ID);

        if (self.page + 1) * ENTRY_COUNT < total {
            b.add_button_type(
                x + NEXT_OFFSET_X,
                y + NEXT_OFFSET_Y,
                NEXT_BUTTON_ID1,
                NEXT_BUTTON_ID2,
                2,
                GumpButtonType::Reply,
                1,
            );
        }

        for (i, row) in self.rows.iter().enumerate() {
            next_row(&mut x, &mut y);

            add_props_entry_button(
                b,
                &mut x,
                &mut y,
                ENTRY_WIDTH,
                &row.name,
                row.has_button.then_some(i as i32 + 3),
                row.hue,
            );
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let total = self.mobiles.len();

        match info.button_id {
            0 => {} // Closed
            1 => {
                // Previous
                if self.page > 0 {
                    let g = new(w, self.mobiles.clone(), self.page - 1);
                    send(w, from, g);
                }
            }
            2 => {
                // Next
                if (self.page + 1) * ENTRY_COUNT < total {
                    let g = new(w, self.mobiles.clone(), self.page + 1);
                    send(w, from, g);
                }
            }
            button => {
                let index = self.page as i64 * ENTRY_COUNT as i64 + i64::from(button - 3);

                if index >= 0 && (index as usize) < total {
                    let m = self.mobiles[index as usize];
                    let (deleted, ns) = w
                        .mobile(m)
                        .map_or((true, None), |mm| (mm.deleted, mm.net_state));

                    if deleted {
                        muo_content::mx::send_message(
                            w,
                            from,
                            "That player has deleted their character.",
                        );
                        let g = new(w, self.mobiles.clone(), self.page);
                        send(w, from, g);
                    } else if ns.is_none() {
                        muo_content::mx::send_message(w, from, "That player is no longer online.");
                        let g = new(w, self.mobiles.clone(), self.page);
                        send(w, from, g);
                    } else {
                        let hidden = w.mobile(m).is_some_and(|mm| mm.hidden);
                        let visible_to_from = m == from
                            || !hidden
                            || muo_content::mx::access_level(w, from)
                                >= muo_content::mx::access_level(w, m)
                            || muo_content::player_mobile::data(w, m)
                                .is_some_and(|d| d.visibility_list.contains(&from));
                        if visible_to_from {
                            // from.SendGump(new ClientGump(from, m.NetState))
                            if let (Some(f), Some(ns)) =
                                (hooks(w).and_then(|h| h.open_client_gump), ns)
                            {
                                f(w, from, ns);
                            }
                        } else {
                            muo_content::mx::send_message(w, from, "You cannot see them.");
                            let g = new(w, self.mobiles.clone(), self.page);
                            send(w, from, g);
                        }
                    }
                }
            }
        }
    }
}
