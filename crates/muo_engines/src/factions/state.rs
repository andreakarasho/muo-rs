//! `FactionState`, `PlayerState`, `TownState`, `Election`, `FactionItem` and `SilverGivenEntry`:
//! the data of the faction system and the logic that needs no `World` (rank bookkeeping, atrophy, stability), plus
//! the save layout of each (`Serialize` and the `IGenericReader` constructors, every version).
//!
//! A faction or town is the index into the tables of [`super::tables`] (`Reflector`'s lists; the save writes
//! `index + 1`). `members` is one list: kill-point ranked players first (`0..zero_rank_offset`), the rest after.

use std::net::{IpAddr, Ipv4Addr};

use muo_io::{GenericReader, GenericWriter, IoError, Result};
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};

use super::tables::{FACTION_DEFS, TOWN_DEFS};

pub const BROADCASTS_PER_PERIOD: usize = 2;
pub const BROADCAST_PERIOD: TimeSpan = TimeSpan::from_hours(1);
pub const ATROPHY_PERIOD: TimeSpan = TimeSpan::from_hours(47);
pub const SILVER_EXPIRE_PERIOD: TimeSpan = TimeSpan::from_hours(3);
/// `FactionItem.ExpirationPeriod`.
pub const ITEM_EXPIRATION: TimeSpan = TimeSpan::from_days(21);

/// `Faction.WriteReference`: the index in `Faction.Factions` plus one, zero for null.
pub fn write_faction_ref(writer: &mut dyn GenericWriter, f: Option<usize>) {
    writer.write_encoded_int(f.map_or(0, |i| i as i32 + 1));
}

/// `Faction.ReadReference`.
pub fn read_faction_ref(reader: &mut dyn GenericReader) -> Result<Option<usize>> {
    let idx = reader.read_encoded_int()? - 1;
    Ok((idx >= 0 && (idx as usize) < FACTION_DEFS.len()).then_some(idx as usize))
}

/// `Town.WriteReference`.
pub fn write_town_ref(writer: &mut dyn GenericWriter, t: Option<usize>) {
    writer.write_encoded_int(t.map_or(0, |i| i as i32 + 1));
}

/// `Town.ReadReference`.
pub fn read_town_ref(reader: &mut dyn GenericReader) -> Result<Option<usize>> {
    let idx = reader.read_encoded_int()? - 1;
    Ok((idx >= 0 && (idx as usize) < TOWN_DEFS.len()).then_some(idx as usize))
}

fn date(reader: &mut dyn GenericReader) -> Result<DateTime> {
    Ok(DateTime::from_ticks(reader.read_date_time_ticks()?))
}

/// `ElectionState`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum ElectionState {
    #[default]
    Pending,
    Campaign,
    Election,
}

impl ElectionState {
    /// The enum member's name (`{election.State}`).
    pub fn name(self) -> &'static str {
        match self {
            ElectionState::Pending => "Pending",
            ElectionState::Campaign => "Campaign",
            ElectionState::Election => "Election",
        }
    }

    fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Campaign,
            2 => Self::Election,
            _ => Self::Pending,
        }
    }
}

/// `Voter`.
#[derive(Clone, Debug)]
pub struct Voter {
    pub from: MobileId,
    pub address: IpAddr,
    pub time: DateTime,
}

/// `Candidate`.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub mobile: MobileId,
    pub voters: Vec<Voter>,
}

impl Candidate {
    pub fn votes(&self) -> usize {
        self.voters.len()
    }

