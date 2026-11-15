//! Player bulletin boards: `PlayerBBSouth`, `PlayerBBEast`, `BasePlayerBB` with its two
//! prompts, `PlayerBBMessage` and `PlayerBBGump`.
//!
//! The GM-only "post props" button (8) opens a `PropertiesGump` over the message (`props_hand::PLAYER_BB_POST`).

use std::rc::Rc;

use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_gumps::m_z::props::registry::SubObj;
use muo_io::{GenericReader, GenericWriter};
use muo_multis::house::{self, access};
use muo_primitives::{DateTime, MessageType};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

use super::{read_version, write_version};

/// `PlayerBBMessage`.
#[derive(Clone, Debug, Default)]
pub struct BbMessage {
    pub time: DateTime,
    pub poster: Option<MobileId>,
    pub message: Option<String>,
}

/// What `BasePlayerBB` adds to an `Item`.
#[derive(Clone, Debug)]
pub struct BbData {
    /// `SecureLevel`: Owner 0, CoOwners 1, Friends 2, Anyone 3, Guild 4.
    pub level: i32,
    pub title: Option<String>,
    pub greeting: Option<BbMessage>,
    pub messages: Vec<BbMessage>,
}

pub(crate) fn data(w: &World, id: ItemId) -> Option<&BbData> {
    w.item(id)?.ext.get::<BbData>()
}

/// `Greeting`.
pub fn greeting(w: &World, id: ItemId) -> Option<BbMessage> {
    data(w, id)?.greeting.clone()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut BbData> {
    w.item_mut(id)?.ext.get_mut::<BbData>()
}

fn bb(w: &World, id: ItemId) -> BbData {
    data(w, id).cloned().unwrap_or(BbData {
        level: 3,
        title: None,
        greeting: None,
        messages: Vec::new(),
    })
}

pub fn is_player_bb(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_PLAYER_BB))
}

/// `BasePlayerBB(itemID)`: `Level = SecureLevel.Anyone`.
pub fn player_bb_new(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BbData {
            level: 3,
            title: None,
            greeting: None,
            messages: Vec::new(),
        });
    }
    id
}

/// `BasePlayerBB.CheckAccess(house, from)`.
pub fn check_access(w: &World, house_id: ItemId, from: MobileId) -> bool {
    if house::d(w, house_id).public || !house::is_aos_rules(w, house_id) {
        return !access::is_banned(w, house_id, from);
    }
    access::has_access(w, house_id, from)
}

fn locked_down_in_house(w: &World, board: ItemId) -> Option<ItemId> {
    access::find_house_at_item(w, board)
        .filter(|h| muo_multis::house::storage::has_locked_down_item(w, *h, board))
}

fn cant_reach(w: &mut World, from: MobileId) {
    muo_server::mobile::part2::messages::local_overhead_message_number(
        w,
        from,
        MessageType::REGULAR,
        0x3B2,
        1019045,
        "",
    );
}

fn reach_ok(w: &mut World, from: MobileId, board: ItemId) -> bool {
    let loc = ops::get_world_location(w, board);
    deps::mobile_in_range(w, from, loc, 2) && deps::in_los_item(w, from, board)
}

fn bb_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(house_id) = locked_down_in_house(w, id) else {
        deps::send_localized_message(w, from, 1062396); // This bulletin board must be locked down in a house to be usable.
        return;
    };
    if !reach_ok(w, from, id) {
        cant_reach(w, from);
    } else if check_access(w, house_id, from) {
        display_gump(w, from, house_id, id, 0);
    }
}

