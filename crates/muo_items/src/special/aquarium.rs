//! `Aquarium`, `AquariumState` and `FishBowl`. The gump is
//! `muo_engines::special_gumps`, reached through [`super::hooks::SpecialHooks::aquarium_gump`].
//!
//! AquariumFishNet, Rewards/AquariumMessage and Rewards/CaptainBlackheartsFishingPole are `aquarium2`. A reward
//! whose type cannot be made gives nothing.

use muo_content::factory;
use muo_content::items::addon::{self};
use muo_content::items::addon_container::{
    self, ADDON_CONTAINER_COMPONENT, BASE_ADDON_CONTAINER, BASE_ADDON_CONTAINER_CLASS,
    BASE_ADDON_CONTAINER_DEED, ContainerAddonClass, ContainerDeedClass,
};
use muo_content::items::cont::{BASE_CONTAINER, new_base_container};
use muo_io::Result;
use muo_primitives::random::Rng;
use muo_primitives::{DateTime, TimeSpan, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ItemType, container, deps, enumerable, ops};
use muo_server::lr_reason::LRReason;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::{
    effects as mobile_effects, items as mobile_items, messages as mobile_messages,
};
use muo_server::property_list::PropertyList;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::fish::{self, AQUARIUM_FOOD, BASE_FISH, VACATION_WAFER};
use super::hooks::hooks;
use crate::food::beverage::{self, BeverageType};
use crate::simple::leaf;

/// `Aquarium.EvaluationInterval`.
pub const EVALUATION_INTERVAL: TimeSpan = TimeSpan::from_days(1);
/// `VacationWafer.VacationDays`.
const VACATION_DAYS: i32 = 7;

/// `FoodState` / `WaterState`.
const DEAD: i32 = 0;
const FULL: i32 = 3;
const OVERFED: i32 = 4;
const STRONG: i32 = 4;

/// `m_Decorations`: what `Accepts` lets in and `GiveReward` hands out (exact classes).
const DECORATIONS: [&str; 9] = [
    "Server.Items.FishBones",
    "Server.Items.WaterloggedBoots",
    "Server.Items.CaptainBlackheartsFishingPole",
    "Server.Items.CraftysFishingHat",
    "Server.Items.AquariumFishNet",
    "Server.Items.AquariumMessage",
    "Server.Items.IslandStatue",
    "Server.Items.Shell",
    "Server.Items.ToyBoat",
];

/// `FishHues`.
const FISH_HUES: [i32; 5] = [0x1C2, 0x1C3, 0x2A3, 0x47E, 0x51D];

/// `AquariumState`.
#[derive(Clone, Copy, Debug, Default)]
pub struct StateData {
    pub state: i32,
    pub maintain: i32,
    pub improve: i32,
    pub added: i32,
}

/// What `Aquarium` adds to a `BaseAddonContainer`.
#[derive(Clone, Debug, Default)]
pub struct AquariumData {
    pub timer: Option<TimerHandle>,
    /// `m_EvaluateDay` (not serialized).
    pub evaluate_day: bool,
    pub live_creatures: i32,
    pub vacation_left: i32,
    pub food: StateData,
    pub water: StateData,
    pub events: Vec<i32>,
    pub reward_available: bool,
}

pub fn data(w: &World, id: ItemId) -> Option<&AquariumData> {
    w.item(id)?.ext.get::<AquariumData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut AquariumData> {
    w.item_mut(id)?.ext.get_mut::<AquariumData>()
}

fn get(w: &World, id: ItemId) -> AquariumData {
    data(w, id).cloned().unwrap_or_default()
}

/// `AquariumState.State` setter.
fn clamp_state(v: i32) -> i32 {
    v.clamp(0, 4)
}

fn items_of(w: &World, id: ItemId) -> Vec<ItemId> {
    w.item(id).map(|i| i.items.clone()).unwrap_or_default()
}

fn is_fish(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_FISH))
}

fn send_msg(w: &mut World, to: MobileId, number: i32, args: &str) {
    mobile_messages::send_localized_message(w, to, number, args, 0x3B2);
}

/// `DeadCreatures`.
pub fn dead_creatures(w: &World, id: ItemId) -> i32 {
    items_of(w, id)
        .into_iter()
        .filter(|&i| is_fish(w, i) && fish::is_dead(w, i))
        .count() as i32
}

/// `MaxLiveCreatures`.
pub fn max_live_creatures(w: &World, id: ItemId) -> i32 {
    let d = get(w, id);
    let mut state = if d.food.state == OVERFED {
        1
    } else {
        FULL - d.food.state
    };
    state += STRONG - d.water.state;
    state = f64::from(state).powf(1.75) as i32;
    container::max_items(w, id) - state
}

/// `IsFull`.
pub fn is_full(w: &World, id: ItemId) -> bool {
    items_of(w, id).len() as i32 >= container::max_items(w, id)
}

/// `OptimalState`.
pub fn optimal_state(w: &World, id: ItemId) -> bool {
    let d = get(w, id);
    d.food.state == FULL && d.water.state == STRONG
}

/// `FoodNumber()`.
pub fn food_number(w: &World, id: ItemId) -> i32 {
    match get(w, id).food.state {
        FULL => 1074240,
        OVERFED => 1074239,
        s => 1074236 + s,
    }
}

/// `WaterNumber()`.
pub fn water_number(w: &World, id: ItemId) -> i32 {
    1074242 + get(w, id).water.state
}

