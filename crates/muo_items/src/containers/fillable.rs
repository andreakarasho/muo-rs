//! `FillableContainer` and its subclasses, `FillableContent` and `FillableEntry`. `fillable_data.rs` holds the content
//! types.
//!
//! State is [`FillData`] in `Item::ext` next to the lockable container's `ContExt`.

use muo_content::items::cont::{
    self, ContClass, LOCKABLE_CLASS, LOCKABLE_CONTAINER, TrapType, new_lockable,
};
use muo_content::items::gold::KeyType;
use muo_content::loot;
use muo_primitives::{DateTime, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ItemType, ops};
use muo_server::map::MapId;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::fillable_data::{CONTENT_TYPES, NAMES};
use crate::food::beverage::{self, BeverageType};

/// One weighted line of a `FillableContent` (`FillableEntry` / `FillableBvrge`).
pub enum Entry {
    /// `FillableEntry(weight, types)`: `Loot.Construct(types)` (one class here).
    Types {
        weight: i32,
        types: &'static [&'static str],
    },
    /// `FillableEntry(weight, Loot.XTypes, offset, count)`; a negative `count` is the whole list.
    Loot {
        weight: i32,
        list: &'static [&'static str],
        offset: i32,
        count: i32,
    },
    /// `FillableBvrge(weight, type, content)`.
    Bvrge {
        weight: i32,
        ty: &'static str,
        content: &'static str,
    },
}

/// A `FillableContent`.
pub struct Content {
    pub level: i32,
    pub vendors: &'static [&'static str],
    pub entries: &'static [Entry],
}

impl Entry {
    pub fn weight(&self) -> i32 {
        match self {
            Entry::Types { weight, .. }
            | Entry::Loot { weight, .. }
            | Entry::Bvrge { weight, .. } => *weight,
        }
    }

    fn types(&self) -> &'static [&'static str] {
        match self {
            Entry::Types { types, .. } => types,
            Entry::Loot {
                list,
                offset,
                count,
                ..
            } => {
                let o = (*offset as usize).min(list.len());
                let end = if *count < 0 {
                    list.len()
                } else {
                    (o + *count as usize).min(list.len())
                };
                &list[o..end]
            }
            Entry::Bvrge { .. } => &[],
        }
    }
}

/// `FillableEntry.Construct()`.
fn construct_entry(w: &mut World, e: &Entry) -> Option<ItemId> {
    if let Entry::Bvrge { ty, content, .. } = e {
        let kind = BeverageType::from_i32(
            NAME_OF_BEVERAGE
                .iter()
                .position(|n| n == content)
                .unwrap_or(0) as i32,
        );
        // `Utility.Random(_types.Length)` over the single type
        let _ = utility::random(&mut w.rng, 1);
        let item = if matches!(*ty, "Server.Items.BeverageBottle" | "Server.Items.Jug") {
            let id = muo_content::factory::make_item(w, ty)?;
            beverage::set_content(w, id, kind);
            let max = beverage::max_quantity(w, id);
            beverage::set_quantity(w, id, max);
            Some(id)
        } else {
            let id = loot::construct_any(w, &[ty])?;
            if beverage::is_beverage(w, id) {
                beverage::set_content(w, id, kind);
                let max = beverage::max_quantity(w, id);
                beverage::set_quantity(w, id, max);
            }
            Some(id)
        };
        return item;
    }
    let item = loot::construct_any(w, e.types())?;
    let name = w.item(item).map(|i| i.ty);
    let is = |n: &str| {
        name.is_some_and(|t| muo_content::factory::item_is(w, item, n) || t.full_name == n)
    };
    if is("Server.Items.Key") {
        let k = [
            KeyType::Copper,
            KeyType::Gold,
            KeyType::Iron,
            KeyType::Rusty,
        ];
        let i = utility::random(&mut w.rng, 4) as usize;
        crate::simple::item_id(w, item, k[i] as i32);
    } else if is("Server.Items.Arrow") || is("Server.Items.Bolt") {
        let a = utility::random_min_max(&mut w.rng, 2, 6);
        ops::set_amount(w, item, a);
    } else if is("Server.Items.Bandage") || is("Server.Items.Lockpick") {
        let a = utility::random_min_max(&mut w.rng, 1, 3);
        ops::set_amount(w, item, a);
    }
    Some(item)
}

