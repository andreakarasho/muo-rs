//! The AI classes MeleeAI, AnimalAI, BerserkAI, PredatorAI, ArcherAI, ThiefAI, VendorAI and HealerAI.
//! MageAI is `mage.rs` and FamiliarAI `familiar.rs`.

use muo_server::ids::{ItemId, MobileId};
use muo_server::world::World;

use super::base::{self, BASE_AI, acquire_focus_mob, debug_say, is_valid_combatant};
use super::movement::{move_to, walk_mobile_range, walk_random_in_home};
use super::{ActionType, AiClass, set_action};
use crate::abilities::{self, MonsterAbilityTrigger};
use crate::creature::data::{cd_mut, read};
use crate::creature::enums::*;
use crate::creature::misc::distance_sqrt;
use crate::creature::vtable::ctype;
use crate::hooks::rnd;
use crate::mx;

fn rp(w: &World, id: MobileId) -> i32 {
    read(w, id, 0, |d| d.range_perception)
}

fn fight_mode(w: &World, id: MobileId) -> FightMode {
    read(w, id, FightMode::None, |d| d.fight_mode)
}

fn name(w: &World, id: MobileId) -> String {
    muo_server::mobile::base::name_base(w, id).unwrap_or_default()
}

fn focus(w: &World, id: MobileId) -> Option<MobileId> {
    read(w, id, None, |d| d.focus_mob)
}

fn set_focus(w: &mut World, id: MobileId, f: Option<MobileId>) {
    if let Some(d) = cd_mut(w, id) {
        d.focus_mob = f;
    }
}

/// `combatant == null || Deleted || Map != Mobile.Map || !Alive || IsDeadBondedPet`, inverted.
fn combatant_usable(w: &World, id: MobileId, c: Option<MobileId>) -> bool {
    c.is_some_and(|c| {
        !mx::deleted(w, c)
            && mx::map(w, c) == mx::map(w, id)
            && mx::alive(w, c)
            && !w.mobile(c).is_some_and(|m| (m.ty.is_dead_bonded_pet)(w, c))
    })
}

fn face(w: &mut World, id: MobileId, c: MobileId) {
    let last = w.mobile(id).map_or(0, |m| m.last_move_time);
    if w.tick_count - last > 400 {
        let d = mx::get_direction_to(w, id, c);
        muo_server::mobile::part1::props_b::set_direction(w, id, d);
    }
}

fn engage_focus(w: &mut World, id: MobileId) {
    let f = focus(w, id);
    if let Some(f) = f {
        debug_say(w, id, &format!("I have detected {}, attacking", name(w, f)));
    }
    mx::set_combatant(w, id, f);
    set_action(w, id, ActionType::Combat);
}

fn use_abilities(w: &mut World, id: MobileId, c: Option<MobileId>) -> bool {
    if abilities::trigger_ability(w, id, MonsterAbilityTrigger::COMBAT_ACTION, c) {
        let n = c.map(|c| name(w, c)).unwrap_or_default();
        debug_say(w, id, &format!("I used my abilities on {n}!"));
        return true;
    }
    false
}

// ---- MeleeAI -------------------------------------------------------------------------------------------

pub static MELEE_AI: AiClass = AiClass {
    name: "MeleeAI",
    flee_health_threshold: 0.2,
    flee_chance: 0.1,
    do_action_wander: melee_wander,
    do_action_combat: melee_combat,
    do_action_guard: melee_guard,
    do_action_flee: melee_flee,
    ..BASE_AI
};

fn acquire(w: &mut World, id: MobileId) -> bool {
    let (r, m) = (rp(w, id), fight_mode(w, id));
    acquire_focus_mob(w, id, r, m, false, false, true)
}

fn melee_wander(w: &mut World, id: MobileId) -> bool {
    if acquire(w, id) {
        engage_focus(w, id);
    } else {
        debug_say(w, id, "I am wandering");
        mx::set_warmode(w, id, false);
        base::do_action_wander(w, id);
    }
    true
}