/// `LiveCreatures` setter.
fn set_live(w: &mut World, id: ItemId, v: i32) {
    if let Some(d) = data_mut(w, id) {
        d.live_creatures = v;
    }
}

/// `VacationLeft` setter.
pub fn set_vacation_left(w: &mut World, id: ItemId, v: i32) {
    match data_mut(w, id) {
        Some(d) if d.vacation_left != v => d.vacation_left = v,
        _ => return,
    }
    ops::invalidate_properties(w, id);
}

/// `RewardAvailable` setter.
pub fn set_reward_available(w: &mut World, id: ItemId, v: bool) {
    match data_mut(w, id) {
        Some(d) if d.reward_available != v => d.reward_available = v,
        _ => return,
    }
    ops::invalidate_properties(w, id);
}

fn add_event(w: &mut World, id: ItemId, number: i32) {
    if let Some(d) = data_mut(w, id) {
        d.events.push(number);
    }
}

/// `Food` / `Water` field edits (`aquarium.Food.Added += 1`): `food` picks the state.
fn edit_state(w: &mut World, id: ItemId, food: bool, f: impl FnOnce(&mut StateData)) {
    if let Some(d) = data_mut(w, id) {
        let s = if food { &mut d.food } else { &mut d.water };
        f(s);
        s.state = clamp_state(s.state);
    }
}

// ---- construction, timer ----

fn start_timer(w: &mut World, id: ItemId, delay: TimeSpan) {
    let h = timer::start(w, delay, EVALUATION_INTERVAL, 0, move |w| evaluate(w, id));
    if let Some(d) = data_mut(w, id) {
        d.timer = Some(h);
    }
}

/// `new Aquarium(itemID)`.
pub fn aquarium_new(w: &mut World, item_id: i32) -> ItemId {
    let id = addon_container::new_addon_container(w, &AQUARIUM, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(AquariumData::default());
    }
    ops::set_movable(w, id, false);

    if item_id == 0x3060 {
        let c = addon_container::new_component(w, &ADDON_CONTAINER_COMPONENT, 0x3061);
        addon_container::add_component(w, id, c, -1, 0, 0);
    }
    if item_id == 0x3062 {
        let c = addon_container::new_component(w, &ADDON_CONTAINER_COMPONENT, 0x3063);
        addon_container::add_component(w, id, c, 0, -1, 0);
    }

    container::set_max_items(w, id, 30);

    let food_maintain = utility::random_min_max(&mut w.rng, 1, 2);
    let food_improve = food_maintain + utility::random_min_max(&mut w.rng, 1, 2);
    let water_maintain = utility::random_min_max(&mut w.rng, 1, 3);
    if let Some(d) = data_mut(w, id) {
        d.food = StateData {
            state: FULL,
            maintain: food_maintain,
            improve: food_improve,
            added: 0,
        };
        d.water = StateData {
            state: STRONG,
            maintain: water_maintain,
            improve: 0,
            added: 0,
        };
    }

    start_timer(w, id, EVALUATION_INTERVAL);
    id
}

// ---- access, drops ----

/// `HasAccess(from)`.
pub fn has_access(w: &World, id: ItemId, from: MobileId) -> bool {
    if deps::mobile_deleted(w, from) || w.mobile(from).is_none() {
        return false;
    }
    is_gm(w, from) || addon::item_house_co_owner(w, id, from)
}

fn on_drag_drop(w: &mut World, id: ItemId, from: MobileId, dropped: ItemId) -> bool {
    if !has_access(w, id, from) {
        send_msg(w, from, 1073821, ""); // You do not have access to that item for use with the aquarium.
        return false;
    }
    if get(w, id).vacation_left > 0 {
        send_msg(w, from, 1074427, ""); // The aquarium is in vacation mode.
        return false;
    }

    let mut take_item = true;
    let Some(dty) = w.item(dropped).map(|i| i.ty) else {
        return false;
    };

    if dty.is(&FISH_BOWL) {
        let fish = bowl_fish(w, dropped);
        match fish {
            Some(f) if add_fish(w, id, Some(from), Some(f)) => {}
            _ => return false,
        }
        ops::invalidate_properties(w, dropped);
        take_item = false;
    } else if dty.is(&BASE_FISH) {
        if !add_fish(w, id, Some(from), Some(dropped)) {
            return false;
        }
    } else if dty.is(&VACATION_WAFER) {
        set_vacation_left(w, id, VACATION_DAYS);
        (dty.delete)(w, dropped);
        // The aquarium will be in vacation mode for ~1_DAYS~ days
        send_msg(w, from, 1074428, &get(w, id).vacation_left.to_string());
    } else if dty.is(&AQUARIUM_FOOD) {
        edit_state(w, id, true, |s| s.added += 1);
        (dty.delete)(w, dropped);
        send_msg(w, from, 1074259, "1"); // ~1_NUM~ unit(s) of food have been added to the aquarium.
    } else if beverage::is_beverage(w, dropped) {
        // Pourable is true for every class.
        if beverage::is_empty(w, dropped) || beverage::content(w, dropped) != BeverageType::Water {
            send_msg(w, from, 500840, ""); // Can't pour that in there.
            return false;
        }
        edit_state(w, id, false, |s| s.added += 1);
        let q = beverage::quantity(w, dropped);
        beverage::set_quantity(w, dropped, q - 1);
        mobile_effects::play_sound(w, from, 0x4E);
        send_msg(w, from, 1074260, "1"); // ~1_NUM~ unit(s) of water have been added to the aquarium.
        take_item = false;
    } else if !add_decoration(w, id, Some(from), Some(dropped)) {
        take_item = false;
    }

    if let Some(f) = hooks(w).and_then(|h| h.aquarium_close_gump) {
        f(w, from);
    }
    ops::invalidate_properties(w, id);
    if take_item {
        mobile_effects::play_sound(w, from, 0x42);
    }
    take_item
}

