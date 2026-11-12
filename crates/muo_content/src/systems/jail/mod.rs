//! The jail system: `[jail`, `[unjail`,
//! `[jailinfo` and `[jailrecord`, escalating jail time per offense, and the region that holds the jailed.
//!
//! The jail records are saved by a `GenericPersistence` named "Jail" (priority 3) in the layout the generated
//! `JailRecord.Serialize` writes.

use muo_primitives::dotnet::IgnoreCase;
use std::rc::Rc;

use indexmap::{IndexMap, IndexSet};
use muo_io::serialization::reader::GenericReader;
use muo_io::serialization::writer::GenericWriter;
use muo_primitives::{DateTime, Point3D, Serial, TimeSpan, utility};
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::MobileId;
use muo_server::interfaces::SpellId;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::messages as mm;
use muo_server::persistence::{GenericPayload, GenericPersistence};
use muo_server::region::{RegionId, RegionType};
use muo_server::skills::SkillName;
use muo_server::timer::{self, TimerHandle};

use crate::creature;
use crate::player_mobile::{self, PLAYER_MOBILE};
use crate::spells::TravelCheckType;
use crate::systems::regions::base_region::{
    BASE_REGION, BASE_REGION_FAMILY, BaseRegionType, TravelResult,
    check_travel as base_check_travel_of,
};
use crate::systems::targets::command_log;

/// Felucca void.
pub const JAIL_LOCATIONS: [Point3D; 10] = [
    Point3D::new(5276, 1164, 0),
    Point3D::new(5286, 1164, 0),
    Point3D::new(5296, 1164, 0),
    Point3D::new(5306, 1164, 0),
    Point3D::new(5276, 1174, 0),
    Point3D::new(5286, 1174, 0),
    Point3D::new(5296, 1174, 0),
    Point3D::new(5306, 1174, 0),
    Point3D::new(5283, 1184, 0),
    Point3D::new(5304, 1184, 0),
];

/// `JailMap`; change this for custom maps.
pub const JAIL_MAP: MapId = MapId::FELUCCA;
/// Britain Bank.
pub const RELEASE_LOCATION: Point3D = Point3D::new(1444, 1697, 10);
pub const RELEASE_MAP: MapId = MapId::FELUCCA;

/// Jail time scales from 5 minutes to 12 hours based on the number of offenses.
pub const MIN_JAIL_TIME: TimeSpan = TimeSpan::from_mins(5);
pub const MAX_JAIL_TIME: TimeSpan = TimeSpan::from_hours(12);
const JAIL_RECORD_COOLDOWN: TimeSpan = TimeSpan::from_secs(30);

/// `JailRecord`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JailRecord {
    pub jail_count: i32,
    pub last_jailed: DateTime,
    pub jail_end_time: DateTime,
    pub last_jail_reason: Option<String>,
    pub jailed_by: Option<MobileId>,
}

impl JailRecord {
    /// `IsCurrentlyJailed`.
    pub fn is_currently_jailed(&self, now: DateTime) -> bool {
        self.jail_end_time > now
    }

    /// The generated `Serialize`: version 0, then the five fields.
    pub fn serialize(&self, w: &World, writer: &mut dyn GenericWriter) {
        use muo_server::serialization_ext::WriteEntityExt as _;
        writer.write_encoded_int(0); // version
        writer.write_i32(self.jail_count);
        writer.write_i64(self.last_jailed.ticks());
        writer.write_i64(self.jail_end_time.ticks());
        writer.write_opt_string(self.last_jail_reason.as_deref());
        writer.write_mobile(w, self.jailed_by);
    }

    /// The generated `Deserialize`.
    pub fn deserialize(w: &World, reader: &mut dyn GenericReader) -> muo_io::Result<JailRecord> {
        use muo_server::serialization_ext::ReadEntityExt as _;
        let _version = reader.read_encoded_int()?;
        Ok(JailRecord {
            jail_count: reader.read_i32()?,
            last_jailed: DateTime::from_ticks(reader.read_date_time_ticks()?),
            jail_end_time: DateTime::from_ticks(reader.read_date_time_ticks()?),
            last_jail_reason: reader.read_string()?,
            jailed_by: reader.read_mobile(w)?,
        })
    }
}

