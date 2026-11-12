//! `HelpInfo` and `GenCommands`: the command help table, `[HelpInfo`, its two gumps and
//! `[GenCommands`.
//!
//! A handler's usage, description and aliases are the [`META`] rows (command name, usage, description, aliases) for the handlers this crate registers; a command from
//! another crate has no row and gets the defaults (its name as the usage, "No description available."), the
//! text a handler without attributes gets. Commands the generic system registers (`[global`, `[delete`..) are
//! described by their command object.

use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use std::collections::HashSet;

use indexmap::IndexMap;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::{DynamicGumpBuilder, HtmlStyle};
use muo_content::gumps::grid::GridBuilderExt;
use muo_content::gumps::grid::{GridCell, GridEntryStyle, compute_from_spec_gap};
use muo_primitives::html::TextAlignment;
use muo_server::commands::{CommandEntry, CommandEventArgs, CommandSystem};
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::net_state::NetStateId;
use muo_server::world::World;
use serde_json::{Map, Value, json};

use crate::generic::GenericCommands;
use muo_content::gumps::base_grid::{mobile_of, send};
use muo_primitives::utility::fix_html;

/// `CommandInfo`.
#[derive(Clone, Debug)]
pub struct CommandInfo {
    pub access_level: AccessLevel,
    pub name: String,
    pub aliases: Vec<String>,
    pub usage: String,
    pub modifiers: Option<Vec<String>>,
    pub description: String,
}

/// `HelpInfo`'s statics.
#[derive(Default)]
pub struct HelpTable {
    sorted: Option<Vec<CommandInfo>>,
    infos: IndexMap<String, usize>,
    by_level: IndexMap<u8, Vec<CommandInfo>>,
}

