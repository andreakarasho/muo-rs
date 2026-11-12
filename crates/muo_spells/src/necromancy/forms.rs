//! Horrific Beast, Lich Form, Wraith Form and Vampiric Embrace: the four transformation spells.

use muo_content::spells::helper::{TransformationHooks, TransformationInfo};
use muo_content::spells::spell::{self, SpellId, SpellType};
use muo_content::spells::{SpellInfo, reagent};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::item::effect_item;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::effects;
use muo_server::mobile::{AccessLevel, MobileDelta, base as mb};
use muo_server::skills::{self, SkillName};

use super::necromancer::{NecroType, TRANSFORMATION_SPELL};
use crate::common::is_player_mobile;

fn nop(_: &mut World, _: MobileId) {}

const BASE: TransformationInfo = TransformationInfo {
    body: 0,
    hue: 0,
    phys_resist_offset: 0,
    fire_resist_offset: 0,
    cold_resist_offset: 0,
    pois_resist_offset: 0,
    nrgy_resist_offset: 0,
    tick_rate: 1.0,
    on_tick: nop,
    do_effect: nop,
    remove_effect: nop,
    cast_skill: SkillName::Necromancy,
    min_skill: |_, _| 0.0,
};

// ---- HorrificBeast ----

static HORRIFIC_BEAST: TransformationInfo = TransformationInfo {
    body: 746,
    min_skill: |_, _| 40.0,
    do_effect: |w, m| {
        effects::play_sound(w, m, 0x165);
        effects::fixed_particles_hue(w, m, 0x3728, 1, 13, 9918, 92, 3, EffectLayer::Head);

        mb::delta(w, m, MobileDelta::WEAPON_DAMAGE);
        mb::check_stat_timers(w, m);
    },
    remove_effect: |w, m| mb::delta(w, m, MobileDelta::WEAPON_DAMAGE),
    ..BASE
};

/// `HorrificBeastSpell`: Rel Xen Vas Bal.
pub static HORRIFIC_BEAST_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.HorrificBeastSpell",
    parent: Some(&TRANSFORMATION_SPELL),
    info: SpellInfo::new(
        "Horrific Beast",
        "Rel Xen Vas Bal",
        &[reagent::BAT_WING, reagent::DAEMON_BLOOD],
    )
    .action(203)
    .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_secs(2),
    family: Some(&HORRIFIC_BEAST_TYPE),
    ..TRANSFORMATION_SPELL
};

static HORRIFIC_BEAST_TYPE: NecroType = NecroType {
    required_skill: 40.0,
    required_mana: 11,
    transformation: Some(|_, _| &HORRIFIC_BEAST),
};

// ---- LichForm ----

static LICH_FORM: TransformationInfo = TransformationInfo {
    body: 749,
    min_skill: |_, _| 70.0,
    fire_resist_offset: -10,
    cold_resist_offset: 10,
    pois_resist_offset: 10,
    tick_rate: 2.5,
    do_effect: |w, m| {
        effects::play_sound(w, m, 0x19C);
        effects::fixed_particles_hue(w, m, 0x3709, 1, 30, 9904, 1108, 6, EffectLayer::RightFoot);
    },
    on_tick: |w, m| {
        let hits = w.mobile(m).map_or(0, |x| x.hits);
        props_b::set_hits(w, m, hits - 1);
    },
    ..BASE
};

/// `LichFormSpell`: Rel Xen Corp Ort.
pub static LICH_FORM_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.LichFormSpell",
    parent: Some(&TRANSFORMATION_SPELL),
    info: SpellInfo::new(
        "Lich Form",
        "Rel Xen Corp Ort",
        &[
            reagent::GRAVE_DUST,
            reagent::DAEMON_BLOOD,
            reagent::NOX_CRYSTAL,
        ],
    )
    .action(203)
    .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_secs(2),
    family: Some(&LICH_FORM_TYPE),
    ..TRANSFORMATION_SPELL
};

static LICH_FORM_TYPE: NecroType = NecroType {
    required_skill: 70.0,
    required_mana: 23,
    transformation: Some(|_, _| &LICH_FORM),
};

// ---- WraithForm ----

fn wraith_do_effect(w: &mut World, m: MobileId) {
    if is_player_mobile(w, m) {
        muo_server::mobile::base::set_ignore_mobiles(w, m, true);
    }

    effects::play_sound(w, m, 0x17F);
    effects::fixed_particles_hue(w, m, 0x374A, 1, 15, 9902, 1108, 4, EffectLayer::Waist);
}

fn wraith_remove_effect(w: &mut World, m: MobileId) {
    if is_player_mobile(w, m)
        && w.mobile(m)
            .is_some_and(|x| x.access_level == AccessLevel::Player)
    {
        muo_server::mobile::base::set_ignore_mobiles(w, m, false);
    }
}

static WRAITH_FORM_FEMALE: TransformationInfo = TransformationInfo {
    body: 747,
    min_skill: |_, _| 20.0,
    hue: 0,
    phys_resist_offset: 15,
    fire_resist_offset: -5,
    nrgy_resist_offset: -5,
    do_effect: wraith_do_effect,
    remove_effect: wraith_remove_effect,
    ..BASE
};

