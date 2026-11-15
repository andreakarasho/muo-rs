//! The Haochi's Trials quest's items and mobiles (namespace `Server.Engines.Quests.Samurai`).
//!
//! `HonorCandle` is a `CandleLong` whose `Burn()` and `Douse()` put out the flame and rewind the duration.

use muo_content::items::cont;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::lr_reason::LRReason;
use muo_server::mobile::{AccessLevel, MobileType};

use super::core::{Conversation, Quest, load, with_quest};
use super::haochi::*;
use super::items::{QUEST_ITEM, QuestItemClass};
use super::quester::{
    BASE_QUESTER, QUESTER_C, QUESTER_OPS, QUESTER_VC, QuesterOps, get_new_container,
};
use super::util::{add_item_hued, add_item_named, set_body};
use crate::{quest_creature, quester};

fn quest(w: &World, player: MobileId) -> Option<Quest> {
    load(w, player).filter(|q| q.is(&QUEST))
}

fn pending(q: &Quest, c: &super::core::ObjClass) -> Option<usize> {
    q.find_objective(c)
        .filter(|&i| !q.objectives[i].completed(q))
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
}

fn give(w: &mut World, to: MobileId, name: &str) -> bool {
    match muo_content::factory::make_item(w, name) {
        Some(i) => {
            muo_server::mobile::part2::items::add_to_backpack(w, to, i);
            true
        }
        None => false,
    }
}

// ---- Items ----

static KATANA_CLASS: QuestItemClass = QuestItemClass {
    can_drop: |w, _, player| !super::core::quest_is(w, player, &QUEST),
    accepted: |w, id| w.item(id).is_none_or(|i| i.deleted()),
};

