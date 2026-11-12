//! The `[add <type> <args>` overload tables of this crate's `[Constructible]` constructors, in declaration order.

use muo_content::props::{Ctor, Kind, Param, Value, add_generated_ctors, ctor_amount, ctor_default, ctor_hue, item_ty};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ids::EntityId;
use muo_server::type_registry::{Phase, TypeRegistry};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const ULONG: Kind = Kind::Plain(PropType::Numeric(Numeric::ULong));
const BOOL: Kind = Kind::Plain(PropType::Bool);
const STR: Kind = Kind::Plain(PropType::Str);
const POINT3D: Kind = Kind::Plain(PropType::IPoint3D);

static C_SERVER_ENGINES_EVENTS_PLAYERBONES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "name", kind: STR, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday4::player_bones_new(w, &v[0].string().unwrap_or_default())),
    },
];

static C_SERVER_ITEMS_ACID: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Acid"),
    },
];

static C_SERVER_ITEMS_AGILITYSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.AgilityScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_AMBER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Amber", v[0].i32()),
    },
];

static C_SERVER_ITEMS_AMETHYST: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Amethyst", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ANCIENTSMITHYHAMMER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "bonus", kind: INT, default: None, ty: "" }, Param { name: "uses", kind: INT, default: Some(Value::Int(600)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::rewards2::ancient_smithy_hammer_new(w, v[0].i32(), v[1].i32())),
    },
];

static C_SERVER_ITEMS_ANIMATEDEADSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.AnimateDeadScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ANIMATEDWEAPONSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.AnimatedWeaponScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ANKHOFSACRIFICEADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::vet3::ankh_of_sacrifice_addon_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_ARCANECIRCLESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ArcaneCircleScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ARCANEEMPOWERMENTSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ArcaneEmpowermentScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ARCHCURESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ArchCureScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ARCHPROTECTIONSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ArchProtectionScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ARCHERYBUTTE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: Some(Value::Int(0x100A)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::devices::archery_butte_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_ARROW: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Arrow", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ATTUNEWEAPONSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.AttuneWeaponScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_AXLE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::plain::new_axle(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_AXLEGEARS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::plain::new_axle_gears(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_BAGOFSENDING: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.BagOfSending"),
    },
];

static C_SERVER_ITEMS_BANDANA: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Bandana", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BANNER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::vet2::banner_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_BARKFRAGMENT: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.BarkFragment");
            EntityId::Item(crate::resources::leaves::bark_fragment_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.BarkFragment");
            EntityId::Item(crate::resources::leaves::bark_fragment_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_BEARMASK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.BearMask", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BEESWAX: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Beeswax");
            EntityId::Item(crate::misc::leaves::beeswax_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_BLADESPIRITSSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BladeSpiritsScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BLESSSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BlessScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BLIGHT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Blight");
            EntityId::Item(crate::resources::leaves::blight_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_BLOOD: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Blood"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Blood");
            EntityId::Item(crate::misc::basic::blood_new_with_id(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_BLOODOATHSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BloodOathScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BLOWPIPE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Blowpipe"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_blowpipe(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_BLUEDIAMOND: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.BlueDiamond");
            EntityId::Item(crate::resources::leaves::blue_diamond_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.BlueDiamond");
            EntityId::Item(crate::resources::leaves::blue_diamond_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_BODYSASH: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.BodySash", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BOLA: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Bola");
            EntityId::Item(crate::misc::gadgets::bola_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_BOLABALL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.BolaBall");
            EntityId::Item(crate::misc::leaves::bola_ball_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_BOLT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Bolt", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BOLTOFCLOTH: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BoltOfCloth", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BOMBARDSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BombardScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BONNET: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Bonnet", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BOOTS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Boots", v[0].i32()),
    },
];

static C_SERVER_ITEMS_BOTTLE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::plain::new_bottle(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_BRILLIANTAMBER: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.BrilliantAmber");
            EntityId::Item(crate::resources::leaves::brilliant_amber_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.BrilliantAmber");
            EntityId::Item(crate::resources::leaves::brilliant_amber_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_BROADCASTCRYSTAL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "charges", kind: INT, default: Some(Value::Int(2000)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.BroadcastCrystal");
            EntityId::Item(crate::misc::machines::broadcast_crystal_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_BROCADEGOZAMATEASTADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::brocade_goza_mat_east_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_BROCADEGOZAMATSOUTHADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::brocade_goza_mat_south_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_BROCADESQUAREGOZAMATEASTADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::brocade_square_goza_mat_east_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_BROCADESQUAREGOZAMATSOUTHADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::brocade_square_goza_mat_south_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_BROKENBEDADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::addons::broken_bed_addon_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_BROKENVANITYADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::addons::broken_vanity_addon_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_BUCKET: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "filled", kind: BOOL, default: Some(Value::Bool(false)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::water::new_bucket(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_CAPTUREDESSENCE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.CapturedEssence");
            EntityId::Item(crate::resources::leaves::captured_essence_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_CERAMICMUG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.CeramicMug"),
    },
];

static C_SERVER_ITEMS_CHAINLIGHTNINGSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ChainLightningScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CITRINE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Citrine", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CLEANSINGWINDSSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CleansingWindsScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CLOAK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Cloak");
            EntityId::Item(crate::clothing::special::cloak_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_CLOCK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: Some(Value::Int(0x104B)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::small::new_clock(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_CLOCKFRAME: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::plain::new_clock_frame(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_CLOCKPARTS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::plain::new_clock_parts(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_CLOTHNINJAHOOD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.ClothNinjaHood", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CLOTHNINJAJACKET: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.ClothNinjaJacket", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CLUMSYSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ClumsyScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_COCOAPULP: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.CocoaPulp");
            EntityId::Item(crate::food::leaves::cocoa_pulp_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_CONFIRMATIONMOONGATE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.ConfirmationMoongate"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "target", kind: POINT3D, default: None, ty: "" }, Param { name: "targetMap", kind: Kind::Map, default: Some(Value::Null), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.ConfirmationMoongate");
            EntityId::Item(crate::misc::moongates::confirmation_moongate_new(w, ty, v[0].point(), v[1].map()))
        },
    },
];

static C_SERVER_ITEMS_CONTESTMINIHOUSE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.ContestMiniHouse"),
    },
];

static C_SERVER_ITEMS_CONTESTMINIHOUSEDEED: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.ContestMiniHouseDeed"),
    },
];

static C_SERVER_ITEMS_COPPERWIRE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.CopperWire");
            EntityId::Item(crate::misc::leaves::copper_wire_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_CORPSESKINSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CorpseSkinScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CORRUPTION: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Corruption");
            EntityId::Item(crate::resources::leaves::corruption_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Corruption");
            EntityId::Item(crate::resources::leaves::corruption_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_COTTON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Cotton", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CREATEFOODSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CreateFoodScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CUNNINGSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CunningScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CURESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CureScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CURSESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CurseScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CURSEWEAPONSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.CurseWeaponScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CURTAINSADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::decor::curtains_addon_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_DARKSAPPHIRE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.DarkSapphire");
            EntityId::Item(crate::resources::leaves::dark_sapphire_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.DarkSapphire");
            EntityId::Item(crate::resources::leaves::dark_sapphire_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_DARKYARN: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.DarkYarn", v[0].i32()),
    },
];

static C_SERVER_ITEMS_DARTBOARD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: Some(Value::Bool(true)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::devices::dart_board_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_DAWNSMUSICGEAR: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.DawnsMusicGear"),
    },
];

static C_SERVER_ITEMS_DEADWOOD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.DeadWood", v[0].i32()),
    },
];

static C_SERVER_ITEMS_DECORATIVESHIELD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: Some(Value::Int(0x156C)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::vet3::decorative_shield_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_DEERMASK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.DeerMask", v[0].i32()),
    },
];

static C_SERVER_ITEMS_DIAMOND: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Diamond", v[0].i32()),
    },
];

static C_SERVER_ITEMS_DISEASEDBARK: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.DiseasedBark");
            EntityId::Item(crate::resources::leaves::diseased_bark_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.DiseasedBark");
            EntityId::Item(crate::resources::leaves::diseased_bark_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_DISPELFIELDSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.DispelFieldScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_DISPELSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.DispelScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_DOUBLET: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Doublet", v[0].i32()),
    },
];

static C_SERVER_ITEMS_DOVETAILSAW: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.DovetailSaw"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_dovetail_saw(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_DRAWKNIFE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.DrawKnife"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_draw_knife(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_DREADHORNMANE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.DreadHornMane");
            EntityId::Item(crate::resources::leaves::dread_horn_mane_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.DreadHornMane");
            EntityId::Item(crate::resources::leaves::dread_horn_mane_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_DRIEDHERBS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.DriedHerbs");
            EntityId::Item(crate::misc::leaves::dried_herbs_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_DRIEDONIONS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.DriedOnions");
            EntityId::Item(crate::misc::leaves::dried_onions_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_DRYADALLURESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.DryadAllureScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_EAGLESTRIKESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.EagleStrikeScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_EARRINGSOFPROTECTION: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.EarringsOfProtection"),
    },
];

static C_SERVER_ITEMS_EARTHQUAKESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.EarthquakeScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ECRUCITRINE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.EcruCitrine");
            EntityId::Item(crate::resources::leaves::ecru_citrine_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.EcruCitrine");
            EntityId::Item(crate::resources::leaves::ecru_citrine_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_EGGS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Eggs", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ELVENBOOTS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.ElvenBoots", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ELVENDARKSHIRT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.ElvenDarkShirt", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ELVENPANTS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.ElvenPants", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ELVENSHIRT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.ElvenShirt", v[0].i32()),
    },
];

static C_SERVER_ITEMS_EMERALD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Emerald", v[0].i32()),
    },
];

static C_SERVER_ITEMS_EMPTYOILFLASK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.EmptyOilFlask");
            EntityId::Item(crate::misc::small::empty_oil_flask_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_ENCHANTSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.EnchantScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ENERGYBOLTSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.EnergyBoltScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ENERGYFIELDSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.EnergyFieldScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ENERGYVORTEXSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.EnergyVortexScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ENHANCEDBANDAGE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::anniv2::enhanced_bandage_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_ESSENCEOFWINDSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.EssenceOfWindScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ETHEREALVOYAGESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.EtherealVoyageScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_EVILOMENSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.EvilOmenScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_EXORCISMSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ExorcismScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_EXPLOSIONSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ExplosionScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_EYEOFTHETRAVESTY: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.EyeOfTheTravesty");
            EntityId::Item(crate::resources::leaves::eye_of_the_travesty_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.EyeOfTheTravesty");
            EntityId::Item(crate::resources::leaves::eye_of_the_travesty_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_FANCYDRESS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.FancyDress", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FEATHER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Feather");
            EntityId::Item(crate::resources::leaves::feather_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_FEATHEREDHAT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.FeatheredHat", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FEEBLEMINDSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.FeeblemindScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FEMALEELVENROBE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.FemaleElvenRobe", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FEMALEKIMONO: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.FemaleKimono", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FIRECOLUMNADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "bloody", kind: BOOL, default: Some(Value::Bool(false)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::fire_column_addon_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_FIREFIELDSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.FireFieldScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FIRERUBY: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.FireRuby");
            EntityId::Item(crate::resources::leaves::fire_ruby_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.FireRuby");
            EntityId::Item(crate::resources::leaves::fire_ruby_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_FIREBALLSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.FireballScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FIREBOMB: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: Some(Value::Int(0x99B)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Firebomb");
            EntityId::Item(crate::misc::gadgets::firebomb_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_FIREFLIES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: Some(Value::Int(0x1596)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday5::fireflies_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_FISH: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Fish", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FISHOILFLASK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.FishOilFlask", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FLAMESTRIKESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.FlamestrikeScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FLAX: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Flax", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FLETCHERTOOLS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.FletcherTools"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_fletcher_tools(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_FLOPPYHAT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.FloppyHat", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FLOURSIFTER: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.FlourSifter"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_flour_sifter(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_FLOWERGARLAND: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.FlowerGarland", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FORMALSHIRT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.FormalShirt", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FROE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Froe"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_froe(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_FUKIYADARTS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::ninja::new_fukiya_darts(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_FULLAPRON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.FullApron", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FURBOOTS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.FurBoots", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FURCAPE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.FurCape", v[0].i32()),
    },
];

static C_SERVER_ITEMS_FURSARONG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.FurSarong", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GARGOYLESPICKAXE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.GargoylesPickaxe"),
    },
];

static C_SERVER_ITEMS_GASTRAP: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.GasTrap"),
    },
];

static C_SERVER_ITEMS_GATETRAVELSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.GateTravelScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GEARS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::plain::new_gears(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_GIFTBOX: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.GiftBox"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday::gift_box_new_with(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_GIFTOFLIFESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.GiftOfLifeScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GIFTOFRENEWALSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.GiftOfRenewalScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GILDEDDRESS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.GildedDress", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GLASSMUG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.GlassMug"),
    },
];

static C_SERVER_ITEMS_GOBLET: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Goblet"),
    },
];

static C_SERVER_ITEMS_GOLDWIRE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.GoldWire");
            EntityId::Item(crate::misc::leaves::gold_wire_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_GOZAMATEASTADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::goza_mat_east_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_GOZAMATSOUTHADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::goza_mat_south_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_GREATERHEALSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.GreaterHealScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GREENDRIEDFLOWERS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.GreenDriedFlowers");
            EntityId::Item(crate::misc::leaves::green_dried_flowers_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_GRIZZLEDBONES: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.GrizzledBones");
            EntityId::Item(crate::resources::leaves::grizzled_bones_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.GrizzledBones");
            EntityId::Item(crate::resources::leaves::grizzled_bones_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_HAILSTORMSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.HailStormScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HAKAMA: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Hakama", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HAKAMASHITA: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.HakamaShita", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HALFAPRON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.HalfApron", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HAMMER: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Hammer"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_hammer(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_HANGINGAXESADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::addons::hanging_axes_addon_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_HANGINGSKELETON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: Some(Value::Int(0x1596)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::vet4::hanging_skeleton_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_HANGINGSWORDSADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::addons::hanging_swords_addon_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_HARMSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.HarmScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HEALSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.HealScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HEALINGSTONESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.HealingStoneScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HEARTHOFHOMEFIRE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::gifts::hearth_of_home_fire_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_HINGE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::plain::new_hinge(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_HOLIDAYBELL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.HolidayBell"),
    },
];

static C_SERVER_ITEMS_HOLIDAYPOTTEDPLANT: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.HolidayPottedPlant"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday::holiday_potted_plant_new_with(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_HORNEDTRIBALMASK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.HornedTribalMask", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HORRIFICBEASTSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.HorrificBeastScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HOUSELADDERADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "type", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::decor::house_ladder_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_IMMOLATINGWEAPONSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ImmolatingWeaponScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_INCOGNITOSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.IncognitoScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_INSHAVE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Inshave"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_inshave(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_INTERACTIONTELEPORTER: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.InteractionTeleporter"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "pointDest", kind: POINT3D, default: None, ty: "" }, Param { name: "mapDest", kind: Kind::Map, default: Some(Value::Null), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.InteractionTeleporter");
            EntityId::Item(crate::misc::teleporter::interaction_teleporter_new(w, ty, v[0].point(), v[1].map()))
        },
    },
];

static C_SERVER_ITEMS_INVISIBILITYSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.InvisibilityScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_IRONWIRE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.IronWire");
            EntityId::Item(crate::misc::leaves::iron_wire_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_JACKOLANTERN: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.JackOLantern"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "south", kind: BOOL, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::hand::jack_o_lantern_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_JELLYBEANS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.JellyBeans", v[0].i32()),
    },
];

static C_SERVER_ITEMS_JESTERHAT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.JesterHat", v[0].i32()),
    },
];

