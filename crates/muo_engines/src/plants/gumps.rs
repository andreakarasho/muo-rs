//! The plant gumps: `MainPlantGump`, `ReproductionGump`, `EmptyTheBowlGump`, `SetToDecorativeGump`, and the
//! `PlantPourTarget` and `PollinateTarget` targets.
//!
//! A gump snapshots what its layout reads; `from.SendGump(this)` is a fresh display of the same gump.

use std::rc::Rc;

use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{deps, ops};
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType;
use muo_server::target::{self, Target, TargetFlags, Targeted};

use super::item::{self, health, indicator, status};
use super::{PlantHueInfo, PlantTypeInfo, plant_type as pt};
use crate::help::help_topic;
use crate::quests::util::send_display_help_topic;

fn mobile_of(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_states.get(&ns).and_then(|n| n.mobile)
}

fn has_net_state(w: &World, m: MobileId) -> bool {
    w.mobile(m).and_then(|x| x.net_state).is_some()
}

/// `from?.NetState == null || plant == null || plant.Deleted`.
fn can_display(w: &World, from: MobileId, plant: ItemId) -> bool {
    has_net_state(w, from) && w.item(plant).is_some_and(|i| !i.deleted())
}

fn label(w: &mut World, plant: ItemId, to: MobileId, number: i32) {
    ops::label_to_number(w, plant, to, number, "");
}

fn too_far(w: &mut World, from: MobileId, plant: ItemId) -> bool {
    let loc = ops::get_world_location(w, plant);
    if muo_content::mx::in_range_point(w, from, loc, 3) {
        return false;
    }
    deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3E9, 500446); // That is too far away.
    true
}

fn in_range(w: &World, from: MobileId, plant: ItemId) -> bool {
    muo_content::mx::in_range_point(w, from, ops::get_world_location(w, plant), 3)
}

fn background(b: &mut DynamicGumpBuilder) {
    b.add_background(50, 50, 200, 150, 0xE10);
}

fn corners(b: &mut DynamicGumpBuilder) {
    b.add_item(45, 45, 0xCEF, 0);
    b.add_item(45, 118, 0xCF0, 0);
    b.add_item(211, 45, 0xCEB, 0);
    b.add_item(211, 118, 0xCEC, 0);
}

// ---- MainPlantGump ----

pub struct MainPlantGump {
    plant: ItemId,
    status: i32,
    plant_type: i32,
    plant_hue: i32,
    growable: bool,
    infestation: i32,
    fungus: i32,
    poison: i32,
    disease: i32,
    water: i32,
    poison_potion: i32,
    cure_potion: i32,
    heal_potion: i32,
    strength_potion: i32,
    indicator: i32,
    health: i32,
    health_message: i32,
}

/// `MainPlantGump.DisplayTo(from, plant)`.
pub fn display_main(w: &mut World, from: MobileId, plant: ItemId) {
    if !can_display(w, from, plant) {
        return;
    }
    let Some(d) = item::data(w, plant) else {
        return;
    };
    let Some(s) = d.system.clone() else { return };
    let g = MainPlantGump {
        plant,
        status: d.status,
        plant_type: d.plant_type,
        plant_hue: d.plant_hue,
        growable: item::is_growable(w, plant),
        infestation: s.infestation,
        fungus: s.fungus,
        poison: s.poison,
        disease: s.disease,
        water: s.water,
        poison_potion: s.poison_potion,
        cure_potion: s.cure_potion,
        heal_potion: s.heal_potion,
        strength_potion: s.strength_potion,
        indicator: s.growth_indicator,
        health: s.health(d.status),
        health_message: s.localized_health(d.status),
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(20, 20, g)), true);
}

fn add_plus(b: &mut DynamicGumpBuilder, x: i32, y: i32, value: i32) {
    match value {
        1 => b.add_label(x, y, 0x35, "+"),
        2 => b.add_label(x, y, 0x21, "+"),
        _ => {}
    }
}

