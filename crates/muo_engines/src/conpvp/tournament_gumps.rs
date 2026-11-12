//! `ConfirmSignupGump`, `AcceptTeamGump` and `TournamentBracketGump`.
//!
//! A gump's layout cannot read the world here, so each gump is laid out from a snapshot taken when it is sent
//! (`SignupView`; the bracket gump's `Op` list). "Refresh via this" sends the same gump logic again, re-taking the snapshot.
//!
//! The team of a signup (`List<Mobile> _players`) is shared by the signup gump and the `AcceptTeamGump`s it makes.
//!
//! `TournamentBracketGump` never has a list (`_list` is always null), and the Participant_List, Round_List and
//! Round_Info layouts read `_list.Count` in `StartPage`: they throw in `BuildLayout`, so those pages never open. That is
//! kept (a page that cannot be laid out is not sent).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use muo_content::gumps::{DynamicGumpBuilder, DynamicGumpLogic, RelayInfo, close_gump, has_gump};
use muo_primitives::html;
use muo_primitives::{MessageType, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::part2::messages as mm;
use muo_server::net_state::NetStateId;
use muo_server::target::{self, Target, TargetFlags, Targeted};
use muo_server::timer;

use super::flow::name;
use super::gumps::{self, AcceptDuelGump, center, golden_button, mobile_of, ns_of, send_dynamic};
use super::tournament::{self as tn, GroupingType, Stage, TieType, Tournament, TourneyType};

const LABEL_COLOR32: i32 = 0xFFFFFF;
const BLACK_COLOR32: i32 = 0x000008;

/// `Team`: the players of a signup, shared between the gumps (`List<Mobile>`).
pub type Team = Rc<RefCell<Vec<MobileId>>>;

fn say_private(w: &mut World, registrar: Option<MobileId>, to: MobileId, hue: i32, text: &str) {
    if let Some(r) = registrar {
        let ns = ns_of(w, to);
        mm::private_overhead_message(w, r, MessageType::REGULAR, hue, false, text, ns);
    }
}

/// `AddBorderedText`: the text four times in black, offset on both axes, then in colour.
fn bordered(
    b: &mut DynamicGumpBuilder,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    text: &str,
    color: i32,
) {
    let black = html::color_rgb(text, BLACK_COLOR32);
    for (dx, dy) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        b.add_html(x + dx, y + dy, width, height, &black, false, false);
    }
    b.add_html(
        x,
        y,
        width,
        height,
        &html::color_rgb(text, color),
        false,
        false,
    );
}

// ---- the tournament as the signup gumps show it --------------------------------------------------------------------

struct SignupView {
    title: String,
    group_text: String,
    tie_text: String,
    sd_text: String,
    base_title: String,
    flavors: Vec<String>,
    /// (name, enabled) of every option that differs from the base and flavors.
    changes: Vec<(String, bool)>,
    players_per_participant: i32,
}

fn tourney_title(w: &World, t: &Tournament, ffa_word: &str) -> String {
    let mut sb = String::new();
    match t.tourney_type {
        TourneyType::FreeForAll => sb.push_str(ffa_word),
        TourneyType::RandomTeam => sb.push_str(&format!("{}-Team", t.participants_per_match)),
        TourneyType::Faction => sb.push_str(&format!("{}-Team Faction", t.participants_per_match)),
        TourneyType::RedVsBlue => sb.push_str("Red v Blue"),
        TourneyType::Standard => {
            for _ in 0..t.participants_per_match {
                if !sb.is_empty() {
                    sb.push('v');
                }
                sb.push_str(&t.players_per_participant.to_string());
            }
        }
    }
    if let Some(ec) = t.event_controller {
        sb.push_str(&format!(" {}", tn::event_title(w, ec)));
    }
    sb
}

fn group_text(t: &Tournament) -> &'static str {
    match t.group_type {
        GroupingType::HighVsLow => "High vs Low",
        GroupingType::Nearest => "Closest opponent",
        GroupingType::Random => "Random",
    }
}

fn tie_text(t: &Tournament) -> &'static str {
    match t.tie_type {
        TieType::Random => "Random",
        TieType::Highest => "Highest advances",
        TieType::Lowest => "Lowest advances",
        TieType::FullAdvancement => {
            if t.participants_per_match == 2 {
                "Both advance"
            } else {
                "Everyone advances"
            }
        }
        TieType::FullElimination => {
            if t.participants_per_match == 2 {
                "Both eliminated"
            } else {
                "Everyone eliminated"
            }
        }
    }
}

fn sudden_death_text(t: &Tournament) -> String {
    if t.sudden_death > TimeSpan::ZERO {
        let mins = t.sudden_death.total_minutes() as i32;
        let secs = t.sudden_death.seconds();
        if t.sudden_death_rounds > 0 {
            format!(
                "Sudden Death: {mins}:{secs:02} (first {} rounds)",
                t.sudden_death_rounds
            )
        } else {
            format!("Sudden Death: {mins}:{secs:02} (all rounds)")
        }
    } else {
        "Sudden Death: Off".to_owned()
    }
}

