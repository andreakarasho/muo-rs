//! `AnimalTrainer`, `GypsyAnimalTrainer` and `RealEstateBroker`: the stable master (stable
//! and claim by speech, context menu, target and the claim list gump) and the broker who buys house deeds.
//! Both classes are `[SerializationGenerator(0, false)]` over `BaseVendor`.

use muo_content::creature::data::cd_mut;
use muo_content::creature::enums::MAX_LOYALTY;
use muo_content::creature::{cd, control};
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::system::send_gump_to_mobile;
use muo_content::hooks::rnd;
use muo_content::player_mobile::{self, add_stabled, remove_stabled};
use muo_content::vendor::{
    BASE_VENDOR, BASE_VENDOR_C, BASE_VENDOR_CLASS, ShoeType, VendorClass, banker,
    init_outfit as base_init_outfit, new_vendor,
};
use muo_content::{factory, mx};
use muo_primitives::{MessageType, Point3D, TimeSpan};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{items, messages};
use muo_server::mobile::{MobileType, part2};
use muo_server::net_state::NetStateId;
use muo_server::skills::SkillName;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::TypeRegistry;

use super::hand::rehue_layer_bright;
use super::sb::*;
use crate::common::{OrderType, add_new};

// ---- AnimalTrainer ----

/// `GetMaxStabled(from)`.
pub fn get_max_stabled(w: &mut World, from: MobileId) -> i32 {
    let taming = muo_server::skills::value(w, from, SkillName::AnimalTaming);
    let anlore = muo_server::skills::value(w, from, SkillName::AnimalLore);
    let vetern = muo_server::skills::value(w, from, SkillName::Veterinary);
    let sum = taming + anlore + vetern;
    let mut max = if sum >= 240.0 {
        5
    } else if sum >= 200.0 {
        4
    } else if sum >= 160.0 {
        3
    } else {
        2
    };
    if taming >= 100.0 {
        max += ((taming - 90.0) / 10.0) as i32;
    }
    if anlore >= 100.0 {
        max += ((anlore - 90.0) / 10.0) as i32;
    }
    if vetern >= 100.0 {
        max += ((vetern - 90.0) / 10.0) as i32;
    }
    max
}

fn stabled_of(w: &World, pm: MobileId) -> Vec<MobileId> {
    player_mobile::data(w, pm)
        .and_then(|d| d.stabled.as_ref())
        .map_or_else(Vec::new, |s| s.iter().copied().collect())
}

fn is_stabled_by_pm(w: &World, pm: MobileId, pet: MobileId) -> bool {
    player_mobile::data(w, pm)
        .and_then(|d| d.stabled.as_ref())
        .is_some_and(|s| s.contains(&pet))
}

fn set_pet_stabled(w: &mut World, pet: MobileId, stabled: bool, by: Option<MobileId>) {
    if let Some(d) = cd_mut(w, pet) {
        d.is_stabled = stabled;
        d.stabled_by = by;
    }
}

fn is_player(w: &World, m: MobileId) -> bool {
    player_mobile::data(w, m).is_some()
}

fn say_to(w: &mut World, id: MobileId, to: MobileId, number: i32) {
    part2::speech::say_to_number(w, id, to, number, "");
}

fn say_to_args(w: &mut World, id: MobileId, to: MobileId, number: i32, args: &str) {
    part2::speech::say_to_number(w, id, to, number, args);
}

fn close_claim_list(w: &mut World, from: MobileId) {
    if let Some(ns) = w.mobile(from).and_then(|m| m.net_state) {
        muo_content::gumps::system::close_gump::<ClaimListGump>(w, ns);
    }
}

/// `BeginClaimList(from)`.
pub fn begin_claim_list(w: &mut World, id: MobileId, from: MobileId) {
    if mx::deleted(w, id) || !check_alive(w, from) || !is_player(w, from) {
        return;
    }
    let mut list = Vec::new();
    let mut queue = Vec::new();
    for m in stabled_of(w, from) {
        if cd(w, m).is_some() {
            if !mx::deleted(w, m) {
                list.push(m);
                continue;
            }
            set_pet_stabled(w, m, false, None);
        }
        queue.push(m);
    }
    for m in queue {
        remove_stabled(w, from, m);
    }
    if !list.is_empty() {
        claim_list_display_to(w, id, from, list);
    } else {
        say_to(w, id, from, 502671); // But I have no animals stabled with me at the moment!
    }
}

