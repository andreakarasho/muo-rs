//! The Twofish engine of the game encryption.
//!
//! Twofish with a 128-bit key, ECB, exactly as the UO client computes it (words are little-endian; the key
//! schedule and `F32` must not change, or results stop matching for the same key).

const BLOCK_SIZE: usize = 16;
const ROUNDS: usize = 16;
const INPUT_WHITEN: usize = 0;
const OUTPUT_WHITEN: usize = 4;
const ROUND_SUBKEYS: usize = 8;
const TOTAL_SUBKEYS: usize = ROUND_SUBKEYS + 2 * ROUNDS;

const SK_STEP: u32 = 0x0202_0202;
const SK_BUMP: u32 = 0x0101_0101;
const SK_ROTL: u32 = 9;
const RS_GF_FDBK: u32 = 0x14D;
const MDS_GF_FDBK: i32 = 0x169;

static P0: [u8; 256] = [
    0xA9, 0x67, 0xB3, 0xE8, 0x04, 0xFD, 0xA3, 0x76, 0x9A, 0x92, 0x80, 0x78, 0xE4, 0xDD, 0xD1, 0x38,
    0x0D, 0xC6, 0x35, 0x98, 0x18, 0xF7, 0xEC, 0x6C, 0x43, 0x75, 0x37, 0x26, 0xFA, 0x13, 0x94, 0x48,
    0xF2, 0xD0, 0x8B, 0x30, 0x84, 0x54, 0xDF, 0x23, 0x19, 0x5B, 0x3D, 0x59, 0xF3, 0xAE, 0xA2, 0x82,
    0x63, 0x01, 0x83, 0x2E, 0xD9, 0x51, 0x9B, 0x7C, 0xA6, 0xEB, 0xA5, 0xBE, 0x16, 0x0C, 0xE3, 0x61,
    0xC0, 0x8C, 0x3A, 0xF5, 0x73, 0x2C, 0x25, 0x0B, 0xBB, 0x4E, 0x89, 0x6B, 0x53, 0x6A, 0xB4, 0xF1,
    0xE1, 0xE6, 0xBD, 0x45, 0xE2, 0xF4, 0xB6, 0x66, 0xCC, 0x95, 0x03, 0x56, 0xD4, 0x1C, 0x1E, 0xD7,
    0xFB, 0xC3, 0x8E, 0xB5, 0xE9, 0xCF, 0xBF, 0xBA, 0xEA, 0x77, 0x39, 0xAF, 0x33, 0xC9, 0x62, 0x71,
    0x81, 0x79, 0x09, 0xAD, 0x24, 0xCD, 0xF9, 0xD8, 0xE5, 0xC5, 0xB9, 0x4D, 0x44, 0x08, 0x86, 0xE7,
    0xA1, 0x1D, 0xAA, 0xED, 0x06, 0x70, 0xB2, 0xD2, 0x41, 0x7B, 0xA0, 0x11, 0x31, 0xC2, 0x27, 0x90,
    0x20, 0xF6, 0x60, 0xFF, 0x96, 0x5C, 0xB1, 0xAB, 0x9E, 0x9C, 0x52, 0x1B, 0x5F, 0x93, 0x0A, 0xEF,
    0x91, 0x85, 0x49, 0xEE, 0x2D, 0x4F, 0x8F, 0x3B, 0x47, 0x87, 0x6D, 0x46, 0xD6, 0x3E, 0x69, 0x64,
    0x2A, 0xCE, 0xCB, 0x2F, 0xFC, 0x97, 0x05, 0x7A, 0xAC, 0x7F, 0xD5, 0x1A, 0x4B, 0x0E, 0xA7, 0x5A,
    0x28, 0x14, 0x3F, 0x29, 0x88, 0x3C, 0x4C, 0x02, 0xB8, 0xDA, 0xB0, 0x17, 0x55, 0x1F, 0x8A, 0x7D,
    0x57, 0xC7, 0x8D, 0x74, 0xB7, 0xC4, 0x9F, 0x72, 0x7E, 0x15, 0x22, 0x12, 0x58, 0x07, 0x99, 0x34,
    0x6E, 0x50, 0xDE, 0x68, 0x65, 0xBC, 0xDB, 0xF8, 0xC8, 0xA8, 0x2B, 0x40, 0xDC, 0xFE, 0x32, 0xA4,
    0xCA, 0x10, 0x21, 0xF0, 0xD3, 0x5D, 0x0F, 0x00, 0x6F, 0x9D, 0x36, 0x42, 0x4A, 0x5E, 0xC1, 0xE0,
];

