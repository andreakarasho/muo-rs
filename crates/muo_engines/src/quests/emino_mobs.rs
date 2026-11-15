//! The Emino's Undertaking quest's items and mobiles (namespace `Server.Engines.Quests.Ninja`).

use muo_content::creature::enums::{AIType, FightMode};
use muo_content::creature::setters::{new_creature, set_skill};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_content::items::cont::{self, WOODEN_CHEST};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::lr_reason::LRReason;
use muo_server::map::MapId;
use muo_server::mobile::{AccessLevel, MobileType};
use muo_server::skills::SkillName;

use super::core::{Conversation, Quest, load, with_quest};
use super::emino::*;
use super::items::{
    DYNAMIC_TELEPORTER, QUEST_ITEM, QuestItemClass, TeleporterClass, new_teleporter,
};
use super::quester::{
    BASE_QUESTER, QUESTER_C, QUESTER_OPS, QUESTER_VC, QuesterOps, get_new_container,
};
use super::util::{add_item_hued, add_item_named, set_body};
use crate::quester;

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

// ---- the teleporters ----

macro_rules! teleporter {
    ($(#[$m:meta])* $name:ident, $class:ident = $full:literal, hue $hue:literal, $dest:expr) => {
        static $class: TeleporterClass = TeleporterClass { get_destination: $dest, not_working_message: 1063198 };
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&DYNAMIC_TELEPORTER),
            construct: Some(|w| new_teleporter(w, &$name, 0x51C, $hue)),
            family: Some(&$class),
            label_number: |_, _| 1026157, // teleporter
            serialize: |w, id, out| {
                (DYNAMIC_TELEPORTER.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (DYNAMIC_TELEPORTER.deserialize)(w, id, r)?;
                r.read_i32()?;
                Ok(())
            },
            ..DYNAMIC_TELEPORTER
        };
    };
}

teleporter!(
    /// `BlueNinjaQuestTeleporter`.
    BLUE_NINJA_QUEST_TELEPORTER, BLUE_CLASS = "Server.Engines.Quests.Ninja.BlueNinjaQuestTeleporter", hue 0x2,
    |w, _, player| {
        let ok = quest(w, player).is_some_and(|q| q.has_objective(&GAIN_INN_INFORMATION_OBJECTIVE));
        ok.then_some((Point3D::new(411, 1116, 0), MapId::MALAS))
    }
);

teleporter!(
    /// `GreenNinjaQuestTeleporter`.
    GREEN_NINJA_QUEST_TELEPORTER, GREEN_CLASS = "Server.Engines.Quests.Ninja.GreenNinjaQuestTeleporter", hue 0x17E,
    |w, _, player| {
        let ok = quest(w, player).is_some_and(|q| q.has_objective(&USE_TELEPORTER_OBJECTIVE));
        ok.then_some((Point3D::new(410, 1125, 0), MapId::MALAS))
    }
);

teleporter!(
    /// `WhiteNinjaQuestTeleporter`.
    WHITE_NINJA_QUEST_TELEPORTER, WHITE_CLASS = "Server.Engines.Quests.Ninja.WhiteNinjaQuestTeleporter", hue 0x47E,
    |w, _, player| {
        let qs = quest(w, player)?;
        let i = qs.find_objective(&SEARCH_FOR_SWORD_OBJECTIVE)?;
        if !qs.objectives[i].completed(&qs) {
            with_quest(w, player, |w, q| q.complete_objective(w, i));
        }
        Some((Point3D::new(411, 1085, 0), MapId::MALAS))
    }
);

// ---- the quest items ----

static EMINOS_KATANA_CLASS: QuestItemClass = QuestItemClass {
    can_drop: |w, _, player| !super::core::quest_is(w, player, &QUEST),
    accepted: |w, id| w.item(id).is_none_or(|i| i.deleted()),
};

