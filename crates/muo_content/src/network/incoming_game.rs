//! The incoming message, player, movement, entity, item, mobile and targeting packets: what a client in the world
//! sends.
//!
//! The systems some handlers call that live in other crates (spellbooks, doors, virtues, guilds, quests, party, the
//! vendors, the help gump) are `fn` fields of [`GameHooks`]; an unset one does nothing.

use muo_io::{SeekOrigin, SpanReader};
use muo_net::net_state::NetState;
use muo_primitives::MessageType;
use muo_primitives::{Point3D, Serial, parse, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::net_state::NetStateId;
use muo_server::skills::{SkillLock, SkillName};
use muo_server::target::{LandTarget, TargetCancelType, Targeted};
use muo_server::vendor::{BuyItemResponse, SellItemResponse};
use muo_server::{mobile, packets};

use super::register_packet as reg;

pub(super) fn begin(w: &mut World, ns: &NetState) -> (NetStateId, Option<MobileId>) {
    let id = muo_server::net_glue::ensure_net_state(w, ns);
    (id, w.net_state(id).and_then(|d| d.mobile))
}

pub fn configure(w: &mut World) {
    // IncomingMessagePackets
    reg(w, 0x03, 0, true, false, ascii_speech);
    reg(w, 0xAD, 0, true, false, unicode_speech);
    // IncomingPlayerPackets
    reg(w, 0x01, 5, false, false, disconnect);
    reg(w, 0x05, 5, true, false, attack_req);
    reg(w, 0x12, 0, true, false, text_command);
    reg(w, 0x22, 3, true, false, resynchronize);
    reg(w, 0x2C, 2, true, false, |_, _, _| {}); // DeathStatusResponse: ignored
    reg(w, 0x34, 10, true, false, mobile_query);
    reg(w, 0x3A, 0, true, false, change_skill_lock);
    reg(w, 0x72, 5, true, false, set_war_mode);
    reg(w, 0x73, 2, false, false, ping_req);
    reg(w, 0x7D, 13, true, false, menu_response);
    reg(w, 0x95, 9, true, false, hue_picker_response);
    reg(w, 0x9A, 0, true, false, ascii_prompt_response);
    reg(w, 0x9B, 258, true, false, help_request);
    reg(w, 0xA4, 149, false, false, |_, _, _| {}); // SystemInfo: read and dropped
    reg(w, 0xA7, 4, true, false, |_, _, _| {}); // RequestScrollWindow
    reg(w, 0xC2, 0, true, false, unicode_prompt_response);
    reg(w, 0xC8, 2, true, false, set_update_range);
    reg(w, 0xD0, 0, true, false, |_, _, _| {}); // ConfigurationFile
    reg(w, 0xD1, 2, true, false, logout_req);
    reg(w, 0xD7, 0, true, false, encoded_command);
    reg(w, 0xF4, 0, false, false, |_, _, _| {}); // CrashReport: read and dropped
    register_encoded_commands(w);
    // IncomingMovementPackets
    reg(w, 0x02, 7, true, false, movement_req);
    // IncomingEntityPackets
    reg(w, 0x06, 5, true, false, use_req);
    reg(w, 0x09, 5, true, false, look_req);
    reg(w, 0xB6, 9, true, false, object_help_request);
    reg(w, 0xD6, 0, true, false, batch_query_properties);
    // IncomingItemPackets
    reg(w, 0x07, 7, true, false, lift_req);
    super::register_packet_length_fn(w, 0x08, 14, true, false, drop_req, |ns| {
        14 + i32::from(ns.container_grid_lines())
    });
    reg(w, 0x13, 10, true, false, equip_req);
    reg(w, 0xEC, 0, false, false, equip_macro);
    reg(w, 0xED, 0, false, false, unequip_macro);
    // IncomingMobilePackets
    reg(w, 0x75, 35, true, false, rename_request);
    reg(w, 0x98, 0, true, false, mobile_name_request);
    reg(w, 0xB8, 0, true, false, profile_req);
    reg(w, 0x6F, 0, true, false, secure_trade);
    // IncomingVendorPackets
    reg(w, 0x3B, 0, true, false, vendor_buy_reply);
    reg(w, 0x9F, 0, true, false, vendor_sell_reply);
    // IncomingHousePackets: ShowPublicHouseContent reads a bool and does nothing
    reg(w, 0xFB, 2, false, false, |_, _, _| {});
    // UltimaStorePackets
    reg(w, 0xFA, 1, true, false, ultima_store_open_request);
    // IncomingTargetingPackets
    reg(w, 0x6C, 19, true, false, target_response);
}

/// `UltimaStorePackets.UltimaStoreOpenRequest`.
fn ultima_store_open_request(w: &mut World, ns: &mut NetState, _body: &[u8]) {
    if let Some(from) = begin(w, ns).1 {
        mobile::part2::messages::send_message(w, from, "Ultima Store is not currently available.");
    }
}

// ---- IncomingMessagePackets ----

fn message_type(b: u8) -> MessageType {
    MessageType::from_bits_retain(i32::from(b))
}

/// `Enum.IsDefined(typeof(MessageType), type)` for the values the enum names.
fn is_defined(t: u8) -> bool {
    matches!(
        t,
        0x00 | 0x01 | 0x02 | 0x06 | 0x07 | 0x08 | 0x09 | 0x0A | 0x0D | 0x0E | 0x0F | 0xC0
    )
}

fn ascii_speech(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let (Ok(t), Ok(hue), Ok(_font)) = (r.read_byte(), r.read_i16(), r.read_i16()) else {
        return;
    };
    let Ok(text) = r.read_latin1_safe(None) else {
        return;
    };
    let text = text.trim();
    let n = text.chars().count();
    if n == 0 || n > 128 {
        return;
    }
    let t = if is_defined(t) {
        message_type(t)
    } else {
        MessageType::REGULAR
    };
    mobile::base::do_speech(
        w,
        from,
        text,
        &[],
        t,
        utility::clip_dyed_hue(i32::from(hue)),
    );
}

fn unicode_speech(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let (Ok(t), Ok(hue), Ok(_font)) = (r.read_byte(), r.read_i16(), r.read_i16()) else {
        return;
    };
    let Ok(lang) = r.read_ascii(Some(4)) else {
        return;
    };
    let encoded = t & 0xC0 != 0;
    let mut keywords: Vec<i32> = Vec::new();
    let text = if encoded {
        let Ok(value) = r.read_i16() else { return };
        let count = (i32::from(value) & 0xFFF0) >> 4;
        let mut hold = i32::from(value) & 0xF;
        if !(0..=50).contains(&count) {
            return;
        }
        for i in 0..count {
            let speech_id;
            if i & 1 == 0 {
                hold <<= 8;
                let Ok(b) = r.read_byte() else { return };
                hold |= i32::from(b);
                speech_id = hold;
                hold = 0;
            } else {
                let Ok(v) = r.read_i16() else { return };
                speech_id = (i32::from(v) & 0xFFF0) >> 4;
                hold = i32::from(v) & 0xF;
            }
            if !keywords.contains(&speech_id) {
                keywords.push(speech_id);
            }
        }
        let Ok(t) = r.read_utf8_safe(None) else {
            return;
        };
        t
    } else {
        let Ok(t) = r.read_big_uni_safe(None) else {
            return;
        };
        t
    };
    let text = text.trim();
    let n = text.chars().count();
    if n == 0 || n > 128 {
        return;
    }
    let t = t & !0xC0;
    let t = if is_defined(t) {
        message_type(t)
    } else {
        MessageType::REGULAR
    };
    if let Some(m) = w.mobile_mut(from) {
        m.language = Some(lang);
    }
    mobile::base::do_speech(
        w,
        from,
        text,
        &keywords,
        t,
        utility::clip_dyed_hue(i32::from(hue)),
    );
}

// ---- IncomingPlayerPackets ----

fn disconnect(_w: &mut World, _ns: &mut NetState, _body: &[u8]) {
    // reads the -1 and does nothing
}

fn attack_req(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let Ok(serial) = SpanReader::new(body).read_u32() else {
        return;
    };
    if let Some(m) = w.find_mobile(Serial::new(serial)) {
        mobile::base::attack(w, from, m);
    }
}

fn text_command(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let Ok(kind) = r.read_byte() else { return };
    let Ok(command) = r.read_ascii(None) else {
        return;
    };
    let mut tokens = command.split(' ');
    match kind {
        0xC7 => crate::misc::small::animate_request(w, from, &command),
        0x24 => {
            if let Some(Ok(skill)) = tokens.next().map(|t| t.trim().parse::<i32>()) {
                muo_server::skills::use_skill(w, from, skill);
            }
        }
        0x43 => {
            let book = command.trim().parse::<i32>().unwrap_or(1);
            if let Some(f) = w
                .resources
                .get::<GameHooks>()
                .and_then(|h| h.open_spellbook_request)
            {
                f(w, from, book);
            }
        }
        0x27 => {
            let spell = tokens.next().map_or(0, parse::to_i32) - 1;
            let serial = tokens
                .next()
                .map_or(Serial::MINUS_ONE, |t| Serial::new(parse::to_u32(t)));
            let book = w.find_item(serial);
            if let Some(f) = w
                .resources
                .get::<GameHooks>()
                .and_then(|h| h.cast_spell_request)
            {
                f(w, from, spell, book);
            }
        }
        0x58 => {
            if let Some(f) = w
                .resources
                .get::<GameHooks>()
                .and_then(|h| h.open_door_macro)
            {
                f(w, from);
            }
        }
        0x56 => {
            let spell = parse::to_i32(&command) - 1;
            if let Some(f) = w
                .resources
                .get::<GameHooks>()
                .and_then(|h| h.cast_spell_request)
            {
                f(w, from, spell, None);
            }
        }
        0xF4 => {
            let virtue = parse::to_i32(&command) - 1;
            if let Some(f) = w.resources.get::<GameHooks>().and_then(|h| h.virtue_macro) {
                f(w, from, virtue);
            }
        }
        0x2F => {} // Old scroll double click
        _ => ns.log_info(&format!(
            "Unknown text-command type 0x{kind:02X}: {kind} ({command})"
        )),
    }
}

fn prompt_response(w: &mut World, from: MobileId, serial: u32, prompt: i32, kind: i32, text: &str) {
    use muo_server::mobile::part1::props_b::set_prompt;
    let Some(p) = w.mobile(from).and_then(|m| m.prompt) else {
        return;
    };
    // `p.Serial == prompt` compares a uint with an int as longs: a negative prompt never matches.
    if p.0 == serial && i64::from(p.0) == i64::from(prompt) {
        set_prompt(w, from, None);
        if kind == 0 {
            muo_server::prompt::on_cancel(w, p, from);
        } else {
            muo_server::prompt::on_response(w, p, from, text);
        }
    }
}

fn ascii_prompt_response(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let (Ok(serial), Ok(prompt), Ok(kind)) = (r.read_u32(), r.read_i32(), r.read_i32()) else {
        return;
    };
    let Ok(text) = r.read_latin1_safe(None) else {
        return;
    };
    if text.chars().count() > 128 {
        return;
    }
    prompt_response(w, from, serial, prompt, kind, &text);
}

fn unicode_prompt_response(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let (Ok(serial), Ok(prompt), Ok(kind)) = (r.read_u32(), r.read_i32(), r.read_i32()) else {
        return;
    };
    let (Ok(_lang), Ok(text)) = (r.read_ascii(Some(4)), r.read_little_uni_safe(None)) else {
        return;
    };
    if text.chars().count() > 128 {
        return;
    }
    prompt_response(w, from, serial, prompt, kind, &text);
}

fn menu_response(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (id, _) = begin(w, ns);
    let mut r = SpanReader::new(body);
    let (Ok(serial), Ok(_menu_id), Ok(index)) = (r.read_u32(), r.read_i16(), r.read_i16()) else {
        return;
    };
    let index = i32::from(index) - 1; // convert from 1-based to 0-based
    let Some(at) = w
        .net_state(id)
        .and_then(|d| d.menus.iter().position(|m| m.serial as u32 == serial))
    else {
        return;
    };
    let Some(menu) = muo_server::menus::remove_menu_at(w, id, at) else {
        return;
    };
    let entry_length = match &menu.kind {
        muo_server::menus::MenuKind::ItemList { entries, .. } => entries.len() as i32,
        muo_server::menus::MenuKind::Question { answers, .. } => answers.len() as i32,
    };
    if index >= 0 && index < entry_length {
        if let Some(f) = menu.on_response.clone() {
            f(w, id, index);
        }
    } else if let Some(f) = menu.on_cancel.clone() {
        f(w, id);
    }
}

fn hue_picker_response(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (id, _) = begin(w, ns);
    let mut r = SpanReader::new(body);
    let (Ok(serial), Ok(_item), Ok(hue)) = (r.read_u32(), r.read_i16(), r.read_i16()) else {
        return;
    };
    let hue = utility::clip_dyed_hue(i32::from(hue) & 0x3FFF);
    let at = w.net_state(id).and_then(|d| {
        d.hue_pickers
            .iter()
            .position(|p| p.serial.value() == serial)
    });
    if let Some(at) = at
        && let Some(p) = muo_server::hue_picker::remove_hue_picker_at(w, id, at)
    {
        p.respond(w, hue);
    }
}

fn logout_req(w: &mut World, ns: &mut NetState, _body: &[u8]) {
    let (id, _) = begin(w, ns);
    packets::player::send_logout_ack(w, id);
}

fn change_skill_lock(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let (Ok(skill), Ok(lock)) = (r.read_i16(), r.read_byte()) else {
        return;
    };
    // `Skills[index]` is null for an index outside the table
    if let (Some(name), Some(lock)) = (
        SkillName::from_i32(i32::from(skill)),
        SkillLock::from_u8(lock),
    ) {
        muo_server::skills::set_lock_no_relay(w, from, name, lock);
    }
}

fn help_request(w: &mut World, ns: &mut NetState, _body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    if let Some(f) = w.resources.get::<GameHooks>().and_then(|h| h.help_request) {
        f(w, from);
    }
}

pub type CastSpellFn = fn(&mut World, MobileId, i32, Option<ItemId>);
pub type TargetedSpellFn = fn(&mut World, MobileId, Option<EntityId>, i16);
pub type VendorBuyFn = fn(&mut World, MobileId, MobileId, &[BuyItemResponse]) -> bool;
pub type VendorSellFn = fn(&mut World, MobileId, MobileId, &[SellItemResponse]) -> bool;

/// Entry points other crates fill in; unset is "that system is absent".
#[derive(Default)]
pub struct GameHooks {
    /// `(vendor as IVendor).OnBuyItems(from, buyList)`: true when the vendor took the list. Unset: the mobile is
    /// not an `IVendor`.
    pub vendor_buy_items: Option<VendorBuyFn>,
    /// `(vendor as IVendor).OnSellItems(from, sellList)`.
    pub vendor_sell_items: Option<VendorSellFn>,
    /// `HelpGump.HelpRequest(from)`.
    pub help_request: Option<fn(&mut World, MobileId)>,
    /// `Spellbook.OpenSpellbookRequest(from, bookType)`.
    pub open_spellbook_request: Option<fn(&mut World, MobileId, i32)>,
    /// `Spellbook.CastSpellRequest(from, spellId, spellbook)`.
    pub cast_spell_request: Option<CastSpellFn>,
    /// `Spellbook.TargetedSpell(from, target, spellId)`.
    pub targeted_spell: Option<TargetedSpellFn>,
    /// `BaseDoor.OpenDoorMacroUsed(from)`.
    pub open_door_macro: Option<fn(&mut World, MobileId)>,
    /// `VirtueGump.RequestVirtueMacro(pm, virtueId)`.
    pub virtue_macro: Option<fn(&mut World, MobileId, i32)>,
    /// `Guild.GuildGumpRequest(from)` and `MLQuestSystem.QuestGumpRequest(from)`.
    pub guild_gump_request: Option<fn(&mut World, MobileId)>,
    pub quest_gump_request: Option<fn(&mut World, MobileId)>,
    /// `Fists.StunRequest(from)` / `Fists.DisarmRequest(from)`.
    pub fists_stun_request: Option<fn(&mut World, MobileId)>,
    pub fists_disarm_request: Option<fn(&mut World, MobileId)>,
    /// `Bandage.BandageTargetRequest(from, bandage, target)`.
    pub bandage_target_request: Option<fn(&mut World, MobileId, ItemId, MobileId)>,
    /// `PartyCommands.Handler`.
    pub party: Option<PartyHandler>,
}

/// `PartyCommands`' handler methods the 0xBF/0x06 packet calls.
#[derive(Copy, Clone)]
pub struct PartyHandler {
    pub on_add: fn(&mut World, MobileId),
    pub on_remove: fn(&mut World, MobileId, Option<MobileId>),
    pub on_private_message: fn(&mut World, MobileId, Option<MobileId>, &str),
    pub on_public_message: fn(&mut World, MobileId, &str),
    pub on_set_can_loot: fn(&mut World, MobileId, bool),
    pub on_accept: fn(&mut World, MobileId, Option<MobileId>),
    pub on_decline: fn(&mut World, MobileId, Option<MobileId>),
}

fn set_war_mode(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let (Some(from), Ok(on)) = (from, SpanReader::new(body).read_boolean()) else {
        return;
    };
    mobile::part1::combat::delay_change_warmode(w, from, on);
}

/// TODO: Throttle/make this more safe
fn resynchronize(w: &mut World, ns: &mut NetState, _body: &[u8]) {
    let (id, from) = begin(w, ns);
    let Some(from) = from else { return };
    packets::mobile::send_mobile_update(w, id, from);
    packets::mobile::send_mobile_incoming(w, id, from, from);
    mobile::part2::effects::send_everything(w, from);
    muo_server::net_glue::reset_movement_state(w, id);
}

fn ping_req(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (id, _) = begin(w, ns);
    if let Ok(ping) = SpanReader::new(body).read_byte() {
        packets::player::send_ping_ack(w, id, ping);
    }
}

fn set_update_range(w: &mut World, ns: &mut NetState, _body: &[u8]) {
    let (id, _) = begin(w, ns);
    packets::player::send_change_update_range(w, id, 18);
}

fn mobile_query(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let (Ok(_), Ok(kind), Ok(serial)) = (r.read_i32(), r.read_byte(), r.read_u32()) else {
        return;
    };
    let Some(m) = w.find_mobile(Serial::new(serial)) else {
        return;
    };
    match kind {
        0x04 => mobile::base::on_stats_query(w, m, from),
        0x05 => mobile::base::on_skills_query(w, m, from),
        _ => ns.trace(body),
    }
}

fn encoded_command(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let mut r = SpanReader::new(body);
    let (Ok(serial), Ok(packet_id)) = (r.read_u32(), r.read_u16()) else {
        return;
    };
    // We will add support if this is ever a real thing.
    if packet_id > 0xFF {
        let reason = format!("Sent unsupported encoded packet (0xD7x{packet_id:04X}");
        ns.log_info(&reason);
        ns.disconnect(&reason);
        ns.trace(body);
        return;
    }
    let handler = w
        .resources
        .get::<super::IncomingRegistrations>()
        .and_then(|r| {
            r.0.iter().rev().find_map(|r| match r {
                super::Registration::Encoded {
                    id,
                    in_game,
                    on_receive,
                } if u16::from(*id) == packet_id => Some((*in_game, *on_receive)),
                _ => None,
            })
        });
    let Some((in_game, on_receive)) = handler else {
        ns.trace(body);
        return;
    };
    if in_game {
        match ns.attachment {
            muo_net::net_state::Attachment::None => {
                let reason = format!(
                    "Sent in-game packet (0xD7x{packet_id:04X}) before being attached to a mobile."
                );
                ns.log_info(&reason);
                ns.disconnect(&reason);
                return;
            }
            muo_net::net_state::Attachment::Deleted => {
                ns.disconnect(&format!(
                    "Sent in-game packet(0xD7x{packet_id:04X}) but mobile is deleted."
                ));
                return;
            }
            muo_net::net_state::Attachment::Mobile(_) => {}
        }
    }
    on_receive(w, ns, serial, &body[6..]);
}

fn guild_gump_request(w: &mut World, ns: &mut NetState, _serial: u32, _body: &[u8]) {
    let (_, from) = begin(w, ns);
    if let (Some(from), Some(f)) = (
        from,
        w.resources
            .get::<GameHooks>()
            .and_then(|h| h.guild_gump_request),
    ) {
        f(w, from);
    }
}

fn quest_gump_request(w: &mut World, ns: &mut NetState, _serial: u32, _body: &[u8]) {
    let (_, from) = begin(w, ns);
    if let (Some(from), Some(f)) = (
        from,
        w.resources
            .get::<GameHooks>()
            .and_then(|h| h.quest_gump_request),
    ) {
        f(w, from);
    }
}

pub(super) fn register_encoded_commands(w: &mut World) {
    super::register_encoded(w, 0x28, true, guild_gump_request);
    super::register_encoded(w, 0x32, true, quest_gump_request);
}

// ---- IncomingMovementPackets ----

fn movement_req(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (id, from) = begin(w, ns);
    if from.is_none() {
        return;
    }
    let mut r = SpanReader::new(body);
    let (Ok(dir), Ok(seq), Ok(_key)) = (r.read_byte(), r.read_byte(), r.read_u32()) else {
        return;
    };
    // Delegate to MovementThrottle which has full context for timing validation
    muo_server::net_glue::validate_and_queue_movement(w, id, dir, i32::from(seq));
}

// ---- IncomingEntityPackets ----

/// `IncomingEntityPackets.SingleClickProps`.
#[derive(Default)]
pub struct SingleClickProps(pub bool);

fn use_req(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let Ok(value) = SpanReader::new(body).read_u32() else {
        return;
    };
    let (access, next, tick) = {
        let m = w.mobile(from);
        (
            m.map(|m| m.access_level),
            m.map_or(0, |m| m.next_action_time),
            w.tick_count,
        )
    };
    if access >= Some(mobile::AccessLevel::Counselor) || tick.wrapping_sub(next) >= 0 {
        if value & !0x7FFF_FFFF != 0 {
            mobile::base::on_paperdoll_request(w, from);
        } else {
            let s = Serial::new(value);
            if s.is_mobile() {
                if let Some(m) = w
                    .find_mobile(s)
                    .filter(|m| w.mobile(*m).is_some_and(|mm| !mm.deleted))
                {
                    mobile::base::use_mobile(w, from, m);
                }
            } else if s.is_item()
                && let Some(i) = w
                    .find_item(s)
                    .filter(|i| w.item(*i).is_some_and(|ii| !ii.deleted()))
            {
                mobile::base::use_item(w, from, i);
            }
        }
        let delay = i64::from(mobile::part1::props_a::action_delay(w));
        if let Some(m) = w.mobile_mut(from) {
            m.next_action_time = tick + delay;
        }
    } else {
        mobile::base::send_action_message(w, from);
    }
}

fn look_req(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let Ok(v) = SpanReader::new(body).read_u32() else {
        return;
    };
    let s = Serial::new(v);
    let single_click_props = w.resources.get::<SingleClickProps>().is_some_and(|p| p.0);
    let from_loc = w.mobile(from).map(|m| m.location).unwrap_or_default();
    if s.is_mobile() {
        if let Some(m) = w.find_mobile(s)
            && mobile::base::can_see_mobile(w, from, m)
            && mobile::part1::combat::in_update_range(
                from_loc,
                w.mobile(m).map(|mm| mm.location).unwrap_or_default(),
            )
        {
            if single_click_props {
                mobile::base::on_aos_single_click(w, m, from);
            } else {
                if region_on_single_click(w, from, EntityId::Mobile(m))
                    && let Some(ty) = w.mobile(m).map(|mm| mm.ty)
                {
                    (ty.on_single_click)(w, m, from);
                }
            }
        }
    } else if s.is_item()
        && let Some(i) = w
            .find_item(s)
            .filter(|i| w.item(*i).is_some_and(|ii| !ii.deleted()))
        && mobile::base::can_see_item(w, from, i)
        && mobile::part1::combat::in_update_range(
            from_loc,
            muo_server::item::ops::get_world_location(w, i),
        )
    {
        if single_click_props {
            muo_server::item::base::on_aos_single_click(w, i, from);
        } else {
            if region_on_single_click(w, from, EntityId::Item(i)) {
                if let Some(muo_server::ids::EntityId::Item(parent)) =
                    w.item(i).and_then(|ii| ii.parent)
                {
                    muo_server::item::base::on_single_click_contained(w, parent, from, i);
                }
                muo_server::item::base::on_single_click(w, i, from);
            }
        }
    }
}

/// `from.Region.OnSingleClick(from, o)`.
fn region_on_single_click(w: &mut World, from: MobileId, o: EntityId) -> bool {
    let region = w.mobile(from).map(|m| m.region);
    match region.and_then(|r| w.region(r).map(|reg| (r, reg.ty))) {
        Some((r, ty)) => (ty.on_single_click)(w, r, from, &Targeted::Entity(o)),
        None => true,
    }
}

fn object_help_request(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let (Ok(v), Ok(_unk), Ok(_lang)) = (r.read_u32(), r.read_byte(), r.read_ascii(Some(3))) else {
        return;
    };
    let s = Serial::new(v);
    let (fmap, floc) = w
        .mobile(from)
        .map(|m| (m.map, m.location))
        .unwrap_or_default();
    if s.is_item() {
        if let Some(i) = w.find_item(s)
            && w.item(i).is_some_and(|ii| ii.map == fmap)
            && mobile::part1::combat::in_update_range(
                muo_server::item::ops::get_world_location(w, i),
                floc,
            )
            && mobile::base::can_see_item(w, from, i)
        {
            muo_server::item::base::on_help_request(w, i, from);
        }
    } else if s.is_mobile()
        && let Some(m) = w.find_mobile(s)
        && w.mobile(m).is_some_and(|mm| mm.map == fmap)
        && mobile::part1::combat::in_update_range(
            w.mobile(m).map(|mm| mm.location).unwrap_or_default(),
            floc,
        )
        && mobile::base::can_see_mobile(w, from, m)
    {
        // calls `m.OnHelpRequest(m)` (the target, not the asker): kept
        mobile::base::on_help_request(w, m, m);
    }
}

fn batch_query_properties(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (id, from) = begin(w, ns);
    let Some(from) = from else { return };
    if !w.core.opl_enabled || !body.len().is_multiple_of(4) {
        return;
    }
    let floc = w.mobile(from).map(|m| m.location).unwrap_or_default();
    let mut r = SpanReader::new(body);
    while r.remaining() > 0 {
        let Ok(v) = r.read_u32() else { return };
        let s = Serial::new(v);
        if s.is_mobile() {
            if let Some(m) = w.find_mobile(s)
                && mobile::base::can_see_mobile(w, from, m)
                && mobile::part1::combat::in_update_range(
                    floc,
                    w.mobile(m).map(|mm| mm.location).unwrap_or_default(),
                )
            {
                mobile::base::send_properties_to(w, m, id);
            }
        } else if s.is_item()
            && let Some(i) = w
                .find_item(s)
                .filter(|i| w.item(*i).is_some_and(|ii| !ii.deleted()))
            && mobile::base::can_see_item(w, from, i)
            && mobile::part1::combat::in_update_range(
                floc,
                muo_server::item::ops::get_world_location(w, i),
            )
        {
            muo_server::item::base::send_properties_to(w, i, id);
        }
    }
}

// ---- IncomingItemPackets ----

fn lift_req(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let (Ok(serial), Ok(amount)) = (r.read_u32(), r.read_u16()) else {
        return;
    };
    if let Some(item) = w.find_item(Serial::new(serial)) {
        mobile::base::lift(w, from, item, i32::from(amount));
    }
}

fn equip_req(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let item = w.mobile(from).and_then(|m| m.holding);
    let valid = item.is_some_and(|i| {
        w.item(i).is_some_and(|ii| {
            ii.held_by == Some(from) && ii.map == Some(muo_server::map::MapId::INTERNAL)
        })
    });
    mobile::part1::props_a::set_holding(w, from, None);
    let Some(item) = item.filter(|_| valid) else {
        return;
    };

    let mut r = SpanReader::new(body);
    let to = r
        .seek(5, SeekOrigin::Current)
        .ok()
        .and_then(|_| r.read_u32().ok())
        .and_then(|s| w.find_mobile(Serial::new(s)))
        .unwrap_or(from);
    if !mobile::base::allow_equip_from(w, to, from) || !mobile::base::equip_item(w, to, item) {
        muo_server::item::ops::bounce(w, item, from);
    }
    muo_server::item::ops::clear_bounce(w, item);
}

fn drop_req(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let Some(from) = from else { return };
    let grid = ns.container_grid_lines();
    let mut r = SpanReader::new(body);
    let read = (|| {
        r.read_i32().ok()?; // serial, ignored
        let x = i32::from(r.read_i16().ok()?);
        let y = i32::from(r.read_i16().ok()?);
        let z = i32::from(r.read_sbyte().ok()?);
        if grid {
            r.read_byte().ok()?; // Grid Location?
        }
        Some((Point3D::new(x, y, z), r.read_u32().ok()?))
    })();
    let Some((loc, dest)) = read else { return };
    let dest = Serial::new(dest);
    if dest.is_mobile() {
        match w.find_mobile(dest) {
            Some(m) => mobile::base::drop_mobile(w, from, m, loc),
            None => mobile::base::drop_missing(w, from),
        };
    } else if dest.is_item() {
        if let Some(i) = w.find_item(dest) {
            let multi = w.item(i).and_then(|it| {
                (it.ty.is(&muo_server::item::multi::BASE_MULTI)
                    && (it.ty.allows_relative_drop)(w, i))
                .then_some(it.location)
            });
            if let Some(at) = multi {
                let loc = Point3D::new(loc.x + at.x, loc.y + at.y, loc.z);
                mobile::base::drop_world(w, from, loc);
            } else {
                mobile::base::drop_item(w, from, i, loc);
            }
        } else {
            mobile::base::drop_missing(w, from);
        }
    } else {
        mobile::base::drop_world(w, from, loc);
    }
}

// ---- IncomingMobilePackets ----

fn mobile_name_request(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (id, from) = begin(w, ns);
    let Some(from) = from else { return };
    let Ok(v) = SpanReader::new(body).read_u32() else {
        return;
    };
    if let Some(m) = w.find_mobile(Serial::new(v)) {
        let (floc, mloc) = (
            w.mobile(from).map(|mm| mm.location).unwrap_or_default(),
            w.mobile(m).map(|mm| mm.location).unwrap_or_default(),
        );
        if mobile::part1::combat::in_update_range(floc, mloc)
            && mobile::base::can_see_mobile(w, from, m)
        {
            packets::mobile::send_mobile_name(w, id, m);
        }
    }
}

fn profile_req(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, beholder) = begin(w, ns);
    let Some(beholder) = beholder else { return };
    let mut r = SpanReader::new(body);
    let (Ok(kind), Ok(serial)) = (r.read_byte(), r.read_u32()) else {
        return;
    };
    let Some(beheld) = w.find_mobile(Serial::new(serial)) else {
        return;
    };
    match kind {
        0x00 => crate::login::small::profile_request(w, beholder, beheld),
        0x01 => {
            let (Ok(_), Ok(length)) = (r.read_i16(), r.read_u16()) else {
                return;
            };
            if length > 511 {
                return;
            }
            if let Ok(text) = r.read_big_uni(Some(usize::from(length))) {
                crate::login::small::change_profile_request(w, beholder, beheld, &text);
            }
        }
        _ => {}
    }
}

