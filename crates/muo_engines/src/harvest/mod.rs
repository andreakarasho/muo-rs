//! The harvest engine (HarvestSystem, HarvestDefinition, HarvestBank, HarvestVein, HarvestResource,
//! BonusHarvestResource, HarvestTarget, HarvestTimer, HarvestSoundTimer) and the three systems: `Mining`,
//! `Lumberjacking`, `Fishing`.
//!
//! A system is data ([`HarvestSystem`]) built once; its virtuals are the `match`es on [`Kind`] in this file and the
//! per-system modules. A resource, a vein and a bank are referred to by index into their definition.
//!
//! Fishing's shipwreck chests (SOS, ShipwreckedItem, SpecialFishingNet, TreasureMapChest.Fill), the Collector quest
//! pearl, the Witch Apprentice grave and the TreasureMap dig of `HarvestTarget` plug in through [`HarvestHooks`].

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexMap;
use muo_content::items::tool as tool_mod;
use muo_content::mx;
use muo_io::text::TextDefinition as Td;
use muo_primitives::{DateTime, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{deps, ops};
use muo_server::map::MapId;
use muo_server::skills::{self, SkillName};
use muo_server::target::{self, StaticTarget, Target, TargetFlags, Targeted};
use muo_server::type_registry::{Phase, TypeRegistry};

pub mod axe;
pub mod fishing;
pub mod lumberjacking;
pub mod mining;

/// Which system a [`HarvestSystem`] is (its virtuals).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Kind {
    Mining,
    Lumberjacking,
    Fishing,
}

/// `HarvestResource`.
#[derive(Clone, Debug)]
pub struct HarvestResource {
    pub req_skill: f64,
    pub min_skill: f64,
    pub max_skill: f64,
    pub types: Vec<&'static str>,
    pub success_message: Td,
}

impl HarvestResource {
    pub fn new(
        req: f64,
        min: f64,
        max: f64,
        message: i32,
        types: &[&'static str],
    ) -> HarvestResource {
        HarvestResource {
            req_skill: req,
            min_skill: min,
            max_skill: max,
            types: types.to_vec(),
            success_message: Td::of_number(message),
        }
    }
}

/// `HarvestVein`: resources by index into [`HarvestDefinition::resources`].
#[derive(Copy, Clone, Debug)]
pub struct HarvestVein {
    pub vein_chance: u32,
    pub chance_to_fallback: f64,
    pub primary: usize,
    pub fallback: Option<usize>,
}

/// `BonusHarvestResource`.
#[derive(Clone, Debug)]
pub struct BonusResource {
    pub req_skill: f64,
    pub chance: f64,
    pub message: Td,
    pub ty: Option<&'static str>,
}

/// `HarvestDefinition`.
#[derive(Clone, Debug)]
pub struct HarvestDefinition {
    pub bank_width: i32,
    pub bank_height: i32,
    pub min_total: i32,
    pub max_total: i32,
    pub land_tiles: Vec<i32>,
    pub static_tiles: Vec<i32>,
    pub ranged_tiles: bool,
    pub min_respawn: TimeSpan,
    pub max_respawn: TimeSpan,
    pub max_range: i32,
    pub consumed_per_harvest: i32,
    pub consumed_per_felucca_harvest: i32,
    pub place_at_feet_if_full: bool,
    pub skill: SkillName,
    pub effect_actions: Vec<i32>,
    pub effect_counts: Vec<i32>,
    pub effect_sounds: Vec<i32>,
    pub effect_sound_delay: TimeSpan,
    pub effect_delay: TimeSpan,
    pub no_resources_message: Td,
    pub out_of_range_message: Td,
    pub timed_out_of_range_message: Td,
    pub double_harvest_message: Td,
    pub fail_message: Td,
    pub pack_full_message: Td,
    pub tool_broke_message: Td,
    pub resources: Vec<HarvestResource>,
    pub veins: Vec<HarvestVein>,
    pub bonus_resources: Option<Vec<BonusResource>>,
    pub race_bonus: bool,
    pub randomize_veins: bool,
    pub vein_weights: u32,
}

impl Default for HarvestDefinition {
    fn default() -> Self {
        HarvestDefinition {
            bank_width: 0,
            bank_height: 0,
            min_total: 0,
            max_total: 0,
            land_tiles: Vec::new(),
            static_tiles: Vec::new(),
            ranged_tiles: false,
            min_respawn: TimeSpan::ZERO,
            max_respawn: TimeSpan::ZERO,
            max_range: 0,
            consumed_per_harvest: 0,
            consumed_per_felucca_harvest: 0,
            place_at_feet_if_full: false,
            skill: SkillName::Alchemy,
            effect_actions: Vec::new(),
            effect_counts: Vec::new(),
            effect_sounds: Vec::new(),
            effect_sound_delay: TimeSpan::ZERO,
            effect_delay: TimeSpan::ZERO,
            no_resources_message: Td::EMPTY,
            out_of_range_message: Td::EMPTY,
            timed_out_of_range_message: Td::EMPTY,
            double_harvest_message: Td::EMPTY,
            fail_message: Td::EMPTY,
            pack_full_message: Td::EMPTY,
            tool_broke_message: Td::EMPTY,
            resources: Vec::new(),
            veins: Vec::new(),
            bonus_resources: None,
            race_bonus: false,
            randomize_veins: false,
            vein_weights: 0,
        }
    }
}

impl HarvestDefinition {
    /// `Veins = value`: the weights add up.
    pub fn set_veins(&mut self, veins: Vec<HarvestVein>) {
        self.vein_weights = veins.iter().map(|v| v.vein_chance).sum();
        self.veins = veins;
    }

