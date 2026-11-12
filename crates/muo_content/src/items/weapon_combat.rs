//! The combat half of BaseWeapon: `OnSwing`, `CheckHit`, `OnHit`/`OnMiss`, the damage and
//! delay formulas, parry and armor absorption, the elemental on-hit effects, slayers and the animations.
//!
//! The `virtual` members that a weapon class may override are fn pointers in [`WeaponClass`]; the `*_base`
//! functions here are the base bodies (a leaf's `base.OnHit(..)` is `on_hit_base`), and the functions without
//! suffix dispatch through the class table, like a virtual call. Everything the weapon asks of systems that live
//! elsewhere is a hook in [`super::weapon_hooks`].

use muo_primitives::{Direction, MessageType, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::ops;
use muo_server::mobile::MobileDelta;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::combat::in_los;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects as fx, messages as msgs};
use muo_server::skills::{self, SkillName};

use super::slayer::{self, SlayerName};
use super::weapon::{
    self, Ability, WeaponAnimation, WeaponDamageLevel, WeaponQuality, WeaponType, data, data_mut,
};
use super::weapon_ability as wa;
use super::weapon_hooks as hk;
use crate::aos::{AosAttribute, AosElementAttribute, AosWeaponAttribute, AttrKind};
use crate::core;
use crate::craft;
use crate::creature::data::{cd, is_creature};
use crate::misc::aos_helpers::{self, DamageOptions, DamageSplit};
use crate::player_mobile::{self, PLAYER_MOBILE};

/// `CheckSlayerResult`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CheckSlayerResult {
    None,
    Slayer,
    Opposition,
}

/// The seven percentages `GetDamageTypes` hands out.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct DamageTypes {
    pub phys: i32,
    pub fire: i32,
    pub cold: i32,
    pub pois: i32,
    pub nrgy: i32,
    pub chaos: i32,
    pub direct: i32,
}

/// `BaseWeapon._enableInstaHit` (`melee.enableInstaHit`, default `!Core.UOR`), set by [`configure`].
#[derive(Default)]
pub struct WeaponSettings {
    pub enable_insta_hit: bool,
}

/// `BaseWeapon.InDoubleStrike`.
#[derive(Default)]
pub struct DoubleStrikeFlag(pub bool);

/// `BaseWeapon.Configure()`; also hands `AosAttributes.GetValue` the sum over worn weapons, armor, jewels and
/// clothing.
// (spellbook, quiver and talisman bags are read by `super::attr` like the others)
pub fn configure(w: &mut World) {
    let v = w
        .config
        .get_setting("melee.enableInstaHit", !super::core_uor(w));
    w.resources.get_mut::<WeaponSettings>().enable_insta_hit = v;
    w.resources
        .get_mut::<aos_helpers::AosHooks>()
        .attribute_value = Some(equipped_attribute_value);
    // `item is IWeapon` for `Mobile.Weapon`.
    w.resources
        .get_mut::<muo_server::mobile::part1::props_b::PropsBStatics>()
        .item_is_weapon = Some(weapon::is_weapon);
    w.resources
        .get_mut::<muo_server::mobile::part1::props_b::PropsBStatics>()
        .combat_swing = Some(combat_swing);
    w.resources
        .get_mut::<muo_server::mobile::part1::props_b::PropsBStatics>()
        .weapon_status_damage = Some(|w, m| match wielded(w, m) {
        Some(wp) => get_status_damage(w, wp, m),
        None => (0, 0),
    });
}

/// The swing at the end of `Mobile.CheckCombatTime`, for the combat timer to call once the
/// combatant checks above it passed: `weapon == null || !InRange(combatant, weapon.MaxRange)` returns, else
/// `OnBeforeSwing`, `RevealingAction()` and `NextCombatTime = TickCount + (int)OnSwing(..).TotalMilliseconds`.
// `configure` sets `PropsBStatics::combat_swing` to this. A mobile with nothing wielded swings the `Fists` singleton (`hooks::fists_item`).
pub fn combat_swing(w: &mut World, m: MobileId, combatant: MobileId) {
    let Some(wp) = wielded(w, m) else { return };
    let range = weapon::max_range(w, wp);
    if !mb::in_range_point3d(w, m, point(w, combatant), range) {
        return;
    }
    if in_los(w, m, combatant) {
        on_before_swing(w, wp, m, combatant);
        mb::revealing_action(w, m);
        let ms = swing(w, wp, m, combatant, 1.0).total_milliseconds() as i32;
        let tick = w.tick_count;
        if let Some(mob) = w.mobile_mut(m) {
            mob.next_combat_time = tick + i64::from(ms);
        }
    }
}

/// `AosAttributes.GetValue(m, attribute)` without the `Core.AOS` check, over the item families of this crate.
pub fn equipped_attribute_value(w: &World, m: MobileId, attribute: AosAttribute) -> i32 {
    let Some(mob) = w.mobile(m) else { return 0 };
    let mut value = 0;
    for &item in &mob.items {
        value += super::attr(w, item, AttrKind::Attributes, attribute as u32);
        if attribute == AosAttribute::Luck && weapon::is_weapon(w, item) {
            value += luck_bonus(w, item);
        }
        if attribute == AosAttribute::Luck && super::armor::is_armor(w, item) {
            value += super::armor::get_luck_bonus(w, item);
        }
    }
    value
}

type ResistFn = fn(&mut World, MobileId) -> i32;

fn enable_insta_hit(w: &World) -> bool {
    w.resources
        .get::<WeaponSettings>()
        .map_or(!super::core_uor(w), |s| s.enable_insta_hit)
}

fn aos_attr(w: &World, m: MobileId, a: AosAttribute) -> i32 {
    aos_helpers::attribute_value(w, m, a)
}

fn weapon_attr(w: &World, id: ItemId, a: AosWeaponAttribute) -> i32 {
    super::attr(w, id, AttrKind::Weapon, a as u32)
}

fn attacker_weapon_attr(w: &World, m: MobileId, a: AosWeaponAttribute) -> i32 {
    // `AosWeaponAttributes.GetValue(m, attribute)`: the sum over the worn items' weapon bags.
    if !core::aos(w) {
        return 0;
    }
    let Some(mob) = w.mobile(m) else { return 0 };
    mob.items
        .iter()
        .map(|&i| super::attr(w, i, AttrKind::Weapon, a as u32))
        .sum()
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| x.player)
}

fn is_player_mobile(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| x.ty.is(&PLAYER_MOBILE))
}

fn body_is_human(w: &World, m: MobileId) -> bool {
    if w.mobile(m).is_none() {
        return false;
    }
    w.resources
        .get::<muo_primitives::BodyTable>()
        .is_some_and(|t| t.is_human(muo_server::mobile::part1::props_b::body(w, m)))
}

fn mobile_is_named(w: &World, m: MobileId, full_name: &str) -> bool {
    let mut t = w.mobile(m).map(|x| x.ty);
    while let Some(c) = t {
        if c.full_name == full_name {
            return true;
        }
        t = c.parent;
    }
    false
}

fn point(w: &World, m: MobileId) -> Point3D {
    w.mobile(m).map_or(Point3D::ZERO, |x| x.location)
}

fn msg(w: &mut World, m: MobileId, number: i32) {
    muo_server::item::deps::send_localized_message(w, m, number);
}

fn val(w: &mut World, m: MobileId, s: SkillName) -> f64 {
    skills::value(w, m, s)
}

/// `Mobile.Weapon as BaseWeapon`.
fn wielded(w: &mut World, m: MobileId) -> Option<ItemId> {
    hk::weapon_item(w, m)
}

/// `ISlayer` of `m.Weapon`.
fn combat_distance_ok(w: &mut World, a: MobileId, d: MobileId, range: i32) -> bool {
    mb::in_range_point3d(w, a, point(w, d), range) && in_los(w, a, d)
}

// ---------------------------------------------------------------------------------------------------------------
// Virtual dispatch
// ---------------------------------------------------------------------------------------------------------------

/// `weapon.OnSwing(attacker, defender, damageBonus)`.
pub fn swing(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    bonus: f64,
) -> TimeSpan {
    (weapon::class(w, id).on_swing)(w, id, attacker, defender, bonus)
}

/// `weapon.CheckHit(attacker, defender)`.
pub fn check_hit(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) -> bool {
    (weapon::class(w, id).check_hit)(w, id, attacker, defender)
}

/// `weapon.OnHit(attacker, defender, damageBonus)`.
pub fn on_hit(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId, bonus: f64) {
    (weapon::class(w, id).on_hit)(w, id, attacker, defender, bonus);
}

/// `weapon.OnMiss(attacker, defender)`.
pub fn on_miss(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) {
    (weapon::class(w, id).on_miss)(w, id, attacker, defender);
}

/// `weapon.OnBeforeSwing(attacker, defender)`.
pub fn on_before_swing(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) {
    (weapon::class(w, id).on_before_swing)(w, id, attacker, defender);
}

/// `weapon.GetDelay(m)`.
pub fn get_delay(w: &mut World, id: ItemId, m: MobileId) -> TimeSpan {
    (weapon::class(w, id).get_delay)(w, id, m)
}

/// `weapon.AbsorbDamage(attacker, defender, damage)`.
pub fn absorb_damage(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    damage: i32,
) -> i32 {
    (weapon::class(w, id).absorb_damage)(w, id, attacker, defender, damage)
}

/// `weapon.GetDamageTypes(wielder, ..)`.
pub fn get_damage_types(w: &World, id: ItemId, wielder: Option<MobileId>) -> DamageTypes {
    (weapon::class(w, id).get_damage_types)(w, id, wielder)
}

/// `weapon.GetBaseDamage(attacker)`.
pub fn get_base_damage(w: &mut World, id: ItemId, attacker: MobileId) -> f64 {
    (weapon::class(w, id).get_base_damage)(w, id, attacker)
}

/// `weapon.GetDefendSkillValue(attacker, defender)`.
pub fn get_defend_skill_value(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
) -> f64 {
    (weapon::class(w, id).get_defend_skill_value)(w, id, attacker, defender)
}

// ---------------------------------------------------------------------------------------------------------------
// Small virtuals
// ---------------------------------------------------------------------------------------------------------------

/// `UseSkillMod`.
pub fn use_skill_mod(w: &World) -> bool {
    !core::aos(w)
}

/// `GetLuckBonus()`.
pub fn luck_bonus(w: &World, id: ItemId) -> i32 {
    data(w, id)
        .and_then(|d| craft::attribute_info(d.resource))
        .map_or(0, |a| a.weapon_luck)
}

