//! Craft resources, the part items need: `CraftResource`, `CraftResources.GetHue` and the
//! `CraftAttributeInfo` numbers durability and resistances read, plus the type registry (`from_type`,
//! `resource_types`), `localization_number`/`name` and the runic ranges.

use muo_server::World;

use crate::core::aos as core_aos;

/// `CraftResource`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum CraftResource {
    #[default]
    None = 0,
    Iron = 1,
    DullCopper = 2,
    ShadowIron = 3,
    Copper = 4,
    Bronze = 5,
    Gold = 6,
    Agapite = 7,
    Verite = 8,
    Valorite = 9,
    RegularLeather = 101,
    SpinedLeather = 102,
    HornedLeather = 103,
    BarbedLeather = 104,
    RedScales = 201,
    YellowScales = 202,
    BlackScales = 203,
    GreenScales = 204,
    WhiteScales = 205,
    BlueScales = 206,
    RegularWood = 301,
    OakWood = 302,
    AshWood = 303,
    YewWood = 304,
    Heartwood = 305,
    Bloodwood = 306,
    Frostwood = 307,
}

impl CraftResource {
    /// The resource for a raw value (a number without a name reads as an error).
    pub fn from_i32(v: i32) -> Option<CraftResource> {
        use CraftResource::*;
        Some(match v {
            0 => None,
            1 => Iron,
            2 => DullCopper,
            3 => ShadowIron,
            4 => Copper,
            5 => Bronze,
            6 => Gold,
            7 => Agapite,
            8 => Verite,
            9 => Valorite,
            101 => RegularLeather,
            102 => SpinedLeather,
            103 => HornedLeather,
            104 => BarbedLeather,
            201 => RedScales,
            202 => YellowScales,
            203 => BlackScales,
            204 => GreenScales,
            205 => WhiteScales,
            206 => BlueScales,
            301 => RegularWood,
            302 => OakWood,
            303 => AshWood,
            304 => YewWood,
            305 => Heartwood,
            306 => Bloodwood,
            307 => Frostwood,
            _ => return Option::None,
        })
    }
}

/// `CraftAttributeInfo` (the members items read).
#[derive(Copy, Clone, Debug, Default)]
pub struct CraftAttr {
    pub weapon_fire_damage: i32,
    pub weapon_cold_damage: i32,
    pub weapon_poison_damage: i32,
    pub weapon_energy_damage: i32,
    pub weapon_chaos_damage: i32,
    pub weapon_direct_damage: i32,
    pub weapon_luck: i32,
    pub weapon_durability: i32,
    pub weapon_lower_requirements: i32,
    pub armor_luck: i32,
    pub armor_physical_resist: i32,
    pub armor_fire_resist: i32,
    pub armor_cold_resist: i32,
    pub armor_poison_resist: i32,
    pub armor_energy_resist: i32,
    pub armor_durability: i32,
    pub armor_lower_requirements: i32,
}

const fn armor(p: i32, f: i32, c: i32, po: i32, e: i32) -> CraftAttr {
    CraftAttr {
        armor_physical_resist: p,
        armor_fire_resist: f,
        armor_cold_resist: c,
        armor_poison_resist: po,
        armor_energy_resist: e,
        weapon_fire_damage: 0,
        weapon_cold_damage: 0,
        weapon_poison_damage: 0,
        weapon_energy_damage: 0,
        weapon_chaos_damage: 0,
        weapon_direct_damage: 0,
        weapon_luck: 0,
        weapon_durability: 0,
        weapon_lower_requirements: 0,
        armor_luck: 0,
        armor_durability: 0,
        armor_lower_requirements: 0,
    }
}

/// `CraftAttributeInfo.Blank`.
const BLANK: CraftAttr = armor(0, 0, 0, 0, 0);

