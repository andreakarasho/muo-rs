//! The concrete monster classes that are plain tables. Classes with logic of their own are hand-written in this
//! folder.

use crate::animals::leaves::*;
use crate::common::*;
use muo_content::abilities::{self, MonsterAbility};
use muo_content::creature::mount::{BASE_MOUNT, BASE_MOUNT_C, new_mount};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::items::weapon::Ability;
use muo_content::templates::{self, creature_props as tp};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;
use muo_server::poison::PoisonId;
use muo_server::type_registry::TypeRegistry;

fn abysmal_horror_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &ABYSMAL_HORROR)
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
    templates::new_creature(w, &BONE_DEMON)
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
    templates::new_creature(w, &CRYSTAL_ELEMENTAL)
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
        get_weapon_ability: |_w, _id| Some(Ability::BleedAttack),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn darknight_creeper_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &DARKNIGHT_CREEPER)
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
        hit_poison: |_w, _id| Some(PoisonId(4)),
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: crate::hand::darknight_creeper_on_death,
    }
}

fn devourer_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &DEVOURER)
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
    templates::new_creature(w, &FLESH_GOLEM)
}
creature_leaf! {
    /// `Server.Mobiles.FleshGolem`.
    FLESH_GOLEM, FLESH_GOLEM_C = "Server.Mobiles.FleshGolem" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(flesh_golem_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: |_w, _id| Some(Ability::BleedAttack),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn flesh_renderer_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &FLESH_RENDERER)
}
creature_leaf! {
    /// `Server.Mobiles.FleshRenderer`.
    FLESH_RENDERER, FLESH_RENDERER_C = "Server.Mobiles.FleshRenderer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(flesh_renderer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        ignore_young_protection: |w, _id| w.core_ml(),
        default_name: tp::default_name,
        auto_dispel: |_w, _id| true,
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
    templates::new_creature(w, &GIBBERLING)
}
creature_leaf! {
    /// `Server.Mobiles.Gibberling`.
    GIBBERLING, GIBBERLING_C = "Server.Mobiles.Gibberling" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gibberling_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: |_w, _id| Some(Ability::Dismount),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn gore_fiend_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &GORE_FIEND)
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
    templates::new_creature(w, &IMPALER)
}
creature_leaf! {
    /// `Server.Mobiles.Impaler`.
    IMPALER, IMPALER_C = "Server.Mobiles.Impaler" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(impaler_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        ignore_young_protection: |w, _id| w.core_ml(),
        auto_dispel: |_w, _id| true,
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
    templates::new_creature(w, &MOUND_OF_MAGGOTS)
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
    templates::new_creature(w, &PATCHWORK_SKELETON)
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
        get_weapon_ability: |_w, _id| Some(Ability::Dismount),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ravager_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &RAVAGER)
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
    templates::new_creature(w, &SHADOW_KNIGHT)
}
creature_leaf! {
    /// `Server.Mobiles.ShadowKnight`.
    SHADOW_KNIGHT, SHADOW_KNIGHT_C = "Server.Mobiles.ShadowKnight" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(shadow_knight_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        ignore_young_protection: |w, _id| w.core_ml(),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
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
    templates::new_creature(w, &SKITTERING_HOPPER)
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
    templates::new_creature(w, &TREEFELLOW)
}
creature_leaf! {
    /// `Server.Mobiles.Treefellow`.
    TREEFELLOW, TREEFELLOW_C = "Server.Mobiles.Treefellow" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(treefellow_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        bleed_immune: tp::bleed_immune,
        get_weapon_ability: |_w, _id| Some(Ability::Dismount),
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 443,
        get_death_sound: |_w, _id| 31,
        get_attack_sound: |_w, _id| 672,
    }
}

pub(crate) fn vampire_bat_init(w: &mut World, id: MobileId) {
    body(w, id, 317);
    base_sound_id(w, id, 0x270);
    set_str_range(w, id, 91, 110);
    set_dex_range(w, id, 91, 115);
    set_int_range(w, id, 26, 50);
    set_hits_range(w, id, 55, 66);
    set_damage(w, id, 7, 9);
    set_damage_type(w, id, ResistanceType::Physical, 80);
    set_damage_type(w, id, ResistanceType::Poison, 20);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 15, 25);
    set_resistance_range(w, id, ResistanceType::Cold, 15, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 60, 70);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 50);
    set_skill_range(w, id, SkillName::MagicResist, 70.1, 95.0);
    set_skill_range(w, id, SkillName::Tactics, 55.1, 80.0);
    set_skill_range(w, id, SkillName::Wrestling, 30.1, 55.0);
    fame(w, id, 1000);
    karma(w, id, -1000);
    virtual_armor(w, id, 14);
}
fn vampire_bat_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &VAMPIRE_BAT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    vampire_bat_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.VampireBat`.
    VAMPIRE_BAT, VAMPIRE_BAT_C = "Server.Mobiles.VampireBat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(vampire_bat_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a vampire bat corpse"),
        default_name: |_w, _id| Some("a vampire bat"),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::poor(w));
        },
    }
    m {
        get_idle_sound: |_w, _id| 0x29B,
    }
}

fn wailing_banshee_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &WAILING_BANSHEE)
}
creature_leaf! {
    /// `Server.Mobiles.WailingBanshee`.
    WAILING_BANSHEE, WAILING_BANSHEE_C = "Server.Mobiles.WailingBanshee" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(wailing_banshee_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        get_weapon_ability: |_w, _id| Some(Ability::MortalStrike),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn wanderer_of_the_void_init(w: &mut World, id: MobileId) {
    body(w, id, 316);
    base_sound_id(w, id, 377);
    set_str_range(w, id, 111, 200);
    set_dex_range(w, id, 101, 125);
    set_int_range(w, id, 301, 390);
    set_hits_range(w, id, 351, 400);
    set_damage(w, id, 11, 13);
    set_damage_type(w, id, ResistanceType::Physical, 0);
    set_damage_type(w, id, ResistanceType::Cold, 15);
    set_damage_type(w, id, ResistanceType::Energy, 85);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 50);
    set_resistance_range(w, id, ResistanceType::Fire, 15, 25);
    set_resistance_range(w, id, ResistanceType::Cold, 40, 50);
    set_resistance_range(w, id, ResistanceType::Poison, 50, 75);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 50);
    set_skill_range(w, id, SkillName::EvalInt, 60.1, 70.0);
    set_skill_range(w, id, SkillName::Magery, 60.1, 70.0);
    set_skill_range(w, id, SkillName::Meditation, 60.1, 70.0);
    set_skill_range(w, id, SkillName::MagicResist, 50.1, 75.0);
    set_skill_range(w, id, SkillName::Tactics, 60.1, 70.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 70.0);
    fame(w, id, 20000);
    karma(w, id, -20000);
    virtual_armor(w, id, 44);
    crate::hand::wanderer_of_the_void_ctor1(w, id);
}
fn wanderer_of_the_void_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &WANDERER_OF_THE_VOID,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    wanderer_of_the_void_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.WandererOfTheVoid`.
    WANDERER_OF_THE_VOID, WANDERER_OF_THE_VOID_C = "Server.Mobiles.WandererOfTheVoid" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(wanderer_of_the_void_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a wanderer of the void corpse"),
        default_name: |_w, _id| Some("a wanderer of the void"),
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(4)),
        treasure_map_level: |w, _id| if w.core_aos() { 4 } else { 1 },
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
        },
    }
    m {
    }
}

pub(crate) fn ant_lion_init(w: &mut World, id: MobileId) {
    body(w, id, 787);
    base_sound_id(w, id, 1006);
    set_str_range(w, id, 296, 320);
    set_dex_range(w, id, 81, 105);
    set_int_range(w, id, 36, 60);
    set_hits_range(w, id, 151, 162);
    set_damage(w, id, 7, 21);
    set_damage_type(w, id, ResistanceType::Physical, 70);
    set_damage_type(w, id, ResistanceType::Poison, 30);
    set_resistance_range(w, id, ResistanceType::Physical, 45, 60);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 35);
    set_skill(w, id, SkillName::MagicResist, 70.0);
    set_skill(w, id, SkillName::Tactics, 90.0);
    set_skill(w, id, SkillName::Wrestling, 90.0);
    fame(w, id, 4500);
    karma(w, id, -4500);
    virtual_armor(w, id, 45);
    pack_new_amount(w, id, "Server.Items.Bone", 3);
    {
        let a0 = "Server.Items.FertileDirt";
        let a1 = rnd::min_max(w, 1, 5);
        pack_new_amount(w, id, a0, a1);
    }
    crate::hand::ant_lion_ctor1(w, id);
    crate::hand::ant_lion_ctor2(w, id);
}
fn ant_lion_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ANT_LION,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ant_lion_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.AntLion`.
    ANT_LION, ANT_LION_C = "Server.Mobiles.AntLion" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ant_lion_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an ant lion corpse"),
        default_name: |_w, _id| Some("an ant lion"),
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::average(w), 2);
        },
    }
    m {
        get_anger_sound: |_w, _id| 0x5A,
        get_idle_sound: |_w, _id| 0x5A,
        get_attack_sound: |_w, _id| 0x164,
        get_hurt_sound: |_w, _id| 0x187,
        get_death_sound: |_w, _id| 0x1BA,
    }
}

pub(crate) fn black_solen_infiltrator_queen_init(w: &mut World, id: MobileId) {
    body(w, id, 807);
    base_sound_id(w, id, 959);
    hue(w, id, 0x453);
    set_str_range(w, id, 326, 350);
    set_dex_range(w, id, 141, 165);
    set_int_range(w, id, 96, 120);
    set_hits_range(w, id, 151, 162);
    set_damage(w, id, 10, 15);
    set_damage_type(w, id, ResistanceType::Physical, 70);
    set_damage_type(w, id, ResistanceType::Poison, 30);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 35, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 30);
    set_skill(w, id, SkillName::MagicResist, 90.0);
    set_skill(w, id, SkillName::Tactics, 90.0);
    set_skill(w, id, SkillName::Wrestling, 90.0);
    fame(w, id, 6500);
    karma(w, id, -6500);
    virtual_armor(w, id, 50);
    muo_content::creature::solen::pack_picnic_basket(w, id);
    {
        let a0 = "Server.Items.ZoogiFungus";
        let a1 = if rnd::double(w) < 0.05 { 16 } else { 4 };
        pack_new_amount(w, id, a0, a1);
    }
}
fn black_solen_infiltrator_queen_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BLACK_SOLEN_INFILTRATOR_QUEEN,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    black_solen_infiltrator_queen_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.BlackSolenInfiltratorQueen`.
    BLACK_SOLEN_INFILTRATOR_QUEEN, BLACK_SOLEN_INFILTRATOR_QUEEN_C = "Server.Mobiles.BlackSolenInfiltratorQueen" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(black_solen_infiltrator_queen_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a solen infiltrator corpse"),
        default_name: |_w, _id| Some("a black solen infiltrator"),
        is_enemy: crate::hand::black_solen_infiltrator_queen_is_enemy,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
        },
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

pub(crate) fn black_solen_infiltrator_warrior_init(w: &mut World, id: MobileId) {
    body(w, id, 806);
    base_sound_id(w, id, 959);
    hue(w, id, 0x453);
    set_str_range(w, id, 206, 230);
    set_dex_range(w, id, 121, 145);
    set_int_range(w, id, 66, 90);
    set_hits_range(w, id, 96, 107);
    set_damage(w, id, 5, 15);
    set_damage_type(w, id, ResistanceType::Physical, 80);
    set_damage_type(w, id, ResistanceType::Poison, 20);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 25);
    set_skill(w, id, SkillName::MagicResist, 80.0);
    set_skill(w, id, SkillName::Tactics, 80.0);
    set_skill(w, id, SkillName::Wrestling, 80.0);
    fame(w, id, 3000);
    karma(w, id, -3000);
    virtual_armor(w, id, 40);
    muo_content::creature::solen::pack_picnic_basket(w, id);
    {
        let a0 = "Server.Items.ZoogiFungus";
        let a1 = if rnd::double(w) < 0.05 { 13 } else { 3 };
        pack_new_amount(w, id, a0, a1);
    }
}
fn black_solen_infiltrator_warrior_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BLACK_SOLEN_INFILTRATOR_WARRIOR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    black_solen_infiltrator_warrior_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.BlackSolenInfiltratorWarrior`.
    BLACK_SOLEN_INFILTRATOR_WARRIOR, BLACK_SOLEN_INFILTRATOR_WARRIOR_C = "Server.Mobiles.BlackSolenInfiltratorWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(black_solen_infiltrator_warrior_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a solen infiltrator corpse"),
        default_name: |_w, _id| Some("a black solen infiltrator"),
        is_enemy: crate::hand::black_solen_infiltrator_warrior_is_enemy,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::average(w), 2);
            {
                let a0 = &loot_pack::GEMS;
                let a1 = rnd::min_max(w, 1, 4);
                add_loot_n(w, id, a0, a1);
            }
        },
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

pub(crate) fn black_solen_queen_init(w: &mut World, id: MobileId) {
    crate::hand::burst_sac_init(w, id);
    body(w, id, 807);
    base_sound_id(w, id, 959);
    hue(w, id, 0x453);
    set_str_range(w, id, 296, 320);
    set_dex_range(w, id, 121, 145);
    set_int_range(w, id, 76, 100);
    set_hits_range(w, id, 151, 162);
    set_damage(w, id, 10, 15);
    set_damage_type(w, id, ResistanceType::Physical, 70);
    set_damage_type(w, id, ResistanceType::Poison, 30);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 35, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 30);
    set_skill(w, id, SkillName::MagicResist, 70.0);
    set_skill(w, id, SkillName::Tactics, 90.0);
    set_skill(w, id, SkillName::Wrestling, 90.0);
    fame(w, id, 4500);
    karma(w, id, -4500);
    virtual_armor(w, id, 45);
    muo_content::creature::solen::pack_picnic_basket(w, id);
    {
        let a0 = "Server.Items.ZoogiFungus";
        let a1 = if rnd::double(w) < 0.95 { 5 } else { 25 };
        pack_new_amount(w, id, a0, a1);
    }
    if rnd::double(w) < 0.05 {
        pack_new(w, id, "Server.Items.BallOfSummoning");
    }
}
fn black_solen_queen_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BLACK_SOLEN_QUEEN,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    black_solen_queen_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.BlackSolenQueen`.
    BLACK_SOLEN_QUEEN, BLACK_SOLEN_QUEEN_C = "Server.Mobiles.BlackSolenQueen" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(black_solen_queen_new), state crate::hand::burst_sac_serialize, crate::hand::burst_sac_deserialize;
    c {
        corpse_name: |_w, _id| Some("a solen queen corpse"),
        default_name: |_w, _id| Some("a black solen queen"),
        is_enemy: crate::hand::black_solen_queen_is_enemy,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
        },
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

pub(crate) fn black_solen_warrior_init(w: &mut World, id: MobileId) {
    crate::hand::burst_sac_init(w, id);
    body(w, id, 806);
    base_sound_id(w, id, 959);
    hue(w, id, 0x453);
    set_str_range(w, id, 196, 220);
    set_dex_range(w, id, 101, 125);
    set_int_range(w, id, 36, 60);
    set_hits_range(w, id, 96, 107);
    set_damage(w, id, 5, 15);
    set_damage_type(w, id, ResistanceType::Physical, 80);
    set_damage_type(w, id, ResistanceType::Poison, 20);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 25);
    set_skill(w, id, SkillName::MagicResist, 60.0);
    set_skill(w, id, SkillName::Tactics, 80.0);
    set_skill(w, id, SkillName::Wrestling, 80.0);
    fame(w, id, 3000);
    karma(w, id, -3000);
    virtual_armor(w, id, 35);
    muo_content::creature::solen::pack_picnic_basket(w, id);
    {
        let a0 = "Server.Items.ZoogiFungus";
        let a1 = if rnd::double(w) < 0.05 { 13 } else { 3 };
        pack_new_amount(w, id, a0, a1);
    }
    if rnd::double(w) < 0.05 {
        pack_new(w, id, "Server.Items.BraceletOfBinding");
    }
}
fn black_solen_warrior_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BLACK_SOLEN_WARRIOR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    black_solen_warrior_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.BlackSolenWarrior`.
    BLACK_SOLEN_WARRIOR, BLACK_SOLEN_WARRIOR_C = "Server.Mobiles.BlackSolenWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(black_solen_warrior_new), state crate::hand::burst_sac_serialize, crate::hand::burst_sac_deserialize;
    c {
        corpse_name: |_w, _id| Some("a solen warrior corpse"),
        default_name: |_w, _id| Some("a black solen warrior"),
        is_enemy: crate::hand::black_solen_warrior_is_enemy,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
            {
                let a0 = &loot_pack::GEMS;
                let a1 = rnd::min_max(w, 1, 4);
                add_loot_n(w, id, a0, a1);
            }
        },
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

pub(crate) fn black_solen_worker_init(w: &mut World, id: MobileId) {
    body(w, id, 805);
    base_sound_id(w, id, 959);
    hue(w, id, 0x453);
    set_str_range(w, id, 96, 120);
    set_dex_range(w, id, 81, 105);
    set_int_range(w, id, 36, 60);
    set_hits_range(w, id, 58, 72);
    set_damage(w, id, 5, 7);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 30);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 20);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 20);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);
    set_skill(w, id, SkillName::MagicResist, 60.0);
    set_skill(w, id, SkillName::Tactics, 65.0);
    set_skill(w, id, SkillName::Wrestling, 60.0);
    fame(w, id, 1500);
    karma(w, id, -1500);
    virtual_armor(w, id, 28);
    {
        let a0 = rnd::from(w, 100, 180);
        pack_gold(w, id, a0);
    }
    muo_content::creature::solen::pack_picnic_basket(w, id);
    pack_new(w, id, "Server.Items.ZoogiFungus");
}
fn black_solen_worker_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BLACK_SOLEN_WORKER,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    black_solen_worker_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.BlackSolenWorker`.
    BLACK_SOLEN_WORKER, BLACK_SOLEN_WORKER_C = "Server.Mobiles.BlackSolenWorker" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(black_solen_worker_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a solen worker corpse"),
        default_name: |_w, _id| Some("a black solen worker"),
        is_enemy: crate::hand::black_solen_worker_is_enemy,
        generate_loot: |w, id| {
            {
                let a0 = &loot_pack::GEMS;
                let a1 = rnd::min_max(w, 1, 2);
                add_loot_n(w, id, a0, a1);
            }
        },
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

pub(crate) fn red_solen_infiltrator_queen_init(w: &mut World, id: MobileId) {
    body(w, id, 783);
    base_sound_id(w, id, 959);
    set_str_range(w, id, 326, 350);
    set_dex_range(w, id, 141, 165);
    set_int_range(w, id, 96, 120);
    set_hits_range(w, id, 151, 162);
    set_damage(w, id, 10, 15);
    set_damage_type(w, id, ResistanceType::Physical, 70);
    set_damage_type(w, id, ResistanceType::Poison, 30);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 35, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 30);
    set_skill(w, id, SkillName::MagicResist, 90.0);
    set_skill(w, id, SkillName::Tactics, 90.0);
    set_skill(w, id, SkillName::Wrestling, 90.0);
    fame(w, id, 6500);
    karma(w, id, -6500);
    virtual_armor(w, id, 50);
    muo_content::creature::solen::pack_picnic_basket(w, id);
    {
        let a0 = "Server.Items.ZoogiFungus";
        let a1 = if rnd::double(w) < 0.95 { 4 } else { 16 };
        pack_new_amount(w, id, a0, a1);
    }
}
fn red_solen_infiltrator_queen_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &RED_SOLEN_INFILTRATOR_QUEEN,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    red_solen_infiltrator_queen_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.RedSolenInfiltratorQueen`.
    RED_SOLEN_INFILTRATOR_QUEEN, RED_SOLEN_INFILTRATOR_QUEEN_C = "Server.Mobiles.RedSolenInfiltratorQueen" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(red_solen_infiltrator_queen_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a solen infiltrator corpse"),
        default_name: |_w, _id| Some("a red solen infiltrator"),
        is_enemy: crate::hand::red_solen_infiltrator_queen_is_enemy,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
        },
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

pub(crate) fn red_solen_infiltrator_warrior_init(w: &mut World, id: MobileId) {
    body(w, id, 782);
    base_sound_id(w, id, 959);
    set_str_range(w, id, 206, 230);
    set_dex_range(w, id, 121, 145);
    set_int_range(w, id, 66, 90);
    set_hits_range(w, id, 96, 107);
    set_damage(w, id, 5, 15);
    set_damage_type(w, id, ResistanceType::Physical, 80);
    set_damage_type(w, id, ResistanceType::Poison, 20);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 25);
    set_skill(w, id, SkillName::MagicResist, 80.0);
    set_skill(w, id, SkillName::Tactics, 80.0);
    set_skill(w, id, SkillName::Wrestling, 80.0);
    fame(w, id, 3000);
    karma(w, id, -3000);
    virtual_armor(w, id, 40);
    muo_content::creature::solen::pack_picnic_basket(w, id);
    {
        let a0 = "Server.Items.ZoogiFungus";
        let a1 = if rnd::double(w) < 0.95 { 3 } else { 13 };
        pack_new_amount(w, id, a0, a1);
    }
}
fn red_solen_infiltrator_warrior_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &RED_SOLEN_INFILTRATOR_WARRIOR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    red_solen_infiltrator_warrior_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.RedSolenInfiltratorWarrior`.
    RED_SOLEN_INFILTRATOR_WARRIOR, RED_SOLEN_INFILTRATOR_WARRIOR_C = "Server.Mobiles.RedSolenInfiltratorWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(red_solen_infiltrator_warrior_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a solen infiltrator corpse"),
        default_name: |_w, _id| Some("a red solen infiltrator"),
        is_enemy: crate::hand::red_solen_infiltrator_warrior_is_enemy,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::average(w), 2);
            {
                let a0 = &loot_pack::GEMS;
                let a1 = rnd::min_max(w, 1, 4);
                add_loot_n(w, id, a0, a1);
            }
        },
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

pub(crate) fn red_solen_queen_init(w: &mut World, id: MobileId) {
    crate::hand::burst_sac_init(w, id);
    body(w, id, 783);
    base_sound_id(w, id, 959);
    set_str_range(w, id, 296, 320);
    set_dex_range(w, id, 121, 145);
    set_int_range(w, id, 76, 100);
    set_hits_range(w, id, 151, 162);
    set_damage(w, id, 10, 15);
    set_damage_type(w, id, ResistanceType::Physical, 70);
    set_damage_type(w, id, ResistanceType::Poison, 30);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 35, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 30);
    set_skill(w, id, SkillName::MagicResist, 70.0);
    set_skill(w, id, SkillName::Tactics, 90.0);
    set_skill(w, id, SkillName::Wrestling, 90.0);
    fame(w, id, 4500);
    karma(w, id, -4500);
    virtual_armor(w, id, 45);
    muo_content::creature::solen::pack_picnic_basket(w, id);
    {
        let a0 = "Server.Items.ZoogiFungus";
        let a1 = if rnd::double(w) < 0.95 { 5 } else { 25 };
        pack_new_amount(w, id, a0, a1);
    }
    if rnd::double(w) < 0.05 {
        pack_new(w, id, "Server.Items.BallOfSummoning");
    }
}
fn red_solen_queen_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &RED_SOLEN_QUEEN,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    red_solen_queen_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.RedSolenQueen`.
    RED_SOLEN_QUEEN, RED_SOLEN_QUEEN_C = "Server.Mobiles.RedSolenQueen" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(red_solen_queen_new), state crate::hand::burst_sac_serialize, crate::hand::burst_sac_deserialize;
    c {
        corpse_name: |_w, _id| Some("a solen queen corpse"),
        default_name: |_w, _id| Some("a red solen queen"),
        is_enemy: crate::hand::red_solen_queen_is_enemy,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
        },
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

pub(crate) fn red_solen_warrior_init(w: &mut World, id: MobileId) {
    crate::hand::burst_sac_init(w, id);
    body(w, id, 782);
    base_sound_id(w, id, 959);
    set_str_range(w, id, 196, 220);
    set_dex_range(w, id, 101, 125);
    set_int_range(w, id, 36, 60);
    set_hits_range(w, id, 96, 107);
    set_damage(w, id, 5, 15);
    set_damage_type(w, id, ResistanceType::Physical, 80);
    set_damage_type(w, id, ResistanceType::Poison, 20);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 25);
    set_skill(w, id, SkillName::MagicResist, 60.0);
    set_skill(w, id, SkillName::Tactics, 80.0);
    set_skill(w, id, SkillName::Wrestling, 80.0);
    fame(w, id, 3000);
    karma(w, id, -3000);
    virtual_armor(w, id, 35);
    muo_content::creature::solen::pack_picnic_basket(w, id);
    {
        let a0 = "Server.Items.ZoogiFungus";
        let a1 = if rnd::double(w) < 0.95 { 3 } else { 13 };
        pack_new_amount(w, id, a0, a1);
    }
    if rnd::double(w) < 0.05 {
        pack_new(w, id, "Server.Items.BraceletOfBinding");
    }
}
fn red_solen_warrior_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &RED_SOLEN_WARRIOR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    red_solen_warrior_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.RedSolenWarrior`.
    RED_SOLEN_WARRIOR, RED_SOLEN_WARRIOR_C = "Server.Mobiles.RedSolenWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(red_solen_warrior_new), state crate::hand::burst_sac_serialize, crate::hand::burst_sac_deserialize;
    c {
        corpse_name: |_w, _id| Some("a solen warrior corpse"),
        default_name: |_w, _id| Some("a red solen warrior"),
        is_enemy: crate::hand::red_solen_warrior_is_enemy,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
            {
                let a0 = &loot_pack::GEMS;
                let a1 = rnd::min_max(w, 1, 4);
                add_loot_n(w, id, a0, a1);
            }
        },
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

pub(crate) fn red_solen_worker_init(w: &mut World, id: MobileId) {
    body(w, id, 781);
    base_sound_id(w, id, 959);
    set_str_range(w, id, 96, 120);
    set_dex_range(w, id, 81, 105);
    set_int_range(w, id, 36, 60);
    set_hits_range(w, id, 58, 72);
    set_damage(w, id, 5, 7);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 30);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 20);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 20);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);
    set_skill(w, id, SkillName::MagicResist, 60.0);
    set_skill(w, id, SkillName::Tactics, 65.0);
    set_skill(w, id, SkillName::Wrestling, 60.0);
    fame(w, id, 1500);
    karma(w, id, -1500);
    virtual_armor(w, id, 28);
    {
        let a0 = rnd::from(w, 100, 180);
        pack_gold(w, id, a0);
    }
    muo_content::creature::solen::pack_picnic_basket(w, id);
    pack_new(w, id, "Server.Items.ZoogiFungus");
}
fn red_solen_worker_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &RED_SOLEN_WORKER,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    red_solen_worker_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.RedSolenWorker`.
    RED_SOLEN_WORKER, RED_SOLEN_WORKER_C = "Server.Mobiles.RedSolenWorker" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(red_solen_worker_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a solen worker corpse"),
        default_name: |_w, _id| Some("a red solen worker"),
        is_enemy: crate::hand::red_solen_worker_is_enemy,
        generate_loot: |w, id| {
            {
                let a0 = &loot_pack::GEMS;
                let a1 = rnd::min_max(w, 1, 2);
                add_loot_n(w, id, a0, a1);
            }
        },
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
    templates::new_creature(w, &DREAD_SPIDER)
}
creature_leaf! {
    /// `Server.Mobiles.DreadSpider`.
    DREAD_SPIDER, DREAD_SPIDER_C = "Server.Mobiles.DreadSpider" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dread_spider_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        hit_poison: |_w, _id| Some(PoisonId(4)),
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn terathan_avenger_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &TERATHAN_AVENGER)
}
creature_leaf! {
    /// `Server.Mobiles.TerathanAvenger`.
    TERATHAN_AVENGER, TERATHAN_AVENGER_C = "Server.Mobiles.TerathanAvenger" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(terathan_avenger_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        hit_poison: |_w, _id| Some(PoisonId(3)),
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::TERATHANS_AND_OPHIDIANS),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn terathan_matriarch_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &TERATHAN_MATRIARCH)
}
creature_leaf! {
    /// `Server.Mobiles.TerathanMatriarch`.
    TERATHAN_MATRIARCH, TERATHAN_MATRIARCH_C = "Server.Mobiles.TerathanMatriarch" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(terathan_matriarch_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        treasure_map_level: tp::treasure_map_level,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::TERATHANS_AND_OPHIDIANS),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn frost_spider_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &FROST_SPIDER)
}
creature_leaf! {
    /// `Server.Mobiles.FrostSpider`.
    FROST_SPIDER, FROST_SPIDER_C = "Server.Mobiles.FrostSpider" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(frost_spider_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::ARACHNID,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn giant_black_widow_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &GIANT_BLACK_WIDOW)
}
creature_leaf! {
    /// `Server.Mobiles.GiantBlackWidow`.
    GIANT_BLACK_WIDOW, GIANT_BLACK_WIDOW_C = "Server.Mobiles.GiantBlackWidow" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(giant_black_widow_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        favorite_food: |_w, _id| FoodType::MEAT,
        poison_immune: tp::poison_immune,
        hit_poison: |_w, _id| Some(PoisonId(3)),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn giant_spider_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &GIANT_SPIDER)
}
creature_leaf! {
    /// `Server.Mobiles.GiantSpider`.
    GIANT_SPIDER, GIANT_SPIDER_C = "Server.Mobiles.GiantSpider" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(giant_spider_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::ARACHNID,
        poison_immune: tp::poison_immune,
        hit_poison: |_w, _id| Some(PoisonId(1)),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn terathan_drone_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &TERATHAN_DRONE)
}
creature_leaf! {
    /// `Server.Mobiles.TerathanDrone`.
    TERATHAN_DRONE, TERATHAN_DRONE_C = "Server.Mobiles.TerathanDrone" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(terathan_drone_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::TERATHANS_AND_OPHIDIANS),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn terathan_warrior_init(w: &mut World, id: MobileId) {
    body(w, id, 70);
    base_sound_id(w, id, 589);
    set_str_range(w, id, 166, 215);
    set_dex_range(w, id, 96, 145);
    set_int_range(w, id, 41, 65);
    set_hits_range(w, id, 100, 129);
    set_mana(w, id, 0);
    set_damage(w, id, 7, 17);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::Poisoning, 60.1, 80.0);
    set_skill_range(w, id, SkillName::MagicResist, 60.1, 75.0);
    set_skill_range(w, id, SkillName::Tactics, 80.1, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 80.1, 90.0);
    fame(w, id, 4000);
    karma(w, id, -4000);
    virtual_armor(w, id, 30);
    crate::hand::terathan_warrior_ctor1(w, id);
}
fn terathan_warrior_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &TERATHAN_WARRIOR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    terathan_warrior_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.TerathanWarrior`.
    TERATHAN_WARRIOR, TERATHAN_WARRIOR_C = "Server.Mobiles.TerathanWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(terathan_warrior_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a terathan warrior corpse"),
        default_name: |_w, _id| Some("a terathan warrior"),
        treasure_map_level: |_w, _id| 1,
        meat: |_w, _id| 4,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::TERATHANS_AND_OPHIDIANS),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
        },
    }
    m {
    }
}

