//! Townfolk: `BaseEscortable` (and `EscortDestinationInfo`, the three context entries) with its
//! subclasses, and `TownCrier` (`crier.rs`).
//!
//! `BaseEscortable` is `[SerializationGenerator(3, false)]`. The ML quest system pieces it needs (the dynamic quest
//! list, `MLQuestSystem.Tell`, whether a quest can be offered here, the compassion award) are [`EscortableHooks`],
//! set from muo_engines' `special_wiring.rs`.

pub mod classes;
pub mod crier;

use indexmap::IndexMap;
use muo_content::creature::data::cd_mut;
use muo_content::creature::{
    BASE_CREATURE, BASE_CREATURE_C, CreatureType, cd, control, ctype, speeds,
};
use muo_content::hooks::rnd;
use muo_content::{factory, mx, player_mobile};
use muo_io::text::definition::TextDefinition;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, Direction, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, MobileId};
use muo_server::item::container;
use muo_server::map::MapId;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::messages::{self, AffixType};
use muo_server::mobile::{MOBILE_BASE, MobileType};
use muo_server::region::RegionId;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

use crate::common::*;

/// `BaseEscortable.EscortDelay`.
const ESCORT_DELAY: TimeSpan = TimeSpan::from_mins(5);

/// Classic list, used when `!MLQuestSystem.Enabled && !Core.ML`.
pub const TOWN_NAMES: [&str; 12] = [
    "Cove",
    "Britain",
    "Jhelom",
    "Minoc",
    "Ocllo",
    "Trinsic",
    "Vesper",
    "Yew",
    "Skara Brae",
    "Nujel'm",
    "Moonglow",
    "Magincia",
];

/// ML list, pre-ML quest system: used when `!MLQuestSystem.Enabled && Core.ML`.
pub const ML_TOWN_NAMES: [&str; 4] = ["Cove", "Serpent's Hold", "Jhelom", "Nujel'm"];

const QUEST_NS: &str = "Server.Engines.MLQuests.Definitions.";

/// `_mlQuestTypes`: ML quest system general list (the quests have no messages of their own).
const ML_QUEST_TYPES: [&str; 12] = [
    "EscortToYew",
    "EscortToVesper",
    "EscortToTrinsic",
    "EscortToSkaraBrae",
    "EscortToSerpentsHold",
    "EscortToNujelm",
    "EscortToMoonglow",
    "EscortToMinoc",
    "EscortToMagincia",
    "EscortToJhelom",
    "EscortToCove",
    "EscortToBritain",
];

/// `_mlQuestTypesNH`: the New Haven list, with each quest's destination messages and payment message.
const ML_QUEST_TYPES_NH: [(&str, [i32; 4], i32); 11] = [
    (
        "EscortToNHAlchemist",
        [1042767, 1042768, 1042769, 1042824],
        1042811,
    ),
    (
        "EscortToNHBard",
        [1042770, 1042771, 1042772, 1042825],
        1042812,
    ),
    (
        "EscortToNHCarpenter",
        [1042773, 1042774, 1042775, 1042829],
        1042816,
    ),
    (
        "EscortToNHTailor",
        [1042779, 1042780, 1042781, 1042828],
        1042815,
    ),
    (
        "EscortToNHBank",
        [1042782, 1042783, 1042784, 1042832],
        1042819,
    ),
    (
        "EscortToNHWarrior",
        [1042785, 1042786, 1042787, 1042827],
        1042814,
    ),
    (
        "EscortToNHMage",
        [1042788, 1042789, 1042790, 1042833],
        1042820,
    ),
    (
        "EscortToNHMapmaker",
        [1042791, 1042792, 1042793, 1042835],
        1042822,
    ),
    (
        "EscortToNHInn",
        [1042794, 1042795, 1042796, 1042834],
        1042821,
    ),
    (
        "EscortToNHDocks",
        [1042800, 1042801, 1042802, 1042826],
        1042813,
    ),
    (
        "EscortToNHBowyer",
        [1042803, 1042804, 1042805, 1042831],
        1042818,
    ),
];

fn quest_hash(short: &str) -> u64 {
    muo_primitives::hash::compute_hash64(&format!("{QUEST_NS}{short}"))
}

/// The short quest class name whose full name hashes to `hash`.
fn quest_short_name(hash: u64) -> Option<&'static str> {
    ML_QUEST_TYPES
        .iter()
        .copied()
        .chain(ML_QUEST_TYPES_NH.iter().map(|q| q.0))
        .find(|n| quest_hash(n) == hash)
}

