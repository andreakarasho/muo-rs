//! Tests of the quest engine: the class tables against the expected layouts (`quest_tables`), and the
//! bytes a quest in progress is stored as.

use muo_content::player_mobile::quest_record::scan;
use muo_content::player_mobile::quest_tables::{CLASSES, SYSTEMS};
use muo_io::SaveReader;
use muo_primitives::Serial;
use muo_server::World;
use muo_server::ids::MobileId;

use super::core::{Conversation, Objective, Quest, decode, encode};
use super::{
    CONVERSATION_LISTS, OBJECTIVE_LISTS, QUEST_CLASSES, collector, haochi, hatchlings, matriarch,
    uzeraan, witch,
};

fn player() -> MobileId {
    MobileId(Serial::new(1))
}

#[test]
fn quest_slots_are_the_quest_types_order() {
    assert_eq!(QUEST_CLASSES.len(), SYSTEMS.len());
    for (i, (c, s)) in QUEST_CLASSES.iter().zip(SYSTEMS).enumerate() {
        assert_eq!(c.name, s.name);
        assert_eq!(c.slot, i);
    }
}

#[test]
fn every_expected_class_exists_and_nothing_else() {
    let mut ours: Vec<&str> = OBJECTIVE_LISTS
        .iter()
        .copied()
        .flatten()
        .map(|c| c.name)
        .collect();
    ours.extend(CONVERSATION_LISTS.iter().copied().flatten().map(|c| c.name));
    ours.sort_unstable();
    let mut theirs: Vec<&str> = CLASSES.iter().map(|(n, _)| *n).collect();
    theirs.sort_unstable();
    let missing: Vec<_> = theirs.iter().filter(|n| !ours.contains(n)).collect();
    let extra: Vec<_> = ours.iter().filter(|n| !theirs.contains(n)).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "missing {missing:?}, extra {extra:?}"
    );
    let mut dedup = ours.clone();
    dedup.dedup();
    assert_eq!(dedup.len(), ours.len(), "a class name is listed twice");
}

/// Each class, with its default state, written inside its quest, is walked by the generated layout to the byte.
#[test]
fn every_class_writes_the_layout_the_generated_tables_read() {
    let w = World::new();
    for class in QUEST_CLASSES {
        let mut q = Quest::new(&w, class, player());
        q.attached = true;
        for list in OBJECTIVE_LISTS {
            for o in list.iter() {
                // the objective belongs to the quest of its namespace
                if o.name.rsplit_once('.').map(|x| x.0) == class.name.rsplit_once('.').map(|x| x.0)
                {
                    q.objectives.push(Objective::new(o));
                }
            }
        }
        for list in CONVERSATION_LISTS {
            for c in list.iter() {
                if c.name.rsplit_once('.').map(|x| x.0) == class.name.rsplit_once('.').map(|x| x.0)
                {
                    q.conversations.push(Conversation::new(c));
                }
            }
        }
        let mut bytes = encode(&w, &q);
        let n = bytes.len();
        bytes.push(0xAA);
        let mut r = SaveReader::new(&bytes);
        let got = scan(&mut r)
            .unwrap_or_else(|e| panic!("{}: {e}", class.name))
            .unwrap();
        assert_eq!(got.len(), n, "{}", class.name);
        assert_eq!(&got[..], &bytes[..n]);

        // and decodes back to the same bytes
        let q2 =
            decode(&w, &bytes[..n], player()).unwrap_or_else(|e| panic!("{}: {e}", class.name));
        assert_eq!(encode(&w, &q2), &bytes[..n], "{}", class.name);
        assert_eq!(q2.objectives.len(), q.objectives.len());
        assert_eq!(q2.conversations.len(), q.conversations.len());
    }
}

#[test]
fn state_survives_a_round_trip() {
    let mut w = World::new();
    let mut q = matriarch::new_quest(&w, player(), true);
    q.objectives
        .push(Objective::new(&matriarch::GATHER_WATER_OBJECTIVE));
    q.objectives[0].cur_progress = 12;
    q.objectives[0].has_been_read = true;
    q.conversations.push(matriarch::process_fungi(true));
    let bytes = encode(&w, &q);
    let q2 = decode(&w, &bytes, player()).unwrap();
    assert!(matriarch::red_solen(&q2));
    assert_eq!(q2.objectives[0].cur_progress, 12);
    assert!(q2.objectives[0].has_been_read);
    assert!(*q2.conversations[0].state::<bool>());

    // a Haochi first trial remembers its kills
    let mut q = Quest::new(&w, &haochi::QUEST, player());
    q.objectives
        .push(Objective::new(&haochi::FIRST_TRIAL_KILL_OBJECTIVE));
    let bytes = encode(&w, &q);
    assert_eq!(decode(&w, &bytes, player()).unwrap().objectives.len(), 1);

    // the witch's ingredient list
    let o = witch::find_ingredient(&mut w, &[3, 7], false);
    assert_eq!(witch::ingredients(&o).len(), 3);
    let mut q = Quest::new(&w, &witch::QUEST, player());
    q.objectives.push(o);
    let q2 = decode(&w, &encode(&w, &q), player()).unwrap();
    assert_eq!(
        witch::ingredients(&q2.objectives[0]),
        witch::ingredients(&q.objectives[0])
    );
}

