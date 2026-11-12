//! The NPC vendor classes that are plain tables.

use super::sb::*;
use super::{add_item_hued, replace_equipment};
use crate::common::*;
use muo_content::hooks::rnd;
use muo_content::player_mobile::NpcGuild;
use muo_content::vendor::{BASE_VENDOR_CLASS, ShoeType, VendorClass, is_tokuno_vendor, new_vendor};
use muo_primitives::utility;
use muo_server::ids::MobileId;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;
fn alchemist_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &ALCHEMIST, Some("the alchemist"));
    set_skill_range(w, id, SkillName::Alchemy, 85.0, 100.0);
    set_skill_range(w, id, SkillName::TasteID, 65.0, 88.0);
    id
}
pub static ALCHEMIST_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBALCHEMIST.new_for(w, id)],
    npc_guild: |_, _| NpcGuild::MagesGuild,
    shoe_type: |w, _id| {
        if rnd::bool(w) {
            ShoeType::Shoes
        } else {
            ShoeType::Sandals
        }
    },
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        let h = utility::random_pink_hue(&mut w.rng);
        add_item_hued(w, id, "Server.Items.Robe", Some(h));
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(ALCHEMIST, ALCHEMIST_C, ALCHEMIST_V = "Server.Mobiles.Alchemist", ctor alchemist_new);

fn architect_new(w: &mut World) -> MobileId {
    new_vendor(w, &ARCHITECT, Some("the architect"))
}
pub static ARCHITECT_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        let mut v = Vec::new();
        if !w.core_aos() {
            v.push(SBHOUSE_DEED.new_for(w, id));
        }
        v.push(SBARCHITECT.new_for(w, id));
        v
    },
    npc_guild: |_, _| NpcGuild::TinkersGuild,
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(ARCHITECT, ARCHITECT_C, ARCHITECT_V = "Server.Mobiles.Architect", ctor architect_new);

fn armorer_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &ARMORER, Some("the armorer"));
    set_skill_range(w, id, SkillName::ArmsLore, 64.0, 100.0);
    set_skill_range(w, id, SkillName::Blacksmith, 60.0, 83.0);
    id
}
pub static ARMORER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        let mut v = Vec::new();
        match rnd::below(w, 4) {
            0 => {
                v.push(SBLEATHER_ARMOR.new_for(w, id));
                v.push(SBSTUDDED_ARMOR.new_for(w, id));
                v.push(SBMETAL_SHIELDS.new_for(w, id));
                v.push(SBPLATE_ARMOR.new_for(w, id));
                v.push(SBHELMET_ARMOR.new_for(w, id));
                v.push(SBCHAINMAIL_ARMOR.new_for(w, id));
                v.push(SBRINGMAIL_ARMOR.new_for(w, id));
            }
            1 => {
                v.push(SBSTUDDED_ARMOR.new_for(w, id));
                v.push(SBLEATHER_ARMOR.new_for(w, id));
                v.push(SBMETAL_SHIELDS.new_for(w, id));
                v.push(SBHELMET_ARMOR.new_for(w, id));
            }
            2 => {
                v.push(SBMETAL_SHIELDS.new_for(w, id));
                v.push(SBPLATE_ARMOR.new_for(w, id));
                v.push(SBHELMET_ARMOR.new_for(w, id));
                v.push(SBCHAINMAIL_ARMOR.new_for(w, id));
                v.push(SBRINGMAIL_ARMOR.new_for(w, id));
            }
            3 => {
                v.push(SBMETAL_SHIELDS.new_for(w, id));
                v.push(SBHELMET_ARMOR.new_for(w, id));
            }
            _ => {}
        }
        if is_tokuno_vendor(w, id) {
            v.push(SBSELEATHER_ARMOR.new_for(w, id));
            v.push(SBSEARMOR.new_for(w, id));
        }
        v
    },
    shoe_type: |_, _| ShoeType::Boots,
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        let h = utility::random_yellow_hue(&mut w.rng);
        add_item_hued(w, id, "Server.Items.HalfApron", Some(h));
        add_item_hued(w, id, "Server.Items.Bascinet", None);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(ARMORER, ARMORER_C, ARMORER_V = "Server.Mobiles.Armorer", ctor armorer_new);

