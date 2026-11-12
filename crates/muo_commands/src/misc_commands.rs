//! ShardTime, ResetLocalization, ResetRng, DragEffects, SkillsMenu, ResetStaffDress, StaffAccess, movement debug
//! commands, Wipe, Skills, VisibilityList and LoopStats.
//!
//! LoopStats needs an event loop profiler, which the server does not have: it registers nothing.

use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use std::rc::Rc;

use muo_primitives::{Point3D, Rectangle2D};
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::{send_localized_message, send_message};
use muo_server::skills::SkillName;
use muo_server::target::{TargetFlags, Targeted, begin_target};
use muo_server::world::World;

use crate::handlers::{broadcast_message, name_of};
use crate::hooks::CommandHooks;
use crate::logging::{self, format_entity};

type Handler = fn(&mut World, &mut CommandEventArgs);

pub fn configure(w: &mut World) {
    let mut reg = |name: &str, level: AccessLevel, aliases: &[&str], f: Handler| {
        CommandSystem::register(w, name, level, aliases, Rc::new(f));
    };
    reg("Time", AccessLevel::Player, &[], time_on_command);
    reg(
        "ResetLocalization",
        AccessLevel::Developer,
        &["ResetLoc"],
        reset_localization_on_command,
    );
    reg(
        "ResetRng",
        AccessLevel::Administrator,
        &[],
        reset_rng_on_command,
    );
    reg(
        "DragEffects",
        AccessLevel::Developer,
        &[],
        drag_effects_on_command,
    );
    reg("Skills", AccessLevel::Counselor, &[], skills_on_command);
    reg(
        "ResetStaffDress",
        AccessLevel::Counselor,
        &[],
        staff_dress_on_command,
    );
    reg(
        "StaffAccess",
        AccessLevel::Player,
        &[],
        staff_access_on_command,
    );
    reg(
        "MovementDebug",
        AccessLevel::GameMaster,
        &[],
        movement_debug_on_command,
    );
    reg(
        "MovementStats",
        AccessLevel::GameMaster,
        &[],
        movement_stats_on_command,
    );
    reg("Wipe", AccessLevel::GameMaster, &[], |w, e| {
        begin_wipe(w, e.mobile, WIPE_ITEMS | WIPE_MOBILES)
    });
    reg("WipeItems", AccessLevel::GameMaster, &[], |w, e| {
        begin_wipe(w, e.mobile, WIPE_ITEMS)
    });
    reg("WipeNPCs", AccessLevel::GameMaster, &[], |w, e| {
        begin_wipe(w, e.mobile, WIPE_MOBILES)
    });
    reg("WipeMultis", AccessLevel::GameMaster, &[], |w, e| {
        begin_wipe(w, e.mobile, WIPE_MULTIS)
    });
    reg(
        "SetSkill",
        AccessLevel::GameMaster,
        &[],
        set_skill_on_command,
    );
    reg(
        "GetSkill",
        AccessLevel::GameMaster,
        &[],
        get_skill_on_command,
    );
    reg(
        "SetAllSkills",
        AccessLevel::GameMaster,
        &[],
        set_all_skills_on_command,
    );
    reg("Vis", AccessLevel::Counselor, &[], vis_on_command);
    reg("VisList", AccessLevel::Counselor, &[], vis_list_on_command);
    reg(
        "VisClear",
        AccessLevel::Counselor,
        &[],
        vis_clear_on_command,
    );
    // `MovementThrottle.OnSpeedHackDetected += OnSpeedHackDetected`
    w.resources
        .get_mut::<muo_server::net_glue::SpeedHackListeners>()
        .0
        .push(Box::new(on_speed_hack_detected));
    // `[OnEvent(nameof(PlayerMobile.PlayerLoginEvent))] VisibilityList.OnLogin`
    muo_content::network::events::subscribe_ranked::<muo_content::network::events::PlayerLogin>(
        w,
        3,
        |w, e| {
            if let Some(d) = muo_content::player_mobile::data_mut(w, e.0) {
                d.visibility_list.clear();
            }
        },
    );
}

