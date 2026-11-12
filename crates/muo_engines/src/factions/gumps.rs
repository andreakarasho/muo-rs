//! The faction gumps: `FactionGump` (button ids, `Exists`), `JoinStoneGump`, `FactionStoneGump`,
//! `LeaveFactionGump`, `ElectionGump`, `VoteGump`, `ElectionManagementGump`, `TownStoneGump`, `FinanceGump`,
//! `SheriffGump`, `HorseBreederGump` and `FactionImbueGump`.
//!
//! A gump holds the data its layout was drawn from; a response re-reads the world through those fields.

use muo_content::gumps::legacy::LegacyLayout;
use muo_content::gumps::{
    GumpButtonType, LegacyGumpLogic, LegacyGumpObj, RelayInfo, send_gump_to_mobile,
};
use muo_primitives::html;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::container;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages as mm;
use muo_server::net_state::NetStateId;
use muo_server::target::{self, TargetFlags, Targeted};

use muo_content::items::gold::n0;

use super::state::{Election, ElectionState, Voter};
use super::tables::{FACTION_DEFS, TOWN_DEFS, VENDOR_DEFS};
use super::{find, player_state, st, st_mut, town};

const FACTION_GUMPS: [&str; 10] = [
    "Server.Factions.JoinStoneGump",
    "Server.Factions.FactionStoneGump",
    "Server.Factions.LeaveFactionGump",
    "Server.Factions.ElectionGump",
    "Server.Factions.VoteGump",
    "Server.Factions.TownStoneGump",
    "Server.Factions.FinanceGump",
    "Server.Factions.SheriffGump",
    "Server.Factions.HorseBreederGump",
    "Server.Factions.FactionImbueGump",
];

/// `FactionGump` as a base class of the gump named `type_name`.
pub fn is_faction_gump(type_name: &str) -> bool {
    FACTION_GUMPS.contains(&type_name)
}

fn msg(w: &mut World, m: MobileId, n: i32) {
    mm::send_localized_message(w, m, n, "", 0x3B2);
}

/// `ToButtonID(type, index)` with `ButtonTypes`.
fn to_button_id(button_types: i32, ty: i32, index: i32) -> i32 {
    1 + index * button_types + ty
}

/// `FromButtonID(buttonID, out type, out index)`.
fn from_button_id(button_types: i32, button_id: i32) -> Option<(i32, i32)> {
    let offset = button_id - 1;
    (offset >= 0).then(|| (offset % button_types, offset / button_types))
}

/// `AddHtmlLocalized(x, y, w, h, number)`.
fn hl(l: &mut LegacyLayout, x: i32, y: i32, w: i32, h: i32, number: i32) {
    l.add_html_localized(x, y, w, h, number, false, false);
}

/// `AddHtmlText(x, y, w, h, text, back, scroll)` of a definition that is a cliloc number.
#[allow(clippy::too_many_arguments)] // one parameter per field of the call
fn ht(l: &mut LegacyLayout, x: i32, y: i32, w: i32, h: i32, number: i32, back: bool, scroll: bool) {
    if number > 0 {
        l.add_html_localized(x, y, w, h, number, back, scroll);
    }
}

/// `AddHtml(x, y, w, h, text)`.
fn hx(l: &mut LegacyLayout, x: i32, y: i32, w: i32, h: i32, text: &str) {
    l.add_html(x, y, w, h, text, false, false);
}

fn name_of(w: &World, m: MobileId) -> String {
    muo_server::mobile::base::name(w, m).unwrap_or_default()
}

fn send<T: LegacyGumpLogic>(w: &mut World, to: MobileId, g: LegacyGumpObj<T>) {
    send_gump_to_mobile(w, to, Box::new(g), false);
}

fn tithe_row(l: &mut LegacyLayout, x: i32, y: i32, w_: i32, tithe: i32) {
    if (0..=100).contains(&tithe) && tithe % 10 == 0 {
        hl(l, x, y, w_, 20, 1011480 + tithe / 10);
    } else {
        hx(l, x, y, w_, 20, &format!("{tithe}%"));
    }
}

// ---- JoinStoneGump ----

pub struct JoinStoneGump {
    from: MobileId,
    faction: usize,
}

impl LegacyGumpLogic for JoinStoneGump {
    const TYPE_NAME: &'static str = "Server.Factions.JoinStoneGump";
    fn on_response(&mut self, w: &mut World, _s: NetStateId, info: &RelayInfo) {
        if info.button_id == 1 {
            super::flow::on_join_accepted(w, self.faction, self.from);
        }
    }
}

pub fn send_join_stone_gump(w: &mut World, from: MobileId, f: usize) {
    let mut g = LegacyGumpObj::new(20, 30, JoinStoneGump { from, faction: f });
    let l = &mut g.layout;
    let def = FACTION_DEFS[f];
    let (commander, tithe) = st(w).map_or((None, 0), |s| {
        (s.factions[f].commander, s.factions[f].tithe)
    });
    l.add_page(0);
    l.add_background(0, 0, 550, 440, 5054);
    l.add_background(10, 10, 530, 420, 3000);
    ht(l, 20, 30, 510, 20, def.header, false, false);
    ht(l, 20, 130, 510, 100, def.about, true, true);
    hl(l, 20, 60, 100, 20, 1011429); // Led By :
    hx(
        l,
        125,
        60,
        200,
        20,
        &commander.map_or_else(|| "Nobody".to_owned(), |c| name_of(w, c)),
    );
    hl(l, 20, 80, 100, 20, 1011457); // Tithe rate :
    tithe_row(l, 125, 80, 350, tithe);
    l.add_button(20, 400, 4005, 4007, 1);
    hl(l, 55, 400, 200, 20, 1011425); // JOIN THIS FACTION
    l.add_button(300, 400, 4005, 4007, 0);
    hl(l, 335, 400, 200, 20, 1011012); // CANCEL
    send(w, from, g);
}

// ---- FactionStoneGump ----

pub struct FactionStoneGump {
    from: MobileId,
    faction: usize,
}

const FS_TYPES: i32 = 4;

/// `MerchantTitles.Info`: skill, requirement, title, label, assigned.
pub const MERCHANT_INFO: [(muo_server::skills::SkillName, f64, i32, i32, i32); 6] = {
    use muo_server::skills::SkillName::*;
    [
        (Inscribe, 90.0, 1060773, 1011468, 1010121),
        (Carpentry, 90.0, 1060774, 1011469, 1010122),
        (Tinkering, 90.0, 1022984, 1011470, 1010123),
        (Blacksmith, 90.0, 1023016, 1011471, 1010124),
        (Fletching, 90.0, 1023022, 1011472, 1010125),
        (Tailoring, 90.0, 1022982, 1018300, 1042162),
    ]
};

fn is_qualified(w: &mut World, m: MobileId, i: usize) -> bool {
    let (skill, req, ..) = MERCHANT_INFO[i];
    muo_server::skills::value(w, m, skill) >= req
}

