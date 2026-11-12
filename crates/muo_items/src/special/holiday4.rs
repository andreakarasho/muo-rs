//! Holiday items: the Winter 2004 items, the Halloween treats, foods, decorations and masks, the Easter egg
//! and the Valentine items. The 2004 mistletoe and the 2010 fireflies (addons with deeds) are in `holiday5`.

use std::sync::atomic::{AtomicI32, Ordering};

use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::interfaces::register_dyable;
use muo_content::props::{self, Kind, PropDef, Value};
use muo_content::utilities::types::PropType;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::light_type::LightType;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::property_list::PropertyList;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::TypeRegistry;

use super::util::{in_pack, msg};
use crate::lights::LANTERN;
use crate::simple::leaf;

// ---- StaffInfo ----

const STAFF: [&str; 31] = [
    "Prayer",
    "Arch Lich Oro",
    "Tamashii",
    "nibbio",
    "what the moose",
    "Aenima",
    "Alkiser",
    "ASayre",
    "David",
    "Krrios",
    "Mark",
    "Merlin",
    "Merlix",
    "nerun",
    "Outkast",
    "Phantom",
    "Phenos",
    "psz",
    "Quantos",
    "Ryan",
    "Sp1der",
    "V",
    "Zippy",
    "Vorspire",
    "kevin-10",
    "Argalep",
    "dmurphy22",
    "qbradq",
    "Owyn",
    "Luthius",
    "Jaedan",
];

/// `StaffInfo.GetRandomStaff()`.
pub fn random_staff(w: &mut World) -> String {
    utility::random_element(&mut w.rng, &STAFF)
        .copied()
        .unwrap_or_default()
        .to_string()
}

// ---- state: the string fields of the classes (`_staffer`, `_dipper`, `_from`/`_to`, `_owner`/`_line1..3`) ----

/// Up to four `[SerializableField]` strings, by field order of the class.
#[derive(Clone, Debug, Default)]
struct Strs([Option<String>; 4]);

/// `EditLimit` of a St. Valentine bear (not serialized).
#[derive(Clone, Copy, Debug, Default)]
struct EditLimit(DateTime);

fn strs(w: &World, id: ItemId, n: usize) -> Option<String> {
    w.item(id)
        .and_then(|i| i.ext.get::<Strs>())
        .and_then(|s| s.0[n].clone())
}

fn set_strs(w: &mut World, id: ItemId, n: usize, v: Option<String>) {
    if let Some(i) = w.item_mut(id) {
        if let Some(s) = i.ext.get_mut::<Strs>() {
            s.0[n] = v;
        } else {
            let mut s = Strs::default();
            s.0[n] = v;
            i.ext.insert(s);
        }
    }
}

/// Writes the first `n` strings (`writer.Write(string)`).
fn write_strs(w: &World, id: ItemId, out: &mut dyn GenericWriter, n: usize) {
    for k in 0..n {
        out.write_opt_string(strs(w, id, k).as_deref());
    }
}

fn read_strs(w: &mut World, id: ItemId, r: &mut dyn GenericReader, n: usize) -> muo_io::Result<()> {
    for k in 0..n {
        let s = r.read_string()?;
        set_strs(w, id, k, s);
    }
    Ok(())
}

fn blessed(w: &mut World, id: ItemId) {
    ops::set_loot_type(w, id, LootType::Blessed);
}

fn new_item(w: &mut World, ty: &'static ItemType, gid: i32) -> ItemId {
    ops::new_item_with_id(w, ty, gid)
}

// ---- Winter 2004 ----

const WINTER_2004: i32 = 1070880;

fn winter_single_click(w: &mut World, id: ItemId, from: MobileId) {
    (ITEM_BASE.on_single_click)(w, id, from);
    ops::label_to_number(w, id, from, WINTER_2004, "");
}

fn winter_props(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    list.add(WINTER_2004);
}

macro_rules! winter {
    ($($(#[$m:meta])* $name:ident = $full:literal, $gid:expr;)+) => {$(
        leaf! {
            $(#[$m])*
            $name = $full : ITEM_BASE, v0;
            construct: Some(|w| {
                let id = new_item(w, &$name, $gid);
                blessed(w, id);
                id
            }),
            default_weight: |_, _| 1.0,
            on_single_click: winter_single_click,
            get_properties: winter_props,
        }
    )+};
}

winter! {
    /// `Server.Items.DecorativeTopiary`.
    DECORATIVE_TOPIARY = "Server.Items.DecorativeTopiary", 0x2378;
    /// `Server.Items.FestiveCactus`.
    FESTIVE_CACTUS = "Server.Items.FestiveCactus", 0x2376;
    /// `Server.Items.SnowyTree`.
    SNOWY_TREE = "Server.Items.SnowyTree", 0x2377;
}

/// `LightOfTheWinterSolstice(dipper)`.
pub fn light_of_the_winter_solstice_new(w: &mut World, dipper: Option<&str>) -> ItemId {
    let id = new_item(w, &LIGHT_OF_THE_WINTER_SOLSTICE, 0x236E);
    let d = match dipper {
        Some(s) => s.to_string(),
        None => random_staff(w),
    };
    set_strs(w, id, 0, Some(d));
    blessed(w, id);
    ops::set_light(w, id, LightType::Circle300);
    let h = utility::random_dyed_hue(&mut w.rng);
    crate::simple::hue(w, id, h);
    id
}

/// `Server.Items.LightOfTheWinterSolstice` (`[Flippable(0x236E, 0x2371)]` is in the flip table).
pub static LIGHT_OF_THE_WINTER_SOLSTICE: ItemType = ItemType {
    full_name: "Server.Items.LightOfTheWinterSolstice",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| light_of_the_winter_solstice_new(w, None)),
    default_weight: |_, _| 1.0,
    on_single_click: |w, id, from| {
        (ITEM_BASE.on_single_click)(w, id, from);
        ops::label_to_number(w, id, from, 1070881, &strs(w, id, 0).unwrap_or_default()); // Hand Dipped by ~1_name~
        ops::label_to_number(w, id, from, WINTER_2004, "");
    },
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        list.add_str(1070881, &strs(w, id, 0).unwrap_or_default());
        list.add(WINTER_2004);
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        write_strs(w, id, out, 1);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        read_strs(w, id, r, 1)
    },
    ..ITEM_BASE
};

