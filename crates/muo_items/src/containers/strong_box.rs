//! `Strongbox`: a house owner's strong box. Its owner and house are [`BoxData`] in
//! `Item::ext`; the house system reaches it through the `strong_box_*` hooks of `HouseDeps`.
//!
//! `IChoppable.OnChop` is [`on_chop`] ([`chop`] is the body).

use muo_content::items::cont::{BASE_CONTAINER, METAL_BOX, new_base_container};
use muo_content::items::house_deps::deps_mut;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ItemType, TotalType, container, ops};
use muo_server::mobile::AccessLevel;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;
use muo_server::type_registry::{Phase, TypeRegistry};

/// `StrongBox._owner` and `_house`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BoxData {
    pub owner: Option<MobileId>,
    pub house: Option<ItemId>,
}

fn data(w: &World, id: ItemId) -> BoxData {
    w.item(id)
        .and_then(|i| i.ext.get::<BoxData>())
        .copied()
        .unwrap_or_default()
}

fn set_data(w: &mut World, id: ItemId, d: BoxData) {
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
}

/// `new StrongBox(owner, house)`.
pub fn new_strong_box(w: &mut World, owner: Option<MobileId>, house: Option<ItemId>) -> ItemId {
    let id = new_base_container(w, &STRONG_BOX, 0xE80);
    set_data(w, id, BoxData { owner, house });
    container::set_max_items(w, id, 25);
    id
}

fn mobile_deleted(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_none_or(|x| x.deleted)
}

fn house_deleted(w: &World, h: ItemId) -> bool {
    w.item(h).is_none_or(|x| x.deleted())
}

fn destroy(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.destroy)(w, id);
    }
}

/// `Chop(from)`.
pub fn chop(w: &mut World, id: ItemId, from: MobileId) {
    let (loc, map) = (
        ops::get_world_location(w, id),
        w.item(id).and_then(|i| i.map),
    );
    muo_server::effects::play_sound(w, loc, map, 0x3B3);
    muo_server::item::deps::send_localized_message(w, from, 500461); // You destroy the item.
    destroy(w, id);
}

/// `OnChop(from)`.
pub fn on_chop(w: &mut World, id: ItemId, from: MobileId) {
    let d = data(w, id);
    let free = d.house.is_none_or(|h| house_deleted(w, h))
        || d.owner.is_none_or(|o| mobile_deleted(w, o))
        || Some(from) == d.owner
        || d.house
            .is_some_and(|h| muo_multis::house::access::is_owner(w, h, from));
    if free {
        chop(w, id, from);
    }
}

/// `targeted is Strongbox` (`IChoppable`): its `OnChop`.
pub fn strong_box_chop(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !w.item(id).is_some_and(|i| i.ty.is(&STRONG_BOX)) {
        return false;
    }
    on_chop(w, id, from);
    true
}

/// `Validate()`: a strong box whose owner no longer co-owns its house goes away.
fn validate(w: &mut World, id: ItemId) {
    let d = data(w, id);
    let ok = matches!((d.house, d.owner), (Some(h), Some(o)) if muo_multis::house::access::is_co_owner(w, h, o));
    if !ok {
        destroy(w, id);
    }
}

/// `ConvertToStandardContainer()`: the items move into a `MetalBox`.
pub fn convert_to_standard_container(w: &mut World, id: ItemId) -> ItemId {
    let metal = new_base_container(w, &METAL_BOX, 0x9A8);
    let items = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
    for item in items {
        if let Some(ty) = w.item(metal).map(|i| i.ty) {
            (ty.add_item)(w, metal, item);
        }
    }
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
    metal
}