    /// `Validate(tileID, isLand)`.
    pub fn validate(&self, tile_id: i32, is_land: bool) -> bool {
        let tiles = if is_land {
            &self.land_tiles
        } else {
            &self.static_tiles
        };
        if self.ranged_tiles {
            return tiles
                .chunks(2)
                .any(|p| p.len() == 2 && tile_id >= p[0] && tile_id <= p[1]);
        }
        tiles.contains(&tile_id)
    }

    /// `GetVeinFrom(randomValue)`.
    pub fn vein_from(&self, mut random_value: u32) -> Option<usize> {
        if self.veins.len() == 1 {
            return Some(0);
        }
        for (i, v) in self.veins.iter().enumerate() {
            if random_value <= v.vein_chance {
                return Some(i);
            }
            random_value -= v.vein_chance;
        }
        None
    }
}

/// `HarvestSystem` plus the definitions its constructor built.
pub struct HarvestSystem {
    pub kind: Kind,
    pub name: &'static str,
    pub definitions: Vec<HarvestDefinition>,
}

/// What a harvest target hit.
#[derive(Copy, Clone, Debug)]
pub enum ToHarvest {
    /// A `Static` that is not movable.
    Static(ItemId),
    StaticTarget(StaticTarget),
    Land(target::LandTarget),
    /// Any other item.
    Item(ItemId),
    Mobile(MobileId),
}

/// `HarvestBank`.
#[derive(Clone, Debug)]
pub struct HarvestBank {
    maximum: i32,
    current: i32,
    next_respawn: DateTime,
    vein: Option<usize>,
    default_vein: Option<usize>,
}

/// What a lock (`BeginAction(toLock)`) is on.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum LockKey {
    Item(ItemId),
    System(Kind),
}

/// The statics of the harvest engine.
type BankMap = IndexMap<(i32, i32), HarvestBank>;

#[derive(Default)]
pub struct HarvestState {
    pub systems: HashMap<Kind, Rc<HarvestSystem>>,
    /// `HarvestDefinition.Banks`: per system, definition and map, by bank coordinates.
    pub banks: HashMap<(Kind, usize, MapId), BankMap>,
    /// `Mobile.BeginAction(object)` locks of the harvest systems.
    pub locks: HashSet<(MobileId, LockKey)>,
}

fn state(w: &mut World) -> &mut HarvestState {
    w.resources.get_mut::<HarvestState>()
}

/// The system of a kind, built on first use (`Mining.System` and the others).
pub fn system(w: &mut World, kind: Kind) -> Rc<HarvestSystem> {
    if let Some(s) = state(w).systems.get(&kind) {
        return s.clone();
    }
    let s = Rc::new(match kind {
        Kind::Mining => mining::create(w),
        Kind::Lumberjacking => lumberjacking::create(w),
        Kind::Fishing => fishing::create(w),
    });
    state(w).systems.insert(kind, s.clone());
    s
}

/// `Core.UOR`.
pub(crate) fn core_uor(w: &World) -> bool {
    muo_content::items::core_uor(w)
}

