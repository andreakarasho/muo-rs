//! Generated from the `[Constructible]` constructors: the `[add <type> <args>` overload tables of this crate, in
//! declaration order. Do not edit by hand.
//! A constructor with no Rust counterpart is listed in target/ctors-missing.txt.

use muo_content::props::{Ctor, Kind, Param, Value, add_generated_ctors, ctor_amount, ctor_default, ctor_hue, item_ty};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::type_registry::{Phase, TypeRegistry};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const BOOL: Kind = Kind::Plain(PropType::Bool);
const STR: Kind = Kind::Plain(PropType::Str);

static C_SERVER_ENGINES_MLQUESTS_ITEMS_MLQUESTTELEPORTER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Engines.MLQuests.Items.MLQuestTeleporter"),
    },
];

static C_SERVER_ENGINES_MLQUESTS_ITEMS_TICKETTELEPORTER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Engines.MLQuests.Items.TicketTeleporter"),
    },
];

static C_SERVER_ENGINES_PLANTS_SEED: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Engines.Plants.Seed"),
    },
];

static C_SERVER_ENGINES_SPAWNERS_PROXIMITYSPAWNER: [Ctor; 1] = [
    Ctor {
        access: Some(AccessLevel::Developer),
        params: &[],
        make: |w, _| ctor_default(w, "Server.Engines.Spawners.ProximitySpawner"),
    },
];

static C_SERVER_ENGINES_SPAWNERS_REGIONSPAWNER: [Ctor; 1] = [
    Ctor {
        access: Some(AccessLevel::Developer),
        params: &[],
        make: |w, _| ctor_default(w, "Server.Engines.Spawners.RegionSpawner"),
    },
];

static C_SERVER_ENGINES_SPAWNERS_SPAWNER: [Ctor; 2] = [
    Ctor {
        access: Some(AccessLevel::Developer),
        params: &[],
        make: |w, _| ctor_default(w, "Server.Engines.Spawners.Spawner"),
    },
    Ctor {
        access: Some(AccessLevel::Developer),
        params: &[Param { name: "spawnedName", kind: STR, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::doom::gauntlet::new_spawner(w, Some(v[0].string().as_deref().unwrap_or_default()))),
    },
];

static C_SERVER_FACTIONS_SILVER: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Factions.Silver"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: None, ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Factions.Silver", v[0].i32()),
    },
];

static C_SERVER_ITEMS_ANCIENTURN: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.AncientUrn"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "urnName", kind: STR, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::tokuno::artifacts::ancient_urn_new(w, &v[0].string().unwrap_or_default())),
    },
];

static C_SERVER_ITEMS_BLANKSCROLL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.BlankScroll", v[0].i32()),
    },
];

static C_SERVER_ITEMS_CLOTH: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.Cloth", v[0].i32()),
    },
];

static C_SERVER_ITEMS_GAMANHORNS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.GamanHorns");
            EntityId::Item(crate::ml_quests::items_gen::gaman_horns_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_GREENTHORNS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.GreenThorns", v[0].i32()),
    },
];

static C_SERVER_ITEMS_HONORABLESWORDS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.HonorableSwords"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "swordsName", kind: STR, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::tokuno::artifacts::honorable_swords_new(w, &v[0].string().unwrap_or_default())),
    },
];

static C_SERVER_ITEMS_HOODEDSHROUDOFSHADOWS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: Some(Value::Int(0x455)), ty: "" }],
        make: |w, v| ctor_hue(w, "Server.Items.HoodedShroudOfShadows", v[0].i32()),
    },
];

static C_SERVER_ITEMS_KHALDUNPITTELEPORTER: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.KhaldunPitTeleporter"),
    },
];

static C_SERVER_ITEMS_LESSERPIGMENTSOFTOKUNO: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.LesserPigmentsOfTokuno"),
    },
];

static C_SERVER_ITEMS_ORANGEPETALS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.OrangePetals", v[0].i32()),
    },
];

static C_SERVER_ITEMS_PIXIELEG: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.PixieLeg", v[0].i32()),
    },
];

static C_SERVER_ITEMS_REDLEAVES: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.RedLeaves", v[0].i32()),
    },
];

static C_SERVER_ITEMS_RUNEBOOK: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.Runebook"),
    },
];

static C_SERVER_ITEMS_SALVAGEBAG: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.SalvageBag"),
    },
    Ctor {
        access: None,
        params: &[Param { name: "hue", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::craft::salvage_bag::salvage_bag_new_hue(w, v[0].i32())),
    },
];

static C_SERVER_ITEMS_SAPOFSOSARIA: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.SapOfSosaria");
            EntityId::Item(crate::ml_quests::items_gen::sap_of_sosaria_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_SEVEREDELFEARS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.SeveredElfEars");
            EntityId::Item(crate::ml_quests::items_gen::severed_elf_ears_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_SEVEREDHUMANEARS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.SeveredHumanEars");
            EntityId::Item(crate::ml_quests::items_gen::severed_human_ears_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_STARROOMGATE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "decays", kind: BOOL, default: Some(Value::Bool(false)), ty: "" }],
        make: |w, v| EntityId::Item(crate::canned_evil::gates::star_room_gate_new(w, v[0].bool())),
    },
];

