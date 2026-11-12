//! Race changing: `IRaceChanger`, `RaceChangeConfirmGump`, `RaceChangeDeed`, the race changer packets and the boon
//! collectors (`DoneQuestCollector`, `Darius`, `Nedrick`).

use std::collections::HashMap;

use muo_content::gumps::has_gump;
use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, RelayInfo, send_gump_to_mobile,
};
use muo_io::SpanReader;
use muo_io::text::TextDefinition;
use muo_net::net_state::NetState;
use muo_primitives::{MessageType, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::layer::Layer;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::messages as mm;
use muo_server::net_state::NetStateId;
use muo_server::race::{Race, RaceId, Races};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::engine;
use super::npc_support::*;

// ---- MLQuestPackets ------------------------------------------------------------------------------------------

/// `SendRaceChanger(female, newRace)`.
pub fn send_race_changer(w: &mut World, ns: NetStateId, female: bool, race_id: i32) {
    w.send(
        ns,
        &[
            0xBF,
            0x00,
            0x07,
            0x00,
            0x2A,
            u8::from(female),
            (race_id + 1) as u8,
        ],
    );
}

/// `SendCloseRaceChanger()`.
pub fn send_close_race_changer(w: &mut World, ns: NetStateId) {
    w.send(ns, &[0xBF, 0x00, 0x07, 0x00, 0x2A, 0x00, 0xFF]);
}

// ---- IRaceChanger ----------------------------------------------------------------------------------------

/// The `IRaceChanger`s: the deed and the two collectors.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RaceChanger {
    Deed(ItemId),
    Collector(MobileId),
}

fn check_complete(w: &mut World, owner: RaceChanger, pm: MobileId) -> bool {
    match owner {
        RaceChanger::Deed(id) => {
            if w.item(id).is_none_or(|i| i.deleted()) {
                return false;
            }
            let pack = props_b::backpack(w, pm).map(muo_server::ids::EntityId::Item);
            if !ops::is_child_of(w, id, pack) {
                mm::send_localized_message(w, pm, 1042001, "", 0x3B2); // That must be in your pack for you to use it.
                return false;
            }
            true
        }
        RaceChanger::Collector(id) => collector_check_complete(w, id, pm),
    }
}

fn consume_needed(w: &mut World, owner: RaceChanger, pm: MobileId) {
    match owner {
        RaceChanger::Deed(id) => {
            if let Some(ty) = w.item(id).map(|i| i.ty) {
                (ty.consume)(w, id, 1);
            }
        }
        RaceChanger::Collector(id) => collector_consume_needed(w, id, pm),
    }
}

fn on_cancel(w: &mut World, owner: RaceChanger, pm: MobileId) {
    if let RaceChanger::Collector(_) = owner {
        mm::send_localized_message(w, pm, 1073645, "", 0x3B2); // You may try this again later...
    }
}

// ---- RaceChangeConfirmGump -------------------------------------------------------------------------------------

/// `RaceChangeState`: what a client with the race changer open is waiting on.
struct Pending {
    owner: Option<RaceChanger>,
    target: RaceId,
    timeout: TimerHandle,
}

/// `m_Pending`.
#[derive(Default)]
struct PendingTable(HashMap<NetStateId, Pending>);

fn pending(w: &mut World) -> &mut HashMap<NetStateId, Pending> {
    &mut w.resources.get_mut::<PendingTable>().0
}

pub struct RaceChangeConfirmGump {
    owner: Option<RaceChanger>,
    race: RaceId,
    /// `_race == Race.Human`, `== Race.Elf` and `_race.Name`, read when the gump was made.
    kind: RaceKind,
}

enum RaceKind {
    Human,
    Elf,
    Other(String),
}

fn race_named(w: &World, name: &str) -> Option<RaceId> {
    w.resources.get::<Races>().and_then(|r| r.try_parse(name))
}

fn race_kind(w: &World, id: RaceId) -> RaceKind {
    if race_named(w, "Human") == Some(id) {
        RaceKind::Human
    } else if race_named(w, "Elf") == Some(id) {
        RaceKind::Elf
    } else {
        RaceKind::Other(race_ref(w, id, |r| r.name.clone()).unwrap_or_default())
    }
}