fn is_player_mobile(w: &World, m: MobileId) -> bool {
    muo_content::player_mobile::data(w, m).is_some()
}

/// `[Usage("Time")]`
fn time_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let text = w.now.to_string();
    send_message(w, e.mobile, &text);
}

/// `[Usage("ResetLocalization [<command>]")]`
fn reset_localization_on_command(w: &mut World, e: &mut CommandEventArgs) {
    // `Localization.Clear()`: the cliloc table is the engines crate's (`CommandHooks::clear_localization`).
    if let Some(f) = w.resources.get_mut::<CommandHooks>().clear_localization {
        f(w);
    }
    send_message(w, e.mobile, "Localization table cleared.");
}

/// `[Usage("ResetRng")]`
fn reset_rng_on_command(w: &mut World, _e: &mut CommandEventArgs) {
    w.rng = muo_primitives::BuiltInRng::new();
}

/// `[Usage("DragEffects [enable=false]")]`
fn drag_effects_on_command(w: &mut World, e: &mut CommandEventArgs) {
    use muo_server::mobile::part1::props_a::PropsAStatics;
    let from = e.mobile;
    if e.length() == 0 {
        let on = w.resources.get_mut::<PropsAStatics>().drag_effects;
        send_message(
            w,
            from,
            if on {
                "Drag effects are currently enabled."
            } else {
                "Drag effects are currently disabled."
            },
        );
    } else {
        let on = e.get_boolean(0);
        w.resources.get_mut::<PropsAStatics>().drag_effects = on;
        send_message(
            w,
            from,
            if on {
                "Drag effects have been enabled."
            } else {
                "Drag effects have been disabled."
            },
        );
    }
}

/// `[Usage("Skills")]`: a target (ground allowed) that opens the `SkillsGump` of a mobile.
fn skills_on_command(w: &mut World, e: &mut CommandEventArgs) {
    begin_target(w, e.mobile, -1, true, TargetFlags::NONE, |w, from, o| {
        let Targeted::Entity(EntityId::Mobile(m)) = o else {
            return;
        };
        // `SkillsGump` is a `muo_gumps` class, opened through `CommandHooks::skills_gump` (set in `a_l::hooks::configure`).
        if let Some(f) = w.resources.get_mut::<CommandHooks>().skills_gump {
            f(w, from, m);
        }
    });
}

