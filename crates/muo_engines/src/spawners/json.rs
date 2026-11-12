//! `[ImportSpawners` (JSON and Nerun Premium `.map` files; the RunUO `.xml` form is in `export`), `SpawnerDto`
//! and the DTO read side of the spawner JSON serializer.
//!
//! A spawner file is an array of objects with a `$type` discriminator (`Spawner`, `RegionSpawner`,
//! `ProximitySpawner`). Reading builds plain [`SpawnerDto`] values first and only then world objects, so a
//! malformed file fails as a whole before anything is created.
//!
//! `[ExportSpawners` and the `.xml` importer are in `export`/`commands`.

use muo_primitives::dotnet::IgnoreCase;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use indexmap::IndexMap;
use muo_content::mx;
use muo_io::TextDefinition;
use muo_io::serialization::Guid;
use muo_primitives::{Point3D, Rectangle3D, TimeSpan};
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::world::World;
use serde_json::Value;

use super::core::{self, PROXIMITY_SPAWNER, REGION_SPAWNER, SPAWNER};
use super::save::parse_guid;
use super::{Entry, Kind, SpawnPositionMode, data, data_mut};

/// `SpawnerDto` and its three concrete records.
#[derive(Debug)]
pub struct SpawnerDto {
    pub kind: Kind,
    pub guid: Guid,
    pub name: Option<String>,
    pub location: Point3D,
    pub map: Option<MapId>,
    pub count: i32,
    pub min_delay: TimeSpan,
    pub max_delay: TimeSpan,
    pub team: i32,
    pub walking_range: i32,
    pub spawn_location_is_home: bool,
    pub group: bool,
    pub spawn_position_mode: SpawnPositionMode,
    pub max_spawn_attempts: i32,
    pub home_range: i32,
    pub entries: Vec<Entry>,
    pub spawn_bounds: Rectangle3D,
    pub region: Option<String>,
    pub trigger_range: i32,
    pub spawn_message: Option<TextDefinition>,
    pub instant: bool,
}

type Obj = serde_json::Map<String, Value>;

/// Property names match case-insensitively (`PropertyNameCaseInsensitive`).
fn get<'a>(o: &'a Obj, key: &str) -> Option<&'a Value> {
    o.get(key)
        .or_else(|| o.iter().find(|(k, _)| k.eq_ic(key)).map(|(_, v)| v))
}

fn int(o: &Obj, key: &str, default: i32) -> Result<i32, String> {
    match get(o, key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Number(n)) => n
            .as_i64()
            .and_then(|v| i32::try_from(v).ok())
            .ok_or_else(|| format!("{key}: not an int")),
        Some(_) => Err(format!("{key}: not a number")),
    }
}

fn boolean(o: &Obj, key: &str) -> Result<bool, String> {
    match get(o, key) {
        None | Some(Value::Null) => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(format!("{key}: not a bool")),
    }
}

fn string(o: &Obj, key: &str) -> Result<Option<String>, String> {
    match get(o, key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(format!("{key}: not a string")),
    }
}

fn time_span(o: &Obj, key: &str) -> Result<TimeSpan, String> {
    match get(o, key) {
        None | Some(Value::Null) => Ok(TimeSpan::ZERO),
        Some(Value::String(s)) => {
            TimeSpan::try_parse(s).ok_or_else(|| format!("{key}: bad TimeSpan '{s}'"))
        }
        Some(_) => Err(format!("{key}: not a TimeSpan")),
    }
}

fn point3d(v: &Value) -> Result<Point3D, String> {
    let n = |x: &Value| {
        x.as_i64()
            .and_then(|v| i32::try_from(v).ok())
            .ok_or_else(|| "Point3D: bad number".to_owned())
    };
    match v {
        Value::Array(a) if a.len() <= 3 => {
            let mut c = [0; 3];
            for (i, x) in a.iter().enumerate() {
                c[i] = n(x)?;
            }
            Ok(Point3D::new(c[0], c[1], c[2]))
        }
        Value::Object(o) => {
            let g = |k: &str| get(o, k).map_or(Ok(0), n);
            Ok(Point3D::new(g("x")?, g("y")?, g("z")?))
        }
        Value::String(s) => Point3D::try_parse(s).ok_or_else(|| "Point3D: bad text".to_owned()),
        _ => Err("Point3D must be an array of x, y, z".to_owned()),
    }
}

