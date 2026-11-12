//! `BaseReagent`, the reagents and the three bags of them.
//!
//! The reagents' constructors take `amount = 1` (`[add BlackPearl 20`): each class has a one-overload
//! [`Ctor`] table. A bag drops one stack of every reagent through `factory::make_item_amount`.

use muo_content::factory::make_item_amount;
use muo_content::items::cont::{BAG, new_base_container};
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::commodity;

/// `Server.Items.BaseReagent`: `ICommodity`, `DefaultWeight` 0.1.
pub static BASE_REAGENT: ItemType = ItemType {
    full_name: "Server.Items.BaseReagent",
    construct: None,
    parent: Some(&ITEM_BASE),
    default_weight: |_, _| 0.1,
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

/// `new BaseReagent(itemID, amount)` of class `ty`.
pub fn new_reagent(w: &mut World, ty: &'static ItemType, item_id: i32, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

const AMOUNT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
/// `(int amount = 1)`.
static AMOUNT_1: [Param; 1] = [Param {
    name: "amount",
    kind: AMOUNT,
    default: Some(Value::Int(1)),
    ty: "",
}];
/// `(int amount = 50)` of the bags.
static AMOUNT_50: [Param; 1] = [Param {
    name: "amount",
    kind: AMOUNT,
    default: Some(Value::Int(50)),
    ty: "",
}];

macro_rules! reagents {
    ($($name:ident, $ctor:ident = $full:literal, $gid:expr $(, $field:ident : $v:expr)*;)*) => {$(
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_REAGENT),
            construct: Some(|w| new_reagent(w, &$name, $gid, 1)),
            $($field: $v,)*
            serialize: |w, id, out| {
                (BASE_REAGENT.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_REAGENT.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..BASE_REAGENT
        };
        static $ctor: [Ctor; 1] = [Ctor {
            access: None,
            params: &AMOUNT_1,
            make: |w, v| EntityId::Item(new_reagent(w, &$name, $gid, v[0].i32())),
        }];
    )*};
}

reagents! {
    BAT_WING, C_BAT_WING = "Server.Items.BatWing", 0xF78;
    BLACK_PEARL, C_BLACK_PEARL = "Server.Items.BlackPearl", 0xF7A;
    BLOODMOSS, C_BLOODMOSS = "Server.Items.Bloodmoss", 0xF7B;
    DAEMON_BLOOD, C_DAEMON_BLOOD = "Server.Items.DaemonBlood", 0xF7D;
    DAEMON_BONE, C_DAEMON_BONE = "Server.Items.DaemonBone", 0xF80, default_weight: |_, _| 1.0;
    DEAD_WOOD, C_DEAD_WOOD = "Server.Items.DeadWood", 0xF90;
    DRAGONS_BLOOD, C_DRAGONS_BLOOD = "Server.Items.DragonsBlood", 0x4077;
    GARLIC, C_GARLIC = "Server.Items.Garlic", 0xF84;
    GINSENG, C_GINSENG = "Server.Items.Ginseng", 0xF85;
    GRAVE_DUST, C_GRAVE_DUST = "Server.Items.GraveDust", 0xF8F;
    MANDRAKE_ROOT, C_MANDRAKE_ROOT = "Server.Items.MandrakeRoot", 0xF86;
    NIGHTSHADE, C_NIGHTSHADE = "Server.Items.Nightshade", 0xF88;
    NOX_CRYSTAL, C_NOX_CRYSTAL = "Server.Items.NoxCrystal", 0xF8E;
    PIG_IRON, C_PIG_IRON = "Server.Items.PigIron", 0xF8A;
    SPIDERS_SILK, C_SPIDERS_SILK = "Server.Items.SpidersSilk", 0xF8D;
    SULFUROUS_ASH, C_SULFUROUS_ASH = "Server.Items.SulfurousAsh", 0xF8C;
}

const ALL: [&ItemType; 16] = [
    &BAT_WING,
    &BLACK_PEARL,
    &BLOODMOSS,
    &DAEMON_BLOOD,
    &DAEMON_BONE,
    &DEAD_WOOD,
    &DRAGONS_BLOOD,
    &GARLIC,
    &GINSENG,
    &GRAVE_DUST,
    &MANDRAKE_ROOT,
    &NIGHTSHADE,
    &NOX_CRYSTAL,
    &PIG_IRON,
    &SPIDERS_SILK,
    &SULFUROUS_ASH,
];

// ---- the bags ----

const MAGERY: &[&str] = &[
    "BlackPearl",
    "Bloodmoss",
    "Garlic",
    "Ginseng",
    "MandrakeRoot",
    "Nightshade",
    "SulfurousAsh",
    "SpidersSilk",
];
const ALL_REGS: &[&str] = &[
    "BlackPearl",
    "Bloodmoss",
    "Garlic",
    "Ginseng",
    "MandrakeRoot",
    "Nightshade",
    "SulfurousAsh",
    "SpidersSilk",
    "BatWing",
    "GraveDust",
    "DaemonBlood",
    "NoxCrystal",
    "PigIron",
];
const NECRO: &[&str] = &[
    "BatWing",
    "GraveDust",
    "DaemonBlood",
    "NoxCrystal",
    "PigIron",
];

/// `DropItem(new <name>(amount))` for each name, in order.
pub fn drop_reagents(w: &mut World, bag: ItemId, names: &[&str], amount: i32) {
    for n in names {
        let Some(item) = make_item_amount(w, &format!("Server.Items.{n}"), amount) else {
            continue;
        };
        if let Some(ty) = w.item(bag).map(|i| i.ty) {
            (ty.drop_item)(w, bag, item);
        }
    }
}

/// `new BagOfReagents(amount)` and its two kin, of class `ty` holding `names`.
pub fn new_reagent_bag(
    w: &mut World,
    ty: &'static ItemType,
    names: &[&str],
    amount: i32,
) -> ItemId {
    let id = new_base_container(w, ty, 0xE76);
    drop_reagents(w, id, names, amount);
    id
}

macro_rules! bags {
    ($($name:ident, $ctor:ident = $full:literal, $names:expr, $enc:expr;)*) => {$(
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BAG),
            construct: Some(|w| new_reagent_bag(w, &$name, $names, 50)),
            serialize: |w, id, out| {
                (BAG.serialize)(w, id, out);
                crate::simple::write_version(out, $enc);
            },
            deserialize: |w, id, r| {
                (BAG.deserialize)(w, id, r)?;
                let _version = crate::simple::read_version(r, $enc)?;
                Ok(())
            },
            ..BAG
        };
        static $ctor: [Ctor; 1] = [Ctor {
            access: None,
            params: &AMOUNT_50,
            make: |w, v| EntityId::Item(new_reagent_bag(w, &$name, $names, v[0].i32())),
        }];
    )*};
}