/// `[Usage("ResetStaffDress")]`
fn staff_dress_on_command(w: &mut World, e: &mut CommandEventArgs) {
    use muo_server::mobile::part1::{props_a, props_b};
    let pm = e.mobile;
    if !is_player_mobile(w, pm) {
        return;
    }
    // pm.Race = Race.Human
    let human = w
        .resources
        .get::<muo_server::race::Races>()
        .and_then(|r| r.default_race().map(|r| r.id()));
    props_a::set_race(w, pm, human);
    props_a::set_karma(w, pm, 0);
    props_a::set_fame(w, pm, 0);
    props_b::set_kills(w, pm, 0);
    // `ShortTermMurders = 0`: PlayerMurderSystem.ManuallySetShortTermMurders (with its `ResetKillTime` and
    // `UpdateMurderContext`, the murder engine's).
    if let Some(f) = w
        .resources
        .get::<CommandHooks>()
        .and_then(|h| h.set_short_term_murders)
    {
        f(w, pm, 0);
    } else {
        w.resources
            .get_mut::<muo_content::player_mobile::tables::MurderTable>()
            .0
            .entry(pm)
            .or_default()
            .short_term_murders = 0;
    }
    props_b::set_body_mod(w, pm, muo_primitives::Body(0));
    props_b::set_body(w, pm, muo_primitives::Body(987));
    props_b::set_solid_hue_override(w, pm, -1);
    props_b::set_hue_mod(w, pm, -1);
    props_b::set_facial_hair_item_id(w, pm, 0);
    props_a::set_blessed(w, pm, true);
    props_b::set_display_guild_title(w, pm, false);
    if let Some(d) = muo_content::player_mobile::data_mut(w, pm) {
        d.set_flag(
            muo_content::player_mobile::PlayerFlag::DISPLAY_CHAMPION_TITLE,
            false,
        );
    }
    // if (pm.Mount != null) pm.Mount.Rider = null
    if let Some(mount) = props_b::mount(w, pm) {
        muo_server::mobile::part2::deps::mount_clear_rider(w, mount);
    }
    if let Some(ns) = w.mobile(pm).and_then(|m| m.net_state) {
        muo_server::packets::movement::send_speed_control(
            w,
            ns,
            muo_server::packets::movement::SpeedControlSetting::Mount,
        );
    }
    reset_staff_access(w, pm);
    let level = w.mobile(pm).map_or(AccessLevel::Player, |m| m.access_level);
    if level < AccessLevel::Administrator {
        let hue = w.mobile(pm).map_or(0, |m| (m.ty.hue)(w, pm));
        let human_ops = w
            .resources
            .get::<muo_server::race::Races>()
            .and_then(|r| r.default_race().map(|r| r.ops));
        let next = (hue + 1) & 0x3FFF;
        let clipped = human_ops.map_or(next, |ops| (ops.clip_skin_hue)(next));
        muo_server::mobile::base::set_hue(w, pm, clipped);
    }
    let items: Vec<ItemId> = w.mobile(pm).map(|m| m.items.clone()).unwrap_or_default();
    for item in items.into_iter().rev() {
        let Some(layer) = w.item(item).map(|i| i.layer) else {
            continue;
        };
        if layer == Layer::FacialHair {
            muo_server::item::base::delete(w, item);
        }
        if !matches!(
            layer,
            Layer::Backpack
                | Layer::Bank
                | Layer::Hair
                | Layer::Mount
                | Layer::ShopBuy
                | Layer::ShopResale
                | Layer::ShopSell
        ) {
            muo_server::mobile::part2::items::add_to_backpack(w, pm, item);
        }
    }
    if level > AccessLevel::Player {
        let robe = match w.resources.get_mut::<CommandHooks>().new_staff_robe {
            Some(f) => f(w, level),
            None => muo_content::factory::make_item(w, "Server.Items.StaffRobe"),
        };
        if let Some(robe) = robe {
            muo_server::mobile::part2::items::add_item(w, pm, robe);
        }
    }
}

// ---- StaffAccess ----------------------------------------------------------------------------------------------

fn access_level_by_string(s: &str) -> Option<AccessLevel> {
    Some(match s {
        "player" => AccessLevel::Player,
        "counselor" => AccessLevel::Counselor,
        "gamemaster" | "gm" => AccessLevel::GameMaster,
        "seer" => AccessLevel::Seer,
        "administrator" | "admin" => AccessLevel::Administrator,
        "developer" | "dev" => AccessLevel::Developer,
        "owner" => AccessLevel::Owner,
        _ => return None,
    })
}

fn account_tag(w: &World, m: MobileId) -> String {
    format!(
        "OriginalStaffAccess:{}",
        w.mobile(m)
            .map(|x| x.serial.to_string())
            .unwrap_or_default()
    )
}

fn set_account_tag(w: &mut World, m: MobileId, tag: &str, value: Option<&str>) {
    let Some(acct) = w.mobile(m).and_then(|x| x.account) else {
        return;
    };
    if let Some(a) = w
        .resources
        .get_mut::<muo_content::accounting::Accounts>()
        .get_mut(acct)
    {
        match value {
            Some(v) => a.add_tag(tag, v),
            None => a.remove_tag(tag),
        }
    }
}

/// `StaffAccess.ResetStaffAccess(this PlayerMobile)`.
pub fn reset_staff_access(w: &mut World, m: MobileId) {
    let Some(acct) = w.mobile(m).and_then(|x| x.account) else {
        return;
    };
    let tag = account_tag(w, m);
    let Some(original) = muo_content::accounting::account(w, acct)
        .and_then(|a| a.get_tag(&tag))
        .map(str::to_owned)
    else {
        return;
    };
    // `_accessLevelByString[...]` throws for an unknown string; the tag is only ever written by this command.
    let Some(level) = access_level_by_string(&original) else {
        return;
    };
    set_account_tag(w, m, &tag, None);
    muo_server::mobile::part1::props_a::set_access_level(w, m, level);
}

