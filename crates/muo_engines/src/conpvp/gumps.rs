//! The gumps of the challenge flow: `AcceptDuelGump`, `BeginGump`, `DuelContextGump`, `ParticipantGump`,
//! `PickRulesetGump`, `RulesetGump`, `ReadyUpGump`, `ReadyGump` and the `PreferencesGump`.
//!
//! A gump is addressed to a duel by the context's id (and a participant by its `uid`), so a gump that outlives
//! its context (or a participant removed meanwhile) finds nothing and does nothing, like a context that
//! is no longer `Registered`. "Refresh via this" (`from.SendGump(this)`) sends the same gump logic again.
//!
//! `PickRulesetGump`/`RulesetGump` edit the ruleset of a duel context, or of a tournament controller when there is no
//! context (`TournamentController.OnDoubleClick`). The tournament gumps are in `tournament_gumps`.

use std::collections::HashMap;

use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, GumpButtonType, RelayInfo,
    StaticGumpLogic, StaticGumpObj, close_gump, send_gump_to_mobile,
};
use muo_primitives::TimeSpan;
use muo_primitives::html;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::net_state::NetStateId;
use muo_server::target::{self, Target, TargetFlags, Targeted};
use muo_server::timer;

use super::flow::{self, name};
use super::ruleset::RulesetLayout;
use super::{ctx, ctx_mut, ladder, st_mut};

const LABEL_COLOR32: i32 = 0xFFFFFF;
const BLACK_COLOR32: i32 = 0x000008;

pub(super) fn ns_of(w: &World, m: MobileId) -> Option<NetStateId> {
    w.mobile(m).and_then(|x| x.net_state)
}

pub(super) fn mobile_of(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_states.get(&ns).and_then(|n| n.mobile)
}

pub(super) fn send_dynamic<T: DynamicGumpLogic>(
    w: &mut World,
    to: MobileId,
    x: i32,
    y: i32,
    logic: T,
    singleton: bool,
) {
    send_gump_to_mobile(w, to, Box::new(DynamicGumpObj::new(x, y, logic)), singleton);
}

fn msg(w: &mut World, m: MobileId, text: &str) {
    muo_server::mobile::part2::messages::send_message(w, m, text);
}

pub(super) fn msg_hue(w: &mut World, m: MobileId, hue: i32, text: &str) {
    muo_server::mobile::part2::messages::send_message_hue(w, m, hue, text);
}

fn close<L: 'static>(w: &mut World, m: MobileId) {
    if let Some(ns) = ns_of(w, m) {
        close_gump::<L>(w, ns);
    }
}

pub(super) fn center(text: &str) -> String {
    format!("<CENTER>{text}</CENTER>")
}

pub(super) fn golden_button(b: &mut DynamicGumpBuilder, x: i32, y: i32, bid: i32) {
    b.add_button(x, y, 0xD2, 0xD2, bid);
    b.add_button(x + 3, y + 3, 0xD8, 0xD8, bid);
}

fn golden_button_labeled(b: &mut DynamicGumpBuilder, x: i32, y: i32, bid: i32, text: &str) {
    golden_button(b, x, y, bid);
    b.add_html(x + 25, y, 200, 20, text, false, false);
}

/// Text with a one pixel black outline (`AddHtml` four times in black, then once in colour).
fn outlined(
    b: &mut DynamicGumpBuilder,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    black: &str,
    shown: &str,
) {
    b.add_html(x - 1, y, width, height, black, false, false);
    b.add_html(x + 1, y, width, height, black, false, false);
    b.add_html(x, y - 1, width, height, black, false, false);
    b.add_html(x, y + 1, width, height, black, false, false);
    b.add_html(x, y, width, height, shown, false, false);
}

// ---- closing --------------------------------------------------------------------------------------------------------

/// `CloseGump<BeginGump>()`.
pub fn close_begin(w: &mut World, m: MobileId) {
    close::<BeginGump>(w, m);
}

/// `CloseAllGumps(pl)`.
pub fn close_all_gumps_of(w: &mut World, m: MobileId) {
    close::<BeginGump>(w, m);
    close::<DuelContextGump>(w, m);
    close::<ParticipantGump>(w, m);
    close::<PickRulesetGump>(w, m);
    close::<ReadyGump>(w, m);
    close::<ReadyUpGump>(w, m);
    close::<RulesetGump>(w, m);
}

/// The three `RejectReady` closes.
pub fn close_reject_gumps(w: &mut World, m: MobileId) {
    close::<DuelContextGump>(w, m);
    close::<ReadyUpGump>(w, m);
    close::<ReadyGump>(w, m);
}

/// The three closes of `SendBeginGump(10)`.
pub fn close_before_begin(w: &mut World, m: MobileId) {
    close::<ReadyGump>(w, m);
    close::<ReadyUpGump>(w, m);
    close::<BeginGump>(w, m);
}

// ---- BeginGump -----------------------------------------------------------------------------------------------------

/// `BeginGump`.
pub struct BeginGump;

impl StaticGumpLogic for BeginGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.BeginGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        let offset = 50;
        b.add_background(1, 1, 398, 202 - offset, 3600);
        b.add_image_tiled(16, 15, 369, 173 - offset, 3604);
        b.add_alpha_region(16, 15, 369, 173 - offset);
        b.add_image(215, -43, 0xEE40, 0);

        let black = html::center_rgb("Duel Countdown", BLACK_COLOR32);
        outlined(
            b,
            22,
            22,
            294,
            20,
            &black,
            &html::center_rgb("Duel Countdown", LABEL_COLOR32),
        );

        let text = "The arranged duel is about to begin. During this countdown period you may not cast spells and you may not move. This message will close automatically when the period ends.";
        outlined(
            b,
            22,
            50,
            294,
            80,
            &html::color_rgb(text, BLACK_COLOR32),
            &html::color_rgb(text, 0xFFCC66),
        );
        b.add_button(314 - 50, 157 - offset, 247, 248, 1);
    }
}

/// `BeginGump.DisplayTo(from, count)`.
pub fn display_begin(w: &mut World, m: MobileId) {
    if ns_of(w, m).is_none() {
        return;
    }
    send_gump_to_mobile(w, m, Box::new(StaticGumpObj::new(50, 50, BeginGump)), false);
}