const NAME_OF_BEVERAGE: [&str; 6] = ["Ale", "Cider", "Liquor", "Milk", "Wine", "Water"];

/// `FillableContent.Lookup(type)`.
pub fn lookup(kind: i32) -> Option<&'static Content> {
    usize::try_from(kind)
        .ok()
        .and_then(|i| CONTENT_TYPES.get(i))
}

/// `FillableContent.Construct()`.
pub fn construct_content(w: &mut World, c: &Content) -> Option<ItemId> {
    let total: i32 = c.entries.iter().map(Entry::weight).sum();
    let mut index = utility::random(&mut w.rng, total);
    for e in c.entries {
        if index < e.weight() {
            return construct_entry(w, e);
        }
        index -= e.weight();
    }
    None
}

/// `FillableContent.Acquire(loc, map)`: the content type of the nearest vendor within 20 tiles; -1 for none.
pub fn acquire(w: &World, loc: Point3D, map: Option<MapId>) -> i32 {
    let mut content = -1;
    if map.is_none_or(|m| m == MapId::INTERNAL) {
        return content;
    }
    let dist = |p: Point3D| (f64::from((p.x - loc.x).pow(2) + (p.y - loc.y).pow(2))).sqrt();
    let mut nearest: Option<(muo_server::ids::MobileId, f64)> = None;
    for mob in muo_server::map::enumerate::mobiles_in_range(w, map, loc.x, loc.y, 20) {
        let Some(m) = w.mobile(mob) else { continue };
        let d = dist(m.location);
        if let Some((n, nd)) = nearest {
            let nearest_is_cobbler = w
                .mobile(n)
                .is_some_and(|x| x.ty.full_name == "Server.Mobiles.Cobbler");
            if d > nd && !(nearest_is_cobbler && m.ty.full_name == "Server.Mobiles.Provisioner") {
                continue;
            }
        }
        // `_acquireTable[fill.Vendors[j]] = fill.TypeID`: a later content type wins a shared vendor
        let hit = CONTENT_TYPES
            .iter()
            .rposition(|c| c.vendors.contains(&m.ty.full_name));
        if let Some(i) = hit {
            nearest = Some((mob, d));
            content = i as i32;
        }
    }
    content
}

// ---- FillableContainer ----

/// `FillableContainer._contentType` and `_respawnTimer`.
#[derive(Clone, Copy, Debug)]
pub struct FillData {
    pub content_type: i32,
    pub timer: Option<TimerHandle>,
}

/// The `virtual`s of the fillable containers.
pub struct FillClass {
    pub is_lockable: bool,
    /// `LibraryBookcase`: `GetSpawnCount()` and `AcquireContent()` are overridden.
    pub is_library: bool,
    pub acquire_content: fn(&mut World, ItemId),
}

fn data(w: &World, id: ItemId) -> Option<FillData> {
    w.item(id)?.ext.get::<FillData>().copied()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut FillData> {
    w.item_mut(id)?.ext.get_mut::<FillData>()
}

fn class(w: &World, id: ItemId) -> &'static FillClass {
    w.item(id)
        .and_then(|i| i.ty.family::<FillClass>())
        .unwrap_or(&BASE_FILL_CLASS)
}

fn items_count(w: &World, id: ItemId) -> i32 {
    w.item(id).map_or(0, |i| {
        i.items
            .iter()
            .map(|&c| w.item(c).map_or(0, |x| x.amount))
            .sum()
    })
}

static BASE_FILL_CLASS: FillClass = FillClass {
    is_lockable: true,
    is_library: false,
    acquire_content: acquire_content_base,
};

/// `ContentType`.
pub fn content_type(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(-1, |d| d.content_type)
}

