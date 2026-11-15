//! `[Flippable]` as data: the `[Flippable]` table (`muo_content::items::flip`) agrees with what
//! the leaf modules recorded per class (the `FLIPPABLE` statics and the `flippable: Some((a, b))` of the weapon,
//! armor, clothing and jewel tables), and a flip follows `FlippableAttribute.Flip`.

use muo_content::items::flip::{flip, flip_ids};
use muo_server::World;
use muo_server::item::ItemType;
use muo_server::type_registry::TypeRegistry;

type Pairs = &'static [(&'static ItemType, i32, i32)];

fn registry() -> TypeRegistry {
    let mut t = TypeRegistry::default();
    crate::register(&mut t);
    muo_content::items::common::register(&mut t);
    t
}

#[test]
fn per_module_flippable_statics_agree_with_the_attribute_table() {
    let mut checked = 0;
    let pairs: Vec<(&'static str, Pairs)> = vec![
        ("body_parts", crate::body_parts::leaves::FLIPPABLE),
        ("bulletin", crate::bulletin::FLIPPABLE),
        (
            "champion_artifacts",
            crate::champion_artifacts::leaves::FLIPPABLE,
        ),
        ("construction", crate::construction::leaves::FLIPPABLE),
        ("containers", crate::containers::leaves::FLIPPABLE),
        (
            "decoration_artifacts",
            crate::decoration_artifacts::leaves::FLIPPABLE,
        ),
        ("deeds", crate::deeds::leaves::FLIPPABLE),
        ("farming", crate::farming::leaves::FLIPPABLE),
        ("food", crate::food::leaves::FLIPPABLE),
        ("lights", crate::lights::leaves::FLIPPABLE),
        ("minor_artifacts", crate::minor_artifacts::leaves::FLIPPABLE),
        (
            "new_haven_quest_rewards",
            crate::new_haven_quest_rewards::leaves::FLIPPABLE,
        ),
        ("plants_flowers", crate::plants_flowers::leaves::FLIPPABLE),
        ("resources", crate::resources::leaves::FLIPPABLE),
        ("traps", crate::traps::leaves::FLIPPABLE),
        ("treasure_chests", crate::treasure_chests::leaves::FLIPPABLE),
    ];
    for (module, list) in pairs {
        for (ty, a, b) in list {
            assert_eq!(
                flip_ids(ty),
                Some(&[*a, *b][..]),
                "{module}: {}",
                ty.full_name
            );
            checked += 1;
        }
    }
    for (ty, ids) in crate::misc::leaves::FLIPPABLE {
        assert_eq!(flip_ids(ty), Some(*ids), "misc: {}", ty.full_name);
        checked += 1;
    }
    assert!(checked > 40, "{checked} entries");
}

#[test]
fn a_subclass_is_not_flippable_unless_it_declares_the_attribute() {
    let types = registry();
    // `GetCustomAttributes(typeof(FlippableAttribute), false)`: a class below a flippable one has no attribute of
    // its own.
    let mut seen = 0;
    for ty in types.item_types() {
        if flip_ids(ty).is_none()
            && let Some(p) = ty.parent
            && flip_ids(p).is_some()
        {
            let mut w = crate::test_world();
            if let Some(c) = ty.construct {
                let id = c(&mut w);
                assert!(
                    !flip(&mut w, id) || muo_content::items::flip::is_own_flip(ty),
                    "{}",
                    ty.full_name
                );
            }
            seen += 1;
        }
    }
    println!("{seen} classes below a flippable one");
}

#[test]
fn flipping_cycles_through_every_id_of_the_attribute() {
    let types = registry();
    let mut w = crate::test_world();
    let mut n = 0;
    for ty in types.item_types() {
        let Some(ids) = flip_ids(ty) else { continue };
        let Some(construct) = ty.construct else {
            continue;
        };
        let id = construct(&mut w);
        if !ids.contains(&w.item(id).unwrap().item_id) {
            continue; // constructed with a graphic outside its pair (DecorativeBox) starts at index 0
        }
        let start = w.item(id).unwrap().item_id;
        for step in 0..ids.len() {
            assert!(flip(&mut w, id));
            let at = ids.iter().position(|v| *v == start).unwrap();
            assert_eq!(
                w.item(id).unwrap().item_id,
                ids[(at + step + 1) % ids.len()],
                "{}",
                ty.full_name
            );
        }
        n += 1;
    }
    assert!(n > 100, "{n} classes flipped");
}

#[test]
fn a_bare_flippable_runs_the_registered_flip_method() {
    let mut w = crate::test_world();
    crate::own_flips::configure(&mut w);
    let id = (crate::armor::special::LEATHER_GLOVES.construct.unwrap())(&mut w);
    let before = w.item(id).unwrap().item_id;
    assert!(flip(&mut w, id));
    assert_ne!(w.item(id).unwrap().item_id, before);
    assert!(flip(&mut w, id));
    assert_eq!(w.item(id).unwrap().item_id, before);
}

#[test]
fn bare_flips_registered_by_name_turn_the_graphic_and_the_light() {
    use muo_server::light_type::LightType;
    let mut w = crate::test_world();
    crate::own_flips::configure(&mut w);
    let run = |w: &mut World, name: &str, id| {
        let f = w
            .resources
            .get::<muo_content::items::flip::OwnFlips>()
            .unwrap()
            .0[name];
        f(w, id);
    };
    // Futon: 0x295C <-> 0x295D.
    let futon = w.new_item(&muo_server::item::ITEM_BASE);
    muo_server::item::base::set_item_id(&mut w, futon, 0x295C);
    run(&mut w, "Server.Items.Futon", futon);
    assert_eq!(w.item(futon).unwrap().item_id, 0x295D);
    run(&mut w, "Server.Items.Futon", futon);
    assert_eq!(w.item(futon).unwrap().item_id, 0x295C);
    // A graphic the switch doesn't list stays.
    muo_server::item::base::set_item_id(&mut w, futon, 0x1234);
    run(&mut w, "Server.Items.Futon", futon);
    assert_eq!(w.item(futon).unwrap().item_id, 0x1234);
    // WallTorch: the light turns with the graphic; a hanging lantern is always Circle300.
    let torch = w.new_item(&muo_server::item::ITEM_BASE);
    muo_server::item::base::set_item_id(&mut w, torch, 0xA05);
    muo_server::item::ops::set_light(&mut w, torch, LightType::WestBig);
    run(&mut w, "Server.Items.WallTorch", torch);
    assert_eq!(w.item(torch).unwrap().item_id, 0xA0A);
    assert_eq!(
        i32::from(w.item(torch).unwrap().direction.bits()),
        LightType::NorthBig as i32
    );
    let lantern = w.new_item(&muo_server::item::ITEM_BASE);
    muo_server::item::base::set_item_id(&mut w, lantern, 0x24C2);
    run(&mut w, "Server.Items.RedHangingLantern", lantern);
    assert_eq!(w.item(lantern).unwrap().item_id, 0x24C4);
    assert_eq!(
        i32::from(w.item(lantern).unwrap().direction.bits()),
        LightType::Circle300 as i32
    );
}
