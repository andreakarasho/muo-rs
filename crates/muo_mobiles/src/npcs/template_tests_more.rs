//! The NPC classes converted after the first rounds (guards, escortables, the town crier, familiars, the special
//! NPCs, player vendors and the outfits that edit worn items): each has its template, and the constructors with
//! arguments and the new hue forms work.

use muo_server::World;

use crate::template_keys_npcs::Hue;

const CONVERTED: [&str; 37] = [
    "Server.Mobiles.WarriorGuard",
    "Server.Mobiles.ArcherGuard",
    "Server.Mobiles.BaseEscortable",
    "Server.Mobiles.BrideGroom",
    "Server.Mobiles.EscortableMage",
    "Server.Mobiles.Merchant",
    "Server.Mobiles.Messenger",
    "Server.Mobiles.Noble",
    "Server.Mobiles.Peasant",
    "Server.Mobiles.Townfolk.Prisoner",
    "Server.Mobiles.SeekerOfAdventure",
    "Server.Mobiles.TownCrier",
    "Server.Mobiles.BaseFamiliar",
    "Server.Mobiles.DarkWolfFamiliar",
    "Server.Mobiles.DeathAdder",
    "Server.Mobiles.ShadowWispFamiliar",
    "Server.Mobiles.VampireBatFamiliar",
    "Server.Mobiles.HordeMinionFamiliar",
    "Server.Mobiles.Silvani",
    "Server.Mobiles.ServantOfSemidar",
    "Server.Mobiles.DarkGuardian",
    "Server.Mobiles.Barracoon",
    "Server.Mobiles.Mephitis",
    "Server.Mobiles.Rikktor",
    "Server.Mobiles.Semidar",
    "Server.Mobiles.Serado",
    "Server.Mobiles.Neira",
    "Server.Mobiles.LordOaks",
    "Server.Mobiles.Harrower",
    "Server.Mobiles.HarrowerTentacles",
    "Server.Mobiles.ChaosGuard",
    "Server.Mobiles.OrderGuard",
    "Server.Mobiles.PlayerVendor",
    "Server.Mobiles.RentedVendor",
    "Server.Mobiles.PlayerBarkeeper",
    "Server.Mobiles.CustomHairstylist",
    "Server.Mobiles.Vagabond",
];

/// The vendors whose outfit (`npcOutfit`) edits what they wear.
const OUTFITS: [&str; 8] = [
    "Server.Mobiles.Blacksmith",
    "Server.Mobiles.BlacksmithGuildmaster",
    "Server.Mobiles.IronWorker",
    "Server.Mobiles.Vagabond",
    "Server.Mobiles.GypsyMaiden",
    "Server.Mobiles.GypsyBanker",
    "Server.Mobiles.GypsyAnimalTrainer",
    "Server.Mobiles.HolyMage",
];

fn world() -> World {
    let mut w = World::new();
    crate::register(&mut w.types);
    w
}

#[test]
fn every_converted_npc_has_its_template_and_outfit() {
    let mut w = world();
    let t = muo_content::templates::table(&mut w);
    for n in CONVERTED {
        assert!(t.mobiles.contains_key(n), "no template for {n}");
    }
    for n in OUTFITS {
        let tmpl = t
            .mobiles
            .get(n)
            .unwrap_or_else(|| panic!("no template for {n}"));
        assert!(tmpl.props.contains_key("npcOutfit"), "{n}: no npcOutfit");
    }
}

#[test]
fn a_guard_made_for_a_target_stands_on_it_and_focuses_it() {
    let mut w = world();
    let target = muo_content::factory::make_mobile(&mut w, "Server.Mobiles.Horse").unwrap();
    let p = muo_primitives::Point3D::new(100, 200, 0);
    muo_server::mobile::part1::lifecycle::set_location(&mut w, target, p);
    let id = super::guards::warrior_guard_with(&mut w, Some(target));
    assert_eq!(super::guards::focus(&w, id), Some(target));
    let m = w.mobile(id).unwrap();
    assert_eq!(m.location, p);
    assert_eq!(m.title.as_deref(), Some("the guard"));
    assert!(matches!(m.body.0, 400 | 401));
}

#[test]
fn hue_forms_parse_and_roll() {
    let mut w = world();
    let one = Hue::parse(&serde_json::json!({"oneOf": [5, 5, 5]})).unwrap();
    assert_eq!(one.roll(&mut w, None), 5);
    assert!(Hue::parse(&serde_json::json!({"oneOf": []})).is_err());
    assert!(Hue::parse(&serde_json::json!("noSuchHue")).is_err());
    // a variable nobody drew, and a hue of the mobile without one, read as 0
    assert_eq!(
        Hue::parse(&serde_json::json!("$low"))
            .unwrap()
            .roll(&mut w, None),
        0
    );
    assert_eq!(
        Hue::parse(&serde_json::json!("hairHue"))
            .unwrap()
            .roll(&mut w, None),
        0
    );
}
