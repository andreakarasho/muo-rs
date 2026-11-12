//! The code of the abstract weapon classes below BaseWeapon: BaseSword, BaseKnife, BaseBashing, BaseSpear,
//! BaseStaff, BaseAxe, BasePoleArm (OnDoubleClick, OnHit, GetBaseDamage), BaseRanged (OnSwing, OnFired, OnHit,
//! OnMiss, OnCraft) and BaseThrown (hit chance, damage, return), and `BaseWeapon.OnCraft`.
//! Their constants are in the `*_CLASS` statics of `weapon.rs`; this file holds the behavior those tables point at.

use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::mobile::StatType;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::combat::in_los_point;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects as fx, messages as msgs};
use muo_server::mods::StatMod;
use muo_server::skills::SkillName;

use super::equip_hooks::CraftArgs;
use super::weapon::{self, Ability, WeaponQuality, data, data_mut};
use super::weapon_ability as wa;
use super::weapon_combat as wc;
use super::weapon_hooks as hk;
use crate::aos::{AosAttribute, AosElementAttribute, AosWeaponAttribute, AttrKind};
use crate::core;
use crate::craft::CraftResource;

fn msg(w: &mut World, m: MobileId, n: i32) {
    muo_server::item::deps::send_localized_message(w, m, n);
}

fn val(w: &mut World, m: MobileId, s: SkillName) -> f64 {
    muo_server::skills::value(w, m, s)
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| x.player)
}

fn body_is_human(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|_| {
        w.resources
            .get::<muo_primitives::BodyTable>()
            .is_some_and(|t| t.is_human(muo_server::mobile::part1::props_b::body(w, m)))
    })
}

fn layer_of(w: &World, id: ItemId) -> Layer {
    w.item(id).map_or(Layer::Invalid, |i| i.layer)
}

// ---------------------------------------------------------------------------------------------------------------
// BaseWeapon.OnCraft
// ---------------------------------------------------------------------------------------------------------------

/// `OnCraft(quality, makersMark, from, craftSystem, typeRes, tool, craftItem, resHue)` of BaseWeapon.
pub fn on_craft_base(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    weapon::set_quality(
        w,
        id,
        WeaponQuality::from_i32(a.quality).unwrap_or(WeaponQuality::Regular),
    );

    if a.makers_mark {
        let name = props_b::raw_name(w, a.from);
        if let Some(d) = data_mut(w, id) {
            d.crafter = name;
        }
        muo_server::item::ops::invalidate_properties(w, id);
    }

    if let Some(d) = data_mut(w, id) {
        d.identified = true;
    }
    muo_server::item::ops::invalidate_properties(w, id);

    let set_hue = |w: &mut World, hue: i32| {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.set_hue)(w, id, hue);
        }
    };

    if core::aos(w) {
        weapon::set_resource(w, id, a.resource);

        if a.do_not_color {
            set_hue(w, 0);
        }

        if let Some(tool) = a.runic_tool
            && let Some(f) = super::equip_hooks::hooks(w).and_then(|h| h.runic_apply_attributes)
        {
            f(w, tool, id);
        }

        if data(w, id).is_some_and(|d| d.quality == WeaponQuality::Exceptional) {
            super::set_attr(
                w,
                id,
                AttrKind::Attributes,
                AosAttribute::WeaponDamage as u32,
                35,
            );

            if core::ml(w) {
                let arms_lore = val(w, a.from, SkillName::ArmsLore);
                let mut v = 35 + (arms_lore / 20.0) as i32;
                if v > 50 {
                    v = 50;
                }
                super::set_attr(
                    w,
                    id,
                    AttrKind::Attributes,
                    AosAttribute::WeaponDamage as u32,
                    v,
                );

                muo_server::mobile::part2::stats::check_skill(
                    w,
                    a.from,
                    SkillName::ArmsLore,
                    0.0,
                    100.0,
                );
            }
        }
    } else if let Some(tool) = a.runic_tool {
        let this_resource = a.resource;

        if this_resource == hk::runic_tool_resource(w, tool) {
            weapon::set_resource(w, id, this_resource);

            if a.do_not_color {
                set_hue(w, 0);
            }

            use super::weapon::{
                WeaponAccuracyLevel as Acc, WeaponDamageLevel as Dmg, WeaponDurabilityLevel as Dur,
            };
            let (dur, dmg, acc) = match this_resource {
                CraftResource::DullCopper => (Some(Dur::Durable), None, Some(Acc::Accurate)),
                CraftResource::ShadowIron => (Some(Dur::Durable), Some(Dmg::Ruin), None),
                CraftResource::Copper => (
                    Some(Dur::Fortified),
                    Some(Dmg::Ruin),
                    Some(Acc::Surpassingly),
                ),
                CraftResource::Bronze => (
                    Some(Dur::Fortified),
                    Some(Dmg::Might),
                    Some(Acc::Surpassingly),
                ),
                CraftResource::Gold => (
                    Some(Dur::Indestructible),
                    Some(Dmg::Force),
                    Some(Acc::Eminently),
                ),
                CraftResource::Agapite => (
                    Some(Dur::Indestructible),
                    Some(Dmg::Power),
                    Some(Acc::Eminently),
                ),
                CraftResource::Verite => (
                    Some(Dur::Indestructible),
                    Some(Dmg::Power),
                    Some(Acc::Exceedingly),
                ),
                CraftResource::Valorite => (
                    Some(Dur::Indestructible),
                    Some(Dmg::Vanq),
                    Some(Acc::Supremely),
                ),
                _ => (None, None, None),
            };
            if let Some(d) = dur {
                weapon::set_durability_level(w, id, d);
            }
            if let Some(d) = dmg {
                weapon::set_damage_level(w, id, d);
            }
            if let Some(a) = acc {
                weapon::set_accuracy_level(w, id, a);
            }
        }
    }

    a.quality
}