fn baker_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &BAKER, Some("the baker"));
    set_skill_range(w, id, SkillName::Cooking, 75.0, 98.0);
    set_skill_range(w, id, SkillName::TasteID, 36.0, 68.0);
    id
}
pub static BAKER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBBAKER.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(BAKER, BAKER_C, BAKER_V = "Server.Mobiles.Baker", ctor baker_new);

fn bard_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &BARD, Some("the bard"));
    set_skill_range(w, id, SkillName::Discordance, 64.0, 100.0);
    set_skill_range(w, id, SkillName::Musicianship, 64.0, 100.0);
    set_skill_range(w, id, SkillName::Peacemaking, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Provocation, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Archery, 36.0, 68.0);
    set_skill_range(w, id, SkillName::Swords, 36.0, 68.0);
    id
}
pub static BARD_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBBARD.new_for(w, id)],
    npc_guild: |_, _| NpcGuild::BardsGuild,
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(BARD, BARD_C, BARD_V = "Server.Mobiles.Bard", ctor bard_new);

fn barkeeper_new(w: &mut World) -> MobileId {
    new_vendor(w, &BARKEEPER, Some("the barkeeper"))
}
pub static BARKEEPER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBBARKEEPER.new_for(w, id)],
    shoe_type: |w, _id| {
        if rnd::bool(w) {
            ShoeType::ThighBoots
        } else {
            ShoeType::Boots
        }
    },
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        let h = utility::random_bright_hue(&mut w.rng);
        add_item_hued(w, id, "Server.Items.HalfApron", Some(h));
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(BARKEEPER, BARKEEPER_C, BARKEEPER_V = "Server.Mobiles.Barkeeper", ctor barkeeper_new);

fn beekeeper_new(w: &mut World) -> MobileId {
    new_vendor(w, &BEEKEEPER, Some("the beekeeper"))
}
pub static BEEKEEPER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBBEEKEEPER.new_for(w, id)],
    shoe_type: |_, _| ShoeType::Boots,
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(BEEKEEPER, BEEKEEPER_C, BEEKEEPER_V = "Server.Mobiles.Beekeeper", ctor beekeeper_new);

fn bowyer_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &BOWYER, Some("the bowyer"));
    set_skill_range(w, id, SkillName::Fletching, 80.0, 100.0);
    set_skill_range(w, id, SkillName::Archery, 80.0, 100.0);
    id
}
pub static BOWYER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        let mut v = Vec::new();
        v.push(SBBOWYER.new_for(w, id));
        v.push(SBRANGED_WEAPON.new_for(w, id));
        if is_tokuno_vendor(w, id) {
            v.push(SBSEBOWYER.new_for(w, id));
        }
        v
    },
    shoe_type: |w, id| {
        if w.mobile(id).is_some_and(|m| m.female) {
            ShoeType::ThighBoots
        } else {
            ShoeType::Boots
        }
    },
    get_shoe_hue: |_, _| 0,
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        add_item_hued(w, id, "Server.Items.Bow", None);
        add_item_hued(w, id, "Server.Items.LeatherGorget", None);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(BOWYER, BOWYER_C, BOWYER_V = "Server.Mobiles.Bowyer", ctor bowyer_new);

fn butcher_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &BUTCHER, Some("the butcher"));
    set_skill_range(w, id, SkillName::Anatomy, 45.0, 68.0);
    id
}
pub static BUTCHER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBBUTCHER.new_for(w, id)],
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        add_item_hued(w, id, "Server.Items.HalfApron", None);
        add_item_hued(w, id, "Server.Items.Cleaver", None);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(BUTCHER, BUTCHER_C, BUTCHER_V = "Server.Mobiles.Butcher", ctor butcher_new);

fn carpenter_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &CARPENTER, Some("the carpenter"));
    set_skill_range(w, id, SkillName::Carpentry, 85.0, 100.0);
    set_skill_range(w, id, SkillName::Lumberjacking, 60.0, 83.0);
    id
}
pub static CARPENTER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        let mut v = Vec::new();
        v.push(SBSTAVES_WEAPON.new_for(w, id));
        v.push(SBCARPENTER.new_for(w, id));
        v.push(SBWOODEN_SHIELDS.new_for(w, id));
        if is_tokuno_vendor(w, id) {
            v.push(SBSECARPENTER.new_for(w, id));
        }
        v
    },
    npc_guild: |_, _| NpcGuild::TinkersGuild,
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        add_item_hued(w, id, "Server.Items.HalfApron", None);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(CARPENTER, CARPENTER_C, CARPENTER_V = "Server.Mobiles.Carpenter", ctor carpenter_new);