/// `GetHitChanceBonus()`.
pub fn hit_chance_bonus(w: &World, id: ItemId) -> i32 {
    if !core::aos(w) {
        return 0;
    }
    use weapon::WeaponAccuracyLevel as L;
    match data(w, id).map(|d| d.accuracy_level) {
        Some(L::Accurate) => 2,
        Some(L::Surpassingly) => 4,
        Some(L::Eminently) => 6,
        Some(L::Exceedingly) => 8,
        Some(L::Supremely) => 10,
        _ => 0,
    }
}

/// `GetDamageBonus()`.
pub fn damage_bonus(w: &World, id: ItemId) -> i32 {
    weapon::class(w, id).virtual_damage_bonus
}

/// `GetBonus(value, scalar, threshold, offset)`.
pub fn get_bonus(value: f64, scalar: f64, threshold: f64, offset: f64) -> f64 {
    let mut bonus = value * scalar;
    if value >= threshold {
        bonus += offset;
    }
    bonus / 100.0
}

/// `GetBaseDamageRange(attacker, out min, out max)`.
pub fn get_base_damage_range(w: &World, id: ItemId, attacker: MobileId) -> (i32, i32) {
    if let Some(c) = cd(w, attacker) {
        if c.damage_min >= 0 {
            return (c.damage_min, c.damage_max);
        }
        if super::is_named(
            w.item(id).map_or(&weapon::BASE_WEAPON, |i| i.ty),
            "Server.Items.Fists",
        ) && !body_is_human(w, attacker)
        {
            let s = mb::str(w, attacker) / 28;
            return (s, s);
        }
    }
    (weapon::min_damage(w, id), weapon::max_damage(w, id))
}

/// `GetStatusDamage(from, out min, out max)`.
pub fn get_status_damage(w: &mut World, id: ItemId, from: MobileId) -> (i32, i32) {
    let (base_min, base_max) = get_base_damage_range(w, id, from);
    if core::aos(w) {
        (
            (scale_damage_aos(w, id, from, f64::from(base_min), false) as i32).max(1),
            (scale_damage_aos(w, id, from, f64::from(base_max), false) as i32).max(1),
        )
    } else {
        (
            (scale_damage_old(w, id, from, f64::from(base_min), false) as i32).max(1),
            (scale_damage_old(w, id, from, f64::from(base_max), false) as i32).max(1),
        )
    }
}

/// `GetUsedSkill(m, checkSkillAttrs)`.
pub fn get_used_skill(
    w: &mut World,
    id: ItemId,
    m: MobileId,
    check_skill_attrs: bool,
) -> SkillName {
    let use_best = weapon_attr(w, id, AosWeaponAttribute::UseBestSkill);
    let mage = weapon_attr(w, id, AosWeaponAttribute::MageWeapon);
    if check_skill_attrs && use_best != 0 {
        let swrd = val(w, m, SkillName::Swords);
        let fenc = val(w, m, SkillName::Fencing);
        let mcng = val(w, m, SkillName::Macing);
        let mut sk = SkillName::Swords;
        let mut v = swrd;
        if fenc > v {
            sk = SkillName::Fencing;
            v = fenc;
        }
        if mcng > v {
            sk = SkillName::Macing;
        }
        sk
    } else if mage != 0 {
        let own = weapon::skill(w, id);
        if val(w, m, SkillName::Magery) > val(w, m, own) {
            SkillName::Magery
        } else {
            own
        }
    } else {
        let mut sk = weapon::skill(w, id);
        if sk != SkillName::Wrestling
            && !is_player(w, m)
            && !body_is_human(w, m)
            && val(w, m, SkillName::Wrestling) > val(w, m, sk)
        {
            sk = SkillName::Wrestling;
        }
        sk
    }
}

/// `GetAttackSkillValue(attacker, defender)`.
pub fn get_attack_skill_value(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    _defender: MobileId,
) -> f64 {
    let sk = get_used_skill(w, id, attacker, true);
    val(w, attacker, sk)
}

/// `GetDefendSkillValue(attacker, defender)`, the base body: the defender's skill with this weapon.
pub fn get_defend_skill_value_base(
    w: &mut World,
    id: ItemId,
    _attacker: MobileId,
    defender: MobileId,
) -> f64 {
    let sk = get_used_skill(w, id, defender, true);
    val(w, defender, sk)
}

/// `ModifyHitChance` base.
pub fn modify_hit_chance_base(
    _w: &mut World,
    _id: ItemId,
    _a: MobileId,
    _d: MobileId,
    chance: f64,
) -> f64 {
    chance
}

/// `ModifyDamage` base.
pub fn modify_damage_base(
    _w: &mut World,
    _id: ItemId,
    _a: MobileId,
    _d: MobileId,
    damage: i32,
) -> i32 {
    damage
}

// ---------------------------------------------------------------------------------------------------------------
// OnSwing / delay / CheckHit
// ---------------------------------------------------------------------------------------------------------------

/// `OnBeforeSwing(attacker, defender)` base.
pub fn on_before_swing_base(w: &mut World, _id: ItemId, attacker: MobileId, defender: MobileId) {
    if let Some(a) = wa::current_ability(w, attacker)
        && !wa::on_before_swing(w, a, attacker, defender)
    {
        wa::clear_current_ability(w, attacker);
    }
    if hk::special_move_active(w, attacker)
        && !hk::special_move_on_before_swing(w, attacker, defender)
    {
        hk::special_move_clear(w, attacker);
    }
}

/// `OnSwing(attacker, defender, damageBonus)` base.
pub fn swing_base(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    damage_bonus: f64,
) -> TimeSpan {
    let mut can_swing = true;

    if core::aos(w) {
        can_swing = !mb::paralyzed(w, attacker) && !w.mobile(attacker).is_some_and(|m| m.frozen);
        if can_swing {
            can_swing = !hk::spell_blocks_movement(w, attacker);
        }
        if can_swing && is_player_mobile(w, attacker) {
            let peaced = player_mobile::data(w, attacker).map_or(w.now, |p| p.peaced_until);
            can_swing = peaced <= w.now;
        }
    }

    if is_player_mobile(w, attacker) && !hk::duel_check_item_equip(w, attacker, id) {
        can_swing = false;
    }

    if can_swing && mb::harmful_check(w, attacker, defender) {
        mb::disruptive_action(w, attacker);

        if let Some(ns) = w.mobile(attacker).and_then(|m| m.net_state) {
            let (a, d) = (
                w.mobile(attacker).map(|m| m.serial),
                w.mobile(defender).map(|m| m.serial),
            );
            if let (Some(a), Some(d)) = (a, d) {
                muo_server::packets::combat::send_swing(w, ns, a, d);
            }
        }

        if is_creature(w, attacker) {
            if crate::abilities::trigger_ability(
                w,
                attacker,
                crate::abilities::MonsterAbilityTrigger::COMBAT_ACTION,
                Some(defender),
            ) {
                return get_delay(w, id, attacker);
            }

            // Only change direction if they are not a player.
            let loc = point(w, defender);
            let dir = muo_server::mobile::part2::effects::get_direction_to(
                w, attacker, loc.x, loc.y, false,
            );
            props_b::set_direction(w, attacker, dir);
            // The class's own `GetWeaponAbility` override first; the hook is for a system that supplies one.
            let own = crate::creature::vtable::ctype(w, attacker)
                .and_then(|t| (t.get_weapon_ability)(w, attacker));
            if let Some(ab) = own.or_else(|| hk::creature_weapon_ability(w, attacker)) {
                let chance = match w.mobile(attacker).map(|m| m.ty) {
                    Some(_) => crate::creature::vtable::ctype(w, attacker)
                        .map_or(0.4, |t| (t.weapon_ability_chance)(w, attacker)),
                    None => 0.0,
                };
                if chance > utility::random_double(&mut w.rng) {
                    wa::set_current_ability(w, attacker, Some(ab));
                } else {
                    wa::clear_current_ability(w, attacker);
                }
            }
        }

        if check_hit(w, id, attacker, defender) {
            on_hit(w, id, attacker, defender, damage_bonus);
        } else {
            on_miss(w, id, attacker, defender);
        }
    }

    get_delay(w, id, attacker)
}

/// `GetDelay(m)` base.
pub fn get_delay_base(w: &mut World, id: ItemId, m: MobileId) -> TimeSpan {
    let mut speed = f64::from(weapon::speed(w, id));
    if speed == 0.0 {
        return TimeSpan::from_hours(1);
    }

    let stam = w.mobile(m).map_or(0, |x| x.stam);

    let delay_in_seconds = if core::se(w) {
        // This is likely true for Core.AOS as well; the old formula is left in for AOS for legacy.
        let mut bonus = aos_attr(w, m, AosAttribute::WeaponSpeed);
        bonus += hk::divine_fury_weapon_speed(w, m);
        // Bonus granted by successful use of Honorable Execution.
        bonus += hk::honorable_execution_swing_bonus(w, m);
        if let Some(b) = wa::dual_wield_swing_bonus(w, m) {
            bonus += b;
        }
        bonus += hk::reaper_form_swing_bonus(w, m);
        // Discordance gives a malus of -0/-28% to swing speed.
        if let Some(e) = hk::discordance_effect(w, m) {
            bonus -= e;
        }
        bonus -= hk::essence_of_wind_ssi_malus(w, m);
        if bonus > 60 {
            bonus = 60;
        }

        let mut ticks;
        if core::ml(w) {
            let stam_ticks = stam / 30;
            ticks = speed * 4.0;
            ticks = ((ticks - f64::from(stam_ticks)) * (100.0 / f64::from(100 + bonus))).floor();
        } else {
            speed = (speed * (f64::from(bonus) + 100.0) / 100.0).floor();
            if speed <= 0.0 {
                speed = 1.0;
            }
            ticks = (80000.0 / (f64::from(stam + 100) * speed) - 2.0).floor();
        }

        // Swing speed currently capped at one swing every 1.25 seconds (5 ticks).
        if ticks < 5.0 {
            ticks = 5.0;
        }
        ticks * 0.25
    } else if core::aos(w) {
        let mut v = (stam + 100) * (speed as i32);
        let mut bonus = aos_attr(w, m, AosAttribute::WeaponSpeed);
        if hk::divine_fury_under_effect(w, m) {
            bonus += 10;
        }
        if let Some(e) = hk::discordance_effect(w, m) {
            bonus -= e;
        }
        v += aos_helpers::scale(v, bonus);
        if v <= 0 {
            v = 1;
        }
        let mut d = (40000.0 / f64::from(v)).floor() * 0.5;
        // Maximum swing rate capped at one swing per second (OSI dev said it should be 1.25)
        if d < 1.25 {
            d = 1.25;
        }
        d
    } else {
        let mut v = (stam + 100) * (speed as i32);
        if v <= 0 {
            v = 1;
        }
        15000.0 / f64::from(v)
    };

    TimeSpan::from_secs_f64(delay_in_seconds)
}

