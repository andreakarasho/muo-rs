//! `TargetCommands` and the commands it registers.
//!
//! `AddCommand` / `AddToPackCommand` and `FactionKickCommand` are in `crate::add` / a hook;
//! `InterfaceCommand` is `interface.rs`.

use std::rc::Rc;

use muo_primitives::Point3D;
use muo_server::commands::CommandEventArgs;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::deps::{send_localized_message, send_message as item_send_message};
use muo_server::mobile::AccessLevel;
use muo_server::target::Targeted;
use muo_server::world::World;

use super::conditional::ObjectConditional;
use super::implementors::register_command;
use super::{
    BaseCommand, CmdSpec, CommandSupport as S, Obj, ObjectTypes as O, Run, as_item, as_mobile,
};
use crate::hooks::CommandHooks;
use crate::logging::{log_action, write_line};
use crate::properties;

use AccessLevel::{Administrator as ADM, Counselor as CNS, GameMaster as GM};

fn reg(w: &mut World, c: impl BaseCommand + 'static) {
    register_command(w, Rc::new(c));
}

/// `TargetCommands.Configure`.
pub fn register_all(w: &mut World) {
    reg(w, KillCommand::new(true));
    reg(w, KillCommand::new(false));
    reg(w, HideCommand::new(true));
    reg(w, HideCommand::new(false));
    reg(w, KickCommand::new(true));
    reg(w, KickCommand::new(false));
    reg(w, FirewallCommand::new());
    reg(w, TeleCommand::new());
    reg(w, SetCommand::new());
    for (name, prop, value) in [
        ("Immortal", "blessed", "true"),
        ("Invul", "blessed", "true"),
        ("Mortal", "blessed", "false"),
        ("NoInvul", "blessed", "false"),
        ("Squelch", "squelched", "true"),
        ("Unsquelch", "squelched", "false"),
        ("ShaveHair", "HairItemID", "0"),
        ("ShaveBeard", "FacialHairItemID", "0"),
    ] {
        reg(w, AliasedSetCommand::new(GM, name, prop, value, O::Mobiles));
    }
    reg(w, GetCommand::new());
    reg(w, GetTypeCommand::new());
    reg(w, DeleteCommand::new());
    reg(w, RestockCommand::new());
    reg(w, DismountCommand::new());
    reg(w, crate::add::AddCommand::new());
    reg(w, crate::add::AddToPackCommand::new());
    reg(w, TellCommand::new(true));
    reg(w, TellCommand::new(false));
    reg(w, PrivSoundCommand::new());
    reg(w, IncreaseCommand::new());
    reg(w, OpenBrowserCommand::new());
    reg(w, CountCommand::new());
    reg(w, super::interface::InterfaceCommand::new());
    reg(w, RefreshHouseCommand::new());
    reg(w, ConditionCommand::new());
    reg(
        w,
        FactionKickCommand::new(
            "FactionKick",
            0,
            "Kicks the targeted player out of his current faction. This does not prevent them from rejoining.",
        ),
    );
    reg(
        w,
        FactionKickCommand::new(
            "FactionBan",
            1,
            "Bans the account of a targeted player from joining factions. All players on the account are removed from their current faction, if any.",
        ),
    );
    reg(
        w,
        FactionKickCommand::new(
            "FactionUnban",
            2,
            "Unbans the account of a targeted player from joining factions.",
        ),
    );
    reg(w, BringToPackCommand::new());
    reg(w, TraceLockdownCommand::new());
    reg(w, crate::location_command::LocationCommand::new());
    reg(w, crate::teleporters::DesignInsertCommand::new());
}

fn level_of(w: &World, m: MobileId) -> AccessLevel {
    w.mobile(m).map_or(AccessLevel::Player, |x| x.access_level)
}

fn entity(o: Obj) -> Option<EntityId> {
    match o {
        Targeted::Entity(e) => Some(e),
        _ => None,
    }
}

macro_rules! spec_fn {
    () => {
        fn spec(&self) -> &CmdSpec {
            &self.spec
        }
    };
}

// ---- Condition ------------------------------------------------------------------------------------------------

pub struct ConditionCommand {
    spec: CmdSpec,
}
impl ConditionCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["Condition"],
                GM,
                S::SIMPLE | S::COMPLEX | S::SELF,
                O::All,
                "Condition <condition>",
                "Checks that the given condition matches a targeted object.",
            )
            .list_optimized(),
        }
    }
}
impl BaseCommand for ConditionCommand {
    spec_fn!();
    fn execute_list(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        let mut args = e.arguments.clone();
        match ObjectConditional::parse(w, level_of(w, e.mobile), &mut args) {
            Err(m) => muo_server::mobile::part2::messages::send_message(w, e.mobile, &m),
            Ok(cond) => {
                for o in list {
                    if cond.check(w, *o) {
                        run.add_response("True - that object matches the condition.");
                    } else {
                        run.add_response("False - that object does not match the condition.");
                    }
                }
            }
        }
    }
}

