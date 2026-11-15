//! `BagOfSending`, `BallOfSummoning` and `BraceletOfBinding`: the three
//! `TranslocationItem` classes, registered in [`SpecialHooks::translocation`](super::hooks::SpecialHooks) for the
//! powder of translocation.
//!
//! (`PetSummoningSpell.Stop()` is never called.)

use std::rc::Rc;

use muo_content::creature::{self, control};
use muo_content::items::jewel::{BASE_BRACELET, BASE_BRACELET_CLASS};
use muo_content::props::{self, Ctor, Kind, Param, PropDef, Value, add_ctors};
use muo_content::spells::spell::{self, SPELL_BASE, SpellId, SpellType};
use muo_content::spells::{DisturbType, SpellInfo, TravelCheckType, helper};
use muo_content::utilities::types::{Numeric, PropType};

use muo_primitives::{MessageType, TimeSpan, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, base, container, ops};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages as mm;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

use super::hooks::{TranslocationApi, hooks, hooks_mut};
use super::util::{deleted, msg};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));

/// `item.SendLocalizedMessageTo(to, number, hue)`.
fn item_msg(w: &mut World, item: ItemId, to: MobileId, number: i32, hue: i32) {
    muo_server::item::messages::send_localized_hue(w, item, to, number, hue, "");
}

/// `Region.IsPartOf<JailRegion>()`.
fn in_jail(w: &World, m: MobileId) -> bool {
    let r = w.mobile(m).map(|x| x.region);
    r.is_some_and(|r| muo_server::region::is_part_of_type_name(w, r, "JailRegion"))
}

/// `item.IsChildOf(from.Backpack)`.
fn in_pack(w: &mut World, id: ItemId, from: MobileId) -> bool {
    super::util::in_pack(w, id, from)
}

/// `item.IsChildOf(from)`.
fn child_of_mobile(w: &World, id: ItemId, from: MobileId) -> bool {
    ops::is_child_of(w, id, Some(EntityId::Mobile(from)))
}

// ---- BagOfSending ----

/// `BagOfSending._bagOfSendingHue`, `_charges` and `_recharges`.
#[derive(Clone, Copy, Debug, Default)]
struct BagData {
    hue: i32,
    charges: i32,
    recharges: i32,
}

const BAG_MAX_CHARGES: i32 = 30;
const BAG_MAX_RECHARGES: i32 = 255;

fn bag(w: &World, id: ItemId) -> BagData {
    w.item(id)
        .and_then(|i| i.ext.get::<BagData>())
        .copied()
        .unwrap_or_default()
}