/// `CraftAttributeInfo.RunicMinAttributes/RunicMaxAttributes/RunicMinIntensity/RunicMaxIntensity` of a resource:
/// `(min attributes, max attributes, min intensity, max intensity)`. The intensities are the `Core.ML` ones when
/// `ml` is set. Zero for the resources with `CraftAttributeInfo.Blank`; `None` for a resource without info.
pub fn runic_info(r: CraftResource, ml: bool) -> Option<(i32, i32, i32, i32)> {
    use CraftResource::*;
    // (min attributes, max attributes, ML min, ML max, pre-ML min, pre-ML max)
    let (a, b, ml_min, ml_max, min, max) = match r {
        None => return Option::None,
        DullCopper => (1, 2, 40, 100, 10, 35),
        ShadowIron => (2, 2, 45, 100, 20, 45),
        Copper => (2, 3, 50, 100, 25, 50),
        Bronze => (3, 3, 55, 100, 30, 65),
        Gold => (3, 4, 60, 100, 35, 75),
        Agapite => (4, 4, 65, 100, 40, 80),
        Verite => (4, 5, 70, 100, 45, 90),
        Valorite => (5, 5, 85, 100, 50, 100),
        SpinedLeather => (1, 3, 40, 100, 20, 40),
        HornedLeather => (3, 4, 45, 100, 30, 70),
        BarbedLeather => (4, 5, 50, 100, 40, 100),
        _ => (0, 0, 0, 0, 0, 0),
    };
    Some(if ml {
        (a, b, ml_min, ml_max)
    } else {
        (a, b, min, max)
    })
}

/// `CraftResourceInfo.AttributeInfo` of `r`; `None` for a resource without info.
pub fn attribute_info(r: CraftResource) -> Option<CraftAttr> {
    use CraftResource::*;
    Some(match r {
        Iron | RegularLeather | RegularWood | OakWood | AshWood | YewWood | Heartwood
        | Bloodwood | Frostwood => BLANK,
        DullCopper => CraftAttr {
            armor_durability: 50,
            armor_lower_requirements: 20,
            weapon_durability: 100,
            weapon_lower_requirements: 50,
            ..armor(6, 0, 0, 0, 0)
        },
        ShadowIron => CraftAttr {
            armor_durability: 100,
            weapon_cold_damage: 20,
            weapon_durability: 50,
            ..armor(2, 1, 0, 0, 5)
        },
        Copper => CraftAttr {
            weapon_poison_damage: 10,
            weapon_energy_damage: 20,
            ..armor(1, 1, 0, 5, 2)
        },
        Bronze => CraftAttr {
            weapon_fire_damage: 40,
            ..armor(3, 0, 5, 1, 1)
        },
        Gold => CraftAttr {
            armor_luck: 40,
            weapon_luck: 40,
            armor_lower_requirements: 30,
            weapon_lower_requirements: 50,
            ..armor(1, 1, 2, 0, 2)
        },
        Agapite => CraftAttr {
            weapon_cold_damage: 30,
            weapon_energy_damage: 20,
            ..armor(2, 3, 2, 2, 2)
        },
        Verite => CraftAttr {
            weapon_poison_damage: 40,
            weapon_energy_damage: 20,
            ..armor(3, 3, 2, 3, 1)
        },
        Valorite => CraftAttr {
            weapon_fire_damage: 10,
            weapon_cold_damage: 20,
            weapon_poison_damage: 10,
            weapon_energy_damage: 20,
            armor_durability: 50,
            ..armor(4, 0, 3, 3, 3)
        },
        SpinedLeather => CraftAttr {
            armor_luck: 40,
            ..armor(5, 0, 0, 0, 0)
        },
        HornedLeather => armor(2, 3, 2, 2, 2),
        BarbedLeather => armor(2, 1, 2, 3, 4),
        RedScales => armor(0, 10, -3, 0, 0),
        YellowScales => CraftAttr {
            armor_luck: 20,
            ..armor(-3, 0, 0, 0, 0)
        },
        BlackScales => armor(10, 0, 0, 0, -3),
        GreenScales => armor(0, -3, 0, 10, 0),
        WhiteScales => armor(-3, 0, 10, 0, 0),
        BlueScales => armor(0, 0, 0, -3, 10),
        None => return Option::None,
    })
}

/// `CraftResources.GetHue`: 0 for a resource without info.
pub fn hue(w: &World, r: CraftResource) -> i32 {
    use CraftResource::*;
    match r {
        Iron | RegularLeather | RegularWood => 0,
        DullCopper => 0x973,
        ShadowIron => 0x966,
        Copper => 0x96D,
        Bronze => 0x972,
        Gold => 0x8A5,
        Agapite => 0x979,
        Verite => 0x89F,
        Valorite => 0x8AB,
        SpinedLeather => {
            if core_aos(w) {
                0x8AC
            } else {
                0x283
            }
        }
        HornedLeather => {
            if core_aos(w) {
                0x845
            } else {
                0x227
            }
        }
        BarbedLeather => {
            if core_aos(w) {
                0x851
            } else {
                0x1C1
            }
        }
        RedScales => 0x66D,
        YellowScales => 0x8A8,
        BlackScales => 0x455,
        GreenScales => 0x851,
        WhiteScales => 0x8FD,
        BlueScales => 0x8B0,
        OakWood => 0x7DA,
        AshWood => 0x4A7,
        YewWood => 0x4A8,
        Heartwood => 0x4A9,
        Bloodwood => 0x4AA,
        Frostwood => 0x47F,
        None => 0,
    }
}