/// The ruleset lines: the base title, the flavors and the options that differ.
fn ruleset_view(t: &Tournament) -> (String, Vec<String>, Vec<(String, bool)>) {
    let rs = &t.ruleset;
    let layout = &rs.layout;
    let base = rs.base.unwrap_or(0);
    let mut defs = layout.defaults[base].options.clone();
    for f in &rs.flavors {
        for (d, o) in defs.iter_mut().zip(&layout.flavors[*f].options) {
            *d |= *o;
        }
    }
    let mut changes = Vec::new();
    for (i, opt) in rs.options.iter().enumerate() {
        if defs.get(i).copied().unwrap_or(false) != *opt {
            // A nameless option takes space but shows nothing (`name != null`, "sanity").
            changes.push((layout.find_by_index(i).unwrap_or_default(), *opt));
        }
    }
    let flavors = rs
        .flavors
        .iter()
        .map(|f| layout.flavors[*f].title.clone().unwrap_or_default())
        .collect();
    (
        layout.defaults[base].title.clone().unwrap_or_default(),
        flavors,
        changes,
    )
}

fn signup_view(w: &World, t: &Tournament, suffix: &str) -> SignupView {
    let (base_title, flavors, changes) = ruleset_view(t);
    SignupView {
        title: format!("{} {suffix}", tourney_title(w, t, "FFA")),
        group_text: group_text(t).to_owned(),
        tie_text: tie_text(t).to_owned(),
        sd_text: sudden_death_text(t),
        base_title,
        flavors,
        changes,
        players_per_participant: t.players_per_participant,
    }
}

/// The shared part of both gumps up to the "Modifications" lines. Returns the next y.
fn layout_rules(b: &mut DynamicGumpBuilder, v: &SignupView, intro: &str) -> i32 {
    b.add_image(215, -43, 0xEE40, 0);
    bordered(b, 22, 22, 294, 20, &center(&v.title), LABEL_COLOR32);
    bordered(b, 22, 50, 294, 40, intro, 0xB0C868);
    b.add_image_tiled(32, 88, 264, 1, 9107);
    b.add_image_tiled(42, 90, 264, 1, 9157);

    let mut y = 100;
    bordered(
        b,
        35,
        y,
        190,
        20,
        &format!("Grouping: {}", v.group_text),
        LABEL_COLOR32,
    );
    y += 20;
    bordered(
        b,
        35,
        y,
        190,
        20,
        &format!("Tiebreaker: {}", v.tie_text),
        LABEL_COLOR32,
    );
    y += 20;
    bordered(b, 35, y, 240, 20, &v.sd_text, LABEL_COLOR32);
    y += 20;

    y += 6;
    b.add_image_tiled(32, y - 1, 264, 1, 9107);
    b.add_image_tiled(42, y + 1, 264, 1, 9157);
    y += 6;

    bordered(
        b,
        35,
        y,
        190,
        20,
        &format!("Ruleset: {}", v.base_title),
        LABEL_COLOR32,
    );
    y += 20;
    for f in &v.flavors {
        bordered(b, 35, y, 190, 20, &format!(" + {f}"), LABEL_COLOR32);
        y += 18;
    }
    y += 4;

    if !v.changes.is_empty() {
        bordered(b, 35, y, 190, 20, "Modifications:", LABEL_COLOR32);
        y += 20;
        for (name, on) in &v.changes {
            if !name.is_empty() {
                b.add_image(35, y, if *on { 0xD3 } else { 0xD2 }, 0);
                bordered(b, 60, y, 165, 22, name, LABEL_COLOR32);
            }
            y += 22;
        }
    } else {
        bordered(b, 35, y, 190, 20, "Modifications: None", LABEL_COLOR32);
        y += 20;
    }
    y
}

fn view_height(v: &SignupView, base: i32) -> i32 {
    let mut height = base + v.flavors.len() as i32 * 18;
    height += v.changes.len() as i32 * 22;
    height + 10 + 22 + 25 + 25
}

// ---- ConfirmSignupGump ----------------------------------------------------------------------------------------------

/// `ConfirmSignupGump`.
pub struct ConfirmSignupGump {
    registrar: Option<MobileId>,
    ctrl: ItemId,
    players: Team,
    view: SignupView,
    /// The names of the team, in order, when it was laid out.
    names: Vec<String>,
}

/// `ConfirmSignupGump.DisplayTo(from, registrar, tourney, players)`.
pub fn display_confirm_signup(
    w: &mut World,
    from: MobileId,
    registrar: Option<MobileId>,
    ctrl: ItemId,
    players: Vec<MobileId>,
) {
    display_confirm_team(w, from, registrar, ctrl, Rc::new(RefCell::new(players)));
}

fn display_confirm_team(
    w: &mut World,
    from: MobileId,
    registrar: Option<MobileId>,
    ctrl: ItemId,
    players: Team,
) {
    let Some(ns) = ns_of(w, from) else { return };
    let Some(t) = tn::get(w, ctrl).cloned() else {
        return;
    };
    close_gump::<AcceptTeamGump>(w, ns);
    close_gump::<AcceptDuelGump>(w, ns);
    close_gump::<gumps::DuelContextGump>(w, ns);

    let names = players.borrow().iter().map(|m| name(w, *m)).collect();
    let view = signup_view(w, &t, "Tournament Signup");
    send_dynamic(
        w,
        from,
        50,
        50,
        ConfirmSignupGump {
            registrar,
            ctrl,
            players,
            view,
            names,
        },
        true,
    );
}

