//! The spawner commands: the target commands (`[Respawn`, `[ExportSpawners`, `[SpawnProps`, `[EditSpawner`,
//! `[ShowSpawnerBorders`) over `muo_commands::generic::BaseCommand`, and the `SpawnerBorder` item.

use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use std::rc::Rc;

use indexmap::IndexMap;
use muo_commands::generic::implementors::register_command;
use muo_commands::generic::{
    BaseCommand, CmdSpec, CommandSupport as S, Obj, ObjectTypes as O, Run, as_item,
};
use muo_primitives::{Point3D, Rectangle3D};
use muo_server::commands::CommandEventArgs;
use muo_server::ids::ItemId;
use muo_server::item::{ItemType, base as item_base, ops};
use muo_server::mobile::AccessLevel;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::world::World;

use super::export::{serialize_compact, to_dto};
use super::{data, data_mut};

/// `SpawnerBorder : ProjectedItem`: never saved.
pub static SPAWNER_BORDER: ItemType = ItemType {
    full_name: "Server.Engines.Spawners.SpawnerBorder",
    parent: Some(&muo_items::misc::machines::PROJECTED_ITEM),
    construct: None,
    skip_serialization: |_, _| true,
    default_name: |_, _| Some("Spawn Border".to_owned()),
    ..muo_items::misc::machines::PROJECTED_ITEM
};

// ---- [Respawn ----------------------------------------------------------------------------------------------------

struct RespawnCommand(CmdSpec);

impl BaseCommand for RespawnCommand {
    fn spec(&self) -> &CmdSpec {
        &self.0
    }
    fn execute_list(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        if list.is_empty() {
            run.log_failure("No matching objects found.");
            return;
        }
        muo_content::mx::send_message(w, e.mobile, "Respawning...");
        for o in list {
            if let Some(i) = as_item(*o)
                && data(w, i).is_some_and(|d| d.running)
            {
                super::core::respawn(w, i);
            }
        }
        muo_content::mx::send_message(w, e.mobile, "Respawn completed.");
    }
}

// ---- [ExportSpawners ---------------------------------------------------------------------------------------------

struct ExportSpawnersCommand(CmdSpec);

impl BaseCommand for ExportSpawnersCommand {
    fn spec(&self) -> &CmdSpec {
        &self.0
    }
    fn execute_list(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        let args = &e.arguments;
        let mut path = args
            .first()
            .map_or_else(String::new, |a| a.trim().to_owned());
        let condition = if args.len() == 2 {
            args[1].trim().to_owned()
        } else {
            String::new()
        };
        let base = w.core.base_directory.clone();
        let target = if path.is_empty() {
            base.join(format!("Data/Spawns/{}.json", w.now.to_timestamp()))
        } else {
            let p = std::path::PathBuf::from(&path);
            if !p.is_absolute() {
                path = base.join(&p).to_string_lossy().into_owned();
            } else if !p.parent().is_some_and(std::path::Path::exists) {
                run.log_failure("Directory doesn't exist.");
                return;
            }
            std::path::PathBuf::from(&path)
        };
        let mut records = Vec::with_capacity(list.len());
        for o in list {
            let Some(i) = as_item(*o) else { continue };
            // Not a spawner, not on a valid map, or is in a container
            let Some(item) = w.item(i) else { continue };
            if data(w, i).is_none()
                || item.map == Some(muo_server::map::MapId::INTERNAL)
                || item.parent.is_some()
            {
                continue;
            }
            let name = ops::name(w, i);
            if let Some(n) = name.as_deref().filter(|n| !n.is_empty())
                && !n.lower_inv().starts_with(&condition.lower_inv())
            {
                continue;
            }
            if let Some(dto) = to_dto(w, i) {
                records.push(dto);
            }
        }
        if records.is_empty() {
            run.log_failure("No matching spawners found.");
            return;
        }
        muo_content::mx::send_message(w, e.mobile, "Exporting spawners...");
        let text = serialize_compact(w, &records);
        if let Some(dir) = target.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match std::fs::write(&target, text) {
            Ok(()) => muo_content::mx::send_message(
                w,
                e.mobile,
                &format!("Spawners exported to {}", target.display()),
            ),
            Err(err) => tracing::error!("[ExportSpawners failed: {err}"),
        }
    }
}

