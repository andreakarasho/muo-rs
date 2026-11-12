//! The quest givers of Heritage and New Haven skill training on a vendor or monster base
//! (`npcs_gen.rs` has the ones on `BaseCreature`) and `SirHelper`.
//!
//! A class on a ready-made base builds the base with the parent's own constructor, then re-types the mobile to the
//! subclass and runs the subclass's statements. The base constructors of the monsters call no virtual the subclass
//! overrides, so that builds the same mobile; the speeds are read again for the new class name (they are looked up
//! by the class name).

use std::collections::HashSet;

use muo_content::creature::data::cd_mut;
use muo_content::creature::{BASE_CREATURE_C, CreatureType};
use muo_content::vendor::{BASE_VENDOR_C, VendorClass, new_vendor};
use muo_primitives::{Body, Direction, TimeSpan};
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

use super::gumps;
use super::npc_support::*;
use super::quester::{self, Quester};

// ---- helpers ---------------------------------------------------------------------------------------------

/// `AI = ai`.
fn set_ai(w: &mut World, id: MobileId, ai: AIType) {
    if let Some(d) = cd_mut(w, id) {
        d.current_ai = ai;
    }
    muo_content::ai::change_ai_type(w, id, ai);
}

/// `FightMode = mode`.
fn set_fight_mode(w: &mut World, id: MobileId, mode: FightMode) {
    if let Some(d) = cd_mut(w, id) {
        d.fight_mode = mode;
    }
}

/// `Tamable = false`.
fn not_tamable(w: &mut World, id: MobileId) {
    if let Some(d) = cd_mut(w, id) {
        d.tamable_raw = false;
    }
}

/// The speed block of `BaseCreature(...)` read for the mobile's class as it is now.
fn refresh_speeds(w: &mut World, id: MobileId) {
    let (active, passive) = muo_content::creature::speeds::speeds_of(w, id);
    let (am, pm) = muo_content::creature::speeds::move_speeds_of(w, id);
    if let Some(d) = cd_mut(w, id) {
        d.active_move_speed = am;
        d.passive_move_speed = pm;
        if active > 0.0 && passive > 0.0 {
            d.active_speed = active;
            d.passive_speed = passive;
            d.current_speed = passive;
        }
    }
}

/// `new Parent()` for a monster, then the mobile is a `ty`.
fn new_monster_as(w: &mut World, parent: &'static MobileType, ty: &'static MobileType) -> MobileId {
    let id = (parent.construct.expect("a constructible parent"))(w);
    if let Some(m) = w.mobile_mut(id) {
        m.ty = ty;
    }
    refresh_speeds(w, id);
    let invulnerable =
        muo_content::creature::ctype(w, id).is_some_and(|c| (c.is_invulnerable)(w, id));
    if invulnerable
        && !muo_server::mobile::part1::props_a::core_aos(w)
        && let Some(m) = w.mobile_mut(id)
    {
        m.name_hue = 0x35;
    }
    id
}

/// `AI = AIType.AI_Vendor; FightMode = FightMode.None;`
fn vendor_ai(w: &mut World, id: MobileId) {
    set_ai(w, id, AIType::AiVendor);
    set_fight_mode(w, id, FightMode::None);
}

fn shout_tell(w: &mut World, id: MobileId, pm: MobileId, number: i32) {
    quester::tell(w, Quester::Mobile(id), pm, number);
}

// ---- Heritage: classes on monsters -----------------------------------------------------------------------------

macro_rules! monster_sub {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident, init $init:expr, name $dname:literal, invulnerable $inv:expr;
    ) => {
        pub static $class: CreatureType = CreatureType {
            is_invulnerable: |_, _| $inv,
            default_name: |_, _| Some($dname),
            ..$pclass
        };
        $(#[$m])*
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| {
                let id = new_monster_as(w, &$parent, &$name);
                let init: fn(&mut World, MobileId) = $init;
                init(w, id);
                id
            }),
            family: Some(&$class),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                r.read_i32()?;
                Ok(())
            },
            ..$parent
        };
    };
}

