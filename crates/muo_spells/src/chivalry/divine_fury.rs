//! The Divine Fury spell.

use indexmap::IndexMap;
use muo_content::core;
use muo_content::items::weapon_hooks::WeaponHooks;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::SpellInfo;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_io::text::TextDefinition;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::effects;
use muo_server::mobile::{MobileDelta, base as mb};
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use super::paladin::{PALADIN_BASE, PALADIN_SPELL, PaladinType, power};
use crate::common::is_player_mobile;

static DIVINE_FURY: PaladinType = PaladinType {
    required_skill: 25.0,
    required_mana: 15,
    required_tithing: 10,
    mantra_number: 1060722, // Divinum Furis
    ..PALADIN_BASE
};

/// `DivineFurySpell`: Divinum Furis.
pub static DIVINE_FURY_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Chivalry.DivineFurySpell",
    parent: Some(&PALADIN_SPELL),
    info: SpellInfo::new("Divine Fury", "Divinum Furis", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_secs(1),
    on_cast,
    blocks_movement: |_, _| false,
    family: Some(&DIVINE_FURY),
    ..PALADIN_SPELL
};

/// `InternalTimer`'s bonuses and its timer.
#[derive(Copy, Clone)]
struct Fury {
    timer: TimerHandle,
    damage_bonus: i32,
    weapon_speed: i32,
    attack_bonus: i32,
    defend_malus: i32,
}

#[derive(Default)]
struct Table(IndexMap<MobileId, Fury>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, Fury> {
    &mut w.resources.get_mut::<Table>().0
}

/// Sets the weapon hooks.
pub fn init(w: &mut World) {
    let h = w.resources.get_mut::<WeaponHooks>();
    h.divine_fury_under_effect = Some(|w, m| table_ref(w).is_some_and(|t| t.contains_key(&m)));
    h.divine_fury_attack_bonus = Some(get_attack_bonus);
    h.divine_fury_defend_malus = Some(get_defend_malus);
    h.divine_fury_weapon_speed = Some(|w, m| {
        table_ref(w)
            .and_then(|t| t.get(&m))
            .map_or(0, |f| f.weapon_speed)
    });
    h.divine_fury_damage_bonus = Some(|w, m| {
        table_ref(w)
            .and_then(|t| t.get(&m))
            .map_or(0, |f| f.damage_bonus)
    });
}

fn table_ref(w: &World) -> Option<&IndexMap<MobileId, Fury>> {
    w.resources.get::<Table>().map(|t| &t.0)
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        effects::play_sound(w, caster, 0x20F);
        let female = w.mobile(caster).is_some_and(|c| c.female);
        effects::play_sound(w, caster, if female { 0x338 } else { 0x44A });
        effects::fixed_particles_hue(w, caster, 0x376A, 1, 31, 9961, 1160, 0, EffectLayer::Waist);
        effects::fixed_particles_hue(w, caster, 0x37C4, 1, 31, 9502, 43, 2, EffectLayer::Waist);

        let stam_max = mb::stam_max(w, caster);
        props_b::set_stam(w, caster, stam_max);

        remove_timer(w, caster);

        let delay = TimeSpan::from_secs(i64::from(power(w, sp, 10).clamp(7, 24)));

        let boosted = core::hs(w)
            && skills::value(w, caster, SkillName::Chivalry) >= 120.0
            && w.mobile(caster).is_some_and(|c| c.karma >= 10000);
        let (attack_bonus, damage_bonus, weapon_speed, defend_malus) = if boosted {
            (15, 20, 15, 10)
        } else {
            (10, 10, 10, 20)
        };

        let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
            stop_divine_fury(w, caster)
        });
        table(w).insert(
            caster,
            Fury {
                timer: h,
                damage_bonus,
                weapon_speed,
                attack_bonus,
                defend_malus,
            },
        );

        mb::delta(w, caster, MobileDelta::WEAPON_DAMAGE);

        if is_player_mobile(w, caster) {
            if core::hs(w) {
                // Publish 71 with boost
                let args =
                    format!("{attack_bonus}\t{damage_bonus}\t{weapon_speed}\t{defend_malus}");
                add_buff(
                    w,
                    caster,
                    BuffInfo::with_secondary(BuffIcon::DivineFury, 1060589, 1150218)
                        .duration(delay)
                        .args(TextDefinition::of_text(args)),
                );
            } else {
                add_buff(
                    w,
                    caster,
                    BuffInfo::with_secondary(BuffIcon::DivineFury, 1060589, 1075634)
                        .duration(delay),
                );
            }
        }
    }

    spell::finish_sequence(w, sp);
}

fn remove_timer(w: &mut World, m: MobileId) {
    if let Some(mut f) = table(w).shift_remove(&m) {
        f.timer.cancel(w);
    }
}

/// `StopDivineFury(m)`.
pub fn stop_divine_fury(w: &mut World, m: MobileId) {
    remove_timer(w, m);
    mb::delta(w, m, MobileDelta::WEAPON_DAMAGE);
    effects::play_sound(w, m, 0xF8);
    if is_player_mobile(w, m) {
        remove_buff(w, m, BuffIcon::DivineFury);
    }
}

/// `GetAttackBonus(m)`.
pub fn get_attack_bonus(w: &World, m: MobileId) -> i32 {
    table_ref(w)
        .and_then(|t| t.get(&m))
        .map_or(0, |f| f.attack_bonus)
}

/// `GetDefendMalus(m)`.
pub fn get_defend_malus(w: &World, m: MobileId) -> i32 {
    table_ref(w)
        .and_then(|t| t.get(&m))
        .map_or(0, |f| f.defend_malus)
}
