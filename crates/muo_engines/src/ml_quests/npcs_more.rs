//! The quest givers of Heritage and New Haven skill training on a vendor or monster base
//! (`npcs_gen.rs` has the ones on `BaseCreature`) and `SirHelper`.
//!
//! A class on a ready-made base builds the base with the parent's own constructor, then re-types the mobile to the
//! subclass and replays the subclass's template (`Data/Scripts/classes/mobiles/ml_quests/npcs_more.json`). The base
//! constructors of the monsters call no virtual the subclass overrides, so that builds the same mobile; the speeds
//! are read again for the new class name (they are looked up by the class name). The vendors' templates inherit
//! their parent vendor's (its title and skills), their outfit is the `npcOutfit` prop and their `DefaultName` and
//! `CanShout` are props.

use std::collections::HashSet;

use muo_content::creature::data::cd_mut;
use muo_content::creature::{BASE_CREATURE_C, CreatureType};
use muo_content::vendor::{BASE_VENDOR_C, VendorClass, new_vendor};
use muo_mobiles::npcs::vendors::npc::{TEMPLATE_VENDOR_V, new_template_vendor};
use muo_primitives::TimeSpan;
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

use super::gumps;
use super::npc_support::*;
use super::quester::{self, Quester};

// ---- helpers ---------------------------------------------------------------------------------------------

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

fn shout_tell(w: &mut World, id: MobileId, pm: MobileId, number: i32) {
    quester::tell(w, Quester::Mobile(id), pm, number);
}

// ---- Heritage: classes on monsters -----------------------------------------------------------------------------