type QuestUsableFn = fn(&World, &str, RegionId) -> Option<bool>;
/// What `BaseEscortable` needs from systems above this crate. An unset hook behaves as if the system were absent.
#[derive(Default)]
pub struct EscortableHooks {
    /// `MLQuestSystem.FindQuest(type)` and the check that none of its escort objectives' destination contains the
    /// region: `None` when the quest is not registered, `Some(true)` when it can be given here.
    pub quest_usable: Option<QuestUsableFn>,
    /// `MLQuestSystem.Tell(quester, pm, number)`.
    pub tell: Option<fn(&mut World, MobileId, MobileId, i32)>,
    /// The compassion award of `CheckAtDestination` (`gains`, the 5-per-day limit, the messages).
    pub award_compassion: Option<fn(&mut World, MobileId, bool)>,
}

fn hooks(w: &World) -> Option<&EscortableHooks> {
    w.resources.get::<EscortableHooks>()
}

fn ml_enabled(w: &World) -> bool {
    player_mobile::hooks::ml_quest_enabled(w)
}

/// `AbandonDelay`.
fn abandon_delay(w: &World) -> TimeSpan {
    if ml_enabled(w) {
        TimeSpan::from_mins(1)
    } else {
        TimeSpan::from_mins(2)
    }
}

/// `DeleteTime`.
fn delete_time(w: &World) -> TimeSpan {
    if ml_enabled(w) {
        TimeSpan::from_secs(100)
    } else {
        TimeSpan::from_secs(30)
    }
}

// ---- EscortDestinationInfo ----

/// `EscortDestinationInfo._table` and `BaseEscortable.Initialized`.
#[derive(Default)]
pub struct EscortStatics {
    table: Option<IndexMap<String, RegionId>>,
    pub initialized: bool,
    /// `EscortTable`: escorter to escortable.
    pub escort_table: IndexMap<MobileId, MobileId>,
}

fn statics(w: &mut World) -> &mut EscortStatics {
    w.resources.get_mut::<EscortStatics>()
}

/// `EDI.Initialize`.
pub fn edi_initialize(w: &mut World) {
    use muo_content::regions::simple::{DUNGEON_REGION, TOWN_REGION};
    let Some(map) = w.maps.get(MapId::FELUCCA) else {
        return;
    };
    let list: Vec<RegionId> = map.regions.values().copied().collect();
    if list.is_empty() {
        return;
    }
    let mut table = IndexMap::new();
    for r in list {
        let Some(reg) = w.region(r) else { continue };
        if (reg.ty.is(&DUNGEON_REGION) || reg.ty.is(&TOWN_REGION))
            && let Some(name) = &reg.name
        {
            table.insert(name.clone(), r);
        }
    }
    // Validate that we have at least one valid destination
    let towns: &[&str] = if w.core_ml() {
        &ML_TOWN_NAMES
    } else {
        &TOWN_NAMES
    };
    let valid_town = towns.iter().any(|t| table.contains_key(*t));
    let s = statics(w);
    s.table = Some(table);
    s.initialized = valid_town;
    if !valid_town {
        let var = if w.core_ml() {
            "MlTownNames"
        } else {
            "TownNames"
        };
        tracing::error!("No valid escort destinations found. Please check {var}.");
    }
}

/// `EDI.Find(name)`: the region of a destination.
fn edi_find(w: &World, name: Option<&str>) -> Option<RegionId> {
    let name = name?;
    w.resources
        .get::<EscortStatics>()?
        .table
        .as_ref()?
        .get(name)
        .copied()
}

fn edi_contains(w: &World, region: RegionId, p: Point3D) -> bool {
    muo_server::region::contains(w, region, p)
}

// ---- state ----

/// What `BaseEscortable` adds to a creature.
#[derive(Default)]
pub struct EscortData {
    destination_string: Option<String>,
    /// `_destination`: the EDI's name.
    destination: Option<String>,
    delete_timer: TimerHandle,
    /// `_mlQuestType`: the hash of its full name.
    ml_quest_type: Option<u64>,
    ml_quest_destination_message: Option<TextDefinition>,
    ml_quest_payment_message: Option<TextDefinition>,
    delete_corpse: bool,
    last_seen_escorter: DateTime,
}

pub fn data(w: &World, id: MobileId) -> Option<&EscortData> {
    w.mobile(id)?.ext.get::<EscortData>()
}