// ---- IncomingTargetingPackets ----

fn target_response(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let mut r = SpanReader::new(body);
    let read = (|| {
        let kind = r.read_byte().ok()?;
        let target_id = r.read_i32().ok()?;
        let _flags = r.read_byte().ok()?;
        let serial = r.read_u32().ok()?;
        let x = i32::from(r.read_i16().ok()?);
        let y = i32::from(r.read_i16().ok()?);
        r.read_byte().ok()?;
        let z = i32::from(r.read_sbyte().ok()?);
        let graphic = i32::from(r.read_u16().ok()?);
        Some((kind, target_id, serial, x, y, z, graphic))
    })();
    let Some((kind, target_id, serial, x, y, z, graphic)) = read else {
        return;
    };

    if target_id == 0xDEAD_BEEFu32 as i32 {
        return;
    }
    let Some(from) = from else { return };
    let Some(t) = w.mobile(from).and_then(|m| m.target) else {
        return;
    };
    let serial = Serial::new(serial);

    if x == -1 && y == -1 && !serial.is_valid() {
        // User pressed escape
        muo_server::target::cancel(w, t, from, TargetCancelType::Canceled);
        return;
    }
    if t.0 as i32 != target_id {
        return;
    }

    let targeted = if kind == 1 {
        if graphic == 0 {
            // `new LandTarget(new Point3D(x, y, z), from.Map)`: z and the tile come from the map
            let mut location = Point3D::new(x, y, z);
            let mut tile_id = 0;
            if let Some(map) = w.mobile(from).and_then(|m| m.map) {
                location.z = w.maps.get_average_z(map, x, y);
                tile_id = i32::from(w.maps.land_tile(map, x, y).id)
                    & muo_data::TileData::MAX_LAND_VALUE as i32;
            }
            Targeted::Land(LandTarget { location, tile_id })
        } else {
            let map = w.mobile(from).and_then(|m| m.map);
            if map.is_none() || map == Some(muo_server::map::MapId::INTERNAL) {
                muo_server::target::cancel(w, t, from, TargetCancelType::Canceled);
                return;
            }
            let Some(map) = map else { return };
            let mut z = z;
            if ns.high_seas() {
                let td = w
                    .maps
                    .tile_data
                    .item(graphic as usize & muo_data::TileData::MAX_ITEM_VALUE);
                if td.surface() {
                    z -= i32::from(td.height);
                }
            }
            let disallow_multis = w.targets.get(&t).is_some_and(|t| t.disallow_multis);
            let tiles = if disallow_multis {
                muo_server::map::tiles::static_tiles(w, map, x, y)
            } else {
                muo_server::map::tiles::static_and_multi_tiles(w, map, x, y)
            };
            let Some(tile) = tiles
                .iter()
                .find(|tile| i32::from(tile.z) == z && i32::from(tile.id) == graphic)
            else {
                muo_server::target::cancel(w, t, from, TargetCancelType::Canceled);
                return;
            };
            let calc_height = w
                .maps
                .tile_data
                .item(graphic as usize & muo_data::TileData::MAX_ITEM_VALUE)
                .calc_height();
            Targeted::Static(muo_server::target::StaticTarget::new(
                Point3D::new(x, y, z),
                graphic & muo_data::TileData::MAX_ITEM_VALUE as i32,
                calc_height,
                i32::from(tile.hue),
            ))
        }
    } else if serial.is_mobile() {
        match w.find_mobile(serial) {
            Some(m) => Targeted::Entity(EntityId::Mobile(m)),
            None => return muo_server::target::invoke_null(w, t, from),
        }
    } else if serial.is_item() {
        match w.find_item(serial) {
            Some(i) => Targeted::Entity(EntityId::Item(i)),
            None => return muo_server::target::invoke_null(w, t, from),
        }
    } else {
        muo_server::target::cancel(w, t, from, TargetCancelType::Canceled);
        return;
    };
    muo_server::target::invoke(w, t, from, targeted);
}

