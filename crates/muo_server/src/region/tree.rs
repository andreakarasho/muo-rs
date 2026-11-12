//! The `Region` class.

use indexmap::IndexMap;
use muo_data::expansion::Expansion;
use muo_primitives::dotnet::IgnoreCase;
use muo_primitives::{Direction, Point2D, Point3D, Rectangle2D, Rectangle3D, TimeSpan};

use super::{MusicName, RegionId};
use crate::World;
use crate::ext::Ext;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::interfaces::SpellId;
use crate::map::MapId;
use crate::mobile::AccessLevel;
use crate::skills::SkillName;
use crate::speech_event::SpeechEventArgs;
use crate::target::{TargetId, Targeted};

pub const DEFAULT_PRIORITY: i32 = 50;
pub const MIN_Z: i32 = i8::MIN as i32;
pub const MAX_Z: i32 = i8::MAX as i32 + 1;

/// Properties of a JSON region the common part doesn't read (`Entrance`, `RuneName`..), handed to the type.
pub type RegionJson = serde_json::Map<String, serde_json::Value>;

/// A `Region` class: its name, its parent class, and its virtual members. The defaults of [`REGION_BASE`] are
/// the base region's own: delegate to the parent region, or answer the default.
pub struct RegionType {
    /// The class name; the `$type` of its JSON entries.
    pub name: &'static str,
    pub parent: Option<&'static RegionType>,
    /// The JSON properties beyond `Region`'s own, applied after the region is built (before it registers).
    pub from_json: fn(&mut World, RegionId, &RegionJson) -> Result<(), String>,

    pub default_music: fn(&World, RegionId) -> MusicName,
    pub on_register: fn(&mut World, RegionId),
    pub on_unregister: fn(&mut World, RegionId),
    pub on_child_added: fn(&mut World, RegionId, RegionId),
    pub on_child_removed: fn(&mut World, RegionId, RegionId),
    pub accepts_spawns_from: fn(&World, RegionId, RegionId) -> bool,
    pub on_move_into: fn(&mut World, RegionId, MobileId, Direction, Point3D, Point3D) -> bool,
    pub on_enter: fn(&mut World, RegionId, MobileId),
    pub on_exit: fn(&mut World, RegionId, MobileId),
    pub make_guard: fn(&mut World, RegionId, MobileId),
    /// `GetResource(Type)`: types are their full names here.
    pub get_resource: fn(&World, RegionId, &'static str) -> &'static str,
    pub can_use_stuck_menu: fn(&World, RegionId, MobileId) -> bool,
    pub on_aggressed: fn(&mut World, RegionId, MobileId, MobileId, bool),
    pub on_did_harmful: fn(&mut World, RegionId, MobileId, MobileId),
    pub on_got_harmful: fn(&mut World, RegionId, MobileId, MobileId),
    pub on_location_changed: fn(&mut World, RegionId, MobileId, Point3D),
    pub on_target: fn(&mut World, RegionId, MobileId, TargetId, &Targeted) -> bool,
    pub on_combatant_change:
        fn(&mut World, RegionId, MobileId, Option<MobileId>, Option<MobileId>) -> bool,
    pub allow_housing: fn(&World, RegionId, MobileId, Point3D) -> bool,
    pub send_inaccessible_message: fn(&mut World, RegionId, ItemId, MobileId) -> bool,
    pub check_accessibility: fn(&mut World, RegionId, ItemId, MobileId) -> bool,
    pub on_decay: fn(&mut World, RegionId, ItemId) -> bool,
    pub allow_harmful: fn(&mut World, RegionId, MobileId, MobileId) -> bool,
    pub on_criminal_action: fn(&mut World, RegionId, MobileId, bool),
    pub allow_beneficial: fn(&mut World, RegionId, MobileId, MobileId) -> bool,
    pub on_beneficial_action: fn(&mut World, RegionId, MobileId, MobileId),
    pub on_got_beneficial_action: fn(&mut World, RegionId, MobileId, MobileId),
    pub spell_damage_scalar: fn(&mut World, RegionId, MobileId, MobileId, &mut f64),
    pub on_speech: fn(&mut World, RegionId, &mut SpeechEventArgs),
    /// `AllowGain(m, skill, object)`: the object is the entity being used, if it is one.
    pub allow_gain: fn(&World, RegionId, MobileId, SkillName, Option<EntityId>) -> bool,
    pub on_skill_use: fn(&mut World, RegionId, MobileId, i32) -> bool,
    pub on_begin_spell_cast: fn(&mut World, RegionId, MobileId, SpellId) -> bool,
    pub on_spell_cast: fn(&mut World, RegionId, MobileId, SpellId),
    pub on_resurrect: fn(&mut World, RegionId, MobileId) -> bool,
    pub on_before_death: fn(&mut World, RegionId, MobileId) -> bool,
    pub on_death: fn(&mut World, RegionId, MobileId),
    pub on_damage: fn(&mut World, RegionId, MobileId, &mut i32) -> bool,
    pub on_heal: fn(&mut World, RegionId, MobileId, &mut i32) -> bool,
    pub on_double_click: fn(&mut World, RegionId, MobileId, &Targeted) -> bool,
    pub on_single_click: fn(&mut World, RegionId, MobileId, &Targeted) -> bool,
    pub allow_spawn: fn(&World, RegionId) -> bool,
    pub alter_light_level: fn(&mut World, RegionId, MobileId, &mut i32, &mut i32),
    pub get_logout_delay: fn(&World, RegionId, MobileId) -> TimeSpan,
    /// The class family's own vtable (`BaseRegionType` for `BaseRegion`'s virtuals), inherited with `..PARENT`.
    pub family: Option<&'static (dyn std::any::Any + Send + Sync)>,
}

impl RegionType {
    /// The family vtable of type `F`, if this class has one.
    pub fn family<F: std::any::Any>(&self) -> Option<&'static F> {
        self.family?.downcast_ref()
    }

