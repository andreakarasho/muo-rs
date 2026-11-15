//! `SwampDragon`: a mount with optional barding (serialization version 2).

use muo_content::craft::{self, CraftResource};
use muo_content::creature::mount::{BASE_MOUNT, BASE_MOUNT_C, set_item_id};
use muo_io::{GenericReader, GenericWriter};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::type_registry::TypeRegistry;

use crate::common::*;
use crate::leaf::creature_leaf;

/// The fields of the class.
#[derive(Clone, Debug, Default)]
pub struct SwampDragonData {
    pub barding_exceptional: bool,
    pub barding_crafted_by: Option<String>,
    pub barding_hp: i32,
    pub has_barding: bool,
    pub barding_resource: CraftResource,
}

pub fn data(w: &World, id: MobileId) -> Option<&SwampDragonData> {
    w.mobile(id)?.ext.get::<SwampDragonData>()
}
fn data_mut(w: &mut World, id: MobileId) -> Option<&mut SwampDragonData> {
    w.mobile_mut(id)?.ext.get_mut::<SwampDragonData>()
}

pub(crate) fn swamp_dragon_data(w: &mut World, id: MobileId) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(SwampDragonData::default());
    }
}

fn swamp_dragon_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &SWAMP_DRAGON)
}

/// `BardingMaxHP`.
pub fn barding_max_hp(w: &World, id: MobileId) -> i32 {
    if data(w, id).is_some_and(|d| d.barding_exceptional) {
        2500
    } else {
        1000
    }
}

/// `HasBarding` set: the look follows (`OnHasBardingChanged`).
pub fn set_has_barding(w: &mut World, id: MobileId, value: bool) {
    let Some(d) = data_mut(w, id) else { return };
    if d.has_barding == value {
        return;
    }
    d.has_barding = value;
    let res = d.barding_resource;
    if value {
        let h = craft::hue(w, res);
        hue(w, id, h);
        body(w, id, 0x31F);
        set_item_id(w, id, 0x3EBE);
    } else {
        hue(w, id, 0x851);
        body(w, id, 0x31A);
        set_item_id(w, id, 0x3EBD);
    }
    muo_server::mobile::part2::properties::invalidate_properties(w, id);
}

/// `BardingResource` set: a barded dragon takes the resource's hue.
pub fn set_barding_resource(w: &mut World, id: MobileId, value: CraftResource) {
    let Some(d) = data_mut(w, id) else { return };
    if d.barding_resource == value {
        return;
    }
    d.barding_resource = value;
    let has = d.has_barding;
    if has {
        let h = craft::hue(w, value);
        hue(w, id, h);
    }
}

/// `new SwampDragon { HasBarding = true, BardingResource = res }; mt.BardingHP = mt.BardingMaxHP;`
pub fn equip_barding(w: &mut World, id: MobileId, res: CraftResource) {
    set_has_barding(w, id, true);
    set_barding_resource(w, id, res);
    let hp = barding_max_hp(w, id);
    if let Some(d) = data_mut(w, id) {
        d.barding_hp = hp;
    }
}

fn serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(2);
    let d = data(w, id).cloned().unwrap_or_default();
    out.write_bool(d.barding_exceptional);
    out.write_opt_string(d.barding_crafted_by.as_deref());
    out.write_bool(d.has_barding);
    out.write_i32(d.barding_hp);
    out.write_enum(4, d.barding_resource as i64);
}

fn deserialize(w: &mut World, id: MobileId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    // Versions below 2 are laid out the same.
    let _version = r.read_i32()?;
    let d = SwampDragonData {
        barding_exceptional: r.read_bool()?,
        barding_crafted_by: r.read_string()?,
        has_barding: r.read_bool()?,
        barding_hp: r.read_i32()?,
        barding_resource: CraftResource::from_i32(r.read_enum(4)? as i32).unwrap_or_default(),
    };
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(d);
    }
    Ok(())
}

creature_leaf! {
    /// `Server.Mobiles.SwampDragon`.
    SWAMP_DRAGON, SWAMP_DRAGON_C = "Server.Mobiles.SwampDragon" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(swamp_dragon_new), state serialize, deserialize;
    c {
        default_name: muo_content::templates::creature_props::default_name,
        corpse_name: muo_content::templates::creature_props::corpse_name,
        steps_max: muo_content::templates::creature_props::steps_max,
        reacquire_on_movement: crate::cprops::reacquire_on_movement,
        auto_dispel: |w, id| !muo_content::creature::cd(w, id).is_some_and(|d| d.controlled),
        favorite_food: crate::cprops::favorite_food,
        meat: muo_content::templates::creature_props::meat,
        hides: muo_content::templates::creature_props::hides,
        scales: muo_content::templates::creature_props::scales,
        scale_type: crate::cprops::scale_type,
        can_anger_on_tame: crate::cprops::can_anger_on_tame,
        override_bonding_reqs: crate::cprops::override_bonding_reqs,
        get_control_chance: |_w, _id, _m, _b| 1.0,
    }
    m {
        get_idle_sound: |_w, _id| 0x2CE,
        get_death_sound: |_w, _id| 0x2CC,
        get_hurt_sound: |_w, _id| 0x2D1,
        get_attack_sound: |_w, _id| 0x2C8,
        get_properties: get_properties,
    }
}

