//! The Collector quest's mobiles and its items (`PaintedImage`, `EnchantedPaints`, `Obsidian`)
//! (namespace `Server.Engines.Quests.Collector`).

use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, RelayInfo, send_gump_to_mobile,
};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::mobile::MobileType;
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType;
use muo_server::target::Targeted;

use super::collector::*;
use super::core::{
    Conversation, Quest, can_offer_quest, has_quest, load, with_quest, with_quest_of,
};
use super::gumps::{LIGHT_GREEN, WHITE};
use super::items::{QUEST_ITEM, QuestItemClass};
use super::quester::{BASE_QUESTER, QUESTER_C, QUESTER_OPS, QUESTER_VC, QuesterOps};
use super::util::{
    add_item_hued, add_item_named, begin_target, in_backpack, random_min_max, set_body,
};
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

fn dont(q: &mut Quest, w: &mut World, c: &'static super::core::ConvClass) {
    q.add_conversation(w, Conversation::new(c));
}

fn init_stats_body(w: &mut World, id: MobileId, hue: i32, female: bool) {
    muo_server::mobile::part2::stats::init_stats(w, id, 100, 100, 25);
    muo_server::mobile::base::set_hue(w, id, hue);
    muo_server::mobile::part1::props_b::set_female(w, id, female);
    set_body(w, id, if female { 0x191 } else { 0x190 });
}

fn hair(w: &mut World, id: MobileId, item: i32, hue: i32) {
    muo_server::mobile::part1::props_b::set_hair_item_id(w, id, item);
    muo_server::mobile::part1::props_b::set_hair_hue(w, id, hue);
}

fn facial(w: &mut World, id: MobileId, item: i32, hue: i32) {
    muo_server::mobile::part1::props_b::set_facial_hair_item_id(w, id, item);
    muo_server::mobile::part1::props_b::set_facial_hair_hue(w, id, hue);
}

// ---- Alberta Giacco ----