fn acid_elemental_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &ACID_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.AcidElemental`.
    ACID_ELEMENTAL, ACID_ELEMENTAL_C = "Server.Mobiles.AcidElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(acid_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        hit_poison: |_w, _id| Some(PoisonId(4)),
        hit_poison_chance: |_w, _id| 0.6,
        treasure_map_level: |w, _id| if w.core_aos() { 2 } else { 3 },
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn air_elemental_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &AIR_ELEMENTAL)
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
    templates::new_creature(w, &BLOOD_ELEMENTAL)
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
    templates::new_creature(w, &EFREET)
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
    templates::new_creature(w, &FIRE_ELEMENTAL)
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
    templates::new_creature(w, &ICE_ELEMENTAL)
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
    templates::new_creature(w, &POISON_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.PoisonElemental`.
    POISON_ELEMENTAL, POISON_ELEMENTAL_C = "Server.Mobiles.PoisonElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(poison_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        hit_poison: |_w, _id| Some(PoisonId(4)),
        hit_poison_chance: |_w, _id| 0.75,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn water_elemental_init(w: &mut World, id: MobileId) {
    crate::hand::water_elemental_ctor1(w, id);
    body(w, id, 16);
    base_sound_id(w, id, 278);
    set_str_range(w, id, 126, 155);
    set_dex_range(w, id, 66, 85);
    set_int_range(w, id, 101, 125);
    set_hits_range(w, id, 76, 93);
    set_damage(w, id, 7, 9);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 25);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 60, 70);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill_range(w, id, SkillName::EvalInt, 60.1, 75.0);
    set_skill_range(w, id, SkillName::Magery, 60.1, 75.0);
    set_skill_range(w, id, SkillName::MagicResist, 100.1, 115.0);
    set_skill_range(w, id, SkillName::Tactics, 50.1, 70.0);
    set_skill_range(w, id, SkillName::Wrestling, 50.1, 70.0);
    fame(w, id, 4500);
    karma(w, id, -4500);
    virtual_armor(w, id, 40);
    can_swim(w, id, true);
    pack_new_amount(w, id, "Server.Items.BlackPearl", 3);
}
fn water_elemental_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &WATER_ELEMENTAL,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    water_elemental_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.WaterElemental`.
    WATER_ELEMENTAL, WATER_ELEMENTAL_C = "Server.Mobiles.WaterElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(water_elemental_new), state crate::hand::water_elemental_serialize, crate::hand::water_elemental_deserialize;
    c {
        corpse_name: |_w, _id| Some("a water elemental corpse"),
        default_name: |_w, _id| Some("a water elemental"),
        bleed_immune: |_w, _id| true,
        treasure_map_level: |_w, _id| 2,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
            add_loot(w, id, loot_pack::meager(w));
            add_loot(w, id, &loot_pack::POTIONS);
        },
    }
    m {
    }
}

fn earth_elemental_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &EARTH_ELEMENTAL)
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
    templates::new_creature(w, &SNOW_ELEMENTAL)
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
    templates::new_creature(w, &ANCIENT_LICH)
}
static ANCIENT_LICH_ABILITIES: [&MonsterAbility; 1] = [&abilities::SUMMON_LESSER_UNDEAD_COUNTER];
creature_leaf! {
    /// `Server.Mobiles.AncientLich`.
    ANCIENT_LICH, ANCIENT_LICH_C = "Server.Mobiles.AncientLich" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ancient_lich_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        unprovokable: |_w, _id| true,
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
    templates::new_creature(w, &ARCANE_DAEMON)
}
creature_leaf! {
    /// `Server.Mobiles.ArcaneDaemon`.
    ARCANE_DAEMON, ARCANE_DAEMON_C = "Server.Mobiles.ArcaneDaemon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(arcane_daemon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        get_weapon_ability: |_w, _id| Some(Ability::ConcussionBlow),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn balron_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &BALRON)
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

pub(crate) fn betrayer_init(w: &mut World, id: MobileId) {
    body(w, id, 767);
    set_str_range(w, id, 401, 500);
    set_dex_range(w, id, 81, 100);
    set_int_range(w, id, 151, 200);
    set_hits_range(w, id, 241, 300);
    set_damage(w, id, 16, 22);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 60, 70);
    set_resistance_range(w, id, ResistanceType::Fire, 60, 70);
    set_resistance_range(w, id, ResistanceType::Cold, 60, 70);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);
    set_skill_range(w, id, SkillName::Anatomy, 90.1, 100.0);
    set_skill_range(w, id, SkillName::EvalInt, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Magery, 50.1, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 90.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 120.1, 130.0);
    set_skill_range(w, id, SkillName::Tactics, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 90.1, 100.0);
    fame(w, id, 15000);
    karma(w, id, -15000);
    virtual_armor(w, id, 65);
    {
        let a0 = rnd::from(w, 2, 1000);
        speech_hue(w, id, a0);
    }
    pack_new(w, id, "Server.Items.PowerCrystal");
    if rnd::double(w) < 0.02 {
        pack_new(w, id, "Server.Items.BlackthornWelcomeBook");
    }
}
fn betrayer_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BETRAYER,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    betrayer_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Betrayer`.
    BETRAYER, BETRAYER_C = "Server.Mobiles.Betrayer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(betrayer_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a betrayer corpse"),
        default_name: |_w, _id| Some("a betrayer"),
        always_murderer: |_w, _id| true,
        bard_immune: |w, _id| !w.core_aos(),
        poison_immune: |_w, _id| Some(PoisonId(4)),
        meat: |_w, _id| 1,
        treasure_map_level: |_w, _id| 5,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot(w, id, loot_pack::rich(w));
            add_loot_n(w, id, &loot_pack::GEMS, 1);
        },
        get_monster_abilities: crate::hand::betrayer_abilities,
    }
    m {
        on_death: crate::hand::betrayer_on_death,
        get_death_sound: |_w, _id| 0x423,
        get_attack_sound: |_w, _id| 0x23B,
        get_hurt_sound: |_w, _id| 0x140,
    }
}

pub(crate) fn bogle_init(w: &mut World, id: MobileId) {
    body(w, id, 153);
    base_sound_id(w, id, 0x482);
    set_str_range(w, id, 76, 100);
    set_dex_range(w, id, 76, 95);
    set_int_range(w, id, 36, 60);
    set_hits_range(w, id, 46, 60);
    set_damage(w, id, 7, 11);
    set_skill_range(w, id, SkillName::EvalInt, 55.1, 70.0);
    set_skill_range(w, id, SkillName::Magery, 55.1, 70.0);
    set_skill_range(w, id, SkillName::MagicResist, 55.1, 70.0);
    set_skill_range(w, id, SkillName::Tactics, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 45.1, 55.0);
    fame(w, id, 4000);
    karma(w, id, -4000);
    virtual_armor(w, id, 28);
    {
        let it = muo_content::loot::random_weapon(w, false, false, false);
        pack_item(w, id, it);
    }
    pack_new(w, id, "Server.Items.Bone");
}
fn bogle_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BOGLE,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    bogle_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Bogle`.
    BOGLE, BOGLE_C = "Server.Mobiles.Bogle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bogle_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a ghostly corpse"),
        default_name: |_w, _id| Some("a bogle"),
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(4)),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

pub(crate) fn bone_magi_init(w: &mut World, id: MobileId) {
    body(w, id, 148);
    base_sound_id(w, id, 451);
    set_str_range(w, id, 76, 100);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 186, 210);
    set_hits_range(w, id, 46, 60);
    set_damage(w, id, 3, 7);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 60);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::EvalInt, 60.1, 70.0);
    set_skill_range(w, id, SkillName::Magery, 60.1, 70.0);
    set_skill_range(w, id, SkillName::MagicResist, 55.1, 70.0);
    set_skill_range(w, id, SkillName::Tactics, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 45.1, 55.0);
    set_skill_range(w, id, SkillName::Necromancy, 89.0, 99.1);
    set_skill_range(w, id, SkillName::SpiritSpeak, 90.0, 99.0);
    fame(w, id, 3000);
    karma(w, id, -3000);
    virtual_armor(w, id, 38);
    pack_reg(w, id, 3);
    pack_necro_reg_range(w, id, 3, 10);
    pack_new(w, id, "Server.Items.Bone");
}
fn bone_magi_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &BONE_MAGI)
}
creature_leaf! {
    /// `Server.Mobiles.BoneMagi`.
    BONE_MAGI, BONE_MAGI_C = "Server.Mobiles.BoneMagi" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bone_magi_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn daemon_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &DAEMON)
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
    templates::new_creature(w, &ELDER_GAZER)
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

pub(crate) fn evil_mage_init(w: &mut World, id: MobileId) {
    name_random(w, id, "evil mage");
    title(w, id, "the evil mage");
    crate::hand::evil_mage_ctor1(w, id);
    set_str_range(w, id, 81, 105);
    set_dex_range(w, id, 91, 115);
    set_int_range(w, id, 96, 120);
    set_hits_range(w, id, 49, 63);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill_range(w, id, SkillName::EvalInt, 75.1, 100.0);
    set_skill_range(w, id, SkillName::Magery, 75.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 75.0, 97.5);
    set_skill_range(w, id, SkillName::Tactics, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Wrestling, 20.2, 60.0);
    fame(w, id, 2500);
    karma(w, id, -2500);
    virtual_armor(w, id, 16);
    pack_reg(w, id, 6);
    crate::hand::evil_mage_ctor2(w, id);
    crate::hand::evil_mage_ctor3(w, id);
}
fn evil_mage_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &EVIL_MAGE,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    evil_mage_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.EvilMage`.
    EVIL_MAGE, EVIL_MAGE_C = "Server.Mobiles.EvilMage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(evil_mage_new), state crate::hand::evil_mage_serialize, crate::hand::evil_mage_deserialize;
    c {
        corpse_name: |_w, _id| Some("an evil mage corpse"),
        can_rummage_corpses: |_w, _id| true,
        always_murderer: |_w, _id| true,
        meat: |_w, _id| 1,
        treasure_map_level: |w, _id| if w.core_aos() { 1 } else { 0 },
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
            add_loot(w, id, &loot_pack::MED_SCROLLS);
        },
    }
    m {
    }
}

pub(crate) fn evil_mage_lord_init(w: &mut World, id: MobileId) {
    name_random(w, id, "evil mage lord");
    crate::hand::evil_mage_lord_ctor1(w, id);
    set_str_range(w, id, 81, 105);
    set_dex_range(w, id, 191, 215);
    set_int_range(w, id, 126, 150);
    set_hits_range(w, id, 49, 63);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::EvalInt, 80.2, 100.0);
    set_skill_range(w, id, SkillName::Magery, 95.1, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 27.5, 50.0);
    set_skill_range(w, id, SkillName::MagicResist, 77.5, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Wrestling, 20.3, 80.0);
    fame(w, id, 10500);
    karma(w, id, -10500);
    virtual_armor(w, id, 16);
    pack_reg(w, id, 23);
    crate::hand::evil_mage_lord_ctor2(w, id);
    crate::hand::evil_mage_lord_ctor3(w, id);
}
fn evil_mage_lord_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &EVIL_MAGE_LORD,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    evil_mage_lord_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.EvilMageLord`.
    EVIL_MAGE_LORD, EVIL_MAGE_LORD_C = "Server.Mobiles.EvilMageLord" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(evil_mage_lord_new), state crate::hand::evil_mage_serialize, crate::hand::evil_mage_lord_deserialize;
    c {
        corpse_name: |_w, _id| Some("an evil mage lord corpse"),
        can_rummage_corpses: |_w, _id| true,
        always_murderer: |_w, _id| true,
        meat: |_w, _id| 1,
        treasure_map_level: |w, _id| if w.core_aos() { 2 } else { 0 },
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
            add_loot(w, id, loot_pack::meager(w));
            add_loot_n(w, id, &loot_pack::MED_SCROLLS, 2);
        },
    }
    m {
    }
}

fn fire_gargoyle_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &FIRE_GARGOYLE)
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
    templates::new_creature(w, &GARGOYLE)
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
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
            add_loot(w, id, &loot_pack::MED_SCROLLS);
            {
                let a0 = &loot_pack::GEMS;
                let a1 = rnd::min_max(w, 1, 4);
                add_loot_n(w, id, a0, a1);
            }
        },
    }
    m {
    }
}

fn gargoyle_destroyer_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &GARGOYLE_DESTROYER)
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
    templates::new_creature(w, &GARGOYLE_ENFORCER)
}
creature_leaf! {
    /// `Server.Mobiles.GargoyleEnforcer`.
    GARGOYLE_ENFORCER, GARGOYLE_ENFORCER_C = "Server.Mobiles.GargoyleEnforcer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(gargoyle_enforcer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_fly: tp::can_fly,
        meat: tp::meat,
        get_weapon_ability: |_w, _id| Some(Ability::WhirlwindAttack),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn gazer_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &GAZER)
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

pub(crate) fn golem_controller_init(w: &mut World, id: MobileId) {
    name_random(w, id, "golem controller");
    title(w, id, "the controller");
    body(w, id, 400);
    hue(w, id, 0x455);
    crate::hand::golem_controller_ctor1(w, id);
    crate::hand::golem_controller_ctor2(w, id);
    crate::hand::golem_controller_ctor3(w, id);
    crate::hand::golem_controller_ctor4(w, id);
    set_str_range(w, id, 126, 150);
    set_dex_range(w, id, 96, 120);
    set_int_range(w, id, 151, 175);
    set_hits_range(w, id, 76, 90);
    set_damage(w, id, 6, 12);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 35, 45);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 15);
    set_resistance_range(w, id, ResistanceType::Energy, 15, 25);
    set_skill_range(w, id, SkillName::EvalInt, 95.1, 100.0);
    set_skill_range(w, id, SkillName::Magery, 95.1, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 95.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 102.5, 125.0);
    set_skill_range(w, id, SkillName::Tactics, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Wrestling, 65.0, 87.5);
    fame(w, id, 4000);
    karma(w, id, -4000);
    virtual_armor(w, id, 16);
    if rnd::double(w) < 0.7 {
        pack_new(w, id, "Server.Items.ArcaneGem");
    }
}
fn golem_controller_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GOLEM_CONTROLLER,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    golem_controller_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.GolemController`.
    GOLEM_CONTROLLER, GOLEM_CONTROLLER_C = "Server.Mobiles.GolemController" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(golem_controller_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a golem controller corpse"),
        always_murderer: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
        },
    }
    m {
        click_title: |_w, _id| false,
        show_fame_title: |_w, _id| false,
    }
}

fn ice_fiend_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &ICE_FIEND)
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
    templates::new_creature(w, &IMP)
}
creature_leaf! {
    /// `Server.Mobiles.Imp`.
    IMP, IMP_C = "Server.Mobiles.Imp" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(imp_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: |_w, _id| HideType::Spined,
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::DAEMON,
        can_fly: tp::can_fly,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn lich_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &LICH)
}
creature_leaf! {
    /// `Server.Mobiles.Lich`.
    LICH, LICH_C = "Server.Mobiles.Lich" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lich_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
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
    templates::new_creature(w, &LICH_LORD)
}
static LICH_LORD_ABILITIES: [&MonsterAbility; 1] = [&abilities::SUMMON_SKELETONS_COUNTER];
creature_leaf! {
    /// `Server.Mobiles.LichLord`.
    LICH_LORD, LICH_LORD_C = "Server.Mobiles.LichLord" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lich_lord_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
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
    templates::new_creature(w, &ORCISH_MAGE)
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
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::SAVAGES_AND_ORCS),
        is_enemy: crate::hand::orcish_mage_is_enemy,
        generate_loot: tp::generate_loot,
    }
    m {
        aggressive_action_criminal: crate::hand::orcish_mage_aggressive_action_criminal,
    }
}

fn ratman_mage_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &RATMAN_MAGE)
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
        hide_type: |_w, _id| HideType::Spined,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn savage_shaman_init(w: &mut World, id: MobileId) {
    name_random(w, id, "savage shaman");
    if rnd::bool(w) {
        body(w, id, 184);
    } else {
        body(w, id, 183);
    }
    set_str_range(w, id, 126, 145);
    set_dex_range(w, id, 91, 110);
    set_int_range(w, id, 161, 185);
    set_damage(w, id, 4, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 30);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 50);
    set_skill_range(w, id, SkillName::EvalInt, 77.5, 100.0);
    set_skill_range(w, id, SkillName::Fencing, 62.5, 85.0);
    set_skill_range(w, id, SkillName::Macing, 62.5, 85.0);
    set_skill_range(w, id, SkillName::Magery, 72.5, 95.0);
    set_skill_range(w, id, SkillName::Meditation, 77.5, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 77.5, 100.0);
    set_skill_range(w, id, SkillName::Swords, 62.5, 85.0);
    set_skill_range(w, id, SkillName::Tactics, 62.5, 85.0);
    set_skill_range(w, id, SkillName::Wrestling, 62.5, 85.0);
    fame(w, id, 1000);
    karma(w, id, -1000);
    pack_reg_range(w, id, 10, 15);
    {
        let a0 = "Server.Items.Bandage";
        let a1 = rnd::min_max(w, 1, 15);
        pack_new_amount(w, id, a0, a1);
    }
    if rnd::double(w) < 0.1 {
        pack_new(w, id, "Server.Items.TribalBerry");
    }
    add_new(w, id, "Server.Items.BoneArms");
    add_new(w, id, "Server.Items.BoneLegs");
    add_new(w, id, "Server.Items.DeerMask");
}
fn savage_shaman_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SAVAGE_SHAMAN,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    savage_shaman_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SavageShaman`.
    SAVAGE_SHAMAN, SAVAGE_SHAMAN_C = "Server.Mobiles.SavageShaman" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(savage_shaman_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a savage corpse"),
        meat: |_w, _id| 1,
        always_murderer: |_w, _id| true,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::SAVAGES_AND_ORCS),
        is_enemy: crate::hand::savage_shaman_is_enemy,
        alter_melee_damage_to: crate::hand::savage_shaman_alter_melee_damage_to,
        on_got_melee_attack: crate::hand::savage_shaman_on_got_melee_attack,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
        },
    }
    m {
        show_fame_title: |_w, _id| false,
        aggressive_action_criminal: crate::hand::savage_shaman_aggressive_action_criminal,
    }
}

fn shade_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &SHADE)
}
creature_leaf! {
    /// `Server.Mobiles.Shade`.
    SHADE, SHADE_C = "Server.Mobiles.Shade" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(shade_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn skeletal_mage_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &SKELETAL_MAGE)
}
creature_leaf! {
    /// `Server.Mobiles.SkeletalMage`.
    SKELETAL_MAGE, SKELETAL_MAGE_C = "Server.Mobiles.SkeletalMage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(skeletal_mage_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn spectre_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &SPECTRE)
}
creature_leaf! {
    /// `Server.Mobiles.Spectre`.
    SPECTRE, SPECTRE_C = "Server.Mobiles.Spectre" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(spectre_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn spellbinder_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &SPELLBINDER)
}
creature_leaf! {
    /// `Server.Mobiles.Spellbinder`.
    SPELLBINDER, SPELLBINDER_C = "Server.Mobiles.Spellbinder" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(spellbinder_new), encoded true;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        poison_immune: tp::poison_immune,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
            { let it = muo_content::loot::random_weapon(w, false, false, false); pack_item(w, id, it); }
        },
        forced_ai: crate::hand::spellbinder_forced_ai,
    }
    m {
    }
}

fn succubus_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &SUCCUBUS)
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

pub(crate) fn titan_init(w: &mut World, id: MobileId) {
    body(w, id, 76);
    base_sound_id(w, id, 609);
    set_str_range(w, id, 536, 585);
    set_dex_range(w, id, 126, 145);
    set_int_range(w, id, 281, 305);
    set_hits_range(w, id, 322, 351);
    set_damage(w, id, 13, 16);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::EvalInt, 85.1, 100.0);
    set_skill_range(w, id, SkillName::Magery, 85.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 80.2, 110.0);
    set_skill_range(w, id, SkillName::Tactics, 60.1, 80.0);
    set_skill_range(w, id, SkillName::Wrestling, 40.1, 50.0);
    fame(w, id, 11500);
    karma(w, id, -11500);
    virtual_armor(w, id, 40);
    crate::hand::titan_ctor1(w, id);
}
fn titan_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &TITAN,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    titan_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Titan`.
    TITAN, TITAN_C = "Server.Mobiles.Titan" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(titan_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a titans corpse"),
        default_name: |_w, _id| Some("a titan"),
        meat: |_w, _id| 4,
        poison_immune: |_w, _id| Some(PoisonId(1)),
        treasure_map_level: |_w, _id| 5,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot(w, id, loot_pack::average(w));
            add_loot(w, id, &loot_pack::MED_SCROLLS);
        },
    }
    m {
    }
}

fn wraith_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &WRAITH)
}
creature_leaf! {
    /// `Server.Mobiles.Wraith`.
    WRAITH, WRAITH_C = "Server.Mobiles.Wraith" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(wraith_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        poison_immune: tp::poison_immune,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn arctic_ogre_lord_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &ARCTIC_OGRE_LORD)
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

pub(crate) fn bone_knight_init(w: &mut World, id: MobileId) {
    body(w, id, 57);
    base_sound_id(w, id, 451);
    set_str_range(w, id, 196, 250);
    set_dex_range(w, id, 76, 95);
    set_int_range(w, id, 36, 60);
    set_hits_range(w, id, 118, 150);
    set_damage(w, id, 8, 18);
    set_damage_type(w, id, ResistanceType::Physical, 40);
    set_damage_type(w, id, ResistanceType::Cold, 60);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 60);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::MagicResist, 65.1, 80.0);
    set_skill_range(w, id, SkillName::Tactics, 85.1, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 85.1, 95.0);
    fame(w, id, 3000);
    karma(w, id, -3000);
    virtual_armor(w, id, 40);
    crate::hand::bone_knight_ctor1(w, id);
    pack_slayer(w, id, 0.05);
    pack_new(w, id, "Server.Items.Scimitar");
    pack_new(w, id, "Server.Items.WoodenShield");
}
fn bone_knight_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BONE_KNIGHT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    bone_knight_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.BoneKnight`.
    BONE_KNIGHT, BONE_KNIGHT_C = "Server.Mobiles.BoneKnight" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bone_knight_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a skeletal corpse"),
        default_name: |_w, _id| Some("a bone knight"),
        bleed_immune: |_w, _id| true,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

pub(crate) fn brigand_init(w: &mut World, id: MobileId) {
    {
        let a0 = rnd::from(w, 2, 1000);
        speech_hue(w, id, a0);
    }
    title(w, id, "the brigand");
    {
        let a0 = race_skin_hue(w, "Human");
        hue(w, id, a0);
    }
    let is_f = rnd::bool(w);
    female(w, id, is_f);
    if is_f {
        body(w, id, 0x191);
        name_random(w, id, "female");
        {
            let a0 = "Server.Items.Skirt";
            let a1 = rnd::from(w, 1801, 108);
            add_new_hue(w, id, a0, a1);
        }
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        {
            let a0 = "Server.Items.ShortPants";
            let a1 = rnd::from(w, 1801, 108);
            add_new_hue(w, id, a0, a1);
        }
    }
    set_str_range(w, id, 86, 100);
    set_dex_range(w, id, 81, 95);
    set_int_range(w, id, 61, 75);
    set_damage(w, id, 10, 23);
    set_skill_range(w, id, SkillName::Fencing, 66.0, 97.5);
    set_skill_range(w, id, SkillName::Macing, 65.0, 87.5);
    set_skill_range(w, id, SkillName::MagicResist, 25.0, 47.5);
    set_skill_range(w, id, SkillName::Swords, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Tactics, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Wrestling, 15.0, 37.5);
    fame(w, id, 1000);
    karma(w, id, -1000);
    {
        let a0 = "Server.Items.Boots";
        let a1 = rnd::from(w, 1801, 108);
        add_new_hue(w, id, a0, a1);
    }
    add_new(w, id, "Server.Items.FancyShirt");
    add_new(w, id, "Server.Items.Bandana");
    {
        let pick: Option<&str> = match rnd::below(w, 7) {
            0 => Some("Server.Items.Longsword"),
            1 => Some("Server.Items.Cutlass"),
            2 => Some("Server.Items.Broadsword"),
            3 => Some("Server.Items.Axe"),
            4 => Some("Server.Items.Club"),
            5 => Some("Server.Items.Dagger"),
            _ => Some("Server.Items.Spear"),
        };
        add_opt(w, id, pick);
    }
    assign_random_hair(w, id);
}
fn brigand_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BRIGAND,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    brigand_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Brigand`.
    BRIGAND, BRIGAND_C = "Server.Mobiles.Brigand" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(brigand_new), encoded false;
    c {
        always_murderer: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
        },
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
    templates::new_creature(w, &CHAOS_DAEMON)
}
creature_leaf! {
    /// `Server.Mobiles.ChaosDaemon`.
    CHAOS_DAEMON, CHAOS_DAEMON_C = "Server.Mobiles.ChaosDaemon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(chaos_daemon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        get_weapon_ability: |_w, _id| Some(Ability::CrushingBlow),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn cursed_init(w: &mut World, id: MobileId) {
    title(w, id, "the Cursed");
    {
        let a0 = rnd::min_max(w, 0x8596, 0x8599);
        hue(w, id, a0);
    }
    body(w, id, 0x190);
    name_random(w, id, "male");
    base_sound_id(w, id, 471);
    {
        let a0 = "Server.Items.ShortPants";
        let a1 = rnd::from(w, 1801, 108);
        add_new_hue(w, id, a0, a1);
    }
    {
        let a0 = "Server.Items.Shirt";
        let a1 = rnd::from(w, 1801, 108);
        add_new_hue(w, id, a0, a1);
    }
    crate::hand::cursed_ctor1(w, id);
}
fn cursed_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CURSED,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    cursed_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Cursed`.
    CURSED, CURSED_C = "Server.Mobiles.Cursed" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(cursed_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an inhuman corpse"),
        always_murderer: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
        click_title: |_w, _id| false,
        show_fame_title: |_w, _id| false,
        get_attack_sound: |_w, _id| -1,
    }
}

fn cyclops_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &CYCLOPS)
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
    templates::new_creature(w, &DOPPLEGANGER)
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

pub(crate) fn elf_brigand_init(w: &mut World, id: MobileId) {
    {
        let a0 = rnd::from(w, 2, 1000);
        speech_hue(w, id, a0);
    }
    title(w, id, "the brigand");
    crate::hand::elf_brigand_ctor1(w, id);
    crate::hand::elf_brigand_ctor2(w, id);
    crate::hand::elf_brigand_ctor3(w, id);
    set_str_range(w, id, 86, 100);
    set_dex_range(w, id, 81, 95);
    set_int_range(w, id, 61, 75);
    set_damage(w, id, 10, 23);
    set_skill_range(w, id, SkillName::Fencing, 66.0, 97.5);
    set_skill_range(w, id, SkillName::Macing, 65.0, 87.5);
    set_skill_range(w, id, SkillName::MagicResist, 25.0, 47.5);
    set_skill_range(w, id, SkillName::Swords, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Tactics, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Wrestling, 15.0, 37.5);
    fame(w, id, 1000);
    karma(w, id, -1000);
    {
        let pick: Option<&str> = match rnd::below(w, 4) {
            0 => Some("Server.Items.Boots"),
            1 => Some("Server.Items.ThighBoots"),
            2 => Some("Server.Items.Sandals"),
            _ => Some("Server.Items.Shoes"),
        };
        add_opt(w, id, pick);
    }
    crate::hand::elf_brigand_ctor4(w, id);
    {
        let pick: Option<&str> = match rnd::below(w, 7) {
            0 => Some("Server.Items.Longsword"),
            1 => Some("Server.Items.Cutlass"),
            2 => Some("Server.Items.Broadsword"),
            3 => Some("Server.Items.Axe"),
            4 => Some("Server.Items.Club"),
            5 => Some("Server.Items.Dagger"),
            _ => Some("Server.Items.Spear"),
        };
        add_opt(w, id, pick);
    }
    assign_random_hair(w, id);
}
fn elf_brigand_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ELF_BRIGAND,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    elf_brigand_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.ElfBrigand`.
    ELF_BRIGAND, ELF_BRIGAND_C = "Server.Mobiles.ElfBrigand" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(elf_brigand_new), encoded false;
    c {
        always_murderer: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
        },
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
    templates::new_creature(w, &ENSLAVED_GARGOYLE)
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
    templates::new_creature(w, &ETTIN)
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

pub(crate) fn executioner_init(w: &mut World, id: MobileId) {
    {
        let a0 = rnd::from(w, 2, 1000);
        speech_hue(w, id, a0);
    }
    title(w, id, "the executioner");
    {
        let a0 = race_skin_hue(w, "Human");
        hue(w, id, a0);
    }
    let is_f = rnd::bool(w);
    female(w, id, is_f);
    if is_f {
        body(w, id, 0x191);
        name_random(w, id, "female");
        {
            let a0 = "Server.Items.Skirt";
            let a1 = rnd::from(w, 1601, 54);
            add_new_hue(w, id, a0, a1);
        }
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
        {
            let a0 = "Server.Items.ShortPants";
            let a1 = rnd::from(w, 1601, 54);
            add_new_hue(w, id, a0, a1);
        }
    }
    set_str_range(w, id, 386, 400);
    set_dex_range(w, id, 151, 165);
    set_int_range(w, id, 161, 175);
    set_damage(w, id, 8, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 30);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 20);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 20);
    set_skill(w, id, SkillName::Anatomy, 125.0);
    set_skill_range(w, id, SkillName::Fencing, 46.0, 77.5);
    set_skill_range(w, id, SkillName::Macing, 35.0, 57.5);
    set_skill_range(w, id, SkillName::Poisoning, 60.0, 82.5);
    set_skill_range(w, id, SkillName::MagicResist, 83.5, 92.5);
    set_skill(w, id, SkillName::Swords, 125.0);
    set_skill(w, id, SkillName::Tactics, 125.0);
    set_skill(w, id, SkillName::Lumberjacking, 125.0);
    fame(w, id, 5000);
    karma(w, id, -5000);
    virtual_armor(w, id, 40);
    {
        let a0 = "Server.Items.ThighBoots";
        let a1 = rnd::from(w, 1601, 54);
        add_new_hue(w, id, a0, a1);
    }
    {
        let a0 = "Server.Items.Surcoat";
        let a1 = rnd::from(w, 1601, 54);
        add_new_hue(w, id, a0, a1);
    }
    add_new(w, id, "Server.Items.ExecutionersAxe");
    assign_random_hair(w, id);
}
fn executioner_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &EXECUTIONER,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    executioner_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Executioner`.
    EXECUTIONER, EXECUTIONER_C = "Server.Mobiles.Executioner" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(executioner_new), encoded false;
    c {
        always_murderer: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

fn frost_troll_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &FROST_TROLL)
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
    templates::new_creature(w, &GAZER_LARVA)
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

pub(crate) fn ghoul_init(w: &mut World, id: MobileId) {
    body(w, id, 153);
    base_sound_id(w, id, 0x482);
    set_str_range(w, id, 76, 100);
    set_dex_range(w, id, 76, 95);
    set_int_range(w, id, 36, 60);
    set_hits_range(w, id, 46, 60);
    set_mana(w, id, 0);
    set_damage(w, id, 7, 9);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 30);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 20);
    set_skill_range(w, id, SkillName::MagicResist, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Tactics, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 45.1, 55.0);
    fame(w, id, 2500);
    karma(w, id, -2500);
    virtual_armor(w, id, 28);
    {
        let it = muo_content::loot::random_weapon(w, false, false, false);
        pack_item(w, id, it);
    }
}
fn ghoul_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GHOUL,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ghoul_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Ghoul`.
    GHOUL, GHOUL_C = "Server.Mobiles.Ghoul" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ghoul_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a ghostly corpse"),
        default_name: |_w, _id| Some("a ghoul"),
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(1)),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

