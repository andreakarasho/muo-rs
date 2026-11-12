//! The parts of the quest definitions that are code and not tables (the definition modules call these by name):
//! `Spellweaving.AwardTo`, `HumanInNeed.AwardTo`, the `InternalObjective` / `InternalReward` subclasses, and the
//! overrides of `Generate`, `OnAccepted` and `GetRewards` that do more than set fields.

use muo_primitives::{Point3D, TimeSpan};
use muo_server::effects::{self, Targeted};
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::part2::messages as mm;
use muo_server::skills::{self, SkillName};
use muo_server::world::World;

use super::player_of;
use crate::ml_quests::engine;
use crate::ml_quests::model::*;

/// Classes this file needs that other crates own (`Pitcher`, `Spellbook`). Unset: the classes are absent (no water, no fill).
#[derive(Default)]
pub struct QuestItemHooks {
    /// `item is Pitcher { Content: BeverageType.Water, Quantity: > 0 }`.
    pub pitcher_has_water: Option<fn(&World, ItemId) -> bool>,
    /// `book.Content = (1ul << book.BookCount) - 1` for a `Spellbook`.
    pub spellbook_fill: Option<fn(&mut World, ItemId)>,
    /// `new RecipeScroll(recipeID)`. Unset: the satchels drop no scroll.
    pub recipe_scroll: Option<fn(&mut World, i32) -> Option<ItemId>>,
}

fn hooks(w: &World) -> Option<&QuestItemHooks> {
    w.resources.get::<QuestItemHooks>()
}

// ---- Spellweaving: `public static class Spellweaving` -----------------------------------------------------

/// `Spellweaving.AwardTo(pm)`.
pub fn spellweaving_award_to(w: &mut World, pm: MobileId) {
    if w.mobile(pm).is_none() {
        return;
    }

    if engine::get_or_create_context(w, pm).get_flag(flag::SPELLWEAVING) {
        return;
    }
    engine::get_or_create_context(w, pm).set_flag(flag::SPELLWEAVING, true);

    let Some((loc, map)) = w.mobile(pm).map(|m| (m.location, m.map)) else {
        return;
    };

    let item = muo_server::item::effect_item::create(
        w,
        loc,
        map,
        muo_server::item::effect_item::DEFAULT_DURATION,
    );
    effects::send_location_particles_unknown(w, item, 0, 0, 0, 5060, 0);
    effects::play_sound(w, loc, map, 0x243);

    for (dx, dy) in [(-6, -6), (-4, -6), (-6, -4)] {
        let from = Targeted::point(Point3D::new(loc.x + dx, loc.y + dy, loc.z + 15), map);
        effects::send_moving_particles_layer(
            w, from, pm, 0x36D4, 7, 0, false, true, 0x497, 0, 9502, 1, 0, 255, 0x100,
        );
    }

    effects::send_target_particles_full(
        w,
        pm,
        0x375A,
        35,
        90,
        0x00,
        0x00,
        9502,
        muo_server::effects::EffectLayer(255),
        0x100,
    );
}

// ---- Heritage: `HumanInNeed` ------------------------------------------------------------------------------

/// `HumanInNeed.AwardTo(pm)`.
pub fn human_in_need_award_to(w: &mut World, pm: MobileId) {
    if let Some(q) = engine::find_quest(w, "HumanInNeed") {
        engine::set_done_quest(w, pm, q, muo_primitives::DateTime::MIN);
    }
    // You have demonstrated your compassion!  Your kind actions have been noted.
    mm::send_localized_message(w, pm, 1074949, "", 0x2A);
}

// ---- BlightedGrove ----------------------------------------------------------------------------------------

/// `AlreadyDead.InternalObjective`.
pub fn already_dead_objective() -> Objective {
    Objective::collect(10, "Bone", td(1074963)) // (10) workable samples
        .hide_detail()
}

/// `Eureka.GetRewards`.
pub fn eureka_get_rewards(w: &mut World, inst: InstanceId) {
    let pm = player_of(w, inst);

    if !muo_content::player_mobile::has_recipe(w, pm, 32) {
        // The ability is awarded regardless of blacksmithy skill
        muo_content::player_mobile::acquire_recipe(w, pm, 32);

        if skills::base(w, pm, SkillName::Blacksmith) < 45.0 {
            // TODO: Verify threshold
            // You observe carefully but you can't grasp the complexities of smithing a bone handled machete.
            mm::send_localized_message(w, pm, 1075005, "", 0x3B2);
        } else {
            mm::send_localized_message(w, pm, 1075006, "", 0x3B2); // You have learned how to smith a bone handled machete!
        }
    }

    engine::base_get_rewards(w, inst);
}

// ---- Heritage ---------------------------------------------------------------------------------------------

/// `WisdomOfTheSphynx.InternalObjective`.
pub fn wisdom_of_the_sphynx_objective() -> Objective {
    Objective::collect(1, "Pitcher", td(1074869)) // The answer to the riddle.
        .hide_detail()
        .check_item(|w, item| {
            hooks(w)
                .and_then(|h| h.pitcher_has_water)
                .is_some_and(|f| f(w, item))
        })
}

// ---- MistakenIdentity -------------------------------------------------------------------------------------

/// `NotQuiteThatEasy.Generate`: `PutDeco(new BedlamTeleporter(), (2067, 1371, -75), Map.Malas)`.
pub fn not_quite_that_easy_generate(w: &mut World) {
    put_deco(
        w,
        "BedlamTeleporter",
        Point3D::new(2067, 1371, -75),
        MapId::MALAS,
    );
}

/// `NotQuiteThatEasy.OnAccepted`.
pub fn not_quite_that_easy_on_accepted(w: &mut World, inst: InstanceId) {
    let pm = player_of(w, inst);
    engine::get_or_create_context(w, pm).set_flag(flag::BEDLAM_ACCESS, true); // Permanent access
}

/// `MLQuest.PutDeco(deco, loc, map)`: removes the items of the same graphic and Z there first.
fn put_deco(w: &mut World, name: &str, loc: Point3D, map: MapId) {
    let Some(deco) = engine::make_item_by_name(w, name) else {
        return;
    };
    let Some(graphic) = w.item(deco).map(|i| i.item_id) else {
        return;
    };

    let stale: Vec<ItemId> = muo_server::item::deps::items_at(w, map, loc)
        .into_iter()
        .filter(|&i| {
            w.item(i)
                .is_some_and(|x| x.item_id == graphic && x.location.z == loc.z)
        })
        .collect();
    for item in stale {
        muo_server::item::base::delete(w, item);
    }

    muo_server::item::ops::move_to_world_map(w, deco, loc, Some(map));
}

// ---- NewHavenSkillTraining --------------------------------------------------------------------------------

/// `TheAllureOfDarkMagic.InternalReward`: the book comes with every spell.
pub fn the_allure_of_dark_magic_reward() -> Reward {
    Reward::item(td(1078052), "NecromancerSpellbook", 1) // Complete Necromancer Spellbook
        .with_create(|w, item| {
            if let Some(f) = hooks(w).and_then(|h| h.spellbook_fill) {
                f(w, item);
            }
        })
}

// ---- NewHavenTraining -------------------------------------------------------------------------------------

/// `MoreOrePlease.InternalObjective`: any type of ore is allowed, as long as it is the 6585 graphic.
pub fn more_ore_please_objective() -> Objective {
    Objective::collect(5, "BaseOre", td(1026585)) // ore
        .check_item(|w, item| w.item(item).is_some_and(|i| i.item_id == 6585))
}

#[allow(dead_code)]
const _: Option<TimeSpan> = None;
