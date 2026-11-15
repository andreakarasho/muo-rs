//! `BaseBeverage` and its leaves, and `EndlessDecanter`.
//!
//! State is [`BevData`] in `Item::ext`; what a leaf overrides (`MaxQuantity`, `Fillable`, `BaseLabelNumber`,
//! `ComputeItemID`, `OnQuantityChanged`, `Pour_OnTarget`) is [`BevClass`], reached through `ty.family`.
//! `Fill_OnTarget` / `Pour_OnTarget` take what other systems own (cows, plants, quests, trough addons, the water
//! elemental) from [`BeverageHooks`]; an unset hook is "that object is not there".
//!
//! The witch's swamp-water quest, the Solen matriarch's water vat and `PlantItem.Pour` are hooks the engines crate
//! sets; `BaseWaterContainer` is `special::water`.

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::map::MapId;
use muo_server::packets::message::MessageType;
use muo_server::poison::PoisonId;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::special::water;

/// `BeverageType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum BeverageType {
    Ale = 0,
    Cider = 1,
    Liquor = 2,
    Milk = 3,
    Wine = 4,
    Water = 5,
}

impl BeverageType {
    /// `(BeverageType)n`; an unknown value is read as `Ale`.
    pub fn from_i32(n: i32) -> BeverageType {
        match n {
            1 => Self::Cider,
            2 => Self::Liquor,
            3 => Self::Milk,
            4 => Self::Wine,
            5 => Self::Water,
            _ => Self::Ale,
        }
    }
}

/// `BeverageType` names for `[add Pitcher Water`.
pub static TYPE_NAMES: &[(&str, i64)] = &[
    ("Ale", 0),
    ("Cider", 1),
    ("Liquor", 2),
    ("Milk", 3),
    ("Wine", 4),
    ("Water", 5),
];

/// What `BaseBeverage` adds to an `Item`.
#[derive(Clone, Debug)]
pub struct BevData {
    pub poison: Option<PoisonId>,
    pub poisoner: Option<MobileId>,
    pub content: BeverageType,
    pub quantity: i32,
    /// `EndlessDecanter`'s `_linked`, `_linkLocation`, `_linkMap`.
    pub linked: bool,
    pub link_location: Point3D,
    pub link_map: Option<MapId>,
}

impl Default for BevData {
    fn default() -> Self {
        BevData {
            poison: None,
            poisoner: None,
            content: BeverageType::Ale,
            quantity: 0,
            linked: false,
            link_location: Point3D::new(0, 0, 0),
            link_map: None,
        }
    }
}

/// The `virtual`s and constants of a beverage class.
#[derive(Copy, Clone)]
pub struct BevClass {
    pub max_quantity: i32,
    pub fillable: bool,
    pub base_label_number: i32,
    /// `EmptyLabelNumber`; 0 is `base.LabelNumber` (the tile name).
    pub empty_label_number: i32,
    pub compute_item_id: fn(&World, ItemId) -> i32,
    pub on_quantity_changed: fn(&mut World, ItemId),
    pub pour_on_target: fn(&mut World, ItemId, MobileId, Targeted),
}

fn class(w: &World, id: ItemId) -> Option<&'static BevClass> {
    w.item(id)?.ty.family::<BevClass>()
}

type CowTryMilkFn = fn(&mut World, MobileId, MobileId) -> Option<bool>;
type PlantPourFn = fn(&mut World, ItemId, MobileId, ItemId) -> bool;
type VatPourFn = fn(&mut World, MobileId, ItemId, ItemId) -> bool;
/// The systems a beverage reaches into.
#[derive(Default)]
pub struct BeverageHooks {
    /// `Cow.TryMilk(from)`; None when the mobile is no cow.
    pub cow_try_milk: Option<CowTryMilkFn>,
    /// `item as IWaterSource` (an addon component's addon too): `Quantity`, None when it is not one.
    pub water_source: Option<fn(&World, ItemId) -> Option<i32>>,
    /// `src.Quantity = value`.
    pub water_source_set: Option<fn(&mut World, ItemId, i32)>,
    /// `targ is PlantItem item` then `item.Pour(from, bev)`: true if it was a plant.
    pub plant_pour: Option<PlantPourFn>,
    /// The witch's swamp water (`LandTarget` tile): true when the objective took it (the beverage is deleted).
    pub swamp_water: Option<fn(&mut World, MobileId, i32) -> bool>,
    /// The matriarch's water vat (`AddonComponent` of a `WaterVatEast/South` with water in the beverage): true if
    /// the target was one.
    pub vat_pour: Option<VatPourFn>,
    /// `EndlessDecanter.HandleThrow` target: the water elemental's `HasDecanter`, None when it is no elemental.
    pub elemental_has_decanter: Option<fn(&World, MobileId) -> Option<bool>>,
    /// `elemental.HasDecanter = false`.
    pub elemental_take_decanter: Option<fn(&mut World, MobileId)>,
}

fn hooks(w: &World) -> Option<&BeverageHooks> {
    w.resources.get::<BeverageHooks>()
}

pub fn data(w: &World, id: ItemId) -> Option<&BevData> {
    w.item(id)?.ext.get::<BevData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut BevData> {
    w.item_mut(id)?.ext.get_mut::<BevData>()
}

/// `item is BaseBeverage`.
pub fn is_beverage(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_BEVERAGE))
}

pub fn quantity(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.quantity)
}

pub fn content(w: &World, id: ItemId) -> BeverageType {
    data(w, id).map_or(BeverageType::Ale, |d| d.content)
}

pub fn max_quantity(w: &World, id: ItemId) -> i32 {
    class(w, id).map_or(1, |c| c.max_quantity)
}

pub fn is_empty(w: &World, id: ItemId) -> bool {
    quantity(w, id) <= 0
}

pub fn is_full(w: &World, id: ItemId) -> bool {
    quantity(w, id) >= max_quantity(w, id)
}

/// `ContainsAlcohol`.
pub fn contains_alcohol(w: &World, id: ItemId) -> bool {
    !is_empty(w, id) && !matches!(content(w, id), BeverageType::Milk | BeverageType::Water)
}

