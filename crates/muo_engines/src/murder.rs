//! The player murder system: `PlayerMurderSystem` (login, disconnect, the five-minute decay timer,
//! bounties, reports), `MurderContext`'s behavior, `ReportMurdererGump`, `BountyReportMurdererGump` and
//! `BountyMessage` (the bulletin-board packets of the bounty board).
//!
//! The data and the `PlayerMurders` save file are `muo_content::player_mobile::tables::{MurderContext, MurderTable}`.
//! `BountyBoard` (an item over `BaseBulletinBoard`) is not here: there is no bulletin board yet; its
//! packets are [`bounty_container_content_packet`] and [`bounty_bb_message_packet`], and its two decisions are
//! [`get_active_bounties`] and [`get_murder_context`].

use indexmap::{IndexMap, IndexSet};
use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::{DynamicGumpBuilder, GumpStrings, StaticGumpBuilder, send_gump_to_mobile};
use muo_content::mx;
use muo_content::player_mobile::tables::{MurderContext, MurderTable};
use muo_content::player_mobile::{self, NpcGuild};
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::messages::send_localized_message;
use muo_server::net_state::NetStateId;
use muo_server::timer;

/// The statics of `PlayerMurderSystem`.
#[derive(Default)]
pub struct MurderState {
    /// `_contextTerms`: the players that are online and have something to decay.
    terms: IndexSet<MobileId>,
    /// `MurderContext._nextElapse` per player.
    next_elapse: IndexMap<MobileId, DateTime>,
    recently_reported: IndexSet<(MobileId, MobileId)>,
    recently_reported_delay: TimeSpan,
    short_term_murder_duration: TimeSpan,
    long_term_murder_duration: TimeSpan,
    pub bounties_enabled: bool,
    bounty_expiry: TimeSpan,
}

fn st(w: &mut World) -> &mut MurderState {
    w.resources.get_mut::<MurderState>()
}

fn st_ref(w: &World) -> Option<&MurderState> {
    w.resources.get::<MurderState>()
}

/// `PlayerMurderSystem.ShortTermMurderDuration`.
pub fn short_term_murder_duration(w: &World) -> TimeSpan {
    st_ref(w).map_or(TimeSpan::from_hours(8), |s| s.short_term_murder_duration)
}

/// `PlayerMurderSystem.LongTermMurderDuration`.
pub fn long_term_murder_duration(w: &World) -> TimeSpan {
    st_ref(w).map_or(TimeSpan::from_hours(40), |s| s.long_term_murder_duration)
}

/// `PlayerMurderSystem.PingPongEnabled`.
pub fn ping_pong_enabled(w: &World) -> bool {
    muo_content::core::t2a(w) && !muo_content::core::lbr(w)
}

/// `PlayerMurderSystem.Configure`.
pub fn configure(w: &mut World) {
    let lbr = muo_content::core::lbr(w);
    let c = &mut w.config;
    let short = c
        .get_or_update_setting(
            "murderSystem.shortTermMurderDuration",
            TimeSpan::from_hours(8),
        )
        .unwrap_or(TimeSpan::from_hours(8));
    let long = c
        .get_or_update_setting(
            "murderSystem.longTermMurderDuration",
            TimeSpan::from_hours(40),
        )
        .unwrap_or(TimeSpan::from_hours(40));
    let bounties = c
        .get_or_update_setting("murderSystem.bountiesEnabled", !lbr)
        .unwrap_or(!lbr);
    let delay = c
        .get_or_update_setting(
            "murderSystem.recentlyReportedDelay",
            TimeSpan::from_mins(10),
        )
        .unwrap_or(TimeSpan::from_mins(10));
    let expiry = c
        .get_or_update_setting("murderSystem.bountyExpiry", TimeSpan::from_days(14))
        .unwrap_or(TimeSpan::from_days(14));
    let s = st(w);
    s.short_term_murder_duration = short;
    s.long_term_murder_duration = long;
    s.bounties_enabled = bounties;
    s.recently_reported_delay = delay;
    s.bounty_expiry = expiry;
    // The persistence (`PlayerMurders`) is registered by `player_mobile::tables::register_persistence`.
}

/// `PlayerMurderSystem.Initialize` and `MurdererTimer.Initialize`.
pub fn initialize(w: &mut World) {
    w.events
        .disconnected
        .subscribe(|w, m| on_disconnected(w, *m));
    muo_content::network::events::subscribe_ranked::<muo_content::network::events::PlayerLogin>(
        w,
        9,
        |w, e| on_login(w, e.0),
    );
    player_mobile::subscribe_death(w, on_player_death_event);
    player_mobile::subscribe_deleted(w, |w, m| {
        // `OnPlayerDeleted` drops the context and its term entry
        let s = st(w);
        s.terms.shift_remove(&m);
        s.next_elapse.shift_remove(&m);
    });
    // keywords: *i must consider my sins*
    w.events.speech.subscribe(|w, args| {
        if args.keywords.contains(&0x0032) && is_player(w, args.mobile) {
            report_kills_to_self(w, args.mobile);
        }
    });
    let five = TimeSpan::from_mins(5);
    timer::start(w, five, five, 0, murderer_tick);
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.ty.is(&player_mobile::PLAYER_MOBILE))
}

fn table(w: &World) -> Option<&MurderTable> {
    w.resources.get::<MurderTable>()
}

/// `PlayerMurderSystem.GetMurderContext(player, out context)`.
pub fn get_murder_context(w: &World, player: MobileId) -> Option<&MurderContext> {
    table(w)?.0.get(&player)
}

