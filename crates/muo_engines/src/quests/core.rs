//! The quest engine core: `QuestSystem`, `QuestObjective`, `QuestConversation`, `QuestSerializer`,
//! `QuestItemInfo`, `QuestRestartInfo` and `QuestCallbackEntry`.
//!
//! A quest in progress is stored on the player as the bytes `QuestSerializer.Serialize` writes
//! (`PlayerData::quest`), so a resave is exact whatever the engine does. The engine decodes it into a [`Quest`]
//! when something needs it ([`with_quest`]) and writes it back afterwards. A quest offered but not accepted yet
//! is a free-standing [`Quest`] owned by its offer gump.
//!
//! Per-class state (`m_Victory`, `Step`, ...) is a boxed value of a type the class defines ([`St`]); the class
//! table ([`ObjClass`], [`ConvClass`], [`QClass`]) holds every virtual as a `fn`, defaults being the base class's.

use std::any::Any;
use std::rc::Rc;

use muo_content::gumps::DynamicGumpBuilder;
use muo_content::player_mobile::quest_tables::SYSTEMS;
use muo_content::player_mobile::{self as pmob, QuestTypeRef};
use muo_io::{GenericReader, GenericWriter, IoError, Result as IoResult, SaveReader, SaveWriter};
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::timer::{self, TimerHandle};

use super::gumps;

// ---- messages and item info ---------------------------------------------------------------------------------