fn melee_combat(w: &mut World, id: MobileId) -> bool {
    let mut combatant = mx::combatant(w, id);
    let valid = combatant_usable(w, id, combatant)
        && combatant.is_some_and(|c| mx::in_los(w, id, c) || base::is_hostile_pub(w, id, c));
    // `IsValidCombatant` of MeleeAI is its own (private) check: alive, same map, not a dead pet.
    let _ = valid;
    if !combatant_usable(w, id, combatant) {
        debug_say(w, id, "My combatant is gone, so my guard is up");
        set_action(w, id, ActionType::Guard);
        return true;
    }
    let c0 = combatant.expect("usable");
    if !mx::in_range(w, id, c0, rp(w, id)) {
        // HandleOutOfRangeCombatant
        if acquire(w, id) {
            let f = focus(w, id);
            mx::set_combatant(w, id, f);
            set_focus(w, id, None);
        } else {
            let leash = ctype(w, id).map_or(0, |c| (c.chase_leash_range)(w, id));
            if !mx::in_range(w, id, c0, leash) {
                mx::set_combatant(w, id, None);
            }
            if mx::combatant(w, id).is_none() {
                debug_say(w, id, "My combatant has fled, so I am on guard.");
                set_action(w, id, ActionType::Guard);
                return true;
            }
        }
        combatant = mx::combatant(w, id);
    }
    let Some(c) = combatant else { return true };
    // AttemptMoveToCombatant
    let range_fight = read(w, id, 1, |d| d.range_fight);
    if !move_to(w, id, c, range_fight) {
        if acquire(w, id) {
            let f = focus(w, id).map(|f| name(w, f)).unwrap_or_default();
            debug_say(
                w,
                id,
                &format!("My move is blocked, so I am going to attack {f}."),
            );
            let f = focus(w, id);
            mx::set_combatant(w, id, f);
            set_action(w, id, ActionType::Combat);
        } else if distance_sqrt(w, id, c) > (rp(w, id) + 1) as f64 {
            debug_say(
                w,
                id,
                &format!("I cannot find {}, so my guard is up.", name(w, c)),
            );
            set_action(w, id, ActionType::Guard);
            return true;
        } else {
            debug_say(
                w,
                id,
                &format!("I cannot reach {} but continuing to try.", name(w, c)),
            );
        }
    }
    face(w, id, c);
    if abilities::trigger_ability(w, id, MonsterAbilityTrigger::COMBAT_ACTION, Some(c)) {
        debug_say(w, id, "I used my abilities!");
    }
    true
}

fn melee_guard(w: &mut World, id: MobileId) -> bool {
    if acquire(w, id) {
        engage_focus(w, id);
    } else {
        base::do_action_guard(w, id);
    }
    true
}

fn melee_flee(w: &mut World, id: MobileId) -> bool {
    let threshold = MELEE_AI.flee_health_threshold;
    if mx::hits(w, id) as f64 > mx::hits_max(w, id) as f64 * threshold {
        debug_say(w, id, "I am stronger now, so I will continue fighting.");
        set_action(w, id, ActionType::Combat);
    } else {
        let c = mx::combatant(w, id);
        set_focus(w, id, c);
        base::do_action_flee(w, id);
    }
    true
}

// ---- AnimalAI -------------------------------------------------------------------------------------------

pub static ANIMAL_AI: AiClass = AiClass {
    name: "AnimalAI",
    flee_health_threshold: 0.1,
    flee_chance: 0.1,
    backoff_chance: 0.5,
    do_action_wander: |w, id| {
        if acquire(w, id) {
            engage_focus(w, id);
        } else {
            base::do_action_wander(w, id);
        }
        true
    },
    do_action_combat: animal_combat,
    do_action_flee: |w, id| {
        let (r, m) = (rp(w, id), fight_mode(w, id));
        acquire_focus_mob(w, id, r * 2, m, true, false, true);
        if focus(w, id).is_none() {
            let c = mx::combatant(w, id);
            set_focus(w, id, c);
        }
        base::do_action_flee(w, id)
    },
    ..BASE_AI
};

fn animal_combat(w: &mut World, id: MobileId) -> bool {
    let combatant = mx::combatant(w, id);
    let ok = is_valid_combatant(w, id, combatant);
    if !ok {
        debug_say(w, id, "My combatant is gone!");
        set_action(w, id, ActionType::Wander);
        return true;
    }
    let c = combatant.expect("valid");
    let rf = read(w, id, 1, |d| d.range_fight);
    if !walk_mobile_range(w, id, c, 1, rf, rf) {
        if distance_sqrt(w, id, c) > (rp(w, id) + 1) as f64 {
            debug_say(w, id, &format!("I cannot find {}", name(w, c)));
            set_action(w, id, ActionType::Wander);
            return true;
        }
    } else {
        face(w, id, c);
    }
    use_abilities(w, id, Some(c));
    true
}