fn greater_mongbat_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &GREATER_MONGBAT)
}
creature_leaf! {
    /// `Server.Mobiles.GreaterMongbat`.
    GREATER_MONGBAT, GREATER_MONGBAT_C = "Server.Mobiles.GreaterMongbat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(greater_mongbat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: |_w, _id| FoodType::MEAT,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn guardian_init(w: &mut World, id: MobileId) {
    init_stats(w, id, 100, 125, 25);
    title(w, id, "the guardian");
    {
        let a0 = rnd::from(w, 2, 1000);
        speech_hue(w, id, a0);
    }
    {
        let a0 = race_skin_hue(w, "Human");
        hue(w, id, a0);
    }
    let is_f = rnd::bool(w);
    female(w, id, is_f);
    if is_f {
        body(w, id, 0x191);
        name_random(w, id, "female");
    } else {
        body(w, id, 0x190);
        name_random(w, id, "male");
    }
    crate::hand::guardian_ctor1(w, id);
    crate::hand::guardian_ctor2(w, id);
}
fn guardian_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GUARDIAN,
        AIType::AiArcher,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    guardian_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Guardian`.
    GUARDIAN, GUARDIAN_C = "Server.Mobiles.Guardian" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(guardian_new), encoded false;
    c {
    }
    m {
    }
}

pub(crate) fn headless_one_init(w: &mut World, id: MobileId) {
    body(w, id, 31);
    {
        let a0 = race_skin_hue(w, "Human") & 0x7FFF;
        hue(w, id, a0);
    }
    base_sound_id(w, id, 0x39D);
    set_str_range(w, id, 26, 50);
    set_dex_range(w, id, 36, 55);
    set_int_range(w, id, 16, 30);
    set_hits_range(w, id, 16, 30);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 20.0);
    set_skill_range(w, id, SkillName::Tactics, 25.1, 40.0);
    set_skill_range(w, id, SkillName::Wrestling, 25.1, 40.0);
    fame(w, id, 450);
    karma(w, id, -450);
    virtual_armor(w, id, 18);
    crate::hand::headless_one_ctor1(w, id);
}
fn headless_one_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &HEADLESS_ONE,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    headless_one_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.HeadlessOne`.
    HEADLESS_ONE, HEADLESS_ONE_C = "Server.Mobiles.HeadlessOne" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(headless_one_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a headless corpse"),
        default_name: |_w, _id| Some("a headless one"),
        can_rummage_corpses: |_w, _id| true,
        meat: |_w, _id| 1,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::poor(w));
        },
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
    let id = new_creature(
        w,
        &HORDE_MINION,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    horde_minion_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.HordeMinion`.
    HORDE_MINION, HORDE_MINION_C = "Server.Mobiles.HordeMinion" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(horde_minion_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a horde minion corpse"),
        default_name: |_w, _id| Some("a horde minion"),
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
    templates::new_creature(w, &JUGGERNAUT)
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

pub(crate) fn khaldun_summoner_init(w: &mut World, id: MobileId) {
    body(w, id, 0x190);
    title(w, id, "the Summoner");
    set_str_range(w, id, 351, 400);
    set_dex_range(w, id, 101, 150);
    set_int_range(w, id, 502, 700);
    set_hits_range(w, id, 421, 480);
    set_damage(w, id, 5, 15);
    set_damage_type(w, id, ResistanceType::Physical, 75);
    set_damage_type(w, id, ResistanceType::Cold, 25);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 60);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::Wrestling, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 90.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Magery, 90.1, 100.0);
    set_skill(w, id, SkillName::EvalInt, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 120.1, 130.0);
    virtual_armor(w, id, 36);
    fame(w, id, 10000);
    karma(w, id, -10000);
    crate::hand::khaldun_summoner_ctor1(w, id);
}
fn khaldun_summoner_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &KHALDUN_SUMMONER,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    khaldun_summoner_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.KhaldunSummoner`.
    KHALDUN_SUMMONER, KHALDUN_SUMMONER_C = "Server.Mobiles.KhaldunSummoner" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(khaldun_summoner_new), encoded false;
    c {
        default_name: |_w, _id| Some("Zealot of Khaldun"),
        always_murderer: |_w, _id| true,
        unprovokable: |_w, _id| true,
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

pub(crate) fn khaldun_zealot_init(w: &mut World, id: MobileId) {
    body(w, id, 0x190);
    title(w, id, "the Knight");
    hue(w, id, 0);
    set_str_range(w, id, 351, 400);
    set_dex_range(w, id, 151, 165);
    set_int_range(w, id, 76, 100);
    set_hits_range(w, id, 448, 470);
    set_damage(w, id, 15, 25);
    set_damage_type(w, id, ResistanceType::Physical, 75);
    set_damage_type(w, id, ResistanceType::Cold, 25);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 60);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::Wrestling, 70.1, 80.0);
    set_skill_range(w, id, SkillName::Swords, 120.1, 130.0);
    set_skill_range(w, id, SkillName::Anatomy, 120.1, 130.0);
    set_skill_range(w, id, SkillName::MagicResist, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 90.1, 100.0);
    fame(w, id, 10000);
    karma(w, id, -10000);
    virtual_armor(w, id, 40);
    crate::hand::khaldun_zealot_ctor1(w, id);
}
fn khaldun_zealot_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &KHALDUN_ZEALOT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    khaldun_zealot_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.KhaldunZealot`.
    KHALDUN_ZEALOT, KHALDUN_ZEALOT_C = "Server.Mobiles.KhaldunZealot" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(khaldun_zealot_new), encoded false;
    c {
        default_name: |_w, _id| Some("Zealot of Khaldun"),
        always_murderer: |_w, _id| true,
        unprovokable: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(3)),
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
    templates::new_creature(w, &MOLOCH)
}
creature_leaf! {
    /// `Server.Mobiles.Moloch`.
    MOLOCH, MOLOCH_C = "Server.Mobiles.Moloch" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(moloch_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        get_weapon_ability: |_w, _id| Some(Ability::ConcussionBlow),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn mongbat_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &MONGBAT)
}
creature_leaf! {
    /// `Server.Mobiles.Mongbat`.
    MONGBAT, MONGBAT_C = "Server.Mobiles.Mongbat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(mongbat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_fly: tp::can_fly,
        meat: tp::meat,
        favorite_food: |_w, _id| FoodType::MEAT,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn mummy_init(w: &mut World, id: MobileId) {
    body(w, id, 154);
    base_sound_id(w, id, 471);
    set_str_range(w, id, 346, 370);
    set_dex_range(w, id, 71, 90);
    set_int_range(w, id, 26, 40);
    set_hits_range(w, id, 208, 222);
    set_damage(w, id, 13, 23);
    set_damage_type(w, id, ResistanceType::Physical, 40);
    set_damage_type(w, id, ResistanceType::Cold, 60);
    set_resistance_range(w, id, ResistanceType::Physical, 45, 55);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 20);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 60);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 40.0);
    set_skill_range(w, id, SkillName::Tactics, 35.1, 50.0);
    set_skill_range(w, id, SkillName::Wrestling, 35.1, 50.0);
    fame(w, id, 4000);
    karma(w, id, -4000);
    virtual_armor(w, id, 50);
    crate::hand::mummy_ctor1(w, id);
    pack_new_amount(w, id, "Server.Items.Garlic", 5);
    pack_new_amount(w, id, "Server.Items.Bandage", 10);
}
fn mummy_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &MUMMY,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    mummy_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Mummy`.
    MUMMY, MUMMY_C = "Server.Mobiles.Mummy" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(mummy_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a mummy corpse"),
        default_name: |_w, _id| Some("a mummy"),
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(0)),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
            add_loot(w, id, &loot_pack::GEMS);
            add_loot(w, id, &loot_pack::POTIONS);
        },
    }
    m {
    }
}

fn ogre_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &OGRE)
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
    templates::new_creature(w, &OGRE_LORD)
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

pub(crate) fn orc_init(w: &mut World, id: MobileId) {
    name_random(w, id, "orc");
    body(w, id, 17);
    base_sound_id(w, id, 0x45A);
    set_str_range(w, id, 96, 120);
    set_dex_range(w, id, 81, 105);
    set_int_range(w, id, 36, 60);
    set_hits_range(w, id, 58, 72);
    set_damage(w, id, 5, 7);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 30);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 20);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 20);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);
    set_skill_range(w, id, SkillName::MagicResist, 50.1, 75.0);
    set_skill_range(w, id, SkillName::Tactics, 55.1, 80.0);
    set_skill_range(w, id, SkillName::Wrestling, 50.1, 70.0);
    fame(w, id, 1500);
    karma(w, id, -1500);
    virtual_armor(w, id, 28);
    {
        let pick: Option<&str> = match rnd::below(w, 20) {
            0 => Some("Server.Items.Scimitar"),
            1 => Some("Server.Items.Katana"),
            2 => Some("Server.Items.WarMace"),
            3 => Some("Server.Items.WarHammer"),
            4 => Some("Server.Items.Kryss"),
            5 => Some("Server.Items.Pitchfork"),
            _ => None,
        };
        pack_opt(w, id, pick);
    }
    pack_new(w, id, "Server.Items.ThighBoots");
    {
        let pick: Option<&str> = match rnd::below(w, 3) {
            0 => Some("Server.Items.Ribs"),
            1 => Some("Server.Items.Shaft"),
            _ => Some("Server.Items.Candle"),
        };
        pack_opt(w, id, pick);
    }
    if rnd::double(w) < 0.2 {
        pack_new(w, id, "Server.Items.BolaBall");
    }
}
fn orc_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ORC,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    orc_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Orc`.
    ORC, ORC_C = "Server.Mobiles.Orc" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(orc_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an orcish corpse"),
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::ORC),
        can_rummage_corpses: |_w, _id| true,
        treasure_map_level: |_w, _id| 1,
        meat: |_w, _id| 1,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::SAVAGES_AND_ORCS),
        is_enemy: crate::hand::orc_is_enemy,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
        aggressive_action_criminal: crate::hand::orc_aggressive_action_criminal,
    }
}

fn orc_bomber_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &ORC_BOMBER)
}
creature_leaf! {
    /// `Server.Mobiles.OrcBomber`.
    ORC_BOMBER, ORC_BOMBER_C = "Server.Mobiles.OrcBomber" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(orc_bomber_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::ORC),
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::SAVAGES_AND_ORCS),
        is_enemy: crate::hand::orc_bomber_is_enemy,
        on_action_combat: crate::hand::orc_bomber_on_action_combat,
        generate_loot: tp::generate_loot,
    }
    m {
        aggressive_action_criminal: crate::hand::orc_bomber_aggressive_action_criminal,
    }
}

fn orc_brute_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &ORC_BRUTE)
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
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::SAVAGES_AND_ORCS),
        can_rummage_corpses: tp::can_rummage_corpses,
        auto_dispel: |_w, _id| true,
        is_enemy: crate::hand::orc_brute_is_enemy,
        on_damaged_by_spell: crate::hand::orc_brute_on_damaged_by_spell,
        generate_loot: tp::generate_loot,
    }
    m {
        aggressive_action_criminal: crate::hand::orc_brute_aggressive_action_criminal,
    }
}

pub(crate) fn orc_captain_init(w: &mut World, id: MobileId) {
    name_random(w, id, "orc");
    body(w, id, 7);
    base_sound_id(w, id, 0x45A);
    set_str_range(w, id, 111, 145);
    set_dex_range(w, id, 101, 135);
    set_int_range(w, id, 86, 110);
    set_hits_range(w, id, 67, 87);
    set_damage(w, id, 5, 15);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 20);
    set_resistance_range(w, id, ResistanceType::Cold, 15, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 70.1, 85.0);
    set_skill_range(w, id, SkillName::Swords, 70.1, 95.0);
    set_skill_range(w, id, SkillName::Tactics, 85.1, 100.0);
    fame(w, id, 2500);
    karma(w, id, -2500);
    virtual_armor(w, id, 34);
    crate::hand::orc_captain_ctor1(w, id);
    if w.core_aos() {
        {
            let it = muo_content::loot::random_necromancy_reagent(w);
            pack_item(w, id, it);
        }
    }
}
fn orc_captain_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ORC_CAPTAIN,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    orc_captain_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.OrcCaptain`.
    ORC_CAPTAIN, ORC_CAPTAIN_C = "Server.Mobiles.OrcCaptain" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(orc_captain_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an orcish corpse"),
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::ORC),
        can_rummage_corpses: |_w, _id| true,
        meat: |_w, _id| 1,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::SAVAGES_AND_ORCS),
        is_enemy: crate::hand::orc_captain_is_enemy,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::meager(w), 2);
        },
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

pub(crate) fn orcish_lord_init(w: &mut World, id: MobileId) {
    body(w, id, 138);
    base_sound_id(w, id, 0x45A);
    set_str_range(w, id, 147, 215);
    set_dex_range(w, id, 91, 115);
    set_int_range(w, id, 61, 85);
    set_hits_range(w, id, 95, 123);
    set_damage(w, id, 4, 14);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 30);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::MagicResist, 70.1, 85.0);
    set_skill_range(w, id, SkillName::Swords, 60.1, 85.0);
    set_skill_range(w, id, SkillName::Tactics, 75.1, 90.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 85.0);
    fame(w, id, 2500);
    karma(w, id, -2500);
    {
        let pick: Option<&str> = match rnd::below(w, 5) {
            0 => Some("Server.Items.Lockpick"),
            1 => Some("Server.Items.MortarPestle"),
            2 => Some("Server.Items.Bottle"),
            3 => Some("Server.Items.RawRibs"),
            _ => Some("Server.Items.Shovel"),
        };
        pack_opt(w, id, pick);
    }
    pack_new(w, id, "Server.Items.RingmailChest");
    if rnd::double(w) < 0.3 {
        {
            let it = muo_content::loot::random_possible_reagent(w);
            pack_item(w, id, it);
        }
    }
    if rnd::double(w) < 0.2 {
        pack_new(w, id, "Server.Items.BolaBall");
    }
}
fn orcish_lord_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ORCISH_LORD,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    orcish_lord_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.OrcishLord`.
    ORCISH_LORD, ORCISH_LORD_C = "Server.Mobiles.OrcishLord" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(orcish_lord_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an orcish corpse"),
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::ORC),
        default_name: |_w, _id| Some("an orcish lord"),
        can_rummage_corpses: |_w, _id| true,
        treasure_map_level: |_w, _id| 1,
        meat: |_w, _id| 1,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::SAVAGES_AND_ORCS),
        is_enemy: crate::hand::orcish_lord_is_enemy,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
            add_loot(w, id, loot_pack::average(w));
        },
    }
    m {
        aggressive_action_criminal: crate::hand::orcish_lord_aggressive_action_criminal,
    }
}

fn ratman_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &RATMAN)
}
creature_leaf! {
    /// `Server.Mobiles.Ratman`.
    RATMAN, RATMAN_C = "Server.Mobiles.Ratman" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ratman_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::RATMAN),
        can_rummage_corpses: tp::can_rummage_corpses,
        hides: tp::hides,
        hide_type: |_w, _id| HideType::Spined,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ratman_archer_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &RATMAN_ARCHER)
}
creature_leaf! {
    /// `Server.Mobiles.RatmanArcher`.
    RATMAN_ARCHER, RATMAN_ARCHER_C = "Server.Mobiles.RatmanArcher" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ratman_archer_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::RATMAN),
        can_rummage_corpses: tp::can_rummage_corpses,
        hides: tp::hides,
        hide_type: |_w, _id| HideType::Spined,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn restless_soul_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &RESTLESS_SOUL)
}
creature_leaf! {
    /// `Server.Mobiles.RestlessSoul`.
    RESTLESS_SOUL, RESTLESS_SOUL_C = "Server.Mobiles.RestlessSoul" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(restless_soul_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        always_attackable: |_w, _id| true,
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
    templates::new_creature(w, &ROTTING_CORPSE)
}
creature_leaf! {
    /// `Server.Mobiles.RottingCorpse`.
    ROTTING_CORPSE, ROTTING_CORPSE_C = "Server.Mobiles.RottingCorpse" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(rotting_corpse_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        poison_immune: tp::poison_immune,
        hit_poison: |_w, _id| Some(PoisonId(4)),
        treasure_map_level: tp::treasure_map_level,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn savage_init(w: &mut World, id: MobileId) {
    name_random(w, id, "savage");
    let is_f = rnd::bool(w);
    female(w, id, is_f);
    if is_f {
        body(w, id, 184);
    } else {
        body(w, id, 183);
    }
    set_str_range(w, id, 96, 115);
    set_dex_range(w, id, 86, 105);
    set_int_range(w, id, 51, 65);
    set_damage(w, id, 23, 27);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_skill_range(w, id, SkillName::Fencing, 60.0, 82.5);
    set_skill_range(w, id, SkillName::Macing, 60.0, 82.5);
    set_skill_range(w, id, SkillName::Poisoning, 60.0, 82.5);
    set_skill_range(w, id, SkillName::MagicResist, 57.5, 80.0);
    set_skill_range(w, id, SkillName::Swords, 60.0, 82.5);
    set_skill_range(w, id, SkillName::Tactics, 60.0, 82.5);
    fame(w, id, 1000);
    karma(w, id, -1000);
    {
        let a0 = "Server.Items.Bandage";
        let a1 = rnd::min_max(w, 1, 15);
        pack_new_amount(w, id, a0, a1);
    }
    if is_female(w, id) && rnd::double(w) < 0.1 {
        pack_new(w, id, "Server.Items.TribalBerry");
    } else {
        if !is_female(w, id) && rnd::double(w) < 0.1 {
            pack_new(w, id, "Server.Items.BolaBall");
        }
    }
}
fn savage_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SAVAGE,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    savage_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Savage`.
    SAVAGE, SAVAGE_C = "Server.Mobiles.Savage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(savage_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a savage corpse"),
        meat: |_w, _id| 1,
        always_murderer: |_w, _id| true,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::SAVAGES_AND_ORCS),
        is_enemy: crate::hand::savage_is_enemy,
        alter_melee_damage_to: crate::hand::savage_alter_melee_damage_to,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
        show_fame_title: |_w, _id| false,
        aggressive_action_criminal: crate::hand::savage_aggressive_action_criminal,
    }
}

pub(crate) fn savage_rider_init(w: &mut World, id: MobileId) {
    name_random(w, id, "savage rider");
    let is_f = rnd::bool(w);
    female(w, id, is_f);
    if is_f {
        body(w, id, 186);
    } else {
        body(w, id, 185);
    }
    set_str_range(w, id, 151, 170);
    set_dex_range(w, id, 92, 130);
    set_int_range(w, id, 51, 65);
    set_damage(w, id, 29, 34);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_skill_range(w, id, SkillName::Fencing, 72.5, 95.0);
    set_skill_range(w, id, SkillName::Healing, 60.3, 90.0);
    set_skill_range(w, id, SkillName::Macing, 72.5, 95.0);
    set_skill_range(w, id, SkillName::Poisoning, 60.0, 82.5);
    set_skill_range(w, id, SkillName::MagicResist, 72.5, 95.0);
    set_skill_range(w, id, SkillName::Swords, 72.5, 95.0);
    set_skill_range(w, id, SkillName::Tactics, 72.5, 95.0);
    fame(w, id, 1000);
    karma(w, id, -1000);
    {
        let a0 = "Server.Items.Bandage";
        let a1 = rnd::min_max(w, 1, 15);
        pack_new_amount(w, id, a0, a1);
    }
    if rnd::double(w) < 0.1 {
        pack_new(w, id, "Server.Items.BolaBall");
    }
    add_new(w, id, "Server.Items.TribalSpear");
    add_new(w, id, "Server.Items.BoneArms");
    add_new(w, id, "Server.Items.BoneLegs");
    crate::hand::savage_rider_ctor1(w, id);
}
fn savage_rider_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SAVAGE_RIDER,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    savage_rider_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SavageRider`.
    SAVAGE_RIDER, SAVAGE_RIDER_C = "Server.Mobiles.SavageRider" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(savage_rider_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a savage corpse"),
        meat: |_w, _id| 1,
        always_murderer: |_w, _id| true,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::SAVAGES_AND_ORCS),
        is_enemy: crate::hand::savage_rider_is_enemy,
        alter_melee_damage_to: crate::hand::savage_rider_alter_melee_damage_to,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
        },
    }
    m {
        show_fame_title: |_w, _id| false,
        on_before_death: crate::hand::savage_rider_on_before_death,
        aggressive_action_criminal: crate::hand::savage_rider_aggressive_action_criminal,
    }
}

pub(crate) fn shadow_fiend_init(w: &mut World, id: MobileId) {
    body(w, id, 0xA8);
    crate::hand::shadow_fiend_ctor1(w, id);
}
fn shadow_fiend_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SHADOW_FIEND,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    shadow_fiend_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.ShadowFiend`.
    SHADOW_FIEND, SHADOW_FIEND_C = "Server.Mobiles.ShadowFiend" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(shadow_fiend_new), state crate::hand::shadow_fiend_serialize, crate::hand::shadow_fiend_deserialize;
    c {
        delete_corpse_on_death: |_w, _id| true,
        default_name: |_w, _id| Some("a shadow fiend"),
        can_rummage_corpses: |_w, _id| true,
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

pub(crate) fn skeletal_knight_init(w: &mut World, id: MobileId) {
    body(w, id, 147);
    base_sound_id(w, id, 451);
    set_str_range(w, id, 196, 250);
    set_dex_range(w, id, 76, 95);
    set_int_range(w, id, 36, 60);
    set_hits_range(w, id, 118, 150);
    set_damage(w, id, 8, 18);
    set_damage_type(w, id, ResistanceType::Physical, 40);
    set_damage_type(w, id, ResistanceType::Cold, 60);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 60);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::MagicResist, 65.1, 80.0);
    set_skill_range(w, id, SkillName::Tactics, 85.1, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 85.1, 95.0);
    fame(w, id, 3000);
    karma(w, id, -3000);
    virtual_armor(w, id, 40);
    crate::hand::skeletal_knight_ctor1(w, id);
    pack_new(w, id, "Server.Items.Scimitar");
    pack_new(w, id, "Server.Items.WoodenShield");
}
fn skeletal_knight_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SKELETAL_KNIGHT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    skeletal_knight_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SkeletalKnight`.
    SKELETAL_KNIGHT, SKELETAL_KNIGHT_C = "Server.Mobiles.SkeletalKnight" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(skeletal_knight_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a skeletal corpse"),
        default_name: |_w, _id| Some("a skeletal knight"),
        bleed_immune: |_w, _id| true,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

pub(crate) fn skeleton_init(w: &mut World, id: MobileId) {
    body_list(w, id, &[50, 56]);
    base_sound_id(w, id, 0x48D);
    set_str_range(w, id, 56, 80);
    set_dex_range(w, id, 56, 75);
    set_int_range(w, id, 16, 40);
    set_hits_range(w, id, 34, 48);
    set_damage(w, id, 3, 7);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 5, 15);
    set_skill_range(w, id, SkillName::MagicResist, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Tactics, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 45.1, 55.0);
    fame(w, id, 450);
    karma(w, id, -450);
    virtual_armor(w, id, 16);
    crate::hand::skeleton_ctor1(w, id);
}
fn skeleton_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SKELETON,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    skeleton_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Skeleton`.
    SKELETON, SKELETON_C = "Server.Mobiles.Skeleton" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(skeleton_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a skeletal corpse"),
        default_name: |_w, _id| Some("a skeleton"),
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(0)),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::poor(w));
        },
    }
    m {
    }
}

pub(crate) fn spawned_orcish_lord_init(w: &mut World, id: MobileId) {
    orcish_lord_init(w, id);
    crate::hand::spawned_orcish_lord_ctor1(w, id);
}
fn spawned_orcish_lord_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SPAWNED_ORCISH_LORD,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    spawned_orcish_lord_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SpawnedOrcishLord`.
    SPAWNED_ORCISH_LORD, SPAWNED_ORCISH_LORD_C = "Server.Mobiles.SpawnedOrcishLord" : ORCISH_LORD / ORCISH_LORD_C, ctor Some(spawned_orcish_lord_new), state crate::hand::evil_mage_serialize, crate::hand::spawned_orcish_lord_deserialize;
    c {
        corpse_name: |_w, _id| Some("an orcish corpse"),
    }
    m {
        on_death: crate::hand::spawned_orcish_lord_on_death,
    }
}

pub(crate) fn spectral_armour_init(w: &mut World, id: MobileId) {
    body(w, id, 637);
    hue(w, id, 0x8026);
    crate::hand::spectral_armour_ctor1(w, id);
    crate::hand::spectral_armour_ctor2(w, id);
    crate::hand::spectral_armour_ctor3(w, id);
    set_str_range(w, id, 101, 110);
    set_dex_range(w, id, 101, 110);
    set_int_range(w, id, 101, 110);
    set_hits_range(w, id, 178, 201);
    set_stam_range(w, id, 191, 200);
    set_damage(w, id, 10, 22);
    set_damage_type(w, id, ResistanceType::Physical, 75);
    set_damage_type(w, id, ResistanceType::Cold, 25);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);
    set_skill_range(w, id, SkillName::Wrestling, 75.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 90.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 90.1, 100.0);
    virtual_armor(w, id, 40);
    fame(w, id, 7000);
    karma(w, id, -7000);
}
fn spectral_armour_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SPECTRAL_ARMOUR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    spectral_armour_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SpectralArmour`.
    SPECTRAL_ARMOUR, SPECTRAL_ARMOUR_C = "Server.Mobiles.SpectralArmour" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(spectral_armour_new), encoded false;
    c {
        delete_corpse_on_death: |_w, _id| true,
        default_name: |_w, _id| Some("a spectral armour"),
        poison_immune: |_w, _id| Some(PoisonId(1)),
    }
    m {
        get_idle_sound: |_w, _id| 0x200,
        get_anger_sound: |_w, _id| 0x56,
        on_before_death: crate::hand::spectral_armour_on_before_death,
    }
}

fn stone_gargoyle_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &STONE_GARGOYLE)
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
    templates::new_creature(w, &STRONG_MONGBAT)
}
creature_leaf! {
    /// `Server.Mobiles.StrongMongbat`.
    STRONG_MONGBAT, STRONG_MONGBAT_C = "Server.Mobiles.StrongMongbat" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(strong_mongbat_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        favorite_food: |_w, _id| FoodType::MEAT,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn troll_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &TROLL)
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

pub(crate) fn zombie_init(w: &mut World, id: MobileId) {
    body(w, id, 3);
    base_sound_id(w, id, 471);
    set_str_range(w, id, 46, 70);
    set_dex_range(w, id, 31, 50);
    set_int_range(w, id, 26, 40);
    set_hits_range(w, id, 28, 42);
    set_damage(w, id, 3, 7);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Cold, 20, 30);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 40.0);
    set_skill_range(w, id, SkillName::Tactics, 35.1, 50.0);
    set_skill_range(w, id, SkillName::Wrestling, 35.1, 50.0);
    fame(w, id, 600);
    karma(w, id, -600);
    virtual_armor(w, id, 18);
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
}
fn zombie_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ZOMBIE,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    zombie_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Zombie`.
    ZOMBIE, ZOMBIE_C = "Server.Mobiles.Zombie" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(zombie_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a rotting corpse"),
        default_name: |_w, _id| Some("a zombie"),
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(1)),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

pub(crate) fn exodus_minion_init(w: &mut World, id: MobileId) {
    body(w, id, 0x2F5);
    set_str_range(w, id, 851, 950);
    set_dex_range(w, id, 71, 80);
    set_int_range(w, id, 61, 90);
    set_hits_range(w, id, 511, 570);
    set_damage(w, id, 16, 22);
    set_resistance_range(w, id, ResistanceType::Physical, 60, 70);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 15, 25);
    set_resistance_range(w, id, ResistanceType::Poison, 15, 25);
    set_resistance_range(w, id, ResistanceType::Energy, 15, 25);
    set_skill_range(w, id, SkillName::MagicResist, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 90.1, 100.0);
    fame(w, id, 18000);
    karma(w, id, -18000);
    virtual_armor(w, id, 65);
    pack_new(w, id, "Server.Items.PowerCrystal");
    pack_new(w, id, "Server.Items.ArcaneGem");
    pack_new(w, id, "Server.Items.ClockworkAssembly");
    crate::hand::exodus_minion_ctor1(w, id);
}
fn exodus_minion_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &EXODUS_MINION,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    exodus_minion_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.ExodusMinion`.
    EXODUS_MINION, EXODUS_MINION_C = "Server.Mobiles.ExodusMinion" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(exodus_minion_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a minion's corpse"),
        is_scared_of_scary_things: |_w, _id| false,
        is_scary_to_pets: |_w, _id| true,
        default_name: |_w, _id| Some("an exodus minion"),
        auto_dispel: |_w, _id| true,
        bard_immune: |w, _id| !w.core_aos(),
        poison_immune: |_w, _id| Some(PoisonId(4)),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
            add_loot(w, id, loot_pack::rich(w));
        },
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

pub(crate) fn exodus_overseer_init(w: &mut World, id: MobileId) {
    body(w, id, 0x2F4);
    set_str_range(w, id, 561, 650);
    set_dex_range(w, id, 76, 95);
    set_int_range(w, id, 61, 90);
    set_hits_range(w, id, 331, 390);
    set_damage(w, id, 13, 19);
    set_damage_type(w, id, ResistanceType::Physical, 50);
    set_damage_type(w, id, ResistanceType::Energy, 50);
    set_resistance_range(w, id, ResistanceType::Physical, 45, 55);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 60);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::MagicResist, 80.2, 98.0);
    set_skill_range(w, id, SkillName::Tactics, 80.2, 98.0);
    set_skill_range(w, id, SkillName::Wrestling, 80.2, 98.0);
    fame(w, id, 10000);
    karma(w, id, -10000);
    virtual_armor(w, id, 50);
    if rnd::below(w, 2) == 0 {
        pack_new(w, id, "Server.Items.PowerCrystal");
    } else {
        pack_new(w, id, "Server.Items.ArcaneGem");
    }
}
fn exodus_overseer_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &EXODUS_OVERSEER,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    exodus_overseer_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.ExodusOverseer`.
    EXODUS_OVERSEER, EXODUS_OVERSEER_C = "Server.Mobiles.ExodusOverseer" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(exodus_overseer_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an overseer's corpse"),
        is_scared_of_scary_things: |_w, _id| false,
        is_scary_to_pets: |_w, _id| true,
        default_name: |_w, _id| Some("an exodus overseer"),
        auto_dispel: |_w, _id| true,
        bard_immune: |w, _id| !w.core_aos(),
        poison_immune: |_w, _id| Some(PoisonId(4)),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
        },
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

pub(crate) fn chaos_dragoon_init(w: &mut World, id: MobileId) {
    body(w, id, 0x190);
    {
        let a0 = race_skin_hue(w, "Human");
        hue(w, id, a0);
    }
    crate::hand::chaos_dragoon_ctor1(w, id);
    set_str_range(w, id, 176, 225);
    set_dex_range(w, id, 81, 95);
    set_int_range(w, id, 61, 85);
    set_hits_range(w, id, 176, 225);
    set_damage(w, id, 24, 26);
    set_damage_type(w, id, ResistanceType::Physical, 25);
    set_damage_type(w, id, ResistanceType::Fire, 25);
    set_damage_type(w, id, ResistanceType::Cold, 25);
    set_damage_type(w, id, ResistanceType::Energy, 25);
    set_skill_range(w, id, SkillName::Fencing, 77.6, 92.5);
    set_skill_range(w, id, SkillName::Healing, 60.3, 90.0);
    set_skill_range(w, id, SkillName::Macing, 77.6, 92.5);
    set_skill_range(w, id, SkillName::Anatomy, 77.6, 87.5);
    set_skill_range(w, id, SkillName::MagicResist, 77.6, 97.5);
    set_skill_range(w, id, SkillName::Swords, 77.6, 92.5);
    set_skill_range(w, id, SkillName::Tactics, 77.6, 87.5);
    fame(w, id, 5000);
    karma(w, id, -5000);
    crate::hand::chaos_dragoon_ctor2(w, id);
}
fn chaos_dragoon_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CHAOS_DRAGOON,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    chaos_dragoon_init(w, id);
    id
}
static CHAOS_DRAGOON_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.ChaosDragoon`.
    CHAOS_DRAGOON, CHAOS_DRAGOON_C = "Server.Mobiles.ChaosDragoon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(chaos_dragoon_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a chaos dragoon corpse"),
        default_name: |_w, _id| Some("a chaos dragoon"),
        auto_dispel: |_w, _id| true,
        bard_immune: |w, _id| !w.core_aos(),
        can_rummage_corpses: |_w, _id| true,
        always_murderer: |_w, _id| true,
        alter_melee_damage_to: crate::hand::chaos_dragoon_alter_melee_damage_to,
        get_monster_abilities: |_, _| &CHAOS_DRAGOON_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
        },
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

pub(crate) fn chaos_dragoon_elite_init(w: &mut World, id: MobileId) {
    body(w, id, 0x190);
    {
        let a0 = race_skin_hue(w, "Human");
        hue(w, id, a0);
    }
    crate::hand::chaos_dragoon_elite_ctor1(w, id);
    set_str_range(w, id, 276, 350);
    set_dex_range(w, id, 66, 90);
    set_int_range(w, id, 126, 150);
    set_hits_range(w, id, 276, 350);
    set_damage(w, id, 29, 34);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_skill_range(w, id, SkillName::Tactics, 80.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 100.1, 110.0);
    set_skill_range(w, id, SkillName::Anatomy, 80.1, 100.0);
    set_skill_range(w, id, SkillName::Magery, 85.1, 100.0);
    set_skill_range(w, id, SkillName::EvalInt, 85.1, 100.0);
    set_skill_range(w, id, SkillName::Swords, 72.5, 95.0);
    set_skill_range(w, id, SkillName::Fencing, 85.1, 100.0);
    set_skill_range(w, id, SkillName::Macing, 85.1, 100.0);
    fame(w, id, 8000);
    karma(w, id, -8000);
    crate::hand::chaos_dragoon_elite_ctor2(w, id);
}
fn chaos_dragoon_elite_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CHAOS_DRAGOON_ELITE,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    chaos_dragoon_elite_init(w, id);
    id
}
static CHAOS_DRAGOON_ELITE_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.ChaosDragoonElite`.
    CHAOS_DRAGOON_ELITE, CHAOS_DRAGOON_ELITE_C = "Server.Mobiles.ChaosDragoonElite" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(chaos_dragoon_elite_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a chaos dragoon elite corpse"),
        default_name: |_w, _id| Some("a chaos dragoon elite"),
        auto_dispel: |_w, _id| true,
        bard_immune: |w, _id| !w.core_aos(),
        can_rummage_corpses: |_w, _id| true,
        always_murderer: |_w, _id| true,
        alter_melee_damage_to: crate::hand::chaos_dragoon_elite_alter_melee_damage_to,
        get_monster_abilities: |_, _| &CHAOS_DRAGOON_ELITE_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
            add_loot(w, id, &loot_pack::GEMS);
        },
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

pub(crate) fn juka_lord_init(w: &mut World, id: MobileId) {
    body(w, id, 766);
    set_str_range(w, id, 401, 500);
    set_dex_range(w, id, 81, 100);
    set_int_range(w, id, 151, 200);
    set_hits_range(w, id, 241, 300);
    set_damage(w, id, 10, 12);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 50);
    set_resistance_range(w, id, ResistanceType::Fire, 45, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 40, 50);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 25);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 50);
    set_skill_range(w, id, SkillName::Anatomy, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Archery, 95.1, 100.0);
    set_skill_range(w, id, SkillName::Healing, 80.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 120.1, 130.0);
    set_skill_range(w, id, SkillName::Swords, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 95.1, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 90.1, 100.0);
    fame(w, id, 15000);
    karma(w, id, -15000);
    virtual_armor(w, id, 28);
    crate::hand::juka_lord_ctor1(w, id);
}
fn juka_lord_new(w: &mut World) -> MobileId {
    let id = new_creature(w, &JUKA_LORD, AIType::AiArcher, FightMode::Closest, 10, 3);
    juka_lord_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.JukaLord`.
    JUKA_LORD, JUKA_LORD_C = "Server.Mobiles.JukaLord" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(juka_lord_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a jukan corpse"),
        default_name: |_w, _id| Some("a juka lord"),
        always_murderer: |_w, _id| true,
        bard_immune: |w, _id| !w.core_aos(),
        can_rummage_corpses: |_w, _id| true,
        meat: |_w, _id| 1,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
            add_loot(w, id, loot_pack::average(w));
        },
    }
    m {
        on_damage: crate::hand::juka_lord_on_damage,
        get_idle_sound: |_w, _id| 0x262,
        get_anger_sound: |_w, _id| 0x263,
        get_hurt_sound: |_w, _id| 0x1D0,
        get_death_sound: |_w, _id| 0x28D,
    }
}