use muo_mobiles::animals::leaves::{GRIZZLY_BEAR, GRIZZLY_BEAR_C, HIND, HIND_C};
use muo_mobiles::monsters::leaves::{CENTAUR, CENTAUR_C, PIXIE, PIXIE_C, TREEFELLOW, TREEFELLOW_C};

monster_sub! {
    /// `Server.Engines.MLQuests.Definitions.MaulTheBear`.
    MAUL_THE_BEAR, MAUL_THE_BEAR_C = "Server.Engines.MLQuests.Definitions.MaulTheBear" : GRIZZLY_BEAR / GRIZZLY_BEAR_C,
        init |w, id| { vendor_ai(w, id); not_tamable(w, id); }, name "Maul", invulnerable true;
}

monster_sub! {
    /// `Server.Engines.MLQuests.Definitions.Strongroot`.
    STRONGROOT, STRONGROOT_C = "Server.Engines.MLQuests.Definitions.Strongroot" : TREEFELLOW / TREEFELLOW_C,
        init vendor_ai, name "Strongroot", invulnerable true;
}

monster_sub! {
    /// `Server.Engines.MLQuests.Definitions.BravehornsMate`.
    BRAVEHORNS_MATE, BRAVEHORNS_MATE_C = "Server.Engines.MLQuests.Definitions.BravehornsMate" : HIND / HIND_C,
        init not_tamable, name "bravehorn's mate", invulnerable false;
}

monster_sub! {
    /// `Server.Engines.MLQuests.Definitions.Huntsman`.
    HUNTSMAN, HUNTSMAN_C = "Server.Engines.MLQuests.Definitions.Huntsman" : CENTAUR / CENTAUR_C,
        init vendor_ai, name "Huntsman", invulnerable true;
}

monster_sub! {
    /// `Server.Engines.MLQuests.Definitions.Arielle`.
    ARIELLE, ARIELLE_C = "Server.Engines.MLQuests.Definitions.Arielle" : PIXIE / PIXIE_C,
        init vendor_ai, name "Arielle", invulnerable true;
}

// ---- NewHavenSkillTraining: classes on vendors --------------------------------------------------------------------

/// `AddItem(new T(hue))` / `AddItem(new T())`.
fn add(w: &mut World, id: MobileId, name: &str, hue: Option<i32>) {
    add_item_hue(w, id, name, hue);
}

/// `Item item = new T(); item.Hue = hue; AddItem(item);`
fn add_hued(w: &mut World, id: MobileId, name: &str, hue: i32) {
    let item = make_named(w, name);
    set_item_hue(w, item, hue);
    add_item_id(w, id, item);
}

/// The statements every instructor's constructor starts with (`Title`, `Body`, `Hue`, hair, speed, stats).
struct Look {
    title: &'static str,
    body: i32,
    female: bool,
    hue: i32,
    hair: i32,
    hair_hue: i32,
    facial: Option<(i32, i32)>,
}

fn apply_look(w: &mut World, id: MobileId, l: &Look) {
    set_title(w, id, Some(l.title.to_string()));
    set_body(w, id, Body(l.body));
    if l.female {
        set_female(w, id, true);
    }
    set_hue(w, id, l.hue);
    set_hair_item_id(w, id, l.hair);
    set_hair_hue(w, id, l.hair_hue);
    if let Some((item, hue)) = l.facial {
        set_facial_hair_item_id(w, id, item);
        set_facial_hair_hue(w, id, hue);
    }
    set_speed(w, id, 0.5, 2.0, true);
    init_stats(w, id, 100, 100, 25);
}

fn skills(w: &mut World, id: MobileId, list: &[SkillName]) {
    for &s in list {
        set_skill(w, id, s, 120.0);
    }
}

