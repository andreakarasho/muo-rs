//! The `Data/regions.json` reader.
//!
//! The polymorphism by `$type` is a registry of class names ([`crate::region::RegionRegistry`], held by
//! `World.maps.region_types`) that muo_content fills with `register`. An entry without `$type` is the base
//! `Region`; an unknown name is an error. `Region`'s own properties (`Name`, `Map`, `Priority`, `Area`,
//! `Parent`, `GoLocation`, `Music`, `MinExpansion`, `MaxExpansion`) are read here, case-insensitively; every
//! other property goes to the class's `from_json`.

use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use std::path::Path;

use muo_data::expansion::Expansion;
use muo_data::json::converters::rectangle3d;
use muo_primitives::{Point3D, Rectangle3D};
use serde_json::Value;

use crate::World;
use crate::map::MapId;
use crate::region::{
    MusicName, RegionId, RegionJson, RegionType, delete_region, find_by_name, new_region, register,
};

const COMMON: [&str; 10] = [
    "$type",
    "name",
    "map",
    "priority",
    "area",
    "parent",
    "golocation",
    "music",
    "minexpansion",
    "maxexpansion",
];

fn get<'a>(obj: &'a RegionJson, key: &str) -> Option<&'a Value> {
    obj.iter().find(|(k, _)| k.eq_ic(key)).map(|(_, v)| v)
}

/// `MapConverter`: a name or an index into `Map.Maps`.
pub fn resolve_map(w: &World, v: &Value) -> Result<MapId, String> {
    resolve_map_in(&w.maps, v)
}

/// `MapConverter.Read` over the map table.
pub fn resolve_map_in(maps: &crate::map::Maps, v: &Value) -> Result<MapId, String> {
    match v {
        Value::String(s) => maps
            .parse(s)
            .ok_or_else(|| format!("The input string '{s}' was not in a correct format.")),
        Value::Number(n) => {
            let i = n.as_i64().ok_or("Value must be a number or string")?;
            u8::try_from(i)
                .ok()
                .map(MapId)
                .filter(|&id| maps.get(id).is_some())
                .ok_or_else(|| format!("Map {i} does not exist"))
        }
        _ => Err("Value must be a number or string".to_string()),
    }
}

fn expansion_of(v: &Value) -> Result<Expansion, String> {
    match v {
        Value::String(s) => (0..12)
            .filter_map(Expansion::from_index)
            .find(|e| e.name().eq_ic(s.trim()))
            .or_else(|| Expansion::parse(s))
            .ok_or_else(|| format!("The JSON value '{s}' could not be converted to Expansion")),
        Value::Number(n) => n
            .as_i64()
            .and_then(|i| Expansion::from_index(i as i32))
            .ok_or_else(|| "Expansion out of range".to_string()),
        _ => Err("Invalid json for Expansion".to_string()),
    }
}

fn music_of(v: &Value) -> Result<MusicName, String> {
    match v {
        Value::String(s) => MusicName::from_name(s)
            .ok_or_else(|| format!("The JSON value '{s}' could not be converted to MusicName")),
        Value::Number(n) => n
            .as_i64()
            .and_then(|i| MusicName::from_i32(i as i32))
            .ok_or_else(|| "MusicName out of range".to_string()),
        _ => Err("Invalid json for MusicName".to_string()),
    }
}

/// `RegionByNameConverter`: `{ "Name", "Map" }`, found with `Region.Find(name, map)`; `None` when no such
/// region is registered (yet).
fn parent_of_json(w: &World, v: &Value) -> Result<Option<RegionId>, String> {
    let Value::Object(o) = v else {
        return Err("Invalid json for RegionByName".to_string());
    };
    let (mut name, mut map) = (None, None);
    for (k, v) in o {
        let slot = match k.lower_inv().as_str() {
            "name" => &mut name,
            "map" => &mut map,
            _ => continue,
        };
        let Value::String(s) = v else {
            return Err(format!("Value for {k} must be a string"));
        };
        *slot = Some(s.clone());
    }
    let (Some(name), Some(map)) = (name, map) else {
        return Err("Invalid json for RegionByName".to_string());
    };
    let map = w
        .maps
        .parse(&map)
        .ok_or_else(|| format!("The input string '{map}' was not in a correct format."))?;
    Ok(find_by_name(w, &name, map, false))
}

