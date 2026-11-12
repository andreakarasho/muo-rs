//! The faction system (`Faction`, `FactionSystem`, `FactionItem`, `Election`) and the `Town` data: factions,
//! members, kill points, silver, the election clock, faction items, death handling and skill loss, and the hooks
//! `muo_content` waits for (`FactionItemState`, notoriety, `PlayerMobile`, corpses, creature allegiance).
//!
//! The four factions and eight towns are table entries ([`tables`]) in a fixed order:
//! `Server.Factions.CouncilOfMages, Minax, Shadowlords, TrueBritannians`, then `Britain, Magincia, Minoc, Moonglow,
//! SkaraBrae, Trinsic, Vesper, Yew` (alphabetical by class name). The order must not change: saves name a faction or
//! town by `index + 1` in it (`Faction.WriteReference`).
//!
//! `FactionGuardAI.Think` does not cast spells, bandage or drink potions (see [`mobiles`]); the old `Guild` class is a
//! [`flow::GuildHooks`].

pub mod defs;
pub mod flow;
pub mod guard_ai;
pub mod gumps;
pub mod items;
pub mod legacy;
pub mod mobiles;
pub mod power;
pub mod state;
pub mod tables;
pub mod town;
pub mod traps;

#[cfg(test)]
mod tests;

use muo_primitives::dotnet::IgnoreCase;
use std::collections::HashMap;

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::part2::{effects as fx, messages as mm, properties};
use muo_server::mobile::{MobileDelta, base as mb};
use muo_server::mods::{ModId, SkillMod};
use muo_server::persistence::{GenericPayload, GenericPersistence};
use muo_server::property_list::PropertyList;
use muo_server::skills::{SkillName, SkillTable};
use muo_server::timer::{self, TimerHandle};

use muo_content::hooks::rnd;
use muo_content::items::{armor, clothing, equip_hooks, weapon};

use crate::ethics;
use state::{
    Election, ElectionState, FactionData, FactionItem, ITEM_EXPIRATION, PlayerState, TownData,
};
use tables::{FACTION_DEFS, TOWN_DEFS};

/// `Faction.Facet`.
pub const FACET: MapId = MapId::FELUCCA;
/// `Faction.StabilityFactor`, `StabilityActivation`.
pub const STABILITY_FACTOR: usize = 300;
pub const STABILITY_ACTIVATION: usize = 200;
/// `Faction.SkillLossFactor`, `SkillLossPeriod`, `LeavePeriod`.
pub const SKILL_LOSS_FACTOR: f64 = 1.0 / 3.0;
pub const SKILL_LOSS_PERIOD: TimeSpan = TimeSpan::from_mins(20);
pub const LEAVE_PERIOD: TimeSpan = TimeSpan::from_days(3);

/// Indices into the faction table, for the classes the code names.
pub const COUNCIL_OF_MAGES: usize = 0;
pub const MINAX: usize = 1;
pub const SHADOWLORDS: usize = 2;
pub const TRUE_BRITANNIANS: usize = 3;

struct SkillLossContext {
    mods: Vec<ModId>,
    timer: TimerHandle,
}

/// What the faction statics were: `Reflector`'s lists with each `FactionState` / `TownState`, and the loose maps.
pub struct FactionsState {
    /// `FactionSystem.Enabled`.
    pub enabled: bool,
    pub factions: Vec<FactionData>,
    pub towns: Vec<TownData>,
    /// `m_SkillLoss`.
    skill_loss: HashMap<MobileId, SkillLossContext>,
    /// The registration of the `FactionSystem` persistence, while enabled.
    pub persistence_id: Option<muo_io::serialization::persistence::PersistenceId>,
    /// `Sigil.Sigils` and `BaseMonolith.Monoliths`.
    pub sigils: Vec<ItemId>,
    pub monoliths: Vec<ItemId>,
    /// `Town.GuardLists` / `VendorLists`: the registered mobiles, `[town][definition]`.
    pub town_guards: Vec<Vec<Vec<MobileId>>>,
    pub town_vendors: Vec<Vec<Vec<MobileId>>>,
    /// The `StrongholdRegion` of each faction.
    pub stronghold_regions: Vec<Option<muo_server::region::RegionId>>,
    /// Items whose `FactionItemState` is set, with the faction (`FactionItem.Find` without the expiry check).
    attached: HashMap<ItemId, usize>,
}

impl Default for FactionsState {
    fn default() -> Self {
        FactionsState {
            enabled: false,
            factions: (0..FACTION_DEFS.len())
                .map(|_| FactionData::default())
                .collect(),
            towns: (0..TOWN_DEFS.len()).map(|_| TownData::default()).collect(),
            skill_loss: HashMap::new(),
            persistence_id: None,
            sigils: Vec::new(),
            monoliths: Vec::new(),
            town_guards: vec![Vec::new(); TOWN_DEFS.len()],
            town_vendors: vec![vec![Vec::new(); tables::VENDOR_DEFS.len()]; TOWN_DEFS.len()],
            stronghold_regions: vec![None; FACTION_DEFS.len()],
            attached: HashMap::new(),
        }
    }
}

pub fn st(w: &World) -> Option<&FactionsState> {
    w.resources.get::<FactionsState>()
}

fn st_mut(w: &mut World) -> &mut FactionsState {
    w.resources.get_mut::<FactionsState>()
}

/// A `TextDefinition` with only a number, as `{definition}` formats it.
pub(crate) fn number_text(n: i32) -> String {
    muo_io::text::TextDefinition::of_number(n).get_value()
}

// ---- lookups ----

/// `PlayerState.Find(mob)`: the faction index and the state.
pub fn player_state(w: &World, m: MobileId) -> Option<(usize, &PlayerState)> {
    let s = st(w)?;
    s.factions
        .iter()
        .enumerate()
        .find_map(|(f, d)| d.member(m).map(|p| (f, p)))
}

/// `Faction.Find(mob)`.
pub fn find(w: &World, m: MobileId) -> Option<usize> {
    player_state(w, m).map(|(f, _)| f)
}