fn data_mut(w: &mut World, id: MobileId) -> Option<&mut EscortData> {
    w.mobile_mut(id)?.ext.get_mut::<EscortData>()
}

/// `BaseEscortable.IsBeingDeleted`.
pub fn is_being_deleted(w: &World, id: MobileId) -> bool {
    data(w, id).is_some_and(|d| d.delete_timer != TimerHandle::default())
}

/// `m is BaseEscortable`.
pub fn is_escortable(w: &World, id: MobileId) -> bool {
    data(w, id).is_some()
}

/// `StaticMLQuester` is true for the escortables of the ML quest definitions (Bravehorn, Lissbet): their quest list
/// is the registered one of their class, not one `BaseEscortable` makes.
pub fn is_static_ml_quester(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| {
        matches!(
            m.ty.full_name,
            "Server.Engines.MLQuests.Definitions.Bravehorn"
                | "Server.Engines.MLQuests.Definitions.Lissbet"
        )
    })
}

/// `IsPrisoner`.
pub fn is_prisoner(w: &World, id: MobileId) -> bool {
    cd(w, id).is_some_and(|d| d.is_prisoner)
}

/// The full name of the quest class this creature was given (the ML quest system's `MLQuests` for it).
pub fn ml_quest_full_name(w: &World, id: MobileId) -> Option<String> {
    let hash = data(w, id)?.ml_quest_type?;
    quest_short_name(hash).map(|n| format!("{QUEST_NS}{n}"))
}

/// The virtuals of the escortable subclasses.
pub struct EscortClass {
    /// `InitOutfit()`.
    pub init_outfit: fn(&mut World, MobileId),
    /// `GetPossibleDestinations()`.
    pub possible_destinations: fn(&World) -> Vec<&'static str>,
}

fn eclass(w: &World, id: MobileId) -> &'static EscortClass {
    ctype(w, id)
        .and_then(|c| c.subclass)
        .and_then(|s| s.downcast_ref::<EscortClass>())
        .unwrap_or(&BASE_ESCORT_CLASS)
}

pub fn default_destinations(w: &World) -> Vec<&'static str> {
    if w.core_ml() {
        ML_TOWN_NAMES.to_vec()
    } else {
        TOWN_NAMES.to_vec()
    }
}

/// The classes whose outfit is their template's `npcOutfit` prop.
pub static BASE_ESCORT_CLASS: EscortClass = EscortClass {
    init_outfit: crate::template_keys_npcs::outfit_prop,
    possible_destinations: default_destinations,
};

fn delete_mobile(w: &mut World, id: MobileId) {
    if let Some(t) = w.mobile(id).map(|m| m.ty) {
        (t.delete)(w, id);
    }
}

/// `Timer.DelayCall(delay, Delete)`.
fn start_delete(w: &mut World, id: MobileId, delay: TimeSpan) -> TimerHandle {
    timer::start(w, delay, TimeSpan::ZERO, 1, move |w| delete_mobile(w, id))
}

// ---- construction ----

/// `BaseEscortable()` and the class's own constructor: its template (`BaseEscortable`'s: the creature, body,
/// `npcEscortOutfit`, fame/karma and speed; then the class's).
pub(crate) fn escort_make(w: &mut World, ty: &'static MobileType) -> MobileId {
    let id = muo_content::templates::new_creature(w, ty);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(EscortData::default());
    }
    id
}

/// `InitOutfit()` (the `npcEscortOutfit` template key).
pub(crate) fn init_outfit(w: &mut World, id: MobileId) {
    (eclass(w, id).init_outfit)(w, id);
}

/// `BaseEscortable()` of a subclass that overrides `InitBody()`.
pub fn escort_make_with(
    w: &mut World,
    ty: &'static MobileType,
    init_body: fn(&mut World, MobileId),
) -> MobileId {
    let id = new_creature(w, ty, AIType::AiMelee, FightMode::Aggressor, 22, 1);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(EscortData::default());
    }
    init_body(w, id);
    (eclass(w, id).init_outfit)(w, id);
    fame(w, id, 200);
    karma(w, id, 4000);
    speeds::set_speed(w, id, 0.2, 1.0, true);
    id
}

// ---- ML quest list ----

