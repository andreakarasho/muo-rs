//! The concrete animal classes that are plain tables. Classes with logic of their own are hand-written in this
//! folder.

use crate::common::*;
use muo_content::abilities::{self, MonsterAbility};
use muo_content::creature::mount::{BASE_MOUNT, BASE_MOUNT_C, new_mount};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::items::weapon::Ability;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;
use muo_server::poison::PoisonId;
use muo_server::type_registry::TypeRegistry;

pub(crate) fn skeletal_mount_init(w: &mut World, id: MobileId) {
    set_str_range(w, id, 91, 100);
    set_dex_range(w, id, 46, 55);
    set_int_range(w, id, 46, 60);
    set_hits_range(w, id, 41, 50);
    set_damage(w, id, 5, 12);
    set_damage_type(w, id, ResistanceType::Physical, 50);
    set_damage_type(w, id, ResistanceType::Cold, 50);
    set_resistance_range(w, id, ResistanceType::Physical, 50, 60);
    set_resistance_range(w, id, ResistanceType::Cold, 90, 95);
    set_resistance(w, id, ResistanceType::Poison, 100);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 95.1, 100.0);
    set_skill(w, id, SkillName::Tactics, 50.0);
    set_skill_range(w, id, SkillName::Wrestling, 70.1, 80.0);
    fame(w, id, 0);
    karma(w, id, 0);
}
fn skeletal_mount_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &SKELETAL_MOUNT,
        793,
        0x3EBB,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    skeletal_mount_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SkeletalMount`.
    SKELETAL_MOUNT, SKELETAL_MOUNT_C = "Server.Mobiles.SkeletalMount" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(skeletal_mount_new), encoded false;
    c {
        steps_max: |_w, _id| 4480,
        corpse_name: |_w, _id| Some("an undead horse corpse"),
        default_name: |_w, _id| Some("a skeletal steed"),
        poison_immune: |_w, _id| Some(PoisonId(4)),
        bleed_immune: |_w, _id| true,
    }
    m {
    }
}

pub(crate) fn silver_serpent_init(w: &mut World, id: MobileId) {
    body(w, id, 92);
    base_sound_id(w, id, 219);
    set_str_range(w, id, 161, 360);
    set_dex_range(w, id, 151, 300);
    set_int_range(w, id, 21, 40);
    set_hits_range(w, id, 97, 216);
    set_damage(w, id, 5, 21);
    set_damage_type(w, id, ResistanceType::Physical, 50);
    set_damage_type(w, id, ResistanceType::Poison, 50);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 5, 10);
    set_resistance(w, id, ResistanceType::Poison, 100);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill_range(w, id, SkillName::Poisoning, 90.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 95.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 80.1, 95.0);
    set_skill_range(w, id, SkillName::Wrestling, 85.1, 100.0);
    fame(w, id, 7000);
    karma(w, id, -7000);
    virtual_armor(w, id, 40);
}
fn silver_serpent_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SILVER_SERPENT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    silver_serpent_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SilverSerpent`.
    SILVER_SERPENT, SILVER_SERPENT_C = "Server.Mobiles.SilverSerpent" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(silver_serpent_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a silver serpent corpse"),
        faction_allegiance: crate::hand::silver_serpent_faction_allegiance,
        ethic_allegiance: crate::hand::silver_serpent_ethic_allegiance,
        default_name: |_w, _id| Some("a silver serpent"),
        death_adder_charmable: |_w, _id| true,
        meat: |_w, _id| 1,
        poison_immune: |_w, _id| Some(PoisonId(4)),
        hit_poison: |_w, _id| Some(PoisonId(4)),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
            add_loot_n(w, id, &loot_pack::GEMS, 2);
        },
    }
    m {
    }
}

pub(crate) fn alligator_init(w: &mut World, id: MobileId) {
    body(w, id, 0xCA);
    base_sound_id(w, id, 660);
    set_str_range(w, id, 76, 100);
    set_dex_range(w, id, 6, 25);
    set_int_range(w, id, 11, 20);
    set_hits_range(w, id, 46, 60);
    set_stam_range(w, id, 46, 65);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 15);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 40.0);
    set_skill_range(w, id, SkillName::Tactics, 40.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 40.1, 60.0);
    fame(w, id, 600);
    karma(w, id, -600);
    virtual_armor(w, id, 30);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 47.1);
}
fn alligator_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ALLIGATOR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    alligator_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Alligator`.
    ALLIGATOR, ALLIGATOR_C = "Server.Mobiles.Alligator" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(alligator_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an alligator corpse"),
        default_name: |_w, _id| Some("an alligator"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 12,
        hide_type: |_w, _id| HideType::Spined,
        favorite_food: |_w, _id| FoodType::MEAT | FoodType::FISH,
    }
    m {
    }
}

pub(crate) fn phoenix_init(w: &mut World, id: MobileId) {
    body(w, id, 5);
    hue(w, id, 0x674);
    base_sound_id(w, id, 0x8F);
    set_str_range(w, id, 504, 700);
    set_dex_range(w, id, 202, 300);
    set_int_range(w, id, 504, 700);
    set_hits_range(w, id, 340, 383);
    set_damage(w, id, 25, 25);
    set_damage_type(w, id, ResistanceType::Physical, 50);
    set_damage_type(w, id, ResistanceType::Fire, 50);
    set_resistance_range(w, id, ResistanceType::Physical, 45, 55);
    set_resistance_range(w, id, ResistanceType::Fire, 60, 70);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 50);
    set_skill_range(w, id, SkillName::EvalInt, 90.2, 100.0);
    set_skill_range(w, id, SkillName::Magery, 90.2, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 75.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 86.0, 135.0);
    set_skill_range(w, id, SkillName::Tactics, 80.1, 90.0);
    set_skill_range(w, id, SkillName::Wrestling, 90.1, 100.0);
    fame(w, id, 15000);
    karma(w, id, 0);
    virtual_armor(w, id, 60);
}
fn phoenix_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &PHOENIX,
        AIType::AiMage,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    phoenix_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Phoenix`.
    PHOENIX, PHOENIX_C = "Server.Mobiles.Phoenix" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(phoenix_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a phoenix corpse"),
        default_name: |_w, _id| Some("a phoenix"),
        meat: |_w, _id| 1,
        meat_type: |_w, _id| MeatType::Bird,
        feathers: |_w, _id| 36,
        can_fly: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot(w, id, loot_pack::rich(w));
        },
    }
    m {
    }
}

pub(crate) fn black_bear_init(w: &mut World, id: MobileId) {
    body(w, id, 211);
    base_sound_id(w, id, 0xA3);
    set_str_range(w, id, 76, 100);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 11, 14);
    set_hits_range(w, id, 46, 60);
    set_mana(w, id, 0);
    set_damage(w, id, 4, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 15);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 20.1, 40.0);
    set_skill_range(w, id, SkillName::Tactics, 40.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 40.1, 60.0);
    fame(w, id, 450);
    karma(w, id, 0);
    virtual_armor(w, id, 24);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 35.1);
}
fn black_bear_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BLACK_BEAR,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    black_bear_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.BlackBear`.
    BLACK_BEAR, BLACK_BEAR_C = "Server.Mobiles.BlackBear" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(black_bear_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a bear corpse"),
        default_name: |_w, _id| Some("a black bear"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 12,
        favorite_food: |_w, _id| FoodType::FISH | FoodType::MEAT | FoodType::FRUITS_AND_VEGGIES,
        pack_instinct: |_w, _id| PackInstinct::BEAR,
    }
    m {
    }
}

pub(crate) fn dire_wolf_init(w: &mut World, id: MobileId) {
    body(w, id, 23);
    base_sound_id(w, id, 0xE5);
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
    fame(w, id, 2500);
    karma(w, id, -2500);
    virtual_armor(w, id, 22);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 83.1);
}
fn dire_wolf_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &DIRE_WOLF,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    dire_wolf_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.DireWolf`.
    DIRE_WOLF, DIRE_WOLF_C = "Server.Mobiles.DireWolf" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dire_wolf_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a dire wolf corpse"),
        default_name: |_w, _id| Some("a dire wolf"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 7,
        hide_type: |_w, _id| HideType::Spined,
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::CANINE,
    }
    m {
    }
}

pub(crate) fn eagle_init(w: &mut World, id: MobileId) {
    body(w, id, 5);
    base_sound_id(w, id, 0x2EE);
    set_str_range(w, id, 31, 47);
    set_dex_range(w, id, 36, 60);
    set_int_range(w, id, 8, 20);
    set_hits_range(w, id, 20, 27);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 15);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 15.3, 30.0);
    set_skill_range(w, id, SkillName::Tactics, 18.1, 37.0);
    set_skill_range(w, id, SkillName::Wrestling, 20.1, 30.0);
    fame(w, id, 300);
    karma(w, id, 0);
    virtual_armor(w, id, 22);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 17.1);
}
fn eagle_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &EAGLE,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    eagle_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Eagle`.
    EAGLE, EAGLE_C = "Server.Mobiles.Eagle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(eagle_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a bird corpse"),
        default_name: |_w, _id| Some("an eagle"),
        meat: |_w, _id| 1,
        meat_type: |_w, _id| MeatType::Bird,
        feathers: |_w, _id| 36,
        favorite_food: |_w, _id| FoodType::MEAT | FoodType::FISH,
        can_fly: |_w, _id| true,
    }
    m {
    }
}

pub(crate) fn brown_bear_init(w: &mut World, id: MobileId) {
    body(w, id, 167);
    base_sound_id(w, id, 0xA3);
    set_str_range(w, id, 76, 100);
    set_dex_range(w, id, 26, 45);
    set_int_range(w, id, 23, 47);
    set_hits_range(w, id, 46, 60);
    set_mana(w, id, 0);
    set_damage(w, id, 6, 12);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 15, 20);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 35.0);
    set_skill_range(w, id, SkillName::Tactics, 40.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 40.1, 60.0);
    fame(w, id, 450);
    karma(w, id, 0);
    virtual_armor(w, id, 24);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 41.1);
}
fn brown_bear_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BROWN_BEAR,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    brown_bear_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.BrownBear`.
    BROWN_BEAR, BROWN_BEAR_C = "Server.Mobiles.BrownBear" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(brown_bear_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a bear corpse"),
        default_name: |_w, _id| Some("a brown bear"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 12,
        favorite_food: |_w, _id| FoodType::FISH | FoodType::FRUITS_AND_VEGGIES | FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::BEAR,
    }
    m {
    }
}

pub(crate) fn grizzly_bear_init(w: &mut World, id: MobileId) {
    body(w, id, 212);
    base_sound_id(w, id, 0xA3);
    set_str_range(w, id, 126, 155);
    set_dex_range(w, id, 81, 105);
    set_int_range(w, id, 16, 40);
    set_hits_range(w, id, 76, 93);
    set_mana(w, id, 0);
    set_damage(w, id, 8, 13);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 15, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 40.0);
    set_skill_range(w, id, SkillName::Tactics, 70.1, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 45.1, 70.0);
    fame(w, id, 1000);
    karma(w, id, 0);
    virtual_armor(w, id, 24);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 59.1);
}
fn grizzly_bear_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GRIZZLY_BEAR,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    grizzly_bear_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.GrizzlyBear`.
    GRIZZLY_BEAR, GRIZZLY_BEAR_C = "Server.Mobiles.GrizzlyBear" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(grizzly_bear_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a grizzly bear corpse"),
        default_name: |_w, _id| Some("a grizzly bear"),
        meat: |_w, _id| 2,
        hides: |_w, _id| 16,
        favorite_food: |_w, _id| FoodType::FISH | FoodType::FRUITS_AND_VEGGIES | FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::BEAR,
    }
    m {
    }
}

pub(crate) fn polar_bear_init(w: &mut World, id: MobileId) {
    body(w, id, 213);
    base_sound_id(w, id, 0xA3);
    set_str_range(w, id, 116, 140);
    set_dex_range(w, id, 81, 105);
    set_int_range(w, id, 26, 50);
    set_hits_range(w, id, 70, 84);
    set_mana(w, id, 0);
    set_damage(w, id, 7, 12);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 60, 80);
    set_resistance_range(w, id, ResistanceType::Poison, 15, 25);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Tactics, 60.1, 90.0);
    set_skill_range(w, id, SkillName::Wrestling, 45.1, 70.0);
    fame(w, id, 1500);
    karma(w, id, 0);
    virtual_armor(w, id, 18);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 35.1);
}
fn polar_bear_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &POLAR_BEAR,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    polar_bear_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.PolarBear`.
    POLAR_BEAR, POLAR_BEAR_C = "Server.Mobiles.PolarBear" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(polar_bear_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a polar bear corpse"),
        default_name: |_w, _id| Some("a polar bear"),
        meat: |_w, _id| 2,
        hides: |_w, _id| 16,
        favorite_food: |_w, _id| FoodType::FISH | FoodType::FRUITS_AND_VEGGIES | FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::BEAR,
    }
    m {
    }
}