/// `from.CheckAlive()`.
fn check_alive(w: &mut World, from: MobileId) -> bool {
    muo_server::mobile::part2::stats::check_alive(w, from, true)
}

/// `EndClaimList(from, pet)`.
pub fn end_claim_list(w: &mut World, id: MobileId, from: MobileId, pet: MobileId) {
    if mx::deleted(w, pet)
        || mx::map(w, from) != mx::map(w, id)
        || !is_player(w, from)
        || !is_stabled_by_pm(w, from, pet)
        || !check_alive(w, from)
    {
        return;
    }
    if !mx::in_range(w, from, id, 14) {
        mx::msg(w, from, 500446); // That is too far away.
        return;
    }
    if can_claim(w, from, pet) {
        do_claim(w, from, pet);
        remove_stabled(w, from, pet);
        if let Some(d) = player_mobile::data_mut(w, from)
            && let Some(a) = d.auto_stabled.as_mut()
        {
            a.shift_remove(&pet);
        }
    } else {
        let name = w
            .mobile(pet)
            .and_then(|m| (m.ty.name)(w, pet))
            .unwrap_or_default();
        say_to_args(w, id, from, 1049612, &name); // ~1_NAME~ remained in the stables because you have too many followers.
    }
}

/// `BeginStable(from)`.
pub fn begin_stable(w: &mut World, id: MobileId, from: MobileId) {
    if mx::deleted(w, id) || !check_alive(w, from) {
        return;
    }
    let gold = props_b::backpack(w, from).map_or(0, |p| {
        muo_server::item::container::get_amount(w, p, &muo_content::items::gold::GOLD, true)
    });
    if !(gold >= 30) && !(banker::get_balance(w, from) >= 30) {
        say_to(w, id, from, 1042556); // Thou dost not have enough gold, not even in thy bank account.
    } else {
        /* I charge 30 gold per pet for a real week's stable time.
         * I will withdraw it from thy bank account.
         * Which animal wouldst thou like to stable here?
         */
        mx::msg(w, from, 1042558);
        let mut t =
            target::Target::simple(w, 12, TargetFlags::NONE, false, move |w, from, targeted| {
                if let Targeted::Entity(EntityId::Mobile(c)) = targeted
                    && cd(w, c).is_some()
                {
                    end_stable(w, id, from, c);
                } else if targeted == Targeted::Entity(EntityId::Mobile(from)) {
                    say_to(w, id, from, 502672); // HA HA HA! Sorry, I am not an inn.
                } else {
                    say_to(w, id, from, 1048053); // You can't stable that!
                }
            });
        t.check_los = true;
        let tid = target::add_target(w, t);
        target::set_mobile_target(w, from, Some(tid));
    }
}