fn compute_item_id(w: &World, id: ItemId) -> i32 {
    class(w, id).map_or(0, |c| (c.compute_item_id)(w, id))
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// An id is `ComputeItemID() > 0` or the beverage is deleted (`OnContentChanged`, the `Quantity` setter).
fn apply_item_id(w: &mut World, id: ItemId) -> bool {
    let gid = compute_item_id(w, id);
    if gid > 0 {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.set_item_id)(w, id, gid);
        }
        true
    } else {
        delete(w, id);
        false
    }
}

/// `Content = value`.
pub fn set_content(w: &mut World, id: ItemId, value: BeverageType) {
    let Some(d) = data_mut(w, id) else { return };
    if d.content == value {
        return;
    }
    d.content = value;
    ops::invalidate_properties(w, id);
    apply_item_id(w, id);
}

/// `Quantity = value`.
pub fn set_quantity(w: &mut World, id: ItemId, value: i32) {
    let max = max_quantity(w, id);
    if let Some(d) = data_mut(w, id) {
        d.quantity = value.clamp(0, max);
    }
    ops::invalidate_properties(w, id);
    if apply_item_id(w, id)
        && let Some(c) = class(w, id)
    {
        (c.on_quantity_changed)(w, id);
    }
}

fn set_poison(w: &mut World, id: ItemId, poison: Option<PoisonId>, poisoner: Option<MobileId>) {
    if let Some(d) = data_mut(w, id) {
        d.poison = poison;
        d.poisoner = poisoner;
    }
}

/// `new BaseBeverage()`: `ItemID = ComputeItemID()`.
pub fn new_empty(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BevData::default());
    }
    let gid = compute_item_id(w, id);
    (ty.set_item_id)(w, id, gid);
    id
}

/// `new BaseBeverage(type)`: full of `type`.
pub fn new_full(w: &mut World, ty: &'static ItemType, kind: BeverageType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0);
    let max = class(w, id).map_or(1, |c| c.max_quantity);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BevData {
            content: kind,
            quantity: max,
            ..BevData::default()
        });
    }
    let gid = compute_item_id(w, id);
    (ty.set_item_id)(w, id, gid);
    id
}

fn label_number(w: &World, id: ItemId) -> i32 {
    let c = class(w, id);
    let base = c.map_or(0, |c| c.base_label_number);
    if is_empty(w, id) || base == 0 {
        match c.map(|c| c.empty_label_number) {
            Some(n) if n != 0 => n,
            _ => (ITEM_BASE.label_number)(w, id),
        }
    } else {
        base + content(w, id) as i32
    }
}

/// `GetQuantityDescription()`.
pub fn quantity_description(w: &World, id: ItemId) -> i32 {
    let max = max_quantity(w, id);
    let pct = quantity(w, id) * 100 / max.max(1);
    match pct {
        i32::MIN..=0 => 1042975,
        1..=33 => 1042974,
        34..=66 => 1042973,
        _ => 1042972,
    }
}

fn show_quantity(w: &World, id: ItemId) -> bool {
    max_quantity(w, id) > 1
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    if show_quantity(w, id) {
        list.add(quantity_description(w, id));
    }
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    (ITEM_BASE.on_single_click)(w, id, from);
    if show_quantity(w, id) {
        let n = quantity_description(w, id);
        ops::label_to_number(w, id, from, n, "");
    }
}

fn local_msg(w: &mut World, from: MobileId, number: i32) {
    deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, number);
}

/// `ValidateUse(from, message)`.
pub fn validate_use(w: &mut World, id: ItemId, from: MobileId, message: bool) -> bool {
    if w.item(id).is_none_or(|i| i.deleted()) {
        return false;
    }
    let fillable = class(w, id).is_some_and(|c| c.fillable);
    let movable = w.item(id).is_some_and(|i| i.movable());
    if !movable && !fillable {
        let house = muo_multis::house::access::find_house_at_item(w, id);
        if !house.is_some_and(|h| muo_multis::house::storage::has_locked_down_item(w, h, id)) {
            if message {
                muo_server::mobile::part2::messages::send_localized_message(
                    w, from, 502946, "", 0x59,
                );
            }
            return false;
        }
    }
    let same_map = w.mobile(from).map(|m| m.map) == w.item(id).map(|i| i.map);
    if same_map
        && deps::mobile_in_range(w, from, ops::get_world_location(w, id), 2)
        && deps::in_los_item(w, from, id)
    {
        return true;
    }
    if message {
        local_msg(w, from, 1019045); // I can't reach that.
    }
    false
}

fn play_sound(w: &mut World, m: MobileId, sound: i32) {
    muo_server::mobile::part2::effects::play_sound(w, m, sound);
}

fn target_item(t: &Targeted) -> Option<ItemId> {
    match t {
        Targeted::Entity(EntityId::Item(i)) => Some(*i),
        _ => None,
    }
}

fn is_mobile_class(w: &World, m: MobileId, name: &str) -> bool {
    muo_content::factory::mobile_is(w, m, name)
}

