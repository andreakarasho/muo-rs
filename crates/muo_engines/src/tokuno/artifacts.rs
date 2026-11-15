//! The Treasures of Tokuno lesser and greater artifacts.
//!
//! `TomeOfEnlightenment`, `TomeOfLostKnowledge` (`Spellbook`) and `FluteOfRenewal` (`BambooFlute`) are at the end.
//! `FluteOfRenewal.InitMaxUses` (300) is `instruments::init_uses_of`.

use muo_content::aos::{AosAttribute, AttrKind, Attrs};
use muo_content::items::weapon_combat::DamageTypes;
use muo_content::items::{same, set_attr};
use muo_items::armor::leaves::*;
use muo_items::clothing::leaves::*;
use muo_items::weapons::*;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::skills::SkillName;
use muo_server::type_registry::TypeRegistry;

use crate::util::{hue, random_element};

fn version_write(out: &mut dyn muo_io::GenericWriter) {
    out.write_i32(0);
}

/// `SkillBonuses.SetValues(index, skill, bonus)`.
fn skill_bonus(w: &mut World, id: ItemId, index: u32, skill: SkillName, bonus: f64) {
    set_attr(
        w,
        id,
        AttrKind::SkillBonuses,
        1 << index,
        Attrs::skill_bonus_value(skill, bonus),
    );
}

fn attr(w: &mut World, id: ItemId, a: AosAttribute, v: i32) {
    set_attr(w, id, AttrKind::Attributes, a as u32, v);
}

