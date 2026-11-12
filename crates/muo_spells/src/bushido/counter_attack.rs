//! The Counter Attack spell.

use indexmap::IndexMap;
use muo_content::items::armor::BASE_SHIELD;
use muo_content::items::weapon::BASE_WEAPON;
use muo_content::items::weapon_hooks::WeaponHooks;
use muo_content::spells::SpellInfo;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::layer::Layer;
use muo_server::mobile::part2::{effects, items, messages as mm};
use muo_server::timer::{self, TimerHandle};

use super::samurai::{SAMURAI_SPELL, SamuraiType, on_cast_successful, on_effect_end};

static COUNTER_ATTACK: SamuraiType = SamuraiType {
    required_skill: 40.0,
    required_mana: 5,
};

/// `CounterAttack`: CounterAttack.
pub static COUNTER_ATTACK_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Bushido.CounterAttack",
    parent: Some(&SAMURAI_SPELL),
    info: SpellInfo::new("CounterAttack", "", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_millis(250),
    on_cast,
    on_begin_cast,
    check_cast,
    family: Some(&COUNTER_ATTACK),
    ..SAMURAI_SPELL
};

#[derive(Default)]
struct Table(IndexMap<MobileId, TimerHandle>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, TimerHandle> {
    &mut w.resources.get_mut::<Table>().0
}

/// Sets the weapon hooks.
pub fn init(w: &mut World) {
    let h = w.resources.get_mut::<WeaponHooks>();
    h.counter_attack_is_countering = Some(|w, m| {
        w.resources
            .get::<Table>()
            .is_some_and(|t| t.0.contains_key(&m))
    });
    h.counter_attack_stop = Some(stop_countering);
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    if !(SAMURAI_SPELL.check_cast)(w, sp) {
        return false;
    }
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };

    let one = items::find_item_on_layer(w, caster, Layer::OneHanded);
    if one.is_some_and(|i| w.item(i).is_some_and(|x| x.ty.is(&BASE_WEAPON))) {
        return true;
    }

    let two = items::find_item_on_layer(w, caster, Layer::TwoHanded);
    if two.is_some_and(|i| {
        w.item(i)
            .is_some_and(|x| x.ty.is(&BASE_SHIELD) || x.ty.is(&BASE_WEAPON))
    }) {
        return true;
    }

    mm::send_localized_message(w, caster, 1062944, "", 0x3B2); // You must have a weapon or a shield equipped to use this ability!
    false
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
        mm::send_localized_message(w, caster, 1063118, "", 0x3B2); // You prepare to respond immediately to the next blocked blow.

        on_cast_successful(w, sp);

        start_countering(w, caster);
    }

    spell::finish_sequence(w, sp);
}

/// `IsCountering(m)`.
pub fn is_countering(w: &mut World, m: MobileId) -> bool {
    table(w).contains_key(&m)
}

fn stop_counter_timer(w: &mut World, m: MobileId) -> bool {
    if let Some(mut t) = table(w).shift_remove(&m) {
        t.cancel(w);
        return true;
    }
    false
}

/// `StartCountering(m)`.
pub fn start_countering(w: &mut World, m: MobileId) {
    stop_counter_timer(w, m);

    let h = timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
        stop_countering(w, m);
        mm::send_localized_message(w, m, 1063119, "", 0x3B2); // You return to your normal stance.
    });
    table(w).insert(m, h);
}

/// `StopCountering(m)`.
pub fn stop_countering(w: &mut World, m: MobileId) {
    if stop_counter_timer(w, m) {
        on_effect_end(w, m, "Server.Spells.Bushido.CounterAttack");
    }
}