/// `PileOfGlacialSnow.OnDoubleClick` / `SnowPile.OnDoubleClick`.
fn snow_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !in_pack(w, id, from) {
        msg(w, from, 1042010); // You must have the object in your backpack to use it.
    } else if muo_server::mobile::part1::props_b::mounted(w, from) {
        msg(w, from, 1010097); // You cannot use this while mounted.
    } else if muo_server::mobile::part1::combat::can_begin_action(w, from, "Server.Items.SnowPile")
    {
        msg(w, from, 1005575); // You carefully pack the snow into a ball...
        target::begin_target(w, from, 10, false, TargetFlags::NONE, snow_target);
    } else {
        msg(w, from, 1005574); // The snow is not ready to be packed yet.  Keep trying.
    }
}

/// `SnowTarget.OnTarget`.
fn snow_target(w: &mut World, from: MobileId, t: Targeted) {
    if t == Targeted::Entity(EntityId::Mobile(from)) {
        msg(w, from, 1005576); // You can't throw this at yourself.
        return;
    }
    let Targeted::Entity(EntityId::Mobile(targ)) = t else {
        msg(w, from, 1005577); // You can only throw a snowball at something that can throw one back.
        return;
    };
    let pack = muo_server::mobile::part1::props_b::backpack(w, targ);
    if muo_content::player_mobile::hooks::in_safe_zone(w, from)
        || muo_content::player_mobile::hooks::in_safe_zone(w, targ)
    {
        deps::send_message(w, from, "You may not throw snow here.");
        return;
    }
    let has_snow = pack.is_some_and(|p| {
        [&SNOW_PILE, &PILE_OF_GLACIAL_SNOW]
            .iter()
            .any(|ty| muo_server::item::container::find_item_by_type(w, p, ty, true).is_some())
    });
    if !has_snow {
        msg(w, from, 1005577);
        return;
    }
    if !muo_server::mobile::part1::combat::begin_action(w, from, "Server.Items.SnowPile") {
        msg(w, from, 1005574);
        return;
    }
    muo_server::timer::start(
        w,
        TimeSpan::from_secs_f64(5.0),
        TimeSpan::ZERO,
        1,
        move |w| {
            muo_server::mobile::part1::combat::end_action(w, from, "Server.Items.SnowPile");
        },
    );
    muo_content::mx::play_sound(w, from, 0x145);
    muo_content::mx::animate(w, from, 9, 1, 1, true, false, 0);
    msg(w, targ, 1010572); // You have just been hit by a snowball!
    msg(w, from, 1010573); // You throw the snowball and hit the target!
    muo_server::effects::send_moving_effect(w, from, targ, 0x36E4, 7, 0, false, true, 0x47F, 0);
}

leaf! {
    /// `Server.Items.PileOfGlacialSnow`.
    PILE_OF_GLACIAL_SNOW = "Server.Items.PileOfGlacialSnow" : ITEM_BASE, v0;
    construct: Some(|w| {
        let id = new_item(w, &PILE_OF_GLACIAL_SNOW, 0x913);
        crate::simple::hue(w, id, 0x480);
        blessed(w, id);
        id
    }),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1070874, // a Pile of Glacial Snow
    on_single_click: winter_single_click,
    get_properties: winter_props,
    on_double_click: snow_double_click,
}

leaf! {
    /// `Server.Items.SnowPile`.
    SNOW_PILE = "Server.Items.SnowPile" : ITEM_BASE, v0;
    construct: Some(|w| {
        let id = new_item(w, &SNOW_PILE, 0x913);
        crate::simple::hue(w, id, 0x481);
        blessed(w, id);
        id
    }),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1005578, // a pile of snow
    on_double_click: snow_double_click,
}

// ---- Halloween decorations ----

leaf! {
    /// `Server.Items.PumpkinScarecrow`.
    PUMPKIN_SCARECROW = "Server.Items.PumpkinScarecrow" : ITEM_BASE, v0;
    construct: Some(|w| {
        let g = if utility::random_bool(&mut w.rng) { 0x469B } else { 0x469C };
        new_item(w, &PUMPKIN_SCARECROW, g)
    }),
    label_number: |_, _| 1096947,
}

leaf! {
    /// `Server.Items.RuinedTapestry`.
    RUINED_TAPESTRY = "Server.Items.RuinedTapestry" : ITEM_BASE, v0;
    construct: Some(|w| {
        let g = if utility::random_bool(&mut w.rng) { 0x4699 } else { 0x469A };
        new_item(w, &RUINED_TAPESTRY, g)
    }),
    default_name: |_, _| Some("Ruined Tapestry ".to_string()),
}