fn set_bag_raw(w: &mut World, id: ItemId, d: BagData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

/// `BagOfSendingHue = value`: the field changed handler sets `Hue`.
fn set_bag_hue(w: &mut World, id: ItemId, value: i32) {
    let mut d = bag(w, id);
    if d.hue == value {
        return;
    }
    d.hue = value;
    set_bag_raw(w, id, d);
    match value {
        0 => super::hue(w, id, 0x8A5),
        1 => super::hue(w, id, 0x8AD),
        2 => super::hue(w, id, 0x89B),
        _ => {}
    }
}

fn set_bag_charges(w: &mut World, id: ItemId, n: i32) {
    let mut d = bag(w, id);
    let n = n.clamp(0, BAG_MAX_CHARGES);
    if d.charges != n {
        d.charges = n;
        set_bag_raw(w, id, d);
        ops::invalidate_properties(w, id);
    }
}

fn set_bag_recharges(w: &mut World, id: ItemId, n: i32) {
    let mut d = bag(w, id);
    let n = n.clamp(0, BAG_MAX_RECHARGES);
    if d.recharges != n {
        d.recharges = n;
        set_bag_raw(w, id, d);
        ops::invalidate_properties(w, id);
    }
}

/// `BagOfSending.RandomHue()`.
pub fn bag_random_hue(w: &mut World) -> i32 {
    match utility::random(&mut w.rng, 3) {
        0 => 0,
        1 => 1,
        _ => 2,
    }
}

/// `BagOfSending(hue)`.
pub fn bag_of_sending_new_with_hue(w: &mut World, hue: i32) -> ItemId {
    let id = crate::simple::from_template(w, &BAG_OF_SENDING, "Server.Items.BagOfSending");
    set_bag_raw(w, id, BagData::default());
    set_bag_hue(w, id, hue);
    let charges = utility::random_min_max(&mut w.rng, 3, 9);
    let mut d = bag(w, id);
    d.charges = charges;
    set_bag_raw(w, id, d);
    id
}

/// `BagOfSending()`.
pub fn bag_of_sending_new(w: &mut World) -> ItemId {
    let hue = bag_random_hue(w);
    bag_of_sending_new_with_hue(w, hue)
}

fn bag_entries(w: &mut World, id: ItemId, from: MobileId, list: &mut Vec<ContextMenuEntry>) {
    (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
    if muo_server::mobile::base::alive(w, from) {
        let pack = muo_server::mobile::part2::items::find_item_on_layer(w, from, Layer::Backpack)
            .map(EntityId::Item);
        let enabled = bag(w, id).charges > 0 && ops::is_child_of(w, id, pack);
        let mut e = ContextMenuEntry::new(6189, -1);
        e.enabled = enabled;
        e.on_click = Some(Rc::new(|w, from, target| {
            let EntityId::Item(bag) = target else { return };
            if muo_server::mobile::part2::stats::check_alive(w, from, true)
                && !deleted(w, bag)
                && let Some(ty) = w.item(bag).map(|i| i.ty)
            {
                (ty.on_double_click)(w, bag, from);
            }
        }));
        list.push(e);
    }
}

fn bag_send_target(w: &mut World, bag_id: ItemId, from: MobileId, t: Targeted) {
    if deleted(w, bag_id) {
        return;
    }
    let Targeted::Entity(EntityId::Item(item)) = t else {
        return;
    };
    if in_jail(w, from) {
        super::util::text(w, from, "You may not do that in jail.");
        return;
    }
    if !in_pack(w, bag_id, from) {
        // The bag of sending must be in your backpack.
        item_msg(w, bag_id, from, 1062334, 0x59);
        return;
    }
    if bag(w, bag_id).charges == 0 {
        item_msg(w, bag_id, from, 1042544, 0x59); // This item is out of charges.
        return;
    }
    let weight = ops::get_total(w, item, muo_server::item::TotalType::Weight);
    let req_charges = ((weight as f64 / 10.0).ceil() as i32).max(1);
    let ml = muo_content::core::ml(w);
    let (is_bag, is_container, cursed) = w.item(item).map_or((false, false, false), |i| {
        (
            i.ty.is(&BAG_OF_SENDING),
            i.ty.is(&muo_server::item::container::CONTAINER),
            i.loot_type == LootType::Cursed,
        )
    });
    if !in_pack_item(w, item, from) {
        // You may only send items from your backpack to your bank box.
        item_msg(w, bag_id, from, 1054152, 0x59);
    } else if is_bag || is_container {
        // You cannot send a container through the bag of sending
        item_msg(w, bag_id, from, 1079428, 0x59);
    } else if cursed {
        // The bag of sending rejects the cursed item.
        item_msg(w, bag_id, from, 1054108, 0x59);
    } else if !base::verify_move(w, item, from)
        || muo_content::factory::item_is(w, item, "Server.Engines.Quests.QuestItem")
        || base::nontransferable(w, item)
    {
        // The bag of sending rejects that item.
        item_msg(w, bag_id, from, 1054109, 0x59);
    } else if w.mobile(from).is_some_and(|m| {
        m.map
            .is_some_and(|map| helper::is_doom_gauntlet(map, m.location))
    }) {
        msg(w, from, 1062089); // You cannot use that here.
    } else if ml && req_charges > bag(w, bag_id).charges {
        msg(w, from, 1079932); // You don't have enough charges to send that much weight
    } else {
        let sent;
        if muo_content::factory::item_is(w, item, "Server.Items.Gold")
            || muo_content::factory::item_is(w, item, "Server.Items.BankCheck")
        {
            let amount = if muo_content::factory::item_is(w, item, "Server.Items.Gold") {
                w.item(item).map_or(0, |i| i.amount)
            } else {
                muo_content::items::gold::worth(w, item)
            };
            sent = muo_content::vendor::banker::deposit(w, from, amount, true);
            if sent {
                super::util::delete(w, item);
            }
        } else {
            let bank = muo_server::mobile::part1::props_b::bank_box(w, from);
            sent = bank.is_some_and(|b| container::try_drop_item(w, b, Some(from), item, false));
        }
        if sent {
            let c = bag(w, bag_id).charges;
            set_bag_charges(w, bag_id, c - if ml { req_charges } else { 1 });
            // The item was placed in your bank box.
            item_msg(w, bag_id, from, 1054150, 0x59);
        } else {
            item_msg(w, bag_id, from, 1054110, 0x59); // Your bank box is full.
        }
    }
}

/// `item.IsChildOf(from.Backpack)`.
fn in_pack_item(w: &mut World, item: ItemId, from: MobileId) -> bool {
    super::util::in_pack(w, item, from)
}

/// `Server.Items.BagOfSending` (`TranslocationItem`; `[SerializationGenerator(2, false)]`).
pub static BAG_OF_SENDING: ItemType = ItemType {
    full_name: "Server.Items.BagOfSending",
    parent: Some(&ITEM_BASE),
    construct: Some(bag_of_sending_new),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // a bag of sending
    get_properties: |w, id, list: &mut PropertyList| {
        (ITEM_BASE.get_properties)(w, id, list);
        list.add_int(1060741, bag(w, id).charges); // charges: ~1_val~
    },
    on_single_click: |w, id, from| {
        (ITEM_BASE.on_single_click)(w, id, from);
        ops::label_to_number(w, id, from, 1060741, &bag(w, id).charges.to_string()); // charges: ~1_val~
    },
    get_context_menu_entries: bag_entries,
    on_double_click: |w, id, from| {
        if in_jail(w, from) {
            super::util::text(w, from, "You may not do that in jail.");
        } else if !in_pack(w, id, from) {
            // The bag of sending must be in your backpack.
            item_msg(w, id, from, 1062334, 0x59);
        } else if bag(w, id).charges == 0 {
            item_msg(w, id, from, 1042544, 0x59); // This item is out of charges.
        } else {
            target::begin_target(w, from, -1, false, TargetFlags::NONE, move |w, from, t| {
                bag_send_target(w, id, from, t)
            });
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(2);
        let d = bag(w, id);
        out.write_enum(4, i64::from(d.hue));
        out.write_i32(d.charges);
        out.write_i32(d.recharges);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut d = BagData::default();
        if version < 2 {
            d.recharges = r.read_encoded_int()?;
            d.charges = r.read_encoded_int()?;
            d.hue = r.read_encoded_int()?;
        } else {
            d.hue = r.read_enum(4)? as i32;
            d.charges = r.read_i32()?;
            d.recharges = r.read_i32()?;
        }
        set_bag_raw(w, id, d);
        Ok(())
    },
    ..ITEM_BASE
};

// ---- BallOfSummoning ----

/// `BallOfSummoning._recharges`, `_charges`, `_pet` and `_petName`.
#[derive(Clone, Debug, Default)]
struct BallData {
    recharges: i32,
    charges: i32,
    pet: Option<MobileId>,
    pet_name: Option<String>,
}

const BALL_MAX_CHARGES: i32 = 20;
const BALL_MAX_RECHARGES: i32 = 255;

fn ball(w: &World, id: ItemId) -> BallData {
    w.item(id)
        .and_then(|i| i.ext.get::<BallData>())
        .cloned()
        .unwrap_or_default()
}

fn ball_mut(w: &mut World, id: ItemId) -> Option<&mut BallData> {
    w.item_mut(id)?.ext.get_mut::<BallData>()
}

fn set_ball(w: &mut World, id: ItemId, d: BallData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

/// `InternalUpdatePetName()`.
fn update_pet_name(w: &mut World, id: ItemId) {
    let pet = ball(w, id).pet;
    let name = pet
        .and_then(|p| w.mobile(p).and_then(|m| (m.ty.name)(w, p)))
        .unwrap_or_default();
    if let Some(d) = ball_mut(w, id) {
        d.pet_name = Some(name);
    }
    ops::invalidate_properties(w, id);
}

/// `Pet` getter: a deleted pet is forgotten.
fn ball_pet(w: &mut World, id: ItemId) -> Option<MobileId> {
    let pet = ball(w, id).pet;
    if pet.is_some_and(|p| w.mobile(p).is_none_or(|m| m.deleted)) {
        if let Some(d) = ball_mut(w, id) {
            d.pet = None;
        }
        update_pet_name(w, id);
        return None;
    }
    pet
}

/// `Pet = value`.
fn set_ball_pet(w: &mut World, id: ItemId, pet: Option<MobileId>) {
    if let Some(d) = ball_mut(w, id) {
        d.pet = pet;
    }
    update_pet_name(w, id);
}

fn set_ball_charges(w: &mut World, id: ItemId, n: i32) {
    let n = n.clamp(0, BALL_MAX_CHARGES);
    if let Some(d) = ball_mut(w, id)
        && d.charges != n
    {
        d.charges = n;
        ops::invalidate_properties(w, id);
    }
}

fn set_ball_recharges(w: &mut World, id: ItemId, n: i32) {
    let n = n.clamp(0, BALL_MAX_RECHARGES);
    if let Some(d) = ball_mut(w, id)
        && d.recharges != n
    {
        d.recharges = n;
        ops::invalidate_properties(w, id);
    }
}

/// `BallOfSummoning()`.
pub fn ball_of_summoning_new(w: &mut World) -> ItemId {
    let id = crate::simple::from_template(w, &BALL_OF_SUMMONING, "Server.Items.BallOfSummoning");
    let charges = utility::random_min_max(&mut w.rng, 3, 9);
    set_ball(
        w,
        id,
        BallData {
            charges,
            ..BallData::default()
        },
    );
    id
}

fn ball_name_text(w: &World, id: ItemId) -> String {
    let d = ball(w, id);
    let name = d
        .pet_name
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| " ".to_owned());
    format!("{}\t{}", d.charges, name)
}

/// `RootParent == from`.
fn root_is(w: &World, id: ItemId, from: MobileId) -> bool {
    ops::root_parent(w, id) == Some(EntityId::Mobile(from))
}

fn ball_entries(w: &mut World, id: ItemId, from: MobileId, list: &mut Vec<ContextMenuEntry>) {
    (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
    if !muo_server::mobile::base::alive(w, from) || !root_is(w, id, from) {
        return;
    }
    let entry = |number: i32, f: fn(&mut World, ItemId, MobileId)| {
        let mut e = ContextMenuEntry::new(number, 2);
        e.on_click = Some(Rc::new(move |w, from, target| {
            let EntityId::Item(b) = target else { return };
            if muo_server::mobile::part2::stats::check_alive(w, from, true) {
                f(w, b, from);
            }
        }));
        e
    };
    if ball(w, id)
        .pet
        .is_none_or(|p| w.mobile(p).is_none_or(|m| m.deleted))
    {
        list.push(entry(6180, link_pet));
    } else {
        list.push(entry(6181, cast_summon_pet));
        list.push(entry(6183, |w, id, _| update_pet_name(w, id)));
        list.push(entry(6182, unlink_pet));
    }
}

/// `LinkPet(from)`.
fn link_pet(w: &mut World, id: ItemId, from: MobileId) {
    if deleted(w, id) || ball_pet(w, id).is_some() || !root_is(w, id, from) {
        return;
    }
    msg(w, from, 1054114); // Target your pet that you wish to link to this Crystal Ball of Pet Summoning.
    target::begin_target(w, from, -1, false, TargetFlags::NONE, move |w, from, t| {
        pet_link_target(w, id, from, t)
    });
}

fn pet_link_target(w: &mut World, id: ItemId, from: MobileId, t: Targeted) {
    if deleted(w, id) || ball_pet(w, id).is_some() {
        return;
    }
    if !root_is(w, id, from) {
        super::util::overhead(w, from, 0x3B2, 1042001); // That must be in your pack for you to use it.
        return;
    }
    match t {
        Targeted::Entity(EntityId::Mobile(c)) if creature::cd(w, c).is_some() => {
            let d = creature::cd(w, c).map(|d| (d.controlled, d.is_bonded));
            let master = control::control_master(w, c);
            if !d.is_some_and(|d| d.0) || master != Some(from) {
                // You may only link your own pets to a Crystal Ball of Pet Summoning.
                item_msg(w, id, from, 1054117, 0x59);
            } else if !d.is_some_and(|d| d.1) {
                // You must bond with your pet before it can be linked to a Crystal Ball of Pet Summoning.
                item_msg(w, id, from, 1054118, 0x59);
            } else {
                // Your pet is now linked to this Crystal Ball of Pet Summoning.
                item_msg(w, id, from, 1054119, 0x59);
                set_ball_pet(w, id, Some(c));
            }
        }
        Targeted::Entity(EntityId::Item(i)) if i == id => {
            // The Crystal Ball of Pet Summoning cannot summon itself.
            item_msg(w, id, from, 1054115, 0x59);
        }
        _ => {
            // Only pets can be linked to this Crystal Ball of Pet Summoning.
            item_msg(w, id, from, 1054116, 0x59);
        }
    }
}

/// `CastSummonPet(from)`.
fn cast_summon_pet(w: &mut World, id: ItemId, from: MobileId) {
    let pet = ball_pet(w, id);
    let is_pm = w
        .mobile(from)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE));
    let (Some(pet), true) = (pet, is_pm) else {
        return;
    };
    if deleted(w, id) || !root_is(w, id, from) {
        return;
    }
    let ml = muo_content::core::ml(w);
    if ball(w, id).charges == 0 {
        // The Crystal Ball darkens. It must be charged before it can be used again.
        item_msg(w, id, from, 1054122, 0);
        return;
    }
    let cd =
        creature::cd(w, pet).map(|d| (d.is_stabled, d.control_slots, d.controlled, d.is_bonded));
    let (stabled, slots, controlled, bonded) = cd.unwrap_or((false, 1, false, false));
    let (followers, max) = w
        .mobile(from)
        .map_or((0, 0), |m| (m.followers, m.followers_max));
    let pet_internal = w
        .mobile(pet)
        .is_some_and(|m| m.map == Some(MapId::INTERNAL));
    let in_stable = muo_content::player_mobile::data(w, from)
        .and_then(|d| d.stabled.as_ref())
        .is_some_and(|s| s.contains(&pet));
    let from_map = w.mobile(from).and_then(|m| m.map);
    let region = w.mobile(from).map(|m| m.region);
    let restricted_region = region.is_some_and(|r| {
        muo_content::regions::is_part_of_dungeon(w, r)
            || muo_server::region::is_part_of_type_name(w, r, "JailRegion")
            || muo_content::regions::get_safe_zone(w, r).is_some()
    });
    if creature::mount::rider(w, pet) == Some(from) {
        // The Crystal Ball fills with a yellow mist. Why would you summon your pet while riding it?
        item_msg(w, id, from, 1054124, 0x36);
    } else if pet_internal && (!stabled || followers + slots > max) {
        // The Crystal Ball fills with a blue mist. Your pet is not responding to the summons.
        item_msg(w, id, from, 1054125, 0x5);
    } else if (!controlled || control::control_master(w, pet) != Some(from)) && !in_stable {
        // The Crystal Ball fills with a grey mist. You are not the owner of the pet you are attempting to summon.
        item_msg(w, id, from, 1054126, 0x8FD);
    } else if !bonded {
        // The Crystal Ball fills with a red mist. You appear to have let your bond to your pet deteriorate.
        item_msg(w, id, from, 1054127, 0x22);
    } else if from_map == Some(MapId::ILSHENAR) || restricted_region {
        // You cannot summon your pet to this location.
        item_msg(w, id, from, 1080049, 0x22);
    } else if ml
        && muo_content::player_mobile::data(w, from)
            .is_some_and(|d| w.now < d.last_pet_ball_time + TimeSpan::from_secs(15))
    {
        // You must wait a few seconds before you can summon your pet.
        item_msg(w, id, from, 1080072, 0x22);
    } else if ml {
        let s = spell::new_spell(w, &PET_SUMMONING_SPELL, from, None);
        if let Some(sp) = spell::spells(w).get_mut(s) {
            sp.ext = muo_server::Ext::new(SpellBall(id));
        }
        spell::cast(w, s);
    } else {
        summon_pet(w, id, from);
    }
}

/// `SummonPet(from)`.
fn summon_pet(w: &mut World, id: ItemId, from: MobileId) {
    let pet = ball_pet(w, id);
    let is_pm = w
        .mobile(from)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE));
    let (Some(pet), true) = (pet, is_pm) else {
        return;
    };
    let c = ball(w, id).charges;
    set_ball_charges(w, id, c - 1);
    if creature::cd(w, pet).is_some_and(|d| d.is_stabled) {
        control::set_control_master(w, pet, Some(from));
        if let Some(d) = creature::cd_mut(w, pet) {
            d.control_target = Some(from);
        }
        control::set_control_order_raw(w, pet, muo_content::creature::enums::OrderType::Follow);
        if let Some(d) = creature::cd_mut(w, pet) {
            d.is_stabled = false;
            d.stabled_by = None;
        }
        if let Some(d) = muo_content::player_mobile::data_mut(w, from) {
            if let Some(s) = d.stabled.as_mut() {
                s.shift_remove(&pet);
            }
            if let Some(s) = d.auto_stabled.as_mut() {
                s.shift_remove(&pet);
            }
        }
    }
    let (loc, map) = w
        .mobile(from)
        .map_or((Default::default(), None), |m| (m.location, m.map));
    if let Some(map) = map {
        muo_server::mobile::part1::lifecycle::move_to_world(w, pet, loc, map);
    }
    // The Crystal Ball fills with a green mist. Your pet has been summoned.
    item_msg(w, id, from, 1054128, 0x43);
    let now = w.now;
    if let Some(d) = muo_content::player_mobile::data_mut(w, from) {
        d.last_pet_ball_time = now;
    }
}

