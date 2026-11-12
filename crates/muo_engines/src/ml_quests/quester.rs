//! `IQuestGiver`, the quester names and the `MLQuestSystem.Tell*` helpers.

use muo_io::text::TextDefinition;
use muo_primitives::dotnet::InvariantCase;
use muo_primitives::{Direction, MessageType};
use muo_server::ids::{ItemId, MobileId};
use muo_server::world::World;

pub use super::model::Quester;
use super::model::*;

/// `MLQuestSystem.SpeechColor`.
pub const SPEECH_COLOR: i32 = 0x3B2;

type DynamicQuestsFn = fn(&World, MobileId) -> Option<Vec<QuestId>>;
/// What a dynamic quest list needs from a quester that is not a static one (`BaseEscortable.ConstructQuestList`).
#[derive(Default)]
pub struct QuesterHooks {
    /// `BaseCreature.MLQuests` when `StaticMLQuester` is false: `Some(list)` for such a creature.
    pub dynamic_quests: Option<DynamicQuestsFn>,
}

impl Quester {
    /// `Deleted`.
    pub fn deleted(self, w: &World) -> bool {
        match self {
            Quester::Mobile(m) => w.mobile(m).is_none_or(|m| m.deleted),
            Quester::Item(i) => w.item(i).is_none_or(|i| i.deleted()),
        }
    }

    pub fn serial(self, w: &World) -> Option<muo_primitives::Serial> {
        match self {
            Quester::Mobile(m) => w.mobile(m).map(|m| m.serial),
            Quester::Item(i) => w.item(i).map(|i| i.serial),
        }
    }

    /// `GetType().FullName`.
    pub fn type_name(self, w: &World) -> Option<&'static str> {
        match self {
            Quester::Mobile(m) => w.mobile(m).map(|m| m.ty.full_name),
            Quester::Item(i) => w.item(i).map(|i| i.ty.full_name),
        }
    }

    /// `ReadEntity<IEntity>() as IQuestGiver`.
    pub fn from_entity(e: muo_server::ids::EntityId) -> Option<Quester> {
        match e {
            muo_server::ids::EntityId::Mobile(m) => Some(Quester::Mobile(m)),
            muo_server::ids::EntityId::Item(i) => Some(Quester::Item(i)),
        }
    }
}

/// `QuesterNameAttribute.GetQuesterNameFor(type)`: the attribute, else `Type.Name`.
pub fn quester_name_for(full_name: Option<&str>) -> String {
    let Some(full) = full_name else {
        return String::new();
    };
    let short = short_name(full);
    super::quester_names::QUESTER_NAMES
        .iter()
        .find(|(n, _)| *n == short)
        .map_or_else(|| short.to_owned(), |(_, q)| (*q).to_owned())
}

/// `IQuestGiver.MLQuests`: the creature's or item's list (`FindQuestList(GetType())`).
pub fn quests_of(w: &World, q: Quester) -> Vec<QuestId> {
    if let Quester::Mobile(m) = q
        && let Some(f) = w
            .resources
            .get::<QuesterHooks>()
            .and_then(|h| h.dynamic_quests)
        && let Some(list) = f(w, m)
    {
        return list;
    }
    let Some(name) = q.type_name(w) else {
        return Vec::new();
    };
    find_quest_list(w, name)
}

/// `MLQuestSystem.FindQuestList(questerType)`.
pub fn find_quest_list(w: &World, quester_type: &str) -> Vec<QuestId> {
    let key = short_name(quester_type).lower_inv();
    state_ref(w)
        .and_then(|s| s.quest_givers.get(&key))
        .cloned()
        .unwrap_or_default()
}

/// `TurnToFace`.
pub fn turn_to_face(w: &mut World, q: Quester, mob: MobileId) {
    if let Quester::Mobile(m) = q {
        let Some(t) = w.mobile(mob).map(|t| (t.location.x, t.location.y)) else {
            return;
        };
        let dir: Direction =
            muo_server::mobile::part2::effects::get_direction_to(w, m, t.0, t.1, false);
        muo_server::mobile::part2::effects::set_direction(w, m, dir);
    }
}

fn net_state(w: &World, pm: MobileId) -> Option<muo_server::net_state::NetStateId> {
    w.mobile(pm).and_then(|m| m.net_state)
}

/// `Tell(quester, pm, cliloc)`.
pub fn tell(w: &mut World, q: Quester, pm: MobileId, cliloc: i32) {
    tell_args(w, q, pm, cliloc, "");
}

/// `Tell(quester, pm, cliloc, args)`.
pub fn tell_args(w: &mut World, q: Quester, pm: MobileId, cliloc: i32, args: &str) {
    turn_to_face(w, q, pm);
    match q {
        Quester::Mobile(m) => {
            let ns = net_state(w, pm);
            muo_server::mobile::part2::messages::private_overhead_message_number(
                w,
                m,
                MessageType::REGULAR,
                SPEECH_COLOR,
                cliloc,
                args,
                ns,
            );
        }
        Quester::Item(i) => {
            muo_server::item::messages::send_localized_hue(w, i, pm, cliloc, SPEECH_COLOR, args);
        }
    }
}

/// `Tell(quester, pm, message)`.
pub fn tell_text(w: &mut World, q: Quester, pm: MobileId, message: &str) {
    turn_to_face(w, q, pm);
    match q {
        Quester::Mobile(m) => {
            let ns = net_state(w, pm);
            muo_server::mobile::part2::messages::private_overhead_message(
                w,
                m,
                MessageType::REGULAR,
                SPEECH_COLOR,
                false,
                message,
                ns,
            );
        }
        Quester::Item(i) => {
            muo_server::item::messages::send_message_to(w, i, pm, message, SPEECH_COLOR);
        }
    }
}

/// `TellDef(quester, pm, def)`.
pub fn tell_def(w: &mut World, q: Quester, pm: MobileId, def: &TextDefinition) {
    if def.number > 0 {
        tell(w, q, pm, def.number);
    } else if let Some(s) = &def.string {
        tell_text(w, q, pm, s);
    }
}

/// `Item` as a quester: the item id, for callers that only deal with items.
pub fn as_item(q: Quester) -> Option<ItemId> {
    match q {
        Quester::Item(i) => Some(i),
        Quester::Mobile(_) => None,
    }
}