/// `DropItemsToGround()`.
fn drop_items_to_ground(w: &mut World, id: ItemId) {
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    let mut i = items_of(w, id).len();
    while i > 0 {
        i -= 1;
        let Some(item) = w.item(id).and_then(|c| c.items.get(i).copied()) else {
            continue;
        };
        ops::move_to_world_map(w, item, loc, map);
        if is_fish(w, item) && !fish::is_dead(w, item) {
            fish::start_timer(w, item);
        }
    }
}

/// `Accepts(item)`: the exact class.
pub fn accepts(w: &World, item: ItemId) -> bool {
    w.item(item)
        .is_some_and(|i| DECORATIONS.contains(&i.ty.full_name))
}

/// `GetEmptyBowl(from)`.
pub fn get_empty_bowl(w: &World, from: MobileId) -> Option<ItemId> {
    let pack = w.mobile(from)?.backpack?;
    enumerable::find_items_by_type(w, pack, true, &FISH_BOWL, |_, _| true)
        .into_iter()
        .find(|&b| bowl_empty(w, b))
}

/// `AddFish(from, fish)`.
pub fn add_fish(
    w: &mut World,
    id: ItemId,
    from: Option<MobileId>,
    fish_id: Option<ItemId>,
) -> bool {
    let Some(f) = fish_id else { return false };
    let live = get(w, id).live_creatures;
    if is_full(w, id) || live >= max_live_creatures(w, id) || fish::is_dead(w, f) {
        if let Some(m) = from {
            send_msg(w, m, 1073633, ""); // The aquarium can not hold the creature.
        }
        return false;
    }
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.add_item)(w, id, f);
    }
    fish::stop_timer(w, f);
    set_live(w, id, live + 1);
    if let Some(m) = from {
        let label = w.item(f).map_or(0, |i| (i.ty.label_number)(w, f));
        send_msg(w, m, 1073632, &format!("#{label}")); // You add the following creature to your aquarium: ~1_FISH~
    }
    ops::invalidate_properties(w, id);
    true
}

/// `AddDecoration(from, item)`.
pub fn add_decoration(
    w: &mut World,
    id: ItemId,
    from: Option<MobileId>,
    item: Option<ItemId>,
) -> bool {
    let Some(item) = item else { return false };
    if is_full(w, id) {
        if let Some(m) = from {
            send_msg(w, m, 1073636, ""); // The decoration will not fit in the aquarium.
        }
        return false;
    }
    if !accepts(w, item) {
        if let Some(m) = from {
            send_msg(w, m, 1073822, ""); // The aquarium can not hold that item.
        }
        return false;
    }
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.add_item)(w, id, item);
    }
    if let Some(m) = from {
        let label = w.item(item).map_or(0, |i| (i.ty.label_number)(w, item));
        let args = if label != 0 {
            format!("#{label}")
        } else {
            ops::name(w, item).unwrap_or_default()
        };
        send_msg(w, m, 1073635, &args); // You add the following decoration to your aquarium: ~1_NAME~
    }
    ops::invalidate_properties(w, id);
    true
}

/// `RemoveItem(from, at)`.
pub fn remove_item(w: &mut World, id: ItemId, from: MobileId, at: i32) -> bool {
    let items = items_of(w, id);
    if at < 0 || at as usize >= items.len() {
        return false;
    }
    let item = items[at as usize];
    if ops::is_locked_down(w, item) {
        // for legacy aquariums
        send_msg(w, from, 1010449, ""); // You may not use this object while it is locked down.
        return false;
    }

    if is_fish(w, item) {
        if let Some(bowl) = get_empty_bowl(w, from) {
            if let Some(ty) = w.item(bowl).map(|i| i.ty) {
                (ty.add_item)(w, bowl, item);
            }
            send_msg(w, from, 1074511, ""); // You put the creature into a fish bowl.
        } else {
            if !mobile_items::place_in_backpack(w, from, item) {
                send_msg(w, from, 1074514, ""); // You have no place to put it.
                return false;
            }
            send_msg(w, from, 1074512, ""); // You put the gasping creature into your pack.
        }
        if !fish::is_dead(w, item) {
            let live = get(w, id).live_creatures;
            set_live(w, id, live - 1);
        }
    } else {
        if !mobile_items::place_in_backpack(w, from, item) {
            send_msg(w, from, 1074514, ""); // You have no place to put it.
            return false;
        }
        send_msg(w, from, 1074513, ""); // You put the item into your pack.
    }
    ops::invalidate_properties(w, id);
    true
}

