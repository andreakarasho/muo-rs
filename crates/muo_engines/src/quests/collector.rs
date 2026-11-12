//! The Collector quest: the quest, its objectives and conversations, and `ImageTypeInfo`
//! (namespace `Server.Engines.Quests.Collector`).

use muo_primitives::random::Rng;
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::map::MapId;

use super::core::{
    ConvClass, Conversation, Msg, ObjClass, Objective, QClass, QUEST_SYSTEM, Quest, St,
    base_accept, base_decline, base_render_progress, never_logged,
};
use crate::{conversation, objective};

macro_rules! conv {
    ($n:ident, $s:literal $($t:tt)*) => {
        conversation!($n = concat!("Server.Engines.Quests.Collector.", $s) $($t)*);
    };
}
macro_rules! obj {
    ($n:ident, $s:literal $($t:tt)*) => {
        objective!($n = concat!("Server.Engines.Quests.Collector.", $s) $($t)*);
    };
}

// ---- ImageTypeInfo ----

/// `ImageTypeInfo`: the figurine, the creature class (by full name), and the offset the gump draws it at.
pub struct ImageTypeInfo {
    pub figurine: i32,
    pub type_name: &'static str,
    pub x: i32,
    pub y: i32,
}

impl ImageTypeInfo {
    /// `Name`.
    pub fn name(&self) -> i32 {
        if self.figurine < 0x4000 {
            1020000 + self.figurine
        } else {
            1078872 + self.figurine
        }
    }
}

const fn img(figurine: i32, type_name: &'static str, x: i32, y: i32) -> ImageTypeInfo {
    ImageTypeInfo {
        figurine,
        type_name,
        x,
        y,
    }
}

/// `ImageTypeInfo._table`, indexed by `ImageType`.
pub static IMAGE_TABLE: [ImageTypeInfo; 21] = [
    img(9734, "Server.Mobiles.Betrayer", 75, 45),
    img(9735, "Server.Mobiles.Bogling", 75, 45),
    img(9736, "Server.Mobiles.BogThing", 60, 47),
    img(9615, "Server.Mobiles.Gazer", 75, 45),
    img(9743, "Server.Mobiles.Beetle", 60, 55),
    img(9667, "Server.Mobiles.GiantBlackWidow", 55, 52),
    img(9657, "Server.Mobiles.Scorpion", 65, 47),
    img(9758, "Server.Mobiles.JukaMage", 75, 45),
    img(9759, "Server.Mobiles.JukaWarrior", 75, 45),
    img(9636, "Server.Mobiles.Lich", 75, 45),
    img(9756, "Server.Mobiles.MeerMage", 75, 45),
    img(9757, "Server.Mobiles.MeerWarrior", 75, 45),
    img(9638, "Server.Mobiles.Mongbat", 70, 50),
    img(9639, "Server.Mobiles.Mummy", 75, 45),
    img(9654, "Server.Mobiles.Pixie", 75, 45),
    img(9747, "Server.Mobiles.PlagueBeast", 60, 45),
    img(9750, "Server.Mobiles.SandVortex", 60, 43),
    img(9614, "Server.Mobiles.StoneGargoyle", 75, 45),
    img(9753, "Server.Mobiles.SwampDragon", 50, 55),
    img(8448, "Server.Mobiles.Wisp", 75, 45),
    img(9746, "Server.Mobiles.Juggernaut", 55, 38),
];

/// `ImageTypeInfo.Get(image)`: an out of range value is the first entry.
pub fn image_info(image: i32) -> &'static ImageTypeInfo {
    &IMAGE_TABLE[usize::try_from(image)
        .ok()
        .filter(|i| *i < IMAGE_TABLE.len())
        .unwrap_or(0)]
}

/// `ImageTypeInfo._shuffleTable`: the table's order, shuffled in place by every `RandomList`.
struct ShuffleTable(Vec<i32>);

impl Default for ShuffleTable {
    fn default() -> Self {
        ShuffleTable((0..IMAGE_TABLE.len() as i32).collect())
    }
}

/// `ImageTypeInfo.RandomList(count)`.
pub fn random_images(w: &mut World, count: usize) -> Vec<i32> {
    if count == 0 {
        return Vec::new();
    }
    let mut table = std::mem::take(&mut w.resources.get_mut::<ShuffleTable>().0);
    w.rng.shuffle(&mut table);
    let mut out = vec![0; count];
    for (o, t) in out.iter_mut().zip(&table) {
        *o = *t;
    }
    w.resources.get_mut::<ShuffleTable>().0 = table;
    out
}

// ---- Conversations ----

