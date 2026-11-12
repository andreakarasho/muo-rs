//! `[Decorate` and `[DecorateMag` build
//! the world's static decoration from the `Data/Decoration/**/*.cfg` files.
//!
//! The two commands are one program with three differences, kept as `mag`: `DecorateMag` has no `WalkingRange`
//! spawner parameter and no `InteractionTeleporter` branch (the hook gets the flag), and a constructor exception
//! is not caught per entry but aborts the command.
//!
//! The parts that need classes of crates above this one (the per-class parameter blocks, constructors the overload
//! tables cannot call, door pairing) are the `decorate_*` fields of [`CommandHooks`], set by the crates that own the
//! classes.

use std::path::Path;

use muo_primitives::Point3D;
use muo_primitives::parse::to_i32;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::ops;
use muo_server::map::{MapId, enumerate};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message;
use muo_server::world::World;

use muo_content::props::{self, Value};

use crate::hooks::CommandHooks;

/// One `<x> <y> <z> [extra]` line (`DecorationEntry`).
pub struct DecorationEntry {
    pub location: Point3D,
    pub extra: String,
}

impl DecorationEntry {
    /// `DecorationEntry(line)`: three `Pop`ped numbers, the rest is `Extra`.
    pub fn new(line: &str) -> Self {
        let mut rest = line;
        let mut pop = || {
            let (v, r) = match rest.find(' ') {
                Some(i) => (&rest[..i], &rest[i + 1..]),
                None => (rest, ""),
            };
            rest = r;
            to_i32(v)
        };
        let (x, y, z) = (pop(), pop(), pop());
        DecorationEntry {
            location: Point3D::new(x, y, z),
            extra: rest.to_string(),
        }
    }
}

/// A block of the file: a class header and its placement lines (`DecorationList`).
pub struct DecorationList {
    /// The class full name `FindTypeByName` found.
    pub type_name: &'static str,
    pub item_id: i32,
    pub params: Vec<String>,
    pub entries: Vec<DecorationEntry>,
}

/// `Read`: the next block of `lines`, `Ok(None)` at the end. A header naming no class is an
/// `ArgumentException`.
fn read(w: &World, lines: &mut std::str::Lines) -> Result<Option<DecorationList>, String> {
    let mut line = None;
    for l in lines.by_ref() {
        let t = l.trim();
        if !t.is_empty() && !t.starts_with('#') {
            line = Some(t);
            break;
        }
    }
    let Some(line) = line else { return Ok(None) };
    let sp = line.find(' ');
    let name = sp.map_or(line, |i| &line[..i]);
    let Some(type_name) = props::class_by_name(w, name) else {
        return Err(format!("Type not found for header: '{line}'"));
    };
    // `indexOf++` makes -1 into 0: a header without a space reads the whole line as the rest.
    let span = &line[sp.map_or(0, |i| i + 1)..];
    let args_start = span.find('(');
    let params = match args_start {
        Some(a) => {
            let end = if line.ends_with(')') {
                span.len() - 1
            } else {
                span.len()
            };
            let inner = if a < end { &span[a + 1..end] } else { "" };
            if inner.is_empty() {
                Vec::new()
            } else {
                inner.split(';').map(|p| p.trim().to_string()).collect()
            }
        }
        None => Vec::new(),
    };
    let item_id = to_i32(args_start.map_or(span, |a| &span[..a]).trim());
    let mut entries = Vec::new();
    for l in lines.by_ref() {
        let t = l.trim();
        if t.is_empty() {
            break;
        }
        if t.starts_with('#') {
            continue;
        }
        entries.push(DecorationEntry::new(t));
    }
    Ok(Some(DecorationList {
        type_name,
        item_id,
        params,
        entries,
    }))
}

