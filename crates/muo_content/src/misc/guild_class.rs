//! `AllianceInfo`, `WarDeclaration`, `GuildMaintenanceTimer` and the `Guild` class.
//!
//! A `Guild` is a `muo_server::guild::BaseGuild` in the world's guild registry with a [`GuildData`] in its `ext`.
//! `AllianceInfo` objects are shared by reference (a guild holds one, others point at its holder); here they
//! live in [`GuildState`] under an [`AllianceKey`] and a guild holds the key.
//!
//! The guild gumps are `muo_gumps::a_l::new_guilds` and `GuildGumpRequest` is `muo_engines::wiring2`'s.
//! `[GuildProps` and its target live in muo_items (`guilds/mod.rs`).

use indexmap::IndexMap;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::dotnet::InvariantCase;
use muo_primitives::utility;
use muo_primitives::{DateTime, Serial, TimeSpan};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::guild::{self, BaseGuild, GuildId, GuildType, GuildVt};
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::MobileDelta;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{damage, messages, properties};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;

use super::guild::{LEADER_INDEX, LOWEST_INDEX, MEMBER_INDEX, RANKS, RankFlags, new_guild_system};

pub const TYPE_NAME: &str = "Server.Guilds.Guild";
pub const REGISTRATION_FEE: i32 = 25000;
pub const ABBREV_LIMIT: usize = 4;
pub const NAME_LIMIT: usize = 40;
pub const MAJORITY_PERCENTAGE: i32 = 66;
const DEFAULT_HUE: i32 = 0x3B2;

/// `Guild.OrderChaos`.
pub fn order_chaos(w: &World) -> bool {
    !new_guild_system(w)
}

/// A reference to one `AllianceInfo`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AllianceKey(pub u32);

/// `AllianceInfo`.
#[derive(Clone, Debug, Default)]
pub struct AllianceInfo {
    pub name: String,
    /// `m_Leader`.
    pub leader: Option<GuildId>,
    pub members: Vec<GuildId>,
    pub pending_members: Vec<GuildId>,
}

/// `WarStatus`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum WarStatus {
    InProgress = -1,
    Win = 0,
    Lose = 1,
    Draw = 2,
    Pending = 3,
}

/// `WarDeclaration`.
#[derive(Clone, Debug)]
pub struct WarDeclaration {
    pub kills: i32,
    pub max_kills: i32,
    pub war_length: TimeSpan,
    pub opponent: Option<GuildId>,
    pub guild: Option<GuildId>,
    pub war_beginning: DateTime,
    pub war_requester: bool,
}

impl WarDeclaration {
    /// `new WarDeclaration(g, opponent, maxKills, warLength, warRequester)`.
    pub fn new(
        g: GuildId,
        opponent: GuildId,
        max_kills: i32,
        war_length: TimeSpan,
        war_requester: bool,
    ) -> Self {
        Self {
            kills: 0,
            max_kills,
            war_length,
            opponent: Some(opponent),
            guild: Some(g),
            war_beginning: DateTime::MIN,
            war_requester,
        }
    }

    fn deserialize(w: &World, r: &mut dyn GenericReader) -> Result<Self> {
        let mut d = Self {
            kills: 0,
            max_kills: 0,
            war_length: TimeSpan::ZERO,
            opponent: None,
            guild: None,
            war_beginning: DateTime::MIN,
            war_requester: false,
        };
        if r.read_i32()? == 0 {
            d.kills = r.read_i32()?;
            d.max_kills = r.read_i32()?;
            d.war_length = TimeSpan::from_ticks(r.read_time_span_ticks()?);
            d.war_beginning = DateTime::from_ticks(r.read_date_time_ticks()?);
            d.guild = r.read_guild(w)?;
            d.opponent = r.read_guild(w)?;
            d.war_requester = r.read_bool()?;
        }
        Ok(d)
    }

    fn serialize(&self, w: &World, out: &mut dyn GenericWriter) {
        out.write_i32(0);
        out.write_i32(self.kills);
        out.write_i32(self.max_kills);
        out.write_time_span_ticks(self.war_length.ticks());
        out.write_date_time_ticks(self.war_beginning.ticks());
        wg(w, out, self.guild);
        wg(w, out, self.opponent);
        out.write_bool(self.war_requester);
    }
}

/// The fields a `Guild` adds to `BaseGuild` (`Name`, `Abbreviation`, `Type` are the `BaseGuild` fields).
#[derive(Default)]
pub struct GuildData {
    /// `m_Leader`.
    pub leader: Option<MobileId>,
    pub alliance_info: Option<AllianceKey>,
    /// `m_AllianceLeader`.
    pub alliance_leader: Option<GuildId>,
    pub pending_wars: Vec<WarDeclaration>,
    pub accepted_wars: Vec<WarDeclaration>,
    pub guildstone: Option<ItemId>,
    pub teleporter: Option<ItemId>,
    pub website: Option<String>,
    pub charter: Option<String>,
    pub last_fealty: DateTime,
    pub type_last_change: DateTime,
    pub allies: Vec<GuildId>,
    pub enemies: Vec<GuildId>,
    pub ally_declarations: Vec<GuildId>,
    pub ally_invitations: Vec<GuildId>,
    pub war_declarations: Vec<GuildId>,
    pub war_invitations: Vec<GuildId>,
    pub candidates: Vec<MobileId>,
    pub accepted: Vec<MobileId>,
    pub members: Vec<MobileId>,
}

/// `AllianceInfo.Alliances` and the infos themselves.
#[derive(Default)]
pub struct GuildState {
    infos: IndexMap<u32, AllianceInfo>,
    next_key: u32,
    /// `AllianceInfo.Alliances`: lower-cased name to alliance.
    pub alliances: IndexMap<String, AllianceKey>,
}

fn state(w: &mut World) -> &mut GuildState {
    w.resources.get_mut::<GuildState>()
}

fn add_info(w: &mut World, info: AllianceInfo) -> AllianceKey {
    let s = state(w);
    s.next_key += 1;
    s.infos.insert(s.next_key, info);
    AllianceKey(s.next_key)
}

fn info(w: &World, k: AllianceKey) -> Option<&AllianceInfo> {
    w.resources.get::<GuildState>()?.infos.get(&k.0)
}

fn info_mut(w: &mut World, k: AllianceKey) -> Option<&mut AllianceInfo> {
    state(w).infos.get_mut(&k.0)
}

/// `AllianceInfo.Name`.
pub fn alliance_name(w: &World, k: AllianceKey) -> String {
    info(w, k).map_or_else(String::new, |i| i.name.clone())
}

/// `AllianceInfo.Alliances.ContainsKey(name.ToLower())`.
pub fn alliance_name_taken(w: &World, lower_name: &str) -> bool {
    w.resources
        .get::<GuildState>()
        .is_some_and(|s| s.alliances.contains_key(lower_name))
}

/// `Write(Guild)`: -1 for a disbanded guild (`Deleted`).
fn wg(w: &World, out: &mut dyn GenericWriter, g: Option<GuildId>) {
    out.write_guild(w, g.filter(|g| !disbanded(w, *g)));
}

