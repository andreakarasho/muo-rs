//! Virtues: `VirtueSystem` (levels, awards, atrophy, the five-minute timer), the virtues
//! (`CompassionVirtue`, `SacrificeVirtue`, `ValorVirtue`, `JusticeVirtue` + `AcceptProtectorGump`, `HonorVirtue` +
//! `HonorSelfGump`), `HonorContext`, `VirtueGump`, `VirtueInfoGump` and `VirtueStatusGump`.
//!
//! The persistent `VirtueContext` and the `Virtues` save file are in `muo_content::player_mobile::tables`
//! (`VirtueTable`); this module is their behavior. `IHonorTarget.ReceivedHonorContext` and
//! `PlayerMobile.SentHonorContext` live in [`HonorState`].
//!
//! shortcut: Valor's champion idol and spawn (Engines/CannedEvil, another task) are [`ChampionHooks`]; unset, an idol
//! is "not a Champion Idol".

use indexmap::IndexMap;
use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::{DynamicGumpBuilder, GumpStrings, StaticGumpBuilder, send_gump_to_mobile};
use muo_content::mx;
use muo_content::network::incoming_game::GameHooks;
use muo_content::player_mobile::tables::{VirtueContext, VirtueTable, virtues_mut};
use muo_content::player_mobile::{self, hooks as phooks};
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::part1::{combat, props_b};
use muo_server::mobile::part2::messages::{send_localized_message, send_message_hue};
use muo_server::net_state::NetStateId;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};

/// `VirtueLevel`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VirtueLevel {
    None = 0,
    Seeker = 1,
    Follower = 2,
    Knight = 3,
}

/// `VirtueName`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum VirtueName {
    Humility = 0,
    Sacrifice = 1,
    Compassion = 2,
    Spirituality = 3,
    Valor = 4,
    Honor = 5,
    Justice = 6,
    Honesty = 7,
}

impl VirtueName {
    pub fn from_i32(i: i32) -> Option<VirtueName> {
        use VirtueName::*;
        [
            Humility,
            Sacrifice,
            Compassion,
            Spirituality,
            Valor,
            Honor,
            Justice,
            Honesty,
        ]
        .get(usize::try_from(i).ok()?)
        .copied()
    }
}

/// Calls into the Champion Spawn engine (`IdolOfTheChampion`, `ChampionSpawn`), set by that engine.
#[derive(Default)]
pub struct ChampionHooks {
    /// For an `IdolOfTheChampion` that is not deleted and has a live spawn: the spawn's state.
    pub idol_spawn: Option<fn(&World, ItemId) -> Option<IdolSpawn>>,
    /// `idol.Spawn.HasBeenAdvanced = true; idol.Spawn.AdvanceLevel()`.
    pub advance_level: Option<fn(&mut World, ItemId)>,
    /// `idol.Spawn.Start(); idol.Spawn.ActivatedByValor = true`.
    pub start_by_valor: Option<fn(&mut World, ItemId)>,
    /// `from.SendGump(new ResurrectGump(from, fromSacrifice: true))` (`muo_gumps`; set by
    /// `glue::configure`).
    pub sacrifice_resurrect_gump: Option<fn(&mut World, MobileId)>,
}

/// What `ValorVirtue.Valor` reads of `idol.Spawn`.
#[derive(Copy, Clone, Debug)]
pub struct IdolSpawn {
    pub has_been_advanced: bool,
    pub active: bool,
    pub champion_spawned: bool,
    pub sub_level: i32,
}

/// `HonorContext`.
#[derive(Debug)]
pub struct HonorContext {
    pub source: MobileId,
    pub target: MobileId,
    initial_location: Point3D,
    initial_map: Option<MapId>,
    first_hit: FirstHit,
    honor_damage: f64,
    poisoned: bool,
    total_damage: i32,
    pub perfection_damage_bonus: i32,
    timer: TimerHandle,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum FirstHit {
    NotDelivered,
    Delivered,
    Granted,
}

impl HonorContext {
    /// `PerfectionLuckBonus`.
    pub fn perfection_luck_bonus(&self) -> i32 {
        self.perfection_damage_bonus * self.perfection_damage_bonus / 10
    }
}

/// `PlayerMobile.SentHonorContext`, `IHonorTarget.ReceivedHonorContext` and `_honorTime`.
#[derive(Default)]
pub struct HonorState {
    contexts: IndexMap<u32, HonorContext>,
    sent: IndexMap<MobileId, u32>,
    received: IndexMap<MobileId, u32>,
    honor_time: IndexMap<MobileId, DateTime>,
    next: u32,
}

fn hs(w: &mut World) -> &mut HonorState {
    w.resources.get_mut::<HonorState>()
}

fn hs_ref(w: &World) -> Option<&HonorState> {
    w.resources.get::<HonorState>()
}

/// `VirtueGump._callbacks`.
#[derive(Default)]
pub struct VirtueCallbacks {
    callbacks: IndexMap<i32, fn(&mut World, MobileId)>,
}

/// `VirtueGump.Register(gumpID, callback)`.
pub fn register_virtue(w: &mut World, gump_id: i32, callback: fn(&mut World, MobileId)) {
    w.resources
        .get_mut::<VirtueCallbacks>()
        .callbacks
        .insert(gump_id, callback);
}

// ---- VirtueSystem ------------------------------------------------------------------------------------------------

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.ty.is(&player_mobile::PLAYER_MOBILE))
}

/// `VirtueSystem.GetVirtues(from)`.
pub fn get_virtues(w: &World, from: MobileId) -> Option<&VirtueContext> {
    w.resources.get::<VirtueTable>()?.0.get(&from)
}

fn get_value(w: &World, from: MobileId, virtue: VirtueName) -> i32 {
    get_virtues(w, from)
        .and_then(|v| v.values.as_ref())
        .and_then(|v| v.get(virtue as usize))
        .copied()
        .unwrap_or(0)
}

/// `VirtueSystem.GetMaxAmount`.
pub fn get_max_amount(v: VirtueName) -> i32 {
    match v {
        VirtueName::Honor => 20000,
        VirtueName::Sacrifice => 22000,
        _ => 21000,
    }
}

/// `VirtueSystem.IsHighestPath`.
pub fn is_highest_path(w: &World, from: MobileId, v: VirtueName) -> bool {
    get_virtues(w, from)
        .is_some_and(|c| get_value(w, from, v) >= get_max_amount(v) && c.values.is_some())
}

/// `VirtueSystem.GetLevel`.
pub fn get_level(w: &World, from: MobileId, v: VirtueName) -> VirtueLevel {
    let val = if is_player(w, from) {
        get_value(w, from, v)
    } else {
        0
    };
    let vl = if val < 4000 {
        0
    } else if val >= get_max_amount(v) {
        3
    } else {
        (val + 9999) / 10000
    };
    match vl {
        0 => VirtueLevel::None,
        1 => VirtueLevel::Seeker,
        2 => VirtueLevel::Follower,
        _ => VirtueLevel::Knight,
    }
}

pub fn get_name(v: VirtueName) -> &'static str {
    [
        "Humility",
        "Sacrifice",
        "Compassion",
        "Spirituality",
        "Valor",
        "Honor",
        "Justice",
        "Honesty",
    ][v as usize]
}

pub fn get_lower_case_name(v: VirtueName) -> &'static str {
    [
        "humility",
        "sacrifice",
        "compassion",
        "spirituality",
        "valor",
        "honor",
        "justice",
        "honesty",
    ][v as usize]
}

pub fn get_gained_localized_message(v: VirtueName) -> i32 {
    match v {
        VirtueName::Sacrifice => 1054160,
        VirtueName::Compassion => 1053002,
        VirtueName::Spirituality => 1155832,
        VirtueName::Valor => 1054030,
        VirtueName::Honor => 1063225,
        VirtueName::Justice => 1049363,
        VirtueName::Humility => 1052070,
        VirtueName::Honesty => 0,
    }
}

pub fn get_gained_a_path_localized_message(v: VirtueName) -> i32 {
    match v {
        VirtueName::Sacrifice => 1052008,
        VirtueName::Spirituality => 1155833, // "You have gained a path in Spirituality!" (Why are there quotes?)
        VirtueName::Valor => 1054032,
        VirtueName::Honor => 1063226,
        VirtueName::Justice => 1049367,
        VirtueName::Humility => 1155811,
        _ => 0,
    }
}

pub fn get_highest_path_localized_message(v: VirtueName) -> i32 {
    match v {
        VirtueName::Compassion => 1053003,
        VirtueName::Spirituality => 1155831,
        VirtueName::Valor => 1054031,
        VirtueName::Honor => 1063228,
        VirtueName::Justice => 1049534,
        VirtueName::Humility => 1155808,
        VirtueName::Honesty => 1153771,
        _ => 1052050,
    }
}

