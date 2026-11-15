//! The faction mobiles: `BaseFactionGuard`, `Orders`, the ten guard classes, `FactionGuardAI` (its
//! movement and combat decisions), `BaseFactionVendor` and the five vendors, and `FactionWarHorse`.
//!
//! The constructors of the guard classes, the vendors and the war horse are their templates
//! (`Data/Templates/Mobiles/factions.json`); a guard's `GuardAI` flags are its `guardFlags` prop, read when it is
//! made or loaded.
//!
//! (the spell, bandage and potion half of `Think` is guard_ai.rs).

use muo_primitives::dotnet::InvariantCase;

use muo_content::creature::enums::{AIType, DEFAULT_RANGE_PERCEPTION, FightMode, OrderType};
use muo_content::creature::mount::{BASE_MOUNT, BASE_MOUNT_C};
use muo_content::creature::setters::new_creature;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_content::hooks::rnd;
use muo_content::vendor::{
    BASE_VENDOR, BASE_VENDOR_C, BASE_VENDOR_CLASS, BuyInfo, SbInfo, SbInstance, VendorClass,
};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, Direction, MessageType, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::mobile::MobileType;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::messages as mm;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;
use muo_server::type_registry::TypeRegistry;

use muo_content::ai::{ActionType, AiClass, base as ai_base, movement as ai_move, set_action};
use muo_content::mx;

use super::state::{read_faction_ref, read_town_ref, write_faction_ref, write_town_ref};
use super::tables::{FACTION_DEFS, TOWN_DEFS, VENDOR_DEFS};
use super::{FACET, find, st, town};
use muo_mobiles::npcs::vendors::npc::TEMPLATE_VENDOR_V;

// ---- data ----

/// `ReactionType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReactionType {
    Ignore = 0,
    Warn = 1,
    Attack = 2,
}

/// `MovementType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MovementType {
    Stand = 0,
    Patrol = 1,
    Follow = 2,
}

/// `Orders`.
#[derive(Clone, Debug)]
pub struct Orders {
    pub reactions: Vec<(Option<usize>, ReactionType)>,
    pub movement: MovementType,
    pub follow: Option<MobileId>,
}

impl Orders {
    fn new() -> Orders {
        Orders {
            reactions: Vec::new(),
            movement: MovementType::Patrol,
            follow: None,
        }
    }
}

/// What `BaseFactionGuard` adds to a creature.
pub struct GuardData {
    pub faction: Option<usize>,
    pub town: Option<usize>,
    pub orders: Orders,
    /// `m_OrdersEnd`.
    pub orders_end: DateTime,
    /// `GuardAI`.
    pub flags: u8,
}

/// `GuardAI` flags.
pub const BLESS: u8 = 0x01;
pub const CURSE: u8 = 0x02;
pub const MELEE: u8 = 0x04;
pub const MAGIC: u8 = 0x08;
pub const SMART: u8 = 0x10;

/// What `BaseFactionVendor` adds to a vendor.
#[derive(Default)]
pub struct FactionVendorData {
    pub faction: Option<usize>,
    pub town: Option<usize>,
}

/// `FactionWarHorse._faction`.
#[derive(Default)]
pub struct WarHorseData {
    pub faction: Option<usize>,
}

pub(super) fn gd(w: &World, m: MobileId) -> Option<&GuardData> {
    w.mobile(m)?.ext.get::<GuardData>()
}

fn gd_mut(w: &mut World, m: MobileId) -> Option<&mut GuardData> {
    w.mobile_mut(m)?.ext.get_mut::<GuardData>()
}

/// `m is BaseFactionGuard`.
pub fn is_guard(w: &World, m: MobileId) -> bool {
    gd(w, m).is_some()
}

/// `BaseFactionGuard.Faction`.
pub fn guard_faction(w: &World, m: MobileId) -> Option<usize> {
    gd(w, m)?.faction
}

/// `BaseFactionGuard.Town`.
pub fn guard_town(w: &World, m: MobileId) -> Option<usize> {
    gd(w, m)?.town
}

fn guard_unregister(w: &mut World, g: MobileId) {
    if let Some(t) = guard_town(w, g) {
        town::unregister_guard(w, t, g);
    }
}

fn guard_register(w: &mut World, g: MobileId) {
    if let Some(d) = gd(w, g)
        && let (Some(t), Some(_)) = (d.town, d.faction)
    {
        town::register_guard(w, t, g);
    }
}

/// `Faction` setter.
pub fn set_guard_faction(w: &mut World, g: MobileId, f: Option<usize>) {
    guard_unregister(w, g);
    if let Some(d) = gd_mut(w, g) {
        d.faction = f;
    }
    guard_register(w, g);
}

/// `Town` setter.
pub fn set_guard_town(w: &mut World, g: MobileId, t: Option<usize>) {
    guard_unregister(w, g);
    if let Some(d) = gd_mut(w, g) {
        d.town = t;
    }
    guard_register(w, g);
}

fn vd(w: &World, m: MobileId) -> Option<&FactionVendorData> {
    w.mobile(m)?.ext.get::<FactionVendorData>()
}

/// `BaseFactionVendor.Town`.
pub fn vendor_town(w: &World, m: MobileId) -> Option<usize> {
    vd(w, m)?.town
}