pub(crate) fn juka_mage_init(w: &mut World, id: MobileId) {
    body(w, id, 765);
    set_str_range(w, id, 201, 300);
    set_dex_range(w, id, 71, 90);
    set_int_range(w, id, 451, 500);
    set_hits_range(w, id, 121, 180);
    set_damage(w, id, 4, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 20, 30);
    set_resistance_range(w, id, ResistanceType::Fire, 35, 45);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 20);
    set_resistance_range(w, id, ResistanceType::Energy, 35, 45);
    set_skill_range(w, id, SkillName::Anatomy, 80.1, 90.0);
    set_skill_range(w, id, SkillName::EvalInt, 80.2, 100.0);
    set_skill_range(w, id, SkillName::Magery, 99.1, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 80.2, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 140.1, 150.0);
    set_skill_range(w, id, SkillName::Tactics, 80.1, 90.0);
    set_skill_range(w, id, SkillName::Wrestling, 80.1, 90.0);
    fame(w, id, 15000);
    karma(w, id, -15000);
    virtual_armor(w, id, 16);
    crate::hand::juka_mage_ctor1(w, id);
}
fn juka_mage_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &JUKA_MAGE,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    juka_mage_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.JukaMage`.
    JUKA_MAGE, JUKA_MAGE_C = "Server.Mobiles.JukaMage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(juka_mage_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a jukan corpse"),
        default_name: |_w, _id| Some("a juka mage"),
        always_murderer: |_w, _id| true,
        can_rummage_corpses: |_w, _id| true,
        meat: |_w, _id| 1,
        on_think: crate::hand::juka_mage_on_think,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::average(w), 2);
            add_loot_n(w, id, &loot_pack::MED_SCROLLS, 2);
        },
    }
    m {
        get_idle_sound: |_w, _id| 0x1AC,
        get_anger_sound: |_w, _id| 0x1CD,
        get_hurt_sound: |_w, _id| 0x1D0,
        get_death_sound: |_w, _id| 0x28D,
    }
}

fn juka_warrior_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &JUKA_WARRIOR)
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

pub(crate) fn meer_captain_init(w: &mut World, id: MobileId) {
    body(w, id, 773);
    set_str_range(w, id, 96, 110);
    set_dex_range(w, id, 186, 200);
    set_int_range(w, id, 96, 110);
    set_hits_range(w, id, 58, 66);
    set_damage(w, id, 5, 15);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 45, 55);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 20);
    set_resistance_range(w, id, ResistanceType::Cold, 40, 50);
    set_resistance_range(w, id, ResistanceType::Poison, 35, 45);
    set_resistance_range(w, id, ResistanceType::Energy, 35, 45);
    set_skill_range(w, id, SkillName::Archery, 90.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 91.0, 100.0);
    set_skill_range(w, id, SkillName::Swords, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 91.0, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 80.9, 89.9);
    fame(w, id, 2000);
    karma(w, id, 5000);
    virtual_armor(w, id, 28);
    crate::hand::meer_captain_ctor1(w, id);
}
fn meer_captain_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &MEER_CAPTAIN,
        AIType::AiArcher,
        FightMode::Evil,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    meer_captain_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.MeerCaptain`.
    MEER_CAPTAIN, MEER_CAPTAIN_C = "Server.Mobiles.MeerCaptain" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(meer_captain_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a meer corpse"),
        default_name: |_w, _id| Some("a meer captain"),
        bard_immune: |w, _id| !w.core_aos(),
        can_rummage_corpses: |_w, _id| true,
        initial_innocent: |_w, _id| true,
        on_think: crate::hand::meer_captain_on_think,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
        get_hurt_sound: |_w, _id| 0x14D,
        get_death_sound: |_w, _id| 0x314,
        get_attack_sound: |_w, _id| 0x75,
    }
}

pub(crate) fn meer_eternal_init(w: &mut World, id: MobileId) {
    body(w, id, 772);
    set_str_range(w, id, 416, 505);
    set_dex_range(w, id, 146, 165);
    set_int_range(w, id, 566, 655);
    set_hits_range(w, id, 250, 303);
    set_damage(w, id, 11, 13);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 45, 55);
    set_resistance_range(w, id, ResistanceType::Fire, 15, 25);
    set_resistance_range(w, id, ResistanceType::Cold, 45, 55);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::EvalInt, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Magery, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 90.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 150.5, 200.0);
    set_skill_range(w, id, SkillName::Tactics, 50.1, 70.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 80.0);
    fame(w, id, 18000);
    karma(w, id, 18000);
    virtual_armor(w, id, 34);
    crate::hand::meer_eternal_ctor1(w, id);
}
fn meer_eternal_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &MEER_ETERNAL,
        AIType::AiMage,
        FightMode::Evil,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    meer_eternal_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.MeerEternal`.
    MEER_ETERNAL, MEER_ETERNAL_C = "Server.Mobiles.MeerEternal" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(meer_eternal_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a meer's corpse"),
        default_name: |_w, _id| Some("a meer eternal"),
        auto_dispel: |_w, _id| true,
        bard_immune: |w, _id| !w.core_aos(),
        can_rummage_corpses: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(4)),
        treasure_map_level: |w, _id| if w.core_aos() { 5 } else { 4 },
        initial_innocent: |_w, _id| true,
        on_think: crate::hand::meer_eternal_on_think,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::filthy_rich(w), 2);
            add_loot_n(w, id, &loot_pack::MED_SCROLLS, 2);
            add_loot_n(w, id, &loot_pack::HIGH_SCROLLS, 2);
        },
    }
    m {
        get_hurt_sound: |_w, _id| 0x167,
        get_death_sound: |_w, _id| 0xBC,
        get_attack_sound: |_w, _id| 0x28B,
    }
}

pub(crate) fn meer_mage_init(w: &mut World, id: MobileId) {
    body(w, id, 770);
    set_str_range(w, id, 171, 200);
    set_dex_range(w, id, 126, 145);
    set_int_range(w, id, 276, 305);
    set_hits_range(w, id, 103, 120);
    set_damage(w, id, 24, 26);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 45, 55);
    set_resistance_range(w, id, ResistanceType::Fire, 15, 25);
    set_resistance(w, id, ResistanceType::Cold, 50);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill(w, id, SkillName::EvalInt, 100.0);
    set_skill_range(w, id, SkillName::Magery, 70.1, 80.0);
    set_skill_range(w, id, SkillName::Meditation, 85.1, 95.0);
    set_skill_range(w, id, SkillName::MagicResist, 80.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 70.1, 90.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 80.0);
    fame(w, id, 8000);
    karma(w, id, 8000);
    virtual_armor(w, id, 16);
    crate::hand::meer_mage_ctor1(w, id);
}
fn meer_mage_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &MEER_MAGE,
        AIType::AiMage,
        FightMode::Evil,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    meer_mage_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.MeerMage`.
    MEER_MAGE, MEER_MAGE_C = "Server.Mobiles.MeerMage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(meer_mage_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a meer's corpse"),
        default_name: |_w, _id| Some("a meer mage"),
        auto_dispel: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(4)),
        can_rummage_corpses: |_w, _id| true,
        treasure_map_level: |_w, _id| 3,
        initial_innocent: |_w, _id| true,
        is_enemy: crate::hand::meer_mage_is_enemy,
        on_think: crate::hand::meer_mage_on_think,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot_n(w, id, &loot_pack::MED_SCROLLS, 2);
        },
    }
    m {
        get_hurt_sound: |_w, _id| 0x14D,
        get_death_sound: |_w, _id| 0x314,
        get_attack_sound: |_w, _id| 0x75,
    }
}

fn meer_warrior_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &MEER_WARRIOR)
}
creature_leaf! {
    /// `Server.Mobiles.MeerWarrior`.
    MEER_WARRIOR, MEER_WARRIOR_C = "Server.Mobiles.MeerWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(meer_warrior_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bard_immune: |w, _id| !w.core_aos(),
        can_rummage_corpses: tp::can_rummage_corpses,
        initial_innocent: |_w, _id| true,
        generate_loot: tp::generate_loot,
    }
    m {
        on_damage: crate::hand::meer_warrior_on_damage,
        get_hurt_sound: |_w, _id| 0x156,
        get_death_sound: |_w, _id| 0x15C,
    }
}

pub(crate) fn cu_sidhe_init(w: &mut World, id: MobileId) {
    let chance = rnd::double(w) * 23301.0;
    if chance <= 1.0 {
        hue(w, id, 0x489);
    } else {
        if chance < 50.0 {
            {
                let list = [0x657, 0x515, 0x4B1, 0x481, 0x482, 0x455];
                let i = rnd::below(w, list.len() as i32) as usize;
                hue(w, id, list[i]);
            }
        } else {
            if chance < 500.0 {
                {
                    let list = [0x97A, 0x978, 0x901, 0x8AC, 0x5A7, 0x527];
                    let i = rnd::below(w, list.len() as i32) as usize;
                    hue(w, id, list[i]);
                }
            }
        }
    }
}
fn cu_sidhe_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &CU_SIDHE,
        277,
        0x3E91,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    cu_sidhe_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.CuSidhe`.
    CU_SIDHE, CU_SIDHE_C = "Server.Mobiles.CuSidhe" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(cu_sidhe_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a cu sidhe corpse"),
        default_name: |_w, _id| Some("a cu sidhe"),
        can_heal: |_w, _id| true,
        can_heal_owner: |_w, _id| true,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES,
        can_anger_on_tame: |_w, _id| true,
        stat_loss_after_tame: |_w, _id| true,
        hides: |_w, _id| 10,
        meat: |_w, _id| 3,
        get_weapon_ability: |_w, _id| Some(Ability::BleedAttack),
        generate_loot: |w, id| {
            add_loot_n(w, id, &loot_pack::AOS_FILTHY_RICH, 5);
        },
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

pub(crate) fn ferret_init(w: &mut World, id: MobileId) {
    body(w, id, 0x117);
    set_str_range(w, id, 41, 48);
    set_dex(w, id, 55);
    set_int(w, id, 75);
    set_hits_range(w, id, 45, 50);
    set_damage(w, id, 7, 9);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 45, 50);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 14);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 21, 25);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 25);
    set_skill(w, id, SkillName::MagicResist, 4.0);
    set_skill(w, id, SkillName::Tactics, 4.0);
    set_skill(w, id, SkillName::Wrestling, 4.0);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, -21.3);
    crate::hand::ferret_ctor1(w, id);
}
fn ferret_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &FERRET,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ferret_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Ferret`.
    FERRET, FERRET_C = "Server.Mobiles.Ferret" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ferret_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a ferret corpse"),
        default_name: |_w, _id| Some("a ferret"),
        meat: |_w, _id| 1,
        favorite_food: |_w, _id| FoodType::FISH,
    }
    m {
        on_movement: crate::hand::ferret_on_movement,
    }
}

fn raging_grizzly_bear_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &RAGING_GRIZZLY_BEAR)
}
creature_leaf! {
    /// `Server.Mobiles.RagingGrizzlyBear`.
    RAGING_GRIZZLY_BEAR, RAGING_GRIZZLY_BEAR_C = "Server.Mobiles.RagingGrizzlyBear" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(raging_grizzly_bear_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        hides: tp::hides,
        pack_instinct: |_w, _id| PackInstinct::BEAR,
    }
    m {
    }
}

fn squirrel_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &SQUIRREL)
}
creature_leaf! {
    /// `Server.Mobiles.Squirrel`.
    SQUIRREL, SQUIRREL_C = "Server.Mobiles.Squirrel" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(squirrel_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES,
    }
    m {
    }
}

pub(crate) fn lady_jennifyr_init(w: &mut World, id: MobileId) {
    skeletal_knight_init(w, id);
    is_paragon(w, id, true);
    hue(w, id, 0x76D);
    set_str_range(w, id, 208, 309);
    set_dex_range(w, id, 91, 118);
    set_int_range(w, id, 44, 101);
    set_hits_range(w, id, 1113, 1285);
    set_damage(w, id, 15, 25);
    set_damage_type(w, id, ResistanceType::Physical, 40);
    set_damage_type(w, id, ResistanceType::Cold, 60);
    set_resistance_range(w, id, ResistanceType::Physical, 56, 65);
    set_resistance_range(w, id, ResistanceType::Fire, 41, 49);
    set_resistance_range(w, id, ResistanceType::Cold, 71, 80);
    set_resistance_range(w, id, ResistanceType::Poison, 41, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 50, 58);
    set_skill_range(w, id, SkillName::Wrestling, 127.9, 137.1);
    set_skill_range(w, id, SkillName::Tactics, 128.4, 141.9);
    set_skill_range(w, id, SkillName::MagicResist, 102.1, 119.5);
    set_skill_range(w, id, SkillName::Anatomy, 129.0, 137.5);
    fame(w, id, 18000);
    karma(w, id, -18000);
}
fn lady_jennifyr_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LADY_JENNIFYR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    lady_jennifyr_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.LadyJennifyr`.
    LADY_JENNIFYR, LADY_JENNIFYR_C = "Server.Mobiles.LadyJennifyr" : SKELETAL_KNIGHT / SKELETAL_KNIGHT_C, ctor Some(lady_jennifyr_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Lady Jennifyr corpse"),
        default_name: |_w, _id| Some("Lady Jennifyr"),
        gives_ml_minor_artifact: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 3);
        },
        get_monster_abilities: crate::hand::lady_jennifyr_abilities,
    }
    m {
    }
}

pub(crate) fn master_jonath_init(w: &mut World, id: MobileId) {
    bone_magi_init(w, id);
    is_paragon(w, id, true);
    hue(w, id, 0x455);
    set_str_range(w, id, 109, 131);
    set_dex_range(w, id, 98, 110);
    set_int_range(w, id, 232, 259);
    set_hits_range(w, id, 766, 920);
    set_damage(w, id, 10, 15);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 60);
    set_resistance_range(w, id, ResistanceType::Fire, 43, 49);
    set_resistance_range(w, id, ResistanceType::Cold, 45, 80);
    set_resistance_range(w, id, ResistanceType::Poison, 41, 45);
    set_resistance_range(w, id, ResistanceType::Energy, 54, 55);
    set_skill_range(w, id, SkillName::Wrestling, 80.5, 88.6);
    set_skill_range(w, id, SkillName::Tactics, 88.5, 95.1);
    set_skill_range(w, id, SkillName::MagicResist, 102.7, 102.9);
    set_skill_range(w, id, SkillName::Magery, 100.0, 106.6);
    set_skill_range(w, id, SkillName::EvalInt, 99.6, 106.9);
    set_skill_range(w, id, SkillName::Necromancy, 100.0, 106.6);
    set_skill_range(w, id, SkillName::SpiritSpeak, 99.6, 106.9);
    fame(w, id, 18000);
    karma(w, id, -18000);
    if rnd::bool(w) {
        {
            let a0 = rnd::min_max(w, 5, 9);
            pack_necro_scroll(w, id, a0);
        }
    } else {
        pack_scroll_range(w, id, 4, 7);
    }
    pack_reg(w, id, 7);
    pack_reg(w, id, 7);
    pack_reg(w, id, 8);
}
fn master_jonath_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &MASTER_JONATH,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    master_jonath_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.MasterJonath`.
    MASTER_JONATH, MASTER_JONATH_C = "Server.Mobiles.MasterJonath" : BONE_MAGI / BONE_MAGI_C, ctor Some(master_jonath_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Master Jonath corpse"),
        default_name: |_w, _id| Some("Master Jonath"),
        gives_ml_minor_artifact: |_w, _id| true,
        treasure_map_level: |_w, _id| 5,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 3);
        },
    }
    m {
    }
}

pub(crate) fn master_mikael_init(w: &mut World, id: MobileId) {
    bone_magi_init(w, id);
    is_paragon(w, id, true);
    hue(w, id, 0x8FD);
    set_str_range(w, id, 93, 122);
    set_dex_range(w, id, 91, 100);
    set_int_range(w, id, 252, 271);
    set_hits_range(w, id, 789, 1014);
    set_damage(w, id, 11, 19);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 59);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 46);
    set_resistance_range(w, id, ResistanceType::Cold, 72, 80);
    set_resistance_range(w, id, ResistanceType::Poison, 44, 49);
    set_resistance_range(w, id, ResistanceType::Energy, 50, 57);
    set_skill_range(w, id, SkillName::Wrestling, 80.1, 87.2);
    set_skill_range(w, id, SkillName::Tactics, 79.0, 90.9);
    set_skill_range(w, id, SkillName::MagicResist, 90.3, 106.9);
    set_skill_range(w, id, SkillName::Magery, 103.8, 108.0);
    set_skill_range(w, id, SkillName::EvalInt, 96.1, 105.3);
    set_skill_range(w, id, SkillName::Necromancy, 103.8, 108.0);
    set_skill_range(w, id, SkillName::SpiritSpeak, 96.1, 105.3);
    fame(w, id, 18000);
    karma(w, id, -18000);
    if rnd::bool(w) {
        {
            let a0 = rnd::min_max(w, 5, 9);
            pack_necro_scroll(w, id, a0);
        }
    } else {
        pack_scroll_range(w, id, 4, 7);
    }
    pack_reg(w, id, 3);
    pack_necro_reg_range(w, id, 1, 10);
}
fn master_mikael_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &MASTER_MIKAEL,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    master_mikael_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.MasterMikael`.
    MASTER_MIKAEL, MASTER_MIKAEL_C = "Server.Mobiles.MasterMikael" : BONE_MAGI / BONE_MAGI_C, ctor Some(master_mikael_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Master Mikael corpse"),
        default_name: |_w, _id| Some("Master Mikael"),
        gives_ml_minor_artifact: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 2);
        },
    }
    m {
    }
}

pub(crate) fn master_theophilus_init(w: &mut World, id: MobileId) {
    evil_mage_lord_init(w, id);
    is_paragon(w, id, true);
    title(w, id, "the necromancer");
    hue(w, id, 0);
    set_str_range(w, id, 137, 187);
    set_dex_range(w, id, 253, 301);
    set_int_range(w, id, 393, 444);
    set_hits_range(w, id, 663, 876);
    set_damage(w, id, 15, 20);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 60);
    set_resistance_range(w, id, ResistanceType::Fire, 50, 58);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 60);
    set_resistance_range(w, id, ResistanceType::Poison, 50, 60);
    set_resistance_range(w, id, ResistanceType::Energy, 50, 60);
    set_skill_range(w, id, SkillName::Wrestling, 69.9, 105.3);
    set_skill_range(w, id, SkillName::Tactics, 113.0, 117.9);
    set_skill_range(w, id, SkillName::MagicResist, 127.0, 132.8);
    set_skill_range(w, id, SkillName::Magery, 138.1, 143.7);
    set_skill_range(w, id, SkillName::EvalInt, 125.6, 133.8);
    set_skill_range(w, id, SkillName::Necromancy, 125.6, 133.8);
    set_skill_range(w, id, SkillName::SpiritSpeak, 125.6, 133.8);
    set_skill_range(w, id, SkillName::Meditation, 128.8, 132.9);
    fame(w, id, 18000);
    karma(w, id, -18000);
    crate::hand::master_theophilus_ctor1(w, id);
    crate::hand::master_theophilus_ctor2(w, id);
    crate::hand::master_theophilus_ctor3(w, id);
    pack_reg(w, id, 7);
    pack_reg(w, id, 7);
    pack_reg(w, id, 8);
}
fn master_theophilus_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &MASTER_THEOPHILUS,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    master_theophilus_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.MasterTheophilus`.
    MASTER_THEOPHILUS, MASTER_THEOPHILUS_C = "Server.Mobiles.MasterTheophilus" : EVIL_MAGE_LORD / EVIL_MAGE_LORD_C, ctor Some(master_theophilus_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Master Theophilus corpse"),
        default_name: |_w, _id| Some("Master Theophilus"),
        gives_ml_minor_artifact: |_w, _id| true,
        treasure_map_level: |_w, _id| 5,
        get_weapon_ability: |_w, _id| Some(Ability::ParalyzingBlow),
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 3);
        },
    }
    m {
    }
}

pub(crate) fn red_death_init(w: &mut World, id: MobileId) {
    skeletal_mount_init(w, id);
    is_paragon(w, id, true);
    hue(w, id, 0x21);
    base_sound_id(w, id, 0x1C3);
    crate::hand::red_death_ctor1(w, id);
    crate::hand::red_death_ctor2(w, id);
    set_str_range(w, id, 319, 324);
    set_dex_range(w, id, 241, 244);
    set_int_range(w, id, 242, 255);
    set_hits_range(w, id, 1540, 1605);
    set_damage(w, id, 25, 29);
    set_damage_type(w, id, ResistanceType::Physical, 25);
    set_damage_type(w, id, ResistanceType::Fire, 75);
    set_damage_type(w, id, ResistanceType::Cold, 0);
    set_resistance_range(w, id, ResistanceType::Physical, 60, 70);
    set_resistance(w, id, ResistanceType::Fire, 90);
    set_resistance(w, id, ResistanceType::Cold, 0);
    set_resistance(w, id, ResistanceType::Poison, 100);
    set_resistance(w, id, ResistanceType::Energy, 0);
    set_skill_range(w, id, SkillName::Wrestling, 121.4, 143.7);
    set_skill_range(w, id, SkillName::Tactics, 120.9, 142.2);
    set_skill_range(w, id, SkillName::MagicResist, 120.1, 142.3);
    set_skill_range(w, id, SkillName::Anatomy, 120.2, 144.0);
    fame(w, id, 28000);
    karma(w, id, -28000);
    if rnd::bool(w) {
        {
            let a0 = rnd::min_max(w, 5, 9);
            pack_necro_scroll(w, id, a0);
        }
    } else {
        pack_scroll_range(w, id, 4, 7);
    }
}
fn red_death_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &RED_DEATH,
        793,
        0x3EBB,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    red_death_init(w, id);
    id
}
static RED_DEATH_ABILITIES: [&MonsterAbility; 1] = [&abilities::CHAOS_BREATH];
creature_leaf! {
    /// `Server.Mobiles.RedDeath`.
    RED_DEATH, RED_DEATH_C = "Server.Mobiles.RedDeath" : SKELETAL_MOUNT / SKELETAL_MOUNT_C, ctor Some(red_death_new), encoded false;
    c {
        default_name: |_w, _id| Some("Red Death"),
        corpse_name: |_w, _id| Some("a Red Death corpse"),
        gives_ml_minor_artifact: |_w, _id| true,
        always_murderer: |_w, _id| true,
        get_weapon_ability: |_w, _id| Some(Ability::WhirlwindAttack),
        get_monster_abilities: |_, _| &RED_DEATH_ABILITIES,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 3);
        },
    }
    m {
        on_death: |w, id, c| {
            (SKELETAL_MOUNT.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.ResolvesBridle");
        },
    }
}

pub(crate) fn sir_patrick_init(w: &mut World, id: MobileId) {
    skeletal_knight_init(w, id);
    is_paragon(w, id, true);
    hue(w, id, 0x47E);
    set_str_range(w, id, 208, 319);
    set_dex_range(w, id, 98, 132);
    set_int_range(w, id, 45, 91);
    set_hits_range(w, id, 616, 884);
    set_damage(w, id, 15, 25);
    set_damage_type(w, id, ResistanceType::Physical, 40);
    set_damage_type(w, id, ResistanceType::Cold, 60);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 62);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 48);
    set_resistance_range(w, id, ResistanceType::Cold, 71, 80);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 50, 60);
    set_skill_range(w, id, SkillName::Wrestling, 126.3, 136.5);
    set_skill_range(w, id, SkillName::Tactics, 128.5, 143.8);
    set_skill_range(w, id, SkillName::MagicResist, 102.8, 117.9);
    set_skill_range(w, id, SkillName::Anatomy, 127.5, 137.2);
    fame(w, id, 18000);
    karma(w, id, -18000);
}
fn sir_patrick_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SIR_PATRICK,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    sir_patrick_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SirPatrick`.
    SIR_PATRICK, SIR_PATRICK_C = "Server.Mobiles.SirPatrick" : SKELETAL_KNIGHT / SKELETAL_KNIGHT_C, ctor Some(sir_patrick_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Sir Patrick corpse"),
        default_name: |_w, _id| Some("Sir Patrick"),
        gives_ml_minor_artifact: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 2);
        },
        get_monster_abilities: crate::hand::sir_patrick_abilities,
    }
    m {
    }
}

fn hydra_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &HYDRA)
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
    templates::new_creature(w, &ABSCESS)
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
    templates::new_creature(w, &COIL)
}
creature_leaf! {
    /// `Server.Mobiles.Coil`.
    COIL, COIL_C = "Server.Mobiles.Coil" : SILVER_SERPENT / SILVER_SERPENT_C, ctor Some(coil_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hit_poison: |_w, _id| Some(PoisonId(4)),
        poison_immune: tp::poison_immune,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        hides: tp::hides,
        meat: tp::meat,
        get_weapon_ability: |_w, _id| Some(Ability::MortalStrike),
        generate_loot: tp::generate_loot,
    }
    m {
        on_death: |w, id, c| {
            (SILVER_SERPENT.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.CoilsFang");
        },
    }
}

pub(crate) fn satyr_init(w: &mut World, id: MobileId) {
    body(w, id, 271);
    base_sound_id(w, id, 0x586);
    set_str_range(w, id, 177, 195);
    set_dex_range(w, id, 251, 269);
    set_int_range(w, id, 153, 170);
    set_hits_range(w, id, 350, 400);
    set_damage(w, id, 13, 24);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 60);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::MagicResist, 55.0, 65.0);
    set_skill_range(w, id, SkillName::Tactics, 80.0, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 80.0, 100.0);
    fame(w, id, 5000);
    karma(w, id, 0);
    virtual_armor(w, id, 28);
    crate::hand::satyr_ctor1(w, id);
}
fn satyr_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SATYR,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    satyr_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Satyr`.
    SATYR, SATYR_C = "Server.Mobiles.Satyr" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(satyr_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a satyr's corpse"),
        default_name: |_w, _id| Some("a satyr"),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        meat: |_w, _id| 1,
        on_think: crate::hand::satyr_on_think,
        generate_loot: |w, id| {
            add_loot(w, id, &loot_pack::ML_RICH);
        },
    }
    m {
    }
}

pub(crate) fn enslaved_satyr_init(w: &mut World, id: MobileId) {
    satyr_init(w, id);
}
fn enslaved_satyr_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ENSLAVED_SATYR,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    enslaved_satyr_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.EnslavedSatyr`.
    ENSLAVED_SATYR, ENSLAVED_SATYR_C = "Server.Mobiles.EnslavedSatyr" : SATYR / SATYR_C, ctor Some(enslaved_satyr_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an enslaved satyr corpse"),
        default_name: |_w, _id| Some("an enslaved satyr"),
    }
    m {
    }
}

pub(crate) fn ml_dryad_init(w: &mut World, id: MobileId) {
    body(w, id, 266);
    base_sound_id(w, id, 0x57B);
    set_str_range(w, id, 132, 149);
    set_dex_range(w, id, 152, 168);
    set_int_range(w, id, 251, 280);
    set_hits_range(w, id, 304, 321);
    set_damage(w, id, 11, 20);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 50);
    set_resistance_range(w, id, ResistanceType::Fire, 15, 25);
    set_resistance_range(w, id, ResistanceType::Cold, 40, 45);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::Meditation, 80.0, 90.0);
    set_skill_range(w, id, SkillName::EvalInt, 70.0, 80.0);
    set_skill_range(w, id, SkillName::Magery, 70.0, 80.0);
    set_skill(w, id, SkillName::Anatomy, 0.0);
    set_skill_range(w, id, SkillName::MagicResist, 100.0, 120.0);
    set_skill_range(w, id, SkillName::Tactics, 70.0, 80.0);
    set_skill_range(w, id, SkillName::Wrestling, 70.0, 80.0);
    fame(w, id, 5000);
    karma(w, id, 5000);
    virtual_armor(w, id, 28);
    crate::hand::ml_dryad_ctor1(w, id);
    crate::hand::ml_dryad_ctor2(w, id);
}
fn ml_dryad_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ML_DRYAD,
        AIType::AiMage,
        FightMode::Evil,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ml_dryad_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.MLDryad`.
    ML_DRYAD, ML_DRYAD_C = "Server.Mobiles.MLDryad" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ml_dryad_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a dryad's corpse"),
        initial_innocent: |_w, _id| true,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        default_name: |_w, _id| Some("a dryad"),
        meat: |_w, _id| 1,
        on_think: crate::hand::ml_dryad_on_think,
        generate_loot: |w, id| {
            add_loot(w, id, &loot_pack::ML_RICH);
        },
    }
    m {
    }
}

pub(crate) fn insane_dryad_init(w: &mut World, id: MobileId) {
    ml_dryad_init(w, id);
}
fn insane_dryad_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &INSANE_DRYAD,
        AIType::AiMage,
        FightMode::Evil,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    insane_dryad_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.InsaneDryad`.
    INSANE_DRYAD, INSANE_DRYAD_C = "Server.Mobiles.InsaneDryad" : ML_DRYAD / ML_DRYAD_C, ctor Some(insane_dryad_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an insane dryad corpse"),
        initial_innocent: |_w, _id| false,
        default_name: |_w, _id| Some("an insane dryad"),
    }
    m {
    }
}

fn harpy_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &HARPY)
}
creature_leaf! {
    /// `Server.Mobiles.Harpy`.
    HARPY, HARPY_C = "Server.Mobiles.Harpy" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(harpy_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_rummage_corpses: tp::can_rummage_corpses,
        meat: tp::meat,
        meat_type: |_w, _id| MeatType::Bird,
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
    templates::new_creature(w, &SALIVA)
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

pub(crate) fn bog_thing_init(w: &mut World, id: MobileId) {
    body(w, id, 780);
    set_str_range(w, id, 801, 900);
    set_dex_range(w, id, 46, 65);
    set_int_range(w, id, 36, 50);
    set_hits_range(w, id, 481, 540);
    set_mana(w, id, 0);
    set_damage(w, id, 10, 23);
    set_damage_type(w, id, ResistanceType::Physical, 60);
    set_damage_type(w, id, ResistanceType::Poison, 40);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 25);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 15);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 25);
    set_skill_range(w, id, SkillName::MagicResist, 90.1, 95.0);
    set_skill_range(w, id, SkillName::Tactics, 70.1, 85.0);
    set_skill_range(w, id, SkillName::Wrestling, 65.1, 80.0);
    fame(w, id, 8000);
    karma(w, id, -8000);
    virtual_armor(w, id, 28);
    if rnd::double(w) < 0.25 {
        pack_new_amount(w, id, "Server.Items.Board", 10);
    } else {
        pack_new_amount(w, id, "Server.Items.Log", 10);
    }
    pack_reg(w, id, 3);
    pack_new(w, id, "Server.Engines.Plants.Seed");
    pack_new(w, id, "Server.Engines.Plants.Seed");
}
fn bog_thing_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BOG_THING,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    bog_thing_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.BogThing`.
    BOG_THING, BOG_THING_C = "Server.Mobiles.BogThing" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bog_thing_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a plant corpse"),
        default_name: |_w, _id| Some("a bog thing"),
        bard_immune: |w, _id| !w.core_aos(),
        poison_immune: |_w, _id| Some(PoisonId(4)),
        on_got_melee_attack: crate::hand::bog_thing_on_got_melee_attack,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::average(w), 2);
        },
    }
    m {
    }
}