pub fn send_faction_stone_gump(w: &mut World, from: MobileId, f: usize) {
    let mut g = LegacyGumpObj::new(20, 30, FactionStoneGump { from, faction: f });
    let l = &mut g.layout;
    let def = FACTION_DEFS[f];
    let (commander, tithe, silver, traps) = {
        let d = &st(w).expect("state").factions[f];
        (d.commander, d.tithe, d.silver, d.traps.len())
    };
    let pl = player_state(w, from).map(|(_, p)| {
        (
            p.merchant_title,
            p.kill_points,
            FACTION_DEFS[p.faction].ranks[p.rank].rank,
        )
    });
    let commander_opts = super::is_commander(w, f, from);
    let quals: Vec<bool> = (0..MERCHANT_INFO.len())
        .map(|i| is_qualified(w, from, i))
        .collect();
    let qualified = quals.iter().any(|&q| q);
    let owners: Vec<Option<usize>> = (0..TOWN_DEFS.len()).map(|t| town::owner(w, t)).collect();
    let purifying: Vec<bool> = (0..TOWN_DEFS.len())
        .map(|t| {
            town::monolith(w, t)
                .and_then(|m| super::items::monolith_sigil(w, m))
                .is_some_and(|sg| super::items::is_purifying(w, sg))
        })
        .collect();

    l.add_page(0);
    l.add_background(0, 0, 550, 440, 5054);
    l.add_background(10, 10, 530, 420, 3000);

    l.add_page(1);
    ht(l, 20, 30, 510, 20, def.header, false, false);
    hl(l, 20, 60, 100, 20, 1011429); // Led By :
    hx(
        l,
        125,
        60,
        200,
        20,
        &commander.map_or_else(|| "Nobody".to_owned(), |c| name_of(w, c)),
    );
    hl(l, 20, 80, 100, 20, 1011457); // Tithe rate :
    tithe_row(l, 125, 80, 350, tithe);
    hl(l, 20, 100, 100, 20, 1011458); // Traps placed :
    hx(l, 125, 100, 50, 20, &traps.to_string());

    hl(l, 55, 225, 200, 20, 1011428); // VOTE FOR LEADERSHIP
    l.add_button(20, 225, 4005, 4007, to_button_id(FS_TYPES, 0, 0));
    hl(l, 55, 150, 100, 20, 1011430); // CITY STATUS
    l.add_button_type(20, 150, 4005, 4007, 0, GumpButtonType::Page, 2);
    hl(l, 55, 175, 100, 20, 1011444); // STATISTICS
    l.add_button_type(20, 175, 4005, 4007, 0, GumpButtonType::Page, 4);

    let has_title = pl.is_some_and(|(t, ..)| t != 0);
    if has_title {
        hl(l, 55, 200, 250, 20, 1011460); // UNDECLARE FACTION MERCHANT
        l.add_button(20, 200, 4005, 4007, to_button_id(FS_TYPES, 1, 0));
    } else if qualified {
        hl(l, 55, 200, 250, 20, 1011459); // DECLARE FACTION MERCHANT
        l.add_button_type(20, 200, 4005, 4007, 0, GumpButtonType::Page, 5);
    } else {
        hl(l, 55, 200, 250, 20, 1011467); // MERCHANT OPTIONS
        l.add_image(20, 200, 4020, 0);
    }

    hl(l, 55, 250, 300, 20, 1011461); // COMMANDER OPTIONS
    if commander_opts {
        l.add_button_type(20, 250, 4005, 4007, 0, GumpButtonType::Page, 6);
    } else {
        l.add_image(20, 250, 4020, 0);
    }
    hl(l, 55, 275, 300, 20, 1011426); // LEAVE THIS FACTION
    l.add_button(20, 275, 4005, 4007, to_button_id(FS_TYPES, 0, 1));
    hl(l, 55, 300, 200, 20, 1011441); // EXIT
    l.add_button(20, 300, 4005, 4007, 0);

    l.add_page(2);
    hl(l, 20, 30, 250, 20, 1011430); // CITY STATUS
    for (i, t) in TOWN_DEFS.iter().enumerate() {
        let i = i as i32;
        ht(l, 40, 55 + i * 30, 150, 20, t.town_name, false, false);
        match owners[i as usize] {
            None => hl(l, 200, 55 + i * 30, 150, 20, 1011462), // : Neutral
            Some(o) => {
                hl(l, 200, 55 + i * 30, 150, 20, FACTION_DEFS[o].owner_label);
                l.add_image(
                    20,
                    60 + i * 30,
                    if purifying[i as usize] { 0x938 } else { 0x939 },
                    0,
                );
            }
        }
    }
    l.add_image(20, 300, 2361, 0);
    hl(l, 45, 295, 300, 20, 1011491); // sigil may be recaptured
    l.add_image(20, 320, 2360, 0);
    hl(l, 45, 315, 300, 20, 1011492); // sigil may not be recaptured
    hl(l, 55, 350, 100, 20, 1011447); // BACK
    l.add_button_type(20, 350, 4005, 4007, 0, GumpButtonType::Page, 1);

    l.add_page(4);
    hl(l, 20, 30, 150, 20, 1011444); // STATISTICS
    hl(l, 20, 100, 100, 20, 1011445); // Name :
    hx(l, 120, 100, 150, 20, &name_of(w, from));
    hl(l, 20, 130, 100, 20, 1018064); // score :
    hx(
        l,
        120,
        130,
        100,
        20,
        &pl.map_or(0, |(_, k, _)| k).to_string(),
    );
    hl(l, 20, 160, 100, 20, 1011446); // Rank :
    hx(
        l,
        120,
        160,
        100,
        20,
        &pl.map_or(0, |(_, _, r)| r).to_string(),
    );
    hl(l, 55, 250, 100, 20, 1011447); // BACK
    l.add_button_type(20, 250, 4005, 4007, 0, GumpButtonType::Page, 1);

    if (pl.is_none() || !has_title) && qualified {
        l.add_page(5);
        hl(l, 20, 30, 250, 20, 1011467); // MERCHANT OPTIONS
        hl(l, 20, 80, 300, 20, 1011473); // Select the title you wish to display
        for (i, info) in MERCHANT_INFO.iter().enumerate() {
            let y = 100 + i as i32 * 30;
            if quals[i] {
                l.add_button(20, y, 4005, 4007, to_button_id(FS_TYPES, 1, i as i32 + 1));
            } else {
                l.add_image(20, y, 4020, 0);
            }
            ht(l, 55, y, 200, 20, info.3, false, false);
        }
        hl(l, 55, 340, 100, 20, 1011447); // BACK
        l.add_button_type(20, 340, 4005, 4007, 0, GumpButtonType::Page, 1);
    }

    if commander_opts {
        l.add_page(6);
        hl(l, 20, 30, 200, 20, 1011461); // COMMANDER OPTIONS
        hl(l, 20, 70, 120, 20, 1011457); // Tithe rate :
        tithe_row(l, 140, 70, 250, tithe);
        hl(l, 20, 100, 120, 20, 1011474); // Silver available :
        hx(l, 140, 100, 50, 20, &n0(silver as i64));
        hl(l, 55, 130, 200, 20, 1011478); // CHANGE TITHE RATE
        l.add_button_type(20, 130, 4005, 4007, 0, GumpButtonType::Page, 8);
        hl(l, 55, 160, 200, 20, 1018301); // TRANSFER SILVER
        if silver >= 10000 {
            l.add_button_type(20, 160, 4005, 4007, 0, GumpButtonType::Page, 7);
        } else {
            l.add_image(20, 160, 4020, 0);
        }
        hl(l, 55, 310, 100, 20, 1011447); // BACK
        l.add_button_type(20, 310, 4005, 4007, 0, GumpButtonType::Page, 1);

        if silver >= 10000 {
            l.add_page(7);
            hl(l, 20, 30, 250, 20, 1011476); // TOWN FINANCE
            hl(l, 20, 50, 400, 20, 1011477); // Select a town to transfer 10000 silver to
            for (i, t) in TOWN_DEFS.iter().enumerate() {
                let y = 75 + i as i32 * 30;
                ht(l, 55, y, 200, 20, t.town_name, false, false);
                if owners[i] == Some(f) {
                    l.add_button(20, y, 4005, 4007, to_button_id(FS_TYPES, 2, i as i32));
                } else {
                    l.add_image(20, y, 4020, 0);
                }
            }
            hl(l, 55, 310, 100, 20, 1011447); // BACK
            l.add_button_type(20, 310, 4005, 4007, 0, GumpButtonType::Page, 1);
        }

        l.add_page(8);
        hl(l, 20, 30, 400, 20, 1011479); // Select the % for the new tithe rate
        let mut y = 55;
        for i in 0..=10 {
            if i == 5 {
                y += 5;
            }
            hl(l, 55, y, 300, 20, 1011480 + i);
            l.add_button(20, y, 4005, 4007, to_button_id(FS_TYPES, 3, i));
            y += 20;
            if i == 5 {
                y += 5;
            }
        }
        hl(l, 55, 310, 300, 20, 1011447); // BACK
        l.add_button_type(20, 310, 4005, 4007, 0, GumpButtonType::Page, 1);
    }
    send(w, from, g);
}