fn race_ref<R>(w: &World, id: RaceId, f: impl FnOnce(&Race) -> R) -> Option<R> {
    w.resources.get::<Races>().and_then(|r| r.get(id)).map(f)
}

impl DynamicGumpLogic for RaceChangeConfirmGump {
    const TYPE_NAME: &'static str = "Server.Engines.MLQuests.Gumps.RaceChangeConfirmGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 240, 135, 0x2422);

        match &self.kind {
            RaceKind::Human => b.add_html_localized(15, 15, 210, 75, 1073643, false, false), // embrace your humanity?
            RaceKind::Elf => b.add_html_localized(15, 15, 210, 75, 1073642, false, false), // follow the elven ways?
            RaceKind::Other(name) => b.add_html(
                15,
                15,
                210,
                75,
                &format!("Are you sure you want to change your race to {name}?"),
                false,
                false,
            ),
        }

        b.add_button(160, 95, 0xF7, 0xF8, 1);
        b.add_button(90, 95, 0xF2, 0xF1, 0);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_state(sender).and_then(|n| n.mobile) else {
            return;
        };
        if muo_content::player_mobile::data(w, from).is_none() {
            return;
        }
        match info.button_id {
            0 => {
                // Cancel
                if let Some(o) = self.owner {
                    on_cancel(w, o, from);
                }
            }
            1 => {
                // Okay: `_owner?.CheckComplete(from) != false`
                let ok = match self.owner {
                    Some(o) => check_complete(w, o, from),
                    None => true,
                };
                if ok {
                    offer(w, self.owner, from, self.race);
                }
            }
            _ => {}
        }
    }
}

/// `RaceChangeConfirmGump.DisplayTo(from, owner, targetRace)`.
pub fn display_to(
    w: &mut World,
    from: MobileId,
    owner: Option<RaceChanger>,
    target: Option<RaceId>,
) {
    let Some(target) = target else { return };
    if w.mobile(from).is_none_or(|m| m.net_state.is_none()) {
        return;
    }
    let kind = race_kind(w, target);
    let gump = RaceChangeConfirmGump {
        owner,
        race: target,
        kind,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(50, 50, gump)), false);
}

/// `IsPending(ns)`.
pub fn is_pending(w: &mut World, ns: NetStateId) -> bool {
    pending(w).contains_key(&ns)
}

/// `Offer(owner, from, targetRace)`.
fn offer(w: &mut World, owner: Option<RaceChanger>, from: MobileId, target: RaceId) {
    let Some(ns) = w.mobile(from).and_then(|m| m.net_state) else {
        return;
    };
    if !can_change(w, from, target) {
        return;
    }
    close_current(w, ns);

    let timeout = timer::start(w, TimeSpan::from_mins(1), TimeSpan::ZERO, 1, move |w| {
        timeout(w, ns)
    });
    pending(w).insert(
        ns,
        Pending {
            owner,
            target,
            timeout,
        },
    );
    let female = w.mobile(from).is_some_and(|m| m.female);
    let race_id = race_ref(w, target, |r| r.race_id).unwrap_or(0);
    send_race_changer(w, ns, female, race_id);
}

/// `CloseCurrent(ns)`.
fn close_current(w: &mut World, ns: NetStateId) -> Option<Pending> {
    let state = pending(w).remove(&ns);
    if let Some(s) = &state {
        let mut h = s.timeout;
        h.cancel(w);
    }
    send_close_race_changer(w, ns);
    state
}

/// `Timeout(ns)`.
fn timeout(w: &mut World, ns: NetStateId) {
    if is_pending(w, ns) {
        pending(w).remove(&ns);
        send_close_race_changer(w, ns);
    }
}

/// `IsWearingEquipment(from)`.
pub fn is_wearing_equipment(w: &World, from: MobileId) -> bool {
    let Some(m) = w.mobile(from) else {
        return false;
    };
    m.items.iter().any(|&i| {
        !w.item(i).is_some_and(|i| {
            matches!(
                i.layer,
                Layer::Hair | Layer::FacialHair | Layer::Backpack | Layer::Mount | Layer::Bank
            )
        })
    })
}