// ---- [EditSpawner ------------------------------------------------------------------------------------------------

struct EditSpawnerCommand(CmdSpec);

/// `ReadOnlySpan<char>.InsensitiveIndexOf(text)`.
fn insensitive_index_of(hay: &str, needle: &str) -> Option<usize> {
    hay.lower_inv().find(&needle.lower_inv())
}

impl BaseCommand for EditSpawnerCommand {
    fn spec(&self) -> &CmdSpec {
        &self.0
    }
    fn execute_list(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        let args = &e.arguments;
        if args.len() <= 1 {
            run.log_failure(&self.0.usage);
            return;
        }
        if list.is_empty() {
            run.log_failure("No matching objects found.");
            return;
        }
        let name = args[0].clone();
        if w.types.find_type_by_name(&name, false, true).is_none() {
            run.log_failure("No type with that name was found.");
            return;
        }
        // `e.ArgString.AsSpan(name.Length + 1)`
        let Some(mut arg_span) = e.arg_string.get(name.len() + 1..) else {
            return;
        };
        let set_index = insensitive_index_of(arg_span, "set ");
        let where_ = insensitive_index_of(arg_span, "where ");
        let (mut props, mut findmatch): (Option<&str>, Option<&str>) = (None, None);
        if set_index.is_some() || where_.is_some() {
            let start = set_index.map_or(3, |s| s + 4); // `setIndex + 4` with -1 is 3
            let len = where_.unwrap_or(arg_span.len());
            findmatch = arg_span.get(
                if len < arg_span.len() {
                    len + 6
                } else {
                    arg_span.len()
                }..,
            );
            if let Some(si) = set_index {
                // `argSpan[start..^len]`: the end counts from the back, so this range is only valid for some inputs
                let end = arg_span.len().saturating_sub(len);
                let Some(p) = (start <= end).then(|| arg_span.get(start..end)).flatten() else {
                    tracing::error!(
                        "[EditSpawner: the set range {start}..{end} is outside the arguments"
                    );
                    return;
                };
                props = Some(p);
                arg_span = &arg_span[..si];
            } else {
                let end = arg_span.len().saturating_sub(len);
                arg_span = &arg_span[..end];
            }
        }
        let some = |s: &str| (!s.is_empty()).then(|| s.to_owned());
        let arg_str = some(arg_span);
        let props_str = props.and_then(some);
        let where_str = findmatch.and_then(some);
        muo_content::mx::send_message(w, e.mobile, "Updating spawners...");
        for o in list {
            if let Some(i) = as_item(*o) {
                update_spawner(
                    w,
                    i,
                    &name,
                    arg_str.as_deref(),
                    props_str.as_deref(),
                    where_str.as_deref(),
                );
            }
        }
        muo_content::mx::send_message(w, e.mobile, "Update completed.");
    }
}

/// `EditSpawnCommand.UpdateSpawner(spawner, name, arguments, properties, find)`.
pub fn update_spawner(
    w: &mut World,
    spawner: ItemId,
    name: &str,
    arguments: Option<&str>,
    properties: Option<&str>,
    find: Option<&str>,
) {
    let Some(d) = data_mut(w, spawner) else {
        return;
    };
    for entry in &mut d.entries {
        // TODO: should cache the spawn type on the entry
        if !entry.name.as_deref().is_some_and(|n| n.eq_ic(name)) {
            continue;
        }
        let should_update = match find {
            None => true,
            Some(f) => match entry
                .properties
                .as_deref()
                .and_then(|p| p.lower_inv().rfind(&f.lower_inv()).map(|at| (p, at)))
            {
                Some((p, at)) => {
                    at + f.len() == p.len()
                        || p[at + f.len()..]
                            .chars()
                            .next()
                            .is_some_and(char::is_whitespace)
                }
                None => false,
            },
        };
        if should_update {
            if let Some(a) = arguments {
                entry.parameters = Some(a.to_owned());
            }
            entry.properties = properties.map(str::to_owned);
        }
    }
}

