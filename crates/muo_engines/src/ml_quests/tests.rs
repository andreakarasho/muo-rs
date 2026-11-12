//! Tests of the ML quest system: the generated tables, the save layout of `MLQuestPersistence`, and the quest
//! rules that need no client (offering, kills, claiming).

use muo_content::player_mobile::new_player_mobile;
use muo_io::{GenericReader, SaveReader, SaveWriter};
use muo_primitives::DateTime;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::item::{ITEM_BASE, ops};
use muo_server::mobile::{MOBILE_BASE, MobileType};

use super::defs;
use super::engine;
use super::model::*;
use super::persistence::{ML_QUEST_PERSISTENCE, ensure_existence};
use super::quester::Quester;

/// `Server.Mobiles.Rabbit` for the kill objective: only the class chain matters.
static RABBIT: MobileType = MobileType {
    full_name: "Server.Mobiles.Rabbit",
    parent: Some(&MOBILE_BASE),
    ..MOBILE_BASE
};

fn setup() -> (World, MobileId, MobileId) {
    let mut w = World::new();
    state(&mut w).enabled = true;
    engine::register_quest(&mut w, &defs::britannia::AEMAETH1, &["Aurelia"]);
    engine::register_quest(&mut w, &defs::britannia::AEMAETH2, &["SkeletonOfSzandor"]);
    engine::register_quest(
        &mut w,
        &defs::heartwood::THEYRE_BREEDING_LIKE_RABBITS,
        &["Aelorn"],
    );
    let pm = new_player_mobile(&mut w);
    let quester = w.new_mobile(&MOBILE_BASE);
    (w, pm, quester)
}

#[test]
fn every_generated_quest_builds_and_names_are_unique() {
    let mut names: Vec<&str> = defs::ALL.iter().map(|c| c.full_name).collect();
    let n = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), n, "a quest class is listed twice");
    assert!(n > 250);
    for class in defs::ALL {
        let q = (class.create)(class);
        assert!(!q.title.is_empty(), "{} has no title", class.full_name);
        assert!(q.save_enabled && !q.deserialized);
    }
}

/// Every quest `Data/MLQuests.cfg` of the distribution names is a class we have (the file is not in the repo).
#[test]
fn the_distribution_config_names_only_known_quests() {
    let path = std::path::PathBuf::from(std::env::var_os("MUO_DISTRIBUTION").unwrap_or_default())
        .join("Data/MLQuests.cfg");
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    let mut missing = Vec::new();
    for line in text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let name = line.split('\t').next().unwrap_or("");
        if !defs::ALL
            .iter()
            .any(|c| c.name().eq_ignore_ascii_case(name))
        {
            missing.push(name.to_owned());
        }
    }
    // These Heartwood quests are disabled until their content exists: the registry skips them.
    let commented_out = [
        "HowManyHeads",
        "DreadhornQuest",
        "NewLeadership",
        "ExAssassins",
        "ExtinguishingTheFlame",
        "CrimeAndPunishment",
        "AllThatGlittersIsNotGood",
    ];
    missing.retain(|m| !commented_out.contains(&m.as_str()));
    assert!(missing.is_empty(), "not implemented: {missing:?}");
}

#[test]
fn a_one_time_quest_is_not_offered_again_and_a_full_log_offers_nothing() {
    let (mut w, pm, quester) = setup();
    let q = Quester::Mobile(quester);
    let aemaeth1 = engine::find_quest(&w, "Aemaeth1").unwrap();
    let aemaeth2 = engine::find_quest(&w, "Aemaeth2").unwrap();

    assert!(engine::can_offer(&mut w, aemaeth1, q, pm, false));

    // Aemaeth2 is OneTimeOnly: once done, neither it nor a quest that chains into it can be offered.
    engine::set_done_quest(&mut w, pm, aemaeth2, DateTime::MIN);
    assert!(!engine::can_offer(&mut w, aemaeth2, q, pm, false));
    assert!(!engine::can_offer(&mut w, aemaeth1, q, pm, false));

    // MaxConcurrentQuests
    let rabbits = engine::find_quest(&w, "TheyreBreedingLikeRabbits").unwrap();
    engine::get_or_create_context(&mut w, pm).quest_instances = (0..10).map(InstanceId).collect();
    assert!(!engine::can_offer(&mut w, rabbits, q, pm, false));
}