    /// `typeof(this) is T`: `self` is `other` or derives from it.
    pub fn is(&self, other: &'static RegionType) -> bool {
        let mut t = self;
        loop {
            if std::ptr::eq(t, other) {
                return true;
            }
            match t.parent {
                Some(p) => t = p,
                None => return false,
            }
        }
    }
}

/// The `RegionJsonSerializer._derivedTypes` list: JSON `$type` names to classes.
pub struct RegionRegistry {
    types: IndexMap<&'static str, &'static RegionType>,
}

impl RegionRegistry {
    /// Holds `Region` itself, the polymorphic base.
    pub fn new() -> RegionRegistry {
        let mut types = IndexMap::new();
        types.insert(REGION_BASE.name, &REGION_BASE);
        RegionRegistry { types }
    }

    /// `RegionJsonSerializer.Register<T>()`: an error if the name is already taken.
    pub fn register(&mut self, ty: &'static RegionType) -> Result<(), String> {
        if self.types.contains_key(ty.name) {
            return Err(format!(
                "Type '{}' has already been registered for serialization with the region loader.",
                ty.name
            ));
        }
        self.types.insert(ty.name, ty);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&'static RegionType> {
        self.types.get(name).copied()
    }
}

impl Default for RegionRegistry {
    fn default() -> Self {
        RegionRegistry::new()
    }
}

/// A `Region` instance.
pub struct Region {
    pub id: RegionId,
    pub ty: &'static RegionType,
    /// What a subclass adds.
    pub ext: Ext,
    pub name: Option<String>,
    pub map: MapId,
    pub parent: Option<RegionId>,
    pub children: Vec<RegionId>,
    pub area: Vec<Rectangle3D>,
    /// `Sectors`: the sector coordinates this region was registered into.
    pub sectors: Vec<(i32, i32)>,
    pub dynamic: bool,
    pub priority: i32,
    pub child_level: i32,
    pub registered: bool,
    pub go_location: Point3D,
    pub music: MusicName,
    /// Used during deserialization only.
    pub min_expansion: Expansion,
    pub max_expansion: Expansion,
}

impl Region {
    pub fn parent_ty(&self) -> Option<&'static RegionType> {
        self.ty.parent
    }
}

// ---- the table -----------------------------------------------------------------------------------------

impl World {
    pub fn region(&self, id: RegionId) -> Option<&Region> {
        self.maps.regions.get(&id)
    }

    pub fn region_mut(&mut self, id: RegionId) -> Option<&mut Region> {
        self.maps.regions.get_mut(&id)
    }
}

fn ty_of(w: &World, id: RegionId) -> Option<&'static RegionType> {
    w.region(id).map(|r| r.ty)
}

fn parent_of(w: &World, id: RegionId) -> Option<RegionId> {
    w.region(id).and_then(|r| r.parent)
}

/// `ConvertTo3D(Rectangle2D)`.
pub fn convert_to_3d(rect: Rectangle2D) -> Rectangle3D {
    Rectangle3D::from_points(
        Point3D::from_2d(rect.start, MIN_Z),
        Point3D::from_2d(rect.end, MAX_Z),
    )
}

/// `ConvertTo3D(ReadOnlySpan<Rectangle2D>)`.
pub fn convert_all_to_3d(rects: &[Rectangle2D]) -> Vec<Rectangle3D> {
    rects.iter().copied().map(convert_to_3d).collect()
}