static WRAITH_FORM_MALE: TransformationInfo = TransformationInfo {
    body: 748,
    hue: 0x4001,
    ..WRAITH_FORM_FEMALE
};

/// `WraithFormSpell.DoWraithLeech(wraith, defender, damageGiven)`.
pub fn do_wraith_leech(w: &mut World, wraith: MobileId, defender: MobileId, damage_given: i32) {
    let wraith_leech = 5 + (15.0 * skills::value(w, wraith, SkillName::SpiritSpeak) / 100.0) as i32; // Wraith form gives 5-20% mana leech
    let dmana = w.mobile(defender).map_or(0, |x| x.mana);
    let mana_leech = dmana.min(muo_content::misc::aos_helpers::scale(
        damage_given,
        wraith_leech,
    ));
    if mana_leech != 0 {
        let wmana = w.mobile(wraith).map_or(0, |x| x.mana);
        props_b::set_mana(w, wraith, wmana + mana_leech);
        props_b::set_mana(w, defender, dmana - mana_leech);
        effects::play_sound(w, wraith, 0x44D);
    }
}

/// `WraithFormSpell`: Rel Xen Um.
pub static WRAITH_FORM_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.WraithFormSpell",
    parent: Some(&TRANSFORMATION_SPELL),
    info: SpellInfo::new(
        "Wraith Form",
        "Rel Xen Um",
        &[reagent::NOX_CRYSTAL, reagent::PIG_IRON],
    )
    .action(203)
    .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_secs(2),
    family: Some(&WRAITH_FORM_TYPE),
    ..TRANSFORMATION_SPELL
};

static WRAITH_FORM_TYPE: NecroType = NecroType {
    required_skill: 20.0,
    required_mana: 17,
    transformation: Some(|w, m| {
        if w.mobile(m).is_some_and(|x| x.female) {
            &WRAITH_FORM_FEMALE
        } else {
            &WRAITH_FORM_MALE
        }
    }),
};

// ---- VampiricEmbrace ----

fn vampiric_do_effect(w: &mut World, m: MobileId) {
    let (loc, map) = w.mobile(m).map(|x| (x.location, x.map)).unwrap_or_default();
    let e = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
    muo_server::effects::send_location_particles_full(w, e, 0x373A, 1, 17, 1108, 7, 9914, 0);
    let e = effect_item::create_targeted(w, loc, map, effect_item::DEFAULT_DURATION);
    muo_server::effects::send_location_particles_full(w, e, 0x376A, 1, 22, 67, 7, 9502, 0);
    muo_server::effects::play_sound(w, loc, map, 0x4B1);
}

static VAMPIRIC_EMBRACE_FEMALE: TransformationInfo = TransformationInfo {
    body: 745,
    // `vampiric_cast_skills`: 80 for a caster with 99 Necromancy, else the 99 of the family
    min_skill: |w, m| {
        if skills::value(w, m, SkillName::Necromancy) >= 99.0 {
            80.0
        } else {
            99.0
        }
    },
    hue: 0x847E,
    fire_resist_offset: -25,
    do_effect: vampiric_do_effect,
    ..BASE
};

static VAMPIRIC_EMBRACE_MALE: TransformationInfo = TransformationInfo {
    body: 744,
    ..VAMPIRIC_EMBRACE_FEMALE
};

/// `VampiricEmbraceSpell`: Rel Xen An Sanct.
pub static VAMPIRIC_EMBRACE_SPELL: SpellType = SpellType {
    full_name: "Server.Spells.Necromancy.VampiricEmbraceSpell",
    parent: Some(&TRANSFORMATION_SPELL),
    info: SpellInfo::new(
        "Vampiric Embrace",
        "Rel Xen An Sanct",
        &[reagent::BAT_WING, reagent::NOX_CRYSTAL, reagent::PIG_IRON],
    )
    .action(203)
    .hand_effect(9031),
    cast_delay_base: |_, _| TimeSpan::from_secs(2),
    get_cast_skills: vampiric_cast_skills,
    family: Some(&VAMPIRIC_EMBRACE_TYPE),
    ..TRANSFORMATION_SPELL
};

static VAMPIRIC_EMBRACE_TYPE: NecroType = NecroType {
    required_skill: 99.0,
    required_mana: 23,
    transformation: Some(|w, m| {
        if w.mobile(m).is_some_and(|x| x.female) {
            &VAMPIRIC_EMBRACE_FEMALE
        } else {
            &VAMPIRIC_EMBRACE_MALE
        }
    }),
};

fn vampiric_cast_skills(w: &mut World, sp: SpellId) -> (f64, f64) {
    let Some(caster) = spell::caster(w, sp) else {
        return (0.0, 0.0);
    };
    if skills::value(w, caster, SkillName::Necromancy) >= 99.0 {
        (80.0, 120.0)
    } else {
        (TRANSFORMATION_SPELL.get_cast_skills)(w, sp)
    }
}

/// Sets `TransformationHooks::wraith_leech`.
pub fn init(w: &mut World) {
    w.resources.get_mut::<TransformationHooks>().wraith_leech = Some(do_wraith_leech);
}
