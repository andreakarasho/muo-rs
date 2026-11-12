//! Walls and signs: `BaseWall` and its five
//! walls, `BaseSign`, `Sign`, `LocalizedSign` and `SubtextSign`. The logic of the signs is written below the tables.

use muo_content::props::{Ctor, Kind, Param, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::type_registry::{Phase, TypeRegistry};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));

/// `BaseWall(itemID)`: `Movable = false`.
fn new_wall(w: &mut World, ty: &'static ItemType, gid: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, gid);
    ops::set_movable(w, id, false);
    id
}

/// `Server.Items.BaseWall` (abstract).
pub static BASE_WALL: ItemType = ItemType {
    full_name: "Server.Items.BaseWall",
    construct: None,
    parent: Some(&ITEM_BASE),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

static DARK_WOOD_WALL_TYPES: &[(&str, i64)] = &[
    ("Corner", 0),
    ("SouthWall", 1),
    ("EastWall", 2),
    ("CornerPost", 3),
    ("EastDoorFrame", 4),
    ("SouthDoorFrame", 5),
    ("WestDoorFrame", 6),
    ("NorthDoorFrame", 7),
    ("SouthWindow", 8),
    ("EastWindow", 9),
    ("CornerMedium", 10),
    ("EastWallMedium", 11),
    ("SouthWallMedium", 12),
    ("CornerPostMedium", 13),
    ("CornerShort", 14),
    ("EastWallShort", 15),
    ("SouthWallShort", 16),
    ("CornerPostShort", 17),
    ("SouthWallVShort", 18),
    ("EastWallVShort", 19),
];
static DARK_WOOD_WALL_P: [Param; 1] = [Param {
    name: "type",
    kind: Kind::Plain(PropType::Enum(DARK_WOOD_WALL_TYPES)),
    default: None,
    ty: "DarkWoodWallTypes",
}];
static C_DARK_WOOD_WALL: [Ctor; 1] = [Ctor {
    access: None,
    params: &DARK_WOOD_WALL_P,
    make: |w, v| EntityId::Item(new_wall(w, &DARK_WOOD_WALL, 0x6 + v[0].i32())),
}];

/// `Server.Items.DarkWoodWall`.
pub static DARK_WOOD_WALL: ItemType = ItemType {
    full_name: "Server.Items.DarkWoodWall",
    parent: Some(&BASE_WALL),
    // the type is a constructor argument (`construct` is what loading a save uses)
    construct: None,
    make: Some(|w| new_wall(w, &DARK_WOOD_WALL, 0x6)),
    serialize: |w, id, out| {
        (BASE_WALL.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_WALL.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_WALL
};

static THICK_GRAY_STONE_WALL_TYPES: &[(&str, i64)] = &[
    ("WestArch", 0),
    ("NorthArch", 1),
    ("SouthArchTop", 2),
    ("EastArchTop", 3),
    ("EastArch", 4),
    ("SouthArch", 5),
    ("Wall1", 6),
    ("Wall2", 7),
    ("Wall3", 8),
    ("SouthWindow", 9),
    ("Wall4", 10),
    ("EastWindow", 11),
    ("WestArch2", 12),
    ("NorthArch2", 13),
    ("SouthArchTop2", 14),
    ("EastArchTop2", 15),
    ("EastArch2", 16),
    ("SouthArch2", 17),
    ("SWArchEdge2", 18),
    ("SouthWindow2", 19),
    ("NEArchEdge2", 20),
    ("EastWindow2", 21),
];
static THICK_GRAY_STONE_WALL_P: [Param; 1] = [Param {
    name: "type",
    kind: Kind::Plain(PropType::Enum(THICK_GRAY_STONE_WALL_TYPES)),
    default: None,
    ty: "ThickGrayStoneWallTypes",
}];
static C_THICK_GRAY_STONE_WALL: [Ctor; 1] = [Ctor {
    access: None,
    params: &THICK_GRAY_STONE_WALL_P,
    make: |w, v| EntityId::Item(new_wall(w, &THICK_GRAY_STONE_WALL, 0x7A + v[0].i32())),
}];

/// `Server.Items.ThickGrayStoneWall`.
pub static THICK_GRAY_STONE_WALL: ItemType = ItemType {
    full_name: "Server.Items.ThickGrayStoneWall",
    parent: Some(&BASE_WALL),
    // the type is a constructor argument (`construct` is what loading a save uses)
    construct: None,
    make: Some(|w| new_wall(w, &THICK_GRAY_STONE_WALL, 0x7A)),
    serialize: |w, id, out| {
        (BASE_WALL.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_WALL.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_WALL
};

static THIN_BRICK_WALL_TYPES: &[(&str, i64)] = &[
    ("Corner", 0),
    ("SouthWall", 1),
    ("EastWall", 2),
    ("CornerPost", 3),
    ("EastDoorFrame", 4),
    ("SouthDoorFrame", 5),
    ("WestDoorFrame", 6),
    ("NorthDoorFrame", 7),
    ("SouthWindow", 8),
    ("EastWindow", 9),
    ("CornerMedium", 10),
    ("SouthWallMedium", 11),
    ("EastWallMedium", 12),
    ("CornerPostMedium", 13),
    ("CornerShort", 14),
    ("SouthWallShort", 15),
    ("EastWallShort", 16),
    ("CornerPostShort", 17),
    ("CornerArch", 18),
    ("SouthArch", 19),
    ("WestArch", 20),
    ("EastArch", 21),
    ("NorthArch", 22),
    ("SouthCenterArchTall", 23),
    ("EastCenterArchTall", 24),
    ("EastCornerArchTall", 25),
    ("SouthCornerArchTall", 26),
    ("SouthCornerArch", 27),
    ("EastCornerArch", 28),
    ("SouthCenterArch", 29),
    ("EastCenterArch", 30),
    ("CornerVVShort", 31),
    ("SouthWallVVShort", 32),
    ("EastWallVVShort", 33),
    ("SouthWallVShort", 34),
    ("EastWallVShort", 35),
];
static THIN_BRICK_WALL_P: [Param; 1] = [Param {
    name: "type",
    kind: Kind::Plain(PropType::Enum(THIN_BRICK_WALL_TYPES)),
    default: None,
    ty: "ThinBrickWallTypes",
}];
static C_THIN_BRICK_WALL: [Ctor; 1] = [Ctor {
    access: None,
    params: &THIN_BRICK_WALL_P,
    make: |w, v| EntityId::Item(new_wall(w, &THIN_BRICK_WALL, 0x33 + v[0].i32())),
}];

/// `Server.Items.ThinBrickWall`.
pub static THIN_BRICK_WALL: ItemType = ItemType {
    full_name: "Server.Items.ThinBrickWall",
    parent: Some(&BASE_WALL),
    // the type is a constructor argument (`construct` is what loading a save uses)
    construct: None,
    make: Some(|w| new_wall(w, &THIN_BRICK_WALL, 0x33)),
    serialize: |w, id, out| {
        (BASE_WALL.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_WALL.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_WALL
};

static THIN_STONE_WALL_TYPES: &[(&str, i64)] = &[
    ("Corner", 0),
    ("EastWall", 1),
    ("SouthWall", 2),
    ("CornerPost", 3),
    ("EastDoorFrame", 4),
    ("SouthDoorFrame", 5),
    ("NorthDoorFrame", 6),
    ("WestDoorFrame", 7),
    ("SouthWindow", 8),
    ("EastWindow", 9),
    ("CornerMedium", 10),
    ("SouthWallMedium", 11),
    ("EastWallMedium", 12),
    ("CornerPostMedium", 13),
    ("CornerArch", 14),
    ("EastArch", 15),
    ("SouthArch", 16),
    ("NorthArch", 17),
    ("WestArch", 18),
    ("CornerShort", 19),
    ("EastWallShort", 20),
    ("SouthWallShort", 21),
    ("CornerPostShort", 22),
    ("SouthWallShort2", 23),
    ("EastWallShort2", 24),
];
static THIN_STONE_WALL_P: [Param; 1] = [Param {
    name: "type",
    kind: Kind::Plain(PropType::Enum(THIN_STONE_WALL_TYPES)),
    default: None,
    ty: "ThinStoneWallTypes",
}];
static C_THIN_STONE_WALL: [Ctor; 1] = [Ctor {
    access: None,
    params: &THIN_STONE_WALL_P,
    make: |w, v| EntityId::Item(new_wall(w, &THIN_STONE_WALL, 0x1A + v[0].i32())),
}];

/// `Server.Items.ThinStoneWall`.
pub static THIN_STONE_WALL: ItemType = ItemType {
    full_name: "Server.Items.ThinStoneWall",
    parent: Some(&BASE_WALL),
    // the type is a constructor argument (`construct` is what loading a save uses)
    construct: None,
    make: Some(|w| new_wall(w, &THIN_STONE_WALL, 0x1A)),
    serialize: |w, id, out| {
        (BASE_WALL.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_WALL.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_WALL
};

static WHITE_STONE_WALL_TYPES: &[(&str, i64)] = &[
    ("EastWall", 0),
    ("SouthWall", 1),
    ("SECorner", 2),
    ("NWCornerPost", 3),
    ("EastArrowLoop", 4),
    ("SouthArrowLoop", 5),
    ("EastWindow", 6),
    ("SouthWindow", 7),
    ("SouthWallMedium", 8),
    ("EastWallMedium", 9),
    ("SECornerMedium", 10),
    ("NWCornerPostMedium", 11),
    ("SouthWallShort", 12),
    ("EastWallShort", 13),
    ("SECornerShort", 14),
    ("NWCornerPostShort", 15),
    ("NECornerPostShort", 16),
    ("SWCornerPostShort", 17),
    ("SouthWallVShort", 18),
    ("EastWallVShort", 19),
    ("SECornerVShort", 20),
    ("NWCornerPostVShort", 21),
    ("SECornerArch", 22),
    ("SouthArch", 23),
    ("WestArch", 24),
    ("EastArch", 25),
    ("NorthArch", 26),
    ("EastBattlement", 27),
    ("SECornerBattlement", 28),
    ("SouthBattlement", 29),
    ("NECornerBattlement", 30),
    ("SWCornerBattlement", 31),
    ("Column", 32),
    ("SouthWallVVShort", 33),
    ("EastWallVVShort", 34),
];
static WHITE_STONE_WALL_P: [Param; 1] = [Param {
    name: "type",
    kind: Kind::Plain(PropType::Enum(WHITE_STONE_WALL_TYPES)),
    default: None,
    ty: "WhiteStoneWallTypes",
}];
static C_WHITE_STONE_WALL: [Ctor; 1] = [Ctor {
    access: None,
    params: &WHITE_STONE_WALL_P,
    make: |w, v| EntityId::Item(new_wall(w, &WHITE_STONE_WALL, 0x57 + v[0].i32())),
}];

/// `Server.Items.WhiteStoneWall`.
pub static WHITE_STONE_WALL: ItemType = ItemType {
    full_name: "Server.Items.WhiteStoneWall",
    parent: Some(&BASE_WALL),
    // the type is a constructor argument (`construct` is what loading a save uses)
    construct: None,
    make: Some(|w| new_wall(w, &WHITE_STONE_WALL, 0x57)),
    serialize: |w, id, out| {
        (BASE_WALL.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_WALL.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_WALL
};

static SIGN_TYPES: &[(&str, i64)] = &[
    ("Library", 0),
    ("DarkWoodenPost", 1),
    ("LightWoodenPost", 2),
    ("MetalPostC", 3),
    ("MetalPostB", 4),
    ("MetalPostA", 5),
    ("MetalPost", 6),
    ("Bakery", 7),
    ("Tailor", 8),
    ("Tinker", 9),
    ("Butcher", 10),
    ("Healer", 11),
    ("Mage", 12),
    ("Woodworker", 13),
    ("Customs", 14),
    ("Inn", 15),
    ("Shipwright", 16),
    ("Stables", 17),
    ("BarberShop", 18),
    ("Bard", 19),
    ("Fletcher", 20),
    ("Armourer", 21),
    ("Jeweler", 22),
    ("Tavern", 23),
    ("ReagentShop", 24),
    ("Blacksmith", 25),
    ("Painter", 26),
    ("Provisioner", 27),
    ("Bowyer", 28),
    ("WoodenSign", 29),
    ("BrassSign", 30),
    ("ArmamentsGuild", 31),
    ("ArmourersGuild", 32),
    ("BlacksmithsGuild", 33),
    ("WeaponsGuild", 34),
    ("BardicGuild", 35),
    ("BartersGuild", 36),
    ("ProvisionersGuild", 37),
    ("TradersGuild", 38),
    ("CooksGuild", 39),
    ("HealersGuild", 40),
    ("MagesGuild", 41),
    ("SorcerersGuild", 42),
    ("IllusionistGuild", 43),
    ("MinersGuild", 44),
    ("ArchersGuild", 45),
    ("SeamensGuild", 46),
    ("FishermensGuild", 47),
    ("SailorsGuild", 48),
    ("ShipwrightsGuild", 49),
    ("TailorsGuild", 50),
    ("ThievesGuild", 51),
    ("RoguesGuild", 52),
    ("AssassinsGuild", 53),
    ("TinkersGuild", 54),
    ("WarriorsGuild", 55),
    ("CavalryGuild", 56),
    ("FightersGuild", 57),
    ("MerchantsGuild", 58),
    ("Bank", 59),
    ("Theatre", 60),
];
static SIGN_FACINGS: &[(&str, i64)] = &[("North", 0), ("West", 1)];

// ---- signs ----

/// `BaseSign(dispID)`: `Movable = false`.
fn new_sign_item(w: &mut World, ty: &'static ItemType, gid: i32) -> ItemId {
    new_wall(w, ty, gid)
}

/// `Server.Items.BaseSign` (abstract).
pub static BASE_SIGN: ItemType = ItemType {
    full_name: "Server.Items.BaseSign",
    construct: None,
    parent: Some(&ITEM_BASE),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `0xB95 + 2 * (int)type + (int)facing`.
pub fn sign_graphic(sign_type: i32, facing: i32) -> i32 {
    0xB95 + 2 * sign_type + facing
}

/// `Server.Items.Sign`; the parameterless form is `Sign(SignType.Library, SignFacing.North)`.
pub static SIGN: ItemType = ItemType {
    full_name: "Server.Items.Sign",
    parent: Some(&BASE_SIGN),
    construct: None,
    make: Some(|w| new_sign_item(w, &SIGN, sign_graphic(0, 0))),
    serialize: |w, id, out| {
        (BASE_SIGN.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_SIGN.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_SIGN
};

/// `LocalizedSign._number`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NumberData {
    pub number: i32,
}

fn number(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<NumberData>())
        .map_or(0, |d| d.number)
}

/// `new LocalizedSign(itemID, labelNumber)`.
pub fn new_localized_sign(w: &mut World, gid: i32, label: i32) -> ItemId {
    let id = new_sign_item(w, &LOCALIZED_SIGN, gid);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(NumberData { number: label });
    }
    id
}

/// `Server.Items.LocalizedSign`.
pub static LOCALIZED_SIGN: ItemType = ItemType {
    full_name: "Server.Items.LocalizedSign",
    parent: Some(&SIGN),
    construct: None,
    make: Some(|w| new_localized_sign(w, sign_graphic(0, 0), 0)),
    label_number: |w, id| number(w, id),
    serialize: |w, id, out| {
        (SIGN.serialize)(w, id, out);
        out.write_i32(0);
        out.write_i32(number(w, id));
    },
    deserialize: |w, id, r| {
        (SIGN.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let n = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(NumberData { number: n });
        }
        Ok(())
    },
    ..SIGN
};

/// `SubtextSign._subText`.
#[derive(Clone, Debug, Default)]
pub struct SubtextData {
    pub text: Option<String>,
}

fn subtext(w: &World, id: ItemId) -> Option<String> {
    w.item(id)?
        .ext
        .get::<SubtextData>()?
        .text
        .clone()
        .filter(|t| !t.is_empty())
}

/// `new SubtextSign(itemID, subtext)`.
pub fn new_subtext_sign(w: &mut World, gid: i32, text: Option<String>) -> ItemId {
    let id = new_sign_item(w, &SUBTEXT_SIGN, gid);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(SubtextData { text });
    }
    id
}

/// `Server.Items.SubtextSign`.
pub static SUBTEXT_SIGN: ItemType = ItemType {
    full_name: "Server.Items.SubtextSign",
    parent: Some(&SIGN),
    construct: None,
    make: Some(|w| new_subtext_sign(w, sign_graphic(0, 0), None)),
    on_single_click: |w, id, from| {
        (SIGN.on_single_click)(w, id, from);
        if let Some(t) = subtext(w, id) {
            ops::label_to(w, id, from, &t);
        }
    },
    add_name_properties: |w, id, list| {
        (SIGN.add_name_properties)(w, id, list);
        if let Some(t) = subtext(w, id) {
            list.add_text(&t);
        }
    },
    serialize: |w, id, out| {
        (SIGN.serialize)(w, id, out);
        out.write_i32(0);
        out.write_opt_string(
            w.item(id)
                .and_then(|i| i.ext.get::<SubtextData>())
                .and_then(|d| d.text.as_deref()),
        );
    },
    deserialize: |w, id, r| {
        (SIGN.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let text = r.read_string()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(SubtextData { text });
        }
        Ok(())
    },
    ..SIGN
};

const TYPE_PARAM: Param = Param {
    name: "type",
    kind: Kind::Plain(PropType::Enum(SIGN_TYPES)),
    default: None,
    ty: "SignType",
};
const FACING_PARAM: Param = Param {
    name: "facing",
    kind: Kind::Plain(PropType::Enum(SIGN_FACINGS)),
    default: None,
    ty: "SignFacing",
};
const ITEM_ID_PARAM: Param = Param {
    name: "itemID",
    kind: INT,
    default: None,
    ty: "",
};
const LABEL_PARAM: Param = Param {
    name: "labelNumber",
    kind: INT,
    default: None,
    ty: "",
};
const SUBTEXT_PARAM: Param = Param {
    name: "subtext",
    kind: Kind::Plain(PropType::Str),
    default: None,
    ty: "",
};

static SIGN_P1: [Param; 2] = [TYPE_PARAM, FACING_PARAM];
static SIGN_P2: [Param; 1] = [ITEM_ID_PARAM];
static LOCAL_P1: [Param; 3] = [TYPE_PARAM, FACING_PARAM, LABEL_PARAM];
static LOCAL_P2: [Param; 2] = [ITEM_ID_PARAM, LABEL_PARAM];
static SUB_P1: [Param; 3] = [TYPE_PARAM, FACING_PARAM, SUBTEXT_PARAM];
static SUB_P2: [Param; 2] = [ITEM_ID_PARAM, SUBTEXT_PARAM];

static C_SIGN: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &SIGN_P1,
        make: |w, v| {
            EntityId::Item(new_sign_item(
                w,
                &SIGN,
                sign_graphic(v[0].i32(), v[1].i32()),
            ))
        },
    },
    Ctor {
        access: None,
        params: &SIGN_P2,
        make: |w, v| EntityId::Item(new_sign_item(w, &SIGN, v[0].i32())),
    },
];
static C_LOCALIZED_SIGN: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &LOCAL_P1,
        make: |w, v| {
            EntityId::Item(new_localized_sign(
                w,
                sign_graphic(v[0].i32(), v[1].i32()),
                v[2].i32(),
            ))
        },
    },
    Ctor {
        access: None,
        params: &LOCAL_P2,
        make: |w, v| EntityId::Item(new_localized_sign(w, v[0].i32(), v[1].i32())),
    },
];
static C_SUBTEXT_SIGN: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &SUB_P1,
        make: |w, v| {
            EntityId::Item(new_subtext_sign(
                w,
                sign_graphic(v[0].i32(), v[1].i32()),
                v[2].string(),
            ))
        },
    },
    Ctor {
        access: None,
        params: &SUB_P2,
        make: |w, v| EntityId::Item(new_subtext_sign(w, v[0].i32(), v[1].string())),
    },
];

fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.Sign", &C_SIGN);
    add_ctors(w, "Server.Items.LocalizedSign", &C_LOCALIZED_SIGN);
    add_ctors(w, "Server.Items.SubtextSign", &C_SUBTEXT_SIGN);
    add_ctors(w, "Server.Items.DarkWoodWall", &C_DARK_WOOD_WALL);
    add_ctors(
        w,
        "Server.Items.ThickGrayStoneWall",
        &C_THICK_GRAY_STONE_WALL,
    );
    add_ctors(w, "Server.Items.ThinBrickWall", &C_THIN_BRICK_WALL);
    add_ctors(w, "Server.Items.ThinStoneWall", &C_THIN_STONE_WALL);
    add_ctors(w, "Server.Items.WhiteStoneWall", &C_WHITE_STONE_WALL);
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &BASE_WALL,
        &DARK_WOOD_WALL,
        &THICK_GRAY_STONE_WALL,
        &THIN_BRICK_WALL,
        &THIN_STONE_WALL,
        &WHITE_STONE_WALL,
        &BASE_SIGN,
        &SIGN,
        &LOCALIZED_SIGN,
        &SUBTEXT_SIGN,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BaseSign.Configure",
        configure,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};

    #[test]
    fn a_wall_is_its_base_graphic_plus_the_type() {
        let mut w = World::new();
        let id = (WHITE_STONE_WALL
            .construct
            .or(WHITE_STONE_WALL.make)
            .unwrap())(&mut w);
        assert_eq!(w.item(id).unwrap().item_id, 0x57);
        assert!(!w.item(id).unwrap().movable());
    }

    #[test]
    fn sign_graphics_and_subtext_save() {
        assert_eq!(sign_graphic(3, 1), 0xB95 + 7);
        let mut w = World::new();
        let id = new_subtext_sign(&mut w, 0xBA7, Some("open late".to_string()));
        let mut out = SaveWriter::new(true);
        (SUBTEXT_SIGN.serialize)(&w, id, &mut out);
        let bytes = out.into_vec();
        let id2 = w.add_item(muo_server::item::Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0100),
            &SUBTEXT_SIGN,
        ));
        let mut r = SaveReader::new(&bytes);
        (SUBTEXT_SIGN.deserialize)(&mut w, id2, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        assert_eq!(subtext(&w, id2).as_deref(), Some("open late"));
        let l = new_localized_sign(&mut w, 0xBA7, 1011234);
        assert_eq!((LOCALIZED_SIGN.label_number)(&w, l), 1011234);
    }
}
