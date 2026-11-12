//! The `Map` data, sector grid, parsing and map registration.
//!
//! The map statics (`Maps`, `AllMaps`, `TileData.ItemTable`, `MultiData`, `Region.Regions`) are
//! [`Maps`], the `World`'s `maps` field. A `Map` is addressed by [`MapId`], which is its slot in `Maps.Maps`
//! (`MapIndex`; Distribution has `MapID == MapIndex`, and saved maps are read back as `Maps[byte]` too).
//!
//! Operations that read items or mobiles are free functions over `&mut World` (the tile matrix loads blocks
//! lazily, so reading a tile needs `&mut`): [`fit`], [`surface`], [`los`], [`sector`], [`enumerate`], [`tiles`].
//! The enumerators are iterators over `&World` yielding ids in a fixed order; collect
//! first to mutate. An `InvalidOperationException` for a list changed mid-enumeration cannot happen: the
//! borrow checker refuses it.
//!
//! shortcut: a sector's lists are `Vec`s, not intrusive links (`Item.Next`/`Previous` stay unused); removing is
//! linear in the sector's population, order is the same as `ValueLinkList` (append, stable removal).

pub mod enumerate;
pub mod fit;
pub mod loader;
pub mod los;
pub mod sector;
pub mod surface;
pub mod tiles;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
#[cfg(test)]
mod tests;

use indexmap::IndexMap;
use muo_data::client::multi_data::{MultiComponentList, MultiData};
use muo_data::tile_matrix::{LandTile, TileMatrixOptions};
use muo_data::{DataDirectories, TileData, TileMatrix};
use muo_primitives::dotnet::IgnoreCase;
use muo_primitives::{Point2D, Point3D};

pub use muo_data::map_definitions::MapRules;
pub use sector::Sector;

use crate::World;
use crate::ids::{ItemId, MobileId};
use crate::item::ItemType;
use crate::region::{Region, RegionId, RegionRegistry};

/// A `Map` reference: its slot in `Map.Maps` (`Maps[0x7F]` is `Map.Internal`).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct MapId(pub u8);

impl MapId {
    /// `Map.Internal`.
    pub const INTERNAL: MapId = MapId(0x7F);
    pub const FELUCCA: MapId = MapId(0);
    pub const TRAMMEL: MapId = MapId(1);
    pub const ILSHENAR: MapId = MapId(2);
    pub const MALAS: MapId = MapId(3);
    pub const TOKUNO: MapId = MapId(4);
    pub const TER_MUR: MapId = MapId(5);
}

bitflags::bitflags! {
    /// Why a spawn position check failed. Used by spawners to decide if caching should be enabled.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct SpawnFailureReason: i32 {
        /// Internal map or out of bounds.
        const INVALID_MAP = 1 << 0;
        /// Region doesn't allow spawning.
        const REGION_BLOCKED = 1 << 1;
        /// Mobile or movable item blocking.
        const TRANSIENT_BLOCKER = 1 << 2;
        /// Static, multi, impassable land, or non-movable item.
        const NON_TRANSIENT_BLOCKER = 1 << 3;
    }
}

pub const SECTOR_SIZE: i32 = 16;
pub const SECTOR_SHIFT: i32 = 4;
pub const SECTOR_ACTIVE_RANGE: i32 = 2;

/// `Map`.
pub struct Map {
    /// `MapID`: the id the client knows.
    pub map_id: i32,
    /// `MapIndex`: the slot in `Maps`.
    pub map_index: i32,
    file_index: i32,
    pub width: i32,
    pub height: i32,
    pub season: i32,
    name: String,
    pub rules: MapRules,
    /// `Regions`: registered regions by name, case-insensitive (keys are lowercased).
    pub regions: IndexMap<String, RegionId>,
    pub(crate) default_region: Option<RegionId>,
    pub(crate) tiles: Option<TileMatrix>,
    sectors_width: i32,
    sectors_height: i32,
    sectors: Vec<Vec<Option<Box<Sector>>>>,
    invalid_sector: Sector,
}

static EMPTY_SECTOR: Sector = Sector::empty(MapId::INTERNAL);

