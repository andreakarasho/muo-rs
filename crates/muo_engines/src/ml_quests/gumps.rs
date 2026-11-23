//! The ML quest gumps: `BaseQuestGump`, `QuestOfferGump`, `QuestLogGump`, `QuestLogDetailedGump`,
//! `QuestCancelConfirmGump`, `QuestConversationGump`, `QuestReportBackGump`, `QuestRewardGump`, `InfoNPCGump`.
//!
//! A gump here is built (and sent) the moment it is made, so what the layout reads from the instance
//! (the collected total, the time left, the skill state) is read when the gump is constructed: a snapshot in the
//! gump's logic, with the same values a layout drawn at that moment would show.

use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, RelayInfo, close_gump,
    send_gump_to_mobile,
};
use muo_content::text::{HtmlText, add_html_text_builder};
use muo_io::text::TextDefinition;
use muo_primitives::TimeSpan;
use muo_server::ids::MobileId;
use muo_server::net_state::NetStateId;
use muo_server::world::World;

use super::engine;
use super::model::*;
use super::quester::quester_name_for;

/// `AddHtmlText(ref builder, def, x, y, w, h, back, scroll, numberColor, stringColor)`.
#[allow(clippy::too_many_arguments)]
pub fn add_text(
    b: &mut DynamicGumpBuilder,
    def: &TextDefinition,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    back: bool,
    scroll: bool,
    number_color: i32,
    string_color: i32,
) {
    match add_html_text_builder(Some(def), back, scroll, number_color, string_color) {
        Some(HtmlText::Localized {
            number,
            args: _,
            color,
            back,
            scroll,
        }) => match color {
            Some(c) => b.add_html_localized_color(x, y, width, height, number, c, back, scroll),
            None => b.add_html_localized(x, y, width, height, number, back, scroll),
        },
        Some(HtmlText::Html { text, back, scroll }) => {
            b.add_html(x, y, width, height, &text, back, scroll)
        }
        None => {}
    }
}

fn loc(b: &mut DynamicGumpBuilder, x: i32, y: i32, w: i32, h: i32, number: i32, color: i32) {
    b.add_html_localized_color(x, y, w, h, number, color, false, false);
}

// ---- BaseMLQuestGump -------------------------------------------------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq)]
enum ButtonPosition {
    Left,
    Right,
}

/// `ButtonGraphic`.
mod graphic {
    pub const ACCEPT: i32 = 0x2EE0;
    pub const CLOSE: i32 = 0x2EE6;
    pub const CONTINUE: i32 = 0x2EE9;
    pub const OKAY: i32 = 0x2EEC;
    pub const PREVIOUS: i32 = 0x2EEF;
    pub const REFUSE: i32 = 0x2EF2;
    pub const RESIGN: i32 = 0x2EF5;
}

/// What `BaseMLQuestGump`'s constructor and `Register*`/`Set*` calls collect.
struct Frame {
    label: i32,
    max_pages: i32,
    title: Option<String>,
    buttons: Vec<(ButtonPosition, i32, i32)>,
    page: i32,
}

impl Frame {
    fn new(label: i32) -> Self {
        Self {
            label,
            max_pages: 0,
            title: None,
            buttons: Vec::new(),
            page: 0,
        }
    }

    /// `SetTitle(def)`.
    fn set_title(&mut self, def: &TextDefinition) {
        self.title = if def.number > 0 {
            Some(format!("#{}", def.number)) // OSI does "@@#{0}" instead, why? KR client related?
        } else {
            def.string.clone()
        };
    }

    fn register_button(&mut self, pos: ButtonPosition, graphic: i32, id: i32) {
        self.buttons.push((pos, graphic, id));
    }