/// `ContentType = value`.
pub fn set_content_type(w: &mut World, id: ItemId, value: i32) {
    if content_type(w, id) == value {
        return;
    }
    clear_contents(w, id);
    if let Some(d) = data_mut(w, id) {
        d.content_type = value;
    }
    respawn(w, id);
}

fn clear_contents(w: &mut World, id: ItemId) {
    let items = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
    for item in items.into_iter().rev() {
        if let Some(ty) = w.item(item).map(|i| i.ty) {
            (ty.delete)(w, item);
        }
    }
}

fn acquire_content_base(w: &mut World, id: ItemId) {
    if content_type(w, id) == -1 {
        let (loc, map) = (
            ops::get_world_location(w, id),
            w.item(id).and_then(|i| i.map),
        );
        let c = acquire(w, loc, map);
        if let Some(d) = data_mut(w, id) {
            d.content_type = c;
        }
    }
    if content_type(w, id) != -1 {
        respawn(w, id);
    }
}

/// `CheckRespawn()`.
pub fn check_respawn(w: &mut World, id: ItemId) {
    if data(w, id)
        .and_then(|d| d.timer)
        .is_some_and(|t| t.running(w))
    {
        return;
    }
    let delay = TimeSpan::from_mins(i64::from(utility::random_min_max(&mut w.rng, 60, 90)));
    let t = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| respawn(w, id));
    if let Some(d) = data_mut(w, id) {
        d.timer = Some(t);
    }
}

fn stop_timer(w: &mut World, id: ItemId) {
    if let Some(mut t) = data_mut(w, id).and_then(|d| d.timer.take()) {
        t.cancel(w);
    }
}

/// `GetSpawnCount()`.
fn spawn_count(w: &mut World, id: ItemId) -> i32 {
    let count = items_count(w, id);
    if class(w, id).is_library {
        let max = 5 - count;
        return if max <= 0 {
            0
        } else {
            utility::random_min_max(&mut w.rng, 1, max)
        };
    }
    if count > 2 {
        return 0;
    }
    let max = (1 + 2 - count) * 2;
    utility::random_min_max(&mut w.rng, 0, max)
}

/// `GenerateContent()`.
fn generate_content(w: &mut World, id: ItemId) {
    let kind = content_type(w, id);
    if kind == -1 || w.item(id).is_none_or(|i| i.deleted()) {
        return;
    }
    let Some(content) = lookup(kind) else { return };
    let to_spawn = spawn_count(w, id);
    for _ in 0..to_spawn {
        let Some(item) = construct_content(w, content) else {
            continue;
        };
        let children = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
        for sub in children {
            let is_container = w
                .item(sub)
                .is_some_and(|s| s.ty.is(&muo_server::item::container::CONTAINER));
            if !is_container {
                let ty = w.item(sub).map(|s| s.ty);
                if let Some(ty) = ty
                    && (ty.stack_with)(w, sub, None, item, false)
                {
                    break;
                }
            }
        }
        if w.item(item).is_some_and(|i| !i.deleted())
            && let Some(ty) = w.item(id).map(|i| i.ty)
        {
            (ty.drop_item)(w, id, item);
        }
    }
}

/// `Respawn()`.
pub fn respawn(w: &mut World, id: ItemId) {
    stop_timer(w, id);
    let kind = content_type(w, id);
    if kind == -1 || w.item(id).is_none_or(|i| i.deleted()) {
        return;
    }
    generate_content(w, id);
    let Some(level) = lookup(kind).map(|c| c.level) else {
        return;
    };
    let lockable = class(w, id).is_lockable;
    if lockable {
        cont::set_locked(w, id, true);
        let difficulty = (level - 1) * 30;
        if let Some(d) = cont::ext_mut(w, id) {
            d.lock_level = difficulty - 10;
            d.max_lock_level = difficulty + 30;
            d.required_skill = difficulty;
        }
    }
    // `IsTrappable` is `IsLockable`
    if lockable && (level > 1 || utility::random(&mut w.rng, 5) < 4) {
        let poison = level > utility::random(&mut w.rng, 5);
        let power = level * utility::random_min_max(&mut w.rng, 10, 30);
        if let Some(d) = cont::ext_mut(w, id) {
            d.trap_type = if poison {
                TrapType::PoisonTrap
            } else {
                TrapType::ExplosionTrap
            };
            d.trap_power = power;
            d.trap_level = level;
        }
    } else if let Some(d) = cont::ext_mut(w, id) {
        d.trap_type = TrapType::None;
        d.trap_power = 0;
        d.trap_level = 0;
    }
}