/// `UnlinkPet(from)`.
fn unlink_pet(w: &mut World, id: ItemId, from: MobileId) {
    if !deleted(w, id) && ball_pet(w, id).is_some() && root_is(w, id, from) {
        set_ball_pet(w, id, None);
        item_msg(w, id, from, 1054120, 0); // This crystal ball is no longer linked to a pet.
    }
}

/// `PetSummoningSpell._ball`.
#[derive(Clone, Copy)]
struct SpellBall(ItemId);

fn spell_ball(w: &World, s: SpellId) -> Option<ItemId> {
    spell::spell(w, s)
        .and_then(|x| x.ext.get::<SpellBall>())
        .map(|b| b.0)
}

/// `PetSummoningSpell`.
pub static PET_SUMMONING_SPELL: SpellType = SpellType {
    full_name: "Server.Items.BallOfSummoning+PetSummoningSpell",
    parent: Some(&SPELL_BASE),
    info: SpellInfo::new("Ball Of Summoning", "", &[]).action(230),
    reveal_on_cast: true,
    clear_hands_on_cast: false,
    cast_delay_fast_scalar: 0.0,
    cast_delay_base: |_, _| TimeSpan::from_secs(2),
    get_cast_recovery: |_, _| TimeSpan::ZERO,
    get_mana: |_, _| 0,
    consume_reagents: |_, _| true,
    check_fizzle: |_, _| true,
    check_disturb: |_, _, t: DisturbType, _, _| {
        t != DisturbType::EquipRequest && t != DisturbType::UseRequest
    },
    on_disturb: |w, s, _t, message| {
        if message && let Some(c) = spell::caster(w, s) {
            msg(w, c, 1080074); // You have been disrupted while attempting to summon your pet!
        }
    },
    on_cast: |w, s| {
        if let (Some(b), Some(c)) = (spell_ball(w, s), spell::caster(w, s)) {
            summon_pet(w, b, c);
        }
        spell::finish_sequence(w, s);
    },
    ..SPELL_BASE
};