/// `BaseRanged.OnCraft`: the base, then (ML) the wood bonuses.
pub fn ranged_on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    let result = on_craft_base(w, id, a);
    if core::ml(w) {
        apply_wood_bonuses(w, id, a.resource);
    }
    result
}

fn add_attr(w: &mut World, id: ItemId, kind: AttrKind, mask: u32, delta: i32) {
    let v = super::attr(w, id, kind, mask);
    super::set_attr(w, id, kind, mask, v + delta);
}

fn apply_wood_bonuses(w: &mut World, id: ItemId, resource: CraftResource) {
    use AosAttribute as A;
    use AosWeaponAttribute as W;
    match resource {
        CraftResource::OakWood => {
            add_attr(w, id, AttrKind::Attributes, A::Luck as u32, 40);
            add_attr(w, id, AttrKind::Attributes, A::WeaponDamage as u32, 5);
        }
        CraftResource::AshWood => {
            add_attr(w, id, AttrKind::Attributes, A::WeaponSpeed as u32, 10);
            add_attr(w, id, AttrKind::Weapon, W::LowerStatReq as u32, 20);
        }
        CraftResource::YewWood => {
            add_attr(w, id, AttrKind::Attributes, A::AttackChance as u32, 5);
            add_attr(w, id, AttrKind::Attributes, A::WeaponDamage as u32, 10);
        }
        CraftResource::Bloodwood => {
            add_attr(w, id, AttrKind::Attributes, A::RegenHits as u32, 2);
            add_attr(w, id, AttrKind::Weapon, W::HitLeechHits as u32, 16);
        }
        CraftResource::Heartwood => match utility::random(&mut w.rng, 6) {
            0 => add_attr(w, id, AttrKind::Attributes, A::Luck as u32, 40),
            1 => add_attr(w, id, AttrKind::Weapon, W::DurabilityBonus as u32, 50),
            2 => add_attr(w, id, AttrKind::Weapon, W::LowerStatReq as u32, 20),
            3 => add_attr(w, id, AttrKind::Attributes, A::WeaponSpeed as u32, 10),
            4 => add_attr(w, id, AttrKind::Attributes, A::AttackChance as u32, 5),
            _ => add_attr(w, id, AttrKind::Weapon, W::HitLeechHits as u32, 10),
        },
        CraftResource::Frostwood => {
            super::set_attr(
                w,
                id,
                AttrKind::Element,
                AosElementAttribute::Physical as u32,
                60,
            );
            super::set_attr(
                w,
                id,
                AttrKind::Element,
                AosElementAttribute::Cold as u32,
                40,
            );
            add_attr(w, id, AttrKind::Attributes, A::WeaponDamage as u32, 12);
        }
        _ => {}
    }
}

