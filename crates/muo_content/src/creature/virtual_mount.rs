//! `VirtualMountItem`: the invisible mount item a creature that rides wears (layer Mount).
//! Serialized as `[SerializationGenerator(0, false)]`: the version, then the rider.

use muo_io::{GenericReader, GenericWriter};
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{DeathMoveResult, ITEM_BASE, ItemType};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::world::World;

/// `VirtualMountItem._rider`.
pub struct VirtualMountData {
    pub rider: Option<MobileId>,
}

pub static VIRTUAL_MOUNT_ITEM: ItemType = ItemType {
    full_name: "Server.Mobiles.VirtualMountItem",
    parent: Some(&ITEM_BASE),
    // Not [Constructible]: `new VirtualMountItem(Mobile)` is called by code.
    construct: None,
    serialize,
    deserialize,
    on_parent_death,
    ..ITEM_BASE
};

/// `new VirtualMountItem(mob)`: item id 0x3EA0 on the mount layer, riding `mob`.
pub fn new_virtual_mount_item(w: &mut World, rider: MobileId) -> ItemId {
    let id = w.new_item(&VIRTUAL_MOUNT_ITEM);
    if let Some(it) = w.item_mut(id) {
        it.item_id = 0x3EA0;
        it.layer = muo_server::layer::Layer::Mount;
        it.ext = muo_server::Ext::new(VirtualMountData { rider: Some(rider) });
    }
    id
}

/// `Rider`.
pub fn rider(w: &World, id: ItemId) -> Option<MobileId> {
    w.item(id)?.ext.get::<VirtualMountData>()?.rider
}

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(0);
    out.write_mobile(w, rider(w, id));
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _version = r.read_i32()?;
    let rider = r.read_mobile(w)?;
    if let Some(it) = w.item_mut(id) {
        it.ext = muo_server::Ext::new(VirtualMountData { rider });
    }
    // [AfterDeserialization(false)]: a mount item without a rider has nothing to carry.
    if rider.is_none() {
        muo_server::item::base::delete(w, id);
    }
    Ok(())
}

// If this is ever equipped by a player, steps are treated the same as Ethereal mounts.
pub const STEPS_MAX: i32 = 3840;
pub const STEPS_GAINED_PER_IDLE_TIME: i32 = 1;

fn on_parent_death(w: &mut World, id: ItemId, _parent: MobileId) -> DeathMoveResult {
    muo_server::item::base::delete(w, id);
    DeathMoveResult::RemainEquipped
}