/// `ConstructQuestList()`: picks the quest type the first time it is needed (the places that read
/// `MLQuests` call this first).
fn ensure_quest(w: &mut World, id: MobileId) {
    if !ml_enabled(w)
        || is_static_ml_quester(w, id)
        || data(w, id).is_none_or(|d| d.ml_quest_type.is_some())
    {
        return;
    }
    let Some(usable) = hooks(w).and_then(|h| h.quest_usable) else {
        return;
    };
    let reg = props_b::region(w, id);
    let haven = muo_server::region::is_part_of_name(w, reg, "Haven Island", false);
    let mut types: Vec<&'static str> = if haven {
        ML_QUEST_TYPES_NH.iter().map(|q| q.0).collect()
    } else {
        ML_QUEST_TYPES.to_vec()
    };
    // Shuffle
    muo_primitives::random::Rng::shuffle(&mut w.rng, &mut types);
    // Sequentially choose a quest until we have a valid one.
    for q in types {
        if usable(w, q, reg) == Some(true) {
            let (dest, payment) = if haven {
                let e = ML_QUEST_TYPES_NH.iter().find(|e| e.0 == q).copied();
                (e.map(|e| e.1), e.map(|e| e.2))
            } else {
                (None, None)
            };
            let dest_msg = match dest {
                Some(list) if !list.is_empty() => {
                    let n = *utility::random_element(&mut w.rng, &list).unwrap_or(&0);
                    TextDefinition::of_number(n)
                }
                _ => TextDefinition::of_number(0),
            };
            if let Some(d) = data_mut(w, id) {
                d.ml_quest_type = Some(quest_hash(q));
                d.ml_quest_destination_message = Some(dest_msg);
                d.ml_quest_payment_message = payment
                    .map(TextDefinition::of_number)
                    .or(Some(TextDefinition::of_number(0)));
            }
            break;
        }
    }
}

fn text_to_say(w: &mut World, id: MobileId, t: &Option<TextDefinition>) -> bool {
    let Some(t) = t else { return false };
    if t.number > 0 {
        messages::say_number(w, id, t.number, "");
        return true;
    }
    if let Some(s) = t.string.as_deref().filter(|s| !s.is_empty()) {
        messages::say_text(w, id, s);
        return true;
    }
    false
}

// ---- destination ----

/// `PickRandomDestination()`.
fn pick_random_destination(w: &mut World, id: MobileId) -> Option<String> {
    let map = mx::map(w, id);
    let fel_empty = w
        .maps
        .get(MapId::FELUCCA)
        .is_none_or(|m| m.regions.is_empty());
    let loc = mx::location(w, id);
    if fel_empty || map.is_none() || map == Some(MapId::INTERNAL) || loc == Point3D::ZERO {
        return None; // Not yet fully initialized
    }
    let possible = (eclass(w, id).possible_destinations)(w);
    // shortcut: draws until a destination that exists and does not contain us comes up, capped so a list with none
    // cannot hang the server.
    for _ in 0..10_000 {
        let picked = *utility::random_element(&mut w.rng, &possible)?;
        if let Some(r) = edi_find(w, Some(picked))
            && !edi_contains(w, r, loc)
        {
            return Some(picked.to_owned());
        }
    }
    None
}

/// `GetDestination()`: the EDI's name and region.
fn get_destination(w: &mut World, id: MobileId) -> Option<(String, RegionId)> {
    if ml_enabled(w)
        || !w
            .resources
            .get::<EscortStatics>()
            .is_some_and(|s| s.initialized)
    {
        return None;
    }
    let (dest_string, no_timer) = data(w, id).map(|d| {
        (
            d.destination_string.clone(),
            d.delete_timer == TimerHandle::default(),
        )
    })?;
    let mut dest_string = dest_string;
    if dest_string.is_none() && no_timer {
        dest_string = pick_random_destination(w, id);
        if let Some(d) = data_mut(w, id) {
            d.destination_string = dest_string.clone();
        }
    }
    if data(w, id).and_then(|d| d.destination.as_deref()) == dest_string.as_deref()
        && let Some(n) = dest_string.clone()
        && let Some(r) = edi_find(w, Some(&n))
    {
        return Some((n, r));
    }
    if w.maps
        .get(MapId::FELUCCA)
        .is_some_and(|m| !m.regions.is_empty())
    {
        let found = edi_find(w, dest_string.as_deref());
        match found {
            None => {
                // The destination string used to exist, but doesn't anymore.
                if let Some(d) = data_mut(w, id) {
                    d.destination = None;
                    d.destination_string = None;
                }
                None
            }
            Some(r) => {
                if let Some(d) = data_mut(w, id) {
                    d.destination = dest_string.clone();
                }
                dest_string.map(|n| (n, r))
            }
        }
    } else {
        // Destination is invalid, so set it to null and return.
        if let Some(d) = data_mut(w, id) {
            d.destination = None;
        }
        None
    }
}