/// `VirtueSystem.Award(from, virtue, amount, ref gainedPath)`: Some(gained_path) when anything was awarded.
pub fn award(w: &mut World, from: MobileId, virtue: VirtueName, amount: i32) -> Option<bool> {
    let current = get_value(w, from, virtue);
    let max = get_max_amount(virtue);
    if current >= max {
        return None;
    }
    let amount = if current + amount >= max {
        max - current
    } else {
        amount
    };
    let old = get_level(w, from, virtue);
    virtues_mut(w, from).set_value(virtue as usize, current + amount);
    Some(get_level(w, from, virtue) != old)
}

/// `VirtueSystem.Atrophy(from, virtue, amount)`.
pub fn atrophy(w: &mut World, from: MobileId, virtue: VirtueName, amount: i32) -> bool {
    if get_virtues(w, from).is_none() {
        return false;
    }
    let current = get_value(w, from, virtue);
    let new = if current - amount >= 0 {
        current - amount
    } else {
        0
    };
    virtues_mut(w, from).set_value(virtue as usize, new);
    current > 0
}

pub fn is_seeker(w: &World, from: MobileId, v: VirtueName) -> bool {
    get_level(w, from, v) >= VirtueLevel::Seeker
}
pub fn is_follower(w: &World, from: MobileId, v: VirtueName) -> bool {
    get_level(w, from, v) >= VirtueLevel::Follower
}
pub fn is_knight(w: &World, from: MobileId, v: VirtueName) -> bool {
    get_level(w, from, v) >= VirtueLevel::Knight
}

/// `VirtueSystem.AwardVirtue(pm, virtue, amount)`.
pub fn award_virtue(w: &mut World, pm: MobileId, virtue: VirtueName, amount: i32) {
    virtues_mut(w, pm);
    let now = w.now;
    if virtue == VirtueName::Compassion {
        let v = virtues_mut(w, pm);
        if v.compassion_gains > 0 && now > v.next_compassion_day {
            v.next_compassion_day = DateTime::MIN;
            v.compassion_gains = 0;
        }
        if v.compassion_gains >= 5 {
            send_localized_message(w, pm, 1053004, "", 0x3B2); // You must wait about a day before you can gain in compassion again.
            return;
        }
    }
    let name = get_name(virtue);
    match award(w, pm, virtue, amount) {
        Some(gained_path) => {
            if gained_path {
                let m = get_gained_a_path_localized_message(virtue);
                if m != 0 {
                    send_localized_message(w, pm, m, "", 0x3B2);
                } else {
                    mx::send_message(w, pm, &format!("You have gained a path in {name}!"));
                }
            } else {
                let m = get_gained_localized_message(virtue);
                if m != 0 {
                    send_localized_message(w, pm, m, "", 0x3B2);
                } else {
                    mx::send_message(w, pm, &format!("You have gained in {name}."));
                }
            }
            if virtue == VirtueName::Compassion {
                let v = virtues_mut(w, pm);
                v.next_compassion_day = now + TimeSpan::from_days(1);
                v.compassion_gains += 1;
                if v.compassion_gains >= 5 {
                    send_localized_message(w, pm, 1053004, "", 0x3B2);
                }
            }
        }
        None => {
            send_localized_message(w, pm, get_highest_path_localized_message(virtue), "", 0x3B2)
        }
    }
}

/// `VirtueSystem.CheckAtrophies(pm)`.
pub fn check_atrophies(w: &mut World, pm: MobileId) {
    sacrifice_check_atrophy(w, pm);
    justice_check_atrophy(w, pm);
    compassion_check_atrophy(w, pm);
    valor_check_atrophy(w, pm);
}

const FIVE_MINUTES: TimeSpan = TimeSpan::from_mins(5);

/// `VirtueTimer.Initialize`.
pub fn initialize(w: &mut World) {
    // `VirtueSystem.Initialize` (the Mobile.VirtueMigrations) is run by the content crate with the tables.
    let h = timer::start(w, FIVE_MINUTES, FIVE_MINUTES, 0, |w| {
        let players: Vec<MobileId> = match w.resources.get::<VirtueTable>() {
            Some(t) if !t.0.is_empty() => t.0.keys().copied().collect(),
            _ => return,
        };
        for p in players {
            check_atrophies(w, p);
            let now = w.now;
            let unused = w
                .resources
                .get::<VirtueTable>()
                .and_then(|t| t.0.get(&p))
                .is_some_and(|v| !v.is_used(now));
            if unused {
                w.resources.get_mut::<VirtueTable>().0.shift_remove(&p);
            }
        }
    });
    let _ = h;
    // Compassion.Initialize, Sacrifice.Initialize, Justice.Initialize, Honor.Initialize, Valor.Initialize
    register_virtue(w, 105, compassion_on_virtue_used);
    register_virtue(w, 110, sacrifice_on_virtue_used);
    register_virtue(w, 109, justice_on_virtue_used);
    register_virtue(w, 107, honor_on_virtue_used);
    register_virtue(w, 112, valor_on_virtue_used);
}

// ---- CompassionVirtue --------------------------------------------------------------------------------------------

fn compassion_on_virtue_used(w: &mut World, from: MobileId) {
    send_localized_message(w, from, 1053001, "", 0x3B2); // This virtue is not activated through the virtue menu.
}

fn compassion_check_atrophy(w: &mut World, pm: MobileId) {
    let can = get_virtues(w, pm).is_some_and(|v| {
        v.last_compassion_loss
            .checked_add(TimeSpan::from_days(7))
            .is_none_or(|t| t < w.now)
    });
    if get_virtues(w, pm).is_some() && get_value(w, pm, VirtueName::Compassion) > 0 && can {
        if atrophy(w, pm, VirtueName::Compassion, 500) {
            send_localized_message(w, pm, 1114420, "", 0x3B2); // You have lost some Compassion.
        }
        let now = w.now;
        virtues_mut(w, pm).last_compassion_loss = now;
    }
}

// ---- SacrificeVirtue ---------------------------------------------------------------------------------------------

fn sacrifice_on_virtue_used(w: &mut World, from: MobileId) {
    if !is_player(w, from) {
        return;
    }
    if w.mobile(from).is_some_and(|m| m.hidden) {
        send_localized_message(w, from, 1052015, "", 0x3B2); // You cannot do that while hidden.
    } else if mx::alive(w, from) {
        target::begin_target(w, from, 8, false, TargetFlags::NONE, |w, from, t| {
            if is_player(w, from) {
                sacrifice(w, from, t);
            }
        });
    } else {
        sacrifice_resurrect(w, from);
    }
}

fn sacrifice_can_gain(w: &World, v: &VirtueContext) -> bool {
    v.last_sacrifice_gain
        .checked_add(TimeSpan::from_days(1))
        .is_some_and(|t| w.now >= t)
}

fn sacrifice_can_atrophy(w: &World, v: &VirtueContext) -> bool {
    v.last_sacrifice_loss
        .checked_add(TimeSpan::from_days(7))
        .is_none_or(|t| t < w.now)
}

fn sacrifice_check_atrophy(w: &mut World, pm: MobileId) {
    let ok = get_virtues(w, pm).is_some_and(|v| sacrifice_can_atrophy(w, v));
    if ok && get_value(w, pm, VirtueName::Sacrifice) > 0 {
        if atrophy(w, pm, VirtueName::Sacrifice, 500) {
            send_localized_message(w, pm, 1052041, "", 0x3B2); // You have lost some Sacrifice.
        }
        let level = get_level(w, pm, VirtueName::Sacrifice) as i32;
        let now = w.now;
        let v = virtues_mut(w, pm);
        v.available_resurrects = level;
        v.last_sacrifice_loss = now;
    }
}

/// `SacrificeVirtue.Resurrect(from)`.
pub fn sacrifice_resurrect(w: &mut World, from: MobileId) {
    if mx::alive(w, from) {
        return;
    }
    if muo_server::mobile::base::criminal(w, from) {
        send_localized_message(w, from, 1052007, "", 0x3B2); // You cannot use this ability while flagged as a criminal.
    } else if !is_seeker(w, from, VirtueName::Sacrifice) {
        send_localized_message(w, from, 1052004, "", 0x3B2); // You cannot use this ability.
    } else if get_virtues(w, from).is_some_and(|v| v.available_resurrects > 0) {
        // wait for them to accept the gump or they can just use Sacrifice and cancel to have items for free
        if let Some(f) = w
            .resources
            .get::<ChampionHooks>()
            .and_then(|h| h.sacrifice_resurrect_gump)
        {
            f(w, from);
        }
    } else {
        send_localized_message(w, from, 1052005, "", 0x3B2); // You do not have any resurrections left.
    }
}

