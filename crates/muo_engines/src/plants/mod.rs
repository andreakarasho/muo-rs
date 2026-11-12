//! The plant system: `PlantType`, `PlantCategory`, `PlantTypeInfo`, `PlantHue`, `PlantHueInfo`, `Seed`, and (in `item`, `gumps`, `misc`) `PlantItem`, `PlantSystem`, `PlantBowl`,
//! `PlantResources`, the four gumps, the pour and pollinate targets, the plant reward items and `GiantIceWorm`.
//!
//! A seed used on a bowl asks [`PlantHooks::plant_seed`] (set here), unset = "You must use a seed on a bowl of dirt!".

use muo_content::hooks::rnd;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

pub mod gumps;
pub mod item;
pub mod misc;
pub mod seed;

// ---- PlantType ----

pub mod plant_type {
    pub const CAMPION_FLOWERS: i32 = 0;
    pub const POPPIES: i32 = 1;
    pub const BULRUSHES: i32 = 3;
    pub const PAMPAS_GRASS: i32 = 5;
    pub const ELEPHANT_EAR_PLANT: i32 = 7;
    pub const FERN: i32 = 8;
    pub const PONYTAIL_PALM: i32 = 9;
    pub const CENTURY_PLANT: i32 = 11;
    pub const SNAKE_PLANT: i32 = 13;
    pub const BARREL_CACTUS: i32 = 15;
    pub const TRIBARREL_CACTUS: i32 = 16;
    pub const COMMON_GREEN_BONSAI: i32 = 17;
    pub const COMMON_PINK_BONSAI: i32 = 18;
    pub const UNCOMMON_GREEN_BONSAI: i32 = 19;
    pub const UNCOMMON_PINK_BONSAI: i32 = 20;
    pub const RARE_GREEN_BONSAI: i32 = 21;
    pub const RARE_PINK_BONSAI: i32 = 22;
    pub const EXCEPTIONAL_BONSAI: i32 = 23;
    pub const EXOTIC_BONSAI: i32 = 24;
    pub const CACTUS: i32 = 25;
    pub const FLAX_FLOWERS: i32 = 26;
    pub const FOXGLOVE_FLOWERS: i32 = 27;
    pub const HOPS_EAST: i32 = 28;
    pub const ORFLUER_FLOWERS: i32 = 29;
    pub const CYPRESS_TWISTED: i32 = 30;
    pub const HEDGE_SHORT: i32 = 31;
    pub const JUNIPER_BUSH: i32 = 32;
    pub const SNOWDROP_PATCH: i32 = 33;
    pub const CATTAILS: i32 = 34;
    pub const POPPY_PATCH: i32 = 35;
    pub const SPIDER_TREE: i32 = 36;
    pub const WATER_LILY: i32 = 37;
    pub const CYPRESS_STRAIGHT: i32 = 38;
    pub const HEDGE_TALL: i32 = 39;
    pub const HOPS_SOUTH: i32 = 40;
    pub const SUGAR_CANES: i32 = 41;
    pub const COCOA_TREE: i32 = 42;
}

/// `PlantCategory`: the cliloc numbers (0 is `Default`).
pub mod category {
    pub const DEFAULT: i32 = 0;
    pub const COMMON: i32 = 1063335;
    pub const UNCOMMON: i32 = 1063336;
    pub const RARE: i32 = 1063337;
    pub const EXCEPTIONAL: i32 = 1063341;
    pub const EXOTIC: i32 = 1063342;
    pub const PECULIAR: i32 = 1080528;
    pub const FRAGRANT: i32 = 1080529;
}

/// `PlantTypeInfo`.
pub struct PlantTypeInfo {
    pub item_id: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub contains_plant: bool,
    pub flowery: bool,
    pub crossable: bool,
    pub reproduces: bool,
    pub plant_category: i32,
    /// Cliloc overrides: seed (plant label), plant, full grown, decorative, seed label, plural (`-1` = default).
    pub plant_label_seed: i32,
    pub plant_label_plant: i32,
    pub plant_label_full_grown: i32,
    pub plant_label_decorative: i32,
    pub seed_label: i32,
    pub seed_label_plural: i32,
}

