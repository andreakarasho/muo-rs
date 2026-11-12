//! The protocol change flags derived from a client version.

use bitflags::bitflags;

bitflags! {
    /// Wire-format changes a client version brings with it.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct ProtocolChanges: u32 {
        const NEW_SPELLBOOK = 0x0000_0001;
        const DAMAGE_PACKET = 0x0000_0002;
        // 0x4 (Unpack) is unused.
        const BUFF_ICON = 0x0000_0008;
        const NEW_HAVEN = 0x0000_0010;
        const CONTAINER_GRID_LINES = 0x0000_0020;
        const EXTENDED_SUPPORTED_FEATURES = 0x0000_0040;
        const STYGIAN_ABYSS = 0x0000_0080;
        const HIGH_SEAS = 0x0000_0100;
        const NEW_CHARACTER_LIST = 0x0000_0200;
        const NEW_CHARACTER_CREATION = 0x0000_0400;
        const EXTENDED_STATUS = 0x0000_0800;
        const NEW_MOBILE_INCOMING = 0x0000_1000;
        const NEW_SECURE_TRADING = 0x0000_2000;
        const ULTIMA_STORE = 0x0000_4000;
        const ENDLESS_JOURNEY = 0x0000_8000;

        const VERSION_500A = Self::NEW_SPELLBOOK.bits() | Self::DAMAGE_PACKET.bits();
        const VERSION_502B = Self::VERSION_500A.bits() | Self::BUFF_ICON.bits();
        const VERSION_6000 = Self::VERSION_502B.bits() | Self::NEW_HAVEN.bits();
        const VERSION_6017 = Self::VERSION_6000.bits() | Self::CONTAINER_GRID_LINES.bits();
        const VERSION_60142 = Self::VERSION_6017.bits() | Self::EXTENDED_SUPPORTED_FEATURES.bits();
        const VERSION_7000 = Self::VERSION_60142.bits() | Self::STYGIAN_ABYSS.bits();
        const VERSION_7090 = Self::VERSION_7000.bits() | Self::HIGH_SEAS.bits();
        const VERSION_70130 = Self::VERSION_7090.bits() | Self::NEW_CHARACTER_LIST.bits();
        const VERSION_70160 = Self::VERSION_70130.bits() | Self::NEW_CHARACTER_CREATION.bits();
        const VERSION_70300 = Self::VERSION_70160.bits() | Self::EXTENDED_STATUS.bits();
        const VERSION_70331 = Self::VERSION_70300.bits() | Self::NEW_MOBILE_INCOMING.bits();
        const VERSION_704565 = Self::VERSION_70331.bits() | Self::NEW_SECURE_TRADING.bits();
        const VERSION_70500 = Self::VERSION_704565.bits() | Self::ULTIMA_STORE.bits();
        const VERSION_70610 = Self::VERSION_70500.bits() | Self::ENDLESS_JOURNEY.bits();
    }
}

#[cfg(test)]
mod tests {
    use super::ProtocolChanges as P;

    #[test]
    fn composites_have_the_expected_values() {
        assert_eq!(P::VERSION_500A.bits(), 0x3);
        assert_eq!(P::VERSION_502B.bits(), 0xB);
        assert_eq!(P::VERSION_6000.bits(), 0x1B);
        assert_eq!(P::VERSION_70331.bits(), 0x1FFB);
        assert_eq!(P::VERSION_70610.bits(), 0xFFFB);
    }
}
