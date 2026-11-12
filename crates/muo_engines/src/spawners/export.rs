//! `ToDto()` of the three spawner classes (`BaseSpawner`, `Spawner`, `RegionSpawner`, `ProximitySpawner`), the `WhenWritingDefault` JSON shape of `SpawnerDto`, and the compact writer of
//! `SpawnerJsonSerializer.SerializeCompact` (`[ExportSpawners`); plus the RunUO `.xml` importer of
//! `[ImportSpawners`.

use indexmap::IndexMap;
use muo_io::serialization::Guid;
use muo_primitives::{Point3D, Rectangle3D, TimeSpan};
use muo_server::ids::ItemId;
use muo_server::item::ops;
use muo_server::world::World;

use super::json::{SpawnerDto, to_spawner};
use super::{Entry, Kind, SpawnPositionMode, core, data};

// ---- ToDto -------------------------------------------------------------------------------------------------------

/// `BaseSpawner.ToDto()`: None when the item is not a spawner.
pub fn to_dto(w: &World, id: ItemId) -> Option<SpawnerDto> {
    let d = data(w, id)?;
    let i = w.item(id)?;
    let default_name = if d.kind == Kind::Proximity {
        "Proximity Spawner"
    } else {
        "Spawner"
    };
    let name = ops::name(w, id).filter(|n| !n.is_empty() && n != default_name);
    let location = i.location;
    let home_range = {
        // `DtoHomeRange`: the radius when SpawnBounds is exactly what it reconstructs, else -1
        let b = d.spawn_bounds;
        if b == Rectangle3D::default() || !core::is_home_range_style_at(w, id, location) {
            -1
        } else {
            let hr = core::home_range(w, id);
            if b == core::bounds_from_home_range(location, hr) {
                hr
            } else {
                -1
            }
        }
    };
    let mode = if d.spawn_position_mode == SpawnPositionMode::Abandoned {
        SpawnPositionMode::Automatic
    } else {
        d.spawn_position_mode
    };
    let uses_bounds = d.kind != Kind::Region;
    Some(SpawnerDto {
        kind: d.kind,
        guid: d.guid,
        name,
        location,
        map: i.map,
        count: d.count,
        min_delay: d.min_delay,
        max_delay: d.max_delay,
        team: d.team,
        walking_range: d.walking_range,
        spawn_location_is_home: d.spawn_location_is_home,
        group: d.group,
        spawn_position_mode: mode,
        max_spawn_attempts: if d.max_spawn_attempts == super::DEFAULT_MAX_SPAWN_ATTEMPTS {
            0
        } else {
            d.max_spawn_attempts
        },
        home_range: if uses_bounds { home_range } else { -1 },
        entries: d.entries.clone(),
        spawn_bounds: if uses_bounds && home_range < 0 {
            d.spawn_bounds
        } else {
            Rectangle3D::default()
        },
        region: d.spawn_region_name.clone(),
        trigger_range: d.trigger_range,
        spawn_message: d.spawn_message.clone(),
        instant: d.instant_flag,
    })
}

// ---- the JSON shape ----------------------------------------------------------------------------------------------

/// A JSON node that keeps the property order (`JsonPropertyOrder`).
#[derive(Clone, Debug)]
pub enum Node {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Arr(Vec<Node>),
    Obj(Vec<(String, Node)>),
}

fn entry_node(e: &Entry) -> Node {
    let mut o: Vec<(String, Node)> = Vec::new();
    if let Some(n) = &e.name {
        o.push(("name".into(), Node::Str(n.clone())));
    }
    if e.probability != 0 {
        o.push(("probability".into(), Node::Int(i64::from(e.probability))));
    }
    if e.max_count != 0 {
        o.push(("maxCount".into(), Node::Int(i64::from(e.max_count))));
    }
    if let Some(p) = &e.properties {
        o.push(("properties".into(), Node::Str(p.clone())));
    }
    if let Some(p) = &e.parameters {
        o.push(("parameters".into(), Node::Str(p.clone())));
    }
    if e.disabled {
        o.push(("disabled".into(), Node::Bool(true)));
    }
    Node::Obj(o)
}

