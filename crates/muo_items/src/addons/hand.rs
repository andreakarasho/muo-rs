//! The addon classes with code of their own that need no other system: `JackOLantern`.
//!
//! ArcheryButte, BallotBox, DartBoard, the spinning wheels, looms, flour mills and water troughs,
//! RejuvinationAnkhs, SHTeleporter, SolenAntHole, TrainingDummies and PickpocketDips are `devices` and `hive`.

use muo_content::items::addon::{self, ADDON_COMPONENT, AddonClass, BASE_ADDON, BASE_ADDON_CLASS};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ItemType, ops};
use muo_server::type_registry::TypeRegistry;

use crate::simple::leaf;

/// `GetComponent(itemID, hue)`: an `AddonComponent` with `Hue` and `Name = "jack-o-lantern"`.
fn jack_component(w: &mut World, item_id: i32, hue: i32) -> ItemId {
    let c = addon::new_component(w, &ADDON_COMPONENT, item_id);
    if let Some(ty) = w.item(c).map(|i| i.ty) {
        (ty.set_hue)(w, c, hue);
    }
    ops::set_name(w, c, Some("jack-o-lantern".to_string()));
    c
}

/// `JackOLantern(bool south)`.
pub fn jack_o_lantern_new(w: &mut World, south: bool) -> ItemId {
    let id = addon::new_addon(w, &JACK_O_LANTERN);
    let c = addon::new_component(w, &ADDON_COMPONENT, 5703);
    addon::add_component(w, id, c, 0, 0, 0);
    const HUE: i32 = 1161;
    let rows: [(i32, i32, i32); 3] = if south {
        [(3179, 0, 0), (3885, HUE, -1), (3871, HUE, 0)]
    } else {
        [(3178, 0, -1), (3883, HUE, 1), (3862, HUE, 0)]
    };
    for (item_id, hue, z) in rows {
        let c = jack_component(w, item_id, hue);
        addon::add_component(w, id, c, 0, 0, z);
    }
    id
}

static JACK_O_LANTERN_CLASS: AddonClass = AddonClass {
    share_hue: false,
    ..BASE_ADDON_CLASS
};

leaf! {
    /// `Server.Items.JackOLantern`: `JackOLantern() : this(Utility.Random(2) < 1)`.
    JACK_O_LANTERN = "Server.Items.JackOLantern" : BASE_ADDON, enc;
    construct: Some(|w| {
        let south = muo_primitives::utility::random(&mut w.rng, 2) < 1;
        jack_o_lantern_new(w, south)
    }),
    family: Some(&JACK_O_LANTERN_CLASS),
}

pub static ALL: &[(&ItemType, &[&str])] = &[(&JACK_O_LANTERN, &[])];

pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