/// `BaseMeleeWeapon.AbsorbDamage`.
pub fn melee_absorb_damage(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    damage: i32,
) -> i32 {
    let mut damage = wc::absorb_damage_base(w, id, attacker, defender, damage);

    damage = hk::attune_weapon_try_absorb(w, defender, damage);

    if core::aos(w) {
        return damage;
    }

    hk::reactive_armor_handle_melee_hit(w, attacker, defender, damage)
}

// ---------------------------------------------------------------------------------------------------------------
// BaseSword / BaseKnife / BaseSpear: poison charges; BaseBashing / BaseStaff: stamina loss
// ---------------------------------------------------------------------------------------------------------------

/// `!Core.AOS && Poison != null && PoisonCharges > 0 { --PoisonCharges; 50%: defender.ApplyPoison }`.
fn poison_charges_on_hit(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) {
    if core::aos(w) {
        return;
    }
    let Some((poison, charges)) = data(w, id).map(|d| (d.poison, d.poison_charges)) else {
        return;
    };
    if let Some(p) = poison
        && charges > 0
    {
        weapon::set_poison_charges(w, id, charges - 1);
        if utility::random_bool(&mut w.rng) {
            // 50% chance to poison
            mb::apply_poison(w, defender, Some(attacker), p);
        }
    }
}

/// `BaseSword.OnHit` / `BaseKnife.OnHit`.
pub fn blade_on_hit(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId, bonus: f64) {
    wc::on_hit_base(w, id, attacker, defender, bonus);
    poison_charges_on_hit(w, id, attacker, defender);
}

/// `BaseSword.OnDoubleClick` / `BaseKnife.OnDoubleClick`.
pub fn blade_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    msg(w, from, 1010018); // What do you want to use this item on?
    crate::systems::targets::bladed_item_target(w, from, id);
}

fn stamina_loss(w: &mut World, defender: MobileId) {
    let loss = utility::random_from(&mut w.rng, 3, 3); // 3-5 points of stamina loss
    let s = w.mobile(defender).map_or(0, |m| m.stam);
    props_b::set_stam(w, defender, s - loss);
}

/// `BaseBashing.OnHit` / `BaseStaff.OnHit`.
pub fn bashing_on_hit(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    bonus: f64,
) {
    wc::on_hit_base(w, id, attacker, defender, bonus);
    stamina_loss(w, defender);
}

/// `BaseBashing.GetBaseDamage`.
pub fn bashing_base_damage(w: &mut World, id: ItemId, attacker: MobileId) -> f64 {
    let mut damage = wc::get_base_damage_base(w, id, attacker);
    if !core::aos(w)
        && super::core_uor(w)
        && (is_player(w, attacker) || body_is_human(w, attacker))
        && layer_of(w, id) == Layer::TwoHanded
        && val(w, attacker, SkillName::Anatomy) >= 80.0
        && val(w, attacker, SkillName::Anatomy) / 400.0 >= utility::random_double(&mut w.rng)
        && hk::duel_allow_special_ability(w, attacker, "Crushing Blow", false)
    {
        damage *= 1.5;
        msg(w, attacker, 1060090); // You have delivered a crushing blow!
        fx::play_sound(w, attacker, 0x11C);
    }
    damage
}

/// `BaseSpear.OnHit`.
pub fn spear_on_hit(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId, bonus: f64) {
    wc::on_hit_base(w, id, attacker, defender, bonus);

    if !core::aos(w)
        && super::core_uor(w)
        && layer_of(w, id) == Layer::TwoHanded
        && val(w, attacker, SkillName::Anatomy) / 400.0 >= utility::random_double(&mut w.rng)
        && hk::duel_allow_special_ability(w, attacker, "Paralyzing Blow", false)
    {
        msg(w, defender, 1072221); // You have been hit by a paralyzing blow!
        mb::freeze(w, defender, TimeSpan::from_secs(2));
        msg(w, attacker, 1060163); // You deliver a paralyzing blow!
        fx::play_sound(w, attacker, 0x11C);
    }

    poison_charges_on_hit(w, id, attacker, defender);
}