// ---- ReadyGump -----------------------------------------------------------------------------------------------------

/// `ReadyGump`.
pub struct ReadyGump {
    count: i32,
    parts: Vec<PartView>,
}

/// `ReadyGump.DisplayTo(from, context, count)`.
pub fn display_ready(w: &mut World, m: MobileId, id: u32, count: i32) {
    if ns_of(w, m).is_none() || ctx(w, id).is_none() {
        return;
    }
    let g = ReadyGump {
        count,
        parts: parts_view(w, id),
    };
    send_dynamic(w, m, 50, 50, g, true);
}

/// The names and ready flags of the players of a participant, read when the gump is laid out.
#[derive(Clone, Debug, Default)]
struct PartView {
    slots: Vec<Option<(String, bool)>>,
}

fn parts_view(w: &World, id: u32) -> Vec<PartView> {
    ctx(w, id).map_or_else(Vec::new, |c| {
        c.participants
            .iter()
            .map(|p| PartView {
                slots: p
                    .players
                    .iter()
                    .map(|s| s.map(|d| (name(w, d.mobile), d.ready)))
                    .collect(),
            })
            .collect()
    })
}

impl DynamicGumpLogic for ReadyGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.ReadyGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();
        b.set_no_move();
        b.add_page(0);
        let mut height = 25 + 20;
        for p in &self.parts {
            height += 4;
            if p.slots.len() > 1 {
                height += 22;
            }
            height += p.slots.len() as i32 * 22;
        }
        height += 25;
        b.add_background(0, 0, 260, height, 9250);
        b.add_background(10, 10, 240, height - 20, 0xDAC);

        if self.count == -1 {
            b.add_html(35, 25, 190, 20, &center("Ready"), false, false);
        } else {
            b.add_html(35, 25, 190, 20, &center("Starting"), false, false);
            b.add_html(
                35,
                25,
                190,
                20,
                &format!("<DIV ALIGN=RIGHT>{}", self.count),
                false,
                false,
            );
        }

        let mut y = 25 + 20;
        for (i, p) in self.parts.iter().enumerate() {
            y += 4;
            let mut is_all_ready = true;
            let y_store = y;
            let mut offset = 0;
            if p.slots.len() > 1 {
                b.add_html(
                    35 + 14,
                    y,
                    176,
                    20,
                    &format!("Participant #{}", i + 1),
                    false,
                    false,
                );
                y += 22;
                offset = 10;
            }
            for s in &p.slots {
                if s.as_ref().is_some_and(|s| s.1) {
                    b.add_image(35 + offset, y + 4, 0x939, 0);
                } else {
                    b.add_image(35 + offset, y + 4, 0x938, 0);
                    is_all_ready = false;
                }
                let n = s.as_ref().map_or("(Empty)", |s| s.0.as_str());
                b.add_html(35 + offset + 14, y, 166, 20, n, false, false);
                y += 22;
            }
            if p.slots.len() > 1 {
                b.add_image(35, y_store + 4, if is_all_ready { 0x939 } else { 0x938 }, 0);
            }
        }
    }
}

// ---- ReadyUpGump ----------------------------------------------------------------------------------------------------

/// `ReadyUpGump`.
pub struct ReadyUpGump {
    ctx: u32,
    rematch: bool,
    parts: Vec<PartView>,
    /// `Ruleset.Base.Title`, the flavor titles and the modifications.
    rules: RulesView,
}

/// What page 2 of `ReadyUpGump` (and the read-only rules) shows: the base ruleset, flavors and modifications.
#[derive(Clone, Debug, Default)]
struct RulesView {
    base_title: String,
    flavors: Vec<String>,
    /// (name from `FindByIndex`, enabled).
    changes: Vec<(String, bool)>,
}

fn rules_view(r: &super::Ruleset) -> RulesView {
    let layout = &r.layout;
    let base = r.base.and_then(|b| layout.defaults.get(b));
    let mut defs = base
        .map(|b| b.options.clone())
        .unwrap_or_else(|| vec![false; r.options.len()]);
    let mut flavors = Vec::new();
    if !r.flavors.is_empty() {
        for &f in &r.flavors {
            for (d, v) in defs.iter_mut().zip(&layout.flavors[f].options) {
                *d |= *v;
            }
            flavors.push(layout.flavors[f].title.clone().unwrap_or_default());
        }
    }
    let mut changes = Vec::new();
    for (i, (d, o)) in defs.iter().zip(&r.options).enumerate() {
        if d != o
            && let Some(n) = layout.find_by_index(i)
        {
            changes.push((n, *o));
        }
    }
    RulesView {
        base_title: base.and_then(|b| b.title.clone()).unwrap_or_default(),
        flavors,
        changes,
    }
}

/// `ReadyUpGump.DisplayTo(from, context)`.
pub fn display_ready_up(w: &mut World, m: MobileId, id: u32) {
    if ns_of(w, m).is_none() {
        return;
    }
    let Some(c) = ctx(w, id) else { return };
    let g = ReadyUpGump {
        ctx: id,
        rematch: c.rematch,
        parts: parts_view(w, id),
        rules: rules_view(&c.ruleset),
    };
    send_dynamic(w, m, 50, 50, g, true);
}

