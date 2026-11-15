//! Tests of what ties the hand-written `leaf_simple` classes together: the commodity and furniture tables, the
//! constructor overload tables behind `factory::make_item_args`, and the registration of every class.

use muo_content::props::Value;
use muo_server::World;
use muo_server::ids::ItemId;

/// A world whose type registry and `Configure` phases of this crate have run.
fn world() -> World {
    let mut w = World::new();
    crate::register(&mut w.types);
    muo_content::items::common::register(&mut w.types);
    muo_content::items::leaves::register(&mut w.types);
    muo_server::type_registry::invoke_phase(&mut w, muo_server::type_registry::Phase::Configure);
    muo_content::templates::table(&mut w);
    w
}

fn amount(w: &World, id: ItemId) -> i32 {
    w.item(id).unwrap().amount
}

#[test]
fn reagent_constructors_take_an_amount() {
    let mut w = world();
    let id =
        muo_content::factory::make_item_args(&mut w, "Server.Items.BlackPearl", &[Value::Int(5)])
            .unwrap();
    assert_eq!(amount(&w, id), 5);
    let id = muo_content::factory::make_item_args(&mut w, "Server.Items.BlackPearl", &[]).unwrap();
    assert_eq!(amount(&w, id), 1, "the default is 1");
}

#[test]
fn a_bag_of_reagents_holds_eight_stacks_of_the_amount() {
    let mut w = world();
    let bag = muo_content::factory::make_item_args(
        &mut w,
        "Server.Items.BagOfReagents",
        &[Value::Int(30)],
    )
    .unwrap();
    let items = w.item(bag).unwrap().items.clone();
    assert_eq!(items.len(), 8);
    assert!(items.iter().all(|&i| amount(&w, i) == 30));
    let bag = muo_content::factory::make_item_args(&mut w, "Server.Items.BagOfNecroReagents", &[])
        .unwrap();
    assert_eq!(w.item(bag).unwrap().items.len(), 5);
    assert!(
        w.item(bag)
            .unwrap()
            .items
            .iter()
            .all(|&i| amount(&w, i) == 50)
    );
}

#[test]
fn a_pitcher_of_water_is_full() {
    let mut w = world();
    let p = muo_content::factory::make_item_args(&mut w, "Server.Items.Pitcher", &[Value::Enum(5)])
        .unwrap();
    assert_eq!(crate::food::beverage::quantity(&w, p), 5);
    assert_eq!(
        crate::food::beverage::content(&w, p),
        crate::food::beverage::BeverageType::Water
    );
}

#[test]
fn character_creation_items_are_all_registered() {
    let w = world();
    for n in [
        "RedBook",
        "BagOfReagents",
        "BagOfNecroReagents",
        "Pitcher",
        "NewPlayerTicket",
        "IronGate",
        "DarkWoodHouseDoor",
        "BaseDoor",
        "BaseTrap",
        "FarmableCrop",
        "CookableFood",
        "Cloth",
        "SackFlour",
        "TreasureChestLevel1",
        "FillableContainer",
        "BaseWall",
        "Sign",
        "StrongBox",
        "Head",
        "Vines",
        "AnkhWest",
        "Tapestry1N",
    ] {
        assert!(
            w.types
                .find_type_by_full_name(&format!("Server.Items.{n}"), false)
                .is_some(),
            "{n}"
        );
    }
}

#[test]
fn the_door_classes_take_a_facing_in_add() {
    let mut w = world();
    let d =
        muo_content::factory::make_item_args(&mut w, "Server.Items.IronGate", &[Value::Enum(4)])
            .unwrap();
    assert_eq!(w.item(d).unwrap().item_id, 0x824 + 8);
    let d = muo_content::factory::make_item_args(
        &mut w,
        "Server.Items.GenericHouseDoor",
        &[
            Value::Enum(0),
            Value::Int(0x675),
            Value::Int(0xEC),
            Value::Int(0xF3),
        ],
    )
    .unwrap();
    assert_eq!(
        w.item(d).unwrap().item_id,
        0x675,
        "autoAdjust defaults to true: facing 0 adds nothing"
    );
}

#[test]
fn commodity_tables_have_a_consumer() {
    let mut w = world();
    let pearl = muo_content::factory::make_item(&mut w, "Server.Items.BlackPearl").unwrap();
    assert_eq!(crate::commodity::deedable(&w, pearl), Some(true));
    let feather = muo_content::factory::make_item(&mut w, "Server.Items.Feather").unwrap();
    assert_eq!(crate::commodity::deedable(&w, feather), Some(true));
    assert!(crate::commodity::implements(
        &crate::resources::leaves::FEATHER,
        "ICommodity"
    ));
    let bone = muo_content::factory::make_item(&mut w, "Server.Items.DaemonBone").unwrap();
    assert_eq!(
        crate::commodity::deedable(&w, bone),
        Some(muo_content::core::ml(&w))
    );
    let sword = muo_content::factory::make_item(&mut w, "Server.Items.Longsword").unwrap();
    assert_eq!(crate::commodity::deedable(&w, sword), None);
}

#[test]
fn every_furniture_class_is_a_flip_command_furniture() {
    for ty in crate::simple::furniture_tables() {
        assert!(
            muo_content::items::flip_table::FURNITURE.contains(&ty.full_name),
            "{} is [Furniture] but the flip command does not know it",
            ty.full_name
        );
    }
}