/// The concussion blow tail of `BaseAxe.OnHit` and `BasePoleArm.OnHit`.
fn concussion_blow(w: &mut World, attacker: MobileId, defender: MobileId) {
    let has_mod = w.mobile(defender).is_some_and(|m| {
        m.stat_mods
            .iter()
            .any(|s| s.name.as_deref() == Some("Concussion") && !s.has_elapsed(w.now))
    });
    if has_mod {
        return;
    }
    msgs::send_message(w, defender, "You receive a concussion blow!");
    let raw_int = w.mobile(defender).map_or(0, |m| m.int);
    let sm = StatMod::new(
        w,
        StatType::INT,
        Some("Concussion"),
        -(raw_int / 2),
        TimeSpan::from_secs(30),
        None,
    );
    mb::add_stat_mod(w, defender, sm);
    msgs::send_message(w, attacker, "You deliver a concussion blow!");
    fx::play_sound(w, attacker, 0x308);
}

/// `BaseAxe.OnHit`.
pub fn axe_on_hit(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId, bonus: f64) {
    wc::on_hit_base(w, id, attacker, defender, bonus);
    if !core::aos(w)
        && super::core_uor(w)
        && (is_player(w, attacker) || body_is_human(w, attacker))
        && layer_of(w, id) == Layer::TwoHanded
        && val(w, attacker, SkillName::Anatomy) >= 80.0
        && val(w, attacker, SkillName::Anatomy) / 400.0 >= utility::random_double(&mut w.rng)
        && hk::duel_allow_special_ability(w, attacker, "Concussion Blow", false)
    {
        concussion_blow(w, attacker, defender);
    }
}

/// `BasePoleArm.OnHit` (no `Core.UOR` test).
pub fn pole_arm_on_hit(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    bonus: f64,
) {
    wc::on_hit_base(w, id, attacker, defender, bonus);
    if !core::aos(w)
        && (is_player(w, attacker) || body_is_human(w, attacker))
        && layer_of(w, id) == Layer::TwoHanded
        && val(w, attacker, SkillName::Anatomy) >= 80.0
        && val(w, attacker, SkillName::Anatomy) / 400.0 >= utility::random_double(&mut w.rng)
        && hk::duel_allow_special_ability(w, attacker, "Concussion Blow", false)
    {
        concussion_blow(w, attacker, defender);
    }
}

/// `BaseAxe.OnDoubleClick`.
pub fn axe_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if w.item(id).is_none_or(|i| i.deleted()) {
        return;
    }

    if !muo_server::item::ops::is_child_of(w, id, Some(EntityId::Mobile(from))) {
        let loc = muo_server::item::ops::get_world_location(w, id);
        if !in_los_point(w, from, loc) || !mb::in_range_point3d(w, from, loc, 2) {
            msgs::local_overhead_message_number(
                w,
                from,
                muo_primitives::MessageType::REGULAR,
                0x3E9,
                1019045,
                "",
            ); // I can't reach that
            return;
        }
    }

    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    if !(ty.is_accessible_to)(w, id, from) {
        muo_server::item::messages::public_overhead_message_number(
            w,
            id,
            muo_server::packets::message::MessageType::REGULAR,
            0x3E9,
            1061637,
            "",
        ); // You are not allowed to access this.
        return;
    }

    if !hk::harvest_is_mining(w, id) {
        msg(w, from, 1010018); // What do you want to use this item on?
    }
    hk::harvest_begin(w, from, id);
}

/// `BasePoleArm.OnDoubleClick`.
pub fn pole_arm_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let in_pack = props_b::backpack(w, from)
        .is_some_and(|p| muo_server::item::ops::is_child_of(w, id, Some(EntityId::Item(p))))
        || w.item(id)
            .is_some_and(|i| i.parent == Some(EntityId::Mobile(from)));
    if in_pack {
        hk::harvest_begin(w, from, id);
    } else {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
    }
}

// ---------------------------------------------------------------------------------------------------------------
// BaseRanged
// ---------------------------------------------------------------------------------------------------------------

fn item_type(w: &World, full_name: &str) -> Option<&'static muo_server::ItemType> {
    match w.types.find_type_by_full_name(full_name, false)? {
        muo_server::type_registry::EntityType::Item(t) => Some(t),
        _ => None,
    }
}

