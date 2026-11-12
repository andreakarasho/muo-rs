//! FactionAllegiance and EthicAllegiance overrides: the faction index is the factions engine's (0 Council of Mages,
//! 1 Minax, 2 Shadowlords, 3 True Britannians), the ethic 0 Hero and 1 Evil.

use muo_server::World;
use muo_server::ids::MobileId;

macro_rules! faction {
    ($name:ident, $index:expr) => {
        pub fn $name(_w: &World, _id: MobileId) -> Option<i32> {
            Some($index)
        }
    };
}
faction!(daemon_faction_allegiance, 2); // Shadowlords
faction!(ogre_lord_faction_allegiance, 1); // Minax
faction!(wisp_faction_allegiance, 0); // CouncilOfMages
faction!(silver_serpent_faction_allegiance, 3); // TrueBritannians

macro_rules! ethic {
    ($name:ident, $side:expr) => {
        pub fn $name(_w: &World, _id: MobileId) -> Option<i32> {
            Some($side)
        }
    };
}
ethic!(daemon_ethic_allegiance, 1);
ethic!(ogre_lord_ethic_allegiance, 1);
ethic!(dark_wisp_ethic_allegiance, 1);
ethic!(wisp_ethic_allegiance, 0);
ethic!(silver_serpent_ethic_allegiance, 0);
