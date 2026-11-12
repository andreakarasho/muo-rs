//! The JSON converters as serde helpers with the same JSON shapes.
//!
//! Use a module on a field: `#[serde(with = "muo_data::json::converters::point3d")]`. Modules with an optional
//! counterpart have an `option` submodule (`...::client_version::option`) for `Option<T>` fields.
//!
//! Shapes that name a `Map`, `Region` or `Type` resolve at a level above (muo_server): [`MapRef`] and
//! [`RegionRef`] hold their JSON forms (a map is its name or its index number; written as the name), a type is
//! just its full-name string. `WorldLocationConverter` (x/y/z plus a map, as an array or object) lives with
//! `WorldLocation` in muo_server.

use crate::client::ClientVersion;
use muo_io::TextDefinition;
use muo_io::serialization::reader::Guid;
use muo_primitives::dotnet::InvariantCase;
use muo_primitives::{Point2D, Point3D, Rectangle3D, TimeSpan};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::net::SocketAddr;

fn int32<E: serde::de::Error>(v: &Value, what: &str) -> Result<i32, E> {
    v.as_i64()
        .and_then(|n| i32::try_from(n).ok())
        .ok_or_else(|| E::custom(format!("Value for {what} must be a number")))
}

/// Numbers of a JSON array, at most `max` of them (non-numbers are skipped); more is an error.
fn array_numbers<E: serde::de::Error>(a: &[Value], max: usize, err: &str) -> Result<Vec<i32>, E> {
    let nums: Vec<&Value> = a.iter().filter(|v| v.is_number()).collect();
    if nums.len() > max {
        return Err(E::custom(err));
    }
    nums.iter().map(|v| int32(v, "element")).collect()
}

/// `Point3DConverter`: a `"(x, y, z)"` string, `[x, y, z]` (missing entries 0) or `{"x","y","z"}`; written as the object.
pub mod point3d {
    use super::*;

    pub fn serialize<S: Serializer>(p: &Point3D, s: S) -> Result<S::Ok, S::Error> {
        serde_json::json!({"x": p.x, "y": p.y, "z": p.z}).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Point3D, D::Error> {
        from_value(Value::deserialize(d)?)
    }

    pub(crate) fn from_value<E: serde::de::Error>(v: Value) -> Result<Point3D, E> {
        match v {
            Value::String(s) => {
                Point3D::try_parse(&s).ok_or_else(|| E::custom("Invalid Json for Point3D"))
            }
            Value::Array(a) => {
                let n = array_numbers(&a, 3, "Point3D must be an array of x, y, z")?;
                let g = |i: usize| n.get(i).copied().unwrap_or(0);
                Ok(Point3D::new(g(0), g(1), g(2)))
            }
            Value::Object(m) => {
                let mut d = [0i32; 3];
                for (k, v) in &m {
                    let i = match k.as_str() {
                        "x" => 0,
                        "y" => 1,
                        "z" => 2,
                        _ => return Err(E::custom(format!("Invalid property {k} for Point3D"))),
                    };
                    d[i] = int32(v, k)?;
                }
                Ok(Point3D::new(d[0], d[1], d[2]))
            }
            _ => Err(E::custom("Invalid Json for Point3D")),
        }
    }
}

/// `Point2DConverter`: a string, `[x, y]` or `{"x","y"}`; written as the array.
pub mod point2d {
    use super::*;

