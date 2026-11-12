//! Generated: the `[add <type> <args>` overload tables of this crate's `[Constructible]` constructors, in declaration
//! order. Do not edit by hand.

use crate::props::{Ctor, Kind, Param, Value, add_generated_ctors, ctor_amount, ctor_hue};
use crate::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ids::EntityId;
use muo_server::type_registry::{Phase, TypeRegistry};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const UINT: Kind = Kind::Plain(PropType::Numeric(Numeric::UInt));
const STR: Kind = Kind::Plain(PropType::Str);

static C_SERVER_ITEMS_AGAPITEINGOT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.AgapiteIngot", v[0].i32()),
    },
];

static C_SERVER_ITEMS_AGAPITEORE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.AgapiteOre", v[0].i32()),
    },
];

static C_SERVER_ITEMS_APPLE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Apple", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ASHBOARD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.AshBoard", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ASHLOG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.AshLog", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BACON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Bacon", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BANANA: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Banana", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BANANAS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Bananas", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BANDAGE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Bandage", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BANKCHECK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "worth", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::items::gold::new_bank_check(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_BARBEDHIDES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BarbedHides", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BARBEDLEATHER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BarbedLeather", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BATWING: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BatWing", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BLACKPEARL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BlackPearl", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BLACKSCALES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BlackScales", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BLOODMOSS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Bloodmoss", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BLOODWOODBOARD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BloodwoodBoard", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BLOODWOODLOG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BloodwoodLog", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BLUESCALES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BlueScales", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BOARD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Board", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BONE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Bone", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BREADLOAF: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BreadLoaf", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BRONZEINGOT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BronzeIngot", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BRONZEORE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BronzeOre", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CABBAGE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Cabbage", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CANTALOUPE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Cantaloupe", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CAP: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Cap", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CARROT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Carrot", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CHEESESLICE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CheeseSlice", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CHEESEWEDGE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CheeseWedge", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CHEESEWHEEL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CheeseWheel", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CHICKENLEG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ChickenLeg", v[0].i32()),
    },
];

static C_SERVER_ITEMS_COCONUT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Coconut", v[0].i32()),
    },
];

static C_SERVER_ITEMS_COOKEDBIRD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CookedBird", v[0].i32()),
    },
];

static C_SERVER_ITEMS_COPPERINGOT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CopperIngot", v[0].i32()),
    },
];

static C_SERVER_ITEMS_COPPERORE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CopperOre", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CREATUREBACKPACK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "name", kind: STR, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::items::cont::new_creature_backpack(w, &v[0].string().unwrap_or_default())),
    },
];

static C_SERVER_ITEMS_DAEMONBLOOD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.DaemonBlood", v[0].i32()),
    },
];

static C_SERVER_ITEMS_DAEMONBONE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.DaemonBone", v[0].i32()),
    },
];

static C_SERVER_ITEMS_DATES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Dates", v[0].i32()),
    },
];

static C_SERVER_ITEMS_DRAGONSBLOOD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.DragonsBlood", v[0].i32()),
    },
];

static C_SERVER_ITEMS_DULLCOPPERINGOT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.DullCopperIngot", v[0].i32()),
    },
];

static C_SERVER_ITEMS_DULLCOPPERORE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.DullCopperOre", v[0].i32()),
    },
];

static C_SERVER_ITEMS_EAROFCORN: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.EarOfCorn", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FANCYSHIRT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.FancyShirt", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FERTILEDIRT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.FertileDirt", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FISHSTEAK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.FishSteak", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FRENCHBREAD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.FrenchBread", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FRIEDEGGS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.FriedEggs", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FROSTWOODBOARD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.FrostwoodBoard", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FROSTWOODLOG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.FrostwoodLog", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GARLIC: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Garlic", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GINSENG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Ginseng", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GOLD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::items::gold::new_gold(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_GOLDINGOT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.GoldIngot", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GOLDORE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.GoldOre", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GRAPES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Grapes", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GRAVEDUST: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.GraveDust", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GREENGOURD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.GreenGourd", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GREENSCALES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.GreenScales", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HAM: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Ham", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HEARTWOODBOARD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.HeartwoodBoard", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HEARTWOODLOG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.HeartwoodLog", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HIDES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Hides", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HONEYDEWMELON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.HoneydewMelon", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HORNEDHIDES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.HornedHides", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HORNEDLEATHER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.HornedLeather", v[0].i32()),
    },
];

