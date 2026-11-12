//! `Teleporter` and `SkillTeleporter`, `KeywordTeleporter`, `WaitTeleporter`,
//! `TimeoutTeleporter`, `TimeoutGoal`, `ConditionTeleporter`, `InteractionTeleporter`.
//!
//! One [`TeleporterData`] in `Item::ext` holds the fields of the whole family; `TeleporterClass` is the family
//! vtable of the `virtual`s (`CanTeleport`, `StartTeleport`, `DoTeleport`, `AddProperties`).
//!
//! // shortcut: `m.BeginAction(this)` (a lock keyed by the teleporter object) is a `(mobile, teleporter)` set in
//! [`TeleporterTables`]; nothing else asks `CanBeginAction(teleporter)`.

use muo_primitives::dotnet::InvariantCase;
use std::collections::HashSet;

use indexmap::IndexMap;
use muo_io::{GenericReader, GenericWriter, TextDefinition};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, enumerable, ops};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{read_version, write_version};

// ---- TeleporterFlags, ConditionFlag ----

pub mod flags {
    pub const ACTIVE: i32 = 0x1;
    pub const CREATURES: i32 = 0x2;
    pub const COMBAT_CHECK: i32 = 0x4;
    pub const CRIMINAL_CHECK: i32 = 0x8;
    pub const SOURCE_EFFECT: i32 = 0x10;
    pub const DEST_EFFECT: i32 = 0x20;
}

pub mod condition {
    pub const DENY_MOUNTED: i32 = 0x001;
    pub const DENY_FOLLOWERS: i32 = 0x002;
    pub const DENY_PACK_CONTENTS: i32 = 0x004;
    pub const DENY_HOLDING: i32 = 0x008;
    pub const DENY_EQUIPMENT: i32 = 0x010;
    pub const DENY_TRANSFORMED: i32 = 0x020;
    pub const STAFF_ONLY: i32 = 0x040;
    pub const DENY_PACK_ETHEREALS: i32 = 0x080;
    pub const DEAD_ONLY: i32 = 0x100;
}

/// What the Teleporter classes add to an `Item`.
#[derive(Clone, Debug, Default)]
pub struct TeleporterData {
    pub flags: i32,
    pub delay: TimeSpan,
    pub sound_id: i32,
    pub point_dest: Point3D,
    pub map_dest: Option<MapId>,
    // SkillTeleporter
    pub skill: i32,
    pub required: f64,
    pub message: TextDefinition,
    // KeywordTeleporter
    pub substring: Option<String>,
    pub keyword: i32,
    pub range: i32,
    // WaitTeleporter
    pub start_message: TextDefinition,
    pub progress_message: TextDefinition,
    pub show_time_remaining: bool,
    // TimeoutTeleporter
    pub timeout_delay: TimeSpan,
    // ConditionTeleporter
    pub condition_flags: i32,
}

/// The statics of the classes: `WaitTeleporter._table`, the timers of `TimeoutTeleporter._teleporting`, and the
/// 5 second `BeginAction(this)` locks.
#[derive(Default)]
pub struct TeleporterTables {
    wait: IndexMap<MobileId, (ItemId, TimerHandle)>,
    timeout: IndexMap<(ItemId, MobileId), TimerHandle>,
    locks: HashSet<(MobileId, ItemId)>,
}

pub fn data(w: &World, id: ItemId) -> Option<&TeleporterData> {
    w.item(id)?.ext.get::<TeleporterData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut TeleporterData> {
    w.item_mut(id)?.ext.get_mut::<TeleporterData>()
}

fn d(w: &World, id: ItemId) -> TeleporterData {
    data(w, id).cloned().unwrap_or_default()
}

/// `GetFlag(flag)`.
pub fn get_flag(w: &World, id: ItemId, flag: i32) -> bool {
    data(w, id).is_some_and(|d| d.flags & flag != 0)
}

/// `SetFlag(flag, value)`.
pub fn set_flag(w: &mut World, id: ItemId, flag: i32, value: bool) {
    if let Some(d) = data_mut(w, id) {
        if value {
            d.flags |= flag;
        } else {
            d.flags &= !flag;
        }
    }
    ops::invalidate_properties(w, id);
}

fn active(w: &World, id: ItemId) -> bool {
    get_flag(w, id, flags::ACTIVE)
}

fn is_default_text(t: &TextDefinition) -> bool {
    t.number == 0 && t.string.is_none()
}

fn is_blank(s: &Option<String>) -> bool {
    s.as_deref().is_none_or(|s| s.trim().is_empty())
}

/// The virtuals of the family.
pub struct TeleporterClass {
    pub can_teleport: fn(&mut World, ItemId, MobileId) -> bool,
    pub start_teleport: fn(&mut World, ItemId, MobileId),
    pub do_teleport: fn(&mut World, ItemId, MobileId),
    pub add_properties: fn(&World, ItemId, &mut PropertyList),
}

fn class(w: &World, id: ItemId) -> &'static TeleporterClass {
    w.item(id)
        .and_then(|i| i.ty.family::<TeleporterClass>())
        .unwrap_or(&TELEPORTER_CLASS)
}

/// `CanTeleport(m)` through the class.
pub fn can_teleport(w: &mut World, id: ItemId, m: MobileId) -> bool {
    (class(w, id).can_teleport)(w, id, m)
}

/// `StartTeleport(m)` through the class.
pub fn start_teleport(w: &mut World, id: ItemId, m: MobileId) {
    (class(w, id).start_teleport)(w, id, m);
}

/// `DoTeleport(m)` through the class.
pub fn do_teleport(w: &mut World, id: ItemId, m: MobileId) {
    (class(w, id).do_teleport)(w, id, m);
}