/// `CanChange(from, targetRace)`.
fn can_change(w: &mut World, from: MobileId, target: RaceId) -> bool {
    if w.mobile(from).is_none_or(|m| m.deleted) {
        return false;
    }
    let msg = |w: &mut World, n: i32| mm::send_localized_message(w, from, n, "", 0x3B2);

    if w.mobile(from).and_then(|m| m.race) == Some(target) {
        msg(w, 1111918); // You are already that race.
    } else if !muo_items::misc::dyes::check_ml(w, from, false) {
        msg(w, 1073651); // You must have Mondain's Legacy before proceeding...
    } else if !muo_server::mobile::base::alive(w, from) {
        msg(w, 1073646); // Only the living may proceed...
    } else if props_b::mounted(w, from) {
        msg(w, 1073647); // You may not continue while mounted...
    } else if !muo_server::mobile::part1::combat::can_begin_action(
        w,
        from,
        "Server.Spells.Seventh.PolymorphSpell",
    ) || muo_content::spells::hooks::hooks(w)
        .and_then(|h| h.is_disguised)
        .is_some_and(|f| f(w, from))
        || muo_spells::ninjitsu::animal_form::under_transformation(w, from)
        || !muo_server::mobile::part1::combat::can_begin_action(
            w,
            from,
            "Server.Spells.Fifth.IncognitoSpell",
        )
        || props_b::is_body_mod(w, from)
    {
        msg(w, 1073648); // You may only proceed while in your original state...
    } else if w
        .mobile(from)
        .and_then(|m| m.spell)
        .is_some_and(|sp| muo_content::spells::spell::is_casting(w, sp))
    {
        msg(w, 1073649); // One may not proceed while embracing magic...
    } else if props_b::poisoned(w, from) {
        msg(w, 1073652); // You must be healthy to proceed...
    } else if is_wearing_equipment(w, from) {
        msg(w, 1073650); // To proceed you must be unburdened by equipment...
    } else {
        return true;
    }
    false
}

/// `RaceChangeReply(state, reader)`; `body` follows the sub-command id (`reader.Length == 5` is no body at all).
fn race_change_reply(w: &mut World, ns: &mut NetState, body: &[u8]) {
    let id = muo_server::net_glue::ensure_net_state(w, ns);
    if !is_pending(w, id) {
        return;
    }
    let Some(state) = close_current(w, id) else {
        return;
    };

    let Some(pm) = w.net_state(id).and_then(|n| n.mobile) else {
        return;
    };
    if muo_content::player_mobile::data(w, pm).is_none() {
        return;
    }

    let owner = state.owner;
    let target = state.target;

    if body.is_empty() {
        if let Some(o) = owner {
            on_cancel(w, o, pm);
        }
        return;
    }

    if !can_change(w, pm, target) {
        return;
    }
    if let Some(o) = owner
        && !check_complete(w, o, pm)
    {
        return;
    }

    let mut r = SpanReader::new(body);
    let (Ok(hue), Ok(hair_item_id), Ok(hair_hue), Ok(facial_item_id), Ok(facial_hue)) = (
        r.read_u16(),
        r.read_u16(),
        r.read_u16(),
        r.read_u16(),
        r.read_u16(),
    ) else {
        return;
    };
    let (hue, hair_item_id, hair_hue) =
        (i32::from(hue), i32::from(hair_item_id), i32::from(hair_hue));
    let (facial_item_id, facial_hue) = (i32::from(facial_item_id), i32::from(facial_hue));

    props_a::set_race(w, pm, Some(target));
    let clipped = race_ref(w, target, |r| (r.ops.clip_skin_hue)(hue)).unwrap_or(hue);
    muo_server::mobile::base::set_hue(w, pm, clipped | 0x8000);

    let (valid_hair, clip_hair_hue) = {
        let races = w.resources.get::<Races>();
        let race = races.and_then(|r| r.get(target));
        (
            race.is_some_and(|r| muo_server::race::validate_hair(w, r, pm, hair_item_id)),
            race.map_or(hair_hue, |r| (r.ops.clip_hair_hue)(hair_hue)),
        )
    };
    if valid_hair {
        props_b::set_hair_item_id(w, pm, hair_item_id);
        props_b::set_hair_hue(w, pm, clip_hair_hue);
    } else {
        props_b::set_hair_item_id(w, pm, 0);
    }

    let (valid_facial, clip_facial_hue) = {
        let races = w.resources.get::<Races>();
        let race = races.and_then(|r| r.get(target));
        (
            race.is_some_and(|r| muo_server::race::validate_facial_hair(w, r, pm, facial_item_id)),
            race.map_or(facial_hue, |r| (r.ops.clip_hair_hue)(facial_hue)),
        )
    };
    if valid_facial {
        props_b::set_facial_hair_item_id(w, pm, facial_item_id);
        props_b::set_facial_hair_hue(w, pm, clip_facial_hue);
    } else {
        props_b::set_facial_hair_item_id(w, pm, 0);
    }

    match race_kind(w, target) {
        RaceKind::Human => mm::send_localized_message(w, pm, 1073654, "", 0x3B2), // You are now fully human.
        RaceKind::Elf => mm::send_localized_message(w, pm, 1073653, "", 0x3B2), // ...fully initiated into the Elven culture.
        RaceKind::Other(name) => mm::send_message(
            w,
            pm,
            &format!("You have fully changed your race to {name}."),
        ),
    }

    if let Some(o) = owner {
        consume_needed(w, o, pm);
    }
}