pub(crate) fn tangle_init(w: &mut World, id: MobileId) {
    bog_thing_init(w, id);
    hue(w, id, 0x21);
    set_str_range(w, id, 870, 940);
    set_dex_range(w, id, 58, 74);
    set_int_range(w, id, 46, 58);
    set_hits_range(w, id, 2468, 2733);
    set_mana_range(w, id, 8, 12);
    set_damage(w, id, 15, 28);
    set_damage_type(w, id, ResistanceType::Physical, 60);
    set_damage_type(w, id, ResistanceType::Poison, 40);
    set_resistance_range(w, id, ResistanceType::Physical, 50, 57);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 43);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 61, 69);
    set_resistance_range(w, id, ResistanceType::Energy, 41, 45);
    set_skill_range(w, id, SkillName::Wrestling, 77.8, 94.6);
    set_skill_range(w, id, SkillName::Tactics, 90.6, 100.4);
    set_skill_range(w, id, SkillName::MagicResist, 108.4, 114.0);
}
fn tangle_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &TANGLE,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    tangle_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Tangle`.
    TANGLE, TANGLE_C = "Server.Mobiles.Tangle" : BOG_THING / BOG_THING_C, ctor Some(tangle_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Tangle corpse"),
        default_name: |_w, _id| Some("Tangle"),
        poison_immune: |_w, _id| Some(PoisonId(4)),
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 3);
        },
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
    templates::new_creature(w, &THRASHER)
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
        get_weapon_ability: |_w, _id| Some(Ability::ArmorIgnore),
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
    templates::new_creature(w, &FETID_ESSENCE)
}
creature_leaf! {
    /// `Server.Mobiles.FetidEssence`.
    FETID_ESSENCE, FETID_ESSENCE_C = "Server.Mobiles.FetidEssence" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(fetid_essence_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hit_poison: |_w, _id| Some(PoisonId(3)),
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
    templates::new_creature(w, &INTERRED_GRIZZLE)
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

pub(crate) fn corrupted_soul_init(w: &mut World, id: MobileId) {
    body(w, id, 0x3CA);
    hue(w, id, 0x453);
    set_str_range(w, id, 102, 115);
    set_dex_range(w, id, 101, 115);
    set_int_range(w, id, 203, 215);
    set_hits_range(w, id, 61, 69);
    crate::hand::corrupted_soul_ctor1(w, id);
    set_damage(w, id, 4, 40);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 61, 74);
    set_resistance_range(w, id, ResistanceType::Fire, 22, 48);
    set_resistance_range(w, id, ResistanceType::Cold, 73, 100);
    set_resistance(w, id, ResistanceType::Poison, 0);
    set_resistance_range(w, id, ResistanceType::Energy, 51, 60);
    set_skill_range(w, id, SkillName::MagicResist, 80.2, 89.4);
    set_skill_range(w, id, SkillName::Tactics, 81.3, 89.9);
    set_skill_range(w, id, SkillName::Wrestling, 80.1, 88.7);
    fame(w, id, 5000);
    karma(w, id, -5000);
}
fn corrupted_soul_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CORRUPTED_SOUL,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    corrupted_soul_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.CorruptedSoul`.
    CORRUPTED_SOUL, CORRUPTED_SOUL_C = "Server.Mobiles.CorruptedSoul" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(corrupted_soul_new), encoded false;
    c {
        delete_corpse_on_death: |_w, _id| true,
        default_name: |_w, _id| Some("a corrupted soul"),
        always_attackable: |_w, _id| true,
        bleed_immune: |_w, _id| true,
        always_murderer: |_w, _id| true,
    }
    m {
        get_attack_sound: |_w, _id| 0x233,
        on_before_death: crate::hand::corrupted_soul_on_before_death,
    }
}

fn feral_treefellow_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &FERAL_TREEFELLOW)
}
creature_leaf! {
    /// `Server.Mobiles.FeralTreefellow`.
    FERAL_TREEFELLOW, FERAL_TREEFELLOW_C = "Server.Mobiles.FeralTreefellow" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(feral_treefellow_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        bleed_immune: tp::bleed_immune,
        get_weapon_ability: |_w, _id| Some(Ability::Dismount),
        generate_loot: tp::generate_loot,
    }
    m {
        get_idle_sound: |_w, _id| 443,
        get_death_sound: |_w, _id| 31,
        get_attack_sound: |_w, _id| 672,
    }
}

fn minotaur_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &MINOTAUR)
}
creature_leaf! {
    /// `Server.Mobiles.Minotaur`.
    MINOTAUR, MINOTAUR_C = "Server.Mobiles.Minotaur" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(minotaur_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        get_weapon_ability: |_w, _id| Some(Ability::ParalyzingBlow),
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
    templates::new_creature(w, &MINOTAUR_CAPTAIN)
}
creature_leaf! {
    /// `Server.Mobiles.MinotaurCaptain`.
    MINOTAUR_CAPTAIN, MINOTAUR_CAPTAIN_C = "Server.Mobiles.MinotaurCaptain" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(minotaur_captain_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        get_weapon_ability: |_w, _id| Some(Ability::ParalyzingBlow),
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
    templates::new_creature(w, &MINOTAUR_SCOUT)
}
creature_leaf! {
    /// `Server.Mobiles.MinotaurScout`.
    MINOTAUR_SCOUT, MINOTAUR_SCOUT_C = "Server.Mobiles.MinotaurScout" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(minotaur_scout_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        get_weapon_ability: |_w, _id| Some(Ability::ParalyzingBlow),
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
    templates::new_creature(w, &PESTILENT_BANDAGE)
}
creature_leaf! {
    /// `Server.Mobiles.PestilentBandage`.
    PESTILENT_BANDAGE, PESTILENT_BANDAGE_C = "Server.Mobiles.PestilentBandage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(pestilent_bandage_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        hit_poison: |_w, _id| Some(PoisonId(4)),
        can_heal: |_w, _id| true,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn tormented_minotaur_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &TORMENTED_MINOTAUR)
}
creature_leaf! {
    /// `Server.Mobiles.TormentedMinotaur`.
    TORMENTED_MINOTAUR, TORMENTED_MINOTAUR_C = "Server.Mobiles.TormentedMinotaur" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(tormented_minotaur_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: |_w, _id| Some(Ability::Dismount),
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
    templates::new_creature(w, &TROGLODYTE)
}
creature_leaf! {
    /// `Server.Mobiles.Troglodyte`.
    TROGLODYTE, TROGLODYTE_C = "Server.Mobiles.Troglodyte" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(troglodyte_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        can_heal: |_w, _id| true,
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
    templates::new_creature(w, &SCORPION)
}
creature_leaf! {
    /// `Server.Mobiles.Scorpion`.
    SCORPION, SCORPION_C = "Server.Mobiles.Scorpion" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(scorpion_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::ARACHNID,
        poison_immune: tp::poison_immune,
        hit_poison: |w, _id| if rnd::double(w) < 0.8 { Some(PoisonId(2)) } else { Some(PoisonId(3)) },
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn miasma_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &MIASMA)
}
creature_leaf! {
    /// `Server.Mobiles.Miasma`.
    MIASMA, MIASMA_C = "Server.Mobiles.Miasma" : SCORPION / SCORPION_C, ctor Some(miasma_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        weapon_ability_chance: |_w, _id| 0.75,
        hit_poison_chance: |_w, _id| 0.35,
        hit_poison: |_w, _id| Some(PoisonId(4)),
        has_mana_override: |_w, _id| true,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: |_w, _id| Some(Ability::MortalStrike),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn pyre_init(w: &mut World, id: MobileId) {
    phoenix_init(w, id);
    is_paragon(w, id, true);
    hue(w, id, 0x489);
    crate::hand::pyre_ctor1(w, id);
    set_str_range(w, id, 605, 611);
    set_dex_range(w, id, 391, 519);
    set_int_range(w, id, 669, 818);
    set_hits_range(w, id, 1783, 1939);
    set_damage(w, id, 30, 30);
    set_damage_type(w, id, ResistanceType::Physical, 50);
    set_damage_type(w, id, ResistanceType::Fire, 50);
    set_resistance(w, id, ResistanceType::Physical, 65);
    set_resistance_range(w, id, ResistanceType::Fire, 72, 75);
    set_resistance_range(w, id, ResistanceType::Poison, 36, 41);
    set_resistance_range(w, id, ResistanceType::Energy, 50, 51);
    set_skill_range(w, id, SkillName::Wrestling, 121.9, 130.6);
    set_skill_range(w, id, SkillName::Tactics, 114.4, 117.4);
    set_skill_range(w, id, SkillName::MagicResist, 147.7, 153.0);
    set_skill_range(w, id, SkillName::Poisoning, 122.8, 124.0);
    set_skill_range(w, id, SkillName::Magery, 121.8, 127.8);
    set_skill_range(w, id, SkillName::EvalInt, 103.6, 117.0);
    set_skill_range(w, id, SkillName::Meditation, 100.0, 110.0);
    fame(w, id, 21000);
    karma(w, id, -21000);
}
fn pyre_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &PYRE,
        AIType::AiMage,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    pyre_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Pyre`.
    PYRE, PYRE_C = "Server.Mobiles.Pyre" : PHOENIX / PHOENIX_C, ctor Some(pyre_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Pyre corpse"),
        default_name: |_w, _id| Some("Pyre"),
        gives_ml_minor_artifact: |_w, _id| true,
        treasure_map_level: |_w, _id| 5,
        has_aura: |_w, _id| true,
        get_weapon_ability: crate::hand::pyre_get_weapon_ability,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 3);
        },
    }
    m {
    }
}

pub(crate) fn reptalon_init(w: &mut World, id: MobileId) {
    base_sound_id(w, id, 0x16A);
    set_str_range(w, id, 1001, 1025);
    set_dex_range(w, id, 152, 164);
    set_int_range(w, id, 251, 289);
    set_hits_range(w, id, 833, 931);
    set_damage(w, id, 21, 28);
    set_damage_type(w, id, ResistanceType::Physical, 0);
    set_damage_type(w, id, ResistanceType::Poison, 25);
    set_damage_type(w, id, ResistanceType::Energy, 75);
    set_resistance_range(w, id, ResistanceType::Physical, 53, 64);
    set_resistance_range(w, id, ResistanceType::Fire, 35, 45);
    set_resistance_range(w, id, ResistanceType::Cold, 36, 45);
    set_resistance_range(w, id, ResistanceType::Poison, 52, 63);
    set_resistance_range(w, id, ResistanceType::Energy, 71, 83);
    set_skill_range(w, id, SkillName::Wrestling, 101.5, 118.2);
    set_skill_range(w, id, SkillName::Tactics, 101.7, 108.2);
    set_skill_range(w, id, SkillName::MagicResist, 76.4, 89.9);
    set_skill_range(w, id, SkillName::Anatomy, 56.4, 59.7);
    tamable(w, id, true);
    control_slots(w, id, 4);
    min_tame_skill(w, id, 101.1);
}
fn reptalon_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &REPTALON,
        0x114,
        0x3E90,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    reptalon_init(w, id);
    id
}
static REPTALON_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.Reptalon`.
    REPTALON, REPTALON_C = "Server.Mobiles.Reptalon" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(reptalon_new), encoded false;
    c {
        default_name: |_w, _id| Some("a reptalon"),
        corpse_name: |_w, _id| Some("a reptalon corpse"),
        treasure_map_level: |_w, _id| 5,
        meat: |_w, _id| 5,
        hides: |_w, _id| 10,
        can_anger_on_tame: |_w, _id| true,
        stat_loss_after_tame: |_w, _id| true,
        favorite_food: |_w, _id| FoodType::MEAT,
        can_fly: |_w, _id| true,
        get_weapon_ability: |_w, _id| Some(Ability::ParalyzingBlow),
        get_monster_abilities: |_, _| &REPTALON_ABILITIES,
        generate_loot: |w, id| {
            add_loot_n(w, id, &loot_pack::AOS_ULTRA_RICH, 3);
        },
    }
    m {
    }
}

pub(crate) fn rend_init(w: &mut World, id: MobileId) {
    reptalon_init(w, id);
    is_paragon(w, id, true);
    hue(w, id, 0x455);
    set_str_range(w, id, 1261, 1284);
    set_dex_range(w, id, 363, 384);
    set_int_range(w, id, 601, 642);
    set_hits_range(w, id, 5176, 6100);
    set_damage(w, id, 26, 33);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_damage_type(w, id, ResistanceType::Poison, 0);
    set_damage_type(w, id, ResistanceType::Energy, 0);
    set_resistance_range(w, id, ResistanceType::Physical, 75, 85);
    set_resistance_range(w, id, ResistanceType::Fire, 81, 94);
    set_resistance_range(w, id, ResistanceType::Cold, 46, 55);
    set_resistance_range(w, id, ResistanceType::Poison, 35, 44);
    set_resistance_range(w, id, ResistanceType::Energy, 45, 52);
    set_skill_range(w, id, SkillName::Wrestling, 136.3, 150.3);
    set_skill_range(w, id, SkillName::Tactics, 133.4, 141.4);
    set_skill_range(w, id, SkillName::MagicResist, 90.9, 110.0);
    set_skill_range(w, id, SkillName::Anatomy, 66.6, 72.0);
    fame(w, id, 21000);
    karma(w, id, -21000);
}
fn rend_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &REND,
        0x114,
        0x3E90,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    rend_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Rend`.
    REND, REND_C = "Server.Mobiles.Rend" : REPTALON / REPTALON_C, ctor Some(rend_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Rend corpse"),
        default_name: |_w, _id| Some("Rend"),
        gives_ml_minor_artifact: |_w, _id| true,
        get_weapon_ability: crate::hand::rend_get_weapon_ability,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 3);
        },
    }
    m {
    }
}

fn greater_dragon_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &GREATER_DRAGON)
}
static GREATER_DRAGON_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.GreaterDragon`.
    GREATER_DRAGON, GREATER_DRAGON_C = "Server.Mobiles.GreaterDragon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(greater_dragon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        stat_loss_after_tame: |_w, _id| true,
        default_name: tp::default_name,
        reacquire_on_movement: |w, id| !read(w, id, false, |d| d.controlled),
        auto_dispel: |w, id| !read(w, id, false, |d| d.controlled),
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: |_w, _id| HideType::Barbed,
        scales: tp::scales,
        scale_type: crate::hand::greater_dragon_scale_type,
        favorite_food: |_w, _id| FoodType::MEAT,
        can_anger_on_tame: |_w, _id| true,
        can_fly: tp::can_fly,
        get_weapon_ability: |_w, _id| Some(Ability::BleedAttack),
        get_monster_abilities: |_, _| &GREATER_DRAGON_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn corrosive_slime_init(w: &mut World, id: MobileId) {
    body(w, id, 51);
    base_sound_id(w, id, 456);
    {
        let a0 = rnd::from(w, 2201, 24);
        hue(w, id, a0);
    }
    set_str_range(w, id, 22, 34);
    set_dex_range(w, id, 16, 21);
    set_int_range(w, id, 16, 20);
    set_hits_range(w, id, 15, 19);
    set_damage(w, id, 1, 5);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 5, 10);
    set_resistance_range(w, id, ResistanceType::Poison, 15, 20);
    set_skill_range(w, id, SkillName::Poisoning, 36.0, 49.1);
    set_skill(w, id, SkillName::Anatomy, 0.0);
    set_skill_range(w, id, SkillName::MagicResist, 15.9, 18.9);
    set_skill_range(w, id, SkillName::Tactics, 24.6, 26.1);
    set_skill_range(w, id, SkillName::Wrestling, 24.9, 26.1);
    fame(w, id, 300);
    karma(w, id, -300);
    virtual_armor(w, id, 8);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 23.1);
}
fn corrosive_slime_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CORROSIVE_SLIME,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    corrosive_slime_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.CorrosiveSlime`.
    CORROSIVE_SLIME, CORROSIVE_SLIME_C = "Server.Mobiles.CorrosiveSlime" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(corrosive_slime_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a slimey corpse"),
        default_name: |_w, _id| Some("a corrosive slime"),
        poison_immune: |_w, _id| Some(PoisonId(1)),
        hit_poison: |_w, _id| Some(PoisonId(1)),
        favorite_food: |_w, _id| FoodType::FISH,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::poor(w));
            add_loot(w, id, &loot_pack::GEMS);
        },
    }
    m {
    }
}

pub(crate) fn grobu_init(w: &mut World, id: MobileId) {
    black_bear_init(w, id);
    is_paragon(w, id, true);
    hue(w, id, 0x455);
    crate::hand::grobu_ctor1(w, id);
    crate::hand::grobu_ctor2(w, id);
    set_str_range(w, id, 192, 210);
    set_dex_range(w, id, 132, 150);
    set_int_range(w, id, 50, 52);
    set_hits_range(w, id, 1235, 1299);
    set_stam_range(w, id, 132, 150);
    set_mana(w, id, 9);
    set_damage(w, id, 15, 18);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 32, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 22, 34);
    set_skill_range(w, id, SkillName::Wrestling, 96.4, 119.0);
    set_skill_range(w, id, SkillName::Tactics, 96.2, 116.5);
    set_skill_range(w, id, SkillName::MagicResist, 66.2, 83.7);
    fame(w, id, 1000);
    karma(w, id, 1000);
}
fn grobu_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GROBU,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    grobu_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Grobu`.
    GROBU, GROBU_C = "Server.Mobiles.Grobu" : BLACK_BEAR / BLACK_BEAR_C, ctor Some(grobu_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Grobu corpse"),
        default_name: |_w, _id| Some("Grobu"),
        gives_ml_minor_artifact: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::filthy_rich(w), 2);
        },
    }
    m {
        on_death: |w, id, c| {
            (BLACK_BEAR.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.GrobusFur");
        },
    }
}

fn lurg_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &LURG)
}
creature_leaf! {
    /// `Server.Mobiles.Lurg`.
    LURG, LURG_C = "Server.Mobiles.Lurg" : TROGLODYTE / TROGLODYTE_C, ctor Some(lurg_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        treasure_map_level: tp::treasure_map_level,
        get_weapon_ability: |_w, _id| Some(Ability::CrushingBlow),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn putrefier_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &PUTREFIER)
}
creature_leaf! {
    /// `Server.Mobiles.Putrefier`.
    PUTREFIER, PUTREFIER_C = "Server.Mobiles.Putrefier" : BALRON / BALRON_C, ctor Some(putrefier_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        hit_poison: |_w, _id| Some(PoisonId(3)),
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn corporeal_brume_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &CORPOREAL_BRUME)
}
creature_leaf! {
    /// `Server.Mobiles.CorporealBrume`.
    CORPOREAL_BRUME, CORPOREAL_BRUME_C = "Server.Mobiles.CorporealBrume" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(corporeal_brume_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        has_aura: crate::hand::corporeal_brume_has_aura,
        aura_interval: |_w, _id| TimeSpan::from_secs_f64(20_f64),
        aura_range: |_w, _id| 10,
        aura_base_damage: |w, _id| rnd::min_max(w, 25, 35),
        aura_fire_damage: |_w, _id| 0,
        aura_cold_damage: |_w, _id| 100,
        aura_effect: crate::hand::corporeal_brume_aura_effect,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn crystal_daemon_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &CRYSTAL_DAEMON)
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
    templates::new_creature(w, &CRYSTAL_LATTICE_SEEKER)
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

pub(crate) fn sea_serpent_init(w: &mut World, id: MobileId) {
    body(w, id, 150);
    base_sound_id(w, id, 447);
    {
        let a0 = rnd::from(w, 0x530, 9);
        hue(w, id, a0);
    }
    set_str_range(w, id, 168, 225);
    set_dex_range(w, id, 58, 85);
    set_int_range(w, id, 53, 95);
    set_hits_range(w, id, 110, 127);
    set_damage(w, id, 7, 13);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 50, 60);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 15, 20);
    set_skill_range(w, id, SkillName::MagicResist, 60.1, 75.0);
    set_skill_range(w, id, SkillName::Tactics, 60.1, 70.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 70.0);
    fame(w, id, 6000);
    karma(w, id, -6000);
    virtual_armor(w, id, 30);
    can_swim(w, id, true);
    cant_walk(w, id, true);
    if rnd::bool(w) {
        pack_new_amount(w, id, "Server.Items.SulfurousAsh", 4);
    } else {
        pack_new_amount(w, id, "Server.Items.BlackPearl", 4);
    }
    pack_new(w, id, "Server.Items.RawFishSteak");
}
fn sea_serpent_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SEA_SERPENT,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    sea_serpent_init(w, id);
    id
}
static SEA_SERPENT_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.SeaSerpent`.
    SEA_SERPENT, SEA_SERPENT_C = "Server.Mobiles.SeaSerpent" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(sea_serpent_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a sea serpents corpse"),
        default_name: |_w, _id| Some("a sea serpent"),
        treasure_map_level: |_w, _id| 2,
        hides: |_w, _id| 10,
        hide_type: |_w, _id| HideType::Horned,
        scales: |_w, _id| 8,
        scale_type: |_w, _id| ScaleType::Blue,
        get_monster_abilities: |_, _| &SEA_SERPENT_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

pub(crate) fn crystal_sea_serpent_init(w: &mut World, id: MobileId) {
    sea_serpent_init(w, id);
    hue(w, id, 0x47E);
    set_str_range(w, id, 250, 450);
    set_dex_range(w, id, 100, 150);
    set_int_range(w, id, 90, 190);
    set_hits_range(w, id, 230, 330);
    set_damage(w, id, 10, 18);
    set_damage_type(w, id, ResistanceType::Physical, 10);
    set_damage_type(w, id, ResistanceType::Cold, 45);
    set_damage_type(w, id, ResistanceType::Energy, 45);
    set_resistance_range(w, id, ResistanceType::Physical, 50, 70);
    set_resistance(w, id, ResistanceType::Fire, 0);
    set_resistance_range(w, id, ResistanceType::Cold, 70, 90);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 60, 80);
}
fn crystal_sea_serpent_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CRYSTAL_SEA_SERPENT,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    crystal_sea_serpent_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.CrystalSeaSerpent`.
    CRYSTAL_SEA_SERPENT, CRYSTAL_SEA_SERPENT_C = "Server.Mobiles.CrystalSeaSerpent" : SEA_SERPENT / SEA_SERPENT_C, ctor Some(crystal_sea_serpent_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a crystal sea serpent corpse"),
        default_name: |_w, _id| Some("a crystal sea serpent"),
    }
    m {
    }
}

fn crystal_vortex_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &CRYSTAL_VORTEX)
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

pub(crate) fn wisp_init(w: &mut World, id: MobileId) {
    body(w, id, 58);
    base_sound_id(w, id, 466);
    set_str_range(w, id, 196, 225);
    set_dex_range(w, id, 196, 225);
    set_int_range(w, id, 196, 225);
    set_hits_range(w, id, 118, 135);
    set_damage(w, id, 17, 18);
    set_damage_type(w, id, ResistanceType::Physical, 50);
    set_damage_type(w, id, ResistanceType::Energy, 50);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 30);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 50, 70);
    set_skill(w, id, SkillName::EvalInt, 80.0);
    set_skill(w, id, SkillName::Magery, 80.0);
    set_skill(w, id, SkillName::MagicResist, 80.0);
    set_skill(w, id, SkillName::Tactics, 80.0);
    set_skill(w, id, SkillName::Wrestling, 80.0);
    fame(w, id, 4000);
    karma(w, id, 0);
    virtual_armor(w, id, 40);
    crate::hand::wisp_ctor1(w, id);
    add_new(w, id, "Server.Items.LightSource");
}
fn wisp_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &WISP,
        AIType::AiMage,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    wisp_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Wisp`.
    WISP, WISP_C = "Server.Mobiles.Wisp" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(wisp_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a wisp corpse"),
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::WISP),
        faction_allegiance: crate::hand::wisp_faction_allegiance,
        ethic_allegiance: crate::hand::wisp_ethic_allegiance,
        reacquire_delay: |_w, _id| TimeSpan::from_secs_f64(1.0_f64),
        default_name: |_w, _id| Some("a wisp"),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
            add_loot(w, id, loot_pack::average(w));
        },
    }
    m {
    }
}

pub(crate) fn crystal_wisp_init(w: &mut World, id: MobileId) {
    wisp_init(w, id);
    hue(w, id, 0x482);
    pack_arcane_scroll_range(w, id, 0, 1);
}
fn crystal_wisp_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CRYSTAL_WISP,
        AIType::AiMage,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    crystal_wisp_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.CrystalWisp`.
    CRYSTAL_WISP, CRYSTAL_WISP_C = "Server.Mobiles.CrystalWisp" : WISP / WISP_C, ctor Some(crystal_wisp_new), encoded false;
    c {
        default_name: |_w, _id| Some("a crystal wisp"),
    }
    m {
    }
}

fn mantra_effervescence_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &MANTRA_EFFERVESCENCE)
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

pub(crate) fn protector_init(w: &mut World, id: MobileId) {
    body(w, id, 401);
    female(w, id, true);
    {
        let a0 = race_skin_hue(w, "Human");
        hue(w, id, a0);
    }
    crate::hand::protector_ctor1(w, id);
    crate::hand::protector_ctor2(w, id);
    title(w, id, "the mystic llamaherder");
    set_str_range(w, id, 700, 800);
    set_dex_range(w, id, 100, 150);
    set_int_range(w, id, 50, 75);
    set_hits_range(w, id, 350, 450);
    set_damage(w, id, 6, 12);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 35, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
    set_skill_range(w, id, SkillName::Wrestling, 70.0, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 80.0, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 50.0, 70.0);
    set_skill_range(w, id, SkillName::Anatomy, 70.0, 100.0);
    fame(w, id, 10000);
    karma(w, id, -10000);
    crate::hand::protector_ctor3(w, id);
    crate::hand::protector_ctor4(w, id);
}
fn protector_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &PROTECTOR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    protector_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Protector`.
    PROTECTOR, PROTECTOR_C = "Server.Mobiles.Protector" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(protector_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a human corpse"),
        default_name: |_w, _id| Some("a Protector"),
        always_murderer: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
        },
        generate_loot_spawning: crate::hand::protector_generate_loot_spawning,
    }
    m {
        property_title: |_w, _id| false,
        show_fame_title: |_w, _id| false,
    }
}

fn unfrozen_mummy_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &UNFROZEN_MUMMY)
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
    templates::new_creature(w, &CHIIKKAHA)
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
    templates::new_creature(w, &MOUG_GUUR)
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
    templates::new_creature(w, &SZAVETRA)
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

pub(crate) fn changeling_init(w: &mut World, id: MobileId) {
    body(w, id, 264);
    crate::hand::changeling_ctor1(w, id);
    set_str_range(w, id, 36, 105);
    set_dex_range(w, id, 212, 262);
    set_int_range(w, id, 317, 399);
    set_hits_range(w, id, 201, 211);
    set_stam_range(w, id, 212, 262);
    set_mana_range(w, id, 317, 399);
    set_damage(w, id, 9, 15);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 81, 90);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 40, 49);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 43, 50);
    set_skill_range(w, id, SkillName::Wrestling, 10.4, 12.5);
    set_skill_range(w, id, SkillName::Tactics, 101.1, 108.3);
    set_skill_range(w, id, SkillName::MagicResist, 121.6, 132.2);
    set_skill_range(w, id, SkillName::Magery, 91.6, 99.5);
    set_skill_range(w, id, SkillName::EvalInt, 91.5, 98.8);
    set_skill_range(w, id, SkillName::Meditation, 91.7, 98.5);
    fame(w, id, 15000);
    karma(w, id, -15000);
    pack_scroll_range(w, id, 1, 7);
    pack_new_amount(w, id, "Server.Items.Arrow", 35);
    pack_new_amount(w, id, "Server.Items.Bolt", 25);
    pack_gem(w, id, 2);
    pack_arcane_scroll_range(w, id, 0, 1);
}
fn changeling_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CHANGELING,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    changeling_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Changeling`.
    CHANGELING, CHANGELING_C = "Server.Mobiles.Changeling" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(changeling_new), state crate::hand::changeling_serialize, crate::hand::changeling_deserialize;
    c {
        corpse_name: |_w, _id| Some("a changeling corpse"),
        default_name: |_w, _id| Some("a changeling"),
        initial_innocent: crate::hand::changeling_initial_innocent,
        on_think: crate::hand::changeling_on_think,
        check_idle: crate::hand::changeling_check_idle,
        generate_loot: |w, id| {
            add_loot_n(w, id, &loot_pack::AOS_RICH, 3);
        },
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
    templates::new_creature(w, &GNAW)
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

pub(crate) fn guile_init(w: &mut World, id: MobileId) {
    changeling_init(w, id);
    is_paragon(w, id, true);
    crate::hand::guile_ctor1(w, id);
    set_str_range(w, id, 53, 214);
    set_dex_range(w, id, 243, 367);
    set_int_range(w, id, 369, 586);
    set_hits_range(w, id, 1013, 1058);
    set_stam_range(w, id, 243, 367);
    set_mana_range(w, id, 369, 586);
    set_damage(w, id, 14, 20);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 80, 90);
    set_resistance_range(w, id, ResistanceType::Fire, 43, 46);
    set_resistance_range(w, id, ResistanceType::Cold, 42, 44);
    set_resistance_range(w, id, ResistanceType::Poison, 42, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 47, 50);
    set_skill_range(w, id, SkillName::Wrestling, 12.8, 16.7);
    set_skill_range(w, id, SkillName::Tactics, 102.6, 131.0);
    set_skill_range(w, id, SkillName::MagicResist, 141.2, 161.6);
    set_skill_range(w, id, SkillName::Magery, 108.4, 120.0);
    set_skill_range(w, id, SkillName::EvalInt, 108.4, 120.0);
    set_skill_range(w, id, SkillName::Meditation, 109.2, 120.0);
    fame(w, id, 21000);
    karma(w, id, -21000);
}
fn guile_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GUILE,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    guile_init(w, id);
    id
}
static GUILE_ABILITIES: [&MonsterAbility; 1] = [&abilities::DRAIN_LIFE_ATTACK];
creature_leaf! {
    /// `Server.Mobiles.Guile`.
    GUILE, GUILE_C = "Server.Mobiles.Guile" : CHANGELING / CHANGELING_C, ctor Some(guile_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Guile corpse"),
        default_name: |_w, _id| Some("Guile"),
        default_hue: |_w, _id| 0x3F,
        gives_ml_minor_artifact: |_w, _id| true,
        get_monster_abilities: |_, _| &GUILE_ABILITIES,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 2);
        },
    }
    m {
    }
}

pub(crate) fn irk_init(w: &mut World, id: MobileId) {
    changeling_init(w, id);
    is_paragon(w, id, true);
    crate::hand::irk_ctor1(w, id);
    set_str_range(w, id, 23, 183);
    set_dex_range(w, id, 259, 360);
    set_int_range(w, id, 374, 600);
    set_hits_range(w, id, 1006, 1064);
    set_stam_range(w, id, 259, 360);
    set_mana_range(w, id, 374, 600);
    set_damage(w, id, 14, 20);
    set_resistance_range(w, id, ResistanceType::Physical, 80, 90);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 40, 50);
    set_resistance_range(w, id, ResistanceType::Poison, 41, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 49);
    set_skill_range(w, id, SkillName::Wrestling, 120.3, 123.0);
    set_skill_range(w, id, SkillName::Tactics, 120.1, 131.8);
    set_skill_range(w, id, SkillName::MagicResist, 132.3, 165.8);
    set_skill_range(w, id, SkillName::Magery, 108.9, 119.7);
    set_skill_range(w, id, SkillName::EvalInt, 108.4, 120.0);
    set_skill_range(w, id, SkillName::Meditation, 108.9, 119.1);
    fame(w, id, 21000);
    karma(w, id, -21000);
}
fn irk_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &IRK,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    irk_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Irk`.
    IRK, IRK_C = "Server.Mobiles.Irk" : CHANGELING / CHANGELING_C, ctor Some(irk_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an Irk corpse"),
        default_name: |_w, _id| Some("Irk"),
        default_hue: |_w, _id| 0x489,
        gives_ml_minor_artifact: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 2);
        },
    }
    m {
    }
}

fn lady_lissith_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &LADY_LISSITH)
}
creature_leaf! {
    /// `Server.Mobiles.LadyLissith`.
    LADY_LISSITH, LADY_LISSITH_C = "Server.Mobiles.LadyLissith" : GIANT_BLACK_WIDOW / GIANT_BLACK_WIDOW_C, ctor Some(lady_lissith_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        get_weapon_ability: |_w, _id| Some(Ability::BleedAttack),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn lady_sabrix_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &LADY_SABRIX)
}
creature_leaf! {
    /// `Server.Mobiles.LadySabrix`.
    LADY_SABRIX, LADY_SABRIX_C = "Server.Mobiles.LadySabrix" : GIANT_BLACK_WIDOW / GIANT_BLACK_WIDOW_C, ctor Some(lady_sabrix_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        get_weapon_ability: |_w, _id| Some(Ability::ArmorIgnore),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn malefic_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &MALEFIC)
}
creature_leaf! {
    /// `Server.Mobiles.Malefic`.
    MALEFIC, MALEFIC_C = "Server.Mobiles.Malefic" : DREAD_SPIDER / DREAD_SPIDER_C, ctor Some(malefic_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        get_weapon_ability: |_w, _id| Some(Ability::Dismount),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn silk_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &SILK)
}
creature_leaf! {
    /// `Server.Mobiles.Silk`.
    SILK, SILK_C = "Server.Mobiles.Silk" : GIANT_BLACK_WIDOW / GIANT_BLACK_WIDOW_C, ctor Some(silk_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        get_weapon_ability: |_w, _id| Some(Ability::ParalyzingBlow),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn spite_init(w: &mut World, id: MobileId) {
    changeling_init(w, id);
    is_paragon(w, id, true);
    crate::hand::spite_ctor1(w, id);
    set_str_range(w, id, 53, 214);
    set_dex_range(w, id, 243, 367);
    set_int_range(w, id, 369, 586);
    set_hits_range(w, id, 1013, 1052);
    set_stam_range(w, id, 243, 367);
    set_mana_range(w, id, 369, 586);
    set_damage(w, id, 14, 20);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 85, 90);
    set_resistance_range(w, id, ResistanceType::Fire, 41, 46);
    set_resistance_range(w, id, ResistanceType::Cold, 40, 44);
    set_resistance_range(w, id, ResistanceType::Poison, 42, 46);
    set_resistance_range(w, id, ResistanceType::Energy, 45, 47);
    set_skill_range(w, id, SkillName::Wrestling, 12.8, 16.7);
    set_skill_range(w, id, SkillName::Tactics, 102.6, 131.0);
    set_skill_range(w, id, SkillName::MagicResist, 141.2, 161.6);
    set_skill_range(w, id, SkillName::Magery, 108.4, 119.2);
    set_skill_range(w, id, SkillName::EvalInt, 108.4, 120.0);
    set_skill_range(w, id, SkillName::Meditation, 109.2, 120.0);
    fame(w, id, 21000);
    karma(w, id, -21000);
}
fn spite_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SPITE,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    spite_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Spite`.
    SPITE, SPITE_C = "Server.Mobiles.Spite" : CHANGELING / CHANGELING_C, ctor Some(spite_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Spite corpse"),
        default_name: |_w, _id| Some("Spite"),
        default_hue: |_w, _id| 0x21,
        gives_ml_minor_artifact: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 2);
        },
    }
    m {
    }
}

