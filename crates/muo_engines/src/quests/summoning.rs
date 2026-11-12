//! The Summoning quest: the quest, its objectives and conversations
//! (namespace `Server.Engines.Quests.Doom`).

use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};

use super::core::{
    ConvClass, Conversation, Msg, ObjClass, Objective, QClass, QUEST_SYSTEM, Quest, St,
    base_accept, base_render_message, base_render_progress,
};
use super::gumps::{BLUE, add_html_object};
use crate::{conversation, objective};

macro_rules! conv {
    ($n:ident, $s:literal $($t:tt)*) => {
        conversation!($n = concat!("Server.Engines.Quests.Doom.", $s) $($t)*);
    };
}
macro_rules! obj {
    ($n:ident, $s:literal $($t:tt)*) => {
        objective!($n = concat!("Server.Engines.Quests.Doom.", $s) $($t)*);
    };
}

const ALTAR_RANGE: i32 = 24;

// ---- Victoria and the altar ----

/// `Victoria`'s `_altar`, in the mobile's `Ext`.
#[derive(Clone, Default)]
pub struct VictoriaData {
    pub altar: Option<ItemId>,
}

/// `SummoningAltar`'s `Daemon`, in the item's `Ext`.
#[derive(Clone, Default)]
pub struct AltarData {
    pub daemon: Option<MobileId>,
}

/// `Victoria.Altar`: the cached altar, looked for again (the last one in range wins) when it is gone, on another
/// map or too far.
pub fn victoria_altar(w: &mut World, victoria: MobileId) -> Option<ItemId> {
    let cached = w
        .mobile(victoria)?
        .ext
        .get::<VictoriaData>()
        .and_then(|d| d.altar);
    let (loc, map) = (
        muo_content::mx::location(w, victoria),
        muo_content::mx::map(w, victoria),
    );
    let valid = cached.is_some_and(|a| {
        w.item(a).is_some_and(|it| {
            !it.deleted()
                && it.map == map
                && (it.location.x - loc.x).abs() <= ALTAR_RANGE
                && (it.location.y - loc.y).abs() <= ALTAR_RANGE
        })
    });
    if !valid {
        let mut found = cached;
        for it in
            muo_server::mobile::part2::enumerators::get_items_in_range(w, victoria, ALTAR_RANGE)
        {
            if super::util::item_is(w, it, "Server.Engines.Quests.Doom.SummoningAltar") {
                found = Some(it);
            }
        }
        if let Some(m) = w.mobile_mut(victoria) {
            m.ext.insert(VictoriaData { altar: found });
        }
        return found;
    }
    cached
}

/// `altar.Daemon`.
pub fn altar_daemon(w: &World, altar: ItemId) -> Option<MobileId> {
    w.item(altar)?.ext.get::<AltarData>()?.daemon
}

/// `altar.Daemon?.Alive == true`.
fn daemon_alive(w: &World, altar: ItemId) -> bool {
    altar_daemon(w, altar).is_some_and(|d| muo_content::mx::alive(w, d))
}

/// `SummoningAltar.CheckDaemon()`.
pub fn check_daemon(w: &mut World, altar: ItemId) {
    let alive = daemon_alive(w, altar);
    if !alive && let Some(d) = w.item_mut(altar).and_then(|i| i.ext.get_mut::<AltarData>()) {
        d.daemon = None;
    }
    let hue = if alive { 0x66D } else { 0 };
    if let Some(f) = w.item(altar).map(|i| i.ty.set_hue) {
        f(w, altar, hue);
    }
}

// ---- The quest ----

/// `TheSummoningQuest`'s `Victoria` and `WaitForSummon`.
#[derive(Clone, Default)]
pub struct Summoning {
    pub victoria: Option<MobileId>,
    pub wait_for_summon: bool,
}

/// `new TheSummoningQuest(victoria, from)`.
pub fn new_quest(w: &World, victoria: MobileId, from: MobileId) -> Quest {
    Quest::with(
        w,
        &QUEST,
        from,
        Summoning {
            victoria: Some(victoria),
            wait_for_summon: false,
        },
    )
}

fn state(q: &Quest) -> &Summoning {
    q.state::<Summoning>()
}

/// `TheSummoningQuest.GetDaemonBonesFor(creature)`.
pub fn get_daemon_bones_for(w: &mut World, creature: MobileId) -> i32 {
    let controlled = muo_content::creature::read(w, creature, true, |d| d.controlled);
    let summoned = muo_content::creature::read(w, creature, false, |d| d.summoned);
    if controlled || summoned {
        return 0;
    }
    let fame = w.mobile(creature).map_or(0, |m| m.fame);
    if fame < 1500 {
        return utility::dice(&mut w.rng, 2, 5, -1);
    }
    if fame < 20000 {
        return utility::dice(&mut w.rng, 2, 4, 8);
    }
    50
}