#[allow(clippy::too_many_arguments)] // every argument is part of the call
const fn t(
    item_id: i32,
    ox: i32,
    oy: i32,
    contains: bool,
    flowery: bool,
    crossable: bool,
    reproduces: bool,
    cat: i32,
) -> PlantTypeInfo {
    PlantTypeInfo {
        item_id,
        offset_x: ox,
        offset_y: oy,
        contains_plant: contains,
        flowery,
        crossable,
        reproduces,
        plant_category: cat,
        plant_label_seed: -1,
        plant_label_plant: -1,
        plant_label_full_grown: -1,
        plant_label_decorative: -1,
        seed_label: -1,
        seed_label_plural: -1,
    }
}

use category::*;

/// `PlantTypeInfo.m_Table`.
pub static TABLE: [PlantTypeInfo; 43] = [
    t(0xC83, 0, 0, false, true, true, true, DEFAULT),
    t(0xC86, 0, 0, false, true, true, true, DEFAULT),
    t(0xC88, 0, 10, false, true, true, true, DEFAULT),
    t(0xC94, -15, 0, false, true, true, true, DEFAULT),
    t(0xC8B, 0, 0, false, true, true, true, DEFAULT),
    t(0xCA5, -8, 0, false, true, true, true, DEFAULT),
    t(0xCA7, -10, 0, false, true, true, true, DEFAULT),
    t(0xC97, -20, 0, true, false, true, true, DEFAULT),
    t(0xC9F, -20, 0, false, false, true, true, DEFAULT),
    t(0xCA6, -16, -5, false, false, true, true, DEFAULT),
    t(0xC9C, -5, -10, false, false, true, true, DEFAULT),
    t(0xD31, 0, -27, true, false, true, true, DEFAULT),
    t(0xD04, 0, 10, true, false, true, true, DEFAULT),
    t(0xCA9, 0, 0, true, false, true, true, DEFAULT),
    t(0xD2C, 0, 10, false, false, true, true, DEFAULT),
    t(0xD26, 0, 10, false, false, true, true, DEFAULT),
    t(0xD27, 0, 10, false, false, true, true, DEFAULT),
    t(0x28DC, -5, 5, true, false, false, false, COMMON),
    t(0x28DF, -5, 5, true, false, false, false, COMMON),
    t(0x28DD, -5, 5, true, false, false, false, UNCOMMON),
    t(0x28E0, -5, 5, true, false, false, false, UNCOMMON),
    t(0x28DE, -5, 5, true, false, false, false, RARE),
    t(0x28E1, -5, 5, true, false, false, false, RARE),
    t(0x28E2, -5, 5, true, false, false, false, EXCEPTIONAL),
    t(0x28E3, -5, 5, true, false, false, false, EXOTIC),
    t(0x0D25, 0, 0, false, false, false, false, PECULIAR),
    t(0x1A9A, 5, 10, false, true, false, false, PECULIAR),
    t(0x0C84, 0, 0, false, true, false, false, PECULIAR),
    t(0x1A9F, 5, -25, false, false, false, false, PECULIAR),
    t(0x0CC1, 0, 0, false, true, false, false, PECULIAR),
    t(0x0CFE, -45, -30, false, false, false, false, PECULIAR),
    t(0x0C8F, 0, 0, false, false, false, false, PECULIAR),
    t(0x0CC8, 0, 0, true, false, false, false, PECULIAR),
    t(0x0C8E, -20, 0, false, true, false, false, PECULIAR),
    t(0x0CB7, 0, 0, false, false, false, false, PECULIAR),
    t(0x0CBE, -20, 0, false, true, false, false, PECULIAR),
    t(0x0CC9, 0, 0, false, false, false, false, PECULIAR),
    t(0x0DC1, -5, 15, false, true, false, false, PECULIAR),
    t(0x0CFB, -45, -30, false, false, false, false, PECULIAR),
    t(0x0DB8, 0, -20, false, false, false, false, PECULIAR),
    t(0x1AA1, 10, -25, false, false, false, false, PECULIAR),
    PlantTypeInfo {
        plant_label_seed: 1114898,
        plant_label_plant: 1114898,
        plant_label_full_grown: 1094702,
        plant_label_decorative: 1094703,
        seed_label: 1095221,
        seed_label_plural: 1113715,
        ..t(0x246C, -25, -20, false, false, false, false, PECULIAR)
    },
    PlantTypeInfo {
        plant_label_seed: 1080536,
        plant_label_plant: 1080536,
        plant_label_full_grown: 1080534,
        plant_label_decorative: 1080531,
        seed_label: 1080533,
        seed_label_plural: 1113716,
        ..t(0xC9E, -40, -30, false, false, false, true, FRAGRANT)
    },
];