/// `(command, usage, description, aliases)`: the attributes of the handlers registered in this crate.
const META: &[(&str, &str, &str, &[&str])] = &[
    (
        "Go",
        "Go [name | serial | (x y [z]) | (deg min (N | S) deg min (E | W))]",
        "With no arguments, this command brings up the go menu. With one argument, (name), you are moved to that regions \"go location.\" Or, if a numerical value is specified for one argument, (serial), you are moved to that object. Two or three arguments, (x y [z]), will move your character to that location. When six arguments are specified, (deg min (N | S) deg min (E | W)), your character will go to an approximate of those sextant coordinates.",
        &[],
    ),
    (
        "DropHolding",
        "DropHolding",
        "Drops the item, if any, that a targeted player is holding. The item is placed into their backpack, or if that's full, at their feet.",
        &[],
    ),
    (
        "GetFollowers",
        "GetFollowers",
        "Teleports all pets of a targeted player to your location.",
        &[],
    ),
    (
        "Where",
        "Where",
        "Tells the commanding player his coordinates, region, and facet.",
        &[],
    ),
    (
        "AutoPageNotify",
        "AutoPageNotify",
        "Toggles your auto-page-notify status.",
        &["APN"],
    ),
    (
        "Animate",
        "Animate <action> <frameCount> <repeatCount> <forward> <repeat> <delay>",
        "Makes your character do a specified animation.",
        &[],
    ),
    ("Cast", "Cast <name>", "Casts a spell by name.", &[]),
    (
        "Stuck",
        "Stuck",
        "Opens a menu of towns, used for teleporting stuck mobiles.",
        &[],
    ),
    ("Help", "Help", "Lists all available commands.", &[]),
    (
        "Move",
        "Move",
        "Repositions a targeted item or mobile.",
        &[],
    ),
    (
        "Client",
        "Client",
        "Opens the client gump menu for a given player.",
        &[],
    ),
    (
        "SMsg",
        "SMsg <text>",
        "Broadcasts a message to all online staff.",
        &["S", "SM"],
    ),
    (
        "BCast",
        "BCast <text>",
        "Broadcasts a message to everyone online.",
        &["B", "BC"],
    ),
    ("Bank", "Bank", "Opens the bank box of a given target.", &[]),
    (
        "Echo",
        "Echo <text>",
        "Relays (text) as a system message.",
        &[],
    ),
    (
        "Sound",
        "Sound <index> [toAll=true]",
        "Plays a sound to players within 12 tiles of you. The (toAll) argument specifies to everyone, or just those who can see you.",
        &[],
    ),
    (
        "ViewEquip",
        "ViewEquip",
        "Lists equipment of a targeted mobile. From the list you can move, delete, or open props.",
        &[],
    ),
    ("Light", "Light <level>", "Set your local lightlevel.", &[]),
    ("Stats", "Stats", "View some stats about the server.", &[]),
    (
        "SpeedBoost",
        "SpeedBoost [true|false]",
        "Enables a speed boost for the invoker.  Disable with parameters.",
        &[],
    ),
    ("Time", "Time", "Returns the server's local time.", &[]),
    (
        "ResetLocalization",
        "ResetLocalization [<command>]",
        "Clears the localization table. Cliloc will be reloaded the next time they are used for that language.",
        &["ResetLoc"],
    ),
    (
        "ResetRng",
        "ResetRng",
        "Resets the global RNG. Use with care! Do not reset often as this will affect the over-all distribution.",
        &[],
    ),
    (
        "DragEffects",
        "DragEffects [enable=false]",
        "Enables or disables the item drag and drop effects.",
        &[],
    ),
    (
        "Skills",
        "Skills",
        "Opens a menu where you can view or edit skills of a targeted mobile.",
        &[],
    ),
    (
        "ResetStaffDress",
        "ResetStaffDress",
        "Resets staff to proper GM",
        &[],
    ),
    (
        "StaffAccess",
        "StaffAccess <access level>",
        "Changes your access level. Only access levels at or below your current access level can be set. Use the same access level to reset.",
        &[],
    ),
    (
        "MovementDebug",
        "MovementDebug",
        "Target a player to toggle verbose movement logging for their connection.",
        &[],
    ),
    (
        "MovementStats",
        "MovementStats",
        "Target a player to see their current movement throttle state.",
        &[],
    ),
    (
        "Wipe",
        "Wipe",
        "Wipes all items and npcs in a targeted bounding box.",
        &[],
    ),
    (
        "WipeItems",
        "WipeItems",
        "Wipes all items in a targeted bounding box.",
        &[],
    ),
    (
        "WipeNPCs",
        "WipeNPCs",
        "Wipes all npcs in a targeted bounding box.",
        &[],
    ),
    (
        "WipeMultis",
        "WipeMultis",
        "Wipes all multis in a targeted bounding box.",
        &[],
    ),
    (
        "SetSkill",
        "SetSkill <name> <value>",
        "Sets a skill value by name of a targeted mobile.",
        &[],
    ),
    (
        "GetSkill",
        "GetSkill <name>",
        "Gets a skill value by name of a targeted mobile.",
        &[],
    ),
    (
        "SetAllSkills",
        "SetAllSkills <name> <value>",
        "Sets all skill values of a targeted mobile.",
        &[],
    ),
    (
        "Vis",
        "Vis",
        "Adds or removes a targeted player from your visibility list.  Anyone on your visibility list will be able to see you at all times, even when you're hidden.",
        &[],
    ),
    (
        "VisList",
        "VisList",
        "Shows the names of everyone in your visibility list.",
        &[],
    ),
    (
        "VisClear",
        "VisClear",
        "Removes everyone from your visibility list.",
        &[],
    ),
    (
        "Batch",
        "Batch",
        "Allows multiple commands to be run at the same time.",
        &[],
    ),
    (
        "ExportWSC",
        "ExportWSC <file-path>",
        "Exports all static items to a WSC file.  This will delete all static items in the world.  Please make a backup.",
        &[],
    ),
    (
        "SignGen",
        "SignGen",
        "Generates world/shop signs on all facets.",
        &[],
    ),
    ("Dupe", "Dupe [amount]", "Dupes a targeted item.", &[]),
    (
        "DupeInBag",
        "DupeInBag <count>",
        "Dupes an item at it's current location (count) number of times.",
        &[],
    ),
    (
        "ClearFacet",
        "ClearFacet",
        "Deletes all items and mobiles in your facet. Players and their inventory will not be deleted.",
        &[],
    ),
    (
        "ClearAll",
        "ClearAll",
        "Deletes all items and mobiles in all facets. Players and their inventory will not be deleted.",
        &[],
    ),
    (
        "ClearXY",
        "ClearXY x1 y1 x2 y2",
        "Deletes all items and mobiles in the specified rectangular area on your facet. Players and their inventory will not be deleted.",
        &[],
    ),
    (
        "SaveStability",
        "SaveStability [delaySeconds=60] [sampleStride=1] | SaveStability cancel",
        "Hashes every entity, waits, hashes again, and reports the types whose serialized bytes changed.",
        &[],
    ),
    (
        "GenCommands",
        "GenCommands",
        "Generates a webpage to the web folder with a list of all commands.",
        &["GenCmdWeb"],
    ),
    (
        "HelpInfo",
        "HelpInfo [<command>]",
        "Gives information on a specified command, or when no argument specified, displays a gump containing all commands",
        &[],
    ),
    (
        "Props",
        "Props [serial]",
        "Opens a menu where you can view and edit all properties of a targeted (or specified) object.",
        &[],
    ),
    (
        "Tile",
        "Tile <name> [params] [set {<propertyName> <value> ...}]",
        "Tiles an item or npc by name into a targeted bounding box. Optional constructor parameters. Optional set property list.",
        &[],
    ),
    (
        "TileRXYZ",
        "TileRXYZ <x> <y> <w> <h> <z> <name> [params] [set {<propertyName> <value> ...}]",
        "Tiles an item or npc by name into a given bounding box, (x, y) parameters are relative to your characters position. Optional constructor parameters. Optional set property list.",
        &[],
    ),
    (
        "TileXYZ",
        "TileXYZ <x> <y> <w> <h> <z> <name> [params] [set {<propertyName> <value> ...}]",
        "Tiles an item or npc by name into a given bounding box. Optional constructor parameters. Optional set property list.",
        &[],
    ),
    (
        "TileZ",
        "TileZ <z> <name> [params] [set {<propertyName> <value> ...}]",
        "Tiles an item or npc by name into a targeted bounding box at a fixed Z location. Optional constructor parameters. Optional set property list.",
        &[],
    ),
    (
        "TileAvg",
        "TileAvg <name> [params] [set {<propertyName> <value> ...}]",
        "Tiles an item or npc by name into a targeted bounding box on the map's average Z elevation. Optional constructor parameters. Optional set property list.",
        &[],
    ),
    (
        "Outline",
        "Outline <name> [params] [set {<propertyName> <value> ...}]",
        "Tiles an item or npc by name around a targeted bounding box. Optional constructor parameters. Optional set property list.",
        &[],
    ),
    (
        "OutlineRXYZ",
        "OutlineRXYZ <x> <y> <w> <h> <z> <name> [params] [set {<propertyName> <value> ...}]",
        "Tiles an item or npc by name around a given bounding box, (x, y) parameters are relative to your characters position. Optional constructor parameters. Optional set property list.",
        &[],
    ),
    (
        "OutlineXYZ",
        "OutlineXYZ <x> <y> <w> <h> <z> <name> [params] [set {<propertyName> <value> ...}]",
        "Tiles an item or npc by name around a given bounding box. Optional constructor parameters. Optional set property list.",
        &[],
    ),
    (
        "OutlineZ",
        "OutlineZ <z> <name> [params] [set {<propertyName> <value> ...}]",
        "Tiles an item or npc by name around a targeted bounding box at a fixed Z location. Optional constructor parameters. Optional set property list.",
        &[],
    ),
    (
        "OutlineAvg",
        "OutlineAvg <name> [params] [set {<propertyName> <value> ...}]",
        "Tiles an item or npc by name around a targeted bounding box on the map's average Z elevation. Optional constructor parameters. Optional set property list.",
        &[],
    ),
];

pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "HelpInfo",
        AccessLevel::Player,
        &[],
        std::rc::Rc::new(help_info_on_command),
    );
}

/// `HelpInfo.Initialize`.
pub fn initialize(w: &mut World) {
    fill_table(w);
}

fn level_key(l: AccessLevel) -> u8 {
    l as u8
}

/// `HelpInfo.FillTable`.
pub fn fill_table(w: &mut World) {
    let mut list: Vec<CommandInfo> = Vec::new();
    // Names the generic system owns: its accessor commands and the redirects of single-object commands.
    let (generic_names, accessors): (HashSet<String>, HashSet<String>) = {
        let gc = w.resources.get_mut::<GenericCommands>();
        let mut names = HashSet::new();
        for c in &gc.all {
            for n in &c.spec().commands {
                names.insert(n.lower_inv());
            }
        }
        let mut acc = HashSet::new();
        for i in &gc.implementors {
            for a in i.accessors {
                acc.insert(a.lower_inv());
            }
        }
        (names, acc)
    };
    for e in CommandEntry::get_list(w) {
        let key = e.command.lower_inv();
        if generic_names.contains(&key) || accessors.contains(&key) {
            continue; // `e.Handler.Target != null && usage == null && desc == null`
        }
        let meta = META.iter().find(|m| m.0.eq_ic(&e.command));
        list.push(CommandInfo {
            access_level: e.access_level,
            name: e.command.clone(),
            aliases: meta
                .map(|m| m.3.iter().map(|s| (*s).to_string()).collect())
                .unwrap_or_default(),
            usage: meta.map_or_else(|| e.command.clone(), |m| m.1.to_string()),
            modifiers: None,
            description: meta.map_or_else(
                || "No description available.".to_string(),
                |m| fix_html(m.2),
            ),
        });
    }
    let gc = w.resources.get_mut::<GenericCommands>();
    for command in &gc.all {
        let s = command.spec();
        if s.usage.is_empty() && s.description.is_empty() {
            continue;
        }
        let cmds = &s.commands;
        let modifiers: Vec<String> = gc
            .implementors
            .iter()
            .filter(|i| i.support.intersects(s.supports))
            .map(|i| i.accessors[0].to_string())
            .collect();
        list.push(CommandInfo {
            access_level: s.access_level,
            name: cmds[0].clone(),
            aliases: cmds[1..].to_vec(),
            usage: s.usage.clone(),
            modifiers: Some(modifiers),
            description: fix_html(&s.description),
        });
    }
    for i in &gc.implementors {
        list.push(CommandInfo {
            access_level: i.access_level,
            name: i.accessors[0].to_string(),
            aliases: i.accessors[1..].iter().map(|s| (*s).to_string()).collect(),
            usage: i.usage.to_string(),
            modifiers: None,
            description: fix_html(i.description),
        });
    }
    // CommandInfoSorter: highest access level first, then ordinal by name.
    list.sort_by(|a, b| {
        b.access_level
            .cmp(&a.access_level)
            .then_with(|| a.name.cmp(&b.name))
    });
    let mut infos = IndexMap::new();
    for (i, c) in list.iter().enumerate() {
        infos.entry(c.name.lower_inv()).or_insert(i);
    }
    let t = w.resources.get_mut::<HelpTable>();
    t.sorted = Some(list);
    t.infos = infos;
    t.by_level.clear();
}