/// `new Region(name, map, parent, priority, area)`: the general constructor. `priority` `None` is the
/// overloads without one (the parent's, or `DefaultPriority`). The region is not registered.
pub fn new_region(
    w: &mut World,
    ty: &'static RegionType,
    name: Option<&str>,
    map: MapId,
    parent: Option<RegionId>,
    priority: Option<i32>,
    area: Vec<Rectangle3D>,
) -> RegionId {
    let id = RegionId(w.maps.next_region_id);
    w.maps.next_region_id += 1;

    let (child_level, base_priority) = match parent.and_then(|p| w.region(p)) {
        Some(p) => (p.child_level + 1, p.priority),
        None => (0, DEFAULT_PRIORITY),
    };

    w.maps.regions.insert(
        id,
        Region {
            id,
            ty,
            ext: Ext::NONE,
            name: name.map(str::to_string),
            map,
            parent,
            children: Vec::new(),
            area,
            sectors: Vec::new(),
            dynamic: true,
            priority: priority.unwrap_or(base_priority),
            child_level,
            registered: false,
            go_location: Point3D::ZERO,
            music: MusicName::Invalid,
            min_expansion: Expansion::None,
            max_expansion: Expansion::EJ,
        },
    );
    // `Music = DefaultMusic` runs in the constructor, after the parent is known.
    let music = (ty.default_music)(w, id);
    if let Some(r) = w.region_mut(id) {
        r.music = music;
    }
    id
}

/// `new Region(name, map, priority, area)` for the base class, with 2D rectangles.
pub fn new_base_region(
    w: &mut World,
    name: Option<&str>,
    map: MapId,
    priority: i32,
    area: &[Rectangle2D],
) -> RegionId {
    new_region(
        w,
        &REGION_BASE,
        name,
        map,
        None,
        Some(priority),
        convert_all_to_3d(area),
    )
}

/// `Map.DefaultRegion`: the region nothing else claims. Created with the map.
pub fn default_region(w: &World, map: Option<MapId>) -> RegionId {
    map.and_then(|m| w.maps.get(m))
        .and_then(|m| m.default_region)
        .unwrap_or(RegionId(0))
}

/// `Map.DefaultRegion` getter, creating the region on first use.
pub fn ensure_default_region(w: &mut World, map: MapId) -> RegionId {
    if let Some(r) = w.maps.get(map).and_then(|m| m.default_region) {
        return r;
    }
    let r = new_region(w, &REGION_BASE, None, map, None, Some(0), Vec::new());
    if let Some(m) = w.maps.get_mut(map) {
        m.default_region = Some(r);
    }
    r
}

/// `Map.DefaultRegion` setter.
pub fn set_default_region(w: &mut World, map: MapId, region: RegionId) {
    if let Some(m) = w.maps.get_mut(map) {
        m.default_region = Some(region);
    }
}

/// `Region.IsDefault`.
pub fn is_default(w: &World, id: RegionId) -> bool {
    w.region(id)
        .is_some_and(|r| w.maps.get(r.map).and_then(|m| m.default_region) == Some(id))
}

/// `Region.CompareTo`: dynamic regions first, then higher priority, then deeper child level.
pub fn compare(a: &Region, b: &Region) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    if a.dynamic {
        if !b.dynamic {
            return Ordering::Less;
        }
    } else if b.dynamic {
        return Ordering::Greater;
    }
    if a.priority != b.priority {
        b.priority.cmp(&a.priority)
    } else {
        b.child_level.cmp(&a.child_level)
    }
}

// ---- lookup --------------------------------------------------------------------------------------------

/// `Region.Find(name, map, insensitive)`. Not optimized: a linear scan of all registered regions.
pub fn find_by_name(w: &World, name: &str, map: MapId, insensitive: bool) -> Option<RegionId> {
    if name.trim().is_empty() {
        return None;
    }
    let lower = name.ic_key();
    for &id in &w.maps.region_list {
        let Some(r) = w.region(id) else { continue };
        if r.map != map {
            continue;
        }
        // A nameless region never matches.
        let Some(r_name) = r.name.as_deref() else {
            continue;
        };
        let matches = if insensitive {
            r_name.ic_key() == lower
        } else {
            r_name == name
        };
        if matches {
            return Some(id);
        }
    }
    None
}

/// `Region.Find(p, map)`: the most specific region at `p`; the default region when none claims it.
pub fn find(w: &World, p: Point3D, map: Option<MapId>) -> RegionId {
    let Some(map) = map else {
        return default_region(w, Some(MapId::INTERNAL));
    };
    let Some(m) = w.maps.get(map) else {
        return RegionId(0);
    };

    for &rid in &m.sector_at(p.x, p.y).regions {
        if contains(w, rid, p) {
            return rid;
        }
    }

    default_region(w, Some(map))
}

/// `Region.Contains(p)`.
pub fn contains(w: &World, id: RegionId, p: Point3D) -> bool {
    w.region(id)
        .is_some_and(|r| r.area.iter().any(|rect| rect.contains(p)))
}