/// `Fill_OnTarget(from, targ)`.
pub fn fill_on_target(w: &mut World, id: ItemId, from: MobileId, targ: Targeted) {
    let fillable = class(w, id).is_some_and(|c| c.fillable);
    if !is_empty(w, id) || !fillable || !validate_use(w, id, from, false) {
        return;
    }
    let max = max_quantity(w, id);
    if let Some(bev) = target_item(&targ).filter(|&b| is_beverage(w, b)) {
        if is_empty(w, bev) || !validate_use(w, bev, from, true) {
            return;
        }
        let (kind, poison, poisoner) = data(w, bev)
            .map(|d| (d.content, d.poison, d.poisoner))
            .unwrap_or_default_bev();
        set_content(w, id, kind);
        set_poison(w, id, poison, poisoner);
        let theirs = quantity(w, bev);
        if theirs > max {
            set_quantity(w, id, max);
            set_quantity(w, bev, theirs - max);
        } else {
            let mine = quantity(w, id);
            set_quantity(w, id, mine + theirs);
            set_quantity(w, bev, 0);
        }
    } else if let Some(bwc) = target_item(&targ).filter(|&b| {
        w.item(b)
            .is_some_and(|i| i.ty.family::<water::WaterClass>().is_some())
    }) {
        let q = quantity(w, id);
        if q == 0 || (content(w, id) == BeverageType::Water && !is_full(w, id)) {
            let need = (max - q).min(water::quantity(w, bwc));
            if need > 0 && !water::is_empty(w, bwc) && !is_full(w, id) {
                let bq = water::quantity(w, bwc);
                water::set_quantity(w, bwc, bq - need);
                let q = quantity(w, id);
                set_quantity(w, id, q + need);
                set_content(w, id, BeverageType::Water);
                play_sound(w, from, 0x4E);
            }
        }
    } else if let Some(item) = target_item(&targ) {
        let Some(src_q) = hooks(w)
            .and_then(|h| h.water_source)
            .and_then(|f| f(w, item))
            .filter(|&q| q > 0)
        else {
            return;
        };
        let (item_map, from_map) = (
            w.item(item).and_then(|i| i.map),
            w.mobile(from).and_then(|m| m.map),
        );
        if from_map != item_map
            || !deps::mobile_in_range(w, from, ops::get_world_location(w, item), 2)
            || !deps::in_los_item(w, from, item)
        {
            local_msg(w, from, 1019045); // I can't reach that.
            return;
        }
        set_content(w, id, BeverageType::Water);
        set_poison(w, id, None, None);
        let set = hooks(w).and_then(|h| h.water_source_set);
        if src_q > max {
            set_quantity(w, id, max);
            if let Some(f) = set {
                f(w, item, src_q - max);
            }
        } else {
            let mine = quantity(w, id);
            set_quantity(w, id, mine + src_q);
            if let Some(f) = set {
                f(w, item, 0);
            }
        }
        deps::send_localized_message(w, from, 1010089); // You fill the container with water.
    } else if let Targeted::Entity(EntityId::Mobile(cow)) = targ {
        if is_mobile_class(w, cow, "Server.Mobiles.Cow")
            && hooks(w)
                .and_then(|h| h.cow_try_milk)
                .and_then(|f| f(w, cow, from))
                == Some(true)
        {
            set_content(w, id, BeverageType::Milk);
            set_quantity(w, id, max);
            deps::send_localized_message(w, from, 1080197); // You fill the container with milk.
        }
    } else if let Targeted::Land(l) = targ {
        let is_player = is_mobile_class(w, from, "Server.Mobiles.PlayerMobile");
        if is_player
            && hooks(w)
                .and_then(|h| h.swamp_water)
                .is_some_and(|f| f(w, from, l.tile_id))
        {
            delete(w, id);
        }
    }
}

trait UnwrapBev {
    fn unwrap_or_default_bev(self) -> (BeverageType, Option<PoisonId>, Option<MobileId>);
}

impl UnwrapBev for Option<(BeverageType, Option<PoisonId>, Option<MobileId>)> {
    fn unwrap_or_default_bev(self) -> (BeverageType, Option<PoisonId>, Option<MobileId>) {
        self.unwrap_or((BeverageType::Ale, None, None))
    }
}

/// `Pour_OnTarget` of `BaseBeverage`.
pub fn pour_on_target_base(w: &mut World, id: ItemId, from: MobileId, targ: Targeted) {
    if is_empty(w, id) || !validate_use(w, id, from, false) {
        return;
    }
    if let Some(bev) = target_item(&targ).filter(|&b| is_beverage(w, b)) {
        if !validate_use(w, bev, from, true) {
            return;
        }
        let kind = content(w, id);
        if is_full(w, bev) && content(w, bev) == kind {
            deps::send_localized_message(w, from, 500848); // Couldn't pour it there.  It was already full.
        } else if !is_empty(w, bev) {
            deps::send_localized_message(w, from, 500846); // Can't pour it there.
        } else {
            let (poison, poisoner) = data(w, id).map_or((None, None), |d| (d.poison, d.poisoner));
            set_content(w, bev, kind);
            set_poison(w, bev, poison, poisoner);
            let bmax = max_quantity(w, bev);
            let q = quantity(w, id);
            if q > bmax {
                set_quantity(w, bev, bmax);
                set_quantity(w, id, q - bmax);
            } else {
                let bq = quantity(w, bev);
                set_quantity(w, bev, bq + q);
                set_quantity(w, id, 0);
            }
            play_sound(w, from, 0x4E);
        }
    } else if matches!(targ, Targeted::Entity(EntityId::Mobile(m)) if m == from) {
        let thirst = w.mobile(from).map_or(20, |m| m.thirst);
        if thirst < 20 {
            muo_server::mobile::part1::props_a::set_thirst(w, from, thirst + 1);
        }
        if contains_alcohol(w, id) {
            let bac = match content(w, id) {
                BeverageType::Ale => 1,
                BeverageType::Wine => 2,
                BeverageType::Cider => 3,
                BeverageType::Liquor => 4,
                _ => 0,
            };
            if let Some(m) = w.mobile_mut(from) {
                m.bac = (m.bac + bac).min(60);
            }
            check_heave_timer(w, from);
        }
        let s = if utility::random(&mut w.rng, 2) == 0 {
            0x30
        } else {
            0x2D6
        };
        play_sound(w, from, s);
        if let Some((Some(p), poisoner)) = data(w, id).map(|d| (d.poison, d.poisoner)) {
            muo_server::mobile::base::apply_poison(w, from, poisoner, p);
        }
        let q = quantity(w, id);
        set_quantity(w, id, q - 1);
    } else if let Some(bwc) = target_item(&targ).filter(|&b| {
        w.item(b)
            .is_some_and(|i| i.ty.family::<water::WaterClass>().is_some())
    }) {
        if content(w, id) != BeverageType::Water {
            deps::send_localized_message(w, from, 500842); // Can't pour that in there.
        } else if w.item(bwc).is_some_and(|i| !i.items.is_empty()) {
            deps::send_localized_message(w, from, 500841); // That has something in it.
        } else {
            let wmax = w
                .item(bwc)
                .and_then(|i| i.ty.family::<water::WaterClass>())
                .map_or(0, |c| c.max_quantity);
            let needs = (wmax - water::quantity(w, bwc)).min(quantity(w, id));
            if needs > 0 {
                let wq = water::quantity(w, bwc);
                water::set_quantity(w, bwc, wq + needs);
                let q = quantity(w, id);
                set_quantity(w, id, q - needs);
                play_sound(w, from, 0x4E);
            }
        }
    } else if let Some(item) = target_item(&targ) {
        if hooks(w)
            .and_then(|h| h.plant_pour)
            .is_some_and(|f| f(w, item, from, id))
        {
            return;
        }
        if content(w, id) == BeverageType::Water
            && hooks(w)
                .and_then(|h| h.vat_pour)
                .is_some_and(|f| f(w, from, item, id))
        {
            return;
        }
        deps::send_localized_message(w, from, 500846); // Can't pour it there.
    } else {
        deps::send_localized_message(w, from, 500846); // Can't pour it there.
    }
}