/// `BaseFactionVendor.Faction`.
pub fn vendor_faction(w: &World, m: MobileId) -> Option<usize> {
    vd(w, m)?.faction
}

fn vendor_register(w: &mut World, v: MobileId) {
    if let Some(d) = vd(w, v)
        && let (Some(t), Some(_)) = (d.town, d.faction)
    {
        town::register_vendor(w, t, v);
    }
}

fn vendor_unregister(w: &mut World, v: MobileId) {
    if let Some(t) = vendor_town(w, v) {
        town::unregister_vendor(w, t, v);
    }
}

pub fn set_vendor_town_faction(w: &mut World, v: MobileId, t: Option<usize>, f: Option<usize>) {
    vendor_unregister(w, v);
    if let Some(d) = w
        .mobile_mut(v)
        .and_then(|m| m.ext.get_mut::<FactionVendorData>())
    {
        d.town = t;
        d.faction = f;
    }
    vendor_register(w, v);
}

/// `entity.MoveToWorld(location, map); entity.Home = entity.Location`.
pub fn move_and_home(w: &mut World, m: MobileId, loc: Point3D, map: Option<MapId>) {
    if let Some(map) = map {
        muo_server::mobile::base::move_to_world(w, m, loc, map);
    }
    let here = w.mobile(m).map(|x| x.location).unwrap_or(loc);
    muo_content::creature::control::set_home(w, m, here);
}

/// `GuardList.Construct()`: a guard by class name, not yet in a town.
pub fn construct_guard(w: &mut World, type_name: &str) -> Option<MobileId> {
    muo_content::factory::make_mobile(w, type_name)
}

/// `VendorList.Construct(town, faction)`.
pub fn construct_vendor(w: &mut World, def: usize, t: usize, f: usize) -> Option<MobileId> {
    let v = muo_content::factory::make_mobile(w, VENDOR_DEFS[def].type_name)?;
    set_vendor_town_faction(w, v, Some(t), Some(f));
    Some(v)
}

/// The constructor of a guard class: `BaseFactionGuard(title)` (the `GuardAI` flags are the class's `guardFlags`
/// prop), then the class's template.
fn new_guard(w: &mut World, ty: &'static MobileType) -> MobileId {
    let id = new_creature(
        w,
        ty,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    let flags = guard_flags(w, id);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(GuardData {
            faction: None,
            town: None,
            orders: Orders::new(),
            orders_end: DateTime::MIN,
            flags,
        });
    }
    muo_content::templates::apply_mobile(w, id);
    id
}

/// The class's `guardFlags` prop (`["Magic", "Melee", "Smart", "Bless", "Curse"]`).
fn guard_flags(w: &World, id: MobileId) -> u8 {
    let names =
        muo_content::templates::props::mobile(w, id, "guardFlags").and_then(|v| v.as_array());
    names.into_iter().flatten().fold(0, |f, n| {
        f | match n.as_str() {
            Some("Bless") => BLESS,
            Some("Curse") => CURSE,
            Some("Melee") => MELEE,
            Some("Magic") => MAGIC,
            Some("Smart") => SMART,
            _ => 0,
        }
    })
}

/// `Orders.GetReaction(faction)`: the stored reaction, adding the default one.
fn get_reaction(w: &mut World, g: MobileId, f: Option<usize>) -> ReactionType {
    let own = guard_faction(w, g);
    let d = gd_mut(w, g).expect("guard");
    if let Some(&(_, t)) = d.orders.reactions.iter().find(|(rf, _)| *rf == f) {
        return t;
    }
    let t = if f.is_none() || f == own {
        ReactionType::Ignore
    } else {
        ReactionType::Attack
    };
    d.orders.reactions.push((f, t));
    t
}

fn set_reaction(w: &mut World, g: MobileId, f: Option<usize>, t: ReactionType) {
    get_reaction(w, g, f);
    if let Some(e) =
        gd_mut(w, g).and_then(|d| d.orders.reactions.iter_mut().find(|(rf, _)| *rf == f))
    {
        e.1 = t;
    }
}

/// `BaseFactionGuard.IsEnemy(m)`.
fn guard_is_enemy(w: &mut World, g: MobileId, m: MobileId) -> bool {
    let our = guard_faction(w, g);
    let mut theirs = find(w, m);
    if theirs.is_none() {
        theirs = guard_faction(w, m);
    }
    if let (Some(o), Some(t)) = (our, theirs)
        && o != t
    {
        if get_reaction(w, g, Some(t)) == ReactionType::Attack {
            return true;
        }
        let aggressed: Vec<MobileId> = w.mobile(m).map_or_else(Vec::new, |x| {
            x.aggressed.iter().map(|a| a.defender).collect()
        });
        for d in aggressed {
            if guard_faction(w, d) == Some(o) {
                return true;
            }
        }
    }
    false
}