/// The statics of `JailSystem`.
#[derive(Default)]
pub struct JailState {
    currently_being_jailed: IndexSet<MobileId>,
    records: IndexMap<MobileId, JailRecord>,
    timers: IndexMap<MobileId, TimerHandle>,
    record_cooldowns: IndexMap<MobileId, DateTime>,
}

fn state(w: &mut World) -> &mut JailState {
    w.resources.get_mut::<JailState>()
}

/// What the Gumps crate shows for `JailRecordGump`: the strings `BuildStrings` fills.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JailRecordGumpText {
    /// (text, color, font size) per slot.
    pub player_name: (String, &'static str, u8),
    pub jail_count: (String, &'static str, u8),
    pub last_jailed: (String, &'static str, u8),
    pub jail_reason: (String, &'static str, u8),
    pub jail_status: (String, &'static str, u8),
    /// An empty slot when not jailed.
    pub jail_time: (String, &'static str, u8),
}

/// `JailRecordGump.BuildStrings`.
pub fn jail_record_gump_text(
    player_name: &str,
    record: &JailRecord,
    now: DateTime,
) -> JailRecordGumpText {
    let last_jailed = if record.last_jailed > DateTime::MIN {
        let (y, mo, d) = record.last_jailed.ymd();
        format!(
            "Last Jailed: {y:04}/{mo:02}/{d:02} {:02}:{:02}",
            record.last_jailed.hour(),
            record.last_jailed.minute()
        )
    } else {
        "Last Jailed: Never".to_string()
    };
    let jail_reason = match record.last_jail_reason.as_deref() {
        Some(r) if !r.trim().is_empty() => format!("Jail Reason: {r}"),
        _ => "Jail Reason: None given.".to_string(),
    };
    let (jail_status, jail_time) = if record.is_currently_jailed(now) {
        (
            ("Status: Currently Jailed".to_string(), "#FF6666", 5),
            (
                format!(
                    "Jail Time: {}",
                    (record.jail_end_time - now).format_compact(false)
                ),
                "#FF6666",
                5,
            ),
        )
    } else {
        (
            ("Status: Not Jailed".to_string(), "#00FF00", 5),
            (String::new(), "", 0),
        )
    };
    JailRecordGumpText {
        player_name: (format!("Player: {player_name}"), "#FFFF00", 5),
        jail_count: (format!("Jail Count: {}", record.jail_count), "#FFFFFF", 5),
        last_jailed: (last_jailed, "#FFFFFF", 5),
        jail_reason: (jail_reason, "#FFFFFF", 4),
        jail_status,
        jail_time,
    }
}

/// Systems this one reaches that another crate owns.
#[derive(Default)]
pub struct JailHooks {
    /// Shows `JailRecordGump(player, record)` to `to`.
    pub show_jail_record_gump: Option<fn(&mut World, MobileId, MobileId, &JailRecord)>,
}

/// `IsPlayerJailed(player)`: usable by other systems.
pub fn is_player_jailed(w: &World, player: MobileId) -> bool {
    w.resources
        .get::<JailState>()
        .and_then(|s| s.records.get(&player))
        .is_some_and(|r| r.is_currently_jailed(w.now))
}

/// `CalculateJailTime(jailCount)`: 5 minutes at the first offense up to 12 hours at the tenth.
pub fn calculate_jail_time(jail_count: i32) -> TimeSpan {
    let total = MIN_JAIL_TIME.ticks() as f64
        + f64::from(jail_count - 1) * (MAX_JAIL_TIME - MIN_JAIL_TIME).ticks() as f64 / 9.0;
    let ticks = (total as i64).clamp(MIN_JAIL_TIME.ticks(), MAX_JAIL_TIME.ticks());
    TimeSpan::from_ticks(ticks)
}

fn name_of(w: &World, m: MobileId) -> String {
    props_b::raw_name(w, m).unwrap_or_default()
}

fn send_hue(w: &mut World, to: MobileId, text: &str) {
    mm::send_message_hue(w, to, 0x35, text);
}

fn staff_players(w: &World) -> Vec<MobileId> {
    // `NetState.Instances` where `Mobile is PlayerMobile && AccessLevel >= Counselor`.
    let mut v = Vec::new();
    for ns in w.net_states.values() {
        if let Some(m) = ns.mobile
            && w.mobile(m).is_some_and(|x| {
                x.ty.is(&PLAYER_MOBILE) && x.access_level >= AccessLevel::Counselor
            })
        {
            v.push(m);
        }
    }
    v
}

/// `JailPlayer(from, player, reason = "")`.
pub fn jail_player(w: &mut World, from: MobileId, player: MobileId, reason: &str) {
    if !state(w).currently_being_jailed.insert(player) {
        return;
    }

    let now = w.now;
    let s = state(w);
    let record = s.records.entry(player).or_default();
    record.jail_count += 1;
    record.last_jailed = now;
    record.last_jail_reason = Some(reason.to_string());
    record.jailed_by = Some(from);
    let count = record.jail_count;

    let jail_time = calculate_jail_time(count);
    state(w)
        .records
        .get_mut(&player)
        .expect("just inserted")
        .jail_end_time = now + jail_time;

    props_a::set_frozen(w, player, true);
    send_hue(w, player, "You are being sent to jail!");
    muo_server::mobile::part2::effects::play_sound(w, player, 0x204);

    let name = name_of(w, player);
    command_log(
        w,
        from,
        &format!(
            "Player {name} jailed for: {reason} (Offense #{count}, {} minutes)",
            jail_time.total_minutes()
        ),
    );

    for staff in staff_players(w) {
        send_hue(
            w,
            staff,
            &format!(
                "Player {name} has been jailed for {} minutes. Reason: {reason} (Offense #{count})",
                jail_time.total_minutes()
            ),
        );
    }

    timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
        dismount_player(w, from, player, jail_time)
    });
}

fn dismount_player(w: &mut World, from: MobileId, player: MobileId, jail_time: TimeSpan) {
    if let Some(mount) = props_b::mount(w, player) {
        muo_server::mobile::part2::deps::mount_clear_rider(w, mount);
        send_hue(w, player, "You have been dismounted.");
        if creature::is_creature(w, mount) {
            if creature::cd(w, mount).is_some_and(|c| c.summoned) {
                creature::misc::dispel(w, mount, mount);
            } else {
                // Stable the pet
                creature::edit(w, mount, |c| {
                    c.control_target = None;
                    c.control_order = creature::enums::OrderType::Stay;
                });
                muo_server::mobile::part2::items::internalize(w, mount);

                creature::control::set_control_master(w, mount, None);

                creature::edit(w, mount, |c| {
                    c.is_stabled = true;
                    c.stabled_by = Some(from);
                });
                player_mobile::add_stabled(w, player, mount);
            }
        }
    }

    let name = name_of(w, player);
    command_log(
        w,
        from,
        &format!("Player {name} dismounted before jail teleport"),
    );

    timer::start(w, TimeSpan::from_secs(3), TimeSpan::ZERO, 1, move |w| {
        teleport_to_jail(w, from, player, jail_time)
    });
}

fn teleport_to_jail(w: &mut World, from: MobileId, player: MobileId, jail_time: TimeSpan) {
    let jail_location = *utility::random_element(&mut w.rng, &JAIL_LOCATIONS).expect("not empty");

    mb::move_to_world(w, player, jail_location, JAIL_MAP);

    send_hue(w, player, "Use [jailrecord to pull up your record.");
    send_hue(
        w,
        player,
        "Please contact staff if you believe this was a mistake.",
    );

    let name = name_of(w, player);
    command_log(
        w,
        from,
        &format!("Player {name} teleported to jail at {jail_location}"),
    );

    timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
        unfreeze_player(w, from, player, jail_time)
    });
}

fn unfreeze_player(w: &mut World, from: MobileId, player: MobileId, jail_time: TimeSpan) {
    props_a::set_frozen(w, player, false);

    let name = name_of(w, player);
    command_log(w, from, &format!("Player {name} unfrozen in jail"));

    state(w).currently_being_jailed.shift_remove(&player);

    let release = timer::start(w, jail_time, TimeSpan::ZERO, 1, move |w| {
        release_player(w, Some(from), player)
    });
    state(w).timers.insert(player, release);
}

/// `ReleasePlayer(from, player)`; `from` is null for a release found at load.
pub fn release_player(w: &mut World, from: Option<MobileId>, player: MobileId) {
    let now = w.now;
    if let Some(r) = state(w).records.get_mut(&player) {
        r.jail_end_time = now;
    }

    state(w).timers.shift_remove(&player);

    // Freeze player for release sequence
    props_a::set_frozen(w, player, true);
    send_hue(w, player, "You have been released from jail!");
    muo_server::mobile::part2::effects::play_sound(w, player, 0x1FF);

    let name = name_of(w, player);
    if let Some(f) = from {
        command_log(
            w,
            f,
            &format!("Player {name} released from jail, starting teleport sequence"),
        );
    }

    for staff in staff_players(w) {
        send_hue(
            w,
            staff,
            &format!("Player {name} has been released from jail."),
        );
    }

    timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
        teleport_from_jail(w, from, player)
    });
}