/// `Faction.TryParse(s)`: the faction whose friendly name is `s`, ignoring case.
pub fn parse(s: &str) -> Option<usize> {
    FACTION_DEFS.iter().position(|d| d.friendly_name.eq_ic(s))
}

/// `Faction.Find(mob, inherit, creatureAllegiances)`.
pub fn find_full(
    w: &World,
    m: MobileId,
    inherit: bool,
    creature_allegiances: bool,
) -> Option<usize> {
    if let Some(f) = find(w, m) {
        return Some(f);
    }
    if inherit && muo_content::creature::cd(w, m).is_some() {
        if let Some(master) = muo_content::creature::control::get_master(w, m) {
            return find(w, master);
        }
        if creature_allegiances {
            return mobiles::guard_faction(w, m);
        }
    }
    None
}

/// `FactionItem.Find(item)`: the faction of an item that has a state, dropping an expired one on the way.
pub fn item_faction(w: &mut World, item: ItemId) -> Option<usize> {
    let f = st(w)?.attached.get(&item).copied()?;
    if item_expired(w, f, item) {
        detach(w, f, item);
        return None;
    }
    Some(f)
}

/// `item.FactionItemState != null`: the raw field, which `BaseArmor`/`BaseClothing`/`BaseWeapon` read in their
/// properties and `Faction` in its clean-up without `Find`, so an expired state still counts until a `Find` drops it.
pub fn item_has_state(w: &World, item: ItemId) -> bool {
    st(w).is_some_and(|s| s.attached.contains_key(&item))
}

fn item_expired(w: &World, faction: usize, item: ItemId) -> bool {
    let Some(s) = st(w) else { return true };
    let exp = s.factions[faction]
        .items
        .iter()
        .find(|fi| fi.item == Some(item))
        .map(|fi| fi.expiration);
    // `Item?.Deleted != false`
    if w.item(item).is_none() {
        return true;
    }
    exp.is_some_and(|e| e != muo_primitives::DateTime::MIN && w.now >= e)
}

fn is_faction_item_type(w: &World, item: ItemId) -> bool {
    w.item(item).is_some_and(|i| {
        i.ty.is(&armor::BASE_ARMOR)
            || i.ty.is(&clothing::BASE_CLOTHING)
            || i.ty.is(&weapon::BASE_WEAPON)
    })
}

// ---- FactionItemDefinition ----

/// `FactionItemDefinition`: the silver cost and the full name of the vendor class that sells the item.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FactionItemDefinition {
    pub silver_cost: i32,
    pub vendor_type: &'static str,
}

const METAL_ARMOR: FactionItemDefinition = FactionItemDefinition {
    silver_cost: 1000,
    vendor_type: "Server.Mobiles.Blacksmith",
};
const WEAPON: FactionItemDefinition = FactionItemDefinition {
    silver_cost: 1000,
    vendor_type: "Server.Mobiles.Blacksmith",
};
const RANGED_WEAPON: FactionItemDefinition = FactionItemDefinition {
    silver_cost: 1000,
    vendor_type: "Server.Mobiles.Bowyer",
};
const LEATHER_ARMOR: FactionItemDefinition = FactionItemDefinition {
    silver_cost: 750,
    vendor_type: "Server.Mobiles.Tailor",
};
const CLOTHING: FactionItemDefinition = FactionItemDefinition {
    silver_cost: 200,
    vendor_type: "Server.Mobiles.Tailor",
};
const SCROLL: FactionItemDefinition = FactionItemDefinition {
    silver_cost: 500,
    vendor_type: "Server.Mobiles.Mage",
};

/// `FactionItemDefinition.Identify(item)`.
pub fn identify_item(w: &World, item: ItemId) -> Option<FactionItemDefinition> {
    let ty = w.item(item)?.ty;
    if ty.is(&armor::BASE_ARMOR) {
        let leather = armor::data(w, item).is_some_and(|d| {
            muo_content::craft::resource_type(d.resource)
                == muo_content::craft::CraftResourceType::Leather
        });
        return Some(if leather { LEATHER_ARMOR } else { METAL_ARMOR });
    }
    if ty.is(&weapon::BASE_RANGED) {
        return Some(RANGED_WEAPON);
    }
    if ty.is(&weapon::BASE_WEAPON) {
        return Some(WEAPON);
    }
    if ty.is(&clothing::BASE_CLOTHING) {
        return Some(CLOTHING);
    }
    let mut t = Some(ty);
    while let Some(c) = t {
        if c.full_name == "Server.Items.SpellScroll" {
            return Some(SCROLL);
        }
        t = c.parent;
    }
    None
}

// ---- FactionItem ----

/// `FactionItem.Attach()`.
pub fn attach(w: &mut World, faction: usize, item: FactionItem) {
    if let Some(i) = item.item
        && is_faction_item_type(w, i)
    {
        st_mut(w).attached.insert(i, faction);
        equip_hooks::set_faction_item_state(w, i, true);
    }
    st_mut(w).factions[faction].items.push(item);
}

/// `FactionItem.Detach()`.
pub fn detach(w: &mut World, faction: usize, item: ItemId) {
    if st(w).is_some_and(|s| s.attached.contains_key(&item)) {
        st_mut(w).attached.remove(&item);
        equip_hooks::set_faction_item_state(w, item, false);
    }
    st_mut(w).factions[faction]
        .items
        .retain(|fi| fi.item != Some(item));
}

/// `FactionItem.CheckAttach()`.
fn check_attach(w: &mut World, faction: usize, fi: FactionItem) {
    let expired = fi.item.is_none_or(|i| {
        w.item(i).is_none()
            || (fi.expiration != muo_primitives::DateTime::MIN && w.now >= fi.expiration)
    });
    if !expired {
        attach(w, faction, fi);
    } else if let Some(i) = fi.item {
        // Detach on an item that was never attached: only the item side applies.
        detach(w, faction, i);
    }
}