fn guard_on_movement(w: &mut World, g: MobileId, m: MobileId, old: Point3D) {
    (BASE_CREATURE.on_movement)(w, g, m, old);
    let player = w.mobile(m).is_some_and(|x| x.player);
    if !(player && muo_server::mobile::base::alive(w, m)) {
        return;
    }
    let loc = w.mobile(m).map(|x| x.location).unwrap_or_default();
    if !(muo_server::mobile::base::in_range_point3d(w, g, loc, 10)
        && !muo_server::mobile::base::in_range_point3d(w, g, old, 10)
        && muo_server::mobile::part1::combat::in_los(w, g, m))
    {
        return;
    }
    let mf = find(w, m);
    if get_reaction(w, g, mf) != ReactionType::Warn {
        return;
    }
    let d = mx::get_direction_to(w, g, m);
    muo_server::mobile::part1::props_b::set_direction(w, g, d);
    let name = muo_server::mobile::base::name(w, m).unwrap_or_default();
    let line = match rnd::below(w, 6) {
        0 => format!(
            "I warn you, {name}, you would do well to leave this area before someone shows you the world of gray."
        ),
        1 => format!(
            "It would be wise to leave this area, {name}, lest your head become my commanders' trophy."
        ),
        2 => format!(
            "You are bold, {name}, for one of the meager {}. Leave now, lest you be taught the taste of dirt.",
            mf.map_or("civilians", |f| FACTION_DEFS[f].friendly_name)
        ),
        3 => format!("Your presence here is an insult, {name}. Be gone now, knave."),
        4 => format!("Dost thou wish to be hung by your toes, {name}? Nay? Then come no closer."),
        _ => format!("Hey, {name}. Yeah, you. Get out of here before I beat you with a stick."),
    };
    mm::say_text(w, g, &line);
}

fn guard_handles_on_speech(w: &World, g: MobileId, from: MobileId) -> bool {
    let loc = w.mobile(from).map(|x| x.location).unwrap_or_default();
    muo_server::mobile::base::in_range_point3d(w, g, loc, 12)
        || (BASE_CREATURE.handles_on_speech)(w, g, from)
}

/// `ChangeReaction(faction, type)`.
fn change_reaction(w: &mut World, g: MobileId, f: Option<usize>, t: ReactionType) {
    match f {
        None => match t {
            ReactionType::Ignore => mm::say_number(w, g, 1005179, ""), // Civilians will now be ignored.
            ReactionType::Warn => mm::say_number(w, g, 1005180, ""), // Civilians will now be warned of their impending deaths.
            ReactionType::Attack => return,
        },
        Some(f) => {
            let d = FACTION_DEFS[f];
            let n = match t {
                ReactionType::Ignore => d.guard_ignore,
                ReactionType::Warn => d.guard_warn,
                ReactionType::Attack => d.guard_attack,
            };
            if n > 0 {
                mm::say_number(w, g, n, "");
            }
        }
    }
    set_reaction(w, g, f, t);
}

fn guard_on_speech(w: &mut World, g: MobileId, e: &mut muo_server::events::SpeechEventArgs) {
    (BASE_CREATURE.on_speech)(w, g, e);
    let from = e.mobile;
    let loc = w.mobile(from).map(|x| x.location).unwrap_or_default();
    if e.handled
        || !muo_server::mobile::base::in_range_point3d(w, g, loc, 12)
        || !muo_server::mobile::base::alive(w, from)
    {
        return;
    }
    let speech = e.speech.lower_inv();
    let town_of_guard = guard_town(w, g);
    let here = town_of_region(w, g);
    let sheriff = town_of_guard.is_some_and(|t| town::is_sheriff(w, t, from));
    let now = w.now;
    if e.keywords.contains(&0xE6)
        && (speech == "orders"
            || muo_server::mobile::base::name(w, g)
                .is_some_and(|n| speech.starts_with(&n.lower_inv())))
    {
        if !sheriff {
            mm::say_number(w, g, 1042189, ""); // I don't work for you!
        } else if here == town_of_guard {
            mm::say_number(w, g, 1042180, ""); // Your orders, sire?
            if let Some(d) = gd_mut(w, g) {
                d.orders_end = now + TimeSpan::from_secs(10);
            }
        }
    } else if gd(w, g).is_some_and(|d| now < d.orders_end) {
        if !sheriff || here != town_of_guard {
            return;
        }
        if let Some(d) = gd_mut(w, g) {
            d.orders_end = now + TimeSpan::from_secs(10);
        }
        let new_type = if speech.contains("attack") {
            Some(ReactionType::Attack)
        } else if speech.contains("warn") {
            Some(ReactionType::Warn)
        } else if speech.contains("ignore") {
            Some(ReactionType::Ignore)
        } else {
            None
        };
        let mut understood = true;
        if let Some(nt) = new_type {
            understood = false;
            if speech.contains("civil") {
                change_reaction(w, g, None, nt);
                understood = true;
            }
            let own = guard_faction(w, g);
            for (f, def) in FACTION_DEFS.iter().enumerate() {
                if Some(f) != own && speech.contains(def.keyword) {
                    change_reaction(w, g, Some(f), nt);
                    understood = true;
                }
            }
        } else if speech.contains("patrol") {
            let loc = w.mobile(g).map(|x| x.location).unwrap_or_default();
            muo_content::creature::control::set_home(w, g, loc);
            if let Some(c) = muo_content::creature::cd_mut(w, g) {
                c.range_home = 6;
            }
            mx::set_combatant(w, g, None);
            if let Some(d) = gd_mut(w, g) {
                d.orders.movement = MovementType::Patrol;
            }
            mm::say_number(w, g, 1005146, ""); // This spot looks like it needs protection!  I shall guard it with my life.
        } else if speech.contains("follow") {
            let loc = w.mobile(g).map(|x| x.location).unwrap_or_default();
            muo_content::creature::control::set_home(w, g, loc);
            if let Some(c) = muo_content::creature::cd_mut(w, g) {
                c.range_home = 6;
            }
            mx::set_combatant(w, g, None);
            if let Some(d) = gd_mut(w, g) {
                d.orders.follow = Some(from);
                d.orders.movement = MovementType::Follow;
            }
            mm::say_number(w, g, 1005144, ""); // Yes, Sire.
        } else {
            understood = false;
        }
        if !understood {
            mm::say_number(w, g, 1042183, ""); // I'm sorry, I don't understand your orders...
        }
    }
}