    /// `BuildLayout` up to `BuildContent`.
    fn begin(&mut self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();

        b.add_page(0);

        b.add_image_tiled(50, 20, 400, 460, 0x1404);
        b.add_image_tiled(50, 29, 30, 450, 0x28DC);
        b.add_image_tiled(34, 140, 17, 339, 0x242F);
        b.add_image(48, 135, 0x28AB, 0);
        b.add_image(-16, 285, 0x28A2, 0);
        b.add_image(0, 10, 0x28B5, 0);
        b.add_image(25, 0, 0x28B4, 0);
        b.add_image_tiled(83, 15, 350, 15, 0x280A);
        b.add_image(34, 479, 0x2842, 0);
        b.add_image(442, 479, 0x2840, 0);
        b.add_image_tiled(51, 479, 392, 17, 0x2775);
        b.add_image_tiled(415, 29, 44, 450, 0xA2D);
        b.add_image_tiled(415, 29, 30, 450, 0x28DC);
        b.add_image(370, 50, 0x589, 0);
        b.add_image(379, 60, 0x15A9, 0);
        b.add_image(425, 0, 0x28C9, 0);
        b.add_image(90, 33, 0x232D, 0);
        loc(b, 130, 45, 270, 16, self.label, 0x7FFF);
        b.add_image_tiled(130, 65, 175, 1, 0x238D);

        // Reset paging for subclass content
        self.page = 0;
    }

    /// `BuildPage`.
    fn build_page(&mut self, b: &mut DynamicGumpBuilder) {
        self.page += 1;
        b.add_page(self.page);

        if self.page > 1 {
            b.add_button_type(
                130,
                430,
                graphic::PREVIOUS,
                graphic::PREVIOUS + 2,
                0,
                muo_content::gumps::GumpButtonType::Page,
                self.page - 1,
            );
        }

        if self.page < self.max_pages {
            b.add_button_type(
                275,
                430,
                graphic::CONTINUE,
                graphic::CONTINUE + 2,
                0,
                muo_content::gumps::GumpButtonType::Page,
                self.page + 1,
            );
        }

        for &(pos, g, id) in &self.buttons {
            b.add_button(
                if pos == ButtonPosition::Left { 95 } else { 313 },
                455,
                g,
                g + 2,
                id,
            );
        }

        if let Some(title) = &self.title {
            // <DIV ALIGN=CENTER>~1_TOKEN~</DIV>
            b.add_html_localized_args(130, 68, 220, 48, 1114513, title, 0x2710, false, false);
        }
    }
}

/// The part of a quest every gump that shows it reads.
#[derive(Clone)]
struct QuestView {
    title: TextDefinition,
    description: TextDefinition,
    chain: bool,
    objective_type: ObjectiveType,
    objectives: Vec<Objective>,
    rewards: Vec<Reward>,
}

impl QuestView {
    fn of(w: &World, qid: QuestId) -> Self {
        let q = engine::quest(w, qid);
        Self {
            title: q.title.clone(),
            description: q.description.clone(),
            chain: q.is_chain_triggered() || q.class.next_quest.is_some(),
            objective_type: q.objective_type,
            objectives: q.objectives.clone(),
            rewards: q.rewards.clone(),
        }
    }

    /// `AddDescription`.
    fn add_description(&self, b: &mut DynamicGumpBuilder) {
        loc(
            b,
            98,
            140,
            312,
            16,
            if self.chain { 1075024 } else { 1072202 },
            0x2710,
        ); // Description [(quest chain)]
        add_text(
            b,
            &self.description,
            98,
            156,
            312,
            240,
            false,
            true,
            0x5F90,
            0xBDE784,
        );
    }

    fn add_objective_header(&self, b: &mut DynamicGumpBuilder) {
        loc(b, 98, 140, 312, 16, 1049073, 0x2710); // Objective:
        // All of the following / Only one of the following
        loc(
            b,
            98,
            156,
            312,
            16,
            if self.objective_type == ObjectiveType::All {
                1072208
            } else {
                1072209
            },
            0x2710,
        );
    }

    /// `AddObjectives`.
    fn add_objectives(&self, b: &mut DynamicGumpBuilder) {
        self.add_objective_header(b);

        let mut y = 172;

        for objective in &self.objectives {
            write_objective(b, objective, &mut y);

            if let Some(d) = objective.duration {
                if matches!(objective.kind, ObjectiveKind::Collect { .. }) {
                    y -= 16;
                }

                write_time_remaining(b, &mut y, d);
            }
        }
    }

    /// `AddRewardsPage`.
    fn add_rewards_page(&self, b: &mut DynamicGumpBuilder) {
        loc(b, 98, 140, 312, 16, 1072201, 0x2710); // Reward

        let mut y = 162;

        if self.rewards.len() > 1 {
            // TODO: Is this what this is for? Does "Only one of the following" occur?
            loc(b, 98, 156, 312, 16, 1072208, 0x2710); // All of the following
            y += 16;
        }

        add_rewards_at(b, &self.rewards, 105, y, 16);
    }

