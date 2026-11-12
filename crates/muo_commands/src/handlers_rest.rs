//! The rest of `CommandHandlers`: Go, DropHolding, GetFollowers, Cast,
//! Stuck, Move, Client, Bank, ViewEquip, Light, Stats. (`handlers.rs` has the others.)
//!
//! `GoGump`, `ClientGump`, `StuckMenu` and `DesignContext.Check` are other crates' classes
//! (`CommandHooks::{go_gump, client_gump, stuck_menu, design_context_check}`); `Sextant.ReverseLookup` is
//! `CommandHooks::sextant_reverse_lookup` (still unset: Sextant); `PageQueue` is `CommandHooks::page_handler`. Set by
//! `muo_gumps::a_l::hooks::configure`, `muo_engines::help::configure` and `muo_engines::wiring2`; unset, the command
//! says so (or, for the checks, allows the action).

use muo_primitives::dotnet::IgnoreCase;
use std::rc::Rc;

use muo_primitives::{Point3D, Serial};
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::menus::{self, ItemListEntry, Menu};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::{send_localized_message, send_message};
use muo_server::target::{TargetFlags, Targeted, begin_target};
use muo_server::world::World;

use crate::hooks::CommandHooks;
use crate::logging;

type Handler = fn(&mut World, &mut CommandEventArgs);

pub fn configure(w: &mut World) {
    let mut reg = |name: &str, level: AccessLevel, f: Handler| {
        CommandSystem::register(w, name, level, &[], Rc::new(f));
    };
    reg("Go", AccessLevel::Counselor, go_on_command);
    reg(
        "DropHolding",
        AccessLevel::Counselor,
        drop_holding_on_command,
    );
    reg(
        "GetFollowers",
        AccessLevel::GameMaster,
        get_followers_on_command,
    );
    reg("Cast", AccessLevel::Counselor, cast_on_command);
    reg("Stuck", AccessLevel::Counselor, stuck_on_command);
    reg("Move", AccessLevel::GameMaster, |w, e| {
        // e.Mobile.Target = new PickMoveTarget()
        muo_content::targets::pick_move_target(w, e.mobile)
    });
    reg("Client", AccessLevel::Counselor, client_on_command);
    reg("Bank", AccessLevel::GameMaster, bank_on_command);
    reg("ViewEquip", AccessLevel::GameMaster, view_equip_on_command);
    reg("Light", AccessLevel::Counselor, light_on_command);
    reg("Stats", AccessLevel::Counselor, stats_on_command);
}

fn hooks(w: &mut World) -> &mut CommandHooks {
    w.resources.get_mut::<CommandHooks>()
}

fn target_mobile(o: Targeted) -> Option<MobileId> {
    match o {
        Targeted::Entity(EntityId::Mobile(m)) => Some(m),
        _ => None,
    }
}

/// `FixMap(ref map, ref loc, Mobile m)`: an internal mobile goes to where it logged out.
fn fix_map_mobile(w: &World, map: &mut Option<MapId>, loc: &mut Point3D, m: MobileId) -> bool {
    let valid = map.is_some_and(|x| x != MapId::INTERNAL);
    if !valid && let Some(mm) = w.mobile(m) {
        *map = mm.logout_map;
        *loc = mm.logout_location;
    }
    valid
}

/// `FixMap(ref map, ref loc, Item item)`.
fn fix_map_item(w: &World, map: &mut Option<MapId>, loc: &mut Point3D, item: ItemId) -> bool {
    if map.is_some_and(|x| x != MapId::INTERNAL) {
        return true;
    }
    match muo_server::item::ops::root_parent(w, item) {
        Some(EntityId::Mobile(m)) => fix_map_mobile(w, map, loc, m),
        _ => false,
    }
}

fn mobile_hidden_from(w: &World, from: MobileId, owner: MobileId) -> bool {
    let (Some(o), Some(f)) = (w.mobile(owner), w.mobile(from)) else {
        return false;
    };
    (o.map.is_none() || o.map == Some(MapId::INTERNAL))
        && o.hidden
        && o.access_level >= f.access_level
}

