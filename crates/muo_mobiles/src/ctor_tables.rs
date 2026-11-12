//! The `[add <type> <args>` overload tables of this crate's `[Constructible]` constructors, in declaration order.
//! A constructor with no Rust counterpart is listed in target/ctors-missing.txt.

use muo_content::props::{Ctor, Kind, Param, Value, add_generated_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ids::EntityId;
use muo_server::type_registry::{Phase, TypeRegistry};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const BOOL: Kind = Kind::Plain(PropType::Bool);
const DOUBLE: Kind = Kind::Plain(PropType::Double);
const ENTITY: Kind = Kind::Plain(PropType::Entity);

static C_SERVER_MOBILES_ETHEREALMOUNT: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "itemID", kind: INT, default: None, ty: "" }, Param { name: "mountID", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Item(crate::classes::ethereal::ethereal_mount_new(w, v[0].i32(), v[1].i32())),
    },
];

static C_SERVER_MOBILES_GOLEM: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "summoned", kind: BOOL, default: Some(Value::Bool(false)), ty: "" }, Param { name: "scalar", kind: DOUBLE, default: Some(Value::Float(1.0)), ty: "" }],
        make: |w, v| EntityId::Mobile(crate::classes::misc4::new_golem(w, v[0].bool(), v[1].f64())),
    },
];

static C_SERVER_MOBILES_LEVIATHAN: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "fisher", kind: ENTITY, default: Some(Value::Null), ty: "Mobile" }],
        make: |w, v| EntityId::Mobile(crate::classes::misc4::new_leviathan(w, v[0].mobile())),
    },
];

static C_SERVER_MOBILES_PLAGUESPAWN: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "owner", kind: ENTITY, default: Some(Value::Null), ty: "Mobile" }],
        make: |w, v| EntityId::Mobile(crate::classes::misc4::new_plague_spawn(w, v[0].mobile())),
    },
];

static C_SERVER_MOBILES_STAINEDOOZE: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "corrosive", kind: BOOL, default: Some(Value::Bool(false)), ty: "" }],
        make: |w, v| EntityId::Item(crate::classes::ilhenir::new_stained_ooze(w, v[0].bool())),
    },
];

fn configure(w: &mut World) {
    add_generated_ctors(w, "Server.Mobiles.EtherealMount", &C_SERVER_MOBILES_ETHEREALMOUNT);
    add_generated_ctors(w, "Server.Mobiles.Golem", &C_SERVER_MOBILES_GOLEM);
    add_generated_ctors(w, "Server.Mobiles.Leviathan", &C_SERVER_MOBILES_LEVIATHAN);
    add_generated_ctors(w, "Server.Mobiles.PlagueSpawn", &C_SERVER_MOBILES_PLAGUESPAWN);
    add_generated_ctors(w, "Server.Mobiles.StainedOoze", &C_SERVER_MOBILES_STAINEDOOZE);
}

/// A hand-written table for the same class wins, whichever runs first.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(Phase::Configure, "muo_mobiles.CtorTables.Configure", configure);
}