/// `[AfterDeserialization(false)]`: refreshes the pet name.
fn ball_after_deserialization(w: &mut World, id: ItemId) {
    if !deleted(w, id) {
        update_pet_name(w, id);
    }
}

/// `Server.Items.BallOfSummoning` (`TranslocationItem`; `[SerializationGenerator(2)]`).
pub static BALL_OF_SUMMONING: ItemType = ItemType {
    full_name: "Server.Items.BallOfSummoning",
    parent: Some(&ITEM_BASE),
    construct: Some(ball_of_summoning_new),
    default_weight: crate::class_props::default_weight,
    add_name_property: |w, id, list| {
        // a crystal ball of pet summoning: [charges: ~1_charges~] : [linked pet: ~2_petName~]
        list.add_str(1054131, &ball_name_text(w, id));
    },
    on_single_click: |w, id, from| {
        // a crystal ball of pet summoning: [charges: ~1_charges~] : [linked pet: ~2_petName~]
        ops::label_to_number(w, id, from, 1054131, &ball_name_text(w, id));
    },
    get_context_menu_entries: ball_entries,
    on_double_click: |w, id, from| {
        if !root_is(w, id, from) {
            // That must be in your pack for you to use it.
            super::util::overhead(w, from, 0x3B2, 1042001);
            return;
        }
        if muo_content::core::ml(w) && muo_content::player_mobile::hooks::animal_form_under(w, from)
        {
            // You cannot use a Crystal Ball of Pet Summoning while in animal form.
            super::util::overhead(w, from, 0x3B2, 1080073);
            return;
        }
        if ball_pet(w, id).is_none() {
            link_pet(w, id, from);
        } else {
            cast_summon_pet(w, id, from);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(2);
        let d = ball(w, id);
        out.write_i32(d.recharges);
        out.write_i32(d.charges);
        out.write_mobile(w, d.pet);
        out.write_opt_string(d.pet_name.as_deref());
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let mut d = BallData::default();
        if version < 2 {
            d.recharges = r.read_encoded_int()?;
            d.charges = r.read_encoded_int()?;
        } else {
            d.recharges = r.read_i32()?;
            d.charges = r.read_i32()?;
        }
        d.pet = r.read_mobile(w)?;
        d.pet_name = r.read_string()?;
        set_ball(w, id, d);
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            ball_after_deserialization(w, id)
        });
        Ok(())
    },
    ..ITEM_BASE
};