fn is_named_class(ty: &'static muo_server::mobile::MobileType, names: &[&str]) -> bool {
    let mut t = Some(ty);
    while let Some(c) = t {
        if names.contains(&c.full_name) {
            return true;
        }
        t = c.parent;
    }
    false
}

/// `SacrificeVirtue.ValidateCreature(m)`.
pub fn sacrifice_validate_creature(w: &World, m: MobileId) -> bool {
    let Some(mm) = w.mobile(m) else { return false };
    let ok_ctl = muo_content::creature::data::cd(w, m).is_none_or(|d| !d.controlled && !d.summoned);
    ok_ctl
        && is_named_class(
            mm.ty,
            &[
                "Server.Mobiles.Lich",
                "Server.Mobiles.Succubus",
                "Server.Mobiles.Daemon",
                "Server.Mobiles.EvilMage",
                "Server.Mobiles.EnslavedGargoyle",
                "Server.Mobiles.GargoyleEnforcer",
            ],
        )
}

/// `SacrificeVirtue.Sacrifice(from, targeted)`.
pub fn sacrifice(w: &mut World, from: MobileId, targeted: Targeted) {
    if !muo_server::mobile::part2::stats::check_alive(w, from, true) {
        return;
    }
    let Targeted::Entity(EntityId::Mobile(targ)) = targeted else {
        return;
    };
    let (hits, hits_max) = (mx::hits(w, targ), mx::hits_max(w, targ));
    let fame = w.mobile(from).map_or(0, |m| m.fame);
    if !sacrifice_validate_creature(w, targ) {
        send_localized_message(w, from, 1052014, "", 0x3B2); // You cannot sacrifice your fame for that creature.
    } else if hits * 100 / hits_max.max(1) < 90 {
        send_localized_message(w, from, 1052013, "", 0x3B2); // too damaged
    } else if w.mobile(from).is_some_and(|m| m.hidden) {
        send_localized_message(w, from, 1052015, "", 0x3B2);
    } else if is_highest_path(w, from, VirtueName::Sacrifice) {
        send_localized_message(w, from, 1052068, "", 0x3B2); // You have already attained the highest path in this virtue.
    } else if fame < 2500 {
        send_localized_message(w, from, 1052017, "", 0x3B2); // You do not have enough fame to sacrifice.
    } else {
        virtues_mut(w, from);
        let can = get_virtues(w, from).is_some_and(|v| sacrifice_can_gain(w, v));
        if !can {
            send_localized_message(w, from, 1052016, "", 0x3B2); // You must wait approximately one day...
        } else {
            let to_gain = if fame < 5000 {
                500
            } else if fame < 10000 {
                1000
            } else {
                2000
            };
            muo_server::mobile::part1::props_a::set_fame(w, from, 0);
            // I have seen the error of my ways!
            muo_server::mobile::part2::messages::public_overhead_message_number(
                w,
                targ,
                muo_primitives::MessageType::REGULAR,
                0x3B2,
                1052009,
                "",
                true,
            );
            send_localized_message(w, from, 1052010, "", 0x3B2); // You have set the creature free.
            timer::start(
                w,
                TimeSpan::from_secs_f64(1.0),
                TimeSpan::ZERO,
                1,
                move |w| {
                    muo_server::mobile::part1::lifecycle::delete(w, targ);
                },
            );
            let now = w.now;
            virtues_mut(w, from).last_sacrifice_gain = now;
            if let Some(gained_path) = award(w, from, VirtueName::Sacrifice, to_gain) {
                if gained_path {
                    send_localized_message(w, from, 1052008, "", 0x3B2); // You have gained a path in Sacrifice!
                    let v = virtues_mut(w, from);
                    if v.available_resurrects < 3 {
                        v.available_resurrects += 1;
                    }
                } else {
                    send_localized_message(w, from, 1054160, "", 0x3B2); // You have gained in sacrifice.
                }
            }
            send_localized_message(w, from, 1052016, "", 0x3B2);
        }
    }
}

// ---- ValorVirtue -------------------------------------------------------------------------------------------------

fn valor_on_virtue_used(w: &mut World, from: MobileId) {
    if mx::alive(w, from) {
        send_localized_message(w, from, 1054034, "", 0x3B2); // Target the Champion Idol of the Champion you wish to challenge!.
        target::begin_target(w, from, 14, false, TargetFlags::NONE, |w, from, t| {
            if is_player(w, from) {
                valor(w, from, t);
            }
        });
    }
}

fn valor_check_atrophy(w: &mut World, pm: MobileId) {
    let can = get_virtues(w, pm).is_some_and(|v| {
        v.last_valor_loss
            .checked_add(TimeSpan::from_days(7))
            .is_none_or(|t| t < w.now)
    });
    if get_value(w, pm, VirtueName::Valor) > 0 && can {
        if atrophy(w, pm, VirtueName::Valor, 250) {
            send_localized_message(w, pm, 1054040, "", 0x3B2); // You have lost some Valor.
        }
        let now = w.now;
        virtues_mut(w, pm).last_valor_loss = now;
    }
}

/// `ValorVirtue.Valor(from, targ)`.
pub fn valor(w: &mut World, from: MobileId, targ: Targeted) {
    let hooks = w.resources.get::<ChampionHooks>();
    let idol = match targ {
        Targeted::Entity(EntityId::Item(i)) => i,
        _ => {
            send_localized_message(w, from, 1054035, "", 0x3B2); // You must target a Champion Idol...
            return;
        }
    };
    let spawn = hooks.and_then(|h| h.idol_spawn).and_then(|f| f(w, idol));
    let Some(spawn) = spawn else {
        send_localized_message(w, from, 1054035, "", 0x3B2);
        return;
    };
    if w.mobile(from).is_some_and(|m| m.hidden) {
        send_localized_message(w, from, 1052015, "", 0x3B2);
    } else if spawn.has_been_advanced {
        send_localized_message(w, from, 1054038, "", 0x3B2); // The Champion of this region has already been challenged!
    } else if spawn.active {
        if spawn.champion_spawned {
            send_localized_message(w, from, 1112470, "", 0x3B2); // You may not use Valor on this Champion Idol...
        }
        let (needed, consumed) = match spawn.sub_level {
            0 => (2500, 2500),
            1 => (5000, 5000),
            2 => (10000, 7500),
            _ => (20000, 10000),
        };
        if get_virtues(w, from).is_some_and(|_| get_value(w, from, VirtueName::Valor) >= needed) {
            atrophy(w, from, VirtueName::Valor, consumed);
            send_localized_message(w, from, 1054037, "", 0x3B2); // Your challenge is heard...
            if let Some(f) = w
                .resources
                .get::<ChampionHooks>()
                .and_then(|h| h.advance_level)
            {
                f(w, idol);
            }
        } else {
            send_localized_message(w, from, 1054039, "", 0x3B2); // The Champion of this region ignores your challenge...
        }
    } else if get_level(w, from, VirtueName::Valor) == VirtueLevel::Knight {
        atrophy(w, from, VirtueName::Valor, 11000);
        send_localized_message(w, from, 1054037, "", 0x3B2);
        if let Some(f) = w
            .resources
            .get::<ChampionHooks>()
            .and_then(|h| h.start_by_valor)
        {
            f(w, idol);
        }
    } else {
        send_localized_message(w, from, 1054036, "", 0x3B2); // You must be a Knight of Valor...
    }
}

// ---- JusticeVirtue -----------------------------------------------------------------------------------------------

const JUSTICE_ACTION: &str = "Server.Engines.Virtues.JusticeVirtue";

/// `JusticeProtectorStatus`.
pub const PROTECTOR: i32 = 1;
pub const PROTECTED: i32 = 2;

/// `JusticeVirtue.IsProtected(pm)`.
pub fn justice_is_protected(w: &World, pm: MobileId) -> bool {
    get_virtues(w, pm)
        .is_some_and(|v| v.justice_status == PROTECTED && v.justice_protection.is_some())
}

/// `JusticeVirtue.GetProtector(pm)`.
pub fn justice_get_protector(w: &World, pm: MobileId) -> Option<MobileId> {
    get_virtues(w, pm)
        .filter(|v| v.justice_status == PROTECTED)
        .and_then(|v| v.justice_protection)
}

/// `JusticeVirtue.GetProtected(pm)`.
pub fn justice_get_protected(w: &World, pm: MobileId) -> Option<MobileId> {
    get_virtues(w, pm)
        .filter(|v| v.justice_status == PROTECTOR)
        .and_then(|v| v.justice_protection)
}