pub(crate) fn chicken_init(w: &mut World, id: MobileId) {
    body(w, id, 0xD0);
    base_sound_id(w, id, 0x6E);
    set_str(w, id, 5);
    set_dex(w, id, 15);
    set_int(w, id, 5);
    set_hits(w, id, 3);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 1);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 1, 5);
    set_skill(w, id, SkillName::MagicResist, 4.0);
    set_skill(w, id, SkillName::Tactics, 5.0);
    set_skill(w, id, SkillName::Wrestling, 5.0);
    fame(w, id, 150);
    karma(w, id, 0);
    virtual_armor(w, id, 2);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, -0.9);
}
fn chicken_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CHICKEN,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    chicken_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Chicken`.
    CHICKEN, CHICKEN_C = "Server.Mobiles.Chicken" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(chicken_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a chicken corpse"),
        default_name: |_w, _id| Some("a chicken"),
        meat: |_w, _id| 1,
        meat_type: |_w, _id| MeatType::Bird,
        favorite_food: |_w, _id| FoodType::GRAINS_AND_HAY,
        can_fly: |_w, _id| true,
        feathers: |_w, _id| 25,
    }
    m {
    }
}

pub(crate) fn crane_init(w: &mut World, id: MobileId) {
    body(w, id, 254);
    base_sound_id(w, id, 0x4D7);
    set_str_range(w, id, 26, 35);
    set_dex_range(w, id, 16, 25);
    set_int_range(w, id, 11, 15);
    set_hits_range(w, id, 26, 35);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 1);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 5, 5);
    set_skill_range(w, id, SkillName::MagicResist, 4.1, 5.0);
    set_skill_range(w, id, SkillName::Tactics, 10.1, 11.0);
    set_skill_range(w, id, SkillName::Wrestling, 10.1, 11.0);
    fame(w, id, 0);
    karma(w, id, 200);
    virtual_armor(w, id, 5);
}
fn crane_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CRANE,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    crane_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Crane`.
    CRANE, CRANE_C = "Server.Mobiles.Crane" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(crane_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a bird corpse"),
        default_name: |_w, _id| Some("a crane"),
        meat: |_w, _id| 1,
        feathers: |_w, _id| 25,
    }
    m {
        get_anger_sound: |_w, _id| 0x4D9,
        get_idle_sound: |_w, _id| 0x4D8,
        get_attack_sound: |_w, _id| 0x4D7,
        get_hurt_sound: |_w, _id| 0x4DA,
        get_death_sound: |_w, _id| 0x4D6,
    }
}

pub(crate) fn grey_wolf_init(w: &mut World, id: MobileId) {
    body_list(w, id, &[25, 27]);
    base_sound_id(w, id, 0xE5);
    set_str_range(w, id, 56, 80);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 31, 55);
    set_hits_range(w, id, 34, 48);
    set_mana(w, id, 0);
    set_damage(w, id, 3, 7);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 15);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 15);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 20.1, 35.0);
    set_skill_range(w, id, SkillName::Tactics, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 45.1, 60.0);
    fame(w, id, 450);
    karma(w, id, 0);
    virtual_armor(w, id, 16);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 53.1);
}
fn grey_wolf_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GREY_WOLF,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    grey_wolf_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.GreyWolf`.
    GREY_WOLF, GREY_WOLF_C = "Server.Mobiles.GreyWolf" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(grey_wolf_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a grey wolf corpse"),
        default_name: |_w, _id| Some("a grey wolf"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 6,
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::CANINE,
    }
    m {
    }
}

pub(crate) fn timber_wolf_init(w: &mut World, id: MobileId) {
    body(w, id, 225);
    base_sound_id(w, id, 0xE5);
    set_str_range(w, id, 56, 80);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 11, 25);
    set_hits_range(w, id, 34, 48);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 9);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 15);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 27.6, 45.0);
    set_skill_range(w, id, SkillName::Tactics, 30.1, 50.0);
    set_skill_range(w, id, SkillName::Wrestling, 40.1, 60.0);
    fame(w, id, 450);
    karma(w, id, 0);
    virtual_armor(w, id, 16);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 23.1);
}
fn timber_wolf_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &TIMBER_WOLF,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    timber_wolf_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.TimberWolf`.
    TIMBER_WOLF, TIMBER_WOLF_C = "Server.Mobiles.TimberWolf" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(timber_wolf_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a timber wolf corpse"),
        default_name: |_w, _id| Some("a timber wolf"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 5,
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::CANINE,
    }
    m {
    }
}

pub(crate) fn white_wolf_init(w: &mut World, id: MobileId) {
    body_list(w, id, &[34, 37]);
    base_sound_id(w, id, 0xE5);
    set_str_range(w, id, 56, 80);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 31, 55);
    set_hits_range(w, id, 34, 48);
    set_mana(w, id, 0);
    set_damage(w, id, 3, 7);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 15);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 15);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 20.1, 35.0);
    set_skill_range(w, id, SkillName::Tactics, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 45.1, 60.0);
    fame(w, id, 450);
    karma(w, id, 0);
    virtual_armor(w, id, 16);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 65.1);
}
fn white_wolf_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &WHITE_WOLF,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    white_wolf_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.WhiteWolf`.
    WHITE_WOLF, WHITE_WOLF_C = "Server.Mobiles.WhiteWolf" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(white_wolf_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a white wolf corpse"),
        default_name: |_w, _id| Some("a white wolf"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 6,
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::CANINE,
    }
    m {
    }
}

pub(crate) fn bull_init(w: &mut World, id: MobileId) {
    body_list(w, id, &[0xE8, 0xE9]);
    base_sound_id(w, id, 0x64);
    if rnd::bool(w) {
        hue(w, id, 0x901);
    }
    set_str_range(w, id, 77, 111);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 47, 75);
    set_hits_range(w, id, 50, 64);
    set_mana(w, id, 0);
    set_damage(w, id, 4, 9);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 17.6, 25.0);
    set_skill_range(w, id, SkillName::Tactics, 67.6, 85.0);
    set_skill_range(w, id, SkillName::Wrestling, 40.1, 57.5);
    fame(w, id, 600);
    karma(w, id, 0);
    virtual_armor(w, id, 28);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 71.1);
}
fn bull_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BULL,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    bull_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Bull`.
    BULL, BULL_C = "Server.Mobiles.Bull" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bull_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a bull corpse"),
        default_name: |_w, _id| Some("a bull"),
        meat: |_w, _id| 10,
        hides: |_w, _id| 15,
        favorite_food: |_w, _id| FoodType::GRAINS_AND_HAY,
        pack_instinct: |_w, _id| PackInstinct::BULL,
    }
    m {
    }
}

pub(crate) fn cow_init(w: &mut World, id: MobileId) {
    crate::hand::cow_init(w, id);
    body_list(w, id, &[0xD8, 0xE7]);
    base_sound_id(w, id, 0x78);
    set_str(w, id, 30);
    set_dex(w, id, 15);
    set_int(w, id, 5);
    set_hits(w, id, 18);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 4);
    set_damage(w, id, 1, 4);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 5, 15);
    set_skill(w, id, SkillName::MagicResist, 5.5);
    set_skill(w, id, SkillName::Tactics, 5.5);
    set_skill(w, id, SkillName::Wrestling, 5.5);
    fame(w, id, 300);
    karma(w, id, 0);
    virtual_armor(w, id, 10);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 11.1);
    crate::hand::cow_ctor1(w, id);
}
fn cow_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &COW,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    cow_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Cow`.
    COW, COW_C = "Server.Mobiles.Cow" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(cow_new), state crate::hand::cow_serialize, crate::hand::cow_deserialize;
    c {
        corpse_name: |_w, _id| Some("a cow corpse"),
        default_name: |_w, _id| Some("a cow"),
        meat: |_w, _id| 8,
        hides: |_w, _id| 12,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
        on_double_click: crate::hand::cow_on_double_click,
    }
}

pub(crate) fn cougar_init(w: &mut World, id: MobileId) {
    body(w, id, 63);
    base_sound_id(w, id, 0x73);
    set_str_range(w, id, 56, 80);
    set_dex_range(w, id, 66, 85);
    set_int_range(w, id, 26, 50);
    set_hits_range(w, id, 34, 48);
    set_mana(w, id, 0);
    set_damage(w, id, 4, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 15);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 30.0);
    set_skill_range(w, id, SkillName::Tactics, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 45.1, 60.0);
    fame(w, id, 450);
    karma(w, id, 0);
    virtual_armor(w, id, 16);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 41.1);
}
fn cougar_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &COUGAR,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    cougar_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Cougar`.
    COUGAR, COUGAR_C = "Server.Mobiles.Cougar" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(cougar_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a cougar corpse"),
        default_name: |_w, _id| Some("a cougar"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 10,
        favorite_food: |_w, _id| FoodType::FISH | FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::FELINE,
    }
    m {
    }
}

pub(crate) fn hell_cat_init(w: &mut World, id: MobileId) {
    body(w, id, 0xC9);
    {
        let list = [0x647, 0x650, 0x659, 0x662, 0x66B, 0x674];
        let i = rnd::below(w, list.len() as i32) as usize;
        hue(w, id, list[i]);
    }
    base_sound_id(w, id, 0x69);
    set_str_range(w, id, 51, 100);
    set_dex_range(w, id, 52, 150);
    set_int_range(w, id, 13, 85);
    set_hits_range(w, id, 48, 67);
    set_damage(w, id, 6, 12);
    set_damage_type(w, id, ResistanceType::Physical, 40);
    set_damage_type(w, id, ResistanceType::Fire, 60);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 80, 90);
    set_resistance_range(w, id, ResistanceType::Energy, 15, 20);
    set_skill_range(w, id, SkillName::MagicResist, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Tactics, 40.1, 55.0);
    set_skill_range(w, id, SkillName::Wrestling, 30.1, 40.0);
    fame(w, id, 1000);
    karma(w, id, -1000);
    virtual_armor(w, id, 30);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 71.1);
}
fn hell_cat_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &HELL_CAT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    hell_cat_init(w, id);
    id
}
static HELL_CAT_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.HellCat`.
    HELL_CAT, HELL_CAT_C = "Server.Mobiles.HellCat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(hell_cat_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a hell cat corpse"),
        default_name: |_w, _id| Some("a hell cat"),
        hides: |_w, _id| 10,
        hide_type: |_w, _id| HideType::Spined,
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::FELINE,
        get_monster_abilities: |_, _| &HELL_CAT_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

pub(crate) fn panther_init(w: &mut World, id: MobileId) {
    body(w, id, 0xD6);
    hue(w, id, 0x901);
    base_sound_id(w, id, 0x462);
    set_str_range(w, id, 61, 85);
    set_dex_range(w, id, 86, 105);
    set_int_range(w, id, 26, 50);
    set_hits_range(w, id, 37, 51);
    set_mana(w, id, 0);
    set_damage(w, id, 4, 12);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 15);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 30.0);
    set_skill_range(w, id, SkillName::Tactics, 50.1, 65.0);
    set_skill_range(w, id, SkillName::Wrestling, 50.1, 65.0);
    fame(w, id, 450);
    karma(w, id, 0);
    virtual_armor(w, id, 16);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 53.1);
}
fn panther_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &PANTHER,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    panther_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Panther`.
    PANTHER, PANTHER_C = "Server.Mobiles.Panther" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(panther_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a panther corpse"),
        default_name: |_w, _id| Some("a panther"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 10,
        favorite_food: |_w, _id| FoodType::MEAT | FoodType::FISH,
        pack_instinct: |_w, _id| PackInstinct::FELINE,
    }
    m {
    }
}

pub(crate) fn predator_hell_cat_init(w: &mut World, id: MobileId) {
    body(w, id, 127);
    base_sound_id(w, id, 0xBA);
    set_str_range(w, id, 161, 185);
    set_dex_range(w, id, 96, 115);
    set_int_range(w, id, 76, 100);
    set_hits_range(w, id, 97, 131);
    set_damage(w, id, 5, 17);
    set_damage_type(w, id, ResistanceType::Physical, 75);
    set_damage_type(w, id, ResistanceType::Fire, 25);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 15);
    set_skill_range(w, id, SkillName::MagicResist, 75.1, 90.0);
    set_skill_range(w, id, SkillName::Tactics, 50.1, 65.0);
    set_skill_range(w, id, SkillName::Wrestling, 50.1, 65.0);
    fame(w, id, 2500);
    karma(w, id, -2500);
    virtual_armor(w, id, 30);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 89.1);
}
fn predator_hell_cat_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &PREDATOR_HELL_CAT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    predator_hell_cat_init(w, id);
    id
}
static PREDATOR_HELL_CAT_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.PredatorHellCat`.
    PREDATOR_HELL_CAT, PREDATOR_HELL_CAT_C = "Server.Mobiles.PredatorHellCat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(predator_hell_cat_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a hell cat corpse"),
        default_name: |_w, _id| Some("a hell cat"),
        hides: |_w, _id| 10,
        hide_type: |_w, _id| HideType::Spined,
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::FELINE,
        get_monster_abilities: |_, _| &PREDATOR_HELL_CAT_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
        },
    }
    m {
    }
}

pub(crate) fn snow_leopard_init(w: &mut World, id: MobileId) {
    body_list(w, id, &[64, 65]);
    base_sound_id(w, id, 0x73);
    set_str_range(w, id, 56, 80);
    set_dex_range(w, id, 66, 85);
    set_int_range(w, id, 26, 50);
    set_hits_range(w, id, 34, 48);
    set_mana(w, id, 0);
    set_damage(w, id, 3, 9);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 20);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 35.0);
    set_skill_range(w, id, SkillName::Tactics, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 40.1, 50.0);
    fame(w, id, 450);
    karma(w, id, 0);
    virtual_armor(w, id, 24);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 53.1);
}
fn snow_leopard_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SNOW_LEOPARD,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    snow_leopard_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SnowLeopard`.
    SNOW_LEOPARD, SNOW_LEOPARD_C = "Server.Mobiles.SnowLeopard" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(snow_leopard_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a leopard corpse"),
        default_name: |_w, _id| Some("a snow leopard"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 8,
        favorite_food: |_w, _id| FoodType::MEAT | FoodType::FISH,
        pack_instinct: |_w, _id| PackInstinct::FELINE,
    }
    m {
    }
}