impl Map {
    /// `new Map(...)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        map_id: i32,
        map_index: i32,
        file_index: i32,
        width: i32,
        height: i32,
        season: i32,
        name: &str,
        rules: MapRules,
    ) -> Map {
        let sectors_width = width >> SECTOR_SHIFT;
        let sectors_height = height >> SECTOR_SHIFT;
        let slot = MapId(map_index as u8);
        Map {
            map_id,
            map_index,
            file_index,
            width,
            height,
            season,
            name: name.to_string(),
            rules,
            regions: IndexMap::new(),
            default_region: None,
            tiles: None,
            sectors_width,
            sectors_height,
            sectors: (0..sectors_width.max(0)).map(|_| Vec::new()).collect(),
            invalid_sector: Sector::new(0, 0, slot),
        }
    }

    pub fn id(&self) -> MapId {
        MapId(self.map_index as u8)
    }

    pub fn file_index(&self) -> i32 {
        self.file_index
    }

    pub fn is_internal(&self) -> bool {
        self.id() == MapId::INTERNAL
    }

    /// `Name`; the internal map always reads "Internal".
    pub fn name(&self) -> &str {
        if self.is_internal() {
            "Internal"
        } else {
            &self.name
        }
    }

    /// `Name` setter; the internal map's name cannot change (a warning is logged and it keeps "Internal").
    pub fn set_name(&mut self, value: &str) {
        if self.is_internal() && value != "Internal" {
            tracing::warn!("Attempted to set internal map name to '{value}'");
            self.name = "Internal".to_string();
        } else {
            self.name = value.to_string();
        }
    }

    /// `CompareTo`.
    pub fn compare_to(&self, other: Option<&Map>) -> std::cmp::Ordering {
        match other {
            None => std::cmp::Ordering::Less,
            Some(o) => self.map_id.cmp(&o.map_id),
        }
    }

    /// `Bound(x, y)`: clamps into the map.
    pub fn bound(&self, x: i32, y: i32) -> (i32, i32) {
        (x.clamp(0, self.width - 1), y.clamp(0, self.height - 1))
    }

    pub fn bound_point(&self, p: Point2D) -> Point2D {
        let (x, y) = self.bound(p.x, p.y);
        Point2D::new(x, y)
    }

    /// `CalculateSectors`: the sector rectangle (start x/y, end x/y) `bounds` touches.
    pub fn calculate_sectors(&self, bounds: muo_primitives::Rectangle2D) -> (i32, i32, i32, i32) {
        let (left, top) = self.bound(bounds.start.x, bounds.start.y);
        let (right, bottom) = self.bound(bounds.end.x - 1, bounds.end.y - 1);
        (
            left >> SECTOR_SHIFT,
            top >> SECTOR_SHIFT,
            right >> SECTOR_SHIFT,
            bottom >> SECTOR_SHIFT,
        )
    }

    fn sector_in_range(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.sectors_width && y >= 0 && y < self.sectors_height
    }

    /// `GetRealSector(x, y)` for reading: the sector at sector coordinates, the shared invalid sector outside
    /// the map. A sector nobody touched yet reads as empty.
    pub fn sector(&self, x: i32, y: i32) -> &Sector {
        if !self.sector_in_range(x, y) {
            return &self.invalid_sector;
        }
        match self.sectors[x as usize].get(y as usize) {
            Some(Some(s)) => s,
            _ => &EMPTY_SECTOR,
        }
    }

    /// `InternalGetSector(x, y)`: creates the sector on first use.
    pub fn sector_mut(&mut self, x: i32, y: i32) -> &mut Sector {
        if !self.sector_in_range(x, y) {
            return &mut self.invalid_sector;
        }
        let slot = MapId(self.map_index as u8);
        let h = self.sectors_height as usize;
        let col = &mut self.sectors[x as usize];
        if col.is_empty() {
            col.resize_with(h, || None);
        }
        col[y as usize].get_or_insert_with(|| Box::new(Sector::new(x, y, slot)))
    }

    /// Which sector object `(x, y)` names: `None` is the invalid sector. Equal keys are the same sector.
    pub fn sector_key(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        self.sector_in_range(x, y).then_some((x, y))
    }

    /// `GetSector(x, y)` for reading, world coordinates.
    pub fn sector_at(&self, x: i32, y: i32) -> &Sector {
        self.sector(x >> SECTOR_SHIFT, y >> SECTOR_SHIFT)
    }

    pub fn sectors_width(&self) -> i32 {
        self.sectors_width
    }

    pub fn sectors_height(&self) -> i32 {
        self.sectors_height
    }
}

