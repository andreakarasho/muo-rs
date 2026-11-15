use std::path::PathBuf;

use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::MobileId;

use muo_server::mobile::AccessLevel;
use muo_server::script_hooks::{self, ScriptArg, ScriptHook};
use muo_server::world::World;

use super::*;

/// A server folder whose `Data/Scripts` declares the mobile class `Test.Scripted` and item class
/// `Test.ScriptedItem`, both bound to `a.rhai`, plus `scripts`.
fn server(name: &str, scripts: &[(&str, &str)]) -> PathBuf {
    let base = std::env::temp_dir().join(format!("muo-scripting-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("Data/Scripts")).unwrap();
    write(
        &base,
        "test_classes.rhai",
        r#"
        mobile("Test.Scripted", |c| { c.extends = "Server.Mobile"; c.script = "a.rhai"; });
        item("Test.ScriptedItem", |c| { c.extends = "Server.Item"; c.script = "a.rhai"; });
        "#,
    );
    for (f, text) in scripts {
        write(&base, f, text);
    }
    base
}

fn write(base: &std::path::Path, file: &str, text: &str) {
    std::fs::write(base.join("Data/Scripts").join(file), text).unwrap();
}

fn world(base: PathBuf) -> (World, Vec<String>) {
    let mut w = World::new();
    w.core.base_directory = base; // `Server.Mobile` and `Server.Item` are registered by `World::new`
    crate::scripting::register(&mut w.types);
    let errors = reload_scripts(&mut w);
    (w, errors)
}

fn spawn(w: &mut World) -> MobileId {
    let id = crate::factory::make_mobile(w, "Test.Scripted").unwrap();
    let ty = w.mobile(id).unwrap().ty;
    (ty.set_str)(w, id, 100);
    muo_server::mobile::part1::props_b::set_hits(w, id, 60);
    id
}

fn hits(w: &World, id: MobileId) -> i32 {
    w.mobile(id).unwrap().hits
}

fn name(w: &World, id: MobileId) -> String {
    w.mobile(id).unwrap().name.clone().unwrap_or_default()
}

fn damage(w: &mut World, id: MobileId, amount: i32) {
    let ty = w.mobile(id).unwrap().ty;
    (ty.damage)(w, id, amount, None, true, false);
}

#[test]
fn hooks_fire_and_suppress_the_default_and_reload_applies() {
    let base = server(
        "hooks",
        &[(
            "a.rhai",
            r#"
            fn on_spawn(e) { e.name = "spawned"; }
            fn on_damage(m, amount, from) { m.name = `took ${amount}`; true }
            fn on_use(item, from) { from.name = "used"; item.delete(); true }
            fn later(m) { m.name = "later " + random(3, 3); }
            "#,
        )],
    );
    let (mut w, errors) = world(base.clone());
    assert!(errors.is_empty(), "{errors:?}");
    let id = spawn(&mut w);
    assert_eq!(name(&w, id), "spawned");

    damage(&mut w, id, 10);
    assert_eq!((hits(&w, id), name(&w, id).as_str()), (60, "took 10"));

    let it = crate::factory::make_item(&mut w, "Test.ScriptedItem").unwrap();
    assert!(script_hooks::item(
        &mut w,
        it,
        ScriptHook::Use,
        &[ScriptArg::Mobile(Some(id))]
    ));
    assert_eq!(name(&w, id), "used");
    assert!(w.item(it).is_none_or(|i| i.deleted()));

    fire_timer(&mut w, "a.rhai", "later", api::mobile(id));
    assert_eq!(name(&w, id), "later 3");

    // the hook is gone after a reload: the default runs, without a restart
    write(&base, "a.rhai", "fn on_damage(m, amount, from) { false }");
    assert!(reload_scripts(&mut w).is_empty());
    damage(&mut w, id, 10);
    assert_eq!(hits(&w, id), 50);
    write(&base, "a.rhai", "fn on_think(m) { true }");
    assert!(reload_scripts(&mut w).is_empty());
    damage(&mut w, id, 10);
    assert_eq!(hits(&w, id), 40);
}

#[test]
fn a_runaway_script_hits_the_limit_and_the_default_runs() {
    let base = server(
        "runaway",
        &[("a.rhai", "fn on_damage(m, amount, from) { loop { } }")],
    );
    let (mut w, _) = world(base);
    let h = host(&w).unwrap();
    let opts = CallFnOptions::new().eval_ast(false);
    let args = (Dynamic::UNIT, 1_i64, Dynamic::UNIT);
    let e = h
        .engine
        .call_fn_with_options::<Dynamic>(
            opts,
            &mut Scope::new(),
            &h.scripts[0].ast,
            "on_damage",
            args,
        )
        .unwrap_err();
    assert!(
        matches!(*e, rhai::EvalAltResult::ErrorTooManyOperations(_)),
        "{e}"
    );

    let id = spawn(&mut w);
    damage(&mut w, id, 10);
    assert_eq!(hits(&w, id), 50);
    // and again: nothing is left stuck (the nesting depth went back down)
    damage(&mut w, id, 10);
    assert_eq!(hits(&w, id), 40);
}

#[test]
fn nested_hooks_stop_at_the_depth_limit() {
    let base = server(
        "nested",
        &[(
            "a.rhai",
            "fn on_damage(m, amount, from) { damage(m, amount, ()); true }",
        )],
    );
    let (mut w, _) = world(base);
    let id = spawn(&mut w);
    damage(&mut w, id, 1);
    // the innermost call is refused, so its default applies once
    assert_eq!(hits(&w, id), 59);
}

#[test]
fn a_compile_error_fails_the_reload_and_keeps_what_was_loaded() {
    let base = server(
        "compile",
        &[("a.rhai", "fn on_damage(m, amount, from) { true }")],
    );
    let (mut w, errors) = world(base.clone());
    assert!(errors.is_empty(), "{errors:?}");
    // scripts hold classes now: a broken file must not leave classes half loaded
    write(&base, "bad.rhai", "fn broken( {");
    let errors = reload_scripts(&mut w);
    assert!(errors.iter().any(|e| e.contains("bad.rhai")), "{errors:?}");
    let id = spawn(&mut w);
    damage(&mut w, id, 10);
    assert_eq!(hits(&w, id), 60, "the previous scripts still run");
}

#[test]
fn script_commands_register_and_unregister_on_reload() {
    let base = server(
        "commands",
        &[(
            "a.rhai",
            "// Greets.\n// access: Player\nfn command_hello(m, args) { m.name = \"hello \" + args; }\n\
             fn command_staff(m, args) { }",
        )],
    );
    let (mut w, errors) = world(base.clone());
    assert!(errors.is_empty(), "{errors:?}");
    let id = spawn(&mut w);
    let entry = |w: &mut World, k: &str| {
        w.resources
            .get_mut::<CommandSystem>()
            .entries
            .get(k)
            .cloned()
    };
    let hello = entry(&mut w, "HELLO").expect("[hello registered");
    assert_eq!(hello.access_level, AccessLevel::Player);
    assert_eq!(
        entry(&mut w, "STAFF").unwrap().access_level,
        AccessLevel::GameMaster
    );
    let mut args = CommandEventArgs::new(id, "hello".into(), "there".into(), vec!["there".into()]);
    (hello.handler)(&mut w, &mut args);
    assert_eq!(name(&w, id), "hello there");

    write(&base, "a.rhai", "fn command_bye(m, args) { }");
    assert!(reload_scripts(&mut w).is_empty());
    assert!(entry(&mut w, "HELLO").is_none());
    assert!(entry(&mut w, "BYE").is_some());
}

#[test]
fn unbound_classes_cost_no_script_call() {
    let base = server("unbound", &[]);
    let (mut w, errors) = world(base);
    assert_eq!(
        errors.len(),
        1,
        "the templates name a missing script: {errors:?}"
    );
    let id = spawn(&mut w);
    assert!(!script_hooks::mobile_has(&w, id, ScriptHook::Damage));
    damage(&mut w, id, 10);
    assert_eq!(hits(&w, id), 50);
}

#[test]
fn constructor_arguments_reach_on_spawn() {
    let base = server(
        "ctorargs",
        &[(
            "a.rhai",
            r#"fn on_spawn(e, hue, name) { e.hue = hue; e.name = name; }"#,
        )],
    );
    let (mut w, errors) = world(base);
    assert!(errors.is_empty(), "{errors:?}");
    let ty = match w.types.find_type_by_full_name("Test.ScriptedItem", false) {
        Some(muo_server::type_registry::EntityType::Item(t)) => t,
        _ => panic!("registered"),
    };
    let id = muo_server::item::ops::new_item_with_id(&mut w, ty, 0);
    templates::apply_item_with(
        &mut w,
        id,
        "Test.ScriptedItem",
        &[ScriptArg::Int(33), ScriptArg::Str("from the constructor")],
    );
    let i = w.item(id).unwrap();
    assert_eq!(
        (i.hue, i.name.as_deref()),
        (33, Some("from the constructor"))
    );
}

/// A server folder with only `Data/Scripts/classes/mobiles/t.json` = `json` and `Data/Scripts/c.rhai` = `rhai`.
/// A server folder whose `Data/Scripts/c.rhai` is `rhai`; `json` (the templates in their JSON step form, to
/// compare a script with) is kept next to it as `t.json`.
fn class_server(name: &str, json: &str, rhai: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("muo-classes-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("Data/Scripts")).unwrap();
    std::fs::write(base.join("t.json"), json).unwrap();
    std::fs::write(base.join("Data/Scripts/c.rhai"), rhai).unwrap();
    base
}

fn creature_world(base: PathBuf) -> World {
    use crate::creature::speeds::{NpcSpeeds, SpeedClassEntry};
    let mut w = World::new();
    w.core.base_directory = base;
    w.types
        .register_mobile(&crate::creature::vtable::BASE_CREATURE, &[]);
    w.resources
        .get_mut::<NpcSpeeds>()
        .register(SpeedClassEntry {
            level: "Medium".into(),
            active: 0.2,
            passive: 0.4,
            active_move: 0.0,
            passive_move: 0.0,
            types: vec![],
        });
    crate::scripting::register(&mut w.types);
    let errors = reload_scripts(&mut w);
    assert!(errors.is_empty(), "{errors:?}");
    let json = std::fs::read_to_string(w.core.base_directory.join("t.json")).unwrap_or_default();
    if !json.trim().is_empty() && json.trim() != "[]" {
        crate::templates::install_json(&mut w, &json).unwrap();
    }
    w
}

fn record(w: &mut World, seed: u64) -> (Vec<u8>, Option<Option<&'static str>>) {
    w.rng = muo_primitives::BuiltInRng::from_seed(seed);
    let id = crate::factory::make_mobile(w, "Custom.Beast").unwrap();
    let ty = w.mobile(id).unwrap().ty;
    let mut out = muo_io::SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    let corpse = crate::templates::props::as_static_str(
        w,
        crate::templates::props::mobile(w, id, "corpseName"),
    );
    (out.into_vec(), corpse)
}

#[test]
fn a_class_declared_in_a_script_matches_its_json_form() {
    let json = r#"[{ "type": "Custom.Beast", "parent": "Server.Mobiles.BaseCreature", "ai": { "coin": ["AiMelee", "AiMage"] },
        "props": { "corpseName": "a beastly corpse" },
        "body": 17, "str": [96, 120], "hits": [58, 72], "resists": { "Physical": [25, 30], "Fire": [20, 30] },
        "skills": { "Swords": [50.1, 75.0], "Tactics": 60.0 }, "fame": 1500,
        "chance": { "p": 0.5, "then": { "karma": -10 }, "else": { "karma": -20 } },
        "oneOf": [ { "hue": 1 }, { "hue": 2 }, { "hue": 3 } ] }]"#;
    let rhai = r#"
        class("Custom.Beast", #{ parent: "Server.Mobiles.BaseCreature", ai: #{ coin: ["AiMelee", "AiMage"] },
                                 props: #{ corpseName: "a beastly corpse" } }, |m| {
            body(m, 17);
            str(m, [96, 120]);
            hits(m, [58, 72]);
            resist(m, "Physical", 25, 30);
            resist(m, "Fire", 20, 30);
            skill(m, "Swords", 50.1, 75.0);
            skill(m, "Tactics", 60.0);
            fame(m, 1500);
            if random_double() < 0.5 { karma(m, -10) } else { karma(m, -20) }
            switch random_below(3) { 0 => hue(m, 1), 1 => hue(m, 2), _ => hue(m, 3) }
        });
    "#;
    let mut a = creature_world(class_server("json", json, ""));
    let mut b = creature_world(class_server("rhai", "[]", rhai));
    for seed in 0..10 {
        assert_eq!(record(&mut a, seed), record(&mut b, seed), "seed {seed}");
    }
    assert_eq!(record(&mut b, 1).1, Some(Some("a beastly corpse")));
}

