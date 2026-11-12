//! EvilMage, EvilMageLord, SpawnedOrcishLord, Quagmire and SummonedAirElemental: constructor parts and the
//! `[AfterDeserialization]` fixes they run when loaded (the class saves only its version).

use muo_content::creature::BASE_CREATURE;
use muo_content::factory;
use muo_io::{GenericReader, GenericWriter};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::items;

use super::ctors::{add_nondyed, si};
use crate::common::rnd;

/// `Utility.RandomNeutralHue()`.
fn neutral_hue(w: &mut World) -> i32 {
    rnd::from(w, 1801, 108)
}

/// `EquipItem(item)`: equips the item or deletes it when it cannot be worn.
fn equip(w: &mut World, id: MobileId, name: &str, hue: Option<i32>) {
    let Some(it) = factory::make_item(w, &si(name)) else {
        return;
    };
    if let Some(h) = hue {
        muo_server::item::base::set_hue(w, it, h);
    }
    if !muo_server::mobile::base::equip_item(w, id, it)
        && let Some(t) = w.item(it).map(|i| i.ty)
    {
        (t.delete)(w, it);
    }
}

/// The human body of the evil mages before Core.UOR: skin, hair and facial hair.
fn human_look(w: &mut World, id: MobileId) {
    crate::common::body(w, id, 0x190);
    let skin = muo_content::vendor::human_random_skin_hue(w);
    crate::common::hue(w, id, skin);
    muo_content::vendor::assign_random_hair_random_hue(w, id);
    let hh = w.mobile(id).map_or(0, |m| m.hair_hue);
    muo_content::vendor::assign_random_facial_hair(w, id, hh);
}

fn uor(w: &World) -> bool {
    w.expansion() >= muo_data::expansion::Expansion::UOR
}

pub fn evil_mage_ctor1(w: &mut World, id: MobileId) {
    if uor(w) {
        crate::common::body(w, id, 124);
    } else {
        human_look(w, id);
    }
}
pub fn evil_mage_ctor2(w: &mut World, id: MobileId) {
    let h = neutral_hue(w);
    equip(w, id, "Robe", Some(h));
}
pub fn evil_mage_ctor3(w: &mut World, id: MobileId) {
    equip(w, id, "Sandals", None);
}

pub fn evil_mage_lord_ctor1(w: &mut World, id: MobileId) {
    if uor(w) {
        let b = rnd::from(w, 125, 2);
        crate::common::body(w, id, b);
    } else {
        human_look(w, id);
    }
}
pub fn evil_mage_lord_ctor2(w: &mut World, id: MobileId) {
    let hue = neutral_hue(w);
    equip(w, id, "Robe", Some(hue));
    equip(w, id, "WizardsHat", Some(hue));
}
pub fn evil_mage_lord_ctor3(w: &mut World, id: MobileId) {
    equip(w, id, "Sandals", None);
}

/// The look the saved mage had is brought to the current era (shared by both classes' `AfterDeserialization`).
fn mage_after_deserialization(w: &mut World, id: MobileId, _lord: bool) {
    let body = w.mobile(id).map_or(0, |m| m.cur_body().0);
    if uor(w) {
        if body == 0x190 {
            let b = rnd::from(w, 125, 2); // both classes roll the same
            crate::common::body(w, id, b);
            props_b::set_hair_item_id(w, id, 0);
            props_b::set_hair_hue(w, id, 0);
            props_b::set_facial_hair_item_id(w, id, 0);
            props_b::set_facial_hair_item_id(w, id, 0);
            crate::common::hue(w, id, 0);
        }
    } else if body == 125 || body == 126 {
        human_look(w, id);
    }
}

pub fn evil_mage_serialize(_w: &World, _id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
}
pub fn evil_mage_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    mage_after_deserialization(w, id, false);
    Ok(())
}
pub fn evil_mage_lord_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    mage_after_deserialization(w, id, true);
    Ok(())
}

// ---- SpawnedOrcishLord ----

/// `var pack = Backpack; pack?.Delete(); NoKillAwards = true;`
pub fn spawned_orcish_lord_ctor1(w: &mut World, id: MobileId) {
    if let Some(p) = items::find_item_on_layer(w, id, muo_server::layer::Layer::Backpack) {
        muo_server::item::base::delete(w, p);
    }
    set_no_kill_awards(w, id);
}
fn set_no_kill_awards(w: &mut World, id: MobileId) {
    if let Some(d) = muo_content::creature::data::cd_mut(w, id) {
        d.no_kill_awards = true;
    }
}
/// `base.OnDeath(c); c.Delete();`
pub fn spawned_orcish_lord_on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    (BASE_CREATURE.on_death)(w, id, c);
    if let Some(c) = c {
        muo_server::item::base::delete(w, c);
    }
}
pub fn spawned_orcish_lord_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    set_no_kill_awards(w, id);
    Ok(())
}

// ---- Quagmire, SummonedAirElemental ----

pub fn quagmire_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    if w.mobile(id).is_some_and(|m| m.base_sound_id == -1) {
        crate::common::base_sound_id(w, id, 352);
    }
    Ok(())
}
pub fn summoned_air_elemental_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    if w.mobile(id).is_some_and(|m| m.base_sound_id == 263) {
        crate::common::base_sound_id(w, id, 655);
    }
    Ok(())
}

#[allow(dead_code)]
fn _unused(w: &mut World) {
    let _ = add_nondyed;
    let _ = w;
}