/// `PostPrompt` and `SetTitlePrompt`: both treat a cancel as an empty response.
fn begin_prompt(
    w: &mut World,
    from: MobileId,
    page: i32,
    house_id: ItemId,
    board: ItemId,
    kind: PromptKind,
) {
    let cb: muo_server::prompt::PromptCallback =
        Rc::new(move |w: &mut World, from: MobileId, text: &str| {
            prompt_response(w, from, page, house_id, board, kind, text)
        });
    let prompt = muo_server::prompt::SimplePrompt::new(Some(cb), true);
    let pid = muo_server::prompt::add_prompt(w, prompt.into_prompt());
    muo_server::mobile::part1::props_b::set_prompt(w, from, Some(pid));
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum PromptKind {
    Post,
    Greeting,
    Title,
}

fn prompt_response(
    w: &mut World,
    from: MobileId,
    mut page: i32,
    house_id: ItemId,
    board: ItemId,
    kind: PromptKind,
    text: &str,
) {
    if !muo_multis::house::storage::has_locked_down_item(w, house_id, board) {
        deps::send_localized_message(w, from, 1062396);
        return;
    }
    if !reach_ok(w, from, board) {
        cant_reach(w, from);
        return;
    }
    if !check_access(w, house_id, from) {
        deps::send_localized_message(w, from, 1062398); // You are not allowed to post to this bulletin board.
        return;
    }
    if kind == PromptKind::Greeting && !access::is_owner(w, house_id, from) {
        return;
    }
    let mut text: String = text.trim().to_string();
    if text.chars().count() > 255 {
        text = text.chars().take(255).collect();
    }
    if !text.is_empty() {
        match kind {
            PromptKind::Title => {
                if let Some(d) = data_mut(w, board) {
                    d.title = Some(text); // no [InvalidateProperties]
                }
            }
            _ => {
                let msg = BbMessage {
                    time: w.now,
                    poster: Some(from),
                    message: Some(text),
                };
                if kind == PromptKind::Greeting {
                    if let Some(d) = data_mut(w, board) {
                        d.greeting = Some(msg);
                    }
                } else if let Some(d) = data_mut(w, board) {
                    d.messages.push(msg);
                    if d.messages.len() > 50 {
                        d.messages.remove(0);
                        if page > 0 {
                            page -= 1;
                        }
                    }
                }
            }
        }
    }
    display_gump(w, from, house_id, board, page);
}

// ---- the gump ----

const LABEL_COLOR: i32 = 0x7FFF;
const LABEL_HUE: i32 = 1153;

/// `PlayerBBGump`.
pub struct PlayerBbGump {
    from: MobileId,
    house: ItemId,
    board: ItemId,
    page: i32,
    /// The state `BuildLayout` reads, taken when the gump is made.
    owner: bool,
    gm: bool,
    data: BbData,
    poster_name: Option<String>,
}

fn display_gump(w: &mut World, from: MobileId, house_id: ItemId, board: ItemId, page: i32) {
    if w.mobile(from).and_then(|m| m.net_state).is_none() || ops::is_deleted(w, board) {
        return;
    }
    let d = bb(w, board);
    let message = message_at(&d, page);
    let poster_name = message.as_ref().map(|m| {
        m.poster
            .and_then(|p| w.mobile(p).and_then(|mm| (mm.ty.name)(w, p)))
            .unwrap_or_else(|| "Someone".to_string())
    });
    let g = PlayerBbGump {
        from,
        house: house_id,
        board,
        page,
        owner: access::is_owner(w, house_id, from),
        gm: w
            .mobile(from)
            .is_some_and(|m| m.access_level >= AccessLevel::GameMaster),
        data: d,
        poster_name,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(50, 10, g)), false);
}

fn message_at(d: &BbData, page: i32) -> Option<BbMessage> {
    if page >= 1 && page as usize <= d.messages.len() {
        return Some(d.messages[page as usize - 1].clone());
    }
    d.greeting.clone()
}

