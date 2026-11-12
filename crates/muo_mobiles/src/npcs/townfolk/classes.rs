//! `BrideGroom`, `EscortableMage`, `Merchant`, `Messenger`, `Noble`, `Peasant`, `Prisoner` and
//! `SeekerOfAdventure`: every one is `[SerializationGenerator(0, false)]` over `BaseEscortable`.

use muo_content::creature::BASE_CREATURE;
use muo_content::hooks::rnd;
use muo_content::mx;
use muo_primitives::{Point3D, utility};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::{AccessLevel, MobileType};
use muo_server::type_registry::TypeRegistry;

use super::{
    BASE_ESCORTABLE, BASE_ESCORTABLE_C, EscortClass, default_destinations, escort_make, hooks,
};
use crate::common::*;

/// `GetRandomHue()`: `Utility.Random(6) switch { 0 => 0, 1 => blue, 2 => green, 3 => red, 4 => yellow, 5 => neutral }`
/// (the classes without the explicit `0` arm get the same answers from `_ => 0`).
fn random_hue(w: &mut World) -> i32 {
    match rnd::below(w, 6) {
        1 => utility::random_blue_hue(&mut w.rng),
        2 => utility::random_green_hue(&mut w.rng),
        3 => utility::random_red_hue(&mut w.rng),
        4 => utility::random_yellow_hue(&mut w.rng),
        5 => utility::random_neutral_hue(&mut w.rng),
        _ => 0,
    }
}

fn female(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.female)
}

fn human_hair_hue(w: &mut World, id: MobileId) {
    let h = race_hair_hue(w, "Human");
    props_b::set_hair_hue(w, id, h);
}

/// `HairItemID = ..` then `HairHue = Race.RandomHairHue()`.
fn hair(w: &mut World, id: MobileId, item: i32) {
    props_b::set_hair_item_id(w, id, item);
    human_hair_hue(w, id);
}

// ---- BrideGroom ----

fn bride_groom_outfit(w: &mut World, id: MobileId) {
    let f = female(w, id);
    add_new(
        w,
        id,
        if f {
            "Server.Items.FancyDress"
        } else {
            "Server.Items.FancyShirt"
        },
    );
    let low = random_hue(w);
    add_new_hue(w, id, "Server.Items.LongPants", low);
    add_new_hue(
        w,
        id,
        if f {
            "Server.Items.Shoes"
        } else {
            "Server.Items.Boots"
        },
        low,
    );
    let item = if rnd::bool(w) { 0x203B } else { 0x203C };
    hair(w, id, item);
    pack_gold_range(w, id, 200, 250);
}

static BRIDE_GROOM_E: EscortClass = EscortClass {
    init_outfit: bride_groom_outfit,
    possible_destinations: default_destinations,
};

fn bride_groom_new(w: &mut World) -> MobileId {
    let id = escort_make(w, &BRIDE_GROOM);
    title(
        w,
        id,
        if female(w, id) {
            "the bride"
        } else {
            "the groom"
        },
    );
    id
}

creature_leaf! {
    /// `Server.Mobiles.BrideGroom`.
    BRIDE_GROOM, BRIDE_GROOM_C = "Server.Mobiles.BrideGroom" : BASE_ESCORTABLE / BASE_ESCORTABLE_C, ctor Some(bride_groom_new), encoded false;
    c {
        subclass: Some(&BRIDE_GROOM_E),
        can_teach: |_w, _id| true,
    }
    m {
        click_title: |_w, _id| false, // Do not display 'the groom' when single-clicking
    }
}

// ---- EscortableMage ----

fn mage_hue(w: &mut World) -> i32 {
    match rnd::below(w, 5) {
        1 => utility::random_green_hue(&mut w.rng),
        2 => utility::random_red_hue(&mut w.rng),
        3 => utility::random_yellow_hue(&mut w.rng),
        4 => utility::random_neutral_hue(&mut w.rng),
        _ => utility::random_blue_hue(&mut w.rng),
    }
}

fn mage_outfit(w: &mut World, id: MobileId) {
    let h = mage_hue(w);
    add_new_hue(w, id, "Server.Items.Robe", h);
    let low = mage_hue(w);
    add_new_hue(w, id, "Server.Items.ShortPants", low);
    let f = female(w, id);
    add_new_hue(
        w,
        id,
        if f {
            "Server.Items.ThighBoots"
        } else {
            "Server.Items.Boots"
        },
        low,
    );
    assign_random_hair(w, id);
    pack_gold_range(w, id, 200, 250);
}