fn ctx_mut(w: &mut World, player: MobileId) -> Option<&mut MurderContext> {
    w.resources.get_mut::<MurderTable>().0.get_mut(&player)
}

/// `PlayerMurderSystem.GetOrCreateMurderContext(player)`.
pub fn get_or_create_murder_context(w: &mut World, player: MobileId) -> &mut MurderContext {
    w.resources
        .get_mut::<MurderTable>()
        .0
        .entry(player)
        .or_default()
}

fn kills(w: &World, p: MobileId) -> i32 {
    w.mobile(p).map_or(0, |m| m.kills)
}

// ---- MurderContext -------------------------------------------------------------------------------------------------

/// `MurderContext.ResetKillTime()`.
pub fn reset_kill_time(w: &mut World, p: MobileId) {
    let game_time = player_mobile::game_time(w, p);
    let (short, long) = (short_term_murder_duration(w), long_term_murder_duration(w));
    let k = kills(w, p);
    if let Some(c) = ctx_mut(w, p) {
        if c.short_term_murders > 0 {
            c.short_term_elapse = game_time + short;
        }
        if k > 0 {
            c.long_term_elapse = game_time + long;
        }
    }
}

/// `MurderContext.DecayKills()`.
pub fn decay_kills(w: &mut World, p: MobileId) {
    let game_time = player_mobile::game_time(w, p);
    let (short, long) = (short_term_murder_duration(w), long_term_murder_duration(w));
    let k = kills(w, p);
    let mut dec_kills = false;
    if let Some(c) = ctx_mut(w, p) {
        if c.short_term_murders > 0 && c.short_term_elapse < game_time {
            c.short_term_elapse += short;
            c.short_term_murders = (c.short_term_murders - 1).max(0);
        }
        if k > 0 && c.long_term_elapse < game_time {
            c.long_term_elapse += long;
            dec_kills = true;
        }
    }
    if dec_kills {
        props_b::set_kills(w, p, k - 1);
    }
}

/// `MurderContext.CanRemove()`.
pub fn can_remove(w: &World, p: MobileId) -> bool {
    get_murder_context(w, p)
        .is_none_or(|c| c.ping_pongs <= 0 && c.short_term_murders <= 0 && kills(w, p) <= 0)
}

/// `MurderContext.CheckStart()`.
pub fn check_start(w: &mut World, p: MobileId) -> bool {
    let now = w.now;
    let game_time = player_mobile::game_time(w, p);
    let k = kills(w, p);
    let mut next = DateTime::MAX;
    if let Some(c) = get_murder_context(w, p) {
        if c.short_term_murders > 0 {
            next = now + (c.short_term_elapse - game_time);
        }
        if k > 0 {
            let l = now + (c.long_term_elapse - game_time);
            if l < next {
                next = l;
            }
        }
    }
    st(w).next_elapse.insert(p, next);
    next != DateTime::MAX
}

// ---- PlayerMurderSystem ------------------------------------------------------------------------------------------------

/// `PlayerMurderSystem.OnLogin(pm)`.
pub fn on_login(w: &mut World, pm: MobileId) {
    if get_murder_context(w, pm).is_none() {
        return;
    }
    if check_start(w, pm) {
        st(w).terms.insert(pm);
    } else {
        st(w).terms.shift_remove(&pm);
        if can_remove(w, pm) {
            w.resources.get_mut::<MurderTable>().0.shift_remove(&pm);
        }
    }
}

fn on_disconnected(w: &mut World, m: MobileId) {
    if !is_player(w, m) || get_murder_context(w, m).is_none() {
        return;
    }
    decay_kills(w, m);
    st(w).terms.shift_remove(&m);
    if can_remove(w, m) {
        w.resources.get_mut::<MurderTable>().0.shift_remove(&m);
    }
}

/// `UpdateMurderContext(context)`.
pub fn update_murder_context(w: &mut World, p: MobileId) {
    if !check_start(w, p) {
        if can_remove(w, p) {
            w.resources.get_mut::<MurderTable>().0.shift_remove(&p);
        }
        st(w).terms.shift_remove(&p);
    } else if props_b::net_state(w, p).is_some() {
        st(w).terms.insert(p);
    }
}

/// `PlayerMurderSystem.GetBounty(player)`.
pub fn get_bounty(w: &World, p: MobileId) -> i32 {
    get_murder_context(w, p).map_or(0, |c| c.bounty)
}

/// `AddBounty(player, amount)`.
pub fn add_bounty(w: &mut World, p: MobileId, amount: i32) {
    if let Some(c) = ctx_mut(w, p) {
        c.bounty = c.bounty.wrapping_add(amount);
    }
}

/// `ClearBounty(player)`.
pub fn clear_bounty(w: &mut World, p: MobileId) {
    if let Some(c) = ctx_mut(w, p) {
        c.bounty = 0;
    }
}

fn bounty_cutoff(w: &World) -> Option<DateTime> {
    let expiry = st_ref(w).map_or(TimeSpan::from_days(14), |s| s.bounty_expiry);
    (expiry != TimeSpan::ZERO).then(|| w.now - expiry)
}

fn is_active_bounty(w: &World, p: MobileId, c: &MurderContext, cutoff: Option<DateTime>) -> bool {
    let murderer = w.mobile(p).is_some_and(|m| (m.ty.murderer)(w, p));
    murderer && c.bounty > 0 && cutoff.is_none_or(|t| c.last_murder_time >= t)
}

