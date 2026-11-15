//! The concrete monster classes that are plain tables. Classes with logic of their own are hand-written in this
//! folder.

use crate::animals::leaves::*;
use crate::common::*;
use muo_content::abilities::{self, MonsterAbility};
use muo_content::creature::mount::{BASE_MOUNT, BASE_MOUNT_C};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::items::weapon::Ability;
use muo_content::templates::creature_props as tp;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;
use muo_server::poison::PoisonId;
use muo_server::type_registry::TypeRegistry;

fn abysmal_horror_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ABYSMAL_HORROR)
}
creature_leaf! {
    /// `Server.Mobiles.AbysmalHorror`.
    ABYSMAL_HORROR, ABYSMAL_HORROR_C = "Server.Mobiles.AbysmalHorror" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(abysmal_horror_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        ignore_young_protection: |w, _id| w.core_ml(),
        default_name: tp::default_name,
        bard_immune: |w, _id| !w.core_se(),
        unprovokable: |w, _id| w.core_se(),
        area_peace_immune: |w, _id| w.core_se(),
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: |w, _id| if rnd::bool(w) { Some(Ability::MortalStrike) } else { Some(Ability::WhirlwindAttack) },
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: crate::hand::abysmal_horror_on_death,
    }
}

fn bone_demon_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BONE_DEMON)
}
creature_leaf! {
    /// `Server.Mobiles.BoneDemon`.
    BONE_DEMON, BONE_DEMON_C = "Server.Mobiles.BoneDemon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bone_demon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bard_immune: |w, _id| !w.core_se(),
        unprovokable: |w, _id| w.core_se(),
        area_peace_immune: |w, _id| w.core_se(),
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn crystal_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CRYSTAL_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.CrystalElemental`.
    CRYSTAL_ELEMENTAL, CRYSTAL_ELEMENTAL_C = "Server.Mobiles.CrystalElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(crystal_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn darknight_creeper_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DARKNIGHT_CREEPER)
}
creature_leaf! {
    /// `Server.Mobiles.DarknightCreeper`.
    DARKNIGHT_CREEPER, DARKNIGHT_CREEPER_C = "Server.Mobiles.DarknightCreeper" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(darknight_creeper_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        ignore_young_protection: |w, _id| w.core_ml(),
        bard_immune: |w, _id| !w.core_se(),
        unprovokable: |w, _id| w.core_se(),
        area_peace_immune: |w, _id| w.core_se(),
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: crate::hand::darknight_creeper_on_death,
    }
}

fn devourer_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DEVOURER)
}
creature_leaf! {
    /// `Server.Mobiles.Devourer`.
    DEVOURER, DEVOURER_C = "Server.Mobiles.Devourer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(devourer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn flesh_golem_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &FLESH_GOLEM)
}
creature_leaf! {
    /// `Server.Mobiles.FleshGolem`.
    FLESH_GOLEM, FLESH_GOLEM_C = "Server.Mobiles.FleshGolem" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(flesh_golem_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn flesh_renderer_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &FLESH_RENDERER)
}
creature_leaf! {
    /// `Server.Mobiles.FleshRenderer`.
    FLESH_RENDERER, FLESH_RENDERER_C = "Server.Mobiles.FleshRenderer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(flesh_renderer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        ignore_young_protection: |w, _id| w.core_ml(),
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        bard_immune: |w, _id| !w.core_se(),
        unprovokable: |w, _id| w.core_se(),
        area_peace_immune: |w, _id| w.core_se(),
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: |w, _id| if rnd::bool(w) { Some(Ability::Dismount) } else { Some(Ability::ParalyzingBlow) },
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: crate::hand::flesh_renderer_on_death,
        get_attack_sound: |_w, _id| 0x34C,
        get_hurt_sound: |_w, _id| 0x354,
        get_anger_sound: |_w, _id| 0x34C,
        get_idle_sound: |_w, _id| 0x34C,
        get_death_sound: |_w, _id| 0x354,
    }
}

fn gibberling_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GIBBERLING)
}
creature_leaf! {
    /// `Server.Mobiles.Gibberling`.
    GIBBERLING, GIBBERLING_C = "Server.Mobiles.Gibberling" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gibberling_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn gore_fiend_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GORE_FIEND)
}
creature_leaf! {
    /// `Server.Mobiles.GoreFiend`.
    GORE_FIEND, GORE_FIEND_C = "Server.Mobiles.GoreFiend" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gore_fiend_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        generate_loot: tp::generate_loot,
    }
    m {
        get_death_sound: |_w, _id| 1218,
    }
}

fn impaler_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &IMPALER)
}
creature_leaf! {
    /// `Server.Mobiles.Impaler`.
    IMPALER, IMPALER_C = "Server.Mobiles.Impaler" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(impaler_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        ignore_young_protection: |w, _id| w.core_ml(),
        auto_dispel: crate::cprops::auto_dispel,
        bard_immune: |w, _id| !w.core_se(),
        unprovokable: |w, _id| w.core_se(),
        area_peace_immune: |w, _id| w.core_se(),
        poison_immune: tp::poison_immune,
        hit_poison: |w, _id| if rnd::double(w) < 0.8 { Some(PoisonId(2)) } else { Some(PoisonId(3)) },
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: |w, _id| if rnd::bool(w) { Some(Ability::MortalStrike) } else { Some(Ability::BleedAttack) },
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: crate::hand::impaler_on_death,
    }
}

fn mound_of_maggots_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MOUND_OF_MAGGOTS)
}
creature_leaf! {
    /// `Server.Mobiles.MoundOfMaggots`.
    MOUND_OF_MAGGOTS, MOUND_OF_MAGGOTS_C = "Server.Mobiles.MoundOfMaggots" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(mound_of_maggots_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn patchwork_skeleton_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PATCHWORK_SKELETON)
}
creature_leaf! {
    /// `Server.Mobiles.PatchworkSkeleton`.
    PATCHWORK_SKELETON, PATCHWORK_SKELETON_C = "Server.Mobiles.PatchworkSkeleton" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(patchwork_skeleton_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ravager_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RAVAGER)
}
creature_leaf! {
    /// `Server.Mobiles.Ravager`.
    RAVAGER, RAVAGER_C = "Server.Mobiles.Ravager" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ravager_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        get_weapon_ability: |w, _id| if rnd::bool(w) { Some(Ability::Dismount) } else { Some(Ability::CrushingBlow) },
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn shadow_knight_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SHADOW_KNIGHT)
}
creature_leaf! {
    /// `Server.Mobiles.ShadowKnight`.
    SHADOW_KNIGHT, SHADOW_KNIGHT_C = "Server.Mobiles.ShadowKnight" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(shadow_knight_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        ignore_young_protection: |w, _id| w.core_ml(),
        opposition_group: crate::cprops::opposition_group,
        bard_immune: |w, _id| !w.core_se(),
        unprovokable: |w, _id| w.core_se(),
        area_peace_immune: |w, _id| w.core_se(),
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: |w, _id| if rnd::bool(w) { Some(Ability::ConcussionBlow) } else { Some(Ability::CrushingBlow) },
        on_think: crate::hand::shadow_knight_on_think,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: crate::hand::shadow_knight_on_death,
        get_idle_sound: |_w, _id| 0x2CE,
        get_death_sound: |_w, _id| 0x2C1,
        get_hurt_sound: |_w, _id| 0x2D1,
        get_attack_sound: |_w, _id| 0x2C8,
        on_combatant_change: crate::hand::shadow_knight_on_combatant_change,
    }
}

fn skittering_hopper_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SKITTERING_HOPPER)
}
creature_leaf! {
    /// `Server.Mobiles.SkitteringHopper`.
    SKITTERING_HOPPER, SKITTERING_HOPPER_C = "Server.Mobiles.SkitteringHopper" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(skittering_hopper_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn treefellow_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TREEFELLOW)
}
creature_leaf! {
    /// `Server.Mobiles.Treefellow`.
    TREEFELLOW, TREEFELLOW_C = "Server.Mobiles.Treefellow" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(treefellow_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        opposition_group: crate::cprops::opposition_group,
        bleed_immune: tp::bleed_immune,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 443,
        get_death_sound: |_w, _id| 31,
        get_attack_sound: |_w, _id| 672,
    }
}

fn vampire_bat_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &VAMPIRE_BAT)
}
creature_leaf! {
    /// `Server.Mobiles.VampireBat`.
    VAMPIRE_BAT, VAMPIRE_BAT_C = "Server.Mobiles.VampireBat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(vampire_bat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 0x29B,
    }
}

fn wailing_banshee_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &WAILING_BANSHEE)
}
creature_leaf! {
    /// `Server.Mobiles.WailingBanshee`.
    WAILING_BANSHEE, WAILING_BANSHEE_C = "Server.Mobiles.WailingBanshee" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(wailing_banshee_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn wanderer_of_the_void_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &WANDERER_OF_THE_VOID)
}
creature_leaf! {
    /// `Server.Mobiles.WandererOfTheVoid`.
    WANDERER_OF_THE_VOID, WANDERER_OF_THE_VOID_C = "Server.Mobiles.WandererOfTheVoid" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(wanderer_of_the_void_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        treasure_map_level: |w, _id| if w.core_aos() { 4 } else { 1 },
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ant_lion_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ANT_LION)
}
creature_leaf! {
    /// `Server.Mobiles.AntLion`.
    ANT_LION, ANT_LION_C = "Server.Mobiles.AntLion" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ant_lion_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x5A,
        get_idle_sound: |_w, _id| 0x5A,
        get_attack_sound: |_w, _id| 0x164,
        get_hurt_sound: |_w, _id| 0x187,
        get_death_sound: |_w, _id| 0x1BA,
    }
}

fn black_solen_infiltrator_queen_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BLACK_SOLEN_INFILTRATOR_QUEEN)
}
creature_leaf! {
    /// `Server.Mobiles.BlackSolenInfiltratorQueen`.
    BLACK_SOLEN_INFILTRATOR_QUEEN, BLACK_SOLEN_INFILTRATOR_QUEEN_C = "Server.Mobiles.BlackSolenInfiltratorQueen" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(black_solen_infiltrator_queen_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        is_enemy: crate::hand::black_solen_infiltrator_queen_is_enemy,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x259,
        get_idle_sound: |_w, _id| 0x259,
        get_attack_sound: |_w, _id| 0x195,
        get_hurt_sound: |_w, _id| 0x250,
        get_death_sound: |_w, _id| 0x25B,
        on_damage: crate::hand::black_solen_infiltrator_queen_on_damage,
    }
}

fn black_solen_infiltrator_warrior_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BLACK_SOLEN_INFILTRATOR_WARRIOR)
}
creature_leaf! {
    /// `Server.Mobiles.BlackSolenInfiltratorWarrior`.
    BLACK_SOLEN_INFILTRATOR_WARRIOR, BLACK_SOLEN_INFILTRATOR_WARRIOR_C = "Server.Mobiles.BlackSolenInfiltratorWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(black_solen_infiltrator_warrior_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        is_enemy: crate::hand::black_solen_infiltrator_warrior_is_enemy,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0xB5,
        get_idle_sound: |_w, _id| 0xB5,
        get_attack_sound: |_w, _id| 0x289,
        get_hurt_sound: |_w, _id| 0xBC,
        get_death_sound: |_w, _id| 0xE4,
        on_damage: crate::hand::black_solen_infiltrator_warrior_on_damage,
    }
}

fn black_solen_queen_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BLACK_SOLEN_QUEEN)
}
creature_leaf! {
    /// `Server.Mobiles.BlackSolenQueen`.
    BLACK_SOLEN_QUEEN, BLACK_SOLEN_QUEEN_C = "Server.Mobiles.BlackSolenQueen" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(black_solen_queen_new), state crate::hand::burst_sac_serialize, crate::hand::burst_sac_deserialize;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        is_enemy: crate::hand::black_solen_queen_is_enemy,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x259,
        get_idle_sound: |_w, _id| 0x259,
        get_attack_sound: |_w, _id| 0x195,
        get_hurt_sound: |_w, _id| 0x250,
        get_death_sound: |_w, _id| 0x25B,
        on_damage: crate::hand::black_solen_queen_on_damage,
        on_before_death: crate::hand::black_solen_queen_on_before_death,
    }
}

fn black_solen_warrior_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BLACK_SOLEN_WARRIOR)
}
creature_leaf! {
    /// `Server.Mobiles.BlackSolenWarrior`.
    BLACK_SOLEN_WARRIOR, BLACK_SOLEN_WARRIOR_C = "Server.Mobiles.BlackSolenWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(black_solen_warrior_new), state crate::hand::burst_sac_serialize, crate::hand::burst_sac_deserialize;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        is_enemy: crate::hand::black_solen_warrior_is_enemy,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0xB5,
        get_idle_sound: |_w, _id| 0xB5,
        get_attack_sound: |_w, _id| 0x289,
        get_hurt_sound: |_w, _id| 0xBC,
        get_death_sound: |_w, _id| 0xE4,
        on_damage: crate::hand::black_solen_warrior_on_damage,
        on_before_death: crate::hand::black_solen_warrior_on_before_death,
    }
}

fn black_solen_worker_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BLACK_SOLEN_WORKER)
}
creature_leaf! {
    /// `Server.Mobiles.BlackSolenWorker`.
    BLACK_SOLEN_WORKER, BLACK_SOLEN_WORKER_C = "Server.Mobiles.BlackSolenWorker" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(black_solen_worker_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        is_enemy: crate::hand::black_solen_worker_is_enemy,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x269,
        get_idle_sound: |_w, _id| 0x269,
        get_attack_sound: |_w, _id| 0x186,
        get_hurt_sound: |_w, _id| 0x1BE,
        get_death_sound: |_w, _id| 0x8E,
        on_damage: crate::hand::black_solen_worker_on_damage,
    }
}

fn red_solen_infiltrator_queen_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RED_SOLEN_INFILTRATOR_QUEEN)
}
creature_leaf! {
    /// `Server.Mobiles.RedSolenInfiltratorQueen`.
    RED_SOLEN_INFILTRATOR_QUEEN, RED_SOLEN_INFILTRATOR_QUEEN_C = "Server.Mobiles.RedSolenInfiltratorQueen" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(red_solen_infiltrator_queen_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        is_enemy: crate::hand::red_solen_infiltrator_queen_is_enemy,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x259,
        get_idle_sound: |_w, _id| 0x259,
        get_attack_sound: |_w, _id| 0x195,
        get_hurt_sound: |_w, _id| 0x250,
        get_death_sound: |_w, _id| 0x25B,
        on_damage: crate::hand::red_solen_infiltrator_queen_on_damage,
    }
}

fn red_solen_infiltrator_warrior_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RED_SOLEN_INFILTRATOR_WARRIOR)
}
creature_leaf! {
    /// `Server.Mobiles.RedSolenInfiltratorWarrior`.
    RED_SOLEN_INFILTRATOR_WARRIOR, RED_SOLEN_INFILTRATOR_WARRIOR_C = "Server.Mobiles.RedSolenInfiltratorWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(red_solen_infiltrator_warrior_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        is_enemy: crate::hand::red_solen_infiltrator_warrior_is_enemy,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0xB5,
        get_idle_sound: |_w, _id| 0xB5,
        get_attack_sound: |_w, _id| 0x289,
        get_hurt_sound: |_w, _id| 0xBC,
        get_death_sound: |_w, _id| 0xE4,
        on_damage: crate::hand::red_solen_infiltrator_warrior_on_damage,
    }
}

fn red_solen_queen_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RED_SOLEN_QUEEN)
}
creature_leaf! {
    /// `Server.Mobiles.RedSolenQueen`.
    RED_SOLEN_QUEEN, RED_SOLEN_QUEEN_C = "Server.Mobiles.RedSolenQueen" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(red_solen_queen_new), state crate::hand::burst_sac_serialize, crate::hand::burst_sac_deserialize;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        is_enemy: crate::hand::red_solen_queen_is_enemy,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x259,
        get_idle_sound: |_w, _id| 0x259,
        get_attack_sound: |_w, _id| 0x195,
        get_hurt_sound: |_w, _id| 0x250,
        get_death_sound: |_w, _id| 0x25B,
        on_damage: crate::hand::red_solen_queen_on_damage,
        on_before_death: crate::hand::red_solen_queen_on_before_death,
    }
}

fn red_solen_warrior_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RED_SOLEN_WARRIOR)
}
creature_leaf! {
    /// `Server.Mobiles.RedSolenWarrior`.
    RED_SOLEN_WARRIOR, RED_SOLEN_WARRIOR_C = "Server.Mobiles.RedSolenWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(red_solen_warrior_new), state crate::hand::burst_sac_serialize, crate::hand::burst_sac_deserialize;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        is_enemy: crate::hand::red_solen_warrior_is_enemy,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0xB5,
        get_idle_sound: |_w, _id| 0xB5,
        get_attack_sound: |_w, _id| 0x289,
        get_hurt_sound: |_w, _id| 0xBC,
        get_death_sound: |_w, _id| 0xE4,
        on_damage: crate::hand::red_solen_warrior_on_damage,
        on_before_death: crate::hand::red_solen_warrior_on_before_death,
    }
}

fn red_solen_worker_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RED_SOLEN_WORKER)
}
creature_leaf! {
    /// `Server.Mobiles.RedSolenWorker`.
    RED_SOLEN_WORKER, RED_SOLEN_WORKER_C = "Server.Mobiles.RedSolenWorker" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(red_solen_worker_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        is_enemy: crate::hand::red_solen_worker_is_enemy,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x269,
        get_idle_sound: |_w, _id| 0x269,
        get_attack_sound: |_w, _id| 0x186,
        get_hurt_sound: |_w, _id| 0x1BE,
        get_death_sound: |_w, _id| 0x8E,
        on_damage: crate::hand::red_solen_worker_on_damage,
    }
}

fn dread_spider_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DREAD_SPIDER)
}
creature_leaf! {
    /// `Server.Mobiles.DreadSpider`.
    DREAD_SPIDER, DREAD_SPIDER_C = "Server.Mobiles.DreadSpider" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dread_spider_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn terathan_avenger_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TERATHAN_AVENGER)
}
creature_leaf! {
    /// `Server.Mobiles.TerathanAvenger`.
    TERATHAN_AVENGER, TERATHAN_AVENGER_C = "Server.Mobiles.TerathanAvenger" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(terathan_avenger_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn terathan_matriarch_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TERATHAN_MATRIARCH)
}
creature_leaf! {
    /// `Server.Mobiles.TerathanMatriarch`.
    TERATHAN_MATRIARCH, TERATHAN_MATRIARCH_C = "Server.Mobiles.TerathanMatriarch" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(terathan_matriarch_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn frost_spider_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &FROST_SPIDER)
}
creature_leaf! {
    /// `Server.Mobiles.FrostSpider`.
    FROST_SPIDER, FROST_SPIDER_C = "Server.Mobiles.FrostSpider" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(frost_spider_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn giant_black_widow_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GIANT_BLACK_WIDOW)
}
creature_leaf! {
    /// `Server.Mobiles.GiantBlackWidow`.
    GIANT_BLACK_WIDOW, GIANT_BLACK_WIDOW_C = "Server.Mobiles.GiantBlackWidow" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(giant_black_widow_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        favorite_food: crate::cprops::favorite_food,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn giant_spider_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GIANT_SPIDER)
}
creature_leaf! {
    /// `Server.Mobiles.GiantSpider`.
    GIANT_SPIDER, GIANT_SPIDER_C = "Server.Mobiles.GiantSpider" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(giant_spider_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn terathan_drone_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TERATHAN_DRONE)
}
creature_leaf! {
    /// `Server.Mobiles.TerathanDrone`.
    TERATHAN_DRONE, TERATHAN_DRONE_C = "Server.Mobiles.TerathanDrone" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(terathan_drone_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn terathan_warrior_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TERATHAN_WARRIOR)
}
creature_leaf! {
    /// `Server.Mobiles.TerathanWarrior`.
    TERATHAN_WARRIOR, TERATHAN_WARRIOR_C = "Server.Mobiles.TerathanWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(terathan_warrior_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn acid_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ACID_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.AcidElemental`.
    ACID_ELEMENTAL, ACID_ELEMENTAL_C = "Server.Mobiles.AcidElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(acid_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        hit_poison: crate::cprops::hit_poison,
        hit_poison_chance: crate::cprops::hit_poison_chance,
        treasure_map_level: |w, _id| if w.core_aos() { 2 } else { 3 },
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn air_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &AIR_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.AirElemental`.
    AIR_ELEMENTAL, AIR_ELEMENTAL_C = "Server.Mobiles.AirElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(air_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn blood_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BLOOD_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.BloodElemental`.
    BLOOD_ELEMENTAL, BLOOD_ELEMENTAL_C = "Server.Mobiles.BloodElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(blood_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn efreet_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &EFREET)
}
creature_leaf! {
    /// `Server.Mobiles.Efreet`.
    EFREET, EFREET_C = "Server.Mobiles.Efreet" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(efreet_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: |w, _id| if w.core_aos() { 4 } else { 5 },
        generate_loot: crate::hand::efreet_generate_loot,
    }
    m {
    }
}

