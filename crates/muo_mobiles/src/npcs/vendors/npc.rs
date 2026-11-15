//! The NPC vendor classes whose constructor and class members are data (`Data/Templates/Mobiles/npcs/vendors.json`):
//! the template (`title`, `skills`) runs after `new BaseVendor()`, and the `VendorClass` members read the
//! template's props (`template_keys_npcs`): `npcSbInfos`, `npcGuild`, `npcShoeType`, `npcShoeHue`, `npcOutfit`.

use crate::template_keys_npcs as keys;
use muo_content::vendor::{BASE_VENDOR_CLASS, VendorClass, new_vendor};
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

/// The class members of a vendor whose values are its template's props.
pub static TEMPLATE_VENDOR_V: VendorClass = VendorClass {
    init_sb_info: super::sb::from_props,
    npc_guild: keys::npc_guild,
    shoe_type: keys::shoe_type,
    get_shoe_hue: keys::shoe_hue,
    init_outfit: keys::init_outfit,
    ..BASE_VENDOR_CLASS
};

/// `new T()` of a vendor class whose constructor is its template.
pub fn new_template_vendor(w: &mut World, ty: &'static MobileType) -> MobileId {
    let id = new_vendor(w, ty, None);
    muo_content::templates::apply_mobile(w, id);
    id
}

macro_rules! template_vendors {
    ($($name:ident, $class:ident, $vclass:ident = $full:literal;)*) => {
        $(
            pub static $vclass: VendorClass = VendorClass { ..TEMPLATE_VENDOR_V };
            vendor_leaf!($name, $class, $vclass = $full, ctor |w| new_template_vendor(w, &$name));
        )*

        /// Every class of this file.
        pub fn all() -> Vec<&'static MobileType> {
            vec![$(&$name,)*]
        }
    };
}

template_vendors! {
    ALCHEMIST, ALCHEMIST_C, ALCHEMIST_V = "Server.Mobiles.Alchemist";
    ARCHITECT, ARCHITECT_C, ARCHITECT_V = "Server.Mobiles.Architect";
    ARMORER, ARMORER_C, ARMORER_V = "Server.Mobiles.Armorer";
    BAKER, BAKER_C, BAKER_V = "Server.Mobiles.Baker";
    BARD, BARD_C, BARD_V = "Server.Mobiles.Bard";
    BARKEEPER, BARKEEPER_C, BARKEEPER_V = "Server.Mobiles.Barkeeper";
    BEEKEEPER, BEEKEEPER_C, BEEKEEPER_V = "Server.Mobiles.Beekeeper";
    BOWYER, BOWYER_C, BOWYER_V = "Server.Mobiles.Bowyer";
    BUTCHER, BUTCHER_C, BUTCHER_V = "Server.Mobiles.Butcher";
    CARPENTER, CARPENTER_C, CARPENTER_V = "Server.Mobiles.Carpenter";
    COBBLER, COBBLER_C, COBBLER_V = "Server.Mobiles.Cobbler";
    COOK, COOK_C, COOK_V = "Server.Mobiles.Cook";
    FARMER, FARMER_C, FARMER_V = "Server.Mobiles.Farmer";
    FISHERMAN, FISHERMAN_C, FISHERMAN_V = "Server.Mobiles.Fisherman";
    FURTRADER, FURTRADER_C, FURTRADER_V = "Server.Mobiles.Furtrader";
    GLASSBLOWER, GLASSBLOWER_C, GLASSBLOWER_V = "Server.Mobiles.Glassblower";
    GOLEM_CRAFTER, GOLEM_CRAFTER_C, GOLEM_CRAFTER_V = "Server.Mobiles.GolemCrafter";
    HAIR_STYLIST, HAIR_STYLIST_C, HAIR_STYLIST_V = "Server.Mobiles.HairStylist";
    HERBALIST, HERBALIST_C, HERBALIST_V = "Server.Mobiles.Herbalist";
    INN_KEEPER, INN_KEEPER_C, INN_KEEPER_V = "Server.Mobiles.InnKeeper";
    JEWELER, JEWELER_C, JEWELER_V = "Server.Mobiles.Jeweler";
    LEATHER_WORKER, LEATHER_WORKER_C, LEATHER_WORKER_V = "Server.Mobiles.LeatherWorker";
    MAGE, MAGE_C, MAGE_V = "Server.Mobiles.Mage";
    MAPMAKER, MAPMAKER_C, MAPMAKER_V = "Server.Mobiles.Mapmaker";
    MILLER, MILLER_C, MILLER_V = "Server.Mobiles.Miller";
    MINER, MINER_C, MINER_V = "Server.Mobiles.Miner";
    MONK, MONK_C, MONK_V = "Server.Mobiles.Monk";
    PROVISIONER, PROVISIONER_C, PROVISIONER_V = "Server.Mobiles.Provisioner";
    RANCHER, RANCHER_C, RANCHER_V = "Server.Mobiles.Rancher";
    RANGER, RANGER_C, RANGER_V = "Server.Mobiles.Ranger";
    SHIPWRIGHT, SHIPWRIGHT_C, SHIPWRIGHT_V = "Server.Mobiles.Shipwright";
    STONE_CRAFTER, STONE_CRAFTER_C, STONE_CRAFTER_V = "Server.Mobiles.StoneCrafter";
    TANNER, TANNER_C, TANNER_V = "Server.Mobiles.Tanner";
    TAVERN_KEEPER, TAVERN_KEEPER_C, TAVERN_KEEPER_V = "Server.Mobiles.TavernKeeper";
    THIEF, THIEF_C, THIEF_V = "Server.Mobiles.Thief";
    TINKER, TINKER_C, TINKER_V = "Server.Mobiles.Tinker";
    VARIETY_DEALER, VARIETY_DEALER_C, VARIETY_DEALER_V = "Server.Mobiles.VarietyDealer";
    VETERINARIAN, VETERINARIAN_C, VETERINARIAN_V = "Server.Mobiles.Veterinarian";
    WAITER, WAITER_C, WAITER_V = "Server.Mobiles.Waiter";
}

pub(crate) fn register(types: &mut TypeRegistry) {
    for ty in all() {
        let aliases: &[&str] = match ty.full_name {
            "Server.Mobiles.Bowyer" => &["Server.Mobiles.Bower"],
            "Server.Mobiles.Glassblower" => &["Server.Mobiles.GargoyleAlchemist"],
            "Server.Mobiles.StoneCrafter" => &["Server.Mobiles.GargoyleStonecrafter"],
            _ => &[],
        };
        types.register_mobile(ty, aliases);
    }
}
