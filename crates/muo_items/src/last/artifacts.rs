//! The artifact and reward classes with logic of their own: Minor Artifacts (ML and older), Champion Artifacts and
//! New Haven Quest Rewards.

use crate::armor::armor_leaf;
use crate::armor::leaves::*;
use crate::armor::special::*;
use crate::armor::{hue, skill_bonus};
use crate::clothing::clothing_leaf;
use crate::clothing::leaves::*;
use crate::simple::{random, random_bool, weapon_art};
use crate::weapons::*;
use muo_content::aos::{AosAttribute, AosWeaponAttribute, AttrKind};
use muo_content::craft::CraftResource;
use muo_content::items::armor::{ArmorClass, ArmorMaterialType};
use muo_content::items::clothing::{BASE_OUTER_TORSO, BASE_OUTER_TORSO_CLASS, new_clothing};
use muo_content::items::slayer::SlayerName;
use muo_content::items::{attr, same, set_attr};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{ItemType, LootType, deps, ops};
use muo_server::race::ALLOW_ELVES_ONLY;
use muo_server::skills::SkillName;
use muo_server::type_registry::TypeRegistry;

// ---- BurglarsBandana, DreadPirateHat ----

clothing_leaf! {
    /// `Server.Items.BurglarsBandana`.
    BURGLARS_BANDANA, BURGLARS_BANDANA_CLASS, burglars_bandana_new = "Server.Items.BurglarsBandana" : BANDANA / BANDANA_CLASS, bandana_new, hue 0, encoded false;
    @init |w, id| {
        let h = if random_bool(w) { 0x58C } else { 0x10 };
        hue(w, id, h);
        skill_bonus(w, id, 0, SkillName::Stealing, 10.0);
        skill_bonus(w, id, 1, SkillName::Stealth, 10.0);
        skill_bonus(w, id, 2, SkillName::Snooping, 10.0);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusDex as u32, 5);
    };
    base_physical_resistance: 10,
    base_fire_resistance: 5,
    base_cold_resistance: 7,
    base_poison_resistance: 10,
    base_energy_resistance: 10,
    init_min_hits: same(255),
    init_max_hits: same(255),
}

/// `DreadPirateHat._combatSkills`.
const COMBAT_SKILLS: [SkillName; 5] = [
    SkillName::Tactics,
    SkillName::Archery,
    SkillName::Swords,
    SkillName::Macing,
    SkillName::Fencing,
];

clothing_leaf! {
    /// `Server.Items.DreadPirateHat`.
    DREAD_PIRATE_HAT, DREAD_PIRATE_HAT_CLASS, dread_pirate_hat_new = "Server.Items.DreadPirateHat" : TRICORNE_HAT / TRICORNE_HAT_CLASS, tricorne_hat_new, hue 0, encoded false;
    @init |w, id| {
        hue(w, id, 0x497);
        let i = random(w, COMBAT_SKILLS.len() as i32) as usize;
        skill_bonus(w, id, 0, COMBAT_SKILLS[i], 10.0);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusDex as u32, 8);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::NightSight as u32, 1);
    };
    base_cold_resistance: 14,
    base_poison_resistance: 10,
    init_min_hits: same(255),
    init_max_hits: same(255),
}

// ---- Minor Artifacts/ML ----

armor_leaf! {
    /// `Server.Items.AegisOfGrace`.
    AEGIS_OF_GRACE, AEGIS_OF_GRACE_CLASS, aegis_of_grace_new = "Server.Items.AegisOfGrace" : DRAGON_HELM / DRAGON_HELM_CLASS, dragon_helm_new, encoded true;
    @template;
    material_type: ArmorMaterialType::Dragon,
    default_resource: CraftResource::Iron,
    required_races: Some(ALLOW_ELVES_ONLY),
}