    /// `AddRewards` (the claim gump).
    fn add_rewards(&self, b: &mut DynamicGumpBuilder) {
        let mut y = 146;

        if self.rewards.len() > 1 {
            loc(b, 100, 140, 312, 16, 1072208, 0x2710); // All of the following
            y += 16;
        }

        add_rewards_at(b, &self.rewards, 107, y, 26);
    }
}

fn add_rewards_at(
    b: &mut DynamicGumpBuilder,
    rewards: &[Reward],
    x: i32,
    mut y: i32,
    spacing: i32,
) {
    let x_reward = x + 28;

    for reward in rewards {
        b.add_image(x, y + 1, 0x4B9, 0);
        // BaseReward.WriteToGump
        add_text(
            b,
            &reward.name,
            x_reward,
            y,
            280,
            reward.label_height(),
            false,
            false,
            0x5F90,
            0xBDE784,
        );
        y += spacing;
    }
}

fn add_conversation(b: &mut DynamicGumpBuilder, text: &TextDefinition) {
    add_text(b, text, 98, 140, 312, 180, false, true, 0x5F90, 0xBDE784);
}

/// `BaseObjectiveInstance.WriteTimeRemaining`.
fn write_time_remaining(b: &mut DynamicGumpBuilder, y: &mut i32, remaining: TimeSpan) {
    loc(b, 103, *y, 120, 16, 1062379, 0x5F90); // Est. time remaining:
    b.add_label(
        223,
        *y,
        0x481,
        &format!("{}", (remaining.total_seconds() + 0.5).floor()),
    );
    *y += 16;
}

/// `LabelToItemID`.
fn label_to_item_id(label: i32) -> i32 {
    if label < 1078872 {
        label - 1020000
    } else {
        label - 1078872
    }
}

/// `{value:0.#}`.
fn fmt_one_decimal(v: f64) -> String {
    let r = (v * 10.0).round() / 10.0;
    if (r - r.trunc()).abs() < 1e-9 {
        format!("{}", r as i64)
    } else {
        format!("{r:.1}")
    }
}

/// `BaseObjective.WriteToGump` of every objective class.
fn write_objective(b: &mut DynamicGumpBuilder, o: &Objective, y: &mut i32) {
    match &o.kind {
        ObjectiveKind::Kill {
            amount, name, area, ..
        } => {
            let amount = amount.to_string();

            loc(b, 98, *y, 312, 16, 1072204, 0x5F90); // Slay
            b.add_label(133, *y, 0x481, &amount);

            let nx = 133 + amount.len() as i32 * 15;
            if name.number > 0 {
                loc(b, nx, *y, 190, 18, name.number, 0x77BF);
            } else if let Some(s) = &name.string {
                b.add_label(nx, *y, 0x481, s);
            }

            *y += 16;

            if let Some(area) = area {
                loc(b, 103, *y, 312, 20, 1018327, 0x5F90); // Location

                if area.name.number > 0 {
                    loc(b, 223, *y, 312, 20, area.name.number, 0x7FFF);
                } else if let Some(s) = &area.name.string {
                    b.add_label(223, *y, 0x481, s);
                }

                *y += 16;
            }
        }
        ObjectiveKind::Collect {
            amount,
            name,
            show_detailed,
            ..
        } => {
            if *show_detailed {
                let amount = amount.to_string();

                loc(b, 98, *y, 350, 16, 1072205, 0x5F90); // Obtain
                b.add_label(143, *y, 0x481, &amount);

                let nx = 143 + amount.len() as i32 * 15;
                if name.number > 0 {
                    loc(b, nx, *y, 190, 18, name.number, 0x77BF);
                    b.add_item(350, *y, label_to_item_id(name.number), 0);
                } else if let Some(s) = &name.string {
                    b.add_label(nx, *y, 0x481, s);
                }
            } else if name.number > 0 {
                loc(b, 98, *y, 312, 32, name.number, 0x5F90);
            } else if let Some(s) = &name.string {
                b.add_label(98, *y, 0x481, s);
            }

            *y += 32;
        }
        ObjectiveKind::Deliver {
            amount,
            name,
            destination,
            ..
        } => {
            let amount = amount.to_string();

            loc(b, 98, *y, 312, 16, 1072207, 0x5F90); // Deliver
            b.add_label(143, *y, 0x481, &amount);

            let nx = 143 + amount.len() as i32 * 15;
            if name.number > 0 {
                loc(b, nx, *y, 190, 18, name.number, 0x77BF);
                b.add_item(350, *y, label_to_item_id(name.number), 0);
            } else if let Some(s) = &name.string {
                b.add_label(nx, *y, 0x481, s);
            }

            *y += 32;

            loc(b, 103, *y, 120, 16, 1072379, 0x5F90); // Deliver to
            b.add_label(223, *y, 0x481, &quester_name_for_short(destination));

            *y += 16;
        }
        ObjectiveKind::Escort { destination } => {
            loc(b, 98, *y, 312, 16, 1072206, 0x5F90); // Escort to

            if let Some(d) = destination {
                if d.name.number > 0 {
                    loc(b, 173, *y, 312, 20, d.name.number, 0x7FFF);
                } else if let Some(s) = &d.name.string {
                    b.add_label(173, *y, 0x481, s);
                }
            }

            *y += 16;
        }
        ObjectiveKind::GainSkill {
            skill,
            threshold_fixed,
            ..
        } => {
            let skill_label = muo_content::aos::skill_label(*skill);
            let args = format!(
                "#{skill_label}\t{}",
                fmt_one_decimal(f64::from(*threshold_fixed) / 10.0)
            );

            // Increase ~1_SKILL~ to ~2_VALUE~
            b.add_html_localized_args(98, *y, 312, 16, 1077485, &args, 0x5F90, false, false);
            *y += 16;
        }
    }
}