/// `CraftResources.IsStandard`: None, Iron, RegularLeather or RegularWood.
pub fn is_standard(r: CraftResource) -> bool {
    matches!(
        r,
        CraftResource::None
            | CraftResource::Iron
            | CraftResource::RegularLeather
            | CraftResource::RegularWood
    )
}

/// `CraftResources.GetInfo(r)?.Number` and `.Name`: `None` for a resource without info.
fn number_and_name(r: CraftResource) -> Option<(i32, &'static str)> {
    use CraftResource::*;
    Some(match r {
        None => return Option::None,
        Iron => (1053109, "Iron"),
        DullCopper => (1053108, "Dull Copper"),
        ShadowIron => (1053107, "Shadow Iron"),
        Copper => (1053106, "Copper"),
        Bronze => (1053105, "Bronze"),
        Gold => (1053104, "Gold"),
        Agapite => (1053103, "Agapite"),
        Verite => (1053102, "Verite"),
        Valorite => (1053101, "Valorite"),
        RegularLeather => (1049353, "Normal"),
        SpinedLeather => (1049354, "Spined"),
        HornedLeather => (1049355, "Horned"),
        BarbedLeather => (1049356, "Barbed"),
        RedScales => (1053129, "Red Scales"),
        YellowScales => (1053130, "Yellow Scales"),
        BlackScales => (1053131, "Black Scales"),
        GreenScales => (1053132, "Green Scales"),
        WhiteScales => (1053133, "White Scales"),
        BlueScales => (1053134, "Blue Scales"),
        RegularWood => (1011542, "Normal"),
        OakWood => (1072533, "Oak"),
        AshWood => (1072534, "Ash"),
        YewWood => (1072535, "Yew"),
        Heartwood => (1072536, "Heartwood"),
        Bloodwood => (1072538, "Bloodwood"),
        Frostwood => (1072539, "Frostwood"),
    })
}

/// `CraftResources.GetLocalizationNumber`: 0 for a resource without info.
pub fn localization_number(r: CraftResource) -> i32 {
    number_and_name(r).map_or(0, |n| n.0)
}

/// `CraftResources.GetName`: empty for a resource without info.
pub fn name(r: CraftResource) -> &'static str {
    number_and_name(r).map_or("", |n| n.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hue_and_attr() {
        let w = World::new();
        assert_eq!(hue(&w, CraftResource::Valorite), 0x8AB);
        assert_eq!(hue(&w, CraftResource::Iron), 0);
        assert_eq!(
            attribute_info(CraftResource::DullCopper).map(|a| a.weapon_durability),
            Some(100)
        );
        assert!(attribute_info(CraftResource::None).is_none());
        assert_eq!(CraftResource::from_i32(304), Some(CraftResource::YewWood));
    }
}

// ---- the resource-type registry (`CraftResources.RegisterType`/`GetFromType`/`GetInfo(..).ResourceTypes`) ----

/// `CraftResourceType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CraftResourceType {
    None,
    Metal,
    Leather,
    Scales,
    Wood,
}

/// `CraftResources.GetType`.
pub fn resource_type(r: CraftResource) -> CraftResourceType {
    match r as i32 {
        1..=9 => CraftResourceType::Metal,
        101..=104 => CraftResourceType::Leather,
        201..=206 => CraftResourceType::Scales,
        301..=307 => CraftResourceType::Wood,
        _ => CraftResourceType::None,
    }
}