conv!(DONT_OFFER_CONVERSATION, "DontOfferConversation", msg 1055080, logged: never_logged);
conv!(DECLINE_CONVERSATION, "DeclineConversation", msg 1055082, logged: never_logged);
conv!(ACCEPT_CONVERSATION, "AcceptConversation", msg 1055083,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FISH_PEARLS_OBJECTIVE)));
conv!(ELWOOD_DURING_FISH_CONVERSATION, "ElwoodDuringFishConversation", msg 1055089, logged: never_logged);
conv!(RETURN_PEARLS_CONVERSATION, "ReturnPearlsConversation", msg 1055090,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_ALBERTA_OBJECTIVE)));
conv!(ALBERTA_PAINTING_CONVERSATION, "AlbertaPaintingConversation", msg 1055092,
    on_read: |w, q, _| q.add_objective(w, sit_on_the_stool()));
conv!(ALBERTA_STOOL_CONVERSATION, "AlbertaStoolConversation", msg 1055096, logged: never_logged);
conv!(ALBERTA_END_PAINTING_CONVERSATION, "AlbertaEndPaintingConversation", msg 1055098,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&RETURN_PAINTING_OBJECTIVE)));
conv!(ALBERTA_AFTER_PAINTING_CONVERSATION, "AlbertaAfterPaintingConversation", msg 1055102, logged: never_logged);
conv!(ELWOOD_DURING_PAINTING1_CONVERSATION, "ElwoodDuringPainting1Conversation", msg 1055094, logged: never_logged);
conv!(ELWOOD_DURING_PAINTING2_CONVERSATION, "ElwoodDuringPainting2Conversation", msg 1055097, logged: never_logged);
conv!(RETURN_PAINTING_CONVERSATION, "ReturnPaintingConversation", msg 1055100,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_GABRIEL_OBJECTIVE)));
conv!(GABRIEL_AUTOGRAPH_CONVERSATION, "GabrielAutographConversation", msg 1055103,
on_read: |w, q, _| {
    let o = find_sheet_music(w, true);
    q.add_objective(w, o);
});
conv!(GABRIEL_NO_SHEET_MUSIC_CONVERSATION, "GabrielNoSheetMusicConversation", msg 1055111, logged: never_logged);
conv!(NO_SHEET_MUSIC_CONVERSATION, "NoSheetMusicConversation", msg 1055106, logged: never_logged);
conv!(GET_SHEET_MUSIC_CONVERSATION, "GetSheetMusicConversation", msg 1055109,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&RETURN_SHEET_MUSIC_OBJECTIVE)));
conv!(GABRIEL_SHEET_MUSIC_CONVERSATION, "GabrielSheetMusicConversation", msg 1055113,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&RETURN_AUTOGRAPH_OBJECTIVE)));
conv!(GABRIEL_IGNORE_CONVERSATION, "GabrielIgnoreConversation", msg 1055118, logged: never_logged);
conv!(ELWOOD_DURING_AUTOGRAPH1_CONVERSATION, "ElwoodDuringAutograph1Conversation", msg 1055105, logged: never_logged);
conv!(ELWOOD_DURING_AUTOGRAPH2_CONVERSATION, "ElwoodDuringAutograph2Conversation", msg 1055112, logged: never_logged);
conv!(ELWOOD_DURING_AUTOGRAPH3_CONVERSATION, "ElwoodDuringAutograph3Conversation", msg 1055115, logged: never_logged);
conv!(RETURN_AUTOGRAPH_CONVERSATION, "ReturnAutographConversation", msg 1055116,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_TOMAS_OBJECTIVE)));
conv!(TOMAS_TOYS_CONVERSATION, "TomasToysConversation", msg 1055119,
on_read: |w, q, _| {
    let o = capture_images(w, true);
    q.add_objective(w, o);
});
conv!(TOMAS_DURING_COLLECTING_CONVERSATION, "TomasDuringCollectingConversation", msg 1055129, logged: never_logged);
conv!(RETURN_IMAGES_CONVERSATION, "ReturnImagesConversation", msg 1055131,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&RETURN_TOYS_OBJECTIVE)));
conv!(ELWOOD_DURING_TOYS1_CONVERSATION, "ElwoodDuringToys1Conversation", msg 1055123, logged: never_logged);
conv!(ELWOOD_DURING_TOYS2_CONVERSATION, "ElwoodDuringToys2Conversation", msg 1055130, logged: never_logged);
conv!(ELWOOD_DURING_TOYS3_CONVERSATION, "ElwoodDuringToys3Conversation", msg 1055133, logged: never_logged);
conv!(END_CONVERSATION, "EndConversation", msg 1055134, on_read: |w, q, _| q.complete(w));
conv!(FULL_END_CONVERSATION, "FullEndConversation", msg 1055135,
new_state: || St::new(true),
logged: |_, c| *c.state::<bool>(),
on_read: |w, q, c| {
    if *c.state::<bool>() {
        q.add_objective(w, Objective::new(&MAKE_ROOM_OBJECTIVE));
    }
});

