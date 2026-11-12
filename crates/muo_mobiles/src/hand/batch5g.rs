//! PlagueBeastLord (monsters5): a plague beast that hardens when paralyzed and is carved open for its innards.
//! The Mutation Core items (muo_items::special::mutation) reach it through `PlagueLordApi`.

use muo_content::creature::enums::FightMode;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, cd_mut};
use muo_content::factory;
use muo_content::mx;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{MessageType, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{enumerators, items, messages};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};

use crate::common::*;

/// `m_Timer` (a `DecayTimer`) and `OpenedBy`.
#[derive(Clone, Copy, Default)]
pub struct LordState {
    /// `m_Timer != null`.
    pub has_timer: bool,
    pub count: i32,
    pub deadline: i32,
    pub handle: TimerHandle,
    pub opened_by: Option<MobileId>,
}

pub fn state(w: &World, id: MobileId) -> LordState {
    w.mobile(id)
        .and_then(|m| m.ext.get::<LordState>())
        .copied()
        .unwrap_or_default()
}
pub fn set_state(w: &mut World, id: MobileId, s: LordState) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(s);
    }
}

pub fn plague_beast_lord_opened_by(w: &World, id: MobileId) -> Option<MobileId> {
    state(w, id).opened_by
}

/// Saved: the lord's `_count` and `_deadline` (the timer keeps its own, so these stay 0), `_isNotNullTimer` (never
/// assigned, so false) and `OpenedBy`.
pub fn plague_beast_lord_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
    out.write_i32(0);
    out.write_i32(0);
    out.write_bool(false);
    out.write_mobile(w, state(w, id).opened_by);
}

/// `[AfterDeserialization]`: `IsNotNullTimer` is `m_Timer != null`, false after a load; a lord that was frozen stays so.
pub fn plague_beast_lord_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    let _count = r.read_i32()?;
    let _deadline = r.read_i32()?;
    let _is_not_null_timer = r.read_bool()?;
    let opened_by = r.read_mobile(w)?;
    set_state(
        w,
        id,
        LordState {
            opened_by,
            ..LordState::default()
        },
    );
    if muo_content::creature::cd(w, id).is_some_and(|d| d.fight_mode == FightMode::None) {
        props_a::set_frozen(w, id, true);
    }
    Ok(())
}

// ---- the DecayTimer ----

/// `new DecayTimer(lord, count, deadline)` started.
fn start_decay(w: &mut World, lord: MobileId, count: i32, deadline: i32) {
    let h = timer::start(w, TimeSpan::ZERO, TimeSpan::from_secs(1), 0, move |w| {
        decay_tick(w, lord)
    });
    let s = state(w, lord);
    set_state(
        w,
        lord,
        LordState {
            has_timer: true,
            count,
            deadline,
            handle: h,
            ..s
        },
    );
}

fn decay_tick(w: &mut World, lord: MobileId) {
    let mut s = state(w, lord);
    if mx::deleted(w, lord) {
        s.handle.cancel(w);
        set_state(w, lord, s);
        return;
    }
    let opened = s.opened_by.is_some();
    let (count, deadline) = (s.count, s.deadline);
    if count + 15 == deadline {
        if opened {
            // * The plague beast begins to bubble and dissolve! *
            messages::public_overhead_message_number(
                w,
                lord,
                MessageType::REGULAR,
                0x3B2,
                1071921,
                "",
                false,
            );
        }
        mx::play_sound(w, lord, 0x103);
    } else if count + 10 == deadline {
        mx::play_sound(w, lord, 0x21);
    } else if count + 5 == deadline {
        mx::play_sound(w, lord, 0x1C2);
    } else if count == deadline {
        plague_beast_lord_unfreeze(w, lord);
        if opened {
            muo_server::mobile::base::kill(w, lord);
        }
        let mut s = state(w, lord);
        s.handle.cancel(w);
        s.count += 1;
        set_state(w, lord, s);
        return;
    } else if count % 15 == 0 {
        mx::play_sound(w, lord, 0x1BF);
    }
    let mut s = state(w, lord);
    s.count += 1;
    set_state(w, lord, s);
}

