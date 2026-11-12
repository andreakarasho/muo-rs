//! Account password protection: `AccountSecurity`, the Argon2, PBKDF2 and digest password protections, and the
//! password worker.
//!
//! Hashes keep the stored format of existing account saves: Argon2id in the PHC string format
//! (`$argon2id$v=19$m=16384,t=1,p=1$..`, 16 byte salt, 32 byte hash), PBKDF2 as hex of
//! `iterations(u16 LE) + salt(8) + SHA-256 hash(32)`, and the obsolete MD5, SHA1 and SHA-512 digests as upper-case
//! hex of the first 256 ASCII characters.
//!
//! The off-loop hash thread is `password_worker.rs`.

use muo_primitives::dotnet::IgnoreCase;
use std::str::FromStr;

use base64_nopad as b64;
use muo_io::text::hex::to_hex_string;
use sha1::Digest as _;

/// `PasswordProtectionAlgorithm`. The numbers are written to saves.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(i32)]
pub enum PasswordProtectionAlgorithm {
    #[default]
    None = 0,
    Md5 = 1,
    Sha1 = 2,
    /// ServUO compatibility.
    Sha2 = 3,
    Pbkdf2 = 4,
    /// Recommended algorithm for real security.
    Argon2 = 5,
}

impl PasswordProtectionAlgorithm {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Md5,
            2 => Self::Sha1,
            3 => Self::Sha2,
            4 => Self::Pbkdf2,
            5 => Self::Argon2,
            _ => Self::None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Md5 => "MD5",
            Self::Sha1 => "SHA1",
            Self::Sha2 => "SHA2",
            Self::Pbkdf2 => "PBKDF2",
            Self::Argon2 => "Argon2",
        }
    }
}

impl FromStr for PasswordProtectionAlgorithm {
    type Err = ();

    /// `Enum.TryParse(text, ignoreCase: true)`.
    fn from_str(s: &str) -> Result<Self, ()> {
        let s = s.trim();
        if let Ok(n) = s.parse::<i32>() {
            return Ok(Self::from_i32(n));
        }
        [
            Self::None,
            Self::Md5,
            Self::Sha1,
            Self::Sha2,
            Self::Pbkdf2,
            Self::Argon2,
        ]
        .into_iter()
        .find(|a| a.name().eq_ic(s))
        .ok_or(())
    }
}

/// `AccountSecurity.DerivePhrase`: SHA1 and SHA2 salt by username; everything else hashes the password alone.
pub fn derive_phrase(
    algorithm: PasswordProtectionAlgorithm,
    username: &str,
    plain_password: &str,
) -> String {
    match algorithm {
        PasswordProtectionAlgorithm::Sha1 | PasswordProtectionAlgorithm::Sha2 => {
            format!("{username}{plain_password}")
        }
        _ => plain_password.to_owned(),
    }
}

/// `AccountSecurity.GetPasswordProtection(algorithm).EncryptPassword(plain)`. `None` is the refusal
/// "Do not use PasswordProtectionAlgorithm.None".
pub fn encrypt_password(algorithm: PasswordProtectionAlgorithm, plain: &str) -> Option<String> {
    Some(match algorithm {
        PasswordProtectionAlgorithm::None => return None,
        PasswordProtectionAlgorithm::Md5
        | PasswordProtectionAlgorithm::Sha1
        | PasswordProtectionAlgorithm::Sha2 => hash_digest(algorithm, plain),
        PasswordProtectionAlgorithm::Pbkdf2 => pbkdf2_encrypt(plain),
        PasswordProtectionAlgorithm::Argon2 => argon2_encrypt(plain),
    })
}

/// `IPasswordProtection.ValidatePassword(encrypted, plain)`.
pub fn validate_password(
    algorithm: PasswordProtectionAlgorithm,
    encrypted: &str,
    plain: &str,
) -> bool {
    match algorithm {
        PasswordProtectionAlgorithm::None => false,
        PasswordProtectionAlgorithm::Md5
        | PasswordProtectionAlgorithm::Sha1
        | PasswordProtectionAlgorithm::Sha2 => hash_digest(algorithm, plain) == encrypted,
        PasswordProtectionAlgorithm::Pbkdf2 => pbkdf2_validate(encrypted, plain),
        PasswordProtectionAlgorithm::Argon2 => argon2_validate(encrypted, plain),
    }
}