// ---- BerserkAI --------------------------------------------------------------------------------------------

pub static BERSERK_AI: AiClass = AiClass {
    name: "BerserkAI",
    do_action_wander: |w, id| {
        debug_say(w, id, "I have no combatant");
        let r = rp(w, id);
        if acquire_focus_mob(w, id, r, FightMode::Closest, false, true, true) {
            engage_focus(w, id);
        } else {
            base::do_action_wander(w, id);
        }
        true
    },
    do_action_combat: berserk_combat,
    do_action_guard: |w, id| {
        let (r, m) = (rp(w, id), fight_mode(w, id));
        if acquire_focus_mob(w, id, r, m, false, true, true) {
            engage_focus(w, id);
        } else {
            base::do_action_guard(w, id);
        }
        true
    },
    ..BASE_AI
};

fn berserk_combat(w: &mut World, id: MobileId) -> bool {
    let combatant = mx::combatant(w, id);
    if !combatant_usable(w, id, combatant) {
        debug_say(w, id, "My combatant is gone, so my guard is up");
        set_action(w, id, ActionType::Guard);
        return true;
    }
    let c = combatant.expect("usable");
    let rf = read(w, id, 1, |d| d.range_fight);
    if !walk_mobile_range(w, id, c, 1, rf, rf) {
        debug_say(w, id, &format!("I am still not in range of {}", name(w, c)));
        if distance_sqrt(w, id, c) as i32 > rp(w, id) + 1 {
            debug_say(w, id, &format!("I have lost {}", name(w, c)));
            set_action(w, id, ActionType::Guard);
            return true;
        }
    } else {
        face(w, id, c);
    }
    use_abilities(w, id, Some(c));
    true
}

// ---- PredatorAI (not used by ChangeAIType, which builds a MeleeAI for AI_Predator) -----------------------------

pub static PREDATOR_AI: AiClass = AiClass {
    name: "PredatorAI",
    do_action_wander: |w, id| {
        let (r, m) = (rp(w, id), fight_mode(w, id));
        if mx::combatant(w, id).is_some() {
            debug_say(w, id, "I am hurt or being attacked, I kill him");
            set_action(w, id, ActionType::Combat);
        } else if acquire_focus_mob(w, id, r, m, true, false, true) {
            debug_say(w, id, "There is something near, I go away");
            set_action(w, id, ActionType::Backoff);
        } else {
            base::do_action_wander(w, id);
        }
        true
    },
    do_action_combat: |w, id| {
        let combatant = mx::combatant(w, id);
        if !combatant_usable(w, id, combatant) {
            debug_say(w, id, "My combatant is gone, so my guard is up");
            set_action(w, id, ActionType::Guard);
            return true;
        }
        let c = combatant.expect("usable");
        let rf = read(w, id, 1, |d| d.range_fight);
        if !walk_mobile_range(w, id, c, 1, rf, rf) {
            if distance_sqrt(w, id, c) > (rp(w, id) + 1) as f64 {
                debug_say(w, id, &format!("I cannot find {}", name(w, c)));
                set_action(w, id, ActionType::Wander);
                return true;
            }
            debug_say(w, id, &format!("I should be closer to {}", name(w, c)));
        }
        use_abilities(w, id, Some(c));
        true
    },
    do_action_backoff: |w, id| {
        let r = rp(w, id);
        if mx::hits(w, id) != mx::hits_max(w, id) || mx::combatant(w, id).is_some() {
            set_action(w, id, ActionType::Combat);
        } else if acquire_focus_mob(w, id, r * 2, FightMode::Closest, true, false, true) {
            if let Some(f) = focus(w, id)
                && walk_mobile_range(w, id, f, 1, r, r * 2)
            {
                debug_say(w, id, "Well, here I am safe");
                set_action(w, id, ActionType::Wander);
            }
        } else {
            debug_say(w, id, "I have lost my focus, lets relax");
            set_action(w, id, ActionType::Wander);
        }
        true
    },
    ..BASE_AI
};

// ---- ArcherAI -------------------------------------------------------------------------------------------

pub static ARCHER_AI: AiClass = AiClass {
    name: "ArcherAI",
    flee_health_threshold: 0.2,
    flee_chance: 0.1,
    do_action_wander: |w, id| {
        debug_say(w, id, "I have no combatant");
        if acquire(w, id) {
            engage_focus(w, id);
            true
        } else {
            base::do_action_wander(w, id)
        }
    },
    do_action_combat: archer_combat,
    do_action_guard: melee_guard,
    ..BASE_AI
};