/// `FactionItem.Imbue(item, faction, expire, hue)`.
pub fn imbue(w: &mut World, item: ItemId, faction: usize, expire: bool, hue: i32) -> ItemId {
    if !is_faction_item_type(w, item) {
        return item;
    }
    if item_faction(w, item).is_none() {
        attach(
            w,
            faction,
            FactionItem {
                item: Some(item),
                expiration: muo_primitives::DateTime::MIN,
            },
        );
    }
    if expire {
        let exp = w.now + ITEM_EXPIRATION;
        if let Some(fi) = st_mut(w).factions[faction]
            .items
            .iter_mut()
            .find(|fi| fi.item == Some(item))
        {
            fi.expiration = exp;
        }
    }
    muo_server::item::base::set_hue(w, item, hue);
    item
}

/// `FactionItem.GetMaxWearables(mob)`.
pub fn max_wearables(w: &World, m: MobileId) -> i32 {
    match player_state(w, m) {
        None => 0,
        Some((f, ps)) => {
            if is_commander(w, f, m) {
                9
            } else {
                FACTION_DEFS[f].ranks[ps.rank].max_wearables
            }
        }
    }
}

// ---- Faction ----

/// `IsCommander(mob)`.
pub fn is_commander(w: &World, f: usize, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.access_level >= muo_server::mobile::AccessLevel::GameMaster)
        || st(w).is_some_and(|s| s.factions[f].commander == Some(m))
}

/// `PlayerState.Invalidate()`.
pub fn invalidate_player(w: &mut World, m: MobileId) {
    invalidate(w, m);
}

fn invalidate(w: &mut World, m: MobileId) {
    properties::invalidate_properties(w, m);
}

/// `Broadcast(hue, text)`.
pub fn broadcast_text(w: &mut World, f: usize, hue: i32, text: &str) {
    let members: Vec<MobileId> = st(w).map_or_else(Vec::new, |s| {
        s.factions[f].members.iter().map(|p| p.mobile).collect()
    });
    for m in members {
        mm::send_message_hue(w, m, hue, text);
    }
}

/// `Broadcast(number)`.
pub fn broadcast(w: &mut World, f: usize, number: i32) {
    let members: Vec<MobileId> = st(w).map_or_else(Vec::new, |s| {
        s.factions[f].members.iter().map(|p| p.mobile).collect()
    });
    for m in members {
        mm::send_localized_message(w, m, number, "", 0x3B2);
    }
}

/// `EndBroadcast(from, text)`.
pub fn end_broadcast(w: &mut World, f: usize, from: MobileId, text: &str) {
    let (player, name) = w.mobile(from).map_or((false, String::new()), |m| {
        (
            m.access_level == muo_server::mobile::AccessLevel::Player,
            (m.ty.name)(w, from).unwrap_or_default(),
        )
    });
    if player {
        let now = w.now;
        st_mut(w).factions[f].register_broadcast(now);
    }
    let def = FACTION_DEFS[f];
    broadcast_text(
        w,
        f,
        def.hue_broadcast,
        &format!("{name} [Commander] {} : {text}", def.friendly_name),
    );
}

/// `Commander` setter.
pub fn set_commander(w: &mut World, f: usize, value: Option<MobileId>) {
    if let Some(old) = st(w).and_then(|s| s.factions[f].commander) {
        invalidate(w, old);
    }
    st_mut(w).factions[f].commander = value;
    if let Some(c) = value {
        mm::send_localized_message(w, c, 1042227, "", 0x3B2); // You have been elected Commander of your faction
        invalidate(w, c);
        if let Some((_, ps)) = player_state(w, c) {
            let (fin, sher) = (ps.finance, ps.sheriff);
            if let Some(t) = fin {
                set_town_finance(w, t, None);
            }
            if let Some(t) = sher {
                set_town_sheriff(w, t, None);
            }
        }
    }
}

/// `TownState.Sheriff` setter.
pub fn set_town_sheriff(w: &mut World, town: usize, value: Option<MobileId>) {
    let old = st(w).and_then(|s| s.towns[town].sheriff);
    if let Some(o) = old {
        set_ps_town(w, o, |p| p.sheriff = None);
    }
    st_mut(w).towns[town].sheriff = value;
    if let Some(n) = value {
        set_ps_town(w, n, |p| p.sheriff = Some(town));
    }
}

/// `TownState.Finance` setter.
pub fn set_town_finance(w: &mut World, town: usize, value: Option<MobileId>) {
    let old = st(w).and_then(|s| s.towns[town].finance);
    if let Some(o) = old {
        set_ps_town(w, o, |p| p.finance = None);
    }
    st_mut(w).towns[town].finance = value;
    if let Some(n) = value {
        set_ps_town(w, n, |p| p.finance = Some(town));
    }
}

fn set_ps_town(w: &mut World, m: MobileId, f: impl FnOnce(&mut PlayerState)) {
    let Some(s) = w
        .resources
        .get_mut::<FactionsState>()
        .factions
        .iter_mut()
        .find_map(|d| d.members.iter_mut().find(|p| p.mobile == m))
    else {
        return;
    };
    f(s);
    invalidate(w, m); // PlayerState.Invalidate()
}

/// `PlayerState.KillPoints` setter, with its `Invalidate()`.
pub fn set_kill_points(w: &mut World, m: MobileId, value: i32) {
    let Some(f) = find(w, m) else { return };
    let changed = st(w)
        .and_then(|s| s.factions[f].member(m))
        .is_some_and(|p| p.kill_points != value);
    st_mut(w).factions[f].set_kill_points(m, value);
    if changed {
        invalidate(w, m);
    }
}

fn kill_points(w: &World, m: MobileId) -> i32 {
    player_state(w, m).map_or(0, |(_, p)| p.kill_points)
}

/// `AddMember(mob)`.
pub fn add_member(w: &mut World, f: usize, mob: MobileId) {
    invalidate(w, mob); // new PlayerState(): Invalidate()
    st_mut(w).factions[f].insert_member(f, PlayerState::new(mob, f));

    if let Some(robe) = muo_content::factory::make_item(w, "Server.Items.Robe") {
        let hue = FACTION_DEFS[f].hue_primary;
        imbue(w, robe, f, false, hue);
        muo_server::mobile::part2::items::add_to_backpack(w, mob, robe);
    }
    mm::send_localized_message(w, mob, 1010374, "", 0x3B2); // You have been granted a robe which signifies your faction

    invalidate(w, mob);
    mb::delta(w, mob, MobileDelta::NOTO);

    fx::fixed_effect(w, mob, 0x373A, 10, 30);
    fx::play_sound(w, mob, 0x209);
}