fn teleport_from_jail(w: &mut World, from: Option<MobileId>, player: MobileId) {
    mb::move_to_world(w, player, RELEASE_LOCATION, RELEASE_MAP);

    if from.is_some() {
        let name = name_of(w, player);
        command_log(
            w,
            player,
            &format!("Player {name} teleported from jail to {RELEASE_LOCATION}"),
        );
    }

    timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
        unfreeze_from_release(w, from, player)
    });
}

fn unfreeze_from_release(w: &mut World, from: Option<MobileId>, player: MobileId) {
    props_a::set_frozen(w, player, false);
    send_hue(w, player, "Welcome back!");
    send_hue(w, player, "Please follow the shard rules.");
    send_hue(w, player, "Have a nice day!");

    if let Some(f) = from {
        let name = name_of(w, player);
        command_log(w, f, &format!("Player {name} unfrozen after jail release"));
    }
}

// ---- commands ---------------------------------------------------------------------------------------------------

/// `Serial.TryParse(text)`: decimal, or hex with `0x`.
fn parse_serial(text: &str) -> Option<Serial> {
    let t = text.trim();
    match t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        Some(hex) => u32::from_str_radix(hex, 16).ok(),
        None => t.parse::<u32>().ok(),
    }
    .map(Serial::new)
}