fn tables(w: &mut World) -> &mut TeleporterTables {
    w.resources.get_mut::<TeleporterTables>()
}

// ---- Teleporter ----

/// `Teleporter(pointDest, mapDest, creatures)` of class `ty`.
pub fn teleporter_new(
    w: &mut World,
    ty: &'static ItemType,
    point_dest: Point3D,
    map_dest: Option<MapId>,
    creatures: bool,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1BC3);
    ops::set_movable(w, id, false);
    ops::set_visible(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(TeleporterData {
            flags: flags::ACTIVE,
            point_dest,
            map_dest,
            keyword: 0,
            ..Default::default()
        });
    }
    set_flag(w, id, flags::CREATURES, creatures);
    id
}

fn can_teleport_base(w: &mut World, id: ItemId, m: MobileId) -> bool {
    let (player, criminal) = w
        .mobile(m)
        .map_or((false, false), |m| (m.player, m.criminal));
    if !get_flag(w, id, flags::CREATURES) && !player {
        return false;
    }
    if get_flag(w, id, flags::CRIMINAL_CHECK) && criminal {
        // Thou'rt a criminal and cannot escape so easily.
        muo_server::mobile::part2::messages::send_localized_message(w, m, 1005561, "", 0x22);
        return false;
    }
    if get_flag(w, id, flags::COMBAT_CHECK) && muo_content::spells::helper::check_combat(w, m) {
        // Wouldst thou flee during the heat of battle??
        muo_server::mobile::part2::messages::send_localized_message(w, m, 1005564, "", 0x22);
        return false;
    }
    true
}

fn start_teleport_base(w: &mut World, id: ItemId, m: MobileId) {
    let delay = d(w, id).delay;
    if delay == TimeSpan::ZERO {
        do_teleport(w, id, m);
    } else {
        timer::start(w, delay, TimeSpan::ZERO, 1, move |w| do_teleport(w, id, m));
    }
}

fn do_teleport_base(w: &mut World, id: ItemId, m: MobileId) {
    let t = d(w, id);
    let Some((mloc, mmap, hidden, access)) = w
        .mobile(m)
        .map(|m| (m.location, m.map, m.hidden, m.access_level))
    else {
        return;
    };
    let map = match t.map_dest {
        Some(map) if map != MapId::INTERNAL => Some(map),
        _ => mmap,
    };
    let p = if t.point_dest == Point3D::ZERO {
        mloc
    } else {
        t.point_dest
    };
    if let Some(map) = map {
        muo_content::creature::pets::teleport_pets(w, m, p, map, false);
    }
    let send_effect = !hidden || access == AccessLevel::Player;
    if t.flags & flags::SOURCE_EFFECT != 0 && send_effect {
        muo_server::effects::send_location_effect(w, mloc, mmap, 0x3728, 10, 10, 0, 0);
    }
    if let Some(map) = map {
        muo_server::mobile::base::move_to_world(w, m, p, map);
    }
    let (loc, map_now) = w.mobile(m).map_or((p, map), |m| (m.location, m.map));
    if t.flags & flags::DEST_EFFECT != 0 && send_effect {
        muo_server::effects::send_location_effect(w, loc, map_now, 0x3728, 10, 10, 0, 0);
    }
    if t.sound_id > 0 && send_effect {
        muo_server::effects::play_sound(w, loc, map_now, t.sound_id);
    }
}

fn add_properties_base(w: &World, id: ItemId, list: &mut PropertyList) {
    let t = d(w, id);
    list.add(if active(w, id) { 1060742 } else { 1060743 }); // active : inactive
    if let Some(m) = t.map_dest {
        list.add_str(1060658, &format!("Map\t{}", map_name(w, m)));
    }
    if t.point_dest != Point3D::ZERO {
        list.add_str(1060659, &format!("Coords\t{}", t.point_dest));
    }
    list.add_str(
        1060660,
        &format!(
            "Creatures\t{}",
            if get_flag(w, id, flags::CREATURES) {
                "Yes"
            } else {
                "No"
            }
        ),
    );
}

fn map_name(w: &World, m: MapId) -> String {
    w.maps
        .get(m)
        .map_or_else(|| "Internal".to_string(), |m| m.name().to_string())
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    base::on_single_click_base(w, id, from);
    let t = d(w, id);
    if active(w, id) {
        match (t.map_dest, t.point_dest != Point3D::ZERO) {
            (Some(m), true) => ops::label_to(
                w,
                id,
                from,
                &format!("{} [{}]", t.point_dest, map_name(w, m)),
            ),
            (Some(m), false) => ops::label_to(w, id, from, &format!("[{}]", map_name(w, m))),
            (None, true) => ops::label_to(w, id, from, &t.point_dest.to_string()),
            (None, false) => {}
        }
    } else {
        ops::label_to(w, id, from, "(inactive)");
    }
}

fn on_move_over(w: &mut World, id: ItemId, m: MobileId) -> bool {
    if active(w, id) && can_teleport(w, id, m) {
        start_teleport(w, id, m);
        return false;
    }
    true
}

const TELEPORTER_VERSION: i32 = 5;

fn serialize_teleporter(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(TELEPORTER_VERSION);
    let t = d(w, id);
    out.write_enum(4, i64::from(t.flags));
    out.write_time_span_ticks(t.delay.ticks());
    out.write_encoded_int(t.sound_id);
    out.write_point3d(t.point_dest);
    out.write_map(t.map_dest);
}

