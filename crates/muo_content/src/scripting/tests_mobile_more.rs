//! Tests of the script functions for creatures and players (`api_mobile_more.rs`).

use super::tests::{call, script_world};

#[test]
fn stats_flags_health_and_speech() {
    let mut w = script_world(
        "mobile",
        r#"
        fn check(cond, what) { if !cond { throw what; } }
        fn run(m) {
            m.str = 100;
            m.fame = 1200; m.karma = -300; m.kills = 6; m.title = "the brave";
            check(m.fame == 1200 && m.karma == -300 && m.title == "the brave", "fame, karma, title");
            check(m.murderer, "5+ kills");
            m.hidden = true; m.blessed = true; m.female = true;
            check(m.hidden && m.blessed && m.female, "flags");
            m.direction = "West";
            check(m.direction == "West", "direction");
            m.hits = 10;
            m.heal(15);
            check(m.hits == 25, `heal: ${m.hits}`);
            check(m.alive, "alive");
            check(type_of(m.combatant) == "()", "no combatant");
            m.move_to(1500, 1600, 0, 1);
            check(m.map == 1 && m.x == 1500, "moved to Trammel");
            check(m.access_level == "Player" && !m.online && m.account == "", "a plain mobile");
            m.emote("waves"); m.yell("HEY"); m.whisper("psst"); m.animate(17, 7);
            true
        }
        "#,
    );
    let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    assert_eq!(
        call(&mut w, "run", vec![super::api::mobile(m)]).as_bool(),
        Ok(true)
    );
}

#[test]
fn backpacks_hold_count_and_consume() {
    let mut w = script_world(
        "pack",
        r#"
        fn check(cond, what) { if !cond { throw what; } }
        fn run(m) {
            check(type_of(m.backpack) == "()", "no pack yet");
            let pack = create_item("Backpack");
            check(m.equip(pack), "equip a backpack");
            check(m.backpack == pack && m.equipped("Backpack") == pack, "worn");
            let gold = create_item("Gold");
            gold.amount = 100;
            m.give(gold);
            check(m.count("Gold") == 100, `count: ${m.count("Gold")}`);
            check(m.consume("Gold", 30) && m.count("Gold") == 70, "consume");
            check(!m.consume("Gold", 500) && m.count("Gold") == 70, "not enough takes nothing");
            check(m.worn.len() == 1, "worn list");
            true
        }
        "#,
    );
    let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    assert_eq!(
        call(&mut w, "run", vec![super::api::mobile(m)]).as_bool(),
        Ok(true)
    );
}