/// The `GuildVt` of a `Guild`.
pub static GUILD_VT: GuildVt = GuildVt {
    delete,
    serialize,
    deserialize: deserialize_guild,
    on_delete: |w, g, m| remove_member(w, g, m, 1018028),
};

pub fn data(w: &World, g: GuildId) -> Option<&GuildData> {
    guild::guild(w, g)?.ext.get::<GuildData>()
}

pub fn data_mut(w: &mut World, g: GuildId) -> Option<&mut GuildData> {
    guild::guild_mut(w, g)?.ext.get_mut::<GuildData>()
}

/// An empty shell, as `new Guild(Serial)` (the loader fills it).
pub fn blank(serial: Serial, created: DateTime) -> BaseGuild {
    BaseGuild {
        serial,
        created,
        abbreviation: String::new(),
        name: String::new(),
        ty: GuildType::Regular,
        disbanded: false,
        vt: &GUILD_VT,
        ext: Ext::new(GuildData::default()),
    }
}

pub fn is_guild(w: &World, g: GuildId) -> bool {
    data(w, g).is_some()
}

/// `Guild.Disbanded`: `m_Leader?.Deleted != false`. A guild no longer in the registry counts as disbanded.
pub fn disbanded(w: &World, g: GuildId) -> bool {
    data(w, g).is_none_or(|d| d.leader.and_then(|l| w.mobile(l)).is_none_or(|m| m.deleted))
}

fn opt_disbanded(w: &World, g: Option<GuildId>) -> bool {
    g.is_none_or(|g| disbanded(w, g))
}

/// `Guild.Name`.
pub fn name(w: &World, g: GuildId) -> String {
    guild::guild(w, g)
        .map(|b| b.name.clone())
        .unwrap_or_default()
}

/// `Guild.Type`: `OrderChaos ? m_Type : GuildType.Regular`.
pub fn guild_type(w: &World, g: GuildId) -> GuildType {
    if order_chaos(w) {
        guild::guild(w, g).map_or(GuildType::Regular, |b| b.ty)
    } else {
        GuildType::Regular
    }
}

/// `Guild.Type = value`.
pub fn set_type(w: &mut World, g: GuildId, value: GuildType) {
    let Some(b) = guild::guild_mut(w, g) else {
        return;
    };
    if b.ty != value {
        b.ty = value;
        let now = w.now;
        if let Some(d) = data_mut(w, g) {
            d.type_last_change = now;
        }
        invalidate_member_properties(w, g, false);
    }
}

/// `Guild.Name = value`.
pub fn set_name(w: &mut World, g: GuildId, value: &str) {
    if let Some(b) = guild::guild_mut(w, g) {
        b.name = value.to_owned();
    }
    invalidate_member_properties(w, g, true);
    invalidate_guildstone(w, g);
}

/// `Guild.Abbreviation = value`.
pub fn set_abbreviation(w: &mut World, g: GuildId, value: &str) {
    if let Some(b) = guild::guild_mut(w, g) {
        b.abbreviation = value.to_owned();
    }
    invalidate_member_properties(w, g, true);
    invalidate_guildstone(w, g);
}

fn invalidate_guildstone(w: &mut World, g: GuildId) {
    if let Some(s) = data(w, g).and_then(|d| d.guildstone) {
        muo_server::item::ops::invalidate_properties(w, s);
    }
}

fn members(w: &World, g: GuildId) -> Vec<MobileId> {
    data(w, g).map(|d| d.members.clone()).unwrap_or_default()
}

fn set_rank(w: &mut World, m: MobileId, rank_index: i32) {
    if let Some(d) = crate::player_mobile::data_mut(w, m) {
        d.guild_rank = rank_index;
    }
}

pub fn mobile_guild(w: &World, m: MobileId) -> Option<GuildId> {
    w.mobile(m).and_then(|x| x.guild)
}

fn mobile_deleted(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_none_or(|x| x.deleted)
}

/// `new Guild(leader, name, abbreviation)` (including `BaseGuild()`: a new serial and `World.AddGuild`).
pub fn new_guild(w: &mut World, leader: MobileId, name: &str, abbreviation: &str) -> GuildId {
    let serial = guild::new_guild_serial(w);
    let mut b = blank(serial, w.now);
    b.name = name.to_owned();
    b.abbreviation = abbreviation.to_owned();
    if let Some(d) = b.ext.get_mut::<GuildData>() {
        d.leader = Some(leader);
        d.last_fealty = w.now;
        d.type_last_change = DateTime::MIN;
    }
    let g = guild::add_guild(w, b);
    add_member(w, g, leader);
    if crate::player_mobile::data(w, leader).is_some() {
        set_rank(w, leader, LEADER_INDEX);
    }
    g
}

// ---- Leader ----

/// `Guild.Leader` getter.
pub fn leader(w: &mut World, g: GuildId) -> Option<MobileId> {
    if disbanded(w, g) {
        calculate_guildmaster(w, g);
    }
    data(w, g)?.leader
}

/// `Guild.Leader = value`.
pub fn set_leader(w: &mut World, g: GuildId, value: Option<MobileId>) {
    if let Some(v) = value {
        add_member(w, g, v);
    }
    if let Some(old) = data(w, g).and_then(|d| d.leader)
        && crate::player_mobile::data(w, old).is_some()
        && mobile_guild(w, old) == Some(g)
    {
        set_rank(w, old, MEMBER_INDEX);
    }
    if let Some(d) = data_mut(w, g) {
        d.leader = value;
    }
    if let Some(l) = value
        && crate::player_mobile::data(w, l).is_some()
    {
        set_rank(w, l, LEADER_INDEX);
    }
}

// ---- Alliance ----

/// `Guild.Alliance` getter: `m_AllianceInfo ?? m_AllianceLeader?.m_AllianceInfo`.
pub fn alliance(w: &World, g: GuildId) -> Option<AllianceKey> {
    let d = data(w, g)?;
    d.alliance_info.or_else(|| {
        d.alliance_leader
            .and_then(|l| data(w, l))
            .and_then(|l| l.alliance_info)
    })
}

/// `Guild.Alliance = value`.
pub fn set_alliance(w: &mut World, g: GuildId, value: Option<AllianceKey>) {
    let current = alliance(w, g);
    if value == current {
        return;
    }
    if let Some(c) = current {
        alliance_remove_guild(w, c, g);
    }
    match value {
        Some(v) => {
            let l = alliance_leader(w, v);
            if l == Some(g) {
                if let Some(d) = data_mut(w, g) {
                    d.alliance_info = Some(v);
                }
            } else if let Some(d) = data_mut(w, g) {
                d.alliance_leader = l;
            }
            alliance_add_pending(w, v, g);
        }
        None => {
            if let Some(d) = data_mut(w, g) {
                d.alliance_info = None;
                d.alliance_leader = None;
            }
        }
    }
}

/// `new AllianceInfo(leader, name, partner)`.
pub fn create_alliance(
    w: &mut World,
    leader_g: GuildId,
    name: &str,
    partner: GuildId,
) -> AllianceKey {
    let k = add_info(
        w,
        AllianceInfo {
            name: name.to_owned(),
            leader: Some(leader_g),
            ..AllianceInfo::default()
        },
    );
    set_alliance(w, leader_g, Some(k));
    set_alliance(w, partner, Some(k));
    k
}

