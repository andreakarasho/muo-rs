//! EvilMage, EvilMageLord, SpawnedOrcishLord, Quagmire and SummonedAirElemental: constructor parts and the
//! `[AfterDeserialization]` fixes they run when loaded (the class saves only its version).

use muo_content::creature::BASE_CREATURE;
use muo_io::{GenericReader, GenericWriter};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part1::props_b;

use super::ctors::add_nondyed;
use crate::common::rnd;

/// The human body of the evil mages before Core.UOR: skin, hair and facial hair.
pub(crate) fn human_look(w: &mut World, id: MobileId) {
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