/// `RemoveMember(mob)`.
pub fn remove_member(w: &mut World, mob: MobileId) {
    let Some((f, ps)) = player_state(w, mob) else {
        return;
    };
    let (finance, sheriff) = (ps.finance, ps.sheriff);
    let kp = ps.kill_points;
    // Ordinarily this never finds sigils: only with a leave delay under the return period, or a kick / ban.
    for sg in items::sigils_in_pack(w, mob) {
        items::return_home(w, sg);
    }
    st_mut(w).factions[f].remove_member_state(mob);

    invalidate(w, mob);
    mb::delta(w, mob, MobileDelta::NOTO);

    election_remove_candidate_if(w, f, mob);
    st_mut(w).factions[f].election.remove_voter(mob);

    if let Some(t) = finance {
        set_town_finance(w, t, None);
    }
    if let Some(t) = sheriff {
        set_town_sheriff(w, t, None);
    }
    if st(w).is_some_and(|s| s.factions[f].commander == Some(mob)) {
        set_commander(w, f, None);
    }
    // `playerMobile.ValidateEquipment()`
    muo_content::player_mobile::members::validate_equipment(w, mob);
    if kp > 0 {
        distribute_points(w, kp);
    }
}

fn election_remove_candidate_if(w: &mut World, f: usize, mob: MobileId) {
    if st(w).is_some_and(|s| s.factions[f].election.is_candidate(mob)) {
        election_remove_candidate(w, f, mob);
    }
}

/// `Election.AddCandidate(mob)`.
pub fn election_add_candidate(w: &mut World, f: usize, mob: MobileId) {
    if st(w).is_some_and(|s| s.factions[f].election.is_candidate(mob)) {
        return;
    }
    st_mut(w).factions[f]
        .election
        .candidates
        .push(state::Candidate {
            mobile: mob,
            voters: Vec::new(),
        });
    mm::send_localized_message(w, mob, 1010117, "", 0x3B2); // You are now running for office.
}

/// `Election.RemoveCandidate(mob)`.
pub fn election_remove_candidate(w: &mut World, f: usize, mob: MobileId) {
    let Some(i) = st(w).and_then(|s| s.factions[f].election.find_candidate(mob)) else {
        return;
    };
    st_mut(w).factions[f].election.candidates.remove(i);
    mm::send_localized_message(w, mob, 1038031, "", 0x3B2);
    let (state, count) = {
        let e = &st(w).expect("state").factions[f].election;
        (e.state, e.candidates.len())
    };
    if state == ElectionState::Election {
        if count == 1 {
            broadcast(w, f, 1038031);
            let winner = st(w).expect("state").factions[f].election.candidates[0].mobile;
            announce_winner(w, f, winner, 1038026, 1038028);
            let now = w.now;
            let e = &mut st_mut(w).factions[f].election;
            e.candidates.clear();
            e.set_state(ElectionState::Pending, now);
        } else if count == 0 {
            broadcast(w, f, 1038031);
            let now = w.now;
            let e = &mut st_mut(w).factions[f].election;
            e.candidates.clear();
            e.set_state(ElectionState::Pending, now);
        }
    }
}

/// The "faction leadership has not changed / has a new commander" tail the election code repeats.
fn announce_winner(w: &mut World, f: usize, winner: MobileId, unchanged: i32, changed: i32) {
    let same = find(w, winner) != Some(f)
        || st(w).is_some_and(|s| s.factions[f].commander == Some(winner));
    if same {
        broadcast(w, f, unchanged);
    } else {
        broadcast(w, f, changed);
        set_commander(w, f, Some(winner));
    }
}

/// `Election.Slice()` (every minute, `Faction.Election == this` is always so here).
pub fn election_slice(w: &mut World, f: usize) {
    let now = w.now;
    let (state, last, ncand) = {
        let e = &st(w).expect("state").factions[f].election;
        (e.state, e.last_state_time, e.candidates.len())
    };
    match state {
        ElectionState::Pending => {
            if last + Election::PENDING_PERIOD > now {
                return;
            }
            broadcast(w, f, 1038023); // Campaigning for the Faction Commander election has begun.
            let e = &mut st_mut(w).factions[f].election;
            e.candidates.clear();
            e.set_state(ElectionState::Campaign, now);
        }
        ElectionState::Campaign => {
            if last + Election::CAMPAIGN_PERIOD > now {
                return;
            }
            if ncand == 0 {
                broadcast(w, f, 1038025); // Nobody ran for office.
                st_mut(w).factions[f]
                    .election
                    .set_state(ElectionState::Pending, now);
            } else if ncand == 1 {
                broadcast(w, f, 1038029); // Only one member ran for office.
                let winner = st(w).expect("state").factions[f].election.candidates[0].mobile;
                announce_winner(w, f, winner, 1038026, 1038028);
                let e = &mut st_mut(w).factions[f].election;
                e.candidates.clear();
                e.set_state(ElectionState::Pending, now);
            } else {
                broadcast(w, f, 1038030);
                st_mut(w).factions[f]
                    .election
                    .set_state(ElectionState::Election, now);
            }
        }
        ElectionState::Election => {
            if last + Election::VOTING_PERIOD > now {
                return;
            }
            broadcast(w, f, 1038024); // The results for the Faction Commander election are in
            let cands: Vec<(MobileId, usize)> = st(w).expect("state").factions[f]
                .election
                .candidates
                .iter()
                .map(|c| (c.mobile, c.votes()))
                .collect();
            let mut winner: Option<(MobileId, usize)> = None;
            for (m, votes) in cands {
                if find(w, m) != Some(f) {
                    continue;
                }
                if winner.is_none_or(|(_, v)| votes > v) {
                    winner = Some((m, votes));
                }
            }
            match winner {
                None => broadcast(w, f, 1038026), // Faction leadership has not changed.
                Some((m, _)) if st(w).is_some_and(|s| s.factions[f].commander == Some(m)) => {
                    broadcast(w, f, 1038027); // The incumbent won the election.
                }
                Some((m, _)) => {
                    broadcast(w, f, 1038028); // The faction has a new commander.
                    set_commander(w, f, Some(m));
                }
            }
            let e = &mut st_mut(w).factions[f].election;
            e.candidates.clear();
            e.set_state(ElectionState::Pending, now);
        }
    }
}