macro_rules! vendor_sub {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident, $vclass:ident = $full:literal : $parent:ident / $pclass:ident / $pvclass:ident,
            ctor $ctor:ident, name $dname:literal, outfit $outfit:expr $(, $vf:ident : $vv:expr)*;
    ) => {
        static $vclass: VendorClass = VendorClass {
            init_outfit: $outfit,
            $($vf: $vv,)*
            ..$pvclass
        };
        pub static $class: CreatureType = CreatureType {
            subclass: Some(&$vclass),
            can_shout: |_, _| true,
            default_name: |_, _| Some($dname),
            ..$pclass
        };
        $(#[$m])*
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some($ctor),
            family: Some(&$class),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                r.read_i32()?;
                Ok(())
            },
            ..$parent
        };
    };
}

use muo_mobiles::npcs::vendors::hand::{
    BLACKSMITH, BLACKSMITH_C, BLACKSMITH_V, KEEPER_OF_CHIVALRY, KEEPER_OF_CHIVALRY_C,
    KEEPER_OF_CHIVALRY_V,
};
use muo_mobiles::npcs::vendors::npc::{
    BOWYER, BOWYER_C, BOWYER_V, MAGE, MAGE_C, MAGE_V, TINKER, TINKER_C, TINKER_V,
};
use muo_mobiles::npcs::vendors::sb::{SBNINJA, SBSAMURAI};

// Aelorn

fn aelorn_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &AELORN, Some("the Keeper of Chivalry"));
    // the Keeper of Chivalry constructor's skills
    set_skill_range(w, id, SkillName::Fencing, 75.0, 85.0);
    set_skill_range(w, id, SkillName::Macing, 75.0, 85.0);
    set_skill_range(w, id, SkillName::Swords, 75.0, 85.0);
    set_skill(w, id, SkillName::Chivalry, 100.0);
    apply_look(
        w,
        id,
        &Look {
            title: "the Chivalry Instructor",
            body: 0x190,
            female: false,
            hue: 0x83EA,
            hair: 0x203C,
            hair_hue: 0x47D,
            facial: Some((0x204D, 0x47D)),
        },
    );
    skills(
        w,
        id,
        &[
            SkillName::Anatomy,
            SkillName::MagicResist,
            SkillName::Tactics,
            SkillName::Swords,
            SkillName::Meditation,
            SkillName::Focus,
            SkillName::Chivalry,
        ],
    );
    id
}

vendor_sub! {
    /// `Server.Engines.MLQuests.Definitions.Aelorn`.
    AELORN, AELORN_C, AELORN_V = "Server.Engines.MLQuests.Definitions.Aelorn" : KEEPER_OF_CHIVALRY / KEEPER_OF_CHIVALRY_C / KEEPER_OF_CHIVALRY_V,
        ctor aelorn_new, name "Aelorn", outfit |w, id| {
            add(w, id, "Backpack", None);
            add(w, id, "VikingSword", None);
            add(w, id, "PlateChest", None);
            add(w, id, "PlateLegs", None);
            add(w, id, "PlateGloves", None);
            add(w, id, "PlateArms", None);
            add(w, id, "PlateGorget", None);
            add(w, id, "OrderShield", None);
        };
}

// Robyn

fn robyn_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &ROBYN, Some("the bowyer"));
    set_skill_range(w, id, SkillName::Fletching, 80.0, 100.0);
    set_skill_range(w, id, SkillName::Archery, 80.0, 100.0);
    apply_look(
        w,
        id,
        &Look {
            title: "the Archery Instructor",
            body: 0x191,
            female: true,
            hue: 0x83EA,
            hair: 0x203C,
            hair_hue: 0x47D,
            facial: None,
        },
    );
    skills(
        w,
        id,
        &[
            SkillName::Anatomy,
            SkillName::Parry,
            SkillName::Fletching,
            SkillName::Healing,
            SkillName::Tactics,
            SkillName::Archery,
            SkillName::Focus,
        ],
    );
    id
}