fn archer_combat(w: &mut World, id: MobileId) -> bool {
    let combatant = mx::combatant(w, id);
    if !combatant_usable(w, id, combatant) {
        debug_say(w, id, "My combatant is gone, so my guard is up");
        set_action(w, id, ActionType::Guard);
        return true;
    }
    let c = combatant.expect("usable");
    let rf = read(w, id, 1, |d| d.range_fight);
    let max_range = crate::hooks::hooks(w)
        .and_then(|h| h.weapon_max_range)
        .map_or(1, |f| f(w, id));
    if !walk_mobile_range(w, id, c, 1, rf, max_range) {
        debug_say(w, id, &format!("I am still not in range of {}", name(w, c)));
        let leash = ctype(w, id).map_or(0, |c| (c.chase_leash_range)(w, id));
        if !mx::in_range(w, id, c, leash) {
            debug_say(w, id, &format!("I have lost {}", name(w, c)));
            mx::set_combatant(w, id, None);
            set_action(w, id, ActionType::Guard);
            return true;
        }
    } else {
        face(w, id, c);
    }
    if use_abilities(w, id, Some(c)) {
        return true;
    }
    // When we have no ammo, we flee
    let has_arrows = muo_server::mobile::part1::props_b::backpack(w, id)
        .is_some_and(|p| crate::factory::find_item_by_type(w, p, "Server.Items.Arrow").is_some());
    if !has_arrows {
        set_action(w, id, ActionType::Flee);
    }
    true
}

// ---- ThiefAI --------------------------------------------------------------------------------------------

pub static THIEF_AI: AiClass = AiClass {
    name: "ThiefAI",
    do_action_wander: |w, id| {
        debug_say(w, id, "I have no combatant");
        if acquire(w, id) {
            engage_focus(w, id);
        } else {
            base::do_action_wander(w, id);
        }
        true
    },
    do_action_combat: thief_combat,
    do_action_guard: melee_guard,
    do_action_flee: |w, id| {
        if mx::hits(w, id) > mx::hits_max(w, id) / 2 {
            debug_say(w, id, "I am stronger now, so I will continue fighting");
            set_action(w, id, ActionType::Combat);
        } else {
            let c = mx::combatant(w, id);
            set_focus(w, id, c);
            base::do_action_flee(w, id);
        }
        true
    },
    ..BASE_AI
};

/// `ThiefAI._toDisarm`, in the AI's `Ext`.
#[derive(Default)]
struct ThiefState {
    to_disarm: Option<ItemId>,
}

fn to_disarm(w: &World, id: MobileId) -> Option<ItemId> {
    super::ai(w, id)?.ext.get::<ThiefState>()?.to_disarm
}

fn set_to_disarm(w: &mut World, id: MobileId, item: Option<ItemId>) {
    if let Some(a) = super::ai_mut(w, id) {
        match a.ext.get_mut::<ThiefState>() {
            Some(s) => s.to_disarm = item,
            None => a.ext.insert(ThiefState { to_disarm: item }),
        }
    }
}

/// `Fists.DisarmRequest(m)`.
pub fn disarm_request(w: &mut World, m: MobileId) {
    use muo_server::layer::Layer;
    use muo_server::skills::{SkillName, value};
    if crate::core::aos(w) || crate::core::expansion(w) < muo_data::expansion::Expansion::UOR {
        return;
    }
    if !crate::items::weapon_hooks::duel_allow_special_ability(w, m, "Disarm", true) {
        return;
    }
    let (arms, wres) = (
        value(w, m, SkillName::ArmsLore),
        value(w, m, SkillName::Wrestling),
    );
    let one = muo_server::mobile::part2::items::find_item_on_layer(w, m, Layer::OneHanded);
    // `item is null or Spellbook && m.FindItemOnLayer(Layer.TwoHanded) == null`
    let free_hands = match one {
        None => true,
        Some(i) => {
            crate::factory::item_is(w, i, "Server.Items.Spellbook")
                && muo_server::mobile::part2::items::find_item_on_layer(w, m, Layer::TwoHanded)
                    .is_none()
        }
    };
    if !free_hands {
        mx::msg(w, m, 1004029); // You must have your hands free to attempt to disarm your opponent.
        set_disarm_ready(w, m, false);
    } else if arms >= 80.0 && wres >= 80.0 {
        muo_server::mobile::base::disruptive_action(w, m);
        let ready = !w.mobile(m).is_some_and(|x| x.disarm_ready);
        set_disarm_ready(w, m, ready);
        mx::msg(w, m, if ready { 1019013 } else { 1019014 });
    } else {
        mx::msg(w, m, 1004002); // You are not skilled enough to disarm your opponent.
        set_disarm_ready(w, m, false);
    }
}