/// `ExamineAquarium(from)`.
pub fn examine(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    if !deps::mobile_in_range(w, from, loc, 2) {
        deps::local_overhead_message(
            w,
            from,
            muo_server::packets::message::MessageType::REGULAR,
            0x3B2,
            1019045,
        ); // I can't reach that.
        return;
    }
    if let Some(f) = hooks(w).and_then(|h| h.aquarium_gump) {
        f(w, from, id);
    }
    mobile_effects::play_sound(w, from, 0x5A4);
}

// ---- daily evaluation ----

/// `KillFish(amount)`.
pub fn kill_fish(w: &mut World, id: ItemId, amount: i32) {
    let mut to_kill: Vec<ItemId> = items_of(w, id)
        .into_iter()
        .filter(|&i| is_fish(w, i) && !fish::is_dead(w, i))
        .collect();
    w.rng.shuffle(&mut to_kill);
    let mut amount = amount.min(to_kill.len() as i32);
    let mut i = 0;
    // `amount` shrinks while `i` grows, so half of the requested fish die.
    while i < amount {
        fish::kill(w, to_kill[i as usize]);
        amount -= 1;
        let live = get(w, id).live_creatures;
        set_live(w, id, (live - 1).max(0));
        add_event(w, id, 1074366); // An unfortunate accident has left a creature floating upside-down.  It is starting to smell.
        i += 1;
    }
}

/// `UpdateFoodState()`.
fn update_food_state(w: &mut World, id: ItemId) {
    let mut s = get(w, id).food;
    if s.added < s.maintain {
        s.state = if s.state <= 0 { 0 } else { s.state - 1 };
    } else if s.added >= s.improve {
        s.state = if s.state >= OVERFED {
            OVERFED
        } else {
            s.state + 1
        };
    }
    s.maintain = utility::random_from(&mut w.rng, OVERFED + 1 - s.state, 2);
    s.improve = if s.state == OVERFED {
        0
    } else {
        s.maintain + 2
    };
    s.added = 0;
    edit_state(w, id, true, |d| *d = s);
}

/// `UpdateWaterState()`.
fn update_water_state(w: &mut World, id: ItemId) {
    let mut s = get(w, id).water;
    if s.added < s.maintain {
        s.state = if s.state <= 0 { 0 } else { s.state - 1 };
    } else if s.added >= s.improve {
        s.state = if s.state >= STRONG {
            STRONG
        } else {
            s.state + 1
        };
    }
    s.maintain = utility::random_from(&mut w.rng, STRONG + 2 - s.state, 2);
    s.improve = if s.state == STRONG { 0 } else { s.maintain + 2 };
    s.added = 0;
    edit_state(w, id, false, |d| *d = s);
}

/// `Evaluate()`.
pub fn evaluate(w: &mut World, id: ItemId) {
    if w.item(id).is_none_or(|i| i.deleted()) {
        return;
    }
    let d = get(w, id);
    if d.vacation_left > 0 {
        set_vacation_left(w, id, d.vacation_left - 1);
    } else if d.evaluate_day {
        if let Some(x) = data_mut(w, id) {
            x.events.clear(); // reset events
        }
        let (f, wa) = (d.food, d.water);
        // food events
        if (f.added < f.maintain && f.state != OVERFED && f.state != DEAD)
            || (f.added >= f.improve && f.state == FULL)
        {
            add_event(w, id, 1074368); // The tank looks worse than it did yesterday.
        }
        if (f.added >= f.improve && f.state != FULL && f.state != OVERFED)
            || (f.added < f.maintain && f.state == OVERFED)
        {
            add_event(w, id, 1074367); // The tank looks healthier today.
        }
        // water events
        if wa.added < wa.maintain && wa.state != DEAD {
            add_event(w, id, 1074370); // This tank can use more water.
        }
        if wa.added >= wa.improve && wa.state != STRONG {
            add_event(w, id, 1074369); // The water looks clearer today.
        }
        update_food_state(w, id);
        update_water_state(w, id);
        // reward
        if d.live_creatures > 0 {
            set_reward_available(w, id, true);
        }
    } else {
        // new fish
        if optimal_state(w, id)
            && d.live_creatures < max_live_creatures(w, id)
            && utility::random_double(&mut w.rng) < 0.005 * f64::from(d.live_creatures)
        {
            let (message, name) = match utility::random(&mut w.rng, 6) {
                0 => (1074371, "Server.Items.BrineShrimp"), // Brine shrimp have hatched overnight in the tank.
                1 => (1074365, "Server.Items.Coral"), // A new creature has hatched overnight in the tank.
                2 => (1074365, "Server.Items.FullMoonFish"),
                3 => (1074373, "Server.Items.SeaHorseFish"), // A sea horse has hatched overnight in the tank.
                4 => (1074365, "Server.Items.StrippedFlakeFish"),
                _ => (1074365, "Server.Items.StrippedSosarianSwill"),
            };
            if let Some(fish) = factory::make_item(w, name) {
                if utility::random_double(&mut w.rng) < 0.05 {
                    let hue = utility::random_element(&mut w.rng, &FISH_HUES)
                        .copied()
                        .unwrap_or(0);
                    super::hue(w, fish, hue);
                } else if utility::random_bool(&mut w.rng) {
                    let hue = utility::random_min_max(&mut w.rng, 0x100, 0x3E5);
                    super::hue(w, fish, hue);
                }
                if add_fish(w, id, None, Some(fish)) {
                    add_event(w, id, message);
                } else if let Some(ty) = w.item(fish).map(|i| i.ty) {
                    (ty.delete)(w, fish);
                }
            }
        }

        // kill fish *grins*
        let live = get(w, id).live_creatures;
        let max = max_live_creatures(w, id);
        if live < max {
            if utility::random_double(&mut w.rng) < 0.01 {
                kill_fish(w, id, 1);
            }
        } else {
            kill_fish(w, id, live - max);
        }
    }

    if let Some(x) = data_mut(w, id) {
        x.evaluate_day = !x.evaluate_day;
    }
    ops::invalidate_properties(w, id);
}