// ---- BraceletOfBinding ----

/// `BraceletOfBinding` fields; `transporting` is `_timer != null`.
#[derive(Clone, Debug, Default)]
struct BraceletData {
    recharges: i32,
    charges: i32,
    inscription: Option<String>,
    bound: Option<ItemId>,
    transporting: bool,
}

const BRACELET_MAX_CHARGES: i32 = 20;
const BRACELET_MAX_RECHARGES: i32 = 255;

fn bracelet(w: &World, id: ItemId) -> BraceletData {
    w.item(id)
        .and_then(|i| i.ext.get::<BraceletData>())
        .cloned()
        .unwrap_or_default()
}

fn bracelet_mut(w: &mut World, id: ItemId) -> Option<&mut BraceletData> {
    w.item_mut(id)?.ext.get_mut::<BraceletData>()
}

fn set_bracelet(w: &mut World, id: ItemId, d: BraceletData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

fn set_bracelet_charges(w: &mut World, id: ItemId, n: i32) {
    let n = n.clamp(0, BRACELET_MAX_CHARGES);
    if let Some(d) = bracelet_mut(w, id)
        && d.charges != n
    {
        d.charges = n;
        ops::invalidate_properties(w, id);
    }
}

fn set_bracelet_recharges(w: &mut World, id: ItemId, n: i32) {
    let n = n.clamp(0, BRACELET_MAX_RECHARGES);
    if let Some(d) = bracelet_mut(w, id)
        && d.recharges != n
    {
        d.recharges = n;
        ops::invalidate_properties(w, id);
    }
}

/// `Bound` getter.
fn bound(w: &World, id: ItemId) -> Option<ItemId> {
    bracelet(w, id).bound.filter(|&b| !deleted(w, b))
}

/// `BraceletOfBinding()`.
pub fn bracelet_of_binding_new(w: &mut World) -> ItemId {
    let id = muo_content::items::jewel::new_jewel(w, &BRACELET_OF_BINDING, 0x1086);
    set_bracelet(w, id, BraceletData::default());
    super::hue(w, id, 0x489);
    id
}

fn bracelet_label(w: &World, id: ItemId) -> String {
    let d = bracelet(w, id);
    let text = d
        .inscription
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| " ".to_owned());
    format!("{}\t{}", d.charges, text)
}

fn bracelet_entries(w: &mut World, id: ItemId, from: MobileId, list: &mut Vec<ContextMenuEntry>) {
    (BASE_BRACELET.get_context_menu_entries)(w, id, from, list);
    if muo_server::mobile::base::alive(w, from) && child_of_mobile(w, id, from) {
        let bound = bound(w, id).is_some();
        let entry = |number: i32, enabled: bool, f: fn(&mut World, ItemId, MobileId)| {
            let mut e = ContextMenuEntry::new(number, -1);
            e.enabled = enabled;
            e.on_click = Some(Rc::new(move |w, from, target| {
                let EntityId::Item(b) = target else { return };
                if muo_server::mobile::part2::stats::check_alive(w, from, true) {
                    f(w, b, from);
                }
            }));
            e
        };
        list.push(entry(6170, bound, activate));
        list.push(entry(6171, bound, search));
        list.push(entry(if bound { 6174 } else { 6173 }, true, bind));
        list.push(entry(6175, true, inscribe));
    }
}