    /// `Candidate(IGenericReader)`; `None` when the candidate's mobile is gone.
    fn read(w: &World, reader: &mut dyn GenericReader) -> Result<Option<Candidate>> {
        let version = reader.read_encoded_int()?;
        let mut voters = Vec::new();
        let mobile = match version {
            1 => {
                let mobile = reader.read_mobile(w)?;
                let count = reader.read_encoded_int()?;
                for _ in 0..count {
                    let _version = reader.read_encoded_int()?;
                    let from = reader.read_mobile(w)?;
                    let address = reader.read_ip_address()?;
                    let time = date(reader)?;
                    if let Some(from) = from {
                        voters.push(Voter {
                            from,
                            address,
                            time,
                        });
                    }
                }
                mobile
            }
            0 => {
                let mobile = reader.read_mobile(w)?;
                for from in reader.read_mobile_list(w)? {
                    // `new Voter(mob, candidate)`: no connection to take an address from.
                    voters.push(Voter {
                        from,
                        address: IpAddr::V4(Ipv4Addr::BROADCAST),
                        time: w.now,
                    });
                }
                mobile
            }
            _ => return Ok(None),
        };
        Ok(mobile.map(|mobile| Candidate { mobile, voters }))
    }

    fn write(&self, w: &World, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(1); // version
        writer.write_mobile(w, Some(self.mobile));
        writer.write_encoded_int(self.voters.len() as i32);
        for v in &self.voters {
            writer.write_encoded_int(0);
            writer.write_mobile(w, Some(v.from));
            writer.write_ip_address(v.address);
            writer.write_date_time_ticks(v.time.ticks());
        }
    }
}

/// `Election`.
#[derive(Clone, Debug, Default)]
pub struct Election {
    pub last_state_time: DateTime,
    pub state: ElectionState,
    pub candidates: Vec<Candidate>,
}

impl Election {
    pub const MAX_CANDIDATES: usize = 10;
    pub const CANDIDATE_RANK: i32 = 5;
    pub const PENDING_PERIOD: TimeSpan = TimeSpan::from_days(5);
    pub const CAMPAIGN_PERIOD: TimeSpan = TimeSpan::from_days(1);
    pub const VOTING_PERIOD: TimeSpan = TimeSpan::from_days(3);

    /// `State` setter.
    pub fn set_state(&mut self, state: ElectionState, now: DateTime) {
        self.state = state;
        self.last_state_time = now;
    }

    pub fn find_candidate(&self, m: MobileId) -> Option<usize> {
        self.candidates.iter().position(|c| c.mobile == m)
    }

    pub fn is_candidate(&self, m: MobileId) -> bool {
        self.find_candidate(m).is_some()
    }

    /// `FindVoter(mob)`: the candidate `mob` voted for.
    pub fn find_voter(&self, m: MobileId) -> Option<usize> {
        self.candidates
            .iter()
            .position(|c| c.voters.iter().any(|v| v.from == m))
    }

    pub fn has_voted(&self, m: MobileId) -> bool {
        self.find_voter(m).is_some()
    }

    pub fn can_vote(&self, m: MobileId) -> bool {
        self.state == ElectionState::Election && !self.has_voted(m)
    }

    /// `RemoveVoter(mob)`.
    pub fn remove_voter(&mut self, m: MobileId) {
        if self.state == ElectionState::Election {
            for c in &mut self.candidates {
                c.voters.retain(|v| v.from != m);
            }
        }
    }

    /// `NextStateTime`.
    pub fn next_state_time(&self, now: DateTime) -> TimeSpan {
        let period = match self.state {
            ElectionState::Pending => Self::PENDING_PERIOD,
            ElectionState::Election => Self::VOTING_PERIOD,
            ElectionState::Campaign => Self::CAMPAIGN_PERIOD,
        };
        let until = self.last_state_time + period - now;
        if until < TimeSpan::ZERO {
            TimeSpan::ZERO
        } else {
            until
        }
    }

    /// `Election(IGenericReader)`; the faction reference it carries is read and dropped (a state reads its own).
    fn read(w: &World, reader: &mut dyn GenericReader) -> Result<Election> {
        let mut e = Election::default();
        let version = reader.read_encoded_int()?;
        if version == 0 {
            let _faction = read_faction_ref(reader)?;
            e.last_state_time = date(reader)?;
            e.state = ElectionState::from_i32(reader.read_encoded_int()?);
            let count = reader.read_encoded_int()?;
            for _ in 0..count {
                if let Some(c) = Candidate::read(w, reader)? {
                    e.candidates.push(c);
                }
            }
        }
        Ok(e)
    }

