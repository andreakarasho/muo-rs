//! Small overrides of monsters and animals: damage alterations, scale types, mount abilities.

use muo_content::creature::cd;
use muo_content::creature::enums::ScaleType;
use muo_content::creature::vtable::BASE_CREATURE_C;
use muo_content::mx;
use muo_primitives::MessageType;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::{effects, messages};
use muo_server::skills::SkillName;

use crate::common::rnd;

// ---- ore elementals ----

/// `base.AlterMeleeDamageFrom(from, ref damage); damage /= 2;` (50% melee damage)
pub fn copper_elemental_alter_melee_damage_from(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    damage: &mut i32,
) {
    (BASE_CREATURE_C.alter_melee_damage_from)(w, id, from, damage);
    *damage /= 2;
}

macro_rules! reflect_all {
    ($name:ident) => {
        /// `reflect = true;` (every spell is reflected back to the caster)
        pub fn $name(_w: &mut World, _id: MobileId, _caster: MobileId, reflect: &mut bool) {
            *reflect = true;
        }
    };
}
reflect_all!(copper_elemental_check_reflect);
reflect_all!(valorite_elemental_check_reflect);

fn immune_to_pets(w: &World, id: MobileId, from: MobileId) -> bool {
    cd(w, from).is_some_and(|c| c.controlled || c.bard_target == Some(id))
}

/// Pets and provoked creatures do nothing, anything else does half.
pub fn valorite_elemental_alter_melee_damage_from(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    damage: &mut i32,
) {
    if immune_to_pets(w, id, from) {
        *damage = 0;
    } else {
        *damage /= 2;
    }
}

/// Pets and provoked creatures do nothing.
pub fn shadow_iron_elemental_alter_melee_damage_from(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    damage: &mut i32,
) {
    if immune_to_pets(w, id, from) {
        *damage = 0;
    }
}

/// `scalar = 0.0;` (immune to magic)
pub fn shadow_iron_elemental_alter_damage_scalar_from(
    _w: &mut World,
    _id: MobileId,
    _caster: MobileId,
    scalar: &mut f64,
) {
    *scalar = 0.0;
}

/// `damage = 0;`
pub fn shadow_iron_elemental_alter_spell_damage_from(
    _w: &mut World,
    _id: MobileId,
    _from: Option<MobileId>,
    damage: &mut i32,
) {
    *damage = 0;
}

// ---- solens: `SolenHelper.On<Colour>Damage(from)` first ----

macro_rules! solen_damage {
    ($name:ident, $helper:path) => {
        /// `SolenHelper.On<Colour>Damage(from); base.OnDamage(amount, from, willKill);`
        pub fn $name(
            w: &mut World,
            id: MobileId,
            amount: i32,
            from: Option<MobileId>,
            will_kill: bool,
        ) {
            if let Some(f) = from {
                $helper(w, f);
            }
            (muo_content::creature::BASE_CREATURE.on_damage)(w, id, amount, from, will_kill);
        }
    };
}
solen_damage!(
    black_solen_infiltrator_queen_on_damage,
    muo_content::creature::solen::on_black_damage
);
solen_damage!(
    black_solen_infiltrator_warrior_on_damage,
    muo_content::creature::solen::on_black_damage
);
solen_damage!(
    black_solen_worker_on_damage,
    muo_content::creature::solen::on_black_damage
);
solen_damage!(
    red_solen_infiltrator_queen_on_damage,
    muo_content::creature::solen::on_red_damage
);
solen_damage!(
    red_solen_infiltrator_warrior_on_damage,
    muo_content::creature::solen::on_red_damage
);
solen_damage!(
    red_solen_worker_on_damage,
    muo_content::creature::solen::on_red_damage
);

// ---- dragons ----

fn body_is(w: &World, id: MobileId, body: i32) -> bool {
    w.mobile(id).is_some_and(|m| m.cur_body().0 == body)
}

/// `Body == 12 ? ScaleType.Yellow : ScaleType.Red`
pub fn dragon_scale_type(w: &mut World, id: MobileId) -> ScaleType {
    if body_is(w, id, 12) {
        ScaleType::Yellow
    } else {
        ScaleType::Red
    }
}
/// `Body == 12 ? ScaleType.Yellow : ScaleType.Red`
pub fn greater_dragon_scale_type(w: &mut World, id: MobileId) -> ScaleType {
    dragon_scale_type(w, id)
}
/// `Body == 60 ? ScaleType.Yellow : ScaleType.Red`
pub fn drake_scale_type(w: &mut World, id: MobileId) -> ScaleType {
    if body_is(w, id, 60) {
        ScaleType::Yellow
    } else {
        ScaleType::Red
    }
}
/// `(ScaleType)Utility.Random(4)`
pub fn ancient_wyrm_scale_type(w: &mut World, _id: MobileId) -> ScaleType {
    match rnd::below(w, 4) {
        0 => ScaleType::Red,
        1 => ScaleType::Yellow,
        2 => ScaleType::Black,
        _ => ScaleType::Green,
    }
}