impl PlantTypeInfo {
    /// `GetInfo(plantType)`: row 0 for an unknown value.
    pub fn get(plant_type: i32) -> &'static PlantTypeInfo {
        usize::try_from(plant_type)
            .ok()
            .and_then(|i| TABLE.get(i))
            .unwrap_or(&TABLE[0])
    }

    /// `Name`.
    pub fn name(&self) -> i32 {
        if self.item_id < 0x4000 {
            1020000 + self.item_id
        } else {
            1078872 + self.item_id
        }
    }

    /// `GetPlantLabelSeed(hueInfo)`.
    pub fn plant_label_seed(&self, bright: bool) -> i32 {
        if self.plant_label_seed != -1 {
            return self.plant_label_seed;
        }
        if bright { 1061887 } else { 1061888 }
    }

    /// `GetPlantLabelPlant(hueInfo)`.
    pub fn plant_label_plant(&self, bright: bool) -> i32 {
        if self.plant_label_plant != -1 {
            return self.plant_label_plant;
        }
        if self.contains_plant {
            return if bright { 1060832 } else { 1060831 };
        }
        if bright { 1061887 } else { 1061888 }
    }

    /// `GetPlantLabelFullGrown(hueInfo)`.
    pub fn plant_label_full_grown(&self, bright: bool) -> i32 {
        if self.plant_label_full_grown != -1 {
            return self.plant_label_full_grown;
        }
        if self.contains_plant {
            return if bright { 1061891 } else { 1061889 };
        }
        if bright { 1061892 } else { 1061890 }
    }

    /// `GetPlantLabelDecorative(hueInfo)`.
    pub fn plant_label_decorative(&self, bright: bool) -> i32 {
        if self.plant_label_decorative != -1 {
            return self.plant_label_decorative;
        }
        if bright { 1074267 } else { 1070973 }
    }

    /// `GetSeedLabel(hueInfo)`.
    pub fn seed_label(&self, bright: bool) -> i32 {
        if self.seed_label != -1 {
            return self.seed_label;
        }
        if bright { 1061918 } else { 1061917 } // [bright] ~1_COLOR~ ~2_TYPE~ seed
    }

    /// `GetSeedLabelPlural(hueInfo)`.
    pub fn seed_label_plural(&self, bright: bool) -> i32 {
        if self.seed_label_plural != -1 {
            return self.seed_label_plural;
        }
        if bright { 1113493 } else { 1113492 } // ~1_amount~ [bright] ~2_color~ ~3_type~ seeds
    }
}

/// `RandomFirstGeneration()`.
pub fn random_first_generation_type(w: &mut World) -> i32 {
    match rnd::below(w, 3) {
        0 => plant_type::CAMPION_FLOWERS,
        1 => plant_type::FERN,
        _ => plant_type::TRIBARREL_CACTUS,
    }
}