fn dest_display(w: &World, name: &str, m: MobileId) -> String {
    if name == "Ocllo" && mx::map(w, m) == Some(MapId::TRAMMEL) {
        "Haven".to_owned()
    } else {
        name.to_owned()
    }
}

/// `SayDestinationTo(m)`.
pub fn say_destination_to(w: &mut World, id: MobileId, m: MobileId) -> bool {
    let Some((dest, _)) = get_destination(w, id) else {
        return false;
    };
    if !mx::alive(w, m) {
        return false;
    }
    let escorter = get_escorter(w, id);
    match escorter {
        None => {
            let msg = data(w, id).and_then(|d| d.ml_quest_destination_message.clone());
            if text_to_say(w, id, &msg) {
                return true;
            }
            // I'm looking to go somewhere:
            let name = dest_display(w, &dest, m);
            messages::say_affix(w, id, 1042807, AffixType::APPEND, &name, "");
            true
        }
        Some(e) if e == m => {
            let msg = data(w, id).and_then(|d| d.ml_quest_payment_message.clone());
            if text_to_say(w, id, &msg) {
                return true;
            }
            // Lead on! Payment will be made when we arrive at ~1_DESTINATION~!
            let name = dest_display(w, &dest, m);
            messages::say_number(w, id, 1042806, &name);
            true
        }
        _ => false,
    }
}

/// `AcceptEscorter(m)`.
pub fn accept_escorter(w: &mut World, id: MobileId, m: MobileId) -> bool {
    let Some((dest, _)) = get_destination(w, id) else {
        return false;
    };
    if get_escorter(w, id).is_some() || !mx::alive(w, m) {
        return false;
    }
    if let Some(&e) = w
        .resources
        .get::<EscortStatics>()
        .and_then(|s| s.escort_table.get(&m))
        && !mx::deleted(w, e)
        && get_escorter(w, e) == Some(m)
    {
        messages::say_text(w, id, "I see you already have an escort.");
        return false;
    }
    let is_player = player_mobile::data(w, m).is_some();
    if is_player {
        let last = player_mobile::data(w, m).map_or(DateTime::MIN, |d| d.last_escort_time);
        if last + ESCORT_DELAY >= w.now {
            let minutes = ((last + ESCORT_DELAY) - w.now).total_minutes().ceil() as i32;
            let text = if minutes == 1 {
                format!("You must rest {minutes} minute before we set out on this journey.")
            } else {
                format!("You must rest {minutes} minutes before we set out on this journey.")
            };
            messages::say_text(w, id, &text);
            return false;
        }
    }
    if control::set_control_master(w, id, Some(m)) {
        let now = w.now;
        if let Some(d) = data_mut(w, id) {
            d.last_seen_escorter = now;
        }
        if let Some(d) = player_mobile::data_mut(w, m) {
            d.last_escort_time = now;
        }
        let name = dest_display(w, &dest, m);
        messages::say_text(
            w,
            id,
            &format!("Lead on! Payment will be made when we arrive in {name}."),
        );
        statics(w).escort_table.insert(m, id);
        start_follow(w, id);
        return true;
    }
    false
}

fn handles_on_speech(w: &World, id: MobileId, from: MobileId) -> bool {
    !ml_enabled(w)
        && (mx::in_range_point(w, from, mx::location(w, id), 3)
            || (BASE_CREATURE.handles_on_speech)(w, id, from))
}

fn on_speech(w: &mut World, id: MobileId, e: &mut muo_server::events::SpeechEventArgs) {
    (BASE_CREATURE.on_speech)(w, id, e);
    if get_destination(w, id).is_none()
        || e.handled
        || !mx::in_range_point(w, e.mobile, mx::location(w, id), 3)
    {
        return;
    }
    if e.has_keyword(0x1D) {
        // *destination*
        e.handled = say_destination_to(w, id, e.mobile);
    } else if e.has_keyword(0x1E) {
        // *i will take thee*
        e.handled = accept_escorter(w, id, e.mobile);
    }
}

fn on_after_delete(w: &mut World, id: MobileId) {
    let mut t = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.delete_timer))
        .unwrap_or_default();
    t.cancel(w);
    (BASE_CREATURE.on_after_delete)(w, id);
}

fn on_think(w: &mut World, id: MobileId) {
    ensure_quest(w, id);
    (BASE_CREATURE_C.on_think)(w, id);
    check_at_destination(w, id);
}

