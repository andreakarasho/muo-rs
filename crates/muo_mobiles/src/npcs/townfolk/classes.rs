//! `BrideGroom`, `EscortableMage`, `Merchant`, `Messenger`, `Noble`, `Peasant`, `Prisoner` and
//! `SeekerOfAdventure`: every one is `[SerializationGenerator(0, false)]` over `BaseEscortable`. Their constructors
//! and outfits are their templates (`Data/Scripts/classes/mobiles/npcs/escortables.json`).

use muo_content::hooks::rnd;
use muo_content::mx;
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::{AccessLevel, MobileType};
use muo_server::type_registry::TypeRegistry;

use super::{BASE_ESCORTABLE, BASE_ESCORTABLE_C, EscortClass, escort_make, hooks};
use crate::template_keys_npcs as keys;

macro_rules! escortable {
    ($(#[$m:meta])* $name:ident, $class:ident = $full:literal; c { $($cf:ident : $cv:expr,)* } m { $($mf:ident : $mv:expr,)* }) => {
        creature_leaf! {
            $(#[$m])*
            $name, $class = $full : BASE_ESCORTABLE / BASE_ESCORTABLE_C, ctor Some(|w| escort_make(w, &$name)), encoded false;
            c {
                can_teach: keys::can_teach,
                $($cf: $cv,)*
            }
            m {
                // Do not display the title when single-clicking
                click_title: keys::click_title,
                $($mf: $mv,)*
            }
        }
    };
}

escortable!(
    /// `Server.Mobiles.BrideGroom`.
    BRIDE_GROOM, BRIDE_GROOM_C = "Server.Mobiles.BrideGroom"; c {} m {});
escortable!(
    /// `Server.Mobiles.EscortableMage`.
    ESCORTABLE_MAGE, ESCORTABLE_MAGE_C = "Server.Mobiles.EscortableMage"; c {} m {});
escortable!(
    /// `Server.Mobiles.Merchant`.
    MERCHANT, MERCHANT_C = "Server.Mobiles.Merchant"; c {} m {});
escortable!(
    /// `Server.Mobiles.Messenger`.
    MESSENGER, MESSENGER_C = "Server.Mobiles.Messenger"; c {} m {});
escortable!(
    /// `Server.Mobiles.Noble`.
    NOBLE, NOBLE_C = "Server.Mobiles.Noble"; c {} m {});
escortable!(
    /// `Server.Mobiles.Peasant`.
    PEASANT, PEASANT_C = "Server.Mobiles.Peasant"; c {} m {});

// ---- Prisoner (Server.Mobiles.Townfolk.Prisoner) ----

fn prisoner_shout(w: &mut World, id: MobileId, pm: MobileId) {
    // 502261 - HELP! ... 502267 - Go and get some help!
    let n = rnd::from(w, 502261, 7);
    if let Some(f) = hooks(w).and_then(|h| h.tell) {
        f(w, id, pm, n);
    }
}

fn prisoner_on_movement(w: &mut World, id: MobileId, m: MobileId, old: Point3D) {
    (BASE_ESCORTABLE.on_movement)(w, id, m, old);
    let (hidden, level) = (
        w.mobile(m).is_some_and(|x| x.hidden),
        mx::access_level(w, m),
    );
    if w.mobile(id).is_some_and(|x| x.cant_walk)
        && mx::in_range(w, id, m, 1)
        && !mx::in_range_point(w, id, old, 1)
        && (!hidden || level == AccessLevel::Player)
    {
        muo_server::mobile::part2::messages::say_number(w, id, 502268, ""); // Quickly, I beg thee! Unlock my chains! ..
    }
}

escortable!(
    /// `Server.Mobiles.Townfolk.Prisoner`.
    PRISONER, PRISONER_C = "Server.Mobiles.Townfolk.Prisoner";
    c { shout: prisoner_shout, }
    m { on_movement: prisoner_on_movement, }
);

// ---- SeekerOfAdventure ----

const DUNGEONS: [&str; 7] = [
    "Covetous", "Deceit", "Despise", "Destard", "Hythloth",
    "Shame", // Old Code for Pre-ML shards.
    "Wrong",
];
const ML_DESTINATIONS: [&str; 4] = ["Cove", "Serpent's Hold", "Jhelom", "Nujel'm"]; // ML List

fn seeker_destinations(w: &World) -> Vec<&'static str> {
    if w.core_ml() {
        ML_DESTINATIONS.to_vec()
    } else {
        DUNGEONS.to_vec()
    }
}

static SEEKER_E: EscortClass = EscortClass {
    possible_destinations: seeker_destinations,
    ..super::BASE_ESCORT_CLASS
};

escortable!(
    /// `Server.Mobiles.SeekerOfAdventure`.
    SEEKER_OF_ADVENTURE, SEEKER_OF_ADVENTURE_C = "Server.Mobiles.SeekerOfAdventure";
    c { subclass: Some(&SEEKER_E), }
    m {}
);

pub(crate) fn all() -> Vec<&'static MobileType> {
    vec![
        &super::BASE_ESCORTABLE,
        &BRIDE_GROOM,
        &ESCORTABLE_MAGE,
        &MERCHANT,
        &MESSENGER,
        &NOBLE,
        &PEASANT,
        &PRISONER,
        &SEEKER_OF_ADVENTURE,
    ]
}

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in all().into_iter().skip(1) {
        types.register_mobile(t, &[]);
    }
}