static MAGE_E: EscortClass = EscortClass {
    init_outfit: mage_outfit,
    possible_destinations: default_destinations,
};

fn mage_new(w: &mut World) -> MobileId {
    let id = escort_make(w, &ESCORTABLE_MAGE);
    title(w, id, "the mage");
    set_skill_range(w, id, SkillName::EvalInt, 80.0, 100.0);
    set_skill_range(w, id, SkillName::Inscribe, 80.0, 100.0);
    set_skill_range(w, id, SkillName::Magery, 80.0, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 80.0, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 80.0, 100.0);
    id
}

creature_leaf! {
    /// `Server.Mobiles.EscortableMage`.
    ESCORTABLE_MAGE, ESCORTABLE_MAGE_C = "Server.Mobiles.EscortableMage" : BASE_ESCORTABLE / BASE_ESCORTABLE_C, ctor Some(mage_new), encoded false;
    c {
        subclass: Some(&MAGE_E),
        can_teach: |_w, _id| true,
    }
    m {
        click_title: |_w, _id| false, // Do not display 'the mage' when single-clicking
    }
}

// ---- Merchant ----

fn merchant_outfit(w: &mut World, id: MobileId) {
    let low = random_hue(w);
    add_new(w, id, "Server.Items.ThighBoots");
    let f = female(w, id);
    add_new_hue(
        w,
        id,
        if f {
            "Server.Items.FancyDress"
        } else {
            "Server.Items.FancyShirt"
        },
        low,
    );
    add_new_hue(w, id, "Server.Items.LongPants", low);
    if !f {
        add_new_hue(w, id, "Server.Items.BodySash", low);
    }
    assign_random_hair(w, id);
    pack_gold_range(w, id, 200, 250);
}

static MERCHANT_E: EscortClass = EscortClass {
    init_outfit: merchant_outfit,
    possible_destinations: default_destinations,
};

fn merchant_new(w: &mut World) -> MobileId {
    let id = escort_make(w, &MERCHANT);
    title(w, id, "the merchant");
    set_skill_range(w, id, SkillName::ItemID, 55.0, 78.0);
    set_skill_range(w, id, SkillName::ArmsLore, 55.0, 78.0);
    id
}

creature_leaf! {
    /// `Server.Mobiles.Merchant`.
    MERCHANT, MERCHANT_C = "Server.Mobiles.Merchant" : BASE_ESCORTABLE / BASE_ESCORTABLE_C, ctor Some(merchant_new), encoded false;
    c {
        subclass: Some(&MERCHANT_E),
        can_teach: |_w, _id| true,
    }
    m {
        click_title: |_w, _id| false, // Do not display 'the merchant' when single-clicking
    }
}

// ---- Messenger ----

fn messenger_outfit(w: &mut World, id: MobileId) {
    let f = female(w, id);
    if f {
        add_new(w, id, "Server.Items.PlainDress");
    } else {
        let h = random_hue(w);
        add_new_hue(w, id, "Server.Items.Shirt", h);
    }
    let low = random_hue(w);
    add_new_hue(w, id, "Server.Items.ShortPants", low);
    add_new_hue(
        w,
        id,
        if f {
            "Server.Items.Boots"
        } else {
            "Server.Items.Shoes"
        },
        low,
    );
    let r = rnd::below(w, 4);
    hair(w, id, if r == 4 { 0x203B } else { 0x2048 + r });
    pack_gold_range(w, id, 200, 250);
}

static MESSENGER_E: EscortClass = EscortClass {
    init_outfit: messenger_outfit,
    possible_destinations: default_destinations,
};

fn messenger_new(w: &mut World) -> MobileId {
    let id = escort_make(w, &MESSENGER);
    title(w, id, "the messenger");
    id
}

creature_leaf! {
    /// `Server.Mobiles.Messenger`.
    MESSENGER, MESSENGER_C = "Server.Mobiles.Messenger" : BASE_ESCORTABLE / BASE_ESCORTABLE_C, ctor Some(messenger_new), encoded false;
    c {
        subclass: Some(&MESSENGER_E),
        can_teach: |_w, _id| true,
    }
    m {
        click_title: |_w, _id| false, // Do not display 'the messenger' when single-clicking
    }
}

// ---- Noble ----