/// `IsChildOf(region)`.
pub fn is_child_of(w: &World, id: RegionId, region: RegionId) -> bool {
    if region == RegionId(0) {
        return false;
    }
    let mut p = parent_of(w, id);
    while let Some(r) = p {
        if r == region {
            return true;
        }
        p = parent_of(w, r);
    }
    false
}

/// `IsPartOf(region)`.
pub fn is_part_of(w: &World, id: RegionId, region: RegionId) -> bool {
    id == region || is_child_of(w, id, region)
}

/// `GetRegion<T>()` / `GetRegion(Type)`: this region or the nearest ancestor of class `ty`.
pub fn get_region_of_type(w: &World, id: RegionId, ty: &'static RegionType) -> Option<RegionId> {
    let mut r = Some(id);
    while let Some(c) = r {
        let rt = ty_of(w, c)?;
        if rt.is(ty) {
            return Some(c);
        }
        r = parent_of(w, c);
    }
    None
}

/// `IsPartOf<T>()`.
pub fn is_part_of_type(w: &World, id: RegionId, ty: &'static RegionType) -> bool {
    get_region_of_type(w, id, ty).is_some()
}

/// `IsPartOf<T>()` for a class of a crate above this one, known by its name (`HouseRaffleRegion`).
pub fn is_part_of_type_name(w: &World, id: RegionId, type_name: &str) -> bool {
    let mut r = Some(id);
    while let Some(c) = r {
        let mut t = ty_of(w, c);
        while let Some(rt) = t {
            if rt.name == type_name {
                return true;
            }
            t = rt.parent;
        }
        r = parent_of(w, c);
    }
    false
}

/// `IsPartOf<T1, T2>()`.
pub fn is_part_of_either(
    w: &World,
    id: RegionId,
    t1: &'static RegionType,
    t2: &'static RegionType,
) -> bool {
    let mut r = Some(id);
    while let Some(c) = r {
        let Some(rt) = ty_of(w, c) else { return false };
        if rt.is(t1) || rt.is(t2) {
            return true;
        }
        r = parent_of(w, c);
    }
    false
}

/// `GetRegion(string regionName, bool caseSensitive = true)`.
pub fn get_region_by_name(
    w: &World,
    id: RegionId,
    region_name: &str,
    case_sensitive: bool,
) -> Option<RegionId> {
    let mut r = Some(id);
    while let Some(c) = r {
        let reg = w.region(c)?;
        let eq = match reg.name.as_deref() {
            Some(n) if case_sensitive => n == region_name,
            Some(n) => n.ic_key() == region_name.ic_key(),
            None => false,
        };
        if eq {
            return Some(c);
        }
        r = reg.parent;
    }
    None
}

/// `IsPartOf(string regionName, bool caseSensitive = false)`.
pub fn is_part_of_name(w: &World, id: RegionId, region_name: &str, case_sensitive: bool) -> bool {
    get_region_by_name(w, id, region_name, case_sensitive).is_some()
}

// ---- registering ---------------------------------------------------------------------------------------

/// `Sector.UpdateMobileRegions`.
fn update_mobile_regions(w: &mut World, map: MapId, sx: i32, sy: i32) {
    let mobiles = w
        .maps
        .get(map)
        .map_or_else(Vec::new, |m| m.sector(sx, sy).mobiles.clone());
    for m in mobiles {
        update_mobile_region(w, m);
    }
}

/// `Mobile.UpdateRegion()`, which the mobile code owns.
pub fn update_mobile_region(w: &mut World, id: MobileId) {
    crate::mobile::part2::effects::update_region(w, id);
}

/// `Sector.OnEnter(region, rect)`.
fn sector_on_enter(w: &mut World, map: MapId, sx: i32, sy: i32, rid: RegionId) {
    let Some(m) = w.maps.get_mut(map) else { return };
    let sec = m.sector_mut(sx, sy);
    if sec.regions.contains(&rid) {
        return;
    }
    let mut list = std::mem::take(&mut sec.regions);
    list.push(rid);
    // List.Sort: the unstable introsort decides ties (same dynamic, priority and level).
    muo_primitives::dotnet::net_sort(&mut list, |a, b| {
        match (w.maps.regions.get(a), w.maps.regions.get(b)) {
            (Some(a), Some(b)) => compare(a, b),
            _ => std::cmp::Ordering::Equal,
        }
    });
    if let Some(m) = w.maps.get_mut(map) {
        m.sector_mut(sx, sy).regions = list;
    }
    update_mobile_regions(w, map, sx, sy);
}

/// `Sector.OnLeave(region)`.
fn sector_on_leave(w: &mut World, map: MapId, sx: i32, sy: i32, rid: RegionId) {
    if let Some(m) = w.maps.get_mut(map) {
        let sec = m.sector_mut(sx, sy);
        if let Some(i) = sec.regions.iter().rposition(|&r| r == rid) {
            sec.regions.remove(i);
        }
    }
    update_mobile_regions(w, map, sx, sy);
}

