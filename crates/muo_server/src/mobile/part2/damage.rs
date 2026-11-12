//! Damage entries, visible damage, `Heal` (the non-virtual half of the damage code).
//!
//! `_damageEntries` is `Mobile::damage_entries`, a `Vec` in `LastDamage` order. The
//! `Find*Entry` functions return an index into it, valid until the next change.

use muo_primitives::Serial;

use super::{deps, messages};
use crate::ids::MobileId;
use crate::mobile::{DamageEntry, VisibleDamageType};
use crate::net_state::NetStateId;
use crate::world::World;

/// `DamageEntry.HasExpired`.
fn has_expired(w: &World, de: &DamageEntry) -> bool {
    w.now > de.last_damage + crate::mobile::part1::props_a::damage_entry_expire_delay(w)
}

/// Entries are in `LastDamage` order, so expired ones are a prefix.
pub fn prune_expired_damage_entries(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    let n = m
        .damage_entries
        .iter()
        .take_while(|de| has_expired(w, de))
        .count();
    if n > 0
        && let Some(m) = w.mobile_mut(id)
    {
        m.damage_entries.drain(..n);
    }
}

pub fn clear_damage_entries(w: &mut World, id: MobileId) {
    if let Some(m) = w.mobile_mut(id) {
        m.damage_entries.clear();
    }
}

fn entries(w: &World, id: MobileId) -> &[DamageEntry] {
    w.mobile(id).map_or(&[], |m| m.damage_entries.as_slice())
}

pub fn find_most_recent_damager(w: &mut World, id: MobileId, allow_self: bool) -> Option<MobileId> {
    let i = find_most_recent_damage_entry(w, id, allow_self)?;
    Some(entries(w, id)[i].damager)
}

pub fn find_most_recent_damage_entry(
    w: &mut World,
    id: MobileId,
    allow_self: bool,
) -> Option<usize> {
    prune_expired_damage_entries(w, id);
    entries(w, id)
        .iter()
        .rposition(|de| allow_self || de.damager != id)
}

pub fn find_least_recent_damager(
    w: &mut World,
    id: MobileId,
    allow_self: bool,
) -> Option<MobileId> {
    let i = find_least_recent_damage_entry(w, id, allow_self)?;
    Some(entries(w, id)[i].damager)
}

pub fn find_least_recent_damage_entry(
    w: &mut World,
    id: MobileId,
    allow_self: bool,
) -> Option<usize> {
    prune_expired_damage_entries(w, id);
    entries(w, id)
        .iter()
        .position(|de| allow_self || de.damager != id)
}

pub fn find_most_total_damager(w: &mut World, id: MobileId, allow_self: bool) -> Option<MobileId> {
    let i = find_most_total_damage_entry(w, id, allow_self)?;
    Some(entries(w, id)[i].damager)
}

/// Walks most recent first with a strict comparison, so the most
/// recent entry wins ties.
pub fn find_most_total_damage_entry(
    w: &mut World,
    id: MobileId,
    allow_self: bool,
) -> Option<usize> {
    prune_expired_damage_entries(w, id);
    let mut most: Option<usize> = None;
    for (i, de) in entries(w, id).iter().enumerate().rev() {
        if (allow_self || de.damager != id)
            && most.is_none_or(|m| de.damage_given > entries(w, id)[m].damage_given)
        {
            most = Some(i);
        }
    }
    most
}

pub fn find_least_total_damager(w: &mut World, id: MobileId, allow_self: bool) -> Option<MobileId> {
    let i = find_least_total_damage_entry(w, id, allow_self)?;
    Some(entries(w, id)[i].damager)
}

pub fn find_least_total_damage_entry(
    w: &mut World,
    id: MobileId,
    allow_self: bool,
) -> Option<usize> {
    prune_expired_damage_entries(w, id);
    let mut least: Option<usize> = None;
    for (i, de) in entries(w, id).iter().enumerate().rev() {
        if (allow_self || de.damager != id)
            && least.is_none_or(|l| de.damage_given < entries(w, id)[l].damage_given)
        {
            least = Some(i);
        }
    }
    least
}

pub fn find_damage_entry_for(w: &mut World, id: MobileId, m: MobileId) -> Option<usize> {
    prune_expired_damage_entries(w, id);
    entries(w, id).iter().rposition(|de| de.damager == m)
}

