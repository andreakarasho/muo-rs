//! The concrete animal classes that are plain tables. Classes with logic of their own are hand-written in this
//! folder.

use crate::common::*;
use muo_content::creature::mount::{BASE_MOUNT, BASE_MOUNT_C};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::templates::creature_props as tp;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;
use muo_server::type_registry::TypeRegistry;

fn skeletal_mount_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &SKELETAL_MOUNT)
}
creature_leaf! {
    /// `Server.Mobiles.SkeletalMount`.
    SKELETAL_MOUNT, SKELETAL_MOUNT_C = "Server.Mobiles.SkeletalMount" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(skeletal_mount_new), encoded false;
    c {
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        bleed_immune: tp::bleed_immune,
    }
    m {
    }
}

fn silver_serpent_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SILVER_SERPENT)
}
creature_leaf! {
    /// `Server.Mobiles.SilverSerpent`.
    SILVER_SERPENT, SILVER_SERPENT_C = "Server.Mobiles.SilverSerpent" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(silver_serpent_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        faction_allegiance: crate::hand::silver_serpent_faction_allegiance,
        ethic_allegiance: crate::hand::silver_serpent_ethic_allegiance,
        default_name: tp::default_name,
        death_adder_charmable: crate::cprops::death_adder_charmable,
        meat: tp::meat,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn alligator_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ALLIGATOR)
}
creature_leaf! {
    /// `Server.Mobiles.Alligator`.
    ALLIGATOR, ALLIGATOR_C = "Server.Mobiles.Alligator" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(alligator_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn phoenix_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PHOENIX)
}
creature_leaf! {
    /// `Server.Mobiles.Phoenix`.
    PHOENIX, PHOENIX_C = "Server.Mobiles.Phoenix" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(phoenix_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        meat_type: crate::cprops::meat_type,
        feathers: tp::feathers,
        can_fly: tp::can_fly,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn black_bear_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BLACK_BEAR)
}
creature_leaf! {
    /// `Server.Mobiles.BlackBear`.
    BLACK_BEAR, BLACK_BEAR_C = "Server.Mobiles.BlackBear" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(black_bear_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn dire_wolf_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DIRE_WOLF)
}
creature_leaf! {
    /// `Server.Mobiles.DireWolf`.
    DIRE_WOLF, DIRE_WOLF_C = "Server.Mobiles.DireWolf" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dire_wolf_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn eagle_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &EAGLE)
}
creature_leaf! {
    /// `Server.Mobiles.Eagle`.
    EAGLE, EAGLE_C = "Server.Mobiles.Eagle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(eagle_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        meat_type: crate::cprops::meat_type,
        feathers: tp::feathers,
        favorite_food: crate::cprops::favorite_food,
        can_fly: tp::can_fly,
    }
    m {
    }
}

fn brown_bear_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BROWN_BEAR)
}
creature_leaf! {
    /// `Server.Mobiles.BrownBear`.
    BROWN_BEAR, BROWN_BEAR_C = "Server.Mobiles.BrownBear" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(brown_bear_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn grizzly_bear_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GRIZZLY_BEAR)
}
creature_leaf! {
    /// `Server.Mobiles.GrizzlyBear`.
    GRIZZLY_BEAR, GRIZZLY_BEAR_C = "Server.Mobiles.GrizzlyBear" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(grizzly_bear_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn polar_bear_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &POLAR_BEAR)
}
creature_leaf! {
    /// `Server.Mobiles.PolarBear`.
    POLAR_BEAR, POLAR_BEAR_C = "Server.Mobiles.PolarBear" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(polar_bear_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn chicken_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CHICKEN)
}
creature_leaf! {
    /// `Server.Mobiles.Chicken`.
    CHICKEN, CHICKEN_C = "Server.Mobiles.Chicken" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(chicken_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        meat_type: crate::cprops::meat_type,
        favorite_food: crate::cprops::favorite_food,
        can_fly: tp::can_fly,
        feathers: tp::feathers,
    }
    m {
    }
}