armor_leaf! {
    /// `Server.Items.FeyLeggings`.
    FEY_LEGGINGS, FEY_LEGGINGS_CLASS, fey_leggings_new = "Server.Items.FeyLeggings" : CHAIN_LEGS / CHAIN_LEGS_CLASS, chain_legs_new, encoded true;
    @template;
    required_races: Some(ALLOW_ELVES_ONLY),
}

clothing_leaf! {
    /// `Server.Items.PadsOfTheCuSidhe` (`base(0x47E)`).
    PADS_OF_THE_CU_SIDHE, PADS_OF_THE_CU_SIDHE_CLASS, pads_of_the_cu_sidhe_new = "Server.Items.PadsOfTheCuSidhe" : FUR_BOOTS / FUR_BOOTS_CLASS, fur_boots_new, hue 0x47E, encoded true;
}

clothing_leaf! {
    /// `Server.Items.RobeOfTheEclipse`.
    ROBE_OF_THE_ECLIPSE, ROBE_OF_THE_ECLIPSE_CLASS, robe_of_the_eclipse_new = "Server.Items.RobeOfTheEclipse" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, |w, ty, _| new_clothing(w, ty, 0x1F03, 0x486), hue 0, encoded true;
    @template;
    flippable: Some((0x1F03, 0x1F04)),
    default_weight: Some(3.0),
}

clothing_leaf! {
    /// `Server.Items.RobeOfTheEquinox`.
    ROBE_OF_THE_EQUINOX, ROBE_OF_THE_EQUINOX_CLASS, robe_of_the_equinox_new = "Server.Items.RobeOfTheEquinox" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, |w, ty, _| new_clothing(w, ty, 0x1F04, 0xD6), hue 0, encoded true;
    @template;
    flippable: Some((0x1F03, 0x1F04)),
    default_weight: Some(3.0),
}

weapon_art! {
    /// `Server.Items.WildfireBow`.
    WILDFIRE_BOW, WILDFIRE_BOW_CLASS, wildfire_bow_new = "Server.Items.WildfireBow" : ELVEN_COMPOSITE_LONGBOW / ELVEN_COMPOSITE_LONGBOW_CLASS, via elven_composite_longbow_new, enc;
    @init |w, id| {
        hue(w, id, 0x489);
        skill_bonus(w, id, 0, SkillName::Archery, 10.0);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::ResistFireBonus as u32, 25);
        crate::weapons::ctor::set_velocity(w, id, 15);
    };
    label_number: 1075044,
    get_damage_types: |_, _, _| muo_content::items::weapon_combat::DamageTypes { phys: 0, fire: 100, cold: 0, pois: 0, nrgy: 0, chaos: 0, direct: 0 },
}

weapon_art! {
    /// `Server.Items.Windsong`.
    WINDSONG, WINDSONG_CLASS, windsong_new = "Server.Items.Windsong" : MAGICAL_SHORTBOW / MAGICAL_SHORTBOW_CLASS, via magical_shortbow_new, enc;
    @init |w, id| {
        hue(w, id, 0xF7);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 35);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::SelfRepair as u32, 3);
        crate::weapons::ctor::set_velocity(w, id, 25);
    };
    label_number: 1075031,
}

// ---- Minor Artifacts ----

armor_leaf! {
    /// `Server.Items.ShieldOfInvulnerability`; `Validate(m) => true` makes the guild checks of OrderShield pass.
    SHIELD_OF_INVULNERABILITY, SHIELD_OF_INVULNERABILITY_CLASS, shield_of_invulnerability_new = "Server.Items.ShieldOfInvulnerability" : ORDER_SHIELD / ORDER_SHIELD_CLASS, order_shield_new, encoded false;
    @template;
    @ty {
        on_equip: muo_content::items::armor::BASE_SHIELD.on_equip,
        on_single_click: muo_content::items::armor::BASE_SHIELD.on_single_click,
    };
}