/// `RandomPeculiarGroupOne()`..`Four()`.
pub fn random_peculiar(w: &mut World, group: i32) -> i32 {
    use plant_type::*;
    match group {
        1 => [
            CACTUS,
            FLAX_FLOWERS,
            FOXGLOVE_FLOWERS,
            HOPS_EAST,
            COCOA_TREE,
            ORFLUER_FLOWERS,
        ][rnd::below(w, 6) as usize],
        2 => [
            CYPRESS_TWISTED,
            HEDGE_SHORT,
            JUNIPER_BUSH,
            COCOA_TREE,
            SNOWDROP_PATCH,
        ][rnd::below(w, 5) as usize],
        3 => {
            [CATTAILS, POPPY_PATCH, SPIDER_TREE, COCOA_TREE, WATER_LILY][rnd::below(w, 5) as usize]
        }
        _ => [
            CYPRESS_STRAIGHT,
            HEDGE_TALL,
            HOPS_SOUTH,
            COCOA_TREE,
            SUGAR_CANES,
        ][rnd::below(w, 5) as usize],
    }
}

/// `RandomBonsai(increaseRatio)`: each type is `increaseRatio` times as likely as the one before it.
pub fn random_bonsai(w: &mut World, increase_ratio: f64) -> i32 {
    use plant_type::*;
    let k1 = if increase_ratio >= 0.0 {
        increase_ratio
    } else {
        0.0
    };
    let k2 = k1 * k1;
    let k3 = k2 * k1;
    let k4 = k3 * k1;
    let exp1 = k1 + 1.0;
    let exp2 = k2 + exp1;
    let exp3 = k3 + exp2;
    let exp4 = k4 + exp3;
    let rand = rnd::double(w);
    if rand < 0.5 / exp4 {
        COMMON_GREEN_BONSAI
    } else if rand < 1.0 / exp4 {
        COMMON_PINK_BONSAI
    } else if rand < (k1 * 0.5 + 1.0) / exp4 {
        UNCOMMON_GREEN_BONSAI
    } else if rand < exp1 / exp4 {
        UNCOMMON_PINK_BONSAI
    } else if rand < (k2 * 0.5 + exp1) / exp4 {
        RARE_GREEN_BONSAI
    } else if rand < exp2 / exp4 {
        RARE_PINK_BONSAI
    } else if rand < exp3 / exp4 {
        EXCEPTIONAL_BONSAI
    } else {
        EXOTIC_BONSAI
    }
}

/// `IsCrossable(plantType)`.
pub fn is_type_crossable(plant_type: i32) -> bool {
    PlantTypeInfo::get(plant_type).crossable
}

/// `PlantTypeInfo.Cross(first, second)`.
pub fn cross_type(w: &mut World, first: i32, second: i32) -> i32 {
    if !is_type_crossable(first) || !is_type_crossable(second) {
        return plant_type::CAMPION_FLOWERS;
    }
    if first + 1 == second || first == second + 1 {
        return if rnd::bool(w) { first } else { second };
    }
    (first + second) / 2
}

/// `CanReproduce(plantType)`.
pub fn can_type_reproduce(plant_type: i32) -> bool {
    PlantTypeInfo::get(plant_type).reproduces
}

// ---- PlantHue ----

/// `PlantHue` flag values.
pub mod hue {
    pub const NONE: i32 = 0;
    pub const REPRODUCES: i32 = 0x2000000;
    pub const CROSSABLE: i32 = 0x4000000;
    pub const BRIGHT: i32 = 0x8000000;
    pub const PLAIN: i32 = 0x1 | CROSSABLE | REPRODUCES;
    pub const RED: i32 = 0x2 | CROSSABLE | REPRODUCES;
    pub const BLUE: i32 = 0x4 | CROSSABLE | REPRODUCES;
    pub const YELLOW: i32 = 0x8 | CROSSABLE | REPRODUCES;
    pub const BRIGHT_RED: i32 = RED | BRIGHT;
    pub const BRIGHT_BLUE: i32 = BLUE | BRIGHT;
    pub const BRIGHT_YELLOW: i32 = YELLOW | BRIGHT;
    pub const PURPLE: i32 = RED | BLUE;
    pub const GREEN: i32 = BLUE | YELLOW;
    pub const ORANGE: i32 = RED | YELLOW;
    pub const BRIGHT_PURPLE: i32 = PURPLE | BRIGHT;
    pub const BRIGHT_GREEN: i32 = GREEN | BRIGHT;
    pub const BRIGHT_ORANGE: i32 = ORANGE | BRIGHT;
    pub const BLACK: i32 = 0x10;
    pub const WHITE: i32 = 0x20;
    pub const PINK: i32 = 0x40;
    pub const MAGENTA: i32 = 0x80;
    pub const AQUA: i32 = 0x100;
    pub const FIRE_RED: i32 = 0x200;
}