fn deserialize_teleporter(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut t = TeleporterData::default();
    if version < TELEPORTER_VERSION {
        // private void Deserialize(IGenericReader reader, int version)
        if r.read_bool()? {
            t.flags |= flags::CRIMINAL_CHECK;
        }
        if r.read_bool()? {
            t.flags |= flags::COMBAT_CHECK;
        }
        if r.read_bool()? {
            t.flags |= flags::SOURCE_EFFECT;
        }
        if r.read_bool()? {
            t.flags |= flags::DEST_EFFECT;
        }
        t.delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
        t.sound_id = r.read_encoded_int()?;
        if r.read_bool()? {
            t.flags |= flags::CREATURES;
        }
        if r.read_bool()? {
            t.flags |= flags::ACTIVE;
        }
        t.point_dest = r.read_point3d()?;
        t.map_dest = r.read_map()?;
    } else {
        t.flags = r.read_enum(4)? as i32;
        t.delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
        t.sound_id = r.read_encoded_int()?;
        t.point_dest = r.read_point3d()?;
        t.map_dest = r.read_map()?;
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(t);
    }
    Ok(())
}

pub static TELEPORTER_CLASS: TeleporterClass = TeleporterClass {
    can_teleport: can_teleport_base,
    start_teleport: start_teleport_base,
    do_teleport: do_teleport_base,
    add_properties: add_properties_base,
};

/// `Server.Items.Teleporter`.
pub static TELEPORTER: ItemType = ItemType {
    full_name: "Server.Items.Teleporter",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| teleporter_new(w, &TELEPORTER, Point3D::ZERO, None, false)),
    family: Some(&TELEPORTER_CLASS),
    label_number: |_, _| 1026095, // teleporter
    get_properties: |w, id, list| {
        base::get_properties_base(w, id, list);
        (class(w, id).add_properties)(w, id, list);
    },
    on_single_click,
    on_move_over,
    serialize: serialize_teleporter,
    deserialize: deserialize_teleporter,
    ..ITEM_BASE
};

// ---- SkillTeleporter ----

fn skill_can_teleport(w: &mut World, id: ItemId, m: MobileId) -> bool {
    if !can_teleport_base(w, id, m) {
        return false;
    }
    let t = d(w, id);
    let skill = SkillName::from_i32(t.skill);
    // `sk?.Base >= _required`: an unknown skill index is a null skill.
    if skill.is_some_and(|s| muo_server::skills::base(w, m, s) >= t.required) {
        return true;
    }
    if !tables(w).locks.insert((m, id)) {
        return false;
    }
    let (serial, graphic) = w.item(id).map_or((muo_primitives::Serial::new(0), 0), |i| {
        (i.serial, i.item_id)
    });
    if let Some(ns) = deps::net_state_of(w, m) {
        if t.message.number > 0 {
            muo_server::packets::message::send_message_localized(
                w,
                ns,
                serial,
                graphic,
                muo_server::packets::message::MessageType::REGULAR,
                0x3B2,
                3,
                t.message.number,
                "",
                "",
            );
        } else if !is_blank(&t.message.string) {
            muo_server::packets::message::send_message(
                w,
                ns,
                serial,
                graphic,
                muo_server::packets::message::MessageType::REGULAR,
                0x3B2,
                3,
                false,
                Some("ENU"),
                "",
                t.message.string.as_deref().unwrap_or(""),
            );
        }
    }
    timer::start(
        w,
        TimeSpan::from_secs_f64(5.0),
        TimeSpan::ZERO,
        1,
        move |w| {
            tables(w).locks.remove(&(m, id));
        },
    );
    false
}

fn skill_add_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    let t = d(w, id);
    let names = w.resources.get::<muo_server::skills::SkillTable>();
    let skill_name = names
        .and_then(|n| usize::try_from(t.skill).ok().and_then(|i| n.info.get(i)))
        .map_or("(Invalid)".to_string(), |s| s.name.to_string());
    list.add_str(1060661, &format!("{}\t{:.1}", skill_name, t.required));
    if t.message.number > 0 {
        list.add_str(1060662, &format!("Message\t{}", t.message.number));
    } else if !is_blank(&t.message.string) {
        list.add_str(
            1060662,
            &format!("Message\t{}", t.message.string.as_deref().unwrap_or("")),
        );
    }
}

pub static SKILL_TELEPORTER_CLASS: TeleporterClass = TeleporterClass {
    can_teleport: skill_can_teleport,
    add_properties: skill_add_properties,
    ..TELEPORTER_CLASS
};

/// `Server.Items.SkillTeleporter`.
pub static SKILL_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Items.SkillTeleporter",
    parent: Some(&TELEPORTER),
    construct: Some(|w| teleporter_new(w, &SKILL_TELEPORTER, Point3D::ZERO, None, false)),
    family: Some(&SKILL_TELEPORTER_CLASS),
    serialize: |w, id, out| {
        serialize_teleporter(w, id, out);
        out.write_i32(1);
        let t = d(w, id);
        out.write_enum(4, i64::from(t.skill));
        out.write_f64(t.required);
        if is_default_text(&t.message) {
            out.write_bool(false);
        } else {
            out.write_bool(true);
            out.write_text_definition(Some(&t.message));
        }
    },
    deserialize: |w, id, r| {
        deserialize_teleporter(w, id, r)?;
        let version = r.read_i32()?;
        let (skill, required, message);
        if version < 1 {
            skill = r.read_i32()?;
            required = r.read_f64()?;
            let text = r.read_string()?;
            let number = r.read_i32()?;
            message = if number > 0 {
                TextDefinition::of_number(number)
            } else {
                text.map_or(TextDefinition::EMPTY, TextDefinition::of_text)
            };
        } else {
            skill = r.read_enum(4)? as i32;
            required = r.read_f64()?;
            message = if r.read_bool()? {
                r.read_text_definition()?.unwrap_or_default()
            } else {
                TextDefinition::EMPTY
            };
        }
        if let Some(t) = data_mut(w, id) {
            t.skill = skill;
            t.required = required;
            t.message = message;
        }
        Ok(())
    },
    ..TELEPORTER
};

