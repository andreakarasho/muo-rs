//! `FarmableCrop` and its eight crops.

use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, deps, ops};
use muo_server::map::MapId;
use muo_server::packets::message::MessageType;
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

use crate::simple::{item_id, random_from};

/// `FarmableCrop._picked`.
#[derive(Clone, Copy, Debug, Default)]
struct CropData {
    picked: bool,
}

/// The `abstract` members of a crop: `GetCropObject()` and `GetPickedID()`.
pub struct CropClass {
    pub crop_object: fn(&mut World) -> Option<ItemId>,
    pub picked_id: fn(&mut World) -> i32,
}

fn picked(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.ext.get::<CropData>())
        .is_some_and(|d| d.picked)
}

/// `Unlink()`.
fn unlink(w: &mut World, id: ItemId) {
    let Some(sp) = w.item(id).and_then(|i| i.spawner) else {
        return;
    };
    muo_server::mobile::part2::deps::spawner_remove(w, sp, EntityId::Item(id));
    if let Some(i) = w.item_mut(id) {
        i.spawner = None;
    }
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// `OnPicked(from, loc, map)`.
fn on_picked(w: &mut World, id: ItemId, loc: Point3D, map: MapId) {
    let Some(class) = w.item(id).and_then(|i| i.ty.family::<CropClass>()) else {
        return;
    };
    let gid = (class.picked_id)(w);
    item_id(w, id, gid);
    if let Some(spawn) = (class.crop_object)(w) {
        ops::move_to_world_map(w, spawn, loc, Some(map));
    }
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<CropData>()) {
        d.picked = true;
    }
    unlink(w, id);
    timer::start(w, TimeSpan::from_mins(5), TimeSpan::ZERO, 1, move |w| {
        delete(w, id)
    });
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let Some(i) = w.item(id) else { return };
    let (map, loc) = (i.map, i.location);
    if i.parent.is_some()
        || i.movable()
        || ops::is_locked_down(w, id)
        || ops::is_secure(w, id)
        || map.is_none_or(|m| m == MapId::INTERNAL)
    {
        return;
    }
    if !deps::mobile_in_range(w, from, loc, 2) || !deps::in_los_item(w, from, id) {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1019045); // I can't reach that.
    } else if !picked(w, id)
        && let Some(m) = map
    {
        on_picked(w, id, loc, m);
    }
}

fn new_crop(w: &mut World, ty: &'static ItemType, gid: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, gid);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(CropData::default());
    }
    ops::set_movable(w, id, false);
    id
}

/// `Server.Items.FarmableCrop`; `[SerializationGenerator(0)]`.
pub static FARMABLE_CROP: ItemType = ItemType {
    full_name: "Server.Items.FarmableCrop",
    construct: None,
    parent: Some(&ITEM_BASE),
    on_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_bool(picked(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let p = r.read_bool()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(CropData { picked: p });
        }
        if p {
            // AfterDeserialization
            unlink(w, id);
            delete(w, id);
        }
        Ok(())
    },
    ..ITEM_BASE
};

fn make_named(w: &mut World, name: &str, gid: Option<i32>) -> Option<ItemId> {
    let id = muo_content::factory::make_item(w, name)?;
    if let Some(g) = gid {
        item_id(w, id, g);
    }
    Some(id)
}

macro_rules! crops {
    ($($name:ident, $class:ident = $full:literal,
        crop |$ow:ident| $object:expr, picked |$pw:ident| $picked:expr;)*) => {$(
        static $class: CropClass = CropClass { crop_object: |$ow| $object, picked_id: |$pw| $picked };
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&FARMABLE_CROP),
            construct: Some(|w| {
                let id = new_crop(w, &$name, 0);
                crate::simple::own_template(w, id, $full)
            }),
            family: Some(&$class),
            serialize: |w, id, out| {
                (FARMABLE_CROP.serialize)(w, id, out);
                out.write_encoded_int(0);
            },
            deserialize: |w, id, r| {
                (FARMABLE_CROP.deserialize)(w, id, r)?;
                let _version = r.read_encoded_int()?;
                Ok(())
            },
            ..FARMABLE_CROP
        };
    )*};
}