/// `IPasswordProtection.NeedsRehash(encrypted)`: only Argon2 embeds its cost.
pub fn needs_rehash(algorithm: PasswordProtectionAlgorithm, encrypted: &str) -> bool {
    match algorithm {
        PasswordProtectionAlgorithm::Argon2 => match parse_phc(encrypted) {
            // Unparseable but verified: a format this build does not understand, so rewrite it.
            None => true,
            Some(p) => {
                !p.id
                    || p.memory != ARGON_MEMORY
                    || p.time != ARGON_TIME
                    || p.parallel != ARGON_PARALLEL
                    || p.hash.len() != ARGON_HASH_LEN
                    || p.salt.len() != ARGON_SALT_LEN
            }
        },
        _ => false,
    }
}

/// `Encoding.ASCII.GetBytes` of at most the first 256 characters (a non-ASCII character becomes `?`).
fn ascii_bytes(s: &str) -> Vec<u8> {
    // UTF-16 units, like `Math.Min(256, plainPassword.Length)` and the ASCII encoder: a surrogate pair is two `?`.
    s.encode_utf16()
        .take(256)
        .map(|c| if c < 0x80 { c as u8 } else { b'?' })
        .collect()
}

fn hash_digest(algorithm: PasswordProtectionAlgorithm, plain: &str) -> String {
    let bytes = ascii_bytes(plain);
    match algorithm {
        PasswordProtectionAlgorithm::Md5 => to_hex_string(&muo_net::md5::md5(&bytes)),
        PasswordProtectionAlgorithm::Sha1 => to_hex_string(&sha1::Sha1::digest(&bytes)),
        _ => to_hex_string(&sha2::Sha512::digest(&bytes)),
    }
}

fn fill_random(buf: &mut [u8]) {
    // A failing OS random source must not leave the buffer zeroed and silently weaken the salt; refusing to hash
    // would lock everyone out, so this panics.
    getrandom::fill(buf).expect("the operating system's random number generator failed");
}

const PBKDF2_MIN_ITERATIONS: u32 = 1024;
const PBKDF2_MAX_ITERATIONS: u32 = 1536;
const PBKDF2_SALT_SIZE: usize = 8;
const PBKDF2_HASH_SIZE: usize = 32;
const PBKDF2_OUTPUT_SIZE: usize = 2 + PBKDF2_SALT_SIZE + PBKDF2_HASH_SIZE;

fn pbkdf2_encrypt(plain: &str) -> String {
    let mut out = [0u8; PBKDF2_OUTPUT_SIZE];
    let mut r = [0u8; 4];
    fill_random(&mut r);
    // RandomNumberGenerator.GetInt32(1024, 1537)
    let iterations = PBKDF2_MIN_ITERATIONS
        + u32::from_le_bytes(r) % (PBKDF2_MAX_ITERATIONS + 1 - PBKDF2_MIN_ITERATIONS);
    out[..2].copy_from_slice(&(iterations as u16).to_le_bytes());
    fill_random(&mut out[2..2 + PBKDF2_SALT_SIZE]);
    let (head, hash) = out.split_at_mut(2 + PBKDF2_SALT_SIZE);
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(plain.as_bytes(), &head[2..], iterations, hash);
    to_hex_string(&out)
}

fn pbkdf2_validate(encrypted: &str, plain: &str) -> bool {
    // `GetBytes` reads two upper-case hex digits per byte.
    let digits = encrypted.as_bytes();
    if digits.len() != PBKDF2_OUTPUT_SIZE * 2 {
        return false;
    }
    let nibble = |c: u8| (c as i32 - if c >= 65 { 55 } else { 48 }) as u8 & 0xF;
    let bytes: Vec<u8> = digits
        .chunks(2)
        .map(|p| (nibble(p[0]) << 4) | nibble(p[1]))
        .collect();
    let iterations = u32::from(u16::from_le_bytes([bytes[0], bytes[1]]));
    let salt = &bytes[2..2 + PBKDF2_SALT_SIZE];
    let mut hash = [0u8; PBKDF2_HASH_SIZE];
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(plain.as_bytes(), salt, iterations, &mut hash);
    hash[..] == bytes[2 + PBKDF2_SALT_SIZE..]
}

// 16 MiB at t=1 is cheaper than 8 MiB at t=3 and twice as memory-hard. p=1.
const ARGON_MEMORY: u32 = 16384;
const ARGON_TIME: u32 = 1;
const ARGON_PARALLEL: u32 = 1;
const ARGON_SALT_LEN: usize = 16;
const ARGON_HASH_LEN: usize = 32;

struct Phc {
    /// `argon2id` (the only type this server writes).
    id: bool,
    memory: u32,
    time: u32,
    parallel: u32,
    salt: Vec<u8>,
    hash: Vec<u8>,
    version: u32,
}