// ---- IncomingItemPackets: macros ----

fn equip_macro(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let mut r = SpanReader::new(body);
    let Ok(count) = r.read_byte() else { return };
    let mut list = Vec::with_capacity(usize::from(count));
    for _ in 0..count {
        let Ok(s) = r.read_u32() else { return };
        list.push(Serial::new(s));
    }
    if let Some(from) = from {
        crate::player_mobile::members::equip_macro(w, from, &list);
    }
}

fn unequip_macro(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let mut r = SpanReader::new(body);
    let Ok(count) = r.read_byte() else { return };
    let mut layers = Vec::with_capacity(usize::from(count));
    for _ in 0..count {
        let Ok(l) = r.read_u16() else { return };
        // `(Layer)reader.ReadUInt16()`: the unchecked cast to the byte enum keeps the low byte
        layers.push(muo_server::layer::Layer::from_raw(l as u8));
    }
    if let Some(from) = from {
        crate::player_mobile::members::unequip_macro(w, from, &layers);
    }
}

// ---- IncomingMobilePackets: RenameRequest, SecureTrade ----

/// `RenameRequests.RenameRequest(from, targ, name)`.
fn rename_request(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let mut r = SpanReader::new(body);
    let Ok(s) = r.read_u32() else { return };
    let Some(targ) = w.find_mobile(Serial::new(s)) else {
        return;
    };
    let Ok(name) = r.read_latin1_safe(None) else {
        return;
    };
    let Some(from) = from else { return };
    let can_see = w
        .mobile(from)
        .map(|m| m.ty)
        .is_some_and(|m| (m.can_see_mobile)(w, from, targ));
    if !can_see
        || !targ_in_range(w, from, targ, 12)
        || !w
            .mobile(targ)
            .is_some_and(|t| (t.ty.can_be_renamed_by)(w, targ, from))
    {
        return;
    }
    let span = name.trim();
    if crate::misc::name_verification::validate_pet_name(span) {
        // Pet ~1_OLDPETNAME~ renamed to ~2_NEWPETNAME~.
        let old = w
            .mobile(targ)
            .and_then(|m| (m.ty.name)(w, targ))
            .unwrap_or_default();
        mobile::part2::messages::send_localized_message(
            w,
            from,
            1072623,
            &format!("{old}	{span}"),
            0x3B2,
        );
        mobile::base::set_name(w, targ, Some(span.to_string()));
    } else if crate::misc::profanity_protection::disallowed_in(span) {
        mobile::part2::messages::send_localized_message(w, from, 1072622, "", 0x3B2); // That name isn't very polite.
    } else {
        mobile::part2::messages::send_message(w, from, "That name is unacceptable.");
    }
}