/// `Rectangle3DConverter` (strings, arrays, `x1..z2`, `x/y/z/w/h/d` and `start`/`end` objects).
fn rectangle3d(v: &Value) -> Result<Rectangle3D, String> {
    muo_data::json::converters::rectangle3d::deserialize(v.clone()).map_err(|e| e.to_string())
}

fn entry(v: &Value) -> Result<Entry, String> {
    let Value::Object(o) = v else {
        return Err("entry is not an object".to_owned());
    };
    Ok(Entry {
        name: string(o, "name")?,
        probability: int(o, "probability", 100)?,
        max_count: int(o, "maxCount", 1)?,
        properties: string(o, "properties")?,
        parameters: string(o, "parameters")?,
        disabled: boolean(o, "disabled")?,
        ..Entry::default()
    })
}

/// Reads one DTO; `Err` is a `JsonException`.
pub fn read_dto(w: &World, v: &Value) -> Result<SpawnerDto, String> {
    let Value::Object(o) = v else {
        return Err("spawner is not an object".to_owned());
    };
    let kind = match get(o, "$type") {
        Some(Value::String(t)) if t.eq_ic("Spawner") => Kind::Spawner,
        Some(Value::String(t)) if t.eq_ic("RegionSpawner") => Kind::Region,
        Some(Value::String(t)) if t.eq_ic("ProximitySpawner") => Kind::Proximity,
        other => return Err(format!("unknown spawner type {other:?}")),
    };
    let map = match get(o, "map") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => w.maps.parse(s),
        Some(Value::Number(n)) => n.as_u64().and_then(|i| u8::try_from(i).ok()).map(MapId),
        Some(_) => return Err("map: bad value".to_owned()),
    };
    let mode = match get(o, "spawnPositionMode") {
        None | Some(Value::Null) => SpawnPositionMode::Automatic,
        Some(Value::Number(n)) => SpawnPositionMode::from_u8(n.as_u64().unwrap_or(0) as u8),
        Some(Value::String(s)) => match s.to_ascii_lowercase().as_str() {
            "automatic" => SpawnPositionMode::Automatic,
            "enabled" => SpawnPositionMode::Enabled,
            "disabled" => SpawnPositionMode::Disabled,
            "abandoned" => SpawnPositionMode::Abandoned,
            _ => return Err(format!("spawnPositionMode: unknown '{s}'")),
        },
        Some(_) => return Err("spawnPositionMode: bad value".to_owned()),
    };
    let mut entries = Vec::new();
    match get(o, "entries") {
        None | Some(Value::Null) => {}
        Some(Value::Array(a)) => {
            for e in a {
                entries.push(entry(e)?);
            }
        }
        Some(_) => return Err("entries: not an array".to_owned()),
    }
    Ok(SpawnerDto {
        kind,
        guid: match get(o, "guid") {
            None | Some(Value::Null) => Guid::default(),
            Some(Value::String(s)) => {
                parse_guid(s).ok_or_else(|| format!("guid: bad value '{s}'"))?
            }
            Some(_) => return Err("guid: bad value".to_owned()),
        },
        name: string(o, "name")?,
        location: get(o, "location")
            .map(point3d)
            .transpose()?
            .unwrap_or_default(),
        map,
        count: int(o, "count", 0)?,
        min_delay: time_span(o, "minDelay")?,
        max_delay: time_span(o, "maxDelay")?,
        team: int(o, "team", 0)?,
        walking_range: int(o, "walkingRange", 0)?,
        spawn_location_is_home: boolean(o, "spawnLocationIsHome")?,
        group: boolean(o, "group")?,
        spawn_position_mode: mode,
        max_spawn_attempts: int(o, "maxSpawnAttempts", 0)?,
        home_range: int(o, "homeRange", -1)?,
        entries,
        spawn_bounds: get(o, "spawnBounds")
            .map(rectangle3d)
            .transpose()?
            .unwrap_or_default(),
        region: string(o, "region")?,
        trigger_range: int(o, "triggerRange", 0)?,
        spawn_message: match get(o, "spawnMessage") {
            None | Some(Value::Null) => None,
            Some(Value::String(s)) => Some(TextDefinition::parse(s)),
            Some(Value::Number(n)) => {
                Some(TextDefinition::of_number(n.as_i64().unwrap_or(0) as i32))
            }
            Some(_) => return Err("spawnMessage: bad value".to_owned()),
        },
        instant: boolean(o, "instant")?,
    })
}