impl DynamicGumpLogic for PlayerBbGump {
    const TYPE_NAME: &'static str = "Server.Items.PlayerBBGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let page = self.page;
        b.add_page(0);
        b.add_image(30, 30, 5400, 0);
        b.add_button(393, 145, 2084, 2084, 4); // Scroll up
        b.add_button(390, 371, 2085, 2085, 5); // Scroll down
        b.add_button(32, 183, 5412, 5413, 1); // Post message
        if self.owner {
            b.add_button(63, 90, 5601, 5605, 2);
            b.add_html_localized_color(81, 89, 230, 20, 1062400, LABEL_COLOR, false, false); // Set title
            b.add_button(63, 109, 5601, 5605, 3);
            b.add_html_localized_color(81, 108, 230, 20, 1062401, LABEL_COLOR, false, false); // Post greeting
        }
        if let Some(t) = &self.data.title {
            b.add_html(183, 68, 180, 23, t, false, false);
        }
        b.add_html_localized_color(385, 89, 60, 20, 1062409, LABEL_COLOR, false, false); // Post
        b.add_label(440, 89, LABEL_HUE, &page.to_string());
        b.add_label(455, 89, LABEL_HUE, "/");
        b.add_label(470, 89, LABEL_HUE, &self.data.messages.len().to_string());
        let message = message_at(&self.data, page);
        b.add_image_tiled(150, 220, 240, 1, 2700); // Separator
        b.add_html_localized_color(150, 180, 100, 20, 1062405, 16715, false, false); // Posted On:
        b.add_html_localized_color(150, 200, 100, 20, 1062406, 16715, false, false); // Posted By:
        if let Some(m) = message {
            let t = m.time;
            b.add_html(
                255,
                180,
                150,
                20,
                &format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    t.year(),
                    t.month(),
                    t.day(),
                    t.hour(),
                    t.minute(),
                    t.second()
                ),
                false,
                false,
            );
            b.add_html(
                255,
                200,
                150,
                20,
                self.poster_name.as_deref().unwrap_or("Someone").trim(),
                false,
                false,
            );
            b.add_html(
                150,
                240,
                250,
                100,
                m.message.as_deref().unwrap_or(""),
                false,
                false,
            );
            let is_greeting = self.data.greeting.as_ref().is_some_and(|g| {
                g.time == m.time && g.poster == m.poster && g.message == m.message
            }) && !(page >= 1 && page as usize <= self.data.messages.len());
            if !is_greeting && self.owner {
                b.add_button(130, 395, 1209, 1210, 6);
                b.add_html_localized_color(150, 393, 150, 20, 1062410, LABEL_COLOR, false, false); // Banish Poster
                b.add_button(310, 395, 1209, 1210, 7);
                b.add_html_localized_color(330, 393, 150, 20, 1062411, LABEL_COLOR, false, false); // Delete Message
            }
            if self.gm {
                b.add_button(135, 242, 1209, 1210, 8); // Post props
            }
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let (from, house_id, board) = (self.from, self.house, self.board);
        if !muo_multis::house::storage::has_locked_down_item(w, house_id, board) {
            deps::send_localized_message(w, from, 1062396); // This bulletin board must be locked down in a house to be usable.
            return;
        }
        if !reach_ok(w, from, board) {
            cant_reach(w, from);
            return;
        }
        if !check_access(w, house_id, from) {
            deps::send_localized_message(w, from, 1062398); // You are not allowed to post to this bulletin board.
            return;
        }
        let count = bb(w, board).messages.len() as i32;
        let owner = access::is_owner(w, house_id, from);
        let page = self.page;
        match info.button_id {
            1 => {
                // Post message
                begin_prompt(w, from, page, house_id, board, PromptKind::Post);
                deps::send_localized_message(w, from, 1062397); // Please enter your message:
            }
            2 => {
                if owner {
                    begin_prompt(w, from, page, house_id, board, PromptKind::Title);
                    deps::send_localized_message(w, from, 1062402); // Enter new title:
                }
            }
            3 => {
                if owner {
                    begin_prompt(w, from, page, house_id, board, PromptKind::Greeting);
                    deps::send_localized_message(w, from, 1062404); // Enter new greeting (this will always be the first post):
                }
            }
            4 => {
                let p = if page == 0 { count } else { page - 1 };
                display_gump(w, from, house_id, board, p);
            }
            5 => {
                let p = (page + 1) % (count + 1);
                display_gump(w, from, house_id, board, p);
            }
            6 => {
                if owner {
                    if page >= 1 && page <= count {
                        let poster = bb(w, board).messages[page as usize - 1].poster;
                        let Some(poster) = poster else {
                            display_gump(w, from, house_id, board, page);
                            return;
                        };
                        ban_poster(w, from, house_id, poster);
                    }
                    display_gump(w, from, house_id, board, page);
                }
            }
            7 => {
                if owner {
                    if page >= 1
                        && page <= count
                        && let Some(d) = data_mut(w, board)
                    {
                        d.messages.remove(page as usize - 1);
                    }
                    display_gump(w, from, house_id, board, 0);
                }
            }
            8 if w
                .mobile(from)
                .is_some_and(|m| m.access_level >= AccessLevel::GameMaster) =>
            {
                // `board.Greeting`, or `board.Messages[_page - 1]`
                let key = if page >= 1 && page <= count {
                    page as usize - 1
                } else {
                    usize::MAX
                };
                display_gump(w, from, house_id, board, page);
                let obj = if key != usize::MAX {
                    Some(SubObj::keyed(
                        EntityId::Item(board),
                        crate::props_hand::PLAYER_BB_POST,
                        "PlayerBBMessage",
                        key,
                    ))
                } else if data(w, board).is_some_and(|d| d.greeting.is_some()) {
                    Some(SubObj::object(
                        EntityId::Item(board),
                        crate::props_hand::PLAYER_BB_MESSAGE,
                        "PlayerBBMessage",
                    ))
                } else {
                    None // a `PropertiesGump` needs an object
                };
                if let Some(o) = obj {
                    let g = muo_gumps::m_z::props::properties::new(w, from, o);
                    muo_gumps::m_z::common::send(w, from, g);
                }
            }
            _ => {}
        }
    }
}

