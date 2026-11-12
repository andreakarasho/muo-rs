//! Backstab, Death Strike, Focus Attack, Ki Attack and Surprise Attack: the five Ninjitsu special moves. A subclass's `Validate` runs as the move's `pre_validate`.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::items::armor::BASE_SHIELD;
use muo_content::items::weapon::{BASE_RANGED, BASE_WEAPON};
use muo_content::items::weapon_hooks::{self, WeaponHooks};
use muo_content::misc::aos_helpers::{self, DamageOptions, DamageSplit};
use muo_content::skills::tracking;
use muo_content::spells::special_move::{
    AbilityMessage, SpecialMoveType, check_mana, clear_current_move, validate,
};
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::{ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::mobile::part1::combat;
use muo_server::mobile::part2::{damage, effects, items, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use super::ninja::{NINJA_MOVE, ninja_check_gain};
use crate::common::revealing_action;

fn layer_item_is(
    w: &World,
    m: MobileId,
    layer: Layer,
    ty: &'static muo_server::item::ItemType,
) -> Option<ItemId> {
    items::find_item_on_layer(w, m, layer).filter(|&i| w.item(i).is_some_and(|x| x.ty.is(ty)))
}

fn hidden_with_steps(w: &World, from: MobileId) -> bool {
    w.mobile(from)
        .is_some_and(|m| m.hidden && m.allowed_stealth_steps > 0)
}

/// The `OnBeforeSwing` of Backstab and Surprise Attack: `Validate && CheckMana(true)`, then `BeginAction<Stealth>`.
fn stealth_before_swing(w: &mut World, ty: &'static SpecialMoveType, attacker: MobileId) -> bool {
    let valid = validate(w, ty, attacker) && check_mana(w, ty, attacker, true);
    if valid {
        combat::begin_action(w, attacker, "Stealth");
        timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
            combat::end_action(w, attacker, "Stealth")
        });
    }
    valid
}

fn stealth_required(w: &mut World, from: MobileId) -> bool {
    if !hidden_with_steps(w, from) {
        mm::send_localized_message(w, from, 1063087, "", 0x3B2); // You must be in stealth mode to use this ability.
        return false;
    }
    true
}

// ---- Backstab ----

/// `Backstab`.
pub static BACKSTAB: SpecialMoveType = SpecialMoveType {
    full_name: "Server.Spells.Ninjitsu.Backstab",
    base_mana: 30,
    required_skill: 40.0,
    required_skill_pre_ml: Some(20.0),
    ability_message: AbilityMessage::Number(1063089), // You prepare to Backstab your opponent.
    validates_during_hit: false,
    pre_validate: stealth_required,
    get_damage_scalar: |w, attacker, defender| {
        let ninjitsu = skills::value(w, attacker, SkillName::Ninjitsu);
        1.0 + ninjitsu / 360.0 + tracking::get_stalking_bonus(w, attacker, defender) / 100.0
    },
    on_before_swing: |w, attacker, _| stealth_before_swing(w, &BACKSTAB, attacker),
    on_hit: |w, attacker, defender, _| {
        // Validates before swing
        clear_current_move(w, attacker);

        mm::send_localized_message(w, attacker, 1063090, "", 0x3B2); // You quickly stab your opponent as you come out of hiding!

        effects::fixed_particles_hue(
            w,
            defender,
            0x37B9,
            1,
            5,
            0x251D,
            0x651,
            0,
            EffectLayer::Waist,
        );

        revealing_action(w, attacker);
        ninja_check_gain(w, &BACKSTAB, attacker);
    },
    on_miss: |w, attacker, _| {
        clear_current_move(w, attacker);

        mm::send_localized_message(w, attacker, 1063161, "", 0x3B2); // You failed to properly use the element of surprise.
        revealing_action(w, attacker);
    },
    ..NINJA_MOVE
};

// ---- DeathStrike ----

struct DsEntry {
    timer: TimerHandle,
    attacker: MobileId,
    damage_bonus: i32,
    is_ranged: bool,
    steps: i32,
}

#[derive(Default)]
struct DsTable(IndexMap<MobileId, DsEntry>);

fn ds_table(w: &mut World) -> &mut IndexMap<MobileId, DsEntry> {
    &mut w.resources.get_mut::<DsTable>().0
}

/// `DeathStrike`.
pub static DEATH_STRIKE: SpecialMoveType = SpecialMoveType {
    full_name: "Server.Spells.Ninjitsu.DeathStrike",
    base_mana: 30,
    required_skill: 85.0,
    ability_message: AbilityMessage::Number(1063091), // You prepare to hit your opponent with a Death Strike.
    get_damage_scalar: |_, _, _| 0.5,
    on_hit: ds_on_hit,
    ..NINJA_MOVE
};