impl DynamicGumpLogic for ReadyUpGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.ReadyUpGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();
        b.add_page(0);
        if self.rematch {
            let height = 25 + 20 + 10 + 22 + 25;
            b.add_background(0, 0, 210, height, 9250);
            b.add_background(10, 10, 190, height - 20, 0xDAC);
            b.add_html(35, 25, 140, 20, &center("Rematch?"), false, false);
            b.add_button(35, 55, 247, 248, 1);
            b.add_button(115, 55, 242, 241, 2);
            return;
        }

        b.add_page(1);
        let mut height = 25 + 20;
        for p in &self.parts {
            height += 4;
            if p.slots.len() > 1 {
                height += 22;
            }
            height += p.slots.len() as i32 * 22;
        }
        height += 10 + 22 + 25;
        b.add_background(0, 0, 260, height, 9250);
        b.add_background(10, 10, 240, height - 20, 0xDAC);
        b.add_html(35, 25, 190, 20, &center("Participants"), false, false);

        let mut y = 20 + 25;
        for (i, p) in self.parts.iter().enumerate() {
            y += 4;
            let mut offset = 0;
            if p.slots.len() > 1 {
                b.add_html(35, y, 176, 20, &format!("Team #{}", i + 1), false, false);
                y += 22;
                offset = 10;
            }
            for s in &p.slots {
                let n = s.as_ref().map_or("(Empty)", |s| s.0.as_str());
                b.add_html(35 + offset, y, 166, 20, n, false, false);
                y += 22;
            }
        }
        y += 8;
        b.add_html(35, y, 176, 20, "Continue?", false, false);
        y -= 2;
        b.add_button_type(102, y, 247, 248, 0, GumpButtonType::Page, 2);
        b.add_button(169, y, 242, 241, 2);

        b.add_page(2);
        let r = &self.rules;
        let mut height = 25 + 20 + 5 + 20 + 20 + 4;
        height += r.flavors.len() as i32 * 18;
        height += r.changes.len() as i32 * 22;
        height += 10 + 22 + 25;
        b.add_background(0, 0, 260, height, 9250);
        b.add_background(10, 10, 240, height - 20, 0xDAC);
        b.add_html(35, 25, 190, 20, &center("Rules"), false, false);
        b.add_html(
            35,
            50,
            190,
            20,
            &format!("Set: {}", r.base_title),
            false,
            false,
        );

        let mut y = 70;
        for f in &r.flavors {
            b.add_html(35, y, 190, 20, &format!(" + {f}"), false, false);
            y += 18;
        }
        y += 4;
        if !r.changes.is_empty() {
            b.add_html(35, y, 190, 20, "Modifications:", false, false);
            y += 20;
            for (n, on) in &r.changes {
                b.add_image(35, y, if *on { 0xD3 } else { 0xD2 }, 0);
                b.add_html(60, y, 165, 22, n, false, false);
                y += 22;
            }
        } else {
            b.add_html(35, y, 190, 20, "Modifications: None", false, false);
            y += 20;
        }
        y += 8;
        b.add_html(35, y, 176, 20, "Continue?", false, false);
        y -= 2;
        b.add_button(102, y, 247, 248, 1);
        b.add_button(169, y, 242, 241, 3);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let id = self.ctx;
        let Some(c) = ctx(w, id) else { return };
        if !c.registered || !c.ready_wait {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        match info.button_id {
            1 => {
                // okay
                if super::duel_context_of(w, from).is_none() {
                    return;
                }
                if let Some(p) = ctx_mut(w, id).and_then(|c| c.find_mut(from)) {
                    p.ready = true;
                }
                flow::send_ready_gump(w, id);
            }
            2 => flow::reject_ready(w, id, from, Some("participants")),
            3 => flow::reject_ready(w, id, from, Some("rules")),
            _ => {}
        }
    }
}

// ---- DuelContextGump -------------------------------------------------------------------------------------------------

/// `DuelContextGump`.
pub struct DuelContextGump {
    pub ctx: u32,
    /// (`p.Count`, `p.FilledSlots`, `p.NameList`) of each participant.
    parts: Vec<(usize, usize, String)>,
}

fn name_list(w: &World, p: &super::Participant) -> String {
    let names: Vec<String> = p
        .players
        .iter()
        .flatten()
        .map(|d| name(w, d.mobile))
        .collect();
    if names.is_empty() {
        "Empty".to_owned()
    } else {
        names.join(", ")
    }
}

/// `DuelContextGump.DisplayTo(from, context)`.
pub fn display_duel_context(w: &mut World, from: MobileId, id: u32) {
    if ns_of(w, from).is_none() {
        return;
    }
    close::<RulesetGump>(w, from);
    close::<ParticipantGump>(w, from);
    let Some(c) = ctx(w, id) else { return };
    let parts = c
        .participants
        .iter()
        .map(|p| (p.count(), p.filled_slots(), name_list(w, p)))
        .collect();
    send_dynamic(w, from, 50, 50, DuelContextGump { ctx: id, parts }, false);
}

impl DynamicGumpLogic for DuelContextGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.DuelContextGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let count = self.parts.len().max(3) as i32;
        let height = 35 + 10 + 22 + 30 + 22 + 22 + 2 + count * 22 + 2 + 30;
        b.add_page(0);
        b.add_background(0, 0, 300, height, 9250);
        b.add_background(10, 10, 280, height - 20, 0xDAC);
        b.add_html(35, 25, 230, 20, &center("Duel Setup"), false, false);
        let x = 35;
        let mut y = 47;
        golden_button_labeled(b, x, y, 1, "Rules");
        y += 22;
        golden_button_labeled(b, x, y, 2, "Start");
        y += 22;
        golden_button_labeled(b, x, y, 3, "Add Participant");
        y += 30;
        b.add_html(35, y, 230, 20, &center("Participants"), false, false);
        y += 22;
        for (i, (count, filled, names)) in self.parts.iter().enumerate() {
            let text = if *count == 1 {
                format!("Player {}: {}", 1 + i, names)
            } else {
                format!("Team {}: {}/{}: {}", 1 + i, filled, count, names)
            };
            golden_button_labeled(b, x, y, 4 + i as i32, &text);
            y += 22;
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let id = self.ctx;
        if !ctx(w, id).is_some_and(|c| c.registered) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let mut index = info.button_id;
        match index {
            -1 => {}
            0 => flow::unregister(w, id, false),
            1 => display_pick_ruleset(w, from, Some(id), None),
            2 => {
                if flow::check_full(w, id) {
                    flow::close_all_gumps(w, id);
                    flow::send_ready_up_gump(w, id);
                } else {
                    msg(
                        w,
                        from,
                        "You cannot start the duel before all participating players have been assigned.",
                    );
                    display_duel_context_again(w, from, id, self);
                }
            }
            3 => {
                let n = ctx(w, id).map_or(0, |c| c.participants.len());
                if n < 10 {
                    let p = st_mut(w).new_participant(1);
                    if let Some(c) = ctx_mut(w, id) {
                        c.participants.push(p);
                    }
                } else {
                    msg(
                        w,
                        from,
                        "The number of participating parties may not be increased further.",
                    );
                }
                // `from.SendGump(this)`: the participant list changed, so it is laid out again.
                display_duel_context_fresh(w, from, id);
            }
            _ => {
                index -= 4;
                let uid = ctx(w, id)
                    .and_then(|c| c.participants.get(index as usize).map(|p| p.uid))
                    .filter(|_| index >= 0);
                if let Some(uid) = uid {
                    display_participant(w, from, id, uid);
                }
            }
        }
    }
}

