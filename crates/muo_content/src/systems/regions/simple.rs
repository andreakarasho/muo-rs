//! The small region classes: DungeonRegion, TownRegion, GreenAcresRegion, NoHousingRegion,
//! NoHousingGuardedRegion, NoTravelSpellsAllowedRegion, MondainRegion, TwistedWealdDesertRegion, TombOfKingsRegion,
//! NewMaginciaRegion, and the ones that are only a name for now (not implemented yet).

use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::interfaces::SpellId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages as mm;
use muo_server::packets::movement::{SpeedControlSetting, send_speed_control};
use muo_server::region::{RegionId, RegionJson, RegionType};

use super::base_region::{
    BASE_REGION, BASE_REGION_FAMILY, BaseRegionType, RegionHooks, TravelResult, base_from_json,
    data_mut, json_point,
};
use super::guarded::GUARDED_REGION;
use crate::spells::{TravelCheckType, spell};

fn send(w: &mut World, to: MobileId, number: i32) {
    mm::send_localized_message(w, to, number, "", 0x3B2);
}

fn access_level(w: &World, m: MobileId) -> AccessLevel {
    w.mobile(m).map_or(AccessLevel::Player, |m| m.access_level)
}

// ---- DungeonRegion ----------------------------------------------------------------------------------------------

fn dungeon_from_json(w: &mut World, id: RegionId, o: &RegionJson) -> Result<(), String> {
    base_from_json(w, id, o)?;
    if let Some(p) = json_point(o, "Entrance")? {
        data_mut(w, id).ok_or("region is gone")?.entrance = p;
    }
    Ok(())
}

pub static DUNGEON_FAMILY: BaseRegionType = BaseRegionType {
    young_protected: false,
    ..BASE_REGION_FAMILY
};

/// `DungeonRegion`.
pub static DUNGEON_REGION: RegionType = RegionType {
    name: "DungeonRegion",
    parent: Some(&BASE_REGION),
    from_json: dungeon_from_json,
    allow_housing: |_, _, _, _| false,
    alter_light_level: |_, _, _, global, _| *global = crate::misc::light_cycle::DUNGEON_LEVEL,
    can_use_stuck_menu: |w, id, m| {
        w.region(id)
            .is_some_and(|r| r.map != muo_server::map::MapId::FELUCCA)
            && (BASE_REGION.can_use_stuck_menu)(w, id, m)
    },
    family: Some(&DUNGEON_FAMILY),
    ..BASE_REGION
};

// ---- TownRegion -------------------------------------------------------------------------------------------------

/// `TownRegion`: a guarded region with an `Entrance`.
pub static TOWN_REGION: RegionType = RegionType {
    name: "TownRegion",
    parent: Some(&GUARDED_REGION),
    ..GUARDED_REGION
};

/// `NewMaginciaRegion`.
pub static NEW_MAGINCIA_REGION: RegionType = RegionType {
    name: "NewMaginciaRegion",
    parent: Some(&TOWN_REGION),
    ..TOWN_REGION
};

// ---- GreenAcresRegion -------------------------------------------------------------------------------------------

fn green_acres_check_travel(
    w: &mut World,
    _: RegionId,
    m: MobileId,
    _: Point3D,
    _: TravelCheckType,
) -> TravelResult {
    // Use default message
    if access_level(w, m) != AccessLevel::Player {
        TravelResult::allow()
    } else {
        TravelResult::deny()
    }
}

fn green_acres_on_begin_spell_cast(w: &mut World, id: RegionId, m: MobileId, s: SpellId) -> bool {
    let is_mark =
        spell::spell_type(w, s).is_some_and(|t| t.full_name == "Server.Spells.Sixth.MarkSpell");
    if access_level(w, m) == AccessLevel::Player && is_mark {
        send(w, m, 501802); // Thy spell doth not appear to work...
        return false;
    }
    (BASE_REGION.on_begin_spell_cast)(w, id, m, s)
}

pub static GREEN_ACRES_FAMILY: BaseRegionType = BaseRegionType {
    check_travel: green_acres_check_travel,
    ..BASE_REGION_FAMILY
};