/// `RaceChangeConfirmGump.Initialize()`.
fn initialize(w: &mut World) {
    w.resources.get_mut::<PendingTable>().0.clear();
    muo_content::network::extended_commands::register_extended(w, 0x2A, true, race_change_reply);
}

// ---- RaceChangeDeed ---------------------------------------------------------------------------------------------

fn deed_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if muo_content::player_mobile::data(w, from).is_none() {
        return;
    }
    if check_complete(w, RaceChanger::Deed(id), from) {
        let is_human = w.mobile(from).and_then(|m| m.race) == race_named(w, "Human");
        let target = race_named(w, if is_human { "Elf" } else { "Human" });
        display_to(w, from, Some(RaceChanger::Deed(id)), target);
    }
}

/// `Server.Engines.MLQuests.Gumps.RaceChangeDeed`.
pub static RACE_CHANGE_DEED: ItemType = ItemType {
    full_name: "Server.Engines.MLQuests.Gumps.RaceChangeDeed",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &RACE_CHANGE_DEED, 0x14F0);
        ops::set_loot_type(w, id, LootType::Blessed);
        id
    }),
    default_name: |_, _| Some("a race change deed".to_string()),
    on_double_click: deed_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- DoneQuestCollector ---------------------------------------------------------------------------------------------

/// What `DoneQuestCollector` holds besides `BaseCreature`: `m_Timer`.
#[derive(Default)]
struct CollectorState {
    timer: TimerHandle,
}

fn nums(list: &[i32]) -> Vec<TextDefinition> {
    list.iter().map(|&n| TextDefinition::of_number(n)).collect()
}

/// `Offer`, `Incomplete`, `Complete`, `Needed` of a collector class.
struct Script {
    offer: Vec<TextDefinition>,
    incomplete: Vec<TextDefinition>,
    complete: Vec<TextDefinition>,
    needed: &'static [&'static str],
    race: &'static str,
}

fn script(w: &World, id: MobileId) -> Option<Script> {
    let name = w.mobile(id)?.ty.full_name;
    if name == DARIUS.full_name {
        Some(Script {
            offer: nums(&[
                1073998, 1073999, 1074000, 1074001, 1074398, 1074399, 1074400,
            ]),
            incomplete: nums(&[1074002, 1074003]),
            complete: nums(&[1074004, 1074005, 1074006, 1074007]),
            needed: &[
                "Seasons",
                "CaretakerOfTheLand",
                "WisdomOfTheSphynx",
                "DefendingTheHerd",
                "TheBalanceOfNature",
                "TheJoysOfLife",
            ],
            race: "Elf",
        })
    } else if name == NEDRICK.full_name {
        Some(Script {
            offer: nums(&[
                1074403, 1074404, 1074405, 1074406, 1074407, 1074408, 1074409, 1074411,
            ]),
            incomplete: nums(&[1074412, 1074413]),
            complete: nums(&[1074410, 1074531, 1074532]),
            needed: &["Ingenuity", "HeaveHo", "HumanInNeed", "AllSeasonAdventurer"],
            race: "Human",
        })
    } else {
        None
    }
}

