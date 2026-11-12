//! Client encryption: configuration, login keys, and the login and game stream ciphers.
//!
//! `EncryptionManager`'s statics become [`EncryptionConfig`], owned by whoever runs the transport; the
//! `NetState` it detected for is replaced by the values it read (address, version, seed).

use crate::client_version::ClientVersion;
use crate::md5::md5;
use crate::twofish::TwofishEngine;
use bitflags::bitflags;
use std::net::IpAddr;
use tracing::debug;

const LOGIN_PACKET_SIZE: usize = 62;
const GAME_LOGIN_PACKET_SIZE: usize = 65;
const CIPHER_TABLE_SIZE: usize = 256;
const BLOCK_SIZE: usize = 16;

bitflags! {
    /// Which encryption modes the server accepts (`EncryptionMode`; `None` is the empty set).
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct EncryptionMode: u32 {
        /// Accept unencrypted clients (e.g., ClassicUO with encryption disabled).
        const UNENCRYPTED = 0x1;
        /// Accept encrypted clients (original UO client, Enhanced Client).
        const ENCRYPTED = 0x2;
        const BOTH = Self::UNENCRYPTED.bits() | Self::ENCRYPTED.bits();
    }
}

impl EncryptionMode {
    /// Parses a `network.encryptionMode` setting: a comma-separated flags name list ("Both", "Unencrypted, Encrypted")
    /// or the number. `None` for anything else.
    pub fn from_setting(s: &str) -> Option<Self> {
        let s = s.trim();
        if let Ok(n) = s.parse::<u32>() {
            return Some(Self::from_bits_truncate(n));
        }

        let mut mode = Self::empty();
        for part in s.split(',') {
            mode |= match part.trim().to_ascii_lowercase().as_str() {
                "none" => Self::empty(),
                "unencrypted" => Self::UNENCRYPTED,
                "encrypted" => Self::ENCRYPTED,
                "both" => Self::BOTH,
                _ => return None,
            };
        }
        Some(mode)
    }
}

/// `network.encryptionMode` (default `Both`) and `network.encryptionDebug` (default false).
#[derive(Copy, Clone, Debug)]
pub struct EncryptionConfig {
    pub mode: EncryptionMode,
    pub debug: bool,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            mode: EncryptionMode::BOTH,
            debug: false,
        }
    }
}

impl EncryptionConfig {
    /// `EncryptionManager.Enabled`: any mode other than `None`.
    pub fn enabled(&self) -> bool {
        !self.mode.is_empty()
    }
}

/// In-place stream cipher over the connection (`IClientEncryption`).
pub trait ClientEncryption {
    /// Decrypts incoming data from the client.
    fn client_decrypt(&mut self, buffer: &mut [u8]);
    /// Encrypts outgoing data to the client.
    fn server_encrypt(&mut self, buffer: &mut [u8]);
}

// --- LoginKeys ---------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LoginKeys {
    pub key1: u32,
    pub key2: u32,
}

impl LoginKeys {
    pub const EMPTY: LoginKeys = LoginKeys { key1: 0, key2: 0 };

    /// Keys for a client version; `None` gives [`LoginKeys::EMPTY`].
    pub fn get_keys(version: Option<&ClientVersion>) -> LoginKeys {
        match version {
            None => Self::EMPTY,
            Some(v) => Self::compute_keys(v.major as u32, v.minor as u32, v.revision as u32),
        }
    }

    /// The UO key derivation (wrapping 32-bit unsigned math).
    pub fn compute_keys(major: u32, minor: u32, revision: u32) -> LoginKeys {
        let mut key1 = (major << 23) | (minor << 14) | (revision << 4);
        key1 ^= revision.wrapping_mul(revision) << 9;
        key1 ^= minor.wrapping_mul(minor);
        key1 ^= minor.wrapping_mul(11) << 24;
        key1 ^= revision.wrapping_mul(7) << 19;
        key1 ^= 0x2C13_A5FD;

        let mut key2 = (major << 22) | (revision << 13) | (minor << 3);
        key2 ^= revision.wrapping_mul(revision).wrapping_mul(3) << 10;
        key2 ^= minor.wrapping_mul(minor);
        key2 ^= minor.wrapping_mul(13) << 23;
        key2 ^= revision.wrapping_mul(7) << 18;
        key2 ^= 0xA31D_527F;

        LoginKeys { key1, key2 }
    }