// ---- KeywordTeleporter ----

fn keyword_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = teleporter_new(w, ty, Point3D::ZERO, None, false);
    if let Some(t) = data_mut(w, id) {
        t.keyword = -1;
        t.substring = None;
    }
    id
}

fn keyword_do_teleport(w: &mut World, id: ItemId, m: MobileId) {
    let loc = ops::get_world_location(w, id);
    let range = d(w, id).range;
    let same_map = w.mobile(m).map(|m| m.map) == w.item(id).map(|i| i.map);
    if !deps::mobile_in_range(w, m, loc, range) || !same_map {
        return;
    }
    do_teleport_base(w, id, m);
}

fn keyword_add_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    let t = d(w, id);
    list.add_str(1060661, &format!("Range\t{}", t.range));
    if t.keyword >= 0 {
        list.add_str(1060662, &format!("Keyword\t{}", t.keyword));
    }
    if let Some(s) = &t.substring {
        list.add_str(1060663, &format!("Substring\t{s}"));
    }
}

fn keyword_on_speech(w: &mut World, id: ItemId, e: &mut muo_server::events::SpeechEventArgs) {
    if e.handled || !active(w, id) {
        return;
    }
    let m = e.mobile;
    let t = d(w, id);
    let loc = ops::get_world_location(w, id);
    if !deps::mobile_in_range(w, m, loc, t.range) {
        return;
    }
    let is_match = (t.keyword >= 0 && e.has_keyword(t.keyword))
        || t.substring
            .as_deref()
            .is_some_and(|s| e.speech.lower_inv().contains(&s.lower_inv()));
    if !is_match || !can_teleport(w, id, m) {
        return;
    }
    e.handled = true;
    start_teleport(w, id, m);
}

pub static KEYWORD_TELEPORTER_CLASS: TeleporterClass = TeleporterClass {
    do_teleport: keyword_do_teleport,
    add_properties: keyword_add_properties,
    ..TELEPORTER_CLASS
};

/// `Server.Items.KeywordTeleporter`.
pub static KEYWORD_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Items.KeywordTeleporter",
    parent: Some(&TELEPORTER),
    construct: Some(|w| keyword_new(w, &KEYWORD_TELEPORTER)),
    family: Some(&KEYWORD_TELEPORTER_CLASS),
    handles_on_speech: |_, _| true,
    on_speech: keyword_on_speech,
    on_move_over: |_, _, _| true,
    serialize: |w, id, out| {
        serialize_teleporter(w, id, out);
        write_version(out, false);
        let t = d(w, id);
        out.write_opt_string(t.substring.as_deref());
        out.write_i32(t.keyword);
        out.write_i32(t.range);
    },
    deserialize: |w, id, r| {
        deserialize_teleporter(w, id, r)?;
        read_version(r, false)?;
        let substring = r.read_string()?;
        let keyword = r.read_i32()?;
        let range = r.read_i32()?;
        if let Some(t) = data_mut(w, id) {
            t.substring = substring;
            t.keyword = keyword;
            t.range = range;
        }
        Ok(())
    },
    ..TELEPORTER
};

// ---- WaitTeleporter ----

/// `WaitTeleporter.FormatTime(ts)`.
pub fn format_time(ts: TimeSpan) -> String {
    // Math.Round is banker's rounding (to even).
    let round = |v: f64| v.round_ties_even() as i32;
    if ts.total_hours() >= 1.0 {
        let h = round(ts.total_hours());
        return format!("{h} hour{}", if h == 1 { "" } else { "s" });
    }
    if ts.total_minutes() >= 1.0 {
        let m = round(ts.total_minutes());
        return format!("{m} minute{}", if m == 1 { "" } else { "s" });
    }
    let s = round(ts.total_seconds()).max(0);
    format!("{s} second{}", if s == 1 { "" } else { "s" })
}

fn wait_start_teleport(w: &mut World, id: ItemId, m: MobileId) {
    if let Some((tele, mut token)) = tables(w).wait.get(&m).copied() {
        if tele == id {
            if !tables(w).locks.insert((m, id)) {
                return;
            }
            let t = d(w, id);
            muo_content::text::send_message_to(w, Some(&t.progress_message), m);
            if t.show_time_remaining {
                let left = token.next(w) - w.now;
                muo_server::mobile::part2::messages::send_message(
                    w,
                    m,
                    &format!("Time remaining: {}", format_time(left)),
                );
            }
            timer::start(
                w,
                TimeSpan::from_secs_f64(5.0),
                TimeSpan::ZERO,
                1,
                move |w| {
                    tables(w).locks.remove(&(m, id));
                },
            );
            return;
        }
        token.cancel(w);
    }
    let t = d(w, id);
    muo_content::text::send_message_to(w, Some(&t.start_message), m);
    if t.delay == TimeSpan::ZERO {
        do_teleport(w, id, m);
    } else {
        let token = timer::start(w, t.delay, TimeSpan::ZERO, 1, move |w| {
            do_teleport(w, id, m)
        });
        tables(w).wait.insert(m, (id, token));
    }
}

fn wait_do_teleport(w: &mut World, id: ItemId, m: MobileId) {
    tables(w).wait.shift_remove(&m);
    keyword_do_teleport(w, id, m);
}