/// `CanTalkTo(from)`: Darius talks to humans, Nedrick to elves.
fn can_talk_to(w: &World, id: MobileId, from: MobileId) -> bool {
    let Some(s) = script(w, id) else { return true };
    let wanted = if s.race == "Elf" { "Human" } else { "Elf" };
    w.mobile(from).and_then(|m| m.race) == race_named(w, wanted)
}

/// `CompletedCount(pm)`.
fn completed_count(w: &World, id: MobileId, pm: MobileId) -> usize {
    let Some(s) = script(w, id) else { return 0 };
    if engine::get_context(w, pm).is_none() {
        return 0;
    }
    s.needed
        .iter()
        .filter(|n| {
            engine::find_quest(w, n).is_none_or(|q| engine::has_done_quest(w, pm, q).is_some())
        })
        .count()
}

fn collector_check_complete(w: &mut World, id: MobileId, pm: MobileId) -> bool {
    let needed = script(w, id).map_or(0, |s| s.needed.len());
    if completed_count(w, id, pm) == needed {
        return true;
    }
    mm::send_localized_message(w, pm, 1073644, "", 0x3B2); // You must complete all the tasks before proceeding...
    false
}

fn collector_consume_needed(w: &mut World, id: MobileId, pm: MobileId) {
    let Some(s) = script(w, id) else { return };
    if engine::get_context(w, pm).is_some() {
        for n in s.needed {
            if let Some(q) = engine::find_quest(w, n) {
                engine::remove_done_quest(w, pm, q);
            }
        }
    }
}

/// `TryTalkTo(from, fromClick)`.
fn try_talk_to(w: &mut World, id: MobileId, from: MobileId, from_click: bool) {
    let ns = w.mobile(from).and_then(|m| m.net_state);
    let hidden = w.mobile(from).is_some_and(|m| m.hidden);
    let busy = ns.is_some_and(|ns| has_gump::<RaceChangeConfirmGump>(w, ns))
        || ns.is_some_and(|ns| is_pending(w, ns));
    if !hidden && !busy && can_talk_to(w, id, from) {
        talk_to(w, id, from);
    } else if from_click {
        // DenyTalk: He's too busy right now, so he ignores you.
        mm::send_localized_message(w, from, 1074017, "", 0x3B2);
    }
}

/// `GetDelay()`.
fn conversation_delay(w: &mut World) -> TimeSpan {
    TimeSpan::from_secs(if muo_content::hooks::rnd::bool(w) {
        3
    } else {
        4
    })
}

/// `TalkTo(pm)`.
fn talk_to(w: &mut World, id: MobileId, pm: MobileId) {
    if muo_content::player_mobile::data(w, pm).is_none() {
        return;
    }
    let running = w
        .mobile(id)
        .and_then(|m| m.ext.get::<CollectorState>())
        .is_some_and(|s| s.timer.running(w));
    if running {
        return;
    }
    let Some(s) = script(w, id) else { return };
    let completed = completed_count(w, id, pm);

    let (conversation, is_complete) = if completed == s.needed.len() {
        (s.complete, true)
    } else if completed == 0 {
        (s.offer, false)
    } else {
        let mut conv = s.incomplete;
        if engine::get_context(w, pm).is_some() {
            for n in s.needed {
                let q = engine::find_quest(w, n);
                if q.is_none_or(|q| engine::has_done_quest(w, pm, q).is_some()) {
                    continue;
                }
                if let Some(q) = q {
                    conv.push(engine::quest(w, q).title.clone());
                }
            }
        }
        (conv, false)
    };

    // `new InternalTimer(...)`: `base(TimeSpan.Zero, GetDelay())`, started at once
    let interval = conversation_delay(w);
    let mut index = 0usize;
    let h = timer::start(w, TimeSpan::ZERO, interval, 0, move |w| {
        conversation_tick(w, id, pm, &conversation, is_complete, &mut index);
    });
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(CollectorState { timer: h });
    }
}