impl DynamicGumpLogic for ConfirmSignupGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.ConfirmSignupGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();
        let v = &self.view;
        let mut height = 185 + 60 + 12;
        height = view_height(v, height);
        if v.players_per_participant > 1 {
            height += 36 + v.players_per_participant * 20;
        }
        b.add_page(0);
        b.add_background(1, 1, 398, height, 3600);
        b.add_image_tiled(16, 15, 369, height - 29, 3604);
        b.add_alpha_region(16, 15, 369, height - 29);

        let mut y = layout_rules(
            b,
            v,
            "You have requested to join the tournament. Do you accept?",
        );

        if v.players_per_participant > 1 {
            y += 8;
            b.add_image_tiled(32, y - 1, 264, 1, 9107);
            b.add_image_tiled(42, y + 1, 264, 1, 9157);
            y += 8;
            bordered(b, 35, y, 190, 20, "Your Team", LABEL_COLOR32);
            y += 20;
            for (i, n) in self.names.iter().enumerate() {
                if i == 0 {
                    b.add_image(35, y, 0xD2, 0);
                } else {
                    golden_button(b, 35, y, 1 + i as i32);
                }
                bordered(b, 60, y, 200, 20, n, LABEL_COLOR32);
                y += 20;
            }
            for i in self.names.len() as i32..v.players_per_participant {
                if i == 0 {
                    b.add_image(35, y, 0xD2, 0);
                } else {
                    golden_button(b, 35, y, 1 + i);
                }
                bordered(b, 60, y, 200, 20, "(Empty)", LABEL_COLOR32);
                y += 20;
            }
        }

        y += 8;
        b.add_image_tiled(32, y - 1, 264, 1, 9107);
        b.add_image_tiled(42, y + 1, 264, 1, 9157);
        y += 8;

        b.add_radio(24, y, 9727, 9730, true, 1);
        bordered(
            b,
            60,
            y + 5,
            250,
            20,
            "Yes, I wish to join the tournament.",
            LABEL_COLOR32,
        );
        y += 35;
        b.add_radio(24, y, 9727, 9730, false, 2);
        bordered(
            b,
            60,
            y + 5,
            250,
            20,
            "No, I do not wish to join.",
            LABEL_COLOR32,
        );
        y += 35;
        y -= 3;
        b.add_button(314, y, 247, 248, 1);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let (registrar, ctrl) = (self.registrar, self.ctrl);
        let again = |w: &mut World, this: &ConfirmSignupGump| {
            display_confirm_team(w, from, this.registrar, this.ctrl, this.players.clone());
        };

        if info.button_id != 1 || !info.is_switched(1) {
            if info.button_id > 1 {
                let index = info.button_id - 1;
                let count = self.players.borrow().len() as i32;
                if index > 0 && index < count {
                    self.players.borrow_mut().remove(index as usize);
                    again(w, self);
                } else if tn::get(w, ctrl).is_some_and(|t| count < t.players_per_participant) {
                    begin_add_player_target(w, from, registrar, ctrl, self.players.clone());
                    again(w, self);
                }
            }
            return;
        }

        let Some(t) = tn::get(w, ctrl).cloned() else {
            return;
        };
        match t.stage {
            Stage::Fighting => {
                if registrar.is_some() {
                    if t.has_participant(from) {
                        say_private(
                            w,
                            registrar,
                            from,
                            0x35,
                            "Excuse me? You are already signed up.",
                        );
                    } else {
                        say_private(
                            w,
                            registrar,
                            from,
                            0x22,
                            "The tournament has already begun. You are too late to signup now.",
                        );
                    }
                }
            }
            Stage::Inactive => say_private(w, registrar, from, 0x35, "The tournament is closed."),
            Stage::Signup => {
                if self.players.borrow().len() as i32 != t.players_per_participant {
                    say_private(
                        w,
                        registrar,
                        from,
                        0x35,
                        "You have not yet chosen your team.",
                    );
                    again(w, self);
                    return;
                }
                let lad = super::ladder::instance(w);
                for mob in self.players.borrow().clone() {
                    let entry = lad.and_then(|l| super::ladder::find(w, l, mob));
                    let n = name(w, mob);
                    if entry.is_some_and(|e| {
                        super::ladder::get_level(e.experience) < t.level_requirement
                    }) {
                        if registrar.is_some() {
                            if mob == from {
                                say_private(
                                    w,
                                    registrar,
                                    from,
                                    0x35,
                                    "You have not yet proven yourself a worthy dueler.",
                                );
                            } else {
                                say_private(
                                    w,
                                    registrar,
                                    from,
                                    0x35,
                                    &format!("{n} has not yet proven themselves a worthy dueler."),
                                );
                            }
                        }
                    } else if t.is_faction_restricted() && crate::factions::find(w, mob).is_none() {
                        say_private(
                            w,
                            registrar,
                            from,
                            0x35,
                            "Only those who have declared their faction allegiance may participate.",
                        );
                    } else if t.has_participant(mob) {
                        if registrar.is_some() {
                            if mob == from {
                                say_private(
                                    w,
                                    registrar,
                                    from,
                                    0x35,
                                    "You have already entered this tournament.",
                                );
                            } else {
                                say_private(
                                    w,
                                    registrar,
                                    from,
                                    0x35,
                                    &format!("{n} has already entered this tournament."),
                                );
                            }
                        }
                    } else if super::duel_context_of(w, mob).is_some() {
                        if mob == from {
                            say_private(
                                w,
                                registrar,
                                from,
                                0x35,
                                "You are already assigned to a duel. You must yield it before joining this tournament.",
                            );
                        } else {
                            say_private(
                                w,
                                registrar,
                                from,
                                0x35,
                                &format!(
                                    "{n} is already assigned to a duel. They must yield it before joining this tournament."
                                ),
                            );
                        }
                    } else {
                        continue;
                    }
                    again(w, self);
                    return;
                }

                if registrar.is_some() {
                    let minutes_until = ((t.signup_start + t.signup_period - w.now).total_minutes())
                        .round_ties_even() as i32;
                    let time_until = if minutes_until == 0 {
                        "momentarily".to_owned()
                    } else {
                        format!(
                            "in {minutes_until} minute{}",
                            if minutes_until == 1 { "" } else { "s" }
                        )
                    };
                    let title = if w.mobile(from).is_some_and(|m| m.female) {
                        "Lady"
                    } else {
                        "Lord"
                    };
                    let text = match t.players_per_participant {
                        1 => format!(
                            "As you say m'{title}. I've written your name to the bracket. The tournament will begin {time_until}."
                        ),
                        2 => format!(
                            "As you wish m'{title}. The tournament will begin {time_until}, but first you must name your partner."
                        ),
                        _ => format!(
                            "As you wish m'{title}. The tournament will begin {time_until}, but first you must name your team."
                        ),
                    };
                    say_private(w, registrar, from, 0x35, &text);
                }

                let players = self.players.borrow().clone();
                if let Some(tm) = tn::get_mut(w, ctrl) {
                    tm.parts.push(tn::TourneyPart::new(players));
                    let idx = tm.parts.len() - 1;
                    tm.participants.push(idx);
                }
            }
        }
    }
}