/// `[Usage("Go [name | serial | (x y [z]) | (deg min (N | S) deg min (E | W))]")]`
fn go_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    match e.length() {
        0 => match hooks(w).go_gump {
            Some(f) => f(w, from),
            None => send_message(w, from, "The go menu is not available."),
        },
        1 => go_one(w, from, e),
        2 | 3 => {
            let Some(map) = w.mobile(from).and_then(|m| m.map) else {
                return;
            };
            // This to avoid being teleported to (0,0) if trying to teleport to a region with spaces in its name.
            let parse = |i: i32| e.get_string(i).trim().parse::<i32>().ok();
            let (Some(x), Some(y)) = (parse(0), parse(1)) else {
                send_message(w, from, "Region name not found.");
                return;
            };
            let z = if e.length() == 3 {
                match parse(2) {
                    Some(z) => z,
                    None => {
                        send_message(w, from, "Region name not found.");
                        return;
                    }
                }
            } else {
                w.maps.get_average_z(map, x, y)
            };
            muo_server::mobile::part1::lifecycle::set_location(w, from, Point3D::new(x, y, z));
        }
        6 => {
            let Some(map) = w.mobile(from).and_then(|m| m.map) else {
                return;
            };
            let east = e.get_string(5).eq_ic("E");
            let south = e.get_string(2).eq_ic("S");
            let p = match hooks(w).sextant_reverse_lookup {
                Some(f) => f(
                    w,
                    map,
                    e.get_int32(3),
                    e.get_int32(0),
                    e.get_int32(4),
                    e.get_int32(1),
                    east,
                    south,
                ),
                None => Point3D::default(),
            };
            if p != Point3D::default() {
                muo_server::mobile::part1::lifecycle::set_location(w, from, p);
            } else {
                send_message(w, from, "Sextant reverse lookup failed.");
            }
        }
        _ => send_message(
            w,
            from,
            "Format: Go [name | serial | (x y [z]) | (deg min (N | S) deg min (E | W)]",
        ),
    }
}

fn go_one(w: &mut World, from: MobileId, e: &CommandEventArgs) {
    let ser = e.get_uint32(0);
    match w.find_entity(Serial::new(ser)) {
        Some(EntityId::Item(item)) => go_item(w, from, item),
        Some(EntityId::Mobile(m)) => {
            let (mut map, mut loc) = w.mobile(m).map(|x| (x.map, x.location)).unwrap_or_default();
            let accessible =
                crate::generic::is_accessible(w, from, Some(Targeted::Entity(EntityId::Mobile(m))));
            if map.is_some_and(|x| x != MapId::INTERNAL) && !accessible {
                send_message(w, from, "You can not go to what you can not see.");
                return;
            }
            if mobile_hidden_from(w, from, m) {
                send_message(w, from, "You can not go to what you can not see.");
                return;
            }
            if !fix_map_mobile(w, &mut map, &mut loc, m) {
                send_message(
                    w,
                    from,
                    "That is an internal mobile and you cannot go to it.",
                );
                return;
            }
            if let Some(map) = map {
                muo_server::mobile::base::move_to_world(w, from, loc, map);
            }
        }
        None => go_named(w, from, &e.get_string(0), ser),
    }
}

fn go_item(w: &mut World, from: MobileId, item: ItemId) {
    let mut map = w.item(item).and_then(|i| i.map);
    let mut loc = muo_server::item::ops::get_world_location(w, item);
    if let Some(EntityId::Mobile(o)) = muo_server::item::ops::root_parent(w, item) {
        let omap = w.mobile(o).and_then(|m| m.map);
        let accessible =
            crate::generic::is_accessible(w, from, Some(Targeted::Entity(EntityId::Mobile(o))));
        if omap.is_some_and(|x| x != MapId::INTERNAL) && !accessible {
            send_message(w, from, "You can not go to what you can not see.");
            return;
        }
        if mobile_hidden_from(w, from, o) {
            send_message(w, from, "You can not go to what you can not see.");
            return;
        }
    }
    if !fix_map_item(w, &mut map, &mut loc, item) {
        send_message(w, from, "That is an internal item and you cannot go to it.");
        return;
    }
    if let Some(map) = map {
        muo_server::mobile::base::move_to_world(w, from, loc, map);
    }
}

