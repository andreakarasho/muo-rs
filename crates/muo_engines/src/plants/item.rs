//! Plants: `PlantItem`, `PlantSystem` and `PlantResources`.
//!
//! Clients older than 7.0.12.0 get `OldClientPropertyList` (the dirt cliloc arguments leave out the container).

use muo_content::hooks::rnd;
use muo_net::client_version::ClientVersion;
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType;
use muo_server::property_list::PropertyList;
use muo_server::type_registry::TypeRegistry;

use super::{
    PlantHueInfo, PlantTypeInfo, can_hue_reproduce, can_type_reproduce, cross_hue, cross_type,
    hue as hue_c, is_bright, is_hue_crossable, is_type_crossable,
};

/// `PlantStatus`.
pub mod status {
    pub const BOWL_OF_DIRT: i32 = 0;
    pub const SEED: i32 = 1;
    pub const SAPLING: i32 = 2;
    pub const STAGE3: i32 = 3;
    pub const PLANT: i32 = 4;
    pub const STAGE5: i32 = 5;
    pub const STAGE9: i32 = 9;
    pub const FULL_GROWN_PLANT: i32 = 7;
    pub const DECORATIVE_PLANT: i32 = 10;
    pub const DEAD_TWIGS: i32 = 11;
}

/// `PlantHealth`.
pub mod health {
    pub const DYING: i32 = 0;
    pub const WILTED: i32 = 1;
    pub const HEALTHY: i32 = 2;
    pub const VIBRANT: i32 = 3;
}

/// `PlantGrowthIndicator`.
pub mod indicator {
    pub const NONE: i32 = 0;
    pub const INVALID_LOCATION: i32 = 1;
    pub const NOT_HEALTHY: i32 = 2;
    pub const DELAY: i32 = 3;
    pub const GROWN: i32 = 4;
    pub const DOUBLE_GROWN: i32 = 5;
}

pub const CHECK_DELAY: TimeSpan = TimeSpan::from_hours(23);

/// `PlantSystem`.
#[derive(Clone, Debug)]
pub struct PlantSystem {
    pub fertile_dirt: bool,
    pub next_growth: DateTime,
    pub growth_indicator: i32,
    pub pollinated: bool,
    pub water: i32,
    pub hits: i32,
    pub infestation: i32,
    pub fungus: i32,
    pub poison: i32,
    pub disease: i32,
    pub poison_potion: i32,
    pub cure_potion: i32,
    pub heal_potion: i32,
    pub strength_potion: i32,
    /// The raw `_seedType` / `_seedHue` (the getters answer the plant's own while not pollinated).
    pub seed_type: i32,
    pub seed_hue: i32,
    pub available_seeds: i32,
    pub left_seeds: i32,
    pub available_resources: i32,
    pub left_resources: i32,
}

pub fn max_hits(plant_status: i32) -> i32 {
    10 + plant_status * 2
}

impl PlantSystem {
    /// `new PlantSystem(plant)`.
    pub fn new(now: DateTime, plant_status: i32) -> PlantSystem {
        PlantSystem {
            fertile_dirt: false,
            next_growth: now + CHECK_DELAY,
            growth_indicator: indicator::NONE,
            pollinated: false,
            water: 0,
            hits: max_hits(plant_status),
            infestation: 0,
            fungus: 0,
            poison: 0,
            disease: 0,
            poison_potion: 0,
            cure_potion: 0,
            heal_potion: 0,
            strength_potion: 0,
            seed_type: 0,
            seed_hue: 0,
            available_seeds: 0,
            left_seeds: 8,
            available_resources: 0,
            left_resources: 8,
        }
    }

    pub fn is_full_water(&self) -> bool {
        self.water >= 4
    }
    pub fn is_full_poison_potion(&self) -> bool {
        self.poison_potion >= 2
    }
    pub fn is_full_cure_potion(&self) -> bool {
        self.cure_potion >= 2
    }
    pub fn is_full_heal_potion(&self) -> bool {
        self.heal_potion >= 2
    }
    pub fn is_full_strength_potion(&self) -> bool {
        self.strength_potion >= 2
    }
    pub fn has_maladies(&self) -> bool {
        self.infestation > 0
            || self.fungus > 0
            || self.poison > 0
            || self.disease > 0
            || self.water != 2
    }

    /// `Health` for a plant of `plant_status`.
    pub fn health(&self, plant_status: i32) -> i32 {
        match self.hits * 100 / max_hits(plant_status) {
            ..33 => health::DYING,
            33..66 => health::WILTED,
            66..100 => health::HEALTHY,
            _ => health::VIBRANT,
        }
    }

    /// `GetLocalizedDirtStatus()`.
    pub fn localized_dirt_status(&self) -> i32 {
        match self.water {
            ..=1 => 1060826, // hard
            2 => 1060827,    // soft
            3 => 1060828,    // squishy
            _ => 1060829,    // sappy wet
        }
    }

    /// `GetLocalizedHealth()`.
    pub fn localized_health(&self, plant_status: i32) -> i32 {
        match self.health(plant_status) {
            health::DYING => 1060825,
            health::WILTED => 1060824,
            health::HEALTHY => 1060823,
            _ => 1060822,
        }
    }
}

/// `PlantItem`'s state.
#[derive(Clone, Debug)]
pub struct PlantData {
    /// `SecureLevel` (`Owner` = 0).
    pub level: i32,
    pub status: i32,
    pub plant_type: i32,
    pub plant_hue: i32,
    pub show_type: bool,
    pub system: Option<PlantSystem>,
    /// `_oldClientPropertyList` (clients before 7.0.12.0: the container type isn't part of the dirt cliloc arguments).
    pub old_list: Option<PropertyList>,
}

/// `PlantItem.Plants`.
#[derive(Default)]
pub struct PlantList(pub Vec<ItemId>);

pub fn data(w: &World, id: ItemId) -> Option<&PlantData> {
    w.item(id)?.ext.get::<PlantData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut PlantData> {
    w.item_mut(id)?.ext.get_mut::<PlantData>()
}

pub fn is_plant(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&PLANT_ITEM))
}

pub fn plant_status(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.status)
}

pub fn plant_type(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.plant_type)
}

pub fn plant_hue(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(hue_c::NONE, |d| d.plant_hue)
}