/// `JusticeVirtue.CancelProtection(pm, out protector)`: the protector it dropped.
pub fn justice_cancel_protection(w: &mut World, pm: MobileId) -> Option<Option<MobileId>> {
    let status = get_virtues(w, pm).map(|v| v.justice_status)?;
    if status == 0 {
        return None;
    }
    let v = virtues_mut(w, pm);
    let protector = v.justice_protection.take();
    v.justice_status = 0;
    if let Some(p) = protector
        && let Some(t) = w.resources.get_mut::<VirtueTable>().0.get_mut(&p)
    {
        t.justice_protection = None;
        t.justice_status = 0;
    }
    Some(protector)
}

/// `JusticeVirtue.CheckMapRegion(first, second)`.
pub fn justice_check_map_region(w: &World, first: MobileId, second: MobileId) -> bool {
    let (Some(a), Some(b)) = (w.mobile(first), w.mobile(second)) else {
        return false;
    };
    if a.map != b.map {
        return false;
    }
    map_region(a.map, a.location) == map_region(b.map, b.location)
}

fn map_region(map: Option<MapId>, loc: Point3D) -> i32 {
    match map {
        Some(m) if m.0 < 2 => {
            if loc.x < 5120 {
                0
            } else if loc.y < 2304 {
                1
            } else {
                2
            }
        }
        _ => 0,
    }
}

fn justice_on_virtue_used(w: &mut World, from: MobileId) {
    if !muo_server::mobile::part2::stats::check_alive(w, from, true) {
        return;
    }
    if !is_player(w, from) {
        return;
    }
    if let Some(msg) = justice_start_check(w, from) {
        send_localized_message(w, from, msg, "", 0x3B2);
    } else {
        target::begin_target(w, from, 14, false, TargetFlags::NONE, justice_on_targeted);
        send_localized_message(w, from, 1049366, "", 0x3B2); // Choose the player you wish to protect.
    }
}

/// The checks every step of the protection offer repeats; the message number of the first that fails.
fn justice_start_check(w: &World, protector: MobileId) -> Option<i32> {
    if !is_seeker(w, protector, VirtueName::Justice) {
        Some(1049610) // You must reach the first path in this virtue to invoke it.
    } else if !combat::can_begin_action(w, protector, JUSTICE_ACTION) {
        Some(1049370) // You must wait a while before offering your protection again.
    } else if justice_is_protected(w, protector) {
        Some(1049542) // You cannot protect someone while being protected.
    } else if w.mobile(protector).and_then(|m| m.map) != Some(MapId::FELUCCA) {
        Some(1049372) // You cannot use this ability here.
    } else {
        None
    }
}

fn justice_on_targeted(w: &mut World, from: MobileId, t: Targeted) {
    if !is_player(w, from) {
        return;
    }
    let pm = match t {
        Targeted::Entity(EntityId::Mobile(m)) if is_player(w, m) => Some(m),
        _ => None,
    };
    if let Some(msg) = justice_start_check(w, from) {
        send_localized_message(w, from, msg, "", 0x3B2);
        return;
    }
    let Some(pm) = pm else {
        send_localized_message(w, from, 1049678, "", 0x3B2); // Only players can be protected.
        return;
    };
    if w.mobile(pm).and_then(|m| m.map) != Some(MapId::FELUCCA) {
        send_localized_message(w, from, 1049372, "", 0x3B2);
    } else if pm == from
        || muo_server::mobile::base::criminal(w, pm)
        || muo_server::mobile::base::murderer(w, pm)
    {
        send_localized_message(w, from, 1049436, "", 0x3B2); // That player cannot be protected.
    } else if justice_is_protected(w, pm) {
        send_localized_message(w, from, 1049369, "", 0x3B2); // You cannot protect that player right now.
    } else if props_b::net_state(w, pm)
        .is_some_and(|ns| muo_content::gumps::has_gump::<AcceptProtectorGump>(w, ns))
    {
        send_localized_message(w, from, 1049369, "", 0x3B2);
    } else {
        let protector_name = w
            .mobile(from)
            .and_then(|m| (m.ty.name)(w, from))
            .unwrap_or_default();
        let g = StaticGumpObj::new(
            150,
            50,
            AcceptProtectorGump {
                protector: from,
                protectee: pm,
                protector_name,
            },
        );
        send_gump_to_mobile(w, pm, Box::new(g), false);
    }
}

/// `JusticeVirtue.OnVirtueAccepted(protector, protectee)`.
pub fn justice_on_accepted(w: &mut World, protector: MobileId, protectee: MobileId) {
    if let Some(msg) = justice_start_check(w, protector) {
        send_localized_message(w, protector, msg, "", 0x3B2);
    } else if w.mobile(protectee).and_then(|m| m.map) != Some(MapId::FELUCCA) {
        send_localized_message(w, protector, 1049372, "", 0x3B2);
    } else if protectee == protector
        || muo_server::mobile::base::criminal(w, protectee)
        || muo_server::mobile::base::murderer(w, protectee)
    {
        send_localized_message(w, protector, 1049436, "", 0x3B2);
    } else if justice_is_protected(w, protectee) {
        send_localized_message(w, protector, 1049369, "", 0x3B2);
    } else {
        muo_content::player_mobile::tables::add_protection(w, protector, protectee);
        let name = |w: &World, m| {
            w.mobile(m)
                .and_then(|x| (x.ty.name)(w, m))
                .unwrap_or_default()
        };
        let args = format!("{}\t{}", name(w, protector), name(w, protectee));
        send_localized_message(w, protectee, 1049451, &args, 0x3B2); // You are now being protected by ~1_NAME~.
        send_localized_message(w, protector, 1049452, &args, 0x3B2); // You are now protecting ~2_NAME~.
    }
}

/// `JusticeVirtue.OnVirtueRejected(protector, protectee)`.
pub fn justice_on_rejected(w: &mut World, protector: MobileId, protectee: MobileId) {
    let name = |w: &World, m| {
        w.mobile(m)
            .and_then(|x| (x.ty.name)(w, m))
            .unwrap_or_default()
    };
    let args = format!("{}\t{}", name(w, protector), name(w, protectee));
    send_localized_message(w, protectee, 1049453, &args, 0x3B2); // You have declined protection from ~1_NAME~.
    send_localized_message(w, protector, 1049454, &args, 0x3B2); // ~2_NAME~ has declined your protection.
    if combat::begin_action(w, protector, JUSTICE_ACTION) {
        timer::start(
            w,
            TimeSpan::from_mins_f64(15.0),
            TimeSpan::ZERO,
            1,
            move |w| {
                combat::end_action(w, protector, JUSTICE_ACTION);
            },
        );
    }
}

fn justice_check_atrophy(w: &mut World, pm: MobileId) {
    let can = get_virtues(w, pm).is_some_and(|v| {
        v.last_justice_loss
            .checked_add(TimeSpan::from_days(7))
            .is_none_or(|t| t < w.now)
    });
    if get_value(w, pm, VirtueName::Justice) > 0 && can {
        if atrophy(w, pm, VirtueName::Justice, 950) {
            send_localized_message(w, pm, 1049373, "", 0x3B2); // You have lost some Justice.
        }
        let now = w.now;
        virtues_mut(w, pm).last_justice_loss = now;
    }
}

/// `AcceptProtectorGump`.
pub struct AcceptProtectorGump {
    protector: MobileId,
    protectee: MobileId,
    protector_name: String,
}

impl StaticGumpLogic for AcceptProtectorGump {
    const TYPE_NAME: &'static str = "Server.Engines.Virtues.AcceptProtectorGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.set_no_close();
        b.add_page(0);
        b.add_background(0, 0, 396, 218, 3600);
        b.add_image_tiled(15, 15, 365, 190, 2624);
        b.add_alpha_region(15, 15, 365, 190);
        // Another player is offering you their <a href="?ForceTopic88">protection</a>:
        b.add_html_localized_color(30, 20, 360, 25, 1049365, 0x7FFF, false, false);
        b.add_label_placeholder(90, 55, 1153, "protector");
        b.add_image(50, 45, 9005, 0);
        b.add_image_tiled(80, 80, 200, 1, 9107);
        b.add_image_tiled(95, 82, 200, 1, 9157);
        b.add_radio(30, 110, 9727, 9730, true, 1);
        b.add_html_localized_color(65, 115, 300, 25, 1049444, 0x7FFF, false, false); // Yes, I would like their protection.
        b.add_radio(30, 145, 9727, 9730, false, 0);
        b.add_html_localized_color(65, 148, 300, 25, 1049445, 0x7FFF, false, false); // No thanks, I can take care of myself.
        b.add_button(160, 175, 247, 248, 2);
        b.add_image(215, 0, 50581, 0);
        b.add_image_tiled(15, 14, 365, 1, 9107);
        b.add_image_tiled(380, 14, 1, 190, 9105);
        b.add_image_tiled(15, 205, 365, 1, 9107);
        b.add_image_tiled(15, 14, 1, 190, 9105);
        b.add_image_tiled(0, 0, 395, 1, 9157);
        b.add_image_tiled(394, 0, 1, 217, 9155);
        b.add_image_tiled(0, 216, 395, 1, 9157);
        b.add_image_tiled(0, 0, 1, 217, 9155);
    }

    fn build_strings(&self, s: &mut GumpStrings) {
        s.set_string_slot("protector", &self.protector_name);
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id != 2 {
            return;
        }
        if info.is_switched(1) {
            justice_on_accepted(w, self.protector, self.protectee);
        } else {
            justice_on_rejected(w, self.protector, self.protectee);
        }
    }
}

