//! The Mirror Image spell and the clone bookkeeping.

use indexmap::IndexMap;
use muo_content::items::weapon_hooks::WeaponHooks;
use muo_content::spells::SpellInfo;
use muo_content::spells::helper;
use muo_content::spells::hooks::hooks;
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, messages as mm};

use super::ninja::{NINJA_SPELL, NinjaType};

static MIRROR_IMAGE: NinjaType = NinjaType {
    required_skill: 20.0,
    required_skill_pre_ml: Some(40.0),
    required_mana: 10,
    required_mana_pre_ml: None,
};

/// `MirrorImage`.
pub static MIRROR_IMAGE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Ninjitsu.MirrorImage",
    parent: Some(&NINJA_SPELL),
    info: SpellInfo::new("Mirror Image", "", &[])
        .action(-1)
        .hand_effect(9002),
    cast_delay_base: |_, _| TimeSpan::from_millis(1500),
    blocked_by_animal_form: false,
    check_cast,
    check_disturb: |_, _, _, _, _| false,
    on_begin_cast,
    on_cast,
    family: Some(&MIRROR_IMAGE),
    ..NINJA_SPELL
};

/// `_cloneCount`.
#[derive(Default)]
struct CloneCount(IndexMap<MobileId, i32>);

fn counts(w: &mut World) -> &mut IndexMap<MobileId, i32> {
    &mut w.resources.get_mut::<CloneCount>().0
}

/// Sets the weapon hooks.
pub fn init(w: &mut World) {
    let h = w.resources.get_mut::<WeaponHooks>();
    h.mirror_image_has_clone = Some(|w, m| {
        w.resources
            .get::<CloneCount>()
            .is_some_and(|c| c.0.contains_key(&m))
    });
    // `foreach (var m in defender.GetMobilesInRange<Clone>(4)) if (m?.SummonMaster == defender)`
    h.mirror_image_clone_in_range = Some(|w, m| {
        muo_server::mobile::part2::enumerators::get_mobiles_in_range(w, m, 4)
            .into_iter()
            .find(|&c| {
                crate::common::mobile_is(w, c, "Server.Mobiles.Clone")
                    && muo_content::creature::control::summon_master(w, c) == Some(m)
            })
    });
}

/// `HasClone(m)`.
pub fn has_clone(w: &mut World, m: MobileId) -> bool {
    counts(w).contains_key(&m)
}

/// `AddClone(m)`.
pub fn add_clone(w: &mut World, m: MobileId) {
    let n = counts(w).get(&m).copied().unwrap_or(0);
    counts(w).insert(m, 1 + n);
}

/// `RemoveClone(m)`.
pub fn remove_clone(w: &mut World, m: MobileId) {
    let count = counts(w).shift_remove(&m).unwrap_or(0);
    if count <= 0 {
        return;
    }
    counts(w).insert(m, count - 1);
}

fn mounted(w: &mut World, m: MobileId) -> bool {
    props_b::mounted(w, m)
}

fn too_many_followers(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|c| c.followers + 1 > c.followers_max)
}

fn horrific(w: &World, m: MobileId) -> bool {
    helper::under_transformation_of(w, m, "Server.Spells.Necromancy.HorrificBeastSpell")
}

fn check_cast(w: &mut World, sp: SpellId) -> bool {
    let Some(caster) = spell::caster(w, sp) else {
        return false;
    };
    if mounted(w, caster) {
        mm::send_localized_message(w, caster, 1063132, "", 0x3B2); // You cannot use this ability while mounted.
        return false;
    }

    if too_many_followers(w, caster) {
        // You cannot summon a mirror image because you have too many followers.
        mm::send_localized_message(w, caster, 1063133, "", 0x3B2);
        return false;
    }

    if horrific(w, caster) {
        mm::send_localized_message(w, caster, 1061091, "", 0x3B2); // You cannot cast that spell in this form.
        return false;
    }

    (NINJA_SPELL.check_cast)(w, sp)
}

fn on_begin_cast(w: &mut World, sp: SpellId) {
    (NINJA_SPELL.on_begin_cast)(w, sp);
    if let Some(caster) = spell::caster(w, sp) {
        mm::send_localized_message(w, caster, 1063134, "", 0x3B2); // You begin to summon a mirror image of yourself.
    }
}

fn on_cast(w: &mut World, sp: SpellId) {
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if mounted(w, caster) {
        mm::send_localized_message(w, caster, 1063132, "", 0x3B2); // You cannot use this ability while mounted.
    } else if too_many_followers(w, caster) {
        // You cannot summon a mirror image because you have too many followers.
        mm::send_localized_message(w, caster, 1063133, "", 0x3B2);
    } else if horrific(w, caster) {
        mm::send_localized_message(w, caster, 1061091, "", 0x3B2); // You cannot cast that spell in this form.
    } else if spell::check_sequence(w, sp) {
        effects::fixed_particles(w, caster, 0x376A, 1, 14, 0x13B5, EffectLayer::Waist);
        effects::play_sound(w, caster, 0x511);

        if let Some(f) = hooks(w).and_then(|h| h.new_mirror_image_clone) {
            f(w, caster);
        }
    }

    spell::finish_sequence(w, sp);
}
