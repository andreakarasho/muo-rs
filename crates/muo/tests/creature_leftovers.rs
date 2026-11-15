//! Creature classes whose argument constructors, sounds, era/random members and ability lists moved to
//! `Data/Templates/Mobiles` + `Data/Scripts/content/creatures`: the data is there and builds what the Rust built.

use muo_content::creature::{cd, ctype};
use muo_content::templates::{self, Kind};
use muo_data::expansion::Expansion;
use muo_server::World;
use muo_server::ids::MobileId;

fn world(era: Expansion) -> World {
    let mut w = World::new();
    muo::register_content(&mut w.types);
    w.core.base_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    templates::reload(&mut w).unwrap();
    assert!(muo_content::scripting::reload_scripts(&mut w).is_empty());
    w.core.expansion = Some(era);
    w
}

fn backpack_has(w: &World, id: MobileId, class: &str) -> bool {
    let m = w.mobile(id).unwrap();
    m.items.iter().any(|&b| {
        w.item(b).is_some_and(|p| {
            p.items
                .iter()
                .any(|&i| w.item(i).is_some_and(|i| i.ty.full_name == class))
        })
    })
}

#[test]
fn converted_classes_have_their_templates_and_scripts() {
    let mut w = world(Expansion::AOS);
    let t = templates::table(&mut w);
    for (class, script) in [
        ("Server.Mobiles.PumpkinHead", None),
        ("Server.Mobiles.PlagueSpawn", None),
        ("Server.Mobiles.Leviathan", None),
        ("Server.Mobiles.Golem", Some("content/creatures/golem.rhai")),
        (
            "Server.Mobiles.BaseEnraged",
            Some("content/creatures/enraged.rhai"),
        ),
    ] {
        let tm = t
            .get(Kind::Mobile, class)
            .unwrap_or_else(|| panic!("{class}"));
        assert!(!tm.mobile_steps.is_empty(), "{class}: no constructor steps");
        assert_eq!(tm.script.as_deref(), script, "{class}");
    }
    for c in ["Rabbit", "Hart", "Hind", "BlackBear", "Eagle"] {
        let tm = t
            .get(Kind::Mobile, &format!("Server.Mobiles.Enraged{c}"))
            .unwrap();
        assert_eq!(tm.inherits.as_deref(), Some("Server.Mobiles.BaseEnraged"));
    }
}

#[test]
fn golem_arguments_reach_its_script() {
    let mut w = world(Expansion::AOS);
    let id = muo_mobiles::classes::misc4::new_golem(&mut w, true, 0.5);
    let m = w.mobile(id).unwrap();
    assert_eq!((m.body.0, m.hue, m.fame, m.karma), (752, 2101, 10, 10));
    assert!((125..=175).contains(&m.str)); // (int)(251 * 0.5) ..= (int)(350 * 0.5)
    let c = cd(&w, id).unwrap();
    assert_eq!((c.damage_min, c.damage_max, c.control_slots), (6, 12, 3));
    assert!(!backpack_has(&w, id, "Server.Items.IronIngot"));

    let id = muo_mobiles::classes::misc4::new_golem(&mut w, false, 1.0);
    let m = w.mobile(id).unwrap();
    assert_eq!((m.hue, m.fame, m.karma), (0, 3500, -3500));
    assert!((251..=350).contains(&m.str));
    assert!(backpack_has(&w, id, "Server.Items.IronIngot"));
}

#[test]
fn enraged_summons_follow_their_summoner() {
    let mut w = world(Expansion::AOS);
    let summoner = muo_mobiles::classes::misc4::new_plague_spawn(&mut w, None);
    let id = muo_mobiles::classes::enraged::new_enraged_rabbit(&mut w, summoner);
    let m = w.mobile(id).unwrap();
    assert_eq!((m.body.0, m.karma), (0xcd, -1000));
    assert!(m.hits <= m.str && m.stam <= m.dex && (50..=200).contains(&m.str));
    assert_eq!(cd(&w, id).unwrap().master, Some(summoner));
    let ty = m.ty;
    assert_eq!((ty.get_attack_sound)(&mut w, id), 0xC9);
}

#[test]
fn era_and_random_props_resolve_per_read() {
    for (era, se) in [(Expansion::LBR, false), (Expansion::ML, true)] {
        let mut w = world(era);
        let id = (muo_mobiles::monsters::leaves::ABYSMAL_HORROR
            .construct
            .unwrap())(&mut w);
        let c = ctype(&w, id).unwrap();
        assert_eq!((c.bard_immune)(&w, id), !se);
        assert_eq!((c.unprovokable)(&w, id), se);
        assert_eq!((c.ignore_young_protection)(&w, id), se);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..64 {
            seen.insert(format!("{:?}", (c.get_weapon_ability)(&mut w, id)));
        }
        assert_eq!(seen.len(), 2, "{seen:?}"); // {"coin": ["MortalStrike", "WhirlwindAttack"]}
    }
}