impl LegacyGumpLogic for FactionStoneGump {
    const TYPE_NAME: &'static str = "Server.Factions.FactionStoneGump";
    fn on_response(&mut self, w: &mut World, _s: NetStateId, info: &RelayInfo) {
        let (from, f) = (self.from, self.faction);
        let Some((ty, index)) = from_button_id(FS_TYPES, info.button_id) else {
            return;
        };
        match ty {
            0 => match index {
                0 => send_election_gump(w, from, f),
                1 => send_leave_gump(w, from, f),
                _ => {}
            },
            1 => {
                if index >= 0 && index <= MERCHANT_INFO.len() as i32 {
                    let idx = index;
                    // `MerchantTitles.GetInfo(newTitle)`: `Info[index - 1]`.
                    if idx == 0 {
                        msg(w, from, 1010120); // Your merchant title has been removed
                        set_merchant_title(w, from, idx);
                    } else if is_qualified(w, from, idx as usize - 1) {
                        msg(w, from, MERCHANT_INFO[idx as usize - 1].4);
                        set_merchant_title(w, from, idx);
                    }
                }
            }
            2 => {
                if !super::is_commander(w, f, from) {
                    return;
                }
                if (0..TOWN_DEFS.len() as i32).contains(&index) {
                    let t = index as usize;
                    if town::owner(w, t) == Some(f)
                        && st(w).is_some_and(|s| s.factions[f].silver >= 10000)
                    {
                        let s = st_mut(w);
                        s.factions[f].silver -= 10000;
                        s.towns[t].silver += 10000;
                        // 10k in silver has been received by:
                        mm::send_localized_message_affix(
                            w,
                            from,
                            1042726,
                            true,
                            &format!(" {}", TOWN_DEFS[t].friendly_name),
                            "",
                            0x3B2,
                        );
                    }
                }
            }
            3 => {
                if !super::is_commander(w, f, from) {
                    return;
                }
                if (0..=10).contains(&index) {
                    st_mut(w).factions[f].tithe = index * 10;
                }
            }
            _ => {}
        }
    }
}

fn set_merchant_title(w: &mut World, m: MobileId, title: i32) {
    if let Some(f) = find(w, m)
        && let Some(p) = st_mut(w).factions[f]
            .members
            .iter_mut()
            .find(|p| p.mobile == m)
    {
        p.merchant_title = title;
        super::invalidate_player(w, m);
    }
}

// ---- LeaveFactionGump ----

pub struct LeaveFactionGump {
    from: MobileId,
}

pub fn send_leave_gump(w: &mut World, from: MobileId, _f: usize) {
    let mut g = LegacyGumpObj::new(20, 30, LeaveFactionGump { from });
    let l = &mut g.layout;
    l.add_background(0, 0, 270, 120, 5054);
    l.add_background(10, 10, 250, 100, 3000);
    let leader = super::flow::guild_leader_of(w, from) == Some(from);
    if leader {
        // Are you sure you want your entire guild to leave this faction?
        l.add_html_localized(20, 15, 230, 60, 1018057, true, true);
    } else {
        // Are you sure you want to leave this faction?
        l.add_html_localized(20, 15, 230, 60, 1018063, true, true);
    }
    hl(l, 55, 80, 75, 20, 1011011); // CONTINUE
    l.add_button(20, 80, 4005, 4007, 1);
    hl(l, 170, 80, 75, 20, 1011012); // CANCEL
    l.add_button(135, 80, 4005, 4007, 2);
    send(w, from, g);
}

impl LegacyGumpLogic for LeaveFactionGump {
    const TYPE_NAME: &'static str = "Server.Factions.LeaveFactionGump";
    fn on_response(&mut self, w: &mut World, _s: NetStateId, info: &RelayInfo) {
        let from = self.from;
        match info.button_id {
            1 => super::flow::leave_faction(w, from),
            2 => msg(w, from, 500737), // Canceled resignation.
            _ => {}
        }
    }
}

// ---- ElectionGump ----

pub struct ElectionGump {
    from: MobileId,
    faction: usize,
}

pub fn send_election_gump(w: &mut World, from: MobileId, f: usize) {
    let mut g = LegacyGumpObj::new(50, 50, ElectionGump { from, faction: f });
    let l = &mut g.layout;
    let e = &st(w).expect("state").factions[f].election;
    let now = w.now;
    l.add_page(0);
    l.add_background(0, 0, 420, 180, 5054);
    l.add_background(10, 10, 400, 160, 3000);
    ht(l, 20, 20, 380, 20, FACTION_DEFS[f].header, false, false);
    match e.state {
        ElectionState::Pending => {
            let to_go = e.last_state_time + Election::PENDING_PERIOD - now;
            let days = (to_go.total_days() + 0.5) as i32;
            hl(l, 20, 40, 380, 20, 1038034); // A new election campaign is pending
            if days > 0 {
                hl(l, 20, 60, 280, 20, 1018062); // Days until next election :
                l.add_label(300, 60, 0, &days.to_string());
            } else {
                hl(l, 20, 60, 280, 20, 1018059); // Election campaigning begins tonight.
            }
        }
        ElectionState::Campaign => {
            let to_go = e.last_state_time + Election::CAMPAIGN_PERIOD - now;
            let days = (to_go.total_days() + 0.5) as i32;
            hl(l, 20, 40, 380, 20, 1018058); // There is an election campaign in progress.
            if days > 0 {
                hl(l, 20, 60, 280, 20, 1038033); // Days to go:
                l.add_label(300, 60, 0, &days.to_string());
            } else {
                hl(l, 20, 60, 280, 20, 1018061); // Campaign in progress. Voting begins tonight.
            }
            if can_be_candidate(w, f, from) {
                l.add_button(20, 110, 4005, 4007, 2);
                hl(l, 55, 110, 350, 20, 1011427); // CAMPAIGN FOR LEADERSHIP
            } else {
                let rank = player_state(w, from).map(|(pf, p)| FACTION_DEFS[pf].ranks[p.rank].rank);
                if rank.is_none_or(|r| r < Election::CANDIDATE_RANK) {
                    hl(l, 20, 100, 380, 20, 1010118); // You must have a higher rank to run for office
                }
            }
        }
        ElectionState::Election => {
            let to_go = e.last_state_time + Election::VOTING_PERIOD - now;
            let days = to_go.total_days().ceil() as i32;
            hl(l, 20, 40, 380, 20, 1018060); // There is an election vote in progress.
            hl(l, 20, 60, 280, 20, 1038033);
            l.add_label(300, 60, 0, &days.to_string());
            hl(l, 55, 100, 380, 20, 1011428); // VOTE FOR LEADERSHIP
            l.add_button(20, 100, 4005, 4007, 1);
        }
    }
    l.add_button(20, 140, 4005, 4007, 0);
    hl(l, 55, 140, 350, 20, 1011012); // CANCEL
    send(w, from, g);
}

/// `Election.CanBeCandidate(mob)`.
pub fn can_be_candidate(w: &World, f: usize, mob: MobileId) -> bool {
    let e = &st(w).expect("state").factions[f].election;
    if e.is_candidate(mob)
        || e.candidates.len() >= Election::MAX_CANDIDATES
        || e.state != ElectionState::Campaign
    {
        return false;
    }
    player_state(w, mob).is_some_and(|(pf, p)| {
        pf == f && FACTION_DEFS[pf].ranks[p.rank].rank >= Election::CANDIDATE_RANK
    })
}

