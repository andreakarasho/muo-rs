//! Vendor classes with logic of their own (not tables, see `npc.rs`): the bulk order
//! vendors, outfits that edit the worn items, and the classes with their own members.

use muo_content::hooks::rnd;
use muo_content::player_mobile::{self, NpcGuild};
use muo_content::vendor::{
    BASE_VENDOR_CLASS, ShoeType, VendorClass, add_hued, init_outfit as base_init_outfit,
    is_tokuno_vendor, new_vendor,
};
use muo_primitives::{TimeSpan, utility};
use muo_server::ids::{ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::mobile::part2::items;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

use super::add_item_hued;
use crate::common::*;

// ---- shared pieces ----

/// `item = FindItemOnLayer(layer); if (item != null) item.Hue = Utility.RandomBrightHue();`
pub fn rehue_layer_bright(w: &mut World, id: MobileId, layer: Layer) {
    if let Some(item) = items::find_item_on_layer(w, id, layer) {
        let h = utility::random_bright_hue(&mut w.rng);
        muo_server::item::base::set_hue(w, item, h);
    }
}

/// `AddItem(Utility.Random(4) switch { 0 => new JesterHat(bright), 1 => new Bandana(bright), 2 => new SkullCap(bright), _ => null })`.
pub fn add_gypsy_hat(w: &mut World, id: MobileId) {
    let name = match rnd::below(w, 4) {
        0 => "Server.Items.JesterHat",
        1 => "Server.Items.Bandana",
        2 => "Server.Items.SkullCap",
        _ => return,
    };
    let h = utility::random_bright_hue(&mut w.rng);
    add_hued(w, id, name, h);
}

/// `if (!EquipItem(item)) item.Delete();`
fn equip_or_delete(w: &mut World, id: MobileId, item: ItemId) {
    if !muo_server::mobile::base::equip_item(w, id, item)
        && let Some(t) = w.item(item).map(|i| i.ty)
    {
        (t.delete)(w, item);
    }
}

/// `from.Skills.X.Base`.
pub fn skill_base_ref(w: &World, id: MobileId, skill: SkillName) -> f64 {
    w.mobile(id)
        .and_then(|m| m.skills.as_ref())
        .and_then(|s| s.peek(skill as usize))
        .map_or(0.0, |s| f64::from(s.base) / 10.0)
}

/// `from is PlayerMobile`.
pub fn is_player(w: &World, m: MobileId) -> bool {
    player_mobile::data(w, m).is_some()
}

// ---- bulk order vendors (Blacksmith, Weaponsmith, Tailor, Weaver) ----

struct Bod {
    skill: SkillName,
    /// `SmallXBOD.CreateRandomFor(from)` is the `create_small_bod` hook with this kind (0 smith, 1 tailor).
    kind: i32,
    large: &'static str,
    small: &'static str,
    get: fn(&World, MobileId) -> TimeSpan,
    set: fn(&mut World, MobileId, TimeSpan),
}

static SMITH: Bod = Bod {
    skill: SkillName::Blacksmith,
    kind: 0,
    large: "Server.Engines.BulkOrders.LargeSmithBOD",
    small: "Server.Engines.BulkOrders.SmallSmithBOD",
    get: player_mobile::next_smith_bulk_order,
    set: player_mobile::set_next_smith_bulk_order,
};
static TAILOR: Bod = Bod {
    skill: SkillName::Tailoring,
    kind: 1,
    large: "Server.Engines.BulkOrders.LargeTailorBOD",
    small: "Server.Engines.BulkOrders.SmallTailorBOD",
    get: player_mobile::next_tailor_bulk_order,
    set: player_mobile::set_next_tailor_bulk_order,
};

/// `CreateBulkOrder(from, fromContextMenu)`.
fn create_bulk_order(
    w: &mut World,
    b: &Bod,
    from: MobileId,
    from_context_menu: bool,
) -> Option<ItemId> {
    if is_player(w, from)
        && (b.get)(w, from) == TimeSpan::ZERO
        && (from_context_menu || rnd::double(w) < 0.2)
    {
        let their_skill = muo_server::skills::base(w, from, b.skill);

        if their_skill >= 70.1 {
            (b.set)(w, from, TimeSpan::from_hours(6));
        } else if their_skill >= 50.1 {
            (b.set)(w, from, TimeSpan::from_hours(2));
        } else {
            (b.set)(w, from, TimeSpan::from_hours(1));
        }

        if their_skill >= 70.1 && (their_skill - 40.0) / 300.0 > rnd::double(w) {
            return muo_content::factory::make_item(w, b.large);
        }
        return muo_content::vendor::hooks(w)
            .and_then(|h| h.create_small_bod)
            .and_then(|f| f(w, b.kind, from));
    }
    None
}

macro_rules! bod_fns {
    ($m:ident, $bod:ident) => {
        mod $m {
            use super::*;
            pub fn create(w: &mut World, _v: MobileId, from: MobileId, cm: bool) -> Option<ItemId> {
                create_bulk_order(w, &$bod, from, cm)
            }
            pub fn is_valid(w: &World, _v: MobileId, item: ItemId) -> bool {
                muo_content::factory::item_is(w, item, $bod.small)
                    || muo_content::factory::item_is(w, item, $bod.large)
            }
            pub fn next(w: &mut World, _v: MobileId, from: MobileId) -> TimeSpan {
                if is_player(w, from) {
                    ($bod.get)(w, from)
                } else {
                    TimeSpan::ZERO
                }
            }
            pub fn received(w: &mut World, _v: MobileId, from: MobileId) {
                if w.core_se() && is_player(w, from) {
                    ($bod.set)(w, from, TimeSpan::ZERO);
                }
            }
        }
    };
}
bod_fns!(smith_bod, SMITH);
bod_fns!(tailor_bod, TAILOR);

fn blacksmith_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &BLACKSMITH, Some("the blacksmith"));
    set_skill_range(w, id, SkillName::ArmsLore, 36.0, 68.0);
    set_skill_range(w, id, SkillName::Blacksmith, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Fencing, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Macing, 61.0, 93.0);
    set_skill_range(w, id, SkillName::Swords, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Tactics, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Parry, 61.0, 93.0);
    id
}
pub static BLACKSMITH_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        // The older SBInfo set is not used.
        let mut v = vec![super::sb::named(w, id, "SBBlacksmith")];
        if is_tokuno_vendor(w, id) {
            v.push(super::sb::named(w, id, "SBSEArmor"));
            v.push(super::sb::named(w, id, "SBSEWeapons"));
        }
        v
    },
    npc_guild: |_, _| NpcGuild::BlacksmithsGuild,
    shoe_type: |_, _| ShoeType::None,
    init_outfit: |w, id| {
        base_init_outfit(w, id);
        // Item item = Utility.RandomBool() ? new FullApron() : new RingmailChest();
        let name = if rnd::bool(w) {
            "Server.Items.FullApron"
        } else {
            "Server.Items.RingmailChest"
        };
        if let Some(item) = muo_content::factory::make_item(w, name) {
            equip_or_delete(w, id, item);
        }
        add_item_hued(w, id, "Server.Items.Bascinet", None);
        add_item_hued(w, id, "Server.Items.SmithHammer", None);
    },
    supports_bulk_orders: |w, _v, from| {
        is_player(w, from) && skill_base_ref(w, from, SkillName::Blacksmith) > 0.0
    },
    create_bulk_order: smith_bod::create,
    is_valid_bulk_order: smith_bod::is_valid,
    get_next_bulk_order: smith_bod::next,
    on_successful_bulk_order_receive: smith_bod::received,
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(BLACKSMITH, BLACKSMITH_C, BLACKSMITH_V = "Server.Mobiles.Blacksmith", ctor blacksmith_new);

