//! The muo_engines, muo_multis and muo_content classes whose constructors moved to `Data/Scripts/classes` keep a
//! template, and the template builds what the old constructor built.

use muo_content::templates::{self, Kind};
use muo_server::World;

/// The class script files of the converted classes, their kind and how many classes each holds.
const FILES: &[(&str, Kind, usize)] = &[
    ("classes/mobiles/ml_quests/npcs.rhai", Kind::Mobile, 119),
    ("classes/mobiles/quests/creatures.rhai", Kind::Mobile, 8),
    ("classes/mobiles/ml_quests/npcs_more.rhai", Kind::Mobile, 16),
    ("classes/mobiles/quests/questers.rhai", Kind::Mobile, 28),
    ("classes/mobiles/factions.rhai", Kind::Mobile, 16),
    ("classes/mobiles/conpvp.rhai", Kind::Mobile, 1),
    ("classes/mobiles/khaldun/mobiles.rhai", Kind::Mobile, 4),
    ("classes/mobiles/ethics/mobiles.rhai", Kind::Mobile, 4),
    ("classes/items/ml_quests/items.rhai", Kind::Item, 36),
    ("classes/items/ml_quests/items2.rhai", Kind::Item, 27),
    ("classes/items/engines/misc.rhai", Kind::Item, 77),
    ("classes/items/multis/deeds.rhai", Kind::Item, 29),
    ("classes/items/multis/misc.rhai", Kind::Item, 11),
    ("classes/items/multis/boats.rhai", Kind::Item, 13),
    ("classes/items/multis/houses.rhai", Kind::Item, 13),
    ("classes/items/content/misc.rhai", Kind::Item, 89),
    ("classes/items/samples/leaves.rhai", Kind::Item, 10),
];

/// Templates only inherited by the converted classes.
const BASES: &[&str] = &[
    "Server.Multis.Deeds.HouseDeed",
    "Server.Multis.BaseBoatDeed",
    "Server.Engines.MLQuests.Items.BaseCraftmansSatchel",
    "Server.Multis.BaseDockedBoat",
];

fn world() -> World {
    let mut w = World::new();
    muo::register_content(&mut w.types);
    w.core.base_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    // the Configure phase, as startup: the `[set` properties scripts set, the templates, the scripts
    muo_server::type_registry::invoke_phase(&mut w, muo_server::type_registry::Phase::Configure);
    templates::reload(&mut w).unwrap();
    assert!(muo_content::scripting::reload_scripts(&mut w).is_empty());
    w
}

#[test]
fn every_converted_class_has_its_template() {
    let mut w = world();
    let t = templates::table(&mut w);
    for (file, kind, n) in FILES {
        let suffix = std::path::Path::new(file);
        let map = match kind {
            Kind::Mobile => &t.mobiles,
            Kind::Item => &t.items,
        };
        let in_file: Vec<_> = map.values().filter(|t| t.file.ends_with(suffix)).collect();
        assert_eq!(in_file.len(), *n, "{file}");
        for tmpl in in_file {
            let name = tmpl.type_name.as_str();
            let registered = match kind {
                Kind::Mobile => w.types.mobile_types().iter().any(|m| m.full_name == name),
                Kind::Item => w.types.item_types().iter().any(|i| i.full_name == name),
            };
            // An abstract class (no `ItemType` registered) only holds props its subclasses read.
            assert!(
                registered || BASES.contains(&name) || !tmpl.has_constructor(),
                "{name} is not a registered class"
            );
            let empty = tmpl.props.is_empty() && !tmpl.has_constructor() && tmpl.inherits.is_none();
            assert!(!empty, "{name} has an empty template");
        }
    }
}