fn cobbler_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &COBBLER, Some("the cobbler"));
    set_skill_range(w, id, SkillName::Tailoring, 60.0, 83.0);
    id
}
pub static COBBLER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBCOBBLER.new_for(w, id)],
    shoe_type: |w, _id| {
        if rnd::bool(w) {
            ShoeType::Sandals
        } else {
            ShoeType::Shoes
        }
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(COBBLER, COBBLER_C, COBBLER_V = "Server.Mobiles.Cobbler", ctor cobbler_new);

fn cook_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &COOK, Some("the cook"));
    set_skill_range(w, id, SkillName::Cooking, 90.0, 100.0);
    set_skill_range(w, id, SkillName::TasteID, 75.0, 98.0);
    id
}
pub static COOK_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        let mut v = Vec::new();
        v.push(SBCOOK.new_for(w, id));
        if is_tokuno_vendor(w, id) {
            v.push(SBSECOOK.new_for(w, id));
        }
        v
    },
    shoe_type: |w, _id| {
        if rnd::bool(w) {
            ShoeType::Sandals
        } else {
            ShoeType::Shoes
        }
    },
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        add_item_hued(w, id, "Server.Items.HalfApron", None);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(COOK, COOK_C, COOK_V = "Server.Mobiles.Cook", ctor cook_new);

fn farmer_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &FARMER, Some("the farmer"));
    set_skill_range(w, id, SkillName::Lumberjacking, 36.0, 68.0);
    set_skill_range(w, id, SkillName::TasteID, 36.0, 68.0);
    set_skill_range(w, id, SkillName::Cooking, 36.0, 68.0);
    id
}
pub static FARMER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBFARMER.new_for(w, id)],
    shoe_type: |_, _| ShoeType::ThighBoots,
    get_shoe_hue: |_, _| 0,
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        let h = utility::random_neutral_hue(&mut w.rng);
        add_item_hued(w, id, "Server.Items.WideBrimHat", Some(h));
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(FARMER, FARMER_C, FARMER_V = "Server.Mobiles.Farmer", ctor farmer_new);

fn fisherman_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &FISHERMAN, Some("the fisher"));
    set_skill_range(w, id, SkillName::Fishing, 75.0, 98.0);
    id
}
pub static FISHERMAN_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBFISHERMAN.new_for(w, id)],
    npc_guild: |_, _| NpcGuild::FishermensGuild,
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        add_item_hued(w, id, "Server.Items.FishingPole", None);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(FISHERMAN, FISHERMAN_C, FISHERMAN_V = "Server.Mobiles.Fisherman", ctor fisherman_new);

fn furtrader_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &FURTRADER, Some("the furtrader"));
    set_skill_range(w, id, SkillName::Camping, 55.0, 78.0);
    set_skill_range(w, id, SkillName::AnimalLore, 85.0, 100.0);
    set_skill_range(w, id, SkillName::Cooking, 45.0, 68.0);
    set_skill_range(w, id, SkillName::Tracking, 36.0, 68.0);
    id
}
pub static FURTRADER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBFURTRADER.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(FURTRADER, FURTRADER_C, FURTRADER_V = "Server.Mobiles.Furtrader", ctor furtrader_new);

fn glassblower_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &GLASSBLOWER, Some("the alchemist"));
    set_skill_range(w, id, SkillName::Alchemy, 85.0, 100.0);
    set_skill_range(w, id, SkillName::TasteID, 85.0, 100.0);
    id
}
pub static GLASSBLOWER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBGLASSBLOWER.new_for(w, id), SBALCHEMIST.new_for(w, id)],
    npc_guild: |_, _| NpcGuild::MagesGuild,
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(GLASSBLOWER, GLASSBLOWER_C, GLASSBLOWER_V = "Server.Mobiles.Glassblower", ctor glassblower_new);