fn targ_in_range(w: &World, from: MobileId, targ: MobileId, range: i32) -> bool {
    w.mobile(targ)
        .is_some_and(|t| mobile::base::in_range_point3d(w, from, t.location, range))
}

/// `SecureTrade`.
fn secure_trade(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (_, from) = begin(w, ns);
    let mut r = SpanReader::new(body);
    let Ok(sub) = r.read_byte() else { return };
    if !(1..=3).contains(&sub) {
        return;
    }
    let Ok(s) = r.read_u32() else { return };
    let Some(cont) = w.find_item(Serial::new(s)) else {
        return;
    };
    if !w.item(cont).is_some_and(|i| {
        i.ty.is(&muo_server::item::secure_trade_container::SECURE_TRADE_CONTAINER)
    }) {
        return;
    }
    let trade = muo_server::item::secure_trade_container::trade_of(w, cont);
    let side = |w: &World, t| {
        muo_server::secure_trade::trade(w, t)
            .map(|t| (Some(t.from.mobile) == from, Some(t.to.mobile) == from))
    };
    use muo_server::secure_trade as st;
    match sub {
        1 => {
            // Cancel
            if let (Some(t), Some(_)) = (trade, from)
                && side(w, t).is_some_and(|(f, o)| f || o)
            {
                st::cancel(w, t);
            }
        }
        2 => {
            // Check
            let Ok(v) = r.read_i32() else { return };
            if let (Some(t), Some(m)) = (trade, from)
                && side(w, t).is_some_and(|(f, o)| f || o)
            {
                st::set_accepted(w, t, m, v != 0);
                st::update(w, t);
            }
        }
        _ => {
            // Update Gold
            let (Ok(gold), Ok(plat)) = (r.read_i32(), r.read_i32()) else {
                return;
            };
            if let (Some(t), Some(_)) = (trade, from) {
                match side(w, t) {
                    Some((true, _)) => {
                        set_gold(w, t, true, gold, plat);
                        st::update_from_currency(w, t);
                    }
                    Some((_, true)) => {
                        set_gold(w, t, false, gold, plat);
                        st::update_to_currency(w, t);
                    }
                    _ => {}
                }
                muo_server::item::secure_trade_container::clear_checks(w, cont);
            }
        }
    }
}