/// The `else` branch of `Go name`: a map name, a region of this map, a region of another map.
fn go_named(w: &mut World, from: MobileId, name: &str, ser: u32) {
    let hidden_index = |i: i32| i == 0x7F || i == 0xFF;
    let maps: Vec<MapId> = w.maps.all_maps.clone();
    for id in &maps {
        let Some(m) = w.maps.get(*id) else { continue };
        if hidden_index(m.map_index) {
            continue;
        }
        if name.eq_ic(m.name()) {
            muo_server::mobile::part1::lifecycle::set_map(w, from, Some(*id));
            return;
        }
    }
    let cur = w.mobile(from).and_then(|m| m.map);
    let region_go = |w: &World, map: MapId| -> Option<Point3D> {
        let m = w.maps.get(map)?;
        m.regions
            .values()
            .filter_map(|r| w.region(*r))
            .find(|r| r.name.as_deref().is_some_and(|n| n.eq_ic(name)))
            .map(|r| r.go_location)
    };
    // `from.Map.Regions` throws on no map: caught, "Region name not found"
    let Some(cur_map) = cur else {
        send_message(w, from, "Region name not found");
        return;
    };
    if let Some(p) = region_go(w, cur_map) {
        muo_server::mobile::part1::lifecycle::set_location(w, from, p);
        return;
    }
    for id in &maps {
        let Some(m) = w.maps.get(*id) else { continue };
        if hidden_index(m.map_index) || cur == Some(*id) {
            continue;
        }
        if let Some(p) = region_go(w, *id) {
            muo_server::mobile::base::move_to_world(w, from, p, *id);
            return;
        }
    }
    if ser != 0 {
        send_message(w, from, "No object with that serial was found.");
    } else {
        send_message(w, from, "No region with that name was found.");
    }
}

/// `[Usage("DropHolding")]`
fn drop_holding_on_command(w: &mut World, e: &mut CommandEventArgs) {
    begin_target(
        w,
        e.mobile,
        -1,
        false,
        TargetFlags::NONE,
        drop_holding_on_target,
    );
    send_message(
        w,
        e.mobile,
        "Target the player to drop what they are holding.",
    );
}

fn drop_holding_on_target(w: &mut World, from: MobileId, o: Targeted) {
    let targ = target_mobile(o).filter(|t| w.mobile(*t).is_some_and(|m| m.player));
    let Some(targ) = targ else {
        begin_target(
            w,
            from,
            -1,
            false,
            TargetFlags::NONE,
            drop_holding_on_target,
        );
        send_message(w, from, "That is not a player. Try again.");
        return;
    };
    let Some(held) = w.mobile(targ).and_then(|m| m.holding) else {
        send_message(w, from, "They are not holding anything.");
        return;
    };
    if w.mobile(from)
        .is_some_and(|m| m.access_level == AccessLevel::Counselor)
    {
        let handler = hooks(w).page_handler.and_then(|f| f(w, targ));
        if handler == Some(from) {
            send_message(
                w,
                from,
                "You may only use this command if you are handling their help page.",
            );
        } else {
            send_message(
                w,
                from,
                "You may only use this command on someone who has paged you.",
            );
        }
        return;
    }
    if muo_server::mobile::part2::items::add_to_backpack(w, targ, held) {
        send_message(
            w,
            from,
            "The item they were holding has been placed into their backpack.",
        );
    } else {
        send_message(
            w,
            from,
            "The item they were holding has been placed at their feet.",
        );
    }
    muo_server::item::ops::clear_bounce(w, held);
    muo_server::mobile::part1::props_a::set_holding(w, targ, None);
}

/// `[Usage("GetFollowers")]`
fn get_followers_on_command(w: &mut World, e: &mut CommandEventArgs) {
    begin_target(
        w,
        e.mobile,
        -1,
        false,
        TargetFlags::NONE,
        get_followers_on_target,
    );
    send_message(w, e.mobile, "Target a player to get their pets.");
}

