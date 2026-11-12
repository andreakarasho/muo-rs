//! `ArcaneFey`, `ArcaneFiend`, `NatureFury` and `AnimatedWeapon`: the creatures the Spellweaving and Mysticism summons make.

use muo_content::ai::opposition_group::FEY_AND_UNDEAD;
use muo_content::creature::enums::{AIType, FightMode, PackInstinct};
use muo_content::creature::setters::*;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_content::spells::hooks::hooks_mut;
use muo_primitives::{Body, Point3D, TimeSpan};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::effects;
use muo_server::mobile::{MobileType, ResistanceType};
use muo_server::poison::PoisonId;
use muo_server::skills::{self, SkillName};
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

const DEFAULT_RANGE_PERCEPTION: i32 = 16;

/// `Poison.Lethal`.
const LETHAL: PoisonId = PoisonId(4);

/// `[SerializationGenerator(0, false)]` over `BaseCreature`: the parent part and a version int of zero.
macro_rules! creature_class {
    ($name:ident, $class:ident = $full:literal, $ctor:expr; c { $($cf:ident : $cv:expr,)* } m { $($mf:ident : $mv:expr,)* }) => {
        static $class: CreatureType = CreatureType { $($cf: $cv,)* ..BASE_CREATURE_C };
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&BASE_CREATURE),
            construct: $ctor,
            family: Some(&$class),
            serialize: |w, id, out| {
                (BASE_CREATURE.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_CREATURE.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            $($mf: $mv,)*
            ..BASE_CREATURE
        };
    };
}

fn set_body(w: &mut World, id: MobileId, body: i32) {
    props_b::set_body(w, id, Body(body));
}

fn set_base_sound(w: &mut World, id: MobileId, sound: i32) {
    if let Some(m) = w.mobile_mut(id) {
        m.base_sound_id = sound;
    }
}

fn set_control_slots(w: &mut World, id: MobileId, v: i32) {
    if let Some(d) = muo_content::creature::data::cd_mut(w, id) {
        d.control_slots = v;
    }
}

// ---- ArcaneFey ----

fn new_arcane_fey(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ARCANE_FEY,
        AIType::AiMage,
        FightMode::Evil,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    let n = muo_content::misc::small::random_name(w, "pixie");
    if let Some(f) = w.mobile(id).map(|m| m.ty.set_name) {
        f(w, id, Some(n));
    }
    set_body(w, id, 128);
    set_base_sound(w, id, 0x467);
    set_str(w, id, 20);
    set_dex(w, id, 150);
    set_int(w, id, 125);
    set_damage(w, id, 9, 15);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 80, 90);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 40, 50);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 50);
    set_skill_range(w, id, SkillName::EvalInt, 70.1, 80.0);
    set_skill_range(w, id, SkillName::Magery, 70.1, 80.0);
    set_skill_range(w, id, SkillName::Meditation, 70.1, 80.0);
    set_skill_range(w, id, SkillName::MagicResist, 50.5, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 10.1, 20.0);
    set_skill_range(w, id, SkillName::Wrestling, 10.1, 12.5);
    props_a::set_fame(w, id, 0);
    props_a::set_karma(w, id, 0);
    set_control_slots(w, id, 1);
    id
}

creature_class! {
    ARCANE_FEY, ARCANE_FEY_C = "Server.Mobiles.ArcaneFey", Some(new_arcane_fey);
    c {
        corpse_name: |_w, _id| Some("a pixie corpse"),
        dispel_difficulty: |_w, _id| 70.0,
        dispel_focus: |_w, _id| 20.0,
        opposition_group: |_w, _id| Some(&FEY_AND_UNDEAD),
        initial_innocent: |_w, _id| true,
    }
    m {}
}

// ---- ArcaneFiend ----