/// `Fists.StunRequest(m)`.
pub fn stun_request(w: &mut World, m: MobileId) {
    use muo_server::layer::Layer;
    use muo_server::skills::{SkillName, value};
    if crate::core::aos(w) || crate::core::expansion(w) < muo_data::expansion::Expansion::UOR {
        return;
    }
    if !crate::items::weapon_hooks::duel_allow_special_ability(w, m, "Stun", true) {
        return;
    }
    let (anat, wres) = (
        value(w, m, SkillName::Anatomy),
        value(w, m, SkillName::Wrestling),
    );
    let one = muo_server::mobile::part2::items::find_item_on_layer(w, m, Layer::OneHanded);
    let free_hands = match one {
        None => true,
        Some(i) => {
            crate::factory::item_is(w, i, "Server.Items.Spellbook")
                && muo_server::mobile::part2::items::find_item_on_layer(w, m, Layer::TwoHanded)
                    .is_none()
        }
    };
    let set = |w: &mut World, v: bool| {
        if let Some(x) = w.mobile_mut(m) {
            x.stun_ready = v;
        }
    };
    if !free_hands {
        mx::msg(w, m, 1004031); // You must have your hands free to attempt to stun your opponent.
        set(w, false);
    } else if anat >= 80.0 && wres >= 80.0 {
        muo_server::mobile::base::disruptive_action(w, m);
        let ready = !w.mobile(m).is_some_and(|x| x.stun_ready);
        set(w, ready);
        mx::msg(w, m, if ready { 1019011 } else { 1019012 });
    } else {
        mx::msg(w, m, 1004008); // You are not skilled enough to stun your opponent.
        set(w, false);
    }
}

fn set_disarm_ready(w: &mut World, m: MobileId, v: bool) {
    if let Some(x) = w.mobile_mut(m) {
        x.disarm_ready = v;
    }
}

/// `TryStealFrom<T>(combatant)`: uses Stealing on the first `T` in the combatant's pack.
fn try_steal_from(w: &mut World, id: MobileId, combatant: MobileId, type_name: &str) -> bool {
    let Some(pack) = muo_server::mobile::part1::props_b::backpack(w, combatant) else {
        return false;
    };
    let Some(steal) = find_in_pack(w, pack, type_name) else {
        return false;
    };
    use_stealing_on(w, id, steal);
    true
}

/// `container.FindItemByType<T>()` for a type that this crate doesn't hold: by full name, recursing.
fn find_in_pack(w: &World, container: ItemId, type_name: &str) -> Option<ItemId> {
    let items = w.item(container)?.items.clone();
    for it in items {
        if crate::factory::item_is(w, it, type_name) {
            return Some(it);
        }
        if let Some(found) = find_in_pack(w, it, type_name) {
            return Some(found);
        }
    }
    None
}

/// `Mobile.UseSkill(SkillName.Stealing); Mobile.Target?.Invoke(Mobile, item)`.
fn use_stealing_on(w: &mut World, id: MobileId, item: ItemId) {
    mx::use_skill(w, id, muo_server::skills::SkillName::Stealing);
    if let Some(t) = w.mobile(id).and_then(|m| m.target) {
        muo_server::target::invoke(
            w,
            t,
            id,
            muo_server::target::Targeted::Entity(muo_server::ids::EntityId::Item(item)),
        );
    }
}