fn pour_on_target(w: &mut World, id: ItemId, from: MobileId, targ: Targeted) {
    // `Pourable` is true for every class; `IsEmpty` is checked by the base method.
    if let Some(c) = class(w, id) {
        (c.pour_on_target)(w, id, from, targ);
    }
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let fillable = class(w, id).is_some_and(|c| c.fillable);
    if is_empty(w, id) {
        if !fillable || !validate_use(w, id, from, true) {
            return;
        }
        target::begin_target(w, from, -1, true, TargetFlags::NONE, move |w, from, t| {
            fill_on_target(w, id, from, t)
        });
        muo_server::item::messages::send_localized_message_to(w, id, from, 500837, ""); // Fill from what?
    } else if validate_use(w, id, from, true) {
        target::begin_target(w, from, -1, true, TargetFlags::NONE, move |w, from, t| {
            pour_on_target(w, id, from, t)
        });
        deps::send_localized_message(w, from, 1010086); // What do you want to use this on?
    }
}

/// Every item inside `root`, nested containers included (`FindItemsByType<T>()`).
fn descendants(w: &World, root: ItemId) -> Vec<ItemId> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(c) = stack.pop() {
        for &i in w.item(c).map_or(&[][..], |x| &x.items[..]).iter().rev() {
            out.push(i);
            stack.push(i);
        }
    }
    out
}

/// `BaseBeverage.ConsumeTotal(pack, itemType, content, quantity)`: `item_type` is a class full name.
pub fn consume_total(
    w: &mut World,
    pack: ItemId,
    item_type: &str,
    kind: BeverageType,
    qty: i32,
) -> bool {
    let mut total = 0;
    let mut queue = Vec::new();
    for bev in descendants(w, pack)
        .into_iter()
        .filter(|&b| is_beverage(w, b))
    {
        if muo_content::factory::item_is(w, bev, item_type)
            && content(w, bev) == kind
            && !is_empty(w, bev)
        {
            total += quantity(w, bev);
            queue.push(bev);
        }
    }
    if total >= qty {
        let mut need = qty;
        for bev in queue {
            let theirs = quantity(w, bev);
            if theirs < need {
                set_quantity(w, bev, 0);
                need -= theirs;
            } else {
                set_quantity(w, bev, theirs - need);
                return true;
            }
        }
    }
    false
}

// ---- BAC ----

/// `m_Table`: the drunks and their `HeaveTimer`.
#[derive(Default)]
pub struct DrunkTable(std::collections::HashMap<MobileId, TimerHandle>);

/// `CheckHeaveTimer(from)`.
pub fn check_heave_timer(w: &mut World, from: MobileId) {
    let Some(m) = w.mobile(from) else { return };
    let (bac, internal, deleted) = (m.bac, m.map == Some(MapId::INTERNAL), m.deleted);
    if bac > 0 && !internal && !deleted {
        if w.resources.get_mut::<DrunkTable>().0.contains_key(&from) {
            return;
        }
        if bac > 60
            && let Some(m) = w.mobile_mut(from)
        {
            m.bac = 60;
        }
        let t = timer::start(
            w,
            TimeSpan::from_secs_f64(5.0),
            TimeSpan::from_secs_f64(5.0),
            0,
            move |w| heave_tick(w, from),
        );
        w.resources.get_mut::<DrunkTable>().0.insert(from, t);
    } else if let Some(mut t) = w.resources.get_mut::<DrunkTable>().0.remove(&from) {
        t.cancel(w);
        deps::send_localized_message(w, from, 500850); // You feel sober.
    }
}

fn stop_drunk(w: &mut World, drunk: MobileId) {
    if let Some(mut t) = w.resources.get_mut::<DrunkTable>().0.remove(&drunk) {
        t.cancel(w);
    }
}

fn heave_tick(w: &mut World, drunk: MobileId) {
    let Some(m) = w.mobile(drunk) else {
        stop_drunk(w, drunk);
        return;
    };
    if m.deleted || m.map == Some(MapId::INTERNAL) {
        stop_drunk(w, drunk);
    } else if muo_server::mobile::base::alive(w, drunk) {
        if m.bac > 60 {
            w.mobile_mut(drunk).expect("mobile").bac = 60;
        }
        // chance to get sober
        if utility::random(&mut w.rng, 100) < 10 {
            w.mobile_mut(drunk).expect("mobile").bac -= 1;
        }
        // lose some stats
        let (stam, mana) = w.mobile(drunk).map_or((0, 0), |m| (m.stam, m.mana));
        muo_server::mobile::part1::props_b::set_stam(w, drunk, stam - 1);
        muo_server::mobile::part1::props_b::set_mana(w, drunk, mana - 1);
        if utility::random_from(&mut w.rng, 1, 4) == 1 {
            if !muo_server::mobile::part1::props_b::mounted(w, drunk) {
                // turn in a random direction
                let d = utility::random(&mut w.rng, 8);
                muo_server::mobile::part1::props_b::set_direction(
                    w,
                    drunk,
                    muo_primitives::Direction::from_bits_retain(d as u8),
                );
                // heave
                muo_server::mobile::base::animate(w, drunk, 32, 5, 1, true, false, 0);
            }
            // *hic*
            muo_server::mobile::part2::messages::public_overhead_message_number(
                w,
                drunk,
                muo_primitives::MessageType::REGULAR,
                0x3B2,
                500849,
                "",
                true,
            );
        }
        if w.mobile(drunk).is_some_and(|m| m.bac <= 0) {
            stop_drunk(w, drunk);
            deps::send_localized_message(w, drunk, 500850); // You feel sober.
        }
    }
}