/// Subscribes `JusticeVirtue.OnPlayerDeleted` (`[OnEvent(PlayerDeletedEvent)]`). Priority 49, before the
/// Initialize that registers `PlayerMobile.forget_player`: the protection is read from the virtue table.
pub fn subscribe_justice_deleted(w: &mut World) {
    muo_content::player_mobile::subscribe_deleted(w, justice_on_player_deleted);
}

/// `JusticeVirtue.OnPlayerDeleted`.
pub fn justice_on_player_deleted(w: &mut World, pm: MobileId) {
    justice_cancel_protection(w, pm);
}

// ---- HonorVirtue -------------------------------------------------------------------------------------------------

const USE_DELAY: TimeSpan = TimeSpan::from_mins(5);

fn honor_on_virtue_used(w: &mut World, from: MobileId) {
    if mx::alive(w, from) {
        send_localized_message(w, from, 1063160, "", 0x3B2); // Target what you wish to honor.
        let mut t = target::Target::simple(w, 12, TargetFlags::NONE, false, honor_on_target);
        t.check_los = true;
        t.handlers.on_target_out_of_range = Some(std::rc::Rc::new(|w, from, _| {
            send_localized_message(w, from, 1063232, "", 0x3B2)
        })); // You are too far away to honor your opponent
        let id = target::add_target(w, t);
        target::set_mobile_target(w, from, Some(id));
    }
}

fn honor_duration(w: &World, from: MobileId) -> i32 {
    match get_level(w, from, VirtueName::Honor) {
        VirtueLevel::Seeker => 30,
        VirtueLevel::Follower => 90,
        VirtueLevel::Knight => 300,
        VirtueLevel::None => 0,
    }
}

fn honor_on_target(w: &mut World, from: MobileId, t: Targeted) {
    if !is_player(w, from) {
        return;
    }
    match t {
        Targeted::Entity(EntityId::Mobile(m)) if m == from => embrace_honor(w, from),
        Targeted::Entity(EntityId::Mobile(m)) => honor(w, from, m),
        _ => {}
    }
}

fn embrace_honor(w: &mut World, pm: MobileId) {
    if get_virtues(w, pm).is_some_and(|v| v.honor_active) {
        send_localized_message(w, pm, 1063230, "", 0x3B2); // You must wait awhile before you can embrace honor again.
        return;
    }
    if honor_duration(w, pm) == 0 {
        send_localized_message(w, pm, 1063234, "", 0x3B2); // You do not have enough honor to do that
        return;
    }
    if let Some(v) = get_virtues(w, pm) {
        let wait = w.now - v.last_honor_use;
        if wait < USE_DELAY {
            let remaining = USE_DELAY - wait;
            let minutes = remaining.total_minutes().ceil() as i32;
            send_localized_message(w, pm, 1063240, &minutes.to_string(), 0x3B2); // You must wait ~1_HONOR_WAIT~ minutes...
            return;
        }
    }
    send_gump_to_mobile(
        w,
        pm,
        Box::new(StaticGumpObj::new(150, 50, HonorSelfGump { from: pm })),
        false,
    );
}

/// `HonorVirtue.ActivateEmbrace(pm)`.
pub fn activate_embrace(w: &mut World, pm: MobileId) {
    let duration = honor_duration(w, pm);
    virtues_mut(w, pm);
    let honor = get_value(w, pm, VirtueName::Honor);
    let used_points = if honor < 4399 {
        400
    } else if honor < 10599 {
        600
    } else {
        1000
    };
    atrophy(w, pm, VirtueName::Honor, used_points);
    virtues_mut(w, pm).honor_active = true;
    send_localized_message(w, pm, 1063235, "", 0x3B2); // You embrace your honor
    timer::start(
        w,
        TimeSpan::from_secs_f64(f64::from(duration)),
        TimeSpan::ZERO,
        1,
        move |w| {
            // We get the virtues again, in case it was deleted/dereferenced
            let now = w.now;
            let v = virtues_mut(w, pm);
            v.honor_active = false;
            v.last_honor_use = now;
            send_localized_message(w, pm, 1063236, "", 0x3B2); // You no longer embrace your honor
        },
    );
}

/// `HonorVirtue.Honor(source, target)` for a target that is an `IHonorTarget` (`BaseCreature` or `PlayerMobile`).
fn honor(w: &mut World, source: MobileId, target: MobileId) {
    let is_honor_target =
        muo_content::creature::data::cd(w, target).is_some() || is_player(w, target);
    if !is_honor_target {
        return;
    }
    if let Some(&cid) = hs_ref(w).and_then(|s| s.received.get(&target)) {
        let (csource, in_range) = {
            let c = &hs_ref(w).expect("checked").contexts[&cid];
            (c.source, check_distance(w, c.source, c.target))
        };
        if csource == source {
            return;
        }
        if in_range {
            send_localized_message(w, source, 1063233, "", 0x3B2); // Somebody else is honoring this opponent
            return;
        }
    }
    if muo_content::core::ml(w) && is_player(w, target) {
        send_localized_message(w, source, 1075614, "", 0x3B2); // You cannot honor other players.
        return;
    }
    if mx::hits(w, target) < mx::hits_max(w, target) {
        send_localized_message(w, source, 1063166, "", 0x3B2); // You cannot honor this monster because it is too damaged.
        return;
    }
    // Allow honor on blue if not in a guarded region or blue in Felucca
    let human = w
        .resources
        .get::<muo_primitives::BodyTable>()
        .is_some_and(|t| t.is_human(muo_server::mobile::part1::props_b::body(w, target)));
    let always = muo_content::creature::vtable::ctype(w, target)
        .is_some_and(|c| (c.always_attackable)(w, target) || (c.always_murderer)(w, target));
    let creature = muo_content::creature::data::cd(w, target).is_some();
    let guarded_disabled = {
        use muo_content::regions::guarded::{GUARDED_REGION, is_disabled};
        let r = props_b::region(w, source);
        muo_server::region::get_region_of_type(w, r, &GUARDED_REGION)
            .is_some_and(|g| is_disabled(w, g))
    };
    let harmful_restrictions = w
        .mobile(source)
        .and_then(|m| m.map)
        .and_then(|m| w.maps.get(m))
        .is_some_and(|m| {
            m.rules
                .contains(muo_server::map::MapRules::HARMFUL_RESTRICTIONS)
        });
    if human && (!creature || !always) && guarded_disabled && harmful_restrictions {
        send_localized_message(w, source, 1001018, "", 0x3B2); // You cannot perform negative acts
        return;
    }
    if let Some(&old) = hs_ref(w).and_then(|s| s.sent.get(&source)) {
        cancel_context(w, old);
    }
    new_honor_context(w, source, target);
    let dir = mx::get_direction_to(w, source, target);
    mx::set_direction(w, source, dir);
    if !props_b::mounted(w, source) {
        mx::animate(w, source, 32, 5, 1, true, true, 0);
    }
}

fn check_distance(w: &World, a: MobileId, b: MobileId) -> bool {
    match (w.mobile(a), w.mobile(b)) {
        (Some(x), Some(y)) => muo_primitives::utility::in_range(
            x.location.x,
            x.location.y,
            y.location.x,
            y.location.y,
            18,
        ),
        _ => false,
    }
}

// ---- HonorSelfGump -----------------------------------------------------------------------------------------------

pub struct HonorSelfGump {
    from: MobileId,
}

impl StaticGumpLogic for HonorSelfGump {
    const TYPE_NAME: &'static str = "Server.Gumps.HonorSelfGump";
    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 245, 145, 9250);
        b.add_button(157, 101, 247, 248, 1);
        b.add_button(81, 100, 241, 248, 0);
        // Are you sure you want to use honor points on yourself?
        b.add_html_localized(21, 20, 203, 70, 1071218, true, false);
    }
    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 1 {
            activate_embrace(w, self.from);
        }
    }
}