pub(crate) fn boar_init(w: &mut World, id: MobileId) {
    body(w, id, 0x122);
    base_sound_id(w, id, 0xC4);
    set_str(w, id, 25);
    set_dex(w, id, 15);
    set_int(w, id, 5);
    set_hits(w, id, 15);
    set_mana(w, id, 0);
    set_damage(w, id, 3, 6);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 10, 15);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_skill(w, id, SkillName::MagicResist, 9.0);
    set_skill(w, id, SkillName::Tactics, 9.0);
    set_skill(w, id, SkillName::Wrestling, 9.0);
    fame(w, id, 300);
    karma(w, id, 0);
    virtual_armor(w, id, 10);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 29.1);
}
fn boar_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BOAR,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    boar_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Boar`.
    BOAR, BOAR_C = "Server.Mobiles.Boar" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(boar_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a pig corpse"),
        default_name: |_w, _id| Some("a boar"),
        meat: |_w, _id| 2,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
    }
}

pub(crate) fn bull_frog_init(w: &mut World, id: MobileId) {
    body(w, id, 81);
    {
        let list = [0x5AC, 0x5A3, 0x59A, 0x591, 0x588, 0x57F];
        let i = rnd::below(w, list.len() as i32) as usize;
        hue(w, id, list[i]);
    }
    base_sound_id(w, id, 0x266);
    set_str_range(w, id, 46, 70);
    set_dex_range(w, id, 6, 25);
    set_int_range(w, id, 11, 20);
    set_hits_range(w, id, 28, 42);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 2);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 40.0);
    set_skill_range(w, id, SkillName::Tactics, 40.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 40.1, 60.0);
    fame(w, id, 350);
    karma(w, id, 0);
    virtual_armor(w, id, 6);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 23.1);
}
fn bull_frog_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BULL_FROG,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    bull_frog_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.BullFrog`.
    BULL_FROG, BULL_FROG_C = "Server.Mobiles.BullFrog" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bull_frog_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a bull frog corpse"),
        default_name: |_w, _id| Some("a bull frog"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 4,
        favorite_food: |_w, _id| FoodType::FISH | FoodType::MEAT,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::poor(w));
        },
    }
    m {
    }
}

pub(crate) fn dolphin_init(w: &mut World, id: MobileId) {
    body(w, id, 0x97);
    base_sound_id(w, id, 0x8A);
    set_str_range(w, id, 21, 49);
    set_dex_range(w, id, 66, 85);
    set_int_range(w, id, 96, 110);
    set_hits_range(w, id, 15, 27);
    set_damage(w, id, 3, 6);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 70, 80);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 30);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 15);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 20.0);
    set_skill_range(w, id, SkillName::Tactics, 19.2, 29.0);
    set_skill_range(w, id, SkillName::Wrestling, 19.2, 29.0);
    fame(w, id, 500);
    karma(w, id, 2000);
    virtual_armor(w, id, 16);
    can_swim(w, id, true);
    cant_walk(w, id, true);
}
fn dolphin_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &DOLPHIN,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    dolphin_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Dolphin`.
    DOLPHIN, DOLPHIN_C = "Server.Mobiles.Dolphin" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dolphin_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a dolphin corpse"),
        default_name: |_w, _id| Some("a dolphin"),
        meat: |_w, _id| 1,
        on_think: crate::hand::dolphin_on_think,
    }
    m {
        on_double_click: crate::hand::dolphin_on_double_click,
    }
}

pub(crate) fn gaman_init(w: &mut World, id: MobileId) {
    body(w, id, 248);
    set_str_range(w, id, 146, 175);
    set_dex_range(w, id, 111, 150);
    set_int_range(w, id, 46, 60);
    set_hits_range(w, id, 131, 160);
    set_mana(w, id, 0);
    set_damage(w, id, 6, 11);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 50, 70);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 50);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 60);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 50);
    set_skill_range(w, id, SkillName::MagicResist, 37.6, 42.5);
    set_skill_range(w, id, SkillName::Tactics, 70.6, 83.0);
    set_skill_range(w, id, SkillName::Wrestling, 50.1, 57.5);
    fame(w, id, 2000);
    karma(w, id, -2000);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 68.7);
}
fn gaman_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GAMAN,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    gaman_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Gaman`.
    GAMAN, GAMAN_C = "Server.Mobiles.Gaman" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gaman_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a gaman corpse"),
        default_name: |_w, _id| Some("a gaman"),
        meat: |_w, _id| 10,
        hides: |_w, _id| 15,
        favorite_food: |_w, _id| FoodType::GRAINS_AND_HAY,
    }
    m {
        get_anger_sound: |_w, _id| 0x4F8,
        get_idle_sound: |_w, _id| 0x4F7,
        get_attack_sound: |_w, _id| 0x4F6,
        get_hurt_sound: |_w, _id| 0x4F9,
        get_death_sound: |_w, _id| 0x4F5,
        on_death: crate::hand::gaman_on_death,
    }
}

pub(crate) fn giant_toad_init(w: &mut World, id: MobileId) {
    body(w, id, 80);
    base_sound_id(w, id, 0x26B);
    set_str_range(w, id, 76, 100);
    set_dex_range(w, id, 6, 25);
    set_int_range(w, id, 11, 20);
    set_hits_range(w, id, 46, 60);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 17);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 40.0);
    set_skill_range(w, id, SkillName::Tactics, 40.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 40.1, 60.0);
    fame(w, id, 750);
    karma(w, id, -750);
    virtual_armor(w, id, 24);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 77.1);
}
fn giant_toad_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GIANT_TOAD,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    giant_toad_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.GiantToad`.
    GIANT_TOAD, GIANT_TOAD_C = "Server.Mobiles.GiantToad" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(giant_toad_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a giant toad corpse"),
        default_name: |_w, _id| Some("a giant toad"),
        hides: |_w, _id| 12,
        hide_type: |_w, _id| HideType::Spined,
        favorite_food: |_w, _id| FoodType::FISH | FoodType::MEAT,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::poor(w));
        },
    }
    m {
    }
}

pub(crate) fn goat_init(w: &mut World, id: MobileId) {
    body(w, id, 0xD1);
    base_sound_id(w, id, 0x99);
    set_str(w, id, 19);
    set_dex(w, id, 15);
    set_int(w, id, 5);
    set_hits(w, id, 12);
    set_mana(w, id, 0);
    set_damage(w, id, 3, 4);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 5, 15);
    set_skill(w, id, SkillName::MagicResist, 5.0);
    set_skill(w, id, SkillName::Tactics, 5.0);
    set_skill(w, id, SkillName::Wrestling, 5.0);
    fame(w, id, 150);
    karma(w, id, 0);
    virtual_armor(w, id, 10);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 11.1);
}
fn goat_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GOAT,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    goat_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Goat`.
    GOAT, GOAT_C = "Server.Mobiles.Goat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(goat_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a goat corpse"),
        default_name: |_w, _id| Some("a goat"),
        meat: |_w, _id| 2,
        hides: |_w, _id| 8,
        favorite_food: |_w, _id| FoodType::GRAINS_AND_HAY | FoodType::FRUITS_AND_VEGGIES | FoodType::LEATHER,
    }
    m {
    }
}

pub(crate) fn gorilla_init(w: &mut World, id: MobileId) {
    body(w, id, 0x1D);
    base_sound_id(w, id, 0x9E);
    set_str_range(w, id, 53, 95);
    set_dex_range(w, id, 36, 55);
    set_int_range(w, id, 36, 60);
    set_hits_range(w, id, 38, 51);
    set_mana(w, id, 0);
    set_damage(w, id, 4, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Tactics, 43.3, 58.0);
    set_skill_range(w, id, SkillName::Wrestling, 43.3, 58.0);
    fame(w, id, 450);
    karma(w, id, 0);
    virtual_armor(w, id, 20);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, -18.9);
}
fn gorilla_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GORILLA,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    gorilla_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Gorilla`.
    GORILLA, GORILLA_C = "Server.Mobiles.Gorilla" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gorilla_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a gorilla corpse"),
        default_name: |_w, _id| Some("a gorilla"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 6,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
    }
}

pub(crate) fn great_hart_init(w: &mut World, id: MobileId) {
    body(w, id, 0xEA);
    set_str_range(w, id, 41, 71);
    set_dex_range(w, id, 47, 77);
    set_int_range(w, id, 27, 57);
    set_hits_range(w, id, 27, 41);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 9);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Cold, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 26.8, 44.5);
    set_skill_range(w, id, SkillName::Tactics, 29.8, 47.5);
    set_skill_range(w, id, SkillName::Wrestling, 29.8, 47.5);
    fame(w, id, 300);
    karma(w, id, 0);
    virtual_armor(w, id, 24);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 59.1);
}
fn great_hart_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GREAT_HART,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    great_hart_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.GreatHart`.
    GREAT_HART, GREAT_HART_C = "Server.Mobiles.GreatHart" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(great_hart_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a deer corpse"),
        default_name: |_w, _id| Some("a great hart"),
        meat: |_w, _id| 6,
        hides: |_w, _id| 15,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
        get_attack_sound: |_w, _id| 0x82,
        get_hurt_sound: |_w, _id| 0x83,
        get_death_sound: |_w, _id| 0x84,
    }
}

pub(crate) fn hind_init(w: &mut World, id: MobileId) {
    body(w, id, 0xED);
    set_str_range(w, id, 21, 51);
    set_dex_range(w, id, 47, 77);
    set_int_range(w, id, 17, 47);
    set_hits_range(w, id, 15, 29);
    set_mana(w, id, 0);
    set_damage(w, id, 4, 4);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 5, 15);
    set_resistance(w, id, ResistanceType::Cold, 5);
    set_skill(w, id, SkillName::MagicResist, 15.0);
    set_skill(w, id, SkillName::Tactics, 19.0);
    set_skill(w, id, SkillName::Wrestling, 26.0);
    fame(w, id, 300);
    karma(w, id, 0);
    virtual_armor(w, id, 8);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 23.1);
}
fn hind_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &HIND,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    hind_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Hind`.
    HIND, HIND_C = "Server.Mobiles.Hind" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(hind_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a deer corpse"),
        default_name: |_w, _id| Some("a hind"),
        meat: |_w, _id| 5,
        hides: |_w, _id| 8,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
        get_attack_sound: |_w, _id| 0x82,
        get_hurt_sound: |_w, _id| 0x83,
        get_death_sound: |_w, _id| 0x84,
    }
}

pub(crate) fn llama_init(w: &mut World, id: MobileId) {
    body(w, id, 0xDC);
    base_sound_id(w, id, 0x3F3);
    set_str_range(w, id, 21, 49);
    set_dex_range(w, id, 36, 55);
    set_int_range(w, id, 16, 30);
    set_hits_range(w, id, 15, 27);
    set_mana(w, id, 0);
    set_damage(w, id, 3, 5);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 20.0);
    set_skill_range(w, id, SkillName::Tactics, 19.2, 29.0);
    set_skill_range(w, id, SkillName::Wrestling, 19.2, 29.0);
    fame(w, id, 300);
    karma(w, id, 0);
    virtual_armor(w, id, 16);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 35.1);
}
fn llama_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LLAMA,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    llama_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Llama`.
    LLAMA, LLAMA_C = "Server.Mobiles.Llama" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(llama_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a llama corpse"),
        default_name: |_w, _id| Some("a llama"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 12,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
    }
}

pub(crate) fn mountain_goat_init(w: &mut World, id: MobileId) {
    body(w, id, 88);
    base_sound_id(w, id, 0x99);
    set_str_range(w, id, 22, 64);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 16, 30);
    set_hits_range(w, id, 20, 33);
    set_mana(w, id, 0);
    set_damage(w, id, 3, 7);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 10, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 20);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 15);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 30.0);
    set_skill_range(w, id, SkillName::Tactics, 29.3, 44.0);
    set_skill_range(w, id, SkillName::Wrestling, 29.3, 44.0);
    fame(w, id, 300);
    karma(w, id, 0);
    virtual_armor(w, id, 10);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, -0.9);
}
fn mountain_goat_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &MOUNTAIN_GOAT,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    mountain_goat_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.MountainGoat`.
    MOUNTAIN_GOAT, MOUNTAIN_GOAT_C = "Server.Mobiles.MountainGoat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(mountain_goat_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a mountain goat corpse"),
        default_name: |_w, _id| Some("a mountain goat"),
        meat: |_w, _id| 2,
        hides: |_w, _id| 12,
        favorite_food: |_w, _id| FoodType::GRAINS_AND_HAY | FoodType::FRUITS_AND_VEGGIES,
    }
    m {
    }
}

pub(crate) fn pack_horse_init(w: &mut World, id: MobileId) {
    body(w, id, 291);
    base_sound_id(w, id, 0xA8);
    set_str_range(w, id, 44, 120);
    set_dex_range(w, id, 36, 55);
    set_int_range(w, id, 6, 10);
    set_hits_range(w, id, 61, 80);
    set_stam_range(w, id, 81, 100);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 11);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 15);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 15);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 30.0);
    set_skill_range(w, id, SkillName::Tactics, 29.3, 44.0);
    set_skill_range(w, id, SkillName::Wrestling, 29.3, 44.0);
    fame(w, id, 0);
    karma(w, id, 200);
    virtual_armor(w, id, 16);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 11.1);
    crate::hand::pack_horse_ctor1(w, id);
}
fn pack_horse_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &PACK_HORSE,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    pack_horse_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.PackHorse`.
    PACK_HORSE, PACK_HORSE_C = "Server.Mobiles.PackHorse" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(pack_horse_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a horse corpse"),
        default_name: |_w, _id| Some("a pack horse"),
        meat: |_w, _id| 3,
        hides: |_w, _id| 10,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
        on_before_death: crate::hand::pack_horse_on_before_death,
        get_inventory_move_result_for: crate::hand::pack_horse_get_inventory_move_result_for,
        is_snoop: crate::hand::pack_horse_is_snoop,
        on_drag_drop: crate::hand::pack_horse_on_drag_drop,
        check_nonlocal_drop: crate::hand::pack_horse_check_nonlocal_drop,
        check_nonlocal_lift: crate::hand::pack_horse_check_nonlocal_lift,
        on_double_click: crate::hand::pack_horse_on_double_click,
        get_context_menu_entries: crate::hand::pack_horse_get_context_menu_entries,
    }
}