fn ds_on_hit(w: &mut World, attacker: MobileId, defender: MobileId, _damage: i32) {
    if !validate(w, &DEATH_STRIKE, attacker) || !check_mana(w, &DEATH_STRIKE, attacker, true) {
        return;
    }

    clear_current_move(w, attacker);

    let ninjitsu = skills::value(w, attacker, SkillName::Ninjitsu);

    // TODO: should be defined onHit method, what if the player hit and remove the weapon before process? ;)
    let is_ranged = weapon_hooks::weapon_item(w, attacker)
        .is_some_and(|i| w.item(i).is_some_and(|x| x.ty.is(&BASE_RANGED)));

    // This formula is an approximation from OSI data.  TODO: find correct formula
    let chance = if ninjitsu < 100.0 {
        30.0 + (ninjitsu - 85.0) * 2.2
    } else {
        63.0 + (ninjitsu - 100.0) * 1.1
    };

    if chance / 100.0 < utility::random_double(&mut w.rng) {
        mm::send_localized_message(w, attacker, 1070779, "", 0x3B2); // You missed your opponent with a Death Strike.
        return;
    }

    let mut damage_bonus = 0;

    if let Some(mut old) = ds_table(w).shift_remove(&defender) {
        mm::send_localized_message(w, defender, 1063092, "", 0x3B2); // Your opponent lands another Death Strike!

        if old.steps > 0 {
            damage_bonus = (skills::value(w, attacker, SkillName::Ninjitsu) / 15.0) as i32;
        }

        old.timer.cancel(w);
    } else {
        mm::send_localized_message(w, defender, 1063093, "", 0x3B2); // You have been hit by a Death Strike!  Move with caution!
    }

    mm::send_localized_message(w, attacker, 1063094, "", 0x3B2); // You inflict a Death Strike upon your opponent!

    effects::fixed_particles(w, defender, 0x374A, 1, 17, 0x26BC, EffectLayer::Waist);
    let female = w.mobile(attacker).is_some_and(|a| a.female);
    effects::play_sound(w, attacker, if female { 0x50D } else { 0x50E });

    let h = timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
        process_death_strike(w, defender)
    });
    ds_table(w).insert(
        defender,
        DsEntry {
            timer: h,
            attacker,
            damage_bonus,
            is_ranged,
            steps: 0,
        },
    );

    ninja_check_gain(w, &DEATH_STRIKE, attacker);
}

/// `AddStep(m)`.
pub fn add_step(w: &mut World, m: MobileId) {
    let process = match ds_table(w).get_mut(&m) {
        Some(e) => {
            e.steps += 1;
            e.steps >= 5
        }
        None => false,
    };
    if process {
        process_death_strike(w, m);
    }
}

/// `RemoveEffect(m)`.
pub fn remove_effect(w: &mut World, m: MobileId) {
    if let Some(mut e) = ds_table(w).shift_remove(&m) {
        e.timer.cancel(w);
    }
}

/// `DeathStrikeTimer.ProcessDeathStrike()`.
fn process_death_strike(w: &mut World, target: MobileId) {
    let Some(e) = ds_table(w).get(&target) else {
        return;
    };
    let (attacker, steps, damage_bonus, is_ranged) =
        (e.attacker, e.steps, e.damage_bonus, e.is_ranged);

    let ninjitsu = skills::value(w, attacker, SkillName::Ninjitsu);
    let stalking_bonus = tracking::get_stalking_bonus(w, attacker, target);

    if core::ml(w) {
        let scalar = ((skills::value(w, attacker, SkillName::Hiding)
            + skills::value(w, attacker, SkillName::Stealth))
            / 220.0)
            .min(1.0);

        // New formula doesn't apply DamageBonus anymore, caps must be, directly, 60/30.
        let mut dmg = if steps >= 5 {
            (60.0_f64.min(ninjitsu / 3.0 * (0.3 + 0.7 * scalar) + stalking_bonus)).floor() as i32
        } else {
            (30.0_f64.min(ninjitsu / 9.0 * (0.3 + 0.7 * scalar) + stalking_bonus)).floor() as i32
        };

        if is_ranged {
            dmg /= 2;
        }

        damage::damage(w, target, dmg, Some(attacker), true, false); // Damage is direct.
    } else {
        let divisor = if steps >= 5 { 30.0 } else { 80.0 };
        let base_damage = ninjitsu / divisor * 10.0;
        let max_damage = if steps >= 5 { 62 } else { 22 };

        let dmg = ((base_damage + stalking_bonus) as i32).clamp(0, max_damage) + damage_bonus;

        // Damage is physical.
        aos_helpers::damage(
            w,
            target,
            Some(attacker),
            dmg,
            DamageSplit {
                phys: 100,
                ..Default::default()
            },
            DamageOptions {
                ignore_armor: true,
                death_strike: true,
                ..Default::default()
            },
        );
    }

    remove_effect(w, target);
}

