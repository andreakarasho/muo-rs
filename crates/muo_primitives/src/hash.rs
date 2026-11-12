//! String and stream hashes.
//!
//! Saves store these hashes, so the seeds and the byte view of a string (UTF-16 little endian, what
//! `MemoryMarshal.Cast<char, byte>` sees) must not change.

use twox_hash::{XxHash3_64, XxHash32};

// *************** DO NOT CHANGE THIS NUMBER ****************
// * Computed hashes might be serialized against this seed! *
// **********************************************************
const XXHASH3_SEED: u64 = 9_609_125_370_673_258_709; // Randomly generated 64-bit prime number
const XXHASH1_SEED: u32 = 665_738_807; // Randomly generated 32-bit prime number

fn utf16_le(s: &str) -> Vec<u8> {
    s.encode_utf16().flat_map(u16::to_le_bytes).collect()
}

/// `ComputeHash64(ReadOnlySpan<char>)`: XxHash3 over the UTF-16 bytes, 0 for the empty string.
pub fn compute_hash64(s: &str) -> u64 {
    if s.is_empty() {
        return 0;
    }
    compute_hash64_bytes(&utf16_le(s))
}

/// `ComputeHash64(ReadOnlySpan<byte>)`: 0 for no bytes.
pub fn compute_hash64_bytes(bytes: &[u8]) -> u64 {
    if bytes.is_empty() {
        return 0;
    }
    XxHash3_64::oneshot_with_seed(XXHASH3_SEED, bytes)
}

/// `ComputeHash64(Stream)`: reads to the end. An I/O error is returned, not folded into 0.
pub fn compute_hash64_reader(reader: &mut impl std::io::Read) -> std::io::Result<u64> {
    use std::hash::Hasher;
    let mut h = create_xxhash3();
    let mut buf = [0u8; 8192];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            return Ok(h.finish());
        }
        h.write(&buf[..n]);
    }
}

/// `CreateXxHash3()`: a streaming hasher with the standard seed, for composing several inputs
/// (`use std::hash::Hasher`, `write`, `finish`).
pub fn create_xxhash3() -> XxHash3_64 {
    XxHash3_64::with_seed(XXHASH3_SEED)
}

/// `ComputeHash32(ReadOnlySpan<char>)`: 0 for the empty string.
pub fn compute_hash32(s: &str) -> u32 {
    if s.is_empty() {
        return 0;
    }
    XxHash32::oneshot(XXHASH1_SEED, &utf16_le(s))
}

/// `GetNetFrameworkHashCode`: the legacy 32-bit x86 framework string hash. It reads past the last char
/// into the string's null terminator when the length is odd, which is why the units are padded with zeros.
pub fn get_net_framework_hash_code(s: &str) -> i32 {
    let mut units: Vec<u16> = s.encode_utf16().collect();
    let mut length = units.len() as i64;
    units.extend([0u16; 4]);
    let word = |i: usize| u32::from(units[i]) | (u32::from(units[i + 1]) << 16);

    let mut hash1: u32 = (5381 << 16) + 5381;
    let mut hash2 = hash1;
    let mut p = 0usize;
    while length > 2 {
        length -= 4;
        hash1 = (hash1.rotate_left(5).wrapping_add(hash1)) ^ word(p);
        hash2 = (hash2.rotate_left(5).wrapping_add(hash2)) ^ word(p + 2);
        p += 4;
    }
    if length > 0 {
        hash2 = (hash2.rotate_left(5).wrapping_add(hash2)) ^ word(p);
    }
    hash1.wrapping_add(hash2.wrapping_mul(1_566_083_941)) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_zero() {
        assert_eq!(compute_hash64(""), 0);
        assert_eq!(compute_hash64_bytes(&[]), 0);
        assert_eq!(compute_hash32(""), 0);
    }

    #[test]
    fn string_is_utf16_bytes_and_stream_agrees() {
        let direct = compute_hash64("Server.Items.Longsword");
        let bytes: Vec<u8> = "Server.Items.Longsword"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(direct, compute_hash64_bytes(&bytes));
        assert_eq!(
            compute_hash64_reader(&mut bytes.as_slice()).unwrap(),
            direct
        );
        assert_ne!(direct, 0);
    }

    // Recorded reference values of the seeded hashes and GetNetFrameworkHashCode.
    #[test]
    fn golden_values() {
        let long = "héllo wörld, a longer string that is well over sixteen chars to hit more paths in xxh3 ........................................................................................................................................................... and beyond 240 bytes for sure, yes";
        assert_eq!(
            compute_hash64("Server.Items.Longsword"),
            10560130927047166899
        );
        assert_eq!(compute_hash64("a"), 15677392336991074139);
        assert_eq!(compute_hash64("abc"), 284500477701289315);
        assert_eq!(compute_hash64(long), 2901734541797752569);
        assert_eq!(compute_hash32("Server.Items.Longsword"), 2442423790);
        assert_eq!(compute_hash32("a"), 2998202569);
        assert_eq!(compute_hash32("abc"), 3247641947);
        assert_eq!(compute_hash32(long), 3156952691);
        assert_eq!(
            get_net_framework_hash_code("Server.Items.Longsword"),
            18741310
        );
        assert_eq!(get_net_framework_hash_code("a"), 972181027);
        assert_eq!(get_net_framework_hash_code("abc"), 536991770);
        assert_eq!(get_net_framework_hash_code(""), 757602046);
        assert_eq!(get_net_framework_hash_code(long), -658219357);
    }
}
