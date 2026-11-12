//! Monster and animal classes with state or logic of their own (each has its `creature_leaf!` here). Their
//! children are table classes.

pub mod bedlam;
pub mod demon_knight;
pub mod enraged;
pub mod ethereal;
pub mod holiday;
pub mod holiday_events;
mod hooks;
pub mod ilhenir;
pub mod misc4;
pub mod monsters5;
pub use hooks::cow_try_milk;
pub mod swamp_dragon;

use muo_server::mobile::MobileType;

/// Every class of this folder.
pub fn all() -> Vec<&'static MobileType> {
    vec![
        &demon_knight::DEMON_KNIGHT,
        &swamp_dragon::SWAMP_DRAGON,
        &enraged::BASE_ENRAGED,
        &enraged::ENRAGED_RABBIT,
        &enraged::ENRAGED_HART,
        &enraged::ENRAGED_HIND,
        &enraged::ENRAGED_BLACK_BEAR,
        &enraged::ENRAGED_EAGLE,
        &misc4::GOLEM,
        &misc4::PLAGUE_SPAWN,
        &misc4::LEVIATHAN,
        &misc4::TWAULO,
        &bedlam::LADY_MARAI,
        &ilhenir::ILHENIR,
        &ilhenir::MERAKTUS,
        &monsters5::REVENANT,
        &monsters5::KHALDUN_REVENANT,
    ]
}

pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    demon_knight::register(types);
    swamp_dragon::register(types);
    enraged::register(types);
    misc4::register(types);
    monsters5::register(types);
    ilhenir::register(types);
    bedlam::register(types);
    holiday::register(types);
    holiday_events::register(types);
    ethereal::register(types);
    crate::hand::register_cloned_item(types);
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Mobiles.Animals.Configure",
        hooks::configure,
    );
}
