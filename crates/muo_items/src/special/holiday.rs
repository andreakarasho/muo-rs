//! Holiday items: gift boxes, `WinterGiftPackage2003`, stockings, `HolidayBell`, snow piles, `IcyPatch`, `Snowman`
//! and the holiday foods. The flip and furniture lists are the
//! name tables of `muo_content::items::flip_table`.
//!
//! HolidayTimepiece, WristWatch and the tree are in `holiday3`.

use muo_content::items::cont::{BASE_CONTAINER, new_base_container};
use muo_content::items::food::{FOOD, FoodClass, data as food_data, new_food};
use muo_content::items::interfaces::register_dyable;
use muo_io::TextDefinition;
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;
use std::collections::HashMap;

use super::util::{delete, msg};
use crate::simple::leaf;

// ---- GiftBox, HolidayGiftBoxes ----

/// `GiftBox(hue)`: `base(Utility.Random(0x232A, 2))`.
pub fn gift_box_new_with(w: &mut World, hue: i32) -> ItemId {
    let g = utility::random_from(&mut w.rng, 0x232A, 2);
    let id = new_base_container(w, &GIFT_BOX, g);
    super::hue(w, id, hue);
    id
}

/// `GiftBox()`: `this(Utility.RandomDyedHue())`.
pub fn gift_box_new(w: &mut World) -> ItemId {
    let hue = utility::random_dyed_hue(&mut w.rng);
    gift_box_new_with(w, hue)
}

leaf! {
    /// `Server.Items.GiftBox` (`[Furniture]`, `[Flippable(0x232A, 0x232B)]`).
    GIFT_BOX = "Server.Items.GiftBox" : BASE_CONTAINER, v0;
    construct: Some(gift_box_new),
    default_weight: crate::class_props::default_weight,
}

/// `WinterGiftPackage2003()`.
pub fn winter_gift_package_new(w: &mut World) -> ItemId {
    let g = utility::random_from(&mut w.rng, 0x232A, 2);
    let id = new_base_container(w, &WINTER_GIFT_PACKAGE_2003, g);
    let hue = utility::random_dyed_hue(&mut w.rng);
    super::hue(w, id, hue);
    for name in [
        "Server.Items.Snowman",
        "Server.Items.WreathDeed",
        "Server.Items.BlueSnowflake",
        "Server.Items.RedPoinsettia",
    ] {
        if let Some(item) = muo_content::factory::make_item(w, name)
            && let Some(ty) = w.item(id).map(|i| i.ty)
        {
            (ty.drop_item)(w, id, item);
        }
    }
    id
}

leaf! {
    /// `Server.Items.WinterGiftPackage2003` (`[Flippable(0x232A, 0x232B)]`).
    WINTER_GIFT_PACKAGE_2003 = "Server.Items.WinterGiftPackage2003" : GIFT_BOX, enc;
    construct: Some(winter_gift_package_new),
}

const NORMAL_HUES: [i32; 20] = [
    0x672, 0x454, 0x507, 0x4ac, 0x504, 0x84b, 0x495, 0x97c, 0x493, 0x4a8, 0x494, 0x4aa, 0xb8b,
    0x84f, 0x491, 0x851, 0x503, 0xb8c, 0x4ab, 0x84B,
];
const NEON_HUES: [i32; 6] = [0x438, 0x424, 0x433, 0x445, 0x42b, 0x448];

fn random_gift_box_hue(w: &mut World) -> i32 {
    utility::random_element(&mut w.rng, &NORMAL_HUES)
        .copied()
        .unwrap_or(0)
}