/// `Region.Register()`. Idempotent.
pub fn register(w: &mut World, id: RegionId) {
    let Some(r) = w.region(id) else { return };
    if r.registered {
        return;
    }
    let (ty, map, parent, area) = (r.ty, r.map, r.parent, r.area.clone());

    (ty.on_register)(w, id);

    if let Some(r) = w.region_mut(id) {
        r.registered = true;
    }

    if let Some(p) = parent {
        if let Some(pr) = w.region_mut(p) {
            pr.children.push(id);
        }
        if let Some(pt) = ty_of(w, p) {
            (pt.on_child_added)(w, p, id);
        }
    }

    w.maps.region_list.push(id);

    // Map.RegisterRegion
    register_in_map(w, id);

    let mut sectors: Vec<(i32, i32)> = Vec::new();

    for rect in &area {
        let Some(m) = w.maps.get(map) else { break };
        let start = m.bound_point(Point2D::new(rect.start.x, rect.start.y));
        let end = m.bound_point(Point2D::new(rect.end.x, rect.end.y));

        let (start_x, start_y) = (
            start.x >> crate::map::SECTOR_SHIFT,
            start.y >> crate::map::SECTOR_SHIFT,
        );
        let (end_x, end_y) = (
            end.x >> crate::map::SECTOR_SHIFT,
            end.y >> crate::map::SECTOR_SHIFT,
        );

        for x in start_x..=end_x {
            for y in start_y..=end_y {
                // Region Areas are approximate and will overlap. Don't add them multiple times!
                if !sectors.contains(&(x, y)) {
                    sector_on_enter(w, map, x, y, id);
                    sectors.push((x, y));
                }
            }
        }
    }

    if let Some(r) = w.region_mut(id) {
        r.sectors = sectors;
    }
}

/// `Map.RegisterRegion(reg)`.
fn register_in_map(w: &mut World, id: RegionId) {
    let Some(r) = w.region(id) else { return };
    let (Some(name), map) = (r.name.clone(), r.map) else {
        return;
    };
    let Some(m) = w.maps.get_mut(map) else { return };
    let key = name.ic_key();
    if m.regions.contains_key(&key) {
        tracing::warn!("Duplicate region name '{}' for map '{}'", name, m.name());
    } else {
        m.regions.insert(key, id);
    }
}

/// `Region.Unregister()`. Idempotent.
pub fn unregister(w: &mut World, id: RegionId) {
    let Some(r) = w.region(id) else { return };
    if !r.registered {
        return;
    }
    let (ty, map, parent, name) = (r.ty, r.map, r.parent, r.name.clone());

    (ty.on_unregister)(w, id);

    if let Some(r) = w.region_mut(id) {
        r.registered = false;
    }

    if w.region(id).is_some_and(|r| !r.children.is_empty()) {
        tracing::warn!(
            "Unregistering region '{}' with children",
            name.as_deref().unwrap_or(ty.name)
        );
    }

    if let Some(p) = parent {
        if let Some(pr) = w.region_mut(p) {
            pr.children.retain(|&c| c != id);
        }
        if let Some(pt) = ty_of(w, p) {
            (pt.on_child_removed)(w, p, id);
        }
    }

    w.maps.region_list.retain(|&r| r != id);

    // Map.UnregisterRegion
    if let (Some(name), Some(m)) = (&name, w.maps.get_mut(map)) {
        m.regions.shift_remove(&name.ic_key());
    }

    let sectors = w
        .region_mut(id)
        .map(|r| std::mem::take(&mut r.sectors))
        .unwrap_or_default();
    for (x, y) in sectors {
        sector_on_leave(w, map, x, y, id);
    }
}

/// Forgets a region that is no longer needed: unregisters and drops it.
pub fn delete_region(w: &mut World, id: RegionId) {
    unregister(w, id);
    w.maps.regions.shift_remove(&id);
}

// ---- who is in a region --------------------------------------------------------------------------------

fn mobile_in(w: &World, m: MobileId, region: RegionId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.region != RegionId(0) && is_part_of(w, m.region, region))
}

fn player_ids(w: &World, id: RegionId) -> Vec<MobileId> {
    let mut v = Vec::new();
    let Some(r) = w.region(id) else { return v };
    let Some(map) = w.maps.get(r.map) else {
        return v;
    };
    for &(x, y) in &r.sectors {
        for &(_, player) in &map.sector(x, y).clients {
            if w.mobile(player).is_some_and(|p| !p.deleted) && mobile_in(w, player, id) {
                v.push(player);
            }
        }
    }
    v
}

/// `GetPlayers()` / `GetPlayersPooled()`: the mobiles with a client in this region or a child.
pub fn get_players(w: &World, id: RegionId) -> Vec<MobileId> {
    player_ids(w, id)
}