// ---- FocusAttack ----

/// `FocusAttack`.
pub static FOCUS_ATTACK: SpecialMoveType = SpecialMoveType {
    full_name: "Server.Spells.Ninjitsu.FocusAttack",
    base_mana: 10,
    base_mana_pre_ml: Some(20),
    required_skill: 30.0,
    required_skill_pre_ml: Some(60.0),
    ability_message: AbilityMessage::Number(1063095), // You prepare to focus all of your abilities into your next strike.
    pre_validate: |w, from| {
        if layer_item_is(w, from, Layer::TwoHanded, &BASE_SHIELD).is_some() {
            mm::send_localized_message(w, from, 1063096, "", 0x3B2); // You cannot use this ability while holding a shield.
            return false;
        }

        let melee = |w: &World, layer| {
            items::find_item_on_layer(w, from, layer).is_some_and(|i| {
                w.item(i)
                    .is_some_and(|x| x.ty.is(&BASE_WEAPON) && !x.ty.is(&BASE_RANGED))
            })
        };
        let melee_weapon = melee(w, Layer::TwoHanded) || melee(w, Layer::OneHanded);

        if melee_weapon {
            return true;
        }

        mm::send_localized_message(w, from, 1063097, "", 0x3B2); // You must be wielding a melee weapon without a shield to use this ability.
        false
    },
    get_damage_scalar: |w, attacker, _| {
        let ninjitsu = skills::value(w, attacker, SkillName::Ninjitsu);
        1.0 + ninjitsu * ninjitsu / 43636.0
    },
    get_property_bonus: |w, attacker| {
        let ninjitsu = skills::value(w, attacker, SkillName::Ninjitsu);
        let bonus = ninjitsu * ninjitsu / 43636.0;
        1.0 + (bonus * 3.0 + 0.01)
    },
    on_before_damage: |w, attacker, _| {
        validate(w, &FOCUS_ATTACK, attacker) && check_mana(w, &FOCUS_ATTACK, attacker, true)
    },
    on_hit: |w, attacker, _, _| {
        clear_current_move(w, attacker);

        mm::send_localized_message(w, attacker, 1063098, "", 0x3B2); // You focus all of your abilities and strike with deadly force!
        effects::play_sound(w, attacker, 0x510);

        ninja_check_gain(w, &FOCUS_ATTACK, attacker);
    },
    ..NINJA_MOVE
};

// ---- KiAttack ----

struct KiEntry {
    timer: TimerHandle,
    location: Point3D,
}

#[derive(Default)]
struct KiTable(IndexMap<MobileId, KiEntry>);

fn ki_table(w: &mut World) -> &mut IndexMap<MobileId, KiEntry> {
    &mut w.resources.get_mut::<KiTable>().0
}

/// `KiAttack`.
pub static KI_ATTACK: SpecialMoveType = SpecialMoveType {
    full_name: "Server.Spells.Ninjitsu.KiAttack",
    base_mana: 25,
    required_skill: 80.0,
    ability_message: AbilityMessage::Number(1063099), // Your Ki Attack must be complete within 2 seconds for the damage bonus!
    pre_validate: |w, from| {
        if hidden_with_steps(w, from) {
            mm::send_localized_message(w, from, 1063127, "", 0x3B2); // You cannot use this ability while in stealth mode.
            return false;
        }

        if core::ml(w)
            && weapon_hooks::weapon_item(w, from)
                .is_some_and(|i| w.item(i).is_some_and(|x| x.ty.is(&BASE_RANGED)))
        {
            mm::send_localized_message(w, from, 1075858, "", 0x3B2); // You can only use this with melee attacks.
            return false;
        }

        true
    },
    on_use: |w, from| {
        if !validate(w, &KI_ATTACK, from) {
            return;
        }

        let location = w.mobile(from).map(|m| m.location).unwrap_or_default();
        let h = timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
            clear_current_move(w, from);
            mm::send_localized_message(w, from, 1063102, "", 0x3B2); // You failed to complete your Ki Attack in time.
            ki_table(w).shift_remove(&from);
        });
        ki_table(w).insert(from, KiEntry { timer: h, location });
    },
    get_damage_scalar: |w, attacker, defender| {
        if w.mobile(attacker).is_some_and(|a| a.hidden) {
            return 1.0;
        }

        /* Pub40 changed pvp damage max to 55% */
        let pvp = core::ml(w)
            && w.mobile(attacker).is_some_and(|a| a.player)
            && w.mobile(defender).is_some_and(|d| d.player);
        1.0 + ki_bonus(w, attacker) / if pvp { 40.0 } else { 10.0 }
    },
    on_hit: |w, attacker, defender, _| {
        if !validate(w, &KI_ATTACK, attacker) || !check_mana(w, &KI_ATTACK, attacker, true) {
            return;
        }

        clear_current_move(w, attacker);

        if ki_bonus(w, attacker) == 0.0 {
            mm::send_localized_message(w, attacker, 1063101, "", 0x3B2); // You were too close to your target to cause any additional damage.
        } else {
            effects::fixed_particles_hue(
                w,
                attacker,
                0x37BE,
                1,
                5,
                0x26BD,
                0x0,
                0x1,
                EffectLayer::Waist,
            );
            effects::play_sound(w, attacker, 0x510);

            // Your quick flight to your target causes extra damage as you strike!
            mm::send_localized_message(w, attacker, 1063100, "", 0x3B2);

            effects::fixed_particles_hue(
                w,
                defender,
                0x37BE,
                1,
                5,
                0x26BD,
                0,
                0x1,
                EffectLayer::Waist,
            );

            ninja_check_gain(w, &KI_ATTACK, attacker);
        }
    },
    on_clear_move: |w, from| {
        if let Some(mut e) = ki_table(w).shift_remove(&from) {
            e.timer.cancel(w);
        }
    },
    ..NINJA_MOVE
};