pub fn sys(w: &World, id: ItemId) -> Option<&PlantSystem> {
    data(w, id)?.system.as_ref()
}

fn sys_mut(w: &mut World, id: ItemId) -> Option<&mut PlantSystem> {
    data_mut(w, id)?.system.as_mut()
}

/// `PlantHueInfo.IsCrossable(PlantHue) && PlantTypeInfo.IsCrossable(PlantType)`.
pub fn is_crossable(w: &World, id: ItemId) -> bool {
    is_hue_crossable(plant_hue(w, id)) && is_type_crossable(plant_type(w, id))
}

/// `Reproduces`.
pub fn reproduces(w: &World, id: ItemId) -> bool {
    can_hue_reproduce(plant_hue(w, id)) && can_type_reproduce(plant_type(w, id))
}

/// `IsGrowable`.
pub fn is_growable(w: &World, id: ItemId) -> bool {
    (status::BOWL_OF_DIRT..=status::STAGE9).contains(&plant_status(w, id))
}

/// `PollenProducing`.
pub fn pollen_producing(w: &World, id: ItemId) -> bool {
    is_crossable(w, id) && plant_status(w, id) >= status::FULL_GROWN_PLANT
}

/// `PlantSystem.Health` of the plant, or Dying without one.
pub fn health_of(w: &World, id: ItemId) -> i32 {
    sys(w, id).map_or(health::DYING, |s| s.health(plant_status(w, id)))
}

/// `PlantSystem.SeedType` (the plant's own while it is not pollinated).
pub fn seed_type(w: &World, id: ItemId) -> i32 {
    sys(w, id).map_or(0, |s| {
        if s.pollinated {
            s.seed_type
        } else {
            plant_type(w, id)
        }
    })
}

/// `PlantSystem.SeedHue`.
pub fn seed_hue(w: &World, id: ItemId) -> i32 {
    sys(w, id).map_or(0, |s| {
        if s.pollinated {
            s.seed_hue
        } else {
            plant_hue(w, id)
        }
    })
}

// ---- PlantSystem setters ----

/// `Water = value`.
pub fn set_water(w: &mut World, id: ItemId, value: i32) {
    let Some(s) = sys_mut(w, id) else { return };
    if value == s.water {
        return;
    }
    s.water = value.clamp(0, 4);
    ops::invalidate_properties(w, id);
}

/// `Hits = value`.
pub fn set_hits(w: &mut World, id: ItemId, value: i32) {
    let st = plant_status(w, id);
    let Some(s) = sys_mut(w, id) else { return };
    if value == s.hits {
        return;
    }
    s.hits = value.clamp(0, max_hits(st));
    let dead = s.hits == 0;
    if dead {
        die(w, id);
    }
    ops::invalidate_properties(w, id);
}

macro_rules! clamped {
    ($($name:ident: $field:ident, $lo:expr, $hi:expr;)*) => {$(
        pub fn $name(w: &mut World, id: ItemId, value: i32) {
            if let Some(s) = sys_mut(w, id) {
                s.$field = value.clamp($lo, $hi);
            }
        }
    )*};
}

clamped! {
    set_infestation: infestation, 0, 2;
    set_fungus: fungus, 0, 2;
    set_poison: poison, 0, 2;
    set_disease: disease, 0, 2;
    set_poison_potion: poison_potion, 0, 2;
    set_cure_potion: cure_potion, 0, 2;
    set_heal_potion: heal_potion, 0, 2;
    set_strength_potion: strength_potion, 0, 2;
    set_available_seeds: available_seeds, 0, i32::MAX;
    set_left_seeds: left_seeds, 0, i32::MAX;
    set_available_resources: available_resources, 0, i32::MAX;
    set_left_resources: left_resources, 0, i32::MAX;
}

pub fn set_pollinated(w: &mut World, id: ItemId, value: bool) {
    if let Some(s) = sys_mut(w, id) {
        s.pollinated = value;
    }
}

/// `SeedType = value` (the raw field).
pub fn set_seed_type(w: &mut World, id: ItemId, value: i32) {
    if let Some(s) = sys_mut(w, id) {
        s.seed_type = value;
    }
}

/// `SeedHue = value` (the raw field).
pub fn set_seed_hue(w: &mut World, id: ItemId, value: i32) {
    if let Some(s) = sys_mut(w, id) {
        s.seed_hue = value;
    }
}

pub fn set_fertile_dirt(w: &mut World, id: ItemId, value: bool) {
    if let Some(s) = sys_mut(w, id) {
        s.fertile_dirt = value;
    }
}

fn set_growth_indicator(w: &mut World, id: ItemId, value: i32) {
    if let Some(s) = sys_mut(w, id) {
        s.growth_indicator = value;
    }
}

/// `Reset(potions)`.
pub fn reset(w: &mut World, id: ItemId, potions: bool) {
    let now = w.now;
    let st = plant_status(w, id);
    if let Some(s) = sys_mut(w, id) {
        s.next_growth = now + CHECK_DELAY;
        s.growth_indicator = indicator::NONE;
    }
    set_hits(w, id, max_hits(st));
    set_infestation(w, id, 0);
    set_fungus(w, id, 0);
    set_poison(w, id, 0);
    set_disease(w, id, 0);
    if potions {
        set_poison_potion(w, id, 0);
        set_cure_potion(w, id, 0);
        set_heal_potion(w, id, 0);
        set_strength_potion(w, id, 0);
    }
    set_pollinated(w, id, false);
    set_available_seeds(w, id, 0);
    set_left_seeds(w, id, 8);
    set_available_resources(w, id, 0);
    set_left_resources(w, id, 8);
}

// ---- PlantItem ----

/// `PlantItem(fertileDirt)`: `base(0x1602)`.
pub fn plant_new(w: &mut World, fertile_dirt: bool) -> ItemId {
    let id = ops::new_item_with_id(w, &PLANT_ITEM, 0x1602);
    let mut system = PlantSystem::new(w.now, status::BOWL_OF_DIRT);
    system.fertile_dirt = fertile_dirt;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(PlantData {
            level: 0,
            status: status::BOWL_OF_DIRT,
            plant_type: 0,
            plant_hue: hue_c::NONE,
            show_type: false,
            system: Some(system),
            old_list: None,
        });
    }
    w.resources.get_mut::<PlantList>().0.push(id);
    id
}