fn add_plus_minus(b: &mut DynamicGumpBuilder, x: i32, y: i32, value: i32) {
    match value {
        0 => b.add_label(x, y, 0x21, "-"),
        1 => b.add_label(x, y, 0x35, "-"),
        3 => b.add_label(x, y, 0x35, "+"),
        4 => b.add_label(x, y, 0x21, "+"),
        _ => {}
    }
}

fn add_level(b: &mut DynamicGumpBuilder, x: i32, y: i32, value: i32) {
    b.add_label(x, y, 0x835, &format!("{value}"));
}

impl MainPlantGump {
    fn draw_plant(&self, b: &mut DynamicGumpBuilder) {
        let st = self.status;
        if st < status::FULL_GROWN_PLANT {
            b.add_image(110, 85, 0x589, 0);
            b.add_item(122, 94, 0x914, 0);
            b.add_item(135, 94, 0x914, 0);
            b.add_item(120, 112, 0x914, 0);
            b.add_item(135, 112, 0x914, 0);
            if st >= 2 {
                b.add_item(127, 112, 0xC62, 0);
            }
            if st == 3 || st == 4 {
                b.add_item(129, 85, 0xC7E, 0);
            }
            if st >= 4 {
                b.add_item(121, 117, 0xC62, 0);
                b.add_item(133, 117, 0xC62, 0);
            }
            if st >= 5 {
                b.add_item(110, 100, 0xC62, 0);
                b.add_item(140, 100, 0xC62, 0);
                b.add_item(110, 130, 0xC62, 0);
                b.add_item(140, 130, 0xC62, 0);
            }
            if st >= 6 {
                b.add_item(105, 115, 0xC62, 0);
                b.add_item(145, 115, 0xC62, 0);
                b.add_item(125, 90, 0xC62, 0);
                b.add_item(125, 135, 0xC62, 0);
            }
        } else {
            let ty = PlantTypeInfo::get(self.plant_type);
            let hue = PlantHueInfo::get(self.plant_hue).hue;
            // The large images for these trees trigger a client crash, so use a smaller, generic tree.
            if self.plant_type == pt::CYPRESS_TWISTED || self.plant_type == pt::CYPRESS_STRAIGHT {
                b.add_item(130 + ty.offset_x, 96 + ty.offset_y, 0x0CCA, hue);
            } else {
                b.add_item(130 + ty.offset_x, 96 + ty.offset_y, ty.item_id, hue);
            }
        }
        if st != status::BOWL_OF_DIRT {
            let m = self.health_message;
            match self.health {
                health::DYING => {
                    b.add_item(92, 167, 0x1B9D, 0);
                    b.add_item(161, 167, 0x1B9D, 0);
                    b.add_html_localized_color(136, 167, 42, 20, m, 0xFC00, false, false);
                }
                health::WILTED => {
                    b.add_item(91, 164, 0x18E6, 0);
                    b.add_item(161, 164, 0x18E6, 0);
                    b.add_html_localized_color(132, 167, 42, 20, m, 0xC207, false, false);
                }
                health::HEALTHY => {
                    b.add_item(96, 168, 0xC61, 0);
                    b.add_item(162, 168, 0xC61, 0);
                    b.add_html_localized_color(129, 167, 42, 20, m, 0x8200, false, false);
                }
                _ => {
                    b.add_item(93, 162, 0x1A99, 0);
                    b.add_item(162, 162, 0x1A99, 0);
                    b.add_html_localized_color(129, 167, 42, 20, m, 0x83E0, false, false);
                }
            }
        }
    }

    fn add_growth_indicator(&self, b: &mut DynamicGumpBuilder, x: i32, y: i32) {
        if !self.growable {
            return;
        }
        match self.indicator {
            indicator::NOT_HEALTHY => b.add_label(x, y, 0x21, "-"),
            indicator::DELAY => b.add_label(x, y, 0x35, "-"),
            indicator::GROWN => b.add_label(x, y, 0x3, "+"),
            indicator::DOUBLE_GROWN => b.add_label(x, y, 0x3F, "+"),
            _ => b.add_label(x, y, 0x21, "!"),
        }
    }
}