/// `[Usage("StaffAccess <access level>")]`
fn staff_access_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let m = e.mobile;
    let Some(acct) = w.mobile(m).and_then(|x| x.account) else {
        return;
    };
    let tag = account_tag(w, m);
    let original_str = muo_content::accounting::account(w, acct)
        .and_then(|a| a.get_tag(&tag))
        .map(str::to_owned);
    let original = original_str.as_deref().and_then(access_level_by_string);
    let current = w.mobile(m).map_or(AccessLevel::Player, |x| x.access_level);
    if original.is_none() && current == AccessLevel::Player {
        return;
    }
    let arg = e.get_string(0).trim().lower_inv();
    let parsed = if arg.is_empty() {
        None
    } else {
        access_level_by_string(&arg)
    };
    let valid = parsed.is_some();
    let mut new_level = parsed.unwrap_or(AccessLevel::Player);
    if !valid && original.is_none() {
        send_message(w, m, "Invalid access level specified.");
        send_message(w, m, "Usage: [staffaccess <access level>.");
        return;
    }
    if let Some(o) = original
        && (!valid || new_level == o)
    {
        set_account_tag(w, m, &tag, None);
        new_level = o;
        send_message(w, m, "Restoring original staff access...");
    }
    if original.unwrap_or(current) < new_level {
        send_message(
            w,
            m,
            &format!("You cannot set your staff access to {new_level:?}."),
        );
        return;
    }
    if original.is_none() {
        // Save the original access level
        set_account_tag(w, m, &tag, Some(&format!("{current:?}").lower_inv()));
    }
    muo_server::mobile::part1::props_a::set_access_level(w, m, new_level);
    send_message(w, m, &format!("Staff access set to {new_level:?}."));
}

// ---- MovementDebugCommands -------------------------------------------------------------------------------------

fn on_speed_hack_detected(
    w: &mut World,
    ns: muo_server::net_state::NetStateId,
    mobile: MobileId,
    rate: f32,
    verdict: muo_net::movement::DetectionVerdict,
    urgency: &str,
) {
    let hue = match urgency {
        "HIGH" => 0x25,
        "MEDIUM" => 0x35,
        _ => 0x3B,
    };
    let raw = muo_server::mobile::part1::props_b::raw_name(w, mobile).unwrap_or_default();
    let username = w
        .net_state(ns)
        .and_then(|d| d.account.clone())
        .unwrap_or_default();
    let (loc, map) = w
        .mobile(mobile)
        .map(|m| (m.location, m.map))
        .unwrap_or_default();
    let map_name = map
        .and_then(|m| w.maps.get(m))
        .map_or_else(String::new, |m| m.name().to_string());
    let text = format!(
        "[{urgency}] Speed hack: {raw} ({username}) Rate:{rate:.2} Verdict:{verdict:?} @ {loc} {map_name}"
    );
    broadcast_message(w, AccessLevel::Counselor, hue, &text);
}

fn movement_debug_on_command(w: &mut World, e: &mut CommandEventArgs) {
    begin_target(w, e.mobile, -1, false, TargetFlags::NONE, |w, from, o| {
        let Some((target, ns)) = online_target(w, o) else {
            send_message(w, from, "That is not a valid online player.");
            return;
        };
        let on = match w.net_state_mut(ns) {
            Some(d) => {
                d.movement.logging = !d.movement.logging;
                d.movement.logging
            }
            None => return,
        };
        let raw = muo_server::mobile::part1::props_b::raw_name(w, target).unwrap_or_default();
        let state = if on { "ENABLED" } else { "DISABLED" };
        send_message(
            w,
            from,
            &format!("Movement debug logging {state} for {raw}."),
        );
    });
    send_message(
        w,
        e.mobile,
        "Target a player to toggle movement debug logging.",
    );
}

fn online_target(w: &World, o: Targeted) -> Option<(MobileId, muo_server::net_state::NetStateId)> {
    let Targeted::Entity(EntityId::Mobile(m)) = o else {
        return None;
    };
    Some((m, w.mobile(m)?.net_state?))
}