/// `PlantStatus = value`.
pub fn set_status(w: &mut World, id: ItemId, value: i32) {
    let Some(d) = data(w, id) else { return };
    if d.status == value || !(status::BOWL_OF_DIRT..=status::DEAD_TWIGS).contains(&value) {
        return;
    }
    let ratio = d
        .system
        .as_ref()
        .map_or(1.0, |s| f64::from(s.hits) / f64::from(max_hits(d.status)));
    let had_system = d.system.is_some();
    let now = w.now;
    if let Some(d) = data_mut(w, id) {
        d.status = value;
        if value >= status::DECORATIVE_PLANT {
            d.system = None;
        } else if !had_system {
            d.system = Some(PlantSystem::new(now, value));
        }
    }
    if value < status::DECORATIVE_PLANT {
        let hits = (f64::from(max_hits(value)) * ratio) as i32;
        if hits == 0 && value > status::BOWL_OF_DIRT {
            set_hits(w, id, 1);
        } else {
            set_hits(w, id, hits);
        }
    }
    update(w, id);
}

pub fn set_plant_type(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id)
        && d.plant_type != value
    {
        d.plant_type = value;
        update(w, id);
    }
}

pub fn set_plant_hue(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id)
        && d.plant_hue != value
    {
        d.plant_hue = value;
        update(w, id);
    }
}

pub fn set_show_type(w: &mut World, id: ItemId, value: bool) {
    if let Some(d) = data_mut(w, id)
        && d.show_type != value
    {
        d.show_type = value;
        ops::invalidate_properties(w, id);
    }
}

pub fn set_level(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        d.level = value;
    }
}

/// `Update()`.
fn update(w: &mut World, id: ItemId) {
    let Some(d) = data(w, id) else { return };
    let (st, ty, hu) = (d.status, d.plant_type, d.plant_hue);
    let ty_info = PlantTypeInfo::get(ty);
    let (item_id, hue) = if st >= status::DEAD_TWIGS {
        (0x1B9D, PlantHueInfo::get(hu).hue)
    } else if st >= status::FULL_GROWN_PLANT {
        (ty_info.item_id, PlantHueInfo::get(hu).hue)
    } else if st >= status::PLANT {
        (0x1600, 0)
    } else {
        (0x1602, 0)
    };
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.set_item_id)(w, id, item_id);
    }
    crate::util::hue(w, id, hue);
    ops::invalidate_properties(w, id);
}

/// `GetLocalizedPlantStatus()`.
pub fn localized_plant_status(w: &World, id: ItemId) -> i32 {
    match plant_status(w, id) {
        4.. => 1060812, // plant
        2.. => 1023305, // sapling
        1.. => 1060810, // seed
        _ => 1026951,   // dirt
    }
}

/// `ValidGrowthLocation`.
pub fn valid_growth_location(w: &mut World, id: ItemId) -> bool {
    let root = ops::root_parent(w, id);
    if ops::is_locked_down(w, id) && root.is_none() {
        return true;
    }
    let Some(EntityId::Mobile(owner)) = root else {
        return false;
    };
    let pack = w.mobile(owner).and_then(|m| m.backpack).map(EntityId::Item);
    if ops::is_child_of(w, id, pack) {
        return true;
    }
    let bank = muo_server::mobile::part2::items::find_bank_no_create(w, owner).map(EntityId::Item);
    ops::is_child_of(w, id, bank)
}

/// `IsUsableBy(from)`.
pub fn is_usable_by(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    if ops::is_child_of(w, id, pack) {
        return true;
    }
    let bank = muo_server::mobile::part2::items::find_bank_no_create(w, from).map(EntityId::Item);
    if ops::is_child_of(w, id, bank) {
        return true;
    }
    let ty = w.item(id).map(|i| i.ty);
    if ops::is_locked_down(w, id) && ty.is_some_and(|t| (t.is_accessible_to)(w, id, from)) {
        return true;
    }
    if let Some(EntityId::Item(root)) = ops::root_parent(w, id)
        && ops::is_secure(w, root)
    {
        return w
            .item(root)
            .map(|i| i.ty)
            .is_some_and(|t| (t.is_accessible_to)(w, root, from));
    }
    false
}

fn label(w: &mut World, id: ItemId, to: MobileId, number: i32) {
    ops::label_to_number(w, id, to, number, "");
}

/// `Die()`.
pub fn die(w: &mut World, id: ItemId) {
    if plant_status(w, id) >= status::FULL_GROWN_PLANT {
        set_status(w, id, status::DEAD_TWIGS);
    } else {
        set_status(w, id, status::BOWL_OF_DIRT);
        reset(w, id, true);
    }
}

/// `PlantSeed(from, seed)`.
pub fn plant_seed(w: &mut World, from: MobileId, id: ItemId, seed: ItemId) {
    let st = plant_status(w, id);
    if st >= status::FULL_GROWN_PLANT {
        label(w, id, from, 1061919); // You must use a seed on some prepared soil!
    } else if !is_usable_by(w, id, from) {
        label(w, id, from, 1061921); // The bowl of dirt must be in your pack, or you must lock it down.
    } else if st != status::BOWL_OF_DIRT {
        // This bowl of dirt already has a ~1_val~ in it!
        let n = localized_plant_status(w, id);
        muo_content::mx::msg_args(w, from, 1080389, &format!("#{n}"));
    } else if sys(w, id).is_some_and(|s| s.water < 2) {
        label(w, id, from, 1061920); // The dirt needs to be softened first.
    } else {
        let Some(sd) = super::seed::data(w, seed).copied() else {
            return;
        };
        set_plant_type(w, id, sd.plant_type);
        set_plant_hue(w, id, sd.plant_hue);
        set_show_type(w, id, sd.show_type);
        base::consume(w, seed, 1);
        set_status(w, id, status::SEED);
        reset(w, id, false);
        label(w, id, from, 1061922); // You plant the seed in the bowl of dirt.
    }
}