static ALBERTA_OPS: QuesterOps = QuesterOps {
    can_talk_to: |w, _, to| {
        quest(w, to).is_some_and(|q| {
            q.is_objective_in_progress(&FIND_ALBERTA_OBJECTIVE)
                || q.is_objective_in_progress(&SIT_ON_THE_STOOL_OBJECTIVE)
                || q.is_objective_in_progress(&RETURN_PAINTING_OBJECTIVE)
        })
    },
    on_talk: |w, alberta, player, _| {
        if quest(w, player).is_none() {
            return;
        }
        turn_to(w, alberta, player);
        with_quest(w, player, |w, qs| {
            if let Some(i) = pending(qs, &FIND_ALBERTA_OBJECTIVE) {
                qs.complete_objective(w, i);
            } else if qs.is_objective_in_progress(&SIT_ON_THE_STOOL_OBJECTIVE) {
                dont(qs, w, &ALBERTA_STOOL_CONVERSATION);
            } else if qs.is_objective_in_progress(&RETURN_PAINTING_OBJECTIVE) {
                dont(qs, w, &ALBERTA_AFTER_PAINTING_CONVERSATION);
            }
        });
    },
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Collector.AlbertaGiacco`.
    ALBERTA_GIACCO, ALBERTA_GIACCO_C, ALBERTA_GIACCO_V = "Server.Engines.Quests.Collector.AlbertaGiacco"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops ALBERTA_OPS, encoded false, title Some("the respected painter"), extra |_, _| {};
    v {
        init_body: |w, id| init_stats_body(w, id, 0x83F2, true),
        init_outfit: |w, id| {
            add_item_named(w, id, "Server.Items.FancyShirt");
            add_item_hued(w, id, "Server.Items.Skirt", 0x59B);
            add_item_named(w, id, "Server.Items.Boots");
            add_item_hued(w, id, "Server.Items.FeatheredHat", 0x59B);
            add_item_hued(w, id, "Server.Items.FullApron", 0x59B);
            hair(w, id, 0x203D, 0x457); // Pony Tail
        },
    }
    c { default_name: |_, _| Some("Alberta Giacco"), }
    m { }
}

// ---- Elwood McCarrin ----

/// `ElwoodMcCarrin.GiveReward(to)`.
pub fn give_reward(w: &mut World, to: MobileId) -> bool {
    let bag = muo_content::factory::make_item(w, "Server.Items.Bag").expect("Bag is registered");
    let gold = random_min_max(w, 500, 1000);
    if let Some(g) = muo_content::factory::make_item(w, "Server.Items.Gold") {
        ops::set_amount(w, g, gold);
        muo_server::item::container::drop_item_default(w, bag, g);
    }
    let aos = muo_server::mobile::part1::props_a::core_aos(w);
    let hooks_runic = |w: &mut World, item: ItemId| {
        if let Some(f) = super::hooks::quest_hooks(w).and_then(|h| h.apply_runic_attributes) {
            f(w, item, 2, 20, 30);
        }
    };
    if muo_content::hooks::rnd::bool(w) {
        if let Some(weapon) = muo_content::loot::random_weapon(w, false, false, false) {
            if aos {
                hooks_runic(w, weapon);
            } else {
                use muo_content::items::weapon::{
                    WeaponAccuracyLevel, WeaponDamageLevel, WeaponDurabilityLevel,
                    set_accuracy_level, set_damage_level, set_durability_level,
                };
                let n = muo_content::creature::packing::random_min_max_scaled(w, 2, 3);
                set_damage_level(
                    w,
                    weapon,
                    WeaponDamageLevel::from_i32(n).unwrap_or(WeaponDamageLevel::Regular),
                );
                let n = muo_content::creature::packing::random_min_max_scaled(w, 2, 3);
                set_accuracy_level(
                    w,
                    weapon,
                    WeaponAccuracyLevel::from_i32(n).unwrap_or(WeaponAccuracyLevel::Regular),
                );
                let n = muo_content::creature::packing::random_min_max_scaled(w, 2, 3);
                set_durability_level(
                    w,
                    weapon,
                    WeaponDurabilityLevel::from_i32(n).unwrap_or(WeaponDurabilityLevel::Regular),
                );
            }
            muo_server::item::container::drop_item_default(w, bag, weapon);
        }
    } else if aos {
        if let Some(item) = muo_content::loot::random_armor_or_shield_or_jewelry(w, false, false) {
            hooks_runic(w, item); // BaseArmor or BaseJewel: the hook decides by class
            muo_server::item::container::drop_item_default(w, bag, item);
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
        muo_server::item::container::drop_item_default(w, bag, armor);
    }
    if let Some(o) = muo_content::factory::make_item(w, "Server.Engines.Quests.Collector.Obsidian")
    {
        muo_server::item::container::drop_item_default(w, bag, o);
    }
    if muo_server::mobile::part2::items::place_in_backpack(w, to, bag) {
        return true;
    }
    muo_server::item::base::delete(w, bag);
    false
}

static ELWOOD_OPS: QuesterOps = QuesterOps {
    on_talk: |w, elwood, player, _| {
        turn_to(w, elwood, player);
        let Some(qs) = quest(w, player) else {
            let mut new_quest = Quest::new(w, &QUEST, player);
            if !has_quest(w, player) && can_offer_quest(w, player, &QUEST) {
                new_quest.send_offer(w);
            } else {
                dont(&mut new_quest, w, &DONT_OFFER_CONVERSATION);
            }
            return;
        };
        let p = |c| pending(&qs, c);
        with_quest(w, player, |w, q| {
            if qs.is_objective_in_progress(&FISH_PEARLS_OBJECTIVE) {
                dont(q, w, &ELWOOD_DURING_FISH_CONVERSATION);
            } else if let Some(i) = p(&RETURN_PEARLS_OBJECTIVE) {
                q.complete_objective(w, i);
            } else if qs.is_objective_in_progress(&FIND_ALBERTA_OBJECTIVE) {
                dont(q, w, &ELWOOD_DURING_PAINTING1_CONVERSATION);
            } else if qs.is_objective_in_progress(&SIT_ON_THE_STOOL_OBJECTIVE) {
                dont(q, w, &ELWOOD_DURING_PAINTING2_CONVERSATION);
            } else if let Some(i) = p(&RETURN_PAINTING_OBJECTIVE) {
                q.complete_objective(w, i);
            } else if qs.is_objective_in_progress(&FIND_GABRIEL_OBJECTIVE) {
                dont(q, w, &ELWOOD_DURING_AUTOGRAPH1_CONVERSATION);
            } else if qs.is_objective_in_progress(&FIND_SHEET_MUSIC_OBJECTIVE) {
                dont(q, w, &ELWOOD_DURING_AUTOGRAPH2_CONVERSATION);
            } else if qs.is_objective_in_progress(&RETURN_SHEET_MUSIC_OBJECTIVE) {
                dont(q, w, &ELWOOD_DURING_AUTOGRAPH3_CONVERSATION);
            } else if let Some(i) = p(&RETURN_AUTOGRAPH_OBJECTIVE) {
                q.complete_objective(w, i);
            } else if qs.is_objective_in_progress(&FIND_TOMAS_OBJECTIVE) {
                dont(q, w, &ELWOOD_DURING_TOYS1_CONVERSATION);
            } else if qs.is_objective_in_progress(&CAPTURE_IMAGES_OBJECTIVE) {
                dont(q, w, &ELWOOD_DURING_TOYS2_CONVERSATION);
            } else if qs.is_objective_in_progress(&RETURN_IMAGES_OBJECTIVE) {
                dont(q, w, &ELWOOD_DURING_TOYS3_CONVERSATION);
            } else if let Some(i) = p(&RETURN_TOYS_OBJECTIVE) {
                q.complete_objective(w, i);
                if give_reward(w, player) {
                    dont(q, w, &END_CONVERSATION);
                } else {
                    q.add_conversation(w, full_end_conversation(true));
                }
            } else if let Some(i) = p(&MAKE_ROOM_OBJECTIVE) {
                if give_reward(w, player) {
                    q.complete_objective(w, i);
                    dont(q, w, &END_CONVERSATION);
                } else {
                    q.add_conversation(w, full_end_conversation(false));
                }
            }
        });
    },
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Collector.ElwoodMcCarrin`.
    ELWOOD_MCCARRIN, ELWOOD_MCCARRIN_C, ELWOOD_MCCARRIN_V = "Server.Engines.Quests.Collector.ElwoodMcCarrin"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops ELWOOD_OPS, encoded false, title Some("the well-known collector"), extra |_, _| {};
    v {
        init_body: |w, id| init_stats_body(w, id, 0x83ED, false),
        init_outfit: |w, id| {
            add_item_named(w, id, "Server.Items.FancyShirt");
            add_item_hued(w, id, "Server.Items.LongPants", 0x544);
            add_item_hued(w, id, "Server.Items.Shoes", 0x454);
            add_item_hued(w, id, "Server.Items.JesterHat", 0x4D2);
            add_item_hued(w, id, "Server.Items.FullApron", 0x4D2);
            hair(w, id, 0x203D, 0x47D); // Pony Tail
            facial(w, id, 0x2040, 0x47D); // Goatee
        },
    }
    c { default_name: |_, _| Some("Elwood McCarrin"), }
    m { }
}