#[test]
fn ten_kills_complete_a_kill_quest_and_claiming_removes_the_instance() {
    let (mut w, pm, quester) = setup();
    let rabbits = engine::find_quest(&w, "TheyreBreedingLikeRabbits").unwrap();
    let inst = engine::create_instance(&mut w, rabbits, Some(Quester::Mobile(quester)), pm);
    assert_eq!(
        engine::get_context(&w, pm).unwrap().quest_instances,
        vec![inst]
    );

    let rabbit = w.new_mobile(&RABBIT);
    for _ in 0..9 {
        engine::handle_kill(&mut w, pm, rabbit);
    }
    assert!(!engine::is_completed(&mut w, inst));
    let other = w.new_mobile(&MOBILE_BASE);
    engine::handle_kill(&mut w, pm, other); // not a rabbit
    assert!(!engine::is_completed(&mut w, inst));

    engine::handle_kill(&mut w, pm, rabbit);
    assert!(engine::is_completed(&mut w, inst));
    // no completion message and nothing to collect: the quest moved on to ClaimReward by itself
    assert!(engine::instance(&w, inst).unwrap().claim_reward());

    engine::claim_rewards(&mut w, inst);
    assert!(engine::instance(&w, inst).unwrap().removed());
    assert!(
        engine::get_context(&w, pm)
            .unwrap()
            .quest_instances
            .is_empty()
    );
    assert!(engine::quest(&w, rabbits).instances.is_empty());
}

fn persistence_bytes(w: &World) -> Vec<u8> {
    let id = w
        .resources
        .get::<MlState>()
        .and_then(|s| s.persistence)
        .expect("persistence item");
    let mut out = SaveWriter::new(true);
    (ML_QUEST_PERSISTENCE.serialize)(w, id, &mut out);
    out.into_vec()
}

