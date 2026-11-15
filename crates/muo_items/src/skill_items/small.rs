//! `Lockpick`, the clocks (`Clock`, `ClockRight`, `ClockLeft`) and `Spyglass`.
//!
//! `Spyglass.OnDoubleClick`'s Witch Apprentice branch (the star chart objective) is the quest engine's, reached
//! through [`SpyglassHooks`].
//! `ILockpickable` is the LockableContainer family (`PuzzleChest`, `TreasureMapChest`, `MarkContainer` derive from it).

use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_primitives::{DateTime, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, messages as item_messages, ops};
use muo_server::map::MapId;
use muo_server::mobile::part2::effects;
use muo_server::packets::message::{AffixType, MessageType};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{read_version, write_version};

// ---- Lockpick ----

fn lockable(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| i.ty.is(&muo_content::items::cont::LOCKABLE_CONTAINER))
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    deps::send_localized_message(w, from, 502068); // What do you want to pick?
    target::begin_target(w, from, 1, false, TargetFlags::NONE, move |w, from, t| {
        if w.item(id).is_none_or(|i| i.deleted()) {
            return;
        }
        let Targeted::Entity(EntityId::Item(item)) = t else {
            deps::send_localized_message(w, from, 501666); // You can't unlock that!
            return;
        };
        if !lockable(w, item) {
            deps::send_localized_message(w, from, 501666); // You can't unlock that!
            return;
        }
        if muo_content::items::cont::locked(w, item) {
            if ops::root_parent(w, item) != Some(EntityId::Mobile(from)) {
                let loc = ops::get_world_location(w, item);
                let d = effects::get_direction_to(w, from, loc.x, loc.y, false);
                muo_server::mobile::part1::props_b::set_direction(w, from, d);
            }
            effects::play_sound(w, from, 0x241);
            muo_server::timer::start(w, TimeSpan::from_secs(3), TimeSpan::ZERO, 1, move |w| {
                pick_tick(w, from, item, id)
            });
        } else {
            deps::send_localized_message(w, from, 502069); // This does not appear to be locked
        }
    });
}

/// `InternalTimer.OnTick`.
fn pick_tick(w: &mut World, from: MobileId, item: ItemId, lockpick: ItemId) {
    let loc = ops::get_world_location(w, item);
    if !deps::mobile_in_range(w, from, loc, 1) {
        return;
    }
    let Some((lock_level, max_level, required)) = muo_content::items::cont::ext(w, item)
        .map(|d| (d.lock_level, d.max_lock_level, d.required_skill))
    else {
        return;
    };
    if lock_level == 0 || lock_level == -255 {
        // LockLevel of 0 means that the door can't be picklocked; -255 means it's magic locked
        item_messages::send_localized_message_to(w, item, from, 502073, ""); // This lock cannot be picked by normal means
        return;
    }
    let skill = muo_server::skills::value(w, from, muo_server::skills::SkillName::Lockpicking);
    if skill < f64::from(required) {
        item_messages::send_localized_message_to(w, item, from, 502072, ""); // You don't see how that lock can be manipulated.
        return;
    }
    let ok = muo_server::mobile::part2::stats::check_target_skill(
        w,
        from,
        muo_server::skills::SkillName::Lockpicking,
        Targeted::Entity(EntityId::Item(item)),
        f64::from(lock_level),
        f64::from(max_level),
    );
    if ok {
        item_messages::send_localized_message_to(w, item, from, 502076, ""); // The lock quickly yields to your skill.
        effects::play_sound(w, from, 0x4A);
        muo_content::items::cont::lock_pick(w, item, from);
    } else {
        // When failed, a 25% chance to break the lockpick
        if utility::random(&mut w.rng, 4) == 0 {
            item_messages::send_localized_message_to(w, item, from, 502074, ""); // You broke the lockpick.
            effects::play_sound(w, from, 0x3A4);
            base::consume(w, lockpick, 1);
        }
        item_messages::send_localized_message_to(w, item, from, 502075, ""); // You are unable to pick the lock.
    }
}