pub(crate) fn pack_llama_init(w: &mut World, id: MobileId) {
    body(w, id, 292);
    base_sound_id(w, id, 0x3F3);
    set_str_range(w, id, 52, 80);
    set_dex_range(w, id, 36, 55);
    set_int_range(w, id, 16, 30);
    set_hits(w, id, 50);
    set_stam_range(w, id, 86, 105);
    set_mana(w, id, 0);
    set_damage(w, id, 2, 6);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 15);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 15);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 15);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 20.0);
    set_skill_range(w, id, SkillName::Tactics, 19.2, 29.0);
    set_skill_range(w, id, SkillName::Wrestling, 19.2, 29.0);
    fame(w, id, 0);
    karma(w, id, 200);
    virtual_armor(w, id, 16);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 11.1);
    crate::hand::pack_llama_ctor1(w, id);
}
fn pack_llama_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &PACK_LLAMA,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    pack_llama_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.PackLlama`.
    PACK_LLAMA, PACK_LLAMA_C = "Server.Mobiles.PackLlama" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(pack_llama_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a llama corpse"),
        default_name: |_w, _id| Some("a pack llama"),
        meat: |_w, _id| 1,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
        on_before_death: crate::hand::pack_llama_on_before_death,
        get_inventory_move_result_for: crate::hand::pack_llama_get_inventory_move_result_for,
        is_snoop: crate::hand::pack_llama_is_snoop,
        on_drag_drop: crate::hand::pack_llama_on_drag_drop,
        check_nonlocal_drop: crate::hand::pack_llama_check_nonlocal_drop,
        check_nonlocal_lift: crate::hand::pack_llama_check_nonlocal_lift,
        on_double_click: crate::hand::pack_llama_on_double_click,
        get_context_menu_entries: crate::hand::pack_llama_get_context_menu_entries,
    }
}

pub(crate) fn pig_init(w: &mut World, id: MobileId) {
    body(w, id, 0xCB);
    base_sound_id(w, id, 0xC4);
    set_str(w, id, 20);
    set_dex(w, id, 20);
    set_int(w, id, 5);
    set_hits(w, id, 12);
    set_mana(w, id, 0);
    set_damage(w, id, 2, 4);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 10, 15);
    set_skill(w, id, SkillName::MagicResist, 5.0);
    set_skill(w, id, SkillName::Tactics, 5.0);
    set_skill(w, id, SkillName::Wrestling, 5.0);
    fame(w, id, 150);
    karma(w, id, 0);
    virtual_armor(w, id, 12);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 11.1);
}
fn pig_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &PIG,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    pig_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Pig`.
    PIG, PIG_C = "Server.Mobiles.Pig" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(pig_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a pig corpse"),
        default_name: |_w, _id| Some("a pig"),
        meat: |_w, _id| 1,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
    }
}

pub(crate) fn sheep_init(w: &mut World, id: MobileId) {
    crate::hand::sheep_init(w, id);
    body(w, id, 0xCF);
    base_sound_id(w, id, 0xD6);
    set_str(w, id, 19);
    set_dex(w, id, 25);
    set_int(w, id, 5);
    set_hits(w, id, 12);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 2);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 5, 10);
    set_skill(w, id, SkillName::MagicResist, 5.0);
    set_skill(w, id, SkillName::Tactics, 6.0);
    set_skill(w, id, SkillName::Wrestling, 5.0);
    fame(w, id, 300);
    karma(w, id, 0);
    virtual_armor(w, id, 6);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 11.1);
}
fn sheep_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SHEEP,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    sheep_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Sheep`.
    SHEEP, SHEEP_C = "Server.Mobiles.Sheep" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(sheep_new), state crate::hand::sheep_serialize, crate::hand::sheep_deserialize;
    c {
        corpse_name: |_w, _id| Some("a sheep corpse"),
        default_name: |_w, _id| Some("a sheep"),
        meat: |_w, _id| 3,
        meat_type: |_w, _id| MeatType::LambLeg,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
        wool: crate::hand::sheep_wool,
        on_think: crate::hand::sheep_on_think,
    }
    m {
    }
}

pub(crate) fn walrus_init(w: &mut World, id: MobileId) {
    body(w, id, 0xDD);
    base_sound_id(w, id, 0xE0);
    set_str_range(w, id, 21, 29);
    set_dex_range(w, id, 46, 55);
    set_int_range(w, id, 16, 20);
    set_hits_range(w, id, 14, 17);
    set_mana(w, id, 0);
    set_damage(w, id, 4, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 20.0);
    set_skill_range(w, id, SkillName::Tactics, 19.2, 29.0);
    set_skill_range(w, id, SkillName::Wrestling, 19.2, 29.0);
    fame(w, id, 150);
    karma(w, id, 0);
    virtual_armor(w, id, 18);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 35.1);
}
fn walrus_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &WALRUS,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    walrus_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Walrus`.
    WALRUS, WALRUS_C = "Server.Mobiles.Walrus" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(walrus_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a walrus corpse"),
        default_name: |_w, _id| Some("a walrus"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 12,
        favorite_food: |_w, _id| FoodType::FISH,
    }
    m {
    }
}

pub(crate) fn beetle_init(w: &mut World, id: MobileId) {
    set_str(w, id, 300);
    set_dex(w, id, 100);
    set_int(w, id, 500);
    set_hits(w, id, 200);
    set_damage(w, id, 7, 20);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 30);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);
    set_skill(w, id, SkillName::MagicResist, 80.0);
    set_skill(w, id, SkillName::Tactics, 100.0);
    set_skill(w, id, SkillName::Wrestling, 100.0);
    fame(w, id, 4000);
    karma(w, id, -4000);
    tamable(w, id, true);
    control_slots(w, id, 3);
    min_tame_skill(w, id, 29.1);
    crate::hand::beetle_ctor1(w, id);
}
fn beetle_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &BEETLE,
        0x317,
        0x3EBC,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    beetle_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Beetle`.
    BEETLE, BEETLE_C = "Server.Mobiles.Beetle" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(beetle_new), encoded false;
    c {
        default_name: |_w, _id| Some("a giant beetle"),
        steps_max: |_w, _id| 4480,
        corpse_name: |_w, _id| Some("a giant beetle corpse"),
        subdue_before_tame: |_w, _id| true,
        reduce_speed_with_damage: |_w, _id| false,
        favorite_food: |_w, _id| FoodType::MEAT,
        on_harmful_spell: crate::hand::beetle_on_harmful_spell,
    }
    m {
        get_anger_sound: |_w, _id| 0x21D,
        get_idle_sound: |_w, _id| 0x21D,
        get_attack_sound: |_w, _id| 0x162,
        get_hurt_sound: |_w, _id| 0x163,
        get_death_sound: |_w, _id| 0x21D,
        on_combatant_change: crate::hand::beetle_on_combatant_change,
        on_before_death: crate::hand::beetle_on_before_death,
        get_inventory_move_result_for: crate::hand::beetle_get_inventory_move_result_for,
        is_snoop: crate::hand::beetle_is_snoop,
        on_drag_drop: crate::hand::beetle_on_drag_drop,
        check_nonlocal_drop: crate::hand::beetle_check_nonlocal_drop,
        check_nonlocal_lift: crate::hand::beetle_check_nonlocal_lift,
        get_context_menu_entries: crate::hand::beetle_get_context_menu_entries,
    }
}

pub(crate) fn desert_ostard_init(w: &mut World, id: MobileId) {
    base_sound_id(w, id, 0x270);
    set_str_range(w, id, 94, 170);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 6, 10);
    set_hits_range(w, id, 71, 88);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 11);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 15);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 30.0);
    set_skill_range(w, id, SkillName::Tactics, 25.3, 40.0);
    set_skill_range(w, id, SkillName::Wrestling, 29.3, 44.0);
    fame(w, id, 450);
    karma(w, id, 0);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 29.1);
}
fn desert_ostard_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &DESERT_OSTARD,
        0xD2,
        0x3EA3,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    desert_ostard_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.DesertOstard`.
    DESERT_OSTARD, DESERT_OSTARD_C = "Server.Mobiles.DesertOstard" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(desert_ostard_new), encoded false;
    c {
        default_name: |_w, _id| Some("a desert ostard"),
        corpse_name: |_w, _id| Some("an ostard corpse"),
        meat: |_w, _id| 3,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
        pack_instinct: |_w, _id| PackInstinct::OSTARD,
    }
    m {
    }
}

pub(crate) fn fire_steed_init(w: &mut World, id: MobileId) {
    base_sound_id(w, id, 0xA8);
    set_str_range(w, id, 376, 400);
    set_dex_range(w, id, 91, 120);
    set_int_range(w, id, 291, 300);
    set_hits_range(w, id, 226, 240);
    set_damage(w, id, 11, 30);
    set_damage_type(w, id, ResistanceType::Physical, 20);
    set_damage_type(w, id, ResistanceType::Fire, 80);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 70, 80);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 30);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::MagicResist, 100.0, 120.0);
    set_skill(w, id, SkillName::Tactics, 100.0);
    set_skill(w, id, SkillName::Wrestling, 100.0);
    fame(w, id, 20000);
    karma(w, id, -20000);
    tamable(w, id, true);
    control_slots(w, id, 2);
    min_tame_skill(w, id, 106.0);
    {
        let a0 = "Server.Items.SulfurousAsh";
        let a1 = rnd::min_max(w, 151, 300);
        pack_new_amount(w, id, a0, a1);
    }
    crate::hand::fire_steed_ctor1(w, id);
}
fn fire_steed_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &FIRE_STEED,
        0xBE,
        0x3E9E,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    fire_steed_init(w, id);
    id
}
static FIRE_STEED_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.FireSteed`.
    FIRE_STEED, FIRE_STEED_C = "Server.Mobiles.FireSteed" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(fire_steed_new), encoded false;
    c {
        default_name: |_w, _id| Some("a fire steed"),
        corpse_name: |_w, _id| Some("a fire steed corpse"),
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::DAEMON | PackInstinct::EQUINE,
        get_monster_abilities: |_, _| &FIRE_STEED_ABILITIES,
        generate_loot: |_w, _id| {

        },
    }
    m {
    }
}

pub(crate) fn forest_ostard_init(w: &mut World, id: MobileId) {
    {
        let a0 = rnd::from(w, 2201, 24) | 0x8000;
        hue(w, id, a0);
    }
    base_sound_id(w, id, 0x270);
    set_str_range(w, id, 94, 170);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 6, 10);
    set_hits_range(w, id, 71, 88);
    set_mana(w, id, 0);
    set_damage(w, id, 8, 14);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_skill_range(w, id, SkillName::MagicResist, 27.1, 32.0);
    set_skill_range(w, id, SkillName::Tactics, 29.3, 44.0);
    set_skill_range(w, id, SkillName::Wrestling, 29.3, 44.0);
    fame(w, id, 450);
    karma(w, id, 0);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 29.1);
}
fn forest_ostard_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &FOREST_OSTARD,
        0xDB,
        0x3EA5,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    forest_ostard_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.ForestOstard`.
    FOREST_OSTARD, FOREST_OSTARD_C = "Server.Mobiles.ForestOstard" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(forest_ostard_new), encoded false;
    c {
        default_name: |_w, _id| Some("a forest ostard"),
        corpse_name: |_w, _id| Some("an ostard corpse"),
        meat: |_w, _id| 3,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
        pack_instinct: |_w, _id| PackInstinct::OSTARD,
    }
    m {
    }
}

pub(crate) fn frenzied_ostard_init(w: &mut World, id: MobileId) {
    {
        let a0 = race_hair_hue(w, "Human") | 0x8000;
        hue(w, id, a0);
    }
    base_sound_id(w, id, 0x275);
    set_str_range(w, id, 94, 170);
    set_dex_range(w, id, 96, 115);
    set_int_range(w, id, 6, 10);
    set_hits_range(w, id, 71, 110);
    set_mana(w, id, 0);
    set_damage(w, id, 11, 17);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 30);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 15);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 25);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 25);
    set_skill_range(w, id, SkillName::MagicResist, 75.1, 80.0);
    set_skill_range(w, id, SkillName::Tactics, 79.3, 94.0);
    set_skill_range(w, id, SkillName::Wrestling, 79.3, 94.0);
    fame(w, id, 1500);
    karma(w, id, -1500);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 77.1);
}
fn frenzied_ostard_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &FRENZIED_OSTARD,
        0xDA,
        0x3EA4,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    frenzied_ostard_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.FrenziedOstard`.
    FRENZIED_OSTARD, FRENZIED_OSTARD_C = "Server.Mobiles.FrenziedOstard" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(frenzied_ostard_new), encoded false;
    c {
        default_name: |_w, _id| Some("a frenzied ostard"),
        steps_max: |_w, _id| 5120,
        corpse_name: |_w, _id| Some("an ostard corpse"),
        meat: |_w, _id| 3,
        favorite_food: |_w, _id| FoodType::MEAT | FoodType::FISH | FoodType::EGGS | FoodType::FRUITS_AND_VEGGIES,
        pack_instinct: |_w, _id| PackInstinct::OSTARD,
    }
    m {
    }
}

pub(crate) fn hell_steed_init(w: &mut World, id: MobileId) {
    crate::hand::hell_steed_ctor1(w, id);
}
fn hell_steed_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &HELL_STEED,
        793,
        0x3EBB,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    hell_steed_init(w, id);
    id
}
static HELL_STEED_ABILITIES: [&MonsterAbility; 1] = [&abilities::CHAOS_BREATH];
creature_leaf! {
    /// `Server.Mobiles.HellSteed`.
    HELL_STEED, HELL_STEED_C = "Server.Mobiles.HellSteed" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(hell_steed_new), encoded false;
    c {
        default_name: |_w, _id| Some("a hellsteed"),
        steps_max: |_w, _id| 5120,
        corpse_name: |_w, _id| Some("a hellsteed corpse"),
        poison_immune: |_w, _id| Some(PoisonId(4)),
        get_monster_abilities: |_, _| &HELL_STEED_ABILITIES,
    }
    m {
    }
}

pub(crate) fn grizzled_mare_init(w: &mut World, id: MobileId) {
    hell_steed_init(w, id);
}
fn grizzled_mare_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &GRIZZLED_MARE,
        793,
        0x3EBB,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    grizzled_mare_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.GrizzledMare`.
    GRIZZLED_MARE, GRIZZLED_MARE_C = "Server.Mobiles.GrizzledMare" : HELL_STEED / HELL_STEED_C, ctor Some(grizzled_mare_new), encoded false;
    c {
        default_name: |_w, _id| Some("a grizzled mare"),
        delete_on_release: |_w, _id| true,
    }
    m {
    }
}