macro_rules! gift_box {
    ($($(#[$m:meta])* $name:ident = $full:literal, $gump:expr, |$w:ident| $graphic:expr, $hue:ident;)*) => {$(
        leaf! {
            $(#[$m])*
            $name = $full : BASE_CONTAINER, v0;
            construct: Some(|$w| {
                let g = $graphic;
                let id = new_base_container($w, &$name, g);
                let h = $hue($w);
                super::hue($w, id, h);
                id
            }),
            default_gump_id: |_, _| $gump,
        }
    )*};
}

fn random_neon_box_hue(w: &mut World) -> i32 {
    utility::random_element(&mut w.rng, &NEON_HUES)
        .copied()
        .unwrap_or(0)
}

gift_box! {
    /// `Server.Items.GiftBoxRectangle` (`[Flippable(0x46A5, 0x46A6)]`).
    GIFT_BOX_RECTANGLE = "Server.Items.GiftBoxRectangle", 0x11E, |w| if utility::random_bool(&mut w.rng) { 0x46A5 } else { 0x46A6 }, random_gift_box_hue;
    /// `Server.Items.GiftBoxCube`.
    GIFT_BOX_CUBE = "Server.Items.GiftBoxCube", 0x11B, |_w| 0x46A2, random_gift_box_hue;
    /// `Server.Items.GiftBoxCylinder`.
    GIFT_BOX_CYLINDER = "Server.Items.GiftBoxCylinder", 0x11C, |_w| 0x46A3, random_gift_box_hue;
    /// `Server.Items.GiftBoxOctogon`.
    GIFT_BOX_OCTOGON = "Server.Items.GiftBoxOctogon", 0x11D, |_w| 0x46A4, random_gift_box_hue;
    /// `Server.Items.GiftBoxAngel`.
    GIFT_BOX_ANGEL = "Server.Items.GiftBoxAngel", 0x11F, |_w| 0x46A7, random_gift_box_hue;
    /// `Server.Items.GiftBoxNeon` (`[Flippable(0x232A, 0x232B)]`).
    GIFT_BOX_NEON = "Server.Items.GiftBoxNeon", 0x11A, |w| if utility::random_bool(&mut w.rng) { 0x232A } else { 0x232B }, random_neon_box_hue;
}

// ---- Stockings ----

leaf! {
    /// `Server.Items.GreenStocking` (`[Flippable(0x2bd9, 0x2bda)]`).
    GREEN_STOCKING = "Server.Items.GreenStocking" : BASE_CONTAINER, enc;
    construct: Some(|w| {
        let g = utility::random_from(&mut w.rng, 0x2BD9, 2);
        new_base_container(w, &GREEN_STOCKING, g)
    }),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

leaf! {
    /// `Server.Items.RedStocking` (`[Flippable(0x2bdb, 0x2bdc)]`).
    RED_STOCKING = "Server.Items.RedStocking" : BASE_CONTAINER, enc;
    construct: Some(|w| {
        let g = utility::random_from(&mut w.rng, 0x2BDB, 2);
        new_base_container(w, &RED_STOCKING, g)
    }),
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
}

// ---- SnowPiles ----

/// `SnowPileDeco(itemid)`.
pub fn snow_pile_deco_new_with(w: &mut World, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &SNOW_PILE_DECO, item_id);
    super::hue(w, id, 0x481);
    id
}

leaf! {
    /// `Server.Items.SnowPileDeco`.
    SNOW_PILE_DECO = "Server.Items.SnowPileDeco" : ITEM_BASE, v0;
    construct: Some(|w| {
        let types = [0x8E2, 0x8E0, 0x8E6, 0x8E5, 0x8E3];
        let g = utility::random_element(&mut w.rng, &types).copied().unwrap_or(0x8E2);
        snow_pile_deco_new_with(w, g)
    }),
    label_number: crate::class_props::label_number, // decorative snow piles
    default_weight: crate::class_props::default_weight,
}

// ---- HolidayBell ----

const BELL_HUES: [i32; 15] = [
    0xA, 0x24, 0x42, 0x56, 0x1A, 0x4C, 0x3C, 0x60, 0x2E, 0x55, 0x23, 0x38, 0x482, 0x6, 0x10,
];

/// `HolidayBell._maker` and `_soundID`.
#[derive(Clone, Debug)]
pub struct BellData {
    pub maker: Option<TextDefinition>,
    pub sound_id: i32,
}

pub fn bell(w: &World, id: ItemId) -> BellData {
    w.item(id)
        .and_then(|i| i.ext.get::<BellData>())
        .cloned()
        .unwrap_or(BellData {
            maker: None,
            sound_id: 0,
        })
}

pub fn set_bell(w: &mut World, id: ItemId, d: BellData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

/// `HolidayBell(maker)`.
pub fn holiday_bell_new_with(w: &mut World, maker: Option<TextDefinition>) -> ItemId {
    let id = crate::simple::from_template(w, &HOLIDAY_BELL, "Server.Items.HolidayBell");
    let h = utility::random_element(&mut w.rng, &BELL_HUES)
        .copied()
        .unwrap_or(0);
    super::hue(w, id, h);
    let sound_id = 0x0F5 + utility::random(&mut w.rng, 14);
    set_bell(w, id, BellData { maker, sound_id });
    id
}

/// `Server.Items.HolidayBell`.
pub static HOLIDAY_BELL: ItemType = ItemType {
    full_name: "Server.Items.HolidayBell",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let n = 1041441 + utility::random(&mut w.rng, 22);
        holiday_bell_new_with(w, Some(TextDefinition::of_number(n)))
    }),
    label_number: |w, id| match bell(w, id).maker {
        Some(m) if m.number > 0 => m.number,
        _ => (ITEM_BASE.label_number)(w, id),
    },
    default_name: |w, id| match bell(w, id).maker.and_then(|m| m.string) {
        Some(s) => Some(format!("a holiday bell from {s}")),
        None => Some("a holiday bell".to_string()),
    },
    on_double_click: |w, id, from| {
        let loc = ops::get_world_location(w, id);
        if !muo_content::mx::in_range_point(w, from, loc, 2) {
            msg(w, from, 500446); // That is too far away.
        } else {
            muo_content::mx::play_sound(w, from, bell(w, id).sound_id);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let d = bell(w, id);
        out.write_text_definition(d.maker.as_ref());
        out.write_i32(d.sound_id);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let maker = r.read_text_definition()?;
        let sound_id = r.read_i32()?;
        set_bell(w, id, BellData { maker, sound_id });
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Snowman ----

const SNOWMAN_TITLES: [&str; 53] = [
    "Backflash",
    "Carbon",
    "Colbalistic",
    "Comforl",
    "Coppacchia",
    "Cyrus",
    "DannyB",
    "DJSoul",
    "DraconisRex",
    "Earia",
    "Foster",
    "Gonzo",
    "Haan",
    "Halona",
    "Hugo",
    "Hyacinth",
    "Imirian",
    "Jinsol",
    "Liciatia",
    "Loewen",
    "Loke",
    "Magnus",
    "Maleki",
    "Morpheus",
    "Obberron",
    "Odee",
    "Orbeus",
    "Pax",
    "Phields",
    "Pigpen",
    "Platinum",
    "Polpol",
    "Prume",
    "Quinnly",
    "Ragnarok",
    "Rend",
    "Roland",
    "RyanM",
    "Screach",
    "Seraph",
    "Silvani",
    "Sherbear",
    "SkyWalker",
    "Snark",
    "Sowl",
    "Spada",
    "Starblade",
    "Tenacious",
    "Tnez",
    "Wasia",
    "Zilo",
    "Zippy",
    "Zoer",
];

/// `Snowman._title`.
#[derive(Clone, Debug, Default)]
struct Title(Option<String>);

pub fn title(w: &World, id: ItemId) -> Option<String> {
    w.item(id)
        .and_then(|i| i.ext.get::<Title>())
        .and_then(|t| t.0.clone())
}

fn set_title_raw(w: &mut World, id: ItemId, t: Option<String>) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(Title(t));
    }
}

/// `Snowman.Title = value` (`[InvalidateProperties]`).
pub fn set_snowman_title(w: &mut World, id: ItemId, t: Option<String>) {
    if title(w, id) != t {
        set_title_raw(w, id, t);
        ops::invalidate_properties(w, id);
    }
}

/// `Snowman.GetRandomTitle()`.
pub fn random_title(w: &mut World) -> String {
    utility::random_element(&mut w.rng, &SNOWMAN_TITLES)
        .copied()
        .unwrap_or("Zoer")
        .to_string()
}

/// `Snowman(hue, title)`.
pub fn snowman_new_with(w: &mut World, hue: i32, t: Option<String>) -> ItemId {
    let g = utility::random_from(&mut w.rng, 0x2328, 2);
    let id = ops::new_item_with_id(w, &SNOWMAN, g);
    super::hue(w, id, hue);
    ops::set_loot_type(w, id, LootType::Blessed);
    set_title_raw(w, id, t);
    id
}

/// `Server.Items.Snowman` (`IDyable`, `[Flippable(0x2328, 0x2329)]`).
pub static SNOWMAN: ItemType = ItemType {
    full_name: "Server.Items.Snowman",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let hue = utility::random_dyed_hue(&mut w.rng);
        let t = random_title(w);
        snowman_new_with(w, hue, Some(t))
    }),
    default_weight: crate::class_props::default_weight,
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        if let Some(t) = title(w, id) {
            list.add_str(1062841, &t); // ~1_NAME~ the Snowman
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_opt_string(title(w, id).as_deref());
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let t = r.read_string()?;
        set_title_raw(w, id, t);
        Ok(())
    },
    ..ITEM_BASE
};