/// The `Victoria == null` / `Altar == null` messages of the summoning steps.
fn unable(w: &mut World, q: &mut Quest, what: &str) {
    muo_content::mx::send_message(
        w,
        q.from,
        &format!("Internal error: unable to find {what}. Quest unable to continue."),
    );
    let f = q.class.cancel;
    f(w, q);
}

/// `TheSummoningQuest`.
pub static QUEST: QClass = QClass {
    name: "Server.Engines.Quests.Doom.TheSummoningQuest",
    slot: 0,
    name_msg: Msg::Num(1050025),
    offer_message: |_, _| Msg::Num(1050020),
    picture: |_, _| 0x15B5,
    is_tutorial: false,
    restart_delay: TimeSpan::ZERO,
    new_state: || St::new(Summoning::default()),
    slice: |w, q| {
        if state(q).wait_for_summon
            && let Some(victoria) = state(q).victoria
            && let Some(altar) = victoria_altar(w, victoria)
            && !daemon_alive(w, altar)
            && muo_content::mx::map(w, q.from) == muo_content::mx::map(w, victoria)
            && muo_content::mx::in_range(w, q.from, victoria, 8)
        {
            q.state_mut::<Summoning>().wait_for_summon = false;
            q.add_conversation(w, Conversation::new(&VANQUISH_DAEMON_CONVERSATION));
        }
        q.slice(w);
    },
    cancel: |w, q| {
        q.clear_quest(w, false);
        if let Some(i) = q.find_objective(&COLLECT_BONES_OBJECTIVE)
            && q.objectives[i].cur_progress > 0
        {
            let n = q.objectives[i].cur_progress;
            if let Some(bank) = muo_server::mobile::part1::props_b::bank_box(w, q.from)
                && let Some(bone) = muo_content::factory::make_item(w, "Server.Items.DaemonBone")
            {
                muo_server::item::ops::set_amount(w, bone, n);
                muo_server::item::container::drop_item_default(w, bank, bone);
            }
            muo_content::mx::msg(w, q.from, 1050030);
        }
    },
    accept: |w, q: &mut Quest| {
        base_accept(w, q);
        q.add_conversation(w, Conversation::new(&ACCEPT_CONVERSATION));
    },
    child_deserialize: |w, q, r| {
        r.read_encoded_int()?; // version
        let s = q.state_mut::<Summoning>();
        s.victoria = r.read_mobile(w)?;
        s.wait_for_summon = r.read_bool()?;
        Ok(())
    },
    child_serialize: |w, q, out| {
        out.write_encoded_int(0); // version
        out.write_mobile(w, state(q).victoria);
        out.write_bool(state(q).wait_for_summon);
    },
    ..QUEST_SYSTEM
};

// ---- Conversations ----

conv!(ACCEPT_CONVERSATION, "AcceptConversation", msg 1050027,
    on_read: |w, q, _| q.add_objective(w, Objective::new(&COLLECT_BONES_OBJECTIVE)));

conv!(VANQUISH_DAEMON_CONVERSATION, "VanquishDaemonConversation", msg 1050021,
on_read: |w, q, _| {
    let Some(victoria) = state(q).victoria else {
        unable(w, q, "Victoria");
        return;
    };
    let Some(altar) = victoria_altar(w, victoria) else {
        unable(w, q, "summoning altar");
        return;
    };
    if !daemon_alive(w, altar) {
        let Some(daemon) = muo_content::factory::make_mobile(w, "Server.Mobiles.BoneDemon") else { return };
        let (loc, map) = {
            let it = w.item(altar);
            (it.map_or_else(Default::default, |i| i.location), it.and_then(|i| i.map))
        };
        muo_server::mobile::base::move_to_world(w, daemon, loc, map.unwrap_or(muo_server::map::MapId::INTERNAL));
        if let Some(a) = w.item_mut(altar) {
            a.ext.insert(AltarData { daemon: Some(daemon) });
        }
        check_daemon(w, altar); // SummoningAltar.OnDaemonChanged
        q.add_objective(w, vanquish_daemon(daemon));
    } else {
        muo_content::mx::say_to(w, victoria, q.from, "The devourer has already been summoned.");
        q.state_mut::<Summoning>().wait_for_summon = true;
    }
});

pub static CONVERSATIONS: &[&ConvClass] = &[&ACCEPT_CONVERSATION, &VANQUISH_DAEMON_CONVERSATION];

