//! `Types`: how `[set` and the property gumps turn text into a property value.
//!
//! A property's type is described by [`PropType`] (the command and
//! props modules build one per registered property), and the result is a [`Parsed`] value. Types that were
//! reflected as "has a static `Parse`" (`Faction`, `Town`, `TimeSpan`...) are `PropType::Parsable`, carrying the
//! parse function the owning crate registered.
//!
//! `Entity` needs `World.FindEntity`, so the parser takes a `find_entity` callback; `Type` needs the
//! `TypeRegistry` and is looked up by the same callback style (`find_type`).

use muo_primitives::dotnet::IgnoreCase;
use muo_primitives::{Point2D, Point3D, Serial};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Numeric {
    Byte,
    SByte,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
}

/// The kinds of property a command can set.
#[derive(Clone)]
pub enum PropType {
    Numeric(Numeric),
    Float,
    Double,
    Decimal,
    Bool,
    Char,
    Str,
    /// `Serial`.
    Serial,
    /// An `IEntity` (an item or mobile, given as its serial).
    Entity,
    /// An `IEntity` property declared as the class `.0` (short class name): what `SetObjectTarget` checks with `IsInstanceOfType`.
    EntityOf(&'static str),
    /// An enum: its member names with values (case-insensitive parse; a number or comma-separated names for flags).
    Enum(&'static [(&'static str, i64)]),
    /// `Type`: a class looked up by name.
    Type,
    IPoint3D,
    IPoint2D,
    /// A type with a static `Parse`: the function (`None` result is a parse failure).
    Parsable(fn(&str) -> Option<Parsed>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Parsed {
    Null,
    Int(i64),
    UInt(u64),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(String),
    Serial(Serial),
    /// An entity found by serial; `None` when no such entity is in the world.
    Entity(Option<Serial>),
    Enum(i64),
    /// A class full name.
    Type(String),
    Point3D(Point3D),
    Point2D(Point2D),
}

/// What the parser needs from the world.
pub struct ParseContext<'a> {
    pub find_entity: &'a dyn Fn(Serial) -> bool,
    pub find_type: &'a dyn Fn(&str) -> Option<String>,
}

/// `TryGetQuotedLiteral`: `@"..."` is the literal text inside, for values the bare text would be read as something
/// else.
fn quoted_literal(value: &str) -> Option<&str> {
    if value.len() >= 3 && value.starts_with("@\"") && value.ends_with('"') {
        Some(&value[2..value.len() - 1])
    } else {
        None
    }
}

fn parse_numeric(kind: Numeric, text: &str, hex: bool) -> Option<Parsed> {
    macro_rules! p {
        ($t:ty, $wrap:expr) => {{
            let v = if hex {
                <$t>::from_str_radix(text, 16).ok()?
            } else {
                text.parse::<$t>().ok()?
            };
            Some($wrap(v))
        }};
    }
    match kind {
        Numeric::Byte => p!(u8, |v| Parsed::Int(i64::from(v))),
        Numeric::SByte => p!(i8, |v| Parsed::Int(i64::from(v))),
        Numeric::Short => p!(i16, |v| Parsed::Int(i64::from(v))),
        Numeric::UShort => p!(u16, |v| Parsed::Int(i64::from(v))),
        Numeric::Int => p!(i32, |v| Parsed::Int(i64::from(v))),
        Numeric::UInt => p!(u32, |v| Parsed::Int(i64::from(v))),
        Numeric::Long => p!(i64, Parsed::Int),
        Numeric::ULong => p!(u64, Parsed::UInt),
    }
}

