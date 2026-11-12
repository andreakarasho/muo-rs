//! The quest engine's gumps: QuestOfferGump, QuestCancelGump, QuestLogUpdatedGump, QuestObjectivesGump,
//! QuestConversationsGump, QuestItemInfoGump and BaseQuestGump.
//!
//! A gump here holds a copy of the quest to draw from (the layout is built without the world) and the player; a
//! response runs on the player's stored quest, and is dropped if it is no longer a quest of the same class.

use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, GumpButtonType, RelayInfo,
    send_gump_to_mobile,
};
use muo_primitives::TimeSpan;
use muo_primitives::html;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part1::props_b::net_state;
use muo_server::net_state::NetStateId;

use super::core::{Conversation, ItemInfo, Msg, QClass, Quest, with_quest_of};

pub const BLACK: i32 = 0x0000;
pub const WHITE: i32 = 0x7FFF;
pub const DARK_GREEN: i32 = 10000;
pub const LIGHT_GREEN: i32 = 90000;
pub const BLUE: i32 = 19777215;

/// `BaseQuestGump.AddHtmlObject`.
#[allow(clippy::too_many_arguments)]
pub fn add_html_object(
    b: &mut DynamicGumpBuilder,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    message: Msg,
    color: i32,
    back: bool,
    scroll: bool,
) {
    let color = color & 0x7FFF;
    match message {
        Msg::Num(n) => b.add_html_localized_color(x, y, width, height, n, color, back, scroll),
        Msg::Text(t) => b.add_html(
            x,
            y,
            width,
            height,
            &html::color_rgb(t, color),
            back,
            scroll,
        ),
    }
}

fn send(w: &mut World, to: MobileId, x: i32, y: i32, logic: impl DynamicGumpLogic) {
    send_gump_to_mobile(w, to, Box::new(DynamicGumpObj::new(x, y, logic)), false);
}

fn connected(w: &World, m: MobileId) -> bool {
    net_state(w, m).is_some()
}

// ---- QuestOfferGump ------------------------------------------------------------------------------------------------------

pub struct OfferLogic {
    quest: Quest,
    name: Msg,
    offer: Msg,
    picture: i32,
}

/// `QuestOfferGump.DisplayTo(from, system)`.
pub fn offer_display(w: &mut World, q: &Quest) {
    if !connected(w, q.from) {
        return;
    }
    let logic = OfferLogic {
        quest: q.clone(),
        name: q.class.name_msg,
        offer: (q.class.offer_message)(w, q),
        picture: (q.class.picture)(w, q),
    };
    send(w, q.from, 75, 25, logic);
}

impl DynamicGumpLogic for OfferLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Quests.QuestOfferGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();
        b.add_page(0);

        b.add_image_tiled(50, 20, 400, 400, 2624);
        b.add_alpha_region(50, 20, 400, 400);

        b.add_image(90, 33, 9005, 0);
        b.add_html_localized_color(130, 45, 270, 20, 1049010, WHITE, false, false); // Quest Offer
        b.add_image_tiled(130, 65, 175, 1, 9101);

        b.add_image(140, 110, 1209, 0);
        add_html_object(b, 160, 108, 250, 20, self.name, DARK_GREEN, false, false);

        add_html_object(b, 98, 140, 312, 200, self.offer, LIGHT_GREEN, false, true);

        b.add_radio(85, 350, 9720, 9723, true, 1);
        b.add_html_localized_color(120, 356, 280, 20, 1049011, WHITE, false, false); // I accept!

        b.add_radio(85, 385, 9720, 9723, false, 0);
        b.add_html_localized_color(120, 391, 280, 20, 1049012, WHITE, false, false); // No thanks, I decline.

        b.add_button(340, 390, 247, 248, 1);

        b.add_image_tiled(50, 29, 30, 390, 10460);
        b.add_image_tiled(34, 140, 17, 279, 9263);

        b.add_image(48, 135, 10411, 0);
        b.add_image(-16, 285, 10402, 0);
        b.add_image(0, 10, 10421, 0);
        b.add_image(25, 0, 10420, 0);

        b.add_image_tiled(83, 15, 350, 15, 10250);

        b.add_image(34, 419, 10306, 0);
        b.add_image(442, 419, 10304, 0);
        b.add_image_tiled(51, 419, 392, 17, 10101);

        b.add_image_tiled(415, 29, 44, 390, 2605);
        b.add_image_tiled(415, 29, 30, 390, 10460);
        b.add_image(425, 0, 10441, 0);

        b.add_image(370, 50, 1417, 0);
        b.add_image(379, 60, self.picture, 0);
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id != 1 {
            return;
        }
        let mut q = self.quest.clone();
        let f = if info.is_switched(1) {
            q.class.accept
        } else {
            q.class.decline
        };
        f(w, &mut q);
        if q.attached {
            super::core::store(w, q.from, &q);
        }
    }
}