fn crane_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CRANE)
}
creature_leaf! {
    /// `Server.Mobiles.Crane`.
    CRANE, CRANE_C = "Server.Mobiles.Crane" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(crane_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        feathers: tp::feathers,
    }
    m {
        get_anger_sound: crate::cprops::get_anger_sound,
        get_idle_sound: crate::cprops::get_idle_sound,
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
    }
}

fn grey_wolf_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GREY_WOLF)
}
creature_leaf! {
    /// `Server.Mobiles.GreyWolf`.
    GREY_WOLF, GREY_WOLF_C = "Server.Mobiles.GreyWolf" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(grey_wolf_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn timber_wolf_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TIMBER_WOLF)
}
creature_leaf! {
    /// `Server.Mobiles.TimberWolf`.
    TIMBER_WOLF, TIMBER_WOLF_C = "Server.Mobiles.TimberWolf" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(timber_wolf_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn white_wolf_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &WHITE_WOLF)
}
creature_leaf! {
    /// `Server.Mobiles.WhiteWolf`.
    WHITE_WOLF, WHITE_WOLF_C = "Server.Mobiles.WhiteWolf" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(white_wolf_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn bull_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BULL)
}
creature_leaf! {
    /// `Server.Mobiles.Bull`.
    BULL, BULL_C = "Server.Mobiles.Bull" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bull_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn cow_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &COW)
}
creature_leaf! {
    /// `Server.Mobiles.Cow`.
    COW, COW_C = "Server.Mobiles.Cow" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(cow_new), state crate::hand::cow_serialize, crate::hand::cow_deserialize;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
        on_double_click: crate::hand::cow_on_double_click,
    }
}

fn cougar_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &COUGAR)
}
creature_leaf! {
    /// `Server.Mobiles.Cougar`.
    COUGAR, COUGAR_C = "Server.Mobiles.Cougar" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(cougar_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn hell_cat_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &HELL_CAT)
}
creature_leaf! {
    /// `Server.Mobiles.HellCat`.
    HELL_CAT, HELL_CAT_C = "Server.Mobiles.HellCat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(hell_cat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
        get_monster_abilities: crate::cprops::get_monster_abilities,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn panther_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PANTHER)
}
creature_leaf! {
    /// `Server.Mobiles.Panther`.
    PANTHER, PANTHER_C = "Server.Mobiles.Panther" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(panther_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn predator_hell_cat_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PREDATOR_HELL_CAT)
}
creature_leaf! {
    /// `Server.Mobiles.PredatorHellCat`.
    PREDATOR_HELL_CAT, PREDATOR_HELL_CAT_C = "Server.Mobiles.PredatorHellCat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(predator_hell_cat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
        get_monster_abilities: crate::cprops::get_monster_abilities,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn snow_leopard_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SNOW_LEOPARD)
}
creature_leaf! {
    /// `Server.Mobiles.SnowLeopard`.
    SNOW_LEOPARD, SNOW_LEOPARD_C = "Server.Mobiles.SnowLeopard" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(snow_leopard_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn boar_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BOAR)
}
creature_leaf! {
    /// `Server.Mobiles.Boar`.
    BOAR, BOAR_C = "Server.Mobiles.Boar" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(boar_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn bull_frog_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BULL_FROG)
}
creature_leaf! {
    /// `Server.Mobiles.BullFrog`.
    BULL_FROG, BULL_FROG_C = "Server.Mobiles.BullFrog" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bull_frog_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn dolphin_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DOLPHIN)
}
creature_leaf! {
    /// `Server.Mobiles.Dolphin`.
    DOLPHIN, DOLPHIN_C = "Server.Mobiles.Dolphin" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dolphin_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        on_think: crate::hand::dolphin_on_think,
    }
    m {
        on_double_click: crate::hand::dolphin_on_double_click,
    }
}