    fn write(&self, w: &World, faction: usize, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(0); // version
        write_faction_ref(writer, Some(faction));
        writer.write_date_time_ticks(self.last_state_time.ticks());
        writer.write_encoded_int(self.state as i32);
        writer.write_encoded_int(self.candidates.len() as i32);
        for c in &self.candidates {
            c.write(w, writer);
        }
    }
}

/// `FactionItem`: an item that belongs to a faction, optionally expiring.
#[derive(Copy, Clone, Debug)]
pub struct FactionItem {
    pub item: Option<ItemId>,
    pub expiration: DateTime,
}

/// `SilverGivenEntry`.
#[derive(Copy, Clone, Debug)]
pub struct SilverGiven {
    pub given_to: MobileId,
    pub time_of_gift: DateTime,
}

impl SilverGiven {
    pub fn is_expired(&self, now: DateTime) -> bool {
        self.time_of_gift + SILVER_EXPIRE_PERIOD < now
    }
}

/// `PlayerState`.
#[derive(Clone, Debug)]
pub struct PlayerState {
    pub mobile: MobileId,
    pub faction: usize,
    pub kill_points: i32,
    /// `MerchantTitle` as its number (0 is `None`).
    pub merchant_title: i32,
    pub sheriff: Option<usize>,
    pub finance: Option<usize>,
    pub silver_given: Option<Vec<SilverGiven>>,
    pub rank_index: i32,
    /// `Rank`: the index in the faction's `ranks`, the cached `m_Rank`.
    pub rank: usize,
    pub last_honor_time: DateTime,
    pub leaving: DateTime,
    pub is_active: bool,
}

impl PlayerState {
    /// `new PlayerState(mob, faction, owner)`: the lowest rank, not yet ranked.
    pub fn new(mobile: MobileId, faction: usize) -> PlayerState {
        PlayerState {
            mobile,
            faction,
            kill_points: 0,
            merchant_title: 0,
            sheriff: None,
            finance: None,
            silver_given: None,
            rank_index: -1,
            rank: FACTION_DEFS[faction].ranks.len().saturating_sub(1),
            last_honor_time: DateTime::MIN,
            leaving: DateTime::MIN,
            is_active: false,
        }
    }

    pub fn is_leaving(&self) -> bool {
        self.leaving > DateTime::MIN
    }

    /// `CanGiveSilverTo(mob)`: forgets expired entries on the way.
    pub fn can_give_silver_to(&mut self, mob: MobileId, now: DateTime) -> bool {
        if let Some(list) = &mut self.silver_given {
            let mut i = 0;
            while i < list.len() {
                if list[i].is_expired(now) {
                    list.remove(i);
                } else if list[i].given_to == mob {
                    return false;
                } else {
                    i += 1;
                }
            }
        }
        true
    }

    pub fn on_given_silver_to(&mut self, mob: MobileId, now: DateTime) {
        self.silver_given
            .get_or_insert_with(Vec::new)
            .push(SilverGiven {
                given_to: mob,
                time_of_gift: now,
            });
    }

    /// `PlayerState(IGenericReader, faction, owner)`; `None` when its mobile is gone (the state is dropped).
    fn read(
        w: &World,
        reader: &mut dyn GenericReader,
        faction: usize,
    ) -> Result<Option<PlayerState>> {
        let version = reader.read_encoded_int()?;
        let mut is_active = false;
        let mut last_honor_time = DateTime::MIN;
        if version == 1 {
            is_active = reader.read_bool()?;
            last_honor_time = date(reader)?;
        }
        if !(0..=1).contains(&version) {
            return Ok(None);
        }
        let mobile = reader.read_mobile(w)?;
        let kill_points = reader.read_encoded_int()?;
        let merchant_title = reader.read_encoded_int()?;
        let leaving = date(reader)?;
        Ok(mobile.map(|m| {
            let mut ps = PlayerState::new(m, faction);
            ps.is_active = is_active;
            ps.last_honor_time = last_honor_time;
            ps.kill_points = kill_points;
            ps.merchant_title = merchant_title;
            ps.leaving = leaving;
            ps
        }))
    }