/// `Server.Engines.Quests.Samurai.HaochisKatana`: `[SerializationGenerator(0)]`, an encoded version.
pub static HAOCHIS_KATANA: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Samurai.HaochisKatana",
    parent: Some(&QUEST_ITEM),
    construct: Some(|w| crate::ml_quests::item_support::template_new(w, &HAOCHIS_KATANA)),
    family: Some(&KATANA_CLASS),
    default_weight: |w, id| {
        crate::template_keys::item_float(w, id, "defaultWeight", QUEST_ITEM.default_weight)
    },
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", QUEST_ITEM.label_number)
    },
    serialize: |w, id, out| {
        (QUEST_ITEM.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (QUEST_ITEM.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..QUEST_ITEM
};

fn give_katana(w: &mut World, player: MobileId) -> bool {
    match muo_content::factory::make_item(w, "Server.Engines.Quests.Samurai.HaochisKatana") {
        Some(k) if muo_server::mobile::part2::items::place_in_backpack(w, player, k) => true,
        k => {
            if let Some(k) = k {
                muo_server::item::base::delete(w, k);
            }
            muo_content::mx::msg(w, player, 1046260);
            false
        }
    }
}

/// `Server.Engines.Quests.Samurai.HaochisKatanaGenerator`: `[SerializationGenerator(0)]`, an encoded version.
pub static HAOCHIS_KATANA_GENERATOR: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Samurai.HaochisKatanaGenerator",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| crate::ml_quests::item_support::template_new(w, &HAOCHIS_KATANA_GENERATOR)),
    default_name: |w, id| crate::template_keys::item_name(w, id, ITEM_BASE.default_name),
    on_move_over: |w, id, m| {
        if is_player(w, m)
            && let Some(qs) = quest(w, m)
        {
            if has_lost_haochis_katana(w, m) {
                give_katana(w, m);
            } else if let Some(i) = pending(&qs, &FIFTH_TRIAL_INTRO_OBJECTIVE)
                && give_katana(w, m)
            {
                with_quest(w, m, |w, q| q.complete_objective(w, i));
            }
        }
        (ITEM_BASE.on_move_over)(w, id, m)
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `HaochisTreasureChest.GenerateTreasure()`.
fn generate_treasure(w: &mut World, chest: ItemId) {
    let items: Vec<ItemId> = w.item(chest).map(|i| i.items.clone()).unwrap_or_default();
    for it in items.into_iter().rev() {
        muo_server::item::base::delete(w, it);
    }
    for _ in 0..75 {
        let item = match muo_content::hooks::rnd::below(w, 10) {
            0 => muo_content::factory::make_item(w, "Server.Items.GoldBracelet"),
            1 => muo_content::factory::make_item(w, "Server.Items.GoldRing"),
            _ => muo_content::loot::random_gem(w),
        };
        if let Some(i) = item {
            muo_server::item::container::drop_item_default(w, chest, i);
        }
    }
}

/// `Server.Engines.Quests.Samurai.HaochisTreasureChest` (a `WoodenFootLocker`, `GumpID = 0x10B`):
/// `[SerializationGenerator(0)]`, so WoodenFootLocker's int version and then an encoded one.
pub static HAOCHIS_TREASURE_CHEST: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Samurai.HaochisTreasureChest",
    parent: Some(&muo_content::items::cont::WOODEN_FOOT_LOCKER),
    construct: Some(|w| {
        let id = cont::new_lockable(w, &HAOCHIS_TREASURE_CHEST, 0);
        muo_content::templates::apply_item_as(w, id, HAOCHIS_TREASURE_CHEST.full_name);
        muo_server::item::container::set_gump_id(w, id, 0x10B);
        generate_treasure(w, id);
        id
    }),
    is_deco_container: |_, _| false,
    check_hold: |_, _, _, _, _, _, _, _| false,
    check_item_use: |_, id, _, item| item == id,
    check_lift: |w, _, from, _, reject: &mut LRReason| {
        let _ = reject;
        if w.mobile(from)
            .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
        {
            return true;
        }
        if is_player(w, from)
            && let Some(qs) = quest(w, from)
        {
            let stolen = qs
                .find_objective(&FIFTH_TRIAL_INTRO_OBJECTIVE)
                .is_some_and(|i| stolen_treasure(&qs.objectives[i]));
            if stolen {
                muo_content::mx::msg(w, from, 1063247);
            } else {
                return true;
            }
        }
        false
    },
    on_item_lifted: |w, chest, from, _| {
        if is_player(w, from)
            && let Some(qs) = quest(w, from)
            && let Some(i) = qs.find_objective(&FIFTH_TRIAL_INTRO_OBJECTIVE)
        {
            with_quest(w, from, |_, q| *q.objectives[i].state_mut::<bool>() = true);
        }
        muo_server::timer::start(w, TimeSpan::from_mins(2), TimeSpan::ZERO, 1, move |w| {
            generate_treasure(w, chest)
        });
    },
    serialize: |w, id, out| {
        (muo_content::items::cont::WOODEN_FOOT_LOCKER.serialize)(w, id, out); // ..and WoodenFootLocker's int version
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (muo_content::items::cont::WOODEN_FOOT_LOCKER.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..muo_content::items::cont::WOODEN_FOOT_LOCKER
};

// ---- Haochi, the guardsman and Relnia ----

static HAOCHI_OPS: QuesterOps = QuesterOps {
    talk_number: -1,
    auto_talk_range: |_, _, _| 2,
    on_talk: |w, _haochi, player, _| {
        let Some(qs) = quest(w, player) else { return };
        if has_lost_haochis_katana(w, player) {
            with_quest(w, player, |w, q| {
                q.add_conversation(w, Conversation::new(&LOST_SWORD_CONVERSATION))
            });
            return;
        }
        let p = |c| pending(&qs, c);
        if let Some(i) = p(&FIND_HAOCHI_OBJECTIVE) {
            with_quest(w, player, |w, q| q.complete_objective(w, i));
            return;
        }
        if let Some(i) = p(&FIRST_TRIAL_RETURN_OBJECTIVE) {
            give(w, player, "Server.Items.LeatherDo");
            with_quest(w, player, |w, q| q.complete_objective(w, i));
            return;
        }
        if let Some(i) = p(&SECOND_TRIAL_RETURN_OBJECTIVE) {
            if dragon(&qs.objectives[i]) {
                give(w, player, "Server.Items.LeatherSuneate");
            }
            with_quest(w, player, |w, q| q.complete_objective(w, i));
            return;
        }
        if let Some(i) = p(&THIRD_TRIAL_RETURN_OBJECTIVE) {
            give(w, player, "Server.Items.LeatherHaidate");
            with_quest(w, player, |w, q| q.complete_objective(w, i));
            return;
        }
        if let Some(i) = p(&FOURTH_TRIAL_RETURN_OBJECTIVE) {
            if !killed_cat(&qs.objectives[i]) {
                let cont = get_new_container(w);
                for name in ["Server.Items.LeatherHiroSode", "Server.Items.JinBaori"] {
                    if let Some(it) = muo_content::factory::make_item(w, name) {
                        muo_server::item::container::drop_item_default(w, cont, it);
                    }
                }
                muo_server::mobile::part2::items::add_to_backpack(w, player, cont);
            }
            with_quest(w, player, |w, q| q.complete_objective(w, i));
            return;
        }
        if let Some(i) = p(&FIFTH_TRIAL_RETURN_OBJECTIVE) {
            let pack = w.mobile(player).and_then(|m| m.backpack);
            let Some(katana) = pack.and_then(|b| {
                muo_content::factory::find_item_by_type(
                    w,
                    b,
                    "Server.Engines.Quests.Samurai.HaochisKatana",
                )
            }) else {
                return;
            };
            muo_server::item::base::delete(w, katana);
            let stolen = qs
                .find_objective(&FIFTH_TRIAL_INTRO_OBJECTIVE)
                .is_some_and(|k| stolen_treasure(&qs.objectives[k]));
            with_quest(w, player, |w, q| {
                q.complete_objective(w, i);
                q.add_conversation(w, sixth_trial_intro(stolen));
            });
        }
        // no return after the fifth trial: the next checks run on the same quest object
        let Some(qs) = quest(w, player) else { return };
        if let Some(i) = pending(&qs, &SIXTH_TRIAL_RETURN_OBJECTIVE) {
            with_quest(w, player, |w, q| q.complete_objective(w, i));
            return;
        }
        if let Some(i) = pending(&qs, &SEVENTH_TRIAL_RETURN_OBJECTIVE) {
            let n = 1 + muo_content::hooks::rnd::below(w, 3); // Utility.Random(1, 3)
            if let Some(weapon) = muo_content::factory::make_item(w, "Server.Items.Daisho") {
                apply_runic(w, weapon, n, 10, 30);
                muo_server::mobile::part2::items::add_to_backpack(w, player, weapon);
            }
            let n = 1 + muo_content::hooks::rnd::below(w, 3);
            if let Some(armor) = muo_content::factory::make_item(w, "Server.Items.LeatherDo") {
                apply_runic(w, armor, n, 10, 20);
                muo_server::mobile::part2::items::add_to_backpack(w, player, armor);
            }
            with_quest(w, player, |w, q| q.complete_objective(w, i));
        }
    },
    ..QUESTER_OPS
};

fn apply_runic(w: &mut World, item: ItemId, count: i32, min: i32, max: i32) {
    if let Some(f) = super::hooks::quest_hooks(w).and_then(|h| h.apply_runic_attributes) {
        f(w, item, count, min, max);
    }
}

quester! {
    /// `Server.Engines.Quests.Samurai.Haochi`.
    HAOCHI, HAOCHI_C, HAOCHI_V = "Server.Engines.Quests.Samurai.Haochi" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops HAOCHI_OPS, encoded false, title Some("the Honorable Samurai Legend"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Samurai.Haochi"),
        init_outfit: |_, _| {},
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m { }
}

static GUARDSMAN_OPS: QuesterOps = QuesterOps { ..QUESTER_OPS };

quester! {
    /// `Server.Engines.Quests.Samurai.HaochisGuardsman`.
    HAOCHIS_GUARDSMAN, HAOCHIS_GUARDSMAN_C, HAOCHIS_GUARDSMAN_V = "Server.Engines.Quests.Samurai.HaochisGuardsman"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops GUARDSMAN_OPS, encoded true, title Some("the Guardsman of Daimyo Haochi"), extra |_, _| {};
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
            muo_content::vendor::assign_random_hair_random_hue(w, id);
            add_item_named(w, id, "Server.Items.LeatherDo");
            add_item_named(w, id, "Server.Items.LeatherHiroSode");
            let h = muo_primitives::utility::random_nondyed_hue(&mut w.rng);
            add_item_hued(w, id, "Server.Items.SamuraiTabi", h);
            let legs = ["Server.Items.StuddedHaidate", "Server.Items.PlateSuneate", "Server.Items.LeatherSuneate"];
            let n = legs[muo_content::hooks::rnd::below(w, 3) as usize];
            add_item_named(w, id, n);
            let hats = [
                "Server.Items.DecorativePlateKabuto",
                "Server.Items.ChainHatsuburi",
                "Server.Items.LightPlateJingasa",
                "Server.Items.LeatherJingasa",
            ];
            let n = hats[muo_content::hooks::rnd::below(w, 4) as usize];
            add_item_named(w, id, n);
            let weapons = ["Server.Items.NoDachi", "Server.Items.Lajatang", "Server.Items.Wakizashi"];
            let n = weapons[muo_content::hooks::rnd::below(w, 3) as usize];
            if let Some(it) = add_item_named(w, id, n) {
                ops::set_movable(w, it, false);
            }
        },
    }
    c { }
    m { }
}

static RELNIA_OPS: QuesterOps = QuesterOps {
    talk_number: -1,
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Samurai.Relnia`.
    RELNIA, RELNIA_C, RELNIA_V = "Server.Engines.Quests.Samurai.Relnia" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops RELNIA_OPS, encoded true, title Some("the Gypsy"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Samurai.Relnia"),
        init_outfit: |_, _| {},
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m {
        on_drag_drop: |w, relnia, from, dropped| {
            if is_player(w, from)
                && let Some(qs) = quest(w, from)
                && let Some(i) = pending(&qs, &FOURTH_TRIAL_CATS_OBJECTIVE)
                && super::util::item_is(w, dropped, "Server.Items.Gold")
            {
                with_quest(w, from, |w, q| {
                    q.complete_objective(w, i);
                    q.add_objective(w, fourth_trial_return(false));
                });
                super::util::say_to_number(w, relnia, from, 1063241); // I thank thee.  This gold will be a great help to me and mine!
                // Intentional difference from OSI: don't take all the gold of poor newbies!
                muo_server::item::base::consume(w, dropped, 1);
                return super::util::deleted_item(w, dropped);
            }
            (BASE_QUESTER.on_drag_drop)(w, relnia, from, dropped)
        },
    }
}

// ---- The creatures ----

quest_creature! {
    /// `Server.Engines.Quests.Samurai.CursedSoul`.
    CURSED_SOUL, CURSED_SOUL_C = "Server.Engines.Quests.Samurai.CursedSoul", template;
    m { }
}

fn quest_aggression(w: &mut World, aggressor: MobileId, dragon: bool) {
    if is_player(w, aggressor)
        && let Some(qs) = quest(w, aggressor)
        && let Some(i) = pending(&qs, &SECOND_TRIAL_ATTACK_OBJECTIVE)
    {
        with_quest(w, aggressor, |w, q| {
            q.complete_objective(w, i);
            q.add_objective(w, second_trial_return(dragon));
        });
    }
}

quest_creature! {
    /// `Server.Engines.Quests.Samurai.DeadlyImp`.
    DEADLY_IMP, DEADLY_IMP_C = "Server.Engines.Quests.Samurai.DeadlyImp", template;
    m {
        aggressive_action_criminal: |w, id, aggressor, criminal| {
            (muo_content::creature::BASE_CREATURE.aggressive_action_criminal)(w, id, aggressor, criminal);
            quest_aggression(w, aggressor, false);
        },
    }
}

quest_creature! {
    /// `Server.Engines.Quests.Samurai.DiseasedCat`.
    DISEASED_CAT, DISEASED_CAT_C = "Server.Engines.Quests.Samurai.DiseasedCat", template;
    m { }
}

quest_creature! {
    /// `Server.Engines.Quests.Samurai.FierceDragon`.
    FIERCE_DRAGON, FIERCE_DRAGON_C = "Server.Engines.Quests.Samurai.FierceDragon", template;
    m {
        get_idle_sound: |_, _| 0x2C4,
        get_attack_sound: |_, _| 0x2C0,
        get_death_sound: |_, _| 0x2C1,
        get_anger_sound: |_, _| 0x2C4,
        get_hurt_sound: |_, _| 0x2C3,
        aggressive_action_criminal: |w, id, aggressor, criminal| {
            (muo_content::creature::BASE_CREATURE.aggressive_action_criminal)(w, id, aggressor, criminal);
            quest_aggression(w, aggressor, true);
        },
    }
}

quest_creature! {
    /// `Server.Engines.Quests.Samurai.InjuredWolf`.
    INJURED_WOLF, INJURED_WOLF_C = "Server.Engines.Quests.Samurai.InjuredWolf", template;
    m { get_idle_sound: |_, _| 0xE9, }
}

quest_creature! {
    /// `Server.Engines.Quests.Samurai.YoungNinja`.
    YOUNG_NINJA, YOUNG_NINJA_C = "Server.Engines.Quests.Samurai.YoungNinja", template;
    m { }
}

quest_creature! {
    /// `Server.Engines.Quests.Samurai.YoungRonin`.
    YOUNG_RONIN, YOUNG_RONIN_C = "Server.Engines.Quests.Samurai.YoungRonin", template;
    m { }
}

/// The mobile classes of this folder.
pub static MOBILES: &[&MobileType] = &[
    &CURSED_SOUL,
    &DEADLY_IMP,
    &DISEASED_CAT,
    &FIERCE_DRAGON,
    &HAOCHI,
    &HAOCHIS_GUARDSMAN,
    &INJURED_WOLF,
    &RELNIA,
    &YOUNG_NINJA,
    &YOUNG_RONIN,
];

// ---- HonorCandle ----

const LIT_DURATION: TimeSpan = TimeSpan::from_secs(20);

/// `HonorCandle.Douse()`: the base, then `Duration = LitDuration`.
fn honor_candle_douse(w: &mut World, id: ItemId) {
    muo_items::lights::douse_base(w, id);
    muo_items::lights::set_duration(w, id, LIT_DURATION);
}

static HONOR_CANDLE_CLASS: muo_items::lights::LightClass = muo_items::lights::LightClass {
    lit_sound: 0,
    unlit_sound: 0,
    // `Burn()` is `Douse()`
    burn: honor_candle_douse,
    douse: honor_candle_douse,
    ..muo_items::lights::leaves::CANDLE_LONG_CLASS
};

/// `Server.Engines.Quests.Samurai.HonorCandle` `[SerializationGenerator(0)]`.
pub static HONOR_CANDLE: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Samurai.HonorCandle",
    parent: Some(&muo_items::lights::leaves::CANDLE_LONG),
    construct: Some(|w| {
        let id = muo_items::lights::leaves::candle_long_new(w, &HONOR_CANDLE);
        ops::set_movable(w, id, false);
        muo_items::lights::set_duration(w, id, LIT_DURATION);
        id
    }),
    family: Some(&HONOR_CANDLE_CLASS),
    on_double_click: |w, id, from| {
        let was_burning = muo_items::lights::burning(w, id);
        (muo_items::lights::leaves::CANDLE_LONG.on_double_click)(w, id, from);
        if !was_burning && muo_items::lights::burning(w, id) {
            if !is_player(w, from) {
                return;
            }
            if let Some(qs) = quest(w, from) {
                // the player's quest is `HaochisTrialsQuest`
                if let Some(i) = pending(&qs, &SIXTH_TRIAL_INTRO_OBJECTIVE) {
                    with_quest(w, from, |w, q| q.complete_objective(w, i));
                }
                muo_server::item::messages::send_localized_message_to(w, id, from, 1063251, ""); // You light a candle in honor.
            }
        }
    },
    serialize: |w, id, out| {
        (muo_items::lights::leaves::CANDLE_LONG.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (muo_items::lights::leaves::CANDLE_LONG.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..muo_items::lights::leaves::CANDLE_LONG
};

/// The item classes of this folder.
pub static ITEMS: &[&ItemType] = &[
    &HAOCHIS_KATANA,
    &HAOCHIS_KATANA_GENERATOR,
    &HAOCHIS_TREASURE_CHEST,
    &HONOR_CANDLE,
];