fn weaponsmith_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &WEAPONSMITH, Some("the weaponsmith"));
    set_skill_range(w, id, SkillName::ArmsLore, 64.0, 100.0);
    set_skill_range(w, id, SkillName::Blacksmith, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Fencing, 45.0, 68.0);
    set_skill_range(w, id, SkillName::Macing, 45.0, 68.0);
    set_skill_range(w, id, SkillName::Swords, 45.0, 68.0);
    set_skill_range(w, id, SkillName::Tactics, 36.0, 68.0);
    id
}
static WEAPONSMITH_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        let mut v = vec![super::sb::named(w, id, "SBWeaponSmith")];
        if is_tokuno_vendor(w, id) {
            v.push(super::sb::named(w, id, "SBSEWeapons"));
        }
        v
    },
    shoe_type: |w, _| {
        if rnd::bool(w) {
            ShoeType::Boots
        } else {
            ShoeType::ThighBoots
        }
    },
    get_shoe_hue: |_, _| 0,
    init_outfit: |w, id| {
        base_init_outfit(w, id);
        add_item_hued(w, id, "Server.Items.HalfApron", None);
    },
    supports_bulk_orders: |w, _v, from| {
        is_player(w, from) && w.core_aos() && skill_base_ref(w, from, SkillName::Blacksmith) > 0.0
    },
    create_bulk_order: smith_bod::create,
    is_valid_bulk_order: smith_bod::is_valid,
    get_next_bulk_order: smith_bod::next,
    on_successful_bulk_order_receive: smith_bod::received,
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(WEAPONSMITH, WEAPONSMITH_C, WEAPONSMITH_V = "Server.Mobiles.Weaponsmith", ctor weaponsmith_new);

