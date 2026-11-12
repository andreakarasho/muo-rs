//! The plain skill `Item` classes: a
//! constructor of simple statements and constant overrides. Same shape as `armor_leaf!`.

use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::ids::EntityId;
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{read_version, write_version};

/// `new Anvil()`.
pub fn new_anvil(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0xFAF;
    let id = ops::new_item_with_id(w, &ANVIL, gid);
    ops::set_movable(w, id, false);
    id
}

/// `Server.Items.Anvil`.
pub static ANVIL: ItemType = ItemType {
    full_name: "Server.Items.Anvil",
    parent: Some(&ITEM_BASE),
    construct: Some(new_anvil),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new Forge()`.
pub fn new_forge(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0xFB1;
    let id = ops::new_item_with_id(w, &FORGE, gid);
    ops::set_movable(w, id, false);
    id
}

/// `Server.Items.Forge`.
pub static FORGE: ItemType = ItemType {
    full_name: "Server.Items.Forge",
    parent: Some(&ITEM_BASE),
    construct: Some(new_forge),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new BlankScroll(amount)`.
pub fn new_blank_scroll(w: &mut muo_server::World, amount: i32) -> muo_server::ids::ItemId {
    let gid = 0xEF3;
    let id = ops::new_item_with_id(w, &BLANK_SCROLL, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Server.Items.BlankScroll`.
pub static BLANK_SCROLL: ItemType = ItemType {
    full_name: "Server.Items.BlankScroll",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_blank_scroll(w, 1)),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

static BLANK_SCROLL_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_blank_scroll(w, v[0].i32())),
}];

/// `new Bottle(amount)`.
pub fn new_bottle(w: &mut muo_server::World, amount: i32) -> muo_server::ids::ItemId {
    let gid = 0xF0E;
    let id = ops::new_item_with_id(w, &BOTTLE, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Server.Items.Bottle`.
pub static BOTTLE: ItemType = ItemType {
    full_name: "Server.Items.Bottle",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_bottle(w, 1)),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

static BOTTLE_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_bottle(w, v[0].i32())),
}];

/// `new Sand(amount)`.
pub fn new_sand(w: &mut muo_server::World, _amount: i32) -> muo_server::ids::ItemId {
    let gid = 0x11EA;
    let id = ops::new_item_with_id(w, &SAND, gid);
    let ml = muo_content::core::ml(w);
    ops::set_stackable(w, id, ml);
    id
}

/// `Server.Items.Sand`.
pub static SAND: ItemType = ItemType {
    full_name: "Server.Items.Sand",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_sand(w, 1)),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1044626,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

static SAND_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_sand(w, v[0].i32())),
}];

/// `new Dressform()`.
pub fn new_dressform(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0xec6;

    ops::new_item_with_id(w, &DRESSFORM, gid)
}

/// `Server.Items.Dressform`.
pub static DRESSFORM: ItemType = ItemType {
    full_name: "Server.Items.Dressform",
    parent: Some(&ITEM_BASE),
    construct: Some(new_dressform),
    default_weight: |_, _| 10.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new Axle(amount)`.
pub fn new_axle(w: &mut muo_server::World, amount: i32) -> muo_server::ids::ItemId {
    let gid = 0x105B;
    let id = ops::new_item_with_id(w, &AXLE, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Server.Items.Axle`.
pub static AXLE: ItemType = ItemType {
    full_name: "Server.Items.Axle",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_axle(w, 1)),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

static AXLE_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_axle(w, v[0].i32())),
}];

/// `new AxleGears(amount)`.
pub fn new_axle_gears(w: &mut muo_server::World, amount: i32) -> muo_server::ids::ItemId {
    let gid = 0x1051;
    let id = ops::new_item_with_id(w, &AXLE_GEARS, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Server.Items.AxleGears`.
pub static AXLE_GEARS: ItemType = ItemType {
    full_name: "Server.Items.AxleGears",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_axle_gears(w, 1)),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

static AXLE_GEARS_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_axle_gears(w, v[0].i32())),
}];

/// `new ClockFrame(amount)`.
pub fn new_clock_frame(w: &mut muo_server::World, amount: i32) -> muo_server::ids::ItemId {
    let gid = 0x104D;
    let id = ops::new_item_with_id(w, &CLOCK_FRAME, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Server.Items.ClockFrame`.
pub static CLOCK_FRAME: ItemType = ItemType {
    full_name: "Server.Items.ClockFrame",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_clock_frame(w, 1)),
    default_weight: |_, _| 2.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

static CLOCK_FRAME_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_clock_frame(w, v[0].i32())),
}];