fn ban_poster(w: &mut World, from: MobileId, house_id: ItemId, poster: MobileId) {
    let (from_level, poster_level) = (
        w.mobile(from)
            .map_or(AccessLevel::Player, |m| m.access_level),
        w.mobile(poster)
            .map_or(AccessLevel::Player, |m| m.access_level),
    );
    if poster_level > AccessLevel::Player && from_level <= poster_level {
        deps::send_localized_message(w, from, 501354); // Uh oh...a bigger boot may be required.
    } else if access::is_friend(w, house_id, poster) {
        // That person is a friend, co-owner, or owner of this house, and therefore cannot be banished!
        deps::send_localized_message(w, from, 1060750);
    } else if muo_content::factory::mobile_is(w, poster, "Server.Mobiles.PlayerVendor") {
        deps::send_localized_message(w, from, 501351); // You cannot eject a vendor.
    } else if house::data(w, house_id).is_some_and(|h| h.bans.len() >= house::max_bans(w)) {
        deps::send_localized_message(w, from, 501355); // The ban limit for this house has been reached!
    } else if access::is_banned(w, house_id, poster) {
        deps::send_localized_message(w, from, 501356); // This person is already banned!
    } else if muo_content::creature::vtable::ctype(w, poster)
        .is_some_and(|c| (c.no_house_restrictions)(w, poster))
    {
        deps::send_localized_message(w, from, 1062040); // You cannot ban that.
    } else {
        if let Some(h) = house::data_mut(w, house_id)
            && !h.bans.contains(&poster)
        {
            h.bans.push(poster);
        }
        deps::send_localized_message(w, from, 1062417); // That person has been banned from this house.
        if access::is_inside_mobile(w, house_id, poster)
            && !check_access(w, house_id, poster)
            && let Some(Some(map)) = w.item(house_id).map(|i| i.map)
        {
            let loc = house::ban_location(w, house_id);
            muo_server::mobile::base::move_to_world(w, poster, loc, map);
        }
    }
}

// ---- serialization ----

fn write_message(w: &World, out: &mut dyn GenericWriter, m: &BbMessage) {
    out.write_encoded_int(0);
    out.write_date_time_ticks(m.time.ticks());
    out.write_mobile(w, m.poster);
    out.write_opt_string(m.message.as_deref());
}