// ---- BringToPack ----------------------------------------------------------------------------------------------

pub struct BringToPackCommand {
    spec: CmdSpec,
}
impl BringToPackCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["BringToPack"],
                GM,
                S::ALL_ITEMS,
                O::Items,
                "BringToPack",
                "Brings a targeted item to your backpack.",
            ),
        }
    }
}
impl BaseCommand for BringToPackCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        if let Some(item) = as_item(obj) {
            if muo_server::mobile::part2::items::place_in_backpack(w, e.mobile, item) {
                run.add_response("The item has been placed in your backpack.");
            } else {
                run.add_response("Your backpack could not hold the item.");
            }
        }
    }
}

// ---- RefreshHouse ---------------------------------------------------------------------------------------------

pub struct RefreshHouseCommand {
    spec: CmdSpec,
}
impl RefreshHouseCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["RefreshHouse"],
                GM,
                S::SIMPLE,
                O::Items,
                "RefreshHouse",
                "Refreshes a targeted house sign.",
            ),
        }
    }
}
impl BaseCommand for RefreshHouseCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, _e: &CommandEventArgs, obj: Obj) {
        // `CommandHooks::refresh_house` is set by `muo_engines::wiring2`.
        let hook = w.resources.get_mut::<CommandHooks>().refresh_house;
        match (hook, as_item(obj)) {
            (Some(h), Some(i)) => match h(w, i) {
                Some(true) => run.add_response("The house has been refreshed."),
                Some(false) => run.log_failure("That sign has no house attached."),
                None => run.log_failure("That is not a house sign."),
            },
            _ => run.log_failure("That is not a house sign."),
        }
    }
}

// ---- Count ----------------------------------------------------------------------------------------------------

pub struct CountCommand {
    spec: CmdSpec,
}
impl CountCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(&["Count"], GM, S::COMPLEX, O::All, "Count",
                "Counts the number of objects that a command modifier would use. Generally used with condition arguments.").list_optimized(),
        }
    }
}
impl BaseCommand for CountCommand {
    spec_fn!();
    fn execute_list(&self, _w: &mut World, run: &mut Run, _e: &CommandEventArgs, list: &[Obj]) {
        if list.len() == 1 {
            run.add_response("There is one matching object.");
        } else {
            run.add_response(&format!("There are {} matching objects.", list.len()));
        }
    }
}

// ---- OpenBrowser ----------------------------------------------------------------------------------------------

pub struct OpenBrowserCommand {
    spec: CmdSpec,
}
impl OpenBrowserCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["OpenBrowser", "OB"],
                GM,
                S::ALL_MOBILES,
                O::Mobiles,
                "OpenBrowser <url>",
                "Opens the web browser of a targeted player to a specified url.",
            ),
        }
    }

    fn execute_echo(
        &self,
        w: &mut World,
        run: &mut Run,
        e: &CommandEventArgs,
        obj: Obj,
        echo: bool,
    ) {
        if e.length() != 1 {
            return run.log_failure("Format: OpenBrowser <url>");
        }
        let Some(mob) = as_mobile(obj) else { return };
        let from = e.mobile;
        if !w.mobile(mob).is_some_and(|m| m.player) {
            return run.log_failure("That is not a player.");
        }
        if w.mobile(mob).and_then(|m| m.net_state).is_none() {
            return run.log_failure("That player is not online.");
        }
        let url = e.get_string(0);
        let text = format!(
            "{:?} {} requesting to open web browser of {} to {url}",
            level_of(w, from),
            crate::logging::format_entity(w, EntityId::Mobile(from)),
            crate::logging::format_entity(w, EntityId::Mobile(mob))
        );
        write_line(w, from, &text);
        run.add_response(if echo {
            "Awaiting user confirmation..."
        } else {
            "Open web browser request sent."
        });
        let Some(gump) = w.resources.get_mut::<CommandHooks>().warning_gump else {
            // no confirmation gump installed (`muo_gumps` sets it): nobody can confirm, so the browser is not opened
            return;
        };
        let content = format!(
            "A game master is requesting to open your web browser to the following URL:<br>{url}"
        );
        let url2 = url.clone();
        gump(
            w,
            mob,
            &content,
            (320, 240),
            Box::new(move |w, okay| open_browser_callback(w, mob, okay, from, &url2, echo)),
        );
    }
}

/// `OpenBrowser_Callback`.
fn open_browser_callback(
    w: &mut World,
    from: MobileId,
    okay: bool,
    gm: MobileId,
    url: &str,
    echo: bool,
) {
    use muo_server::mobile::part2::messages::send_message;
    let name = w
        .mobile(from)
        .and_then(|m| (m.ty.name)(w, from))
        .unwrap_or_default();
    if okay {
        if echo {
            send_message(
                w,
                gm,
                &format!("{name} : has opened their web browser to : {url}"),
            );
        }
        muo_server::mobile::part2::stats::launch_browser(w, from, url);
    } else {
        if echo {
            send_message(
                w,
                gm,
                &format!("{name} : has chosen not to open their web browser to : {url}"),
            );
        }
        send_message(w, from, "You have chosen not to open your web browser.");
    }
}