weapon_art! {
    /// `Server.Items.WrathOfTheDryad`.
    WRATH_OF_THE_DRYAD, WRATH_OF_THE_DRYAD_CLASS, wrath_of_the_dryad_new = "Server.Items.WrathOfTheDryad" : GNARLED_STAFF / GNARLED_STAFF_CLASS, via gnarled_staff_new, v0;
    @init |w, id| {
        hue(w, id, 0x29C);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechMana as u32, 50);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLightning as u32, 33);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 15);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 40);
    };
    primary_ability: Some(muo_content::items::weapon::Ability::ConcussionBlow),
    secondary_ability: Some(muo_content::items::weapon::Ability::ParalyzingBlow),
    label_number: 1070853,
    get_damage_types: |_, _, _| muo_content::items::weapon_combat::DamageTypes { phys: 0, fire: 0, cold: 0, pois: 100, nrgy: 0, chaos: 0, direct: 0 },
}

// ---- Champion Artifacts ----

clothing_leaf! {
    /// `Server.Items.CaptainJohnsHat`.
    CAPTAIN_JOHNS_HAT, CAPTAIN_JOHNS_HAT_CLASS, captain_johns_hat_new = "Server.Items.CaptainJohnsHat" : TRICORNE_HAT / TRICORNE_HAT_CLASS, tricorne_hat_new, hue 0, encoded false;
    @template;
    base_physical_resistance: 2,
    base_fire_resistance: 6,
    base_cold_resistance: 9,
    base_poison_resistance: 7,
    base_energy_resistance: 23,
    init_min_hits: same(150),
    init_max_hits: same(150),
    can_fortify: false,
}

/// `DetectiveBoots.Level`: kept in a field that is not saved and restores it from `Attributes.BonusInt`
/// after a load, so it is always `BonusInt - 2` within 0..=2.
fn detective_level(w: &World, id: ItemId) -> i32 {
    (attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32) - 2).clamp(0, 2)
}

clothing_leaf! {
    /// `Server.Items.DetectiveBoots`.
    DETECTIVE_BOOTS, DETECTIVE_BOOTS_CLASS, detective_boots_new = "Server.Items.DetectiveBoots" : BOOTS / BOOTS_CLASS, boots_new, hue 0, encoded false;
    @init |w, id| {
        hue(w, id, 0x455);
        let level = crate::simple::random_min_max(w, 0, 2);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, 2 + level);
    };
    @ty {
        label_number: |w, id| 1094894 + detective_level(w, id),
    };
    init_min_hits: same(150),
    init_max_hits: same(150),
    can_fortify: false,
}

clothing_leaf! {
    /// `Server.Items.EmbroideredOakLeafCloak`.
    EMBROIDERED_OAK_LEAF_CLOAK, EMBROIDERED_OAK_LEAF_CLOAK_CLASS, embroidered_oak_leaf_cloak_new = "Server.Items.EmbroideredOakLeafCloak" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, |w, ty, _| new_clothing(w, ty, 0x2684, 0), hue 0, encoded false;
    @init |w, id| {
        hue(w, id, 0x483);
        muo_content::items::clothing::set_str_requirement(w, id, 0);
        skill_bonus(w, id, 0, SkillName::Stealth, 5.0);
    };
    init_min_hits: same(150),
    init_max_hits: same(150),
    can_fortify: false,
}

clothing_leaf! {
    /// `Server.Items.TheMostKnowledgePerson`.
    THE_MOST_KNOWLEDGE_PERSON, THE_MOST_KNOWLEDGE_PERSON_CLASS, the_most_knowledge_person_new = "Server.Items.TheMostKnowledgePerson" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, |w, ty, _| new_clothing(w, ty, 0x2684, 0), hue 0, encoded false;
    @init |w, id| {
        hue(w, id, 0x117);
        muo_content::items::clothing::set_str_requirement(w, id, 0);
        let bonus = 3 + crate::simple::random_min_max(w, 0, 2);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, bonus);
    };
    init_min_hits: same(150),
    init_max_hits: same(150),
    can_fortify: false,
    can_be_blessed: false,
}