/// `Server.Items.Lockpick`.
pub static LOCKPICK: ItemType = ItemType {
    full_name: "Server.Items.Lockpick",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_lockpick(w, 1)),
    on_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new Lockpick(amount)`.
pub fn new_lockpick(w: &mut World, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, &LOCKPICK, "Server.Items.Lockpick");
    ops::set_amount(w, id, amount);
    id
}

// ---- Clock ----

/// `MoonPhase` count of the table the spyglass reads (`1008146 + phase`).
const SECONDS_PER_UO_MINUTE: f64 = 5.0;

/// `Clock.GetTime(map, x, y, out hours, out minutes, out totalMinutes)`.
pub fn get_time(w: &World, map: Option<MapId>, x: i32) -> (i32, i32, i32) {
    let start = DateTime::from_ymd(1997, 9, 1).unwrap_or(DateTime::MIN);
    let mut total = ((w.now - start).total_seconds() / SECONDS_PER_UO_MINUTE) as i32;
    if let Some(m) = map {
        total += i32::from(m.0) * 320;
    }
    // Really on OSI this must be by subserver
    total += x / 16;
    (total / 60 % 24, total % 60, total)
}

/// `Clock.GetTime(map, x, y, out generalNumber, out exactTime)`.
pub fn get_time_text(w: &World, map: Option<MapId>, x: i32) -> (i32, String) {
    let (mut hours, minutes, _) = get_time(w, map, x);
    let general = match hours {
        20.. => 1042957, // It's late at night
        16.. => 1042956, // It's early in the evening
        13.. => 1042955, // It's the afternoon
        12.. => 1042954, // It's around noon
        8.. => 1042953,  // It's late in the morning
        4.. => 1042952,  // It's early in the morning
        1.. => 1042951,  // It's the middle of the night
        _ => 1042950,    // 'Tis the witching hour. 12 Midnight.
    };
    hours %= 12;
    if hours == 0 {
        hours = 12;
    }
    (general, format!("{hours}:{minutes:02}"))
}

/// `Clock.GetMoonPhase(map, x, y)`.
pub fn get_moon_phase(w: &World, map: Option<MapId>, x: i32) -> i32 {
    let (_, _, mut total) = get_time(w, map, x);
    if let Some(m) = map {
        total /= 10 + i32::from(m.0) * 20;
    }
    total % 8
}

fn clock_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some((map, x)) = w.mobile(from).map(|m| (m.map, m.location.x)) else {
        return;
    };
    let (general, exact) = get_time_text(w, map, x);
    item_messages::send_localized_message_to(w, id, from, general, "");
    item_messages::send_localized_message_to(w, id, from, 1042958, &exact); // ~1_TIME~ to be exact
}

macro_rules! clocks {
    ($($name:ident, $parent:expr, $full:literal, gid $gid:expr, $ctor_arg:literal, $enc:expr;)*) => {$(
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| ops::new_item_with_id(w, &$name, $gid)),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                write_version(out, $enc);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                read_version(r, $enc)?;
                Ok(())
            },
            ..$parent
        };
    )*};
}

/// `Server.Items.Clock`.
pub static CLOCK: ItemType = ItemType {
    full_name: "Server.Items.Clock",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_clock(w, 0x104B)),
    default_weight: crate::class_props::default_weight,
    on_double_click: clock_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new Clock(itemID)`.
pub fn new_clock(w: &mut World, item_id: i32) -> ItemId {
    ops::new_item_with_id(w, &CLOCK, item_id)
}

clocks! {
    CLOCK_RIGHT, CLOCK, "Server.Items.ClockRight", gid 0x104B, "", false;
    CLOCK_LEFT, CLOCK, "Server.Items.ClockLeft", gid 0x104C, "", false;
}

// ---- Spyglass ----

/// The quest side of the spyglass (`muo_engines::quests`).
#[derive(Default)]
pub struct SpyglassHooks {
    /// `player.Quest` is a `WitchApprenticeQuest` wanting the star chart: given the hour at the player's location.
    pub star_chart: Option<fn(&mut World, MobileId, i32)>,
}

fn spyglass_double_click(w: &mut World, _id: ItemId, from: MobileId) {
    // You peer into the heavens, seeking the moons...
    deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1008155);
    let Some(x) = w.mobile(from).map(|m| m.location.x) else {
        return;
    };
    for (map, prefix) in [(MapId(1), "Trammel : "), (MapId(0), "Felucca : ")] {
        let number = 1008146 + get_moon_phase(w, Some(map), x);
        let (serial, body) = w
            .mobile(from)
            .map_or((muo_primitives::Serial::new(0), 0), |m| {
                (m.serial, m.cur_body().0)
            });
        if let Some(ns) = deps::net_state_of(w, from) {
            muo_server::packets::message::send_message_localized_affix(
                w,
                ns,
                serial,
                body,
                MessageType::REGULAR,
                0x3B2,
                3,
                number,
                "",
                AffixType::PREPEND,
                prefix,
                "",
            );
        }
    }
    if w.mobile(from).is_some_and(|m| m.player) {
        let map = w.mobile(from).and_then(|m| m.map);
        let (hours, _, _) = get_time(w, map, x);
        if let Some(f) = w
            .resources
            .get::<SpyglassHooks>()
            .and_then(|h| h.star_chart)
        {
            f(w, from, hours);
        }
    }
}

/// `Server.Items.Spyglass`.
pub static SPYGLASS: ItemType = ItemType {
    full_name: "Server.Items.Spyglass",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| crate::simple::from_template(w, &SPYGLASS, "Server.Items.Spyglass")),
    default_weight: crate::class_props::default_weight,
    on_double_click: spyglass_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
static LOCKPICK_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: INT,
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_lockpick(w, v[0].i32())),
}];
static CLOCK_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "itemID",
        kind: INT,
        default: Some(Value::Int(0x104B)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_clock(w, v[0].i32())),
}];

fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.Lockpick", &LOCKPICK_CTOR);
    add_ctors(w, "Server.Items.Clock", &CLOCK_CTOR);
}

/// Every class of this file.
pub static ALL: &[&ItemType] = &[&LOCKPICK, &CLOCK, &CLOCK_RIGHT, &CLOCK_LEFT, &SPYGLASS];

/// Registers the small classes.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.Lockpick.Configure",
        configure,
    );
}