#[test]
fn a_script_class_takes_constructor_arguments_and_rejects_unknown_parents() {
    let rhai = r#"
        class("Custom.Beast", #{ parent: "Server.Mobiles.BaseCreature" }, |m, level| {
            fame(m, 1000 * if level == () { 1 } else { level });
        });
    "#;
    let mut w = creature_world(class_server("args", "[]", rhai));
    let ty = match w.types.find_type_by_full_name("Custom.Beast", false) {
        Some(muo_server::type_registry::EntityType::Mobile(t)) => t,
        _ => panic!("registered"),
    };
    let plain = (ty.construct.unwrap())(&mut w);
    assert_eq!(w.mobile(plain).unwrap().fame, 1000);
    let id = crate::creature::setters::new_creature(
        &mut w,
        ty,
        crate::creature::enums::AIType::AiMelee,
        crate::creature::enums::FightMode::Closest,
        16,
        1,
    );
    templates::apply_mobile_with(&mut w, id, "Custom.Beast", &[ScriptArg::Int(3)]);
    assert_eq!(w.mobile(id).unwrap().fame, 3000);

    let base = class_server("noparent", "[]", r#"class("Custom.Nothing", |m| {});"#);
    let mut w = World::new();
    w.core.base_directory = base;
    crate::scripting::register(&mut w.types);
    let errors = reload_scripts(&mut w);
    assert!(
        errors.iter().any(|e| e.contains("Custom.Nothing")),
        "{errors:?}"
    );
}