clothing_leaf! {
    /// `Server.Items.TheRobeOfBritanniaAri`.
    THE_ROBE_OF_BRITANNIA_ARI, THE_ROBE_OF_BRITANNIA_ARI_CLASS, the_robe_of_britannia_ari_new = "Server.Items.TheRobeOfBritanniaAri" : BASE_OUTER_TORSO / BASE_OUTER_TORSO_CLASS, |w, ty, _| new_clothing(w, ty, 0x2684, 0), hue 0, encoded false;
    @init |w, id| {
        hue(w, id, 0x48b);
        muo_content::items::clothing::set_str_requirement(w, id, 0);
    };
    base_physical_resistance: 10,
    init_min_hits: same(150),
    init_max_hits: same(150),
    can_fortify: false,
}

armor_leaf! {
    /// `Server.Items.OrcChieftainHelm`.
    ORC_CHIEFTAIN_HELM, ORC_CHIEFTAIN_HELM_CLASS, orc_chieftain_helm_new = "Server.Items.OrcChieftainHelm" : ORC_HELM / ORC_HELM_CLASS, orc_helm_new, encoded false;
    @init |w, id| {
        hue(w, id, 0x2a3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::Luck as u32, 100);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenHits as u32, 3);
        if random_bool(w) {
            set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, 30);
        } else {
            set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 30);
        }
    };
    can_fortify: false,
}

armor_leaf! {
    /// `Server.Items.ShroudOfDeciet`.
    SHROUD_OF_DECIET, SHROUD_OF_DECIET_CLASS, shroud_of_deciet_new = "Server.Items.ShroudOfDeciet" : BONE_CHEST / BONE_CHEST_CLASS, bone_chest_new, encoded false;
    @template;
    can_fortify: false,
}

// ---- New Haven Quest Rewards ----

armor_leaf! {
    /// `Server.Items.EscutcheonDeAriadne`.
    ESCUTCHEON_DE_ARIADNE, ESCUTCHEON_DE_ARIADNE_CLASS, escutcheon_de_ariadne_new = "Server.Items.EscutcheonDeAriadne" : METAL_KITE_SHIELD / METAL_KITE_SHIELD_CLASS, metal_kite_shield_new, encoded true;
    @template;
}

weapon_art! {
    /// `Server.Items.JacobsPickaxe`.
    JACOBS_PICKAXE, JACOBS_PICKAXE_CLASS, jacobs_pickaxe_new = "Server.Items.JacobsPickaxe" : PICKAXE / PICKAXE_CLASS, via pickaxe_new, enc;
    @init |w, id| {
        crate::weapons::ctor::set_uses_remaining(w, id, 20);
        ops::set_loot_type(w, id, LootType::Blessed);
        skill_bonus(w, id, 0, SkillName::Mining, 10.0);
    };
    label_number: 1077758,
}

// ---- hand-written: statuettes, instruments, talismans, quivers, tools, spellbook ----

