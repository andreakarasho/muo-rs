//! Client versions, client types and `ClientFlags`.
//!
//! Lives here because the transport derives `ProtocolChanges` and the login keys from it.

use crate::protocol_changes::ProtocolChanges;
use bitflags::bitflags;
use muo_primitives::dotnet::InvariantCase;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum ClientType {
    None = 0,
    #[default]
    Classic = 1,
    Uotd = 2,
    Kr = 4,
    Sa = 8,
}

impl ClientType {
    /// The display name of the client type.
    pub fn type_name(self) -> &'static str {
        match self {
            ClientType::Uotd => "UO:TD",
            ClientType::Kr => "UO:KR",
            ClientType::Sa => "UO:SA",
            _ => "classic",
        }
    }
}

bitflags! {
    /// Flags the client sends at character login (`ClientFlags`).
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct ClientFlags: u32 {
        const FELUCCA = 0x01;
        const TRAMMEL = 0x02;
        const ILSHENAR = 0x04;
        const MALAS = 0x08;
        const TOKUNO = 0x10;
        const TER_MUR = 0x20;
        const KR = 0x40;
        const UNK2 = 0x80;
        const UOTD = 0x100;
    }
}

#[derive(Clone, Debug)]
pub struct ClientVersion {
    pub major: i32,
    pub minor: i32,
    pub revision: i32,
    pub patch: i32,
    pub kind: ClientType,
    source: String,
}

// Equality includes the type; ordering deliberately does not (see `compare_to`).
impl PartialEq for ClientVersion {
    fn eq(&self, o: &Self) -> bool {
        self.major == o.major
            && self.minor == o.minor
            && self.revision == o.revision
            && self.patch == o.patch
            && self.kind == o.kind
    }
}
impl Eq for ClientVersion {}

impl Hash for ClientVersion {
    fn hash<H: Hasher>(&self, h: &mut H) {
        (self.major, self.minor, self.revision, self.patch).hash(h);
    }
}

impl std::fmt::Display for ClientVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.source)
    }
}

/// `Utility.ToInt32`: `0x` prefix is hex, anything unparsable is 0.
fn to_int32(s: &str) -> i32 {
    if let Some(hex) = s.strip_prefix("0x") {
        return u32::from_str_radix(hex.trim(), 16)
            .map(|v| v as i32)
            .unwrap_or(0);
    }
    s.trim().parse::<i32>().unwrap_or(0)
}

macro_rules! known {
    ($($name:ident = $s:literal),* $(,)?) => {
        impl ClientVersion {
            $(pub fn $name() -> ClientVersion { ClientVersion::parse($s) })*
        }
    };
}

known! {
    v500a = "5.0.0a", v502b = "5.0.2b", v6000 = "6.0.0.0",
    v6000_kr = "66.55.38", // KR 2.44.0.15 (First release)
    v6017 = "6.0.1.7", v6050 = "6.0.5.0", v60142 = "6.0.14.2",
    v60142_kr = "66.55.53", // KR 2.59.0.2
    v7000 = "7.0.0.0", v7090 = "7.0.9.0",
    v70120 = "7.0.12.0", // Plant localization change
    v70130 = "7.0.13.0", v70160 = "7.0.16.0", v70300 = "7.0.30.0", v70331 = "7.0.33.1",
    v704565 = "7.0.45.65", v70500 = "7.0.50.0", v70610 = "7.0.61.0",
    v70654 = "7.0.65.4", // Insufficient mana change
}

impl ClientVersion {
    pub fn new(maj: i32, min: i32, rev: i32, pat: i32, kind: ClientType) -> Self {
        let (major, kind) = if maj >= 67 {
            (maj - 60, ClientType::Sa)
        } else {
            (maj, if maj == 66 { ClientType::Kr } else { kind })
        };

        let mut v = Self {
            major,
            minor: min,
            revision: rev,
            patch: pat,
            kind,
            source: String::new(),
        };
        v.source = v.to_string_impl();
        v
    }

