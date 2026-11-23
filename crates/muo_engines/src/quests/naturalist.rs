//! The Study of the Solen Hive quest: the quest, `NestArea`, its objectives and conversations
//! (namespace `Server.Engines.Quests.Naturalist`).

use muo_primitives::{DateTime, Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};

use super::core::{
    ConvClass, Conversation, Msg, ObjClass, Objective, QClass, QUEST_SYSTEM, Quest, St,
    base_accept, base_render_progress, never_logged,
};
use super::gumps::BLUE;
use crate::{conversation, objective};

macro_rules! conv {
    ($n:ident, $s:literal $($t:tt)*) => {
        conversation!($n = concat!("Server.Engines.Quests.Naturalist.", $s) $($t)*);
    };
}
macro_rules! obj {
    ($n:ident, $s:literal $($t:tt)*) => {
        objective!($n = concat!("Server.Engines.Quests.Naturalist.", $s) $($t)*);
    };
}

// ---- NestArea ----

/// `NestArea`: a solen nest, a few rectangles; the special one is the odd egg nest.
pub struct NestArea {
    pub special: bool,
    rects: &'static [Rectangle2D],
}

static AREAS: [NestArea; 5] = [
    NestArea {
        special: false,
        rects: &[Rectangle2D::new(5861, 1787, 26, 25)],
    },
    NestArea {
        special: false,
        rects: &[
            Rectangle2D::new(5734, 1788, 14, 50),
            Rectangle2D::new(5748, 1800, 3, 34),
            Rectangle2D::new(5751, 1808, 2, 20),
        ],
    },
    NestArea {
        special: false,
        rects: &[Rectangle2D::new(5907, 1908, 19, 43)],
    },
    NestArea {
        special: false,
        rects: &[
            Rectangle2D::new(5721, 1926, 24, 29),
            Rectangle2D::new(5745, 1935, 7, 22),
        ],
    },
    NestArea {
        special: true,
        rects: &[
            Rectangle2D::new(5651, 1853, 21, 32),
            Rectangle2D::new(5672, 1857, 6, 20),
        ],
    },
];

impl NestArea {
    /// `NonSpecialCount`.
    pub fn non_special_count() -> i32 {
        AREAS.iter().filter(|a| !a.special).count() as i32
    }

    /// `Find(p)`: the area's `ID`.
    pub fn find(p: Point3D) -> Option<usize> {
        AREAS.iter().position(|a| a.contains(p))
    }

    /// `GetByID(id)`.
    pub fn get_by_id(id: i32) -> Option<&'static NestArea> {
        usize::try_from(id).ok().and_then(|i| AREAS.get(i))
    }

    /// `Contains(p)`.
    pub fn contains(&self, p: Point3D) -> bool {
        self.rects.iter().any(|r| r.contains_xy(p.x, p.y))
    }
}

// ---- Conversations ----

conv!(DONT_OFFER_CONVERSATION, "DontOfferConversation", msg 1054052, logged: never_logged);
conv!(ACCEPT_CONVERSATION, "AcceptConversation", msg 1054043,
    on_read: |w, q, _| q.add_objective(w, study_nests()));
conv!(NATURALIST_DURING_STUDY_CONVERSATION, "NaturalistDuringStudyConversation", msg 1054049, logged: never_logged);
conv!(END_CONVERSATION, "EndConversation", msg 1054050, on_read: |w, q, _| q.complete(w));
conv!(SPECIAL_END_CONVERSATION, "SpecialEndConversation", msg 1054051, on_read: |w, q, _| q.complete(w));
conv!(FULL_BACKPACK_CONVERSATION, "FullBackpackConversation", msg 1054053, logged: never_logged);

pub static CONVERSATIONS: &[&ConvClass] = &[
    &DONT_OFFER_CONVERSATION,
    &ACCEPT_CONVERSATION,
    &NATURALIST_DURING_STUDY_CONVERSATION,
    &END_CONVERSATION,
    &SPECIAL_END_CONVERSATION,
    &FULL_BACKPACK_CONVERSATION,
];

// ---- Objectives ----

