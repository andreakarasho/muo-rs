//! The Evil Omen spell.

use indexmap::IndexMap;
use muo_content::creature;
use muo_content::hooks::hooks_mut as content_hooks_mut;
use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::targeting::{SpellTargetKind, begin_spell_target};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::base as mb;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::mods::{ModId, SkillMod};
use muo_server::skills::{self, SkillName};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::timer;

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};
use crate::common::{is_player_mobile, mobile_of, turn_to};

static EVIL_OMEN: NecroType = NecroType {
    required_skill: 20.0,
    required_mana: 11,
    ..NECRO_BASE
};

/// `EvilOmenSpell`: Pas Tym An Sanct.
pub static EVIL_OMEN_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.EvilOmenSpell",
    parent: Some(&NECROMANCER_SPELL),
    target_range: Some(muo_content::spells::targeting::target_range),
    info: SpellInfo::new(
        "Evil Omen",
        "Pas Tym An Sanct",
        &[reagent::BAT_WING, reagent::NOX_CRYSTAL],
    )
    .action(203)
    .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_millis(750),
    on_cast,
    family: Some(&EVIL_OMEN),
    ..NECROMANCER_SPELL
};

#[derive(Default)]
struct Table(IndexMap<MobileId, ModId>);

fn table(w: &mut World) -> &mut IndexMap<MobileId, ModId> {
    &mut w.resources.get_mut::<Table>().0
}

/// Sets `Hooks::evil_omen_end_effect`.
pub fn init(w: &mut World) {
    content_hooks_mut(w).evil_omen_end_effect = Some(end_effect);
}

fn target(w: &mut World, sp: SpellId, targeted: Targeted) {
    let Some(m) = mobile_of(targeted) else {
        return;
    };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if !(creature::is_creature(w, m) || is_player_mobile(w, m)) {
        mm::send_localized_message(w, caster, 1060508, "", 0x3B2); // You can't curse that.
    } else if spell::check_h_sequence(w, sp, m) {
        turn_to(w, caster, m);

        /* Curses the target so that the next harmful event that affects them is magnified.
         * Damage to the target's hit points is increased 25%, the poison level of the attack will be 1 higher
         * and the Resist Magic skill of the target will be fixed on 50.
         *
         * The effect lasts for one harmful event only.
         */
        spell::caster_hurt(w, m);

        effects::play_sound(w, m, 0xFC);
        effects::fixed_particles_hue(w, m, 0x3728, 1, 13, 9912, 1150, 7, EffectLayer::Head);
        effects::fixed_particles_hue(w, m, 0x3779, 1, 15, 9502, 67, 7, EffectLayer::Head);

        if !table(w).contains_key(&m) {
            let half = skills::value(w, m, SkillName::MagicResist) / 2.0;
            let sm =
                SkillMod::new_default(w, SkillName::MagicResist, Some("EvilOmen"), false, half);
            let id = sm.id;
            mb::add_skill_mod(w, m, sm);
            table(w).insert(m, id);
        }

        let duration =
            TimeSpan::from_secs_f64(skills::value(w, caster, SkillName::SpiritSpeak) / 12.0 + 1.0);
        timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
            end_effect(w, m);
        });

        spell::harmful_spell(w, sp, m);

        if is_player_mobile(w, m) {
            add_buff(
                w,
                m,
                BuffInfo::with_secondary(BuffIcon::EvilOmen, 1075647, 1075648).duration(duration),
            );
        }
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

/// `EndEffect(m)`.
pub fn end_effect(w: &mut World, m: MobileId) -> bool {
    if let Some(id) = table(w).shift_remove(&m) {
        mb::remove_skill_mod(w, m, id);
        return true;
    }
    false
}