    /// Keys for every client from 4.0.11 through 6.0.4: those do not send 0xEF, so there is no version at
    /// login time and detection tries each.
    pub fn legacy_keys() -> Vec<LoginKeys> {
        const VERSIONS: [(u32, u32, u32); 16] = [
            (4, 0, 11),
            (5, 0, 0),
            (5, 0, 1),
            (5, 0, 2),
            (5, 0, 3),
            (5, 0, 4),
            (5, 0, 5),
            (5, 0, 6),
            (5, 0, 7),
            (5, 0, 8),
            (5, 0, 9),
            (6, 0, 0),
            (6, 0, 1),
            (6, 0, 2),
            (6, 0, 3),
            (6, 0, 4),
        ];
        VERSIONS
            .iter()
            .map(|&(a, b, c)| Self::compute_keys(a, b, c))
            .collect()
    }
}

// --- LoginEncryption ---------------------------------------------------------------------------------

/// XOR stream keyed by the seed and the version-derived keys; client to server only.
pub struct LoginEncryption {
    table1: u32,
    table2: u32,
    key1: u32,
    key2: u32,
}

impl LoginEncryption {
    pub fn new(seed: u32, keys: LoginKeys) -> Self {
        Self {
            table1: ((!seed ^ 0x0000_1357) << 16) | ((seed ^ 0xFFFF_AAAA) & 0x0000_FFFF),
            table2: ((seed ^ 0x4321_0000) >> 16) | ((!seed ^ 0xABCD_FFFF) & 0xFFFF_0000),
            key1: keys.key1,
            key2: keys.key2,
        }
    }

    /// True when the packet decrypts to a plausible 0x80 login with `version`'s keys, or, with no version
    /// (pre-6.0.5 clients that never sent 0xEF), with any legacy key.
    pub fn try_decrypt(
        version: Option<&ClientVersion>,
        seed: u32,
        encrypted: &[u8],
    ) -> Option<LoginEncryption> {
        if encrypted.len() < LOGIN_PACKET_SIZE {
            return None;
        }

        if version.is_some() {
            let keys = LoginKeys::get_keys(version);
            return if keys == LoginKeys::EMPTY {
                None
            } else {
                Self::try_decrypt_with_keys(keys, seed, encrypted)
            };
        }

        LoginKeys::legacy_keys()
            .into_iter()
            .find_map(|k| Self::try_decrypt_with_keys(k, seed, encrypted))
    }

    fn try_decrypt_with_keys(
        keys: LoginKeys,
        seed: u32,
        encrypted: &[u8],
    ) -> Option<LoginEncryption> {
        let mut decrypted = [0u8; LOGIN_PACKET_SIZE];
        decrypted.copy_from_slice(&encrypted[..LOGIN_PACKET_SIZE]);
        Self::new(seed, keys).client_decrypt(&mut decrypted);

        // 0x80 = account login id, and the username (30) and password (60) fields end in a null.
        if decrypted[0] != 0x80 || decrypted[30] != 0 || decrypted[60] != 0 {
            return None;
        }

        // Fresh state for actual use
        Some(Self::new(seed, keys))
    }
}

impl ClientEncryption for LoginEncryption {
    fn client_decrypt(&mut self, buffer: &mut [u8]) {
        for b in buffer {
            *b ^= self.table1 as u8;

            let mut edx = self.table2;
            let esi = self.table1 << 31;
            let mut eax = self.table2 >> 1;

            eax |= esi;
            eax ^= self.key1.wrapping_sub(1);
            edx <<= 31;
            eax >>= 1;

            let mut ecx = self.table1 >> 1;

            eax |= esi;
            ecx |= edx;
            eax ^= self.key1;
            ecx ^= self.key2;

            self.table1 = ecx;
            self.table2 = eax;
        }
    }