pub(crate) fn hiryu_init(w: &mut World, id: MobileId) {
    crate::hand::hiryu_ctor1(w, id);
    set_str_range(w, id, 1201, 1410);
    set_dex_range(w, id, 171, 270);
    set_int_range(w, id, 301, 325);
    set_hits_range(w, id, 901, 1100);
    set_mana(w, id, 60);
    set_damage(w, id, 20, 30);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 70);
    set_resistance_range(w, id, ResistanceType::Fire, 70, 90);
    set_resistance_range(w, id, ResistanceType::Cold, 15, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 50);
    set_skill_range(w, id, SkillName::Anatomy, 75.1, 80.0);
    set_skill_range(w, id, SkillName::MagicResist, 85.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 100.1, 110.0);
    set_skill_range(w, id, SkillName::Wrestling, 100.1, 120.0);
    fame(w, id, 18000);
    karma(w, id, -18000);
    tamable(w, id, true);
    control_slots(w, id, 4);
    min_tame_skill(w, id, 98.7);
    crate::hand::hiryu_ctor2(w, id);
    crate::hand::hiryu_ctor3(w, id);
}
fn hiryu_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &HIRYU,
        243,
        0x3E94,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    hiryu_init(w, id);
    id
}
static HIRYU_ABILITIES: [&MonsterAbility; 1] = [&abilities::GRASPING_CLAW];
creature_leaf! {
    /// `Server.Mobiles.Hiryu`.
    HIRYU, HIRYU_C = "Server.Mobiles.Hiryu" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(hiryu_new), encoded false;
    c {
        default_name: |_w, _id| Some("a hiryu"),
        steps_max: |_w, _id| 4480,
        corpse_name: |_w, _id| Some("a hiryu corpse"),
        weapon_ability_chance: |_w, _id| 0.07,
        stat_loss_after_tame: |_w, _id| true,
        treasure_map_level: |_w, _id| 5,
        meat: |_w, _id| 16,
        hides: |_w, _id| 60,
        favorite_food: |_w, _id| FoodType::MEAT,
        can_anger_on_tame: |_w, _id| true,
        get_weapon_ability: |_w, _id| Some(Ability::Dismount),
        get_monster_abilities: |_, _| &HIRYU_ABILITIES,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::filthy_rich(w), 3);
            add_loot_n(w, id, &loot_pack::GEMS, 4);
        },
    }
    m {
        get_anger_sound: |_w, _id| 0x4FE,
        get_idle_sound: |_w, _id| 0x4FD,
        get_attack_sound: |_w, _id| 0x4FC,
        get_hurt_sound: |_w, _id| 0x4FF,
        get_death_sound: |_w, _id| 0x4FB,
    }
}

static HORSE_M_I_DS: [i32; 8] = [0xC8, 0x3E9F, 0xE2, 0x3EA0, 0xE4, 0x3EA1, 0xCC, 0x3EA2];
pub(crate) fn horse_init(w: &mut World, id: MobileId) {
    let random = rnd::below(w, 4);
    body(w, id, HORSE_M_I_DS[(random * 2) as usize]);
    muo_content::creature::mount::set_item_id(w, id, HORSE_M_I_DS[(random * 2 + 1) as usize]);
    base_sound_id(w, id, 0xA8);
    set_str_range(w, id, 22, 98);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 6, 10);
    set_hits_range(w, id, 28, 45);
    set_mana(w, id, 0);
    set_damage(w, id, 3, 4);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 30.0);
    set_skill_range(w, id, SkillName::Tactics, 29.3, 44.0);
    set_skill_range(w, id, SkillName::Wrestling, 29.3, 44.0);
    fame(w, id, 300);
    karma(w, id, 300);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 29.1);
}
fn horse_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &HORSE,
        0xE2,
        0x3EA0,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    horse_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Horse`.
    HORSE, HORSE_C = "Server.Mobiles.Horse" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(horse_new), encoded false;
    c {
        default_name: |_w, _id| Some("a horse"),
        corpse_name: |_w, _id| Some("a horse corpse"),
        meat: |_w, _id| 3,
        hides: |_w, _id| 10,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
    }
}

pub(crate) fn kirin_init(w: &mut World, id: MobileId) {
    base_sound_id(w, id, 0x3C5);
    set_str_range(w, id, 296, 325);
    set_dex_range(w, id, 86, 105);
    set_int_range(w, id, 186, 225);
    set_hits_range(w, id, 191, 210);
    set_damage(w, id, 16, 22);
    set_damage_type(w, id, ResistanceType::Physical, 70);
    set_damage_type(w, id, ResistanceType::Fire, 10);
    set_damage_type(w, id, ResistanceType::Cold, 10);
    set_damage_type(w, id, ResistanceType::Energy, 10);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 65);
    set_resistance_range(w, id, ResistanceType::Fire, 35, 45);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::EvalInt, 80.1, 90.0);
    set_skill_range(w, id, SkillName::Magery, 60.4, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 90.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 85.3, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 20.1, 22.5);
    set_skill_range(w, id, SkillName::Wrestling, 80.5, 92.5);
    fame(w, id, 9000);
    karma(w, id, 9000);
    tamable(w, id, true);
    control_slots(w, id, 2);
    min_tame_skill(w, id, 95.1);
}
fn kirin_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &KIRIN,
        132,
        0x3EAD,
        AIType::AiMage,
        FightMode::Evil,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    kirin_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Kirin`.
    KIRIN, KIRIN_C = "Server.Mobiles.Kirin" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(kirin_new), encoded false;
    c {
        default_name: |_w, _id| Some("a ki-rin"),
        steps_max: |_w, _id| 4480,
        corpse_name: |_w, _id| Some("a ki-rin corpse"),
        allow_female_rider: |_w, _id| false,
        allow_female_tamer: |_w, _id| false,
        initial_innocent: |_w, _id| true,
        mount_ability_delay: |_w, _id| TimeSpan::from_hours_f64(1.0_f64),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        meat: |_w, _id| 3,
        hides: |_w, _id| 10,
        hide_type: |_w, _id| HideType::Horned,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
        on_disallowed_rider: crate::hand::kirin_on_disallowed_rider,
        do_mount_ability: crate::hand::kirin_do_mount_ability,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
            add_loot(w, id, &loot_pack::LOW_SCROLLS);
            add_loot(w, id, &loot_pack::POTIONS);
        },
    }
    m {
        on_death: |w, id, c| {
            (BASE_MOUNT.on_death)(w, id, c);
            if rnd::double(w) < 0.35 {
                drop_new(w, c, "Server.Items.KirinBrains");
            }
        },
    }
}

pub(crate) fn lesser_hiryu_init(w: &mut World, id: MobileId) {
    crate::hand::lesser_hiryu_ctor1(w, id);
    set_str_range(w, id, 301, 410);
    set_dex_range(w, id, 171, 270);
    set_int_range(w, id, 301, 325);
    set_hits_range(w, id, 401, 600);
    set_mana(w, id, 60);
    set_damage(w, id, 18, 23);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 45, 70);
    set_resistance_range(w, id, ResistanceType::Fire, 60, 80);
    set_resistance_range(w, id, ResistanceType::Cold, 5, 15);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::Anatomy, 75.1, 80.0);
    set_skill_range(w, id, SkillName::MagicResist, 85.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 100.1, 110.0);
    set_skill_range(w, id, SkillName::Wrestling, 100.1, 120.0);
    fame(w, id, 10000);
    karma(w, id, -10000);
    tamable(w, id, true);
    control_slots(w, id, 3);
    min_tame_skill(w, id, 98.7);
    crate::hand::lesser_hiryu_ctor2(w, id);
}
fn lesser_hiryu_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &LESSER_HIRYU,
        243,
        0x3E94,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    lesser_hiryu_init(w, id);
    id
}
static LESSER_HIRYU_ABILITIES: [&MonsterAbility; 1] = [&abilities::GRASPING_CLAW];
creature_leaf! {
    /// `Server.Mobiles.LesserHiryu`.
    LESSER_HIRYU, LESSER_HIRYU_C = "Server.Mobiles.LesserHiryu" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(lesser_hiryu_new), encoded false;
    c {
        default_name: |_w, _id| Some("a lesser hiryu"),
        steps_max: |_w, _id| 4480,
        corpse_name: |_w, _id| Some("a hiryu corpse"),
        weapon_ability_chance: |_w, _id| 0.07,
        stat_loss_after_tame: |_w, _id| true,
        treasure_map_level: |_w, _id| 3,
        meat: |_w, _id| 16,
        hides: |_w, _id| 60,
        favorite_food: |_w, _id| FoodType::MEAT,
        can_anger_on_tame: |_w, _id| true,
        get_weapon_ability: |_w, _id| Some(Ability::Dismount),
        override_bonding_reqs: crate::hand::lesser_hiryu_override_bonding_reqs,
        get_control_chance: crate::hand::lesser_hiryu_get_control_chance,
        get_monster_abilities: |_, _| &LESSER_HIRYU_ABILITIES,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::filthy_rich(w), 2);
            add_loot_n(w, id, &loot_pack::GEMS, 4);
        },
    }
    m {
        get_anger_sound: |_w, _id| 0x4FE,
        get_idle_sound: |_w, _id| 0x4FD,
        get_attack_sound: |_w, _id| 0x4FC,
        get_hurt_sound: |_w, _id| 0x4FF,
        get_death_sound: |_w, _id| 0x4FB,
    }
}

pub(crate) fn nightmare_init(w: &mut World, id: MobileId) {
    base_sound_id(w, id, if w.core_aos() { 0xA8 } else { 0x16A });
    if w.core_tol() {
        if rnd::double(w) < 0.3 {
            set_str_range(w, id, 296, 315);
            control_slots(w, id, 2);
        } else {
            set_str_range(w, id, 496, 525);
            control_slots(w, id, 3);
        }
    } else {
        set_str_range(w, id, 496, 525);
        control_slots(w, id, 2);
    }
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
    set_skill_range(w, id, SkillName::EvalInt, 10.4, 50.0);
    set_skill_range(w, id, SkillName::Magery, 10.4, 50.0);
    set_skill_range(w, id, SkillName::MagicResist, 85.3, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 97.6, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 80.5, 92.5);
    fame(w, id, 14000);
    karma(w, id, -14000);
    virtual_armor(w, id, 60);
    tamable(w, id, true);
    min_tame_skill(w, id, 95.1);
    crate::hand::nightmare_ctor1(w, id);
    {
        let a0 = "Server.Items.SulfurousAsh";
        let a1 = rnd::min_max(w, 3, 5);
        pack_new_amount(w, id, a0, a1);
    }
}
fn nightmare_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &NIGHTMARE,
        0x74,
        0x3EA7,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    nightmare_init(w, id);
    id
}
static NIGHTMARE_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.Nightmare`.
    NIGHTMARE, NIGHTMARE_C = "Server.Mobiles.Nightmare" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(nightmare_new), encoded false;
    c {
        default_name: |_w, _id| Some("a nightmare"),
        steps_max: |_w, _id| 6400,
        corpse_name: |_w, _id| Some("a nightmare corpse"),
        meat: |_w, _id| 5,
        hides: |_w, _id| 10,
        hide_type: |_w, _id| HideType::Barbed,
        favorite_food: |_w, _id| FoodType::MEAT,
        can_anger_on_tame: |_w, _id| true,
        get_monster_abilities: |_, _| &NIGHTMARE_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
            add_loot(w, id, loot_pack::average(w));
            add_loot(w, id, &loot_pack::LOW_SCROLLS);
            add_loot(w, id, &loot_pack::POTIONS);
        },
    }
    m {
        get_anger_sound: crate::hand::nightmare_get_anger_sound,
    }
}

pub(crate) fn ridable_llama_init(w: &mut World, id: MobileId) {
    base_sound_id(w, id, 0x3F3);
    set_str_range(w, id, 21, 49);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 16, 30);
    set_hits_range(w, id, 15, 27);
    set_mana(w, id, 0);
    set_damage(w, id, 3, 5);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 10, 15);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 5, 10);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 20.0);
    set_skill_range(w, id, SkillName::Tactics, 19.2, 29.0);
    set_skill_range(w, id, SkillName::Wrestling, 19.2, 29.0);
    fame(w, id, 300);
    karma(w, id, 0);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 29.1);
}
fn ridable_llama_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &RIDABLE_LLAMA,
        0xDC,
        0x3EA6,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ridable_llama_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.RidableLlama`.
    RIDABLE_LLAMA, RIDABLE_LLAMA_C = "Server.Mobiles.RidableLlama" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(ridable_llama_new), encoded false;
    c {
        default_name: |_w, _id| Some("a ridable llama"),
        steps_max: |_w, _id| 2560,
        corpse_name: |_w, _id| Some("a llama corpse"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 12,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
    }
}