fn gaman_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GAMAN)
}
creature_leaf! {
    /// `Server.Mobiles.Gaman`.
    GAMAN, GAMAN_C = "Server.Mobiles.Gaman" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gaman_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
        get_anger_sound: crate::cprops::get_anger_sound,
        get_idle_sound: crate::cprops::get_idle_sound,
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
        on_death: crate::hand::gaman_on_death,
    }
}

fn giant_toad_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GIANT_TOAD)
}
creature_leaf! {
    /// `Server.Mobiles.GiantToad`.
    GIANT_TOAD, GIANT_TOAD_C = "Server.Mobiles.GiantToad" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(giant_toad_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn goat_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GOAT)
}
creature_leaf! {
    /// `Server.Mobiles.Goat`.
    GOAT, GOAT_C = "Server.Mobiles.Goat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(goat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn gorilla_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GORILLA)
}
creature_leaf! {
    /// `Server.Mobiles.Gorilla`.
    GORILLA, GORILLA_C = "Server.Mobiles.Gorilla" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gorilla_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn great_hart_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GREAT_HART)
}
creature_leaf! {
    /// `Server.Mobiles.GreatHart`.
    GREAT_HART, GREAT_HART_C = "Server.Mobiles.GreatHart" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(great_hart_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
    }
}

fn hind_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &HIND)
}
creature_leaf! {
    /// `Server.Mobiles.Hind`.
    HIND, HIND_C = "Server.Mobiles.Hind" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(hind_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
    }
}

fn llama_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LLAMA)
}
creature_leaf! {
    /// `Server.Mobiles.Llama`.
    LLAMA, LLAMA_C = "Server.Mobiles.Llama" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(llama_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn mountain_goat_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MOUNTAIN_GOAT)
}
creature_leaf! {
    /// `Server.Mobiles.MountainGoat`.
    MOUNTAIN_GOAT, MOUNTAIN_GOAT_C = "Server.Mobiles.MountainGoat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(mountain_goat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn pack_horse_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PACK_HORSE)
}
creature_leaf! {
    /// `Server.Mobiles.PackHorse`.
    PACK_HORSE, PACK_HORSE_C = "Server.Mobiles.PackHorse" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(pack_horse_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
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

fn pack_llama_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PACK_LLAMA)
}
creature_leaf! {
    /// `Server.Mobiles.PackLlama`.
    PACK_LLAMA, PACK_LLAMA_C = "Server.Mobiles.PackLlama" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(pack_llama_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
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

fn pig_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PIG)
}
creature_leaf! {
    /// `Server.Mobiles.Pig`.
    PIG, PIG_C = "Server.Mobiles.Pig" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(pig_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn sheep_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SHEEP)
}
creature_leaf! {
    /// `Server.Mobiles.Sheep`.
    SHEEP, SHEEP_C = "Server.Mobiles.Sheep" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(sheep_new), state crate::hand::sheep_serialize, crate::hand::sheep_deserialize;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        meat_type: crate::cprops::meat_type,
        favorite_food: crate::cprops::favorite_food,
        wool: crate::hand::sheep_wool,
        on_think: crate::hand::sheep_on_think,
    }
    m {
    }
}

fn walrus_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &WALRUS)
}
creature_leaf! {
    /// `Server.Mobiles.Walrus`.
    WALRUS, WALRUS_C = "Server.Mobiles.Walrus" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(walrus_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn beetle_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &BEETLE)
}
creature_leaf! {
    /// `Server.Mobiles.Beetle`.
    BEETLE, BEETLE_C = "Server.Mobiles.Beetle" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(beetle_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        subdue_before_tame: crate::cprops::subdue_before_tame,
        reduce_speed_with_damage: crate::cprops::reduce_speed_with_damage,
        favorite_food: crate::cprops::favorite_food,
        on_harmful_spell: crate::hand::beetle_on_harmful_spell,
    }
    m {
        get_anger_sound: crate::cprops::get_anger_sound,
        get_idle_sound: crate::cprops::get_idle_sound,
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
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

fn desert_ostard_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &DESERT_OSTARD)
}
creature_leaf! {
    /// `Server.Mobiles.DesertOstard`.
    DESERT_OSTARD, DESERT_OSTARD_C = "Server.Mobiles.DesertOstard" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(desert_ostard_new), encoded false;
    c {
        default_name: tp::default_name,
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn fire_steed_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &FIRE_STEED)
}
creature_leaf! {
    /// `Server.Mobiles.FireSteed`.
    FIRE_STEED, FIRE_STEED_C = "Server.Mobiles.FireSteed" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(fire_steed_new), encoded false;
    c {
        default_name: tp::default_name,
        corpse_name: tp::corpse_name,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
        get_monster_abilities: crate::cprops::get_monster_abilities,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
    }
}