/// `CraftResources.GetInfo(r).ResourceTypes` by full type name (ingot, ore, granite / leather, hides / scales /
/// log, board). Empty for a resource without info. The leather and wood lists are the same under AOS.
pub fn resource_types(r: CraftResource) -> &'static [&'static str] {
    use CraftResource::*;
    match r {
        Iron => &[
            "Server.Items.IronIngot",
            "Server.Items.IronOre",
            "Server.Items.Granite",
        ],
        DullCopper => &[
            "Server.Items.DullCopperIngot",
            "Server.Items.DullCopperOre",
            "Server.Items.DullCopperGranite",
        ],
        ShadowIron => &[
            "Server.Items.ShadowIronIngot",
            "Server.Items.ShadowIronOre",
            "Server.Items.ShadowIronGranite",
        ],
        Copper => &[
            "Server.Items.CopperIngot",
            "Server.Items.CopperOre",
            "Server.Items.CopperGranite",
        ],
        Bronze => &[
            "Server.Items.BronzeIngot",
            "Server.Items.BronzeOre",
            "Server.Items.BronzeGranite",
        ],
        Gold => &[
            "Server.Items.GoldIngot",
            "Server.Items.GoldOre",
            "Server.Items.GoldGranite",
        ],
        Agapite => &[
            "Server.Items.AgapiteIngot",
            "Server.Items.AgapiteOre",
            "Server.Items.AgapiteGranite",
        ],
        Verite => &[
            "Server.Items.VeriteIngot",
            "Server.Items.VeriteOre",
            "Server.Items.VeriteGranite",
        ],
        Valorite => &[
            "Server.Items.ValoriteIngot",
            "Server.Items.ValoriteOre",
            "Server.Items.ValoriteGranite",
        ],
        RegularLeather => &["Server.Items.Leather", "Server.Items.Hides"],
        SpinedLeather => &["Server.Items.SpinedLeather", "Server.Items.SpinedHides"],
        HornedLeather => &["Server.Items.HornedLeather", "Server.Items.HornedHides"],
        BarbedLeather => &["Server.Items.BarbedLeather", "Server.Items.BarbedHides"],
        RedScales => &["Server.Items.RedScales"],
        YellowScales => &["Server.Items.YellowScales"],
        BlackScales => &["Server.Items.BlackScales"],
        GreenScales => &["Server.Items.GreenScales"],
        WhiteScales => &["Server.Items.WhiteScales"],
        BlueScales => &["Server.Items.BlueScales"],
        RegularWood => &["Server.Items.Log", "Server.Items.Board"],
        OakWood => &["Server.Items.OakLog", "Server.Items.OakBoard"],
        AshWood => &["Server.Items.AshLog", "Server.Items.AshBoard"],
        YewWood => &["Server.Items.YewLog", "Server.Items.YewBoard"],
        Heartwood => &["Server.Items.HeartwoodLog", "Server.Items.HeartwoodBoard"],
        Bloodwood => &["Server.Items.BloodwoodLog", "Server.Items.BloodwoodBoard"],
        Frostwood => &["Server.Items.FrostwoodLog", "Server.Items.FrostwoodBoard"],
        None => &[],
    }
}

const ALL_RESOURCES: [CraftResource; 27] = {
    use CraftResource::*;
    [
        Iron,
        DullCopper,
        ShadowIron,
        Copper,
        Bronze,
        Gold,
        Agapite,
        Verite,
        Valorite,
        RegularLeather,
        SpinedLeather,
        HornedLeather,
        BarbedLeather,
        RedScales,
        YellowScales,
        BlackScales,
        GreenScales,
        WhiteScales,
        BlueScales,
        RegularWood,
        OakWood,
        AshWood,
        YewWood,
        Heartwood,
        Bloodwood,
        Frostwood,
        None,
    ]
};

/// `CraftResources.GetFromType(type)`: `None` for a type no resource lists.
pub fn from_type(full_name: &str) -> CraftResource {
    ALL_RESOURCES
        .iter()
        .copied()
        .find(|r| resource_types(*r).contains(&full_name))
        .unwrap_or(CraftResource::None)
}

/// `CraftResources.GetStart(r)`: the first resource of `r`'s series.
pub fn start(r: CraftResource) -> CraftResource {
    match resource_type(r) {
        CraftResourceType::Metal => CraftResource::Iron,
        CraftResourceType::Leather => CraftResource::RegularLeather,
        CraftResourceType::Scales => CraftResource::RedScales,
        CraftResourceType::Wood => CraftResource::RegularWood,
        CraftResourceType::None => CraftResource::None,
    }
}

type CanCraftFn =
    fn(&mut World, &str, muo_server::ids::MobileId, muo_server::ids::ItemId, Option<&str>) -> i32;
type RepairDeedCreateFn =
    fn(&mut World, i32, f64, muo_server::ids::MobileId) -> Option<muo_server::ids::ItemId>;
type CraftableFn =
    fn(&mut World, muo_server::ids::ItemId, &crate::items::equip_hooks::CraftArgs) -> Option<i32>;