impl LegacyGumpLogic for ElectionGump {
    const TYPE_NAME: &'static str = "Server.Factions.ElectionGump";
    fn on_response(&mut self, w: &mut World, _s: NetStateId, info: &RelayInfo) {
        let (from, f) = (self.from, self.faction);
        match info.button_id {
            0 => send_faction_stone_gump(w, from, f),
            1 => {
                if st(w).is_some_and(|s| s.factions[f].election.state == ElectionState::Election) {
                    send_vote_gump(w, from, f);
                }
            }
            2 if can_be_candidate(w, f, from) => {
                super::election_add_candidate(w, f, from);
            }
            _ => {}
        }
    }
}

// ---- VoteGump ----

pub struct VoteGump {
    from: MobileId,
    faction: usize,
}

pub fn send_vote_gump(w: &mut World, from: MobileId, f: usize) {
    let mut g = LegacyGumpObj::new(50, 50, VoteGump { from, faction: f });
    let l = &mut g.layout;
    let e = &st(w).expect("state").factions[f].election;
    let can_vote = e.can_vote(from);
    l.add_page(0);
    l.add_background(0, 0, 420, 350, 5054);
    l.add_background(10, 10, 400, 330, 3000);
    ht(l, 20, 20, 380, 20, FACTION_DEFS[f].header, false, false);
    if can_vote {
        hl(l, 20, 60, 380, 20, 1011428); // VOTE FOR LEADERSHIP
    } else {
        hl(l, 20, 60, 380, 20, 1038032); // You have already voted in this election.
    }
    for (i, cd) in e.candidates.iter().enumerate() {
        let i = i as i32;
        if can_vote {
            l.add_button(20, 100 + i * 20, 4005, 4007, i + 1);
        }
        l.add_label(55, 100 + i * 20, 0, &name_of(w, cd.mobile));
        l.add_label(300, 100 + i * 20, 0, &cd.votes().to_string());
    }
    l.add_button(20, 310, 4005, 4007, 0);
    hl(l, 55, 310, 100, 20, 1011012); // CANCEL
    send(w, from, g);
}

impl LegacyGumpLogic for VoteGump {
    const TYPE_NAME: &'static str = "Server.Factions.VoteGump";
    fn on_response(&mut self, w: &mut World, _s: NetStateId, info: &RelayInfo) {
        let (from, f) = (self.from, self.faction);
        if info.button_id == 0 {
            send_faction_stone_gump(w, from, f);
            return;
        }
        if !st(w).is_some_and(|s| s.factions[f].election.can_vote(from)) {
            return;
        }
        let index = info.button_id - 1;
        // `Address = From.NetState != null ? NetState.Address : IPAddress.None`
        let address = w
            .mobile(from)
            .and_then(|m| m.net_state)
            .and_then(|ns| w.net_state(ns))
            .and_then(|d| d.address)
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::BROADCAST));
        let now = w.now;
        let e = &mut st_mut(w).factions[f].election;
        if index >= 0 && (index as usize) < e.candidates.len() {
            e.candidates[index as usize].voters.push(Voter {
                from,
                address,
                time: now,
            });
        }
        send_vote_gump(w, from, f);
    }
}

// ---- ElectionManagementGump ----

const LABEL_COLOR: i32 = 0xFFFFFF;

pub struct ElectionManagementGump {
    faction: usize,
    candidate: Option<MobileId>,
    page: usize,
}

fn format_time_span(ts: muo_primitives::TimeSpan) -> String {
    let secs = ts.ticks() / 10_000_000;
    let (d, h, m, s) = (secs / 86400, secs / 3600 % 24, secs / 60 % 60, secs % 60);
    format!("{d:02}:{h:02}:{m:02}:{s:02}")
}

pub fn send_election_management_gump(w: &mut World, to: MobileId, f: usize) {
    send_election_management(w, to, f, None, 0);
}

fn send_election_management(
    w: &mut World,
    to: MobileId,
    f: usize,
    candidate: Option<MobileId>,
    page: usize,
) {
    let mut g = LegacyGumpObj::new(
        40,
        40,
        ElectionManagementGump {
            faction: f,
            candidate,
            page,
        },
    );
    let l = &mut g.layout;
    let e = &st(w).expect("state").factions[f].election;
    let color = |t: &str| html::color_rgb(t, LABEL_COLOR);
    let center = |t: &str| html::center_rgb(t, LABEL_COLOR);
    l.add_page(0);
    let cd = candidate
        .and_then(|c| e.find_candidate(c))
        .map(|i| &e.candidates[i]);
    if let Some(cd) = cd {
        l.add_background(0, 0, 448, 354, 9270);
        l.add_alpha_region(10, 10, 428, 334);
        hx(l, 10, 10, 428, 20, &center("Candidate Management"));
        hx(l, 45, 35, 100, 20, &color("Player Name:"));
        hx(l, 145, 35, 100, 20, &color(&name_of(w, cd.mobile)));
        hx(l, 45, 55, 100, 20, &color("Vote Count:"));
        hx(l, 145, 55, 100, 20, &color(&cd.votes().to_string()));
        l.add_button(12, 73, 4005, 4007, 1);
        hx(l, 45, 75, 100, 20, &color("Drop Candidate"));
        l.add_image_tiled(13, 99, 422, 242, 9264);
        l.add_image_tiled(14, 100, 420, 240, 9274);
        l.add_alpha_region(14, 100, 420, 240);
        hx(l, 14, 100, 420, 20, &center("Voters"));
        if page > 0 {
            l.add_button(397, 104, 0x15E3, 0x15E7, 2);
        } else {
            l.add_image(397, 104, 0x25EA, 0);
        }
        if (page + 1) * 10 < cd.voters.len() {
            l.add_button(414, 104, 0x15E1, 0x15E5, 3);
        } else {
            l.add_image(414, 104, 0x25E6, 0);
        }
        hx(l, 14, 120, 30, 20, &center("DEL"));
        hx(l, 47, 120, 150, 20, &color("Name"));
        hx(l, 195, 120, 100, 20, &center("Address"));
        hx(l, 295, 120, 80, 20, &center("Time"));
        hx(l, 355, 120, 60, 20, &center("Legit"));
        let mut idx = 0;
        let mut i = page * 10;
        while i < cd.voters.len() && i < (page + 1) * 10 {
            let voter = &cd.voters[i];
            let y = 140 + idx * 20;
            l.add_button(13, 138 + idx * 20, 4002, 4004, 4 + i as i32);
            hx(l, 47, y, 150, 20, &color(&name_of(w, voter.from)));
            hx(l, 145 + 50, y, 100, 20, &center(&voter.address.to_string()));
            hx(
                l,
                295,
                y,
                80,
                20,
                &center(&format_time_span(voter.time - e.last_state_time)),
            );
            hx(
                l,
                355,
                y,
                60,
                20,
                &center(&format!("{}%", voter_legitimacy(w, voter))),
            );
            i += 1;
            idx += 1;
        }
    } else {
        l.add_background(0, 0, 288, 334, 9270);
        l.add_alpha_region(10, 10, 268, 314);
        hx(l, 10, 10, 268, 20, &center("Election Management"));
        hx(l, 45, 35, 100, 20, &color("Current State:"));
        hx(l, 145, 35, 100, 20, &color(e.state.name()));
        l.add_button(12, 53, 4005, 4007, 1);
        hx(l, 45, 55, 100, 20, &color("Transition Time:"));
        hx(
            l,
            145,
            55,
            100,
            20,
            &color(&format_time_span(e.next_state_time(w.now))),
        );
        l.add_image_tiled(13, 79, 262, 242, 9264);
        l.add_image_tiled(14, 80, 260, 240, 9274);
        l.add_alpha_region(14, 80, 260, 240);
        hx(l, 14, 80, 260, 20, &center("Candidates"));
        hx(l, 14, 100, 30, 20, &center("-->"));
        hx(l, 47, 100, 150, 20, &color("Name"));
        hx(l, 195, 100, 80, 20, &center("Votes"));
        for (i, c) in e.candidates.iter().enumerate() {
            let i = i as i32;
            l.add_button(13, 118 + i * 20, 4005, 4007, 2 + i);
            hx(l, 47, 120 + i * 20, 150, 20, &color(&name_of(w, c.mobile)));
            hx(
                l,
                195,
                120 + i * 20,
                80,
                20,
                &center(&c.votes().to_string()),
            );
        }
    }
    send(w, to, g);
}

