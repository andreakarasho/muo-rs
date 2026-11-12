//! `Direction` and `MessageType`.

use bitflags::bitflags;

bitflags! {
    /// A facing in the low three bits plus the `Running` flag. A bare facing is not a single flag
    /// (`North` is 0), so test it with [`Direction::facing`].
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, Debug)]
    pub struct Direction: u8 {
        const NORTH = 0x0;
        const RIGHT = 0x1;
        const EAST = 0x2;
        const DOWN = 0x3;
        const SOUTH = 0x4;
        const LEFT = 0x5;
        const WEST = 0x6;
        const UP = 0x7;

        const MASK = 0x7;
        const RUNNING = 0x80;
        const VALUE_MASK = 0x87;
    }
}

impl Direction {
    /// The facing without the running flag (`d & Direction.Mask`).
    pub const fn facing(self) -> Direction {
        Direction::from_bits_retain(self.bits() & Direction::MASK.bits())
    }

    pub const fn is_running(self) -> bool {
        self.bits() & Direction::RUNNING.bits() != 0
    }
}

bitflags! {
    /// Speech types of the unicode/ascii speech packets.
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
    pub struct MessageType: i32 {
        const REGULAR = 0x00;
        const SYSTEM = 0x01;
        const EMOTE = 0x02;
        const LABEL = 0x06;
        const FOCUS = 0x07;
        const WHISPER = 0x08;
        const YELL = 0x09;
        const SPELL = 0x0A;

        const GUILD = 0x0D;
        const ALLIANCE = 0x0E;
        const COMMAND = 0x0F;

        const ENCODED = 0xC0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_bits() {
        let d = Direction::SOUTH | Direction::RUNNING;
        assert_eq!(d.bits(), 0x84);
        assert_eq!(d.facing(), Direction::SOUTH);
        assert!(d.is_running() && !Direction::SOUTH.is_running());
        assert_eq!(d & Direction::VALUE_MASK, d);
        assert_eq!(Direction::NORTH.bits(), 0);
    }

    #[test]
    fn message_type_bits() {
        assert_eq!(MessageType::ENCODED.bits(), 0xC0);
        assert_eq!((MessageType::YELL | MessageType::ENCODED).bits(), 0xC9);
    }
}
