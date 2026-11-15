//! The ethics mobiles: `HolyFamiliar`, `UnholyFamiliar`, and the `HolySteed` and `UnholySteed` `BaseMount`s. `Ethic.Hero` / `Ethic.Evil` read a fresh
//! `[null, null]` array, so they are always null: the name suffix is dropped and a steed may never be ridden.

use muo_content::abilities::{self, MonsterAbility};
use muo_content::creature::enums::{
    AIType, DEFAULT_RANGE_PERCEPTION, FightMode, FoodType, PackInstinct,
};
use muo_content::creature::mount::{BASE_MOUNT, BASE_MOUNT_C, new_mount};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;
use muo_server::type_registry::TypeRegistry;

pub fn new_holy_familiar(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &HOLY_FAMILIAR)
}

pub fn new_unholy_familiar(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &UNHOLY_FAMILIAR)
}

/// `ApplyNameSuffix(suffix)`: `Ethic.Hero` / `Ethic.Evil` are null, so the suffix is dropped.
fn apply_name_suffix(w: &World, id: MobileId, _suffix: &str) -> String {
    muo_content::creature::overrides::apply_name_suffix(w, id, "")
}

macro_rules! familiar {
    ($name:ident, $class:ident, $full:literal, $ctor:ident) => {
        pub static $class: CreatureType = CreatureType {
            corpse_name: muo_content::templates::creature_props::corpse_name,
            is_dispellable: |_, _| false,
            is_bondable: |_, _| false,
            default_name: muo_content::templates::creature_props::default_name,
            meat: muo_content::templates::creature_props::meat,
            hides: muo_content::templates::creature_props::hides,
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
    new_holy_familiar
);
familiar!(
    UNHOLY_FAMILIAR,
    UNHOLY_FAMILIAR_C,
    "Server.Mobiles.UnholyFamiliar",
    new_unholy_familiar
);

static STEED_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];

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
    muo_content::templates::apply_mobile(w, id);
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
    muo_content::templates::apply_mobile(w, id);
    id
}

macro_rules! steed {
    ($name:ident, $class:ident, $full:literal, $ctor:ident) => {
        pub static $class: CreatureType = CreatureType {
            default_name: muo_content::templates::creature_props::default_name,
            corpse_name: muo_content::templates::creature_props::corpse_name,
            is_dispellable: |_, _| false,
            is_bondable: |_, _| false,
            steps_max: muo_content::templates::creature_props::steps_max,
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
    new_holy_steed
);
steed!(
    UNHOLY_STEED,
    UNHOLY_STEED_C,
    "Server.Mobiles.UnholySteed",
    new_unholy_steed
);

pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&HOLY_FAMILIAR, &[]);
    types.register_mobile(&UNHOLY_FAMILIAR, &[]);
    types.register_mobile(&HOLY_STEED, &[]);
    types.register_mobile(&UNHOLY_STEED, &[]);
}