/// Where a map's files are read from when its tiles are first used (`Map.Tiles` is lazy).
pub struct TileSource {
    pub dirs: DataDirectories,
    pub options: TileMatrixOptions,
}

/// `Region.StaffLogoutDelay`/`DefaultLogoutDelay` and the `Mobile.AllowHarmfulHandler`/`AllowBeneficialHandler`
/// delegates Region's defaults end in.
pub struct MapsSettings {
    pub staff_logout_delay: muo_primitives::TimeSpan,
    pub default_logout_delay: muo_primitives::TimeSpan,
    pub allow_harmful_handler: Option<fn(&mut World, MobileId, MobileId) -> bool>,
    pub allow_beneficial_handler: Option<fn(&mut World, MobileId, MobileId) -> bool>,
}

impl Default for MapsSettings {
    fn default() -> Self {
        MapsSettings {
            staff_logout_delay: muo_primitives::TimeSpan::ZERO,
            default_logout_delay: muo_primitives::TimeSpan::from_mins(5),
            allow_harmful_handler: None,
            allow_beneficial_handler: None,
        }
    }
}

/// `BaseMulti.Components` (virtual): the footprint of a multi item. The default is the client's footprint
/// of its item id; a house class with its own footprint replaces this in `Maps.multi_components`.
pub type MultiComponentsFn = for<'a> fn(&'a World, ItemId) -> &'a MultiComponentList;

fn default_multi_components(w: &World, id: ItemId) -> &MultiComponentList {
    let item_id = w.item(id).map_or(0, |i| i.item_id);
    w.maps.multi_data.get_components(item_id)
}

/// All the statics of `Map`, `TileData`, `MultiData` and `Region`.
pub struct Maps {
    slots: Vec<Option<Map>>,
    /// `Map.AllMaps`, in registration order.
    pub all_maps: Vec<MapId>,
    /// `TileData.LandTable`/`ItemTable`.
    pub tile_data: TileData,
    /// `MultiData`.
    pub multi_data: MultiData,
    /// `Map.InvalidLandTiles`.
    pub invalid_land_tiles: Vec<i32>,
    /// `Map.MaxLOSDistance`.
    pub max_los_distance: i32,
    /// `Core.GlobalMaxUpdateRange`.
    pub global_max_update_range: i32,
    /// Without one, a map's tile matrix is empty (tests, generated maps).
    pub tile_source: Option<TileSource>,
    /// The class of `BaseMulti` (and so of every house, boat..): what goes in a sector's multi list.
    // Set by `item::multi` when the base multi class registers.
    pub base_multi: Option<&'static ItemType>,
    pub multi_components: MultiComponentsFn,
    /// Every `Region`, by id (`Region.Regions` is `region_list`).
    pub regions: IndexMap<RegionId, Region>,
    pub(crate) next_region_id: u32,
    /// `Region.Regions`: the registered ones, in order.
    pub region_list: Vec<RegionId>,
    pub region_types: RegionRegistry,
    pub settings: MapsSettings,
}

impl Default for Maps {
    fn default() -> Self {
        Maps {
            slots: (0..0x100).map(|_| None).collect(),
            all_maps: Vec::new(),
            tile_data: TileData::empty(),
            multi_data: MultiData::default(),
            invalid_land_tiles: vec![0x244],
            max_los_distance: 25,
            global_max_update_range: 24,
            tile_source: None,
            base_multi: None,
            multi_components: default_multi_components,
            regions: IndexMap::new(),
            next_region_id: 1,
            region_list: Vec::new(),
            region_types: RegionRegistry::new(),
            settings: MapsSettings::default(),
        }
    }
}

impl Maps {
    pub fn get(&self, id: MapId) -> Option<&Map> {
        self.slots[usize::from(id.0)].as_ref()
    }

    pub fn get_mut(&mut self, id: MapId) -> Option<&mut Map> {
        self.slots[usize::from(id.0)].as_mut()
    }

    /// `Map.Maps[i] = map; Map.AllMaps.Add(map)` (`MapLoader.RegisterMap`).
    pub fn add(&mut self, map: Map) -> MapId {
        let id = map.id();
        self.slots[usize::from(id.0)] = Some(map);
        self.all_maps.push(id);
        id
    }

    /// `Map.GetMapNames()`.
    pub fn map_names(&self) -> Vec<String> {
        self.slots
            .iter()
            .flatten()
            .map(|m| m.name().to_string())
            .collect()
    }