    fn write(&self, w: &World, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(1); // version
        writer.write_bool(self.is_active);
        writer.write_date_time_ticks(self.last_honor_time.ticks());
        writer.write_mobile(w, Some(self.mobile));
        writer.write_encoded_int(self.kill_points);
        writer.write_encoded_int(self.merchant_title);
        writer.write_date_time_ticks(self.leaving.ticks());
    }
}

/// `FactionState` (and the live part of `Faction`: `ZeroRankOffset`).
#[derive(Clone, Debug)]
pub struct FactionData {
    pub commander: Option<MobileId>,
    pub tithe: i32,
    pub silver: i32,
    pub members: Vec<PlayerState>,
    pub zero_rank_offset: i32,
    pub last_atrophy: DateTime,
    pub last_broadcasts: [DateTime; BROADCASTS_PER_PERIOD],
    pub election: Election,
    pub items: Vec<FactionItem>,
    /// `Traps`: `BaseFactionTrap` items.
    pub traps: Vec<ItemId>,
}

impl Default for FactionData {
    /// `new FactionState(faction)`.
    fn default() -> Self {
        FactionData {
            commander: None,
            tithe: 50,
            silver: 0,
            members: Vec::new(),
            zero_rank_offset: 0,
            last_atrophy: DateTime::MIN,
            last_broadcasts: [DateTime::MIN; BROADCASTS_PER_PERIOD],
            election: Election::default(),
            items: Vec::new(),
            traps: Vec::new(),
        }
    }
}

impl FactionData {
    pub fn member(&self, m: MobileId) -> Option<&PlayerState> {
        self.members.iter().find(|p| p.mobile == m)
    }

    pub fn member_index(&self, m: MobileId) -> Option<usize> {
        self.members.iter().position(|p| p.mobile == m)
    }

    /// `FactionMessageReady`.
    pub fn message_ready(&self, now: DateTime) -> bool {
        self.last_broadcasts
            .iter()
            .any(|t| now >= *t + BROADCAST_PERIOD)
    }

    /// `RegisterBroadcast`.
    pub fn register_broadcast(&mut self, now: DateTime) {
        if let Some(t) = self
            .last_broadcasts
            .iter_mut()
            .find(|t| now >= **t + BROADCAST_PERIOD)
        {
            *t = now;
        }
    }

    /// `IsAtrophyReady`.
    pub fn is_atrophy_ready(&self, now: DateTime) -> bool {
        now >= self.last_atrophy + ATROPHY_PERIOD
    }

    /// `CheckAtrophy`: the kill points taken from inactive players, to be handed out again.
    pub fn check_atrophy(&mut self, now: DateTime) -> i32 {
        if now < self.last_atrophy + ATROPHY_PERIOD {
            return 0;
        }
        let mut distrib = 0;
        self.last_atrophy = now;
        let mobiles: Vec<MobileId> = self.members.iter().map(|p| p.mobile).collect();
        for m in mobiles {
            let Some(i) = self.member_index(m) else {
                continue;
            };
            if self.members[i].is_active {
                self.members[i].is_active = false;
                continue;
            }
            let kp = self.members[i].kill_points;
            if kp > 0 {
                let atrophy = (kp + 9) / 10;
                self.set_kill_points(m, kp - atrophy);
                distrib += atrophy;
            }
        }
        distrib
    }