/// `Town.FromRegion(mobile.Region)`.
fn town_of_region(w: &World, m: MobileId) -> Option<usize> {
    town::from_region(w, props_b::region(w, m))
}

fn guard_get_properties(
    w: &World,
    g: MobileId,
    list: &mut muo_server::property_list::PropertyList,
) {
    (BASE_CREATURE.get_properties)(w, g, list);
    if let Some(f) = guard_faction(w, g)
        && w.mobile(g).and_then(|m| m.map) == Some(FACET)
    {
        list.add_str(1060846, &super::number_text(FACTION_DEFS[f].prop_name)); // Guard: ~1_val~
    }
}

fn guard_on_single_click(w: &mut World, g: MobileId, from: MobileId) {
    if let Some(f) = guard_faction(w, g)
        && w.mobile(g).and_then(|m| m.map) == Some(FACET)
    {
        let text = format!("(Guard, {})", FACTION_DEFS[f].friendly_name);
        let hue = if find(w, from) == Some(f) { 98 } else { 38 };
        let ns = w.mobile(from).and_then(|m| m.net_state);
        mm::private_overhead_message(w, g, MessageType::LABEL, hue, true, &text, ns);
    }
    (BASE_CREATURE.on_single_click)(w, g, from);
}

/// `BaseFactionGuard.OnDeath(c)`: no corpse is left.
fn guard_on_death(w: &mut World, g: MobileId, c: Option<muo_server::ids::ItemId>) {
    (BASE_CREATURE.on_death)(w, g, c);
    if let Some(c) = c
        && let Some(t) = w.item(c).map(|i| i.ty)
    {
        (t.delete)(w, c);
    }
}

fn guard_on_after_delete(w: &mut World, g: MobileId) {
    (BASE_CREATURE.on_after_delete)(w, g);
    guard_unregister(w, g);
}

fn guard_serialize(w: &World, g: MobileId, out: &mut dyn GenericWriter) {
    (BASE_CREATURE.serialize)(w, g, out);
    out.write_i32(0); // version
    let Some(d) = gd(w, g) else { return };
    write_faction_ref(out, d.faction);
    write_town_ref(out, d.town);
    // Orders.Serialize
    out.write_encoded_int(1);
    out.write_mobile(w, d.orders.follow);
    out.write_encoded_int(d.orders.reactions.len() as i32);
    for &(f, t) in &d.orders.reactions {
        out.write_encoded_int(0); // Reaction version
        write_faction_ref(out, f);
        out.write_encoded_int(t as i32);
    }
    out.write_encoded_int(d.orders.movement as i32);
}

fn guard_deserialize(w: &mut World, g: MobileId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (BASE_CREATURE.deserialize)(w, g, r)?;
    let flags = guard_flags(w, g);
    let _version = r.read_i32()?;
    let faction = read_faction_ref(r)?;
    let town_ = read_town_ref(r)?;
    let mut orders = Orders::new();
    let ov = r.read_encoded_int()?;
    if ov == 1 {
        orders.follow = r.read_mobile(w)?;
    }
    if ov == 0 || ov == 1 {
        let n = r.read_encoded_int()?;
        for _ in 0..n {
            let _rv = r.read_encoded_int()?;
            let f = read_faction_ref(r)?;
            let t = match r.read_encoded_int()? {
                1 => ReactionType::Warn,
                2 => ReactionType::Attack,
                _ => ReactionType::Ignore,
            };
            orders.reactions.push((f, t));
        }
        orders.movement = match r.read_encoded_int()? {
            0 => MovementType::Stand,
            2 => MovementType::Follow,
            _ => MovementType::Patrol,
        };
    }
    if let Some(m) = w.mobile_mut(g) {
        m.ext.insert(GuardData {
            faction,
            town: town_,
            orders,
            orders_end: DateTime::MIN,
            flags,
        });
    }
    // `Timer.StartTimer(Register)`
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        guard_register(w, g)
    });
    Ok(())
}

// ---- FactionGuardAI ----

/// `FactionGuardAI.RunTo(m)`.
fn run_to(w: &mut World, g: MobileId, m: MobileId) {
    let range_fight = muo_content::creature::cd(w, g).map_or(1, |d| d.range_fight);
    let loc = w.mobile(m).map(|x| x.location).unwrap_or_default();
    if !muo_server::mobile::base::in_range_point3d(w, g, loc, range_fight) {
        if !ai_move::move_to(w, g, m, 1) {
            on_failed_move(w, g);
        }
    } else if muo_server::mobile::base::in_range_point3d(w, g, loc, range_fight - 1) {
        run_from(w, g, m);
    }
}

