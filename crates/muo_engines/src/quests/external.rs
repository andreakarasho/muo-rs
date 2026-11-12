//! The quest side of the places outside Engines/Quests that touch a player's quest: `BankCheck.OnDoubleClick`,
//! `Corpse.OnDoubleClick`, `BaseCreature.OnDeath` (the Doom bones), `HarvestTarget` (graves), `Fishing`, `BaseBeverage`
//! (swamp water), `Spyglass` and `BaseAI.HandleSpecialHerdingCase`. The first three are installed as hooks in
//! [`install`]; the rest are called by the (engine) code that owns the call site.

use muo_server::World;
use muo_server::ids::{ItemId, MobileId};

use super::core::{Quest, with_quest_of};
use super::witch::{self, FIND_INGREDIENT_OBJECTIVE};
use super::{collector, dark_tides, summoning, uzeraan};

/// `Ingredient.SwampWater`, `Bones` and `StarChart`.
const SWAMP_WATER: i32 = 10;
const BONES: i32 = 12;
const STAR_CHART: i32 = 13;

/// `qs.FindObjective<FindIngredientObjective>()` is in progress and wants `ingredient`.
fn wants_ingredient(q: &Quest, ingredient: i32) -> Option<usize> {
    let i = q.find_objective(&FIND_INGREDIENT_OBJECTIVE)?;
    (!q.objectives[i].completed(q) && witch::current_ingredient(&q.objectives[i]) == ingredient)
        .then_some(i)
}

/// The tail of `BankCheck.OnDoubleClick`: cashing a check completes the banker objective of the two quests.
pub fn bank_check_cashed(w: &mut World, from: MobileId) {
    with_quest_of(w, from, &dark_tides::QUEST, |w, q| {
        if let Some(i) = q
            .find_objective(&dark_tides::CASH_BANK_CHECK_OBJECTIVE)
            .filter(|&i| !q.objectives[i].completed(q))
        {
            q.complete_objective(w, i);
        }
    });
    with_quest_of(w, from, &uzeraan::QUEST, |w, q| {
        if let Some(i) = q
            .find_objective(&uzeraan::CASH_BANK_CHECK_OBJECTIVE)
            .filter(|&i| !q.objectives[i].completed(q))
        {
            q.complete_objective(w, i);
        }
    });
}

/// The quest part of `Corpse.OnDoubleClick` after `CheckLoot`: true when the open ends there.
pub fn corpse_quest_open(w: &mut World, player: MobileId, corpse: ItemId) -> bool {
    // UzeraanTurmoilQuest: the daemon bone in a corpse of the Haven graveyard
    let uz = with_quest_of(w, player, &uzeraan::QUEST, |w, q| {
        let Some(i) = q.find_objective(&uzeraan::GET_DAEMON_BONE_OBJECTIVE) else {
            return false;
        };
        let ours = uzeraan::corpse_with_bone(&q.objectives[i]) == Some(corpse);
        if !(ours && (!q.objectives[i].completed(q) || uzeraan::has_lost_daemon_bone(w, player))) {
            return false;
        }
        match muo_content::factory::make_item(w, "Server.Engines.Quests.Haven.QuestDaemonBone") {
            Some(bone) if muo_server::mobile::part2::items::place_in_backpack(w, player, bone) => {
                *q.objectives[i].state_mut::<Option<ItemId>>() = None;
                // You rummage through the bones and find a Daemon Bone!  You quickly place the item in your pack.
                super::util::msg_hue(w, player, 1049341, 0x22);
                if !q.objectives[i].completed(q) {
                    q.complete_objective(w, i);
                }
            }
            bone => {
                if let Some(b) = bone {
                    muo_server::item::base::delete(w, b);
                }
                // Rummaging through the bones you find a Daemon Bone, but can't pick it up because your pack is too
                // full.  Come back when you have more room in your pack.
                super::util::msg_hue(w, player, 1049342, 0x22);
            }
        }
        true
    });
    if let Some(done) = uz {
        return done;
    }

    // TheSummoningQuest: the golden skull from the devourer's corpse
    with_quest_of(w, player, &summoning::QUEST, |w, q| {
        let Some(i) = q.find_objective(&summoning::VANQUISH_DAEMON_OBJECTIVE) else {
            return;
        };
        let skull_corpse = q.objectives[i]
            .state::<summoning::Vanquish>()
            .corpse_with_skull;
        if q.objectives[i].completed(q) && skull_corpse == Some(corpse) {
            match muo_content::factory::make_item(w, "Server.Engines.Quests.Doom.GoldenSkull") {
                Some(sk) if muo_server::mobile::part2::items::place_in_backpack(w, player, sk) => {
                    q.objectives[i]
                        .state_mut::<summoning::Vanquish>()
                        .corpse_with_skull = None;
                    // For your valor in combating the devourer, you have been awarded a golden skull.
                    muo_content::mx::msg(w, player, 1050022);
                    q.complete(w);
                }
                sk => {
                    if let Some(s) = sk {
                        muo_server::item::base::delete(w, s);
                    }
                    // You find a golden skull, but your backpack is too full to carry it.
                    muo_content::mx::msg(w, player, 1050023);
                }
            }
        }
    });
    false
}

