//! Anatomy, EvalInt, ArmsLore, ItemIdentification, TasteID, Cartography and ForensicEval: the skills
//! that tell the player about what they target.
//!
//! `[PlayerVendorTarget]` (on the ArmsLore, ItemID and TasteID targets, which lets them be used on a player vendor's
//! wares) is `Target::player_vendor_target`, which the vendor's backpack reads.

use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::base as mb;
use muo_server::skills::{self, SkillName};
use muo_server::target::Targeted;

use super::common::{
    TargetOptions, check_target_skill, invulnerable_vendor, is_human, item_msg, local_msg,
    mobile_is, msg, private_msg, set_target,
};
use super::hooks::{WeaponKind, ops};
use super::skills_info::set_callback;
use crate::core;

fn entity_mobile(t: Targeted) -> Option<MobileId> {
    match t {
        Targeted::Entity(EntityId::Mobile(m)) => Some(m),
        _ => None,
    }
}

fn entity_item(t: Targeted) -> Option<ItemId> {
    match t {
        Targeted::Entity(EntityId::Item(i)) => Some(i),
        _ => None,
    }
}

// ---- Anatomy ----------------------------------------------------------------------------------------------------

pub fn anatomy_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    set_target(w, m, 8, TargetOptions::default(), anatomy_on_target);
    msg(w, m, 500321); // Whom shall I examine?
    TimeSpan::from_secs(1)
}

fn anatomy_on_target(w: &mut World, from: MobileId, targeted: Targeted) {
    if targeted == Targeted::Entity(EntityId::Mobile(from)) {
        local_msg(w, from, 0x3B2, 500324); // You know yourself quite well enough already.
    } else if let Some(crier) =
        entity_mobile(targeted).filter(|&m| mobile_is(w, m, "Server.Mobiles.TownCrier"))
    {
        private_msg(w, crier, from, 500322); // This person looks fine to me, though he may have some news...
    } else if let Some(vendor) = entity_mobile(targeted).filter(|&m| invulnerable_vendor(w, m)) {
        private_msg(w, vendor, from, 500326); // That can not be inspected.
    } else if let Some(targ) = entity_mobile(targeted) {
        let anatomy = skills::value(w, from, SkillName::Anatomy);
        let margin_of_error = (25 - (anatomy / 4.0) as i32).max(0);

        let err =
            |w: &mut World| utility::random_min_max(&mut w.rng, -margin_of_error, margin_of_error);
        let str = mb::str(w, targ) + err(w);
        let dex = mb::dex(w, targ) + err(w);
        let (stam, stam_max) = w
            .mobile(targ)
            .map_or((0, 1), |t| (t.stam, mb::stam_max(w, targ)));
        let stm = stam * 100 / stam_max.max(1) + err(w);

        let str_mod = (str / 10).clamp(0, 10);
        let dex_mod = (dex / 10).clamp(0, 10);
        let stm_mod = (stm / 10).clamp(0, 10);

        if check_target_skill(w, from, SkillName::Anatomy, targeted, 0.0, 100.0) {
            private_msg(w, targ, from, 1038045 + str_mod * 11 + dex_mod); // That looks [strong] and [dexterous].

            if skills::base(w, from, SkillName::Anatomy) >= 65.0 {
                private_msg(w, targ, from, 1038303 + stm_mod); // That being is at [10,20,...] percent endurance.
            }
        } else {
            private_msg(w, targ, from, 1042666); // You can not quite get a sense of their physical characteristics.
        }
    } else if let Some(item) = entity_item(targeted) {
        item_msg(w, item, from, 500323); // Only living things have anatomies!
    }
}

// ---- EvalInt ----------------------------------------------------------------------------------------------------

pub fn eval_int_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    set_target(w, m, 8, TargetOptions::default(), eval_int_on_target);
    msg(w, m, 500906); // What do you wish to evaluate?
    TimeSpan::from_secs(1)
}