/// `from.BeginTarget(12, false, TargetFlags.None, AddPlayer_OnTarget)`.
fn begin_add_player_target(
    w: &mut World,
    from: MobileId,
    registrar: Option<MobileId>,
    ctrl: ItemId,
    players: Team,
) {
    let mut t = Target::new(w, 12, false, TargetFlags::NONE);
    t.handlers.on_target = Some(Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            add_player_on_target(w, from, registrar, ctrl, players.clone(), targeted)
        },
    ));
    let tid = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(tid));
}

fn add_player_on_target(
    w: &mut World,
    from: MobileId,
    registrar: Option<MobileId>,
    ctrl: ItemId,
    players: Team,
    targeted: Targeted,
) {
    let Some(t) = tn::get(w, ctrl).cloned() else {
        return;
    };
    let mob = match targeted {
        Targeted::Entity(EntityId::Mobile(m)) if m != from => Some(m),
        _ => None,
    };
    let Some(mob) = mob else {
        say_private(w, registrar, from, 0x22, "Excuse me?");
        display_confirm_team(w, from, registrar, ctrl, players);
        return;
    };
    let mob_ns = ns_of(w, mob);
    let player = w.mobile(mob).is_some_and(|m| m.player);
    if !player {
        if muo_content::creature::misc::body_is_human(w, mob) {
            muo_server::mobile::part2::speech::say_to_number(w, mob, from, 1005443, ""); // Nay, I would rather stay here and watch a nail rust.
        } else {
            muo_server::mobile::part2::speech::say_to_number(w, mob, from, 1005444, ""); // The creature ignores your offer.
        }
    } else if gumps::is_ignored(w, mob, from) || w.mobile(mob).is_some_and(|m| m.blessed) {
        say_private(w, registrar, from, 0x22, "They ignore your invitation.");
    } else if muo_content::player_mobile::data(w, mob).is_none() {
        return;
    } else if super::duel_context_of(w, mob).is_some() {
        say_private(
            w,
            registrar,
            from,
            0x22,
            "They are already assigned to another duel.",
        );
    } else if mob_ns.is_some_and(|ns| has_gump::<AcceptTeamGump>(w, ns)) {
        say_private(
            w,
            registrar,
            from,
            0x22,
            "They have already been offered a partnership.",
        );
    } else if mob_ns.is_some_and(|ns| has_gump::<ConfirmSignupGump>(w, ns)) {
        say_private(
            w,
            registrar,
            from,
            0x22,
            "They are already trying to join this tournament.",
        );
    } else if players.borrow().contains(&mob) {
        say_private(
            w,
            registrar,
            from,
            0x22,
            "You have already named them as a team member.",
        );
    } else if t.has_participant(mob) {
        say_private(
            w,
            registrar,
            from,
            0x22,
            "They have already entered this tournament.",
        );
    } else if players.borrow().len() as i32 >= t.players_per_participant {
        say_private(w, registrar, from, 0x22, "Your team is full.");
    } else {
        display_accept_team(w, from, mob, ctrl, registrar, players.clone());
        let n = name(w, mob);
        let lord = if w.mobile(from).is_some_and(|m| m.female) {
            "Lady"
        } else {
            "Lord"
        };
        say_private(
            w,
            registrar,
            from,
            0x59,
            &format!("As you command m'{lord}. I've given your offer to {n}."),
        );
    }
    display_confirm_team(w, from, registrar, ctrl, players);
}

// ---- AcceptTeamGump -------------------------------------------------------------------------------------------------

/// `AcceptTeamGump`.
pub struct AcceptTeamGump {
    from: MobileId,
    requested: MobileId,
    registrar: Option<MobileId>,
    ctrl: ItemId,
    players: Team,
    active: Rc<Cell<bool>>,
    view: SignupView,
    from_name: String,
}

/// `AcceptTeamGump.DisplayTo(from, requested, tourney, registrar, players)`.
pub fn display_accept_team(
    w: &mut World,
    from: MobileId,
    requested: MobileId,
    ctrl: ItemId,
    registrar: Option<MobileId>,
    players: Team,
) {
    if ns_of(w, requested).is_none() {
        return;
    }
    let Some(t) = tn::get(w, ctrl).cloned() else {
        return;
    };
    let active = Rc::new(Cell::new(true));
    let a = active.clone();
    let p = players.clone();
    timer::start(w, TimeSpan::from_secs(15), TimeSpan::ZERO, 1, move |w| {
        auto_reject(w, &a, from, requested, ctrl, registrar, p.clone())
    });
    let view = signup_view(w, &t, "Tournament Invitation");
    let from_name = name(w, from);
    send_dynamic(
        w,
        requested,
        50,
        50,
        AcceptTeamGump {
            from,
            requested,
            registrar,
            ctrl,
            players,
            active,
            view,
            from_name,
        },
        true,
    );
}