fn forest_ostard_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &FOREST_OSTARD)
}
creature_leaf! {
    /// `Server.Mobiles.ForestOstard`.
    FOREST_OSTARD, FOREST_OSTARD_C = "Server.Mobiles.ForestOstard" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(forest_ostard_new), encoded false;
    c {
        default_name: tp::default_name,
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn frenzied_ostard_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &FRENZIED_OSTARD)
}
creature_leaf! {
    /// `Server.Mobiles.FrenziedOstard`.
    FRENZIED_OSTARD, FRENZIED_OSTARD_C = "Server.Mobiles.FrenziedOstard" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(frenzied_ostard_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn hell_steed_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &HELL_STEED)
}
creature_leaf! {
    /// `Server.Mobiles.HellSteed`.
    HELL_STEED, HELL_STEED_C = "Server.Mobiles.HellSteed" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(hell_steed_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        poison_immune: tp::poison_immune,
        get_monster_abilities: crate::cprops::get_monster_abilities,
    }
    m {
    }
}

fn grizzled_mare_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &GRIZZLED_MARE)
}
creature_leaf! {
    /// `Server.Mobiles.GrizzledMare`.
    GRIZZLED_MARE, GRIZZLED_MARE_C = "Server.Mobiles.GrizzledMare" : HELL_STEED / HELL_STEED_C, ctor Some(grizzled_mare_new), encoded false;
    c {
        default_name: tp::default_name,
        delete_on_release: crate::cprops::delete_on_release,
    }
    m {
    }
}

fn hiryu_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &HIRYU)
}
creature_leaf! {
    /// `Server.Mobiles.Hiryu`.
    HIRYU, HIRYU_C = "Server.Mobiles.Hiryu" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(hiryu_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        weapon_ability_chance: crate::cprops::weapon_ability_chance,
        stat_loss_after_tame: crate::cprops::stat_loss_after_tame,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        can_anger_on_tame: crate::cprops::can_anger_on_tame,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        get_monster_abilities: crate::cprops::get_monster_abilities,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: crate::cprops::get_anger_sound,
        get_idle_sound: crate::cprops::get_idle_sound,
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
    }
}

fn horse_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &HORSE)
}
creature_leaf! {
    /// `Server.Mobiles.Horse`.
    HORSE, HORSE_C = "Server.Mobiles.Horse" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(horse_new), encoded false;
    c {
        default_name: tp::default_name,
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn kirin_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &KIRIN)
}
creature_leaf! {
    /// `Server.Mobiles.Kirin`.
    KIRIN, KIRIN_C = "Server.Mobiles.Kirin" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(kirin_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        allow_female_rider: crate::cprops::allow_female_rider,
        allow_female_tamer: crate::cprops::allow_female_tamer,
        initial_innocent: crate::cprops::initial_innocent,
        mount_ability_delay: crate::cprops::mount_ability_delay,
        opposition_group: crate::cprops::opposition_group,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
        on_disallowed_rider: crate::hand::kirin_on_disallowed_rider,
        do_mount_ability: crate::hand::kirin_do_mount_ability,
        generate_loot: tp::generate_loot,
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

fn lesser_hiryu_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &LESSER_HIRYU)
}
creature_leaf! {
    /// `Server.Mobiles.LesserHiryu`.
    LESSER_HIRYU, LESSER_HIRYU_C = "Server.Mobiles.LesserHiryu" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(lesser_hiryu_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        weapon_ability_chance: crate::cprops::weapon_ability_chance,
        stat_loss_after_tame: crate::cprops::stat_loss_after_tame,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        can_anger_on_tame: crate::cprops::can_anger_on_tame,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        override_bonding_reqs: crate::hand::lesser_hiryu_override_bonding_reqs,
        get_control_chance: crate::hand::lesser_hiryu_get_control_chance,
        get_monster_abilities: crate::cprops::get_monster_abilities,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: crate::cprops::get_anger_sound,
        get_idle_sound: crate::cprops::get_idle_sound,
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
    }
}