/// `Voter.AcquireFields()[3]`: how much the vote is believed.
fn voter_legitimacy(w: &World, v: &Voter) -> i32 {
    let game_time = muo_content::player_mobile::game_time(w, v.from);
    let kp = player_state(w, v.from).map_or(0, |(_, p)| p.kill_points);
    let sk = muo_server::skills::total(w, v.from);
    let factor_skills = 50 + sk * 100 / 10000;
    let factor_kill_pts = 100 + kp * 2;
    let factor_game_time =
        50 + (game_time.ticks() * 100 / muo_primitives::TimeSpan::from_days(1).ticks()) as i32;
    (factor_skills * factor_kill_pts * factor_game_time.max(100) / 10000).clamp(0, 100)
}

impl LegacyGumpLogic for ElectionManagementGump {
    const TYPE_NAME: &'static str = "Server.Factions.ElectionManagementGump";
    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_state(sender).and_then(|d| d.mobile) else {
            return;
        };
        let f = self.faction;
        let bid = info.button_id;
        let ncand = st(w).map_or(0, |s| s.factions[f].election.candidates.len());
        match self.candidate {
            None => {
                if bid > 1 {
                    let i = (bid - 2) as usize;
                    if i < ncand {
                        let m = st(w).map(|s| s.factions[f].election.candidates[i].mobile);
                        send_election_management(w, from, f, m, 0);
                    }
                }
            }
            Some(c) => {
                let nvoters = st(w)
                    .and_then(|s| {
                        s.factions[f]
                            .election
                            .find_candidate(c)
                            .map(|i| s.factions[f].election.candidates[i].voters.len())
                    })
                    .unwrap_or(0);
                if bid == 0 {
                    send_election_management(w, from, f, None, 0);
                } else if bid == 1 {
                    super::election_remove_candidate(w, f, c);
                    send_election_management(w, from, f, None, 0);
                } else if bid == 2 && self.page > 0 {
                    send_election_management(w, from, f, Some(c), self.page - 1);
                } else if bid == 3 && (self.page + 1) * 10 < nvoters {
                    send_election_management(w, from, f, Some(c), self.page + 1);
                } else {
                    let i = bid - 4;
                    if i >= 0 && (i as usize) < nvoters {
                        let e = &mut st_mut(w).factions[f].election;
                        if let Some(ci) = e.find_candidate(c) {
                            e.candidates[ci].voters.remove(i as usize);
                        }
                        send_election_management(w, from, f, Some(c), self.page);
                    }
                }
            }
        }
    }
}

// ---- TownStoneGump ----

pub struct TownStoneGump {
    from: MobileId,
    faction: usize,
    town: usize,
}

pub fn send_town_stone_gump(w: &mut World, from: MobileId, f: usize, t: usize) {
    let mut g = LegacyGumpObj::new(
        50,
        50,
        TownStoneGump {
            from,
            faction: f,
            town: t,
        },
    );
    let l = &mut g.layout;
    l.add_page(0);
    l.add_background(0, 0, 320, 250, 5054);
    l.add_background(10, 10, 300, 230, 3000);
    ht(
        l,
        25,
        30,
        250,
        25,
        TOWN_DEFS[t].town_stone_header,
        false,
        false,
    );
    hl(l, 55, 60, 150, 25, 1011557); // Hire Sheriff
    l.add_button(20, 60, 4005, 4007, 1);
    hl(l, 55, 90, 150, 25, 1011559); // Hire Finance Minister
    l.add_button(20, 90, 4005, 4007, 2);
    hl(l, 55, 120, 150, 25, 1011558); // Fire Sheriff
    l.add_button(20, 120, 4005, 4007, 3);
    hl(l, 55, 150, 150, 25, 1011560); // Fire Finance Minister
    l.add_button(20, 150, 4005, 4007, 4);
    hl(l, 55, 210, 150, 25, 1011441); // EXIT
    l.add_button(20, 210, 4005, 4007, 0);
    send(w, from, g);
}

/// `HireSheriff_OnTarget` / `HireFinanceMinister_OnTarget`.
fn hire(w: &mut World, f: usize, t: usize, from: MobileId, obj: Targeted, sheriff: bool) {
    if town::owner(w, t) != Some(f) || !super::is_commander(w, f, from) {
        msg(w, from, 1010339); // You no longer control this city
        return;
    }
    let current = st(w).and_then(|s| {
        if sheriff {
            s.towns[t].sheriff
        } else {
            s.towns[t].finance
        }
    });
    if current.is_some() {
        msg(w, from, 1010342); // You must fire your Sheriff before you can elect a new one
        return;
    }
    let Targeted::Entity(EntityId::Mobile(m)) = obj else {
        msg(w, from, 1010334); // You must select a player to hold a city position!
        return;
    };
    match player_state(w, m) {
        None => msg(w, from, 1010337), // You must pick someone in a faction
        Some((pf, _)) if pf != f => msg(w, from, 1010338), // You must pick someone in the correct faction
        Some(_) if st(w).is_some_and(|s| s.factions[f].commander == Some(m)) => {
            msg(w, from, 1010335)
        }
        Some((_, p)) if p.sheriff.is_some() || p.finance.is_some() => msg(w, from, 1005245),
        Some(_) => {
            if sheriff {
                super::set_town_sheriff(w, t, Some(m));
                msg(w, m, 1010340); // You are now the Sheriff
                msg(w, from, 1010341); // You have elected a Sheriff
            } else {
                super::set_town_finance(w, t, Some(m));
                msg(w, m, 1010343); // You are now the Financial Minister
                msg(w, from, 1010344); // You have elected a Financial Minister
            }
        }
    }
}

impl LegacyGumpLogic for TownStoneGump {
    const TYPE_NAME: &'static str = "Server.Factions.TownStoneGump";
    fn on_response(&mut self, w: &mut World, _s: NetStateId, info: &RelayInfo) {
        let (from, f, t) = (self.from, self.faction, self.town);
        if town::owner(w, t) != Some(f) || !super::is_commander(w, f, from) {
            msg(w, from, 1010339); // You no longer control this city
            return;
        }
        let (sheriff, finance) =
            st(w).map_or((None, None), |s| (s.towns[t].sheriff, s.towns[t].finance));
        match info.button_id {
            1 => {
                if sheriff.is_some() {
                    msg(w, from, 1010342);
                } else {
                    msg(w, from, 1010347); // Who shall be your new sheriff
                    target::begin_target(
                        w,
                        from,
                        12,
                        false,
                        TargetFlags::NONE,
                        move |w, from, obj| hire(w, f, t, from, obj, true),
                    );
                }
            }
            2 => {
                if finance.is_some() {
                    msg(w, from, 1010345);
                } else {
                    msg(w, from, 1010348); // Who shall be your new Minister of Finances?
                    target::begin_target(
                        w,
                        from,
                        12,
                        false,
                        TargetFlags::NONE,
                        move |w, from, obj| hire(w, f, t, from, obj, false),
                    );
                }
            }
            3 => match sheriff {
                None => msg(w, from, 1010350),
                Some(s) => {
                    msg(w, from, 1010349); // You have fired your sheriff
                    msg(w, s, 1010270); // You have been fired as Sheriff
                    super::set_town_sheriff(w, t, None);
                }
            },
            4 => match finance {
                None => msg(w, from, 1010352),
                Some(s) => {
                    msg(w, from, 1010351); // You have fired your financial Minister
                    msg(w, s, 1010151); // You have been fired as Finance Minister
                    super::set_town_finance(w, t, None);
                }
            },
            _ => {}
        }
    }
}