    /// `new ClientVersion(string)`. A malformed string gives 0.0.0.0 classic but keeps the text.
    pub fn parse(fmt: &str) -> Self {
        let fmt = fmt.lower_inv();
        let mut v = Self {
            major: 0,
            minor: 0,
            revision: 0,
            patch: 0,
            kind: ClientType::Classic,
            source: fmt.clone(),
        };
        let Some((major, minor, revision, patch, kind)) = Self::parse_parts(&fmt) else {
            return v;
        };
        v.major = major;
        v.minor = minor;
        v.revision = revision;
        v.patch = patch;
        v.kind = kind;
        v
    }

    fn parse_parts(fmt: &str) -> Option<(i32, i32, i32, i32, ClientType)> {
        let c: Vec<char> = fmt.chars().collect();
        let sub = |a: usize, b: usize| -> String { c[a..b].iter().collect() };

        let br1 = c.iter().position(|&x| x == '.')?;
        let br2 = br1 + 1 + c[br1 + 1..].iter().position(|&x| x == '.')?;

        let mut br3 = br2 + 1;
        while br3 < c.len() && c[br3].is_ascii_digit() {
            br3 += 1;
        }

        let mut major = to_int32(&sub(0, br1));
        let minor = to_int32(&sub(br1 + 1, br2));
        let revision = to_int32(&sub(br2 + 1, br3));
        let mut patch = 0;
        let mut kind = ClientType::Classic;

        if br3 < c.len() {
            if major <= 5 && minor <= 0 && revision <= 6 {
                // Anything before 5.0.7
                if !c[br3].is_whitespace() {
                    patch = c[br3] as i32 - 'a' as i32;
                }
            } else {
                patch = to_int32(&sub(br3 + 1, c.len()));
            }
        }

        if major == 66 {
            kind = ClientType::Kr;
        } else if major > 66 {
            major -= 60;
            kind = ClientType::Sa;
        } else if ["third dawn", "uo:td", "uotd", "uo3d", "uo:3d"]
            .iter()
            .any(|s| fmt.contains(s))
        {
            kind = ClientType::Uotd;
        }

        Some((major, minor, revision, patch, kind))
    }

    fn to_string_impl(&self) -> String {
        let (maj, min, rev, pat) = (self.major, self.minor, self.revision, self.patch);
        let mut s = if self.kind == ClientType::Sa {
            format!("{:02}.{:02}.{:02}", maj + 60, min, rev)
        } else if maj > 5 || min > 0 || rev > 6 {
            format!("{maj}.{min}.{rev}.{pat}")
        } else if pat > 0 {
            let letter = u32::try_from('a' as i32 + pat - 1)
                .ok()
                .and_then(char::from_u32)
                .unwrap_or('?');
            format!("{maj}.{min}.{rev}{letter}")
        } else {
            format!("{maj}.{min}.{rev}")
        };

        if self.kind == ClientType::Uotd {
            s.push_str(" uotd");
        }
        s
    }

    pub fn source_string(&self) -> &str {
        &self.source
    }

    pub fn compare_to(&self, o: &ClientVersion) -> Ordering {
        self.major
            .cmp(&o.major)
            .then(self.minor.cmp(&o.minor))
            .then(self.revision.cmp(&o.revision))
            .then_with(|| {
                // Patch is not tested for EC since it is always 0 but compatible with classic non-zero
                if self.kind == ClientType::Sa || o.kind == ClientType::Sa {
                    Ordering::Equal
                } else {
                    self.patch.cmp(&o.patch)
                }
            })
    }

    /// `ClientVersion.Compare(a, b)`: null sorts first.
    pub fn compare(a: Option<&ClientVersion>, b: Option<&ClientVersion>) -> Ordering {
        match (a, b) {
            (None, None) => Ordering::Equal,
            (None, _) => Ordering::Less,
            (_, None) => Ordering::Greater,
            (Some(a), Some(b)) => a.compare_to(b),
        }
    }

    /// `self >= o` by `compare_to`.
    pub fn ge(&self, o: &ClientVersion) -> bool {
        self.compare_to(o) != Ordering::Less
    }