fn shoot_particles(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) {
    let effect = weapon::effect_id(w, id);
    fx::moving_particles_layer(
        w,
        attacker,
        EntityId::Mobile(defender),
        effect,
        18,
        1,
        false,
        false,
        0,
        0,
        0,
        0,
        0,
        EffectLayer::RightHand,
        0,
    );
}

/// `BaseRanged.OnSwing`.
pub fn ranged_on_swing(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    _bonus: f64,
) -> TimeSpan {
    let last_move = w.mobile(attacker).map_or(0, |m| m.last_move_time);
    let limit = if core::se(w) {
        250
    } else if core::aos(w) {
        500
    } else {
        1000
    };
    if w.tick_count.wrapping_sub(last_move) >= limit
        || (core::aos(w) && wa::current_ability(w, attacker) == Some(Ability::MovingShot))
    {
        let mut can_swing = true;

        if core::aos(w) {
            can_swing =
                !mb::paralyzed(w, attacker) && !w.mobile(attacker).is_some_and(|m| m.frozen);
            if can_swing {
                can_swing = !hk::spell_blocks_movement(w, attacker);
            }
        }

        if w.mobile(attacker)
            .is_some_and(|m| m.ty.is(&crate::player_mobile::PLAYER_MOBILE))
            && !hk::duel_check_item_equip(w, attacker, id)
        {
            can_swing = false;
        }

        if can_swing && mb::harmful_check(w, attacker, defender) {
            mb::disruptive_action(w, attacker);

            if let (Some(ns), Some(a), Some(d)) = (
                w.mobile(attacker).and_then(|m| m.net_state),
                w.mobile(attacker).map(|m| m.serial),
                w.mobile(defender).map(|m| m.serial),
            ) {
                muo_server::packets::combat::send_swing(w, ns, a, d);
            }

            if crate::creature::data::is_creature(w, attacker)
                && crate::abilities::trigger_ability(
                    w,
                    attacker,
                    crate::abilities::MonsterAbilityTrigger::COMBAT_ACTION,
                    Some(defender),
                )
            {
                return wc::get_delay(w, id, attacker);
            }

            if (weapon::class(w, id).on_fired)(w, id, attacker, defender) {
                if wc::check_hit(w, id, attacker, defender) {
                    wc::on_hit(w, id, attacker, defender, 1.0);
                } else {
                    wc::on_miss(w, id, attacker, defender);
                }
            } else if wa::current_ability(w, attacker) == Some(Ability::LightningArrow) {
                shoot_particles(w, id, attacker, defender);
                let at = (
                    w.mobile(defender).map_or(Point3D::ZERO, |m| m.location),
                    w.mobile(defender).and_then(|m| m.map),
                );
                wa::on_hit(w, Ability::LightningArrow, attacker, defender, 0, at);
            }
        }

        mb::revealing_action(w, attacker);

        return wc::get_delay(w, id, attacker);
    }

    mb::revealing_action(w, attacker);

    TimeSpan::from_secs_f64(0.25)
}

/// `BaseRanged.OnFired`.
pub fn ranged_on_fired(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) -> bool {
    if is_player(w, attacker) {
        let ammo_name = weapon::class(w, id).ammo_type;
        let ammo_type = ammo_name.and_then(|n| item_type(w, n));
        let quiver = hk::quiver_of(w, attacker);
        let pack = props_b::backpack(w, attacker);

        let lower = quiver.map_or(0, |q| hk::quiver_lower_ammo_cost(w, q));
        if quiver.is_none() || utility::random(&mut w.rng, 100) >= lower {
            let Some(t) = ammo_type else { return false };
            if quiver
                .is_some_and(|q| muo_server::item::container::consume_total(w, q, t, 1, true, None))
            {
                hk::quiver_invalidate_weight(w, quiver.unwrap_or(id));
            } else if !pack
                .is_some_and(|p| muo_server::item::container::consume_total(w, p, t, 1, true, None))
            {
                return false;
            }
        } else {
            let Some(t) = ammo_type else { return false };
            let in_quiver =
                quiver.and_then(|q| muo_server::item::container::find_item_by_type(w, q, t, true));
            let in_pack =
                pack.and_then(|p| muo_server::item::container::find_item_by_type(w, p, t, true));
            if in_quiver.is_none() && in_pack.is_none() {
                return false;
            }
        }
    }

    shoot_particles(w, id, attacker, defender);
    true
}