// ---- Gabriel Piete ----

static GABRIEL_OPS: QuesterOps = QuesterOps {
    can_talk_to: |w, _, to| {
        quest(w, to).is_some_and(|q| {
            q.is_objective_in_progress(&FIND_GABRIEL_OBJECTIVE)
                || q.is_objective_in_progress(&FIND_SHEET_MUSIC_OBJECTIVE)
                || q.is_objective_in_progress(&RETURN_SHEET_MUSIC_OBJECTIVE)
                || q.is_objective_in_progress(&RETURN_AUTOGRAPH_OBJECTIVE)
        })
    },
    on_talk: |w, gabriel, player, _| {
        if quest(w, player).is_none() {
            return;
        }
        turn_to(w, gabriel, player);
        with_quest(w, player, |w, q| {
            if let Some(i) = pending(q, &FIND_GABRIEL_OBJECTIVE) {
                q.complete_objective(w, i);
            } else if q.is_objective_in_progress(&FIND_SHEET_MUSIC_OBJECTIVE) {
                dont(q, w, &GABRIEL_NO_SHEET_MUSIC_CONVERSATION);
            } else if let Some(i) = pending(q, &RETURN_SHEET_MUSIC_OBJECTIVE) {
                q.complete_objective(w, i);
            } else if q.is_objective_in_progress(&RETURN_AUTOGRAPH_OBJECTIVE) {
                dont(q, w, &GABRIEL_IGNORE_CONVERSATION);
            }
        });
    },
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Collector.GabrielPiete`.
    GABRIEL_PIETE, GABRIEL_PIETE_C, GABRIEL_PIETE_V = "Server.Engines.Quests.Collector.GabrielPiete"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops GABRIEL_OPS, encoded false, title Some("the renowned minstrel"), extra |_, _| {};
    v {
        init_body: |w, id| init_stats_body(w, id, 0x83EF, false),
        init_outfit: |w, id| {
            add_item_named(w, id, "Server.Items.FancyShirt");
            add_item_hued(w, id, "Server.Items.LongPants", 0x5F7);
            add_item_hued(w, id, "Server.Items.Shoes", 0x5F7);
            hair(w, id, 0x2049, 0x460); // Pig Tails
            facial(w, id, 0x2041, 0x460); // Mustache
        },
    }
    c { default_name: |_, _| Some("Gabriel Piete"), }
    m { }
}

// ---- Impresario ----

static IMPRESARIO_OPS: QuesterOps = QuesterOps {
    can_talk_to: |w, _, to| {
        quest(w, to).is_some_and(|q| q.is_objective_in_progress(&FIND_SHEET_MUSIC_OBJECTIVE))
    },
    on_talk: |w, impresario, player, _| {
        let Some(qs) = quest(w, player) else { return };
        let Some(i) = pending(&qs, &FIND_SHEET_MUSIC_OBJECTIVE) else {
            return;
        };
        turn_to(w, impresario, player);
        if is_in_right_theater(w, &qs, i) {
            sheet_music_offer_display(w, player);
        } else {
            with_quest(w, player, |w, q| dont(q, w, &NO_SHEET_MUSIC_CONVERSATION));
        }
    },
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Collector.Impresario`.
    IMPRESARIO, IMPRESARIO_C, IMPRESARIO_V = "Server.Engines.Quests.Collector.Impresario"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops IMPRESARIO_OPS, encoded false, title Some("the impresario"), extra |_, _| {};
    v {
        init_body: |w, id| {
            muo_server::mobile::part2::stats::init_stats(w, id, 100, 100, 25);
            let hue = muo_content::vendor::human_random_skin_hue(w);
            muo_server::mobile::base::set_hue(w, id, hue);
            muo_server::mobile::part1::props_b::set_female(w, id, false);
            set_body(w, id, 0x190);
            let name = muo_content::misc::small::random_name(w, "male");
            if let Some(f) = w.mobile(id).map(|m| m.ty.set_name) {
                f(w, id, Some(name));
            }
        },
        init_outfit: |w, id| {
            let h = muo_primitives::utility::random_dyed_hue(&mut w.rng);
            add_item_hued(w, id, "Server.Items.FancyShirt", h);
            let h = muo_primitives::utility::random_nondyed_hue(&mut w.rng);
            add_item_hued(w, id, "Server.Items.LongPants", h);
            let h = muo_primitives::utility::random_neutral_hue(&mut w.rng);
            add_item_hued(w, id, "Server.Items.Shoes", h);
            muo_content::vendor::assign_random_hair_random_hue(w, id);
            super::util::assign_facial_random_hue(w, id);
        },
    }
    c { }
    m { }
}