/// `ReadAll(path)` over the file's text.
pub fn read_all(w: &World, text: &str) -> Result<Vec<DecorationList>, String> {
    let mut lines = text.lines();
    let mut out = Vec::new();
    while let Some(v) = read(w, &mut lines)? {
        out.push(v);
    }
    Ok(out)
}

/// The text after `=` of the first parameter that starts with `key`, as `StartsWithOrdinal(key)` + `IndexOf('=')`.
fn value_of<'a>(params: &'a [String], key: &str) -> Option<&'a str> {
    params
        .iter()
        .find(|p| p.starts_with(key) && p.contains('='))
        .map(|p| &p[p.find('=').map_or(0, |i| i + 1)..])
}

fn is_class(w: &World, id: ItemId, class: &str) -> bool {
    props::is_class(w, EntityId::Item(id), class)
}

/// `new T(args)` through the class's `[Constructible]` overload table, the arguments given as text and parsed by
/// the parameter types (what `Activator.CreateInstance` does with typed values).
fn make_with_text(w: &mut World, full: &str, args: &[String]) -> Option<Result<ItemId, String>> {
    let ctors = props::ctors_of(w, full)?;
    for c in ctors {
        let required = c.params.iter().filter(|p| p.default.is_none()).count();
        if args.len() < required || args.len() > c.params.len() {
            continue;
        }
        let mut values = Vec::new();
        for (i, p) in c.params.iter().enumerate() {
            match args.get(i) {
                Some(t) => match props::parse_kind(w, &p.kind, t) {
                    Ok(v) => values.push(v),
                    Err(e) => return Some(Err(format!("{t}: {e}"))),
                },
                None => values.push(p.default.clone().unwrap_or(Value::Null)),
            }
        }
        if let EntityId::Item(id) = (c.make)(w, &values) {
            return Some(Ok(id));
        }
    }
    None
}

fn make_with_values(w: &mut World, full: &str, args: &[Value]) -> Result<ItemId, String> {
    try_args(w, full, args)
        .ok_or_else(|| format!("no constructor of {full} takes {} arguments", args.len()))
}

fn try_args(w: &mut World, full: &str, args: &[Value]) -> Option<ItemId> {
    muo_content::factory::make_item_args(w, full, args)
}

/// `Construct()`. `Err` is an exception thrown by the constructor part (the `try` block: the caller reports it as
/// a `TypeInitializationException`); the parameters applied after it are `Err(Fatal)` below.
pub enum ConstructError {
    /// A construction failure: caught per entry by `Decorate`, fatal for `DecorateMag`.
    Type(String),
    /// After it (`Enum.Parse` of `Light`): not caught.
    Fatal(String),
}