fn on_login(w: &mut World, e: &mut muo_content::network::events::PlayerLogin) {
    check_heave_timer(w, e.0);
}

// ---- BaseBeverage and the leaves ----

const VERSION: i32 = 2;

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(VERSION);
    let d = data(w, id).cloned().unwrap_or_default();
    out.write_poison(d.poison);
    out.write_mobile(w, d.poisoner);
    out.write_enum(4, d.content as i64); // WriteEnum<BeverageType>: an encoded int
    out.write_i32(d.quantity);
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _version = r.read_i32()?; // versions below 2 read the same four fields
    let d = BevData {
        poison: r.read_poison()?,
        poisoner: r.read_mobile(w)?,
        content: BeverageType::from_i32(r.read_enum(4)? as i32),
        quantity: r.read_i32()?,
        ..BevData::default()
    };
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    Ok(())
}

static BASE_BEVERAGE_CLASS: BevClass = BevClass {
    max_quantity: 1,
    fillable: true,
    base_label_number: 0,
    empty_label_number: 0,
    compute_item_id: |_, _| 0,
    on_quantity_changed: |_, _| {},
    pour_on_target: pour_on_target_base,
};

/// `Server.Items.BaseBeverage`; `[SerializationGenerator(2, false)]`.
pub static BASE_BEVERAGE: ItemType = ItemType {
    full_name: "Server.Items.BaseBeverage",
    construct: None,
    parent: Some(&ITEM_BASE),
    family: Some(&BASE_BEVERAGE_CLASS),
    label_number,
    get_properties,
    on_single_click,
    on_double_click,
    serialize,
    deserialize,
    ..ITEM_BASE
};

macro_rules! beverage {
    (
        $name:ident, $class:ident = $full:literal, weight $weight:expr,
        { $($f:ident : $v:expr),* $(,)? },
        ctor $ctor:ident
    ) => {
        static $class: BevClass = BevClass { $($f: $v,)* ..BASE_BEVERAGE_CLASS };
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_BEVERAGE),
            // only the parameterless class is `[Constructible]` here; the typed ones (bottle, jug) need a BeverageType
            construct: match BeverageCtor::$ctor {
                BeverageCtor::Empty => Some(|w| new_empty_or_full(w, &$name, BeverageCtor::$ctor)),
                BeverageCtor::Typed => None,
            },
            make: Some(|w| new_empty_or_full(w, &$name, BeverageCtor::$ctor)),
            family: Some(&$class),
            default_weight: |_, _| $weight,
            serialize: |w, id, out| {
                (BASE_BEVERAGE.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_BEVERAGE.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..BASE_BEVERAGE
        };
    };
}

/// Which constructor `construct` stands for: the parameterless one, or (for `BeverageBottle` and `Jug`, which have
/// only `(BeverageType)`) a full Ale one.
enum BeverageCtor {
    Empty,
    Typed,
}

fn new_empty_or_full(w: &mut World, ty: &'static ItemType, c: BeverageCtor) -> ItemId {
    match c {
        BeverageCtor::Empty => new_empty(w, ty),
        BeverageCtor::Typed => new_full(w, ty, BeverageType::Ale),
    }
}

fn clamp_id(id: i32, lo: i32, hi: i32) -> i32 {
    id.clamp(lo, hi)
}

fn bottle_id(w: &World, id: ItemId) -> i32 {
    if is_empty(w, id) {
        return 0;
    }
    match content(w, id) {
        BeverageType::Ale | BeverageType::Cider => 0x99F,
        BeverageType::Liquor | BeverageType::Milk | BeverageType::Water => 0x99B,
        BeverageType::Wine => 0x9C7,
    }
}

fn gid(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| i.item_id)
}

fn mug_id(w: &World, id: ItemId) -> i32 {
    let cur = gid(w, id);
    if is_empty(w, id) {
        return if (0x1F81..=0x1F84).contains(&cur) {
            cur
        } else {
            0x1F81
        };
    }
    match content(w, id) {
        BeverageType::Ale => {
            if cur == 0x9EF {
                0x9EF
            } else {
                0x9EE
            }
        }
        BeverageType::Cider => clamp_id(cur, 0x1F7D, 0x1F80),
        BeverageType::Liquor => clamp_id(cur, 0x1F85, 0x1F88),
        BeverageType::Milk => clamp_id(cur, 0x1F89, 0x1F8C),
        BeverageType::Wine => clamp_id(cur, 0x1F8D, 0x1F90),
        BeverageType::Water => clamp_id(cur, 0x1F91, 0x1F94),
    }
}

fn pitcher_id(w: &World, id: ItemId) -> i32 {
    let cur = gid(w, id);
    let keep = |a: i32, b: i32| if cur == a { cur } else { b };
    if is_empty(w, id) {
        return if cur == 0x9A7 || cur == 0xFF7 {
            cur
        } else {
            0xFF6
        };
    }
    match content(w, id) {
        BeverageType::Ale => keep(0x1F96, 0x1F95),
        BeverageType::Cider => keep(0x1F98, 0x1F97),
        BeverageType::Liquor => keep(0x1F9A, 0x1F99),
        BeverageType::Milk => keep(0x9AD, 0x9F0),
        BeverageType::Wine => keep(0x1F9C, 0x1F9B),
        BeverageType::Water => {
            if matches!(cur, 0xFF8 | 0xFF9 | 0x1F9E) {
                cur
            } else {
                0x1F9D
            }
        }
    }
}