/// `GetActiveBountyCount()`.
pub fn get_active_bounty_count(w: &World) -> usize {
    let cutoff = bounty_cutoff(w);
    table(w).map_or(0, |t| {
        t.0.iter()
            .filter(|(p, c)| is_active_bounty(w, **p, c, cutoff))
            .count()
    })
}

/// `GetActiveBounties()`: highest bounty first (ties keep table order).
pub fn get_active_bounties(w: &World) -> Vec<(MobileId, i32)> {
    let cutoff = bounty_cutoff(w);
    let mut v: Vec<(MobileId, i32)> = table(w)
        .map(|t| {
            t.0.iter()
                .filter(|(p, c)| is_active_bounty(w, **p, c, cutoff))
                .map(|(p, c)| (*p, c.bounty))
                .collect()
        })
        .unwrap_or_default();
    muo_primitives::dotnet::net_sort(&mut v, |a, b| b.1.cmp(&a.1));
    v
}

/// `ManuallySetPingPong(player, pingPong)`.
pub fn manually_set_ping_pong(w: &mut World, p: MobileId, ping_pong: i32) {
    get_or_create_murder_context(w, p).ping_pongs = ping_pong.max(0);
    update_murder_context(w, p);
}

/// `ManuallySetShortTermMurders(player, n)`.
pub fn manually_set_short_term_murders(w: &mut World, p: MobileId, n: i32) {
    get_or_create_murder_context(w, p).short_term_murders = n.max(0); // AllowShortTermMurdersChange
    reset_kill_time(w, p);
    update_murder_context(w, p);
}

/// `OnPlayerMurder(player)`.
pub fn on_player_murder(w: &mut World, p: MobileId) {
    let c = get_or_create_murder_context(w, p);
    c.short_term_murders += 1;
    let k = kills(w, p) + 1;
    props_b::set_kills(w, p, k);
    let pp = ping_pong_enabled(w);
    let now = w.now;
    let c = get_or_create_murder_context(w, p);
    if pp && k == 5 {
        c.ping_pongs += 1;
    }
    c.last_murder_time = now;
    reset_kill_time(w, p);
    update_murder_context(w, p);
}

/// `IsRecentlyReported(reporter, killer)`.
pub fn is_recently_reported(w: &World, reporter: MobileId, killer: MobileId) -> bool {
    st_ref(w).is_some_and(|s| s.recently_reported.contains(&(reporter, killer)))
}

/// `ReportMurder(reporter, killer)`.
pub fn report_murder(w: &mut World, reporter: MobileId, killer: MobileId) -> bool {
    if w.mobile(killer).is_none() || !is_player(w, killer) {
        return false;
    }
    if !st(w).recently_reported.insert((reporter, killer)) {
        return false;
    }
    let delay = st_ref(w).map_or(TimeSpan::from_mins(10), |s| s.recently_reported_delay);
    timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        st(w).recently_reported.shift_remove(&(reporter, killer));
    });
    let was_murderer = w.mobile(killer).is_some_and(|m| (m.ty.murderer)(w, killer));
    on_player_murder(w, killer);
    let k = kills(w, killer);
    let karma = k * if muo_content::core::aos(w) {
        -2000
    } else {
        -1000
    };
    muo_content::misc::titles::set_karma(w, killer, karma, true);
    send_localized_message(w, killer, 1049067, "", 0x3B2); // You have been reported for murder!
    let is_murderer = w.mobile(killer).is_some_and(|m| (m.ty.murderer)(w, killer));
    if !was_murderer && is_murderer {
        send_localized_message(w, killer, 502134, "", 0x3B2); // You are now known as a murderer!
    }
    // with the introduction of PingPongs, a red can technically have 1 kill.
    let suspend = w
        .resources
        .get::<muo_content::skills::stealing::StealingSettings>()
        .is_none_or(|s| s.suspend_on_murder);
    let thief =
        player_mobile::data(w, killer).is_some_and(|d| d.npc_guild == NpcGuild::ThievesGuild);
    if suspend && k == 1 && thief {
        send_localized_message(w, killer, 501562, "", 0x3B2); // You have been suspended by the Thieves Guild.
    }
    true
}

/// `ReportKillsToSelf(player)`.
pub fn report_kills_to_self(w: &mut World, p: MobileId) {
    let (short, pingpongs, k) = get_murder_context(w, p).map_or((0, 0, kills(w, p)), |c| {
        (c.short_term_murders, c.ping_pongs, kills(w, p))
    });
    if muo_content::core::sa(w) {
        send_localized_message(w, p, 1114370, &format!("{short}\t{k}"), 0x3B2);
    } else if muo_content::core::se(w) {
        mx::send_message(
            w,
            p,
            &format!("Short Term Murders: {short} Long Term Murders: {k}"),
        );
    } else if muo_content::core::aos(w) {
        mx::send_message(w, p, &format!("Short Term Murders: {short}"));
        mx::send_message(w, p, &format!("Long Term Murders: {k}"));
        if ping_pong_enabled(w) {
            mx::send_message(w, p, &format!("Ping Pongs: {pingpongs}"));
        }
    } else if !muo_content::core::t2a(w) {
        // No "i must consider my sins" before t2a
    } else if ping_pong_enabled(w) && pingpongs >= 5 {
        // Thou art known throughout the land as a murderous brigand.
        send_localized_message(w, p, 502123, "", if short >= 5 { 0x22 } else { 0x59 });
    } else if short >= 5 {
        // If thou should return to the land of the living, the innocent shall wreak havoc upon thy soul
        send_localized_message(w, p, 502126, "", 0x22);
    } else if short > 0 {
        // Although thou hast slain the innocent, thy deeds shall not bring retribution upon thy return to the living
        send_localized_message(w, p, 502125, "", 0x59);
    } else if k > 0 {
        // Fear not, thou hast not slain the innocent in some time...
        send_localized_message(w, p, 502124, "", 0x59);
    } else {
        // Fear not, thou hast not slain the innocent.
        send_localized_message(w, p, 502122, "", 0x59);
    }
}