impl DynamicGumpLogic for MainPlantGump {
    const TYPE_NAME: &'static str = "Server.Engines.Plants.MainPlantGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        background(b);
        corners(b);
        self.draw_plant(b);

        b.add_button(71, 67, 0xD4, 0xD4, 1); // Reproduction menu
        b.add_item(59, 68, 0xD08, 0);

        b.add_button(71, 91, 0xD4, 0xD4, 2); // Infestation
        b.add_item(8, 96, 0x372, 0);
        add_plus(b, 95, 92, self.infestation);

        b.add_button(71, 115, 0xD4, 0xD4, 3); // Fungus
        b.add_item(58, 115, 0xD16, 0);
        add_plus(b, 95, 116, self.fungus);

        b.add_button(71, 139, 0xD4, 0xD4, 4); // Poison
        b.add_item(59, 143, 0x1AE4, 0);
        add_plus(b, 95, 140, self.poison);

        b.add_button(71, 163, 0xD4, 0xD4, 5); // Disease
        b.add_item(55, 167, 0x1727, 0);
        add_plus(b, 95, 164, self.disease);

        b.add_button(209, 67, 0xD2, 0xD2, 6); // Water
        b.add_item(193, 67, 0x1F9D, 0);
        add_plus_minus(b, 196, 67, self.water);

        b.add_button(209, 91, 0xD4, 0xD4, 7); // Poison potion
        b.add_item(201, 91, 0xF0A, 0);
        add_level(b, 196, 91, self.poison_potion);

        b.add_button(209, 115, 0xD4, 0xD4, 8); // Cure potion
        b.add_item(201, 115, 0xF07, 0);
        add_level(b, 196, 115, self.cure_potion);

        b.add_button(209, 139, 0xD4, 0xD4, 9); // Heal potion
        b.add_item(201, 139, 0xF0C, 0);
        add_level(b, 196, 139, self.heal_potion);

        b.add_button(209, 163, 0xD4, 0xD4, 10); // Strength potion
        b.add_item(201, 163, 0xF09, 0);
        add_level(b, 196, 163, self.strength_potion);

        b.add_image(48, 47, 0xD2, 0);
        add_level(b, 54, 47, self.status);

        b.add_image(232, 47, 0xD2, 0);
        self.add_growth_indicator(b, 239, 47);

        b.add_button(48, 183, 0xD2, 0xD2, 11); // Help
        b.add_label(54, 183, 0x835, "?");

