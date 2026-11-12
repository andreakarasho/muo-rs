//! `[TelGen`, `[TelGenDelete` and `[DesignInsert`.
//!
//! `new Teleporter(dest, map)` and the `HouseFoundation` design states are other crates' classes. They are
//! `CommandHooks::new_teleporter` and `CommandHooks::design_insert` (set by the engines crate).

use std::rc::Rc;

use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::{send_message, send_message_hue};
use muo_server::target::{self, Target, TargetFlags, Targeted};
use muo_server::world::World;
use muo_server::world_location::WorldLocation;

use crate::generic::{
    BaseCommand, CmdSpec, CommandSupport as S, Obj, ObjectTypes as O, Run, as_item,
};
use crate::hooks::CommandHooks;

const SUCCESS_HUE: i32 = 72;
const WARNING_HUE: i32 = 53;
const ERROR_HUE: i32 = 33;

pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "TelGen",
        AccessLevel::Developer,
        &[],
        Rc::new(gen_teleporter_on_command),
    );
    CommandSystem::register(
        w,
        "TelGenDelete",
        AccessLevel::Developer,
        &[],
        Rc::new(tel_gen_delete_on_command),
    );
}

/// `TeleporterDefinition`.
struct Definition {
    source: WorldLocation,
    destination: WorldLocation,
    back: bool,
}

/// `ProcessTeleporterData`: the definitions, or `Err` after telling the player the file failed.
fn load(w: &mut World, m: MobileId) -> Result<Vec<Definition>, ()> {
    let path = w.core.base_directory.join("Data/teleporters.json");
    let parsed = (|| -> Result<Vec<Definition>, String> {
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let v: serde_json::Value = muo_data::json::from_str(text.trim_start_matches('\u{feff}'))
            .map_err(|e| e.to_string())?;
        let list = v
            .as_array()
            .ok_or("The JSON value could not be converted to a list.")?;
        list.iter()
            .map(|d| {
                let loc = |k: &str| {
                    WorldLocation::from_json(&w.maps, d.get(k).unwrap_or(&serde_json::Value::Null))
                };
                Ok(Definition {
                    source: loc("src")?,
                    destination: loc("dst")?,
                    back: d.get("back").and_then(|b| b.as_bool()).unwrap_or(false),
                })
            })
            .collect()
    })();
    match parsed {
        Ok(l) => Ok(l),
        Err(err) => {
            println!("{err}");
            send_message_hue(
                w,
                m,
                ERROR_HUE,
                &format!("Failed to load/process data file '{}'", path.display()),
            );
            Err(())
        }
    }
}

fn is_within_z(delta: i32) -> bool {
    (-12..=12).contains(&delta)
}

fn short_in_chain(w: &World, e: EntityId, name: &str) -> bool {
    muo_content::props::class_chain(w, e)
        .iter()
        .any(|c| c.rsplit(['.', '+']).next() == Some(name))
}

/// `TeleportersCreator.DeleteTeleporters(worldLocation)`.
fn delete_teleporters(w: &mut World, at: WorldLocation) -> i32 {
    let found: Vec<ItemId> = muo_server::map::enumerate::items_at(w, at.map, at.loc.x, at.loc.y)
        .filter(|i| {
            let e = EntityId::Item(*i);
            short_in_chain(w, e, "Teleporter")
                && !(short_in_chain(w, e, "KeywordTeleporter")
                    || short_in_chain(w, e, "SkillTeleporter"))
                && w.item(*i)
                    .is_some_and(|x| is_within_z(x.location.z - at.loc.z))
        })
        .collect();
    let n = found.len() as i32;
    for i in found {
        muo_server::item::base::delete(w, i);
    }
    n
}

/// `new Teleporter(dest, destMap).MoveToWorld(src, srcMap)`.
fn place_teleporter(w: &mut World, dest: WorldLocation, src: WorldLocation) {
    let Some(f) = w.resources.get_mut::<CommandHooks>().new_teleporter else {
        return;
    };
    if let (Some(item), Some(map)) = (f(w, dest.loc, dest.map), src.map) {
        muo_server::item::ops::move_to_world_map(w, item, src.loc, Some(map));
    }
}

