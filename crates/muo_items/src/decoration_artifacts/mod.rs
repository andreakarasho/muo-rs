//! Decoration artifacts: `BaseDecorationArtifact` (the two abstract bases) and the artifact classes in `leaves.rs`.

use muo_content::items::cont::BASE_CONTAINER;
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, ItemType, base};
use muo_server::property_list::PropertyList;
use muo_server::type_registry::TypeRegistry;

pub mod leaves;

/// `ArtifactRarity`, the abstract property of both bases.
#[derive(Copy, Clone)]
pub struct DecoClass {
    pub artifact_rarity: i32,
}

fn rarity(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ty.family::<DecoClass>())
        .map_or(0, |c| c.artifact_rarity)
}

/// `Server.Items.BaseDecorationArtifact` (abstract): `[SerializationGenerator(0)]`.
pub static BASE_DECORATION_ARTIFACT: ItemType = ItemType {
    full_name: "Server.Items.BaseDecorationArtifact",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_weight: crate::class_props::default_weight,
    force_show_properties: |_, _| true,
    get_properties: |w, id, list: &mut PropertyList| {
        base::get_properties_base(w, id, list);
        list.add_int(1061078, rarity(w, id)); // artifact rarity ~1_val~
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `Server.Items.BaseDecorationContainerArtifact` (abstract): `[SerializationGenerator(0)]`.
pub static BASE_DECORATION_CONTAINER_ARTIFACT: ItemType = ItemType {
    full_name: "Server.Items.BaseDecorationContainerArtifact",
    parent: Some(&BASE_CONTAINER),
    construct: None,
    default_weight: crate::class_props::default_weight,
    force_show_properties: |_, _| true,
    add_name_properties: |w, id, list: &mut PropertyList| {
        base::add_name_properties_base(w, id, list);
        list.add_int(1061078, rarity(w, id)); // artifact rarity ~1_val~
    },
    serialize: |w, id, out| {
        (BASE_CONTAINER.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_CONTAINER
};

/// Registers the classes of this file (the bases are abstract).
pub fn register(types: &mut TypeRegistry) {
    leaves::register(types);
}