/// `AutoReject()`.
fn auto_reject(
    w: &mut World,
    active: &Cell<bool>,
    from: MobileId,
    requested: MobileId,
    ctrl: ItemId,
    registrar: Option<MobileId>,
    players: Team,
) {
    if !active.get() {
        return;
    }
    active.set(false);
    if let Some(ns) = ns_of(w, requested) {
        close_gump::<AcceptTeamGump>(w, ns);
    }
    display_confirm_team(w, from, registrar, ctrl, players);
    if registrar.is_some() {
        let rn = name(w, requested);
        say_private(
            w,
            registrar,
            from,
            0x22,
            &format!("{rn} seems unresponsive."),
        );
        let fnm = name(w, from);
        say_private(
            w,
            registrar,
            requested,
            0x22,
            &format!("You have declined the partnership with {fnm}."),
        );
    }
}

impl DynamicGumpLogic for AcceptTeamGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.AcceptTeamGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();
        let v = &self.view;
        let height = view_height(v, 185 + 35 + 60 + 12);
        b.add_page(0);
        b.add_background(1, 1, 398, height, 3600);
        b.add_image_tiled(16, 15, 369, height - 29, 3604);
        b.add_alpha_region(16, 15, 369, height - 29);

        let mut y = layout_rules(
            b,
            v,
            &format!(
                "You have been asked to partner with {} in a tournament. Do you accept?",
                self.from_name
            ),
        );

        y += 8;
        b.add_image_tiled(32, y - 1, 264, 1, 9107);
        b.add_image_tiled(42, y + 1, 264, 1, 9157);
        y += 8;

        for (i, text) in [
            "Yes, I will join them.",
            "No, I do not wish to fight.",
            "No, most certainly not. Do not ask again.",
        ]
        .iter()
        .enumerate()
        {
            b.add_radio(24, y, 9727, 9730, i == 0, 1 + i as i32);
            bordered(
                b,
                60,
                y + 5,
                if i == 2 { 270 } else { 250 },
                20,
                text,
                LABEL_COLOR32,
            );
            y += 35;
        }
        y -= 3;
        b.add_button(314, y, 247, 248, 1);
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let (from, mob) = (self.from, self.requested);
        if info.button_id != 1 || !self.active.get() {
            return;
        }
        self.active.set(false);
        let (registrar, ctrl) = (self.registrar, self.ctrl);
        let back = |w: &mut World, this: &AcceptTeamGump| {
            display_confirm_team(
                w,
                this.from,
                this.registrar,
                this.ctrl,
                this.players.clone(),
            )
        };

        if info.is_switched(1) {
            if muo_content::player_mobile::data(w, mob).is_none() {
                return;
            }
            let Some(t) = tn::get(w, ctrl).cloned() else {
                return;
            };
            let refuse = |w: &mut World, this: &AcceptTeamGump, text: &str| {
                back(w, this);
                say_private(w, registrar, from, 0x22, text);
            };
            if gumps::is_ignored(w, mob, from) || w.mobile(mob).is_some_and(|m| m.blessed) {
                refuse(w, self, "They ignore your invitation.");
            } else if super::duel_context_of(w, mob).is_some() {
                refuse(w, self, "They are already assigned to another duel.");
            } else if self.players.borrow().contains(&mob) {
                refuse(w, self, "You have already named them as a team member.");
            } else if t.has_participant(mob) {
                refuse(w, self, "They have already entered this tournament.");
            } else if self.players.borrow().len() as i32 >= t.players_per_participant {
                refuse(w, self, "Your team is full.");
            } else {
                self.players.borrow_mut().push(mob);
                back(w, self);
                if registrar.is_some() {
                    let mn = name(w, mob);
                    say_private(
                        w,
                        registrar,
                        from,
                        0x59,
                        &format!("{mn} has accepted your offer of partnership."),
                    );
                    let fnm = name(w, from);
                    say_private(
                        w,
                        registrar,
                        mob,
                        0x59,
                        &format!("You have accepted the partnership with {fnm}."),
                    );
                }
            }
        } else {
            if info.is_switched(3) {
                gumps::begin_ignore(w, mob, from);
            }
            back(w, self);
            if registrar.is_some() {
                let mn = name(w, mob);
                say_private(
                    w,
                    registrar,
                    from,
                    0x22,
                    &format!("{mn} has declined your offer of partnership."),
                );
                let fnm = name(w, from);
                say_private(
                    w,
                    registrar,
                    mob,
                    0x22,
                    &format!("You have declined the partnership with {fnm}."),
                );
            }
        }
    }
}

// ---- TournamentBracketGump ------------------------------------------------------------------------------------------

/// `TourneyBracketGumpType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BracketType {
    Index,
    RulesInfo,
    ParticipantList,
    ParticipantInfo,
    RoundList,
    RoundInfo,
    MatchInfo,
    PlayerInfo,
}

/// `_object`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Obj {
    Part(usize),
    Mob(MobileId),
    Level(usize),
    Match(usize, usize),
}

enum Op {
    Page,
    Background(i32, i32, i32, i32, i32),
    Html(i32, i32, i32, i32, String, bool),
    Button(i32, i32, i32, i32, i32),
    Image(i32, i32, i32),
}

/// `TournamentBracketGump`.
pub struct TournamentBracketGump {
    ctrl: ItemId,
    ty: BracketType,
    obj: Option<Obj>,
    ops: Vec<Op>,
}