// ---- HonorContext ------------------------------------------------------------------------------------------------

/// `new HonorContext(source, target)`.
fn new_honor_context(w: &mut World, source: MobileId, target: MobileId) -> u32 {
    let (loc, map) = w
        .mobile(source)
        .map_or((Point3D::default(), None), |m| (m.location, m.map));
    let s = hs(w);
    s.next += 1;
    let id = s.next;
    let one = TimeSpan::from_secs_f64(1.0);
    let timer = timer::start(w, one, one, 0, move |w| {
        let alive = hs_ref(w)
            .and_then(|s| s.contexts.get(&id))
            .map(|c| (c.source, c.target));
        if let Some((a, b)) = alive
            && !check_distance(w, a, b)
        {
            cancel_context(w, id);
        }
    });
    let s = hs(w);
    s.contexts.insert(
        id,
        HonorContext {
            source,
            target,
            initial_location: loc,
            initial_map: map,
            first_hit: FirstHit::NotDelivered,
            honor_damage: 0.0,
            poisoned: false,
            total_damage: 0,
            perfection_damage_bonus: 0,
            timer,
        },
    );
    s.sent.insert(source, id);
    s.received.insert(target, id);
    let fortymin = TimeSpan::from_mins_f64(40.0);
    let until = w.now + fortymin;
    hs(w).honor_time.insert(source, until);
    timer::start(w, fortymin, TimeSpan::ZERO, 1, move |w| {
        let t = hs_ref(w).and_then(|s| s.honor_time.get(&source).copied());
        let has_sent = hs_ref(w).is_some_and(|s| s.sent.contains_key(&source));
        if t.is_some_and(|t| t < w.now) && has_sent {
            cancel_context(w, id);
        }
    });
    id
}

/// `HonorContext.Cancel()`.
fn cancel_context(w: &mut World, id: u32) {
    let Some(c) = hs(w).contexts.shift_remove(&id) else {
        return;
    };
    let s = hs(w);
    s.sent.shift_remove(&c.source);
    s.received.shift_remove(&c.target);
    let mut t = c.timer;
    t.cancel(w);
}

/// `pm.SentHonorContext` as an id.
pub fn sent_context(w: &World, source: MobileId) -> Option<u32> {
    hs_ref(w)?.sent.get(&source).copied()
}

/// `target.ReceivedHonorContext` as an id.
pub fn received_context(w: &World, target: MobileId) -> Option<u32> {
    hs_ref(w)?.received.get(&target).copied()
}

fn ctx_mut(w: &mut World, id: u32) -> Option<&mut HonorContext> {
    hs(w).contexts.get_mut(&id)
}

/// `SentHonorContext?.Cancel()`.
pub fn cancel_sent(w: &mut World, source: MobileId) {
    if let Some(c) = sent_context(w, source) {
        cancel_context(w, c);
    }
}

/// `ReceivedHonorContext?.Cancel()`.
pub fn cancel_received(w: &mut World, target: MobileId) {
    if let Some(c) = received_context(w, target) {
        cancel_context(w, c);
    }
}

/// `OnSourceDamaged(from, amount)`.
pub fn on_source_damaged(w: &mut World, source: MobileId, from: MobileId, _amount: i32) {
    let Some(id) = sent_context(w, source) else {
        return;
    };
    if let Some(c) = ctx_mut(w, id)
        && from == c.target
        && c.first_hit == FirstHit::NotDelivered
    {
        c.first_hit = FirstHit::Granted;
    }
}

/// `OnTargetPoisoned()`.
pub fn on_target_poisoned(w: &mut World, target: MobileId) {
    if let Some(id) = received_context(w, target)
        && let Some(c) = ctx_mut(w, id)
    {
        c.poisoned = true; // Set this flag for OnTargetDamaged which will be called next
    }
}

/// `OnTargetDamaged(from, amount)`.
pub fn on_target_damaged(w: &mut World, target: MobileId, from: MobileId, amount: i32) {
    let Some(id) = received_context(w, target) else {
        return;
    };
    let Some(c) = hs_ref(w).and_then(|s| s.contexts.get(&id)) else {
        return;
    };
    let (source, loc, map, poisoned) = (c.source, c.initial_location, c.initial_map, c.poisoned);
    if let Some(c) = ctx_mut(w, id) {
        if c.first_hit == FirstHit::NotDelivered {
            c.first_hit = FirstHit::Delivered;
        }
        if poisoned {
            c.honor_damage += f64::from(amount) * 0.8;
            c.poisoned = false; // Reset the flag
            return;
        }
        c.total_damage += amount;
    }
    if from == source {
        let sees = muo_server::mobile::base::can_see_mobile(w, target, source)
            && mx::in_los(w, target, source);
        let src_at = w
            .mobile(source)
            .is_some_and(|m| m.location == loc && m.map == map);
        let close = mx::in_range(w, source, target, 1);
        let full = sees && (close || src_at);
        if let Some(c) = ctx_mut(w, id) {
            c.honor_damage += if full {
                f64::from(amount)
            } else {
                f64::from(amount) * 0.8
            };
        }
    } else if muo_content::creature::control::get_master(w, from) == Some(source)
        && let Some(c) = ctx_mut(w, id)
    {
        c.honor_damage += f64::from(amount) * 0.8;
    }
}

/// `OnTargetHit(from)`.
pub fn on_target_hit(w: &mut World, target: MobileId, from: MobileId) {
    let Some(id) = received_context(w, target) else {
        return;
    };
    let (source, bonus) = match hs_ref(w).and_then(|s| s.contexts.get(&id)) {
        Some(c) => (c.source, c.perfection_damage_bonus),
        None => return,
    };
    if from != source || bonus == 100 {
        return;
    }
    let bushido = muo_server::skills::value(w, from, muo_server::skills::SkillName::Bushido) as i32;
    if bushido < 50 {
        return;
    }
    let new = bonus + bushido / 10;
    if new >= 100 {
        if let Some(c) = ctx_mut(w, id) {
            c.perfection_damage_bonus = 100;
        }
        send_localized_message(w, source, 1063254, "", 0x3B2); // You have Achieved Perfection...
    } else {
        if let Some(c) = ctx_mut(w, id) {
            c.perfection_damage_bonus = new;
        }
        send_localized_message(w, source, 1063255, "", 0x3B2); // You gain in Perfection...
    }
}

/// `OnTargetMissed(from)`.
pub fn on_target_missed(w: &mut World, target: MobileId, from: MobileId) {
    let Some(id) = received_context(w, target) else {
        return;
    };
    let (source, bonus) = match hs_ref(w).and_then(|s| s.contexts.get(&id)) {
        Some(c) => (c.source, c.perfection_damage_bonus),
        None => return,
    };
    if from != source || bonus == 0 {
        return;
    }
    let new = bonus - 25;
    if new <= 0 {
        if let Some(c) = ctx_mut(w, id) {
            c.perfection_damage_bonus = 0;
        }
        send_localized_message(w, source, 1063256, "", 0x3B2); // You have lost all Perfection...
    } else {
        if let Some(c) = ctx_mut(w, id) {
            c.perfection_damage_bonus = new;
        }
        send_localized_message(w, source, 1063257, "", 0x3B2); // You have lost some Perfection...
    }
}

/// `OnSourceBeneficialAction(to)`.
pub fn on_source_beneficial_action(w: &mut World, source: MobileId, to: MobileId) {
    let Some(id) = sent_context(w, source) else {
        return;
    };
    let Some(c) = ctx_mut(w, id) else { return };
    if to != c.target {
        return;
    }
    if c.perfection_damage_bonus >= 0 {
        c.perfection_damage_bonus = 0;
        send_localized_message(w, source, 1063256, "", 0x3B2);
    }
}

