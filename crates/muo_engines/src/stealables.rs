//! Stealable artifacts: the artifacts placed in Doom, the Yomotsu Mines, Fan Dancer's
//! Dojo and the Stygian Abyss that thieves can steal and that respawn; `[GenStealArties` / `[RemoveStealArties`.
//!
//! The obsolete `StealableArtifactsSpawner` item (legacy persistence, deletes itself on load) is
//! [`STEALABLE_ARTIFACTS_SPAWNER`].

use indexmap::IndexMap;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::ItemId;
use muo_server::item::{base as item_base, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::persistence::{GenericPayload, GenericPersistence};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::EntityType;
use tracing::warn;

/// `StealableEntry`.
pub struct Entry {
    pub map: MapId,
    pub loc: Point3D,
    pub min_delay: i32,
    pub max_delay: i32,
    /// `Type.Name`.
    pub ty: &'static str,
    /// `GetLampPostHue()`: a random hue chosen once (the lamp post artifact).
    pub lamp_hue: bool,
}

const fn p(x: i32, y: i32, z: i32) -> Point3D {
    Point3D { x, y, z }
}

/// `StealableArtifacts.Entries`.
pub static ENTRIES: [Entry; 81] = [
    Entry {
        map: MapId::MALAS,
        loc: p(317, 56, -1),
        min_delay: 72,
        max_delay: 108,
        ty: "RockArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(360, 31, 8),
        min_delay: 72,
        max_delay: 108,
        ty: "SkullCandleArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(369, 372, -1),
        min_delay: 72,
        max_delay: 108,
        ty: "BottleArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(378, 372, 0),
        min_delay: 72,
        max_delay: 108,
        ty: "DamagedBooksArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(432, 16, -1),
        min_delay: 144,
        max_delay: 216,
        ty: "StretchedHideArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(489, 9, 0),
        min_delay: 144,
        max_delay: 216,
        ty: "BrazierArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(471, 96, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "LampPostArtifact",
        lamp_hue: true,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(421, 198, 2),
        min_delay: 288,
        max_delay: 432,
        ty: "BooksNorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(431, 189, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "BooksWestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(435, 196, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "BooksFaceDownArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(447, 9, 8),
        min_delay: 1152,
        max_delay: 1728,
        ty: "StuddedLeggingsArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(423, 28, 0),
        min_delay: 1152,
        max_delay: 1728,
        ty: "EggCaseArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(347, 44, 4),
        min_delay: 1152,
        max_delay: 1728,
        ty: "SkinnedGoatArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(497, 57, -1),
        min_delay: 1152,
        max_delay: 1728,
        ty: "GruesomeStandardArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(381, 375, 11),
        min_delay: 1152,
        max_delay: 1728,
        ty: "BloodyWaterArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(489, 369, 2),
        min_delay: 1152,
        max_delay: 1728,
        ty: "TarotCardsArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(497, 369, 5),
        min_delay: 1152,
        max_delay: 1728,
        ty: "BackpackArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(475, 23, 4),
        min_delay: 4608,
        max_delay: 6912,
        ty: "StuddedTunicArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(423, 28, 0),
        min_delay: 4608,
        max_delay: 6912,
        ty: "CocoonArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(354, 36, -1),
        min_delay: 9216,
        max_delay: 13824,
        ty: "SkinnedDeerArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(433, 11, -1),
        min_delay: 18432,
        max_delay: 27648,
        ty: "SaddleArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(403, 31, 4),
        min_delay: 18432,
        max_delay: 27648,
        ty: "LeatherTunicArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(257, 70, -2),
        min_delay: 36864,
        max_delay: 55296,
        ty: "ZyronicClaw",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(354, 176, 7),
        min_delay: 36864,
        max_delay: 55296,
        ty: "TitansHammer",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(369, 389, -1),
        min_delay: 36864,
        max_delay: 55296,
        ty: "BladeOfTheRighteous",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(467, 92, 4),
        min_delay: 36864,
        max_delay: 55296,
        ty: "InquisitorsResolution",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(487, 364, -1),
        min_delay: 147456,
        max_delay: 221184,
        ty: "RuinedPaintingArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(18, 110, -1),
        min_delay: 72,
        max_delay: 108,
        ty: "Basket1Artifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(66, 114, -1),
        min_delay: 72,
        max_delay: 108,
        ty: "Basket2Artifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(63, 12, 11),
        min_delay: 144,
        max_delay: 216,
        ty: "Basket4Artifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(5, 29, -1),
        min_delay: 144,
        max_delay: 216,
        ty: "Basket5NorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(30, 81, 3),
        min_delay: 144,
        max_delay: 216,
        ty: "Basket5WestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(115, 7, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "Urn1Artifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(85, 13, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "Urn2Artifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(110, 53, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "Sculpture1Artifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(108, 37, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "Sculpture2Artifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(121, 14, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "TeapotNorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(121, 115, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "TeapotWestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(84, 40, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "TowerLanternArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(94, 7, -1),
        min_delay: 18432,
        max_delay: 27648,
        ty: "ManStatuetteSouthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(113, 640, -2),
        min_delay: 72,
        max_delay: 108,
        ty: "Basket3NorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(102, 355, -1),
        min_delay: 72,
        max_delay: 108,
        ty: "Basket3WestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(99, 370, -1),
        min_delay: 144,
        max_delay: 216,
        ty: "Basket6Artifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(100, 357, -1),
        min_delay: 144,
        max_delay: 216,
        ty: "ZenRock1Artifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(73, 473, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "FanNorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(99, 372, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "FanWestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(92, 326, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "BowlsVerticalArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(97, 470, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "ZenRock2Artifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(103, 691, -1),
        min_delay: 288,
        max_delay: 432,
        ty: "ZenRock3Artifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(103, 336, 4),
        min_delay: 576,
        max_delay: 864,
        ty: "Painting1NorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(59, 381, 4),
        min_delay: 576,
        max_delay: 864,
        ty: "Painting1WestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(84, 401, 2),
        min_delay: 576,
        max_delay: 864,
        ty: "Painting2NorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(59, 392, 2),
        min_delay: 576,
        max_delay: 864,
        ty: "Painting2WestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(107, 483, -1),
        min_delay: 576,
        max_delay: 864,
        ty: "TripleFanNorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(50, 475, -1),
        min_delay: 576,
        max_delay: 864,
        ty: "TripleFanWestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(107, 460, -1),
        min_delay: 576,
        max_delay: 864,
        ty: "BowlArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(90, 502, -1),
        min_delay: 576,
        max_delay: 864,
        ty: "CupsArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(107, 688, -1),
        min_delay: 576,
        max_delay: 864,
        ty: "BowlsHorizontalArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(112, 676, -1),
        min_delay: 576,
        max_delay: 864,
        ty: "SakeArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(135, 614, -1),
        min_delay: 1152,
        max_delay: 1728,
        ty: "SwordDisplay1NorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(50, 482, -1),
        min_delay: 1152,
        max_delay: 1728,
        ty: "SwordDisplay1WestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(119, 672, -1),
        min_delay: 1152,
        max_delay: 1728,
        ty: "Painting3Artifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(90, 326, -1),
        min_delay: 2304,
        max_delay: 3456,
        ty: "Painting4NorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(99, 354, -1),
        min_delay: 2304,
        max_delay: 3456,
        ty: "Painting4WestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(179, 652, -1),
        min_delay: 2304,
        max_delay: 3456,
        ty: "SwordDisplay2NorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(118, 627, -1),
        min_delay: 2304,
        max_delay: 3456,
        ty: "SwordDisplay2WestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(90, 483, -1),
        min_delay: 4608,
        max_delay: 6912,
        ty: "FlowersArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(71, 562, -1),
        min_delay: 9216,
        max_delay: 13824,
        ty: "DolphinLeftArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(102, 677, -1),
        min_delay: 9216,
        max_delay: 13824,
        ty: "DolphinRightArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(61, 499, 0),
        min_delay: 9216,
        max_delay: 13824,
        ty: "SwordDisplay3SouthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(182, 669, -1),
        min_delay: 9216,
        max_delay: 13824,
        ty: "SwordDisplay3EastArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(162, 647, -1),
        min_delay: 9216,
        max_delay: 13824,
        ty: "SwordDisplay4WestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(124, 624, 0),
        min_delay: 9216,
        max_delay: 13824,
        ty: "Painting5NorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(146, 649, 2),
        min_delay: 9216,
        max_delay: 13824,
        ty: "Painting5WestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(100, 488, -1),
        min_delay: 18432,
        max_delay: 27648,
        ty: "SwordDisplay4NorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(175, 606, 0),
        min_delay: 18432,
        max_delay: 27648,
        ty: "SwordDisplay5NorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(157, 608, -1),
        min_delay: 18432,
        max_delay: 27648,
        ty: "SwordDisplay5WestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(187, 643, 1),
        min_delay: 18432,
        max_delay: 27648,
        ty: "Painting6NorthArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(146, 623, 1),
        min_delay: 18432,
        max_delay: 27648,
        ty: "Painting6WestArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::MALAS,
        loc: p(178, 629, -1),
        min_delay: 18432,
        max_delay: 27648,
        ty: "ManStatuetteEastArtifact",
        lamp_hue: false,
    },
    Entry {
        map: MapId::TER_MUR,
        loc: p(843, 665, 27),
        min_delay: 1152,
        max_delay: 1728,
        ty: "ValkyriesGlaive",
        lamp_hue: false,
    },
];

/// `StealableInstance`.
#[derive(Clone, Debug)]
pub struct Instance {
    pub item: Option<ItemId>,
    pub next_respawn: DateTime,
}

/// The statics of `StealableArtifacts`.
#[derive(Default)]
pub struct StealableState {
    enabled: bool,
    artifacts: Vec<Instance>,
    table: IndexMap<ItemId, usize>,
    respawn_timer: Option<TimerHandle>,
    lamp_hue: Option<i32>,
}

fn st(w: &mut World) -> &mut StealableState {
    w.resources.get_mut::<StealableState>()
}

fn st_ref(w: &World) -> Option<&StealableState> {
    w.resources.get::<StealableState>()
}

/// `GetLampPostHue()`, drawn once the first time it is needed.
fn lamp_post_hue(w: &mut World) -> i32 {
    if let Some(h) = st(w).lamp_hue {
        return h;
    }
    use muo_primitives::utility::{random, random_double};
    let h = if random_double(&mut w.rng) < 0.9 {
        0
    } else {
        const HUES: [i32; 8] = [0x455, 0x47E, 0x482, 0x486, 0x48F, 0x4F2, 0x58C, 0x66C];
        HUES[random(&mut w.rng, HUES.len() as i32) as usize]
    };
    st(w).lamp_hue = Some(h);
    h
}

/// `StealableArtifacts.Configure`.
pub fn configure(w: &mut World) {
    w.persistence
        .register(10, Box::new(GenericPersistence::new(StealablePayload)));
    CommandSystem::register(
        w,
        "GenStealArties",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(gen_steal_arties),
    );
    CommandSystem::register(
        w,
        "RemoveStealArties",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(remove_steal_arties),
    );
    let f = &mut w
        .resources
        .get_mut::<muo_content::skills::hooks::SkillHooks>()
        .fns;
    f.stealable_instance = Some(|w, item| get_stealable_instance(w, item).is_some());
    f.clear_stealable_instance = Some(|w, item| {
        if let Some(i) = get_stealable_instance(w, item) {
            set_item(w, i, None);
        }
    });
}

/// `StealableArtifacts.TypesOfEntries`: the class names.
pub fn types_of_entries() -> Vec<&'static str> {
    ENTRIES.iter().map(|e| e.ty).collect()
}

/// `GetStealableInstance(item)`: the index of its entry.
pub fn get_stealable_instance(w: &World, item: ItemId) -> Option<usize> {
    let s = st_ref(w)?;
    if s.enabled {
        s.table.get(&item).copied()
    } else {
        None
    }
}

fn full_name_of(w: &World, short: &str) -> Option<&'static str> {
    match w.types.find_type_by_name(short, false, false)? {
        EntityType::Item(t) => Some(t.full_name),
        EntityType::Mobile(_) => None,
    }
}

/// `StealableEntry.CreateInstance()`.
fn create_instance(w: &mut World, index: usize) -> Option<ItemId> {
    let e = &ENTRIES[index];
    let hue = if e.lamp_hue { lamp_post_hue(w) } else { 0 };
    let made = full_name_of(w, e.ty).and_then(|f| muo_content::factory::make_item(w, f));
    let Some(item) = made else {
        warn!("Failed to construct stealable artifact: {}", e.ty);
        return None;
    };
    if hue > 0 {
        item_base::set_hue(w, item, hue);
    }
    ops::set_movable(w, item, false);
    ops::move_to_world_map(w, item, e.loc, Some(e.map));
    Some(item)
}

/// `StealableInstance.Item = value`.
fn set_item(w: &mut World, index: usize, value: Option<ItemId>) {
    let Some(old) = st(w).artifacts.get(index).map(|i| i.item) else {
        return;
    };
    if old.is_some() && value.is_none() {
        let e = &ENTRIES[index];
        let delay = muo_primitives::utility::random_min_max(&mut w.rng, e.min_delay, e.max_delay);
        let at = w.now + TimeSpan::from_mins(i64::from(delay));
        st(w).artifacts[index].next_respawn = at;
    }
    let s = st(w);
    if s.enabled {
        if let Some(o) = old {
            s.table.shift_remove(&o);
        }
        if let Some(v) = value {
            s.table.insert(v, index);
        }
    }
    s.artifacts[index].item = value;
}

/// `StealableInstance.CheckRespawn()`.
fn check_respawn_one(w: &mut World, index: usize) {
    if let Some(item) = st(w).artifacts.get(index).and_then(|i| i.item) {
        let gone = match w.item(item) {
            None => true,
            Some(i) => i.deleted() || i.movable() || i.parent.is_some(),
        };
        if gone {
            set_item(w, index, None);
        }
    }
    let now = w.now;
    let (none, due) = {
        let i = &st(w).artifacts[index];
        (i.item.is_none(), now >= i.next_respawn)
    };
    if none && due {
        let made = create_instance(w, index);
        set_item(w, index, made);
    }
}

/// `StealableArtifacts.CheckRespawn()`.
pub fn check_respawn(w: &mut World) {
    for i in 0..st(w).artifacts.len() {
        check_respawn_one(w, i);
    }
}

fn start_timer(w: &mut World) {
    let h = timer::start(w, TimeSpan::ZERO, TimeSpan::from_mins(15), 0, check_respawn);
    st(w).respawn_timer = Some(h);
}

/// `CreateStealableArtifacts()`.
fn create_stealable_artifacts(w: &mut World) {
    let now = w.now;
    let s = st(w);
    s.enabled = true;
    s.artifacts = ENTRIES
        .iter()
        .map(|_| Instance {
            item: None,
            next_respawn: now,
        })
        .collect();
    start_timer(w);
}

/// `RemoveStealableArtifacts()`.
fn remove_stealable_artifacts(w: &mut World) {
    st(w).enabled = false;
    if let Some(mut h) = st(w).respawn_timer.take() {
        h.cancel(w);
    }
    let items: Vec<ItemId> = st(w).artifacts.iter().filter_map(|i| i.item).collect();
    for item in items {
        if w.item(item)
            .is_some_and(|i| !i.deleted() && !i.movable() && i.parent.is_none())
        {
            item_base::delete(w, item);
        }
    }
    st(w).artifacts.clear();
}

fn gen_steal_arties(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if st(w).enabled {
        muo_content::mx::send_message(w, from, "Stealable artifacts spawner already present.");
        return;
    }
    create_stealable_artifacts(w);
    muo_content::mx::send_message(w, from, "Stealable artifacts spawner generated.");
}

fn remove_steal_arties(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if !st(w).enabled {
        muo_content::mx::send_message(w, from, "Stealable artifacts spawner not present.");
        return;
    }
    remove_stealable_artifacts(w);
    muo_content::mx::send_message(w, from, "Stealable artifacts spawner removed.");
}

/// `StealableArtifacts : GenericPersistence("StealableArtifacts", 10)`.
struct StealablePayload;

impl GenericPayload for StealablePayload {
    fn name(&self) -> &str {
        "StealableArtifacts"
    }

    fn serialize(&mut self, w: &World, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(2); // version
        let s = st_ref(w);
        let enabled = s.is_some_and(|s| s.enabled);
        writer.write_bool(enabled);
        if enabled && let Some(s) = s {
            writer.write_encoded_int(s.artifacts.len() as i32);
            for si in &s.artifacts {
                writer.write_item(w, si.item);
                writer.write_anchored_time_ticks(si.next_respawn.ticks());
            }
        }
    }

    fn deserialize(&mut self, w: &mut World, reader: &mut dyn GenericReader) -> muo_io::Result<()> {
        let version = reader.read_encoded_int()?;
        let enabled = version > 0 && reader.read_bool()?;
        if !enabled {
            return Ok(());
        }
        let mut artifacts: Vec<Instance> = Vec::with_capacity(ENTRIES.len());
        let mut table = IndexMap::new();
        let length = reader.read_encoded_int()?.max(0) as usize;
        for i in 0..length {
            let item = reader.read_item(w)?;
            let ticks = if version >= 2 {
                reader.read_anchored_time_ticks()?
            } else {
                reader.read_delta_time_ticks(DateTime::utc_now().ticks())?
            };
            if i < ENTRIES.len() {
                if let Some(it) = item {
                    table.insert(it, i);
                }
                artifacts.push(Instance {
                    item,
                    next_respawn: DateTime::from_ticks(
                        ticks.clamp(0, muo_primitives::datetime::MAX_TICKS),
                    ),
                });
            }
        }
        let now = w.now;
        for _ in length..ENTRIES.len() {
            artifacts.push(Instance {
                item: None,
                next_respawn: now,
            });
        }
        let on = !artifacts.is_empty();
        let s = st(w);
        s.enabled = on;
        if on {
            s.artifacts = artifacts;
            s.table = table;
            start_timer(w);
        }
        Ok(())
    }
}

/// `[Obsolete] StealableArtifactsSpawner : Item`, `[TypeAlias("Server.Items.StealableArtifactsSpawner")]`: only for
/// loading an old save. It reads the artifacts the old way into the persistence and deletes itself.
pub static STEALABLE_ARTIFACTS_SPAWNER: muo_server::item::ItemType = muo_server::item::ItemType {
    full_name: "Server.Engines.Stealables.StealableArtifactsSpawner",
    parent: Some(&muo_server::item::ITEM_BASE),
    construct: None,
    serialize: |_, _, _| {},
    deserialize: |w, id, r| {
        (muo_server::item::ITEM_BASE.deserialize)(w, id, r)?;
        StealablePayload.deserialize(w, r)?; // `_stealableArtifactsPersistence.Deserialize(reader)`
        muo_server::timer::start(
            w,
            muo_primitives::TimeSpan::ZERO,
            muo_primitives::TimeSpan::ZERO,
            1,
            move |w| {
                if let Some(ty) = w.item(id).map(|i| i.ty) {
                    (ty.delete)(w, id);
                }
            },
        );
        Ok(())
    },
    ..muo_server::item::ITEM_BASE
};

pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    types.register_item(
        &STEALABLE_ARTIFACTS_SPAWNER,
        &["Server.Items.StealableArtifactsSpawner"],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_entry_table_is_complete() {
        assert_eq!(ENTRIES.len(), 81);
        assert_eq!(types_of_entries()[0], "RockArtifact");
        let last = ENTRIES.last().unwrap();
        assert_eq!((last.map, last.ty), (MapId::TER_MUR, "ValkyriesGlaive"));
        assert_eq!(ENTRIES.iter().filter(|e| e.lamp_hue).count(), 1);
    }

    #[test]
    fn disabled_system_has_no_instances() {
        let w = World::new();
        assert!(get_stealable_instance(&w, ItemId(muo_primitives::Serial::new(5))).is_none());
    }
}
