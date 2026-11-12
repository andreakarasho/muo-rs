//! Small hand-written classes of the `leaf_simple` folders whose constructor rolls a graphic or takes an argument:
//! Champion Artifacts/Decorative/{Futon, Pier, WallBlood, Web}, Construction/Misc/Vines and
//! Decoration Artifacts/TowerLanternArtifact. (`Futon.Flip()` is registered in `own_flips`.)

use muo_content::props::{Ctor, Kind, Param, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{ITEM_BASE, ItemType, deps, ops};
use muo_server::light_type::LightType;
use muo_server::packets::message::MessageType;
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::decoration_artifacts::{BASE_DECORATION_ARTIFACT, DecoClass};
use crate::simple::{item_id, random, random_bool};

fn write_v0(w: &World, id: ItemId, out: &mut dyn muo_io::GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(0);
}

fn read_v0(w: &mut World, id: ItemId, r: &mut dyn muo_io::GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _version = r.read_i32()?;
    Ok(())
}

/// `Server.Items.Futon`: `Utility.RandomBool() ? 0x295C : 0x295E`.
pub static FUTON: ItemType = ItemType {
    full_name: "Server.Items.Futon",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let g = if random_bool(w) { 0x295C } else { 0x295E };
        ops::new_item_with_id(w, &FUTON, g)
    }),
    serialize: write_v0,
    deserialize: read_v0,
    ..ITEM_BASE
};

/// `new Pier(itemid)`.
pub fn new_pier(w: &mut World, gid: i32) -> ItemId {
    ops::new_item_with_id(w, &PIER, gid)
}

/// `Server.Items.Pier`: one of three rock graphics.
pub static PIER: ItemType = ItemType {
    full_name: "Server.Items.Pier",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let g = [0x3486, 0x348b, 0x3ae][random(w, 3) as usize];
        new_pier(w, g)
    }),
    serialize: write_v0,
    deserialize: read_v0,
    ..ITEM_BASE
};

/// `Server.Items.WallBlood`.
pub static WALL_BLOOD: ItemType = ItemType {
    full_name: "Server.Items.WallBlood",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let g = if random_bool(w) { 0x1D95 } else { 0x1D94 };
        ops::new_item_with_id(w, &WALL_BLOOD, g)
    }),
    serialize: write_v0,
    deserialize: read_v0,
    ..ITEM_BASE
};

/// `new Web(itemid)`.
pub fn new_web(w: &mut World, gid: i32) -> ItemId {
    ops::new_item_with_id(w, &WEB, gid)
}

/// `Server.Items.Web`: one of three graphics.
pub static WEB: ItemType = ItemType {
    full_name: "Server.Items.Web",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let g = [0x10d7, 0x10d8, 0x10dd][random(w, 3) as usize];
        new_web(w, g)
    }),
    serialize: write_v0,
    deserialize: read_v0,
    ..ITEM_BASE
};

/// `new Vines(v)`: `0xCEB + Math.Clamp(v, 0, 7)`.
pub fn new_vines(w: &mut World, v: i32) -> ItemId {
    ops::new_item_with_id(w, &VINES, 0xCEB + v.clamp(0, 7))
}

/// `Server.Items.Vines`.
pub static VINES: ItemType = ItemType {
    full_name: "Server.Items.Vines",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let v = random(w, 8);
        new_vines(w, v)
    }),
    default_weight: |_, _| 1.0,
    force_show_properties: |w, _| w.core.opl_enabled,
    serialize: write_v0,
    deserialize: read_v0,
    ..ITEM_BASE
};

fn tower_is_on(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.item_id == 0x24BF)
}

static TOWER_LANTERN_CLASS: DecoClass = DecoClass { artifact_rarity: 3 };

/// `Server.Items.TowerLanternArtifact`; `[SerializationGenerator(0)]`.
pub static TOWER_LANTERN_ARTIFACT: ItemType = ItemType {
    full_name: "Server.Items.TowerLanternArtifact",
    parent: Some(&BASE_DECORATION_ARTIFACT),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &TOWER_LANTERN_ARTIFACT, 0x24C0);
        ops::set_light(w, id, LightType::Circle225);
        id
    }),
    family: Some(&TOWER_LANTERN_CLASS),
    on_double_click: |w, id, from| {
        if !deps::mobile_in_range(w, from, ops::get_world_location(w, id), 2) {
            deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1019045); // I can't reach that.
            return;
        }
        let on = tower_is_on(w, id);
        item_id(w, id, if on { 0x24C0 } else { 0x24BF });
        muo_server::mobile::part2::effects::play_sound(w, from, if on { 0x3BE } else { 0x47 });
    },
    serialize: |w, id, out| {
        (BASE_DECORATION_ARTIFACT.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_DECORATION_ARTIFACT.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_DECORATION_ARTIFACT
};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
static ITEMID_PARAM: [Param; 1] = [Param {
    name: "itemid",
    kind: INT,
    default: None,
    ty: "",
}];
static V_PARAM: [Param; 1] = [Param {
    name: "v",
    kind: INT,
    default: None,
    ty: "",
}];

static C_WEB: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| EntityId::Item((WEB.construct.expect("set"))(w)),
    },
    Ctor {
        access: None,
        params: &ITEMID_PARAM,
        make: |w, v| EntityId::Item(new_web(w, v[0].i32())),
    },
];
static C_PIER: [Ctor; 1] = [Ctor {
    access: None,
    params: &[],
    make: |w, _| EntityId::Item((PIER.construct.expect("set"))(w)),
}];
static C_VINES: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| EntityId::Item((VINES.construct.expect("set"))(w)),
    },
    Ctor {
        access: None,
        params: &V_PARAM,
        make: |w, v| EntityId::Item(new_vines(w, v[0].i32())),
    },
];

fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.Web", &C_WEB);
    add_ctors(w, "Server.Items.Pier", &C_PIER);
    add_ctors(w, "Server.Items.Vines", &C_VINES);
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &FUTON,
        &PIER,
        &WALL_BLOOD,
        &WEB,
        &VINES,
        &TOWER_LANTERN_ARTIFACT,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(Phase::Configure, "Server.Items.Vines.Configure", configure);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolled_graphics_and_vines_clamp() {
        let mut w = World::new();
        for _ in 0..20 {
            let id = (WEB.construct.unwrap())(&mut w);
            assert!([0x10d7, 0x10d8, 0x10dd].contains(&w.item(id).unwrap().item_id));
        }
        let v = new_vines(&mut w, 99);
        assert_eq!(w.item(v).unwrap().item_id, 0xCEB + 7);
        let v = new_vines(&mut w, -3);
        assert_eq!(w.item(v).unwrap().item_id, 0xCEB);
    }

    #[test]
    fn tower_lantern_toggles_graphic_and_has_rarity() {
        let mut w = World::new();
        let id = (TOWER_LANTERN_ARTIFACT.construct.unwrap())(&mut w);
        assert_eq!(w.item(id).unwrap().item_id, 0x24C0);
        assert!(!tower_is_on(&w, id));
        assert_eq!(
            TOWER_LANTERN_ARTIFACT
                .family::<DecoClass>()
                .unwrap()
                .artifact_rarity,
            3
        );
    }
}