// ---- QuestCancelGump ------------------------------------------------------------------------------------------------------

pub struct CancelLogic {
    player: MobileId,
    class: &'static QClass,
    tutorial: bool,
    name: Msg,
    restartable: bool,
}

/// `QuestCancelGump.DisplayTo(from, system)`.
pub fn cancel_display(w: &mut World, q: &Quest) {
    if !connected(w, q.from) {
        return;
    }
    let logic = CancelLogic {
        player: q.from,
        class: q.class,
        tutorial: q.class.is_tutorial,
        name: q.class.name_msg,
        restartable: q.class.restart_delay < TimeSpan::MAX,
    };
    send(w, q.from, 120, 50, logic);
}

impl DynamicGumpLogic for CancelLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Quests.QuestCancelGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();
        b.add_page(0);

        b.add_image_tiled(0, 0, 348, 262, 2702);
        b.add_alpha_region(0, 0, 348, 262);

        b.add_image(0, 15, 10152, 0);
        b.add_image_tiled(0, 30, 17, 200, 10151);
        b.add_image(0, 230, 10154, 0);

        b.add_image(15, 0, 10252, 0);
        b.add_image_tiled(30, 0, 300, 17, 10250);
        b.add_image(315, 0, 10254, 0);

        b.add_image(15, 244, 10252, 0);
        b.add_image_tiled(30, 244, 300, 17, 10250);
        b.add_image(315, 244, 10254, 0);

        b.add_image(330, 15, 10152, 0);
        b.add_image_tiled(330, 30, 17, 200, 10151);
        b.add_image(330, 230, 10154, 0);

        b.add_image(333, 2, 10006, 0);
        b.add_image(333, 248, 10006, 0);
        b.add_image(2, 248, 10006, 0);
        b.add_image(2, 2, 10006, 0);

        b.add_html_localized_color(25, 22, 200, 20, 1049000, 32000, false, false); // Confirm Quest Cancellation
        b.add_image(25, 40, 3007, 0);

        if self.tutorial {
            // This quest will give you valuable information, skills and equipment that will help you advance in
            // the game at a quicker pace.<BR><BR>Are you certain you wish to cancel at this time?
            b.add_html_localized_color(25, 55, 300, 120, 1060836, WHITE, false, false);
        } else {
            b.add_html_localized_color(25, 60, 300, 20, 1049001, WHITE, false, false); // You have chosen to abort your quest:
            b.add_image(25, 81, 0x25E7, 0);
            add_html_object(b, 48, 80, 280, 20, self.name, DARK_GREEN, false, false);

            b.add_html_localized_color(25, 120, 280, 20, 1049002, WHITE, false, false); // Can this quest be restarted after quitting?
            b.add_image(25, 141, 0x25E7, 0);
            let yes_no = if self.restartable { 1049016 } else { 1049017 };
            b.add_html_localized_color(48, 140, 280, 20, yes_no, DARK_GREEN, false, false); // Yes/No
        }

        b.add_radio(25, 175, 9720, 9723, true, 1);
        b.add_html_localized_color(60, 180, 280, 20, 1049005, WHITE, false, false); // Yes, I really want to quit!

        b.add_radio(25, 210, 9720, 9723, false, 0);
        b.add_html_localized_color(60, 215, 280, 20, 1049006, WHITE, false, false); // No, I don't want to quit.

        b.add_button(265, 220, 247, 248, 1);
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 1 {
            let cancel = info.is_switched(1);
            with_quest_of(w, self.player, self.class, |w, q| {
                q.end_cancel_quest(w, cancel)
            });
        }
    }
}

// ---- QuestLogUpdatedGump ----------------------------------------------------------------------------------------------------

pub struct LogUpdatedLogic {
    player: MobileId,
    class: &'static QClass,
}

/// `QuestLogUpdatedGump.DisplayTo(from, system)`.
pub fn log_updated_display(w: &mut World, q: &Quest) {
    if connected(w, q.from) {
        send(
            w,
            q.from,
            3,
            30,
            LogUpdatedLogic {
                player: q.from,
                class: q.class,
            },
        );
    }
}

impl DynamicGumpLogic for LogUpdatedLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Quests.QuestLogUpdatedGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_image(20, 5, 1417, 0);
        b.add_html_localized_color(0, 78, 120, 40, 1049079, WHITE, false, false); // Quest Log Updated
        b.add_image_tiled(0, 78, 120, 40, 2624);
        b.add_alpha_region(0, 78, 120, 40);
        b.add_button(30, 15, 5575, 5576, 1);
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 1 {
            with_quest_of(w, self.player, self.class, |w, q| q.show_quest_log(w));
        }
    }
}

