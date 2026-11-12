//! Number parsing (`ToInt32`, `ToUInt32`, ...).
//!
//! A `0x` prefix selects hex, and surrounding white space is allowed (`NumberStyles.Integer`).

use std::fmt;

/// `FormatException` for the `Parse` methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatError;

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("The input string was not in a correct format.")
    }
}

impl std::error::Error for FormatError {}

fn is_net_white(c: char) -> bool {
    // NumberStyles.AllowLeadingWhite / AllowTrailingWhite: U+0009..U+000D and U+0020
    matches!(c, '\u{9}'..='\u{D}' | ' ')
}

fn trim_white(s: &str) -> &str {
    s.trim_matches(is_net_white)
}

fn strip_hex_prefix(s: &str, ignore_case: bool) -> Option<&str> {
    let head = s.get(..2)?;
    if head == "0x" || (ignore_case && head.eq_ignore_ascii_case("0x")) {
        Some(&s[2..])
    } else {
        None
    }
}

fn parse_hex(s: &str) -> Option<u32> {
    let s = trim_white(s);
    if s.is_empty() {
        return None;
    }
    let mut v: u64 = 0;
    for c in s.chars() {
        v = (v << 4) | u64::from(c.to_digit(16)?);
        if v > u64::from(u32::MAX) {
            return None;
        }
    }
    Some(v as u32)
}

fn parse_dec(s: &str) -> Option<(bool, u64)> {
    let s = trim_white(s);
    let (neg, digits) = match s.as_bytes().first()? {
        b'-' => (true, &s[1..]),
        b'+' => (false, &s[1..]),
        _ => (false, s),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut v: u64 = 0;
    for b in digits.bytes() {
        v = v.checked_mul(10)?.checked_add(u64::from(b - b'0'))?;
    }
    Some((neg, v))
}

/// `Utility.ToInt32(span, out i)`. Hex that fits 32 bits wraps to negative (`0xFFFFFFFF` is -1).
pub fn try_to_i32(s: &str) -> Option<i32> {
    if let Some(h) = strip_hex_prefix(s, true) {
        return parse_hex(h).map(|v| v as i32);
    }
    let (neg, v) = parse_dec(s)?;
    if neg {
        i32::try_from(-(v as i128)).ok()
    } else {
        i32::try_from(v).ok()
    }
}

/// `Utility.ToUInt32(span, out i)`.
pub fn try_to_u32(s: &str) -> Option<u32> {
    if let Some(h) = strip_hex_prefix(s, true) {
        return parse_hex(h);
    }
    let (neg, v) = parse_dec(s)?;
    if neg && v != 0 {
        return None;
    }
    u32::try_from(v).ok()
}

/// `Utility.ToInt32(span)`: 0 when it doesn't parse. Only a lowercase `0x` is hex.
pub fn to_i32(s: &str) -> i32 {
    if let Some(h) = s.strip_prefix("0x") {
        return parse_hex(h).map_or(0, |v| v as i32);
    }
    match parse_dec(s) {
        Some((true, v)) => i32::try_from(-(v as i128)).unwrap_or(0),
        Some((false, v)) => i32::try_from(v).unwrap_or(0),
        None => 0,
    }
}

/// `Utility.ToUInt32(span)`: 0 when it doesn't parse.
pub fn to_u32(s: &str) -> u32 {
    try_to_u32(s).unwrap_or(0)
}

/// `Utility.ToBoolean`: `bool.TryParse` (case-insensitive, trimmed), else "enabled" or "on".
pub fn to_bool(s: &str) -> bool {
    let t = trim_white(s);
    if t.eq_ignore_ascii_case("true") {
        true
    } else if t.eq_ignore_ascii_case("false") {
        false
    } else {
        s.eq_ignore_ascii_case("enabled") || s.eq_ignore_ascii_case("on")
    }
}

/// `Utility.ToDouble`: 0 when it doesn't parse. `double.TryParse` allows thousands separators in the integer part
/// (`1,000`) and the words `Infinity` and `NaN` but not Rust's `inf`.
pub fn to_f64(s: &str) -> f64 {
    let t = trim_white(s);
    let body = t.trim_start_matches(['+', '-']);
    if body.starts_with(|c: char| c.is_ascii_alphabetic())
        && !body.eq_ignore_ascii_case("infinity")
        && !body.eq_ignore_ascii_case("nan")
    {
        return 0.0;
    }
    let end = t.find(['.', 'e', 'E']).unwrap_or(t.len());
    let (int, rest) = t.split_at(end);
    let ok_commas = !int.contains(',')
        || int
            .char_indices()
            .all(|(i, c)| c != ',' || int[..i].ends_with(|p: char| p.is_ascii_digit()));
    if !ok_commas {
        return 0.0;
    }
    format!("{}{rest}", int.replace(',', ""))
        .parse()
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ints() {
        assert_eq!(try_to_i32("101"), Some(101));
        assert_eq!(try_to_i32(" -2 "), Some(-2));
        assert_eq!(try_to_i32("+7"), Some(7));
        assert_eq!(try_to_i32("0x1F"), Some(31));
        assert_eq!(try_to_i32("0XFFFFFFFF"), Some(-1));
        assert_eq!(try_to_i32(""), None);
        assert_eq!(try_to_i32("|"), None);
        assert_eq!(try_to_i32("2147483648"), None);
        assert_eq!(try_to_i32("-2147483648"), Some(i32::MIN));
        assert_eq!(try_to_u32("4294967295"), Some(u32::MAX));
        assert_eq!(try_to_u32("0xABCD1234"), Some(0xABCD1234));
        assert_eq!(try_to_u32("-1"), None);
        assert_eq!(try_to_u32("0x"), None);
        assert_eq!(to_i32("0X10"), 0); // the non-`out` overload is ordinal
        assert_eq!(to_f64(" 1,000.5 "), 1000.5);
        assert_eq!(
            (to_f64("inf"), to_f64(",5"), to_f64("1e2")),
            (0.0, 0.0, 100.0)
        );
        assert_eq!(to_f64("-Infinity"), f64::NEG_INFINITY);
        assert!(to_bool("On") && to_bool("TRUE") && !to_bool("no"));
    }
}