/// `CheckHit(attacker, defender)` base.
pub fn check_hit_base(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) -> bool {
    let atk_weapon = wielded(w, attacker);
    let def_weapon = wielded(w, defender);

    let atk_skill = atk_weapon.map_or(SkillName::Wrestling, |a| weapon::skill(w, a));

    // `atkWeapon?.GetAttackSkillValue(..)` is the attacker's weapon, whatever `this` is.
    let mut atk_value = match atk_weapon {
        Some(a) => get_attack_skill_value(w, a, attacker, defender),
        None => 0.0,
    };
    let mut def_value = match def_weapon {
        Some(d) => get_defend_skill_value(w, d, attacker, defender),
        None => 0.0,
    };

    let our_value;
    let their_value;
    let mut bonus = hit_chance_bonus(w, id);

    if core::aos(w) {
        if atk_value <= -20.0 {
            atk_value = -19.9;
        }
        if def_value <= -20.0 {
            def_value = -19.9;
        }

        bonus += aos_attr(w, attacker, AosAttribute::AttackChance);

        if hk::divine_fury_under_effect(w, attacker) {
            bonus += 10; // attacker gets 10% bonus when they're under divine fury
        }

        if hk::animal_form_under(w, attacker, Some("Server.Mobiles.GreyWolf"))
            || hk::animal_form_under(w, attacker, Some("Server.Mobiles.BakeKitsune"))
        {
            bonus += 20; // attacker gets 20% bonus when under Wolf or Bake Kitsune form
        }

        if wa::hit_lower_is_under_attack_effect(w, attacker) {
            bonus -= 25; // Under Hit Lower Attack effect -> 25% malus
        }

        if let Some(ability) = wa::current_ability(w, attacker) {
            bonus += wa::accuracy_bonus(ability);
        }

        if hk::special_move_active(w, attacker) {
            bonus += hk::special_move_accuracy_bonus(w, attacker);
        }

        // Max Hit Chance Increase = 45%
        if bonus > 45 {
            bonus = 45;
        }

        our_value = (atk_value + 20.0) * f64::from(100 + bonus);

        bonus = aos_attr(w, defender, AosAttribute::DefendChance);

        if let Some(malus) = wa::force_arrow_defense_malus(w, attacker, defender) {
            bonus -= malus;
        }

        if hk::divine_fury_under_effect(w, defender) {
            bonus -= 20; // defender loses 20% bonus when they're under divine fury
        }

        if wa::hit_lower_is_under_defense_effect(w, defender) {
            bonus -= 25; // Under Hit Lower Defense effect -> 25% malus
        }

        if let Some(b) = wa::block_bonus(w, defender) {
            bonus += b;
        }

        if let Some(m) = hk::surprise_attack_malus(w, defender) {
            bonus -= m;
        }

        // Defender loses -0/-28% if under the effect of Discordance.
        if let Some(e) = hk::discordance_effect(w, attacker) {
            bonus -= e;
        }

        // Defense Chance Increase = 45%
        if bonus > 45 {
            bonus = 45;
        }

        their_value = (def_value + 20.0) * f64::from(100 + bonus);

        bonus = 0;
    } else {
        our_value = (atk_value + 50.0).max(0.1);
        their_value = (def_value + 50.0).max(0.1);
    }

    let mut chance = our_value / (their_value * 2.0) * 1.0 + f64::from(bonus) / 100.0;

    chance = (weapon::class(w, id).modify_hit_chance)(w, id, attacker, defender, chance);

    if core::aos(w) && chance < 0.02 {
        chance = 0.02;
    }

    muo_server::mobile::part2::stats::check_skill_chance(w, attacker, atk_skill, chance)
}

// ---------------------------------------------------------------------------------------------------------------
// Parry and absorption
// ---------------------------------------------------------------------------------------------------------------

fn shield_of(w: &World, m: MobileId) -> Option<ItemId> {
    let item = muo_server::mobile::part2::items::find_item_on_layer(
        w,
        m,
        muo_server::layer::Layer::TwoHanded,
    )?;
    w.item(item)
        .is_some_and(|i| i.ty.is(&super::armor::BASE_SHIELD))
        .then_some(item)
}

/// `CheckParry(defender)`.
pub fn check_parry(w: &mut World, defender: MobileId) -> bool {
    if w.mobile(defender).is_none() {
        return false;
    }
    let shield = shield_of(w, defender);

    let parry = val(w, defender, SkillName::Parry);
    let bushido_non_racial = skills::non_racial_value(w, defender, SkillName::Bushido);
    let bushido = val(w, defender, SkillName::Bushido);
    let dex = mb::dex(w, defender);
    let mut chance;

    if shield.is_some() {
        // As per OSI, no genitive effect from the Racial stuffs, ie, 120 parry and '0' bushido with humans
        chance = ((parry - bushido_non_racial) / 400.0).max(0.0);

        // Parry/Bushido over 100 grants a 5% bonus.
        if parry >= 100.0 || bushido >= 100.0 {
            chance += 0.05;
        }

        // Evasion grants a variable bonus post ML. 50% prior.
        if hk::evasion_is_evading(w, defender) {
            chance *= hk::evasion_parry_scalar(w, defender);
        }

        // Low dexterity lowers the chance.
        if dex < 80 {
            chance = chance * f64::from(20 + dex) / 100.0;
        }

        return muo_server::mobile::part2::stats::check_skill_chance(
            w,
            defender,
            SkillName::Parry,
            chance,
        );
    }

    let weapon_item = wielded(w, defender);
    let is_fists_or_ranged = match mb::weapon(w, defender) {
        Some(muo_server::interfaces::WeaponRef::Default) => true,
        Some(muo_server::interfaces::WeaponRef::Item(i)) => w.item(i).is_some_and(|x| {
            super::is_named(x.ty, "Server.Items.Fists") || x.ty.is(&weapon::BASE_RANGED)
        }),
        None => false,
    };
    if is_fists_or_ranged {
        return false;
    }

    let one_handed = weapon_item
        .and_then(|i| w.item(i))
        .is_some_and(|i| i.layer == muo_server::layer::Layer::OneHanded);
    let divisor = if one_handed { 48000.0 } else { 41140.0 };

    chance = parry * bushido / divisor;

    let mut aos_chance = parry / 800.0;

    // Parry or Bushido over 100 grant a 5% bonus.
    if parry >= 100.0 {
        chance += 0.05;
        aos_chance += 0.05;
    } else if bushido >= 100.0 {
        chance += 0.05;
    }

    // Evasion grants a variable bonus post ML. 50% prior.
    if hk::evasion_is_evading(w, defender) {
        chance *= hk::evasion_parry_scalar(w, defender);
    }

    // Low dexterity lowers the chance.
    if dex < 80 {
        chance = chance * f64::from(20 + dex) / 100.0;
    }

    if chance > aos_chance {
        return muo_server::mobile::part2::stats::check_skill_chance(
            w,
            defender,
            SkillName::Parry,
            chance,
        );
    }

    // Only skillcheck if wielding a shield & there's no effect from Bushido
    aos_chance > utility::random_double(&mut w.rng)
}

fn armor_by_position(w: &World, defender: MobileId, chance: f64) -> Option<ItemId> {
    if chance < 0.07 {
        props_b::neck_armor(w, defender)
    } else if chance < 0.14 {
        props_b::hand_armor(w, defender)
    } else if chance < 0.28 {
        props_b::arms_armor(w, defender)
    } else if chance < 0.43 {
        props_b::head_armor(w, defender)
    } else if chance < 0.65 {
        props_b::legs_armor(w, defender)
    } else {
        props_b::chest_armor(w, defender)
    }
}

/// `AbsorbDamageAOS(attacker, defender, damage)`.
pub fn absorb_damage_aos(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    mut damage: i32,
) -> i32 {
    let mut blocked = false;

    if is_player(w, defender) || body_is_human(w, defender) {
        blocked = check_parry(w, defender);

        if blocked {
            fx::fixed_effect(w, defender, 0x37B9, 10, 16);
            damage = 0;

            // Successful block removes the Honorable Execution penalty.
            hk::honorable_execution_remove_penalty(w, defender);

            if hk::counter_attack_is_countering(w, defender) {
                if let Some(wp) = wielded(w, defender) {
                    fx::fixed_particles_hue(
                        w,
                        defender,
                        0x3779,
                        1,
                        15,
                        0x158B,
                        0x0,
                        0x3,
                        EffectLayer::Waist,
                    );
                    swing(w, wp, defender, attacker, 1.0);
                }
                hk::counter_attack_stop(w, defender);
            }

            if hk::confidence_is_confident(w, defender) {
                // Your confidence reassures you as you successfully block your opponent's blow.
                msg(w, defender, 1063117);

                let bushido = val(w, defender, SkillName::Bushido);
                let hits_gain = super::random_min_max(w, 1, (bushido / 12.0) as i32);
                let stam_gain = super::random_min_max(w, 1, (bushido / 5.0) as i32);
                let (h, s) = w.mobile(defender).map_or((0, 0), |m| (m.hits, m.stam));
                props_b::set_hits(w, defender, h + hits_gain);
                props_b::set_stam(w, defender, s + stam_gain);
            }

            if let Some(shield) = shield_of(w, defender) {
                hk::shield_on_hit(w, shield, id, damage);
            }
        }
    }

    if !blocked {
        let position_chance = utility::random_double(&mut w.rng);
        if let Some(armor) = armor_by_position(w, defender, position_chance) {
            // call OnHit to lose durability
            hk::wearable_on_hit(w, armor, id, damage);
        }
    }

    damage
}

/// `AbsorbDamage(attacker, defender, damage)` base.
pub fn absorb_damage_base(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    defender: MobileId,
    mut damage: i32,
) -> i32 {
    if core::aos(w) {
        return absorb_damage_aos(w, id, attacker, defender, damage);
    }

    if let Some(shield) = shield_of(w, defender) {
        damage = hk::shield_on_hit(w, shield, id, damage);
    }

    let chance = utility::random_double(&mut w.rng);

    if let Some(armor) = armor_by_position(w, defender, chance)
        && let Some(d) = hk::wearable_on_hit(w, armor, id, damage)
    {
        damage = d;
    }

    let (va, vam) = w
        .mobile(defender)
        .map_or((0, 0), |m| (m.virtual_armor, m.virtual_armor_mod));
    let virtual_armor = va + vam;

    if virtual_armor > 0 {
        let scalar = if chance < 0.14 {
            0.07
        } else if chance < 0.28 {
            0.14
        } else if chance < 0.43 {
            0.15
        } else if chance < 0.65 {
            0.22
        } else {
            0.35
        };

        let from = (f64::from(virtual_armor) * scalar) as i32 / 2;
        let to = (f64::from(virtual_armor) * scalar) as i32;

        damage -= utility::random_from(&mut w.rng, from, to - from + 1);
    }

    damage
}