/// `Server.Items.StrongBox`; `[Flippable(0xE80, 0x9A8)]`.
pub static STRONG_BOX: ItemType = ItemType {
    full_name: "Server.Items.StrongBox",
    parent: Some(&BASE_CONTAINER),
    // no parameterless constructor: `construct` is what loading a save uses
    construct: None,
    make: Some(|w| new_strong_box(w, None, None)),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number,
    default_max_weight: crate::class_props::default_max_weight,
    add_name_property: |w, id, list| match data(w, id).owner {
        Some(o) => {
            let name = w
                .mobile(o)
                .and_then(|m| (m.ty.name)(w, o))
                .unwrap_or_default();
            list.add_str(1042887, &name); // a strong box owned by ~1_OWNER_NAME~
        }
        None => (BASE_CONTAINER.add_name_property)(w, id, list),
    },
    on_single_click: |w, id, from| {
        let Some(o) = data(w, id).owner else {
            return (BASE_CONTAINER.on_single_click)(w, id, from);
        };
        let name = w
            .mobile(o)
            .and_then(|m| (m.ty.name)(w, o))
            .unwrap_or_default();
        ops::label_to_number(w, id, from, 1042887, &name); // a strong box owned by ~1_OWNER_NAME~
        let ty = w.item(id).map(|i| i.ty);
        if ty.is_some_and(|t| (t.check_content_display)(w, id, from)) {
            let items = ops::get_total(w, id, TotalType::Items);
            let weight = ops::get_total(w, id, TotalType::Weight);
            ops::label_to(w, id, from, &format!("({items} items, {weight} stones)"));
        }
    },
    is_accessible_to: |w, id, m| {
        let d = data(w, id);
        let gm = w
            .mobile(m)
            .is_some_and(|x| x.access_level >= AccessLevel::GameMaster);
        if d.owner.is_none_or(|o| mobile_deleted(w, o))
            || d.house.is_none_or(|h| house_deleted(w, h))
            || gm
        {
            return true;
        }
        let co = d
            .house
            .is_some_and(|h| muo_multis::house::access::is_co_owner(w, h, m));
        Some(m) == d.owner && co && (BASE_CONTAINER.is_accessible_to)(w, id, m)
    },
    serialize: |w, id, out| {
        (BASE_CONTAINER.serialize)(w, id, out);
        let d = data(w, id);
        out.write_i32(0);
        out.write_mobile(w, d.owner);
        out.write_item(w, d.house);
    },
    deserialize: |w, id, r| {
        (BASE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let owner = r.read_mobile(w)?;
        let house = r.read_item(w)?;
        set_data(w, id, BoxData { owner, house });
        // [AfterDeserialization]
        timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            validate(w, id)
        });
        Ok(())
    },
    ..BASE_CONTAINER
};

fn configure(w: &mut World) {
    let d = deps_mut(w);
    d.make_strong_box = Some(|w, owner, house| Some(new_strong_box(w, Some(owner), Some(house))));
    d.strong_box_owner = Some(|w, id| {
        w.item(id).filter(|i| i.ty.is(&STRONG_BOX))?;
        Some(data(w, id).owner)
    });
    d.strong_box_convert = Some(convert_to_standard_container);
    d.strong_box_destroy = Some(destroy);
}

/// Registers the class and the house hooks.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&STRONG_BOX, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.StrongBox.Configure",
        configure,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::GenericWriter;
    use muo_io::{SaveReader, SaveWriter};

    #[test]
    fn strong_box_saves_owner_and_house_after_the_container() {
        let mut w = World::new();
        let id = new_strong_box(&mut w, None, None);
        assert_eq!(w.item(id).unwrap().item_id, 0xE80);
        let mut out = SaveWriter::new(true);
        (STRONG_BOX.serialize)(&w, id, &mut out);
        let bytes = out.into_vec();
        let mut want = SaveWriter::new(true);
        (BASE_CONTAINER.serialize)(&w, id, &mut want);
        want.write_i32(0);
        want.write_mobile(&w, None);
        want.write_item(&w, None);
        assert_eq!(bytes, want.into_vec());
        let id2 = w.add_item(muo_server::item::Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0100),
            &STRONG_BOX,
        ));
        let mut r = SaveReader::new(&bytes);
        (STRONG_BOX.deserialize)(&mut w, id2, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        assert_eq!((STRONG_BOX.default_max_weight)(&w, id), 0);
    }
}