    /// The server does not encrypt login responses.
    fn server_encrypt(&mut self, _buffer: &mut [u8]) {}
}

// --- GameEncryption ----------------------------------------------------------------------------------

/// Twofish + MD5 stream used after the 0x91 game login.
pub struct GameEncryption {
    twofish: TwofishEngine,
    cipher_table: [u8; CIPHER_TABLE_SIZE],
    xor_key: [u8; 16],
    recv_pos: usize,
    send_pos: usize,
}

impl GameEncryption {
    pub fn new(seed: u32) -> Self {
        // 16-byte key: the big-endian seed repeated 4 times
        let mut key = [0u8; 16];
        for chunk in key.as_chunks_mut::<4>().0 {
            chunk.copy_from_slice(&seed.to_be_bytes());
        }

        let mut cipher_table = [0u8; CIPHER_TABLE_SIZE];
        for (i, b) in cipher_table.iter_mut().enumerate() {
            *b = i as u8;
        }

        let mut enc = Self {
            twofish: TwofishEngine::new(&key),
            cipher_table,
            xor_key: [0; 16],
            recv_pos: 0,
            send_pos: 0,
        };
        enc.refresh_cipher_table();
        // MD5 of the cipher table is the server-to-client xor key
        enc.xor_key = md5(&enc.cipher_table);
        enc
    }

    /// Encrypts the cipher table with Twofish; every 256 received bytes.
    fn refresh_cipher_table(&mut self) {
        for block in self.cipher_table.as_chunks_mut::<BLOCK_SIZE>().0 {
            self.twofish.encrypt_block(block);
        }
        self.recv_pos = 0;
    }

    pub fn try_decrypt(seed: u32, encrypted: &[u8], debug_log: bool) -> Option<GameEncryption> {
        if encrypted.len() < GAME_LOGIN_PACKET_SIZE {
            if debug_log {
                debug!(
                    "GameEncryption.TryDecrypt: Invalid buffer length {}",
                    encrypted.len()
                );
            }
            return None;
        }

        let mut decrypted = [0u8; GAME_LOGIN_PACKET_SIZE];
        decrypted.copy_from_slice(&encrypted[..GAME_LOGIN_PACKET_SIZE]);
        let mut enc = Self::new(seed);
        enc.client_decrypt(&mut decrypted);

        // First byte must be 0x91 (game server login packet id)
        if decrypted[0] != 0x91 {
            if debug_log {
                debug!("GameEncryption.TryDecrypt: Validation FAILED - first byte is not 0x91");
            }
            return None;
        }

        Some(Self::new(seed))
    }

    #[cfg(test)]
    fn cipher_table(&self) -> &[u8; CIPHER_TABLE_SIZE] {
        &self.cipher_table
    }

    #[cfg(test)]
    fn xor_key(&self) -> &[u8; 16] {
        &self.xor_key
    }
}

impl ClientEncryption for GameEncryption {
    fn client_decrypt(&mut self, buffer: &mut [u8]) {
        for b in buffer {
            if self.recv_pos >= CIPHER_TABLE_SIZE {
                self.refresh_cipher_table();
            }
            *b ^= self.cipher_table[self.recv_pos];
            self.recv_pos += 1;
        }
    }

    fn server_encrypt(&mut self, buffer: &mut [u8]) {
        for b in buffer {
            *b ^= self.xor_key[self.send_pos];
            self.send_pos = (self.send_pos + 1) & 0x0F;
        }
    }
}

// --- EncryptionManager detection ---------------------------------------------------------------------

/// What `DetectLoginEncryption` / `DetectGameEncryption` decided.
#[allow(clippy::large_enum_variant)]
pub enum Detected {
    /// Not acceptable under the mode, or undecryptable.
    Rejected,
    /// Plain traffic, accepted.
    Unencrypted,
    Encrypted(Box<dyn ClientEncryption>),
}

