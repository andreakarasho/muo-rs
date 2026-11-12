//! The Mind Rot spell.

use indexmap::IndexMap;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::hooks::hooks_mut;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::{effects, messages as mm, stats};
use muo_server::skills::SkillName;
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};
use crate::common::{alive, is_player_mobile, mobile_of, turn_to};

static MIND_ROT: NecroType = NecroType {
    required_skill: 30.0,
    required_mana: 17,
    ..NECRO_BASE
};

/// `MindRotSpell`: Wis An Ben.
pub static MIND_ROT_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.MindRotSpell",
    parent: Some(&NECROMANCER_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Mind Rot",
        "Wis An Ben",
        &[reagent::BAT_WING, reagent::PIG_IRON, reagent::DAEMON_BLOOD],
    )
    .action(203)
    .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_millis(1500),
    on_cast,
    family: Some(&MIND_ROT),
    ..NECROMANCER_SPELL
};

struct Entry {
    scalar: f64,
    timer: TimerHandle,
}

#[derive(Default)]
struct Table(IndexMap<MobileId, Entry>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, Entry> {
    &mut w.resources.get_mut::<Table>().0
}

/// Sets `SpellHooks::mind_rot_scalar`.
pub fn init(w: &mut World) {
    hooks_mut(w).mind_rot_scalar =
        Some(|w, m| w.resources.get::<Table>()?.0.get(&m).map(|e| e.scalar));
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if has_mind_rot_scalar(w, m) {
        mm::send_localized_message(w, caster, 1005559, "", 0x3B2); // This spell is already in effect.
    } else if spell::check_h_sequence(w, sp, m) {
        turn_to(w, caster, m);

        /* Attempts to place a curse on the Target that increases the mana cost of any spells they cast, for a
         * duration based off a comparison between the Caster's Spirit Speak skill and the Target's Resisting Spells
         * skill. The effect lasts for ((Spirit Speak skill level - target's Resist Magic skill level) / 50 ) + 20
         * seconds.
         */
        spell::caster_hurt(w, m);

        effects::play_sound(w, m, 0x1FB);
        effects::play_sound(w, m, 0x258);

        effects::fixed_particles_hue(w, m, 0x373A, 1, 17, 9903, 15, 4, EffectLayer::Head);

        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        let player = w.mobile(m).is_some_and(|x| x.player);
        let duration =
            (((ty.get_damage_skill)(w, sp, caster) - (ty.get_resist_skill)(w, sp, m)) / 5.0 + 20.0)
                * if player { 1.0 } else { 2.0 };
        stats::check_skill(w, m, SkillName::MagicResist, 0.0, 120.0); // Skill check for gain

        set_mind_rot_scalar(
            w,
            m,
            if player { 1.25 } else { 2.00 },
            TimeSpan::from_secs_f64(duration),
        );

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

/// `ClearMindRotScalar(m)`.
pub fn clear_mind_rot_scalar(w: &mut World, m: MobileId) -> bool {
    if let Some(mut e) = table(w).shift_remove(&m) {
        e.timer.cancel(w);
        mm::send_localized_message(w, m, 1060872, "", 0x3B2); // Your mind feels normal again.
        if is_player_mobile(w, m) {
            remove_buff(w, m, BuffIcon::Mindrot);
        }
        return true;
    }
    false
}

/// `GetMindRotScalar(m, ref scalar)`, read-only.
pub fn scalar_of(w: &World, m: MobileId) -> Option<f64> {
    w.resources.get::<Table>()?.0.get(&m).map(|e| e.scalar)
}

/// `HasMindRotScalar(m)`.
pub fn has_mind_rot_scalar(w: &mut World, m: MobileId) -> bool {
    table(w).contains_key(&m)
}

/// `SetMindRotScalar(caster, target, scalar, duration)`.
pub fn set_mind_rot_scalar(w: &mut World, target: MobileId, scalar: f64, duration: TimeSpan) {
    if !table(w).contains_key(&target) {
        // `MRExpireTimer`: every second, until the end time, death or deletion.
        let end: DateTime = w.now + duration;
        let h = timer::start(
            w,
            TimeSpan::from_secs(1),
            TimeSpan::from_secs(1),
            0,
            move |w| {
                let gone = w.mobile(target).is_none_or(|t| t.deleted) || !alive(w, target);
                if gone || w.now >= end {
                    // The timer stops itself: clear_mind_rot_scalar cancels the stored handle.
                    clear_mind_rot_scalar(w, target);
                }
            },
        );
        table(w).insert(target, Entry { scalar, timer: h });

        if is_player_mobile(w, target) {
            add_buff(
                w,
                target,
                BuffInfo::new(BuffIcon::Mindrot, 1075665).duration(duration),
            );
        }
        mm::send_localized_message(w, target, 1074384, "", 0x3B2);
    }
}