vendor_sub! {
    /// `Server.Engines.MLQuests.Definitions.Robyn`.
    ROBYN, ROBYN_C, ROBYN_V = "Server.Engines.MLQuests.Definitions.Robyn" : BOWYER / BOWYER_C / BOWYER_V,
        ctor robyn_new, name "Robyn", outfit |w, id| {
            add(w, id, "Backpack", None);
            add(w, id, "Boots", Some(0x592));
            add(w, id, "Cloak", Some(0x592));
            add(w, id, "Bandana", Some(0x592));
            add(w, id, "CompositeBow", None);
            add_hued(w, id, "StuddedLegs", 0x592);
            add_hued(w, id, "StuddedGloves", 0x592);
            add_hued(w, id, "StuddedGorget", 0x592);
            add_hued(w, id, "StuddedChest", 0x592);
            add_hued(w, id, "StuddedArms", 0x592);
        };
}

// Amelia Youngstone

fn amelia_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &AMELIA_YOUNGSTONE, Some("the tinker"));
    set_skill_range(w, id, SkillName::Lockpicking, 60.0, 83.0);
    set_skill_range(w, id, SkillName::RemoveTrap, 75.0, 98.0);
    set_skill_range(w, id, SkillName::Tinkering, 64.0, 100.0);
    apply_look(
        w,
        id,
        &Look {
            title: "the Tinkering Instructor",
            body: 0x191,
            female: true,
            hue: 0x83EA,
            hair: 0x203D,
            hair_hue: 0x46C,
            facial: None,
        },
    );
    skills(
        w,
        id,
        &[
            SkillName::ArmsLore,
            SkillName::Blacksmith,
            SkillName::Magery,
            SkillName::Tactics,
            SkillName::Swords,
            SkillName::Tinkering,
            SkillName::Mining,
        ],
    );
    id
}

vendor_sub! {
    /// `Server.Engines.MLQuests.Definitions.AmeliaYoungstone`.
    AMELIA_YOUNGSTONE, AMELIA_YOUNGSTONE_C, AMELIA_YOUNGSTONE_V = "Server.Engines.MLQuests.Definitions.AmeliaYoungstone" : TINKER / TINKER_C / TINKER_V,
        ctor amelia_new, name "Amelia Youngstone", outfit |w, id| {
            add(w, id, "Backpack", None);
            add(w, id, "Sandals", None);
            add(w, id, "Doublet", None);
            add(w, id, "ShortPants", None);
            add(w, id, "HalfApron", Some(0x8AB));
        };
}

// Mulcivikh, Morganna

fn mage_skills(w: &mut World, id: MobileId) {
    set_skill_range(w, id, SkillName::EvalInt, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Inscribe, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Magery, 64.0, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 60.0, 83.0);
    set_skill_range(w, id, SkillName::MagicResist, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Wrestling, 36.0, 68.0);
}

const NECRO_SKILLS: [SkillName; 6] = [
    SkillName::Magery,
    SkillName::MagicResist,
    SkillName::SpiritSpeak,
    SkillName::Swords,
    SkillName::Meditation,
    SkillName::Necromancy,
];

fn mulcivikh_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &MULCIVIKH, Some("the mage"));
    mage_skills(w, id);
    apply_look(
        w,
        id,
        &Look {
            title: "the Necromancy Instructor",
            body: 0x190,
            female: false,
            hue: 0x83EA,
            hair: 0x203D,
            hair_hue: 0x457,
            facial: None,
        },
    );
    skills(w, id, &NECRO_SKILLS);
    id
}