/// `AllianceInfo.IsPendingMember(g)`.
pub fn alliance_is_pending_member(w: &World, k: AllianceKey, g: GuildId) -> bool {
    alliance(w, g) == Some(k) && info(w, k).is_some_and(|i| i.pending_members.contains(&g))
}

/// `AllianceInfo.IsMember(g)`.
pub fn alliance_is_member(w: &World, k: AllianceKey, g: GuildId) -> bool {
    alliance(w, g) == Some(k) && info(w, k).is_some_and(|i| i.members.contains(&g))
}

/// `AllianceInfo.Leader` getter (runs `CheckLeader`).
pub fn alliance_leader(w: &mut World, k: AllianceKey) -> Option<GuildId> {
    alliance_check_leader(w, k);
    info(w, k)?.leader
}

/// `AllianceInfo.Leader = value`.
pub fn alliance_set_leader(w: &mut World, k: AllianceKey, value: Option<GuildId>) {
    let cur = info(w, k).and_then(|i| i.leader);
    if cur != value
        && let Some(v) = value
    {
        let n = name(w, v);
        alliance_message_args(w, k, 1070765, &n, DEFAULT_HUE); // Your Alliance is now led by ~1_GUILDNAME~
    }
    if let Some(i) = info_mut(w, k) {
        i.leader = value;
    }
    if value.is_none() {
        alliance_calculate_leader(w, k);
    }
}

/// `AllianceInfo.CalculateAllianceLeader`.
pub fn alliance_calculate_leader(w: &mut World, k: AllianceKey) {
    let members = info(w, k).map(|i| i.members.clone()).unwrap_or_default();
    let pick = if members.len() >= 2 {
        utility::random_element(&mut w.rng, &members).copied()
    } else {
        None
    };
    if let Some(i) = info_mut(w, k) {
        i.leader = pick;
    }
}

/// `AllianceInfo.CheckLeader`.
pub fn alliance_check_leader(w: &mut World, k: AllianceKey) {
    let l = info(w, k).and_then(|i| i.leader);
    if opt_disbanded(w, l) {
        alliance_calculate_leader(w, k);
        if info(w, k).is_none_or(|i| i.leader.is_none()) {
            alliance_disband(w, k);
        }
    }
}

fn alliance_add_pending(w: &mut World, k: AllianceKey, g: GuildId) {
    if alliance(w, g) != Some(k) {
        return;
    }
    let Some(i) = info_mut(w, k) else { return };
    if i.pending_members.contains(&g) || i.members.contains(&g) {
        return;
    }
    i.pending_members.push(g);
}

/// `AllianceInfo.TurnToMember(g)`.
pub fn alliance_turn_to_member(w: &mut World, k: AllianceKey, g: GuildId) {
    if alliance(w, g) != Some(k) {
        return;
    }
    let Some(i) = info_mut(w, k) else { return };
    if i.members.contains(&g) {
        return;
    }
    let Some(pos) = i.pending_members.iter().position(|x| *x == g) else {
        return;
    };
    i.pending_members.remove(pos);
    let n = i.name.clone();
    guild_message_args(w, g, 1070760, &n, DEFAULT_HUE); // Your Guild has joined the ~1_ALLIANCENAME~ Alliance.
    let gn = name(w, g);
    alliance_message_args(w, k, 1070761, &gn, DEFAULT_HUE); // A new Guild has joined your Alliance: ~1_GUILDNAME~
    if let Some(i) = info_mut(w, k) {
        i.members.push(g);
    }
    if let Some(a) = alliance(w, g) {
        alliance_invalidate_member_properties(w, a, false);
    }
}

/// `AllianceInfo.RemoveGuild(g)`.
pub fn alliance_remove_guild(w: &mut World, k: AllianceKey, g: GuildId) {
    let Some(i) = info_mut(w, k) else { return };
    i.pending_members.retain(|x| *x != g);
    let was_member = i
        .members
        .iter()
        .position(|x| *x == g)
        .map(|p| i.members.remove(p))
        .is_some();
    let aname = i.name.clone();
    if was_member {
        invalidate_member_properties(w, g, false);
        guild_message_args(w, g, 1070763, &aname, DEFAULT_HUE); // Your Guild has been removed from the ~1_ALLIANCENAME~ Alliance.
        let gn = name(w, g);
        alliance_message_args(w, k, 1070764, &gn, DEFAULT_HUE); // A Guild has left your Alliance: ~1_GUILDNAME~
    }
    // No `g.Alliance = null` here: the caller does it.
    if info(w, k).and_then(|i| i.leader) == Some(g) {
        alliance_calculate_leader(w, k);
    }
    if info(w, k).is_none_or(|i| i.members.len() < 2) {
        alliance_disband(w, k);
    }
}

/// `AllianceInfo.Disband()`.
pub fn alliance_disband(w: &mut World, k: AllianceKey) {
    alliance_message(w, k, 1070762); // Your Alliance has dissolved.
    let mut i = 0;
    while let Some(g) = info(w, k).and_then(|a| a.pending_members.get(i).copied()) {
        set_alliance(w, g, None);
        i += 1;
    }
    let mut i = 0;
    while let Some(g) = info(w, k).and_then(|a| a.members.get(i).copied()) {
        set_alliance(w, g, None);
        i += 1;
    }
    let lower = info(w, k).map(|a| a.name.lower_inv()).unwrap_or_default();
    let s = state(w);
    if s.alliances.get(&lower) == Some(&k) {
        s.alliances.shift_remove(&lower);
    }
}

pub fn alliance_members(w: &World, k: AllianceKey) -> Vec<GuildId> {
    info(w, k).map(|i| i.members.clone()).unwrap_or_default()
}

pub fn alliance_invalidate_member_properties(w: &mut World, k: AllianceKey, only_opl: bool) {
    for g in alliance_members(w, k) {
        invalidate_member_properties(w, g, only_opl);
    }
}

pub fn alliance_invalidate_member_notoriety(w: &mut World, k: AllianceKey) {
    for g in alliance_members(w, k) {
        invalidate_member_notoriety(w, g);
    }
}

/// `AllianceInfo.AllianceMessage(number)`.
pub fn alliance_message(w: &mut World, k: AllianceKey, number: i32) {
    for g in alliance_members(w, k) {
        guild_message(w, g, number);
    }
}

/// `AllianceInfo.AllianceMessage(number, args, hue)`.
pub fn alliance_message_args(w: &mut World, k: AllianceKey, number: i32, args: &str, hue: i32) {
    for g in alliance_members(w, k) {
        guild_message_args(w, g, number, args, hue);
    }
}

/// `AllianceInfo.AllianceMessage(number, append, affix, args, hue)`.
pub fn alliance_message_affix(
    w: &mut World,
    k: AllianceKey,
    number: i32,
    append: bool,
    affix: &str,
    args: &str,
    hue: i32,
) {
    for g in alliance_members(w, k) {
        guild_message_affix(w, g, number, append, affix, args, hue);
    }
}