fn fire_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &FIRE_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.FireElemental`.
    FIRE_ELEMENTAL, FIRE_ELEMENTAL_C = "Server.Mobiles.FireElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(fire_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ice_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ICE_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.IceElemental`.
    ICE_ELEMENTAL, ICE_ELEMENTAL_C = "Server.Mobiles.IceElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ice_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn poison_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &POISON_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.PoisonElemental`.
    POISON_ELEMENTAL, POISON_ELEMENTAL_C = "Server.Mobiles.PoisonElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(poison_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        hit_poison_chance: crate::cprops::hit_poison_chance,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn water_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &WATER_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.WaterElemental`.
    WATER_ELEMENTAL, WATER_ELEMENTAL_C = "Server.Mobiles.WaterElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(water_elemental_new), state crate::hand::water_elemental_serialize, crate::hand::water_elemental_deserialize;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn earth_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &EARTH_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.EarthElemental`.
    EARTH_ELEMENTAL, EARTH_ELEMENTAL_C = "Server.Mobiles.EarthElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(earth_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn snow_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SNOW_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.SnowElemental`.
    SNOW_ELEMENTAL, SNOW_ELEMENTAL_C = "Server.Mobiles.SnowElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(snow_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: |w, _id| { let list = [2, 3]; list[rnd::below(w, list.len() as i32) as usize] },
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ancient_lich_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ANCIENT_LICH)
}
static ANCIENT_LICH_ABILITIES: [&MonsterAbility; 1] = [&abilities::SUMMON_LESSER_UNDEAD_COUNTER];
creature_leaf! {
    /// `Server.Mobiles.AncientLich`.
    ANCIENT_LICH, ANCIENT_LICH_C = "Server.Mobiles.AncientLich" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ancient_lich_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        opposition_group: crate::cprops::opposition_group,
        unprovokable: crate::cprops::unprovokable,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        get_monster_abilities: |_, _| &ANCIENT_LICH_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 0x19D,
        get_anger_sound: |_w, _id| 0x175,
        get_death_sound: |_w, _id| 0x108,
        get_attack_sound: |_w, _id| 0xE2,
        get_hurt_sound: |_w, _id| 0x28B,
    }
}

fn arcane_daemon_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ARCANE_DAEMON)
}
creature_leaf! {
    /// `Server.Mobiles.ArcaneDaemon`.
    ARCANE_DAEMON, ARCANE_DAEMON_C = "Server.Mobiles.ArcaneDaemon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(arcane_daemon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn balron_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BALRON)
}
creature_leaf! {
    /// `Server.Mobiles.Balron`.
    BALRON, BALRON_C = "Server.Mobiles.Balron" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(balron_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn betrayer_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BETRAYER)
}
creature_leaf! {
    /// `Server.Mobiles.Betrayer`.
    BETRAYER, BETRAYER_C = "Server.Mobiles.Betrayer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(betrayer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        always_murderer: tp::always_murderer,
        bard_immune: |w, _id| !w.core_aos(),
        poison_immune: tp::poison_immune,
        meat: tp::meat,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
        get_monster_abilities: crate::hand::betrayer_abilities,
    }
    m {
        on_death: crate::hand::betrayer_on_death,
        get_death_sound: |_w, _id| 0x423,
        get_attack_sound: |_w, _id| 0x23B,
        get_hurt_sound: |_w, _id| 0x140,
    }
}