fn get_followers_on_target(w: &mut World, from: MobileId, o: Targeted) {
    let Some(targ) = target_mobile(o) else {
        begin_target(
            w,
            from,
            -1,
            false,
            TargetFlags::NONE,
            get_followers_on_target,
        );
        send_message(w, from, "That is not a player. Try again.");
        return;
    };
    let is_pm = muo_content::player_mobile::data(w, targ).is_some();
    let pets: Vec<MobileId> = if is_pm {
        // `pm.AllFollowers`; `!(pets?.Count > 0)` is the "no pets" message
        muo_content::player_mobile::data(w, targ)
            .and_then(|d| d.all_followers.as_ref())
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    } else if w.mobile(targ).is_some_and(|m| m.player) {
        w.mobiles
            .keys()
            .copied()
            .filter(|m| {
                muo_content::creature::data::is_creature(w, *m)
                    && muo_content::creature::control::get_master(w, *m) == Some(targ)
            })
            .collect()
    } else {
        begin_target(
            w,
            from,
            -1,
            false,
            TargetFlags::NONE,
            get_followers_on_target,
        );
        send_message(w, from, "That is not a player. Try again.");
        return;
    };
    if pets.is_empty() {
        send_message(w, from, "There were no pets found for that player.");
        return;
    }
    logging::log_action(w, from, "getting all followers of", EntityId::Mobile(targ));
    let n = pets.len();
    send_message(
        w,
        from,
        &if n == 1 {
            format!("That player has {n} pet.")
        } else {
            format!("That player has {n} pets.")
        },
    );
    let (loc, map) = w
        .mobile(from)
        .map(|m| (m.location, m.map))
        .unwrap_or_default();
    for pet in pets {
        if muo_content::creature::mount::is_base_mount(w, pet) {
            muo_content::creature::mount::set_rider(w, pet, None); // make sure it's dismounted
        }
        if let Some(map) = map {
            muo_server::mobile::base::move_to_world(w, pet, loc, map);
        }
    }
}

/// `[Usage("Cast <name>")]`
fn cast_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() != 1 {
        send_message(w, from, "Format: Cast <name>");
        return;
    }
    if let Some(f) = hooks(w).design_context_check
        && !f(w, from)
    {
        return; // They are customizing
    }
    match muo_content::spells::registry::new_spell_by_name(w, &e.get_string(0), from, None) {
        Some(spell) => {
            muo_content::spells::spell::cast(w, spell);
        }
        None => send_message(w, from, "That spell was not found."),
    }
}

/// `[Usage("Stuck")]`
fn stuck_on_command(w: &mut World, e: &mut CommandEventArgs) {
    begin_target(w, e.mobile, -1, false, TargetFlags::NONE, |w, from, o| {
        let Some(mobile) = target_mobile(o) else {
            return;
        };
        let higher =
            w.mobile(mobile).map(|m| m.access_level) >= w.mobile(from).map(|m| m.access_level);
        if higher && mobile != from {
            send_message(
                w,
                from,
                "You can't do that to someone with higher Accesslevel than you!",
            );
        } else if let Some(f) = hooks(w).stuck_menu {
            f(w, from, mobile);
        }
    });
}

/// `[Usage("Client")]`
fn client_on_command(w: &mut World, e: &mut CommandEventArgs) {
    begin_target(w, e.mobile, -1, false, TargetFlags::NONE, |w, from, o| {
        let Some(targ) = target_mobile(o) else {
            return;
        };
        if w.mobile(targ).and_then(|m| m.net_state).is_none() {
            return;
        }
        logging::log_action(w, from, "opening client menu of", EntityId::Mobile(targ));
        if let Some(f) = hooks(w).client_gump {
            f(w, from, targ);
        }
    });
}

/// `[Usage("Bank")]`
fn bank_on_command(w: &mut World, e: &mut CommandEventArgs) {
    begin_target(w, e.mobile, -1, false, TargetFlags::NONE, |w, from, o| {
        let Some(m) = target_mobile(o) else { return };
        let player = w.mobile(m).is_some_and(|x| x.player);
        let bank = if player {
            muo_server::mobile::part1::props_b::bank_box(w, m)
        } else {
            muo_server::mobile::part2::items::find_bank_no_create(w, m)
        };
        let Some(bank) = bank else {
            send_message(w, from, "They have no bank box.");
            return;
        };
        logging::log_action(w, from, "opening bank box of", EntityId::Mobile(m));
        if from == m {
            muo_server::item::containers::open(w, bank);
        } else if let Some(ty) = w.item(bank).map(|i| i.ty) {
            (ty.display_to)(w, bank, from);
        }
    });
}

