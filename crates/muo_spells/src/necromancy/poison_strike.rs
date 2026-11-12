//! The Poison Strike spell.

use muo_content::core;
use muo_content::creature::{self, ctype};
use muo_content::misc::aos_helpers;
use muo_content::spells::helper::{self, SpellDamageSplit};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::item::effect_item;
use muo_server::mobile::base as mb;
use muo_server::target::{TargetFlags, Targeted};

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};
use crate::common::{mobile_of, mobiles_in_range, turn_to};

static POISON_STRIKE: NecroType = NecroType {
    required_skill: 50.0,
    required_mana: 17,
    ..NECRO_BASE
};

/// `PoisonStrikeSpell`: In Vas Nox.
pub static POISON_STRIKE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.PoisonStrikeSpell",
    parent: Some(&NECROMANCER_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Poison Strike", "In Vas Nox", &[reagent::NOX_CRYSTAL])
        .action(203)
        .hand_effect(9031),
    cast_delay_base: |w, _| TimeSpan::from_millis(if core::ml(w) { 1750 } else { 1500 }),
    delayed_damage: false,
    on_cast,
    family: Some(&POISON_STRIKE),
    ..NECROMANCER_SPELL
};

/// `cbc?.Controlled == false && (cbc.IsAnimatedDead || !cbc.Summoned)`.
pub fn is_monster(w: &World, caster: MobileId) -> bool {
    match creature::cd(w, caster) {
        Some(d) => !d.controlled && (creature::misc::is_animated_dead(w, caster) || !d.summoned),
        None => false,
    }
}

/// The creature filter shared with Wither: `targ is BaseCreature bc` animated dead, and for a monster caster the
/// controlled, summoned, same team and necro familiar creatures. True means skip the target.
pub fn skip_creature(w: &World, caster: MobileId, targ: MobileId, monster: bool) -> bool {
    let Some(d) = creature::cd(w, targ) else {
        return false;
    };
    if creature::misc::is_animated_dead(w, targ) {
        return true;
    }

    if monster {
        let team = creature::cd(w, caster).map_or(0, |c| c.team);
        let familiar = ctype(w, targ).is_some_and(|c| (c.is_necro_familiar)(w, targ));
        if d.controlled || d.summoned || d.team == team || familiar {
            return true;
        }
    }
    false
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_h_sequence(w, sp, m) {
        turn_to(w, caster, m);

        /* Creates a blast of poisonous energy centered on the target. The main target is inflicted with a large
         * amount of Poison damage, and all valid targets in a radius of 2 tiles around the main target are
         * inflicted with a lesser effect. One tile from main target receives 50% damage, two tiles from target
         * receives 33% damage.
         */
        // CheckResisted( m );
        // Check magic resist for skill, but do not use return value
        // reports from OSI:  Necro spells don't give Resist gain
        let (mloc, map) = w.mobile(m).map(|x| (x.location, x.map)).unwrap_or_default();
        if map.is_some() {
            let e = effect_item::create_targeted(w, mloc, map, effect_item::DEFAULT_DURATION);
            muo_server::effects::send_location_particles_full(w, e, 0x36B0, 1, 14, 63, 7, 9915, 0);
            muo_server::effects::play_sound(w, mloc, map, 0x229);

            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            let low = if core::ml(w) { 32 } else { 36 };
            let damage = f64::from(utility::random_min_max(&mut w.rng, low, 40))
                * ((300.0 + (ty.get_damage_skill)(w, sp, caster) * 9.0) / 1000.0);

            let mut sdi_bonus = f64::from(aos_helpers::attribute_value(
                w,
                caster,
                muo_content::aos::AosAttribute::SpellDamage,
            )) / 100.0;

            let pvm_damage = damage * (1.0 + sdi_bonus);

            if core::ml(w) && sdi_bonus > 0.15 {
                sdi_bonus = 0.15;
            }

            let pvp_damage = damage * (1.0 + sdi_bonus);

            let mut pool = Vec::new();

            if mb::can_be_harmful_msg(w, caster, m, false) {
                pool.push(m);
            }

            let monster = is_monster(w, caster);

            for targ in mobiles_in_range(w, map, mloc, 2) {
                if targ == caster
                    || m == targ
                    || !helper::valid_indirect_target(w, caster, targ)
                    || !mb::can_be_harmful_msg(w, caster, targ, false)
                {
                    continue;
                }

                if monster && w.mobile(targ).is_some_and(|x| x.player) {
                    continue;
                }

                // Animate dead casting poison strike shouldn't hit: familiars or player or pets
                if skip_creature(w, caster, targ, monster) {
                    continue;
                }

                pool.push(targ);
            }

            let pvp =
                w.mobile(m).is_some_and(|x| x.player) && w.mobile(caster).is_some_and(|x| x.player);
            for targ in pool {
                let num = if mb::in_range_point3d(w, targ, mloc, 0) {
                    1.0
                } else if mb::in_range_point3d(w, targ, mloc, 1) {
                    2.0
                } else {
                    3.0
                };

                mb::do_harmful(w, caster, targ, false);
                helper::damage_aos_spell(
                    w,
                    sp,
                    targ,
                    (if pvp { pvp_damage } else { pvm_damage }) / num,
                    SpellDamageSplit {
                        pois: 100,
                        ..Default::default()
                    },
                );
            }
        }
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    begin_spell_target(
        w,
        sp,
        false,
        TargetFlags::HARMFUL,
        false,
        SpellTargetKind::Mobile,
        target,
    );
}
