//! The Witch Apprentice quest's mobiles and items (namespace `Server.Engines.Quests.Hag`, and
//! `Server.Items` for `Cauldron`, `HagCauldron` and `HagStew`).

use muo_content::items::addon::{self, ADDON_COMPONENT, AddonClass, BASE_ADDON, BASE_ADDON_CLASS};
use muo_content::items::corpse::{self, CORPSE};
use muo_primitives::{Direction, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::layer::Layer;
use muo_server::mobile::MobileType;

use super::core::{Conversation, Quest, can_offer_quest_ex, load, with_quest};
use super::hooks::quest_hooks;
use super::quester::{
    BASE_QUESTER, QUESTER_C, QUESTER_OPS, QUESTER_VC, QuesterOps, get_new_container,
};
use super::util::{in_backpack, public_overhead, random_min_max, say_to_number, set_body};
use super::witch::*;
use crate::quest_item;
use crate::quester;

fn quest(w: &World, player: MobileId) -> Option<Quest> {
    load(w, player).filter(|q| q.is(&QUEST))
}

fn turn_to(w: &mut World, npc: MobileId, to: MobileId) {
    let d = muo_content::mx::get_direction_to(w, npc, to);
    muo_content::mx::set_direction(w, npc, d);
}

fn pending(q: &Quest, c: &super::core::ObjClass) -> Option<usize> {
    q.find_objective(c)
        .filter(|&i| !q.objectives[i].completed(q))
}

fn sounds(w: &mut World, npc: MobileId, a: i32, b: i32) {
    muo_content::mx::play_sound(w, npc, a);
    muo_content::mx::play_sound(w, npc, b);
}

fn gold_item(w: &mut World, cont: ItemId, amount: i32) {
    if let Some(g) = muo_content::factory::make_item(w, "Server.Items.Gold") {
        ops::set_amount(w, g, amount);
        muo_server::item::container::drop_item_default(w, cont, g);
    }
}

fn reagent(w: &mut World, cont: ItemId, name: &str, amount: i32) {
    if let Some(i) = muo_content::factory::make_item(w, name) {
        ops::set_amount(w, i, amount);
        muo_server::item::container::drop_item_default(w, cont, i);
    }
}

// ---- Blackheart ----

/// `Blackheart._nextHeave` (not saved).
#[derive(Clone, Default)]
struct Heave(i64);

static BLACKHEART_OPS: QuesterOps = QuesterOps {
    on_talk: |w, blackheart, player, _| {
        turn_to(w, blackheart, player);
        muo_content::mx::animate(w, blackheart, 33, 20, 1, true, false, 0);

        if let Some(qs) = quest(w, player)
            && let Some(i) = pending(&qs, &FIND_INGREDIENT_OBJECTIVE)
                .filter(|&i| current_ingredient(&qs.objectives[i]) == WHISKEY)
        {
            let s = if muo_content::hooks::rnd::bool(w) {
                0x42E
            } else {
                0x43F
            };
            muo_content::mx::play_sound(w, blackheart, s);
            let tricorne =
                muo_server::mobile::part2::items::find_item_on_layer(w, player, Layer::Helm)
                    .is_some_and(|h| super::util::item_is(w, h, "Server.Items.TricorneHat"));
            let bac = w.mobile(player).map_or(0, |m| m.bac);
            let met = blackheart_met(&qs.objectives[i]);
            with_quest(w, player, |w, q| {
                if tricorne && bac >= 20 {
                    q.complete_objective(w, i);
                    q.add_conversation(w, blackheart_pirate(!met));
                } else if !met {
                    q.complete_objective(w, i);
                    q.add_conversation(w, Conversation::new(&BLACKHEART_FIRST_CONVERSATION));
                } else {
                    q.add_conversation(w, blackheart_no_pirate(tricorne, bac > 0));
                }
            });
            return;
        }
        muo_content::mx::play_sound(w, blackheart, 0x42C);
        say_to_number(w, blackheart, player, 1055041); // The drunken pirate shakes his fist at you and goes back to drinking.
    },
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Hag.Blackheart`.
    BLACKHEART, BLACKHEART_C, BLACKHEART_V = "Server.Engines.Quests.Hag.Blackheart"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops BLACKHEART_OPS, encoded false, title Some("the Drunken Pirate"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Hag.Blackheart"),
        init_outfit: |_, _| {},
    }
    c {
        default_name: muo_content::templates::creature_props::default_name,
        on_think: |w, id| {
            let next = w.mobile(id).and_then(|m| m.ext.get::<Heave>()).map_or(0, |h| h.0);
            if w.tick_count.wrapping_sub(next) < 0 {
                return;
            }
            let delay = i64::from(muo_content::hooks::rnd::from(w, 60000, 120000));
            let next = w.tick_count.wrapping_add(delay);
            if let Some(m) = w.mobile_mut(id) {
                m.ext.insert(Heave(next));
            }
            public_overhead(w, id, 0x3B2, 500849, ""); // *hic*
        },
    }
    m { }
}

// ---- Grizelda ----

static GRIZELDA_OPS: QuesterOps = QuesterOps {
    on_talk: |w, grizelda, player, _| {
        turn_to(w, grizelda, player);
        let Some(qs) = quest(w, player) else {
            let mut new_quest = Quest::new(w, &QUEST, player);
            if super::core::has_quest(w, player) {
                new_quest.add_conversation(w, Conversation::new(&DONT_OFFER_CONVERSATION));
            } else {
                let (can, in_restart) = can_offer_quest_ex(w, player, &QUEST);
                if can {
                    sounds(w, grizelda, 0x20, 0x206);
                    new_quest.send_offer(w);
                } else if in_restart {
                    sounds(w, grizelda, 0x259, 0x206);
                    new_quest
                        .add_conversation(w, Conversation::new(&RECENTLY_FINISHED_CONVERSATION));
                }
            }
            return;
        };

        if qs.is_objective_in_progress(&FIND_APPRENTICE_OBJECTIVE) {
            sounds(w, grizelda, 0x259, 0x206);
            with_quest(w, player, |w, q| {
                q.add_conversation(w, Conversation::new(&HAG_DURING_CORPSE_SEARCH_CONVERSATION))
            });
        } else if let Some(i) = pending(&qs, &FIND_GRIZELDA_ABOUT_MURDER_OBJECTIVE) {
            sounds(w, grizelda, 0x420, 0x20);
            with_quest(w, player, |w, q| q.complete_objective(w, i));
        } else if qs.is_objective_in_progress(&KILL_IMPS_OBJECTIVE)
            || qs.is_objective_in_progress(&FIND_ZEEFZORPUL_OBJECTIVE)
        {
            sounds(w, grizelda, 0x259, 0x206);
            with_quest(w, player, |w, q| {
                q.add_conversation(w, Conversation::new(&HAG_DURING_IMP_SEARCH_CONVERSATION))
            });
        } else if let Some(i) = pending(&qs, &RETURN_RECIPE_OBJECTIVE) {
            sounds(w, grizelda, 0x258, 0x41B);
            with_quest(w, player, |w, q| q.complete_objective(w, i));
        } else if qs.is_objective_in_progress(&FIND_INGREDIENT_OBJECTIVE) {
            sounds(w, grizelda, 0x259, 0x206);
            with_quest(w, player, |w, q| {
                q.add_conversation(w, Conversation::new(&HAG_DURING_INGREDIENTS_CONVERSATION))
            });
        } else if let Some(i) = pending(&qs, &RETURN_INGREDIENTS_OBJECTIVE) {
            give_final_reward(w, grizelda, player, i);
        }
    },
    ..QUESTER_OPS
};

/// Grizelda's reward bag for the finished list.
fn give_final_reward(w: &mut World, grizelda: MobileId, player: MobileId, obj: usize) {
    let cont = get_new_container(w);
    for name in [
        "BlackPearl",
        "Bloodmoss",
        "Garlic",
        "Ginseng",
        "MandrakeRoot",
        "Nightshade",
        "SulfurousAsh",
        "SpidersSilk",
    ] {
        reagent(w, cont, &format!("Server.Items.{name}"), 30);
    }
    for name in [
        "Server.Items.Cauldron",
        "Server.Engines.Quests.Hag.MoonfireBrew",
    ] {
        if let Some(i) = muo_content::factory::make_item(w, name) {
            muo_server::item::container::drop_item_default(w, cont, i);
        }
    }
    let level = random_min_max(w, 1, 4);
    let map = w
        .mobile(grizelda)
        .and_then(|m| m.map)
        .unwrap_or(muo_server::map::MapId::INTERNAL);
    let m = muo_items::maps::treasure::treasure_map_new(
        w,
        &muo_items::maps::treasure::TREASURE_MAP,
        level,
        Some(map),
    );
    muo_server::item::container::drop_item_default(w, cont, m);
    let gold = random_min_max(w, 2000, 2200);
    gold_item(w, cont, gold);

    let aos = muo_server::mobile::part1::props_a::core_aos(w);
    let runic = |w: &mut World, item: ItemId| {
        if let Some(f) = quest_hooks(w).and_then(|h| h.apply_runic_attributes) {
            f(w, item, 2, 20, 30);
        }
    };
    if muo_content::hooks::rnd::bool(w) {
        if let Some(weapon) = muo_content::loot::random_weapon(w, false, false, false) {
            if aos {
                runic(w, weapon);
            } else {
                use muo_content::items::weapon::{
                    WeaponAccuracyLevel, WeaponDamageLevel, WeaponDurabilityLevel,
                    set_accuracy_level, set_damage_level, set_durability_level,
                };
                let scaled =
                    |w: &mut World| muo_content::creature::packing::random_min_max_scaled(w, 2, 3);
                let n = scaled(w);
                set_damage_level(
                    w,
                    weapon,
                    WeaponDamageLevel::from_i32(n).unwrap_or(WeaponDamageLevel::Regular),
                );
                let n = scaled(w);
                set_accuracy_level(
                    w,
                    weapon,
                    WeaponAccuracyLevel::from_i32(n).unwrap_or(WeaponAccuracyLevel::Regular),
                );
                let n = scaled(w);
                set_durability_level(
                    w,
                    weapon,
                    WeaponDurabilityLevel::from_i32(n).unwrap_or(WeaponDurabilityLevel::Regular),
                );
            }
            muo_server::item::container::drop_item_default(w, cont, weapon);
        }
    } else if aos {
        if let Some(item) = muo_content::loot::random_armor_or_shield_or_jewelry(w, false, false) {
            runic(w, item);
            muo_server::item::container::drop_item_default(w, cont, item);
        }
    } else if let Some(armor) = muo_content::loot::random_armor_or_shield(w, false, false) {
        use muo_content::items::armor::{
            ArmorDurabilityLevel, ArmorProtectionLevel, set_durability_level, set_protection_level,
        };
        let n = muo_content::creature::packing::random_min_max_scaled(w, 2, 3);
        set_protection_level(
            w,
            armor,
            ArmorProtectionLevel::from_i32(n).unwrap_or(ArmorProtectionLevel::Regular),
        );
        let n = muo_content::creature::packing::random_min_max_scaled(w, 2, 3);
        set_durability_level(
            w,
            armor,
            ArmorDurabilityLevel::from_i32(n).unwrap_or(ArmorDurabilityLevel::Regular),
        );
        muo_server::item::container::drop_item_default(w, cont, armor);
    }
    if w.mobile(player).is_some_and(|m| m.bac > 0)
        && let Some(cure) =
            muo_content::factory::make_item(w, "Server.Engines.Quests.Hag.HangoverCure")
    {
        muo_server::item::container::drop_item_default(w, cont, cure);
    }

    if muo_server::mobile::part2::items::place_in_backpack(w, player, cont) {
        // VirtueSystem.Award(player, VirtueName.Sacrifice, 250, ref gainedPath)
        if quest_hooks(w)
            .and_then(|h| h.virtue_award)
            .is_some_and(|f| f(w, player, 1, 250))
        {
            muo_content::mx::msg(w, player, 1054160); // You have gained in sacrifice.
        }
        sounds(w, grizelda, 0x253, 0x20);
        with_quest(w, player, |w, q| q.complete_objective(w, obj));
    } else {
        muo_server::item::base::delete(w, cont);
        muo_content::mx::msg(w, player, 1046260);
    }
}

quester! {
    /// `Server.Engines.Quests.Hag.Grizelda`.
    GRIZELDA, GRIZELDA_C, GRIZELDA_V = "Server.Engines.Quests.Hag.Grizelda"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops GRIZELDA_OPS, encoded false, title Some("the Hag"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Hag.Grizelda"),
        init_outfit: |_, _| {},
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m { click_title: |_, _| true, }
}

// ---- Zeefzorpul ----

static ZEEFZORPUL_OPS: QuesterOps = QuesterOps {
    can_talk_to: |_, _, _| false,
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Hag.Zeefzorpul`.
    ZEEFZORPUL, ZEEFZORPUL_C, ZEEFZORPUL_V = "Server.Engines.Quests.Hag.Zeefzorpul"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops ZEEFZORPUL_OPS, encoded false, title None, extra |_, _| {}, makeonly;
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Hag.Zeefzorpul"),
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m { skip_serialization: |_, _| true, }
}

// ---- Cauldron ----

quest_item! {
    /// `Server.Items.Cauldron`.
    CAULDRON = "Server.Items.Cauldron" : ITEM_BASE, |w| ops::new_item_with_id(w, &CAULDRON, 0x9ED);
    default_weight: |_, _| 1.0,
    default_name: |_, _| Some("a cauldron".to_owned()),
}

// ---- HagApprenticeCorpse ----

/// `HagApprenticeCorpse()`: the corpse of a throwaway human, facing south.
fn new_hag_apprentice_corpse(w: &mut World) -> ItemId {
    // GetOwner(): a Mobile with a human skin hue, male, body 0x190, deleted at once.
    let owner = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let hue = muo_content::vendor::human_random_skin_hue(w);
    muo_server::mobile::base::set_hue(w, owner, hue);
    muo_server::mobile::part1::props_b::set_female(w, owner, false);
    set_body(w, owner, 0x190);
    let id = corpse::new_corpse_of(w, &HAG_APPRENTICE_CORPSE, owner, (0, 0), (0, 0), Vec::new());
    muo_server::mobile::base::delete(w, owner);
    muo_server::item::ops::set_direction(w, id, Direction::SOUTH);
    id
}

/// `Server.Engines.Quests.Hag.HagApprenticeCorpse`.
pub static HAG_APPRENTICE_CORPSE: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Hag.HagApprenticeCorpse",
    parent: Some(&CORPSE),
    construct: Some(new_hag_apprentice_corpse),
    add_name_property: |_, _, list| list.add_text("a charred corpse"),
    on_single_click: |w, id, from| {
        let noto = corpse::corpse_notoriety(w, from, id);
        let hue = muo_server::notoriety::get_hue(w, noto);
        if let Some(ns) = muo_server::mobile::part1::props_b::net_state(w, from) {
            let (serial, graphic) = w
                .item(id)
                .map_or((muo_primitives::Serial::MINUS_ONE, 0), |i| {
                    (i.serial, i.item_id)
                });
            muo_server::packets::message::send_message(
                w,
                ns,
                serial,
                graphic,
                muo_server::packets::message::MessageType::LABEL,
                hue,
                3,
                true,
                None,
                "",
                "a charred corpse",
            );
        }
    },
    on_double_click: |w, id, from| {
        // Open(from, Core.AOS)
        let loc = ops::get_world_location(w, id);
        if !muo_content::mx::in_range_point(w, from, loc, 2) {
            return;
        }
        let is_player = w
            .mobile(from)
            .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE));
        if is_player
            && let Some(qs) = quest(w, from)
            && let Some(i) = pending(&qs, &FIND_APPRENTICE_OBJECTIVE)
        {
            if apprentice_corpse(&qs.objectives[i]) == Some(id) {
                with_quest(w, from, |w, q| q.complete_objective(w, i));
                muo_server::item::base::delete(w, id);
            } else {
                muo_server::item::messages::send_localized_message_to(w, id, from, 1055047, "");
            }
            return;
        }
        // You examine the corpse, but find nothing of interest.
        muo_server::item::messages::send_localized_message_to(w, id, from, 1055048, "");
    },
    serialize: |w, id, out| {
        (CORPSE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (CORPSE.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..CORPSE
};

// ---- HagCauldron, HagStew ----

quest_item! {
    /// `Server.Items.HagCauldron`.
    HAG_CAULDRON = "Server.Items.HagCauldron" : BASE_ADDON, |w| {
        let id = addon::new_addon(w, &HAG_CAULDRON);
        let pot = addon::new_component(w, &ADDON_COMPONENT, 2420);
        addon::add_component(w, id, pot, 0, 0, 0); // pot w/ support
        let fire = addon::new_component(w, &ADDON_COMPONENT, 4012); // fire pit
        ops::set_light(w, fire, muo_server::light_type::LightType::Circle150);
        addon::add_component(w, id, fire, 0, 0, 0);
        id
    };
}

static HAG_STEW_CLASS: AddonClass = AddonClass {
    on_component_used: |w, _addon, stew, from| {
        let loc = ops::get_world_location(w, stew);
        if !muo_content::mx::in_range_point(w, from, loc, 2) {
            muo_content::mx::send_message(w, from, "You are too far away.");
            return;
        }
        ops::set_visible(w, stew, false);
        if let Some(bread) = muo_content::factory::make_item(w, "Server.Items.BreadLoaf") {
            muo_content::items::food::eat(w, bread, from); // this decides your fillrate
        }
        muo_server::timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
            if w.item(stew).is_some_and(|i| !i.visible()) {
                ops::set_visible(w, stew, true);
            }
        });
    },
    ..BASE_ADDON_CLASS
};

