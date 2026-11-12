//! The wire encodings the server uses, with replacement behaviour: unencodable characters become `?`
//! (ASCII, Latin1), undecodable bytes become `?` (ASCII) or U+FFFD (UTF-8, UTF-16).
//! "Safe" decoding also drops what the UO client renders as nothing: C0, DEL, C1 and U+FFFE/U+FFFF.
//!
//! shortcut: Rust strings are UTF-8, so a lone UTF-16 surrogate (which `LittleUni` would keep) decodes to U+FFFD;
//! upgrade if a save or packet is ever found carrying one.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TextEncoding {
    Ascii,
    Latin1,
    Utf8,
    /// UTF-16 big endian (`TextEncoding.Unicode`).
    BigUni,
    /// UTF-16 little endian (`TextEncoding.UnicodeLE`).
    LittleUni,
}

impl TextEncoding {
    /// `GetByteLengthForEncoding`: the size of one terminator / fixed-length unit.
    pub fn unit_size(self) -> usize {
        match self {
            Self::BigUni | Self::LittleUni => 2,
            _ => 1,
        }
    }

    /// `GetBytes`.
    pub fn encode(self, s: &str) -> Vec<u8> {
        match self {
            Self::Ascii => s
                .chars()
                .map(|c| if c.is_ascii() { c as u8 } else { b'?' })
                .collect(),
            Self::Latin1 => s
                .chars()
                .map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?'))
                .collect(),
            Self::Utf8 => s.as_bytes().to_vec(),
            Self::BigUni => s.encode_utf16().flat_map(u16::to_be_bytes).collect(),
            Self::LittleUni => s.encode_utf16().flat_map(u16::to_le_bytes).collect(),
        }
    }

    /// `GetString*(bytes, safeString)`.
    pub fn decode(self, bytes: &[u8], safe: bool) -> String {
        let s = match self {
            Self::Ascii => bytes
                .iter()
                .filter(|&&b| !(safe && (b < 0x20 || b == 0x7F)))
                .map(|&b| if b < 0x80 { b as char } else { '?' })
                .collect::<String>(),
            Self::Latin1 => bytes
                .iter()
                .filter(|&&b| !(safe && (b < 0x20 || (0x80..=0x9F).contains(&b))))
                .map(|&b| b as char)
                .collect::<String>(),
            Self::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
            Self::BigUni => decode_utf16(bytes, u16::from_be_bytes),
            Self::LittleUni => decode_utf16(bytes, u16::from_le_bytes),
        };

        if safe && matches!(self, Self::Utf8 | Self::BigUni | Self::LittleUni) {
            s.chars().filter(|&c| !is_invalid_unicode_char(c)).collect()
        } else {
            s
        }
    }
}

fn decode_utf16(bytes: &[u8], f: fn([u8; 2]) -> u16) -> String {
    let units = bytes.as_chunks::<2>().0.iter().map(|c| f(*c));
    let mut s: String = char::decode_utf16(units)
        .map(|r| r.unwrap_or('\u{FFFD}'))
        .collect();
    if bytes.len() % 2 == 1 && !bytes.is_empty() {
        // UnicodeEncoding.GetString replaces a dangling byte; the LE reader (a span cast) just ignores it.
        if f([1, 0]) == 0x0100 {
            // big endian
            s.push('\u{FFFD}');
        }
    }
    s
}

/// C0 (0x00-0x1F), DEL + C1 (0x7F-0x9F), non-characters (0xFFFE-0xFFFF).
fn is_invalid_unicode_char(c: char) -> bool {
    matches!(c, '\u{0}'..='\u{1F}' | '\u{7F}'..='\u{9F}' | '\u{FFFE}' | '\u{FFFF}')
}

/// A string as UTF-16 code units (UTF-16 string indexing, used by fixed-length writes).
pub fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// The first `n` UTF-16 code units of `s`; a cut surrogate pair becomes U+FFFD.
pub fn truncate_utf16(s: &str, n: usize) -> String {
    let units: Vec<u16> = s.encode_utf16().take(n).collect();
    String::from_utf16_lossy(&units)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_replaces_high_bytes() {
        assert_eq!(
            TextEncoding::Ascii.decode(&[b'H', b'e', 0xFF, b'l', b'o'], true),
            "He?lo"
        );
        assert_eq!(
            TextEncoding::Ascii.decode(&[b'H', 0, 0x7F, b'i'], true),
            "Hi"
        );
        assert_eq!(
            TextEncoding::Ascii.decode(&[b'H', 0x01, b'i'], false),
            "H\u{1}i"
        );
    }

    #[test]
    fn latin1_safe_drops_c0_c1_but_keeps_del() {
        assert_eq!(
            TextEncoding::Latin1.decode(&[b'a', 0x85, 0x7F, 0xE9, 0x01], true),
            "a\u{7F}\u{E9}"
        );
        assert_eq!(
            TextEncoding::Latin1.encode("a\u{E9}\u{263A}"),
            [b'a', 0xE9, b'?']
        );
    }

    #[test]
    fn utf8_safe_drops_nonchars() {
        assert_eq!(
            TextEncoding::Utf8.decode("a\u{FFFE}\u{85}b\u{FFFD}".as_bytes(), true),
            "ab\u{FFFD}"
        );
        assert_eq!(TextEncoding::Utf8.decode(&[b'a', 0xC3], false), "a\u{FFFD}");
    }

    #[test]
    fn utf16_round_trip_with_pairs() {
        let s = "a\u{1F600}b";
        for e in [TextEncoding::BigUni, TextEncoding::LittleUni] {
            assert_eq!(e.decode(&e.encode(s), false), s);
        }
        assert_eq!(TextEncoding::BigUni.encode("Hi"), [0, b'H', 0, b'i']);
        assert_eq!(TextEncoding::LittleUni.encode("Hi"), [b'H', 0, b'i', 0]);
        assert_eq!(truncate_utf16("a\u{1F600}", 2), "a\u{FFFD}");
        assert_eq!(utf16_len("a\u{1F600}"), 3);
    }
}
