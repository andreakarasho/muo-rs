//! The `AOS` static class. The attribute bags (`AosAttributes` .. `BaseAttributes`) are in `crate::aos`, with
//! the item families.
//!
//! `AosAttributes.GetValue(m, attribute)` walks the equipped items by class (`BaseWeapon`, `BaseArmor`, ..) and
//! sums their bags; [`attribute_value`] asks `items::weapon_combat::equipped_attribute_value` through a hook.

use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::ResistanceType;

use crate::aos::AosAttribute;
use crate::mx;
use crate::player_mobile::PLAYER_MOBILE;

/// What other content crates answer for `AosAttributes.GetValue`.
#[derive(Default)]
pub struct AosHooks {
    /// The sum of `attribute` over the mobile's equipped items (without the `Core.AOS` check).
    pub attribute_value: Option<fn(&World, MobileId, AosAttribute) -> i32>,
}

/// `AosAttributes.GetValue(m, attribute)`.
pub fn attribute_value(w: &World, m: MobileId, attribute: AosAttribute) -> i32 {
    if !crate::core::aos(w) {
        return 0;
    }
    // `items::weapon_combat::equipped_attribute_value` (set by `BaseWeapon.Configure`) sums the
    // weapon, armor, jewel, clothing, spellbook, quiver and talisman bags and the Luck bonuses.
    w.resources
        .get::<AosHooks>()
        .and_then(|h| h.attribute_value)
        .map_or(0, |f| f(w, m, attribute))
}

/// `AOS.Fix`.
pub fn fix(val: &mut i32) {
    if *val < 0 {
        *val = 0;
    }
}

/// `AOS.Scale`.
pub fn scale(input: i32, percent: i32) -> i32 {
    input.wrapping_mul(percent) / 100
}

/// `AOS.DisableStatInfluences`: zeroes the stat scales of every skill.
pub fn disable_stat_influences(w: &mut World) {
    for info in &mut w.resources.get_mut::<muo_server::skills::SkillTable>().info {
        info.str_scale = 0.0;
        info.dex_scale = 0.0;
        info.int_scale = 0.0;
        info.stat_total = 0.0;
    }
}

/// `AOS.GetStatus(from, index)`.
pub fn get_status(w: &World, from: MobileId, index: i32) -> i32 {
    let Some(ty) = w.mobile(from).map(|m| m.ty) else {
        return 0;
    };
    let a = |attr| attribute_value(w, from, attr);
    match index {
        // TODO: account for buffs/debuffs
        0 => (ty.get_max_resistance)(w, from, ResistanceType::Physical),
        1 => (ty.get_max_resistance)(w, from, ResistanceType::Fire),
        2 => (ty.get_max_resistance)(w, from, ResistanceType::Cold),
        3 => (ty.get_max_resistance)(w, from, ResistanceType::Poison),
        4 => (ty.get_max_resistance)(w, from, ResistanceType::Energy),
        5 => a(AosAttribute::DefendChance),
        6 => 45,
        7 => a(AosAttribute::AttackChance),
        8 => a(AosAttribute::WeaponSpeed),
        9 => a(AosAttribute::WeaponDamage),
        10 => a(AosAttribute::LowerRegCost),
        11 => a(AosAttribute::SpellDamage),
        12 => a(AosAttribute::CastRecovery),
        13 => a(AosAttribute::CastSpeed),
        14 => a(AosAttribute::LowerManaCost),
        _ => 0,
    }
}

/// The five damage percentages of an `AOS.Damage` call.
#[derive(Copy, Clone, Debug, Default)]
pub struct DamageSplit {
    pub phys: i32,
    pub fire: i32,
    pub cold: i32,
    pub pois: i32,
    pub nrgy: i32,
}

/// The optional arguments of `AOS.Damage`.
#[derive(Copy, Clone, Debug, Default)]
pub struct DamageOptions {
    pub ignore_armor: bool,
    pub chaos: i32,
    pub direct: i32,
    pub keep_alive: bool,
    pub archer: bool,
    pub death_strike: bool,
}