/// `GiveReward(to)`.
pub fn give_reward(w: &mut World, id: ItemId, to: MobileId) {
    let d = get(w, id);
    if !d.reward_available {
        return;
    }
    let max = (f64::from(d.live_creatures) / 30.0 * DECORATIONS.len() as f64) as i32;
    let mut random = if max <= 0 {
        0
    } else {
        utility::random(&mut w.rng, max)
    };
    if random >= DECORATIONS.len() as i32 {
        random = DECORATIONS.len() as i32 - 1;
    }
    let Some(item) = factory::make_item(w, DECORATIONS[random as usize]) else {
        return;
    };
    if !mobile_items::place_in_backpack(w, to, item) {
        if let Some(ty) = w.item(item).map(|i| i.ty) {
            (ty.delete)(w, item);
        }
        send_msg(w, to, 1074361, ""); // The reward could not be given.  Make sure you have room in your pack.
        return;
    }
    let label = w.item(item).map_or(0, |i| (i.ty.label_number)(w, item));
    send_msg(w, to, 1074360, &format!("#{label}")); // You receive a reward: ~1_REWARD~
    mobile_effects::play_sound(w, to, 0x5A3);
    set_reward_available(w, id, false);
    ops::invalidate_properties(w, id);
}

// ---- display ----

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    if w.item(id).is_none_or(|i| i.deleted()) || !deps::can_see(w, from, id) {
        return;
    }
    (BASE_ADDON_CONTAINER.on_single_click)(w, id, from);
    let d = get(w, id);
    let max = max_live_creatures(w, id);
    if d.vacation_left > 0 {
        ops::label_to_number(w, id, from, 1074430, &d.vacation_left.to_string()); // Vacation days left: ~1_DAYS
    }
    if !d.events.is_empty() {
        ops::label_to_number(w, id, from, 1074426, &d.events.len().to_string()); // ~1_NUM~ event(s) to view!
    }
    if d.reward_available {
        ops::label_to_number(w, id, from, 1074362, ""); // A reward is available!
    }
    // Live Creatures: ~1_NUM~ / ~2_MAX~
    ops::label_to_number(
        w,
        id,
        from,
        1074247,
        &format!("{}\t{}", d.live_creatures, max),
    );
    let dead = dead_creatures(w, id);
    if dead > 0 {
        ops::label_to_number(w, id, from, 1074248, &dead.to_string()); // Dead Creatures: ~1_NUM~
    }
    let decorations = items_of(w, id).len() as i32 - d.live_creatures - dead;
    if decorations > 0 {
        ops::label_to_number(w, id, from, 1074249, &decorations.to_string()); // Decorations: ~1_NUM~
    }
    let (food_n, water_n) = (food_number(w, id), water_number(w, id));
    ops::label_to_number(w, id, from, 1074250, &format!("#{food_n}")); // Food state: ~1_STATE~
    ops::label_to_number(w, id, from, 1074251, &format!("#{water_n}")); // Water state: ~1_STATE~

    let (f, wa) = (d.food, d.water);
    if f.state == DEAD {
        ops::label_to_number(w, id, from, 1074577, &format!("{}\t{}", f.added, f.improve)); // Food Added: ~1_CUR~ Needed: ~2_NEED~
    } else if f.state == OVERFED {
        ops::label_to_number(
            w,
            id,
            from,
            1074577,
            &format!("{}\t{}", f.added, f.maintain),
        );
    } else {
        // Food Added: ~1_CUR~ Feed: ~2_NEED~ Improve: ~3_GROW~
        ops::label_to_number(
            w,
            id,
            from,
            1074253,
            &format!("{}\t{}\t{}", f.added, f.maintain, f.improve),
        );
    }
    if wa.state == DEAD {
        ops::label_to_number(
            w,
            id,
            from,
            1074578,
            &format!("{}\t{}", wa.added, wa.improve),
        ); // Water Added: ~1_CUR~ Needed: ~2_NEED~
    } else if wa.state == STRONG {
        ops::label_to_number(
            w,
            id,
            from,
            1074578,
            &format!("{}\t{}", wa.added, wa.maintain),
        );
    } else {
        // Water Added: ~1_CUR~ Maintain: ~2_NEED~ Improve: ~3_GROW~
        ops::label_to_number(
            w,
            id,
            from,
            1074254,
            &format!("{}\t{}\t{}", wa.added, wa.maintain, wa.improve),
        );
    }
}