impl BaseCommand for OpenBrowserCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        self.execute_echo(w, run, e, obj, true);
    }
    fn execute_list(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        for o in list {
            self.execute_echo(w, run, e, *o, false);
        }
    }
}

// ---- Increase -------------------------------------------------------------------------------------------------

pub struct IncreaseCommand {
    spec: CmdSpec,
}
impl IncreaseCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["Increase", "Inc"],
                CNS,
                S::ALL,
                O::Both,
                "Increase {<propertyName> <offset> ...}",
                "Increases the value of a specified property by the specified offset.",
            ),
        }
    }
}
impl BaseCommand for IncreaseCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(ent) = entity(obj) else { return };
        if matches!(ent, EntityId::Item(i) if muo_content::factory::item_is(w, i, "Server.Items.BaseMulti"))
        {
            run.log_failure("This command does not work on multis.");
        } else if e.length() >= 2 {
            let result = properties::increase_value(w, e.mobile, ent, &e.arguments);
            match result.as_str() {
                "The property has been increased."
                | "The properties have been increased."
                | "The property has been decreased."
                | "The properties have been decreased."
                | "The properties have been changed." => run.add_response(&result),
                _ => run.log_failure(&result),
            }
        } else {
            run.log_failure("Format: Increase {<propertyName> <offset> ...}");
        }
    }
}

// ---- PrivSound ------------------------------------------------------------------------------------------------

pub struct PrivSoundCommand {
    spec: CmdSpec,
}
impl PrivSoundCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["PrivSound"],
                GM,
                S::ALL_MOBILES,
                O::Mobiles,
                "PrivSound <index>",
                "Plays a sound to a given target.",
            ),
        }
    }
}
impl BaseCommand for PrivSoundCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, _run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let from = e.mobile;
        if e.length() == 1 {
            let index = e.get_int32(0);
            let Some(mob) = as_mobile(obj) else { return };
            let text = format!(
                "{:?} {} playing sound {index} for {}",
                level_of(w, from),
                crate::logging::format_entity(w, EntityId::Mobile(from)),
                crate::logging::format_entity(w, EntityId::Mobile(mob))
            );
            write_line(w, from, &text);
            muo_server::mobile::part2::effects::send_sound(w, mob, index);
        } else {
            muo_server::mobile::part2::messages::send_message(w, from, "Format: PrivSound <index>");
        }
    }
}

// ---- Tell / Message -------------------------------------------------------------------------------------------

pub struct TellCommand {
    spec: CmdSpec,
    in_gump: bool,
}
impl TellCommand {
    fn new(in_gump: bool) -> Self {
        let spec = if in_gump {
            CmdSpec::new(
                &["Message", "Msg"],
                CNS,
                S::ALL_MOBILES,
                O::Mobiles,
                "Message <text>",
                "Sends a message to a targeted player.",
            )
        } else {
            CmdSpec::new(
                &["Tell"],
                CNS,
                S::ALL_MOBILES,
                O::Mobiles,
                "Tell <text>",
                "Sends a system message to a targeted player.",
            )
        };
        Self { spec, in_gump }
    }
}
impl BaseCommand for TellCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, _run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(mob) = as_mobile(obj) else { return };
        let from = e.mobile;
        let text = format!(
            "{:?} {} {} {} \"{}\"",
            level_of(w, from),
            crate::logging::format_entity(w, EntityId::Mobile(from)),
            if self.in_gump { "messaging" } else { "telling" },
            crate::logging::format_entity(w, EntityId::Mobile(mob)),
            e.arg_string
        );
        write_line(w, from, &text);
        if self.in_gump {
            let from_name = w
                .mobile(from)
                .and_then(|m| (m.ty.name)(w, from))
                .unwrap_or_default();
            // `MessageSentGump` is `muo_engines::help`'s (hook set in `help::configure`); unset, a system message.
            match w.resources.get_mut::<CommandHooks>().message_sent_gump {
                Some(g) => g(w, mob, &from_name, &e.arg_string),
                None => muo_server::mobile::part2::messages::send_message(w, mob, &e.arg_string),
            }
        } else {
            muo_server::mobile::part2::messages::send_message(w, mob, &e.arg_string);
        }
    }
}

// ---- Teleport -------------------------------------------------------------------------------------------------

pub struct TeleCommand {
    spec: CmdSpec,
}
impl TeleCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["Teleport", "Tele"],
                CNS,
                S::SIMPLE,
                O::All,
                "Teleport",
                "Teleports your character to a targeted location.",
            ),
        }
    }
}