static C_SERVER_ITEMS_JESTERHATOFCHUCKLES: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.JesterHatofChuckles"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: None, ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.JesterHatofChuckles", v[0].i32()),
    },
];

static C_SERVER_ITEMS_JESTERSUIT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.JesterSuit", v[0].i32()),
    },
];

static C_SERVER_ITEMS_JINBAORI: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.JinBaori", v[0].i32()),
    },
];

static C_SERVER_ITEMS_JOINTINGPLANE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.JointingPlane"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_jointing_plane(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_KAMISHIMO: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Kamishimo", v[0].i32()),
    },
];

static C_SERVER_ITEMS_KASA: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Kasa", v[0].i32()),
    },
];

static C_SERVER_ITEMS_KILT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Kilt", v[0].i32()),
    },
];

static C_SERVER_ITEMS_KINDLING: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::camping::new_kindling(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_LARDOFPAROXYSMUS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.LardOfParoxysmus");
            EntityId::Item(crate::resources::leaves::lard_of_paroxysmus_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.LardOfParoxysmus");
            EntityId::Item(crate::resources::leaves::lard_of_paroxysmus_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_LARGESTONETABLEEASTADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::large_stone_table_east_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_LARGESTONETABLESOUTHADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::large_stone_table_south_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_LEATHERGLOVESOFMINING: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "bonus", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::rewards2::leather_gloves_of_mining_new_bonus(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_LICHFORMSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.LichFormScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LIGHTOFTHEWINTERSOLSTICE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "dipper", kind: STR, default: Some(Value::Null), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday4::light_of_the_winter_solstice_new(w, Some(v[0].string().as_deref().unwrap_or_default()))),
    },
];

static C_SERVER_ITEMS_LIGHTYARN: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.LightYarn", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LIGHTYARNUNRAVELED: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.LightYarnUnraveled", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LIGHTNINGSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.LightningScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LOCKPICK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::small::new_lockpick(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_LOLLIPOPS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Lollipops", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LONGPANTS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.LongPants", v[0].i32()),
    },
];

static C_SERVER_ITEMS_LUMINESCENTFUNGI: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.LuminescentFungi");
            EntityId::Item(crate::resources::leaves::luminescent_fungi_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_MAGICARROWSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MagicArrowScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MAGICLOCKSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MagicLockScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MAGICREFLECTSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MagicReflectScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MAGICTRAPSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MagicTrapScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MAGICUNTRAPSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MagicUnTrapScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MAGICWIZARDSHAT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.MagicWizardsHat", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MALEELVENROBE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.MaleElvenRobe", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MALEKIMONO: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.MaleKimono", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MALLETANDCHISEL: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.MalletAndChisel"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_mallet_and_chisel(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_MANADRAINSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ManaDrainScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MANAVAMPIRESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ManaVampireScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MAPITEM: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "facet", kind: Kind::Map, default: Some(Value::Null), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.MapItem");
            EntityId::Item(crate::maps::map_item_new(w, ty, v[0].map()))
        },
    },
];

static C_SERVER_ITEMS_MAPMAKERSPEN: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.MapmakersPen"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_mapmakers_pen(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_MARKCONTAINER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "bone", kind: BOOL, default: Some(Value::Bool(false)), ty: "" }, Param { name: "locked", kind: BOOL, default: Some(Value::Bool(false)), ty: "" }],
        make: |w, v| EntityId::Item(crate::last::mark::mark_container_new(w, v[0].bool(), v[1].bool())),
    },
];

static C_SERVER_ITEMS_MARKSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MarkScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MASSCURSESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MassCurseScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MASSDISPELSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MassDispelScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MASSSLEEPSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MassSleepScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MEDIUMSTONETABLEEASTADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::medium_stone_table_east_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_MEDIUMSTONETABLESOUTHADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::medium_stone_table_south_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_METEORSWARMSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MeteorSwarmScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MINDBLASTSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MindBlastScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MINDROTSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.MindRotScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MISTLETOEADDON: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.MistletoeAddon"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday5::mistletoe_addon_new_with(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_MISTLETOEDEED: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday5::mistletoe_deed_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_MONKROBE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0x21E)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.MonkRobe", v[0].i32()),
    },
];

static C_SERVER_ITEMS_MOONGATE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "target", kind: POINT3D, default: None, ty: "" }, Param { name: "targetMap", kind: Kind::Map, default: Some(Value::Null), ty: "" }, Param { name: "dispellable", kind: BOOL, default: Some(Value::Bool(true)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Moongate");
            EntityId::Item(crate::misc::moongates::moongate_new(w, ty, v[0].point(), v[1].map(), v[2].bool()))
        },
    },
];

static C_SERVER_ITEMS_MORPHITEM: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "inactiveItemID", kind: INT, default: None, ty: "" }, Param { name: "activeItemID", kind: INT, default: None, ty: "" }, Param { name: "inRange", kind: INT, default: None, ty: "" }, Param { name: "outRange", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.MorphItem");
            EntityId::Item(crate::misc::basic::morph_item_new(w, ty, v[0].i32(), v[1].i32(), v[2].i32(), v[3].i32()))
        },
    },
];

static C_SERVER_ITEMS_MORTARPESTLE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.MortarPestle"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_mortar_pestle(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_MOULDINGPLANE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.MouldingPlane"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_moulding_plane(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_MUCULENT: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Muculent");
            EntityId::Item(crate::resources::leaves::muculent_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Muculent");
            EntityId::Item(crate::resources::leaves::muculent_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_NAILS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Nails"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_nails(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_NATUREFURYSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.NatureFuryScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_NETHERBOLTSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.NetherBoltScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_NETHERCYCLONESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.NetherCycloneScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_NIGHTSIGHTSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.NightSightScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_NINJATABI: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.NinjaTabi", v[0].i32()),
    },
];

static C_SERVER_ITEMS_NOUGATSWIRL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.NougatSwirl", v[0].i32()),
    },
];

static C_SERVER_ITEMS_OBI: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Obi", v[0].i32()),
    },
];

static C_SERVER_ITEMS_OILFLASK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.OilFlask");
            EntityId::Item(crate::misc::gadgets::oil_flask_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_ORCISHKINMASK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0x8A4)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.OrcishKinMask");
            EntityId::Item(crate::clothing::special::orcish_kin_mask_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_PAINSPIKESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.PainSpikeScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_PARAGONCHEST: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "name", kind: STR, default: None, ty: "" }, Param { name: "level", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::last::chests::paragon_chest_new(w, Some(v[0].string().unwrap_or_default()), v[1].i32())),
    },
];

static C_SERVER_ITEMS_PARALYZEFIELDSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ParalyzeFieldScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_PARALYZESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ParalyzeScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_PARASITICPLANT: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.ParasiticPlant");
            EntityId::Item(crate::resources::leaves::parasitic_plant_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.ParasiticPlant");
            EntityId::Item(crate::resources::leaves::parasitic_plant_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_PERFECTEMERALD: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.PerfectEmerald");
            EntityId::Item(crate::resources::leaves::perfect_emerald_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.PerfectEmerald");
            EntityId::Item(crate::resources::leaves::perfect_emerald_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_PEWTERMUG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.PewterMug"),
    },
];

static C_SERVER_ITEMS_PITCHER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Pitcher"),
    },
];