static C_SERVER_ITEMS_IRONINGOT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.IronIngot", v[0].i32()),
    },
];

static C_SERVER_ITEMS_IRONORE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.IronOre", v[0].i32()),
    },
];

static C_SERVER_ITEMS_KEY: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "val", kind: UINT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::items::gold::new_key(w, v[0].i64() as u32)),
    },
];

static C_SERVER_ITEMS_LAMBLEG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.LambLeg", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LEATHER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Leather", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LEMON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Lemon", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LEMONS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Lemons", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LETTUCE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Lettuce", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LIME: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Lime", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LIMES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Limes", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LOG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Log", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MANDRAKEROOT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MandrakeRoot", v[0].i32()),
    },
];

static C_SERVER_ITEMS_NIGHTSHADE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Nightshade", v[0].i32()),
    },
];

static C_SERVER_ITEMS_NOXCRYSTAL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.NoxCrystal", v[0].i32()),
    },
];

static C_SERVER_ITEMS_OAKBOARD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.OakBoard", v[0].i32()),
    },
];

static C_SERVER_ITEMS_OAKLOG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.OakLog", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ONION: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Onion", v[0].i32()),
    },
];

static C_SERVER_ITEMS_OPENCOCONUT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.OpenCoconut", v[0].i32()),
    },
];

static C_SERVER_ITEMS_PEACH: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Peach", v[0].i32()),
    },
];

static C_SERVER_ITEMS_PEAR: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Pear", v[0].i32()),
    },
];

static C_SERVER_ITEMS_PIGIRON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.PigIron", v[0].i32()),
    },
];

static C_SERVER_ITEMS_PUMPKIN: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Pumpkin", v[0].i32()),
    },
];

static C_SERVER_ITEMS_REDSCALES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.RedScales", v[0].i32()),
    },
];

static C_SERVER_ITEMS_RIBS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Ribs", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ROASTPIG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.RoastPig", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SAUSAGE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Sausage", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SHADOWIRONINGOT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ShadowIronIngot", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SHADOWIRONORE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ShadowIronOre", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SLABOFBACON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SlabOfBacon", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SMALLPUMPKIN: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SmallPumpkin", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SMALLWATERMELON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SmallWatermelon", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SPIDERSSILK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SpidersSilk", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SPINEDHIDES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SpinedHides", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SPINEDLEATHER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SpinedLeather", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SPLITCOCONUT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SplitCoconut", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SQUASH: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Squash", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SULFUROUSASH: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SulfurousAsh", v[0].i32()),
    },
];

static C_SERVER_ITEMS_TURNIP: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Turnip", v[0].i32()),
    },
];

static C_SERVER_ITEMS_VALORITEINGOT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ValoriteIngot", v[0].i32()),
    },
];

static C_SERVER_ITEMS_VALORITEORE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ValoriteOre", v[0].i32()),
    },
];

static C_SERVER_ITEMS_VERITEINGOT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.VeriteIngot", v[0].i32()),
    },
];

static C_SERVER_ITEMS_VERITEORE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.VeriteOre", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WATERMELON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Watermelon", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WHITESCALES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.WhiteScales", v[0].i32()),
    },
];

static C_SERVER_ITEMS_YELLOWGOURD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.YellowGourd", v[0].i32()),
    },
];

static C_SERVER_ITEMS_YELLOWSCALES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.YellowScales", v[0].i32()),
    },
];

static C_SERVER_ITEMS_YEWBOARD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.YewBoard", v[0].i32()),
    },
];

static C_SERVER_ITEMS_YEWLOG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.YewLog", v[0].i32()),
    },
];