/// `Pour(from, item)`.
pub fn pour(w: &mut World, id: ItemId, from: MobileId, item: ItemId) {
    use muo_items::food::beverage;
    use muo_items::skill_items::{potion_keg, potions};

    let st = plant_status(w, id);
    if st >= status::DEAD_TWIGS {
        return;
    }
    if st == status::DECORATIVE_PLANT {
        label(w, id, from, 1053049); // This is a decorative plant, it does not need watering!
        return;
    }
    if !is_usable_by(w, id, from) {
        label(w, id, from, 1061856);
        return;
    }
    if beverage::is_beverage(w, item) {
        // `Pourable` is always true.
        if beverage::is_empty(w, item)
            || beverage::content(w, item) != beverage::BeverageType::Water
        {
            label(w, id, from, 1053069); // You can't use that on a plant!
            return;
        }
        if !beverage::validate_use(w, item, from, true) {
            return;
        }
        let q = beverage::quantity(w, item);
        beverage::set_quantity(w, item, q - 1);
        let water = sys(w, id).map_or(0, |s| s.water);
        set_water(w, id, water + 1);
        muo_content::mx::play_sound(w, from, 0x4E);
        label(w, id, from, 1061858); // You soften the dirt with water.
    } else if potions::is_potion(w, item) {
        let effect = potions::effect(w, item).unwrap_or(0);
        let (ok, message) = apply_potion(w, id, effect, false);
        if ok {
            base::consume(w, item, 1);
            muo_content::mx::play_sound(w, from, 0x240);
            if let Some(bottle) = muo_content::factory::make_item(w, "Server.Items.Bottle") {
                muo_server::mobile::part2::items::add_to_backpack(w, from, bottle);
            }
        }
        label(w, id, from, message);
    } else if w
        .item(item)
        .is_some_and(|i| i.ty.is(&potion_keg::POTION_KEG))
    {
        if potion_keg::held(w, item) <= 0 {
            label(w, id, from, 1053069);
            return;
        }
        let (ok, message) = apply_potion(w, id, potion_keg::keg_type(w, item), false);
        if ok {
            let h = potion_keg::held(w, item);
            potion_keg::set_held(w, item, h - 1);
            muo_content::mx::play_sound(w, from, 0x240);
        }
        label(w, id, from, message);
    } else {
        label(w, id, from, 1053069);
    }
}

/// `ApplyPotion(effect, testOnly, out message)`.
pub fn apply_potion(w: &mut World, id: ItemId, effect: i32, test_only: bool) -> (bool, i32) {
    use muo_items::skill_items::potions::fx;
    let st = plant_status(w, id);
    if st >= status::DECORATIVE_PLANT {
        return (false, 1053049);
    }
    if st == status::BOWL_OF_DIRT {
        return (false, 1053066); // You should only pour potions on a plant or seed!
    }
    let Some(s) = sys(w, id).cloned() else {
        return (false, 1053069);
    };
    let full;
    if matches!(effect, fx::POISON_GREATER | fx::POISON_DEADLY) {
        full = s.is_full_poison_potion();
        if !full && !test_only {
            set_poison_potion(w, id, s.poison_potion + 1);
        }
    } else if effect == fx::CURE_GREATER {
        full = s.is_full_cure_potion();
        if !full && !test_only {
            set_cure_potion(w, id, s.cure_potion + 1);
        }
    } else if effect == fx::HEAL_GREATER {
        full = s.is_full_heal_potion();
        if !full && !test_only {
            set_heal_potion(w, id, s.heal_potion + 1);
        }
    } else if effect == fx::STRENGTH_GREATER {
        full = s.is_full_strength_potion();
        if !full && !test_only {
            set_strength_potion(w, id, s.strength_potion + 1);
        }
    } else if matches!(
        effect,
        fx::POISON_LESSER
            | fx::POISON
            | fx::CURE_LESSER
            | fx::CURE
            | fx::HEAL_LESSER
            | fx::HEAL
            | fx::STRENGTH
    ) {
        return (false, 1053068); // This potion is not powerful enough to use on a plant!
    } else {
        return (false, 1053069);
    }
    if full {
        return (false, 1053065); // The plant is already soaked with this type of potion!
    }
    (true, 1053067) // You pour the potion over the plant.
}

// ---- growth ----

/// `GrowAll()`.
pub fn grow_all(w: &mut World) {
    let plants = w.resources.get_mut::<PlantList>().0.clone();
    let now = w.now;
    for &p in plants.iter().rev() {
        let due = is_plant(w, p)
            && is_growable(w, p)
            && !matches!(ops::root_parent(w, p), Some(EntityId::Mobile(_)))
            && sys(w, p).is_some_and(|s| now >= s.next_growth);
        if due {
            do_growth_check(w, p);
        }
    }
}

/// `PlantSystem.OnLogin(from)`.
pub fn on_login(w: &mut World, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    let bank = muo_server::mobile::part2::items::find_bank_no_create(w, from);
    for cont in [pack, bank].into_iter().flatten() {
        let found =
            muo_server::item::enumerable::find_items_by_type(w, cont, true, &PLANT_ITEM, |_, _| {
                true
            });
        for p in found {
            if is_growable(w, p) {
                do_growth_check(w, p);
            }
        }
    }
}

/// `DoGrowthCheck()`.
pub fn do_growth_check(w: &mut World, id: ItemId) {
    if !is_growable(w, id) {
        return;
    }
    let now = w.now;
    let Some(next) = sys(w, id).map(|s| s.next_growth) else {
        return;
    };
    if now < next {
        set_growth_indicator(w, id, indicator::DELAY);
        return;
    }
    if let Some(s) = sys_mut(w, id) {
        s.next_growth = now + CHECK_DELAY;
    }
    if !valid_growth_location(w, id) {
        set_growth_indicator(w, id, indicator::INVALID_LOCATION);
        return;
    }
    if plant_status(w, id) == status::BOWL_OF_DIRT {
        let water = sys(w, id).map_or(0, |s| s.water);
        if water > 2 || rnd::double(w) < 0.9 {
            set_water(w, id, water - 1);
        }
        return;
    }
    apply_beneficial_effects(w, id);
    if !apply_maladies_effects(w, id) {
        return; // Dead
    }
    grow(w, id);
    update_maladies(w, id);
}

fn s(w: &World, id: ItemId) -> PlantSystem {
    sys(w, id)
        .cloned()
        .unwrap_or_else(|| PlantSystem::new(DateTime::MIN, 0))
}