/// `OnTargetKilled()`.
pub fn on_target_killed(w: &mut World, target: MobileId) {
    let Some(id) = received_context(w, target) else {
        return;
    };
    let Some(c) = hs_ref(w).and_then(|s| s.contexts.get(&id)) else {
        return;
    };
    let (source, bonus, first_hit, honor_damage, total_damage) = (
        c.source,
        c.perfection_damage_bonus,
        c.first_hit,
        c.honor_damage,
        c.total_damage,
    );
    cancel_context(w, id);
    let target_fame = w.mobile(target).map_or(0, |m| m.fame);
    if bonus > 0 {
        let restore = (bonus * (target_fame + 5000) / 25000).min(10);
        let (h, s, m) = {
            let x = w.mobile(source);
            (
                x.map_or(0, |x| x.hits),
                x.map_or(0, |x| x.stam),
                x.map_or(0, |x| x.mana),
            )
        };
        props_b::set_hits(w, source, h + restore);
        props_b::set_stam(w, source, s + restore);
        props_b::set_mana(w, source, m + restore);
    }
    if get_value(w, source, VirtueName::Honor) > target_fame {
        return;
    }
    // Initial honor gain is 100th of the monsters honor
    let mut d_gain = f64::from(target_fame) / 100.0 * (honor_damage / f64::from(total_damage));
    if (honor_damage - f64::from(total_damage)).abs() < 0.01 && first_hit == FirstHit::Granted {
        d_gain *= 1.5; // honor gain is increased a lot more if the combat was fully honorable
    } else {
        d_gain *= 0.9;
    }
    // Minimum gain of 1 honor when the honor is under the monsters fame
    let gain = (d_gain as i32).clamp(1, 200);
    if is_highest_path(w, source, VirtueName::Honor) {
        send_localized_message(w, source, 1063228, "", 0x3B2); // You cannot gain more Honor.
        return;
    }
    if let Some(gained_path) = award(w, source, VirtueName::Honor, gain) {
        send_localized_message(
            w,
            source,
            if gained_path { 1063226 } else { 1063225 },
            "",
            0x3B2,
        );
    }
}

/// `SentHonorContext.PerfectionDamageBonus` when its target is `defender`, else 0.
pub fn perfection_bonus(w: &World, attacker: MobileId, defender: MobileId) -> i32 {
    sent_context(w, attacker)
        .and_then(|id| hs_ref(w)?.contexts.get(&id))
        .filter(|c| c.target == defender)
        .map_or(0, |c| c.perfection_damage_bonus)
}

/// `SentHonorContext.PerfectionLuckBonus` when its target is `victim`, else 0.
pub fn perfection_luck(w: &World, killer: MobileId, victim: MobileId) -> i32 {
    sent_context(w, killer)
        .and_then(|id| hs_ref(w)?.contexts.get(&id))
        .filter(|c| c.target == victim)
        .map_or(0, HonorContext::perfection_luck_bonus)
}

// ---- VirtueGump ----------------------------------------------------------------------------------------------------

const HUE_TABLE: [i32; 24] = [
    0x0481, 0x0963, 0x0965, 0x060A, 0x060F, 0x002A, 0x08A4, 0x08A7, 0x0034, 0x0965, 0x08FD, 0x0480,
    0x00EA, 0x0845, 0x0020, 0x0011, 0x0269, 0x013D, 0x08A1, 0x08A3, 0x0042, 0x0543, 0x0547, 0x0061,
];

pub struct VirtueGump {
    beholder: MobileId,
    beheld: MobileId,
    values: [i32; 8],
}

fn hue_for(values: &[i32; 8], index: usize) -> i32 {
    let mut value = values[index];
    if value < 4000 {
        return 2402;
    }
    if value >= 30000 {
        value = 20000;
    }
    let vl = if value < 10000 {
        0
    } else if (value >= 20000 && index == 5)
        || (value >= 22000 && index == 1)
        || (value >= 21000 && index != 1)
    {
        2
    } else {
        1
    };
    HUE_TABLE[index * 3 + vl]
}

impl DynamicGumpLogic for VirtueGump {
    const TYPE_NAME: &'static str = "Server.Engines.Virtues.VirtueGump";
    fn singleton(&self) -> bool {
        true
    }
    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_image(30, 40, 104, 0);
        b.add_page(1);
        let items = [
            (61, 71, 108, 0),
            (123, 46, 112, 4),
            (187, 70, 107, 5),
            (35, 135, 110, 1),
            (211, 133, 105, 2),
            (61, 195, 111, 3),
            (186, 195, 109, 6),
            (121, 221, 106, 7),
        ];
        for (x, y, id, index) in items {
            b.add_image_class(x, y, id, hue_for(&self.values, index), "VirtueGumpItem");
        }
        if self.beholder == self.beheld {
            b.add_button(57, 269, 2027, 2027, 1);
            b.add_button(186, 269, 2071, 2071, 2);
        }
    }
    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 1 && self.beholder == self.beheld {
            virtue_status_display_to(w, self.beholder);
        }
    }
}

/// `VirtueGump.RequestVirtueGump(beholder, beheld)`.
pub fn request_virtue_gump(w: &mut World, beholder: MobileId, beheld: MobileId) {
    if !is_player(w, beholder) || !is_player(w, beheld) {
        return;
    }
    if beholder == beheld && muo_server::mobile::base::murderer(w, beholder) {
        send_localized_message(w, beholder, 1049609, "", 0x3B2); // Murderers cannot invoke this virtue.
    } else if w.mobile(beholder).map(|m| m.map) == w.mobile(beheld).map(|m| m.map)
        && mx::in_range(w, beholder, beheld, 12)
    {
        let values: [i32; 8] = std::array::from_fn(|i| {
            get_virtues(w, beheld)
                .and_then(|v| v.values.as_ref())
                .and_then(|v| v.get(i))
                .copied()
                .unwrap_or(0)
        });
        let mut g = DynamicGumpObj::new(
            0,
            0,
            VirtueGump {
                beholder,
                beheld,
                values,
            },
        );
        g.core.serial = w
            .mobile(beheld)
            .map_or(muo_primitives::Serial::new(0), |m| m.serial);
        send_gump_to_mobile(w, beholder, Box::new(g), false);
    }
}

/// `VirtueGump.RequestVirtueItem(beholder, beheld, gumpID)`.
pub fn request_virtue_item(w: &mut World, beholder: MobileId, beheld: MobileId, gump_id: i32) {
    if !is_player(w, beholder) || beholder != beheld {
        return;
    }
    if let Some(ns) = props_b::net_state(w, beholder) {
        muo_content::gumps::close_gump::<VirtueGump>(w, ns);
    }
    if muo_server::mobile::base::murderer(w, beholder) {
        send_localized_message(w, beholder, 1049609, "", 0x3B2);
        return;
    }
    let cb = w
        .resources
        .get::<VirtueCallbacks>()
        .and_then(|c| c.callbacks.get(&gump_id).copied());
    match cb {
        Some(f) => f(w, beholder),
        None => send_localized_message(w, beholder, 1052066, "", 0x3B2), // That virtue is not active yet.
    }
}

/// `VirtueGump.RequestVirtueMacro(beholder, virtue)`.
pub fn request_virtue_macro(w: &mut World, beholder: MobileId, virtue: i32) {
    let id = match virtue {
        0 => 107, // Honor
        1 => 110, // Sacrifice
        2 => 112, // Valor
        _ => 0,
    };
    request_virtue_item(w, beholder, beholder, id);
}

// ---- VirtueInfoGump / VirtueStatusGump ---------------------------------------------------------------------------------

pub struct VirtueInfoGump {
    beholder: MobileId,
    desc: i32,
    site: Option<String>,
    virtue: VirtueName,
    value: i32,
}

/// `VirtueInfoGump.DisplayTo`.
pub fn virtue_info_display_to(
    w: &mut World,
    beholder: MobileId,
    virtue: VirtueName,
    description: i32,
    web_page: Option<&str>,
) {
    if props_b::net_state(w, beholder).is_none() {
        return;
    }
    let value = get_value(w, beholder, virtue);
    let g = DynamicGumpObj::new(
        0,
        0,
        VirtueInfoGump {
            beholder,
            desc: description,
            site: web_page.map(str::to_owned),
            virtue,
            value,
        },
    );
    send_gump_to_mobile(w, beholder, Box::new(g), false);
}