/// `SecureTradeInfo.Gold = gold; Plat = plat` for one side: the side's virtual check holds them.
fn set_gold(
    w: &mut World,
    t: muo_server::net_state::TradeId,
    from_side: bool,
    gold: i32,
    plat: i32,
) {
    let check = muo_server::secure_trade::trade(w, t).and_then(|t| {
        if from_side {
            t.from.virtual_check
        } else {
            t.to.virtual_check
        }
    });
    if let Some(c) = check {
        muo_server::item::virtual_check::set_gold(w, c, gold);
        muo_server::item::virtual_check::set_plat(w, c, plat);
    }
}

// ---- IncomingVendorPackets ----

fn vendor_buy_reply(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (id, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let Ok(serial) = r.read_u32() else { return };
    let Some(vendor) = w.find_mobile(Serial::new(serial)) else {
        return;
    };
    let Ok(flag) = r.read_byte() else { return };

    let (vloc, floc) = (
        w.mobile(vendor).map(|m| m.location).unwrap_or_default(),
        w.mobile(from).map(|m| m.location).unwrap_or_default(),
    );
    let deleted = w.mobile(vendor).is_none_or(|m| m.deleted);
    if !deleted && utility::in_range_3d(vloc, floc, 10) && flag == 0x02 {
        let mut msg_size = r.remaining();
        if msg_size / 7 > 100 {
            return;
        }
        let mut buy_list = Vec::with_capacity(msg_size / 7);
        while msg_size > 0 {
            let (Ok(_layer), Ok(serial), Ok(amount)) = (r.read_byte(), r.read_u32(), r.read_i16())
            else {
                return;
            };
            buy_list.push(BuyItemResponse::new(Serial::new(serial), i32::from(amount)));
            msg_size = msg_size.saturating_sub(7);
        }
        let taken = w
            .resources
            .get::<GameHooks>()
            .and_then(|h| h.vendor_buy_items)
            .is_some_and(|f| f(w, vendor, from, &buy_list));
        if buy_list.is_empty() || !taken {
            return;
        }
    }
    let vserial = w.mobile(vendor).map(|m| m.serial).unwrap_or_default();
    packets::vendor_buy::send_end_vendor_buy(w, id, vserial);
}

fn vendor_sell_reply(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let (id, from) = begin(w, ns);
    let Some(from) = from else { return };
    let mut r = SpanReader::new(body);
    let Ok(serial) = r.read_u32() else { return };
    let Some(vendor) = w.find_mobile(Serial::new(serial)) else {
        return;
    };
    let vserial = w.mobile(vendor).map(|m| m.serial).unwrap_or_default();

    let (vloc, floc) = (
        w.mobile(vendor).map(|m| m.location).unwrap_or_default(),
        w.mobile(from).map(|m| m.location).unwrap_or_default(),
    );
    if w.mobile(vendor).is_none_or(|m| m.deleted) || !utility::in_range_3d(vloc, floc, 10) {
        packets::vendor_sell::send_end_vendor_sell(w, id, vserial);
        return;
    }
    let Ok(count) = r.read_u16() else { return };
    let count = usize::from(count);
    if count >= 100 || r.remaining() != count * 6 {
        return;
    }
    let mut sell_list = Vec::with_capacity(count);
    for _ in 0..count {
        let (Ok(item), Ok(amount)) = (r.read_u32(), r.read_i16()) else {
            return;
        };
        if let Some(item) = w.find_item(Serial::new(item))
            && amount > 0
        {
            sell_list.push(SellItemResponse::new(item, i32::from(amount)));
        }
    }
    let taken = !sell_list.is_empty()
        && w.resources
            .get::<GameHooks>()
            .and_then(|h| h.vendor_sell_items)
            .is_some_and(|f| f(w, vendor, from, &sell_list));
    if taken {
        packets::vendor_sell::send_end_vendor_sell(w, id, vserial);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::item::ITEM_BASE;
    use muo_server::mobile::mobile_type::MOBILE_BASE;

    // `Drop(null, loc)` still takes the held item off the cursor (it bounces); it must not stay held.
    #[test]
    fn drop_on_missing_target_clears_holding() {
        let mut w = World::new();
        let m = w.new_mobile(&MOBILE_BASE);
        let i = w.new_item(&ITEM_BASE);
        muo_server::mobile::part1::props_a::set_holding(&mut w, m, Some(i));
        w.item_mut(i).unwrap().held_by = Some(m);
        w.item_mut(i).unwrap().map = Some(muo_server::map::MapId::INTERNAL);
        mobile::base::drop_missing(&mut w, m);
        assert!(w.mobile(m).unwrap().holding.is_none());
    }

    // `Enum.IsDefined(typeof(MessageType), 0xC0)`: Encoded is a member, so an ascii 0xC0 stays.
    #[test]
    fn encoded_message_type_is_defined() {
        assert!(is_defined(0xC0) && !is_defined(0x03));
    }
}