impl DecorationList {
    pub fn construct(&self, w: &mut World, mag: bool) -> Result<Option<ItemId>, ConstructError> {
        let full = self.type_name;
        let ty = ConstructError::Type;
        let ps = &self.params;
        let hooks_construct: Vec<_> = w
            .resources
            .get::<CommandHooks>()
            .map(|h| h.decorate_construct.clone())
            .unwrap_or_default();
        let via_hook = |w: &mut World| {
            hooks_construct
                .iter()
                .find_map(|f| f(w, full, self.item_id, ps))
        };
        let label = |ps: &[String]| value_of(ps, "LabelNumber").map_or(0, to_i32);
        let chain = props::find_class(w, full)
            .map(|c| c.chain)
            .unwrap_or_default();
        let sub = |c: &str| chain.contains(&c) && full != c;
        let made: Result<ItemId, String> = if full == "Server.Items.Static" {
            make_with_values(w, full, &[Value::Int(i64::from(self.item_id))])
        } else if full == "Server.Items.LocalizedStatic" {
            make_with_values(
                w,
                full,
                &[
                    Value::Int(i64::from(self.item_id)),
                    Value::Int(i64::from(label(ps))),
                ],
            )
        } else if full == "Server.Items.LocalizedSign" {
            match try_args(
                w,
                full,
                &[
                    Value::Int(i64::from(self.item_id)),
                    Value::Int(i64::from(label(ps))),
                ],
            ) {
                Some(i) => Ok(i),
                None => via_hook(w).unwrap_or_else(|| Err("no constructor".into())),
            }
        } else if full == "Server.Items.AnkhWest" || full == "Server.Items.AnkhNorth" {
            let bloodied = ps.iter().any(|p| p == "Bloodied");
            make_with_values(w, full, &[Value::Bool(bloodied)])
        } else if [
            "Server.Items.MarkContainer",
            "Server.Items.HintItem",
            "Server.Items.WarningItem",
            "Server.Items.Cannon",
            "Server.Engines.Quests.Haven.Cannon",
            "Server.Items.SerpentPillar",
        ]
        .contains(&full)
        {
            via_hook(w).unwrap_or_else(|| Err(format!("{full} has no Decorate constructor")))
        } else if sub("Server.Items.BaseBeverage") {
            let fill = value_of(ps, "Content");
            let args: Vec<String> = fill.map(|c| vec![c.to_string()]).unwrap_or_default();
            match make_with_text(w, full, &args) {
                Some(r) => r,
                None => match via_hook(w) {
                    Some(r) => r,
                    None if args.is_empty() => muo_content::factory::make_item(w, full)
                        .ok_or_else(|| "no constructor".to_string()),
                    None => Err(format!("{full} has no constructor taking BeverageType")),
                },
            }
        } else if sub("Server.Items.BaseDoor") && !ps.is_empty() {
            // `Facing` defaults to `DoorFacing.WestCW`.
            let facing = value_of(ps, "Facing").unwrap_or("WestCW").to_string();
            make_with_text(w, full, &[facing])
                .unwrap_or_else(|| Err(format!("{full} has no door constructor")))
        } else {
            // `CreateInstance`: a public parameterless constructor needs no `[Constructible]`; the hooks know those.
            match muo_content::factory::make_item(w, full) {
                Some(i) => Ok(i),
                None => via_hook(w).unwrap_or_else(|| Err(format!("{full} cannot be constructed"))),
            }
        };
        let item = made.map_err(ty)?;
        let _ = mag;

        // The `item is BaseAddon / BaseLight / ...` chain.
        let appliers: Vec<_> = w
            .resources
            .get::<CommandHooks>()
            .map(|h| h.decorate_apply.clone())
            .unwrap_or_default();
        let mut handled = false;
        for f in &appliers {
            if f(w, item, self.item_id, ps, mag).map_err(ConstructError::Type)? {
                handled = true;
                break;
            }
        }
        if !handled && self.item_id > 0 {
            ops_set_item_id(w, item, self.item_id);
        }

        ops::set_movable(w, item, false);
        for p in ps {
            let Some(eq) = p.find('=') else { continue };
            let text = &p[eq + 1..];
            if p.starts_with("Light") {
                set_prop(w, item, "Light", text).map_err(ConstructError::Fatal)?;
            } else if p.starts_with("Hue") {
                let hue = to_i32(text);
                if is_class(w, item, "Server.Items.DyeTub") {
                    set_prop(w, item, "DyedHue", &hue.to_string())
                        .map_err(ConstructError::Fatal)?;
                } else {
                    set_prop(w, item, "Hue", &hue.to_string()).map_err(ConstructError::Fatal)?;
                }
            } else if p.starts_with("Name") {
                ops::set_name(w, item, Some(text.to_string()));
            } else if p.starts_with("Amount") {
                // Must suppress stackable warnings
                let was = w.item(item).is_some_and(|i| i.stackable());
                ops::set_stackable(w, item, true);
                ops::set_amount(w, item, to_i32(text));
                ops::set_stackable(w, item, was);
            }
        }
        Ok(Some(item))
    }

