//! Cooking: the raw foods, the bowls of food, the doughs, the flour.
//!
//! `IT2ACombinable` and `T2ACooking` are at the end: [`t2a_prompt`], [`try_combine`].

use muo_content::items::food::{BASE_FOOD_CLASS, FOOD, FoodClass, new_food, set_fill_factor};
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::mobile::part2::messages::send_ascii_message;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::simple::{hue, random};

fn make(w: &mut World, name: &str) -> Option<ItemId> {
    muo_content::factory::make_item(w, name)
}

// ---- bowls ----

fn bowl_eat(w: &mut World, id: ItemId, from: MobileId, empty: &str) -> bool {
    if !(BASE_FOOD_CLASS.eat)(w, id, from) {
        return false;
    }
    if let Some(b) = make(w, empty) {
        muo_server::mobile::part2::items::add_to_backpack(w, from, b);
    }
    true
}

macro_rules! bowls {
    ($($name:ident, $class:ident = $full:literal, $gid:expr, $weight:expr, $empty:literal;)*) => {$(
        static $class: FoodClass = FoodClass { eat: |w, id, from| bowl_eat(w, id, from, $empty) };
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&FOOD),
            construct: Some(|w| {
                let id = new_food(w, &$name, $gid, 1);
                ops::set_stackable(w, id, false);
                set_fill_factor(w, id, 2);
                id
            }),
            family: Some(&$class),
            default_weight: |_, _| $weight,
            serialize: |w, id, out| {
                (FOOD.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (FOOD.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..FOOD
        };
    )*};
}

bowls! {
    WOODEN_BOWL_OF_CARROTS, C1 = "Server.Items.WoodenBowlOfCarrots", 0x15F9, 1.0, "Server.Items.EmptyWoodenBowl";
    WOODEN_BOWL_OF_CORN, C2 = "Server.Items.WoodenBowlOfCorn", 0x15FA, 1.0, "Server.Items.EmptyWoodenBowl";
    WOODEN_BOWL_OF_LETTUCE, C3 = "Server.Items.WoodenBowlOfLettuce", 0x15FB, 1.0, "Server.Items.EmptyWoodenBowl";
    WOODEN_BOWL_OF_PEAS, C4 = "Server.Items.WoodenBowlOfPeas", 0x15FC, 1.0, "Server.Items.EmptyWoodenBowl";
    PEWTER_BOWL_OF_CARROTS, C5 = "Server.Items.PewterBowlOfCarrots", 0x15FE, 1.0, "Server.Items.EmptyPewterBowl";
    PEWTER_BOWL_OF_CORN, C6 = "Server.Items.PewterBowlOfCorn", 0x15FF, 1.0, "Server.Items.EmptyPewterBowl";
    PEWTER_BOWL_OF_LETTUCE, C7 = "Server.Items.PewterBowlOfLettuce", 0x1600, 1.0, "Server.Items.EmptyPewterBowl";
    PEWTER_BOWL_OF_PEAS, C8 = "Server.Items.PewterBowlOfPeas", 0x1601, 1.0, "Server.Items.EmptyPewterBowl";
    PEWTER_BOWL_OF_POTATOS, C9 = "Server.Items.PewterBowlOfPotatos", 0x1602, 1.0, "Server.Items.EmptyPewterBowl";
    WOODEN_BOWL_OF_STEW, C10 = "Server.Items.WoodenBowlOfStew", 0x1604, 2.0, "Server.Items.EmptyWoodenTub";
    WOODEN_BOWL_OF_TOMATO_SOUP, C11 = "Server.Items.WoodenBowlOfTomatoSoup", 0x1606, 2.0, "Server.Items.EmptyWoodenTub";
}

// ---- CookableFood ----

/// `CookableFood._cookingLevel`.
#[derive(Clone, Copy, Debug, Default)]
pub struct CookData {
    pub level: i32,
}

/// `Cook()`: the class of the cooked food.
pub struct CookClass {
    pub cooked: &'static str,
}

/// `cookableFood.CookingLevel`.
pub fn cooking_level(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<CookData>())
        .map_or(0, |d| d.level)
}

fn new_cookable(w: &mut World, ty: &'static ItemType, gid: i32, level: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, gid);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(CookData { level });
    }
    id
}

/// `Server.Items.CookableFood` (abstract).
pub static COOKABLE_FOOD: ItemType = ItemType {
    full_name: "Server.Items.CookableFood",
    construct: None,
    parent: Some(&ITEM_BASE),
    on_double_click: t2a_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_i32(cooking_level(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let level = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(CookData { level });
        }
        Ok(())
    },
    ..ITEM_BASE
};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
static AMOUNT_1: [Param; 1] = [Param {
    name: "amount",
    kind: INT,
    default: Some(Value::Int(1)),
    ty: "",
}];