impl Detected {
    pub fn is_rejected(&self) -> bool {
        matches!(self, Detected::Rejected)
    }
}

/// Detects the encryption of a login packet (0x80). `buffer` is the 62-byte packet.
pub fn detect_login_encryption(
    config: &EncryptionConfig,
    address: Option<IpAddr>,
    version: Option<&ClientVersion>,
    seed: u32,
    buffer: &[u8],
) -> Detected {
    if buffer.len() < LOGIN_PACKET_SIZE {
        return Detected::Rejected;
    }

    // Unencrypted if the id is 0x80, or both the username and password null terminators are present
    let is_unencrypted = buffer[0] == 0x80 || (buffer[30] == 0 && buffer[60] == 0);

    if is_unencrypted {
        if !config.mode.contains(EncryptionMode::UNENCRYPTED) {
            if config.debug {
                debug!(
                    "Client {address:?}: Unencrypted login rejected (mode: {:?})",
                    config.mode
                );
            }
            return Detected::Rejected;
        }
        if config.debug {
            debug!("Client {address:?}: Unencrypted login detected");
        }
        return Detected::Unencrypted;
    }

    if !config.mode.contains(EncryptionMode::ENCRYPTED) {
        if config.debug {
            debug!(
                "Client {address:?}: Encrypted login rejected (mode: {:?})",
                config.mode
            );
        }
        return Detected::Rejected;
    }

    if let Some(enc) = LoginEncryption::try_decrypt(version, seed, buffer) {
        if config.debug {
            debug!("Client {address:?}: Encrypted login detected (version: {version:?})");
        }
        return Detected::Encrypted(Box::new(enc));
    }

    if config.debug {
        debug!("Client {address:?}: Login encryption detection failed");
    }
    Detected::Rejected
}

