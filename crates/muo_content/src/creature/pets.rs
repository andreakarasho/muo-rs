//! Pet members of `BaseCreature`: control chance, Pacify, Provoke, FindMyName, TeleportPets and SpillAcid.

use muo_primitives::dotnet::IgnoreCase;
use muo_primitives::{MessageType, Point3D, TimeSpan};
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages;
use muo_server::skills::{self, SkillName};
use muo_server::world::World;

use super::control::{control_master, set_loyalty};
use super::data::{cd, cd_mut, read};
use super::enums::*;
use super::vtable::ctype;
use crate::hooks::{hooks, rnd};
use crate::mx;

/// `GetControlChance`
pub fn get_control_chance(w: &mut World, id: MobileId, m: MobileId, use_base_skill: bool) -> f64 {
    let (min_tame, summoned, loyalty) = read(w, id, (0.0, false, 0), |d| {
        (d.min_tame_skill, d.summoned, d.loyalty)
    });
    if min_tame <= 29.1 || summoned || mx::access_level(w, m) >= AccessLevel::GameMaster {
        return 1.0;
    }
    let mut min_tame = min_tame;
    if min_tame > -24.9
        && hooks(w)
            .and_then(|h| h.animal_taming_mastery)
            .is_some_and(|f| f(w, m, id))
    {
        min_tame = -24.9;
    }
    let (taming, lore) = if use_base_skill {
        (
            skills::base_fixed(w, m, SkillName::AnimalTaming),
            skills::base_fixed(w, m, SkillName::AnimalLore),
        )
    } else {
        (
            skills::fixed(w, m, SkillName::AnimalTaming),
            skills::fixed(w, m, SkillName::AnimalLore),
        )
    };
    let bonus = if w.core_ml() {
        let mut skill_bonus = taming - (min_tame * 10.0) as i32;
        let mut lore_bonus = lore - (min_tame * 10.0) as i32;
        let skill_mod = if skill_bonus < 0 { 28 } else { 6 };
        let lore_mod = if lore_bonus < 0 { 14 } else { 6 };
        skill_bonus *= skill_mod;
        lore_bonus *= lore_mod;
        (skill_bonus + lore_bonus) / 2
    } else {
        let difficulty = (min_tame * 10.0) as i32;
        let weighted = (taming * 4 + lore) / 5;
        let mut bonus = weighted - difficulty;
        if bonus <= 0 {
            bonus *= 14;
        } else {
            bonus *= 6;
        }
        bonus
    };
    let mut chance = (700 + bonus).clamp(220, 990);
    chance -= (MAX_LOYALTY - loyalty) * 10;
    chance as f64 / 1000.0
}

/// `CheckControlChance`
pub fn check_control_chance(w: &mut World, id: MobileId, m: MobileId) -> bool {
    let c = ctype(w, id).expect("creature");
    if (c.get_control_chance)(w, id, m, false) > rnd::double(w) {
        let l = read(w, id, 0, |d| d.loyalty);
        set_loyalty(w, id, l + 1);
        return true;
    }
    let snd = w
        .mobile(id)
        .map(|x| x.ty)
        .map_or(0, |x| (x.get_anger_sound)(w, id));
    mx::play_sound(w, id, snd);
    if super::misc::body_is_animal(w, id) {
        mx::animate(w, id, 10, 5, 1, true, false, 0);
    } else if super::misc::body_is_monster(w, id) {
        mx::animate(w, id, 18, 5, 1, true, false, 0);
    }
    let l = read(w, id, 0, |d| d.loyalty);
    set_loyalty(w, id, l - 3);
    false
}

/// `CanBeControlledBy`
pub fn can_be_controlled_by(w: &mut World, id: MobileId, m: MobileId) -> bool {
    (ctype(w, id).expect("creature").get_control_chance)(w, id, m, false) > 0.0
}

/// `Pacify`
pub fn pacify(w: &mut World, id: MobileId, end: muo_primitives::DateTime) {
    if let Some(d) = cd_mut(w, id) {
        d.bard_pacified = true;
        d.bard_end_time = end;
    }
}