pub(crate) fn ridgeback_init(w: &mut World, id: MobileId) {
    base_sound_id(w, id, 0x3F3);
    set_str_range(w, id, 58, 100);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 16, 30);
    set_hits_range(w, id, 41, 54);
    set_mana(w, id, 0);
    set_damage(w, id, 3, 5);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 25);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 5, 10);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 25.3, 40.0);
    set_skill_range(w, id, SkillName::Tactics, 29.3, 44.0);
    set_skill_range(w, id, SkillName::Wrestling, 35.1, 45.0);
    fame(w, id, 300);
    karma(w, id, 0);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 83.1);
}
fn ridgeback_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &RIDGEBACK,
        187,
        0x3EBA,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ridgeback_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Ridgeback`.
    RIDGEBACK, RIDGEBACK_C = "Server.Mobiles.Ridgeback" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(ridgeback_new), encoded false;
    c {
        default_name: |_w, _id| Some("a ridgeback"),
        steps_max: |_w, _id| 4480,
        corpse_name: |_w, _id| Some("a ridgeback corpse"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 12,
        hide_type: |_w, _id| HideType::Spined,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
        override_bonding_reqs: |_w, _id| true,
        get_control_chance: crate::hand::ridgeback_get_control_chance,
    }
    m {
    }
}

pub(crate) fn savage_ridgeback_init(w: &mut World, id: MobileId) {
    base_sound_id(w, id, 0x3F3);
    set_str_range(w, id, 58, 100);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 16, 30);
    set_hits_range(w, id, 41, 54);
    set_mana(w, id, 0);
    set_damage(w, id, 3, 5);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 15);
    set_resistance_range(w, id, ResistanceType::Cold, 15, 20);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 15);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 25.3, 40.0);
    set_skill_range(w, id, SkillName::Tactics, 29.3, 44.0);
    set_skill_range(w, id, SkillName::Wrestling, 35.1, 45.0);
    fame(w, id, 300);
    karma(w, id, 0);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 83.1);
}
fn savage_ridgeback_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &SAVAGE_RIDGEBACK,
        188,
        0x3EB8,
        AIType::AiMelee,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    savage_ridgeback_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SavageRidgeback`.
    SAVAGE_RIDGEBACK, SAVAGE_RIDGEBACK_C = "Server.Mobiles.SavageRidgeback" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(savage_ridgeback_new), encoded false;
    c {
        default_name: |_w, _id| Some("a savage ridgeback"),
        steps_max: |_w, _id| 4480,
        corpse_name: |_w, _id| Some("a savage ridgeback corpse"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 12,
        hide_type: |_w, _id| HideType::Spined,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
        override_bonding_reqs: |_w, _id| true,
        get_control_chance: crate::hand::savage_ridgeback_get_control_chance,
    }
    m {
    }
}

pub(crate) fn scaled_swamp_dragon_init(w: &mut World, id: MobileId) {
    set_str_range(w, id, 201, 300);
    set_dex_range(w, id, 66, 85);
    set_int_range(w, id, 61, 100);
    set_hits_range(w, id, 121, 180);
    set_damage(w, id, 3, 4);
    set_damage_type(w, id, ResistanceType::Physical, 75);
    set_damage_type(w, id, ResistanceType::Poison, 25);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::Anatomy, 45.1, 55.0);
    set_skill_range(w, id, SkillName::MagicResist, 45.1, 55.0);
    set_skill_range(w, id, SkillName::Tactics, 45.1, 55.0);
    set_skill_range(w, id, SkillName::Wrestling, 45.1, 55.0);
    fame(w, id, 2000);
    karma(w, id, -2000);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 93.9);
}
fn scaled_swamp_dragon_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &SCALED_SWAMP_DRAGON,
        0x31F,
        0x3EBE,
        AIType::AiMelee,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    scaled_swamp_dragon_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.ScaledSwampDragon`.
    SCALED_SWAMP_DRAGON, SCALED_SWAMP_DRAGON_C = "Server.Mobiles.ScaledSwampDragon" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(scaled_swamp_dragon_new), encoded false;
    c {
        default_name: |_w, _id| Some("a swamp dragon"),
        steps_max: |_w, _id| 4480,
        corpse_name: |_w, _id| Some("a swamp dragon corpse"),
        auto_dispel: |w, id| !read(w, id, false, |d| d.controlled),
        favorite_food: |_w, _id| FoodType::MEAT,
        override_bonding_reqs: |_w, _id| true,
        get_control_chance: crate::hand::scaled_swamp_dragon_get_control_chance,
    }
    m {
    }
}

pub(crate) fn sea_horse_init(w: &mut World, id: MobileId) {
    {
        let a0 = rnd::from(w, 50, 30);
        let a1 = rnd::from(w, 50, 30);
        let a2 = 10;
        init_stats(w, id, a0, a1, a2);
    }
    {
        let a0 = SkillName::MagicResist;
        let a1 = 25.0 + rnd::double(w) * 5.0;
        skill_base(w, id, a0, a1);
    }
    {
        let a0 = SkillName::Wrestling;
        let a1 = 35.0 + rnd::double(w) * 10.0;
        skill_base(w, id, a0, a1);
    }
    {
        let a0 = SkillName::Tactics;
        let a1 = 30.0 + rnd::double(w) * 15.0;
        skill_base(w, id, a0, a1);
    }
}
fn sea_horse_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &SEA_HORSE,
        0x90,
        0x3EB3,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    sea_horse_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SeaHorse`.
    SEA_HORSE, SEA_HORSE_C = "Server.Mobiles.SeaHorse" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(sea_horse_new), encoded false;
    c {
        default_name: |_w, _id| Some("a sea horse"),
        corpse_name: |_w, _id| Some("a sea horse corpse"),
    }
    m {
    }
}

pub(crate) fn silver_steed_init(w: &mut World, id: MobileId) {
    {
        let a0 = rnd::from(w, 50, 30);
        let a1 = rnd::from(w, 50, 30);
        let a2 = 10;
        init_stats(w, id, a0, a1, a2);
    }
    {
        let a0 = SkillName::MagicResist;
        let a1 = 25.0 + rnd::double(w) * 5.0;
        skill_base(w, id, a0, a1);
    }
    {
        let a0 = SkillName::Wrestling;
        let a1 = 35.0 + rnd::double(w) * 10.0;
        skill_base(w, id, a0, a1);
    }
    {
        let a0 = SkillName::Tactics;
        let a1 = 30.0 + rnd::double(w) * 15.0;
        skill_base(w, id, a0, a1);
    }
    control_slots(w, id, 1);
    tamable(w, id, true);
    min_tame_skill(w, id, 103.1);
}
fn silver_steed_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &SILVER_STEED,
        0x75,
        0x3EA8,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    silver_steed_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SilverSteed`.
    SILVER_STEED, SILVER_STEED_C = "Server.Mobiles.SilverSteed" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(silver_steed_new), encoded false;
    c {
        default_name: |_w, _id| Some("a silver steed"),
        steps_max: |_w, _id| 4480,
        corpse_name: |_w, _id| Some("a silver steed corpse"),
    }
    m {
    }
}

pub(crate) fn unicorn_init(w: &mut World, id: MobileId) {
    base_sound_id(w, id, 0x4BC);
    set_str_range(w, id, 296, 325);
    set_dex_range(w, id, 96, 115);
    set_int_range(w, id, 186, 225);
    set_hits_range(w, id, 191, 210);
    set_damage(w, id, 16, 22);
    set_damage_type(w, id, ResistanceType::Physical, 75);
    set_damage_type(w, id, ResistanceType::Energy, 25);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 65);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 55, 65);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 40);
    set_skill_range(w, id, SkillName::EvalInt, 80.1, 90.0);
    set_skill_range(w, id, SkillName::Magery, 60.2, 80.0);
    set_skill_range(w, id, SkillName::Meditation, 50.1, 60.0);
    set_skill_range(w, id, SkillName::MagicResist, 75.3, 90.0);
    set_skill_range(w, id, SkillName::Tactics, 20.1, 22.5);
    set_skill_range(w, id, SkillName::Wrestling, 80.5, 92.5);
    fame(w, id, 9000);
    karma(w, id, 9000);
    tamable(w, id, true);
    control_slots(w, id, 2);
    min_tame_skill(w, id, 95.1);
}
fn unicorn_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &UNICORN,
        0x7A,
        0x3EB4,
        AIType::AiMage,
        FightMode::Evil,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    unicorn_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Unicorn`.
    UNICORN, UNICORN_C = "Server.Mobiles.Unicorn" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(unicorn_new), encoded false;
    c {
        default_name: |_w, _id| Some("a unicorn"),
        steps_max: |_w, _id| 4480,
        corpse_name: |_w, _id| Some("a unicorn corpse"),
        allow_male_rider: |_w, _id| false,
        allow_male_tamer: |_w, _id| false,
        initial_innocent: |_w, _id| true,
        mount_ability_delay: |_w, _id| TimeSpan::from_hours_f64(1.0_f64),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        poison_immune: |_w, _id| Some(PoisonId(4)),
        meat: |_w, _id| 3,
        hides: |_w, _id| 10,
        hide_type: |_w, _id| HideType::Horned,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
        on_disallowed_rider: crate::hand::unicorn_on_disallowed_rider,
        do_mount_ability: crate::hand::unicorn_do_mount_ability,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
            add_loot(w, id, &loot_pack::LOW_SCROLLS);
            add_loot(w, id, &loot_pack::POTIONS);
        },
    }
    m {
        on_death: |w, id, c| {
            (BASE_MOUNT.on_death)(w, id, c);
            if rnd::double(w) < 0.35 {
                drop_new(w, c, "Server.Items.UnicornRibs");
            }
        },
    }
}

pub(crate) fn base_war_horse_init(w: &mut World, id: MobileId) {
    base_sound_id(w, id, 0xA8);
    {
        let a0 = rnd::from(w, 300, 100);
        let a1 = 125;
        let a2 = 60;
        init_stats(w, id, a0, a1, a2);
    }
    set_str(w, id, 400);
    set_dex(w, id, 125);
    set_int_range(w, id, 51, 55);
    set_hits(w, id, 240);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 8);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 50);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 30.0);
    set_skill_range(w, id, SkillName::Tactics, 29.3, 44.0);
    set_skill_range(w, id, SkillName::Wrestling, 29.3, 44.0);
    fame(w, id, 300);
    karma(w, id, 300);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 29.1);
}
creature_leaf! {
    /// `Server.Mobiles.BaseWarHorse`.
    BASE_WAR_HORSE, BASE_WAR_HORSE_C = "Server.Mobiles.BaseWarHorse" : BASE_MOUNT / BASE_MOUNT_C, ctor None, encoded false;
    c {
        default_name: |_w, _id| Some("a war horse"),
        steps_max: |_w, _id| 6400,
        corpse_name: |_w, _id| Some("a war horse corpse"),
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
    }
}

pub(crate) fn co_m_war_horse_init(w: &mut World, id: MobileId) {
    base_war_horse_init(w, id);
}
fn co_m_war_horse_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &CO_M_WAR_HORSE,
        0x77,
        0x3EB1,
        AIType::AiMelee,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    co_m_war_horse_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.CoMWarHorse`.
    CO_M_WAR_HORSE, CO_M_WAR_HORSE_C = "Server.Mobiles.CoMWarHorse" : BASE_WAR_HORSE / BASE_WAR_HORSE_C, ctor Some(co_m_war_horse_new), encoded false;
    c {
    }
    m {
    }
}

pub(crate) fn minax_war_horse_init(w: &mut World, id: MobileId) {
    base_war_horse_init(w, id);
}
fn minax_war_horse_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &MINAX_WAR_HORSE,
        0x78,
        0x3EAF,
        AIType::AiMelee,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    minax_war_horse_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.MinaxWarHorse`.
    MINAX_WAR_HORSE, MINAX_WAR_HORSE_C = "Server.Mobiles.MinaxWarHorse" : BASE_WAR_HORSE / BASE_WAR_HORSE_C, ctor Some(minax_war_horse_new), encoded false;
    c {
    }
    m {
    }
}

pub(crate) fn sl_war_horse_init(w: &mut World, id: MobileId) {
    base_war_horse_init(w, id);
}
fn sl_war_horse_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &SL_WAR_HORSE,
        0x79,
        0x3EB0,
        AIType::AiMelee,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    sl_war_horse_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SLWarHorse`.
    SL_WAR_HORSE, SL_WAR_HORSE_C = "Server.Mobiles.SLWarHorse" : BASE_WAR_HORSE / BASE_WAR_HORSE_C, ctor Some(sl_war_horse_new), encoded false;
    c {
    }
    m {
    }
}

pub(crate) fn tb_war_horse_init(w: &mut World, id: MobileId) {
    base_war_horse_init(w, id);
}
fn tb_war_horse_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &TB_WAR_HORSE,
        0x76,
        0x3EB2,
        AIType::AiMelee,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    tb_war_horse_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.TBWarHorse`.
    TB_WAR_HORSE, TB_WAR_HORSE_C = "Server.Mobiles.TBWarHorse" : BASE_WAR_HORSE / BASE_WAR_HORSE_C, ctor Some(tb_war_horse_new), encoded false;
    c {
    }
    m {
    }
}