vendor_sub! {
    /// `Server.Engines.MLQuests.Definitions.Mulcivikh`.
    MULCIVIKH, MULCIVIKH_C, MULCIVIKH_V = "Server.Engines.MLQuests.Definitions.Mulcivikh" : MAGE / MAGE_C / MAGE_V,
        ctor mulcivikh_new, name "Mulcivikh", outfit |w, id| {
            add(w, id, "Backpack", None);
            add(w, id, "Sandals", Some(0x8FD));
            add(w, id, "BoneHelm", None);
            add_hued(w, id, "LeatherLegs", 0x2C3);
            add_hued(w, id, "LeatherGloves", 0x2C3);
            add_hued(w, id, "LeatherGorget", 0x2C3);
            add_hued(w, id, "LeatherChest", 0x2C3);
            add_hued(w, id, "LeatherArms", 0x2C3);
        };
}

fn morganna_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &MORGANNA, Some("the mage"));
    mage_skills(w, id);
    apply_look(
        w,
        id,
        &Look {
            title: "the Spirit Speak Instructor",
            body: 0x191,
            female: true,
            hue: 0x83EA,
            hair: 0x203C,
            hair_hue: 0x455,
            facial: None,
        },
    );
    skills(w, id, &NECRO_SKILLS);
    id
}

vendor_sub! {
    /// `Server.Engines.MLQuests.Definitions.Morganna`.
    MORGANNA, MORGANNA_C, MORGANNA_V = "Server.Engines.MLQuests.Definitions.Morganna" : MAGE / MAGE_C / MAGE_V,
        ctor morganna_new, name "Morganna", outfit |w, id| {
            add(w, id, "Backpack", None);
            add(w, id, "Sandals", None);
            add(w, id, "Robe", Some(0x47D));
            add(w, id, "SkullCap", Some(0x455));
        };
}

// George Hephaestus

fn george_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &GEORGE_HEPHAESTUS, Some("the blacksmith"));
    set_skill_range(w, id, SkillName::ArmsLore, 36.0, 68.0);
    set_skill_range(w, id, SkillName::Blacksmith, 65.0, 88.0);
    set_skill_range(w, id, SkillName::Fencing, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Macing, 61.0, 93.0);
    set_skill_range(w, id, SkillName::Swords, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Tactics, 60.0, 83.0);
    set_skill_range(w, id, SkillName::Parry, 61.0, 93.0);
    apply_look(
        w,
        id,
        &Look {
            title: "the Blacksmith Instructor",
            body: 0x190,
            female: false,
            hue: 0x83EA,
            hair: 0x203B,
            hair_hue: 0x47B,
            facial: None,
        },
    );
    skills(
        w,
        id,
        &[
            SkillName::ArmsLore,
            SkillName::Blacksmith,
            SkillName::Magery,
            SkillName::Tactics,
            SkillName::Tinkering,
            SkillName::Swords,
            SkillName::Mining,
        ],
    );
    id
}

vendor_sub! {
    /// `Server.Engines.MLQuests.Definitions.GeorgeHephaestus`.
    GEORGE_HEPHAESTUS, GEORGE_HEPHAESTUS_C, GEORGE_HEPHAESTUS_V = "Server.Engines.MLQuests.Definitions.GeorgeHephaestus" : BLACKSMITH / BLACKSMITH_C / BLACKSMITH_V,
        ctor george_new, name "George Hephaestus", outfit |w, id| {
            add(w, id, "Backpack", None);
            add(w, id, "Boots", Some(0x973));
            add(w, id, "LongPants", None);
            add(w, id, "Bascinet", None);
            add(w, id, "FullApron", Some(0x8AB));
            add_item_props(w, id, "SmithHammer", None, Some(0x8AB));
        };
}

// Ryuichi, Hamato: BaseVendor classes with an SBInfo list of their own

fn ryuichi_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &RYUICHI, Some("the Ninjitsu Instructor"));
    set_hue(w, id, 0x8403);
    skills(
        w,
        id,
        &[
            SkillName::Hiding,
            SkillName::Tactics,
            SkillName::Tracking,
            SkillName::Fencing,
            SkillName::Stealth,
            SkillName::Ninjitsu,
        ],
    );
    id
}