/// `Server.Engines.Quests.Ninja.EminosKatana`: `[SerializationGenerator(0)]`, so an encoded version.
pub static EMINOS_KATANA: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Ninja.EminosKatana",
    parent: Some(&QUEST_ITEM),
    construct: Some(|w| crate::ml_quests::item_support::template_new(w, &EMINOS_KATANA)),
    family: Some(&EMINOS_KATANA_CLASS),
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

static NOTE_FOR_ZOEL_CLASS: QuestItemClass = QuestItemClass {
    can_drop: |w, _, player| !super::core::quest_is(w, player, &QUEST),
    accepted: |w, id| w.item(id).is_none_or(|i| i.deleted()),
};

crate::quest_item! {
    /// `Server.Engines.Quests.Ninja.NoteForZoel`.
    NOTE_FOR_ZOEL = "Server.Engines.Quests.Ninja.NoteForZoel" : QUEST_ITEM, |w| {
        let id = ops::new_item_with_id(w, &NOTE_FOR_ZOEL, 0x14EF);
        muo_server::item::base::set_hue(w, id, 0x6B9);
        id
    };
    family: Some(&NOTE_FOR_ZOEL_CLASS),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1063186, // A Note for Zoel
}

crate::quest_item! {
    /// `Server.Engines.Quests.Ninja.GuardianBarrier`.
    GUARDIAN_BARRIER = "Server.Engines.Quests.Ninja.GuardianBarrier" : ITEM_BASE, |w| {
        let id = ops::new_item_with_id(w, &GUARDIAN_BARRIER, 0x3967);
        ops::set_movable(w, id, false);
        ops::set_visible(w, id, false);
        id
    };
    on_move_over: |w, id, m| {
        if w.mobile(m).is_some_and(|x| x.access_level > AccessLevel::Player) {
            return true;
        }
        let y = w.item(id).map_or(0, |i| i.location.y);
        if y >= muo_content::mx::location(w, m).y {
            return true;
        }
        if muo_content::creature::is_creature(w, m) {
            let master = muo_content::creature::control::get_master(w, m);
            return master.is_some_and(|ms| {
                y >= muo_content::mx::location(w, ms).y
                    && muo_content::mx::in_range_point(w, ms, ops::get_world_location(w, id), 4)
            });
        }
        if is_player(w, m)
            && let Some(qs) = quest(w, m)
            && let Some(i) = qs.find_objective(&SNEAK_PAST_GUARDIANS_OBJECTIVE)
        {
            if w.mobile(m).is_some_and(|x| x.hidden) {
                return true; // Hidden ninjas can pass
            }
            if !taught_how_to_use_skills(&qs.objectives[i]) {
                with_quest(w, m, |w, q| {
                    *q.objectives[i].state_mut::<bool>() = true;
                    q.add_conversation(w, Conversation::new(&NEED_TO_HIDE_CONVERSATION));
                });
            }
        }
        false
    },
}

// ---- EminosKatanaChest ----

/// `EminosKatanaChest.GenerateTreasure()`.
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