type AxeTargetFn = fn(&mut World, ItemId, MobileId, ItemId) -> Option<bool>;
type MakeSeaItemFn = fn(&mut World, &'static str, i32, MapId) -> Option<ItemId>;
/// Calls the harvest systems make into classes that live elsewhere (quests, treasure maps, shipwrecks, furniture,
/// choppables). Each is set by the owner of the class; an unset one answers "not that kind of thing".
#[derive(Default)]
pub struct HarvestHooks {
    /// The Witch Apprentice quest's bones objective: true if it consumed the click.
    pub witch_apprentice_bones: Option<fn(&mut World, MobileId) -> bool>,
    /// `TreasureMap.OnBeginDig(from)`.
    pub treasure_map_begin_dig: Option<fn(&mut World, ItemId, MobileId)>,
    /// `targeted is IChoppable` then `OnChop(from)`: true when it is one.
    pub chop: Option<fn(&mut World, ItemId, MobileId) -> bool>,
    /// `targeted is IAxe` with a `BaseAxe`: `None` when it is not an IAxe, else whether `Axe(from, axe)` was done.
    pub axe_target: Option<AxeTargetFn>,
    /// `FurnitureAttribute.Check(item)`.
    pub is_furniture: Option<fn(&World, ItemId) -> bool>,
    /// The first `SOS` in the pack whose target map and location are within 60 tiles.
    pub sos_near: Option<fn(&World, MobileId) -> Option<ItemId>>,
    /// The item an SOS brings up (a shipwrecked item or the chest, deleting the SOS).
    pub sos_catch: Option<fn(&mut World, MobileId) -> Option<ItemId>>,
    /// The Collector quest's pearl objective: true if it consumed the catch.
    pub collector_pearl: Option<fn(&mut World, MobileId) -> bool>,
    /// `new TreasureMap(level, map)` / `new MessageInABottle(map)`.
    pub make_sea_item: Option<MakeSeaItemFn>,
    /// `TreasureMap.IsInHavenIsland(from)`.
    pub in_haven_island: Option<fn(&World, MobileId) -> bool>,
    /// `SpellHelper.CheckMulti(p, map)`.
    pub check_multi: Option<fn(&mut World, Point3D, MapId) -> bool>,
    /// `serpent.FindMostRecentDamager(false) != null`.
    pub has_recent_damager: Option<fn(&mut World, MobileId) -> bool>,
    /// `BigFish.Fisher = from`.
    pub set_fisher: Option<fn(&mut World, ItemId, MobileId)>,
}

pub fn state_hooks(w: &World) -> Option<&HarvestHooks> {
    w.resources.get::<HarvestHooks>()
}

fn kind_by_name(name: &str) -> Option<Kind> {
    match name {
        "Mining" => Some(Kind::Mining),
        "Lumberjacking" => Some(Kind::Lumberjacking),
        "Fishing" => Some(Kind::Fishing),
        _ => None,
    }
}

// ---- messages ----

/// `HarvestDefinition.SendMessageTo(from, message)`.
pub fn send_message_to(w: &mut World, from: MobileId, m: &Td) {
    if m.number > 0 {
        deps::send_localized_message(w, from, m.number);
    } else if let Some(s) = m.string.as_deref().filter(|s| !s.trim().is_empty()) {
        deps::send_message(w, from, s);
    }
}

// ---- banks ----

fn get_vein_at(
    w: &mut World,
    def: &HarvestDefinition,
    map: MapId,
    x: i32,
    y: i32,
) -> Option<usize> {
    if def.veins.len() == 1 {
        return Some(0);
    }
    if def.randomize_veins {
        return def.vein_from(utility::random(&mut w.rng, 1000) as u32);
    }
    // TODO: introduce pulling primes from a config
    let seed = (x * 17 + y * 11 + i32::from(map.0) * 3) as u64;
    def.vein_from(muo_primitives::random::stable_first(seed, i64::from(def.vein_weights)) as u32)
}

fn new_bank(w: &mut World, def: &HarvestDefinition, default_vein: Option<usize>) -> HarvestBank {
    let maximum = utility::random_min_max(&mut w.rng, def.min_total, def.max_total);
    HarvestBank {
        maximum,
        current: maximum,
        next_respawn: DateTime::default(),
        vein: default_vein,
        default_vein,
    }
}

/// `GetBank(map, x, y)`: made on first use. Returns the bank's key for the other bank calls.
fn bank_key(
    w: &mut World,
    kind: Kind,
    di: usize,
    def: &HarvestDefinition,
    map: Option<MapId>,
    x: i32,
    y: i32,
) -> Option<(Kind, usize, MapId, i32, i32)> {
    let map = map.filter(|m| *m != MapId::INTERNAL)?;
    let (bx, by) = (x / def.bank_width, y / def.bank_height);
    let exists = state(w)
        .banks
        .get(&(kind, di, map))
        .is_some_and(|b| b.contains_key(&(bx, by)));
    if !exists {
        let vein = get_vein_at(w, def, map, bx, by);
        let bank = new_bank(w, def, vein);
        state(w)
            .banks
            .entry((kind, di, map))
            .or_default()
            .insert((bx, by), bank);
    }
    Some((kind, di, map, bx, by))
}

fn bank_mut(w: &mut World, k: (Kind, usize, MapId, i32, i32)) -> &mut HarvestBank {
    state(w)
        .banks
        .get_mut(&(k.0, k.1, k.2))
        .and_then(|b| b.get_mut(&(k.3, k.4)))
        .expect("made by bank_key")
}

/// `HarvestBank.CheckRespawn()`.
fn check_respawn(w: &mut World, def: &HarvestDefinition, k: (Kind, usize, MapId, i32, i32)) {
    let now = w.now;
    let (needs,) = {
        let b = bank_mut(w, k);
        (b.current != b.maximum && b.next_respawn <= now,)
    };
    if !needs {
        return;
    }
    let new_default = if def.randomize_veins {
        let v = utility::random(&mut w.rng, def.vein_weights as i32) as u32;
        Some(def.vein_from(v))
    } else {
        None
    };
    let b = bank_mut(w, k);
    b.current = b.maximum;
    if let Some(v) = new_default {
        b.default_vein = v;
    }
    b.vein = b.default_vein;
}

fn bank_current(w: &mut World, def: &HarvestDefinition, k: (Kind, usize, MapId, i32, i32)) -> i32 {
    check_respawn(w, def, k);
    bank_mut(w, k).current
}

fn bank_vein(
    w: &mut World,
    def: &HarvestDefinition,
    k: (Kind, usize, MapId, i32, i32),
) -> Option<usize> {
    check_respawn(w, def, k);
    bank_mut(w, k).vein
}

/// `HarvestBank.Consume(amount, from)`.
fn bank_consume(
    w: &mut World,
    def: &HarvestDefinition,
    k: (Kind, usize, MapId, i32, i32),
    amount: i32,
    from: MobileId,
) {
    check_respawn(w, def, k);
    let full = {
        let b = bank_mut(w, k);
        b.current == b.maximum
    };
    if full {
        let (min, max) = (
            def.min_respawn.total_minutes(),
            def.max_respawn.total_minutes(),
        );
        let rnd = utility::random_double(&mut w.rng);
        let mut minutes = min + rnd * (max - min);
        if def.race_bonus
            && w.mobile(from)
                .is_some_and(|m| m.race == Some(muo_content::player_mobile::RACE_ELF))
        {
            minutes *= 0.75; // 25% off the time.
        }
        let next = w.now + TimeSpan::from_mins_f64(minutes);
        let b = bank_mut(w, k);
        b.current = b.maximum - amount;
        b.next_respawn = next;
    } else {
        bank_mut(w, k).current -= amount;
    }
    let b = bank_mut(w, k);
    if b.current < 0 {
        b.current = 0;
    }
}

// ---- virtuals ----

fn to_harvest_of(w: &World, t: Targeted) -> ToHarvest {
    match t {
        Targeted::Static(s) => ToHarvest::StaticTarget(s),
        Targeted::Land(l) => ToHarvest::Land(l),
        Targeted::Entity(EntityId::Item(i)) => {
            let is_static = w.item(i).is_some_and(|x| {
                !x.movable() && muo_content::items::is_named(x.ty, "Server.Items.Static")
            });
            if is_static {
                ToHarvest::Static(i)
            } else {
                ToHarvest::Item(i)
            }
        }
        Targeted::Entity(EntityId::Mobile(m)) => ToHarvest::Mobile(m),
    }
}

/// `CheckTool(from, tool)`.
fn check_tool(w: &mut World, from: MobileId, tool: ItemId) -> bool {
    let worn_out =
        w.item(tool).is_none_or(|i| i.deleted()) || uses_of(w, tool).is_some_and(|u| u <= 0);
    if worn_out {
        deps::send_localized_message(w, from, 1044038); // You have worn out your tool!
    }
    !worn_out
}

/// `(tool as IUsesRemaining)?.UsesRemaining`.
fn uses_of(w: &World, tool: ItemId) -> Option<i32> {
    crate::craft::uses_remaining_api(w, tool)
}

/// `CheckHarvest(from, tool)`.
fn check_harvest(w: &mut World, sys: &HarvestSystem, from: MobileId, tool: ItemId) -> bool {
    if !check_tool(w, from, tool) {
        return false;
    }
    match sys.kind {
        Kind::Mining => mining::check_harvest(w, from),
        Kind::Lumberjacking => lumberjacking::check_harvest(w, from, tool),
        Kind::Fishing => fishing::check_harvest(w, from),
    }
}

/// `CheckHarvest(from, tool, def, toHarvest)`.
fn check_harvest_def(
    w: &mut World,
    sys: &HarvestSystem,
    from: MobileId,
    tool: ItemId,
    di: usize,
    to: ToHarvest,
) -> bool {
    if !check_tool(w, from, tool) {
        return false;
    }
    match sys.kind {
        Kind::Mining => mining::check_harvest_def(w, from, di, to),
        Kind::Lumberjacking => lumberjacking::check_harvest(w, from, tool),
        Kind::Fishing => fishing::check_harvest(w, from),
    }
}

/// `CheckRange(from, tool, def, map, loc, timed)`.
fn check_range(
    w: &mut World,
    from: MobileId,
    def: &HarvestDefinition,
    map: Option<MapId>,
    loc: Point3D,
    timed: bool,
) -> bool {
    let in_range = mx::map(w, from) == map && mx::in_range_point(w, from, loc, def.max_range);
    if !in_range {
        let m = if timed {
            &def.timed_out_of_range_message
        } else {
            &def.out_of_range_message
        };
        send_message_to(w, from, m);
    }
    in_range
}

/// `CheckResources(from, tool, def, map, loc, timed)`.
#[allow(clippy::too_many_arguments)]
fn check_resources(
    w: &mut World,
    sys: &HarvestSystem,
    from: MobileId,
    di: usize,
    map: Option<MapId>,
    loc: Point3D,
    timed: bool,
) -> bool {
    if sys.kind == Kind::Fishing && fishing::check_resources_sos(w, from) {
        return true;
    }
    let def = &sys.definitions[di];
    let available = match bank_key(w, sys.kind, di, def, map, loc.x, loc.y) {
        Some(k) => bank_current(w, def, k) >= def.consumed_per_harvest,
        None => false,
    };
    if !available {
        let m = if timed {
            &def.double_harvest_message
        } else {
            &def.no_resources_message
        };
        send_message_to(w, from, m);
    }
    available
}

/// `OnBadHarvestTarget(from, tool, toHarvest)`.
fn on_bad_harvest_target(w: &mut World, sys: &HarvestSystem, from: MobileId, to: ToHarvest) {
    match sys.kind {
        Kind::Mining => mining::on_bad_harvest_target(w, from, to),
        Kind::Lumberjacking => lumberjacking::on_bad_harvest_target(w, from, to),
        Kind::Fishing => {}
    }
}

/// `GetLock(from, tool, def, toHarvest)`: the tool, or for the three systems the system itself.
fn get_lock(sys: &HarvestSystem, _tool: ItemId) -> LockKey {
    LockKey::System(sys.kind)
}

/// `GetHarvestDetails(from, tool, toHarvest, out tileID, out map, out loc, out isLand)`.
pub(crate) fn get_harvest_details(
    w: &World,
    from: MobileId,
    to: ToHarvest,
) -> Option<(i32, Option<MapId>, Point3D, bool)> {
    let (tile, map, loc, land) = match to {
        ToHarvest::Static(i) => {
            let it = w.item(i)?;
            (it.item_id, it.map, ops::get_world_location(w, i), false)
        }
        ToHarvest::StaticTarget(s) => (s.item_id, mx::map(w, from), s.location, false),
        ToHarvest::Land(l) => (l.tile_id, mx::map(w, from), l.location, true),
        ToHarvest::Item(_) | ToHarvest::Mobile(_) => return None,
    };
    let ok = map.is_some_and(|m| m != MapId::INTERNAL);
    ok.then_some((tile, map, loc, land))
}

fn get_definition(sys: &HarvestSystem, tile_id: i32, is_land: bool) -> Option<usize> {
    sys.definitions
        .iter()
        .position(|d| d.validate(tile_id, is_land))
}

/// `BladedItemTarget`'s kindling branch: `Lumberjacking.System`'s `GetHarvestDetails`, `Validate`, `GetBank`, and
/// consuming 5 from the bank (`TargetHooks::lumberjacking_kindling`).
pub fn kindling_bank(
    w: &mut World,
    from: MobileId,
    _blade: ItemId,
    targeted: Targeted,
) -> muo_content::targets::KindlingBank {
    use muo_content::targets::KindlingBank as K;
    let sys = system(w, Kind::Lumberjacking);
    let to = to_harvest_of(w, targeted);
    let Some((tile, map, loc, land)) = get_harvest_details(w, from, to) else {
        return K::Invalid;
    };
    let Some(di) = sys
        .definitions
        .first()
        .filter(|d| d.validate(tile, land))
        .map(|_| 0)
    else {
        return K::Invalid;
    };
    let def = &sys.definitions[di];
    let Some(k) = bank_key(w, sys.kind, di, def, map, loc.x, loc.y) else {
        return K::NoBank;
    };
    if bank_current(w, def, k) < 5 {
        return K::NotEnough;
    }
    bank_consume(w, def, k, 5, from);
    K::Consumed
}

// ---- harvesting ----

/// `BeginHarvesting(from, tool)`.
pub fn begin_harvesting(
    w: &mut World,
    sys: &Rc<HarvestSystem>,
    from: MobileId,
    tool: ItemId,
) -> bool {
    if !check_harvest(w, sys, from, tool) {
        return false;
    }
    let mut t = Target::new(w, -1, true, TargetFlags::NONE);
    t.disallow_multis = true;
    let sys2 = sys.clone();
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            on_target(w, &sys2, from, tool, targeted);
        },
    ));
    let id = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(id));
    match sys.kind {
        Kind::Mining => deps::send_localized_message(w, from, 503033), // Where do you wish to dig?
        Kind::Fishing => deps::send_localized_message(w, from, 500974), // What water do you want to fish in?
        Kind::Lumberjacking => {}
    }
    true
}