fn movement_stats_on_command(w: &mut World, e: &mut CommandEventArgs) {
    begin_target(w, e.mobile, -1, false, TargetFlags::NONE, |w, from, o| {
        let Some((target, ns)) = online_target(w, o) else {
            send_message(w, from, "That is not a valid online player.");
            return;
        };
        let now = w.tick_count;
        let Some(d) = w.net_state_mut(ns) else { return };
        let mut st = std::mem::replace(&mut d.movement, muo_net::movement::MovementState::new(now));
        let s = w.movement_throttle.movement_stats(&mut st);
        if let Some(d) = w.net_state_mut(ns) {
            d.movement = st;
        }
        let raw = muo_server::mobile::part1::props_b::raw_name(w, target).unwrap_or_default();
        let lines = [
            format!("--- Movement Stats for {raw} ---"),
            format!(
                "Rate: {:.3} ({} samples) | Verdict: {:?} | Confidence: {:.0} %",
                s.rate,
                s.sample_count,
                s.verdict,
                s.confidence * 100.0
            ),
            format!(
                "RTT: avg={}ms last={}ms var={} stable={} samples={}",
                s.average_rtt,
                s.last_rtt,
                s.rtt_variance,
                py_bool(s.stable_connection),
                s.rtt_sample_count
            ),
            format!(
                "Queue: depth={} | Credit: {}ms",
                s.queue_depth, s.movement_credit
            ),
            format!(
                "Packet rate: {}/s (peak: {}/s)",
                s.current_packet_rate, s.peak_packet_rate
            ),
            format!(
                "Burst: size={} gap={}ms | Sustained: {}s",
                s.burst_size, s.preceding_gap, s.sustained_seconds
            ),
            format!(
                "Debug logging: {}",
                if s.debug_logging { "ON" } else { "OFF" }
            ),
        ];
        for l in lines {
            send_message(w, from, &l);
        }
    });
    send_message(w, e.mobile, "Target a player to view their movement stats.");
}

/// A bool printed as `True` / `False`.
fn py_bool(b: bool) -> &'static str {
    if b { "True" } else { "False" }
}

// ---- Wipe ------------------------------------------------------------------------------------------------------

/// `Wipe.WipeType`.
pub const WIPE_ITEMS: u8 = 1;
pub const WIPE_MOBILES: u8 = 2;
pub const WIPE_MULTIS: u8 = 4;

fn wipe_type_name(t: u8) -> String {
    // `WipeType.ToString()` of a [Flags] enum: `All` for 7, else the set names joined by ", ".
    if t == 7 {
        return "All".into();
    }
    let mut parts = Vec::new();
    for (bit, n) in [(1, "Items"), (2, "Mobiles"), (4, "Multis")] {
        if t & bit != 0 {
            parts.push(n);
        }
    }
    if parts.is_empty() {
        "0".into()
    } else {
        parts.join(", ")
    }
}

pub fn begin_wipe(w: &mut World, from: MobileId, wipe: u8) {
    crate::generic::implementors::begin_bounding_box(
        w,
        from,
        Rc::new(move |w, map, start, end| do_wipe(w, from, map, start, end, wipe)),
    );
}

pub fn do_wipe(
    w: &mut World,
    from: MobileId,
    map: muo_server::map::MapId,
    start: Point3D,
    end: Point3D,
    wipe: u8,
) {
    let level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    let map_name = w
        .maps
        .get(map)
        .map_or_else(String::new, |m| m.name().to_string());
    let text = format!(
        "{level:?} {} wiping from {start} to {end} in {} ({map_name})",
        format_entity(w, EntityId::Mobile(from)),
        wipe_type_name(wipe)
    );
    logging::write_line(w, from, &text);
    let rect = Rectangle2D::new(start.x, start.y, end.x - start.x + 1, end.y - start.y + 1);
    let mut to_delete: Vec<EntityId> = Vec::new();
    if wipe & WIPE_MOBILES != 0 {
        for m in muo_server::map::enumerate::mobiles_in_bounds(w, Some(map), rect, true) {
            if !w.mobile(m).is_some_and(|x| x.player) {
                to_delete.push(EntityId::Mobile(m));
            }
        }
    }
    if wipe & (WIPE_ITEMS | WIPE_MULTIS) != 0 {
        for i in muo_server::map::enumerate::items_in_bounds(w, Some(map), rect, true) {
            if muo_content::props::is_class(w, EntityId::Item(i), "Server.Items.BaseMulti") {
                if wipe & WIPE_MULTIS != 0 {
                    to_delete.push(EntityId::Item(i));
                }
            } else if wipe & WIPE_ITEMS != 0 && !is_house_sign(w, i) {
                to_delete.push(EntityId::Item(i));
            }
        }
    }
    for e in to_delete {
        match e {
            EntityId::Item(i) => muo_server::item::base::delete(w, i),
            EntityId::Mobile(m) => muo_server::mobile::base::delete(w, m),
        }
    }
}