fn nightmare_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &NIGHTMARE)
}
creature_leaf! {
    /// `Server.Mobiles.Nightmare`.
    NIGHTMARE, NIGHTMARE_C = "Server.Mobiles.Nightmare" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(nightmare_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
        can_anger_on_tame: crate::cprops::can_anger_on_tame,
        get_monster_abilities: crate::cprops::get_monster_abilities,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: crate::hand::nightmare_get_anger_sound,
    }
}

fn ridable_llama_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &RIDABLE_LLAMA)
}
creature_leaf! {
    /// `Server.Mobiles.RidableLlama`.
    RIDABLE_LLAMA, RIDABLE_LLAMA_C = "Server.Mobiles.RidableLlama" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(ridable_llama_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn ridgeback_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &RIDGEBACK)
}
creature_leaf! {
    /// `Server.Mobiles.Ridgeback`.
    RIDGEBACK, RIDGEBACK_C = "Server.Mobiles.Ridgeback" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(ridgeback_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
        override_bonding_reqs: crate::cprops::override_bonding_reqs,
        get_control_chance: crate::hand::ridgeback_get_control_chance,
    }
    m {
    }
}

fn savage_ridgeback_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &SAVAGE_RIDGEBACK)
}
creature_leaf! {
    /// `Server.Mobiles.SavageRidgeback`.
    SAVAGE_RIDGEBACK, SAVAGE_RIDGEBACK_C = "Server.Mobiles.SavageRidgeback" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(savage_ridgeback_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
        override_bonding_reqs: crate::cprops::override_bonding_reqs,
        get_control_chance: crate::hand::savage_ridgeback_get_control_chance,
    }
    m {
    }
}

fn scaled_swamp_dragon_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &SCALED_SWAMP_DRAGON)
}
creature_leaf! {
    /// `Server.Mobiles.ScaledSwampDragon`.
    SCALED_SWAMP_DRAGON, SCALED_SWAMP_DRAGON_C = "Server.Mobiles.ScaledSwampDragon" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(scaled_swamp_dragon_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        auto_dispel: |w, id| !read(w, id, false, |d| d.controlled),
        favorite_food: crate::cprops::favorite_food,
        override_bonding_reqs: crate::cprops::override_bonding_reqs,
        get_control_chance: crate::hand::scaled_swamp_dragon_get_control_chance,
    }
    m {
    }
}

fn sea_horse_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &SEA_HORSE)
}
creature_leaf! {
    /// `Server.Mobiles.SeaHorse`.
    SEA_HORSE, SEA_HORSE_C = "Server.Mobiles.SeaHorse" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(sea_horse_new), encoded false;
    c {
        default_name: tp::default_name,
        corpse_name: tp::corpse_name,
    }
    m {
    }
}

fn silver_steed_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &SILVER_STEED)
}
creature_leaf! {
    /// `Server.Mobiles.SilverSteed`.
    SILVER_STEED, SILVER_STEED_C = "Server.Mobiles.SilverSteed" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(silver_steed_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
    }
    m {
    }
}

