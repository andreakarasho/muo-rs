//! The `Mobile`/`Item`-facing interfaces, as ids.
//!
//! // shortcut: `ISpell`, `IParty` and `ISpawner` are handles here, their behavior sits in `SpellDeps`/`SpawnerDeps`
//! and the owning crates; entities store these handles. `IWeapon` is an item or the static default weapon.

use crate::ids::ItemId;

/// An `ISpell` reference (`Mobile.Spell`).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SpellId(pub u32);

/// An `IParty` reference (`Mobile.Party`, an `object`).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PartyId(pub u32);

/// An `ISpawner` reference (`Item.Spawner`, `Mobile.Spawner`).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct SpawnerId(pub u32);

/// An `IWeapon`: a weapon item, or `Mobile.DefaultWeapon` (fists).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum WeaponRef {
    Item(ItemId),
    Default,
}