/// `HarvestTarget.OnTarget`.
fn on_target(
    w: &mut World,
    sys: &Rc<HarvestSystem>,
    from: MobileId,
    tool: ItemId,
    targeted: Targeted,
) {
    if sys.kind == Kind::Lumberjacking && lumberjacking::on_target_special(w, from, tool, targeted)
    {
        return;
    }
    if sys.kind == Kind::Mining && mining::on_target_special(w, from, targeted) {
        return;
    }
    start_harvesting(w, sys, from, tool, to_harvest_of(w, targeted));
}

/// `StartHarvesting(from, tool, toHarvest)`.
pub fn start_harvesting(
    w: &mut World,
    sys: &Rc<HarvestSystem>,
    from: MobileId,
    tool: ItemId,
    to: ToHarvest,
) {
    if !check_harvest(w, sys, from, tool) {
        return;
    }
    let Some((tile, map, loc, land)) = get_harvest_details(w, from, to) else {
        on_bad_harvest_target(w, sys, from, to);
        return;
    };
    let Some(di) = get_definition(sys, tile, land) else {
        on_bad_harvest_target(w, sys, from, to);
        return;
    };
    let def = &sys.definitions[di];
    if !check_range(w, from, def, map, loc, false) {
        return;
    }
    if !check_resources(w, sys, from, di, map, loc, false) {
        return;
    }
    if !check_harvest_def(w, sys, from, tool, di, to) {
        return;
    }

    let lock = get_lock(sys, tool);
    if !state(w).locks.insert((from, lock)) {
        // OnConcurrentHarvest
        if sys.kind == Kind::Fishing {
            deps::send_localized_message(w, from, 500972); // You are already fishing.
        }
        return;
    }

    // HarvestTimer: first tick now, then every EffectDelay, for one of EffectCounts ticks.
    let count = pick(w, &def.effect_counts).unwrap_or(1);
    let mut index = 0;
    let sys2 = sys.clone();
    let handle = muo_server::timer::create(w, TimeSpan::ZERO, def.effect_delay, 0, move |w| {
        index += 1;
        let keep = on_harvesting(w, &sys2, from, tool, di, to, lock, index == count);
        if !keep && let Some(h) = muo_server::timer::current(w) {
            let mut h = h;
            h.cancel(w);
        }
    });
    muo_server::timer::begin(w, handle);

    // OnHarvestStarted
    match sys.kind {
        Kind::Mining | Kind::Lumberjacking => {
            if muo_content::core::ml(w) {
                muo_server::mobile::base::revealing_action(w, from);
            }
        }
        Kind::Fishing => fishing::on_harvest_started(w, from, to),
    }
}

