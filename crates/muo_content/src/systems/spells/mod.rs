//! The spell base (and Reagent, UnsummonTimer): the spell system every concrete spell stands on. The concrete
//! spells are `muo_spells`.
//!
//! Every concrete spell is registered by id; each spell crate registers its own classes into
//! [`registry::SpellRegistry`] from a `Configure` phase function (see `muo_spells::register`).

pub mod helper;
pub mod hooks;
pub mod magery;
pub mod registry;
pub mod special_move;
pub mod spell;
pub mod targeting;
pub mod types;
pub mod unsummon_timer;

use muo_server::World;
use muo_server::type_registry::{Phase, TypeRegistry};

pub use spell::{Spell, SpellId, SpellType, Spells};
pub use types::{DisturbType, SpellCircle, SpellInfo, SpellState, TravelCheckType, reagent};

/// Hands muo_server's mobile code the spell system: `Mobile.m_Spell?.OnCasterHurt()` and the rest.
// muo_server's `mobile::part2::deps` spell stubs are replaced by this table (see `SpellDeps` there).
pub fn configure(w: &mut World) {
    let d = w
        .resources
        .get_mut::<muo_server::mobile::part2::deps::SpellDeps>();
    d.on_caster_hurt = Some(spell::caster_hurt);
    d.on_caster_killed = Some(spell::caster_killed);
    d.on_connection_changed = Some(spell::connection_changed);
    d.on_caster_using_object = Some(spell::caster_using_object);
    d.on_caster_equipping = Some(spell::caster_equipping);
    d.blocks_movement = Some(spell::spell_blocks_movement);
    d.on_caster_moving = Some(spell::caster_moving);

    // `SpellHelper.IsMLDungeon`
    hooks::hooks_mut(w).is_ml_dungeon = Some(|w, map, loc| {
        crate::loot_pack::is_ml_region(w, muo_server::region::find(w, loc, Some(map)))
    });
}

pub fn register(t: &mut TypeRegistry) {
    t.register_phase_default(
        Phase::Configure,
        "Server.Spells.SpellSystem.Configure",
        configure,
    );
}
