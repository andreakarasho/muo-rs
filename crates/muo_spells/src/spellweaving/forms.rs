//! The Reaper Form and Ethereal Voyage spells.

use muo_content::items::weapon_hooks::WeaponHooks;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::SpellInfo;
use muo_content::spells::helper::{self, TransformationInfo};
use muo_content::spells::hooks::hooks_mut;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::combat;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::packets::movement::{SpeedControlSetting, send_speed_control};
use muo_server::skills::{self, SkillName};
use muo_server::timer;

use super::arcanist::{ARCANE_FORM, ArcaneType, caster_focus_level};
use crate::common::is_player_mobile;

fn nop(_: &mut World, _: MobileId) {}

fn send_speed(w: &mut World, m: MobileId, s: SpeedControlSetting) {
    if let Some(ns) = w.mobile(m).and_then(|x| x.net_state) {
        send_speed_control(w, ns, s);
    }
}

// ---- ReaperFormSpell ----

const REAPER_BASE: TransformationInfo = TransformationInfo {
    body: 0x11D,
    hue: 0,
    phys_resist_offset: 5,
    fire_resist_offset: -25,
    cold_resist_offset: 5,
    pois_resist_offset: 5,
    nrgy_resist_offset: 5,
    tick_rate: 1.0,
    on_tick: nop,
    do_effect: |w, m| {
        effects::play_sound(w, m, 0x1BA);
        send_speed(w, m, SpeedControlSetting::Walk);
    },
    remove_effect: |w, m| send_speed(w, m, SpeedControlSetting::Disable),
    cast_skill: SkillName::Spellweaving,
    min_skill: |_, _| 24.0,
};

const fn reaper(focus: i32) -> TransformationInfo {
    TransformationInfo {
        phys_resist_offset: 5 + focus,
        cold_resist_offset: 5 + focus,
        pois_resist_offset: 5 + focus,
        nrgy_resist_offset: 5 + focus,
        ..REAPER_BASE
    }
}

/// One `TransformationInfo` per focus level (`5 + FocusLevel` resistances).
static REAPER_FORMS: [TransformationInfo; 7] = [
    reaper(0),
    reaper(1),
    reaper(2),
    reaper(3),
    reaper(4),
    reaper(5),
    reaper(6),
];

const REAPER_NAME: &str = "Server.Spells.Spellweaving.ReaperFormSpell";

static REAPER_FORM: ArcaneType = ArcaneType {
    required_skill: 24.0,
    required_mana: 34,
    transformation: Some(|_, focus| &REAPER_FORMS[focus.clamp(0, 6) as usize]),
};

/// `ReaperFormSpell`: Tarisstree.
pub static REAPER_FORM_SPELL: SpellType = SpellType {
    full_name: REAPER_NAME,
    parent: Some(&ARCANE_FORM),
    info: SpellInfo::new("Reaper Form", "Tarisstree", &[]).action(-1),
    cast_delay_base: |_, _| TimeSpan::from_millis(2500),
    family: Some(&REAPER_FORM),
    ..ARCANE_FORM
};

/// The focus level of the active Reaper Form of `m`: `SwingSpeedBonus` and `SpellDamageBonus` are `10 + FocusLevel`.
fn reaper_focus(w: &World, m: MobileId) -> Option<i32> {
    let c = helper::get_context(w, m)?;
    if c.ty != REAPER_NAME {
        return None;
    }
    REAPER_FORMS
        .iter()
        .position(|f| std::ptr::eq(f, c.spell))
        .map(|i| i as i32)
}

/// `OnLogin(pm)` and the hooks.
pub fn init(w: &mut World) {
    w.events
        .aggressive_action
        .subscribe(|w, e| remove_transformation_on_aggressive_action(w, e.aggressor));
    let h = w.resources.get_mut::<WeaponHooks>();
    h.reaper_form_swing_bonus = Some(|w, m| reaper_focus(w, m).map_or(0, |f| 10 + f));
    h.reaper_form_spell_damage_bonus = Some(|w, m| reaper_focus(w, m).map_or(0, |f| 10 + f));
    hooks_mut(w).reaper_form_damage_bonus = Some(|w, m| reaper_focus(w, m).map_or(0, |f| 10 + f));

    muo_content::network::events::subscribe_ranked::<muo_content::network::events::PlayerLogin>(
        w,
        24,
        |w, e| {
            let pm = e.0;
            if helper::under_transformation_of(w, pm, REAPER_NAME) {
                send_speed(w, pm, SpeedControlSetting::Walk);
            }
        },
    );
}

