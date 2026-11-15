//! `BaseFish`, `AquariumFood`, `VacationWafer`, the aquarium fish and reward fish, and the plain aquarium reward
//! items.
//!
//! Rewards/AquariumMessage, Rewards/CaptainBlackheartsFishingPole and AquariumFishingNet are `aquarium2.rs`; Aquarium,
//! FishBowl and the gump are `aquarium.rs`.

use muo_content::items::clothing::{
    BASE_HAT, BASE_HAT_CLASS, BASE_SHOES, BASE_SHOES_CLASS, new_clothing,
};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use crate::clothing::clothing_leaf;

const DEATH_DELAY: TimeSpan = TimeSpan::from_secs(300);
/// `Dead => ItemID == 0x3B0C`.
const DEAD_ITEM_ID: i32 = 0x3B0C;

/// `BaseFish._timerToken`.
#[derive(Clone, Copy, Debug, Default)]
struct FishData {
    timer: Option<TimerHandle>,
}

fn running(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.ext.get::<FishData>())
        .and_then(|d| d.timer)
        .is_some_and(|h| h.running(w))
}

/// `Dead`.
pub fn is_dead(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.item_id == DEAD_ITEM_ID)
}

/// `StopTimer()`.
pub fn stop_timer(w: &mut World, id: ItemId) {
    if let Some(h) = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<FishData>())
        .and_then(|d| d.timer.take())
    {
        timer::stop(w, h);
    }
    ops::invalidate_properties(w, id);
}

/// `Kill()`.
pub fn kill(w: &mut World, id: ItemId) {
    super::item_id_of(w, id, DEAD_ITEM_ID);
    stop_timer(w, id);
}

/// `StartTimer()`: five minutes to live.
pub fn start_timer(w: &mut World, id: ItemId) {
    stop_timer(w, id);
    let h = timer::start(w, DEATH_DELAY, TimeSpan::ZERO, 1, move |w| {
        if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<FishData>()) {
            d.timer = None;
        }
        kill(w, id);
    });
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<FishData>()) {
        d.timer = Some(h);
    }
    ops::invalidate_properties(w, id);
}

/// `BaseFish(itemID)` for class `ty`.
pub fn base_fish_new(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(FishData::default());
    }
    start_timer(w, id);
    id
}

/// `GetDescription()`.
pub fn description(w: &World, id: ItemId) -> i32 {
    let Some(i) = w.item(id) else { return 0 };
    let dead = is_dead(w, id);
    if i.item_id > 0x3B0F {
        return if dead { 1074424 } else { 1074422 }; // A very unusual [dead/live] aquarium creature
    }
    if i.hue != 0 {
        return if dead { 1074425 } else { 1074423 }; // A [dead/live] aquarium creature of unusual color
    }
    if dead { 1073623 } else { 1073622 } // A [dead/live] aquarium creature
}

/// Server.Items.BaseFish (abstract).
pub static BASE_FISH: ItemType = ItemType {
    full_name: "Server.Items.BaseFish",
    parent: Some(&ITEM_BASE),
    construct: None,
    on_delete: |w, id| {
        (ITEM_BASE.on_delete)(w, id);
        stop_timer(w, id);
    },
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        list.add(description(w, id));
        if !is_dead(w, id) && running(w, id) {
            list.add(1074507); // Gasping for air
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(FishData::default());
        }
        // [AfterDeserialization]: a fish that is not in an Aquarium or a FishBowl starts its clock.
        let in_tank = match w.item(id).and_then(|i| i.parent) {
            Some(EntityId::Item(p)) => w.item(p).is_some_and(|x| {
                muo_content::items::is_named(x.ty, "Server.Items.Aquarium")
                    || muo_content::items::is_named(x.ty, "Server.Items.FishBowl")
            }),
            _ => false,
        };
        if !in_tank {
            start_timer(w, id);
        }
        Ok(())
    },
    ..ITEM_BASE
};