/// `EndStable(from, pet)`.
pub fn end_stable(w: &mut World, id: MobileId, from: MobileId, pet: MobileId) {
    if mx::deleted(w, id) || !check_alive(w, from) || !is_player(w, from) {
        return;
    }
    let d = cd(w, pet).map(|d| (d.controlled, d.is_dead_pet, d.summoned));
    let Some((controlled, dead_pet, summoned)) = d else {
        return;
    };
    let human = muo_content::skills::common::is_human(w, pet);
    let busy = mx::combatant(w, pet)
        .is_some_and(|c| mx::in_range(w, pet, c, 12) && mx::map(w, pet) == mx::map(w, c));
    let pack_animal = ["PackLlama", "PackHorse", "Beetle"]
        .iter()
        .any(|n| factory::mobile_is(w, pet, &format!("Server.Mobiles.{n}")));
    let pack_has_items =
        props_b::backpack(w, pet).is_some_and(|p| w.item(p).is_some_and(|i| !i.items.is_empty()));
    let stabled_count = stabled_of(w, from).len() as i32;
    let max_stabled = get_max_stabled(w, from);
    if human {
        say_to(w, id, from, 502672); // HA HA HA! Sorry, I am not an inn.
    } else if !controlled {
        say_to(w, id, from, 1048053); // You can't stable that!
    } else if control::control_master(w, pet) != Some(from) {
        say_to(w, id, from, 1042562); // You do not own that pet!
    } else if dead_pet {
        say_to(w, id, from, 1049668); // Living pets only, please.
    } else if summoned {
        say_to(w, id, from, 502673); // I can not stable summoned creatures.
    } else if pack_animal && pack_has_items {
        say_to(w, id, from, 1042563); // You need to unload your pet.
    } else if busy {
        say_to(w, id, from, 1042564); // I'm sorry.  Your pet seems to be busy.
    } else if stabled_count >= max_stabled {
        say_to(w, id, from, 1042565); // You have too many pets in the stables!
    } else {
        let paid = props_b::backpack(w, from).is_some_and(|p| {
            muo_server::item::container::consume_total(
                w,
                p,
                &muo_content::items::gold::GOLD,
                30,
                true,
                None,
            )
        }) || banker::withdraw(w, from, 30);
        if paid {
            if let Some(d) = cd_mut(w, pet) {
                d.control_target = None;
            }
            control::set_control_order_raw(w, pet, OrderType::Stay);
            items::internalize(w, pet);
            control::set_control_master(w, pet, None);
            set_pet_stabled(w, pet, true, Some(from));
            if w.core_se() {
                control::set_loyalty(w, pet, MAX_LOYALTY); // Wonderfully happy
            }
            add_stabled(w, from, pet);
            // [AOS: Your pet has been stabled.] Very well, thy pet is stabled. Thou mayst recover it by saying 'claim' to me. In one real world week, I shall sell it off if it is not claimed!
            let n = if w.core_aos() { 1049677 } else { 502679 };
            say_to(w, id, from, n);
        } else {
            say_to(w, id, from, 502677); // But thou hast not the funds in thy bank account!
        }
    }
}

/// `Claim(from, petName)`.
pub fn claim(w: &mut World, id: MobileId, from: MobileId, pet_name: Option<&str>) {
    if mx::deleted(w, id) || !check_alive(w, from) || !is_player(w, from) {
        return;
    }
    let mut claimed = false;
    let mut stabled = 0;
    let by_name = pet_name.is_some();
    let mut queue = Vec::new();
    for m in stabled_of(w, from) {
        let is_pet = cd(w, m).is_some();
        if !is_pet || mx::deleted(w, m) {
            if is_pet {
                set_pet_stabled(w, m, false, None);
            }
            queue.push(m);
            continue;
        }
        stabled += 1;
        if let Some(n) = pet_name {
            let name = w
                .mobile(m)
                .and_then(|x| (x.ty.name)(w, m))
                .unwrap_or_default();
            if !name.eq_ignore_ascii_case(n) {
                continue;
            }
        }
        if can_claim(w, from, m) {
            do_claim(w, from, m);
            queue.push(m);
            claimed = true;
            if let Some(a) = player_mobile::data_mut(w, from).and_then(|d| d.auto_stabled.as_mut())
            {
                a.shift_remove(&m);
            }
        } else {
            let name = w
                .mobile(m)
                .and_then(|x| (x.ty.name)(w, m))
                .unwrap_or_default();
            say_to_args(w, id, from, 1049612, &name); // ~1_NAME~ remained in the stables because you have too many followers.
        }
    }
    for m in queue {
        remove_stabled(w, from, m);
    }
    if claimed {
        say_to(w, id, from, 1042559); // Here you go... and good day to you!
    } else if stabled == 0 {
        say_to(w, id, from, 502671); // But I have no animals stabled with me at the moment!
    } else if by_name {
        begin_claim_list(w, id, from);
    }
}

/// `CanClaim(from, pet)`.
pub fn can_claim(w: &World, from: MobileId, pet: MobileId) -> bool {
    let (followers, max) = w
        .mobile(from)
        .map_or((0, 0), |m| (m.followers, m.followers_max));
    followers + cd(w, pet).map_or(1, |d| d.control_slots) <= max
}