static C_SERVER_ITEMS_UNCUTCLOTH: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.UncutCloth", v[0].i32()),
    },
];

static C_SERVER_ITEMS_UNICORNRIBS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| {
            let ty = item_ty(w, "Server.Items.UnicornRibs");
            EntityId::Item(crate::ml_quests::items_gen::unicorn_ribs_new(w, ty, v[0].i32()))
        },
    },
];

static C_SERVER_ITEMS_WHITEPEARL: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "amount", kind: INT, default: Some(Value::Int(1)), ty: "" }],
        make: |w, v| ctor_amount(w, "Server.Items.WhitePearl", v[0].i32()),
    },
];

fn configure(w: &mut World) {
    add_generated_ctors(w, "Server.Engines.MLQuests.Items.MLQuestTeleporter", &C_SERVER_ENGINES_MLQUESTS_ITEMS_MLQUESTTELEPORTER);
    add_generated_ctors(w, "Server.Engines.MLQuests.Items.TicketTeleporter", &C_SERVER_ENGINES_MLQUESTS_ITEMS_TICKETTELEPORTER);
    add_generated_ctors(w, "Server.Engines.Plants.Seed", &C_SERVER_ENGINES_PLANTS_SEED);
    add_generated_ctors(w, "Server.Engines.Spawners.ProximitySpawner", &C_SERVER_ENGINES_SPAWNERS_PROXIMITYSPAWNER);
    add_generated_ctors(w, "Server.Engines.Spawners.RegionSpawner", &C_SERVER_ENGINES_SPAWNERS_REGIONSPAWNER);
    add_generated_ctors(w, "Server.Engines.Spawners.Spawner", &C_SERVER_ENGINES_SPAWNERS_SPAWNER);
    add_generated_ctors(w, "Server.Factions.Silver", &C_SERVER_FACTIONS_SILVER);
    add_generated_ctors(w, "Server.Items.AncientUrn", &C_SERVER_ITEMS_ANCIENTURN);
    add_generated_ctors(w, "Server.Items.BlankScroll", &C_SERVER_ITEMS_BLANKSCROLL);
    add_generated_ctors(w, "Server.Items.Cloth", &C_SERVER_ITEMS_CLOTH);
    add_generated_ctors(w, "Server.Items.GamanHorns", &C_SERVER_ITEMS_GAMANHORNS);
    add_generated_ctors(w, "Server.Items.GreenThorns", &C_SERVER_ITEMS_GREENTHORNS);
    add_generated_ctors(w, "Server.Items.HonorableSwords", &C_SERVER_ITEMS_HONORABLESWORDS);
    add_generated_ctors(w, "Server.Items.HoodedShroudOfShadows", &C_SERVER_ITEMS_HOODEDSHROUDOFSHADOWS);
    add_generated_ctors(w, "Server.Items.KhaldunPitTeleporter", &C_SERVER_ITEMS_KHALDUNPITTELEPORTER);
    add_generated_ctors(w, "Server.Items.LesserPigmentsOfTokuno", &C_SERVER_ITEMS_LESSERPIGMENTSOFTOKUNO);
    add_generated_ctors(w, "Server.Items.OrangePetals", &C_SERVER_ITEMS_ORANGEPETALS);
    add_generated_ctors(w, "Server.Items.PixieLeg", &C_SERVER_ITEMS_PIXIELEG);
    add_generated_ctors(w, "Server.Items.RedLeaves", &C_SERVER_ITEMS_REDLEAVES);
    add_generated_ctors(w, "Server.Items.Runebook", &C_SERVER_ITEMS_RUNEBOOK);
    add_generated_ctors(w, "Server.Items.SalvageBag", &C_SERVER_ITEMS_SALVAGEBAG);
    add_generated_ctors(w, "Server.Items.SapOfSosaria", &C_SERVER_ITEMS_SAPOFSOSARIA);
    add_generated_ctors(w, "Server.Items.SeveredElfEars", &C_SERVER_ITEMS_SEVEREDELFEARS);
    add_generated_ctors(w, "Server.Items.SeveredHumanEars", &C_SERVER_ITEMS_SEVEREDHUMANEARS);
    add_generated_ctors(w, "Server.Items.StarRoomGate", &C_SERVER_ITEMS_STARROOMGATE);
    add_generated_ctors(w, "Server.Items.UncutCloth", &C_SERVER_ITEMS_UNCUTCLOTH);
    add_generated_ctors(w, "Server.Items.UnicornRibs", &C_SERVER_ITEMS_UNICORNRIBS);
    add_generated_ctors(w, "Server.Items.WhitePearl", &C_SERVER_ITEMS_WHITEPEARL);
}

/// A hand-written table for the same class wins, whichever runs first.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(Phase::Configure, "muo_engines.CtorTables.Configure", configure);
}