/// `Pitcher.Pour_OnTarget`: a water elemental is the endless decanter's quest.
fn pitcher_pour(w: &mut World, id: ItemId, from: MobileId, targ: Targeted) {
    let Targeted::Entity(EntityId::Mobile(m)) = targ else {
        return pour_on_target_base(w, id, from, targ);
    };
    let Some(has) = hooks(w)
        .and_then(|h| h.elemental_has_decanter)
        .and_then(|f| f(w, m))
    else {
        return pour_on_target_base(w, id, from, targ);
    };
    if content(w, id) == BeverageType::Water {
        handle_throw(w, id, m, has, from);
    } else {
        deps::send_localized_message(w, from, 500846); // Can't pour it there.
    }
}

/// `EndlessDecanter.HandleThrow(pitcher, elemental, thrower)`.
fn handle_throw(
    w: &mut World,
    pitcher: ItemId,
    elemental: MobileId,
    has_decanter: bool,
    thrower: MobileId,
) {
    if !is_full(w, pitcher) {
        deps::send_localized_message(w, thrower, 1113038); // It is not full.
        return;
    }
    let loc = w.mobile(elemental).map(|m| m.location);
    if !loc.is_some_and(|l| deps::mobile_in_range(w, thrower, l, 5)) {
        deps::send_localized_message(w, thrower, 500295); // You are too far away to do that.
        return;
    }
    if !has_decanter {
        deps::send_localized_message(w, thrower, 1115895); // It seems that this water elemental no longer has a magical decanter...
        return;
    }
    muo_server::mobile::base::revealing_action(w, thrower);
    muo_server::mobile::base::damage(w, elemental, 1, Some(thrower), true, false);
    if 0.1 > utility::random_double(&mut w.rng) {
        if let Some(f) = hooks(w).and_then(|h| h.elemental_take_decanter) {
            f(w, elemental);
        }
        delete(w, pitcher);
        let d = (ENDLESS_DECANTER.construct.expect("set"))(w);
        muo_server::mobile::part2::items::add_to_backpack(w, thrower, d);
        deps::send_localized_message(w, thrower, 1115897); // The water elemental has thrown a magical decanter back to you!
    } else {
        delete(w, pitcher);
        play_sound(w, thrower, 0x040);
        deps::send_localized_message(w, thrower, 1115896); // The water pitcher has shattered.
    }
}

beverage! { BEVERAGE_BOTTLE, BEVERAGE_BOTTLE_CLASS = "Server.Items.BeverageBottle", weight 1.0, {
    max_quantity: 5, fillable: false, base_label_number: 1042959, compute_item_id: |w, id| bottle_id(w, id),
}, ctor Typed }

beverage! { JUG, JUG_CLASS = "Server.Items.Jug", weight 1.0, {
    max_quantity: 10, fillable: false, base_label_number: 1042965,
    compute_item_id: |w, id| if is_empty(w, id) { 0 } else { 0x9C8 },
}, ctor Typed }

beverage! { CERAMIC_MUG, CERAMIC_MUG_CLASS = "Server.Items.CeramicMug", weight 1.0, {
    max_quantity: 1, base_label_number: 1042982,
    compute_item_id: |w, id| {
        let cur = gid(w, id);
        if (!(0x995..=0x999).contains(&cur)) && cur != 0x9CA { 0x995 } else { cur }
    },
}, ctor Empty }

beverage! { PEWTER_MUG, PEWTER_MUG_CLASS = "Server.Items.PewterMug", weight 1.0, {
    max_quantity: 1, base_label_number: 1042994,
    compute_item_id: |w, id| {
        let cur = gid(w, id);
        if (0xFFF..=0x1002).contains(&cur) { cur } else { 0xFFF }
    },
}, ctor Empty }

beverage! { GOBLET, GOBLET_CLASS = "Server.Items.Goblet", weight 1.0, {
    max_quantity: 1, base_label_number: 1043000,
    compute_item_id: |w, id| {
        let cur = gid(w, id);
        if matches!(cur, 0x99A | 0x9B3 | 0x9BF | 0x9CB) { cur } else { 0x99A }
    },
}, ctor Empty }

beverage! { GLASS_MUG, GLASS_MUG_CLASS = "Server.Items.GlassMug", weight 1.0, {
    max_quantity: 5, base_label_number: 1042976, empty_label_number: 1022456, compute_item_id: |w, id| mug_id(w, id),
}, ctor Empty }

beverage! { PITCHER, PITCHER_CLASS = "Server.Items.Pitcher", weight 2.0, {
    max_quantity: 5, base_label_number: 1048128, compute_item_id: |w, id| pitcher_id(w, id),
    pour_on_target: pitcher_pour,
}, ctor Empty }

// ---- EndlessDecanter ----

fn decanter_on_quantity_changed(w: &mut World, id: ItemId) {
    let Some(d) = data(w, id).cloned() else {
        return;
    };
    if !(d.linked && d.content == BeverageType::Water && d.quantity == 0) {
        return;
    }
    let owner = match ops::root_parent(w, id) {
        Some(EntityId::Mobile(m)) => m,
        _ => return,
    };
    let near = w.mobile(owner).is_some_and(|m| m.map == d.link_map)
        && deps::mobile_in_range(w, owner, d.link_location, 10);
    if near {
        let max = max_quantity(w, id);
        set_quantity(w, id, max);
        deps::send_localized_message(w, owner, 1115901); // The decanter has automatically been filled from the linked water trough.
        play_sound(w, owner, 0x4E);
    } else {
        deps::send_localized_message(w, owner, 1115972); // The decanter's refill attempt failed because the linked water trough is not in the area.
    }
}

static ENDLESS_DECANTER_CLASS: BevClass = BevClass {
    compute_item_id: |_, _| 0x0FF6,
    on_quantity_changed: decanter_on_quantity_changed,
    ..PITCHER_CLASS
};