/// `AllianceInfo.AllianceTextMessage(hue, text)`.
pub fn alliance_text_message(w: &mut World, k: AllianceKey, hue: i32, text: &str) {
    for g in alliance_members(w, k) {
        guild_text_message(w, g, hue, text);
    }
}

/// `AllianceInfo.AllianceChat(from, hue, text)`.
pub fn alliance_chat(w: &mut World, k: AllianceKey, from: MobileId, hue: i32, text: &str) {
    for g in alliance_members(w, k) {
        for m in members(w, g) {
            send_chat(
                w,
                m,
                from,
                muo_server::packets::message::MessageType::ALLIANCE,
                hue,
                text,
            );
        }
    }
}

/// `AllianceInfo.AllianceChat(from, text)`.
pub fn alliance_chat_default(w: &mut World, k: AllianceKey, from: MobileId, text: &str) {
    let hue = crate::player_mobile::data(w, from).map_or(DEFAULT_HUE, |d| d.alliance_message_hue);
    alliance_chat(w, k, from, hue, text);
}

fn send_chat(
    w: &mut World,
    to: MobileId,
    from: MobileId,
    kind: muo_server::packets::message::MessageType,
    hue: i32,
    text: &str,
) {
    let Some(ns) = w.mobile(to).and_then(|m| m.net_state) else {
        return;
    };
    let Some(f) = w.mobile(from) else { return };
    let (serial, body, lang, nm) = (
        f.serial,
        muo_server::mobile::part1::props_b::body(w, from).0,
        f.language.clone(),
        (f.ty.name)(w, from).unwrap_or_default(),
    );
    muo_server::packets::message::send_message(
        w,
        ns,
        serial,
        body,
        kind,
        hue,
        3,
        false,
        lang.as_deref(),
        &nm,
        text,
    );
}

// ---- notoriety / messages ----

/// `Guild.InvalidateMemberProperties`.
pub fn invalidate_member_properties(w: &mut World, g: GuildId, only_opl: bool) {
    for m in members(w, g) {
        properties::invalidate_properties(w, m);
        if !only_opl {
            muo_server::mobile::base::delta(w, m, MobileDelta::NOTO);
        }
    }
}

/// `Guild.InvalidateMemberNotoriety`.
pub fn invalidate_member_notoriety(w: &mut World, g: GuildId) {
    for m in members(w, g) {
        muo_server::mobile::base::delta(w, m, MobileDelta::NOTO);
    }
}

/// `Guild.InvalidateWarNotoriety`.
pub fn invalidate_war_notoriety(w: &mut World, g: GuildId) {
    let lead = get_alliance_leader(w, g);
    match alliance(w, lead) {
        Some(a) => alliance_invalidate_member_notoriety(w, a),
        None => invalidate_member_notoriety(w, lead),
    }
    let opponents: Vec<Option<GuildId>> = data(w, lead)
        .map(|d| d.accepted_wars.iter().map(|x| x.opponent).collect())
        .unwrap_or_default();
    for o in opponents.into_iter().flatten() {
        match alliance(w, o) {
            Some(a) => alliance_invalidate_member_notoriety(w, a),
            None => invalidate_member_notoriety(w, o),
        }
    }
}

/// `Guild.GuildMessage(number)`.
pub fn guild_message(w: &mut World, g: GuildId, number: i32) {
    for m in members(w, g) {
        messages::send_localized_message(w, m, number, "", DEFAULT_HUE);
    }
}

/// `Guild.GuildMessage(number, args, hue)`.
pub fn guild_message_args(w: &mut World, g: GuildId, number: i32, args: &str, hue: i32) {
    for m in members(w, g) {
        messages::send_localized_message(w, m, number, args, hue);
    }
}

/// `Guild.GuildMessage(number, append, affix, args, hue)`.
pub fn guild_message_affix(
    w: &mut World,
    g: GuildId,
    number: i32,
    append: bool,
    affix: &str,
    args: &str,
    hue: i32,
) {
    for m in members(w, g) {
        messages::send_localized_message_affix(w, m, number, append, affix, args, hue);
    }
}

/// `Guild.GuildTextMessage(hue, text)`.
pub fn guild_text_message(w: &mut World, g: GuildId, hue: i32, text: &str) {
    for m in members(w, g) {
        messages::send_message_hue(w, m, hue, text);
    }
}

/// `Guild.GuildChat(from, hue, text)`.
pub fn guild_chat(w: &mut World, g: GuildId, from: MobileId, hue: i32, text: &str) {
    for m in members(w, g) {
        send_chat(
            w,
            m,
            from,
            muo_server::packets::message::MessageType::GUILD,
            hue,
            text,
        );
    }
}

/// `Guild.GuildChat(from, text)`.
pub fn guild_chat_default(w: &mut World, g: GuildId, from: MobileId, text: &str) {
    let hue = crate::player_mobile::data(w, from).map_or(DEFAULT_HUE, |d| d.guild_message_hue);
    guild_chat(w, g, from, hue, text);
}

// ---- wars ----

/// `Guild.GetAllianceLeader(g)`.
pub fn get_alliance_leader(w: &mut World, g: GuildId) -> GuildId {
    if let Some(a) = alliance(w, g)
        && let Some(l) = alliance_leader(w, a)
        && alliance_is_member(w, a, g)
    {
        return l;
    }
    g
}

/// `Guild.FindPendingWar(g)`: the index in `PendingWars`.
pub fn find_pending_war(w: &World, this: GuildId, g: Option<GuildId>) -> Option<usize> {
    data(w, this)?
        .pending_wars
        .iter()
        .position(|x| x.opponent == g)
}

/// `Guild.FindActiveWar(g)`: the index in `AcceptedWars`.
pub fn find_active_war(w: &World, this: GuildId, g: Option<GuildId>) -> Option<usize> {
    data(w, this)?
        .accepted_wars
        .iter()
        .position(|x| x.opponent == g)
}

/// `WarDeclaration.Status` of `war`.
pub fn war_status(w: &World, war: &WarDeclaration) -> WarStatus {
    if opt_disbanded(w, war.opponent) {
        return WarStatus::Win;
    }
    if opt_disbanded(w, war.guild) {
        return WarStatus::Lose;
    }
    let (Some(opp), Some(gd)) = (war.opponent, war.guild) else {
        return WarStatus::Win;
    };
    let active = find_active_war(w, opp, Some(gd))
        .and_then(|i| data(w, opp).map(|d| d.accepted_wars[i].clone()));
    if find_pending_war(w, opp, Some(gd)).is_some() && find_pending_war(w, gd, Some(opp)).is_some()
    {
        return WarStatus::Pending;
    }
    let Some(other) = active else {
        return WarStatus::Win;
    };
    if war.war_length != TimeSpan::ZERO && war.war_beginning + war.war_length < w.now {
        if war.kills > other.kills {
            return WarStatus::Win;
        }
        return if war.kills < other.kills {
            WarStatus::Lose
        } else {
            WarStatus::Draw
        };
    }
    if war.max_kills > 0 {
        if war.kills >= war.max_kills {
            return WarStatus::Win;
        }
        if other.kills >= other.max_kills {
            return WarStatus::Lose;
        }
    }
    WarStatus::InProgress
}

