use std::path::PathBuf;

use muo_io::{SaveReader, SaveWriter};
use muo_server::mobile::ResistanceType;
use muo_server::world::World;

use super::*;
use crate::creature::speeds::{NpcSpeeds, SpeedClassEntry};
use crate::creature::vtable::BASE_CREATURE;

/// A fresh `<tmp>/<name>/Data/Templates` with `Mobiles/t.json` = `mobiles`; returns the server folder.
fn server_dir(name: &str, mobiles: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("muo-templates-{name}-{}", std::process::id()));
    let dir = base.join("Data").join("Templates").join("Mobiles");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("t.json"), mobiles).unwrap();
    base
}

fn world(base: PathBuf) -> World {
    let mut w = World::new();
    w.core.base_directory = base;
    w.types.register_mobile(&BASE_CREATURE, &[]);
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
    w
}

#[test]
fn errors_name_the_file_type_and_key() {
    let base = server_dir("err", r#"[{ "type": "X.Y", "str": "lots" }]"#);
    let e = load_dir(&base.join("Data/Templates")).unwrap_err();
    assert!(
        e.contains("t.json") && e.contains("X.Y") && e.contains("\"str\""),
        "{e}"
    );

    let base = server_dir("unknown", r#"[{ "type": "X.Y", "strength": 5 }]"#);
    let e = load_dir(&base.join("Data/Templates")).unwrap_err();
    assert!(e.contains("\"strength\": unknown key"), "{e}");

    let base = server_dir(
        "cycle",
        r#"[{ "type": "A", "inherits": "B" }, { "type": "B", "inherits": "A" }]"#,
    );
    let e = load_dir(&base.join("Data/Templates")).unwrap_err();
    assert!(e.contains("cycle"), "{e}");
}

#[test]
fn keys_keep_file_order() {
    let base = server_dir(
        "order",
        r#"[{ "type": "A", "hits": 5, "str": [1, 2], "body": 9 }]"#,
    );
    let t = load_dir(&base.join("Data/Templates")).unwrap();
    let steps = &t.mobiles["A"].mobile_steps;
    assert_eq!(
        steps,
        &vec![
            mobile_steps::Step::Hits(mobile_steps::IRange::Fixed(5)),
            mobile_steps::Step::Str(mobile_steps::IRange::Range(1, 2)),
            mobile_steps::Step::Body(9),
        ]
    );
}

#[test]
fn json_only_creature_constructs_saves_and_loads() {
    let base = server_dir(
        "jsononly",
        r#"[
          { "type": "Custom.Base", "parent": "Server.Mobiles.BaseCreature", "ai": "AiMage", "fightMode": "Strongest",
            "body": 400, "str": 100, "hits": 250, "resists": { "Fire": 60 }, "skills": { "Magery": 90.0 },
            "props": { "corpseName": "a test corpse" } },
          { "type": "Custom.Child", "parent": "Custom.Base", "inherits": "Custom.Base", "name": "child", "hits": 300 }
        ]"#,
    );
    let mut w = world(base);
    reload(&mut w).unwrap();

    let id = crate::factory::make_mobile(&mut w, "Custom.Child").expect("registered");
    let m = w.mobile(id).unwrap();
    assert_eq!(m.ty.full_name, "Custom.Child");
    assert_eq!(m.body.0, 400);
    assert_eq!(m.name.as_deref(), Some("child"));
    assert_eq!(m.str, 100);
    assert_eq!(m.hits, 300);
    let ty = m.ty;
    assert_eq!(
        crate::creature::cd(&w, id).unwrap().current_ai,
        AIType::AiMage
    );
    assert_eq!((ty.get_resistance)(&mut w, id, ResistanceType::Fire), 60);
    assert_eq!(
        props::as_static_str(&w, props::mobile(&w, id, "corpseName")),
        Some(Some("a test corpse"))
    );

    // The save is the parent's record format under the class's own name.
    let mut out = SaveWriter::new(true);
    (ty.serialize)(&w, id, &mut out);
    let bytes = out.into_vec();
    let id2 = w.new_mobile(ty);
    let mut r = SaveReader::new(&bytes);
    (ty.deserialize)(&mut w, id2, &mut r).unwrap();
    assert_eq!(w.mobile(id2).unwrap().hits, 300);

    // A reload keeps the class (same pointer, same slot).
    reload(&mut w).unwrap();
    let again = w
        .types
        .find_type_by_full_name("Custom.Child", false)
        .unwrap();
    assert!(
        matches!(again, muo_server::type_registry::EntityType::Mobile(t) if std::ptr::eq(t, ty))
    );
}

#[test]
fn json_only_class_cannot_shadow_a_rust_class() {
    let base = server_dir(
        "shadow",
        r#"[{ "type": "Server.Mobiles.BaseCreature", "parent": "Server.Mobile" }]"#,
    );
    let mut w = world(base);
    let e = reload(&mut w).unwrap_err();
    assert!(e.contains("drop \"parent\""), "{e}");
}

#[test]
fn item_attributes_parse_by_name() {
    use crate::aos::{AosArmorAttribute, AosAttribute, AttrKind};
    let s = item_steps::parse(
        "attributes",
        &serde_json::json!({ "Luck": 200, "DefendChance": 15 }),
        &keys::Keys::default(),
    )
    .unwrap();
    assert_eq!(
        s,
        item_steps::Step::Attrs(vec![
            (AttrKind::Attributes, AosAttribute::Luck as u32, 200),
            (AttrKind::Attributes, AosAttribute::DefendChance as u32, 15),
        ])
    );
    let s = item_steps::parse(
        "armorAttributes",
        &serde_json::json!({ "MageArmor": 1 }),
        &keys::Keys::default(),
    )
    .unwrap();
    assert_eq!(
        s,
        item_steps::Step::Attrs(vec![(
            AttrKind::Armor,
            AosArmorAttribute::MageArmor as u32,
            1
        )])
    );
    let e = item_steps::parse(
        "attributes",
        &serde_json::json!({ "Lucky": 1 }),
        &keys::Keys::default(),
    )
    .unwrap_err();
    assert!(e.contains("unknown attribute Lucky"), "{e}");
    let e = item_steps::parse(
        "skillBonuses",
        &serde_json::json!([["Swordz", 10]]),
        &keys::Keys::default(),
    )
    .unwrap_err();
    assert!(e.contains("unknown skill"), "{e}");
}

#[test]
fn era_coin_and_registered_keys() {
    let base = server_dir(
        "flow",
        r#"[{ "type": "Custom.Flow", "parent": "Server.Mobiles.BaseCreature",
              "era": { "atLeast": "AOS", "then": { "title": "new" }, "else": { "title": "old" } },
              "coin": { "then": { "body": 1 }, "else": { "body": 2 } },
              "oneOf": [ { "fame": 10 }, { "fame": 20 }, { "fame": 30 } ],
              "chance": { "p": 1.0, "then": { "karma": -5 } },
              "testName": "keyed" }]"#,
    );
    let mut w = world(base);
    keys::mobile_key(
        &mut w.types,
        "testName",
        |v| v.as_str().map(|_| ()).ok_or_else(|| "a string".into()),
        |w, id, v| crate::creature::init_ops::name(w, id, v.as_str().map(str::to_owned)),
    );
    reload(&mut w).unwrap();

    for (era, title) in [
        (muo_data::expansion::Expansion::AOS, "new"),
        (muo_data::expansion::Expansion::LBR, "old"),
    ] {
        w.core.expansion = Some(era);
        w.rng = muo_primitives::BuiltInRng::from_seed(3);
        let id = crate::factory::make_mobile(&mut w, "Custom.Flow").unwrap();
        // the same draws, in the same order, outside the template
        let mut r = muo_primitives::BuiltInRng::from_seed(3);
        let body = if muo_primitives::utility::random_bool(&mut r) {
            1
        } else {
            2
        };
        let fame = [10, 20, 30][muo_primitives::utility::random(&mut r, 3) as usize];
        let m = w.mobile(id).unwrap();
        assert_eq!(m.title.as_deref(), Some(title));
        assert_eq!(m.body.0, body);
        assert_eq!(m.fame, fame);
        assert_eq!(m.karma, -5);
        assert_eq!(m.name.as_deref(), Some("keyed"));
    }

    let base = server_dir(
        "badkey",
        r#"[{ "type": "X", "era": { "atLeast": "AOZ" } }]"#,
    );
    let e = load_dir(&base.join("Data/Templates")).unwrap_err();
    assert!(e.contains("unknown era AOZ"), "{e}");
}