// ---- Carve, Scissor, OnParalyzed ----

/// `IsAccessibleTo(check)`.
pub fn plague_beast_lord_is_accessible_to(w: &mut World, id: MobileId, check: MobileId) -> bool {
    if mx::access_level(w, check) >= AccessLevel::GameMaster {
        return true;
    }
    let ns = w.mobile(check).and_then(|m| m.net_state);
    let opened_by = state(w, id).opened_by;
    if !mx::in_range(w, id, check, 2) {
        messages::private_overhead_message_number(w, id, MessageType::LABEL, 0x3B2, 500446, "", ns); // That is too far away.
    } else if opened_by.is_some_and(|b| b != check) {
        messages::private_overhead_message_number(w, id, MessageType::LABEL, 0x3B2, 500365, "", ns); // That is being used by someone else
    } else if w.mobile(id).is_some_and(|m| m.frozen) {
        return true;
    }
    false
}

/// `Carve(from, item)`.
pub fn plague_beast_lord_carve(w: &mut World, id: MobileId, from: MobileId, _item: ItemId) {
    if state(w, id).opened_by.is_none() && plague_beast_lord_is_accessible_to(w, id, from) {
        let mut s = state(w, id);
        s.opened_by = Some(from);
        set_state(w, id, s);
        if !state(w, id).has_timer {
            // m_Timer ??= new DecayTimer(this)
            let s = state(w, id);
            set_state(
                w,
                id,
                LordState {
                    has_timer: true,
                    count: 0,
                    deadline: 120,
                    ..s
                },
            );
        }
        if !state(w, id).handle.running(w) {
            let s = state(w, id);
            start_decay(w, id, s.count, s.deadline);
        }
        // StartDissolving
        let mut s = state(w, id);
        s.deadline = s.deadline.min(s.count + 60);
        set_state(w, id, s);

        let pack = muo_items::special::mutation::backpack_new(w);
        items::add_item(w, id, pack);
        muo_items::special::mutation::backpack_initialize(w, pack);

        let from_name = w
            .mobile(from)
            .and_then(|m| (m.ty.name)(w, from))
            .unwrap_or_default();
        for m in enumerators::get_mobiles_in_range(w, id, 12) {
            let ns = w.mobile(m).and_then(|x| x.net_state);
            if ns.is_some() && mx::player(w, m) && m != from {
                // * ~1_VAL~ slices through the plague beast's amorphous tissue *
                messages::private_overhead_message_number(
                    w,
                    id,
                    MessageType::REGULAR,
                    0x3B2,
                    1071919,
                    &from_name,
                    ns,
                );
            }
        }
        // * You slice through the plague beast's amorphous tissue *
        messages::local_overhead_message_number(w, from, MessageType::REGULAR, 0x21, 1071904, "");
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            muo_content::items::cont::open(w, pack, from)
        });
    }
}

/// `OnParalyzed(from)`.
pub fn plague_beast_lord_on_paralyzed(w: &mut World, id: MobileId, from: MobileId) {
    super::ctors::set_fight_mode(w, id, FightMode::None);
    props_a::set_frozen(w, id, true);
    props_a::set_blessed(w, id, true);
    mx::set_combatant(w, id, None);
    hue(w, id, 0x480);
    mx::set_combatant(w, from, None);
    mx::set_warmode(w, from, false);

    start_decay(w, id, 0, 120);

    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        // * The plague beast's amorphous flesh hardens and becomes immobilized *
        messages::public_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            1071920,
            "",
            false,
        );
    });
}

/// `Unfreeze()`.
pub fn plague_beast_lord_unfreeze(w: &mut World, id: MobileId) {
    super::ctors::set_fight_mode(w, id, FightMode::Closest);
    props_a::set_frozen(w, id, false);
    props_a::set_blessed(w, id, false);
    if state(w, id).opened_by.is_none() {
        hue(w, id, 0);
    }
}