fn noble_outfit(w: &mut World, id: MobileId) {
    let f = female(w, id);
    if f {
        add_new(w, id, "Server.Items.FancyDress");
    } else {
        let h = random_hue(w);
        add_new_hue(w, id, "Server.Items.FancyShirt", h);
    }
    let low = random_hue(w);
    add_new_hue(w, id, "Server.Items.ShortPants", low);
    add_new_hue(
        w,
        id,
        if f {
            "Server.Items.ThighBoots"
        } else {
            "Server.Items.Boots"
        },
        low,
    );
    if !f {
        add_new_hue(w, id, "Server.Items.BodySash", low);
    }
    let h = random_hue(w);
    add_new_hue(w, id, "Server.Items.Cloak", h);
    if !f {
        add_new(w, id, "Server.Items.Longsword");
    }
    assign_random_hair(w, id);
    pack_gold_range(w, id, 200, 250);
}

static NOBLE_E: EscortClass = EscortClass {
    init_outfit: noble_outfit,
    possible_destinations: default_destinations,
};

fn noble_new(w: &mut World) -> MobileId {
    let id = escort_make(w, &NOBLE);
    title(w, id, "the noble");
    set_skill_range(w, id, SkillName::Parry, 80.0, 100.0);
    set_skill_range(w, id, SkillName::Swords, 80.0, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 80.0, 100.0);
    id
}

creature_leaf! {
    /// `Server.Mobiles.Noble`.
    NOBLE, NOBLE_C = "Server.Mobiles.Noble" : BASE_ESCORTABLE / BASE_ESCORTABLE_C, ctor Some(noble_new), encoded false;
    c {
        subclass: Some(&NOBLE_E),
        can_teach: |_w, _id| true,
    }
    m {
        click_title: |_w, _id| false, // Do not display 'the noble' when single-clicking
    }
}

// ---- Peasant ----

fn peasant_outfit(w: &mut World, id: MobileId) {
    let f = female(w, id);
    if f {
        add_new(w, id, "Server.Items.PlainDress");
    } else {
        let h = random_hue(w);
        add_new_hue(w, id, "Server.Items.Shirt", h);
    }
    let low = random_hue(w);
    add_new_hue(w, id, "Server.Items.ShortPants", low);
    add_new_hue(
        w,
        id,
        if f {
            "Server.Items.Boots"
        } else {
            "Server.Items.Shoes"
        },
        low,
    );
    assign_random_hair(w, id);
    pack_gold_range(w, id, 200, 250);
}

static PEASANT_E: EscortClass = EscortClass {
    init_outfit: peasant_outfit,
    possible_destinations: default_destinations,
};

fn peasant_new(w: &mut World) -> MobileId {
    let id = escort_make(w, &PEASANT);
    title(w, id, "the peasant");
    id
}

creature_leaf! {
    /// `Server.Mobiles.Peasant`.
    PEASANT, PEASANT_C = "Server.Mobiles.Peasant" : BASE_ESCORTABLE / BASE_ESCORTABLE_C, ctor Some(peasant_new), encoded false;
    c {
        subclass: Some(&PEASANT_E),
        can_teach: |_w, _id| true,
    }
    m {
        click_title: |_w, _id| false, // Do not display 'the peasant' when single-clicking
    }
}

// ---- Prisoner (Server.Mobiles.Townfolk.Prisoner) ----

fn nondyed(w: &mut World) -> i32 {
    utility::random_nondyed_hue(&mut w.rng)
}

fn prisoner_outfit(w: &mut World, id: MobileId) {
    if female(w, id) {
        let h = nondyed(w);
        add_new_hue(w, id, "Server.Items.FancyDress", h);
    } else {
        let h = nondyed(w);
        add_new_hue(w, id, "Server.Items.FancyShirt", h);
        let h = nondyed(w);
        add_new_hue(w, id, "Server.Items.LongPants", h);
    }
    let boots = if rnd::bool(w) {
        "Server.Items.Boots"
    } else {
        "Server.Items.ThighBoots"
    };
    add_new(w, id, boots);
    assign_random_hair(w, id);
    let hair_hue = w.mobile(id).map_or(0, |m| m.hair_hue);
    muo_content::vendor::assign_random_facial_hair(w, id, hair_hue);
}

static PRISONER_E: EscortClass = EscortClass {
    init_outfit: prisoner_outfit,
    possible_destinations: default_destinations,
};

fn prisoner_new(w: &mut World) -> MobileId {
    let id = escort_make(w, &PRISONER);
    title(
        w,
        id,
        if female(w, id) {
            "the noblewoman"
        } else {
            "the nobleman"
        },
    );
    cant_walk(w, id, true);
    if let Some(d) = muo_content::creature::data::cd_mut(w, id) {
        d.is_prisoner = true;
    }
    id
}