    /// `UpdateRank()` of the member at `i`.
    pub fn update_rank(&mut self, faction: usize, i: usize) {
        let ranks = FACTION_DEFS[faction].ranks;
        if ranks.is_empty() {
            return;
        }
        let count = self.members.len() as i32;
        let ri = self.members[i].rank_index;
        let percent = if count == 1 {
            1000
        } else if ri == -1 || self.zero_rank_offset <= 0 {
            0
        } else {
            (self.zero_rank_offset - ri) * 1000 / self.zero_rank_offset
        };
        // Ranks run required-descending ending at 0; a negative percent matches none and keeps the lowest.
        let mut rank = ranks.len() - 1;
        for (n, check) in ranks.iter().enumerate() {
            if percent >= check.required {
                rank = n;
                break;
            }
        }
        self.members[i].rank = rank;
    }

    /// `RankIndex` setter of the member at `i`.
    fn set_rank_index(&mut self, i: usize, value: i32) {
        if self.members[i].rank_index != value {
            self.members[i].rank_index = value;
            let faction = self.members[i].faction;
            self.update_rank(faction, i);
        }
    }

    /// `KillPoints` setter: keeps `members` ordered by kill points.
    pub fn set_kill_points(&mut self, m: MobileId, value: i32) {
        let Some(mut i) = self.member_index(m) else {
            return;
        };
        let faction = self.members[i].faction;
        let old = self.members[i].kill_points;
        if old == value {
            return;
        }
        if value > old {
            if old <= 0 {
                if value <= 0 {
                    self.members[i].kill_points = value;
                    return;
                }
                let me = self.members.remove(i);
                let at = (self.zero_rank_offset as usize).min(self.members.len());
                self.members.insert(at, me);
                i = at;
                // Direct, not through RankIndex: ZeroRankOffset is mid-update.
                self.members[i].rank_index = self.zero_rank_offset;
                self.zero_rank_offset += 1;
            }
            while self.members[i].rank_index > 0 {
                let ri = self.members[i].rank_index as usize;
                if value > self.members[ri - 1].kill_points {
                    // this moves to ri - 1 (rank index down by one), the other to ri (up by one)
                    self.members.swap(ri, ri - 1);
                    self.set_rank_index(ri - 1, ri as i32 - 1);
                    self.set_rank_index(ri, ri as i32);
                    i = ri - 1;
                } else {
                    break;
                }
            }
        } else if value <= 0 {
            if old <= 0 {
                self.members[i].kill_points = value;
                return;
            }
            while self.members[i].rank_index + 1 < self.zero_rank_offset {
                let ri = self.members[i].rank_index as usize;
                self.members.swap(ri, ri + 1);
                self.set_rank_index(ri + 1, ri as i32 + 1);
                self.set_rank_index(ri, ri as i32);
                i = ri + 1;
            }
            self.members[i].rank_index = -1;
            self.zero_rank_offset -= 1;
        } else {
            while self.members[i].rank_index + 1 < self.zero_rank_offset {
                let ri = self.members[i].rank_index as usize;
                if value < self.members[ri + 1].kill_points {
                    self.members.swap(ri, ri + 1);
                    self.set_rank_index(ri + 1, ri as i32 + 1);
                    self.set_rank_index(ri, ri as i32);
                    i = ri + 1;
                } else {
                    break;
                }
            }
        }
        self.members[i].kill_points = value;
        self.update_rank(faction, i);
    }

    /// `RemoveMember`/`RemovePlayerState`'s list work: a ranked member sinks to the end of the ranked block,
    /// `ZeroRankOffset` shrinks, and the state is removed and returned.
    pub fn remove_member_state(&mut self, m: MobileId) -> Option<PlayerState> {
        let mut i = self.member_index(m)?;
        if self.members[i].rank_index != -1 {
            while self.members[i].rank_index + 1 < self.zero_rank_offset {
                let ri = self.members[i].rank_index as usize;
                self.members.swap(ri, ri + 1);
                self.members[ri + 1].rank_index += 1;
                self.members[ri].rank_index -= 1;
                i = ri + 1;
            }
            self.zero_rank_offset -= 1;
        }
        Some(self.members.remove(i))
    }