/// `InternalTimer.OnTick()`.
fn conversation_tick(
    w: &mut World,
    owner: MobileId,
    target: MobileId,
    conversation: &[TextDefinition],
    is_complete: bool,
    index: &mut usize,
) {
    let stop = |w: &mut World| {
        if let Some(mut h) = w
            .mobile(owner)
            .and_then(|m| m.ext.get::<CollectorState>())
            .map(|s| s.timer)
        {
            timer::stop(w, h);
            h.cancel(w);
        }
    };
    if w.mobile(owner).is_none_or(|m| m.deleted) {
        stop(w);
        return;
    }

    if *index >= conversation.len() {
        if is_complete {
            collector_on_complete(w, owner, target);
        }
        stop(w);
        return;
    }

    if *index == 0 {
        let (show, fame, female, name) =
            w.mobile(target)
                .map_or((false, 0, false, String::new()), |m| {
                    (
                        (m.ty.show_fame_title)(w, target),
                        m.fame,
                        m.female,
                        (m.ty.name)(w, target).unwrap_or_default(),
                    )
                });
        if show && fame >= 10000 {
            mm::say(
                w,
                owner,
                true,
                &format!("{} {}", if female { "Lady" } else { "Lord" }, name),
            );
        } else {
            mm::say(w, owner, true, &name);
        }
    }

    let def = conversation[*index].clone();
    *index += 1;
    muo_content::text::public_overhead_message_mobile(
        w,
        Some(&def),
        owner,
        MessageType::REGULAR,
        0x3B2,
    );
    let interval = conversation_delay(w);
    if let Some(h) = w
        .mobile(owner)
        .and_then(|m| m.ext.get::<CollectorState>())
        .map(|s| s.timer)
    {
        timer::set_interval(w, h, interval);
    }
}

/// `OnComplete(pm)`.
fn collector_on_complete(w: &mut World, id: MobileId, pm: MobileId) {
    let Some(s) = script(w, id) else { return };
    display_to(
        w,
        pm,
        Some(RaceChanger::Collector(id)),
        race_named(w, s.race),
    );
}

fn collector_double_click(w: &mut World, id: MobileId, from: MobileId) {
    try_talk_to(w, id, from, true);
}

fn collector_movement(w: &mut World, id: MobileId, m: MobileId, old: Point3D) {
    let player = w.mobile(m).is_some_and(|x| x.player);
    if player {
        let in_range = w.mobile(id).is_some_and(|c| {
            (c.ty.in_range_point3d)(w, id, w.mobile(m).map_or(c.location, |x| x.location), 6)
        });
        let was_in_range = w
            .mobile(id)
            .is_some_and(|c| (c.ty.in_range_point3d)(w, id, old, 6));
        if in_range && !was_in_range {
            try_talk_to(w, id, m, false);
        }
    }
    (muo_content::creature::BASE_CREATURE.on_movement)(w, id, m, old);
}

#[allow(clippy::too_many_arguments)] // every argument is part of the call
fn collector_new(
    w: &mut World,
    ty: &'static muo_server::mobile::MobileType,
    race: &str,
    title: &str,
    str_: (i32, i32),
    dex: (i32, i32),
    int: (i32, i32),
    f: impl FnOnce(&mut World, MobileId),
) -> MobileId {
    let id = new_creature(w, ty, AIType::AiVendor, FightMode::None, 2, 1);
    set_speed(w, id, 0.5, 2.0, true); // SetSpeed(0.5, 2.0)
    set_title(w, id, Some(title.to_string()));
    set_race_named(w, id, race);
    let h = race_skin_hue(w, id);
    set_hue(w, id, h);
    let sh = utility::random_dyed_hue(&mut w.rng);
    if let Some(m) = w.mobile_mut(id) {
        m.speech_hue = sh;
    }
    f(w, id);
    set_str_range(w, id, str_.0, str_.1);
    set_dex_range(w, id, dex.0, dex.1);
    set_int_range(w, id, int.0, int.1);
    id
}