/// `RegisterDamage` (the base behavior of the virtual; with `from == None`
/// nothing is registered).
pub fn register_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
) -> Option<usize> {
    let from = from?;
    let now = w.now;
    let mut de = match find_damage_entry_for(w, id, from) {
        // Move to the tail so the list stays in LastDamage order.
        Some(i) => w.mobile_mut(id)?.damage_entries.remove(i),
        None => DamageEntry {
            damager: from,
            damage_given: 0,
            last_damage: now,
            responsible: Vec::new(),
        },
    };
    de.damage_given += amount;
    de.last_damage = now;

    let master = w
        .mobile(from)
        .and_then(|f| (f.ty.get_damage_master)(w, from, id));
    if let Some(master) = master {
        match de.responsible.iter_mut().find(|c| c.damager == master) {
            Some(resp) => {
                resp.damage_given += amount;
                resp.last_damage = now;
            }
            None => de.responsible.push(DamageEntry {
                damager: master,
                damage_given: amount,
                last_damage: now,
                responsible: Vec::new(),
            }),
        }
    }

    let m = w.mobile_mut(id)?;
    m.damage_entries.push(de);
    Some(m.damage_entries.len() - 1)
}

/// `Damage` (the base behavior of the virtual).
pub fn damage(
    w: &mut World,
    id: MobileId,
    mut amount: i32,
    from: Option<MobileId>,
    inform_mount: bool,
    _ignore_evil_omen: bool,
) {
    if amount <= 0 {
        return;
    }
    let Some(m) = w.mobile(id) else { return };
    let ty = m.ty;
    if !(ty.can_be_damaged)(w, id) || m.deleted {
        return;
    }

    if !deps::region_on_damage(w, id, &mut amount) {
        return;
    }

    let old_hits = w.mobile(id).map_or(0, |m| m.hits);
    let new_hits = old_hits - amount;

    deps::spell_on_caster_hurt(w, id);

    if from.is_some() {
        (ty.register_damage)(w, id, amount, from);
    }

    (ty.disruptive_action)(w, id);

    (ty.set_paralyzed)(w, id, false);

    match crate::mobile::part1::props_a::visible_damage_type(w) {
        VisibleDamageType::Related => send_visible_damage_related(w, id, from, amount),
        VisibleDamageType::Everyone => send_visible_damage_everyone(w, id, amount),
        VisibleDamageType::Selective => send_visible_damage_selective(w, id, from, amount),
        VisibleDamageType::None => {}
    }

    (ty.on_damage)(w, id, amount, from, new_hits < 0);

    if inform_mount {
        deps::mount_on_rider_damaged(w, id, amount, from, new_hits < 0);
    }

    if new_hits < 0 {
        if let Some(m) = w.mobile_mut(id) {
            m.last_killer = from;
        }
        deps::set_hits(w, id, 0);
        if old_hits >= 0 {
            (ty.kill)(w, id);
        }
    } else {
        deps::set_hits(w, id, new_hits);
    }
}

fn net_state_of(w: &World, m: Option<MobileId>) -> Option<NetStateId> {
    m.and_then(|m| w.mobile(m)).and_then(|m| m.net_state)
}

fn damage_master(w: &World, of: MobileId, damagee: MobileId) -> Option<MobileId> {
    w.mobile(of)
        .and_then(|m| (m.ty.get_damage_master)(w, of, damagee))
}

/// `SendVisibleDamageRelated`. A mobile with
/// no client and no master's client sends nothing here.
pub fn send_visible_damage_related(
    w: &mut World,
    id: MobileId,
    from: Option<MobileId>,
    amount: i32,
) {
    let own = net_state_of(w, Some(id));
    let our_state = own.or_else(|| {
        let master = from.and_then(|f| damage_master(w, id, f));
        net_state_of(w, master)
    });
    let their_state = from.and_then(|f| {
        net_state_of(w, Some(f)).or_else(|| net_state_of(w, damage_master(w, f, id)))
    });
    let serial = w.mobile(id).map_or(Serial::ZERO, |m| m.serial);

    if amount > 0 {
        if let Some(ns) = our_state {
            crate::packets::damage::send_damage(w, ns, serial, amount);
        }
        if let Some(theirs) = their_state
            && Some(theirs) != our_state
        {
            crate::packets::damage::send_damage(w, theirs, serial, amount);
        }
    }
}

pub fn send_visible_damage_everyone(w: &mut World, id: MobileId, amount: i32) {
    if amount < 0 {
        return;
    }
    let Some(m) = w.mobile(id) else { return };
    let (Some(map), loc, serial) = (m.map, m.location, m.serial) else {
        return;
    };
    for (ns, viewer) in deps::clients_in_range(w, map, loc) {
        if deps::can_see_mobile(w, viewer, id) {
            crate::packets::damage::send_damage(w, ns, serial, amount);
        }
    }
}

