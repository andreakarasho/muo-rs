use std::path::PathBuf;

use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::MobileId;

use muo_server::mobile::AccessLevel;
use muo_server::script_hooks::{self, ScriptArg, ScriptHook};
use muo_server::world::World;

use super::*;

/// A server folder with a JSON-only mobile class `Test.Scripted` and item class `Test.ScriptedItem`, both bound
/// to `Data/Scripts/a.rhai`.
fn server(name: &str, scripts: &[(&str, &str)]) -> PathBuf {
    let base = std::env::temp_dir().join(format!("muo-scripting-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let t = base.join("Data").join("Templates");
    std::fs::create_dir_all(t.join("Mobiles")).unwrap();
    std::fs::create_dir_all(t.join("Items")).unwrap();
    std::fs::write(
        t.join("Mobiles/t.json"),
        r#"[{ "type": "Test.Scripted", "parent": "Server.Mobile", "script": "a.rhai" }]"#,
    )
    .unwrap();
    std::fs::write(
        t.join("Items/t.json"),
        r#"[{ "type": "Test.ScriptedItem", "parent": "Server.Item", "script": "a.rhai" }]"#,
    )
    .unwrap();
    std::fs::create_dir_all(base.join("Data/Scripts")).unwrap();
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
    templates::reload(&mut w).unwrap();
    let errors = reload_scripts(&mut w);
    (w, errors)
}

fn spawn(w: &mut World) -> MobileId {
    let id = muo_content::factory::make_mobile(w, "Test.Scripted").unwrap();
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

    let it = muo_content::factory::make_item(&mut w, "Test.ScriptedItem").unwrap();
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
fn a_compile_error_leaves_the_other_scripts_working() {
    let base = server(
        "compile",
        &[
            ("a.rhai", "fn on_damage(m, amount, from) { true }"),
            ("bad.rhai", "fn broken( {"),
        ],
    );
    let (mut w, errors) = world(base);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].starts_with("bad.rhai"), "{errors:?}");
    let id = spawn(&mut w);
    damage(&mut w, id, 10);
    assert_eq!(hits(&w, id), 60);
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