/// `HarvestTarget.OnTarget` for a grave (`itemID` 0xED3, 0xEDF, 0xEE0, 0xEE1, 0xEE2, 0xEE8) while mining: true when
/// the Witch Apprentice quest took the bones, or the player is not on that quest (`qs is not WitchApprenticeQuest` returns;
/// the harvest ends); false: the dig goes on.
pub fn mining_grave(w: &mut World, player: MobileId) -> bool {
    with_quest_of(w, player, &witch::QUEST, |w, q| {
        let Some(i) = wants_ingredient(q, BONES) else {
            return false;
        };
        // You finish your grim work, finding some of the specific bones listed in the Hag's recipe.
        muo_content::mx::msg(w, player, 1055037);
        q.complete_objective(w, i);
        true
    })
    .unwrap_or(true) // `qs is not WitchApprenticeQuest` returns: no digging in a grave without the quest
}

/// `Fishing.SpecialHarvest`: true when the Collector's pearls took the cast.
pub fn fishing_special_harvest(w: &mut World, player: MobileId) -> bool {
    with_quest_of(w, player, &collector::QUEST, |w, q| {
        let Some(i) = q
            .find_objective(&collector::FISH_PEARLS_OBJECTIVE)
            .filter(|&i| !q.objectives[i].completed(q))
        else {
            return false;
        };
        if muo_content::hooks::rnd::bool(w) {
            // You pull a shellfish out of the water, and find a rainbow pearl inside of it.
            super::util::msg_hue(w, player, 1055086, 0x59);
            q.add_progress(w, i);
        } else {
            // You pull a shellfish out of the water, but it doesn't have a rainbow pearl.
            super::util::msg_hue(w, player, 1055087, 0x2C);
        }
        true
    })
    .unwrap_or(false)
}

/// `BaseBeverage.Fill_OnTarget` onto a land tile: does the player's Witch Apprentice quest want swamp water?
pub fn wants_swamp_water(w: &World, player: MobileId) -> bool {
    super::core::load(w, player)
        .is_some_and(|q| q.is(&witch::QUEST) && wants_ingredient(&q, SWAMP_WATER).is_some())
}

/// ... and the container was dipped into a swamp tile (after `Delete()`).
pub fn swamp_water_collected(w: &mut World, player: MobileId) {
    with_quest_of(w, player, &witch::QUEST, |w, q| {
        if let Some(i) = wants_ingredient(q, SWAMP_WATER) {
            // You dip the container into the disgusting swamp water, collecting enough for the Hag's vile stew.
            muo_content::mx::msg(w, player, 1055035);
            q.complete_objective(w, i);
        }
    });
}