/// `[Usage("Light <level>")]`
fn light_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let level = e.get_int32(0);
    muo_server::mobile::part1::props_a::set_light_level(w, e.mobile, level);
}

/// `[Usage("Stats")]`
fn stats_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let (c, m, i) = (w.net_states.len(), w.mobiles.len(), w.items.len());
    send_message(w, from, &format!("Open Connections: {c}"));
    send_message(w, from, &format!("Mobiles: {m}"));
    send_message(w, from, &format!("Items: {i}"));
}

// ---- ViewEquip -------------------------------------------------------------------------------------------------

fn item_title(w: &World, item: ItemId) -> Option<(String, i32, i32)> {
    let i = w.item(item)?;
    let ty = i.ty.full_name.rsplit(['.', '+']).next().unwrap_or("");
    Some((format!("{:?}: {ty}", i.layer), i.item_id, i.hue))
}

fn equip_entries(w: &World, m: MobileId) -> Vec<ItemListEntry> {
    let items = w.mobile(m).map(|x| x.items.clone()).unwrap_or_default();
    items
        .into_iter()
        .filter_map(|i| item_title(w, i))
        .map(|(name, id, hue)| ItemListEntry::new(&name, id, hue, 0))
        .collect()
}

/// `[Usage("ViewEquip")]`
fn view_equip_on_command(w: &mut World, e: &mut CommandEventArgs) {
    begin_target(w, e.mobile, -1, false, TargetFlags::NONE, |w, from, o| {
        if !crate::generic::is_accessible(w, from, Some(o)) {
            send_localized_message(w, from, 500447, "", 0x3B2); // That is not accessible.
            return;
        }
        if let Some(mobile) = target_mobile(o) {
            send_equip_menu(w, from, mobile);
        }
    });
}

/// `from.SendMenu(new EquipMenu(from, m, GetEquip(m)))`.
fn send_equip_menu(w: &mut World, from: MobileId, m: MobileId) {
    let Some(ns) = w.mobile(from).and_then(|x| x.net_state) else {
        return;
    };
    let entries = equip_entries(w, m);
    let mut menu = Menu::item_list(w, "Equipment", entries);
    logging::log_text(
        w,
        from,
        &format!(
            "viewing equipment of {}",
            logging::format_entity(w, EntityId::Mobile(m))
        ),
    );
    menu.on_response = Some(Rc::new(move |w, state, index| {
        let item = w.mobile(m).and_then(|x| {
            usize::try_from(index)
                .ok()
                .and_then(|i| x.items.get(i).copied())
        });
        if let (Some(item), Some(viewer)) = (item, w.net_mobile(state)) {
            send_equip_details(w, viewer, m, item);
        }
    }));
    menus::send_to(w, ns, menu);
}

fn send_equip_details(w: &mut World, viewer: MobileId, m: MobileId, item: ItemId) {
    let Some(ns) = w.mobile(viewer).and_then(|x| x.net_state) else {
        return;
    };
    let Some((title, _, _)) = item_title(w, item) else {
        return;
    };
    let mut menu = Menu::question(
        w,
        &title,
        vec!["Move".into(), "Delete".into(), "Props".into()],
    );
    menu.on_cancel = Some(Rc::new(move |w, state| {
        if let Some(v) = w.net_mobile(state) {
            send_equip_menu(w, v, m);
        }
    }));
    menu.on_response = Some(Rc::new(move |w, state, index| {
        let Some(v) = w.net_mobile(state) else { return };
        let what = EntityId::Item(item);
        let (fi, fm) = (
            logging::format_entity(w, what),
            logging::format_entity(w, EntityId::Mobile(m)),
        );
        match index {
            0 => {
                logging::log_text(w, v, &format!("moving equipment item {fi} of {fm}"));
                muo_content::targets::move_target(w, v, what);
            }
            1 => {
                logging::log_text(w, v, &format!("deleting equipment item {fi} of {fm}"));
                muo_server::item::base::delete(w, item);
            }
            2 => {
                logging::log_text(
                    w,
                    v,
                    &format!("opening properties for equipment item {fi} of {fm}"),
                );
                crate::properties::open_properties_gump(w, v, what);
            }
            _ => {}
        }
    }));
    menus::send_to(w, ns, menu);
}