// ---- IcyPatch ----

/// `IcyPatch(itemid)`.
pub fn icy_patch_new_with(w: &mut World, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &ICY_PATCH, item_id);
    super::hue(w, id, 0x481);
    id
}

/// `IcyPatch.RunSequence(m, message, freeze)`.
fn run_sequence(w: &mut World, id: ItemId, m: MobileId, message: i32, freeze: bool) {
    if freeze {
        muo_content::mx::set_frozen(w, m, true);
        let secs = if message == 1095162 { 2.0 } else { 1.25 };
        timer::start(
            w,
            TimeSpan::from_secs_f64(secs),
            TimeSpan::ZERO,
            1,
            move |w| {
                muo_content::mx::set_frozen(w, m, false);
            },
        );
    }
    msg(w, m, message);
    let (mut action, mut sound) = (0, 0);
    let female = w.mobile(m).is_some_and(|x| x.female);
    if message == 1095162 {
        // You lose your footing and ungracefully splatter on the ground.
        muo_content::creature::mount::dismount(w, m);
        let (loc, map) = w
            .item(id)
            .map_or((Point3D::default(), None), |i| (i.location, i.map));
        let mut p = loc;
        if muo_content::spells::helper::find_valid_spawn_location(w, map, &mut p, true) {
            timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                if let Some(mm) = map {
                    muo_server::mobile::base::move_to_world(w, m, p, mm);
                }
            });
        }
        action = 21 + utility::random(&mut w.rng, 2);
        sound = if female { 0x317 } else { 0x426 };
    } else if message == 1095161 {
        // You skillfully manage to maintain your balance.
        action = 17;
        sound = if female { 0x319 } else { 0x429 };
    }
    if action > 0 {
        timer::start(
            w,
            TimeSpan::from_secs_f64(0.4),
            TimeSpan::ZERO,
            1,
            move |w| {
                if !muo_server::mobile::part1::props_b::mounted(w, m) {
                    muo_server::mobile::base::animate(w, m, action, 1, 1, false, true, 0);
                }
                muo_content::mx::play_sound(w, m, sound);
            },
        );
    }
}