/// `AwardSilver(mob, silver)`: the share the faction keeps is tithed; the rest is given to `mob`. Returns it.
pub fn award_silver(w: &mut World, f: usize, mob: MobileId, silver: i32) -> i32 {
    if silver <= 0 {
        return 0;
    }
    let tithed = silver * st(w).map_or(0, |s| s.factions[f].tithe) / 100;
    st_mut(w).factions[f].silver += tithed;
    let silver = silver - tithed;
    if silver > 0
        && let Some(item) = muo_content::factory::make_item(w, "Server.Factions.Silver")
    {
        muo_server::item::ops::set_amount(w, item, silver);
        muo_server::mobile::part2::items::add_to_backpack(w, mob, item);
    }
    silver
}

/// `DistributePoints(distrib)`: one point at a time to a random active player with kill points.
pub fn distribute_points(w: &mut World, distrib: i32) {
    let active = active_players(w);
    if !active.is_empty() {
        for _ in 0..distrib {
            let m = active[rnd::below(w, active.len() as i32) as usize];
            let kp = kill_points(w, m);
            set_kill_points(w, m, kp + 1);
        }
    }
}

fn active_players(w: &World) -> Vec<MobileId> {
    st(w).map_or_else(Vec::new, |s| {
        s.factions
            .iter()
            .flat_map(|d| d.members.iter())
            .filter(|p| p.kill_points > 0 && p.is_active)
            .map(|p| p.mobile)
            .collect()
    })
}

/// `HandleAtrophy()`: every 10 minutes.
pub fn handle_atrophy(w: &mut World) {
    let now = w.now;
    if st(w).is_none_or(|s| s.factions.iter().any(|d| !d.is_atrophy_ready(now))) {
        return;
    }
    let active = active_players(w);
    let mut distrib = 0;
    for f in 0..FACTION_DEFS.len() {
        let before: Vec<MobileId> = st(w).expect("state").factions[f]
            .members
            .iter()
            .map(|p| p.mobile)
            .collect();
        distrib += st_mut(w).factions[f].check_atrophy(now);
        for m in before {
            invalidate(w, m);
        }
    }
    if active.is_empty() {
        return;
    }
    for _ in 0..distrib {
        let m = active[rnd::below(w, active.len() as i32) as usize];
        let kp = kill_points(w, m);
        set_kill_points(w, m, kp + 1);
    }
}

/// `FindSmallestFaction()`.
pub fn smallest_faction(w: &World) -> Option<usize> {
    let s = st(w)?;
    let mut smallest: Option<usize> = None;
    for f in 0..s.factions.len() {
        if smallest.is_none_or(|sm| s.factions[f].members.len() < s.factions[sm].members.len()) {
            smallest = Some(f);
        }
    }
    smallest
}

/// `StabilityActive()`.
pub fn stability_active(w: &World) -> bool {
    st(w).is_some_and(|s| {
        s.factions
            .iter()
            .any(|d| d.members.len() > STABILITY_ACTIVATION)
    })
}

/// `CanHandleInflux(influx)`.
pub fn can_handle_influx(w: &World, f: usize, influx: usize) -> bool {
    if !stability_active(w) {
        return true;
    }
    match smallest_faction(w) {
        None => true,
        Some(sm) => {
            let s = st(w).expect("state");
            (s.factions[f].members.len() + influx) * 100 / STABILITY_FACTOR
                <= s.factions[sm].members.len()
        }
    }
}

/// `CheckLeaveTimer(mob)`.
pub fn check_leave_timer(w: &mut World, mob: MobileId) -> bool {
    let Some((_, ps)) = player_state(w, mob) else {
        return false;
    };
    if !ps.is_leaving() || ps.leaving + LEAVE_PERIOD >= w.now {
        return false;
    }
    mm::send_localized_message(w, mob, 1005163, "", 0x3B2); // You have now quit your faction
    remove_member(w, mob);
    true
}

/// `HonorLeadership_OnTarget`.
pub fn honor_leadership(w: &mut World, from: MobileId, recv: Option<MobileId>) {
    let Some(recv) = recv else {
        mm::send_localized_message(w, from, 1042496, "", 0x3B2); // You may only honor another player.
        return;
    };
    let Some((gf, _)) = player_state(w, from) else {
        return;
    };
    let same = find(w, recv) == Some(gf);
    if !same {
        mm::send_localized_message(w, from, 1042497, "", 0x3B2); // Only faction mates can be honored this way.
    } else if kill_points(w, from) < 5 {
        mm::send_localized_message(w, from, 1042499, "", 0x3B2); // You must have at least five kill points to honor them.
    } else {
        let now = w.now;
        if let Some(p) = w.resources.get_mut::<FactionsState>().factions[gf]
            .members
            .iter_mut()
            .find(|p| p.mobile == recv)
        {
            p.last_honor_time = now;
        }
        let g = kill_points(w, from);
        set_kill_points(w, from, g - 5);
        let r = kill_points(w, recv);
        set_kill_points(w, recv, r + 4);
        mm::send_localized_message(w, recv, 1042500, "", 0x3B2); // You have been honored with four kill points.
    }
}

// ---- death and skill loss ----

/// `InSkillLoss(mob)`.
pub fn in_skill_loss(w: &World, m: MobileId) -> bool {
    st(w).is_some_and(|s| s.skill_loss.contains_key(&m))
}