fn pick(w: &mut World, list: &[i32]) -> Option<i32> {
    if list.is_empty() {
        return None;
    }
    let i = utility::random(&mut w.rng, list.len() as i32) as usize;
    list.get(i).copied()
}

fn end_lock(w: &mut World, from: MobileId, lock: LockKey) {
    state(w).locks.remove(&(from, lock));
}

/// `OnHarvesting(from, tool, def, toHarvest, locked, last)`: true to keep the timer going.
#[allow(clippy::too_many_arguments)]
fn on_harvesting(
    w: &mut World,
    sys: &Rc<HarvestSystem>,
    from: MobileId,
    tool: ItemId,
    di: usize,
    to: ToHarvest,
    lock: LockKey,
    last: bool,
) -> bool {
    if !check_harvest(w, sys, from, tool) {
        end_lock(w, from, lock);
        return false;
    }
    let Some((tile, map, loc, land)) = get_harvest_details(w, from, to) else {
        end_lock(w, from, lock);
        on_bad_harvest_target(w, sys, from, to);
        return false;
    };
    let def = &sys.definitions[di];
    if !def.validate(tile, land) {
        end_lock(w, from, lock);
        on_bad_harvest_target(w, sys, from, to);
        return false;
    }
    if !check_range(w, from, def, map, loc, true) {
        end_lock(w, from, lock);
        return false;
    }
    if !check_resources(w, sys, from, di, map, loc, true) {
        end_lock(w, from, lock);
        return false;
    }
    if !check_harvest_def(w, sys, from, tool, di, to) {
        end_lock(w, from, lock);
        return false;
    }

    // DoHarvestingEffect
    let dir = muo_server::mobile::part2::effects::get_direction_to(w, from, loc.x, loc.y, false);
    muo_server::mobile::part2::effects::set_direction(w, from, dir);
    if !muo_server::mobile::part1::props_b::mounted(w, from) {
        let action = pick(w, &def.effect_actions).unwrap_or(0);
        mx::animate(w, from, action, 5, 1, true, false, 0);
    }

    // HarvestSoundTimer
    let sys2 = sys.clone();
    muo_server::timer::start(w, def.effect_sound_delay, TimeSpan::ZERO, 1, move |w| {
        // DoHarvestingSound
        let def = &sys2.definitions[di];
        if let Some(s) = pick(w, &def.effect_sounds) {
            mx::play_sound(w, from, s);
        } else {
            mx::play_sound(w, from, -1);
        }
        if last {
            finish_harvesting(w, &sys2, from, tool, di, to, lock);
        }
    });

    !last
}