/// `Activate(from)`.
fn activate(w: &mut World, id: ItemId, from: MobileId) {
    if deleted(w, id) || bound(w, id).is_none() {
        return;
    }
    if !child_of_mobile(w, id, from) {
        msg(w, from, 1042664); // You must have the object in your backpack to use it.
    } else if bracelet(w, id).transporting {
        // The bracelet is already attempting contact. You decide to wait a moment.
        msg(w, from, 1054013);
    } else {
        mm_play_sound(w, from, 0xF9);
        // * You concentrate on the bracelet to summon its power *
        mm::local_overhead_message_number(w, from, MessageType::REGULAR, 0x5D, 1151783, "");
        muo_server::mobile::part1::props_a::set_frozen(w, from, true);
        if let Some(d) = bracelet_mut(w, id) {
            d.transporting = true;
        }
        timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
            transport_tick(w, id, from)
        });
    }
}

fn mm_play_sound(w: &mut World, m: MobileId, sound: i32) {
    muo_server::mobile::part2::effects::play_sound(w, m, sound);
}

/// `TransportTimer.OnTick`.
fn transport_tick(w: &mut World, id: ItemId, from: MobileId) {
    if let Some(d) = bracelet_mut(w, id) {
        d.transporting = false;
    }
    muo_server::mobile::part1::props_a::set_frozen(w, from, false);
    if deleted(w, id) || w.mobile(from).is_none_or(|m| m.deleted) || !check_use(w, id, from, false)
    {
        return;
    }
    let Some(root) = bound(w, id).and_then(|b| ops::root_parent(w, b)) else {
        return;
    };
    let EntityId::Mobile(bound_root) = root else {
        return;
    };
    let c = bracelet(w, id).charges;
    set_bracelet_charges(w, id, c - 1);
    let (loc, map) = w
        .mobile(bound_root)
        .map_or((Default::default(), None), |m| (m.location, m.map));
    let Some(map) = map else { return };
    creature::pets::teleport_pets(w, from, loc, map, true);
    mm_play_sound(w, from, 0x1FC);
    muo_server::mobile::part1::lifecycle::move_to_world(w, from, loc, map);
    mm_play_sound(w, from, 0x1FC);
}

/// `Search(from)`.
fn search(w: &mut World, id: ItemId, from: MobileId) {
    if deleted(w, id) || bound(w, id).is_none() {
        return;
    }
    if !child_of_mobile(w, id, from) {
        msg(w, from, 1042664); // You must have the object in your backpack to use it.
    } else {
        check_use(w, id, from, true);
    }
}

/// `CheckUse(from, successMessage)`.
fn check_use(w: &mut World, id: ItemId, from: MobileId, success_message: bool) -> bool {
    let Some(b) = bound(w, id) else { return false };
    let bound_root = match ops::root_parent(w, b) {
        Some(EntityId::Mobile(m)) => Some(m),
        _ => None,
    };
    let send = |w: &mut World, number: i32, hue: i32| {
        muo_server::mobile::part2::messages::send_localized_message(w, from, number, "", hue);
    };
    if bracelet(w, id).charges == 0 {
        send(w, 1054005, 0); // The bracelet glows black. It must be charged before it can be used again.
        return false;
    }
    if muo_server::mobile::part2::items::find_item_on_layer(w, from, Layer::Bracelet) != Some(id) {
        send(w, 1054004, 0); // You must equip the bracelet in order to use its power.
        return false;
    }
    let root_ok = bound_root.is_some_and(|r| {
        w.mobile(r).is_some_and(|m| m.net_state.is_some())
            && muo_server::mobile::part2::items::find_item_on_layer(w, r, Layer::Bracelet)
                == Some(b)
    });
    let Some(bound_root) = bound_root.filter(|_| root_ok) else {
        // The bracelet emits a red glow. The bracelet's twin is not available for transport.
        send(w, 1054006, 0);
        return false;
    };
    let (from_map, from_region) = w
        .mobile(from)
        .map_or((None, None), |m| (m.map, Some(m.region)));
    let (root_map, root_loc, root_region) = w
        .mobile(bound_root)
        .map_or((None, Default::default(), None), |m| {
            (m.map, m.location, Some(m.region))
        });
    if !muo_content::core::aos(w) && from_map != root_map {
        send(w, 1054014, 0); // The bracelet glows black. The bracelet's target is on another facet.
        return false;
    }
    if hooks(w)
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, from))
    {
        send(w, 1061632, 0); // You can't do that while carrying the sigil.
        return false;
    }
    let c = helper::check_travel(w, from, TravelCheckType::RecallFrom);
    if !c.allowed {
        if let Some(m) = &c.message {
            muo_content::text::send_message_to(w, Some(m), from);
        }
        return false;
    }
    let c = helper::check_travel_at(w, Some(from), root_map, root_loc, TravelCheckType::RecallTo);
    if !c.allowed {
        if let Some(m) = &c.message {
            muo_content::text::send_message_to(w, Some(m), from);
        }
        return false;
    }
    let is_pm = w
        .mobile(from)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE));
    if root_map == Some(MapId::FELUCCA) && is_pm && muo_content::player_mobile::young(w, from) {
        send(w, 1049543, 0); // You decide against traveling to Felucca while you are still young.
        return false;
    }
    if muo_server::mobile::base::murderer(w, from) && root_map != Some(MapId::FELUCCA) {
        send(w, 1019004, 0); // You are not allowed to travel there.
        return false;
    }
    if w.mobile(from).is_some_and(|m| m.criminal) {
        send(w, 1005561, 0x22); // Thou'rt a criminal and cannot escape so easily.
        return false;
    }
    if helper::check_combat(w, from) {
        send(w, 1005564, 0x22); // Wouldst thou flee during the heat of battle??
        return false;
    }
    if muo_content::misc::stamina_system::is_overloaded(w, from) {
        send(w, 502359, 0x22); // Thou art too encumbered to move.
        return false;
    }
    let jail = |w: &World, r: Option<muo_server::region::RegionId>| {
        r.is_some_and(|r| muo_server::region::is_part_of_type_name(w, r, "JailRegion"))
    };
    if jail(w, from_region) {
        send(w, 1114345, 0x35); // You'll need a better jailbreak plan than that!
        return false;
    }
    if jail(w, root_region) {
        send(w, 1019004, 0); // You are not allowed to travel there.
        return false;
    }
    if success_message {
        send(w, 1054015, 0); // The bracelet's twin is available for transport.
    }
    true
}