fn add_name_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (BASE_ADDON_CONTAINER.add_name_properties)(w, id, list);
    let d = get(w, id);
    if d.vacation_left > 0 {
        list.add_str(1074430, &d.vacation_left.to_string()); // Vacation days left: ~1_DAYS
    }
    if !d.events.is_empty() {
        list.add_str(1074426, &d.events.len().to_string()); // ~1_NUM~ event(s) to view!
    }
    if d.reward_available {
        list.add(1074362); // A reward is available!
    }
    // Live Creatures: ~1_NUM~ / ~2_MAX~
    list.add_str(
        1074247,
        &format!("{}\t{}", d.live_creatures, max_live_creatures(w, id)),
    );
    let dead = dead_creatures(w, id);
    if dead > 0 {
        list.add_int(1074248, dead); // Dead Creatures: ~1_NUM~
    }
    let decorations = items_of(w, id).len() as i32 - d.live_creatures - dead;
    if decorations > 0 {
        list.add_int(1074249, decorations); // Decorations: ~1_NUM~
    }
    list.add_localized_number(1074250, food_number(w, id)); // Food state: ~1_STATE~
    list.add_localized_number(1074251, water_number(w, id)); // Water state: ~1_STATE~

    let (f, wa) = (d.food, d.water);
    if f.state == DEAD {
        list.add_str(1074577, &format!("{}\t{}", f.added, f.improve)); // Food Added: ~1_CUR~ Needed: ~2_NEED~
    } else if f.state == OVERFED {
        list.add_str(1074577, &format!("{}\t{}", f.added, f.maintain));
    } else {
        // Food Added: ~1_CUR~ Feed: ~2_NEED~ Improve: ~3_GROW~
        list.add_str(
            1074253,
            &format!("{}\t{}\t{}", f.added, f.maintain, f.improve),
        );
    }
    if wa.state == DEAD {
        list.add_str(1074578, &format!("{}\t{}", wa.added, wa.improve)); // Water Added: ~1_CUR~ Needed: ~2_NEED~
    } else if wa.state == STRONG {
        list.add_str(1074578, &format!("{}\t{}", wa.added, wa.maintain));
    } else {
        // Water Added: ~1_CUR~ Maintain: ~2_NEED~ Improve: ~3_GROW~
        list.add_str(
            1074254,
            &format!("{}\t{}\t{}", wa.added, wa.maintain, wa.improve),
        );
    }
}

fn entry(
    number: i32,
    range: i32,
    on_click: impl Fn(&mut World, MobileId, ItemId) + 'static,
) -> ContextMenuEntry {
    let mut e = ContextMenuEntry::new(number, range);
    e.on_click = Some(std::rc::Rc::new(move |w: &mut World, from, target| {
        if let EntityId::Item(a) = target {
            on_click(w, from, a);
        }
    }));
    e
}

fn live_aquarium(w: &World, a: ItemId) -> bool {
    w.item(a)
        .is_some_and(|i| !i.deleted() && i.ty.is(&AQUARIUM))
}

fn is_gm(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
}

fn get_context_menu_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    (BASE_ADDON_CONTAINER.get_context_menu_entries)(w, id, from, list);
    let d = get(w, id);
    if muo_server::mobile::base::alive(w, from) {
        list.push(entry(6235, 2, |w, from, a| {
            if muo_server::mobile::base::alive(w, from) && live_aquarium(w, a) {
                examine(w, a, from);
            }
        }));
        if has_access(w, id, from) {
            if d.reward_available {
                list.push(entry(6237, 2, |w, from, a| {
                    if muo_server::mobile::base::alive(w, from) && live_aquarium(w, a) {
                        give_reward(w, a, from);
                    }
                }));
            }
            if !d.events.is_empty() {
                list.push(entry(6239, 2, |w, from, a| {
                    // `|| aquarium.HasAccess(from)` is intended: a keeper cannot view the events from the menu.
                    if !muo_server::mobile::base::alive(w, from)
                        || !live_aquarium(w, a)
                        || has_access(w, a, from)
                    {
                        return;
                    }
                    let Some(first) = data(w, a).and_then(|d| d.events.first().copied()) else {
                        return;
                    };
                    send_msg(w, from, first, "");
                    if first == 1074366 {
                        mobile_effects::play_sound(w, from, 0x5A2);
                    }
                    if let Some(d) = data_mut(w, a) {
                        d.events.remove(0);
                    }
                    ops::invalidate_properties(w, a);
                }));
            }
            if d.vacation_left > 0 {
                list.push(entry(6240, 2, |w, from, a| {
                    if !muo_server::mobile::base::alive(w, from)
                        || !live_aquarium(w, a)
                        || has_access(w, a, from)
                    {
                        return;
                    }
                    send_msg(w, from, 1074429, ""); // Vacation mode has been cancelled.
                    set_vacation_left(w, a, 0);
                    ops::invalidate_properties(w, a);
                }));
            }
        }
    }

    if is_gm(w, from) {
        list.push(entry(6231, -1, |w, from, a| {
            if is_gm(w, from) && live_aquarium(w, a) {
                edit_state(w, a, true, |s| s.added += 1);
                ops::invalidate_properties(w, a);
            }
        }));
        list.push(entry(6232, -1, |w, from, a| {
            if is_gm(w, from) && live_aquarium(w, a) {
                edit_state(w, a, false, |s| s.added += 1);
                ops::invalidate_properties(w, a);
            }
        }));
        list.push(entry(6233, -1, |w, from, a| {
            if is_gm(w, from) && live_aquarium(w, a) {
                evaluate(w, a);
            }
        }));
        list.push(entry(6234, -1, |w, from, a| {
            if is_gm(w, from)
                && live_aquarium(w, a)
                && let Some(f) = hooks(w).and_then(|h| h.aquarium_gump)
            {
                f(w, from, a);
            }
        }));
        list.push(entry(6236, -1, |w, from, a| {
            if is_gm(w, from) && live_aquarium(w, a) {
                let (fm, wm) = (get(w, a).food.maintain, get(w, a).water.maintain);
                edit_state(w, a, true, |s| s.added = fm);
                edit_state(w, a, false, |s| s.added = wm);
                ops::invalidate_properties(w, a);
            }
        }));
    }
}