/// `from.SendGump(this)` without a change: the same gump.
fn display_duel_context_again(w: &mut World, from: MobileId, id: u32, g: &DuelContextGump) {
    send_dynamic(
        w,
        from,
        50,
        50,
        DuelContextGump {
            ctx: id,
            parts: g.parts.clone(),
        },
        false,
    );
}

/// `from.SendGump(this)` after a change: the gump is laid out again.
fn display_duel_context_fresh(w: &mut World, from: MobileId, id: u32) {
    let Some(c) = ctx(w, id) else { return };
    let parts = c
        .participants
        .iter()
        .map(|p| (p.count(), p.filled_slots(), name_list(w, p)))
        .collect();
    send_dynamic(w, from, 50, 50, DuelContextGump { ctx: id, parts }, false);
}

// ---- ParticipantGump --------------------------------------------------------------------------------------------------

/// `ParticipantGump`.
pub struct ParticipantGump {
    pub ctx: u32,
    pub participant: u32,
    /// The names in the slots (`None` is empty).
    slots: Vec<Option<String>>,
}

/// `ParticipantGump.DisplayTo(from, context, p)`.
pub fn display_participant(w: &mut World, from: MobileId, id: u32, uid: u32) {
    if ns_of(w, from).is_none() {
        return;
    }
    close::<RulesetGump>(w, from);
    close::<DuelContextGump>(w, from);
    let Some(p) = ctx(w, id).and_then(|c| c.participant_index(uid).map(|i| &c.participants[i]))
    else {
        return;
    };
    let slots = p
        .players
        .iter()
        .map(|s| s.map(|d| name(w, d.mobile)))
        .collect();
    send_dynamic(
        w,
        from,
        50,
        50,
        ParticipantGump {
            ctx: id,
            participant: uid,
            slots,
        },
        false,
    );
}

impl DynamicGumpLogic for ParticipantGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.ParticipantGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let count = self.slots.len().max(4) as i32;
        b.add_page(0);
        let height = 35 + 10 + 22 + 22 + 30 + 22 + 2 + count * 22 + 2 + 30;
        b.add_background(0, 0, 300, height, 9250);
        b.add_background(10, 10, 280, height - 20, 0xDAC);
        b.add_button(240, 25, 0xFB1, 0xFB3, 3);
        b.add_html(35, 25, 230, 20, &center("Participant Setup"), false, false);
        let x = 35;
        let mut y = 47;
        b.add_html(
            x,
            y,
            200,
            20,
            &format!("Team Size: {}", self.slots.len()),
            false,
            false,
        );
        y += 22;
        golden_button_labeled(b, x + 20, y, 1, "Increase");
        y += 22;
        golden_button_labeled(b, x + 20, y, 2, "Decrease");
        y += 30;
        b.add_html(35, y, 230, 20, &center("Players"), false, false);
        y += 22;
        for (i, s) in self.slots.iter().enumerate() {
            golden_button_labeled(
                b,
                x,
                y,
                5 + i as i32,
                &format!("{}: {}", 1 + i, s.as_deref().unwrap_or("Empty")),
            );
            y += 22;
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let id = self.ctx;
        if !ctx(w, id).is_some_and(|c| c.registered) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let uid = self.participant;
        let Some(pi) = ctx(w, id).and_then(|c| c.participant_index(uid)) else {
            return;
        };
        let (count, filled, nparts) = {
            let c = ctx(w, id).expect("context");
            (
                c.participants[pi].count(),
                c.participants[pi].filled_slots(),
                c.participants.len(),
            )
        };
        let mut bid = info.button_id;
        match bid {
            0 => {
                display_duel_context(w, from, id);
                return;
            }
            1 if count < 8 => resize(w, id, pi, count + 1),
            1 => msg(w, from, "You may not raise the team size any further."),
            2 if count > 1 && count > filled => resize(w, id, pi, count - 1),
            2 => msg(w, from, "You may not lower the team size any further."),
            3 if filled > 0 => msg(
                w,
                from,
                "There is at least one currently active player. You must remove them first.",
            ),
            3 if nparts > 2 => {
                if let Some(c) = ctx_mut(w, id) {
                    c.participants.remove(pi);
                }
                display_duel_context(w, from, id);
                return;
            }
            3 => msg(
                w,
                from,
                "Duels must have at least two participating parties.",
            ),
            _ => {
                bid -= 5;
                if bid >= 0 && (bid as usize) < count {
                    let slot = bid as usize;
                    let occupant = ctx(w, id).and_then(|c| c.participants[pi].players[slot]);
                    match occupant {
                        None => {
                            begin_participant_target(w, from, id, uid, slot);
                            return;
                        }
                        Some(p) => {
                            msg(w, p.mobile, "You have been removed from the duel.");
                            super::release_mobile(w, p.mobile);
                            if let Some(c) = ctx_mut(w, id) {
                                c.participants[pi].players[slot] = None;
                            }
                            msg(w, from, "They have been removed from the duel.");
                        }
                    }
                } else {
                    return;
                }
            }
        }
        display_participant(w, from, id, uid);
    }
}

fn resize(w: &mut World, id: u32, pi: usize, n: usize) {
    if let Some(c) = ctx_mut(w, id) {
        c.participants[pi].resize(n);
    }
}

/// `from.Target = new ParticipantTarget(context, p, index)`.
fn begin_participant_target(w: &mut World, from: MobileId, id: u32, uid: u32, slot: usize) {
    let mut t = Target::new(w, 12, false, TargetFlags::NONE);
    t.handlers.on_target = Some(std::rc::Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            participant_on_target(w, from, id, uid, slot, targeted)
        },
    ));
    t.handlers.on_target_finish = Some(std::rc::Rc::new(move |w: &mut World, from: MobileId| {
        display_participant(w, from, id, uid)
    }));
    let tid = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(tid));
    msg(w, from, "Target a player.");
}