/// `GetPackInstinctBonus(attacker, defender)`.
pub fn pack_instinct_bonus(w: &mut World, attacker: MobileId, defender: MobileId) -> i32 {
    if is_player(w, attacker) || is_player(w, defender) {
        return 0;
    }
    if !is_creature(w, attacker) {
        return 0;
    }
    let Some(ct) = crate::creature::vtable::ctype(w, attacker) else {
        return 0;
    };
    let pack = (ct.pack_instinct)(w, attacker);
    if pack.is_empty() {
        return 0;
    }
    let Some(master) = crate::creature::control::get_master(w, attacker) else {
        return 0;
    };

    let mut in_pack = 1;
    for m in muo_server::mobile::part2::enumerators::get_mobiles_in_range(w, defender, 1) {
        if m == attacker || !is_creature(w, m) {
            continue;
        }
        let Some(mt) = crate::creature::vtable::ctype(w, m) else {
            continue;
        };
        if (mt.pack_instinct)(w, m).intersects(pack)
            && crate::creature::control::get_master(w, m) == Some(master)
            && mb::combatant(w, m) == Some(defender)
        {
            in_pack += 1;
        }
    }

    match in_pack {
        n if n >= 5 => 100,
        4 => 75,
        3 => 50,
        2 => 25,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------------------------------------------
// OnHit / OnMiss
// ---------------------------------------------------------------------------------------------------------------

/// `OnHit(attacker, defender, damageBonus)` base.
pub fn on_hit_base(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    mut defender: MobileId,
    damage_bonus: f64,
) {
    if hk::mirror_image_has_clone(w, defender)
        && val(w, defender, SkillName::Ninjitsu) / 150.0 > utility::random_double(&mut w.rng)
        && let Some(clone) = hk::mirror_image_clone_in_range(w, defender)
    {
        // Your attack has been diverted to a nearby mirror image of your target!
        msg(w, attacker, 1063141);
        // You manage to divert the attack onto one of your nearby mirror images.
        msg(w, defender, 1063140);
        defender = clone;
    }

    play_swing_animation(w, id, attacker);
    play_hurt_animation(w, defender);

    let s = get_hit_attack_sound(w, id, attacker, defender);
    fx::play_sound(w, attacker, s);
    let s = get_hit_defend_sound(w, defender);
    fx::play_sound(w, defender, s);

    let mut damage = compute_damage(w, id, attacker, defender);

    // The following damage bonuses multiply damage by a factor. Capped at x3 (300%).
    let mut percentage_bonus = 0;

    let mut a = wa::current_ability(w, attacker);
    let mut mv = hk::special_move_active(w, attacker);

    if let Some(ab) = a {
        percentage_bonus += (wa::damage_scalar(ab) * 100.0) as i32 - 100;
    }

    if mv {
        percentage_bonus +=
            (hk::special_move_damage_scalar(w, attacker, defender) * 100.0) as i32 - 100;
    }

    percentage_bonus +=
        (wa::force_of_nature_damage_scalar(w, attacker, defender) * 100.0) as i32 - 100;

    percentage_bonus += (damage_bonus * 100.0) as i32 - 100;

    let cs = check_slayers(w, id, attacker, defender);

    if cs != CheckSlayerResult::None {
        if cs == CheckSlayerResult::Slayer {
            fx::fixed_effect(w, defender, 0x37B9, 10, 5);
        }
        percentage_bonus += 100;
    }

    if !is_player(w, attacker) {
        if is_player_mobile(w, defender)
            && let Some(p) = player_mobile::data(w, defender)
            && let Some(t) = p.enemy_of_one_type
            && !w.mobile(attacker).is_some_and(|m| std::ptr::eq(m.ty, t))
        {
            percentage_bonus += 100;
        }
    } else if !is_player(w, defender) && is_player_mobile(w, attacker) {
        let dty = w.mobile(defender).map(|m| m.ty);
        if player_mobile::data(w, attacker).is_some_and(|p| p.waiting_for_enemy) {
            // `EnemyOfOneType = defender.GetType()` runs the setter (DeltaEnemies resends the movement of both types)
            player_mobile::members::set_enemy_of_one_type(w, attacker, dty);
            if let Some(p) = player_mobile::data_mut(w, attacker) {
                p.waiting_for_enemy = false;
            }
        }

        let eoo = player_mobile::data(w, attacker).and_then(|p| p.enemy_of_one_type);
        if let (Some(e), Some(d)) = (eoo, dty)
            && std::ptr::eq(e, d)
        {
            fx::fixed_effect_hue(w, defender, 0x37B9, 10, 5, 1160, 0);
            percentage_bonus += 50;
        }
    }

    let pack_bonus = pack_instinct_bonus(w, attacker, defender);
    if pack_bonus != 0 {
        percentage_bonus += pack_bonus;
    }

    if w.resources.get::<DoubleStrikeFlag>().is_some_and(|f| f.0) {
        percentage_bonus -= 10;
    }

    let (slayer1, slayer2) = data(w, id).map_or((0, 0), |d| (d.slayer, d.slayer2));
    if (slayer1 == SlayerName::Silver as i32 || slayer2 == SlayerName::Silver as i32)
        && hk::in_necromancer_form_not_horrific(w, defender)
    {
        percentage_bonus += 25;
    }

    if is_player_mobile(w, attacker) && !(core::ml(w) && is_player_mobile(w, defender)) {
        if hk::virtue_honor_active(w, attacker)
            && mb::in_range_point3d(w, attacker, point(w, defender), 1)
        {
            percentage_bonus += 25;
        }
        percentage_bonus += hk::honor_perfection_bonus(w, attacker, defender);
    }

    percentage_bonus += hk::talisman_killer_bonus(w, attacker, defender);

    percentage_bonus = percentage_bonus.min(300);

    damage = aos_helpers::scale(damage, 100 + percentage_bonus);
    damage = (weapon::class(w, id).modify_damage)(w, id, attacker, defender, damage);

    let def_loc = (point(w, defender), w.mobile(defender).and_then(|m| m.map));
    let att_creature = is_creature(w, attacker);
    let def_creature = is_creature(w, defender);

    if att_creature && let Some(ct) = crate::creature::vtable::ctype(w, attacker) {
        (ct.alter_melee_damage_to)(w, attacker, defender, &mut damage);
    }
    if def_creature && let Some(ct) = crate::creature::vtable::ctype(w, defender) {
        (ct.alter_melee_damage_from)(w, defender, attacker, &mut damage);
    }

    damage = absorb_damage(w, id, attacker, defender, damage);

    if !core::aos(w) && damage < 1 {
        damage = 1;
    } else if core::aos(w) && damage == 0 && a.is_some_and(|ab| wa::validate(w, ab, attacker)) {
        // Parried
        a = None;
        wa::clear_current_ability(w, attacker);
        msg(w, attacker, 1061140); // Your attack was parried!
    }

    add_blood(w, attacker, defender, damage);

    let mut t = get_damage_types(w, id, Some(attacker));

    if core::ml(w) && w.item(id).is_some_and(|i| i.ty.is(&weapon::BASE_RANGED)) {
        let r = hk::quiver_alter_bow_damage(
            w,
            attacker,
            [t.phys, t.fire, t.cold, t.pois, t.nrgy, t.chaos, t.direct],
        );
        t = DamageTypes {
            phys: r[0],
            fire: r[1],
            cold: r[2],
            pois: r[3],
            nrgy: r[4],
            chaos: r[5],
            direct: r[6],
        };
    }

    if data(w, id).is_some_and(|d| d.consecrated) {
        let ty = w.mobile(defender).map(|m| m.ty);
        let mut res = |f: fn(&muo_server::MobileType) -> ResistFn| -> i32 {
            ty.map_or(0, |ty| f(ty)(w, defender))
        };
        let phys = res(|t| t.physical_resistance);
        let fire = res(|t| t.fire_resistance);
        let cold = res(|t| t.cold_resistance);
        let pois = res(|t| t.poison_resistance);
        let nrgy = res(|t| t.energy_resistance);

        let (mut low, mut kind) = (phys, 0);
        if fire < low {
            low = fire;
            kind = 1;
        }
        if cold < low {
            low = cold;
            kind = 2;
        }
        if pois < low {
            low = pois;
            kind = 3;
        }
        if nrgy < low {
            kind = 4;
        }

        t = DamageTypes::default();
        match kind {
            0 => t.phys = 100,
            1 => t.fire = 100,
            2 => t.cold = 100,
            3 => t.pois = 100,
            _ => t.nrgy = 100,
        }
    }

    // TODO: scale damage, alongside the leech effects below, to weapon speed.
    if damage > 0 && hk::immolating_is(w, id) {
        hk::immolating_do_effect(w, id, defender);
    }

    if let Some(ab) = a
        && !wa::on_before_damage(w, ab, attacker, defender)
    {
        wa::clear_current_ability(w, attacker);
        a = None;
    }

    if mv && !hk::special_move_on_before_damage(w, attacker, defender) {
        hk::special_move_clear(w, attacker);
        mv = false;
    }

    if let Some(reduction) = wa::feint_damage_reduction(w, attacker, defender) {
        // example: 35 damage * 50 / 100 = 17 damage
        damage -= damage * reduction / 100;
    }

    let ignore_armor = a == Some(Ability::ArmorIgnore)
        || (mv && hk::special_move_ignore_armor(w, attacker))
        || wa::blade_weaving(w, attacker) == Some(Ability::ArmorIgnore);

    let is_ranged = w.item(id).is_some_and(|i| i.ty.is(&weapon::BASE_RANGED));
    let damage_given = aos_helpers::damage(
        w,
        defender,
        Some(attacker),
        damage,
        DamageSplit {
            phys: t.phys,
            fire: t.fire,
            cold: t.cold,
            pois: t.pois,
            nrgy: t.nrgy,
        },
        DamageOptions {
            ignore_armor,
            chaos: t.chaos,
            direct: t.direct,
            keep_alive: false,
            archer: is_ranged,
            death_strike: false,
        },
    );

    if damage_given > 0 {
        let property_bonus = if mv {
            hk::special_move_property_bonus(w, attacker)
        } else {
            1.0
        };

        // Leech abilities
        if core::aos(w) {
            let mut life_leech = 0;
            let mut stam_leech = 0;
            let mut mana_leech = 0;

            let roll = |w: &mut World| utility::random(&mut w.rng, 100);

            if (f64::from(attacker_weapon_attr(
                w,
                attacker,
                AosWeaponAttribute::HitLeechHits,
            )) * property_bonus) as i32
                > roll(w)
            {
                life_leech += 30; // HitLeechHits% chance to leech 30% of damage as hit points
            }
            if (f64::from(attacker_weapon_attr(
                w,
                attacker,
                AosWeaponAttribute::HitLeechStam,
            )) * property_bonus) as i32
                > roll(w)
            {
                stam_leech += 100; // HitLeechStam% chance to leech 100% of damage as stamina
            }
            if (f64::from(attacker_weapon_attr(
                w,
                attacker,
                AosWeaponAttribute::HitLeechMana,
            )) * property_bonus) as i32
                > roll(w)
            {
                mana_leech += 40; // HitLeechMana% chance to leech 40% of damage as mana
            }

            if data(w, id).is_some_and(|d| d.cursed) {
                life_leech += 50; // Additional 50% life leech for cursed weapons (necro spell)
            }

            if life_leech != 0 {
                let h = w.mobile(attacker).map_or(0, |m| m.hits);
                props_b::set_hits(
                    w,
                    attacker,
                    h + aos_helpers::scale(damage_given, life_leech),
                );
            }
            if stam_leech != 0 {
                let s = w.mobile(attacker).map_or(0, |m| m.stam);
                props_b::set_stam(
                    w,
                    attacker,
                    s + aos_helpers::scale(damage_given, stam_leech),
                );
            }
            if mana_leech != 0 {
                let m = w.mobile(attacker).map_or(0, |m| m.mana);
                props_b::set_mana(
                    w,
                    attacker,
                    m + aos_helpers::scale(damage_given, mana_leech),
                );
            }
            if life_leech != 0 || stam_leech != 0 || mana_leech != 0 {
                fx::play_sound(w, attacker, 0x44D);
            }
        }

        let (max_hp, hp) = data(w, id).map_or((0, 0), |d| (d.max_hit_points, d.hit_points));
        let is_acid_monster = max_hp > 0
            && weapon::max_range(w, id) <= 1
            && super::attr(
                w,
                id,
                AttrKind::Attributes,
                AosAttribute::SpellChanneling as u32,
            ) == 0
            && (mobile_is_named(w, defender, "Server.Mobiles.Slime")
                || mobile_is_named(w, defender, "Server.Mobiles.AcidElemental"));

        // Stratics says 50% chance, seems more like 4%..
        if is_acid_monster || utility::random(&mut w.rng, 25) == 0 {
            if is_acid_monster {
                msgs::local_overhead_message_number(
                    w,
                    attacker,
                    MessageType::REGULAR,
                    0x3B2,
                    500263,
                    "",
                );
                // *Acid blood scars your weapon!*
            }

            if core::aos(w)
                && weapon_attr(w, id, AosWeaponAttribute::SelfRepair)
                    > utility::random(&mut w.rng, 10)
            {
                weapon::set_hit_points(w, id, hp + 2);
            } else if hp > 0 {
                weapon::set_hit_points(w, id, hp - 1);
            } else if max_hp > 1 {
                weapon::set_max_hit_points(w, id, max_hp - 1);

                if let Some(m) = super::parent_mobile(w, id) {
                    // Your equipment is severely damaged.
                    msgs::local_overhead_message_number(
                        w,
                        m,
                        MessageType::REGULAR,
                        0x3B2,
                        1061121,
                        "",
                    );
                }
            } else if let Some(ty) = w.item(id).map(|i| i.ty) {
                (ty.delete)(w, id);
            }
        }

        if mobile_is_named(w, attacker, "Server.Mobiles.VampireBatFamiliar")
            && let Some(caster) = crate::creature::control::get_master(w, attacker)
            && w.mobile(caster).map(|m| m.map) == w.mobile(attacker).map(|m| m.map)
        {
            let h = w.mobile(caster).map_or(0, |m| m.hits);
            props_b::set_hits(w, caster, h + damage);
        }

        if core::aos(w) {
            use AosWeaponAttribute as W;
            let pb = |w: &World, a: W| {
                (f64::from(attacker_weapon_attr(w, attacker, a)) * property_bonus) as i32
            };

            let phys_chance = pb(w, W::HitPhysicalArea);
            let fire_chance = pb(w, W::HitFireArea);
            let cold_chance = pb(w, W::HitColdArea);
            let pois_chance = pb(w, W::HitPoisonArea);
            let nrgy_chance = pb(w, W::HitEnergyArea);

            let roll = |w: &mut World, chance: i32| {
                chance != 0 && chance > utility::random(&mut w.rng, 100)
            };

            if roll(w, phys_chance) {
                do_area_attack(w, id, attacker, defender, 0x10E, 50, [100, 0, 0, 0, 0]);
            }
            if roll(w, fire_chance) {
                do_area_attack(w, id, attacker, defender, 0x11D, 1160, [0, 100, 0, 0, 0]);
            }
            if roll(w, cold_chance) {
                do_area_attack(w, id, attacker, defender, 0x0FC, 2100, [0, 0, 100, 0, 0]);
            }
            if roll(w, pois_chance) {
                do_area_attack(w, id, attacker, defender, 0x205, 1166, [0, 0, 0, 100, 0]);
            }
            if roll(w, nrgy_chance) {
                do_area_attack(w, id, attacker, defender, 0x1F1, 120, [0, 0, 0, 0, 100]);
            }

            let ma_chance = pb(w, W::HitMagicArrow);
            let harm_chance = pb(w, W::HitHarm);
            let fireball_chance = pb(w, W::HitFireball);
            let lightning_chance = pb(w, W::HitLightning);
            let dispel_chance = pb(w, W::HitDispel);

            if roll(w, ma_chance) {
                do_magic_arrow(w, id, attacker, defender);
            }
            if roll(w, harm_chance) {
                do_harm(w, id, attacker, defender);
            }
            if roll(w, fireball_chance) {
                do_fireball(w, id, attacker, defender);
            }
            if roll(w, lightning_chance) {
                do_lightning(w, id, attacker, defender);
            }
            if roll(w, dispel_chance) {
                do_dispel(w, attacker, defender);
            }

            let la_chance = pb(w, W::HitLowerAttack);
            let ld_chance = pb(w, W::HitLowerDefend);

            if roll(w, la_chance) {
                do_lower_attack(w, defender);
            }
            if roll(w, ld_chance) {
                do_lower_defense(w, defender);
            }
        }
    }

    if att_creature && let Some(ct) = crate::creature::vtable::ctype(w, attacker) {
        (ct.on_gave_melee_attack)(w, attacker, defender, damage);
    }
    if def_creature && let Some(ct) = crate::creature::vtable::ctype(w, defender) {
        (ct.on_got_melee_attack)(w, defender, attacker, damage);
    }

    if let Some(ab) = a {
        wa::on_hit(w, ab, attacker, defender, damage, def_loc);
    }
    if mv {
        hk::special_move_on_hit(w, attacker, defender, damage);
    }

    wa::force_of_nature_on_hit(w, attacker, defender);

    hk::honor_on_target_hit(w, defender, attacker);

    if !w.item(id).is_some_and(|i| i.ty.is(&weapon::BASE_RANGED)) {
        if hk::animal_form_under(w, attacker, Some("Server.Mobiles.GiantSerpent")) {
            mb::apply_poison(
                w,
                defender,
                Some(attacker),
                crate::misc::poison_kinds::by_name(w, "Lesser")
                    .unwrap_or(muo_server::poison::PoisonId(0)),
            );
        }
        if hk::animal_form_under(w, defender, Some("Server.Mobiles.BullFrog")) {
            mb::apply_poison(
                w,
                attacker,
                Some(defender),
                crate::misc::poison_kinds::by_name(w, "Regular")
                    .unwrap_or(muo_server::poison::PoisonId(1)),
            );
        }
    }
}

/// `OnMiss(attacker, defender)` base.
pub fn on_miss_base(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) {
    play_swing_animation(w, id, attacker);
    let s = get_miss_attack_sound(w, id, attacker);
    fx::play_sound(w, attacker, s);
    let s = get_miss_defend_sound();
    fx::play_sound(w, defender, s);

    if let Some(a) = wa::current_ability(w, attacker) {
        wa::on_miss(w, a, attacker, defender);
    }
    if hk::special_move_active(w, attacker) {
        hk::special_move_on_miss(w, attacker, defender);
    }

    hk::honor_on_target_missed(w, defender, attacker);
}

// ---------------------------------------------------------------------------------------------------------------
// Damage
// ---------------------------------------------------------------------------------------------------------------

/// `GetAosDamage(attacker, bonus, dice, sides)`.
pub fn get_aos_damage(w: &mut World, attacker: MobileId, bonus: i32, dice: i32, sides: i32) -> f64 {
    let mut damage = utility::dice(&mut w.rng, dice, sides, bonus) * 100;

    // Inscription bonus
    let inscribe_skill = skills::fixed(w, attacker, SkillName::Inscribe);

    let mut damage_bonus = inscribe_skill / 200;

    if inscribe_skill >= 1000 {
        damage_bonus += 5;
    }

    if is_player(w, attacker) {
        // Int bonus
        damage_bonus += mb::int(w, attacker) / 10;

        // SDI bonus
        damage_bonus += aos_attr(w, attacker, AosAttribute::SpellDamage);

        if let Some(m) = wa::psychic_attack_spell_damage_malus(w, attacker) {
            damage_bonus -= m;
        }

        damage_bonus += hk::reaper_form_spell_damage_bonus(w, attacker);
    }

    damage = aos_helpers::scale(damage, 100 + damage_bonus);

    f64::from(damage) / 100.0
}

/// `CheckSlayers(attacker, defender)`.
pub fn check_slayers(
    w: &mut World,
    _id: ItemId,
    attacker: MobileId,
    defender: MobileId,
) -> CheckSlayerResult {
    if wa::current_ability(w, attacker) != Some(Ability::LightningArrow) {
        let atk_weapon = wielded(w, attacker);
        let (s1, s2) = atk_weapon
            .and_then(|i| data(w, i))
            .map_or((0, 0), |d| (d.slayer, d.slayer2));
        let atk_slayer = SlayerName::from_i32(s1).and_then(|n| slayer::entry_by_name(w, n));
        let atk_slayer2 = SlayerName::from_i32(s2).and_then(|n| slayer::entry_by_name(w, n));

        if atk_weapon.is_some_and(|i| {
            w.item(i)
                .is_some_and(|x| super::is_named(x.ty, "Server.Items.ButchersWarCleaver"))
        }) && hk::talisman_slayer_bovine_slays(w, defender)
        {
            return CheckSlayerResult::Slayer;
        }

        if atk_slayer.is_some_and(|(e, _)| slayer::slays(w, e, defender))
            || atk_slayer2.is_some_and(|(e, _)| slayer::slays(w, e, defender))
        {
            return CheckSlayerResult::Slayer;
        }

        if hk::talisman_slays(w, attacker, defender) {
            return CheckSlayerResult::Slayer;
        }
    }

    if !core::se(w) {
        let def_slayers = hk::equipped_spellbook_slayers(w, defender).or_else(|| {
            wielded(w, defender)
                .and_then(|i| data(w, i))
                .map(|d| (d.slayer, d.slayer2))
        });

        if let Some((s1, s2)) = def_slayers {
            let e1 = SlayerName::from_i32(s1).and_then(|n| slayer::entry_by_name(w, n));
            let e2 = SlayerName::from_i32(s2).and_then(|n| slayer::entry_by_name(w, n));

            if e1.is_some_and(|(_, g)| slayer::opposition_super_slays(w, g, attacker))
                || e2.is_some_and(|(_, g)| slayer::opposition_super_slays(w, g, attacker))
            {
                return CheckSlayerResult::Opposition;
            }
        }
    }

    CheckSlayerResult::None
}

/// `AddBlood(attacker, defender, damage)`.
pub fn add_blood(w: &mut World, _attacker: MobileId, defender: MobileId, damage: i32) {
    if damage <= 0 {
        return;
    }
    let Some((loc, map)) = w.mobile(defender).map(|m| (m.location, m.map)) else {
        return;
    };

    if let Some(b) = crate::factory::make_item(w, "Server.Items.Blood") {
        ops::move_to_world_map(w, b, loc, map);
    }

    let extra_blood = if core::se(w) {
        super::random_min_max(w, 3, 4)
    } else {
        super::random_min_max(w, 0, 1)
    };

    for _ in 0..extra_blood {
        let dx = super::random_min_max(w, -1, 1);
        let dy = super::random_min_max(w, -1, 1);
        if let Some(b) = crate::factory::make_item(w, "Server.Items.Blood") {
            ops::move_to_world_map(w, b, Point3D::new(loc.x + dx, loc.y + dy, loc.z), map);
        }
    }
}

/// `GetDamageTypes(wielder, ..)` base. `wielder` is `None` for the property list and the hue.
pub fn get_damage_types_base(w: &World, id: ItemId, wielder: Option<MobileId>) -> DamageTypes {
    if let Some(bc) = wielder
        && let Some(c) = cd(w, bc)
    {
        return DamageTypes {
            phys: c.physical_damage,
            fire: c.fire_damage,
            cold: c.cold_damage,
            pois: c.poison_damage,
            nrgy: c.energy_damage,
            chaos: c.chaos_damage,
            direct: c.direct_damage,
        };
    }

    let Some(d) = data(w, id) else {
        return DamageTypes {
            phys: 100,
            ..Default::default()
        };
    };
    let el = |a: AosElementAttribute| d.element_damages.get(core::aos(w), a as u32);
    let (mut fire, mut cold, mut pois, mut nrgy, mut chaos, mut direct) = (
        el(AosElementAttribute::Fire),
        el(AosElementAttribute::Cold),
        el(AosElementAttribute::Poison),
        el(AosElementAttribute::Energy),
        el(AosElementAttribute::Chaos),
        el(AosElementAttribute::Direct),
    );

    let mut phys = 100 - fire - cold - pois - nrgy - chaos - direct;

    if let Some(info) = craft::attribute_info(d.resource) {
        let mut left = phys;

        left = apply_craft_attribute_element_damage(info.weapon_cold_damage, &mut cold, left);
        left = apply_craft_attribute_element_damage(info.weapon_energy_damage, &mut nrgy, left);
        left = apply_craft_attribute_element_damage(info.weapon_fire_damage, &mut fire, left);
        left = apply_craft_attribute_element_damage(info.weapon_poison_damage, &mut pois, left);
        left = apply_craft_attribute_element_damage(info.weapon_chaos_damage, &mut chaos, left);
        left = apply_craft_attribute_element_damage(info.weapon_direct_damage, &mut direct, left);

        phys = left;
    }

    DamageTypes {
        phys,
        fire,
        cold,
        pois,
        nrgy,
        chaos,
        direct,
    }
}

fn apply_craft_attribute_element_damage(
    attr_damage: i32,
    element: &mut i32,
    total_remaining: i32,
) -> i32 {
    if total_remaining <= 0 {
        return 0;
    }
    if attr_damage <= 0 {
        return total_remaining;
    }
    let mut applied = attr_damage;
    if applied + *element > 100 {
        applied = 100 - *element;
    }
    if applied > total_remaining {
        applied = total_remaining;
    }
    *element += applied;
    total_remaining - applied
}

/// `GetBaseDamage(attacker)` base.
pub fn get_base_damage_base(w: &mut World, id: ItemId, attacker: MobileId) -> f64 {
    let (min, max) = get_base_damage_range(w, id, attacker);

    let mut damage = super::random_min_max(w, min, max);

    if core::aos(w) {
        return f64::from(damage);
    }

    // Apply damage level offset: Regular 0, Ruin 1, Might 3, Force 5, Power 7, Vanq 9
    let level = data(w, id).map_or(WeaponDamageLevel::Regular, |d| d.damage_level);
    if level != WeaponDamageLevel::Regular {
        // Toward the end of T2A, calculations were changed
        if !core::t2a(w) {
            damage += level as i32;
        } else {
            damage += 2 * level as i32 - 1;
        }
    }

    f64::from(damage)
}

/// `ScaleDamageAOS(attacker, damage, checkSkills)`.
pub fn scale_damage_aos(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    damage: f64,
    check_skills: bool,
) -> f64 {
    let ty = weapon::weapon_type(w, id);
    if check_skills {
        // Passively check tactics for gain
        let cap = skills::cap(w, attacker, SkillName::Tactics);
        muo_server::mobile::part2::stats::check_skill(w, attacker, SkillName::Tactics, 0.0, cap);

        // Passively check Anatomy for gain
        let cap = skills::cap(w, attacker, SkillName::Anatomy);
        muo_server::mobile::part2::stats::check_skill(w, attacker, SkillName::Anatomy, 0.0, cap);

        if ty == WeaponType::Axe {
            // Passively check Lumberjacking for gain
            muo_server::mobile::part2::stats::check_skill(
                w,
                attacker,
                SkillName::Lumberjacking,
                0.0,
                100.0,
            );
        }
    }

    // These are the bonuses given by the physical characteristics of the mobile. No caps apply.
    let strength_bonus = get_bonus(f64::from(mb::str(w, attacker)), 0.300, 100.0, 5.00);
    let anatomy_bonus = get_bonus(val(w, attacker, SkillName::Anatomy), 0.500, 100.0, 5.00);
    let tactics_bonus = get_bonus(val(w, attacker, SkillName::Tactics), 0.625, 100.0, 6.25);
    let lumber_bonus = if ty == WeaponType::Axe {
        get_bonus(
            val(w, attacker, SkillName::Lumberjacking),
            0.200,
            100.0,
            10.00,
        )
    } else {
        0.0
    };

    // The following are damage modifiers whose effect shows on the status bar. Capped at 100% total.
    let mut dmg_bonus = aos_attr(w, attacker, AosAttribute::WeaponDamage);

    // Horrific Beast transformation gives a +25% bonus to damage.
    if hk::under_horrific_beast(w, attacker) {
        dmg_bonus += 25;
    }

    // Divine Fury gives a +10% bonus to damage.
    dmg_bonus += hk::divine_fury_damage_bonus(w, attacker);

    // Defense Mastery gives a -50%/-80% malus to damage.
    if let Some(m) = wa::defense_mastery_malus(w, attacker) {
        dmg_bonus -= m;
    }

    // Discordance gives a -2%/-48% malus to damage.
    if let Some(e) = hk::discordance_effect(w, attacker) {
        dmg_bonus -= e * 2;
    }

    if dmg_bonus > 100 {
        dmg_bonus = 100;
    }

    let total_bonus = strength_bonus
        + anatomy_bonus
        + tactics_bonus
        + lumber_bonus
        + f64::from(dmg_bonus + damage_bonus(w, id)) / 100.0;

    damage + damage * total_bonus
}

/// `ComputeDamageAOS(attacker, defender)`.
pub fn compute_damage_aos(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    _defender: MobileId,
) -> i32 {
    let base = get_base_damage(w, id, attacker);
    scale_damage_aos(w, id, attacker, base, true) as i32
}

/// `ScaleDamageOld(attacker, damage, checkSkills)`.
pub fn scale_damage_old(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    mut damage: f64,
    check_skills: bool,
) -> f64 {
    let ty = weapon::weapon_type(w, id);
    let uor = super::core_uor(w);
    if check_skills {
        // Passively check tactics for gain
        let cap = skills::cap(w, attacker, SkillName::Tactics);
        muo_server::mobile::part2::stats::check_skill(w, attacker, SkillName::Tactics, 0.0, cap);

        // Passively check Anatomy for gain
        let cap = skills::cap(w, attacker, SkillName::Anatomy);
        muo_server::mobile::part2::stats::check_skill(w, attacker, SkillName::Anatomy, 0.0, cap);

        if uor && ty == WeaponType::Axe {
            // Passively check Lumberjacking for gain
            muo_server::mobile::part2::stats::check_skill(
                w,
                attacker,
                SkillName::Lumberjacking,
                0.0,
                100.0,
            );
        }
    }

    // Compute tactics modifier: 0.0 = 50% loss, 50.0 = unchanged, 100.0 = 50% bonus
    damage += damage * ((val(w, attacker, SkillName::Tactics) - 50.0) / 100.0);

    // Compute strength modifier: 1% bonus for every 5 strength
    let mut modifiers = f64::from(mb::str(w, attacker)) / 5.0 / 100.0;

    // Compute anatomy modifier: 1% bonus for every 5 points, +10% at Grandmaster or higher (UOTD+)
    let anatomy_value = val(w, attacker, SkillName::Anatomy);
    modifiers += anatomy_value / 5.0 / 100.0;

    if super::core_uotd(w) && anatomy_value >= 100.0 {
        modifiers += 0.1;
    }

    // Compute lumberjacking bonus (UOR+): 1% for every 5 points, +10% at Grandmaster or higher (UOTD+)
    if uor && ty == WeaponType::Axe {
        let lumber_value = val(w, attacker, SkillName::Lumberjacking);

        modifiers += lumber_value / 5.0 / 100.0;

        if super::core_uotd(w) && lumber_value >= 100.0 {
            modifiers += 0.1;
        }
    }

    // New quality bonus:
    let quality = data(w, id).map_or(WeaponQuality::Regular, |d| d.quality);
    if quality != WeaponQuality::Regular {
        modifiers += f64::from(quality as i32 - 1) * 0.2;
    }

    // Virtual damage bonus:
    let vdb = weapon::class(w, id).virtual_damage_bonus;
    if vdb != 0 {
        modifiers += f64::from(vdb) / 100.0;
    }

    // Apply bonuses
    damage += damage * modifiers;

    f64::from(scale_damage_by_durability(w, id, damage as i32))
}

/// `ScaleDamageByDurability(damage)`.
pub fn scale_damage_by_durability(w: &World, id: ItemId, damage: i32) -> i32 {
    let mut scale = 100;
    if let Some(d) = data(w, id)
        && d.max_hit_points > 0
        && d.hit_points < d.max_hit_points
    {
        scale = 50 + 50 * d.hit_points / d.max_hit_points;
    }
    aos_helpers::scale(damage, scale)
}

/// `ComputeDamage(attacker, defender)`.
pub fn compute_damage(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) -> i32 {
    if core::aos(w) {
        return compute_damage_aos(w, id, attacker, defender);
    }

    let base = get_base_damage(w, id, attacker);
    let mut damage = scale_damage_old(w, id, attacker, base, true) as i32;

    // pre-AOS, halve damage if the defender is a player or the attacker is not a player
    if is_player_mobile(w, defender) || !is_player_mobile(w, attacker) {
        damage /= 2;
    }

    damage
}

// ---------------------------------------------------------------------------------------------------------------
// Animations and sounds
// ---------------------------------------------------------------------------------------------------------------

fn body_type(w: &World, m: MobileId) -> muo_primitives::body::BodyType {
    let Some(_) = w.mobile(m) else {
        return muo_primitives::body::BodyType::Empty;
    };
    w.resources
        .get::<muo_primitives::BodyTable>()
        .map_or(muo_primitives::body::BodyType::Empty, |t| {
            t.type_of(muo_server::mobile::part1::props_b::body(w, m))
        })
}

/// `PlayHurtAnimation(from)`.
pub fn play_hurt_animation(w: &mut World, from: MobileId) {
    use muo_primitives::body::BodyType as B;
    if props_b::mounted(w, from) {
        return;
    }
    let (action, frames) = match body_type(w, from) {
        B::Sea | B::Animal => (7, 5),
        B::Monster => (10, 4),
        B::Human => (20, 5),
        _ => return,
    };
    mb::animate(w, from, action, frames, 1, true, false, 0);
}

/// `PlaySwingAnimation(from)`.
pub fn play_swing_animation(w: &mut World, id: ItemId, from: MobileId) {
    use muo_primitives::body::BodyType as B;
    let animation = weapon::animation(w, id);
    let action = match body_type(w, from) {
        B::Sea | B::Animal => utility::random_from(&mut w.rng, 5, 2),
        B::Monster => match animation {
            WeaponAnimation::ShootBow | WeaponAnimation::ShootXBow => return,
            _ => utility::random_from(&mut w.rng, 4, 3),
        },
        B::Human => {
            if !props_b::mounted(w, from) {
                animation as i32
            } else {
                match animation {
                    WeaponAnimation::Wrestle
                    | WeaponAnimation::Bash1H
                    | WeaponAnimation::Pierce1H
                    | WeaponAnimation::Slash1H => 26,
                    WeaponAnimation::Bash2H
                    | WeaponAnimation::Pierce2H
                    | WeaponAnimation::Slash2H => 29,
                    WeaponAnimation::ShootBow => 27,
                    WeaponAnimation::ShootXBow => 28,
                    _ => 26,
                }
            }
        }
        _ => return,
    };
    mb::animate(w, from, action, 7, 1, true, false, 0);
}

/// `GetElementalDamageHue()`.
pub fn elemental_damage_hue(w: &World, id: ItemId) -> i32 {
    let t = get_damage_types(w, id, None);
    let (fire, cold, pois, nrgy) = (t.fire, t.cold, t.pois, t.nrgy);

    let mut current_max = 50;
    let mut hue = 0;

    // Order is Cold, Energy, Fire, Poison, Physical
    if pois >= current_max {
        hue = 1267 + (pois - 50) / 10;
        current_max = pois;
    }
    if fire >= current_max {
        hue = 1255 + (fire - 50) / 10;
        current_max = fire;
    }
    if nrgy >= current_max {
        hue = 1273 + (nrgy - 50) / 10;
        current_max = nrgy;
    }
    if cold >= current_max {
        hue = 1261 + (cold - 50) / 10;
    }

    hue
}

/// `GetHitAttackSound(attacker, defender)`.
pub fn get_hit_attack_sound(
    w: &mut World,
    id: ItemId,
    attacker: MobileId,
    _defender: MobileId,
) -> i32 {
    let sound = w
        .mobile(attacker)
        .map(|m| m.ty)
        .map_or(-1, |t| (t.get_attack_sound)(w, attacker));
    if sound == -1 {
        weapon::hit_sound(w, id)
    } else {
        sound
    }
}

/// `GetHitDefendSound(attacker, defender)`.
pub fn get_hit_defend_sound(w: &mut World, defender: MobileId) -> i32 {
    w.mobile(defender)
        .map(|m| m.ty)
        .map_or(-1, |t| (t.get_hurt_sound)(w, defender))
}

/// `GetMissAttackSound(attacker, defender)`.
pub fn get_miss_attack_sound(w: &mut World, id: ItemId, attacker: MobileId) -> i32 {
    let s = w
        .mobile(attacker)
        .map(|m| m.ty)
        .map_or(-1, |t| (t.get_attack_sound)(w, attacker));
    if s == -1 {
        weapon::miss_sound(w, id)
    } else {
        -1
    }
}

/// `GetMissDefendSound(attacker, defender)`.
pub fn get_miss_defend_sound() -> i32 {
    -1
}

// ---------------------------------------------------------------------------------------------------------------
// Hit effects
// ---------------------------------------------------------------------------------------------------------------

/// `DoMagicArrow(attacker, defender)`.
pub fn do_magic_arrow(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) {
    if !mb::can_be_harmful_msg(w, attacker, defender, false) {
        return;
    }
    mb::do_harmful(w, attacker, defender, false);

    let damage = get_aos_damage(w, attacker, 10, 1, 4);

    fx::moving_particles(
        w,
        attacker,
        EntityId::Mobile(defender),
        0x36E4,
        5,
        0,
        false,
        true,
        3006,
        4006,
        0,
        0,
    );
    fx::play_sound(w, attacker, 0x1E5);

    let _ = id;
    hk::spell_helper_damage(
        w,
        TimeSpan::from_secs(1),
        defender,
        attacker,
        damage,
        [0, 100, 0, 0, 0],
    );
}

/// `DoHarm(attacker, defender)`.
pub fn do_harm(w: &mut World, _id: ItemId, attacker: MobileId, defender: MobileId) {
    if !mb::can_be_harmful_msg(w, attacker, defender, false) {
        return;
    }
    mb::do_harmful(w, attacker, defender, false);

    let mut damage = get_aos_damage(w, attacker, 17, 1, 5);

    let dloc = point(w, attacker);
    if !mb::in_range_point3d(w, defender, dloc, 2) {
        damage *= 0.25; // 1/4 damage at > 2 tile range
    } else if !mb::in_range_point3d(w, defender, dloc, 1) {
        damage *= 0.50; // 1/2 damage at 2 tile range
    }

    fx::fixed_particles_hue(
        w,
        defender,
        0x374A,
        10,
        30,
        5013,
        1153,
        2,
        EffectLayer::Waist,
    );
    fx::play_sound(w, defender, 0x0FC);

    hk::spell_helper_damage(
        w,
        TimeSpan::ZERO,
        defender,
        attacker,
        damage,
        [0, 0, 100, 0, 0],
    );
}

/// `DoFireball(attacker, defender)`.
pub fn do_fireball(w: &mut World, _id: ItemId, attacker: MobileId, defender: MobileId) {
    if !mb::can_be_harmful_msg(w, attacker, defender, false) {
        return;
    }
    mb::do_harmful(w, attacker, defender, false);

    let damage = get_aos_damage(w, attacker, 19, 1, 5);

    fx::moving_particles(
        w,
        attacker,
        EntityId::Mobile(defender),
        0x36D4,
        7,
        0,
        false,
        true,
        9502,
        4019,
        0x160,
        0,
    );
    fx::play_sound(w, attacker, 0x15E);

    hk::spell_helper_damage(
        w,
        TimeSpan::from_secs(1),
        defender,
        attacker,
        damage,
        [0, 100, 0, 0, 0],
    );
}

/// `DoLightning(attacker, defender)`.
pub fn do_lightning(w: &mut World, _id: ItemId, attacker: MobileId, defender: MobileId) {
    if !mb::can_be_harmful_msg(w, attacker, defender, false) {
        return;
    }
    mb::do_harmful(w, attacker, defender, false);

    let damage = get_aos_damage(w, attacker, 23, 1, 4);

    fx::bolt_effect(w, defender, 0);

    hk::spell_helper_damage(
        w,
        TimeSpan::ZERO,
        defender,
        attacker,
        damage,
        [0, 0, 0, 0, 100],
    );
}

/// `DoDispel(attacker, defender)`.
pub fn do_dispel(w: &mut World, attacker: MobileId, defender: MobileId) {
    let dispellable = is_creature(w, defender)
        && crate::creature::vtable::ctype(w, defender).is_some_and(|_| {
            cd(w, defender).is_some_and(|c| c.summoned)
                && !crate::creature::misc::is_animated_dead(w, defender)
        });

    if !dispellable {
        return;
    }
    if !mb::can_be_harmful_msg(w, attacker, defender, false) {
        return;
    }
    mb::do_harmful(w, attacker, defender, false);

    if hk::dispel_check_resisted(w, attacker, defender) {
        fx::fixed_effect(w, defender, 0x3779, 10, 20);
    } else {
        if let Some((loc, map)) = w.mobile(defender).map(|m| (m.location, m.map)) {
            let item = muo_server::item::effect_item::create(
                w,
                loc,
                map,
                muo_server::item::effect_item::DEFAULT_DURATION,
            );
            muo_server::effects::send_location_particles(w, item, 0x3728, 8, 20, 5042);
            muo_server::effects::play_sound_entity(w, EntityId::Mobile(defender), 0x201);
        }
        if let Some(ty) = w.mobile(defender).map(|m| m.ty) {
            (ty.delete)(w, defender);
        }
    }
}

/// `DoLowerAttack(from, defender)`.
pub fn do_lower_attack(w: &mut World, defender: MobileId) {
    if wa::hit_lower_apply_attack(w, defender) {
        fx::play_sound(w, defender, 0x28E);
        muo_server::effects::send_target_effect(
            w,
            EntityId::Mobile(defender),
            0x37BE,
            1,
            4,
            0xA,
            3,
        );
    }
}

/// `DoLowerDefense(from, defender)`.
pub fn do_lower_defense(w: &mut World, defender: MobileId) {
    if wa::hit_lower_apply_defense(w, defender) {
        fx::play_sound(w, defender, 0x28E);
        muo_server::effects::send_target_effect(
            w,
            EntityId::Mobile(defender),
            0x37BE,
            1,
            4,
            0x23,
            3,
        );
    }
}

/// `DoAreaAttack(from, defender, sound, hue, phys, fire, cold, pois, nrgy)`; `split` is `[phys, fire, cold, pois,
/// nrgy]`.
pub fn do_area_attack(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    defender: MobileId,
    sound: i32,
    hue: i32,
    split: [i32; 5],
) {
    let Some((map, loc)) = w.mobile(from).map(|m| (m.map, m.location)) else {
        return;
    };
    let Some(map) = map else { return };

    let range = if core::ml(w) { 5 } else { 10 };
    let mut queue = Vec::new();
    for m in muo_server::mobile::part2::enumerators::get_mobiles_in_range(w, from, range) {
        if from != m
            && defender != m
            && hk::spell_helper_valid_indirect_target(w, from, m)
            && mb::can_be_harmful_msg(w, from, m, false)
            && (!core::ml(w) || in_los(w, from, m))
        {
            queue.push(m);
        }
    }

    if queue.is_empty() {
        return;
    }

    muo_server::effects::play_sound(w, loc, Some(map), sound);

    for m in queue {
        let dist = distance_sqrt(point(w, from), point(w, m));
        let scalar = if core::ml(w) {
            1.0
        } else {
            (11.0 - dist) / 10.0
        };

        if scalar <= 0.0 {
            continue;
        }

        let mut damage = get_base_damage(w, id, from);

        if scalar < 1.0 {
            damage *= (11.0 - dist) / 10.0;
        }

        mb::do_harmful(w, from, m, true);
        fx::fixed_effect_hue(w, m, 0x3779, 1, 15, hue, 0);
        aos_helpers::damage(
            w,
            m,
            Some(from),
            damage as i32,
            DamageSplit {
                phys: split[0],
                fire: split[1],
                cold: split[2],
                pois: split[3],
                nrgy: split[4],
            },
            DamageOptions::default(),
        );
    }
}

/// `GetDistanceToSqrt(m)` (`Math.Sqrt(dx*dx + dy*dy)`).
fn distance_sqrt(a: Point3D, b: Point3D) -> f64 {
    let (dx, dy) = (f64::from(a.x - b.x), f64::from(a.y - b.y));
    (dx * dx + dy * dy).sqrt()
}

// ---------------------------------------------------------------------------------------------------------------
// Equip / unequip
// ---------------------------------------------------------------------------------------------------------------

/// The skill mods of a worn weapon (`m_SkillMod`, `m_MageMod`): remove the old ones.
fn remove_mod(w: &mut World, which: Option<(muo_server::mods::ModId, MobileId)>) {
    if let Some((mod_id, owner)) = which
        && let Some(ty) = w.mobile(owner).map(|m| m.ty)
    {
        (ty.remove_skill_mod)(w, owner, mod_id);
    }
}

fn add_default_skill_mod(
    w: &mut World,
    to: MobileId,
    skill: SkillName,
    name: &str,
    value: f64,
) -> Option<(muo_server::mods::ModId, MobileId)> {
    let sk = muo_server::mods::SkillMod::new_default(w, skill, Some(name), true, value);
    let id = sk.id;
    let ty = w.mobile(to).map(|m| m.ty)?;
    (ty.add_skill_mod)(w, to, sk);
    Some((id, to))
}

/// The accuracy skill mod of the weapon for its accuracy level (`new DefaultSkillMod(AccuracySkill,
/// "WeaponAccuracy", true, (int)_accuracyLevel * 5)`).
pub fn add_accuracy_mod(w: &mut World, id: ItemId, to: MobileId) {
    let level = data(w, id).map_or(0, |d| d.accuracy_level as i32);
    let skill = weapon::class(w, id).accuracy_skill;
    let m = add_default_skill_mod(w, to, skill, "WeaponAccuracy", f64::from(level * 5));
    if let Some(d) = data_mut(w, id) {
        d.skill_mod = m;
    }
}

/// The mage-weapon skill mod (`new DefaultSkillMod(SkillName.Magery, "MageWeapon", true, -30 + MageWeapon)`).
pub fn add_mage_mod(w: &mut World, id: ItemId, to: MobileId) {
    let v = weapon_attr(w, id, AosWeaponAttribute::MageWeapon);
    let m = add_default_skill_mod(w, to, SkillName::Magery, "MageWeapon", f64::from(-30 + v));
    if let Some(d) = data_mut(w, id) {
        d.mage_mod = m;
    }
}

/// `OnEquip(from)`.
pub fn on_equip(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let (s, d, i) = (
        super::attr(w, id, AttrKind::Attributes, AosAttribute::BonusStr as u32),
        super::attr(w, id, AttrKind::Attributes, AosAttribute::BonusDex as u32),
        super::attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32),
    );
    if s != 0 || d != 0 || i != 0 {
        super::add_stat_mods(w, id, from, s, d, i);
    }

    if !enable_insta_hit(w) {
        let ms = get_delay(w, id, from).total_milliseconds() as i64;
        let tick = w.tick_count;
        if let Some(m) = w.mobile_mut(from) {
            m.next_combat_time = tick + i64::from(ms as i32);
        }
    }

    let level = data(w, id).map_or(weapon::WeaponAccuracyLevel::Regular, |d| d.accuracy_level);
    if use_skill_mod(w) && level != weapon::WeaponAccuracyLevel::Regular {
        let old = data_mut(w, id).and_then(|d| d.skill_mod.take());
        remove_mod(w, old);
        add_accuracy_mod(w, id, from);
    }

    let mage = weapon_attr(w, id, AosWeaponAttribute::MageWeapon);
    if core::aos(w) && mage != 0 && mage != 30 {
        let old = data_mut(w, id).and_then(|d| d.mage_mod.take());
        remove_mod(w, old);
        add_mage_mod(w, id, from);
    }

    true
}

/// `OnAdded(parent)`.
pub fn on_added(w: &mut World, id: ItemId, parent: EntityId) {
    muo_server::item::base::on_added_base(w, id, parent);

    if let EntityId::Mobile(from) = parent {
        if core::aos(w) {
            super::skill_bonuses_add_to(w, id, from);
        }
        if let Some(ty) = w.mobile(from).map(|m| m.ty) {
            (ty.check_stat_timers)(w, from);
            (ty.delta)(w, from, MobileDelta::WEAPON_DAMAGE);
        }
    }
}

/// `OnRemoved(parent)`.
pub fn on_removed(w: &mut World, id: ItemId, parent: EntityId) {
    let EntityId::Mobile(m) = parent else { return };

    super::remove_stat_mods(w, id, m);

    if !enable_insta_hit(w)
        && let Some(wp) = wielded(w, m)
        && weapon::is_weapon(w, wp)
    {
        let ms = get_delay(w, wp, m).total_milliseconds() as i64;
        let tick = w.tick_count;
        if let Some(mm) = w.mobile_mut(m) {
            mm.next_combat_time = tick + ms;
        }
    }

    if use_skill_mod(w) {
        let old = data_mut(w, id).and_then(|d| d.skill_mod.take());
        remove_mod(w, old);
    }

    let old = data_mut(w, id).and_then(|d| d.mage_mod.take());
    remove_mod(w, old);

    if core::aos(w) {
        super::skill_bonuses_remove(w, id);
    }

    hk::immolating_stop(w, id);
    wa::force_of_nature_remove(w, m);

    if let Some(ty) = w.mobile(m).map(|x| x.ty) {
        (ty.check_stat_timers)(w, m);
        (ty.delta)(w, m, MobileDelta::WEAPON_DAMAGE);
    }
}

/// `BlockEquip(m, duration)`.
pub fn block_equip(w: &mut World, m: MobileId, duration: TimeSpan) {
    if muo_server::mobile::part1::combat::begin_action(w, m, "Server.Items.BaseWeapon") {
        muo_server::timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
            muo_server::mobile::part1::combat::end_action(w, m, "Server.Items.BaseWeapon");
        });
    }
}

