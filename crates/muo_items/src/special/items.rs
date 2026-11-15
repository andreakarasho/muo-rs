//! The plain special `Item` classes: a
//! constructor of simple statements and constant overrides. Same shape as `armor_leaf!`.

use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::type_registry::TypeRegistry;

use super::{hue, read_version, write_version};

/// `Server.Items.IcicleLargeSouth`.
pub static ICICLE_LARGE_SOUTH: ItemType = ItemType {
    full_name: "Server.Items.IcicleLargeSouth",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x4572;

        ops::new_item_with_id(w, &ICICLE_LARGE_SOUTH, gid)
    }),
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

/// `Server.Items.IcicleMedSouth`.
pub static ICICLE_MED_SOUTH: ItemType = ItemType {
    full_name: "Server.Items.IcicleMedSouth",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x4573;

        ops::new_item_with_id(w, &ICICLE_MED_SOUTH, gid)
    }),
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

/// `Server.Items.IcicleSmallSouth`.
pub static ICICLE_SMALL_SOUTH: ItemType = ItemType {
    full_name: "Server.Items.IcicleSmallSouth",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x4574;

        ops::new_item_with_id(w, &ICICLE_SMALL_SOUTH, gid)
    }),
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

/// `Server.Items.IcicleLargeEast`.
pub static ICICLE_LARGE_EAST: ItemType = ItemType {
    full_name: "Server.Items.IcicleLargeEast",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x4575;

        ops::new_item_with_id(w, &ICICLE_LARGE_EAST, gid)
    }),
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

/// `Server.Items.IcicleMedEast`.
pub static ICICLE_MED_EAST: ItemType = ItemType {
    full_name: "Server.Items.IcicleMedEast",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x4576;

        ops::new_item_with_id(w, &ICICLE_MED_EAST, gid)
    }),
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

/// `Server.Items.IcicleSmallEast`.
pub static ICICLE_SMALL_EAST: ItemType = ItemType {
    full_name: "Server.Items.IcicleSmallEast",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x4577;

        ops::new_item_with_id(w, &ICICLE_SMALL_EAST, gid)
    }),
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

/// `Server.Items.Coal`.
pub static COAL: ItemType = ItemType {
    full_name: "Server.Items.Coal",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x19B9;
        let id = ops::new_item_with_id(w, &COAL, gid);
        ops::set_stackable(w, id, false);
        ops::set_loot_type(w, id, LootType::Blessed);
        hue(w, id, 0x965);
        id
    }),
    label_number: crate::class_props::label_number,
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