fn is_house_sign(w: &World, i: ItemId) -> bool {
    muo_content::props::is_class(w, EntityId::Item(i), "Server.Multis.HouseSign")
}

// ---- Skills ----------------------------------------------------------------------------------------------------

/// `Enum.TryParse<SkillName>(text, true, out skill)`: a name (any case) or a number.
pub fn parse_skill_name(text: &str) -> Option<SkillName> {
    let t = text.trim();
    if let Ok(n) = t.parse::<i32>() {
        // Enum.TryParse accepts any number; one without a name is not a skill the table knows.
        return SkillName::from_i32(n);
    }
    (0..)
        .map_while(SkillName::from_i32)
        .find(|s| format!("{s:?}").eq_ic(t))
}

fn set_skill_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() != 2 {
        send_message(w, from, "SetSkill <skill name> <value>");
        return;
    }
    match parse_skill_name(&e.get_string(0)) {
        Some(skill) => skill_target(w, from, skill, Some(e.get_double(1))),
        None => send_localized_message(w, from, 1005631, "", 0x3B2), // You have specified an invalid skill to set.
    }
}

fn get_skill_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() != 1 {
        send_message(w, from, "GetSkill <skill name>");
        return;
    }
    match parse_skill_name(&e.get_string(0)) {
        Some(skill) => skill_target(w, from, skill, None),
        None => send_message(w, from, "You have specified an invalid skill to get."),
    }
}

/// `SkillTarget`.
fn skill_target(w: &mut World, from: MobileId, skill: SkillName, set: Option<f64>) {
    begin_target(w, from, -1, false, TargetFlags::NONE, move |w, from, o| {
        let Targeted::Entity(EntityId::Mobile(targ)) = o else {
            send_message(w, from, "That does not have skills!");
            return;
        };
        if let Some(v) = set {
            muo_server::skills::set_base(w, targ, skill, v);
            logging::log_change_property(
                w,
                from,
                EntityId::Mobile(targ),
                &format!("{skill:?}.Base"),
                &muo_content::props::format_double(v),
            );
        }
        let value = muo_server::skills::value(w, targ, skill);
        let base = muo_server::skills::base(w, targ, skill);
        send_message(
            w,
            from,
            &format!(
                "{skill:?} : {} (Base: {})",
                muo_content::props::format_double(value),
                muo_content::props::format_double(base)
            ),
        );
    });
}

fn set_all_skills_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() != 1 {
        send_message(w, from, "SetAllSkills <value>");
        return;
    }
    let value = e.get_double(0);
    begin_target(w, from, -1, false, TargetFlags::NONE, move |w, from, o| {
        let Targeted::Entity(EntityId::Mobile(targ)) = o else {
            send_message(w, from, "That does not have skills!");
            return;
        };
        let len = w
            .resources
            .get::<muo_server::skills::SkillTable>()
            .map_or(0, |t| t.info.len());
        for s in (0..len as i32).map_while(SkillName::from_i32) {
            muo_server::skills::set_base(w, targ, s, value);
        }
        logging::log_change_property(
            w,
            from,
            EntityId::Mobile(targ),
            "EverySkill.Base",
            &muo_content::props::format_double(value),
        );
    });
}

// ---- VisibilityList --------------------------------------------------------------------------------------------