/// The player named by a serial or (case-insensitively) a raw name.
fn find_player(w: &World, name: &str) -> Option<MobileId> {
    let serial = parse_serial(name);
    w.mobiles.iter().find_map(|(&id, m)| {
        let is_pm = m.ty.is(&PLAYER_MOBILE) && !w.detaching.contains(&m.serial);
        (is_pm && (Some(m.serial) == serial || m.name.as_deref().is_some_and(|n| n.eq_ic(name))))
            .then_some(id)
    })
}

/// `[Usage("Jail <player> [reason]")]`
fn jail_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() < 1 {
        send_hue(w, from, "Usage: [jail <player> [reason]");
        return;
    }

    let reason = e.get_string(1);
    let player_name = e.get_string(0);
    let Some(player) = find_player(w, &player_name) else {
        send_hue(w, from, &format!("Player '{player_name}' not found."));
        return;
    };

    if w.mobile(player)
        .is_some_and(|m| m.access_level > AccessLevel::Player)
    {
        send_hue(w, from, "You cannot jail staff members.");
        return;
    }

    let name = name_of(w, player);
    if is_player_jailed(w, player) {
        send_hue(w, from, &format!("Player {name} is already jailed."));
        return;
    }

    jail_player(w, from, player, &reason);
    send_hue(
        w,
        from,
        &format!("Player {name} is being jailed. Reason: {reason}"),
    );
}