/// `SpellHelper.GetSurfaceTop(ref p)` then `new Point3D(p)` of what was targeted: an item that is not a surface is its
/// own location (not its top), a static target loses the tile's height unless the tile is a surface.
pub fn targeted_point(w: &World, o: Obj) -> Point3D {
    use muo_content::spells::helper::{self, Point3dRef};
    match o {
        Targeted::Entity(e) => {
            helper::point_of(w, helper::get_surface_top(w, Point3dRef::Entity(e)))
        }
        Targeted::Land(l) => l.location,
        Targeted::Static(s) => helper::static_target_surface_top(w, s),
    }
}

impl BaseCommand for TeleCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, _run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let from = e.mobile;
        // `SpellHelper.GetSurfaceTop(ref p)`: an item is a surface at its top, a mobile at its location.
        let to = targeted_point(w, obj);
        let Some((from_loc, map, hidden)) = w.mobile(from).map(|m| (m.location, m.map, m.hidden))
        else {
            return;
        };
        muo_server::mobile::part1::lifecycle::set_location(w, from, to);
        muo_server::mobile::part1::lifecycle::process_delta(w, from);
        if !hidden {
            // `EffectItem.Create(loc, map, EffectItem.DefaultDuration)` is the particle origin.
            let dur = muo_server::item::effect_item::DEFAULT_DURATION;
            let a = muo_server::item::effect_item::create(w, from_loc, map, dur);
            muo_server::effects::send_location_particles(
                w,
                EntityId::Item(a),
                0x3728,
                10,
                10,
                2023,
            );
            let b = muo_server::item::effect_item::create(w, to, map, dur);
            muo_server::effects::send_location_particles(
                w,
                EntityId::Item(b),
                0x3728,
                10,
                10,
                5023,
            );
            muo_server::mobile::part2::effects::play_sound(w, from, 0x1FE);
        }
    }
}

// ---- Dismount -------------------------------------------------------------------------------------------------

pub struct DismountCommand {
    spec: CmdSpec,
}
impl DismountCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["Dismount"],
                GM,
                S::ALL_MOBILES,
                O::Mobiles,
                "Dismount",
                "Forcefully dismounts a given target.",
            ),
        }
    }
}
impl BaseCommand for DismountCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let from = e.mobile;
        let Some(mob) = as_mobile(obj) else { return };
        log_action(w, from, "dismounting", EntityId::Mobile(mob));
        let mut taken = false;
        // `item is IMountItem`: each worn item's mount loses its rider. Walking a copy of the list
        // covers clearing the rider removing the item.
        let worn: Vec<ItemId> = w.mobile(mob).map(|m| m.items.to_vec()).unwrap_or_default();
        let mount_of = w
            .resources
            .get::<muo_server::mobile::part1::props_b::PropsBStatics>()
            .and_then(|s| s.mount_of_item);
        for item in worn {
            if let Some(mount) = mount_of.and_then(|f| f(w, item)) {
                muo_server::mobile::part2::deps::mount_clear_rider(w, mount);
                taken = true;
            }
        }
        let mounts: Vec<ItemId> = w
            .mobile(mob)
            .map(|m| {
                m.items
                    .iter()
                    .copied()
                    .filter(|i| {
                        w.item(*i)
                            .is_some_and(|x| x.layer == muo_server::layer::Layer::Mount)
                    })
                    .collect()
            })
            .unwrap_or_default();
        for i in mounts {
            taken = true;
            muo_server::item::base::delete(w, i);
        }
        if taken {
            run.add_response("They have been dismounted.");
        } else {
            run.log_failure("They were not mounted.");
        }
    }
}

// ---- Restock --------------------------------------------------------------------------------------------------

pub struct RestockCommand {
    spec: CmdSpec,
}
impl RestockCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["Restock"],
                GM,
                S::ALL_NPCS,
                O::Mobiles,
                "Restock",
                "Manually restocks a targeted vendor, refreshing the quantity of every item the vendor sells to the maximum. This also invokes the maximum quantity adjustment algorithms.",
            ),
        }
    }
}
impl BaseCommand for RestockCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        match as_mobile(obj) {
            Some(m) if muo_content::factory::mobile_is(w, m, "Server.Mobiles.BaseVendor") => {
                log_action(w, e.mobile, "restocking", EntityId::Mobile(m));
                muo_content::vendor::restock(w, m);
                run.add_response("The vendor has been restocked.");
            }
            _ => run.add_response("That is not a vendor."),
        }
    }
}

// ---- GetType / Get --------------------------------------------------------------------------------------------