/// `GetPlayerCount()`.
pub fn get_player_count(w: &World, id: RegionId) -> usize {
    player_ids(w, id).len()
}

/// `GetMobiles()` / `GetMobilesPooled()`.
pub fn get_mobiles(w: &World, id: RegionId) -> Vec<MobileId> {
    let mut v = Vec::new();
    let Some(r) = w.region(id) else { return v };
    let Some(map) = w.maps.get(r.map) else {
        return v;
    };
    for &(x, y) in &r.sectors {
        for &m in &map.sector(x, y).mobiles {
            if mobile_in(w, m, id) {
                v.push(m);
            }
        }
    }
    v
}

/// `GetMobileCount()`.
pub fn get_mobile_count(w: &World, id: RegionId) -> usize {
    get_mobiles(w, id).len()
}

/// `GetItems()` / `GetItemsPooled()`.
pub fn get_items(w: &World, id: RegionId) -> Vec<ItemId> {
    let mut v = Vec::new();
    let Some(r) = w.region(id) else { return v };
    let Some(map) = w.maps.get(r.map) else {
        return v;
    };
    for &(x, y) in &r.sectors {
        for &i in &map.sector(x, y).items {
            let Some(item) = w.item(i) else { continue };
            if is_part_of(w, find(w, item.location, item.map), id) {
                v.push(i);
            }
        }
    }
    v
}

/// `GetItemCount()`.
pub fn get_item_count(w: &World, id: RegionId) -> usize {
    get_items(w, id).len()
}

/// `Region.ToString()`: the name, else the class name.
pub fn region_to_string(w: &World, id: RegionId) -> String {
    match w.region(id) {
        Some(r) => r.name.clone().unwrap_or_else(|| r.ty.name.to_string()),
        None => String::new(),
    }
}

// ---- movement and region change -----------------------------------------------------------------------

/// `Region.CanMove(m, d, newLocation, oldLocation, map)`.
pub fn can_move(
    w: &mut World,
    m: MobileId,
    d: Direction,
    new_location: Point3D,
    old_location: Point3D,
    map: Option<MapId>,
) -> bool {
    let old_region = w.mobile(m).map_or(RegionId(0), |m| m.region);
    let mut new_region = find(w, new_location, map);

    while old_region != new_region {
        let Some(ty) = ty_of(w, new_region) else {
            return true;
        };
        if !(ty.on_move_into)(w, new_region, m, d, new_location, old_location) {
            return false;
        }

        match parent_of(w, new_region) {
            None => return true,
            Some(p) => new_region = p,
        }
    }

    true
}

/// `Region.OnRegionChange(m, oldRegion, newRegion)`; `RegionId(0)` is the null region.
pub fn on_region_change(w: &mut World, m: MobileId, old_region: RegionId, new_region: RegionId) {
    let has_ns = w.mobile(m).is_some_and(|m| m.net_state.is_some());
    if new_region != RegionId(0) && has_ns {
        if let Some(ty) = w.mobile(m).map(|m| m.ty) {
            (ty.check_light_levels)(w, m, false);
        }

        let new_music = w.region(new_region).map(|r| r.music);
        let old_music = w.region(old_region).map(|r| r.music);
        if (old_region == RegionId(0) || old_music != new_music)
            && let (Some(ns), Some(music)) = (w.mobile(m).and_then(|m| m.net_state), new_music)
        {
            crate::packets::player::send_play_music(w, ns, music);
        }
    }

    let mut old_r = old_region;
    let mut new_r = new_region;

    while old_r != new_r {
        let level = |r: RegionId| {
            if r == RegionId(0) {
                -1
            } else {
                w.region(r).map_or(-1, |r| r.child_level)
            }
        };
        let old_child = level(old_r);
        let new_child = level(new_r);

        if old_child >= new_child {
            if old_r != RegionId(0)
                && let Some(ty) = ty_of(w, old_r)
            {
                (ty.on_exit)(w, old_r, m);
            }
            old_r = parent_of(w, old_r).unwrap_or(RegionId(0));
        }

        if new_child >= old_child {
            if new_r != RegionId(0)
                && let Some(ty) = ty_of(w, new_r)
            {
                (ty.on_enter)(w, new_r, m);
            }
            new_r = parent_of(w, new_r).unwrap_or(RegionId(0));
        }
    }
}

// ---- the virtuals' base behavior -----------------------------------------------------------------------

/// Calls `f` on the parent region if there is one, else `default`.
macro_rules! up {
    ($w:ident, $id:ident, $field:ident ( $($arg:expr),* ), $default:expr) => {
        match parent_of($w, $id) {
            Some(p) => match ty_of($w, p) {
                Some(t) => (t.$field)($w, p, $($arg),*),
                None => $default,
            },
            None => $default,
        }
    };
}