/// Detects the encryption of a game server login packet (0x91). `buffer` is the 65-byte packet.
pub fn detect_game_encryption(
    config: &EncryptionConfig,
    address: Option<IpAddr>,
    seed: u32,
    buffer: &[u8],
) -> Detected {
    if buffer.len() < GAME_LOGIN_PACKET_SIZE {
        return Detected::Rejected;
    }

    // Auth id: bytes 1-4, big-endian
    let auth_id = u32::from_be_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]);

    // Unencrypted if the id is 0x91, or the auth id equals the seed (no encryption applied)
    let is_unencrypted = buffer[0] == 0x91 || auth_id == seed;

    if is_unencrypted {
        if !config.mode.contains(EncryptionMode::UNENCRYPTED) {
            if config.debug {
                debug!(
                    "Client {address:?}: Unencrypted game login rejected (mode: {:?})",
                    config.mode
                );
            }
            return Detected::Rejected;
        }
        if config.debug {
            debug!("Client {address:?}: Unencrypted game login detected");
        }
        return Detected::Unencrypted;
    }

    if !config.mode.contains(EncryptionMode::ENCRYPTED) {
        if config.debug {
            debug!(
                "Client {address:?}: Encrypted game login rejected (mode: {:?})",
                config.mode
            );
        }
        return Detected::Rejected;
    }

    if let Some(enc) = GameEncryption::try_decrypt(seed, buffer, config.debug) {
        if config.debug {
            debug!("Client {address:?}: Encrypted game login detected (seed: 0x{seed:08X})");
        }
        return Detected::Encrypted(Box::new(enc));
    }

    if config.debug {
        debug!("Client {address:?}: Game encryption detection failed");
    }
    Detected::Rejected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_settings() {
        assert_eq!(
            EncryptionMode::from_setting("Both"),
            Some(EncryptionMode::BOTH)
        );
        assert_eq!(
            EncryptionMode::from_setting("Unencrypted, Encrypted"),
            Some(EncryptionMode::BOTH)
        );
        assert_eq!(
            EncryptionMode::from_setting("None"),
            Some(EncryptionMode::empty())
        );
        assert_eq!(
            EncryptionMode::from_setting("2"),
            Some(EncryptionMode::ENCRYPTED)
        );
        assert_eq!(EncryptionMode::from_setting("bogus"), None);
    }

    #[test]
    fn short_or_garbage_buffers_never_panic() {
        let cfg = EncryptionConfig::default();
        assert!(detect_login_encryption(&cfg, None, None, 1, &[0; 10]).is_rejected());
        assert!(detect_game_encryption(&cfg, None, 1, &[0; 10]).is_rejected());
        let noise: Vec<u8> = (0..65u32).map(|i| (i * 37 + 11) as u8).collect();
        let _ = detect_login_encryption(&cfg, None, None, 0xDEAD_BEEF, &noise[..62]);
        let _ = detect_game_encryption(&cfg, None, 0xDEAD_BEEF, &noise);
    }

    #[test]
    fn unencrypted_policy() {
        let mut packet = [0u8; 65];
        packet[0] = 0x80;
        let mut cfg = EncryptionConfig::default();
        assert!(matches!(
            detect_login_encryption(&cfg, None, None, 1, &packet[..62]),
            Detected::Unencrypted
        ));
        cfg.mode = EncryptionMode::ENCRYPTED;
        assert!(detect_login_encryption(&cfg, None, None, 1, &packet[..62]).is_rejected());
        packet[0] = 0x91;
        cfg.mode = EncryptionMode::BOTH;
        assert!(matches!(
            detect_game_encryption(&cfg, None, 1, &packet),
            Detected::Unencrypted
        ));
    }

    #[test]
    fn game_encryption_round_trip_and_detect() {
        // Encrypt a 0x91 packet the way a client would (the stream is its own inverse), then detect it.
        let seed = 0x7F00_0001;
        let mut packet = [0u8; 65];
        packet[0] = 0x91;
        packet[1..5].copy_from_slice(&0x1234_5678u32.to_be_bytes());
        let mut client = GameEncryption::new(seed);
        client.client_decrypt(&mut packet); // xor with the same table = client encrypt
        match detect_game_encryption(&EncryptionConfig::default(), None, seed, &packet) {
            Detected::Encrypted(_) => {}
            _ => panic!("encrypted game login not detected"),
        }
        let _ = GameEncryption::new(seed).cipher_table();
        let _ = GameEncryption::new(seed).xor_key();
    }

    // Golden values below are known-good outputs of the reference login keys, login and game encryption.
    #[test]
    fn legacy_login_keys_match_reference() {
        let expected: &[(u32, u32)] = &[
            (0x2c7b574d, 0xa32d9e7f),
            (0x2e93a5fd, 0xa25d527f),
            (0x2eaba7ed, 0xa2417e7f),
            (0x2ee3addd, 0xa265227f),
            (0x2e3bb7cd, 0xa2095e7f),
            (0x2e7385bd, 0xa22d127f),
            (0x2f8b97ad, 0xa2d0de7f),
            (0x2fc3ed9d, 0xa2f4227f),
            (0x2f1bc78d, 0xa29bfe7f),
            (0x2f53257d, 0xa2bf527f),
            (0x2f6b076d, 0xa2a3be7f),
            (0x2f13a5fd, 0xa29d527f),
            (0x2f2ba7ed, 0xa2817e7f),
            (0x2f63addd, 0xa2a5227f),
            (0x2fbbb7cd, 0xa2c95e7f),
            (0x2ff385bd, 0xa2ed127f),
        ];
        let keys = LoginKeys::legacy_keys();
        assert_eq!(keys.len(), expected.len());
        for (k, e) in keys.iter().zip(expected) {
            assert_eq!((k.key1, k.key2), *e);
        }
    }

    #[test]
    fn version_login_keys_match_reference() {
        for (maj, min, rev, k1, k2) in [
            (7, 0, 0, 0x2f93a5fdu32, 0xa2dd527fu32),
            (7, 0, 61, 0x22d6b42du32, 0xa4d89e7fu32),
            (6, 0, 14, 0x2c022d1du32, 0xa31da27fu32),
            (7, 0, 255, 0x19a7a80du32, 0xb2cebe7fu32),
        ] {
            let k = LoginKeys::compute_keys(maj, min, rev);
            assert_eq!((k.key1, k.key2), (k1, k2));
        }
        assert_eq!(LoginKeys::get_keys(None), LoginKeys::EMPTY);
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    #[test]
    fn login_stream_matches_reference() {
        for (seed, want) in [
            (
                0x7f000001u32,
                "a820fb9225c4a30cd8ccf1f3f9f64e06b960f3ced3478a502e8fd88f9f1d43e806e7883be1764a48ca35ddb58a9feae4f8708be25594f3bc681c210309069ed609d0a39e8337",
            ),
            (
                0xdeadbeefu32,
                "4657404fcb33587166935e241283f4db571748133db0712d905037f8a4800dcf156e4c59d0ee066ed93c59776bef52381607303fbb6308c1d6438ed4e273240be7a718436dc0",
            ),
            (
                0x00000001u32,
                "a820fb9225c4a30cd8ccf1f3f9f64e06b960f3ced3478a502e8fd88f9f9d8308f69fb425eef189a93a4de1ab8598e9e5f8708be25594f3bc681c210309069ed609d0a39e8337",
            ),
        ] {
            let mut enc = LoginEncryption::new(seed, LoginKeys::legacy_keys()[5]);
            let mut buf: Vec<u8> = (0..70u32).map(|i| (i * 7 + 3) as u8).collect();
            enc.client_decrypt(&mut buf);
            assert_eq!(hex(&buf), want);
        }
    }

    #[test]
    fn login_detection_round_trip() {
        // 0x80 login encrypted with the 5.0.5 keys and a seed: the stream is its own inverse.
        let keys = LoginKeys::legacy_keys()[7];
        let seed = 0x0A0B_0C0D;
        let mut packet = [0u8; 62];
        packet[0] = 0x80;
        packet[1..5].copy_from_slice(b"user");
        packet[31..35].copy_from_slice(b"pass");
        LoginEncryption::new(seed, keys).client_decrypt(&mut packet);
        assert!(LoginEncryption::try_decrypt(None, seed, &packet).is_some());
        assert!(LoginEncryption::try_decrypt(None, seed ^ 1, &packet).is_none());
        assert!(LoginEncryption::try_decrypt(None, seed, &packet[..61]).is_none());
    }

    #[test]
    fn game_stream_matches_reference() {
        for (seed, table, xor, dec, send) in [
            (
                0x7f000001u32,
                "b31dd95046c764887de83057ed456ffcfca3cf8dcf23d65306a1d860ddc0dc34b6a8ee0b0256ab721bd2f87c99557a853badb96b36974a7d5b2eed2e23a312cd0db5952f85cbfaba0fb65a35ca15f384df48a3dc138168c1e491ff7a42f49dcb69b1b5b6d27cdf3d96630c8a832ff05278f31e41aaa700dfab1a227b79ff0316051a54dcc4e2143a42d335351c0ff3048f0da9ce7790b85e7fd7c1d99f974d5ae83814bf9672f7b7beb864083857aadc0b41cdb9db78b46cfdd9cacab4f3f245f8209867fac3720f7a0e551693fbd9069e774192057c6cc7392b3b8217cc33939d9e4acaebcbf4ff201ad396b7600ff19cf524a16e778d4ec70ea0951b54c34d",
                "059266236714e362dc608cd6fe7c2569",
                [
                    "b213c27873852bd4149eb3c770efd8382d7d2475ca31c97f3fe78b00b0ba5ba0",
                    "1355d68e24b951355944f00141ff9a60",
                    "15e86b2625ac705a2d32f9e9801fe152eebf7a73d0a82604d836846691a1854a",
                ],
                "0591602a6b1bf177c47b92f7da5b0f4435a1501a5b2ba127942bc287aa2b7f3465f1004a0b7b9117a41bf2577afbafe49501",
            ),
            (
                0xdeadbeefu32,
                "1691accf7abd17ac4dea4074fbe57a8f3e5b1de8d27a8f2dd2291c7909c22026fbeefefb1c820bab1d64b5e47d02d94a7a521cf15684194a7098a88e17cd974c7f462c37ae15031ec21978d719638e993455354711994fbc576b17a5ab623f0f769df56723d2e06fe0a79efa80f6122d3a7f9ba32a12b4e51ebab77341a6556cd200634f971de41ecfe420354102f52121725bf2bc408c2e4c10edd5ec8909b41ca09ec810a5a0b2f63a50910deb9413d13a297d3d8e22f65b99f51f9cb23b43ca14f3784d9fb70f73f3c2bbb5d1e22c52564d6ba8d404ea9b9e5b87403c5f84ffa301d679e2348205745c83b31ce6b2435a2aef1cfb682e340a5d3ad6def81e",
                "83e84114bd44e6f45b1170a5e844c3b8",
                [
                    "179fb7e74fff58f0249cc3e4664fcd4bef85f610d7689001eb6f4f1964b8a7b2",
                    "eefa1b041fea0ec03dc3c926dc28cbc2",
                    "a3d7a41cf8206397fcfda6ed842fa35a741b812ae39a0ee77a3129ace0694266",
                ],
                "83eb471db14bf4e1430a6e84cc63e995b3db772d817ba4b1135a3ef4bc1399e5e38b277dd12b9481236a0e246cc34935137b",
            ),
            (
                0x00000001u32,
                "31f8f64be5ff64bc313688c6034f6b580a27d9569262e1e96a469d757113e3b5a6ca37b2ab0ce388d0541e587b11da9470e4b285276dbf62d5648bc0a6ca4a41819fa081cd5dfde1fdc7cd60e0000afe34a9408ccb7953198acbf6128ab3cc113b004293a9aebf4d1b670481f8768036bc9b9947d79e05dc5f42e962daf7aa53560526d16d9a13446d2fa84b8b29a5bd4ad07f9ae69cd2df27cb2760ffa01b00b197d676dfb194cdbaa66250082a0172d86e408728e14199e2a7e14cf1611cbe27af7e7379bd50c0a009984f5eb6e3d8adb6327893584c0a07e763afb7a50221691ac3c0f303f4f2cf844dc3f246e98d1abc0e0fc1a9ecde11d29ca9a1e7cf74",
                "19d23007e24c8b31fcb5bf9d51680a52",
                [
                    "30f6ed63d0bd2be058400b569ee5dc9cdbf932ae9770fec55300ce151c696421",
                    "2f696c3d2880035fb3ab5113d139d637",
                    "2acf378499009b6b5203af36b208ec4edd4357264064d1e2dcfa9b205dd2f6be",
                ],
                "19d1360eee439924e4aea1bc754f207f29e1063ede73c974b4fef1cc053f500f79b1566e8e23f94484cec11cd5ef80df8941",
            ),
        ] {
            let mut g = GameEncryption::new(seed);
            assert_eq!(hex(g.cipher_table()), table);
            assert_eq!(hex(g.xor_key()), xor);

            let mut buf: Vec<u8> = (0..600u32).map(|i| (i * 13 + 1) as u8).collect();
            g.client_decrypt(&mut buf);
            assert_eq!(hex(&buf[0..32]), dec[0]);
            assert_eq!(hex(&buf[250..266]), dec[1]);
            assert_eq!(hex(&buf[500..532]), dec[2]);

            let mut out: Vec<u8> = (0..50u32).map(|i| (i * 3) as u8).collect();
            g.server_encrypt(&mut out[..5]);
            g.server_encrypt(&mut out[5..]);
            assert_eq!(hex(&out), send);
        }
    }
}