    /// `Map.GetMapValues()`.
    pub fn map_values(&self) -> Vec<MapId> {
        self.slots.iter().flatten().map(Map::id).collect()
    }

    /// `Map.Parse`/`TryParse`: "Internal", a map index, or a name, case-insensitive, trimmed.
    pub fn parse(&self, s: &str) -> Option<MapId> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        if s.eq_ic("Internal") {
            return self.get(MapId::INTERNAL).map(Map::id);
        }
        let index = match s.parse::<i32>() {
            Ok(127) => return self.get(MapId::INTERNAL).map(Map::id),
            Ok(i) => i,
            Err(_) => -1,
        };
        self.slots
            .iter()
            .flatten()
            .find(|m| (index >= 0 && m.map_index == index) || s.eq_ic(m.name()))
            .map(Map::id)
    }

    /// `Map.Tiles`: opens the map's files on first use. A map whose files fail to open serves an empty matrix.
    pub fn tiles_mut(&mut self, id: MapId) -> Option<&mut TileMatrix> {
        let slot = usize::from(id.0);
        let m = self.slots[slot].as_ref()?;
        if m.tiles.is_none() {
            let (file_index, map_id, width, height) = (m.file_index, m.map_id, m.width, m.height);
            let matrix = match &self.tile_source {
                Some(src) => {
                    TileMatrix::open(&src.dirs, file_index, map_id, width, height, &src.options)
                        .unwrap_or_else(|e| {
                            tracing::error!("Failed to open the tile matrix of map {map_id}: {e}");
                            TileMatrix::new_empty(width, height)
                        })
                }
                None => TileMatrix::new_empty(width, height),
            };
            self.slots[slot].as_mut()?.tiles = Some(matrix);
        }
        self.slots[slot].as_mut()?.tiles.as_mut()
    }

    /// `Tiles.GetLandTile(x, y)`; a missing map reads as the zero tile.
    pub fn land_tile(&mut self, id: MapId, x: i32, y: i32) -> LandTile {
        self.tiles_mut(id)
            .map_or_else(LandTile::default, |t| t.get_land_tile(x, y))
    }

    /// `GetAverageZ(x, y, out z, out avg, out top)`: `(z, avg, top)`.
    pub fn get_average_z_full(&mut self, id: MapId, x: i32, y: i32) -> (i32, i32, i32) {
        let z_top = i32::from(self.land_tile(id, x, y).z);
        let z_left = i32::from(self.land_tile(id, x, y + 1).z);
        let z_right = i32::from(self.land_tile(id, x + 1, y).z);
        let z_bottom = i32::from(self.land_tile(id, x + 1, y + 1).z);

        let z = z_top.min(z_left).min(z_right).min(z_bottom);
        let top = z_top.max(z_left).max(z_right).max(z_bottom);
        let avg = if (z_top - z_bottom).abs() > (z_left - z_right).abs() {
            floor_average(z_left, z_right)
        } else {
            floor_average(z_top, z_bottom)
        };
        (z, avg, top)
    }

    /// `GetAverageZ(x, y)`.
    pub fn get_average_z(&mut self, id: MapId, x: i32, y: i32) -> i32 {
        self.get_average_z_full(id, x, y).1
    }

    /// `Map.GetMultiMinSector`: the sector (key) of the multi's corner. `None` is the invalid sector.
    pub fn multi_min_sector(
        &self,
        id: MapId,
        loc: Point3D,
        mcl: &MultiComponentList,
    ) -> Option<(i32, i32)> {
        self.multi_corner(id, loc.x + mcl.min().x, loc.y + mcl.min().y)
    }

    pub fn multi_max_sector(
        &self,
        id: MapId,
        loc: Point3D,
        mcl: &MultiComponentList,
    ) -> Option<(i32, i32)> {
        self.multi_corner(id, loc.x + mcl.max().x, loc.y + mcl.max().y)
    }

    fn multi_corner(&self, id: MapId, x: i32, y: i32) -> Option<(i32, i32)> {
        let m = self.get(id)?;
        let (bx, by) = m.bound(x, y);
        m.sector_key(bx >> SECTOR_SHIFT, by >> SECTOR_SHIFT)
    }
}

/// `FloorAverage`: halves toward zero after nudging a negative sum.
fn floor_average(a: i32, b: i32) -> i32 {
    let mut v = a + b;
    if v < 0 {
        v -= 1;
    }
    v / 2
}