/// `QuesterNameAttribute.GetQuesterNameFor(typeof(X))` for a short class name.
fn quester_name_for_short(short: &str) -> String {
    quester_name_for(Some(short))
}

/// A snapshot of an objective instance for the log gump.
#[derive(Clone)]
struct ObjectiveView {
    def: Objective,
    remaining: Option<TimeSpan>,
    slain: i32,
    total: i32,
    completed: bool,
}

/// `BaseObjectiveInstance.WriteToGump` of every class.
fn write_objective_instance(
    b: &mut DynamicGumpBuilder,
    v: &ObjectiveView,
    quester_name: &str,
    y: &mut i32,
) {
    let base = |b: &mut DynamicGumpBuilder, y: &mut i32| {
        if let Some(r) = v.remaining {
            write_time_remaining(b, y, r);
        }
    };
    match &v.def.kind {
        ObjectiveKind::Kill { .. } => {
            write_objective(b, &v.def, y);
            base(b, y);

            loc(b, 103, *y, 120, 16, 3000087, 0x5F90); // Total
            b.add_label(223, *y, 0x481, &v.slain.to_string());
            *y += 16;

            loc(b, 103, *y, 120, 16, 1074782, 0x5F90); // Return to
            b.add_label(223, *y, 0x481, quester_name);
            *y += 16;
        }
        ObjectiveKind::Collect { show_detailed, .. } => {
            write_objective(b, &v.def, y);
            *y -= 16;

            if *show_detailed {
                base(b, y);

                loc(b, 103, *y, 120, 16, 3000087, 0x5F90); // Total
                b.add_label(223, *y, 0x481, &v.total.to_string());
                *y += 16;

                loc(b, 103, *y, 120, 16, 1074782, 0x5F90); // Return to
                b.add_label(223, *y, 0x481, quester_name);
                *y += 16;
            }
        }
        ObjectiveKind::Deliver { .. } | ObjectiveKind::Escort { .. } => {
            write_objective(b, &v.def, y);
            base(b, y);
            // No extra instance stuff printed for this objective
        }
        ObjectiveKind::GainSkill { .. } => {
            write_objective(b, &v.def, y);
            base(b, y);

            if v.completed {
                loc(b, 113, *y, 312, 20, 1055121, 0x7FFF); // Complete
                *y += 16;
            }
        }
    }
}

/// `BaseMLQuestGump.CloseOtherGumps(pm)`.
pub fn close_other_gumps(w: &mut World, pm: MobileId) {
    let Some(ns) = w.mobile(pm).and_then(|m| m.net_state) else {
        return;
    };
    close_gump::<InfoNpcGump>(w, ns);
    close_gump::<QuestRewardGump>(w, ns);
    close_gump::<QuestConversationGump>(w, ns);
    close_gump::<QuestReportBackGump>(w, ns);
    close_gump::<QuestCancelConfirmGump>(w, ns);
}