fn bogle_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BOGLE)
}
creature_leaf! {
    /// `Server.Mobiles.Bogle`.
    BOGLE, BOGLE_C = "Server.Mobiles.Bogle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bogle_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn bone_magi_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BONE_MAGI)
}
creature_leaf! {
    /// `Server.Mobiles.BoneMagi`.
    BONE_MAGI, BONE_MAGI_C = "Server.Mobiles.BoneMagi" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bone_magi_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: crate::cprops::opposition_group,
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn daemon_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DAEMON)
}
creature_leaf! {
    /// `Server.Mobiles.Daemon`.
    DAEMON, DAEMON_C = "Server.Mobiles.Daemon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(daemon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        faction_allegiance: crate::hand::daemon_faction_allegiance,
        ethic_allegiance: crate::hand::daemon_ethic_allegiance,
        can_rummage_corpses: tp::can_rummage_corpses,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        can_fly: tp::can_fly,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn elder_gazer_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ELDER_GAZER)
}
creature_leaf! {
    /// `Server.Mobiles.ElderGazer`.
    ELDER_GAZER, ELDER_GAZER_C = "Server.Mobiles.ElderGazer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(elder_gazer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: |w, _id| if w.core_aos() { 4 } else { 0 },
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn evil_mage_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &EVIL_MAGE)
}
creature_leaf! {
    /// `Server.Mobiles.EvilMage`.
    EVIL_MAGE, EVIL_MAGE_C = "Server.Mobiles.EvilMage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(evil_mage_new), state crate::hand::evil_mage_serialize, crate::hand::evil_mage_deserialize;
    c {
        corpse_name: tp::corpse_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        always_murderer: tp::always_murderer,
        meat: tp::meat,
        treasure_map_level: |w, _id| if w.core_aos() { 1 } else { 0 },
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn evil_mage_lord_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &EVIL_MAGE_LORD)
}
creature_leaf! {
    /// `Server.Mobiles.EvilMageLord`.
    EVIL_MAGE_LORD, EVIL_MAGE_LORD_C = "Server.Mobiles.EvilMageLord" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(evil_mage_lord_new), state crate::hand::evil_mage_serialize, crate::hand::evil_mage_lord_deserialize;
    c {
        corpse_name: tp::corpse_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        always_murderer: tp::always_murderer,
        meat: tp::meat,
        treasure_map_level: |w, _id| if w.core_aos() { 2 } else { 0 },
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn fire_gargoyle_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &FIRE_GARGOYLE)
}
static FIRE_GARGOYLE_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.FireGargoyle`.
    FIRE_GARGOYLE, FIRE_GARGOYLE_C = "Server.Mobiles.FireGargoyle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(fire_gargoyle_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        can_fly: tp::can_fly,
        get_monster_abilities: |_, _| &FIRE_GARGOYLE_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn gargoyle_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GARGOYLE)
}
creature_leaf! {
    /// `Server.Mobiles.Gargoyle`.
    GARGOYLE, GARGOYLE_C = "Server.Mobiles.Gargoyle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gargoyle_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_fly: tp::can_fly,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
    }
}

fn gargoyle_destroyer_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GARGOYLE_DESTROYER)
}
static GARGOYLE_DESTROYER_ABILITIES: [&MonsterAbility; 1] = [&abilities::THROW_HATCHET_COUNTER];
creature_leaf! {
    /// `Server.Mobiles.GargoyleDestroyer`.
    GARGOYLE_DESTROYER, GARGOYLE_DESTROYER_C = "Server.Mobiles.GargoyleDestroyer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gargoyle_destroyer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bard_immune: |w, _id| !w.core_aos(),
        meat: tp::meat,
        can_fly: tp::can_fly,
        get_monster_abilities: |_, _| &GARGOYLE_DESTROYER_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn gargoyle_enforcer_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GARGOYLE_ENFORCER)
}
creature_leaf! {
    /// `Server.Mobiles.GargoyleEnforcer`.
    GARGOYLE_ENFORCER, GARGOYLE_ENFORCER_C = "Server.Mobiles.GargoyleEnforcer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gargoyle_enforcer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_fly: tp::can_fly,
        meat: tp::meat,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn gazer_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GAZER)
}
creature_leaf! {
    /// `Server.Mobiles.Gazer`.
    GAZER, GAZER_C = "Server.Mobiles.Gazer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gazer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn golem_controller_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GOLEM_CONTROLLER)
}
creature_leaf! {
    /// `Server.Mobiles.GolemController`.
    GOLEM_CONTROLLER, GOLEM_CONTROLLER_C = "Server.Mobiles.GolemController" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(golem_controller_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        always_murderer: tp::always_murderer,
        generate_loot: tp::generate_loot,
    }
    m {
        click_title: |_w, _id| false,
        show_fame_title: |_w, _id| false,
    }
}

fn ice_fiend_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ICE_FIEND)
}
creature_leaf! {
    /// `Server.Mobiles.IceFiend`.
    ICE_FIEND, ICE_FIEND_C = "Server.Mobiles.IceFiend" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ice_fiend_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        can_fly: tp::can_fly,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn imp_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &IMP)
}
creature_leaf! {
    /// `Server.Mobiles.Imp`.
    IMP, IMP_C = "Server.Mobiles.Imp" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(imp_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
        can_fly: tp::can_fly,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn lich_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LICH)
}
creature_leaf! {
    /// `Server.Mobiles.Lich`.
    LICH, LICH_C = "Server.Mobiles.Lich" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lich_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        opposition_group: crate::cprops::opposition_group,
        can_rummage_corpses: tp::can_rummage_corpses,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn lich_lord_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LICH_LORD)
}
static LICH_LORD_ABILITIES: [&MonsterAbility; 1] = [&abilities::SUMMON_SKELETONS_COUNTER];
creature_leaf! {
    /// `Server.Mobiles.LichLord`.
    LICH_LORD, LICH_LORD_C = "Server.Mobiles.LichLord" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lich_lord_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        opposition_group: crate::cprops::opposition_group,
        can_rummage_corpses: tp::can_rummage_corpses,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        get_monster_abilities: |_, _| &LICH_LORD_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn orcish_mage_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ORCISH_MAGE)
}
creature_leaf! {
    /// `Server.Mobiles.OrcishMage`.
    ORCISH_MAGE, ORCISH_MAGE_C = "Server.Mobiles.OrcishMage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(orcish_mage_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::ORC),
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        opposition_group: crate::cprops::opposition_group,
        is_enemy: crate::hand::orcish_mage_is_enemy,
        generate_loot: tp::generate_loot,
    }
    m {
        aggressive_action_criminal: crate::hand::orcish_mage_aggressive_action_criminal,
    }
}

fn ratman_mage_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RATMAN_MAGE)
}
creature_leaf! {
    /// `Server.Mobiles.RatmanMage`.
    RATMAN_MAGE, RATMAN_MAGE_C = "Server.Mobiles.RatmanMage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ratman_mage_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::RATMAN),
        can_rummage_corpses: tp::can_rummage_corpses,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn savage_shaman_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SAVAGE_SHAMAN)
}
creature_leaf! {
    /// `Server.Mobiles.SavageShaman`.
    SAVAGE_SHAMAN, SAVAGE_SHAMAN_C = "Server.Mobiles.SavageShaman" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(savage_shaman_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        always_murderer: tp::always_murderer,
        opposition_group: crate::cprops::opposition_group,
        is_enemy: crate::hand::savage_shaman_is_enemy,
        alter_melee_damage_to: crate::hand::savage_shaman_alter_melee_damage_to,
        on_got_melee_attack: crate::hand::savage_shaman_on_got_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
        show_fame_title: |_w, _id| false,
        aggressive_action_criminal: crate::hand::savage_shaman_aggressive_action_criminal,
    }
}

fn shade_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SHADE)
}
creature_leaf! {
    /// `Server.Mobiles.Shade`.
    SHADE, SHADE_C = "Server.Mobiles.Shade" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(shade_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: crate::cprops::opposition_group,
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn skeletal_mage_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SKELETAL_MAGE)
}
creature_leaf! {
    /// `Server.Mobiles.SkeletalMage`.
    SKELETAL_MAGE, SKELETAL_MAGE_C = "Server.Mobiles.SkeletalMage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(skeletal_mage_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: crate::cprops::opposition_group,
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn spectre_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SPECTRE)
}
creature_leaf! {
    /// `Server.Mobiles.Spectre`.
    SPECTRE, SPECTRE_C = "Server.Mobiles.Spectre" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(spectre_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: crate::cprops::opposition_group,
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn spellbinder_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SPELLBINDER)
}
creature_leaf! {
    /// `Server.Mobiles.Spellbinder`.
    SPELLBINDER, SPELLBINDER_C = "Server.Mobiles.Spellbinder" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(spellbinder_new), encoded true;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: crate::cprops::opposition_group,
        poison_immune: tp::poison_immune,
        generate_loot: crate::cprops::generate_loot,
        forced_ai: crate::hand::spellbinder_forced_ai,
    }
    m {
    }
}

fn succubus_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SUCCUBUS)
}
static SUCCUBUS_ABILITIES: [&MonsterAbility; 1] = [&abilities::DRAIN_LIFE_AREA_ATTACK];
creature_leaf! {
    /// `Server.Mobiles.Succubus`.
    SUCCUBUS, SUCCUBUS_C = "Server.Mobiles.Succubus" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(succubus_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        treasure_map_level: tp::treasure_map_level,
        get_monster_abilities: |_, _| &SUCCUBUS_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn titan_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TITAN)
}
creature_leaf! {
    /// `Server.Mobiles.Titan`.
    TITAN, TITAN_C = "Server.Mobiles.Titan" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(titan_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn wraith_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &WRAITH)
}
creature_leaf! {
    /// `Server.Mobiles.Wraith`.
    WRAITH, WRAITH_C = "Server.Mobiles.Wraith" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(wraith_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: crate::cprops::opposition_group,
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn arctic_ogre_lord_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ARCTIC_OGRE_LORD)
}
creature_leaf! {
    /// `Server.Mobiles.ArcticOgreLord`.
    ARCTIC_OGRE_LORD, ARCTIC_OGRE_LORD_C = "Server.Mobiles.ArcticOgreLord" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(arctic_ogre_lord_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn bone_knight_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BONE_KNIGHT)
}
creature_leaf! {
    /// `Server.Mobiles.BoneKnight`.
    BONE_KNIGHT, BONE_KNIGHT_C = "Server.Mobiles.BoneKnight" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bone_knight_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn brigand_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BRIGAND)
}
creature_leaf! {
    /// `Server.Mobiles.Brigand`.
    BRIGAND, BRIGAND_C = "Server.Mobiles.Brigand" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(brigand_new), encoded false;
    c {
        always_murderer: tp::always_murderer,
        generate_loot: tp::generate_loot,
    }
    m {
        click_title: |_w, _id| false,
        on_death: |w, id, c| {
            (BASE_CREATURE.on_death)(w, id, c);
            if rnd::double(w) < 0.9 {
                drop_new(w, c, "Server.Items.SeveredHumanEars");
            }
        },
    }
}

fn chaos_daemon_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CHAOS_DAEMON)
}
creature_leaf! {
    /// `Server.Mobiles.ChaosDaemon`.
    CHAOS_DAEMON, CHAOS_DAEMON_C = "Server.Mobiles.ChaosDaemon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(chaos_daemon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn cursed_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CURSED)
}
creature_leaf! {
    /// `Server.Mobiles.Cursed`.
    CURSED, CURSED_C = "Server.Mobiles.Cursed" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(cursed_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        always_murderer: tp::always_murderer,
        generate_loot: tp::generate_loot,
    }
    m {
        click_title: |_w, _id| false,
        show_fame_title: |_w, _id| false,
        get_attack_sound: |_w, _id| -1,
    }
}

fn cyclops_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CYCLOPS)
}
creature_leaf! {
    /// `Server.Mobiles.Cyclops`.
    CYCLOPS, CYCLOPS_C = "Server.Mobiles.Cyclops" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(cyclops_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn doppleganger_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DOPPLEGANGER)
}
creature_leaf! {
    /// `Server.Mobiles.Doppleganger`.
    DOPPLEGANGER, DOPPLEGANGER_C = "Server.Mobiles.Doppleganger" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(doppleganger_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hides: tp::hides,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn elf_brigand_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ELF_BRIGAND)
}
creature_leaf! {
    /// `Server.Mobiles.ElfBrigand`.
    ELF_BRIGAND, ELF_BRIGAND_C = "Server.Mobiles.ElfBrigand" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(elf_brigand_new), encoded false;
    c {
        always_murderer: tp::always_murderer,
        generate_loot: tp::generate_loot,
    }
    m {
        click_title: |_w, _id| false,
        on_death: |w, id, c| {
            (BASE_CREATURE.on_death)(w, id, c);
            if rnd::double(w) < 0.9 {
                drop_new(w, c, "Server.Items.SeveredElfEars");
            }
        },
    }
}

fn enslaved_gargoyle_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ENSLAVED_GARGOYLE)
}
creature_leaf! {
    /// `Server.Mobiles.EnslavedGargoyle`.
    ENSLAVED_GARGOYLE, ENSLAVED_GARGOYLE_C = "Server.Mobiles.EnslavedGargoyle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(enslaved_gargoyle_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ettin_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ETTIN)
}
creature_leaf! {
    /// `Server.Mobiles.Ettin`.
    ETTIN, ETTIN_C = "Server.Mobiles.Ettin" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ettin_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn executioner_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &EXECUTIONER)
}
creature_leaf! {
    /// `Server.Mobiles.Executioner`.
    EXECUTIONER, EXECUTIONER_C = "Server.Mobiles.Executioner" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(executioner_new), encoded false;
    c {
        always_murderer: tp::always_murderer,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn frost_troll_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &FROST_TROLL)
}
creature_leaf! {
    /// `Server.Mobiles.FrostTroll`.
    FROST_TROLL, FROST_TROLL_C = "Server.Mobiles.FrostTroll" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(frost_troll_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn gazer_larva_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GAZER_LARVA)
}
creature_leaf! {
    /// `Server.Mobiles.GazerLarva`.
    GAZER_LARVA, GAZER_LARVA_C = "Server.Mobiles.GazerLarva" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gazer_larva_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ghoul_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GHOUL)
}
creature_leaf! {
    /// `Server.Mobiles.Ghoul`.
    GHOUL, GHOUL_C = "Server.Mobiles.Ghoul" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ghoul_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn greater_mongbat_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GREATER_MONGBAT)
}
creature_leaf! {
    /// `Server.Mobiles.GreaterMongbat`.
    GREATER_MONGBAT, GREATER_MONGBAT_C = "Server.Mobiles.GreaterMongbat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(greater_mongbat_new), encoded false;
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

fn guardian_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GUARDIAN)
}
creature_leaf! {
    /// `Server.Mobiles.Guardian`.
    GUARDIAN, GUARDIAN_C = "Server.Mobiles.Guardian" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(guardian_new), encoded false;
    c {
    }
    m {
    }
}

fn headless_one_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &HEADLESS_ONE)
}
creature_leaf! {
    /// `Server.Mobiles.HeadlessOne`.
    HEADLESS_ONE, HEADLESS_ONE_C = "Server.Mobiles.HeadlessOne" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(headless_one_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn horde_minion_init(w: &mut World, id: MobileId) {
    body(w, id, 776);
    base_sound_id(w, id, 357);
    set_str_range(w, id, 16, 40);
    set_dex_range(w, id, 31, 60);
    set_int_range(w, id, 11, 25);
    set_hits_range(w, id, 10, 24);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_skill(w, id, SkillName::MagicResist, 10.0);
    set_skill_range(w, id, SkillName::Tactics, 0.1, 15.0);
    set_skill_range(w, id, SkillName::Wrestling, 25.1, 40.0);
    fame(w, id, 500);
    karma(w, id, -500);
    virtual_armor(w, id, 18);
    add_new(w, id, "Server.Items.LightSource");
    pack_new_amount(w, id, "Server.Items.Bone", 3);
}
fn horde_minion_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &HORDE_MINION)
}
creature_leaf! {
    /// `Server.Mobiles.HordeMinion`.
    HORDE_MINION, HORDE_MINION_C = "Server.Mobiles.HordeMinion" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(horde_minion_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
    }
    m {
        get_idle_sound: |_w, _id| 338,
        get_anger_sound: |_w, _id| 338,
        get_death_sound: |_w, _id| 338,
        get_attack_sound: |_w, _id| 406,
        get_hurt_sound: |_w, _id| 194,
    }
}

fn juggernaut_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &JUGGERNAUT)
}
static JUGGERNAUT_ABILITIES: [&MonsterAbility; 1] = [&abilities::COLOSSAL_BLOW];
creature_leaf! {
    /// `Server.Mobiles.Juggernaut`.
    JUGGERNAUT, JUGGERNAUT_C = "Server.Mobiles.Juggernaut" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(juggernaut_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        always_murderer: tp::always_murderer,
        bard_immune: |w, _id| !w.core_aos(),
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        meat: tp::meat,
        treasure_map_level: tp::treasure_map_level,
        get_monster_abilities: |_, _| &JUGGERNAUT_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: crate::hand::juggernaut_on_death,
        get_death_sound: |_w, _id| 0x423,
        get_attack_sound: |_w, _id| 0x23B,
        get_hurt_sound: |_w, _id| 0x140,
    }
}

fn khaldun_summoner_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &KHALDUN_SUMMONER)
}
creature_leaf! {
    /// `Server.Mobiles.KhaldunSummoner`.
    KHALDUN_SUMMONER, KHALDUN_SUMMONER_C = "Server.Mobiles.KhaldunSummoner" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(khaldun_summoner_new), encoded false;
    c {
        default_name: tp::default_name,
        always_murderer: tp::always_murderer,
        unprovokable: crate::cprops::unprovokable,
    }
    m {
        click_title: |_w, _id| false,
        show_fame_title: |_w, _id| false,
        get_idle_sound: |_w, _id| 0x184,
        get_anger_sound: |_w, _id| 0x286,
        get_death_sound: |_w, _id| 0x288,
        get_hurt_sound: |_w, _id| 0x19F,
        on_before_death: crate::hand::khaldun_summoner_on_before_death,
    }
}

fn khaldun_zealot_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &KHALDUN_ZEALOT)
}
creature_leaf! {
    /// `Server.Mobiles.KhaldunZealot`.
    KHALDUN_ZEALOT, KHALDUN_ZEALOT_C = "Server.Mobiles.KhaldunZealot" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(khaldun_zealot_new), encoded false;
    c {
        default_name: tp::default_name,
        always_murderer: tp::always_murderer,
        unprovokable: crate::cprops::unprovokable,
        poison_immune: tp::poison_immune,
    }
    m {
        click_title: |_w, _id| false,
        show_fame_title: |_w, _id| false,
        get_idle_sound: |_w, _id| 0x184,
        get_anger_sound: |_w, _id| 0x286,
        get_death_sound: |_w, _id| 0x288,
        get_hurt_sound: |_w, _id| 0x19F,
        on_before_death: crate::hand::khaldun_zealot_on_before_death,
    }
}

fn moloch_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MOLOCH)
}
creature_leaf! {
    /// `Server.Mobiles.Moloch`.
    MOLOCH, MOLOCH_C = "Server.Mobiles.Moloch" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(moloch_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn mongbat_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MONGBAT)
}
creature_leaf! {
    /// `Server.Mobiles.Mongbat`.
    MONGBAT, MONGBAT_C = "Server.Mobiles.Mongbat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(mongbat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_fly: tp::can_fly,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn mummy_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MUMMY)
}
creature_leaf! {
    /// `Server.Mobiles.Mummy`.
    MUMMY, MUMMY_C = "Server.Mobiles.Mummy" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(mummy_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ogre_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &OGRE)
}
creature_leaf! {
    /// `Server.Mobiles.Ogre`.
    OGRE, OGRE_C = "Server.Mobiles.Ogre" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ogre_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ogre_lord_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &OGRE_LORD)
}
creature_leaf! {
    /// `Server.Mobiles.OgreLord`.
    OGRE_LORD, OGRE_LORD_C = "Server.Mobiles.OgreLord" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ogre_lord_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        faction_allegiance: crate::hand::ogre_lord_faction_allegiance,
        ethic_allegiance: crate::hand::ogre_lord_ethic_allegiance,
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn orc_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ORC)
}
creature_leaf! {
    /// `Server.Mobiles.Orc`.
    ORC, ORC_C = "Server.Mobiles.Orc" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(orc_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::ORC),
        can_rummage_corpses: tp::can_rummage_corpses,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        opposition_group: crate::cprops::opposition_group,
        is_enemy: crate::hand::orc_is_enemy,
        generate_loot: tp::generate_loot,
    }
    m {
        aggressive_action_criminal: crate::hand::orc_aggressive_action_criminal,
    }
}

fn orc_bomber_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ORC_BOMBER)
}
creature_leaf! {
    /// `Server.Mobiles.OrcBomber`.
    ORC_BOMBER, ORC_BOMBER_C = "Server.Mobiles.OrcBomber" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(orc_bomber_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::ORC),
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        opposition_group: crate::cprops::opposition_group,
        is_enemy: crate::hand::orc_bomber_is_enemy,
        on_action_combat: crate::hand::orc_bomber_on_action_combat,
        generate_loot: tp::generate_loot,
    }
    m {
        aggressive_action_criminal: crate::hand::orc_bomber_aggressive_action_criminal,
    }
}

fn orc_brute_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ORC_BRUTE)
}
creature_leaf! {
    /// `Server.Mobiles.OrcBrute`.
    ORC_BRUTE, ORC_BRUTE_C = "Server.Mobiles.OrcBrute" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(orc_brute_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bard_immune: |w, _id| !w.core_aos(),
        poison_immune: tp::poison_immune,
        meat: tp::meat,
        opposition_group: crate::cprops::opposition_group,
        can_rummage_corpses: tp::can_rummage_corpses,
        auto_dispel: crate::cprops::auto_dispel,
        is_enemy: crate::hand::orc_brute_is_enemy,
        on_damaged_by_spell: crate::hand::orc_brute_on_damaged_by_spell,
        generate_loot: tp::generate_loot,
    }
    m {
        aggressive_action_criminal: crate::hand::orc_brute_aggressive_action_criminal,
    }
}

fn orc_captain_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ORC_CAPTAIN)
}
creature_leaf! {
    /// `Server.Mobiles.OrcCaptain`.
    ORC_CAPTAIN, ORC_CAPTAIN_C = "Server.Mobiles.OrcCaptain" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(orc_captain_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::ORC),
        can_rummage_corpses: tp::can_rummage_corpses,
        meat: tp::meat,
        opposition_group: crate::cprops::opposition_group,
        is_enemy: crate::hand::orc_captain_is_enemy,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (BASE_CREATURE.on_death)(w, id, c);
            if rnd::double(w) < 0.05 {
                drop_new(w, c, "Server.Items.StoutWhip");
            }
        },
        aggressive_action_criminal: crate::hand::orc_captain_aggressive_action_criminal,
    }
}

fn orcish_lord_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ORCISH_LORD)
}
creature_leaf! {
    /// `Server.Mobiles.OrcishLord`.
    ORCISH_LORD, ORCISH_LORD_C = "Server.Mobiles.OrcishLord" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(orcish_lord_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::ORC),
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        opposition_group: crate::cprops::opposition_group,
        is_enemy: crate::hand::orcish_lord_is_enemy,
        generate_loot: tp::generate_loot,
    }
    m {
        aggressive_action_criminal: crate::hand::orcish_lord_aggressive_action_criminal,
    }
}

fn ratman_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RATMAN)
}
creature_leaf! {
    /// `Server.Mobiles.Ratman`.
    RATMAN, RATMAN_C = "Server.Mobiles.Ratman" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ratman_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::RATMAN),
        can_rummage_corpses: tp::can_rummage_corpses,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ratman_archer_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RATMAN_ARCHER)
}
creature_leaf! {
    /// `Server.Mobiles.RatmanArcher`.
    RATMAN_ARCHER, RATMAN_ARCHER_C = "Server.Mobiles.RatmanArcher" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ratman_archer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::RATMAN),
        can_rummage_corpses: tp::can_rummage_corpses,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn restless_soul_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RESTLESS_SOUL)
}
creature_leaf! {
    /// `Server.Mobiles.RestlessSoul`.
    RESTLESS_SOUL, RESTLESS_SOUL_C = "Server.Mobiles.RestlessSoul" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(restless_soul_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        always_attackable: crate::cprops::always_attackable,
        bleed_immune: tp::bleed_immune,
        is_enemy: crate::hand::restless_soul_is_enemy,
        generate_loot: tp::generate_loot,
    }
    m {
        display_paperdoll_to: crate::hand::restless_soul_display_paperdoll_to,
        get_context_menu_entries: crate::hand::restless_soul_get_context_menu_entries,
        get_idle_sound: |_w, _id| 0x107,
        get_anger_sound: |_w, _id| 0x1BF,
        get_death_sound: |_w, _id| 0xFD,
    }
}

fn rotting_corpse_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ROTTING_CORPSE)
}
creature_leaf! {
    /// `Server.Mobiles.RottingCorpse`.
    ROTTING_CORPSE, ROTTING_CORPSE_C = "Server.Mobiles.RottingCorpse" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(rotting_corpse_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        treasure_map_level: tp::treasure_map_level,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn savage_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SAVAGE)
}
creature_leaf! {
    /// `Server.Mobiles.Savage`.
    SAVAGE, SAVAGE_C = "Server.Mobiles.Savage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(savage_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        always_murderer: tp::always_murderer,
        opposition_group: crate::cprops::opposition_group,
        is_enemy: crate::hand::savage_is_enemy,
        alter_melee_damage_to: crate::hand::savage_alter_melee_damage_to,
        generate_loot: tp::generate_loot,
    }
    m {
        show_fame_title: |_w, _id| false,
        aggressive_action_criminal: crate::hand::savage_aggressive_action_criminal,
    }
}

fn savage_rider_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SAVAGE_RIDER)
}
creature_leaf! {
    /// `Server.Mobiles.SavageRider`.
    SAVAGE_RIDER, SAVAGE_RIDER_C = "Server.Mobiles.SavageRider" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(savage_rider_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        always_murderer: tp::always_murderer,
        opposition_group: crate::cprops::opposition_group,
        is_enemy: crate::hand::savage_rider_is_enemy,
        alter_melee_damage_to: crate::hand::savage_rider_alter_melee_damage_to,
        generate_loot: tp::generate_loot,
    }
    m {
        show_fame_title: |_w, _id| false,
        on_before_death: crate::hand::savage_rider_on_before_death,
        aggressive_action_criminal: crate::hand::savage_rider_aggressive_action_criminal,
    }
}

fn shadow_fiend_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SHADOW_FIEND)
}
creature_leaf! {
    /// `Server.Mobiles.ShadowFiend`.
    SHADOW_FIEND, SHADOW_FIEND_C = "Server.Mobiles.ShadowFiend" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(shadow_fiend_new), state crate::hand::shadow_fiend_serialize, crate::hand::shadow_fiend_deserialize;
    c {
        delete_corpse_on_death: crate::cprops::delete_corpse_on_death,
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
    }
    m {
        get_idle_sound: |_w, _id| 0x37A,
        get_anger_sound: |_w, _id| 0x379,
        get_death_sound: |_w, _id| 0x381,
        get_attack_sound: |_w, _id| 0x37F,
        get_hurt_sound: |_w, _id| 0x380,
        on_before_death: crate::hand::shadow_fiend_on_before_death,
        on_after_delete: crate::hand::shadow_fiend_on_after_delete,
    }
}

fn skeletal_knight_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SKELETAL_KNIGHT)
}
creature_leaf! {
    /// `Server.Mobiles.SkeletalKnight`.
    SKELETAL_KNIGHT, SKELETAL_KNIGHT_C = "Server.Mobiles.SkeletalKnight" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(skeletal_knight_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn skeleton_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SKELETON)
}
creature_leaf! {
    /// `Server.Mobiles.Skeleton`.
    SKELETON, SKELETON_C = "Server.Mobiles.Skeleton" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(skeleton_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn spawned_orcish_lord_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SPAWNED_ORCISH_LORD)
}
creature_leaf! {
    /// `Server.Mobiles.SpawnedOrcishLord`.
    SPAWNED_ORCISH_LORD, SPAWNED_ORCISH_LORD_C = "Server.Mobiles.SpawnedOrcishLord" : ORCISH_LORD / ORCISH_LORD_C, ctor Some(spawned_orcish_lord_new), state crate::hand::evil_mage_serialize, crate::hand::spawned_orcish_lord_deserialize;
    c {
        corpse_name: tp::corpse_name,
    }
    m {
        on_death: crate::hand::spawned_orcish_lord_on_death,
    }
}

fn spectral_armour_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SPECTRAL_ARMOUR)
}
creature_leaf! {
    /// `Server.Mobiles.SpectralArmour`.
    SPECTRAL_ARMOUR, SPECTRAL_ARMOUR_C = "Server.Mobiles.SpectralArmour" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(spectral_armour_new), encoded false;
    c {
        delete_corpse_on_death: crate::cprops::delete_corpse_on_death,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
    }
    m {
        get_idle_sound: |_w, _id| 0x200,
        get_anger_sound: |_w, _id| 0x56,
        on_before_death: crate::hand::spectral_armour_on_before_death,
    }
}

fn stone_gargoyle_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &STONE_GARGOYLE)
}
creature_leaf! {
    /// `Server.Mobiles.StoneGargoyle`.
    STONE_GARGOYLE, STONE_GARGOYLE_C = "Server.Mobiles.StoneGargoyle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(stone_gargoyle_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn strong_mongbat_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &STRONG_MONGBAT)
}
creature_leaf! {
    /// `Server.Mobiles.StrongMongbat`.
    STRONG_MONGBAT, STRONG_MONGBAT_C = "Server.Mobiles.StrongMongbat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(strong_mongbat_new), encoded false;
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

fn troll_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TROLL)
}
creature_leaf! {
    /// `Server.Mobiles.Troll`.
    TROLL, TROLL_C = "Server.Mobiles.Troll" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(troll_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn zombie_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ZOMBIE)
}
creature_leaf! {
    /// `Server.Mobiles.Zombie`.
    ZOMBIE, ZOMBIE_C = "Server.Mobiles.Zombie" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(zombie_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn exodus_minion_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &EXODUS_MINION)
}
creature_leaf! {
    /// `Server.Mobiles.ExodusMinion`.
    EXODUS_MINION, EXODUS_MINION_C = "Server.Mobiles.ExodusMinion" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(exodus_minion_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        is_scared_of_scary_things: crate::cprops::is_scared_of_scary_things,
        is_scary_to_pets: crate::cprops::is_scary_to_pets,
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        bard_immune: |w, _id| !w.core_aos(),
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
        get_monster_abilities: crate::hand::exodus_abilities,
    }
    m {
        get_idle_sound: |_w, _id| 0x218,
        get_anger_sound: |_w, _id| 0x26C,
        get_death_sound: |_w, _id| 0x211,
        get_attack_sound: |_w, _id| 0x232,
        get_hurt_sound: |_w, _id| 0x140,
    }
}

fn exodus_overseer_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &EXODUS_OVERSEER)
}
creature_leaf! {
    /// `Server.Mobiles.ExodusOverseer`.
    EXODUS_OVERSEER, EXODUS_OVERSEER_C = "Server.Mobiles.ExodusOverseer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(exodus_overseer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        is_scared_of_scary_things: crate::cprops::is_scared_of_scary_things,
        is_scary_to_pets: crate::cprops::is_scary_to_pets,
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        bard_immune: |w, _id| !w.core_aos(),
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
        get_monster_abilities: crate::hand::exodus_abilities,
    }
    m {
        get_idle_sound: |_w, _id| 0xFD,
        get_anger_sound: |_w, _id| 0x26C,
        get_death_sound: |_w, _id| 0x211,
        get_attack_sound: |_w, _id| 0x23B,
        get_hurt_sound: |_w, _id| 0x140,
    }
}

fn chaos_dragoon_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CHAOS_DRAGOON)
}
static CHAOS_DRAGOON_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.ChaosDragoon`.
    CHAOS_DRAGOON, CHAOS_DRAGOON_C = "Server.Mobiles.ChaosDragoon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(chaos_dragoon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        bard_immune: |w, _id| !w.core_aos(),
        can_rummage_corpses: tp::can_rummage_corpses,
        always_murderer: tp::always_murderer,
        alter_melee_damage_to: crate::hand::chaos_dragoon_alter_melee_damage_to,
        get_monster_abilities: |_, _| &CHAOS_DRAGOON_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        show_fame_title: |_w, _id| false,
        get_idle_sound: |_w, _id| 0x2CE,
        get_death_sound: |_w, _id| 0x2CC,
        get_hurt_sound: |_w, _id| 0x2D1,
        get_attack_sound: |_w, _id| 0x2C8,
        on_before_death: crate::hand::chaos_dragoon_on_before_death,
    }
}

