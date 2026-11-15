//! Tests of the script functions for the world (`api_world.rs`).

use super::tests::{call, script_world};

#[test]
fn lookups_spawning_maps_and_time() {
    let mut w = script_world(
        "world",
        r#"
        fn check(cond, what) { if !cond { throw what; } }
        fn run() {
            let gold = spawn_item("Gold", 1000, 1000, 0, 0);
            check(type_of(gold) == "Item", "spawn_item by short name");
            check(gold.x == 1000 && gold.y == 1000 && gold.map == 0, "placed");
            check(find_item(gold.serial) == gold, "find_item");
            check(items_in_range(1000, 1000, 0, 2).len() == 1, "items_in_range");
            check(items_in_range(1000, 1000, 1, 2).is_empty(), "another map");
            check(type_of(spawn_item("NoSuchThing", 1, 1, 0, 0)) == "()", "unknown class");
            let loose = create_item("Server.Items.Gold");
            check(type_of(loose) == "Item" && loose.map == -1, "create_item is nowhere");
            check(type_of(find_mobile(123456)) == "()", "no such mobile");
            check(map_id("Trammel") == 1 && map_name(0) == "Felucca" && map_id("Nowhere") == -1, "maps");
            check(ground_z(10, 10, 0) == 0, "an empty map is flat");
            check(server_time() != "" && ticks() >= 0, "time");
            check(players().is_empty() && online_count() == 0, "nobody online");
            broadcast("hello");
            broadcast("hello", 33);
            broadcast_staff("staff only");
            true
        }
        "#,
    );
    assert_eq!(call(&mut w, "run", vec![]).as_bool(), Ok(true));
}