macro_rules! fish_leaves {
    ($($(#[$m:meta])* $name:ident = $full:literal, $label:expr, $gid:expr, $hue:expr;)*) => {$(
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_FISH),
            construct: Some(|w| {
                let gid: i32 = ($gid)(w);
                let id = base_fish_new(w, &$name, gid);
                if $hue != 0 {
                    super::hue(w, id, $hue);
                }
                id
            }),
            label_number: |_, _| $label,
            serialize: |w, id, out| {
                (BASE_FISH.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_FISH.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..BASE_FISH
        };
    )*};
}

include!("fish_rows.inc");

// ---- AquariumFood, VacationWafer and the rewards ----

macro_rules! plain_item {
    ($(#[$m:meta])* $name:ident = $full:literal, $gid:expr, $label:expr, weight $wt:expr, decoration $deco:literal;) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: Some(|w| {
                let gid: i32 = ($gid)(w);
                ops::new_item_with_id(w, &$name, gid)
            }),
            label_number: |_, _| $label,
            default_weight: |_, _| $wt,
            add_name_properties: |w, id, list| {
                (ITEM_BASE.add_name_properties)(w, id, list);
                if $deco {
                    list.add(1073634); // An aquarium decoration
                }
            },
            serialize: |w, id, out| {
                (ITEM_BASE.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (ITEM_BASE.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..ITEM_BASE
        };
    };
}

plain_item!(
    /// `Server.Items.AquariumFood`.
    AQUARIUM_FOOD = "Server.Items.AquariumFood", |_: &mut World| 0xEFC, 1074819, weight 1.0, decoration false;
);
plain_item!(
    /// `Server.Items.FishBones`.
    FISH_BONES = "Server.Items.FishBones", |_: &mut World| 0x3B0C, 1074601, weight 1.0, decoration true;
);
plain_item!(
    /// `Server.Items.IslandStatue`.
    ISLAND_STATUE = "Server.Items.IslandStatue", |_: &mut World| 0x3B0F, 1074600, weight 1.0, decoration true;
);
plain_item!(
    /// `Server.Items.Shell`.
    SHELL = "Server.Items.Shell", |w: &mut World| utility::random_from(&mut w.rng, 0x3B12, 2), 1074598, weight 1.0, decoration true;
);
plain_item!(
    /// `Server.Items.ToyBoat` (`[Flippable(0x14F3, 0x14F4)]`).
    TOY_BOAT = "Server.Items.ToyBoat", |_: &mut World| 0x14F4, 1074363, weight 1.0, decoration true;
);

/// `Server.Items.VacationWafer`.
pub static VACATION_WAFER: ItemType = ItemType {
    full_name: "Server.Items.VacationWafer",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| ops::new_item_with_id(w, &VACATION_WAFER, 0x973)),
    label_number: crate::class_props::label_number, // An aquarium flake sphere
    add_name_properties: |w, id, list| {
        (ITEM_BASE.add_name_properties)(w, id, list);
        list.add_localized_number(1074432, 7); // Vacation days: ~1_DAYS~
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

clothing_leaf! {
    /// `Server.Items.CraftysFishingHat`.
    CRAFTYS_FISHING_HAT, CRAFTYS_FISHING_HAT_CLASS, craftys_fishing_hat_new = "Server.Items.CraftysFishingHat" : BASE_HAT / BASE_HAT_CLASS, |w, ty, hue| new_clothing(w, ty, 0x1713, hue), hue 0, encoded false;
    @ty {
        add_name_properties: |w, id, list| {
            (BASE_HAT.add_name_properties)(w, id, list);
            list.add(1073634); // An aquarium decoration
        },
    };
    base_physical_resistance: 0,
    base_fire_resistance: 5,
    base_cold_resistance: 9,
    base_poison_resistance: 5,
    base_energy_resistance: 5,
    init_min_hits: muo_content::items::same(20),
    init_max_hits: muo_content::items::same(30),
}

clothing_leaf! {
    /// `Server.Items.WaterloggedBoots`.
    WATERLOGGED_BOOTS, WATERLOGGED_BOOTS_CLASS, waterlogged_boots_new = "Server.Items.WaterloggedBoots" : BASE_SHOES / BASE_SHOES_CLASS, |w, ty, hue| new_clothing(w, ty, 0x1711, hue), hue 0, encoded false;
    @init |w, id| {
        // thigh boots or boots
        let thigh = utility::random_bool(&mut w.rng);
        super::item_id_of(w, id, if thigh { 0x1711 } else { 0x170B });
    };
    @ty {
        default_weight: |w, id| if w.item(id).is_some_and(|i| i.item_id == 0x1711) { 4.0 } else { 3.0 },
        add_name_properties: |w, id, list| {
            (BASE_SHOES.add_name_properties)(w, id, list);
            list.add(1073634); // An aquarium decoration
        },
    };
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BASE_FISH, &[]);
    for t in FISH {
        types.register_item(t, &[]);
    }
    for t in [
        &AQUARIUM_FOOD,
        &FISH_BONES,
        &ISLAND_STATUE,
        &SHELL,
        &TOY_BOAT,
        &VACATION_WAFER,
        &CRAFTYS_FISHING_HAT,
        &WATERLOGGED_BOOTS,
    ] {
        types.register_item(t, &[]);
    }
}