fn apply_beneficial_effects(w: &mut World, id: ItemId) {
    let a = s(w, id);
    if a.poison_potion >= a.infestation {
        set_poison_potion(w, id, a.poison_potion - a.infestation);
        set_infestation(w, id, 0);
    } else {
        set_infestation(w, id, a.infestation - a.poison_potion);
        set_poison_potion(w, id, 0);
    }
    let a = s(w, id);
    if a.cure_potion >= a.fungus {
        set_cure_potion(w, id, a.cure_potion - a.fungus);
        set_fungus(w, id, 0);
    } else {
        set_fungus(w, id, a.fungus - a.cure_potion);
        set_cure_potion(w, id, 0);
    }
    let a = s(w, id);
    if a.heal_potion >= a.poison {
        set_heal_potion(w, id, a.heal_potion - a.poison);
        set_poison(w, id, 0);
    } else {
        set_poison(w, id, a.poison - a.heal_potion);
        set_heal_potion(w, id, 0);
    }
    let a = s(w, id);
    if a.heal_potion >= a.disease {
        set_heal_potion(w, id, a.heal_potion - a.disease);
        set_disease(w, id, 0);
    } else {
        set_disease(w, id, a.disease - a.heal_potion);
        set_heal_potion(w, id, 0);
    }
    let a = s(w, id);
    if !a.has_maladies() {
        let hits = if a.heal_potion > 0 {
            a.hits + a.heal_potion * 7
        } else {
            a.hits + 2
        };
        set_hits(w, id, hits);
    }
    set_heal_potion(w, id, 0);
}

fn apply_maladies_effects(w: &mut World, id: ItemId) -> bool {
    let a = s(w, id);
    let mut damage = 0;
    for v in [a.infestation, a.fungus, a.poison, a.disease] {
        if v > 0 {
            damage += v * rnd::min_max(w, 3, 6);
        }
    }
    if a.water > 2 {
        damage += (a.water - 2) * rnd::min_max(w, 3, 6);
    } else if a.water < 2 {
        damage += (2 - a.water) * rnd::min_max(w, 3, 6);
    }
    let hits = s(w, id).hits;
    set_hits(w, id, hits - damage);
    is_growable(w, id) && plant_status(w, id) != status::BOWL_OF_DIRT
}

fn grow(w: &mut World, id: ItemId) {
    let st = plant_status(w, id);
    let a = s(w, id);
    if health_of(w, id) < health::HEALTHY {
        set_growth_indicator(w, id, indicator::NOT_HEALTHY);
    } else if a.fertile_dirt && st <= status::STAGE5 && rnd::double(w) < 0.1 {
        set_status(w, id, st + 2);
        set_growth_indicator(w, id, indicator::DOUBLE_GROWN);
    } else if st < status::STAGE9 {
        set_status(w, id, st + 1);
        set_growth_indicator(w, id, indicator::GROWN);
    } else {
        if a.pollinated && a.left_seeds > 0 && reproduces(w, id) {
            set_left_seeds(w, id, a.left_seeds - 1);
            set_available_seeds(w, id, a.available_seeds + 1);
        }
        let a = s(w, id);
        if a.left_resources > 0 && resource_info(plant_type(w, id), plant_hue(w, id)).is_some() {
            set_left_resources(w, id, a.left_resources - 1);
            set_available_resources(w, id, a.available_resources + 1);
        }
        set_growth_indicator(w, id, indicator::GROWN);
    }
    if plant_status(w, id) >= status::STAGE9 && !s(w, id).pollinated {
        set_pollinated(w, id, true);
        set_seed_type(w, id, plant_type(w, id));
        set_seed_hue(w, id, plant_hue(w, id));
    }
}

fn update_maladies(w: &mut World, id: ItemId) {
    let a = s(w, id);
    let mut infestation_chance =
        0.30 - f64::from(a.strength_potion) * 0.075 + f64::from(a.water - 2) * 0.10;
    let type_info = PlantTypeInfo::get(plant_type(w, id));
    if type_info.flowery {
        infestation_chance += 0.10;
    }
    if is_bright(plant_hue(w, id)) {
        infestation_chance += 0.10;
    }
    if rnd::double(w) < infestation_chance {
        let v = s(w, id).infestation + 1;
        set_infestation(w, id, v);
    }
    let a = s(w, id);
    let fungus_chance = 0.15 - f64::from(a.strength_potion) * 0.075 + f64::from(a.water - 2) * 0.10;
    if rnd::double(w) < fungus_chance {
        let v = s(w, id).fungus + 1;
        set_fungus(w, id, v);
    }
    let water = s(w, id).water;
    if water > 2 || rnd::double(w) < 0.9 {
        set_water(w, id, water - 1);
    }
    let a = s(w, id);
    if a.poison_potion > 0 {
        set_poison(w, id, a.poison + a.poison_potion);
        set_poison_potion(w, id, 0);
    }
    let a = s(w, id);
    if a.cure_potion > 0 {
        set_disease(w, id, a.disease + a.cure_potion);
        set_cure_potion(w, id, 0);
    }
    set_strength_potion(w, id, 0);
}

// ---- PlantResourceInfo ----

/// `PlantResourceInfo`: `(plant type, plant hue, resource class)`.
static RESOURCE_LIST: [(i32, i32, &str); 9] = [
    (
        super::plant_type::ELEPHANT_EAR_PLANT,
        hue_c::BRIGHT_RED,
        "Server.Items.RedLeaves",
    ),
    (
        super::plant_type::PONYTAIL_PALM,
        hue_c::BRIGHT_RED,
        "Server.Items.RedLeaves",
    ),
    (
        super::plant_type::CENTURY_PLANT,
        hue_c::BRIGHT_RED,
        "Server.Items.RedLeaves",
    ),
    (
        super::plant_type::POPPIES,
        hue_c::BRIGHT_ORANGE,
        "Server.Items.OrangePetals",
    ),
    (
        super::plant_type::BULRUSHES,
        hue_c::BRIGHT_ORANGE,
        "Server.Items.OrangePetals",
    ),
    (
        super::plant_type::PAMPAS_GRASS,
        hue_c::BRIGHT_ORANGE,
        "Server.Items.OrangePetals",
    ),
    (
        super::plant_type::SNAKE_PLANT,
        hue_c::BRIGHT_GREEN,
        "Server.Items.GreenThorns",
    ),
    (
        super::plant_type::BARREL_CACTUS,
        hue_c::BRIGHT_GREEN,
        "Server.Items.GreenThorns",
    ),
    (
        super::plant_type::COCOA_TREE,
        hue_c::PLAIN,
        "Server.Items.CocoaPulp",
    ),
];