macro_rules! plain {
    ($($(#[$m:meta])* $name:ident = $full:literal, $gid:expr, $weight:expr;)+) => {$(
        leaf! {
            $(#[$m])*
            $name = $full : ITEM_BASE, v0;
            construct: Some(|w| new_item(w, &$name, $gid)),
            default_weight: |_, _| $weight,
        }
    )+};
}

plain! {
    /// `Server.Items.GrimWarning`.
    GRIM_WARNING = "Server.Items.GrimWarning", 0x42BD, 1.0;
    /// `Server.Items.SkullsOnPike`.
    SKULLS_ON_PIKE = "Server.Items.SkullsOnPike", 0x42B5, 1.0;
    /// `Server.Items.ChairInAGhostCostume`.
    CHAIR_IN_A_GHOST_COSTUME = "Server.Items.ChairInAGhostCostume", 0x3F26, 5.0;
    /// `Server.Items.ExcellentIronMaiden`.
    EXCELLENT_IRON_MAIDEN = "Server.Items.ExcellentIronMaiden", 0x3f15, 5.0;
    /// `Server.Items.HalloweenGuillotine`.
    HALLOWEEN_GUILLOTINE = "Server.Items.HalloweenGuillotine", 0x3F27, 5.0;
}

leaf! {
    /// `Server.Items.ColoredSmallWebs`.
    COLORED_SMALL_WEBS = "Server.Items.ColoredSmallWebs" : ITEM_BASE, v0;
    construct: Some(|w| {
        let g = if utility::random_bool(&mut w.rng) { 0x10d6 } else { 0x10d7 };
        let id = new_item(w, &COLORED_SMALL_WEBS, g);
        let h = if utility::random_bool(&mut w.rng) { 0x455 } else { 0x4E9 };
        crate::simple::hue(w, id, h);
        id
    }),
    default_weight: |_, _| 5.0,
}

/// `HalloweenPumpkin()`.
pub fn halloween_pumpkin_new(w: &mut World) -> ItemId {
    let id = new_item(w, &HALLOWEEN_PUMPKIN, 0);
    let weight = utility::random_min_max(&mut w.rng, 3, 20);
    ops::set_weight(w, id, f64::from(weight));
    let g = if utility::random_double(&mut w.rng) < 0.02 {
        *utility::random_element(&mut w.rng, &[0x4694, 0x4698]).unwrap_or(&0x4694)
    } else {
        *utility::random_element(&mut w.rng, &[0xc6a, 0xc6b, 0xc6c]).unwrap_or(&0xc6a)
    };
    crate::simple::item_id(w, id, g);
    id
}

fn pumpkin_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !deps::mobile_in_range(w, from, ops::get_world_location(w, id), 2) {
        return;
    }
    let cur = w.item(id).map_or(0, |i| i.item_id);
    let douse = matches!(cur, 0x4695 | 0x4691);
    let next = match cur {
        0x4694 => 0x4691,
        0x4691 => 0x4694,
        0x4698 => 0x4695,
        0x4695 => 0x4698,
        c => c,
    };
    crate::simple::item_id(w, id, next);
    msg(w, from, if douse { 1113988 } else { 1113987 }); // You extinguish/light the Jack-O-Lantern
    let (loc, map) = (
        ops::get_world_location(w, id),
        w.item(id).and_then(|i| i.map),
    );
    muo_server::effects::play_sound(w, loc, map, if douse { 0x3be } else { 0x47 });
}

/// `HalloweenPumpkin.OnDragLift`: an unnamed lit/unlit pumpkin may turn into a `PumpkinHead` or get a staffer.
fn pumpkin_drag_lift(w: &mut World, id: ItemId, _from: MobileId) -> bool {
    let (unnamed, cur) = w
        .item(id)
        .map_or((false, 0), |i| (i.name.is_none(), i.item_id));
    if unnamed && matches!(cur, 0x4694 | 0x4691 | 0x4698 | 0x4695) {
        if utility::random_bool(&mut w.rng) {
            if let Some(m) = muo_content::factory::make_mobile(w, "Server.Mobiles.PumpkinHead")
                && let Some(map) = w.item(id).and_then(|i| i.map)
            {
                muo_server::mobile::base::move_to_world(w, m, ops::get_world_location(w, id), map);
            }
            super::util::delete(w, id);
            return false;
        }
        let s = random_staff(w);
        set_strs(w, id, 0, Some(s));
    }
    true
}

/// `Server.Items.HalloweenPumpkin`.
pub static HALLOWEEN_PUMPKIN: ItemType = ItemType {
    full_name: "Server.Items.HalloweenPumpkin",
    parent: Some(&ITEM_BASE),
    construct: Some(halloween_pumpkin_new),
    default_name: |w, id| {
        Some(match strs(w, id, 0) {
            Some(s) => format!("{s}'s Jack-O-Lantern"),
            None => "A Jack-O-Lantern".to_string(),
        })
    },
    on_double_click: pumpkin_double_click,
    on_drag_lift: pumpkin_drag_lift,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        write_strs(w, id, out, 1);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        read_strs(w, id, r, 1)
    },
    ..ITEM_BASE
};

// ---- Masks ----

/// `BasePaintedMask(staffer, itemid)`: the staffer first, then `itemid + Utility.Random(2)`.
fn mask_new(w: &mut World, ty: &'static ItemType, itemid: i32) -> ItemId {
    let staffer = random_staff(w);
    let g = itemid + utility::random(&mut w.rng, 2);
    let id = new_item(w, ty, g);
    set_strs(w, id, 0, Some(staffer));
    id
}