/// `Provoke`
pub fn provoke(w: &mut World, id: MobileId, master: MobileId, target: MobileId, success: bool) {
    if let Some(d) = cd_mut(w, id) {
        d.bard_provoked = true;
    }
    if !w.core_ml() {
        let hue = w.mobile(id).map_or(0x22, |m| m.emote_hue);
        messages::public_overhead_message(
            w,
            id,
            MessageType::EMOTE,
            hue,
            false,
            "*looks furious*",
            false,
            AccessLevel::Player,
        );
    }
    let end = w.now + TimeSpan::from_secs(30);
    if success {
        let snd = w
            .mobile(id)
            .map(|m| m.ty)
            .map_or(0, |m| (m.get_idle_sound)(w, id));
        mx::play_sound(w, id, snd);
        if let Some(d) = cd_mut(w, id) {
            d.bard_master = Some(master);
            d.bard_target = Some(target);
            d.bard_end_time = end;
        }
        mx::set_combatant(w, id, Some(target));
        if cd(w, target).is_some() {
            let unprovokable = ctype(w, target).is_some_and(|c| (c.unprovokable)(w, target));
            let paragon = read(w, target, false, |d| d.is_paragon);
            let base_difficulty = hooks(w).and_then(|h| h.instrument_base_difficulty);
            let hard = base_difficulty.is_some_and(|f| f(w, target) >= 160.0);
            if unprovokable || paragon && hard {
                return;
            }
            if let Some(d) = cd_mut(w, target) {
                d.bard_provoked = true;
                d.bard_master = Some(master);
                d.bard_target = Some(id);
                d.bard_end_time = end;
            }
            mx::set_combatant(w, target, Some(id));
        }
    } else {
        let snd = w
            .mobile(id)
            .map(|m| m.ty)
            .map_or(0, |m| (m.get_anger_sound)(w, id));
        mx::play_sound(w, id, snd);
        if let Some(d) = cd_mut(w, id) {
            d.bard_master = Some(master);
            d.bard_target = Some(target);
        }
    }
}

/// `FindMyName`: does `s` contain every word of the creature's name (and, with `with_all`, the word "all")?
pub fn find_my_name(w: &World, id: MobileId, s: &str, with_all: bool) -> bool {
    let Some(name) = muo_server::mobile::base::name(w, id) else {
        return false;
    };
    if s.encode_utf16().count() < name.encode_utf16().count() {
        return false;
    }
    let words_string: Vec<&str> = s.split(' ').collect();
    for word_name in name.split(' ') {
        let mut found = false;
        for word in &words_string {
            if word.eq_ic(word_name) {
                found = true;
            }
            if with_all && word.eq_ic("all") {
                return true;
            }
        }
        if !found {
            return false;
        }
    }
    true
}

/// `TeleportPets`: moves the pets near `master` that are following, coming or guarding to `loc`.
pub fn teleport_pets(
    w: &mut World,
    master: MobileId,
    loc: Point3D,
    map: muo_server::map::MapId,
    only_bonded: bool,
) {
    let master_map = mx::map(w, master);
    let candidate = |w: &World, m: MobileId| {
        let Some(d) = cd(w, m) else { return false };
        d.controlled
            && matches!(
                d.control_order,
                OrderType::Guard | OrderType::Follow | OrderType::Come
            )
            && control_master(w, m) == Some(master)
            && (!only_bonded || d.is_bonded)
    };
    let pets: Vec<MobileId> = match crate::hooks::pet_player_hooks(w)
        .and_then(|h| h.all_followers)
        .map(|f| f(w, master))
    {
        Some(all) if !all.is_empty() => all
            .into_iter()
            .filter(|&m| {
                mx::map(w, m) == master_map && mx::in_range(w, master, m, 3) && candidate(w, m)
            })
            .collect(),
        _ => mx::mobiles_in_range(w, master, 3)
            .into_iter()
            .filter(|&m| candidate(w, m))
            .collect(),
    };
    for m in pets {
        if let Some(ty) = w.mobile(m).map(|x| x.ty) {
            (ty.move_to_world)(w, m, loc, map);
        }
    }
}

/// `SpillAcid`(target, amount)
pub fn spill_acid(w: &mut World, id: MobileId, target: Option<MobileId>, amount: i32) {
    let map = mx::map(w, id);
    if target.is_some_and(|t| mx::map(w, t).is_none()) || map.is_none() {
        return;
    }
    for _ in 0..amount {
        let (loc, m) = match target {
            Some(t) if amount == 1 => (mx::location(w, t), mx::map(w, t)),
            _ => {
                let here = mx::location(w, id);
                let map = map.expect("checked");
                (
                    muo_server::map::fit::get_random_nearby_location(
                        w, map, here, 2, 0, 10, 16, false, false,
                    ),
                    Some(map),
                )
            }
        };
        let acid = ctype(w, id).and_then(|c| (c.new_harmful_item)(w, id));
        if let (Some(acid), Some(m)) = (acid, m) {
            muo_server::item::ops::move_to_world_map(w, acid, loc, Some(m));
        }
    }
}