/// `MurdererTimer.OnTick`.
fn murderer_tick(w: &mut World) {
    let terms: Vec<MobileId> = match st_ref(w) {
        Some(s) if !s.terms.is_empty() => s.terms.iter().copied().collect(),
        _ => return,
    };
    for p in terms {
        decay_kills(w, p);
        if !check_start(w, p) && get_murder_context(w, p).is_some() {
            if can_remove(w, p) {
                w.resources.get_mut::<MurderTable>().0.shift_remove(&p);
            }
            st(w).terms.shift_remove(&p);
        }
    }
}

// ---- ReportMurdererGump / BountyReportMurdererGump ----------------------------------------------------------------------------

/// `ReportMurdererGump.OnPlayerDeathEvent(m)`.
pub fn on_player_death_event(w: &mut World, m: MobileId) {
    let mut killers: Vec<MobileId> = Vec::new();
    let mut to_give: IndexSet<MobileId> = IndexSet::new();
    // Guards won't take reports of the death of a thief!
    let not_in_thieves =
        player_mobile::data(w, m).is_none_or(|d| d.npc_guild != NpcGuild::ThievesGuild);
    let now = w.now;
    let aggressors = w
        .mobile(m)
        .map(|x| x.aggressors.clone())
        .unwrap_or_default();
    let mut marks: Vec<MobileId> = Vec::new();
    for ai in &aggressors {
        let attacker_player = is_player(w, ai.attacker);
        if attacker_player
            && ai.can_report_murder
            && !ai.reported
            && !is_recently_reported(w, m, ai.attacker)
        {
            if not_in_thieves {
                killers.push(ai.attacker);
            }
            marks.push(ai.attacker);
        }
        if attacker_player && now - ai.last_combat_time < TimeSpan::from_secs_f64(30.0) {
            to_give.insert(ai.attacker);
        }
    }
    if let Some(x) = w.mobile_mut(m) {
        for ai in x
            .aggressors
            .iter_mut()
            .filter(|ai| marks.contains(&ai.attacker))
        {
            ai.reported = true;
            ai.can_report_murder = false;
        }
    }
    let aggressed = w.mobile(m).map(|x| x.aggressed.clone()).unwrap_or_default();
    for ai in &aggressed {
        if is_player(w, ai.defender) && now - ai.last_combat_time < TimeSpan::from_secs_f64(30.0) {
            to_give.insert(ai.defender);
        }
    }
    if !to_give.is_empty() {
        let map = w.mobile(m).and_then(|x| x.map);
        let (fame_award, _) = muo_content::misc::titles::compute_kill_awards(w, m, map);
        for g in to_give {
            muo_content::misc::titles::award_fame(w, g, fame_award, false);
        }
    }
    if not_in_thieves && !killers.is_empty() {
        timer::start(
            w,
            TimeSpan::from_secs_f64(4.0),
            TimeSpan::ZERO,
            1,
            move |w| {
                if st_ref(w).is_some_and(|s| s.bounties_enabled) && is_player(w, m) {
                    send_bounty_report_gump(w, m, killers.clone(), 0);
                } else {
                    send_report_gump(w, m, killers.clone(), 0);
                }
            },
        );
    }
}

fn name_of(w: &World, m: MobileId) -> String {
    w.mobile(m)
        .and_then(|x| (x.ty.name)(w, m))
        .unwrap_or_default()
}

pub struct ReportMurdererGump {
    killers: Vec<MobileId>,
    idx: usize,
    killer_name: String,
}

fn send_report_gump(w: &mut World, to: MobileId, killers: Vec<MobileId>, idx: usize) {
    let killer_name = killers.get(idx).map(|k| name_of(w, *k)).unwrap_or_default();
    let g = StaticGumpObj::new(
        0,
        0,
        ReportMurdererGump {
            killers,
            idx,
            killer_name,
        },
    );
    send_gump_to_mobile(w, to, Box::new(g), false);
}

impl StaticGumpLogic for ReportMurdererGump {
    const TYPE_NAME: &'static str = "Server.Engines.PlayerMurderSystem.ReportMurdererGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.set_no_close();
        b.set_no_resize();
        b.add_background(265, 205, 320, 290, 5054);
        b.add_page(0);
        b.add_image_tiled(225, 175, 50, 45, 0xCE); // Top left corner
        b.add_image_tiled(267, 175, 315, 44, 0xC9); // Top bar
        b.add_image_tiled(582, 175, 43, 45, 0xCF); // Top right corner
        b.add_image_tiled(225, 219, 44, 270, 0xCA); // Left side
        b.add_image_tiled(582, 219, 44, 270, 0xCB); // Right side
        b.add_image_tiled(225, 489, 44, 43, 0xCC); // Lower left corner
        b.add_image_tiled(267, 489, 315, 43, 0xE9); // Lower Bar
        b.add_image_tiled(582, 489, 43, 43, 0xCD); // Lower right corner
        b.add_page(1);
        b.add_html_placeholder(260, 234, 300, 140, "killerName", false, false); // Player's Name
        b.add_html_localized(260, 254, 300, 140, 1049066, false, false); // Would you like to report...
        b.add_button(260, 300, 0xFA5, 0xFA7, 1);
        b.add_html_localized(300, 300, 300, 50, 1046362, false, false); // Yes
        b.add_button(360, 300, 0xFA5, 0xFA7, 2);
        b.add_html_localized(400, 300, 300, 50, 1046363, false, false); // No
    }

    fn build_strings(&self, s: &mut GumpStrings) {
        s.set_string_slot("killerName", &self.killer_name);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        if info.button_id == 1
            && let Some(&k) = self.killers.get(self.idx)
        {
            report_murder(w, from, k);
        }
        if self.idx + 1 < self.killers.len() {
            send_report_gump(w, from, std::mem::take(&mut self.killers), self.idx + 1);
        }
    }
}