fn participant_on_target(
    w: &mut World,
    from: MobileId,
    id: u32,
    uid: u32,
    slot: usize,
    targeted: Targeted,
) {
    let Some(c) = ctx(w, id) else { return };
    if !c.registered {
        return;
    }
    let Some(pi) = c.participant_index(uid) else {
        return;
    };
    if slot >= c.participants[pi].count() {
        return;
    }
    let Targeted::Entity(EntityId::Mobile(mob)) = targeted else {
        msg(w, from, "That is not a player.");
        return;
    };
    let player = w.mobile(mob).is_some_and(|m| m.player);
    let human = muo_content::creature::misc::body_is_human(w, mob);
    let n = name(w, mob);
    if !player {
        if human {
            muo_server::mobile::part2::speech::say_to_number(w, mob, from, 1005443, ""); // Nay, I would rather stay here and watch a nail rust.
        } else {
            muo_server::mobile::part2::speech::say_to_number(w, mob, from, 1005444, ""); // The creature ignores your offer.
        }
    } else if is_ignored(w, mob, from) || w.mobile(mob).is_some_and(|m| m.blessed) {
        msg(w, from, "They ignore your offer.");
    } else if muo_content::player_mobile::data(w, mob).is_none() {
        // `mob is not PlayerMobile pm`
    } else if super::duel_context_of(w, mob).is_some() {
        msg(
            w,
            from,
            &format!("{n} cannot fight because they are already assigned to another duel."),
        );
    } else if flow::check_combat(w, mob) {
        msg(
            w,
            from,
            &format!(
                "{n} cannot fight because they have recently been in combat with another player."
            ),
        );
    } else if ns_of(w, mob).is_some_and(|ns| muo_content::gumps::has_gump::<AcceptDuelGump>(w, ns))
    {
        msg(w, from, &format!("{n} has already been offered a duel."));
    } else {
        if !ctx(w, id).is_some_and(|c| c.participants[pi].contains(from)) {
            msg(w, from, &format!("You send a challenge to {n}."));
        } else {
            msg(w, from, &format!("You send an invitation to {n}."));
        }
        display_accept_duel(w, from, mob, id, uid, slot as i32);
    }
}

/// The ruleset a ruleset gump edits: the duel context's, or else the tournament's.
fn ruleset_of(w: &World, id: Option<u32>, tourney: Option<ItemId>) -> Option<&super::Ruleset> {
    match (id, tourney) {
        (Some(i), _) => ctx(w, i).map(|c| &c.ruleset),
        (None, Some(t)) => super::tournament::get(w, t).map(|t| &t.ruleset),
        _ => None,
    }
}

fn ruleset_mut(
    w: &mut World,
    id: Option<u32>,
    tourney: Option<ItemId>,
) -> Option<&mut super::Ruleset> {
    match (id, tourney) {
        (Some(i), _) => ctx_mut(w, i).map(|c| &mut c.ruleset),
        (None, Some(t)) => super::tournament::get_mut(w, t).map(|t| &mut t.ruleset),
        _ => None,
    }
}

// ---- PickRulesetGump -------------------------------------------------------------------------------------------------

/// `PickRulesetGump`: edits the ruleset of the duel context `ctx`.
pub struct PickRulesetGump {
    ctx: Option<u32>,
    /// The tournament controller whose ruleset is edited (no duel context then).
    tourney: Option<ItemId>,
    layout: std::sync::Arc<RulesetLayout>,
    /// (`Base` index, `Changed`, flavors in the ruleset).
    base: Option<usize>,
    changed: bool,
    flavors: Vec<usize>,
}

/// `PickRulesetGump.DisplayTo(from, context, ruleset)`.
pub fn display_pick_ruleset(
    w: &mut World,
    from: MobileId,
    id: Option<u32>,
    tourney: Option<ItemId>,
) {
    let (Some(_), Some(r)) = (ns_of(w, from), ruleset_of(w, id, tourney)) else {
        return;
    };
    let g = PickRulesetGump {
        ctx: id,
        tourney,
        layout: r.layout.clone(),
        base: r.base,
        changed: r.changed,
        flavors: r.flavors.clone(),
    };
    send_dynamic(w, from, 50, 50, g, false);
}

impl DynamicGumpLogic for PickRulesetGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.PickRulesetGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let defaults = &self.layout.defaults;
        let flavors = &self.layout.flavors;
        let height =
            25 + 20 + (defaults.len() as i32 + 1) * 22 + 6 + 20 + flavors.len() as i32 * 22 + 25;
        b.add_page(0);
        b.add_background(0, 0, 260, height, 9250);
        b.add_background(10, 10, 240, height - 20, 0xDAC);
        b.add_html(35, 25, 190, 20, &center("Rules"), false, false);

        let mut y = 25 + 20;
        for (i, cur) in defaults.iter().enumerate() {
            b.add_html(
                35 + 14,
                y,
                176,
                20,
                cur.title.as_deref().unwrap_or(""),
                false,
                false,
            );
            if self.base == Some(i) && !self.changed {
                b.add_image(35, y + 4, 0x939, 0);
            } else if self.base == Some(i) {
                b.add_button(35, y + 4, 0x93A, 0x939, 2 + i as i32);
            } else {
                b.add_button(35, y + 4, 0x938, 0x939, 2 + i as i32);
            }
            y += 22;
        }
        b.add_html(35 + 14, y, 176, 20, "Custom", false, false);
        b.add_button(
            35,
            y + 4,
            if self.changed { 0x939 } else { 0x938 },
            0x939,
            1,
        );
        y += 22;
        y += 6;
        b.add_html(35, y, 190, 20, &center("Flavors"), false, false);
        y += 20;
        for (i, cur) in flavors.iter().enumerate() {
            b.add_html(
                35 + 14,
                y,
                176,
                20,
                cur.title.as_deref().unwrap_or(""),
                false,
                false,
            );
            let bid = 2 + defaults.len() as i32 + i as i32;
            if self.flavors.contains(&i) {
                b.add_button(35, y + 4, 0x939, 0x938, bid);
            } else {
                b.add_button(35, y + 4, 0x938, 0x939, bid);
            }
            y += 22;
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if self
            .ctx
            .is_some_and(|c| !ctx(w, c).is_some_and(|c| c.registered))
        {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        match info.button_id {
            0 => {
                if let Some(c) = self.ctx {
                    display_duel_context(w, from, c);
                }
            }
            1 => display_ruleset(w, from, self.ctx, self.tourney, None, false),
            bid => {
                let ndefaults = self.layout.defaults.len() as i32;
                let nflavors = self.layout.flavors.len() as i32;
                let mut idx = bid - 2;
                if (0..ndefaults).contains(&idx) {
                    if let Some(r) = ruleset_mut(w, self.ctx, self.tourney) {
                        r.apply_default(idx as usize);
                    }
                    display_pick_ruleset(w, from, self.ctx, self.tourney); // refresh-via-this
                } else {
                    idx -= ndefaults;
                    if (0..nflavors).contains(&idx) {
                        if let Some(r) = ruleset_mut(w, self.ctx, self.tourney) {
                            let f = idx as usize;
                            if r.flavors.contains(&f) {
                                r.remove_flavor(f);
                            } else {
                                r.add_flavor(f);
                            }
                        }
                        display_pick_ruleset(w, from, self.ctx, self.tourney);
                    }
                }
            }
        }
    }
}