fn status_from(n: i32) -> i32 {
    n
}

/// `Guild.CheckExpiredWars`.
pub fn check_expired_wars(w: &mut World, g: GuildId) {
    let mut i = 0;
    while let Some(war) = data(w, g).and_then(|d| d.accepted_wars.get(i).cloned()) {
        let opp = war.opponent;
        let mut status = war_status(w, &war);
        if status != WarStatus::InProgress {
            let my_alliance = alliance(w, g);
            let in_alliance = my_alliance.is_some_and(|a| alliance_is_member(w, a, g));
            let other_alliance = opp.and_then(|o| alliance(w, o));
            // This asks `otherAlliance?.IsMember(this)`, not the opponent: kept for identical war-state results.
            let other_in_alliance = other_alliance.is_some_and(|a| alliance_is_member(w, a, g));

            let text = |w: &World| match opp {
                None => "a deleted opponent".to_owned(),
                Some(_) if other_in_alliance => other_alliance
                    .and_then(|a| info(w, a))
                    .map(|a| a.name.clone())
                    .unwrap_or_default(),
                Some(o) => name(w, o),
            };
            let n = 1070739 + status as i32;
            let t = text(w);
            if in_alliance {
                let a = my_alliance.expect("in_alliance");
                alliance_message_args(w, a, n, &t, DEFAULT_HUE);
                alliance_invalidate_member_properties(w, a, false);
            } else {
                guild_message_args(w, g, n, &t, DEFAULT_HUE);
                invalidate_member_properties(w, g, false);
            }
            if let Some(d) = data_mut(w, g) {
                d.accepted_wars.remove(i);
            }
            let Some(o) = opp else {
                i += 1;
                continue;
            };
            if status != WarStatus::Draw {
                // `(WarStatus)((int)status + 1 % 2)`: `1 % 2` binds first, so this is status + 1.
                status = match status_from(status as i32 + 1) {
                    0 => WarStatus::Win,
                    1 => WarStatus::Lose,
                    2 => WarStatus::Draw,
                    _ => WarStatus::Pending,
                };
            }
            let n2 = 1070739 + status as i32;
            let mine = if in_alliance {
                my_alliance
                    .and_then(|a| info(w, a))
                    .map(|a| a.name.clone())
                    .unwrap_or_default()
            } else {
                name(w, g)
            };
            if other_in_alliance {
                let a = other_alliance.expect("other_in_alliance");
                alliance_message_args(w, a, n2, &mine, DEFAULT_HUE);
                alliance_invalidate_member_properties(w, a, false);
            } else {
                guild_message_args(w, o, n2, &mine, DEFAULT_HUE);
                invalidate_member_properties(w, o, false);
            }
            if let Some(idx) = find_active_war(w, o, Some(g))
                && let Some(d) = data_mut(w, o)
            {
                d.accepted_wars.remove(idx);
            }
        }
        i += 1;
    }

    let mut i = 0;
    while let Some(war) = data(w, g).and_then(|d| d.pending_wars.get(i).cloned()) {
        if war_status(w, &war) != WarStatus::Pending {
            if let Some(d) = data_mut(w, g) {
                d.pending_wars.remove(i);
            }
            if let Some(o) = war.opponent
                && let Some(idx) = find_pending_war(w, o, Some(g))
                && let Some(d) = data_mut(w, o)
            {
                d.pending_wars.remove(idx);
            }
        }
        i += 1;
    }
}

/// `Guild.HandleDeath(victim, killer)`.
pub fn handle_death(w: &mut World, victim: MobileId, killer: Option<MobileId>) {
    if !new_guild_system(w) {
        return;
    }
    let killer = killer.or_else(|| damage::find_most_recent_damager(w, victim, false));
    let (Some(killer), Some(vg)) = (killer, mobile_guild(w, victim)) else {
        return;
    };
    let Some(kg) = mobile_guild(w, killer) else {
        return;
    };
    let victim_guild = get_alliance_leader(w, vg);
    let killer_guild = get_alliance_leader(w, kg);
    let Some(idx) = find_active_war(w, killer_guild, Some(victim_guild)) else {
        return;
    };
    let opponent = {
        let Some(d) = data_mut(w, killer_guild) else {
            return;
        };
        d.accepted_wars[idx].kills += 1;
        d.accepted_wars[idx].opponent
    };
    if opponent == Some(victim_guild) {
        check_expired_wars(w, killer_guild);
    } else {
        check_expired_wars(w, victim_guild);
    }
}

// ---- relations ----

/// `Guild.IsMember(m)`.
pub fn is_member(w: &World, g: GuildId, m: MobileId) -> bool {
    data(w, g).is_some_and(|d| d.members.contains(&m))
}

/// `Guild.IsAlly(g)`.
pub fn is_ally(w: &World, this: GuildId, g: GuildId) -> bool {
    if new_guild_system(w) {
        alliance(w, this)
            .is_some_and(|a| alliance_is_member(w, a, this) && alliance_is_member(w, a, g))
    } else {
        data(w, this).is_some_and(|d| d.allies.contains(&g))
    }
}

/// `Guild.IsEnemy(g)`.
pub fn is_enemy(w: &mut World, this: GuildId, g: GuildId) -> bool {
    let (a, b) = (guild_type(w, this), guild_type(w, g));
    (a != GuildType::Regular && b != GuildType::Regular && a != b) || is_war(w, this, Some(g))
}

/// `Guild.IsWar(g)`.
pub fn is_war(w: &mut World, this: GuildId, g: Option<GuildId>) -> bool {
    let Some(g) = g else { return false };
    if !new_guild_system(w) {
        return data(w, this).is_some_and(|d| d.enemies.contains(&g));
    }
    let a = get_alliance_leader(w, this);
    let b = get_alliance_leader(w, g);
    find_active_war(w, a, Some(b)).is_some()
}

/// The `&World` form of [`is_enemy`] for the notoriety hook: the alliance leader is only recomputed when it is
/// already valid (a disbanded alliance leader would be replaced, which needs `&mut`).
pub fn relation_hook(w: &World, a: GuildId, b: GuildId) -> crate::misc::notoriety::GuildRelation {
    use crate::misc::notoriety::GuildRelation;
    if is_ally(w, a, b) {
        return GuildRelation::Ally;
    }
    let (ta, tb) = (guild_type(w, a), guild_type(w, b));
    let war = if new_guild_system(w) {
        let la = alliance_leader_ro(w, a);
        let lb = alliance_leader_ro(w, b);
        find_active_war(w, la, Some(lb)).is_some()
    } else {
        data(w, a).is_some_and(|d| d.enemies.contains(&b))
    };
    if (ta != GuildType::Regular && tb != GuildType::Regular && ta != tb) || war {
        GuildRelation::Enemy
    } else {
        GuildRelation::None
    }
}

fn alliance_leader_ro(w: &World, g: GuildId) -> GuildId {
    if let Some(a) = alliance(w, g)
        && let Some(l) = info(w, a).and_then(|i| i.leader)
        && !disbanded(w, l)
        && alliance_is_member(w, a, g)
    {
        return l;
    }
    g
}