macro_rules! cookables {
    ($($name:ident, $class:ident, $ctor:ident = $full:literal, $gid:expr, $level:expr, $cooked:literal,
        ($($body:tt)*), [$($field:ident : $v:expr),*];)*) => {$(
        static $class: CookClass = CookClass { cooked: $cooked };
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&COOKABLE_FOOD),
            construct: Some(|w| {
                let id = new_cookable(w, &$name, $gid, $level);
                cookables!(@body w id $($body)*);
                id
            }),
            family: Some(&$class),
            $($field: $v,)*
            serialize: |w, id, out| {
                (COOKABLE_FOOD.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (COOKABLE_FOOD.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..COOKABLE_FOOD
        };
        cookables!(@ctor $name, $ctor, $gid, $level, $($body)*);
    )*};
    (@body $w:ident $id:ident none) => {};
    (@body $w:ident $id:ident stack) => { ops::set_stackable($w, $id, true); };
    (@body $w:ident $id:ident amount) => { ops::set_stackable($w, $id, true); };
    (@body $w:ident $id:ident eggs) => { { let h = 3 + random($w, 20) * 5; hue($w, $id, h); } };
    (@ctor $name:ident, $ctor:ident, $gid:expr, $level:expr, amount) => {
        static $ctor: [Ctor; 1] = [Ctor {
            access: None,
            params: &AMOUNT_1,
            make: |w, v| {
                let id = new_cookable(w, &$name, $gid, $level);
                ops::set_stackable(w, id, true);
                ops::set_amount(w, id, v[0].i32());
                EntityId::Item(id)
            },
        }];
    };
    (@ctor $name:ident, $ctor:ident, $gid:expr, $level:expr, $($other:tt)*) => {};
}

cookables! {
    RAW_RIBS, RAW_RIBS_C, C_RAW_RIBS = "Server.Items.RawRibs", 0x9F1, 10, "Server.Items.Ribs", (amount),
        [default_weight: |_, _| 1.0];
    RAW_LAMB_LEG, RAW_LAMB_LEG_C, C_RAW_LAMB_LEG = "Server.Items.RawLambLeg", 0x1609, 10, "Server.Items.LambLeg", (amount), [];
    RAW_CHICKEN_LEG, RAW_CHICKEN_LEG_C, C_RAW_CHICKEN_LEG = "Server.Items.RawChickenLeg", 0x1607, 10,
        "Server.Items.ChickenLeg", (stack), [default_weight: |_, _| 1.0];
    RAW_BIRD, RAW_BIRD_C, C_RAW_BIRD = "Server.Items.RawBird", 0x9B9, 10, "Server.Items.CookedBird", (amount),
        [default_weight: |_, _| 1.0];
    UNBAKED_PEACH_COBBLER, UPC_C, C_UPC = "Server.Items.UnbakedPeachCobbler", 0x1042, 25, "Server.Items.PeachCobbler", (none),
        [default_weight: |_, _| 1.0, label_number: |_, _| 1041335];
    UNBAKED_FRUIT_PIE, UFP_C, C_UFP = "Server.Items.UnbakedFruitPie", 0x1042, 25, "Server.Items.FruitPie", (none),
        [default_weight: |_, _| 1.0, label_number: |_, _| 1041334];
    UNBAKED_MEAT_PIE, UMP_C, C_UMP = "Server.Items.UnbakedMeatPie", 0x1042, 25, "Server.Items.MeatPie", (none),
        [default_weight: |_, _| 1.0, label_number: |_, _| 1041338];
    UNBAKED_PUMPKIN_PIE, UPP_C, C_UPP = "Server.Items.UnbakedPumpkinPie", 0x1042, 25, "Server.Items.PumpkinPie", (none),
        [default_weight: |_, _| 1.0, label_number: |_, _| 1041342];
    UNBAKED_APPLE_PIE, UAP_C, C_UAP = "Server.Items.UnbakedApplePie", 0x1042, 25, "Server.Items.ApplePie", (none),
        [default_weight: |_, _| 1.0, label_number: |_, _| 1041336];
    UNCOOKED_CHEESE_PIZZA, UCP_C, C_UCP = "Server.Items.UncookedCheesePizza", 0x1083, 20, "Server.Items.CheesePizza", (none),
        [default_weight: |_, _| 1.0, label_number: |_, _| 1041341];
    UNCOOKED_SAUSAGE_PIZZA, USP_C, C_USP = "Server.Items.UncookedSausagePizza", 0x1083, 20, "Server.Items.SausagePizza", (none),
        [default_weight: |_, _| 1.0, label_number: |_, _| 1041337];
    UNBAKED_QUICHE, UQ_C, C_UQ = "Server.Items.UnbakedQuiche", 0x1042, 25, "Server.Items.Quiche", (none),
        [default_weight: |_, _| 1.0, label_number: |_, _| 1041339];
    EGGS, EGGS_C, C_EGGS = "Server.Items.Eggs", 0x9B5, 15, "Server.Items.FriedEggs", (amount),
        [default_weight: |_, _| 1.0];
    BRIGHTLY_COLORED_EGGS, BCE_C, C_BCE = "Server.Items.BrightlyColoredEggs", 0x9B5, 15, "Server.Items.FriedEggs", (eggs),
        [default_weight: |_, _| 0.5, default_name: |_, _| Some("brightly colored eggs".to_string())];
    EASTER_EGGS, EE_C, C_EE = "Server.Items.EasterEggs", 0x9B5, 15, "Server.Items.FriedEggs", (eggs),
        [default_weight: |_, _| 0.5, label_number: |_, _| 1016105];
    COOKIE_MIX, CM_C, C_CM = "Server.Items.CookieMix", 0x103F, 20, "Server.Items.Cookies", (none),
        [default_weight: |_, _| 1.0];
    CAKE_MIX, CAKEM_C, C_CAKEM = "Server.Items.CakeMix", 0x103F, 40, "Server.Items.Cake", (none),
        [default_weight: |_, _| 1.0, label_number: |_, _| 1041002];
    RAW_FISH_STEAK, RFS_C, C_RFS = "Server.Items.RawFishSteak", 0x097A, 10, "Server.Items.FishSteak", (amount),
        [default_weight: |_, _| 0.1];
}

// ---- Dough, SweetDough, JarHoney, BowlFlour ----

macro_rules! plain {
    ($($name:ident = $full:literal, $gid:expr, $weight:expr, [$($f:ident : $v:expr),*], |$w:ident, $id:ident| $init:block;)*) => {$(
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: Some(|$w| {
                let $id = ops::new_item_with_id($w, &$name, $gid);
                $init
                $id
            }),
            default_weight: |_, _| $weight,
            on_double_click: t2a_double_click,
            $($f: $v,)*
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
    )*};
}

fn stackable_ml(w: &mut World, id: ItemId) {
    let ml = muo_content::core::ml(w);
    ops::set_stackable(w, id, ml);
}

plain! {
    DOUGH = "Server.Items.Dough", 0x103D, 1.0, [], |w, id| { stackable_ml(w, id); };
    SWEET_DOUGH = "Server.Items.SweetDough", 0x103D, 1.0, [label_number: |_, _| 1041340], |w, id| {
        stackable_ml(w, id);
        hue(w, id, 150);
    };
    JAR_HONEY = "Server.Items.JarHoney", 0x9EC, 1.0, [], |w, id| { ops::set_stackable(w, id, true); };
    BOWL_FLOUR = "Server.Items.BowlFlour", 0xA1E, 1.0, [], |_w, _id| {};
}

// ---- SackFlour ----

/// `SackFlour._quantity`.
#[derive(Clone, Copy, Debug, Default)]
pub struct FlourData {
    pub quantity: i32,
}

/// `SackFlour.Quantity`.
pub fn flour_quantity(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<FlourData>())
        .map_or(0, |d| d.quantity)
}

