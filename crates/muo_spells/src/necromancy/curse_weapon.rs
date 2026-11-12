//! The Curse Weapon spell.

use indexmap::IndexMap;
use muo_content::items::{is_named, weapon, weapon_hooks};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::ItemId;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::{self, SkillName};
use muo_server::timer::{self, TimerHandle};

use super::necromancer::{NECRO_BASE, NECROMANCER_SPELL, NecroType};

static CURSE_WEAPON: NecroType = NecroType {
    required_skill: 0.0,
    required_mana: 7,
    ..NECRO_BASE
};

/// `CurseWeaponSpell`: An Sanct Gra Char.
pub static CURSE_WEAPON_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.CurseWeaponSpell",
    parent: Some(&NECROMANCER_SPELL),
    info: SpellInfo::new("Curse Weapon", "An Sanct Gra Char", &[reagent::PIG_IRON])
        .action(203)
        .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_millis(750),
    on_cast,
    family: Some(&CURSE_WEAPON),
    ..NECROMANCER_SPELL
};

#[derive(Default)]
struct Table(IndexMap<ItemId, TimerHandle>);

fn table(w: &mut World) -> &mut IndexMap<ItemId, TimerHandle> {
    &mut w.resources.get_mut::<Table>().0
}

fn set_cursed(w: &mut World, wp: ItemId, v: bool) {
    if let Some(d) = weapon::data_mut(w, wp) {
        d.cursed = v;
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    let wp = weapon_hooks::weapon_item(w, caster).filter(|&i| {
        !w.item(i)
            .is_some_and(|x| is_named(x.ty, "Server.Items.Fists"))
    });
    let Some(wp) = wp else {
        mm::send_localized_message(w, caster, 501078, "", 0x3B2); // You must be holding a weapon.
        spell::finish_sequence(w, sp);
        return;
    };

    if spell::check_sequence(w, sp) {
        /* Temporarily imbues a weapon with a life draining effect.
         * Half the damage that the weapon inflicts is added to the necromancer's health.
         * The effects lasts for (Spirit Speak skill level / 34) + 1 seconds.
         *
         * NOTE: Above algorithm is fixed point, should be : (Spirit Speak skill level / 3.4) + 1
         *
         * TODO: What happens if you curse a weapon then give it to someone else? Should they get the drain effect?
         */
        effects::play_sound(w, caster, 0x387);
        effects::fixed_particles_hue(w, caster, 0x3779, 1, 15, 9905, 32, 2, EffectLayer::Head);
        effects::fixed_particles_hue(w, caster, 0x37B9, 1, 14, 9502, 32, 5, EffectLayer(255));

        timer::start(w, TimeSpan::from_millis(750), TimeSpan::ZERO, 1, move |w| {
            effects::play_sound(w, caster, 0xFA)
        });

        let duration =
            TimeSpan::from_secs_f64(skills::value(w, caster, SkillName::SpiritSpeak) / 3.4 + 1.0);

        if let Some(mut t) = table(w).shift_remove(&wp) {
            t.cancel(w);
        }
        set_cursed(w, wp, true);
        let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
            set_cursed(w, wp, false);
            let map = w.item(wp).and_then(|i| i.map);
            let wl = muo_server::item::ops::get_world_location(w, wp);
            muo_server::effects::play_sound(w, wl, map, 0xFA);
            table(w).shift_remove(&wp);
        });
        table(w).insert(wp, h);
    }

    spell::finish_sequence(w, sp);
}