/// `Bind(from)`.
fn bind(w: &mut World, id: ItemId, from: MobileId) {
    if deleted(w, id) {
        return;
    }
    if !child_of_mobile(w, id, from) {
        msg(w, from, 1042664); // You must have the object in your backpack to use it.
    } else {
        msg(w, from, 1054001); // Target the bracelet of binding you wish to bind this bracelet to.
        target::begin_target(w, from, -1, false, TargetFlags::NONE, move |w, from, t| {
            bind_target(w, id, from, t)
        });
    }
}

fn bind_target(w: &mut World, id: ItemId, from: MobileId, t: Targeted) {
    if deleted(w, id) {
        return;
    }
    if !child_of_mobile(w, id, from) {
        msg(w, from, 1042664); // You must have the object in your backpack to use it.
        return;
    }
    match t {
        Targeted::Entity(EntityId::Item(other))
            if w.item(other).is_some_and(|i| i.ty.is(&BRACELET_OF_BINDING)) =>
        {
            if other == id {
                msg(w, from, 1054012); // You cannot bind a bracelet of binding to itself!
            } else if !child_of_mobile(w, other, from) {
                msg(w, from, 1042664); // You must have the object in your backpack to use it.
            } else {
                // You bind the bracelet to its counterpart. The bracelets glow with power.
                msg(w, from, 1054003);
                mm_play_sound(w, from, 0x1FA);
                if bound(w, id) != Some(other)
                    && let Some(d) = bracelet_mut(w, id)
                {
                    d.bound = Some(other);
                }
            }
        }
        _ => msg(w, from, 1054002), // You can only bind this bracelet to another bracelet of binding!
    }
}

/// `Inscribe(from)`.
fn inscribe(w: &mut World, id: ItemId, from: MobileId) {
    if deleted(w, id) {
        return;
    }
    if !child_of_mobile(w, id, from) {
        msg(w, from, 1042664); // You must have the object in your backpack to use it.
    } else {
        msg(w, from, 1054009); // Enter the text to inscribe upon the bracelet :
        let cb: muo_server::prompt::PromptCallback =
            Rc::new(move |w: &mut World, from: MobileId, text: &str| {
                if deleted(w, id) {
                    return;
                }
                if !child_of_mobile(w, id, from) {
                    msg(w, from, 1042664);
                } else {
                    msg(w, from, 1054011); // You mark the bracelet with your inscription.
                    if let Some(d) = bracelet_mut(w, id) {
                        d.inscription = Some(text.to_owned());
                    }
                    ops::invalidate_properties(w, id);
                }
            });
        let cancel: muo_server::prompt::PromptCallback =
            Rc::new(|w: &mut World, from: MobileId, _: &str| {
                msg(w, from, 1054010); // You decide not to inscribe the bracelet at this time.
            });
        let p = muo_server::prompt::SimplePrompt::with_cancel(Some(cb), Some(cancel));
        muo_server::mobile::part2::speech::begin_prompt(w, from, p);
    }
}

/// `Server.Items.BraceletOfBinding` (`TranslocationItem`; `[SerializationGenerator(2)]`).
pub static BRACELET_OF_BINDING: ItemType = ItemType {
    full_name: "Server.Items.BraceletOfBinding",
    parent: Some(&BASE_BRACELET),
    construct: Some(bracelet_of_binding_new),
    family: Some(&BASE_BRACELET_CLASS),
    default_weight: crate::class_props::default_weight,
    add_name_property: |w, id, list| {
        // a bracelet of binding : ~1_val~ ~2_val~
        list.add_str(1054000, &bracelet_label(w, id));
    },
    on_single_click: |w, id, from| {
        // a bracelet of binding : ~1_val~ ~2_val~
        ops::label_to_number(w, id, from, 1054000, &bracelet_label(w, id));
    },
    get_context_menu_entries: bracelet_entries,
    on_double_click: |w, id, from| {
        if bound(w, id).is_none() {
            bind(w, id, from);
        } else {
            activate(w, id, from);
        }
    },
    serialize: |w, id, out| {
        (BASE_BRACELET.serialize)(w, id, out);
        out.write_encoded_int(2);
        let d = bracelet(w, id);
        out.write_i32(d.recharges);
        out.write_i32(d.charges);
        out.write_opt_string(d.inscription.as_deref());
        out.write_item(w, d.bound);
    },
    deserialize: |w, id, r| {
        (BASE_BRACELET.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let mut d = BraceletData::default();
        if version < 2 {
            d.recharges = r.read_encoded_int()?;
            d.charges = r.read_encoded_int()?.min(BRACELET_MAX_CHARGES);
            d.inscription = r.read_string()?;
            d.bound = r.read_item(w)?;
        } else {
            d.recharges = r.read_i32()?;
            d.charges = r.read_i32()?;
            d.inscription = r.read_string()?;
            d.bound = r.read_item(w)?;
        }
        set_bracelet(w, id, d);
        Ok(())
    },
    ..BASE_BRACELET
};

// ---- properties, constructors, translocation ----

macro_rules! int_prop {
    ($name:literal, $get:expr, $set:expr) => {
        PropDef {
            name: $name,
            kind: INT,
            read: AccessLevel::GameMaster,
            write: AccessLevel::GameMaster,
            get: |w, e| match e {
                EntityId::Item(i) => Value::Int(i64::from(($get)(w, i))),
                _ => Value::Null,
            },
            set: $set,
        }
    };
}

macro_rules! int_setter {
    ($f:expr) => {
        Some(|w: &mut World, e: EntityId, v: Value| {
            let EntityId::Item(i) = e else {
                return Err("not an item".to_string());
            };
            ($f)(w, i, v.i32());
            Ok(())
        })
    };
}

static BAG_PROPS: [PropDef; 4] = [
    PropDef {
        name: "BagOfSendingHue",
        kind: Kind::Plain(PropType::Enum(&[("Yellow", 0), ("Blue", 1), ("Red", 2)])),
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Enum(i64::from(bag(w, i).hue)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err("not an item".to_string());
            };
            set_bag_hue(w, i, v.i64() as i32);
            Ok(())
        }),
    },
    int_prop!(
        "Charges",
        |w: &World, i| bag(w, i).charges,
        int_setter!(set_bag_charges)
    ),
    int_prop!(
        "Recharges",
        |w: &World, i| bag(w, i).recharges,
        int_setter!(set_bag_recharges)
    ),
    int_prop!("MaxCharges", |_: &World, _| BAG_MAX_CHARGES, None),
];