fn rect_node(r: Rectangle3D) -> Node {
    // `Rectangle3DConverter`: x1 y1 [z1] x2 y2 [z2], z omitted for exactly the full range
    let write_z = r.start.z != -128 || r.end.z != 127;
    let mut o = vec![
        ("x1".to_owned(), Node::Int(i64::from(r.start.x))),
        ("y1".to_owned(), Node::Int(i64::from(r.start.y))),
    ];
    if write_z {
        o.push(("z1".into(), Node::Int(i64::from(r.start.z))));
    }
    o.push(("x2".into(), Node::Int(i64::from(r.end.x))));
    o.push(("y2".into(), Node::Int(i64::from(r.end.y))));
    if write_z {
        o.push(("z2".into(), Node::Int(i64::from(r.end.z))));
    }
    Node::Obj(o)
}

/// The DTO as `SpawnerJsonSerializer.Options` writes it (`WhenWritingDefault`, property order from
/// `JsonPropertyOrder`, `$type` first).
pub fn dto_node(w: &World, dto: &SpawnerDto) -> Node {
    let mut o: Vec<(String, Node)> = Vec::new();
    let type_name = match dto.kind {
        Kind::Spawner => "Spawner",
        Kind::Region => "RegionSpawner",
        Kind::Proximity => "ProximitySpawner",
    };
    o.push(("$type".into(), Node::Str(type_name.into())));
    o.push(("guid".into(), Node::Str(dto.guid.to_string())));
    if let Some(n) = &dto.name {
        o.push(("name".into(), Node::Str(n.clone())));
    }
    let l = dto.location;
    o.push((
        "location".into(),
        Node::Arr(vec![
            Node::Int(i64::from(l.x)),
            Node::Int(i64::from(l.y)),
            Node::Int(i64::from(l.z)),
        ]),
    ));
    let map = dto
        .map
        .and_then(|m| w.maps.get(m))
        .map(|m| m.name().to_owned());
    o.push(("map".into(), map.map_or(Node::Null, Node::Str)));
    o.push(("count".into(), Node::Int(i64::from(dto.count))));
    o.push(("minDelay".into(), Node::Str(dto.min_delay.to_string())));
    o.push(("maxDelay".into(), Node::Str(dto.max_delay.to_string())));
    if dto.team != 0 {
        o.push(("team".into(), Node::Int(i64::from(dto.team))));
    }
    // order 8: homeRange (when >= 0), spawnBounds (non-default) or region
    if dto.home_range >= 0 {
        o.push(("homeRange".into(), Node::Int(i64::from(dto.home_range))));
    }
    match dto.kind {
        Kind::Spawner | Kind::Proximity => {
            if dto.spawn_bounds != Rectangle3D::default() {
                o.push(("spawnBounds".into(), rect_node(dto.spawn_bounds)));
            }
        }
        Kind::Region => {
            if let Some(r) = &dto.region {
                o.push(("region".into(), Node::Str(r.clone())));
            }
        }
    }
    if dto.walking_range != 0 {
        o.push((
            "walkingRange".into(),
            Node::Int(i64::from(dto.walking_range)),
        ));
    }
    o.push((
        "entries".into(),
        Node::Arr(dto.entries.iter().map(entry_node).collect()),
    ));
    if dto.spawn_location_is_home {
        o.push(("spawnLocationIsHome".into(), Node::Bool(true)));
    }
    if dto.group {
        o.push(("group".into(), Node::Bool(true)));
    }
    if dto.spawn_position_mode != SpawnPositionMode::Automatic {
        let n = match dto.spawn_position_mode {
            SpawnPositionMode::Enabled => "Enabled",
            SpawnPositionMode::Disabled => "Disabled",
            SpawnPositionMode::Abandoned => "Abandoned",
            SpawnPositionMode::Automatic => "Automatic",
        };
        o.push(("spawnPositionMode".into(), Node::Str(n.into())));
    }
    if dto.max_spawn_attempts != 0 {
        o.push((
            "maxSpawnAttempts".into(),
            Node::Int(i64::from(dto.max_spawn_attempts)),
        ));
    }
    if dto.kind == Kind::Proximity {
        if dto.trigger_range != 0 {
            o.push((
                "triggerRange".into(),
                Node::Int(i64::from(dto.trigger_range)),
            ));
        }
        if let Some(m) = &dto.spawn_message {
            o.push(("spawnMessage".into(), Node::Str(m.to_string())));
        }
        if dto.instant {
            o.push(("instant".into(), Node::Bool(true)));
        }
    }
    Node::Obj(o)
}