/// `new FullEndConversation(logged)`.
pub fn full_end_conversation(logged: bool) -> Conversation {
    Conversation::with(&FULL_END_CONVERSATION, logged)
}

pub static CONVERSATIONS: &[&ConvClass] = &[
    &DONT_OFFER_CONVERSATION,
    &DECLINE_CONVERSATION,
    &ACCEPT_CONVERSATION,
    &ELWOOD_DURING_FISH_CONVERSATION,
    &RETURN_PEARLS_CONVERSATION,
    &ALBERTA_PAINTING_CONVERSATION,
    &ALBERTA_STOOL_CONVERSATION,
    &ALBERTA_END_PAINTING_CONVERSATION,
    &ALBERTA_AFTER_PAINTING_CONVERSATION,
    &ELWOOD_DURING_PAINTING1_CONVERSATION,
    &ELWOOD_DURING_PAINTING2_CONVERSATION,
    &RETURN_PAINTING_CONVERSATION,
    &GABRIEL_AUTOGRAPH_CONVERSATION,
    &GABRIEL_NO_SHEET_MUSIC_CONVERSATION,
    &NO_SHEET_MUSIC_CONVERSATION,
    &GET_SHEET_MUSIC_CONVERSATION,
    &GABRIEL_SHEET_MUSIC_CONVERSATION,
    &GABRIEL_IGNORE_CONVERSATION,
    &ELWOOD_DURING_AUTOGRAPH1_CONVERSATION,
    &ELWOOD_DURING_AUTOGRAPH2_CONVERSATION,
    &ELWOOD_DURING_AUTOGRAPH3_CONVERSATION,
    &RETURN_AUTOGRAPH_CONVERSATION,
    &TOMAS_TOYS_CONVERSATION,
    &TOMAS_DURING_COLLECTING_CONVERSATION,
    &RETURN_IMAGES_CONVERSATION,
    &ELWOOD_DURING_TOYS1_CONVERSATION,
    &ELWOOD_DURING_TOYS2_CONVERSATION,
    &ELWOOD_DURING_TOYS3_CONVERSATION,
    &END_CONVERSATION,
    &FULL_END_CONVERSATION,
];

// ---- Objectives ----

obj!(FISH_PEARLS_OBJECTIVE, "FishPearlsObjective", msg 1055084,
    max_progress: |_, _| 6,
    render_progress: |q, o, b| {
        if !o.completed(q) {
            super::gumps::add_html_object(b, 70, 260, 270, 100, Msg::Num(1055085), super::gumps::BLUE, false, false);
            b.add_label(70, 280, 0x64, &o.cur_progress.to_string());
            b.add_label(100, 280, 0x64, "/");
            b.add_label(130, 280, 0x64, &o.max_progress(q).to_string());
        } else {
            base_render_progress(q, o, b);
        }
    },
    on_complete: |w, q, _| q.add_objective(w, Objective::new(&RETURN_PEARLS_OBJECTIVE)));

obj!(RETURN_PEARLS_OBJECTIVE, "ReturnPearlsObjective", msg 1055088,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&RETURN_PEARLS_CONVERSATION)));

obj!(FIND_ALBERTA_OBJECTIVE, "FindAlbertaObjective", msg 1055091,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&ALBERTA_PAINTING_CONVERSATION)));

const STOOL_LOCATION: Point3D = Point3D::new(2899, 706, 0);

/// `SitOnTheStoolObjective`: when the player sat down (`DateTime.MaxValue` is not sitting). Not saved.
#[derive(Clone)]
struct StoolState {
    begin: DateTime,
}

/// `new SitOnTheStoolObjective()`.
pub fn sit_on_the_stool() -> Objective {
    Objective::with(
        &SIT_ON_THE_STOOL_OBJECTIVE,
        StoolState {
            begin: DateTime::MAX,
        },
    )
}