#[test]
fn the_friendly_syntax_matches_the_json_form() {
    let json = r#"[{ "type": "Custom.Beast", "parent": "Server.Mobiles.BaseCreature", "ai": { "coin": ["AiMelee", "AiMage"] },
        "props": { "corpseName": "a beastly corpse", "loot": [["Meager", 1]] },
        "body": 17, "str": [96, 120], "hits": [58, 72], "resists": { "Physical": [25, 30], "Fire": 20 },
        "skills": { "Swords": [50.1, 75.0], "Tactics": 60.0 }, "virtualArmor": 28, "fame": 1500,
        "chance": { "p": 0.5, "then": { "karma": -10 }, "else": { "karma": -20 } } }]"#;
    let rhai = r#"
        mobile("Custom.Beast", |c| {
            c.extends = "BaseCreature";
            c.ai = coin("Melee", "Mage");
            c.corpse_name = "a beastly corpse";
            c.loot = "Meager";
            c.on_create(|m| {
                m.body = 17;
                m.str = 96..=120;
                m.hits = 58..=72;
                m.resist("Physical", 25, 30);
                m.resist("Fire", 20);
                m.skill("Swords", 50.1, 75.0);
                m.skill("Tactics", 60.0);
                m.armor = 28;
                m.fame = 1500;
                if chance(0.5) { m.karma = -10 } else { m.karma = -20 }
            });
        });
    "#;
    let mut a = creature_world(class_server("fjson", json, ""));
    let mut b = creature_world(class_server("frhai", "[]", rhai));
    for seed in 0..10 {
        assert_eq!(record(&mut a, seed), record(&mut b, seed), "seed {seed}");
    }
    let t = crate::templates::loaded(&b).unwrap();
    assert_eq!(
        t.mobiles["Custom.Beast"].props["loot"],
        serde_json::json!([["Meager", 1]])
    );
}