/// `SpawnerDto.ToSpawner()` with `BaseSpawner.ApplyDto`: builds the spawner item (not yet in the world).
pub fn to_spawner(w: &mut World, dto: &SpawnerDto) -> ItemId {
    let ty = match dto.kind {
        Kind::Spawner => &SPAWNER,
        Kind::Region => &REGION_SPAWNER,
        Kind::Proximity => &PROXIMITY_SPAWNER,
    };
    let id = (ty.construct.expect("spawner classes are constructible"))(w);

    if let Some(d) = data_mut(w, id) {
        d.guid = if dto.guid == Guid::default() {
            core::new_guid()
        } else {
            dto.guid
        };
    }
    if dto.name.as_deref().is_some_and(|n| !n.is_empty()) {
        muo_server::item::ops::set_name(w, id, dto.name.clone());
    }
    // compact homeRange -> spawnBounds (the map is not known yet: use the DTO location)
    if dto.home_range >= 0 {
        core::set_spawn_bounds(
            w,
            id,
            core::bounds_from_home_range(dto.location, dto.home_range),
        );
    }
    let bounds = data(w, id).map_or_else(Rectangle3D::default, |d| d.spawn_bounds);
    core::init_spawn(
        w,
        id,
        dto.count,
        dto.min_delay,
        dto.max_delay,
        dto.team,
        bounds,
    );
    if let Some(d) = data_mut(w, id) {
        d.group = dto.group;
        d.walking_range = dto.walking_range;
        d.spawn_location_is_home = dto.spawn_location_is_home;
        d.spawn_position_mode = dto.spawn_position_mode;
        d.max_spawn_attempts = dto.max_spawn_attempts;
        // AdoptEntries
        d.entries = dto.entries.clone();
    }

    match dto.kind {
        Kind::Spawner => {
            if dto.spawn_bounds != Rectangle3D::default() {
                core::set_spawn_bounds(w, id, dto.spawn_bounds);
            }
        }
        Kind::Region => {
            let region = dto
                .map
                .zip(dto.region.as_deref())
                .and_then(|(m, n)| muo_server::region::find_by_name(w, n, m, true));
            core::set_spawn_region(w, id, region);
        }
        Kind::Proximity => {
            if dto.spawn_bounds != Rectangle3D::default() {
                core::set_spawn_bounds(w, id, dto.spawn_bounds);
            }
            if let Some(d) = data_mut(w, id) {
                d.trigger_range = dto.trigger_range;
                d.spawn_message = dto.spawn_message.clone();
                d.instant_flag = dto.instant;
            }
        }
    }
    id
}

/// `Guid -> spawner` of every spawner in the world.
fn all_spawners(w: &World) -> IndexMap<Guid, ItemId> {
    w.items
        .iter()
        .filter_map(|(id, i)| i.ext.get::<super::SpawnerData>().map(|d| (d.guid, *id)))
        .collect()
}

/// `ImportJsonSpawners`: `(generated, failures)`.
pub fn import_json(
    w: &mut World,
    from: Option<MobileId>,
    file: &Path,
    all: &mut IndexMap<Guid, ItemId>,
) -> (i32, i32) {
    let (mut generated, mut failures) = (0, 0);
    let say = |w: &mut World, text: String| {
        if let Some(f) = from {
            mx::send_message(w, f, &text);
        }
    };
    let name = file
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());

    let parsed = std::fs::read_to_string(file)
        .map_err(|e| e.to_string())
        .and_then(|t| {
            muo_data::json::from_str::<Value>(t.trim_start_matches('\u{feff}'))
                .map_err(|e| e.to_string())
        })
        .and_then(|v| match v {
            Value::Array(a) => a
                .iter()
                .map(|x| read_dto(w, x))
                .collect::<Result<Vec<_>, _>>(),
            Value::Null => Ok(Vec::new()),
            _ => Err("expected an array".to_owned()),
        });
    let dtos = match parsed {
        Ok(d) => d,
        Err(e) => {
            tracing::debug!("spawner file {}: {e}", file.display());
            say(
                w,
                format!(
                    "GenerateSpawners: Exception parsing {}, file may not be in the correct format.",
                    file.display()
                ),
            );
            return (0, 0);
        }
    };
    if dtos.is_empty() {
        say(
            w,
            format!("GenerateSpawners: Skipping empty spawner file {name}"),
        );
        tracing::info!("skipping empty spawner file {}", file.display());
        return (0, 0);
    }

    for (i, dto) in dtos.iter().enumerate() {
        let (location, Some(map)) = (dto.location, dto.map.filter(|m| *m != MapId::INTERNAL))
        else {
            tracing::error!("Spawner {} ({i}) has no valid map; skipping.", dto.guid);
            failures += 1;
            continue;
        };

        let id = to_spawner(w, dto);
        let ty = w.item(id).map(|x| x.ty);

        // Delete all existing spawners of the same concrete type at this location.
        let existing: Vec<ItemId> =
            muo_server::map::enumerate::items_at(w, Some(map), location.x, location.y)
                .filter(|e| {
                    *e != id
                        && w.item(*e)
                            .is_some_and(|x| Some(x.ty as *const _) == ty.map(|t| t as *const _))
                })
                .collect();
        for e in existing {
            if let Some(g) = data(w, e).map(|d| d.guid) {
                all.shift_remove(&g);
            }
            delete_item(w, e);
        }

        muo_server::item::ops::move_to_world_map(w, id, location, Some(map));
        core::respawn(w, id);

        let guid = data(w, id).map_or_else(Guid::default, |d| d.guid);
        if let Some(old) = all.shift_remove(&guid) {
            delete_item(w, old);
        }
        all.insert(guid, id);
        generated += 1;
    }
    (generated, failures)
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.delete)(w, id);
    }
}

