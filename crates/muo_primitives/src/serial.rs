//! Entity serial numbers.

use crate::parse::{FormatError, try_to_u32};
use std::fmt;
use std::ops::{Add, Sub};
use std::str::FromStr;

/// World.ItemOffset: serials below it are mobiles, from it up are items.
pub const ITEM_OFFSET: u32 = 0x4000_0000;
/// World.MaxItemSerial
pub const MAX_ITEM_SERIAL: u32 = 0x7EEE_EEEE;
/// World.MaxMobileSerial
pub const MAX_MOBILE_SERIAL: u32 = ITEM_OFFSET - 1;
/// World.MaxVirtualSerial (also `ResetVirtualSerial` is `MAX_ITEM_SERIAL`).
pub const MAX_VIRTUAL_SERIAL: u32 = 0x7FFF_FFFF;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct Serial(u32);

impl Serial {
    pub const MINUS_ONE: Serial = Serial(0xFFFF_FFFF);
    pub const ZERO: Serial = Serial(0);

    pub const fn new(value: u32) -> Serial {
        Serial(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }

    pub const fn is_mobile(self) -> bool {
        self.0 > 0 && self.0 < ITEM_OFFSET
    }

    pub const fn is_item(self) -> bool {
        self.0 >= ITEM_OFFSET && self.0 < MAX_ITEM_SERIAL
    }

    pub const fn is_valid(self) -> bool {
        self.0 > 0
    }

    /// `Serial.ToInt32()`
    pub const fn to_i32(self) -> i32 {
        self.0 as i32
    }

    /// `++serial` (unchecked, wraps)
    pub const fn next(self) -> Serial {
        Serial(self.0.wrapping_add(1))
    }

    /// `--serial`
    pub const fn prev(self) -> Serial {
        Serial(self.0.wrapping_sub(1))
    }
}

impl From<u32> for Serial {
    fn from(v: u32) -> Serial {
        Serial(v)
    }
}

impl From<Serial> for u32 {
    fn from(s: Serial) -> u32 {
        s.0
    }
}

impl PartialEq<u32> for Serial {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<u32> for Serial {
    fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl Add for Serial {
    type Output = Serial;
    fn add(self, r: Serial) -> Serial {
        Serial(self.0.wrapping_add(r.0))
    }
}

impl Add<u32> for Serial {
    type Output = Serial;
    fn add(self, r: u32) -> Serial {
        Serial(self.0.wrapping_add(r))
    }
}

impl Sub for Serial {
    type Output = Serial;
    fn sub(self, r: Serial) -> Serial {
        Serial(self.0.wrapping_sub(r.0))
    }
}

impl Sub<u32> for Serial {
    type Output = Serial;
    fn sub(self, r: u32) -> Serial {
        Serial(self.0.wrapping_sub(r))
    }
}

/// `0xABCD1234`, always eight hex digits.
impl fmt::Display for Serial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:08X}", self.0)
    }
}

impl fmt::Debug for Serial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::UpperHex for Serial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}
impl FromStr for Serial {
    type Err = FormatError;

    /// Decimal or `0x` hex, like `Utility.ToUInt32`.
    fn from_str(s: &str) -> Result<Serial, FormatError> {
        try_to_u32(s).map(Serial).ok_or(FormatError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_default() {
        let serial = Serial::new(0xABCD1234);
        assert_eq!(serial.to_string(), "0xABCD1234");
        assert_eq!(format!("{serial}"), "0xABCD1234");
    }

    #[test]
    fn format_custom() {
        // `{serial:##}` formats as the plain decimal number.
        assert_eq!(Serial::new(0xABCD1234).value().to_string(), "2882343476");
        assert_eq!(format!("{:X}", Serial::new(0xABCD1234)), "ABCD1234");
    }

    #[test]
    fn ranges() {
        assert!(!Serial::ZERO.is_valid() && !Serial::ZERO.is_mobile() && !Serial::ZERO.is_item());
        assert!(Serial::new(1).is_mobile() && Serial::new(MAX_MOBILE_SERIAL).is_mobile());
        assert!(!Serial::new(ITEM_OFFSET).is_mobile() && Serial::new(ITEM_OFFSET).is_item());
        assert!(
            Serial::new(MAX_ITEM_SERIAL - 1).is_item() && !Serial::new(MAX_ITEM_SERIAL).is_item()
        );
        assert!(Serial::MINUS_ONE.is_valid() && !Serial::MINUS_ONE.is_item());
        assert_eq!(Serial::MINUS_ONE.to_i32(), -1);
    }

    #[test]
    fn arithmetic_and_parse() {
        assert_eq!(Serial::MINUS_ONE + 1u32, Serial::ZERO);
        assert_eq!(Serial::ZERO.prev(), Serial::MINUS_ONE);
        assert_eq!(Serial::new(5) - Serial::new(2), Serial::new(3));
        assert!(Serial::new(5) > 4u32 && Serial::new(5) == 5u32);
        assert_eq!("0x40000000".parse::<Serial>(), Ok(Serial::new(ITEM_OFFSET)));
        assert_eq!("1073741824".parse::<Serial>(), Ok(Serial::new(ITEM_OFFSET)));
        assert!("-1".parse::<Serial>().is_err());
    }
}