// ---- Objectives ----

obj!(COLLECT_BONES_OBJECTIVE, "CollectBonesObjective", msg 1050026,
max_progress: |_, _| 1000,
render_message: |q, o, b| {
    if o.cur_progress > 0 && o.cur_progress < o.max_progress(q) {
        add_html_object(b, 70, 130, 300, 100, Msg::Num(1050028), BLUE, false, false);
    } else {
        base_render_message(q, o, b);
    }
},
render_progress: |q, o, b| {
    if o.cur_progress > 0 && o.cur_progress < o.max_progress(q) {
        add_html_object(b, 70, 260, 270, 100, Msg::Num(1050019), BLUE, false, false);
        b.add_label(70, 280, 100, &o.cur_progress.to_string());
        b.add_label(100, 280, 100, "/");
        b.add_label(130, 280, 100, &o.max_progress(q).to_string());
    } else {
        base_render_progress(q, o, b);
    }
},
on_complete: |w, q, _| {
    let Some(victoria) = state(q).victoria else {
        unable(w, q, "Victoria");
        return;
    };
    let Some(altar) = victoria_altar(w, victoria) else {
        unable(w, q, "summoning altar");
        return;
    };
    if !daemon_alive(w, altar) {
        q.add_conversation(w, Conversation::new(&VANQUISH_DAEMON_CONVERSATION));
    } else {
        muo_content::mx::say_to(
            w,
            victoria,
            q.from,
            "The devourer has already been summoned. Return when the devourer has been slain and I will summon it for you.",
        );
        q.state_mut::<Summoning>().wait_for_summon = true;
    }
});

/// `VanquishDaemonObjective`'s `m_Daemon` and `CorpseWithSkull`.
#[derive(Clone, Default)]
pub struct Vanquish {
    pub daemon: Option<MobileId>,
    pub corpse_with_skull: Option<ItemId>,
}

/// `new VanquishDaemonObjective(daemon)`.
pub fn vanquish_daemon(daemon: MobileId) -> Objective {
    Objective::with(
        &VANQUISH_DAEMON_OBJECTIVE,
        Vanquish {
            daemon: Some(daemon),
            corpse_with_skull: None,
        },
    )
}

obj!(VANQUISH_DAEMON_OBJECTIVE, "VanquishDaemonObjective", msg 1050037,
new_state: || St::new(Vanquish::default()),
check_progress: |w, q, i| {
    let alive = q.objectives[i].state::<Vanquish>().daemon.is_some_and(|d| muo_content::mx::alive(w, d));
    if !alive {
        q.complete_objective(w, i);
    }
},
on_complete: |w, q, i| {
    let from = q.from;
    let daemon = q.objectives[i].state::<Vanquish>().daemon;
    let altar = state(q).victoria.and_then(|v| victoria_altar(w, v));
    if let Some(a) = altar {
        check_daemon(w, a);
    }
    if !muo_content::mx::alive(w, from) {
        muo_content::mx::msg(w, from, 1050033);
        q.state_mut::<Summoning>().wait_for_summon = true;
        return;
    }
    let mut has_rights = false;
    if let Some(d) = daemon {
        let hits_max = muo_content::mx::hits_max(w, d);
        let rights = muo_content::creature::death::get_looting_rights(w, d, hits_max);
        has_rights = rights.iter().any(|ds| ds.has_right && ds.mobile == from);
    }
    if !has_rights {
        muo_content::mx::msg(w, from, 1050034);
        q.state_mut::<Summoning>().wait_for_summon = true;
    } else {
        muo_content::mx::msg(w, from, 1050035); // The devourer lies dead.  Search his corpse to claim your prize!
        if let Some(d) = daemon {
            let corpse = w.mobile(d).and_then(|m| m.corpse);
            q.objectives[i].state_mut::<Vanquish>().corpse_with_skull = corpse;
        }
    }
},
child_deserialize: |w, o, r| {
    r.read_encoded_int()?; // version
    let s = o.state_mut::<Vanquish>();
    s.daemon = r.read_mobile(w)?;
    s.corpse_with_skull = r.read_item(w)?;
    Ok(())
},
child_serialize: |w, o, out| {
    let s = o.state::<Vanquish>();
    out.write_encoded_int(0); // version
    out.write_mobile(w, s.daemon);
    out.write_item(w, s.corpse_with_skull);
});

pub static OBJECTIVES: &[&ObjClass] = &[&COLLECT_BONES_OBJECTIVE, &VANQUISH_DAEMON_OBJECTIVE];