/// `GetTimeSpan(time)`: `TimeSpan.MinValue` for an empty string, `Err` for a number that does not parse.
fn get_time_span(time: &str) -> Result<TimeSpan, String> {
    let Some(last) = time.chars().last() else {
        return Ok(TimeSpan::MIN);
    };
    let num = |t: &str| t.trim().parse::<f64>().map_err(|e| e.to_string());
    Ok(match last.to_ascii_lowercase() {
        'h' => TimeSpan::from_hours_f64(num(&time[..time.len() - 1])?),
        'm' => TimeSpan::from_mins_f64(num(&time[..time.len() - 1])?),
        's' => TimeSpan::from_secs_f64(num(&time[..time.len() - 1])?),
        _ => TimeSpan::from_mins_f64(num(time)?),
    })
}

/// `GetTimeSpan(timeOverride, time)`: an override other than `MinValue` wins.
fn get_time_span_or(over: TimeSpan, time: &str) -> Result<TimeSpan, String> {
    if over != TimeSpan::MIN {
        Ok(over)
    } else {
        get_time_span(time)
    }
}

fn convert_type(t: &str) -> &str {
    match t.to_ascii_lowercase().as_str() {
        "treasurelevel1" => "treasurechestlevel1",
        "treasurelevel2" => "treasurechestlevel2",
        "treasurelevel3" => "treasurechestlevel3",
        "treasurelevel4" => "treasurechestlevel4",
        "treasurelevel5" => "treasurechestlevel5",
        _ => t,
    }
}

fn create_spawner_entries(w: &World, type_list: &str, max_count: i32) -> Vec<(String, i32)> {
    if type_list.trim().is_empty() {
        return Vec::new();
    }
    type_list
        .split(':')
        .map(|t| {
            let actual = w
                .types
                .find_type_by_name(convert_type(t), false, true)
                .map(|e| {
                    e.full_name()
                        .rsplit(['.', '+'])
                        .next()
                        .unwrap_or(t)
                        .to_owned()
                })
                .unwrap_or_else(|| t.to_owned());
            (actual, max_count)
        })
        .collect()
}