/// `Types.TryParse(type, value, out constructed)`: `Err` is the message shown to the user.
pub fn try_parse(ty: &PropType, value: Option<&str>, ctx: &ParseContext) -> Result<Parsed, String> {
    let is_serial = matches!(ty, PropType::Serial);
    let is_entity = matches!(ty, PropType::Entity | PropType::EntityOf(_));
    // Serial and entity properties are mutated into int32.
    let ty = if is_serial || is_entity {
        &PropType::Numeric(Numeric::Int)
    } else {
        ty
    };

    let mut value = value;
    if value == Some("(-null-)")
        && !matches!(
            ty,
            PropType::Numeric(_)
                | PropType::Float
                | PropType::Double
                | PropType::Decimal
                | PropType::Bool
                | PropType::Char
        )
    {
        value = None;
    }

    if let PropType::Enum(members) = ty {
        let text = value.unwrap_or("");
        return parse_enum(members, text)
            .map(Parsed::Enum)
            .ok_or_else(|| "That is not a valid enumeration member.".to_string());
    }

    if matches!(ty, PropType::Type) {
        return match value.and_then(|v| (ctx.find_type)(v)) {
            Some(t) => Ok(Parsed::Type(t)),
            None => Err("No type with that name was found.".to_string()),
        };
    }

    let Some(value) = value else {
        return Ok(Parsed::Null);
    };

    match ty {
        PropType::Str => {
            // Decodes what InternalGetValue writes, so [get output pastes back into [set.
            return Ok(Parsed::Str(
                quoted_literal(value).unwrap_or(value).to_string(),
            ));
        }
        PropType::IPoint3D | PropType::IPoint2D => {
            if let Some(p) = Point3D::try_parse(value) {
                return Ok(Parsed::Point3D(p));
            }
            if matches!(ty, PropType::IPoint2D)
                && let Some(p) = Point2D::try_parse(value)
            {
                return Ok(Parsed::Point2D(p));
            }
            return Err("That is not properly formatted.".to_string());
        }
        PropType::Bool => {
            let t = value.trim();
            return if t.eq_ic("true") {
                Ok(Parsed::Bool(true))
            } else if t.eq_ic("false") {
                Ok(Parsed::Bool(false))
            } else {
                Err("Not a valid boolean string.".to_string())
            };
        }
        PropType::Numeric(kind) => {
            let (text, hex) = match value.strip_prefix("0x") {
                Some(rest) => (rest, true),
                None => (value, false),
            };
            if is_entity || is_serial {
                // A Serial is a uint: parse the full 32-bit range as ulong and resolve.
                if let Ok(num) = if hex {
                    u64::from_str_radix(text, 16)
                } else {
                    text.parse::<u64>()
                } {
                    let serial = Serial::new(num as u32);
                    return Ok(if is_entity {
                        Parsed::Entity((ctx.find_entity)(serial).then_some(serial))
                    } else {
                        Parsed::Serial(serial)
                    });
                }
            } else if let Some(p) = parse_numeric(*kind, text, hex) {
                // Parsed directly into the target type: signed and per-type ranges are honored.
                return Ok(p);
            }
            // Any other parse failure gets the same message.
            return Err("That is not properly formatted.".to_string());
        }
        PropType::Parsable(f) => {
            return f(value).ok_or_else(|| "That is not properly formatted.".to_string());
        }
        _ => {}
    }

    // Convert.ChangeType
    let converted = match ty {
        PropType::Float | PropType::Double | PropType::Decimal => {
            value.trim().parse::<f64>().ok().map(Parsed::Float)
        }
        PropType::Char => {
            let mut it = value.chars();
            match (it.next(), it.next()) {
                (Some(c), None) => Some(Parsed::Char(c)),
                _ => None,
            }
        }
        _ => None,
    };
    converted.ok_or_else(|| "That is not properly formatted.".to_string())
}