/// `PlantHueInfo`: hue, name cliloc and gump hue.
#[derive(Copy, Clone, Debug)]
pub struct PlantHueInfo {
    pub hue: i32,
    pub name: i32,
    pub plant_hue: i32,
    pub gump_hue: i32,
}

const fn h(hue: i32, name: i32, plant_hue: i32, gump_hue: i32) -> PlantHueInfo {
    PlantHueInfo {
        hue,
        name,
        plant_hue,
        gump_hue,
    }
}

impl PlantHueInfo {
    /// `GetInfo(plantHue)`: `Plain`'s for an unknown value.
    pub fn get(plant_hue: i32) -> PlantHueInfo {
        use hue::*;
        match plant_hue {
            PLAIN => h(0, 1060813, PLAIN, 0x835),
            RED => h(0x66D, 1060814, RED, 0x24),
            BLUE => h(0x53D, 1060815, BLUE, 0x6),
            YELLOW => h(0x8A5, 1060818, YELLOW, 0x38),
            BRIGHT_RED => h(0x21, 1060814, BRIGHT_RED, 0x21),
            BRIGHT_BLUE => h(0x5, 1060815, BRIGHT_BLUE, 0x6),
            BRIGHT_YELLOW => h(0x38, 1060818, BRIGHT_YELLOW, 0x35),
            PURPLE => h(0xD, 1060816, PURPLE, 0x10),
            GREEN => h(0x59B, 1060819, GREEN, 0x42),
            ORANGE => h(0x46F, 1060817, ORANGE, 0x2E),
            BRIGHT_PURPLE => h(0x10, 1060816, BRIGHT_PURPLE, 0xD),
            BRIGHT_GREEN => h(0x42, 1060819, BRIGHT_GREEN, 0x3F),
            BRIGHT_ORANGE => h(0x2B, 1060817, BRIGHT_ORANGE, 0x2B),
            BLACK => h(0x455, 1060820, BLACK, 0),
            WHITE => h(0x481, 1060821, WHITE, 0x481),
            PINK => h(0x48E, 1061854, PINK, 0x48E),
            MAGENTA => h(0x486, 1061852, MAGENTA, 0x486),
            AQUA => h(0x495, 1061853, AQUA, 0x495),
            FIRE_RED => h(0x489, 1061855, FIRE_RED, 0x489),
            _ => h(0, 1060813, PLAIN, 0x835),
        }
    }

    pub fn is_bright(&self) -> bool {
        is_bright(self.plant_hue)
    }

    pub fn is_crossable(&self) -> bool {
        is_hue_crossable(self.plant_hue)
    }

    pub fn is_primary(&self) -> bool {
        is_primary(self.plant_hue)
    }
}

/// `RandomFirstGeneration()`.
pub fn random_first_generation_hue(w: &mut World) -> i32 {
    match rnd::below(w, 4) {
        0 => hue::PLAIN,
        1 => hue::RED,
        2 => hue::BLUE,
        _ => hue::YELLOW,
    }
}

pub fn can_hue_reproduce(h: i32) -> bool {
    h & hue::REPRODUCES != 0
}

pub fn is_hue_crossable(h: i32) -> bool {
    h & hue::CROSSABLE != 0
}

pub fn is_bright(h: i32) -> bool {
    h & hue::BRIGHT != 0
}

pub fn get_not_bright(h: i32) -> i32 {
    h & !hue::BRIGHT
}

pub fn is_primary(h: i32) -> bool {
    matches!(h, hue::RED | hue::BLUE | hue::YELLOW)
}