/// `new ClockParts(amount)`.
pub fn new_clock_parts(w: &mut muo_server::World, amount: i32) -> muo_server::ids::ItemId {
    let gid = 0x104F;
    let id = ops::new_item_with_id(w, &CLOCK_PARTS, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Server.Items.ClockParts`.
pub static CLOCK_PARTS: ItemType = ItemType {
    full_name: "Server.Items.ClockParts",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_clock_parts(w, 1)),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

static CLOCK_PARTS_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_clock_parts(w, v[0].i32())),
}];

/// `new Gears(amount)`.
pub fn new_gears(w: &mut muo_server::World, amount: i32) -> muo_server::ids::ItemId {
    let gid = 0x1053;
    let id = ops::new_item_with_id(w, &GEARS, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Server.Items.Gears`.
pub static GEARS: ItemType = ItemType {
    full_name: "Server.Items.Gears",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_gears(w, 1)),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

static GEARS_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_gears(w, v[0].i32())),
}];

/// `new Globe()`.
pub fn new_globe(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0x1047;

    ops::new_item_with_id(w, &GLOBE, gid)
}

/// `Server.Items.Globe`.
pub static GLOBE: ItemType = ItemType {
    full_name: "Server.Items.Globe",
    parent: Some(&ITEM_BASE),
    construct: Some(new_globe),
    default_weight: |_, _| 3.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, true);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, true)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new Hinge(amount)`.
pub fn new_hinge(w: &mut muo_server::World, amount: i32) -> muo_server::ids::ItemId {
    let gid = 0x1055;
    let id = ops::new_item_with_id(w, &HINGE, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Server.Items.Hinge`.
pub static HINGE: ItemType = ItemType {
    full_name: "Server.Items.Hinge",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_hinge(w, 1)),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

static HINGE_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_hinge(w, v[0].i32())),
}];