fn table(w: &mut World) -> &mut HelpTable {
    if w.resources.get_mut::<HelpTable>().sorted.is_none() {
        fill_table(w);
    }
    w.resources.get_mut::<HelpTable>()
}

/// `HelpInfo.SortedHelpInfo`.
pub fn sorted_help_info(w: &mut World) -> Vec<CommandInfo> {
    table(w).sorted.clone().unwrap_or_default()
}

/// `HelpInfo.GetCommandsForAccessLevel`.
pub fn commands_for_access_level(w: &mut World, level: AccessLevel) -> Vec<CommandInfo> {
    let t = table(w);
    if let Some(c) = t.by_level.get(&level_key(level)) {
        return c.clone();
    }
    let list: Vec<CommandInfo> = t
        .sorted
        .iter()
        .flatten()
        .filter(|c| level >= c.access_level)
        .cloned()
        .collect();
    t.by_level.insert(level_key(level), list.clone());
    list
}

/// `[Usage("HelpInfo [<command>]")]`
fn help_info_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let m = e.mobile;
    if e.length() > 0 {
        let arg = e.get_string(0).lower_inv();
        let found = {
            let t = table(w);
            t.infos
                .get(&arg)
                .and_then(|i| t.sorted.as_ref().and_then(|s| s.get(*i)).cloned())
        };
        if let Some(c) = found {
            let level = w.mobile(m).map_or(AccessLevel::Player, |x| x.access_level);
            if level >= c.access_level {
                send(w, m, new_info_gump(c, 320, 200));
            } else {
                send_message(w, m, "You don't have access to that command.");
            }
            return;
        }
        send_message(w, m, &format!("Command '{arg}' not found!"));
    }
    let g = new_list_gump(w, 0, m);
    send(w, m, g);
}