/// `Server.Items.IcyPatch`.
pub static ICY_PATCH: ItemType = ItemType {
    full_name: "Server.Items.IcyPatch",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let g = if utility::random(&mut w.rng, 10) == 0 {
            0x122A
        } else {
            0x122F
        };
        icy_patch_new_with(w, g)
    }),
    label_number: crate::class_props::label_number, // An Icy Patch
    default_weight: crate::class_props::default_weight,
    on_move_over: |w, id, m| {
        let ok = w.mobile(m).is_some_and(|x| {
            x.player
                && (x.ty.alive)(w, m)
                && x.access_level == muo_server::mobile::AccessLevel::Player
        });
        if ok {
            match utility::random(&mut w.rng, 3) {
                0 => run_sequence(w, id, m, 1095160, false), // You steadily walk over the slippery surface.
                1 => run_sequence(w, id, m, 1095161, true),
                _ => run_sequence(w, id, m, 1095162, true),
            }
        }
        (ITEM_BASE.on_move_over)(w, id, m)
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

// ---- HolidayFoods ----

/// `CandyCane._toothAches`: the eaten counter and the timer of each eater.
#[derive(Default)]
struct ToothAches(HashMap<MobileId, (i32, Option<TimerHandle>)>);

/// `CandyCane.GetToothAche(from)`.
pub fn get_tooth_ache(w: &World, from: MobileId) -> i32 {
    w.resources
        .get::<ToothAches>()
        .and_then(|t| t.0.get(&from))
        .map_or(0, |e| e.0)
}

fn ensure_timer(w: &mut World, from: MobileId) {
    if w.resources
        .get::<ToothAches>()
        .is_some_and(|t| t.0.contains_key(&from))
    {
        return;
    }
    let h = timer::start(
        w,
        TimeSpan::from_secs(30),
        TimeSpan::from_secs(30),
        0,
        move |w| candy_tick(w, from),
    );
    w.resources
        .get_mut::<ToothAches>()
        .0
        .insert(from, (0, Some(h)));
}

/// `CandyCane.SetToothAche(from, value)`.
pub fn set_tooth_ache(w: &mut World, from: MobileId, value: i32) {
    ensure_timer(w, from);
    if let Some(e) = w.resources.get_mut::<ToothAches>().0.get_mut(&from) {
        e.0 = value;
    }
}

fn candy_tick(w: &mut World, eater: MobileId) {
    let Some(entry) = w.resources.get_mut::<ToothAches>().0.get_mut(&eater) else {
        return;
    };
    entry.0 -= 1;
    let eaten = entry.0;
    let gone = muo_content::mx::deleted(w, eater);
    if gone || eaten <= 0 {
        if let Some((_, Some(h))) = w.resources.get_mut::<ToothAches>().0.remove(&eater) {
            timer::stop(w, h);
        }
    } else if muo_content::mx::map(w, eater) != Some(muo_server::map::MapId::INTERNAL)
        && muo_content::mx::alive(w, eater)
    {
        if eaten > 60 {
            let n = 1077388 + utility::random(&mut w.rng, 5);
            muo_server::mobile::part2::messages::say_number(w, eater, n, "");
            let human = w.mobile(eater).is_some_and(|_| {
                w.resources
                    .get::<muo_primitives::BodyTable>()
                    .is_some_and(|t| t.is_human(muo_server::mobile::part1::props_b::body(w, eater)))
            });
            if utility::random_bool(&mut w.rng)
                && human
                && !muo_server::mobile::part1::props_b::mounted(w, eater)
            {
                muo_server::mobile::base::animate(w, eater, 32, 5, 1, true, false, 0);
            }
        } else if eaten == 60 {
            msg(w, eater, 1077393); // The extreme pain in your teeth subsides.
        }
    }
}

/// `CandyCane.Eat(from)`: `CheckHunger` adds 32 to the tooth ache instead of filling hunger.
fn candy_eat(w: &mut World, id: ItemId, from: MobileId) -> bool {
    ensure_timer(w, from);
    if let Some(e) = w.resources.get_mut::<ToothAches>().0.get_mut(&from) {
        e.0 += 32;
    }
    msg(w, from, 1077387); // You feel as if you could eat as much as you wanted!
    let s = utility::random_from(&mut w.rng, 0x3A, 3);
    muo_server::mobile::part2::effects::play_sound(w, from, s);
    let human = w.mobile(from).is_some_and(|_| {
        w.resources
            .get::<muo_primitives::BodyTable>()
            .is_some_and(|t| t.is_human(muo_server::mobile::part1::props_b::body(w, from)))
    });
    if human && !muo_server::mobile::part1::props_b::mounted(w, from) {
        muo_server::mobile::base::animate(w, from, 34, 5, 1, true, false, 0);
    }
    if let Some(d) = food_data(w, id).cloned()
        && let Some(p) = d.poison
    {
        muo_server::mobile::base::apply_poison(w, from, d.poisoner, p);
    }
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.consume)(w, id, 1);
    }
    true
}