fn send<L: DynamicGumpLogic>(w: &mut World, to: MobileId, x: i32, y: i32, logic: L) {
    send_gump_to_mobile(w, to, Box::new(DynamicGumpObj::new(x, y, logic)), false);
}

fn has_net_state(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|m| m.net_state.is_some())
}

fn sender_mobile(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_state(ns).and_then(|n| n.mobile)
}

fn is_player(w: &World, m: MobileId) -> bool {
    muo_content::player_mobile::data(w, m).is_some()
}

// ---- QuestOfferGump --------------------------------------------------------------------------------------

struct QuestOfferGump {
    quest: QuestId,
    quester: Quester,
    view: QuestView,
}

impl DynamicGumpLogic for QuestOfferGump {
    const TYPE_NAME: &'static str = "Server.Engines.MLQuests.Gumps.QuestOfferGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let mut f = Frame::new(1049010); // Quest Offer
        f.set_title(&self.view.title);
        f.register_button(ButtonPosition::Left, graphic::ACCEPT, 1);
        f.register_button(ButtonPosition::Right, graphic::REFUSE, 2);
        f.max_pages = 3;
        f.begin(b);

        f.build_page(b);
        self.view.add_description(b);

        f.build_page(b);
        self.view.add_objectives(b);

        f.build_page(b);
        self.view.add_rewards_page(b);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(pm) = sender_mobile(w, sender).filter(|&m| is_player(w, m)) else {
            return;
        };
        match info.button_id {
            1 => engine::on_accept(w, self.quest, self.quester, pm),
            2 => engine::on_refuse(w, self.quest, pm),
            _ => {}
        }
    }
}

/// `QuestOfferGump.DisplayTo(pm, quest, quester)`.
pub fn display_offer(w: &mut World, pm: MobileId, quest: QuestId, quester: Quester) {
    if !has_net_state(w, pm) {
        return;
    }
    close_other_gumps(w, pm);
    let view = QuestView::of(w, quest);
    send(
        w,
        pm,
        75,
        25,
        QuestOfferGump {
            quest,
            quester,
            view,
        },
    );
}

// ---- QuestLogGump ----------------------------------------------------------------------------------------

struct QuestLogGump {
    close_gumps: bool,
    owner: MobileId,
    /// (title, failed) per instance.
    entries: Vec<(TextDefinition, bool)>,
}

impl DynamicGumpLogic for QuestLogGump {
    const TYPE_NAME: &'static str = "Server.Engines.MLQuests.Gumps.QuestLogGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let mut f = Frame::new(1046026); // Quest Log
        f.register_button(ButtonPosition::Right, graphic::OKAY, 3);
        f.max_pages = 1;
        f.begin(b);

        f.build_page(b);

        for (i, (title, failed)) in self.entries.iter().enumerate() {
            let (number_color, string_color) = if *failed {
                (0x3C00, 0x7B0000)
            } else {
                (0xFFFFFF, 0xFFFFFF)
            };
            let y = 140 + 21 * i as i32;

            add_text(
                b,
                title,
                98,
                y,
                270,
                21,
                false,
                false,
                number_color,
                string_color,
            );
            b.add_button_type(
                368,
                y,
                0x26B0,
                0x26B1,
                6 + i as i32,
                muo_content::gumps::GumpButtonType::Reply,
                1,
            );
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id < 6 {
            return;
        }
        let Some(ctx) = engine::get_context(w, self.owner) else {
            return;
        };
        let index = (info.button_id - 6) as usize;
        let Some(&inst) = ctx.quest_instances.get(index) else {
            return;
        };
        let Some(from) = sender_mobile(w, sender) else {
            return;
        };
        display_log_detailed(w, from, inst, self.close_gumps);
    }
}

fn log_entries(w: &World, pm: MobileId) -> Vec<(TextDefinition, bool)> {
    engine::get_context(w, pm)
        .map(|c| {
            c.quest_instances
                .iter()
                .filter_map(|&i| engine::instance(w, i))
                .map(|i| (engine::quest(w, i.quest).title.clone(), i.failed()))
                .collect()
        })
        .unwrap_or_default()
}

