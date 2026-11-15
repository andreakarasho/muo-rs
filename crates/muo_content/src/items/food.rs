//! Food: the `Food` base and its leaves (fruits, vegetables and the Asian foods).
//!
//! State is [`FoodData`] (Poisoner, Poison, FillFactor); the leaves are one row of `food_leaves!` each
//! (`graphic`, `fill` = the constructor's `FillFactor`, `weight` = `DefaultWeight`, optional `label` and
//! `stack no|ml` = `Stackable = false` / `Stackable = Core.ML`). `[Flippable]` is data for the flip command.
//!
//! Beverages, empty beverages, bowls, chocolatiering, cookable food, cooking, the endless decanter, the empty
//! bento box and tea basket and `SheafOfHay` are `muo_items::food`.

use muo_primitives::utility;
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::poison::PoisonId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

use crate::core;

/// What `Food` adds to an `Item`.
#[derive(Clone, Debug)]
pub struct FoodData {
    pub poisoner: Option<MobileId>,
    pub poison: Option<PoisonId>,
    pub fill_factor: i32,
}

impl Default for FoodData {
    fn default() -> Self {
        FoodData {
            poisoner: None,
            poison: None,
            fill_factor: 1,
        }
    }
}

pub fn data(w: &World, id: ItemId) -> Option<&FoodData> {
    w.item(id)?.ext.get::<FoodData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut FoodData> {
    w.item_mut(id)?.ext.get_mut::<FoodData>()
}

/// `item is Food`.
pub fn is_food(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&FOOD))
}

pub fn fill_factor(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(1, |d| d.fill_factor)
}

pub fn set_fill_factor(w: &mut World, id: ItemId, v: i32) {
    if let Some(d) = data_mut(w, id) {
        d.fill_factor = v;
    }
}