/// `new SextantParts(amount)`.
pub fn new_sextant_parts(w: &mut muo_server::World, amount: i32) -> muo_server::ids::ItemId {
    let gid = 0x1059;
    let id = ops::new_item_with_id(w, &SEXTANT_PARTS, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Server.Items.SextantParts`.
pub static SEXTANT_PARTS: ItemType = ItemType {
    full_name: "Server.Items.SextantParts",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_sextant_parts(w, 1)),
    default_weight: |_, _| 2.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

static SEXTANT_PARTS_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_sextant_parts(w, v[0].i32())),
}];

/// `new Springs(amount)`.
pub fn new_springs(w: &mut muo_server::World, amount: i32) -> muo_server::ids::ItemId {
    let gid = 0x105D;
    let id = ops::new_item_with_id(w, &SPRINGS, gid);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `Server.Items.Springs`.
pub static SPRINGS: ItemType = ItemType {
    full_name: "Server.Items.Springs",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_springs(w, 1)),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

static SPRINGS_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_springs(w, v[0].i32())),
}];

/// `new Fork()`.
pub fn new_fork(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0x9F4;

    ops::new_item_with_id(w, &FORK, gid)
}

/// `Server.Items.Fork`.
pub static FORK: ItemType = ItemType {
    full_name: "Server.Items.Fork",
    parent: Some(&ITEM_BASE),
    construct: Some(new_fork),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new ForkLeft()`.
pub fn new_fork_left(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0x9F4;

    ops::new_item_with_id(w, &FORK_LEFT, gid)
}

/// `Server.Items.ForkLeft`.
pub static FORK_LEFT: ItemType = ItemType {
    full_name: "Server.Items.ForkLeft",
    parent: Some(&ITEM_BASE),
    construct: Some(new_fork_left),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new ForkRight()`.
pub fn new_fork_right(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0x9F5;

    ops::new_item_with_id(w, &FORK_RIGHT, gid)
}

/// `Server.Items.ForkRight`.
pub static FORK_RIGHT: ItemType = ItemType {
    full_name: "Server.Items.ForkRight",
    parent: Some(&ITEM_BASE),
    construct: Some(new_fork_right),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new Spoon()`.
pub fn new_spoon(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0x9F8;

    ops::new_item_with_id(w, &SPOON, gid)
}

/// `Server.Items.Spoon`.
pub static SPOON: ItemType = ItemType {
    full_name: "Server.Items.Spoon",
    parent: Some(&ITEM_BASE),
    construct: Some(new_spoon),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new SpoonLeft()`.
pub fn new_spoon_left(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0x9F8;

    ops::new_item_with_id(w, &SPOON_LEFT, gid)
}

/// `Server.Items.SpoonLeft`.
pub static SPOON_LEFT: ItemType = ItemType {
    full_name: "Server.Items.SpoonLeft",
    parent: Some(&ITEM_BASE),
    construct: Some(new_spoon_left),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new SpoonRight()`.
pub fn new_spoon_right(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0x9F9;

    ops::new_item_with_id(w, &SPOON_RIGHT, gid)
}

/// `Server.Items.SpoonRight`.
pub static SPOON_RIGHT: ItemType = ItemType {
    full_name: "Server.Items.SpoonRight",
    parent: Some(&ITEM_BASE),
    construct: Some(new_spoon_right),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new Knife()`.
pub fn new_knife(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0x9F6;

    ops::new_item_with_id(w, &KNIFE, gid)
}

/// `Server.Items.Knife`.
pub static KNIFE: ItemType = ItemType {
    full_name: "Server.Items.Knife",
    parent: Some(&ITEM_BASE),
    construct: Some(new_knife),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new KnifeLeft()`.
pub fn new_knife_left(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0x9F6;

    ops::new_item_with_id(w, &KNIFE_LEFT, gid)
}

/// `Server.Items.KnifeLeft`.
pub static KNIFE_LEFT: ItemType = ItemType {
    full_name: "Server.Items.KnifeLeft",
    parent: Some(&ITEM_BASE),
    construct: Some(new_knife_left),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new KnifeRight()`.
pub fn new_knife_right(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0x9F7;

    ops::new_item_with_id(w, &KNIFE_RIGHT, gid)
}

/// `Server.Items.KnifeRight`.
pub static KNIFE_RIGHT: ItemType = ItemType {
    full_name: "Server.Items.KnifeRight",
    parent: Some(&ITEM_BASE),
    construct: Some(new_knife_right),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `new Plate()`.
pub fn new_plate(w: &mut muo_server::World) -> muo_server::ids::ItemId {
    let gid = 0x9D7;

    ops::new_item_with_id(w, &PLATE, gid)
}

/// `Server.Items.Plate`.
pub static PLATE: ItemType = ItemType {
    full_name: "Server.Items.Plate",
    parent: Some(&ITEM_BASE),
    construct: Some(new_plate),
    default_weight: |_, _| 1.0,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// Every class of this file, with its `[TypeAlias]`es.
pub const ALL: &[(&ItemType, &[&str])] = &[
    (&ANVIL, &[]),
    (&FORGE, &[]),
    (&BLANK_SCROLL, &[]),
    (&BOTTLE, &[]),
    (&SAND, &[]),
    (&DRESSFORM, &[]),
    (&AXLE, &[]),
    (&AXLE_GEARS, &[]),
    (&CLOCK_FRAME, &[]),
    (&CLOCK_PARTS, &[]),
    (&GEARS, &[]),
    (&GLOBE, &[]),
    (&HINGE, &[]),
    (&SEXTANT_PARTS, &[]),
    (&SPRINGS, &[]),
    (&FORK, &[]),
    (&FORK_LEFT, &[]),
    (&FORK_RIGHT, &[]),
    (&SPOON, &[]),
    (&SPOON_LEFT, &[]),
    (&SPOON_RIGHT, &[]),
    (&KNIFE, &[]),
    (&KNIFE_LEFT, &[]),
    (&KNIFE_RIGHT, &[]),
    (&PLATE, &[]),
];

/// The `ICommodity` classes (`IsDeedable` true, or `Core.ML`).
pub const COMMODITIES: &[(&ItemType, bool)] =
    &[(&BLANK_SCROLL, false), (&BOTTLE, false), (&SAND, true)];

fn configure(w: &mut muo_server::World) {
    for (t, always) in COMMODITIES {
        crate::commodity::register(
            w,
            t,
            if *always {
                |_| true
            } else {
                muo_content::core::ml
            },
        );
    }
    add_ctors(w, "Server.Items.BlankScroll", &BLANK_SCROLL_CTOR);
    add_ctors(w, "Server.Items.Bottle", &BOTTLE_CTOR);
    add_ctors(w, "Server.Items.Sand", &SAND_CTOR);
    add_ctors(w, "Server.Items.Axle", &AXLE_CTOR);
    add_ctors(w, "Server.Items.AxleGears", &AXLE_GEARS_CTOR);
    add_ctors(w, "Server.Items.ClockFrame", &CLOCK_FRAME_CTOR);
    add_ctors(w, "Server.Items.ClockParts", &CLOCK_PARTS_CTOR);
    add_ctors(w, "Server.Items.Gears", &GEARS_CTOR);
    add_ctors(w, "Server.Items.Hinge", &HINGE_CTOR);
    add_ctors(w, "Server.Items.SextantParts", &SEXTANT_PARTS_CTOR);
    add_ctors(w, "Server.Items.Springs", &SPRINGS_CTOR);
}

pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.SkillItems.Plain.Configure",
        configure,
    );
}