// ---- [SpawnProps -------------------------------------------------------------------------------------------------

struct SpawnPropsCommand(CmdSpec);

impl BaseCommand for SpawnPropsCommand {
    fn spec(&self) -> &CmdSpec {
        &self.0
    }
    fn execute_list(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, list: &[Obj]) {
        if list.is_empty() {
            run.log_failure("No matching objects found.");
            return;
        }
        muo_content::mx::send_message(
            w,
            e.mobile,
            "Target the object you want to use as a template for modifying the spawner properties.",
        );
        let spawners: Vec<ItemId> = list.iter().filter_map(|o| as_item(*o)).collect();
        target::begin_target(
            w,
            e.mobile,
            -1,
            false,
            TargetFlags::NONE,
            move |w, from, t| {
                // `if (!IsEntity(type)) SendMessage(..)` does not return: the gump opens on any target
                let obj = match t {
                    Targeted::Entity(ent) => muo_gumps::m_z::props::PropObj::Entity(ent),
                    other => {
                        muo_content::mx::send_message(w, from, "No type with that name was found.");
                        muo_gumps::m_z::props::PropObj::Other(std::rc::Rc::new(other))
                    }
                };
                super::gumps::send_spawn_props(w, from, obj, spawners.clone());
            },
        );
    }
}

// ---- [ShowSpawnerBorders -----------------------------------------------------------------------------------------

const SOUTH_EAST_CORNER: i32 = 0xF8;
const NORTH_SOUTH_WALL: i32 = 0xF9;
const EAST_WEST_WALL: i32 = 0xFA;
const NORTH_WEST_CORNER: i32 = 0xFB;

/// `_projectedSpawners` and each border's `Spawner`.
#[derive(Default)]
pub struct Projections {
    pub spawners: IndexMap<ItemId, Vec<ItemId>>,
}

struct ShowSpawnerBordersCommand(CmdSpec);

fn create_border(
    w: &mut World,
    spawner: ItemId,
    item_id: i32,
    x: i32,
    y: i32,
    z: i32,
    map: muo_server::map::MapId,
) -> ItemId {
    let b = muo_items::misc::machines::projected_item_new_of(
        w,
        &SPAWNER_BORDER,
        item_id,
        AccessLevel::Developer,
    );
    item_base::set_hue(w, b, 0x3F);
    let _ = spawner;
    ops::move_to_world_map(w, b, Point3D::new(x, y, z), Some(map));
    b
}

fn create_borders(w: &mut World, spawner: ItemId, bounds: Rectangle3D) -> Vec<ItemId> {
    let Some(i) = w.item(spawner) else {
        return Vec::new();
    };
    let (Some(map), z) = (i.map, i.location.z) else {
        return Vec::new();
    };
    let (min_x, min_y) = (bounds.start.x, bounds.start.y);
    let (max_x, max_y) = (bounds.end.x, bounds.end.y); // Exclusive
    let mut b = Vec::new();
    b.push(create_border(
        w,
        spawner,
        NORTH_WEST_CORNER,
        min_x - 1,
        min_y - 1,
        z,
        map,
    ));
    b.push(create_border(
        w,
        spawner,
        SOUTH_EAST_CORNER,
        max_x - 1,
        max_y - 1,
        z,
        map,
    ));
    for x in min_x..max_x {
        b.push(create_border(
            w,
            spawner,
            NORTH_SOUTH_WALL,
            x,
            min_y - 1,
            z,
            map,
        ));
    }
    for x in min_x..max_x - 1 {
        b.push(create_border(
            w,
            spawner,
            NORTH_SOUTH_WALL,
            x,
            max_y - 1,
            z,
            map,
        ));
    }
    for y in min_y..max_y {
        b.push(create_border(
            w,
            spawner,
            EAST_WEST_WALL,
            min_x - 1,
            y,
            z,
            map,
        ));
    }
    for y in min_y..max_y - 1 {
        b.push(create_border(
            w,
            spawner,
            EAST_WEST_WALL,
            max_x - 1,
            y,
            z,
            map,
        ));
    }
    b
}

