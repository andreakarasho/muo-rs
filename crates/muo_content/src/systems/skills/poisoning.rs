//! The Poisoning skill.
//!
//! `BasePoisonPotion`, `FukiyaDarts` and `Shuriken` are items of other crates; the skill asks for them
//! through [`SkillItemFns::poison_potion`], [`SkillItemFns::poisonable_throwing`] and
//! [`SkillItemFns::poison_throwing`].

use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::items as mitems;
use muo_server::poison::{self, PoisonId};
use muo_server::skills::{self, SkillName};
use muo_server::target::Targeted;
use muo_server::timer;

use super::common::{TargetOptions, check_target_skill, msg, set_target};
use super::hooks::ops;
use super::skills_info::set_callback;
use crate::core;
use crate::items::food;
use crate::items::weapon::{self, Ability, WeaponType};
use crate::misc::titles;
use crate::mx;

fn entity_item(t: Targeted) -> Option<ItemId> {
    match t {
        Targeted::Entity(EntityId::Item(i)) => Some(i),
        _ => None,
    }
}

pub fn on_use(w: &mut World, m: MobileId) -> TimeSpan {
    set_target(w, m, 2, TargetOptions::default(), on_target_poison);

    msg(w, m, 502137); // Select the poison you wish to use

    TimeSpan::from_secs(10) // 10 second delay before being able to re-use a skill
}

fn on_target_poison(w: &mut World, from: MobileId, targeted: Targeted) {
    if let Some(potion) = entity_item(targeted).filter(|&i| ops(w).poison_potion(w, i).is_some()) {
        msg(w, from, 502142); // To what do you wish to apply the poison?
        set_target(
            w,
            from,
            2,
            TargetOptions::default(),
            move |w, from, targeted| on_target(w, from, targeted, potion),
        );
    } else {
        // Not a Poison Potion
        msg(w, from, 502139); // That is not a poison potion.
    }
}

fn is_free_consume(w: &World, m: MobileId) -> bool {
    w.resources
        .get::<crate::spells::hooks::SpellHooks>()
        .and_then(|h| h.duel_is_free_consume)
        .is_some_and(|f| f(w, m))
}

fn on_target(w: &mut World, from: MobileId, targeted: Targeted, potion: ItemId) {
    if muo_server::item::ops::is_deleted(w, potion) {
        return;
    }
    let Some(info) = ops(w).poison_potion(w, potion) else {
        return;
    };

    let mut start_timer = false;
    let item = entity_item(targeted);

    if let Some(item) = item {
        if food::is_food(w, item) || ops(w).poisonable_throwing(w, item).is_some() {
            start_timer = true;
        } else if weapon::is_weapon(w, item) {
            if core::aos(w) {
                let c = weapon::class(w, item);
                start_timer = c.primary_ability == Some(Ability::InfectiousStrike)
                    || c.secondary_ability == Some(Ability::InfectiousStrike);
            } else if w.item(item).is_some_and(|i| i.layer == Layer::OneHanded) {
                start_timer = matches!(
                    weapon::weapon_type(w, item),
                    WeaponType::Slashing | WeaponType::Piercing
                );
            }
        }
    }

    if start_timer {
        let item = item.expect("start_timer implies an item");
        timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
            apply_tick(w, from, item, info.poison, info.min_skill, info.max_skill)
        });

        mx::play_sound(w, from, 0x4F);

        if !is_free_consume(w, from) {
            muo_server::item::base::consume(w, potion, 1);
            if let Some(bottle) = crate::factory::make_item(w, "Server.Items.Bottle") {
                mitems::add_to_backpack(w, from, bottle);
            }
        }
    } else {
        // Target can't be poisoned
        if core::aos(w) {
            // You cannot poison that! You can only poison infectious weapons, food or drink.
            msg(w, from, 1060204);
        } else {
            // You cannot poison that! You can only poison bladed or piercing weapons, food or drink.
            msg(w, from, 502145);
        }
    }
}

fn apply_tick(w: &mut World, from: MobileId, target: ItemId, p: PoisonId, min: f64, max: f64) {
    let t = Targeted::Entity(EntityId::Item(target));
    if check_target_skill(w, from, SkillName::Poisoning, t, min, max) {
        let level = poison::level(w, p);
        if food::is_food(w, target) {
            if let Some(d) = food::data_mut(w, target) {
                d.poison = Some(p);
            }
            muo_server::item::ops::invalidate_properties(w, target);
        } else if weapon::is_weapon(w, target) {
            if let Some(d) = weapon::data_mut(w, target) {
                d.poison = Some(p);
            }
            muo_server::item::ops::invalidate_properties(w, target);
            weapon::set_poison_charges(w, target, 18 - level * 2);
        } else if let Some(uses) = ops(w).poisonable_throwing(w, target) {
            ops(w).poison_throwing(w, target, p, (18 - level * 2).min(uses));
        }

        msg(w, from, 1010517); // You apply the poison

        titles::award_karma(w, from, -20, true);
    } else {
        // Failed
        // 5% of chance of getting poisoned if failed
        if skills::base(w, from, SkillName::Poisoning) < 80.0
            && muo_primitives::utility::random(&mut w.rng, 20) == 0
        {
            msg(w, from, 502148); // You make a grave mistake while applying the poison.
            mb::apply_poison(w, from, Some(from), p);
        } else if weapon::is_weapon(w, target) {
            if weapon::weapon_type(w, target) == WeaponType::Slashing {
                // You fail to apply a sufficient dose of poison on the blade
                msg(w, from, 1010516);
            } else {
                msg(w, from, 1010518); // You fail to apply a sufficient dose of poison
            }
        } else {
            msg(w, from, 1010518); // You fail to apply a sufficient dose of poison
        }
    }
}

pub fn initialize(w: &mut World) {
    set_callback(w, SkillName::Poisoning, on_use);
}