static P1: [u8; 256] = [
    0x75, 0xF3, 0xC6, 0xF4, 0xDB, 0x7B, 0xFB, 0xC8, 0x4A, 0xD3, 0xE6, 0x6B, 0x45, 0x7D, 0xE8, 0x4B,
    0xD6, 0x32, 0xD8, 0xFD, 0x37, 0x71, 0xF1, 0xE1, 0x30, 0x0F, 0xF8, 0x1B, 0x87, 0xFA, 0x06, 0x3F,
    0x5E, 0xBA, 0xAE, 0x5B, 0x8A, 0x00, 0xBC, 0x9D, 0x6D, 0xC1, 0xB1, 0x0E, 0x80, 0x5D, 0xD2, 0xD5,
    0xA0, 0x84, 0x07, 0x14, 0xB5, 0x90, 0x2C, 0xA3, 0xB2, 0x73, 0x4C, 0x54, 0x92, 0x74, 0x36, 0x51,
    0x38, 0xB0, 0xBD, 0x5A, 0xFC, 0x60, 0x62, 0x96, 0x6C, 0x42, 0xF7, 0x10, 0x7C, 0x28, 0x27, 0x8C,
    0x13, 0x95, 0x9C, 0xC7, 0x24, 0x46, 0x3B, 0x70, 0xCA, 0xE3, 0x85, 0xCB, 0x11, 0xD0, 0x93, 0xB8,
    0xA6, 0x83, 0x20, 0xFF, 0x9F, 0x77, 0xC3, 0xCC, 0x03, 0x6F, 0x08, 0xBF, 0x40, 0xE7, 0x2B, 0xE2,
    0x79, 0x0C, 0xAA, 0x82, 0x41, 0x3A, 0xEA, 0xB9, 0xE4, 0x9A, 0xA4, 0x97, 0x7E, 0xDA, 0x7A, 0x17,
    0x66, 0x94, 0xA1, 0x1D, 0x3D, 0xF0, 0xDE, 0xB3, 0x0B, 0x72, 0xA7, 0x1C, 0xEF, 0xD1, 0x53, 0x3E,
    0x8F, 0x33, 0x26, 0x5F, 0xEC, 0x76, 0x2A, 0x49, 0x81, 0x88, 0xEE, 0x21, 0xC4, 0x1A, 0xEB, 0xD9,
    0xC5, 0x39, 0x99, 0xCD, 0xAD, 0x31, 0x8B, 0x01, 0x18, 0x23, 0xDD, 0x1F, 0x4E, 0x2D, 0xF9, 0x48,
    0x4F, 0xF2, 0x65, 0x8E, 0x78, 0x5C, 0x58, 0x19, 0x8D, 0xE5, 0x98, 0x57, 0x67, 0x7F, 0x05, 0x64,
    0xAF, 0x63, 0xB6, 0xFE, 0xF5, 0xB7, 0x3C, 0xA5, 0xCE, 0xE9, 0x68, 0x44, 0xE0, 0x4D, 0x43, 0x69,
    0x29, 0x2E, 0xAC, 0x15, 0x59, 0xA8, 0x0A, 0x9E, 0x6E, 0x47, 0xDF, 0x34, 0x35, 0x6A, 0xCF, 0xDC,
    0x22, 0xC9, 0xC0, 0x9B, 0x89, 0xD4, 0xED, 0xAB, 0x12, 0xA2, 0x0D, 0x52, 0xBB, 0x02, 0x2F, 0xA9,
    0xD7, 0x61, 0x1E, 0xB4, 0x50, 0x04, 0xF6, 0xC2, 0x16, 0x25, 0x86, 0x56, 0x55, 0x09, 0xBE, 0x91,
];

pub struct TwofishEngine {
    sbox_keys: [u32; 2],
    sub_keys: [u32; TOTAL_SUBKEYS],
}