/// `new Food(itemID, amount)` of class `ty`.
pub fn new_food(w: &mut World, ty: &'static ItemType, item_id: i32, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(FoodData::default());
    }
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Food.FillHunger(from, fillFactor)`.
pub fn fill_hunger(w: &mut World, from: MobileId, fill_factor: i32) -> bool {
    let Some(hunger) = w.mobile(from).map(|m| m.hunger) else {
        return false;
    };
    if hunger >= 20 {
        deps::send_localized_message(w, from, 500867); // You are simply too full to eat any more!
        return false;
    }
    let i_hunger = hunger + fill_factor;
    let stam = w.mobile(from).map_or(0, |m| m.stam);
    if stam < muo_server::mobile::base::stam_max(w, from) {
        let add = utility::random_from(&mut w.rng, 6, 3) + fill_factor / 5;
        muo_server::mobile::part1::props_b::set_stam(w, from, stam + add);
    }
    if i_hunger >= 20 {
        muo_server::mobile::part1::props_a::set_hunger(w, from, 20);
        deps::send_localized_message(w, from, 500872); // You manage to eat the food, but you are stuffed!
    } else {
        muo_server::mobile::part1::props_a::set_hunger(w, from, i_hunger);
        let msg = if i_hunger < 5 {
            500868 // You eat the food, but are still extremely hungry.
        } else if i_hunger < 10 {
            500869 // You eat the food, and begin to feel more satiated.
        } else if i_hunger < 15 {
            500870 // After eating the food, you feel much less hungry.
        } else {
            500871 // You feel quite full after consuming the food.
        };
        deps::send_localized_message(w, from, msg);
    }
    true
}

/// The `Eat` virtual of a food class.
#[derive(Copy, Clone)]
pub struct FoodClass {
    pub eat: fn(&mut World, ItemId, MobileId) -> bool,
}

fn eat_base(w: &mut World, id: ItemId, from: MobileId) -> bool {
    // CheckHunger(from) => FillHunger(from, FillFactor)
    let ff = fill_factor(w, id);
    if !fill_hunger(w, from, ff) {
        return false;
    }
    // Play a random "eat" sound
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
    if let Some(d) = data(w, id).cloned()
        && let Some(p) = d.poison
    {
        muo_server::mobile::base::apply_poison(w, from, d.poisoner, p);
    }
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.consume)(w, id, 1);
    }
    true
}

pub static BASE_FOOD_CLASS: FoodClass = FoodClass { eat: eat_base };

fn class(w: &World, id: ItemId) -> &'static FoodClass {
    w.item(id)
        .and_then(|i| i.ty.family::<FoodClass>())
        .unwrap_or(&BASE_FOOD_CLASS)
}

/// `Eat(from)` through the class.
pub fn eat(w: &mut World, id: ItemId, from: MobileId) -> bool {
    (class(w, id).eat)(w, id, from)
}

/// `Server.Items.Food` (abstract).
pub static FOOD: ItemType = ItemType {
    full_name: "Server.Items.Food",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_FOOD_CLASS),
    get_context_menu_entries: |w, id, from, list| {
        base::get_context_menu_entries_base(w, id, from, list);
        if w.mobile(from).is_some_and(|m| (m.ty.alive)(w, from)) {
            // EatEntry: 6135, range 1; click eats when the item is still there.
            let mut e = ContextMenuEntry::new(6135, 1);
            e.on_click = Some(std::rc::Rc::new(|w: &mut World, from, target| {
                if let EntityId::Item(f) = target
                    && !ops::is_deleted(w, f)
                    && is_food(w, f)
                {
                    let loc = ops::get_world_location(w, f);
                    if deps::mobile_in_range(w, from, loc, 1) {
                        eat(w, f, from);
                    }
                }
            }));
            list.push(e);
        }
    },
    on_double_click: |w, id, from| {
        if !w
            .item(id)
            .is_some_and(|i| i.flags.contains(muo_server::item::ImplFlag::MOVABLE))
        {
            return;
        }
        let loc = ops::get_world_location(w, id);
        if deps::mobile_in_range(w, from, loc, 1) {
            eat(w, id, from);
        }
    },
    can_stack_with: |w, id, dropped| {
        let same = if is_food(w, dropped) {
            let (a, b) = (data(w, id), data(w, dropped));
            a.map(|d| (d.poison, d.poisoner)) == b.map(|d| (d.poison, d.poisoner))
        } else {
            true
        };
        same && base::can_stack_with_base(w, id, dropped)
    },
    serialize: |w, id, out| {
        base::serialize_base(w, id, out);
        let d = data(w, id).cloned().unwrap_or_default();
        out.write_i32(0);
        out.write_mobile(w, d.poisoner);
        out.write_poison(d.poison);
        out.write_i32(d.fill_factor);
    },
    deserialize: |w, id, r| {
        base::deserialize_base(w, id, r)?;
        let _version = r.read_i32()?;
        let poisoner = r.read_mobile(w)?;
        let poison = r.read_poison()?;
        let fill_factor = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(FoodData {
                poisoner,
                poison,
                fill_factor,
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// `stack no` / `stack ml` rows: `Stackable = false` / `Stackable = Core.ML` after the base constructor.
fn set_stack(w: &mut World, id: ItemId, mode: Option<bool>) {
    match mode {
        None => {}
        Some(false) => ops::set_stackable(w, id, false),
        Some(true) => {
            let ml = core::ml(w);
            ops::set_stackable(w, id, ml);
        }
    }
}

macro_rules! food_leaves {
    (@stack no) => { Some(false) };
    (@stack ml) => { Some(true) };
    ($($name:ident = $full:literal, $gid:expr, fill $fill:expr, weight $weight:expr
        $(, label $label:expr)? $(, stack $stack:ident)?;)+) => {
        $(
            pub static $name: ItemType = ItemType {
                full_name: $full,
                parent: Some(&FOOD),
                construct: Some(|w| {
                    let id = new_food(w, &$name, $gid, 1);
                    $( set_stack(w, id, food_leaves!(@stack $stack)); )?
                    set_fill_factor(w, id, $fill);
                    id
                }),
                default_weight: |_, _| $weight,
                $( label_number: |_, _| $label, )?
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
        )+

        /// Every leaf generated by `food_leaves!`.
        pub static FOOD_LEAVES: &[&ItemType] = &[$(&$name),+];
    };
}

/// `FruitBasket`: eating it adds a Basket to the pack.
fn fruit_basket_eat(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !eat_base(w, id, from) {
        return false;
    }
    let basket = super::cont::new_base_container(w, &super::cont::BASKET, 0x990);
    muo_server::mobile::part2::items::add_to_backpack(w, from, basket);
    true
}

static FRUIT_BASKET_CLASS: FoodClass = FoodClass {
    eat: fruit_basket_eat,
};

/// `Server.Items.FruitBasket`.
pub static FRUIT_BASKET: ItemType = ItemType {
    full_name: "Server.Items.FruitBasket",
    parent: Some(&FOOD),
    construct: Some(|w| {
        let id = new_food(w, &FRUIT_BASKET, 0x993, 1);
        set_fill_factor(w, id, 5);
        ops::set_stackable(w, id, false);
        id
    }),
    family: Some(&FRUIT_BASKET_CLASS),
    default_weight: |w, id| {
        crate::template_keys::item_float(w, id, "defaultWeight", FOOD.default_weight)
    },
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

food_leaves! {
    BREAD_LOAF = "Server.Items.BreadLoaf", 0x103B, fill 3, weight 1.0;
    BACON = "Server.Items.Bacon", 0x979, fill 1, weight 1.0;
    SLAB_OF_BACON = "Server.Items.SlabOfBacon", 0x976, fill 3, weight 1.0;
    FISH_STEAK = "Server.Items.FishSteak", 0x97B, fill 3, weight 0.1;
    CHEESE_WHEEL = "Server.Items.CheeseWheel", 0x97E, fill 3, weight 0.1;
    CHEESE_WEDGE = "Server.Items.CheeseWedge", 0x97D, fill 3, weight 0.1;
    CHEESE_SLICE = "Server.Items.CheeseSlice", 0x97C, fill 1, weight 0.1;
    FRENCH_BREAD = "Server.Items.FrenchBread", 0x98C, fill 3, weight 2.0;
    FRIED_EGGS = "Server.Items.FriedEggs", 0x9B6, fill 4, weight 1.0;
    COOKED_BIRD = "Server.Items.CookedBird", 0x9B7, fill 5, weight 1.0;
    ROAST_PIG = "Server.Items.RoastPig", 0x9BB, fill 20, weight 45.0;
    SAUSAGE = "Server.Items.Sausage", 0x9C0, fill 4, weight 1.0;
    HAM = "Server.Items.Ham", 0x9C9, fill 5, weight 1.0;
    CAKE = "Server.Items.Cake", 0x9E9, fill 10, weight 1.0, stack no;
    RIBS = "Server.Items.Ribs", 0x9F2, fill 5, weight 1.0;
    COOKIES = "Server.Items.Cookies", 0x160b, fill 4, weight 1.0, stack ml;
    MUFFINS = "Server.Items.Muffins", 0x9eb, fill 4, weight 1.0, stack no;
    CHEESE_PIZZA = "Server.Items.CheesePizza", 0x1040, fill 6, weight 1.0, label 1044516, stack no;
    SAUSAGE_PIZZA = "Server.Items.SausagePizza", 0x1040, fill 6, weight 1.0, label 1044517, stack no;
    FRUIT_PIE = "Server.Items.FruitPie", 0x1041, fill 5, weight 1.0, label 1041346, stack no;
    MEAT_PIE = "Server.Items.MeatPie", 0x1041, fill 5, weight 1.0, label 1041347, stack no;
    PUMPKIN_PIE = "Server.Items.PumpkinPie", 0x1041, fill 5, weight 1.0, label 1041348, stack no;
    APPLE_PIE = "Server.Items.ApplePie", 0x1041, fill 5, weight 1.0, label 1041343, stack no;
    PEACH_COBBLER = "Server.Items.PeachCobbler", 0x1041, fill 5, weight 1.0, label 1041344, stack no;
    QUICHE = "Server.Items.Quiche", 0x1041, fill 5, weight 1.0, label 1041345, stack ml;
    LAMB_LEG = "Server.Items.LambLeg", 0x160a, fill 5, weight 2.0;
    CHICKEN_LEG = "Server.Items.ChickenLeg", 0x1608, fill 4, weight 1.0;
    HONEYDEW_MELON = "Server.Items.HoneydewMelon", 0xC74, fill 1, weight 1.0;
    YELLOW_GOURD = "Server.Items.YellowGourd", 0xC64, fill 1, weight 1.0;
    GREEN_GOURD = "Server.Items.GreenGourd", 0xC66, fill 1, weight 1.0;
    EAR_OF_CORN = "Server.Items.EarOfCorn", 0xC81, fill 1, weight 1.0;
    TURNIP = "Server.Items.Turnip", 0xD3A, fill 1, weight 1.0;
    BANANA = "Server.Items.Banana", 0x171f, fill 1, weight 1.0;
    BANANAS = "Server.Items.Bananas", 0x1721, fill 1, weight 1.0;
    SPLIT_COCONUT = "Server.Items.SplitCoconut", 0x1725, fill 1, weight 1.0;
    LEMON = "Server.Items.Lemon", 0x1728, fill 1, weight 1.0;
    LEMONS = "Server.Items.Lemons", 0x1729, fill 1, weight 1.0;
    LIME = "Server.Items.Lime", 0x172a, fill 1, weight 1.0;
    LIMES = "Server.Items.Limes", 0x172B, fill 1, weight 1.0;
    COCONUT = "Server.Items.Coconut", 0x1726, fill 1, weight 1.0;
    OPEN_COCONUT = "Server.Items.OpenCoconut", 0x1723, fill 1, weight 1.0;
    DATES = "Server.Items.Dates", 0x1727, fill 1, weight 1.0;
    GRAPES = "Server.Items.Grapes", 0x9D1, fill 1, weight 1.0;
    PEACH = "Server.Items.Peach", 0x9D2, fill 1, weight 1.0;
    PEAR = "Server.Items.Pear", 0x994, fill 1, weight 1.0;
    APPLE = "Server.Items.Apple", 0x9D0, fill 1, weight 1.0;
    WATERMELON = "Server.Items.Watermelon", 0xC5C, fill 5, weight 5.0;
    SMALL_WATERMELON = "Server.Items.SmallWatermelon", 0xC5D, fill 5, weight 5.0;
    SQUASH = "Server.Items.Squash", 0xc72, fill 1, weight 1.0;
    CANTALOUPE = "Server.Items.Cantaloupe", 0xc79, fill 1, weight 1.0;
    CARROT = "Server.Items.Carrot", 0xc78, fill 1, weight 1.0;
    CABBAGE = "Server.Items.Cabbage", 0xc7b, fill 1, weight 1.0;
    ONION = "Server.Items.Onion", 0xc6d, fill 1, weight 1.0;
    LETTUCE = "Server.Items.Lettuce", 0xc70, fill 1, weight 1.0;
    PUMPKIN = "Server.Items.Pumpkin", 0xC6A, fill 8, weight 1.0;
    SMALL_PUMPKIN = "Server.Items.SmallPumpkin", 0xC6C, fill 8, weight 1.0;
    WASABI = "Server.Items.Wasabi", 0x24E8, fill 1, weight 1.0;
    WASABI_CLUMPS = "Server.Items.WasabiClumps", 0x24EB, fill 2, weight 1.0, stack no;
    BENTO_BOX = "Server.Items.BentoBox", 0x2836, fill 2, weight 5.0, stack no;
    SUSHI_ROLLS = "Server.Items.SushiRolls", 0x283E, fill 2, weight 3.0, stack no;
    SUSHI_PLATTER = "Server.Items.SushiPlatter", 0x2840, fill 2, weight 3.0, stack ml;
    GREEN_TEA = "Server.Items.GreenTea", 0x284C, fill 2, weight 4.0, stack no;
    MISO_SOUP = "Server.Items.MisoSoup", 0x284D, fill 2, weight 4.0, stack no;
    WHITE_MISO_SOUP = "Server.Items.WhiteMisoSoup", 0x284E, fill 2, weight 4.0, stack no;
    RED_MISO_SOUP = "Server.Items.RedMisoSoup", 0x284F, fill 2, weight 4.0, stack no;
    AWASE_MISO_SOUP = "Server.Items.AwaseMisoSoup", 0x2850, fill 2, weight 4.0, stack no;
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&FRUIT_BASKET, &[]);
    for ty in FOOD_LEAVES {
        // [TypeAlias("Server.Items.Pizza")] on CheesePizza
        let aliases: &[&str] = if std::ptr::eq(*ty, &CHEESE_PIZZA) {
            &["Server.Items.Pizza"]
        } else {
            &[]
        };
        types.register_item(ty, aliases);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_old_pizza_name_loads_as_cheese_pizza() {
        let mut t = TypeRegistry::default();
        register(&mut t);
        let found = t.find_type_by_full_name("Server.Items.Pizza", false);
        assert!(
            matches!(found, Some(muo_server::type_registry::EntityType::Item(i)) if std::ptr::eq(i, &CHEESE_PIZZA))
        );
    }
}