fn decanter_link_target(w: &mut World, id: ItemId, from: MobileId, t: Targeted) {
    let movable = w.item(id).is_some_and(|i| i.movable());
    if w.item(id).is_none_or(|i| i.deleted())
        || !movable
        || !muo_server::mobile::part2::stats::check_alive(w, from, true)
    {
        return;
    }
    let (gid, loc, map) = match t {
        Targeted::Static(s) => (s.item_id, s.location, w.mobile(from).and_then(|m| m.map)),
        Targeted::Entity(EntityId::Item(i)) => {
            let it = w.item(i);
            (
                it.map_or(0, |x| x.item_id),
                it.map_or(Point3D::new(0, 0, 0), |x| x.location),
                it.and_then(|x| x.map),
            )
        }
        _ => {
            deps::send_localized_message(w, from, 1115900); // Invalid target. Please target a water trough.
            return;
        }
    };
    if (0xB41..=0xB44).contains(&gid) {
        if let Some(d) = data_mut(w, id) {
            d.linked = true;
            d.link_location = loc;
            d.link_map = map;
        }
        ops::invalidate_properties(w, id);
        deps::send_localized_message(w, from, 1115899); // That water trough has been linked to this decanter.
        if is_empty(w, id) && content(w, id) == BeverageType::Water {
            decanter_on_quantity_changed(w, id);
        }
    } else {
        deps::send_localized_message(w, from, 1115900); // Invalid target. Please target a water trough.
    }
}