fn chaos_dragoon_elite_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CHAOS_DRAGOON_ELITE)
}
static CHAOS_DRAGOON_ELITE_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.ChaosDragoonElite`.
    CHAOS_DRAGOON_ELITE, CHAOS_DRAGOON_ELITE_C = "Server.Mobiles.ChaosDragoonElite" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(chaos_dragoon_elite_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        bard_immune: |w, _id| !w.core_aos(),
        can_rummage_corpses: tp::can_rummage_corpses,
        always_murderer: tp::always_murderer,
        alter_melee_damage_to: crate::hand::chaos_dragoon_elite_alter_melee_damage_to,
        get_monster_abilities: |_, _| &CHAOS_DRAGOON_ELITE_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        show_fame_title: |_w, _id| false,
        get_idle_sound: |_w, _id| 0x2CE,
        get_death_sound: |_w, _id| 0x2CC,
        get_hurt_sound: |_w, _id| 0x2D1,
        get_attack_sound: |_w, _id| 0x2C8,
        on_before_death: crate::hand::chaos_dragoon_elite_on_before_death,
    }
}

fn juka_lord_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &JUKA_LORD)
}
creature_leaf! {
    /// `Server.Mobiles.JukaLord`.
    JUKA_LORD, JUKA_LORD_C = "Server.Mobiles.JukaLord" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(juka_lord_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        always_murderer: tp::always_murderer,
        bard_immune: |w, _id| !w.core_aos(),
        can_rummage_corpses: tp::can_rummage_corpses,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
        on_damage: crate::hand::juka_lord_on_damage,
        get_idle_sound: |_w, _id| 0x262,
        get_anger_sound: |_w, _id| 0x263,
        get_hurt_sound: |_w, _id| 0x1D0,
        get_death_sound: |_w, _id| 0x28D,
    }
}

fn juka_mage_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &JUKA_MAGE)
}
creature_leaf! {
    /// `Server.Mobiles.JukaMage`.
    JUKA_MAGE, JUKA_MAGE_C = "Server.Mobiles.JukaMage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(juka_mage_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        always_murderer: tp::always_murderer,
        can_rummage_corpses: tp::can_rummage_corpses,
        meat: tp::meat,
        on_think: crate::hand::juka_mage_on_think,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 0x1AC,
        get_anger_sound: |_w, _id| 0x1CD,
        get_hurt_sound: |_w, _id| 0x1D0,
        get_death_sound: |_w, _id| 0x28D,
    }
}

fn juka_warrior_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &JUKA_WARRIOR)
}
creature_leaf! {
    /// `Server.Mobiles.JukaWarrior`.
    JUKA_WARRIOR, JUKA_WARRIOR_C = "Server.Mobiles.JukaWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(juka_warrior_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        always_murderer: tp::always_murderer,
        can_rummage_corpses: tp::can_rummage_corpses,
        meat: tp::meat,
        on_gave_melee_attack: crate::hand::juka_warrior_on_gave_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 0x1AC,
        get_anger_sound: |_w, _id| 0x1CD,
        get_hurt_sound: |_w, _id| 0x1D0,
        get_death_sound: |_w, _id| 0x28D,
    }
}

fn meer_captain_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MEER_CAPTAIN)
}
creature_leaf! {
    /// `Server.Mobiles.MeerCaptain`.
    MEER_CAPTAIN, MEER_CAPTAIN_C = "Server.Mobiles.MeerCaptain" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(meer_captain_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bard_immune: |w, _id| !w.core_aos(),
        can_rummage_corpses: tp::can_rummage_corpses,
        initial_innocent: crate::cprops::initial_innocent,
        on_think: crate::hand::meer_captain_on_think,
        generate_loot: tp::generate_loot,
    }
    m {
        get_hurt_sound: |_w, _id| 0x14D,
        get_death_sound: |_w, _id| 0x314,
        get_attack_sound: |_w, _id| 0x75,
    }
}

fn meer_eternal_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MEER_ETERNAL)
}
creature_leaf! {
    /// `Server.Mobiles.MeerEternal`.
    MEER_ETERNAL, MEER_ETERNAL_C = "Server.Mobiles.MeerEternal" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(meer_eternal_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        bard_immune: |w, _id| !w.core_aos(),
        can_rummage_corpses: tp::can_rummage_corpses,
        poison_immune: tp::poison_immune,
        treasure_map_level: |w, _id| if w.core_aos() { 5 } else { 4 },
        initial_innocent: crate::cprops::initial_innocent,
        on_think: crate::hand::meer_eternal_on_think,
        generate_loot: tp::generate_loot,
    }
    m {
        get_hurt_sound: |_w, _id| 0x167,
        get_death_sound: |_w, _id| 0xBC,
        get_attack_sound: |_w, _id| 0x28B,
    }
}

fn meer_mage_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MEER_MAGE)
}
creature_leaf! {
    /// `Server.Mobiles.MeerMage`.
    MEER_MAGE, MEER_MAGE_C = "Server.Mobiles.MeerMage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(meer_mage_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        poison_immune: tp::poison_immune,
        can_rummage_corpses: tp::can_rummage_corpses,
        treasure_map_level: tp::treasure_map_level,
        initial_innocent: crate::cprops::initial_innocent,
        is_enemy: crate::hand::meer_mage_is_enemy,
        on_think: crate::hand::meer_mage_on_think,
        generate_loot: tp::generate_loot,
    }
    m {
        get_hurt_sound: |_w, _id| 0x14D,
        get_death_sound: |_w, _id| 0x314,
        get_attack_sound: |_w, _id| 0x75,
    }
}

fn meer_warrior_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MEER_WARRIOR)
}
creature_leaf! {
    /// `Server.Mobiles.MeerWarrior`.
    MEER_WARRIOR, MEER_WARRIOR_C = "Server.Mobiles.MeerWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(meer_warrior_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bard_immune: |w, _id| !w.core_aos(),
        can_rummage_corpses: tp::can_rummage_corpses,
        initial_innocent: crate::cprops::initial_innocent,
        generate_loot: tp::generate_loot,
    }
    m {
        on_damage: crate::hand::meer_warrior_on_damage,
        get_hurt_sound: |_w, _id| 0x156,
        get_death_sound: |_w, _id| 0x15C,
    }
}

fn cu_sidhe_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &CU_SIDHE)
}
creature_leaf! {
    /// `Server.Mobiles.CuSidhe`.
    CU_SIDHE, CU_SIDHE_C = "Server.Mobiles.CuSidhe" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(cu_sidhe_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_heal: crate::cprops::can_heal,
        can_heal_owner: crate::cprops::can_heal_owner,
        favorite_food: crate::cprops::favorite_food,
        can_anger_on_tame: crate::cprops::can_anger_on_tame,
        stat_loss_after_tame: crate::cprops::stat_loss_after_tame,
        hides: tp::hides,
        meat: tp::meat,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
        on_double_click: crate::hand::cu_sidhe_on_double_click,
        get_idle_sound: |_w, _id| 0x577,
        get_attack_sound: |_w, _id| 0x576,
        get_anger_sound: |_w, _id| 0x578,
        get_hurt_sound: |_w, _id| 0x576,
        get_death_sound: |_w, _id| 0x579,
    }
}

fn ferret_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &FERRET)
}
creature_leaf! {
    /// `Server.Mobiles.Ferret`.
    FERRET, FERRET_C = "Server.Mobiles.Ferret" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ferret_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
        on_movement: crate::hand::ferret_on_movement,
    }
}

fn raging_grizzly_bear_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RAGING_GRIZZLY_BEAR)
}
creature_leaf! {
    /// `Server.Mobiles.RagingGrizzlyBear`.
    RAGING_GRIZZLY_BEAR, RAGING_GRIZZLY_BEAR_C = "Server.Mobiles.RagingGrizzlyBear" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(raging_grizzly_bear_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        pack_instinct: crate::cprops::pack_instinct,
    }
    m {
    }
}

fn squirrel_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SQUIRREL)
}
creature_leaf! {
    /// `Server.Mobiles.Squirrel`.
    SQUIRREL, SQUIRREL_C = "Server.Mobiles.Squirrel" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(squirrel_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
    }
    m {
    }
}

fn lady_jennifyr_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LADY_JENNIFYR)
}
creature_leaf! {
    /// `Server.Mobiles.LadyJennifyr`.
    LADY_JENNIFYR, LADY_JENNIFYR_C = "Server.Mobiles.LadyJennifyr" : SKELETAL_KNIGHT / SKELETAL_KNIGHT_C, ctor Some(lady_jennifyr_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        generate_loot: tp::generate_loot,
        get_monster_abilities: crate::hand::lady_jennifyr_abilities,
    }
    m {
    }
}

fn master_jonath_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MASTER_JONATH)
}
creature_leaf! {
    /// `Server.Mobiles.MasterJonath`.
    MASTER_JONATH, MASTER_JONATH_C = "Server.Mobiles.MasterJonath" : BONE_MAGI / BONE_MAGI_C, ctor Some(master_jonath_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn master_mikael_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MASTER_MIKAEL)
}
creature_leaf! {
    /// `Server.Mobiles.MasterMikael`.
    MASTER_MIKAEL, MASTER_MIKAEL_C = "Server.Mobiles.MasterMikael" : BONE_MAGI / BONE_MAGI_C, ctor Some(master_mikael_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn master_theophilus_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MASTER_THEOPHILUS)
}
creature_leaf! {
    /// `Server.Mobiles.MasterTheophilus`.
    MASTER_THEOPHILUS, MASTER_THEOPHILUS_C = "Server.Mobiles.MasterTheophilus" : EVIL_MAGE_LORD / EVIL_MAGE_LORD_C, ctor Some(master_theophilus_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn red_death_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &RED_DEATH)
}
static RED_DEATH_ABILITIES: [&MonsterAbility; 1] = [&abilities::CHAOS_BREATH];
creature_leaf! {
    /// `Server.Mobiles.RedDeath`.
    RED_DEATH, RED_DEATH_C = "Server.Mobiles.RedDeath" : SKELETAL_MOUNT / SKELETAL_MOUNT_C, ctor Some(red_death_new), encoded false;
    c {
        default_name: tp::default_name,
        corpse_name: tp::corpse_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        always_murderer: tp::always_murderer,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        get_monster_abilities: |_, _| &RED_DEATH_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (SKELETAL_MOUNT.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.ResolvesBridle");
        },
    }
}

fn sir_patrick_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SIR_PATRICK)
}
creature_leaf! {
    /// `Server.Mobiles.SirPatrick`.
    SIR_PATRICK, SIR_PATRICK_C = "Server.Mobiles.SirPatrick" : SKELETAL_KNIGHT / SKELETAL_KNIGHT_C, ctor Some(sir_patrick_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        generate_loot: tp::generate_loot,
        get_monster_abilities: crate::hand::sir_patrick_abilities,
    }
    m {
    }
}

fn hydra_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &HYDRA)
}
static HYDRA_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.Hydra`.
    HYDRA, HYDRA_C = "Server.Mobiles.Hydra" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(hydra_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hides: tp::hides,
        meat: tp::meat,
        treasure_map_level: tp::treasure_map_level,
        get_monster_abilities: |_, _| &HYDRA_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (BASE_CREATURE.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.HydraScale");
        },
    }
}

fn abscess_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ABSCESS)
}
creature_leaf! {
    /// `Server.Mobiles.Abscess`.
    ABSCESS, ABSCESS_C = "Server.Mobiles.Abscess" : HYDRA / HYDRA_C, ctor Some(abscess_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (HYDRA.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.AbscessTail");
        },
    }
}

fn coil_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &COIL)
}
creature_leaf! {
    /// `Server.Mobiles.Coil`.
    COIL, COIL_C = "Server.Mobiles.Coil" : SILVER_SERPENT / SILVER_SERPENT_C, ctor Some(coil_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hit_poison: crate::cprops::hit_poison,
        poison_immune: tp::poison_immune,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        hides: tp::hides,
        meat: tp::meat,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (SILVER_SERPENT.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.CoilsFang");
        },
    }
}

fn satyr_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SATYR)
}
creature_leaf! {
    /// `Server.Mobiles.Satyr`.
    SATYR, SATYR_C = "Server.Mobiles.Satyr" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(satyr_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        opposition_group: crate::cprops::opposition_group,
        meat: tp::meat,
        on_think: crate::hand::satyr_on_think,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
    }
}

fn enslaved_satyr_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ENSLAVED_SATYR)
}
creature_leaf! {
    /// `Server.Mobiles.EnslavedSatyr`.
    ENSLAVED_SATYR, ENSLAVED_SATYR_C = "Server.Mobiles.EnslavedSatyr" : SATYR / SATYR_C, ctor Some(enslaved_satyr_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
    }
    m {
    }
}

fn ml_dryad_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ML_DRYAD)
}
creature_leaf! {
    /// `Server.Mobiles.MLDryad`.
    ML_DRYAD, ML_DRYAD_C = "Server.Mobiles.MLDryad" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ml_dryad_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        initial_innocent: crate::cprops::initial_innocent,
        opposition_group: crate::cprops::opposition_group,
        default_name: tp::default_name,
        meat: tp::meat,
        on_think: crate::hand::ml_dryad_on_think,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
    }
}

fn insane_dryad_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &INSANE_DRYAD)
}
creature_leaf! {
    /// `Server.Mobiles.InsaneDryad`.
    INSANE_DRYAD, INSANE_DRYAD_C = "Server.Mobiles.InsaneDryad" : ML_DRYAD / ML_DRYAD_C, ctor Some(insane_dryad_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        initial_innocent: crate::cprops::initial_innocent,
        default_name: tp::default_name,
    }
    m {
    }
}

fn harpy_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &HARPY)
}
creature_leaf! {
    /// `Server.Mobiles.Harpy`.
    HARPY, HARPY_C = "Server.Mobiles.Harpy" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(harpy_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        meat: tp::meat,
        meat_type: crate::cprops::meat_type,
        feathers: tp::feathers,
        can_fly: tp::can_fly,
        generate_loot: tp::generate_loot,
    }
    m {
        get_attack_sound: |_w, _id| 916,
        get_anger_sound: |_w, _id| 916,
        get_death_sound: |_w, _id| 917,
        get_hurt_sound: |_w, _id| 919,
        get_idle_sound: |_w, _id| 918,
    }
}

fn saliva_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SALIVA)
}
creature_leaf! {
    /// `Server.Mobiles.Saliva`.
    SALIVA, SALIVA_C = "Server.Mobiles.Saliva" : HARPY / HARPY_C, ctor Some(saliva_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (HARPY.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.SalivasFeather");
        },
    }
}

fn bog_thing_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BOG_THING)
}
creature_leaf! {
    /// `Server.Mobiles.BogThing`.
    BOG_THING, BOG_THING_C = "Server.Mobiles.BogThing" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bog_thing_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bard_immune: |w, _id| !w.core_aos(),
        poison_immune: tp::poison_immune,
        on_got_melee_attack: crate::hand::bog_thing_on_got_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn tangle_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TANGLE)
}
creature_leaf! {
    /// `Server.Mobiles.Tangle`.
    TANGLE, TANGLE_C = "Server.Mobiles.Tangle" : BOG_THING / BOG_THING_C, ctor Some(tangle_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (BOG_THING.on_death)(w, id, c);
            if rnd::double(w) < 0.3 {
                drop_new(w, c, "Server.Items.TaintedSeeds");
            }
        },
    }
}

fn thrasher_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &THRASHER)
}
creature_leaf! {
    /// `Server.Mobiles.Thrasher`.
    THRASHER, THRASHER_C = "Server.Mobiles.Thrasher" : ALLIGATOR / ALLIGATOR_C, ctor Some(thrasher_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        hides: tp::hides,
        meat: tp::meat,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (ALLIGATOR.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.ThrashersTail");
        },
    }
}

fn fetid_essence_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &FETID_ESSENCE)
}
creature_leaf! {
    /// `Server.Mobiles.FetidEssence`.
    FETID_ESSENCE, FETID_ESSENCE_C = "Server.Mobiles.FetidEssence" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(fetid_essence_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hit_poison: crate::cprops::hit_poison,
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x56d,
        get_idle_sound: |_w, _id| 0x56b,
        get_attack_sound: |_w, _id| 0x56c,
        get_hurt_sound: |_w, _id| 0x56c,
        get_death_sound: |_w, _id| 0x56e,
    }
}

fn interred_grizzle_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &INTERRED_GRIZZLE)
}
creature_leaf! {
    /// `Server.Mobiles.InterredGrizzle`.
    INTERRED_GRIZZLE, INTERRED_GRIZZLE_C = "Server.Mobiles.InterredGrizzle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(interred_grizzle_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x581,
        get_idle_sound: |_w, _id| 0x582,
        get_attack_sound: |_w, _id| 0x580,
        get_hurt_sound: |_w, _id| 0x583,
        get_death_sound: |_w, _id| 0x584,
        on_damage: crate::hand::interred_grizzle_on_damage,
    }
}