/// `[Usage("TelGenDelete")]`
fn tel_gen_delete_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    send_message(w, from, "Removing teleporters, please wait.");
    let mut count = 0;
    let Ok(defs) = load(w, from) else {
        return;
    };
    for d in defs {
        count += delete_teleporters(w, d.source);
        if d.back {
            count += delete_teleporters(w, d.destination);
        }
    }
    send_message_hue(
        w,
        from,
        WARNING_HUE,
        &format!("{count} Teleporters Removed."),
    );
}

/// `[Usage("TelGen")]`
fn gen_teleporter_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    send_message(w, from, "Generating teleporters, please wait.");
    let Ok(defs) = load(w, from) else {
        return;
    };
    let (mut count, mut del_count) = (0, 0);
    for d in defs {
        del_count += delete_teleporters(w, d.source);
        count += 1;
        place_teleporter(w, d.destination, d.source);
        if !d.back {
            continue;
        }
        del_count += delete_teleporters(w, d.destination);
        count += 1;
        place_teleporter(w, d.source, d.destination);
    }
    send_message_hue(w, from, SUCCESS_HUE, "Teleporter generating complete.");
    send_message_hue(
        w,
        from,
        WARNING_HUE,
        &format!("{del_count} Teleporters Removed."),
    );
    send_message_hue(w, from, SUCCESS_HUE, &format!("{count} Teleporters Added."));
}

// ---- DesignInsert ----------------------------------------------------------------------------------------------

/// `DesignInsertCommand.DesignInsertResult`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DesignInsertResult {
    Valid,
    InvalidItem,
    NotInHouse,
    OutsideHouseBounds,
}

/// `ProcessInsert`'s result and the foundation it found.
pub type InsertOutcome = (DesignInsertResult, Option<ItemId>);

/// `ProcessInsert(item, staticsOnly, out house)`; the part before the house lookup is here, the design states are
/// the hook's.
pub fn process_insert(w: &mut World, item: Option<ItemId>, statics_only: bool) -> InsertOutcome {
    let Some(item) = item else {
        return (DesignInsertResult::InvalidItem, None);
    };
    let e = EntityId::Item(item);
    let is_static = muo_content::props::is_class(w, e, "Server.Items.Static");
    if short_in_chain(w, e, "BaseMulti")
        || short_in_chain(w, e, "HouseSign")
        || (statics_only && !is_static)
    {
        return (DesignInsertResult::InvalidItem, None);
    }
    match w.resources.get_mut::<CommandHooks>().design_insert {
        Some(f) => f(w, item),
        None => (DesignInsertResult::NotInHouse, None),
    }
}

pub struct DesignInsertCommand {
    spec: CmdSpec,
}

impl DesignInsertCommand {
    pub fn new() -> Self {
        Self {
            spec: CmdSpec::new(
                &["DesignInsert"],
                AccessLevel::GameMaster,
                S::SINGLE | S::AREA,
                O::Items,
                "DesignInsert [allItems=false]",
                "Inserts multiple targeted items into a customizable house's design.",
            ),
        }
    }
}

impl Default for DesignInsertCommand {
    fn default() -> Self {
        Self::new()
    }
}

fn statics_only(e: &CommandEventArgs) -> bool {
    e.length() < 1 || !e.get_boolean(0)
}

impl BaseCommand for DesignInsertCommand {
    fn spec(&self) -> &CmdSpec {
        &self.spec
    }

    fn execute(&self, w: &mut World, _run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        // `Target t = new DesignInsertTarget(..); t.Invoke(e.Mobile, obj)`
        let t = new_target(w, Vec::new(), statics_only(e));
        target::invoke(w, t, e.mobile, obj);
    }