/// `CanEquip(from)`.
pub fn can_equip(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let (is_pl, level) = w
        .mobile(from)
        .map_or((false, muo_server::mobile::AccessLevel::Player), |m| {
            (m.player, m.access_level)
        });
    let can_begin = |w: &World| {
        muo_server::mobile::part1::combat::can_begin_action(w, from, "Server.Items.BaseWeapon")
    };

    if !is_pl || level >= muo_server::mobile::AccessLevel::GameMaster {
        return can_begin(w) && muo_server::item::base::can_equip_base(w, id, from);
    }

    if let Some(f) = super::equip_hooks::hooks(w).and_then(|h| h.ethic_check_equip)
        && !f(w, from, id)
    {
        return false;
    }

    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return false;
    };
    if !(ty.check_race)(w, id, from, true) {
        return false;
    }

    if mb::dex(w, from) < weapon::dex_requirement(w, id) {
        muo_server::mobile::part2::messages::send_message(
            w,
            from,
            "You are not nimble enough to equip that.",
        );
        return false;
    }

    let str_req = aos_helpers::scale(
        weapon::str_requirement(w, id),
        100 - weapon::lower_stat_req(w, id),
    );
    if mb::str(w, from) < str_req {
        msg(w, from, 500213); // You are not strong enough to equip that.
        return false;
    }

    if mb::int(w, from) < weapon::int_requirement(w, id) {
        muo_server::mobile::part2::messages::send_message(
            w,
            from,
            "You are not smart enough to equip that.",
        );
        return false;
    }

    can_begin(w) && muo_server::item::base::can_equip_base(w, id, from)
}

/// `AllowSecureTrade(from, to, newOwner, accepted)`.
pub fn allow_secure_trade(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    to: MobileId,
    new_owner: MobileId,
    accepted: bool,
) -> bool {
    let ethic = super::equip_hooks::hooks(w)
        .and_then(|h| h.ethic_check_trade)
        .is_none_or(|f| f(w, from, to, new_owner, id));
    ethic && muo_server::item::base::allow_secure_trade_base(w, id, from, to, new_owner, accepted)
}

/// `AllowEquippedCast(from)`.
pub fn allow_equipped_cast(w: &World, id: ItemId, from: MobileId) -> bool {
    muo_server::item::base::allow_equipped_cast_base(w, id, from)
        || super::attr(
            w,
            id,
            AttrKind::Attributes,
            AosAttribute::SpellChanneling as u32,
        ) != 0
}

/// `Direction`-free helper kept for the abilities: the weapon's `MaxRange` test of `DoubleStrike`.
pub fn in_weapon_range(w: &mut World, id: ItemId, attacker: MobileId, defender: MobileId) -> bool {
    let range = weapon::max_range(w, id);
    combat_distance_ok(w, attacker, defender, range)
}

#[allow(dead_code)]
fn _unused(_: Direction) {}
