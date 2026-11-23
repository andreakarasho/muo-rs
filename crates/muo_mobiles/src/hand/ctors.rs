//! Helpers of the monster and animal classes and of the creature template keys (`crate::template_keys`).

use muo_content::creature::data::cd_mut;
use muo_content::creature::enums::{AIType, FightMode};
use muo_content::factory;
use muo_primitives::utility;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::base as ibase;
use muo_server::mobile::part2::items;

pub(crate) fn si(name: &str) -> String {
    format!("Server.Items.{name}")
}

/// `AddItem(new T { Hue = hue })` / `new T(hue)`.
pub(crate) fn add_hue(w: &mut World, id: MobileId, name: &str, hue: i32) -> Option<ItemId> {
    let it = factory::make_item(w, &si(name))?;
    ibase::set_hue(w, it, hue);
    items::add_item(w, id, it);
    Some(it)
}

/// `AddItem(new T(Utility.RandomNondyedHue()))`: the hue is rolled before the constructor.
pub(crate) fn add_nondyed(w: &mut World, id: MobileId, name: &str) {
    let hue = utility::random_nondyed_hue(&mut w.rng);
    add_hue(w, id, name, hue);
}

/// `Race.<name>`.
pub(crate) fn race_named(w: &World, name: &str) -> Option<muo_server::race::RaceId> {
    w.resources
        .get::<muo_server::race::Races>()
        .and_then(|r| r.try_parse(name))
}

/// `AI = ai`.
pub(crate) fn set_ai(w: &mut World, id: MobileId, ai: AIType) {
    let ai = if ai == AIType::AiUseDefault {
        muo_content::creature::data::read(w, id, ai, |d| d.default_ai)
    } else {
        ai
    };
    if let Some(d) = cd_mut(w, id) {
        d.current_ai = ai;
    }
    muo_content::ai::change_ai_type(w, id, ai);
}

/// `FightMode = mode`.
pub(crate) fn set_fight_mode(w: &mut World, id: MobileId, mode: FightMode) {
    if let Some(d) = cd_mut(w, id) {
        d.fight_mode = mode;
    }
}

// ---- armour and clothing packs ----

// ---- the dragoons ----