pub(crate) fn giant_serpent_init(w: &mut World, id: MobileId) {
    body(w, id, 0x15);
    {
        let a0 = rnd::from(w, 2001, 18);
        hue(w, id, a0);
    }
    base_sound_id(w, id, 219);
    set_str_range(w, id, 186, 215);
    set_dex_range(w, id, 56, 80);
    set_int_range(w, id, 66, 85);
    set_hits_range(w, id, 112, 129);
    set_mana(w, id, 0);
    set_damage(w, id, 7, 17);
    set_damage_type(w, id, ResistanceType::Physical, 40);
    set_damage_type(w, id, ResistanceType::Poison, 60);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 20);
    set_resistance_range(w, id, ResistanceType::Poison, 70, 90);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 20);
    set_skill_range(w, id, SkillName::Poisoning, 70.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 40.0);
    set_skill_range(w, id, SkillName::Tactics, 65.1, 70.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 80.0);
    fame(w, id, 2500);
    karma(w, id, -2500);
    virtual_armor(w, id, 32);
    pack_new(w, id, "Server.Items.Bone");
}
fn giant_serpent_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GIANT_SERPENT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    giant_serpent_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.GiantSerpent`.
    GIANT_SERPENT, GIANT_SERPENT_C = "Server.Mobiles.GiantSerpent" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(giant_serpent_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a giant serpent corpse"),
        default_name: |_w, _id| Some("a giant snake"),
        poison_immune: |_w, _id| Some(PoisonId(2)),
        hit_poison: |w, _id| if rnd::double(w) < 0.8 { Some(PoisonId(2)) } else { Some(PoisonId(3)) },
        death_adder_charmable: |_w, _id| true,
        meat: |_w, _id| 4,
        hides: |_w, _id| 15,
        hide_type: |_w, _id| HideType::Spined,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
        },
    }
    m {
    }
}

pub(crate) fn ice_serpent_init(w: &mut World, id: MobileId) {
    body(w, id, 89);
    base_sound_id(w, id, 219);
    set_str_range(w, id, 216, 245);
    set_dex_range(w, id, 26, 50);
    set_int_range(w, id, 66, 85);
    set_hits_range(w, id, 130, 147);
    set_mana(w, id, 0);
    set_damage(w, id, 7, 17);
    set_damage_type(w, id, ResistanceType::Physical, 10);
    set_damage_type(w, id, ResistanceType::Cold, 90);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 80, 90);
    set_resistance_range(w, id, ResistanceType::Poison, 15, 25);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 20);
    set_skill_range(w, id, SkillName::Anatomy, 27.5, 50.0);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 40.0);
    set_skill_range(w, id, SkillName::Tactics, 75.1, 80.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 80.0);
    fame(w, id, 3500);
    karma(w, id, -3500);
    virtual_armor(w, id, 32);
    {
        let it = muo_content::loot::random_armor_or_shield_or_weapon(w, false, false);
        pack_item(w, id, it);
    }
    {
        let pick: Option<&str> = match rnd::below(w, 10) {
            0 => Some("Server.Items.LeftArm"),
            1 => Some("Server.Items.RightArm"),
            2 => Some("Server.Items.Torso"),
            3 => Some("Server.Items.Bone"),
            4 => Some("Server.Items.RibCage"),
            5 => Some("Server.Items.RibCage"),
            _ => Some("Server.Items.BonePile"),
        };
        pack_opt(w, id, pick);
    }
    if rnd::double(w) < 0.025 {
        pack_new(w, id, "Server.Items.GlacialStaff");
    }
}
fn ice_serpent_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ICE_SERPENT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ice_serpent_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.IceSerpent`.
    ICE_SERPENT, ICE_SERPENT_C = "Server.Mobiles.IceSerpent" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ice_serpent_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an ice serpent corpse"),
        default_name: |_w, _id| Some("a giant ice serpent"),
        death_adder_charmable: |_w, _id| true,
        meat: |_w, _id| 4,
        hides: |_w, _id| 15,
        hide_type: |_w, _id| HideType::Spined,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

pub(crate) fn ice_snake_init(w: &mut World, id: MobileId) {
    body(w, id, 52);
    hue(w, id, 0x480);
    base_sound_id(w, id, 0xDB);
    set_str_range(w, id, 42, 54);
    set_dex_range(w, id, 36, 45);
    set_int_range(w, id, 26, 30);
    set_mana(w, id, 0);
    set_damage(w, id, 4, 12);
    set_damage_type(w, id, ResistanceType::Physical, 25);
    set_damage_type(w, id, ResistanceType::Cold, 25);
    set_damage_type(w, id, ResistanceType::Poison, 50);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Cold, 80, 90);
    set_resistance_range(w, id, ResistanceType::Poison, 60, 70);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 20.0);
    set_skill_range(w, id, SkillName::Tactics, 39.3, 54.0);
    set_skill_range(w, id, SkillName::Wrestling, 39.3, 54.0);
    fame(w, id, 900);
    karma(w, id, -900);
    virtual_armor(w, id, 30);
}
fn ice_snake_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ICE_SNAKE,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ice_snake_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.IceSnake`.
    ICE_SNAKE, ICE_SNAKE_C = "Server.Mobiles.IceSnake" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ice_snake_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an ice snake corpse"),
        default_name: |_w, _id| Some("an ice snake"),
        death_adder_charmable: |_w, _id| true,
        meat: |_w, _id| 1,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

pub(crate) fn lava_lizard_init(w: &mut World, id: MobileId) {
    body(w, id, 0xCE);
    {
        let list = [0x647, 0x650, 0x659, 0x662, 0x66B, 0x674];
        let i = rnd::below(w, list.len() as i32) as usize;
        hue(w, id, list[i]);
    }
    base_sound_id(w, id, 0x5A);
    set_str_range(w, id, 126, 150);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 11, 20);
    set_hits_range(w, id, 76, 90);
    set_mana(w, id, 0);
    set_damage(w, id, 6, 24);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 45);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::MagicResist, 55.1, 70.0);
    set_skill_range(w, id, SkillName::Tactics, 60.1, 80.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 80.0);
    fame(w, id, 3000);
    karma(w, id, -3000);
    virtual_armor(w, id, 40);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 80.7);
    {
        let a0 = "Server.Items.SulfurousAsh";
        let a1 = rnd::from(w, 4, 10);
        pack_new_amount(w, id, a0, a1);
    }
}
fn lava_lizard_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LAVA_LIZARD,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    lava_lizard_init(w, id);
    id
}
static LAVA_LIZARD_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.LavaLizard`.
    LAVA_LIZARD, LAVA_LIZARD_C = "Server.Mobiles.LavaLizard" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lava_lizard_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a lava lizard corpse"),
        default_name: |_w, _id| Some("a lava lizard"),
        hides: |_w, _id| 12,
        hide_type: |_w, _id| HideType::Spined,
        favorite_food: |_w, _id| FoodType::METAL | FoodType::GOLD,
        get_monster_abilities: |_, _| &LAVA_LIZARD_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

pub(crate) fn lava_serpent_init(w: &mut World, id: MobileId) {
    body(w, id, 90);
    base_sound_id(w, id, 219);
    set_str_range(w, id, 386, 415);
    set_dex_range(w, id, 56, 80);
    set_int_range(w, id, 66, 85);
    set_hits_range(w, id, 232, 249);
    set_mana(w, id, 0);
    set_damage(w, id, 10, 22);
    set_damage_type(w, id, ResistanceType::Physical, 20);
    set_damage_type(w, id, ResistanceType::Fire, 80);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 70, 80);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 20);
    set_skill_range(w, id, SkillName::MagicResist, 25.3, 70.0);
    set_skill_range(w, id, SkillName::Tactics, 65.1, 70.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 80.0);
    fame(w, id, 4500);
    karma(w, id, -4500);
    virtual_armor(w, id, 40);
    pack_new_amount(w, id, "Server.Items.SulfurousAsh", 3);
    pack_new(w, id, "Server.Items.Bone");
}
fn lava_serpent_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LAVA_SERPENT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    lava_serpent_init(w, id);
    id
}
static LAVA_SERPENT_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.LavaSerpent`.
    LAVA_SERPENT, LAVA_SERPENT_C = "Server.Mobiles.LavaSerpent" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lava_serpent_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a lava serpent corpse"),
        default_name: |_w, _id| Some("a lava serpent"),
        death_adder_charmable: |_w, _id| true,
        meat: |_w, _id| 4,
        hides: |_w, _id| 15,
        hide_type: |_w, _id| HideType::Spined,
        get_monster_abilities: |_, _| &LAVA_SERPENT_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
        },
    }
    m {
    }
}

pub(crate) fn lava_snake_init(w: &mut World, id: MobileId) {
    body(w, id, 52);
    {
        let list = [0x647, 0x650, 0x659, 0x662, 0x66B, 0x674];
        let i = rnd::below(w, list.len() as i32) as usize;
        hue(w, id, list[i]);
    }
    base_sound_id(w, id, 0xDB);
    set_str_range(w, id, 43, 55);
    set_dex_range(w, id, 16, 25);
    set_int_range(w, id, 6, 10);
    set_hits_range(w, id, 28, 32);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 8);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 25);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 20);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 20.0);
    set_skill_range(w, id, SkillName::Tactics, 19.3, 34.0);
    set_skill_range(w, id, SkillName::Wrestling, 19.3, 34.0);
    fame(w, id, 600);
    karma(w, id, -600);
    virtual_armor(w, id, 24);
    pack_new(w, id, "Server.Items.SulfurousAsh");
}
fn lava_snake_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LAVA_SNAKE,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    lava_snake_init(w, id);
    id
}
static LAVA_SNAKE_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.LavaSnake`.
    LAVA_SNAKE, LAVA_SNAKE_C = "Server.Mobiles.LavaSnake" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lava_snake_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a lava snake corpse"),
        default_name: |_w, _id| Some("a lava snake"),
        death_adder_charmable: |_w, _id| true,
        meat: |_w, _id| 1,
        get_monster_abilities: |_, _| &LAVA_SNAKE_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::poor(w));
        },
    }
    m {
    }
}

pub(crate) fn snake_init(w: &mut World, id: MobileId) {
    body(w, id, 52);
    {
        let a0 = rnd::from(w, 2001, 18);
        hue(w, id, a0);
    }
    base_sound_id(w, id, 0xDB);
    set_str_range(w, id, 22, 34);
    set_dex_range(w, id, 16, 25);
    set_int_range(w, id, 6, 10);
    set_hits_range(w, id, 15, 19);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 4);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_skill_range(w, id, SkillName::Poisoning, 50.1, 70.0);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 20.0);
    set_skill_range(w, id, SkillName::Tactics, 19.3, 34.0);
    set_skill_range(w, id, SkillName::Wrestling, 19.3, 34.0);
    fame(w, id, 300);
    karma(w, id, -300);
    virtual_armor(w, id, 16);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 59.1);
}
fn snake_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SNAKE,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    snake_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Snake`.
    SNAKE, SNAKE_C = "Server.Mobiles.Snake" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(snake_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a snake corpse"),
        default_name: |_w, _id| Some("a snake"),
        poison_immune: |_w, _id| Some(PoisonId(0)),
        hit_poison: |_w, _id| Some(PoisonId(0)),
        death_adder_charmable: |_w, _id| true,
        meat: |_w, _id| 1,
        favorite_food: |_w, _id| FoodType::EGGS,
    }
    m {
    }
}

pub(crate) fn giant_rat_init(w: &mut World, id: MobileId) {
    body(w, id, 0xD7);
    base_sound_id(w, id, 0x188);
    set_str_range(w, id, 32, 74);
    set_dex_range(w, id, 46, 65);
    set_int_range(w, id, 16, 30);
    set_hits_range(w, id, 26, 39);
    set_mana(w, id, 0);
    set_damage(w, id, 4, 8);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_skill_range(w, id, SkillName::MagicResist, 25.1, 30.0);
    set_skill_range(w, id, SkillName::Tactics, 29.3, 44.0);
    set_skill_range(w, id, SkillName::Wrestling, 29.3, 44.0);
    fame(w, id, 300);
    karma(w, id, -300);
    virtual_armor(w, id, 18);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 29.1);
}
fn giant_rat_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GIANT_RAT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    giant_rat_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.GiantRat`.
    GIANT_RAT, GIANT_RAT_C = "Server.Mobiles.GiantRat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(giant_rat_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a giant rat corpse"),
        default_name: |_w, _id| Some("a giant rat"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 6,
        favorite_food: |_w, _id| FoodType::FISH | FoodType::MEAT | FoodType::FRUITS_AND_VEGGIES | FoodType::EGGS,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::poor(w));
        },
    }
    m {
    }
}

pub(crate) fn jack_rabbit_init(w: &mut World, id: MobileId) {
    body(w, id, 0xCD);
    hue(w, id, 0x1BB);
    set_str(w, id, 15);
    set_dex(w, id, 25);
    set_int(w, id, 5);
    set_hits(w, id, 9);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 2);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 2, 5);
    set_skill(w, id, SkillName::MagicResist, 5.0);
    set_skill(w, id, SkillName::Tactics, 5.0);
    set_skill(w, id, SkillName::Wrestling, 5.0);
    fame(w, id, 150);
    karma(w, id, 0);
    virtual_armor(w, id, 4);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, -18.9);
}
fn jack_rabbit_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &JACK_RABBIT,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    jack_rabbit_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.JackRabbit`.
    JACK_RABBIT, JACK_RABBIT_C = "Server.Mobiles.JackRabbit" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(jack_rabbit_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a jack rabbit corpse"),
        default_name: |_w, _id| Some("a jack rabbit"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 1,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES,
    }
    m {
        get_attack_sound: |_w, _id| 0xC9,
        get_hurt_sound: |_w, _id| 0xCA,
        get_death_sound: |_w, _id| 0xCB,
    }
}

pub(crate) fn rabbit_init(w: &mut World, id: MobileId) {
    body(w, id, 205);
    if rnd::bool(w) {
        {
            let a0 = rnd::from(w, 2301, 18);
            hue(w, id, a0);
        }
    }
    set_str_range(w, id, 6, 10);
    set_dex_range(w, id, 26, 38);
    set_int_range(w, id, 6, 14);
    set_hits_range(w, id, 4, 6);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 1);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 5, 10);
    set_skill(w, id, SkillName::MagicResist, 5.0);
    set_skill(w, id, SkillName::Tactics, 5.0);
    set_skill(w, id, SkillName::Wrestling, 5.0);
    fame(w, id, 150);
    karma(w, id, 0);
    virtual_armor(w, id, 6);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, -18.9);
}
fn rabbit_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &RABBIT,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    rabbit_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Rabbit`.
    RABBIT, RABBIT_C = "Server.Mobiles.Rabbit" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(rabbit_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a hare corpse"),
        default_name: |_w, _id| Some("a rabbit"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 1,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES,
    }
    m {
        get_attack_sound: |_w, _id| 0xC9,
        get_hurt_sound: |_w, _id| 0xCA,
        get_death_sound: |_w, _id| 0xCB,
    }
}