static C_SERVER_ITEMS_PLAINDRESS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.PlainDress", v[0].i32()),
    },
];

static C_SERVER_ITEMS_POISONFIELDSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.PoisonFieldScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_POISONSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.PoisonScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_POISONSTRIKESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.PoisonStrikeScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_POLYMORPHSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.PolymorphScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_POWDEROFTEMPERAMENT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "charges", kind: INT, default: Some(Value::Int(10)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::rewards2::powder_of_temperament_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_POWDEROFTRANSLOCATION: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::gifts::powder_of_translocation_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_POWERGENERATOR: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.PowerGenerator"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "sideLength", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::misc::machines::power_generator_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_PROJECTEDITEM: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "effectItemId", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::misc::machines::projected_item_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_PROTECTIONSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ProtectionScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_PURGEMAGICSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.PurgeMagicScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_PUTREFICATION: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Putrefication");
            EntityId::Item(crate::resources::leaves::putrefication_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Putrefication");
            EntityId::Item(crate::resources::leaves::putrefication_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_RAWBIRD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.RawBird", v[0].i32()),
    },
];

static C_SERVER_ITEMS_RAWFISHSTEAK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.RawFishSteak", v[0].i32()),
    },
];

static C_SERVER_ITEMS_RAWLAMBLEG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.RawLambLeg", v[0].i32()),
    },
];

static C_SERVER_ITEMS_RAWRIBS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.RawRibs", v[0].i32()),
    },
];

static C_SERVER_ITEMS_REACTIVEARMORSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ReactiveArmorScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_REAPERFORMSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ReaperFormScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_RECALLSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.RecallScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_RECIPESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "recipeID", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::misc::new_recipe_scroll(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_REDVELVETGIFTBOX: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "fill", kind: BOOL, default: Some(Value::Bool(false)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.RedVelvetGiftBox");
            EntityId::Item(crate::special::boxes::red_velvet_gift_box_new(w, ty, v[0].bool()))
        },
    },
];

static C_SERVER_ITEMS_RESURRECTIONSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ResurrectionScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_REVEALSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.RevealScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_REWARDBRAZIER: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.RewardBrazier"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::vet::reward_brazier_new_with(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_REWARDPOTTEDCACTUS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.RewardPottedCactus"),
    },
];

static C_SERVER_ITEMS_RINGMAILGLOVESOFMINING: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "bonus", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::rewards2::ringmail_gloves_of_mining_new_bonus(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_RISINGCOLOSSUSSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.RisingColossusScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ROBE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Robe");
            EntityId::Item(crate::clothing::special::robe_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_ROLLINGPIN: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.RollingPin"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_rolling_pin(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_ROPE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Rope");
            EntityId::Item(crate::misc::leaves::rope_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_ROSEOFTRINSICPETAL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.RoseOfTrinsicPetal", v[0].i32()),
    },
];

static C_SERVER_ITEMS_RUBY: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Ruby", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SHTELEPORTER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "external", kind: BOOL, default: Some(Value::Bool(true)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::hive::sh_teleporter_new_with(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_SOS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "map", kind: Kind::Map, default: Some(Value::Null), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::fishing::new_sos(w, v[0].map(), None)),
    },
    Ctor {
        access: None,
        params: &[Param { name: "map", kind: Kind::Map, default: None, ty: "" }, Param { name: "level", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::fishing::new_sos(w, v[0].map(), Some(v[1].i32()))),
    },
];

static C_SERVER_ITEMS_SACKOFSUGAR: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.SackOfSugar");
            EntityId::Item(crate::food::leaves::sack_of_sugar_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_SAMURAITABI: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.SamuraiTabi", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SAND: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::plain::new_sand(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_SANDALS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Sandals", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SAPPHIRE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Sapphire", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SAVAGEMASK: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.SavageMask"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.SavageMask");
            EntityId::Item(crate::clothing::special::savage_mask_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_SAW: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Saw"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_saw(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_SCORP: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Scorp"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_scorp(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_SCOURGE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Scourge");
            EntityId::Item(crate::resources::leaves::scourge_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Scourge");
            EntityId::Item(crate::resources::leaves::scourge_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_SCRIBESPEN: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.ScribesPen"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_scribes_pen(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_SEWINGKIT: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.SewingKit"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_sewing_kit(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_SEXTANTPARTS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::plain::new_sextant_parts(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_SHAFT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Shaft");
            EntityId::Item(crate::resources::leaves::shaft_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_SHIRT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Shirt", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SHOES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Shoes", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SHORTPANTS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.ShortPants", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SHOVEL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: Some(Value::Int(50)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_shovel(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_SHRINEOFWISDOMCOMPONENT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::shrine_of_wisdom_component_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_SHURIKEN: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::ninja::new_shuriken(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_SILVERWIRE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.SilverWire");
            EntityId::Item(crate::misc::leaves::silver_wire_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_SKILLET: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Skillet"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_skillet(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_SKIRT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Skirt", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SKULLCAP: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.SkullCap", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SLEDGEHAMMER: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.SledgeHammer"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_sledge_hammer(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_SLEEPSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SleepScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SMITHHAMMER: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.SmithHammer"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_smith_hammer(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_SMOOTHINGPLANE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.SmoothingPlane"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_smoothing_plane(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_SNOWGLOBEONE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.SnowGlobeOne"),
    },
];

static C_SERVER_ITEMS_SNOWGLOBETHREE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.SnowGlobeThree"),
    },
];

static C_SERVER_ITEMS_SNOWGLOBETWO: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.SnowGlobeTwo"),
    },
];

static C_SERVER_ITEMS_SNOWPILEDECO: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.SnowPileDeco"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "itemid", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday::snow_pile_deco_new_with(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_SNOWMAN: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Snowman"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday::snowman_new_with(w, v[0].i32(), None)),
    },
];

static C_SERVER_ITEMS_SOULSTONEFRAGMENT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "usesRemaining", kind: INT, default: Some(Value::Int(5)), ty: "" }, Param { name: "account", kind: STR, default: Some(Value::Null), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::soulstone::soulstone_fragment_new(w, v[0].i32(), Some(v[1].string().unwrap_or_default()))),
    },
];

static C_SERVER_ITEMS_SPELLPLAGUESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SpellPlagueScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SPELLTRIGGERSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SpellTriggerScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SPELLBOOK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "content", kind: ULONG, default: Some(Value::Int(0)), ty: "" }, Param { name: "itemID", kind: INT, default: Some(Value::Int(0xEFA)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Spellbook");
            EntityId::Item(crate::skill_items::spellbook::new_spellbook(w, ty, v[0].i64() as u64, v[1].i32()))
        },
    },
];

static C_SERVER_ITEMS_SPOOLOFTHREAD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SpoolOfThread", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SPRINGS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::plain::new_springs(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_SQUAREGOZAMATEASTADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::square_goza_mat_east_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_SQUAREGOZAMATSOUTHADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::leaves::square_goza_mat_south_addon_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_STVALENTINESPANDA: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "name", kind: STR, default: Some(Value::Null), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday4::st_valentines_panda_new(w, Some(v[0].string().as_deref().unwrap_or_default()))),
    },
];

static C_SERVER_ITEMS_STVALENTINESPOLARBEAR: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "name", kind: STR, default: Some(Value::Null), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday4::st_valentines_polar_bear_new(w, Some(v[0].string().as_deref().unwrap_or_default()))),
    },
];

static C_SERVER_ITEMS_STARSAPPHIRE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.StarSapphire", v[0].i32()),
    },
];

static C_SERVER_ITEMS_STATCAPSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "value", kind: INT, default: Some(Value::Int(230)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::scrolls::stat_cap_scroll_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_STONEANKH: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: Some(Value::Bool(true)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::vet::stone_ankh_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_STONEFORMSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.StoneFormScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_STONESTATUEADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::addons::stone_statue_addon_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_STRANGLESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.StrangleScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_STRAWHAT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.StrawHat", v[0].i32()),
    },
];

static C_SERVER_ITEMS_STRENGTHSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.StrengthScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_STUDDEDGLOVESOFMINING: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "bonus", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::rewards2::studded_gloves_of_mining_new_bonus(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_STURDYSHOVEL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: Some(Value::Int(180)), ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_sturdy_shovel(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_SUBTEXTSIGN: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: None, ty: "" }, Param { name: "subtext", kind: STR, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::construction::walls::new_subtext_sign(w, v[0].i32(), Some(v[1].string().unwrap_or_default()))),
    },
];

static C_SERVER_ITEMS_SUMMONAIRELEMENTALSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SummonAirElementalScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SUMMONCREATURESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SummonCreatureScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SUMMONDAEMONSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SummonDaemonScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SUMMONEARTHELEMENTALSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SummonEarthElementalScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SUMMONFAMILIARSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SummonFamiliarScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SUMMONFEYSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SummonFeyScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SUMMONFIENDSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SummonFiendScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SUMMONFIREELEMENTALSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SummonFireElementalScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SUMMONWATERELEMENTALSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.SummonWaterElementalScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SURCOAT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Surcoat", v[0].i32()),
    },
];

static C_SERVER_ITEMS_SWITCHITEM: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.SwitchItem");
            EntityId::Item(crate::resources::leaves::switch_item_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.SwitchItem");
            EntityId::Item(crate::resources::leaves::switch_item_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_TAFFY: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Taffy", v[0].i32()),
    },
];

static C_SERVER_ITEMS_TAINT: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Taint");
            EntityId::Item(crate::resources::leaves::taint_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Taint");
            EntityId::Item(crate::resources::leaves::taint_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_TAINTEDWOOL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.TaintedWool", v[0].i32()),
    },
];

static C_SERVER_ITEMS_TALLSTRAWHAT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.TallStrawHat", v[0].i32()),
    },
];

static C_SERVER_ITEMS_TATTSUKEHAKAMA: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.TattsukeHakama", v[0].i32()),
    },
];

static C_SERVER_ITEMS_TELEKINESISSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.TelekinesisScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_TELEPORTSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.TeleportScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_TELEPORTER: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Teleporter"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "pointDest", kind: POINT3D, default: None, ty: "" }, Param { name: "mapDest", kind: Kind::Map, default: Some(Value::Null), ty: "" }, Param { name: "creatures", kind: BOOL, default: Some(Value::Bool(false)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Teleporter");
            EntityId::Item(crate::misc::teleporter::teleporter_new(w, ty, v[0].point(), v[1].map(), v[2].bool()))
        },
    },
];

static C_SERVER_ITEMS_THIGHBOOTS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.ThighBoots");
            EntityId::Item(crate::clothing::special::thigh_boots_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_THUNDERSTORMSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.ThunderstormScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_TIMEOUTTELEPORTER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.TimeoutTeleporter"),
    },
];

static C_SERVER_ITEMS_TINKERTOOLS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.TinkerTools"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_tinker_tools(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_TINKERSTOOLS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.TinkersTools"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_tinkers_tools(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_TOGGLEITEM: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "inactiveItemID", kind: INT, default: None, ty: "" }, Param { name: "activeItemID", kind: INT, default: None, ty: "" }, Param { name: "playersCanToggle", kind: BOOL, default: Some(Value::Bool(false)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::toggle::toggle_item_new(w, v[0].i32(), v[1].i32(), Some(v[2].bool()))),
    },
];