/// One concrete class under an armor, clothing or weapon class of muo_items (the same shape as `armor_leaf!`,
/// `clothing_leaf!` and `weapon_art!`, which are private to that crate): `NAME, NAME_CLASS: ClassTable, ctor =
/// "Full.Name" : PARENT, PARENT_CLASS, <parent constructor>;` then `@init` and one `field: value,` per `override`.
macro_rules! art {
    // `@template;`: the rest of the constructor is the class's item template (`Data/Templates/Items`).
    (
        $(#[$m:meta])*
        $name:ident, $class:ident: $cty:ident, $ctor:ident = $full:literal : $parent:path, $pclass:path, $pnew:expr;
        @template;
        $($rest:tt)*
    ) => {
        art! {
            $(#[$m])*
            $name, $class: $cty, $ctor = $full : $parent, $pclass, $pnew;
            @init |w, id| ::muo_content::templates::apply_item_as(w, id, $full);
            $($rest)*
        }
    };
    (
        $(#[$m:meta])*
        $name:ident, $class:ident: $cty:ident, $ctor:ident = $full:literal : $parent:path, $pclass:path, $pnew:expr;
        $(@init $init:expr;)?
        $($f:ident : $v:expr,)*
    ) => {
        /// The `override`s of the class.
        pub static $class: $cty = $cty { $($f: $v,)* ..$pclass };

        /// `new T()` for the dynamic type `ty`.
        pub fn $ctor(w: &mut World, ty: &'static ItemType) -> ItemId {
            let base: fn(&mut World, &'static ItemType) -> ItemId = $pnew;
            let id = base(w, ty);
            $( let init: fn(&mut World, ItemId) = $init; init(w, id); )?
            id
        }

        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| $ctor(w, &$name)),
            family: Some(&$class),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                version_write(out);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..$parent
        };
    };
}

type ArmorClass = muo_content::items::armor::ArmorClass;
type ClothingClass = muo_content::items::clothing::ClothingClass;
type WeaponClass = muo_content::items::weapon::WeaponClass;

fn dmg(phys: i32, fire: i32, cold: i32, pois: i32, nrgy: i32) -> DamageTypes {
    DamageTypes {
        phys,
        fire,
        cold,
        pois,
        nrgy,
        chaos: 0,
        direct: 0,
    }
}

// ---- Lesser artifacts ----

art! {
    /// `Server.Items.AncientFarmersKasa`.
    ANCIENT_FARMERS_KASA, ANCIENT_FARMERS_KASA_CLASS: ClothingClass, ancient_farmers_kasa_new = "Server.Items.AncientFarmersKasa" : KASA, KASA_CLASS, |w, ty| kasa_new(w, ty, 0);
    @template;
    base_cold_resistance: 19,
    init_min_hits: same(255),
    init_max_hits: same(255),
}

art! {
    /// `Server.Items.AncientSamuraiDo`.
    ANCIENT_SAMURAI_DO, ANCIENT_SAMURAI_DO_CLASS: ArmorClass, ancient_samurai_do_new = "Server.Items.AncientSamuraiDo" : PLATE_DO, PLATE_DO_CLASS, plate_do_new;
    @template;
}

art! {
    /// `Server.Items.ArmsOfTacticalExcellence`.
    ARMS_OF_TACTICAL_EXCELLENCE, ARMS_OF_TACTICAL_EXCELLENCE_CLASS: ArmorClass, arms_of_tactical_excellence_new = "Server.Items.ArmsOfTacticalExcellence" : LEATHER_HIRO_SODE, LEATHER_HIRO_SODE_CLASS, leather_hiro_sode_new;
    @template;
}

art! {
    /// `Server.Items.BlackLotusHood`.
    BLACK_LOTUS_HOOD, BLACK_LOTUS_HOOD_CLASS: ClothingClass, black_lotus_hood_new = "Server.Items.BlackLotusHood" : CLOTH_NINJA_HOOD, CLOTH_NINJA_HOOD_CLASS, |w, ty| cloth_ninja_hood_new(w, ty, 0);
    @template;
    base_physical_resistance: 0,
    base_fire_resistance: 11,
    base_cold_resistance: 15,
    base_poison_resistance: 11,
    base_energy_resistance: 11,
    init_min_hits: same(255),
    init_max_hits: same(255),
}

art! {
    /// `Server.Items.DaimyosHelm`.
    DAIMYOS_HELM, DAIMYOS_HELM_CLASS: ArmorClass, daimyos_helm_new = "Server.Items.DaimyosHelm" : PLATE_BATTLE_KABUTO, PLATE_BATTLE_KABUTO_CLASS, plate_battle_kabuto_new;
    @template;
}

art! {
    /// `Server.Items.DemonForks`.
    DEMON_FORKS, DEMON_FORKS_CLASS: WeaponClass, demon_forks_new = "Server.Items.DemonForks" : SAI, SAI_CLASS, sai_new;
    @template;
}

art! {
    /// `Server.Items.DragonNunchaku`.
    DRAGON_NUNCHAKU, DRAGON_NUNCHAKU_CLASS: WeaponClass, dragon_nunchaku_new = "Server.Items.DragonNunchaku" : NUNCHAKU, NUNCHAKU_CLASS, nunchaku_new;
    @template;
}

art! {
    /// `Server.Items.Exiler`.
    EXILER, EXILER_CLASS: WeaponClass, exiler_new = "Server.Items.Exiler" : TETSUBO, TETSUBO_CLASS, tetsubo_new;
    @template;
    get_damage_types: |_, _, _| dmg(0, 0, 0, 0, 100),
}

art! {
    /// `Server.Items.GlovesOfTheSun`.
    GLOVES_OF_THE_SUN, GLOVES_OF_THE_SUN_CLASS: ArmorClass, gloves_of_the_sun_new = "Server.Items.GlovesOfTheSun" : LEATHER_NINJA_MITTS, LEATHER_NINJA_MITTS_CLASS, leather_ninja_mitts_new;
    @template;
}

art! {
    /// `Server.Items.HanzosBow`.
    HANZOS_BOW, HANZOS_BOW_CLASS: WeaponClass, hanzos_bow_new = "Server.Items.HanzosBow" : YUMI, YUMI_CLASS, yumi_new;
    @template;
}

art! {
    /// `Server.Items.LegsOfStability`.
    LEGS_OF_STABILITY, LEGS_OF_STABILITY_CLASS: ArmorClass, legs_of_stability_new = "Server.Items.LegsOfStability" : PLATE_SUNEATE, PLATE_SUNEATE_CLASS, plate_suneate_new;
    @template;
}

art! {
    /// `Server.Items.PeasantsBokuto`.
    PEASANTS_BOKUTO, PEASANTS_BOKUTO_CLASS: WeaponClass, peasants_bokuto_new = "Server.Items.PeasantsBokuto" : BOKUTO, BOKUTO_CLASS, bokuto_new;
    @template;
}

art! {
    /// `Server.Items.PilferedDancerFans`.
    PILFERED_DANCER_FANS, PILFERED_DANCER_FANS_CLASS: WeaponClass, pilfered_dancer_fans_new = "Server.Items.PilferedDancerFans" : TESSEN, TESSEN_CLASS, tessen_new;
    @template;
}

art! {
    /// `Server.Items.TheDestroyer`.
    THE_DESTROYER, THE_DESTROYER_CLASS: WeaponClass, the_destroyer_new = "Server.Items.TheDestroyer" : NO_DACHI, NO_DACHI_CLASS, no_dachi_new;
    @template;
}

art! {
    /// `Server.Items.LeurociansMempoOfFortune`: `LootType = Regular; Hue = 0x501`.
    LEUROCIANS_MEMPO_OF_FORTUNE, LEUROCIANS_MEMPO_OF_FORTUNE_CLASS: ArmorClass, leurocians_mempo_of_fortune_new = "Server.Items.LeurociansMempoOfFortune" : LEATHER_MEMPO, LEATHER_MEMPO_CLASS, leather_mempo_new;
    @template;
}

// ---- Greater artifacts ----

art! {
    /// `Server.Items.DarkenedSky`.
    DARKENED_SKY, DARKENED_SKY_CLASS: WeaponClass, darkened_sky_new = "Server.Items.DarkenedSky" : KAMA, KAMA_CLASS, kama_new;
    @template;
    get_damage_types: |_, _, _| dmg(0, 0, 50, 0, 50),
}

art! {
    /// `Server.Items.KasaOfTheRajin`.
    KASA_OF_THE_RAJIN, KASA_OF_THE_RAJIN_CLASS: ClothingClass, kasa_of_the_rajin_new = "Server.Items.KasaOfTheRajin" : KASA, KASA_CLASS, |w, ty| kasa_new(w, ty, 0);
    @template;
    base_physical_resistance: 12,
    base_fire_resistance: 17,
    base_cold_resistance: 21,
    base_poison_resistance: 17,
    base_energy_resistance: 17,
    init_min_hits: same(255),
    init_max_hits: same(255),
}

art! {
    /// `Server.Items.RuneBeetleCarapace`.
    RUNE_BEETLE_CARAPACE, RUNE_BEETLE_CARAPACE_CLASS: ArmorClass, rune_beetle_carapace_new = "Server.Items.RuneBeetleCarapace" : PLATE_DO, PLATE_DO_CLASS, plate_do_new;
    @template;
}

art! {
    /// `Server.Items.Stormgrip`.
    STORMGRIP, STORMGRIP_CLASS: ArmorClass, stormgrip_new = "Server.Items.Stormgrip" : LEATHER_NINJA_MITTS, LEATHER_NINJA_MITTS_CLASS, leather_ninja_mitts_new;
    @template;
}

art! {
    /// `Server.Items.SwordOfTheStampede`.
    SWORD_OF_THE_STAMPEDE, SWORD_OF_THE_STAMPEDE_CLASS: WeaponClass, sword_of_the_stampede_new = "Server.Items.SwordOfTheStampede" : NO_DACHI, NO_DACHI_CLASS, no_dachi_new;
    @template;
    get_damage_types: |_, _, _| dmg(0, 0, 100, 0, 0),
}

art! {
    /// `Server.Items.SwordsOfProsperity`.
    SWORDS_OF_PROSPERITY, SWORDS_OF_PROSPERITY_CLASS: WeaponClass, swords_of_prosperity_new = "Server.Items.SwordsOfProsperity" : DAISHO, DAISHO_CLASS, daisho_new;
    @template;
    get_damage_types: |_, _, _| dmg(0, 100, 0, 0, 0),
}

art! {
    /// `Server.Items.TheHorselord`.
    THE_HORSELORD, THE_HORSELORD_CLASS: WeaponClass, the_horselord_new = "Server.Items.TheHorselord" : YUMI, YUMI_CLASS, yumi_new;
    @template;
}

art! {
    /// `Server.Items.WindsEdge`.
    WINDS_EDGE, WINDS_EDGE_CLASS: WeaponClass, winds_edge_new = "Server.Items.WindsEdge" : TESSEN, TESSEN_CLASS, tessen_new;
    @template;
    get_damage_types: |_, _, _| dmg(0, 0, 0, 0, 100),
}

// ---- AncientUrn, HonorableSwords ----

/// The names `AncientUrn.Names` hands out (also used by `HonorableSwords`).
pub static NAMES: [&str; 35] = [
    "Akira",
    "Avaniaga",
    "Aya",
    "Chie",
    "Emiko",
    "Fumiyo",
    "Gennai",
    "Gennosuke",
    "Genjo",
    "Hamato",
    "Harumi",
    "Ikuyo",
    "Juri",
    "Kaori",
    "Kaoru",
    "Kiyomori",
    "Mayako",
    "Motoki",
    "Musashi",
    "Nami",
    "Nobukazu",
    "Roku",
    "Romi",
    "Ryo",
    "Sanzo",
    "Sakamae",
    "Satoshi",
    "Takamori",
    "Takuro",
    "Teruyo",
    "Toshiro",
    "Yago",
    "Yeijiro",
    "Yoshi",
    "Zeshin",
];

/// `_urnName` / `_swordsName`.
#[derive(Clone, Debug, Default)]
pub struct NameField(pub Option<String>);

/// `AncientUrn(string urnName)`.
pub fn ancient_urn_new(w: &mut World, name: Option<String>) -> ItemId {
    let id = ops::new_item_with_id(w, &ANCIENT_URN, 0x241D);
    if let Some(i) = w.item_mut(id) {
        i.ext = muo_server::ext::Ext::new(NameField(name));
    }
    id
}

/// `AncientUrn()`: `this(Names.RandomElement())`.
pub fn ancient_urn_new_random(w: &mut World) -> ItemId {
    let name = random_element(w, &NAMES).to_string();
    ancient_urn_new(w, Some(name))
}

/// `HonorableSwords(string swordsName)`.
pub fn honorable_swords_new(w: &mut World, name: Option<String>) -> ItemId {
    let id = ops::new_item_with_id(w, &HONORABLE_SWORDS, 0x2853);
    if let Some(i) = w.item_mut(id) {
        i.ext = muo_server::ext::Ext::new(NameField(name));
    }
    id
}

/// `HonorableSwords()`: `this(AncientUrn.Names.RandomElement())`.
pub fn honorable_swords_new_random(w: &mut World) -> ItemId {
    let name = random_element(w, &NAMES).to_string();
    honorable_swords_new(w, Some(name))
}

fn name_of(w: &World, id: ItemId) -> String {
    w.item(id)
        .and_then(|i| i.ext.get::<NameField>())
        .and_then(|n| n.0.clone())
        .unwrap_or_default()
}

fn name_read(w: &mut World, id: ItemId, r: &mut dyn muo_io::GenericReader) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    let name = r.read_string()?;
    if let Some(i) = w.item_mut(id) {
        i.ext = muo_server::ext::Ext::new(NameField(name));
    }
    Ok(())
}

/// `[SerializationGenerator(0, false)] AncientUrn`.
pub static ANCIENT_URN: ItemType = ItemType {
    full_name: "Server.Items.AncientUrn",
    parent: Some(&ITEM_BASE),
    construct: Some(ancient_urn_new_random),
    default_weight: |w, id| {
        crate::template_keys::item_float(w, id, "defaultWeight", ITEM_BASE.default_weight)
    },
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", ITEM_BASE.label_number)
    },
    add_name_property: |w, id, list| list.add_str(1070935, &name_of(w, id)), // Ancient Urn of ~1_name~
    on_single_click: |w, id, from| ops::label_to_number(w, id, from, 1070935, &name_of(w, id)),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_string(name_of(w, id).as_str());
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        name_read(w, id, r)
    },
    ..ITEM_BASE
};

