//! The NPC classes whose constructors and class constants are data: each has its template, every nested block of
//! NPC keys parses with the registered keys, and every SBInfo a vendor names exists.

use serde_json::Value;

use muo_content::templates::keys::Keys;
use muo_content::templates::mobile_steps;
use muo_server::World;
use muo_server::mobile::MobileType;
use muo_server::type_registry::TypeRegistry;

fn converted() -> Vec<&'static MobileType> {
    let mut v = super::vendors::npc::all();
    v.extend(super::humans::all());
    v.extend(super::guildmasters::all());
    v.extend(super::healers::all());
    v.extend(super::banker::all());
    v.extend(super::vendors::hand::all());
    v.extend(super::vendors::stable::all());
    v
}

fn world() -> World {
    let mut w = World::new();
    crate::template_keys::register(&mut w.types);
    crate::template_keys_npcs::register(&mut w.types);
    w
}

/// A block of template keys and every block nested in it.
fn blocks<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
    let Some(o) = v.as_object() else { return };
    out.push(v);
    for (k, x) in o {
        match (k.as_str(), x) {
            ("npcSeq" | "oneOf", Value::Array(a)) => a.iter().for_each(|b| blocks(b, out)),
            ("coin" | "chance" | "era", Value::Object(f)) => f
                .get("then")
                .into_iter()
                .chain(f.get("else"))
                .for_each(|b| blocks(b, out)),
            ("npcIfLayerEmpty", Value::Object(f)) => {
                f.get("then").into_iter().for_each(|b| blocks(b, out))
            }
            _ => {}
        }
    }
}

/// The SBInfo names of an `npcSbInfos` list.
fn sb_names<'a>(v: &'a Value, out: &mut Vec<&'a str>) {
    match v {
        Value::String(s) => out.push(s),
        Value::Array(a) => a.iter().for_each(|x| sb_names(x, out)),
        Value::Object(o) => o
            .iter()
            .filter(|(k, _)| matches!(k.as_str(), "oneOf" | "then" | "else"))
            .for_each(|(_, x)| sb_names(x, out)),
        _ => {}
    }
}

#[test]
fn every_converted_npc_has_its_template() {
    let mut w = world();
    let t = muo_content::templates::table(&mut w);
    let classes = converted();
    assert_eq!(classes.len(), 91);
    for ty in classes {
        assert!(
            t.mobiles.contains_key(ty.full_name),
            "no template for {}",
            ty.full_name
        );
    }
    assert!(t.mobiles.contains_key("Server.Mobiles.BaseHealer"));
}

#[test]
fn every_npc_block_parses_and_every_sb_info_exists() {
    let mut w = world();
    let t = muo_content::templates::table(&mut w);
    let sbs = super::vendors::sb::table(&mut w);
    let mut types = TypeRegistry::default();
    crate::register(&mut types);
    // only this crate's keys are registered here: the strict load of every file with every crate's keys is
    // `crates/muo/tests/templates_load.rs`
    let keys = Keys::lenient(&types);
    let dir = std::env::current_dir()
        .unwrap()
        .ancestors()
        .map(|a| a.join("Data").join("Templates"))
        .find(|p| p.is_dir())
        .unwrap();
    muo_content::templates::load_dir_with(&dir, &keys).unwrap();
    // the blocks nested in NPC keys and the npcOutfit props are parsed when they run: parse them here
    let mut checked = 0;
    for e in std::fs::read_dir(dir.join("Mobiles").join("npcs")).unwrap() {
        let file: Value =
            serde_json::from_str(&std::fs::read_to_string(e.unwrap().path()).unwrap()).unwrap();
        for tmpl in file.as_array().unwrap() {
            let ty = tmpl["type"].as_str().unwrap();
            let mut list = Vec::new();
            blocks(tmpl, &mut list);
            if let Some(v) = tmpl["props"].get("npcOutfit") {
                blocks(v, &mut list);
            }
            for b in list {
                for (k, v) in b.as_object().unwrap() {
                    if ["type", "inherits", "ai", "fightMode", "props"].contains(&k.as_str()) {
                        continue;
                    }
                    if let Err(e) = mobile_steps::parse(k, v, &keys) {
                        panic!("{ty}: \"{k}\": {e}");
                    }
                    checked += 1;
                }
            }
        }
    }
    for tmpl in t.mobiles.values() {
        let mut names = Vec::new();
        if let Some(v) = tmpl.props.get("npcSbInfos") {
            sb_names(v, &mut names);
        }
        for n in names {
            assert!(sbs.contains(n), "{}: no SBInfo {n}", tmpl.type_name);
        }
    }
    assert!(checked > 100, "{checked}");
    // the ones the Rust classes name
    for n in [
        "SBMage",
        "SBNinja",
        "SBSamurai",
        "SBScribe",
        "SBPlayerBarkeeper",
        "SBHealer",
        "SBBanker",
    ] {
        assert!(sbs.contains(n), "no SBInfo {n}");
    }
}

#[test]
fn a_hireling_and_a_vendor_come_out_of_their_templates() {
    let mut w = world();
    w.core.expansion = Some(muo_data::expansion::Expansion::AOS);
    let id = (super::humans::HIRE_PALADIN.construct.unwrap())(&mut w);
    let m = w.mobile(id).unwrap();
    assert_eq!(m.title.as_deref(), Some("the paladin"));
    assert!(matches!(m.body.0, 400 | 401));
    assert!((100..=100).contains(&m.str), "{}", m.str); // the AOS branch of the era block

    let id = (super::vendors::npc::ARCHITECT.construct.unwrap())(&mut w);
    let d = muo_content::vendor::data(&w, id).unwrap();
    // AOS: only SBArchitect, whose AOS block adds the house placement tool
    assert_eq!(d.sell_info.len(), 1);
    assert!(
        d.buy_info
            .iter()
            .any(|b| b.type_name == "Server.Items.HousePlacementTool")
    );
    assert_eq!(
        crate::template_keys_npcs::npc_guild(&w, id),
        muo_content::player_mobile::NpcGuild::TinkersGuild
    );
}