fn do_claim(w: &mut World, from: MobileId, pet: MobileId) {
    control::set_control_master(w, pet, Some(from));
    if let Some(d) = cd_mut(w, pet) {
        d.control_target = Some(from);
    }
    control::set_control_order_raw(w, pet, OrderType::Follow);
    if let Some(map) = mx::map(w, from) {
        muo_server::mobile::base::move_to_world(w, pet, mx::location(w, from), map);
    }
    set_pet_stabled(w, pet, false, None);
    if w.core_se() {
        control::set_loyalty(w, pet, MAX_LOYALTY); // Wonderfully Happy
    }
}

fn handles_on_speech(_w: &World, _id: MobileId, _from: MobileId) -> bool {
    true
}

fn on_speech(w: &mut World, id: MobileId, e: &mut muo_server::events::SpeechEventArgs) {
    if !e.handled && e.has_keyword(0x0008) {
        // *stable*
        e.handled = true;
        close_claim_list(w, e.mobile);
        begin_stable(w, id, e.mobile);
    } else if !e.handled && e.has_keyword(0x0009) {
        // *claim*
        e.handled = true;
        close_claim_list(w, e.mobile);
        match e.speech.find(' ') {
            Some(index) => {
                let name = e.speech[index..].trim().to_owned();
                claim(w, id, e.mobile, Some(&name));
            }
            None => claim(w, id, e.mobile, None),
        }
    } else {
        (BASE_VENDOR.on_speech)(w, id, e);
    }
}

fn add_custom_context_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    if is_player(w, from) && mx::alive(w, from) {
        let mut e = ContextMenuEntry::new(6126, 12);
        e.on_click = Some(std::rc::Rc::new(|w, from, target| {
            if let EntityId::Mobile(t) = target
                && factory::mobile_is(w, t, "Server.Mobiles.AnimalTrainer")
            {
                begin_stable(w, t, from);
            }
        }));
        list.push(e);
        if !stabled_of(w, from).is_empty() {
            let mut e = ContextMenuEntry::new(6127, 12);
            e.on_click = Some(std::rc::Rc::new(|w, from, target| {
                if let EntityId::Mobile(t) = target
                    && factory::mobile_is(w, t, "Server.Mobiles.AnimalTrainer")
                {
                    claim(w, t, from, None);
                }
            }));
            list.push(e);
        }
    }
    (BASE_VENDOR_C.add_custom_context_entries)(w, id, from, list);
}

// ---- ClaimListGump ----

pub struct ClaimListGump {
    trainer: MobileId,
    list: Vec<(MobileId, String)>,
}

/// `ClaimListGump.DisplayTo(trainer, from, list)`.
fn claim_list_display_to(w: &mut World, trainer: MobileId, from: MobileId, list: Vec<MobileId>) {
    if w.mobile(from).is_none_or(|m| m.net_state.is_none()) || list.is_empty() {
        return;
    }
    let list = list
        .into_iter()
        .map(|p| {
            (
                p,
                w.mobile(p)
                    .and_then(|m| (m.ty.name)(w, p))
                    .unwrap_or_default(),
            )
        })
        .collect();
    send_gump_to_mobile(
        w,
        from,
        Box::new(DynamicGumpObj::new(50, 50, ClaimListGump { trainer, list })),
        true,
    );
}

impl DynamicGumpLogic for ClaimListGump {
    const TYPE_NAME: &'static str = "Server.Mobiles.AnimalTrainer+ClaimListGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let n = self.list.len() as i32;
        b.add_page(0);
        b.add_background(0, 0, 325, 50 + n * 20, 9250);
        b.add_alpha_region(5, 5, 315, 40 + n * 20);
        b.add_html_localized_color(15, 15, 275, 20, 1080333, 0x7FFF, false, false); // Select a pet to retrieve from the stables:
        for (i, (_, name)) in self.list.iter().enumerate() {
            let i = i as i32;
            b.add_button(15, 39 + i * 20, 10006, 10006, i + 1);
            // `Color(int)` closes with `</CENTER>`.
            b.add_html(
                32,
                35 + i * 20,
                275,
                18,
                &format!("<BASEFONT COLOR=#{:06X}>{name}</CENTER>", 0xC0C0EE),
                false,
                false,
            );
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let index = info.button_id - 1;
        if index >= 0
            && let Some(&(pet, _)) = self.list.get(index as usize)
            && let Some(from) = w.net_state(sender).and_then(|n| n.mobile)
        {
            end_claim_list(w, self.trainer, from, pet);
        }
    }
}