/// `RunFrom(m)`.
fn run_from(w: &mut World, g: MobileId, m: MobileId) {
    let d = mx::get_direction_to(w, g, m);
    run(
        w,
        g,
        Direction::from_bits_truncate((d.bits().wrapping_sub(4)) & Direction::MASK.bits()),
    );
}

/// `Run(d)`.
fn run(w: &mut World, g: MobileId, d: Direction) {
    let (paralyzed, frozen) = w
        .mobile(g)
        .map_or((false, false), |m| (m.paralyzed, m.frozen));
    let casting = w
        .mobile(g)
        .and_then(|m| m.spell)
        .is_some_and(|sp| muo_content::spells::spell::is_casting(w, sp));
    let no_moves =
        muo_content::creature::vtable::ctype(w, g).is_some_and(|c| (c.disallow_all_moves)(w, g));
    if casting || paralyzed || frozen || no_moves {
        return;
    }
    let dir = d | Direction::RUNNING;
    muo_server::mobile::part1::props_b::set_direction(w, g, dir);
    if !ai_move::do_move(w, g, dir, true) {
        on_failed_move(w, g);
    }
}

/// `OnFailedMove()`.
fn on_failed_move(w: &mut World, g: MobileId) {
    let (rp, mode) = muo_content::creature::cd(w, g)
        .map_or((0, FightMode::None), |d| (d.range_perception, d.fight_mode));
    if ai_base::acquire_focus_mob(w, g, rp, mode, false, false, true) {
        let f = muo_content::creature::cd(w, g).and_then(|d| d.focus_mob);
        mx::set_combatant(w, g, f);
        set_action(w, g, ActionType::Combat);
    } else {
        ai_base::debug_say(w, g, "I am stuck");
    }
}

/// `FactionGuardAI.Think()`.
fn guard_think(w: &mut World, g: MobileId) -> bool {
    if mx::deleted(w, g) {
        return false;
    }
    let (rp, mode) = muo_content::creature::cd(w, g)
        .map_or((0, FightMode::None), |d| (d.range_perception, d.fight_mode));
    let mut combatant = mx::combatant(w, g);
    let bad = match combatant {
        None => true,
        Some(c) => {
            mx::deleted(w, c)
                || !mx::alive(w, c)
                || w.mobile(c).is_some_and(|m| (m.ty.is_dead_bonded_pet)(w, c))
                || !muo_server::mobile::base::can_see_mobile(w, g, c)
                || !mx::can_be_harmful_quiet(w, g, c)
                || mx::map(w, c) != mx::map(w, g)
        }
    };
    if bad {
        if ai_base::acquire_focus_mob(w, g, rp, mode, false, false, true) {
            combatant = muo_content::creature::cd(w, g).and_then(|d| d.focus_mob);
            mx::set_combatant(w, g, combatant);
            if let Some(d) = muo_content::creature::cd_mut(w, g) {
                d.focus_mob = None;
            }
        } else {
            combatant = None;
            mx::set_combatant(w, g, None);
        }
    }
    if let Some(c) = combatant {
        let loc = w.mobile(c).map(|x| x.location).unwrap_or_default();
        if !mx::in_los(w, g, c) || !muo_server::mobile::base::in_range_point3d(w, g, loc, 12) {
            if ai_base::acquire_focus_mob(w, g, rp, mode, false, false, true) {
                combatant = muo_content::creature::cd(w, g).and_then(|d| d.focus_mob);
                mx::set_combatant(w, g, combatant);
                if let Some(d) = muo_content::creature::cd_mut(w, g) {
                    d.focus_mob = None;
                }
            } else if !muo_server::mobile::base::in_range_point3d(w, g, loc, 36) {
                combatant = None;
                mx::set_combatant(w, g, None);
            }
        }
    }

    let dispel = super::guard_ai::before_movement(w, g, combatant);
    let movement = gd(w, g).map_or(MovementType::Stand, |d| d.orders.movement);
    if let Some(d) = dispel {
        if muo_content::ai::action(w, g) != Some(ActionType::Combat) {
            set_action(w, g, ActionType::Combat);
        }
        mx::set_warmode(w, g, true);
        run_from(w, g, d);
    } else if let Some(c) = combatant {
        if muo_content::ai::action(w, g) != Some(ActionType::Combat) {
            set_action(w, g, ActionType::Combat);
        }
        mx::set_warmode(w, g, true);
        run_to(w, g, c);
    } else if movement != MovementType::Stand {
        let mut to_follow = None;
        if let Some(t) = guard_town(w, g)
            && movement == MovementType::Follow
        {
            to_follow = gd(w, g)
                .and_then(|d| d.orders.follow)
                .or_else(|| st(w).and_then(|s| s.towns[t].sheriff));
        }
        let follow_ok = to_follow.filter(|&f| {
            mx::map(w, f) == mx::map(w, g)
                && {
                    let loc = w.mobile(g).map(|x| x.location).unwrap_or_default();
                    muo_server::mobile::base::in_range_point3d(w, f, loc, rp * 3)
                }
                && town_of_region(w, f) == guard_town(w, g)
        });
        if let Some(f) = follow_ok {
            if muo_content::ai::action(w, g) != Some(ActionType::Combat) {
                set_action(w, g, ActionType::Combat);
            }
            muo_content::creature::speeds::set_current_speed_to_active(w, g);
            mx::set_warmode(w, g, true);
            run_to(w, g, f);
        } else {
            if muo_content::ai::action(w, g) != Some(ActionType::Wander) {
                set_action(w, g, ActionType::Wander);
            }
            muo_content::creature::speeds::set_current_speed_to_passive(w, g);
            mx::set_warmode(w, g, false);
            ai_move::walk_random_in_home(w, g, 2, 2, 1);
        }
    } else {
        if muo_content::ai::action(w, g) != Some(ActionType::Wander) {
            set_action(w, g, ActionType::Wander);
        }
        mx::set_warmode(w, g, false);
    }
    super::guard_ai::spells(w, g, combatant, dispel);
    true
}