// ---- mounts ----

macro_rules! full_control {
    ($name:ident) => {
        /// `=> 1.0`
        pub fn $name(_w: &mut World, _id: MobileId, _m: MobileId, _use_base_skill: bool) -> f64 {
            1.0
        }
    };
}
full_control!(ridgeback_get_control_chance);
full_control!(savage_ridgeback_get_control_chance);
full_control!(scaled_swamp_dragon_get_control_chance);

/// `m.SendLocalizedMessage(1042319); // The Ki-Rin refuses your attempts to mount it.`
pub fn kirin_on_disallowed_rider(w: &mut World, _id: MobileId, m: MobileId) {
    mx::msg(w, m, 1042319);
}

/// `m.SendLocalizedMessage(1042318); // The unicorn refuses to allow you to ride it.`
pub fn unicorn_on_disallowed_rider(w: &mut World, _id: MobileId, m: MobileId) {
    mx::msg(w, m, 1042318);
}

pub fn kirin_do_mount_ability(
    w: &mut World,
    id: MobileId,
    damage: i32,
    attacker: Option<MobileId>,
) -> bool {
    use muo_content::creature::mount;
    let (Some(rider), Some(attacker)) = (mount::rider(w, id), attacker) else {
        return false; // sanity
    };
    let rhits = w.mobile(rider).map_or(0, |m| m.hits);
    let (rmap, amap) = (mx::map(w, rider), mx::map(w, attacker));
    // Range and map checked here instead of other base function because of abilities that don't need to check this
    if rhits - damage < 30 && rmap == amap && mx::in_range(w, rider, attacker, 18) {
        effects::bolt_effect(w, attacker, 0);
        // 35~100 damage, unresistable, by the Ki-rin; the mount is not told about it.
        let amount = rnd::min_max(w, 35, 100);
        muo_server::mobile::base::damage(w, attacker, amount, Some(id), false, false);
        // Your mount calls down the forces of nature on your opponent.
        messages::local_overhead_message_number(w, rider, MessageType::REGULAR, 0x3B2, 1042534, "");
        effects::fixed_particles(
            w,
            rider,
            0,
            0,
            0,
            0x13A7,
            muo_server::effects::EffectLayer::Waist,
        );
        effects::play_sound(w, rider, 0xA9); // Ki-rin whinny.
        return true;
    }
    false
}

pub fn unicorn_do_mount_ability(
    w: &mut World,
    id: MobileId,
    damage: i32,
    attacker: Option<MobileId>,
) -> bool {
    use muo_content::creature::mount;
    let (Some(rider), Some(_)) = (mount::rider(w, id), attacker) else {
        return false; // sanity
    };
    let (poison, hits) = w.mobile(rider).map_or((None, 0), |m| (m.poison, m.hits));
    if let Some(p) = poison.filter(|_| hits - damage < 40) {
        let level = muo_server::poison::level(w, p);
        let magery = muo_server::skills::value(w, id, SkillName::Magery);
        let mut chance = 10000 + (magery * 75.0) as i32
            - (level + 1)
                * if w.core_aos() {
                    if level < 4 { 3300 } else { 3100 }
                } else {
                    1750
                };
        chance /= 100;
        // `chanceToCure > Utility.Random(100)`: the roll happens first only when the chance is positive.
        if chance > rnd::below(w, 100) && muo_server::mobile::base::cure_poison(w, rider, Some(id))
        {
            // Your mount senses you are in danger and aids you with magic.
            messages::local_overhead_message_number(
                w,
                rider,
                MessageType::REGULAR,
                0x3B2,
                1080039,
                "",
            );
            effects::fixed_particles(
                w,
                rider,
                0x373A,
                10,
                15,
                5012,
                muo_server::effects::EffectLayer::Waist,
            );
            effects::play_sound(w, rider, 0x1E0); // Cure spell effect.
            effects::play_sound(w, rider, 0xA9); // Unicorn whinny.
            return true;
        }
    }
    false
}

/// `if (!Controlled) return 0x16A; return base.GetAngerSound();`
pub fn nightmare_get_anger_sound(w: &mut World, id: MobileId) -> i32 {
    if !cd(w, id).is_some_and(|c| c.controlled) {
        return 0x16A;
    }
    (muo_content::creature::BASE_CREATURE.get_anger_sound)(w, id)
}