crops! {
    FARMABLE_CABBAGE, FC_CABBAGE = "Server.Items.FarmableCabbage",
        crop |w| { let g = random_from(w, 3195, 2); make_named(w, "Server.Items.Cabbage", Some(g)) },
        picked |_w| 3254;
    FARMABLE_CARROT, FC_CARROT = "Server.Items.FarmableCarrot",
        crop |w| { let g = random_from(w, 3191, 2); make_named(w, "Server.Items.Carrot", Some(g)) },
        picked |_w| 3254;
    FARMABLE_COTTON, FC_COTTON = "Server.Items.FarmableCotton",
        crop |w| make_named(w, "Server.Items.Cotton", None), picked |_w| 3254;
    FARMABLE_FLAX, FC_FLAX = "Server.Items.FarmableFlax",
        crop |w| { let g = random_from(w, 6812, 2); make_named(w, "Server.Items.Flax", Some(g)) },
        picked |_w| 3254;
    FARMABLE_LETTUCE, FC_LETTUCE = "Server.Items.FarmableLettuce",
        crop |w| { let g = random_from(w, 3184, 2); make_named(w, "Server.Items.Lettuce", Some(g)) },
        picked |_w| 3254;
    FARMABLE_ONION, FC_ONION = "Server.Items.FarmableOnion",
        crop |w| { let g = random_from(w, 3181, 2); make_named(w, "Server.Items.Onion", Some(g)) },
        picked |_w| 3254;
    FARMABLE_PUMPKIN, FC_PUMPKIN = "Server.Items.FarmablePumpkin",
        crop |w| { let g = random_from(w, 3178, 3); make_named(w, "Server.Items.Pumpkin", Some(g)) },
        picked |w| random_from(w, 3166, 3);
    FARMABLE_TURNIP, FC_TURNIP = "Server.Items.FarmableTurnip",
        crop |w| { let g = random_from(w, 3385, 2); make_named(w, "Server.Items.Turnip", Some(g)) },
        picked |_w| 3254;
    FARMABLE_WHEAT, FC_WHEAT = "Server.Items.FarmableWheat",
        crop |w| make_named(w, "Server.Items.WheatSheaf", None), picked |w| random_from(w, 3502, 2);
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&FARMABLE_CROP, &[]);
    for t in [
        &FARMABLE_CABBAGE,
        &FARMABLE_CARROT,
        &FARMABLE_COTTON,
        &FARMABLE_FLAX,
        &FARMABLE_LETTUCE,
        &FARMABLE_ONION,
        &FARMABLE_PUMPKIN,
        &FARMABLE_TURNIP,
        &FARMABLE_WHEAT,
    ] {
        types.register_item(t, &[]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simple::random;
    use muo_io::GenericWriter;
    use muo_io::{SaveReader, SaveWriter};

    #[test]
    fn crop_graphics_and_save() {
        let mut w = crate::test_world();
        let id = (FARMABLE_ONION.construct.unwrap())(&mut w);
        assert!(!w.item(id).unwrap().movable());
        assert_eq!(w.item(id).unwrap().item_id, 3183);
        let mut out = SaveWriter::new(true);
        (FARMABLE_ONION.serialize)(&w, id, &mut out);
        let bytes = out.into_vec();
        let mut want = SaveWriter::new(true);
        (ITEM_BASE.serialize)(&w, id, &mut want);
        want.write_encoded_int(0); // FarmableCrop
        want.write_bool(false);
        want.write_encoded_int(0); // FarmableOnion
        assert_eq!(bytes, want.into_vec());
        let id2 = w.add_item(muo_server::item::Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0100),
            &FARMABLE_ONION,
        ));
        let mut r = SaveReader::new(&bytes);
        (FARMABLE_ONION.deserialize)(&mut w, id2, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        for _ in 0..30 {
            let c = (FARMABLE_PUMPKIN.construct.unwrap())(&mut w);
            assert!((3166..3169).contains(&w.item(c).unwrap().item_id));
        }
        let _ = random(&mut w, 2);
    }
}
