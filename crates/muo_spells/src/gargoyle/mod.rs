//! `FlySpell` (Gargoyle Flight), which is cast, not registered.

use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff};
use muo_content::spells::spell::{self, SPELL_BASE, SpellId, SpellType};
use muo_content::spells::{DisturbType, SpellInfo};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::messages as mm;

use crate::common::is_player_mobile;

/// `FlySpell`: Gargoyle Flight.
pub static FLY_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.FlySpell",
    parent: Some(&SPELL_BASE),
    info: SpellInfo::new("Gargoyle Flight", "", &[])
        .action(-1)
        .hand_effect(9002),
    clear_hands_on_cast: false,
    reveal_on_cast: false,
    cast_delay_fast_scalar: 0.0,
    cast_delay_base: |_, _| TimeSpan::from_millis(250),
    get_cast_recovery: |_, _| TimeSpan::ZERO,
    get_mana: |_, _| 0,
    consume_reagents: |_, _| true,
    check_fizzle: |_, _| true,
    check_disturb: |_, _, t, _, _| t != DisturbType::EquipRequest && t != DisturbType::UseRequest,
    do_hurt_fizzle: |_, _| {},
    do_fizzle: |_, _| {},
    on_disturb,
    on_cast,
    ..SPELL_BASE
};

/// `m_Stop`.
#[derive(Clone, Copy, Default)]
struct Stopped(bool);

fn on_disturb(w: &mut World, sp: SpellId, _t: DisturbType, message: bool) {
    let stopped = spell::spell(w, sp)
        .and_then(|s| s.ext.get::<Stopped>().copied())
        .is_some_and(|s| s.0);
    if message
        && !stopped
        && let Some(caster) = spell::caster(w, sp)
    {
        mm::send_localized_message(w, caster, 1113192, "", 0x3B2); // You have been disrupted while attempting to fly!
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    props_b::set_flying(w, caster, false);
    mb::animate(w, caster, 60, 10, 1, true, false, 0);
    mm::send_localized_message(w, caster, 1112567, "", 0x3B2); // You are flying.
    props_b::set_flying(w, caster, true);
    if is_player_mobile(w, caster) {
        add_buff(w, caster, BuffInfo::new(BuffIcon::Fly, 1112567));
    }
    spell::finish_sequence(w, sp);
}

/// `Stop()`.
fn stop(w: &mut World, sp: SpellId) {
    if let Some(s) = spell::spells(w).get_mut(sp) {
        s.ext = muo_server::Ext::new(Stopped(true));
    }
    spell::disturb(w, sp, DisturbType::Hurt, false, false);
}

/// The `fly_cast` hook: stops a running `FlySpell` of `m`, then `new FlySpell(m).Cast()`.
pub fn fly_cast(w: &mut World, m: MobileId) {
    if let Some(s) = w.mobile(m).and_then(|x| x.spell)
        && spell::spell_type(w, s).is_some_and(|t| std::ptr::eq(t, &FLY_SPELL))
    {
        stop(w, s);
    }

    let sp = spell::new_spell(w, &FLY_SPELL, m, None);
    spell::cast(w, sp);
}