/// The stealing/disarm branch of `ThiefAI.DoActionCombat` once the thief is in range. False when it chose to flee.
fn thief_steal(w: &mut World, id: MobileId, combatant: MobileId) -> bool {
    use muo_server::item::LootType;
    use muo_server::layer::Layer;
    let find = |w: &World, m, l| muo_server::mobile::part2::items::find_item_on_layer(w, m, l);
    let pack_of = |w: &mut World, m| muo_server::mobile::part1::props_b::backpack(w, m);
    let my_pack = pack_of(w, id);
    let child_of = |w: &World, item: ItemId, pack: Option<ItemId>| {
        pack.is_some_and(|p| {
            muo_server::item::ops::is_child_of(w, item, Some(muo_server::ids::EntityId::Item(p)))
        })
    };
    // `_toDisarm?.IsChildOf(Mobile.Backpack) != false`
    let cur = to_disarm(w, id);
    if cur.is_none_or(|i| child_of(w, i, my_pack)) {
        let found =
            find(w, combatant, Layer::OneHanded).or_else(|| find(w, combatant, Layer::TwoHanded));
        set_to_disarm(w, id, found);
    }
    let mut disarm = to_disarm(w, id);
    let ready = w.mobile(id).is_some_and(|m| m.disarm_ready);
    let (wres, arms) = {
        use muo_server::skills::{SkillName, value};
        (
            value(w, id, SkillName::Wrestling),
            value(w, id, SkillName::ArmsLore),
        )
    };
    if !crate::core::aos(w) && !ready && wres >= 80.0 && arms >= 80.0 && disarm.is_some() {
        disarm_request(w, id);
    }
    let next_skill = w.mobile(id).map_or(0, |m| m.next_skill_time);
    let skill_ready = w.tick_count.wrapping_sub(next_skill) >= 0;
    let their_pack = pack_of(w, combatant);
    disarm = to_disarm(w, id);
    if let Some(d) = disarm.filter(|&d| child_of(w, d, their_pack)) {
        let lt = w.item(d).map(|i| i.loot_type);
        if skill_ready && lt != Some(LootType::Blessed) && lt != Some(LootType::Newbied) {
            debug_say(w, id, "Trying to steal from combatant.");
            use_stealing_on(w, id, d);
        }
    } else if disarm.is_none() && skill_ready {
        debug_say(
            w,
            id,
            &format!("Trying to steal from {}.", name(w, combatant)),
        );
        let mut did_steal = try_steal_from(w, id, combatant, "Server.Items.Bandage");
        did_steal = try_steal_from(w, id, combatant, "Server.Items.Nightshade") || did_steal;
        did_steal = try_steal_from(w, id, combatant, "Server.Items.BlackPearl") || did_steal;
        did_steal = try_steal_from(w, id, combatant, "Server.Items.MandrakeRoot") || did_steal;
        if !did_steal {
            debug_say(
                w,
                id,
                &format!("I am going to flee from {}", name(w, combatant)),
            );
            set_action(w, id, ActionType::Flee);
            return false;
        }
    }
    true
}

fn thief_combat(w: &mut World, id: MobileId) -> bool {
    let combatant = mx::combatant(w, id);
    if !combatant_usable(w, id, combatant) {
        debug_say(w, id, "My combatant is gone, so my guard is up");
        set_action(w, id, ActionType::Guard);
        return true;
    }
    let c = combatant.expect("usable");
    let rf = read(w, id, 1, |d| d.range_fight);
    if !walk_mobile_range(w, id, c, 1, rf, rf) {
        debug_say(w, id, &format!("I should be closer to {}", name(w, c)));
    } else if !thief_steal(w, id, c) {
        return true; // fled
    }
    // We are low on health, should we flee? (10 + diff)% chance to flee
    let can_flee = ctype(w, id).is_some_and(|x| (x.can_flee)(w, id));
    if mx::hits(w, id) < mx::hits_max(w, id) * 20 / 100 && can_flee {
        let flee_chance = 10 + (mx::hits(w, c) - mx::hits(w, id)).max(0);
        if rnd::from(w, 0, 100) > flee_chance {
            debug_say(w, id, &format!("I am going to flee from {}", name(w, c)));
            set_action(w, id, ActionType::Flee);
        }
        return true;
    }
    use_abilities(w, id, mx::combatant(w, id));
    true
}

// ---- VendorAI -------------------------------------------------------------------------------------------

fn guard_message(w: &mut World) -> i32 {
    // Guards! A villan attacks me! / Guards! Help!
    if rnd::bool(w) { 1005305 } else { 501603 }
}