obj!(SIT_ON_THE_STOOL_OBJECTIVE, "SitOnTheStoolObjective", msg 1055093,
    new_state: || St::new(StoolState { begin: DateTime::MAX }),
    check_progress: |w, q, i| {
        let pm = q.from;
        let (map, loc) = (muo_content::mx::map(w, pm), muo_content::mx::location(w, pm));
        let begin = q.objectives[i].state::<StoolState>().begin;
        if map == Some(MapId::TRAMMEL) && loc == STOOL_LOCATION {
            if begin == DateTime::MAX {
                q.objectives[i].state_mut::<StoolState>().begin = w.now;
            } else if w.now - begin > TimeSpan::from_secs(30) {
                q.complete_objective(w, i);
            }
        } else if begin != DateTime::MAX {
            q.objectives[i].state_mut::<StoolState>().begin = DateTime::MAX;
            // You must remain seated on the stool until the portrait is complete. Alberta will now have to start
            // again with a fresh canvas.
            super::util::msg_hue(w, pm, 1055095, 0x26);
        }
    },
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&ALBERTA_END_PAINTING_CONVERSATION)));

obj!(RETURN_PAINTING_OBJECTIVE, "ReturnPaintingObjective", msg 1055099,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&RETURN_PAINTING_CONVERSATION)));

obj!(FIND_GABRIEL_OBJECTIVE, "FindGabrielObjective", msg 1055101,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&GABRIEL_AUTOGRAPH_CONVERSATION)));

/// `Theater`.
pub const THEATER_BRITAIN: i32 = 0;
pub const THEATER_NUJELM: i32 = 1;
pub const THEATER_JHELOM: i32 = 2;

/// `new FindSheetMusicObjective(init)`: `InitTheater()` picks one at random.
pub fn find_sheet_music(w: &mut World, init: bool) -> Objective {
    let mut o = Objective::with(&FIND_SHEET_MUSIC_OBJECTIVE, 0_i32);
    if init {
        // Utility.Random(3) switch { 1 => Britain, 2 => Nujelm, _ => Jhelom }
        *o.state_mut::<i32>() = match muo_content::hooks::rnd::below(w, 3) {
            1 => THEATER_BRITAIN,
            2 => THEATER_NUJELM,
            _ => THEATER_JHELOM,
        };
    }
    o
}

/// `FindSheetMusicObjective.IsInRightTheater()`.
pub fn is_in_right_theater(w: &World, q: &Quest, i: usize) -> bool {
    let (loc, map) = (
        muo_content::mx::location(w, q.from),
        muo_content::mx::map(w, q.from),
    );
    let region = muo_server::mobile::part2::deps::region_find(w, loc, map);
    let name = match *q.objectives[i].state::<i32>() {
        THEATER_BRITAIN => "Britain",
        THEATER_NUJELM => "Nujel'm",
        THEATER_JHELOM => "Jhelom",
        _ => return false,
    };
    muo_server::region::is_part_of_name(w, region, name, false)
}

obj!(FIND_SHEET_MUSIC_OBJECTIVE, "FindSheetMusicObjective", msg 1055104,
new_state: || St::new(0_i32),
on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&GET_SHEET_MUSIC_CONVERSATION)),
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    *o.state_mut::<i32>() = r.read_encoded_int()?;
    Ok(())
},
child_serialize: |_, o, out| {
    out.write_encoded_int(0); // version
    out.write_encoded_int(*o.state::<i32>());
});

obj!(RETURN_SHEET_MUSIC_OBJECTIVE, "ReturnSheetMusicObjective", msg 1055110,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&GABRIEL_SHEET_MUSIC_CONVERSATION)));

obj!(RETURN_AUTOGRAPH_OBJECTIVE, "ReturnAutographObjective", msg 1055114,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&RETURN_AUTOGRAPH_CONVERSATION)));

obj!(FIND_TOMAS_OBJECTIVE, "FindTomasObjective", msg 1055117,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&TOMAS_TOYS_CONVERSATION)));

/// `CaptureResponse`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CaptureResponse {
    Valid,
    AlreadyDone,
    Invalid,
}

/// `CaptureImagesObjective`'s `m_Images` and `m_Done`.
#[derive(Clone, Default)]
struct CaptureState {
    images: Vec<i32>,
    done: Vec<bool>,
}

/// `new CaptureImagesObjective(init)`.
pub fn capture_images(w: &mut World, init: bool) -> Objective {
    let mut st = CaptureState::default();
    if init {
        st.images = random_images(w, 4);
        st.done = vec![false; 4];
    }
    Objective::with(&CAPTURE_IMAGES_OBJECTIVE, st)
}

