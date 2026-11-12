//! The Witch Apprentice quest: the quest, its objectives, conversations and `Ingredient`
//! (namespace `Server.Engines.Quests.Hag`).

use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};

use super::core::{
    ConvClass, Conversation, Msg, ObjClass, Objective, QClass, QUEST_SYSTEM, Quest, St,
    base_accept, base_render_progress, never_logged, with_quest_of,
};
use super::gumps::BLUE;
use super::util::mobile_is;
use crate::{conversation, objective};

macro_rules! conv {
    ($n:ident, $s:literal $($t:tt)*) => {
        conversation!($n = concat!("Server.Engines.Quests.Hag.", $s) $($t)*);
    };
}
macro_rules! obj {
    ($n:ident, $s:literal $($t:tt)*) => {
        objective!($n = concat!("Server.Engines.Quests.Hag.", $s) $($t)*);
    };
}

// ---- Ingredient ----

/// `Ingredient.Whiskey`, the last one.
pub const WHISKEY: i32 = 14;

/// `IngredientInfo`: the cliloc name, how many to bring, and the creatures that drop it (exact classes).
pub struct IngredientInfo {
    pub name: i32,
    pub quantity: i32,
    pub creatures: &'static [&'static str],
}

const fn ing(name: i32, quantity: i32, creatures: &'static [&'static str]) -> IngredientInfo {
    IngredientInfo {
        name,
        quantity,
        creatures,
    }
}

/// `IngredientInfo.m_Table`, indexed by `Ingredient`.
pub static INGREDIENTS: [IngredientInfo; 15] = [
    ing(1055020, 5, &["Server.Mobiles.Sheep"]),
    ing(
        1055021,
        5,
        &["Server.Mobiles.Rabbit", "Server.Mobiles.JackRabbit"],
    ),
    ing(
        1055022,
        5,
        &["Server.Mobiles.Mongbat", "Server.Mobiles.GreaterMongbat"],
    ),
    ing(1055023, 5, &["Server.Mobiles.Chicken"]),
    ing(
        1055024,
        5,
        &[
            "Server.Mobiles.Rat",
            "Server.Mobiles.GiantRat",
            "Server.Mobiles.SewerRat",
        ],
    ),
    ing(1055025, 5, &["Server.Mobiles.BullFrog"]),
    ing(
        1055026,
        5,
        &["Server.Mobiles.Hind", "Server.Mobiles.GreatHart"],
    ),
    ing(
        1055027,
        5,
        &["Server.Mobiles.LavaLizard", "Server.Mobiles.Lizardman"],
    ),
    ing(1055028, 5, &["Server.Mobiles.Slime"]),
    ing(
        1055029,
        5,
        &[
            "Server.Mobiles.Ghoul",
            "Server.Mobiles.Spectre",
            "Server.Mobiles.Shade",
            "Server.Mobiles.Wraith",
            "Server.Mobiles.Bogle",
        ],
    ),
    ing(1055030, 1, &[]),
    ing(1055031, 1, &[]),
    ing(1055032, 1, &[]),
    ing(1055033, 1, &[]),
    ing(1055034, 1, &[]),
];

/// `IngredientInfo.Get(ingredient)`.
pub fn ingredient_info(ingredient: i32) -> &'static IngredientInfo {
    &INGREDIENTS[usize::try_from(ingredient)
        .ok()
        .filter(|i| *i < INGREDIENTS.len())
        .unwrap_or(0)]
}

/// `IngredientInfo.RandomIngredient(oldIngredients)`.
pub fn random_ingredient(w: &mut World, old: &[i32]) -> i32 {
    let left: Vec<i32> = (0..INGREDIENTS.len() as i32)
        .filter(|i| !old.contains(i))
        .collect();
    left[muo_content::hooks::rnd::below(w, left.len() as i32) as usize]
}

// ---- The quest ----

static ZEEFZORPUL_LOCATIONS: [Point3D; 20] = [
    Point3D::new(1226, 1573, 0),
    Point3D::new(1929, 1148, 0),
    Point3D::new(1366, 2723, 0),
    Point3D::new(1675, 2984, 0),
    Point3D::new(2177, 3367, 10),
    Point3D::new(1171, 3594, 0),
    Point3D::new(1010, 2667, 5),
    Point3D::new(1591, 2156, 5),
    Point3D::new(2592, 464, 60),
    Point3D::new(474, 1654, 0),
    Point3D::new(897, 2411, 0),
    Point3D::new(1471, 2505, 5),
    Point3D::new(1257, 872, 16),
    Point3D::new(2581, 1118, 0),
    Point3D::new(2513, 1102, 0),
    Point3D::new(1608, 3371, 0),
    Point3D::new(4687, 1179, 0),
    Point3D::new(3704, 2196, 20),
    Point3D::new(3346, 572, 0),
    Point3D::new(569, 1309, 0),
];