fn new_arcane_fiend(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ARCANE_FIEND,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    set_body(w, id, 74);
    set_base_sound(w, id, 422);
    set_str(w, id, 55);
    set_dex(w, id, 40);
    set_int(w, id, 60);
    set_damage(w, id, 10, 14);
    set_damage_type(w, id, ResistanceType::Physical, 0);
    set_damage_type(w, id, ResistanceType::Fire, 50);
    set_damage_type(w, id, ResistanceType::Poison, 50);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 30);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::EvalInt, 20.1, 30.0);
    set_skill_range(w, id, SkillName::Magery, 60.1, 70.0);
    set_skill_range(w, id, SkillName::MagicResist, 30.1, 50.0);
    set_skill_range(w, id, SkillName::Tactics, 42.1, 50.0);
    set_skill_range(w, id, SkillName::Wrestling, 40.1, 44.0);
    props_a::set_fame(w, id, 0);
    props_a::set_karma(w, id, 0);
    set_control_slots(w, id, 1);
    id
}

creature_class! {
    ARCANE_FIEND, ARCANE_FIEND_C = "Server.Mobiles.ArcaneFiend", Some(new_arcane_fiend);
    c {
        corpse_name: |_w, _id| Some("an imp corpse"),
        dispel_difficulty: |_w, _id| 70.0,
        dispel_focus: |_w, _id| 20.0,
        pack_instinct: |_w, _id| PackInstinct::DAEMON,
        bleed_immune: |_w, _id| true, // TODO: Verify on OSI.  Guide says this.
        default_name: |_w, _id| Some("an imp"),
    }
    m {}
}

// ---- NatureFury ----

fn new_nature_fury(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &NATURE_FURY,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    set_body(w, id, 0x33);
    muo_server::mobile::base::set_hue(w, id, 0x4001);
    set_str(w, id, 150);
    set_dex(w, id, 150);
    set_int(w, id, 100);
    set_hits(w, id, 80);
    set_stam(w, id, 250);
    set_mana(w, id, 0);
    set_damage(w, id, 6, 8);
    set_damage_type(w, id, ResistanceType::Poison, 100);
    set_damage_type(w, id, ResistanceType::Physical, 0);
    set_resistance(w, id, ResistanceType::Physical, 90);
    set_skill(w, id, SkillName::Wrestling, 90.0);
    set_skill(w, id, SkillName::MagicResist, 70.0);
    set_skill(w, id, SkillName::Tactics, 100.0);
    props_a::set_fame(w, id, 0);
    props_a::set_karma(w, id, 0);
    set_control_slots(w, id, 1);
    id
}

/// `NatureFury.MoveToWorld(loc, map)`: `base.MoveToWorld`, then `Timer.StartTimer(DoEffects)`.
fn nature_fury_move_to_world(w: &mut World, id: MobileId, loc: Point3D, map: MapId) {
    (BASE_CREATURE.move_to_world)(w, id, loc, map);
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        do_effects(w, id)
    });
}

/// `DoEffects()`.
pub fn do_effects(w: &mut World, id: MobileId) {
    effects::fixed_particles_hue(w, id, 0x91C, 10, 180, 0x2543, 0, 0, EffectLayer::Waist);
    effects::play_sound(w, id, 0xE);
    effects::play_sound(w, id, 0x1BC);

    if crate::common::alive(w, id) && !w.mobile(id).is_none_or(|m| m.deleted) {
        timer::start(w, TimeSpan::from_secs(7), TimeSpan::ZERO, 1, move |w| {
            do_effects(w, id)
        });
    }
}

creature_class! {
    NATURE_FURY, NATURE_FURY_C = "Server.Mobiles.NatureFury", Some(new_nature_fury);
    c {
        delete_corpse_on_death: |w, _id| muo_content::core::aos(w),
        is_house_summonable: |_w, _id| true,
        dispel_difficulty: |_w, _id| 125.0,
        dispel_focus: |_w, _id| 90.0,
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(LETHAL),
        always_murderer: |_w, _id| true,
        default_name: |_w, _id| Some("a nature's fury"),
    }
    m {
        skip_serialization: |_w, _id| true,
        move_to_world: nature_fury_move_to_world,
    }
}

// ---- AnimatedWeapon ----