/// `QuestLogGump.DisplayTo(from, pm, closeGumps)`.
pub fn display_log(w: &mut World, from: MobileId, pm: MobileId, close_gumps: bool) {
    if !has_net_state(w, from) {
        return;
    }

    if close_gumps && let Some(ns) = w.mobile(pm).and_then(|m| m.net_state) {
        close_gump::<QuestLogDetailedGump>(w, ns);
    }

    let entries = log_entries(w, pm);
    send(
        w,
        from,
        75,
        25,
        QuestLogGump {
            close_gumps,
            owner: pm,
            entries,
        },
    );
}

/// `QuestLogGump.DisplayTo(pm)` (`QuestGumpRequest`).
pub fn display_quest_log(w: &mut World, pm: MobileId) {
    if !has_net_state(w, pm) {
        return;
    }
    display_log(w, pm, pm, true);
}

// ---- QuestLogDetailedGump --------------------------------------------------------------------------------

struct QuestLogDetailedGump {
    instance: InstanceId,
    close_gumps: bool,
    view: QuestView,
    failed: bool,
    progress: Vec<ObjectiveView>,
    quester_name: String,
}

impl DynamicGumpLogic for QuestLogDetailedGump {
    const TYPE_NAME: &'static str = "Server.Engines.MLQuests.Gumps.QuestLogDetailedGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let mut f = Frame::new(1046026); // Quest Log
        f.set_title(&self.view.title);
        f.register_button(ButtonPosition::Left, graphic::RESIGN, 1);
        f.register_button(ButtonPosition::Right, graphic::OKAY, 2);
        f.max_pages = 3;
        f.begin(b);

        f.build_page(b);
        self.view.add_description(b);

        if self.failed {
            // only displayed on the first page
            loc(b, 160, 80, 250, 16, 500039, 0x3C00); // Failed!
        }

        f.build_page(b);
        // AddObjectivesProgress
        self.view.add_objective_header(b);
        let mut y = 172;
        for v in &self.progress {
            write_objective_instance(b, v, &self.quester_name, &mut y);
        }

        f.build_page(b);
        self.view.add_rewards_page(b);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if engine::instance(w, self.instance).is_none_or(|i| i.removed()) {
            return;
        }
        let Some(from) = sender_mobile(w, sender) else {
            return;
        };
        match info.button_id {
            1 => {
                // Resign
                // TODO: Custom reward loss protection? OSI doesn't have this
                display_cancel_confirm(w, from, self.instance, self.close_gumps);
            }
            2 => {
                // Okay
                let player = engine::instance(w, self.instance).map(|i| i.player);
                if let Some(p) = player {
                    display_log(w, from, p, self.close_gumps);
                }
            }
            _ => {}
        }
    }
}

/// `QuestLogDetailedGump.DisplayTo(from, instance, closeGumps)`.
pub fn display_log_detailed(w: &mut World, from: MobileId, inst: InstanceId, close_gumps: bool) {
    if !has_net_state(w, from) {
        return;
    }
    let Some(i) = engine::instance(w, inst) else {
        return;
    };
    let (player, qid, quester_type) = (i.player, i.quest, i.quester_type);

    if close_gumps {
        close_other_gumps(w, player);
    }

    let view = QuestView::of(w, qid);
    let failed = engine::instance(w, inst).is_some_and(|i| i.failed());
    let mut progress = Vec::new();
    for (idx, def) in view.objectives.iter().enumerate() {
        let Some(i) = engine::instance(w, inst) else {
            return;
        };
        let Some(o) = i.objectives.get(idx) else {
            continue;
        };
        let remaining = o
            .is_timed()
            .then(|| (o.end_time - w.now).max(TimeSpan::ZERO));
        let slain = match o.state {
            ObjectiveState::Kill { slain } => slain,
            _ => 0,
        };
        let total = engine::collect_current_total(w, inst, idx);
        let completed = engine::objective_is_completed(w, inst, idx);
        progress.push(ObjectiveView {
            def: def.clone(),
            remaining,
            slain,
            total,
            completed,
        });
    }

    send(
        w,
        from,
        75,
        25,
        QuestLogDetailedGump {
            instance: inst,
            close_gumps,
            view,
            failed,
            progress,
            quester_name: quester_name_for(quester_type),
        },
    );
}

// ---- QuestCancelConfirmGump ------------------------------------------------------------------------------

struct QuestCancelConfirmGump {
    instance: InstanceId,
    close_gumps: bool,
    chain: bool,
}