fn on_after_duped(w: &mut World, id: ItemId, new_item: ItemId) {
    (BASE_ADDON_CONTAINER.on_after_duped)(w, id, new_item);
    if !w.item(new_item).is_some_and(|i| i.ty.is(&AQUARIUM)) {
        return;
    }
    let d = get(w, id);
    if let Some(n) = data_mut(w, new_item) {
        n.food = d.food;
        n.water = d.water;
        n.events = d.events.clone();
    }
}

// ---- serialization ----

fn write_state(out: &mut dyn muo_io::GenericWriter, s: &StateData) {
    out.write_i32(0);
    out.write_i32(s.state);
    out.write_i32(s.maintain);
    out.write_i32(s.improve);
    out.write_i32(s.added);
}

fn read_state(r: &mut dyn muo_io::GenericReader) -> Result<StateData> {
    let _version = r.read_i32()?;
    Ok(StateData {
        state: r.read_i32()?,
        maintain: r.read_i32()?,
        improve: r.read_i32()?,
        added: r.read_i32()?,
    })
}

fn to_date(ticks: i64) -> DateTime {
    DateTime::try_from_ticks(ticks).unwrap_or(DateTime::MIN)
}

fn serialize(w: &World, id: ItemId, out: &mut dyn muo_io::GenericWriter) {
    (BASE_ADDON_CONTAINER.serialize)(w, id, out);
    out.write_i32(4);
    let d = get(w, id);
    let next = d.timer.map_or(DateTime::MIN, |t| t.next(w));
    out.write_date_time_ticks(next.ticks());
    out.write_i32(d.live_creatures);
    out.write_i32(d.vacation_left);
    write_state(out, &d.food);
    write_state(out, &d.water);
    out.write_encoded_int(d.events.len() as i32);
    for e in &d.events {
        out.write_i32(*e);
    }
    out.write_bool(d.reward_available);
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn muo_io::GenericReader) -> Result<()> {
    (BASE_ADDON_CONTAINER.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = AquariumData::default();
    let mut delay = None;
    let now = w.now;
    if version >= 4 {
        let next = to_date(r.read_date_time_ticks()?);
        if next != DateTime::MIN {
            delay = Some(next - now);
        }
    } else if version >= 1 {
        let mut next = to_date(r.read_date_time_ticks()?);
        if next < now {
            next = now;
        }
        delay = Some(next - now);
    }
    d.live_creatures = r.read_i32()?;
    d.vacation_left = r.read_i32()?;
    d.food = read_state(r)?;
    d.water = read_state(r)?;
    let count = if version >= 4 {
        r.read_encoded_int()?
    } else {
        r.read_i32()?
    };
    for _ in 0..count.max(0) {
        d.events.push(r.read_i32()?);
    }
    d.reward_available = r.read_bool()?;
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
    if let Some(delay) = delay {
        // DeserializeEvaluateTimer: a negative delay (a missed tick) fires at once.
        start_timer(
            w,
            id,
            if delay < TimeSpan::ZERO {
                TimeSpan::ZERO
            } else {
                delay
            },
        );
    }
    Ok(())
}

fn deed_of(w: &mut World, id: ItemId) -> ItemId {
    if w.item(id).is_some_and(|i| i.item_id == 0x3062) {
        east_deed_new(w)
    } else {
        north_deed_new(w)
    }
}

static AQUARIUM_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed_of: Some(deed_of),
    drop_items_to_ground,
    ..BASE_ADDON_CONTAINER_CLASS
};

/// `Server.Items.Aquarium` (no `[Constructible]` constructor).
pub static AQUARIUM: ItemType = ItemType {
    full_name: "Server.Items.Aquarium",
    parent: Some(&BASE_ADDON_CONTAINER),
    construct: None,
    family: Some(&AQUARIUM_CLASS),
    contents_decay: |_, _| false, // fish and decorations are part of the aquarium
    default_weight: |_, _| 10.0,
    // Aquarium.OnDelete does not call base.OnDelete.
    on_delete: |w, id| {
        if let Some(t) = data_mut(w, id).and_then(|d| d.timer.take()) {
            timer::stop(w, t);
        }
    },
    on_double_click: |w, id, from| examine(w, id, from),
    on_drag_drop,
    check_item_use: |w, id, from, item| {
        item == id && (BASE_ADDON_CONTAINER.check_item_use)(w, id, from, item)
    },
    check_lift: |w, id, from, item, reject: &mut LRReason| {
        if item != id {
            *reject = LRReason::CannotLift;
            return false;
        }
        (BASE_ADDON_CONTAINER.check_lift)(w, id, from, item, reject)
    },
    on_single_click,
    add_name_properties,
    get_context_menu_entries,
    on_after_duped,
    serialize,
    deserialize,
    ..BASE_ADDON_CONTAINER
};