fn acquire_content(w: &mut World, id: ItemId) {
    (class(w, id).acquire_content)(w, id);
}

fn fill_lock_pick(w: &mut World, id: ItemId, from: muo_server::ids::MobileId) {
    (LOCKABLE_CLASS.lock_pick)(w, id, from);
    check_respawn(w, id);
}

static FILLABLE_CONT_CLASS: ContClass = ContClass {
    lock_pick: fill_lock_pick,
    ..LOCKABLE_CLASS
};

/// `new FillableContainer(itemID)` of class `ty`.
pub fn new_fillable(w: &mut World, ty: &'static ItemType, gid: i32) -> ItemId {
    let id = new_lockable(w, ty, gid);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(FillData {
            content_type: -1,
            timer: None,
        });
    }
    id
}

/// `FillableContentType` names for `[set ContentType`.
pub fn content_type_names() -> &'static [&'static str] {
    NAMES
}

/// `Server.Items.FillableContainer`; `[SerializationGenerator(3, false)]`.
pub static FILLABLE_CONTAINER: ItemType = ItemType {
    full_name: "Server.Items.FillableContainer",
    parent: Some(&LOCKABLE_CONTAINER),
    family: Some(&FILLABLE_CONT_CLASS),
    on_map_change: |w, id| {
        (LOCKABLE_CONTAINER.on_map_change)(w, id);
        acquire_content(w, id);
    },
    on_location_change: |w, id, old| {
        (LOCKABLE_CONTAINER.on_location_change)(w, id, old);
        acquire_content(w, id);
    },
    // OnItemRemoved: `CheckRespawn()` only (it does not call the base)
    on_item_removed: |w, id, _item| check_respawn(w, id),
    on_after_delete: |w, id| {
        (LOCKABLE_CONTAINER.on_after_delete)(w, id);
        stop_timer(w, id);
    },
    serialize: |w, id, out| {
        (LOCKABLE_CONTAINER.serialize)(w, id, out);
        out.write_i32(3);
        out.write_enum(4, i64::from(content_type(w, id))); // WriteEnum<FillableContentType>: an encoded int
        let next = data(w, id)
            .and_then(|d| d.timer)
            .filter(|t| t.running(w))
            .map_or(DateTime::MIN, |t| t.next(w));
        out.write_anchored_time_ticks(next.ticks());
    },
    deserialize: |w, id, r| {
        (LOCKABLE_CONTAINER.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let now = w.now;
        let (content, delay) = if version >= 3 {
            let c = r.read_enum(4)? as i32;
            let t =
                DateTime::try_from_ticks(r.read_anchored_time_ticks()?).unwrap_or(DateTime::MIN);
            (
                c,
                (t != DateTime::MIN).then(|| TimeSpan::from_ticks(t.ticks() - now.ticks())),
            )
        } else {
            // V2Content and the older `Deserialize(reader, version)` both: a content type, then a delta time
            let c = r.read_enum(4)? as i32;
            let t = DateTime::try_from_ticks(r.read_delta_time_ticks(now.ticks())?)
                .unwrap_or(DateTime::MIN);
            (
                c,
                (t != DateTime::MIN).then(|| TimeSpan::from_ticks(t.ticks() - now.ticks())),
            )
        };
        let timer = delay.map(|d| timer::start(w, d, TimeSpan::ZERO, 1, move |w| respawn(w, id)));
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(FillData {
                content_type: content,
                timer,
            });
        }
        // [AfterDeserialization]
        if !data(w, id)
            .and_then(|d| d.timer)
            .is_some_and(|t| t.running(w))
        {
            check_respawn(w, id);
        }
        Ok(())
    },
    ..LOCKABLE_CONTAINER
};