fn configure(w: &mut World) {
    add_generated_ctors(w, "Server.Items.AgapiteIngot", &C_SERVER_ITEMS_AGAPITEINGOT);
    add_generated_ctors(w, "Server.Items.AgapiteOre", &C_SERVER_ITEMS_AGAPITEORE);
    add_generated_ctors(w, "Server.Items.Apple", &C_SERVER_ITEMS_APPLE);
    add_generated_ctors(w, "Server.Items.AshBoard", &C_SERVER_ITEMS_ASHBOARD);
    add_generated_ctors(w, "Server.Items.AshLog", &C_SERVER_ITEMS_ASHLOG);
    add_generated_ctors(w, "Server.Items.Bacon", &C_SERVER_ITEMS_BACON);
    add_generated_ctors(w, "Server.Items.Banana", &C_SERVER_ITEMS_BANANA);
    add_generated_ctors(w, "Server.Items.Bananas", &C_SERVER_ITEMS_BANANAS);
    add_generated_ctors(w, "Server.Items.Bandage", &C_SERVER_ITEMS_BANDAGE);
    add_generated_ctors(w, "Server.Items.BankCheck", &C_SERVER_ITEMS_BANKCHECK);
    add_generated_ctors(w, "Server.Items.BarbedHides", &C_SERVER_ITEMS_BARBEDHIDES);
    add_generated_ctors(w, "Server.Items.BarbedLeather", &C_SERVER_ITEMS_BARBEDLEATHER);
    add_generated_ctors(w, "Server.Items.BatWing", &C_SERVER_ITEMS_BATWING);
    add_generated_ctors(w, "Server.Items.BlackPearl", &C_SERVER_ITEMS_BLACKPEARL);
    add_generated_ctors(w, "Server.Items.BlackScales", &C_SERVER_ITEMS_BLACKSCALES);
    add_generated_ctors(w, "Server.Items.Bloodmoss", &C_SERVER_ITEMS_BLOODMOSS);
    add_generated_ctors(w, "Server.Items.BloodwoodBoard", &C_SERVER_ITEMS_BLOODWOODBOARD);
    add_generated_ctors(w, "Server.Items.BloodwoodLog", &C_SERVER_ITEMS_BLOODWOODLOG);
    add_generated_ctors(w, "Server.Items.BlueScales", &C_SERVER_ITEMS_BLUESCALES);
    add_generated_ctors(w, "Server.Items.Board", &C_SERVER_ITEMS_BOARD);
    add_generated_ctors(w, "Server.Items.Bone", &C_SERVER_ITEMS_BONE);
    add_generated_ctors(w, "Server.Items.BreadLoaf", &C_SERVER_ITEMS_BREADLOAF);
    add_generated_ctors(w, "Server.Items.BronzeIngot", &C_SERVER_ITEMS_BRONZEINGOT);
    add_generated_ctors(w, "Server.Items.BronzeOre", &C_SERVER_ITEMS_BRONZEORE);
    add_generated_ctors(w, "Server.Items.Cabbage", &C_SERVER_ITEMS_CABBAGE);
    add_generated_ctors(w, "Server.Items.Cantaloupe", &C_SERVER_ITEMS_CANTALOUPE);
    add_generated_ctors(w, "Server.Items.Cap", &C_SERVER_ITEMS_CAP);
    add_generated_ctors(w, "Server.Items.Carrot", &C_SERVER_ITEMS_CARROT);
    add_generated_ctors(w, "Server.Items.CheeseSlice", &C_SERVER_ITEMS_CHEESESLICE);
    add_generated_ctors(w, "Server.Items.CheeseWedge", &C_SERVER_ITEMS_CHEESEWEDGE);
    add_generated_ctors(w, "Server.Items.CheeseWheel", &C_SERVER_ITEMS_CHEESEWHEEL);
    add_generated_ctors(w, "Server.Items.ChickenLeg", &C_SERVER_ITEMS_CHICKENLEG);
    add_generated_ctors(w, "Server.Items.Coconut", &C_SERVER_ITEMS_COCONUT);
    add_generated_ctors(w, "Server.Items.CookedBird", &C_SERVER_ITEMS_COOKEDBIRD);
    add_generated_ctors(w, "Server.Items.CopperIngot", &C_SERVER_ITEMS_COPPERINGOT);
    add_generated_ctors(w, "Server.Items.CopperOre", &C_SERVER_ITEMS_COPPERORE);
    add_generated_ctors(w, "Server.Items.CreatureBackpack", &C_SERVER_ITEMS_CREATUREBACKPACK);
    add_generated_ctors(w, "Server.Items.DaemonBlood", &C_SERVER_ITEMS_DAEMONBLOOD);
    add_generated_ctors(w, "Server.Items.DaemonBone", &C_SERVER_ITEMS_DAEMONBONE);
    add_generated_ctors(w, "Server.Items.Dates", &C_SERVER_ITEMS_DATES);
    add_generated_ctors(w, "Server.Items.DragonsBlood", &C_SERVER_ITEMS_DRAGONSBLOOD);
    add_generated_ctors(w, "Server.Items.DullCopperIngot", &C_SERVER_ITEMS_DULLCOPPERINGOT);
    add_generated_ctors(w, "Server.Items.DullCopperOre", &C_SERVER_ITEMS_DULLCOPPERORE);
    add_generated_ctors(w, "Server.Items.EarOfCorn", &C_SERVER_ITEMS_EAROFCORN);
    add_generated_ctors(w, "Server.Items.FancyShirt", &C_SERVER_ITEMS_FANCYSHIRT);
    add_generated_ctors(w, "Server.Items.FertileDirt", &C_SERVER_ITEMS_FERTILEDIRT);
    add_generated_ctors(w, "Server.Items.FishSteak", &C_SERVER_ITEMS_FISHSTEAK);
    add_generated_ctors(w, "Server.Items.FrenchBread", &C_SERVER_ITEMS_FRENCHBREAD);
    add_generated_ctors(w, "Server.Items.FriedEggs", &C_SERVER_ITEMS_FRIEDEGGS);
    add_generated_ctors(w, "Server.Items.FrostwoodBoard", &C_SERVER_ITEMS_FROSTWOODBOARD);
    add_generated_ctors(w, "Server.Items.FrostwoodLog", &C_SERVER_ITEMS_FROSTWOODLOG);
    add_generated_ctors(w, "Server.Items.Garlic", &C_SERVER_ITEMS_GARLIC);
    add_generated_ctors(w, "Server.Items.Ginseng", &C_SERVER_ITEMS_GINSENG);
    add_generated_ctors(w, "Server.Items.Gold", &C_SERVER_ITEMS_GOLD);
    add_generated_ctors(w, "Server.Items.GoldIngot", &C_SERVER_ITEMS_GOLDINGOT);
    add_generated_ctors(w, "Server.Items.GoldOre", &C_SERVER_ITEMS_GOLDORE);
    add_generated_ctors(w, "Server.Items.Grapes", &C_SERVER_ITEMS_GRAPES);
    add_generated_ctors(w, "Server.Items.GraveDust", &C_SERVER_ITEMS_GRAVEDUST);
    add_generated_ctors(w, "Server.Items.GreenGourd", &C_SERVER_ITEMS_GREENGOURD);
    add_generated_ctors(w, "Server.Items.GreenScales", &C_SERVER_ITEMS_GREENSCALES);
    add_generated_ctors(w, "Server.Items.Ham", &C_SERVER_ITEMS_HAM);
    add_generated_ctors(w, "Server.Items.HeartwoodBoard", &C_SERVER_ITEMS_HEARTWOODBOARD);
    add_generated_ctors(w, "Server.Items.HeartwoodLog", &C_SERVER_ITEMS_HEARTWOODLOG);
    add_generated_ctors(w, "Server.Items.Hides", &C_SERVER_ITEMS_HIDES);
    add_generated_ctors(w, "Server.Items.HoneydewMelon", &C_SERVER_ITEMS_HONEYDEWMELON);
    add_generated_ctors(w, "Server.Items.HornedHides", &C_SERVER_ITEMS_HORNEDHIDES);
    add_generated_ctors(w, "Server.Items.HornedLeather", &C_SERVER_ITEMS_HORNEDLEATHER);
    add_generated_ctors(w, "Server.Items.IronIngot", &C_SERVER_ITEMS_IRONINGOT);
    add_generated_ctors(w, "Server.Items.IronOre", &C_SERVER_ITEMS_IRONORE);
    add_generated_ctors(w, "Server.Items.Key", &C_SERVER_ITEMS_KEY);
    add_generated_ctors(w, "Server.Items.LambLeg", &C_SERVER_ITEMS_LAMBLEG);
    add_generated_ctors(w, "Server.Items.Leather", &C_SERVER_ITEMS_LEATHER);
    add_generated_ctors(w, "Server.Items.Lemon", &C_SERVER_ITEMS_LEMON);
    add_generated_ctors(w, "Server.Items.Lemons", &C_SERVER_ITEMS_LEMONS);
    add_generated_ctors(w, "Server.Items.Lettuce", &C_SERVER_ITEMS_LETTUCE);
    add_generated_ctors(w, "Server.Items.Lime", &C_SERVER_ITEMS_LIME);
    add_generated_ctors(w, "Server.Items.Limes", &C_SERVER_ITEMS_LIMES);
    add_generated_ctors(w, "Server.Items.Log", &C_SERVER_ITEMS_LOG);
    add_generated_ctors(w, "Server.Items.MandrakeRoot", &C_SERVER_ITEMS_MANDRAKEROOT);
    add_generated_ctors(w, "Server.Items.Nightshade", &C_SERVER_ITEMS_NIGHTSHADE);
    add_generated_ctors(w, "Server.Items.NoxCrystal", &C_SERVER_ITEMS_NOXCRYSTAL);
    add_generated_ctors(w, "Server.Items.OakBoard", &C_SERVER_ITEMS_OAKBOARD);
    add_generated_ctors(w, "Server.Items.OakLog", &C_SERVER_ITEMS_OAKLOG);
    add_generated_ctors(w, "Server.Items.Onion", &C_SERVER_ITEMS_ONION);
    add_generated_ctors(w, "Server.Items.OpenCoconut", &C_SERVER_ITEMS_OPENCOCONUT);
    add_generated_ctors(w, "Server.Items.Peach", &C_SERVER_ITEMS_PEACH);
    add_generated_ctors(w, "Server.Items.Pear", &C_SERVER_ITEMS_PEAR);
    add_generated_ctors(w, "Server.Items.PigIron", &C_SERVER_ITEMS_PIGIRON);
    add_generated_ctors(w, "Server.Items.Pumpkin", &C_SERVER_ITEMS_PUMPKIN);
    add_generated_ctors(w, "Server.Items.RedScales", &C_SERVER_ITEMS_REDSCALES);
    add_generated_ctors(w, "Server.Items.Ribs", &C_SERVER_ITEMS_RIBS);
    add_generated_ctors(w, "Server.Items.RoastPig", &C_SERVER_ITEMS_ROASTPIG);
    add_generated_ctors(w, "Server.Items.Sausage", &C_SERVER_ITEMS_SAUSAGE);
    add_generated_ctors(w, "Server.Items.ShadowIronIngot", &C_SERVER_ITEMS_SHADOWIRONINGOT);
    add_generated_ctors(w, "Server.Items.ShadowIronOre", &C_SERVER_ITEMS_SHADOWIRONORE);
    add_generated_ctors(w, "Server.Items.SlabOfBacon", &C_SERVER_ITEMS_SLABOFBACON);
    add_generated_ctors(w, "Server.Items.SmallPumpkin", &C_SERVER_ITEMS_SMALLPUMPKIN);
    add_generated_ctors(w, "Server.Items.SmallWatermelon", &C_SERVER_ITEMS_SMALLWATERMELON);
    add_generated_ctors(w, "Server.Items.SpidersSilk", &C_SERVER_ITEMS_SPIDERSSILK);
    add_generated_ctors(w, "Server.Items.SpinedHides", &C_SERVER_ITEMS_SPINEDHIDES);
    add_generated_ctors(w, "Server.Items.SpinedLeather", &C_SERVER_ITEMS_SPINEDLEATHER);
    add_generated_ctors(w, "Server.Items.SplitCoconut", &C_SERVER_ITEMS_SPLITCOCONUT);
    add_generated_ctors(w, "Server.Items.Squash", &C_SERVER_ITEMS_SQUASH);
    add_generated_ctors(w, "Server.Items.SulfurousAsh", &C_SERVER_ITEMS_SULFUROUSASH);
    add_generated_ctors(w, "Server.Items.Turnip", &C_SERVER_ITEMS_TURNIP);
    add_generated_ctors(w, "Server.Items.ValoriteIngot", &C_SERVER_ITEMS_VALORITEINGOT);
    add_generated_ctors(w, "Server.Items.ValoriteOre", &C_SERVER_ITEMS_VALORITEORE);
    add_generated_ctors(w, "Server.Items.VeriteIngot", &C_SERVER_ITEMS_VERITEINGOT);
    add_generated_ctors(w, "Server.Items.VeriteOre", &C_SERVER_ITEMS_VERITEORE);
    add_generated_ctors(w, "Server.Items.Watermelon", &C_SERVER_ITEMS_WATERMELON);
    add_generated_ctors(w, "Server.Items.WhiteScales", &C_SERVER_ITEMS_WHITESCALES);
    add_generated_ctors(w, "Server.Items.YellowGourd", &C_SERVER_ITEMS_YELLOWGOURD);
    add_generated_ctors(w, "Server.Items.YellowScales", &C_SERVER_ITEMS_YELLOWSCALES);
    add_generated_ctors(w, "Server.Items.YewBoard", &C_SERVER_ITEMS_YEWBOARD);
    add_generated_ctors(w, "Server.Items.YewLog", &C_SERVER_ITEMS_YEWLOG);
}

/// A hand-written table for the same class wins, whichever runs first.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(Phase::Configure, "muo_content.CtorTables.Configure", configure);
}