/// `[Usage("Unjail <player>")]`
fn unjail_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() < 1 {
        send_hue(w, from, "Usage: [unjail <player>");
        return;
    }

    let player_name = e.get_string(0);
    let Some(player) = find_player(w, &player_name) else {
        send_hue(w, from, &format!("Player '{player_name}' not found."));
        return;
    };

    let name = name_of(w, player);
    if !is_player_jailed(w, player) {
        send_hue(w, from, &format!("Player {name} is not currently jailed."));
        return;
    }

    if let Some(mut t) = state(w).timers.shift_remove(&player) {
        t.cancel(w);
    }

    release_player(w, Some(from), player);
    send_hue(
        w,
        from,
        &format!("Player {name} has been manually released from jail."),
    );
}

fn show_record(w: &mut World, to: MobileId, player: MobileId) {
    let record = state(w).records.get(&player).cloned().unwrap_or_default();
    if let Some(f) = w
        .resources
        .get::<JailHooks>()
        .and_then(|h| h.show_jail_record_gump)
    {
        f(w, to, player, &record);
    }
}

/// `[Usage("JailInfo <player>")]`
fn jail_info_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() < 1 {
        send_hue(w, from, "Usage: [jailinfo <player>");
        return;
    }

    let player_name = e.get_string(0);
    let Some(player) = find_player(w, &player_name) else {
        send_hue(w, from, &format!("Player '{player_name}' not found."));
        return;
    };

    show_record(w, from, player);
}

/// `[Usage("JailRecord")]`
fn my_jail_info_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let player = e.mobile;

    if let Some(&cooldown) = state(w).record_cooldowns.get(&player) {
        let since = w.now - cooldown;
        if since < JAIL_RECORD_COOLDOWN {
            let remaining = JAIL_RECORD_COOLDOWN - since;
            send_hue(
                w,
                player,
                &format!("You must wait {:.0} seconds.", remaining.total_seconds()),
            );
            return;
        }
    }

    let now = w.now;
    state(w).record_cooldowns.insert(player, now);

    show_record(w, player, player);
}

// ---- persistence ------------------------------------------------------------------------------------------------

struct JailPayload;

impl GenericPayload for JailPayload {
    fn name(&self) -> &str {
        "Jail"
    }

    fn serialize(&mut self, w: &World, writer: &mut dyn GenericWriter) {
        use muo_server::serialization_ext::WriteEntityExt as _;
        writer.write_encoded_int(0); // version
        let records = w.resources.get::<JailState>().map(|s| &s.records);
        writer.write_encoded_int(records.map_or(0, |r| r.len() as i32));
        for (&m, record) in records.into_iter().flatten() {
            writer.write_mobile(w, Some(m));
            record.serialize(w, writer);
        }
    }

    fn deserialize(&mut self, w: &mut World, reader: &mut dyn GenericReader) -> muo_io::Result<()> {
        use muo_server::serialization_ext::ReadEntityExt as _;
        let _version = reader.read_encoded_int()?;

        let count = reader.read_encoded_int()?;
        for _ in 0..count {
            let player = reader.read_mobile(w)?;
            let record = JailRecord::deserialize(w, reader)?;

            if let Some(player) = player {
                let jailed = record.is_currently_jailed(w.now);
                let jail_time = record.jail_end_time - w.now;
                let by = record.jailed_by;
                state(w).records.insert(player, record);
                if jailed {
                    let t = timer::start(w, jail_time, TimeSpan::ZERO, 1, move |w| {
                        release_player(w, by, player)
                    });
                    state(w).timers.insert(player, t);
                }
            }
        }
        Ok(())
    }
}