// ---- classes ----

fn animal_trainer_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &ANIMAL_TRAINER, Some("the animal trainer"));
    crate::common::set_skill_range(w, id, SkillName::AnimalLore, 64.0, 100.0);
    crate::common::set_skill_range(w, id, SkillName::AnimalTaming, 90.0, 100.0);
    crate::common::set_skill_range(w, id, SkillName::Veterinary, 65.0, 88.0);
    id
}

fn female(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.female)
}

fn animal_trainer_outfit(w: &mut World, id: MobileId) {
    base_init_outfit(w, id);
    let name = if rnd::bool(w) {
        "Server.Items.QuarterStaff"
    } else {
        "Server.Items.ShepherdsCrook"
    };
    add_new(w, id, name);
}

static ANIMAL_TRAINER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBANIMAL_TRAINER.new_for(w, id)],
    shoe_type: |w, id| {
        if female(w, id) {
            ShoeType::ThighBoots
        } else {
            ShoeType::Boots
        }
    },
    get_shoe_hue: |_, _| 0,
    init_outfit: animal_trainer_outfit,
    ..BASE_VENDOR_CLASS
};

pub static ANIMAL_TRAINER_C: muo_content::creature::CreatureType =
    muo_content::creature::CreatureType {
        subclass: Some(&ANIMAL_TRAINER_V),
        add_custom_context_entries,
        ..BASE_VENDOR_C
    };