fn tailor_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &TAILOR_NPC, Some("the tailor"));
    set_skill_range(w, id, SkillName::Tailoring, 64.0, 100.0);
    id
}
static TAILOR_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![super::sb::named(w, id, "SBTailor")],
    npc_guild: |_, _| NpcGuild::TailorsGuild,
    shoe_type: |w, _| {
        if rnd::bool(w) {
            ShoeType::Sandals
        } else {
            ShoeType::Shoes
        }
    },
    supports_bulk_orders: |w, _v, from| {
        is_player(w, from) && skill_base_ref(w, from, SkillName::Tailoring) > 0.0
    },
    create_bulk_order: tailor_bod::create,
    is_valid_bulk_order: tailor_bod::is_valid,
    get_next_bulk_order: tailor_bod::next,
    on_successful_bulk_order_receive: tailor_bod::received,
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(TAILOR_NPC, TAILOR_C, TAILOR_V = "Server.Mobiles.Tailor", ctor tailor_new);

fn weaver_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &WEAVER, Some("the weaver"));
    set_skill_range(w, id, SkillName::Tailoring, 65.0, 88.0);
    id
}
static WEAVER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![super::sb::named(w, id, "SBWeaver")],
    npc_guild: |_, _| NpcGuild::TailorsGuild,
    shoe_type: |_, _| ShoeType::Sandals,
    supports_bulk_orders: |w, _v, from| {
        is_player(w, from) && skill_base_ref(w, from, SkillName::Tailoring) > 0.0
    },
    create_bulk_order: tailor_bod::create,
    is_valid_bulk_order: tailor_bod::is_valid,
    get_next_bulk_order: tailor_bod::next,
    on_successful_bulk_order_receive: tailor_bod::received,
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(WEAVER, WEAVER_C, WEAVER_V = "Server.Mobiles.Weaver", ctor weaver_new);

// ---- outfits that edit the worn items ----