pub struct GetTypeCommand {
    spec: CmdSpec,
}
impl GetTypeCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["GetType"],
                CNS,
                S::ALL,
                O::All,
                "GetType",
                "Gets the type name of a targeted object.",
            ),
        }
    }
}
impl BaseCommand for GetTypeCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, _e: &CommandEventArgs, obj: Obj) {
        let full = match obj {
            Targeted::Entity(EntityId::Item(i)) => w.item(i).map(|x| x.ty.full_name),
            Targeted::Entity(EntityId::Mobile(m)) => w.mobile(m).map(|x| x.ty.full_name),
            Targeted::Land(_) => Some("Server.Targeting.LandTarget"),
            Targeted::Static(_) => Some("Server.Targeting.StaticTarget"),
        };
        match full {
            None => run.add_response("The object is null."),
            Some(f) => {
                // `type.DeclaringType == null ? type.Name : type.FullName`
                let text = if f.contains('+') {
                    f.to_string()
                } else {
                    f.rsplit('.').next().unwrap_or(f).to_string()
                };
                run.add_response(&format!("The type of that object is {text}."));
            }
        }
    }
}

/// `Properties.GetValue` over a `LandTarget` / `StaticTarget`: their `[CommandProperty]` members (all Counselor).
fn target_get_value(w: &World, obj: Obj, name: &str) -> String {
    let (loc, tile, hue, is_land) = match obj {
        Targeted::Land(l) => (l.location, l.tile_id as usize, 0, true),
        Targeted::Static(s) => (s.location, s.item_id as usize, s.hue, false),
        Targeted::Entity(_) => return "Property not found.".into(),
    };
    let num = |v: i32| format!("{v} (0x{:X})", v as u32);
    let flags = |f: muo_data::tile_data::TileFlag| {
        let names: Vec<String> = f
            .iter_names()
            .map(|(n, _)| {
                if n.starts_with("HS") {
                    return n.to_owned();
                }
                n.split('_')
                    .map(|p| format!("{}{}", &p[..1], p[1..].to_ascii_lowercase()))
                    .collect()
            })
            .collect();
        if names.is_empty() {
            "0".into()
        } else {
            names.join(", ")
        }
    };
    let tile_name = |n: &str| {
        if n == "null" {
            "@\"null\"".to_string()
        } else {
            format!("\"{n}\"")
        }
    };
    let (shown, value) = match name.to_ascii_lowercase().as_str() {
        "x" => ("X", num(loc.x)),
        "y" => ("Y", num(loc.y)),
        "z" => ("Z", num(loc.z)),
        "location" => ("Location", format!("({}, {}, {})", loc.x, loc.y, loc.z)),
        "tileid" if is_land => ("TileID", num(tile as i32)),
        "itemid" if !is_land => ("ItemID", num(tile as i32)),
        "hue" if !is_land => ("Hue", num(hue)),
        "name" if is_land => ("Name", tile_name(&w.maps.tile_data.land(tile).name)),
        "name" => ("Name", tile_name(&w.maps.tile_data.item(tile).name)),
        "flags" if is_land => ("Flags", flags(w.maps.tile_data.land(tile).flags)),
        "flags" => ("Flags", flags(w.maps.tile_data.item(tile).flags)),
        _ => return "Property not found.".into(),
    };
    format!("{shown} = {value}")
}

pub struct GetCommand {
    spec: CmdSpec,
}
impl GetCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["Get"],
                CNS,
                S::ALL,
                O::All,
                "Get <propertyName>",
                "Gets one or more property values by name of a targeted object.",
            ),
        }
    }
}
impl BaseCommand for GetCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        if e.length() >= 1 {
            for i in 0..e.length() {
                let name = e.get_string(i as i32);
                let result = match entity(obj) {
                    Some(ent) => properties::get_value(w, e.mobile, ent, &name),
                    None => target_get_value(w, obj, &name),
                };
                if result == "Property not found."
                    || result == "Property is write only."
                    || result.starts_with("Getting this property")
                {
                    run.log_failure(&result);
                } else {
                    run.add_response(&result);
                }
            }
        } else {
            run.log_failure("Format: Get <propertyName>");
        }
    }
}

// ---- Set ------------------------------------------------------------------------------------------------------

pub struct AliasedSetCommand {
    spec: CmdSpec,
    name: String,
    value: String,
}
impl AliasedSetCommand {
    fn new(level: AccessLevel, command: &str, name: &str, value: &str, objects: O) -> Self {
        let supports = match objects {
            O::Items => S::ALL_ITEMS,
            O::Mobiles => S::ALL_MOBILES,
            _ => S::ALL,
        };
        Self {
            spec: CmdSpec::new(
                &[command],
                level,
                supports,
                objects,
                command,
                &format!("Sets the {name} property to {value}."),
            ),
            name: name.into(),
            value: value.into(),
        }
    }
}
impl BaseCommand for AliasedSetCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(ent) = entity(obj) else { return };
        let result = properties::set_value(w, e.mobile, ent, &self.name, &self.value);
        if result == "Property has been set." {
            run.add_response(&result);
        } else {
            run.log_failure(&result);
        }
    }
}