#[test]
fn friendly_items_set_attributes_and_hooks_run() {
    let rhai = r#"
        item("Custom.Stone", |c| {
            c.extends = "Server.Item";
            c.weight = 2.0;
            c.on_create(|i| {
                i.item_id = 0x1F19;
                i.hue = 0x501;
                i.movable = false;
                i.attributes.luck = 200;
                i.attributes.defend_chance = 15;
                i.hue = 0x501;
            });
            c.on_use(|stone, from| {
                stone.hue = 33;
                true
            });
        });
    "#;
    let base = class_server("fitem", "[]", rhai);
    let mut w = World::new();
    w.core.base_directory = base;
    crate::scripting::register(&mut w.types);
    let errors = reload_scripts(&mut w);
    assert!(errors.is_empty(), "{errors:?}");
    let id = crate::factory::make_item(&mut w, "Custom.Stone").unwrap();
    let i = w.item(id).unwrap();
    assert_eq!((i.item_id, i.hue, i.movable()), (0x1F19, 0x501, false));
    // the hue set after the magic properties shows every statement after them ran (a plain item has no bags)
    let from = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    assert!(script_hooks::item(
        &mut w,
        id,
        ScriptHook::Use,
        &[ScriptArg::Mobile(Some(from))]
    ));
    assert_eq!(w.item(id).unwrap().hue, 33);
}
