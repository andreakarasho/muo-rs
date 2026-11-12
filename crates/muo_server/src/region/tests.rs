//! Tests of the region tree, `regions.json` and `WorldLocation`. Subclasses are tested
//! with the content that defines them.

use muo_data::expansion::Expansion;
use muo_primitives::{Point3D, Rectangle2D};

use super::*;
use crate::map::MapId;
use crate::map::test_support::{p, place_mobile, world};
use crate::region_json::load_regions_from_str;
use crate::world_location::WorldLocation;

const FEL: MapId = MapId::FELUCCA;

fn rect(x1: i32, y1: i32, x2: i32, y2: i32) -> Rectangle2D {
    Rectangle2D::new(x1, y1, x2 - x1, y2 - y1)
}

static TOWN: RegionType = RegionType {
    name: "TownRegion",
    parent: Some(&REGION_BASE),
    from_json: town_json,
    ..REGION_BASE
};

fn town_json(w: &mut crate::World, id: RegionId, o: &RegionJson) -> Result<(), String> {
    let name = o.get("Entrance").map(|v| v.to_string()).unwrap_or_default();
    w.region_mut(id).ok_or("gone")?.ext = crate::Ext::new(name);
    Ok(())
}

static NO_SPAWN: RegionType = RegionType {
    name: "NoSpawn",
    parent: Some(&REGION_BASE),
    allow_spawn: |_, _| false,
    ..REGION_BASE
};

#[test]
fn defaults_and_the_default_region() {
    let w = world();
    let d = default_region(&w, Some(FEL));
    assert_ne!(d, RegionId(0));
    assert!(is_default(&w, d));
    let r = w.region(d).unwrap();
    assert_eq!(
        (r.priority, r.child_level, r.dynamic, r.registered),
        (0, 0, true, false)
    );
    assert_eq!(find(&w, p(10, 10, 0), Some(FEL)), d);
    assert_eq!(
        find(&w, p(10, 10, 0), None),
        default_region(&w, Some(MapId::INTERNAL))
    );
    assert_eq!(region_to_string(&w, d), "Region");
}

#[test]
fn find_prefers_children_then_priority_and_unregister_forgets() {
    let mut w = world();
    let town = new_base_region(&mut w, Some("Town"), FEL, 50, &[rect(100, 100, 200, 200)]);
    let inn = new_region(
        &mut w,
        &REGION_BASE,
        Some("Inn"),
        FEL,
        Some(town),
        None,
        vec![convert_to_3d(rect(120, 120, 140, 140))],
    );
    let wild = new_base_region(&mut w, Some("Wild"), FEL, 10, &[rect(0, 0, 1000, 1000)]);
    for r in [wild, town, inn] {
        register(&mut w, r);
    }
    assert_eq!(w.region(inn).unwrap().child_level, 1);
    assert_eq!(
        w.region(inn).unwrap().priority,
        50,
        "a child starts at its parent's priority"
    );
    assert_eq!(w.region(town).unwrap().children, vec![inn]);

    assert_eq!(
        find(&w, p(130, 130, 0), Some(FEL)),
        inn,
        "deeper child first"
    );
    assert_eq!(
        find(&w, p(110, 110, 0), Some(FEL)),
        town,
        "priority 50 over 10"
    );
    assert_eq!(find(&w, p(500, 500, 0), Some(FEL)), wild);
    assert_eq!(
        find(&w, p(5000, 500, 0), Some(FEL)),
        default_region(&w, Some(FEL))
    );

    // a dynamic region ties with the rest (all are dynamic here); register order breaks the tie
    assert_eq!(find_by_name(&w, "inn", FEL, true), Some(inn));
    assert_eq!(find_by_name(&w, "inn", FEL, false), None);
    assert_eq!(find_by_name(&w, " ", FEL, false), None);
    assert_eq!(w.maps.get(FEL).unwrap().regions.get("TOWN"), Some(&town));

    assert!(is_child_of(&w, inn, town) && is_part_of(&w, inn, inn) && !is_child_of(&w, town, inn));
    assert!(is_part_of_name(&w, inn, "town", false) && !is_part_of_name(&w, inn, "town", true));
    assert!(is_part_of_type(&w, inn, &REGION_BASE) && !is_part_of_type(&w, inn, &TOWN));

    unregister(&mut w, inn);
    assert_eq!(find(&w, p(130, 130, 0), Some(FEL)), town);
    assert!(w.region(town).unwrap().children.is_empty());
    assert_eq!(w.maps.region_list, vec![wild, town]);
    unregister(&mut w, inn); // idempotent
    register(&mut w, town); // idempotent
    assert_eq!(w.maps.region_list, vec![wild, town]);
}