fn unicorn_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &UNICORN)
}
creature_leaf! {
    /// `Server.Mobiles.Unicorn`.
    UNICORN, UNICORN_C = "Server.Mobiles.Unicorn" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(unicorn_new), encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        allow_male_rider: crate::cprops::allow_male_rider,
        allow_male_tamer: crate::cprops::allow_male_tamer,
        initial_innocent: crate::cprops::initial_innocent,
        mount_ability_delay: crate::cprops::mount_ability_delay,
        opposition_group: crate::cprops::opposition_group,
        poison_immune: tp::poison_immune,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
        on_disallowed_rider: crate::hand::unicorn_on_disallowed_rider,
        do_mount_ability: crate::hand::unicorn_do_mount_ability,
        generate_loot: tp::generate_loot,
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

creature_leaf! {
    /// `Server.Mobiles.BaseWarHorse`.
    BASE_WAR_HORSE, BASE_WAR_HORSE_C = "Server.Mobiles.BaseWarHorse" : BASE_MOUNT / BASE_MOUNT_C, ctor None, encoded false;
    c {
        default_name: tp::default_name,
        steps_max: tp::steps_max,
        corpse_name: tp::corpse_name,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn co_m_war_horse_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &CO_M_WAR_HORSE)
}
creature_leaf! {
    /// `Server.Mobiles.CoMWarHorse`.
    CO_M_WAR_HORSE, CO_M_WAR_HORSE_C = "Server.Mobiles.CoMWarHorse" : BASE_WAR_HORSE / BASE_WAR_HORSE_C, ctor Some(co_m_war_horse_new), encoded false;
    c {
    }
    m {
    }
}

fn minax_war_horse_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &MINAX_WAR_HORSE)
}
creature_leaf! {
    /// `Server.Mobiles.MinaxWarHorse`.
    MINAX_WAR_HORSE, MINAX_WAR_HORSE_C = "Server.Mobiles.MinaxWarHorse" : BASE_WAR_HORSE / BASE_WAR_HORSE_C, ctor Some(minax_war_horse_new), encoded false;
    c {
    }
    m {
    }
}

fn sl_war_horse_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &SL_WAR_HORSE)
}
creature_leaf! {
    /// `Server.Mobiles.SLWarHorse`.
    SL_WAR_HORSE, SL_WAR_HORSE_C = "Server.Mobiles.SLWarHorse" : BASE_WAR_HORSE / BASE_WAR_HORSE_C, ctor Some(sl_war_horse_new), encoded false;
    c {
    }
    m {
    }
}

fn tb_war_horse_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &TB_WAR_HORSE)
}
creature_leaf! {
    /// `Server.Mobiles.TBWarHorse`.
    TB_WAR_HORSE, TB_WAR_HORSE_C = "Server.Mobiles.TBWarHorse" : BASE_WAR_HORSE / BASE_WAR_HORSE_C, ctor Some(tb_war_horse_new), encoded false;
    c {
    }
    m {
    }
}