fn corrupted_soul_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CORRUPTED_SOUL)
}
creature_leaf! {
    /// `Server.Mobiles.CorruptedSoul`.
    CORRUPTED_SOUL, CORRUPTED_SOUL_C = "Server.Mobiles.CorruptedSoul" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(corrupted_soul_new), encoded false;
    c {
        delete_corpse_on_death: crate::cprops::delete_corpse_on_death,
        default_name: tp::default_name,
        always_attackable: crate::cprops::always_attackable,
        bleed_immune: tp::bleed_immune,
        always_murderer: tp::always_murderer,
    }
    m {
        get_attack_sound: |_w, _id| 0x233,
        on_before_death: crate::hand::corrupted_soul_on_before_death,
    }
}

fn feral_treefellow_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &FERAL_TREEFELLOW)
}
creature_leaf! {
    /// `Server.Mobiles.FeralTreefellow`.
    FERAL_TREEFELLOW, FERAL_TREEFELLOW_C = "Server.Mobiles.FeralTreefellow" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(feral_treefellow_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        opposition_group: crate::cprops::opposition_group,
        bleed_immune: tp::bleed_immune,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 443,
        get_death_sound: |_w, _id| 31,
        get_attack_sound: |_w, _id| 672,
    }
}

fn minotaur_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MINOTAUR)
}
creature_leaf! {
    /// `Server.Mobiles.Minotaur`.
    MINOTAUR, MINOTAUR_C = "Server.Mobiles.Minotaur" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(minotaur_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x597,
        get_idle_sound: |_w, _id| 0x596,
        get_attack_sound: |_w, _id| 0x599,
        get_hurt_sound: |_w, _id| 0x59a,
        get_death_sound: |_w, _id| 0x59c,
    }
}

fn minotaur_captain_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MINOTAUR_CAPTAIN)
}
creature_leaf! {
    /// `Server.Mobiles.MinotaurCaptain`.
    MINOTAUR_CAPTAIN, MINOTAUR_CAPTAIN_C = "Server.Mobiles.MinotaurCaptain" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(minotaur_captain_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x597,
        get_idle_sound: |_w, _id| 0x596,
        get_attack_sound: |_w, _id| 0x599,
        get_hurt_sound: |_w, _id| 0x59a,
        get_death_sound: |_w, _id| 0x59c,
    }
}

fn minotaur_scout_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MINOTAUR_SCOUT)
}
creature_leaf! {
    /// `Server.Mobiles.MinotaurScout`.
    MINOTAUR_SCOUT, MINOTAUR_SCOUT_C = "Server.Mobiles.MinotaurScout" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(minotaur_scout_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x597,
        get_idle_sound: |_w, _id| 0x596,
        get_attack_sound: |_w, _id| 0x599,
        get_hurt_sound: |_w, _id| 0x59a,
        get_death_sound: |_w, _id| 0x59c,
    }
}

fn pestilent_bandage_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PESTILENT_BANDAGE)
}
creature_leaf! {
    /// `Server.Mobiles.PestilentBandage`.
    PESTILENT_BANDAGE, PESTILENT_BANDAGE_C = "Server.Mobiles.PestilentBandage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(pestilent_bandage_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hit_poison: crate::cprops::hit_poison,
        can_heal: crate::cprops::can_heal,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn tormented_minotaur_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TORMENTED_MINOTAUR)
}
creature_leaf! {
    /// `Server.Mobiles.TormentedMinotaur`.
    TORMENTED_MINOTAUR, TORMENTED_MINOTAUR_C = "Server.Mobiles.TormentedMinotaur" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(tormented_minotaur_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
        get_death_sound: |_w, _id| 0x596,
        get_attack_sound: |_w, _id| 0x597,
        get_idle_sound: |_w, _id| 0x598,
        get_anger_sound: |_w, _id| 0x599,
        get_hurt_sound: |_w, _id| 0x59A,
    }
}

fn troglodyte_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TROGLODYTE)
}
creature_leaf! {
    /// `Server.Mobiles.Troglodyte`.
    TROGLODYTE, TROGLODYTE_C = "Server.Mobiles.Troglodyte" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(troglodyte_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_heal: crate::cprops::can_heal,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (BASE_CREATURE.on_death)(w, id, c);
            if rnd::double(w) < 0.1 {
                drop_new(w, c, "Server.Items.PrimitiveFetish");
            }
        },
    }
}

fn scorpion_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SCORPION)
}
creature_leaf! {
    /// `Server.Mobiles.Scorpion`.
    SCORPION, SCORPION_C = "Server.Mobiles.Scorpion" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(scorpion_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
        poison_immune: tp::poison_immune,
        hit_poison: |w, _id| if rnd::double(w) < 0.8 { Some(PoisonId(2)) } else { Some(PoisonId(3)) },
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn miasma_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MIASMA)
}
creature_leaf! {
    /// `Server.Mobiles.Miasma`.
    MIASMA, MIASMA_C = "Server.Mobiles.Miasma" : SCORPION / SCORPION_C, ctor Some(miasma_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        weapon_ability_chance: crate::cprops::weapon_ability_chance,
        hit_poison_chance: crate::cprops::hit_poison_chance,
        hit_poison: crate::cprops::hit_poison,
        has_mana_override: crate::cprops::has_mana_override,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn pyre_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PYRE)
}
creature_leaf! {
    /// `Server.Mobiles.Pyre`.
    PYRE, PYRE_C = "Server.Mobiles.Pyre" : PHOENIX / PHOENIX_C, ctor Some(pyre_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        treasure_map_level: tp::treasure_map_level,
        has_aura: crate::cprops::has_aura,
        get_weapon_ability: crate::hand::pyre_get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn reptalon_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &REPTALON)
}
static REPTALON_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.Reptalon`.
    REPTALON, REPTALON_C = "Server.Mobiles.Reptalon" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(reptalon_new), encoded false;
    c {
        default_name: tp::default_name,
        corpse_name: tp::corpse_name,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        can_anger_on_tame: crate::cprops::can_anger_on_tame,
        stat_loss_after_tame: crate::cprops::stat_loss_after_tame,
        favorite_food: crate::cprops::favorite_food,
        can_fly: tp::can_fly,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        get_monster_abilities: |_, _| &REPTALON_ABILITIES,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
    }
}

fn rend_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &REND)
}
creature_leaf! {
    /// `Server.Mobiles.Rend`.
    REND, REND_C = "Server.Mobiles.Rend" : REPTALON / REPTALON_C, ctor Some(rend_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        get_weapon_ability: crate::hand::rend_get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn greater_dragon_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GREATER_DRAGON)
}
static GREATER_DRAGON_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.GreaterDragon`.
    GREATER_DRAGON, GREATER_DRAGON_C = "Server.Mobiles.GreaterDragon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(greater_dragon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        stat_loss_after_tame: crate::cprops::stat_loss_after_tame,
        default_name: tp::default_name,
        reacquire_on_movement: |w, id| !read(w, id, false, |d| d.controlled),
        auto_dispel: |w, id| !read(w, id, false, |d| d.controlled),
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        scales: tp::scales,
        scale_type: crate::hand::greater_dragon_scale_type,
        favorite_food: crate::cprops::favorite_food,
        can_anger_on_tame: crate::cprops::can_anger_on_tame,
        can_fly: tp::can_fly,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        get_monster_abilities: |_, _| &GREATER_DRAGON_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn corrosive_slime_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CORROSIVE_SLIME)
}
creature_leaf! {
    /// `Server.Mobiles.CorrosiveSlime`.
    CORROSIVE_SLIME, CORROSIVE_SLIME_C = "Server.Mobiles.CorrosiveSlime" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(corrosive_slime_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        favorite_food: crate::cprops::favorite_food,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn grobu_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GROBU)
}
creature_leaf! {
    /// `Server.Mobiles.Grobu`.
    GROBU, GROBU_C = "Server.Mobiles.Grobu" : BLACK_BEAR / BLACK_BEAR_C, ctor Some(grobu_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (BLACK_BEAR.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.GrobusFur");
        },
    }
}

fn lurg_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LURG)
}
creature_leaf! {
    /// `Server.Mobiles.Lurg`.
    LURG, LURG_C = "Server.Mobiles.Lurg" : TROGLODYTE / TROGLODYTE_C, ctor Some(lurg_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn putrefier_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PUTREFIER)
}
creature_leaf! {
    /// `Server.Mobiles.Putrefier`.
    PUTREFIER, PUTREFIER_C = "Server.Mobiles.Putrefier" : BALRON / BALRON_C, ctor Some(putrefier_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        hit_poison: crate::cprops::hit_poison,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn corporeal_brume_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CORPOREAL_BRUME)
}
creature_leaf! {
    /// `Server.Mobiles.CorporealBrume`.
    CORPOREAL_BRUME, CORPOREAL_BRUME_C = "Server.Mobiles.CorporealBrume" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(corporeal_brume_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        has_aura: crate::hand::corporeal_brume_has_aura,
        aura_interval: crate::cprops::aura_interval,
        aura_range: crate::cprops::aura_range,
        aura_base_damage: |w, _id| rnd::min_max(w, 25, 35),
        aura_fire_damage: crate::cprops::aura_fire_damage,
        aura_cold_damage: crate::cprops::aura_cold_damage,
        aura_effect: crate::hand::corporeal_brume_aura_effect,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn crystal_daemon_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CRYSTAL_DAEMON)
}
creature_leaf! {
    /// `Server.Mobiles.CrystalDaemon`.
    CRYSTAL_DAEMON, CRYSTAL_DAEMON_C = "Server.Mobiles.CrystalDaemon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(crystal_daemon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn crystal_lattice_seeker_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CRYSTAL_LATTICE_SEEKER)
}
creature_leaf! {
    /// `Server.Mobiles.CrystalLatticeSeeker`.
    CRYSTAL_LATTICE_SEEKER, CRYSTAL_LATTICE_SEEKER_C = "Server.Mobiles.CrystalLatticeSeeker" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(crystal_lattice_seeker_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        feathers: tp::feathers,
        treasure_map_level: tp::treasure_map_level,
        on_gave_melee_attack: crate::hand::crystal_lattice_seeker_on_gave_melee_attack,
        on_got_melee_attack: crate::hand::crystal_lattice_seeker_on_got_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
        get_attack_sound: |_w, _id| 0x2F6,
        get_death_sound: |_w, _id| 0x2F7,
        get_anger_sound: |_w, _id| 0x2F8,
        get_hurt_sound: |_w, _id| 0x2F9,
        get_idle_sound: |_w, _id| 0x2FA,
    }
}

fn sea_serpent_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SEA_SERPENT)
}
static SEA_SERPENT_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.SeaSerpent`.
    SEA_SERPENT, SEA_SERPENT_C = "Server.Mobiles.SeaSerpent" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(sea_serpent_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        scales: tp::scales,
        scale_type: crate::cprops::scale_type,
        get_monster_abilities: |_, _| &SEA_SERPENT_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn crystal_sea_serpent_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CRYSTAL_SEA_SERPENT)
}
creature_leaf! {
    /// `Server.Mobiles.CrystalSeaSerpent`.
    CRYSTAL_SEA_SERPENT, CRYSTAL_SEA_SERPENT_C = "Server.Mobiles.CrystalSeaSerpent" : SEA_SERPENT / SEA_SERPENT_C, ctor Some(crystal_sea_serpent_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
    }
    m {
    }
}

fn crystal_vortex_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CRYSTAL_VORTEX)
}
creature_leaf! {
    /// `Server.Mobiles.CrystalVortex`.
    CRYSTAL_VORTEX, CRYSTAL_VORTEX_C = "Server.Mobiles.CrystalVortex" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(crystal_vortex_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x15,
        get_attack_sound: |_w, _id| 0x28,
    }
}

fn wisp_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &WISP)
}
creature_leaf! {
    /// `Server.Mobiles.Wisp`.
    WISP, WISP_C = "Server.Mobiles.Wisp" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(wisp_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::WISP),
        faction_allegiance: crate::hand::wisp_faction_allegiance,
        ethic_allegiance: crate::hand::wisp_ethic_allegiance,
        reacquire_delay: crate::cprops::reacquire_delay,
        default_name: tp::default_name,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn crystal_wisp_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CRYSTAL_WISP)
}
creature_leaf! {
    /// `Server.Mobiles.CrystalWisp`.
    CRYSTAL_WISP, CRYSTAL_WISP_C = "Server.Mobiles.CrystalWisp" : WISP / WISP_C, ctor Some(crystal_wisp_new), encoded false;
    c {
        default_name: tp::default_name,
    }
    m {
    }
}

fn mantra_effervescence_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MANTRA_EFFERVESCENCE)
}
creature_leaf! {
    /// `Server.Mobiles.MantraEffervescence`.
    MANTRA_EFFERVESCENCE, MANTRA_EFFERVESCENCE_C = "Server.Mobiles.MantraEffervescence" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(mantra_effervescence_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn protector_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PROTECTOR)
}
creature_leaf! {
    /// `Server.Mobiles.Protector`.
    PROTECTOR, PROTECTOR_C = "Server.Mobiles.Protector" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(protector_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        always_murderer: tp::always_murderer,
        generate_loot: tp::generate_loot,
        generate_loot_spawning: crate::hand::protector_generate_loot_spawning,
    }
    m {
        property_title: |_w, _id| false,
        show_fame_title: |_w, _id| false,
    }
}

fn unfrozen_mummy_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &UNFROZEN_MUMMY)
}
creature_leaf! {
    /// `Server.Mobiles.UnfrozenMummy`.
    UNFROZEN_MUMMY, UNFROZEN_MUMMY_C = "Server.Mobiles.UnfrozenMummy" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(unfrozen_mummy_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn chiikkaha_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CHIIKKAHA)
}
creature_leaf! {
    /// `Server.Mobiles.Chiikkaha`.
    CHIIKKAHA, CHIIKKAHA_C = "Server.Mobiles.Chiikkaha" : RATMAN_MAGE / RATMAN_MAGE_C, ctor Some(chiikkaha_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
    }
    m {
    }
}

fn moug_guur_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MOUG_GUUR)
}
creature_leaf! {
    /// `Server.Mobiles.MougGuur`.
    MOUG_GUUR, MOUG_GUUR_C = "Server.Mobiles.MougGuur" : ETTIN / ETTIN_C, ctor Some(moug_guur_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
    }
    m {
    }
}

fn szavetra_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SZAVETRA)
}
creature_leaf! {
    /// `Server.Mobiles.Szavetra`.
    SZAVETRA, SZAVETRA_C = "Server.Mobiles.Szavetra" : SUCCUBUS / SUCCUBUS_C, ctor Some(szavetra_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
    }
    m {
    }
}

fn changeling_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CHANGELING)
}
creature_leaf! {
    /// `Server.Mobiles.Changeling`.
    CHANGELING, CHANGELING_C = "Server.Mobiles.Changeling" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(changeling_new), state crate::hand::changeling_serialize, crate::hand::changeling_deserialize;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        initial_innocent: crate::hand::changeling_initial_innocent,
        on_think: crate::hand::changeling_on_think,
        check_idle: crate::hand::changeling_check_idle,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
        show_fame_title: |_w, _id| false,
        get_anger_sound: |_w, _id| 0x46E,
        get_idle_sound: |_w, _id| 0x470,
        get_attack_sound: |_w, _id| 0x46D,
        get_hurt_sound: |_w, _id| 0x471,
        get_death_sound: |_w, _id| 0x46F,
        on_after_delete: crate::hand::changeling_on_after_delete,
        clear_hands: crate::hand::changeling_clear_hands,
    }
}

fn gnaw_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GNAW)
}
creature_leaf! {
    /// `Server.Mobiles.Gnaw`.
    GNAW, GNAW_C = "Server.Mobiles.Gnaw" : DIRE_WOLF / DIRE_WOLF_C, ctor Some(gnaw_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        hides: tp::hides,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn guile_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GUILE)
}
static GUILE_ABILITIES: [&MonsterAbility; 1] = [&abilities::DRAIN_LIFE_ATTACK];
creature_leaf! {
    /// `Server.Mobiles.Guile`.
    GUILE, GUILE_C = "Server.Mobiles.Guile" : CHANGELING / CHANGELING_C, ctor Some(guile_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        default_hue: crate::cprops::default_hue,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        get_monster_abilities: |_, _| &GUILE_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn irk_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &IRK)
}
creature_leaf! {
    /// `Server.Mobiles.Irk`.
    IRK, IRK_C = "Server.Mobiles.Irk" : CHANGELING / CHANGELING_C, ctor Some(irk_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        default_hue: crate::cprops::default_hue,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn lady_lissith_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LADY_LISSITH)
}
creature_leaf! {
    /// `Server.Mobiles.LadyLissith`.
    LADY_LISSITH, LADY_LISSITH_C = "Server.Mobiles.LadyLissith" : GIANT_BLACK_WIDOW / GIANT_BLACK_WIDOW_C, ctor Some(lady_lissith_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn lady_sabrix_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LADY_SABRIX)
}
creature_leaf! {
    /// `Server.Mobiles.LadySabrix`.
    LADY_SABRIX, LADY_SABRIX_C = "Server.Mobiles.LadySabrix" : GIANT_BLACK_WIDOW / GIANT_BLACK_WIDOW_C, ctor Some(lady_sabrix_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn malefic_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MALEFIC)
}
creature_leaf! {
    /// `Server.Mobiles.Malefic`.
    MALEFIC, MALEFIC_C = "Server.Mobiles.Malefic" : DREAD_SPIDER / DREAD_SPIDER_C, ctor Some(malefic_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn silk_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SILK)
}
creature_leaf! {
    /// `Server.Mobiles.Silk`.
    SILK, SILK_C = "Server.Mobiles.Silk" : GIANT_BLACK_WIDOW / GIANT_BLACK_WIDOW_C, ctor Some(silk_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn spite_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SPITE)
}
creature_leaf! {
    /// `Server.Mobiles.Spite`.
    SPITE, SPITE_C = "Server.Mobiles.Spite" : CHANGELING / CHANGELING_C, ctor Some(spite_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        default_hue: crate::cprops::default_hue,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn swoop_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SWOOP)
}
creature_leaf! {
    /// `Server.Mobiles.Swoop`.
    SWOOP, SWOOP_C = "Server.Mobiles.Swoop" : EAGLE / EAGLE_C, ctor Some(swoop_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_fly: tp::can_fly,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        feathers: tp::feathers,
        on_gave_melee_attack: crate::hand::swoop_on_gave_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn virulent_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &VIRULENT)
}
creature_leaf! {
    /// `Server.Mobiles.Virulent`.
    VIRULENT, VIRULENT_C = "Server.Mobiles.Virulent" : DREAD_SPIDER / DREAD_SPIDER_C, ctor Some(virulent_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn hell_hound_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &HELL_HOUND)
}
static HELL_HOUND_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.HellHound`.
    HELL_HOUND, HELL_HOUND_C = "Server.Mobiles.HellHound" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(hell_hound_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
        pack_instinct: crate::cprops::pack_instinct,
        get_monster_abilities: |_, _| &HELL_HOUND_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn vorpal_bunny_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &VORPAL_BUNNY)
}
creature_leaf! {
    /// `Server.Mobiles.VorpalBunny`.
    VORPAL_BUNNY, VORPAL_BUNNY_C = "Server.Mobiles.VorpalBunny" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(vorpal_bunny_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        bard_immune: |w, _id| !w.core_aos(),
        generate_loot: tp::generate_loot,
    }
    m {
        get_attack_sound: |_w, _id| 0xC9,
        get_hurt_sound: |_w, _id| 0xCA,
        get_death_sound: |_w, _id| 0xCB,
    }
}