bags! {
    BAG_OF_REAGENTS, C_BAG_OF_REAGENTS = "Server.Items.BagOfReagents", MAGERY, false;
    BAG_OF_ALL_REAGENTS, C_BAG_OF_ALL_REAGENTS = "Server.Items.BagOfAllReagents", ALL_REGS, false;
    BAG_OF_NECRO_REAGENTS, C_BAG_OF_NECRO_REAGENTS = "Server.Items.BagOfNecroReagents", NECRO, false;
    // BagOfNecromancerReagents (a New Haven quest reward): the same five, `[SerializationGenerator(0)]`
    BAG_OF_NECROMANCER_REAGENTS, C_BAG_OF_NECROMANCER_REAGENTS = "Server.Items.BagOfNecromancerReagents", NECRO, true;
}

// ---- BagOfSmokeBombs ----

/// `new BagOfSmokeBombs(amount)`: `amount` smoke bombs.
pub fn new_smoke_bomb_bag(w: &mut World, amount: i32) -> ItemId {
    let id = new_base_container(w, &BAG_OF_SMOKE_BOMBS, 0xE76);
    for _ in 0..amount {
        if let Some(b) = muo_content::factory::make_item(w, "Server.Items.SmokeBomb")
            && let Some(ty) = w.item(id).map(|i| i.ty)
        {
            (ty.drop_item)(w, id, b);
        }
    }
    id
}