// ---- RulesetGump ----------------------------------------------------------------------------------------------------

/// `RulesetGump`: one page (node) of the option tree.
pub struct RulesetGump {
    ctx: Option<u32>,
    tourney: Option<ItemId>,
    layout: std::sync::Arc<RulesetLayout>,
    page: usize,
    read_only: bool,
    /// The option bits of this page when the gump was laid out.
    enabled: Vec<bool>,
}

/// `RulesetGump.DisplayTo(from, ruleset, page, duelContext, readOnly)`: `page` `None` is the root.
pub fn display_ruleset(
    w: &mut World,
    from: MobileId,
    id: Option<u32>,
    tourney: Option<ItemId>,
    page: Option<usize>,
    read_only: bool,
) {
    if ns_of(w, from).is_none() {
        return;
    }
    let Some(r) = ruleset_of(w, id, tourney) else {
        return;
    };
    let layout = r.layout.clone();
    let page = page.unwrap_or_else(|| layout.root_id());
    let n = &layout.nodes[page];
    let enabled = r.options[n.offset..n.offset + n.options.len()].to_vec();
    close::<DuelContextGump>(w, from);
    close::<ParticipantGump>(w, from);
    let x = if read_only { 310 } else { 50 };
    send_dynamic(
        w,
        from,
        x,
        50,
        RulesetGump {
            ctx: id,
            tourney,
            layout,
            page,
            read_only,
            enabled,
        },
        false,
    );
}

impl DynamicGumpLogic for RulesetGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.RulesetGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        if self.read_only {
            b.set_no_move();
        }
        let page = &self.layout.nodes[self.page];
        let count = (page.children.len() + page.options.len()) as i32;
        b.add_page(0);
        let height = 35 + 10 + 2 + count * 22 + 2 + 30;
        b.add_background(0, 0, 260, height, 9250);
        b.add_background(10, 10, 240, height - 20, 0xDAC);
        b.add_html(35, 25, 190, 20, &center(&page.title), false, false);

        let x = 35;
        let mut y = 47;
        for (i, c) in page.children.iter().enumerate() {
            golden_button(b, x, y, 1 + i as i32);
            b.add_html(
                x + 25,
                y,
                250,
                22,
                &self.layout.nodes[*c].title,
                false,
                false,
            );
            y += 22;
        }
        for (i, o) in page.options.iter().enumerate() {
            let enabled = self.enabled[i];
            if self.read_only {
                b.add_image(x, y, if enabled { 0xD3 } else { 0xD2 }, 0);
            } else {
                b.add_checkbox(x, y, 0xD2, 0xD3, enabled, i as i32);
            }
            b.add_html(x + 25, y, 250, 22, o, false, false);
            y += 22;
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if self
            .ctx
            .is_some_and(|c| !ctx(w, c).is_some_and(|c| c.registered))
        {
            return;
        }
        let page = self.layout.nodes[self.page].clone();
        if !self.read_only
            && let Some(c) = ruleset_mut(w, self.ctx, self.tourney)
        {
            let mut opts = vec![false; page.options.len()];
            for &sid in &info.switches {
                if sid >= 0 && (sid as usize) < page.options.len() {
                    opts[sid as usize] = true;
                }
            }
            for (i, o) in opts.iter().enumerate() {
                if c.options[page.offset + i] != *o {
                    c.options[page.offset + i] = *o;
                    c.changed = true;
                }
            }
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let bid = info.button_id;
        if bid == 0 {
            if let Some(parent) = page.parent {
                display_ruleset(
                    w,
                    from,
                    self.ctx,
                    self.tourney,
                    Some(parent),
                    self.read_only,
                );
            } else if !self.read_only {
                display_pick_ruleset(w, from, self.ctx, self.tourney);
            }
        } else {
            let bid = bid - 1;
            if bid >= 0 && (bid as usize) < page.children.len() {
                display_ruleset(
                    w,
                    from,
                    self.ctx,
                    self.tourney,
                    Some(page.children[bid as usize]),
                    self.read_only,
                );
            }
        }
    }
}

// ---- AcceptDuelGump -------------------------------------------------------------------------------------------------------

/// `_ignoreLists`: who ignores whom until when, and which `AcceptDuelGump`s are still `_active`.
#[derive(Default)]
pub struct AcceptState {
    ignore: HashMap<MobileId, Vec<(MobileId, muo_primitives::DateTime)>>,
    active: HashMap<u32, bool>,
    next: u32,
}

const IGNORE_EXPIRE: TimeSpan = TimeSpan::from_millis(15 * 60_000);

fn accept(w: &mut World) -> &mut AcceptState {
    w.resources.get_mut::<AcceptState>()
}

/// `BeginIgnore(source, toIgnore)`.
pub fn begin_ignore(w: &mut World, source: MobileId, to_ignore: MobileId) {
    let now = w.now;
    let list = accept(w).ignore.entry(source).or_default();
    let mut i = 0;
    while i < list.len() {
        if list[i].0 == to_ignore {
            list[i].1 = now + IGNORE_EXPIRE;
            return;
        }
        if now >= list[i].1 {
            list.remove(i);
        } else {
            i += 1;
        }
    }
    list.push((to_ignore, now + IGNORE_EXPIRE));
}