/// `GetProperties`: an exceptional barding names its maker.
fn get_properties(w: &World, id: MobileId, list: &mut muo_server::property_list::PropertyList) {
    (BASE_MOUNT.get_properties)(w, id, list);
    if let Some(d) = data(w, id)
        && d.has_barding
        && d.barding_exceptional
        && let Some(by) = &d.barding_crafted_by
    {
        list.add_str(1060853, by); // armor exceptionally crafted by ~1_val~
    }
}

/// `AOS.Damage`: `m.Mount is SwampDragon { HasBarding: true } pet` absorbs 10% (20% exceptional) of the damage.
pub fn barding_absorb(w: &mut World, m: MobileId, total: i32) -> i32 {
    let Some(pet) = muo_server::mobile::part1::props_b::mount(w, m) else {
        return total;
    };
    if !muo_content::factory::mobile_is(w, pet, "Server.Mobiles.SwampDragon")
        || !data(w, pet).is_some_and(|d| d.has_barding)
    {
        return total;
    }
    let percent = if data(w, pet).is_some_and(|d| d.barding_exceptional) {
        20
    } else {
        10
    };
    let absorbed = muo_content::misc::aos_helpers::scale(total, percent);
    let hp = data(w, pet).map_or(0, |d| d.barding_hp) - absorbed;
    if let Some(d) = data_mut(w, pet) {
        d.barding_hp = hp;
    }
    if hp < 0 {
        set_has_barding(w, pet, false);
        if let Some(d) = data_mut(w, pet) {
            d.barding_hp = 0;
        }
        muo_content::mx::msg(w, m, 1053031); // Your dragon's barding has been destroyed!
    }
    muo_server::mobile::part2::properties::invalidate_properties(w, pet);
    total - absorbed
}

pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&SWAMP_DRAGON, &[]);
}

/// `targeted is SwampDragon pet && pet.HasBarding` (BladedItemTarget): the whole branch; false if it is no barded
/// swamp dragon.
pub fn remove_barding(w: &mut World, from: MobileId, pet: MobileId) -> bool {
    if !muo_content::factory::mobile_is(w, pet, "Server.Mobiles.SwampDragon")
        || !data(w, pet).is_some_and(|d| d.has_barding)
    {
        return false;
    }
    let (controlled, master) =
        muo_content::creature::cd(w, pet).map_or((false, None), |d| (d.controlled, d.master));
    if !controlled || master != Some(from) {
        muo_content::mx::msg(w, from, 1053022); // You cannot remove barding from a swamp dragon you do not own.
    } else {
        set_has_barding(w, pet, false);
    }
    true
}

/// `targeted is SwampDragon pet && pet.HasBarding` (ArmsLore): `(BardingHP, BardingMaxHP)`.
pub fn barding_facts(w: &World, pet: MobileId) -> Option<(i32, i32)> {
    let d = data(w, pet).filter(|d| d.has_barding)?;
    Some((d.barding_hp, barding_max_hp(w, pet)))
}

/// `(HasBarding, Controlled, ControlMaster)` of a swamp dragon (`DragonBardingDeed.OnTarget`); None for another mobile.
pub fn deed_pet_state(w: &World, pet: MobileId) -> Option<(bool, bool, Option<MobileId>)> {
    if !muo_content::factory::mobile_is(w, pet, "Server.Mobiles.SwampDragon") {
        return None;
    }
    let has = data(w, pet).is_some_and(|d| d.has_barding);
    let (controlled, master) =
        muo_content::creature::cd(w, pet).map_or((false, None), |d| (d.controlled, d.master));
    Some((has, controlled, master))
}

/// The end of `DragonBardingDeed.OnTarget`: the deed's barding goes on the dragon.
pub fn apply_deed_barding(
    w: &mut World,
    pet: MobileId,
    exceptional: bool,
    crafted_by: Option<String>,
    resource: CraftResource,
    deed_hue: i32,
) {
    if let Some(d) = data_mut(w, pet) {
        d.barding_exceptional = exceptional;
        d.barding_crafted_by = crafted_by;
    }
    let hp = barding_max_hp(w, pet);
    if let Some(d) = data_mut(w, pet) {
        d.barding_hp = hp;
    }
    set_barding_resource(w, pet, resource);
    set_has_barding(w, pet, true);
    hue(w, pet, deed_hue);
}
