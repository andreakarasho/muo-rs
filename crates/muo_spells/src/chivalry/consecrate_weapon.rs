//! The Consecrate Weapon spell.

use indexmap::IndexMap;
use muo_content::items::{is_named, weapon, weapon_hooks};
use muo_content::spells::SpellInfo;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::effects::{EffectLayer, Targeted as FxTargeted, send_moving_particles_layer};
use muo_server::ids::ItemId;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::skills::SkillName;
use muo_server::timer::{self, TimerHandle};

use super::paladin::{PALADIN_BASE, PALADIN_SPELL, PaladinType, power};

static CONSECRATE_WEAPON: PaladinType = PaladinType {
    required_skill: 15.0,
    required_mana: 10,
    required_tithing: 10,
    mantra_number: 1060720, // Consecrus Arma
    ..PALADIN_BASE
};

/// `ConsecrateWeaponSpell`: Consecrus Arma.
pub static CONSECRATE_WEAPON_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Chivalry.ConsecrateWeaponSpell",
    parent: Some(&PALADIN_SPELL),
    info: SpellInfo::new("Consecrate Weapon", "Consecrus Arma", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_millis(500),
    on_cast,
    blocks_movement: |_, _| false,
    family: Some(&CONSECRATE_WEAPON),
    ..PALADIN_SPELL
};

#[derive(Default)]
struct Table(IndexMap<ItemId, TimerHandle>);

fn table(w: &mut World) -> &mut IndexMap<ItemId, TimerHandle> {
    &mut w.resources.get_mut::<Table>().0
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    // `Caster.Weapon is not (BaseWeapon weapon and not Fists)`
    let weapon_item = weapon_hooks::weapon_item(w, caster).filter(|&i| {
        !w.item(i)
            .is_some_and(|x| is_named(x.ty, "Server.Items.Fists"))
    });
    let Some(wp) = weapon_item else {
        mm::send_localized_message(w, caster, 501078, "", 0x3B2); // You must be holding a weapon.
        spell::finish_sequence(w, sp);
        return;
    };

    if spell::check_sequence(w, sp) {
        /* Temporarily enchants the weapon the caster is currently wielding.
         * The type of damage the weapon inflicts when hitting a target will
         * be converted to the target's worst Resistance type.
         * Duration of the effect is affected by the caster's Karma and lasts for 3 to 11 seconds.
         */
        let (item_id, sound_id) = match weapon::skill(w, wp) {
            SkillName::Macing => (0xFB4, 0x232),
            SkillName::Archery => (0x13B1, 0x145),
            _ => (0xF5F, 0x56),
        };

        effects::play_sound(w, caster, 0x20C);
        effects::play_sound(w, caster, sound_id);
        effects::fixed_particles_hue(w, caster, 0x3779, 1, 30, 9964, 3, 3, EffectLayer::Waist);

        let (loc, map) = w
            .mobile(caster)
            .map(|c| (c.location, c.map))
            .unwrap_or_default();
        let from = FxTargeted::point(Point3D::new(loc.x, loc.y, loc.z), map);
        let to = FxTargeted::point(Point3D::new(loc.x, loc.y, loc.z + 50), map);
        send_moving_particles_layer(
            w,
            from,
            to,
            item_id,
            1,
            0,
            false,
            false,
            33,
            3,
            9501,
            1,
            0,
            i32::from(EffectLayer::Head.0),
            0x100,
        );

        let seconds = f64::from(power(w, sp, 20)).clamp(3.0, 11.0);
        let duration = TimeSpan::from_secs_f64(seconds);

        if let Some(mut t) = table(w).shift_remove(&wp) {
            t.cancel(w);
        }
        set_consecrated(w, wp, true);
        let h = timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
            set_consecrated(w, wp, false);
            let map = w.item(wp).and_then(|i| i.map);
            let wl = muo_server::item::ops::get_world_location(w, wp);
            muo_server::effects::play_sound(w, wl, map, 0x1F8);
            table(w).shift_remove(&wp);
        });
        table(w).insert(wp, h);
    }

    spell::finish_sequence(w, sp);
}

fn set_consecrated(w: &mut World, wp: ItemId, v: bool) {
    if let Some(d) = weapon::data_mut(w, wp) {
        d.consecrated = v;
    }
}
