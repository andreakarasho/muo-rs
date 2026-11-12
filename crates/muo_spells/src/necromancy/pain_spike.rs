//! The Pain Spike spell.

use indexmap::IndexMap;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::hooks::hooks;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, helper, reagent};
use muo_io::text::TextDefinition;
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{damage, effects, stats};
use muo_server::skills::SkillName;
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};
use crate::common::{alive, is_player_mobile, mobile_of, turn_to};

static PAIN_SPIKE: NecroType = NecroType {
    required_skill: 20.0,
    required_mana: 5,
    ..NECRO_BASE
};

/// `PainSpikeSpell`: In Sar.
pub static PAIN_SPIKE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.PainSpikeSpell",
    parent: Some(&NECROMANCER_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Pain Spike",
        "In Sar",
        &[reagent::GRAVE_DUST, reagent::PIG_IRON],
    )
    .action(203)
    .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_secs(1),
    delayed_damage: false,
    on_cast,
    family: Some(&PAIN_SPIKE),
    ..NECROMANCER_SPELL
};

struct Entry {
    timer: TimerHandle,
}

#[derive(Default)]
struct Table(IndexMap<MobileId, Entry>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, Entry> {
    &mut w.resources.get_mut::<Table>().0
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

        // SpellHelper.CheckReflect( (int)this.Circle, Caster, ref m ); //Irrelevent after AoS
        /* Temporarily causes intense physical pain to the target, dealing direct damage. After 10 seconds the
         * spell wears off, and if the target is still alive, some of the Hit Points lost through Pain Spike are
         * restored.
         */
        effects::fixed_particles_hue(w, m, 0x37C4, 1, 8, 9916, 39, 3, EffectLayer::Head);
        effects::fixed_particles_hue(w, m, 0x37C4, 1, 8, 9502, 39, 4, EffectLayer::Head);
        effects::play_sound(w, m, 0x210);

        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        let player = w.mobile(m).is_some_and(|x| x.player);
        let mut dmg = (((ty.get_damage_skill)(w, sp, caster) - (ty.get_resist_skill)(w, sp, m))
            / 10.0
            + if player { 18.0 } else { 30.0 })
        .max(1.0);

        stats::check_skill(w, m, SkillName::MagicResist, 0.0, 120.0); // Skill check for gain

        let mut buff_time = TimeSpan::from_secs(10);

        if let Some(existing) = table(w).get(&m).map(|e| e.timer) {
            dmg = f64::from(utility::random_min_max(&mut w.rng, 3, 7));
            // `timer.Delay += 2s` does not move a timer that is already scheduled, so only `Next` is read.
            let next = existing.next(w);
            buff_time = next - w.now;
        } else {
            let to_restore = dmg as i32;
            let h = timer::start(w, TimeSpan::from_secs(10), TimeSpan::ZERO, 1, move |w| {
                table(w).shift_remove(&m);

                if alive(w, m) && !mb::is_dead_bonded_pet(w, m) {
                    let hits = w.mobile(m).map_or(0, |x| x.hits);
                    props_b::set_hits(w, m, hits + to_restore);
                }

                if is_player_mobile(w, m) {
                    remove_buff(w, m, BuffIcon::PainSpike);
                }
            });
            table(w).insert(m, Entry { timer: h });
        }

        if is_player_mobile(w, m) {
            add_buff(
                w,
                m,
                BuffInfo::new(BuffIcon::PainSpike, 1075667)
                    .duration(buff_time)
                    .args(TextDefinition::of_text((dmg as i32).to_string())),
            );
        }

        // TODO: Find a better way to do this
        if let Some(f) = hooks(w).and_then(|h| h.set_dfa) {
            f(w, 1); // DFAlgorithm.PainSpike
        }
        damage::damage(w, m, dmg as i32, Some(caster), true, true);
        helper::do_leech(w, dmg as i32, caster, m);
        if let Some(f) = hooks(w).and_then(|h| h.set_dfa) {
            f(w, 0); // DFAlgorithm.Standard
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