static CANDY_CANE_CLASS: FoodClass = FoodClass { eat: candy_eat };

/// `CandyCane(itemID)`.
pub fn candy_cane_new_with(w: &mut World, item_id: i32) -> ItemId {
    let id = new_food(w, &CANDY_CANE, item_id, 1);
    ops::set_stackable(w, id, false);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

leaf! {
    /// `Server.Items.CandyCane`.
    CANDY_CANE = "Server.Items.CandyCane" : FOOD, v0;
    construct: Some(|w| {
        let g = 0x2bdd + utility::random(&mut w.rng, 4);
        candy_cane_new_with(w, g)
    }),
    family: Some(&CANDY_CANE_CLASS),
}

const GINGER_MESSAGES: [i32; 8] = [
    0, 1077396, 1077397, 1077405, 1077406, 1077407, 1077408, 1077409,
];

fn ginger_eat(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let m = utility::random_element(&mut w.rng, &GINGER_MESSAGES)
        .copied()
        .unwrap_or(0);
    if m != 0 {
        if !ops::is_deleted(w, id) && deps::can_see(w, from, id) {
            muo_server::item::messages::send_localized_hue(w, id, from, m, 0, "");
        }
        return false;
    }
    (muo_content::items::food::BASE_FOOD_CLASS.eat)(w, id, from)
}

static GINGER_CLASS: FoodClass = FoodClass { eat: ginger_eat };

leaf! {
    /// `Server.Items.GingerBreadCookie`.
    GINGER_BREAD_COOKIE = "Server.Items.GingerBreadCookie" : FOOD, v0;
    construct: Some(|w| {
        let g = if utility::random_bool(&mut w.rng) { 0x2be1 } else { 0x2be2 };
        let id = new_food(w, &GINGER_BREAD_COOKIE, g, 1);
        ops::set_stackable(w, id, false);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    family: Some(&GINGER_CLASS),
}

// ---- registration ----

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &GIFT_BOX,
        &WINTER_GIFT_PACKAGE_2003,
        &GIFT_BOX_RECTANGLE,
        &GIFT_BOX_CUBE,
        &GIFT_BOX_CYLINDER,
        &GIFT_BOX_OCTOGON,
        &GIFT_BOX_ANGEL,
        &GIFT_BOX_NEON,
        &GREEN_STOCKING,
        &RED_STOCKING,
        &SNOW_PILE_DECO,
        &HOLIDAY_BELL,
        &SNOWMAN,
        &ICY_PATCH,
        &CANDY_CANE,
        &GINGER_BREAD_COOKIE,
    ] {
        types.register_item(t, &[]);
    }
}

pub(super) fn configure(w: &mut World) {
    register_dyable(w, &SNOWMAN, |w, id, _from, tub| {
        if ops::is_deleted(w, id) {
            return false;
        }
        let hue = muo_content::items::clothing::tub_dyed_hue(w, tub);
        super::hue(w, id, hue);
        true
    });
}

#[allow(dead_code)]
fn unused(w: &mut World, id: ItemId) {
    delete(w, id);
}

// ---- SnowGlobes ----

/// `SnowGlobe()` of class `ty`: `base(0xE2F)`, blessed, `Light = Circle150`.
fn snow_globe_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SnowGlobe")
}