fn on_move(w: &mut World, id: MobileId, d: Direction) -> bool {
    if !(BASE_CREATURE.on_move)(w, id, d) {
        return false;
    }
    check_at_destination(w, id);
    true
}

fn on_movement(w: &mut World, id: MobileId, m: MobileId, old: Point3D) {
    ensure_quest(w, id);
    (BASE_CREATURE.on_movement)(w, id, m, old);
}

fn on_double_click(w: &mut World, id: MobileId, from: MobileId) {
    ensure_quest(w, id);
    (BASE_CREATURE.on_double_click)(w, id, from);
}

// ---- following ----

/// `StartFollow(escorter)`.
pub fn start_follow_of(w: &mut World, id: MobileId, escorter: Option<MobileId>) {
    let Some(escorter) = escorter else { return };
    control::set_control_order_raw(w, id, OrderType::Follow);
    if let Some(d) = cd_mut(w, id) {
        d.control_target = Some(escorter);
    }
    if is_prisoner(w, id) && w.mobile(id).is_some_and(|m| m.cant_walk) {
        cant_walk(w, id, false);
    }
    speeds::set_speed(w, id, 0.1, 0.2, false);
}

pub fn start_follow(w: &mut World, id: MobileId) {
    let e = get_escorter(w, id);
    start_follow_of(w, id, e);
}

/// `StopFollow()`.
pub fn stop_follow(w: &mut World, id: MobileId) {
    control::set_control_order_raw(w, id, OrderType::None);
    if let Some(d) = cd_mut(w, id) {
        d.control_target = None;
    }
    speeds::set_speed(w, id, 0.2, 1.0, true);
}

/// `GetEscorter()`.
pub fn get_escorter(w: &mut World, id: MobileId) -> Option<MobileId> {
    if !cd(w, id).is_some_and(|d| d.controlled) {
        return None;
    }
    let master = control::control_master(w, id);
    if ml_enabled(w) || master.is_none() {
        return master;
    }
    let master = master?;
    let here = mx::location(w, id);
    if mx::deleted(w, master)
        || mx::map(w, master) != mx::map(w, id)
        || !mx::in_range_point(w, master, here, 30)
        || !mx::alive(w, master)
    {
        stop_follow(w, id);
        let last_seen = data(w, id).map_or(w.now, |d| d.last_seen_escorter);
        if w.now - last_seen >= abandon_delay(w) {
            mx::msg(w, master, 1042473); // You have lost the person you were escorting.
            messages::say_number(w, id, 1005653, ""); // Hmmm. I seem to have lost my master.
            control::set_control_master(w, id, None);
            statics(w).escort_table.shift_remove(&master);
            timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
                delete_mobile(w, id)
            });
            return None;
        }
        control::set_control_order_raw(w, id, OrderType::Stay);
        return Some(master);
    }
    if cd(w, id).map(|d| d.control_order) != Some(OrderType::Follow) {
        start_follow_of(w, id, Some(master));
    }
    let now = w.now;
    if let Some(d) = data_mut(w, id) {
        d.last_seen_escorter = now;
    }
    Some(master)
}

/// `BeginDelete()`.
pub fn begin_delete(w: &mut World, id: MobileId) {
    let delay = delete_time(w);
    let existing = data(w, id)
        .map(|d| d.delete_timer)
        .filter(|t| *t != TimerHandle::default());
    if let Some(mut t) = existing {
        t.cancel(w);
    }
    let h = start_delete(w, id, delay);
    if let Some(d) = data_mut(w, id) {
        d.delete_timer = h;
    }
}

/// `CheckAtDestination()`.
pub fn check_at_destination(w: &mut World, id: MobileId) -> bool {
    if ml_enabled(w) {
        return false;
    }
    let Some((_, region)) = get_destination(w, id) else {
        return false;
    };
    let Some(escorter) = get_escorter(w, id) else {
        return false;
    };
    if !edi_contains(w, region, mx::location(w, id)) {
        return false;
    }
    // We have arrived! I thank thee, ~1_PLAYER_NAME~! I have no further need of thy services. Here is thy pay.
    let name = w
        .mobile(escorter)
        .and_then(|m| (m.ty.name)(w, escorter))
        .unwrap_or_default();
    messages::say_number(w, id, 1042809, &name);
    // not going anywhere
    if let Some(d) = data_mut(w, id) {
        d.destination = None;
        d.destination_string = None;
    }
    let cont = props_b::backpack(w, escorter).or_else(|| props_b::bank_box(w, escorter));
    let amount = rnd::min_max(w, 500, 1000);
    if let Some(gold) = factory::make_item_amount(w, "Server.Items.Gold", amount) {
        let dropped =
            cont.is_some_and(|c| container::try_drop_item(w, c, Some(escorter), gold, false));
        if !dropped {
            muo_server::item::ops::move_to_world_map(
                w,
                gold,
                mx::location(w, escorter),
                mx::map(w, escorter),
            );
        }
    }
    stop_follow(w, id);
    control::set_control_master(w, id, None);
    statics(w).escort_table.shift_remove(&escorter);
    begin_delete(w, id);
    muo_content::misc::titles::award_fame(w, escorter, 10, true);
    if player_mobile::data(w, escorter).is_some()
        && let Some(f) = hooks(w).and_then(|h| h.award_compassion)
    {
        let prisoner = is_prisoner(w, id);
        f(w, escorter, prisoner);
    }
    true
}

