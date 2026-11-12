//! `WorldLocation` and its JSON converter.
//!
//! A location with the map it is on. Maps are [`MapId`] slots, so text forms (`format`, `try_parse`) and the JSON
//! reader take the [`Maps`] that name them. The operators are kept as named methods because several are
//! not the negation/mirror of each other: `!=` is `loc != loc && map != map`, and `<`/`>=`/`<=` inherit
//! `Point3D`'s comparison of `Z` with `>`.

use std::cmp::Ordering;

use muo_primitives::Point3D;
use serde_json::Value;

use crate::World;
use crate::ids::EntityId;
use crate::map::{MapId, Maps};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct WorldLocation {
    pub loc: Point3D,
    pub map: Option<MapId>,
}

impl WorldLocation {
    /// `WorldLocation.Zero`.
    pub const ZERO: WorldLocation = WorldLocation {
        loc: Point3D::ZERO,
        map: Some(MapId::INTERNAL),
    };

    pub const fn new(x: i32, y: i32, z: i32, map: Option<MapId>) -> WorldLocation {
        WorldLocation {
            loc: Point3D::new(x, y, z),
            map,
        }
    }

    pub const fn from_point(loc: Point3D, map: Option<MapId>) -> WorldLocation {
        WorldLocation { loc, map }
    }

    /// `new WorldLocation(IEntity)`.
    pub fn from_entity(w: &World, e: EntityId) -> Option<WorldLocation> {
        match e {
            EntityId::Item(i) => w
                .item(i)
                .map(|i| WorldLocation::from_point(i.location, i.map)),
            EntityId::Mobile(m) => w
                .mobile(m)
                .map(|m| WorldLocation::from_point(m.location, m.map)),
        }
    }

    /// `CompareTo`: the point, then `Comparer<Map>.Default` (null first, then `Map.CompareTo`).
    pub fn compare_to(&self, other: &WorldLocation) -> Ordering {
        self.loc
            .cmp(&other.loc)
            .then_with(|| self.map.cmp(&other.map))
    }

    /// `operator !=`: true only when both the point and the map differ.
    pub fn op_ne(&self, r: &WorldLocation) -> bool {
        self.loc != r.loc && self.map != r.map
    }

    pub fn op_gt(&self, r: &WorldLocation) -> bool {
        self.loc.gt_all(r.loc) && self.map == r.map
    }

    pub fn op_lt(&self, r: &WorldLocation) -> bool {
        self.loc.lt_all(r.loc) && self.map == r.map
    }

    pub fn op_ge(&self, r: &WorldLocation) -> bool {
        self.loc.ge_all(r.loc) && self.map == r.map
    }

    pub fn op_le(&self, r: &WorldLocation) -> bool {
        self.loc.le_all(r.loc) && self.map == r.map
    }

    /// `operator ==(WorldLocation, IEntity)`: false for a null entity.
    pub fn eq_entity(&self, w: &World, r: Option<EntityId>) -> bool {
        r.and_then(|e| WorldLocation::from_entity(w, e))
            .is_some_and(|r| self.loc == r.loc && self.map == r.map)
    }

    /// `operator !=(WorldLocation, IEntity)`: both must differ, and a null entity is false.
    pub fn ne_entity(&self, w: &World, r: Option<EntityId>) -> bool {
        r.and_then(|e| WorldLocation::from_entity(w, e))
            .is_some_and(|r| self.op_ne(&r))
    }

    /// `(x, y, z) [Map]`, or `[(-null-)]` without a map.
    pub fn format(&self, maps: &Maps) -> String {
        let map = self
            .map
            .and_then(|m| maps.get(m))
            .map_or("(-null-)", |m| m.name());
        format!("({}, {}, {}) [{}]", self.loc.x, self.loc.y, self.loc.z, map)
    }

    /// `TryParse`: `"(x, y, z) [Map]"`.
    pub fn try_parse(maps: &Maps, s: &str) -> Option<WorldLocation> {
        let s = s.trim();
        if !s.ends_with(']') {
            return None;
        }
        let start = s.find('[')?;
        let loc = Point3D::try_parse(s.get(..start.checked_sub(1)?)?)?;
        let map_slice = s.get(start + 1..s.len() - 1)?;
        if map_slice == "(-null-)" {
            return Some(WorldLocation { loc, map: None });
        }
        Some(WorldLocation {
            loc,
            map: Some(maps.parse(map_slice)?),
        })
    }

    /// `WorldLocationConverter.Read`: a `"(x, y, z) [Map]"` string, `[x, y, z, "Map"]`, or an object with
    /// `x`/`y`/`z` or `loc`, and `map`.
    pub fn from_json(maps: &Maps, v: &Value) -> Result<WorldLocation, String> {
        match v {
            Value::String(s) => Self::try_parse(maps, s)
                .ok_or_else(|| format!("The input string '{s}' was not in a correct format.")),
            Value::Array(a) => Self::from_json_array(maps, a),
            Value::Object(o) => Self::from_json_object(maps, o),
            _ => Err("Invalid Json for Point3D".to_string()),
        }
    }

