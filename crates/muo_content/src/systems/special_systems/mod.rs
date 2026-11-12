//! Special systems: `ItemFixes`, `PreventInaccess`, `GiftGiving`, `ResGate` and the supply stones.
//!
//! The supply bags and `TestCenter` are in `supply_bags` and `test_center`; the bags are made of items other crates
//! define, by class name (see their documentation).

pub mod gift_giving;
pub mod item_fixes;
pub mod prevent_inaccess;
pub mod stones;
pub mod supply_bags;
pub mod test_center;

use muo_server::type_registry::{Phase, TypeRegistry};

pub fn register(t: &mut TypeRegistry) {
    t.register_phase_default(
        Phase::Initialize,
        "Server.Misc.ItemFixes.Initialize",
        item_fixes::initialize,
    );
    t.register_phase_default(
        Phase::Initialize,
        "Server.Misc.PreventInaccess.Initialize",
        prevent_inaccess::initialize,
    );
    for ty in stones::ALL_ITEMS.into_iter().chain(supply_bags::ALL_ITEMS) {
        t.register_item(ty, &[]);
    }
    t.register_phase_default(
        Phase::Configure,
        "Server.Items.SupplyBags.Configure",
        supply_bags::configure,
    );
    test_center::register(t);
}