fn wait_logout(w: &mut World, m: MobileId) {
    if let Some((_, mut token)) = tables(w).wait.shift_remove(&m) {
        token.cancel(w);
    }
}

pub static WAIT_TELEPORTER_CLASS: TeleporterClass = TeleporterClass {
    start_teleport: wait_start_teleport,
    do_teleport: wait_do_teleport,
    ..KEYWORD_TELEPORTER_CLASS
};

/// `Server.Items.WaitTeleporter`.
pub static WAIT_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Items.WaitTeleporter",
    parent: Some(&KEYWORD_TELEPORTER),
    construct: Some(|w| keyword_new(w, &WAIT_TELEPORTER)),
    family: Some(&WAIT_TELEPORTER_CLASS),
    serialize: |w, id, out| {
        (KEYWORD_TELEPORTER.serialize)(w, id, out);
        out.write_i32(1);
        let t = d(w, id);
        for msg in [&t.start_message, &t.progress_message] {
            if is_default_text(msg) {
                out.write_bool(false);
            } else {
                out.write_bool(true);
                out.write_text_definition(Some(msg));
            }
        }
        out.write_bool(t.show_time_remaining);
    },
    deserialize: |w, id, r| {
        (KEYWORD_TELEPORTER.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let (start, progress, show);
        if version < 1 {
            let read_msg = |r: &mut dyn GenericReader| -> muo_io::Result<TextDefinition> {
                let number = r.read_i32()?;
                let message = r.read_string()?;
                Ok(if number > 0 {
                    TextDefinition::of_number(number)
                } else {
                    message.map_or(TextDefinition::EMPTY, TextDefinition::of_text)
                })
            };
            start = read_msg(r)?;
            progress = read_msg(r)?;
            show = r.read_bool()?;
        } else {
            let read_opt = |r: &mut dyn GenericReader| -> muo_io::Result<TextDefinition> {
                Ok(if r.read_bool()? {
                    r.read_text_definition()?.unwrap_or_default()
                } else {
                    TextDefinition::EMPTY
                })
            };
            start = read_opt(r)?;
            progress = read_opt(r)?;
            show = r.read_bool()?;
        }
        if let Some(t) = data_mut(w, id) {
            t.start_message = start;
            t.progress_message = progress;
            t.show_time_remaining = show;
        }
        Ok(())
    },
    ..KEYWORD_TELEPORTER
};

// ---- TimeoutTeleporter, TimeoutGoal ----

/// `TimeoutTeleporter.StartTimer(m)`.
pub fn timeout_start_timer(w: &mut World, id: ItemId, m: MobileId) {
    let delay = d(w, id).timeout_delay;
    timeout_stop_timer(w, id, m);
    let token = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        start_teleport(w, id, m)
    });
    tables(w).timeout.insert((id, m), token);
}

/// `TimeoutTeleporter.StopTimer(m)`.
pub fn timeout_stop_timer(w: &mut World, id: ItemId, m: MobileId) {
    if let Some(mut t) = tables(w).timeout.shift_remove(&(id, m)) {
        t.cancel(w);
    }
}

fn timeout_do_teleport(w: &mut World, id: ItemId, m: MobileId) {
    tables(w).timeout.shift_remove(&(id, m));
    do_teleport_base(w, id, m);
}

pub static TIMEOUT_TELEPORTER_CLASS: TeleporterClass = TeleporterClass {
    do_teleport: timeout_do_teleport,
    ..TELEPORTER_CLASS
};

/// `Server.Items.TimeoutTeleporter`.
pub static TIMEOUT_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Items.TimeoutTeleporter",
    parent: Some(&TELEPORTER),
    construct: Some(|w| teleporter_new(w, &TIMEOUT_TELEPORTER, Point3D::ZERO, None, false)),
    family: Some(&TIMEOUT_TELEPORTER_CLASS),
    on_move_over: |w, id, m| {
        if active(w, id) {
            if !can_teleport(w, id, m) {
                return false;
            }
            timeout_start_timer(w, id, m);
        }
        true
    },
    serialize: |w, id, out| {
        serialize_teleporter(w, id, out);
        out.write_i32(1);
        out.write_time_span_ticks(d(w, id).timeout_delay.ticks());
    },
    deserialize: |w, id, r| {
        deserialize_teleporter(w, id, r)?;
        let version = r.read_i32()?;
        let delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
        if version < 1 {
            // the saved list of (mobile, DateTime) is read and dropped
            let count = r.read_i32()?;
            for _ in 0..count {
                r.read_entity(w)?;
                r.read_date_time_ticks()?;
            }
        }
        if let Some(t) = data_mut(w, id) {
            t.timeout_delay = delay;
        }
        Ok(())
    },
    ..TELEPORTER
};

/// What `TimeoutGoal` adds.
#[derive(Clone, Debug, Default)]
struct GoalData {
    teleporter: Option<ItemId>,
}

fn goal_teleporter(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.ext.get::<GoalData>()?.teleporter
}