pub static VENDOR_AI: AiClass = AiClass {
    name: "VendorAI",
    do_action_wander: |w, id| {
        debug_say(w, id, "I'm fine");
        if let Some(c) = mx::combatant(w, id) {
            debug_say(w, id, &format!("{} is attacking me", name(w, c)));
            let n = guard_message(w);
            muo_server::mobile::part2::messages::say_number(w, id, n, "");
            set_action(w, id, ActionType::Flee);
        } else if let Some(f) = focus(w, id) {
            debug_say(w, id, &format!("{} has talked to me", name(w, f)));
            set_action(w, id, ActionType::Interact);
        } else {
            mx::set_warmode(w, id, false);
            base::do_action_wander(w, id);
        }
        true
    },
    do_action_interact: |w, id| {
        let customer = focus(w, id);
        if let Some(c) = mx::combatant(w, id) {
            debug_say(w, id, &format!("{} is attacking me", name(w, c)));
            let n = guard_message(w);
            muo_server::mobile::part2::messages::say_number(w, id, n, "");
            set_action(w, id, ActionType::Flee);
            return true;
        }
        match customer.filter(|&c| !mx::deleted(w, c) && mx::map(w, c) == mx::map(w, id)) {
            None => {
                debug_say(w, id, "My customer has disappeared");
                set_focus(w, id, None);
                set_action(w, id, ActionType::Wander);
            }
            Some(c) => {
                let rf = read(w, id, 1, |d| d.range_fight);
                if mx::in_range(w, c, id, rf) {
                    debug_say(w, id, &format!("I am with {}", name(w, c)));
                    let d = mx::get_direction_to(w, id, c);
                    muo_server::mobile::part1::props_b::set_direction(w, id, d);
                } else {
                    debug_say(w, id, &format!("{} is gone", name(w, c)));
                    set_focus(w, id, None);
                    set_action(w, id, ActionType::Wander);
                }
            }
        }
        true
    },
    do_action_guard: |w, id| {
        let c = mx::combatant(w, id);
        set_focus(w, id, c);
        base::do_action_guard(w, id)
    },
    handles_on_speech: |w, id, from| {
        mx::in_range(w, from, id, 4) || super::speech::handles_on_speech(w, id, from)
    },
    on_speech: vendor_on_speech,
    ..BASE_AI
};

/// `VendorAI.OnSpeech(e)`: the "vendor buy/sell" and "<name> buy/sell" keywords.
fn vendor_on_speech(
    w: &mut World,
    id: MobileId,
    e: &mut muo_server::speech_event::SpeechEventArgs,
) {
    super::speech::on_speech(w, id, e);
    let from = e.mobile;
    let is_vendor = crate::vendor::data(w, id).is_some();
    if is_vendor && mx::in_range(w, from, id, if w.core_aos() { 1 } else { 4 }) && !e.handled {
        if e.has_keyword(0x14D) {
            // *vendor sell*
            e.handled = true;
            crate::vendor::trade::vendor_sell(w, id, from);
            set_focus(w, id, Some(from));
        } else if e.has_keyword(0x3C) {
            // *vendor buy*
            e.handled = true;
            crate::vendor::trade::vendor_buy(w, id, from);
            set_focus(w, id, Some(from));
        } else if super::speech::was_named(w, id, &e.speech) {
            if e.has_keyword(0x177) {
                // *sell*
                e.handled = true;
                crate::vendor::trade::vendor_sell(w, id, from);
            } else if e.has_keyword(0x171) {
                // *buy*
                e.handled = true;
                crate::vendor::trade::vendor_buy(w, id, from);
            }
            set_focus(w, id, Some(from));
        }
    }
}

// ---- HealerAI -------------------------------------------------------------------------------------------

pub static HEALER_AI: AiClass = AiClass {
    name: "HealerAI",
    think: healer_think,
    ..BASE_AI
};

fn need_cure(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| x.poison.is_some())
}
fn need_gheal(w: &World, m: MobileId) -> bool {
    mx::hits(w, m) < mx::hits_max(w, m) - 40
}
fn need_lheal(w: &World, m: MobileId) -> bool {
    mx::hits(w, m) < mx::hits_max(w, m) - 10
}

/// `Find(funcs)`: the closest same-team creature that needs one of the helps.
fn find_to_help(
    w: &mut World,
    id: MobileId,
    funcs: &[fn(&World, MobileId) -> bool],
) -> Option<MobileId> {
    if mx::deleted(w, id) || mx::map(w, id).is_none() {
        return None;
    }
    let team = read(w, id, 0, |d| d.team);
    let mut prio = 0.0;
    let mut found: Option<MobileId> = None;
    for m in mx::mobiles_in_range(w, id, rp(w, id)) {
        let sees = w
            .mobile(id)
            .map(|x| x.ty)
            .is_some_and(|x| (x.can_see_mobile)(w, id, m));
        if !sees || !crate::creature::is_creature(w, m) || read(w, m, -1, |d| d.team) != team {
            continue;
        }
        if funcs.iter().any(|f| f(w, m)) {
            let val = -distance_sqrt(w, id, m);
            if found.is_none() || val > prio {
                prio = val;
                found = Some(m);
            }
        }
    }
    found
}