fn eval_int_on_target(w: &mut World, from: MobileId, targeted: Targeted) {
    if targeted == Targeted::Entity(EntityId::Mobile(from)) {
        local_msg(w, from, 0x3B2, 500910); // Hmm, that person looks really silly.
    } else if let Some(crier) =
        entity_mobile(targeted).filter(|&m| mobile_is(w, m, "Server.Mobiles.TownCrier"))
    {
        private_msg(w, crier, from, 500907); // He looks smart enough to remember the news.  Ask him about it.
    } else if let Some(vendor) = entity_mobile(targeted).filter(|&m| invulnerable_vendor(w, m)) {
        private_msg(w, vendor, from, 500909); // That person could probably calculate the cost of what you buy from them.
    } else if let Some(targ) = entity_mobile(targeted) {
        let eval = skills::value(w, from, SkillName::EvalInt);
        let margin_of_error = (20 - (eval / 5.0) as i32).max(0);

        let err =
            |w: &mut World| utility::random_min_max(&mut w.rng, -margin_of_error, margin_of_error);
        let intel = mb::int(w, targ) + err(w);
        let (mana, mana_max) = w
            .mobile(targ)
            .map_or((0, 1), |t| (t.mana, mb::mana_max(w, targ)));
        let mana = mana * 100 / mana_max.max(1) + err(w);

        let int_mod = (intel / 10).clamp(0, 10);
        let mn_mod = (mana / 10).clamp(0, 10);

        let body = if is_human(w, targ) {
            if w.mobile(targ).is_some_and(|t| t.female) {
                11
            } else {
                0
            }
        } else {
            22
        };

        if check_target_skill(w, from, SkillName::EvalInt, targeted, 0.0, 120.0) {
            // He/She/It looks [slighly less intelligent than a rock.]  [Of Average intellect] [etc...]
            private_msg(w, targ, from, 1038169 + int_mod + body);

            if skills::base(w, from, SkillName::EvalInt) >= 76.0 {
                private_msg(w, targ, from, 1038202 + mn_mod); // That being is at [10,20,...] percent mental strength.
            }
        } else {
            private_msg(w, targ, from, 1038166 + body / 11); // You cannot judge his/her/its mental abilities.
        }
    } else if let Some(item) = entity_item(targeted) {
        item_msg(w, item, from, 500908); // It looks smarter than a rock, but dumber than a piece of wood.
    }
}

// ---- ArmsLore ---------------------------------------------------------------------------------------------------

pub fn arms_lore_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    set_target(
        w,
        m,
        2,
        TargetOptions {
            allow_nonlocal: true,
            player_vendor_target: true,
            ..Default::default()
        },
        arms_lore_on_target,
    );
    msg(w, m, 500349); // What item do you wish to get information about?
    TimeSpan::from_secs(1)
}

fn arms_lore_on_target(w: &mut World, from: MobileId, targeted: Targeted) {
    if let Some(item) = entity_item(targeted) {
        if let Some(weap) = ops(w).weapon_facts(w, item) {
            if check_target_skill(w, from, SkillName::ArmsLore, targeted, 0.0, 100.0) {
                if weap.max_hit_points != 0 {
                    let hp = ((f64::from(weap.hit_points) / f64::from(weap.max_hit_points) * 10.0)
                        as i32)
                        .clamp(0, 9);
                    msg(w, from, 1038285 + hp);
                }

                let mut damage = (weap.max_damage + weap.min_damage) / 2;
                let hand = if weap.one_handed { 0 } else { 1 };

                if damage < 3 {
                    damage = 0;
                } else {
                    damage = (f64::from(damage.min(30)) / 5.0).ceil() as i32;
                }

                let number = match weap.kind {
                    WeaponKind::Ranged => 1038224 + damage * 9,
                    WeaponKind::Piercing => 1038218 + hand + damage * 9,
                    WeaponKind::Slashing => 1038220 + hand + damage * 9,
                    WeaponKind::Bashing => 1038222 + hand + damage * 9,
                    WeaponKind::Other => 1038216 + hand + damage * 9,
                };
                msg(w, from, number);

                if weap.poisoned {
                    msg(w, from, 1038284); // It appears to have poison smeared on it.
                }
            } else {
                msg(w, from, 500353); // You are not certain...
            }
            return;
        }
        if let Some(arm) = ops(w).armor_facts(w, item) {
            if check_target_skill(w, from, SkillName::ArmsLore, targeted, 0.0, 100.0) {
                if arm.max_hit_points != 0 {
                    let hp = ((f64::from(arm.hit_points) / f64::from(arm.max_hit_points) * 10.0)
                        as i32)
                        .clamp(0, 9);
                    msg(w, from, 1038285 + hp);
                }

                msg(
                    w,
                    from,
                    1038295 + (arm.armor_rating.min(35.0) / 5.0).ceil() as i32,
                );
            } else {
                msg(w, from, 500353); // You are not certain...
            }
            return;
        }
    } else if let Some(pet) = entity_mobile(targeted)
        && let Some((hp, max_hp)) = ops(w).swamp_dragon_barding(w, pet)
    {
        if check_target_skill(w, from, SkillName::ArmsLore, targeted, 0.0, 100.0) {
            let perc = (4 * hp / max_hp.max(1)).clamp(0, 4);
            private_msg(w, pet, from, 1053021 - perc);
        } else {
            msg(w, from, 500353); // You are not certain...
        }
        return;
    }
    msg(w, from, 500352); // This is neither weapon nor armor.
}