/// `AquariumEastDeed()`.
pub fn east_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &AQUARIUM_EAST_DEED)
}
static EAST_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, _| aquarium_new(w, 0x3062),
    select_direction: None,
};
leaf! {
    /// `Server.Items.AquariumEastDeed`.
    AQUARIUM_EAST_DEED = "Server.Items.AquariumEastDeed" : BASE_ADDON_CONTAINER_DEED, v0;
    construct: Some(east_deed_new),
    family: Some(&EAST_DEED_CLASS),
    label_number: |_, _| 1074501, // Large Aquarium (east)
}

/// `AquariumNorthDeed()`.
pub fn north_deed_new(w: &mut World) -> ItemId {
    addon_container::new_deed(w, &AQUARIUM_NORTH_DEED)
}
static NORTH_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, _| aquarium_new(w, 0x3060),
    select_direction: None,
};
leaf! {
    /// `Server.Items.AquariumNorthDeed`.
    AQUARIUM_NORTH_DEED = "Server.Items.AquariumNorthDeed" : BASE_ADDON_CONTAINER_DEED, v0;
    construct: Some(north_deed_new),
    family: Some(&NORTH_DEED_CLASS),
    label_number: |_, _| 1074497, // Large Aquarium (north)
}

// ---- FishBowl ----

/// `FishBowl.Fish`.
pub fn bowl_fish(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?
        .items
        .first()
        .copied()
        .filter(|&f| is_fish(w, f))
}

/// `FishBowl.Empty`.
pub fn bowl_empty(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.items.is_empty())
}

fn bowl_get_context_menu_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    (BASE_CONTAINER.get_context_menu_entries)(w, id, from, list);
    if !bowl_empty(w, id)
        && w.item(id)
            .map(|i| i.ty)
            .is_some_and(|ty| (ty.is_accessible_to)(w, id, from))
    {
        list.push(entry(6242, 3, |w, from, bowl| {
            let ty = w.item(bowl).filter(|i| !i.deleted()).map(|i| i.ty);
            let Some(ty) = ty else { return };
            if !(ty.is_accessible_to)(w, bowl, from) {
                return;
            }
            let Some(f) = bowl_fish(w, bowl) else { return };
            if ops::is_locked_down(w, f) {
                // for legacy fish bowls
                send_msg(w, from, 1010449, ""); // You may not use this object while it is locked down.
            } else if !mobile_items::place_in_backpack(w, from, f) {
                send_msg(w, from, 1074496, ""); // There is no room in your pack for the creature.
            } else {
                send_msg(w, from, 1074495, ""); // The creature has been removed from the fish bowl.
                fish::start_timer(w, f);
                ops::invalidate_properties(w, bowl);
            }
        }));
    }
}

/// `Server.Items.FishBowl`.
pub static FISH_BOWL: ItemType = ItemType {
    full_name: "Server.Items.FishBowl",
    parent: Some(&BASE_CONTAINER),
    construct: Some(|w| {
        let id = new_base_container(w, &FISH_BOWL, 0x241C);
        super::hue(w, id, 0x47E);
        container::set_max_items(w, id, 1);
        id
    }),
    contents_decay: |_, _| false, // the fish is part of the bowl
    label_number: |_, _| 1074499, // A fish bowl
    default_weight: |_, _| 2.0,
    on_double_click: |_, _, _| {},
    try_drop_item: |w, id, from, dropped, send_full, play_sound| {
        if play_sound {
            return container::try_drop_item_default(w, id, from, dropped, send_full, play_sound);
        }
        let Some(ty) = w.item(id).map(|i| i.ty) else {
            return false;
        };
        if !(ty.check_hold)(w, id, from, dropped, send_full, true, 0, 0) {
            return false;
        }
        (ty.drop_item)(w, id, dropped);
        true
    },
    on_drag_drop: |w, id, from, dropped| {
        let Some(ty) = w.item(id).map(|i| i.ty) else {
            return false;
        };
        if !(ty.is_accessible_to)(w, id, from) {
            send_msg(w, from, 502436, ""); // That is not accessible.
            return false;
        }
        if !is_fish(w, dropped) {
            send_msg(w, from, 1074836, ""); // The container can not hold that type of object.
            return false;
        }
        if (BASE_CONTAINER.on_drag_drop)(w, id, from, dropped) {
            fish::stop_timer(w, dropped);
            ops::invalidate_properties(w, id);
            return true;
        }
        false
    },
    check_item_use: |w, id, from, item| {
        item == id && (BASE_CONTAINER.check_item_use)(w, id, from, item)
    },
    check_lift: |w, id, from, item, reject: &mut LRReason| {
        if item != id {
            *reject = LRReason::CannotLift;
            return false;
        }
        (BASE_CONTAINER.check_lift)(w, id, from, item, reject)
    },
    add_name_properties: |w, id, list| {
        (BASE_CONTAINER.add_name_properties)(w, id, list);
        if let Some(f) = bowl_fish(w, id) {
            let label = w.item(f).map_or(0, |i| (i.ty.label_number)(w, f));
            list.add_localized_number(1074494, label); // Contains: ~1_CREATURE~
        }
    },
    get_context_menu_entries: bowl_get_context_menu_entries,
    serialize: |w, id, out| {
        (BASE_CONTAINER.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_CONTAINER
};

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &AQUARIUM,
        &AQUARIUM_EAST_DEED,
        &AQUARIUM_NORTH_DEED,
        &FISH_BOWL,
    ] {
        types.register_item(t, &[]);
    }
}