/// `RemoveProjection(spawner)`.
pub fn remove_projection(w: &mut World, spawner: ItemId) {
    let Some(borders) = w
        .resources
        .get_mut::<Projections>()
        .spawners
        .shift_remove(&spawner)
    else {
        return;
    };
    for b in borders {
        if let Some(t) = w.item(b).map(|x| x.ty) {
            (t.delete)(w, b);
        }
    }
}

impl BaseCommand for ShowSpawnerBordersCommand {
    fn spec(&self) -> &CmdSpec {
        &self.0
    }
    fn execute(&self, w: &mut World, run: &mut Run, e: &CommandEventArgs, obj: Obj) {
        let Some(spawner) = as_item(obj).filter(|i| data(w, *i).is_some()) else {
            run.log_failure("That is not a spawner.");
            return;
        };
        if w.resources
            .get::<Projections>()
            .is_some_and(|p| p.spawners.contains_key(&spawner))
        {
            remove_projection(w, spawner);
            muo_content::mx::send_message(w, e.mobile, "Spawner projection removed.");
            return;
        }
        let bounds = data(w, spawner).map_or_else(Rectangle3D::default, |d| d.spawn_bounds);
        if bounds == Rectangle3D::default() {
            run.log_failure("Spawner has no spawn bounds defined.");
            return;
        }
        let borders = create_borders(w, spawner, bounds);
        if borders.is_empty() {
            run.log_failure("Could not create borders for spawner bounds.");
            return;
        }
        let n = borders.len();
        w.resources
            .get_mut::<Projections>()
            .spawners
            .insert(spawner, borders);
        muo_content::mx::send_message(
            w,
            e.mobile,
            &format!("Spawner projection created with {n} border tiles."),
        );
    }
}

// ---- registration --------------------------------------------------------------------------------------------------

/// The four `Configure` methods of the commands (`TargetCommands.Register`).
pub fn configure(w: &mut World) {
    use AccessLevel::{Developer, GameMaster};
    let spec = |names: &[&str], level, supports, usage: &str, description: &str| {
        CmdSpec::new(names, level, supports, O::Items, usage, description).list_optimized()
    };
    register_command(
        w,
        Rc::new(RespawnCommand(spec(
            &["Respawn"],
            GameMaster,
            S::COMPLEX | S::SIMPLE,
            "Respawn",
            "Respawns the given the spawners.",
        ))),
    );
    register_command(
        w,
        Rc::new(SpawnPropsCommand(spec(
            &["SpawnProps"],
            GameMaster,
            S::COMPLEX | S::SIMPLE,
            "SpawnProps",
            "Shows a props gump that will modify the properties of spawn entries related to the chosen entity",
        ))),
    );
    register_command(
        w,
        Rc::new(ExportSpawnersCommand(spec(
            &["ExportSpawners"],
            GameMaster,
            S::ALL_ITEMS & !S::CONTAINED,
            "ExportSpawners",
            "Exports the given spawners to a file",
        ))),
    );
    register_command(
        w,
        Rc::new(EditSpawnerCommand(spec(
            &["EditSpawner"],
            GameMaster,
            S::COMPLEX | S::SIMPLE,
            "EditSpawner <type> <arguments> set <properties> where <properties>",
            "Modifies spawners arguments and properties for the given type",
        ))),
    );
    // not ListOptimized
    register_command(
        w,
        Rc::new(ShowSpawnerBordersCommand(CmdSpec::new(
            &["ShowSpawnerBorders"],
            Developer,
            S::SIMPLE,
            O::Items,
            "ShowSpawnerBorders",
            "Toggles a projection of the targeted spawner's borders.",
        ))),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_of_ignores_case() {
        assert_eq!(insensitive_index_of("Orc SET str 5", "set "), Some(4));
        assert_eq!(insensitive_index_of("Orc", "set "), None);
    }
}