/// `Enum.Parse(type, value, ignoreCase: true)`: a number, a name, or names joined with commas (flags).
fn parse_enum(members: &[(&str, i64)], text: &str) -> Option<i64> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    if let Ok(n) = t.parse::<i64>() {
        return Some(n);
    }
    let mut bits = 0;
    for part in t.split(',') {
        let part = part.trim();
        bits |= members.iter().find(|(n, _)| n.eq_ic(part))?.1;
    }
    Some(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> ParseContext<'static> {
        ParseContext {
            find_entity: &|s| s.value() == 0x1000,
            find_type: &|n| (n == "Gold").then(|| "Server.Items.Gold".to_string()),
        }
    }

    fn num(kind: Numeric, v: &str) -> Result<Parsed, String> {
        try_parse(&PropType::Numeric(kind), Some(v), &ctx())
    }

    #[test]
    fn numeric_parses_directly_into_the_target_type() {
        assert_eq!(num(Numeric::Int, "42"), Ok(Parsed::Int(42)));
        assert_eq!(num(Numeric::Int, "-5"), Ok(Parsed::Int(-5))); // signed values parse directly
        assert_eq!(num(Numeric::Int, "0xFF"), Ok(Parsed::Int(255))); // hex
        assert!(num(Numeric::Int, "notanumber").is_err());
        assert_eq!(num(Numeric::Byte, "255"), Ok(Parsed::Int(255)));
        assert!(num(Numeric::Byte, "256").is_err()); // out of the byte range
        assert_eq!(
            num(Numeric::UInt, "4294967295"),
            Ok(Parsed::Int(4_294_967_295))
        );
        assert_eq!(
            num(Numeric::Long, "-9000000000"),
            Ok(Parsed::Int(-9_000_000_000))
        );
    }

    #[test]
    fn interface_points() {
        for (ty, text) in [
            (PropType::IPoint3D, "(1, 2, 3)"),
            (PropType::IPoint2D, "(1, 2, 3)"),
        ] {
            assert_eq!(
                try_parse(&ty, Some(text), &ctx()),
                Ok(Parsed::Point3D(Point3D::new(1, 2, 3)))
            );
        }
        assert_eq!(
            try_parse(&PropType::IPoint2D, Some("(4, 5)"), &ctx()),
            Ok(Parsed::Point2D(Point2D::new(4, 5)))
        );
        assert!(try_parse(&PropType::IPoint3D, Some("(4, 5)"), &ctx()).is_err());
        assert_eq!(
            try_parse(&PropType::IPoint2D, Some("(-null-)"), &ctx()),
            Ok(Parsed::Null)
        );
    }

    #[test]
    fn strings_bools_enums_types_and_entities() {
        let c = ctx();
        assert_eq!(
            try_parse(&PropType::Str, Some("@\"0x10\""), &c),
            Ok(Parsed::Str("0x10".into()))
        );
        assert_eq!(
            try_parse(&PropType::Str, Some("plain"), &c),
            Ok(Parsed::Str("plain".into()))
        );
        assert_eq!(
            try_parse(&PropType::Str, Some("(-null-)"), &c),
            Ok(Parsed::Null)
        );
        assert_eq!(
            try_parse(&PropType::Bool, Some("True"), &c),
            Ok(Parsed::Bool(true))
        );
        assert_eq!(
            try_parse(&PropType::Bool, Some("maybe"), &c),
            Err("Not a valid boolean string.".into())
        );
        const E: &[(&str, i64)] = &[("A", 1), ("B", 2)];
        assert_eq!(
            try_parse(&PropType::Enum(E), Some("a, B"), &c),
            Ok(Parsed::Enum(3))
        );
        assert!(try_parse(&PropType::Enum(E), Some("C"), &c).is_err());
        assert_eq!(
            try_parse(&PropType::Type, Some("Gold"), &c),
            Ok(Parsed::Type("Server.Items.Gold".into()))
        );
        assert_eq!(
            try_parse(&PropType::Type, Some("Nope"), &c),
            Err("No type with that name was found.".into())
        );
        assert_eq!(
            try_parse(&PropType::Entity, Some("0x1000"), &c),
            Ok(Parsed::Entity(Some(Serial::new(0x1000))))
        );
        assert_eq!(
            try_parse(&PropType::Entity, Some("5"), &c),
            Ok(Parsed::Entity(None))
        );
        assert_eq!(
            try_parse(&PropType::Serial, Some("0xFFFFFFFF"), &c),
            Ok(Parsed::Serial(Serial::new(0xFFFF_FFFF)))
        );
    }
}