fn golem_crafter_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &GOLEM_CRAFTER, Some("the golem crafter"));
    set_skill_range(w, id, SkillName::Lockpicking, 60.0, 83.0);
    set_skill_range(w, id, SkillName::RemoveTrap, 75.0, 98.0);
    set_skill_range(w, id, SkillName::Tinkering, 64.0, 100.0);
    id
}
pub static GOLEM_CRAFTER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBTINKER.new_for(w, id), SBVAGABOND.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(GOLEM_CRAFTER, GOLEM_CRAFTER_C, GOLEM_CRAFTER_V = "Server.Mobiles.GolemCrafter", ctor golem_crafter_new);

fn hair_stylist_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &HAIR_STYLIST, Some("the hair stylist"));
    set_skill_range(w, id, SkillName::Alchemy, 80.0, 100.0);
    set_skill_range(w, id, SkillName::Magery, 90.0, 110.0);
    set_skill_range(w, id, SkillName::TasteID, 85.0, 100.0);
    id
}
pub static HAIR_STYLIST_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBHAIR_STYLIST.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(HAIR_STYLIST, HAIR_STYLIST_C, HAIR_STYLIST_V = "Server.Mobiles.HairStylist", ctor hair_stylist_new);

fn herbalist_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &HERBALIST, Some("the herbalist"));
    set_skill_range(w, id, SkillName::Alchemy, 80.0, 100.0);
    set_skill_range(w, id, SkillName::Cooking, 80.0, 100.0);
    set_skill_range(w, id, SkillName::TasteID, 80.0, 100.0);
    id
}
pub static HERBALIST_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBHERBALIST.new_for(w, id)],
    npc_guild: |_, _| NpcGuild::MagesGuild,
    shoe_type: |w, _id| {
        if rnd::bool(w) {
            ShoeType::Shoes
        } else {
            ShoeType::Sandals
        }
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(HERBALIST, HERBALIST_C, HERBALIST_V = "Server.Mobiles.Herbalist", ctor herbalist_new);

fn inn_keeper_new(w: &mut World) -> MobileId {
    new_vendor(w, &INN_KEEPER, Some("the innkeeper"))
}
pub static INN_KEEPER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        let mut v = Vec::new();
        v.push(SBINN_KEEPER.new_for(w, id));
        if is_tokuno_vendor(w, id) {
            v.push(SBSEFOOD.new_for(w, id));
        }
        v
    },
    shoe_type: |w, _id| {
        if rnd::bool(w) {
            ShoeType::Sandals
        } else {
            ShoeType::Shoes
        }
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(INN_KEEPER, INN_KEEPER_C, INN_KEEPER_V = "Server.Mobiles.InnKeeper", ctor inn_keeper_new);

fn jeweler_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &JEWELER, Some("the jeweler"));
    set_skill_range(w, id, SkillName::ItemID, 64.0, 100.0);
    id
}
pub static JEWELER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBJEWEL.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(JEWELER, JEWELER_C, JEWELER_V = "Server.Mobiles.Jeweler", ctor jeweler_new);

fn leather_worker_new(w: &mut World) -> MobileId {
    new_vendor(w, &LEATHER_WORKER, Some("the leather worker"))
}
pub static LEATHER_WORKER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        vec![
            SBLEATHER_ARMOR.new_for(w, id),
            SBSTUDDED_ARMOR.new_for(w, id),
            SBLEATHER_WORKER.new_for(w, id),
        ]
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(LEATHER_WORKER, LEATHER_WORKER_C, LEATHER_WORKER_V = "Server.Mobiles.LeatherWorker", ctor leather_worker_new);

fn mage_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &MAGE, Some("the mage"));
    set_skill_range(w, id, SkillName::EvalInt, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Inscribe, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Magery, 64.0, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 83.0);
    set_skill_range(w, id, SkillName::MagicResist, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Wrestling, 36.0, 68.0);
    id
}
pub static MAGE_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBMAGE.new_for(w, id)],
    npc_guild: |_, _| NpcGuild::MagesGuild,
    shoe_type: |w, _id| {
        if rnd::bool(w) {
            ShoeType::Shoes
        } else {
            ShoeType::Sandals
        }
    },
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        let h = utility::random_blue_hue(&mut w.rng);
        add_item_hued(w, id, "Server.Items.Robe", Some(h));
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(MAGE, MAGE_C, MAGE_V = "Server.Mobiles.Mage", ctor mage_new);