/// `Server.Items.PhillipsWoodenSteed`.
pub static PHILLIPS_WOODEN_STEED: ItemType = ItemType {
    full_name: "Server.Items.PhillipsWoodenSteed",
    parent: Some(&crate::special::statuette::MONSTER_STATUETTE),
    construct: Some(|w| {
        let id = crate::special::statuette::monster_statuette_new_ty(w, &PHILLIPS_WOODEN_STEED, 23);
        ops::set_loot_type(w, id, LootType::Regular);
        id
    }),
    force_show_properties: |w, _| deps::opl_enabled(w),
    serialize: |w, id, out| {
        (crate::special::statuette::MONSTER_STATUETTE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (crate::special::statuette::MONSTER_STATUETTE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..crate::special::statuette::MONSTER_STATUETTE
};

/// `Server.Items.SeahorseStatuette`.
pub static SEAHORSE_STATUETTE: ItemType = ItemType {
    full_name: "Server.Items.SeahorseStatuette",
    parent: Some(&crate::special::statuette::MONSTER_STATUETTE),
    construct: Some(|w| {
        const HUES: [i32; 5] = [0, 0x482, 0x489, 0x495, 0x4F2];
        let id = crate::special::statuette::monster_statuette_new_ty(w, &SEAHORSE_STATUETTE, 24);
        ops::set_loot_type(w, id, LootType::Regular);
        let i = random(w, HUES.len() as i32) as usize;
        hue(w, id, HUES[i]);
        id
    }),
    serialize: |w, id, out| {
        (crate::special::statuette::MONSTER_STATUETTE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (crate::special::statuette::MONSTER_STATUETTE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..crate::special::statuette::MONSTER_STATUETTE
};

/// An instrument with `Slayer`, `Slayer2` and a fixed number of uses (`InitMinUses = InitMaxUses`).
macro_rules! slayer_instrument {
    ($(#[$m:meta])* $name:ident, $new:ident = $full:literal : $parent:ident, gid $gid:expr, $well:expr, $badly:expr,
     hue $hue:expr, slayers ($s1:expr, $s2:expr), label $label:expr, uses $uses:expr;) => {
        /// The constructor for the dynamic class `ty`.
        pub fn $new(w: &mut World, ty: &'static ItemType) -> ItemId {
            let id = crate::skill_items::instruments::new_instrument_uses(w, ty, $gid, $well, $badly, ($uses, $uses));
            hue(w, id, $hue);
            if let Some(d) = crate::skill_items::instruments::data_mut(w, id) {
                d.slayer = $s1 as i32;
                d.slayer2 = $s2 as i32;
            }
            id
        }

        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&crate::skill_items::instruments::$parent),
            construct: Some(|w| $new(w, &$name)),
            label_number: |_, _| $label,
            serialize: |w, id, out| {
                (crate::skill_items::instruments::$parent.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (crate::skill_items::instruments::$parent.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..crate::skill_items::instruments::$parent
        };
    };
}

slayer_instrument! {
    /// `Server.Items.GwennosHarp`.
    GWENNOS_HARP, gwennos_harp_new = "Server.Items.GwennosHarp" : LAP_HARP, gid 0xEB2, 0x45, 0x46,
    hue 0x47E, slayers (SlayerName::Repond, SlayerName::ReptilianDeath), label 1063480, uses 1600;
}

slayer_instrument! {
    /// `Server.Items.IolosLute`.
    IOLOS_LUTE, iolos_lute_new = "Server.Items.IolosLute" : LUTE, gid 0xEB3, 0x4C, 0x4D,
    hue 0x47E, slayers (SlayerName::Silver, SlayerName::Exorcism), label 1063479, uses 1600;
}

use crate::talismans::{self, TalismanClass, removal};

/// `BloodwoodSpirit`' class table.
pub static BLOODWOOD_SPIRIT_CLASS: TalismanClass = TalismanClass {
    force_show_name: true,
    ..talismans::BASE_TALISMAN_CLASS
};

/// `BloodwoodSpirit()`.
pub fn bloodwood_spirit_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = talismans::new_talisman(w, ty, 0x2F5A);
    hue(w, id, 0x27);
    talismans::set_max_charge_time(w, id, 1200);
    talismans::set_removal(w, id, removal::DAMAGE);
    let blessed = talismans::random_blessed(w);
    talismans::set_blessed(w, id, blessed);
    let p = talismans::random_protection(w, false);
    talismans::set_protection(w, id, p);
    skill_bonus(w, id, 0, SkillName::SpiritSpeak, 10.0);
    skill_bonus(w, id, 1, SkillName::Necromancy, 5.0);
    id
}

/// `Server.Items.BloodwoodSpirit`.
pub static BLOODWOOD_SPIRIT: ItemType = ItemType {
    full_name: "Server.Items.BloodwoodSpirit",
    parent: Some(&talismans::BASE_TALISMAN),
    construct: Some(|w| bloodwood_spirit_new(w, &BLOODWOOD_SPIRIT)),
    family: Some(&BLOODWOOD_SPIRIT_CLASS),
    label_number: |_, _| 1075034,
    serialize: |w, id, out| {
        (talismans::BASE_TALISMAN.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (talismans::BASE_TALISMAN.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..talismans::BASE_TALISMAN
};

/// `TotemOfVoid`' class table: `GetSummoner() => Utility.RandomBool() ? SummonedSkeletalKnight : SummonedSheep`.
pub static TOTEM_OF_VOID_CLASS: TalismanClass = TalismanClass {
    force_show_name: true,
    get_summoner: |w, _| {
        Some(if random_bool(w) {
            "Server.Mobiles.SummonedSkeletalKnight"
        } else {
            "Server.Mobiles.SummonedSheep"
        })
    },
};

/// `TotemOfVoid()`.
pub fn totem_of_void_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = talismans::new_talisman(w, ty, 0x2F5B);
    hue(w, id, 0x2D0);
    talismans::set_max_charge_time(w, id, 1800);
    let blessed = talismans::random_blessed(w);
    talismans::set_blessed(w, id, blessed);
    let p = talismans::random_protection(w, false);
    talismans::set_protection(w, id, p);
    set_attr(
        w,
        id,
        AttrKind::Attributes,
        AosAttribute::RegenHits as u32,
        2,
    );
    set_attr(
        w,
        id,
        AttrKind::Attributes,
        AosAttribute::LowerManaCost as u32,
        10,
    );
    id
}

/// `Server.Items.TotemOfVoid`.
pub static TOTEM_OF_VOID: ItemType = ItemType {
    full_name: "Server.Items.TotemOfVoid",
    parent: Some(&talismans::BASE_TALISMAN),
    construct: Some(|w| totem_of_void_new(w, &TOTEM_OF_VOID)),
    family: Some(&TOTEM_OF_VOID_CLASS),
    label_number: |_, _| 1075035,
    serialize: |w, id, out| {
        (talismans::BASE_TALISMAN.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (talismans::BASE_TALISMAN.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..talismans::BASE_TALISMAN
};

use crate::quivers::{self, QuiverClass};

/// `QuiverOfElements`: all chaos damage.
pub static QUIVER_OF_ELEMENTS_CLASS: QuiverClass = QuiverClass {
    alter_bow_damage: |s| *s = [0, 0, 0, 0, 0, 100, 0],
};

/// `QuiverOfRage`: 20 of each element, no chaos or direct.
pub static QUIVER_OF_RAGE_CLASS: QuiverClass = QuiverClass {
    alter_bow_damage: |s| *s = [20, 20, 20, 20, 20, 0, 0],
};

/// `QuiverOfElements()`: `BaseQuiver()` (graphic 0x2FB7), `Hue = 0xEB`, `WeightReduction = 50`.
pub fn quiver_of_elements_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = quivers::new_quiver(w, ty, 0x2FB7);
    hue(w, id, 0xEB);
    quivers::set_weight_reduction(w, id, 50);
    id
}

/// `QuiverOfRage()`.
pub fn quiver_of_rage_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = quivers::new_quiver(w, ty, 0x2FB7);
    hue(w, id, 0x24C);
    quivers::set_weight_reduction(w, id, 25);
    quivers::set_damage_increase(w, id, 10);
    id
}

macro_rules! quiver_leaf {
    ($name:ident = $full:literal, $new:ident, $class:ident, label $label:expr) => {
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&quivers::BASE_QUIVER),
            construct: Some(|w| $new(w, &$name)),
            family: Some(&$class),
            label_number: |_, _| $label,
            serialize: |w, id, out| {
                (quivers::BASE_QUIVER.serialize)(w, id, out);
                out.write_encoded_int(0);
            },
            deserialize: |w, id, r| {
                (quivers::BASE_QUIVER.deserialize)(w, id, r)?;
                let _version = r.read_encoded_int()?;
                Ok(())
            },
            ..quivers::BASE_QUIVER
        };
    };
}

quiver_leaf!(QUIVER_OF_ELEMENTS = "Server.Items.QuiverOfElements", quiver_of_elements_new, QUIVER_OF_ELEMENTS_CLASS, label 1075040);
quiver_leaf!(QUIVER_OF_RAGE = "Server.Items.QuiverOfRage", quiver_of_rage_new, QUIVER_OF_RAGE_CLASS, label 1075038);

use muo_content::items::tool;

/// `AmeliasToolbox()`: `base(500)` of `TinkerTools`.
pub fn amelias_toolbox_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = tool::new_tool(w, ty, 0x1EB8, 500);
    ops::set_loot_type(w, id, LootType::Blessed);
    hue(w, id, 1895);
    id
}

/// `Server.Items.AmeliasToolbox`.
pub static AMELIAS_TOOLBOX: ItemType = ItemType {
    full_name: "Server.Items.AmeliasToolbox",
    parent: Some(&crate::skill_items::tools::TINKER_TOOLS),
    construct: Some(|w| amelias_toolbox_new(w, &AMELIAS_TOOLBOX)),
    label_number: |_, _| 1077749,
    serialize: |w, id, out| {
        (crate::skill_items::tools::TINKER_TOOLS.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (crate::skill_items::tools::TINKER_TOOLS.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..crate::skill_items::tools::TINKER_TOOLS
};

/// `HammerOfHephaestus`' class table: `BreakOnDepletion => false`.
pub static HAMMER_OF_HEPHAESTUS_CLASS: tool::ToolClass = tool::ToolClass {
    break_on_depletion: false,
    ..crate::skill_items::tools::SMITH_HAMMER_CLASS
};

/// `HammerOfHephaestus()`: `SmithHammer()`, `UsesRemaining = 20`, blessed. The recharge timer is not implemented
/// (`StartRechargeTimer` is empty), so nothing recharges.
pub fn hammer_of_hephaestus_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = tool::new_tool_random(w, ty, 0x13E3);
    (ty.set_layer)(w, id, muo_server::layer::Layer::OneHanded);
    tool::set_uses_remaining(w, id, 20);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

/// `Server.Items.HammerOfHephaestus` (`[Flippable(0x13E3, 0x13E4)]`, in the flip table).
pub static HAMMER_OF_HEPHAESTUS: ItemType = ItemType {
    full_name: "Server.Items.HammerOfHephaestus",
    parent: Some(&crate::skill_items::tools::SMITH_HAMMER),
    construct: Some(|w| hammer_of_hephaestus_new(w, &HAMMER_OF_HEPHAESTUS)),
    family: Some(&HAMMER_OF_HEPHAESTUS_CLASS),
    label_number: |_, _| 1077740,
    on_double_click: |w, id, from| {
        let pack = muo_server::mobile::part1::props_b::backpack(w, from);
        let held = w.item(id).and_then(|i| i.parent) == Some(EntityId::Mobile(from));
        if !ops::is_child_of(w, id, pack.map(EntityId::Item)) && !held {
            deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        } else if tool::uses_remaining(w, id) <= 0 {
            deps::send_localized_message(w, from, 1072306); // You must wait a moment for it to recharge.
        } else {
            (crate::skill_items::tools::SMITH_HAMMER.on_double_click)(w, id, from);
        }
    },
    serialize: |w, id, out| {
        (crate::skill_items::tools::SMITH_HAMMER.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (crate::skill_items::tools::SMITH_HAMMER.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..crate::skill_items::tools::SMITH_HAMMER
};

/// `HallowedSpellbook()`: `base(0x3FFFFFFFF)`, blessed, `Slayer = Silver`.
pub fn hallowed_spellbook_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = crate::skill_items::spellbook::new_spellbook(w, ty, 0x3_FFFF_FFFF, 0xEFA);
    ops::set_loot_type(w, id, LootType::Blessed);
    if let Some(d) = crate::skill_items::spellbook::data_mut(w, id) {
        d.slayer = SlayerName::Silver as i32;
    }
    ops::invalidate_properties(w, id);
    id
}

/// `Server.Items.HallowedSpellbook`.
pub static HALLOWED_SPELLBOOK: ItemType = ItemType {
    full_name: "Server.Items.HallowedSpellbook",
    parent: Some(&crate::skill_items::spellbook::SPELLBOOK),
    construct: Some(|w| hallowed_spellbook_new(w, &HALLOWED_SPELLBOOK)),
    label_number: |_, _| 1077620,
    serialize: |w, id, out| {
        (crate::skill_items::spellbook::SPELLBOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (crate::skill_items::spellbook::SPELLBOOK.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..crate::skill_items::spellbook::SPELLBOOK
};

/// Every class of this file.
pub static ALL: &[&ItemType] = &[
    &BURGLARS_BANDANA,
    &DREAD_PIRATE_HAT,
    &AEGIS_OF_GRACE,
    &FEY_LEGGINGS,
    &PADS_OF_THE_CU_SIDHE,
    &ROBE_OF_THE_ECLIPSE,
    &ROBE_OF_THE_EQUINOX,
    &WILDFIRE_BOW,
    &WINDSONG,
    &SHIELD_OF_INVULNERABILITY,
    &WRATH_OF_THE_DRYAD,
    &CAPTAIN_JOHNS_HAT,
    &DETECTIVE_BOOTS,
    &EMBROIDERED_OAK_LEAF_CLOAK,
    &THE_MOST_KNOWLEDGE_PERSON,
    &THE_ROBE_OF_BRITANNIA_ARI,
    &ORC_CHIEFTAIN_HELM,
    &SHROUD_OF_DECIET,
    &ESCUTCHEON_DE_ARIADNE,
    &JACOBS_PICKAXE,
    &PHILLIPS_WOODEN_STEED,
    &SEAHORSE_STATUETTE,
    &GWENNOS_HARP,
    &IOLOS_LUTE,
    &BLOODWOOD_SPIRIT,
    &TOTEM_OF_VOID,
    &QUIVER_OF_ELEMENTS,
    &QUIVER_OF_RAGE,
    &AMELIAS_TOOLBOX,
    &HAMMER_OF_HEPHAESTUS,
    &HALLOWED_SPELLBOOK,
];

/// `DetectiveBoots.Level`: clamped to 0..=2, kept in `Attributes.BonusInt` as `2 + Level`.
static DETECTIVE_BOOTS_PROPS: &[muo_content::props::PropDef] = &[muo_content::props::PropDef {
    name: "Level",
    kind: muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Numeric(
        muo_content::utilities::types::Numeric::Int,
    )),
    read: muo_server::mobile::AccessLevel::GameMaster,
    write: muo_server::mobile::AccessLevel::GameMaster,
    get: |w, e| match e {
        EntityId::Item(i) => muo_content::props::Value::Int(i64::from(detective_level(w, i))),
        _ => muo_content::props::Value::Null,
    },
    set: Some(|w, e, v| {
        if let EntityId::Item(i) = e {
            let level = v.i32().clamp(0, 2);
            set_attr(
                w,
                i,
                AttrKind::Attributes,
                AosAttribute::BonusInt as u32,
                2 + level,
            );
        }
        Ok(())
    }),
}];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.DetectiveBoots.Properties",
        |w| {
            muo_content::props::add(w, "Server.Items.DetectiveBoots", DETECTIVE_BOOTS_PROPS);
        },
    );
}