fn decanter_context_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>,
) {
    use muo_server::context_menu_entry::ContextMenuEntry;
    (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
    if !muo_server::mobile::base::alive(w, from) {
        return;
    }
    let mut link = ContextMenuEntry::new(1115891, 0); // Link
    link.on_click = Some(std::rc::Rc::new(move |w, from, _| {
        let movable = w.item(id).is_some_and(|i| i.movable());
        if w.item(id).is_none_or(|i| i.deleted())
            || !movable
            || !muo_server::mobile::part2::stats::check_alive(w, from, true)
            || !muo_server::item::base::check_item_use(w, id, from, id)
        {
            return;
        }
        deps::send_localized_message(w, from, 1115892); // Target a water trough you wish to link.
        target::begin_target(w, from, 10, false, TargetFlags::NONE, move |w, from, t| {
            if !muo_server::item::base::check_item_use(w, id, from, id) {
                return;
            }
            decanter_link_target(w, id, from, t);
        });
    }));
    list.push(link);
    if data(w, id).is_some_and(|d| d.linked) {
        let mut unlink = ContextMenuEntry::new(1115930, 0); // Unlink
        unlink.on_click = Some(std::rc::Rc::new(move |w, from, _| {
            let movable = w.item(id).is_some_and(|i| i.movable());
            if w.item(id).is_none_or(|i| i.deleted())
                || !movable
                || !muo_server::mobile::part2::stats::check_alive(w, from, true)
                || !muo_server::item::base::check_item_use(w, id, from, id)
            {
                return;
            }
            deps::send_localized_message(w, from, 1115898); // The link between this decanter and the water trough has been removed.
            if let Some(d) = data_mut(w, id) {
                d.linked = false;
            }
            ops::invalidate_properties(w, id);
        }));
        list.push(unlink);
    }
}

/// `Server.Items.EndlessDecanter`; `[SerializationGenerator(0)]` (an encoded version).
pub static ENDLESS_DECANTER: ItemType = ItemType {
    full_name: "Server.Items.EndlessDecanter",
    parent: Some(&PITCHER),
    construct: Some(|w| {
        let id = new_full(w, &ENDLESS_DECANTER, BeverageType::Water);
        crate::simple::hue(w, id, 0x399);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    family: Some(&ENDLESS_DECANTER_CLASS),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // Endless Decanter of Water
    get_context_menu_entries: decanter_context_entries,
    get_properties: |w, id, list| {
        get_properties(w, id, list);
        list.add(1115889); // Auto Water Refill
        list.add(if data(w, id).is_some_and(|d| d.linked) {
            1115893
        } else {
            1115894
        }); // Linked / Unlinked
    },
    serialize: |w, id, out| {
        (PITCHER.serialize)(w, id, out);
        out.write_encoded_int(0);
        let d = data(w, id).cloned().unwrap_or_default();
        out.write_bool(d.linked);
        out.write_point3d(d.link_location);
        out.write_map(d.link_map);
    },
    deserialize: |w, id, r| {
        (PITCHER.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let (linked, loc, map) = (r.read_bool()?, r.read_point3d()?, r.read_map()?);
        if let Some(d) = data_mut(w, id) {
            d.linked = linked;
            d.link_location = loc;
            d.link_map = map;
        }
        Ok(())
    },
    ..PITCHER
};

/// `Server.Items.AdmiralsHeartyRum` (Minor Artifacts): `base(BeverageType.Ale)`, `Hue = 0x66C`.
pub static ADMIRALS_HEARTY_RUM: ItemType = ItemType {
    full_name: "Server.Items.AdmiralsHeartyRum",
    parent: Some(&BEVERAGE_BOTTLE),
    construct: Some(|w| {
        let id = new_full(w, &ADMIRALS_HEARTY_RUM, BeverageType::Ale);
        crate::simple::hue(w, id, 0x66C);
        id
    }),
    label_number: crate::class_props::label_number,
    serialize: |w, id, out| {
        (BEVERAGE_BOTTLE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BEVERAGE_BOTTLE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BEVERAGE_BOTTLE
};

// ---- constructors for `[add` ----

use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::PropType;

const TYPE_KIND: Kind = Kind::Plain(PropType::Enum(TYPE_NAMES));
static TYPE_PARAM: [Param; 1] = [Param {
    name: "type",
    kind: TYPE_KIND,
    default: None,
    ty: "BeverageType",
}];

fn kind_of(v: &[Value]) -> BeverageType {
    BeverageType::from_i32(v[0].i32())
}

macro_rules! typed_ctors {
    ($($tbl:ident, $name:ident, $empty:expr;)*) => {$(
        static $tbl: [Ctor; 2] = [
            Ctor { access: None, params: &[], make: |w, _| EntityId::Item(new_empty(w, &$name)) },
            Ctor { access: None, params: &TYPE_PARAM, make: |w, v| EntityId::Item(new_full(w, &$name, kind_of(v))) },
        ];
    )*};
}
typed_ctors! {
    C_CERAMIC_MUG, CERAMIC_MUG, true;
    C_PEWTER_MUG, PEWTER_MUG, true;
    C_GOBLET, GOBLET, true;
    C_GLASS_MUG, GLASS_MUG, true;
    C_PITCHER, PITCHER, true;
}
static C_BEVERAGE_BOTTLE: [Ctor; 1] = [Ctor {
    access: None,
    params: &TYPE_PARAM,
    make: |w, v| EntityId::Item(new_full(w, &BEVERAGE_BOTTLE, kind_of(v))),
}];
static C_JUG: [Ctor; 1] = [Ctor {
    access: None,
    params: &TYPE_PARAM,
    make: |w, v| EntityId::Item(new_full(w, &JUG, kind_of(v))),
}];

fn configure(w: &mut World) {
    muo_content::network::events::subscribe_ranked::<muo_content::network::events::PlayerLogin>(
        w, 11, on_login,
    );
    let tables: [(&str, &'static [Ctor]); 7] = [
        ("Server.Items.BeverageBottle", &C_BEVERAGE_BOTTLE),
        ("Server.Items.Jug", &C_JUG),
        ("Server.Items.CeramicMug", &C_CERAMIC_MUG),
        ("Server.Items.PewterMug", &C_PEWTER_MUG),
        ("Server.Items.Goblet", &C_GOBLET),
        ("Server.Items.GlassMug", &C_GLASS_MUG),
        ("Server.Items.Pitcher", &C_PITCHER),
    ];
    for (name, t) in tables {
        add_ctors(w, name, t);
    }
}

/// Registers the beverage classes. `[TypeAlias]` names are the old per-content classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BASE_BEVERAGE, &[]);
    types.register_item(
        &BEVERAGE_BOTTLE,
        &[
            "Server.Items.BottleAle",
            "Server.Items.BottleLiquor",
            "Server.Items.BottleWine",
        ],
    );
    types.register_item(&JUG, &[]);
    types.register_item(&CERAMIC_MUG, &[]);
    types.register_item(&PEWTER_MUG, &[]);
    types.register_item(&GOBLET, &[]);
    types.register_item(
        &GLASS_MUG,
        &[
            "Server.Items.MugAle",
            "Server.Items.GlassCider",
            "Server.Items.GlassLiquor",
            "Server.Items.GlassMilk",
            "Server.Items.GlassWine",
            "Server.Items.GlassWater",
        ],
    );
    types.register_item(
        &PITCHER,
        &[
            "Server.Items.PitcherAle",
            "Server.Items.PitcherCider",
            "Server.Items.PitcherLiquor",
            "Server.Items.PitcherMilk",
            "Server.Items.PitcherWine",
            "Server.Items.PitcherWater",
            "Server.Items.GlassPitcher",
        ],
    );
    types.register_item(&ENDLESS_DECANTER, &[]);
    types.register_item(&ADMIRALS_HEARTY_RUM, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BaseBeverage.Configure",
        configure,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};

    fn save(w: &World, id: ItemId) -> Vec<u8> {
        let mut out = SaveWriter::new(true);
        (w.item(id).unwrap().ty.serialize)(w, id, &mut out);
        out.into_vec()
    }

    #[test]
    fn pitcher_of_water_is_full_and_saves_the_generated_layout() {
        let mut w = World::new();
        let id = new_full(&mut w, &PITCHER, BeverageType::Water);
        assert_eq!((quantity(&w, id), w.item(id).unwrap().item_id), (5, 0x1F9D));
        assert_eq!((PITCHER.label_number)(&w, id), 1048128 + 5);
        let bytes = save(&w, id);
        let mut want = SaveWriter::new(true);
        (ITEM_BASE.serialize)(&w, id, &mut want);
        want.write_i32(2); // BaseBeverage version
        want.write_bool(false); // no poison
        want.write_mobile(&w, None);
        want.write_encoded_int(5); // Water (an enum: encoded int)
        want.write_i32(5); // Quantity
        want.write_i32(0); // Pitcher version
        assert_eq!(bytes, want.into_vec());
        let id2 = w.add_item(muo_server::item::Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0100),
            &PITCHER,
        ));
        let mut r = SaveReader::new(&bytes);
        (PITCHER.deserialize)(&mut w, id2, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        assert_eq!(save(&w, id2), bytes);
    }

    #[test]
    fn emptying_a_pitcher_deletes_it_but_a_mug_keeps_its_graphic() {
        let mut w = World::new();
        let mug = new_full(&mut w, &CERAMIC_MUG, BeverageType::Ale);
        assert_eq!(w.item(mug).unwrap().item_id, 0x995);
        set_quantity(&mut w, mug, 0);
        assert_eq!(w.item(mug).unwrap().item_id, 0x995);
        let jug = new_full(&mut w, &JUG, BeverageType::Wine);
        assert_eq!(quantity(&w, jug), 10);
        set_quantity(&mut w, jug, 0);
        assert!(
            w.item(jug).is_none_or(|i| i.deleted()),
            "an empty jug computes item id 0 and is deleted"
        );
    }

    #[test]
    fn quantity_description_thresholds() {
        let mut w = World::new();
        let id = new_full(&mut w, &PITCHER, BeverageType::Milk);
        assert_eq!(quantity_description(&w, id), 1042972);
        set_quantity(&mut w, id, 3); // 60%
        assert_eq!(quantity_description(&w, id), 1042973);
        set_quantity(&mut w, id, 1); // 20%
        assert_eq!(quantity_description(&w, id), 1042974);
    }

    #[test]
    fn endless_decanter_constructs_blessed_with_water() {
        let mut w = crate::keyed_world();
        let id = (ENDLESS_DECANTER.construct.unwrap())(&mut w);
        let i = w.item(id).unwrap();
        assert_eq!(
            (i.item_id, i.hue, i.loot_type),
            (0xFF6, 0x399, LootType::Blessed)
        );
        assert_eq!((ENDLESS_DECANTER.label_number)(&w, id), 1115929);
        assert_eq!(content(&w, id), BeverageType::Water);
    }
}
