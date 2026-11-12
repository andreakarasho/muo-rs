//! Helpers the table class constructors call: one per property/method a constructor sets, named after it.

use muo_content::creature::data::cd_mut;
pub use muo_content::creature::setters::pack_item;
use muo_primitives::Body;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::items;
use muo_server::world::World;

pub use muo_content::creature::enums::*;
pub use muo_content::creature::packing::*;
pub use muo_content::creature::setters::*;
pub use muo_content::hooks::rnd;
pub use muo_content::loot_pack;
pub use muo_server::mobile::ResistanceType;
pub use muo_server::skills::SkillName;

/// `Name = NameList.RandomName(kind)`.
pub fn name_random(w: &mut World, id: MobileId, kind: &str) {
    let n = muo_content::misc::small::random_name(w, kind);
    name(w, id, Some(n));
}

/// `Name = value`.
pub fn name(w: &mut World, id: MobileId, value: Option<String>) {
    if let Some(f) = w.mobile(id).map(|m| m.ty.set_name) {
        f(w, id, value);
    }
}

pub fn title(w: &mut World, id: MobileId, value: &str) {
    props_a::set_title(w, id, Some(value.to_owned()));
}

pub fn body(w: &mut World, id: MobileId, value: i32) {
    props_b::set_body(w, id, Body(value));
}

/// `Body = Utility.RandomList(a, b, ..)`.
pub fn body_list(w: &mut World, id: MobileId, list: &[i32]) {
    let i = rnd::below(w, list.len() as i32) as usize;
    body(w, id, list[i]);
}

pub fn hue(w: &mut World, id: MobileId, value: i32) {
    muo_server::mobile::base::set_hue(w, id, value);
}

pub fn base_sound_id(w: &mut World, id: MobileId, value: i32) {
    if let Some(m) = w.mobile_mut(id) {
        m.base_sound_id = value;
    }
}

pub fn speech_hue(w: &mut World, id: MobileId, value: i32) {
    if let Some(m) = w.mobile_mut(id) {
        m.speech_hue = value;
    }
}

pub fn fame(w: &mut World, id: MobileId, value: i32) {
    props_a::set_fame(w, id, value);
}

pub fn karma(w: &mut World, id: MobileId, value: i32) {
    props_a::set_karma(w, id, value);
}

pub fn virtual_armor(w: &mut World, id: MobileId, value: i32) {
    props_a::set_virtual_armor(w, id, value);
}

pub fn can_swim(w: &mut World, id: MobileId, value: bool) {
    if let Some(m) = w.mobile_mut(id) {
        m.can_swim = value;
    }
}

pub fn cant_walk(w: &mut World, id: MobileId, value: bool) {
    if let Some(m) = w.mobile_mut(id) {
        m.cant_walk = value;
    }
}

pub fn control_slots(w: &mut World, id: MobileId, value: i32) {
    if let Some(d) = cd_mut(w, id) {
        d.control_slots = value;
    }
}

pub fn min_tame_skill(w: &mut World, id: MobileId, value: f64) {
    if let Some(d) = cd_mut(w, id) {
        d.min_tame_skill = value;
    }
}

pub fn tamable(w: &mut World, id: MobileId, value: bool) {
    if let Some(d) = cd_mut(w, id) {
        d.tamable_raw = value;
    }
}

/// `IsParagon = value`: the property setter (`Paragon.Convert` scales the stats, skills, speeds and hue).
pub fn is_paragon(w: &mut World, id: MobileId, value: bool) {
    muo_content::creature::overrides::set_is_paragon(w, id, value);
}

/// `PackItem(new T())`: the item class is made by its full name.
pub fn pack_new(w: &mut World, id: MobileId, full_name: &str) {
    let it = muo_content::factory::make_item(w, full_name);
    pack_item(w, id, it);
}

/// `PackItem(new T(amount))` for a stackable class whose `(int amount)` constructor only sets `Amount`.
pub fn pack_new_amount(w: &mut World, id: MobileId, full_name: &str, amount: i32) {
    let it = muo_content::factory::make_item(w, full_name);
    if let Some(i) = it {
        muo_server::item::ops::set_amount(w, i, amount);
    }
    pack_item(w, id, it);
}

/// `AddItem(new T())`.
pub fn add_new(w: &mut World, id: MobileId, full_name: &str) -> Option<ItemId> {
    let it = muo_content::factory::make_item(w, full_name)?;
    items::add_item(w, id, it);
    Some(it)
}

