//! The Corpse Skin spell.

use indexmap::IndexMap;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff, remove_buff};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::ResistanceType;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm, stats};
use muo_server::mods::{ModId, ResistanceMod};
use muo_server::skills::SkillName;
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};
use crate::common::{is_player_mobile, mobile_of, turn_to};

static CORPSE_SKIN: NecroType = NecroType {
    required_skill: 20.0,
    required_mana: 11,
    ..NECRO_BASE
};

/// `CorpseSkinSpell`: In Agle Corp Ylem.
pub static CORPSE_SKIN_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.CorpseSkinSpell",
    parent: Some(&NECROMANCER_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Corpse Skin",
        "In Agle Corp Ylem",
        &[reagent::BAT_WING, reagent::GRAVE_DUST],
    )
    .action(203)
    .hand_effect(9051),
    cast_delay_base: |_, _| TimeSpan::from_millis(1500),
    on_cast,
    family: Some(&CORPSE_SKIN),
    ..NECROMANCER_SPELL
};

struct Entry {
    timer: TimerHandle,
    mods: Vec<ModId>,
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

        /* Transmogrifies the flesh of the target creature or player to resemble rotted corpse flesh, making them
         * more vulnerable to Fire and Poison damage, but increasing their resistance to Physical and Cold damage.
         *
         * The effect lasts for ((Spirit Speak skill level - target's Resist Magic skill level) / 25 ) + 40 seconds.
         *
         * NOTE: Algorithm above is fixed point, should be: ((ss-mr)/2.5) + 40
         *
         * NOTE: Resistance is not checked if targeting yourself
         */
        if table(w).contains_key(&m) {
            do_expire(w, m);
        } else {
            mm::send_localized_message(w, m, 1061689, "", 0x3B2); // Your skin turns dry and corpselike.
        }

        spell::caster_hurt(w, m);

        effects::fixed_particles_hue(w, m, 0x373A, 1, 15, 9913, 67, 7, EffectLayer::Head);
        effects::play_sound(w, m, 0x1BB);

        let ty = spell::spell_type(w, sp).expect("the spell is casting");
        let ss = (ty.get_damage_skill)(w, sp, caster);
        let mr = if caster == m {
            0.0
        } else {
            (ty.get_resist_skill)(w, sp, m)
        };
        stats::check_skill(w, m, SkillName::MagicResist, 0.0, 120.0); // Skill check for gain

        let duration = TimeSpan::from_secs_f64((ss - mr) / 2.5 + 40.0);

        let defs = [
            (ResistanceType::Fire, "FireResistCorpseSkinSpell", -15),
            (ResistanceType::Poison, "PoisonResistCorpseSkinSpell", -15),
            (ResistanceType::Cold, "ColdResistCorpseSkinSpell", 10),
            (
                ResistanceType::Physical,
                "PhysicalResistCorpseSkinSpell",
                10,
            ),
        ];
        let mut ids = Vec::new();
        let mut mods = Vec::new();
        for (t, name, offset) in defs {
            let rm = ResistanceMod::new(w, t, Some(name), offset, None);
            ids.push(rm.id);
            mods.push(rm);
        }

        let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
            mm::send_localized_message(w, m, 1061688, "", 0x3B2); // Your skin returns to normal.
            do_expire(w, m);
        });

        if is_player_mobile(w, m) {
            add_buff(
                w,
                m,
                BuffInfo::new(BuffIcon::CorpseSkin, 1075663).duration(duration),
            );
        }

        table(w).insert(
            m,
            Entry {
                timer: h,
                mods: ids,
            },
        );

        for rm in mods {
            mb::add_resistance_mod(w, m, rm);
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

/// `RemoveCurse(m)`.
pub fn remove_curse(w: &mut World, m: MobileId) -> bool {
    if !table(w).contains_key(&m) {
        return false;
    }

    mm::send_localized_message(w, m, 1061688, "", 0x3B2); // Your skin returns to normal.
    do_expire(w, m);
    true
}

/// `ExpireTimer.DoExpire()`.
fn do_expire(w: &mut World, m: MobileId) {
    let Some(mut e) = table(w).shift_remove(&m) else {
        return;
    };
    for id in e.mods.drain(..) {
        mb::remove_resistance_mod(w, m, id);
    }
    e.timer.cancel(w);
    if is_player_mobile(w, m) {
        remove_buff(w, m, BuffIcon::CorpseSkin);
    }
}