fn mapmaker_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &MAPMAKER, Some("the mapmaker"));
    set_skill_range(w, id, SkillName::Cartography, 90.0, 100.0);
    id
}
pub static MAPMAKER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBMAPMAKER.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(MAPMAKER, MAPMAKER_C, MAPMAKER_V = "Server.Mobiles.Mapmaker", ctor mapmaker_new);

fn miller_new(w: &mut World) -> MobileId {
    new_vendor(w, &MILLER, Some("the miller"))
}
pub static MILLER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBMILLER.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(MILLER, MILLER_C, MILLER_V = "Server.Mobiles.Miller", ctor miller_new);

fn miner_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &MINER, Some("the miner"));
    set_skill_range(w, id, SkillName::Mining, 65.0, 88.0);
    id
}
pub static MINER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBMINER.new_for(w, id)],
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        let h = 0x3E4;
        replace_equipment(w, id, "Server.Items.FancyShirt", Some(h));
        let h = 0x192;
        replace_equipment(w, id, "Server.Items.LongPants", Some(h));
        add_item_hued(w, id, "Server.Items.Pickaxe", None);
        let h = 0x283;
        replace_equipment(w, id, "Server.Items.ThighBoots", Some(h));
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(MINER, MINER_C, MINER_V = "Server.Mobiles.Miner", ctor miner_new);

fn monk_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &MONK, Some("the Monk"));
    set_skill(w, id, SkillName::EvalInt, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 70.0, 90.0);
    set_skill_range(w, id, SkillName::Wrestling, 70.0, 90.0);
    set_skill_range(w, id, SkillName::MagicResist, 70.0, 90.0);
    set_skill_range(w, id, SkillName::Macing, 70.0, 90.0);
    id
}
pub static MONK_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBMONK.new_for(w, id)],
    init_outfit: |w, id| {
        add_item_hued(w, id, "Server.Items.Sandals", None);
        add_item_hued(w, id, "Server.Items.MonkRobe", None);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(MONK, MONK_C, MONK_V = "Server.Mobiles.Monk", ctor monk_new);

fn provisioner_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &PROVISIONER, Some("the provisioner"));
    set_skill_range(w, id, SkillName::Camping, 45.0, 68.0);
    set_skill_range(w, id, SkillName::Tactics, 45.0, 68.0);
    id
}
pub static PROVISIONER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        let mut v = Vec::new();
        v.push(SBPROVISIONER.new_for(w, id));
        if is_tokuno_vendor(w, id) {
            v.push(SBSEHATS.new_for(w, id));
        }
        v
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(PROVISIONER, PROVISIONER_C, PROVISIONER_V = "Server.Mobiles.Provisioner", ctor provisioner_new);

fn rancher_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &RANCHER, Some("the rancher"));
    set_skill_range(w, id, SkillName::AnimalLore, 55.0, 78.0);
    set_skill_range(w, id, SkillName::AnimalTaming, 55.0, 78.0);
    set_skill_range(w, id, SkillName::Herding, 64.0, 100.0);
    set_skill_range(w, id, SkillName::Veterinary, 60.0, 83.0);
    id
}
pub static RANCHER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBRANCHER.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(RANCHER, RANCHER_C, RANCHER_V = "Server.Mobiles.Rancher", ctor rancher_new);

fn ranger_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &RANGER, Some("the ranger"));
    set_skill_range(w, id, SkillName::Camping, 55.0, 78.0);
    set_skill_range(w, id, SkillName::DetectHidden, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Hiding, 45.0, 68.0);
    set_skill_range(w, id, SkillName::Archery, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Tracking, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Veterinary, 60.0, 83.0);
    id
}
pub static RANGER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBRANGER.new_for(w, id)],
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        let h = utility::random_neutral_hue(&mut w.rng);
        replace_equipment(w, id, "Server.Items.Shirt", Some(h));
        let h = utility::random_neutral_hue(&mut w.rng);
        replace_equipment(w, id, "Server.Items.LongPants", Some(h));
        add_item_hued(w, id, "Server.Items.Bow", None);
        let h = utility::random_neutral_hue(&mut w.rng);
        replace_equipment(w, id, "Server.Items.ThighBoots", Some(h));
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(RANGER, RANGER_C, RANGER_V = "Server.Mobiles.Ranger", ctor ranger_new);