pub(crate) fn swoop_init(w: &mut World, id: MobileId) {
    eagle_init(w, id);
    is_paragon(w, id, true);
    hue(w, id, 0xE0);
    crate::hand::swoop_ctor1(w, id);
    set_str_range(w, id, 100, 150);
    set_dex_range(w, id, 400, 500);
    set_int_range(w, id, 80, 90);
    set_hits_range(w, id, 1500, 2000);
    set_damage(w, id, 20, 30);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 75, 90);
    set_resistance_range(w, id, ResistanceType::Fire, 60, 77);
    set_resistance_range(w, id, ResistanceType::Cold, 70, 85);
    set_resistance_range(w, id, ResistanceType::Poison, 55, 85);
    set_resistance_range(w, id, ResistanceType::Energy, 50, 60);
    set_skill_range(w, id, SkillName::Wrestling, 120.0, 140.0);
    set_skill_range(w, id, SkillName::Tactics, 120.0, 140.0);
    set_skill_range(w, id, SkillName::MagicResist, 95.0, 105.0);
    fame(w, id, 18000);
    karma(w, id, 0);
    pack_reg(w, id, 4);
    pack_arcane_scroll_range(w, id, 0, 1);
}
fn swoop_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SWOOP,
        AIType::AiAnimal,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    swoop_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Swoop`.
    SWOOP, SWOOP_C = "Server.Mobiles.Swoop" : EAGLE / EAGLE_C, ctor Some(swoop_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a Swoop corpse"),
        default_name: |_w, _id| Some("Swoop"),
        can_fly: |_w, _id| true,
        gives_ml_minor_artifact: |_w, _id| true,
        feathers: |_w, _id| 72,
        on_gave_melee_attack: crate::hand::swoop_on_gave_melee_attack,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::ultra_rich(w), 2);
        },
    }
    m {
    }
}

fn virulent_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &VIRULENT)
}
creature_leaf! {
    /// `Server.Mobiles.Virulent`.
    VIRULENT, VIRULENT_C = "Server.Mobiles.Virulent" : DREAD_SPIDER / DREAD_SPIDER_C, ctor Some(virulent_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        gives_ml_minor_artifact: tp::gives_ml_minor_artifact,
        get_weapon_ability: |_w, _id| Some(Ability::MortalStrike),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn hell_hound_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &HELL_HOUND)
}
static HELL_HOUND_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.HellHound`.
    HELL_HOUND, HELL_HOUND_C = "Server.Mobiles.HellHound" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(hell_hound_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        meat: tp::meat,
        favorite_food: |_w, _id| FoodType::MEAT,
        pack_instinct: |_w, _id| PackInstinct::CANINE,
        get_monster_abilities: |_, _| &HELL_HOUND_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn vorpal_bunny_init(w: &mut World, id: MobileId) {
    body(w, id, 205);
    hue(w, id, 0x480);
    set_str(w, id, 15);
    set_dex(w, id, 2000);
    set_int(w, id, 1000);
    set_hits(w, id, 2000);
    set_stam(w, id, 500);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 1);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_skill(w, id, SkillName::MagicResist, 200.0);
    set_skill(w, id, SkillName::Tactics, 5.0);
    set_skill(w, id, SkillName::Wrestling, 5.0);
    fame(w, id, 1000);
    karma(w, id, 0);
    virtual_armor(w, id, 4);
    crate::hand::vorpal_bunny_ctor1(w, id);
    if rnd::below(w, 5) == 0 {
        pack_new(w, id, "Server.Items.BrightlyColoredEggs");
    }
    pack_statue(w, id);
    crate::hand::vorpal_bunny_ctor2(w, id);
}
fn vorpal_bunny_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &VORPAL_BUNNY,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    vorpal_bunny_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.VorpalBunny`.
    VORPAL_BUNNY, VORPAL_BUNNY_C = "Server.Mobiles.VorpalBunny" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(vorpal_bunny_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a vorpal bunny corpse"),
        default_name: |_w, _id| Some("a vorpal bunny"),
        meat: |_w, _id| 1,
        hides: |_w, _id| 1,
        bard_immune: |w, _id| !w.core_aos(),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot_n(w, id, loot_pack::rich(w), 2);
        },
    }
    m {
        get_attack_sound: |_w, _id| 0xC9,
        get_hurt_sound: |_w, _id| 0xCA,
        get_death_sound: |_w, _id| 0xCB,
    }
}

fn dark_wisp_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &DARK_WISP)
}
creature_leaf! {
    /// `Server.Mobiles.DarkWisp`.
    DARK_WISP, DARK_WISP_C = "Server.Mobiles.DarkWisp" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dark_wisp_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        speech_type: |_, _| Some(&muo_content::misc::inhuman_speech::WISP),
        ethic_allegiance: crate::hand::dark_wisp_ethic_allegiance,
        reacquire_delay: |_w, _id| TimeSpan::from_secs_f64(1.0_f64),
        default_name: tp::default_name,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn ethereal_warrior_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &ETHEREAL_WARRIOR)
}
creature_leaf! {
    /// `Server.Mobiles.EtherealWarrior`.
    ETHEREAL_WARRIOR, ETHEREAL_WARRIOR_C = "Server.Mobiles.EtherealWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ethereal_warrior_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        initial_innocent: |_w, _id| true,
        treasure_map_level: |w, _id| if w.core_aos() { 5 } else { 0 },
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
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
    templates::new_creature(w, &PIXIE)
}
creature_leaf! {
    /// `Server.Mobiles.Pixie`.
    PIXIE, PIXIE_C = "Server.Mobiles.Pixie" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(pixie_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        initial_innocent: |_w, _id| true,
        hide_type: |_w, _id| HideType::Spined,
        hides: tp::hides,
        meat: tp::meat,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
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

pub(crate) fn shadow_wisp_init(w: &mut World, id: MobileId) {
    body(w, id, 165);
    base_sound_id(w, id, 466);
    set_str_range(w, id, 16, 40);
    set_dex_range(w, id, 16, 45);
    set_int_range(w, id, 11, 25);
    set_hits_range(w, id, 10, 24);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 5, 10);
    set_resistance_range(w, id, ResistanceType::Poison, 5, 10);
    set_resistance_range(w, id, ResistanceType::Energy, 15, 20);
    set_skill(w, id, SkillName::EvalInt, 40.0);
    set_skill(w, id, SkillName::Magery, 50.0);
    set_skill(w, id, SkillName::Meditation, 40.0);
    set_skill(w, id, SkillName::MagicResist, 10.0);
    set_skill_range(w, id, SkillName::Tactics, 0.1, 15.0);
    set_skill_range(w, id, SkillName::Wrestling, 25.1, 40.0);
    fame(w, id, 500);
    virtual_armor(w, id, 18);
    add_new(w, id, "Server.Items.LightSource");
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
}
fn shadow_wisp_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SHADOW_WISP,
        AIType::AiMage,
        FightMode::Aggressor,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    shadow_wisp_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.ShadowWisp`.
    SHADOW_WISP, SHADOW_WISP_C = "Server.Mobiles.ShadowWisp" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(shadow_wisp_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a wisp corpse"),
        default_name: |_w, _id| Some("a shadow wisp"),
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
    }
    m {
    }
}

pub(crate) fn blade_spirits_init(w: &mut World, id: MobileId) {
    body(w, id, 574);
    set_str(w, id, 150);
    set_dex(w, id, 150);
    set_int(w, id, 100);
    set_hits(w, id, if w.core_se() { 160 } else { 80 });
    set_stam(w, id, 250);
    set_mana(w, id, 0);
    set_damage(w, id, 10, 14);
    set_damage_type(w, id, ResistanceType::Physical, 60);
    set_damage_type(w, id, ResistanceType::Poison, 20);
    set_damage_type(w, id, ResistanceType::Energy, 20);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance(w, id, ResistanceType::Poison, 100);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);
    set_skill(w, id, SkillName::MagicResist, 70.0);
    set_skill(w, id, SkillName::Tactics, 90.0);
    set_skill(w, id, SkillName::Wrestling, 90.0);
    fame(w, id, 0);
    karma(w, id, 0);
    virtual_armor(w, id, 40);
    control_slots(w, id, if w.core_se() { 2 } else { 1 });
}
fn blade_spirits_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BLADE_SPIRITS,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    blade_spirits_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.BladeSpirits`.
    BLADE_SPIRITS, BLADE_SPIRITS_C = "Server.Mobiles.BladeSpirits" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(blade_spirits_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a blade spirit corpse"),
        default_name: |_w, _id| Some("a blade spirit"),
        delete_corpse_on_death: |w, _id| w.core_aos(),
        is_house_summonable: |_w, _id| true,
        dispel_difficulty: |_w, _id| 0.0,
        dispel_focus: |_w, _id| 20.0,
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(4)),
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
    templates::new_creature(w, &CENTAUR)
}
creature_leaf! {
    /// `Server.Mobiles.Centaur`.
    CENTAUR, CENTAUR_C = "Server.Mobiles.Centaur" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(centaur_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::FEY_AND_UNDEAD),
        meat: tp::meat,
        hides: tp::hides,
        hide_type: |_w, _id| HideType::Spined,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn energy_vortex_init(w: &mut World, id: MobileId) {
    if w.core_se() && rnd::below(w, 500) == 0 {
        body(w, id, 0xDC);
        hue(w, id, 0x76);
    } else {
        body(w, id, 164);
    }
    set_str(w, id, 200);
    set_dex(w, id, 200);
    set_int(w, id, 100);
    set_hits(w, id, if w.core_se() { 140 } else { 70 });
    set_stam(w, id, 250);
    set_mana(w, id, 0);
    set_damage(w, id, 14, 17);
    set_damage_type(w, id, ResistanceType::Physical, 0);
    set_damage_type(w, id, ResistanceType::Energy, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 60, 70);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 40, 50);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 90, 100);
    set_skill(w, id, SkillName::MagicResist, 99.9);
    set_skill(w, id, SkillName::Tactics, 100.0);
    set_skill(w, id, SkillName::Wrestling, 120.0);
    fame(w, id, 0);
    karma(w, id, 0);
    virtual_armor(w, id, 40);
    control_slots(w, id, if w.core_se() { 2 } else { 1 });
}
fn energy_vortex_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ENERGY_VORTEX,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    energy_vortex_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.EnergyVortex`.
    ENERGY_VORTEX, ENERGY_VORTEX_C = "Server.Mobiles.EnergyVortex" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(energy_vortex_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an energy vortex corpse"),
        delete_corpse_on_death: |w, id| read(w, id, false, |d| d.summoned),
        always_murderer: |_w, _id| true,
        dispel_difficulty: |_w, _id| 80.0,
        dispel_focus: |_w, _id| 20.0,
        default_name: |_w, _id| Some("an energy vortex"),
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(4)),
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
    templates::new_creature(w, &FROST_OOZE)
}
creature_leaf! {
    /// `Server.Mobiles.FrostOoze`.
    FROST_OOZE, FROST_OOZE_C = "Server.Mobiles.FrostOoze" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(frost_ooze_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        generate_loot: |w, id| {
            {
                let a0 = &loot_pack::GEMS;
                let a1 = rnd::min_max(w, 1, 2);
                add_loot_n(w, id, a0, a1);
            }
        },
    }
    m {
    }
}

pub(crate) fn plague_beast_init(w: &mut World, id: MobileId) {
    body(w, id, 775);
    set_str_range(w, id, 302, 500);
    set_dex(w, id, 80);
    set_int_range(w, id, 16, 20);
    set_hits_range(w, id, 318, 404);
    set_damage(w, id, 20, 24);
    set_damage_type(w, id, ResistanceType::Physical, 60);
    set_damage_type(w, id, ResistanceType::Poison, 40);
    set_resistance_range(w, id, ResistanceType::Physical, 45, 55);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 65, 75);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill(w, id, SkillName::MagicResist, 35.0);
    set_skill(w, id, SkillName::Tactics, 100.0);
    set_skill(w, id, SkillName::Wrestling, 100.0);
    fame(w, id, 13000);
    karma(w, id, -13000);
    virtual_armor(w, id, 30);
    crate::hand::plague_beast_ctor1(w, id);
    if rnd::double(w) < 0.80 {
        pack_new(w, id, "Server.Items.PlagueBeastGland");
    }
    crate::hand::plague_beast_ctor2(w, id);
    crate::hand::plague_beast_ctor3(w, id);
    crate::hand::plague_beast_ctor4(w, id);
}
fn plague_beast_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &PLAGUE_BEAST,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    plague_beast_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.PlagueBeast`.
    PLAGUE_BEAST, PLAGUE_BEAST_C = "Server.Mobiles.PlagueBeast" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(plague_beast_new), state crate::hand::plague_beast_serialize, crate::hand::plague_beast_deserialize;
    c {
        corpse_name: |_w, _id| Some("a plague beast corpse"),
        default_name: |_w, _id| Some("a plague beast"),
        auto_dispel: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(4)),
        on_gave_melee_attack: crate::hand::plague_beast_on_gave_melee_attack,
        on_damaged_by_spell: crate::hand::plague_beast_on_damaged_by_spell,
        on_got_melee_attack: crate::hand::plague_beast_on_got_melee_attack,
        on_think: crate::hand::plague_beast_on_think,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            {
                let a0 = &loot_pack::GEMS;
                let a1 = rnd::from(w, 1, 3);
                add_loot_n(w, id, a0, a1);
            }
        },
    }
    m {
        get_idle_sound: |_w, _id| 0x1BF,
        get_attack_sound: |_w, _id| 0x1C0,
        get_hurt_sound: |_w, _id| 0x1C1,
        get_death_sound: |_w, _id| 0x1C2,
    }
}

fn plague_beast_lord_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &PLAGUE_BEAST_LORD)
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
    templates::new_creature(w, &SAND_VORTEX)
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

pub(crate) fn slime_init(w: &mut World, id: MobileId) {
    body(w, id, 51);
    base_sound_id(w, id, 456);
    {
        let a0 = rnd::from(w, 2201, 24);
        hue(w, id, a0);
    }
    set_str_range(w, id, 22, 34);
    set_dex_range(w, id, 16, 21);
    set_int_range(w, id, 16, 20);
    set_hits_range(w, id, 15, 19);
    set_damage(w, id, 1, 5);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 5, 10);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 20);
    set_skill_range(w, id, SkillName::Poisoning, 30.1, 50.0);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 20.0);
    set_skill_range(w, id, SkillName::Tactics, 19.3, 34.0);
    set_skill_range(w, id, SkillName::Wrestling, 19.3, 34.0);
    fame(w, id, 300);
    karma(w, id, -300);
    virtual_armor(w, id, 8);
    tamable(w, id, true);
    control_slots(w, id, 1);
    min_tame_skill(w, id, 23.1);
}
fn slime_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SLIME,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    slime_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Slime`.
    SLIME, SLIME_C = "Server.Mobiles.Slime" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(slime_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a slimey corpse"),
        default_name: |_w, _id| Some("a slime"),
        poison_immune: |_w, _id| Some(PoisonId(0)),
        hit_poison: |_w, _id| Some(PoisonId(0)),
        favorite_food: |_w, _id| FoodType::MEAT | FoodType::FISH | FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY | FoodType::EGGS,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::poor(w));
            add_loot(w, id, &loot_pack::GEMS);
        },
    }
    m {
    }
}

fn agapite_elemental_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &AGAPITE_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.AgapiteElemental`.
    AGAPITE_ELEMENTAL, AGAPITE_ELEMENTAL_C = "Server.Mobiles.AgapiteElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(agapite_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        auto_dispel: |_w, _id| true,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn bronze_elemental_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &BRONZE_ELEMENTAL)
}
static BRONZE_ELEMENTAL_ABILITIES: [&MonsterAbility; 1] = [&abilities::POISON_GAS_COUNTER];
creature_leaf! {
    /// `Server.Mobiles.BronzeElemental`.
    BRONZE_ELEMENTAL, BRONZE_ELEMENTAL_C = "Server.Mobiles.BronzeElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bronze_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        auto_dispel: |_w, _id| true,
        treasure_map_level: tp::treasure_map_level,
        get_monster_abilities: |_, _| &BRONZE_ELEMENTAL_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn copper_elemental_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &COPPER_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.CopperElemental`.
    COPPER_ELEMENTAL, COPPER_ELEMENTAL_C = "Server.Mobiles.CopperElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(copper_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        bleed_immune: tp::bleed_immune,
        auto_dispel: |_w, _id| true,
        treasure_map_level: tp::treasure_map_level,
        alter_melee_damage_from: crate::hand::copper_elemental_alter_melee_damage_from,
        check_reflect: crate::hand::copper_elemental_check_reflect,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn dull_copper_elemental_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &DULL_COPPER_ELEMENTAL)
}
static DULL_COPPER_ELEMENTAL_ABILITIES: [&MonsterAbility; 1] = [&abilities::DEATH_EXPLOSION];
creature_leaf! {
    /// `Server.Mobiles.DullCopperElemental`.
    DULL_COPPER_ELEMENTAL, DULL_COPPER_ELEMENTAL_C = "Server.Mobiles.DullCopperElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(dull_copper_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: |_w, _id| true,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        get_monster_abilities: |_, _| &DULL_COPPER_ELEMENTAL_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn golden_elemental_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &GOLDEN_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.GoldenElemental`.
    GOLDEN_ELEMENTAL, GOLDEN_ELEMENTAL_C = "Server.Mobiles.GoldenElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(golden_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: |_w, _id| true,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn shadow_iron_elemental_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &SHADOW_IRON_ELEMENTAL)
}
creature_leaf! {
    /// `Server.Mobiles.ShadowIronElemental`.
    SHADOW_IRON_ELEMENTAL, SHADOW_IRON_ELEMENTAL_C = "Server.Mobiles.ShadowIronElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(shadow_iron_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: |_w, _id| true,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        poison_immune: tp::poison_immune,
        breath_immune: |_w, _id| true,
        alter_melee_damage_from: crate::hand::shadow_iron_elemental_alter_melee_damage_from,
        alter_damage_scalar_from: crate::hand::shadow_iron_elemental_alter_damage_scalar_from,
        alter_spell_damage_from: crate::hand::shadow_iron_elemental_alter_spell_damage_from,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn valorite_elemental_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &VALORITE_ELEMENTAL)
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
        auto_dispel: |_w, _id| true,
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
    templates::new_creature(w, &VERITE_ELEMENTAL)
}
static VERITE_ELEMENTAL_ABILITIES: [&MonsterAbility; 1] = [&abilities::DESTROY_EQUIPMENT];
creature_leaf! {
    /// `Server.Mobiles.VeriteElemental`.
    VERITE_ELEMENTAL, VERITE_ELEMENTAL_C = "Server.Mobiles.VeriteElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(verite_elemental_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        auto_dispel: |_w, _id| true,
        bleed_immune: tp::bleed_immune,
        treasure_map_level: tp::treasure_map_level,
        get_monster_abilities: |_, _| &VERITE_ELEMENTAL_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn reaper_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &REAPER)
}
creature_leaf! {
    /// `Server.Mobiles.Reaper`.
    REAPER, REAPER_C = "Server.Mobiles.Reaper" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(reaper_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        disallow_all_moves: |_w, _id| true,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

fn bogling_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &BOGLING)
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

pub(crate) fn corpser_init(w: &mut World, id: MobileId) {
    body(w, id, 8);
    base_sound_id(w, id, 684);
    set_str_range(w, id, 156, 180);
    set_dex_range(w, id, 26, 45);
    set_int_range(w, id, 26, 40);
    set_hits_range(w, id, 94, 108);
    set_mana(w, id, 0);
    set_damage(w, id, 10, 23);
    set_damage_type(w, id, ResistanceType::Physical, 60);
    set_damage_type(w, id, ResistanceType::Poison, 40);
    set_resistance_range(w, id, ResistanceType::Physical, 15, 20);
    set_resistance_range(w, id, ResistanceType::Fire, 15, 25);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 20);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 20.0);
    set_skill_range(w, id, SkillName::Tactics, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 45.1, 60.0);
    fame(w, id, 1000);
    karma(w, id, -1000);
    virtual_armor(w, id, 18);
    if rnd::double(w) < 0.25 {
        pack_new_amount(w, id, "Server.Items.Board", 10);
    } else {
        pack_new_amount(w, id, "Server.Items.Log", 10);
    }
    pack_new_amount(w, id, "Server.Items.MandrakeRoot", 3);
}
fn corpser_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &CORPSER,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    corpser_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Corpser`.
    CORPSER, CORPSER_C = "Server.Mobiles.Corpser" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(corpser_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a corpser corpse"),
        default_name: |_w, _id| Some("a corpser"),
        poison_immune: |_w, _id| Some(PoisonId(0)),
        disallow_all_moves: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

fn quagmire_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &QUAGMIRE)
}
creature_leaf! {
    /// `Server.Mobiles.Quagmire`.
    QUAGMIRE, QUAGMIRE_C = "Server.Mobiles.Quagmire" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(quagmire_new), state crate::hand::evil_mage_serialize, crate::hand::quagmire_deserialize;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        hit_poison: |_w, _id| Some(PoisonId(4)),
        hit_poison_chance: |_w, _id| 0.1,
        generate_loot: tp::generate_loot,
    }
    m {
        get_anger_sound: |_w, _id| 353,
    }
}

fn swamp_tentacle_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &SWAMP_TENTACLE)
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
    templates::new_creature(w, &WHIPPING_VINE)
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
    templates::new_creature(w, &ANCIENT_WYRM)
}
static ANCIENT_WYRM_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.AncientWyrm`.
    ANCIENT_WYRM, ANCIENT_WYRM_C = "Server.Mobiles.AncientWyrm" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ancient_wyrm_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: |_w, _id| true,
        auto_dispel: |_w, _id| true,
        hide_type: |_w, _id| HideType::Barbed,
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

pub(crate) fn deep_sea_serpent_init(w: &mut World, id: MobileId) {
    body(w, id, 150);
    base_sound_id(w, id, 447);
    {
        let a0 = rnd::from(w, 0x8A0, 5);
        hue(w, id, a0);
    }
    set_str_range(w, id, 251, 425);
    set_dex_range(w, id, 87, 135);
    set_int_range(w, id, 87, 155);
    set_hits_range(w, id, 151, 255);
    set_damage(w, id, 6, 14);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 70, 80);
    set_resistance_range(w, id, ResistanceType::Cold, 40, 50);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 15, 20);
    set_skill_range(w, id, SkillName::MagicResist, 60.1, 75.0);
    set_skill_range(w, id, SkillName::Tactics, 60.1, 70.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 70.0);
    fame(w, id, 6000);
    karma(w, id, -6000);
    virtual_armor(w, id, 60);
    can_swim(w, id, true);
    cant_walk(w, id, true);
    if rnd::bool(w) {
        pack_new_amount(w, id, "Server.Items.SulfurousAsh", 4);
    } else {
        pack_new_amount(w, id, "Server.Items.BlackPearl", 4);
    }
}
fn deep_sea_serpent_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &DEEP_SEA_SERPENT,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    deep_sea_serpent_init(w, id);
    id
}
static DEEP_SEA_SERPENT_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.DeepSeaSerpent`.
    DEEP_SEA_SERPENT, DEEP_SEA_SERPENT_C = "Server.Mobiles.DeepSeaSerpent" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(deep_sea_serpent_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a deep sea serpents corpse"),
        default_name: |_w, _id| Some("a deep sea serpent"),
        meat: |_w, _id| 1,
        scales: |_w, _id| 8,
        scale_type: |_w, _id| ScaleType::Blue,
        get_monster_abilities: |_, _| &DEEP_SEA_SERPENT_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
    }
}

fn dragon_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &DRAGON)
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
        hide_type: |_w, _id| HideType::Barbed,
        scales: tp::scales,
        scale_type: crate::hand::dragon_scale_type,
        favorite_food: |_w, _id| FoodType::MEAT,
        can_anger_on_tame: |_w, _id| true,
        can_fly: tp::can_fly,
        get_monster_abilities: |_, _| &DRAGON_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn ophidian_archmage_init(w: &mut World, id: MobileId) {
    crate::hand::ophidian_archmage_ctor1(w, id);
    body(w, id, 85);
    base_sound_id(w, id, 639);
    set_str_range(w, id, 281, 305);
    set_dex_range(w, id, 191, 215);
    set_int_range(w, id, 226, 250);
    set_hits_range(w, id, 169, 183);
    set_stam_range(w, id, 36, 45);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 35, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::EvalInt, 95.1, 100.0);
    set_skill_range(w, id, SkillName::Magery, 95.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 75.0, 97.5);
    set_skill_range(w, id, SkillName::Tactics, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Wrestling, 20.2, 60.0);
    fame(w, id, 11500);
    karma(w, id, -11500);
    virtual_armor(w, id, 44);
    pack_reg_range(w, id, 5, 15);
    pack_necro_reg_range(w, id, 5, 15);
}
fn ophidian_archmage_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &OPHIDIAN_ARCHMAGE,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ophidian_archmage_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.OphidianArchmage`.
    OPHIDIAN_ARCHMAGE, OPHIDIAN_ARCHMAGE_C = "Server.Mobiles.OphidianArchmage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ophidian_archmage_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an ophidian corpse"),
        meat: |_w, _id| 1,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::TERATHANS_AND_OPHIDIANS),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
            add_loot_n(w, id, &loot_pack::MED_SCROLLS, 2);
        },
    }
    m {
    }
}

pub(crate) fn ophidian_mage_init(w: &mut World, id: MobileId) {
    crate::hand::ophidian_mage_ctor1(w, id);
    body(w, id, 85);
    base_sound_id(w, id, 639);
    set_str_range(w, id, 181, 205);
    set_dex_range(w, id, 191, 215);
    set_int_range(w, id, 96, 120);
    set_hits_range(w, id, 109, 123);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 35);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 35, 45);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 35, 45);
    set_skill_range(w, id, SkillName::EvalInt, 85.1, 100.0);
    set_skill_range(w, id, SkillName::Magery, 85.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 75.0, 97.5);
    set_skill_range(w, id, SkillName::Tactics, 65.0, 87.5);
    set_skill_range(w, id, SkillName::Wrestling, 20.2, 60.0);
    fame(w, id, 4000);
    karma(w, id, -4000);
    virtual_armor(w, id, 30);
    pack_reg(w, id, 10);
}
fn ophidian_mage_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &OPHIDIAN_MAGE,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ophidian_mage_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.OphidianMage`.
    OPHIDIAN_MAGE, OPHIDIAN_MAGE_C = "Server.Mobiles.OphidianMage" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ophidian_mage_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an ophidian corpse"),
        meat: |_w, _id| 1,
        treasure_map_level: |_w, _id| 2,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::TERATHANS_AND_OPHIDIANS),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
            add_loot(w, id, &loot_pack::LOW_SCROLLS);
            add_loot(w, id, &loot_pack::MED_SCROLLS);
            add_loot(w, id, &loot_pack::POTIONS);
        },
    }
    m {
    }
}

fn ophidian_matriarch_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &OPHIDIAN_MATRIARCH)
}
creature_leaf! {
    /// `Server.Mobiles.OphidianMatriarch`.
    OPHIDIAN_MATRIARCH, OPHIDIAN_MATRIARCH_C = "Server.Mobiles.OphidianMatriarch" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ophidian_matriarch_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        poison_immune: tp::poison_immune,
        treasure_map_level: tp::treasure_map_level,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::TERATHANS_AND_OPHIDIANS),
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn serpentine_dragon_init(w: &mut World, id: MobileId) {
    body(w, id, 103);
    base_sound_id(w, id, 362);
    set_str_range(w, id, 111, 140);
    set_dex_range(w, id, 201, 220);
    set_int_range(w, id, 1001, 1040);
    set_hits(w, id, 480);
    set_damage(w, id, 5, 12);
    set_damage_type(w, id, ResistanceType::Physical, 75);
    set_damage_type(w, id, ResistanceType::Poison, 25);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 35);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::EvalInt, 100.1, 110.0);
    set_skill_range(w, id, SkillName::Magery, 110.1, 120.0);
    set_skill(w, id, SkillName::Meditation, 100.0);
    set_skill(w, id, SkillName::MagicResist, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 50.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 30.1, 100.0);
    fame(w, id, 15000);
    karma(w, id, 15000);
    virtual_armor(w, id, 36);
    crate::hand::serpentine_dragon_ctor1(w, id);
}
fn serpentine_dragon_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SERPENTINE_DRAGON,
        AIType::AiMage,
        FightMode::Evil,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    serpentine_dragon_init(w, id);
    id
}
static SERPENTINE_DRAGON_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.SerpentineDragon`.
    SERPENTINE_DRAGON, SERPENTINE_DRAGON_C = "Server.Mobiles.SerpentineDragon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(serpentine_dragon_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a dragon corpse"),
        default_name: |_w, _id| Some("a serpentine dragon"),
        reacquire_on_movement: |_w, _id| true,
        bonus_pet_damage_scalar: |w, _id| if w.core_se() { 3.0 } else { 1.0 },
        auto_dispel: |_w, _id| true,
        hide_type: |_w, _id| HideType::Barbed,
        hides: |_w, _id| 20,
        meat: |_w, _id| 19,
        scales: |_w, _id| 6,
        scale_type: |w, _id| if rnd::bool(w) { ScaleType::Black } else { ScaleType::White },
        treasure_map_level: |_w, _id| 4,
        on_got_melee_attack: crate::hand::serpentine_dragon_on_got_melee_attack,
        get_monster_abilities: |_, _| &SERPENTINE_DRAGON_ABILITIES,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::filthy_rich(w), 2);
            add_loot_n(w, id, &loot_pack::GEMS, 2);
        },
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
    templates::new_creature(w, &SHADOW_WYRM)
}
static SHADOW_WYRM_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.ShadowWyrm`.
    SHADOW_WYRM, SHADOW_WYRM_C = "Server.Mobiles.ShadowWyrm" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(shadow_wyrm_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: |_w, _id| true,
        auto_dispel: |_w, _id| true,
        poison_immune: tp::poison_immune,
        hit_poison: |_w, _id| Some(PoisonId(3)),
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        scales: tp::scales,
        scale_type: |_w, _id| ScaleType::Black,
        hide_type: |_w, _id| HideType::Barbed,
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
    templates::new_creature(w, &SKELETAL_DRAGON)
}
static SKELETAL_DRAGON_ABILITIES: [&MonsterAbility; 1] = [&abilities::COLD_BREATH];
creature_leaf! {
    /// `Server.Mobiles.SkeletalDragon`.
    SKELETAL_DRAGON, SKELETAL_DRAGON_C = "Server.Mobiles.SkeletalDragon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(skeletal_dragon_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: |_w, _id| true,
        bonus_pet_damage_scalar: |w, _id| if w.core_se() { 3.0 } else { 1.0 },
        auto_dispel: |_w, _id| true,
        poison_immune: tp::poison_immune,
        bleed_immune: tp::bleed_immune,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: |_w, _id| HideType::Barbed,
        get_monster_abilities: |_, _| &SKELETAL_DRAGON_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn white_wyrm_init(w: &mut World, id: MobileId) {
    {
        let a0 = if rnd::bool(w) { 180 } else { 49 };
        body(w, id, a0);
    }
    base_sound_id(w, id, 362);
    set_str_range(w, id, 721, 760);
    set_dex_range(w, id, 101, 130);
    set_int_range(w, id, 386, 425);
    set_hits_range(w, id, 433, 456);
    set_damage(w, id, 17, 25);
    set_damage_type(w, id, ResistanceType::Physical, 50);
    set_damage_type(w, id, ResistanceType::Cold, 50);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 70);
    set_resistance_range(w, id, ResistanceType::Fire, 15, 25);
    set_resistance_range(w, id, ResistanceType::Cold, 80, 90);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 50);
    set_skill_range(w, id, SkillName::EvalInt, 99.1, 100.0);
    set_skill_range(w, id, SkillName::Magery, 99.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 99.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 97.6, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 90.1, 100.0);
    fame(w, id, 18000);
    karma(w, id, -18000);
    virtual_armor(w, id, 64);
    tamable(w, id, true);
    control_slots(w, id, 3);
    min_tame_skill(w, id, 96.3);
}
fn white_wyrm_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &WHITE_WYRM,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    white_wyrm_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.WhiteWyrm`.
    WHITE_WYRM, WHITE_WYRM_C = "Server.Mobiles.WhiteWyrm" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(white_wyrm_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a white wyrm corpse"),
        default_name: |_w, _id| Some("a white wyrm"),
        reacquire_on_movement: |_w, _id| true,
        treasure_map_level: |_w, _id| 4,
        meat: |_w, _id| 19,
        hides: |_w, _id| 20,
        hide_type: |_w, _id| HideType::Barbed,
        scales: |_w, _id| 9,
        scale_type: |_w, _id| ScaleType::White,
        favorite_food: |_w, _id| FoodType::MEAT | FoodType::GOLD,
        can_anger_on_tame: |_w, _id| true,
        can_fly: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::filthy_rich(w), 2);
            add_loot(w, id, loot_pack::average(w));
            {
                let a0 = &loot_pack::GEMS;
                let a1 = rnd::from(w, 1, 5);
                add_loot_n(w, id, a0, a1);
            }
        },
    }
    m {
    }
}