/// `[SerializationGenerator(0, false)] HonorableSwords`.
pub static HONORABLE_SWORDS: ItemType = ItemType {
    full_name: "Server.Items.HonorableSwords",
    parent: Some(&ITEM_BASE),
    construct: Some(honorable_swords_new_random),
    default_weight: |w, id| {
        crate::template_keys::item_float(w, id, "defaultWeight", ITEM_BASE.default_weight)
    },
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", ITEM_BASE.label_number)
    },
    add_name_property: |w, id, list| list.add_str(1070936, &name_of(w, id)), // Honorable Swords of ~1_name~
    on_single_click: |w, id, from| ops::label_to_number(w, id, from, 1070936, &name_of(w, id)),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_string(name_of(w, id).as_str());
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        name_read(w, id, r)
    },
    ..ITEM_BASE
};

// ---- ChestOfHeirlooms ----

/// `ChestOfHeirlooms()`.
pub fn chest_of_heirlooms_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    use muo_content::items::cont;
    let id = cont::new_lockable(w, ty, 0x2811);
    if let Some(d) = cont::ext_mut(w, id) {
        d.locked = true;
        d.lock_level = 95;
        d.max_lock_level = 140;
        d.required_skill = 95;
        d.trap_type = cont::TrapType::ExplosionTrap;
        d.trap_level = 10;
        d.trap_power = 100;
    }
    muo_server::item::container::set_gump_id(w, id, 0x10B);

    for _ in 0..10 {
        let Some(item) = muo_content::loot::random_chest_of_heirlooms_content(w) else {
            continue;
        };
        apply_heirloom_attributes(w, item);
        muo_server::item::container::drop_item_default(w, id, item);
    }
    id
}