    fn from_json_array(maps: &Maps, a: &[Value]) -> Result<WorldLocation, String> {
        let mut data = [0i32; 3];
        let mut count = 0;
        let mut has_map = false;
        let mut map = None;

        for v in a {
            if let Value::Number(n) = v {
                let n = n.as_i64().ok_or("Value is not a valid integer")? as i32;
                if count < 3 {
                    data[count] = n;
                } else if count == 3 {
                    map = u8::try_from(n)
                        .ok()
                        .map(MapId)
                        .filter(|&id| maps.get(id).is_some());
                    has_map = true;
                }
                count += 1;
            }

            if let Value::String(key) = v {
                if count != 3 || has_map {
                    return Err(format!("Value {key} is not valid for this element."));
                }
                map = Some(maps.parse(key).ok_or_else(|| {
                    format!("The input string '{key}' was not in a correct format.")
                })?);
                has_map = true;
                break;
            }
        }

        // A numeric map index is counted too, so only the string form passes this check.
        if !has_map || count != 3 {
            return Err("WorldLocation must be an array of x, y, z, and map".to_string());
        }
        Ok(WorldLocation::new(data[0], data[1], data[2], map))
    }

    fn from_json_object(
        maps: &Maps,
        o: &serde_json::Map<String, Value>,
    ) -> Result<WorldLocation, String> {
        let mut data = [0i32; 3];
        let (mut has_loc, mut has_xyz, mut has_map) = (false, false, false);
        let mut map = None;

        for (key, v) in o {
            let i = match key.as_str() {
                "x" => 0,
                "y" => 1,
                "z" => 2,
                "loc" => 3,
                "map" => 4,
                _ => continue,
            };

            if i < 3 {
                if has_loc {
                    return Err("WorldLocation must have loc or x, y, z, but not both".to_string());
                }
                let Value::Number(n) = v else {
                    return Err(format!("Value for {key} must be a number"));
                };
                has_xyz = true;
                data[i] = n.as_i64().ok_or("Value is not a valid integer")? as i32;
                continue;
            }

            if i == 3 {
                if has_xyz {
                    return Err("WorldLocation must have loc or x, y, z, but not both".to_string());
                }
                has_loc = true;
                let p = muo_data::json::converters::point3d::deserialize(v.clone())
                    .map_err(|e| e.to_string())?;
                data = [p.x, p.y, p.z];
                continue;
            }

            map = Some(crate::region_json::resolve_map_in(maps, v)?);
            has_map = true;
        }

        if !has_map {
            return Err("WorldLocation must have an x, y, z, and map properties".to_string());
        }
        Ok(WorldLocation::new(data[0], data[1], data[2], map))
    }

    /// `WorldLocationConverter.Write`: `[x, y, z, "Map"]`.
    pub fn to_json(&self, maps: &Maps) -> Value {
        let map = self
            .map
            .and_then(|m| maps.get(m))
            .map_or(String::new(), |m| m.name().to_string());
        serde_json::json!([self.loc.x, self.loc.y, self.loc.z, map])
    }
}

impl Default for WorldLocation {
    fn default() -> Self {
        WorldLocation {
            loc: Point3D::ZERO,
            map: None,
        }
    }
}

#[cfg(test)]
mod tests {
    //! WorldLocation tests (the map name is Felucca here; the test world has the fixed facets
    //! rather than an ad-hoc map per name).
    use super::*;
    use crate::map::test_support::world;

    #[test]
    fn world_location_to_string() {
        let w = world();
        let map = Some(MapId::FELUCCA);
        let name = w.maps.get(MapId::FELUCCA).unwrap().name().to_string();
        let (max, min) = (i32::MAX, i32::MIN);
        let null = "(-null-)";

        assert_eq!(
            WorldLocation::new(0, 0, 0, map).format(&w.maps),
            format!("(0, 0, 0) [{name}]")
        );
        assert_eq!(
            WorldLocation::new(1, 1, 1, map).format(&w.maps),
            format!("(1, 1, 1) [{name}]")
        );
        assert_eq!(
            WorldLocation::new(0, 0, 0, None).format(&w.maps),
            format!("(0, 0, 0) [{null}]")
        );
        assert_eq!(
            WorldLocation::new(1, 1, 1, None).format(&w.maps),
            format!("(1, 1, 1) [{null}]")
        );
        assert_eq!(
            WorldLocation::new(max, max, max, map).format(&w.maps),
            format!("({max}, {max}, {max}) [{name}]")
        );
        assert_eq!(
            WorldLocation::new(min, min, min, map).format(&w.maps),
            format!("({min}, {min}, {min}) [{name}]")
        );
        assert_eq!(
            WorldLocation::new(max, max, max, None).format(&w.maps),
            format!("({max}, {max}, {max}) [{null}]")
        );
        assert_eq!(
            WorldLocation::new(min, min, min, None).format(&w.maps),
            format!("({min}, {min}, {min}) [{null}]")
        );
    }
}