    /// `AddMember`'s list work: `Members.Insert(ZeroRankOffset, state)`, then the new member is ranked.
    pub fn insert_member(&mut self, faction: usize, state: PlayerState) {
        let at = (self.zero_rank_offset as usize).min(self.members.len());
        self.members.insert(at, state);
        self.update_rank(faction, at);
    }

    /// `FactionState(IGenericReader)`; the index of the faction the record itself names.
    pub fn read(w: &World, reader: &mut dyn GenericReader) -> Result<(usize, FactionData)> {
        let version = reader.read_encoded_int()?;
        let mut d = FactionData::default();
        if version >= 5 {
            d.last_atrophy = date(reader)?;
        }
        if version >= 4 {
            let count = reader.read_encoded_int()?;
            for i in 0..count {
                let time = date(reader)?;
                if (i as usize) < BROADCASTS_PER_PERIOD {
                    d.last_broadcasts[i as usize] = time;
                }
            }
        }
        let mut election = None;
        if version >= 1 {
            election = Some(Election::read(w, reader)?);
        }
        let faction = read_faction_ref(reader)?
            .ok_or_else(|| IoError::InvalidData("FactionState names no faction".into()))?;
        d.commander = reader.read_mobile(w)?;
        if version < 5 {
            d.last_atrophy = w.now;
        }
        if version < 4 {
            d.last_broadcasts[0] = date(reader)?;
        }
        d.tithe = reader.read_encoded_int()?;
        d.silver = reader.read_encoded_int()?;
        let member_count = reader.read_encoded_int()?;
        for _ in 0..member_count {
            if let Some(ps) = PlayerState::read(w, reader, faction)? {
                d.members.push(ps);
            }
        }
        d.zero_rank_offset = d.members.len() as i32;
        // `Members.Sort()` is List.Sort: ordered by descending kill points.
        muo_primitives::dotnet::net_sort(&mut d.members, |a, b| b.kill_points.cmp(&a.kill_points));
        for i in (0..d.members.len()).rev() {
            if d.members[i].kill_points <= 0 {
                d.zero_rank_offset = i as i32;
            } else {
                d.members[i].rank_index = i as i32;
            }
        }
        // Nothing computes rank on read, so rank everyone now that the ordering has settled.
        for i in 0..d.members.len() {
            d.update_rank(faction, i);
        }
        if version >= 2 {
            let n = reader.read_encoded_int()?;
            for _ in 0..n {
                let _version = reader.read_encoded_int()?;
                let item = reader.read_item(w)?;
                let expiration = date(reader)?;
                d.items.push(FactionItem { item, expiration });
            }
        }
        if version >= 3 {
            let n = reader.read_encoded_int()?;
            for _ in 0..n {
                // `reader.ReadEntity<Item>() is BaseFactionTrap trap && !trap.CheckDecay()`
                if let Some(t) = reader.read_item(w)?
                    && super::traps::is_trap(w, t)
                    && !super::traps::has_decayed(w, t)
                {
                    d.traps.push(t);
                }
            }
        }
        d.election = election.unwrap_or_default();
        Ok((faction, d))
    }

    pub fn write(&self, w: &World, faction: usize, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(5); // version
        writer.write_date_time_ticks(self.last_atrophy.ticks());
        writer.write_encoded_int(BROADCASTS_PER_PERIOD as i32);
        for t in &self.last_broadcasts {
            writer.write_date_time_ticks(t.ticks());
        }
        self.election.write(w, faction, writer);
        write_faction_ref(writer, Some(faction));
        writer.write_mobile(w, self.commander);
        writer.write_encoded_int(self.tithe);
        writer.write_encoded_int(self.silver);
        writer.write_encoded_int(self.members.len() as i32);
        for p in &self.members {
            p.write(w, writer);
        }
        writer.write_encoded_int(self.items.len() as i32);
        for fi in &self.items {
            writer.write_encoded_int(0);
            writer.write_item(w, fi.item);
            writer.write_date_time_ticks(fi.expiration.ticks());
        }
        writer.write_encoded_int(self.traps.len() as i32);
        for t in &self.traps {
            writer.write_item(w, Some(*t));
        }
    }
}