#[test]
fn a_lenient_load_skips_unregistered_keys_and_a_strict_one_rejects_them() {
    let base = server_dir(
        "lenient",
        r#"[{ "type": "A", "hits": 5, "someCrateKey": 1 }]"#,
    );
    let root = base.join("Data/Templates");
    let w = World::new();
    let t = load_dir_with(&root, &keys::Keys::lenient(&w.types)).unwrap();
    assert_eq!(t.mobiles["A"].mobile_steps.len(), 2);
    let e = load_dir_with(&root, &keys::Keys::of(&w.types)).unwrap_err();
    assert!(e.contains("\"someCrateKey\": unknown key"), "{e}");
}

#[test]
fn a_random_ai_is_drawn_before_the_creature_exists() {
    let base = server_dir(
        "aichoice",
        r#"[{ "type": "Custom.Pumpkin", "parent": "Server.Mobiles.BaseCreature",
              "ai": { "coin": ["AiMelee", "AiMage"] }, "fightMode": { "oneOf": ["Closest", "Strongest"] } }]"#,
    );
    let mut w = world(base);
    reload(&mut w).unwrap();
    for seed in 0..8 {
        w.rng = muo_primitives::BuiltInRng::from_seed(seed);
        let id = crate::factory::make_mobile(&mut w, "Custom.Pumpkin").unwrap();
        let mut r = muo_primitives::BuiltInRng::from_seed(seed);
        let ai = if muo_primitives::utility::random_bool(&mut r) {
            AIType::AiMelee
        } else {
            AIType::AiMage
        };
        let mode = [FightMode::Closest, FightMode::Strongest]
            [muo_primitives::utility::random(&mut r, 2) as usize];
        let d = crate::creature::cd(&w, id).unwrap();
        assert_eq!((d.current_ai, d.fight_mode), (ai, mode), "seed {seed}");
    }
    let base = server_dir(
        "aibad",
        r#"[{ "type": "X", "ai": { "coin": ["AiMelee"] } }]"#,
    );
    assert!(
        load_dir(&base.join("Data/Templates"))
            .unwrap_err()
            .contains("coin")
    );
}