fn dark_wisp_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DARK_WISP)
}
creature_leaf! {
    /// `Server.Mobiles.DarkWisp`.
    DARK_WISP, DARK_WISP_C = "Server.Mobiles.DarkWisp" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dark_wisp_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::WISP),
        ethic_allegiance: crate::hand::dark_wisp_ethic_allegiance,
        reacquire_delay: crate::cprops::reacquire_delay,
        default_name: tp::default_name,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ethereal_warrior_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ETHEREAL_WARRIOR)
}
creature_leaf! {
    /// `Server.Mobiles.EtherealWarrior`.
    ETHEREAL_WARRIOR, ETHEREAL_WARRIOR_C = "Server.Mobiles.EtherealWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ethereal_warrior_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        initial_innocent: crate::cprops::initial_innocent,
        treasure_map_level: |w, _id| if w.core_aos() { 5 } else { 0 },
        opposition_group: crate::cprops::opposition_group,
        feathers: tp::feathers,
        on_gave_melee_attack: crate::hand::ethereal_warrior_on_gave_melee_attack,
        on_got_melee_attack: crate::hand::ethereal_warrior_on_got_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
        on_movement: crate::hand::ethereal_warrior_on_movement,
        get_anger_sound: |_w, _id| 0x2F8,
        get_idle_sound: |_w, _id| 0x2F8,
        get_attack_sound: |w, _id| rnd::from(w, 0x2F5, 2),
        get_hurt_sound: |_w, _id| 0x2F9,
        get_death_sound: |_w, _id| 0x2F7,
    }
}

fn pixie_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PIXIE)
}
creature_leaf! {
    /// `Server.Mobiles.Pixie`.
    PIXIE, PIXIE_C = "Server.Mobiles.Pixie" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(pixie_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        initial_innocent: crate::cprops::initial_innocent,
        hide_type: crate::cprops::hide_type,
        hides: tp::hides,
        meat: tp::meat,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (BASE_CREATURE.on_death)(w, id, c);
            if rnd::double(w) < 0.35 {
                drop_new(w, c, "Server.Items.PixieLeg");
            }
        },
    }
}

fn shadow_wisp_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SHADOW_WISP)
}
creature_leaf! {
    /// `Server.Mobiles.ShadowWisp`.
    SHADOW_WISP, SHADOW_WISP_C = "Server.Mobiles.ShadowWisp" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(shadow_wisp_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        opposition_group: crate::cprops::opposition_group,
    }
    m {
    }
}

fn blade_spirits_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BLADE_SPIRITS)
}
creature_leaf! {
    /// `Server.Mobiles.BladeSpirits`.
    BLADE_SPIRITS, BLADE_SPIRITS_C = "Server.Mobiles.BladeSpirits" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(blade_spirits_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        delete_corpse_on_death: |w, _id| w.core_aos(),
        is_house_summonable: crate::cprops::is_house_summonable,
        dispel_difficulty: crate::cprops::dispel_difficulty,
        dispel_focus: crate::cprops::dispel_focus,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        follows_acquire_rules: crate::hand::blade_spirits_follows_acquire_rules,
        get_fight_mode_ranking: crate::hand::blade_spirits_get_fight_mode_ranking,
        on_think: crate::hand::blade_spirits_on_think,
    }
    m {
        get_anger_sound: |_w, _id| 0x23A,
        get_attack_sound: |_w, _id| 0x3B8,
        get_hurt_sound: |_w, _id| 0x23A,
    }
}

fn centaur_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CENTAUR)
}
creature_leaf! {
    /// `Server.Mobiles.Centaur`.
    CENTAUR, CENTAUR_C = "Server.Mobiles.Centaur" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(centaur_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        opposition_group: crate::cprops::opposition_group,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn energy_vortex_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ENERGY_VORTEX)
}
creature_leaf! {
    /// `Server.Mobiles.EnergyVortex`.
    ENERGY_VORTEX, ENERGY_VORTEX_C = "Server.Mobiles.EnergyVortex" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(energy_vortex_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        delete_corpse_on_death: |w, id| read(w, id, false, |d| d.summoned),
        always_murderer: tp::always_murderer,
        dispel_difficulty: crate::cprops::dispel_difficulty,
        dispel_focus: crate::cprops::dispel_focus,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        follows_acquire_rules: crate::hand::energy_vortex_follows_acquire_rules,
        get_fight_mode_ranking: crate::hand::energy_vortex_get_fight_mode_ranking,
        on_think: crate::hand::energy_vortex_on_think,
    }
    m {
        get_anger_sound: |_w, _id| 0x15,
        get_attack_sound: |_w, _id| 0x28,
    }
}

fn frost_ooze_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &FROST_OOZE)
}
creature_leaf! {
    /// `Server.Mobiles.FrostOoze`.
    FROST_OOZE, FROST_OOZE_C = "Server.Mobiles.FrostOoze" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(frost_ooze_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
    }
}

fn plague_beast_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PLAGUE_BEAST)
}
creature_leaf! {
    /// `Server.Mobiles.PlagueBeast`.
    PLAGUE_BEAST, PLAGUE_BEAST_C = "Server.Mobiles.PlagueBeast" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(plague_beast_new), state crate::hand::plague_beast_serialize, crate::hand::plague_beast_deserialize;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        poison_immune: tp::poison_immune,
        on_gave_melee_attack: crate::hand::plague_beast_on_gave_melee_attack,
        on_damaged_by_spell: crate::hand::plague_beast_on_damaged_by_spell,
        on_got_melee_attack: crate::hand::plague_beast_on_got_melee_attack,
        on_think: crate::hand::plague_beast_on_think,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 0x1BF,
        get_attack_sound: |_w, _id| 0x1C0,
        get_hurt_sound: |_w, _id| 0x1C1,
        get_death_sound: |_w, _id| 0x1C2,
    }
}

fn plague_beast_lord_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &PLAGUE_BEAST_LORD)
}
creature_leaf! {
    /// `Server.Mobiles.PlagueBeastLord`.
    PLAGUE_BEAST_LORD, PLAGUE_BEAST_LORD_C = "Server.Mobiles.PlagueBeastLord" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(plague_beast_lord_new), state crate::hand::plague_beast_lord_serialize, crate::hand::plague_beast_lord_deserialize;
    c {
        corpse_name: tp::corpse_name,
        poison_immune: tp::poison_immune,
        default_name: tp::default_name,
    }
    m {
        on_double_click: crate::hand::plague_beast_lord_on_double_click,
        on_drag_drop: crate::hand::plague_beast_lord_on_drag_drop,
        on_death: crate::hand::plague_beast_lord_on_death,
        on_delete: crate::hand::plague_beast_lord_on_delete,
        on_movement: crate::hand::plague_beast_lord_on_movement,
        check_nonlocal_lift: crate::hand::plague_beast_lord_check_nonlocal_lift,
        check_nonlocal_drop: crate::hand::plague_beast_lord_check_nonlocal_drop,
        is_snoop: crate::hand::plague_beast_lord_is_snoop,
        get_idle_sound: |_w, _id| 0x1BF,
        get_attack_sound: |_w, _id| 0x1C0,
        get_hurt_sound: |_w, _id| 0x1C1,
        get_death_sound: |_w, _id| 0x1C2,
    }
}

fn sand_vortex_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SAND_VORTEX)
}
creature_leaf! {
    /// `Server.Mobiles.SandVortex`.
    SAND_VORTEX, SAND_VORTEX_C = "Server.Mobiles.SandVortex" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(sand_vortex_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        on_action_combat: crate::hand::sand_vortex_on_action_combat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn slime_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SLIME)
}
creature_leaf! {
    /// `Server.Mobiles.Slime`.
    SLIME, SLIME_C = "Server.Mobiles.Slime" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(slime_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        favorite_food: crate::cprops::favorite_food,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn agapite_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &AGAPITE_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.AgapiteElemental`.
    AGAPITE_ELEMENTAL, AGAPITE_ELEMENTAL_C = "Server.Mobiles.AgapiteElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(agapite_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        auto_dispel: crate::cprops::auto_dispel,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn bronze_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BRONZE_ELEMENTAL)
}
static BRONZE_ELEMENTAL_ABILITIES: [&MonsterAbility; 1] = [&abilities::POISON_GAS_COUNTER];
creature_leaf! {
    /// `Server.Mobiles.BronzeElemental`.
    BRONZE_ELEMENTAL, BRONZE_ELEMENTAL_C = "Server.Mobiles.BronzeElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bronze_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        auto_dispel: crate::cprops::auto_dispel,
        treasure_map_level: tp::treasure_map_level,
        get_monster_abilities: |_, _| &BRONZE_ELEMENTAL_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn copper_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &COPPER_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.CopperElemental`.
    COPPER_ELEMENTAL, COPPER_ELEMENTAL_C = "Server.Mobiles.CopperElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(copper_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        auto_dispel: crate::cprops::auto_dispel,
        treasure_map_level: tp::treasure_map_level,
        alter_melee_damage_from: crate::hand::copper_elemental_alter_melee_damage_from,
        check_reflect: crate::hand::copper_elemental_check_reflect,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn dull_copper_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DULL_COPPER_ELEMENTAL)
}
static DULL_COPPER_ELEMENTAL_ABILITIES: [&MonsterAbility; 1] = [&abilities::DEATH_EXPLOSION];
creature_leaf! {
    /// `Server.Mobiles.DullCopperElemental`.
    DULL_COPPER_ELEMENTAL, DULL_COPPER_ELEMENTAL_C = "Server.Mobiles.DullCopperElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dull_copper_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        get_monster_abilities: |_, _| &DULL_COPPER_ELEMENTAL_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn golden_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &GOLDEN_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.GoldenElemental`.
    GOLDEN_ELEMENTAL, GOLDEN_ELEMENTAL_C = "Server.Mobiles.GoldenElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(golden_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn shadow_iron_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SHADOW_IRON_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.ShadowIronElemental`.
    SHADOW_IRON_ELEMENTAL, SHADOW_IRON_ELEMENTAL_C = "Server.Mobiles.ShadowIronElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(shadow_iron_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        poison_immune: tp::poison_immune,
        breath_immune: crate::cprops::breath_immune,
        alter_melee_damage_from: crate::hand::shadow_iron_elemental_alter_melee_damage_from,
        alter_damage_scalar_from: crate::hand::shadow_iron_elemental_alter_damage_scalar_from,
        alter_spell_damage_from: crate::hand::shadow_iron_elemental_alter_spell_damage_from,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn valorite_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &VALORITE_ELEMENTAL)
}
static VALORITE_ELEMENTAL_ABILITIES: [&MonsterAbility; 2] = [
    &abilities::POISON_GAS_COUNTER,
    &abilities::DESTROY_EQUIPMENT,
];
creature_leaf! {
    /// `Server.Mobiles.ValoriteElemental`.
    VALORITE_ELEMENTAL, VALORITE_ELEMENTAL_C = "Server.Mobiles.ValoriteElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(valorite_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        alter_melee_damage_from: crate::hand::valorite_elemental_alter_melee_damage_from,
        check_reflect: crate::hand::valorite_elemental_check_reflect,
        get_monster_abilities: |_, _| &VALORITE_ELEMENTAL_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn verite_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &VERITE_ELEMENTAL)
}
static VERITE_ELEMENTAL_ABILITIES: [&MonsterAbility; 1] = [&abilities::DESTROY_EQUIPMENT];
creature_leaf! {
    /// `Server.Mobiles.VeriteElemental`.
    VERITE_ELEMENTAL, VERITE_ELEMENTAL_C = "Server.Mobiles.VeriteElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(verite_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: crate::cprops::auto_dispel,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        get_monster_abilities: |_, _| &VERITE_ELEMENTAL_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn reaper_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &REAPER)
}
creature_leaf! {
    /// `Server.Mobiles.Reaper`.
    REAPER, REAPER_C = "Server.Mobiles.Reaper" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(reaper_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        disallow_all_moves: crate::cprops::disallow_all_moves,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn bogling_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BOGLING)
}
creature_leaf! {
    /// `Server.Mobiles.Bogling`.
    BOGLING, BOGLING_C = "Server.Mobiles.Bogling" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bogling_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hides: tp::hides,
        meat: tp::meat,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn corpser_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &CORPSER)
}
creature_leaf! {
    /// `Server.Mobiles.Corpser`.
    CORPSER, CORPSER_C = "Server.Mobiles.Corpser" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(corpser_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        disallow_all_moves: crate::cprops::disallow_all_moves,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn quagmire_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &QUAGMIRE)
}
creature_leaf! {
    /// `Server.Mobiles.Quagmire`.
    QUAGMIRE, QUAGMIRE_C = "Server.Mobiles.Quagmire" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(quagmire_new), state crate::hand::evil_mage_serialize, crate::hand::quagmire_deserialize;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        hit_poison_chance: crate::cprops::hit_poison_chance,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 353,
    }
}

fn swamp_tentacle_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SWAMP_TENTACLE)
}
creature_leaf! {
    /// `Server.Mobiles.SwampTentacle`.
    SWAMP_TENTACLE, SWAMP_TENTACLE_C = "Server.Mobiles.SwampTentacle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(swamp_tentacle_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn whipping_vine_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &WHIPPING_VINE)
}
creature_leaf! {
    /// `Server.Mobiles.WhippingVine`.
    WHIPPING_VINE, WHIPPING_VINE_C = "Server.Mobiles.WhippingVine" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(whipping_vine_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bard_immune: |w, _id| !w.core_aos(),
        poison_immune: tp::poison_immune,
    }
    m {
    }
}

fn ancient_wyrm_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ANCIENT_WYRM)
}
static ANCIENT_WYRM_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.AncientWyrm`.
    ANCIENT_WYRM, ANCIENT_WYRM_C = "Server.Mobiles.AncientWyrm" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ancient_wyrm_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: crate::cprops::reacquire_on_movement,
        auto_dispel: crate::cprops::auto_dispel,
        hide_type: crate::cprops::hide_type,
        hides: tp::hides,
        meat: tp::meat,
        scales: tp::scales,
        scale_type: crate::hand::ancient_wyrm_scale_type,
        poison_immune: tp::poison_immune,
        hit_poison: |w, _id| if rnd::bool(w) { Some(PoisonId(0)) } else { Some(PoisonId(1)) },
        treasure_map_level: tp::treasure_map_level,
        can_fly: tp::can_fly,
        get_monster_abilities: |_, _| &ANCIENT_WYRM_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 0x2D3,
        get_hurt_sound: |_w, _id| 0x2D1,
    }
}

fn deep_sea_serpent_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DEEP_SEA_SERPENT)
}
static DEEP_SEA_SERPENT_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.DeepSeaSerpent`.
    DEEP_SEA_SERPENT, DEEP_SEA_SERPENT_C = "Server.Mobiles.DeepSeaSerpent" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(deep_sea_serpent_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        scales: tp::scales,
        scale_type: crate::cprops::scale_type,
        get_monster_abilities: |_, _| &DEEP_SEA_SERPENT_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn dragon_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DRAGON)
}
static DRAGON_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.Dragon`.
    DRAGON, DRAGON_C = "Server.Mobiles.Dragon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dragon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: |w, id| !read(w, id, false, |d| d.controlled),
        auto_dispel: |w, id| !read(w, id, false, |d| d.controlled),
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        scales: tp::scales,
        scale_type: crate::hand::dragon_scale_type,
        favorite_food: crate::cprops::favorite_food,
        can_anger_on_tame: crate::cprops::can_anger_on_tame,
        can_fly: tp::can_fly,
        get_monster_abilities: |_, _| &DRAGON_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ophidian_archmage_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &OPHIDIAN_ARCHMAGE)
}
creature_leaf! {
    /// `Server.Mobiles.OphidianArchmage`.
    OPHIDIAN_ARCHMAGE, OPHIDIAN_ARCHMAGE_C = "Server.Mobiles.OphidianArchmage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ophidian_archmage_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ophidian_mage_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &OPHIDIAN_MAGE)
}
creature_leaf! {
    /// `Server.Mobiles.OphidianMage`.
    OPHIDIAN_MAGE, OPHIDIAN_MAGE_C = "Server.Mobiles.OphidianMage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ophidian_mage_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        treasure_map_level: tp::treasure_map_level,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ophidian_matriarch_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &OPHIDIAN_MATRIARCH)
}
creature_leaf! {
    /// `Server.Mobiles.OphidianMatriarch`.
    OPHIDIAN_MATRIARCH, OPHIDIAN_MATRIARCH_C = "Server.Mobiles.OphidianMatriarch" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ophidian_matriarch_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn serpentine_dragon_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SERPENTINE_DRAGON)
}
static SERPENTINE_DRAGON_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.SerpentineDragon`.
    SERPENTINE_DRAGON, SERPENTINE_DRAGON_C = "Server.Mobiles.SerpentineDragon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(serpentine_dragon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: crate::cprops::reacquire_on_movement,
        bonus_pet_damage_scalar: |w, _id| if w.core_se() { 3.0 } else { 1.0 },
        auto_dispel: crate::cprops::auto_dispel,
        hide_type: crate::cprops::hide_type,
        hides: tp::hides,
        meat: tp::meat,
        scales: tp::scales,
        scale_type: |w, _id| if rnd::bool(w) { ScaleType::Black } else { ScaleType::White },
        treasure_map_level: tp::treasure_map_level,
        on_got_melee_attack: crate::hand::serpentine_dragon_on_got_melee_attack,
        get_monster_abilities: |_, _| &SERPENTINE_DRAGON_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 0x2C4,
        get_attack_sound: |_w, _id| 0x2C0,
        get_death_sound: |_w, _id| 0x2C1,
        get_anger_sound: |_w, _id| 0x2C4,
        get_hurt_sound: |_w, _id| 0x2C3,
    }
}