impl TwofishEngine {
    pub fn new(key: &[u8; 16]) -> Self {
        let k = |i: usize| {
            u32::from_le_bytes([key[i * 4], key[i * 4 + 1], key[i * 4 + 2], key[i * 4 + 3]])
        };
        let (k0, k1, k2, k3) = (k(0), k(1), k(2), k(3));

        let sbox_keys = [rs_mds_encode(k2, k3), rs_mds_encode(k0, k1)];
        let mut sub_keys = [0u32; TOTAL_SUBKEYS];
        for i in 0..TOTAL_SUBKEYS / 2 {
            let step = (i as u32).wrapping_mul(SK_STEP);
            let a = f32_key(step, k0, k2);
            let b = f32_key(step.wrapping_add(SK_BUMP), k1, k3).rotate_left(8);

            sub_keys[2 * i] = a.wrapping_add(b);
            sub_keys[2 * i + 1] = a.wrapping_add(b.wrapping_mul(2)).rotate_left(SK_ROTL);
        }

        Self {
            sbox_keys,
            sub_keys,
        }
    }

    /// Encrypts the first 16 bytes of `block` in place. Shorter input is left untouched.
    pub fn encrypt_block(&self, block: &mut [u8]) {
        if block.len() < BLOCK_SIZE {
            return;
        }

        let mut x = [0u32; 4];
        for (i, w) in x.iter_mut().enumerate() {
            *w = u32::from_le_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
            *w ^= self.sub_keys[INPUT_WHITEN + i];
        }

        for r in 0..ROUNDS {
            let t0 = self.f32_sbox(x[0]);
            let t1 = self.f32_sbox(x[1].rotate_left(8));

            x[3] = x[3].rotate_left(1);
            x[2] ^= t0
                .wrapping_add(t1)
                .wrapping_add(self.sub_keys[ROUND_SUBKEYS + 2 * r]);
            x[3] ^= t0
                .wrapping_add(t1.wrapping_mul(2))
                .wrapping_add(self.sub_keys[ROUND_SUBKEYS + 2 * r + 1]);
            x[2] = x[2].rotate_right(1);

            if r < ROUNDS - 1 {
                x.swap(0, 2);
                x.swap(1, 3);
            }
        }

        for (i, w) in x.iter().enumerate() {
            block[i * 4..i * 4 + 4]
                .copy_from_slice(&(w ^ self.sub_keys[OUTPUT_WHITEN + i]).to_le_bytes());
        }
    }

    fn f32_sbox(&self, x: u32) -> u32 {
        let [k0, k1] = self.sbox_keys;
        sbox_layers(x, k1, k0)
    }
}

/// Shared P-box layers of `F32Sbox` / `F32`: `outer` keys the first layer, `inner` the second.
fn sbox_layers(x: u32, outer: u32, inner: u32) -> u32 {
    let [mut b0, mut b1, mut b2, mut b3] = x.to_le_bytes();
    let [o0, o1, o2, o3] = outer.to_le_bytes();
    let [i0, i1, i2, i3] = inner.to_le_bytes();

    b0 = P0[b0 as usize] ^ o0;
    b1 = P1[b1 as usize] ^ o1;
    b2 = P0[b2 as usize] ^ o2;
    b3 = P1[b3 as usize] ^ o3;

    b0 = P0[b0 as usize] ^ i0;
    b1 = P0[b1 as usize] ^ i1;
    b2 = P1[b2 as usize] ^ i2;
    b3 = P1[b3 as usize] ^ i3;

    mds_multiply(
        P1[b0 as usize],
        P0[b1 as usize],
        P1[b2 as usize],
        P0[b3 as usize],
    )
}

fn f32_key(x: u32, k0: u32, k2: u32) -> u32 {
    sbox_layers(x, k2, k0)
}

fn mds_multiply(b0: u8, b1: u8, b2: u8, b3: u8) -> u32 {
    let (b0, b1, b2, b3) = (b0 as i32, b1 as i32, b2 as i32, b3 as i32);
    let m0 = (b0 ^ lfsr2(b1) ^ lfsr1(b2) ^ lfsr1(b3)) as u32;
    let m1 = (lfsr1(b0) ^ lfsr2(b1) ^ lfsr2(b2) ^ b3) as u32;
    let m2 = (lfsr2(b0) ^ lfsr1(b1) ^ b2 ^ lfsr2(b3)) as u32;
    let m3 = (lfsr2(b0) ^ b1 ^ lfsr2(b2) ^ lfsr1(b3)) as u32;

    m0 | (m1 << 8) | (m2 << 16) | (m3 << 24)
}

fn lfsr1(v: i32) -> i32 {
    v ^ lfsr4(v)
}

fn lfsr2(v: i32) -> i32 {
    v ^ lfsr3(v) ^ lfsr4(v)
}