// ---- QuestObjectivesGump ------------------------------------------------------------------------------------------------------

pub struct ObjectivesLogic {
    player: MobileId,
    quest: Quest,
}

/// `QuestObjectivesGump.DisplayTo(from, objectives)` for the quest's whole list.
pub fn objectives_display(w: &mut World, q: &Quest) {
    if q.objectives.is_empty() || !connected(w, q.from) {
        return;
    }
    send(
        w,
        q.from,
        90,
        50,
        ObjectivesLogic {
            player: q.from,
            quest: q.clone(),
        },
    );
}

impl DynamicGumpLogic for ObjectivesLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Quests.QuestObjectivesGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();
        b.add_page(0);

        b.add_image(0, 0, 3600, 0);
        b.add_image_tiled(0, 14, 15, 375, 3603);
        b.add_image_tiled(380, 14, 14, 375, 3605);
        b.add_image(0, 376, 3606, 0);
        b.add_image_tiled(15, 376, 370, 16, 3607);
        b.add_image_tiled(15, 0, 370, 16, 3601);
        b.add_image(380, 0, 3602, 0);
        b.add_image(380, 376, 3608, 0);

        b.add_image_tiled(15, 15, 365, 365, 2624);
        b.add_alpha_region(15, 15, 365, 365);

        b.add_image(20, 87, 1231, 0);
        b.add_image(75, 62, 9307, 0);

        b.add_html_localized_color(117, 35, 230, 20, 1046026, BLUE, false, false); // Quest Log

        b.add_image(77, 33, 9781, 0);
        b.add_image(65, 110, 2104, 0);

        b.add_html_localized_color(79, 106, 230, 20, 1049073, BLUE, false, false); // Objective:

        b.add_image_tiled(68, 125, 120, 1, 9101);
        b.add_image(65, 240, 2104, 0);

        b.add_html_localized_color(79, 237, 230, 20, 1049076, BLUE, false, false); // Progress details:

        b.add_image_tiled(68, 255, 120, 1, 9101);
        b.add_button(175, 355, 2313, 2312, 1);

        b.add_image(341, 15, 10450, 0);
        b.add_image(341, 330, 10450, 0);
        b.add_image(15, 330, 10450, 0);
        b.add_image(15, 15, 10450, 0);

        b.add_page(1);

        let list = &self.quest.objectives;
        for i in 0..list.len() {
            let obj = &list[list.len() - 1 - i];
            let i = i as i32;

            if i > 0 {
                b.add_button_type(55, 346, 9909, 9911, 0, GumpButtonType::Page, 1 + i);
                b.add_html_localized_color(82, 347, 50, 20, 1043354, WHITE, false, false); // Previous

                b.add_page(1 + i);
            }

            (obj.class.render_message)(&self.quest, obj, b);
            (obj.class.render_progress)(&self.quest, obj, b);

            if i > 0 {
                b.add_button_type(317, 346, 9903, 9905, 0, GumpButtonType::Page, i);
                b.add_html_localized_color(278, 347, 50, 20, 1043353, WHITE, false, false); // Next
            }
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, _info: &RelayInfo) {
        let class = self.quest.class;
        with_quest_of(w, self.player, class, |w, q| {
            for i in (0..q.objectives.len()).rev() {
                if !q.objectives[i].has_been_read {
                    q.objectives[i].has_been_read = true;
                    let f = q.objectives[i].class.on_read;
                    f(w, q, i);
                }
            }
        });
    }
}

// ---- QuestConversationsGump -----------------------------------------------------------------------------------------------------

pub struct ConversationsLogic {
    player: MobileId,
    quest: Quest,
    /// What the gump shows: the quest's log, or the one unlogged conversation.
    list: Vec<Conversation>,
    /// The list is the quest's own `Conversations` (a response marks those), not one conversation.
    live: bool,
}

/// `QuestConversationsGump.DisplayTo(from, conversations)` for the quest's log.
pub fn conversations_display_all(w: &mut World, q: &Quest) {
    if q.conversations.is_empty() || !connected(w, q.from) {
        return;
    }
    let logic = ConversationsLogic {
        player: q.from,
        quest: q.clone(),
        list: q.conversations.clone(),
        live: true,
    };
    send(w, q.from, 30, 50, logic);
}

/// `QuestConversationsGump.DisplayTo(from, conv)` for one that is not logged.
pub fn conversation_display_one(w: &mut World, q: &Quest, conv: Conversation) {
    if !connected(w, q.from) {
        return;
    }
    send(
        w,
        q.from,
        30,
        50,
        ConversationsLogic {
            player: q.from,
            quest: q.clone(),
            list: vec![conv],
            live: false,
        },
    );
}