    pub fn serialize<S: Serializer>(p: &Point2D, s: S) -> Result<S::Ok, S::Error> {
        [p.x, p.y].serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Point2D, D::Error> {
        match Value::deserialize(d)? {
            Value::String(s) => {
                Point2D::try_parse(&s).ok_or_else(|| D::Error::custom("Invalid json for Point2D"))
            }
            Value::Array(a) => {
                let n = array_numbers(&a, 2, "Point2D must be an array of x, y")?;
                let g = |i: usize| n.get(i).copied().unwrap_or(0);
                Ok(Point2D::new(g(0), g(1)))
            }
            Value::Object(m) => {
                let mut d = [0i32; 2];
                for (k, v) in &m {
                    let i = match k.as_str() {
                        "x" => 0,
                        "y" => 1,
                        _ => {
                            return Err(D::Error::custom(format!(
                                "Invalid property {k} for Point2D"
                            )));
                        }
                    };
                    d[i] = int32(v, k)?;
                }
                Ok(Point2D::new(d[0], d[1]))
            }
            _ => Err(D::Error::custom("Invalid json for Point2D")),
        }
    }
}

/// `Rectangle3DConverter`: a string, `[x, y, z, w, h, d]`, or an object of exactly one style: `x y z w|width h|height
/// d|depth`, `x1 y1 z1 x2 y2 z2`, or `start`/`end` points. Without a z the box spans -128..127. Written as
/// `x1 y1 [z1] x2 y2 [z2]`, z omitted only for exactly that full range.
pub mod rectangle3d {
    use super::*;

    pub fn serialize<S: Serializer>(r: &Rectangle3D, s: S) -> Result<S::Ok, S::Error> {
        let write_z = r.start.z != i32::from(i8::MIN) || r.end.z != i32::from(i8::MAX);
        let mut m = serde_json::Map::new();
        m.insert("x1".into(), r.start.x.into());
        m.insert("y1".into(), r.start.y.into());
        if write_z {
            m.insert("z1".into(), r.start.z.into());
        }
        m.insert("x2".into(), r.end.x.into());
        m.insert("y2".into(), r.end.y.into());
        if write_z {
            m.insert("z2".into(), r.end.z.into());
        }
        Value::Object(m).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Rectangle3D, D::Error> {
        type E<'de, D> = <D as Deserializer<'de>>::Error;
        match Value::deserialize(d)? {
            Value::String(s) => Rectangle3D::try_parse(&s)
                .ok_or_else(|| E::<D>::custom("Invalid Json for Rectangle3D")),
            Value::Array(a) => {
                let n = array_numbers::<E<D>>(
                    &a,
                    6,
                    "Rectangle3D must be an array of x, y, z, h, w, d",
                )?;
                let g = |i: usize| n.get(i).copied().unwrap_or(0);
                Ok(Rectangle3D::new(g(0), g(1), g(2), g(3), g(4), g(5)))
            }
            Value::Object(m) => from_object::<E<D>>(&m),
            _ => Err(E::<D>::custom("Invalid Json for Rectangle3D")),
        }
    }

    fn from_object<E: serde::de::Error>(
        m: &serde_json::Map<String, Value>,
    ) -> Result<Rectangle3D, E> {
        const BOTH: &str = "Rectangle3D must have a start/end, or x/y/z/w/h/d, but not both.";
        let mut data = [0i32; 6];
        // 0 - xyzwhd, 1 - x1y1z1x2y2z2, 2 - start/end
        let mut obj_type = -1;
        let mut has_z = false;
        for (key, v) in m {
            if key == "start" || key == "end" {
                if obj_type > -1 && obj_type != 2 {
                    return Err(E::custom(BOTH));
                }
                obj_type = 2;
                let p = point3d::from_value::<E>(v.clone())?;
                let off = if key == "end" { 3 } else { 0 };
                data[off] = p.x;
                data[1 + off] = p.y;
                data[2 + off] = p.z;
                continue;
            }
            let i = match key.as_str() {
                "x" => 0,
                "y" => 1,
                "z" => 2,
                "w" | "width" => 3,
                "h" | "height" => 4,
                "d" | "depth" => 5,
                "x1" => 10,
                "y1" => 11,
                "z1" => 12,
                "x2" => 13,
                "y2" => 14,
                "z2" => 15,
                _ => return Err(E::custom(format!("Invalid property {key} for Rectangle3D"))),
            };
            let want = if i < 10 { 0 } else { 1 };
            if obj_type > -1 && obj_type != want {
                return Err(E::custom(BOTH));
            }
            obj_type = want;
            let slot = if i < 10 { i } else { i - 10 };
            data[slot] = int32(v, key)?;
            if i == 2 || i == 12 || i == 15 {
                has_z = true;
            }
        }
        if !has_z {
            // Bottom to top?
            data[2] = -128;
            data[5] = 127;
        }
        Ok(if obj_type == 0 {
            Rectangle3D::new(data[0], data[1], data[2], data[3], data[4], data[5])
        } else {
            Rectangle3D::from_points(
                Point3D::new(data[0], data[1], data[2]),
                Point3D::new(data[3], data[4], data[5]),
            )
        })
    }
}

/// `TimeSpanConverter`: the constant format string, `"00:05:00"`.
pub mod timespan {
    use super::*;