/// `GreenAcresRegion`.
pub static GREEN_ACRES_REGION: RegionType = RegionType {
    name: "GreenAcresRegion",
    parent: Some(&BASE_REGION),
    allow_housing: |w, id, from, p| {
        access_level(w, from) != AccessLevel::Player && (BASE_REGION.allow_housing)(w, id, from, p)
    },
    on_begin_spell_cast: green_acres_on_begin_spell_cast,
    family: Some(&GREEN_ACRES_FAMILY),
    ..BASE_REGION
};

// ---- NoHousing --------------------------------------------------------------------------------------------------

/// `NoHousingRegion`: housing is allowed here as far as the region is concerned, because it is checked manually so
/// that a part of a house cannot overlap.
pub static NO_HOUSING_REGION: RegionType = RegionType {
    name: "NoHousingRegion",
    parent: Some(&BASE_REGION),
    allow_housing: |_, _, _, _| true,
    ..BASE_REGION
};

/// `NoHousingGuardedRegion`.
pub static NO_HOUSING_GUARDED_REGION: RegionType = RegionType {
    name: "NoHousingGuardedRegion",
    parent: Some(&GUARDED_REGION),
    ..GUARDED_REGION
};

/// `TokunoDocksRegion`.
pub static TOKUNO_DOCKS_REGION: RegionType = RegionType {
    name: "TokunoDocksRegion",
    parent: Some(&NO_HOUSING_GUARDED_REGION),
    ..NO_HOUSING_GUARDED_REGION
};

// ---- NoTravelSpellsAllowedRegion and the Mondain regions ---------------------------------------------------------

fn no_travel_check_travel(
    w: &mut World,
    _: RegionId,
    m: MobileId,
    _: Point3D,
    _: TravelCheckType,
) -> TravelResult {
    // Use default message
    if access_level(w, m) > AccessLevel::Player {
        TravelResult::allow()
    } else {
        TravelResult::deny()
    }
}

pub static NO_TRAVEL_FAMILY: BaseRegionType = BaseRegionType {
    check_travel: no_travel_check_travel,
    ..DUNGEON_FAMILY
};

/// `NoTravelSpellsAllowedRegion`.
pub static NO_TRAVEL_SPELLS_ALLOWED_REGION: RegionType = RegionType {
    name: "NoTravelSpellsAllowedRegion",
    parent: Some(&DUNGEON_REGION),
    family: Some(&NO_TRAVEL_FAMILY),
    ..DUNGEON_REGION
};

/// `MondainRegion`.
pub static MONDAIN_REGION: RegionType = RegionType {
    name: "MondainRegion",
    parent: Some(&NO_TRAVEL_SPELLS_ALLOWED_REGION),
    ..NO_TRAVEL_SPELLS_ALLOWED_REGION
};

fn under_animal_form(w: &World, m: MobileId) -> bool {
    w.resources
        .get::<RegionHooks>()
        .and_then(|h| h.under_animal_form)
        .is_some_and(|f| f(w, m))
}

fn twisted_weald_on_enter(w: &mut World, _: RegionId, m: MobileId) {
    let ns = w.mobile(m).and_then(|m| m.net_state);
    if let Some(ns) = ns
        && !under_animal_form(w, m)
        && access_level(w, m) == AccessLevel::Player
    {
        send_speed_control(w, ns, SpeedControlSetting::Walk);
    }
}

fn twisted_weald_on_exit(w: &mut World, _: RegionId, m: MobileId) {
    let ns = w.mobile(m).and_then(|m| m.net_state);
    if let Some(ns) = ns
        && !under_animal_form(w, m)
    {
        send_speed_control(w, ns, SpeedControlSetting::Disable);
    }
}

/// `TwistedWealdDesertRegion`.
pub static TWISTED_WEALD_DESERT_REGION: RegionType = RegionType {
    name: "TwistedWealdDesertRegion",
    parent: Some(&MONDAIN_REGION),
    on_enter: twisted_weald_on_enter,
    on_exit: twisted_weald_on_exit,
    ..MONDAIN_REGION
};