impl DynamicGumpLogic for ConversationsLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Quests.QuestConversationsGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();
        b.add_page(0);

        b.add_image(349, 10, 9392, 0);
        b.add_image_tiled(349, 130, 100, 120, 9395);
        b.add_image_tiled(149, 10, 200, 140, 9391);
        b.add_image_tiled(149, 250, 200, 140, 9397);
        b.add_image(349, 250, 9398, 0);
        b.add_image(35, 10, 9390, 0);
        b.add_image_tiled(35, 150, 120, 100, 9393);
        b.add_image(35, 250, 9396, 0);

        b.add_html_localized_color(110, 60, 200, 20, 1049069, WHITE, false, false); // <STRONG>Conversation Event</STRONG>

        b.add_image(65, 14, 10102, 0);
        b.add_image_tiled(81, 14, 349, 17, 10101);
        b.add_image(426, 14, 10104, 0);

        b.add_image_tiled(55, 40, 388, 323, 2624);
        b.add_alpha_region(55, 40, 388, 323);

        b.add_image_tiled(75, 90, 200, 1, 9101);
        b.add_image(75, 58, 9781, 0);
        b.add_image(380, 45, 223, 0);

        b.add_button(220, 335, 2313, 2312, 1);
        b.add_image(0, 0, 10440, 0);

        b.add_page(1);

        let n = self.list.len();
        for i in 0..n {
            let conv = &self.list[n - 1 - i];
            let i = i as i32;

            if i > 0 {
                b.add_button_type(65, 366, 9909, 9911, 0, GumpButtonType::Page, 1 + i);
                b.add_html_localized_color(90, 367, 50, 20, 1043354, BLACK, false, false); // Previous

                b.add_page(1 + i);
            }

            let message = (conv.class.message)(&self.quest, conv);
            add_html_object(
                b,
                70,
                110,
                365,
                220,
                Msg::Num(message),
                LIGHT_GREEN,
                false,
                true,
            );

            if i > 0 {
                b.add_button_type(420, 366, 9903, 9905, 0, GumpButtonType::Page, i);
                b.add_html_localized_color(370, 367, 50, 20, 1043353, BLACK, false, false); // Next
            }
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, _info: &RelayInfo) {
        let class = self.quest.class;
        if self.live {
            with_quest_of(w, self.player, class, |w, q| {
                for i in (0..q.conversations.len()).rev() {
                    if !q.conversations[i].has_been_read {
                        q.conversations[i].has_been_read = true;
                        let c = q.conversations[i].clone();
                        let f = c.class.on_read;
                        f(w, q, &c);
                    }
                }
            });
        } else {
            let mut conv = self.list[0].clone();
            if !conv.has_been_read {
                conv.has_been_read = true;
                let f = conv.class.on_read;
                with_quest_of(w, self.player, class, |w, q| f(w, q, &conv));
            }
        }
    }
}

// ---- QuestItemInfoGump ------------------------------------------------------------------------------------------------------------

pub struct ItemInfoLogic {
    info: &'static [ItemInfo],
}

/// `QuestItemInfoGump.DisplayTo(from, info)`.
pub fn item_info_display(w: &mut World, to: MobileId, info: &'static [ItemInfo]) {
    if info.is_empty() || !connected(w, to) {
        return;
    }
    send(w, to, 485, 75, ItemInfoLogic { info });
}

impl DynamicGumpLogic for ItemInfoLogic {
    const TYPE_NAME: &'static str = "Server.Engines.Quests.QuestItemInfoGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let height = 100 + self.info.len() as i32 * 75;

        b.add_page(0);

        b.add_background(5, 10, 145, height, 5054);

        b.add_image_tiled(13, 20, 125, 10, 2624);
        b.add_alpha_region(13, 20, 125, 10);

        b.add_image_tiled(13, height - 10, 128, 10, 2624);
        b.add_alpha_region(13, height - 10, 128, 10);

        b.add_image_tiled(13, 20, 10, height - 30, 2624);
        b.add_alpha_region(13, 20, 10, height - 30);

        b.add_image_tiled(131, 20, 10, height - 30, 2624);
        b.add_alpha_region(131, 20, 10, height - 30);

        b.add_html_localized_color(67, 35, 120, 20, 1011233, WHITE, false, false); // INFO

        b.add_image(62, 52, 9157, 0);
        b.add_image(72, 52, 9157, 0);
        b.add_image(82, 52, 9157, 0);

        b.add_button(25, 31, 1209, 1210, 777);

        b.add_page(1);

        for (i, cur) in self.info.iter().enumerate() {
            let i = i as i32;
            add_html_object(b, 25, 65 + i * 75, 110, 20, cur.name, 1153, false, false);
            b.add_item(45, 85 + i * 75, cur.item_id, 0);
        }
    }
}