/// `WitchApprenticeQuest.RandomZeefzorpulLocation()`.
pub fn random_zeefzorpul_location(w: &mut World) -> Point3D {
    ZEEFZORPUL_LOCATIONS
        [muo_content::hooks::rnd::below(w, ZEEFZORPUL_LOCATIONS.len() as i32) as usize]
}

/// `WitchApprenticeQuest`.
pub static QUEST: QClass = QClass {
    name: "Server.Engines.Quests.Hag.WitchApprenticeQuest",
    slot: 4,
    name_msg: Msg::Num(1055042),
    offer_message: |_, _| Msg::Num(1055001),
    picture: |_, _| 0x15D3,
    is_tutorial: false,
    restart_delay: TimeSpan::from_mins(5),
    accept: |w, q: &mut Quest| {
        base_accept(w, q);
        q.add_conversation(w, Conversation::new(&ACCEPT_CONVERSATION));
    },
    ..QUEST_SYSTEM
};

// ---- Conversations ----

conv!(DONT_OFFER_CONVERSATION, "DontOfferConversation", msg 1055000, logged: never_logged);
conv!(ACCEPT_CONVERSATION, "AcceptConversation", msg 1055002,
on_read: |w, q, _| {
    let o = find_apprentice(w, true);
    q.add_objective(w, o);
});
conv!(HAG_DURING_CORPSE_SEARCH_CONVERSATION, "HagDuringCorpseSearchConversation", msg 1055003, logged: never_logged);
conv!(APPRENTICE_CORPSE_CONVERSATION, "ApprenticeCorpseConversation", msg 1055004,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&FIND_GRIZELDA_ABOUT_MURDER_OBJECTIVE)));
conv!(MURDER_CONVERSATION, "MurderConversation", msg 1055005,
on_read: |w, q, _| {
    let o = kill_imps(w, true);
    q.add_objective(w, o);
});
conv!(HAG_DURING_IMP_SEARCH_CONVERSATION, "HagDuringImpSearchConversation", msg 1055006, logged: never_logged);

/// `new ImpDeathConversation(impLocation)`.
pub fn imp_death(imp_location: Point3D) -> Conversation {
    Conversation::with(&IMP_DEATH_CONVERSATION, imp_location)
}

conv!(IMP_DEATH_CONVERSATION, "ImpDeathConversation", msg 1055007,
new_state: || St::new(Point3D::ZERO),
on_read: |w, q, c| q.add_objective(w, find_zeefzorpul(*c.state::<Point3D>())),
child_deserialize: |_, c, r| {
    r.read_encoded_int()?; // version
    *c.st.get_mut::<Point3D>() = r.read_point3d()?;
    Ok(())
},
child_serialize: |_, c, out| {
    out.write_encoded_int(0); // version
    out.write_point3d(*c.state::<Point3D>());
});

conv!(ZEEFZORPUL_CONVERSATION, "ZeefzorpulConversation", msg 1055008,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&RETURN_RECIPE_OBJECTIVE)));
conv!(RECIPE_CONVERSATION, "RecipeConversation", msg 1055009,
on_read: |w, q, _| {
    let o = find_ingredient(w, &[], false);
    q.add_objective(w, o);
});
conv!(HAG_DURING_INGREDIENTS_CONVERSATION, "HagDuringIngredientsConversation", msg 1055012, logged: never_logged);
conv!(BLACKHEART_FIRST_CONVERSATION, "BlackheartFirstConversation", msg 1055010,
on_read: |w, q, _| {
    if let Some(i) = q.find_objective(&FIND_INGREDIENT_OBJECTIVE) {
        let old = ingredients(&q.objectives[i]).to_vec();
        let o = find_ingredient(w, &old, true);
        q.add_objective(w, o);
    }
});

/// `BlackheartNoPirateConversation`'s `m_Tricorne` and `m_Drunken`.
#[derive(Clone, Default)]
struct NoPirate {
    tricorne: bool,
    drunken: bool,
}