#[test]
fn the_persistence_item_round_trips_and_has_the_expected_layout() {
    let (mut w, pm, quester) = setup();
    let aemaeth1 = engine::find_quest(&w, "Aemaeth1").unwrap();
    let aemaeth2 = engine::find_quest(&w, "Aemaeth2").unwrap();
    let rabbits = engine::find_quest(&w, "TheyreBreedingLikeRabbits").unwrap();

    let inst = engine::create_instance(&mut w, rabbits, Some(Quester::Mobile(quester)), pm);
    // 3 kills so far
    if let Some(ObjectiveState::Kill { slain }) = w
        .resources
        .get_mut::<MlState>()
        .instances
        .get_mut(&inst)
        .map(|i| &mut i.objectives[0].state)
    {
        *slain = 3;
    }
    let ctx = engine::get_or_create_context(&mut w, pm);
    ctx.done_quests.push(DoneQuest {
        quest: aemaeth2,
        next_available: DateTime::MIN,
    });
    ctx.chain_offers.push(aemaeth2);
    ctx.set_flag(flag::BEDLAM_ACCESS, true);
    ensure_existence(&mut w);
    let bytes = persistence_bytes(&w);

    // The part after the Item's own: version 3, one context, then every registered quest with its version.
    let mut base = SaveWriter::new(true);
    let pid = w
        .resources
        .get::<MlState>()
        .and_then(|s| s.persistence)
        .unwrap();
    (ITEM_BASE.serialize)(&w, pid, &mut base);
    let tail = &bytes[base.as_slice().len()..];
    let mut r = SaveReader::new(tail);
    assert_eq!(r.read_i32().unwrap(), 3);
    assert_eq!(r.read_i32().unwrap(), 1); // contexts
    assert_eq!(r.read_serial().unwrap(), w.mobile(pm).unwrap().serial);
    assert_eq!(r.read_i32().unwrap(), 1); // instances
    assert_eq!(
        r.read_string().unwrap().as_deref(),
        Some("Server.Engines.MLQuests.Definitions.TheyreBreedingLikeRabbits")
    );
    assert_eq!(r.read_serial().unwrap(), w.mobile(quester).unwrap().serial);
    assert!(!r.read_bool().unwrap()); // ClaimReward
    assert_eq!(r.read_i32().unwrap(), 1); // objectives
    assert!(!r.read_bool().unwrap()); // not timed
    assert_eq!(r.read_u8().unwrap(), 2); // KillObjective
    assert_eq!(r.read_i32().unwrap(), 3);
    assert_eq!(r.read_i32().unwrap(), 1); // done quests
    assert_eq!(
        r.read_string().unwrap().as_deref(),
        Some("Server.Engines.MLQuests.Definitions.Aemaeth2")
    );
    r.read_date_time_ticks().unwrap();
    assert_eq!(r.read_i32().unwrap(), 1); // chain offers
    assert_eq!(
        r.read_string().unwrap().as_deref(),
        Some("Server.Engines.MLQuests.Definitions.Aemaeth2")
    );
    assert_eq!(r.read_encoded_int().unwrap(), flag::BEDLAM_ACCESS);
    assert_eq!(r.read_i32().unwrap(), 3); // quests
    for (name, version) in [
        ("Aemaeth1", 0),
        ("Aemaeth2", 0),
        ("TheyreBreedingLikeRabbits", 0),
    ] {
        assert_eq!(
            r.read_string().unwrap().as_deref(),
            Some(format!("Server.Engines.MLQuests.Definitions.{name}").as_str())
        );
        assert_eq!(r.read_i32().unwrap(), version);
    }
    assert_eq!(r.remaining(), 0);
    let _ = aemaeth1;

    // Load it into a world with the same entities and save again: the same bytes.
    let (mut w2, _, _) = setup();
    let id2 = ops::new_item_with_id(&mut w2, &ML_QUEST_PERSISTENCE, 1);
    let mut r = SaveReader::new(&bytes);
    (ML_QUEST_PERSISTENCE.deserialize)(&mut w2, id2, &mut r).unwrap();
    assert_eq!(r.remaining(), 0, "exact consumption");
    assert_eq!(persistence_bytes(&w2), bytes);
    let pm2 = w2.find_mobile(w.mobile(pm).unwrap().serial).unwrap();
    let ctx2 = engine::get_context(&w2, pm2).unwrap();
    assert_eq!(ctx2.quest_instances.len(), 1);
    assert!(ctx2.get_flag(flag::BEDLAM_ACCESS));
    assert!(engine::quest(&w2, rabbits).deserialized);
}

#[test]
fn a_record_of_a_quest_that_is_not_saved_reads_as_nothing() {
    // SaveQuest <type> false writes a null reference and the quest is dropped from the context on load.
    let (mut w, pm, quester) = setup();
    let rabbits = engine::find_quest(&w, "TheyreBreedingLikeRabbits").unwrap();
    engine::create_instance(&mut w, rabbits, Some(Quester::Mobile(quester)), pm);
    state(&mut w).quests[rabbits.0].save_enabled = false;
    ensure_existence(&mut w);
    let bytes = persistence_bytes(&w);

    let (mut w2, _, _) = setup();
    let id2 = ops::new_item_with_id(&mut w2, &ML_QUEST_PERSISTENCE, 1);
    let mut r = SaveReader::new(&bytes);
    (ML_QUEST_PERSISTENCE.deserialize)(&mut w2, id2, &mut r).unwrap();
    assert_eq!(r.remaining(), 0);
    let pm2 = w2.find_mobile(w.mobile(pm).unwrap().serial).unwrap();
    assert!(
        engine::get_context(&w2, pm2)
            .unwrap()
            .quest_instances
            .is_empty()
    );
}
