//! `ClosedBarrel`.
// BaseWaterContainer, Bucket, WaterBarrel and WaterTub are in water.rs.

use muo_content::items::cont::{TRAPPABLE_CONTAINER, new_trappable};
use muo_server::item::ItemType;

/// `Server.Items.ClosedBarrel`: `TrappableContainer(0x0FAE)`, `DefaultGumpID => 0x3e`.
pub static CLOSED_BARREL: ItemType = ItemType {
    full_name: "Server.Items.ClosedBarrel",
    parent: Some(&TRAPPABLE_CONTAINER),
    construct: Some(|w| new_trappable(w, &CLOSED_BARREL, 0x0FAE)),
    default_gump_id: crate::class_props::default_gump_id,
    serialize: |w, id, out| {
        (TRAPPABLE_CONTAINER.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (TRAPPABLE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..TRAPPABLE_CONTAINER
};