#[test]
fn golden_bytes_of_a_quest_in_progress() {
    // TerribleHatchlings: type indexed 10 of QuestTypes, version 0, one objective (indexed 4 of its table,
    // FirstKillObjective), version 1, HasBeenRead false, progress 0, not completed, child version 0; one
    // conversation (AcceptConversation, indexed 0), version 0, not read, child version 0; the system's child version.
    let w = World::new();
    let mut q = Quest::new(&w, &hatchlings::QUEST, player());
    q.objectives
        .push(Objective::new(&hatchlings::FIRST_KILL_OBJECTIVE));
    q.conversations
        .push(Conversation::new(&hatchlings::ACCEPT_CONVERSATION));
    let want = [
        1, 10, // QuestTypes[10]
        0,  // version
        1,  // one objective
        1, 1, 4, // present, indexed, FirstKillObjective
        1, 0, 0, 0, 0, // version 1, HasBeenRead, progress, HasCompleted, child version
        1, // one conversation
        1, 1, 0, // present, indexed, AcceptConversation
        0, 0, 0, // version, HasBeenRead, child version
        0, // system child version
    ];
    assert_eq!(encode(&w, &q), want);
}

#[test]
fn class_defaults() {
    let w = World::new();
    // CaptureImagesObjective completes when every image is done, none being done yet is not complete once
    // initialised; QuestObjective.MaxProgress is 1.
    let o = Objective::new(&uzeraan::TITHE_GOLD_OBJECTIVE);
    let q = Quest::new(&w, &uzeraan::QUEST, player());
    assert_eq!(o.max_progress(&q), 1);
    assert!(!o.completed(&q));
    let o = Objective::new(&collector::FISH_PEARLS_OBJECTIVE);
    assert_eq!(o.max_progress(&q), 6);
}

fn world_with_quests() -> World {
    let mut w = World::new();
    muo_content::register(&mut w.types);
    super::register(&mut w.types);
    w
}

#[test]
fn every_quester_constructs_and_round_trips_its_save() {
    use muo_io::{SaveReader, SaveWriter};
    let mut w = world_with_quests();
    for list in super::mobile_lists() {
        for ty in list.iter().filter(|t| t.construct.is_some()) {
            let id = (ty.construct.expect("filtered"))(&mut w);
            let mut out = SaveWriter::new(true);
            (ty.serialize)(&w, id, &mut out);
            let bytes = out.into_vec();
            let id2 = w.new_mobile(ty);
            (ty.deserialize)(&mut w, id2, &mut SaveReader::new(&bytes))
                .unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
            let mut out2 = SaveWriter::new(true);
            (ty.serialize)(&w, id2, &mut out2);
            assert_eq!(out2.into_vec().len(), bytes.len(), "{}", ty.full_name);
        }
    }
}

#[test]
fn every_quest_item_constructs_and_round_trips_its_save() {
    use muo_io::{SaveReader, SaveWriter};
    let mut w = world_with_quests();
    for list in super::item_lists() {
        for ty in list.iter().filter(|t| t.construct.is_some()) {
            let id = (ty.construct.expect("filtered"))(&mut w);
            let mut out = SaveWriter::new(true);
            (ty.serialize)(&w, id, &mut out);
            let bytes = out.into_vec();
            let id2 = w.new_item(ty);
            (ty.deserialize)(&mut w, id2, &mut SaveReader::new(&bytes))
                .unwrap_or_else(|e| panic!("{}: {e}", ty.full_name));
            let mut out2 = SaveWriter::new(true);
            (ty.serialize)(&w, id2, &mut out2);
            assert_eq!(out2.into_vec(), bytes, "{}", ty.full_name);
        }
    }
}