static BALL_PROPS: [PropDef; 4] = [
    int_prop!(
        "Recharges",
        |w: &World, i| ball(w, i).recharges,
        int_setter!(set_ball_recharges)
    ),
    int_prop!(
        "Charges",
        |w: &World, i| ball(w, i).charges,
        int_setter!(set_ball_charges)
    ),
    PropDef {
        name: "Pet",
        kind: Kind::Plain(PropType::Entity),
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Entity(ball(w, i).pet.map(EntityId::Mobile)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err("not an item".to_string());
            };
            let pet = match v.entity() {
                Some(EntityId::Mobile(m)) if creature::cd(w, m).is_some() => Some(m),
                None => None,
                Some(_) => return Err("not a creature".to_string()),
            };
            set_ball_pet(w, i, pet);
            Ok(())
        }),
    },
    PropDef {
        name: "PetName",
        kind: Kind::Plain(PropType::Str),
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Str(ball(w, i).pet_name.unwrap_or_default()),
            _ => Value::Null,
        },
        set: None,
    },
];

static BRACELET_PROPS: [PropDef; 4] = [
    int_prop!(
        "Recharges",
        |w: &World, i| bracelet(w, i).recharges,
        int_setter!(set_bracelet_recharges)
    ),
    int_prop!(
        "Charges",
        |w: &World, i| bracelet(w, i).charges,
        int_setter!(set_bracelet_charges)
    ),
    PropDef {
        name: "Inscription",
        kind: Kind::Plain(PropType::Str),
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Str(bracelet(w, i).inscription.unwrap_or_default()),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err("not an item".to_string());
            };
            if let Some(d) = bracelet_mut(w, i) {
                d.inscription = v.string();
            }
            ops::invalidate_properties(w, i);
            Ok(())
        }),
    },
    PropDef {
        name: "Bound",
        kind: Kind::Plain(PropType::Entity),
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Entity(bound(w, i).map(EntityId::Item)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err("not an item".to_string());
            };
            let b = match v.entity() {
                Some(EntityId::Item(b))
                    if w.item(b).is_some_and(|x| x.ty.is(&BRACELET_OF_BINDING)) =>
                {
                    Some(b)
                }
                None => None,
                Some(_) => return Err("not a bracelet of binding".to_string()),
            };
            if let Some(d) = bracelet_mut(w, i) {
                d.bound = b;
            }
            Ok(())
        }),
    },
];

static BAG_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| EntityId::Item(bag_of_sending_new(w)),
    },
    Ctor {
        access: None,
        params: &[Param {
            name: "hue",
            kind: Kind::Plain(PropType::Enum(&[("Yellow", 0), ("Blue", 1), ("Red", 2)])),
            default: None,
            ty: "BagOfSendingHue",
        }],
        make: |w, v| EntityId::Item(bag_of_sending_new_with_hue(w, v[0].i64() as i32)),
    },
];

pub fn register(types: &mut TypeRegistry) {
    for t in [&BAG_OF_SENDING, &BALL_OF_SUMMONING, &BRACELET_OF_BINDING] {
        types.register_item(t, &[]);
    }
}

pub(super) fn configure(w: &mut World) {
    props::add(w, "Server.Items.BagOfSending", &BAG_PROPS);
    props::add(w, "Server.Items.BallOfSummoning", &BALL_PROPS);
    props::add(w, "Server.Items.BraceletOfBinding", &BRACELET_PROPS);
    add_ctors(w, "Server.Items.BagOfSending", &BAG_CTORS);
    let t = &mut hooks_mut(w).translocation;
    t.push(TranslocationApi {
        is_item: |w, id| w.item(id).is_some_and(|i| i.ty.is(&BAG_OF_SENDING)),
        get: |w, id| {
            let d = bag(w, id);
            (d.charges, d.recharges, BAG_MAX_CHARGES, BAG_MAX_RECHARGES)
        },
        set: |w, id, c, r| {
            set_bag_charges(w, id, c);
            set_bag_recharges(w, id, r);
        },
        name: |_, _| (1150423, None), // bag of sending
    });
    t.push(TranslocationApi {
        is_item: |w, id| w.item(id).is_some_and(|i| i.ty.is(&BALL_OF_SUMMONING)),
        get: |w, id| {
            let d = ball(w, id);
            (d.charges, d.recharges, BALL_MAX_CHARGES, BALL_MAX_RECHARGES)
        },
        set: |w, id, c, r| {
            set_ball_charges(w, id, c);
            set_ball_recharges(w, id, r);
        },
        name: |_, _| (1054129, None), // crystal ball of pet summoning
    });
    t.push(TranslocationApi {
        is_item: |w, id| w.item(id).is_some_and(|i| i.ty.is(&BRACELET_OF_BINDING)),
        get: |w, id| {
            let d = bracelet(w, id);
            (
                d.charges,
                d.recharges,
                BRACELET_MAX_CHARGES,
                BRACELET_MAX_RECHARGES,
            )
        },
        set: |w, id, c, r| {
            set_bracelet_charges(w, id, c);
            set_bracelet_recharges(w, id, r);
        },
        name: |_, _| (0, Some("bracelet of binding".to_owned())),
    });
}