/// The `if (item is BaseWeapon) .. else if (item is BaseArmor) .. else if (Core.AOS) hat/jewel` block of the
/// constructor, with `attributeCount = RandomMinMax(1, 5)` drawn first.
fn apply_heirloom_attributes(w: &mut World, item: ItemId) {
    use muo_content::factory::item_is;
    use muo_content::hooks::rnd;
    let count = rnd::min_max(w, 1, 5);
    let (min, max) = (20, 80);
    let aos = w.core_aos();
    let runic = |w: &mut World| {
        if let Some(f) = crate::quests::hooks::quest_hooks(w).and_then(|h| h.apply_runic_attributes)
        {
            f(w, item, count, min, max);
        }
    };
    if item_is(w, item, "Server.Items.BaseWeapon") {
        if aos {
            runic(w);
        } else {
            let dmg = rnd::below(w, 6);
            let acc = rnd::below(w, 6);
            let dur = rnd::below(w, 6);
            if let Some(f) = w
                .resources
                .get::<muo_content::creature::packing::PackHooks>()
                .and_then(|h| h.set_weapon_levels)
            {
                f(w, item, false, dmg, acc, dur);
            }
        }
    } else if item_is(w, item, "Server.Items.BaseArmor") {
        if aos {
            runic(w);
        } else {
            let prot = rnd::below(w, 6);
            let dur = rnd::below(w, 6);
            if let Some(f) = w
                .resources
                .get::<muo_content::creature::packing::PackHooks>()
                .and_then(|h| h.set_armor_levels)
            {
                f(w, item, prot, dur);
            }
        }
    } else if aos
        && (item_is(w, item, "Server.Items.BaseHat") || item_is(w, item, "Server.Items.BaseJewel"))
    {
        runic(w);
    }
}