fn shipwright_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &SHIPWRIGHT, Some("the shipwright"));
    set_skill_range(w, id, SkillName::Carpentry, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Macing, 36.0, 68.0);
    id
}
pub static SHIPWRIGHT_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBSHIPWRIGHT.new_for(w, id)],
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        add_item_hued(w, id, "Server.Items.SmithHammer", None);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(SHIPWRIGHT, SHIPWRIGHT_C, SHIPWRIGHT_V = "Server.Mobiles.Shipwright", ctor shipwright_new);

fn stone_crafter_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &STONE_CRAFTER, Some("the stone crafter"));
    set_skill_range(w, id, SkillName::Carpentry, 85.0, 100.0);
    id
}
pub static STONE_CRAFTER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        vec![
            SBSTONE_CRAFTER.new_for(w, id),
            SBSTAVES_WEAPON.new_for(w, id),
            SBCARPENTER.new_for(w, id),
            SBWOODEN_SHIELDS.new_for(w, id),
        ]
    },
    npc_guild: |_, _| NpcGuild::TinkersGuild,
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(STONE_CRAFTER, STONE_CRAFTER_C, STONE_CRAFTER_V = "Server.Mobiles.StoneCrafter", ctor stone_crafter_new);

fn tanner_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &TANNER, Some("the tanner"));
    set_skill_range(w, id, SkillName::Tailoring, 36.0, 68.0);
    id
}
pub static TANNER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBTANNER.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(TANNER, TANNER_C, TANNER_V = "Server.Mobiles.Tanner", ctor tanner_new);

fn tavern_keeper_new(w: &mut World) -> MobileId {
    new_vendor(w, &TAVERN_KEEPER, Some("the tavern keeper"))
}
pub static TAVERN_KEEPER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBTAVERN_KEEPER.new_for(w, id)],
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        add_item_hued(w, id, "Server.Items.HalfApron", None);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(TAVERN_KEEPER, TAVERN_KEEPER_C, TAVERN_KEEPER_V = "Server.Mobiles.TavernKeeper", ctor tavern_keeper_new);

fn thief_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &THIEF, Some("the thief"));
    set_skill_range(w, id, SkillName::Camping, 55.0, 78.0);
    set_skill_range(w, id, SkillName::DetectHidden, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Hiding, 45.0, 68.0);
    set_skill_range(w, id, SkillName::Archery, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Tracking, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Veterinary, 60.0, 83.0);
    id
}
pub static THIEF_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBTHIEF.new_for(w, id)],
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        let h = utility::random_neutral_hue(&mut w.rng);
        replace_equipment(w, id, "Server.Items.Shirt", Some(h));
        let h = utility::random_neutral_hue(&mut w.rng);
        replace_equipment(w, id, "Server.Items.LongPants", Some(h));
        add_item_hued(w, id, "Server.Items.Dagger", None);
        let h = utility::random_neutral_hue(&mut w.rng);
        replace_equipment(w, id, "Server.Items.ThighBoots", Some(h));
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(THIEF, THIEF_C, THIEF_V = "Server.Mobiles.Thief", ctor thief_new);

fn tinker_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &TINKER, Some("the tinker"));
    set_skill_range(w, id, SkillName::Lockpicking, 60.0, 83.0);
    set_skill_range(w, id, SkillName::RemoveTrap, 75.0, 98.0);
    set_skill_range(w, id, SkillName::Tinkering, 64.0, 100.0);
    id
}
pub static TINKER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBTINKER.new_for(w, id)],
    npc_guild: |_, _| NpcGuild::TinkersGuild,
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(TINKER, TINKER_C, TINKER_V = "Server.Mobiles.Tinker", ctor tinker_new);

fn variety_dealer_new(w: &mut World) -> MobileId {
    new_vendor(w, &VARIETY_DEALER, Some("the variety dealer"))
}
pub static VARIETY_DEALER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBVARIETY_DEALER.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(VARIETY_DEALER, VARIETY_DEALER_C, VARIETY_DEALER_V = "Server.Mobiles.VarietyDealer", ctor variety_dealer_new);