/// Reads one entry, builds the region, and registers it when the expansion allows. `Ok(true)` if registered.
fn load_entry(w: &mut World, entry: &Value, expansion: Expansion) -> Result<bool, String> {
    let Value::Object(obj) = entry else {
        return Err("Invalid json for Region".to_string());
    };

    let ty: &'static RegionType = match obj.get("$type") {
        None | Some(Value::Null) => &crate::region::REGION_BASE,
        Some(Value::String(name)) => w
            .maps
            .region_types
            .get(name)
            .ok_or_else(|| format!("Unknown region type '{name}' in regions.json"))?,
        Some(_) => return Err("The metadata property $type must be a string".to_string()),
    };

    let name = match get(obj, "name") {
        Some(Value::String(s)) => Some(s.as_str()),
        Some(Value::Null) | None => None,
        Some(_) => return Err("Region Name must be a string".to_string()),
    };
    let map = resolve_map(w, get(obj, "map").ok_or("Region has no Map")?)?;
    // A constructor parameter that is absent is default(int).
    let priority = match get(obj, "priority") {
        Some(v) => v.as_i64().ok_or("Region Priority must be a number")? as i32,
        None => 0,
    };
    let area: Vec<Rectangle3D> = match get(obj, "area") {
        Some(Value::Array(a)) => a
            .iter()
            .map(|v| rectangle3d::deserialize(v.clone()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?,
        _ => return Err("Region has no Area".to_string()),
    };

    // Built with no parent, as the JSON constructor does: the parent is assigned after, so a child that doesn't
    // name its Music keeps `Invalid`.
    let id = new_region(w, ty, name, map, None, Some(priority), area);

    let result = apply_properties(w, id, obj, ty, expansion);
    match result {
        Ok(true) => {
            register(w, id);
            Ok(true)
        }
        Ok(false) => {
            delete_region(w, id);
            Ok(false)
        }
        Err(e) => {
            delete_region(w, id);
            Err(e)
        }
    }
}

fn apply_properties(
    w: &mut World,
    id: RegionId,
    obj: &RegionJson,
    ty: &'static RegionType,
    expansion: Expansion,
) -> Result<bool, String> {
    if let Some(v) = get(obj, "parent") {
        let parent = if v.is_null() {
            None
        } else {
            parent_of_json(w, v)?
        };
        if let Some(r) = w.region_mut(id) {
            r.parent = parent;
        }
    }
    if let Some(v) = get(obj, "golocation") {
        let p: Point3D = muo_data::json::converters::point3d::deserialize(v.clone())
            .map_err(|e| e.to_string())?;
        if let Some(r) = w.region_mut(id) {
            r.go_location = p;
        }
    }
    if let Some(v) = get(obj, "music") {
        let music = music_of(v)?;
        if let Some(r) = w.region_mut(id) {
            r.music = music;
        }
    }
    if let Some(v) = get(obj, "minexpansion") {
        let e = expansion_of(v)?;
        if let Some(r) = w.region_mut(id) {
            r.min_expansion = e;
        }
    }
    if let Some(v) = get(obj, "maxexpansion") {
        let e = expansion_of(v)?;
        if let Some(r) = w.region_mut(id) {
            r.max_expansion = e;
        }
    }

    let rest: RegionJson = obj
        .iter()
        .filter(|(k, _)| !COMMON.iter().any(|c| k.eq_ic(c)))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    (ty.from_json)(w, id, &rest)?;

    let Some(r) = w.region(id) else {
        return Ok(false);
    };
    if expansion >= r.min_expansion && expansion <= r.max_expansion {
        // The child level is not part of the JSON data.
        let parent_level = r
            .parent
            .and_then(|p| w.region(p))
            .map(|p| p.child_level + 1);
        if let (Some(level), Some(r)) = (parent_level, w.region_mut(id)) {
            r.child_level = level;
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

/// `LoadRegions` over the text of `regions.json`: how many regions were registered.
pub fn load_regions_from_str(
    w: &mut World,
    text: &str,
    expansion: Expansion,
) -> Result<usize, String> {
    let entries: Vec<Value> =
        muo_data::json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())?;
    let mut count = 0;
    for entry in &entries {
        if load_entry(w, entry, expansion)? {
            count += 1;
        }
    }
    Ok(count)
}

/// `RegionJsonSerializer.LoadRegions()`: `Data/regions.json` under `base_dir`, for `Core.Expansion`.
pub fn load_regions(w: &mut World, base_dir: &Path, expansion: Expansion) -> Result<usize, String> {
    let path = base_dir.join("Data/regions.json");
    tracing::info!("Loading regions");
    let started = std::time::Instant::now();
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let count = load_regions_from_str(w, &text, expansion)?;
    tracing::info!(
        "Loading regions done ({count} regions) ({:.2} seconds)",
        started.elapsed().as_secs_f64()
    );
    Ok(count)
}
