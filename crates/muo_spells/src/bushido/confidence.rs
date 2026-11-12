//! The Confidence spell.

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
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use super::samurai::{SAMURAI_SPELL, SamuraiType, on_cast_successful, on_effect_end};
use crate::common::is_player_mobile;

static CONFIDENCE: SamuraiType = SamuraiType {
    required_skill: 25.0,
    required_mana: 10,
};

/// `Confidence`: Confidence.
pub static CONFIDENCE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Bushido.Confidence",
    parent: Some(&SAMURAI_SPELL),
    info: SpellInfo::new("Confidence", "", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_millis(250),
    on_cast,
    on_begin_cast,
    family: Some(&CONFIDENCE),
    ..SAMURAI_SPELL
};

#[derive(Default)]
struct Tables {
    table: IndexMap<MobileId, TimerHandle>,
    regen: IndexMap<MobileId, TimerHandle>,
}

fn tables(w: &mut World) -> &mut Tables {
    w.resources.get_mut::<Tables>()
}

/// Sets the weapon hook.
pub fn init(w: &mut World) {
    w.resources.get_mut::<WeaponHooks>().confidence_is_confident = Some(|w, m| {
        w.resources
            .get::<Tables>()
            .is_some_and(|t| t.table.contains_key(&m))
    });
}

fn on_begin_cast(w: &mut World, sp: SpellId) {
    (SAMURAI_SPELL.on_begin_cast)(w, sp);
    if let Some(caster) = spell::caster(w, sp) {
        effects::fixed_effect_hue(w, caster, 0x37C4, 10, 7, 4, 3);
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        mm::send_localized_message(w, caster, 1063115, "", 0x3B2); // You exude confidence.

        effects::fixed_particles_hue(
            w,
            caster,
            0x375A,
            1,
            17,
            0x7DA,
            0x960,
            0x3,
            EffectLayer::Waist,
        );
        effects::play_sound(w, caster, 0x51A);

        on_cast_successful(w, sp);

        begin_confidence(w, caster);
        begin_regenerating(w, caster);
    }

    spell::finish_sequence(w, sp);
}

/// `IsConfident(m)`.
pub fn is_confident(w: &mut World, m: MobileId) -> bool {
    tables(w).table.contains_key(&m)
}

/// `BeginConfidence(m)`.
pub fn begin_confidence(w: &mut World, m: MobileId) {
    stop_confidence_timer(w, m);

    let h = timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
        end_confidence(w, m);
        mm::send_localized_message(w, m, 1063116, "", 0x3B2); // Your confidence wanes.
    });
    tables(w).table.insert(m, h);

    if is_player_mobile(w, m) {
        if core::hs(w) {
            let bushido = skills::fixed(w, m, SkillName::Bushido);
            add_buff(
                w,
                m,
                BuffInfo::with_secondary(BuffIcon::Confidence, 1060596, 1153809)
                    .duration(TimeSpan::from_secs(30))
                    .args(TextDefinition::of_text(format!(
                        "{}\t{}\t100",
                        bushido / 120,
                        bushido / 50
                    ))),
            );
        } else {
            add_buff(
                w,
                m,
                BuffInfo::new(BuffIcon::Confidence, 1060596).duration(TimeSpan::from_secs(30)),
            );
        }
    }
}

fn stop_confidence_timer(w: &mut World, m: MobileId) -> bool {
    if let Some(mut t) = tables(w).table.shift_remove(&m) {
        t.cancel(w);
        return true;
    }
    false
}

/// `EndConfidence(m)`.
pub fn end_confidence(w: &mut World, m: MobileId) {
    stop_regenerating(w, m);

    if stop_confidence_timer(w, m) {
        on_effect_end(w, m, "Server.Spells.Bushido.Confidence");

        if is_player_mobile(w, m) {
            remove_buff(w, m, BuffIcon::Confidence);
        }
    }
}

/// `IsRegenerating(m)`.
pub fn is_regenerating(w: &mut World, m: MobileId) -> bool {
    tables(w).regen.contains_key(&m)
}

// TODO: Move this to a central regeneration so it actually works properly
/// `BeginRegenerating(m)`.
pub fn begin_regenerating(w: &mut World, m: MobileId) {
    stop_regenerating(w, m);

    let mut ticks = 0;
    let h = timer::start(
        w,
        TimeSpan::from_secs(1),
        TimeSpan::from_secs(1),
        4,
        move |w| {
            ticks += 1;
            // `timerToken.RemainingCount == 0` on the last tick: the table entry goes, the heal still happens.
            if ticks == 4 {
                tables(w).regen.shift_remove(&m);
            }

            // RunUO says this goes for 5 seconds, but UOGuide says 4 seconds during normal regeneration
            // Divide by 4 because this is per second.
            let b = skills::value(w, m, SkillName::Bushido);
            let hits = ((15.0 + b * b / 576.0) / 4.0) as i32;
            let cur = w.mobile(m).map_or(0, |x| x.hits);
            props_b::set_hits(w, m, cur + hits);
        },
    );
    tables(w).regen.insert(m, h);
}

/// `StopRegenerating(m)`.
pub fn stop_regenerating(w: &mut World, m: MobileId) {
    if let Some(mut t) = tables(w).regen.shift_remove(&m) {
        t.cancel(w);
    }
}