// ---- FinanceGump / SheriffGump ----

const PRICE_OFFSETS: [i32; 12] = [-30, -25, -20, -15, -10, -5, 50, 100, 150, 200, 250, 300];

pub struct FinanceGump {
    from: MobileId,
    faction: usize,
    town: usize,
}

fn finance_statement(w: &World, l: &mut LegacyLayout, t: usize, with_n0_silver: bool) {
    let s = st(w).expect("state");
    let (fu, su, di, nc) = (
        town::finance_upkeep(w, t),
        town::sheriff_upkeep(w, t),
        town::daily_income(w, t),
        town::net_cash_flow(w, t),
    );
    hl(l, 20, 30, 300, 25, 1011524); // FINANCE STATEMENT
    hl(l, 20, 80, 300, 25, 1011538); // Current total money for town :
    let silver = s.towns[t].silver;
    l.add_label(
        20,
        100,
        0x44,
        &if with_n0_silver {
            n0(silver as i64)
        } else {
            silver.to_string()
        },
    );
    hl(l, 20, 130, 300, 25, 1011520); // Finance Minister Upkeep :
    l.add_label(20, 150, 0x44, &n0(fu as i64));
    hl(l, 20, 180, 300, 25, 1011521); // Sheriff Upkeep :
    l.add_label(20, 200, 0x44, &n0(su as i64));
    hl(l, 20, 230, 300, 25, 1011522); // Town Income :
    l.add_label(20, 250, 0x44, &n0(di as i64));
    hl(l, 20, 280, 300, 25, 1011523); // Net Cash flow per day :
    l.add_label(20, 300, 0x44, &n0(nc as i64));
    hl(l, 55, 360, 200, 25, 1011067); // Previous page
    l.add_button_type(20, 360, 4005, 4007, 0, GumpButtonType::Page, 1);
}

pub fn send_finance_gump(w: &mut World, from: MobileId, f: usize, t: usize) {
    let mut g = LegacyGumpObj::new(
        50,
        50,
        FinanceGump {
            from,
            faction: f,
            town: t,
        },
    );
    let l = &mut g.layout;
    let s = st(w).expect("state");
    let tax = s.towns[t].tax;
    l.add_page(0);
    l.add_background(0, 0, 320, 410, 5054);
    l.add_background(10, 10, 300, 390, 3000);
    l.add_page(1);
    hl(l, 20, 30, 260, 25, 1011541); // FINANCE MINISTER
    hl(l, 55, 90, 200, 25, 1011539); // CHANGE PRICES
    l.add_button_type(20, 90, 4005, 4007, 0, GumpButtonType::Page, 2);
    hl(l, 55, 120, 200, 25, 1011540); // BUY SHOPKEEPERS
    l.add_button_type(20, 120, 4005, 4007, 0, GumpButtonType::Page, 3);
    hl(l, 55, 150, 200, 25, 1011495); // VIEW FINANCES
    l.add_button_type(20, 150, 4005, 4007, 0, GumpButtonType::Page, 4);
    hl(l, 55, 360, 200, 25, 1011441); // EXIT
    l.add_button(20, 360, 4005, 4007, 0);

    l.add_page(2);
    hl(l, 20, 30, 200, 25, 1011539); // CHANGE PRICES
    for (i, &ofs) in PRICE_OFFSETS.iter().enumerate() {
        let (x, y) = (20 + i as i32 / 6 * 150, 90 + i as i32 % 6 * 30);
        l.add_radio(x, y, 208, 209, tax == ofs, i as i32 + 1);
        if ofs < 0 {
            l.add_label(x + 35, y, 0x26, &format!("- {}%", -ofs));
        } else {
            l.add_label(x + 35, y, 0x12A, &format!("+ {ofs}%"));
        }
    }
    l.add_radio(20, 270, 208, 209, tax == 0, 0);
    hl(l, 55, 270, 90, 25, 1011542); // normal
    hl(l, 55, 330, 200, 25, 1011509); // Set Prices
    l.add_button(20, 330, 4005, 4007, to_button_id(2, 0, 0));
    hl(l, 55, 360, 200, 25, 1011067); // Previous page
    l.add_button_type(20, 360, 4005, 4007, 0, GumpButtonType::Page, 1);

    l.add_page(3);
    hl(l, 20, 30, 200, 25, 1011540); // BUY SHOPKEEPERS
    for (i, d) in VENDOR_DEFS.iter().enumerate() {
        let y = 90 + i as i32 * 40;
        l.add_button_type(20, y, 4005, 4007, 0, GumpButtonType::Page, 5 + i as i32);
        l.add_item(55, y, d.item_id, 0);
        ht(l, 100, y, 200, 25, d.label, false, false);
    }
    hl(l, 55, 360, 200, 25, 1011067); // Previous page
    l.add_button_type(20, 360, 4005, 4007, 0, GumpButtonType::Page, 1);

    l.add_page(4);
    finance_statement(w, l, t, false);

    let (fu, silver) = (town::finance_upkeep(w, t), s.towns[t].silver);
    for (i, d) in VENDOR_DEFS.iter().enumerate() {
        l.add_page(5 + i as i32);
        ht(l, 60, 30, 300, 25, d.header, false, false);
        l.add_item(20, 30, d.item_id, 0);
        hl(l, 20, 90, 200, 25, 1011514); // You have :
        l.add_label(230, 90, 0x26, &s.town_vendors[t][i].len().to_string());
        hl(l, 20, 120, 200, 25, 1011515); // Maximum :
        l.add_label(230, 120, 0x256, &d.maximum.to_string());
        hl(l, 20, 150, 200, 25, 1011516); // Cost :
        l.add_label(230, 150, 0x44, &n0(d.price as i64));
        hl(l, 20, 180, 200, 25, 1011517); // Daily Pay :
        l.add_label(230, 180, 0x37, &n0(d.upkeep as i64));
        hl(l, 20, 210, 200, 25, 1011518); // Current Silver :
        l.add_label(230, 210, 0x44, &n0(silver as i64));
        hl(l, 20, 240, 200, 25, 1011519); // Current Payroll :
        l.add_label(230, 240, 0x44, &n0(fu as i64));
        ht(l, 55, 300, 200, 25, d.label, false, false);
        if silver >= d.price {
            l.add_button(20, 300, 4005, 4007, to_button_id(2, 1, i as i32));
        } else {
            l.add_image(20, 300, 4020, 0);
        }
        hl(l, 55, 360, 200, 25, 1011067); // Previous page
        l.add_button_type(20, 360, 4005, 4007, 0, GumpButtonType::Page, 3);
    }
    send(w, from, g);
}

impl LegacyGumpLogic for FinanceGump {
    const TYPE_NAME: &'static str = "Server.Factions.FinanceGump";
    fn on_response(&mut self, w: &mut World, _s: NetStateId, info: &RelayInfo) {
        let (from, f, t) = (self.from, self.faction, self.town);
        if !town::is_finance(w, t, from) || town::owner(w, t) != Some(f) {
            msg(w, from, 1010339); // You no longer control this city
            return;
        }
        let Some((ty, index)) = from_button_id(2, info.button_id) else {
            return;
        };
        match ty {
            0 if index == 0 => set_prices(w, from, t, info),
            1 => {
                if !(0..VENDOR_DEFS.len() as i32).contains(&index) {
                    return;
                }
                let i = index as usize;
                let d = &VENDOR_DEFS[i];
                let here = muo_server::mobile::part1::props_b::region(w, from);
                let (count, silver) = st(w).map_or((0, 0), |s| {
                    (s.town_vendors[t][i].len() as i32, s.towns[t].silver)
                });
                if town::from_region(w, here) != Some(t) {
                    msg(w, from, 1010305); // You must be in your controlled city to buy Items
                } else if count >= d.maximum {
                    msg(w, from, 1010306); // You currently have too many of this enhancement type to place another
                } else if boat_at(w, from) {
                    mm::send_message(w, from, "You cannot place a vendor here");
                } else if silver >= d.price
                    && let Some(v) = super::mobiles::construct_vendor(w, i, t, f)
                {
                    st_mut(w).towns[t].silver -= d.price;
                    let (loc, map) = w
                        .mobile(from)
                        .map(|m| (m.location, m.map))
                        .unwrap_or_default();
                    super::mobiles::move_and_home(w, v, loc, map);
                }
            }
            _ => {}
        }
    }
}