// ---- the compact writer ------------------------------------------------------------------------------------------------

const COMPACT_PRINT_WIDTH: usize = 125;

/// `JsonNode.ToJsonString` with `UnsafeRelaxedJsonEscaping` for a scalar.
fn scalar(n: &Node) -> String {
    match n {
        Node::Null => "null".into(),
        Node::Bool(b) => b.to_string(),
        Node::Int(i) => i.to_string(),
        Node::Str(s) => {
            let mut out = String::with_capacity(s.len() + 2);
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    '\u{8}' => out.push_str("\\b"),
                    '\u{c}' => out.push_str("\\f"),
                    c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        Node::Arr(_) | Node::Obj(_) => String::new(),
    }
}

fn all_scalar(n: &Node) -> bool {
    match n {
        Node::Arr(a) => a.iter().all(|x| !matches!(x, Node::Arr(_) | Node::Obj(_))),
        Node::Obj(o) => o
            .iter()
            .all(|(_, x)| !matches!(x, Node::Arr(_) | Node::Obj(_))),
        _ => true,
    }
}

fn inline(n: &Node, out: &mut String) {
    match n {
        Node::Arr(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                inline(x, out);
            }
            out.push(']');
        }
        Node::Obj(o) if o.is_empty() => out.push_str("{}"),
        Node::Obj(o) => {
            out.push_str("{ ");
            for (i, (k, v)) in o.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push('"');
                out.push_str(k);
                out.push_str("\": ");
                inline(v, out);
            }
            out.push_str(" }");
        }
        s => out.push_str(&scalar(s)),
    }
}

fn write_node(n: &Node, out: &mut String, indent: usize, column: usize) {
    if !matches!(n, Node::Arr(_) | Node::Obj(_)) {
        out.push_str(&scalar(n));
        return;
    }
    if all_scalar(n) {
        let mut line = String::new();
        inline(n, &mut line);
        if column + line.chars().count() <= COMPACT_PRINT_WIDTH {
            out.push_str(&line);
            return;
        }
    }
    let child_indent = indent + 1;
    let child_column = child_indent * 2;
    match n {
        Node::Arr(a) => {
            out.push_str("[\n");
            for (i, x) in a.iter().enumerate() {
                out.push_str(&" ".repeat(child_column));
                write_node(x, out, child_indent, child_column);
                out.push_str(if i + 1 < a.len() { ",\n" } else { "\n" });
            }
            out.push_str(&" ".repeat(indent * 2));
            out.push(']');
        }
        Node::Obj(o) => {
            out.push_str("{\n");
            for (i, (k, v)) in o.iter().enumerate() {
                out.push_str(&" ".repeat(child_column));
                let prefix = format!("\"{k}\": ");
                out.push_str(&prefix);
                write_node(v, out, child_indent, child_column + prefix.chars().count());
                out.push_str(if i + 1 < o.len() { ",\n" } else { "\n" });
            }
            out.push_str(&" ".repeat(indent * 2));
            out.push('}');
        }
        _ => {}
    }
}

/// `SpawnerJsonSerializer.SerializeCompact(list)`: UTF-8, LF.
pub fn serialize_compact(w: &World, dtos: &[SpawnerDto]) -> String {
    let node = Node::Arr(dtos.iter().map(|d| dto_node(w, d)).collect());
    let mut out = String::new();
    write_node(&node, &mut out, 0, 0);
    out.push('\n');
    out
}

// ---- the RunUO xml importer ------------------------------------------------------------------------------------------

/// The text of `<tag>..</tag>` inside `src` (first match, inner text unescaped for the five XML entities).
fn tag<'a>(src: &'a str, name: &str) -> Option<&'a str> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let s = src.find(&open)? + open.len();
    let e = src[s..].find(&close)? + s;
    Some(&src[s..e])
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// `Utility.GetText(node["x"], default)`.
fn text(src: &str, name: &str, default: &str) -> String {
    tag(src, name).map_or_else(|| default.to_owned(), |t| unescape(t.trim()))
}