/// `BasePaintedMask.DefaultName` with the class's `MaskName`.
fn mask_name(w: &World, id: ItemId, mask: &str) -> Option<String> {
    Some(match strs(w, id, 0) {
        Some(s) => format!("{mask} hand painted by {s}"),
        None => mask.to_string(),
    })
}

/// `Server.Items.Holiday.BasePaintedMask` (no `[Constructible]`; the old Clown/Daemon/Plague masks load into it).
pub static BASE_PAINTED_MASK: ItemType = ItemType {
    full_name: "Server.Items.Holiday.BasePaintedMask",
    parent: Some(&ITEM_BASE),
    construct: None,
    make: None,
    default_name: |w, id| mask_name(w, id, "A Mask"),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        write_strs(w, id, out, 1);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        read_strs(w, id, r, 1)
    },
    ..ITEM_BASE
};

macro_rules! masks {
    ($($(#[$m:meta])* $name:ident = $full:literal, $gid:expr, $mask:literal;)+) => {$(
        leaf! {
            $(#[$m])*
            $name = $full : BASE_PAINTED_MASK, v0;
            construct: Some(|w| mask_new(w, &$name, $gid)),
            default_name: |w, id| mask_name(w, id, $mask),
        }
    )+};
}

masks! {
    /// `Server.Items.Holiday.PaintedDaemonMask`.
    PAINTED_DAEMON_MASK = "Server.Items.Holiday.PaintedDaemonMask", 0x4a92, "Daemon Mask";
    /// `Server.Items.Holiday.PaintedEvilClownMask`.
    PAINTED_EVIL_CLOWN_MASK = "Server.Items.Holiday.PaintedEvilClownMask", 0x4a90, "Evil Clown Mask";
    /// `Server.Items.Holiday.PaintedPlagueMask`.
    PAINTED_PLAGUE_MASK = "Server.Items.Holiday.PaintedPlagueMask", 0x4A8E, "Plague Mask";
    /// `Server.Items.Holiday.PaintedEvilJesterMask`.
    PAINTED_EVIL_JESTER_MASK = "Server.Items.Holiday.PaintedEvilJesterMask", 0x4BA5, "Evil Jester Mask";
    /// `Server.Items.Holiday.PaintedPorcelainMask`.
    PAINTED_PORCELAIN_MASK = "Server.Items.Holiday.PaintedPorcelainMask", 0x4BA7, "Porcelain Mask";
}

// ---- Winter 2010 ----

macro_rules! winter_2010 {
    ($($(#[$m:meta])* $name:ident = $full:literal, $gid:expr;)+) => {$(
        leaf! {
            $(#[$m])*
            $name = $full : ITEM_BASE, v0;
            // No `[Constructible]` on these two.
            construct: None,
            make: Some(|w| {
                let id = new_item(w, &$name, $gid);
                blessed(w, id);
                id
            }),
            default_weight: |_, _| 30.0,
        }
    )+};
}

winter_2010! {
    /// `Server.Items.Holiday.AngelDecoration` (`[Flippable(0x46FA, 0x46FB)]`).
    ANGEL_DECORATION = "Server.Items.Holiday.AngelDecoration", 0x46FA;
    /// `Server.Items.Holiday.RockingHorse` (`[Flippable(0x4214, 0x4215)]`).
    ROCKING_HORSE = "Server.Items.Holiday.RockingHorse", 0x4214;
}

// ---- Easter 2011 ----

leaf! {
    /// `Server.Items.DragonEasterEgg` (`IDyable`).
    DRAGON_EASTER_EGG = "Server.Items.DragonEasterEgg" : ITEM_BASE, v0;
    construct: Some(|w| new_item(w, &DRAGON_EASTER_EGG, 0x47E6)),
    label_number: |_, _| 1097278,
}

// ---- Halloween treats ----

use super::holiday::CANDY_CANE;

macro_rules! treats {
    ($($(#[$m:meta])* $name:ident = $full:literal, |$w:ident| $gid:expr $(, label $label:expr)?;)+) => {$(
        leaf! {
            $(#[$m])*
            $name = $full : CANDY_CANE, v0;
            construct: Some(|$w| {
                let g = $gid;
                // `CandyCane(itemID)`: not stackable and blessed; the treat then makes itself stackable.
                let id = new_food($w, &$name, g, 1);
                ops::set_stackable($w, id, false);
                ops::set_loot_type($w, id, LootType::Blessed);
                ops::set_stackable($w, id, true);
                id
            }),
            $( label_number: |_, _| $label, )?
        }
    )+};
}

treats! {
    /// `Server.Items.JellyBeans`.
    JELLY_BEANS = "Server.Items.JellyBeans", |_w| 0x468C, label 1096932;
    /// `Server.Items.Lollipops` (`[TypeAlias("Server.Items.Lollipop")]`).
    LOLLIPOPS = "Server.Items.Lollipops", |w| 0x468D + utility::random(&mut w.rng, 3);
    /// `Server.Items.NougatSwirl`.
    NOUGAT_SWIRL = "Server.Items.NougatSwirl", |_w| 0x4690, label 1096936;
    /// `Server.Items.Taffy`.
    TAFFY = "Server.Items.Taffy", |_w| 0x469D, label 1096949;
    /// `Server.Items.WrappedCandy`.
    WRAPPED_CANDY = "Server.Items.WrappedCandy", |_w| 0x469e, label 1096950;
}

// ---- Halloween foods ----

use muo_content::items::food::{CHEESE_PIZZA, FOOD, new_food, set_fill_factor};

leaf! {
    /// `Server.Items.CreepyCake`.
    CREEPY_CAKE = "Server.Items.CreepyCake" : FOOD, v0;
    construct: Some(|w| {
        let id = new_food(w, &CREEPY_CAKE, 0x9e9, 1);
        crate::simple::hue(w, id, 0x3E4);
        id
    }),
    default_name: |_, _| Some("Creepy Cake".to_string()),
}

leaf! {
    /// `Server.Items.MrPlainsCookies`.
    MR_PLAINS_COOKIES = "Server.Items.MrPlainsCookies" : FOOD, v0;
    construct: Some(|w| {
        let id = new_food(w, &MR_PLAINS_COOKIES, 0x160C, 1);
        set_fill_factor(w, id, 4);
        crate::simple::hue(w, id, 0xF4);
        id
    }),
    default_weight: |_, _| 1.0,
    default_name: |_, _| Some("Mr Plain's Cookies".to_string()),
}

leaf! {
    /// `Server.Items.PumpkinPizza` (`CheesePizza()`: 0x1040, fill 6, not stackable).
    PUMPKIN_PIZZA = "Server.Items.PumpkinPizza" : CHEESE_PIZZA, v0;
    construct: Some(|w| {
        let id = new_food(w, &PUMPKIN_PIZZA, 0x1040, 1);
        ops::set_stackable(w, id, false);
        set_fill_factor(w, id, 6);
        crate::simple::hue(w, id, 0xF3);
        id
    }),
    default_name: |_, _| Some("Pumpkin Pizza".to_string()),
}

use crate::food::beverage::{BEVERAGE_BOTTLE, BeverageType, PITCHER, new_full, set_quantity};

leaf! {
    /// `Server.Items.HarvestWine`.
    HARVEST_WINE = "Server.Items.HarvestWine" : BEVERAGE_BOTTLE, v0;
    construct: Some(|w| {
        let id = new_full(w, &HARVEST_WINE, BeverageType::Wine);
        crate::simple::hue(w, id, 0xe0);
        id
    }),
    default_name: |_, _| Some("Harvest Wine".to_string()),
    default_weight: |_, _| 1.0,
}

leaf! {
    /// `Server.Items.MurkyMilk` (`MaxQuantity` 5 is the Pitcher's).
    MURKY_MILK = "Server.Items.MurkyMilk" : PITCHER, v0;
    construct: Some(|w| {
        let id = new_full(w, &MURKY_MILK, BeverageType::Milk);
        crate::simple::hue(w, id, 0x3e5);
        let max = crate::food::beverage::max_quantity(w, id);
        set_quantity(w, id, max);
        let g = if utility::random_bool(&mut w.rng) { 0x09F0 } else { 0x09AD };
        crate::simple::item_id(w, id, g);
        id
    }),
    default_name: |_, _| Some("Murky Milk".to_string()),
    default_weight: |_, _| 1.0,
}

// ---- TwilightLantern ----

leaf! {
    /// `Server.Items.TwilightLantern`.
    TWILIGHT_LANTERN = "Server.Items.TwilightLantern" : LANTERN, v0;
    construct: Some(|w| {
        let id = crate::lights::lantern_new(w, &TWILIGHT_LANTERN);
        let h = if utility::random_bool(&mut w.rng) { 244 } else { 997 };
        crate::simple::hue(w, id, h);
        id
    }),
    default_name: |_, _| Some("Twilight Lantern".to_string()),
    allow_equipped_cast: |_, _, _| true,
    get_properties: |w, id, list| {
        (crate::lights::LANTERN.get_properties)(w, id, list);
        list.add(1060482); // Spell Channeling
    },
}

// ---- Valentine ----

leaf! {
    /// `Server.Items.CupidStatue` (`[Flippable(0x4F7C, 0x4F7D)]`).
    CUPID_STATUE = "Server.Items.CupidStatue" : ITEM_BASE, v0;
    construct: Some(|w| {
        let id = new_item(w, &CUPID_STATUE, 0x4F7D);
        blessed(w, id);
        id
    }),
    label_number: |_, _| 1099220, // cupid statue
}

/// `HeartShapedBox.m_DropSound` (static).
static HEART_DROP_SOUND: AtomicI32 = AtomicI32::new(0);

fn heart_prepare_sound(w: &World, from: MobileId) {
    let female = w.mobile(from).is_some_and(|m| m.female);
    HEART_DROP_SOUND.store(if female { 0x430 } else { 0x320 }, Ordering::Relaxed);
}

use muo_content::items::cont::{BASE_CONTAINER, new_base_container};

leaf! {
    /// `Server.Items.HeartShapedBox` (`[Flippable(0x49CA, 0x49CB)]`).
    HEART_SHAPED_BOX = "Server.Items.HeartShapedBox" : BASE_CONTAINER, v0;
    construct: Some(|w| new_base_container(w, &HEART_SHAPED_BOX, 0x49CA)),
    default_drop_sound: |_, _| HEART_DROP_SOUND.load(Ordering::Relaxed),
    on_drag_drop_into: |w, id, from, item, p| {
        heart_prepare_sound(w, from);
        (BASE_CONTAINER.on_drag_drop_into)(w, id, from, item, p)
    },
    on_drag_drop: |w, id, from, dropped| {
        heart_prepare_sound(w, from);
        (BASE_CONTAINER.on_drag_drop)(w, id, from, dropped)
    },
}

leaf! {
    /// `Server.Items.AnimatedHeartShapedBox` (`[Flippable(0x49CC, 0x49D0)]`).
    ANIMATED_HEART_SHAPED_BOX = "Server.Items.AnimatedHeartShapedBox" : HEART_SHAPED_BOX, v0;
    construct: Some(|w| {
        let id = new_base_container(w, &ANIMATED_HEART_SHAPED_BOX, 0x49CA);
        crate::simple::item_id(w, id, 0x49CC);
        id
    }),
}

/// `CupidsArrow.IsSigned`.
fn arrow_signed(w: &World, id: ItemId) -> bool {
    strs(w, id, 0).is_some() && strs(w, id, 1).is_some()
}

/// `CupidsArrow.CheckSeason(from)`: outside February it only warns (and returns true both ways).
fn check_season(w: &mut World, from: MobileId) -> bool {
    if w.now.month() == 2 {
        return true;
    }
    msg(w, from, 1152318); // You may not use this item out of season.
    true
}

fn arrow_text(w: &World, id: ItemId) -> String {
    format!(
        "{}\t{}",
        strs(w, id, 0).unwrap_or_default(),
        strs(w, id, 1).unwrap_or_default()
    )
}

/// `Server.Items.CupidsArrow`.
pub static CUPIDS_ARROW: ItemType = ItemType {
    full_name: "Server.Items.CupidsArrow",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = new_item(w, &CUPIDS_ARROW, 0x4F7F);
        blessed(w, id);
        id
    }),
    label_number: |_, _| 1152270, // Cupid's Arrow 2012
    add_name_property: |w, id, list| {
        (ITEM_BASE.add_name_property)(w, id, list);
        if arrow_signed(w, id) {
            list.add_str(1152273, &arrow_text(w, id)); // ~1_val~ is madly in love with ~2_val~
        }
    },
    on_single_click: |w, id, from| {
        (ITEM_BASE.on_single_click)(w, id, from);
        if arrow_signed(w, id) {
            ops::label_to_number(w, id, from, 1152273, &arrow_text(w, id));
        }
    },
    on_double_click: |w, id, from| {
        if arrow_signed(w, id) || !check_season(w, from) {
            return;
        }
        if !in_pack(w, id, from) {
            msg(w, from, 1080063); // This must be in your backpack to use it.
            return;
        }
        target::begin_target(w, from, 10, false, TargetFlags::NONE, move |w, from, t| {
            arrow_target(w, id, from, t)
        });
        deps::send_message(w, from, "Who do you wish to use this on?");
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        write_strs(w, id, out, 2);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        read_strs(w, id, r, 2)
    },
    ..ITEM_BASE
};

fn mobile_name(w: &World, m: MobileId) -> Option<String> {
    let ty = w.mobile(m)?.ty;
    (ty.name)(w, m)
}

/// `CupidsArrow.OnTarget`.
fn arrow_target(w: &mut World, id: ItemId, from: MobileId, t: Targeted) {
    if arrow_signed(w, id) || !in_pack(w, id, from) {
        return;
    }
    if let Targeted::Entity(EntityId::Mobile(m)) = t {
        if !muo_content::mx::alive(w, m) {
            // That target is dead and even Cupid's arrow won't make them love you.
            msg(w, from, 1152269);
            return;
        }
        let (a, b) = (mobile_name(w, from), mobile_name(w, m));
        set_strs(w, id, 0, a);
        set_strs(w, id, 1, b);
        ops::invalidate_properties(w, id);
        deps::send_message(w, from, "You inscribe the arrow.");
    } else {
        deps::send_message(w, from, "That is not a person.");
    }
}

// ---- StValentinesBear ----

fn bear_signed(w: &World, id: ItemId) -> bool {
    (1..=3).any(|n| strs(w, id, n).is_some())
}

fn bear_can_sign(w: &World, id: ItemId) -> bool {
    let limit = w
        .item(id)
        .and_then(|i| i.ext.get::<EditLimit>())
        .map_or(DateTime::default(), |e| e.0);
    !bear_signed(w, id) || w.now <= limit
}

/// `StValentinesBear(itemid, name)` of class `ty`.
fn bear_new(w: &mut World, ty: &'static ItemType, itemid: i32, name: Option<&str>) -> ItemId {
    let id = new_item(w, ty, itemid);
    set_strs(w, id, 0, name.map(String::from));
    blessed(w, id);
    id
}

/// `Server.Items.StValentinesBear` (abstract).
pub static ST_VALENTINES_BEAR: ItemType = ItemType {
    full_name: "Server.Items.StValentinesBear",
    parent: Some(&ITEM_BASE),
    construct: None,
    make: None,
    default_name: |w, id| {
        Some(match strs(w, id, 0) {
            Some(o) => format!("{o}'s St. Valentine Bear"),
            None => "St. Valentine Bear".to_string(),
        })
    },
    add_name_property: |w, id, list| {
        match strs(w, id, 0) {
            Some(o) => list.add_str(1150295, &o), // ~1_NAME~'s St. Valentine Bear
            None => list.add(1150294),            // St. Valentine Bear
        }
        for (n, cliloc) in [(1, 1150301), (2, 1150302), (3, 1150303)] {
            if let Some(l) = strs(w, id, n) {
                list.add_str(cliloc, &l); // [ ~1_LINEn~ ]
            }
        }
    },
    on_single_click: |w, id, from| {
        (ITEM_BASE.on_single_click)(w, id, from);
        for (n, cliloc) in [(1, 1150301), (2, 1150302), (3, 1150303)] {
            if let Some(l) = strs(w, id, n) {
                ops::label_to_number(w, id, from, cliloc, &l);
            }
        }
    },
    on_double_click: |w, id, from| {
        if !check_season(w, from) || !bear_can_sign(w, id) {
            return;
        }
        if !in_pack(w, id, from) {
            msg(w, from, 1080063); // This must be in your backpack to use it.
            return;
        }
        if super::util::net_state(w, from).is_none() || super::util::deleted(w, id) {
            return;
        }
        send_gump_to_mobile(
            w,
            from,
            Box::new(StaticGumpObj::new(50, 50, BearGump { bear: id })),
            false,
        );
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        write_strs(w, id, out, 4);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        read_strs(w, id, r, 4)
    },
    ..ITEM_BASE
};

/// `StValentinesBearsGump`.
struct BearGump {
    bear: ItemId,
}

impl StaticGumpLogic for BearGump {
    const TYPE_NAME: &'static str = "Server.Items.StValentinesBear+StValentinesBearsGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 420, 320, 9300);
        b.add_html_localized_args(10, 10, 400, 21, 1154645, "#1150294", 0, false, false); // St. Valentine Bear
        // Enter up to three lines of personalized greeting for your St. Valentine Bear. ..
        b.add_html_localized(10, 40, 400, 75, 1150293, false, false);
        b.add_html_localized(10, 129, 400, 21, 1150296, false, false); // Line 1:
        b.add_background(10, 150, 400, 24, 9350);
        b.add_text_entry_limited(15, 152, 390, 20, 0, 0, "", 25);
        b.add_html_localized(10, 179, 400, 21, 1150297, false, false); // Line 2:
        b.add_background(10, 200, 400, 24, 9350);
        b.add_text_entry_limited(15, 202, 390, 20, 0, 1, "", 25);
        b.add_html_localized(10, 229, 400, 21, 1150298, false, false); // Line 3:
        b.add_background(10, 250, 400, 24, 9350);
        b.add_text_entry_limited(15, 252, 390, 20, 0, 2, "", 25);
        b.add_button(15, 285, 242, 241, 0);
        b.add_button(335, 285, 247, 248, 1);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let bear = self.bear;
        if super::util::deleted(w, bear)
            || !in_pack(w, bear, from)
            || !bear_can_sign(w, bear)
            || info.button_id != 1
        {
            return;
        }
        let line = |n| info.get_text_entry(n).unwrap_or_default().to_string();
        let (l1, l2, l3) = (line(0), line(1), line(2));
        if l1.is_empty() || l2.is_empty() || l3.is_empty() {
            deps::send_message(w, from, "Lines cannot be left blank.");
            return;
        }
        if [&l1, &l2, &l3].iter().any(|l| l.chars().count() > 25) {
            deps::send_message(w, from, "Lines may not exceed 25 characters.");
            return;
        }
        if !bear_signed(w, bear) {
            let limit = w.now + TimeSpan::from_mins(10);
            if let Some(i) = w.item_mut(bear) {
                i.ext.insert(EditLimit(limit));
            }
        }
        for (n, l) in [(1, l1), (2, l2), (3, l3)] {
            set_strs(w, bear, n, Some(utility::fix_html(&l)));
        }
        ops::invalidate_properties(w, bear);
        deps::send_message(
            w,
            from,
            "You add the personalized greeting to your St. Valentine Bear.",
        );
    }
}

/// `StValentinesPanda(name)`.
pub fn st_valentines_panda_new(w: &mut World, name: Option<&str>) -> ItemId {
    bear_new(w, &ST_VALENTINES_PANDA, 0x48E0, name)
}

/// `StValentinesPolarBear(name)`.
pub fn st_valentines_polar_bear_new(w: &mut World, name: Option<&str>) -> ItemId {
    bear_new(w, &ST_VALENTINES_POLAR_BEAR, 0x48E2, name)
}

leaf! {
    /// `Server.Items.StValentinesPanda` (`[Flippable(0x48E0, 0x48E1)]`).
    ST_VALENTINES_PANDA = "Server.Items.StValentinesPanda" : ST_VALENTINES_BEAR, v0;
    construct: Some(|w| st_valentines_panda_new(w, None)),
}

leaf! {
    /// `Server.Items.StValentinesPolarBear` (`[Flippable(0x48E2, 0x48E3)]`).
    ST_VALENTINES_POLAR_BEAR = "Server.Items.StValentinesPolarBear" : ST_VALENTINES_BEAR, v0;
    construct: Some(|w| st_valentines_polar_bear_new(w, None)),
}

// ---- `[SerializedCommandProperty(AccessLevel.GameMaster)]` string properties ----

macro_rules! str_props {
    ($($name:literal => $idx:expr),+ $(,)?) => {&[$(
        PropDef {
            name: $name,
            kind: Kind::Plain(PropType::Str),
            read: AccessLevel::GameMaster,
            write: AccessLevel::GameMaster,
            get: |w, e| match e {
                EntityId::Item(i) => strs(w, i, $idx).map_or(Value::Null, Value::Str),
                _ => Value::Null,
            },
            set: Some(|w, e, v| {
                let EntityId::Item(i) = e else { return Err("not an item".to_string()) };
                set_strs(w, i, $idx, v.string());
                ops::invalidate_properties(w, i);
                Ok(())
            }),
        },
    )+]};
}

static DIPPER_PROPS: &[PropDef] = str_props!("Dipper" => 0);
static ARROW_PROPS: &[PropDef] = str_props!("From" => 0, "To" => 1);
static BEAR_PROPS: &[PropDef] = str_props!("Owner" => 0, "Line1" => 1, "Line2" => 2, "Line3" => 3);

/// `[CommandProperty(AccessLevel.GameMaster)] public DateTime EditLimit { get; set; }` (not serialized).
pub(crate) static BEAR_EDIT_LIMIT_PROPS: &[PropDef] = &[PropDef {
    name: "EditLimit",
    kind: Kind::DateTime,
    read: AccessLevel::GameMaster,
    write: AccessLevel::GameMaster,
    get: |w, e| match e {
        EntityId::Item(i) => Value::DateTime(
            w.item(i)
                .and_then(|x| x.ext.get::<EditLimit>())
                .map_or(DateTime::default(), |l| l.0),
        ),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let (EntityId::Item(i), Value::DateTime(d)) = (e, v) else {
            return Err("not a date".to_string());
        };
        if let Some(x) = w.item_mut(i) {
            x.ext.insert(EditLimit(d));
        }
        Ok(())
    }),
}];

// ---- PlayerBones ----

/// `PlayerBones(name)`: `base(Utility.RandomMinMax(0x0ECA, 0x0ED2))`, `Name = "{name}'s bones"`, 3 in 10 get a hue.
pub fn player_bones_new(w: &mut World, name: &str) -> ItemId {
    let g = utility::random_min_max(&mut w.rng, 0x0ECA, 0x0ED2);
    let id = new_base_container(w, &PLAYER_BONES, g);
    ops::set_name(w, id, Some(format!("{name}'s bones")));
    match utility::random(&mut w.rng, 10) {
        0 => crate::simple::hue(w, id, 0xa09),
        1 => crate::simple::hue(w, id, 0xa93),
        2 => crate::simple::hue(w, id, 0xa47),
        _ => {}
    }
    id
}

leaf! {
    /// `Server.Engines.Events.PlayerBones`.
    PLAYER_BONES = "Server.Engines.Events.PlayerBones" : BASE_CONTAINER, v0;
    construct: None,
    make: Some(|w| player_bones_new(w, "")),
}

// ---- registration ----

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &DECORATIVE_TOPIARY,
        &FESTIVE_CACTUS,
        &SNOWY_TREE,
        &LIGHT_OF_THE_WINTER_SOLSTICE,
        &PILE_OF_GLACIAL_SNOW,
        &SNOW_PILE,
        &PUMPKIN_SCARECROW,
        &RUINED_TAPESTRY,
        &GRIM_WARNING,
        &SKULLS_ON_PIKE,
        &CHAIR_IN_A_GHOST_COSTUME,
        &EXCELLENT_IRON_MAIDEN,
        &HALLOWEEN_GUILLOTINE,
        &COLORED_SMALL_WEBS,
        &HALLOWEEN_PUMPKIN,
        &PAINTED_DAEMON_MASK,
        &PAINTED_EVIL_CLOWN_MASK,
        &PAINTED_PLAGUE_MASK,
        &PAINTED_EVIL_JESTER_MASK,
        &PAINTED_PORCELAIN_MASK,
        &DRAGON_EASTER_EGG,
        &JELLY_BEANS,
        &NOUGAT_SWIRL,
        &TAFFY,
        &WRAPPED_CANDY,
        &CREEPY_CAKE,
        &MR_PLAINS_COOKIES,
        &PUMPKIN_PIZZA,
        &HARVEST_WINE,
        &MURKY_MILK,
        &TWILIGHT_LANTERN,
        &CUPID_STATUE,
        &HEART_SHAPED_BOX,
        &ANIMATED_HEART_SHAPED_BOX,
        &CUPIDS_ARROW,
        &ST_VALENTINES_PANDA,
        &ST_VALENTINES_POLAR_BEAR,
        &PLAYER_BONES,
    ] {
        types.register_item(t, &[]);
    }
    types.register_item(&ANGEL_DECORATION, &["Server.Items.AngelDecoration"]);
    types.register_item(&ROCKING_HORSE, &["Server.Items.RockingHorse"]);
    types.register_item(&LOLLIPOPS, &["Server.Items.Lollipop"]);
    types.register_item(
        &BASE_PAINTED_MASK,
        &[
            "Server.Items.ClownMask",
            "Server.Items.DaemonMask",
            "Server.Items.PlagueMask",
        ],
    );
}

pub(super) fn configure(w: &mut World) {
    props::add(w, "Server.Items.LightOfTheWinterSolstice", DIPPER_PROPS);
    props::add(w, "Server.Items.CupidsArrow", ARROW_PROPS);
    props::add(w, "Server.Items.StValentinesBear", BEAR_PROPS);
    props::add(w, "Server.Items.StValentinesBear", BEAR_EDIT_LIMIT_PROPS);
    // `Dye(from, sender)`: `Deleted` fails, otherwise `Hue = sender.DyedHue`.
    register_dyable(w, &DRAGON_EASTER_EGG, |w, id, _from, tub| {
        if ops::is_deleted(w, id) {
            return false;
        }
        let hue = muo_content::items::clothing::tub_dyed_hue(w, tub);
        crate::simple::hue(w, id, hue);
        true
    });
}

#[allow(dead_code)]
fn unused(_: Point3D) {}