static C_SERVER_ITEMS_TONGS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Tongs"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::skill_items::tools::new_tongs(w, Some(v[0].i32()))),
    },
];

static C_SERVER_ITEMS_TOURMALINE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Tourmaline", v[0].i32()),
    },
];

static C_SERVER_ITEMS_TRAININGDUMMY: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: Some(Value::Int(0x1074)), ty: "" }],
        make: |w, v| EntityId::Item(crate::addons::devices::training_dummy_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_TREESTUMP: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::vet::tree_stump_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_TRIBALBERRY: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.TribalBerry");
            EntityId::Item(crate::misc::leaves::tribal_berry_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_TRIBALMASK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.TribalMask", v[0].i32()),
    },
];

static C_SERVER_ITEMS_TRICORNEHAT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.TricorneHat", v[0].i32()),
    },
];

static C_SERVER_ITEMS_TUB: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "filled", kind: BOOL, default: Some(Value::Bool(false)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::water::new_tub(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_TUNIC: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Tunic", v[0].i32()),
    },
];

static C_SERVER_ITEMS_TURQUOISE: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[Param { name: "amountFrom", kind: INT, default: None, ty: "" }, Param { name: "amountTo", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Turquoise");
            EntityId::Item(crate::resources::leaves::turquoise_new_1(w, ty, v[0].i32(), v[1].i32()))
        },
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Turquoise");
            EntityId::Item(crate::resources::leaves::turquoise_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_UNLOCKSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.UnlockScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_UNMADEBEDADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::addons::unmade_bed_addon_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_VALENTINESCARD: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemid", kind: INT, default: None, ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.ValentinesCard");
            EntityId::Item(crate::special::boxes::valentines_card_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_VAMPIRICEMBRACESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.VampiricEmbraceScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_VANILLA: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.Vanilla");
            EntityId::Item(crate::food::leaves::vanilla_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_VANITYADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::addons::vanity_addon_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_VENGEFULSPIRITSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.VengefulSpiritScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_VINES: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Vines"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "v", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::small_items::new_vines(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_WALLBANNER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "bannerID", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::vet5::wall_banner_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_WALLOFSTONESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.WallOfStoneScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WARAJI: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.Waraji", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WATERBARREL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "filled", kind: BOOL, default: Some(Value::Bool(false)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::water::new_water_barrel(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_WEAKENSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.WeakenScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WEAPONENGRAVINGTOOL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "uses", kind: INT, default: Some(Value::Int(10)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::vet5::weapon_engraving_tool_new(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_WEB: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Web"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "itemid", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::small_items::new_web(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_WHEATSHEAF: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.WheatSheaf", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WHITEDRIEDFLOWERS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.WhiteDriedFlowers");
            EntityId::Item(crate::misc::leaves::white_dried_flowers_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_WIDEBRIMHAT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.WideBrimHat", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WILDFIRESCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.WildfireScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WITHERSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.WitherScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WIZARDSHAT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.WizardsHat", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WOODENCOFFINADDON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "east", kind: BOOL, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::addons::wooden_coffin_addon_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_WOODLANDBELT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.WoodlandBelt", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WOOL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Wool", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WORDOFDEATHSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.WordOfDeathScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WRAITHFORMSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.WraithFormScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WRAPPEDCANDY: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.WrappedCandy", v[0].i32()),
    },
];

static C_SERVER_ITEMS_WREATHADDON: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.WreathAddon"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday2::wreath_addon_new_with(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_WREATHDEED: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.WreathDeed"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::special::holiday2::wreath_deed_new_with(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_ZOOGIFUNGUS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| EntityId::Item(crate::special::gifts::zoogi_fungus_new(w, v[0].i32())),
    },
];

fn configure(w: &mut World) {
    add_generated_ctors(w, "Server.Engines.Events.PlayerBones", &C_SERVER_ENGINES_EVENTS_PLAYERBONES);
    add_generated_ctors(w, "Server.Items.Acid", &C_SERVER_ITEMS_ACID);
    add_generated_ctors(w, "Server.Items.AgilityScroll", &C_SERVER_ITEMS_AGILITYSCROLL);
    add_generated_ctors(w, "Server.Items.Amber", &C_SERVER_ITEMS_AMBER);
    add_generated_ctors(w, "Server.Items.Amethyst", &C_SERVER_ITEMS_AMETHYST);
    add_generated_ctors(w, "Server.Items.AncientSmithyHammer", &C_SERVER_ITEMS_ANCIENTSMITHYHAMMER);
    add_generated_ctors(w, "Server.Items.AnimateDeadScroll", &C_SERVER_ITEMS_ANIMATEDEADSCROLL);
    add_generated_ctors(w, "Server.Items.AnimatedWeaponScroll", &C_SERVER_ITEMS_ANIMATEDWEAPONSCROLL);
    add_generated_ctors(w, "Server.Items.AnkhOfSacrificeAddon", &C_SERVER_ITEMS_ANKHOFSACRIFICEADDON);
    add_generated_ctors(w, "Server.Items.ArcaneCircleScroll", &C_SERVER_ITEMS_ARCANECIRCLESCROLL);
    add_generated_ctors(w, "Server.Items.ArcaneEmpowermentScroll", &C_SERVER_ITEMS_ARCANEEMPOWERMENTSCROLL);
    add_generated_ctors(w, "Server.Items.ArchCureScroll", &C_SERVER_ITEMS_ARCHCURESCROLL);
    add_generated_ctors(w, "Server.Items.ArchProtectionScroll", &C_SERVER_ITEMS_ARCHPROTECTIONSCROLL);
    add_generated_ctors(w, "Server.Items.ArcheryButte", &C_SERVER_ITEMS_ARCHERYBUTTE);
    add_generated_ctors(w, "Server.Items.Arrow", &C_SERVER_ITEMS_ARROW);
    add_generated_ctors(w, "Server.Items.AttuneWeaponScroll", &C_SERVER_ITEMS_ATTUNEWEAPONSCROLL);
    add_generated_ctors(w, "Server.Items.Axle", &C_SERVER_ITEMS_AXLE);
    add_generated_ctors(w, "Server.Items.AxleGears", &C_SERVER_ITEMS_AXLEGEARS);
    add_generated_ctors(w, "Server.Items.BagOfSending", &C_SERVER_ITEMS_BAGOFSENDING);
    add_generated_ctors(w, "Server.Items.Bandana", &C_SERVER_ITEMS_BANDANA);
    add_generated_ctors(w, "Server.Items.Banner", &C_SERVER_ITEMS_BANNER);
    add_generated_ctors(w, "Server.Items.BarkFragment", &C_SERVER_ITEMS_BARKFRAGMENT);
    add_generated_ctors(w, "Server.Items.BearMask", &C_SERVER_ITEMS_BEARMASK);
    add_generated_ctors(w, "Server.Items.Beeswax", &C_SERVER_ITEMS_BEESWAX);
    add_generated_ctors(w, "Server.Items.BladeSpiritsScroll", &C_SERVER_ITEMS_BLADESPIRITSSCROLL);
    add_generated_ctors(w, "Server.Items.BlessScroll", &C_SERVER_ITEMS_BLESSSCROLL);
    add_generated_ctors(w, "Server.Items.Blight", &C_SERVER_ITEMS_BLIGHT);
    add_generated_ctors(w, "Server.Items.Blood", &C_SERVER_ITEMS_BLOOD);
    add_generated_ctors(w, "Server.Items.BloodOathScroll", &C_SERVER_ITEMS_BLOODOATHSCROLL);
    add_generated_ctors(w, "Server.Items.Blowpipe", &C_SERVER_ITEMS_BLOWPIPE);
    add_generated_ctors(w, "Server.Items.BlueDiamond", &C_SERVER_ITEMS_BLUEDIAMOND);
    add_generated_ctors(w, "Server.Items.BodySash", &C_SERVER_ITEMS_BODYSASH);
    add_generated_ctors(w, "Server.Items.Bola", &C_SERVER_ITEMS_BOLA);
    add_generated_ctors(w, "Server.Items.BolaBall", &C_SERVER_ITEMS_BOLABALL);
    add_generated_ctors(w, "Server.Items.Bolt", &C_SERVER_ITEMS_BOLT);
    add_generated_ctors(w, "Server.Items.BoltOfCloth", &C_SERVER_ITEMS_BOLTOFCLOTH);
    add_generated_ctors(w, "Server.Items.BombardScroll", &C_SERVER_ITEMS_BOMBARDSCROLL);
    add_generated_ctors(w, "Server.Items.Bonnet", &C_SERVER_ITEMS_BONNET);
    add_generated_ctors(w, "Server.Items.Boots", &C_SERVER_ITEMS_BOOTS);
    add_generated_ctors(w, "Server.Items.Bottle", &C_SERVER_ITEMS_BOTTLE);
    add_generated_ctors(w, "Server.Items.BrilliantAmber", &C_SERVER_ITEMS_BRILLIANTAMBER);
    add_generated_ctors(w, "Server.Items.BroadcastCrystal", &C_SERVER_ITEMS_BROADCASTCRYSTAL);
    add_generated_ctors(w, "Server.Items.BrocadeGozaMatEastAddon", &C_SERVER_ITEMS_BROCADEGOZAMATEASTADDON);
    add_generated_ctors(w, "Server.Items.BrocadeGozaMatSouthAddon", &C_SERVER_ITEMS_BROCADEGOZAMATSOUTHADDON);
    add_generated_ctors(w, "Server.Items.BrocadeSquareGozaMatEastAddon", &C_SERVER_ITEMS_BROCADESQUAREGOZAMATEASTADDON);
    add_generated_ctors(w, "Server.Items.BrocadeSquareGozaMatSouthAddon", &C_SERVER_ITEMS_BROCADESQUAREGOZAMATSOUTHADDON);
    add_generated_ctors(w, "Server.Items.BrokenBedAddon", &C_SERVER_ITEMS_BROKENBEDADDON);
    add_generated_ctors(w, "Server.Items.BrokenVanityAddon", &C_SERVER_ITEMS_BROKENVANITYADDON);
    add_generated_ctors(w, "Server.Items.Bucket", &C_SERVER_ITEMS_BUCKET);
    add_generated_ctors(w, "Server.Items.CapturedEssence", &C_SERVER_ITEMS_CAPTUREDESSENCE);
    add_generated_ctors(w, "Server.Items.CeramicMug", &C_SERVER_ITEMS_CERAMICMUG);
    add_generated_ctors(w, "Server.Items.ChainLightningScroll", &C_SERVER_ITEMS_CHAINLIGHTNINGSCROLL);
    add_generated_ctors(w, "Server.Items.Citrine", &C_SERVER_ITEMS_CITRINE);
    add_generated_ctors(w, "Server.Items.CleansingWindsScroll", &C_SERVER_ITEMS_CLEANSINGWINDSSCROLL);
    add_generated_ctors(w, "Server.Items.Cloak", &C_SERVER_ITEMS_CLOAK);
    add_generated_ctors(w, "Server.Items.Clock", &C_SERVER_ITEMS_CLOCK);
    add_generated_ctors(w, "Server.Items.ClockFrame", &C_SERVER_ITEMS_CLOCKFRAME);
    add_generated_ctors(w, "Server.Items.ClockParts", &C_SERVER_ITEMS_CLOCKPARTS);
    add_generated_ctors(w, "Server.Items.ClothNinjaHood", &C_SERVER_ITEMS_CLOTHNINJAHOOD);
    add_generated_ctors(w, "Server.Items.ClothNinjaJacket", &C_SERVER_ITEMS_CLOTHNINJAJACKET);
    add_generated_ctors(w, "Server.Items.ClumsyScroll", &C_SERVER_ITEMS_CLUMSYSCROLL);
    add_generated_ctors(w, "Server.Items.CocoaPulp", &C_SERVER_ITEMS_COCOAPULP);
    add_generated_ctors(w, "Server.Items.ConfirmationMoongate", &C_SERVER_ITEMS_CONFIRMATIONMOONGATE);
    add_generated_ctors(w, "Server.Items.ContestMiniHouse", &C_SERVER_ITEMS_CONTESTMINIHOUSE);
    add_generated_ctors(w, "Server.Items.ContestMiniHouseDeed", &C_SERVER_ITEMS_CONTESTMINIHOUSEDEED);
    add_generated_ctors(w, "Server.Items.CopperWire", &C_SERVER_ITEMS_COPPERWIRE);
    add_generated_ctors(w, "Server.Items.CorpseSkinScroll", &C_SERVER_ITEMS_CORPSESKINSCROLL);
    add_generated_ctors(w, "Server.Items.Corruption", &C_SERVER_ITEMS_CORRUPTION);
    add_generated_ctors(w, "Server.Items.Cotton", &C_SERVER_ITEMS_COTTON);
    add_generated_ctors(w, "Server.Items.CreateFoodScroll", &C_SERVER_ITEMS_CREATEFOODSCROLL);
    add_generated_ctors(w, "Server.Items.CunningScroll", &C_SERVER_ITEMS_CUNNINGSCROLL);
    add_generated_ctors(w, "Server.Items.CureScroll", &C_SERVER_ITEMS_CURESCROLL);
    add_generated_ctors(w, "Server.Items.CurseScroll", &C_SERVER_ITEMS_CURSESCROLL);
    add_generated_ctors(w, "Server.Items.CurseWeaponScroll", &C_SERVER_ITEMS_CURSEWEAPONSCROLL);
    add_generated_ctors(w, "Server.Items.CurtainsAddon", &C_SERVER_ITEMS_CURTAINSADDON);
    add_generated_ctors(w, "Server.Items.DarkSapphire", &C_SERVER_ITEMS_DARKSAPPHIRE);
    add_generated_ctors(w, "Server.Items.DarkYarn", &C_SERVER_ITEMS_DARKYARN);
    add_generated_ctors(w, "Server.Items.DartBoard", &C_SERVER_ITEMS_DARTBOARD);
    add_generated_ctors(w, "Server.Items.DawnsMusicGear", &C_SERVER_ITEMS_DAWNSMUSICGEAR);
    add_generated_ctors(w, "Server.Items.DeadWood", &C_SERVER_ITEMS_DEADWOOD);
    add_generated_ctors(w, "Server.Items.DecorativeShield", &C_SERVER_ITEMS_DECORATIVESHIELD);
    add_generated_ctors(w, "Server.Items.DeerMask", &C_SERVER_ITEMS_DEERMASK);
    add_generated_ctors(w, "Server.Items.Diamond", &C_SERVER_ITEMS_DIAMOND);
    add_generated_ctors(w, "Server.Items.DiseasedBark", &C_SERVER_ITEMS_DISEASEDBARK);
    add_generated_ctors(w, "Server.Items.DispelFieldScroll", &C_SERVER_ITEMS_DISPELFIELDSCROLL);
    add_generated_ctors(w, "Server.Items.DispelScroll", &C_SERVER_ITEMS_DISPELSCROLL);
    add_generated_ctors(w, "Server.Items.Doublet", &C_SERVER_ITEMS_DOUBLET);
    add_generated_ctors(w, "Server.Items.DovetailSaw", &C_SERVER_ITEMS_DOVETAILSAW);
    add_generated_ctors(w, "Server.Items.DrawKnife", &C_SERVER_ITEMS_DRAWKNIFE);
    add_generated_ctors(w, "Server.Items.DreadHornMane", &C_SERVER_ITEMS_DREADHORNMANE);
    add_generated_ctors(w, "Server.Items.DriedHerbs", &C_SERVER_ITEMS_DRIEDHERBS);
    add_generated_ctors(w, "Server.Items.DriedOnions", &C_SERVER_ITEMS_DRIEDONIONS);
    add_generated_ctors(w, "Server.Items.DryadAllureScroll", &C_SERVER_ITEMS_DRYADALLURESCROLL);
    add_generated_ctors(w, "Server.Items.EagleStrikeScroll", &C_SERVER_ITEMS_EAGLESTRIKESCROLL);
    add_generated_ctors(w, "Server.Items.EarringsOfProtection", &C_SERVER_ITEMS_EARRINGSOFPROTECTION);
    add_generated_ctors(w, "Server.Items.EarthquakeScroll", &C_SERVER_ITEMS_EARTHQUAKESCROLL);
    add_generated_ctors(w, "Server.Items.EcruCitrine", &C_SERVER_ITEMS_ECRUCITRINE);
    add_generated_ctors(w, "Server.Items.Eggs", &C_SERVER_ITEMS_EGGS);
    add_generated_ctors(w, "Server.Items.ElvenBoots", &C_SERVER_ITEMS_ELVENBOOTS);
    add_generated_ctors(w, "Server.Items.ElvenDarkShirt", &C_SERVER_ITEMS_ELVENDARKSHIRT);
    add_generated_ctors(w, "Server.Items.ElvenPants", &C_SERVER_ITEMS_ELVENPANTS);
    add_generated_ctors(w, "Server.Items.ElvenShirt", &C_SERVER_ITEMS_ELVENSHIRT);
    add_generated_ctors(w, "Server.Items.Emerald", &C_SERVER_ITEMS_EMERALD);
    add_generated_ctors(w, "Server.Items.EmptyOilFlask", &C_SERVER_ITEMS_EMPTYOILFLASK);
    add_generated_ctors(w, "Server.Items.EnchantScroll", &C_SERVER_ITEMS_ENCHANTSCROLL);
    add_generated_ctors(w, "Server.Items.EnergyBoltScroll", &C_SERVER_ITEMS_ENERGYBOLTSCROLL);
    add_generated_ctors(w, "Server.Items.EnergyFieldScroll", &C_SERVER_ITEMS_ENERGYFIELDSCROLL);
    add_generated_ctors(w, "Server.Items.EnergyVortexScroll", &C_SERVER_ITEMS_ENERGYVORTEXSCROLL);
    add_generated_ctors(w, "Server.Items.EnhancedBandage", &C_SERVER_ITEMS_ENHANCEDBANDAGE);
    add_generated_ctors(w, "Server.Items.EssenceOfWindScroll", &C_SERVER_ITEMS_ESSENCEOFWINDSCROLL);
    add_generated_ctors(w, "Server.Items.EtherealVoyageScroll", &C_SERVER_ITEMS_ETHEREALVOYAGESCROLL);
    add_generated_ctors(w, "Server.Items.EvilOmenScroll", &C_SERVER_ITEMS_EVILOMENSCROLL);
    add_generated_ctors(w, "Server.Items.ExorcismScroll", &C_SERVER_ITEMS_EXORCISMSCROLL);
    add_generated_ctors(w, "Server.Items.ExplosionScroll", &C_SERVER_ITEMS_EXPLOSIONSCROLL);
    add_generated_ctors(w, "Server.Items.EyeOfTheTravesty", &C_SERVER_ITEMS_EYEOFTHETRAVESTY);
    add_generated_ctors(w, "Server.Items.FancyDress", &C_SERVER_ITEMS_FANCYDRESS);
    add_generated_ctors(w, "Server.Items.Feather", &C_SERVER_ITEMS_FEATHER);
    add_generated_ctors(w, "Server.Items.FeatheredHat", &C_SERVER_ITEMS_FEATHEREDHAT);
    add_generated_ctors(w, "Server.Items.FeeblemindScroll", &C_SERVER_ITEMS_FEEBLEMINDSCROLL);
    add_generated_ctors(w, "Server.Items.FemaleElvenRobe", &C_SERVER_ITEMS_FEMALEELVENROBE);
    add_generated_ctors(w, "Server.Items.FemaleKimono", &C_SERVER_ITEMS_FEMALEKIMONO);
    add_generated_ctors(w, "Server.Items.FireColumnAddon", &C_SERVER_ITEMS_FIRECOLUMNADDON);
    add_generated_ctors(w, "Server.Items.FireFieldScroll", &C_SERVER_ITEMS_FIREFIELDSCROLL);
    add_generated_ctors(w, "Server.Items.FireRuby", &C_SERVER_ITEMS_FIRERUBY);
    add_generated_ctors(w, "Server.Items.FireballScroll", &C_SERVER_ITEMS_FIREBALLSCROLL);
    add_generated_ctors(w, "Server.Items.Firebomb", &C_SERVER_ITEMS_FIREBOMB);
    add_generated_ctors(w, "Server.Items.Fireflies", &C_SERVER_ITEMS_FIREFLIES);
    add_generated_ctors(w, "Server.Items.Fish", &C_SERVER_ITEMS_FISH);
    add_generated_ctors(w, "Server.Items.FishOilFlask", &C_SERVER_ITEMS_FISHOILFLASK);
    add_generated_ctors(w, "Server.Items.FlamestrikeScroll", &C_SERVER_ITEMS_FLAMESTRIKESCROLL);
    add_generated_ctors(w, "Server.Items.Flax", &C_SERVER_ITEMS_FLAX);
    add_generated_ctors(w, "Server.Items.FletcherTools", &C_SERVER_ITEMS_FLETCHERTOOLS);
    add_generated_ctors(w, "Server.Items.FloppyHat", &C_SERVER_ITEMS_FLOPPYHAT);
    add_generated_ctors(w, "Server.Items.FlourSifter", &C_SERVER_ITEMS_FLOURSIFTER);
    add_generated_ctors(w, "Server.Items.FlowerGarland", &C_SERVER_ITEMS_FLOWERGARLAND);
    add_generated_ctors(w, "Server.Items.FormalShirt", &C_SERVER_ITEMS_FORMALSHIRT);
    add_generated_ctors(w, "Server.Items.Froe", &C_SERVER_ITEMS_FROE);
    add_generated_ctors(w, "Server.Items.FukiyaDarts", &C_SERVER_ITEMS_FUKIYADARTS);
    add_generated_ctors(w, "Server.Items.FullApron", &C_SERVER_ITEMS_FULLAPRON);
    add_generated_ctors(w, "Server.Items.FurBoots", &C_SERVER_ITEMS_FURBOOTS);
    add_generated_ctors(w, "Server.Items.FurCape", &C_SERVER_ITEMS_FURCAPE);
    add_generated_ctors(w, "Server.Items.FurSarong", &C_SERVER_ITEMS_FURSARONG);
    add_generated_ctors(w, "Server.Items.GargoylesPickaxe", &C_SERVER_ITEMS_GARGOYLESPICKAXE);
    add_generated_ctors(w, "Server.Items.GasTrap", &C_SERVER_ITEMS_GASTRAP);
    add_generated_ctors(w, "Server.Items.GateTravelScroll", &C_SERVER_ITEMS_GATETRAVELSCROLL);
    add_generated_ctors(w, "Server.Items.Gears", &C_SERVER_ITEMS_GEARS);
    add_generated_ctors(w, "Server.Items.GiftBox", &C_SERVER_ITEMS_GIFTBOX);
    add_generated_ctors(w, "Server.Items.GiftOfLifeScroll", &C_SERVER_ITEMS_GIFTOFLIFESCROLL);
    add_generated_ctors(w, "Server.Items.GiftOfRenewalScroll", &C_SERVER_ITEMS_GIFTOFRENEWALSCROLL);
    add_generated_ctors(w, "Server.Items.GildedDress", &C_SERVER_ITEMS_GILDEDDRESS);
    add_generated_ctors(w, "Server.Items.GlassMug", &C_SERVER_ITEMS_GLASSMUG);
    add_generated_ctors(w, "Server.Items.Goblet", &C_SERVER_ITEMS_GOBLET);
    add_generated_ctors(w, "Server.Items.GoldWire", &C_SERVER_ITEMS_GOLDWIRE);
    add_generated_ctors(w, "Server.Items.GozaMatEastAddon", &C_SERVER_ITEMS_GOZAMATEASTADDON);
    add_generated_ctors(w, "Server.Items.GozaMatSouthAddon", &C_SERVER_ITEMS_GOZAMATSOUTHADDON);
    add_generated_ctors(w, "Server.Items.GreaterHealScroll", &C_SERVER_ITEMS_GREATERHEALSCROLL);
    add_generated_ctors(w, "Server.Items.GreenDriedFlowers", &C_SERVER_ITEMS_GREENDRIEDFLOWERS);
    add_generated_ctors(w, "Server.Items.GrizzledBones", &C_SERVER_ITEMS_GRIZZLEDBONES);
    add_generated_ctors(w, "Server.Items.HailStormScroll", &C_SERVER_ITEMS_HAILSTORMSCROLL);
    add_generated_ctors(w, "Server.Items.Hakama", &C_SERVER_ITEMS_HAKAMA);
    add_generated_ctors(w, "Server.Items.HakamaShita", &C_SERVER_ITEMS_HAKAMASHITA);
    add_generated_ctors(w, "Server.Items.HalfApron", &C_SERVER_ITEMS_HALFAPRON);
    add_generated_ctors(w, "Server.Items.Hammer", &C_SERVER_ITEMS_HAMMER);
    add_generated_ctors(w, "Server.Items.HangingAxesAddon", &C_SERVER_ITEMS_HANGINGAXESADDON);
    add_generated_ctors(w, "Server.Items.HangingSkeleton", &C_SERVER_ITEMS_HANGINGSKELETON);
    add_generated_ctors(w, "Server.Items.HangingSwordsAddon", &C_SERVER_ITEMS_HANGINGSWORDSADDON);
    add_generated_ctors(w, "Server.Items.HarmScroll", &C_SERVER_ITEMS_HARMSCROLL);
    add_generated_ctors(w, "Server.Items.HealScroll", &C_SERVER_ITEMS_HEALSCROLL);
    add_generated_ctors(w, "Server.Items.HealingStoneScroll", &C_SERVER_ITEMS_HEALINGSTONESCROLL);
    add_generated_ctors(w, "Server.Items.HearthOfHomeFire", &C_SERVER_ITEMS_HEARTHOFHOMEFIRE);
    add_generated_ctors(w, "Server.Items.Hinge", &C_SERVER_ITEMS_HINGE);
    add_generated_ctors(w, "Server.Items.HolidayBell", &C_SERVER_ITEMS_HOLIDAYBELL);
    add_generated_ctors(w, "Server.Items.HolidayPottedPlant", &C_SERVER_ITEMS_HOLIDAYPOTTEDPLANT);
    add_generated_ctors(w, "Server.Items.HornedTribalMask", &C_SERVER_ITEMS_HORNEDTRIBALMASK);
    add_generated_ctors(w, "Server.Items.HorrificBeastScroll", &C_SERVER_ITEMS_HORRIFICBEASTSCROLL);
    add_generated_ctors(w, "Server.Items.HouseLadderAddon", &C_SERVER_ITEMS_HOUSELADDERADDON);
    add_generated_ctors(w, "Server.Items.ImmolatingWeaponScroll", &C_SERVER_ITEMS_IMMOLATINGWEAPONSCROLL);
    add_generated_ctors(w, "Server.Items.IncognitoScroll", &C_SERVER_ITEMS_INCOGNITOSCROLL);
    add_generated_ctors(w, "Server.Items.Inshave", &C_SERVER_ITEMS_INSHAVE);
    add_generated_ctors(w, "Server.Items.InteractionTeleporter", &C_SERVER_ITEMS_INTERACTIONTELEPORTER);
    add_generated_ctors(w, "Server.Items.InvisibilityScroll", &C_SERVER_ITEMS_INVISIBILITYSCROLL);
    add_generated_ctors(w, "Server.Items.IronWire", &C_SERVER_ITEMS_IRONWIRE);
    add_generated_ctors(w, "Server.Items.JackOLantern", &C_SERVER_ITEMS_JACKOLANTERN);
    add_generated_ctors(w, "Server.Items.JellyBeans", &C_SERVER_ITEMS_JELLYBEANS);
    add_generated_ctors(w, "Server.Items.JesterHat", &C_SERVER_ITEMS_JESTERHAT);
    add_generated_ctors(w, "Server.Items.JesterHatofChuckles", &C_SERVER_ITEMS_JESTERHATOFCHUCKLES);
    add_generated_ctors(w, "Server.Items.JesterSuit", &C_SERVER_ITEMS_JESTERSUIT);
    add_generated_ctors(w, "Server.Items.JinBaori", &C_SERVER_ITEMS_JINBAORI);
    add_generated_ctors(w, "Server.Items.JointingPlane", &C_SERVER_ITEMS_JOINTINGPLANE);
    add_generated_ctors(w, "Server.Items.Kamishimo", &C_SERVER_ITEMS_KAMISHIMO);
    add_generated_ctors(w, "Server.Items.Kasa", &C_SERVER_ITEMS_KASA);
    add_generated_ctors(w, "Server.Items.Kilt", &C_SERVER_ITEMS_KILT);
    add_generated_ctors(w, "Server.Items.Kindling", &C_SERVER_ITEMS_KINDLING);
    add_generated_ctors(w, "Server.Items.LardOfParoxysmus", &C_SERVER_ITEMS_LARDOFPAROXYSMUS);
    add_generated_ctors(w, "Server.Items.LargeStoneTableEastAddon", &C_SERVER_ITEMS_LARGESTONETABLEEASTADDON);
    add_generated_ctors(w, "Server.Items.LargeStoneTableSouthAddon", &C_SERVER_ITEMS_LARGESTONETABLESOUTHADDON);
    add_generated_ctors(w, "Server.Items.LeatherGlovesOfMining", &C_SERVER_ITEMS_LEATHERGLOVESOFMINING);
    add_generated_ctors(w, "Server.Items.LichFormScroll", &C_SERVER_ITEMS_LICHFORMSCROLL);
    add_generated_ctors(w, "Server.Items.LightOfTheWinterSolstice", &C_SERVER_ITEMS_LIGHTOFTHEWINTERSOLSTICE);
    add_generated_ctors(w, "Server.Items.LightYarn", &C_SERVER_ITEMS_LIGHTYARN);
    add_generated_ctors(w, "Server.Items.LightYarnUnraveled", &C_SERVER_ITEMS_LIGHTYARNUNRAVELED);
    add_generated_ctors(w, "Server.Items.LightningScroll", &C_SERVER_ITEMS_LIGHTNINGSCROLL);
    add_generated_ctors(w, "Server.Items.Lockpick", &C_SERVER_ITEMS_LOCKPICK);
    add_generated_ctors(w, "Server.Items.Lollipops", &C_SERVER_ITEMS_LOLLIPOPS);
    add_generated_ctors(w, "Server.Items.LongPants", &C_SERVER_ITEMS_LONGPANTS);
    add_generated_ctors(w, "Server.Items.LuminescentFungi", &C_SERVER_ITEMS_LUMINESCENTFUNGI);
    add_generated_ctors(w, "Server.Items.MagicArrowScroll", &C_SERVER_ITEMS_MAGICARROWSCROLL);
    add_generated_ctors(w, "Server.Items.MagicLockScroll", &C_SERVER_ITEMS_MAGICLOCKSCROLL);
    add_generated_ctors(w, "Server.Items.MagicReflectScroll", &C_SERVER_ITEMS_MAGICREFLECTSCROLL);
    add_generated_ctors(w, "Server.Items.MagicTrapScroll", &C_SERVER_ITEMS_MAGICTRAPSCROLL);
    add_generated_ctors(w, "Server.Items.MagicUnTrapScroll", &C_SERVER_ITEMS_MAGICUNTRAPSCROLL);
    add_generated_ctors(w, "Server.Items.MagicWizardsHat", &C_SERVER_ITEMS_MAGICWIZARDSHAT);
    add_generated_ctors(w, "Server.Items.MaleElvenRobe", &C_SERVER_ITEMS_MALEELVENROBE);
    add_generated_ctors(w, "Server.Items.MaleKimono", &C_SERVER_ITEMS_MALEKIMONO);
    add_generated_ctors(w, "Server.Items.MalletAndChisel", &C_SERVER_ITEMS_MALLETANDCHISEL);
    add_generated_ctors(w, "Server.Items.ManaDrainScroll", &C_SERVER_ITEMS_MANADRAINSCROLL);
    add_generated_ctors(w, "Server.Items.ManaVampireScroll", &C_SERVER_ITEMS_MANAVAMPIRESCROLL);
    add_generated_ctors(w, "Server.Items.MapItem", &C_SERVER_ITEMS_MAPITEM);
    add_generated_ctors(w, "Server.Items.MapmakersPen", &C_SERVER_ITEMS_MAPMAKERSPEN);
    add_generated_ctors(w, "Server.Items.MarkContainer", &C_SERVER_ITEMS_MARKCONTAINER);
    add_generated_ctors(w, "Server.Items.MarkScroll", &C_SERVER_ITEMS_MARKSCROLL);
    add_generated_ctors(w, "Server.Items.MassCurseScroll", &C_SERVER_ITEMS_MASSCURSESCROLL);
    add_generated_ctors(w, "Server.Items.MassDispelScroll", &C_SERVER_ITEMS_MASSDISPELSCROLL);
    add_generated_ctors(w, "Server.Items.MassSleepScroll", &C_SERVER_ITEMS_MASSSLEEPSCROLL);
    add_generated_ctors(w, "Server.Items.MediumStoneTableEastAddon", &C_SERVER_ITEMS_MEDIUMSTONETABLEEASTADDON);
    add_generated_ctors(w, "Server.Items.MediumStoneTableSouthAddon", &C_SERVER_ITEMS_MEDIUMSTONETABLESOUTHADDON);
    add_generated_ctors(w, "Server.Items.MeteorSwarmScroll", &C_SERVER_ITEMS_METEORSWARMSCROLL);
    add_generated_ctors(w, "Server.Items.MindBlastScroll", &C_SERVER_ITEMS_MINDBLASTSCROLL);
    add_generated_ctors(w, "Server.Items.MindRotScroll", &C_SERVER_ITEMS_MINDROTSCROLL);
    add_generated_ctors(w, "Server.Items.MistletoeAddon", &C_SERVER_ITEMS_MISTLETOEADDON);
    add_generated_ctors(w, "Server.Items.MistletoeDeed", &C_SERVER_ITEMS_MISTLETOEDEED);
    add_generated_ctors(w, "Server.Items.MonkRobe", &C_SERVER_ITEMS_MONKROBE);
    add_generated_ctors(w, "Server.Items.Moongate", &C_SERVER_ITEMS_MOONGATE);
    add_generated_ctors(w, "Server.Items.MorphItem", &C_SERVER_ITEMS_MORPHITEM);
    add_generated_ctors(w, "Server.Items.MortarPestle", &C_SERVER_ITEMS_MORTARPESTLE);
    add_generated_ctors(w, "Server.Items.MouldingPlane", &C_SERVER_ITEMS_MOULDINGPLANE);
    add_generated_ctors(w, "Server.Items.Muculent", &C_SERVER_ITEMS_MUCULENT);
    add_generated_ctors(w, "Server.Items.Nails", &C_SERVER_ITEMS_NAILS);
    add_generated_ctors(w, "Server.Items.NatureFuryScroll", &C_SERVER_ITEMS_NATUREFURYSCROLL);
    add_generated_ctors(w, "Server.Items.NetherBoltScroll", &C_SERVER_ITEMS_NETHERBOLTSCROLL);
    add_generated_ctors(w, "Server.Items.NetherCycloneScroll", &C_SERVER_ITEMS_NETHERCYCLONESCROLL);
    add_generated_ctors(w, "Server.Items.NightSightScroll", &C_SERVER_ITEMS_NIGHTSIGHTSCROLL);
    add_generated_ctors(w, "Server.Items.NinjaTabi", &C_SERVER_ITEMS_NINJATABI);
    add_generated_ctors(w, "Server.Items.NougatSwirl", &C_SERVER_ITEMS_NOUGATSWIRL);
    add_generated_ctors(w, "Server.Items.Obi", &C_SERVER_ITEMS_OBI);
    add_generated_ctors(w, "Server.Items.OilFlask", &C_SERVER_ITEMS_OILFLASK);
    add_generated_ctors(w, "Server.Items.OrcishKinMask", &C_SERVER_ITEMS_ORCISHKINMASK);
    add_generated_ctors(w, "Server.Items.PainSpikeScroll", &C_SERVER_ITEMS_PAINSPIKESCROLL);
    add_generated_ctors(w, "Server.Items.ParagonChest", &C_SERVER_ITEMS_PARAGONCHEST);
    add_generated_ctors(w, "Server.Items.ParalyzeFieldScroll", &C_SERVER_ITEMS_PARALYZEFIELDSCROLL);
    add_generated_ctors(w, "Server.Items.ParalyzeScroll", &C_SERVER_ITEMS_PARALYZESCROLL);
    add_generated_ctors(w, "Server.Items.ParasiticPlant", &C_SERVER_ITEMS_PARASITICPLANT);
    add_generated_ctors(w, "Server.Items.PerfectEmerald", &C_SERVER_ITEMS_PERFECTEMERALD);
    add_generated_ctors(w, "Server.Items.PewterMug", &C_SERVER_ITEMS_PEWTERMUG);
    add_generated_ctors(w, "Server.Items.Pitcher", &C_SERVER_ITEMS_PITCHER);
    add_generated_ctors(w, "Server.Items.PlainDress", &C_SERVER_ITEMS_PLAINDRESS);
    add_generated_ctors(w, "Server.Items.PoisonFieldScroll", &C_SERVER_ITEMS_POISONFIELDSCROLL);
    add_generated_ctors(w, "Server.Items.PoisonScroll", &C_SERVER_ITEMS_POISONSCROLL);
    add_generated_ctors(w, "Server.Items.PoisonStrikeScroll", &C_SERVER_ITEMS_POISONSTRIKESCROLL);
    add_generated_ctors(w, "Server.Items.PolymorphScroll", &C_SERVER_ITEMS_POLYMORPHSCROLL);
    add_generated_ctors(w, "Server.Items.PowderOfTemperament", &C_SERVER_ITEMS_POWDEROFTEMPERAMENT);
    add_generated_ctors(w, "Server.Items.PowderOfTranslocation", &C_SERVER_ITEMS_POWDEROFTRANSLOCATION);
    add_generated_ctors(w, "Server.Items.PowerGenerator", &C_SERVER_ITEMS_POWERGENERATOR);
    add_generated_ctors(w, "Server.Items.ProjectedItem", &C_SERVER_ITEMS_PROJECTEDITEM);
    add_generated_ctors(w, "Server.Items.ProtectionScroll", &C_SERVER_ITEMS_PROTECTIONSCROLL);
    add_generated_ctors(w, "Server.Items.PurgeMagicScroll", &C_SERVER_ITEMS_PURGEMAGICSCROLL);
    add_generated_ctors(w, "Server.Items.Putrefication", &C_SERVER_ITEMS_PUTREFICATION);
    add_generated_ctors(w, "Server.Items.RawBird", &C_SERVER_ITEMS_RAWBIRD);
    add_generated_ctors(w, "Server.Items.RawFishSteak", &C_SERVER_ITEMS_RAWFISHSTEAK);
    add_generated_ctors(w, "Server.Items.RawLambLeg", &C_SERVER_ITEMS_RAWLAMBLEG);
    add_generated_ctors(w, "Server.Items.RawRibs", &C_SERVER_ITEMS_RAWRIBS);
    add_generated_ctors(w, "Server.Items.ReactiveArmorScroll", &C_SERVER_ITEMS_REACTIVEARMORSCROLL);
    add_generated_ctors(w, "Server.Items.ReaperFormScroll", &C_SERVER_ITEMS_REAPERFORMSCROLL);
    add_generated_ctors(w, "Server.Items.RecallScroll", &C_SERVER_ITEMS_RECALLSCROLL);
    add_generated_ctors(w, "Server.Items.RecipeScroll", &C_SERVER_ITEMS_RECIPESCROLL);
    add_generated_ctors(w, "Server.Items.RedVelvetGiftBox", &C_SERVER_ITEMS_REDVELVETGIFTBOX);
    add_generated_ctors(w, "Server.Items.ResurrectionScroll", &C_SERVER_ITEMS_RESURRECTIONSCROLL);
    add_generated_ctors(w, "Server.Items.RevealScroll", &C_SERVER_ITEMS_REVEALSCROLL);
    add_generated_ctors(w, "Server.Items.RewardBrazier", &C_SERVER_ITEMS_REWARDBRAZIER);
    add_generated_ctors(w, "Server.Items.RewardPottedCactus", &C_SERVER_ITEMS_REWARDPOTTEDCACTUS);
    add_generated_ctors(w, "Server.Items.RingmailGlovesOfMining", &C_SERVER_ITEMS_RINGMAILGLOVESOFMINING);
    add_generated_ctors(w, "Server.Items.RisingColossusScroll", &C_SERVER_ITEMS_RISINGCOLOSSUSSCROLL);
    add_generated_ctors(w, "Server.Items.Robe", &C_SERVER_ITEMS_ROBE);
    add_generated_ctors(w, "Server.Items.RollingPin", &C_SERVER_ITEMS_ROLLINGPIN);
    add_generated_ctors(w, "Server.Items.Rope", &C_SERVER_ITEMS_ROPE);
    add_generated_ctors(w, "Server.Items.RoseOfTrinsicPetal", &C_SERVER_ITEMS_ROSEOFTRINSICPETAL);
    add_generated_ctors(w, "Server.Items.Ruby", &C_SERVER_ITEMS_RUBY);
    add_generated_ctors(w, "Server.Items.SHTeleporter", &C_SERVER_ITEMS_SHTELEPORTER);
    add_generated_ctors(w, "Server.Items.SOS", &C_SERVER_ITEMS_SOS);
    add_generated_ctors(w, "Server.Items.SackOfSugar", &C_SERVER_ITEMS_SACKOFSUGAR);
    add_generated_ctors(w, "Server.Items.SamuraiTabi", &C_SERVER_ITEMS_SAMURAITABI);
    add_generated_ctors(w, "Server.Items.Sand", &C_SERVER_ITEMS_SAND);
    add_generated_ctors(w, "Server.Items.Sandals", &C_SERVER_ITEMS_SANDALS);
    add_generated_ctors(w, "Server.Items.Sapphire", &C_SERVER_ITEMS_SAPPHIRE);
    add_generated_ctors(w, "Server.Items.SavageMask", &C_SERVER_ITEMS_SAVAGEMASK);
    add_generated_ctors(w, "Server.Items.Saw", &C_SERVER_ITEMS_SAW);
    add_generated_ctors(w, "Server.Items.Scorp", &C_SERVER_ITEMS_SCORP);
    add_generated_ctors(w, "Server.Items.Scourge", &C_SERVER_ITEMS_SCOURGE);
    add_generated_ctors(w, "Server.Items.ScribesPen", &C_SERVER_ITEMS_SCRIBESPEN);
    add_generated_ctors(w, "Server.Items.SewingKit", &C_SERVER_ITEMS_SEWINGKIT);
    add_generated_ctors(w, "Server.Items.SextantParts", &C_SERVER_ITEMS_SEXTANTPARTS);
    add_generated_ctors(w, "Server.Items.Shaft", &C_SERVER_ITEMS_SHAFT);
    add_generated_ctors(w, "Server.Items.Shirt", &C_SERVER_ITEMS_SHIRT);
    add_generated_ctors(w, "Server.Items.Shoes", &C_SERVER_ITEMS_SHOES);
    add_generated_ctors(w, "Server.Items.ShortPants", &C_SERVER_ITEMS_SHORTPANTS);
    add_generated_ctors(w, "Server.Items.Shovel", &C_SERVER_ITEMS_SHOVEL);
    add_generated_ctors(w, "Server.Items.ShrineOfWisdomComponent", &C_SERVER_ITEMS_SHRINEOFWISDOMCOMPONENT);
    add_generated_ctors(w, "Server.Items.Shuriken", &C_SERVER_ITEMS_SHURIKEN);
    add_generated_ctors(w, "Server.Items.SilverWire", &C_SERVER_ITEMS_SILVERWIRE);
    add_generated_ctors(w, "Server.Items.Skillet", &C_SERVER_ITEMS_SKILLET);
    add_generated_ctors(w, "Server.Items.Skirt", &C_SERVER_ITEMS_SKIRT);
    add_generated_ctors(w, "Server.Items.SkullCap", &C_SERVER_ITEMS_SKULLCAP);
    add_generated_ctors(w, "Server.Items.SledgeHammer", &C_SERVER_ITEMS_SLEDGEHAMMER);
    add_generated_ctors(w, "Server.Items.SleepScroll", &C_SERVER_ITEMS_SLEEPSCROLL);
    add_generated_ctors(w, "Server.Items.SmithHammer", &C_SERVER_ITEMS_SMITHHAMMER);
    add_generated_ctors(w, "Server.Items.SmoothingPlane", &C_SERVER_ITEMS_SMOOTHINGPLANE);
    add_generated_ctors(w, "Server.Items.SnowGlobeOne", &C_SERVER_ITEMS_SNOWGLOBEONE);
    add_generated_ctors(w, "Server.Items.SnowGlobeThree", &C_SERVER_ITEMS_SNOWGLOBETHREE);
    add_generated_ctors(w, "Server.Items.SnowGlobeTwo", &C_SERVER_ITEMS_SNOWGLOBETWO);
    add_generated_ctors(w, "Server.Items.SnowPileDeco", &C_SERVER_ITEMS_SNOWPILEDECO);
    add_generated_ctors(w, "Server.Items.Snowman", &C_SERVER_ITEMS_SNOWMAN);
    add_generated_ctors(w, "Server.Items.SoulstoneFragment", &C_SERVER_ITEMS_SOULSTONEFRAGMENT);
    add_generated_ctors(w, "Server.Items.SpellPlagueScroll", &C_SERVER_ITEMS_SPELLPLAGUESCROLL);
    add_generated_ctors(w, "Server.Items.SpellTriggerScroll", &C_SERVER_ITEMS_SPELLTRIGGERSCROLL);
    add_generated_ctors(w, "Server.Items.Spellbook", &C_SERVER_ITEMS_SPELLBOOK);
    add_generated_ctors(w, "Server.Items.SpoolOfThread", &C_SERVER_ITEMS_SPOOLOFTHREAD);
    add_generated_ctors(w, "Server.Items.Springs", &C_SERVER_ITEMS_SPRINGS);
    add_generated_ctors(w, "Server.Items.SquareGozaMatEastAddon", &C_SERVER_ITEMS_SQUAREGOZAMATEASTADDON);
    add_generated_ctors(w, "Server.Items.SquareGozaMatSouthAddon", &C_SERVER_ITEMS_SQUAREGOZAMATSOUTHADDON);
    add_generated_ctors(w, "Server.Items.StValentinesPanda", &C_SERVER_ITEMS_STVALENTINESPANDA);
    add_generated_ctors(w, "Server.Items.StValentinesPolarBear", &C_SERVER_ITEMS_STVALENTINESPOLARBEAR);
    add_generated_ctors(w, "Server.Items.StarSapphire", &C_SERVER_ITEMS_STARSAPPHIRE);
    add_generated_ctors(w, "Server.Items.StatCapScroll", &C_SERVER_ITEMS_STATCAPSCROLL);
    add_generated_ctors(w, "Server.Items.StoneAnkh", &C_SERVER_ITEMS_STONEANKH);
    add_generated_ctors(w, "Server.Items.StoneFormScroll", &C_SERVER_ITEMS_STONEFORMSCROLL);
    add_generated_ctors(w, "Server.Items.StoneStatueAddon", &C_SERVER_ITEMS_STONESTATUEADDON);
    add_generated_ctors(w, "Server.Items.StrangleScroll", &C_SERVER_ITEMS_STRANGLESCROLL);
    add_generated_ctors(w, "Server.Items.StrawHat", &C_SERVER_ITEMS_STRAWHAT);
    add_generated_ctors(w, "Server.Items.StrengthScroll", &C_SERVER_ITEMS_STRENGTHSCROLL);
    add_generated_ctors(w, "Server.Items.StuddedGlovesOfMining", &C_SERVER_ITEMS_STUDDEDGLOVESOFMINING);
    add_generated_ctors(w, "Server.Items.SturdyShovel", &C_SERVER_ITEMS_STURDYSHOVEL);
    add_generated_ctors(w, "Server.Items.SubtextSign", &C_SERVER_ITEMS_SUBTEXTSIGN);
    add_generated_ctors(w, "Server.Items.SummonAirElementalScroll", &C_SERVER_ITEMS_SUMMONAIRELEMENTALSCROLL);
    add_generated_ctors(w, "Server.Items.SummonCreatureScroll", &C_SERVER_ITEMS_SUMMONCREATURESCROLL);
    add_generated_ctors(w, "Server.Items.SummonDaemonScroll", &C_SERVER_ITEMS_SUMMONDAEMONSCROLL);
    add_generated_ctors(w, "Server.Items.SummonEarthElementalScroll", &C_SERVER_ITEMS_SUMMONEARTHELEMENTALSCROLL);
    add_generated_ctors(w, "Server.Items.SummonFamiliarScroll", &C_SERVER_ITEMS_SUMMONFAMILIARSCROLL);
    add_generated_ctors(w, "Server.Items.SummonFeyScroll", &C_SERVER_ITEMS_SUMMONFEYSCROLL);
    add_generated_ctors(w, "Server.Items.SummonFiendScroll", &C_SERVER_ITEMS_SUMMONFIENDSCROLL);
    add_generated_ctors(w, "Server.Items.SummonFireElementalScroll", &C_SERVER_ITEMS_SUMMONFIREELEMENTALSCROLL);
    add_generated_ctors(w, "Server.Items.SummonWaterElementalScroll", &C_SERVER_ITEMS_SUMMONWATERELEMENTALSCROLL);
    add_generated_ctors(w, "Server.Items.Surcoat", &C_SERVER_ITEMS_SURCOAT);
    add_generated_ctors(w, "Server.Items.SwitchItem", &C_SERVER_ITEMS_SWITCHITEM);
    add_generated_ctors(w, "Server.Items.Taffy", &C_SERVER_ITEMS_TAFFY);
    add_generated_ctors(w, "Server.Items.Taint", &C_SERVER_ITEMS_TAINT);
    add_generated_ctors(w, "Server.Items.TaintedWool", &C_SERVER_ITEMS_TAINTEDWOOL);
    add_generated_ctors(w, "Server.Items.TallStrawHat", &C_SERVER_ITEMS_TALLSTRAWHAT);
    add_generated_ctors(w, "Server.Items.TattsukeHakama", &C_SERVER_ITEMS_TATTSUKEHAKAMA);
    add_generated_ctors(w, "Server.Items.TelekinesisScroll", &C_SERVER_ITEMS_TELEKINESISSCROLL);
    add_generated_ctors(w, "Server.Items.TeleportScroll", &C_SERVER_ITEMS_TELEPORTSCROLL);
    add_generated_ctors(w, "Server.Items.Teleporter", &C_SERVER_ITEMS_TELEPORTER);
    add_generated_ctors(w, "Server.Items.ThighBoots", &C_SERVER_ITEMS_THIGHBOOTS);
    add_generated_ctors(w, "Server.Items.ThunderstormScroll", &C_SERVER_ITEMS_THUNDERSTORMSCROLL);
    add_generated_ctors(w, "Server.Items.TimeoutTeleporter", &C_SERVER_ITEMS_TIMEOUTTELEPORTER);
    add_generated_ctors(w, "Server.Items.TinkerTools", &C_SERVER_ITEMS_TINKERTOOLS);
    add_generated_ctors(w, "Server.Items.TinkersTools", &C_SERVER_ITEMS_TINKERSTOOLS);
    add_generated_ctors(w, "Server.Items.ToggleItem", &C_SERVER_ITEMS_TOGGLEITEM);
    add_generated_ctors(w, "Server.Items.Tongs", &C_SERVER_ITEMS_TONGS);
    add_generated_ctors(w, "Server.Items.Tourmaline", &C_SERVER_ITEMS_TOURMALINE);
    add_generated_ctors(w, "Server.Items.TrainingDummy", &C_SERVER_ITEMS_TRAININGDUMMY);
    add_generated_ctors(w, "Server.Items.TreeStump", &C_SERVER_ITEMS_TREESTUMP);
    add_generated_ctors(w, "Server.Items.TribalBerry", &C_SERVER_ITEMS_TRIBALBERRY);
    add_generated_ctors(w, "Server.Items.TribalMask", &C_SERVER_ITEMS_TRIBALMASK);
    add_generated_ctors(w, "Server.Items.TricorneHat", &C_SERVER_ITEMS_TRICORNEHAT);
    add_generated_ctors(w, "Server.Items.Tub", &C_SERVER_ITEMS_TUB);
    add_generated_ctors(w, "Server.Items.Tunic", &C_SERVER_ITEMS_TUNIC);
    add_generated_ctors(w, "Server.Items.Turquoise", &C_SERVER_ITEMS_TURQUOISE);
    add_generated_ctors(w, "Server.Items.UnlockScroll", &C_SERVER_ITEMS_UNLOCKSCROLL);
    add_generated_ctors(w, "Server.Items.UnmadeBedAddon", &C_SERVER_ITEMS_UNMADEBEDADDON);
    add_generated_ctors(w, "Server.Items.ValentinesCard", &C_SERVER_ITEMS_VALENTINESCARD);
    add_generated_ctors(w, "Server.Items.VampiricEmbraceScroll", &C_SERVER_ITEMS_VAMPIRICEMBRACESCROLL);
    add_generated_ctors(w, "Server.Items.Vanilla", &C_SERVER_ITEMS_VANILLA);
    add_generated_ctors(w, "Server.Items.VanityAddon", &C_SERVER_ITEMS_VANITYADDON);
    add_generated_ctors(w, "Server.Items.VengefulSpiritScroll", &C_SERVER_ITEMS_VENGEFULSPIRITSCROLL);
    add_generated_ctors(w, "Server.Items.Vines", &C_SERVER_ITEMS_VINES);
    add_generated_ctors(w, "Server.Items.WallBanner", &C_SERVER_ITEMS_WALLBANNER);
    add_generated_ctors(w, "Server.Items.WallOfStoneScroll", &C_SERVER_ITEMS_WALLOFSTONESCROLL);
    add_generated_ctors(w, "Server.Items.Waraji", &C_SERVER_ITEMS_WARAJI);
    add_generated_ctors(w, "Server.Items.WaterBarrel", &C_SERVER_ITEMS_WATERBARREL);
    add_generated_ctors(w, "Server.Items.WeakenScroll", &C_SERVER_ITEMS_WEAKENSCROLL);
    add_generated_ctors(w, "Server.Items.WeaponEngravingTool", &C_SERVER_ITEMS_WEAPONENGRAVINGTOOL);
    add_generated_ctors(w, "Server.Items.Web", &C_SERVER_ITEMS_WEB);
    add_generated_ctors(w, "Server.Items.WheatSheaf", &C_SERVER_ITEMS_WHEATSHEAF);
    add_generated_ctors(w, "Server.Items.WhiteDriedFlowers", &C_SERVER_ITEMS_WHITEDRIEDFLOWERS);
    add_generated_ctors(w, "Server.Items.WideBrimHat", &C_SERVER_ITEMS_WIDEBRIMHAT);
    add_generated_ctors(w, "Server.Items.WildfireScroll", &C_SERVER_ITEMS_WILDFIRESCROLL);
    add_generated_ctors(w, "Server.Items.WitherScroll", &C_SERVER_ITEMS_WITHERSCROLL);
    add_generated_ctors(w, "Server.Items.WizardsHat", &C_SERVER_ITEMS_WIZARDSHAT);
    add_generated_ctors(w, "Server.Items.WoodenCoffinAddon", &C_SERVER_ITEMS_WOODENCOFFINADDON);
    add_generated_ctors(w, "Server.Items.WoodlandBelt", &C_SERVER_ITEMS_WOODLANDBELT);
    add_generated_ctors(w, "Server.Items.Wool", &C_SERVER_ITEMS_WOOL);
    add_generated_ctors(w, "Server.Items.WordOfDeathScroll", &C_SERVER_ITEMS_WORDOFDEATHSCROLL);
    add_generated_ctors(w, "Server.Items.WraithFormScroll", &C_SERVER_ITEMS_WRAITHFORMSCROLL);
    add_generated_ctors(w, "Server.Items.WrappedCandy", &C_SERVER_ITEMS_WRAPPEDCANDY);
    add_generated_ctors(w, "Server.Items.WreathAddon", &C_SERVER_ITEMS_WREATHADDON);
    add_generated_ctors(w, "Server.Items.WreathDeed", &C_SERVER_ITEMS_WREATHDEED);
    add_generated_ctors(w, "Server.Items.ZoogiFungus", &C_SERVER_ITEMS_ZOOGIFUNGUS);
}

/// A hand-written table for the same class wins, whichever runs first.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(Phase::Configure, "muo_items.CtorTables.Configure", configure);
}