fn shadow_wyrm_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SHADOW_WYRM)
}
static SHADOW_WYRM_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.ShadowWyrm`.
    SHADOW_WYRM, SHADOW_WYRM_C = "Server.Mobiles.ShadowWyrm" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(shadow_wyrm_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: crate::cprops::reacquire_on_movement,
        auto_dispel: crate::cprops::auto_dispel,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        scales: tp::scales,
        scale_type: crate::cprops::scale_type,
        hide_type: crate::cprops::hide_type,
        can_fly: tp::can_fly,
        get_monster_abilities: |_, _| &SHADOW_WYRM_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 0x2D5,
        get_hurt_sound: |_w, _id| 0x2D1,
    }
}

fn skeletal_dragon_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SKELETAL_DRAGON)
}
static SKELETAL_DRAGON_ABILITIES: [&MonsterAbility; 1] = [&abilities::COLD_BREATH];
creature_leaf! {
    /// `Server.Mobiles.SkeletalDragon`.
    SKELETAL_DRAGON, SKELETAL_DRAGON_C = "Server.Mobiles.SkeletalDragon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(skeletal_dragon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: crate::cprops::reacquire_on_movement,
        bonus_pet_damage_scalar: |w, _id| if w.core_se() { 3.0 } else { 1.0 },
        auto_dispel: crate::cprops::auto_dispel,
        poison_immune: tp::poison_immune,
        bleed_immune: tp::bleed_immune,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        get_monster_abilities: |_, _| &SKELETAL_DRAGON_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn white_wyrm_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &WHITE_WYRM)
}
creature_leaf! {
    /// `Server.Mobiles.WhiteWyrm`.
    WHITE_WYRM, WHITE_WYRM_C = "Server.Mobiles.WhiteWyrm" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(white_wyrm_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: crate::cprops::reacquire_on_movement,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        scales: tp::scales,
        scale_type: crate::cprops::scale_type,
        favorite_food: crate::cprops::favorite_food,
        can_anger_on_tame: crate::cprops::can_anger_on_tame,
        can_fly: tp::can_fly,
        generate_loot: crate::cprops::generate_loot,
    }
    m {
    }
}

fn drake_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DRAKE)
}
static DRAKE_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.Drake`.
    DRAKE, DRAKE_C = "Server.Mobiles.Drake" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(drake_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: crate::cprops::reacquire_on_movement,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        scales: tp::scales,
        scale_type: crate::hand::drake_scale_type,
        favorite_food: crate::cprops::favorite_food,
        can_fly: tp::can_fly,
        get_monster_abilities: |_, _| &DRAKE_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn kraken_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &KRAKEN)
}
creature_leaf! {
    /// `Server.Mobiles.Kraken`.
    KRAKEN, KRAKEN_C = "Server.Mobiles.Kraken" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(kraken_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
        on_before_death: crate::hand::kraken_on_before_death,
    }
}

fn lizardman_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LIZARDMAN)
}
creature_leaf! {
    /// `Server.Mobiles.Lizardman`.
    LIZARDMAN, LIZARDMAN_C = "Server.Mobiles.Lizardman" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lizardman_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::LIZARDMAN),
        can_rummage_corpses: tp::can_rummage_corpses,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ophidian_knight_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &OPHIDIAN_KNIGHT)
}
creature_leaf! {
    /// `Server.Mobiles.OphidianKnight`.
    OPHIDIAN_KNIGHT, OPHIDIAN_KNIGHT_C = "Server.Mobiles.OphidianKnight" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ophidian_knight_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        treasure_map_level: tp::treasure_map_level,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ophidian_warrior_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &OPHIDIAN_WARRIOR)
}
creature_leaf! {
    /// `Server.Mobiles.OphidianWarrior`.
    OPHIDIAN_WARRIOR, OPHIDIAN_WARRIOR_C = "Server.Mobiles.OphidianWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ophidian_warrior_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        meat: tp::meat,
        treasure_map_level: tp::treasure_map_level,
        opposition_group: crate::cprops::opposition_group,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn stone_harpy_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &STONE_HARPY)
}
creature_leaf! {
    /// `Server.Mobiles.StoneHarpy`.
    STONE_HARPY, STONE_HARPY_C = "Server.Mobiles.StoneHarpy" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(stone_harpy_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        feathers: tp::feathers,
        can_fly: tp::can_fly,
        generate_loot: tp::generate_loot,
    }
    m {
        get_attack_sound: |_w, _id| 916,
        get_anger_sound: |_w, _id| 916,
        get_death_sound: |_w, _id| 917,
        get_hurt_sound: |_w, _id| 919,
        get_idle_sound: |_w, _id| 918,
    }
}

fn wyvern_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &WYVERN)
}
creature_leaf! {
    /// `Server.Mobiles.Wyvern`.
    WYVERN, WYVERN_C = "Server.Mobiles.Wyvern" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(wyvern_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: crate::cprops::reacquire_on_movement,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        can_fly: tp::can_fly,
        generate_loot: tp::generate_loot,
    }
    m {
        get_attack_sound: |_w, _id| 713,
        get_anger_sound: |_w, _id| 718,
        get_death_sound: |_w, _id| 716,
        get_hurt_sound: |_w, _id| 721,
        get_idle_sound: |_w, _id| 725,
    }
}

fn raptor_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RAPTOR)
}
creature_leaf! {
    /// `Server.Mobiles.Raptor`.
    RAPTOR, RAPTOR_C = "Server.Mobiles.Raptor" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(raptor_new), encoded true;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        pack_instinct: crate::cprops::pack_instinct,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 1573,
        get_anger_sound: |_w, _id| 1570,
        get_hurt_sound: |_w, _id| 1572,
        get_death_sound: |_w, _id| 1571,
        on_death: |w, id, c| {
            (BASE_CREATURE.on_death)(w, id, c);
            if !read(w, id, false, |d| d.controlled) && rnd::double(w) <= 0.005 {
                drop_new(w, c, "Server.Items.RaptorClaw");
            }
        },
    }
}

fn slith_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SLITH)
}
static SLITH_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.Slith`.
    SLITH, SLITH_C = "Server.Mobiles.Slith" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(slith_new), encoded true;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        get_monster_abilities: |_, _| &SLITH_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn stone_slith_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &STONE_SLITH)
}
static STONE_SLITH_ABILITIES: [&MonsterAbility; 2] =
    [&abilities::GRASPING_CLAW, &abilities::TAIL_SWIPE];
creature_leaf! {
    /// `Server.Mobiles.StoneSlith`.
    STONE_SLITH, STONE_SLITH_C = "Server.Mobiles.StoneSlith" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(stone_slith_new), encoded true;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        get_monster_abilities: |_, _| &STONE_SLITH_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (BASE_CREATURE.on_death)(w, id, c);
            if !read(w, id, false, |d| d.controlled) && rnd::double(w) <= 0.005 {
                drop_new(w, c, "Server.Items.StoneSlithClaw");
            }
        },
    }
}

fn toxic_slith_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TOXIC_SLITH)
}
static TOXIC_SLITH_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.ToxicSlith`.
    TOXIC_SLITH, TOXIC_SLITH_C = "Server.Mobiles.ToxicSlith" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(toxic_slith_new), encoded true;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        get_monster_abilities: |_, _| &TOXIC_SLITH_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn bake_kitsune_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &BAKE_KITSUNE)
}
static BAKE_KITSUNE_ABILITIES: [&MonsterAbility; 1] = [&abilities::BLOOD_BATH_ATTACK];
creature_leaf! {
    /// `Server.Mobiles.BakeKitsune`.
    BAKE_KITSUNE, BAKE_KITSUNE_C = "Server.Mobiles.BakeKitsune" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bake_kitsune_new), state crate::hand::bake_kitsune_serialize, crate::hand::bake_kitsune_deserialize;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: crate::cprops::hide_type,
        favorite_food: crate::cprops::favorite_food,
        get_monster_abilities: |_, _| &BAKE_KITSUNE_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        show_fame_title: |_w, _id| false,
        click_title: |_w, _id| false,
        property_title: |_w, _id| false,
        on_combatant_change: crate::hand::bake_kitsune_on_combatant_change,
        on_before_death: crate::hand::bake_kitsune_on_before_death,
        get_anger_sound: |_w, _id| 0x4DE,
        get_idle_sound: |_w, _id| 0x4DD,
        get_attack_sound: |_w, _id| 0x4DC,
        get_hurt_sound: |_w, _id| 0x4DF,
        get_death_sound: |_w, _id| 0x4DB,
    }
}

fn deathwatch_beetle_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DEATHWATCH_BEETLE)
}
creature_leaf! {
    /// `Server.Mobiles.DeathwatchBeetle`.
    DEATHWATCH_BEETLE, DEATHWATCH_BEETLE_C = "Server.Mobiles.DeathwatchBeetle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(deathwatch_beetle_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hides: tp::hides,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        alter_melee_damage_to: crate::hand::deathwatch_beetle_alter_melee_damage_to,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x4F3,
        get_idle_sound: |_w, _id| 0x4F2,
        get_attack_sound: |_w, _id| 0x4F1,
        get_hurt_sound: |_w, _id| 0x4F4,
        get_death_sound: |_w, _id| 0x4F0,
        on_damage: crate::hand::deathwatch_beetle_on_damage,
    }
}

fn deathwatch_beetle_hatchling_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &DEATHWATCH_BEETLE_HATCHLING)
}
creature_leaf! {
    /// `Server.Mobiles.DeathwatchBeetleHatchling`.
    DEATHWATCH_BEETLE_HATCHLING, DEATHWATCH_BEETLE_HATCHLING_C = "Server.Mobiles.DeathwatchBeetleHatchling" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(deathwatch_beetle_hatchling_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hides: tp::hides,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x4F3,
        get_idle_sound: |_w, _id| 0x4F2,
        get_attack_sound: |_w, _id| 0x4F1,
        get_hurt_sound: |_w, _id| 0x4F4,
        get_death_sound: |_w, _id| 0x4F0,
    }
}

fn elite_ninja_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ELITE_NINJA)
}
creature_leaf! {
    /// `Server.Mobiles.EliteNinja`.
    ELITE_NINJA, ELITE_NINJA_C = "Server.Mobiles.EliteNinja" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(elite_ninja_new), encoded false;
    c {
        default_name: tp::default_name,
        bard_immune: crate::cprops::bard_immune,
        always_murderer: tp::always_murderer,
        generate_loot: tp::generate_loot,
    }
    m {
        click_title: |_w, _id| false,
        on_death: |w, id, c| {
            (BASE_CREATURE.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.BookOfNinjitsu");
        },
    }
}

fn fan_dancer_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &FAN_DANCER)
}
static FAN_DANCER_ABILITIES: [&MonsterAbility; 3] = [
    &abilities::REFLECT_PHYSICAL_DAMAGE,
    &abilities::FANNING_FIRE,
    &abilities::FAN_THROW_COUNTER,
];
creature_leaf! {
    /// `Server.Mobiles.FanDancer`.
    FAN_DANCER, FAN_DANCER_C = "Server.Mobiles.FanDancer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(fan_dancer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        uncalmable: crate::cprops::uncalmable,
        get_monster_abilities: |_, _| &FAN_DANCER_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn fire_beetle_new(w: &mut World) -> MobileId {
    crate::cprops::new_mount(w, &FIRE_BEETLE)
}
creature_leaf! {
    /// `Server.Mobiles.FireBeetle`.
    FIRE_BEETLE, FIRE_BEETLE_C = "Server.Mobiles.FireBeetle" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(fire_beetle_new), state crate::hand::fire_beetle_serialize, crate::hand::fire_beetle_deserialize;
    c {
        default_name: tp::default_name,
        corpse_name: tp::corpse_name,
        subdue_before_tame: crate::cprops::subdue_before_tame,
        stat_loss_after_tame: crate::cprops::stat_loss_after_tame,
        reduce_speed_with_damage: crate::cprops::reduce_speed_with_damage,
        meat: tp::meat,
        favorite_food: crate::cprops::favorite_food,
        on_harmful_spell: crate::hand::fire_beetle_on_harmful_spell,
        override_bonding_reqs: crate::cprops::override_bonding_reqs,
        get_control_chance: crate::hand::fire_beetle_get_control_chance,
    }
    m {
        on_combatant_change: crate::hand::fire_beetle_on_combatant_change,
        get_anger_sound: |_w, _id| 0x21D,
        get_idle_sound: |_w, _id| 0x21D,
        get_attack_sound: |_w, _id| 0x162,
        get_hurt_sound: |_w, _id| 0x163,
        get_death_sound: |_w, _id| 0x21D,
    }
}

fn kappa_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &KAPPA)
}
static KAPPA_ABILITIES: [&MonsterAbility; 1] = [&abilities::DRAIN_LIFE_ATTACK];
creature_leaf! {
    /// `Server.Mobiles.Kappa`.
    KAPPA, KAPPA_C = "Server.Mobiles.Kappa" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(kappa_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        new_harmful_item: crate::hand::kappa_new_harmful_item,
        get_monster_abilities: |_, _| &KAPPA_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x50B,
        get_idle_sound: |_w, _id| 0x50A,
        get_attack_sound: |_w, _id| 0x509,
        get_hurt_sound: |_w, _id| 0x50C,
        get_death_sound: |_w, _id| 0x508,
        on_damage: crate::hand::kappa_on_damage,
    }
}

fn kaze_kemono_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &KAZE_KEMONO)
}
creature_leaf! {
    /// `Server.Mobiles.KazeKemono`.
    KAZE_KEMONO, KAZE_KEMONO_C = "Server.Mobiles.KazeKemono" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(kaze_kemono_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        on_gave_melee_attack: crate::hand::kaze_kemono_on_gave_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn lady_of_the_snow_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &LADY_OF_THE_SNOW)
}
creature_leaf! {
    /// `Server.Mobiles.LadyOfTheSnow`.
    LADY_OF_THE_SNOW, LADY_OF_THE_SNOW_C = "Server.Mobiles.LadyOfTheSnow" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lady_of_the_snow_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        can_rummage_corpses: tp::can_rummage_corpses,
        treasure_map_level: tp::treasure_map_level,
        on_gave_melee_attack: crate::hand::lady_of_the_snow_on_gave_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
        get_death_sound: |_w, _id| 0x370,
    }
}

fn oni_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ONI)
}
creature_leaf! {
    /// `Server.Mobiles.Oni`.
    ONI, ONI_C = "Server.Mobiles.Oni" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(oni_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x4E3,
        get_idle_sound: |_w, _id| 0x4E2,
        get_attack_sound: |_w, _id| 0x4E1,
        get_hurt_sound: |_w, _id| 0x4E4,
        get_death_sound: |_w, _id| 0x4E0,
    }
}

fn rai_ju_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RAI_JU)
}
creature_leaf! {
    /// `Server.Mobiles.RaiJu`.
    RAI_JU, RAI_JU_C = "Server.Mobiles.RaiJu" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(rai_ju_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        on_gave_melee_attack: crate::hand::rai_ju_on_gave_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn revenant_lion_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &REVENANT_LION)
}
creature_leaf! {
    /// `Server.Mobiles.RevenantLion`.
    REVENANT_LION, REVENANT_LION_C = "Server.Mobiles.RevenantLion" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(revenant_lion_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x518,
        get_idle_sound: |_w, _id| 0x517,
        get_attack_sound: |_w, _id| 0x516,
        get_hurt_sound: |_w, _id| 0x519,
        get_death_sound: |_w, _id| 0x515,
    }
}

fn ronin_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RONIN)
}
creature_leaf! {
    /// `Server.Mobiles.Ronin`.
    RONIN, RONIN_C = "Server.Mobiles.Ronin" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ronin_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        always_murderer: tp::always_murderer,
        bard_immune: crate::cprops::bard_immune,
        can_rummage_corpses: tp::can_rummage_corpses,
        generate_loot: tp::generate_loot,
    }
    m {
        click_title: |_w, _id| false,
        on_death: |w, id, c| {
            (BASE_CREATURE.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.BookOfBushido");
        },
    }
}

fn rune_beetle_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &RUNE_BEETLE)
}
static RUNE_BEETLE_ABILITIES: [&MonsterAbility; 1] = [&abilities::RUNE_CORRUPTION];
creature_leaf! {
    /// `Server.Mobiles.RuneBeetle`.
    RUNE_BEETLE, RUNE_BEETLE_C = "Server.Mobiles.RuneBeetle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(rune_beetle_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        hit_poison: crate::cprops::hit_poison,
        favorite_food: crate::cprops::favorite_food,
        can_anger_on_tame: crate::cprops::can_anger_on_tame,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        get_monster_abilities: |_, _| &RUNE_BEETLE_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x4E8,
        get_idle_sound: |_w, _id| 0x4E7,
        get_attack_sound: |_w, _id| 0x4E6,
        get_hurt_sound: |_w, _id| 0x4E9,
        get_death_sound: |_w, _id| 0x4E5,
    }
}

fn tsuki_wolf_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &TSUKI_WOLF)
}
static TSUKI_WOLF_ABILITIES: [&MonsterAbility; 1] = [&abilities::BLOOD_BATH_ATTACK];
creature_leaf! {
    /// `Server.Mobiles.TsukiWolf`.
    TSUKI_WOLF, TSUKI_WOLF_C = "Server.Mobiles.TsukiWolf" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(tsuki_wolf_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: crate::cprops::favorite_food,
        get_monster_abilities: |_, _| &TSUKI_WOLF_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 0x52D,
        get_idle_sound: |_w, _id| 0x52C,
        get_attack_sound: |_w, _id| 0x52B,
        get_hurt_sound: |_w, _id| 0x52E,
        get_death_sound: |_w, _id| 0x52A,
    }
}

fn yamandon_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &YAMANDON)
}
creature_leaf! {
    /// `Server.Mobiles.Yamandon`.
    YAMANDON, YAMANDON_C = "Server.Mobiles.Yamandon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(yamandon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: crate::cprops::reacquire_on_movement,
        poison_immune: tp::poison_immune,
        hit_poison: |w, _id| if rnd::bool(w) { Some(PoisonId(3)) } else { Some(PoisonId(4)) },
        treasure_map_level: tp::treasure_map_level,
        hides: tp::hides,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        on_damaged_by_spell: crate::hand::yamandon_on_damaged_by_spell,
        on_got_melee_attack: crate::hand::yamandon_on_got_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
        get_attack_sound: |_w, _id| 1260,
        get_anger_sound: |_w, _id| 1262,
        get_death_sound: |_w, _id| 1259,
        get_hurt_sound: |_w, _id| 1263,
        get_idle_sound: |_w, _id| 1261,
    }
}