/// `PlantHueInfo.Cross(first, second)`.
pub fn cross_hue(w: &mut World, first: i32, second: i32) -> i32 {
    if !is_hue_crossable(first) || !is_hue_crossable(second) {
        return hue::NONE;
    }
    if rnd::double(w) < 0.01 {
        return if rnd::bool(w) { hue::BLACK } else { hue::WHITE };
    }
    if first == hue::PLAIN || second == hue::PLAIN {
        return hue::PLAIN;
    }
    let (nb_first, nb_second) = (get_not_bright(first), get_not_bright(second));
    if nb_first == nb_second {
        return first | hue::BRIGHT;
    }
    let (fp, sp) = (is_primary(nb_first), is_primary(nb_second));
    if fp && sp {
        return nb_first | nb_second;
    }
    if fp {
        return nb_first;
    }
    if sp {
        return nb_second;
    }
    nb_first & nb_second
}

// ---- hooks ----

type PlantSeedFn = fn(&mut World, MobileId, ItemId, ItemId) -> bool;
/// Calls into classes another crate owns.
#[derive(Default)]
pub struct PlantHooks {
    /// `plant.PlantSeed(from, seed)` for a targeted `PlantItem`; `true` when the target was one.
    /// Unset: a seed on a bowl of dirt is "You must use a seed on a bowl of dirt!".
    pub plant_seed: Option<PlantSeedFn>,
}

fn configure(w: &mut World) {
    // `new Seed(plantType, plantHue)` for the quests (Naturalist) that hand one out.
    let q = w.resources.get_mut::<crate::quests::hooks::QuestHooks>();
    q.make_seed = Some(|w, plant_type, plant_hue| seed::seed_new(w, plant_type, plant_hue, false));
}

fn configure_hooks(w: &mut World) {
    w.resources.get_mut::<PlantHooks>().plant_seed = Some(|w, from, target, seed| {
        if !item::is_plant(w, target) {
            return false;
        }
        item::plant_seed(w, from, target, seed);
        true
    });
    // `targ is PlantItem item` then `item.Pour(from, bev)` (BaseBeverage.Pour_OnTarget).
    w.resources
        .get_mut::<muo_items::food::beverage::BeverageHooks>()
        .plant_pour = Some(|w, target, from, bev| {
        if !item::is_plant(w, target) {
            return false;
        }
        item::pour(w, target, from, bev);
        true
    });
    // `PlantItem` is an `ISecurable`.
    let d = muo_content::items::house_deps::deps_mut(w);
    d.securables.push(|w, id| {
        item::is_plant(w, id).then(|| item::data(w, id).map_or(0, |d| d.level) as u8)
    });
    d.securable_setters.push(|w, id, level| {
        if !item::is_plant(w, id) {
            return false;
        }
        item::set_level(w, id, i32::from(level));
        true
    });
    // `OrangePetals.UnderEffect(m)` for the poison timer.
    w.resources
        .get_mut::<muo_content::misc::poison_kinds::PoisonHooks>()
        .orange_petals = Some(misc::orange_petals_under_effect);
    // `EventSink.WorldLoad`, and `WorldSave` unless the server restarts by itself (`AutoRestart`).
    w.events.world_load.subscribe(|w, _| item::grow_all(w));
    w.events.world_save.subscribe(|w, _| {
        if !muo_content::login::small::auto_restart_enabled(w) {
            item::grow_all(w);
        }
    });
    muo_content::network::events::subscribe_ranked::<muo_content::network::events::PlayerLogin>(
        w,
        8,
        |w, e| {
            item::on_login(w, e.0);
        },
    );
}

pub fn register(types: &mut TypeRegistry) {
    seed::register(types);
    item::register(types);
    misc::register(types);
    types.register_phase(
        Phase::Configure,
        "PlantSystem.Configure",
        DEFAULT_CALL_PRIORITY,
        configure,
    );
    types.register_phase(
        Phase::Configure,
        "PlantSystem.Hooks",
        DEFAULT_CALL_PRIORITY,
        configure_hooks,
    );
}
