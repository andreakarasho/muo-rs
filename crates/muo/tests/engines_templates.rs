//! The muo_engines, muo_multis and muo_content classes whose constructors moved to `Data/Templates` keep a
//! template, and the template builds what the old constructor built.

use muo_content::templates::{self, Kind};
use muo_server::World;

/// The converted template files, their kind and how many classes each holds.
const FILES: &[(&str, Kind, usize)] = &[
    ("Mobiles/ml_quests/npcs.json", Kind::Mobile, 119),
    ("Mobiles/quests/creatures.json", Kind::Mobile, 8),
    ("Mobiles/quests/questers.json", Kind::Mobile, 20),
    ("Mobiles/khaldun/mobiles.json", Kind::Mobile, 4),
    ("Mobiles/ethics/mobiles.json", Kind::Mobile, 4),
    ("Items/ml_quests/items.json", Kind::Item, 36),
    ("Items/ml_quests/items2.json", Kind::Item, 27),
    ("Items/engines/misc.json", Kind::Item, 67),
    ("Items/multis/deeds.json", Kind::Item, 29),
    ("Items/multis/misc.json", Kind::Item, 4),
    ("Items/multis/houses.json", Kind::Item, 13),
    ("Items/content/misc.json", Kind::Item, 89),
    ("Items/samples/leaves.json", Kind::Item, 6),
];

/// Templates only inherited by the converted classes.
const BASES: &[&str] = &[
    "Server.Multis.Deeds.HouseDeed",
    "Server.Multis.BaseBoatDeed",
    "Server.Engines.MLQuests.Items.BaseCraftmansSatchel",
];

fn world() -> World {
    let mut w = World::new();
    muo::register_content(&mut w.types);
    w.core.base_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    templates::reload(&mut w).unwrap();
    assert!(muo_scripting::reload_scripts(&mut w).is_empty());
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
                registered
                    || BASES.contains(&name)
                    || (tmpl.item_steps.is_empty() && tmpl.mobile_steps.is_empty()),
                "{name} is not a registered class"
            );
            let empty = tmpl.props.is_empty()
                && tmpl.item_steps.is_empty()
                && tmpl.mobile_steps.is_empty()
                && tmpl.inherits.is_none();
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

    let ingot = make_item(&mut w, "Server.Items.VeriteIngot").unwrap();
    assert_eq!(
        muo_content::items::resource::resource(&w, ingot),
        muo_content::craft::CraftResource::Verite
    );
}