/// `TournamentBracketGump.DisplayTo(from, tourney, type, list, page, obj)` (the list is never given).
pub fn display_bracket(
    w: &mut World,
    from: MobileId,
    ctrl: ItemId,
    ty: BracketType,
    obj: Option<Obj>,
) {
    if ns_of(w, from).is_none() || tn::get(w, ctrl).is_none() {
        return;
    }
    // A layout that fails to build is not sent.
    if let Some(ops) = compose(w, ctrl, ty, obj) {
        send_dynamic(
            w,
            from,
            50,
            50,
            TournamentBracketGump { ctrl, ty, obj, ops },
            true,
        );
    }
}

fn to_button_id(ty: i32, index: i32) -> i32 {
    1 + index * 7 + ty
}

fn right_arrow(ops: &mut Vec<Op>, x: i32, y: i32, bid: i32, text: Option<&str>) {
    ops.push(Op::Button(x, y, 0x15E1, 0x15E5, bid));
    if let Some(t) = text {
        ops.push(Op::Html(x + 20, y - 1, 230, 20, t.to_owned(), false));
    }
}

fn left_arrow(ops: &mut Vec<Op>, x: i32, y: i32, bid: i32) {
    ops.push(Op::Button(x, y, 0x15E3, 0x15E7, bid));
}

fn html_at(ops: &mut Vec<Op>, x: i32, y: i32, w: i32, h: i32, text: String) {
    ops.push(Op::Html(x, y, w, h, text, false));
}

fn color(text: &str, c: i32) -> String {
    html::color_rgb(text, c)
}

/// A team's label in a match list: "Team 1 (3)", "Minax (2)", ...
fn team_label(w: &World, t: &Tournament, j: usize, count: usize) -> String {
    if let Some(ec) = t.event_controller {
        format!("Team {} ({count})", tn::event_team_name(w, ec, j))
    } else if t.tourney_type == TourneyType::RandomTeam {
        format!("Team {} ({count})", j + 1)
    } else if t.tourney_type == TourneyType::Faction {
        if t.participants_per_match == 4 {
            let n = match j {
                0 => "Minax",
                1 => "Council of Mages",
                2 => "True Britannians",
                3 => "Shadowlords",
                _ => "(null)",
            };
            format!("{n} ({count})")
        } else if t.participants_per_match == 2 {
            format!("{} Team ({count})", if j == 0 { "Evil" } else { "Hero" })
        } else {
            format!("Team {} ({count})", j + 1)
        }
    } else {
        format!("Team {} ({count})", if j == 0 { "Red" } else { "Blue" })
    }
}

fn has_team_labels(t: &Tournament) -> bool {
    t.event_controller.is_some()
        || matches!(
            t.tourney_type,
            TourneyType::RandomTeam | TourneyType::RedVsBlue | TourneyType::Faction
        )
}