#[test]
fn the_sector_list_is_sorted_by_compare() {
    let mut w = world();
    let low = new_base_region(&mut w, Some("low"), FEL, 10, &[rect(0, 0, 16, 16)]);
    let high = new_base_region(&mut w, Some("high"), FEL, 90, &[rect(0, 0, 16, 16)]);
    let mid = new_base_region(&mut w, Some("mid"), FEL, 50, &[rect(0, 0, 16, 16)]);
    for r in [low, high, mid] {
        register(&mut w, r);
    }
    assert_eq!(
        w.maps.get(FEL).unwrap().sector(0, 0).regions,
        vec![high, mid, low]
    );
    // a static (non-dynamic) region sorts after every dynamic one
    let stat = new_base_region(&mut w, Some("static"), FEL, 99, &[rect(0, 0, 16, 16)]);
    w.region_mut(stat).unwrap().dynamic = false;
    register(&mut w, stat);
    assert_eq!(
        w.maps.get(FEL).unwrap().sector(0, 0).regions,
        vec![high, mid, low, stat]
    );
}

#[test]
fn registering_across_sectors_lists_each_once() {
    let mut w = world();
    // two overlapping rectangles over the same four sectors (an area's end is inclusive: 32 would reach sector 2)
    let r = new_region(
        &mut w,
        &REGION_BASE,
        Some("wide"),
        FEL,
        None,
        None,
        vec![
            convert_to_3d(rect(0, 0, 31, 31)),
            convert_to_3d(rect(8, 8, 24, 24)),
        ],
    );
    register(&mut w, r);
    assert_eq!(
        w.region(r).unwrap().sectors,
        vec![(0, 0), (0, 1), (1, 0), (1, 1)]
    );
    unregister(&mut w, r);
    assert!(w.maps.get(FEL).unwrap().sector(1, 1).regions.is_empty());
}

#[test]
fn virtuals_delegate_to_the_parent() {
    let mut w = world();
    let outer = new_region(
        &mut w,
        &NO_SPAWN,
        Some("outer"),
        FEL,
        None,
        None,
        vec![convert_to_3d(rect(0, 0, 64, 64))],
    );
    let inner = new_region(
        &mut w,
        &REGION_BASE,
        Some("inner"),
        FEL,
        Some(outer),
        None,
        vec![convert_to_3d(rect(8, 8, 16, 16))],
    );
    assert!(!allow_spawn_of(&w, outer));
    assert!(!allow_spawn_of(&w, inner), "AllowSpawn asks the parent");
    let free = new_base_region(&mut w, Some("free"), FEL, 5, &[rect(0, 0, 4, 4)]);
    assert!(allow_spawn_of(&w, free));
    assert!(!(w.region(inner).unwrap().ty.accepts_spawns_from)(
        &w, inner, free
    ));
    assert!((w.region(free).unwrap().ty.accepts_spawns_from)(
        &w, free, free
    ));

    let m = place_mobile(&mut w, FEL, p(1, 1, 0));
    assert!(
        (REGION_BASE.allow_harmful)(&mut w, free, m, m),
        "no handler: true"
    );
    assert!((REGION_BASE.allow_beneficial)(&mut w, free, m, m));
    w.maps.settings.allow_harmful_handler = Some(|_, _, _| false);
    assert!(
        !(REGION_BASE.allow_harmful)(&mut w, inner, m, m),
        "the handler at the root"
    );
    assert_eq!(
        (REGION_BASE.get_logout_delay)(&w, inner, m),
        muo_primitives::TimeSpan::from_mins(5)
    );
    w.mobile_mut(m).unwrap().access_level = crate::mobile::AccessLevel::GameMaster;
    assert_eq!(
        (REGION_BASE.get_logout_delay)(&w, inner, m),
        muo_primitives::TimeSpan::ZERO
    );
    assert_eq!(
        (REGION_BASE.get_resource)(&w, inner, "Server.Items.IronOre"),
        "Server.Items.IronOre"
    );
    assert_eq!((REGION_BASE.default_music)(&w, inner), MusicName::Invalid);
}