quest_item! {
    /// `Server.Items.HagStew`.
    HAG_STEW = "Server.Items.HagStew" : BASE_ADDON, |w| {
        let id = addon::new_addon(w, &HAG_STEW);
        let c = addon::new_component(w, &ADDON_COMPONENT, 2416);
        if let Some(i) = w.item_mut(c) {
            i.name = Some("stew".to_owned());
        }
        ops::set_visible(w, c, true);
        addon::add_component(w, id, c, 0, 0, -7);
        id
    };
    family: Some(&HAG_STEW_CLASS),
}

// ---- HangoverCure ----

fn uses(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<i32>())
        .copied()
        .unwrap_or(0)
}

/// `Server.Engines.Quests.Hag.HangoverCure`.
pub static HANGOVER_CURE: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Hag.HangoverCure",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &HANGOVER_CURE, 0);
        muo_content::templates::apply_item_as(w, id, HANGOVER_CURE.full_name);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(20_i32);
        }
        id
    }),
    default_weight: |w, id| {
        crate::template_keys::item_float(w, id, "defaultWeight", ITEM_BASE.default_weight)
    },
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", ITEM_BASE.label_number)
    },
    on_double_click: |w, id, from| {
        if !in_backpack(w, id, from) {
            muo_server::item::messages::send_localized_message_to(w, id, from, 1042038, ""); // You must have the object in your backpack to use it.
            return;
        }
        let left = uses(w, id);
        if left > 0 {
            muo_content::mx::play_sound(w, from, 0x2D6);
            muo_content::mx::msg(w, from, 501206); // An awful taste fills your mouth.
            if w.mobile(from).is_some_and(|m| m.bac > 0) {
                if let Some(m) = w.mobile_mut(from) {
                    m.bac = 0;
                }
                muo_content::mx::msg(w, from, 501204); // You are now sober!
            }
            if let Some(i) = w.item_mut(id) {
                i.ext.insert(left - 1);
            }
        } else {
            muo_server::item::base::delete(w, id);
            muo_content::mx::msg(w, from, 501201); // There wasn't enough left to have any effect.
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_encoded_int(uses(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        let uses = r.read_encoded_int()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(uses);
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- MagicFlute ----

quest_item! {
    /// `Server.Engines.Quests.Hag.MagicFlute`.
    MAGIC_FLUTE = "Server.Engines.Quests.Hag.MagicFlute" : ITEM_BASE, |w| {
        let id = ops::new_item_with_id(w, &MAGIC_FLUTE, 0x1421);
        muo_server::item::base::set_hue(w, id, 0x8AB);
        id
    };
    label_number: |_, _| 1055051, // magic flute
    on_double_click: |w, id, from| {
        if !in_backpack(w, id, from) {
            muo_server::item::messages::send_localized_message_to(w, id, from, 1042292, ""); // You must have the object in your backpack to use it.
            return;
        }
        muo_content::mx::play_sound(w, from, 0x3D);
        let is_player = w.mobile(from).is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE));
        if !is_player {
            return;
        }
        let Some(qs) = quest(w, from) else { return };
        let Some(obj) = pending(&qs, &FIND_ZEEFZORPUL_OBJECTIVE) else { return };
        let imp = *qs.objectives[obj].state::<muo_primitives::Point3D>();
        let map = muo_content::mx::map(w, from);
        if (map != Some(muo_server::map::MapId::TRAMMEL) && map != Some(muo_server::map::MapId::FELUCCA))
            || !muo_content::mx::in_range_point(w, from, imp, 8)
        {
            muo_content::mx::msg(w, from, 1055053);
            return;
        }
        if muo_content::mx::in_range_point(w, from, imp, 4) {
            muo_server::item::base::delete(w, id);
            with_quest(w, from, |w, q| q.complete_objective(w, obj));
            return;
        }
        muo_content::mx::msg(w, from, 1055052);
    },
}

// ---- MoonfireBrew ----

quest_item! {
    /// `Server.Engines.Quests.Hag.MoonfireBrew`.
    MOONFIRE_BREW = "Server.Engines.Quests.Hag.MoonfireBrew" : ITEM_BASE, |w| ops::new_item_with_id(w, &MOONFIRE_BREW, 0xF04);
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1055065, // a bottle of magical moonfire brew
}

/// The mobile classes of this folder.
pub static MOBILES: &[&MobileType] = &[&BLACKHEART, &GRIZELDA, &ZEEFZORPUL];

/// The item classes of this folder.
pub static ITEMS: &[&ItemType] = &[
    &CAULDRON,
    &HAG_APPRENTICE_CORPSE,
    &HAG_CAULDRON,
    &HAG_STEW,
    &HANGOVER_CURE,
    &MAGIC_FLUTE,
    &MOONFIRE_BREW,
];
