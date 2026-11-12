//! `FactionDefinition`, `RankDefinition` and `TownDefinition`: the constants of a
//! faction and a town. The values are in [`super::tables`]; text definitions are all cliloc numbers there.
//! with `StrongholdDefinition` (the area rectangles stay in [`super::tables::STRONGHOLD_AREAS`]), `GuardDefinition`
//! and `VendorDefinition`.

/// `RankDefinition`.
#[derive(Debug)]
pub struct RankDef {
    pub rank: i32,
    pub required: i32,
    pub max_wearables: i32,
    pub title: i32,
}

/// `FactionDefinition`.
#[derive(Debug)]
pub struct FactionDef {
    pub sort: i32,
    pub hue_primary: i32,
    pub hue_secondary: i32,
    pub hue_join: i32,
    pub hue_broadcast: i32,
    pub war_horse_body: i32,
    pub war_horse_item: i32,
    pub friendly_name: &'static str,
    pub keyword: &'static str,
    pub abbreviation: &'static str,
    pub name: i32,
    pub prop_name: i32,
    pub header: i32,
    pub about: i32,
    pub city_control: i32,
    pub sigil_control: i32,
    pub signup_name: i32,
    pub faction_stone_name: i32,
    pub owner_label: i32,
    pub guard_ignore: i32,
    pub guard_warn: i32,
    pub guard_attack: i32,
    pub stronghold: StrongholdDef,
    pub ranks: &'static [RankDef],
    pub guards: &'static [GuardDef],
}

/// `StrongholdDefinition` without `Area`.
#[derive(Debug)]
pub struct StrongholdDef {
    pub join_stone: (i32, i32, i32),
    pub faction_stone: (i32, i32, i32),
    pub monoliths: &'static [(i32, i32, i32)],
}

/// `GuardDefinition` / `VendorDefinition`: the type is its full serialized type name.
#[derive(Debug)]
pub struct GuardDef {
    pub type_name: &'static str,
    pub item_id: i32,
    pub price: i32,
    pub upkeep: i32,
    pub maximum: i32,
    pub header: i32,
    pub label: i32,
}

pub type VendorDef = GuardDef;

/// `TownDefinition`.
#[derive(Debug)]
pub struct TownDef {
    pub sort: i32,
    pub sigil_id: i32,
    pub region: &'static str,
    pub friendly_name: &'static str,
    pub town_name: i32,
    pub town_stone_header: i32,
    pub stronghold_monolith_name: i32,
    pub town_monolith_name: i32,
    pub town_stone_name: i32,
    pub sigil_name: i32,
    pub corrupted_sigil_name: i32,
    pub monolith: (i32, i32, i32),
    pub town_stone: (i32, i32, i32),
}

/// Where `special_wiring` looks for the tables.
pub use super::tables::{FACTION_DEFS, TOWN_DEFS};
