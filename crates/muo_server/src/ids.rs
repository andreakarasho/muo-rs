//! Ids that stand in for object references. A deleted entity's id simply stops resolving.

use muo_primitives::Serial;

/// An `Item` reference.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ItemId(pub Serial);

/// A `Mobile` reference.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct MobileId(pub Serial);

/// An `IEntity` reference.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum EntityId {
    Item(ItemId),
    Mobile(MobileId),
}

impl EntityId {
    /// The kind follows from the serial's range, as `World.FindEntity` does.
    pub fn from_serial(serial: Serial) -> Option<Self> {
        if serial.is_mobile() {
            Some(Self::Mobile(MobileId(serial)))
        } else if serial.is_item() {
            Some(Self::Item(ItemId(serial)))
        } else {
            None
        }
    }

    pub fn serial(self) -> Serial {
        match self {
            Self::Item(ItemId(s)) | Self::Mobile(MobileId(s)) => s,
        }
    }
}

impl From<ItemId> for EntityId {
    fn from(id: ItemId) -> Self {
        Self::Item(id)
    }
}

impl From<MobileId> for EntityId {
    fn from(id: MobileId) -> Self {
        Self::Mobile(id)
    }
}
