//! The `[add <type> <args>` overload tables of this crate's `[Constructible]` constructors, in declaration order.

use muo_content::props::{Ctor, Kind, Param, add_generated_ctors, ctor_default};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ids::EntityId;
use muo_server::type_registry::{Phase, TypeRegistry};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const ENTITY: Kind = Kind::Plain(PropType::Entity);

static C_SERVER_ITEMS_ARCANEFOCUS: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| ctor_default(w, "Server.Items.ArcaneFocus"),
    },
];

static C_SERVER_MOBILES_ANIMATEDWEAPON: [Ctor; 1] = [
    Ctor {
        access: None,
        params: &[Param { name: "caster", kind: ENTITY, default: None, ty: "Mobile" }, Param { name: "level", kind: INT, default: None, ty: "" }],
        make: |w, v| EntityId::Mobile(crate::spellweaving::mobiles::animated_weapon_new(w, v[0].mobile(), v[1].i32())),
    },
];

fn configure(w: &mut World) {
    add_generated_ctors(w, "Server.Items.ArcaneFocus", &C_SERVER_ITEMS_ARCANEFOCUS);
    add_generated_ctors(w, "Server.Mobiles.AnimatedWeapon", &C_SERVER_MOBILES_ANIMATEDWEAPON);
}

/// A hand-written table for the same class wins, whichever runs first.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(Phase::Configure, "muo_spells.CtorTables.Configure", configure);
}