static RYUICHI_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBNINJA.new_for(w, id)],
    get_gender: |_, _| false,
    init_outfit: |w, id| {
        set_hair_item_id(w, id, 0x203B);
        set_hair_hue(w, id, 0x455);
        for n in [
            "SamuraiTabi",
            "LeatherNinjaPants",
            "LeatherNinjaMitts",
            "LeatherNinjaHood",
            "LeatherNinjaJacket",
            "LeatherNinjaBelt",
        ] {
            add(w, id, n, None);
        }
        muo_content::creature::setters::pack_gold_range(w, id, 100, 200);
    },
    ..BASE_VENDOR_CLASS_REF
};
use muo_content::vendor::BASE_VENDOR;
use muo_content::vendor::BASE_VENDOR_CLASS as BASE_VENDOR_CLASS_REF;

/// A `BaseVendor` leaf with a `VendorClass` of its own and a `Shout`.
macro_rules! base_vendor_leaf {
    ($(#[$m:meta])* $name:ident, $class:ident = $full:literal, ctor $ctor:ident, vendor $vclass:ident, name $dname:literal) => {
        pub static $class: CreatureType = CreatureType {
            subclass: Some(&$vclass),
            can_shout: |_, _| true,
            default_name: |_, _| Some($dname),
            ..BASE_VENDOR_C
        };
        $(#[$m])*
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&BASE_VENDOR),
            construct: Some($ctor),
            family: Some(&$class),
            serialize: |w, id, out| {
                (BASE_VENDOR.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BASE_VENDOR.deserialize)(w, id, r)?;
                r.read_i32()?;
                Ok(())
            },
            ..BASE_VENDOR
        };
    };
}

base_vendor_leaf!(
    /// `Server.Engines.MLQuests.Definitions.Ryuichi`.
    RYUICHI, RYUICHI_C = "Server.Engines.MLQuests.Definitions.Ryuichi", ctor ryuichi_new, vendor RYUICHI_V, name "Ryuichi"
);

fn hamato_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &HAMATO, Some("the Bushido Instructor"));
    set_hue(w, id, 0x8403);
    skills(
        w,
        id,
        &[
            SkillName::Anatomy,
            SkillName::Parry,
            SkillName::Healing,
            SkillName::Tactics,
            SkillName::Swords,
            SkillName::Bushido,
        ],
    );
    id
}

static HAMATO_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBSAMURAI.new_for(w, id)],
    get_gender: |_, _| false,
    init_outfit: |w, id| {
        set_hair_item_id(w, id, 0x203D);
        set_hair_hue(w, id, 0x497);
        for n in [
            "Backpack",
            "NoDachi",
            "NinjaTabi",
            "PlateSuneate",
            "LightPlateJingasa",
            "LeatherDo",
            "LeatherHiroSode",
        ] {
            add(w, id, n, None);
        }
        muo_content::creature::setters::pack_gold_range(w, id, 100, 200);
    },
    ..BASE_VENDOR_CLASS_REF
};

base_vendor_leaf!(
    /// `Server.Engines.MLQuests.Definitions.Hamato`.
    HAMATO, HAMATO_C = "Server.Engines.MLQuests.Definitions.Hamato", ctor hamato_new, vendor HAMATO_V, name "Hamato"
);

// ---- SirHelper --------------------------------------------------------------------------------------------------

/// `m_NextShout`, which is not saved.
#[derive(Default)]
struct SirHelperState {
    next_shout: muo_primitives::DateTime,
}

/// `from.BeginAction(this)` keys: `(player, Sir Helper)`.
#[derive(Default)]
struct HelperLocks(HashSet<(MobileId, MobileId)>);