        b.add_button(232, 183, 0xD4, 0xD4, 12); // Empty the bowl
        b.add_item(219, 180, 0x15FD, 0);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let plant = self.plant;
        let button = info.button_id;
        if button == 0
            || w.item(plant).is_none_or(|i| i.deleted())
            || item::plant_status(w, plant) >= status::DECORATIVE_PLANT
        {
            return;
        }
        if ((6..=10).contains(&button) || button == 12) && too_far(w, from, plant) {
            return;
        }
        if !item::is_usable_by(w, plant, from) {
            label(w, plant, from, 1061856);
            return;
        }
        use muo_items::skill_items::potions::fx;
        match button {
            1 => {
                if item::plant_status(w, plant) > status::BOWL_OF_DIRT {
                    display_reproduction(w, from, plant);
                } else {
                    muo_content::mx::msg(w, from, 1061885); // You need to plant a seed in the bowl first.
                    display_main(w, from, plant);
                }
            }
            2 => help_and_redisplay(w, from, plant, help_topic::INFESTATION_LEVEL),
            3 => help_and_redisplay(w, from, plant, help_topic::FUNGUS_LEVEL),
            4 => help_and_redisplay(w, from, plant, help_topic::POISON_LEVEL),
            5 => help_and_redisplay(w, from, plant, help_topic::DISEASE_LEVEL),
            6 => {
                // Water
                use muo_items::food::beverage as bev;
                let pack = w.mobile(from).and_then(|m| m.backpack);
                let found = pack.and_then(|p| {
                    muo_server::item::enumerable::find_items_by_type(
                        w,
                        p,
                        true,
                        &bev::BASE_BEVERAGE,
                        |w, b| {
                            bev::is_empty(w, b) && bev::content(w, b) == bev::BeverageType::Water
                        },
                    )
                    .into_iter()
                    .next()
                });
                match found {
                    None => {
                        begin_pour_target(w, from, plant);
                    }
                    Some(b) => item::pour(w, plant, from, b),
                }
                display_main(w, from, plant);
            }
            7 => add_potion(w, from, plant, &[fx::POISON_GREATER, fx::POISON_DEADLY]),
            8 => add_potion(w, from, plant, &[fx::CURE_GREATER]),
            9 => add_potion(w, from, plant, &[fx::HEAL_GREATER]),
            10 => add_potion(w, from, plant, &[fx::STRENGTH_GREATER]),
            11 => help_and_redisplay(w, from, plant, help_topic::PLANT_GROWING),
            12 => display_empty_bowl(w, from, plant),
            _ => {}
        }
    }
}

fn help_and_redisplay(w: &mut World, from: MobileId, plant: ItemId, topic: i32) {
    send_display_help_topic(w, from, topic, true);
    display_main(w, from, plant);
}

/// `from.Target = new PlantPourTarget(plant)` and its message.
fn begin_pour_target(w: &mut World, from: MobileId, plant: ItemId) {
    let mut t = Target::new(w, 3, true, TargetFlags::NONE);
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            if w.item(plant).is_some_and(|i| !i.deleted())
                && in_range(w, from, plant)
                && let Targeted::Entity(EntityId::Item(it)) = targeted
            {
                item::pour(w, plant, from, it);
            }
        },
    ));
    t.handlers.on_target_finish = Some(Rc::new(move |w: &mut World, from: MobileId| {
        if w.item(plant).is_some_and(|i| !i.deleted())
            && item::plant_status(w, plant) < status::DECORATIVE_PLANT
            && in_range(w, from, plant)
            && item::is_usable_by(w, plant, from)
        {
            display_main(w, from, plant);
        }
    }));
    let tid = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(tid));
    // Target the container you wish to use to water the ~1_val~.
    let n = item::localized_plant_status(w, plant);
    muo_content::mx::msg_args(w, from, 1060808, &format!("#{n}"));
}

/// `GetPotion(from, effects)`.
fn get_potion(w: &World, from: MobileId, effects: &[i32]) -> Option<ItemId> {
    use muo_items::skill_items::{potion_keg, potions};
    let pack = w.mobile(from)?.backpack?;
    muo_server::item::enumerable::find_items(w, pack, true, |w, it| {
        if potions::is_potion(w, it) {
            return potions::effect(w, it).is_some_and(|e| effects.contains(&e));
        }
        w.item(it).is_some_and(|i| i.ty.is(&potion_keg::POTION_KEG))
            && potion_keg::held(w, it) > 0
            && effects.contains(&potion_keg::keg_type(w, it))
    })
    .into_iter()
    .next()
}

fn add_potion(w: &mut World, from: MobileId, plant: ItemId, effects: &[i32]) {
    match get_potion(w, from, effects) {
        Some(it) => item::pour(w, plant, from, it),
        None => {
            let (ok, message) = item::apply_potion(w, plant, effects[0], true);
            if ok {
                muo_content::mx::msg(w, from, 1061884); // You don't have any strong potions of that type in your pack.
                begin_pour_target(w, from, plant);
                return;
            }
            label(w, plant, from, message);
        }
    }
    display_main(w, from, plant);
}