fn darius_new(w: &mut World) -> MobileId {
    collector_new(
        w,
        &DARIUS,
        "Elf",
        "the wise",
        (40, 50),
        (60, 70),
        (90, 100),
        |w, id| {
            add_item_hue(w, id, "WildStaff", None);
            add_item_hue(w, id, "Sandals", Some(0x1BB));
            add_item_hue(w, id, "GemmedCirclet", None);
            let h = utility::random_bright_hue(&mut w.rng);
            add_item_hue(w, id, "Tunic", Some(h));
            assign_random_hair(w, id);
        },
    )
}

fn nedrick_new(w: &mut World) -> MobileId {
    collector_new(
        w,
        &NEDRICK,
        "Human",
        "the iron worker",
        (70, 80),
        (50, 60),
        (60, 70),
        |w, id| {
            add_item_hue(w, id, "Boots", None);
            let h = utility::random_nondyed_hue(&mut w.rng);
            add_item_hue(w, id, "LongPants", Some(h));
            let h = utility::random_nondyed_hue(&mut w.rng);
            add_item_hue(w, id, "FancyShirt", Some(h));
            assign_random_hair(w, id);
            let hh = hair_hue(w, id);
            assign_random_facial_hair_hue(w, id, hh);
        },
    )
}

use muo_content::creature::setters::{set_dex_range, set_int_range, set_str_range};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};

macro_rules! collector {
    ($(#[$m:meta])* $name:ident, $class:ident = $full:literal, ctor $ctor:ident, $dname:literal) => {
        pub static $class: muo_content::creature::CreatureType = muo_content::creature::CreatureType {
            is_invulnerable: |_, _| true,
            default_name: |_, _| Some($dname),
            ..BASE_CREATURE_C
        };
        $(#[$m])*
        pub static $name: muo_server::mobile::MobileType = muo_server::mobile::MobileType {
            full_name: $full,
            parent: Some(&DONE_QUEST_COLLECTOR),
            construct: Some($ctor),
            family: Some(&$class),
            serialize: |w, id, out| {
                (DONE_QUEST_COLLECTOR.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (DONE_QUEST_COLLECTOR.deserialize)(w, id, r)?;
                r.read_i32()?;
                Ok(())
            },
            ..DONE_QUEST_COLLECTOR
        };
    };
}

/// `[SerializationGenerator(0, false)] abstract DoneQuestCollector : BaseCreature, IRaceChanger`.
pub static DONE_QUEST_COLLECTOR: muo_server::mobile::MobileType = muo_server::mobile::MobileType {
    full_name: "Server.Engines.MLQuests.Mobiles.DoneQuestCollector",
    parent: Some(&BASE_CREATURE),
    construct: None,
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    on_double_click: collector_double_click,
    on_double_click_dead: collector_double_click,
    on_movement: collector_movement,
    ..BASE_CREATURE
};

collector!(
    /// `Server.Engines.MLQuests.Mobiles.Darius`.
    DARIUS, DARIUS_C = "Server.Engines.MLQuests.Mobiles.Darius", ctor darius_new, "Darius"
);
collector!(
    /// `Server.Engines.MLQuests.Mobiles.Nedrick`.
    NEDRICK, NEDRICK_C = "Server.Engines.MLQuests.Mobiles.Nedrick", ctor nedrick_new, "Nedrick"
);

/// Registers the classes and the phase of this file.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&RACE_CHANGE_DEED, &[]);
    types.register_mobile(&DONE_QUEST_COLLECTOR, &[]);
    types.register_mobile(&DARIUS, &[]);
    types.register_mobile(&NEDRICK, &[]);
    types.register_phase_default(
        Phase::Initialize,
        "Server.Engines.MLQuests.Gumps.RaceChangeConfirmGump.Initialize",
        initialize,
    );
}