/// `new BlackheartNoPirateConversation(tricorne, drunken)`.
pub fn blackheart_no_pirate(tricorne: bool, drunken: bool) -> Conversation {
    Conversation::with(
        &BLACKHEART_NO_PIRATE_CONVERSATION,
        NoPirate { tricorne, drunken },
    )
}

conv!(BLACKHEART_NO_PIRATE_CONVERSATION, "BlackheartNoPirateConversation",
new_state: || St::new(NoPirate::default()),
message: |_, c| {
    let s = c.state::<NoPirate>();
    match (s.tricorne, s.drunken) {
        (true, true) => 1055059,
        (true, false) => 1055057,
        (false, true) => 1055056,
        (false, false) => 1055058,
    }
},
logged: never_logged,
child_deserialize: |_, c, r| {
    r.read_encoded_int()?; // version
    let s = c.st.get_mut::<NoPirate>();
    s.tricorne = r.read_bool()?;
    s.drunken = r.read_bool()?;
    Ok(())
},
child_serialize: |_, c, out| {
    let s = c.state::<NoPirate>();
    out.write_encoded_int(0); // version
    out.write_bool(s.tricorne);
    out.write_bool(s.drunken);
});

/// `new BlackheartPirateConversation(firstMet)`.
pub fn blackheart_pirate(first_met: bool) -> Conversation {
    Conversation::with(&BLACKHEART_PIRATE_CONVERSATION, first_met)
}

conv!(BLACKHEART_PIRATE_CONVERSATION, "BlackheartPirateConversation",
new_state: || St::new(false),
message: |_, c| if *c.state::<bool>() { 1055054 } else { 1055011 },
on_read: |w, q, _| {
    if let Some(i) = q.find_objective(&FIND_INGREDIENT_OBJECTIVE) {
        find_ingredient_next_step(w, q, i);
    }
},
child_deserialize: |_, c, r| {
    r.read_encoded_int()?; // version
    *c.st.get_mut::<bool>() = r.read_bool()?;
    Ok(())
},
child_serialize: |_, c, out| {
    out.write_encoded_int(0); // version
    out.write_bool(*c.state::<bool>());
});

conv!(END_CONVERSATION, "EndConversation", msg 1055013, on_read: |w, q, _| q.complete(w));
conv!(RECENTLY_FINISHED_CONVERSATION, "RecentlyFinishedConversation", msg 1055064, logged: never_logged);

pub static CONVERSATIONS: &[&ConvClass] = &[
    &DONT_OFFER_CONVERSATION,
    &ACCEPT_CONVERSATION,
    &HAG_DURING_CORPSE_SEARCH_CONVERSATION,
    &APPRENTICE_CORPSE_CONVERSATION,
    &MURDER_CONVERSATION,
    &HAG_DURING_IMP_SEARCH_CONVERSATION,
    &IMP_DEATH_CONVERSATION,
    &ZEEFZORPUL_CONVERSATION,
    &RECIPE_CONVERSATION,
    &HAG_DURING_INGREDIENTS_CONVERSATION,
    &BLACKHEART_FIRST_CONVERSATION,
    &BLACKHEART_NO_PIRATE_CONVERSATION,
    &BLACKHEART_PIRATE_CONVERSATION,
    &END_CONVERSATION,
    &RECENTLY_FINISHED_CONVERSATION,
];

// ---- Objectives ----

static CORPSE_LOCATIONS: [Point3D; 5] = [
    Point3D::new(778, 1158, 0),
    Point3D::new(698, 1443, 0),
    Point3D::new(785, 1548, 0),
    Point3D::new(734, 1504, 0),
    Point3D::new(819, 1266, 0),
];

/// `FindApprenticeObjective`'s `m_CorpseLocation` and `Corpse`.
#[derive(Clone)]
struct Apprentice {
    location: Point3D,
    corpse: Option<ItemId>,
}

/// `new FindApprenticeObjective(init)`.
pub fn find_apprentice(w: &mut World, init: bool) -> Objective {
    let mut st = Apprentice {
        location: Point3D::ZERO,
        corpse: None,
    };
    if init {
        st.location = CORPSE_LOCATIONS[muo_content::hooks::rnd::below(w, 5) as usize];
    }
    Objective::with(&FIND_APPRENTICE_OBJECTIVE, st)
}

/// `FindApprenticeObjective.Corpse`.
pub fn apprentice_corpse(o: &Objective) -> Option<ItemId> {
    o.state::<Apprentice>().corpse
}