/// `ImportPremiumSpawners`: a Nerun Premium `.map` file.
pub fn import_premium(
    w: &mut World,
    from: Option<MobileId>,
    file: &Path,
    all: &mut IndexMap<Guid, ItemId>,
) -> (i32, i32) {
    let (mut generated, mut failures) = (0, 0);
    let Ok(text) = std::fs::read_to_string(file) else {
        return (0, 0);
    };
    let mut min_over = TimeSpan::MIN;
    let mut max_over = TimeSpan::MIN;
    let mut map_over: i32 = -1;
    let mut id_over: Option<String> = None;
    let report = |w: &mut World| {
        if let Some(f) = from {
            mx::send_message(
                w,
                f,
                &format!(
                    "GenerateSpawners: Exception parsing {}, file may not be in the correct format.",
                    file.display()
                ),
            );
        }
    };

    for (line_no, line) in text.lines().enumerate() {
        if line.starts_with('#') {
            continue;
        }
        if line.starts_with('*') {
            let parts: Vec<&str> = line.split('|').collect();
            let p = |i: usize| parts.get(i).copied().ok_or_else(|| "short line".to_owned());
            let num = |i: usize| p(i)?.trim().parse::<i32>().map_err(|e| e.to_string());
            // An exception before the maps loop ends the whole file (the outer catch returns).
            let maps = (|| -> Result<Vec<MapId>, String> {
                let map_id = if map_over != -1 { map_over } else { num(10)? };
                Ok(if map_id == 0 {
                    vec![MapId::FELUCCA, MapId::TRAMMEL]
                } else {
                    vec![MapId(u8::try_from(map_id - 1).map_err(|e| e.to_string())?)]
                })
            })();
            let maps = match maps {
                Ok(m) => m,
                Err(e) => {
                    tracing::debug!("spawner line {line_no} in {}: {e}", file.display());
                    report(w);
                    return (generated, failures);
                }
            };
            for map in maps {
                // Inside the maps loop an exception costs this map only.
                let result: Result<(), String> = (|| {
                    let mut entries = Vec::new();
                    let mut total = 0;
                    for j in 0..6 {
                        let count = num(j + 16)?;
                        total += count;
                        entries.extend(create_spawner_entries(w, p(j + 1)?, count));
                    }
                    if entries.is_empty() {
                        return Ok(());
                    }
                    let min = get_time_span_or(min_over, p(11)?)?;
                    let max = get_time_span_or(max_over, p(12)?)?;
                    let home_range = num(14)?;
                    let id = core::create(
                        w,
                        &SPAWNER,
                        Kind::Spawner,
                        total,
                        min,
                        max,
                        0,
                        Rectangle3D::default(),
                        &[],
                    );
                    core::set_walking_range(w, id, num(13)?);
                    let label = match &id_over {
                        Some(o) => o.clone(),
                        None => p(15)?.to_owned(),
                    };
                    muo_server::item::ops::set_name(w, id, Some(format!("Spawner ({label})")));
                    for (t, amount) in entries {
                        core::add_entry(w, id, Some(&t), 100, amount, true, None, None);
                    }
                    muo_server::item::ops::move_to_world_map(
                        w,
                        id,
                        Point3D::new(num(7)?, num(8)?, num(9)?),
                        Some(map),
                    );
                    core::set_home_range(w, id, home_range);
                    core::respawn(w, id);
                    if let Some(g) = data(w, id).map(|d| d.guid) {
                        all.insert(g, id);
                    }
                    generated += 1;
                    Ok(())
                })();
                if let Err(e) = result {
                    tracing::debug!("spawner line {line_no} in {}: {e}", file.display());
                    report(w);
                    failures += 1;
                }
            }
            continue;
        }
        let over: Vec<&str> = line.split(' ').collect();
        // `over[1]` missing or a number that doesn't parse throws: the outer catch reports and returns.
        let applied: Result<(), String> = match over[0].to_ascii_lowercase().as_str() {
            "overrideid" => over
                .get(1)
                .map(|v| id_over = Some((*v).to_owned()))
                .ok_or_else(|| "no value".into()),
            "overridemap" => over
                .get(1)
                .ok_or_else(|| "no value".to_owned())
                .and_then(|v| {
                    v.trim()
                        .parse()
                        .map(|m| map_over = m)
                        .map_err(|e: std::num::ParseIntError| e.to_string())
                }),
            "overridemintime" => over
                .get(1)
                .ok_or_else(|| "no value".to_owned())
                .and_then(|v| get_time_span(v).map(|t| min_over = t)),
            "overridemaxtime" => over
                .get(1)
                .ok_or_else(|| "no value".to_owned())
                .and_then(|v| get_time_span(v).map(|t| max_over = t)),
            _ => Ok(()),
        };
        if let Err(e) = applied {
            tracing::debug!("spawner line {line_no} in {}: {e}", file.display());
            report(w);
            return (generated, failures);
        }
    }
    (generated, failures)
}