// ---- EmptyTheBowlGump ----

pub struct EmptyTheBowlGump {
    plant: ItemId,
    status: i32,
}

/// `EmptyTheBowlGump.DisplayTo(from, plant)`.
pub fn display_empty_bowl(w: &mut World, from: MobileId, plant: ItemId) {
    if !can_display(w, from, plant) {
        return;
    }
    let g = EmptyTheBowlGump {
        plant,
        status: item::plant_status(w, plant),
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(20, 20, g)), true);
}

impl DynamicGumpLogic for EmptyTheBowlGump {
    const TYPE_NAME: &'static str = "Server.Engines.Plants.EmptyTheBowlGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        background(b);
        corners(b);
        b.add_label(90, 70, 0x44, "Empty the bowl?");
        b.add_item(90, 100, 0x1602, 0);
        b.add_image(140, 102, 0x15E1, 0);
        b.add_item(160, 100, 0x15FD, 0);
        if self.status != status::BOWL_OF_DIRT && self.status < status::PLANT {
            b.add_item(156, 130, 0xDCF, 0); // Seed
        }
        b.add_button(98, 150, 0x47E, 0x480, 1); // Cancel
        b.add_button(138, 151, 0xD2, 0xD2, 2); // Help
        b.add_label(143, 151, 0x835, "?");
        b.add_button(168, 150, 0x481, 0x483, 3); // Ok
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let plant = self.plant;
        let button = info.button_id;
        if button == 0
            || w.item(plant).is_none_or(|i| i.deleted())
            || item::plant_status(w, plant) >= status::DECORATIVE_PLANT
        {
            return;
        }
        if button == 3 && too_far(w, from, plant) {
            return;
        }
        if !item::is_usable_by(w, plant, from) {
            label(w, plant, from, 1061856);
            return;
        }
        match button {
            1 => display_main(w, from, plant),
            2 => {
                send_display_help_topic(w, from, help_topic::EMPTYING_BOWL, true);
                display_empty_bowl(w, from, plant);
            }
            3 => {
                use muo_server::mobile::part2::items::place_in_backpack;
                let Some(bowl) =
                    muo_content::factory::make_item(w, "Server.Engines.Plants.PlantBowl")
                else {
                    return;
                };
                if !place_in_backpack(w, from, bowl) {
                    delete(w, bowl);
                    label(w, plant, from, 1053047); // You cannot empty a bowl with a full pack!
                    display_main(w, from, plant);
                    return;
                }
                let st = item::plant_status(w, plant);
                if st != status::BOWL_OF_DIRT && st < status::PLANT {
                    let d = item::data(w, plant).cloned();
                    let seed = d.and_then(|d| {
                        super::seed::seed_new(w, d.plant_type, d.plant_hue, d.show_type)
                    });
                    let placed = seed.is_some_and(|s| place_in_backpack(w, from, s));
                    if !placed {
                        delete(w, bowl);
                        if let Some(s) = seed {
                            delete(w, s);
                        }
                        label(w, plant, from, 1053047);
                        display_main(w, from, plant);
                        return;
                    }
                }
                delete(w, plant);
            }
            _ => {}
        }
    }
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

// ---- ReproductionGump ----

pub struct ReproductionGump {
    plant: ItemId,
    status: i32,
    pollen_producing: bool,
    pollinated: bool,
    resources: Option<(i32, String)>,
    seeds: Option<(i32, String)>,
}