/// `ApplySkillLoss(mob)`.
pub fn apply_skill_loss(w: &mut World, mob: MobileId) {
    if in_skill_loss(w, mob) {
        return;
    }
    let mut mods = Vec::new();
    let len = w
        .mobile(mob)
        .and_then(|m| m.skills.as_ref())
        .map_or(0, |s| s.len());
    for i in 0..len {
        let Some(skill) = SkillName::from_i32(i as i32) else {
            continue;
        };
        let base = muo_server::skills::base(w, mob, skill);
        if base > 0.0 {
            let name = w
                .resources
                .get::<SkillTable>()
                .and_then(|t| t.info.get(i))
                .map_or_else(String::new, |s| s.name.clone());
            let m = SkillMod::new_default(
                w,
                skill,
                Some(&format!("{name}FactionSkillLoss")),
                true,
                -(base * SKILL_LOSS_FACTOR),
            );
            mods.push(m.id);
            mb::add_skill_mod(w, mob, m);
        }
    }
    let t = timer::start(w, SKILL_LOSS_PERIOD, TimeSpan::ZERO, 1, move |w| {
        clear_skill_loss(w, mob);
    });
    st_mut(w)
        .skill_loss
        .insert(mob, SkillLossContext { mods, timer: t });
}

/// `ClearSkillLoss(mob)`.
pub fn clear_skill_loss(w: &mut World, mob: MobileId) -> bool {
    let Some(mut ctx) = st_mut(w).skill_loss.remove(&mob) else {
        return false;
    };
    for m in ctx.mods.drain(..) {
        mb::remove_skill_mod(w, mob, m);
    }
    ctx.timer.cancel(w);
    true
}

/// `Faction.HandleDeath(victim, killer)`.
pub fn handle_death(w: &mut World, victim: MobileId, killer: Option<MobileId>) {
    let killer = killer
        .or_else(|| muo_server::mobile::part2::damage::find_most_recent_damager(w, victim, true));
    let killer_state = killer.and_then(|k| player_state(w, k).map(|(f, _)| f));

    for sg in items::sigils_in_pack(w, victim) {
        let killer_pack = killer.and_then(|k| w.mobile(k).and_then(|m| m.backpack));
        match (killer, killer_state, killer_pack) {
            (Some(k), Some(_), Some(pack)) => {
                if muo_content::creature::misc::distance_sqrt(w, k, victim) > 64.0 {
                    items::return_home(w, sg);
                    mm::send_localized_message(w, k, 1042230, "", 0x3B2); // The sigil has gone back to its home location.
                } else if items::exists_on(w, k) {
                    items::return_home(w, sg);
                    // The sigil has gone back to its home location because you already have a sigil.
                    mm::send_localized_message(w, k, 1010258, "", 0x3B2);
                } else if !muo_server::item::container::try_drop_item(w, pack, Some(k), sg, false) {
                    items::return_home(w, sg);
                    // The sigil has gone home because your backpack is full.
                    mm::send_localized_message(w, k, 1010259, "", 0x3B2);
                }
            }
            _ => {
                items::return_home(w, sg);
            }
        }
    }

    let (Some(killer), Some(kf)) = (killer, killer_state) else {
        return;
    };

    if let Some(ct) = muo_content::creature::ctype(w, victim) {
        // BaseCreature
        let victim_faction = mobiles::guard_faction(w, victim);
        let on_facet = w.mobile(victim).and_then(|m| m.map) == Some(FACET);
        if on_facet
            && let Some(vf) = victim_faction
            && kf != vf
        {
            let worth = (ct.faction_silver_worth)(w, victim);
            let silver = award_silver(w, kf, killer, worth);
            if silver > 0 {
                mm::send_localized_message(
                    w,
                    killer,
                    1042748,
                    &muo_content::items::gold::n0(silver as i64),
                    0x3B2,
                );
            }
        }
        if on_facet
            && (ct.get_ethic_allegiance)(w, victim, killer)
                == muo_content::creature::enums::Allegiance::Enemy
            && let Some(pl) = ethics::find_player(w, killer)
            && 100 - ethics::power(w, pl) > rnd::below(w, 100)
        {
            ethics::add_power(w, pl, 1, 1);
        }
        return;
    }

    let Some((vf, _)) = player_state(w, victim) else {
        return;
    };

    if muo_content::player_mobile::hooks::in_safe_zone(w, victim) {
        return;
    }

    if killer == victim || kf != vf {
        apply_skill_loss(w, victim);
    }

    if kf != vf {
        let vkp = kill_points(w, victim);
        if vkp <= -6 {
            mm::send_localized_message(w, killer, 501693, "", 0x3B2); // This victim is not worth enough to get kill points from.
            let (kepl, vepl) = (
                ethics::find_player(w, killer),
                ethics::find_player(w, victim),
            );
            let now = w.now;
            let can = can_give_silver(w, victim, killer, now);
            if let (Some(kepl), Some(vepl)) = (kepl, vepl)
                && ethics::power(w, vepl) > 0
                && can
            {
                let transfer = (ethics::power(w, vepl) / 5)
                    .max(1)
                    .min(100 - ethics::power(w, kepl));
                if transfer > 0 {
                    ethics::add_power(w, vepl, -((transfer + 1) / 2), 0);
                    ethics::add_power(w, kepl, transfer, transfer);
                    given_silver_to(w, victim, killer, now);
                }
            }
        } else {
            let award = (vkp / 10).clamp(1, 40);
            let now = w.now;
            if can_give_silver(w, victim, killer, now) {
                power::check_spawn(w, killer, victim);
                if vkp > 0 {
                    set_active(w, victim, true);
                    if rnd::below(w, 3) < 1 {
                        set_active(w, killer, true);
                    }
                    let silver = award_silver(w, kf, killer, award * 40);
                    if silver > 0 {
                        let vname = w
                            .mobile(victim)
                            .and_then(|m| (m.ty.name)(w, victim))
                            .unwrap_or_default();
                        mm::send_localized_message(
                            w,
                            killer,
                            1042736,
                            &format!(
                                "{} silver\t{vname}",
                                muo_content::items::gold::n0(silver as i64)
                            ),
                            0x3B2,
                        );
                    }
                }
                let v = kill_points(w, victim);
                set_kill_points(w, victim, v - award);
                let k = kill_points(w, killer);
                set_kill_points(w, killer, k + award);

                let offset = if award != 1 { 0 } else { 2 }; // for pluralization
                let vname = w
                    .mobile(victim)
                    .and_then(|m| (m.ty.name)(w, victim))
                    .unwrap_or_default();
                let kname = w
                    .mobile(killer)
                    .and_then(|m| (m.ty.name)(w, killer))
                    .unwrap_or_default();
                let args = format!("{award}\t{vname}\t{kname}");
                mm::send_localized_message(w, killer, 1042737 + offset, &args, 0x3B2);
                mm::send_localized_message(w, victim, 1042738 + offset, &args, 0x3B2);

                let (kepl, vepl) = (
                    ethics::find_player(w, killer),
                    ethics::find_player(w, victim),
                );
                if let (Some(kepl), Some(vepl)) = (kepl, vepl)
                    && ethics::power(w, vepl) > 0
                {
                    let mut transfer = (ethics::power(w, vepl) / 5).max(1);
                    if transfer > 100 - ethics::power(w, kepl) {
                        transfer = 100 - ethics::power(w, kepl);
                    }
                    if transfer > 0 {
                        ethics::add_power(w, vepl, -((transfer + 1) / 2), 0);
                        ethics::add_power(w, kepl, transfer, transfer);
                    }
                }
                given_silver_to(w, victim, killer, now);
            } else {
                mm::send_localized_message(w, killer, 1042231, "", 0x3B2);
            }
        }
    }
}