pub fn send_visible_damage_selective(
    w: &mut World,
    id: MobileId,
    from: Option<MobileId>,
    amount: i32,
) {
    let mut our_state = net_state_of(w, Some(id));
    let mut their_state = net_state_of(w, from);

    let mut damager = from;
    let mut damaged = id;

    if our_state.is_none() {
        let master = from.and_then(|f| damage_master(w, id, f));
        if let Some(master) = master {
            damaged = master;
            our_state = net_state_of(w, Some(master));
        }
    }

    let Some(dm) = w.mobile(damaged) else { return };
    if !(dm.ty.show_visible_damage)(w, damaged) {
        return;
    }

    if their_state.is_none()
        && let Some(f) = from
    {
        let master = damage_master(w, f, id);
        if let Some(master) = master {
            damager = Some(master);
            their_state = net_state_of(w, Some(master));
        }
    }

    let serial = w.mobile(id).map_or(Serial::ZERO, |m| m.serial);
    if amount > 0 {
        if (dm.ty.can_see_visible_damage)(w, damaged)
            && let Some(ns) = our_state
        {
            crate::packets::damage::send_damage(w, ns, serial, amount);
        }
        if let Some(theirs) = their_state
            && Some(theirs) != our_state
            && let Some(d) = damager.and_then(|d| w.mobile(d))
            && (d.ty.can_see_visible_damage)(w, damager.unwrap_or(id))
        {
            crate::packets::damage::send_damage(w, theirs, serial, amount);
        }
    }
}

pub fn heal(w: &mut World, id: MobileId, amount: i32) {
    heal_from(w, id, amount, Some(id), true);
}

pub fn heal_from(
    w: &mut World,
    id: MobileId,
    mut amount: i32,
    from: Option<MobileId>,
    message: bool,
) {
    let Some(m) = w.mobile(id) else { return };
    let ty = m.ty;
    if !(ty.alive)(w, id) || (ty.is_dead_bonded_pet)(w, id) {
        return;
    }

    if !deps::region_on_heal(w, id, &mut amount) {
        return;
    }

    (ty.on_heal)(w, id, &mut amount, from);

    let hits = w.mobile(id).map_or(0, |m| m.hits);
    let hits_max = (ty.hits_max)(w, id);
    if hits + amount > hits_max {
        amount = hits_max - hits;
    }

    deps::set_hits(w, id, hits + amount);

    if message && amount > 0 {
        let ns = w.mobile(id).and_then(|m| m.net_state);
        if let Some(ns) = ns {
            crate::packets::message::send_message_localized_affix(
                w,
                ns,
                Serial::MINUS_ONE,
                -1,
                crate::packets::message::MessageType::LABEL,
                0x3B2,
                3,
                1008158,
                "",
                messages::AffixType::APPEND | messages::AffixType::SYSTEM,
                &amount.to_string(),
                "",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::MOBILE_BASE;
    use muo_primitives::TimeSpan;

    fn world_with_two() -> (World, MobileId, MobileId) {
        let mut w = World::new();
        let a = w.new_mobile(&MOBILE_BASE);
        let b = w.new_mobile(&MOBILE_BASE);
        (w, a, b)
    }

    #[test]
    fn register_keeps_last_damage_order_and_totals() {
        let (mut w, a, b) = world_with_two();
        w.now += TimeSpan::from_secs(1);
        register_damage(&mut w, a, 5, Some(b));
        register_damage(&mut w, a, 7, Some(a));
        let i = register_damage(&mut w, a, 3, Some(b)).unwrap();
        // b moved to the tail and its total is 8.
        assert_eq!(i, 1);
        assert_eq!(entries(&w, a)[1].damager, b);
        assert_eq!(entries(&w, a)[1].damage_given, 8);
        assert_eq!(find_most_recent_damager(&mut w, a, false), Some(b));
        assert_eq!(find_most_recent_damager(&mut w, a, true), Some(b));
        assert_eq!(find_least_recent_damager(&mut w, a, true), Some(a));
        assert_eq!(find_most_total_damager(&mut w, a, true), Some(b));
        assert_eq!(find_least_total_damager(&mut w, a, true), Some(a));
        assert_eq!(find_least_total_damager(&mut w, a, false), Some(b));
    }

    #[test]
    fn expired_entries_are_pruned_from_the_head() {
        let (mut w, a, b) = world_with_two();
        register_damage(&mut w, a, 5, Some(b));
        w.now += TimeSpan::from_mins(3);
        register_damage(&mut w, a, 1, Some(a));
        assert_eq!(find_damage_entry_for(&mut w, a, b), None);
        assert_eq!(entries(&w, a).len(), 1);
        clear_damage_entries(&mut w, a);
        assert!(entries(&w, a).is_empty());
    }
}