/// `Guild.Tidy(list)`: drops disbanded guilds.
pub fn tidy(w: &World, list: &[GuildId]) -> Vec<GuildId> {
    list.iter().copied().filter(|g| !disbanded(w, *g)).collect()
}

fn tidy_mobiles(w: &World, list: &[MobileId]) -> Vec<MobileId> {
    list.iter()
        .copied()
        .filter(|m| !mobile_deleted(w, *m))
        .collect()
}

// ---- members ----

/// `Guild.AddMember(m)`.
pub fn add_member(w: &mut World, g: GuildId, m: MobileId) {
    if is_member(w, g, m) {
        return;
    }
    let old = mobile_guild(w, m).filter(|o| is_guild(w, *o));
    if let Some(o) = old
        && o != g
    {
        remove_member(w, o, m, 1018028);
    }
    if let Some(d) = data_mut(w, g) {
        d.members.push(m);
    }
    props_b::set_guild(w, m, Some(g));
    let fealty = if !new_guild_system(w) {
        data(w, g).and_then(|d| d.leader)
    } else {
        None
    };
    if let Some(x) = w.mobile_mut(m) {
        x.guild_fealty = fealty;
    }
    if crate::player_mobile::data(w, m).is_some() {
        set_rank(w, m, LOWEST_INDEX);
    }
    if let Some(o) = old {
        invalidate_war_notoriety(w, o);
    }
    invalidate_war_notoriety(w, g);
}

/// `Guild.RemoveMember(m, message)`; `message` 0 sends none (the default is 1018028).
pub fn remove_member(w: &mut World, g: GuildId, m: MobileId, message: i32) {
    let Some(d) = data_mut(w, g) else { return };
    let Some(pos) = d.members.iter().position(|x| *x == m) else {
        return;
    };
    d.members.remove(pos);
    let old = mobile_guild(w, m).filter(|o| is_guild(w, *o));
    props_b::set_guild(w, m, None);
    if crate::player_mobile::data(w, m).is_some() {
        set_rank(w, m, LOWEST_INDEX);
    }
    if message > 0 {
        messages::send_localized_message(w, m, message, "", DEFAULT_HUE);
    }
    if data(w, g).and_then(|d| d.leader) == Some(m) {
        calculate_guildmaster(w, g);
        if data(w, g).is_none_or(|d| d.leader.is_none()) {
            disband(w, g);
        }
    }
    if data(w, g).is_none_or(|d| d.members.is_empty()) {
        disband(w, g);
    }
    if let Some(o) = old {
        invalidate_war_notoriety(w, o);
    }
    muo_server::mobile::base::delta(w, m, MobileDelta::NOTO);
}

/// `Guild.AddAlly(g)`.
pub fn add_ally(w: &mut World, this: GuildId, g: GuildId) {
    if data(w, this).is_some_and(|d| !d.allies.contains(&g)) {
        if let Some(d) = data_mut(w, this) {
            d.allies.push(g);
        }
        add_ally(w, g, this);
    }
}

/// `Guild.RemoveAlly(g)`.
pub fn remove_ally(w: &mut World, this: GuildId, g: GuildId) {
    if let Some(d) = data_mut(w, this)
        && let Some(p) = d.allies.iter().position(|x| *x == g)
    {
        d.allies.remove(p);
        remove_ally(w, g, this);
    }
}

/// `Guild.AddEnemy(g)`.
pub fn add_enemy(w: &mut World, this: GuildId, g: GuildId) {
    if data(w, this).is_some_and(|d| !d.enemies.contains(&g)) {
        if let Some(d) = data_mut(w, this) {
            d.enemies.push(g);
        }
        add_enemy(w, g, this);
    }
}

/// `Guild.RemoveEnemy(g)`.
pub fn remove_enemy(w: &mut World, this: GuildId, g: GuildId) {
    if let Some(d) = data_mut(w, this)
        && let Some(p) = d.enemies.iter().position(|x| *x == g)
    {
        d.enemies.remove(p);
        remove_enemy(w, g, this);
    }
}

/// `Guild.Disband()`.
pub fn disband(w: &mut World, g: GuildId) {
    if let Some(d) = data_mut(w, g) {
        d.leader = None;
    }
    // `World.RemoveGuild(this)` belongs here; nothing below reads the registry for `this`, so it moves to the end.
    for m in members(w, g) {
        messages::send_localized_message(w, m, 502131, "", DEFAULT_HUE); // Your guild has disbanded.
        if crate::player_mobile::data(w, m).is_some() {
            set_rank(w, m, LOWEST_INDEX);
        }
        props_b::set_guild(w, m, None);
    }
    if let Some(d) = data_mut(w, g) {
        d.members.clear();
    }
    let mut i = data(w, g).map_or(0, |d| d.allies.len());
    while i > 0 {
        i -= 1;
        if let Some(a) = data(w, g).and_then(|d| d.allies.get(i).copied()) {
            remove_ally(w, g, a);
        }
    }
    let mut i = data(w, g).map_or(0, |d| d.enemies.len());
    while i > 0 {
        i -= 1;
        if let Some(e) = data(w, g).and_then(|d| d.enemies.get(i).copied()) {
            remove_enemy(w, g, e);
        }
    }
    if !new_guild_system(w)
        && let Some(s) = data(w, g).and_then(|d| d.guildstone)
    {
        muo_server::item::base::delete(w, s);
    }
    if let Some(d) = data_mut(w, g) {
        d.guildstone = None;
    }
    check_expired_wars(w, g);
    set_alliance(w, g, None);
    guild::remove_guild(w, g);
}

/// `Guild.CanVote(m)`.
pub fn can_vote(w: &World, g: GuildId, m: Option<MobileId>) -> bool {
    let Some(m) = m else { return false };
    let rank_ok = !new_guild_system(w)
        || crate::player_mobile::data(w, m).is_some_and(|d| {
            RANKS
                .get(d.guild_rank as usize)
                .is_some_and(|r| r.get_flag(RankFlags::VOTE))
        });
    rank_ok && !mobile_deleted(w, m) && mobile_guild(w, m) == Some(g)
}

/// `Guild.CanBeVotedFor(m)`.
pub fn can_be_voted_for(w: &World, g: GuildId, m: Option<MobileId>) -> bool {
    let Some(m) = m else { return false };
    let active = !new_guild_system(w)
        || crate::player_mobile::data(w, m)
            .is_some_and(|d| d.last_online + super::guild::INACTIVE_TIME >= w.now);
    active && !mobile_deleted(w, m) && mobile_guild(w, m) == Some(g)
}