pub struct SetCommand {
    spec: CmdSpec,
}
impl SetCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["Set"],
                CNS,
                S::ALL,
                O::Both,
                "Set <propertyName> <value> [...]",
                "Sets one or more property values by name of a targeted object.",
            ),
        }
    }
}
impl BaseCommand for SetCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(ent) = entity(obj) else { return };
        if e.length() >= 2 {
            let mut i = 0;
            while i + 1 < e.length() {
                let result = properties::set_value(
                    w,
                    e.mobile,
                    ent,
                    &e.get_string(i as i32),
                    &e.get_string(i as i32 + 1),
                );
                if result == "Property has been set." {
                    run.add_response(&result);
                } else {
                    run.log_failure(&result);
                }
                i += 2;
            }
        } else {
            run.log_failure("Format: Set <propertyName> <value>");
        }
    }
}

// ---- Delete ---------------------------------------------------------------------------------------------------

pub struct DeleteCommand {
    spec: CmdSpec,
}
impl DeleteCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["Delete", "Remove", "Rm"],
                GM,
                S::ALL_NPCS | S::ALL_ITEMS,
                O::Both,
                "Delete",
                "Deletes a targeted item or mobile. Does not delete players.",
            ),
        }
    }

    fn run_each(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        for o in list {
            self.execute(w, run, e, *o);
        }
    }
}
impl BaseCommand for DeleteCommand {
    spec_fn!();
    fn execute_list(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        if list.len() > 1 {
            let from = e.mobile;
            let count = list.len();
            let content = format!(
                "You are about to delete {count} objects. This cannot be undone without a full server revert.<br><br>Continue?"
            );
            let Some(gump) = w.resources.get_mut::<CommandHooks>().warning_gump else {
                // no confirmation gump installed (`muo_gumps` sets it): nothing is deleted
                run.log_failure("The confirmation gump is not available.");
                return;
            };
            // The run is moved into the callback (its responses are kept across the gump);
            // the caller flushes the "Awaiting confirmation..." that is left.
            let mut taken = std::mem::take(run);
            let e2 = CommandEventArgs::new(
                e.mobile,
                e.command.clone(),
                e.arg_string.clone(),
                e.arguments.clone(),
            );
            let list2: Vec<Obj> = list.to_vec();
            run.add_response("Awaiting confirmation...");
            gump(
                w,
                from,
                &content,
                (420, 280),
                Box::new(move |w, okay| {
                    let cmd = DeleteCommand::new();
                    let mut flush_to_log = false;
                    if okay {
                        taken.add_response("Delete command confirmed.");
                        if list2.len() > 20 {
                            w.resources
                                .get_mut::<crate::logging::CommandLogging>()
                                .enabled = false;
                            // `NetState.FlushAll()`: packets are sent at the end of the tick.
                        }
                        cmd.run_each(w, &mut taken, &e2, &list2);
                        if list2.len() > 20 {
                            flush_to_log = true;
                            w.resources
                                .get_mut::<crate::logging::CommandLogging>()
                                .enabled = true;
                        }
                    } else {
                        taken.add_response("Delete command aborted.");
                    }
                    taken.flush(w, from, flush_to_log);
                }),
            );
        } else {
            self.run_each(w, run, e, list);
        }
    }

    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        if let Some(item) = as_item(obj) {
            log_action(w, e.mobile, "deleting", EntityId::Item(item));
            muo_server::item::base::delete(w, item);
            run.add_response("The item has been deleted.");
        } else if let Some(m) = as_mobile(obj).filter(|m| !w.mobile(*m).is_some_and(|x| x.player)) {
            log_action(w, e.mobile, "deleting", EntityId::Mobile(m));
            muo_server::mobile::base::delete(w, m);
            run.add_response("The mobile has been deleted.");
        } else {
            run.log_failure("That cannot be deleted.");
        }
    }
}

// ---- Kill / Resurrect -----------------------------------------------------------------------------------------

pub struct KillCommand {
    spec: CmdSpec,
    kill: bool,
}
impl KillCommand {
    fn new(kill: bool) -> Self {
        let spec = if kill {
            CmdSpec::new(
                &["Kill"],
                GM,
                S::ALL_MOBILES,
                O::Mobiles,
                "Kill",
                "Kills a targeted player or npc.",
            )
        } else {
            CmdSpec::new(
                &["Resurrect", "Res"],
                GM,
                S::ALL_MOBILES,
                O::Mobiles,
                "Resurrect",
                "Resurrects a targeted ghost.",
            )
        };
        Self { spec, kill }
    }
}
impl BaseCommand for KillCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(mob) = as_mobile(obj) else { return };
        let from = e.mobile;
        let target = EntityId::Mobile(mob);
        let alive = w.mobile(mob).is_some_and(|m| (m.ty.alive)(w, mob));
        if self.kill {
            if !alive {
                run.log_failure("They are already dead.");
            } else if !muo_server::mobile::base::can_be_damaged(w, mob) {
                run.log_failure("They cannot be harmed.");
            } else {
                log_action(w, from, "killing", target);
                muo_server::mobile::base::kill(w, mob);
                run.add_response("They have been killed.");
            }
        } else if muo_content::creature::overrides::is_dead_bonded_pet(w, mob) {
            if muo_content::factory::mobile_is(w, mob, "Server.Mobiles.BaseCreature") {
                log_action(w, from, "resurrecting", target);
                muo_server::mobile::part2::effects::play_sound(w, mob, 0x214);
                muo_server::mobile::part2::effects::fixed_effect(w, mob, 0x376A, 10, 16);
                muo_content::creature::misc::resurrect_pet(w, mob);
                run.add_response("It has been resurrected.");
            }
        } else if !alive {
            log_action(w, from, "resurrecting", target);
            muo_server::mobile::part2::effects::play_sound(w, mob, 0x214);
            muo_server::mobile::part2::effects::fixed_effect(w, mob, 0x376A, 10, 16);
            let ty = w.mobile(mob).map(|m| m.ty);
            if let Some(ty) = ty {
                (ty.resurrect)(w, mob);
            }
            run.add_response("They have been resurrected.");
        } else {
            run.log_failure("They are not dead.");
        }
    }
}