/// `Effects.SendLocationEffect(loc, map, 0x3728, 10); Effects.PlaySound(loc, map, 0x1FE)`.
fn poof(w: &mut World, loc: Point3D, map: Option<MapId>) {
    muo_server::effects::send_location_effect(w, loc, map, 0x3728, 10, 10, 0, 0);
    muo_server::effects::play_sound(w, loc, map, 0x1FE);
}

/// `DeleteImp(m)`: the imp vanishes in a puff.
fn delete_imp(w: &mut World, imp: MobileId) {
    if muo_content::mx::deleted(w, imp) {
        return;
    }
    let (loc, map) = (
        muo_content::mx::location(w, imp),
        muo_content::mx::map(w, imp),
    );
    poof(w, loc, map);
    muo_server::mobile::base::delete(w, imp);
}

obj!(FIND_APPRENTICE_OBJECTIVE, "FindApprenticeObjective", msg 1055014,
new_state: || St::new(Apprentice { location: Point3D::ZERO, corpse: None }),
check_progress: |w, q, i| {
    let player = q.from;
    let map = muo_content::mx::map(w, player);
    let (loc, corpse) = {
        let s = q.objectives[i].state::<Apprentice>();
        (s.location, s.corpse)
    };
    let corpse_alive = corpse.is_some_and(|c| w.item(c).is_some_and(|it| !it.deleted()));
    if corpse_alive
        || (map != Some(MapId::TRAMMEL) && map != Some(MapId::FELUCCA))
        || !muo_content::mx::in_range_point(w, player, loc, 8)
    {
        return;
    }
    let c = muo_content::factory::make_item(w, "Server.Engines.Quests.Hag.HagApprenticeCorpse");
    q.objectives[i].state_mut::<Apprentice>().corpse = c;
    if let Some(c) = c {
        muo_server::item::ops::move_to_world_map(w, c, loc, map);
    }
    poof(w, loc, map);
    let imp = muo_content::factory::make_mobile(w, "Server.Engines.Quests.Hag.Zeefzorpul");
    if let Some(imp) = imp {
        muo_server::mobile::base::move_to_world(w, imp, loc, map.unwrap_or(MapId::INTERNAL));
    }
    if let Some(c) = c {
        muo_server::item::messages::send_localized_message_to(w, c, player, 1055049, "");
    }
    if let Some(imp) = imp {
        muo_server::timer::start(w, TimeSpan::from_secs(3), TimeSpan::ZERO, 1, move |w| delete_imp(w, imp));
    }
},
on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&APPRENTICE_CORPSE_CONVERSATION)),
child_deserialize: |w, o, r| {
    let version = r.read_encoded_int()?;
    let s = o.state_mut::<Apprentice>();
    if version >= 1 {
        s.location = r.read_point3d()?;
    }
    if version >= 0 {
        s.corpse = r.read_item(w)?;
    }
    if version == 0 {
        // m_CorpseLocation = RandomCorpseLocation(): the first location stands in for the random one.
        // shortcut: a version 0 record would pick a random location on load; the first is used here
        // (the choice is not saved either way), upgrade when the deserializer can reach the generator.
        s.location = CORPSE_LOCATIONS[0];
    }
    Ok(())
},
child_serialize: |w, o, out| {
    let s = o.state::<Apprentice>();
    let corpse = s.corpse.filter(|c| w.item(*c).is_some_and(|it| !it.deleted()));
    out.write_encoded_int(1); // version
    out.write_point3d(s.location);
    out.write_item(w, corpse);
});

obj!(FIND_GRIZELDA_ABOUT_MURDER_OBJECTIVE, "FindGrizeldaAboutMurderObjective", msg 1055015,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&MURDER_CONVERSATION)));

/// `new KillImpsObjective(init)`: `m_MaxProgress`, saved as a plain int.
pub fn kill_imps(w: &mut World, init: bool) -> Objective {
    let max = if init {
        muo_content::hooks::rnd::min_max(w, 1, 4)
    } else {
        0
    };
    Objective::with(&KILL_IMPS_OBJECTIVE, max)
}