// ---- CommandListGump -------------------------------------------------------------------------------------------

const ENTRIES_PER_PAGE: usize = 15;
const CONTENT_WIDTH: i32 = 360;
const MAX_ROWS: i32 = 16;
const COLUMN_SPEC: &str = "20 320 20";
const STYLE: GridEntryStyle = GridEntryStyle::DEFAULT;

pub struct CommandListGump {
    access_level: AccessLevel,
    page: usize,
    list: Vec<CommandInfo>,
}

pub fn new_list_gump(
    w: &mut World,
    page: usize,
    from: MobileId,
) -> DynamicGumpObj<CommandListGump> {
    let level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    let list = commands_for_access_level(w, level);
    DynamicGumpObj::new(
        30,
        30,
        CommandListGump {
            access_level: level,
            page,
            list,
        },
    )
}

impl DynamicGumpLogic for CommandListGump {
    const TYPE_NAME: &'static str = "Server.Commands.HelpInfo+CommandListGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let list = &self.list;
        let total_width = STYLE.total_width(CONTENT_WIDTH);
        let total_height = STYLE.total_height(MAX_ROWS);
        let mut col_pos = [0i32; 3];
        let mut col_widths = [0i32; 3];
        compute_from_spec_gap(
            COLUMN_SPEC,
            CONTENT_WIDTH,
            STYLE.content_origin_x(),
            STYLE.offset_size,
            &mut col_pos,
            &mut col_widths,
        );
        b.add_grid_background(total_width, total_height, &STYLE);
        let mut row_y = STYLE.content_origin_y();
        let total_pages = list.len().div_ceil(ENTRIES_PER_PAGE);

        let cell =
            |i: usize, y: i32| GridCell::new(col_pos[i], y, col_widths[i], STYLE.entry_height);
        let (h0, h1, h2) = (cell(0, row_y), cell(1, row_y), cell(2, row_y));
        if self.page > 0 {
            b.add_entry_arrow_left(&h0, &STYLE, 1);
        } else {
            b.add_entry_header(&h0, &STYLE);
        }
        b.add_image_tiled_cell(&h1, STYLE.entry_gump_id);
        b.add_html_cell_offset(
            &h1,
            &format!("Page {} of {}", self.page + 1, total_pages),
            STYLE.text_offset_x,
            0,
            HtmlStyle {
                align: TextAlignment::Center,
                ..HtmlStyle::NONE
            },
            false,
            false,
        );
        if (self.page + 1) * ENTRIES_PER_PAGE < list.len() {
            b.add_entry_arrow_right(&h2, &STYLE, 2);
        } else {
            b.add_entry_header(&h2, &STYLE);
        }