/// `MutateResource`: the resource used, as `(resource, index in the definition)`.
fn mutate_resource(
    w: &mut World,
    sys: &HarvestSystem,
    from: MobileId,
    di: usize,
    map: Option<MapId>,
    loc: Point3D,
    vein: HarvestVein,
) -> (HarvestResource, Option<usize>) {
    let def = &sys.definitions[di];
    if sys.kind == Kind::Fishing
        && let Some(r) = fishing::deep_water_resource(w, map, loc)
    {
        return (r, None);
    }
    let racial_bonus = def.race_bonus
        && w.mobile(from)
            .is_some_and(|m| m.race == Some(muo_content::player_mobile::RACE_ELF));
    let primary = &def.resources[vein.primary];
    let fallback = vein.fallback.map(|f| &def.resources[f]);
    if vein.chance_to_fallback
        > utility::random_double(&mut w.rng) + if racial_bonus { 0.20 } else { 0.0 }
    {
        // `return fallback` (null for a vein without one)
        return match vein.fallback {
            Some(f) => (def.resources[f].clone(), Some(f)),
            None => (primary.clone(), Some(vein.primary)),
        };
    }
    let skill_value = skills::value(w, from, def.skill);
    if let (Some(f), Some(_)) = (vein.fallback, fallback)
        && (skill_value < primary.req_skill || skill_value < primary.min_skill)
    {
        return (def.resources[f].clone(), Some(f));
    }
    (primary.clone(), Some(vein.primary))
}

/// `FinishHarvesting(from, tool, def, toHarvest, locked)`.
#[allow(clippy::too_many_arguments)]
fn finish_harvesting(
    w: &mut World,
    sys: &Rc<HarvestSystem>,
    from: MobileId,
    tool: ItemId,
    di: usize,
    to: ToHarvest,
    lock: LockKey,
) {
    end_lock(w, from, lock);
    if !check_harvest(w, sys, from, tool) {
        return;
    }
    let Some((tile, map, loc, land)) = get_harvest_details(w, from, to) else {
        on_bad_harvest_target(w, sys, from, to);
        return;
    };
    let def = &sys.definitions[di];
    if !def.validate(tile, land) {
        on_bad_harvest_target(w, sys, from, to);
        return;
    }
    if !check_range(w, from, def, map, loc, true) {
        return;
    }
    if !check_resources(w, sys, from, di, map, loc, true) {
        return;
    }
    if !check_harvest_def(w, sys, from, tool, di, to) {
        return;
    }
    if sys.kind == Kind::Fishing && fishing::special_harvest(w, from) {
        return;
    }

    let Some(k) = bank_key(w, sys.kind, di, def, map, loc.x, loc.y) else {
        return;
    };
    let mut vein = bank_vein(w, def, k);
    if let Some(v) = vein {
        vein = Some(mutate_vein(w, sys, from, tool, di, v));
    }
    let Some(vein_idx) = vein else { return };
    let vein = def.veins[vein_idx];

    let (resource, res_index) = mutate_resource(w, sys, from, di, map, loc, vein);
    let skill_base = skills::base(w, from, def.skill);

    let mut made: Option<&'static str> = None;
    if skill_base >= resource.req_skill
        && mx::check_skill(w, from, def.skill, resource.min_skill, resource.max_skill)
    {
        let mut ty = get_resource_type(w, sys, from, di, &resource);
        if let Some(t) = ty {
            ty = mutate_type(w, sys, t, from, map, loc);
        }
        if let Some(t) = ty {
            match fishing_or_plain_construct(w, sys, t, from) {
                None => {}
                Some(item) => {
                    if w.item(item).is_some_and(|i| i.stackable()) {
                        let amount = def.consumed_per_harvest;
                        let felucca_amount = def.consumed_per_felucca_harvest;
                        let racial_amount = (f64::from(amount) * 1.1).ceil() as i32;
                        let felucca_racial = (f64::from(felucca_amount) * 1.1).ceil() as i32;
                        let human = w.mobile(from).is_some_and(|m| {
                            m.race == Some(muo_content::player_mobile::RACE_HUMAN)
                        });
                        let eligible = def.race_bonus && human;
                        let in_felucca = map == Some(MapId::FELUCCA);
                        let current = bank_current(w, def, k);
                        let new_amount = if eligible
                            && in_felucca
                            && current >= felucca_racial
                            && utility::random_double(&mut w.rng) < 0.1
                        {
                            felucca_racial
                        } else if in_felucca && current >= felucca_amount {
                            felucca_amount
                        } else if eligible
                            && current >= racial_amount
                            && utility::random_double(&mut w.rng) < 0.1
                        {
                            racial_amount
                        } else {
                            amount
                        };
                        ops::set_amount(w, item, new_amount);
                    }
                    let amount = w.item(item).map_or(1, |i| i.amount);
                    bank_consume(w, def, k, amount, from);

                    if give(w, sys, from, item, def.place_at_feet_if_full) {
                        send_success_to(w, sys, from, item, &resource);
                    } else {
                        send_message_to(w, from, &def.pack_full_message);
                        if let Some(t) = w.item(item).map(|i| i.ty) {
                            (t.delete)(w, item);
                        }
                    }

                    if let Some(bonus) = get_bonus_resource(w, def)
                        && let Some(bt) = bonus.ty
                        && skill_base >= bonus.req_skill
                    {
                        let bonus_item = construct(w, bt);
                        // Bonuses always allow placing at feet, even if pack is full irregrdless of def
                        match bonus_item {
                            Some(bi) if give(w, sys, from, bi, true) => {
                                send_message_to(w, from, &bonus.message)
                            }
                            Some(bi) => {
                                if let Some(t) = w.item(bi).map(|i| i.ty) {
                                    (t.delete)(w, bi);
                                }
                            }
                            None => {}
                        }
                    }

                    if let Some(uses) = uses_of(w, tool) {
                        // IUsesRemaining: ShowUsesRemaining = true
                        if let Some(d) = tool_mod::data_mut(w, tool) {
                            d.show_uses_remaining = true;
                        } else if let Some(d) = muo_content::items::weapon::data_mut(w, tool) {
                            d.show_uses_remaining = true;
                        }
                        let mut left = uses;
                        if left > 0 {
                            left -= 1;
                            crate::craft::set_uses_remaining_api(w, tool, left);
                        }
                        if left < 1 {
                            if let Some(t) = w.item(tool).map(|i| i.ty) {
                                (t.delete)(w, tool);
                            }
                            send_message_to(w, from, &def.tool_broke_message);
                        }
                    }
                    made = Some(t);
                }
            }
        }
    }
    if made.is_none() {
        send_message_to(w, from, &def.fail_message);
    }

    on_harvest_finished(w, sys, from, tool, di, vein, &resource, res_index, to);
}