/// `BaseRanged.Ammo`: a new ammunition item of the class's type.
fn new_ammo(w: &mut World, id: ItemId) -> Option<ItemId> {
    let name = weapon::class(w, id).ammo_type?;
    crate::factory::make_item(w, name)
}

/// `BaseRanged.OnHit`.
pub fn ranged_on_hit(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    bonus: f64,
) {
    let animal_or_monster = w.mobile(defender).is_some_and(|_| {
        w.resources
            .get::<muo_primitives::BodyTable>()
            .is_some_and(|t| {
                let b = muo_server::mobile::part1::props_b::body(w, defender);
                t.is_animal(b) || t.is_monster(b)
            })
    });
    if is_player(w, attacker)
        && !is_player(w, defender)
        && animal_or_monster
        && utility::random_double(&mut w.rng) < 0.4
        && let Some(ammo) = new_ammo(w, id)
    {
        muo_server::item::deps::add_to_backpack(w, defender, ammo);
        // `!defender.AddToBackpack(ammo)` deletes the ammo when the add fails: AddToBackpack here always places it.
    }

    let velocity = data(w, id).map_or(0, |d| d.velocity);
    if core::ml(w) && velocity > 0 {
        let (a, d) = (
            w.mobile(attacker).map_or(Point3D::ZERO, |m| m.location),
            w.mobile(defender).map_or(Point3D::ZERO, |m| m.location),
        );
        let (dx, dy) = (f64::from(a.x - d.x), f64::from(a.y - d.y));
        let bonus_d = (dx * dx + dy * dy).sqrt() as i32;
        if bonus_d > 0 && velocity > utility::random(&mut w.rng, 100) {
            crate::misc::aos_helpers::damage(
                w,
                defender,
                Some(attacker),
                bonus_d * 3,
                crate::misc::aos_helpers::DamageSplit {
                    phys: 100,
                    ..Default::default()
                },
                Default::default(),
            );
            if is_player(w, attacker) {
                msg(w, attacker, 1072794); // Your arrow hits its mark with velocity!
            }
            if is_player(w, defender) {
                msg(w, defender, 1072795); // You have been hit by an arrow with velocity!
            }
        }
    }

    wc::on_hit_base(w, id, attacker, defender, bonus);
}

/// `BaseRanged.OnMiss`.
pub fn ranged_on_miss(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) {
    if is_player(w, attacker) && utility::random_double(&mut w.rng) < 0.4 {
        if core::se(w) {
            if w.mobile(attacker)
                .is_some_and(|m| m.ty.is(&crate::player_mobile::PLAYER_MOBILE))
                && let Some(n) = weapon::class(w, id).ammo_type
            {
                hk::ammo_recovery_bank(w, attacker, n);
            }
        } else if let Some(ammo) = new_ammo(w, id) {
            let (loc, map) = w
                .mobile(defender)
                .map_or((Point3D::ZERO, None), |m| (m.location, m.map));
            let dx = super::random_min_max(w, -1, 1);
            let dy = super::random_min_max(w, -1, 1);
            muo_server::item::ops::move_to_world_map(
                w,
                ammo,
                Point3D::new(loc.x + dx, loc.y + dy, loc.z),
                map,
            );
        }
    }
    wc::on_miss_base(w, id, attacker, defender);
}

// ---------------------------------------------------------------------------------------------------------------
// BaseThrown
// ---------------------------------------------------------------------------------------------------------------