fn give_katana(w: &mut World, player: MobileId) -> bool {
    match muo_content::factory::make_item(w, "Server.Engines.Quests.Ninja.EminosKatana") {
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

/// `Server.Engines.Quests.Ninja.EminosKatanaChest`: `[SerializationGenerator(0)]`, so an encoded version.
pub static EMINOS_KATANA_CHEST: ItemType = ItemType {
    full_name: "Server.Engines.Quests.Ninja.EminosKatanaChest",
    parent: Some(&WOODEN_CHEST),
    construct: Some(|w| {
        let id = cont::new_lockable(w, &EMINOS_KATANA_CHEST, 0);
        muo_content::templates::apply_item_as(w, id, EMINOS_KATANA_CHEST.full_name);
        generate_treasure(w, id);
        id
    }),
    is_deco_container: |_, _| false,
    on_double_click: |w, id, from| {
        if is_player(w, from)
            && muo_content::mx::in_range_point(w, from, ops::get_world_location(w, id), 2)
            && let Some(qs) = quest(w, from)
        {
            if has_lost_eminos_katana(w, from) {
                give_katana(w, from);
            } else if let Some(i) = pending(&qs, &HALLWAY_WALK_OBJECTIVE)
                && give_katana(w, from)
            {
                generate_treasure(w, id);
                with_quest(w, from, |w, q| q.complete_objective(w, i));
            }
        }
        (WOODEN_CHEST.on_double_click)(w, id, from);
    },
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
                .find_objective(&HALLWAY_WALK_OBJECTIVE)
                .is_some_and(|i| stolen_treasure(&qs.objectives[i]));
            if stolen {
                muo_content::mx::msg(w, from, 1063247);
            } else {
                return true;
            }
        }
        false
    },
    on_item_lifted: |w, _, from, _| {
        if is_player(w, from)
            && let Some(qs) = quest(w, from)
            && let Some(i) = qs.find_objective(&HALLWAY_WALK_OBJECTIVE)
        {
            with_quest(w, from, |_, q| *q.objectives[i].state_mut::<bool>() = true);
        }
    },
    serialize: |w, id, out| {
        (WOODEN_CHEST.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (WOODEN_CHEST.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..WOODEN_CHEST
};

// ---- the mobiles ----

/// The resurrection offer the healers and Mardoth/Emino/Zoel make in `OnMovement`.
pub fn resurrect_on_movement(
    w: &mut World,
    npc: MobileId,
    m: MobileId,
    old: Point3D,
    players_only: bool,
) {
    let frozen = w.mobile(m).is_some_and(|x| x.frozen);
    if (players_only && !is_player(w, m))
        || frozen
        || muo_content::mx::alive(w, m)
        || !muo_content::mx::in_range(w, npc, m, 4)
        || muo_content::mx::in_range_point(w, npc, old, 4)
        || !muo_content::mx::in_los(w, npc, m)
    {
        return;
    }
    let (loc, map) = (muo_content::mx::location(w, m), muo_content::mx::map(w, m));
    let fits = map.is_some_and(|map| {
        muo_server::map::fit::can_fit(w, map, loc.x, loc.y, loc.z, 16, false, false, true)
    });
    if !fits {
        muo_content::mx::msg(w, m, 502391); // Thou can not be resurrected there!
    } else {
        let d = muo_content::mx::get_direction_to(w, npc, m);
        muo_content::mx::set_direction(w, npc, d);
        muo_content::mx::play_sound(w, m, 0x214);
        muo_content::mx::fixed_effect(w, m, 0x376A, 10, 16);
        if let Some(f) = super::hooks::quest_hooks(w).and_then(|h| h.resurrect_gump_healer) {
            f(w, m);
        }
    }
}

fn on_movement_resurrect(w: &mut World, npc: MobileId, m: MobileId, old: Point3D) {
    resurrect_on_movement(w, npc, m, old, false);
}

static EMINO_OPS: QuesterOps = QuesterOps {
    talk_number: -1,
    auto_talk_range: |_, _, _| 2,
    on_talk: |w, emino, player, _| {
        let Some(qs) = quest(w, player) else { return };
        if has_lost_note_for_zoel(w, player) {
            match muo_content::factory::make_item(w, "Server.Engines.Quests.Ninja.NoteForZoel") {
                Some(n) if muo_server::mobile::part2::items::place_in_backpack(w, player, n) => {
                    with_quest(w, player, |w, q| {
                        q.add_conversation(w, Conversation::new(&LOST_NOTE_CONVERSATION))
                    });
                }
                n => {
                    if let Some(n) = n {
                        muo_server::item::base::delete(w, n);
                    }
                    muo_content::mx::msg(w, player, 1046260);
                }
            }
            return;
        }
        if has_lost_eminos_katana(w, player) {
            with_quest(w, player, |w, q| {
                q.add_conversation(w, Conversation::new(&LOST_SWORD_CONVERSATION))
            });
            return;
        }
        if let Some(i) = pending(&qs, &FIND_EMINO_BEGIN_OBJECTIVE) {
            with_quest(w, player, |w, q| q.complete_objective(w, i));
            return;
        }
        if let Some(i) = pending(&qs, &USE_TELEPORTER_OBJECTIVE) {
            match muo_content::factory::make_item(w, "Server.Engines.Quests.Ninja.NoteForZoel") {
                Some(n) if muo_server::mobile::part2::items::place_in_backpack(w, player, n) => {
                    with_quest(w, player, |w, q| q.complete_objective(w, i));
                    for name in [
                        "Server.Items.LeatherNinjaPants",
                        "Server.Items.LeatherNinjaMitts",
                    ] {
                        if let Some(it) = muo_content::factory::make_item(w, name) {
                            muo_server::mobile::part2::items::add_to_backpack(w, player, it);
                        }
                    }
                }
                n => {
                    if let Some(n) = n {
                        muo_server::item::base::delete(w, n);
                    }
                    muo_content::mx::msg(w, player, 1046260);
                }
            }
            return;
        }
        if let Some(i) = pending(&qs, &RETURN_FROM_INN_OBJECTIVE) {
            let cont = get_new_container(w);
            for _ in 0..10 {
                if let Some(p) = muo_content::factory::make_item(w, "Server.Items.LesserHealPotion")
                {
                    muo_server::item::container::drop_item_default(w, cont, p);
                }
            }
            for name in [
                "Server.Items.LeatherNinjaHood",
                "Server.Items.LeatherNinjaJacket",
            ] {
                if let Some(it) = muo_content::factory::make_item(w, name) {
                    muo_server::item::container::drop_item_default(w, cont, it);
                }
            }
            if muo_server::mobile::part2::items::place_in_backpack(w, player, cont) {
                with_quest(w, player, |w, q| q.complete_objective(w, i));
            } else {
                muo_server::item::base::delete(w, cont);
                muo_content::mx::msg(w, player, 1046260);
            }
            return;
        }
        if qs.is_objective_in_progress(&SLAY_HENCHMEN_OBJECTIVE) {
            with_quest(w, player, |w, q| {
                q.add_conversation(w, Conversation::new(&CONTINUE_SLAY_HENCHMEN_CONVERSATION))
            });
            return;
        }
        if let Some(i) = pending(&qs, &GIVE_EMINO_SWORD_OBJECTIVE) {
            let pack = w.mobile(player).and_then(|m| m.backpack);
            let Some(katana) = pack.and_then(|p| {
                muo_content::factory::find_item_by_type(
                    w,
                    p,
                    "Server.Engines.Quests.Ninja.EminosKatana",
                )
            }) else {
                return;
            };
            let stolen = qs
                .find_objective(&HALLWAY_WALK_OBJECTIVE)
                .is_some_and(|i| stolen_treasure(&qs.objectives[i]));
            let Some(kama) = muo_content::factory::make_item(w, "Server.Items.Kama") else {
                return;
            };
            if let Some(f) = super::hooks::quest_hooks(w).and_then(|h| h.apply_runic_attributes) {
                f(w, kama, 1, 10, if stolen { 20 } else { 30 });
            }
            if muo_server::mobile::part2::items::place_in_backpack(w, player, kama) {
                muo_server::item::base::delete(w, katana);
                with_quest(w, player, |w, q| {
                    q.complete_objective(w, i);
                    if stolen {
                        q.add_conversation(w, Conversation::new(&EARN_LESS_GIFTS_CONVERSATION));
                    } else {
                        q.add_conversation(w, Conversation::new(&EARN_GIFTS_CONVERSATION));
                    }
                });
            } else {
                muo_server::item::base::delete(w, kama);
                muo_content::mx::msg(w, player, 1046260);
            }
        }
        let _ = emino;
    },
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Ninja.Emino`.
    EMINO, EMINO_C, EMINO_V = "Server.Engines.Quests.Ninja.Emino" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops EMINO_OPS, encoded false, title Some("the Notorious"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Ninja.Emino"),
        init_outfit: |_, _| {},
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m { on_movement: on_movement_resurrect, }
}

static SILENT_OPS: QuesterOps = QuesterOps {
    talk_number: -1,
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Ninja.EnshroudedFigure`.
    ENSHROUDED_FIGURE, ENSHROUDED_FIGURE_C, ENSHROUDED_FIGURE_V = "Server.Engines.Quests.Ninja.EnshroudedFigure"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops SILENT_OPS, encoded true, title None, extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Ninja.EnshroudedFigure"),
        init_outfit: |_, _| {},
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m { }
}

quester! {
    /// `Server.Engines.Quests.Ninja.JedahEntille`.
    JEDAH_ENTILLE, JEDAH_ENTILLE_C, JEDAH_ENTILLE_V = "Server.Engines.Quests.Ninja.JedahEntille"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops SILENT_OPS, encoded true, title Some("the Silent"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Ninja.JedahEntille"),
        init_outfit: |_, _| {},
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m { }
}

// ---- HiddenFigure ----

static HIDDEN_FIGURE_OPS: QuesterOps = QuesterOps {
    talk_number: -1,
    save: |w, id, out| {
        out.write_i32(
            w.mobile(id)
                .and_then(|m| m.ext.get::<i32>())
                .copied()
                .unwrap_or(0),
        )
    },
    load: |w, id, r| {
        let message = r.read_i32()?;
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(message);
        }
        Ok(())
    },
    auto_talk_range: |_, _, _| 3,
    on_talk: |w, figure, player, _| {
        let message = w
            .mobile(figure)
            .and_then(|m| m.ext.get::<i32>())
            .copied()
            .unwrap_or(0);
        let ns = muo_server::mobile::part1::props_b::net_state(w, player);
        muo_server::mobile::part2::messages::private_overhead_message_number(
            w,
            figure,
            muo_primitives::MessageType::REGULAR,
            0x3B2,
            message,
            "",
            ns,
        );
    },
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Ninja.HiddenFigure`: its `Message` is saved (`[SerializableField(0)]`).
    HIDDEN_FIGURE, HIDDEN_FIGURE_C, HIDDEN_FIGURE_V = "Server.Engines.Quests.Ninja.HiddenFigure"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops HIDDEN_FIGURE_OPS, encoded true, title None, extra |w, id| {
            // Message = _messages.RandomElement()
            let m = if muo_content::hooks::rnd::below(w, 2) == 0 { 1063191 } else { 1063192 }; // They won't find me here. / Ah, a quiet hideout.
            if let Some(mob) = w.mobile_mut(id) {
                mob.ext.insert(m);
            }
        };
    v {
        init_body: |w, id| {
            muo_server::mobile::part2::stats::init_stats(w, id, 100, 100, 25);
            let hue = muo_content::vendor::human_random_skin_hue(w);
            muo_server::mobile::base::set_hue(w, id, hue);
            let female = muo_content::hooks::rnd::bool(w);
            muo_server::mobile::part1::props_b::set_female(w, id, female);
            set_body(w, id, if female { 0x191 } else { 0x190 });
            let name = muo_content::misc::small::random_name(w, if female { "female" } else { "male" });
            if let Some(f) = w.mobile(id).map(|m| m.ty.set_name) {
                f(w, id, Some(name));
            }
        },
        init_outfit: |w, id| {
            muo_content::vendor::assign_random_hair_random_hue(w, id);
            let h = muo_content::vendor::get_random_hue(w, id);
            add_item_hued(w, id, "Server.Items.TattsukeHakama", h);
            add_item_named(w, id, "Server.Items.Kasa");
            let h = muo_content::vendor::get_random_hue(w, id);
            add_item_hued(w, id, "Server.Items.HakamaShita", h);
            let shoe = if muo_content::hooks::rnd::bool(w) { "Server.Items.Shoes" } else { "Server.Items.Sandals" };
            let h = muo_content::vendor::get_shoe_hue(w, id);
            add_item_hued(w, id, shoe, h);
        },
    }
    c { }
    m { }
}

// ---- Zoel ----

static ZOEL_OPS: QuesterOps = QuesterOps {
    talk_number: -1,
    auto_talk_range: |_, _, _| 2,
    can_talk_to: |w, _, to| super::core::quest_is(w, to, &QUEST),
    on_talk: |w, _, player, _| {
        if let Some(qs) = quest(w, player)
            && let Some(i) = pending(&qs, &FIND_ZOEL_OBJECTIVE)
        {
            with_quest(w, player, |w, q| q.complete_objective(w, i));
        }
    },
    ..QUESTER_OPS
};

quester! {
    /// `Server.Engines.Quests.Ninja.Zoel`.
    ZOEL, ZOEL_C, ZOEL_V = "Server.Engines.Quests.Ninja.Zoel" : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops ZOEL_OPS, encoded false, title Some("the Masterful Tactician"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Ninja.Zoel"),
        init_outfit: |_, _| {},
    }
    c { default_name: muo_content::templates::creature_props::default_name, }
    m {
        on_movement: on_movement_resurrect,
        on_drag_drop: |w, zoel, from, dropped| {
            if is_player(w, from)
                && quest(w, from).is_some()
                && super::util::item_is(w, dropped, "Server.Engines.Quests.Ninja.NoteForZoel")
                && let Some(qs) = quest(w, from)
                && let Some(i) = pending(&qs, &GIVE_ZOEL_NOTE_OBJECTIVE)
            {
                muo_server::item::base::delete(w, dropped);
                with_quest(w, from, |w, q| q.complete_objective(w, i));
                return true;
            }
            (BASE_QUESTER.on_drag_drop)(w, zoel, from, dropped)
        },
    }
}

// ---- Henchman ----

static HENCHMAN_C: CreatureType = CreatureType {
    default_name: |_, _| Some("a henchman"),
    always_murderer: |_, _| true,
    ..BASE_CREATURE_C
};

/// `Server.Engines.Quests.Ninja.Henchman`.
pub static HENCHMAN: MobileType = MobileType {
    full_name: "Server.Engines.Quests.Ninja.Henchman",
    parent: Some(&BASE_CREATURE),
    construct: Some(|w| {
        let id = new_creature(
            w,
            &HENCHMAN,
            AIType::AiMelee,
            FightMode::Aggressor,
            muo_content::creature::enums::DEFAULT_RANGE_PERCEPTION,
            1,
        );
        muo_server::mobile::part2::stats::init_stats(w, id, 45, 30, 5);
        let hue = muo_content::vendor::human_random_skin_hue(w);
        muo_server::mobile::base::set_hue(w, id, hue);
        set_body(w, id, 0x190);
        muo_content::vendor::assign_random_hair_random_hue(w, id);
        super::util::assign_facial_random_hue(w, id);
        add_item_named(w, id, "Server.Items.LeatherNinjaJacket");
        add_item_named(w, id, "Server.Items.LeatherNinjaPants");
        add_item_named(w, id, "Server.Items.NinjaTabi");
        let weapon = if muo_content::hooks::rnd::bool(w) {
            "Server.Items.Kama"
        } else {
            "Server.Items.Tessen"
        };
        add_item_named(w, id, weapon);
        set_skill(w, id, SkillName::Swords, 50.0);
        set_skill(w, id, SkillName::Tactics, 50.0);
        id
    }),
    family: Some(&HENCHMAN_C),
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_CREATURE
};

/// The mobile classes of this folder.
pub static MOBILES: &[&MobileType] = &[
    &EMINO,
    &ENSHROUDED_FIGURE,
    &HENCHMAN,
    &HIDDEN_FIGURE,
    &JEDAH_ENTILLE,
    &ZOEL,
];

/// The item classes of this folder.
pub static ITEMS: &[&ItemType] = &[
    &BLUE_NINJA_QUEST_TELEPORTER,
    &EMINOS_KATANA,
    &EMINOS_KATANA_CHEST,
    &GREEN_NINJA_QUEST_TELEPORTER,
    &GUARDIAN_BARRIER,
    &NOTE_FOR_ZOEL,
    &WHITE_NINJA_QUEST_TELEPORTER,
];