/// `Server.Items.RedPoinsettia`.
pub static RED_POINSETTIA: ItemType = ItemType {
    full_name: "Server.Items.RedPoinsettia",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x2330;
        let id = ops::new_item_with_id(w, &RED_POINSETTIA, gid);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    default_weight: crate::class_props::default_weight,
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

/// `Server.Items.WhitePoinsettia`.
pub static WHITE_POINSETTIA: ItemType = ItemType {
    full_name: "Server.Items.WhitePoinsettia",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x2331;
        let id = ops::new_item_with_id(w, &WHITE_POINSETTIA, gid);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    default_weight: crate::class_props::default_weight,
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

/// `Server.Items.BlueSnowflake`.
pub static BLUE_SNOWFLAKE: ItemType = ItemType {
    full_name: "Server.Items.BlueSnowflake",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x232E;
        let id = ops::new_item_with_id(w, &BLUE_SNOWFLAKE, gid);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    default_weight: crate::class_props::default_weight,
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

/// `Server.Items.WhiteSnowflake`.
pub static WHITE_SNOWFLAKE: ItemType = ItemType {
    full_name: "Server.Items.WhiteSnowflake",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x232F;
        let id = ops::new_item_with_id(w, &WHITE_SNOWFLAKE, gid);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    default_weight: crate::class_props::default_weight,
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

/// `Server.Items.UnfinishedBarrel`.
pub static UNFINISHED_BARREL: ItemType = ItemType {
    full_name: "Server.Items.UnfinishedBarrel",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1EB5;
        let id = ops::new_item_with_id(w, &UNFINISHED_BARREL, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoRock`.
pub static DECO_ROCK: ItemType = ItemType {
    full_name: "Server.Items.DecoRock",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1778;
        let id = ops::new_item_with_id(w, &DECO_ROCK, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoRock2`.
pub static DECO_ROCK2: ItemType = ItemType {
    full_name: "Server.Items.DecoRock2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1363;
        let id = ops::new_item_with_id(w, &DECO_ROCK2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoRocks`.
pub static DECO_ROCKS: ItemType = ItemType {
    full_name: "Server.Items.DecoRocks",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1367;
        let id = ops::new_item_with_id(w, &DECO_ROCKS, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoRocks2`.
pub static DECO_ROCKS2: ItemType = ItemType {
    full_name: "Server.Items.DecoRocks2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x136D;
        let id = ops::new_item_with_id(w, &DECO_ROCKS2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoFlower`.
pub static DECO_FLOWER: ItemType = ItemType {
    full_name: "Server.Items.DecoFlower",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18DA;
        let id = ops::new_item_with_id(w, &DECO_FLOWER, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoFlower2`.
pub static DECO_FLOWER2: ItemType = ItemType {
    full_name: "Server.Items.DecoFlower2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18D9;
        let id = ops::new_item_with_id(w, &DECO_FLOWER2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoRoseOfTrinsic`.
pub static DECO_ROSE_OF_TRINSIC: ItemType = ItemType {
    full_name: "Server.Items.DecoRoseOfTrinsic",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x234C;
        let id = ops::new_item_with_id(w, &DECO_ROSE_OF_TRINSIC, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoRoseOfTrinsic2`.
pub static DECO_ROSE_OF_TRINSIC2: ItemType = ItemType {
    full_name: "Server.Items.DecoRoseOfTrinsic2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x234D;
        let id = ops::new_item_with_id(w, &DECO_ROSE_OF_TRINSIC2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoRoseOfTrinsic3`.
pub static DECO_ROSE_OF_TRINSIC3: ItemType = ItemType {
    full_name: "Server.Items.DecoRoseOfTrinsic3",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x234B;
        let id = ops::new_item_with_id(w, &DECO_ROSE_OF_TRINSIC3, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoBottlesOfLiquor`.
pub static DECO_BOTTLES_OF_LIQUOR: ItemType = ItemType {
    full_name: "Server.Items.DecoBottlesOfLiquor",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x99E;
        let id = ops::new_item_with_id(w, &DECO_BOTTLES_OF_LIQUOR, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoTray2`.
pub static DECO_TRAY2: ItemType = ItemType {
    full_name: "Server.Items.DecoTray2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x991;
        let id = ops::new_item_with_id(w, &DECO_TRAY2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoTray`.
pub static DECO_TRAY: ItemType = ItemType {
    full_name: "Server.Items.DecoTray",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = muo_primitives::utility::random(&mut w.rng, 2) + 0x991;
        let id = ops::new_item_with_id(w, &DECO_TRAY, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.BrokenChair`.
pub static BROKEN_CHAIR: ItemType = ItemType {
    full_name: "Server.Items.BrokenChair",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = muo_primitives::utility::random(&mut w.rng, 2) + 0xC19;
        let id = ops::new_item_with_id(w, &BROKEN_CHAIR, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Checkers`.
pub static CHECKERS: ItemType = ItemType {
    full_name: "Server.Items.Checkers",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE1A;
        let id = ops::new_item_with_id(w, &CHECKERS, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Checkers2`.
pub static CHECKERS2: ItemType = ItemType {
    full_name: "Server.Items.Checkers2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE1B;
        let id = ops::new_item_with_id(w, &CHECKERS2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Chessmen`.
pub static CHESSMEN: ItemType = ItemType {
    full_name: "Server.Items.Chessmen",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE13;
        let id = ops::new_item_with_id(w, &CHESSMEN, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Chessmen2`.
pub static CHESSMEN2: ItemType = ItemType {
    full_name: "Server.Items.Chessmen2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE12;
        let id = ops::new_item_with_id(w, &CHESSMEN2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Chessmen3`.
pub static CHESSMEN3: ItemType = ItemType {
    full_name: "Server.Items.Chessmen3",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE14;
        let id = ops::new_item_with_id(w, &CHESSMEN3, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGoldIngot`.
pub static DECO_GOLD_INGOT: ItemType = ItemType {
    full_name: "Server.Items.DecoGoldIngot",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BE9;
        let id = ops::new_item_with_id(w, &DECO_GOLD_INGOT, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, true);
        id
    }),
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

/// `Server.Items.DecoGoldIngot2`.
pub static DECO_GOLD_INGOT2: ItemType = ItemType {
    full_name: "Server.Items.DecoGoldIngot2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BEC;
        let id = ops::new_item_with_id(w, &DECO_GOLD_INGOT2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGoldIngots`.
pub static DECO_GOLD_INGOTS: ItemType = ItemType {
    full_name: "Server.Items.DecoGoldIngots",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BEA;
        let id = ops::new_item_with_id(w, &DECO_GOLD_INGOTS, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGoldIngots2`.
pub static DECO_GOLD_INGOTS2: ItemType = ItemType {
    full_name: "Server.Items.DecoGoldIngots2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BEB;
        let id = ops::new_item_with_id(w, &DECO_GOLD_INGOTS2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGoldIngots3`.
pub static DECO_GOLD_INGOTS3: ItemType = ItemType {
    full_name: "Server.Items.DecoGoldIngots3",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BED;
        let id = ops::new_item_with_id(w, &DECO_GOLD_INGOTS3, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGoldIngots4`.
pub static DECO_GOLD_INGOTS4: ItemType = ItemType {
    full_name: "Server.Items.DecoGoldIngots4",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BEE;
        let id = ops::new_item_with_id(w, &DECO_GOLD_INGOTS4, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoIronIngot`.
pub static DECO_IRON_INGOT: ItemType = ItemType {
    full_name: "Server.Items.DecoIronIngot",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BEF;
        let id = ops::new_item_with_id(w, &DECO_IRON_INGOT, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, true);
        id
    }),
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

/// `Server.Items.DecoIronIngot2`.
pub static DECO_IRON_INGOT2: ItemType = ItemType {
    full_name: "Server.Items.DecoIronIngot2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BEF;
        let id = ops::new_item_with_id(w, &DECO_IRON_INGOT2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoIronIngots`.
pub static DECO_IRON_INGOTS: ItemType = ItemType {
    full_name: "Server.Items.DecoIronIngots",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BF1;
        let id = ops::new_item_with_id(w, &DECO_IRON_INGOTS, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoIronIngots2`.
pub static DECO_IRON_INGOTS2: ItemType = ItemType {
    full_name: "Server.Items.DecoIronIngots2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BF0;
        let id = ops::new_item_with_id(w, &DECO_IRON_INGOTS2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoIronIngots3`.
pub static DECO_IRON_INGOTS3: ItemType = ItemType {
    full_name: "Server.Items.DecoIronIngots3",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BF0;
        let id = ops::new_item_with_id(w, &DECO_IRON_INGOTS3, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoIronIngots4`.
pub static DECO_IRON_INGOTS4: ItemType = ItemType {
    full_name: "Server.Items.DecoIronIngots4",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BF1;
        let id = ops::new_item_with_id(w, &DECO_IRON_INGOTS4, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoIronIngots5`.
pub static DECO_IRON_INGOTS5: ItemType = ItemType {
    full_name: "Server.Items.DecoIronIngots5",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BF3;
        let id = ops::new_item_with_id(w, &DECO_IRON_INGOTS5, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoIronIngots6`.
pub static DECO_IRON_INGOTS6: ItemType = ItemType {
    full_name: "Server.Items.DecoIronIngots6",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BF4;
        let id = ops::new_item_with_id(w, &DECO_IRON_INGOTS6, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoSilverIngot`.
pub static DECO_SILVER_INGOT: ItemType = ItemType {
    full_name: "Server.Items.DecoSilverIngot",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BF5;
        let id = ops::new_item_with_id(w, &DECO_SILVER_INGOT, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, true);
        id
    }),
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

/// `Server.Items.DecoSilverIngot2`.
pub static DECO_SILVER_INGOT2: ItemType = ItemType {
    full_name: "Server.Items.DecoSilverIngot2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BF8;
        let id = ops::new_item_with_id(w, &DECO_SILVER_INGOT2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoSilverIngots`.
pub static DECO_SILVER_INGOTS: ItemType = ItemType {
    full_name: "Server.Items.DecoSilverIngots",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BFA;
        let id = ops::new_item_with_id(w, &DECO_SILVER_INGOTS, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoSilverIngots2`.
pub static DECO_SILVER_INGOTS2: ItemType = ItemType {
    full_name: "Server.Items.DecoSilverIngots2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BF6;
        let id = ops::new_item_with_id(w, &DECO_SILVER_INGOTS2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoSilverIngots3`.
pub static DECO_SILVER_INGOTS3: ItemType = ItemType {
    full_name: "Server.Items.DecoSilverIngots3",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BF7;
        let id = ops::new_item_with_id(w, &DECO_SILVER_INGOTS3, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoSilverIngots4`.
pub static DECO_SILVER_INGOTS4: ItemType = ItemType {
    full_name: "Server.Items.DecoSilverIngots4",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BF9;
        let id = ops::new_item_with_id(w, &DECO_SILVER_INGOTS4, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoSilverIngots5`.
pub static DECO_SILVER_INGOTS5: ItemType = ItemType {
    full_name: "Server.Items.DecoSilverIngots5",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BFA;
        let id = ops::new_item_with_id(w, &DECO_SILVER_INGOTS5, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.EmptyJar`.
pub static EMPTY_JAR: ItemType = ItemType {
    full_name: "Server.Items.EmptyJar",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1005;
        let id = ops::new_item_with_id(w, &EMPTY_JAR, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.EmptyJars`.
pub static EMPTY_JARS: ItemType = ItemType {
    full_name: "Server.Items.EmptyJars",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE44;
        let id = ops::new_item_with_id(w, &EMPTY_JARS, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.EmptyJars2`.
pub static EMPTY_JARS2: ItemType = ItemType {
    full_name: "Server.Items.EmptyJars2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE45;
        let id = ops::new_item_with_id(w, &EMPTY_JARS2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.EmptyJars3`.
pub static EMPTY_JARS3: ItemType = ItemType {
    full_name: "Server.Items.EmptyJars3",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE46;
        let id = ops::new_item_with_id(w, &EMPTY_JARS3, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.EmptyJars4`.
pub static EMPTY_JARS4: ItemType = ItemType {
    full_name: "Server.Items.EmptyJars4",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE47;
        let id = ops::new_item_with_id(w, &EMPTY_JARS4, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoFullJar`.
pub static DECO_FULL_JAR: ItemType = ItemType {
    full_name: "Server.Items.DecoFullJar",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1006;
        let id = ops::new_item_with_id(w, &DECO_FULL_JAR, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoFullJars3`.
pub static DECO_FULL_JARS3: ItemType = ItemType {
    full_name: "Server.Items.DecoFullJars3",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE4A;
        let id = ops::new_item_with_id(w, &DECO_FULL_JARS3, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoFullJars4`.
pub static DECO_FULL_JARS4: ItemType = ItemType {
    full_name: "Server.Items.DecoFullJars4",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE4B;
        let id = ops::new_item_with_id(w, &DECO_FULL_JARS4, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.HalfEmptyJar`.
pub static HALF_EMPTY_JAR: ItemType = ItemType {
    full_name: "Server.Items.HalfEmptyJar",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1007;
        let id = ops::new_item_with_id(w, &HALF_EMPTY_JAR, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.HalfEmptyJars`.
pub static HALF_EMPTY_JARS: ItemType = ItemType {
    full_name: "Server.Items.HalfEmptyJars",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE4C;
        let id = ops::new_item_with_id(w, &HALF_EMPTY_JARS, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Jars2`.
pub static JARS2: ItemType = ItemType {
    full_name: "Server.Items.Jars2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE4D;
        let id = ops::new_item_with_id(w, &JARS2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Jars3`.
pub static JARS3: ItemType = ItemType {
    full_name: "Server.Items.Jars3",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE4E;
        let id = ops::new_item_with_id(w, &JARS3, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Jars4`.
pub static JARS4: ItemType = ItemType {
    full_name: "Server.Items.Jars4",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE4F;
        let id = ops::new_item_with_id(w, &JARS4, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoCrystalBall`.
pub static DECO_CRYSTAL_BALL: ItemType = ItemType {
    full_name: "Server.Items.DecoCrystalBall",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE2E;
        let id = ops::new_item_with_id(w, &DECO_CRYSTAL_BALL, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoMagicalCrystal`.
pub static DECO_MAGICAL_CRYSTAL: ItemType = ItemType {
    full_name: "Server.Items.DecoMagicalCrystal",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1F19;
        let id = ops::new_item_with_id(w, &DECO_MAGICAL_CRYSTAL, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoSpittoon`.
pub static DECO_SPITTOON: ItemType = ItemType {
    full_name: "Server.Items.DecoSpittoon",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1003;
        let id = ops::new_item_with_id(w, &DECO_SPITTOON, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoBlackmoor`.
pub static DECO_BLACKMOOR: ItemType = ItemType {
    full_name: "Server.Items.DecoBlackmoor",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xF79;
        let id = ops::new_item_with_id(w, &DECO_BLACKMOOR, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoBloodspawn`.
pub static DECO_BLOODSPAWN: ItemType = ItemType {
    full_name: "Server.Items.DecoBloodspawn",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xF7C;
        let id = ops::new_item_with_id(w, &DECO_BLOODSPAWN, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoBrimstone`.
pub static DECO_BRIMSTONE: ItemType = ItemType {
    full_name: "Server.Items.DecoBrimstone",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xF7F;
        let id = ops::new_item_with_id(w, &DECO_BRIMSTONE, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoDragonsBlood`.
pub static DECO_DRAGONS_BLOOD: ItemType = ItemType {
    full_name: "Server.Items.DecoDragonsBlood",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x4077;
        let id = ops::new_item_with_id(w, &DECO_DRAGONS_BLOOD, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoDragonsBlood2`.
pub static DECO_DRAGONS_BLOOD2: ItemType = ItemType {
    full_name: "Server.Items.DecoDragonsBlood2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xF82;
        let id = ops::new_item_with_id(w, &DECO_DRAGONS_BLOOD2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoEyeOfNewt`.
pub static DECO_EYE_OF_NEWT: ItemType = ItemType {
    full_name: "Server.Items.DecoEyeOfNewt",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xF87;
        let id = ops::new_item_with_id(w, &DECO_EYE_OF_NEWT, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGarlic`.
pub static DECO_GARLIC: ItemType = ItemType {
    full_name: "Server.Items.DecoGarlic",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18E1;
        let id = ops::new_item_with_id(w, &DECO_GARLIC, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGarlic2`.
pub static DECO_GARLIC2: ItemType = ItemType {
    full_name: "Server.Items.DecoGarlic2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18E2;
        let id = ops::new_item_with_id(w, &DECO_GARLIC2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGarlicBulb`.
pub static DECO_GARLIC_BULB: ItemType = ItemType {
    full_name: "Server.Items.DecoGarlicBulb",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18E3;
        let id = ops::new_item_with_id(w, &DECO_GARLIC_BULB, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGarlicBulb2`.
pub static DECO_GARLIC_BULB2: ItemType = ItemType {
    full_name: "Server.Items.DecoGarlicBulb2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18E4;
        let id = ops::new_item_with_id(w, &DECO_GARLIC_BULB2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGinseng`.
pub static DECO_GINSENG: ItemType = ItemType {
    full_name: "Server.Items.DecoGinseng",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18E9;
        let id = ops::new_item_with_id(w, &DECO_GINSENG, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGinseng2`.
pub static DECO_GINSENG2: ItemType = ItemType {
    full_name: "Server.Items.DecoGinseng2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18EA;
        let id = ops::new_item_with_id(w, &DECO_GINSENG2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGinsengRoot`.
pub static DECO_GINSENG_ROOT: ItemType = ItemType {
    full_name: "Server.Items.DecoGinsengRoot",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18EB;
        let id = ops::new_item_with_id(w, &DECO_GINSENG_ROOT, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoGinsengRoot2`.
pub static DECO_GINSENG_ROOT2: ItemType = ItemType {
    full_name: "Server.Items.DecoGinsengRoot2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18EC;
        let id = ops::new_item_with_id(w, &DECO_GINSENG_ROOT2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoMandrake`.
pub static DECO_MANDRAKE: ItemType = ItemType {
    full_name: "Server.Items.DecoMandrake",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18DF;
        let id = ops::new_item_with_id(w, &DECO_MANDRAKE, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoMandrake2`.
pub static DECO_MANDRAKE2: ItemType = ItemType {
    full_name: "Server.Items.DecoMandrake2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18E0;
        let id = ops::new_item_with_id(w, &DECO_MANDRAKE2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoMandrake3`.
pub static DECO_MANDRAKE3: ItemType = ItemType {
    full_name: "Server.Items.DecoMandrake3",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18DF;
        let id = ops::new_item_with_id(w, &DECO_MANDRAKE3, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoMandrakeRoot`.
pub static DECO_MANDRAKE_ROOT: ItemType = ItemType {
    full_name: "Server.Items.DecoMandrakeRoot",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18DE;
        let id = ops::new_item_with_id(w, &DECO_MANDRAKE_ROOT, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoMandrakeRoot2`.
pub static DECO_MANDRAKE_ROOT2: ItemType = ItemType {
    full_name: "Server.Items.DecoMandrakeRoot2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18DD;
        let id = ops::new_item_with_id(w, &DECO_MANDRAKE_ROOT2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoNightshade`.
pub static DECO_NIGHTSHADE: ItemType = ItemType {
    full_name: "Server.Items.DecoNightshade",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18E7;
        let id = ops::new_item_with_id(w, &DECO_NIGHTSHADE, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoNightshade2`.
pub static DECO_NIGHTSHADE2: ItemType = ItemType {
    full_name: "Server.Items.DecoNightshade2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18E5;
        let id = ops::new_item_with_id(w, &DECO_NIGHTSHADE2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoNightshade3`.
pub static DECO_NIGHTSHADE3: ItemType = ItemType {
    full_name: "Server.Items.DecoNightshade3",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x18E6;
        let id = ops::new_item_with_id(w, &DECO_NIGHTSHADE3, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoObsidian`.
pub static DECO_OBSIDIAN: ItemType = ItemType {
    full_name: "Server.Items.DecoObsidian",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xF89;
        let id = ops::new_item_with_id(w, &DECO_OBSIDIAN, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoPumice`.
pub static DECO_PUMICE: ItemType = ItemType {
    full_name: "Server.Items.DecoPumice",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xF8B;
        let id = ops::new_item_with_id(w, &DECO_PUMICE, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoWyrmsHeart`.
pub static DECO_WYRMS_HEART: ItemType = ItemType {
    full_name: "Server.Items.DecoWyrmsHeart",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xF91;
        let id = ops::new_item_with_id(w, &DECO_WYRMS_HEART, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Cards`.
pub static CARDS: ItemType = ItemType {
    full_name: "Server.Items.Cards",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE19;
        let id = ops::new_item_with_id(w, &CARDS, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Cards2`.
pub static CARDS2: ItemType = ItemType {
    full_name: "Server.Items.Cards2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE16;
        let id = ops::new_item_with_id(w, &CARDS2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Cards3`.
pub static CARDS3: ItemType = ItemType {
    full_name: "Server.Items.Cards3",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE15;
        let id = ops::new_item_with_id(w, &CARDS3, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Cards4`.
pub static CARDS4: ItemType = ItemType {
    full_name: "Server.Items.Cards4",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE17;
        let id = ops::new_item_with_id(w, &CARDS4, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoCards5`.
pub static DECO_CARDS5: ItemType = ItemType {
    full_name: "Server.Items.DecoCards5",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xE18;
        let id = ops::new_item_with_id(w, &DECO_CARDS5, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.PlayingCards`.
pub static PLAYING_CARDS: ItemType = ItemType {
    full_name: "Server.Items.PlayingCards",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xFA3;
        let id = ops::new_item_with_id(w, &PLAYING_CARDS, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.PlayingCards2`.
pub static PLAYING_CARDS2: ItemType = ItemType {
    full_name: "Server.Items.PlayingCards2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xFA2;
        let id = ops::new_item_with_id(w, &PLAYING_CARDS2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoBridle`.
pub static DECO_BRIDLE: ItemType = ItemType {
    full_name: "Server.Items.DecoBridle",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1374;
        let id = ops::new_item_with_id(w, &DECO_BRIDLE, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoBridle2`.
pub static DECO_BRIDLE2: ItemType = ItemType {
    full_name: "Server.Items.DecoBridle2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1375;
        let id = ops::new_item_with_id(w, &DECO_BRIDLE2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoHay`.
pub static DECO_HAY: ItemType = ItemType {
    full_name: "Server.Items.DecoHay",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xF35;
        let id = ops::new_item_with_id(w, &DECO_HAY, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoHay2`.
pub static DECO_HAY2: ItemType = ItemType {
    full_name: "Server.Items.DecoHay2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xF34;
        let id = ops::new_item_with_id(w, &DECO_HAY2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoHorseDung`.
pub static DECO_HORSE_DUNG: ItemType = ItemType {
    full_name: "Server.Items.DecoHorseDung",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0xF3B;
        let id = ops::new_item_with_id(w, &DECO_HORSE_DUNG, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoDeckOfTarot`.
pub static DECO_DECK_OF_TAROT: ItemType = ItemType {
    full_name: "Server.Items.DecoDeckOfTarot",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x12AB;
        let id = ops::new_item_with_id(w, &DECO_DECK_OF_TAROT, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoDeckOfTarot2`.
pub static DECO_DECK_OF_TAROT2: ItemType = ItemType {
    full_name: "Server.Items.DecoDeckOfTarot2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x12AC;
        let id = ops::new_item_with_id(w, &DECO_DECK_OF_TAROT2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoTarot`.
pub static DECO_TAROT: ItemType = ItemType {
    full_name: "Server.Items.DecoTarot",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x12A5;
        let id = ops::new_item_with_id(w, &DECO_TAROT, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoTarot2`.
pub static DECO_TAROT2: ItemType = ItemType {
    full_name: "Server.Items.DecoTarot2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x12A6;
        let id = ops::new_item_with_id(w, &DECO_TAROT2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoTarot3`.
pub static DECO_TAROT3: ItemType = ItemType {
    full_name: "Server.Items.DecoTarot3",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x12A7;
        let id = ops::new_item_with_id(w, &DECO_TAROT3, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoTarot4`.
pub static DECO_TAROT4: ItemType = ItemType {
    full_name: "Server.Items.DecoTarot4",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x12A8;
        let id = ops::new_item_with_id(w, &DECO_TAROT4, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoTarot5`.
pub static DECO_TAROT5: ItemType = ItemType {
    full_name: "Server.Items.DecoTarot5",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x12A9;
        let id = ops::new_item_with_id(w, &DECO_TAROT5, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoTarot6`.
pub static DECO_TAROT6: ItemType = ItemType {
    full_name: "Server.Items.DecoTarot6",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x12AA;
        let id = ops::new_item_with_id(w, &DECO_TAROT6, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoTarot7`.
pub static DECO_TAROT7: ItemType = ItemType {
    full_name: "Server.Items.DecoTarot7",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x12A5;
        let id = ops::new_item_with_id(w, &DECO_TAROT7, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.DecoArrowShafts`.
pub static DECO_ARROW_SHAFTS: ItemType = ItemType {
    full_name: "Server.Items.DecoArrowShafts",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = muo_primitives::utility::random(&mut w.rng, 2) + 0x1024;
        let id = ops::new_item_with_id(w, &DECO_ARROW_SHAFTS, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.CrossbowBolts`.
pub static CROSSBOW_BOLTS: ItemType = ItemType {
    full_name: "Server.Items.CrossbowBolts",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1BFC;
        let id = ops::new_item_with_id(w, &CROSSBOW_BOLTS, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.EmptyToolKit`.
pub static EMPTY_TOOL_KIT: ItemType = ItemType {
    full_name: "Server.Items.EmptyToolKit",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1EB6;
        let id = ops::new_item_with_id(w, &EMPTY_TOOL_KIT, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.EmptyToolKit2`.
pub static EMPTY_TOOL_KIT2: ItemType = ItemType {
    full_name: "Server.Items.EmptyToolKit2",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x1EB7;
        let id = ops::new_item_with_id(w, &EMPTY_TOOL_KIT2, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.Lockpicks`.
pub static LOCKPICKS: ItemType = ItemType {
    full_name: "Server.Items.Lockpicks",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = muo_primitives::utility::random(&mut w.rng, 2) + 0x14FD;
        let id = ops::new_item_with_id(w, &LOCKPICKS, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.ToolKit`.
pub static TOOL_KIT: ItemType = ItemType {
    full_name: "Server.Items.ToolKit",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = muo_primitives::utility::random(&mut w.rng, 2) + 0x1EBA;
        let id = ops::new_item_with_id(w, &TOOL_KIT, gid);
        ops::set_movable(w, id, true);
        ops::set_stackable(w, id, false);
        id
    }),
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

/// `Server.Items.RoseInAVase`.
pub static ROSE_IN_AVASE: ItemType = ItemType {
    full_name: "Server.Items.RoseInAVase",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let gid = 0x0EB0;
        let id = ops::new_item_with_id(w, &ROSE_IN_AVASE, gid);
        hue(w, id, 0x20);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    label_number: crate::class_props::label_number,
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
    (&ICICLE_LARGE_SOUTH, &[]),
    (&ICICLE_MED_SOUTH, &[]),
    (&ICICLE_SMALL_SOUTH, &[]),
    (&ICICLE_LARGE_EAST, &[]),
    (&ICICLE_MED_EAST, &[]),
    (&ICICLE_SMALL_EAST, &[]),
    (&COAL, &[]),
    (&RED_POINSETTIA, &[]),
    (&WHITE_POINSETTIA, &[]),
    (&BLUE_SNOWFLAKE, &[]),
    (&WHITE_SNOWFLAKE, &[]),
    (&UNFINISHED_BARREL, &[]),
    (&DECO_ROCK, &[]),
    (&DECO_ROCK2, &[]),
    (&DECO_ROCKS, &[]),
    (&DECO_ROCKS2, &[]),
    (&DECO_FLOWER, &[]),
    (&DECO_FLOWER2, &[]),
    (&DECO_ROSE_OF_TRINSIC, &[]),
    (&DECO_ROSE_OF_TRINSIC2, &[]),
    (&DECO_ROSE_OF_TRINSIC3, &[]),
    (&DECO_BOTTLES_OF_LIQUOR, &[]),
    (&DECO_TRAY2, &[]),
    (&DECO_TRAY, &[]),
    (&BROKEN_CHAIR, &[]),
    (&CHECKERS, &[]),
    (&CHECKERS2, &[]),
    (&CHESSMEN, &[]),
    (&CHESSMEN2, &[]),
    (&CHESSMEN3, &[]),
    (&DECO_GOLD_INGOT, &[]),
    (&DECO_GOLD_INGOT2, &[]),
    (&DECO_GOLD_INGOTS, &[]),
    (&DECO_GOLD_INGOTS2, &[]),
    (&DECO_GOLD_INGOTS3, &[]),
    (&DECO_GOLD_INGOTS4, &[]),
    (&DECO_IRON_INGOT, &[]),
    (&DECO_IRON_INGOT2, &[]),
    (&DECO_IRON_INGOTS, &[]),
    (&DECO_IRON_INGOTS2, &[]),
    (&DECO_IRON_INGOTS3, &[]),
    (&DECO_IRON_INGOTS4, &[]),
    (&DECO_IRON_INGOTS5, &[]),
    (&DECO_IRON_INGOTS6, &[]),
    (&DECO_SILVER_INGOT, &[]),
    (&DECO_SILVER_INGOT2, &[]),
    (&DECO_SILVER_INGOTS, &[]),
    (&DECO_SILVER_INGOTS2, &[]),
    (&DECO_SILVER_INGOTS3, &[]),
    (&DECO_SILVER_INGOTS4, &[]),
    (&DECO_SILVER_INGOTS5, &[]),
    (&EMPTY_JAR, &[]),
    (&EMPTY_JARS, &[]),
    (&EMPTY_JARS2, &[]),
    (&EMPTY_JARS3, &[]),
    (&EMPTY_JARS4, &[]),
    (&DECO_FULL_JAR, &[]),
    (&DECO_FULL_JARS3, &[]),
    (&DECO_FULL_JARS4, &[]),
    (&HALF_EMPTY_JAR, &[]),
    (&HALF_EMPTY_JARS, &[]),
    (&JARS2, &[]),
    (&JARS3, &[]),
    (&JARS4, &[]),
    (&DECO_CRYSTAL_BALL, &[]),
    (&DECO_MAGICAL_CRYSTAL, &[]),
    (&DECO_SPITTOON, &[]),
    (&DECO_BLACKMOOR, &[]),
    (&DECO_BLOODSPAWN, &[]),
    (&DECO_BRIMSTONE, &[]),
    (&DECO_DRAGONS_BLOOD, &[]),
    (&DECO_DRAGONS_BLOOD2, &[]),
    (&DECO_EYE_OF_NEWT, &[]),
    (&DECO_GARLIC, &[]),
    (&DECO_GARLIC2, &[]),
    (&DECO_GARLIC_BULB, &[]),
    (&DECO_GARLIC_BULB2, &[]),
    (&DECO_GINSENG, &[]),
    (&DECO_GINSENG2, &[]),
    (&DECO_GINSENG_ROOT, &[]),
    (&DECO_GINSENG_ROOT2, &[]),
    (&DECO_MANDRAKE, &[]),
    (&DECO_MANDRAKE2, &[]),
    (&DECO_MANDRAKE3, &[]),
    (&DECO_MANDRAKE_ROOT, &[]),
    (&DECO_MANDRAKE_ROOT2, &[]),
    (&DECO_NIGHTSHADE, &[]),
    (&DECO_NIGHTSHADE2, &[]),
    (&DECO_NIGHTSHADE3, &[]),
    (&DECO_OBSIDIAN, &[]),
    (&DECO_PUMICE, &[]),
    (&DECO_WYRMS_HEART, &[]),
    (&CARDS, &[]),
    (&CARDS2, &[]),
    (&CARDS3, &[]),
    (&CARDS4, &[]),
    (&DECO_CARDS5, &[]),
    (&PLAYING_CARDS, &[]),
    (&PLAYING_CARDS2, &[]),
    (&DECO_BRIDLE, &[]),
    (&DECO_BRIDLE2, &[]),
    (&DECO_HAY, &[]),
    (&DECO_HAY2, &[]),
    (&DECO_HORSE_DUNG, &[]),
    (&DECO_DECK_OF_TAROT, &[]),
    (&DECO_DECK_OF_TAROT2, &[]),
    (&DECO_TAROT, &[]),
    (&DECO_TAROT2, &[]),
    (&DECO_TAROT3, &[]),
    (&DECO_TAROT4, &[]),
    (&DECO_TAROT5, &[]),
    (&DECO_TAROT6, &[]),
    (&DECO_TAROT7, &[]),
    (&DECO_ARROW_SHAFTS, &[]),
    (&CROSSBOW_BOLTS, &[]),
    (&EMPTY_TOOL_KIT, &[]),
    (&EMPTY_TOOL_KIT2, &[]),
    (&LOCKPICKS, &[]),
    (&TOOL_KIT, &[]),
    (&ROSE_IN_AVASE, &[]),
];

pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in ALL {
        types.register_item(t, aliases);
    }
}