obj!(KILL_IMPS_OBJECTIVE, "KillImpsObjective", msg 1055016,
new_state: || St::new(0_i32),
max_progress: |_, o| *o.state::<i32>(),
ignore_young_protection: |w, q, i, from| {
    !q.objectives[i].completed(q) && mobile_is(w, from, "Server.Mobiles.Imp")
},
on_kill: |w, q, i, creature, _| {
    if mobile_is(w, creature, "Server.Mobiles.Imp") {
        q.add_progress(w, i);
    }
},
on_complete: |w, q, _| {
    let from = q.from;
    let loc = random_zeefzorpul_location(w);
    let map = muo_items::maps::map_item_new(w, &muo_items::maps::MAP_ITEM, None);
    muo_items::maps::set_display(w, map, loc.x - 200, loc.y - 200, loc.x + 200, loc.y + 200, 200, 200);
    muo_items::maps::add_world_pin(w, map, loc.x, loc.y);
    muo_server::mobile::part2::items::add_to_backpack(w, from, map);
    if let Some(flute) = muo_content::factory::make_item(w, "Server.Engines.Quests.Hag.MagicFlute") {
        muo_server::mobile::part2::items::add_to_backpack(w, from, flute);
    }
    muo_content::mx::msg(w, from, 1055061); // You have received a map and a magic flute.
    q.add_conversation(w, imp_death(loc));
},
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    *o.state_mut::<i32>() = r.read_i32()?;
    Ok(())
},
child_serialize: |_, o, out| {
    out.write_encoded_int(0); // version
    out.write_i32(*o.state::<i32>());
});

/// `new FindZeefzorpulObjective(impLocation)`.
pub fn find_zeefzorpul(imp_location: Point3D) -> Objective {
    Objective::with(&FIND_ZEEFZORPUL_OBJECTIVE, imp_location)
}

obj!(FIND_ZEEFZORPUL_OBJECTIVE, "FindZeefzorpulObjective", msg 1055017,
new_state: || St::new(Point3D::ZERO),
on_complete: |w, q, i| {
    let from = q.from;
    let loc = *q.objectives[i].state::<Point3D>();
    let map = muo_content::mx::map(w, from);
    poof(w, loc, map);
    let Some(imp) = muo_content::factory::make_mobile(w, "Server.Engines.Quests.Hag.Zeefzorpul") else { return };
    muo_server::mobile::base::move_to_world(w, imp, loc, map.unwrap_or(MapId::INTERNAL));
    let d = muo_content::mx::get_direction_to(w, imp, from);
    muo_content::mx::set_direction(w, imp, d);
    muo_server::timer::start(w, TimeSpan::from_secs(3), TimeSpan::ZERO, 1, move |w| {
        delete_imp(w, imp);
        muo_content::mx::msg(w, from, 1055062); // You have received the Magic Brew Recipe.
        with_quest_of(w, from, &QUEST, |w, q| q.add_conversation(w, Conversation::new(&ZEEFZORPUL_CONVERSATION)));
    });
},
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    *o.state_mut::<Point3D>() = r.read_point3d()?;
    Ok(())
},
child_serialize: |_, o, out| {
    out.write_encoded_int(0); // version
    out.write_point3d(*o.state::<Point3D>());
});

obj!(RETURN_RECIPE_OBJECTIVE, "ReturnRecipeObjective", msg 1055018,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&RECIPE_CONVERSATION)));

/// `FindIngredientObjective`'s `Ingredients` and `BlackheartMet`.
#[derive(Clone, Default)]
struct Ingredients {
    list: Vec<i32>,
    blackheart_met: bool,
}

/// `FindIngredientObjective.Ingredients`.
pub fn ingredients(o: &Objective) -> &[i32] {
    &o.state::<Ingredients>().list
}

/// `new FindIngredientObjective(oldIngredients, blackheartMet)`.
pub fn find_ingredient(w: &mut World, old: &[i32], blackheart_met: bool) -> Objective {
    let mut list = old.to_vec();
    if !blackheart_met {
        let next = random_ingredient(w, old);
        list.push(next);
    }
    Objective::with(
        &FIND_INGREDIENT_OBJECTIVE,
        Ingredients {
            list,
            blackheart_met,
        },
    )
}

/// `FindIngredientObjective.Ingredient`.
pub fn current_ingredient(o: &Objective) -> i32 {
    o.state::<Ingredients>().list.last().copied().unwrap_or(0)
}

/// `FindIngredientObjective.BlackheartMet`.
pub fn blackheart_met(o: &Objective) -> bool {
    o.state::<Ingredients>().blackheart_met
}

/// `FindIngredientObjective.Step`.
pub fn step(o: &Objective) -> usize {
    o.state::<Ingredients>().list.len()
}