/// `Argon2PasswordHasher.TryExtractMetadataValues`: `$argon2id$v=19$m=..,t=..,p=..$salt$hash`.
fn parse_phc(s: &str) -> Option<Phc> {
    let mut parts = s.split('$');
    if !parts.next()?.is_empty() {
        return None;
    }
    let kind = parts.next()?;
    let mut next = parts.next()?;
    let mut version = 0x10;
    if let Some(v) = next.strip_prefix("v=") {
        version = v.parse().ok()?;
        next = parts.next()?;
    }
    let (mut memory, mut time, mut parallel) = (None, None, None);
    for kv in next.split(',') {
        let (k, v) = kv.split_once('=')?;
        let v: u32 = v.parse().ok()?;
        match k {
            "m" => memory = Some(v),
            "t" => time = Some(v),
            "p" => parallel = Some(v),
            _ => return None,
        }
    }
    let salt = b64::decode(parts.next()?)?;
    let hash = b64::decode(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some(Phc {
        id: kind == "argon2id",
        memory: memory?,
        time: time?,
        parallel: parallel?,
        salt,
        hash,
        version,
    })
}

fn argon_algorithm(kind_id: bool) -> argon2::Algorithm {
    if kind_id {
        argon2::Algorithm::Argon2id
    } else {
        argon2::Algorithm::Argon2i
    }
}

#[allow(clippy::too_many_arguments)]
fn argon_hash(
    id: bool,
    version: u32,
    m: u32,
    t: u32,
    p: u32,
    salt: &[u8],
    plain: &str,
    out_len: usize,
) -> Option<Vec<u8>> {
    let params = argon2::Params::new(m, t, p, Some(out_len)).ok()?;
    let version = if version == 0x10 {
        argon2::Version::V0x10
    } else {
        argon2::Version::V0x13
    };
    let a = argon2::Argon2::new(argon_algorithm(id), version, params);
    let mut out = vec![0u8; out_len];
    a.hash_password_into(plain.as_bytes(), salt, &mut out)
        .ok()?;
    Some(out)
}

fn argon2_encrypt(plain: &str) -> String {
    let mut salt = [0u8; ARGON_SALT_LEN];
    fill_random(&mut salt);
    let hash = argon_hash(
        true,
        0x13,
        ARGON_MEMORY,
        ARGON_TIME,
        ARGON_PARALLEL,
        &salt,
        plain,
        ARGON_HASH_LEN,
    )
    .expect("the configured Argon2 parameters are valid");
    format!(
        "$argon2id$v=19$m={ARGON_MEMORY},t={ARGON_TIME},p={ARGON_PARALLEL}${}${}",
        b64::encode(&salt),
        b64::encode(&hash)
    )
}

/// Verification uses the parameters embedded in the PHC string, not the configured ones.
fn argon2_validate(encrypted: &str, plain: &str) -> bool {
    let Some(p) = parse_phc(encrypted) else {
        return false;
    };
    let Some(h) = argon_hash(
        p.id,
        p.version,
        p.memory,
        p.time,
        p.parallel,
        &p.salt,
        plain,
        p.hash.len(),
    ) else {
        return false;
    };
    // constant time compare
    h.len() == p.hash.len() && h.iter().zip(&p.hash).fold(0u8, |a, (x, y)| a | (x ^ y)) == 0
}

/// Standard base64 without padding, as PHC strings use.
mod base64_nopad {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn encode(data: &[u8]) -> String {
        let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
        for chunk in data.chunks(3) {
            let n = (u32::from(chunk[0]) << 16)
                | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
                | u32::from(*chunk.get(2).unwrap_or(&0));
            let chars = chunk.len() + 1;
            for i in 0..chars {
                out.push(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize] as char);
            }
        }
        out
    }

    pub fn decode(s: &str) -> Option<Vec<u8>> {
        let mut out = Vec::with_capacity(s.len() * 3 / 4);
        let (mut acc, mut bits) = (0u32, 0);
        for c in s.bytes() {
            let v = ALPHABET.iter().position(|&a| a == c)? as u32;
            acc = (acc << 6) | v;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((acc >> bits) as u8);
                acc &= (1 << bits) - 1;
            }
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The account hash of an existing `Saves/Accounts/Accounts.bin`.
    const STORED_HASH: &str = "$argon2id$v=19$m=16384,t=1,p=1$4GNsKkr19s/EYh69BEok7g$DaCza1+WDuiHwDhADouzsvDjw5MOZxp/JGbO/JJga0A";

    #[test]
    fn base64_round_trips() {
        for n in 0..20 {
            let data: Vec<u8> = (0..n).map(|i| (i * 37 + 5) as u8).collect();
            assert_eq!(b64::decode(&b64::encode(&data)).unwrap(), data);
        }
        assert_eq!(b64::encode(b"\xe0\x63\x6c"), "4GNs");
    }

    #[test]
    fn a_stored_argon2_hash_is_understood() {
        let p = parse_phc(STORED_HASH).unwrap();
        assert!(p.id && p.memory == 16384 && p.time == 1 && p.parallel == 1);
        assert_eq!((p.salt.len(), p.hash.len()), (16, 32));
        assert!(!needs_rehash(
            PasswordProtectionAlgorithm::Argon2,
            STORED_HASH
        ));
        assert!(!validate_password(
            PasswordProtectionAlgorithm::Argon2,
            STORED_HASH,
            "not-the-password"
        ));
    }

    #[test]
    fn argon2_round_trip_and_format() {
        let h = encrypt_password(PasswordProtectionAlgorithm::Argon2, "hunter2").unwrap();
        assert!(h.starts_with("$argon2id$v=19$m=16384,t=1,p=1$"), "{h}");
        assert_eq!(h.len(), STORED_HASH.len());
        assert!(validate_password(
            PasswordProtectionAlgorithm::Argon2,
            &h,
            "hunter2"
        ));
        assert!(!validate_password(
            PasswordProtectionAlgorithm::Argon2,
            &h,
            "hunter3"
        ));
    }

    #[test]
    fn pbkdf2_round_trip() {
        let h = encrypt_password(PasswordProtectionAlgorithm::Pbkdf2, "hunter2").unwrap();
        assert_eq!(h.len(), 84);
        assert!(validate_password(
            PasswordProtectionAlgorithm::Pbkdf2,
            &h,
            "hunter2"
        ));
        assert!(!validate_password(
            PasswordProtectionAlgorithm::Pbkdf2,
            &h,
            "hunter3"
        ));
    }

    #[test]
    fn ascii_bytes_counts_utf16_units() {
        assert_eq!(ascii_bytes("a\u{1F600}b"), b"a??b");
        assert_eq!(ascii_bytes(&"x".repeat(300)).len(), 256);
        assert_eq!(ascii_bytes(&"\u{1F600}".repeat(200)).len(), 256);
    }

    #[test]
    fn digests_are_upper_hex() {
        assert_eq!(
            hash_digest(PasswordProtectionAlgorithm::Md5, "abc"),
            "900150983CD24FB0D6963F7D28E17F72"
        );
        assert_eq!(
            hash_digest(PasswordProtectionAlgorithm::Sha1, "abc"),
            "A9993E364706816ABA3E25717850C26C9CD0D89D"
        );
        assert_eq!(
            derive_phrase(PasswordProtectionAlgorithm::Sha2, "u", "p"),
            "up"
        );
        assert_eq!(
            derive_phrase(PasswordProtectionAlgorithm::Argon2, "u", "p"),
            "p"
        );
    }

    const PLAIN: &str = "hello-good-sir";
    const ALGORITHMS: [PasswordProtectionAlgorithm; 5] = [
        PasswordProtectionAlgorithm::Argon2,
        PasswordProtectionAlgorithm::Pbkdf2,
        PasswordProtectionAlgorithm::Md5,
        PasswordProtectionAlgorithm::Sha1,
        PasswordProtectionAlgorithm::Sha2,
    ];

    #[test]
    fn validates() {
        for a in ALGORITHMS {
            let encrypted = encrypt_password(a, PLAIN).unwrap();
            assert!(validate_password(a, &encrypted, PLAIN), "{a:?}");
        }
    }

    #[test]
    fn password_does_not_validate() {
        for a in ALGORITHMS {
            let encrypted = encrypt_password(a, PLAIN).unwrap();
            assert!(
                !validate_password(a, &encrypted, "Not the same password"),
                "{a:?}"
            );
        }
    }

    // Literal digests of PLAIN, so the stored format cannot drift: a casing or encoding change would lock out
    // every SHA and MD5 account on the shard at once.
    #[test]
    fn hash_algorithm_stored_format_is_stable() {
        for (a, expected) in [
            (
                PasswordProtectionAlgorithm::Md5,
                "52284053181040AC90DBDE74A0E7FF5E",
            ),
            (
                PasswordProtectionAlgorithm::Sha1,
                "9AC635509803AAE2D8312BA1879289259A50C5F0",
            ),
            (
                PasswordProtectionAlgorithm::Sha2,
                "5A727BFF8F8E08A24BDF6B0CD5065F30A1F8E0060B857BB8AFD6955BE0ACBC489DA63F19B8F4CF08D73DE4069CF4B29D94B353F31513B2FB2D9382EFE15AE975",
            ),
        ] {
            assert_eq!(encrypt_password(a, PLAIN).unwrap(), expected, "{a:?}");
            assert!(validate_password(a, expected, PLAIN), "{a:?}");
        }
    }

    // The shipping default before the argon2id change. Password: "hunter2".
    const LEGACY_ARGON2I_HASH: &str = "$argon2i$v=19$m=8192,t=3,p=1$LD1XJz7P3wQmIJ+Tu6ScgA$NO5hBABsHQ172C5nDO2X4gWnB4jDef3x6WhLdVE2LFw";
    const SALT_AND_DIGEST: &str =
        "LD1XJz7P3wQmIJ+Tu6ScgA$NO5hBABsHQ172C5nDO2X4gWnB4jDef3x6WhLdVE2LFw";

    #[test]
    fn argon2_validates_legacy_argon2i_hash() {
        let a = PasswordProtectionAlgorithm::Argon2;
        assert!(validate_password(a, LEGACY_ARGON2I_HASH, "hunter2"));
        assert!(!validate_password(a, LEGACY_ARGON2I_HASH, "wrong"));
    }

    /// The prefix the configured parameters produce, ending at the salt: `$argon2id$v=19$m=16384,t=1,p=1$`.
    fn current_prefix() -> String {
        let h = encrypt_password(PasswordProtectionAlgorithm::Argon2, "prefix-probe").unwrap();
        let end = h.match_indices('$').nth(3).unwrap().0 + 1;
        h[..end].to_string()
    }

    fn needs(hash: &str) -> bool {
        needs_rehash(PasswordProtectionAlgorithm::Argon2, hash)
    }

    #[test]
    fn argon2_needs_rehash_is_false_for_current_parameters() {
        assert!(!needs(&format!("{}{SALT_AND_DIGEST}", current_prefix())));
    }

    // Each case differs from the configured parameters in exactly one field.
    #[test]
    fn argon2_needs_rehash_compares_type_and_cost() {
        let prefix = current_prefix();
        let parts: Vec<&str> = prefix.split('$').collect();
        let (ty, version) = (parts[1], parts[2]);
        let params: Vec<i32> = parts[3]
            .split(',')
            .map(|kv| kv[2..].parse().unwrap())
            .collect();
        let (memory, time, parallelism) = (params[0], params[1], params[2]);
        let other_type = if ty == "argon2id" {
            "argon2i"
        } else {
            "argon2id"
        };

        for stale in ["type", "memory", "time", "parallelism"] {
            let (t, m, ti, p) = match stale {
                "type" => (other_type, memory, time, parallelism),
                "memory" => (ty, memory * 2, time, parallelism),
                "time" => (ty, memory, time + 1, parallelism),
                _ => (ty, memory, time, parallelism + 1),
            };
            let hash = format!("${t}${version}$m={m},t={ti},p={p}${SALT_AND_DIGEST}");
            assert!(needs(&hash), "{stale}");
        }
    }

    #[test]
    fn argon2_needs_rehash_is_true_for_legacy_shipping_default() {
        assert!(needs(LEGACY_ARGON2I_HASH));
    }

    // Digest and salt lengths are decoded base64 sizes rather than parameter-list entries.
    #[test]
    fn argon2_needs_rehash_compares_salt_and_digest_lengths() {
        for salt_and_digest in [
            // 16-byte digest: 22 base64 chars instead of the 43 a 32-byte digest encodes to
            "LD1XJz7P3wQmIJ+Tu6ScgA$NO5hBABsHQ172C5nDO2X4g",
            // 8-byte salt: 11 base64 chars instead of the 22 a 16-byte salt encodes to
            "LD1XJz7P3wQ$NO5hBABsHQ172C5nDO2X4gWnB4jDef3x6WhLdVE2LFw",
        ] {
            assert!(
                needs(&format!("{}{salt_and_digest}", current_prefix())),
                "{salt_and_digest}"
            );
        }
    }

    #[test]
    fn argon2_needs_rehash_is_true_for_unparseable_hashes() {
        for hash in ["", "not-a-hash"] {
            assert!(needs(hash), "{hash:?}");
        }
    }
}