static FACTION_GUARD_AI: AiClass = AiClass {
    name: "FactionGuardAI",
    think: guard_think,
    ..ai_base::BASE_AI
};

// ---- guard classes ----

macro_rules! guard_class {
    ($name:ident, $class:ident, $full:literal) => {
        static $class: CreatureType = CreatureType {
            bard_immune: |_, _| true,
            reacquire_delay: |_, _| TimeSpan::from_secs(2),
            forced_ai: |_, _| Some(&FACTION_GUARD_AI),
            is_enemy: guard_is_enemy,
            ..BASE_CREATURE_C
        };
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&BASE_CREATURE),
            construct: Some(|w| new_guard(w, &$name)),
            family: Some(&$class),
            serialize: |w, id, out| {
                guard_serialize(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                guard_deserialize(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            on_movement: guard_on_movement,
            handles_on_speech: guard_handles_on_speech,
            on_speech: guard_on_speech,
            get_properties: guard_get_properties,
            on_single_click: guard_on_single_click,
            on_after_delete: guard_on_after_delete,
            click_title: |_, _| false,
            on_death: guard_on_death,
            ..BASE_CREATURE
        };
    };
}

guard_class!(
    FACTION_KNIGHT,
    FACTION_KNIGHT_C,
    "Server.Factions.FactionKnight"
);
guard_class!(
    FACTION_BERSERKER,
    FACTION_BERSERKER_C,
    "Server.Factions.FactionBerserker"
);
guard_class!(
    FACTION_DEATH_KNIGHT,
    FACTION_DEATH_KNIGHT_C,
    "Server.Factions.FactionDeathKnight"
);
guard_class!(
    FACTION_DRAGOON,
    FACTION_DRAGOON_C,
    "Server.Factions.FactionDragoon"
);
guard_class!(
    FACTION_HENCHMAN,
    FACTION_HENCHMAN_C,
    "Server.Factions.FactionHenchman"
);
guard_class!(
    FACTION_MERCENARY,
    FACTION_MERCENARY_C,
    "Server.Factions.FactionMercenary"
);
guard_class!(
    FACTION_NECROMANCER,
    FACTION_NECROMANCER_C,
    "Server.Factions.FactionNecromancer"
);
guard_class!(
    FACTION_PALADIN,
    FACTION_PALADIN_C,
    "Server.Factions.FactionPaladin"
);
guard_class!(
    FACTION_SORCERESS,
    FACTION_SORCERESS_C,
    "Server.Factions.FactionSorceress"
);
guard_class!(
    FACTION_WIZARD,
    FACTION_WIZARD_C,
    "Server.Factions.FactionWizard"
);

// ---- vendors ----

fn buy_items(
    items: &'static [(&'static str, i32, i32)],
    repeat: usize,
) -> impl Fn(&mut World, MobileId, &mut SbInstance) {
    move |_w, _v, sb| {
        for _ in 0..repeat {
            for &(ty, price, item_id) in items {
                sb.buy.push(BuyInfo::new(None, ty, price, 20, item_id, 0));
            }
        }
    }
}

static SB_BOTTLE: SbInfo = SbInfo {
    name: "SBFactionBottle",
    build: |w, v, sb| buy_items(&[("Server.Items.Bottle", 5, 0xF0E)], 5)(w, v, sb),
};
static SB_BOARD: SbInfo = SbInfo {
    name: "SBFactionBoard",
    build: |w, v, sb| buy_items(&[("Server.Items.Board", 3, 0x1BD7)], 5)(w, v, sb),
};
static SB_ORE: SbInfo = SbInfo {
    name: "SBFactionOre",
    build: |_w, _v, sb| {
        for _ in 0..5 {
            // `new object[] { true }`: a fixed-size IronOre.
            sb.buy
                .push(BuyInfo::new(None, "Server.Items.IronOre", 16, 20, 0x19B8, 0).with_arg(1));
        }
    },
};
static SB_REAGENT: SbInfo = SbInfo {
    name: "SBFactionReagent",
    build: |w, v, sb| {
        buy_items(
            &[
                ("Server.Items.BlackPearl", 5, 0xF7A),
                ("Server.Items.Bloodmoss", 5, 0xF7B),
                ("Server.Items.MandrakeRoot", 3, 0xF86),
                ("Server.Items.Garlic", 3, 0xF84),
                ("Server.Items.Ginseng", 3, 0xF85),
                ("Server.Items.Nightshade", 3, 0xF88),
                ("Server.Items.SpidersSilk", 3, 0xF8D),
                ("Server.Items.SulfurousAsh", 3, 0xF8C),
            ],
            2,
        )(w, v, sb)
    },
};

