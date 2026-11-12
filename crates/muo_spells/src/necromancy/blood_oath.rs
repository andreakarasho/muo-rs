//! The Blood Oath spell.

use indexmap::IndexMap;
use muo_content::creature::{self, events};
use muo_content::hooks::hooks_mut as content_hooks_mut;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_io::text::TextDefinition;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::{effects, messages as mm, stats};
use muo_server::skills::SkillName;
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};
use crate::common::{is_player_mobile, mobile_of, turn_to};

static BLOOD_OATH: NecroType = NecroType {
    required_skill: 20.0,
    required_mana: 13,
    ..NECRO_BASE
};

/// `BloodOathSpell`: In Jux Mani Xen.
pub static BLOOD_OATH_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.BloodOathSpell",
    parent: Some(&NECROMANCER_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new("Blood Oath", "In Jux Mani Xen", &[reagent::DAEMON_BLOOD])
        .action(203)
        .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_millis(1500),
    on_cast,
    family: Some(&BLOOD_OATH),
    ..NECROMANCER_SPELL
};

#[derive(Copy, Clone)]
struct Oath {
    caster: MobileId,
    target: MobileId,
    timer: TimerHandle,
}

/// Keyed by BOTH participants (caster and target) -> shared timer, so the oath resolves and removes from either
/// side. Required so the death/delete events can break it from either mobile.
#[derive(Default)]
struct Table(IndexMap<MobileId, Oath>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, Oath> {
    &mut w.resources.get_mut::<Table>().0
}

/// Sets the content hooks and the four death/deletion events.
pub fn init(w: &mut World) {
    let h = content_hooks_mut(w);
    h.blood_oath_of = Some(get_blood_oath);
    h.blood_oath_reflected = Some(compute_reflected_damage);
    muo_content::player_mobile::subscribe_death(w, remove_curse_event);
    muo_content::player_mobile::subscribe_deleted(w, remove_curse_event);
    events::subscribe_death(w, remove_curse_event);
    events::subscribe_deleted(w, remove_curse_event);
}

fn remove_curse_event(w: &mut World, m: MobileId) {
    remove_curse(w, m);
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    // only PlayerMobile and BaseCreature implement blood oath checking
    let ok_type = is_player_mobile(w, m) || creature::is_creature(w, m);
    if caster == m || !ok_type {
        mm::send_localized_message(w, caster, 1060508, "", 0x3B2); // You can't curse that.
    } else if table(w).contains_key(&caster) {
        mm::send_localized_message(w, caster, 1061607, "", 0x3B2); // You are already bonded in a Blood Oath.
    } else if table(w).contains_key(&m) {
        if w.mobile(m).is_some_and(|x| x.player) {
            mm::send_localized_message(w, caster, 1061608, "", 0x3B2); // That player is already bonded in a Blood Oath.
        } else {
            mm::send_localized_message(w, caster, 1061609, "", 0x3B2); // That creature is already bonded in a Blood Oath.
        }
    } else if spell::check_h_sequence(w, sp, m) {
        turn_to(w, caster, m);

        /* Temporarily creates a dark pact between the caster and the target.
         * Any damage dealt by the target to the caster is increased, but the target receives the same amount of
         * damage. The effect lasts for ((Spirit Speak skill level - target's Resist Magic skill level) / 8) + 8
         * seconds.
         */
        spell::caster_hurt(w, m);

        effects::play_sound(w, caster, 0x175);

        effects::fixed_particles_hue(w, caster, 0x375A, 1, 17, 9919, 33, 7, EffectLayer::Waist);
        effects::fixed_particles_hue(w, caster, 0x3728, 1, 13, 9502, 33, 7, EffectLayer(255));
        effects::fixed_particles_hue(w, m, 0x375A, 1, 17, 9919, 33, 7, EffectLayer::Waist);
        effects::fixed_particles_hue(w, m, 0x3728, 1, 13, 9502, 33, 7, EffectLayer(255));

        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        let duration = TimeSpan::from_secs_f64(get_duration_seconds(
            (ty.get_damage_skill)(w, sp, caster),
            (ty.get_resist_skill)(w, sp, m),
        ));
        stats::check_skill(w, m, SkillName::MagicResist, 0.0, 120.0); // Skill check for gain

        register_oath(w, caster, m, duration);

        spell::harmful_spell(w, sp, m);
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

// ((Spirit Speak - Resisting Spells) / 8) + 8 seconds.
pub fn get_duration_seconds(damage_skill: f64, resist_skill: f64) -> f64 {
    (damage_skill - resist_skill) / 8.0 + 8.0
}

// The attacker takes the original (un-bonused) damage reflected back. Publish 48 lets the attacker's Resisting
// Spells reduce the reflected damage, but only against creature casters.
pub fn compute_reflected_damage(
    original_damage: i32,
    attacker_magic_resist: f64,
    apply_resist_mitigation: bool,
) -> i32 {
    if !apply_resist_mitigation {
        return original_damage;
    }

    // ((Resisting Spells * 10) / 20) + 10 = percentage of damage resisted
    let resisted = (attacker_magic_resist * 0.5 + 10.0) / 100.0;
    (f64::from(original_damage) * (1.0 - resisted)) as i32
}

fn register_oath(w: &mut World, caster: MobileId, target: MobileId, duration: TimeSpan) {
    let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
        remove_curse(w, target);
    });
    let oath = Oath {
        caster,
        target,
        timer: h,
    };
    table(w).insert(caster, oath);
    table(w).insert(target, oath);

    if is_player_mobile(w, caster) {
        let name = w
            .mobile(target)
            .and_then(|t| (t.ty.name)(w, target))
            .unwrap_or_default();
        add_buff(
            w,
            caster,
            BuffInfo::new(BuffIcon::BloodOathCaster, 1075659)
                .duration(duration)
                .args(TextDefinition::of_text(name)),
        );
    }
    if is_player_mobile(w, target) {
        let name = w
            .mobile(caster)
            .and_then(|t| (t.ty.name)(w, caster))
            .unwrap_or_default();
        add_buff(
            w,
            target,
            BuffInfo::new(BuffIcon::BloodOathCurse, 1075661)
                .duration(duration)
                .args(TextDefinition::of_text(name)),
        );
    }
}

/// `RemoveCurse(m)`.
pub fn remove_curse(w: &mut World, m: MobileId) -> bool {
    let Some(oath) = table(w).get(&m).copied() else {
        return false;
    };

    let mut t = oath.timer;
    t.cancel(w);
    table(w).shift_remove(&oath.caster);
    table(w).shift_remove(&oath.target);

    mm::send_localized_message(w, oath.caster, 1061620, "", 0x3B2); // Your Blood Oath has been broken.
    mm::send_localized_message(w, oath.target, 1061620, "", 0x3B2); // Your Blood Oath has been broken.

    if is_player_mobile(w, oath.caster) {
        remove_buff(w, oath.caster, BuffIcon::BloodOathCaster);
    }
    if is_player_mobile(w, oath.target) {
        remove_buff(w, oath.target, BuffIcon::BloodOathCurse);
    }

    true
}

/// `GetBloodOath(m)`: the caster, when `m` is the target of the oath.
pub fn get_blood_oath(w: &World, m: MobileId) -> Option<MobileId> {
    w.resources
        .get::<Table>()?
        .0
        .get(&m)
        .filter(|o| o.target == m)
        .map(|o| o.caster)
}

/// Blood Oath tests.
#[cfg(test)]
mod behavior_tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;

    fn new_mobile(w: &mut World) -> MobileId {
        let m = w.new_mobile(&MOBILE_BASE);
        stats::default_mobile_init(w, m);
        m
    }

    // Player caster: Publish 48 resist mitigation does NOT apply; the attacker takes the full reflected (original,
    // un-bonused) damage. Creature caster (Publish 48 / SA+): ((Resist * 10) / 20) + 10 = % of reflected damage resisted.
    #[test]
    fn compute_reflected_damage_with_mitigation_reduces_by_resist() {
        for (dmg, resist, expected) in [
            (40, 0.0, 36),   // 10% resisted -> 40 * 0.90
            (40, 100.0, 16), // 60% resisted -> 40 * 0.40
            (40, 120.0, 12), // 70% resisted -> 40 * 0.30
        ] {
            assert_eq!(
                compute_reflected_damage(dmg, resist, true),
                expected,
                "{resist}"
            );
        }
    }

    // The cursed target reflects to the caster; the caster attacking other mobiles must not reflect.
    #[test]
    fn register_oath_binds_target_to_caster_not_caster_to_self() {
        let mut w = World::new();
        let (caster, target) = (new_mobile(&mut w), new_mobile(&mut w));

        register_oath(&mut w, caster, target, TimeSpan::from_mins(5));

        assert_eq!(get_blood_oath(&w, target), Some(caster));
        assert_eq!(get_blood_oath(&w, caster), None);

        remove_curse(&mut w, target);
    }

    // Death/delete of either party removes the oath, so RemoveCurse must resolve from either the caster or the target
    // key (the death hooks call it with `this`).
    #[test]
    fn remove_curse_by_caster_clears_both_entries() {
        let mut w = World::new();
        let (caster, target) = (new_mobile(&mut w), new_mobile(&mut w));
        register_oath(&mut w, caster, target, TimeSpan::from_mins(5));

        assert!(remove_curse(&mut w, caster));

        assert_eq!(get_blood_oath(&w, target), None);
        assert!(!remove_curse(&mut w, target)); // already removed
    }

    // Death/delete of either party must break the oath immediately: wired to the central PlayerMobile/BaseCreature
    // death+delete events instead of a polling timer.
    #[test]
    fn player_deleted_event_breaks_oath() {
        let mut w = World::new();
        init(&mut w);
        let caster = muo_content::player_mobile::new_player_mobile(&mut w);
        let target = muo_content::player_mobile::new_player_mobile(&mut w);

        register_oath(&mut w, caster, target, TimeSpan::from_mins(5));

        muo_content::player_mobile::player_deleted_event(&mut w, caster); // central handler breaks the oath from the caster side

        assert_eq!(get_blood_oath(&w, target), None);
        assert!(!remove_curse(&mut w, target));
    }

    #[test]
    fn creature_deleted_event_breaks_oath() {
        let mut w = World::new();
        init(&mut w);
        let caster = muo_content::player_mobile::new_player_mobile(&mut w);
        let target = w.new_mobile(&muo_content::creature::BASE_CREATURE);
        stats::default_mobile_init(&mut w, target);

        register_oath(&mut w, caster, target, TimeSpan::from_mins(5));

        events::creature_deleted_event(&mut w, target); // central handler breaks the oath from the target side

        assert_eq!(get_blood_oath(&w, target), None);
        assert!(!remove_curse(&mut w, caster));
    }
}