/// `ReproductionGump.DisplayTo(from, plant)`.
pub fn display_reproduction(w: &mut World, from: MobileId, plant: ItemId) {
    if !can_display(w, from, plant) {
        return;
    }
    let Some(d) = item::data(w, plant) else {
        return;
    };
    let Some(s) = d.system.clone() else { return };
    let total_resources = s.available_resources + s.left_resources;
    let resources =
        if item::resource_info(d.plant_type, d.plant_hue).is_none() || total_resources == 0 {
            None
        } else {
            Some((
                PlantHueInfo::get(d.plant_hue).gump_hue,
                format!("{}/{}", s.available_resources, total_resources),
            ))
        };
    let total_seeds = s.available_seeds + s.left_seeds;
    let seeds = if !item::reproduces(w, plant) || total_seeds == 0 {
        None
    } else {
        let seed_hue = item::seed_hue(w, plant);
        Some((
            PlantHueInfo::get(seed_hue).gump_hue,
            format!("{}/{}", s.available_seeds, total_seeds),
        ))
    };
    let g = ReproductionGump {
        plant,
        status: d.status,
        pollen_producing: item::pollen_producing(w, plant),
        pollinated: s.pollinated,
        resources,
        seeds,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(20, 20, g)), true);
}

impl DynamicGumpLogic for ReproductionGump {
    const TYPE_NAME: &'static str = "Server.Engines.Plants.ReproductionGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        background(b);
        b.add_image(60, 90, 0xE17, 0);
        b.add_image(120, 90, 0xE17, 0);
        b.add_image(60, 145, 0xE17, 0);
        b.add_image(120, 145, 0xE17, 0);
        corners(b);

        b.add_button(70, 67, 0xD4, 0xD4, 1); // Main menu
        b.add_item(57, 65, 0x1600, 0);
        b.add_label(108, 67, 0x835, "Reproduction");

        if self.status == status::STAGE9 {
            b.add_button(212, 67, 0xD4, 0xD4, 2); // Set to decorative
            b.add_item(202, 68, 0xC61, 0);
            b.add_label(216, 66, 0x21, "/");
        }

        b.add_button(80, 116, 0xD4, 0xD4, 3); // Pollination
        b.add_item(66, 117, 0x1AA2, 0);
        if !self.pollen_producing {
            b.add_label(106, 116, 0x35, "-");
        } else if !self.pollinated {
            b.add_label(106, 116, 0x21, "!");
        } else {
            b.add_label(106, 116, 0x3F, "+");
        }

        b.add_button(128, 116, 0xD4, 0xD4, 4); // Resources
        b.add_item(113, 120, 0x1021, 0);
        match &self.resources {
            None => b.add_label(149 + 5, 116, 0x21, "X"),
            Some((hue, text)) => b.add_label(149, 116, *hue, text),
        }

        b.add_button(177, 116, 0xD4, 0xD4, 5); // Seeds
        b.add_item(160, 121, 0xDCF, 0);
        match &self.seeds {
            None => b.add_label(199 + 5, 116, 0x21, "X"),
            Some((hue, text)) => b.add_label(199, 116, *hue, text),
        }