/// `FindIngredientObjective.NextStep()`.
pub fn find_ingredient_next_step(w: &mut World, q: &mut Quest, i: usize) {
    // You have completed your current task on the Hag's Magic Brew Recipe list.
    muo_content::mx::msg(w, q.from, 1055046);
    if step(&q.objectives[i]) < 3 {
        let old = ingredients(&q.objectives[i]).to_vec();
        let o = find_ingredient(w, &old, false);
        q.add_objective(w, o);
    } else {
        q.add_objective(w, Objective::new(&RETURN_INGREDIENTS_OBJECTIVE));
    }
}

obj!(FIND_INGREDIENT_OBJECTIVE, "FindIngredientObjective",
new_state: || St::new(Ingredients::default()),
message: |_, o| {
    if !blackheart_met(o) {
        match step(o) {
            1 => 1055019,
            2 => 1055044,
            _ => 1055045,
        }
    } else {
        1055055
    }
},
max_progress: |_, o| ingredient_info(current_ingredient(o)).quantity,
render_progress: |q, o, b| {
    if !o.completed(q) {
        let info = ingredient_info(current_ingredient(o));
        b.add_html_localized_color(70, 260, 270, 100, info.name, BLUE, false, false);
        b.add_label(70, 280, 0x64, &o.cur_progress.to_string());
        b.add_label(100, 280, 0x64, "/");
        b.add_label(130, 280, 0x64, &info.quantity.to_string());
    } else {
        base_render_progress(q, o, b);
    }
},
ignore_young_protection: |w, q, i, from| {
    if q.objectives[i].completed(q) {
        return false;
    }
    let info = ingredient_info(current_ingredient(&q.objectives[i]));
    let ty = w.mobile(from).map(|m| m.ty.full_name);
    ty.is_some_and(|t| info.creatures.contains(&t))
},
on_kill: |w, q, i, creature, _| {
    let info = ingredient_info(current_ingredient(&q.objectives[i]));
    let ty = w.mobile(creature).map(|m| m.ty.full_name);
    if ty.is_some_and(|t| info.creatures.contains(&t)) {
        muo_content::mx::msg_args(w, q.from, 1055043, &format!("#{}", info.name));
        q.add_progress(w, i);
    }
},
on_complete: |w, q, i| {
    if current_ingredient(&q.objectives[i]) != WHISKEY {
        find_ingredient_next_step(w, q, i);
    }
},
child_deserialize: |_, o, r| {
    r.read_encoded_int()?; // version
    let n = r.read_encoded_int()?.max(0);
    let s = o.state_mut::<Ingredients>();
    for _ in 0..n {
        s.list.push(r.read_encoded_int()?);
    }
    s.blackheart_met = r.read_bool()?;
    Ok(())
},
child_serialize: |_, o, out| {
    let s = o.state::<Ingredients>();
    out.write_encoded_int(0); // version
    out.write_encoded_int(s.list.len() as i32);
    for i in &s.list {
        out.write_encoded_int(*i);
    }
    out.write_bool(s.blackheart_met);
});

obj!(RETURN_INGREDIENTS_OBJECTIVE, "ReturnIngredientsObjective", msg 1055050,
    on_complete: |w, q, _| q.add_conversation(w, Conversation::new(&END_CONVERSATION)));

pub static OBJECTIVES: &[&ObjClass] = &[
    &FIND_APPRENTICE_OBJECTIVE,
    &FIND_GRIZELDA_ABOUT_MURDER_OBJECTIVE,
    &KILL_IMPS_OBJECTIVE,
    &FIND_ZEEFZORPUL_OBJECTIVE,
    &RETURN_RECIPE_OBJECTIVE,
    &FIND_INGREDIENT_OBJECTIVE,
    &RETURN_INGREDIENTS_OBJECTIVE,
];

/// `Ingredient.RedMushrooms`.
const RED_MUSHROOMS: i32 = 11;

/// `BladedItemTarget`'s red mushroom branch: slicing one completes the `FindIngredientObjective` that wants it.
/// True when it did (the click is consumed).
pub fn slice_red_mushroom(w: &mut World, from: MobileId) -> bool {
    with_quest_of(w, from, &QUEST, |w, q| {
        let Some(i) = q.find_objective(&FIND_INGREDIENT_OBJECTIVE) else {
            return false;
        };
        if q.objectives[i].completed(q) || current_ingredient(&q.objectives[i]) != RED_MUSHROOMS {
            return false;
        }
        // You slice a red cap mushroom from its stem.
        muo_server::mobile::part2::messages::send_localized_message(w, from, 1055036, "", 0x3B2);
        q.complete_objective(w, i);
        true
    })
    .unwrap_or(false)
}