// ---- ItemIdentification -----------------------------------------------------------------------------------------

pub fn item_id_on_use(w: &mut World, from: MobileId) -> TimeSpan {
    msg(w, from, 500343); // What do you wish to appraise and identify?
    set_target(
        w,
        from,
        8,
        TargetOptions {
            allow_nonlocal: true,
            player_vendor_target: true,
            ..Default::default()
        },
        item_id_on_target,
    );
    TimeSpan::from_secs(1)
}

fn item_id_on_target(w: &mut World, from: MobileId, o: Targeted) {
    if let Some(m) = entity_mobile(o) {
        if let Some(ty) = w.mobile(m).map(|x| x.ty) {
            (ty.on_single_click)(w, m, from);
        }
        return;
    }

    let mut identified = false;

    if let Some(item) = entity_item(o) {
        if ops(w).is_identifiable(w, item)
            && check_target_skill(w, from, SkillName::ItemID, o, 0.0, 100.0)
        {
            ops(w).set_identified(w, item);
            identified = true;
        }

        if !core::aos(w)
            && let Some(ty) = w.item(item).map(|i| i.ty)
        {
            (ty.on_single_click)(w, item, from);
        }
    }

    if !identified {
        msg(w, from, 500353); // You are not certain...
    }
}

// ---- TasteID ----------------------------------------------------------------------------------------------------

pub fn taste_id_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    set_target(
        w,
        m,
        2,
        TargetOptions {
            allow_nonlocal: true,
            player_vendor_target: true,
            on_out_of_range: Some(taste_id_out_of_range),
            ..Default::default()
        },
        taste_id_on_target,
    );
    msg(w, m, 502807); // What would you like to taste?
    TimeSpan::from_secs(1)
}

fn taste_id_out_of_range(w: &mut World, from: MobileId, _: Targeted) {
    msg(w, from, 502815); // You are too far away to taste that.
}

fn taste_id_on_target(w: &mut World, from: MobileId, targeted: Targeted) {
    if entity_mobile(targeted).is_some() {
        msg(w, from, 502816); // You feel that such an action would be inappropriate.
        return;
    }
    let Some(item) = entity_item(targeted) else {
        msg(w, from, 502820); // That's not something you can taste.
        return;
    };

    if let Some(poisoned) = ops(w).food_poisoned(w, item) {
        if check_target_skill(w, from, SkillName::TasteID, targeted, 0.0, 100.0) {
            if poisoned {
                item_msg(w, item, from, 1038284); // It appears to have poison smeared on it.
            } else {
                item_msg(w, item, from, 1010600); // You detect nothing unusual about this substance.
            }
        } else {
            // Skill check failed
            item_msg(w, item, from, 502823); // You cannot discern anything about this substance.
        }
    } else if let Some(label) = ops(w).potion_label(w, item) {
        item_msg(w, item, from, 502813); // You already know what kind of potion that is.
        item_msg(w, item, from, label);
    } else if let Some((held, label)) = ops(w).potion_keg(w, item) {
        if held <= 0 {
            item_msg(w, item, from, 502228); // There is nothing in the keg to taste!
        } else {
            item_msg(w, item, from, 502229); // You are already familiar with this keg's contents.
            item_msg(w, item, from, label);
        }
    } else {
        // The target is not food or potion or potion keg.
        msg(w, from, 502820); // That's not something you can taste.
    }
}

// ---- Cartography ------------------------------------------------------------------------------------------------

/// `Cartography.OnUse`: the T2A craft menu, or a hint to use a mapmaker's pen.
pub fn cartography_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    let enabled = w
        .resources
        .get::<crate::configuration::expansion_configuration::T2aCraftSystemEnabled>()
        .is_some_and(|e| e.0);
    if !enabled {
        msg(w, m, 1046444); // Use a mapmaker's pen to draw maps.
        return TimeSpan::ZERO;
    }

    // `T2ACraftSystem.ShowMenu(m, DefCartography.CraftSystem, null)` is the craft engine's, through the hook.
    if let Some(f) = w
        .resources
        .get::<super::SkillGumpHooks>()
        .and_then(|h| h.show_cartography_menu)
    {
        f(w, m);
    }
    TimeSpan::from_secs(1)
}

// ---- ForensicEvaluation -----------------------------------------------------------------------------------------