/// `Guild.CalculateGuildmaster()`.
pub fn calculate_guildmaster(w: &mut World, g: GuildId) {
    let mut votes: IndexMap<MobileId, i32> = IndexMap::new();
    let is_disbanded = disbanded(w, g);
    let cur_leader = data(w, g).and_then(|d| d.leader);
    let has_leader = !is_disbanded && cur_leader.is_some_and(|l| mobile_guild(w, l) == Some(g));
    let mut voting_members = 0;

    for memb in members(w, g) {
        if !can_vote(w, g, Some(memb)) {
            continue;
        }
        let mut m = w.mobile(memb).and_then(|x| x.guild_fealty);
        if !can_be_voted_for(w, g, m) {
            m = if has_leader { cur_leader } else { Some(memb) };
        }
        let Some(m) = m else { continue };
        *votes.entry(m).or_insert(0) += 1;
        voting_members += 1;
    }

    let mut winner: Option<MobileId> = None;
    let mut high = 0;
    for (m, val) in &votes {
        if winner.is_none() || *val > high {
            winner = Some(*m);
            high = *val;
        }
    }

    if has_leader
        && (winner.is_none()
            || (new_guild_system(w)
                && high * 100 / std::cmp::max(voting_members, 1) < MAJORITY_PERCENTAGE))
    {
        winner = cur_leader;
    }

    if winner.is_none() {
        if !votes.is_empty() {
            let n = utility::random(&mut w.rng, votes.len() as i32);
            winner = votes.keys().nth(n as usize).copied();
        } else {
            let ms = members(w, g);
            winner = utility::random_element(&mut w.rng, &ms).copied();
        }
    }

    if let Some(win) = winner
        && cur_leader != Some(win)
    {
        set_leader(w, g, Some(win));
        let raw = props_b::raw_name(w, win).unwrap_or_default();
        guild_message_affix(w, g, 1018015, true, &raw, "", DEFAULT_HUE); // Guild Message: Guildmaster changed to:
    }

    let now = w.now;
    if let Some(d) = data_mut(w, g) {
        d.last_fealty = now;
    }
}

/// `Guild.RunMaintenance()`.
pub fn run_maintenance(w: &mut World, g: GuildId) {
    if disbanded(w, g) {
        return;
    }
    let last = data(w, g).map_or(DateTime::MIN, |d| d.last_fealty);
    if last + TimeSpan::from_days(1) < w.now {
        calculate_guildmaster(w, g);
    }
    check_expired_wars(w, g);
    if let Some(a) = alliance(w, g) {
        alliance_check_leader(w, a);
    }
}

/// `GuildMaintenanceTimer.Initialize`: every minute, every guild.
pub fn maintenance_initialize(w: &mut World) {
    timer::start(w, TimeSpan::from_mins(1), TimeSpan::from_mins(1), 0, |w| {
        for g in guild::guild_ids(w) {
            if is_guild(w, g) {
                run_maintenance(w, g);
            }
        }
    });
}

fn verify_guild(w: &mut World, g: GuildId) {
    let no_stone = data(w, g).is_some_and(|d| d.guildstone.is_none());
    let empty = data(w, g).is_none_or(|d| d.members.is_empty());
    if (!new_guild_system(w) && no_stone) || empty {
        disband(w, g);
    }
    check_expired_wars(w, g);
    if let Some(a) = alliance(w, g) {
        alliance_check_leader(w, a);
    }
    let a = alliance(w, g);
    if let Some(a) = a
        && !alliance_is_member(w, a, g)
        && !alliance_is_pending_member(w, a, g)
    {
        set_alliance(w, g, None); // A bug fix for an older version.
    }
}

/// `Timer.StartTimer(VerifyGuild_Callback)` of every guild just loaded.
pub fn schedule_verify(w: &mut World) {
    for g in guild::guild_ids(w) {
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            verify_guild(w, g)
        });
    }
}

/// `Guild.Delete()`.
pub fn delete(w: &mut World, g: GuildId) {
    guild::remove_guild(w, g);
}

// ---- save bytes ----

impl AllianceInfo {
    fn serialize(&self, w: &World, out: &mut dyn GenericWriter) {
        out.write_i32(0);
        out.write_string(&self.name);
        wg(w, out, self.leader);
        out.write_guild_list(w, &tidy(w, &self.members));
        out.write_guild_list(w, &tidy(w, &self.pending_members));
    }
}

/// `new AllianceInfo(reader)`; the `Alliances` entry is added a tick later.
fn read_alliance_info(w: &mut World, r: &mut dyn GenericReader) -> Result<AllianceKey> {
    let mut i = AllianceInfo::default();
    if r.read_i32()? == 0 {
        i.name = r.read_string()?.unwrap_or_default();
        i.leader = r.read_guild(w)?;
        i.members = r.read_guild_list(w)?;
        i.pending_members = r.read_guild_list(w)?;
    }
    let lower = i.name.lower_inv();
    let k = add_info(w, i);
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        state(w).alliances.entry(lower.clone()).or_insert(k);
    });
    Ok(k)
}

/// `Guild.Serialize`.
pub fn serialize(w: &World, g: GuildId, out: &mut dyn GenericWriter) {
    let (Some(b), Some(d)) = (guild::guild(w, g), data(w, g)) else {
        return;
    };
    out.write_i32(5);
    out.write_i32(d.pending_wars.len() as i32);
    for x in &d.pending_wars {
        x.serialize(w, out);
    }
    out.write_i32(d.accepted_wars.len() as i32);
    for x in &d.accepted_wars {
        x.serialize(w, out);
    }
    let is_alliance_leader = d.alliance_leader.is_none() && d.alliance_info.is_some();
    out.write_bool(is_alliance_leader);
    if is_alliance_leader {
        if let Some(i) = d.alliance_info.and_then(|k| info(w, k)) {
            i.serialize(w, out);
        }
    } else {
        wg(w, out, d.alliance_leader);
    }

    out.write_guild_list(w, &tidy(w, &d.ally_declarations));
    out.write_guild_list(w, &tidy(w, &d.ally_invitations));
    out.write_date_time_ticks(d.type_last_change.ticks());
    out.write_i32(b.ty as i32);
    out.write_date_time_ticks(d.last_fealty.ticks());
    out.write_mobile(w, d.leader);
    out.write_string(&b.name);
    out.write_string(&b.abbreviation);
    out.write_guild_list(w, &tidy(w, &d.allies));
    out.write_guild_list(w, &tidy(w, &d.enemies));
    out.write_guild_list(w, &tidy(w, &d.war_declarations));
    out.write_guild_list(w, &tidy(w, &d.war_invitations));
    out.write_mobile_list(w, &tidy_mobiles(w, &d.members));
    out.write_mobile_list(w, &tidy_mobiles(w, &d.candidates));
    out.write_mobile_list(w, &tidy_mobiles(w, &d.accepted));
    out.write_item(w, d.guildstone);
    out.write_item(w, d.teleporter);
    out.write_opt_string(d.charter.as_deref());
    out.write_opt_string(d.website.as_deref());
}