/// `TwistedWealdDesertRegion.OnLogin(pm)`: `PlayerMobile.PlayerLoginEvent`.
pub fn twisted_weald_on_login(w: &mut World, pm: MobileId) {
    let region = w.mobile(pm).map(|m| m.region);
    let in_desert = region
        .is_some_and(|r| muo_server::region::is_part_of_type(w, r, &TWISTED_WEALD_DESERT_REGION));
    if access_level(w, pm) == AccessLevel::Player
        && in_desert
        && let Some(ns) = w.mobile(pm).and_then(|m| m.net_state)
    {
        send_speed_control(w, ns, SpeedControlSetting::Walk);
    }
}

// ---- the rest ---------------------------------------------------------------------------------------------------

/// `TombOfKingsRegion`: no light underground.
pub static TOMB_OF_KINGS_REGION: RegionType = RegionType {
    name: "TombOfKingsRegion",
    parent: Some(&BASE_REGION),
    alter_light_level: |_, _, _, global, _| *global = 0,
    ..BASE_REGION
};

macro_rules! plain_region {
    ($(#[$m:meta])* $name:ident, $class:literal, $parent:ident) => {
        $(#[$m])*
        pub static $name: RegionType = RegionType { name: $class, parent: Some(&$parent), ..$parent };
    };
}

// TODO: Implement damage boost regions
plain_region!(CRYSTAL_FIELD_REGION, "CrystalFieldRegion", MONDAIN_REGION);
plain_region!(ICY_RIVER_REGION, "IcyRiverRegion", MONDAIN_REGION);
plain_region!(ACID_RIVER_REGION, "AcidRiverRegion", MONDAIN_REGION);
plain_region!(POISONED_TREE_REGION, "PoisonedTreeRegion", MONDAIN_REGION);
plain_region!(
    LOST_CITY_ENTRANCE_REGION,
    "LostCityEntranceRegion",
    MONDAIN_REGION
);
plain_region!(
    POISONED_CEMETERY_REGION,
    "PoisonedCemeteryRegion",
    MONDAIN_REGION
);
// TODO: Implement jail
plain_region!(WRONG_LEVEL3_REGION, "WrongLevel3Region", DUNGEON_REGION);
plain_region!(WRONG_JAIL_REGION, "WrongJailRegion", DUNGEON_REGION);
plain_region!(
    BLACKTHORN_DUNGEON_REGION,
    "BlackthornDungeonRegion",
    DUNGEON_REGION
);
plain_region!(DOOM_GUARDIAN_REGION, "DoomGuardianRegion", DUNGEON_REGION);
plain_region!(EXODUS_DUNGEON_REGION, "ExodusDungeonRegion", DUNGEON_REGION);
plain_region!(
    EXPLORING_DEEP_CREATURES_REGION,
    "ExploringDeepCreaturesRegion",
    DUNGEON_REGION
);
plain_region!(BATTLE_REGION, "BattleRegion", DUNGEON_REGION);
plain_region!(APPRENTICE_REGION, "ApprenticeRegion", BASE_REGION);
plain_region!(SEA_MARKET_REGION, "SeaMarketRegion", BASE_REGION);
plain_region!(TOK_BRIDGE_REGION, "ToKBridgeRegion", BASE_REGION);
plain_region!(UNDERWATER_REGION, "UnderwaterRegion", BASE_REGION);
plain_region!(
    COUSTEAU_PERRON_HOUSE_REGION,
    "CousteauPerronHouseRegion",
    GUARDED_REGION
);
// Not registered with the JSON loader, but the class exists.
plain_region!(
    BLACKTHORN_CASTLE_REGION,
    "BlackthornCastleRegion",
    GUARDED_REGION
);

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::map::MapId;
    use muo_server::region::{REGION_BASE, new_region, register};

    #[test]
    fn class_hierarchy_is_as_expected() {
        assert!(
            TOWN_REGION.is(&GUARDED_REGION)
                && TOWN_REGION.is(&BASE_REGION)
                && TOWN_REGION.is(&REGION_BASE)
        );
        assert!(NEW_MAGINCIA_REGION.is(&TOWN_REGION));
        assert!(
            TWISTED_WEALD_DESERT_REGION.is(&MONDAIN_REGION)
                && MONDAIN_REGION.is(&NO_TRAVEL_SPELLS_ALLOWED_REGION)
        );
        assert!(
            NO_TRAVEL_SPELLS_ALLOWED_REGION.is(&DUNGEON_REGION)
                && !DUNGEON_REGION.is(&GUARDED_REGION)
        );
        assert!(TOKUNO_DOCKS_REGION.is(&NO_HOUSING_GUARDED_REGION));
        assert!(WRONG_JAIL_REGION.is(&DUNGEON_REGION));
        assert_eq!(
            COUSTEAU_PERRON_HOUSE_REGION.name,
            "CousteauPerronHouseRegion"
        );
    }

    #[test]
    fn capability_flags() {
        let fam = |r: &RegionType| r.family::<BaseRegionType>().unwrap();
        assert!(fam(&BASE_REGION).young_protected);
        assert!(!fam(&DUNGEON_REGION).young_protected);
        assert!(
            !fam(&MONDAIN_REGION).young_protected,
            "Mondain regions are dungeons"
        );
        assert!(fam(&TOWN_REGION).young_protected);
    }

    fn world_with(ty: &'static RegionType) -> (World, RegionId, MobileId) {
        let mut w = muo_server::map::test_support::world();
        let r = new_region(
            &mut w,
            ty,
            Some("x"),
            MapId::FELUCCA,
            None,
            Some(50),
            vec![],
        );
        super::super::base_region::init_data(&mut w, r);
        register(&mut w, r);
        let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        (w, r, m)
    }

    #[test]
    fn travel_rules() {
        let (mut w, r, m) = world_with(&GREEN_ACRES_REGION);
        let p = Point3D::default();
        let t = TravelCheckType::RecallTo;
        assert!(
            !super::super::check_travel(&mut w, r, m, p, t)
                .unwrap()
                .allowed
        );
        w.mobile_mut(m).unwrap().access_level = AccessLevel::Counselor;
        assert!(
            super::super::check_travel(&mut w, r, m, p, t)
                .unwrap()
                .allowed
        );

        let (mut w, r, m) = world_with(&MONDAIN_REGION);
        assert!(
            !super::super::check_travel(&mut w, r, m, p, t)
                .unwrap()
                .allowed
        );
        w.mobile_mut(m).unwrap().access_level = AccessLevel::Counselor;
        assert!(
            super::super::check_travel(&mut w, r, m, p, t)
                .unwrap()
                .allowed
        );

        // A plain BaseRegion allows, and a child asks its parent first.
        let (mut w, parent, m) = world_with(&MONDAIN_REGION);
        let child = new_region(
            &mut w,
            &BASE_REGION,
            Some("c"),
            MapId::FELUCCA,
            Some(parent),
            None,
            vec![],
        );
        super::super::base_region::init_data(&mut w, child);
        assert!(
            !super::super::check_travel(&mut w, child, m, p, t)
                .unwrap()
                .allowed
        );
        // Not a BaseRegion class at all: no answer.
        let plain = new_region(
            &mut w,
            &REGION_BASE,
            None,
            MapId::FELUCCA,
            None,
            None,
            vec![],
        );
        assert!(super::super::check_travel(&mut w, plain, m, p, t).is_none());
    }

    #[test]
    fn housing_and_light() {
        let (mut w, r, m) = world_with(&DUNGEON_REGION);
        let rt = w.region(r).unwrap().ty;
        assert!(!(rt.allow_housing)(&w, r, m, Point3D::default()));
        let (mut g, mut p) = (5, 7);
        (rt.alter_light_level)(&mut w, r, m, &mut g, &mut p);
        assert_eq!((g, p), (crate::misc::light_cycle::DUNGEON_LEVEL, 7));
        let (mut w, r, m) = world_with(&TOMB_OF_KINGS_REGION);
        let rt = w.region(r).unwrap().ty;
        let (mut g, mut p) = (5, 7);
        (rt.alter_light_level)(&mut w, r, m, &mut g, &mut p);
        assert_eq!((g, p), (0, 7));
        let (w, r, m) = world_with(&NO_HOUSING_REGION);
        assert!((w.region(r).unwrap().ty.allow_housing)(
            &w,
            r,
            m,
            Point3D::default()
        ));
    }
}