/// `BuildLayout` for the page, or `None` where it cannot be built.
fn compose(w: &mut World, ctrl: ItemId, ty: BracketType, obj: Option<Obj>) -> Option<Vec<Op>> {
    let t = tn::get(w, ctrl)?.clone();
    let mut ops = Vec::new();
    match ty {
        BracketType::Index => {
            ops.push(Op::Page);
            ops.push(Op::Background(0, 0, 300, 300, 9380));
            let title = format!("{} Tournament Bracket", tourney_title(w, &t, "FFA"));
            html_at(&mut ops, 25, 35, 250, 20, center(&title));
            right_arrow(&mut ops, 25, 53, to_button_id(0, 4), Some("Rules"));
            right_arrow(&mut ops, 25, 71, to_button_id(0, 1), Some("Participants"));
            if t.stage == Stage::Signup {
                let until = t.signup_start + t.signup_period - w.now;
                let secs = until.total_seconds().round_ties_even() as i32;
                let text = if secs > 0 {
                    let (mins, secs) = (secs / 60, secs % 60);
                    let s = |n: i32| if n == 1 { "" } else { "s" };
                    if mins > 0 && secs > 0 {
                        format!(
                            "The tournament will begin in {mins} minute{} and {secs} second{}.",
                            s(mins),
                            s(secs)
                        )
                    } else if mins > 0 {
                        format!("The tournament will begin in {mins} minute{}.", s(mins))
                    } else {
                        format!("The tournament will begin in {secs} second{}.", s(secs))
                    }
                } else {
                    "The tournament will begin shortly.".to_owned()
                };
                html_at(&mut ops, 25, 92, 250, 40, text);
            } else {
                right_arrow(&mut ops, 25, 89, to_button_id(0, 2), Some("Rounds"));
            }
        }
        BracketType::RulesInfo => {
            let (base_title, flavors, changes) = ruleset_view(&t);
            ops.push(Op::Page);
            ops.push(Op::Background(
                0,
                0,
                300,
                60 + 18
                    + 20
                    + 20
                    + 20
                    + 8
                    + 20
                    + flavors.len() as i32 * 18
                    + 4
                    + 20
                    + changes.len() as i32 * 22
                    + 6,
                9380,
            ));
            left_arrow(&mut ops, 25, 11, to_button_id(0, 0));
            html_at(&mut ops, 25, 35, 250, 20, center("Rules"));
            let mut y = 53;
            html_at(
                &mut ops,
                35,
                y,
                190,
                20,
                format!("Grouping: {}", group_text(&t)),
            );
            y += 20;
            html_at(
                &mut ops,
                35,
                y,
                190,
                20,
                format!("Tiebreaker: {}", tie_text(&t)),
            );
            y += 20;
            html_at(&mut ops, 35, y, 240, 20, sudden_death_text(&t));
            y += 20;
            y += 8;
            html_at(&mut ops, 35, y, 190, 20, format!("Ruleset: {base_title}"));
            y += 20;
            for f in &flavors {
                html_at(&mut ops, 35, y, 190, 20, format!(" + {f}"));
                y += 18;
            }
            y += 4;
            if !changes.is_empty() {
                html_at(&mut ops, 35, y, 190, 20, "Modifications:".to_owned());
                y += 20;
                for (name, on) in &changes {
                    if !name.is_empty() {
                        ops.push(Op::Image(35, y, if *on { 0xD3 } else { 0xD2 }));
                        html_at(&mut ops, 60, y, 165, 22, name.clone());
                    }
                    y += 22;
                }
            } else {
                html_at(&mut ops, 35, y, 190, 20, "Modifications: None".to_owned());
            }
        }
        // `StartPage` reads `_list.Count` of a null list.
        BracketType::ParticipantList | BracketType::RoundList => return None,
        BracketType::RoundInfo => {
            if matches!(obj, Some(Obj::Level(_))) {
                return None;
            }
            ops.push(Op::Page);
            ops.push(Op::Background(0, 0, 300, 300, 9380));
            left_arrow(&mut ops, 25, 11, to_button_id(0, 2));
            html_at(&mut ops, 25, 35, 250, 20, center("Rounds"));
        }
        BracketType::ParticipantInfo => {
            let Some(Obj::Part(p)) = obj else {
                return Some(ops);
            };
            let part = t.parts.get(p)?.clone();
            ops.push(Op::Page);
            ops.push(Op::Background(
                0,
                0,
                300,
                60 + 18 + 20 + part.players.len() as i32 * 18 + 20 + 20 + 160,
                9380,
            ));
            left_arrow(&mut ops, 25, 11, to_button_id(0, 1));
            html_at(&mut ops, 25, 35, 250, 20, center("Participants"));
            let mut y = 53;
            html_at(
                &mut ops,
                25,
                y,
                200,
                20,
                (if part.players.len() == 1 {
                    "Players"
                } else {
                    "Team"
                })
                .to_owned(),
            );
            y += 20;
            for (i, &mob) in part.players.iter().enumerate() {
                let mut n = name(w, mob);
                if t.tourney_type != TourneyType::Standard
                    && let Some(e) = duel_player_eliminated(w, mob)
                {
                    n = color(&n, if e { 0x663333 } else { 0x336666 });
                }
                right_arrow(&mut ops, 35, y, to_button_id(4, i as i32), Some(&n));
                y += 18;
            }
            html_at(
                &mut ops,
                25,
                y,
                200,
                20,
                format!(
                    "Free Advances: {}",
                    if part.free_advances == 0 {
                        "None".to_owned()
                    } else {
                        part.free_advances.to_string()
                    }
                ),
            );
            y += 20;
            html_at(&mut ops, 25, y, 200, 20, "Log:".to_owned());
            y += 20;
            let mut sb = part.log.join("<br>");
            if sb.is_empty() {
                sb.push_str("Nothing logged yet.");
            }
            ops.push(Op::Html(25, y, 250, 150, color(&sb, BLACK_COLOR32), true));
        }
        BracketType::PlayerInfo => {
            ops.push(Op::Page);
            ops.push(Op::Background(0, 0, 300, 300, 9380));
            left_arrow(&mut ops, 25, 11, to_button_id(0, 3));
            html_at(&mut ops, 25, 35, 250, 20, center("Participants"));
            let Some(Obj::Mob(mob)) = obj else {
                return Some(ops);
            };
            let lad = super::ladder::instance(w);
            let entry = lad.and_then(|l| super::ladder::find(w, l, mob));
            html_at(&mut ops, 25, 53, 250, 20, format!("Name: {}", name(w, mob)));
            let guild = w
                .mobile(mob)
                .and_then(|m| m.guild)
                .and_then(|g| muo_server::guild::guild(w, g))
                .map(|g| format!("{} [{}]", g.name, g.abbreviation));
            html_at(
                &mut ops,
                25,
                73,
                250,
                20,
                format!("Guild: {}", guild.unwrap_or_else(|| "None".to_owned())),
            );
            html_at(
                &mut ops,
                25,
                93,
                250,
                20,
                format!(
                    "Rank: {}",
                    entry
                        .as_ref()
                        .map_or("N/A".to_owned(), |e| super::speech::rank(
                            e.index as i32 + 1
                        ))
                ),
            );
            html_at(
                &mut ops,
                25,
                113,
                250,
                20,
                format!(
                    "Level: {}",
                    entry
                        .as_ref()
                        .map_or(0, |e| super::ladder::get_level(e.experience))
                ),
            );
            html_at(
                &mut ops,
                25,
                133,
                250,
                20,
                format!(
                    "Wins: {}",
                    super::speech::n0(entry.as_ref().map_or(0, |e| e.wins))
                ),
            );
            html_at(
                &mut ops,
                25,
                153,
                250,
                20,
                format!(
                    "Losses: {}",
                    super::speech::n0(entry.as_ref().map_or(0, |e| e.losses))
                ),
            );
        }
        BracketType::MatchInfo => {
            let Some(Obj::Match(l, i)) = obj else {
                return Some(ops);
            };
            let m = t.levels.get(l)?.matches.get(i)?.clone();
            let ct = if t.tourney_type == TourneyType::FreeForAll {
                2
            } else {
                m.participants.len() as i32
            };
            ops.push(Op::Page);
            ops.push(Op::Background(
                0,
                0,
                300,
                60 + 18 + 20 + 20 + 20 + ct * 18 + 6,
                9380,
            ));
            left_arrow(&mut ops, 25, 11, to_button_id(0, 5));
            html_at(&mut ops, 25, 35, 250, 20, center("Rounds"));
            let winner = m.winner.map_or("N/A".to_owned(), |p| t.name_list(w, p));
            html_at(&mut ops, 25, 53, 250, 20, format!("Winner: {winner}"));
            let in_progress = tn::match_in_progress(w, &m);
            html_at(
                &mut ops,
                25,
                73,
                250,
                20,
                format!(
                    "State: {}",
                    if in_progress {
                        "In progress"
                    } else if m.context.is_some() {
                        "Complete"
                    } else {
                        "Waiting"
                    }
                ),
            );
            html_at(&mut ops, 25, 93, 250, 20, "Participants:".to_owned());
            if t.tourney_type == TourneyType::Standard {
                for (i, &p) in m.participants.iter().enumerate() {
                    right_arrow(
                        &mut ops,
                        25,
                        113 + i as i32 * 18,
                        to_button_id(6, i as i32),
                        Some(&t.name_list(w, p)),
                    );
                }
            } else if has_team_labels(&t) {
                for (i, &p) in m.participants.iter().enumerate() {
                    let label = team_label(w, &t, i, t.parts[p].players.len());
                    right_arrow(
                        &mut ops,
                        25,
                        113 + i as i32 * 18,
                        to_button_id(6, i as i32),
                        Some(&label),
                    );
                }
            } else if t.tourney_type == TourneyType::FreeForAll {
                html_at(&mut ops, 25, 113, 250, 20, "Free For All".to_owned());
            }
        }
    }
    Some(ops)
}