#[test]
fn region_change_walks_enter_and_exit_up_the_chain() {
    use std::sync::Mutex;
    static LOG: Mutex<Vec<(char, u32)>> = Mutex::new(Vec::new());
    static LOGGING: RegionType = RegionType {
        name: "Logging",
        parent: Some(&REGION_BASE),
        on_enter: |_, r, _| {
            LOG.lock().unwrap().push(('e', r.0));
        },
        on_exit: |_, r, _| {
            LOG.lock().unwrap().push(('x', r.0));
        },
        ..REGION_BASE
    };

    let mut w = world();
    let a = new_region(
        &mut w,
        &LOGGING,
        Some("a"),
        FEL,
        None,
        None,
        vec![convert_to_3d(rect(0, 0, 64, 64))],
    );
    let b = new_region(
        &mut w,
        &LOGGING,
        Some("b"),
        FEL,
        Some(a),
        None,
        vec![convert_to_3d(rect(0, 0, 16, 16))],
    );
    let c = new_region(
        &mut w,
        &LOGGING,
        Some("c"),
        FEL,
        None,
        None,
        vec![convert_to_3d(rect(100, 100, 164, 164))],
    );
    for r in [a, b, c] {
        register(&mut w, r);
    }
    let m = place_mobile(&mut w, FEL, p(1, 1, 0));
    LOG.lock().unwrap().clear();
    on_region_change(&mut w, m, RegionId(0), b);
    // The deeper region is entered first, then its parent
    assert_eq!(*LOG.lock().unwrap(), [('e', b.0), ('e', a.0)]);

    LOG.lock().unwrap().clear();
    on_region_change(&mut w, m, b, c);
    // b (level 1) exits first, then a and c at level 0 move together: exit a, enter c
    assert_eq!(*LOG.lock().unwrap(), [('x', b.0), ('x', a.0), ('e', c.0)]);

    // update_mobile_region moves a mobile into the region its tile is in
    w.mobile_mut(m).unwrap().region = RegionId(0);
    update_mobile_region(&mut w, m);
    assert_eq!(w.mobile(m).unwrap().region, b);
}

#[test]
fn registering_a_region_updates_mobiles_standing_in_it() {
    let mut w = world();
    let m = place_mobile(&mut w, FEL, p(5, 5, 0));
    let r = new_base_region(&mut w, Some("here"), FEL, 50, &[rect(0, 0, 16, 16)]);
    // a new mobile starts in Map.Internal.DefaultRegion
    assert_eq!(
        w.mobile(m).unwrap().region,
        default_region(&w, Some(MapId::INTERNAL))
    );
    register(&mut w, r);
    assert_eq!(w.mobile(m).unwrap().region, r);
    assert_eq!(get_mobiles(&w, r), vec![m]);
    assert_eq!(get_mobile_count(&w, r), 1);
    assert_eq!(get_player_count(&w, r), 0);
    unregister(&mut w, r);
    assert_eq!(w.mobile(m).unwrap().region, default_region(&w, Some(FEL)));
}

#[test]
fn can_move_asks_each_region_up_to_the_root() {
    static CLOSED: RegionType = RegionType {
        name: "Closed",
        parent: Some(&REGION_BASE),
        on_move_into: |_, _, _, _, _, _| false,
        ..REGION_BASE
    };
    let mut w = world();
    let r = new_region(
        &mut w,
        &CLOSED,
        Some("closed"),
        FEL,
        None,
        None,
        vec![convert_to_3d(rect(0, 0, 16, 16))],
    );
    register(&mut w, r);
    let m = place_mobile(&mut w, FEL, p(100, 100, 0));
    let d = muo_primitives::Direction::NORTH;
    assert!(!can_move(
        &mut w,
        m,
        d,
        p(5, 5, 0),
        p(100, 100, 0),
        Some(FEL)
    ));
    assert!(can_move(
        &mut w,
        m,
        d,
        p(200, 200, 0),
        p(100, 100, 0),
        Some(FEL)
    ));
}

// ---- regions.json --------------------------------------------------------------------------------------

const JSON: &str = r#"[
  { "$type": "TownRegion", "Entrance": {"x": 1, "y": 2, "z": 3}, "Map": "Felucca", "Name": "Britain",
    "Priority": 50, "Area": [{"x1": 1400, "y1": 1500, "x2": 1500, "y2": 1700}], "GoLocation": {"x": 1496, "y": 1628, "z": 10},
    "Music": "Britain1" },
  { "$type": "TownRegion", "Map": "Felucca", "Name": "Bank", "Parent": {"Name": "Britain", "Map": "Felucca"},
    "Priority": 60, "Area": [{"x1": 1430, "y1": 1600, "x2": 1440, "y2": 1610}] },
  { "Map": 1, "Name": "Plain", "Priority": 1, "Area": [{"x1": 0, "y1": 0, "x2": 16, "y2": 16}] },
  { "$type": "TownRegion", "Map": "Felucca", "Name": "Later", "Priority": 50, "MinExpansion": "SA",
    "Area": [{"x1": 10, "y1": 10, "x2": 20, "y2": 20}] },
  { "$type": "TownRegion", "Map": "Felucca", "Name": "Gone", "Priority": 50, "MaxExpansion": "T2A",
    "Area": [{"x1": 10, "y1": 10, "x2": 20, "y2": 20}] }
]"#;