/// `PlantResourceInfo.GetInfo(plantType, plantHue)`: the resource's class name.
pub fn resource_info(plant_type: i32, plant_hue: i32) -> Option<&'static str> {
    RESOURCE_LIST
        .iter()
        .find(|(t, h, _)| *t == plant_type && *h == plant_hue)
        .map(|(_, _, n)| *n)
}

// ---- pollination (PollinateTarget) ----

/// `targ.PlantSystem.Pollinated = true; SeedType/SeedHue = ...` for a cross.
pub fn pollinate(w: &mut World, targ: ItemId, source: ItemId) {
    set_pollinated(w, targ, true);
    if targ == source {
        set_seed_type(w, targ, plant_type(w, source));
        set_seed_hue(w, targ, plant_hue(w, source));
    } else {
        let t = cross_type(w, plant_type(w, source), plant_type(w, targ));
        let h = cross_hue(w, plant_hue(w, source), plant_hue(w, targ));
        set_seed_type(w, targ, t);
        set_seed_hue(w, targ, h);
    }
}

// ---- properties and the class ----

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    fill_properties(w, id, list, false);
}

/// `GetProperties(list)`; `old` is `list == _oldClientPropertyList`.
fn fill_properties(w: &World, id: ItemId, list: &mut PropertyList, old: bool) {
    let Some(d) = data(w, id) else { return };
    if d.status >= status::DEAD_TWIGS {
        (ITEM_BASE.get_properties)(w, id, list);
        return;
    }
    let type_info = PlantTypeInfo::get(d.plant_type);
    let hue_info = PlantHueInfo::get(d.plant_hue);
    let bright = hue_info.is_bright();
    if d.status >= status::DECORATIVE_PLANT {
        list.add_str(
            type_info.plant_label_decorative(bright),
            &format!("#{}\t#{}", hue_info.name, type_info.name()),
        );
        return;
    }
    let container = 1150435; // GetLocalizedContainerType
    let Some(system) = d.system.as_ref() else {
        return;
    };
    let dirt = system.localized_dirt_status();
    let health = system.localized_health(d.status);
    let plant_status = localized_plant_status(w, id);

    if d.status < status::SEED {
        // a ~1_val~ of ~2_val~ dirt
        list.add_str(
            1060830,
            &if old {
                format!("#{dirt}")
            } else {
                format!("#{container}\t#{dirt}")
            },
        );
        return;
    }
    if d.status >= status::FULL_GROWN_PLANT {
        list.add_str(
            type_info.plant_label_full_grown(bright),
            &format!("#{health}\t#{}\t#{}", hue_info.name, type_info.name()),
        );
        return;
    }
    if d.show_type {
        let number = if d.status == status::PLANT {
            type_info.plant_label_plant(bright)
        } else {
            type_info.plant_label_seed(bright)
        };
        let args = if old {
            format!(
                "#{dirt}\t#{health}\t#{}\t#{}\t#{plant_status}",
                hue_info.name,
                type_info.name()
            )
        } else {
            format!(
                "#{container}\t#{dirt}\t#{health}\t#{}\t#{}\t#{plant_status}",
                hue_info.name,
                type_info.name()
            )
        };
        list.add_str(number, &args);
    } else {
        let category = if type_info.plant_category == super::category::DEFAULT {
            hue_info.name
        } else {
            type_info.plant_category
        };
        let number = if bright { 1060832 } else { 1060831 };
        let args = if old {
            format!("#{dirt}\t#{health}\t#{category}\t#{plant_status}")
        } else {
            format!("#{container}\t#{dirt}\t#{health}\t#{category}\t#{plant_status}")
        };
        list.add_str(number, &args);
    }
}

/// `ns.Version < ClientVersion.Version70120` (a client without a version compares lower).
fn is_old_client(w: &World, ns: NetStateId) -> bool {
    let v = w.net_state(ns).and_then(|d| d.caps.version.as_ref());
    ClientVersion::compare(v, Some(&ClientVersion::v70120())).is_lt()
}

/// `InitializePropertyList(list)` on `_oldClientPropertyList`.
fn init_old_list(w: &World, id: ItemId, list: &mut PropertyList) {
    fill_properties(w, id, list, true);
    (ITEM_BASE.append_child_properties)(w, id, list);
    list.terminate();
}

/// The `OldClientPropertyList` getter: built once, `InvalidateProperties` rebuilds it.
pub(crate) fn old_client_list(w: &mut World, id: ItemId) -> Option<&PropertyList> {
    if data(w, id)?.old_list.is_none() {
        let mut list = PropertyList::new(id.0);
        init_old_list(w, id, &mut list);
        data_mut(w, id)?.old_list = Some(list);
    }
    data(w, id)?.old_list.as_ref()
}

fn send_opl_packet_to(w: &mut World, id: ItemId, ns: NetStateId) {
    if !deps::opl_enabled(w) {
        return;
    }
    if is_old_client(w, ns) {
        if let Some(hash) = old_client_list(w, id).map(|l| l.hash()) {
            muo_server::packets::entity::send_opl_info(w, ns, id.0, hash);
        }
        return;
    }
    base::send_opl_packet_to_base(w, id, ns);
}

fn send_properties_to(w: &mut World, id: ItemId, ns: NetStateId) {
    if is_old_client(w, ns) {
        if let Some(buf) = old_client_list(w, id).map(|l| l.buffer().to_vec()) {
            w.send(ns, &buf);
        }
        return;
    }
    base::send_properties_to_base(w, id, ns);
}

fn clear_properties(w: &mut World, id: ItemId) {
    base::clear_properties_base(w, id);
    if let Some(d) = data_mut(w, id) {
        d.old_list = None;
    }
}

fn invalidate_properties(w: &mut World, id: ItemId) {
    base::invalidate_properties_base(w, id);
    if !deps::opl_enabled(w) {
        return;
    }
    let Some(i) = w.item(id) else { return };
    if i.map.is_some_and(|m| m != muo_server::map::MapId::INTERNAL) && !w.loading() {
        let (old_hash, new_hash);
        if let Some(mut list) = data_mut(w, id).and_then(|d| d.old_list.take()) {
            old_hash = Some(list.hash());
            list.reset();
            init_old_list(w, id, &mut list);
            new_hash = list.hash();
            if let Some(d) = data_mut(w, id) {
                d.old_list = Some(list);
            }
        } else {
            old_hash = None;
            new_hash = old_client_list(w, id).map_or(0, |l| l.hash());
        }
        if old_hash != Some(new_hash) {
            ops::delta(w, id, muo_server::item::ItemDelta::PROPERTIES);
        }
    } else {
        clear_properties(w, id);
    }
}