/// `pm.DuelPlayer?.Eliminated`.
fn duel_player_eliminated(w: &World, mob: MobileId) -> Option<bool> {
    let id = super::duel_context_of(w, mob)?;
    Some(super::ctx(w, id)?.find(mob)?.eliminated)
}

impl DynamicGumpLogic for TournamentBracketGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.TournamentBracketGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        for op in &self.ops {
            match op {
                Op::Page => b.add_page(0),
                Op::Background(x, y, w, h, id) => b.add_background(*x, *y, *w, *h, *id),
                Op::Html(x, y, w, h, text, scroll) => {
                    b.add_html(*x, *y, *w, *h, text, false, *scroll)
                }
                Op::Button(x, y, n, p, id) => b.add_button(*x, *y, *n, *p, *id),
                Op::Image(x, y, id) => b.add_image(*x, *y, *id, 0),
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let bid = info.button_id;
        if bid < 1 {
            return;
        }
        let (ty, index) = ((bid - 1) % 7, (bid - 1) / 7);
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(t) = tn::get(w, self.ctrl).cloned() else {
            return;
        };

        match ty {
            0 => match index {
                0 => (self.ty, self.obj) = (BracketType::Index, None),
                1 => (self.ty, self.obj) = (BracketType::ParticipantList, None),
                2 => (self.ty, self.obj) = (BracketType::RoundList, None),
                4 => (self.ty, self.obj) = (BracketType::RulesInfo, None),
                3 => {
                    let Some(Obj::Mob(mob)) = self.obj else {
                        return;
                    };
                    let Some(&p) = t
                        .participants
                        .iter()
                        .find(|p| t.parts[**p].players.contains(&mob))
                    else {
                        return;
                    };
                    (self.ty, self.obj) = (BracketType::ParticipantInfo, Some(Obj::Part(p)));
                }
                5 => {
                    if let Some(Obj::Match(l, _)) = self.obj {
                        (self.ty, self.obj) = (BracketType::RoundInfo, Some(Obj::Level(l)));
                    }
                }
                _ => {}
            },
            // The list is always null: paging returns (`_list != null` fails) or throws.
            1..=3 => return,
            4 => {
                let Some(Obj::Part(p)) = self.obj else { return };
                if self.ty != BracketType::ParticipantInfo {
                    return;
                }
                let Some(&mob) = t.parts[p].players.get(index as usize) else {
                    return;
                };
                (self.ty, self.obj) = (BracketType::PlayerInfo, Some(Obj::Mob(mob)));
            }
            5 => {
                let Some(Obj::Level(l)) = self.obj else {
                    return;
                };
                if self.ty != BracketType::RoundInfo {
                    return;
                }
                let Some(level) = t.levels.get(l) else { return };
                if index == 0 {
                    if let Some(fa) = level.free_advance {
                        (self.ty, self.obj) = (BracketType::ParticipantInfo, Some(Obj::Part(fa)));
                    }
                } else if index >= 1 && index as usize <= level.matches.len() {
                    (self.ty, self.obj) = (
                        BracketType::MatchInfo,
                        Some(Obj::Match(l, index as usize - 1)),
                    );
                }
            }
            6 => {
                let Some(Obj::Match(l, i)) = self.obj else {
                    return;
                };
                if self.ty != BracketType::MatchInfo {
                    return;
                }
                let Some(&p) = t
                    .levels
                    .get(l)
                    .and_then(|l| l.matches.get(i))
                    .and_then(|m| m.participants.get(index as usize))
                else {
                    return;
                };
                (self.ty, self.obj) = (BracketType::ParticipantInfo, Some(Obj::Part(p)));
                display_bracket(w, from, self.ctrl, self.ty, self.obj);
            }
            _ => {}
        }
        display_bracket(w, from, self.ctrl, self.ty, self.obj);
    }
}