fn default_music(w: &World, id: RegionId) -> MusicName {
    // `Parent?.Music ?? MusicName.Invalid`
    parent_of(w, id)
        .and_then(|p| w.region(p))
        .map_or(MusicName::Invalid, |p| p.music)
}

fn nop(_: &mut World, _: RegionId) {}
fn nop_child(_: &mut World, _: RegionId, _: RegionId) {}
fn nop_mobile(_: &mut World, _: RegionId, _: MobileId) {}

fn accepts_spawns_from(w: &World, id: RegionId, region: RegionId) -> bool {
    (ty_of(w, id).is_none_or(|t| (t.allow_spawn)(w, id)))
        && (region == id
            || parent_of(w, id).is_some_and(|p| {
                ty_of(w, p).is_some_and(|t| (t.accepts_spawns_from)(w, p, region))
            }))
}

fn on_move_into(
    w: &mut World,
    id: RegionId,
    m: MobileId,
    _d: Direction,
    _new: Point3D,
    _old: Point3D,
) -> bool {
    let walk = w.mobile(m).and_then(|m| m.walk_region);
    match walk {
        None => true,
        Some(r) => {
            let ty = ty_of(w, id);
            ty.is_none_or(|t| (t.accepts_spawns_from)(w, id, r))
        }
    }
}

fn make_guard(w: &mut World, id: RegionId, focus: MobileId) {
    if let Some(p) = parent_of(w, id)
        && let Some(t) = ty_of(w, p)
    {
        (t.make_guard)(w, p, focus);
    }
}