/// `SackFlour.Quantity = value`: clamped to 0..=20; an empty sack is deleted, a part-used closed one opens a step.
pub fn set_flour_quantity(w: &mut World, id: ItemId, value: i32) {
    let q = value.clamp(0, 20);
    if q == flour_quantity(w, id) {
        return;
    }
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<FlourData>()) {
        d.quantity = q;
    }
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    if q == 0 {
        (ty.delete)(w, id);
    } else if q < 20 {
        let gid = w.item(id).map_or(0, |i| i.item_id);
        if gid == 0x1039 || gid == 0x1045 {
            (ty.set_item_id)(w, id, gid + 1);
        }
    }
}

/// `SackFlour.IsOpen`.
pub fn is_open(w: &World, id: ItemId) -> bool {
    !matches!(w.item(id).map(|i| i.item_id), Some(0x1039 | 0x1045))
}

/// `Server.Items.SackFlour`.
pub static SACK_FLOUR: ItemType = ItemType {
    full_name: "Server.Items.SackFlour",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &SACK_FLOUR, 0x1039);
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(FlourData { quantity: 20 });
        }
        id
    }),
    default_weight: crate::class_props::default_weight,
    on_double_click: |w, id, from| {
        let movable = w.item(id).is_some_and(|i| i.movable());
        let gid = w.item(id).map_or(0, |i| i.item_id);
        if movable
            && (gid == 0x1039 || gid == 0x1045)
            && let Some(ty) = w.item(id).map(|i| i.ty)
        {
            (ty.set_item_id)(w, id, gid + 1);
        }
        if !t2a_enabled(w) || !validate_in_pack(w, from, id) {
            return;
        }
        if is_open(w, id) {
            t2a_prompt(w, from, id);
        } else {
            send_ascii_message(w, from, "You must open the sack of flour first.");
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_i32(flour_quantity(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let quantity = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(FlourData { quantity });
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- WheatSheaf ----

fn wheat_on_target(w: &mut World, id: ItemId, _from: MobileId, t: Targeted) {
    use crate::addons::devices::{
        MILL_MAX_FLOUR, flour_mill_cur_flour, flour_mill_set_cur_flour, is_flour_mill,
    };
    let Targeted::Entity(EntityId::Item(mut obj)) = t else {
        return;
    };
    // `obj is AddonComponent addon` -> `addon.Addon`
    if let Some(a) = muo_content::items::addon::component_addon(w, obj) {
        obj = a;
    }
    if !is_flour_mill(w, obj) {
        return;
    }
    let cur = flour_mill_cur_flour(w, obj);
    let amount = w.item(id).map_or(0, |i| i.amount);
    let needs = (MILL_MAX_FLOUR - cur).min(amount);
    flour_mill_set_cur_flour(w, obj, cur + needs);
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.consume)(w, id, needs);
    }
}

/// `Server.Items.WheatSheaf`.
pub static WHEAT_SHEAF: ItemType = ItemType {
    full_name: "Server.Items.WheatSheaf",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_wheat_sheaf(w, 1)),
    default_weight: crate::class_props::default_weight,
    on_double_click: |w, id, from| {
        if w.item(id).is_some_and(|i| i.movable()) {
            target::begin_target(w, from, 4, false, TargetFlags::NONE, move |w, from, t| {
                wheat_on_target(w, id, from, t)
            });
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

fn new_wheat_sheaf(w: &mut World, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, &WHEAT_SHEAF, "Server.Items.WheatSheaf");
    ops::set_amount(w, id, amount);
    id
}

static C_WHEAT_SHEAF: [Ctor; 1] = [Ctor {
    access: None,
    params: &AMOUNT_1,
    make: |w, v| EntityId::Item(new_wheat_sheaf(w, v[0].i32())),
}];

fn configure(w: &mut World) {
    let tables: [(&str, &'static [Ctor]); 6] = [
        ("Server.Items.RawRibs", &C_RAW_RIBS),
        ("Server.Items.RawLambLeg", &C_RAW_LAMB_LEG),
        ("Server.Items.RawBird", &C_RAW_BIRD),
        ("Server.Items.Eggs", &C_EGGS),
        ("Server.Items.RawFishSteak", &C_RFS),
        ("Server.Items.WheatSheaf", &C_WHEAT_SHEAF),
    ];
    for (name, t) in tables {
        add_ctors(w, name, t);
    }
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &WOODEN_BOWL_OF_CARROTS,
        &WOODEN_BOWL_OF_CORN,
        &WOODEN_BOWL_OF_LETTUCE,
        &WOODEN_BOWL_OF_PEAS,
        &PEWTER_BOWL_OF_CARROTS,
        &PEWTER_BOWL_OF_CORN,
        &PEWTER_BOWL_OF_LETTUCE,
        &PEWTER_BOWL_OF_PEAS,
        &PEWTER_BOWL_OF_POTATOS,
        &WOODEN_BOWL_OF_STEW,
        &WOODEN_BOWL_OF_TOMATO_SOUP,
        &COOKABLE_FOOD,
        &RAW_RIBS,
        &RAW_LAMB_LEG,
        &RAW_CHICKEN_LEG,
        &RAW_BIRD,
        &UNBAKED_PEACH_COBBLER,
        &UNBAKED_FRUIT_PIE,
        &UNBAKED_MEAT_PIE,
        &UNBAKED_PUMPKIN_PIE,
        &UNBAKED_APPLE_PIE,
        &UNCOOKED_SAUSAGE_PIZZA,
        &UNBAKED_QUICHE,
        &EGGS,
        &BRIGHTLY_COLORED_EGGS,
        &EASTER_EGGS,
        &COOKIE_MIX,
        &CAKE_MIX,
        &RAW_FISH_STEAK,
        &DOUGH,
        &SWEET_DOUGH,
        &JAR_HONEY,
        &BOWL_FLOUR,
        &WHEAT_SHEAF,
    ] {
        types.register_item(t, &[]);
    }
    types.register_item(&UNCOOKED_CHEESE_PIZZA, &["Server.Items.UncookedPizza"]);
    types.register_item(&SACK_FLOUR, &["Server.Items.SackFlourOpen"]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.CookableFood.Configure",
        configure,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::GenericWriter;
    use muo_io::{SaveReader, SaveWriter};

    fn save(w: &World, id: ItemId) -> Vec<u8> {
        let mut out = SaveWriter::new(true);
        (w.item(id).unwrap().ty.serialize)(w, id, &mut out);
        out.into_vec()
    }

    #[test]
    fn cookable_food_saves_version_and_cooking_level() {
        let mut w = World::new();
        let id = (CAKE_MIX.construct.unwrap())(&mut w);
        assert_eq!(cooking_level(&w, id), 40);
        let bytes = save(&w, id);
        let mut want = SaveWriter::new(true);
        (ITEM_BASE.serialize)(&w, id, &mut want);
        want.write_i32(0);
        want.write_i32(40);
        want.write_i32(0);
        assert_eq!(bytes, want.into_vec());
        let id2 = w.add_item(muo_server::item::Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0100),
            &CAKE_MIX,
        ));
        let mut r = SaveReader::new(&bytes);
        (CAKE_MIX.deserialize)(&mut w, id2, &mut r).unwrap();
        assert_eq!((r.remaining(), cooking_level(&w, id2)), (0, 40));
    }

    #[test]
    fn eggs_pick_a_hue_in_the_easter_range() {
        let mut w = World::new();
        for _ in 0..20 {
            let id = (EASTER_EGGS.construct.unwrap())(&mut w);
            let h = w.item(id).unwrap().hue;
            assert!((3..=98).contains(&h) && (h - 3) % 5 == 0, "{h}");
        }
    }

    #[test]
    fn heat_sources_are_the_ranges_of_cookable_food() {
        use muo_server::target::StaticTarget;
        let w = World::new();
        let at = |item_id| {
            Targeted::Static(StaticTarget {
                location: Default::default(),
                item_id,
                hue: 0,
            })
        };
        for (id, hot) in [
            (0xDE3, true),
            (0xDE9, true),
            (0xDEA, false),
            (0x48E, true),
            (0x96C, true),
            (0xFAC, true),
            (0x184D, false),
            (0x184E, true),
            (0x399F, true),
            (0x39A0, false),
        ] {
            assert_eq!(is_heat_source(&w, at(id)), hot, "{id:#X}");
        }
    }

    #[test]
    fn sack_of_flour_opens_when_used_and_is_deleted_when_empty() {
        let mut w = World::new();
        let id = (SACK_FLOUR.construct.unwrap())(&mut w);
        assert!(!is_open(&w, id));
        set_flour_quantity(&mut w, id, 19);
        assert!(is_open(&w, id), "a used sack opens a step");
        set_flour_quantity(&mut w, id, 99);
        assert_eq!(flour_quantity(&w, id), 20);
        set_flour_quantity(&mut w, id, 0);
        assert!(w.item(id).is_none_or(|i| i.deleted()));
    }
}

// ---- T2ACooking ----

const HEAT_SOURCE_RANGE: i32 = 1;
const WATER_RANGE: i32 = 2;
const BURN_RANGE: i32 = 3;
/// `BeginAction<T2ACooking>()`.
const T2A_COOKING: &str = "Server.Engines.Craft.T2A.T2ACooking";

type CraftRangeFn = fn(&mut World, &str) -> Option<(f64, f64)>;

/// What cooking asks of the craft engine: the skill range of a craft item of `DefCooking` (`CraftRange`).
#[derive(Default)]
pub struct CookingHooks {
    pub craft_range: Option<CraftRangeFn>,
}

fn t2a_enabled(w: &World) -> bool {
    w.resources
        .get::<muo_content::configuration::expansion_configuration::T2aCraftSystemEnabled>()
        .is_some_and(|e| e.0)
}

/// `OnDoubleClick` of every `IT2ACombinable`: `T2ACraftSystem.Enabled ? T2ACooking.Prompt(from, this) : base`.
fn t2a_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if t2a_enabled(w) {
        t2a_prompt(w, from, id);
    } else {
        (ITEM_BASE.on_double_click)(w, id, from);
    }
}

/// `T2ACooking.ValidateInPack(from, item)`.
pub fn validate_in_pack(w: &mut World, from: MobileId, item: ItemId) -> bool {
    let pack = muo_server::mobile::part1::props_b::backpack(w, from).map(EntityId::Item);
    if w.item(item).is_some_and(|i| !i.deleted()) && ops::is_child_of(w, item, pack) {
        return true;
    }
    muo_server::item::deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
    false
}

/// `T2ACooking.Prompt(from, source)`.
pub fn t2a_prompt(w: &mut World, from: MobileId, source: ItemId) {
    if !validate_in_pack(w, from, source) {
        return;
    }
    send_ascii_message(
        w,
        from,
        "What do you wish to combine this with, or cook it over?",
    );
    target::begin_target(
        w,
        from,
        WATER_RANGE,
        false,
        TargetFlags::NONE,
        move |w, from, t| {
            if validate_in_pack(w, from, source) {
                try_combine(w, source, from, t);
            }
        },
    );
}

/// `T2ACooking.CannotCombine(from)`.
fn cannot_combine(w: &mut World, from: MobileId) -> bool {
    send_ascii_message(w, from, "You cannot combine those.");
    false
}

fn is(w: &World, item: ItemId, name: &str) -> bool {
    w.item(item)
        .is_some_and(|i| muo_content::items::is_named(i.ty, name))
}

/// `CookableFood.IsHeatSource(targeted)`.
pub fn is_heat_source(w: &World, t: Targeted) -> bool {
    let id = match t {
        Targeted::Entity(EntityId::Item(i)) => w.item(i).map(|x| x.item_id),
        Targeted::Static(s) => Some(s.item_id),
        _ => None,
    };
    id.is_some_and(|id| {
        (0xDE3..=0xDE9).contains(&id) // Campfire
            || (0x461..=0x48E).contains(&id) // Sandstone oven/fireplace
            || (0x92B..=0x96C).contains(&id) // Stone oven/fireplace
            || id == 0xFAC // Firepit
            || (0x184A..=0x184C).contains(&id) // Heating stand (left)
            || (0x184E..=0x1850).contains(&id) // Heating stand (right)
            || (0x398C..=0x399F).contains(&id) // Fire field
    })
}

/// A water source of `GetWaterSource`.
#[derive(Copy, Clone)]
enum Water {
    Beverage(ItemId),
    Container(ItemId),
    Source(ItemId),
}

/// `T2ACooking.GetWaterSource(o)`.
fn water_source(w: &World, item: ItemId) -> Option<Water> {
    use super::beverage::{self, BeverageType};
    if beverage::data(w, item).is_some() {
        return (beverage::content(w, item) == BeverageType::Water
            && beverage::quantity(w, item) > 0)
            .then_some(Water::Beverage(item));
    }
    if w.item(item)?
        .ty
        .family::<crate::special::water::WaterClass>()
        .is_some()
    {
        return (!crate::special::water::is_empty(w, item)).then_some(Water::Container(item));
    }
    let f = w.resources.get::<beverage::BeverageHooks>()?.water_source?;
    f(w, item).filter(|&q| q > 0).map(|_| Water::Source(item))
}

/// `ConsumeOne` of an `IHasQuantity`: `Quantity--`.
fn consume_water(w: &mut World, water: Water) {
    use super::beverage;
    match water {
        Water::Beverage(i) => {
            let q = beverage::quantity(w, i);
            beverage::set_quantity(w, i, q - 1);
        }
        Water::Container(i) => {
            let q = crate::special::water::quantity(w, i);
            crate::special::water::set_quantity(w, i, q - 1);
        }
        Water::Source(i) => {
            let hooks = w.resources.get::<beverage::BeverageHooks>();
            let q = hooks
                .and_then(|h| h.water_source)
                .and_then(|f| f(w, i))
                .unwrap_or(0);
            if let Some(f) = hooks.and_then(|h| h.water_source_set) {
                f(w, i, q - 1);
            }
        }
    }
}

/// `T2ACooking.CanReachWater(from, water)`: ground water only.
fn can_reach_water(w: &mut World, from: MobileId, water: ItemId) -> bool {
    let pack = muo_server::mobile::part1::props_b::backpack(w, from).map(EntityId::Item);
    let (parent, map) = w.item(water).map_or((None, None), |i| (i.parent, i.map));
    let reach = ops::is_child_of(w, water, pack)
        || parent == Some(EntityId::Mobile(from))
        || (ops::root_parent(w, water).is_none()
            && map == w.mobile(from).and_then(|m| m.map)
            && muo_server::item::deps::mobile_in_range(
                w,
                from,
                ops::get_world_location(w, water),
                WATER_RANGE,
            )
            && muo_server::item::deps::in_los_item(w, from, water));
    if !reach {
        muo_server::mobile::part2::messages::local_overhead_message_number(
            w,
            from,
            muo_primitives::MessageType::REGULAR,
            0x3B2,
            1019045,
            "",
        ); // I can't reach that.
    }
    reach
}

/// What a `Combine` spends: an item (`Consume()`) or a quantity (`Quantity--`).
enum Ingredient {
    Item(ItemId),
    Water(Water),
    Flour(ItemId),
}

fn consume_one(w: &mut World, i: &Ingredient) {
    match *i {
        Ingredient::Item(i) => muo_server::item::base::consume(w, i, 1),
        Ingredient::Water(x) => consume_water(w, x),
        Ingredient::Flour(i) => {
            let q = flour_quantity(w, i);
            set_flour_quantity(w, i, q - 1);
        }
    }
}

/// `GiveResult`: into the pack, or left where `AddToBackpack` dropped it; deleted with no valid map.
fn give_result(w: &mut World, from: MobileId, result: ItemId) -> bool {
    let added = muo_server::mobile::part2::items::add_to_backpack(w, from, result);
    let on_map = w
        .item(result)
        .and_then(|i| i.map)
        .is_some_and(|m| m != muo_server::map::MapId::INTERNAL);
    if added || on_map {
        return true;
    }
    muo_server::item::base::delete(w, result);
    false
}

/// `T2ACooking.Combine(from, source, target, result, message)` (the result is made first, as an argument).
fn combine(
    w: &mut World,
    from: MobileId,
    source: Ingredient,
    target: Ingredient,
    result: &str,
    message: &str,
) -> bool {
    let Some(result) = make(w, result) else {
        return false;
    };
    consume_one(w, &source);
    consume_one(w, &target);
    if !give_result(w, from, result) {
        return false;
    }
    send_ascii_message(w, from, message);
    true
}

fn flour_water(w: &mut World, from: MobileId, source: Ingredient, item: ItemId) -> Option<bool> {
    let water = water_source(w, item)?;
    Some(
        can_reach_water(w, from, item)
            && combine(
                w,
                from,
                source,
                Ingredient::Water(water),
                "Server.Items.Dough",
                "You mix the flour with water to make dough.",
            ),
    )
}

const CAKE_MSG: &str = "You mix the flour with sweet dough to make cake mix.";
const SWEET_MSG: &str = "You mix the dough with honey to make sweet dough.";
const COOKIE_MSG: &str = "You mix the sweet dough with honey to make cookie mix.";

/// `IT2ACombinable.TryCombine(from, target)` of `source`.
pub fn try_combine(w: &mut World, source: ItemId, from: MobileId, t: Targeted) -> bool {
    let target_item = match t {
        Targeted::Entity(EntityId::Item(i)) => Some(i),
        _ => None,
    };
    if is(w, source, "Server.Items.CookableFood") {
        return if is_heat_source(w, t) {
            begin_heat(w, from, source, t)
        } else {
            cannot_combine(w, from)
        };
    }
    if is(w, source, "Server.Items.Dough") {
        if is_heat_source(w, t) {
            return begin_heat(w, from, source, t);
        }
        let Some(item) = target_item else {
            return cannot_combine(w, from);
        };
        if !validate_in_pack(w, from, item) {
            return false;
        }
        let result = [
            ("Server.Items.JarHoney", "Server.Items.SweetDough"),
            ("Server.Items.Pear", "Server.Items.UnbakedFruitPie"),
            ("Server.Items.Peach", "Server.Items.UnbakedPeachCobbler"),
            ("Server.Items.Apple", "Server.Items.UnbakedApplePie"),
            ("Server.Items.Pumpkin", "Server.Items.UnbakedPumpkinPie"),
            (
                "Server.Items.SmallPumpkin",
                "Server.Items.UnbakedPumpkinPie",
            ),
            ("Server.Items.Eggs", "Server.Items.UnbakedQuiche"),
            ("Server.Items.Ham", "Server.Items.UnbakedMeatPie"),
            ("Server.Items.Spam", "Server.Items.UnbakedMeatPie"),
            ("Server.Items.CookedBird", "Server.Items.UnbakedMeatPie"),
            ("Server.Items.FishSteak", "Server.Items.UnbakedMeatPie"),
            ("Server.Items.Sausage", "Server.Items.UncookedSausagePizza"),
        ]
        .into_iter()
        .find(|(c, _)| is(w, item, c))
        .map(|(_, r)| r);
        let Some(result) = result else {
            return cannot_combine(w, from);
        };
        let msg = if is(w, item, "Server.Items.JarHoney") {
            SWEET_MSG
        } else {
            "You combine the dough with the ingredient."
        };
        return combine(
            w,
            from,
            Ingredient::Item(source),
            Ingredient::Item(item),
            result,
            msg,
        );
    }
    if is(w, source, "Server.Items.SweetDough") {
        if is_heat_source(w, t) {
            return begin_heat(w, from, source, t);
        }
        let Some(item) = target_item else {
            return cannot_combine(w, from);
        };
        if is(w, item, "Server.Items.JarHoney") {
            return validate_in_pack(w, from, item)
                && combine(
                    w,
                    from,
                    Ingredient::Item(source),
                    Ingredient::Item(item),
                    "Server.Items.CookieMix",
                    COOKIE_MSG,
                );
        }
        if is(w, item, "Server.Items.SackFlour") {
            if !validate_in_pack(w, from, item) {
                return false;
            }
            if !is_open(w, item) {
                send_ascii_message(w, from, "You must open the sack of flour first.");
                return false;
            }
            return combine(
                w,
                from,
                Ingredient::Item(source),
                Ingredient::Flour(item),
                "Server.Items.CakeMix",
                CAKE_MSG,
            );
        }
        if is(w, item, "Server.Items.BowlFlour") {
            return validate_in_pack(w, from, item)
                && combine(
                    w,
                    from,
                    Ingredient::Item(source),
                    Ingredient::Item(item),
                    "Server.Items.CakeMix",
                    CAKE_MSG,
                );
        }
        return cannot_combine(w, from);
    }
    if is(w, source, "Server.Items.JarHoney") {
        let Some(item) = target_item else {
            return cannot_combine(w, from);
        };
        if is(w, item, "Server.Items.Dough") {
            return validate_in_pack(w, from, item)
                && combine(
                    w,
                    from,
                    Ingredient::Item(source),
                    Ingredient::Item(item),
                    "Server.Items.SweetDough",
                    SWEET_MSG,
                );
        }
        if is(w, item, "Server.Items.SweetDough") {
            return validate_in_pack(w, from, item)
                && combine(
                    w,
                    from,
                    Ingredient::Item(source),
                    Ingredient::Item(item),
                    "Server.Items.CookieMix",
                    COOKIE_MSG,
                );
        }
        return cannot_combine(w, from);
    }
    // SackFlour, BowlFlour
    let flour = if is(w, source, "Server.Items.SackFlour") {
        if !is_open(w, source) {
            send_ascii_message(w, from, "You must open the sack of flour first.");
            return false;
        }
        Ingredient::Flour(source)
    } else if is(w, source, "Server.Items.BowlFlour") {
        Ingredient::Item(source)
    } else {
        return false;
    };
    let Some(item) = target_item else {
        return cannot_combine(w, from);
    };
    if is(w, item, "Server.Items.SweetDough") {
        return validate_in_pack(w, from, item)
            && combine(
                w,
                from,
                flour,
                Ingredient::Item(item),
                "Server.Items.CakeMix",
                CAKE_MSG,
            );
    }
    flour_water(w, from, flour, item).unwrap_or_else(|| cannot_combine(w, from))
}

/// `T2ACooking.BeginHeat(from, source, target)`.
fn begin_heat(w: &mut World, from: MobileId, source: ItemId, t: Targeted) -> bool {
    let point = match t {
        Targeted::Entity(EntityId::Item(i)) => w.item(i).map(|x| x.location),
        Targeted::Entity(EntityId::Mobile(m)) => w.mobile(m).map(|x| x.location),
        Targeted::Land(l) => Some(l.location),
        Targeted::Static(s) => Some(s.location),
    };
    let point = point.unwrap_or_else(|| w.mobile(from).map(|m| m.location).unwrap_or_default());
    let map = w.mobile(from).and_then(|m| m.map);
    if map.is_none() || !muo_server::item::deps::mobile_in_range(w, from, point, HEAT_SOURCE_RANGE)
    {
        muo_server::item::deps::send_localized_message(w, from, 500446); // That is too far away.
        return false;
    }
    if !muo_server::mobile::part1::combat::begin_action(w, from, T2A_COOKING) {
        muo_server::item::deps::send_localized_message(w, from, 500119); // You must wait to perform another action
        return false;
    }
    muo_content::mx::play_sound(w, from, 0x225);

    // The source is consumed here and may be deleted, so only its type and `CookingLevel` are kept.
    let ty = w.item(source).map(|i| i.ty);
    let level = cooking_level(w, source);
    muo_server::item::base::consume(w, source, 1);

    muo_server::timer::start(
        w,
        muo_primitives::TimeSpan::from_secs(5),
        muo_primitives::TimeSpan::ZERO,
        1,
        move |w| {
            if let Some(ty) = ty {
                finish_heat(w, from, ty, level, map, point);
            }
        },
    );
    true
}

fn craft_range(w: &mut World, class: &str) -> (f64, f64) {
    let f = w
        .resources
        .get::<CookingHooks>()
        .and_then(|h| h.craft_range);
    f.and_then(|f| f(w, class)).unwrap_or((0.0, 100.0))
}

/// `T2ACooking.FinishHeat`.
fn finish_heat(
    w: &mut World,
    from: MobileId,
    ty: &'static ItemType,
    level: i32,
    map: Option<muo_server::map::MapId>,
    point: muo_primitives::Point3D,
) {
    use muo_server::skills::SkillName;
    muo_server::mobile::part1::combat::end_action(w, from, T2A_COOKING);
    if w.mobile(from).is_none_or(|m| m.deleted) {
        return;
    }
    let burn = |w: &mut World| muo_server::item::deps::send_localized_message(w, from, 500686); // You burn the food to a crisp! It's ruined.
    if w.mobile(from).and_then(|m| m.map) != map
        || !muo_server::item::deps::mobile_in_range(w, from, point, BURN_RANGE)
    {
        burn(w);
        return;
    }
    let named = |n| muo_content::items::is_named(ty, n);
    let (min, max) = if named("Server.Items.CookableFood") {
        (f64::from(level), 100.0)
    } else if named("Server.Items.Dough") {
        craft_range(w, "Server.Items.BreadLoaf")
    } else if named("Server.Items.SweetDough") {
        craft_range(w, "Server.Items.Muffins")
    } else {
        (0.0, 100.0)
    };
    if !muo_server::mobile::part2::stats::check_skill(w, from, SkillName::Cooking, min, max) {
        burn(w);
        return;
    }
    let result = if let Some(c) = ty.family::<CookClass>() {
        make(w, c.cooked)
    } else if named("Server.Items.Dough") {
        make(w, "Server.Items.BreadLoaf")
    } else if named("Server.Items.SweetDough") {
        make(w, "Server.Items.Muffins")
    } else {
        None
    };
    if let Some(r) = result
        && give_result(w, from, r)
    {
        muo_content::mx::play_sound(w, from, 0x57);
    }
}