    /// `Generate(maps)`: places the block on each map, skipping places that already hold the item. `log` gets the
    /// `Console.WriteLine` lines.
    pub fn generate(
        &self,
        w: &mut World,
        maps: &[MapId],
        mag: bool,
        log: &mut Vec<String>,
    ) -> Result<usize, String> {
        let mut count = 0;
        let mut item: Option<ItemId> = None;
        for entry in &self.entries {
            let loc = entry.location;
            for &map in maps {
                if item.is_none() {
                    match self.construct(w, mag) {
                        Ok(i) => item = i,
                        Err(ConstructError::Type(e)) if !mag => {
                            log.push(format!(
                                "Generate() failed to load type: {}: {e}",
                                self.type_name
                            ));
                            continue;
                        }
                        Err(ConstructError::Type(e)) => {
                            return Err(format!("Bad type: {}: {e}", self.type_name));
                        }
                        Err(ConstructError::Fatal(e)) => return Err(e),
                    }
                }
                let Some(it) = item else { continue };
                if find_item(w, loc, map, it) {
                    continue;
                }
                ops::move_to_world_map(w, it, loc, Some(map));
                count += 1;
                if let Some(f) = w
                    .resources
                    .get::<CommandHooks>()
                    .and_then(|h| h.decorate_placed)
                {
                    f(w, it, loc, &entry.extra);
                }
                item = None;
            }
        }
        if let Some(it) = item {
            muo_server::item::base::delete(w, it);
        }
        Ok(count)
    }
}

fn ops_set_item_id(w: &mut World, id: ItemId, value: i32) {
    muo_server::item::ops::set_item_id(w, id, value);
}

/// Sets a `[CommandProperty]` by name from text (`Enum.Parse`/`Utility.To*` over the property's type).
fn set_prop(w: &mut World, id: ItemId, name: &str, text: &str) -> Result<(), String> {
    let e = EntityId::Item(id);
    let def = props::find_prop(w, e, name)
        .ok_or_else(|| format!("{name} is not a property of the item"))?;
    let v = props::parse_for(w, def, text).map_err(|e| format!("{name} = {text}: {e}"))?;
    (def.set.ok_or_else(|| format!("{name} is read only"))?)(w, e, v)
}

/// `FindItem(x, y, z, map, srcItem)`: whether the placement is already there; look-alikes that differ are deleted.
fn find_item(w: &mut World, loc: Point3D, map: MapId, src: ItemId) -> bool {
    let (x, y, z) = (loc.x, loc.y, loc.z);
    let Some(s) = w.item(src) else { return false };
    let (item_id, lt, src_ty) = (s.item_id, s.light(), s.ty.full_name);
    let src_name = ops::item_data(w, src).name.clone();
    let src_is_door = is_class(w, src, "Server.Items.BaseDoor");
    let src_light_tile = ops::item_data(w, src).light_source();
    let src_special = [
        "Server.Items.Teleporter",
        "Server.Items.FillableContainer",
        "Server.Items.BaseBook",
    ]
    .iter()
    .any(|c| is_class(w, src, c));
    let door_probe = w
        .resources
        .get::<CommandHooks>()
        .and_then(|h| h.decorate_door_probe);

    let mut res = false;
    let mut queue = Vec::new();
    let at: Vec<ItemId> = enumerate::items_at(w, Some(map), x, y).collect();
    for item in at {
        let Some(it) = w.item(item) else { continue };
        let (it_z, it_id, it_light, it_ty) =
            (it.location.z, it.item_id, it.light(), it.ty.full_name);
        if src_is_door {
            let Some((p, bd_item_id)) = door_probe.and_then(|f| f(w, item)) else {
                continue;
            };
            if p.x != x || p.y != y {
                continue;
            }
            if it_z == z && bd_item_id == item_id {
                res = true;
            } else if (it_z - z).abs() < 8 {
                queue.push(item);
            }
        } else if src_light_tile {
            if it_z == z {
                if it_id == item_id {
                    if it_light != lt {
                        queue.push(item);
                    } else {
                        res = true;
                    }
                } else {
                    let d = ops::item_data(w, item);
                    if d.light_source() && d.name == src_name {
                        queue.push(item);
                    }
                }
            }
        } else if src_special {
            if it_z == z && it_id == item_id {
                if it_ty != src_ty {
                    queue.push(item);
                } else {
                    res = true;
                }
            }
        } else if it_z == z && it_id == item_id {
            return true;
        }
    }
    for q in queue {
        muo_server::item::base::delete(w, q);
    }
    res
}