/// `CaptureImagesObjective.CaptureImage(type, out image)`: `type_name` is the creature's full class name.
pub fn capture_image(
    w: &mut World,
    q: &mut Quest,
    i: usize,
    type_name: &str,
) -> (CaptureResponse, i32) {
    let n = q.objectives[i].state::<CaptureState>().images.len();
    for k in 0..n {
        let image = q.objectives[i].state::<CaptureState>().images[k];
        if image_info(image).type_name == type_name {
            if q.objectives[i].state::<CaptureState>().done[k] {
                return (CaptureResponse::AlreadyDone, image);
            }
            q.objectives[i].state_mut::<CaptureState>().done[k] = true;
            q.check_completion_status(w, i);
            return (CaptureResponse::Valid, image);
        }
    }
    (CaptureResponse::Invalid, 0)
}

obj!(CAPTURE_IMAGES_OBJECTIVE, "CaptureImagesObjective", msg 1055120,
new_state: || St::new(CaptureState::default()),
completed: |_, o| o.state::<CaptureState>().done.iter().all(|d| *d),
ignore_young_protection: |w, q, i, from| {
    let o = &q.objectives[i];
    if o.completed(q) {
        return false;
    }
    let Some(m) = w.mobile(from) else { return false };
    let ty = m.ty.full_name;
    o.state::<CaptureState>().images.iter().any(|img| image_info(*img).type_name == ty)
},
render_progress: |q, o, b| {
    if !o.completed(q) {
        let st = o.state::<CaptureState>();
        for (i, image) in st.images.iter().enumerate() {
            let y = 260 + 20 * i as i32;
            let info = image_info(*image);
            super::gumps::add_html_object(b, 70, y, 200, 100, Msg::Num(info.name()), super::gumps::BLUE, false, false);
            b.add_label(200, y, 0x64, " : ");
            let n = if st.done[i] { 1055121 } else { 1055122 };
            super::gumps::add_html_object(b, 220, y, 100, 100, Msg::Num(n), super::gumps::BLUE, false, false);
        }
    } else {
        base_render_progress(q, o, b);
    }
},
on_complete: |w, q, _| q.add_objective(w, Objective::new(&RETURN_IMAGES_OBJECTIVE)),
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    let count = r.read_encoded_int()?.max(0) as usize;
    let st = o.state_mut::<CaptureState>();
    st.images = Vec::with_capacity(count.min(64));
    st.done = Vec::with_capacity(count.min(64));
    for _ in 0..count {
        st.images.push(r.read_encoded_int()?);
        st.done.push(r.read_bool()?);
    }
    Ok(())
},
child_serialize: |_, o, out| {
    let st = o.state::<CaptureState>();
    out.write_encoded_int(0); // version
    out.write_encoded_int(st.images.len() as i32);
    for (image, done) in st.images.iter().zip(&st.done) {
        out.write_encoded_int(*image);
        out.write_bool(*done);
    }
});

obj!(RETURN_IMAGES_OBJECTIVE, "ReturnImagesObjective", msg 1055128,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&RETURN_IMAGES_CONVERSATION)));

obj!(RETURN_TOYS_OBJECTIVE, "ReturnToysObjective", msg 1055132);
obj!(MAKE_ROOM_OBJECTIVE, "MakeRoomObjective", msg 1055136);

pub static OBJECTIVES: &[&ObjClass] = &[
    &FISH_PEARLS_OBJECTIVE,
    &RETURN_PEARLS_OBJECTIVE,
    &FIND_ALBERTA_OBJECTIVE,
    &SIT_ON_THE_STOOL_OBJECTIVE,
    &RETURN_PAINTING_OBJECTIVE,
    &FIND_GABRIEL_OBJECTIVE,
    &FIND_SHEET_MUSIC_OBJECTIVE,
    &RETURN_SHEET_MUSIC_OBJECTIVE,
    &RETURN_AUTOGRAPH_OBJECTIVE,
    &FIND_TOMAS_OBJECTIVE,
    &CAPTURE_IMAGES_OBJECTIVE,
    &RETURN_IMAGES_OBJECTIVE,
    &RETURN_TOYS_OBJECTIVE,
    &MAKE_ROOM_OBJECTIVE,
];

// ---- The quest ----

/// `CollectorQuest`.
pub static QUEST: QClass = QClass {
    name: "Server.Engines.Quests.Collector.CollectorQuest",
    slot: 3,
    name_msg: Msg::Text("Collector's Quest"),
    offer_message: |_, _| Msg::Num(1055081),
    picture: |_, _| 0x15A9,
    is_tutorial: false,
    restart_delay: TimeSpan::ZERO,
    accept: |w, q: &mut Quest| {
        base_accept(w, q);
        q.add_conversation(w, Conversation::new(&ACCEPT_CONVERSATION));
    },
    decline: |w, q: &mut Quest| {
        base_decline(w, q);
        q.add_conversation(w, Conversation::new(&DECLINE_CONVERSATION));
    },
    ..QUEST_SYSTEM
};