/// `BaseThrown.ModifyHitChance`.
pub fn thrown_modify_hit_chance(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    mut chance: f64,
) -> f64 {
    let (a, d) = (
        w.mobile(attacker).map_or(Point3D::ZERO, |m| m.location),
        w.mobile(defender).map_or(Point3D::ZERO, |m| m.location),
    );
    let distance = (a.x - d.x).abs().max((a.y - d.y).abs());
    let min_range = weapon::class(w, id).min_throw_range;

    if distance <= 1 {
        let throw_skill = val(w, attacker, SkillName::Throwing);
        let raw_dex = w.mobile(attacker).map_or(0, |m| m.dex);
        let mitigation = ((throw_skill + f64::from(raw_dex)) / 20.0).min(12.0);
        chance -= (12.0 - mitigation) / 100.0;
    } else if distance < min_range {
        chance -= 0.12;
    }

    let has_shield =
        muo_server::mobile::part2::items::find_item_on_layer(w, attacker, Layer::TwoHanded)
            .and_then(|i| w.item(i))
            .is_some_and(|i| i.ty.is(&super::armor::BASE_SHIELD));
    if has_shield {
        let parry = val(w, attacker, SkillName::Parry);
        let penalty = if parry > 0.0 { 1200.0 / parry } else { 90.0 };
        chance *= 1.0 - penalty.min(90.0) / 100.0;
    }

    chance
}

/// `BaseThrown.ModifyDamage`.
pub fn thrown_modify_damage(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    mut damage: i32,
) -> i32 {
    let range = weapon::max_range(w, id) - 1;
    let dloc = w.mobile(defender).map_or(Point3D::ZERO, |m| m.location);
    if !mb::in_range_point3d(w, attacker, dloc, range) {
        damage = damage * 53 / 100;
    }
    damage
}

/// `BaseThrown.OnFired`.
pub fn thrown_on_fired(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) -> bool {
    let dloc = w.mobile(defender).map_or(Point3D::ZERO, |m| m.location);
    if !mb::in_range_point3d(w, attacker, dloc, 1) {
        let (graphic, hue) = w.item(id).map_or((0, 0), |i| (i.item_id, i.hue));
        fx::moving_effect(
            w,
            attacker,
            EntityId::Mobile(defender),
            graphic,
            18,
            1,
            false,
            false,
            hue,
            0,
        );
    }
    true
}

fn schedule_return(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) {
    if wa::current_ability(w, attacker) != Some(Ability::MysticArc) {
        let loc = (
            w.mobile(defender).map_or(Point3D::ZERO, |m| m.location),
            w.mobile(attacker).and_then(|m| m.map),
        );
        muo_server::timer::start(w, TimeSpan::from_millis(300), TimeSpan::ZERO, 1, move |w| {
            thrown_return(w, id, attacker, defender, loc);
        });
    }
}

/// `BaseThrown.OnHit`.
pub fn thrown_on_hit(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    bonus: f64,
) {
    schedule_return(w, id, attacker, defender);
    ranged_on_hit(w, id, attacker, defender, bonus);
}

/// `BaseThrown.OnMiss`.
pub fn thrown_on_miss(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) {
    schedule_return(w, id, attacker, defender);
    ranged_on_miss(w, id, attacker, defender);
}

/// `BaseThrown.Return(thrower, target, worldLocation)`.
pub fn thrown_return(
    w: &mut World,
    id: ItemId,
    thrower: MobileId,
    target: MobileId,
    at: (Point3D, Option<muo_server::map::MapId>),
) {
    let Some(t) = w.mobile(thrower) else { return };
    if t.deleted || t.map.is_none() || t.map == Some(muo_server::map::MapId::INTERNAL) {
        return;
    }
    let (graphic, hue) = w.item(id).map_or((0, 0), |i| (i.item_id, i.hue));
    if w.mobile(target).is_some_and(|m| !m.deleted) {
        fx::moving_effect(
            w,
            target,
            EntityId::Mobile(thrower),
            graphic,
            18,
            1,
            false,
            false,
            hue,
            0,
        );
    } else {
        muo_server::effects::send_moving_particles_layer(
            w,
            muo_server::effects::Targeted::point(at.0, at.1),
            EntityId::Mobile(thrower),
            graphic,
            18,
            0,
            false,
            false,
            hue,
            0,
            9502,
            1,
            0,
            muo_server::effects::EffectLayer::None.0.into(),
            0x100,
        );
    }
}

/// `BaseThrown.GetProperties`.
pub fn thrown_get_properties(
    w: &World,
    id: ItemId,
    list: &mut muo_server::property_list::PropertyList,
) {
    super::weapon_text::get_properties(w, id, list);
    list.add_int(1149791, weapon::class(w, id).min_throw_range); // Min Throw Range: ~1_val~
}