/// `Guild.Deserialize`: every version (5 down to 0, each case falling into the next).
pub fn deserialize_guild(w: &mut World, g: GuildId, r: &mut dyn GenericReader) -> Result<()> {
    let version = r.read_i32()?;
    if !(0..=5).contains(&version) {
        return Ok(()); // no case matches: nothing is read
    }
    let mut d = GuildData::default();
    let mut ty = GuildType::Regular;
    if version >= 5 {
        let n = r.read_i32()?;
        for _ in 0..n {
            d.pending_wars.push(WarDeclaration::deserialize(w, r)?);
        }
        let n = r.read_i32()?;
        for _ in 0..n {
            d.accepted_wars.push(WarDeclaration::deserialize(w, r)?);
        }
        if r.read_bool()? {
            d.alliance_info = Some(read_alliance_info(w, r)?);
        } else {
            d.alliance_leader = r.read_guild(w)?;
        }
    }
    if version >= 4 {
        d.ally_declarations = r.read_guild_list(w)?;
        d.ally_invitations = r.read_guild_list(w)?;
    }
    if version >= 3 {
        d.type_last_change = DateTime::from_ticks(r.read_date_time_ticks()?);
    }
    if version >= 2 {
        ty = match r.read_i32()? {
            1 => GuildType::Chaos,
            2 => GuildType::Order,
            _ => GuildType::Regular,
        };
    }
    if version >= 1 {
        d.last_fealty = DateTime::from_ticks(r.read_date_time_ticks()?);
    }
    let (gname, abbr);
    {
        d.leader = r.read_mobile(w)?;
        gname = r.read_string()?.unwrap_or_default();
        abbr = r.read_string()?.unwrap_or_default();
        d.allies = r.read_guild_list(w)?;
        d.enemies = r.read_guild_list(w)?;
        d.war_declarations = r.read_guild_list(w)?;
        d.war_invitations = r.read_guild_list(w)?;
        d.members = r.read_mobile_list(w)?;
        d.candidates = r.read_mobile_list(w)?;
        d.accepted = r.read_mobile_list(w)?;
        d.guildstone = r.read_item(w)?;
        d.teleporter = r.read_item(w)?;
        d.charter = r.read_string()?;
        d.website = r.read_string()?;
    }
    if let Some(b) = guild::guild_mut(w, g) {
        b.name = gname;
        b.abbreviation = abbr;
        b.ty = ty;
        b.ext = Ext::new(d);
    }
    Ok(())
}

/// `Guild.Configure` (the `[GuildProps` command is partial) plus the hooks other systems call guilds through.
pub fn configure(w: &mut World) {
    use crate::misc::notoriety::NotorietyHooks;
    use crate::player_mobile::hooks::SystemHooks;
    let h = w.resources.get_mut::<SystemHooks>();
    h.guild_handle_death = Some(handle_death);
    h.guild_in_alliance = Some(|w, m| {
        mobile_guild(w, m)
            .is_some_and(|g| alliance(w, g).is_some_and(|a| alliance_is_member(w, a, g)))
    });
    h.guild_alliance_chat = Some(|w, m, text| {
        if let Some(g) = mobile_guild(w, m)
            && let Some(a) = alliance(w, g)
        {
            alliance_chat_default(w, a, m, text);
        }
    });
    h.guild_chat = Some(|w, m, text| {
        if let Some(g) = mobile_guild(w, m) {
            guild_chat_default(w, g, m, text);
        }
    });
    let n = w.resources.get_mut::<NotorietyHooks>();
    n.guild_relation = Some(relation_hook);
    n.guild_peaceful = Some(|w, g| {
        data(w, g).is_some_and(|d| d.enemies.is_empty()) && guild_type(w, g) == GuildType::Regular
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};

    #[test]
    fn guild_save_bytes_round_trip() {
        let mut w = World::new();
        let mut b = blank(Serial::new(7), DateTime::MIN);
        b.name = "Test".into();
        b.abbreviation = "TST".into();
        b.ty = GuildType::Chaos;
        b.ext.get_mut::<GuildData>().unwrap().charter = Some("c".into());
        let g = guild::add_guild(&mut w, b);

        let mut out = SaveWriter::new(true);
        serialize(&w, g, &mut out);
        let bytes = out.as_slice().to_vec();

        let mut expect = SaveWriter::new(true);
        expect.write_i32(5);
        expect.write_i32(0); // pending wars
        expect.write_i32(0); // accepted wars
        expect.write_bool(false); // not an alliance leader
        expect.write_serial(Serial::MINUS_ONE); // m_AllianceLeader
        expect.write_i32(0); // ally declarations
        expect.write_i32(0); // ally invitations
        expect.write_date_time_ticks(DateTime::MIN.ticks());
        expect.write_i32(1); // Chaos
        expect.write_date_time_ticks(DateTime::MIN.ticks());
        expect.write_serial(Serial::MINUS_ONE); // leader
        expect.write_string("Test");
        expect.write_string("TST");
        for _ in 0..7 {
            expect.write_i32(0); // allies, enemies, war lists, members, candidates, accepted
        }
        expect.write_serial(Serial::MINUS_ONE);
        expect.write_serial(Serial::MINUS_ONE);
        expect.write_opt_string(Some("c"));
        expect.write_opt_string(None);
        assert_eq!(bytes, expect.as_slice());

        let g2 = guild::add_guild(&mut w, blank(Serial::new(8), DateTime::MIN));
        let mut r = SaveReader::new(&bytes);
        deserialize_guild(&mut w, g2, &mut r).unwrap();
        assert_eq!(r.position() as usize, bytes.len());
        let mut again = SaveWriter::new(true);
        serialize(&w, g2, &mut again);
        assert_eq!(again.as_slice(), &bytes[..]);
        assert_eq!(guild::world_find_by_abbrev(&w, "tst"), Some(g));
    }
}

/// Guild serialization purity tests.
#[cfg(test)]
mod purity_tests {
    use super::*;
    use muo_io::SaveWriter;
    use muo_primitives::Point3D;

    fn create_player(w: &mut World) -> MobileId {
        let m = crate::player_mobile::new_player_mobile(w);
        muo_server::mobile::base::move_to_world(
            w,
            m,
            Point3D::new(1500, 1500, 0),
            muo_server::map::MapId::FELUCCA,
        );
        m
    }

    #[test]
    fn serialize_does_not_recalculate_fealty_or_mutate_state() {
        let mut w = muo_server::map::test_support::world();
        let leader = create_player(&mut w);
        let g = new_guild(&mut w, leader, "Purity Test Guild", "PTG");
        let stale = w.now - TimeSpan::from_days(3);
        data_mut(&mut w, g).unwrap().last_fealty = stale;

        let mut out = SaveWriter::new(true);
        serialize(&w, g, &mut out);

        assert_eq!(data(&w, g).unwrap().last_fealty, stale);
        assert_eq!(data(&w, g).unwrap().leader, Some(leader));

        let first = out.as_slice().to_vec();
        let mut again = SaveWriter::new(true);
        serialize(&w, g, &mut again);
        assert_eq!(first, again.as_slice());
    }

    #[test]
    fn run_maintenance_recalculates_stale_fealty() {
        let mut w = muo_server::map::test_support::world();
        let leader = create_player(&mut w);
        let g = new_guild(&mut w, leader, "Maintenance Test Guild", "MTG");
        let stale = w.now - TimeSpan::from_days(3);
        data_mut(&mut w, g).unwrap().last_fealty = stale;

        run_maintenance(&mut w, g);

        assert!(w.now - data(&w, g).unwrap().last_fealty < TimeSpan::from_mins(1));
    }
}