/// `[Furniture] [Flippable(0x2811, 0x2812)] ChestOfHeirlooms` (the flip data is `flip_table`'s).
pub static CHEST_OF_HEIRLOOMS: ItemType = ItemType {
    full_name: "Server.Items.ChestOfHeirlooms",
    parent: Some(&muo_content::items::cont::LOCKABLE_CONTAINER),
    construct: Some(|w| chest_of_heirlooms_new(w, &CHEST_OF_HEIRLOOMS)),
    label_number: |w, id| {
        crate::template_keys::item_int(
            w,
            id,
            "labelNumber",
            muo_content::items::cont::LOCKABLE_CONTAINER.label_number,
        )
    },
    serialize: |w, id, out| {
        (muo_content::items::cont::LOCKABLE_CONTAINER.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (muo_content::items::cont::LOCKABLE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..muo_content::items::cont::LOCKABLE_CONTAINER
};

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &ANCIENT_FARMERS_KASA,
        &ANCIENT_SAMURAI_DO,
        &ARMS_OF_TACTICAL_EXCELLENCE,
        &BLACK_LOTUS_HOOD,
        &DAIMYOS_HELM,
        &DEMON_FORKS,
        &DRAGON_NUNCHAKU,
        &EXILER,
        &GLOVES_OF_THE_SUN,
        &HANZOS_BOW,
        &LEGS_OF_STABILITY,
        &PEASANTS_BOKUTO,
        &PILFERED_DANCER_FANS,
        &THE_DESTROYER,
        &LEUROCIANS_MEMPO_OF_FORTUNE,
        &DARKENED_SKY,
        &KASA_OF_THE_RAJIN,
        &RUNE_BEETLE_CARAPACE,
        &STORMGRIP,
        &SWORD_OF_THE_STAMPEDE,
        &SWORDS_OF_PROSPERITY,
        &THE_HORSELORD,
        &WINDS_EDGE,
        &ANCIENT_URN,
        &HONORABLE_SWORDS,
        &TOME_OF_ENLIGHTENMENT,
        &TOME_OF_LOST_KNOWLEDGE,
        &FLUTE_OF_RENEWAL,
        &CHEST_OF_HEIRLOOMS,
    ] {
        types.register_item(t, &[]);
    }
}

// ---- Spellbook and BambooFlute artifacts ----

/// `[SerializationGenerator(0, false)]` over a parent: parent part + version int 0.
macro_rules! v0_class {
    ($(#[$m:meta])* $name:ident = $full:literal : $parent:path, construct $ctor:expr; $($f:ident : $v:expr,)*) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some($ctor),
            $($f: $v,)*
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                version_write(out);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..$parent
        };
    };
}

v0_class! {
    /// `Server.Items.TomeOfEnlightenment`.
    TOME_OF_ENLIGHTENMENT = "Server.Items.TomeOfEnlightenment" : muo_items::skill_items::spellbook::SPELLBOOK,
    construct |w| {
        let id = muo_items::skill_items::spellbook::new_spellbook(w, &TOME_OF_ENLIGHTENMENT, 0, 0xEFA);
        ops::set_loot_type(w, id, muo_server::item::LootType::Regular);
        hue(w, id, 0x455);
        attr(w, id, AosAttribute::BonusInt, 5);
        attr(w, id, AosAttribute::SpellDamage, 10);
        attr(w, id, AosAttribute::CastSpeed, 1);
        id
    };
    label_number: |_, _| 1070934,
}

v0_class! {
    /// `Server.Items.TomeOfLostKnowledge`.
    TOME_OF_LOST_KNOWLEDGE = "Server.Items.TomeOfLostKnowledge" : muo_items::skill_items::spellbook::SPELLBOOK,
    construct |w| {
        let id = muo_items::skill_items::spellbook::new_spellbook(w, &TOME_OF_LOST_KNOWLEDGE, 0, 0xEFA);
        ops::set_loot_type(w, id, muo_server::item::LootType::Regular);
        hue(w, id, 0x530);
        skill_bonus(w, id, 0, SkillName::Magery, 15.0);
        attr(w, id, AosAttribute::BonusInt, 8);
        attr(w, id, AosAttribute::LowerManaCost, 15);
        attr(w, id, AosAttribute::SpellDamage, 15);
        id
    };
    label_number: |_, _| 1070971,
}

v0_class! {
    /// `Server.Items.FluteOfRenewal`: a `BambooFlute` of 300 uses with a random super slayer (not Fey).
    FLUTE_OF_RENEWAL = "Server.Items.FluteOfRenewal" : muo_items::skill_items::instruments::BAMBOO_FLUTE,
    construct |w| {
        use muo_items::skill_items::instruments as ins;
        // `InitMinUses`/`InitMaxUses` => 300; the `Slayer` setter is called after the base constructor.
        let id = ins::new_instrument_uses(w, &FLUTE_OF_RENEWAL, 0x2805, 0x504, 0x503, (300, 300));
        // -1 to exclude Fey slayer.
        let slayer = muo_content::items::slayer::random_super_slayer(w);
        if let Some(d) = ins::data_mut(w, id) {
            d.slayer = slayer as i32;
            d.replenishes_charges = true;
        }
        id
    };
    label_number: |_, _| 1070927,
}

/// `UrnName` / `SwordsName` of an urn or a pair of swords.
pub fn name(w: &World, id: ItemId) -> Option<String> {
    w.item(id)?.ext.get::<NameField>()?.0.clone()
}

/// `UrnName = value` / `SwordsName = value`.
pub fn set_name(w: &mut World, id: ItemId, value: Option<String>) {
    if let Some(n) = w.item_mut(id).and_then(|i| i.ext.get_mut::<NameField>()) {
        n.0 = value;
    }
    ops::invalidate_properties(w, id);
}
