//! `AquariumFishingNet`, `AquariumMessage` and `CaptainBlackheartsFishingPole`: the three aquarium rewards that derive from fishing items.

use muo_primitives::{Point3D, utility};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{base, deps, ops};
use muo_server::map::MapId;
use muo_server::skills::SkillName;
use muo_server::type_registry::TypeRegistry;

use crate::simple::leaf;
use crate::skill_items::fishing::{
    FISHING_POLE, MESSAGE_IN_A_BOTTLE, NetClass, SPECIAL_FISHING_NET, new_bottle_of, new_net_of,
    new_pole_of,
};

/// `AquariumFishNet.GiveFish(from)`.
fn give_fish(w: &mut World, from: MobileId) -> Option<ItemId> {
    const FISH: [&str; 20] = [
        "MinocBlueFish",
        "Shrimp",
        "FandancerFish",
        "GoldenBroadtail",
        "RedDartFish",
        "AlbinoCourtesanFish",
        "MakotoCourtesanFish",
        "NujelmHoneyFish",
        "Jellyfish",
        "SpeckledCrab",
        "LongClawCrab",
        "AlbinoFrog",
        "KillerFrog",
        "VesperReefTiger",
        "PurpleFrog",
        "BritainCrownFish",
        "YellowFinBluebelly",
        "SpottedBuccaneer",
        "SpinedScratcherFish",
        "SmallMouthSuckerFin",
    ];
    let skill = muo_server::skills::value_ref(w, from, SkillName::Fishing);
    let name = if skill / 100.0 >= utility::random_double(&mut w.rng) {
        let max = ((skill as i32) / 5).min(20);
        FISH[utility::random(&mut w.rng, max) as usize]
    } else {
        FISH[0]
    };
    muo_content::factory::make_item(w, &format!("Server.Items.{name}"))
}

/// `AquariumFishNet.FinishEffect(p, map, from)`.
fn finish_effect(w: &mut World, id: ItemId, _p: Point3D, _map: Option<MapId>, from: MobileId) {
    use crate::special::{aquarium, fish};
    if muo_server::skills::value_ref(w, from, SkillName::Fishing) < 10.0 {
        deps::send_localized_message(w, from, 1074487); // The creatures are too quick for you!
    } else {
        if let Some(f) = give_fish(w, from) {
            if let Some(bowl) = aquarium::get_empty_bowl(w, from) {
                fish::stop_timer(w, f);
                if let Some(ty) = w.item(bowl).map(|i| i.ty) {
                    (ty.add_item)(w, bowl, f);
                }
                deps::send_localized_message(w, from, 1074489); // A live creature jumps into the fish bowl in your pack!
                base::delete(w, id);
                return;
            }
            if muo_server::mobile::part2::items::place_in_backpack(w, from, f) {
                muo_content::mx::play_sound(w, from, 0x5A2);
                // A live creature flops around in your pack before running out of air.
                deps::send_localized_message(w, from, 1074490);
                fish::kill(w, f);
                base::delete(w, id);
                return;
            }
            base::delete(w, f);
        }
        deps::send_localized_message(w, from, 1074488); // You could not hold the creature.
    }
    crate::skill_items::fishing::set_in_use_pub(w, id, false);
    ops::set_movable(w, id, true);
    if !muo_server::mobile::part2::items::place_in_backpack(w, from, id) {
        let (map, loc) = w
            .mobile(from)
            .map_or((None, Point3D::default()), |m| (m.map, m.location));
        if map.is_none_or(|m| m == MapId::INTERNAL) {
            base::delete(w, id);
        } else {
            ops::move_to_world_map(w, id, loc, map);
        }
    }
}

static AQUARIUM_NET_CLASS: NetClass = NetClass {
    require_deep_water: false,
    finish_effect,
};

leaf! {
    /// `Server.Items.AquariumFishNet`.
    AQUARIUM_FISH_NET = "Server.Items.AquariumFishNet" : SPECIAL_FISHING_NET, v0;
    construct: Some(|w| {
        let id = new_net_of(w, &AQUARIUM_FISH_NET);
        crate::simple::item_id(w, id, 0xDC8);
        id
    }),
    family: Some(&AQUARIUM_NET_CLASS),
    label_number: |_, _| 1074463, // An aquarium fishing net
    get_properties: |w, id, list| base::get_properties_base(w, id, list),
}

leaf! {
    /// `Server.Items.AquariumMessage`.
    AQUARIUM_MESSAGE = "Server.Items.AquariumMessage" : MESSAGE_IN_A_BOTTLE, v0;
    construct: Some(|w| new_bottle_of(w, &AQUARIUM_MESSAGE, None, None)),
    label_number: |_, _| 1073894, // Message in a Bottle
    add_name_properties: |w, id, list| {
        (MESSAGE_IN_A_BOTTLE.add_name_properties)(w, id, list);
        list.add(1073634); // An aquarium decoration
    },
}

leaf! {
    /// `Server.Items.CaptainBlackheartsFishingPole`.
    CAPTAIN_BLACKHEARTS_FISHING_POLE = "Server.Items.CaptainBlackheartsFishingPole" : FISHING_POLE, v0;
    construct: Some(|w| new_pole_of(w, &CAPTAIN_BLACKHEARTS_FISHING_POLE)),
    label_number: |_, _| 1074571, // Captain Blackheart's Fishing Pole
    add_name_properties: |w, id, list| {
        (FISHING_POLE.add_name_properties)(w, id, list);
        list.add(1073634); // An aquarium decoration
    },
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &AQUARIUM_FISH_NET,
        &AQUARIUM_MESSAGE,
        &CAPTAIN_BLACKHEARTS_FISHING_POLE,
    ] {
        types.register_item(t, &[]);
    }
}