/// `PackItem(item)` of an item made by `new T()` that may be `null` (a `switch` arm that is `_ => null`).
pub fn pack_opt(w: &mut World, id: MobileId, full_name: Option<&str>) {
    if let Some(n) = full_name {
        pack_new(w, id, n);
    }
}

pub use muo_content::creature::data::read;

/// `c.DropItem(new T())` on the corpse container.
pub fn drop_new(w: &mut World, c: Option<ItemId>, full_name: &str) {
    let Some(c) = c else { return };
    if let Some(it) = muo_content::factory::make_item(w, full_name) {
        muo_server::item::container::drop_item_default(w, c, it);
    }
}

/// `Female = value`.
pub fn female(w: &mut World, id: MobileId, value: bool) {
    props_b::set_female(w, id, value);
}

/// `Female`.
pub fn is_female(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.female)
}

/// `Utility.AssignRandomHair(this)`.
pub fn assign_random_hair(w: &mut World, id: MobileId) {
    muo_content::vendor::assign_random_hair_random_hue(w, id);
}

fn named_race_ops(w: &World, name: &str) -> Option<&'static muo_server::race::RaceOps> {
    let races = w.resources.get::<muo_server::race::Races>()?;
    let id = races.try_parse(name)?;
    Some(races.get(id)?.ops)
}

/// `Race.<name>.RandomSkinHue()`.
pub fn race_skin_hue(w: &mut World, name: &str) -> i32 {
    match named_race_ops(w, name) {
        Some(o) => (o.random_skin_hue)(&mut w.rng),
        None => 0,
    }
}

/// `Race.<name>.RandomHairHue()`.
pub fn race_hair_hue(w: &mut World, name: &str) -> i32 {
    match named_race_ops(w, name) {
        Some(o) => (o.random_hair_hue)(&mut w.rng),
        None => 0,
    }
}

/// `Skills.<name>.Base = value`.
pub fn skill_base(w: &mut World, id: MobileId, name: SkillName, value: impl Into<f64>) {
    muo_server::skills::set_base(w, id, name, value.into());
}

/// `InitStats(str, dex, int)`.
pub fn init_stats(w: &mut World, id: MobileId, str: i32, dex: i32, intel: i32) {
    muo_server::mobile::part2::stats::init_stats(w, id, str, dex, intel);
}

/// `PackItem(new <Ore>(amount) { ItemID = item_id })`.
pub fn pack_ore(w: &mut World, id: MobileId, full_name: &str, amount: i32, item_id: i32) {
    let it = muo_content::factory::make_item(w, full_name);
    if let Some(i) = it {
        muo_server::item::ops::set_amount(w, i, amount);
        muo_server::item::base::set_item_id(w, i, item_id);
    }
    pack_item(w, id, it);
}

/// `AddItem(item)` of an item that may be `null`.
pub fn add_opt(w: &mut World, id: MobileId, full_name: Option<&str>) {
    if let Some(n) = full_name {
        add_new(w, id, n);
    }
}

/// `AddItem(new T(hue))` of a clothing class whose `(int hue)` constructor only sets the hue.
pub fn add_new_hue(w: &mut World, id: MobileId, full_name: &str, hue: i32) {
    if let Some(i) = muo_content::factory::make_item(w, full_name) {
        muo_server::item::base::set_hue(w, i, hue);
        items::add_item(w, id, i);
    }
}

/// The per-world tables of single monster classes (class-wide state).
#[derive(Default)]
pub struct MobileTables {
    /// Ilhenir `m_Table`.
    pub cacophonic: std::collections::HashSet<MobileId>,
    /// KhaldunRevenant `m_Set`.
    pub khaldun: std::collections::HashSet<MobileId>,
    /// RaiJu `m_Table`.
    pub rai_ju: std::collections::HashSet<MobileId>,
    /// Satyr `m_Suppressed`.
    pub satyr: std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>,
    /// MeerMage `m_Table`.
    pub meer_mage: std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>,
    pub swoop: std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>,
    pub flurry: std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>,
    pub chlorophyll: std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>,
    pub snow: std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>,
    /// DemonKnight's re-entry guard.
    pub demon_knight_in_here: bool,
}

pub fn tables(w: &mut World) -> &mut MobileTables {
    w.resources.get_mut::<MobileTables>()
}
