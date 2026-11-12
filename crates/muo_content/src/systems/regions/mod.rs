//! The region classes, registered by `$type` name for `Data/regions.json`
//! (`RegionJsonRegistration`, in [`configure`]).

pub mod base_region;
pub mod guarded;
pub mod house;
pub mod simple;

use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::region::{self, RegionId, RegionType};
use muo_server::type_registry::{Phase, TypeRegistry};

pub use base_region::{BASE_REGION, RegionData, RegionHooks, TravelResult};
pub use guarded::GUARDED_REGION;
pub use simple::*;

use crate::spells::TravelCheckType;
use crate::systems::jail::JAIL_REGION;

/// `Region.Find(..) as BaseRegion` then `CheckTravel`: `None` for a region that isn't a BaseRegion class.
pub fn check_travel(
    w: &mut World,
    id: RegionId,
    m: MobileId,
    loc: Point3D,
    t: TravelCheckType,
) -> Option<TravelResult> {
    base_region::check_travel(w, id, m, loc, t)
}

/// `region.IsPartOf<DungeonRegion>()`.
pub fn is_part_of_dungeon(w: &World, id: RegionId) -> bool {
    region::is_part_of_type(w, id, &DUNGEON_REGION)
}

/// `region.GetRegion<GuardedRegion>()`.
pub fn get_guarded(w: &World, id: RegionId) -> Option<RegionId> {
    region::get_region_of_type(w, id, &GUARDED_REGION)
}

/// `region.GetRegion<SafeZone>()`: the SafeZone class (ConPVP) is registered by its owner in [`RegionHooks`].
pub fn get_safe_zone(w: &World, id: RegionId) -> Option<RegionId> {
    let ty = w.resources.get::<RegionHooks>()?.safe_zone?;
    region::get_region_of_type(w, id, ty)
}

/// Every class `RegionJsonRegistration.Configure()` registers, in its order.
pub fn json_types() -> [&'static RegionType; 33] {
    [
        &BASE_REGION,
        &TOWN_REGION,
        &DUNGEON_REGION,
        &GUARDED_REGION,
        // Travel restricted regions
        &GREEN_ACRES_REGION,
        &JAIL_REGION,
        &MONDAIN_REGION,
        &NEW_MAGINCIA_REGION,
        &NO_HOUSING_REGION,
        &NO_HOUSING_GUARDED_REGION,
        &NO_TRAVEL_SPELLS_ALLOWED_REGION,
        &TWISTED_WEALD_DESERT_REGION,
        // Damage boost regions
        &CRYSTAL_FIELD_REGION,
        &ICY_RIVER_REGION,
        &ACID_RIVER_REGION,
        &POISONED_TREE_REGION,
        &LOST_CITY_ENTRANCE_REGION,
        &POISONED_CEMETERY_REGION,
        // Wrong Jail
        &WRONG_LEVEL3_REGION,
        &WRONG_JAIL_REGION,
        // Blackthorn Dungeon
        &BLACKTHORN_DUNGEON_REGION,
        &COUSTEAU_PERRON_HOUSE_REGION,
        &APPRENTICE_REGION,
        &SEA_MARKET_REGION,
        // Exploring the Deep
        &EXPLORING_DEEP_CREATURES_REGION,
        &UNDERWATER_REGION,
        // Exodus Dungeon
        &EXODUS_DUNGEON_REGION,
        // Doom
        &DOOM_GUARDIAN_REGION,
        // Tokuno
        &TOKUNO_DOCKS_REGION,
        // Tomb of Kings
        &TOMB_OF_KINGS_REGION,
        &TOK_BRIDGE_REGION,
        // Myrmidex
        &BATTLE_REGION,
        // Not in the default list, but a class: kept registrable for custom data.
        &BLACKTHORN_CASTLE_REGION,
    ]
}

/// `RegionJsonRegistration.Configure()`: `RegionJsonSerializer.Register<T>()` for each class.
pub fn configure(w: &mut World) {
    // BLACKTHORN_CASTLE_REGION is not registered by default (the last entry).
    let types = json_types();
    for ty in &types[..types.len() - 1] {
        if let Err(e) = w.maps.region_types.register(ty) {
            tracing::error!("{e}");
        }
    }
}

pub fn register(t: &mut TypeRegistry) {
    t.register_phase_default(
        Phase::Configure,
        "Server.Regions.RegionJsonRegistration.Configure",
        configure,
    );
    t.register_phase_default(
        Phase::Configure,
        "Server.Regions.GuardedRegion.Configure",
        guarded::configure,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_registration_names() {
        let mut w = World::new();
        configure(&mut w);
        for name in [
            "BaseRegion",
            "TownRegion",
            "DungeonRegion",
            "GuardedRegion",
            "JailRegion",
            "TokunoDocksRegion",
            "BattleRegion",
        ] {
            assert!(w.maps.region_types.get(name).is_some(), "{name}");
        }
        assert!(
            w.maps.region_types.get("BlackthornCastleRegion").is_none(),
            "not registered by default"
        );
        assert!(
            w.maps.region_types.get("Region").is_some(),
            "the base class is always there"
        );
        // Registering twice is an error that is logged, not a panic.
        configure(&mut w);
    }

    #[test]
    fn regions_json_builds_these_classes() {
        let mut w = muo_server::map::test_support::world();
        configure(&mut w);
        let json = r#"[
          { "$type": "TownRegion", "Entrance": {"x": 1, "y": 2, "z": 3}, "Map": "Felucca", "Name": "Britain", "Priority": 50,
            "Area": [{"x1": 1400, "y1": 1500, "x2": 1500, "y2": 1700}], "RuneName": "Britain", "GuardType": "Server.Mobiles.ArcherGuard" },
          { "$type": "DungeonRegion", "Map": "Felucca", "Name": "Covetous", "Priority": 50,
            "Area": [{"x1": 5400, "y1": 1800, "x2": 5500, "y2": 1900}], "Entrance": {"x": 5455, "y": 1862, "z": 0} }
        ]"#;
        let n = muo_server::region_json::load_regions_from_str(
            &mut w,
            json,
            muo_data::expansion::Expansion::AOS,
        )
        .unwrap();
        assert_eq!(n, 2);
        let britain =
            region::find_by_name(&w, "Britain", muo_server::map::MapId::FELUCCA, false).unwrap();
        assert!(w.region(britain).unwrap().ty.is(&GUARDED_REGION));
        assert_eq!(
            guarded::guard_type(&w, britain),
            "Server.Mobiles.ArcherGuard"
        );
        assert_eq!(
            base_region::data(&w, britain).unwrap().entrance,
            Point3D::new(1, 2, 3)
        );
        assert_eq!(base_region::to_string(&w, britain), "Britain");
        let covetous =
            region::find_by_name(&w, "Covetous", muo_server::map::MapId::FELUCCA, false).unwrap();
        assert!(is_part_of_dungeon(&w, covetous) && get_guarded(&w, covetous).is_none());
        assert_eq!(
            base_region::data(&w, covetous).unwrap().entrance,
            Point3D::new(5455, 1862, 0)
        );
    }
}