fn on_before_death(w: &mut World, id: MobileId) -> bool {
    let controlled = cd(w, id).is_some_and(|d| d.controlled);
    let deleting = is_being_deleted(w, id);
    if let Some(d) = data_mut(w, id) {
        d.delete_corpse = controlled || deleting;
    }
    (BASE_CREATURE.on_before_death)(w, id)
}

fn add_custom_context_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    if mx::alive(w, from) {
        let escorter = get_escorter(w, id);
        if !ml_enabled(w) && get_destination(w, id).is_some() {
            if escorter.is_none() || escorter == Some(from) {
                let mut e = ContextMenuEntry::new(6100, 3);
                e.on_click = Some(std::rc::Rc::new(|w, from, target| {
                    if let EntityId::Mobile(m) = target
                        && is_escortable(w, m)
                    {
                        say_destination_to(w, m, from);
                    }
                }));
                list.push(e);
            }
            if escorter.is_none() {
                let mut e = ContextMenuEntry::new(6101, 3);
                e.on_click = Some(std::rc::Rc::new(|w, from, target| {
                    if let EntityId::Mobile(m) = target
                        && is_escortable(w, m)
                    {
                        accept_escorter(w, m, from);
                    }
                }));
                list.push(e);
            }
        }
        if escorter == Some(from) {
            let mut e = ContextMenuEntry::new(6102, 3);
            e.on_click = Some(std::rc::Rc::new(|w, _from, target| {
                if let EntityId::Mobile(m) = target
                    && is_escortable(w, m)
                {
                    delete_mobile(w, m); // OSI just seems to delete instantly
                }
            }));
            list.push(e);
        }
    }
    (BASE_CREATURE_C.add_custom_context_entries)(w, id, from, list);
}

/// `Shout(pm)`: `MLQuestSystem.Tell(this, pm, Utility.Random(1072301, 3))`.
fn shout(w: &mut World, id: MobileId, pm: MobileId) {
    let n = rnd::from(w, 1072301, 3);
    if let Some(f) = hooks(w).and_then(|h| h.tell) {
        f(w, id, pm, n);
    }
}

// ---- serialization ----

fn serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    (BASE_CREATURE.serialize)(w, id, out);
    out.write_i32(3);
    let Some(d) = data(w, id) else { return };
    out.write_opt_string(d.destination_string.as_deref());
    let next = if d.delete_timer == TimerHandle::default() {
        DateTime::MIN
    } else {
        d.delete_timer.next(w)
    };
    out.write_anchored_time_ticks(next.ticks());
    match d.ml_quest_type {
        None => out.write_u8(0),
        Some(h) => {
            out.write_u8(2);
            out.write_u64(h);
        }
    }
    out.write_text_definition(d.ml_quest_destination_message.as_ref());
    out.write_text_definition(d.ml_quest_payment_message.as_ref());
}

fn read_type_hash(r: &mut dyn GenericReader) -> muo_io::Result<Option<u64>> {
    use muo_io::serialization::TypeRef;
    Ok(match r.read_type()? {
        None => None,
        Some(TypeRef::Hash(h)) => Some(h),
        Some(TypeRef::Name(n)) => Some(muo_primitives::hash::compute_hash64(&n)),
    })
}

fn date(ticks: i64) -> DateTime {
    DateTime::from_ticks(ticks)
}

