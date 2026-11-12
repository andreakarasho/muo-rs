//! The Enemy Of One spell.

use indexmap::IndexMap;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff};
use muo_content::player_mobile::{self, members};
use muo_content::spells::SpellInfo;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::effects;
use muo_server::timer::{self, TimerHandle};

use super::paladin::{PALADIN_BASE, PALADIN_SPELL, PaladinType, power};
use crate::common::is_player_mobile;

static ENEMY_OF_ONE: PaladinType = PaladinType {
    required_skill: 45.0,
    required_mana: 20,
    required_tithing: 10,
    mantra_number: 1060723, // Forul Solum
    ..PALADIN_BASE
};

/// `EnemyOfOneSpell`: Forul Solum.
pub static ENEMY_OF_ONE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Chivalry.EnemyOfOneSpell",
    parent: Some(&PALADIN_SPELL),
    info: SpellInfo::new("Enemy of One", "Forul Solum", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_millis(500),
    on_cast,
    blocks_movement: |_, _| false,
    family: Some(&ENEMY_OF_ONE),
    ..PALADIN_SPELL
};

#[derive(Default)]
struct Table(IndexMap<MobileId, TimerHandle>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, TimerHandle> {
    &mut w.resources.get_mut::<Table>().0
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_sequence(w, sp) {
        effects::play_sound(w, caster, 0x0F5);
        effects::play_sound(w, caster, 0x1ED);
        effects::fixed_particles_hue(w, caster, 0x375A, 1, 30, 9966, 33, 2, EffectLayer::Head);
        effects::fixed_particles_hue(w, caster, 0x37B9, 1, 30, 9502, 43, 3, EffectLayer::Head);

        remove_timer(w, caster);

        let delay = (f64::from(power(w, sp, 1)) / 60.0).clamp(1.5, 3.5);
        let duration = TimeSpan::from_mins_f64(delay);

        let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
            expire_callback(w, caster)
        });
        table(w).insert(caster, h);

        if is_player_mobile(w, caster) {
            members::set_enemy_of_one_type(w, caster, None);
            if let Some(d) = player_mobile::data_mut(w, caster) {
                d.waiting_for_enemy = true;
            }
            add_buff(
                w,
                caster,
                BuffInfo::with_secondary(BuffIcon::EnemyOfOne, 1075653, 1044111).duration(duration),
            );
        }
    }

    spell::finish_sequence(w, sp);
}

fn remove_timer(w: &mut World, m: MobileId) {
    if let Some(mut t) = table(w).shift_remove(&m) {
        t.cancel(w);
    }
}

fn expire_callback(w: &mut World, m: MobileId) {
    remove_timer(w, m);

    effects::play_sound(w, m, 0x1F8);

    if is_player_mobile(w, m) {
        members::set_enemy_of_one_type(w, m, None);
        if let Some(d) = player_mobile::data_mut(w, m) {
            d.waiting_for_enemy = false;
        }
    }
}