fn boat_at(w: &World, from: MobileId) -> bool {
    // `BaseBoat.FindBoatAt(Location, Map)`
    w.mobile(from)
        .is_some_and(|m| muo_multis::boats::find_boat_at(w, m.location, m.map).is_some())
}

fn set_prices(w: &mut World, from: MobileId, t: usize, info: &RelayInfo) {
    let Some(&opt) = info.switches.first() else {
        return;
    };
    let new_tax = if opt >= 1 && opt <= PRICE_OFFSETS.len() as i32 {
        PRICE_OFFSETS[opt as usize - 1]
    } else {
        0
    };
    let (tax, last) = st(w).map_or((0, muo_primitives::DateTime::MIN), |s| {
        (s.towns[t].tax, s.towns[t].last_tax_change)
    });
    if tax == new_tax {
        return;
    }
    let player = w
        .mobile(from)
        .is_some_and(|m| m.access_level == AccessLevel::Player);
    if player && !town::tax_change_ready(w, t) {
        let remaining = w.now - (last + town::TAX_CHANGE_PERIOD);
        let minutes = remaining.total_minutes();
        let hours = remaining.total_hours();
        let n = if minutes < 4.0 {
            1042165 // You must wait a short while before changing prices again.
        } else if minutes < 10.0 {
            1042166 // You must wait several minutes before changing prices again.
        } else if hours < 1.0 {
            1042167 // You must wait up to an hour before changing prices again.
        } else if hours < 4.0 {
            1042168 // You must wait a few hours before changing prices again.
        } else {
            1042169 // You must wait several hours before changing prices again.
        };
        msg(w, from, n);
    } else {
        let now = w.now;
        let s = st_mut(w);
        s.towns[t].tax = new_tax;
        if player {
            s.towns[t].last_tax_change = now;
        }
    }
}

pub struct SheriffGump {
    from: MobileId,
    faction: usize,
    town: usize,
}

/// `CenterItem(itemID, x, y, w, h)`.
fn center_item(
    w: &mut World,
    l: &mut LegacyLayout,
    item_id: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) {
    muo_server::item::bounds::ensure_loaded(w);
    let b = muo_server::item::bounds::bounds(w, item_id as usize);
    l.add_item(
        x + (width - b.width()) / 2 - b.x(),
        y + (height - b.height()) / 2 - b.y(),
        item_id,
        0,
    );
}

pub fn send_sheriff_gump(w: &mut World, from: MobileId, f: usize, t: usize) {
    let mut g = LegacyGumpObj::new(
        50,
        50,
        SheriffGump {
            from,
            faction: f,
            town: t,
        },
    );
    let mut l = std::mem::take(&mut g.layout);
    {
        let l = &mut l;
        let guards = FACTION_DEFS[f].guards;
        l.add_page(0);
        l.add_background(0, 0, 320, 410, 5054);
        l.add_background(10, 10, 300, 390, 3000);
        l.add_page(1);
        hl(l, 20, 30, 260, 25, 1011431); // Sheriff
        hl(l, 55, 90, 200, 25, 1011494); // HIRE GUARDS
        l.add_button_type(20, 90, 4005, 4007, 0, GumpButtonType::Page, 3);
        hl(l, 55, 120, 200, 25, 1011495); // VIEW FINANCES
        l.add_button_type(20, 120, 4005, 4007, 0, GumpButtonType::Page, 2);
        hl(l, 55, 360, 200, 25, 1011441); // Exit
        l.add_button(20, 360, 4005, 4007, 0);
        l.add_page(2);
        finance_statement(w, l, t, true);
        l.add_page(3);
        hl(l, 20, 30, 300, 25, 1011494); // HIRE GUARDS
        for (i, d) in guards.iter().enumerate() {
            let y = 90 + i as i32 * 60;
            l.add_button_type(20, y, 4005, 4007, 0, GumpButtonType::Page, 4 + i as i32);
            center_item(w, l, d.item_id, 50, y - 20, 70, 60);
            ht(l, 120, y, 200, 25, d.header, false, false);
        }
        hl(l, 55, 360, 200, 25, 1011067); // Previous page
        l.add_button_type(20, 360, 4005, 4007, 0, GumpButtonType::Page, 1);
        let counts: Vec<usize> = st(w).map_or_else(Vec::new, |s| {
            s.town_guards[t].iter().map(Vec::len).collect()
        });
        let silver = st(w).map_or(0, |s| s.towns[t].silver);
        let su = town::sheriff_upkeep(w, t);
        for (i, d) in guards.iter().enumerate() {
            l.add_page(4 + i as i32);
            ht(l, 90, 30, 300, 25, d.header, false, false);
            center_item(w, l, d.item_id, 10, 10, 80, 80);
            hl(l, 20, 90, 200, 25, 1011514); // You have :
            l.add_label(
                230,
                90,
                0x26,
                &counts.get(i).copied().unwrap_or(0).to_string(),
            );
            hl(l, 20, 120, 200, 25, 1011515); // Maximum :
            l.add_label(230, 120, 0x12A, &d.maximum.to_string());
            hl(l, 20, 150, 200, 25, 1011516); // Cost :
            l.add_label(230, 150, 0x44, &n0(d.price as i64));
            hl(l, 20, 180, 200, 25, 1011517); // Daily Pay :
            l.add_label(230, 180, 0x37, &n0(d.upkeep as i64));
            hl(l, 20, 210, 200, 25, 1011518); // Current Silver :
            l.add_label(230, 210, 0x44, &n0(silver as i64));
            hl(l, 20, 240, 200, 25, 1011519); // Current Payroll :
            l.add_label(230, 240, 0x44, &n0(su as i64));
            ht(l, 55, 300, 200, 25, d.label, false, false);
            l.add_button(20, 300, 4005, 4007, 1 + i as i32);
            hl(l, 55, 360, 200, 25, 1011067); // Previous page
            l.add_button_type(20, 360, 4005, 4007, 0, GumpButtonType::Page, 3);
        }
    }
    g.layout = l;
    send(w, from, g);
}

impl LegacyGumpLogic for SheriffGump {
    const TYPE_NAME: &'static str = "Server.Factions.SheriffGump";
    fn on_response(&mut self, w: &mut World, _s: NetStateId, info: &RelayInfo) {
        let (from, f, t) = (self.from, self.faction, self.town);
        if !town::is_sheriff(w, t, from) || town::owner(w, t) != Some(f) {
            msg(w, from, 1010339); // You no longer control this city
            return;
        }
        let index = info.button_id - 1;
        let n = st(w).map_or(0, |s| s.town_guards[t].len() as i32);
        if index < 0 || index >= n {
            return;
        }
        let i = index as usize;
        let d = &FACTION_DEFS[f].guards[i];
        let here = muo_server::mobile::part1::props_b::region(w, from);
        let (count, silver) = st(w).map_or((0, 0), |s| {
            (s.town_guards[t][i].len() as i32, s.towns[t].silver)
        });
        if town::from_region(w, here) != Some(t) {
            msg(w, from, 1010305); // You must be in your controlled city to buy Items
        } else if count >= d.maximum {
            msg(w, from, 1010306);
        } else if boat_at(w, from) {
            mm::send_message(w, from, "You cannot place a guard here");
        } else if silver >= d.price
            && let Some(guard) = super::mobiles::construct_guard(w, d.type_name)
        {
            super::mobiles::set_guard_faction(w, guard, Some(f));
            super::mobiles::set_guard_town(w, guard, Some(t));
            st_mut(w).towns[t].silver -= d.price;
            let (loc, map) = w
                .mobile(from)
                .map(|m| (m.location, m.map))
                .unwrap_or_default();
            super::mobiles::move_and_home(w, guard, loc, map);
        }
    }
}