pub fn forensics_on_use(w: &mut World, m: MobileId) -> TimeSpan {
    set_target(w, m, 10, TargetOptions::default(), forensics_on_target);
    mb::revealing_action(w, m);

    msg(w, m, 501000); // Show me the crime.

    TimeSpan::from_secs(1)
}

fn forensics_on_target(w: &mut World, from: MobileId, target: Targeted) {
    if let Some(m) = entity_mobile(target) {
        if check_target_skill(w, from, SkillName::Forensics, target, 40.0, 100.0) {
            if ops(w).is_thieves_guild_member(w, m) {
                msg(w, from, 501004); // That individual is a thief!
            } else {
                msg(w, from, 501003); // You notice nothing unusual.
            }
        } else {
            msg(w, from, 501001); // You cannot determine anything useful.
        }
    } else if let Some(item) = entity_item(target) {
        if let Some(c) = ops(w).corpse_facts(w, item) {
            if check_target_skill(w, from, SkillName::Forensics, target, 0.0, 100.0) {
                match &c.forensicist {
                    Some(name) => {
                        // The forensicist ~1_NAME~ has already discovered that:
                        super::common::msg_args(w, from, 1042750, name);
                    }
                    None => {
                        let name = w
                            .mobile(from)
                            .and_then(|m| (m.ty.name)(w, from))
                            .unwrap_or_default();
                        ops(w).set_corpse_forensicist(w, item, name);
                    }
                }

                if c.is_human_body {
                    // This person was killed by ~1_KILLER_NAME~
                    super::common::msg_args(
                        w,
                        from,
                        1042751,
                        c.killer_name.as_deref().unwrap_or("no one"),
                    );
                }

                if !c.looter_names.is_empty() {
                    let n = c.looter_names.len();
                    let mut sb = String::new();
                    for (i, looter) in c.looter_names.iter().enumerate() {
                        if i == n - 1 {
                            // (the last index is tested first, so a single looter is written as ", and name")
                            sb.push_str(&format!(", and {looter}"));
                        } else if i > 0 {
                            sb.push_str(&format!(", {looter}"));
                        } else {
                            sb.push_str(looter);
                        }
                    }
                    // This body has been disturbed by ~1_PLAYER_NAMES~
                    super::common::msg_args(w, from, 1042752, &sb);
                } else {
                    msg(w, from, 501002); // The corpse has not be desecrated.
                }
            } else {
                msg(w, from, 501001); // You cannot determine anything useful.
            }
        } else if let Some(picker) = ops(w).lockpickable_picker(w, item) {
            match picker {
                // This lock was opened by ~1_PICKER_NAME~
                Some(name) => super::common::msg_args(w, from, 1042749, &name),
                None => msg(w, from, 501003), // You notice nothing unusual.
            }
        }
    }
}

/// The `Initialize()` of each: `SkillInfo.Table[skill].Callback = OnUse`.
pub fn initialize(w: &mut World) {
    set_callback(w, SkillName::Anatomy, anatomy_on_use);
    set_callback(w, SkillName::EvalInt, eval_int_on_use);
    set_callback(w, SkillName::ArmsLore, arms_lore_on_use);
    set_callback(w, SkillName::ItemID, item_id_on_use);
    set_callback(w, SkillName::TasteID, taste_id_on_use);
    set_callback(w, SkillName::Cartography, cartography_on_use);
    set_callback(w, SkillName::Forensics, forensics_on_use);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;

    #[test]
    fn using_a_skill_opens_a_target_and_returns_the_delay() {
        let mut w = World::new();
        let m = w.new_mobile(&MOBILE_BASE);
        assert_eq!(anatomy_on_use(&mut w, m), TimeSpan::from_secs(1));
        let t = w.mobile(m).unwrap().target.expect("a target was set");
        assert_eq!(w.targets[&t].range, 8);
        assert!(!w.targets[&t].allow_nonlocal);
        assert_eq!(arms_lore_on_use(&mut w, m), TimeSpan::from_secs(1));
        let t = w.mobile(m).unwrap().target.unwrap();
        assert_eq!(
            (w.targets[&t].range, w.targets[&t].allow_nonlocal),
            (2, true)
        );
        assert_eq!(forensics_on_use(&mut w, m), TimeSpan::from_secs(1));
        assert_eq!(w.targets[&w.mobile(m).unwrap().target.unwrap()].range, 10);
    }

    #[test]
    fn cartography_without_the_craft_menu_hints_at_the_pen() {
        let mut w = World::new();
        let m = w.new_mobile(&MOBILE_BASE);
        assert_eq!(cartography_on_use(&mut w, m), TimeSpan::ZERO);
    }
}