leaf! {
    /// `Server.Items.SnowGlobe` (no `[Constructible]`).
    SNOW_GLOBE = "Server.Items.SnowGlobe" : ITEM_BASE, v0;
    construct: None,
    make: Some(|w| snow_globe_new(w, &SNOW_GLOBE)),
    default_weight: crate::class_props::default_weight,
}

/// `SnowGlobeOne/Two/Three._place`.
#[derive(Clone, Copy, Debug, Default)]
struct Place(i32);

pub fn place(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<Place>())
        .map_or(0, |p| p.0)
}

/// `Place = value` (`[InvalidateProperties]`).
pub fn set_place(w: &mut World, id: ItemId, v: i32) {
    if place(w, id) != v {
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(Place(v));
        }
        ops::invalidate_properties(w, id);
    }
}

fn globe_new(w: &mut World, ty: &'static ItemType, p: i32) -> ItemId {
    let id = snow_globe_new(w, ty);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(Place(p));
    }
    id
}

macro_rules! globe {
    ($(#[$m:meta])* $name:ident = $full:literal, $count:expr, $new:ident; $($f:ident : $v:expr,)*) => {
        /// `Class(type)`.
        pub fn $new(w: &mut World, p: i32) -> ItemId {
            globe_new(w, &$name, p)
        }

        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&SNOW_GLOBE),
            construct: Some(|w| {
                let p = utility::random(&mut w.rng, $count);
                $new(w, p)
            }),
            $($f: $v,)*
            serialize: |w, id, out| {
                (SNOW_GLOBE.serialize)(w, id, out);
                out.write_i32(1);
                out.write_enum(4, i64::from(place(w, id)));
            },
            deserialize: |w, id, r| {
                (SNOW_GLOBE.deserialize)(w, id, r)?;
                let version = r.read_i32()?;
                let p = if version < 1 { r.read_encoded_int()? } else { r.read_enum(4)? as i32 };
                if let Some(i) = w.item_mut(id) {
                    i.ext.insert(Place(p));
                }
                Ok(())
            },
            ..SNOW_GLOBE
        };
    };
}

globe! {
    /// `Server.Items.SnowGlobeOne`.
    SNOW_GLOBE_ONE = "Server.Items.SnowGlobeOne", 19, snow_globe_one_new;
    label_number: |w, id| 1041454 + place(w, id),
}