pub(crate) fn can_give_silver_pub(w: &mut World, victim: MobileId, to: MobileId) -> bool {
    let now = w.now;
    can_give_silver(w, victim, to, now)
}

pub(crate) fn given_silver_to_pub(w: &mut World, victim: MobileId, to: MobileId) {
    let now = w.now;
    given_silver_to(w, victim, to, now);
}

fn can_give_silver(
    w: &mut World,
    victim: MobileId,
    to: MobileId,
    now: muo_primitives::DateTime,
) -> bool {
    let Some(f) = find(w, victim) else {
        return true;
    };
    st_mut(w).factions[f]
        .members
        .iter_mut()
        .find(|p| p.mobile == victim)
        .is_none_or(|p| p.can_give_silver_to(to, now))
}

fn given_silver_to(w: &mut World, victim: MobileId, to: MobileId, now: muo_primitives::DateTime) {
    let Some(f) = find(w, victim) else { return };
    if let Some(p) = st_mut(w).factions[f]
        .members
        .iter_mut()
        .find(|p| p.mobile == victim)
    {
        p.on_given_silver_to(to, now);
    }
}

fn set_active(w: &mut World, m: MobileId, v: bool) {
    let Some(f) = find(w, m) else { return };
    if let Some(p) = st_mut(w).factions[f]
        .members
        .iter_mut()
        .find(|p| p.mobile == m)
    {
        p.is_active = v;
    }
}

// ---- persistence (FactionSystem) ----

struct FactionPayload;

impl GenericPayload for FactionPayload {
    fn name(&self) -> &str {
        "Factions"
    }

    fn serialize(&mut self, w: &World, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(0); // version
        let Some(s) = st(w) else { return };
        for (f, d) in s.factions.iter().enumerate() {
            d.write(w, f, writer);
        }
        for (t, d) in s.towns.iter().enumerate() {
            d.write(w, t, writer);
        }
    }

    fn deserialize(&mut self, w: &mut World, reader: &mut dyn GenericReader) -> muo_io::Result<()> {
        let _version = reader.read_encoded_int()?;
        for _ in 0..FACTION_DEFS.len() {
            let (f, mut d) = FactionData::read(w, reader)?;
            let pending = std::mem::take(&mut d.items);
            st_mut(w).factions[f] = d;
            // `Timer.StartTimer(factionItem.CheckAttach)`: attached once the world is up.
            for fi in pending {
                timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                    check_attach(w, f, fi)
                });
            }
        }
        for _ in 0..TOWN_DEFS.len() {
            let (t, d) = TownData::read(w, reader)?;
            st_mut(w).towns[t] = d;
        }
        Ok(())
    }
}

// ---- hooks ----

/// `PlayerMobile.GetProperties`' faction lines.
fn add_properties(w: &World, m: MobileId, list: &mut PropertyList) {
    let Some((f, ps)) = player_state(w, m) else {
        return;
    };
    let def = FACTION_DEFS[f];
    let prop = number_text(def.prop_name);
    if st(w).is_some_and(|s| s.factions[f].commander == Some(m)) {
        list.add_str(1042733, &prop); // Commanding Lord of the ~1_FACTION_NAME~
    } else if let Some(t) = ps.sheriff {
        // The Sheriff of  ~1_CITY~, ~2_FACTION_NAME~
        list.add_str(1042734, &format!("{}\t{prop}", TOWN_DEFS[t].friendly_name));
    } else if let Some(t) = ps.finance {
        // The Finance Minister of ~1_CITY~, ~2_FACTION_NAME~
        list.add_str(1042735, &format!("{}\t{prop}", TOWN_DEFS[t].friendly_name));
    } else if let Some(title) = merchant_title(ps.merchant_title) {
        list.add_str(1060776, &format!("{}\t{prop}", number_text(title))); // ~1_val~, ~2_val~
    } else {
        list.add_str(
            1060776,
            &format!("{}\t{prop}", number_text(def.ranks[ps.rank].title)),
        );
    }
}

/// `MerchantTitles.GetInfo(title)?.Title`: the cliloc number.
fn merchant_title(t: i32) -> Option<i32> {
    // Scribe, Carpenter, Tinker, Blacksmith, Bowyer, Tailor
    const TITLES: [i32; 6] = [1060773, 1060774, 1022984, 1023016, 1023022, 1022982];
    usize::try_from(t - 1)
        .ok()
        .and_then(|i| TITLES.get(i).copied())
}

/// `PlayerMobile.OnSingleClick`'s label: `(text, ascii, hue)`.
fn single_click_label(w: &World, m: MobileId, from: MobileId) -> Option<(String, bool, i32)> {
    let (f, ps) = player_state(w, m)?;
    let def = FACTION_DEFS[f];
    let mut ascii = false;
    let text = if st(w).is_some_and(|s| s.factions[f].commander == Some(m)) {
        let lord = if w.mobile(m).is_some_and(|x| x.female) {
            "(Commanding Lady of the "
        } else {
            "(Commanding Lord of the "
        };
        format!("{lord}{})", def.friendly_name)
    } else if let Some(t) = ps.sheriff {
        format!(
            "(The Sheriff of {}, {})",
            TOWN_DEFS[t].friendly_name, def.friendly_name
        )
    } else if let Some(t) = ps.finance {
        format!(
            "(The Finance Minister of {}, {})",
            TOWN_DEFS[t].friendly_name, def.friendly_name
        )
    } else {
        ascii = true;
        // `Title?.String ?? Rank.Title.String`: both titles are cliloc-only definitions, so `String` is null and
        // the label reads "(, Faction)" (bug stays).
        format!("(, {})", def.friendly_name)
    };
    let hue = if find(w, from) == Some(f) { 98 } else { 38 };
    Some((text, ascii, hue))
}