        // Data rows
        let mut last = AccessLevel::Player as i32 - 1;
        let data_content_width = col_widths[0] + STYLE.offset_size + col_widths[1];
        let mut line = 0;
        let mut i = self.page * ENTRIES_PER_PAGE;
        while line < ENTRIES_PER_PAGE && i < list.len() {
            let c = &list[i];
            // Access level separator
            if c.access_level as i32 != last {
                row_y += STYLE.entry_height + STYLE.offset_size;
                let sep_content =
                    GridCell::new(col_pos[0], row_y, data_content_width, STYLE.entry_height);
                let sep_button = cell(2, row_y);
                b.add_image_tiled_cell(&sep_content, STYLE.entry_gump_id);
                b.add_html_cell_offset(
                    &sep_content,
                    &format!("{:?}", c.access_level),
                    STYLE.text_offset_x,
                    0,
                    HtmlStyle {
                        color: muo_content::gumps::base::text_colors::BRIGHT_RED,
                        ..HtmlStyle::NONE
                    },
                    false,
                    false,
                );
                b.add_entry_header(&sep_button, &STYLE);
                line += 1;
            }
            last = c.access_level as i32;

            // Entry row
            row_y += STYLE.entry_height + STYLE.offset_size;
            let content = GridCell::new(col_pos[0], row_y, data_content_width, STYLE.entry_height);
            let button = cell(2, row_y);
            b.add_image_tiled_cell(&content, STYLE.entry_gump_id);
            if c.aliases.is_empty() {
                b.add_html_cell_offset(
                    &content,
                    &c.name,
                    STYLE.text_offset_x,
                    0,
                    HtmlStyle::NONE,
                    false,
                    false,
                );
            } else {
                let text = format!("{} <i>({})</i>", c.name, c.aliases.join(", "));
                b.add_html_cell_offset(
                    &content,
                    &text,
                    STYLE.text_offset_x,
                    0,
                    HtmlStyle::NONE,
                    false,
                    false,
                );
            }
            b.add_entry_arrow_right(&button, &STYLE, 3 + i as i32);
            i += 1;
            line += 1;
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(m) = mobile_of(w, sender) else {
            return;
        };
        let list = commands_for_access_level(w, self.access_level);
        match info.button_id {
            0 => {
                muo_content::gumps::close_gump::<CommandInfoGump>(w, sender);
                return;
            }
            1 => {
                if self.page == 0 {
                    return;
                }
                self.page -= 1;
            }
            2 => {
                if (self.page + 1) * ENTRIES_PER_PAGE >= list.len() {
                    return;
                }
                self.page += 1;
            }
            b => {
                let v = b - 3;
                if v < 0 || v as usize >= list.len() {
                    return;
                }
                let c = list[v as usize].clone();
                let level = w.mobile(m).map_or(AccessLevel::Player, |x| x.access_level);
                if level >= c.access_level {
                    send(w, m, new_info_gump(c, 320, 200));
                } else {
                    send_message(w, m, "You no longer have access to that command.");
                    self.access_level = level;
                }
            }
        }
        let g = DynamicGumpObj::new(
            30,
            30,
            CommandListGump {
                access_level: self.access_level,
                page: self.page,
                list: commands_for_access_level(w, self.access_level),
            },
        );
        send(w, m, g);
    }
}

// ---- CommandInfoGump -------------------------------------------------------------------------------------------

pub struct CommandInfoGump {
    width: i32,
    height: i32,
    info: CommandInfo,
}

pub fn new_info_gump(
    info: CommandInfo,
    width: i32,
    height: i32,
) -> DynamicGumpObj<CommandInfoGump> {
    DynamicGumpObj::new(
        300,
        50,
        CommandInfoGump {
            width,
            height,
            info,
        },
    )
}

impl DynamicGumpLogic for CommandInfoGump {
    const TYPE_NAME: &'static str = "Server.Commands.HelpInfo+CommandInfoGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, self.width, self.height, 5054);
        b.add_html_styled(
            10,
            10,
            self.width - 20,
            20,
            &self.info.name,
            HtmlStyle {
                color: "#FF0000",
                align: TextAlignment::Center,
                ..HtmlStyle::NONE
            },
            false,
            false,
        );
        let i = &self.info;
        let mut sb = String::from("Usage: ");
        sb.push_str(&fix_html(&i.usage));
        sb.push_str("<BR>");
        if !i.aliases.is_empty() {
            sb.push_str(&format!(
                "Alias{}: {}<BR>",
                if i.aliases.len() == 1 { "" } else { "es" },
                i.aliases.join(", ")
            ));
        }
        sb.push_str(&format!("AccessLevel: {:?}<BR>", i.access_level));
        if let Some(mods) = i.modifiers.as_ref().filter(|m| !m.is_empty()) {
            sb.push_str(&format!("Modifiers: {}<BR><BR>", mods.join(", ")));
        }
        sb.push_str(&i.description);
        b.add_html(10, 40, self.width - 20, self.height - 80, &sb, false, true);
    }
}