    fn execute_list(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        let from = e.mobile;
        let items: Vec<Option<ItemId>> = list.iter().map(|o| as_item(*o)).collect();
        let only = statics_only(e);
        let count = items.len();
        let Some(gump) = w.resources.get_mut::<CommandHooks>().warning_gump else {
            run.log_failure("The confirmation gump is not available.");
            return;
        };
        // `InsertObjectsNoticeGump`: width 420, height 280.
        let content = format!(
            "You are about to insert {count} objects. This cannot be undone without a full server revert.<br><br>Continue?"
        );
        gump(
            w,
            from,
            &content,
            (420, 280),
            Box::new(move |w, okay| on_confirm(w, from, okay, &items, only)),
        );
        run.add_response("Awaiting confirmation...");
    }
}

/// `OnConfirmCallback`; the command object's own `AddResponse` / `Flush` are a fresh [`Run`] here.
fn on_confirm(w: &mut World, from: MobileId, okay: bool, items: &[Option<ItemId>], only: bool) {
    let mut run = Run::default();
    let mut flush_to_log = false;
    if okay {
        let mut foundations: Vec<ItemId> = Vec::new();
        flush_to_log = items.len() > 20;
        for item in items {
            let (result, house) = process_insert(w, *item, only);
            match result {
                DesignInsertResult::Valid => {
                    run.add_response("The item has been inserted into the house design.");
                    if let Some(h) = house.filter(|h| !foundations.contains(h)) {
                        foundations.push(h);
                    }
                }
                DesignInsertResult::InvalidItem => run.log_failure("That cannot be inserted."),
                DesignInsertResult::NotInHouse | DesignInsertResult::OutsideHouseBounds => {
                    run.log_failure("That item is not inside a customizable house.")
                }
            }
        }
        for h in foundations {
            muo_server::item::ops::delta(w, h, muo_server::item::ItemDelta::UPDATE);
        }
    } else {
        run.add_response("Command aborted.");
    }
    run.flush(w, from, flush_to_log);
}

/// `DesignInsertTarget.OnTarget`, then `from.Target = new DesignInsertTarget(..)`.
fn on_target(w: &mut World, from: MobileId, obj: Obj, mut foundations: Vec<ItemId>, only: bool) {
    let item = as_item(obj);
    let (result, house) = process_insert(w, item, only);
    match result {
        DesignInsertResult::Valid => {
            if foundations.is_empty() {
                send_message(
                    w,
                    from,
                    "The item has been inserted into the house design. Press ESC when you are finished.",
                );
            } else {
                send_message(w, from, "The item has been inserted into the house design.");
            }
            if let Some(h) = house.filter(|h| !foundations.contains(h)) {
                foundations.push(h);
            }
        }
        DesignInsertResult::InvalidItem => {
            send_message(w, from, "That cannot be inserted. Try again.")
        }
        DesignInsertResult::NotInHouse | DesignInsertResult::OutsideHouseBounds => send_message(
            w,
            from,
            "That item is not inside a customizable house. Try again.",
        ),
    }
    begin_target(w, from, foundations, only);
}

fn begin_target(w: &mut World, from: MobileId, foundations: Vec<ItemId>, only: bool) {
    let id = new_target(w, foundations, only);
    target::set_mobile_target(w, from, Some(id));
}

/// `new DesignInsertTarget(foundations, staticsOnly)`.
fn new_target(w: &mut World, foundations: Vec<ItemId>, only: bool) -> target::TargetId {
    let mut t = Target::new(w, -1, false, TargetFlags::NONE);
    let f1 = foundations.clone();
    t.handlers.on_target = Some(Rc::new(move |w, from, o: Targeted| {
        on_target(w, from, o, f1.clone(), only)
    }));
    t.handlers.on_target_cancel = Some(Rc::new(move |w, from, _| {
        if !foundations.is_empty() {
            send_message(w, from, "Your changes have been committed. Updating...");
            for h in &foundations {
                muo_server::item::ops::delta(w, *h, muo_server::item::ItemDelta::UPDATE);
            }
        }
    }));
    target::add_target(w, t)
}