const SNOW_GLOBE_TWO_NAMES: [&str; 19] = [
    "Ancient Citadel",
    "Blackthorne's Castle",
    "City of Montor",
    "City of Mistas",
    "Exodus' Lair",
    "Lake of Fire",
    "Lakeshire",
    "Pass of Karnaugh",
    "The Ethereal Fortress",
    "Twin Oaks Tavern",
    "Chaos Shrine",
    "Shrine of Humility",
    "Shrine of Sacrifice",
    "Shrine of Compassion",
    "Shrine of Honor",
    "Shrine of Honesty",
    "Shrine of Spirituality",
    "Shrine of Justice",
    "Shrine of Valor",
];

globe! {
    /// `Server.Items.SnowGlobeTwo`.
    SNOW_GLOBE_TWO = "Server.Items.SnowGlobeTwo", 19, snow_globe_two_new;
    default_name: |w, id| {
        let idx = place(w, id);
        match usize::try_from(idx).ok().and_then(|i| SNOW_GLOBE_TWO_NAMES.get(i)) {
            Some(n) => Some(format!("a snowy scene of {n}")),
            None => Some("a snowy scene".to_string()),
        }
    },
}

globe! {
    /// `Server.Items.SnowGlobeThree`.
    SNOW_GLOBE_THREE = "Server.Items.SnowGlobeThree", 16, snow_globe_three_new;
    label_number: |w, id| {
        let p = place(w, id);
        if p >= 4 { 1075440 + (p - 4) } else { 1075294 + p }
    },
}

// ---- HolidayPottedPlant ----

/// `HolidayPottedPlant(itemID)`.
pub fn holiday_potted_plant_new_with(w: &mut World, item_id: i32) -> ItemId {
    ops::new_item_with_id(w, &HOLIDAY_POTTED_PLANT, item_id)
}

leaf! {
    /// `Server.Items.HolidayPottedPlant`.
    HOLIDAY_POTTED_PLANT = "Server.Items.HolidayPottedPlant" : ITEM_BASE, enc;
    construct: Some(|w| {
        let g = utility::random_min_max(&mut w.rng, 0x11C8, 0x11CC);
        holiday_potted_plant_new_with(w, g)
    }),
    force_show_properties: |w, _| w.core.opl_enabled,
}

struct PlantPick;

impl super::vet::Pick for PlantPick {
    const NAME: &'static str = "Server.Items.PottedPlantDeed+HolidayPottedPlantGump";
    const POS: (i32, i32) = (100, 200);
    const CHECK_PACK: bool = true;

    fn layout(b: &mut muo_content::gumps::builder::StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 360, 195, 0xA28);
        b.add_page(1);
        b.add_label(45, 15, 0, "Choose a Potted Plant:");
        for (i, (x, bx)) in [(45, 55), (100, 115), (160, 175), (225, 235), (280, 295)]
            .into_iter()
            .enumerate()
        {
            b.add_item(x, 75, 0x11C8 + i as i32, 0);
            b.add_button(bx, 50, 0x845, 0x846, i as i32 + 1);
        }
    }

    fn valid(button: i32) -> bool {
        (1..=5).contains(&button)
    }

    fn make(w: &mut World, button: i32, _deed: ItemId) -> ItemId {
        holiday_potted_plant_new_with(w, 0x11C8 + button - 1)
    }
}

leaf! {
    /// `Server.Items.PottedPlantDeed`.
    POTTED_PLANT_DEED = "Server.Items.PottedPlantDeed" : ITEM_BASE, enc;
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &POTTED_PLANT_DEED, 0x14F0);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    label_number: crate::class_props::label_number, // A deed for a potted plant.
    default_weight: crate::class_props::default_weight,
    on_double_click: |w, id, from| {
        if super::util::in_pack(w, id, from) {
            super::vet::show_pick::<PlantPick>(w, from, id);
        } else {
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
        }
    },
}

pub fn register_globes(types: &mut TypeRegistry) {
    for t in [
        &SNOW_GLOBE,
        &SNOW_GLOBE_ONE,
        &SNOW_GLOBE_TWO,
        &SNOW_GLOBE_THREE,
        &HOLIDAY_POTTED_PLANT,
        &POTTED_PLANT_DEED,
    ] {
        types.register_item(t, &[]);
    }
}

// ---- PKHolidayStuff (Coal is generated in items.rs), SnowStatue ----