fn prisoner_shout(w: &mut World, id: MobileId, pm: MobileId) {
    // 502261 - HELP! ... 502267 - Go and get some help!
    let n = rnd::from(w, 502261, 7);
    if let Some(f) = hooks(w).and_then(|h| h.tell) {
        f(w, id, pm, n);
    }
}

fn prisoner_on_movement(w: &mut World, id: MobileId, m: MobileId, old: Point3D) {
    (BASE_ESCORTABLE.on_movement)(w, id, m, old);
    let (hidden, level) = (
        w.mobile(m).is_some_and(|x| x.hidden),
        mx::access_level(w, m),
    );
    if w.mobile(id).is_some_and(|x| x.cant_walk)
        && mx::in_range(w, id, m, 1)
        && !mx::in_range_point(w, id, old, 1)
        && (!hidden || level == AccessLevel::Player)
    {
        muo_server::mobile::part2::messages::say_number(w, id, 502268, ""); // Quickly, I beg thee! Unlock my chains! ..
    }
}

creature_leaf! {
    /// `Server.Mobiles.Townfolk.Prisoner`.
    PRISONER, PRISONER_C = "Server.Mobiles.Townfolk.Prisoner" : BASE_ESCORTABLE / BASE_ESCORTABLE_C, ctor Some(prisoner_new), encoded false;
    c {
        subclass: Some(&PRISONER_E),
        can_teach: |_w, _id| true,
        shout: prisoner_shout,
    }
    m {
        click_title: |_w, _id| false,
        on_movement: prisoner_on_movement,
    }
}

// ---- SeekerOfAdventure ----

const DUNGEONS: [&str; 7] = [
    "Covetous", "Deceit", "Despise", "Destard", "Hythloth",
    "Shame", // Old Code for Pre-ML shards.
    "Wrong",
];
const ML_DESTINATIONS: [&str; 4] = ["Cove", "Serpent's Hold", "Jhelom", "Nujel'm"]; // ML List

fn seeker_destinations(w: &World) -> Vec<&'static str> {
    if w.core_ml() {
        ML_DESTINATIONS.to_vec()
    } else {
        DUNGEONS.to_vec()
    }
}

fn seeker_outfit(w: &mut World, id: MobileId) {
    let f = female(w, id);
    let h = random_hue(w);
    add_new_hue(
        w,
        id,
        if f {
            "Server.Items.FancyDress"
        } else {
            "Server.Items.FancyShirt"
        },
        h,
    );
    let low = random_hue(w);
    add_new_hue(w, id, "Server.Items.ShortPants", low);
    add_new_hue(
        w,
        id,
        if f {
            "Server.Items.ThighBoots"
        } else {
            "Server.Items.Boots"
        },
        low,
    );
    if !f {
        add_new_hue(w, id, "Server.Items.BodySash", low);
    }
    let h = random_hue(w);
    add_new_hue(w, id, "Server.Items.Cloak", h);
    add_new(w, id, "Server.Items.Longsword");
    assign_random_hair(w, id);
    pack_gold_range(w, id, 100, 150);
}

static SEEKER_E: EscortClass = EscortClass {
    init_outfit: seeker_outfit,
    possible_destinations: seeker_destinations,
};

fn seeker_new(w: &mut World) -> MobileId {
    let id = escort_make(w, &SEEKER_OF_ADVENTURE);
    title(w, id, "the seeker of adventure");
    id
}

creature_leaf! {
    /// `Server.Mobiles.SeekerOfAdventure`.
    SEEKER_OF_ADVENTURE, SEEKER_OF_ADVENTURE_C = "Server.Mobiles.SeekerOfAdventure" : BASE_ESCORTABLE / BASE_ESCORTABLE_C, ctor Some(seeker_new), encoded false;
    c {
        subclass: Some(&SEEKER_E),
    }
    m {
        click_title: |_w, _id| false, // Do not display 'the seeker of adventure' when single-clicking
    }
}

pub(crate) fn all() -> Vec<&'static MobileType> {
    vec![
        &super::BASE_ESCORTABLE,
        &BRIDE_GROOM,
        &ESCORTABLE_MAGE,
        &MERCHANT,
        &MESSENGER,
        &NOBLE,
        &PEASANT,
        &PRISONER,
        &SEEKER_OF_ADVENTURE,
    ]
}

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in all().into_iter().skip(1) {
        types.register_mobile(t, &[]);
    }
}

#[allow(dead_code)]
fn _unused(w: &World, id: MobileId) -> bool {
    let _ = &BASE_CREATURE;
    mx::alive(w, id)
}