/// `new FactionXVendor(town, faction)` up to the town and faction (`VendorList.Construct` sets those): the
/// `BaseFactionVendor` statements and the class's own are its template.
fn new_faction_vendor(w: &mut World, ty: &'static MobileType) -> MobileId {
    let id = muo_content::vendor::new_vendor(w, ty, None);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(FactionVendorData::default());
    }
    muo_content::templates::apply_mobile(w, id);
    id
}

macro_rules! vendor_class {
    ($name:ident, $class:ident, $vclass:ident, $full:literal) => {
        pub static $class: CreatureType = CreatureType {
            subclass: Some(&$vclass),
            ..BASE_VENDOR_C
        };
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&BASE_VENDOR),
            construct: None, // `FactionVendor(town, faction)` has no parameterless constructor
            make: Some(|w| new_faction_vendor(w, &$name)),
            family: Some(&$class),
            serialize: |w, id, out| {
                (BASE_VENDOR.serialize)(w, id, out);
                out.write_i32(0); // generated version
                faction_vendor_serialize(w, id, out);
            },
            deserialize: |w, id, r| {
                (BASE_VENDOR.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                faction_vendor_deserialize(w, id, r)
            },
            on_after_delete: |w, id| {
                (BASE_VENDOR.on_after_delete)(w, id);
                vendor_unregister(w, id);
            },
            // `BaseFactionVendor.OnMoveOver`: from ML on anyone may walk through a faction vendor.
            on_move_over: |w, id, m| {
                muo_content::core::ml(w) || (BASE_VENDOR.on_move_over)(w, id, m)
            },
            ..BASE_VENDOR
        };
    };
}

fn faction_vendor_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0); // BaseFactionVendor version
    write_town_ref(out, vendor_town(w, id));
    write_faction_ref(out, vendor_faction(w, id));
}

fn faction_vendor_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let version = r.read_i32()?;
    let mut d = FactionVendorData::default();
    if version == 0 {
        d.town = read_town_ref(r)?;
        d.faction = read_faction_ref(r)?;
    }
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(FactionVendorData::default());
    }
    set_vendor_town_faction(w, id, d.town, d.faction);
    props_a::set_frozen(w, id, true);
    Ok(())
}

static BOTTLE_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SB_BOTTLE.new_for(w, id)],
    shoe_type: TEMPLATE_VENDOR_V.shoe_type,
    init_outfit: TEMPLATE_VENDOR_V.init_outfit,
    check_vendor_access: |_, _, _| true,
    ..BASE_VENDOR_CLASS
};
static BOARD_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SB_BOARD.new_for(w, id)],
    init_outfit: TEMPLATE_VENDOR_V.init_outfit,
    check_vendor_access: |_, _, _| true,
    ..BASE_VENDOR_CLASS
};
static ORE_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SB_ORE.new_for(w, id)],
    init_outfit: TEMPLATE_VENDOR_V.init_outfit,
    check_vendor_access: |_, _, _| true,
    ..BASE_VENDOR_CLASS
};
static REAGENT_V: VendorClass = VendorClass {
    init_sb_info: |w, id| vec![SB_REAGENT.new_for(w, id)],
    shoe_type: TEMPLATE_VENDOR_V.shoe_type,
    init_outfit: TEMPLATE_VENDOR_V.init_outfit,
    check_vendor_access: |_, _, _| true,
    ..BASE_VENDOR_CLASS
};
static HORSE_V: VendorClass = VendorClass {
    init_sb_info: |_, _| Vec::new(),
    shoe_type: TEMPLATE_VENDOR_V.shoe_type,
    get_shoe_hue: TEMPLATE_VENDOR_V.get_shoe_hue,
    init_outfit: TEMPLATE_VENDOR_V.init_outfit,
    vendor_buy: |w, id, from| {
        let f = vendor_faction(w, id);
        let Some(f) = f.filter(|_| find(w, from) == f) else {
            let ns = w.mobile(from).and_then(|m| m.net_state);
            // You are not in my faction, I cannot sell you a horse!
            mm::private_overhead_message_number(
                w,
                id,
                MessageType::REGULAR,
                0x3B2,
                1042201,
                "",
                ns,
            );
            return;
        };
        if super::items::faction_gump_exists(w, from) {
            mm::send_localized_message(w, from, 1042160, "", 0x3B2); // You already have a faction menu open.
        } else if muo_content::player_mobile::data(w, from).is_some() {
            super::gumps::send_horse_breeder_gump(w, from, f);
        }
    },
    vendor_sell: |_, _, _| {},
    on_buy_items: |_, _, _, _| false,
    on_sell_items: |_, _, _, _| false,
    check_vendor_access: |_, _, _| true,
    ..BASE_VENDOR_CLASS
};