fn on_aos_single_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(ns) = deps::net_state_of(w, from) else {
        return;
    };
    let (header, args) = if is_old_client(w, ns) {
        match old_client_list(w, id) {
            Some(l) => (l.header, l.header_args.clone().unwrap_or_default()),
            None => return,
        }
    } else {
        match ops::property_list(w, id) {
            Some(l) => (l.header, l.header_args.clone().unwrap_or_default()),
            None => return,
        }
    };
    if header > 0 {
        let name = ops::name(w, id).unwrap_or_default();
        deps::label_localized(w, ns, id, header, &name, &args);
    }
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if plant_status(w, id) >= status::DECORATIVE_PLANT {
        return;
    }
    if !ops::is_child_of(w, id, Some(EntityId::Mobile(from))) {
        let loc = ops::get_world_location(w, id);
        if !deps::in_los_point(w, from, loc) || !muo_content::mx::in_range_point(w, from, loc, 2) {
            deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3E9, 1019045); // I can't reach that
            return;
        }
    }
    if !is_usable_by(w, id, from) {
        label(w, id, from, 1061856);
        return;
    }
    super::gumps::display_main(w, from, id);
}

fn read_system(
    r: &mut dyn muo_io::serialization::GenericReader,
    w: &World,
    status_now: i32,
) -> muo_io::Result<PlantSystem> {
    let mut sys = PlantSystem::new(w.now, status_now);
    let version = r.read_i32()?;
    if version < 3 {
        sys.fertile_dirt = r.read_bool()?;
        sys.next_growth = DateTime::from_ticks(r.read_date_time_ticks()?);
        sys.growth_indicator = r.read_i32()?;
        sys.water = r.read_i32()?;
        sys.hits = r.read_i32()?;
        sys.infestation = r.read_i32()?;
        sys.fungus = r.read_i32()?;
        sys.poison = r.read_i32()?;
        sys.disease = r.read_i32()?;
        sys.poison_potion = r.read_i32()?;
        sys.cure_potion = r.read_i32()?;
        sys.heal_potion = r.read_i32()?;
        sys.strength_potion = r.read_i32()?;
        sys.pollinated = r.read_bool()?;
        sys.seed_type = r.read_i32()?;
        sys.seed_hue = r.read_i32()?;
        sys.available_seeds = r.read_i32()?;
        sys.left_seeds = r.read_i32()?;
        sys.available_resources = r.read_i32()?;
        sys.left_resources = r.read_i32()?;
        return Ok(sys);
    }
    let flags = r.read_enum(4)? as u32;
    sys.fertile_dirt = flags & 0x1 != 0;
    sys.next_growth = DateTime::from_ticks(r.read_date_time_ticks()?);
    if flags & 0x2 != 0 {
        sys.growth_indicator = r.read_enum(4)? as i32;
    }
    for (bit, field) in [
        (0x4, &mut sys.water),
        (0x8, &mut sys.hits),
        (0x10, &mut sys.infestation),
        (0x20, &mut sys.fungus),
        (0x40, &mut sys.poison),
        (0x80, &mut sys.disease),
        (0x100, &mut sys.poison_potion),
        (0x200, &mut sys.cure_potion),
        (0x400, &mut sys.heal_potion),
        (0x800, &mut sys.strength_potion),
    ] {
        if flags & bit != 0 {
            *field = r.read_i32()?;
        }
    }
    sys.pollinated = flags & 0x1000 != 0;
    if flags & 0x2000 != 0 {
        sys.seed_type = r.read_enum(4)? as i32;
    }
    if flags & 0x4000 != 0 {
        sys.seed_hue = r.read_enum(4)? as i32;
    }
    if flags & 0x8000 != 0 {
        sys.available_seeds = r.read_i32()?;
    }
    sys.left_seeds = if flags & 0x10000 != 0 {
        r.read_i32()?
    } else {
        8
    };
    if flags & 0x20000 != 0 {
        sys.available_resources = r.read_i32()?;
    }
    sys.left_resources = if flags & 0x40000 != 0 {
        r.read_i32()?
    } else {
        8
    };
    Ok(sys)
}

fn write_system(s: &PlantSystem, out: &mut dyn muo_io::serialization::GenericWriter) {
    out.write_i32(3);
    let mut flags = 0u32;
    for (bit, on) in [
        (0x1, s.fertile_dirt),
        (0x2, s.growth_indicator != 0),
        (0x4, s.water != 0),
        (0x8, s.hits != 0),
        (0x10, s.infestation != 0),
        (0x20, s.fungus != 0),
        (0x40, s.poison != 0),
        (0x80, s.disease != 0),
        (0x100, s.poison_potion != 0),
        (0x200, s.cure_potion != 0),
        (0x400, s.heal_potion != 0),
        (0x800, s.strength_potion != 0),
        (0x1000, s.pollinated),
        (0x2000, s.pollinated),
        (0x4000, s.pollinated),
        (0x8000, s.available_seeds != 0),
        (0x10000, s.left_seeds != 8),
        (0x20000, s.available_resources != 0),
        (0x40000, s.left_resources != 8),
    ] {
        if on {
            flags |= bit;
        }
    }
    out.write_enum(4, i64::from(flags));
    out.write_date_time_ticks(s.next_growth.ticks());
    if flags & 0x2 != 0 {
        out.write_enum(4, i64::from(s.growth_indicator));
    }
    for (bit, v) in [
        (0x4, s.water),
        (0x8, s.hits),
        (0x10, s.infestation),
        (0x20, s.fungus),
        (0x40, s.poison),
        (0x80, s.disease),
        (0x100, s.poison_potion),
        (0x200, s.cure_potion),
        (0x400, s.heal_potion),
        (0x800, s.strength_potion),
    ] {
        if flags & bit != 0 {
            out.write_i32(v);
        }
    }
    if flags & 0x2000 != 0 {
        out.write_enum(4, i64::from(s.seed_type));
    }
    if flags & 0x4000 != 0 {
        out.write_enum(4, i64::from(s.seed_hue));
    }
    for (bit, v) in [
        (0x8000, s.available_seeds),
        (0x10000, s.left_seeds),
        (0x20000, s.available_resources),
        (0x40000, s.left_resources),
    ] {
        if flags & bit != 0 {
            out.write_i32(v);
        }
    }
}