/// A `Microsoft.Extensions.FileSystemGlobbing` subset: `*`, `?` and `**` over `/`-separated segments.
pub fn glob_files(base: &Path, pattern: &str) -> Vec<PathBuf> {
    fn seg_match(p: &[char], s: &[char]) -> bool {
        match p.first() {
            None => s.is_empty(),
            Some('*') => (0..=s.len()).any(|i| seg_match(&p[1..], &s[i..])),
            Some('?') => !s.is_empty() && seg_match(&p[1..], &s[1..]),
            Some(c) => {
                s.first().is_some_and(|x| {
                    muo_primitives::dotnet::simple_upper(*x)
                        == muo_primitives::dotnet::simple_upper(*c)
                }) && seg_match(&p[1..], &s[1..])
            }
        }
    }
    fn walk(dir: &Path, segs: &[&str], out: &mut Vec<PathBuf>) {
        let Some((first, rest)) = segs.split_first() else {
            return;
        };
        if *first == "**" {
            walk(dir, rest, out);
            if let Ok(rd) = std::fs::read_dir(dir) {
                let mut dirs: Vec<PathBuf> = rd
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.is_dir())
                    .collect();
                dirs.sort();
                for d in dirs {
                    walk(&d, segs, out);
                }
            }
            return;
        }
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        let pat: Vec<char> = first.chars().collect();
        for p in entries {
            let name: Vec<char> = p
                .file_name()
                .map(|n| n.to_string_lossy().chars().collect())
                .unwrap_or_default();
            if !seg_match(&pat, &name) {
                continue;
            }
            if rest.is_empty() {
                if p.is_file() {
                    out.push(p);
                }
            } else if p.is_dir() {
                walk(&p, rest, out);
            }
        }
    }
    let pattern = pattern.replace('\\', "/");
    let segs: Vec<&str> = pattern
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();
    let mut out = Vec::new();
    walk(base, &segs, &mut out);
    out
}

/// `[ImportSpawners <pattern>`.
fn import_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.arguments.is_empty() {
        mx::send_message(
            w,
            from,
            "Usage: [GenerateSpawners <relative search pattern to distribution>",
        );
        return;
    }
    let files = glob_files(&w.core.base_directory.clone(), &e.arguments[0]);
    if files.is_empty() {
        mx::send_message(
            w,
            from,
            "GenerateSpawners: No files found matching the pattern.",
        );
        return;
    }
    let start = std::time::Instant::now();
    let mut all = all_spawners(w);
    let (mut total, mut failed) = (0, 0);
    for file in files {
        let fname = file
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        mx::send_message(
            w,
            from,
            &format!("GenerateSpawners: Generating spawners from {fname}..."),
        );
        tracing::info!(
            "{} is generating spawners from {}",
            from.0.value(),
            file.display()
        );
        let ext = file
            .extension()
            .map(|e| e.to_string_lossy().into_owned()) // `file.Extension == ".json"` is case sensitive
            .unwrap_or_default();
        let (g, f) = match ext.as_str() {
            "json" => import_json(w, Some(from), &file, &mut all),
            "map" => import_premium(w, Some(from), &file, &mut all),
            "xml" => super::export::import_xml(w, Some(from), &file, &mut all),
            other => {
                mx::send_message(
                    w,
                    from,
                    &format!("GenerateSpawners: Unsupported file type .{other}."),
                );
                (0, 0)
            }
        };
        total += g;
        failed += f;
    }
    let secs = start.elapsed().as_secs_f64();
    tracing::info!("Generated {total} spawners ({secs:.2} seconds, {failed} failures)");
    mx::send_message(
        w,
        from,
        &format!(
            "GenerateSpawners: Generated {total} spawners ({secs:.2} seconds, {failed} failures)"
        ),
    );
}

/// `ImportSpawnersCommand.Configure`.
pub fn register_commands(w: &mut World) {
    super::gumps::register(w);
    CommandSystem::register(
        w,
        "ImportSpawners",
        AccessLevel::Developer,
        &["ImportSpawner", "GenerateSpawners", "GenSpawners"],
        Rc::new(import_command),
    );
}

#[cfg(test)]
mod premium_time_tests {
    use super::*;

    #[test]
    fn get_time_span_follows_the_expected_units() {
        assert_eq!(get_time_span("5").unwrap(), TimeSpan::from_mins(5));
        assert_eq!(get_time_span("2H").unwrap(), TimeSpan::from_hours_f64(2.0));
        assert_eq!(get_time_span("30s").unwrap(), TimeSpan::from_secs(30));
        assert_eq!(get_time_span("").unwrap(), TimeSpan::MIN);
        assert!(get_time_span("x").is_err());
        // an override other than MinValue wins and is not parsed
        assert_eq!(
            get_time_span_or(TimeSpan::from_secs(9), "x").unwrap(),
            TimeSpan::from_secs(9)
        );
        assert!(get_time_span_or(TimeSpan::MIN, "x").is_err());
    }
}