/// The `object` of `QuestSystem.Name`, `OfferMessage` and `QuestItemInfo.Name`: a cliloc number or a plain text.
#[derive(Clone, Copy, Debug)]
pub enum Msg {
    Num(i32),
    Text(&'static str),
}

/// `QuestItemInfo`.
#[derive(Clone, Copy, Debug)]
pub struct ItemInfo {
    pub name: Msg,
    pub item_id: i32,
}

/// `new QuestItemInfo(number, itemID)`.
pub const fn info(name: i32, item_id: i32) -> ItemInfo {
    ItemInfo {
        name: Msg::Num(name),
        item_id,
    }
}

// ---- per-class state ----------------------------------------------------------------------------------------

pub trait QState: Any + Send + Sync {
    fn box_clone(&self) -> Box<dyn QState>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any + Clone + Send + Sync> QState for T {
    fn box_clone(&self) -> Box<dyn QState> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// The fields a class adds to its base: none, or one value of a type the class chose.
#[derive(Default)]
pub struct St(Option<Box<dyn QState>>);

impl Clone for St {
    fn clone(&self) -> Self {
        St(self.0.as_deref().map(|s| s.box_clone()))
    }
}

impl St {
    pub fn new<T: Any + Clone + Send + Sync>(value: T) -> St {
        St(Some(Box::new(value)))
    }
    /// The state of a class that has one: the class table's `new_state` always makes it.
    pub fn get<T: Any>(&self) -> &T {
        self.0
            .as_deref()
            .and_then(|s| s.as_any().downcast_ref())
            .expect("quest class state")
    }
    pub fn get_mut<T: Any>(&mut self) -> &mut T {
        self.0
            .as_deref_mut()
            .and_then(|s| s.as_any_mut().downcast_mut())
            .expect("quest class state")
    }
}

pub fn no_state() -> St {
    St(None)
}

// ---- objectives -----------------------------------------------------------------------------------------------

/// `QuestObjective`.
#[derive(Clone)]
pub struct Objective {
    pub class: &'static ObjClass,
    pub has_been_read: bool,
    /// `CurProgress` is read here; write it with [`Quest::set_progress`].
    pub cur_progress: i32,
    pub has_completed: bool,
    pub st: St,
}

impl Objective {
    pub fn new(class: &'static ObjClass) -> Objective {
        Objective {
            class,
            has_been_read: false,
            cur_progress: 0,
            has_completed: false,
            st: (class.new_state)(),
        }
    }
    /// A class constructor that sets a field of its own.
    pub fn with<T: Any + Clone + Send + Sync>(class: &'static ObjClass, state: T) -> Objective {
        Objective {
            st: St::new(state),
            ..Objective::new(class)
        }
    }
    /// `new Objective(startingProgress)`: the constructor sets `CurProgress` before it has a system.
    pub fn with_progress(class: &'static ObjClass, progress: i32) -> Objective {
        Objective {
            cur_progress: progress,
            ..Objective::new(class)
        }
    }
    pub fn state<T: Any>(&self) -> &T {
        self.st.get()
    }
    pub fn state_mut<T: Any>(&mut self) -> &mut T {
        self.st.get_mut()
    }
    pub fn max_progress(&self, q: &Quest) -> i32 {
        (self.class.max_progress)(q, self)
    }
    /// `Completed`.
    pub fn completed(&self, q: &Quest) -> bool {
        (self.class.completed)(q, self)
    }
    /// `IsSingleObjective`.
    pub fn is_single(&self, q: &Quest) -> bool {
        self.max_progress(q) == 1
    }
}

/// The virtuals of a `QuestObjective` subclass. `OBJECTIVE` holds the base class's behavior.
pub struct ObjClass {
    pub name: &'static str,
    pub new_state: fn() -> St,
    pub message: fn(&Quest, &Objective) -> i32,
    pub max_progress: fn(&Quest, &Objective) -> i32,
    pub info: fn(&Quest, &Objective) -> &'static [ItemInfo],
    pub render_message: fn(&Quest, &Objective, &mut DynamicGumpBuilder),
    pub render_progress: fn(&Quest, &Objective, &mut DynamicGumpBuilder),
    pub completed: fn(&Quest, &Objective) -> bool,
    pub on_read: fn(&mut World, &mut Quest, usize),
    pub get_timer_event: fn(&Quest, &Objective) -> bool,
    pub check_progress: fn(&mut World, &mut Quest, usize),
    pub on_complete: fn(&mut World, &mut Quest, usize),
    pub get_kill_event: fn(&World, &Quest, usize, MobileId, Option<ItemId>) -> bool,
    pub on_kill: fn(&mut World, &mut Quest, usize, MobileId, Option<ItemId>),
    pub ignore_young_protection: fn(&World, &Quest, usize, MobileId) -> bool,
    pub child_deserialize: fn(&World, &mut Objective, &mut dyn GenericReader) -> IoResult<()>,
    pub child_serialize: fn(&World, &Objective, &mut dyn GenericWriter),
}

/// `QuestObjective.Message` for a class whose message is not a constant: one cliloc per class.
fn no_message(_: &Quest, _: &Objective) -> i32 {
    0
}

fn objective_render_message(q: &Quest, o: &Objective, b: &mut DynamicGumpBuilder) {
    base_render_message(q, o, b);
}

/// `base.RenderMessage`.
pub fn base_render_message(q: &Quest, o: &Objective, b: &mut DynamicGumpBuilder) {
    let n = (o.class.message)(q, o);
    gumps::add_html_object(b, 70, 130, 300, 100, Msg::Num(n), gumps::BLUE, false, false);
}

fn objective_render_progress(q: &Quest, o: &Objective, b: &mut DynamicGumpBuilder) {
    base_render_progress(q, o, b);
}

/// `base.RenderProgress`.
pub fn base_render_progress(q: &Quest, o: &Objective, b: &mut DynamicGumpBuilder) {
    let n = if o.completed(q) { 1049077 } else { 1049078 };
    gumps::add_html_object(b, 70, 260, 270, 100, Msg::Num(n), gumps::BLUE, false, false);
}

pub fn obj_default_deserialize(
    _: &World,
    _: &mut Objective,
    r: &mut dyn GenericReader,
) -> IoResult<()> {
    r.read_encoded_int()?; // version
    Ok(())
}

pub fn obj_default_serialize(_: &World, _: &Objective, out: &mut dyn GenericWriter) {
    out.write_encoded_int(0); // version
}

/// The `QuestObjective` base class.
pub static OBJECTIVE: ObjClass = ObjClass {
    name: "Server.Engines.Quests.QuestObjective",
    new_state: no_state,
    message: no_message,
    max_progress: |_, _| 1,
    info: |_, _| &[],
    render_message: objective_render_message,
    render_progress: objective_render_progress,
    completed: |q, o| o.cur_progress >= o.max_progress(q),
    on_read: |_, _, _| {},
    get_timer_event: |q, o| !o.completed(q),
    check_progress: |_, _, _| {},
    on_complete: |_, _, _| {},
    get_kill_event: |_, q, i, _, _| !q.objectives[i].completed(q),
    on_kill: |_, _, _, _, _| {},
    ignore_young_protection: |_, _, _, _| false,
    child_deserialize: obj_default_deserialize,
    child_serialize: obj_default_serialize,
};

/// An objective class whose message is a constant.
#[macro_export]
macro_rules! objective {
    ($(#[$m:meta])* $name:ident = $full:expr, msg $msg:expr $(, $f:ident : $v:expr)* $(,)?) => {
        $(#[$m])*
        pub static $name: $crate::quests::core::ObjClass = $crate::quests::core::ObjClass {
            name: $full,
            message: |_, _| $msg,
            $($f: $v,)*
            ..$crate::quests::core::OBJECTIVE
        };
    };
    ($(#[$m:meta])* $name:ident = $full:expr $(, $f:ident : $v:expr)* $(,)?) => {
        $(#[$m])*
        pub static $name: $crate::quests::core::ObjClass = $crate::quests::core::ObjClass {
            name: $full,
            $($f: $v,)*
            ..$crate::quests::core::OBJECTIVE
        };
    };
}

// ---- conversations --------------------------------------------------------------------------------------------

/// `QuestConversation`.
#[derive(Clone)]
pub struct Conversation {
    pub class: &'static ConvClass,
    pub has_been_read: bool,
    pub st: St,
}

impl Conversation {
    pub fn new(class: &'static ConvClass) -> Conversation {
        Conversation {
            class,
            has_been_read: false,
            st: (class.new_state)(),
        }
    }
    pub fn with<T: Any + Clone + Send + Sync>(class: &'static ConvClass, state: T) -> Conversation {
        Conversation {
            st: St::new(state),
            ..Conversation::new(class)
        }
    }
    pub fn state<T: Any>(&self) -> &T {
        self.st.get()
    }
}

pub struct ConvClass {
    pub name: &'static str,
    pub new_state: fn() -> St,
    pub message: fn(&Quest, &Conversation) -> i32,
    pub info: fn(&Quest, &Conversation) -> &'static [ItemInfo],
    pub logged: fn(&Quest, &Conversation) -> bool,
    /// `OnRead`: the conversation is a copy (an unlogged one is not in the quest).
    pub on_read: fn(&mut World, &mut Quest, &Conversation),
    pub child_deserialize: fn(&World, &mut Conversation, &mut dyn GenericReader) -> IoResult<()>,
    pub child_serialize: fn(&World, &Conversation, &mut dyn GenericWriter),
}

pub fn conv_default_deserialize(
    _: &World,
    _: &mut Conversation,
    r: &mut dyn GenericReader,
) -> IoResult<()> {
    r.read_encoded_int()?;
    Ok(())
}

pub fn conv_default_serialize(_: &World, _: &Conversation, out: &mut dyn GenericWriter) {
    out.write_encoded_int(0);
}

/// `QuestConversation.Logged => false`.
pub fn never_logged(_: &Quest, _: &Conversation) -> bool {
    false
}

/// The `QuestConversation` base class.
pub static CONVERSATION: ConvClass = ConvClass {
    name: "Server.Engines.Quests.QuestConversation",
    new_state: no_state,
    message: |_, _| 0,
    info: |_, _| &[],
    logged: |_, _| true,
    on_read: |_, _, _| {},
    child_deserialize: conv_default_deserialize,
    child_serialize: conv_default_serialize,
};

#[macro_export]
macro_rules! conversation {
    ($(#[$m:meta])* $name:ident = $full:expr, msg $msg:expr $(, $f:ident : $v:expr)* $(,)?) => {
        $(#[$m])*
        pub static $name: $crate::quests::core::ConvClass = $crate::quests::core::ConvClass {
            name: $full,
            message: |_, _| $msg,
            $($f: $v,)*
            ..$crate::quests::core::CONVERSATION
        };
    };
    ($(#[$m:meta])* $name:ident = $full:expr $(, $f:ident : $v:expr)* $(,)?) => {
        $(#[$m])*
        pub static $name: $crate::quests::core::ConvClass = $crate::quests::core::ConvClass {
            name: $full,
            $($f: $v,)*
            ..$crate::quests::core::CONVERSATION
        };
    };
}

// ---- the common one-bool child state ----

/// `new_state` of a class whose only field is a `bool` (saved as version 0, the bool).
pub fn bool_state() -> St {
    St::new(false)
}

pub fn obj_bool_deserialize(
    _: &World,
    o: &mut Objective,
    r: &mut dyn GenericReader,
) -> IoResult<()> {
    r.read_encoded_int()?; // version
    *o.state_mut::<bool>() = r.read_bool()?;
    Ok(())
}

pub fn obj_bool_serialize(_: &World, o: &Objective, out: &mut dyn GenericWriter) {
    out.write_encoded_int(0); // version
    out.write_bool(*o.state::<bool>());
}

pub fn conv_bool_deserialize(
    _: &World,
    c: &mut Conversation,
    r: &mut dyn GenericReader,
) -> IoResult<()> {
    r.read_encoded_int()?; // version
    *c.st.get_mut::<bool>() = r.read_bool()?;
    Ok(())
}

pub fn conv_bool_serialize(_: &World, c: &Conversation, out: &mut dyn GenericWriter) {
    out.write_encoded_int(0); // version
    out.write_bool(*c.state::<bool>());
}

// ---- the quest ----------------------------------------------------------------------------------------------------

/// The virtuals of a `QuestSystem` subclass.
pub struct QClass {
    pub name: &'static str,
    /// Index in `QuestSystem.QuestTypes` (its `TypeReferenceTable` is the one in `quest_tables::SYSTEMS`).
    pub slot: usize,
    pub name_msg: Msg,
    pub offer_message: fn(&World, &Quest) -> Msg,
    pub picture: fn(&World, &Quest) -> i32,
    pub is_tutorial: bool,
    pub restart_delay: TimeSpan,
    /// The `Profession` values `CanOfferQuest` accepts for this quest (empty: any).
    pub professions: &'static [i32],
    pub new_state: fn() -> St,
    pub accept: fn(&mut World, &mut Quest),
    pub decline: fn(&mut World, &mut Quest),
    pub slice: fn(&mut World, &mut Quest),
    pub cancel: fn(&mut World, &mut Quest),
    pub ignore_young_protection: fn(&World, &Quest, MobileId) -> bool,
    pub child_deserialize: fn(&World, &mut Quest, &mut dyn GenericReader) -> IoResult<()>,
    pub child_serialize: fn(&World, &Quest, &mut dyn GenericWriter),
}

fn q_default_deserialize(_: &World, _: &mut Quest, r: &mut dyn GenericReader) -> IoResult<()> {
    r.read_encoded_int()?;
    Ok(())
}

fn q_default_serialize(_: &World, _: &Quest, out: &mut dyn GenericWriter) {
    out.write_encoded_int(0);
}

/// The `QuestSystem` base class.
pub static QUEST_SYSTEM: QClass = QClass {
    name: "Server.Engines.Quests.QuestSystem",
    slot: usize::MAX,
    name_msg: Msg::Num(0),
    offer_message: |_, _| Msg::Num(0),
    picture: |_, _| 0,
    is_tutorial: false,
    restart_delay: TimeSpan::ZERO,
    professions: &[],
    new_state: no_state,
    accept: base_accept,
    decline: base_decline,
    slice: base_slice,
    cancel: |w, q| q.clear_quest(w, false),
    ignore_young_protection: base_ignore_young_protection,
    child_deserialize: q_default_deserialize,
    child_serialize: q_default_serialize,
};

/// `QuestSystem`.
#[derive(Clone)]
pub struct Quest {
    pub class: &'static QClass,
    pub from: MobileId,
    pub objectives: Vec<Objective>,
    pub conversations: Vec<Conversation>,
    pub st: St,
    /// `From.Quest == this`.
    pub attached: bool,
    /// `From.Profession` when the quest was loaded or made (gump text is built without the world).
    pub profession: i32,
}

impl Quest {
    /// `new Quest(from)`.
    pub fn new(w: &World, class: &'static QClass, from: MobileId) -> Quest {
        Quest {
            class,
            from,
            objectives: Vec::new(),
            conversations: Vec::new(),
            st: (class.new_state)(),
            attached: false,
            profession: pmob::data(w, from).map_or(0, |d| d.profession),
        }
    }
    pub fn with<T: Any + Clone + Send + Sync>(
        w: &World,
        class: &'static QClass,
        from: MobileId,
        state: T,
    ) -> Quest {
        Quest {
            st: St::new(state),
            ..Quest::new(w, class, from)
        }
    }
    pub fn state<T: Any>(&self) -> &T {
        self.st.get()
    }
    pub fn state_mut<T: Any>(&mut self) -> &mut T {
        self.st.get_mut()
    }
    pub fn is(&self, class: &QClass) -> bool {
        std::ptr::eq(self.class, class)
    }

    /// `FindObjective(type)`: the last objective of exactly this class.
    pub fn find_objective(&self, class: &ObjClass) -> Option<usize> {
        self.objectives
            .iter()
            .rposition(|o| std::ptr::eq(o.class, class))
    }
    /// `FindObjective<T>()` over several classes (a base type).
    pub fn find_objective_in(&self, classes: &[&ObjClass]) -> Option<usize> {
        self.objectives
            .iter()
            .rposition(|o| classes.iter().any(|c| std::ptr::eq(o.class, *c)))
    }
    /// `FindObjective(type)?.Completed == false`... `IsObjectiveInProgress`.
    pub fn is_objective_in_progress(&self, class: &ObjClass) -> bool {
        self.find_objective(class)
            .is_some_and(|i| !self.objectives[i].completed(self))
    }
    /// `FindObjective(type) != null`.
    pub fn has_objective(&self, class: &ObjClass) -> bool {
        self.find_objective(class).is_some()
    }

    // ---- the objective calls ----

    /// `CurProgress = value`.
    pub fn set_progress(&mut self, w: &mut World, i: usize, value: i32) {
        self.objectives[i].cur_progress = value;
        self.check_completion_status(w, i);
    }
    /// `CurProgress++`.
    pub fn add_progress(&mut self, w: &mut World, i: usize) {
        let v = self.objectives[i].cur_progress + 1;
        self.set_progress(w, i, v);
    }
    /// `QuestObjective.Complete()`.
    pub fn complete_objective(&mut self, w: &mut World, i: usize) {
        let max = self.objectives[i].max_progress(self);
        self.set_progress(w, i, max);
    }
    /// `CheckCompletionStatus()`.
    pub fn check_completion_status(&mut self, w: &mut World, i: usize) {
        if self.objectives[i].completed(self) && !self.objectives[i].has_completed {
            self.objectives[i].has_completed = true;
            let f = self.objectives[i].class.on_complete;
            f(w, self, i);
        }
    }

    // ---- QuestSystem ----

    pub fn slice(&mut self, w: &mut World) {
        for i in (0..self.objectives.len()).rev() {
            if (self.objectives[i].class.get_timer_event)(self, &self.objectives[i]) {
                let f = self.objectives[i].class.check_progress;
                f(w, self, i);
            }
        }
    }

    /// `OnKill(creature, corpse)`.
    pub fn on_kill(&mut self, w: &mut World, creature: MobileId, corpse: Option<ItemId>) {
        for i in (0..self.objectives.len()).rev() {
            let c = self.objectives[i].class;
            if (c.get_kill_event)(w, self, i, creature, corpse) {
                (c.on_kill)(w, self, i, creature, corpse);
            }
        }
    }

    pub fn ignore_young_protection(&self, w: &World, from: MobileId) -> bool {
        (self.class.ignore_young_protection)(w, self, from)
    }

    pub fn add_conversation(&mut self, w: &mut World, conv: Conversation) {
        let logged = (conv.class.logged)(self, &conv);
        let info = (conv.class.info)(self, &conv);
        let single = if logged { None } else { Some(conv.clone()) };
        if logged {
            self.conversations.push(conv);
        }

        if let Some(ns) = muo_server::mobile::part1::props_b::net_state(w, self.from) {
            muo_content::gumps::close_gump::<gumps::ItemInfoLogic>(w, ns);
            muo_content::gumps::close_gump::<gumps::ObjectivesLogic>(w, ns);
        }

        match single {
            None => gumps::conversations_display_all(w, self),
            Some(c) => gumps::conversation_display_one(w, self, c),
        }
        gumps::item_info_display(w, self.from, info);
    }

    pub fn add_objective(&mut self, w: &mut World, obj: Objective) {
        self.objectives.push(obj);
        gumps::log_updated_display(w, self);
    }

    /// `ShowQuestLog()`.
    pub fn show_quest_log(&self, w: &mut World) {
        if self.objectives.is_empty() {
            return;
        }
        if let Some(ns) = muo_server::mobile::part1::props_b::net_state(w, self.from) {
            muo_content::gumps::close_gump::<gumps::ItemInfoLogic>(w, ns);
            muo_content::gumps::close_gump::<gumps::LogUpdatedLogic>(w, ns);
            muo_content::gumps::close_gump::<gumps::ConversationsLogic>(w, ns);
            muo_content::gumps::close_gump::<gumps::ObjectivesLogic>(w, ns);
        }
        gumps::objectives_display(w, self);
        if let Some(last) = self.objectives.last() {
            let info = (last.class.info)(self, last);
            gumps::item_info_display(w, self.from, info);
        }
    }

    /// `ShowQuestConversation()`.
    pub fn show_quest_conversation(&self, w: &mut World) {
        if self.conversations.is_empty() {
            return;
        }
        if let Some(ns) = muo_server::mobile::part1::props_b::net_state(w, self.from) {
            muo_content::gumps::close_gump::<gumps::ItemInfoLogic>(w, ns);
            muo_content::gumps::close_gump::<gumps::ObjectivesLogic>(w, ns);
            muo_content::gumps::close_gump::<gumps::ConversationsLogic>(w, ns);
        }
        gumps::conversations_display_all(w, self);
        if let Some(last) = self.conversations.last() {
            let info = (last.class.info)(self, last);
            gumps::item_info_display(w, self.from, info);
        }
    }

    /// `EndCancelQuest(shouldCancel)`.
    pub fn end_cancel_quest(&mut self, w: &mut World, should_cancel: bool) {
        if !self.attached {
            return;
        }
        if should_cancel {
            muo_content::mx::msg(w, self.from, 1049015); // You have canceled your quest.
            let f = self.class.cancel;
            f(w, self);
        } else {
            muo_content::mx::msg(w, self.from, 1049014); // You have chosen not to cancel your quest.
        }
    }

    pub fn complete(&mut self, w: &mut World) {
        self.clear_quest(w, true);
    }

    /// `ClearQuest(completed)`.
    pub fn clear_quest(&mut self, w: &mut World, completed: bool) {
        stop_timer(w, self.from);
        if !self.attached {
            return;
        }
        self.attached = false;
        let from = self.from;
        if let Some(d) = pmob::data_mut(w, from) {
            d.quest = None;
        }
        set_live(w, from, None);

        let delay = self.class.restart_delay;
        if (completed && delay > TimeSpan::ZERO) || (!completed && delay == TimeSpan::MAX) {
            let restart = if delay < TimeSpan::MAX {
                // `Core.Now + restartDelay` throws on overflow: the quest stays cleared, nothing is recorded.
                let Some(t) = w.now.checked_add(delay) else {
                    return;
                };
                t
            } else {
                DateTime::MAX
            };
            let class = self.class;
            if let Some(d) = pmob::data_mut(w, from) {
                match d
                    .done_quests
                    .iter_mut()
                    .find(|(t, _)| done_matches(t, class))
                {
                    Some(e) => e.1 = restart,
                    None => d
                        .done_quests
                        .push((QuestTypeRef::Indexed(class.slot as i32), restart)),
                }
            }
        }
    }

    /// `SendOffer()`.
    pub fn send_offer(&self, w: &mut World) {
        gumps::offer_display(w, self);
    }

    /// `BeginCancelQuest()`.
    pub fn begin_cancel_quest(&self, w: &mut World) {
        gumps::cancel_display(w, self);
    }
}

/// `QuestSystem.Accept()`.
pub fn base_accept(w: &mut World, q: &mut Quest) {
    let from = q.from;
    if pmob::data(w, from).is_some_and(|d| d.quest.is_some()) {
        return;
    }
    q.attached = true;
    muo_content::mx::msg(w, from, 1049019); // You have accepted the Quest.
    start_timer(w, from);
}

/// `QuestSystem.Decline()`.
pub fn base_decline(w: &mut World, q: &mut Quest) {
    muo_content::mx::msg(w, q.from, 1049018); // You have declined the Quest.
}

/// `QuestSystem.Slice()`.
pub fn base_slice(w: &mut World, q: &mut Quest) {
    q.slice(w);
}

pub fn base_ignore_young_protection(w: &World, q: &Quest, from: MobileId) -> bool {
    (0..q.objectives.len())
        .rev()
        .any(|i| (q.objectives[i].class.ignore_young_protection)(w, q, i, from))
}

/// `Quest` matched `QuestType == ourQuestType` in a `DoneQuests` entry.
fn done_matches(t: &QuestTypeRef, class: &QClass) -> bool {
    match t {
        QuestTypeRef::Indexed(i) => usize::try_from(*i)
            .ok()
            .and_then(|i| SYSTEMS.get(i))
            .is_some_and(|s| s.name == class.name),
        QuestTypeRef::Named(n) => n == class.name,
        QuestTypeRef::None => false,
    }
}

/// `QuestSystem.CanOfferQuest(check, questType, out inRestartPeriod)`.
pub fn can_offer_quest_ex(w: &mut World, check: MobileId, class: &'static QClass) -> (bool, bool) {
    if !w
        .mobile(check)
        .is_some_and(|m| m.ty.is(&pmob::PLAYER_MOBILE))
    {
        return (false, false);
    }
    if muo_server::mobile::part1::props_b::net_state(w, check)
        .is_some_and(|ns| muo_content::gumps::has_gump::<gumps::OfferLogic>(w, ns))
    {
        return (false, false);
    }
    let profession = pmob::data(w, check).map_or(0, |d| d.profession);
    if !class.professions.is_empty() && !class.professions.contains(&profession) {
        return (false, false);
    }

    let now = w.now;
    let Some(d) = pmob::data_mut(w, check) else {
        return (false, false);
    };
    if let Some(i) = d
        .done_quests
        .iter()
        .position(|(t, _)| done_matches(t, class))
    {
        if now < d.done_quests[i].1 {
            return (false, true);
        }
        d.done_quests.remove(i);
        return (true, false);
    }
    (true, false)
}

/// `QuestSystem.CanOfferQuest(check, questType)`.
pub fn can_offer_quest(w: &mut World, check: MobileId, class: &'static QClass) -> bool {
    can_offer_quest_ex(w, check, class).0
}

/// `QuestSystem.FocusTo(who, to)`.
pub fn focus_to(w: &mut World, who: MobileId, to: MobileId) {
    if muo_content::hooks::rnd::bool(w) {
        muo_content::mx::animate(w, who, 17, 7, 1, true, false, 0);
    } else {
        let a = 32 + muo_content::hooks::rnd::below(w, 3);
        muo_content::mx::animate(w, who, a, 7, 1, true, false, 0);
    }
    let d = muo_content::mx::get_direction_to(w, who, to);
    muo_content::mx::set_direction(w, who, d);
}

/// `QuestSystem.RandomBrightHue()`.
pub fn random_bright_hue(w: &mut World) -> i32 {
    use muo_content::hooks::rnd;
    if rnd::double(w) < 0.1 {
        return if rnd::below(w, 2) == 0 { 0x62 } else { 0x71 };
    }
    const HUES: [i32; 10] = [0x03, 0x0D, 0x13, 0x1C, 0x21, 0x30, 0x37, 0x3A, 0x44, 0x59];
    HUES[rnd::below(w, HUES.len() as i32) as usize]
}

// ---- the timer ------------------------------------------------------------------------------------------------------

/// The player's quest timer (`_timerToken`), in the mobile's `Ext`.
#[derive(Default, Clone, Copy)]
struct QuestTimer(TimerHandle);

fn timer_of(w: &World, m: MobileId) -> TimerHandle {
    w.mobile(m)
        .and_then(|m| m.ext.get::<QuestTimer>())
        .map_or_else(TimerHandle::default, |t| t.0)
}

fn set_timer(w: &mut World, m: MobileId, h: TimerHandle) {
    if let Some(mob) = w.mobile_mut(m) {
        mob.ext.insert(QuestTimer(h));
    }
}

/// `StartTimer()`: every half second the quest's `Slice` runs.
pub fn start_timer(w: &mut World, m: MobileId) {
    if timer_of(w, m).running(w) {
        return;
    }
    let half = TimeSpan::from_millis(500);
    let h = timer::start(w, half, half, 0, move |w| {
        if with_quest(w, m, |w, q| (q.class.slice)(w, q)).is_none() {
            stop_timer(w, m);
        }
    });
    set_timer(w, m, h);
}

/// `StopTimer()`.
pub fn stop_timer(w: &mut World, m: MobileId) {
    let mut h = timer_of(w, m);
    h.cancel(w);
    set_timer(w, m, h);
}

// ---- storage ----------------------------------------------------------------------------------------------------------

/// The decoded quest of a player with the bytes it was stored as: fields a class never saves (a study timer, the
/// nest being studied) live on in it between calls. It is valid while the player's record still holds `bytes`.
#[derive(Default)]
struct LiveQuest(Option<(Vec<u8>, Quest)>);

/// `pm.Quest`, decoded: `None` without a quest (or one that cannot be read).
pub fn load(w: &World, pm: MobileId) -> Option<Quest> {
    let bytes = pmob::data(w, pm)?.quest.as_ref()?;
    if let Some((b, q)) = w
        .mobile(pm)
        .and_then(|m| m.ext.get::<LiveQuest>())
        .and_then(|l| l.0.as_ref())
        && b == bytes
    {
        return Some(q.clone());
    }
    let mut r = SaveReader::new(bytes);
    let mut q = read_quest(w, &mut r, pm).ok()?;
    q.attached = true;
    Some(q)
}

fn set_live(w: &mut World, pm: MobileId, live: Option<(Vec<u8>, Quest)>) {
    if let Some(m) = w.mobile_mut(pm) {
        m.ext.insert(LiveQuest(live));
    }
}

/// `pm.Quest = q` (the bytes `QuestSerializer.Serialize` writes after its leading 1).
pub fn store(w: &mut World, pm: MobileId, q: &Quest) {
    let mut out = SaveWriter::new(true);
    write_quest(w, q, &mut out);
    let bytes = out.into_vec();
    if let Some(d) = pmob::data_mut(w, pm) {
        d.quest = Some(bytes.clone());
    }
    set_live(w, pm, Some((bytes, q.clone())));
}

/// Runs `f` on the player's quest and keeps what it did, unless it cleared the quest.
pub fn with_quest<R>(
    w: &mut World,
    pm: MobileId,
    f: impl FnOnce(&mut World, &mut Quest) -> R,
) -> Option<R> {
    let mut q = load(w, pm)?;
    let r = f(w, &mut q);
    if q.attached {
        store(w, pm, &q);
    }
    Some(r)
}

/// As [`with_quest`] but only when the player's quest is of `class` (`pm.Quest is T`).
pub fn with_quest_of<R>(
    w: &mut World,
    pm: MobileId,
    class: &'static QClass,
    f: impl FnOnce(&mut World, &mut Quest) -> R,
) -> Option<R> {
    if !pmob::data(w, pm).is_some_and(|d| d.quest.is_some()) {
        return None;
    }
    let mut q = load(w, pm)?;
    if !q.is(class) {
        return None;
    }
    let r = f(w, &mut q);
    if q.attached {
        store(w, pm, &q);
    }
    Some(r)
}

/// `pm.Quest is T`, without decoding more than the leading type.
pub fn quest_is(w: &World, pm: MobileId, class: &QClass) -> bool {
    let Some(bytes) = pmob::data(w, pm).and_then(|d| d.quest.as_ref()) else {
        return false;
    };
    let mut r = SaveReader::new(bytes);
    let names: Vec<&str> = SYSTEMS.iter().map(|s| s.name).collect();
    matches!(read_type_name(&mut r, &names), Ok(Some(n)) if n == class.name)
}

/// `pm.Quest != null`.
pub fn has_quest(w: &World, pm: MobileId) -> bool {
    pmob::data(w, pm).is_some_and(|d| d.quest.is_some())
}

/// `new QuestSystem(pm)` accepted at once (`pm.Quest = qs`), for the code that attaches without an offer.
pub fn attach(w: &mut World, q: &mut Quest) {
    q.attached = true;
    let pm = q.from;
    store(w, pm, q);
}

// ---- QuestSerializer ------------------------------------------------------------------------------------------------------

fn bad(what: String) -> IoError {
    IoError::InvalidData(format!("quest: {what}"))
}

/// `Write(type, referenceTable, writer)`.
fn write_type(name: Option<&str>, table: &[&str], out: &mut dyn GenericWriter) {
    let Some(name) = name else {
        out.write_encoded_int(0);
        return;
    };
    match table.iter().position(|t| *t == name) {
        Some(i) => {
            out.write_encoded_int(1);
            out.write_encoded_int(i as i32);
        }
        None => {
            out.write_encoded_int(2);
            out.write_string(name);
        }
    }
}

fn read_type_name(r: &mut dyn GenericReader, table: &[&str]) -> IoResult<Option<String>> {
    match r.read_encoded_int()? {
        1 => {
            let i = r.read_encoded_int()?;
            Ok(usize::try_from(i)
                .ok()
                .and_then(|i| table.get(i))
                .map(|s| (*s).to_owned()))
        }
        2 => r.read_string(),
        _ => Ok(None),
    }
}

fn write_quest(w: &World, q: &Quest, out: &mut dyn GenericWriter) {
    let names: Vec<&str> = SYSTEMS.iter().map(|s| s.name).collect();
    write_type(Some(q.class.name), &names, out);

    let table = SYSTEMS.get(q.class.slot).map_or(&[][..], |s| s.table);
    out.write_encoded_int(0); // version
    out.write_encoded_int(q.objectives.len() as i32);
    for o in &q.objectives {
        out.write_encoded_int(1);
        write_type(Some(o.class.name), table, out);
        out.write_encoded_int(1); // version
        out.write_bool(o.has_been_read);
        out.write_encoded_int(o.cur_progress);
        out.write_bool(o.has_completed);
        (o.class.child_serialize)(w, o, out);
    }
    out.write_encoded_int(q.conversations.len() as i32);
    for c in &q.conversations {
        out.write_encoded_int(1);
        write_type(Some(c.class.name), table, out);
        out.write_encoded_int(0); // version
        out.write_bool(c.has_been_read);
        (c.class.child_serialize)(w, c, out);
    }
    (q.class.child_serialize)(w, q, out);
}

fn read_quest(w: &World, r: &mut dyn GenericReader, pm: MobileId) -> IoResult<Quest> {
    let names: Vec<&str> = SYSTEMS.iter().map(|s| s.name).collect();
    let name = read_type_name(r, &names)?.ok_or_else(|| bad("unknown quest type".into()))?;
    let class = super::quest_class(&name).ok_or_else(|| bad(format!("unknown quest {name}")))?;
    let table = SYSTEMS.get(class.slot).map_or(&[][..], |s| s.table);

    let mut q = Quest::new(w, class, pm);
    if r.read_encoded_int()? == 0 {
        let count = r.read_encoded_int()?;
        for _ in 0..count.max(0) {
            if r.read_encoded_int()? != 1 {
                continue; // a null objective
            }
            let n =
                read_type_name(r, table)?.ok_or_else(|| bad("unknown objective type".into()))?;
            let c =
                super::objective_class(&n).ok_or_else(|| bad(format!("unknown objective {n}")))?;
            let mut o = Objective::new(c);
            let version = r.read_encoded_int()?;
            if version == 1 {
                o.has_been_read = r.read_bool()?;
            }
            if version == 0 || version == 1 {
                o.cur_progress = r.read_encoded_int()?;
                o.has_completed = r.read_bool()?;
            }
            (c.child_deserialize)(w, &mut o, r)?;
            q.objectives.push(o);
        }
        let count = r.read_encoded_int()?;
        for _ in 0..count.max(0) {
            if r.read_encoded_int()? != 1 {
                continue;
            }
            let n =
                read_type_name(r, table)?.ok_or_else(|| bad("unknown conversation type".into()))?;
            let c = super::conversation_class(&n)
                .ok_or_else(|| bad(format!("unknown conversation {n}")))?;
            let mut v = Conversation::new(c);
            if r.read_encoded_int()? == 0 {
                v.has_been_read = r.read_bool()?;
            }
            (c.child_deserialize)(w, &mut v, r)?;
            q.conversations.push(v);
        }
    }
    (class.child_deserialize)(w, &mut q, r)?;
    Ok(q)
}

/// Encodes a quest the way the player's record holds it (for tests and for code that builds one).
pub fn encode(w: &World, q: &Quest) -> Vec<u8> {
    let mut out = SaveWriter::new(true);
    write_quest(w, q, &mut out);
    out.into_vec()
}

/// Decodes the bytes [`encode`] and [`store`] write.
pub fn decode(w: &World, bytes: &[u8], pm: MobileId) -> IoResult<Quest> {
    read_quest(w, &mut SaveReader::new(bytes), pm)
}

// ---- QuestCallbackEntry -----------------------------------------------------------------------------------------------------

/// `new QuestCallbackEntry(number, callback)`: a context menu entry that runs `callback` for the player's quest.
pub fn callback_entry(
    number: i32,
    callback: impl Fn(&mut World, MobileId) + 'static,
) -> muo_server::context_menu_entry::ContextMenuEntry {
    let mut e = muo_server::context_menu_entry::ContextMenuEntry::new(number, -1);
    e.on_click = Some(Rc::new(
        move |w: &mut World, from: MobileId, _t: EntityId| callback(w, from),
    ));
    e
}