fn sir_helper_new(w: &mut World) -> MobileId {
    // `Mage()` through `BaseVendor("the mage")`, as `new_vendor` does for the class `SirHelper`
    let id = new_vendor(w, &SIR_HELPER, Some("the mage"));
    mage_skills(w, id);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(SirHelperState::default());
    }
    set_title(w, id, Some("the Profession Guide".to_string()));
    set_hue(w, id, 0x83EA);
    muo_server::mobile::part2::effects::set_direction(w, id, Direction::SOUTH);
    set_frozen(w, id, true);
    id
}

static SIR_HELPER_V: VendorClass = VendorClass {
    is_active_vendor: |_, _| false,
    init_sb_info: |_, _| Vec::new(),
    get_gender: |_, _| false,
    check_morph: |_, _| {},
    init_outfit: |w, id| {
        set_hair_item_id(w, id, 0x203C);
        set_facial_hair_item_id(w, id, 0x204D);
        set_hair_hue(w, id, 0x8A7);
        set_facial_hair_hue(w, id, 0x8A7);

        add(w, id, "Sandals", None);
        add_item_props(w, id, "Cloak", Some(0x26AD), Some(0x455));
        add_item_props(w, id, "Robe", Some(0x26AE), Some(0x4AB));
        let pack = make_named(w, "Backpack");
        if let Some(p) = pack {
            muo_server::item::ops::set_movable(w, p, false);
        }
        add_item_id(w, id, pack);
    },
    ..MAGE_V
};

/// `OnThink()`.
fn sir_helper_think(w: &mut World, id: MobileId) {
    (BASE_CREATURE_C.on_think)(w, id);

    let next = w
        .mobile(id)
        .and_then(|m| m.ext.get::<SirHelperState>())
        .map_or(muo_primitives::DateTime::MIN, |s| s.next_shout);
    if next > w.now {
        return;
    }

    let Some((loc, map, serial, body, name)) = w.mobile(id).and_then(|m| {
        Some((
            m.location,
            m.map?,
            m.serial,
            muo_server::mobile::part1::props_b::body(w, id),
            (m.ty.name)(w, id).unwrap_or_default(),
        ))
    }) else {
        return;
    };
    for (ns, m) in muo_server::item::deps::clients_in_range(w, map, loc, 12) {
        let can_see = w
            .mobile(m)
            .map(|x| x.ty)
            .is_some_and(|x| (x.can_see_mobile)(w, m, id));
        if can_see && muo_content::mx::in_los(w, m, id) && !locked(w, m, id) {
            // Double Click On Me For Help!
            muo_server::packets::message::send_message_localized(
                w,
                ns,
                serial,
                body.0,
                muo_server::packets::message::MessageType::REGULAR,
                946,
                3,
                1078099,
                &name,
                "",
            );
        }
    }
    let now = w.now;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(SirHelperState {
            next_shout: now + TimeSpan::from_secs(20),
        });
    }
}

fn locked(w: &World, player: MobileId, helper: MobileId) -> bool {
    w.resources
        .get::<HelperLocks>()
        .is_some_and(|l| l.0.contains(&(player, helper)))
}

/// `OnDoubleClick(from)`.
fn sir_helper_double_click(w: &mut World, id: MobileId, from: MobileId) {
    // `from.BeginAction(this)` then, a day later, `EndLock(from)`
    if w.resources.get_mut::<HelperLocks>().0.insert((from, id)) {
        muo_server::timer::start(w, TimeSpan::from_days(1), TimeSpan::ZERO, 1, move |w| {
            w.resources.get_mut::<HelperLocks>().0.remove(&(from, id));
        });
    }

    quester::turn_to_face(w, Quester::Mobile(id), from);
    gumps::display_info_npc(
        w,
        from,
        muo_io::text::TextDefinition::of_number(1078029),
        muo_io::text::TextDefinition::of_number(1078028),
    );
    // Paperdoll doesn't open
}