/// `IsIgnored(source, check)`.
pub fn is_ignored(w: &mut World, source: MobileId, check: MobileId) -> bool {
    let now = w.now;
    let Some(list) = accept(w).ignore.get_mut(&source) else {
        return false;
    };
    let mut i = 0;
    while i < list.len() {
        if now >= list[i].1 {
            list.remove(i);
        } else if list[i].0 == check {
            return true;
        } else {
            i += 1;
        }
    }
    false
}

/// `AcceptDuelGump`.
pub struct AcceptDuelGump {
    key: u32,
    challenger: MobileId,
    challenged: MobileId,
    ctx: u32,
    participant: u32,
    slot: i32,
    /// `_participant.Contains(_challenger)` when it was laid out.
    joins_sides: bool,
    /// The question, with the challenger's name.
    text: String,
}

/// `AcceptDuelGump.DisplayTo(challenger, challenged, context, p, slot)`.
pub fn display_accept_duel(
    w: &mut World,
    challenger: MobileId,
    challenged: MobileId,
    id: u32,
    uid: u32,
    slot: i32,
) {
    if ns_of(w, challenged).is_none() {
        return;
    }
    let Some(c) = ctx(w, id) else { return };
    let Some(pi) = c.participant_index(uid) else {
        return;
    };
    let joins_sides = c.participants[pi].contains(challenger);
    let cn = name(w, challenger);
    let text = if joins_sides {
        format!("You have been asked to join sides with {cn} in a duel. Do you accept?")
    } else {
        format!("You have been challenged to a duel from {cn}. Do you accept?")
    };
    let st = accept(w);
    st.next += 1;
    let key = st.next;
    st.active.insert(key, true);
    timer::start(
        w,
        TimeSpan::from_millis(15_000),
        TimeSpan::ZERO,
        1,
        move |w| auto_reject(w, key, challenger, challenged),
    );
    send_dynamic(
        w,
        challenged,
        50,
        50,
        AcceptDuelGump {
            key,
            challenger,
            challenged,
            ctx: id,
            participant: uid,
            slot,
            joins_sides,
            text,
        },
        false,
    );
}

/// `AutoReject()`.
fn auto_reject(w: &mut World, key: u32, challenger: MobileId, challenged: MobileId) {
    if !accept(w).active.get(&key).copied().unwrap_or(false) {
        return;
    }
    accept(w).active.insert(key, false);
    close::<AcceptDuelGump>(w, challenged);
    let n = name(w, challenged);
    msg(w, challenger, &format!("{n} seems unresponsive."));
    msg(w, challenged, "You decline the challenge.");
}

impl DynamicGumpLogic for AcceptDuelGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.AcceptDuelGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();
        b.add_page(0);
        b.add_background(1, 1, 398, 218, 3600);
        b.add_image_tiled(16, 15, 369, 189, 3604);
        b.add_alpha_region(16, 15, 369, 189);
        b.add_image(215, -43, 0xEE40, 0);

        let black = html::center_rgb("Duel Challenge", BLACK_COLOR32);
        outlined(
            b,
            22,
            22,
            294,
            20,
            &black,
            &html::center_rgb("Duel Challenge", LABEL_COLOR32),
        );

        let accept = &self.text;
        outlined(
            b,
            22,
            50,
            294,
            40,
            &html::color_rgb(accept, BLACK_COLOR32),
            &html::color_rgb(accept, 0xB0C868),
        );
        b.add_image_tiled(32, 88, 264, 1, 9107);
        b.add_image_tiled(42, 90, 264, 1, 9157);

        for (y, text, selected, id) in [
            (100, "Yes, I will fight this duel.", true, 1),
            (135, "No, I do not wish to fight.", false, 2),
            (170, "No, knave. Do not ask again.", false, 3),
        ] {
            b.add_radio(24, y, 9727, 9730, selected, id);
            outlined(
                b,
                60,
                y + 5,
                250,
                20,
                &html::color_rgb(text, BLACK_COLOR32),
                &html::color_rgb(text, LABEL_COLOR32),
            );
        }
        b.add_button(314, 173, 247, 248, 1);
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let active = accept(w).active.get(&self.key).copied().unwrap_or(false);
        if info.button_id != 1 || !active || !ctx(w, self.ctx).is_some_and(|c| c.registered) {
            return;
        }
        accept(w).active.insert(self.key, false);
        let Some(pi) = ctx(w, self.ctx).and_then(|c| c.participant_index(self.participant)) else {
            return;
        };
        let (challenger, challenged) = (self.challenger, self.challenged);
        let cn = name(w, challenger);
        let dn = name(w, challenged);

        if info.is_switched(1) {
            if muo_content::player_mobile::data(w, challenged).is_none() {
                return;
            }
            if let Some(cid) = super::duel_context_of(w, challenged) {
                if ctx(w, cid).is_some_and(|c| c.initiator == challenged) {
                    msg_hue(w, challenged, 0x22, "You have already started a duel.");
                } else {
                    msg_hue(
                        w,
                        challenged,
                        0x22,
                        "You have already been challenged in a duel.",
                    );
                }
                msg(
                    w,
                    challenger,
                    &format!(
                        "{dn} cannot fight because they are already assigned to another duel."
                    ),
                );
            } else if flow::check_combat(w, challenged) {
                msg_hue(
                    w,
                    challenged,
                    0x22,
                    "You have recently been in combat with another player and must wait before starting a duel.",
                );
                msg(
                    w,
                    challenger,
                    &format!(
                        "{dn} cannot fight because they have recently been in combat with another player."
                    ),
                );
            } else {
                // `TournamentController.IsActive`
                if super::tournament::is_active(w) {
                    msg_hue(
                        w,
                        challenged,
                        0x22,
                        "A tournament is currently active and you may not duel.",
                    );
                    msg_hue(
                        w,
                        challenger,
                        0x22,
                        "A tournament is currently active and you may not duel.",
                    );
                    return;
                }
                let slot = self.slot;
                let p_len = ctx(w, self.ctx).map_or(0, |c| c.participants[pi].count());
                let target_slot = if slot >= 0
                    && (slot as usize) < p_len
                    && ctx(w, self.ctx)
                        .is_some_and(|c| c.participants[pi].players[slot as usize].is_none())
                {
                    Some(slot as usize)
                } else {
                    ctx(w, self.ctx)
                        .and_then(|c| c.participants[pi].players.iter().position(|s| s.is_none()))
                };
                if let Some(s) = target_slot {
                    super::set_slot(w, self.ctx, pi, s, challenged);
                    msg(w, challenger, &format!("{dn} has accepted the request."));
                    msg(
                        w,
                        challenged,
                        &format!("You have accepted the request from {cn}."),
                    );
                    // The challenger's open setup gump is shown again with the new player.
                    let uid = self.participant;
                    // `foreach (var g in GetGumps())` in the order the gumps were sent: the first match decides.
                    let first = ns_of(w, challenger).and_then(|ns| {
                        muo_content::gumps::system::gumps_of(w, ns).find_map(|g| {
                            if muo_content::gumps::system::downcast_logic::<ParticipantGump>(g)
                                .is_some_and(|p| p.participant == uid)
                            {
                                Some(true)
                            } else if muo_content::gumps::system::downcast_logic::<DuelContextGump>(
                                g,
                            )
                            .is_some_and(|d| d.ctx == self.ctx)
                            {
                                Some(false)
                            } else {
                                None
                            }
                        })
                    });
                    match first {
                        Some(true) => display_participant(w, challenger, self.ctx, uid),
                        Some(false) => display_duel_context(w, challenger, self.ctx),
                        None => {}
                    }
                } else {
                    msg(
                        w,
                        challenger,
                        &format!("The participant list was full and so {dn} could not join."),
                    );
                    if self.joins_sides {
                        msg(
                            w,
                            challenged,
                            &format!(
                                "The participant list was full and so you could not join the fight with {cn}."
                            ),
                        );
                    } else {
                        msg(
                            w,
                            challenged,
                            &format!(
                                "The participant list was full and so you could not join the fight against {cn}."
                            ),
                        );
                    }
                }
            }
        } else {
            if info.is_switched(3) {
                begin_ignore(w, challenged, challenger);
            }
            msg(w, challenger, &format!("{dn} does not wish to fight."));
            if self.joins_sides {
                msg(w, challenged, &format!("You chose not to fight with {cn}."));
            } else {
                msg(
                    w,
                    challenged,
                    &format!("You chose not to fight against {cn}."),
                );
            }
        }
    }
}