#[derive(Copy, Clone, Eq, PartialEq)]
enum StudyState {
    Inactive,
    FirstStep,
    SecondStep,
}

/// `StudyNestsObjective`'s fields; only the nests studied and `StudiedSpecialNest` are saved.
#[derive(Clone)]
pub struct StudyNests {
    studied: Vec<i32>,
    current: Option<usize>,
    begin: DateTime,
    state: StudyState,
    pub studied_special_nest: bool,
}

fn new_study() -> StudyNests {
    StudyNests {
        studied: Vec::new(),
        current: None,
        begin: DateTime::MIN,
        state: StudyState::Inactive,
        studied_special_nest: false,
    }
}

/// `new StudyNestsObjective()`.
pub fn study_nests() -> Objective {
    Objective::with(&STUDY_NESTS_OBJECTIVE, new_study())
}

/// `StudyNestsObjective.StudiedSpecialNest`.
pub fn studied_special_nest(o: &Objective) -> bool {
    o.state::<StudyNests>().studied_special_nest
}

fn on_trammel_or_felucca(w: &World, m: MobileId) -> bool {
    let map = muo_content::mx::map(w, m);
    map == Some(MapId::TRAMMEL) || map == Some(MapId::FELUCCA)
}

obj!(STUDY_NESTS_OBJECTIVE, "StudyNestsObjective", msg 1054044,
new_state: || St::new(new_study()),
max_progress: |_, _| NestArea::non_special_count(),
get_timer_event: |_, _| true,
check_progress: |w, q, i| {
    let from = q.from;
    let loc = muo_content::mx::location(w, from);
    let current = q.objectives[i].state::<StudyNests>().current;
    if let Some(n) = current {
        let nest = &AREAS[n];
        if on_trammel_or_felucca(w, from) && nest.contains(loc) {
            let (state, begin) = {
                let s = q.objectives[i].state::<StudyNests>();
                (s.state, s.begin)
            };
            if state != StudyState::Inactive {
                let time = w.now - begin;
                if time > TimeSpan::from_secs(30) {
                    let s = q.objectives[i].state_mut::<StudyNests>();
                    s.studied.push(n as i32);
                    s.state = StudyState::Inactive;
                    if nest.special {
                        // You complete your examination of this bizarre Egg Nest. The Naturalist will
                        // undoubtedly be quite interested in these notes!
                        s.studied_special_nest = true;
                        muo_content::mx::msg(w, from, 1054057);
                    } else {
                        // You have completed your study of this Solen Egg Nest. You put your notes away.
                        muo_content::mx::msg(w, from, 1054054);
                        q.add_progress(w, i);
                    }
                } else if state == StudyState::FirstStep && time > TimeSpan::from_secs(15) {
                    if !nest.special {
                        // You begin recording your completed notes on a bit of parchment.
                        muo_content::mx::msg(w, from, 1054058);
                    }
                    q.objectives[i].state_mut::<StudyNests>().state = StudyState::SecondStep;
                }
            }
        } else {
            if q.objectives[i].state::<StudyNests>().state != StudyState::Inactive {
                // You abandon your study of the Solen Egg Nest without gathering the needed information.
                muo_content::mx::msg(w, from, 1054046);
            }
            q.objectives[i].state_mut::<StudyNests>().current = None;
        }
    } else if on_trammel_or_felucca(w, from)
        && let Some(n) = NestArea::find(loc) {
            let nest = &AREAS[n];
            let now = w.now;
            let already = q.objectives[i].state::<StudyNests>().studied.contains(&(n as i32));
            {
                let s = q.objectives[i].state_mut::<StudyNests>();
                s.current = Some(n);
                s.begin = now;
                s.state = if already { StudyState::Inactive } else { StudyState::FirstStep };
            }
            if already {
                // You glance at the Egg Nest, realizing you've already studied this one.
                muo_content::mx::msg(w, from, 1054047);
            } else {
                if nest.special {
                    // You notice something very odd about this Solen Egg Nest. You begin taking notes.
                    muo_content::mx::msg(w, from, 105405);
                } else {
                    // You begin studying the Solen Egg Nest to gather information.
                    muo_content::mx::msg(w, from, 1054045);
                }
                let female = w.mobile(from).is_some_and(|m| m.female);
                muo_content::mx::play_sound(w, from, if female { 0x30B } else { 0x419 });
            }
        }
},
render_progress: |q, o, b| {
    if !o.completed(q) {
        b.add_html_localized_color(70, 260, 270, 100, 1054055, BLUE, false, false); // Solen Nests Studied :
        b.add_label(70, 280, 0x64, &o.cur_progress.to_string());
        b.add_label(100, 280, 0x64, "/");
        b.add_label(130, 280, 0x64, &o.max_progress(q).to_string());
    } else {
        base_render_progress(q, o, b);
    }
},
on_complete: |w, q, _| q.add_objective(w, Objective::new(&RETURN_TO_NATURALIST_OBJECTIVE)),
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    let count = r.read_encoded_int()?;
    let s = o.state_mut::<StudyNests>();
    for _ in 0..count.max(0) {
        s.studied.push(r.read_encoded_int()?);
    }
    s.studied_special_nest = r.read_bool()?;
    Ok(())
},
child_serialize: |_, o, out| {
    let s = o.state::<StudyNests>();
    out.write_encoded_int(0); // version
    out.write_encoded_int(s.studied.len() as i32);
    for id in &s.studied {
        out.write_encoded_int(*id);
    }
    out.write_bool(s.studied_special_nest);
});