fn ironworker_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &IRON_WORKER, Some("the iron worker"));
    set_skill_range(w, id, SkillName::ArmsLore, 36.0, 68.0);
    set_skill_range(w, id, SkillName::Blacksmith, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Fencing, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Macing, 61.0, 93.0);
    set_skill_range(w, id, SkillName::Swords, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Tactics, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Parry, 61.0, 93.0);
    id
}
static IRON_WORKER_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        vec![
            super::sb::named(w, id, "SBAxeWeapon"),
            super::sb::named(w, id, "SBKnifeWeapon"),
            super::sb::named(w, id, "SBMaceWeapon"),
            super::sb::named(w, id, "SBSmithTools"),
            super::sb::named(w, id, "SBPoleArmWeapon"),
            super::sb::named(w, id, "SBSpearForkWeapon"),
            super::sb::named(w, id, "SBSwordWeapon"),
            super::sb::named(w, id, "SBMetalShields"),
            super::sb::named(w, id, "SBHelmetArmor"),
            super::sb::named(w, id, "SBPlateArmor"),
            super::sb::named(w, id, "SBChainmailArmor"),
            super::sb::named(w, id, "SBRingmailArmor"),
            super::sb::named(w, id, "SBStuddedArmor"),
            super::sb::named(w, id, "SBLeatherArmor"),
        ]
    },
    shoe_type: |_, _| ShoeType::None,
    init_outfit: |w, id| {
        base_init_outfit(w, id);
        // Item item = Utility.RandomBool() ? null : new RingmailChest();
        let item = if rnd::bool(w) {
            None
        } else {
            muo_content::factory::make_item(w, "Server.Items.RingmailChest")
        };
        if let Some(item) = item
            && !muo_server::mobile::base::equip_item(w, id, item)
        {
            if let Some(t) = w.item(item).map(|i| i.ty) {
                (t.delete)(w, item);
            }
            let h = utility::random_bright_hue(&mut w.rng);
            add_hued(w, id, "Server.Items.FullApron", h);
        }
        add_item_hued(w, id, "Server.Items.Bascinet", None);
        add_item_hued(w, id, "Server.Items.SmithHammer", None);
        for layer in [
            Layer::Pants,
            Layer::OuterLegs,
            Layer::InnerLegs,
            Layer::OuterTorso,
            Layer::InnerTorso,
            Layer::Shirt,
        ] {
            rehue_layer_bright(w, id, layer);
        }
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(IRON_WORKER, IRON_WORKER_C, IRON_WORKER_V = "Server.Mobiles.IronWorker", ctor ironworker_new);

fn vagabond_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &VAGABOND, Some("the vagabond"));
    set_skill_range(w, id, SkillName::ItemID, 60.0, 83.0);
    id
}
static VAGABOND_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![super::sb::named(w, id, "SBTinker"), super::sb::named(w, id, "SBVagabond")],
    // The outfit does not call the base InitOutfit.
    init_outfit: |w, id| {
        let h = utility::random_bright_hue(&mut w.rng);
        add_hued(w, id, "Server.Items.FancyShirt", h);
        let c = muo_content::vendor::class(w, id).unwrap_or(&BASE_VENDOR_CLASS);
        let sh = (c.get_shoe_hue)(w, id);
        add_hued(w, id, "Server.Items.Shoes", sh);
        let rh = (c.get_random_hue)(w, id);
        add_hued(w, id, "Server.Items.LongPants", rh);

        if rnd::bool(w) {
            let h = utility::random_bright_hue(&mut w.rng);
            add_hued(w, id, "Server.Items.Cloak", h);
        }

        let hat = if rnd::bool(w) {
            "Server.Items.SkullCap"
        } else {
            "Server.Items.Bandana"
        };
        let h = utility::random_neutral_hue(&mut w.rng);
        add_hued(w, id, hat, h);

        // Utility.AssignRandomHair(this): the item, then a random hue; Utility.AssignRandomFacialHair(this, HairHue).
        muo_content::vendor::assign_random_hair_random_hue(w, id);
        let hair_hue = w.mobile(id).map_or(0, |m| m.hair_hue);
        muo_content::vendor::assign_random_facial_hair(w, id, hair_hue);

        pack_gold_range(w, id, 100, 200);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(VAGABOND, VAGABOND_C, VAGABOND_V = "Server.Mobiles.Vagabond", ctor vagabond_new);

fn keeper_of_chivalry_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &KEEPER_OF_CHIVALRY, Some("the Keeper of Chivalry"));
    set_skill_range(w, id, SkillName::Fencing, 75.0, 85.0);
    set_skill_range(w, id, SkillName::Macing, 75.0, 85.0);
    set_skill_range(w, id, SkillName::Swords, 75.0, 85.0);
    set_skill(w, id, SkillName::Chivalry, 100.0);
    id
}
pub static KEEPER_OF_CHIVALRY_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![super::sb::named(w, id, "SBKeeperOfChivalry")],
    init_outfit: |w, id| {
        for n in [
            "PlateArms",
            "PlateChest",
            "PlateGloves",
            "StuddedGorget",
            "PlateLegs",
        ] {
            add_item_hued(w, id, &format!("Server.Items.{n}"), None);
        }
        let helm = match rnd::below(w, 4) {
            0 => "PlateHelm",
            1 => "NorseHelm",
            2 => "CloseHelm",
            _ => "Helmet",
        };
        add_item_hued(w, id, &format!("Server.Items.{helm}"), None);
        let top = match rnd::below(w, 3) {
            0 => "BodySash",
            1 => "Doublet",
            _ => "Tunic",
        };
        add_hued(w, id, &format!("Server.Items.{top}"), 0x482);
        add_item_hued(w, id, "Server.Items.Broadsword", None);
        // new MetalKiteShield { Hue = Utility.RandomNondyedHue() }
        if let Some(s) = add_item_hued(w, id, "Server.Items.MetalKiteShield", None) {
            let h = utility::random_nondyed_hue(&mut w.rng);
            muo_server::item::base::set_hue(w, s, h);
        }
        let boots = if rnd::bool(w) {
            "Server.Items.Boots"
        } else {
            "Server.Items.ThighBoots"
        };
        add_item_hued(w, id, boots, None);
        pack_gold_range(w, id, 100, 200);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(KEEPER_OF_CHIVALRY, KEEPER_OF_CHIVALRY_C, KEEPER_OF_CHIVALRY_V = "Server.Mobiles.KeeperOfChivalry", ctor keeper_of_chivalry_new);

fn gypsy_maiden_new(w: &mut World) -> MobileId {
    new_vendor(w, &GYPSY_MAIDEN, Some("the gypsy maiden"))
}
static GYPSY_MAIDEN_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![super::sb::named(w, id, "SBProvisioner")],
    get_gender: |_, _| true,
    init_outfit: |w, id| {
        base_init_outfit(w, id);
        add_gypsy_hat(w, id);
        if rnd::bool(w) {
            let h = utility::random_bright_hue(&mut w.rng);
            add_hued(w, id, "Server.Items.HalfApron", h);
        }
        for layer in [Layer::Pants, Layer::OuterLegs, Layer::InnerLegs] {
            rehue_layer_bright(w, id, layer);
        }
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(GYPSY_MAIDEN, GYPSY_MAIDEN_C, GYPSY_MAIDEN_V = "Server.Mobiles.GypsyMaiden", ctor gypsy_maiden_new);

fn holy_mage_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &HOLY_MAGE, Some("the Holy Mage"));
    set_skill_range(w, id, SkillName::EvalInt, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Inscribe, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Magery, 64.0, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 83.0);
    set_skill_range(w, id, SkillName::MagicResist, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Wrestling, 36.0, 68.0);
    id
}
static HOLY_MAGE_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![super::sb::named(w, id, "SBHolyMage")],
    // The outfit does not call the base InitOutfit.
    init_outfit: |w, id| {
        for n in ["Robe", "ThighBoots", "BlackStaff"] {
            add_hued(w, id, &format!("Server.Items.{n}"), 0x47E);
        }
        let female = w.mobile(id).is_some_and(|m| m.female);
        if female {
            add_hued(w, id, "Server.Items.LeatherGloves", 0x47E);
            add_hued(w, id, "Server.Items.GoldNecklace", 0x47E);
        } else {
            add_hued(w, id, "Server.Items.PlateGloves", 0x47E);
            add_hued(w, id, "Server.Items.PlateGorget", 0x47E);
        }
        // HairItemID = Utility.Random(Female ? 2 : 1) switch { 0 => 0x203C, 1 => 0x203D, _ => HairItemID };
        match rnd::below(w, if female { 2 } else { 1 }) {
            0 => muo_server::mobile::part1::props_b::set_hair_item_id(w, id, 0x203C),
            1 => muo_server::mobile::part1::props_b::set_hair_item_id(w, id, 0x203D),
            _ => {}
        }
        muo_server::mobile::part1::props_b::set_hair_hue(w, id, 0x47E);
        pack_gold_range(w, id, 100, 200);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(HOLY_MAGE, HOLY_MAGE_C, HOLY_MAGE_V = "Server.Mobiles.HolyMage", ctor holy_mage_new);

// ---- Scribe ----

/// `m_NextShush`.
struct NextShush(muo_primitives::DateTime);

fn scribe_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &SCRIBE, Some("the scribe"));
    set_skill_range(w, id, SkillName::EvalInt, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Inscribe, 90.0, 100.0);
    id
}