const CURE_SPELL: &str = "CureSpell";
const GREATER_HEAL_SPELL: &str = "GreaterHealSpell";
const HEAL_SPELL: &str = "HealSpell";

/// `new XSpell(Mobile).Cast()`; false when the class isn't registered or the cast is refused.
fn cast_named(w: &mut World, id: MobileId, name: &str) -> bool {
    use crate::spells::{registry::new_spell_by_name, spell};
    match new_spell_by_name(w, name, id, None) {
        Some(sp) => spell::cast(w, sp),
        None => false,
    }
}

/// `ProcessTarget(targ, funcs)`: answers the target a healing spell opened.
fn healer_process_target(
    w: &mut World,
    id: MobileId,
    tid: muo_server::target::TargetId,
    funcs: &[fn(&World, MobileId) -> bool],
) {
    use muo_server::target::{self, TargetCancelType, Targeted};
    if funcs.is_empty() {
        target::cancel(w, tid, id, TargetCancelType::Canceled);
        return;
    }
    let Some(to_help) = find_to_help(w, id, funcs) else {
        target::cancel(w, tid, id, TargetCancelType::Canceled);
        return;
    };
    let range = w.targets.get(&tid).map_or(-1, |t| t.range);
    if range != -1 && !mx::in_range(w, id, to_help, range) {
        let d = mx::get_direction_to(w, id, to_help) | muo_primitives::Direction::RUNNING;
        super::movement::do_move(w, id, d, false);
    } else {
        target::invoke(
            w,
            tid,
            id,
            Targeted::Entity(muo_server::ids::EntityId::Mobile(to_help)),
        );
    }
}

fn healer_think(w: &mut World, id: MobileId) -> bool {
    if mx::deleted(w, id) {
        return false;
    }
    if let Some(tid) = w.mobile(id).and_then(|m| m.target) {
        // `targ as ISpellTarget<Mobile>` then `.Spell switch { CureSpell, GreaterHealSpell, HealSpell }`
        let spell_name = w
            .targets
            .get(&tid)
            .and_then(|t| t.spell)
            .and_then(|sp| crate::spells::spell::spell_type(w, sp))
            .map(|t| t.full_name.rsplit('.').next().unwrap_or(""));
        let funcs: &[fn(&World, MobileId) -> bool] = match spell_name {
            Some(CURE_SPELL) => &[need_cure],
            Some(GREATER_HEAL_SPELL) => &[need_gheal],
            Some(HEAL_SPELL) => &[need_lheal],
            _ => &[],
        };
        healer_process_target(w, id, tid, funcs);
        return true;
    }
    let all: [fn(&World, MobileId) -> bool; 3] = [need_cure, need_gheal, need_lheal];
    if let Some(to_help) = find_to_help(w, id, &all) {
        if need_cure(w, to_help) {
            debug_say(w, id, &format!("{} needs a cure", name(w, to_help)));
            if !cast_named(w, id, CURE_SPELL) {
                cast_named(w, id, CURE_SPELL);
            }
        } else if need_gheal(w, to_help) {
            debug_say(w, id, &format!("{} needs a greater heal", name(w, to_help)));
            if !cast_named(w, id, GREATER_HEAL_SPELL) {
                cast_named(w, id, HEAL_SPELL);
            }
        } else if need_lheal(w, to_help) {
            debug_say(w, id, &format!("{} needs a lesser heal", name(w, to_help)));
            cast_named(w, id, HEAL_SPELL);
        }
        return true;
    }
    let r = rp(w, id);
    if !acquire_focus_mob(w, id, r, FightMode::Weakest, false, true, false) {
        walk_random_in_home(w, id, 3, 2, 1);
        return true;
    }
    if let Some(f) = focus(w, id) {
        walk_mobile_range(w, id, f, 1, 4, 7);
    }
    // TODO: should it be able to do this?
    let c = mx::combatant(w, id);
    if abilities::trigger_ability(w, id, MonsterAbilityTrigger::COMBAT_ACTION, c) {
        let n = c.map(|c| name(w, c)).unwrap_or_default();
        debug_say(w, id, &format!("I used my abilities on {n}!"));
    }
    true
}
