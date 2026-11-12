//! `DuelTeleporterAddon`: the addon of a dueling teleporter pad.
//!
//! The `Type` property (`[CommandProperty]`) is `TYPE_PROPS`.

use muo_content::items::addon::{self, BASE_ADDON, Comp};
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::PropType;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::ItemType;
use muo_server::type_registry::TypeRegistry;

/// `DuelTeleporterType`.
const TYPE_NAMES: &[(&str, i64)] = &[
    ("Squares", 6095),
    ("Buds", 6104),
    ("Flowers", 6113),
    ("Spikes", 6122),
    ("Arrows", 6140),
    ("Links", 6149),
];

/// `new DuelTeleporterAddon(type)`.
pub fn new_duel_teleporter(w: &mut World, item_id: i32) -> ItemId {
    let id = addon::new_addon(w, &DUEL_TELEPORTER_ADDON);
    let rows: [(Comp, i32, i32, i32); 25] = [
        (Comp::Plain(item_id), -1, -1, 5),
        (Comp::Plain(item_id + 1), -1, 0, 5),
        (Comp::Plain(item_id + 2), 0, -1, 5),
        (Comp::Plain(item_id + 3), -1, 1, 5),
        (Comp::Plain(item_id + 4), 0, 0, 5),
        (Comp::Plain(item_id + 5), 1, -1, 5),
        (Comp::Plain(item_id + 6), 0, 1, 5),
        (Comp::Plain(item_id + 7), 1, 0, 5),
        (Comp::Plain(item_id + 8), 1, 1, 5),
        (Comp::Plain(0x759), -2, -2, 0),
        (Comp::Plain(0x75A), 2, 2, 0),
        (Comp::Plain(0x75B), -2, 2, 0),
        (Comp::Plain(0x75C), 2, -2, 0),
        (Comp::Plain(0x751), -1, 2, 0),
        (Comp::Plain(0x751), 0, 2, 0),
        (Comp::Plain(0x751), 1, 2, 0),
        (Comp::Plain(0x752), 2, -1, 0),
        (Comp::Plain(0x752), 2, 0, 0),
        (Comp::Plain(0x752), 2, 1, 0),
        (Comp::Plain(0x753), -1, -2, 0),
        (Comp::Plain(0x753), 0, -2, 0),
        (Comp::Plain(0x753), 1, -2, 0),
        (Comp::Plain(0x754), -2, -1, 0),
        (Comp::Plain(0x754), -2, 0, 0),
        (Comp::Plain(0x754), -2, 1, 0),
    ];
    addon::add_components(w, id, &rows);
    id
}

/// `DuelTeleporterAddon`.
pub static DUEL_TELEPORTER_ADDON: ItemType = ItemType {
    full_name: "Server.Engines.ConPVP.DuelTeleporterAddon",
    parent: Some(&BASE_ADDON),
    construct: Some(|w| new_duel_teleporter(w, 6095)),
    serialize: |w, id, out| {
        (BASE_ADDON.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_ADDON.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_ADDON
};

static PARAMS: [Param; 1] = [Param {
    name: "type",
    kind: Kind::Plain(PropType::Enum(TYPE_NAMES)),
    default: Some(Value::Enum(6095)),
    ty: "DuelTeleporterType",
}];

/// `DuelTeleporterAddon(type = Squares)`.
static CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &PARAMS,
    make: |w, v| EntityId::Item(new_duel_teleporter(w, v[0].i64() as i32)),
}];

/// `Type`: the item id of the first component picks the pattern; setting it re-lays the first nine.
static TYPE_PROPS: &[muo_content::props::PropDef] = &[muo_content::props::PropDef {
    name: "Type",
    kind: Kind::Plain(PropType::Enum(TYPE_NAMES)),
    read: muo_server::mobile::AccessLevel::GameMaster,
    write: muo_server::mobile::AccessLevel::GameMaster,
    get: |w, e| {
        let EntityId::Item(i) = e else {
            return Value::Null;
        };
        let first = addon::components(w, i)
            .first()
            .and_then(|c| w.item(*c))
            .map_or(6095, |c| c.item_id);
        Value::Enum(i64::from(first))
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err("not an item".to_owned());
        };
        for (n, c) in addon::components(w, i).into_iter().take(9).enumerate() {
            if let Some(ty) = w.item(c).map(|x| x.ty) {
                (ty.set_item_id)(w, c, n as i32 + v.i32());
            }
        }
        Ok(())
    }),
}];

/// Registers the item class.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&DUEL_TELEPORTER_ADDON, &[]);
}

/// The `[Constructible]` overload.
pub fn configure(w: &mut World) {
    add_ctors(w, "Server.Engines.ConPVP.DuelTeleporterAddon", &CTORS);
    muo_content::props::add(w, "Server.Engines.ConPVP.DuelTeleporterAddon", TYPE_PROPS);
}