fn vis_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if !is_player_mobile(w, from) {
        return;
    }
    begin_target(w, from, -1, false, TargetFlags::NONE, vis_on_target);
    send_message(
        w,
        from,
        "Select person to add or remove from your visibility list.",
    );
}

fn vis_on_target(w: &mut World, from: MobileId, o: Targeted) {
    use muo_server::packets::{entity::send_remove_entity, mobile::send_mobile_incoming};
    let Targeted::Entity(EntityId::Mobile(targ)) = o else {
        send_message(w, from, "Add only mobiles to your visibility list.");
        return;
    };
    if !is_player_mobile(w, from) {
        send_message(w, from, "Add only mobiles to your visibility list.");
        return;
    }
    let (my_level, targ_level) = (
        w.mobile(from)
            .map_or(AccessLevel::Player, |m| m.access_level),
        w.mobile(targ)
            .map_or(AccessLevel::Player, |m| m.access_level),
    );
    if targ_level > my_level {
        send_message(w, from, "They can already see you!");
        return;
    }
    let targ_name = name_of(w, targ);
    let removed = muo_content::player_mobile::data_mut(w, from).is_some_and(|d| {
        if let Some(i) = d.visibility_list.iter().position(|x| *x == targ) {
            d.visibility_list.remove(i);
            true
        } else {
            d.visibility_list.push(targ);
            false
        }
    });
    send_message(
        w,
        from,
        &if removed {
            format!("{targ_name} has been removed from your visibility list.")
        } else {
            format!("{targ_name} has been added to your visibility list.")
        },
    );
    let (tloc, floc) = (
        w.mobile(targ).map(|m| m.location).unwrap_or_default(),
        w.mobile(from).map(|m| m.location).unwrap_or_default(),
    );
    if muo_server::mobile::part1::combat::in_update_range(tloc, floc)
        && let Some(ns) = w.mobile(targ).and_then(|m| m.net_state)
    {
        if muo_server::mobile::base::can_see_object(w, targ, Some(EntityId::Mobile(from))) {
            send_mobile_incoming(w, ns, targ, from);
            muo_server::mobile::base::send_opl_packet_to(w, from, ns);
            let items: Vec<ItemId> = w.mobile(from).map(|m| m.items.clone()).unwrap_or_default();
            for i in items {
                muo_server::item::base::send_opl_packet_to(w, i, ns);
            }
        } else {
            let serial = w.mobile(from).map(|m| m.serial).unwrap_or_default();
            send_remove_entity(w, ns, serial);
        }
    }
}

fn vis_list_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let pm = e.mobile;
    let Some(list) = muo_content::player_mobile::data(w, pm).map(|d| d.visibility_list.clone())
    else {
        return;
    };
    if list.is_empty() {
        send_message(w, pm, "Your visibility list is empty.");
        return;
    }
    let n = list.len();
    send_message(
        w,
        pm,
        &if n == 1 {
            format!("You are visible to {n} mobile:")
        } else {
            format!("You are visible to {n} mobiles:")
        },
    );
    for (i, m) in list.into_iter().enumerate() {
        let name = name_of(w, m);
        send_message(w, pm, &format!("#{}: {name}", i + 1));
    }
}

fn vis_clear_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let pm = e.mobile;
    let Some(list) =
        muo_content::player_mobile::data_mut(w, pm).map(|d| std::mem::take(&mut d.visibility_list))
    else {
        return;
    };
    send_message(w, pm, "Your visibility list has been cleared.");
    let ploc = w.mobile(pm).map(|m| m.location).unwrap_or_default();
    let serial = w.mobile(pm).map(|m| m.serial).unwrap_or_default();
    for m in list {
        let mloc = w.mobile(m).map(|x| x.location).unwrap_or_default();
        if !muo_server::mobile::base::can_see_object(w, m, Some(EntityId::Mobile(pm)))
            && muo_server::mobile::part1::combat::in_update_range(mloc, ploc)
            && let Some(ns) = w.mobile(m).and_then(|x| x.net_state)
        {
            muo_server::packets::entity::send_remove_entity(w, ns, serial);
        }
    }
}