vendor_class!(
    FACTION_BOTTLE_VENDOR,
    FACTION_BOTTLE_VENDOR_C,
    BOTTLE_V,
    "Server.Factions.FactionBottleVendor"
);
vendor_class!(
    FACTION_BOARD_VENDOR,
    FACTION_BOARD_VENDOR_C,
    BOARD_V,
    "Server.Factions.FactionBoardVendor"
);
vendor_class!(
    FACTION_ORE_VENDOR,
    FACTION_ORE_VENDOR_C,
    ORE_V,
    "Server.Factions.FactionOreVendor"
);
vendor_class!(
    FACTION_REAGENT_VENDOR,
    FACTION_REAGENT_VENDOR_C,
    REAGENT_V,
    "Server.Factions.FactionReagentVendor"
);
vendor_class!(
    FACTION_HORSE_VENDOR,
    FACTION_HORSE_VENDOR_C,
    HORSE_V,
    "Server.Factions.FactionHorseVendor"
);

// ---- FactionWarHorse ----

pub fn war_horse_set_faction(w: &mut World, id: MobileId, f: Option<usize>) {
    if let Some(d) = w
        .mobile_mut(id)
        .and_then(|m| m.ext.get_mut::<WarHorseData>())
    {
        d.faction = f;
    }
    props_b::set_body(
        w,
        id,
        muo_primitives::Body(f.map_or(0xE2, |f| FACTION_DEFS[f].war_horse_body)),
    );
    muo_content::creature::mount::set_item_id(
        w,
        id,
        f.map_or(0x3EA0, |f| FACTION_DEFS[f].war_horse_item),
    );
}

/// `new FactionWarHorse(faction)`.
pub fn new_war_horse(w: &mut World, f: Option<usize>) -> Option<MobileId> {
    let id = muo_content::factory::make_mobile(w, "Server.Factions.FactionWarHorse")?;
    war_horse_set_faction(w, id, f);
    Some(id)
}

fn war_horse_construct(w: &mut World) -> MobileId {
    let id = muo_mobiles::cprops::new_mount(w, &FACTION_WAR_HORSE);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(WarHorseData::default());
    }
    id
}

static FACTION_WAR_HORSE_C: CreatureType = CreatureType {
    default_name: muo_content::templates::creature_props::default_name,
    corpse_name: muo_content::templates::creature_props::corpse_name,
    steps_max: muo_content::templates::creature_props::steps_max,
    favorite_food: muo_mobiles::cprops::favorite_food,
    ..BASE_MOUNT_C
};

pub static FACTION_WAR_HORSE: MobileType = MobileType {
    full_name: "Server.Factions.FactionWarHorse",
    parent: Some(&BASE_MOUNT),
    construct: Some(war_horse_construct),
    family: Some(&FACTION_WAR_HORSE_C),
    serialize: |w, id, out| {
        (BASE_MOUNT.serialize)(w, id, out);
        out.write_i32(0);
        write_faction_ref(
            out,
            w.mobile(id)
                .and_then(|m| m.ext.get::<WarHorseData>())
                .and_then(|d| d.faction),
        );
    },
    deserialize: |w, id, r| {
        (BASE_MOUNT.deserialize)(w, id, r)?;
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(WarHorseData::default());
        }
        let version = r.read_i32()?;
        if version == 0 {
            let f = read_faction_ref(r)?;
            war_horse_set_faction(w, id, f);
        }
        Ok(())
    },
    on_double_click: |w, id, from| {
        let own = w
            .mobile(id)
            .and_then(|m| m.ext.get::<WarHorseData>())
            .and_then(|d| d.faction);
        let msg = |w: &mut World, n: i32| mm::send_localized_message(w, from, n, "", 0x3B2);
        match super::player_state(w, from) {
            None => msg(w, 1010366), // You cannot mount a faction war horse!
            Some((pf, _)) if Some(pf) != own => msg(w, 1010367), // You cannot ride an opposing faction's war horse!
            Some((pf, p)) if FACTION_DEFS[pf].ranks[p.rank].rank < 2 => {
                msg(w, 1010368); // You must achieve a faction rank of at least two before riding a war horse!
            }
            Some(_) => (BASE_MOUNT.on_double_click)(w, id, from),
        }
    },
    ..BASE_MOUNT
};

// ---- setup ----

/// Nothing runs at `Initialize`: guards register themselves when they load.
pub fn initialize(_w: &mut World) {
    let _ = OrderType::Follow;
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &FACTION_KNIGHT,
        &FACTION_BERSERKER,
        &FACTION_DEATH_KNIGHT,
        &FACTION_DRAGOON,
        &FACTION_HENCHMAN,
        &FACTION_MERCENARY,
        &FACTION_NECROMANCER,
        &FACTION_PALADIN,
        &FACTION_SORCERESS,
        &FACTION_WIZARD,
    ] {
        types.register_mobile(t, &[]);
    }
    for t in [
        &FACTION_BOTTLE_VENDOR,
        &FACTION_BOARD_VENDOR,
        &FACTION_ORE_VENDOR,
        &FACTION_REAGENT_VENDOR,
        &FACTION_HORSE_VENDOR,
        &FACTION_WAR_HORSE,
    ] {
        types.register_mobile(t, &[]);
    }
    let _ = TOWN_DEFS.len();
}
