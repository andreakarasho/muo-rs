//! Class props of muo_multis content (`Data/Scripts/classes/items/multis`): the deeds' multi and placement offset, and
//! the constant members (`labelNumber`, `defaultWeight`) of the converted classes.

use serde_json::Value;

use muo_content::templates::props;
use muo_primitives::Point3D;
use muo_server::ids::ItemId;
use muo_server::world::World;

/// The deed's `multiId` and `offset` (`[x, y, z]`) props; 0 and the origin when absent.
pub fn deed_props(w: &World, id: ItemId) -> (i32, Point3D) {
    let multi_id = props::as_i32(props::item(w, id, "multiId")).unwrap_or(0);
    let n = |i: usize| {
        props::item(w, id, "offset")
            .and_then(|o| o.get(i))
            .and_then(Value::as_i64)
            .and_then(|v| i32::try_from(v).ok())
            .unwrap_or(0)
    };
    (multi_id, Point3D::new(n(0), n(1), n(2)))
}

pub use muo_content::template_keys::{item_float, item_int, item_name};

/// `BaseBanLocation`: the house class's `baseBanLocation` prop (`[x, y, z]`), else the origin.
pub fn house_ban_location(w: &World, id: ItemId) -> Point3D {
    let n = |i: usize| {
        props::item(w, id, "baseBanLocation")
            .and_then(|o| o.get(i))
            .and_then(Value::as_i64)
            .and_then(|v| i32::try_from(v).ok())
            .unwrap_or(0)
    };
    Point3D::new(n(0), n(1), n(2))
}