// ---- EtherealVoyageSpell ----

const VOYAGE_NAME: &str = "Server.Spells.Spellweaving.EtherealVoyageSpell";

fn voyage_do_effect(w: &mut World, m: MobileId) {
    effects::play_sound(w, m, 0x5C8);
    mm::send_localized_message(w, m, 1074770, "", 0x3B2); // You are now under the effects of Ethereal Voyage.

    let skill = skills::value(w, m, SkillName::Spellweaving);
    let focus = caster_focus_level(w, m);
    let duration = TimeSpan::from_secs(12 + (skill / 24.0) as i64 + i64::from(focus) * 2);

    timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
        voyage_remove_effect(w, m)
    });

    // Cannot cast this spell for another 5 minutes(300sec) after effect removed.
    combat::begin_action(w, m, VOYAGE_NAME);

    if is_player_mobile(w, m) {
        add_buff(
            w,
            m,
            BuffInfo::with_secondary(BuffIcon::EtherealVoyage, 1031613, 1075805).duration(duration),
        );
    }
}

fn voyage_remove_effect(w: &mut World, m: MobileId) {
    mm::send_localized_message(w, m, 1074771, "", 0x3B2); // You are no longer under the effects of Ethereal Voyage.
    helper::remove_context(w, m, true);

    timer::start(w, TimeSpan::from_mins(5), TimeSpan::ZERO, 1, move |w| {
        combat::end_action(w, m, VOYAGE_NAME)
    });

    if is_player_mobile(w, m) {
        remove_buff(w, m, BuffIcon::EtherealVoyage);
    }
}

static VOYAGE_FORM: TransformationInfo = TransformationInfo {
    body: 0x302,
    hue: 0x48F,
    phys_resist_offset: 0,
    fire_resist_offset: 0,
    cold_resist_offset: 0,
    pois_resist_offset: 0,
    nrgy_resist_offset: 0,
    tick_rate: 1.0,
    on_tick: nop,
    do_effect: voyage_do_effect,
    remove_effect: voyage_remove_effect,
    cast_skill: SkillName::Spellweaving,
    min_skill: |_, _| 24.0,
};

static ETHEREAL_VOYAGE: ArcaneType = ArcaneType {
    required_skill: 24.0,
    required_mana: 32,
    transformation: Some(|_, _| &VOYAGE_FORM),
};

/// `EtherealVoyageSpell`: Orlavdra.
pub static ETHEREAL_VOYAGE_SPELL: SpellType = SpellType {
    full_name: VOYAGE_NAME,
    parent: Some(&ARCANE_FORM),
    info: SpellInfo::new("Ethereal Voyage", "Orlavdra", &[]).action(-1),
    cast_delay_base: |_, _| TimeSpan::from_millis(3500),
    check_cast: voyage_check_cast,
    family: Some(&ETHEREAL_VOYAGE),
    ..ARCANE_FORM
};

fn voyage_check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if helper::under_transformation_of(w, caster, VOYAGE_NAME) {
        mm::send_localized_message(w, caster, 501775, "", 0x3B2); // This spell is already in effect.
    } else if !combat::can_begin_action(w, caster, VOYAGE_NAME) {
        mm::send_localized_message(w, caster, 1075124, "", 0x3B2); // You must wait before casting that spell again.
    } else if w.mobile(caster).is_some_and(|c| c.combatant.is_some()) {
        mm::send_localized_message(w, caster, 1072586, "", 0x3B2); // You cannot cast Ethereal Voyage while you are in combat.
    } else {
        return (ARCANE_FORM.check_cast)(w, sp);
    }

    false
}

/// `RemoveTransformationOnAggressiveAction(e)`: call it from the aggressive-action event with the aggressor.
pub fn remove_transformation_on_aggressive_action(w: &mut World, aggressor: MobileId) {
    if helper::under_transformation_of(w, aggressor, VOYAGE_NAME) {
        helper::remove_context(w, aggressor, true);
    }
}