/// `Faction.Configure` / `FactionSystem.Configure` / `Faction.Initialize`.
pub fn configure(w: &mut World) {
    use muo_content::player_mobile::hooks as ph;
    let enabled = w.config.get_setting("factions.enabled", false);
    *w.resources.get_mut::<FactionsState>() = FactionsState {
        enabled,
        ..FactionsState::default()
    };
    if enabled {
        let id = w
            .persistence
            .register(10, Box::new(GenericPersistence::new(FactionPayload)));
        st_mut(w).persistence_id = Some(id);
    }
    flow::register_commands(w);
    traps::configure(w);

    let h = w.resources.get_mut::<ph::SystemHooks>();
    h.faction_of = Some(|w, m| find(w, m).map(|f| f as i32));
    h.faction_item_faction = Some(|w, item| item_faction(w, item).map(|f| f as i32));
    h.faction_max_wearables = Some(max_wearables);
    h.faction_facet = Some(|_| Some(FACET));
    h.faction_abbreviation =
        Some(|w, m| find(w, m).map(|f| format!("[{}]", FACTION_DEFS[f].abbreviation)));
    h.faction_handle_death = Some(handle_death);
    h.faction_remove_member = Some(remove_member);
    h.faction_add_properties = Some(add_properties);
    h.faction_single_click_label = Some(single_click_label);

    let n = w
        .resources
        .get_mut::<muo_content::misc::notoriety::NotorietyHooks>();
    n.faction = Some(|w, m| find_full(w, m, true, true).map(|f| f as u32));
    n.faction_inherit = Some(|w, m| find_full(w, m, true, false).map(|f| f as u32));
    n.faction_facet = Some(FACET);

    w.resources
        .get_mut::<equip_hooks::EquipHooks>()
        .faction_item = Some(item_has_state);

    muo_content::spells::hooks::hooks_mut(w).is_faction_stronghold =
        Some(|_, map, p| is_stronghold(map, p));

    let c = muo_content::hooks::hooks_mut(w);
    c.faction_allegiance = Some(|w, creature, mob| {
        use muo_content::creature::enums::Allegiance;
        // `FactionAllegiance` is the guard's faction or the class's own (Daemon: Shadowlords); else none.
        let own = muo_content::creature::ctype(w, creature)
            .and_then(|c| (c.faction_allegiance)(w, creature))
            .map(|i| i as usize)
            .or_else(|| mobiles::guard_faction(w, creature));
        let Some(own) = own else {
            return Allegiance::None;
        };
        if w.mobile(mob).and_then(|m| m.map) != Some(FACET) {
            return Allegiance::None;
        }
        match find_full(w, mob, true, false) {
            None => Allegiance::None,
            Some(f) if f == own => Allegiance::Ally,
            Some(_) => Allegiance::Enemy,
        }
    });
}

/// `FactionSystem.Enable()`: turns the persistence on.
pub fn enable(w: &mut World) {
    if st(w).is_none_or(|s| s.enabled) {
        return;
    }
    let id = w
        .persistence
        .register(10, Box::new(GenericPersistence::new(FactionPayload)));
    st_mut(w).persistence_id = Some(id);
    st_mut(w).enabled = true;
    let _ = w.config.set_setting("factions.enabled", &true);
}

/// `FactionSystem.Disable()`: turns the persistence off (the faction objects stay).
pub fn disable(w: &mut World) {
    if st(w).is_none_or(|s| !s.enabled) {
        return;
    }
    if let Some(id) = st_mut(w).persistence_id.take() {
        w.persistence.unregister(id);
    }
    st_mut(w).enabled = false;
    let _ = w.config.set_setting("factions.enabled", &false);
}

/// `Faction.Initialize`: the atrophy and election clocks.
pub fn initialize(w: &mut World) {
    timer::start(
        w,
        TimeSpan::from_mins(1),
        TimeSpan::from_mins(10),
        0,
        handle_atrophy,
    );
    for f in 0..FACTION_DEFS.len() {
        timer::start(
            w,
            TimeSpan::from_mins(1),
            TimeSpan::from_mins(1),
            0,
            move |w| election_slice(w, f),
        );
    }
    timer::start(
        w,
        TimeSpan::from_secs(30),
        TimeSpan::from_secs(30),
        0,
        flow::process_tick,
    );
    // `EventSink_Logout`: the sigils in the backpack go home.
    w.events.logout.subscribe(|w, m| {
        for sg in items::sigils_in_pack(w, *m) {
            items::return_home(w, sg);
        }
    });
    town::initialize(w);
    mobiles::initialize(w);
    power::create_stronghold_regions(w);
    // `Keywords.Initialize`
    w.events.speech.subscribe(flow::on_speech);
    // `Faction.OnLogin` ([OnEvent(PlayerLoginEvent)]) -> `CheckLeaveTimer`.
    muo_content::network::events::subscribe_ranked::<muo_content::network::events::PlayerLogin>(
        w,
        6,
        |w, e| {
            check_leave_timer(w, e.0);
        },
    );
}

pub use power::stronghold_region_faction;

/// `StrongholdRegion` at a location: inside the area of a faction's stronghold on `Faction.Facet`.
pub fn is_stronghold(map: MapId, p: muo_primitives::Point3D) -> bool {
    map == FACET
        && tables::STRONGHOLD_AREAS
            .iter()
            .flat_map(|a| a.iter())
            .any(|&(x, y, wd, h)| p.x >= x && p.y >= y && p.x < x + wd && p.y < y + h)
}