#[test]
fn regions_json_builds_registers_and_filters_by_expansion() {
    let mut w = world();
    w.maps.region_types.register(&TOWN).unwrap();
    assert!(
        w.maps.region_types.register(&TOWN).is_err(),
        "registering twice is an error"
    );

    let n = load_regions_from_str(&mut w, JSON, Expansion::AOS).unwrap();
    assert_eq!(n, 3, "Later needs SA, Gone ends at T2A");

    let britain = find_by_name(&w, "Britain", FEL, false).unwrap();
    let bank = find_by_name(&w, "Bank", FEL, false).unwrap();
    let r = w.region(britain).unwrap();
    assert_eq!(
        (r.priority, r.child_level, r.music),
        (50, 0, MusicName::Britain1)
    );
    assert_eq!(r.go_location, Point3D::new(1496, 1628, 10));
    assert_eq!(r.area[0].start.z, -128);
    assert_eq!(r.ty.name, "TownRegion");
    assert!(
        r.ext.get::<String>().is_some_and(|s| s.contains("\"x\":1")),
        "extra properties go to the type"
    );

    let b = w.region(bank).unwrap();
    assert_eq!(
        (b.parent, b.child_level, b.priority),
        (Some(britain), 1, 60)
    );
    assert_eq!(
        b.music,
        MusicName::Invalid,
        "a child that names no music keeps Invalid"
    );
    assert_eq!(find(&w, p(1435, 1605, 0), Some(FEL)), bank);

    let plain = find_by_name(&w, "Plain", MapId::TRAMMEL, false).unwrap();
    assert_eq!(w.region(plain).unwrap().ty.name, "Region");
    assert!(
        find_by_name(&w, "Later", FEL, false).is_none()
            && find_by_name(&w, "Gone", FEL, false).is_none()
    );
    assert_eq!(w.maps.region_list.len(), 3);

    let n = load_regions_from_str(&mut w, JSON, Expansion::EJ).unwrap();
    assert_eq!(
        n, 4,
        "Later is in, Gone is out; names already taken are only warned about"
    );
}

#[test]
fn regions_json_errors() {
    let mut w = world();
    for bad in [
        r#"[{"$type": "Nope", "Map": "Felucca", "Area": []}]"#,
        r#"[{"Map": "Nowhere", "Area": []}]"#,
        r#"[{"Area": []}]"#,
        r#"[{"Map": "Felucca"}]"#,
        r#"[{"Map": "Felucca", "Area": [], "Music": "NotMusic"}]"#,
        r#"{"not": "a list"}"#,
    ] {
        assert!(
            load_regions_from_str(&mut w, bad, Expansion::EJ).is_err(),
            "{bad}"
        );
    }
    assert!(w.maps.region_list.is_empty());
}

// ---- WorldLocation -------------------------------------------------------------------------------------