pub struct BountyReportMurdererGump {
    victim: MobileId,
    killers: Vec<MobileId>,
    idx: usize,
    killer_name: String,
    killer_kills: i32,
    bounty_max: i32,
}

fn send_bounty_report_gump(w: &mut World, victim: MobileId, killers: Vec<MobileId>, idx: usize) {
    let killer = killers.get(idx).copied();
    let killer_name = killer.map(|k| name_of(w, k)).unwrap_or_default();
    let killer_kills = killer.map_or(0, |k| kills(w, k));
    let bounty_max = muo_content::vendor::banker::get_balance(w, victim);
    let g = DynamicGumpObj::new(
        0,
        0,
        BountyReportMurdererGump {
            victim,
            killers,
            idx,
            killer_name,
            killer_kills,
            bounty_max,
        },
    );
    send_gump_to_mobile(w, victim, Box::new(g), false);
}

impl DynamicGumpLogic for BountyReportMurdererGump {
    const TYPE_NAME: &'static str = "Server.Engines.PlayerMurderSystem.BountyReportMurdererGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();
        b.set_no_resize();
        b.add_background(265, 205, 393, 270, 70000);
        b.add_image(265, 205, 1140, 0);
        b.add_page(0);
        b.add_html(
            325,
            255,
            300,
            60,
            &format!(
                "<BIG>Would you like to report {} as a murderer?</BIG>",
                self.killer_name
            ),
            false,
            false,
        );
        if self.killer_kills >= 4 && self.bounty_max > 0 {
            b.add_html(
                325,
                325,
                300,
                60,
                &format!("<BIG>Optional Bounty: [{} max] </BIG>", self.bounty_max),
                false,
                false,
            );
            b.add_image(323, 343, 0x475, 0);
            b.add_text_entry(329, 346, 311, 16, 0, 1, "");
        }
        b.add_button(385, 395, 0x47B, 0x47D, 1);
        b.add_button(465, 395, 0x478, 0x47A, 2);
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.victim;
        if info.button_id == 1
            && let Some(&killer) = self.killers.get(self.idx)
            && report_murder(w, from, killer)
            && is_player(w, killer)
        {
            let text = info.get_text_entry(1).unwrap_or("");
            if let Some(requested) = (!text.trim().is_empty())
                .then(|| text.trim().parse::<i32>().ok())
                .flatten()
                .filter(|&r| r > 0)
            {
                let bounty = requested.min(muo_content::vendor::banker::get_balance(w, from));
                if bounty > 0 && muo_content::vendor::banker::withdraw(w, from, bounty) {
                    add_bounty(w, killer, bounty);
                    let who = name_of(w, from);
                    let unit = if bounty == 1 {
                        "gold piece"
                    } else {
                        "gold pieces"
                    };
                    mx::send_message(
                        w,
                        killer,
                        &format!("{who} has placed a bounty of {bounty} {unit} on your head!"),
                    );
                    mx::send_message(
                        w,
                        from,
                        &format!(
                            "You place a bounty of {bounty}gp on {}'s head.",
                            name_of(w, killer)
                        ),
                    );
                }
            }
        }
        if self.idx + 1 < self.killers.len() {
            send_bounty_report_gump(w, from, std::mem::take(&mut self.killers), self.idx + 1);
        }
    }
}

// ---- BountyMessage (packets) ---------------------------------------------------------------------------------------------

/// `BountyBoard.SyntheticSerialBase`.
pub const SYNTHETIC_SERIAL_BASE: u32 = 0x8000_0000;

/// `SendBountyContainerContent`: the 0x3C packet whose entries are `base + player serial`.
pub fn bounty_container_content_packet(
    board_serial: u32,
    players: &[u32],
    grid_lines: bool,
) -> Vec<u8> {
    let mut p = vec![0x3C, 0, 0, 0, 0];
    for &s in players {
        p.extend_from_slice(&SYNTHETIC_SERIAL_BASE.wrapping_add(s).to_be_bytes());
        p.extend_from_slice(&0xEB0u16.to_be_bytes()); // BulletinMessage ItemID
        p.push(0); // signed, itemID offset
        p.extend_from_slice(&1u16.to_be_bytes()); // Amount
        p.extend_from_slice(&0i16.to_be_bytes()); // X
        p.extend_from_slice(&0i16.to_be_bytes()); // Y
        if grid_lines {
            p.push(0); // Grid location
        }
        p.extend_from_slice(&board_serial.to_be_bytes()); // Container = this board
        p.extend_from_slice(&0u16.to_be_bytes()); // Hue
    }
    let len = p.len() as u16;
    p[1..3].copy_from_slice(&len.to_be_bytes());
    p[3..5].copy_from_slice(&(players.len() as u16).to_be_bytes());
    p
}