/// `JailSystem.Configure()`: makes the persistence (registered by its constructor) and the commands.
pub fn configure(w: &mut World) {
    *w.resources.get_mut::<JailState>() = JailState::default();
    w.persistence
        .register(3, Box::new(GenericPersistence::new(JailPayload)));
    #[allow(clippy::type_complexity)] // a callback type, spelled out
    let c = |f: fn(&mut World, &mut CommandEventArgs)| -> Rc<dyn Fn(&mut World, &mut CommandEventArgs)> { Rc::new(f) };
    CommandSystem::register(w, "Jail", AccessLevel::GameMaster, &[], c(jail_on_command));
    CommandSystem::register(
        w,
        "Unjail",
        AccessLevel::GameMaster,
        &[],
        c(unjail_on_command),
    );
    CommandSystem::register(
        w,
        "JailInfo",
        AccessLevel::Counselor,
        &[],
        c(jail_info_on_command),
    );
    CommandSystem::register(
        w,
        "JailRecord",
        AccessLevel::Player,
        &[],
        c(my_jail_info_on_command),
    );
}

// ---- JailRegion -------------------------------------------------------------------------------------------------

fn message(w: &mut World, from: MobileId, text: &str) {
    mm::send_message(w, from, text);
}

fn is_player_level(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.access_level == AccessLevel::Player)
}

fn jail_check_travel(
    w: &mut World,
    id: RegionId,
    m: MobileId,
    loc: Point3D,
    t: TravelCheckType,
) -> TravelResult {
    if is_player_level(w, m) {
        // You'll need a better jailbreak plan than that!
        return TravelResult {
            allowed: false,
            message: Some(muo_io::text::TextDefinition::of_number(1114345)),
        };
    }
    // base.CheckTravel: the parent BaseRegion's answer
    if let Some(parent) = w.region(id).and_then(|r| r.parent)
        && let Some(res) = base_check_travel_of(w, parent, m, loc, t)
    {
        return res;
    }
    TravelResult::allow()
}

/// `AllowBeneficial` and `AllowHarmful`: a player may do neither in jail.
fn jail_allow(w: &mut World, from: MobileId) -> bool {
    if is_player_level(w, from) {
        message(w, from, "You may not do that in jail.");
        return false;
    }
    true
}

pub static JAIL_FAMILY: BaseRegionType = BaseRegionType {
    check_travel: jail_check_travel,
    ..BASE_REGION_FAMILY
};

/// `JailRegion`.
pub static JAIL_REGION: RegionType = RegionType {
    name: "JailRegion",
    parent: Some(&BASE_REGION),
    allow_gain: |w, _, m, _: SkillName, _| {
        w.mobile(m)
            .is_some_and(|m| m.access_level > AccessLevel::Player)
    },
    allow_beneficial: |w, _, from, _| jail_allow(w, from),
    allow_harmful: |w, _, from, _| jail_allow(w, from),
    allow_housing: |_, _, _, _| false,
    alter_light_level: |_, _, _, global, _| *global = crate::misc::light_cycle::JAIL_LEVEL,
    on_begin_spell_cast: |w, _, from, _: SpellId| {
        if is_player_level(w, from) {
            mm::send_localized_message(w, from, 502629, "", 0x3B2); // You cannot cast spells here.
            return false;
        }
        true
    },
    on_skill_use: |w, _, from, _| {
        if is_player_level(w, from) {
            message(w, from, "You may not use skills in jail.");
            return false;
        }
        true
    },
    on_combatant_change: |w, _, from, _, _| {
        w.mobile(from)
            .is_some_and(|m| m.access_level > AccessLevel::Player)
    },
    family: Some(&JAIL_FAMILY),
    ..BASE_REGION
};

#[cfg(test)]
mod tests {
    use super::*;

    fn d(h: u32, mi: u32) -> DateTime {
        DateTime::from_ymd_hms(2026, 3, 22, h, mi, 0).unwrap()
    }