// ---- Hide / Unhide --------------------------------------------------------------------------------------------

pub struct HideCommand {
    spec: CmdSpec,
    hide: bool,
}
impl HideCommand {
    fn new(hide: bool) -> Self {
        let spec = if hide {
            CmdSpec::new(
                &["Hide"],
                CNS,
                S::ALL_MOBILES,
                O::Mobiles,
                "Hide",
                "Makes a targeted mobile disappear in a puff of smoke.",
            )
        } else {
            CmdSpec::new(
                &["Unhide"],
                CNS,
                S::ALL_MOBILES,
                O::Mobiles,
                "Unhide",
                "Makes a targeted mobile appear in a puff of smoke.",
            )
        };
        Self { spec, hide }
    }
}
impl BaseCommand for HideCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(m) = as_mobile(obj) else { return };
        let Some((loc, map)) = w.mobile(m).map(|x| (x.location, x.map)) else {
            return;
        };
        let text = format!(
            "{:?} {} {} {}",
            level_of(w, e.mobile),
            crate::logging::format_entity(w, EntityId::Mobile(e.mobile)),
            if self.hide { "hiding" } else { "unhiding" },
            crate::logging::format_entity(w, EntityId::Mobile(m))
        );
        write_line(w, e.mobile, &text);
        let (x, y, z) = (loc.x, loc.y, loc.z);
        for (dx, dy, dz) in [
            (1, 0, 4),
            (1, 0, 0),
            (1, 0, -4),
            (0, 1, 4),
            (0, 1, 0),
            (0, 1, -4),
            (1, 1, 11),
            (1, 1, 7),
            (1, 1, 3),
            (1, 1, -1),
        ] {
            muo_server::effects::send_location_effect(
                w,
                Point3D::new(x + dx, y + dy, z + dz),
                map,
                0x3728,
                13,
                10,
                0,
                0,
            );
        }
        muo_server::mobile::part2::effects::play_sound(w, m, 0x228);
        muo_server::mobile::part1::props_b::set_hidden(w, m, self.hide);
        run.add_response(if self.hide {
            "They have been hidden."
        } else {
            "They have been revealed."
        });
    }
}

// ---- Firewall / Kick / Ban ------------------------------------------------------------------------------------

pub struct FirewallCommand {
    spec: CmdSpec,
}
impl FirewallCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["Firewall"],
                ADM,
                S::ALL_MOBILES,
                O::Mobiles,
                "Firewall",
                "Adds a targeted player to the firewall (list of blocked IP addresses). This command does not ban or kick.",
            ),
        }
    }
}
impl BaseCommand for FirewallCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(targ) = as_mobile(obj) else { return };
        let ns = w.mobile(targ).and_then(|m| m.net_state);
        let Some(ns) = ns else {
            return run.log_failure("They are not online.");
        };
        log_action(w, e.mobile, "firewalling", EntityId::Mobile(targ));
        let addr = w.net_state(ns).and_then(|d| d.address);
        match (addr, w.resources.get_mut::<CommandHooks>().firewall_add) {
            (Some(a), Some(f)) => match f(w, a) {
                Ok(()) => run.add_response("They have been firewalled."),
                Err(m) => run.log_failure(&m),
            },
            _ => run.log_failure("The firewall is not available."),
        }
    }
}