/// `[SerializationGenerator(3, false)] PlantItem`.
pub static PLANT_ITEM: ItemType = ItemType {
    full_name: "Server.Engines.Plants.PlantItem",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| plant_new(w, false)),
    default_weight: |_, _| 1.0,
    force_show_properties: |_, _| true,
    get_properties,
    send_opl_packet_to,
    send_properties_to,
    clear_properties,
    invalidate_properties,
    on_aos_single_click,
    on_single_click: |w, id, from| {
        let Some(d) = data(w, id) else { return };
        let n = if d.status >= status::DEAD_TWIGS {
            (ITEM_BASE.label_number)(w, id) // LabelNumber
        } else if d.status >= status::DECORATIVE_PLANT {
            1061924 // a decorative plant
        } else if d.status >= status::FULL_GROWN_PLANT {
            PlantTypeInfo::get(d.plant_type).name()
        } else {
            1029913 // plant bowl
        };
        label(w, id, from, n);
    },
    on_double_click,
    get_context_menu_entries: |w, id, from, list| {
        base::get_context_menu_entries_base(w, id, from, list);
        if let Some(f) = muo_content::items::common::hooks(w).and_then(|h| h.secure_level_entry)
            && let Some(e) = f(w, from, id)
        {
            list.push(e);
        }
    },
    dupe: |w, id, new_item| {
        // CopyProperties: PlantStatus, PlantType, PlantHue and ShowType; Level and PlantSystem are [IgnoreDupe].
        if let Some(src) = data(w, id).cloned()
            && is_plant(w, new_item)
        {
            set_status(w, new_item, src.status);
            set_plant_type(w, new_item, src.plant_type);
            set_plant_hue(w, new_item, src.plant_hue);
            set_show_type(w, new_item, src.show_type);
        }
        base::dupe_base(w, id, new_item);
    },
    on_after_duped: |w, id, new_item| {
        // `PlantSystem.OnAfterDuped(newItem)`
        let Some(src) = sys(w, id).cloned() else {
            return;
        };
        if !is_plant(w, new_item) || sys(w, new_item).is_none() {
            return;
        }
        set_water(w, new_item, src.water);
        set_hits(w, new_item, src.hits);
        set_infestation(w, new_item, src.infestation);
        set_fungus(w, new_item, src.fungus);
        set_poison_potion(w, new_item, src.poison_potion);
        set_disease(w, new_item, src.disease);
        set_poison_potion(w, new_item, src.poison_potion);
        set_cure_potion(w, new_item, src.cure_potion);
        set_heal_potion(w, new_item, src.heal_potion);
        set_strength_potion(w, new_item, src.strength_potion);
        set_pollinated(w, new_item, src.pollinated);
        set_seed_type(w, new_item, src.seed_type);
        set_seed_hue(w, new_item, src.seed_hue);
        set_available_seeds(w, new_item, src.available_seeds);
        set_left_seeds(w, new_item, src.left_seeds);
        set_available_resources(w, new_item, src.available_resources);
        set_left_resources(w, new_item, src.left_resources);
        set_fertile_dirt(w, new_item, src.fertile_dirt);
        if let Some(n) = sys_mut(w, new_item) {
            n.next_growth = src.next_growth;
            n.growth_indicator = src.growth_indicator;
        }
    },
    on_after_delete: |w, id| {
        (ITEM_BASE.on_after_delete)(w, id);
        w.resources.get_mut::<PlantList>().0.retain(|p| *p != id);
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        let Some(d) = data(w, id) else { return };
        out.write_i32(3);
        let write_system_flag = d.status < status::DECORATIVE_PLANT;
        let mut flags = 0;
        for (bit, on) in [
            (0x1, d.level != 0),
            (0x2, d.status != 0),
            (0x4, d.plant_type != 0),
            (0x8, d.plant_hue != hue_c::NONE),
            (0x10, d.show_type),
            (0x20, write_system_flag),
        ] {
            if on {
                flags |= bit;
            }
        }
        out.write_enum(4, i64::from(flags));
        if flags & 0x1 != 0 {
            out.write_enum(4, i64::from(d.level));
        }
        if flags & 0x2 != 0 {
            out.write_enum(4, i64::from(d.status));
        }
        if flags & 0x4 != 0 {
            out.write_enum(4, i64::from(d.plant_type));
        }
        if flags & 0x8 != 0 {
            out.write_enum(4, i64::from(d.plant_hue));
        }
        if flags & 0x20 != 0
            && let Some(s) = d.system.as_ref()
        {
            write_system(s, out);
        }
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut d = PlantData {
            level: 0,
            status: status::BOWL_OF_DIRT,
            plant_type: 0,
            plant_hue: hue_c::NONE,
            show_type: false,
            system: None,
            old_list: None,
        };
        if version < 3 {
            d.level = r.read_i32()?;
            d.status = r.read_i32()?;
            d.plant_type = r.read_i32()?;
            d.plant_hue = r.read_i32()?;
            d.show_type = r.read_bool()?;
            if d.status < status::DECORATIVE_PLANT {
                d.system = Some(read_system(r, w, d.status)?);
            }
        } else {
            let flags = r.read_enum(4)? as u32;
            if flags & 0x1 != 0 {
                d.level = r.read_enum(4)? as i32;
            }
            if flags & 0x2 != 0 {
                d.status = r.read_enum(4)? as i32;
            }
            if flags & 0x4 != 0 {
                d.plant_type = r.read_enum(4)? as i32;
            }
            if flags & 0x8 != 0 {
                d.plant_hue = r.read_enum(4)? as i32;
            }
            d.show_type = flags & 0x10 != 0;
            if flags & 0x20 != 0 {
                d.system = Some(read_system(r, w, d.status)?);
            }
        }
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        w.resources.get_mut::<PlantList>().0.push(id);
        Ok(())
    },
    ..ITEM_BASE
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&PLANT_ITEM, &[]);
}
