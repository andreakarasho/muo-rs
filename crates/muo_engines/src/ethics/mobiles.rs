//! The ethics mobiles: `HolyFamiliar`, `UnholyFamiliar`, and the `HolySteed` and `UnholySteed` `BaseMount`s. `Ethic.Hero` / `Ethic.Evil` read a fresh
//! `[null, null]` array, so they are always null: the name suffix is dropped and a steed may never be ridden.

use muo_content::abilities::{self, MonsterAbility};
use muo_content::creature::enums::{
    AIType, DEFAULT_RANGE_PERCEPTION, FightMode, FoodType, PackInstinct,
};
use muo_content::creature::mount::{BASE_MOUNT, BASE_MOUNT_C, new_mount};
use muo_content::creature::setters::{
    new_creature, set_damage, set_damage_type, set_dex_range, set_hits_range, set_int_range,
    set_mana, set_resistance_range, set_skill_range, set_str_range,
};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_primitives::Body;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::{MobileType, ResistanceType};
use muo_server::skills::SkillName;
use muo_server::type_registry::TypeRegistry;

fn familiar_init(w: &mut World, id: MobileId, body: i32) {
    props_b::set_body(w, id, Body(body));
    if let Some(m) = w.mobile_mut(id) {
        m.base_sound_id = 0xE5;
    }
    set_str_range(w, id, 96, 120);
    set_dex_range(w, id, 81, 105);
    set_int_range(w, id, 36, 60);
    set_hits_range(w, id, 58, 72);
    set_mana(w, id, 0);
    set_damage(w, id, 11, 17);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 20);
    set_resistance_range(w, id, ResistanceType::Cold, 5, 10);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 57.6, 75.0);
    set_skill_range(w, id, SkillName::Tactics, 50.1, 70.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 80.0);
    props_a::set_fame(w, id, 2500);
    props_a::set_karma(w, id, 2500);
    props_a::set_virtual_armor(w, id, 22);
    if let Some(d) = muo_content::creature::cd_mut(w, id) {
        d.tamable_raw = false;
        d.control_slots = 1;
    }
}

pub fn new_holy_familiar(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &HOLY_FAMILIAR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    familiar_init(w, id, 100);
    id
}

pub fn new_unholy_familiar(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &UNHOLY_FAMILIAR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    familiar_init(w, id, 99);
    id
}

/// `ApplyNameSuffix(suffix)`: `Ethic.Hero` / `Ethic.Evil` are null, so the suffix is dropped.
fn apply_name_suffix(w: &World, id: MobileId, _suffix: &str) -> String {
    muo_content::creature::overrides::apply_name_suffix(w, id, "")
}

macro_rules! familiar {
    ($name:ident, $class:ident, $full:literal, $ctor:ident, $corpse:literal, $default:literal) => {
        pub static $class: CreatureType = CreatureType {
            corpse_name: |_, _| Some($corpse),
            is_dispellable: |_, _| false,
            is_bondable: |_, _| false,
            default_name: |_, _| Some($default),
            meat: |_, _| 1,
            hides: |_, _| 7,
            favorite_food: |_, _| FoodType::MEAT,
            pack_instinct: |_, _| PackInstinct::CANINE,
            ..BASE_CREATURE_C
        };

        #[doc = concat!("`", $full, "`.")]
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&BASE_CREATURE),
            construct: Some($ctor),
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
            apply_name_suffix,
            ..BASE_CREATURE
        };
    };
}

familiar!(
    HOLY_FAMILIAR,
    HOLY_FAMILIAR_C,
    "Server.Mobiles.HolyFamiliar",
    new_holy_familiar,
    "a holy corpse",
    "a silver wolf"
);
familiar!(
    UNHOLY_FAMILIAR,
    UNHOLY_FAMILIAR_C,
    "Server.Mobiles.UnholyFamiliar",
    new_unholy_familiar,
    "an evil corpse",
    "a dark wolf"
);

static STEED_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];

fn steed_init(w: &mut World, id: MobileId, karma: i32) {
    set_str_range(w, id, 496, 525);
    set_dex_range(w, id, 86, 105);
    set_int_range(w, id, 86, 125);
    set_hits_range(w, id, 298, 315);
    set_damage(w, id, 16, 22);
    set_damage_type(w, id, ResistanceType::Physical, 40);
    set_damage_type(w, id, ResistanceType::Fire, 40);
    set_damage_type(w, id, ResistanceType::Energy, 20);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 65);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 30.0);
    set_skill_range(w, id, SkillName::Tactics, 97.6, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 80.5, 92.5);
    props_a::set_fame(w, id, 14000);
    props_a::set_karma(w, id, karma);
    props_a::set_virtual_armor(w, id, 60);
    if let Some(d) = muo_content::creature::cd_mut(w, id) {
        d.tamable_raw = false;
        d.control_slots = 1;
    }
}

pub fn new_holy_steed(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &HOLY_STEED,
        0x75,
        0x3EA8,
        AIType::AiMelee,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    steed_init(w, id, 14000);
    id
}

pub fn new_unholy_steed(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &UNHOLY_STEED,
        0x74,
        0x3EA7,
        AIType::AiMelee,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    steed_init(w, id, -14000);
    id
}

macro_rules! steed {
    ($name:ident, $class:ident, $full:literal, $ctor:ident, $corpse:literal, $default:literal) => {
        pub static $class: CreatureType = CreatureType {
            default_name: |_, _| Some($default),
            corpse_name: |_, _| Some($corpse),
            is_dispellable: |_, _| false,
            is_bondable: |_, _| false,
            steps_max: |_, _| 6400,
            favorite_food: |_, _| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
            get_monster_abilities: |_, _| &STEED_ABILITIES,
            ..BASE_MOUNT_C
        };

        #[doc = concat!("`", $full, "`.")]
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&BASE_MOUNT),
            construct: Some($ctor),
            family: Some(&$class),
            serialize: |w, id, out| {
                (BASE_MOUNT.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_MOUNT.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            apply_name_suffix,
            on_double_click: |w, _, from| {
                // `Ethic.Hero` / `Ethic.Evil` is always null: nobody may ride a steed.
                muo_server::mobile::part2::messages::send_message(
                    w,
                    from,
                    "You may not ride this steed.",
                );
            },
            ..BASE_MOUNT
        };
    };
}

steed!(
    HOLY_STEED,
    HOLY_STEED_C,
    "Server.Mobiles.HolySteed",
    new_holy_steed,
    "a holy corpse",
    "a silver steed"
);
steed!(
    UNHOLY_STEED,
    UNHOLY_STEED_C,
    "Server.Mobiles.UnholySteed",
    new_unholy_steed,
    "an unholy corpse",
    "a dark steed"
);

pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&HOLY_FAMILIAR, &[]);
    types.register_mobile(&UNHOLY_FAMILIAR, &[]);
    types.register_mobile(&HOLY_STEED, &[]);
    types.register_mobile(&UNHOLY_STEED, &[]);
}