/// `BulletinBoardPackets.WriteString(text, buffer, pad)`.
fn write_bb_string(p: &mut Vec<u8>, text: &str, pad: bool) {
    let tail = if pad { 2 } else { 1 };
    let mut bytes = text.as_bytes();
    if bytes.len() > 768 {
        bytes = &bytes[..768];
    }
    let length = bytes.len().min(if pad { 253 } else { 254 });
    p.push((length + tail) as u8);
    p.extend_from_slice(&bytes[..length]);
    if pad {
        p.extend_from_slice(&[0, 0]); // Compensating for an old client bug
    } else {
        p.push(0);
    }
}

/// What `SendBountyBBMessage` reads of the wanted player.
pub struct BountyPlayer {
    pub name: String,
    pub raw_name: String,
    pub kills: i32,
    pub female: bool,
    pub body: i16,
    pub hue: i16,
    pub hair_item_id: i32,
    pub hair_hue: i32,
    /// `(ItemID, Hue)` of the items on layers `OneHanded..=Mount`, in item order.
    pub equipment: Vec<(i16, i16)>,
}

/// `SendBountyBBMessage`: the 0x71 packet (`content` false is the header).
pub fn bounty_bb_message_packet(
    board_serial: u32,
    message_serial: u32,
    player: &BountyPlayer,
    bounty: i32,
    last_murder: DateTime,
    content: bool,
    rng: &mut impl muo_primitives::Rng,
) -> Vec<u8> {
    let mut p = vec![0x71, 0, 0, if content { 2 } else { 1 }];
    p.extend_from_slice(&board_serial.to_be_bytes());
    p.extend_from_slice(&message_serial.to_be_bytes());
    if !content {
        p.extend_from_slice(&0u32.to_be_bytes()); // Thread serial (all root-level)
    }
    write_bb_string(&mut p, &player.name, false);
    write_bb_string(&mut p, &format!("{bounty} gold"), false);
    write_bb_string(&mut p, &mmm_dd_yyyy(last_murder), false);
    if content {
        p.extend_from_slice(&player.body.to_be_bytes());
        p.extend_from_slice(&player.hue.to_be_bytes());
        let n = player.equipment.len().min(255);
        p.push(n as u8);
        for (id, hue) in player.equipment.iter().take(n) {
            p.extend_from_slice(&id.to_be_bytes());
            p.extend_from_slice(&hue.to_be_bytes());
        }
        write_content_lines(&mut p, player, bounty, rng);
    }
    let len = p.len() as u16;
    p[1..3].copy_from_slice(&len.to_be_bytes());
    p
}

fn write_content_lines(
    p: &mut Vec<u8>,
    player: &BountyPlayer,
    bounty: i32,
    rng: &mut impl muo_primitives::Rng,
) {
    use muo_primitives::utility::random;
    let (pronoun, possessive, objective) = if player.female {
        ("she", "her", "her")
    } else {
        ("he", "his", "him")
    };
    let line_count_pos = p.len();
    p.push(0);
    let mut line_count = 0usize;
    let raw = &player.raw_name;
    let title = match random(rng, 6) {
        0 => format!("Bounty for {raw}!"),
        1 => format!("{raw} must die!"),
        2 => format!("A price on {raw}!"),
        3 => format!("{raw} outlawed!"),
        4 => format!("Execute {raw}!"),
        _ => format!("WANTED: {raw}!"),
    };
    write_bb_string(p, &title, true);
    line_count += 1;
    write_bb_string(p, "", true);
    line_count += 1;
    let verb = match random(rng, 18) {
        0 => "hath murdered one too many!",
        1 => "shall not slay again!",
        2 => "hath slain too many!",
        3 => "cannot continue to kill!",
        4 => "must be stopped.",
        5 => "is a bloodthirsty monster.",
        6 => "is a killer of the worst sort.",
        7 => "hath no conscience!",
        8 => "hath cowardly slain many.",
        9 => "must die for all our sakes.",
        10 => "sheds innocent blood!",
        11 => "must fall to preserve us.",
        12 => "must be taken care of.",
        13 => "is a thug and must die.",
        14 => "cannot be redeemed.",
        15 => "is a shameless butcher.",
        16 => "is a callous monster.",
        _ => "is a cruel, casual killer.",
    };
    let intro = match random(rng, 7) {
        0 => "  A bounty is hereby offered",
        1 => "  Lord British sets a price",
        2 => "  Claim the reward! 'Tis",
        3 => "  Lord Blackthorn set a price",
        4 => "  The Paladins set a price",
        5 => "  The Merchants set a price",
        _ => "  Lord British's bounty ",
    };
    let para = format!(
        "The foul scum known as {raw} {verb}  For {pronoun} is responsible for {} murders.  {intro} of {bounty} gold pieces for {possessive} head!",
        player.kills
    );
    for l in word_wrap(&para, 28) {
        write_bb_string(p, &l, true);
        line_count += 1;
    }
    write_bb_string(p, "", true);
    line_count += 1;
    write_bb_string(p, "  A description:", true);
    line_count += 1;
    for l in [
        format!("    - {}", hair_style(player.hair_item_id)),
        format!("    - {} hair", hair_color(player.hair_hue)),
        format!("    - {} skin", skin_tone(i32::from(player.hue))),
    ] {
        write_bb_string(p, &l, true);
        line_count += 1;
    }
    write_bb_string(p, "", true);
    line_count += 1;
    write_bb_string(p, &format!("If you kill {objective}, remove the"), true);
    write_bb_string(p, "head, and give it to a guard", true);
    write_bb_string(p, "to claim your reward.", true);
    line_count += 3;
    p[line_count_pos] = line_count.min(255) as u8;
}

