//! Hex string conversion.

use crate::error::{IoError, Result};

/// `ToHexString`: upper-case, no separator.
pub fn to_hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

/// `ToSpacedHexString`: `"01 AB FF"`.
pub fn to_spaced_hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// `ToDelimitedHexString`: `"[01, AB, FF]"`; empty input gives `"[]"`.
pub fn to_delimited_hex_string(bytes: &[u8]) -> String {
    format!(
        "[{}]",
        bytes
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// `GetBytes`. An odd length or a non-hex character is an error. Lower-case digits are accepted.
pub fn from_hex_string(s: &str) -> Result<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return Err(IoError::InvalidData("odd hex string length".into()));
    }
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| {
            let hi = (p[0] as char).to_digit(16);
            let lo = (p[1] as char).to_digit(16);
            match (hi, lo) {
                (Some(h), Some(l)) => Ok((h * 16 + l) as u8),
                _ => Err(IoError::InvalidData("non-hex digit".into())),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        assert_eq!(to_hex_string(&[0, 0xAB, 0xFF]), "00ABFF");
        assert_eq!(to_spaced_hex_string(&[0, 0xAB, 0xFF]), "00 AB FF");
        assert_eq!(to_spaced_hex_string(&[]), "");
        assert_eq!(to_delimited_hex_string(&[1, 2]), "[01, 02]");
        assert_eq!(to_delimited_hex_string(&[]), "[]");
    }

    #[test]
    fn parses() {
        assert_eq!(from_hex_string("00abFF").unwrap(), [0, 0xAB, 0xFF]);
        assert!(from_hex_string("0").is_err());
        assert!(from_hex_string("zz").is_err());
    }
}
