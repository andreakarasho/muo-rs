//! The `HousePlacementEntry` data of the house placement tool: what a house type stores and costs. The tables
//! are in `house_entries.rs`; the placement tool's gumps and targets are
//! `placement_tool.rs`.

use muo_data::expansion::Expansion;
use muo_primitives::{Direction, Point3D};
use muo_server::World;

use crate::house_entries::{
    CLASSIC_HOUSES, HOUSES_EJ, THREE_STORY_FOUNDATIONS, TWO_STORY_FOUNDATIONS,
};

/// One `HousePlacementEntry`; `class` is the short name of the house class (`Type.Name`).
#[derive(Debug)]
pub struct Entry {
    pub class: &'static str,
    pub description: i32,
    pub storage: i32,
    pub lockdowns: i32,
    pub new_storage: i32,
    pub new_lockdowns: i32,
    pub vendors: i32,
    pub cost: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub multi_id: i32,
    pub direction: Direction,
}

impl Entry {
    /// `Offset`.
    pub fn offset(&self) -> Point3D {
        Point3D::new(self.x, self.y, self.z)
    }

    /// `Storage`: `BaseHouse.NewVendorSystem` (Core.AOS) picks the new numbers.
    pub fn storage(&self, new_vendor_system: bool) -> i32 {
        if new_vendor_system {
            self.new_storage
        } else {
            self.storage
        }
    }

    /// `Lockdowns`.
    pub fn lockdowns(&self, new_vendor_system: bool) -> i32 {
        if new_vendor_system {
            self.new_lockdowns
        } else {
            self.lockdowns
        }
    }
}

/// `Core.EJ`.
fn core_ej(w: &World) -> bool {
    muo_content::core::expansion(w) >= Expansion::EJ
}

/// `ClassicHouses` or `HousesEJ`, as `Core.EJ` picks.
pub fn houses(w: &World) -> &'static [Entry] {
    if core_ej(w) {
        &HOUSES_EJ
    } else {
        &CLASSIC_HOUSES
    }
}

/// `HousePlacementEntry.TwoStoryFoundations`.
pub fn two_story() -> &'static [Entry] {
    &TWO_STORY_FOUNDATIONS
}

/// `HousePlacementEntry.ThreeStoryFoundations`.
pub fn three_story() -> &'static [Entry] {
    &THREE_STORY_FOUNDATIONS
}

/// Which table a class' `ConvertEntry` indexes.
#[derive(Copy, Clone, Debug)]
pub enum Table {
    Two,
    Three,
}

/// `HousePlacementEntry.Find(house)`: the entry of the house's class (by `Type.Name`); a class with several entries
/// is told apart by `ItemID`, a class with one entry gets it whatever the `ItemID`.
pub fn find(w: &World, class: &str, item_id: i32) -> Option<&'static Entry> {
    let mut all = houses(w)
        .iter()
        .chain(two_story())
        .chain(three_story())
        .filter(|e| e.class == class);
    let first = all.next()?;
    let Some(second) = all.next() else {
        return Some(first);
    };
    // A dictionary keeps the last entry written for an id; no table repeats one.
    std::iter::once(first)
        .chain(std::iter::once(second))
        .chain(all)
        .find(|e| e.multi_id == item_id)
}