macro_rules! monster_sub {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal : $parent:ident / $pclass:ident;
    ) => {
        pub static $class: CreatureType = CreatureType {
            is_invulnerable: |w, id| crate::template_keys::mobile_bool(w, id, "isInvulnerable", $pclass.is_invulnerable),
            default_name: muo_content::templates::creature_props::default_name,
            ..$pclass
        };
        $(#[$m])*
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| {
                let id = new_monster_as(w, &$parent, &$name);
                muo_content::templates::apply_mobile_as(w, id, $full);
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
    MAUL_THE_BEAR, MAUL_THE_BEAR_C = "Server.Engines.MLQuests.Definitions.MaulTheBear" : GRIZZLY_BEAR / GRIZZLY_BEAR_C;
}

monster_sub! {
    /// `Server.Engines.MLQuests.Definitions.Strongroot`.
    STRONGROOT, STRONGROOT_C = "Server.Engines.MLQuests.Definitions.Strongroot" : TREEFELLOW / TREEFELLOW_C;
}

monster_sub! {
    /// `Server.Engines.MLQuests.Definitions.BravehornsMate`.
    BRAVEHORNS_MATE, BRAVEHORNS_MATE_C = "Server.Engines.MLQuests.Definitions.BravehornsMate" : HIND / HIND_C;
}

monster_sub! {
    /// `Server.Engines.MLQuests.Definitions.Huntsman`.
    HUNTSMAN, HUNTSMAN_C = "Server.Engines.MLQuests.Definitions.Huntsman" : CENTAUR / CENTAUR_C;
}

monster_sub! {
    /// `Server.Engines.MLQuests.Definitions.Arielle`.
    ARIELLE, ARIELLE_C = "Server.Engines.MLQuests.Definitions.Arielle" : PIXIE / PIXIE_C;
}

// ---- NewHavenSkillTraining: classes on vendors --------------------------------------------------------------------

macro_rules! vendor_sub {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident, $vclass:ident = $full:literal : $parent:ident / $pclass:ident / $pvclass:ident;
    ) => {
        // the outfit is the `npcOutfit` prop, whatever the parent vendor wears
        static $vclass: VendorClass = VendorClass { init_outfit: TEMPLATE_VENDOR_V.init_outfit, ..$pvclass };
        pub static $class: CreatureType = CreatureType {
            subclass: Some(&$vclass),
            can_shout: |w, id| crate::template_keys::mobile_bool(w, id, "canShout", $pclass.can_shout),
            default_name: muo_content::templates::creature_props::default_name,
            ..$pclass
        };
        $(#[$m])*
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| new_template_vendor(w, &$name)),
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

vendor_sub! {
    /// `Server.Engines.MLQuests.Definitions.Aelorn`.
    AELORN, AELORN_C, AELORN_V = "Server.Engines.MLQuests.Definitions.Aelorn" : KEEPER_OF_CHIVALRY / KEEPER_OF_CHIVALRY_C / KEEPER_OF_CHIVALRY_V;
}

vendor_sub! {
    /// `Server.Engines.MLQuests.Definitions.Robyn`.
    ROBYN, ROBYN_C, ROBYN_V = "Server.Engines.MLQuests.Definitions.Robyn" : BOWYER / BOWYER_C / BOWYER_V;
}

vendor_sub! {
    /// `Server.Engines.MLQuests.Definitions.AmeliaYoungstone`.
    AMELIA_YOUNGSTONE, AMELIA_YOUNGSTONE_C, AMELIA_YOUNGSTONE_V = "Server.Engines.MLQuests.Definitions.AmeliaYoungstone" : TINKER / TINKER_C / TINKER_V;
}

vendor_sub! {
    /// `Server.Engines.MLQuests.Definitions.Mulcivikh`.
    MULCIVIKH, MULCIVIKH_C, MULCIVIKH_V = "Server.Engines.MLQuests.Definitions.Mulcivikh" : MAGE / MAGE_C / MAGE_V;
}

vendor_sub! {
    /// `Server.Engines.MLQuests.Definitions.Morganna`.
    MORGANNA, MORGANNA_C, MORGANNA_V = "Server.Engines.MLQuests.Definitions.Morganna" : MAGE / MAGE_C / MAGE_V;
}

vendor_sub! {
    /// `Server.Engines.MLQuests.Definitions.GeorgeHephaestus`.
    GEORGE_HEPHAESTUS, GEORGE_HEPHAESTUS_C, GEORGE_HEPHAESTUS_V = "Server.Engines.MLQuests.Definitions.GeorgeHephaestus" : BLACKSMITH / BLACKSMITH_C / BLACKSMITH_V;
}

// Ryuichi, Hamato: BaseVendor classes with an SBInfo list of their own

static RYUICHI_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBNINJA.new_for(w, id)],
    get_gender: |_, _| false,
    init_outfit: TEMPLATE_VENDOR_V.init_outfit,
    ..BASE_VENDOR_CLASS_REF
};
use muo_content::vendor::BASE_VENDOR;
use muo_content::vendor::BASE_VENDOR_CLASS as BASE_VENDOR_CLASS_REF;

/// A `BaseVendor` leaf with a `VendorClass` of its own and a `Shout`.
macro_rules! base_vendor_leaf {
    ($(#[$m:meta])* $name:ident, $class:ident = $full:literal, vendor $vclass:ident) => {
        pub static $class: CreatureType = CreatureType {
            subclass: Some(&$vclass),
            can_shout: |w, id| crate::template_keys::mobile_bool(w, id, "canShout", BASE_VENDOR_C.can_shout),
            default_name: muo_content::templates::creature_props::default_name,
            ..BASE_VENDOR_C
        };
        $(#[$m])*
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&BASE_VENDOR),
            construct: Some(|w| new_template_vendor(w, &$name)),
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
    RYUICHI, RYUICHI_C = "Server.Engines.MLQuests.Definitions.Ryuichi", vendor RYUICHI_V
);

static HAMATO_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SBSAMURAI.new_for(w, id)],
    get_gender: |_, _| false,
    init_outfit: TEMPLATE_VENDOR_V.init_outfit,
    ..BASE_VENDOR_CLASS_REF
};

base_vendor_leaf!(
    /// `Server.Engines.MLQuests.Definitions.Hamato`.
    HAMATO, HAMATO_C = "Server.Engines.MLQuests.Definitions.Hamato", vendor HAMATO_V
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
    // `Mage()` through `BaseVendor("the mage")`, then the template (which inherits the Mage's)
    let id = new_vendor(w, &SIR_HELPER, None);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(SirHelperState::default());
    }
    muo_content::templates::apply_mobile(w, id);
    id
}

static SIR_HELPER_V: VendorClass = VendorClass {
    is_active_vendor: |_, _| false,
    get_gender: |_, _| false,
    check_morph: |_, _| {},
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
    default_name: muo_content::templates::creature_props::default_name,
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