/// `SheetMusicOfferGump`.
pub struct SheetMusicOfferLogic;

/// `SheetMusicOfferGump.DisplayTo(from)`.
pub fn sheet_music_offer_display(w: &mut World, to: MobileId) {
    if muo_server::mobile::part1::props_b::net_state(w, to).is_some() {
        send_gump_to_mobile(
            w,
            to,
            Box::new(DynamicGumpObj::new(75, 25, SheetMusicOfferLogic)),
            false,
        );
    }
}

impl DynamicGumpLogic for SheetMusicOfferLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Quests.Collector.SheetMusicOfferGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();
        b.add_page(0);
        b.add_image(349, 10, 0x24B0, 0);
        b.add_image_tiled(349, 130, 100, 120, 0x24B3);
        b.add_image_tiled(149, 10, 200, 140, 0x24AF);
        b.add_image_tiled(149, 300, 200, 140, 0x24B5);
        b.add_image(349, 300, 0x24B6, 0);
        b.add_image(35, 10, 0x24AE, 0);
        b.add_image_tiled(35, 150, 120, 100, 0x24B1);
        b.add_image(35, 300, 0x24B4, 0);
        b.add_html_localized_color(110, 60, 200, 20, 1049069, WHITE, false, false); // <STRONG>Conversation Event</STRONG>
        b.add_image(65, 14, 0x2776, 0);
        b.add_image_tiled(81, 14, 349, 17, 0x2775);
        b.add_image(426, 14, 0x2778, 0);
        b.add_image_tiled(50, 37, 400, 376, 0xA40);
        b.add_alpha_region(50, 37, 400, 376);
        b.add_image(0, 0, 0x28C8, 0);
        b.add_image_tiled(75, 90, 200, 1, 0x238D);
        b.add_image(75, 58, 0x2635, 0);
        b.add_image(380, 45, 0xDF, 0);
        b.add_html_localized_color(98, 140, 312, 200, 1055107, LIGHT_GREEN, false, true);
        b.add_radio(85, 350, 0x25F8, 0x25FB, true, 1);
        b.add_html_localized_color(120, 356, 280, 20, 1014088, WHITE, false, false); // I accept.
        b.add_radio(85, 385, 0x25F8, 0x25FB, false, 0);
        b.add_html_localized_color(120, 391, 280, 20, 1049012, WHITE, false, false); // No thanks, I decline.
        b.add_button(340, 390, 0xF7, 0xF8, 1);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id != 1 || !info.is_switched(1) {
            return;
        }
        let Some(player) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        with_quest_of(w, player, &QUEST, |w, q| {
            let Some(i) = pending(q, &FIND_SHEET_MUSIC_OBJECTIVE) else {
                return;
            };
            let pack = w.mobile(player).and_then(|m| m.backpack);
            let paid = pack.is_some_and(|p| {
                muo_server::item::container::consume_total(
                    w,
                    p,
                    &muo_content::items::gold::GOLD,
                    10,
                    true,
                    None,
                )
            }) || muo_content::vendor::banker::withdraw(w, player, 10);
            if paid {
                q.complete_objective(w, i);
            } else {
                muo_content::mx::msg(w, player, 1055108);
            }
        });
    }
}

