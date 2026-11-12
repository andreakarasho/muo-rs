//! Client files and data tables: tiledata, tile matrix, UOP, multis, skill info, cliloc, expansions, JSON config.
//!
//! Nothing here is global: the tables (`TileData.ItemTable`, `MultiData._components`,
//! `ServerConfiguration._settings`, `Localization._localizations`) is an owned value the host keeps (in
//! muo_server's `World` resources) and passes where it is needed.
//!
//! Left to muo_server (they need `Map`, `Item`, `Mobile`, `Region` or reflection):
//! - The `{Map,Type,RegionByName,WorldLocation}Converter` JSON converters: the JSON shapes are documented in
//!   `json::converters` (a map is a name string or an index number; a type is its full name; a region is
//!   `{"Name", "Map"}`); resolving them is the caller's job.
//! - `TileMatrix.GetStaticTiles/GetStaticAndMultiTiles/GetMultiTiles` (`Map.StaticTileEnumerable`) and the `Map`
//!   field of `TileMatrix`.
//! - `Race`, and `Skills` beyond `SkillInfo` (`Mobile` state).
//! - `TileMatrixLoader`, `Core.Expansion`, `ExpansionInfo.GetEraFolder`'s caller wiring, the
//!   `[CallPriority]` startup order of the `Configure()` methods.

pub mod client;
pub mod configuration;
pub mod data_directories;
pub mod error;
pub mod expansion;
pub mod json;
pub mod localization;
pub mod map_definitions;
pub mod skill_info;
pub mod tile_data;
pub mod tile_matrix;

pub use client::{ClientFlags, ClientType, ClientVersion};
mod le;
pub use data_directories::DataDirectories;
pub use error::{DataError, Result};
pub use tile_data::{ItemData, LandData, TileData, TileFlag};
pub use tile_matrix::{LandTile, StaticBlock, StaticTile, TileMatrix, TileMatrixOptions};