impl DynamicGumpLogic for QuestCancelConfirmGump {
    const TYPE_NAME: &'static str = "Server.Engines.MLQuests.Gumps.QuestCancelConfirmGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();

        b.add_page(0);

        b.add_image_tiled(0, 0, 348, 262, 0xA8E);
        b.add_alpha_region(0, 0, 348, 262);

        b.add_image(0, 15, 0x27A8, 0);
        b.add_image_tiled(0, 30, 17, 200, 0x27A7);
        b.add_image(0, 230, 0x27AA, 0);

        b.add_image(15, 0, 0x280C, 0);
        b.add_image_tiled(30, 0, 300, 17, 0x280A);
        b.add_image(315, 0, 0x280E, 0);

        b.add_image(15, 244, 0x280C, 0);
        b.add_image_tiled(30, 244, 300, 17, 0x280A);
        b.add_image(315, 244, 0x280E, 0);

        b.add_image(330, 15, 0x27A8, 0);
        b.add_image_tiled(330, 30, 17, 200, 0x27A7);
        b.add_image(330, 230, 0x27AA, 0);

        b.add_image(333, 2, 0x2716, 0);
        b.add_image(333, 248, 0x2716, 0);
        b.add_image(2, 248, 0x2716, 0);
        b.add_image(2, 2, 0x2716, 0);

        loc(b, 25, 22, 200, 20, 1049000, 0x7D00); // Confirm Quest Cancellation
        b.add_image(25, 40, 0xBBF, 0);

        /*
         * This quest will give you valuable information, skills
         * and equipment that will help you advance in the
         * game at a quicker pace.<BR>
         * <BR>
         * Are you certain you wish to cancel at this time?
         */
        loc(b, 25, 55, 300, 120, 1060836, 0x7FFF);

        if self.chain {
            b.add_radio(25, 145, 0x25F8, 0x25FB, false, 2);
            loc(b, 60, 150, 280, 20, 1075023, 0x7FFF); // Yes, I want to quit this entire chain!
        }

        b.add_radio(25, 180, 0x25F8, 0x25FB, true, 1);
        loc(b, 60, 185, 280, 20, 1049005, 0x7FFF); // Yes, I really want to quit this quest!

        b.add_radio(25, 215, 0x25F8, 0x25FB, false, 0);
        loc(b, 60, 220, 280, 20, 1049006, 0x7FFF); // No, I don't want to quit.

        b.add_button(265, 220, 0xF7, 0xF8, 7);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if engine::instance(w, self.instance).is_none_or(|i| i.removed()) {
            return;
        }
        if info.button_id == 7 {
            // Okay
            if info.is_switched(2) {
                engine::instance_cancel(w, self.instance, true);
            } else if info.is_switched(1) {
                engine::instance_cancel(w, self.instance, false);
            }

            let (Some(from), Some(player)) = (
                sender_mobile(w, sender),
                engine::instance(w, self.instance).map(|i| i.player),
            ) else {
                return;
            };
            display_log(w, from, player, self.close_gumps);
        }
    }
}

/// `QuestCancelConfirmGump.DisplayTo(from, instance, closeGumps)`.
pub fn display_cancel_confirm(w: &mut World, from: MobileId, inst: InstanceId, close_gumps: bool) {
    if !has_net_state(w, from) {
        return;
    }
    let Some(i) = engine::instance(w, inst) else {
        return;
    };
    let (player, qid) = (i.player, i.quest);

    if close_gumps {
        close_other_gumps(w, player);
    }

    let q = engine::quest(w, qid);
    let chain = q.is_chain_triggered() || q.class.next_quest.is_some();
    send(
        w,
        from,
        120,
        50,
        QuestCancelConfirmGump {
            instance: inst,
            close_gumps,
            chain,
        },
    );
}

// ---- QuestConversationGump -------------------------------------------------------------------------------

struct QuestConversationGump {
    title: TextDefinition,
    text: TextDefinition,
}

impl DynamicGumpLogic for QuestConversationGump {
    const TYPE_NAME: &'static str = "Server.Engines.MLQuests.Gumps.QuestConversationGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let mut f = Frame::new(3006156); // Quest Conversation
        f.set_title(&self.title);
        f.register_button(ButtonPosition::Right, graphic::CLOSE, 3);
        f.max_pages = 1;
        f.begin(b);

        f.build_page(b);
        add_conversation(b, &self.text);
    }
}