fn construct(w: &mut World, ty: &str) -> Option<ItemId> {
    muo_content::factory::make_item(w, ty)
}

fn fishing_or_plain_construct(
    w: &mut World,
    sys: &HarvestSystem,
    ty: &'static str,
    from: MobileId,
) -> Option<ItemId> {
    if sys.kind == Kind::Fishing {
        return fishing::construct(w, ty, from);
    }
    construct(w, ty)
}

/// `GetBonusResource()`.
fn get_bonus_resource(w: &mut World, def: &HarvestDefinition) -> Option<BonusResource> {
    let list = def.bonus_resources.as_ref()?;
    let mut random_value = utility::random_double(&mut w.rng) * 100.0;
    for b in list {
        if random_value <= b.chance {
            return Some(b.clone());
        }
        random_value -= b.chance;
    }
    None
}

/// `MutateVein`.
fn mutate_vein(
    w: &mut World,
    sys: &HarvestSystem,
    from: MobileId,
    tool: ItemId,
    di: usize,
    vein: usize,
) -> usize {
    if sys.kind == Kind::Mining {
        return mining::mutate_vein(w, sys, tool, di, vein);
    }
    let _ = from;
    vein
}

/// `GetResourceType`.
fn get_resource_type(
    w: &mut World,
    sys: &HarvestSystem,
    from: MobileId,
    di: usize,
    resource: &HarvestResource,
) -> Option<&'static str> {
    if sys.kind == Kind::Mining && di == 0 {
        return mining::get_resource_type(w, from, di, resource);
    }
    if resource.types.is_empty() {
        return None;
    }
    let i = utility::random(&mut w.rng, resource.types.len() as i32) as usize;
    resource.types.get(i).copied()
}

/// `MutateType`: `from.Region.GetResource(type)`, and Fishing's table.
fn mutate_type(
    w: &mut World,
    sys: &HarvestSystem,
    ty: &'static str,
    from: MobileId,
    map: Option<MapId>,
    loc: Point3D,
) -> Option<&'static str> {
    if sys.kind == Kind::Fishing {
        return Some(fishing::mutate_type(w, ty, from, map, loc));
    }
    Some(region_resource(w, from, ty))
}

