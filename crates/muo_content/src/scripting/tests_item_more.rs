//! Tests of the script functions for items (`api_item_more.rs`).

use super::tests::{call, script_world};

#[test]
fn place_flags_and_containers() {
    let mut w = script_world(
        "item",
        r#"
        fn check(cond, what) { if !cond { throw what; } }
        fn run() {
            let bag = spawn_item("Bag", 100, 200, 5, 0);
            let gold = create_item("Gold");
            gold.amount = 50;
            check(bag.is_container && !gold.is_container, "containers");
            check(bag.drop_in(gold), "drop in");
            check(bag.contents.len() == 1 && bag.contents[0] == gold, "contents");
            check(gold.x == 100 && gold.y == 200 && gold.map == 0 && !gold.on_ground, "inside: the bag's place");
            check(!gold.drop_in(bag), "gold is no container");
            gold.move_to(110, 210, 0);
            check(gold.on_ground && gold.x == 110 && bag.contents.is_empty(), "moved out");
            gold.move_to(10, 20, 0, 1);
            check(gold.map == 1, "to Trammel");
            gold.consume(20);
            check(gold.amount == 30, "consume");
            gold.movable = false; gold.loot_type = "Blessed"; gold.weight = 2.5;
            check(!gold.movable && gold.loot_type == "Blessed" && gold.weight == 2.5, "flags");
            bag.layer = "Backpack";
            check(bag.layer == "Backpack", "layer");
            bag.say("hello"); bag.say("hello", 33);
            true
        }
        "#,
    );
    assert_eq!(call(&mut w, "run", vec![]).as_bool(), Ok(true));
}