// ---- PreferencesGump ---------------------------------------------------------------------------------------------------------

/// `PreferencesGump`.
pub struct PreferencesGump {
    controller: muo_server::ids::ItemId,
    mobile: MobileId,
    /// `Arena.Arenas` names, and whether each is disliked.
    arenas: Vec<(String, bool)>,
}

/// `PreferencesGump.DisplayTo(from, prefs)`.
pub fn display_preferences(w: &mut World, from: MobileId) {
    let Some(controller) = ladder::preferences_instance(w) else {
        return;
    };
    if ns_of(w, from).is_none() {
        return;
    }
    let disliked = ladder::disliked(w, controller, from);
    let arenas = super::arena::arenas(w)
        .into_iter()
        .map(|a| {
            let n = super::arena::data(w, a).and_then(|d| d.name.clone());
            let d = disliked.contains(&n);
            (n.unwrap_or_else(|| "(no name)".to_owned()), d)
        })
        .collect();
    send_dynamic(
        w,
        from,
        50,
        50,
        PreferencesGump {
            controller,
            mobile: from,
            arenas,
        },
        false,
    );
}

impl DynamicGumpLogic for PreferencesGump {
    const TYPE_NAME: &'static str = "Server.Engines.ConPVP.PreferencesGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let mut column_x = 12;
        let n = self.arenas.len() as i32;
        b.add_page(0);
        let height = 12 + 20 + n * 31 + 24 + 12;
        b.add_background(0, 0, 499 + 40 - 365, height, 0x2436);
        let mut i = 1;
        while i < n {
            b.add_image_tiled(12, 32 + i * 31, 475 + 40 - 365, 30, 0x2430);
            i += 2;
        }
        b.add_alpha_region(10, 10, 479 + 40 - 365, height - 20);

        let mut header = |b: &mut DynamicGumpBuilder, width: i32, text: Option<&str>| {
            b.add_background(column_x, 12, width, 20, 0x242C);
            b.add_image_tiled(column_x + 2, 14, width - 4, 16, 0x2430);
            if let Some(t) = text {
                b.add_html(
                    column_x,
                    13,
                    width,
                    20,
                    &html::center_rgb(t, 0xFFFFFF),
                    false,
                    false,
                );
            }
            column_x += width;
        };
        header(b, 35, None);
        header(b, 115, Some("Arena"));

        b.add_button(
            499 + 40 - 365 - 12 - 63 - 4 - 63,
            height - 12 - 24,
            247,
            248,
            1,
        );
        b.add_button(499 + 40 - 365 - 12 - 63, height - 12 - 24, 241, 242, 2);

        for (i, (name, disliked)) in self.arenas.iter().enumerate() {
            let x = 12;
            let y = 32 + i as i32 * 31;
            b.add_checkbox(x + 3, y + 1, 9730, 9727, *disliked, i as i32);
            b.add_html(
                x + 35 + 5,
                y + 5,
                115 - 5,
                20,
                &html::color_rgb(name, 0xCCFFCC),
                false,
                false,
            );
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id != 1 {
            return;
        }
        let arenas = super::arena::arenas(w);
        let names: Vec<Option<String>> = info
            .switches
            .iter()
            .filter(|i| **i >= 0 && (**i as usize) < arenas.len())
            .map(|i| super::arena::data(w, arenas[*i as usize]).and_then(|d| d.name.clone()))
            .collect();
        let (controller, mobile) = (self.controller, self.mobile);
        if let Some(p) = w
            .item_mut(controller)
            .and_then(|i| i.ext.get_mut::<ladder::Preferences>())
        {
            p.find(mobile).disliked = names;
        }
    }
}