#[test]
fn converted_classes_build_what_their_constructors_built() {
    use muo_content::factory::{make_item, make_mobile};
    let mut w = world();

    let ben = make_mobile(&mut w, "Server.Engines.MLQuests.Definitions.Ben").unwrap();
    let m = w.mobile(ben).unwrap();
    assert_eq!(m.title.as_deref(), Some("the Apprentice Necromancer"));
    assert_eq!(
        (m.hue, m.hair_item_id, m.facial_hair_hue),
        (0x83FD, 0x2048, 0x463)
    );
    assert_eq!((m.ty.name)(&w, ben).as_deref(), Some("Ben"));
    let shoes = m
        .items
        .iter()
        .copied()
        .find(|&i| w.item(i).unwrap().ty.full_name == "Server.Items.Shoes")
        .unwrap();
    assert_eq!(w.item(shoes).unwrap().hue, 0x901);

    let bauble = make_item(&mut w, "Server.Items.ABauble").unwrap();
    let i = w.item(bauble).unwrap();
    assert_eq!(
        (i.item_id, (i.ty.label_number)(&w, bauble)),
        (0x23B, 1073137)
    );

    let satchel = make_item(&mut w, "Server.Items.AndricSatchel").unwrap();
    let i = w.item(satchel).unwrap();
    assert_ne!(i.hue, 0);
    assert_eq!(i.items.len(), 2);

    let exiler = make_item(&mut w, "Server.Items.Exiler").unwrap();
    let d = muo_content::items::weapon::data(&w, exiler).unwrap();
    assert_eq!(
        d.slayer,
        muo_content::items::slayer::SlayerName::Exorcism as i32
    );
    assert_eq!(d.max_hit_points, 255);

    let tower = make_item(&mut w, "Server.Multis.Deeds.TowerDeed").unwrap();
    assert_eq!(muo_multis::deeds::multi_id(&w, tower), 0x7A);
    let ty = w.item(tower).unwrap().ty;
    assert_eq!((ty.label_number)(&w, tower), 1041222);

    let charley =
        make_mobile(&mut w, "Server.Engines.MLQuests.Definitions.GrandpaCharley").unwrap();
    let m = w.mobile(charley).unwrap();
    assert_eq!(m.facial_hair_item_id, 0x203E, "the on_spawn script ran");

    // the constructors with arguments hand them to the class's script
    {
        use muo_engines::tokuno::{artifacts, pigments};
        let made = |w: &mut World, kind, uses| {
            let p = pigments::pigments_new(w, kind, uses);
            (pigments::uses_remaining(w, p), w.item(p).unwrap().hue)
        };
        assert_eq!(made(&mut w, 3, None), (50, 0x4F2), "InvulnerabilityBlue");
        assert_eq!(made(&mut w, 0, None).0, 10);
        assert_eq!(made(&mut w, 11, None).0, 10);
        assert_eq!(made(&mut w, 3, Some(7)).0, 7);
        let urn = artifacts::ancient_urn_new(&mut w, "Akira");
        assert_eq!(artifacts::name(&w, urn).as_deref(), Some("Akira"));
        let bag = muo_engines::craft::salvage_bag::salvage_bag_new_hue(&mut w, 0x480);
        assert_eq!(w.item(bag).unwrap().hue, 0x480);
    }

    let horse = make_mobile(&mut w, "Server.Factions.FactionWarHorse").unwrap();
    assert_eq!(w.mobile(horse).unwrap().body.0, 0xE2);
    let knight = make_mobile(&mut w, "Server.Factions.FactionKnight").unwrap();
    assert_eq!(
        w.mobile(knight).unwrap().title.as_deref(),
        Some("the knight")
    );

    let boat = make_item(&mut w, "Server.Multis.LargeBoat").unwrap();
    assert_eq!(muo_multis::boats::layout(&w, boat).hold_distance, 5);

    let ingot = make_item(&mut w, "Server.Items.VeriteIngot").unwrap();
    assert_eq!(
        muo_content::items::resource::resource(&w, ingot),
        muo_content::craft::CraftResource::Verite
    );
}