fn drake_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &DRAKE)
}
static DRAKE_ABILITIES: [&MonsterAbility; 1] = [&abilities::FIRE_BREATH];
creature_leaf! {
    /// `Server.Mobiles.Drake`.
    DRAKE, DRAKE_C = "Server.Mobiles.Drake" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(drake_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: |_w, _id| true,
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: |_w, _id| HideType::Horned,
        scales: tp::scales,
        scale_type: crate::hand::drake_scale_type,
        favorite_food: |_w, _id| FoodType::MEAT | FoodType::FISH,
        can_fly: tp::can_fly,
        get_monster_abilities: |_, _| &DRAKE_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn kraken_init(w: &mut World, id: MobileId) {
    body(w, id, 77);
    base_sound_id(w, id, 353);
    set_str_range(w, id, 756, 780);
    set_dex_range(w, id, 226, 245);
    set_int_range(w, id, 26, 40);
    set_hits_range(w, id, 454, 468);
    set_mana(w, id, 0);
    set_damage(w, id, 19, 33);
    set_damage_type(w, id, ResistanceType::Physical, 70);
    set_damage_type(w, id, ResistanceType::Cold, 30);
    set_resistance_range(w, id, ResistanceType::Physical, 45, 55);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 20);
    set_skill_range(w, id, SkillName::MagicResist, 15.1, 20.0);
    set_skill_range(w, id, SkillName::Tactics, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Wrestling, 45.1, 60.0);
    fame(w, id, 11000);
    karma(w, id, -11000);
    virtual_armor(w, id, 50);
    can_swim(w, id, true);
    cant_walk(w, id, true);
    crate::hand::kraken_ctor1(w, id);
}
fn kraken_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &KRAKEN,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    kraken_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Kraken`.
    KRAKEN, KRAKEN_C = "Server.Mobiles.Kraken" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(kraken_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a krakens corpse"),
        default_name: |_w, _id| Some("a kraken"),
        treasure_map_level: |_w, _id| 4,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::rich(w));
        },
    }
    m {
        on_before_death: crate::hand::kraken_on_before_death,
    }
}

fn lizardman_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &LIZARDMAN)
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
        hide_type: |_w, _id| HideType::Spined,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn ophidian_knight_init(w: &mut World, id: MobileId) {
    crate::hand::ophidian_knight_ctor1(w, id);
    body(w, id, 86);
    base_sound_id(w, id, 634);
    set_str_range(w, id, 417, 595);
    set_dex_range(w, id, 166, 175);
    set_int_range(w, id, 46, 70);
    set_hits_range(w, id, 266, 342);
    set_mana(w, id, 0);
    set_damage(w, id, 16, 19);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 35, 45);
    set_resistance_range(w, id, ResistanceType::Poison, 90, 100);
    set_resistance_range(w, id, ResistanceType::Energy, 35, 45);
    set_skill_range(w, id, SkillName::Poisoning, 60.1, 80.0);
    set_skill_range(w, id, SkillName::MagicResist, 65.1, 80.0);
    set_skill_range(w, id, SkillName::Tactics, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 90.1, 100.0);
    fame(w, id, 10000);
    karma(w, id, -10000);
    virtual_armor(w, id, 40);
    pack_new(w, id, "Server.Items.LesserPoisonPotion");
}
fn ophidian_knight_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &OPHIDIAN_KNIGHT,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ophidian_knight_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.OphidianKnight`.
    OPHIDIAN_KNIGHT, OPHIDIAN_KNIGHT_C = "Server.Mobiles.OphidianKnight" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ophidian_knight_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an ophidian corpse"),
        meat: |_w, _id| 2,
        poison_immune: |_w, _id| Some(PoisonId(4)),
        hit_poison: |_w, _id| Some(PoisonId(4)),
        treasure_map_level: |_w, _id| 3,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::TERATHANS_AND_OPHIDIANS),
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::rich(w), 2);
        },
    }
    m {
    }
}

pub(crate) fn ophidian_warrior_init(w: &mut World, id: MobileId) {
    crate::hand::ophidian_warrior_ctor1(w, id);
    body(w, id, 86);
    base_sound_id(w, id, 634);
    set_str_range(w, id, 150, 320);
    set_dex_range(w, id, 94, 190);
    set_int_range(w, id, 64, 160);
    set_hits_range(w, id, 128, 155);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 11);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 35);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::MagicResist, 70.1, 85.0);
    set_skill_range(w, id, SkillName::Swords, 60.1, 85.0);
    set_skill_range(w, id, SkillName::Tactics, 75.1, 90.0);
    fame(w, id, 4500);
    karma(w, id, -4500);
    virtual_armor(w, id, 36);
}
fn ophidian_warrior_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &OPHIDIAN_WARRIOR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ophidian_warrior_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.OphidianWarrior`.
    OPHIDIAN_WARRIOR, OPHIDIAN_WARRIOR_C = "Server.Mobiles.OphidianWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ophidian_warrior_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an ophidian corpse"),
        meat: |_w, _id| 1,
        treasure_map_level: |_w, _id| 1,
        opposition_group: |_w, _id| Some(&muo_content::ai::opposition_group::TERATHANS_AND_OPHIDIANS),
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
            add_loot(w, id, loot_pack::average(w));
            add_loot(w, id, &loot_pack::GEMS);
        },
    }
    m {
    }
}

fn stone_harpy_new(w: &mut World) -> MobileId {
    templates::new_creature(w, &STONE_HARPY)
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
    templates::new_creature(w, &WYVERN)
}
creature_leaf! {
    /// `Server.Mobiles.Wyvern`.
    WYVERN, WYVERN_C = "Server.Mobiles.Wyvern" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(wyvern_new), encoded false;
    c {
        corpse_name: tp::corpse_name,
        default_name: tp::default_name,
        reacquire_on_movement: |_w, _id| true,
        poison_immune: tp::poison_immune,
        hit_poison: |_w, _id| Some(PoisonId(3)),
        treasure_map_level: tp::treasure_map_level,
        meat: tp::meat,
        hides: tp::hides,
        hide_type: |_w, _id| HideType::Horned,
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
    templates::new_creature(w, &RAPTOR)
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
        hide_type: |_w, _id| HideType::Horned,
        pack_instinct: |_w, _id| PackInstinct::OSTARD,
        get_weapon_ability: |_w, _id| Some(Ability::BleedAttack),
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
    templates::new_creature(w, &SLITH)
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
    templates::new_creature(w, &STONE_SLITH)
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
        hide_type: |_w, _id| HideType::Spined,
        get_weapon_ability: |_w, _id| Some(Ability::BleedAttack),
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
    templates::new_creature(w, &TOXIC_SLITH)
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
        hide_type: |_w, _id| HideType::Horned,
        get_monster_abilities: |_, _| &TOXIC_SLITH_ABILITIES,
        generate_loot: tp::generate_loot,
    }
    m {
    }
}

pub(crate) fn bake_kitsune_init(w: &mut World, id: MobileId) {
    body(w, id, 246);
    set_str_range(w, id, 171, 220);
    set_dex_range(w, id, 126, 145);
    set_int_range(w, id, 376, 425);
    set_hits_range(w, id, 301, 350);
    set_damage(w, id, 15, 22);
    set_damage_type(w, id, ResistanceType::Physical, 70);
    set_damage_type(w, id, ResistanceType::Energy, 30);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 60);
    set_resistance_range(w, id, ResistanceType::Fire, 70, 90);
    set_resistance_range(w, id, ResistanceType::Cold, 40, 60);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 60);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 60);
    set_skill_range(w, id, SkillName::EvalInt, 80.1, 90.0);
    set_skill_range(w, id, SkillName::Magery, 80.1, 90.0);
    set_skill_range(w, id, SkillName::MagicResist, 80.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 70.1, 90.0);
    set_skill_range(w, id, SkillName::Wrestling, 50.1, 55.0);
    fame(w, id, 8000);
    karma(w, id, -8000);
    tamable(w, id, true);
    control_slots(w, id, 2);
    min_tame_skill(w, id, 80.7);
    crate::hand::bake_kitsune_ctor1(w, id);
}
fn bake_kitsune_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &BAKE_KITSUNE,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    bake_kitsune_init(w, id);
    id
}
static BAKE_KITSUNE_ABILITIES: [&MonsterAbility; 1] = [&abilities::BLOOD_BATH_ATTACK];
creature_leaf! {
    /// `Server.Mobiles.BakeKitsune`.
    BAKE_KITSUNE, BAKE_KITSUNE_C = "Server.Mobiles.BakeKitsune" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(bake_kitsune_new), state crate::hand::bake_kitsune_serialize, crate::hand::bake_kitsune_deserialize;
    c {
        corpse_name: |_w, _id| Some("a bake kitsune corpse"),
        default_name: |_w, _id| Some("a bake kitsune"),
        meat: |_w, _id| 5,
        hides: |_w, _id| 10,
        hide_type: |_w, _id| HideType::Barbed,
        favorite_food: |_w, _id| FoodType::FISH,
        get_monster_abilities: |_, _| &BAKE_KITSUNE_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot(w, id, loot_pack::rich(w));
            add_loot_n(w, id, &loot_pack::MED_SCROLLS, 2);
        },
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

pub(crate) fn deathwatch_beetle_init(w: &mut World, id: MobileId) {
    body(w, id, 242);
    set_str_range(w, id, 136, 160);
    set_dex_range(w, id, 41, 52);
    set_int_range(w, id, 31, 40);
    set_hits_range(w, id, 121, 145);
    set_mana(w, id, 20);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 15, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 15, 30);
    set_resistance_range(w, id, ResistanceType::Poison, 50, 80);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 35);
    set_skill_range(w, id, SkillName::MagicResist, 50.1, 58.0);
    set_skill_range(w, id, SkillName::Tactics, 67.1, 77.0);
    set_skill_range(w, id, SkillName::Wrestling, 50.1, 60.0);
    set_skill_range(w, id, SkillName::Anatomy, 30.1, 34.0);
    fame(w, id, 1400);
    karma(w, id, -1400);
    crate::hand::deathwatch_beetle_ctor1(w, id);
    crate::hand::deathwatch_beetle_ctor2(w, id);
    tamable(w, id, true);
    min_tame_skill(w, id, 41.1);
    control_slots(w, id, 1);
}
fn deathwatch_beetle_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &DEATHWATCH_BEETLE,
        AIType::AiMelee,
        if w.core_ml() {
            FightMode::Aggressor
        } else {
            FightMode::Closest
        },
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    deathwatch_beetle_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.DeathwatchBeetle`.
    DEATHWATCH_BEETLE, DEATHWATCH_BEETLE_C = "Server.Mobiles.DeathwatchBeetle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(deathwatch_beetle_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a deathwatchbeetle corpse"),
        default_name: |_w, _id| Some("a deathwatch beetle"),
        hides: |_w, _id| 8,
        get_weapon_ability: |_w, _id| Some(Ability::CrushingBlow),
        alter_melee_damage_to: crate::hand::deathwatch_beetle_alter_melee_damage_to,
        generate_loot: |w, id| {
            add_loot_n(w, id, &loot_pack::LOW_SCROLLS, 1);
            add_loot_n(w, id, &loot_pack::POTIONS, 1);
        },
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

pub(crate) fn deathwatch_beetle_hatchling_init(w: &mut World, id: MobileId) {
    body(w, id, 242);
    set_str_range(w, id, 26, 50);
    set_dex_range(w, id, 41, 52);
    set_int_range(w, id, 21, 30);
    set_hits_range(w, id, 51, 60);
    set_mana(w, id, 20);
    set_damage(w, id, 2, 8);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 15, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 15, 30);
    set_resistance_range(w, id, ResistanceType::Poison, 20, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 35);
    set_skill_range(w, id, SkillName::Wrestling, 30.1, 40.0);
    set_skill_range(w, id, SkillName::Tactics, 47.1, 57.0);
    set_skill_range(w, id, SkillName::MagicResist, 30.1, 38.0);
    set_skill_range(w, id, SkillName::Anatomy, 20.1, 24.0);
    fame(w, id, 700);
    karma(w, id, -700);
    crate::hand::deathwatch_beetle_hatchling_ctor1(w, id);
    crate::hand::deathwatch_beetle_hatchling_ctor2(w, id);
}
fn deathwatch_beetle_hatchling_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &DEATHWATCH_BEETLE_HATCHLING,
        AIType::AiMelee,
        if w.core_ml() {
            FightMode::Aggressor
        } else {
            FightMode::Closest
        },
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    deathwatch_beetle_hatchling_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.DeathwatchBeetleHatchling`.
    DEATHWATCH_BEETLE_HATCHLING, DEATHWATCH_BEETLE_HATCHLING_C = "Server.Mobiles.DeathwatchBeetleHatchling" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(deathwatch_beetle_hatchling_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a deathwatchbeetle hatchling corpse"),
        default_name: |_w, _id| Some("a deathwatch beetle hatchling"),
        hides: |_w, _id| 8,
        generate_loot: |w, id| {
            add_loot_n(w, id, &loot_pack::LOW_SCROLLS, 1);
            add_loot_n(w, id, &loot_pack::POTIONS, 1);
        },
    }
    m {
        get_anger_sound: |_w, _id| 0x4F3,
        get_idle_sound: |_w, _id| 0x4F2,
        get_attack_sound: |_w, _id| 0x4F1,
        get_hurt_sound: |_w, _id| 0x4F4,
        get_death_sound: |_w, _id| 0x4F0,
    }
}

pub(crate) fn elite_ninja_init(w: &mut World, id: MobileId) {
    {
        let a0 = rnd::from(w, 2, 1000);
        speech_hue(w, id, a0);
    }
    {
        let a0 = race_skin_hue(w, "Human");
        hue(w, id, a0);
    }
    {
        let a0 = rnd::bool(w);
        female(w, id, a0);
    }
    body(w, id, if is_female(w, id) { 0x191 } else { 0x190 });
    set_hits_range(w, id, 251, 350);
    set_str_range(w, id, 126, 225);
    set_dex_range(w, id, 175, 275);
    set_int_range(w, id, 151, 165);
    set_damage(w, id, 12, 20);
    set_damage_type(w, id, ResistanceType::Physical, 65);
    set_damage_type(w, id, ResistanceType::Fire, 15);
    set_damage_type(w, id, ResistanceType::Poison, 15);
    set_damage_type(w, id, ResistanceType::Energy, 5);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 65);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 60);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 45);
    set_resistance_range(w, id, ResistanceType::Poison, 40, 60);
    set_resistance_range(w, id, ResistanceType::Energy, 35, 55);
    set_skill_range(w, id, SkillName::Anatomy, 105.0, 120.0);
    set_skill_range(w, id, SkillName::MagicResist, 80.0, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 115.0, 130.0);
    set_skill_range(w, id, SkillName::Wrestling, 95.0, 120.0);
    set_skill_range(w, id, SkillName::Fencing, 95.0, 120.0);
    set_skill_range(w, id, SkillName::Macing, 95.0, 120.0);
    set_skill_range(w, id, SkillName::Swords, 95.0, 120.0);
    set_skill_range(w, id, SkillName::Ninjitsu, 95.0, 120.0);
    fame(w, id, 8500);
    karma(w, id, -8500);
    add_new(w, id, "Server.Items.NinjaTabi");
    add_new(w, id, "Server.Items.LeatherNinjaJacket");
    add_new(w, id, "Server.Items.LeatherNinjaHood");
    add_new(w, id, "Server.Items.LeatherNinjaPants");
    add_new(w, id, "Server.Items.LeatherNinjaMitts");
    if rnd::double(w) < 0.33 {
        add_new(w, id, "Server.Items.SmokeBomb");
    }
    {
        let pick: Option<&str> = match rnd::below(w, 8) {
            0 => Some("Server.Items.Tessen"),
            1 => Some("Server.Items.Wakizashi"),
            2 => Some("Server.Items.Nunchaku"),
            3 => Some("Server.Items.Daisho"),
            4 => Some("Server.Items.Sai"),
            5 => Some("Server.Items.Tekagi"),
            6 => Some("Server.Items.Kama"),
            _ => Some("Server.Items.Katana"),
        };
        add_opt(w, id, pick);
    }
    assign_random_hair(w, id);
}
fn elite_ninja_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ELITE_NINJA,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    elite_ninja_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.EliteNinja`.
    ELITE_NINJA, ELITE_NINJA_C = "Server.Mobiles.EliteNinja" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(elite_ninja_new), encoded false;
    c {
        default_name: |_w, _id| Some("an elite ninja"),
        bard_immune: |_w, _id| true,
        always_murderer: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot(w, id, loot_pack::rich(w));
            add_loot_n(w, id, &loot_pack::GEMS, 2);
        },
    }
    m {
        click_title: |_w, _id| false,
        on_death: |w, id, c| {
            (BASE_CREATURE.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.BookOfNinjitsu");
        },
    }
}

pub(crate) fn fan_dancer_init(w: &mut World, id: MobileId) {
    body(w, id, 247);
    base_sound_id(w, id, 0x372);
    set_str_range(w, id, 301, 375);
    set_dex_range(w, id, 201, 255);
    set_int_range(w, id, 21, 25);
    set_hits_range(w, id, 351, 430);
    set_damage(w, id, 12, 17);
    set_damage_type(w, id, ResistanceType::Physical, 70);
    set_damage_type(w, id, ResistanceType::Fire, 10);
    set_damage_type(w, id, ResistanceType::Cold, 10);
    set_damage_type(w, id, ResistanceType::Poison, 10);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 60);
    set_resistance_range(w, id, ResistanceType::Fire, 50, 70);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 70);
    set_resistance_range(w, id, ResistanceType::Poison, 50, 70);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 60);
    set_skill_range(w, id, SkillName::MagicResist, 100.1, 110.0);
    set_skill_range(w, id, SkillName::Tactics, 85.1, 95.0);
    set_skill_range(w, id, SkillName::Wrestling, 85.1, 95.0);
    set_skill_range(w, id, SkillName::Anatomy, 85.1, 95.0);
    fame(w, id, 9000);
    karma(w, id, -9000);
    crate::hand::fan_dancer_ctor1(w, id);
    add_new(w, id, "Server.Items.Tessen");
    if rnd::double(w) < 0.02 {
        pack_new(w, id, "Server.Items.OrigamiPaper");
    }
}
fn fan_dancer_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &FAN_DANCER,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    fan_dancer_init(w, id);
    id
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
        corpse_name: |_w, _id| Some("a fan dancer corpse"),
        default_name: |_w, _id| Some("a fan dancer"),
        uncalmable: |_w, _id| true,
        get_monster_abilities: |_, _| &FAN_DANCER_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot(w, id, loot_pack::rich(w));
            add_loot_n(w, id, &loot_pack::GEMS, 2);
        },
    }
    m {
    }
}

pub(crate) fn fire_beetle_init(w: &mut World, id: MobileId) {
    set_stam(w, id, 100);
    set_str(w, id, 300);
    set_dex_range(w, id, 65, 100);
    set_int(w, id, 500);
    set_hits(w, id, 200);
    set_damage(w, id, 7, 20);
    set_damage_type(w, id, ResistanceType::Physical, 0);
    set_damage_type(w, id, ResistanceType::Fire, 100);
    set_resistance(w, id, ResistanceType::Physical, 40);
    set_resistance_range(w, id, ResistanceType::Fire, 70, 75);
    set_resistance(w, id, ResistanceType::Cold, 10);
    set_resistance(w, id, ResistanceType::Poison, 30);
    set_resistance(w, id, ResistanceType::Energy, 30);
    set_skill(w, id, SkillName::MagicResist, 90.0);
    set_skill(w, id, SkillName::Tactics, 100.0);
    set_skill(w, id, SkillName::Wrestling, 100.0);
    fame(w, id, 4000);
    karma(w, id, -4000);
    tamable(w, id, true);
    control_slots(w, id, 3);
    min_tame_skill(w, id, 93.9);
    {
        let a0 = "Server.Items.SulfurousAsh";
        let a1 = rnd::min_max(w, 16, 25);
        pack_new_amount(w, id, a0, a1);
    }
    pack_new_amount(w, id, "Server.Items.IronIngot", 2);
    hue(w, id, 0x489);
}
fn fire_beetle_new(w: &mut World) -> MobileId {
    let id = new_mount(
        w,
        &FIRE_BEETLE,
        0xA9,
        0x3E95,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    fire_beetle_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.FireBeetle`.
    FIRE_BEETLE, FIRE_BEETLE_C = "Server.Mobiles.FireBeetle" : BASE_MOUNT / BASE_MOUNT_C, ctor Some(fire_beetle_new), state crate::hand::fire_beetle_serialize, crate::hand::fire_beetle_deserialize;
    c {
        default_name: |_w, _id| Some("a fire beetle"),
        corpse_name: |_w, _id| Some("a fire beetle corpse"),
        subdue_before_tame: |_w, _id| true,
        stat_loss_after_tame: |_w, _id| true,
        reduce_speed_with_damage: |_w, _id| false,
        meat: |_w, _id| 16,
        favorite_food: |_w, _id| FoodType::MEAT,
        on_harmful_spell: crate::hand::fire_beetle_on_harmful_spell,
        override_bonding_reqs: |_w, _id| true,
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

pub(crate) fn kappa_init(w: &mut World, id: MobileId) {
    body(w, id, 240);
    set_str_range(w, id, 186, 230);
    set_dex_range(w, id, 51, 75);
    set_int_range(w, id, 41, 55);
    set_mana(w, id, 30);
    set_hits_range(w, id, 151, 180);
    set_damage(w, id, 6, 12);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 50);
    set_resistance_range(w, id, ResistanceType::Fire, 35, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 50);
    set_resistance_range(w, id, ResistanceType::Poison, 35, 50);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 30);
    set_skill_range(w, id, SkillName::MagicResist, 60.1, 70.0);
    set_skill_range(w, id, SkillName::Tactics, 79.1, 89.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 70.0);
    fame(w, id, 1700);
    karma(w, id, -1700);
    crate::hand::kappa_ctor1(w, id);
    crate::hand::kappa_ctor2(w, id);
    crate::hand::kappa_ctor3(w, id);
}
fn kappa_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &KAPPA,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    kappa_init(w, id);
    id
}
static KAPPA_ABILITIES: [&MonsterAbility; 1] = [&abilities::DRAIN_LIFE_ATTACK];
creature_leaf! {
    /// `Server.Mobiles.Kappa`.
    KAPPA, KAPPA_C = "Server.Mobiles.Kappa" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(kappa_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a kappa corpse"),
        default_name: |_w, _id| Some("a kappa"),
        new_harmful_item: crate::hand::kappa_new_harmful_item,
        get_monster_abilities: |_, _| &KAPPA_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::meager(w));
            add_loot(w, id, loot_pack::average(w));
        },
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
    templates::new_creature(w, &KAZE_KEMONO)
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

pub(crate) fn lady_of_the_snow_init(w: &mut World, id: MobileId) {
    body(w, id, 252);
    base_sound_id(w, id, 0x482);
    set_str_range(w, id, 276, 305);
    set_dex_range(w, id, 106, 125);
    set_int_range(w, id, 471, 495);
    set_hits_range(w, id, 596, 625);
    set_damage(w, id, 13, 20);
    set_damage_type(w, id, ResistanceType::Physical, 20);
    set_damage_type(w, id, ResistanceType::Cold, 80);
    set_resistance_range(w, id, ResistanceType::Physical, 45, 55);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 55);
    set_resistance_range(w, id, ResistanceType::Cold, 70, 90);
    set_resistance_range(w, id, ResistanceType::Poison, 60, 70);
    set_resistance_range(w, id, ResistanceType::Energy, 65, 85);
    set_skill_range(w, id, SkillName::Magery, 95.1, 110.0);
    set_skill_range(w, id, SkillName::MagicResist, 90.1, 105.0);
    set_skill_range(w, id, SkillName::Tactics, 80.1, 100.0);
    set_skill_range(w, id, SkillName::Wrestling, 80.1, 100.0);
    set_skill_range(w, id, SkillName::Necromancy, 90.0, 110.0);
    set_skill_range(w, id, SkillName::SpiritSpeak, 90.0, 110.0);
    fame(w, id, 15200);
    karma(w, id, -15200);
    pack_reg(w, id, 3);
    pack_new(w, id, "Server.Items.Necklace");
    crate::hand::lady_of_the_snow_ctor1(w, id);
}
fn lady_of_the_snow_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &LADY_OF_THE_SNOW,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    lady_of_the_snow_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.LadyOfTheSnow`.
    LADY_OF_THE_SNOW, LADY_OF_THE_SNOW_C = "Server.Mobiles.LadyOfTheSnow" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(lady_of_the_snow_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a lady of the snow corpse"),
        default_name: |_w, _id| Some("a lady of the snow"),
        bleed_immune: |_w, _id| true,
        can_rummage_corpses: |_w, _id| true,
        treasure_map_level: |_w, _id| 4,
        on_gave_melee_attack: crate::hand::lady_of_the_snow_on_gave_melee_attack,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot(w, id, loot_pack::rich(w));
        },
    }
    m {
        get_death_sound: |_w, _id| 0x370,
    }
}

pub(crate) fn oni_init(w: &mut World, id: MobileId) {
    body(w, id, 241);
    set_str_range(w, id, 801, 910);
    set_dex_range(w, id, 151, 300);
    set_int_range(w, id, 171, 195);
    set_hits_range(w, id, 401, 530);
    set_damage(w, id, 14, 20);
    set_damage_type(w, id, ResistanceType::Physical, 70);
    set_damage_type(w, id, ResistanceType::Fire, 10);
    set_damage_type(w, id, ResistanceType::Energy, 20);
    set_resistance_range(w, id, ResistanceType::Physical, 65, 80);
    set_resistance_range(w, id, ResistanceType::Fire, 50, 70);
    set_resistance_range(w, id, ResistanceType::Cold, 35, 50);
    set_resistance_range(w, id, ResistanceType::Poison, 45, 70);
    set_resistance_range(w, id, ResistanceType::Energy, 45, 65);
    set_skill_range(w, id, SkillName::EvalInt, 100.1, 125.0);
    set_skill_range(w, id, SkillName::Magery, 96.1, 106.0);
    set_skill_range(w, id, SkillName::Anatomy, 85.1, 95.0);
    set_skill_range(w, id, SkillName::MagicResist, 85.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 86.1, 101.0);
    set_skill_range(w, id, SkillName::Wrestling, 90.1, 100.0);
    fame(w, id, 12000);
    karma(w, id, -12000);
    crate::hand::oni_ctor1(w, id);
}
fn oni_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &ONI,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    oni_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Oni`.
    ONI, ONI_C = "Server.Mobiles.Oni" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(oni_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("an oni corpse"),
        default_name: |_w, _id| Some("an oni"),
        can_rummage_corpses: |_w, _id| true,
        treasure_map_level: |_w, _id| 4,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::filthy_rich(w), 3);
        },
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
    templates::new_creature(w, &RAI_JU)
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

pub(crate) fn revenant_lion_init(w: &mut World, id: MobileId) {
    body(w, id, 251);
    set_str_range(w, id, 276, 325);
    set_dex_range(w, id, 156, 175);
    set_int_range(w, id, 76, 105);
    set_hits_range(w, id, 251, 280);
    set_damage(w, id, 18, 24);
    set_damage_type(w, id, ResistanceType::Physical, 30);
    set_damage_type(w, id, ResistanceType::Cold, 30);
    set_damage_type(w, id, ResistanceType::Poison, 10);
    set_damage_type(w, id, ResistanceType::Energy, 30);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 60);
    set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 60);
    set_resistance_range(w, id, ResistanceType::Poison, 55, 65);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 50);
    set_skill_range(w, id, SkillName::EvalInt, 80.1, 90.0);
    set_skill_range(w, id, SkillName::Magery, 80.1, 90.0);
    set_skill_range(w, id, SkillName::Poisoning, 120.1, 130.0);
    set_skill_range(w, id, SkillName::MagicResist, 70.1, 90.0);
    set_skill_range(w, id, SkillName::Tactics, 60.1, 80.0);
    set_skill_range(w, id, SkillName::Wrestling, 80.1, 88.0);
    fame(w, id, 4000);
    karma(w, id, -4000);
    pack_necro_reg_range(w, id, 6, 8);
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
}
fn revenant_lion_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &REVENANT_LION,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    revenant_lion_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.RevenantLion`.
    REVENANT_LION, REVENANT_LION_C = "Server.Mobiles.RevenantLion" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(revenant_lion_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a revenant lion corpse"),
        default_name: |_w, _id| Some("a Revenant Lion"),
        bleed_immune: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(2)),
        hit_poison: |_w, _id| Some(PoisonId(2)),
        get_weapon_ability: |_w, _id| Some(Ability::BleedAttack),
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::rich(w), 2);
            add_loot_n(w, id, &loot_pack::MED_SCROLLS, 2);
        },
    }
    m {
        get_anger_sound: |_w, _id| 0x518,
        get_idle_sound: |_w, _id| 0x517,
        get_attack_sound: |_w, _id| 0x516,
        get_hurt_sound: |_w, _id| 0x519,
        get_death_sound: |_w, _id| 0x515,
    }
}