/// `GetBonus(from)`.
pub fn ki_bonus(w: &World, from: MobileId) -> f64 {
    let Some(e) = w.resources.get::<KiTable>().and_then(|t| t.0.get(&from)) else {
        return 0.0;
    };
    let Some(m) = w.mobile(from) else {
        return 0.0;
    };
    let x_delta = f64::from(e.location.x - m.location.x);
    let y_delta = f64::from(e.location.y - m.location.y);
    (x_delta * x_delta + y_delta * y_delta).sqrt().min(20.0)
}

// ---- SurpriseAttack ----

struct SaEntry {
    timer: TimerHandle,
    malus: i32,
}

#[derive(Default)]
struct SaTable(IndexMap<MobileId, SaEntry>);

fn sa_table(w: &mut World) -> &mut IndexMap<MobileId, SaEntry> {
    &mut w.resources.get_mut::<SaTable>().0
}

/// `SurpriseAttack`.
pub static SURPRISE_ATTACK: SpecialMoveType = SpecialMoveType {
    full_name: "Server.Spells.Ninjitsu.SurpriseAttack",
    base_mana: 20,
    required_skill: 60.0,
    required_skill_pre_ml: Some(30.0),
    ability_message: AbilityMessage::Number(1063128), // You prepare to surprise your prey.
    validates_during_hit: false,
    pre_validate: stealth_required,
    on_before_swing: |w, attacker, _| stealth_before_swing(w, &SURPRISE_ATTACK, attacker),
    on_hit: |w, attacker, defender, _| {
        // Validates before swing
        clear_current_move(w, attacker);

        mm::send_localized_message(w, attacker, 1063129, "", 0x3B2); // You catch your opponent off guard with your Surprise Attack!
        mm::send_localized_message(w, defender, 1063130, "", 0x3B2); // Your defenses are lowered as your opponent surprises you!

        effects::fixed_particles_hue(w, defender, 0x37B9, 1, 5, 0x26DA, 0, 3, EffectLayer::Head);

        revealing_action(w, attacker);

        stop_timer(w, defender);

        let ninjitsu = skills::fixed(w, attacker, SkillName::Ninjitsu);
        let malus = ninjitsu / 60 + tracking::get_stalking_bonus(w, attacker, defender) as i32;

        let h = timer::start(w, TimeSpan::from_secs(8), TimeSpan::ZERO, 1, move |w| {
            stop_timer(w, defender);
            mm::send_localized_message(w, defender, 1063131, "", 0x3B2); // Your defenses have returned to normal.
        });
        sa_table(w).insert(defender, SaEntry { timer: h, malus });

        ninja_check_gain(w, &SURPRISE_ATTACK, attacker);
    },
    on_miss: |w, attacker, _| {
        clear_current_move(w, attacker);

        mm::send_localized_message(w, attacker, 1063161, "", 0x3B2); // You failed to properly use the element of surprise.
        revealing_action(w, attacker);
    },
    ..NINJA_MOVE
};

/// `GetMalus(target, ref malus)`.
pub fn get_malus(w: &World, target: MobileId) -> Option<i32> {
    w.resources
        .get::<SaTable>()?
        .0
        .get(&target)
        .map(|e| e.malus)
}

fn stop_timer(w: &mut World, m: MobileId) {
    if let Some(mut e) = sa_table(w).shift_remove(&m) {
        e.timer.cancel(w);
    }
}

/// Sets the weapon hook.
pub fn init(w: &mut World) {
    let h = w.resources.get_mut::<WeaponHooks>();
    h.surprise_attack_malus = Some(get_malus);
}