/// `AOS.Damage(m, from, damage, ignoreArmor, phys, fire, cold, pois, nrgy, chaos, direct, keepAlive, archer,
/// deathStrike)`: the damage dealt.
pub fn damage(
    w: &mut World,
    m: MobileId,
    from: Option<MobileId>,
    damage: i32,
    split: DamageSplit,
    opt: DamageOptions,
) -> i32 {
    let Some(mm) = w.mobile(m) else { return 0 };
    if mm.deleted || damage <= 0 {
        return 0;
    }
    let ty = mm.ty;
    if !(ty.alive)(w, m) {
        return 0;
    }
    if split.phys == 0
        && split.fire == 100
        && split.cold == 0
        && split.pois == 0
        && split.nrgy == 0
        && let Some(f) = w
            .resources
            .get::<crate::hooks::ContentHooks>()
            .and_then(|h| h.meer_mage_stop)
    {
        f(w, m, true); // MeerMage.StopEffect(m, true)
    }
    if !crate::core::aos(w) {
        (ty.damage)(w, m, damage, from, true, false);
        return damage;
    }
    let DamageSplit {
        mut phys,
        mut fire,
        mut cold,
        mut pois,
        mut nrgy,
    } = split;
    let (mut chaos, mut direct) = (opt.chaos, opt.direct);
    for v in [
        &mut phys,
        &mut fire,
        &mut cold,
        &mut pois,
        &mut nrgy,
        &mut chaos,
        &mut direct,
    ] {
        fix(v);
    }

    if crate::core::ml(w) && chaos > 0 {
        match muo_primitives::utility::random(&mut w.rng, 5) {
            0 => phys += chaos,
            1 => fire += chaos,
            2 => cold += chaos,
            3 => pois += chaos,
            _ => nrgy += chaos,
        }
    }

    let is_player = |w: &World, id: MobileId| w.mobile(id).is_some_and(|x| x.ty.is(&PLAYER_MOBILE));
    // `from.FindItemOnLayer<BaseQuiver>(Layer.Cloak)`
    let quiver_increase = from
        .filter(|_| opt.archer)
        .and_then(|f| {
            muo_server::mobile::part2::items::find_item_on_layer(
                w,
                f,
                muo_server::layer::Layer::Cloak,
            )
        })
        .and_then(|q| {
            w.item(q)
                .and_then(|i| i.ext.get::<crate::items::ext_bags::QuiverData>())
        })
        .map(|d| d.damage_increase);
    let mut damage = damage;
    let mut total;
    if !opt.ignore_armor {
        // Armor Ignore on OSI ignores all defenses, not just physical.
        total = damage * phys * (100 - (ty.physical_resistance)(w, m));
        total += damage * fire * (100 - (ty.fire_resistance)(w, m));
        total += damage * cold * (100 - (ty.cold_resistance)(w, m));
        total += damage * pois * (100 - (ty.poison_resistance)(w, m));
        total += damage * nrgy * (100 - (ty.energy_resistance)(w, m));
        total /= 10000;
        if crate::core::ml(w) {
            total += damage * direct / 100;
            if let Some(inc) = quiver_increase {
                total += total * inc / 100;
            }
        }
        if total < 1 {
            total = 1;
        }
    } else if crate::core::ml(w) && is_player(w, m) && from.is_some_and(|f| is_player(w, f)) {
        if let Some(inc) = quiver_increase {
            damage += damage * inc / 100;
        }
        total = if opt.death_strike {
            damage.min(70)
        } else {
            damage.min(35)
        }; // direct damage caps
    } else {
        total = damage;
        if crate::core::ml(w)
            && let Some(inc) = quiver_increase
        {
            total += total * inc / 100;
        }
    }

    // A player's barded SwampDragon mount takes a share of what a non-player deals.
    if !from.is_some_and(|f| w.mobile(f).is_some_and(|x| x.player))
        && w.mobile(m).is_some_and(|x| x.player)
        && let Some(f) = crate::hooks::hooks(w).and_then(|h| h.swamp_dragon_absorb)
    {
        total = f(w, m, total);
    }

    if opt.keep_alive {
        let hits = w.mobile(m).map_or(0, |x| x.hits);
        if total > hits {
            total = hits;
        }
    }
    // Reflect physical damage back at a live attacker.
    if let Some(f) = from.filter(|&f| w.mobile(f).is_some_and(|x| !x.deleted) && mx::alive(w, f)) {
        use crate::abilities::{self as ab, MonsterAbilityTrigger, MonsterAbilityType};
        let bc_from = crate::creature::data::cd(w, f).is_some();
        let mut reflect_phys = attribute_value(w, m, AosAttribute::ReflectPhysical);
        if bc_from
            && let Some(a) = ab::get_ability(w, f, MonsterAbilityType::ReflectPhysicalDamage)
            && (a.can_trigger)(w, a, f, MonsterAbilityTrigger::COMBAT_ACTION)
        {
            reflect_phys += 10; // ReflectPhysicalDamage.PercentReflected
            mx::msg(w, m, 1070844); // The creature repels the attack back at you.
        }
        if reflect_phys > 0 {
            let res = if opt.ignore_armor {
                0
            } else {
                (ty.physical_resistance)(w, m)
            };
            let mut reflect_damage = scale(damage * phys * (100 - res) / 10000, reflect_phys);
            if bc_from && let Some(b) = ab::get_ability(w, f, MonsterAbilityType::MagicalBarrier) {
                (b.alter_melee_from)(w, b, f, m, &mut reflect_damage);
            }
            if reflect_damage > 0
                && let Some(fty) = w.mobile(f).map(|x| x.ty)
            {
                (fty.damage)(w, f, reflect_damage, Some(m), true, false);
            }
        }
    }
    if total <= 0 {
        return 0;
    }
    if let Some(f) = from {
        crate::systems::spells::helper::do_leech(w, total, f, m); // sanity check: from != null
    }
    (ty.damage)(w, m, total, from, true, false);
    total
}

/// `ContentHooks::aos_damage`: `AOS.Damage` for the callers that only know the hooks (creature abilities, auras).
pub fn damage_hook(
    w: &mut World,
    m: MobileId,
    from: Option<MobileId>,
    d: crate::hooks::AosDamage,
) -> i32 {
    let split = DamageSplit {
        phys: d.phys,
        fire: d.fire,
        cold: d.cold,
        pois: d.pois,
        nrgy: d.nrgy,
    };
    let opt = DamageOptions {
        ignore_armor: d.ignore_armor,
        chaos: d.chaos,
        direct: d.direct,
        ..Default::default()
    };
    damage(w, m, from, d.amount, split, opt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_and_fix() {
        assert_eq!(scale(50, 30), 15);
        let mut v = -3;
        fix(&mut v);
        assert_eq!(v, 0);
    }
}