macro_rules! fillables {
    ($($name:ident = $full:literal, $gid:expr, $enc:expr;)*) => {$(
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&FILLABLE_CONTAINER),
            construct: Some(|w| new_fillable(w, &$name, $gid)),
            serialize: |w, id, out| {
                (FILLABLE_CONTAINER.serialize)(w, id, out);
                crate::simple::write_version(out, $enc);
            },
            deserialize: |w, id, r| {
                (FILLABLE_CONTAINER.deserialize)(w, id, r)?;
                let _version = crate::simple::read_version(r, $enc)?;
                Ok(())
            },
            ..FILLABLE_CONTAINER
        };
    )*};
}

fillables! {
    FILLABLE_LARGE_CRATE = "Server.Items.FillableLargeCrate", 0xE3D, true;
    FILLABLE_SMALL_CRATE = "Server.Items.FillableSmallCrate", 0x9A9, true;
    FILLABLE_WOODEN_BOX = "Server.Items.FillableWoodenBox", 0x9AA, true;
    FILLABLE_METAL_BOX = "Server.Items.FillableMetalBox", 0x9A8, true;
    FILLABLE_BARREL = "Server.Items.FillableBarrel", 0xE77, true;
    FILLABLE_METAL_CHEST = "Server.Items.FillableMetalChest", 0x9AB, false;
    FILLABLE_METAL_GOLDEN_CHEST = "Server.Items.FillableMetalGoldenChest", 0xE41, false;
    FILLABLE_WOODEN_CHEST = "Server.Items.FillableWoodenChest", 0xE43, false;
}

fn library_acquire(w: &mut World, id: ItemId) {
    if content_type(w, id) == -1 {
        set_content_type(w, id, 22); // FillableContentType.Library
    }
}

static LIBRARY_CLASS: FillClass = FillClass {
    is_lockable: false,
    is_library: true,
    acquire_content: library_acquire,
};

/// `Server.Items.LibraryBookcase`; `[SerializationGenerator(0)]`.
pub static LIBRARY_BOOKCASE: ItemType = ItemType {
    full_name: "Server.Items.LibraryBookcase",
    parent: Some(&FILLABLE_CONTAINER),
    construct: Some(|w| new_fillable(w, &LIBRARY_BOOKCASE, 0xA97)),
    family: Some(&LIBRARY_CLASS),
    serialize: |w, id, out| {
        (FILLABLE_CONTAINER.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (FILLABLE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..FILLABLE_CONTAINER
};

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&FILLABLE_CONTAINER, &[]);
    for t in [
        &FILLABLE_LARGE_CRATE,
        &FILLABLE_SMALL_CRATE,
        &FILLABLE_WOODEN_BOX,
        &FILLABLE_METAL_BOX,
        &FILLABLE_BARREL,
        &FILLABLE_METAL_CHEST,
        &FILLABLE_METAL_GOLDEN_CHEST,
        &FILLABLE_WOODEN_CHEST,
        &LIBRARY_BOOKCASE,
    ] {
        types.register_item(t, &[]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_table_matches_the_enum() {
        assert_eq!(CONTENT_TYPES.len(), NAMES.len());
        assert_eq!(NAMES[22], "Library");
        assert_eq!(CONTENT_TYPES[0].level, 2); // Weaponsmith
        assert_eq!(CONTENT_TYPES[3].entries.len(), 9); // Alchemist
    }

    #[test]
    fn acquire_picks_nothing_without_a_map() {
        let w = crate::test_world();
        assert_eq!(acquire(&w, Point3D::new(0, 0, 0), None), -1);
    }

    #[test]
    fn an_empty_crate_is_unlocked_and_untrapped_until_it_has_content() {
        let mut w = crate::test_world();
        let id = (FILLABLE_WOODEN_BOX.construct.unwrap())(&mut w);
        assert_eq!(content_type(&w, id), -1);
        assert!(!w.item(id).unwrap().movable());
        respawn(&mut w, id);
        assert!(
            !cont::locked(&w, id),
            "no content type: Respawn returns first"
        );
    }
}