fn region_resource(w: &World, from: MobileId, ty: &'static str) -> &'static str {
    let region = muo_server::mobile::part1::props_b::region(w, from);
    match w.region(region).map(|r| r.ty) {
        Some(t) => (t.get_resource)(w, region, ty),
        None => ty,
    }
}

/// `Give(m, item, placeAtFeet)`.
fn give(
    w: &mut World,
    sys: &HarvestSystem,
    m: MobileId,
    item: ItemId,
    place_at_feet: bool,
) -> bool {
    if sys.kind == Kind::Fishing {
        let (handled, result) = fishing::give(w, m, item, place_at_feet);
        if handled {
            return result;
        }
    }
    if muo_server::mobile::part2::items::place_in_backpack(w, m, item) {
        return true;
    }
    if !place_at_feet {
        return false;
    }
    let Some(map) = mx::map(w, m) else {
        return false;
    };
    for i in muo_server::mobile::part2::enumerators::get_items_at(w, m) {
        if muo_server::item::base::stack_with(w, i, Some(m), i, false) {
            return true;
        }
    }
    let loc = mx::location(w, m);
    ops::move_to_world_map(w, item, loc, Some(map));
    true
}

/// `SendSuccessTo(from, item, resource)`.
fn send_success_to(
    w: &mut World,
    sys: &HarvestSystem,
    from: MobileId,
    item: ItemId,
    resource: &HarvestResource,
) {
    match sys.kind {
        Kind::Mining if muo_content::factory::item_is(w, item, "Server.Items.BaseGranite") => {
            deps::send_localized_message(w, from, 1044606); // You carefully extract some workable stone from the ore vein!
        }
        Kind::Fishing => fishing::send_success_to(w, from, item),
        _ => send_message_to(w, from, &resource.success_message),
    }
}

/// `OnHarvestFinished(..)`.
#[allow(clippy::too_many_arguments)]
fn on_harvest_finished(
    w: &mut World,
    sys: &HarvestSystem,
    from: MobileId,
    tool: ItemId,
    di: usize,
    vein: HarvestVein,
    resource: &HarvestResource,
    res_index: Option<usize>,
    to: ToHarvest,
) {
    match sys.kind {
        Kind::Mining => mining::on_harvest_finished(w, from, tool, di, vein, resource, res_index),
        Kind::Fishing => fishing::on_harvest_finished(w, from, to),
        Kind::Lumberjacking => {}
    }
}

// ---- hooks for the items ----

fn tool_system(w: &World, tool: ItemId) -> Option<Kind> {
    let it = w.item(tool)?;
    if let Some(c) = it.ty.family::<tool_mod::ToolClass>()
        && !c.harvest_system.is_empty()
    {
        return kind_by_name(c.harvest_system);
    }
    it.ty.family::<muo_content::items::weapon::WeaponClass>()?;
    muo_content::items::weapon::harvest_system(w, tool).and_then(kind_by_name)
}

fn hook_harvest_begin(w: &mut World, from: MobileId, tool: ItemId) {
    if let Some(kind) = tool_system(w, tool) {
        let sys = system(w, kind);
        begin_harvesting(w, &sys, from, tool);
    }
}

fn hook_harvest_is_mining(w: &World, tool: ItemId) -> bool {
    tool_system(w, tool) == Some(Kind::Mining)
}

fn hook_context_entries(
    w: &mut World,
    from: MobileId,
    tool: ItemId,
    list: &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>,
) {
    mining::add_context_menu_entries(w, from, tool, list);
}

fn configure(w: &mut World) {
    w.resources
        .get_mut::<muo_content::targets::TargetHooks>()
        .lumberjacking_kindling = Some(kindling_bank);
    let h = w
        .resources
        .get_mut::<muo_content::items::weapon_hooks::WeaponHooks>();
    h.harvest_begin = Some(hook_harvest_begin);
    h.harvest_is_mining = Some(hook_harvest_is_mining);
    h.harvest_context_entries = Some(hook_context_entries);
    // IAxe: logs cut into boards
    let hh = w.resources.get_mut::<HarvestHooks>();
    hh.axe_target = Some(axe::axe_target);
    hh.witch_apprentice_bones = Some(crate::quests::external::mining_grave);
    hh.collector_pearl = Some(crate::quests::external::fishing_special_harvest);
    hh.in_haven_island = Some(|w, m| {
        let l = mx::location(w, m);
        muo_items::maps::treasure::is_in_haven_island(muo_primitives::Point2D::new(l.x, l.y))
    });
    hh.treasure_map_begin_dig = Some(muo_items::maps::treasure::on_begin_dig);
    // `targeted is IChoppable`: the classes that implement it pushed their `OnChop` on `CommonItemHooks`
    hh.chop = Some(|w, item, from| {
        let choppables = muo_content::items::common::hooks(w)
            .map(|h| h.choppables.clone())
            .unwrap_or_default();
        choppables.iter().any(|f| f(w, item, from))
    });
    hh.check_multi =
        Some(|w, p, map| muo_content::spells::helper::check_multi(w, p, Some(map), true, 0));
    hh.has_recent_damager = Some(|w, m| {
        muo_server::mobile::part2::damage::find_most_recent_damager(w, m, false).is_some()
    });
    hh.set_fisher = Some(muo_items::resources::cloth::set_fisher);
    hh.make_sea_item = Some(|w, ty, level, map| match ty {
        "Server.Items.TreasureMap" => Some(muo_items::maps::treasure::treasure_map_new(
            w,
            &muo_items::maps::treasure::TREASURE_MAP,
            level,
            Some(map),
        )),
        "Server.Items.MessageInABottle" => Some(
            muo_items::skill_items::fishing::new_message_in_a_bottle(w, Some(map), None),
        ),
        _ => None,
    });
    hh.sos_near = Some(muo_items::skill_items::fishing::sos_near);
    hh.sos_catch = Some(muo_items::skill_items::fishing::sos_catch);
    w.resources
        .get_mut::<muo_items::weapons::pickaxes::ProspectHooks>()
        .prospect = Some(mining::prospect);
    let f = w
        .resources
        .get_mut::<muo_items::skill_items::fishing::FishingHooks>();
    f.make_leviathan = Some(|w, from| muo_mobiles::classes::misc4::new_leviathan(w, Some(from)));
    // FishingHooks.localized_format is set in wiring2 (the Localization table lives there) and
    // FishingHooks.fill_treasure_chest by muo_items::last::chests (TreasureMapChest.Fill).
}

pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        Phase::Configure,
        "Server.Engines.Harvest.Configure",
        configure,
    );
}