fn veterinarian_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &VETERINARIAN, Some("the vet"));
    set_skill_range(w, id, SkillName::AnimalLore, 85.0, 100.0);
    set_skill_range(w, id, SkillName::Veterinary, 90.0, 100.0);
    id
}
pub static VETERINARIAN_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBVETERINARIAN.new_for(w, id)],
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(VETERINARIAN, VETERINARIAN_C, VETERINARIAN_V = "Server.Mobiles.Veterinarian", ctor veterinarian_new);

fn waiter_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &WAITER, Some("the waiter"));
    set_skill_range(w, id, SkillName::Discordance, 36.0, 68.0);
    id
}
pub static WAITER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBWAITER.new_for(w, id)],
    init_outfit: |w, id| {
        muo_content::vendor::init_outfit(w, id);
        add_item_hued(w, id, "Server.Items.HalfApron", None);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(WAITER, WAITER_C, WAITER_V = "Server.Mobiles.Waiter", ctor waiter_new);

pub(crate) fn register(types: &mut TypeRegistry) {
    types.register_mobile(&ALCHEMIST, &[]);
    types.register_mobile(&ARCHITECT, &[]);
    types.register_mobile(&ARMORER, &[]);
    types.register_mobile(&BAKER, &[]);
    types.register_mobile(&BARD, &[]);
    types.register_mobile(&BARKEEPER, &[]);
    types.register_mobile(&BEEKEEPER, &[]);
    types.register_mobile(&BOWYER, &["Server.Mobiles.Bower"]);
    types.register_mobile(&BUTCHER, &[]);
    types.register_mobile(&CARPENTER, &[]);
    types.register_mobile(&COBBLER, &[]);
    types.register_mobile(&COOK, &[]);
    types.register_mobile(&FARMER, &[]);
    types.register_mobile(&FISHERMAN, &[]);
    types.register_mobile(&FURTRADER, &[]);
    types.register_mobile(&GLASSBLOWER, &["Server.Mobiles.GargoyleAlchemist"]);
    types.register_mobile(&GOLEM_CRAFTER, &[]);
    types.register_mobile(&HAIR_STYLIST, &[]);
    types.register_mobile(&HERBALIST, &[]);
    types.register_mobile(&INN_KEEPER, &[]);
    types.register_mobile(&JEWELER, &[]);
    types.register_mobile(&LEATHER_WORKER, &[]);
    types.register_mobile(&MAGE, &[]);
    types.register_mobile(&MAPMAKER, &[]);
    types.register_mobile(&MILLER, &[]);
    types.register_mobile(&MINER, &[]);
    types.register_mobile(&MONK, &[]);
    types.register_mobile(&PROVISIONER, &[]);
    types.register_mobile(&RANCHER, &[]);
    types.register_mobile(&RANGER, &[]);
    types.register_mobile(&SHIPWRIGHT, &[]);
    types.register_mobile(&STONE_CRAFTER, &["Server.Mobiles.GargoyleStonecrafter"]);
    types.register_mobile(&TANNER, &[]);
    types.register_mobile(&TAVERN_KEEPER, &[]);
    types.register_mobile(&THIEF, &[]);
    types.register_mobile(&TINKER, &[]);
    types.register_mobile(&VARIETY_DEALER, &[]);
    types.register_mobile(&VETERINARIAN, &[]);
    types.register_mobile(&WAITER, &[]);
}

#[cfg(test)]
pub(crate) fn all() -> Vec<&'static muo_server::mobile::MobileType> {
    vec![
        &ALCHEMIST,
        &ARCHITECT,
        &ARMORER,
        &BAKER,
        &BARD,
        &BARKEEPER,
        &BEEKEEPER,
        &BOWYER,
        &BUTCHER,
        &CARPENTER,
        &COBBLER,
        &COOK,
        &FARMER,
        &FISHERMAN,
        &FURTRADER,
        &GLASSBLOWER,
        &GOLEM_CRAFTER,
        &HAIR_STYLIST,
        &HERBALIST,
        &INN_KEEPER,
        &JEWELER,
        &LEATHER_WORKER,
        &MAGE,
        &MAPMAKER,
        &MILLER,
        &MINER,
        &MONK,
        &PROVISIONER,
        &RANCHER,
        &RANGER,
        &SHIPWRIGHT,
        &STONE_CRAFTER,
        &TANNER,
        &TAVERN_KEEPER,
        &THIEF,
        &TINKER,
        &VARIETY_DEALER,
        &VETERINARIAN,
        &WAITER,
    ]
}
