//! Equipment layers.

/// Item layer values: `Layer` is a byte enum, and a cast keeps any byte (a save can carry one without a name), so
/// this is the byte with the named values as constants. The aliases `FirstValid` (= `OneHanded`), `LastUserValid`
/// (= `InnerLegs`) and `LastValid` (= `Bank`).
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Layer(pub u8);

#[allow(non_upper_case_globals)]
impl Layer {
    pub const Invalid: Layer = Layer(0x00);
    pub const OneHanded: Layer = Layer(0x01);
    pub const TwoHanded: Layer = Layer(0x02);
    pub const Shoes: Layer = Layer(0x03);
    pub const Pants: Layer = Layer(0x04);
    pub const Shirt: Layer = Layer(0x05);
    pub const Helm: Layer = Layer(0x06);
    pub const Gloves: Layer = Layer(0x07);
    pub const Ring: Layer = Layer(0x08);
    pub const Talisman: Layer = Layer(0x09);
    pub const Neck: Layer = Layer(0x0A);
    pub const Hair: Layer = Layer(0x0B);
    pub const Waist: Layer = Layer(0x0C);
    pub const InnerTorso: Layer = Layer(0x0D);
    pub const Bracelet: Layer = Layer(0x0E);
    pub const UnusedXF: Layer = Layer(0x0F);
    pub const FacialHair: Layer = Layer(0x10);
    pub const MiddleTorso: Layer = Layer(0x11);
    pub const Earrings: Layer = Layer(0x12);
    pub const Arms: Layer = Layer(0x13);
    pub const Cloak: Layer = Layer(0x14);
    pub const Backpack: Layer = Layer(0x15);
    pub const OuterTorso: Layer = Layer(0x16);
    pub const OuterLegs: Layer = Layer(0x17);
    pub const InnerLegs: Layer = Layer(0x18);
    pub const Mount: Layer = Layer(0x19);
    pub const ShopBuy: Layer = Layer(0x1A);
    pub const ShopResale: Layer = Layer(0x1B);
    pub const ShopSell: Layer = Layer(0x1C);
    pub const Bank: Layer = Layer(0x1D);

    pub const FIRST_VALID: Layer = Layer::OneHanded;
    pub const LAST_USER_VALID: Layer = Layer::InnerLegs;
    pub const LAST_VALID: Layer = Layer::Bank;

    /// The layer of a named value; `None` for a byte without a name (`from_raw` keeps it).
    pub const fn from_u8(v: u8) -> Option<Layer> {
        if v <= Layer::Bank.0 {
            Some(Layer(v))
        } else {
            None
        }
    }

    /// `(Layer)v`: any byte, as a cast keeps it.
    pub const fn from_raw(v: u8) -> Layer {
        Layer(v)
    }

    /// The name of the value, `None` for a byte without one.
    pub const fn name(self) -> Option<&'static str> {
        Some(match self {
            Layer::Invalid => "Invalid",
            Layer::OneHanded => "OneHanded",
            Layer::TwoHanded => "TwoHanded",
            Layer::Shoes => "Shoes",
            Layer::Pants => "Pants",
            Layer::Shirt => "Shirt",
            Layer::Helm => "Helm",
            Layer::Gloves => "Gloves",
            Layer::Ring => "Ring",
            Layer::Talisman => "Talisman",
            Layer::Neck => "Neck",
            Layer::Hair => "Hair",
            Layer::Waist => "Waist",
            Layer::InnerTorso => "InnerTorso",
            Layer::Bracelet => "Bracelet",
            Layer::UnusedXF => "UnusedXF",
            Layer::FacialHair => "FacialHair",
            Layer::MiddleTorso => "MiddleTorso",
            Layer::Earrings => "Earrings",
            Layer::Arms => "Arms",
            Layer::Cloak => "Cloak",
            Layer::Backpack => "Backpack",
            Layer::OuterTorso => "OuterTorso",
            Layer::OuterLegs => "OuterLegs",
            Layer::InnerLegs => "InnerLegs",
            Layer::Mount => "Mount",
            Layer::ShopBuy => "ShopBuy",
            Layer::ShopResale => "ShopResale",
            Layer::ShopSell => "ShopSell",
            Layer::Bank => "Bank",
            _ => return None,
        })
    }
}

impl std::fmt::Debug for Layer {
    /// `Layer.ToString()`: the name, or the number for a byte without one.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.name() {
            Some(n) => f.write_str(n),
            None => write!(f, "{}", self.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_fixed() {
        assert_eq!(Layer::Bank.0, 0x1D);
        assert_eq!(Layer::from_u8(0x19), Some(Layer::Mount));
        assert_eq!(Layer::from_u8(0x1E), None);
        assert_eq!(Layer::LAST_USER_VALID.0, 0x18);
    }

    #[test]
    fn a_byte_without_a_name_is_kept() {
        let l = Layer::from_raw(0x1E);
        assert_eq!(l.0, 0x1E);
        assert_eq!(format!("{l:?}"), "30");
        assert_eq!(format!("{:?}", Layer::Mount), "Mount");
    }
}
