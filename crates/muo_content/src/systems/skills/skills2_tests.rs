//! Tests of Animal Lore/Taming, the bard skills, Poisoning, Inscribe and
//! Stealing.

use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::mobile::MOBILE_BASE;

use super::*;

fn target_of(w: &World, m: muo_server::ids::MobileId) -> &muo_server::target::Target {
    &w.targets[&w.mobile(m).unwrap().target.expect("a target was set")]
}

#[test]
fn animal_lore_and_taming_open_their_targets() {
    let mut w = World::new();
    w.core.expansion = Some(muo_data::expansion::Expansion::T2A);
    let m = w.new_mobile(&MOBILE_BASE);
    assert_eq!(animal::lore_on_use(&mut w, m), TimeSpan::from_secs(1));
    assert_eq!(target_of(&w, m).range, 8);

    assert_eq!(animal::taming_on_use(&mut w, m), TimeSpan::from_secs(30));
    assert_eq!(target_of(&w, m).range, 2, "2 before AOS");
}

#[test]
fn taming_stays_silent_while_the_context_menu_runs_it() {
    let mut w = World::new();
    w.resources
        .get_mut::<animal::AnimalTamingState>()
        .disable_message = true;
    assert!(animal::disable_message(&w));
}

#[test]
fn without_instruments_the_bard_skills_do_nothing_but_wait_a_second() {
    let mut w = World::new();
    let m = w.new_mobile(&MOBILE_BASE);
    for f in [
        bard::peacemaking_on_use,
        bard::discordance_on_use,
        bard::provocation_on_use,
    ] {
        assert_eq!(f(&mut w, m), TimeSpan::from_secs(1));
        assert!(
            w.mobile(m).unwrap().target.is_none(),
            "no instrument, no targeter"
        );
    }
}

#[test]
fn the_bard_range_grows_with_the_skill() {
    let mut w = World::new();
    let m = w.new_mobile(&MOBILE_BASE);
    assert_eq!(
        bard::get_bard_range(&mut w, m, muo_server::skills::SkillName::Peacemaking),
        8
    );
    assert!(bard::discordance_effect(&w, m).is_none());
}

#[test]
fn poisoning_asks_for_the_potion_first() {
    let mut w = World::new();
    let m = w.new_mobile(&MOBILE_BASE);
    assert_eq!(poisoning::on_use(&mut w, m), TimeSpan::from_secs(10));
    assert_eq!(target_of(&w, m).range, 2);
}

#[test]
fn inscribe_waits_a_minute_for_a_book() {
    let mut w = World::new();
    let m = w.new_mobile(&MOBILE_BASE);
    assert_eq!(inscribe::on_use(&mut w, m), TimeSpan::from_secs(1));
    let t = target_of(&w, m);
    assert_eq!(t.range, 3);
    assert_ne!(t.timeout_time, 0, "BeginTimeout(from, 60000)");
}

#[test]
fn stealing_needs_both_hands_free_and_a_safe_area() {
    let mut w = World::new();
    let m = w.new_mobile(&MOBILE_BASE);
    assert_eq!(stealing::on_use(&mut w, m), TimeSpan::from_secs(30));
    let t = target_of(&w, m);
    assert_eq!((t.range, t.allow_nonlocal), (1, true));
}

#[test]
fn a_stolen_item_is_remembered_for_two_minutes() {
    let mut w = World::new();
    w.now = muo_primitives::DateTime::utc_now();
    let thief = w.new_mobile(&MOBILE_BASE);
    let victim = w.new_mobile(&MOBILE_BASE);
    let item = muo_server::item::ops::new_item_with_id(&mut w, &muo_server::item::ITEM_BASE, 0xEED);

    assert_eq!(stealing::is_stolen(&mut w, item), None);
    stealing::stolen_add(&mut w, item, thief, Some(victim));
    assert_eq!(stealing::is_stolen(&mut w, item), Some(Some(victim)));
    assert!(stealing::is_stolen_ref(&w, item));

    w.now += TimeSpan::from_secs(121);
    assert_eq!(stealing::is_stolen(&mut w, item), None, "expired");
    assert!(!stealing::is_stolen_ref(&w, item));
}

#[test]
fn the_skill_handlers_register_their_callbacks() {
    let mut w = World::new();
    initialize_handlers(&mut w);
    let t = w.resources.get::<muo_server::skills::SkillTable>().unwrap();
    for s in [
        muo_server::skills::SkillName::AnimalLore,
        muo_server::skills::SkillName::AnimalTaming,
        muo_server::skills::SkillName::Discordance,
        muo_server::skills::SkillName::Inscribe,
        muo_server::skills::SkillName::Peacemaking,
        muo_server::skills::SkillName::Poisoning,
        muo_server::skills::SkillName::Provocation,
        muo_server::skills::SkillName::Stealing,
    ] {
        assert!(t.callbacks[s as usize].is_some(), "{s:?}");
    }
}
