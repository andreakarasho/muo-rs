//! The Mana Drain spell.

use indexmap::IndexSet;
use muo_content::core;
use muo_content::spells::helper;
use muo_content::spells::magery::{MAGERY_BASE, MAGERY_SPELL, MageryType, check_resisted};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellCircle, SpellInfo, reagent};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer;

use crate::common::{alive, mobile_of, turn_to};

/// `GetResistPercent(target) => 99.0`.
static MANA_DRAIN_FAMILY: MageryType = MageryType {
    circle: SpellCircle::Fourth,
    get_resist_percent: |_, _, _| 99.0,
    ..MAGERY_BASE
};

/// `ManaDrainSpell`: Ort Rel.
pub static MANA_DRAIN_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Fourth.ManaDrainSpell",
    parent: Some(&MAGERY_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Mana Drain",
        "Ort Rel",
        &[
            reagent::BLACK_PEARL,
            reagent::MANDRAKE_ROOT,
            reagent::SPIDERS_SILK,
        ],
    )
    .action(215)
    .hand_effect(9031),
    on_cast,
    family: Some(&MANA_DRAIN_FAMILY),
    ..MAGERY_SPELL
};

/// `_table`.
#[derive(Default)]
struct Table(IndexSet<MobileId>);

fn table(w: &mut World) -> &mut IndexSet<MobileId> {
    &mut w.resources.get_mut::<Table>().0
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(mut m) = mobile_of(targeted) else {
        return;
    };
    if spell::check_h_sequence(w, sp, m) {
        let Some(mut caster) = spell::caster(w, sp) else {
            return;
        };
        turn_to(w, caster, m);
        helper::check_reflect(w, SpellCircle::Fourth as i32, &mut caster, &mut m);
        let caster = spell::caster(w, sp).unwrap_or(caster);

        spell::caster_hurt(w, m);
        mb::set_paralyzed(w, m, false);

        let mana = w.mobile(m).map_or(0, |x| x.mana);
        if core::aos(w) {
            let ty = spell::spell_type(w, sp).expect("the spell is casting");
            let dmg = (ty.get_damage_skill)(w, sp, caster);
            let resist = (ty.get_resist_skill)(w, sp, m);
            let mut to_drain = (40 + (dmg - resist) as i32).clamp(0, mana);

            if table(w).contains(&m) {
                to_drain = 0;
            }

            effects::fixed_particles(w, m, 0x3789, 10, 25, 5032, EffectLayer::Head);
            effects::play_sound(w, m, 0x1F8);

            if to_drain > 0 {
                props_b::set_mana(w, m, mana - to_drain);

                table(w).insert(m);
                timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
                    aos_delay_callback(w, m, to_drain)
                });
            }
        } else {
            if check_resisted(w, sp, m) {
                mm::send_localized_message(w, m, 501783, "", 0x3B2); // You feel yourself resisting magical energy.
            } else if mana > 0 {
                let cut = utility::random_from(&mut w.rng, 1, mana.min(100));
                props_b::set_mana(w, m, mana - cut);
            }

            effects::fixed_particles(w, m, 0x374A, 10, 15, 5032, EffectLayer::Head);
            effects::play_sound(w, m, 0x1F8);
        }

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

fn aos_delay_callback(w: &mut World, m: MobileId, mana: i32) {
    if alive(w, m) && !mb::is_dead_bonded_pet(w, m) {
        let cur = w.mobile(m).map_or(0, |x| x.mana);
        props_b::set_mana(w, m, cur + mana);
        effects::fixed_effect(w, m, 0x3779, 10, 25);
        effects::play_sound(w, m, 0x28E);
    }
    table(w).shift_remove(&m);
}
