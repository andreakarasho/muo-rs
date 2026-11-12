//! `Beard` and `Hair`: the two static classes are tables of tile graphics (no item class).

/// `Beard`.
pub mod beard {
    pub const LONG_BEARD: i32 = 0x203E;
    pub const SHORT_BEARD: i32 = 0x203F;
    pub const GOATEE: i32 = 0x2040;
    pub const MUSTACHE: i32 = 0x2041;
    pub const MEDIUM_SHORT_BEARD: i32 = 0x204B;
    pub const MEDIUM_LONG_BEARD: i32 = 0x204C;
    pub const VANDYKE: i32 = 0x204D;
}

/// `Hair`.
pub mod hair {
    // Human
    pub const LONG: i32 = 0x203C;
    /// `Hair.Shor` (sic).
    pub const SHOR: i32 = 0x203B;
    pub const PONY_TAIL: i32 = 0x203D;
    pub const MOHAWK: i32 = 0x2044;
    pub const PAGEBOY: i32 = 0x2045;
    /// Female only.
    pub const BUN: i32 = 0x2046;
    pub const AFRO: i32 = 0x2047;
    /// Male only.
    pub const RECEDING: i32 = 0x2048;
    pub const TWO_PIG_TAILS: i32 = 0x2049;
    pub const KRISNA: i32 = 0x204A;
    // Elf
    /// Male only.
    pub const MID_LONG_ELF: i32 = 0x2FBF;
    pub const LONG_FEATHER: i32 = 0x2FC0;
    pub const SHORT_ELF: i32 = 0x2FC1;
    pub const MULLET: i32 = 0x2FC2;
    /// Female only.
    pub const FLOWER: i32 = 0x2FCC;
    /// Male only.
    pub const LONG_ELF: i32 = 0x2FCD;
    pub const KNOB: i32 = 0x2FCE;
    pub const BRAIDED: i32 = 0x2FCF;
    /// Female only.
    pub const BUN_ELF: i32 = 0x2FD0;
    pub const SPIKED: i32 = 0x2FD1;
}