leaf! {
    /// `Server.Items.BadCard`.
    BAD_CARD = "Server.Items.BadCard" : ITEM_BASE, v0;
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &BAD_CARD, 0x14ef);
        let hues = [0x45, 0x27, 0x3d0];
        let h = utility::random_element(&mut w.rng, &hues).copied().unwrap_or(0x45);
        super::hue(w, id, h);
        ops::set_stackable(w, id, false);
        ops::set_loot_type(w, id, LootType::Blessed);
        ops::set_movable(w, id, true);
        id
    }),
    label_number: crate::class_props::label_number, // Maybe next year you will get a nicer gift.
}

leaf! {
    /// `Server.Items.Spam`.
    SPAM = "Server.Items.Spam" : FOOD, v0;
    construct: Some(|w| {
        let id = new_food(w, &SPAM, 0x1044, 1);
        ops::set_stackable(w, id, false);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
}

macro_rules! snow_statue {
    ($($(#[$m:meta])* $name:ident = $full:literal, $graphic:expr;)*) => {$(
        leaf! {
            $(#[$m])*
            $name = $full : ITEM_BASE, enc;
            construct: Some(|w| ops::new_item_with_id(w, &$name, $graphic)),
            force_show_properties: |w, _| w.core.opl_enabled,
        }
    )*};
}

snow_statue! {
    /// `Server.Items.SnowStatuePegasus` (`[Flippable(0x456E, 0x456F)]`).
    SNOW_STATUE_PEGASUS = "Server.Items.SnowStatuePegasus", 0x456E;
    /// `Server.Items.SnowStatueSeahorse` (`[Flippable(0x4578, 0x4579)]`).
    SNOW_STATUE_SEAHORSE = "Server.Items.SnowStatueSeahorse", 0x4578;
    /// `Server.Items.SnowStatueMermaid` (`[Flippable(0x457A, 0x457B)]`).
    SNOW_STATUE_MERMAID = "Server.Items.SnowStatueMermaid", 0x457A;
    /// `Server.Items.SnowStatueGriffon` (`[Flippable(0x457C, 0x457D)]`).
    SNOW_STATUE_GRIFFON = "Server.Items.SnowStatueGriffon", 0x457C;
}

struct SnowStatuePick;

impl super::vet::Pick for SnowStatuePick {
    const NAME: &'static str = "Server.Items.SnowStatueDeed+SnowStatueGump";
    const POS: (i32, i32) = (100, 200);
    const CHECK_PACK: bool = true;

    fn layout(b: &mut muo_content::gumps::builder::StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 360, 225, 0xA28);
        b.add_page(1);
        b.add_html_localized(45, 15, 200, 20, 1156487, false, false); // Select One:
        for (i, (x, g, bx)) in [
            (35, 0x456E, 65),
            (120, 0x4578, 135),
            (190, 0x457A, 205),
            (250, 0x457C, 275),
        ]
        .into_iter()
        .enumerate()
        {
            b.add_item(x, 75, g, 0);
            b.add_button(bx, 50, 0x845, 0x846, i as i32 + 1);
        }
    }

    fn valid(button: i32) -> bool {
        (1..=4).contains(&button)
    }

    fn make(w: &mut World, button: i32, _deed: ItemId) -> ItemId {
        let ty = match button {
            1 => &SNOW_STATUE_PEGASUS,
            2 => &SNOW_STATUE_SEAHORSE,
            3 => &SNOW_STATUE_MERMAID,
            _ => &SNOW_STATUE_GRIFFON,
        };
        (ty.construct.unwrap_or(|_| unreachable!()))(w)
    }
}

leaf! {
    /// `Server.Items.SnowStatueDeed`.
    SNOW_STATUE_DEED = "Server.Items.SnowStatueDeed" : ITEM_BASE, enc;
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &SNOW_STATUE_DEED, 0x14F0);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    label_number: crate::class_props::label_number, // snow statue deed
    default_weight: crate::class_props::default_weight,
    on_double_click: |w, id, from| {
        if super::util::in_pack(w, id, from) {
            super::vet::show_pick::<SnowStatuePick>(w, from, id);
        } else {
            msg(w, from, 1042038); // You must have the object in your backpack to use it.
        }
    },
}

pub fn register_more(types: &mut TypeRegistry) {
    for t in [
        &BAD_CARD,
        &SPAM,
        &SNOW_STATUE_PEGASUS,
        &SNOW_STATUE_SEAHORSE,
        &SNOW_STATUE_MERMAID,
        &SNOW_STATUE_GRIFFON,
        &SNOW_STATUE_DEED,
    ] {
        types.register_item(t, &[]);
    }
}