/// A quest from the offer to its end, through the bytes on the player.
#[test]
fn the_hatchlings_quest_runs_from_accept_to_complete() {
    use muo_content::player_mobile::{self as pmob, PLAYER_MOBILE, QuestTypeRef};
    use muo_primitives::DateTime;

    let mut w = world_with_quests();
    let pm = w.new_mobile(&PLAYER_MOBILE);
    assert!(pmob::data_mut(&mut w, pm).is_some());

    // the offer: nothing stored yet
    let (can, _) = super::core::can_offer_quest_ex(&mut w, pm, &hatchlings::QUEST);
    assert!(can);
    let mut q = Quest::new(&w, &hatchlings::QUEST, pm);
    let accept = q.class.accept;
    accept(&mut w, &mut q);
    assert!(q.attached);
    super::core::store(&mut w, pm, &q);
    assert!(super::core::has_quest(&w, pm));
    assert!(super::core::quest_is(&w, pm, &hatchlings::QUEST));

    // reading the accept conversation hands out the first objective
    super::core::with_quest(&mut w, pm, |w, q| {
        let c = q.conversations[0].clone();
        (c.class.on_read)(w, q, &c);
    });
    let q = super::core::load(&w, pm).expect("stored");
    assert_eq!(q.objectives.len(), 1);

    // the kills: the first completes into the second; reading the second takes the player to the third (one
    // hatchling already counted), whose tenth kill hands out the return
    super::core::with_quest(&mut w, pm, |w, q| q.complete_objective(w, 0));
    super::core::with_quest(&mut w, pm, |w, q| {
        let on_read = q.objectives[1].class.on_read;
        on_read(w, q, 1);
    });
    let q = super::core::load(&w, pm).expect("stored");
    assert_eq!(q.objectives[2].cur_progress, 1);
    super::core::with_quest(&mut w, pm, |w, q| q.set_progress(w, 2, 10));
    let q = super::core::load(&w, pm).expect("stored");
    assert!(q.has_objective(&hatchlings::RETURN_OBJECTIVE));
    assert_eq!(q.conversations.len(), 1);

    // the return: the last conversation, read, ends the quest and remembers it
    super::core::with_quest(&mut w, pm, |w, q| {
        let i = q.objectives.len() - 1;
        q.complete_objective(w, i);
        let c = q.conversations.last().cloned().expect("end conversation");
        (c.class.on_read)(w, q, &c);
    });
    assert!(!super::core::has_quest(&w, pm));
    let d = pmob::data(&w, pm).expect("player");
    assert_eq!(
        d.done_quests,
        vec![(QuestTypeRef::Indexed(10), DateTime::MAX)]
    );
    let (can, in_restart) = super::core::can_offer_quest_ex(&mut w, pm, &hatchlings::QUEST);
    assert!(!can && in_restart);
}

/// `Victoria.CanTalkTo` is `CanOfferQuest`, which drops an expired done-quest entry on the first talk (not the next).
#[test]
fn victoria_can_talk_to_removes_an_expired_done_entry() {
    use muo_content::player_mobile::{self as pmob, PLAYER_MOBILE, QuestTypeRef};
    use muo_primitives::DateTime;

    let mut w = world_with_quests();
    let pm = w.new_mobile(&PLAYER_MOBILE);
    let victoria = (super::summoning_mobs::VICTORIA
        .construct
        .expect("constructible"))(&mut w);
    let slot = QuestTypeRef::Indexed(super::summoning::QUEST.slot as i32);
    pmob::data_mut(&mut w, pm)
        .expect("player")
        .done_quests
        .push((slot, DateTime::MIN));
    assert!(super::quester::can_talk_to(&mut w, victoria, pm));
    assert!(pmob::data(&w, pm).expect("player").done_quests.is_empty());
}

/// `Core.Now + restartDelay` throws when it overflows: the quest is cleared and no restart entry is written.
#[test]
fn an_overflowing_restart_delay_records_nothing() {
    use muo_content::player_mobile::{self as pmob, PLAYER_MOBILE};
    use muo_primitives::TimeSpan;

    static HUGE: super::core::QClass = super::core::QClass {
        restart_delay: TimeSpan::from_ticks(i64::MAX - 1),
        ..hatchlings::QUEST
    };
    let mut w = world_with_quests();
    let pm = w.new_mobile(&PLAYER_MOBILE);
    let mut q = Quest::new(&w, &HUGE, pm);
    q.attached = true;
    q.clear_quest(&mut w, true);
    assert!(pmob::data(&w, pm).expect("player").done_quests.is_empty());
}