obj!(RETURN_TO_NATURALIST_OBJECTIVE, "ReturnToNaturalistObjective", msg 1054048,
render_progress: |_, _, b| {
    let n = NestArea::non_special_count().to_string();
    b.add_html_localized_color(70, 260, 270, 100, 1054055, BLUE, false, false); // Solen Nests Studied :
    b.add_label(70, 280, 0x64, &n);
    b.add_label(100, 280, 0x64, "/");
    b.add_label(130, 280, 0x64, &n);
});

pub static OBJECTIVES: &[&ObjClass] = &[&STUDY_NESTS_OBJECTIVE, &RETURN_TO_NATURALIST_OBJECTIVE];

// ---- The quest ----

/// `StudyOfSolenQuest.Naturalist`.
pub fn naturalist(q: &Quest) -> Option<MobileId> {
    *q.state::<Option<MobileId>>()
}

/// `new StudyOfSolenQuest(from, naturalist)`.
pub fn new_quest(w: &World, from: MobileId, naturalist: MobileId) -> Quest {
    Quest::with(w, &QUEST, from, Some(naturalist))
}

/// `StudyOfSolenQuest`: the state is the `Naturalist`.
pub static QUEST: QClass = QClass {
    name: "Server.Engines.Quests.Naturalist.StudyOfSolenQuest",
    slot: 5,
    name_msg: Msg::Num(1054041),
    offer_message: |_, _| Msg::Num(1054042),
    picture: |_, _| 0x15C7,
    is_tutorial: false,
    restart_delay: TimeSpan::ZERO,
    new_state: || St::new(None::<MobileId>),
    accept: |w, q: &mut Quest| {
        base_accept(w, q);
        if let Some(n) = naturalist(q) {
            muo_content::mx::play_sound(w, n, 0x431);
        }
        q.add_conversation(w, Conversation::new(&ACCEPT_CONVERSATION));
    },
    child_deserialize: |w, q, r| {
        r.read_encoded_int()?; // version
        *q.state_mut::<Option<MobileId>>() = r.read_mobile(w)?;
        Ok(())
    },
    child_serialize: |w, q, out| {
        out.write_encoded_int(0); // version
        out.write_mobile(w, naturalist(q));
    },
    ..QUEST_SYSTEM
};

// ---- Mobiles (Naturalist) ----

use muo_server::mobile::MobileType;

use super::core::{can_offer_quest, has_quest, load, with_quest};
use super::quester::{BASE_QUESTER, QUESTER_C, QUESTER_OPS, QUESTER_VC, QuesterOps};