/// The `*.cfg` files of a folder, in the order the file system lists them.
fn cfg_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut v: Vec<_> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("cfg")))
                .collect()
        })
        .unwrap_or_default();
    // `Directory.GetFiles` on NTFS: the directory index order, which compares upper-cased names ('_' sorts after letters).
    v.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_uppercase()));
    v
}

/// `Generate(folder, maps)`: the count of items placed.
pub fn generate_folder(
    w: &mut World,
    folder: &Path,
    maps: &[MapId],
    mag: bool,
    log: &mut Vec<String>,
) -> Result<usize, String> {
    if !folder.is_dir() {
        return Ok(0);
    }
    let mut count = 0;
    for file in cfg_files(folder) {
        let text =
            std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        for list in read_all(w, &text).map_err(|e| format!("{}: {e}", file.display()))? {
            count += list.generate(w, maps, mag, log)?;
        }
    }
    Ok(count)
}

fn map_named(w: &World, name: &str) -> Vec<MapId> {
    w.maps.parse(name).into_iter().collect()
}

/// The folders each command reads, with the maps they fill.
pub fn folders(w: &World, mag: bool) -> Vec<(&'static str, Vec<MapId>)> {
    let m = |n: &str| map_named(w, n);
    if mag {
        return vec![
            ("Data/Decoration/RuinedMaginciaTram", m("Trammel")),
            ("Data/Decoration/RuinedMaginciaFel", m("Felucca")),
        ];
    }
    let mut v = vec![
        (
            "Data/Decoration/Britannia",
            [m("Trammel"), m("Felucca")].concat(),
        ),
        ("Data/Decoration/Trammel", m("Trammel")),
        ("Data/Decoration/Felucca", m("Felucca")),
        ("Data/Decoration/Ilshenar", m("Ilshenar")),
        ("Data/Decoration/Malas", m("Malas")),
        ("Data/Decoration/Tokuno", m("Tokuno")),
    ];
    if w.resources
        .get::<CommandHooks>()
        .and_then(|h| h.bounties_enabled)
        .is_some_and(|f| f(w))
    {
        v.push(("Data/Decoration/BountyBoards", m("Felucca")));
    }
    v
}

fn run(w: &mut World, e: &mut CommandEventArgs, mag: bool) {
    let from = e.mobile;
    send_message(w, from, "Generating world decoration, please wait.");
    // NetState.FlushAll(): nothing is queued behind this handler, the replies go out with the tick.
    let mut count = 0;
    let mut log = Vec::new();
    let base = w.core.base_directory.clone();
    for (folder, maps) in folders(w, mag) {
        match generate_folder(w, &base.join(folder), &maps, mag, &mut log) {
            Ok(n) => count += n,
            Err(err) => {
                tracing::error!("{}: {err}", if mag { "DecorateMag" } else { "Decorate" });
                send_message(w, from, &err);
                return;
            }
        }
    }
    for l in log {
        tracing::warn!("{l}");
    }
    send_message(
        w,
        from,
        &format!("World generating complete. {count} items were generated."),
    );
}

pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "Decorate",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(|w: &mut World, e: &mut CommandEventArgs| run(w, e, false)),
    );
    CommandSystem::register(
        w,
        "DecorateMag",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(|w: &mut World, e: &mut CommandEventArgs| run(w, e, true)),
    );
}