// ---- GenCommands -----------------------------------------------------------------------------------------------

fn command_json(c: &CommandInfo) -> Value {
    let mut o = Map::new();
    o.insert("accessLevel".into(), json!(format!("{:?}", c.access_level)));
    o.insert("name".into(), json!(c.name));
    o.insert("aliases".into(), json!(c.aliases));
    o.insert("usage".into(), json!(c.usage));
    if let Some(m) = &c.modifiers {
        o.insert("modifiers".into(), json!(m));
    }
    o.insert("description".into(), json!(c.description));
    Value::Object(o)
}

/// A `[Flags]` enum with one set bit is written as its name.
fn support_name(bits: u32) -> String {
    const NAMES: [(u32, &str); 9] = [
        (0x1, "Single"),
        (0x2, "Global"),
        (0x4, "Online"),
        (0x8, "Multi"),
        (0x10, "Area"),
        (0x20, "Self"),
        (0x40, "Region"),
        (0x80, "Contained"),
        (0x100, "IPAddress"),
    ];
    NAMES
        .iter()
        .filter(|(b, _)| bits & b != 0)
        .map(|(_, n)| *n)
        .collect::<Vec<_>>()
        .join(", ")
}

/// `[Usage("GenCommands")]`
pub fn gen_commands_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let all = sorted_help_info(w);
    let mut modifiers = Map::new();
    if let Some(gc) = w.resources.get::<GenericCommands>() {
        for i in &gc.implementors {
            modifiers.insert(
                i.accessors[0].to_string(),
                json!({
                    "supportsConditionals": i.supports_conditionals,
                    "accessors": i.accessors,
                    "usage": i.usage,
                    "description": i.description,
                    "accessLevel": format!("{:?}", i.access_level),
                    "supportRequirement": support_name(i.support.bits()),
                }),
            );
        }
    }
    // Dictionary<AccessLevel, List<CommandInfo>>, in the order the levels first appear (highest first).
    let mut by_level: IndexMap<AccessLevel, Vec<Value>> = IndexMap::new();
    for c in &all {
        by_level
            .entry(c.access_level)
            .or_default()
            .push(command_json(c));
    }
    let mut commands = Map::new();
    for (l, list) in by_level {
        commands.insert(format!("{l:?}"), Value::Array(list));
    }
    let modifiers_json =
        serde_json::to_string_pretty(&Value::Object(modifiers)).unwrap_or_default();
    let commands_json = serde_json::to_string_pretty(&Value::Object(commands)).unwrap_or_default();
    let base = w.core.base_directory.clone();
    // Template is in Distribution/Data/commands
    let template_path = base.join("Data").join("commands").join("template.html");
    let template = match std::fs::read_to_string(&template_path) {
        Ok(t) => t,
        Err(err) => {
            send_message(w, from, &err.to_string());
            return;
        }
    };
    let template = template
        .replace(
            "<!-- COMMANDS -->",
            &muo_primitives::html::escape_html(&commands_json),
        )
        .replace(
            "<!-- MODIFIERS -->",
            &muo_primitives::html::escape_html(&modifiers_json),
        );
    // Output to Distribution/web folder (typically ignored by servers for external systems)
    let web = base.join("web");
    let output = web.join("commands.html");
    if let Err(err) = std::fs::create_dir_all(&web)
        .and_then(|()| std::fs::write(&output, format!("{}{template}", '\u{feff}')))
    {
        send_message(w, from, &err.to_string());
        return;
    }
    send_message(
        w,
        from,
        &format!("Commands webpage generated at {}.", output.display()),
    );
}