/// `QuestConversationGump.DisplayTo(pm, quest, text)`.
pub fn display_conversation(w: &mut World, pm: MobileId, quest: QuestId, text: TextDefinition) {
    if !has_net_state(w, pm) {
        return;
    }
    close_other_gumps(w, pm);
    let title = engine::quest(w, quest).title.clone();
    send(w, pm, 75, 25, QuestConversationGump { title, text });
}

// ---- QuestReportBackGump ---------------------------------------------------------------------------------

struct QuestReportBackGump {
    instance: InstanceId,
    title: TextDefinition,
    message: TextDefinition,
}

impl DynamicGumpLogic for QuestReportBackGump {
    const TYPE_NAME: &'static str = "Server.Engines.MLQuests.Gumps.QuestReportBackGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let mut f = Frame::new(3006156); // Quest Conversation
        f.set_title(&self.title);
        f.register_button(ButtonPosition::Left, graphic::CONTINUE, 4);
        f.register_button(ButtonPosition::Right, graphic::CLOSE, 3);
        f.max_pages = 1;
        f.begin(b);

        f.build_page(b);
        add_conversation(b, &self.message);
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 4 {
            engine::continue_report_back(w, self.instance, true);
        }
    }
}

/// `QuestReportBackGump.DisplayTo(from, instance)`.
pub fn display_report_back(w: &mut World, inst: InstanceId) {
    let Some(i) = engine::instance(w, inst) else {
        return;
    };
    let (player, qid) = (i.player, i.quest);
    if !has_net_state(w, player) {
        return;
    }

    // TODO: Check close sequence
    close_other_gumps(w, player);

    let q = engine::quest(w, qid);
    let (title, message) = (q.title.clone(), q.completion_message.clone());
    send(
        w,
        player,
        75,
        25,
        QuestReportBackGump {
            instance: inst,
            title,
            message,
        },
    );
}

// ---- QuestRewardGump -------------------------------------------------------------------------------------

struct QuestRewardGump {
    instance: InstanceId,
    view: QuestView,
}

impl DynamicGumpLogic for QuestRewardGump {
    const TYPE_NAME: &'static str = "Server.Engines.MLQuests.Gumps.QuestRewardGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let mut f = Frame::new(1072201); // Reward
        f.set_title(&self.view.title);
        f.register_button(ButtonPosition::Left, graphic::ACCEPT, 1);
        f.max_pages = 1;
        f.begin(b);

        f.build_page(b);
        self.view.add_rewards(b);
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 1 {
            engine::claim_rewards(w, self.instance);
        }
    }
}

/// `QuestRewardGump.DisplayTo(from, instance)`.
pub fn display_reward(w: &mut World, from: MobileId, inst: InstanceId) {
    if !has_net_state(w, from) {
        return;
    }
    let Some(i) = engine::instance(w, inst) else {
        return;
    };
    let (player, qid) = (i.player, i.quest);

    close_other_gumps(w, player);

    let view = QuestView::of(w, qid);
    send(
        w,
        from,
        75,
        25,
        QuestRewardGump {
            instance: inst,
            view,
        },
    );
}

// ---- InfoNPCGump -----------------------------------------------------------------------------------------

pub struct InfoNpcGump {
    title: TextDefinition,
    message: TextDefinition,
}

impl DynamicGumpLogic for InfoNpcGump {
    const TYPE_NAME: &'static str = "Server.Engines.MLQuests.Gumps.InfoNPCGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let mut f = Frame::new(1060668); // INFORMATION
        f.register_button(ButtonPosition::Left, graphic::CLOSE, 3);
        f.max_pages = 1;
        f.begin(b);

        f.build_page(b);
        add_text(
            b,
            &self.title,
            160,
            108,
            250,
            16,
            false,
            false,
            0x2710,
            0x4AC684,
        );
        add_text(
            b,
            &self.message,
            98,
            156,
            312,
            180,
            false,
            true,
            0x5F90,
            0xBDE784,
        );
    }
}

/// `InfoNPCGump.DisplayTo(from, title, message)`.
pub fn display_info_npc(
    w: &mut World,
    from: MobileId,
    title: TextDefinition,
    message: TextDefinition,
) {
    if !has_net_state(w, from) {
        return;
    }
    send(w, from, 75, 25, InfoNpcGump { title, message });
}