pub(crate) fn ronin_init(w: &mut World, id: MobileId) {
    {
        let a0 = rnd::from(w, 2, 1000);
        speech_hue(w, id, a0);
    }
    {
        let a0 = race_skin_hue(w, "Human");
        hue(w, id, a0);
    }
    {
        let a0 = rnd::bool(w);
        female(w, id, a0);
    }
    body(w, id, if is_female(w, id) { 0x191 } else { 0x190 });
    set_str_range(w, id, 326, 375);
    set_dex_range(w, id, 31, 45);
    set_int_range(w, id, 101, 110);
    set_hits_range(w, id, 301, 400);
    set_mana_range(w, id, 101, 110);
    set_damage(w, id, 17, 25);
    set_damage_type(w, id, ResistanceType::Physical, 90);
    set_damage_type(w, id, ResistanceType::Poison, 10);
    set_resistance_range(w, id, ResistanceType::Physical, 55, 75);
    set_resistance_range(w, id, ResistanceType::Fire, 40, 60);
    set_resistance_range(w, id, ResistanceType::Cold, 35, 55);
    set_resistance_range(w, id, ResistanceType::Poison, 50, 70);
    set_resistance_range(w, id, ResistanceType::Energy, 55, 75);
    set_skill_range(w, id, SkillName::MagicResist, 42.6, 57.5);
    set_skill_range(w, id, SkillName::Tactics, 115.1, 130.0);
    set_skill_range(w, id, SkillName::Wrestling, 92.6, 107.5);
    set_skill_range(w, id, SkillName::Anatomy, 110.1, 125.0);
    set_skill_range(w, id, SkillName::Fencing, 92.6, 107.5);
    set_skill_range(w, id, SkillName::Macing, 92.6, 107.5);
    set_skill_range(w, id, SkillName::Swords, 92.6, 107.5);
    fame(w, id, 8500);
    karma(w, id, -8500);
    add_new(w, id, "Server.Items.SamuraiTabi");
    add_new(w, id, "Server.Items.LeatherHiroSode");
    add_new(w, id, "Server.Items.LeatherDo");
    crate::hand::ronin_ctor1(w, id);
    crate::hand::ronin_ctor2(w, id);
    if rnd::double(w) < 0.80 {
        add_new(w, id, "Server.Items.NoDachi");
    } else {
        add_new(w, id, "Server.Items.Halberd");
    }
    pack_new(w, id, "Server.Items.Wakizashi");
    pack_new(w, id, "Server.Items.Longsword");
    assign_random_hair(w, id);
}
fn ronin_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &RONIN,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    ronin_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Ronin`.
    RONIN, RONIN_C = "Server.Mobiles.Ronin" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(ronin_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a ronin corpse"),
        default_name: |_w, _id| Some("a ronin"),
        always_murderer: |_w, _id| true,
        bard_immune: |_w, _id| true,
        can_rummage_corpses: |_w, _id| true,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot(w, id, loot_pack::rich(w));
            add_loot_n(w, id, &loot_pack::GEMS, 2);
        },
    }
    m {
        click_title: |_w, _id| false,
        on_death: |w, id, c| {
            (BASE_CREATURE.on_death)(w, id, c);
            drop_new(w, c, "Server.Items.BookOfBushido");
        },
    }
}

pub(crate) fn rune_beetle_init(w: &mut World, id: MobileId) {
    body(w, id, 244);
    set_str_range(w, id, 401, 460);
    set_dex_range(w, id, 121, 170);
    set_int_range(w, id, 376, 450);
    set_hits_range(w, id, 301, 360);
    set_damage(w, id, 15, 22);
    set_damage_type(w, id, ResistanceType::Physical, 20);
    set_damage_type(w, id, ResistanceType::Poison, 10);
    set_damage_type(w, id, ResistanceType::Energy, 70);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 65);
    set_resistance_range(w, id, ResistanceType::Fire, 35, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 35, 50);
    set_resistance_range(w, id, ResistanceType::Poison, 75, 95);
    set_resistance_range(w, id, ResistanceType::Energy, 40, 60);
    set_skill_range(w, id, SkillName::EvalInt, 100.1, 125.0);
    set_skill_range(w, id, SkillName::Magery, 100.1, 110.0);
    set_skill_range(w, id, SkillName::Poisoning, 120.1, 140.0);
    set_skill_range(w, id, SkillName::MagicResist, 95.1, 110.0);
    set_skill_range(w, id, SkillName::Tactics, 78.1, 93.0);
    set_skill_range(w, id, SkillName::Wrestling, 70.1, 77.5);
    fame(w, id, 15000);
    karma(w, id, -15000);
    crate::hand::rune_beetle_ctor1(w, id);
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
    tamable(w, id, true);
    control_slots(w, id, 3);
    min_tame_skill(w, id, 93.9);
}
fn rune_beetle_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &RUNE_BEETLE,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    rune_beetle_init(w, id);
    id
}
static RUNE_BEETLE_ABILITIES: [&MonsterAbility; 1] = [&abilities::RUNE_CORRUPTION];
creature_leaf! {
    /// `Server.Mobiles.RuneBeetle`.
    RUNE_BEETLE, RUNE_BEETLE_C = "Server.Mobiles.RuneBeetle" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(rune_beetle_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a rune beetle corpse"),
        default_name: |_w, _id| Some("a rune beetle"),
        poison_immune: |_w, _id| Some(PoisonId(2)),
        hit_poison: |_w, _id| Some(PoisonId(2)),
        favorite_food: |_w, _id| FoodType::FRUITS_AND_VEGGIES | FoodType::GRAINS_AND_HAY,
        can_anger_on_tame: |_w, _id| true,
        get_weapon_ability: |_w, _id| Some(Ability::BleedAttack),
        get_monster_abilities: |_, _| &RUNE_BEETLE_ABILITIES,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::filthy_rich(w), 2);
            add_loot_n(w, id, &loot_pack::MED_SCROLLS, 1);
        },
    }
    m {
        get_anger_sound: |_w, _id| 0x4E8,
        get_idle_sound: |_w, _id| 0x4E7,
        get_attack_sound: |_w, _id| 0x4E6,
        get_hurt_sound: |_w, _id| 0x4E9,
        get_death_sound: |_w, _id| 0x4E5,
    }
}

pub(crate) fn tsuki_wolf_init(w: &mut World, id: MobileId) {
    body(w, id, 250);
    {
        let a0 = if rnd::below(w, 3) == 0 {
            rnd::from(w, 1801, 108)
        } else {
            0
        };
        hue(w, id, a0);
    }
    set_str_range(w, id, 401, 450);
    set_dex_range(w, id, 151, 200);
    set_int_range(w, id, 66, 76);
    set_hits_range(w, id, 376, 450);
    set_mana(w, id, 40);
    set_damage(w, id, 14, 18);
    set_damage_type(w, id, ResistanceType::Physical, 90);
    set_damage_type(w, id, ResistanceType::Cold, 5);
    set_damage_type(w, id, ResistanceType::Energy, 5);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 60);
    set_resistance_range(w, id, ResistanceType::Fire, 50, 70);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 70);
    set_resistance_range(w, id, ResistanceType::Poison, 50, 70);
    set_resistance_range(w, id, ResistanceType::Energy, 50, 70);
    set_skill_range(w, id, SkillName::Anatomy, 65.1, 72.0);
    set_skill_range(w, id, SkillName::MagicResist, 65.1, 70.0);
    set_skill_range(w, id, SkillName::Tactics, 95.1, 110.0);
    set_skill_range(w, id, SkillName::Wrestling, 97.6, 107.5);
    fame(w, id, 8500);
    karma(w, id, -8500);
    crate::hand::tsuki_wolf_ctor1(w, id);
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
}
fn tsuki_wolf_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &TSUKI_WOLF,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    tsuki_wolf_init(w, id);
    id
}
static TSUKI_WOLF_ABILITIES: [&MonsterAbility; 1] = [&abilities::BLOOD_BATH_ATTACK];
creature_leaf! {
    /// `Server.Mobiles.TsukiWolf`.
    TSUKI_WOLF, TSUKI_WOLF_C = "Server.Mobiles.TsukiWolf" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(tsuki_wolf_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a tsuki wolf corpse"),
        default_name: |_w, _id| Some("a tsuki wolf"),
        meat: |_w, _id| 4,
        hides: |_w, _id| 25,
        favorite_food: |_w, _id| FoodType::MEAT,
        get_monster_abilities: |_, _| &TSUKI_WOLF_ABILITIES,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::average(w));
            add_loot(w, id, loot_pack::rich(w));
        },
    }
    m {
        get_anger_sound: |_w, _id| 0x52D,
        get_idle_sound: |_w, _id| 0x52C,
        get_attack_sound: |_w, _id| 0x52B,
        get_hurt_sound: |_w, _id| 0x52E,
        get_death_sound: |_w, _id| 0x52A,
    }
}

pub(crate) fn yamandon_init(w: &mut World, id: MobileId) {
    body(w, id, 249);
    set_str_range(w, id, 786, 930);
    set_dex_range(w, id, 251, 365);
    set_int_range(w, id, 101, 115);
    set_hits_range(w, id, 1601, 1800);
    set_damage(w, id, 19, 35);
    set_damage_type(w, id, ResistanceType::Physical, 70);
    set_damage_type(w, id, ResistanceType::Poison, 20);
    set_damage_type(w, id, ResistanceType::Energy, 10);
    set_resistance_range(w, id, ResistanceType::Physical, 65, 85);
    set_resistance_range(w, id, ResistanceType::Fire, 70, 90);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 70);
    set_resistance_range(w, id, ResistanceType::Poison, 50, 70);
    set_resistance_range(w, id, ResistanceType::Energy, 50, 70);
    set_skill_range(w, id, SkillName::Anatomy, 115.1, 130.0);
    set_skill_range(w, id, SkillName::MagicResist, 117.6, 132.5);
    set_skill_range(w, id, SkillName::Poisoning, 120.1, 140.0);
    set_skill_range(w, id, SkillName::Tactics, 117.1, 132.0);
    set_skill_range(w, id, SkillName::Wrestling, 112.6, 132.5);
    fame(w, id, 22000);
    karma(w, id, -22000);
    crate::hand::yamandon_ctor1(w, id);
    crate::hand::yamandon_ctor2(w, id);
}
fn yamandon_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &YAMANDON,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    yamandon_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.Yamandon`.
    YAMANDON, YAMANDON_C = "Server.Mobiles.Yamandon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(yamandon_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a yamandon corpse"),
        default_name: |_w, _id| Some("a yamandon"),
        reacquire_on_movement: |_w, _id| true,
        poison_immune: |_w, _id| Some(PoisonId(4)),
        hit_poison: |w, _id| if rnd::bool(w) { Some(PoisonId(3)) } else { Some(PoisonId(4)) },
        treasure_map_level: |_w, _id| 5,
        hides: |_w, _id| 20,
        get_weapon_ability: |_w, _id| Some(Ability::DoubleStrike),
        on_damaged_by_spell: crate::hand::yamandon_on_damaged_by_spell,
        on_got_melee_attack: crate::hand::yamandon_on_got_melee_attack,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::ultra_rich(w));
            add_loot_n(w, id, loot_pack::filthy_rich(w), 2);
            add_loot_n(w, id, &loot_pack::GEMS, 6);
        },
    }
    m {
        get_attack_sound: |_w, _id| 1260,
        get_anger_sound: |_w, _id| 1262,
        get_death_sound: |_w, _id| 1259,
        get_hurt_sound: |_w, _id| 1263,
        get_idle_sound: |_w, _id| 1261,
    }
}

pub(crate) fn yomotsu_elder_init(w: &mut World, id: MobileId) {
    body(w, id, 255);
    base_sound_id(w, id, 0x452);
    set_str_range(w, id, 686, 830);
    set_dex_range(w, id, 251, 365);
    set_int_range(w, id, 17, 31);
    set_hits_range(w, id, 801, 900);
    set_damage(w, id, 19, 27);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 65, 85);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 45, 65);
    set_resistance_range(w, id, ResistanceType::Poison, 35, 55);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 50);
    set_skill_range(w, id, SkillName::Anatomy, 115.1, 130.0);
    set_skill_range(w, id, SkillName::MagicResist, 100.1, 120.0);
    set_skill_range(w, id, SkillName::Tactics, 115.1, 130.0);
    set_skill_range(w, id, SkillName::Wrestling, 110.1, 130.0);
    fame(w, id, 12000);
    karma(w, id, -12000);
    pack_new(w, id, "Server.Items.GreenGourd");
    pack_new(w, id, "Server.Items.ExecutionersAxe");
    {
        let pick: Option<&str> = match rnd::below(w, 3) {
            0 => Some("Server.Items.LongPants"),
            1 => Some("Server.Items.ShortPants"),
            _ => None,
        };
        pack_opt(w, id, pick);
    }
    {
        let pick: Option<&str> = match rnd::below(w, 6) {
            0 => Some("Server.Items.Shoes"),
            1 => Some("Server.Items.Sandals"),
            2 => Some("Server.Items.Boots"),
            3 => Some("Server.Items.ThighBoots"),
            _ => None,
        };
        pack_opt(w, id, pick);
    }
    crate::hand::yomotsu_elder_ctor1(w, id);
}
fn yomotsu_elder_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &YOMOTSU_ELDER,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    yomotsu_elder_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.YomotsuElder`.
    YOMOTSU_ELDER, YOMOTSU_ELDER_C = "Server.Mobiles.YomotsuElder" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(yomotsu_elder_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a wrinkly yomotsu corpse"),
        default_name: |_w, _id| Some("a yomotsu elder"),
        favorite_food: |_w, _id| FoodType::FISH,
        meat: |_w, _id| 1,
        can_rummage_corpses: |_w, _id| true,
        treasure_map_level: |_w, _id| 5,
        get_weapon_ability: |_w, _id| Some(Ability::DoubleStrike),
        on_gave_melee_attack: crate::hand::yomotsu_elder_on_gave_melee_attack,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::filthy_rich(w), 3);
            add_loot_n(w, id, &loot_pack::GEMS, 2);
        },
    }
    m {
        get_idle_sound: |_w, _id| 0x42A,
        get_attack_sound: |_w, _id| 0x435,
        get_hurt_sound: |_w, _id| 0x436,
        get_death_sound: |_w, _id| 0x43A,
    }
}

pub(crate) fn yomotsu_priest_init(w: &mut World, id: MobileId) {
    body(w, id, 253);
    base_sound_id(w, id, 0x452);
    set_str_range(w, id, 486, 530);
    set_dex_range(w, id, 101, 115);
    set_int_range(w, id, 601, 670);
    set_hits_range(w, id, 486, 530);
    set_damage(w, id, 8, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 65, 85);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 45, 65);
    set_resistance_range(w, id, ResistanceType::Poison, 35, 55);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 50);
    set_skill_range(w, id, SkillName::EvalInt, 92.6, 107.5);
    set_skill_range(w, id, SkillName::Magery, 105.1, 115.0);
    set_skill_range(w, id, SkillName::Meditation, 100.1, 110.0);
    set_skill_range(w, id, SkillName::MagicResist, 112.6, 122.5);
    set_skill_range(w, id, SkillName::Tactics, 55.1, 105.0);
    set_skill_range(w, id, SkillName::Wrestling, 47.6, 57.5);
    fame(w, id, 9000);
    karma(w, id, -9000);
    pack_new(w, id, "Server.Items.GreenGourd");
    pack_new(w, id, "Server.Items.ExecutionersAxe");
    crate::hand::yomotsu_priest_ctor1(w, id);
    crate::hand::yomotsu_priest_ctor2(w, id);
    crate::hand::yomotsu_priest_ctor3(w, id);
}
fn yomotsu_priest_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &YOMOTSU_PRIEST,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    yomotsu_priest_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.YomotsuPriest`.
    YOMOTSU_PRIEST, YOMOTSU_PRIEST_C = "Server.Mobiles.YomotsuPriest" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(yomotsu_priest_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a glowing yomotsu corpse"),
        default_name: |_w, _id| Some("a yomotsu priest"),
        favorite_food: |_w, _id| FoodType::FISH,
        meat: |_w, _id| 1,
        can_rummage_corpses: |_w, _id| true,
        get_weapon_ability: |_w, _id| Some(Ability::DoubleStrike),
        on_gave_melee_attack: crate::hand::yomotsu_priest_on_gave_melee_attack,
        generate_loot: |w, id| {
            add_loot(w, id, loot_pack::filthy_rich(w));
            add_loot(w, id, loot_pack::rich(w));
            add_loot_n(w, id, &loot_pack::GEMS, 4);
        },
    }
    m {
        get_idle_sound: |_w, _id| 0x42A,
        get_attack_sound: |_w, _id| 0x435,
        get_hurt_sound: |_w, _id| 0x436,
        get_death_sound: |_w, _id| 0x43A,
    }
}

pub(crate) fn yomotsu_warrior_init(w: &mut World, id: MobileId) {
    body(w, id, 245);
    base_sound_id(w, id, 0x452);
    set_str_range(w, id, 486, 530);
    set_dex_range(w, id, 151, 165);
    set_int_range(w, id, 17, 31);
    set_hits_range(w, id, 486, 530);
    set_mana_range(w, id, 17, 31);
    set_damage(w, id, 8, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 65, 85);
    set_resistance_range(w, id, ResistanceType::Fire, 30, 50);
    set_resistance_range(w, id, ResistanceType::Cold, 45, 65);
    set_resistance_range(w, id, ResistanceType::Poison, 35, 55);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 50);
    set_skill_range(w, id, SkillName::Anatomy, 85.1, 95.0);
    set_skill_range(w, id, SkillName::MagicResist, 82.6, 90.5);
    set_skill_range(w, id, SkillName::Tactics, 95.1, 105.0);
    set_skill_range(w, id, SkillName::Wrestling, 97.6, 107.5);
    fame(w, id, 4200);
    karma(w, id, -4200);
    pack_new(w, id, "Server.Items.GreenGourd");
    pack_new(w, id, "Server.Items.ExecutionersAxe");
    if rnd::bool(w) {
        pack_new(w, id, "Server.Items.LongPants");
    } else {
        pack_new(w, id, "Server.Items.ShortPants");
    }
    crate::hand::yomotsu_warrior_ctor1(w, id);
    crate::hand::yomotsu_warrior_ctor2(w, id);
}
fn yomotsu_warrior_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &YOMOTSU_WARRIOR,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    yomotsu_warrior_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.YomotsuWarrior`.
    YOMOTSU_WARRIOR, YOMOTSU_WARRIOR_C = "Server.Mobiles.YomotsuWarrior" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(yomotsu_warrior_new), encoded false;
    c {
        corpse_name: |_w, _id| Some("a yomotsu corpse"),
        default_name: |_w, _id| Some("a yomotsu warrior"),
        favorite_food: |_w, _id| FoodType::FISH,
        meat: |_w, _id| 1,
        can_rummage_corpses: |_w, _id| true,
        treasure_map_level: |_w, _id| 3,
        get_weapon_ability: |_w, _id| Some(Ability::DoubleStrike),
        on_gave_melee_attack: crate::hand::yomotsu_warrior_on_gave_melee_attack,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::rich(w), 2);
            add_loot_n(w, id, &loot_pack::GEMS, 2);
        },
    }
    m {
        get_idle_sound: |_w, _id| 0x42A,
        get_attack_sound: |_w, _id| 0x435,
        get_hurt_sound: |_w, _id| 0x436,
        get_death_sound: |_w, _id| 0x43A,
    }
}

pub(crate) fn summoned_air_elemental_init(w: &mut World, id: MobileId) {
    body(w, id, 13);
    hue(w, id, 0x4001);
    base_sound_id(w, id, 655);
    if w.core_aos() {
        set_str(w, id, 200);
        set_dex(w, id, 200);
        set_int(w, id, 100);
        set_hits(w, id, 150);
        set_stam(w, id, 50);
        set_damage(w, id, 6, 9);
        set_damage_type(w, id, ResistanceType::Physical, 50);
        set_damage_type(w, id, ResistanceType::Energy, 50);
        set_resistance_range(w, id, ResistanceType::Physical, 40, 50);
        set_resistance_range(w, id, ResistanceType::Fire, 30, 40);
        set_resistance_range(w, id, ResistanceType::Cold, 35, 45);
        set_resistance_range(w, id, ResistanceType::Poison, 50, 60);
        set_resistance_range(w, id, ResistanceType::Energy, 70, 80);
        set_skill(w, id, SkillName::Meditation, 90.0);
        set_skill(w, id, SkillName::EvalInt, 70.0);
        set_skill(w, id, SkillName::Magery, 70.0);
        set_skill(w, id, SkillName::MagicResist, 60.0);
        set_skill(w, id, SkillName::Tactics, 100.0);
        set_skill(w, id, SkillName::Wrestling, 80.0);
    } else {
        set_str_range(w, id, 126, 155);
        set_dex_range(w, id, 166, 185);
        set_int_range(w, id, 101, 125);
        set_hits_range(w, id, 76, 93);
        set_damage(w, id, 8, 10);
        set_damage_type(w, id, ResistanceType::Physical, 20);
        set_damage_type(w, id, ResistanceType::Cold, 40);
        set_damage_type(w, id, ResistanceType::Energy, 40);
        set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
        set_resistance_range(w, id, ResistanceType::Fire, 15, 25);
        set_resistance_range(w, id, ResistanceType::Cold, 10, 20);
        set_resistance_range(w, id, ResistanceType::Poison, 10, 20);
        set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
        set_skill_range(w, id, SkillName::EvalInt, 60.1, 75.0);
        set_skill_range(w, id, SkillName::Magery, 60.1, 75.0);
        set_skill_range(w, id, SkillName::MagicResist, 60.1, 75.0);
        set_skill_range(w, id, SkillName::Tactics, 60.1, 80.0);
        set_skill_range(w, id, SkillName::Wrestling, 60.1, 80.0);
    }
    virtual_armor(w, id, 40);
    control_slots(w, id, 2);
}
fn summoned_air_elemental_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SUMMONED_AIR_ELEMENTAL,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    summoned_air_elemental_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SummonedAirElemental`.
    SUMMONED_AIR_ELEMENTAL, SUMMONED_AIR_ELEMENTAL_C = "Server.Mobiles.SummonedAirElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(summoned_air_elemental_new), state crate::hand::evil_mage_serialize, crate::hand::summoned_air_elemental_deserialize;
    c {
        delete_corpse_on_death: |w, id| read(w, id, false, |d| d.summoned),
        corpse_name: |_w, _id| Some("an air elemental corpse"),
        default_name: |_w, _id| Some("an air elemental"),
        bleed_immune: |_w, _id| true,
        dispel_difficulty: |_w, _id| 117.5,
        dispel_focus: |_w, _id| 45.0,
    }
    m {
    }
}

pub(crate) fn summoned_daemon_init(w: &mut World, id: MobileId) {
    name_random(w, id, "daemon");
    body(w, id, if w.core_aos() { 10 } else { 9 });
    base_sound_id(w, id, 357);
    if w.core_aos() {
        set_str(w, id, 200);
        set_dex(w, id, 110);
        set_int(w, id, 150);
        set_damage(w, id, 14, 21);
        set_damage_type(w, id, ResistanceType::Physical, 0);
        set_damage_type(w, id, ResistanceType::Poison, 100);
        set_resistance_range(w, id, ResistanceType::Physical, 45, 55);
        set_resistance_range(w, id, ResistanceType::Fire, 50, 60);
        set_resistance_range(w, id, ResistanceType::Cold, 20, 30);
        set_resistance_range(w, id, ResistanceType::Poison, 70, 80);
        set_resistance_range(w, id, ResistanceType::Energy, 40, 50);
        set_skill_range(w, id, SkillName::EvalInt, 90.1, 100.0);
        set_skill_range(w, id, SkillName::Meditation, 90.1, 100.0);
        set_skill_range(w, id, SkillName::Magery, 90.1, 100.0);
        set_skill_range(w, id, SkillName::MagicResist, 90.1, 100.0);
        set_skill(w, id, SkillName::Tactics, 100.0);
        set_skill_range(w, id, SkillName::Wrestling, 98.1, 99.0);
    } else {
        set_str_range(w, id, 476, 505);
        set_dex_range(w, id, 76, 95);
        set_int_range(w, id, 301, 325);
        set_hits_range(w, id, 286, 303);
        set_damage(w, id, 7, 14);
        set_damage_type(w, id, ResistanceType::Physical, 100);
        set_resistance_range(w, id, ResistanceType::Physical, 45, 60);
        set_resistance_range(w, id, ResistanceType::Fire, 50, 60);
        set_resistance_range(w, id, ResistanceType::Cold, 30, 40);
        set_resistance_range(w, id, ResistanceType::Poison, 20, 30);
        set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
        set_skill_range(w, id, SkillName::EvalInt, 70.1, 80.0);
        set_skill_range(w, id, SkillName::Magery, 70.1, 80.0);
        set_skill_range(w, id, SkillName::MagicResist, 85.1, 95.0);
        set_skill_range(w, id, SkillName::Tactics, 70.1, 80.0);
        set_skill_range(w, id, SkillName::Wrestling, 60.1, 80.0);
    }
    virtual_armor(w, id, 58);
    control_slots(w, id, if w.core_se() { 4 } else { 5 });
}
fn summoned_daemon_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SUMMONED_DAEMON,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    summoned_daemon_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SummonedDaemon`.
    SUMMONED_DAEMON, SUMMONED_DAEMON_C = "Server.Mobiles.SummonedDaemon" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(summoned_daemon_new), encoded false;
    c {
        delete_corpse_on_death: |w, id| read(w, id, false, |d| d.summoned),
        corpse_name: |_w, _id| Some("a daemon corpse"),
        dispel_difficulty: |_w, _id| 125.0,
        dispel_focus: |_w, _id| 45.0,
        poison_immune: |_w, _id| Some(PoisonId(1)),
        can_fly: |_w, _id| true,
    }
    m {
    }
}

pub(crate) fn summoned_earth_elemental_init(w: &mut World, id: MobileId) {
    body(w, id, 14);
    base_sound_id(w, id, 268);
    if w.core_aos() {
        set_str(w, id, 200);
        set_dex(w, id, 70);
        set_int(w, id, 70);
        set_hits(w, id, 180);
        set_damage(w, id, 14, 21);
        set_damage_type(w, id, ResistanceType::Physical, 100);
        set_resistance_range(w, id, ResistanceType::Physical, 65, 75);
        set_resistance_range(w, id, ResistanceType::Fire, 40, 50);
        set_resistance_range(w, id, ResistanceType::Cold, 40, 50);
        set_resistance_range(w, id, ResistanceType::Poison, 40, 50);
        set_resistance_range(w, id, ResistanceType::Energy, 40, 50);
        set_skill(w, id, SkillName::MagicResist, 65.0);
        set_skill(w, id, SkillName::Tactics, 100.0);
        set_skill(w, id, SkillName::Wrestling, 90.0);
    } else {
        set_str_range(w, id, 126, 155);
        set_dex_range(w, id, 66, 85);
        set_int_range(w, id, 71, 92);
        set_hits_range(w, id, 76, 93);
        set_damage(w, id, 9, 16);
        set_damage_type(w, id, ResistanceType::Physical, 100);
        set_resistance_range(w, id, ResistanceType::Physical, 30, 35);
        set_resistance_range(w, id, ResistanceType::Fire, 10, 20);
        set_resistance_range(w, id, ResistanceType::Cold, 10, 20);
        set_resistance_range(w, id, ResistanceType::Poison, 15, 25);
        set_resistance_range(w, id, ResistanceType::Energy, 15, 25);
        set_skill_range(w, id, SkillName::MagicResist, 50.1, 95.0);
        set_skill_range(w, id, SkillName::Tactics, 60.1, 100.0);
        set_skill_range(w, id, SkillName::Wrestling, 60.1, 100.0);
    }
    virtual_armor(w, id, 34);
    control_slots(w, id, 2);
}
fn summoned_earth_elemental_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SUMMONED_EARTH_ELEMENTAL,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    summoned_earth_elemental_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SummonedEarthElemental`.
    SUMMONED_EARTH_ELEMENTAL, SUMMONED_EARTH_ELEMENTAL_C = "Server.Mobiles.SummonedEarthElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(summoned_earth_elemental_new), encoded false;
    c {
        delete_corpse_on_death: |w, id| read(w, id, false, |d| d.summoned),
        corpse_name: |_w, _id| Some("an earth elemental corpse"),
        default_name: |_w, _id| Some("an earth elemental"),
        bleed_immune: |_w, _id| true,
        dispel_difficulty: |_w, _id| 117.5,
        dispel_focus: |_w, _id| 45.0,
    }
    m {
    }
}

pub(crate) fn summoned_fire_elemental_init(w: &mut World, id: MobileId) {
    body(w, id, 15);
    base_sound_id(w, id, 838);
    if w.core_aos() {
        set_str(w, id, 200);
        set_dex(w, id, 200);
        set_int(w, id, 100);
        set_damage(w, id, 9, 14);
        set_damage_type(w, id, ResistanceType::Physical, 0);
        set_damage_type(w, id, ResistanceType::Fire, 100);
        set_resistance_range(w, id, ResistanceType::Physical, 50, 60);
        set_resistance_range(w, id, ResistanceType::Fire, 70, 80);
        set_resistance_range(w, id, ResistanceType::Cold, 0, 10);
        set_resistance_range(w, id, ResistanceType::Poison, 50, 60);
        set_resistance_range(w, id, ResistanceType::Energy, 50, 60);
        set_skill(w, id, SkillName::EvalInt, 90.0);
        set_skill(w, id, SkillName::Magery, 90.0);
        set_skill(w, id, SkillName::MagicResist, 85.0);
        set_skill(w, id, SkillName::Tactics, 100.0);
        set_skill(w, id, SkillName::Wrestling, 92.0);
    } else {
        set_str_range(w, id, 126, 155);
        set_dex_range(w, id, 166, 185);
        set_int_range(w, id, 101, 125);
        set_hits_range(w, id, 76, 93);
        set_damage(w, id, 7, 9);
        set_damage_type(w, id, ResistanceType::Physical, 25);
        set_damage_type(w, id, ResistanceType::Fire, 75);
        set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
        set_resistance_range(w, id, ResistanceType::Fire, 60, 80);
        set_resistance_range(w, id, ResistanceType::Cold, 5, 10);
        set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
        set_resistance_range(w, id, ResistanceType::Energy, 30, 40);
        set_skill_range(w, id, SkillName::EvalInt, 60.1, 75.0);
        set_skill_range(w, id, SkillName::Magery, 60.1, 75.0);
        set_skill_range(w, id, SkillName::MagicResist, 75.2, 105.0);
        set_skill_range(w, id, SkillName::Tactics, 80.1, 100.0);
        set_skill_range(w, id, SkillName::Wrestling, 70.1, 100.0);
    }
    virtual_armor(w, id, 40);
    control_slots(w, id, 4);
    add_new(w, id, "Server.Items.LightSource");
}
fn summoned_fire_elemental_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SUMMONED_FIRE_ELEMENTAL,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    summoned_fire_elemental_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SummonedFireElemental`.
    SUMMONED_FIRE_ELEMENTAL, SUMMONED_FIRE_ELEMENTAL_C = "Server.Mobiles.SummonedFireElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(summoned_fire_elemental_new), encoded false;
    c {
        delete_corpse_on_death: |w, id| read(w, id, false, |d| d.summoned),
        corpse_name: |_w, _id| Some("a fire elemental corpse"),
        default_name: |_w, _id| Some("a fire elemental"),
        bleed_immune: |_w, _id| true,
        dispel_difficulty: |_w, _id| 117.5,
        dispel_focus: |_w, _id| 45.0,
    }
    m {
    }
}

pub(crate) fn summoned_water_elemental_init(w: &mut World, id: MobileId) {
    body(w, id, 16);
    base_sound_id(w, id, 278);
    if w.core_aos() {
        set_str(w, id, 200);
        set_dex(w, id, 70);
        set_int(w, id, 100);
        set_hits(w, id, 165);
        set_damage(w, id, 12, 16);
        set_damage_type(w, id, ResistanceType::Physical, 0);
        set_damage_type(w, id, ResistanceType::Cold, 100);
        set_resistance_range(w, id, ResistanceType::Physical, 50, 60);
        set_resistance_range(w, id, ResistanceType::Fire, 20, 30);
        set_resistance_range(w, id, ResistanceType::Cold, 70, 80);
        set_resistance_range(w, id, ResistanceType::Poison, 45, 55);
        set_resistance_range(w, id, ResistanceType::Energy, 40, 50);
        set_skill(w, id, SkillName::Meditation, 90.0);
        set_skill(w, id, SkillName::EvalInt, 80.0);
        set_skill(w, id, SkillName::Magery, 80.0);
        set_skill(w, id, SkillName::MagicResist, 75.0);
        set_skill(w, id, SkillName::Tactics, 100.0);
        set_skill(w, id, SkillName::Wrestling, 85.0);
    } else {
        set_str_range(w, id, 126, 155);
        set_dex_range(w, id, 66, 85);
        set_int_range(w, id, 101, 125);
        set_hits_range(w, id, 76, 93);
        set_damage(w, id, 7, 9);
        set_damage_type(w, id, ResistanceType::Physical, 100);
        set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
        set_resistance_range(w, id, ResistanceType::Fire, 10, 25);
        set_resistance_range(w, id, ResistanceType::Cold, 10, 25);
        set_resistance_range(w, id, ResistanceType::Poison, 60, 70);
        set_resistance_range(w, id, ResistanceType::Energy, 5, 10);
        set_skill_range(w, id, SkillName::EvalInt, 60.1, 75.0);
        set_skill_range(w, id, SkillName::Magery, 60.1, 75.0);
        set_skill_range(w, id, SkillName::MagicResist, 100.1, 115.0);
        set_skill_range(w, id, SkillName::Tactics, 50.1, 70.0);
        set_skill_range(w, id, SkillName::Wrestling, 50.1, 70.0);
    }
    virtual_armor(w, id, 40);
    control_slots(w, id, 3);
    can_swim(w, id, true);
}
fn summoned_water_elemental_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &SUMMONED_WATER_ELEMENTAL,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    summoned_water_elemental_init(w, id);
    id
}
creature_leaf! {
    /// `Server.Mobiles.SummonedWaterElemental`.
    SUMMONED_WATER_ELEMENTAL, SUMMONED_WATER_ELEMENTAL_C = "Server.Mobiles.SummonedWaterElemental" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(summoned_water_elemental_new), encoded false;
    c {
        delete_corpse_on_death: |w, id| read(w, id, false, |d| d.summoned),
        corpse_name: |_w, _id| Some("a water elemental corpse"),
        default_name: |_w, _id| Some("a water elemental"),
        bleed_immune: |_w, _id| true,
        dispel_difficulty: |_w, _id| 117.5,
        dispel_focus: |_w, _id| 45.0,
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