fn get_resource(w: &World, id: RegionId, ty: &'static str) -> &'static str {
    up!(w, id, get_resource(ty), ty)
}

fn can_use_stuck_menu(w: &World, id: RegionId, m: MobileId) -> bool {
    up!(w, id, can_use_stuck_menu(m), true)
}

fn on_aggressed(
    w: &mut World,
    id: RegionId,
    aggressor: MobileId,
    aggressed: MobileId,
    criminal: bool,
) {
    up!(w, id, on_aggressed(aggressor, aggressed, criminal), ())
}

fn on_did_harmful(w: &mut World, id: RegionId, harmer: MobileId, harmed: MobileId) {
    up!(w, id, on_did_harmful(harmer, harmed), ())
}

fn on_got_harmful(w: &mut World, id: RegionId, harmer: MobileId, harmed: MobileId) {
    up!(w, id, on_got_harmful(harmer, harmed), ())
}

fn on_location_changed(w: &mut World, id: RegionId, m: MobileId, old: Point3D) {
    up!(w, id, on_location_changed(m, old), ())
}

fn on_target(w: &mut World, id: RegionId, m: MobileId, t: TargetId, o: &Targeted) -> bool {
    up!(w, id, on_target(m, t, o), true)
}

fn on_combatant_change(
    w: &mut World,
    id: RegionId,
    m: MobileId,
    old: Option<MobileId>,
    new: Option<MobileId>,
) -> bool {
    up!(w, id, on_combatant_change(m, old, new), true)
}

fn allow_housing(w: &World, id: RegionId, from: MobileId, p: Point3D) -> bool {
    up!(w, id, allow_housing(from, p), true)
}

fn send_inaccessible_message(w: &mut World, id: RegionId, item: ItemId, from: MobileId) -> bool {
    up!(w, id, send_inaccessible_message(item, from), false)
}

fn check_accessibility(w: &mut World, id: RegionId, item: ItemId, from: MobileId) -> bool {
    up!(w, id, check_accessibility(item, from), true)
}

fn on_decay(w: &mut World, id: RegionId, item: ItemId) -> bool {
    up!(w, id, on_decay(item), true)
}

fn allow_harmful(w: &mut World, id: RegionId, from: MobileId, target: MobileId) -> bool {
    let handler = w.maps.settings.allow_harmful_handler;
    up!(
        w,
        id,
        allow_harmful(from, target),
        handler.is_none_or(|h| h(w, from, target))
    )
}

fn on_criminal_action(w: &mut World, id: RegionId, m: MobileId, message: bool) {
    match parent_of(w, id) {
        Some(p) => {
            if let Some(t) = ty_of(w, p) {
                (t.on_criminal_action)(w, p, m, message);
            }
        }
        None => {
            if message {
                // You've committed a criminal act!!
                crate::mobile::part2::messages::send_localized_message(w, m, 1005040, "", 0x3B2);
            }
        }
    }
}

fn allow_beneficial(w: &mut World, id: RegionId, from: MobileId, target: MobileId) -> bool {
    let handler = w.maps.settings.allow_beneficial_handler;
    up!(
        w,
        id,
        allow_beneficial(from, target),
        handler.is_none_or(|h| h(w, from, target))
    )
}

fn on_beneficial_action(w: &mut World, id: RegionId, helper: MobileId, target: MobileId) {
    up!(w, id, on_beneficial_action(helper, target), ())
}

fn on_got_beneficial_action(w: &mut World, id: RegionId, helper: MobileId, target: MobileId) {
    up!(w, id, on_got_beneficial_action(helper, target), ())
}

fn spell_damage_scalar(
    w: &mut World,
    id: RegionId,
    caster: MobileId,
    target: MobileId,
    damage: &mut f64,
) {
    up!(w, id, spell_damage_scalar(caster, target, damage), ())
}

fn on_speech(w: &mut World, id: RegionId, args: &mut SpeechEventArgs) {
    up!(w, id, on_speech(args), ())
}

fn allow_gain(
    w: &World,
    id: RegionId,
    m: MobileId,
    skill: SkillName,
    obj: Option<EntityId>,
) -> bool {
    up!(w, id, allow_gain(m, skill, obj), true)
}

fn on_skill_use(w: &mut World, id: RegionId, m: MobileId, skill: i32) -> bool {
    up!(w, id, on_skill_use(m, skill), true)
}

fn on_begin_spell_cast(w: &mut World, id: RegionId, m: MobileId, s: SpellId) -> bool {
    up!(w, id, on_begin_spell_cast(m, s), true)
}

fn on_spell_cast(w: &mut World, id: RegionId, m: MobileId, s: SpellId) {
    up!(w, id, on_spell_cast(m, s), ())
}

fn on_resurrect(w: &mut World, id: RegionId, m: MobileId) -> bool {
    up!(w, id, on_resurrect(m), true)
}

fn on_before_death(w: &mut World, id: RegionId, m: MobileId) -> bool {
    up!(w, id, on_before_death(m), true)
}

fn on_death(w: &mut World, id: RegionId, m: MobileId) {
    up!(w, id, on_death(m), ())
}

fn on_damage(w: &mut World, id: RegionId, m: MobileId, damage: &mut i32) -> bool {
    up!(w, id, on_damage(m, damage), true)
}

fn on_heal(w: &mut World, id: RegionId, m: MobileId, heal: &mut i32) -> bool {
    up!(w, id, on_heal(m, heal), true)
}

fn on_double_click(w: &mut World, id: RegionId, m: MobileId, o: &Targeted) -> bool {
    up!(w, id, on_double_click(m, o), true)
}

fn on_single_click(w: &mut World, id: RegionId, m: MobileId, o: &Targeted) -> bool {
    up!(w, id, on_single_click(m, o), true)
}

fn allow_spawn(w: &World, id: RegionId) -> bool {
    up!(w, id, allow_spawn(), true)
}

fn alter_light_level(
    w: &mut World,
    id: RegionId,
    m: MobileId,
    global: &mut i32,
    personal: &mut i32,
) {
    up!(w, id, alter_light_level(m, global, personal), ())
}

fn get_logout_delay(w: &World, id: RegionId, m: MobileId) -> TimeSpan {
    match parent_of(w, id) {
        Some(p) => match ty_of(w, p) {
            Some(t) => (t.get_logout_delay)(w, p, m),
            None => TimeSpan::ZERO,
        },
        None => {
            if w.mobile(m)
                .is_some_and(|m| m.access_level > AccessLevel::Player)
            {
                w.maps.settings.staff_logout_delay
            } else {
                w.maps.settings.default_logout_delay
            }
        }
    }
}

fn from_json(_: &mut World, _: RegionId, _: &RegionJson) -> Result<(), String> {
    Ok(())
}

/// `Region`: every virtual at its base behavior.
pub static REGION_BASE: RegionType = RegionType {
    name: "Region",
    parent: None,
    from_json,
    default_music,
    on_register: nop,
    on_unregister: nop,
    on_child_added: nop_child,
    on_child_removed: nop_child,
    accepts_spawns_from,
    on_move_into,
    on_enter: nop_mobile,
    on_exit: nop_mobile,
    make_guard,
    get_resource,
    can_use_stuck_menu,
    on_aggressed,
    on_did_harmful,
    on_got_harmful,
    on_location_changed,
    on_target,
    on_combatant_change,
    allow_housing,
    send_inaccessible_message,
    check_accessibility,
    on_decay,
    allow_harmful,
    on_criminal_action,
    allow_beneficial,
    on_beneficial_action,
    on_got_beneficial_action,
    spell_damage_scalar,
    on_speech,
    allow_gain,
    on_skill_use,
    on_begin_spell_cast,
    on_spell_cast,
    on_resurrect,
    on_before_death,
    on_death,
    on_damage,
    on_heal,
    on_double_click,
    on_single_click,
    allow_spawn,
    alter_light_level,
    get_logout_delay,
    family: None,
};

/// `Region.AllowSpawn()` through the region's class.
pub fn allow_spawn_of(w: &World, id: RegionId) -> bool {
    ty_of(w, id).is_none_or(|t| (t.allow_spawn)(w, id))
}