/// `BaseBeverage.Fill_OnTarget` onto a land tile: the swamp tiles of the Witch Apprentice quest. True when the
/// objective took the water (the beverage is then deleted).
pub fn swamp_water(w: &mut World, player: MobileId, tile_id: i32) -> bool {
    const SWAMP_TILES: [(i32, i32); 7] = [
        (0x9C4, 0x9EB),
        (0x3D65, 0x3D65),
        (0x3DC0, 0x3DD9),
        (0x3DDB, 0x3DDC),
        (0x3DDE, 0x3EF0),
        (0x3FF6, 0x3FF6),
        (0x3FFC, 0x3FFE),
    ];
    if !wants_swamp_water(w, player)
        || !SWAMP_TILES
            .iter()
            .any(|&(a, b)| tile_id >= a && tile_id <= b)
    {
        return false;
    }
    swamp_water_collected(w, player);
    true
}

/// `Spyglass.OnDoubleClick`: the star chart, given the hour of the player's location.
pub fn spyglass_star_chart(w: &mut World, player: MobileId, hours: i32) {
    with_quest_of(w, player, &witch::QUEST, |w, q| {
        if let Some(i) = wants_ingredient(q, STAR_CHART) {
            if !(5..=17).contains(&hours) {
                // You gaze up into the glittering night sky.  With great care, you compose a chart of the most prominent
                // star patterns.
                muo_content::mx::msg(w, player, 1055040);
                q.complete_objective(w, i);
            } else {
                // You gaze up into the sky, but it is not dark enough to see any stars.
                muo_content::mx::msg(w, player, 1055039);
            }
        }
    });
}

/// `BaseAI.HandleSpecialHerdingCase`: the familiar of a Dark Tides necromancer reached (1076, 450).
pub fn herding_fetch_abraxus(w: &mut World, familiar: MobileId, master: MobileId) {
    with_quest_of(w, master, &dark_tides::QUEST, |w, q| {
        if let Some(i) = q
            .find_objective(&dark_tides::FETCH_ABRAXUS_SCROLL_OBJECTIVE)
            .filter(|&i| !q.objectives[i].completed(q))
        {
            if let Some(scroll) =
                muo_content::factory::make_item(w, "Server.Engines.Quests.Necro.ScrollOfAbraxus")
            {
                muo_server::mobile::part2::items::add_to_backpack(w, familiar, scroll);
            }
            q.complete_objective(w, i);
        }
    });
}

/// Sets the hooks that code outside the quests calls through.
pub fn install(w: &mut World) {
    let c = w
        .resources
        .get_mut::<muo_content::items::common::CommonItemHooks>();
    c.bank_check_cashed = Some(bank_check_cashed);
    c.corpse_quest_open = Some(corpse_quest_open);
    w.resources
        .get_mut::<muo_content::items::corpse::CorpseClassHooks>()
        .quest_corpse = Some(|w, owner, hair, facial, equip| {
        let ours = w
            .mobile(owner)
            .is_some_and(|m| m.ty.is(&super::uzeraan_mobs::MILITIA_FIGHTER));
        ours.then(|| super::uzeraan_mobs::new_militia_fighter_corpse(w, owner, hair, facial, equip))
    });
    w.resources
        .get_mut::<muo_items::food::beverage::BeverageHooks>()
        .swamp_water = Some(swamp_water);
    w.resources
        .get_mut::<muo_items::skill_items::small::SpyglassHooks>()
        .star_chart = Some(spyglass_star_chart);
    w.resources
        .get_mut::<muo_content::ai::base::HerdingHooks>()
        .fetch_abraxus = Some(herding_fetch_abraxus);
    let d = w.resources.get_mut::<muo_content::hooks::DeathHooks>();
    d.daemon_bones_for = Some(summoning::get_daemon_bones_for);
}