fn deserialize(w: &mut World, id: MobileId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (BASE_CREATURE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = EscortData::default();
    let mut delete_at: Option<DateTime> = None;
    if version == 2 {
        d.destination_string = r.read_string()?;
        let next = date(r.read_delta_time_ticks(w.now.ticks())?);
        if next != DateTime::MIN {
            delete_at = Some(next);
        }
        d.ml_quest_type = read_type_hash(r)?;
        d.ml_quest_destination_message = r.read_text_definition()?;
        d.ml_quest_payment_message = r.read_text_definition()?;
    } else if version < 2 {
        // NOTE: We cannot EDI.Find here, regions have not yet been loaded :-(
        if r.read_bool()? {
            d.destination_string = r.read_string()?;
        }
        if r.read_bool()? {
            delete_at = Some(date(r.read_delta_time_ticks(w.now.ticks())?));
        }
        // MLQuestSystem.ReadQuestRef: the quest's full type name
        let quest = r.read_string()?;
        if ml_enabled(w)
            && let Some(q) = quest
            && let Some(short) = q.strip_prefix(QUEST_NS)
        {
            let known =
                ML_QUEST_TYPES.contains(&short) || ML_QUEST_TYPES_NH.iter().any(|n| n.0 == short);
            if known {
                d.ml_quest_type = Some(quest_hash(short));
                // This isn't serialized before codegen
                if let Some(e) = ML_QUEST_TYPES_NH.iter().find(|e| e.0 == short) {
                    let n = *utility::random_element(&mut w.rng, &e.1).unwrap_or(&0);
                    d.ml_quest_destination_message = Some(TextDefinition::of_number(n));
                    d.ml_quest_payment_message = Some(TextDefinition::of_number(e.2));
                }
            }
        }
    } else {
        d.destination_string = r.read_string()?;
        let next = date(r.read_anchored_time_ticks()?);
        if next != DateTime::MIN {
            delete_at = Some(next);
        }
        d.ml_quest_type = read_type_hash(r)?;
        d.ml_quest_destination_message = r.read_text_definition()?;
        d.ml_quest_payment_message = r.read_text_definition()?;
    }
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(d);
    }
    if let Some(at) = delete_at {
        let delay = at.since(w.now);
        let h = start_delete(w, id, delay);
        if let Some(d) = data_mut(w, id) {
            d.delete_timer = h;
        }
    }
    Ok(())
}

// ---- BaseEscortable ----

pub static BASE_ESCORTABLE_C: CreatureType = CreatureType {
    can_shout: |w, id| !cd(w, id).is_some_and(|d| d.controlled) && !is_being_deleted(w, id),
    commandable: |_w, _id| false, // Our master cannot boss us around!
    delete_corpse_on_death: |w, id| data(w, id).is_some_and(|d| d.delete_corpse),
    on_think,
    shout,
    add_custom_context_entries,
    ..BASE_CREATURE_C
};

pub static BASE_ESCORTABLE: MobileType = MobileType {
    full_name: "Server.Mobiles.BaseEscortable",
    parent: Some(&BASE_CREATURE),
    construct: Some(|w| escort_make(w, &BASE_ESCORTABLE)),
    family: Some(&BASE_ESCORTABLE_C),
    serialize,
    deserialize,
    handles_on_speech,
    on_speech,
    on_after_delete,
    on_before_death,
    on_move,
    on_movement,
    on_double_click,
    can_be_renamed_by: |w, _id, from| {
        mx::access_level(w, from) >= muo_server::mobile::AccessLevel::GameMaster
    },
    ..BASE_CREATURE
};

pub(crate) fn register(types: &mut TypeRegistry) {
    types.register_mobile(&BASE_ESCORTABLE, &[]);
    types.register_phase(
        Phase::Initialize,
        "EscortDestinationInfo.Initialize",
        DEFAULT_CALL_PRIORITY,
        edi_initialize,
    );
    classes::register(types);
    crier::register(types);
}

#[allow(dead_code)]
fn _unused(w: &mut World, id: MobileId) {
    let _ = (&MOBILE_BASE, props_a::set_title, speeds::set_current_speed);
    let _ = w.mobile(id);
}

/// `BaseEscortable.Destination` getter: `_destination?.Name`.
pub fn destination(w: &World, id: MobileId) -> Option<String> {
    data(w, id)?.destination.clone()
}

/// `Destination = value`: `DestinationString = value; _destination = EDI.Find(value);`.
pub fn set_destination(w: &mut World, id: MobileId, value: Option<String>) {
    let found = edi_find(w, value.as_deref()).is_some();
    if let Some(d) = data_mut(w, id) {
        d.destination = if found { value.clone() } else { None };
        d.destination_string = value;
    }
}
