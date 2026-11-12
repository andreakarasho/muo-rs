//! The Cartography craft list (`InitCraftList`), generated: do not edit by hand.

use crate::craft::model::*;
use muo_io::text::TextDefinition as Td;
use muo_server::World;

/// `InitCraftList()`.
pub fn init(_w: &World, s: &mut CraftSystem) {
    s.add_craft(
        "Server.Items.LocalMap",
        Td::of_number(1044448),
        Td::of_number(1015230),
        None,
        None,
        10.0,
        70.0,
        "Server.Items.BlankMap",
        Td::of_number(1044449),
        1,
        Td::of_number(1044450),
    );
    s.add_craft(
        "Server.Items.CityMap",
        Td::of_number(1044448),
        Td::of_number(1015231),
        None,
        None,
        25.0,
        85.0,
        "Server.Items.BlankMap",
        Td::of_number(1044449),
        1,
        Td::of_number(1044450),
    );
    s.add_craft(
        "Server.Items.SeaChart",
        Td::of_number(1044448),
        Td::of_number(1015232),
        None,
        None,
        35.0,
        95.0,
        "Server.Items.BlankMap",
        Td::of_number(1044449),
        1,
        Td::of_number(1044450),
    );
    s.add_craft(
        "Server.Items.WorldMap",
        Td::of_number(1044448),
        Td::of_number(1015233),
        None,
        None,
        39.5,
        99.5,
        "Server.Items.BlankMap",
        Td::of_number(1044449),
        1,
        Td::of_number(1044450),
    );
}