/// `TownState`.
#[derive(Clone, Debug, Default)]
pub struct TownData {
    pub owner: Option<usize>,
    pub sheriff: Option<MobileId>,
    pub finance: Option<MobileId>,
    pub silver: i32,
    pub tax: i32,
    pub last_tax_change: DateTime,
    pub last_income: DateTime,
}

impl TownData {
    /// `TownState(IGenericReader)`; the index of the town it names, and its data.
    pub fn read(w: &World, reader: &mut dyn GenericReader) -> Result<(usize, TownData)> {
        let version = reader.read_encoded_int()?;
        let mut d = TownData::default();
        if version >= 3 {
            d.last_income = date(reader)?;
        }
        if version >= 2 {
            d.tax = reader.read_encoded_int()?;
            d.last_tax_change = date(reader)?;
        }
        if version >= 1 {
            d.silver = reader.read_encoded_int()?;
        }
        let town = read_town_ref(reader)?
            .ok_or_else(|| IoError::InvalidData("TownState names no town".into()))?;
        d.owner = read_faction_ref(reader)?;
        d.sheriff = reader.read_mobile(w)?;
        d.finance = reader.read_mobile(w)?;
        Ok((town, d))
    }

    pub fn write(&self, w: &World, town: usize, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(3); // version
        writer.write_date_time_ticks(self.last_income.ticks());
        writer.write_encoded_int(self.tax);
        writer.write_date_time_ticks(self.last_tax_change.ticks());
        writer.write_encoded_int(self.silver);
        write_town_ref(writer, Some(town));
        write_faction_ref(writer, self.owner);
        writer.write_mobile(w, self.sheriff);
        writer.write_mobile(w, self.finance);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};
    use muo_server::mobile::MOBILE_BASE;

    fn mobiles(w: &mut World, n: usize) -> Vec<MobileId> {
        (0..n).map(|_| w.new_mobile(&MOBILE_BASE)).collect()
    }

    fn faction_with(ms: &[MobileId], points: &[i32]) -> FactionData {
        let mut d = FactionData::default();
        for &m in ms {
            d.insert_member(1, PlayerState::new(m, 1));
        }
        for (&m, &p) in ms.iter().zip(points) {
            d.set_kill_points(m, p);
        }
        d
    }

    #[test]
    fn pvp_kill_points_keep_members_ranked() {
        let mut w = World::new();
        let ms = mobiles(&mut w, 4);
        let mut d = faction_with(&ms, &[10, 30, 0, 20]);
        let order: Vec<i32> = d.members.iter().map(|p| p.kill_points).collect();
        assert_eq!(order, vec![30, 20, 10, 0]);
        assert_eq!(d.zero_rank_offset, 3);
        for (i, p) in d.members.iter().take(3).enumerate() {
            assert_eq!(p.rank_index, i as i32);
        }
        assert_eq!(d.members[3].rank_index, -1);

        // dropping the leader to zero sinks it below the ranked block
        d.set_kill_points(ms[1], 0);
        assert_eq!(d.zero_rank_offset, 2);
        assert_eq!(d.members[0].kill_points, 20);
        assert_eq!(d.members[1].kill_points, 10);
        // the top of two ranked players is the best rank, the other the lower one
        assert!(d.members[0].rank < d.members[1].rank);
    }

    #[test]
    fn pvp_remove_member_compacts_ranks() {
        let mut w = World::new();
        let ms = mobiles(&mut w, 3);
        let mut d = faction_with(&ms, &[30, 20, 10]);
        let gone = d.remove_member_state(ms[0]).unwrap();
        assert_eq!(gone.kill_points, 30);
        assert_eq!(d.zero_rank_offset, 2);
        assert_eq!(
            d.members.iter().map(|p| p.rank_index).collect::<Vec<_>>(),
            vec![0, 1]
        );
    }

