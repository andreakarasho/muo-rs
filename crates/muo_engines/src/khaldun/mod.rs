//! Khaldun: the pit teleporter, the raisable item and its switches, the four cursed expedition
//! members, the 28 journals/notebooks, the puzzle chests (`PuzzleChest`) and `[GenKhaldun`.

use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

pub mod books;
pub mod items;
pub mod khaldun_gen;
pub mod mobiles;
pub mod puzzle;

pub fn register(types: &mut TypeRegistry) {
    items::register(types);
    mobiles::register(types);
    books::register(types);
    puzzle::register(types);
    types.register_phase(
        Phase::Configure,
        "GenKhaldun.Configure",
        DEFAULT_CALL_PRIORITY,
        khaldun_gen::configure,
    );
}