fn lfsr3(v: i32) -> i32 {
    let a = if v & 0x01 == 0x01 { MDS_GF_FDBK / 2 } else { 0 };
    (v >> 1) ^ a
}

fn lfsr4(v: i32) -> i32 {
    let a = if v & 0x02 == 0x02 { MDS_GF_FDBK / 2 } else { 0 };
    let b = if v & 0x01 == 0x01 { MDS_GF_FDBK / 4 } else { 0 };
    (v >> 2) ^ a ^ b
}

fn rs_mds_encode(k0: u32, k1: u32) -> u32 {
    let mut r: u32 = 0;
    for i in 0..2 {
        r ^= if i > 0 { k0 } else { k1 };

        for _ in 0..4 {
            let v1 = (r >> 24) as u8 as u32;
            let fb2 = if v1 & 0x80 == 0x80 { RS_GF_FDBK } else { 0 };
            let fb3 = if v1 & 1 == 1 { RS_GF_FDBK >> 1 } else { 0 };
            let v2 = ((v1 << 1) ^ fb2) & 0xFF;
            let v3 = ((v1 >> 1) & 0x7F) ^ fb3 ^ v2;

            r = (r << 8) ^ (v3 << 24) ^ (v2 << 16) ^ (v3 << 8) ^ v1;
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::TwofishEngine;

    // Known-good outputs. The first row is also the published Twofish-128 ECB known answer (all-zero key and
    // block), so this engine agrees with the spec.
    const VECTORS: &[(&str, &str, &str)] = &[
        (
            "00000000000000000000000000000000",
            "00000000000000000000000000000000",
            "9f589f5cf6122c32b6bfec2f2ae8c35a",
        ),
        (
            "00000000000000000000000000000000",
            "000102030405060708090a0b0c0d0e0f",
            "670ff829d07bac240ec196160e814558",
        ),
        (
            "00000000000000000000000000000000",
            "ffffffffffffffffffffffffffffffff",
            "72fe83874d481d0eda30afb9eb12cf45",
        ),
        (
            "000102030405060708090a0b0c0d0e0f",
            "00000000000000000000000000000000",
            "6275e8ca35b36c108ad6d5f84f0cc5a3",
        ),
        (
            "000102030405060708090a0b0c0d0e0f",
            "000102030405060708090a0b0c0d0e0f",
            "9fb63337151be9c71306d159ea7afaa4",
        ),
        (
            "000102030405060708090a0b0c0d0e0f",
            "ffffffffffffffffffffffffffffffff",
            "197baec90528a8d99e050937d1401a02",
        ),
        (
            "deadbeefdeadbeefdeadbeefdeadbeef",
            "00000000000000000000000000000000",
            "17edddf77e62880cb4c977e148544986",
        ),
        (
            "deadbeefdeadbeefdeadbeefdeadbeef",
            "000102030405060708090a0b0c0d0e0f",
            "1691accf7abd17ac4dea4074fbe57a8f",
        ),
        (
            "deadbeefdeadbeefdeadbeefdeadbeef",
            "ffffffffffffffffffffffffffffffff",
            "d417167b0babbd39a246c8af2768ec4a",
        ),
        (
            "7f0000017f0000017f0000017f000001",
            "00000000000000000000000000000000",
            "7c1b520d5d7c602d26b73b8035396f3a",
        ),
        (
            "7f0000017f0000017f0000017f000001",
            "000102030405060708090a0b0c0d0e0f",
            "b31dd95046c764887de83057ed456ffc",
        ),
        (
            "7f0000017f0000017f0000017f000001",
            "ffffffffffffffffffffffffffffffff",
            "40131501aa4c5d57ab1a3c1e3719b8e2",
        ),
    ];

    fn unhex<const N: usize>(s: &str) -> [u8; N] {
        let mut out = [0u8; N];
        for (i, b) in out.iter_mut().enumerate() {
            *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
        }
        out
    }

    #[test]
    fn matches_reference_vectors() {
        for (key, pt, ct) in VECTORS {
            let engine = TwofishEngine::new(&unhex::<16>(key));
            let mut block = unhex::<16>(pt);
            engine.encrypt_block(&mut block);
            assert_eq!(block, unhex::<16>(ct), "key {key} pt {pt}");
        }
    }

    #[test]
    fn short_block_is_left_alone() {
        let engine = TwofishEngine::new(&[0; 16]);
        let mut b = [1u8; 8];
        engine.encrypt_block(&mut b);
        assert_eq!(b, [1u8; 8]);
    }
}