fn giant_serpent_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GIANT_SERPENT)
}
creature_leaf! {
    /// `Server.Mobiles.GiantSerpent`.
    GIANT_SERPENT, GIANT_SERPENT_C = "Server.Mobiles.GiantSerpent" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(giant_serpent_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        death_adder_charmable: crate::cprops::death_adder_charmable,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ice_serpent_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ICE_SERPENT)
}
creature_leaf! {
    /// `Server.Mobiles.IceSerpent`.
    ICE_SERPENT, ICE_SERPENT_C = "Server.Mobiles.IceSerpent" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ice_serpent_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        death_adder_charmable: crate::cprops::death_adder_charmable,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ice_snake_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ICE_SNAKE)
}
creature_leaf! {
    /// `Server.Mobiles.IceSnake`.
    ICE_SNAKE, ICE_SNAKE_C = "Server.Mobiles.IceSnake" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ice_snake_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        death_adder_charmable: crate::cprops::death_adder_charmable,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn lava_lizard_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LAVA_LIZARD)
}
creature_leaf! {
    /// `Server.Mobiles.LavaLizard`.
    LAVA_LIZARD, LAVA_LIZARD_C = "Server.Mobiles.LavaLizard" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lava_lizard_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
        get_monster_abilities: crate::cprops::get_monster_abilities,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn lava_serpent_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LAVA_SERPENT)
}
creature_leaf! {
    /// `Server.Mobiles.LavaSerpent`.
    LAVA_SERPENT, LAVA_SERPENT_C = "Server.Mobiles.LavaSerpent" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lava_serpent_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        death_adder_charmable: crate::cprops::death_adder_charmable,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        get_monster_abilities: crate::cprops::get_monster_abilities,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn lava_snake_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LAVA_SNAKE)
}
creature_leaf! {
    /// `Server.Mobiles.LavaSnake`.
    LAVA_SNAKE, LAVA_SNAKE_C = "Server.Mobiles.LavaSnake" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lava_snake_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        death_adder_charmable: crate::cprops::death_adder_charmable,
        meat: tp::meat,
        get_monster_abilities: crate::cprops::get_monster_abilities,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn snake_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SNAKE)
}
creature_leaf! {
    /// `Server.Mobiles.Snake`.
    SNAKE, SNAKE_C = "Server.Mobiles.Snake" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(snake_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        death_adder_charmable: crate::cprops::death_adder_charmable,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn giant_rat_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GIANT_RAT)
}
creature_leaf! {
    /// `Server.Mobiles.GiantRat`.
    GIANT_RAT, GIANT_RAT_C = "Server.Mobiles.GiantRat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(giant_rat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn jack_rabbit_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &JACK_RABBIT)
}
creature_leaf! {
    /// `Server.Mobiles.JackRabbit`.
    JACK_RABBIT, JACK_RABBIT_C = "Server.Mobiles.JackRabbit" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(jack_rabbit_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
    }
}

fn rabbit_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RABBIT)
}
creature_leaf! {
    /// `Server.Mobiles.Rabbit`.
    RABBIT, RABBIT_C = "Server.Mobiles.Rabbit" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(rabbit_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
    }
}

fn sewer_rat_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SEWER_RAT)
}
creature_leaf! {
    /// `Server.Mobiles.SewerRat`.
    SEWER_RAT, SEWER_RAT_C = "Server.Mobiles.SewerRat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(sewer_rat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn jwilson_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &JWILSON)
}
creature_leaf! {
    /// `Server.Mobiles.Jwilson`.
    JWILSON, JWILSON_C = "Server.Mobiles.Jwilson" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(jwilson_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
    }
    m {
        get_anger_sound: crate::cprops::get_anger_sound,
        get_idle_sound: crate::cprops::get_idle_sound,
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
    }
}

fn parrot_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PARROT)
}
creature_leaf! {
    /// `Server.Mobiles.Parrot`.
    PARROT, PARROT_C = "Server.Mobiles.Parrot" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(parrot_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
    }
    m {
        get_anger_sound: crate::cprops::get_anger_sound,
        get_idle_sound: crate::cprops::get_idle_sound,
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
    }
}

fn bird_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BIRD)
}
creature_leaf! {
    /// `Server.Mobiles.Bird`.
    BIRD, BIRD_C = "Server.Mobiles.Bird" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bird_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        meat_type: crate::cprops::meat_type,
        meat: tp::meat,
        feathers: tp::feathers,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn tropical_bird_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TROPICAL_BIRD)
}
creature_leaf! {
    /// `Server.Mobiles.TropicalBird`.
    TROPICAL_BIRD, TROPICAL_BIRD_C = "Server.Mobiles.TropicalBird" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(tropical_bird_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat_type: crate::cprops::meat_type,
        meat: tp::meat,
        feathers: tp::feathers,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn cat_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CAT)
}
creature_leaf! {
    /// `Server.Mobiles.Cat`.
    CAT, CAT_C = "Server.Mobiles.Cat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(cat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn dog_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DOG)
}
creature_leaf! {
    /// `Server.Mobiles.Dog`.
    DOG, DOG_C = "Server.Mobiles.Dog" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dog_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn rat_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RAT)
}
creature_leaf! {
    /// `Server.Mobiles.Rat`.
    RAT, RAT_C = "Server.Mobiles.Rat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(rat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
        generate_loot: tp::generate_loot,
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