/// `WriteWordWrappedLines`: breaks at the last space within `max_width`, a hard break when there is none; the last
/// segment gets a trailing space.
pub fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut current = 0;
    while current < chars.len() {
        let remaining = chars.len() - current;
        if remaining > max_width {
            let mut length = max_width;
            while length > 0 && chars[current + length] != ' ' {
                length -= 1;
            }
            length = if length == 0 { max_width } else { length + 1 }; // include the space
            out.push(chars[current..current + length].iter().collect());
            current += length;
        } else {
            let mut s: String = chars[current..current + remaining].iter().collect();
            s.push(' ');
            out.push(s);
            current += remaining;
        }
    }
    out
}

fn hair_style(item_id: i32) -> &'static str {
    match item_id {
        0x203B => "hair worn short",
        0x203C => "hair worn long",
        0x203D => "hair tied back",
        0x2044 => "a mohawk hairstyle",
        0x2045 => "pageboy hair",
        0x2046 => "hair tied in buns",
        0x2047 => "curly hair",
        0x2048 => "receding hairline",
        0x2049 => "hair in two pigtails",
        0x204A => "shaved head and topknot",
        _ => "bald",
    }
}

fn hair_color(hue: i32) -> &'static str {
    match hue {
        0 => "indeterminate color",
        0x44E..=0x450 => "white",
        0x451..=0x453 => "graying",
        0x454 | 0x455 => "black",
        0x456..=0x458 => "copper",
        0x459..=0x45C => "brown",
        0x45D => "reddish brown",
        0x45E..=0x460 => "blonde",
        0x461..=0x463 => "light brown",
        0x464 | 0x465 => "golden brown",
        0x466..=0x468 => "golden",
        0x469..=0x46B => "bronze",
        0x46C | 0x46D => "dark brown",
        0x46E | 0x46F => "sandy",
        0x470..=0x472 => "honey-colored",
        0x473..=0x475 => "red",
        0x476..=0x478 => "nut brown",
        0x479..=0x47B => "rich brown",
        0x47C | 0x47D => "very dark brown",
        _ => "outlandishly colored",
    }
}

fn skin_tone(hue: i32) -> &'static str {
    match hue & 0x7FFF {
        0x3F1 | 0x3F2 | 0x3F8 | 0x3FF | 0x400 | 0x406 | 0x407 | 0x408 | 0x40D | 0x40E | 0x415
        | 0x416 => "pale",
        0x3EA | 0x3EB | 0x3F9 | 0x3FA | 0x3FB | 0x417 => "fair",
        0x3F3 | 0x3F4 | 0x3FC | 0x401 | 0x402 | 0x409 | 0x40F | 0x410 | 0x411 | 0x418 => "tanned",
        0x3EC | 0x3ED | 0x3EE | 0x3F5 | 0x403 | 0x412 | 0x413 | 0x419 | 0x421 => "copper",
        0x3EF | 0x3F0 | 0x3F6 | 0x3F7 | 0x3FD | 0x3FE | 0x404 | 0x405 | 0x40A | 0x40B | 0x40C
        | 0x414 | 0x41A | 0x41B | 0x420 => "dark",
        0x41C..=0x41E => "yellow",
        _ => "deathly",
    }
}

/// `ToString("MMM dd, yyyy")` (invariant culture).
fn mmm_dd_yyyy(d: DateTime) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (y, m, day) = d.ymd();
    format!("{} {day:02}, {y:04}", MONTHS[(m as usize - 1) % 12])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_wrap_breaks_on_spaces_and_pads_the_tail() {
        let l = word_wrap("The foul scum known as Bob hath murdered one too many!", 28);
        assert_eq!(l[0], "The foul scum known as Bob ");
        assert!(l.last().unwrap().ends_with("many! "));
        // a word longer than the width is cut hard
        let l = word_wrap(&"x".repeat(40), 28);
        assert_eq!(l[0].len(), 28);
        assert_eq!(l[1], format!("{} ", "x".repeat(12)));
    }

    #[test]
    fn container_content_packet_layout() {
        let p = bounty_container_content_packet(0x4000_0001, &[5], false);
        assert_eq!(p.len(), 5 + 19);
        assert_eq!(&p[1..3], &[0, 24]);
        assert_eq!(&p[3..5], &[0, 1]);
        assert_eq!(&p[5..9], &0x8000_0005u32.to_be_bytes());
        let g = bounty_container_content_packet(1, &[5], true);
        assert_eq!(g.len(), 5 + 20);
    }

    #[test]
    fn descriptions_follow_the_tables() {
        assert_eq!(hair_style(0x2044), "a mohawk hairstyle");
        assert_eq!(hair_style(0), "bald");
        assert_eq!(hair_color(0x455), "black");
        assert_eq!(skin_tone(0x8000 | 0x3EA), "fair");
        assert_eq!(skin_tone(1), "deathly");
    }

    #[test]
    fn date_format() {
        assert_eq!(
            mmm_dd_yyyy(DateTime::from_ymd(2024, 3, 9).unwrap()),
            "Mar 09, 2024"
        );
    }

    #[test]
    fn bb_strings_are_length_prefixed() {
        let mut p = Vec::new();
        write_bb_string(&mut p, "ab", false);
        assert_eq!(p, [3, b'a', b'b', 0]);
        let mut p = Vec::new();
        write_bb_string(&mut p, "ab", true);
        assert_eq!(p, [4, b'a', b'b', 0, 0]);
    }
}