// ---- HorseBreederGump ----

pub const WAR_HORSE_SILVER_PRICE: i32 = 500;
pub const WAR_HORSE_GOLD_PRICE: i32 = 3000;

pub struct HorseBreederGump {
    from: MobileId,
    faction: usize,
}

pub fn send_horse_breeder_gump(w: &mut World, from: MobileId, f: usize) {
    let mut g = LegacyGumpObj::new(20, 30, HorseBreederGump { from, faction: f });
    let l = &mut g.layout;
    l.add_page(0);
    l.add_background(0, 0, 320, 280, 5054);
    l.add_background(10, 10, 300, 260, 3000);
    ht(l, 20, 30, 300, 25, FACTION_DEFS[f].header, false, false);
    hl(l, 20, 60, 300, 25, 1018306); // Purchase a Faction War Horse
    l.add_item(70, 120, 0x3FFE, 0);
    l.add_item(150, 120, 0xEF2, 0);
    l.add_label(190, 122, 0x3E3, &n0(WAR_HORSE_SILVER_PRICE as i64));
    l.add_item(150, 150, 0xEEF, 0);
    l.add_label(190, 152, 0x3E3, &n0(WAR_HORSE_GOLD_PRICE as i64));
    hl(l, 55, 210, 200, 25, 1011011); // CONTINUE
    l.add_button(20, 210, 4005, 4007, 1);
    hl(l, 55, 240, 200, 25, 1011012); // CANCEL
    l.add_button(20, 240, 4005, 4007, 0);
    send(w, from, g);
}

impl LegacyGumpLogic for HorseBreederGump {
    const TYPE_NAME: &'static str = "Server.Factions.HorseBreederGump";
    fn on_response(&mut self, w: &mut World, _s: NetStateId, info: &RelayInfo) {
        let (from, f) = (self.from, self.faction);
        if info.button_id != 1 || find(w, from) != Some(f) {
            return;
        }
        let Some(pack) = w.mobile(from).and_then(|m| m.backpack) else {
            return;
        };
        let Some(horse) = super::mobiles::new_war_horse(w, Some(f)) else {
            return;
        };
        let (followers, max) = w
            .mobile(from)
            .map_or((0, 0), |m| (m.followers, m.followers_max));
        let slots = muo_content::creature::cd(w, horse).map_or(1, |d| d.control_slots);
        let delete = |w: &mut World| muo_server::mobile::base::delete(w, horse);
        if followers + slots > max {
            msg(w, from, 1049607); // You have too many followers to control that creature.
            delete(w);
        } else if container::get_amount(w, pack, &crate::faction_items::SILVER, true)
            < WAR_HORSE_SILVER_PRICE
        {
            msg(w, from, 1042204); // You do not have enough silver.
            delete(w);
        } else if container::get_amount(w, pack, &muo_content::items::gold::GOLD, true)
            < WAR_HORSE_GOLD_PRICE
        {
            msg(w, from, 1042205); // You do not have enough gold.
            delete(w);
        } else if container::consume_total(
            w,
            pack,
            &crate::faction_items::SILVER,
            WAR_HORSE_SILVER_PRICE,
            true,
            None,
        ) && container::consume_total(
            w,
            pack,
            &muo_content::items::gold::GOLD,
            WAR_HORSE_GOLD_PRICE,
            true,
            None,
        ) {
            muo_content::creature::control::set_control_master(w, horse, Some(from));
            muo_content::creature::control::set_control_order_raw(
                w,
                horse,
                muo_content::creature::enums::OrderType::Follow,
            );
            if let Some(d) = muo_content::creature::cd_mut(w, horse) {
                d.control_target = Some(from);
            }
            let (loc, map) = w
                .mobile(from)
                .map(|m| (m.location, m.map))
                .unwrap_or_default();
            if let Some(map) = map {
                muo_server::mobile::base::move_to_world(w, horse, loc, map);
            }
        } else {
            delete(w);
        }
    }
}

// ---- FactionImbueGump ----

type AfterFn = Box<dyn Fn(&mut World)>;

pub struct FactionImbueGump {
    from: MobileId,
    faction: usize,
    item: ItemId,
    silver_cost: i32,
    is_scroll: bool,
    /// `CraftItem.ShowCraftMenu(from, system, tool, notice)` or the notice message, run after the response.
    after: Option<AfterFn>,
}

/// `new FactionImbueGump(quality, item, from, ..., availableSilver, faction, def)`; `after` is what happens
/// with the craft system, tool and notice once the player answers.
#[allow(clippy::too_many_arguments)] // one parameter per field of the call
pub fn send_imbue_gump(
    w: &mut World,
    from: MobileId,
    f: usize,
    item: ItemId,
    quality: i32,
    available_silver: i32,
    silver_cost: i32,
    after: AfterFn,
) {
    let is_scroll = w
        .item(item)
        .is_some_and(|i| i.ty.full_name.contains("SpellScroll"));
    let mut g = LegacyGumpObj::new(
        100,
        200,
        FactionImbueGump {
            from,
            faction: f,
            item,
            silver_cost,
            is_scroll,
            after: Some(after),
        },
    );
    let l = &mut g.layout;
    let def = FACTION_DEFS[f];
    l.add_page(0);
    l.add_background(0, 0, 320, 270, 5054);
    l.add_background(10, 10, 300, 250, 3000);
    hl(l, 20, 20, 210, 25, 1011569); // Imbue with Faction properties?
    hl(l, 20, 60, 170, 25, 1018302); // Item quality:
    hl(l, 175, 60, 100, 25, 1018305 - quality); // Exceptional, Average, Low
    hl(l, 20, 80, 170, 25, 1011572); // Item Cost :
    l.add_label(175, 80, 0x34, &n0(silver_cost as i64));
    hl(l, 20, 100, 170, 25, 1011573); // Your Silver :
    l.add_label(175, 100, 0x34, &n0(available_silver as i64));
    l.add_radio(20, 140, 210, 211, true, 1);
    l.add_label(55, 140, def.hue_primary - 1, "*****");
    hl(l, 150, 140, 150, 25, 1011570); // Primary Color
    l.add_radio(20, 160, 210, 211, false, 2);
    l.add_label(55, 160, def.hue_secondary - 1, "*****");
    hl(l, 150, 160, 150, 25, 1011571); // Secondary Color
    hl(l, 55, 200, 200, 25, 1011011); // CONTINUE
    l.add_button(20, 200, 4005, 4007, 1);
    hl(l, 55, 230, 200, 25, 1011012); // CANCEL
    l.add_button(20, 230, 4005, 4007, 0);
    send(w, from, g);
}

impl LegacyGumpLogic for FactionImbueGump {
    const TYPE_NAME: &'static str = "Server.Factions.FactionImbueGump";
    fn on_response(&mut self, w: &mut World, _s: NetStateId, info: &RelayInfo) {
        let (from, f) = (self.from, self.faction);
        if info.button_id == 1 {
            let pack = w.mobile(from).and_then(|m| m.backpack);
            if let Some(pack) = pack
                && muo_server::item::ops::is_child_of(w, self.item, Some(EntityId::Item(pack)))
            {
                if container::consume_total(
                    w,
                    pack,
                    &crate::faction_items::SILVER,
                    self.silver_cost,
                    true,
                    None,
                ) {
                    let def = FACTION_DEFS[f];
                    let hue = if self.is_scroll {
                        0
                    } else if info.is_switched(1) {
                        def.hue_primary
                    } else {
                        def.hue_secondary
                    };
                    super::imbue(w, self.item, f, true, hue);
                } else {
                    msg(w, from, 1042204); // You do not have enough silver.
                }
            }
        }
        if let Some(after) = self.after.take() {
            after(w);
        }
    }
}