#[test]
fn world_location_text_and_json() {
    let w = world();
    let loc = WorldLocation::new(1, 2, 3, Some(FEL));
    assert_eq!(loc.format(&w.maps), "(1, 2, 3) [Felucca]");
    assert_eq!(
        WorldLocation::default().format(&w.maps),
        "(0, 0, 0) [(-null-)]"
    );
    assert_eq!(WorldLocation::ZERO.format(&w.maps), "(0, 0, 0) [Internal]");

    assert_eq!(
        WorldLocation::try_parse(&w.maps, " (1, 2, 3) [Felucca] "),
        Some(loc)
    );
    assert_eq!(
        WorldLocation::try_parse(&w.maps, "(1, 2, 3) [1]"),
        Some(WorldLocation::new(1, 2, 3, Some(MapId(1))))
    );
    assert_eq!(
        WorldLocation::try_parse(&w.maps, "(1, 2, 3) [(-null-)]"),
        Some(WorldLocation::new(1, 2, 3, None))
    );
    for bad in [
        "(1, 2, 3)",
        "(1, 2, 3) [Mars]",
        "[Felucca]",
        "(1, 2) [Felucca]",
    ] {
        assert_eq!(WorldLocation::try_parse(&w.maps, bad), None, "{bad}");
    }

    let j = |s: &str| WorldLocation::from_json(&w.maps, &serde_json::from_str(s).unwrap());
    assert_eq!(j(r#""(1, 2, 3) [Felucca]""#), Ok(loc));
    assert_eq!(j(r#"[1, 2, 3, "Felucca"]"#), Ok(loc));
    assert!(
        j(r#"[1, 2, 3, 0]"#).is_err(),
        "a numeric map in an array fails"
    );
    assert_eq!(j(r#"{"x": 1, "y": 2, "z": 3, "map": "Felucca"}"#), Ok(loc));
    assert_eq!(j(r#"{"loc": [1, 2, 3], "map": 0}"#), Ok(loc));
    assert!(j(r#"{"x": 1, "loc": [1, 2, 3], "map": 0}"#).is_err());
    assert!(j(r#"{"x": 1, "y": 2, "z": 3}"#).is_err());
    assert_eq!(loc.to_json(&w.maps).to_string(), r#"[1,2,3,"Felucca"]"#);
}

#[test]
fn world_location_operators_keep_their_quirks() {
    let a = WorldLocation::new(1, 2, 3, Some(MapId(0)));
    let b = WorldLocation::new(1, 2, 3, Some(MapId(1)));
    assert!(a != b && !a.op_ne(&b), "!= needs both parts to differ");
    assert!(a.op_ne(&WorldLocation::new(9, 9, 9, Some(MapId(1)))));
    assert!(WorldLocation::new(5, 5, 5, Some(MapId(0))).op_gt(&a));
    assert!(
        !WorldLocation::new(5, 5, 5, Some(MapId(1))).op_gt(&a),
        "maps must match"
    );
    // `<` compares z with `>`: (0, 0, 9) < (1, 1, 1)
    assert!(
        WorldLocation::new(0, 0, 9, Some(MapId(0))).op_lt(&WorldLocation::new(
            1,
            1,
            1,
            Some(MapId(0))
        ))
    );
    assert_eq!(a.compare_to(&b), std::cmp::Ordering::Less);
    assert_eq!(
        WorldLocation::new(0, 9, 9, None).compare_to(&a),
        std::cmp::Ordering::Less
    );

    let mut w = world();
    let m = place_mobile(&mut w, FEL, p(1, 2, 3));
    let e = Some(crate::ids::EntityId::Mobile(m));
    assert!(a.eq_entity(&w, e) && !a.eq_entity(&w, None) && !a.ne_entity(&w, None));
    assert!(!b.eq_entity(&w, e));
}

// ---- real data -----------------------------------------------------------------------------------------

fn distribution() -> std::path::PathBuf {
    std::env::var("MUO_DISTRIBUTION").unwrap_or_default().into()
}

#[test]
#[ignore = "reads Distribution/Data (set MUO_DISTRIBUTION; read-only)"]
fn distribution_map_definitions_and_regions_load() {
    let mut w = crate::World::new();
    let n = crate::map::loader::configure(&mut w, &distribution()).unwrap();
    assert_eq!(n, 7);
    assert_eq!(w.maps.parse("Trammel"), Some(MapId::TRAMMEL));
    assert_eq!(w.maps.parse("Internal"), Some(MapId::INTERNAL));

    // every `$type` of the shipped file; muo_content registers the real ones
    for name in [
        "BaseRegion",
        "TownRegion",
        "DungeonRegion",
        "NoHousingRegion",
        "GuardedRegion",
        "JailRegion",
        "GreenAcresRegion",
    ] {
        let ty: &'static RegionType = Box::leak(Box::new(RegionType {
            name: Box::leak(name.to_string().into_boxed_str()),
            parent: Some(&REGION_BASE),
            ..REGION_BASE
        }));
        w.maps.region_types.register(ty).unwrap();
    }
    let n = crate::region_json::load_regions(&mut w, &distribution(), Expansion::EJ).unwrap();
    assert!(n > 300, "regions.json has 398 entries, {n} registered");
    assert!(w.maps.region_list.len() == n);
    let britain = find_by_name(&w, "Britain", FEL, false).expect("Britain");
    assert!(w.region(britain).unwrap().registered);
    let found = find(&w, Point3D::new(1496, 1628, 10), Some(FEL));
    assert!(
        is_part_of(&w, found, britain),
        "a point in Britain is in Britain or a child of it"
    );
}