impl DynamicGumpLogic for VirtueInfoGump {
    const TYPE_NAME: &'static str = "Server.Engines.Virtues.VirtueInfoGump";
    fn singleton(&self) -> bool {
        true
    }
    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let value = self.value;
        b.add_page(0);
        b.add_image(30, 40, 2080, 0);
        b.add_image(47, 77, 2081, 0);
        b.add_image(47, 147, 2081, 0);
        b.add_image(47, 217, 2081, 0);
        b.add_image(47, 267, 2083, 0);
        b.add_image(70, 213, 2091, 0);
        b.add_page(1);
        let max_value = get_max_amount(self.virtue);
        let dots = if value < 4000 {
            value / 400
        } else if value < 10000 {
            (value - 4000) / 600
        } else if value < max_value {
            (value - 10000) / ((max_value - 10000) / 10)
        } else {
            10
        };
        for i in 0..10 {
            b.add_image(95 + i * 17, 50, if i < dots { 2362 } else { 2360 }, 0);
        }
        let value_desc = if value < 1 {
            1052044
        } else if value < 400 {
            1052045
        } else if value < 2000 {
            1052046
        } else if value < 3600 {
            1052047
        } else if value < 4000 {
            1052048
        } else if dots < 1 {
            1052049
        } else if dots < 9 {
            1052047
        } else if dots < 10 {
            1052048
        } else {
            1052050
        };
        b.add_html_localized(157, 73, 200, 40, 1051000 + self.virtue as i32, false, false);
        b.add_html_localized(75, 95, 220, 140, self.desc, false, false);
        b.add_html_localized(70, 224, 229, 60, value_desc, false, false);
        b.add_button(65, 277, 1209, 1209, 1);
        b.add_button(280, 43, 4014, 4014, 2);
        // This virtue is not yet defined. OR -click to learn more (opens webpage)
        b.add_html_localized(
            83,
            275,
            400,
            40,
            if self.site.is_none() {
                1052055
            } else {
                1052052
            },
            false,
            false,
        );
    }
    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        match info.button_id {
            1 => {
                let (b, d, s, v) = (self.beholder, self.desc, self.site.clone(), self.virtue);
                virtue_info_display_to(w, b, v, d, s.as_deref());
                if let Some(site) = s {
                    // state.SendLaunchBrowser: no message about the web browser starting
                    muo_server::packets::player::send_launch_browser(w, _sender, &site);
                }
            }
            2 => virtue_status_display_to(w, self.beholder),
            _ => {}
        }
    }
}

pub struct VirtueStatusGump {
    beholder: MobileId,
}

/// `VirtueStatusGump.DisplayTo`.
pub fn virtue_status_display_to(w: &mut World, beholder: MobileId) {
    if props_b::net_state(w, beholder).is_none() {
        return;
    }
    send_gump_to_mobile(
        w,
        beholder,
        Box::new(StaticGumpObj::new(0, 0, VirtueStatusGump { beholder })),
        false,
    );
}

fn virtue_description(v: VirtueName) -> i32 {
    match v {
        VirtueName::Humility => 1052051,
        VirtueName::Sacrifice => 1052053,
        VirtueName::Compassion => 1053000,
        VirtueName::Spirituality => 1052056,
        VirtueName::Valor => 1054033,
        VirtueName::Honor => 1052058,
        VirtueName::Justice => 1052059,
        VirtueName::Honesty => 1052060,
    }
}

impl StaticGumpLogic for VirtueStatusGump {
    const TYPE_NAME: &'static str = "Server.Engines.Virtues.VirtueStatusGump";
    fn singleton(&self) -> bool {
        true
    }
    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_image(30, 40, 2080, 0);
        b.add_image(47, 77, 2081, 0);
        b.add_image(47, 147, 2081, 0);
        b.add_image(47, 217, 2081, 0);
        b.add_image(47, 267, 2083, 0);
        b.add_image(70, 213, 2091, 0);
        b.add_page(1);
        b.add_html_localized(140, 73, 200, 20, 1077972, false, false); // The Virtues
        let col1 = [
            (100, 1051000),
            (129, 1051001),
            (159, 1051002),
            (189, 1051003),
        ];
        let col2 = [
            (100, 1051004),
            (129, 1051005),
            (159, 1051006),
            (189, 1051007),
        ];
        for (y, n) in col1 {
            b.add_html_localized(80, y, 100, 40, n, false, false);
        }
        for (y, n) in col2 {
            b.add_html_localized(200, y, 200, 40, n, false, false);
        }
        b.add_html_localized(75, 224, 220, 60, 1052062, false, false); // Click on a blue gem to view your status in that virtue.
        for (i, y) in [100, 129, 159, 189].into_iter().enumerate() {
            b.add_button(60, y, 1210, 1210, i as i32 + 1);
            b.add_button(180, y, 1210, 1210, i as i32 + 5);
        }
        b.add_button(280, 43, 4014, 4014, 9);
    }
    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 9 {
            request_virtue_gump(w, self.beholder, self.beholder);
            return;
        }
        let Some(virtue) = (if (1..=7).contains(&info.button_id) {
            VirtueName::from_i32(info.button_id - 1)
        } else {
            None
        }) else {
            return;
        };
        let url = format!(
            "https://uo.com/wiki/ultima-online-wiki/gameplay/the-virtues/#{}",
            get_lower_case_name(virtue)
        );
        virtue_info_display_to(
            w,
            self.beholder,
            virtue,
            virtue_description(virtue),
            Some(&url),
        );
    }
}

// ---- registration --------------------------------------------------------------------------------------------------

/// `VirtueSystem.Configure` and the hooks other systems call the virtues through.
pub fn configure(w: &mut World) {
    use muo_content::gumps::system::GumpHooks;
    use muo_content::items::weapon_hooks::WeaponHooks;
    w.resources.get_mut::<VirtueCallbacks>();
    w.resources.get_mut::<GumpHooks>().request_virtue_gump = Some(request_virtue_gump);
    w.resources.get_mut::<GumpHooks>().request_virtue_item = Some(request_virtue_item);
    w.resources.get_mut::<GameHooks>().virtue_macro = Some(request_virtue_macro);
    let h = w.resources.get_mut::<WeaponHooks>();
    h.virtue_honor_active = Some(|w, m| get_virtues(w, m).is_some_and(|v| v.honor_active));
    // `BaseCreature.IsEnemy`: a player under Honor, or in Ethereal Voyage, is not attacked on sight.
    muo_content::hooks::hooks_mut(w).enemy_protection = Some(|w, m| {
        get_virtues(w, m).is_some_and(|v| v.honor_active)
            || muo_content::spells::helper::under_transformation_of(
                w,
                m,
                "Server.Spells.Spellweaving.EtherealVoyageSpell",
            )
    });
    let h = w.resources.get_mut::<WeaponHooks>();
    h.honor_perfection_bonus = Some(perfection_bonus);
    h.honor_on_target_hit = Some(on_target_hit);
    h.honor_on_target_missed = Some(on_target_missed);
    let p = w.resources.get_mut::<phooks::SystemHooks>();
    p.virtue_check_atrophies = Some(check_atrophies);
    p.virtue_award_justice = Some(
        |w, m, points| match award(w, m, VirtueName::Justice, points) {
            Some(gained) => (true, gained),
            None => (false, false),
        },
    );
    p.justice_is_protected = Some(justice_is_protected);
    p.justice_cancel_protection = Some(|w, m| justice_cancel_protection(w, m).flatten());
    p.honor_on_damaged = Some(|w, m, from, amount| {
        if let Some(from) = from {
            on_target_damaged(w, m, from, amount);
            on_source_damaged(w, m, from, amount);
        }
    });
    p.honor_on_killed = Some(on_target_killed);
    p.honor_on_source_beneficial_action = Some(on_source_beneficial_action);
    p.honor_cancel = Some(|w, m| {
        cancel_received(w, m);
        cancel_sent(w, m);
    });
    p.honor_on_target_poisoned = Some(on_target_poisoned);
    p.honor_perfection_luck = Some(perfection_luck);
    let _ = send_message_hue;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_follow_the_thresholds() {
        let mut w = World::new();
        let m = MobileId(muo_primitives::Serial::new(5));
        // not a PlayerMobile: level None whatever the table says
        assert_eq!(get_level(&w, m, VirtueName::Honor), VirtueLevel::None);
        let _ = &mut w;
        assert_eq!(get_max_amount(VirtueName::Honor), 20000);
        assert_eq!(get_max_amount(VirtueName::Sacrifice), 22000);
        assert_eq!(get_max_amount(VirtueName::Valor), 21000);
    }

    #[test]
    fn gump_hues_pick_the_level_column() {
        let mut v = [0; 8];
        assert_eq!(hue_for(&v, 0), 2402);
        v[0] = 4000;
        assert_eq!(hue_for(&v, 0), 0x0481);
        v[0] = 10000;
        assert_eq!(hue_for(&v, 0), 0x0963);
        v[0] = 21000;
        assert_eq!(hue_for(&v, 0), 0x0965);
        v[5] = 20000;
        assert_eq!(hue_for(&v, 5), HUE_TABLE[17]);
        v[1] = 21000;
        assert_eq!(hue_for(&v, 1), HUE_TABLE[4]); // sacrifice needs 22000 for the top column
    }

    #[test]
    fn map_regions_split_felucca_and_trammel() {
        assert_eq!(
            map_region(Some(MapId::FELUCCA), Point3D::new(100, 100, 0)),
            0
        );
        assert_eq!(
            map_region(Some(MapId::FELUCCA), Point3D::new(5200, 100, 0)),
            1
        );
        assert_eq!(
            map_region(Some(MapId::TRAMMEL), Point3D::new(5200, 2400, 0)),
            2
        );
        assert_eq!(map_region(None, Point3D::new(5200, 2400, 0)), 0);
    }
}