        b.add_button(70, 163, 0xD2, 0xD2, 6); // Gather pollen
        b.add_item(56, 164, 0x1AA2, 0);
        b.add_button(138, 163, 0xD2, 0xD2, 7); // Gather resources
        b.add_item(123, 167, 0x1021, 0);
        b.add_button(212, 163, 0xD2, 0xD2, 8); // Gather seeds
        b.add_item(195, 168, 0xDCF, 0);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let plant = self.plant;
        let button = info.button_id;
        let st = item::plant_status(w, plant);
        if button == 0
            || w.item(plant).is_none_or(|i| i.deleted())
            || st >= status::DECORATIVE_PLANT
            || st == status::BOWL_OF_DIRT
        {
            return;
        }
        if (6..=8).contains(&button) && too_far(w, from, plant) {
            return;
        }
        if !item::is_usable_by(w, plant, from) {
            label(w, plant, from, 1061856);
            return;
        }
        match button {
            1 => {
                display_main(w, from, plant);
                return;
            }
            2 => {
                if st == status::STAGE9 {
                    display_set_to_decorative(w, from, plant);
                }
                return;
            }
            3 => send_display_help_topic(w, from, help_topic::POLLINATION_STATE, true),
            4 => send_display_help_topic(w, from, help_topic::RESOURCE_PRODUCTION, true),
            5 => send_display_help_topic(w, from, help_topic::SEED_PRODUCTION, true),
            6 => {
                if !item::is_crossable(w, plant) {
                    label(w, plant, from, 1053050); // You cannot gather pollen from a mutated plant!
                } else if !item::pollen_producing(w, plant) {
                    label(w, plant, from, 1053051); // ... in this stage of development!
                } else if item::health_of(w, plant) < health::HEALTHY {
                    label(w, plant, from, 1053052); // You cannot gather pollen from an unhealthy plant!
                } else {
                    begin_pollinate_target(w, from, plant);
                    muo_content::mx::msg(w, from, 1053054); // Target the plant you wish to cross-pollinate to.
                    return;
                }
            }
            7 => {
                let res =
                    item::resource_info(item::plant_type(w, plant), item::plant_hue(w, plant));
                let available = item::sys(w, plant).map_or(0, |s| s.available_resources);
                match res {
                    None => {
                        if item::is_crossable(w, plant) {
                            label(w, plant, from, 1053056); // This plant has no resources to gather!
                        } else {
                            label(w, plant, from, 1053055); // Mutated plants do not produce resources!
                        }
                    }
                    Some(_) if available == 0 => label(w, plant, from, 1053056),
                    Some(name) => {
                        let resource = muo_content::factory::make_item(w, name);
                        if resource.is_some_and(|r| {
                            muo_server::mobile::part2::items::place_in_backpack(w, from, r)
                        }) {
                            item::set_available_resources(w, plant, available - 1);
                            label(w, plant, from, 1053059); // You gather resources from the plant.
                        } else {
                            if let Some(r) = resource {
                                delete(w, r);
                            }
                            label(w, plant, from, 1053058);
                        }
                    }
                }
            }
            8 => {
                let available = item::sys(w, plant).map_or(0, |s| s.available_seeds);
                if !item::reproduces(w, plant) {
                    label(w, plant, from, 1053060); // Mutated plants do not produce seeds!
                } else if available == 0 {
                    label(w, plant, from, 1053061); // This plant has no seeds to gather!
                } else {
                    let (t, h) = (item::seed_type(w, plant), item::seed_hue(w, plant));
                    let seed = super::seed::seed_new(w, t, h, true);
                    if seed.is_some_and(|s| {
                        muo_server::mobile::part2::items::place_in_backpack(w, from, s)
                    }) {
                        item::set_available_seeds(w, plant, available - 1);
                        label(w, plant, from, 1053063); // You gather seeds from the plant.
                    } else {
                        if let Some(s) = seed {
                            delete(w, s);
                        }
                        label(w, plant, from, 1053062);
                    }
                }
            }
            _ => {}
        }
        display_reproduction(w, from, plant);
    }
}