/// `PlantHue.Pink`, `Magenta`, `Aqua`, `FireRed`.
const PLANT_PINK: i32 = 0x40;
const PLANT_MAGENTA: i32 = 0x80;
const PLANT_AQUA: i32 = 0x100;
const PLANT_FIRE_RED: i32 = 0x200;

fn naturalist_on_talk(
    w: &mut World,
    naturalist_id: MobileId,
    player: MobileId,
    _context_menu: bool,
) {
    let ours = load(w, player).filter(|q| q.is(&QUEST) && naturalist(q) == Some(naturalist_id));
    let Some(qs) = ours else {
        let mut new_quest = new_quest(w, player, naturalist_id);
        if !has_quest(w, player) && can_offer_quest(w, player, &QUEST) {
            muo_content::mx::play_sound(w, naturalist_id, 0x42F);
            new_quest.send_offer(w);
        } else {
            muo_content::mx::play_sound(w, naturalist_id, 0x448);
            new_quest.add_conversation(w, Conversation::new(&DONT_OFFER_CONVERSATION));
        }
        return;
    };

    let Some(study) = qs.find_objective(&STUDY_NESTS_OBJECTIVE) else {
        return;
    };
    if !qs.objectives[study].completed(&qs) {
        muo_content::mx::play_sound(w, naturalist_id, 0x41F);
        with_quest(w, player, |w, q| {
            q.add_conversation(w, Conversation::new(&NATURALIST_DURING_STUDY_CONVERSATION))
        });
        return;
    }

    let Some(obj) = qs
        .find_objective(&RETURN_TO_NATURALIST_OBJECTIVE)
        .filter(|&i| !qs.objectives[i].completed(&qs))
    else {
        return;
    };

    // Utility.Random(17) switch { 0 => CampionFlowers, ... _ => TribarrelCactus }: the enum order
    let plant_type = muo_content::hooks::rnd::below(w, 17);
    let special = studied_special_nest(&qs.objectives[study]);
    let hue = if special {
        PLANT_FIRE_RED
    } else {
        match muo_content::hooks::rnd::below(w, 3) {
            0 => PLANT_PINK,
            1 => PLANT_MAGENTA,
            _ => PLANT_AQUA,
        }
    };
    let reward = super::hooks::quest_hooks(w)
        .and_then(|h| h.make_seed)
        .and_then(|f| f(w, plant_type, hue));

    if reward.is_some_and(|r| muo_server::mobile::part2::items::place_in_backpack(w, player, r)) {
        with_quest(w, player, |w, q| {
            q.complete_objective(w, obj);
            muo_content::mx::play_sound(w, naturalist_id, 0x449);
            muo_content::mx::play_sound(w, naturalist_id, 0x41B);
            if special {
                q.add_conversation(w, Conversation::new(&SPECIAL_END_CONVERSATION));
            } else {
                q.add_conversation(w, Conversation::new(&END_CONVERSATION));
            }
        });
    } else {
        if let Some(r) = reward {
            muo_server::item::base::delete(w, r);
        }
        with_quest(w, player, |w, q| {
            q.add_conversation(w, Conversation::new(&FULL_BACKPACK_CONVERSATION))
        });
    }
}

static NATURALIST_OPS: QuesterOps = QuesterOps {
    on_talk: naturalist_on_talk,
    ..QUESTER_OPS
};

crate::quester! {
    /// `Server.Engines.Quests.Naturalist.Naturalist`.
    NATURALIST_MOBILE, NATURALIST_C, NATURALIST_V = "Server.Engines.Quests.Naturalist.Naturalist"
        : BASE_QUESTER / QUESTER_C / QUESTER_VC,
        ops NATURALIST_OPS, encoded true, title Some("the Naturalist"), extra |_, _| {};
    v {
        init_body: |w, id| muo_content::templates::apply_mobile_as(w, id, "Server.Engines.Quests.Naturalist.Naturalist"),
        init_outfit: |_, _| {},
    }
    c { }
    m { }
}

/// The mobile classes of this folder.
pub static MOBILES: &[&MobileType] = &[&NATURALIST_MOBILE];