fn read_message(w: &World, r: &mut dyn GenericReader) -> muo_io::Result<BbMessage> {
    r.read_encoded_int()?;
    Ok(BbMessage {
        time: DateTime::from_ticks(r.read_date_time_ticks()?),
        poster: r.read_mobile(w)?,
        message: r.read_string()?,
    })
}

fn serialize_bb(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    write_version(out, false);
    let d = bb(w, id);
    out.write_enum(4, i64::from(d.level));
    out.write_opt_string(d.title.as_deref());
    match &d.greeting {
        Some(g) => {
            out.write_bool(true);
            write_message(w, out, g);
        }
        None => out.write_bool(false),
    }
    out.write_encoded_int(d.messages.len() as i32);
    for m in &d.messages {
        write_message(w, out, m);
    }
}

fn deserialize_bb(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    read_version(r, false)?;
    let level = r.read_enum(4)? as i32;
    let title = r.read_string()?;
    let greeting = if r.read_bool()? {
        Some(read_message(w, r)?)
    } else {
        None
    };
    let n = r.read_encoded_int()?.max(0);
    let mut messages = Vec::new();
    for _ in 0..n {
        messages.push(read_message(w, r)?);
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BbData {
            level,
            title,
            greeting,
            messages,
        });
    }
    Ok(())
}

item_leaf_raw! {
    /// `Server.Items.BasePlayerBB` `[SerializationGenerator(0, false)]` (abstract).
    BASE_PLAYER_BB = "Server.Items.BasePlayerBB" : ITEM_BASE;
    construct: None,
    get_context_menu_entries: |w, id, from, list| {
        base::get_context_menu_entries_base(w, id, from, list);
        if let Some(f) = muo_content::items::common::hooks(w).and_then(|h| h.secure_level_entry)
            && let Some(e) = f(w, from, id) {
                list.push(e);
            }
    },
    on_after_duped: |w, id, new_item| {
        let _ = id;
        if !is_player_bb(w, new_item) {
            return;
        }
        let d = bb(w, id);
        let now_board = new_item;
        if let Some(g) = d.greeting
            && let Some(nd) = data_mut(w, now_board) {
                nd.greeting = Some(g);
            }
        for m in d.messages {
            if let Some(nd) = data_mut(w, now_board) {
                nd.messages.push(m);
            }
        }
    },
    on_double_click: bb_double_click,
    serialize: |w, id, out| serialize_bb(w, id, out),
    deserialize: |w, id, r| deserialize_bb(w, id, r),
}

item_leaf_raw! {
    /// `Server.Items.PlayerBBSouth`.
    PLAYER_BB_SOUTH = "Server.Items.PlayerBBSouth" : BASE_PLAYER_BB;
    construct: Some(|w| player_bb_new(w, &PLAYER_BB_SOUTH, 0x2311)),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // bulletin board (south)
    serialize: |w, id, out| {
        serialize_bb(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        deserialize_bb(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
}

item_leaf_raw! {
    /// `Server.Items.PlayerBBEast`.
    PLAYER_BB_EAST = "Server.Items.PlayerBBEast" : BASE_PLAYER_BB;
    construct: Some(|w| player_bb_new(w, &PLAYER_BB_EAST, 0x2312)),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // bulletin board (east)
    serialize: |w, id, out| {
        serialize_bb(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        deserialize_bb(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
}

fn configure(w: &mut World) {
    let d = muo_content::items::house_deps::deps_mut(w);
    d.securables
        .push(|w, id| is_player_bb(w, id).then(|| bb(w, id).level as u8));
    d.securable_setters.push(|w, id, level| {
        if !is_player_bb(w, id) {
            return false;
        }
        if let Some(d) = data_mut(w, id) {
            d.level = i32::from(level);
        }
        true
    });
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in [&BASE_PLAYER_BB, &PLAYER_BB_SOUTH, &PLAYER_BB_EAST] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.BasePlayerBB.Configure",
        configure,
    );
}