fn yomotsu_elder_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &YOMOTSU_ELDER)
}
creature_leaf! {
    /// `Server.Mobiles.YomotsuElder`.
    YOMOTSU_ELDER, YOMOTSU_ELDER_C = "Server.Mobiles.YomotsuElder" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(yomotsu_elder_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        favorite_food: crate::cprops::favorite_food,
        meat: tp::meat,
        can_rummage_corpses: tp::can_rummage_corpses,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        on_gave_melee_attack: crate::hand::yomotsu_elder_on_gave_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 0x42A,
        get_attack_sound: |_w, _id| 0x435,
        get_hurt_sound: |_w, _id| 0x436,
        get_death_sound: |_w, _id| 0x43A,
    }
}

fn yomotsu_priest_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &YOMOTSU_PRIEST)
}
creature_leaf! {
    /// `Server.Mobiles.YomotsuPriest`.
    YOMOTSU_PRIEST, YOMOTSU_PRIEST_C = "Server.Mobiles.YomotsuPriest" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(yomotsu_priest_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        favorite_food: crate::cprops::favorite_food,
        meat: tp::meat,
        can_rummage_corpses: tp::can_rummage_corpses,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        on_gave_melee_attack: crate::hand::yomotsu_priest_on_gave_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 0x42A,
        get_attack_sound: |_w, _id| 0x435,
        get_hurt_sound: |_w, _id| 0x436,
        get_death_sound: |_w, _id| 0x43A,
    }
}

fn yomotsu_warrior_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &YOMOTSU_WARRIOR)
}
creature_leaf! {
    /// `Server.Mobiles.YomotsuWarrior`.
    YOMOTSU_WARRIOR, YOMOTSU_WARRIOR_C = "Server.Mobiles.YomotsuWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(yomotsu_warrior_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        favorite_food: crate::cprops::favorite_food,
        meat: tp::meat,
        can_rummage_corpses: tp::can_rummage_corpses,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        on_gave_melee_attack: crate::hand::yomotsu_warrior_on_gave_melee_attack,
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 0x42A,
        get_attack_sound: |_w, _id| 0x435,
        get_hurt_sound: |_w, _id| 0x436,
        get_death_sound: |_w, _id| 0x43A,
    }
}

fn summoned_air_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SUMMONED_AIR_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.SummonedAirElemental`.
    SUMMONED_AIR_ELEMENTAL, SUMMONED_AIR_ELEMENTAL_C = "Server.Mobiles.SummonedAirElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(summoned_air_elemental_new), state crate::hand::evil_mage_serialize, crate::hand::summoned_air_elemental_deserialize;
    c {
        delete_corpse_on_death: |w, id| read(w, id, false, |d| d.summoned),
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        dispel_difficulty: crate::cprops::dispel_difficulty,
        dispel_focus: crate::cprops::dispel_focus,
    }
    m {
    }
}

fn summoned_daemon_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SUMMONED_DAEMON)
}
creature_leaf! {
    /// `Server.Mobiles.SummonedDaemon`.
    SUMMONED_DAEMON, SUMMONED_DAEMON_C = "Server.Mobiles.SummonedDaemon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(summoned_daemon_new), encoded false;
    c {
        delete_corpse_on_death: |w, id| read(w, id, false, |d| d.summoned),
        corpse_name: tp::corpse_name,
        dispel_difficulty: crate::cprops::dispel_difficulty,
        dispel_focus: crate::cprops::dispel_focus,
        poison_immune: tp::poison_immune,
        can_fly: tp::can_fly,
    }
    m {
    }
}

fn summoned_earth_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SUMMONED_EARTH_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.SummonedEarthElemental`.
    SUMMONED_EARTH_ELEMENTAL, SUMMONED_EARTH_ELEMENTAL_C = "Server.Mobiles.SummonedEarthElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(summoned_earth_elemental_new), encoded false;
    c {
        delete_corpse_on_death: |w, id| read(w, id, false, |d| d.summoned),
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        dispel_difficulty: crate::cprops::dispel_difficulty,
        dispel_focus: crate::cprops::dispel_focus,
    }
    m {
    }
}

fn summoned_fire_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SUMMONED_FIRE_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.SummonedFireElemental`.
    SUMMONED_FIRE_ELEMENTAL, SUMMONED_FIRE_ELEMENTAL_C = "Server.Mobiles.SummonedFireElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(summoned_fire_elemental_new), encoded false;
    c {
        delete_corpse_on_death: |w, id| read(w, id, false, |d| d.summoned),
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        dispel_difficulty: crate::cprops::dispel_difficulty,
        dispel_focus: crate::cprops::dispel_focus,
    }
    m {
    }
}

fn summoned_water_elemental_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &SUMMONED_WATER_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.SummonedWaterElemental`.
    SUMMONED_WATER_ELEMENTAL, SUMMONED_WATER_ELEMENTAL_C = "Server.Mobiles.SummonedWaterElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(summoned_water_elemental_new), encoded false;
    c {
        delete_corpse_on_death: |w, id| read(w, id, false, |d| d.summoned),
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        dispel_difficulty: crate::cprops::dispel_difficulty,
        dispel_focus: crate::cprops::dispel_focus,
    }
    m {
    }
}

/// Every class of this file.
pub static ALL: &[(&MobileType, &[&str])] = &[
    (&ABYSMAL_HORROR, &[]),
    (&BONE_DEMON, &[]),
    (&CRYSTAL_ELEMENTAL, &[]),
    (&DARKNIGHT_CREEPER, &[]),
    (&DEVOURER, &[]),
    (&FLESH_GOLEM, &[]),
    (&FLESH_RENDERER, &[]),
    (&GIBBERLING, &[]),
    (&GORE_FIEND, &[]),
    (&IMPALER, &[]),
    (&MOUND_OF_MAGGOTS, &[]),
    (&PATCHWORK_SKELETON, &[]),
    (&RAVAGER, &[]),
    (&SHADOW_KNIGHT, &[]),
    (&SKITTERING_HOPPER, &[]),
    (&TREEFELLOW, &[]),
    (&VAMPIRE_BAT, &[]),
    (&WAILING_BANSHEE, &[]),
    (&WANDERER_OF_THE_VOID, &[]),
    (&ANT_LION, &[]),
    (&BLACK_SOLEN_INFILTRATOR_QUEEN, &[]),
    (&BLACK_SOLEN_INFILTRATOR_WARRIOR, &[]),
    (&BLACK_SOLEN_QUEEN, &[]),
    (&BLACK_SOLEN_WARRIOR, &[]),
    (&BLACK_SOLEN_WORKER, &[]),
    (&RED_SOLEN_INFILTRATOR_QUEEN, &[]),
    (&RED_SOLEN_INFILTRATOR_WARRIOR, &[]),
    (&RED_SOLEN_QUEEN, &[]),
    (&RED_SOLEN_WARRIOR, &[]),
    (&RED_SOLEN_WORKER, &[]),
    (&DREAD_SPIDER, &[]),
    (&TERATHAN_AVENGER, &[]),
    (&TERATHAN_MATRIARCH, &[]),
    (&FROST_SPIDER, &[]),
    (&GIANT_BLACK_WIDOW, &[]),
    (&GIANT_SPIDER, &[]),
    (&TERATHAN_DRONE, &[]),
    (&TERATHAN_WARRIOR, &[]),
    (&ACID_ELEMENTAL, &["Server.Mobiles.ToxicElemental"]),
    (&AIR_ELEMENTAL, &[]),
    (&BLOOD_ELEMENTAL, &[]),
    (&EFREET, &[]),
    (&FIRE_ELEMENTAL, &[]),
    (&ICE_ELEMENTAL, &[]),
    (&POISON_ELEMENTAL, &[]),
    (&WATER_ELEMENTAL, &[]),
    (&EARTH_ELEMENTAL, &[]),
    (&SNOW_ELEMENTAL, &[]),
    (&ANCIENT_LICH, &[]),
    (&ARCANE_DAEMON, &[]),
    (&BALRON, &[]),
    (&BETRAYER, &[]),
    (&BOGLE, &[]),
    (&BONE_MAGI, &["Server.Mobiles.BoneMage"]),
    (&DAEMON, &[]),
    (&ELDER_GAZER, &[]),
    (&EVIL_MAGE, &[]),
    (&EVIL_MAGE_LORD, &[]),
    (&FIRE_GARGOYLE, &[]),
    (&GARGOYLE, &[]),
    (&GARGOYLE_DESTROYER, &[]),
    (&GARGOYLE_ENFORCER, &[]),
    (&GAZER, &[]),
    (&GOLEM_CONTROLLER, &[]),
    (&ICE_FIEND, &[]),
    (&IMP, &[]),
    (&LICH, &[]),
    (&LICH_LORD, &[]),
    (&ORCISH_MAGE, &[]),
    (&RATMAN_MAGE, &[]),
    (&SAVAGE_SHAMAN, &[]),
    (&SHADE, &[]),
    (&SKELETAL_MAGE, &[]),
    (&SPECTRE, &[]),
    (&SPELLBINDER, &[]),
    (&SUCCUBUS, &[]),
    (&TITAN, &[]),
    (&WRAITH, &[]),
    (&ARCTIC_OGRE_LORD, &["Server.Mobiles.ArticOgreLord"]),
    (&BONE_KNIGHT, &[]),
    (&BRIGAND, &[]),
    (&CHAOS_DAEMON, &[]),
    (&CURSED, &[]),
    (&CYCLOPS, &[]),
    (&DOPPLEGANGER, &[]),
    (&ELF_BRIGAND, &[]),
    (&ENSLAVED_GARGOYLE, &[]),
    (&ETTIN, &[]),
    (&EXECUTIONER, &[]),
    (&FROST_TROLL, &[]),
    (&GAZER_LARVA, &[]),
    (&GHOUL, &[]),
    (&GREATER_MONGBAT, &[]),
    (&GUARDIAN, &[]),
    (&HEADLESS_ONE, &[]),
    (&HORDE_MINION, &[]),
    (&JUGGERNAUT, &[]),
    (&KHALDUN_SUMMONER, &[]),
    (&KHALDUN_ZEALOT, &[]),
    (&MOLOCH, &[]),
    (&MONGBAT, &[]),
    (&MUMMY, &[]),
    (&OGRE, &[]),
    (&OGRE_LORD, &[]),
    (&ORC, &[]),
    (&ORC_BOMBER, &[]),
    (&ORC_BRUTE, &[]),
    (&ORC_CAPTAIN, &[]),
    (&ORCISH_LORD, &[]),
    (&RATMAN, &[]),
    (&RATMAN_ARCHER, &[]),
    (&RESTLESS_SOUL, &[]),
    (&ROTTING_CORPSE, &[]),
    (&SAVAGE, &[]),
    (&SAVAGE_RIDER, &[]),
    (&SHADOW_FIEND, &[]),
    (&SKELETAL_KNIGHT, &[]),
    (&SKELETON, &[]),
    (&SPAWNED_ORCISH_LORD, &[]),
    (&SPECTRAL_ARMOUR, &[]),
    (&STONE_GARGOYLE, &[]),
    (&STRONG_MONGBAT, &[]),
    (&TROLL, &[]),
    (&ZOMBIE, &[]),
    (&EXODUS_MINION, &[]),
    (&EXODUS_OVERSEER, &[]),
    (&CHAOS_DRAGOON, &[]),
    (&CHAOS_DRAGOON_ELITE, &[]),
    (&JUKA_LORD, &[]),
    (&JUKA_MAGE, &[]),
    (&JUKA_WARRIOR, &[]),
    (&MEER_CAPTAIN, &[]),
    (&MEER_ETERNAL, &[]),
    (&MEER_MAGE, &[]),
    (&MEER_WARRIOR, &[]),
    (&CU_SIDHE, &[]),
    (&FERRET, &[]),
    (&RAGING_GRIZZLY_BEAR, &[]),
    (&SQUIRREL, &[]),
    (&LADY_JENNIFYR, &[]),
    (&MASTER_JONATH, &[]),
    (&MASTER_MIKAEL, &[]),
    (&MASTER_THEOPHILUS, &[]),
    (&RED_DEATH, &[]),
    (&SIR_PATRICK, &[]),
    (&HYDRA, &[]),
    (&ABSCESS, &[]),
    (&COIL, &[]),
    (&SATYR, &[]),
    (&ENSLAVED_SATYR, &[]),
    (&ML_DRYAD, &[]),
    (&INSANE_DRYAD, &[]),
    (&HARPY, &[]),
    (&SALIVA, &[]),
    (&BOG_THING, &[]),
    (&TANGLE, &[]),
    (&THRASHER, &[]),
    (&FETID_ESSENCE, &[]),
    (&INTERRED_GRIZZLE, &[]),
    (&CORRUPTED_SOUL, &[]),
    (&FERAL_TREEFELLOW, &["Server.Mobiles.FerelTreefellow"]),
    (&MINOTAUR, &[]),
    (&MINOTAUR_CAPTAIN, &[]),
    (&MINOTAUR_SCOUT, &[]),
    (&PESTILENT_BANDAGE, &[]),
    (&TORMENTED_MINOTAUR, &[]),
    (&TROGLODYTE, &[]),
    (&SCORPION, &[]),
    (&MIASMA, &[]),
    (&PYRE, &[]),
    (&REPTALON, &[]),
    (&REND, &[]),
    (&GREATER_DRAGON, &[]),
    (&CORROSIVE_SLIME, &[]),
    (&GROBU, &[]),
    (&LURG, &[]),
    (&PUTREFIER, &[]),
    (&CORPOREAL_BRUME, &[]),
    (&CRYSTAL_DAEMON, &[]),
    (&CRYSTAL_LATTICE_SEEKER, &[]),
    (&SEA_SERPENT, &["Server.Mobiles.Seaserpant"]),
    (&CRYSTAL_SEA_SERPENT, &[]),
    (&CRYSTAL_VORTEX, &[]),
    (&WISP, &[]),
    (&CRYSTAL_WISP, &[]),
    (&MANTRA_EFFERVESCENCE, &[]),
    (&PROTECTOR, &[]),
    (&UNFROZEN_MUMMY, &[]),
    (&CHIIKKAHA, &[]),
    (&MOUG_GUUR, &[]),
    (&SZAVETRA, &[]),
    (&CHANGELING, &[]),
    (&GNAW, &[]),
    (&GUILE, &[]),
    (&IRK, &[]),
    (&LADY_LISSITH, &[]),
    (&LADY_SABRIX, &[]),
    (&MALEFIC, &[]),
    (&SILK, &[]),
    (&SPITE, &[]),
    (&SWOOP, &[]),
    (&VIRULENT, &[]),
    (&HELL_HOUND, &[]),
    (&VORPAL_BUNNY, &[]),
    (&DARK_WISP, &[]),
    (&ETHEREAL_WARRIOR, &[]),
    (&PIXIE, &[]),
    (&SHADOW_WISP, &[]),
    (&BLADE_SPIRITS, &[]),
    (&CENTAUR, &[]),
    (&ENERGY_VORTEX, &[]),
    (&FROST_OOZE, &[]),
    (&PLAGUE_BEAST, &[]),
    (&PLAGUE_BEAST_LORD, &[]),
    (&SAND_VORTEX, &[]),
    (&SLIME, &[]),
    (&AGAPITE_ELEMENTAL, &[]),
    (&BRONZE_ELEMENTAL, &[]),
    (&COPPER_ELEMENTAL, &[]),
    (&DULL_COPPER_ELEMENTAL, &[]),
    (&GOLDEN_ELEMENTAL, &[]),
    (&SHADOW_IRON_ELEMENTAL, &[]),
    (&VALORITE_ELEMENTAL, &[]),
    (&VERITE_ELEMENTAL, &[]),
    (&REAPER, &[]),
    (&BOGLING, &[]),
    (&CORPSER, &[]),
    (&QUAGMIRE, &[]),
    (&SWAMP_TENTACLE, &[]),
    (&WHIPPING_VINE, &[]),
    (&ANCIENT_WYRM, &[]),
    (&DEEP_SEA_SERPENT, &[]),
    (&DRAGON, &[]),
    (
        &OPHIDIAN_ARCHMAGE,
        &[
            "Server.Mobiles.OphidianJusticar",
            "Server.Mobiles.OphidianZealot",
        ],
    ),
    (&OPHIDIAN_MAGE, &["Server.Mobiles.OphidianShaman"]),
    (&OPHIDIAN_MATRIARCH, &[]),
    (&SERPENTINE_DRAGON, &[]),
    (&SHADOW_WYRM, &[]),
    (&SKELETAL_DRAGON, &[]),
    (&WHITE_WYRM, &[]),
    (&DRAKE, &[]),
    (&KRAKEN, &[]),
    (&LIZARDMAN, &[]),
    (&OPHIDIAN_KNIGHT, &["Server.Mobiles.OphidianAvenger"]),
    (&OPHIDIAN_WARRIOR, &[]),
    (&STONE_HARPY, &[]),
    (&WYVERN, &[]),
    (&RAPTOR, &[]),
    (&SLITH, &[]),
    (&STONE_SLITH, &[]),
    (&TOXIC_SLITH, &[]),
    (&BAKE_KITSUNE, &[]),
    (&DEATHWATCH_BEETLE, &["Server.Mobiles.DeathWatchBeetle"]),
    (
        &DEATHWATCH_BEETLE_HATCHLING,
        &["Server.Mobiles.DeathWatchBeetleHatchling"],
    ),
    (&ELITE_NINJA, &[]),
    (&FAN_DANCER, &[]),
    (&FIRE_BEETLE, &[]),
    (&KAPPA, &[]),
    (&KAZE_KEMONO, &[]),
    (&LADY_OF_THE_SNOW, &[]),
    (&ONI, &[]),
    (&RAI_JU, &[]),
    (&REVENANT_LION, &[]),
    (&RONIN, &[]),
    (&RUNE_BEETLE, &[]),
    (&TSUKI_WOLF, &[]),
    (&YAMANDON, &["Server.Mobiles.Yamadon"]),
    (&YOMOTSU_ELDER, &[]),
    (&YOMOTSU_PRIEST, &[]),
    (&YOMOTSU_WARRIOR, &[]),
    (&SUMMONED_AIR_ELEMENTAL, &[]),
    (&SUMMONED_DAEMON, &[]),
    (&SUMMONED_EARTH_ELEMENTAL, &[]),
    (&SUMMONED_FIRE_ELEMENTAL, &[]),
    (&SUMMONED_WATER_ELEMENTAL, &[]),
];

pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_mobile(t, aliases);
    }
}