// ---- the overrides ----

pub fn plague_beast_lord_on_double_click(w: &mut World, id: MobileId, from: MobileId) {
    if plague_beast_lord_is_accessible_to(w, id, from) {
        let pack = props_b::backpack(w, id);
        match (state(w, id).opened_by, pack) {
            (Some(_), Some(pack)) => {
                if let Some(ty) = w.item(pack).map(|i| i.ty) {
                    (ty.display_to)(w, pack, from);
                }
            }
            _ => {
                let ns = w.mobile(from).and_then(|m| m.net_state);
                // * You attempt to tear open the amorphous flesh, but it resists *
                messages::private_overhead_message_number(
                    w,
                    id,
                    MessageType::REGULAR,
                    0x3B2,
                    1071917,
                    "",
                    ns,
                );
            }
        }
    }
}

pub fn plague_beast_lord_on_drag_drop(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    dropped: ItemId,
) -> bool {
    if plague_beast_lord_is_accessible_to(w, id, from)
        && (factory::item_is(w, dropped, "Server.Items.PlagueBeastInnard")
            || factory::item_is(w, dropped, "Server.Items.PlagueBeastGland"))
    {
        return (BASE_CREATURE.on_drag_drop)(w, id, from, dropped);
    }
    false
}

/// `base.OnDeath(c)`, then the corpse is emptied.
pub fn plague_beast_lord_on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    (BASE_CREATURE.on_death)(w, id, c);
    if let Some(c) = c {
        let inside: Vec<ItemId> = w.item(c).map(|i| i.items.clone()).unwrap_or_default();
        for i in inside.into_iter().rev() {
            muo_server::item::base::delete(w, i);
        }
    }
}

pub fn plague_beast_lord_on_delete(w: &mut World, id: MobileId) {
    if let Some(h) = state(w, id)
        .opened_by
        .and_then(|b| w.mobile(b))
        .and_then(|m| m.holding)
        && factory::item_is(w, h, "Server.Items.PlagueBeastInnard")
    {
        muo_server::item::base::delete(w, h);
    }
    if let Some(pack) = props_b::backpack(w, id) {
        let inside: Vec<ItemId> = w.item(pack).map(|i| i.items.clone()).unwrap_or_default();
        for i in inside.into_iter().rev() {
            muo_server::item::base::delete(w, i);
        }
        muo_server::item::base::delete(w, pack);
    }
    (BASE_CREATURE.on_delete)(w, id);
}

pub fn plague_beast_lord_on_movement(w: &mut World, id: MobileId, m: MobileId, old: Point3D) {
    (BASE_CREATURE.on_movement)(w, id, m, old);
    if let Some(pack) = props_b::backpack(w, id)
        && plague_beast_lord_is_accessible_to(w, id, m)
        && mx::in_range_point(w, m, old, 3)
        && !mx::in_range(w, m, id, 3)
    {
        muo_server::item::ops::send_remove_packet(w, pack);
    }
}

pub fn plague_beast_lord_check_nonlocal_lift(
    _w: &mut World,
    _id: MobileId,
    _from: MobileId,
    _item: ItemId,
) -> bool {
    true
}

pub fn plague_beast_lord_check_nonlocal_drop(
    _w: &mut World,
    _id: MobileId,
    _from: MobileId,
    _item: ItemId,
    _target: ItemId,
) -> bool {
    true
}

pub fn plague_beast_lord_is_snoop(_w: &World, _id: MobileId, _from: MobileId) -> bool {
    false
}

/// `IsBleeding` is `muo_items::special::mutation::is_bleeding`.
pub fn plague_beast_lord_is_lord(w: &World, id: MobileId) -> bool {
    factory::mobile_is(w, id, "Server.Mobiles.PlagueBeastLord")
}

#[allow(unused)]
fn unused(w: &mut World, id: MobileId) {
    let _ = (BASE_CREATURE_C.on_think, cd_mut(w, id));
}