/// The constructor `AnimatedWeapon(Mobile caster, int level)` after `new_animated_weapon`: the body, stats and skills
/// of the level, the caster's Anatomy and Tactics halved.
pub fn init_animated_weapon(w: &mut World, id: MobileId, caster: MobileId, level: i32) {
    set_body(w, id, 692);
    set_str(w, id, 10 + level);
    set_dex(w, id, 10 + level);
    set_int(w, id, 10);
    set_hits(w, id, 20 + level * 3 / 2);
    set_stam(w, id, 10 + level);
    set_mana(w, id, 0);

    if level >= 120 {
        set_damage(w, id, 14, 18);
    } else if level >= 105 {
        set_damage(w, id, 13, 17);
    } else if level >= 90 {
        set_damage(w, id, 12, 15);
    } else if level >= 75 {
        set_damage(w, id, 11, 14);
    } else if level >= 60 {
        set_damage(w, id, 10, 12);
    } else if level >= 45 {
        set_damage(w, id, 9, 11);
    } else if level >= 30 {
        set_damage(w, id, 8, 9);
    } else {
        set_damage(w, id, 7, 8);
    }

    set_damage_type(w, id, ResistanceType::Physical, 60);
    set_damage_type(w, id, ResistanceType::Poison, 20);
    set_damage_type(w, id, ResistanceType::Energy, 20);

    set_resistance_range(w, id, ResistanceType::Physical, 40, 50);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance(w, id, ResistanceType::Poison, 100);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);

    set_skill(w, id, SkillName::MagicResist, f64::from(level));
    set_skill(w, id, SkillName::Wrestling, f64::from(level));
    let anatomy = skills::value(w, caster, SkillName::Anatomy) / 2.0;
    let tactics = skills::value(w, caster, SkillName::Tactics) / 2.0;
    set_skill(w, id, SkillName::Anatomy, anatomy);
    set_skill(w, id, SkillName::Tactics, tactics);

    props_a::set_fame(w, id, 0);
    props_a::set_karma(w, id, 0);
    set_control_slots(w, id, 4);
}

/// `new AnimatedWeapon(caster, level)`.
pub fn animated_weapon_new(w: &mut World, caster: Option<MobileId>, level: i32) -> MobileId {
    let id = new_animated_weapon(w);
    if let Some(caster) = caster {
        init_animated_weapon(w, id, caster, level);
    }
    id
}

/// `[Constructible]` without arguments only builds the base (`AIType.AI_Melee`); the stats come from
/// [`init_animated_weapon`], which `SpellHooks::init_animated_weapon` calls.
fn new_animated_weapon(w: &mut World) -> MobileId {
    new_creature(
        w,
        &ANIMATED_WEAPON,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    )
}

/// `GetFightModeRanking`: `m.Str / Math.Max(this.GetDistanceToSqrt(m), 1.0)`.
fn animated_weapon_ranking(
    w: &mut World,
    id: MobileId,
    m: MobileId,
    _mode: FightMode,
    _player_only: bool,
) -> f64 {
    let s = w.mobile(m).map_or(0, |x| x.str);
    f64::from(s) / crate::common::distance_sqrt(w, id, m).max(1.0)
}

creature_class! {
    ANIMATED_WEAPON, ANIMATED_WEAPON_C = "Server.Mobiles.AnimatedWeapon", None;
    c {
        corpse_name: |_w, _id| Some("an animated weapon corpse"),
        delete_corpse_on_death: |_w, _id| true,
        is_house_summonable: |_w, _id| true,
        dispel_difficulty: |_w, _id| 0.0,
        dispel_focus: |_w, _id| 20.0,
        default_name: |_w, _id| Some("an animated weapon"),
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(LETHAL),
        get_fight_mode_ranking: animated_weapon_ranking,
    }
    m {
        make: Some(new_animated_weapon),
        get_anger_sound: |_w, _id| 0x23A,
        get_attack_sound: |_w, _id| 0x3B8,
        get_hurt_sound: |_w, _id| 0x23A,
    }
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    for t in [&ARCANE_FEY, &ARCANE_FIEND, &NATURE_FURY, &ANIMATED_WEAPON] {
        types.register_mobile(t, &[]);
    }
}

/// Sets the hooks of the classes.
pub fn init(w: &mut World) {
    hooks_mut(w).init_animated_weapon = Some(init_animated_weapon);
}