    pub fn serialize<S: Serializer>(t: &TimeSpan, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(t)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<TimeSpan, D::Error> {
        let text = String::deserialize(d)?;
        TimeSpan::try_parse(&text)
            .ok_or_else(|| D::Error::custom(format!("The TimeSpan '{text}' could not be parsed")))
    }
}

/// `ClientVersionConverter`: a string.
pub mod client_version {
    use super::*;

    pub fn serialize<S: Serializer>(v: &ClientVersion, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(v.source_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<ClientVersion, D::Error> {
        Ok(ClientVersion::parse(&String::deserialize(d)?))
    }

    /// For `Option<ClientVersion>` fields (`null` and absent are `None`).
    pub mod option {
        use super::*;

        pub fn serialize<S: Serializer>(
            v: &Option<ClientVersion>,
            s: S,
        ) -> Result<S::Ok, S::Error> {
            match v {
                Some(v) => s.serialize_str(v.source_string()),
                None => s.serialize_none(),
            }
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(
            d: D,
        ) -> Result<Option<ClientVersion>, D::Error> {
            Ok(Option::<String>::deserialize(d)?.map(|s| ClientVersion::parse(&s)))
        }
    }
}

/// `IPEndPointConverter`: `"ip:port"` / `"[v6]:port"`.
pub mod ip_endpoint {
    use super::*;

    pub fn serialize<S: Serializer>(v: &SocketAddr, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(v)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<SocketAddr, D::Error> {
        String::deserialize(d)?
            .parse()
            .map_err(|_| D::Error::custom("IPEndPoint must be in the correct format"))
    }
}

/// `GuidConverter`: the 36 character hyphenated lowercase form.
pub mod guid {
    use super::*;

    pub fn serialize<S: Serializer>(v: &Guid, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(v)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Guid, D::Error> {
        let text = String::deserialize(d)?;
        parse_guid(&text).ok_or_else(|| D::Error::custom("The value is not a valid Guid"))
    }

    /// Hyphenated (8-4-4-4-12), or 32 bare hex digits, optionally wrapped in `{}` or `()`.
    pub fn parse_guid(text: &str) -> Option<Guid> {
        let t = text.trim();
        let t = t
            .strip_prefix('{')
            .and_then(|r| r.strip_suffix('}'))
            .unwrap_or(t);
        let t = t
            .strip_prefix('(')
            .and_then(|r| r.strip_suffix(')'))
            .unwrap_or(t);
        let hex: String = if t.len() == 36 {
            let parts: Vec<&str> = t.split('-').collect();
            if parts.iter().map(|p| p.len()).collect::<Vec<_>>() != [8, 4, 4, 4, 12] {
                return None;
            }
            parts.concat()
        } else {
            t.to_string()
        };
        if hex.len() != 32 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let mut b = [0u8; 16];
        for (i, byte) in b.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
        }
        Some(Guid(b))
    }
}

/// `TextDefinitionConverter`: a number (cliloc) or a string.
pub mod text_definition {
    use super::*;

    pub fn serialize<S: Serializer>(v: &TextDefinition, s: S) -> Result<S::Ok, S::Error> {
        if v.number > 0 {
            s.serialize_i32(v.number)
        } else {
            match &v.string {
                Some(t) => s.serialize_str(t),
                None => s.serialize_none(),
            }
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<TextDefinition, D::Error> {
        match Value::deserialize(d)? {
            Value::String(s) => Ok(TextDefinition::of_text(s)),
            Value::Number(n) => Ok(TextDefinition::of_number(int32(
                &Value::Number(n),
                "TextDefinition",
            )?)),
            _ => Err(D::Error::custom(
                "TextDefinition value must be an integer or string",
            )),
        }
    }
}

/// A flags enum as `{ "Name": bool, ... }` (`FlagsConverter<T>`): every single-bit name (aliases included, in
/// ascending value order) is written; reading ORs the names set to true and ignores unknown names.
pub trait JsonFlags: Copy {
    /// `Enum.GetNames` order: by value, ties in declaration order.
    const NAMES: &'static [(&'static str, u64)];
    fn to_bits(self) -> u64;
    fn from_bits(bits: u64) -> Self;
}

/// Implements [`JsonFlags`] for a `bitflags` type from its name table.
#[macro_export]
macro_rules! json_flags {
    ($ty:ty, [$(($name:literal, $value:expr)),* $(,)?]) => {
        impl $crate::json::converters::JsonFlags for $ty {
            const NAMES: &'static [(&'static str, u64)] = &[$(($name, $value as u64)),*];
            fn to_bits(self) -> u64 {
                self.bits() as u64
            }
            fn from_bits(bits: u64) -> Self {
                <$ty>::from_bits_retain(bits as _)
            }
        }
    };
}

pub mod flags {
    use super::*;

    pub fn to_value<T: JsonFlags>(v: T) -> Value {
        let bits = v.to_bits();
        let mut m = serde_json::Map::new();
        for &(name, flag) in T::NAMES {
            // Do not write out multi-bit values. This is a custom behavior
            if flag > 0 && flag & (flag - 1) == 0 {
                m.insert(name.to_string(), Value::Bool(bits & flag == flag));
            }
        }
        Value::Object(m)
    }

    pub fn from_value<T: JsonFlags, E: serde::de::Error>(v: &Value) -> Result<T, E> {
        let Value::Object(m) = v else {
            return Err(E::custom("Invalid Json structure for Flag object"));
        };
        let mut bits = 0u64;
        for (key, v) in m {
            let on = v
                .as_bool()
                .ok_or_else(|| E::custom("Invalid Json structure for Flag object"))?;
            if on && let Some(&(_, flag)) = T::NAMES.iter().find(|(n, _)| n == key) {
                bits |= flag;
            }
        }
        Ok(T::from_bits(bits))
    }

    pub fn serialize<T: JsonFlags, S: Serializer>(v: &T, s: S) -> Result<S::Ok, S::Error> {
        to_value(*v).serialize(s)
    }

    pub fn deserialize<'de, T: JsonFlags, D: Deserializer<'de>>(d: D) -> Result<T, D::Error> {
        from_value(&Value::deserialize(d)?)
    }
}

/// `BitArrayEnumIndexConverter<T>`: a `{ "Name": bool }` object over enum names, with the enum value as the bit
/// index. `names` is the enum's `(name, value)` table.
pub mod bit_array {
    use super::*;

    pub fn to_value(names: &[(&str, usize)], bits: &[bool]) -> Value {
        Value::Object(
            names
                .iter()
                .map(|&(n, i)| {
                    (
                        n.to_string(),
                        Value::Bool(bits.get(i).copied().unwrap_or(false)),
                    )
                })
                .collect(),
        )
    }

    pub fn from_value<E: serde::de::Error>(
        names: &[(&str, usize)],
        v: &Value,
    ) -> Result<Vec<bool>, E> {
        let Value::Object(m) = v else {
            return Err(E::custom("Invalid Json structure for Flag object"));
        };
        let mut bits = vec![false; names.iter().map(|n| n.1).max().map_or(0, |max| max + 1)];
        for (key, v) in m {
            let on = v
                .as_bool()
                .ok_or_else(|| E::custom("Invalid Json structure for Flag object"))?;
            if on && let Some(&(_, i)) = names.iter().find(|(n, _)| n == key) {
                bits[i] = true;
            }
        }
        Ok(bits)
    }
}

/// `MapConverter`'s JSON: a map name, or an index into `Map.Maps`. Written as the name. muo_server resolves it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MapRef {
    Name(String),
    Index(i32),
}

/// `RegionByNameConverter`'s JSON: `{ "Name": ..., "Map": ... }` (keys read case-insensitively; any other key is an
/// error, since the reader moves on without skipping its value and the value token isn't a property name; both
/// required). muo_server resolves it with `Region.Find(name, map)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionRef {
    pub name: String,
    pub map: String,
}

impl Serialize for RegionRef {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        serde_json::json!({"Name": self.name, "Map": self.map}).serialize(s)
    }
}

impl<'de> Deserialize<'de> for RegionRef {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let Value::Object(m) = Value::deserialize(d)? else {
            return Err(D::Error::custom("Invalid json for RegionByName"));
        };
        let (mut name, mut map) = (None, None);
        for (k, v) in &m {
            let k = k.lower_inv();
            let slot = match k.as_str() {
                "name" => &mut name,
                "map" => &mut map,
                _ => return Err(D::Error::custom("Invalid json for RegionByName")),
            };
            let Value::String(s) = v else {
                return Err(D::Error::custom(format!("Value for {k} must be a string")));
            };
            *slot = Some(s.clone());
        }
        match (name, map) {
            (Some(name), Some(map)) => Ok(RegionRef { name, map }),
            _ => Err(D::Error::custom("Invalid json for RegionByName")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_by_name_rejects_unknown_keys() {
        assert!(serde_json::from_str::<RegionRef>(r#"{"Name":"a","MAP":"Felucca"}"#).is_ok());
        assert!(
            serde_json::from_str::<RegionRef>(r#"{"Name":"a","Map":"Felucca","x":1}"#).is_err()
        );
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Rect(#[serde(with = "rectangle3d")] Rectangle3D);

    fn round_trip(r: Rectangle3D) -> Rectangle3D {
        let json = crate::json::to_string(&Rect(r)).unwrap();
        crate::json::from_str::<Rect>(&json).unwrap().0
    }

    #[test]
    fn rectangle3d_round_trips() {
        for (x, y, z, w, h, d) in [
            // homeRange-style full vertical range (z1=-128, depth 256 -> z2=128): must survive losslessly.
            (100, 100, -128, 11, 11, 256),
            // The omitted-z sentinel itself (z1=-128, z2=127 -> depth 255).
            (100, 100, -128, 11, 11, 255),
            (100, 100, 0, 11, 11, 5),
            (100, 100, 5, 11, 11, 122),    // z2 == 127 but z1 != -128
            (100, 100, -128, 11, 11, 200), // z1 == -128 but z2 != 127
        ] {
            let rect = Rectangle3D::new(x, y, z, w, h, d);
            assert_eq!(rect, round_trip(rect));
        }
    }

    #[test]
    fn rectangle3d_full_range_sentinel_omits_z() {
        let json =
            crate::json::to_string(&Rect(Rectangle3D::new(0, 0, -128, 10, 10, 255))).unwrap();
        assert!(!json.contains("z1"));
        assert!(!json.contains("z2"));
    }

    #[test]
    fn rectangle3d_read_shapes() {
        let r: Rect = crate::json::from_str(r#"[1, 2, 3, 4, 5, 6]"#).unwrap();
        assert_eq!(r.0, Rectangle3D::new(1, 2, 3, 4, 5, 6));
        let r: Rect = crate::json::from_str(r#"{"x":1,"y":2,"width":4,"height":5}"#).unwrap();
        assert_eq!(r.0, Rectangle3D::new(1, 2, -128, 4, 5, 127));
        // Kept for existing config files: the start/end form never counts as having a z, so its z values are replaced by -128..127
        let r: Rect =
            crate::json::from_str(r#"{"start":[1,2,3],"end":{"x":4,"y":5,"z":6}}"#).unwrap();
        assert_eq!(
            r.0,
            Rectangle3D::from_points(Point3D::new(1, 2, -128), Point3D::new(4, 5, 127))
        );
        assert!(crate::json::from_str::<Rect>(r#"{"x":1,"x1":2}"#).is_err());
        assert!(crate::json::from_str::<Rect>(r#"{"nope":1}"#).is_err());
        assert!(crate::json::from_str::<Rect>(r#"[1,2,3,4,5,6,7]"#).is_err());
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Shapes {
        #[serde(with = "point3d")]
        p3: Point3D,
        #[serde(with = "point2d")]
        p2: Point2D,
        #[serde(with = "timespan")]
        t: TimeSpan,
        #[serde(
            with = "client_version::option",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        v: Option<ClientVersion>,
        #[serde(with = "ip_endpoint")]
        ep: SocketAddr,
        #[serde(with = "guid")]
        g: Guid,
        #[serde(with = "text_definition")]
        td: TextDefinition,
    }

    #[test]
    fn shapes_round_trip() {
        let s = Shapes {
            p3: Point3D::new(1, -2, 3),
            p2: Point2D::new(4, 5),
            t: TimeSpan::from_mins(5),
            v: Some(ClientVersion::parse("7.0.9.0")),
            ep: "0.0.0.0:2593".parse().unwrap(),
            g: guid::parse_guid("00112233-4455-6677-8899-aabbccddeeff").unwrap(),
            td: TextDefinition::of_number(1042),
        };
        let json = crate::json::to_string(&s).unwrap();
        assert!(json.contains(r#""t": "00:05:00""#) && json.contains(r#""ep": "0.0.0.0:2593""#));
        assert!(
            json.contains("00112233-4455-6677-8899-aabbccddeeff") && json.contains("\"td\": 1042")
        );
        assert_eq!(crate::json::from_str::<Shapes>(&json).unwrap(), s);

        let p: Shapes = crate::json::from_str(
            r#"{"p3":[1,2],"p2":{"y":3},"t":"1.01:00:00","ep":"[::1]:5","g":"00112233445566778899AABBCCDDEEFF","td":"hi"}"#,
        )
        .unwrap();
        assert_eq!((p.p3, p.p2), (Point3D::new(1, 2, 0), Point2D::new(0, 3)));
        assert_eq!(p.t, TimeSpan::from_hours(25));
        assert!(p.v.is_none());
        assert_eq!(p.td.string.as_deref(), Some("hi"));
        assert!(crate::json::from_str::<Shapes>(r#"{"p3":[1,2,3,4]}"#).is_err());
    }

    bitflags::bitflags! {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        struct F: u32 {
            const A = 1;
            const B = 2;
            const ALIAS_A = 1;
            const BOTH = 3;
        }
    }
    json_flags!(F, [("A", 1), ("ALIAS_A", 1), ("B", 2), ("BOTH", 3)]);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Holder {
        #[serde(with = "flags")]
        f: F,
    }

    #[test]
    fn flags_write_single_bits_and_read_names() {
        let json = crate::json::to_string(&Holder { f: F::A }).unwrap();
        assert!(
            json.contains("\"A\": true")
                && json.contains("\"ALIAS_A\": true")
                && json.contains("\"B\": false")
        );
        assert!(!json.contains("BOTH"));
        let h: Holder = crate::json::from_str(r#"{"f":{"A":false,"B":true,"Nope":true}}"#).unwrap();
        assert_eq!(h.f, F::B);
        assert!(crate::json::from_str::<Holder>(r#"{"f":[]}"#).is_err());
    }

    #[test]
    fn refs() {
        assert_eq!(
            serde_json::from_str::<MapRef>("\"Felucca\"").unwrap(),
            MapRef::Name("Felucca".into())
        );
        assert_eq!(
            serde_json::from_str::<MapRef>("3").unwrap(),
            MapRef::Index(3)
        );
        let r: RegionRef = serde_json::from_str(r#"{"NAME":"Britain","map":"Felucca"}"#).unwrap();
        assert_eq!(
            r,
            RegionRef {
                name: "Britain".into(),
                map: "Felucca".into()
            }
        );
        assert!(serde_json::from_str::<RegionRef>(r#"{"name":"x"}"#).is_err());
    }

    #[test]
    fn bit_array() {
        let names = [("Zero", 0usize), ("Two", 2)];
        let v = bit_array::to_value(&names, &[true, false, true]);
        assert_eq!(v, serde_json::json!({"Zero": true, "Two": true}));
        let bits =
            bit_array::from_value::<serde_json::Error>(&names, &serde_json::json!({"Two": true}))
                .unwrap();
        assert_eq!(bits, vec![false, false, true]);
    }
}