/// `Server.Items.TimeoutGoal`.
pub static TIMEOUT_GOAL: ItemType = ItemType {
    full_name: "Server.Items.TimeoutGoal",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &TIMEOUT_GOAL, 0x1822);
        ops::set_movable(w, id, false);
        ops::set_visible(w, id, false);
        super::hue(w, id, 1154);
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(GoalData::default());
        }
        id
    }),
    default_name: |_, _| Some("timeout teleporter goal".to_string()),
    on_move_over: |w, id, m| {
        if let Some(t) = goal_teleporter(w, id) {
            timeout_stop_timer(w, t, m);
        }
        true
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
        out.write_item(w, goal_teleporter(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        let teleporter = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(GoalData { teleporter });
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- ConditionTeleporter ----

fn pack_has_ethereal(w: &World, pack: ItemId) -> bool {
    !enumerable::find_items(w, pack, true, |w, i| {
        w.item(i).is_some_and(|i| {
            muo_content::items::is_named(i.ty, "Server.Mobiles.EtherealMount")
                || muo_content::items::is_named(i.ty, "Server.Items.BaseImprisonedMobile")
        })
    })
    .is_empty()
}

fn condition_can_teleport(w: &mut World, id: ItemId, m: MobileId) -> bool {
    use muo_server::mobile::part1::props_b;
    if !can_teleport_base(w, id, m) {
        return false;
    }
    let f = |w: &World, flag: i32| data(w, id).is_some_and(|d| d.condition_flags & flag != 0);
    let access = w.mobile(m).map_or(AccessLevel::Player, |m| m.access_level);
    if f(w, condition::STAFF_ONLY) && access < AccessLevel::Counselor {
        return false;
    }
    if f(w, condition::DENY_MOUNTED) && props_b::mounted(w, m) {
        deps::send_localized_message(w, m, 1077252); // You must dismount before proceeding.
        return false;
    }
    if f(w, condition::DENY_FOLLOWERS) {
        let followers = w.mobile(m).map_or(0, |m| m.followers);
        let auto_stabled = muo_content::player_mobile::data(w, m)
            .and_then(|d| d.auto_stabled.as_ref())
            .map_or(0, |s| s.len());
        if followers != 0 || auto_stabled > 0 {
            deps::send_localized_message(w, m, 1077250); // No pets permitted beyond this point.
            return false;
        }
    }
    let pack = w.mobile(m).and_then(|m| m.backpack);
    if let Some(pack) = pack {
        if f(w, condition::DENY_PACK_CONTENTS)
            && ops::get_total(w, pack, muo_server::item::TotalType::Items) != 0
        {
            muo_server::mobile::part2::messages::send_message(
                w,
                m,
                "You must empty your backpack before proceeding.",
            );
            return false;
        }
        if f(w, condition::DENY_PACK_ETHEREALS) && pack_has_ethereal(w, pack) {
            muo_server::mobile::part2::messages::send_message(
                w,
                m,
                "You must empty your backpack of ethereal mounts before proceeding.",
            );
            return false;
        }
    }
    if f(w, condition::DENY_HOLDING) && w.mobile(m).is_some_and(|m| m.holding.is_some()) {
        muo_server::mobile::part2::messages::send_message(
            w,
            m,
            "You must let go of what you are holding before proceeding.",
        );
        return false;
    }
    if f(w, condition::DENY_EQUIPMENT) {
        let items = w.mobile(m).map(|m| m.items.clone()).unwrap_or_default();
        for item in items {
            let layer = w.item(item).map(|i| (i.ty.layer)(w, item));
            match layer {
                Some(
                    Layer::Hair | Layer::FacialHair | Layer::Backpack | Layer::Mount | Layer::Bank,
                ) => continue,
                _ => {
                    muo_server::mobile::part2::messages::send_message(
                        w,
                        m,
                        "You must remove all equipment before proceeding.",
                    );
                    return false;
                }
            }
        }
    }
    if f(w, condition::DENY_TRANSFORMED) && props_b::is_body_mod(w, m) {
        muo_server::mobile::part2::messages::send_message(
            w,
            m,
            "You cannot go there in this form.",
        );
        return false;
    }
    if f(w, condition::DEAD_ONLY) && muo_server::mobile::base::alive(w, m) {
        deps::send_localized_message(w, m, 1060014); // Only the dead may pass.
        return false;
    }
    true
}

fn condition_add_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    let f = data(w, id).map_or(0, |d| d.condition_flags);
    let mut props = String::new();
    for (flag, text) in [
        (condition::DENY_MOUNTED, "<BR>Deny Mounted"),
        (condition::DENY_FOLLOWERS, "<BR>Deny Followers"),
        (condition::DENY_PACK_CONTENTS, "<BR>Deny Pack Contents"),
        (condition::DENY_PACK_ETHEREALS, "<BR>Deny Pack Ethereals"),
        (condition::DENY_HOLDING, "<BR>Deny Holding"),
        (condition::DENY_EQUIPMENT, "<BR>Deny Equipment"),
        (condition::DENY_TRANSFORMED, "<BR>Deny Transformed"),
        (condition::STAFF_ONLY, "<BR>Staff Only"),
        (condition::DEAD_ONLY, "<BR>Dead Only"),
    ] {
        if f & flag != 0 {
            props.push_str(text);
        }
    }
    if !props.is_empty() {
        list.add_text(&props[4..]);
    }
}

pub static CONDITION_TELEPORTER_CLASS: TeleporterClass = TeleporterClass {
    can_teleport: condition_can_teleport,
    add_properties: condition_add_properties,
    ..TELEPORTER_CLASS
};

/// `Server.Items.ConditionTeleporter`.
pub static CONDITION_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Items.ConditionTeleporter",
    parent: Some(&TELEPORTER),
    construct: Some(|w| teleporter_new(w, &CONDITION_TELEPORTER, Point3D::ZERO, None, false)),
    family: Some(&CONDITION_TELEPORTER_CLASS),
    serialize: |w, id, out| {
        serialize_teleporter(w, id, out);
        out.write_i32(1);
        out.write_enum(4, i64::from(d(w, id).condition_flags));
    },
    deserialize: |w, id, r| {
        deserialize_teleporter(w, id, r)?;
        let version = r.read_i32()?;
        let f = if version < 1 {
            r.read_i32()?
        } else {
            r.read_enum(4)? as i32
        };
        if let Some(t) = data_mut(w, id) {
            t.condition_flags = f;
        }
        Ok(())
    },
    ..TELEPORTER
};

// ---- InteractionTeleporter ----

fn interaction_on_double_click(w: &mut World, id: ItemId, m: MobileId) {
    base::on_double_click_base(w, id, m);
    let loc = ops::get_world_location(w, id);
    if active(w, id) && can_teleport(w, id, m) && deps::mobile_in_range(w, m, loc, 2) {
        start_teleport(w, id, m);
    }
}

pub static INTERACTION_TELEPORTER_CLASS: TeleporterClass = TeleporterClass {
    add_properties: |_, _, _| {},
    ..TELEPORTER_CLASS
};

/// `InteractionTeleporter(pointDest, mapDest)`: `Visible = true`.
pub fn interaction_teleporter_new(
    w: &mut World,
    ty: &'static ItemType,
    point_dest: Point3D,
    map_dest: Option<MapId>,
) -> ItemId {
    let id = teleporter_new(w, ty, point_dest, map_dest, false);
    ops::set_visible(w, id, true);
    id
}

/// `Server.Items.InteractionTeleporter`.
pub static INTERACTION_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Items.InteractionTeleporter",
    parent: Some(&TELEPORTER),
    construct: Some(|w| {
        interaction_teleporter_new(w, &INTERACTION_TELEPORTER, Point3D::ZERO, None)
    }),
    family: Some(&INTERACTION_TELEPORTER_CLASS),
    on_move_over: |_, _, _| true,
    on_double_click: interaction_on_double_click,
    on_double_click_dead: interaction_on_double_click,
    serialize: |w, id, out| {
        serialize_teleporter(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        deserialize_teleporter(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..TELEPORTER
};

// ---- the [CommandProperty] members ----

mod props_tables {
    use super::*;
    use muo_content::ext_field;
    use muo_content::props::{self, Kind, PropDef, Value};
    use muo_content::text_def_prop;
    use muo_content::utilities::types::{Numeric, PropType};
    use muo_server::ids::EntityId;

    const GM: AccessLevel = AccessLevel::GameMaster;
    const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
    const BOOL: Kind = Kind::Plain(PropType::Bool);
    const FLAGS: &[(&str, i64)] = &[
        ("None", 0),
        ("Active", 1),
        ("Creatures", 2),
        ("CombatCheck", 4),
        ("CriminalCheck", 8),
        ("SourceEffect", 0x10),
        ("DestEffect", 0x20),
    ];

    /// `Active { get => GetFlag(..); set => SetFlag(..) }`.
    macro_rules! flag_prop {
        ($name:literal, $flag:expr) => {
            PropDef {
                name: $name,
                kind: BOOL,
                read: GM,
                write: GM,
                get: |w, e| match e {
                    EntityId::Item(i) => Value::Bool(get_flag(w, i, $flag)),
                    _ => Value::Null,
                },
                set: Some(|w, e, v| {
                    if let EntityId::Item(i) = e {
                        set_flag(w, i, $flag, v.bool());
                    }
                    Ok(())
                }),
            }
        };
    }

    /// `ConditionTeleporter`'s `Deny*` properties: SetFlag, then InvalidateProperties.
    macro_rules! cond_prop {
        ($name:literal, $flag:expr) => {
            ext_field!(
                data,
                data_mut,
                $name,
                BOOL,
                GM,
                GM,
                |d| Value::Bool(d.condition_flags & $flag != 0),
                |d, v| if v.bool() {
                    d.condition_flags |= $flag
                } else {
                    d.condition_flags &= !$flag
                },
                true
            )
        };
    }

    /// A `TextDefinition` property of the family: `$field` of the data, null when it is empty.
    macro_rules! td_prop {
        ($name:literal, $field:ident) => {
            text_def_prop!(
                $name,
                GM,
                GM,
                |w: &World, e: EntityId| match e {
                    EntityId::Item(i) => data(w, i)
                        .and_then(|d| (!is_default_text(&d.$field)).then(|| d.$field.clone())),
                    _ => None,
                },
                Some(|w: &mut World, e: EntityId, v: Value| {
                    if let EntityId::Item(i) = e {
                        if let Some(d) = data_mut(w, i) {
                            d.$field = match &v {
                                Value::TextDef(t) => t.clone(),
                                _ => TextDefinition::EMPTY,
                            };
                        }
                        ops::invalidate_properties(w, i);
                    }
                    Ok(())
                })
            )
        };
    }

    pub static TELEPORTER: &[PropDef] = &[
        ext_field!(
            data,
            data_mut,
            "Flags",
            Kind::Plain(PropType::Enum(FLAGS)),
            GM,
            GM,
            |d| Value::Enum(i64::from(d.flags)),
            |d, v| d.flags = v.i32(),
            true
        ),
        ext_field!(
            data,
            data_mut,
            "Delay",
            Kind::TimeSpan,
            GM,
            GM,
            |d| Value::TimeSpan(d.delay),
            |d, v| if let Value::TimeSpan(t) = v {
                d.delay = *t
            },
            true
        ),
        ext_field!(
            data,
            data_mut,
            "SoundID",
            INT,
            GM,
            GM,
            |d| Value::Int(i64::from(d.sound_id)),
            |d, v| d.sound_id = v.i32(),
            true
        ),
        ext_field!(
            data,
            data_mut,
            "PointDest",
            Kind::Plain(PropType::IPoint3D),
            GM,
            GM,
            |d| Value::Point3D(d.point_dest),
            |d, v| d.point_dest = v.point(),
            true
        ),
        ext_field!(
            data,
            data_mut,
            "MapDest",
            Kind::Map,
            GM,
            GM,
            |d| Value::Map(d.map_dest),
            |d, v| d.map_dest = v.map(),
            true
        ),
        flag_prop!("SourceEffect", flags::SOURCE_EFFECT),
        flag_prop!("DestEffect", flags::DEST_EFFECT),
        flag_prop!("Active", flags::ACTIVE),
        flag_prop!("Creatures", flags::CREATURES),
        flag_prop!("CombatCheck", flags::COMBAT_CHECK),
        flag_prop!("CriminalCheck", flags::CRIMINAL_CHECK),
    ];

    pub static SKILL: &[PropDef] = &[
        ext_field!(
            data,
            data_mut,
            "Skill",
            Kind::Plain(PropType::Enum(props::SKILL_NAMES)),
            GM,
            GM,
            |d| Value::Enum(i64::from(d.skill)),
            |d, v| d.skill = v.i32(),
            true
        ),
        ext_field!(
            data,
            data_mut,
            "Required",
            Kind::Plain(PropType::Double),
            GM,
            GM,
            |d| Value::Float(d.required),
            |d, v| d.required = v.f64(),
            true
        ),
        td_prop!("Message", message),
    ];

    pub static KEYWORD: &[PropDef] = &[
        ext_field!(
            data,
            data_mut,
            "Substring",
            Kind::Plain(PropType::Str),
            GM,
            GM,
            |d| d.substring.clone().map_or(Value::Null, Value::Str),
            |d, v| d.substring = v.string(),
            false
        ),
        ext_field!(
            data,
            data_mut,
            "Keyword",
            INT,
            GM,
            GM,
            |d| Value::Int(i64::from(d.keyword)),
            |d, v| d.keyword = v.i32(),
            false
        ),
        ext_field!(
            data,
            data_mut,
            "Range",
            INT,
            GM,
            GM,
            |d| Value::Int(i64::from(d.range)),
            |d, v| d.range = v.i32(),
            false
        ),
    ];

    pub static WAIT: &[PropDef] = &[
        td_prop!("StartMessage", start_message),
        td_prop!("ProgressMessage", progress_message),
        ext_field!(
            data,
            data_mut,
            "ShowTimeRemaining",
            BOOL,
            GM,
            GM,
            |d| Value::Bool(d.show_time_remaining),
            |d, v| d.show_time_remaining = v.bool(),
            true
        ),
    ];

    pub static TIMEOUT: &[PropDef] = &[ext_field!(
        data,
        data_mut,
        "TimeoutDelay",
        Kind::TimeSpan,
        GM,
        GM,
        |d| Value::TimeSpan(d.timeout_delay),
        |d, v| if let Value::TimeSpan(t) = v {
            d.timeout_delay = *t
        },
        false
    )];

    pub static GOAL: &[PropDef] = &[PropDef {
        name: "Teleporter",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Entity(goal_teleporter(w, i).map(EntityId::Item)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err("That is not an item.".to_string());
            };
            let target = match v.entity() {
                // a `TimeoutTeleporter` only: any other item does nothing
                Some(EntityId::Item(t))
                    if w.item(t).is_some_and(|x| x.ty.is(&TIMEOUT_TELEPORTER)) =>
                {
                    Some(t)
                }
                None if v.is_null() => None,
                _ => return Err("not a TimeoutTeleporter".to_string()),
            };
            if let Some(g) = w.item_mut(i).and_then(|i| i.ext.get_mut::<GoalData>()) {
                g.teleporter = target;
            }
            Ok(())
        }),
    }];

    pub static CONDITION: &[PropDef] = &[
        cond_prop!("DenyMounted", condition::DENY_MOUNTED),
        cond_prop!("DenyFollowers", condition::DENY_FOLLOWERS),
        cond_prop!("DenyPackContents", condition::DENY_PACK_CONTENTS),
        cond_prop!("DenyHolding", condition::DENY_HOLDING),
        cond_prop!("DenyEquipment", condition::DENY_EQUIPMENT),
        cond_prop!("DenyTransformed", condition::DENY_TRANSFORMED),
        cond_prop!("StaffOnly", condition::STAFF_ONLY),
        cond_prop!("DenyPackEthereals", condition::DENY_PACK_ETHEREALS),
        cond_prop!("DeadOnly", condition::DEAD_ONLY),
    ];

    pub fn configure(w: &mut World) {
        props::add(w, "Server.Items.Teleporter", TELEPORTER);
        props::add(w, "Server.Items.SkillTeleporter", SKILL);
        props::add(w, "Server.Items.KeywordTeleporter", KEYWORD);
        props::add(w, "Server.Items.WaitTeleporter", WAIT);
        props::add(w, "Server.Items.TimeoutTeleporter", TIMEOUT);
        props::add(w, "Server.Items.TimeoutGoal", GOAL);
        props::add(w, "Server.Items.ConditionTeleporter", CONDITION);
    }
}

/// The classes of this file.
pub static ALL: &[&ItemType] = &[
    &TELEPORTER,
    &SKILL_TELEPORTER,
    &KEYWORD_TELEPORTER,
    &WAIT_TELEPORTER,
    &TIMEOUT_TELEPORTER,
    &TIMEOUT_GOAL,
    &CONDITION_TELEPORTER,
    &INTERACTION_TELEPORTER,
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.Teleporter.Properties",
        props_tables::configure,
    );
    // WaitTeleporter.Initialize: EventSink.Logout += EventSink_Logout
    types.register_phase_default(
        Phase::Initialize,
        "Server.Items.WaitTeleporter.Initialize",
        |w| {
            w.events.logout.subscribe(|w, m| wait_logout(w, *m));
        },
    );
}
