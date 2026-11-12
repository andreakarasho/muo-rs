//! Primitives every other crate builds on: `Serial`, geometry, `Direction`, enums, hashes, RNG, `Utility`
//! math, `DateTime`/`TimeSpan` ticks.
//!
//! Of the shared interfaces only `IPoint2D`/`IPoint3D` are here; `ICarvable`, `IWeapon`, `ISpell`, `IParty`,
//! `ISpawner`, `ISpawnable` name `Mobile`/`Item`/`Region`/`Map` and are id handles in `muo_server::interfaces`.

pub mod bitmask256;
pub mod body;
pub mod collections;
pub mod comparers;
pub mod datetime;
pub mod direction;
pub mod dotnet;
pub mod geometry;
pub mod hash;
pub mod html;
pub mod net;
pub mod parse;
pub mod path_utility;
pub mod process_memory;
pub mod random;
pub mod serial;
pub mod time_zones;
pub mod timespan;
pub mod utility;

pub use bitmask256::BitMask256;
pub use body::{Body, BodyTable, BodyType};
pub use collections::{OrderedSet, SortedRangeIndex};
pub use datetime::{DateTime, DayOfWeek};
pub use direction::{Direction, MessageType};
pub use geometry::{IPoint2D, IPoint3D, Point2D, Point3D, Point3DList, Rectangle2D, Rectangle3D};
pub use parse::FormatError;
pub use random::{BuiltInRng, Rng, WeightedValue};
pub use serial::Serial;
pub use timespan::{TimeSpan, tick_count};