// ---- Tomas O'Neerlan ----

static TOMAS_OPS: QuesterOps = QuesterOps {
    can_talk_to: |w, _, to| {
        quest(w, to).is_some_and(|q| {
            q.is_objective_in_progress(&FIND_TOMAS_OBJECTIVE)
                || q.is_objective_in_progress(&CAPTURE_IMAGES_OBJECTIVE)
                || q.is_objective_in_progress(&RETURN_IMAGES_OBJECTIVE)
        })
    },
    on_talk: |w, tomas, player, _| {
        if quest(w, player).is_none() {
            return;
        }
        turn_to(w, tomas, player);
        with_quest(w, player, |w, q| {
            if let Some(i) = pending(q, &FIND_TOMAS_OBJECTIVE) {
                let paints = muo_content::factory::make_item(
                    w,
                    "Server.Engines.Quests.Collector.EnchantedPaints",
                );
                if !paints.is_some_and(|p| {
                    muo_server::mobile::part2::items::place_in_backpack(w, player, p)
                }) {
                    if let Some(p) = paints {
                        muo_server::item::base::delete(w, p);
                    }
                    muo_content::mx::msg(w, player, 1046260);
                } else {
                    q.complete_objective(w, i);
                }
            } else if q.is_objective_in_progress(&CAPTURE_IMAGES_OBJECTIVE) {
                dont(q, w, &TOMAS_DURING_COLLECTING_CONVERSATION);
            } else if let Some(i) = pending(q, &RETURN_IMAGES_OBJECTIVE) {
                if let Some(p) = w.mobile(player).and_then(|m| m.backpack) {
                    muo_server::item::container::consume_up_to(w, p, &ENCHANTED_PAINTS, 1, true);
                }
                q.complete_objective(w, i);
            }
        });
    },
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Collector.TomasONeerlan`.
    TOMAS_ONEERLAN, TOMAS_ONEERLAN_C, TOMAS_ONEERLAN_V = "Server.Engines.Quests.Collector.TomasONeerlan"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops TOMAS_OPS, encoded false, title Some("the famed toymaker"), extra |_, _| {};
    v {
        init_body: |w, id| init_stats_body(w, id, 0x83F8, false),
        init_outfit: |w, id| {
            add_item_named(w, id, "Server.Items.FancyShirt");
            add_item_hued(w, id, "Server.Items.LongPants", 0x546);
            add_item_hued(w, id, "Server.Items.Boots", 0x452);
            add_item_hued(w, id, "Server.Items.FullApron", 0x455);
            hair(w, id, 0x203B, 0x455); // ShortHair
        },
    }
    c { default_name: |_, _| Some("Tomas O'Neerlan"), }
    m { }
}

// ---- PaintedImage ----

/// `PaintedImage._image`.
#[derive(Clone, Copy)]
pub struct PaintedImageData {
    pub image: i32,
}

fn painted_image(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<PaintedImageData>())
        .map_or(0, |d| d.image)
}

/// `new PaintedImage(image)`.
pub fn new_painted_image(w: &mut World, image: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &PAINTED_IMAGE, 0xFF3);
    muo_server::item::base::set_hue(w, id, 0x8FD);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(PaintedImageData { image });
    }
    id
}

/// `Server.Engines.Quests.Collector.PaintedImage`.
pub static PAINTED_IMAGE: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Collector.PaintedImage",
    parent: Some(&ITEM_BASE),
    construct: None,
    make: Some(|w| new_painted_image(w, 0)),
    default_weight: |_, _| 1.0,
    add_name_property: |w, id, list| {
        let info = image_info(painted_image(w, id));
        list.add_str(1060847, &format!("#1055126\t#{}", info.name())); // a painted image of:
    },
    on_single_click: |w, id, from| {
        let info = image_info(painted_image(w, id));
        ops::label_to_number(w, id, from, 1060847, &format!("#1055126\t#{}", info.name())); // a painted image of:
    },
    on_double_click: |w, id, from| {
        let loc = ops::get_world_location(w, id);
        if !muo_content::mx::in_range_point(w, from, loc, 2) {
            super::util::local_overhead_number(w, from, 0x3B2, 1019045); // I can't reach that.
            return;
        }
        if muo_server::mobile::part1::props_b::net_state(w, from).is_some() {
            let image = painted_image(w, id);
            send_gump_to_mobile(
                w,
                from,
                Box::new(DynamicGumpObj::new(75, 25, PaintedImageLogic { image })),
                false,
            );
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_enum(4, i64::from(painted_image(w, id)));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        let image = r.read_enum(4)? as i32;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(PaintedImageData { image });
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// `PaintedImageGump`.
pub struct PaintedImageLogic {
    image: i32,
}

impl DynamicGumpLogic for PaintedImageLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Quests.Collector.PaintedImageGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        let info = image_info(self.image);
        b.add_background(45, 20, 100, 100, 0xA3C);
        b.add_background(52, 29, 86, 82, 0xBB8);
        b.add_item(info.x, info.y, info.figurine, 0);
    }
}

// ---- EnchantedPaints ----

static ENCHANTED_PAINTS_CLASS: QuestItemClass = QuestItemClass {
    can_drop: |w, _, player| !super::core::quest_is(w, player, &QUEST),
    accepted: |w, id| w.item(id).is_none_or(|i| i.deleted()),
};

/// `Server.Engines.Quests.Collector.EnchantedPaints`.
pub static ENCHANTED_PAINTS: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Collector.EnchantedPaints",
    parent: Some(&QUEST_ITEM),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &ENCHANTED_PAINTS, 0xFC1);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    family: Some(&ENCHANTED_PAINTS_CLASS),
    default_weight: |_, _| 1.0,
    on_double_click: |w, id, from| {
        let is_player = w
            .mobile(from)
            .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE));
        if is_player
            && load(w, from).is_some_and(|q| {
                q.is(&QUEST) && q.is_objective_in_progress(&CAPTURE_IMAGES_OBJECTIVE)
            })
        {
            muo_server::mobile::part2::messages::send_ascii_message_hue(
                w,
                from,
                0x59,
                "Target the creature whose image you wish to create.",
            );
            begin_target(
                w,
                from,
                false,
                move |w, from, targeted| paints_target(w, id, from, targeted),
                None,
            );
            return;
        }
        muo_content::mx::msg(w, from, 1010085); // You cannot use this.
    },
    serialize: |w, id, out| {
        (QUEST_ITEM.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (QUEST_ITEM.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..QUEST_ITEM
};

/// `EnchantedPaints.InternalTarget.OnTarget`.
fn paints_target(w: &mut World, paints: ItemId, from: MobileId, targeted: Targeted) {
    if w.item(paints).is_none_or(|i| i.deleted()) || !in_backpack(w, paints, from) {
        return;
    }
    let is_player = w
        .mobile(from)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE));
    if !is_player {
        muo_content::mx::msg(w, from, 1010085); // You cannot use this.
        return;
    }
    let Some(qs) = load(w, from).filter(|q| q.is(&QUEST)) else {
        return;
    };
    let Some(obj) = pending(&qs, &CAPTURE_IMAGES_OBJECTIVE) else {
        return;
    };

    let Targeted::Entity(EntityId::Mobile(target)) = targeted else {
        muo_server::mobile::part2::messages::send_ascii_message_hue(
            w,
            from,
            0x35,
            "You have no interest in that.",
        );
        return;
    };
    let mut ty = w.mobile(target).map_or("", |m| m.ty.full_name);
    if ty == "Server.Mobiles.GreaterMongbat" {
        ty = "Server.Mobiles.Mongbat";
    }
    let ty = ty.to_owned();
    let result =
        with_quest(w, from, |w, q| capture_image(w, q, obj, &ty)).expect("the quest was just read");
    match result {
        (CaptureResponse::Valid, image) => {
            muo_content::mx::msg(w, from, 1055125);
            let p = new_painted_image(w, image);
            muo_server::mobile::part2::items::add_to_backpack(w, from, p);
        }
        (CaptureResponse::AlreadyDone, _) => {
            muo_server::mobile::part2::messages::send_ascii_message_hue(
                w,
                from,
                0x2C,
                "You have already captured the image of this creature",
            );
        }
        (CaptureResponse::Invalid, _) => muo_content::mx::msg(w, from, 1055124),
    }
}

// ---- Obsidian ----

const PARTIAL: i32 = 2;
const COMPLETED: i32 = 10;

static NAMES: [&str; 60] = [
    "",
    "an aggressive cavalier",
    "a beguiling rogue",
    "a benevolent physician",
    "a brilliant artisan",
    "a capricious adventurer",
    "a clever beggar",
    "a convincing charlatan",
    "a creative inventor",
    "a creative tinker",
    "a cunning knave",
    "a dauntless explorer",
    "a despicable ruffian",
    "an earnest malcontent",
    "an exultant animal tamer",
    "a famed adventurer",
    "a fanatical crusader",
    "a fastidious clerk",
    "a fearless hunter",
    "a festive harlequin",
    "a fidgety assassin",
    "a fierce soldier",
    "a fierce warrior",
    "a frugal magnate",
    "a glib pundit",
    "a gnomic shaman",
    "a graceful noblewoman",
    "a idiotic madman",
    "a imaginative designer",
    "an inept conjurer",
    "an innovative architect",
    "an inventive blacksmith",
    "a judicious mayor",
    "a masterful chef",
    "a masterful woodworker",
    "a melancholy clown",
    "a melodic bard",
    "a merciful guard",
    "a mirthful jester",
    "a nervous surgeon",
    "a peaceful scholar",
    "a prolific gardener",
    "a quixotic knight",
    "a regal aristocrat",
    "a resourceful smith",
    "a reticent alchemist",
    "a sanctified priest",
    "a scheming patrician",
    "a shrewd mage",
    "a singing minstrel",
    "a skilled tailor",
    "a squeamish assassin",
    "a stoic swordsman",
    "a studious scribe",
    "a thought provoking writer",
    "a treacherous scoundrel",
    "a troubled poet",
    "an unflappable wizard",
    "a valiant warrior",
    "a wayward fool",
];

/// `Obsidian`'s `_quantity` and `_statueName`.
#[derive(Clone)]
pub struct ObsidianData {
    pub quantity: i32,
    pub statue_name: String,
}

fn obsidian(w: &World, id: ItemId) -> Option<&ObsidianData> {
    w.item(id)?.ext.get::<ObsidianData>()
}

fn quantity(w: &World, id: ItemId) -> i32 {
    obsidian(w, id).map_or(1, |d| d.quantity)
}

/// `Obsidian.Quantity = value`.
pub fn set_quantity(w: &mut World, id: ItemId, value: i32) {
    let q = value.clamp(1, COMPLETED);
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<ObsidianData>()) {
        d.quantity = q;
    }
    let graphic = if q < PARTIAL {
        0x1EA7
    } else if q < COMPLETED {
        0x1F13
    } else {
        0x12CB
    };
    if let Some(f) = w.item(id).map(|i| i.ty.set_item_id) {
        f(w, id, graphic);
    }
    if let Some(f) = w.item(id).map(|i| i.ty.invalidate_properties) {
        f(w, id);
    }
}

/// `Obsidian.RandomName(from)`: a name from the table (the first slot is empty, which falls back to `from.Name`).
fn random_name(w: &mut World, from: MobileId) -> String {
    let n = NAMES[muo_content::hooks::rnd::below(w, NAMES.len() as i32) as usize];
    if n.is_empty() {
        w.mobile(from)
            .and_then(|m| (m.ty.name)(w, from))
            .unwrap_or_default()
    } else {
        n.to_owned()
    }
}

fn label(w: &mut World, id: ItemId, to: MobileId) {
    let q = quantity(w, id);
    if q < PARTIAL {
        ops::label_to_number(w, id, to, 1055137, ""); // a section of an obsidian statue
    } else if q < COMPLETED {
        ops::label_to_number(w, id, to, 1055138, ""); // a partially reconstructed obsidian statue
    } else {
        let n = obsidian(w, id)
            .map(|d| d.statue_name.clone())
            .unwrap_or_default();
        ops::label_to_number(w, id, to, 1055139, &n); // an obsidian statue of ~1_STATUE_NAME~
    }
}

fn send_regular(w: &mut World, to: MobileId, item: ItemId, hue: i32, number: i32) {
    let Some(ns) = muo_server::mobile::part1::props_b::net_state(w, to) else {
        return;
    };
    let (serial, graphic) = w
        .item(item)
        .map_or((muo_primitives::Serial::MINUS_ONE, 0), |i| {
            (i.serial, i.item_id)
        });
    muo_server::packets::message::send_message_localized(
        w,
        ns,
        serial,
        graphic,
        MessageType::REGULAR,
        hue,
        3,
        number,
        "",
        "",
    );
}

/// `Server.Engines.Quests.Collector.Obsidian`.
pub static OBSIDIAN: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Collector.Obsidian",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &OBSIDIAN, 0x1EA7);
        muo_server::item::base::set_hue(w, id, 0x497);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(ObsidianData {
                quantity: 1,
                statue_name: String::new(),
            });
        }
        id
    }),
    force_show_properties: |_, _| true, // ObjectPropertyList.Enabled
    add_name_property: |w, id, list| {
        let q = quantity(w, id);
        if q < PARTIAL {
            list.add(1055137);
        } else if q < COMPLETED {
            list.add(1055138);
        } else {
            list.add_str(
                1055139,
                &obsidian(w, id)
                    .map(|d| d.statue_name.clone())
                    .unwrap_or_default(),
            );
        }
    },
    on_single_click: |w, id, from| label(w, id, from),
    get_context_menu_entries: |w, id, from, list| {
        (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
        let q = quantity(w, id);
        if muo_content::mx::alive(w, from)
            && (PARTIAL..COMPLETED).contains(&q)
            && in_backpack(w, id, from)
        {
            // DisassembleEntry
            let mut e = ContextMenuEntry::new(6142, -1);
            e.on_click = Some(std::rc::Rc::new(
                |w: &mut World, from: MobileId, target: EntityId| {
                    let EntityId::Item(o) = target else { return };
                    let q = quantity(w, o);
                    if !muo_server::mobile::part2::stats::check_alive(w, from, true)
                        || w.item(o).is_none_or(|i| i.deleted())
                        || !(PARTIAL..COMPLETED).contains(&q)
                        || !in_backpack(w, o, from)
                    {
                        return;
                    }
                    for _ in 0..q - 1 {
                        if let Some(n) = muo_content::factory::make_item(
                            w,
                            "Server.Engines.Quests.Collector.Obsidian",
                        ) {
                            muo_server::mobile::part2::items::add_to_backpack(w, from, n);
                        }
                    }
                    set_quantity(w, o, 1);
                },
            ));
            list.push(e);
        }
    },
    on_double_click: |w, id, from| {
        if quantity(w, id) < COMPLETED {
            if !in_backpack(w, id, from) {
                send_regular(w, from, id, 0x2C, 500309);
            } else {
                begin_target(
                    w,
                    from,
                    true,
                    move |w, from, targeted| obsidian_target(w, id, from, targeted),
                    None,
                );
            }
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_encoded_int(quantity(w, id));
        out.write_string(
            obsidian(w, id)
                .map(|d| d.statue_name.as_str())
                .unwrap_or(""),
        );
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        let quantity = r.read_encoded_int()?;
        let statue_name = r.read_string()?.unwrap_or_default();
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(ObsidianData {
                quantity,
                statue_name,
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// `Obsidian.InternalTarget.OnTarget`.
fn obsidian_target(w: &mut World, this: ItemId, from: MobileId, targeted: Targeted) {
    let Targeted::Entity(EntityId::Item(targ)) = targeted else {
        return;
    };
    if w.item(this).is_none_or(|i| i.deleted()) || quantity(w, this) >= COMPLETED {
        return;
    }
    if in_backpack(w, this, from)
        && in_backpack(w, targ, from)
        && w.item(targ).is_some_and(|i| i.ty.is(&OBSIDIAN))
        && targ != this
        && quantity(w, targ) < COMPLETED
    {
        let (tq, oq) = (quantity(w, targ), quantity(w, this));
        if tq + oq <= COMPLETED {
            set_quantity(w, targ, tq + oq);
            muo_server::item::base::delete(w, this);
        } else {
            let delta = COMPLETED - tq;
            set_quantity(w, targ, tq + delta);
            set_quantity(w, this, oq - delta);
        }
        if quantity(w, targ) >= COMPLETED {
            let name = random_name(w, from);
            if let Some(d) = w
                .item_mut(targ)
                .and_then(|i| i.ext.get_mut::<ObsidianData>())
            {
                d.statue_name = name;
            }
            if let Some(f) = w.item(targ).map(|i| i.ty.invalidate_properties) {
                f(w, targ);
            }
        }
        if let Some(ns) = muo_server::mobile::part1::props_b::net_state(w, from) {
            let (serial, graphic) = w
                .item(targ)
                .map_or((muo_primitives::Serial::MINUS_ONE, 0), |i| {
                    (i.serial, i.item_id)
                });
            muo_server::packets::message::send_message(
                w,
                ns,
                serial,
                graphic,
                MessageType::REGULAR,
                0x59,
                3,
                true,
                None,
                "",
                "Something Happened.",
            );
        }
        return;
    }
    send_regular(w, from, this, 0x2C, 500309); // Nothing Happens.
}

/// The mobile classes of this folder.
pub static MOBILES: &[&MobileType] = &[
    &ALBERTA_GIACCO,
    &ELWOOD_MCCARRIN,
    &GABRIEL_PIETE,
    &IMPRESARIO,
    &TOMAS_ONEERLAN,
];

/// The item classes of this folder.
pub static ITEMS: &[&ItemType] = &[&PAINTED_IMAGE, &ENCHANTED_PAINTS, &OBSIDIAN];