pub(crate) fn sewer_rat_init(w: &mut World, id: MobileId) {
    body(w, id, 238);
    base_sound_id(w, id, 0xCC);
    set_str(w, id, 9);
    set_dex(w, id, 25);
    set_int_range(w, id, 6, 10);
    set_hits(w, id, 6);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 2);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 5, 10);
    set_resistance_range(w, id, ResistanceType::Poison, 15, 25);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill(w, id, SkillName::MagicResist, 5.0);
    set_skill(w, id, SkillName::Tactics, 5.0);
    set_skill(w, id, SkillName::Wrestling, 5.0);
    fame(w, id, 300);
    karma(w, id, -300);
    virtual_armor(w, id, 6);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, -0.9);
}
fn sewer_rat_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SEWER_RAT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    sewer_rat_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SewerRat`.
    SEWER_RAT, SEWER_RAT_C = "Server.Mobiles.SewerRat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(sewer_rat_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a rat corpse"),
        default_name: |_w, _id| Some("a sewer rat"),
        meat: |_w, _id| 1,
        favorite_food: |_w, _id| FoodType::MEAT | FoodType::EGGS | FoodType::FRUITS_AND_VEGGIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::poor(w));
        },
    }
    m {
    }
}

pub(crate) fn jwilson_init(w: &mut World, id: MobileId) {
    {
        let list = [0x89C, 0x8A2, 0x8A8, 0x8AE];
        let i = rnd::below(w, list.len() as i32) as usize;
        hue(w, id, list[i]);
    }
    body(w, id, 0x33);
    virtual_armor(w, id, 8);
    {
        let a0 = rnd::from(w, 22, 13);
        let a1 = rnd::from(w, 16, 6);
        let a2 = rnd::from(w, 16, 5);
        init_stats(w, id, a0, a1, a2);
    }
    {
        let a0 = SkillName::Wrestling;
        let a1 = rnd::from(w, 24, 17);
        skill_base(w, id, a0, a1);
    }
    {
        let a0 = SkillName::Tactics;
        let a1 = rnd::from(w, 18, 14);
        skill_base(w, id, a0, a1);
    }
    {
        let a0 = SkillName::MagicResist;
        let a1 = rnd::from(w, 15, 6);
        skill_base(w, id, a0, a1);
    }
    {
        let a0 = SkillName::Poisoning;
        let a1 = rnd::from(w, 31, 20);
        skill_base(w, id, a0, a1);
    }
    {
        let a0 = rnd::below(w, 1249);
        fame(w, id, a0);
    }
    {
        let a0 = -rnd::below(w, 624);
        karma(w, id, a0);
    }
}
fn jwilson_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &JWILSON,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    jwilson_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Jwilson`.
    JWILSON, JWILSON_C = "Server.Mobiles.Jwilson" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(jwilson_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a jwilson corpse"),
        default_name: |_w, _id| Some("a jwilson"),
    }
    m {
        get_anger_sound: |_w, _id| 0x1C8,
        get_idle_sound: |_w, _id| 0x1C9,
        get_attack_sound: |_w, _id| 0x1CA,
        get_hurt_sound: |_w, _id| 0x1CB,
        get_death_sound: |_w, _id| 0x1CC,
    }
}

pub(crate) fn parrot_init(w: &mut World, id: MobileId) {
    body(w, id, 831);
    {
        let a0 = rnd::from(w, 0, 6);
        virtual_armor(w, id, a0);
    }
    {
        let a0 = 10;
        let a1 = rnd::from(w, 25, 16);
        let a2 = 10;
        init_stats(w, id, a0, a1, a2);
    }
    skill_base(w, id, SkillName::Wrestling, 6.0);
    skill_base(w, id, SkillName::Tactics, 6.0);
    skill_base(w, id, SkillName::MagicResist, 5.0);
    {
        let a0 = rnd::below(w, 1249);
        fame(w, id, a0);
    }
    {
        let a0 = -rnd::below(w, 624);
        karma(w, id, a0);
    }
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 0.0);
}
fn parrot_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &PARROT,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    parrot_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Parrot`.
    PARROT, PARROT_C = "Server.Mobiles.Parrot" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(parrot_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a parrot corpse"),
        default_name: |_w, _id| Some("a parrot"),
    }
    m {
        get_anger_sound: |_w, _id| 0x1B,
        get_idle_sound: |_w, _id| 0x1C,
        get_attack_sound: |_w, _id| 0x1D,
        get_hurt_sound: |_w, _id| 0x1E,
        get_death_sound: |_w, _id| 0x1F,
    }
}

pub(crate) fn bird_init(w: &mut World, id: MobileId) {
    crate::hand::bird_ctor1(w, id);
    body(w, id, 6);
    base_sound_id(w, id, 0x1B);
    {
        let a0 = rnd::min_max(w, 0, 6);
        virtual_armor(w, id, a0);
    }
    set_str(w, id, 10);
    set_dex_range(w, id, 25, 35);
    set_int(w, id, 10);
    set_damage(w, id, 0, 0);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_skill_range(w, id, SkillName::Wrestling, 4.2, 6.4);
    set_skill_range(w, id, SkillName::Tactics, 4.0, 6.0);
    set_skill_range(w, id, SkillName::MagicResist, 4.0, 5.0);
    fame(w, id, 150);
    karma(w, id, 0);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, -6.9);
}
fn bird_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BIRD,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    bird_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Bird`.
    BIRD, BIRD_C = "Server.Mobiles.Bird" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bird_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a bird corpse"),
        meat_type: |_w, _id| MeatType::Bird,
        meat: |_w, _id| 1,
        feathers: |_w, _id| 25,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
    }
}

pub(crate) fn tropical_bird_init(w: &mut World, id: MobileId) {
    {
        let a0 = rnd::from(w, 2101, 30);
        hue(w, id, a0);
    }
    body(w, id, 6);
    base_sound_id(w, id, 0xBF);
    {
        let a0 = rnd::min_max(w, 0, 6);
        virtual_armor(w, id, a0);
    }
    set_str(w, id, 10);
    set_dex_range(w, id, 25, 35);
    set_int(w, id, 10);
    set_damage(w, id, 0, 0);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_skill_range(w, id, SkillName::Wrestling, 4.2, 6.4);
    set_skill_range(w, id, SkillName::Tactics, 4.0, 6.0);
    set_skill_range(w, id, SkillName::MagicResist, 4.0, 5.0);
    fame(w, id, 150);
    karma(w, id, 0);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, -6.9);
}
fn tropical_bird_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &TROPICAL_BIRD,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    tropical_bird_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.TropicalBird`.
    TROPICAL_BIRD, TROPICAL_BIRD_C = "Server.Mobiles.TropicalBird" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(tropical_bird_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a bird corpse"),
        default_name: |_w, _id| Some("a tropical bird"),
        meat_type: |_w, _id| MeatType::Bird,
        meat: |_w, _id| 1,
        feathers: |_w, _id| 25,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
    }
    m {
    }
}

pub(crate) fn cat_init(w: &mut World, id: MobileId) {
    body(w, id, 0xC9);
    {
        let a0 = rnd::from(w, 2301, 18);
        hue(w, id, a0);
    }
    base_sound_id(w, id, 0x69);
    set_str(w, id, 9);
    set_dex(w, id, 35);
    set_int(w, id, 5);
    set_hits(w, id, 6);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 1);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 5, 10);
    set_skill(w, id, SkillName::MagicResist, 5.0);
    set_skill(w, id, SkillName::Tactics, 4.0);
    set_skill(w, id, SkillName::Wrestling, 5.0);
    fame(w, id, 0);
    karma(w, id, 150);
    virtual_armor(w, id, 8);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, -0.9);
}
fn cat_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CAT,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    cat_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Cat`.
    CAT, CAT_C = "Server.Mobiles.Cat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(cat_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a cat corpse"),
        default_name: |_w, _id| Some("a cat"),
        meat: |_w, _id| 1,
        favorite_food: |_w, _id| FoodType::MEAT | FoodType::FISH,
        pack_instinct: |_w, _id| PackInstinct::FELINE,
    }
    m {
    }
}

pub(crate) fn dog_init(w: &mut World, id: MobileId) {
    body(w, id, 0xD9);
    {
        let a0 = rnd::from(w, 2301, 18);
        hue(w, id, a0);
    }
    base_sound_id(w, id, 0x85);
    set_str_range(w, id, 27, 37);
    set_dex_range(w, id, 28, 43);
    set_int_range(w, id, 29, 37);
    set_hits_range(w, id, 17, 22);
    set_mana(w, id, 0);
    set_damage(w, id, 4, 7);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 10, 15);
    set_skill_range(w, id, SkillName::MagicResist, 22.1, 47.0);
    set_skill_range(w, id, SkillName::Tactics, 19.2, 31.0);
    set_skill_range(w, id, SkillName::Wrestling, 19.2, 31.0);
    fame(w, id, 0);
    karma(w, id, 300);
    virtual_armor(w, id, 12);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, -15.3);
}
fn dog_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &DOG,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    dog_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Dog`.
    DOG, DOG_C = "Server.Mobiles.Dog" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dog_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a dog corpse"),
        default_name: |_w, _id| Some("a dog"),
        meat: |_w, _id| 1,
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::CANINE,
    }
    m {
    }
}

pub(crate) fn rat_init(w: &mut World, id: MobileId) {
    body(w, id, 238);
    base_sound_id(w, id, 0xCC);
    set_str(w, id, 9);
    set_dex(w, id, 35);
    set_int(w, id, 5);
    set_hits(w, id, 6);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 2);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 5, 10);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_skill(w, id, SkillName::MagicResist, 4.0);
    set_skill(w, id, SkillName::Tactics, 4.0);
    set_skill(w, id, SkillName::Wrestling, 4.0);
    fame(w, id, 150);
    karma(w, id, -150);
    virtual_armor(w, id, 6);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, -0.9);
}
fn rat_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &RAT,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    rat_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Rat`.
    RAT, RAT_C = "Server.Mobiles.Rat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(rat_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a rat corpse"),
        default_name: |_w, _id| Some("a rat"),
        meat: |_w, _id| 1,
        favorite_food: |_w, _id| FoodType::MEAT | FoodType::FISH | FoodType::EGGS | FoodType::GRAINS_AND_HAY,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::poor(w));
        },
    }
    m {
    }
}

/// Every class of this file.
pub static ALL: &[(&MobileType, &[&str])] = &[
    (&SKELETAL_MOUNT, &[]),
    (&SILVER_SERPENT, &["Server.Mobiles.Silverserpant"]),
    (&ALLIGATOR, &[]),
    (&PHOENIX, &[]),
    (&BLACK_BEAR, &["Server.Mobiles.Bear"]),
    (&DIRE_WOLF, &["Server.Mobiles.Direwolf"]),
    (&EAGLE, &[]),
    (&BROWN_BEAR, &[]),
    (&GRIZZLY_BEAR, &["Server.Mobiles.Grizzlybear"]),
    (&POLAR_BEAR, &["Server.Mobiles.Polarbear"]),
    (&CHICKEN, &[]),
    (&CRANE, &[]),
    (&GREY_WOLF, &["Server.Mobiles.Greywolf"]),
    (&TIMBER_WOLF, &["Server.Mobiles.Timberwolf"]),
    (&WHITE_WOLF, &["Server.Mobiles.Whitewolf"]),
    (&BULL, &[]),
    (&COW, &[]),
    (&COUGAR, &[]),
    (&HELL_CAT, &["Server.Mobiles.Hellcat"]),
    (&PANTHER, &[]),
    (&PREDATOR_HELL_CAT, &["Server.Mobiles.Preditorhellcat"]),
    (&SNOW_LEOPARD, &["Server.Mobiles.Snowleopard"]),
    (&BOAR, &[]),
    (&BULL_FROG, &["Server.Mobiles.Bullfrog"]),
    (&DOLPHIN, &[]),
    (&GAMAN, &[]),
    (&GIANT_TOAD, &["Server.Mobiles.Gianttoad"]),
    (&GOAT, &[]),
    (&GORILLA, &[]),
    (&GREAT_HART, &["Server.Mobiles.Greathart"]),
    (&HIND, &[]),
    (&LLAMA, &[]),
    (&MOUNTAIN_GOAT, &[]),
    (&PACK_HORSE, &[]),
    (&PACK_LLAMA, &[]),
    (&PIG, &[]),
    (&SHEEP, &[]),
    (&WALRUS, &[]),
    (&BEETLE, &[]),
    (&DESERT_OSTARD, &[]),
    (&FIRE_STEED, &[]),
    (&FOREST_OSTARD, &[]),
    (&FRENZIED_OSTARD, &[]),
    (&HELL_STEED, &[]),
    (&GRIZZLED_MARE, &[]),
    (&HIRYU, &[]),
    (
        &HORSE,
        &[
            "Server.Mobiles.BrownHorse",
            "Server.Mobiles.DirtyHorse",
            "Server.Mobiles.GrayHorse",
            "Server.Mobiles.TanHorse",
        ],
    ),
    (&KIRIN, &[]),
    (&LESSER_HIRYU, &[]),
    (&NIGHTMARE, &[]),
    (&RIDABLE_LLAMA, &[]),
    (&RIDGEBACK, &[]),
    (&SAVAGE_RIDGEBACK, &[]),
    (&SCALED_SWAMP_DRAGON, &[]),
    (&SEA_HORSE, &[]),
    (&SILVER_STEED, &[]),
    (&UNICORN, &[]),
    (&CO_M_WAR_HORSE, &[]),
    (&MINAX_WAR_HORSE, &[]),
    (&SL_WAR_HORSE, &[]),
    (&TB_WAR_HORSE, &[]),
    (&GIANT_SERPENT, &["Server.Mobiles.Serpant"]),
    (&ICE_SERPENT, &["Server.Mobiles.Iceserpant"]),
    (&ICE_SNAKE, &["Server.Mobiles.Icesnake"]),
    (&LAVA_LIZARD, &["Server.Mobiles.Lavalizard"]),
    (&LAVA_SERPENT, &["Server.Mobiles.Lavaserpant"]),
    (&LAVA_SNAKE, &["Server.Mobiles.Lavasnake"]),
    (&SNAKE, &[]),
    (&GIANT_RAT, &["Server.Mobiles.Giantrat"]),
    (&JACK_RABBIT, &["Server.Mobiles.Jackrabbit"]),
    (&RABBIT, &[]),
    (&SEWER_RAT, &["Server.Mobiles.Sewerrat"]),
    (&JWILSON, &[]),
    (&PARROT, &[]),
    (&BIRD, &[]),
    (&TROPICAL_BIRD, &[]),
    (&CAT, &["Server.Mobiles.Housecat"]),
    (&DOG, &[]),
    (&RAT, &[]),
];

pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_mobile(t, aliases);
    }
}