fn scribe_on_speech(w: &mut World, id: MobileId, e: &mut muo_server::events::SpeechEventArgs) {
    (muo_content::vendor::BASE_VENDOR.on_speech)(w, id, e);
    let next = w
        .mobile(id)
        .and_then(|m| m.ext.get::<NextShush>())
        .map_or(muo_primitives::DateTime::MIN, |n| n.0);
    if !e.handled && next <= w.now && muo_content::mx::in_los(w, id, e.mobile) {
        let d = muo_content::mx::get_direction_to(w, id, e.mobile);
        let female = w.mobile(id).is_some_and(|m| m.female);
        muo_server::mobile::part1::props_b::set_direction(w, id, d);
        muo_content::mx::play_sound(w, id, if female { 0x32F } else { 0x441 });
        muo_server::mobile::part2::messages::public_overhead_message_number(
            w,
            id,
            muo_primitives::MessageType::REGULAR,
            0x3B2,
            1073990, // Shhhh!
            "",
            false,
        );
        let t = w.now + TimeSpan::from_mins(1);
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(NextShush(t));
        }
        e.handled = true;
    }
}

static SCRIBE_V: VendorClass = VendorClass {
    init_sb_info: |w, id| {
        let mut v = vec![super::sb::named(w, id, "SBScribe")];
        if (w.core_se() && is_tokuno_vendor(w, id))
            || (w.core_tol() && muo_content::vendor::is_ter_mur_vendor(w, id))
        {
            v.push(super::sb::named(w, id, "SBSamurai"));
            v.push(super::sb::named(w, id, "SBNinja"));
        }
        v
    },
    npc_guild: |_, _| NpcGuild::MagesGuild,
    shoe_type: |w, _| {
        if rnd::bool(w) {
            ShoeType::Shoes
        } else {
            ShoeType::Sandals
        }
    },
    init_outfit: |w, id| {
        base_init_outfit(w, id);
        let h = utility::random_neutral_hue(&mut w.rng);
        add_hued(w, id, "Server.Items.Robe", h);
    },
    ..BASE_VENDOR_CLASS
};
vendor_leaf!(SCRIBE, SCRIBE_C, SCRIBE_V = "Server.Mobiles.Scribe", ctor scribe_new;
    m {
        handles_on_speech: |w, _id, from| w.mobile(from).is_some_and(|m| m.player),
        on_speech: scribe_on_speech,
    }
);

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in [
        &BLACKSMITH,
        &WEAPONSMITH,
        &TAILOR_NPC,
        &WEAVER,
        &IRON_WORKER,
        &VAGABOND,
        &KEEPER_OF_CHIVALRY,
        &GYPSY_MAIDEN,
        &HOLY_MAGE,
        &SCRIBE,
    ] {
        types.register_mobile(t, &[]);
    }
}

#[cfg(test)]
pub(crate) fn all() -> Vec<&'static muo_server::mobile::MobileType> {
    vec![
        &BLACKSMITH,
        &WEAPONSMITH,
        &TAILOR_NPC,
        &WEAVER,
        &IRON_WORKER,
        &VAGABOND,
        &KEEPER_OF_CHIVALRY,
        &GYPSY_MAIDEN,
        &HOLY_MAGE,
        &SCRIBE,
    ]
}