pub static ANIMAL_TRAINER: MobileType = MobileType {
    full_name: "Server.Mobiles.AnimalTrainer",
    parent: Some(&BASE_VENDOR),
    construct: Some(animal_trainer_new),
    family: Some(&ANIMAL_TRAINER_C),
    serialize: |w, id, out| {
        (BASE_VENDOR.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_VENDOR.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    handles_on_speech,
    on_speech,
    ..BASE_VENDOR
};

fn gypsy_new(w: &mut World) -> MobileId {
    let id = animal_trainer_new_as(w, &GYPSY_ANIMAL_TRAINER);
    let t = if rnd::bool(w) {
        "the gypsy animal trainer"
    } else {
        "the gypsy animal herder"
    };
    crate::common::title(w, id, t);
    id
}

/// `AnimalTrainer()` for a subclass.
fn animal_trainer_new_as(w: &mut World, ty: &'static MobileType) -> MobileId {
    let id = new_vendor(w, ty, Some("the animal trainer"));
    crate::common::set_skill_range(w, id, SkillName::AnimalLore, 64.0, 100.0);
    crate::common::set_skill_range(w, id, SkillName::AnimalTaming, 90.0, 100.0);
    crate::common::set_skill_range(w, id, SkillName::Veterinary, 65.0, 88.0);
    id
}

static GYPSY_ANIMAL_TRAINER_V: VendorClass = VendorClass {
    init_outfit: |w, id| {
        animal_trainer_outfit(w, id);
        for layer in [
            Layer::Pants,
            Layer::OuterLegs,
            Layer::InnerLegs,
            Layer::OuterTorso,
            Layer::InnerTorso,
            Layer::Shirt,
        ] {
            rehue_layer_bright(w, id, layer);
        }
    },
    ..ANIMAL_TRAINER_V
};

pub static GYPSY_ANIMAL_TRAINER_C: muo_content::creature::CreatureType =
    muo_content::creature::CreatureType {
        subclass: Some(&GYPSY_ANIMAL_TRAINER_V),
        ..ANIMAL_TRAINER_C
    };

pub static GYPSY_ANIMAL_TRAINER: MobileType = MobileType {
    full_name: "Server.Mobiles.GypsyAnimalTrainer",
    parent: Some(&ANIMAL_TRAINER),
    construct: Some(gypsy_new),
    family: Some(&GYPSY_ANIMAL_TRAINER_C),
    serialize: |w, id, out| {
        (ANIMAL_TRAINER.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ANIMAL_TRAINER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ANIMAL_TRAINER
};

// ---- RealEstateBroker ----

/// `m_NextCheckPack`.
struct NextCheckPack(muo_primitives::DateTime);

fn broker_new(w: &mut World) -> MobileId {
    new_vendor(w, &REAL_ESTATE_BROKER, Some("the real estate broker"))
}

fn broker_handles_on_speech(w: &World, id: MobileId, from: MobileId) -> bool {
    (mx::alive(w, from) && mx::in_range(w, id, from, 3))
        || (BASE_VENDOR.handles_on_speech)(w, id, from)
}

/// `ComputePriceFor(deed)`: 80% of what the deed cost (`AOS.Scale(price, 80)`).
pub fn compute_price_for(w: &World, deed: ItemId) -> i32 {
    const PRICES: [(&str, i32); 20] = [
        ("SmallBrickHouseDeed", 43800),
        ("StonePlasterHouseDeed", 43800),
        ("FieldStoneHouseDeed", 43800),
        ("WoodHouseDeed", 43800),
        ("WoodPlasterHouseDeed", 43800),
        ("ThatchedRoofCottageDeed", 43800),
        ("BrickHouseDeed", 144500),
        ("TwoStoryWoodPlasterHouseDeed", 192400),
        ("TwoStoryStonePlasterHouseDeed", 192400),
        ("TowerDeed", 433200),
        ("KeepDeed", 665200),
        ("CastleDeed", 1022800),
        ("LargePatioDeed", 152800),
        ("LargeMarbleDeed", 192800),
        ("SmallTowerDeed", 88500),
        ("LogCabinDeed", 97800),
        ("SandstonePatioDeed", 90900),
        ("VillaDeed", 136500),
        ("StoneWorkshopDeed", 60600),
        ("MarbleWorkshopDeed", 60300),
    ];
    let price = PRICES
        .iter()
        .find(|(n, _)| factory::item_is(w, deed, &format!("Server.Items.{n}")))
        .map_or(0, |p| p.1);
    muo_content::misc::aos_helpers::scale(price, 80) // refunds 80% of the purchase price
}

/// `pack.FindItemByType<HouseDeed>(false) != null`.
fn has_house_deed(w: &World, pack: ItemId) -> bool {
    w.item(pack)
        .is_some_and(|p| p.items.iter().any(|&i| is_house_deed(w, i)))
}

fn is_house_deed(w: &World, item: ItemId) -> bool {
    factory::item_is(w, item, "Server.Multis.Deeds.HouseDeed")
}

fn broker_on_movement(w: &mut World, id: MobileId, m: MobileId, old: Point3D) {
    let next = w
        .mobile(id)
        .and_then(|x| x.ext.get::<NextCheckPack>())
        .map_or(muo_primitives::DateTime::MIN, |n| n.0);
    if w.now > next
        && mx::in_range(w, id, m, 4)
        && !mx::in_range_point(w, id, old, 4)
        && mx::player(w, m)
        && let Some(pack) = props_b::backpack(w, m)
    {
        let t = w.now + TimeSpan::from_secs(2);
        if let Some(x) = w.mobile_mut(id) {
            x.ext.insert(NextCheckPack(t));
        }
        if has_house_deed(w, pack) {
            // If you have a deed, I can appraise it or buy it from you...
            let ns = w.mobile(m).and_then(|x| x.net_state);
            messages::private_overhead_message_number(
                w,
                id,
                MessageType::REGULAR,
                0x3B2,
                500605,
                "",
                ns,
            );
            // Simply hand me a deed to sell it.
            messages::private_overhead_message_number(
                w,
                id,
                MessageType::REGULAR,
                0x3B2,
                500606,
                "",
                ns,
            );
        }
    }
    (BASE_VENDOR.on_movement)(w, id, m, old);
}

fn broker_on_speech(w: &mut World, id: MobileId, e: &mut muo_server::events::SpeechEventArgs) {
    if !e.handled && mx::alive(w, e.mobile) && e.has_keyword(0x38) {
        // *appraise*
        messages::public_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            500608,
            "",
            false,
        ); // Which deed would you like appraised?
        let t =
            target::Target::simple(w, 12, TargetFlags::NONE, false, move |w, from, targeted| {
                appraise_on_target(w, id, from, targeted);
            });
        let tid = target::add_target(w, t);
        target::set_mobile_target(w, e.mobile, Some(tid));
        e.handled = true;
    }
    (BASE_VENDOR.on_speech)(w, id, e);
}

fn appraise_on_target(w: &mut World, id: MobileId, _from: MobileId, targeted: Targeted) {
    let say = |w: &mut World, n: i32| {
        messages::public_overhead_message_number(w, id, MessageType::REGULAR, 0x3B2, n, "", false);
    };
    if let Targeted::Entity(EntityId::Item(deed)) = targeted
        && is_house_deed(w, deed)
    {
        let price = compute_price_for(w, deed);
        if price > 0 {
            // I will pay you gold for this deed :
            messages::public_overhead_message_affix(
                w,
                id,
                MessageType::REGULAR,
                0x3B2,
                1008001,
                messages::AffixType::APPEND,
                &price.to_string(),
                "",
                false,
                muo_server::mobile::AccessLevel::Player,
            );
            say(w, 500610); // Simply hand me the deed if you wish to sell it.
        } else {
            say(w, 500607); // I'm not interested in that.
        }
    } else {
        say(w, 500609); // I can't appraise things I know nothing about...
    }
}

fn broker_on_drag_drop(w: &mut World, id: MobileId, from: MobileId, dropped: ItemId) -> bool {
    if is_house_deed(w, dropped) {
        let price = compute_price_for(w, dropped);
        if price > 0 {
            if banker::deposit(w, from, price, true) {
                // For the deed I have placed gold in your bankbox :
                messages::public_overhead_message_affix(
                    w,
                    id,
                    MessageType::REGULAR,
                    0x3B2,
                    1008000,
                    messages::AffixType::APPEND,
                    &price.to_string(),
                    "",
                    false,
                    muo_server::mobile::AccessLevel::Player,
                );
                muo_server::item::base::delete(w, dropped);
                return true;
            }
            messages::public_overhead_message_number(
                w,
                id,
                MessageType::REGULAR,
                0x3B2,
                500390,
                "",
                false,
            ); // Your bank box is full.
            return false;
        }
        messages::public_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            500607,
            "",
            false,
        ); // I'm not interested in that.
        return false;
    }
    (BASE_VENDOR.on_drag_drop)(w, id, from, dropped)
}

static REAL_ESTATE_BROKER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBREAL_ESTATE_BROKER.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};

pub static REAL_ESTATE_BROKER_C: muo_content::creature::CreatureType =
    muo_content::creature::CreatureType {
        subclass: Some(&REAL_ESTATE_BROKER_V),
        ..BASE_VENDOR_C
    };

pub static REAL_ESTATE_BROKER: MobileType = MobileType {
    full_name: "Server.Mobiles.RealEstateBroker",
    parent: Some(&BASE_VENDOR),
    construct: Some(broker_new),
    family: Some(&REAL_ESTATE_BROKER_C),
    serialize: |w, id, out| {
        (BASE_VENDOR.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_VENDOR.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    handles_on_speech: broker_handles_on_speech,
    on_movement: broker_on_movement,
    on_speech: broker_on_speech,
    on_drag_drop: broker_on_drag_drop,
    ..BASE_VENDOR
};

pub(crate) fn all() -> Vec<&'static MobileType> {
    vec![&ANIMAL_TRAINER, &GYPSY_ANIMAL_TRAINER, &REAL_ESTATE_BROKER]
}

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in all() {
        types.register_mobile(t, &[]);
    }
}