/// The inner text of every `<tag>` in `src`.
fn all_tags<'a>(src: &'a str, name: &str) -> Vec<&'a str> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(s) = rest.find(&open) {
        let from = s + open.len();
        let Some(e) = rest[from..].find(&close) else {
            break;
        };
        out.push(&rest[from..from + e]);
        rest = &rest[from + e + close.len()..];
    }
    out
}

/// `ImportXmlSpawners`: `(generated, failures)`. A spawner that fails to parse is reported and skipped.
pub fn import_xml(
    w: &mut World,
    from: Option<muo_server::ids::MobileId>,
    file: &std::path::Path,
    all: &mut IndexMap<Guid, ItemId>,
) -> (i32, i32) {
    let (mut generated, mut failures) = (0, 0);
    let say = |w: &mut World, t: String| {
        if let Some(f) = from {
            muo_content::mx::send_message(w, f, &t);
        }
    };
    let Ok(doc) = std::fs::read_to_string(file) else {
        say(
            w,
            format!(
                "GenerateSpawners: Exception parsing {}, file may not be in the correct format.",
                file.display()
            ),
        );
        return (0, 0);
    };
    let Some(root) = tag(&doc, "spawners") else {
        return (0, 0);
    };
    for (index, node) in all_tags(root, "spawner").into_iter().enumerate() {
        let result = (|| -> Result<(), String> {
            let count: i32 = text(node, "count", "1").parse().map_err(|_| "count")?;
            let home_range: i32 = text(node, "homerange", "4")
                .parse()
                .map_err(|_| "homerange")?;
            let walking_range: i32 = text(node, "walkingrange", "-1")
                .parse()
                .map_err(|_| "walkingrange")?;
            let team: i32 = text(node, "team", "0").parse().map_err(|_| "team")?;
            let max_delay =
                TimeSpan::try_parse(&text(node, "maxdelay", "10:00")).ok_or("maxdelay")?;
            let min_delay =
                TimeSpan::try_parse(&text(node, "mindelay", "05:00")).ok_or("mindelay")?;
            let names: Vec<String> = tag(node, "creaturesname")
                .map(|c| {
                    all_tags(c, "creaturename")
                        .into_iter()
                        .map(|n| unescape(n.trim()))
                        .collect()
                })
                .unwrap_or_default();
            let name = text(node, "name", "Spawner");
            let location =
                Point3D::try_parse(&text(node, "location", "Error")).ok_or("location")?;
            let map = w.maps.parse(&text(node, "map", "Error")).ok_or("map")?;
            let refs: Vec<&str> = names.iter().map(String::as_str).collect();
            let id = core::create(
                w,
                &super::core::SPAWNER,
                Kind::Spawner,
                count,
                min_delay,
                max_delay,
                team,
                Rectangle3D::default(),
                &refs,
            );
            if walking_range >= 0 {
                core::set_walking_range(w, id, walking_range);
            }
            ops::set_name(w, id, Some(name));
            ops::move_to_world_map(w, id, location, Some(map));
            core::set_home_range(w, id, home_range);
            if map == muo_server::map::MapId::INTERNAL {
                if let Some(t) = w.item(id).map(|i| i.ty) {
                    (t.delete)(w, id);
                }
                return Err("Spawner created on Internal map.".into());
            }
            core::respawn(w, id);
            let guid = data(w, id).map_or_else(Guid::default, |d| d.guid);
            all.insert(guid, id);
            Ok(())
        })();
        match result {
            Ok(()) => generated += 1,
            Err(e) => {
                // only the internal-map case counts a failure (`totalFailures++` before the error)
                if e == "Spawner created on Internal map." {
                    failures += 1;
                }
                tracing::error!(
                    "Failed to generate spawner {index} in {}: {e}",
                    file.display()
                );
                say(
                    w,
                    format!(
                        "GenerateSpawners: Exception parsing {}, file may not be in the correct format.",
                        file.display()
                    ),
                );
            }
        }
    }
    (generated, failures)
}

/// Round trip helper for tests: builds the spawner of a DTO (`to_spawner`).
pub fn build(w: &mut World, dto: &SpawnerDto) -> ItemId {
    to_spawner(w, dto)
}