/// `from.Target = new PollinateTarget(plant)`.
fn begin_pollinate_target(w: &mut World, from: MobileId, plant: ItemId) {
    let mut t = Target::new(w, 3, true, TargetFlags::NONE);
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            if w.item(plant).is_none_or(|i| i.deleted())
                || item::plant_status(w, plant) >= status::DECORATIVE_PLANT
                || !in_range(w, from, plant)
            {
                return;
            }
            if !item::is_usable_by(w, plant, from) {
                label(w, plant, from, 1061856);
            } else if !item::is_crossable(w, plant) {
                label(w, plant, from, 1053050);
            } else if !item::pollen_producing(w, plant) {
                label(w, plant, from, 1053051);
            } else if item::health_of(w, plant) < health::HEALTHY {
                label(w, plant, from, 1053052);
            } else {
                let targ = match targeted {
                    Targeted::Entity(EntityId::Item(t)) if item::is_plant(w, t) => Some(t),
                    _ => None,
                };
                let ts = targ.map_or(0, |t| item::plant_status(w, t));
                let Some(targ) =
                    targ.filter(|_| ts < status::DECORATIVE_PLANT && ts > status::BOWL_OF_DIRT)
                else {
                    label(w, plant, from, 1053070); // You can only pollinate other specially grown plants!
                    return;
                };
                if !item::is_usable_by(w, targ, from) {
                    label(w, targ, from, 1061856);
                } else if !item::is_crossable(w, targ) {
                    label(w, targ, from, 1053073); // You cannot cross-pollinate with a mutated plant!
                } else if !item::pollen_producing(w, targ) {
                    label(w, targ, from, 1053074); // This plant is not in the flowering stage. You cannot pollinate it!
                } else if item::health_of(w, targ) < health::HEALTHY {
                    label(w, targ, from, 1053075); // You cannot pollinate an unhealthy plant!
                } else if item::sys(w, targ).is_some_and(|s| s.pollinated) {
                    label(w, targ, from, 1053072); // This plant has already been pollinated!
                } else if targ == plant {
                    item::pollinate(w, targ, plant);
                    label(w, targ, from, 1053071); // You pollinate the plant with its own pollen.
                } else {
                    item::pollinate(w, targ, plant);
                    label(w, targ, from, 1053076); // You successfully cross-pollinate the plant.
                }
            }
        },
    ));
    t.handlers.on_target_finish = Some(Rc::new(move |w: &mut World, from: MobileId| {
        if w.item(plant).is_some_and(|i| !i.deleted())
            && item::plant_status(w, plant) < status::DECORATIVE_PLANT
            && item::plant_status(w, plant) != status::BOWL_OF_DIRT
            && in_range(w, from, plant)
            && item::is_usable_by(w, plant, from)
        {
            display_reproduction(w, from, plant);
        }
    }));
    let tid = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(tid));
}

// ---- SetToDecorativeGump ----

pub struct SetToDecorativeGump {
    plant: ItemId,
}

/// `SetToDecorativeGump.DisplayTo(from, plant)`.
pub fn display_set_to_decorative(w: &mut World, from: MobileId, plant: ItemId) {
    if !can_display(w, from, plant) {
        return;
    }
    send_gump_to_mobile(
        w,
        from,
        Box::new(StaticGumpObj::new(20, 20, SetToDecorativeGump { plant })),
        true,
    );
}

impl StaticGumpLogic for SetToDecorativeGump {
    const TYPE_NAME: &'static str = "Server.Engines.Plants.SetToDecorativeGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        background(b);
        b.add_item(25, 45, 0xCEB, 0);
        b.add_item(25, 118, 0xCEC, 0);
        b.add_item(227, 45, 0xCEF, 0);
        b.add_item(227, 118, 0xCF0, 0);
        b.add_label(115, 85, 0x44, "Set plant");
        b.add_label(82, 105, 0x44, "to decorative mode?");
        b.add_button(98, 140, 0x47E, 0x480, 1); // Cancel
        b.add_button(138, 141, 0xD2, 0xD2, 2); // Help
        b.add_label(143, 141, 0x835, "?");
        b.add_button(168, 140, 0x481, 0x483, 3); // Ok
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let plant = self.plant;
        let button = info.button_id;
        if button == 0
            || w.item(plant).is_none_or(|i| i.deleted())
            || item::plant_status(w, plant) != status::STAGE9
        {
            return;
        }
        if button == 3 && too_far(w, from, plant) {
            return;
        }
        if !item::is_usable_by(w, plant, from) {
            label(w, plant, from, 1061856);
            return;
        }
        match button {
            1 => display_reproduction(w, from, plant),
            2 => {
                send_display_help_topic(w, from, help_topic::DECORATIVE_MODE, true);
                display_set_to_decorative(w, from, plant);
            }
            3 => {
                item::set_status(w, plant, status::DECORATIVE_PLANT);
                // You prune the plant. This plant will no longer produce resources or seeds, but will require no upkeep.
                label(w, plant, from, 1053077);
            }
            _ => {}
        }
    }
}