// ---- BountyBoard ---------------------------------------------------------------------------------------------------------

mod bounty_board {
    use super::*;
    use muo_io::SpanReader;
    use muo_items::bulletin::{self, BASE_BULLETIN_BOARD, BoardClass};
    use muo_server::ids::ItemId;
    use muo_server::item::{ItemType, ops};
    use muo_server::layer::Layer;
    use muo_server::mobile::part2::messages::local_overhead_message_number;

    fn player_data(w: &World, p: MobileId) -> BountyPlayer {
        let m = w.mobile(p);
        let body = muo_server::mobile::part1::props_b::body(w, p).0;
        let equipment = m
            .map(|m| {
                m.items
                    .iter()
                    .filter_map(|i| w.item(*i))
                    .filter(|i| i.layer >= Layer::OneHanded && i.layer <= Layer::Mount)
                    .map(|i| (i.item_id as i16, i.hue as i16))
                    .collect()
            })
            .unwrap_or_default();
        BountyPlayer {
            name: name_of(w, p),
            raw_name: props_b::raw_name(w, p).unwrap_or_default(),
            kills: kills(w, p),
            female: muo_server::mobile::part1::props_b::body(w, p).is_female(),
            body: body as i16,
            hue: m.map_or(0, |m| (m.ty.hue)(w, p)) as i16,
            hair_item_id: m.map_or(0, |m| m.hair_item_id),
            hair_hue: m.map_or(0, |m| m.hair_hue),
            equipment,
        }
    }

    fn handle_bb_request(
        w: &mut World,
        board: ItemId,
        packet_id: u8,
        from: MobileId,
        r: &mut SpanReader,
    ) -> bool {
        match packet_id {
            3 | 4 => {
                // Request content / header
                let Ok(synthetic) = r.read_u32() else {
                    return true;
                };
                let player = MobileId(muo_primitives::Serial::new(
                    synthetic.wrapping_sub(SYNTHETIC_SERIAL_BASE),
                ));
                if is_player(w, player)
                    && let Some(c) = get_murder_context(w, player).filter(|c| c.bounty > 0)
                    && let Some(ns) = props_b::net_state(w, from)
                    && !w.cannot_send_packets(ns)
                {
                    let (bounty, last) = (c.bounty, c.last_murder_time);
                    let data = player_data(w, player);
                    let board_serial = w.item(board).map_or(0, |i| i.serial.value());
                    let pk = bounty_bb_message_packet(
                        board_serial,
                        synthetic,
                        &data,
                        bounty,
                        last,
                        packet_id == 3,
                        &mut w.rng,
                    );
                    w.send(ns, &pk);
                }
                true
            }
            5 => {
                send_localized_message(w, from, 1062398, "", 0x3B2); // You are not allowed to post to this bulletin board.
                true
            }
            6 => true, // Remove - blocked
            _ => false,
        }
    }

    pub static BOUNTY_BOARD_CLASS: BoardClass = BoardClass { handle_bb_request };

    fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
        let count = get_active_bounty_count(w);
        if count > 0 {
            let t = format!(
                "a bounty board with {count} posted {}",
                if count == 1 { "bounty" } else { "bounties" }
            );
            ops::label_to(w, id, from, &t);
        } else {
            ops::label_to_number(w, id, from, 1042679, ""); // a bounty board with no bounties posted.
        }
    }

    fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
        if !bulletin::check_range(w, id, from) {
            local_overhead_message_number(
                w,
                from,
                muo_primitives::MessageType::REGULAR,
                0x3B2,
                1019045,
                "",
            ); // I can't reach that.
            return;
        }
        let Some(ns) = props_b::net_state(w, from) else {
            return;
        };
        bulletin::send_bb_display_board(w, ns, id);
        if w.cannot_send_packets(ns) {
            return;
        }
        let grid = w
            .net_state(ns)
            .is_some_and(|d| d.caps.container_grid_lines());
        let players: Vec<u32> = get_active_bounties(w)
            .into_iter()
            .map(|(p, _)| p.0.value())
            .collect();
        let serial = w.item(id).map_or(0, |i| i.serial.value());
        let pk = bounty_container_content_packet(serial, &players, grid);
        w.send(ns, &pk);
    }

    /// `Server.Items.BountyBoard` `[SerializationGenerator(0, false)]`; `BoardName = "bounty board"`.
    pub static BOUNTY_BOARD: ItemType = ItemType {
        full_name: "Server.Items.BountyBoard",
        parent: Some(&BASE_BULLETIN_BOARD),
        construct: Some(|w| {
            let id = bulletin::base_board_new(w, &BOUNTY_BOARD, 0x1E5E);
            if let Some(d) = w
                .item_mut(id)
                .and_then(|i| i.ext.get_mut::<bulletin::BoardData>())
            {
                d.board_name = Some("bounty board".to_owned());
            }
            id
        }),
        family: Some(&BOUNTY_BOARD_CLASS),
        on_single_click,
        on_double_click,
        serialize: |w, id, out| {
            (BASE_BULLETIN_BOARD.serialize)(w, id, out);
            out.write_i32(0); // version (SerializationGenerator(0, false)): an int
        },
        deserialize: |w, id, r| {
            (BASE_BULLETIN_BOARD.deserialize)(w, id, r)?;
            r.read_i32()?;
            Ok(())
        },
        ..BASE_BULLETIN_BOARD
    };
}

pub use bounty_board::BOUNTY_BOARD;

/// Registers the `BountyBoard` class.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    types.register_item(&BOUNTY_BOARD, &[]);
}