pub static SIR_HELPER_C: CreatureType = CreatureType {
    subclass: Some(&SIR_HELPER_V),
    on_think: sir_helper_think,
    default_name: |_, _| Some("Sir Helper"),
    ..MAGE_C
};

/// `Server.Engines.MLQuests.Mobiles.SirHelper`.
pub static SIR_HELPER: MobileType = MobileType {
    full_name: "Server.Engines.MLQuests.Mobiles.SirHelper",
    parent: Some(&MAGE),
    construct: Some(sir_helper_new),
    family: Some(&SIR_HELPER_C),
    on_double_click: sir_helper_double_click,
    serialize: |w, id, out| {
        (MAGE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (MAGE.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        // [AfterDeserialization]: Frozen = true
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(SirHelperState::default());
        }
        set_frozen(w, id, true);
        Ok(())
    },
    ..MAGE
};

// ---- Shout overrides ------------------------------------------------------------------------------------------------

fn aelorn_shout(w: &mut World, id: MobileId, pm: MobileId) {
    shout_tell(w, id, pm, 1078133); // Hail, friend. Want to live the life of a paladin?
}
fn robyn_shout(w: &mut World, id: MobileId, pm: MobileId) {
    shout_tell(w, id, pm, 1078202); // Archery requires a steady aim and dexterous fingers.
}
fn amelia_shout(w: &mut World, id: MobileId, pm: MobileId) {
    shout_tell(w, id, pm, 1078123); // Tinkering is very useful for a blacksmith. You can make your own tools.
}
fn ryuichi_shout(w: &mut World, id: MobileId, pm: MobileId) {
    shout_tell(w, id, pm, 1078155); // I can teach you Ninjitsu. The Art of Stealth.
}
fn hamato_shout(w: &mut World, id: MobileId, pm: MobileId) {
    shout_tell(w, id, pm, 1078134); // Seek me to learn the way of the samurai.
}
fn mulcivikh_shout(w: &mut World, id: MobileId, pm: MobileId) {
    shout_tell(w, id, pm, 1078131); // Allured by dark magic, aren't you?
}
fn morganna_shout(w: &mut World, id: MobileId, pm: MobileId) {
    shout_tell(w, id, pm, 1078132); // Want to learn how to channel the supernatural?
}
fn george_shout(w: &mut World, id: MobileId, pm: MobileId) {
    shout_tell(w, id, pm, 1078122); // Wanna learn how to make powerful weapons and armor? Talk to me.
}

/// `Shout(pm)` overrides: class full name -> function.
#[allow(clippy::type_complexity)] // the callback type, spelled out
pub static SHOUTS: &[(&str, fn(&mut World, MobileId, MobileId))] = &[
    ("Server.Engines.MLQuests.Definitions.Aelorn", aelorn_shout),
    ("Server.Engines.MLQuests.Definitions.Robyn", robyn_shout),
    (
        "Server.Engines.MLQuests.Definitions.AmeliaYoungstone",
        amelia_shout,
    ),
    ("Server.Engines.MLQuests.Definitions.Ryuichi", ryuichi_shout),
    ("Server.Engines.MLQuests.Definitions.Hamato", hamato_shout),
    (
        "Server.Engines.MLQuests.Definitions.Mulcivikh",
        mulcivikh_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.Morganna",
        morganna_shout,
    ),
    (
        "Server.Engines.MLQuests.Definitions.GeorgeHephaestus",
        george_shout,
    ),
];

/// Registers the mobile classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &MAUL_THE_BEAR,
        &STRONGROOT,
        &BRAVEHORNS_MATE,
        &HUNTSMAN,
        &ARIELLE,
        &AELORN,
        &ROBYN,
        &AMELIA_YOUNGSTONE,
        &MULCIVIKH,
        &MORGANNA,
        &GEORGE_HEPHAESTUS,
        &RYUICHI,
        &HAMATO,
        &SIR_HELPER,
    ] {
        types.register_mobile(t, &[]);
    }
}