    pub fn protocol_changes(&self) -> ProtocolChanges {
        if self.kind == ClientType::Kr {
            return if self.ge(&Self::v60142_kr()) {
                ProtocolChanges::VERSION_60142
            } else {
                ProtocolChanges::VERSION_6000
            };
        }

        let table = [
            (Self::v70610(), ProtocolChanges::VERSION_70610),
            (Self::v70500(), ProtocolChanges::VERSION_70500),
            (Self::v704565(), ProtocolChanges::VERSION_704565),
            (Self::v70331(), ProtocolChanges::VERSION_70331),
            (Self::v70300(), ProtocolChanges::VERSION_70300),
            (Self::v70160(), ProtocolChanges::VERSION_70160),
            (Self::v70130(), ProtocolChanges::VERSION_70130),
            (Self::v7090(), ProtocolChanges::VERSION_7090),
            (Self::v7000(), ProtocolChanges::VERSION_7000),
            (Self::v60142(), ProtocolChanges::VERSION_60142),
            (Self::v6017(), ProtocolChanges::VERSION_6017),
            (Self::v6000(), ProtocolChanges::VERSION_6000),
            (Self::v502b(), ProtocolChanges::VERSION_502B),
        ];
        table
            .into_iter()
            .find(|(v, _)| self.ge(v))
            // We do not support versions lower than 5.0.0a
            .map_or(ProtocolChanges::VERSION_500A, |(_, p)| p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cc_and_ec_string_ctor() {
        for (cc, ec, equal, cmp) in [
            ("7.0.33.1", "67.0.33", false, Ordering::Equal),
            ("7.0.45.65", "67.0.33", false, Ordering::Greater),
            ("6.0.0.0", "67.0.33", false, Ordering::Less),
            ("7.0.33.1", "7.0.33.1", true, Ordering::Equal),
        ] {
            let (cc, ec) = (ClientVersion::parse(cc), ClientVersion::parse(ec));
            assert_eq!(cc == ec, equal);
            assert_eq!(cc.compare_to(&ec), cmp);
        }
    }

    #[test]
    fn cc_and_ec_ctor() {
        for (a, b, equal, cmp) in [
            ((7, 0, 33, 1), (67, 0, 33, 0), false, Ordering::Equal),
            ((7, 0, 45, 65), (67, 0, 33, 0), false, Ordering::Greater),
            ((6, 0, 0, 0), (67, 0, 33, 0), false, Ordering::Less),
            ((7, 0, 33, 1), (7, 0, 33, 1), true, Ordering::Equal),
        ] {
            let cc = ClientVersion::new(a.0, a.1, a.2, a.3, ClientType::Classic);
            let ec = ClientVersion::new(b.0, b.1, b.2, b.3, ClientType::Classic);
            assert_eq!(cc == ec, equal);
            assert_eq!(cc.compare_to(&ec), cmp);
        }
    }

    #[test]
    fn parse_details_and_strings() {
        let v = ClientVersion::parse("5.0.2b");
        assert_eq!((v.major, v.minor, v.revision, v.patch), (5, 0, 2, 1));
        assert_eq!(
            ClientVersion::new(5, 0, 2, 2, ClientType::Classic).to_string(),
            "5.0.2b"
        );
        assert_eq!(
            ClientVersion::new(67, 0, 33, 0, ClientType::Classic).to_string(),
            "67.00.33"
        );
        assert_eq!(ClientVersion::parse("7.0.0.0 UOTD").kind, ClientType::Uotd);
        assert_eq!(ClientVersion::parse("66.55.38").kind, ClientType::Kr);
        // Malformed text gives zeros but keeps the source
        let bad = ClientVersion::parse("garbage");
        assert_eq!((bad.major, bad.kind), (0, ClientType::Classic));
        assert_eq!(bad.source_string(), "garbage");
    }

    #[test]
    fn protocol_changes_by_version() {
        let pc = |s: &str| ClientVersion::parse(s).protocol_changes();
        assert_eq!(pc("4.0.11"), ProtocolChanges::VERSION_500A);
        assert_eq!(pc("6.0.0.0"), ProtocolChanges::VERSION_6000);
        assert_eq!(pc("7.0.9.0"), ProtocolChanges::VERSION_7090);
        assert_eq!(pc("7.0.65.4"), ProtocolChanges::VERSION_70610);
        assert_eq!(pc("66.55.38"), ProtocolChanges::VERSION_6000);
        assert_eq!(pc("66.55.53"), ProtocolChanges::VERSION_60142);
    }
}