/// Calls the items make into the craft engine (muo_engines), set by its `register`. An unset hook is a server
/// without crafting: nothing can be crafted, the tool does nothing.
#[derive(Default)]
pub struct CraftHooks {
    /// `tool.CraftSystem.CanCraft(from, tool, itemType)` (`itemType` a full type name, or `None`): 0 or a cliloc.
    pub can_craft: Option<CanCraftFn>,
    /// `CraftItem.ShowCraftMenu(from, system, tool, null)` / the T2A menu for a double-clicked tool.
    pub show_craft_menu:
        Option<fn(&mut World, &str, muo_server::ids::MobileId, muo_server::ids::ItemId)>,
    /// `CraftSystem.GetContext(from)?.DoNotColor`, by system name.
    pub do_not_color: Option<fn(&World, &str, muo_server::ids::MobileId) -> bool>,
    /// The `ICraftable` classes that live in crates this one does not depend on (instruments, spellbooks,
    /// runebooks, potions, quivers, ninja darts, map items, addon deeds..): each answers `Some(quality)` for an item
    /// it owns. Pushed by the owning crate's `register`.
    /// `RepairDeed.Check(from)` (the contract is in the backpack and a repairing NPC is near). Set by the deed's crate.
    pub repair_deed_check:
        Option<fn(&mut World, muo_server::ids::ItemId, muo_server::ids::MobileId) -> bool>,
    /// `RepairDeed.SkillLevel`.
    pub repair_deed_level: Option<fn(&World, muo_server::ids::ItemId) -> f64>,
    /// `new RepairDeed(skillType, level, crafter)` (`skillType` is the `RepairSkillType` index).
    pub repair_deed_create: Option<RepairDeedCreateFn>,
    /// `Repair.Do(from, system, deed)` for the system the deed names (set by muo_engines; `RepairDeed.OnDoubleClick`).
    pub repair_do_deed:
        Option<fn(&mut World, muo_server::ids::MobileId, muo_server::ids::ItemId, i32)>,
    pub craftables: Vec<CraftableFn>,
}

pub fn hooks(w: &World) -> Option<&CraftHooks> {
    w.resources.get::<CraftHooks>()
}

pub fn hooks_mut(w: &mut World) -> &mut CraftHooks {
    w.resources.get_mut::<CraftHooks>()
}

/// `item is ICraftable` then `item.OnCraft(..)`: the quality it answers. `None` if the item is not craftable
/// (then only the resource hue is copied onto an unhued item).
pub fn on_craft(
    w: &mut World,
    id: muo_server::ids::ItemId,
    a: &crate::items::equip_hooks::CraftArgs,
) -> Option<i32> {
    use crate::items::{armor, clothing, cont, jewel, tool, weapon};
    let ty = w.item(id)?.ty;
    if ty.is(&armor::BASE_ARMOR) {
        return Some(armor::on_craft(w, id, a));
    }
    if ty.is(&weapon::BASE_WEAPON) {
        let f = weapon::class(w, id).on_craft;
        return Some(f(w, id, a));
    }
    if ty.is(&clothing::BASE_CLOTHING) {
        return Some(clothing::on_craft(w, id, a));
    }
    if ty.is(&jewel::BASE_JEWEL) {
        return Some(jewel::on_craft(w, id, a));
    }
    if ty.is(&tool::BASE_TOOL) || ty.is(&tool::BASE_HARVEST_TOOL) {
        return Some(tool::on_craft(w, id, a));
    }
    if ty.is(&cont::LOCKABLE_CONTAINER) {
        return Some(cont::lockable_on_craft(w, id, a));
    }
    let list = hooks(w).map(|h| h.craftables.clone()).unwrap_or_default();
    list.into_iter().find_map(|f| f(w, id, a))
}

/// The classes carrying `[Forge]` (`ForgeAttribute`): ore is smelted on them and a smith works next to them.
pub const FORGE_CLASSES: &[&str] = &[
    "Server.Items.Forge",
    "Server.Items.ForgeComponent",
    "Server.Items.LargeForgeWest",
    "Server.Items.LargeForgeEast",
    "Server.Items.LargeForgeWest+InternalItem",
    "Server.Items.LargeForgeWest+InternalItem2",
    "Server.Items.LargeForgeEast+InternalItem",
    "Server.Items.LargeForgeEast+InternalItem2",
    "Server.Items.ShrineOfWisdomComponent",
    "Server.Mobiles.FireBeetle",
];

/// The classes carrying `[Anvil]` (`AnvilAttribute`).
pub const ANVIL_CLASSES: &[&str] = &[
    "Server.Items.Anvil",
    "Server.Items.AnvilComponent",
    "Server.Items.ColoredAnvil",
];