pub struct KickCommand {
    spec: CmdSpec,
    ban: bool,
}
impl KickCommand {
    fn new(ban: bool) -> Self {
        let spec = if ban {
            CmdSpec::new(
                &["Ban"],
                ADM,
                S::ALL_MOBILES,
                O::Mobiles,
                "Ban",
                "Bans the account of a targeted player.",
            )
        } else {
            CmdSpec::new(
                &["Kick"],
                GM,
                S::ALL_MOBILES,
                O::Mobiles,
                "Kick",
                "Disconnects a targeted player.",
            )
        };
        Self { spec, ban }
    }
}
impl BaseCommand for KickCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(targ) = as_mobile(obj) else { return };
        let from = e.mobile;
        let (from_level, targ_level) = (level_of(w, from), level_of(w, targ));
        if from_level <= targ_level {
            return run.log_failure("You do not have the required access level to do this.");
        }
        let from_ns = w.mobile(from).and_then(|m| m.net_state);
        let targ_ns = w.mobile(targ).and_then(|m| m.net_state);
        match (from_ns, targ_ns) {
            (Some(_), Some(tns)) => {
                let targ_acct = w.mobile(targ).and_then(|m| m.account);
                let from_acct = w.mobile(from).and_then(|m| m.account);
                if let (Some(_), Some(ta)) = (from_acct, targ_acct) {
                    let verb = if self.ban { "banning" } else { "kicking" };
                    log_action(w, from, verb, EntityId::Mobile(targ));
                    muo_server::mobile::part2::messages::say(
                        w,
                        targ,
                        false,
                        if self.ban {
                            "I've been banned."
                        } else {
                            "I've been kicked"
                        },
                    );
                    run.add_response(&format!(
                        "They have been {}.",
                        if self.ban { "banned" } else { "kicked" }
                    ));
                    // `$"Banned by {from}."`: `Mobile.ToString()` is `{Serial} "{Name}"`
                    let from_text = muo_server::mobile::part1::combat::to_string(w, from);
                    w.disconnect(tns, &format!("Banned by {from_text}."));
                    if self.ban {
                        muo_content::accounting::set_banned(w, ta, true);
                        if let Some(a) = muo_content::accounting::account_mut(w, ta) {
                            a.set_unspecified_ban(Some(&from_text));
                        }
                        // `Account.SetUnspecifiedBan(from)`; the `BanDurationGump` is `muo_gumps`' (set in `a_l::hooks::configure`).
                        if let Some(g) = w.resources.get_mut::<CommandHooks>().ban_duration_gump {
                            g(w, from, targ);
                        }
                    }
                }
            }
            (_, None) => run.log_failure("They are not online."),
            _ => {}
        }
    }
}

// ---- FactionKick, TraceLockdown -------------------------------------------------------------------------------

pub struct FactionKickCommand {
    spec: CmdSpec,
    kind: u8,
}
impl FactionKickCommand {
    fn new(name: &str, kind: u8, description: &str) -> Self {
        let _ = send_localized_message;
        let _ = item_send_message;
        Self {
            spec: CmdSpec::new(&[name], GM, S::ALL_MOBILES, O::Mobiles, name, description),
            kind,
        }
    }
}
impl BaseCommand for FactionKickCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(m) = as_mobile(obj) else { return };
        match w.resources.get_mut::<CommandHooks>().faction_kick {
            Some(f) => {
                for (ok, text) in f(w, e.mobile, m, self.kind) {
                    if ok {
                        run.add_response(&text);
                    } else {
                        run.log_failure(&text);
                    }
                }
            }
            None => run.log_failure("Factions are not available."),
        }
    }
}

pub struct TraceLockdownCommand {
    spec: CmdSpec,
}
impl TraceLockdownCommand {
    fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["TraceLockdown"],
                ADM,
                S::SIMPLE,
                O::Items,
                "TraceLockdown",
                "Finds the BaseHouse for which a targeted item is locked down or secured.",
            ),
        }
    }
}
impl BaseCommand for TraceLockdownCommand {
    spec_fn!();
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(item) = as_item(obj) else { return };
        if !muo_server::item::ops::is_locked_down(w, item)
            && !muo_server::item::ops::is_secure(w, item)
        {
            run.log_failure("That is not locked down.");
            return;
        }
        // `CommandHooks::trace_lockdown` searches `BaseHouse.AllHouses` and opens the house's props gump.
        let found = match w.resources.get_mut::<CommandHooks>().trace_lockdown {
            Some(f) => f(w, e.mobile, item),
            None => false,
        };
        if !found {
            run.log_failure("No house was found.");
        }
    }
}

#[cfg(test)]
mod target_tests {
    use super::*;
    use muo_server::target::LandTarget;

    /// `[get` on a land or static target reads their `[CommandProperty]` members (it did nothing before).
    #[test]
    fn get_reads_the_properties_of_a_land_target() {
        let w = World::new();
        let land = Targeted::Land(LandTarget {
            location: Point3D::new(10, 20, -3),
            tile_id: 3,
        });
        assert_eq!(target_get_value(&w, land, "x"), "X = 10 (0xA)");
        assert_eq!(target_get_value(&w, land, "Z"), "Z = -3 (0xFFFFFFFD)");
        assert_eq!(
            target_get_value(&w, land, "Location"),
            "Location = (10, 20, -3)"
        );
        assert_eq!(target_get_value(&w, land, "TileID"), "TileID = 3 (0x3)");
        assert_eq!(target_get_value(&w, land, "ItemID"), "Property not found.");
        assert_eq!(target_get_value(&w, land, "Hue"), "Property not found.");
    }
}