    #[test]
    fn jail_time_escalates_from_five_minutes_to_twelve_hours() {
        assert_eq!(calculate_jail_time(1), MIN_JAIL_TIME);
        assert_eq!(calculate_jail_time(10), MAX_JAIL_TIME);
        assert_eq!(calculate_jail_time(11), MAX_JAIL_TIME, "clamped");
        let second = calculate_jail_time(2);
        assert!(second > MIN_JAIL_TIME && second < calculate_jail_time(3));
        // 5 minutes + (12h - 5m) / 9
        assert_eq!(
            second.ticks(),
            MIN_JAIL_TIME.ticks() + (MAX_JAIL_TIME - MIN_JAIL_TIME).ticks() / 9
        );
        assert_eq!(calculate_jail_time(0), MIN_JAIL_TIME, "clamped below");
    }

    #[test]
    fn record_round_trips_in_the_generated_layout() {
        let mut w = World::new();
        let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        let record = JailRecord {
            jail_count: 3,
            last_jailed: d(1, 2),
            jail_end_time: d(5, 6),
            last_jail_reason: Some("spam".into()),
            jailed_by: Some(m),
        };
        let mut writer = muo_io::serialization::save_writer::SaveWriter::new(true);
        record.serialize(&w, &mut writer);
        let bytes = writer.into_vec();
        // version 0 (encoded int), then the int count.
        assert_eq!(&bytes[..5], &[0, 3, 0, 0, 0]);
        let mut reader = muo_io::serialization::save_reader::SaveReader::new(&bytes);
        let back = JailRecord::deserialize(&w, &mut reader).unwrap();
        assert_eq!(back, record);
    }

    #[test]
    fn gump_strings() {
        let now = d(12, 0);
        let empty = jail_record_gump_text("Bob", &JailRecord::default(), now);
        assert_eq!(empty.jail_count.0, "Jail Count: 0");
        assert_eq!(empty.last_jailed.0, "Last Jailed: Never");
        assert_eq!(empty.jail_reason.0, "Jail Reason: None given.");
        assert_eq!(
            empty.jail_status,
            ("Status: Not Jailed".to_string(), "#00FF00", 5)
        );
        assert!(empty.jail_time.0.is_empty());

        let jailed = JailRecord {
            jail_count: 2,
            last_jailed: d(11, 5),
            jail_end_time: d(13, 30),
            last_jail_reason: Some("griefing".into()),
            jailed_by: None,
        };
        let t = jail_record_gump_text("Bob", &jailed, now);
        assert_eq!(t.player_name.0, "Player: Bob");
        assert_eq!(t.last_jailed.0, "Last Jailed: 2026/03/22 11:05");
        assert_eq!(t.jail_reason.0, "Jail Reason: griefing");
        assert_eq!(t.jail_status.0, "Status: Currently Jailed");
        assert_eq!(t.jail_time.0, "Jail Time: 1h 30m");
    }

    #[test]
    fn region_rules_for_players_and_staff() {
        let mut w = muo_server::map::test_support::world();
        let r = muo_server::region::new_region(
            &mut w,
            &JAIL_REGION,
            Some("Jail"),
            MapId::FELUCCA,
            None,
            Some(50),
            vec![],
        );
        crate::regions::base_region::init_data(&mut w, r);
        muo_server::region::register(&mut w, r);
        let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        let rt = &JAIL_REGION;
        assert!(!(rt.allow_harmful)(&mut w, r, m, m));
        assert!(!(rt.allow_beneficial)(&mut w, r, m, m));
        assert!(!(rt.on_combatant_change)(&mut w, r, m, None, None));
        assert!(!(rt.allow_gain)(&w, r, m, SkillName::Magery, None));
        assert!(!(rt.on_skill_use)(&mut w, r, m, 1));
        let res = jail_check_travel(&mut w, r, m, Point3D::default(), TravelCheckType::RecallTo);
        assert_eq!(res.message.map(|t| t.number), Some(1114345));
        w.mobile_mut(m).unwrap().access_level = AccessLevel::GameMaster;
        assert!((rt.allow_harmful)(&mut w, r, m, m));
        assert!((rt.allow_gain)(&w, r, m, SkillName::Magery, None));
        assert!((rt.on_skill_use)(&mut w, r, m, 1));
        assert!(
            jail_check_travel(&mut w, r, m, Point3D::default(), TravelCheckType::RecallTo).allowed
        );
    }
}