static AMOUNT_20: [Param; 1] = [Param {
    name: "amount",
    kind: AMOUNT,
    default: Some(Value::Int(20)),
    ty: "",
}];
static C_BAG_OF_SMOKE_BOMBS: [Ctor; 1] = [Ctor {
    access: None,
    params: &AMOUNT_20,
    make: |w, v| EntityId::Item(new_smoke_bomb_bag(w, v[0].i32())),
}];

/// `Server.Items.BagOfSmokeBombs`; `[SerializationGenerator(0)]`.
pub static BAG_OF_SMOKE_BOMBS: ItemType = ItemType {
    full_name: "Server.Items.BagOfSmokeBombs",
    parent: Some(&BAG),
    construct: Some(|w| new_smoke_bomb_bag(w, 20)),
    serialize: |w, id, out| {
        (BAG.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BAG.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..BAG
};

fn configure(w: &mut World) {
    commodity::register(w, &BASE_REAGENT, |_| true);
    commodity::register(w, &DAEMON_BONE, muo_content::core::ml);
    commodity::register(w, &DRAGONS_BLOOD, muo_content::core::ml);
    let tables: [(&str, &'static [Ctor]); 21] = [
        ("Server.Items.BatWing", &C_BAT_WING),
        ("Server.Items.BlackPearl", &C_BLACK_PEARL),
        ("Server.Items.Bloodmoss", &C_BLOODMOSS),
        ("Server.Items.DaemonBlood", &C_DAEMON_BLOOD),
        ("Server.Items.DaemonBone", &C_DAEMON_BONE),
        ("Server.Items.DeadWood", &C_DEAD_WOOD),
        ("Server.Items.DragonsBlood", &C_DRAGONS_BLOOD),
        ("Server.Items.Garlic", &C_GARLIC),
        ("Server.Items.Ginseng", &C_GINSENG),
        ("Server.Items.GraveDust", &C_GRAVE_DUST),
        ("Server.Items.MandrakeRoot", &C_MANDRAKE_ROOT),
        ("Server.Items.Nightshade", &C_NIGHTSHADE),
        ("Server.Items.NoxCrystal", &C_NOX_CRYSTAL),
        ("Server.Items.PigIron", &C_PIG_IRON),
        ("Server.Items.SpidersSilk", &C_SPIDERS_SILK),
        ("Server.Items.SulfurousAsh", &C_SULFUROUS_ASH),
        ("Server.Items.BagOfReagents", &C_BAG_OF_REAGENTS),
        ("Server.Items.BagOfAllReagents", &C_BAG_OF_ALL_REAGENTS),
        ("Server.Items.BagOfNecroReagents", &C_BAG_OF_NECRO_REAGENTS),
        (
            "Server.Items.BagOfNecromancerReagents",
            &C_BAG_OF_NECROMANCER_REAGENTS,
        ),
        ("Server.Items.BagOfSmokeBombs", &C_BAG_OF_SMOKE_BOMBS),
    ];
    for (name, t) in tables {
        add_ctors(w, name, t);
    }
}

/// Registers the reagent classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BASE_REAGENT, &[]);
    for t in ALL {
        types.register_item(t, &[]);
    }
    for t in [
        &BAG_OF_REAGENTS,
        &BAG_OF_ALL_REAGENTS,
        &BAG_OF_NECRO_REAGENTS,
        &BAG_OF_NECROMANCER_REAGENTS,
        &BAG_OF_SMOKE_BOMBS,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BaseReagent.Configure",
        configure,
    );
}