    #[test]
    fn pvp_faction_state_roundtrip_and_layout() {
        let mut w = World::new();
        let ms = mobiles(&mut w, 2);
        let mut d = faction_with(&ms, &[7, 0]);
        d.silver = 123;
        d.commander = Some(ms[0]);
        let mut out = SaveWriter::new(true);
        d.write(&w, 1, &mut out);
        let bytes = out.as_slice().to_vec();
        assert_eq!(bytes[0], 5, "version");
        let mut r = SaveReader::new(&bytes);
        let (f, back) = FactionData::read(&w, &mut r).unwrap();
        assert_eq!(f, 1);
        assert_eq!(back.silver, 123);
        assert_eq!(back.commander, Some(ms[0]));
        assert_eq!(back.members.len(), 2);
        assert_eq!(back.members[0].kill_points, 7);
        assert_eq!(back.zero_rank_offset, 1);
        // writing what was read gives the same bytes
        let mut again = SaveWriter::new(true);
        back.write(&w, 1, &mut again);
        assert_eq!(again.as_slice(), &bytes[..]);
    }

    #[test]
    fn pvp_town_state_golden_bytes() {
        let w = World::new();
        let t = TownData {
            owner: Some(2),
            tax: 3,
            silver: 9,
            ..TownData::default()
        };
        let mut out = SaveWriter::new(true);
        t.write(&w, 4, &mut out);
        let b = out.as_slice();
        // version 3, LastIncome (8), Tax, LastTaxChange (8), Silver, Town ref + 1, Faction ref + 1, two null mobiles (-1)
        assert_eq!(b[0], 3);
        assert_eq!(&b[1..9], &DateTime::MIN.ticks().to_le_bytes());
        assert_eq!(b[9], 3);
        assert_eq!(b[18], 9);
        assert_eq!(&b[19..21], &[5, 3]);
        assert_eq!(&b[21..], &[0xFF; 8]);
        let mut r = SaveReader::new(b);
        let (town, back) = TownData::read(&w, &mut r).unwrap();
        assert_eq!(
            (town, back.owner, back.tax, back.silver),
            (4, Some(2), 3, 9)
        );
    }

    // Faction ranks: a CouncilOfMages member list, ranked without anyone reading `Rank` first.
    fn council() -> usize {
        assert_eq!(FACTION_DEFS[0].friendly_name, "Council of Mages");
        0
    }

    fn council_members(n: usize) -> FactionData {
        let mut w = World::new();
        let mut d = FactionData::default();
        for m in mobiles(&mut w, n) {
            d.members.push(PlayerState::new(m, council()));
        }
        d
    }

    #[test]
    fn rank_is_populated_before_anything_reads_it() {
        let d = council_members(1);
        assert_ne!(FACTION_DEFS[0].ranks[d.members[0].rank].title, 0);
    }

    #[test]
    fn rank_is_the_lowest_rank_for_an_unranked_member() {
        let mut d = council_members(2);
        d.update_rank(0, 0);
        d.update_rank(0, 1);
        let ranks = FACTION_DEFS[0].ranks;
        let lowest = ranks[ranks.len() - 1].rank;
        assert_eq!(ranks[d.members[0].rank].rank, lowest);
        assert_eq!(ranks[d.members[1].rank].rank, lowest);
    }

    #[test]
    fn setting_rank_index_updates_rank_without_anyone_reading_it() {
        let mut d = council_members(2);
        d.zero_rank_offset = 2;
        d.set_rank_index(0, 0);
        d.set_rank_index(1, 1);
        let ranks = FACTION_DEFS[0].ranks;
        assert!(ranks[d.members[0].rank].rank > ranks[d.members[1].rank].rank);
    }

    #[test]
    fn rank_index_out_of_sync_with_zero_rank_offset_still_resolves_a_rank() {
        let mut d = council_members(2);
        d.zero_rank_offset = 1;
        d.set_rank_index(0, 5);
        assert!(d.members[0].rank < FACTION_DEFS[0].ranks.len());
    }
}
