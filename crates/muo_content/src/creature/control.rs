//! `BaseCreature`: masters and followers, pet orders, friends, summoning, the unsummon and loyalty timers.

use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::mobile::part1::props_a;
use muo_server::mobile::part2::properties::invalidate_properties;
use muo_server::mobile::{MobileDelta, base};
use muo_server::timer;
use muo_server::world::World;

use super::data::{cd, cd_mut, read};
use super::enums::*;
use super::vtable::ctype;
use crate::hooks::{hooks, pet_player_hooks, rnd};
use crate::mx;

// ---- masters -------------------------------------------------------------------------------------------

/// `ControlMaster`: the master while `Controlled`, else null.
pub fn control_master(w: &World, id: MobileId) -> Option<MobileId> {
    cd(w, id).filter(|d| d.controlled).and_then(|d| d.master)
}

/// `SummonMaster`: the master while `Summoned`, else null.
pub fn summon_master(w: &World, id: MobileId) -> Option<MobileId> {
    cd(w, id).filter(|d| d.summoned).and_then(|d| d.master)
}

/// `GetMaster`: whoever answers for the creature.
pub fn get_master(w: &World, id: MobileId) -> Option<MobileId> {
    control_master(w, id).or_else(|| summon_master(w, id))
}

/// `RemoveFollowers`
pub fn remove_followers(w: &mut World, id: MobileId) {
    let Some(d) = cd(w, id) else { return };
    let (Some(master), slots) = (d.master, d.control_slots) else {
        return;
    };
    let followers = w.mobile(master).map_or(0, |m| m.followers);
    props_a::set_followers(w, master, followers - slots.min(followers));
    if let Some(f) = pet_player_hooks(w).and_then(|h| h.remove_follower) {
        f(w, master, id);
    }
}

/// `AddFollowers`
pub fn add_followers(w: &mut World, id: MobileId) {
    let Some(d) = cd(w, id) else { return };
    let (Some(master), slots) = (d.master, d.control_slots) else {
        return;
    };
    let followers = w.mobile(master).map_or(0, |m| m.followers);
    props_a::set_followers(w, master, followers + slots);
    if let Some(f) = pet_player_hooks(w).and_then(|h| h.add_follower) {
        f(w, master, id);
    }
}

/// `Master` setter (also `ControlMaster`/`SummonMaster`): every assignment moves the follower slots.
pub fn set_master(w: &mut World, id: MobileId, value: Option<MobileId>) {
    let current = cd(w, id).and_then(|d| d.master);
    if current == value || value == Some(id) {
        return;
    }
    remove_followers(w, id);
    if let Some(d) = cd_mut(w, id) {
        d.master = value;
    }
    add_followers(w, id);
    if value.is_some() {
        stop_delete_timer(w, id);
    }
    base::delta(w, id, MobileDelta::NOTO);
}

/// `Controlled = value` (`fieldChanged: OnControlledChange`).
pub fn set_controlled(w: &mut World, id: MobileId, value: bool) {
    let Some(d) = cd_mut(w, id) else { return };
    let changed = d.controlled != value;
    d.controlled = value;
    if changed {
        base::delta(w, id, MobileDelta::NOTO);
        invalidate_properties(w, id);
    }
}

/// `Summoned = value` (`fieldChanged: OnSummonedChange`).
pub fn set_summoned(w: &mut World, id: MobileId, value: bool) {
    let tick = w.tick_count;
    let Some(d) = cd_mut(w, id) else { return };
    let changed = d.summoned != value;
    d.summoned = value;
    if changed {
        d.next_reacquire_time = tick;
        base::delta(w, id, MobileDelta::NOTO);
        invalidate_properties(w, id);
    }
}

/// `IsBonded = value` (`fieldChanged: OnBondedChange`).
pub fn set_is_bonded(w: &mut World, id: MobileId, value: bool) {
    let Some(d) = cd_mut(w, id) else { return };
    let changed = d.is_bonded != value;
    d.is_bonded = value;
    if changed {
        invalidate_properties(w, id);
    }
}

/// `Loyalty = value`: `allowFieldChange: ClampLoyalty`.
pub fn set_loyalty(w: &mut World, id: MobileId, value: i32) {
    if let Some(d) = cd_mut(w, id) {
        d.loyalty = value.clamp(0, MAX_LOYALTY);
    }
}

/// `Team = value` (`fieldChanged: OnTeamChange`).
pub fn set_team(w: &mut World, id: MobileId, value: i32) {
    let Some(d) = cd_mut(w, id) else { return };
    let changed = d.team != value;
    d.team = value;
    if changed && let Some(c) = ctype(w, id) {
        (c.on_team_change)(w, id);
    }
}

/// `Home`.
pub fn set_home(w: &mut World, id: MobileId, value: Point3D) {
    if let Some(d) = cd_mut(w, id) {
        d.home = value;
    }
}

/// Resistance seeds change: `UpdateResistances()`.
pub fn update_resistances(w: &mut World, id: MobileId) {
    if let Some(m) = w.mobile(id) {
        (m.ty.update_resistances)(w, id);
    }
}

// ---- orders ---------------------------------------------------------------------------------------------

/// `ControlOrder` setter: a system-issued assignment.
pub fn set_control_order_raw(w: &mut World, id: MobileId, order: OrderType) {
    set_control_order(
        w,
        id,
        order,
        None,
        false,
        cd(w, id).and_then(|d| d.control_target),
    );
}

/// `IssueOrder`: gives this pet a command. `issuer` (None = system-issued) is the only mobile revealed.
pub fn issue_order(
    w: &mut World,
    id: MobileId,
    order: OrderType,
    issuer: Option<MobileId>,
    target: Option<MobileId>,
) {
    // The interrupted order owns this; BaseAI.ResumeInterrupted hands it back.
    let interrupted = cd(w, id).and_then(|d| d.control_target);
    if let Some(d) = cd_mut(w, id) {
        d.control_target = target;
    }
    set_control_order(w, id, order, issuer, false, interrupted);
}

/// `SetControlOrder`: loops until the Issue phase returns an order that rests.
pub fn set_control_order(
    w: &mut World,
    id: MobileId,
    mut order: OrderType,
    mut issuer: Option<MobileId>,
    mut resuming: bool,
    interrupted_target: Option<MobileId>,
) {
    let has_ai = cd(w, id).is_some_and(|d| d.ai.is_some());
    let mut previous = cd(w, id).map_or(OrderType::None, |d| d.control_order);
    if let Some(d) = cd_mut(w, id) {
        d.control_order = order;
    }
    if has_ai {
        let mut depth = 0;
        loop {
            let next = crate::ai::issue_order(
                w,
                id,
                order,
                previous,
                issuer,
                resuming,
                interrupted_target,
            );
            // A nested assignment (SetControlMaster(null), Kill()) already resolved itself; it wins.
            let current = cd(w, id).map_or(order, |d| d.control_order);
            if current != order || next == order {
                break;
            }
            if depth >= 8 {
                // Non-converging override: rest at the standing order.
                let rest = crate::ai::persistent_order(w, id);
                if let Some(d) = cd_mut(w, id) {
                    d.control_order = rest;
                }
                break;
            }
            depth += 1;
            previous = order;
            order = next;
            issuer = None; // chained resolutions reveal nobody
            resuming = true;
            if let Some(d) = cd_mut(w, id) {
                d.control_order = order;
            }
        }
    }
    invalidate_properties(w, id);
    if let Some(cm) = control_master(w, id) {
        invalidate_properties(w, cm);
    }
}

/// `SetControlMaster`
pub fn set_control_master(w: &mut World, id: MobileId, m: Option<MobileId>) -> bool {
    match m {
        None => {
            // An uncontrolled summon keeps its caster.
            if cd(w, id).is_some_and(|d| d.controlled) {
                set_master(w, id, None);
            }
            set_controlled(w, id, false);
            if let Some(d) = cd_mut(w, id) {
                d.control_target = None;
            }
            set_control_order_raw(w, id, OrderType::None);
        }
        Some(m) => {
            if let Some(sp) = w
                .mobile(id)
                .and_then(|x| x.spawner)
                .filter(|&sp| muo_server::mobile::part2::deps::spawner_unlink_on_taming(w, sp))
            {
                muo_server::mobile::part2::deps::spawner_remove(
                    w,
                    sp,
                    muo_server::ids::EntityId::Mobile(id),
                );
                if let Some(x) = w.mobile_mut(id) {
                    x.spawner = None;
                }
            }
            let (followers, max) = w
                .mobile(m)
                .map_or((0, 0), |x| (x.followers, x.followers_max));
            let slots = cd(w, id).map_or(1, |d| d.control_slots);
            if followers + slots > max {
                mx::msg(w, m, 1049607); // You have too many followers to control that creature.
                return false;
            }
            if let Some(d) = cd_mut(w, id) {
                d.current_way_point = None; // so tamed animals don't try to go back
                d.home = Point3D::ZERO;
            }
            set_master(w, id, Some(m));
            set_controlled(w, id, true);
            if let Some(d) = cd_mut(w, id) {
                d.control_target = None;
            }
            set_control_order_raw(w, id, OrderType::Come);
            stop_delete_timer(w, id);
        }
    }
    muo_server::mobile::part1::props_b::set_guild(w, id, None);
    base::delta(w, id, MobileDelta::NOTO);
    invalidate_properties(w, id);
    true
}

// ---- timers ---------------------------------------------------------------------------------------------

/// `StopPendingDeleteTimer` / `StopDeleteTimer`.
pub fn stop_delete_timer(w: &mut World, id: MobileId) {
    let h = cd(w, id).map(|d| d.pending_delete_timer);
    if let Some(mut h) = h {
        if h.running(w) || h != Default::default() {
            h.cancel(w);
        }
        if let Some(d) = cd_mut(w, id) {
            d.pending_delete_timer = h;
        }
    }
}

/// `BeginDeleteTimer`
pub fn begin_delete_timer(w: &mut World, id: MobileId) {
    let (summoned, stabled) = read(w, id, (true, true), |d| (d.summoned, d.is_stabled));
    let escortable = crate::factory::mobile_is(w, id, "Server.Mobiles.BaseEscortable");
    if !escortable && !summoned && !super::misc::deleted(w, id) && !stabled {
        stop_delete_timer(w, id);
        super::misc::start_delete_timer(w, id, TimeSpan::from_days(3));
    }
}

/// `new UnsummonTimer(creature, delay, onUnsummon).Start()`: one per creature.
pub fn start_unsummon_timer(
    w: &mut World,
    id: MobileId,
    delay: TimeSpan,
    on_unsummon: Option<fn(&mut World, MobileId)>,
) {
    stop_unsummon_timer(w, id);
    let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        if let Some(m) = w.mobile(id).map(|m| m.ty) {
            (m.delete)(w, id);
        }
        if let Some(f) = on_unsummon {
            f(w, id);
        }
    });
    if let Some(d) = cd_mut(w, id) {
        d.unsummon_timer = h;
    }
}

/// `UnsummonTimer.StopTimer(creature)`.
pub fn stop_unsummon_timer(w: &mut World, id: MobileId) {
    let h = cd(w, id).map(|d| d.unsummon_timer);
    if let Some(mut h) = h {
        h.cancel(w);
        if let Some(d) = cd_mut(w, id) {
            d.unsummon_timer = h;
        }
    }
}

/// `Summon(creature, controlled, caster, p, sound, duration, onUnsummon)`.
#[allow(clippy::too_many_arguments)] // one argument per summon option
pub fn summon(
    w: &mut World,
    creature: MobileId,
    controlled: bool,
    caster: MobileId,
    p: Point3D,
    sound: i32,
    duration: TimeSpan,
    on_unsummon: Option<fn(&mut World, MobileId)>,
) -> bool {
    let (followers, max) = w
        .mobile(caster)
        .map_or((0, 0), |m| (m.followers, m.followers_max));
    let slots = cd(w, creature).map_or(1, |d| d.control_slots);
    if followers + slots > max {
        mx::msg(w, caster, 1049645); // You have too many followers to summon that creature.
        if let Some(ty) = w.mobile(creature).map(|m| m.ty) {
            (ty.delete)(w, creature);
        }
        return false;
    }
    w.resources.get_mut::<SummonState>().summoning = true;
    if controlled {
        set_control_master(w, creature, Some(caster));
    }
    if let Some(d) = cd_mut(w, creature) {
        d.range_home = 10;
    }
    set_summoned(w, creature, true);
    set_master(w, creature, Some(caster));
    if let Some(pack) = muo_server::mobile::part1::props_b::backpack(w, creature) {
        let items = w.item(pack).map(|i| i.items.clone()).unwrap_or_default();
        for it in items.into_iter().rev() {
            if let Some(ty) = w.item(it).map(|i| i.ty) {
                (ty.delete)(w, it);
            }
        }
    }
    start_unsummon_timer(w, creature, duration, on_unsummon);
    let end = w.now + duration;
    if let Some(d) = cd_mut(w, creature) {
        d.summon_end = end;
    }
    let map = mx::map(w, caster).unwrap_or(MapId::INTERNAL);
    base::move_to_world(w, creature, p, map);
    muo_server::effects::play_sound(w, p, Some(map), sound);
    w.resources.get_mut::<SummonState>().summoning = false;
    true
}

/// `BaseCreature.Summoning` (a static).
#[derive(Default)]
pub struct SummonState {
    pub summoning: bool,
}

// ---- pet friends -------------------------------------------------------------------------------------

pub fn is_pet_friend(w: &World, id: MobileId, m: MobileId) -> bool {
    read(w, id, false, |d| {
        d.friends.as_ref().is_some_and(|f| f.contains(&m))
    })
}

pub fn add_pet_friend(w: &mut World, id: MobileId, m: MobileId) {
    if is_pet_friend(w, id, m) {
        return;
    }
    if let Some(d) = cd_mut(w, id) {
        d.friends.get_or_insert_with(Vec::new).push(m);
    }
}

pub fn remove_pet_friend(w: &mut World, id: MobileId, m: MobileId) {
    let Some(d) = cd_mut(w, id) else { return };
    let Some(f) = d.friends.as_mut() else { return };
    f.retain(|x| *x != m);
    if f.is_empty() {
        d.friends = None;
    }
}

pub fn clear_pet_friends(w: &mut World, id: MobileId) {
    if let Some(d) = cd_mut(w, id) {
        d.friends = None;
    }
}

/// `IsFriend`
pub fn is_friend(w: &World, id: MobileId, m: MobileId) -> bool {
    let group_enemy = ctype(w, id)
        .and_then(|c| (c.opposition_group)(w, id))
        .is_some_and(|g| g.is_enemy(w, id, m));
    if group_enemy {
        return false;
    }
    let (Some(a), Some(b)) = (cd(w, id), cd(w, m)) else {
        return false;
    };
    a.team == b.team && (a.summoned || a.controlled) == (b.summoned || b.controlled)
}

// ---- minions ----------------------------------------------------------------------------------------

/// What the champion spawn engine (Engines/CannedEvil) answers for `SpawnMinion`, set by its crate.
#[derive(Default)]
pub struct MinionHooks {
    /// `ChampionSpawn.FindSpawnOf(creature)?.AddMinion(minion)`: `(creature, minion)`. Unset: nothing.
    pub champion_add_minion: Option<fn(&mut World, MobileId, MobileId)>,
}

/// SpawnMinion for a minion already made. False, and the minion is deleted, if the creature isn't in the world.
pub fn spawn_minion(
    w: &mut World,
    id: MobileId,
    minion: MobileId,
    target: Option<MobileId>,
    location: Option<Point3D>,
) -> bool {
    let map = mx::map(w, id);
    let deleted = super::misc::deleted(w, id);
    let Some(map) = map.filter(|m| !deleted && *m != MapId::INTERNAL) else {
        if let Some(ty) = w.mobile(minion).map(|m| m.ty) {
            (ty.delete)(w, minion);
        }
        return false;
    };
    let team = cd(w, id).map_or(0, |d| d.team);
    set_team(w, minion, team);
    // `ChampionSpawn.FindSpawnOf(this)?.AddMinion(minion)`
    if let Some(f) = w
        .resources
        .get::<MinionHooks>()
        .and_then(|h| h.champion_add_minion)
    {
        f(w, id, minion);
    }
    let loc = match location {
        Some(l) => l,
        None => {
            let here = mx::location(w, id);
            muo_server::map::fit::get_random_nearby_location(
                w, map, here, 2, 0, 10, 16, false, false,
            )
        }
    };
    base::move_to_world(w, minion, loc, map);
    if let Some(t) = target {
        mx::set_combatant(w, minion, Some(t));
    }
    true
}

/// `SpawnMinion(new T { FightMode = mode }, target)` for the class named `full_name`.
pub fn spawn_minion_by_name(
    w: &mut World,
    id: MobileId,
    full_name: &str,
    target: Option<MobileId>,
    fight_mode: Option<FightMode>,
) -> bool {
    let Some(m) = crate::factory::make_mobile(w, full_name) else {
        return false;
    };
    if let Some(fm) = fight_mode {
        super::data::edit(w, m, |d| d.fight_mode = fm);
    }
    spawn_minion(w, id, m, target, None)
}

/// `Utility.GetValidLocationInLOS(map, from, range, retries = 10)`.
pub fn valid_location_in_los(
    w: &mut World,
    map: Option<MapId>,
    from: MobileId,
    range: i32,
) -> Point3D {
    use muo_server::map::{fit::can_spawn_mobile_at, los::line_of_sight_mobile_point};
    let center = mx::location(w, from);
    let Some(map) = map else { return center };
    for _ in 0..10 {
        let x = center.x + (crate::hooks::rnd::below(w, range * 2 + 1) - range);
        let y = center.y + (crate::hooks::rnd::below(w, range * 2 + 1) - range);
        let mut loc = Point3D::new(x, y, center.z);
        if can_spawn_mobile_at(w, map, loc.x, loc.y, loc.z)
            && line_of_sight_mobile_point(w, map, from, loc)
        {
            return loc;
        }
        loc.z = w.maps.get_average_z(map, x, y);
        if can_spawn_mobile_at(w, map, loc.x, loc.y, loc.z)
            && line_of_sight_mobile_point(w, map, from, loc)
        {
            return loc;
        }
    }
    center
}

// ---- the LoyaltyTimer -------------------------------------------------------------------------------

/// `LoyaltyTimer.Initialize`: every 5 minutes, hourly work.
pub fn initialize_loyalty_timer(w: &mut World) {
    let mut next_hourly = w.now + TimeSpan::from_hours(1);
    timer::start(
        w,
        TimeSpan::from_mins(5),
        TimeSpan::from_mins(5),
        0,
        move |w| {
            if w.now < next_hourly {
                return;
            }
            next_hourly = w.now + TimeSpan::from_hours(1);
            loyalty_tick(w);
        },
    );
}

fn loyalty_tick(w: &mut World) {
    let ids: Vec<MobileId> = w
        .mobiles
        .keys()
        .copied()
        .filter(|&m| cd(w, m).is_some())
        .collect();
    let mut to_release = Vec::new();
    let mut to_remove = Vec::new();
    let now = w.now;
    for c in ids {
        if crate::factory::mobile_is(w, c, "Server.Mobiles.BaseMount")
            && crate::hooks::world_hooks(w)
                .and_then(|h| h.mount_rider)
                .is_some_and(|f| f(w, c).is_some())
        {
            if let Some(d) = cd_mut(w, c) {
                d.owner_abandon_time = DateTime::MIN;
            }
            continue;
        }
        let d = cd(w, c).expect("filtered above");
        let (dead, stabled, controlled, abandon) = (
            d.is_dead_pet,
            d.is_stabled,
            d.controlled,
            d.owner_abandon_time,
        );
        let commandable = ctype(w, c).is_some_and(|t| (t.commandable)(w, c));
        if dead {
            let owner = control_master(w, c);
            let far = owner.is_none_or(|o| {
                mx::deleted(w, o)
                    || mx::map(w, o) != mx::map(w, c)
                    || !mx::in_range(w, o, c, 12)
                    || !w
                        .mobile(c)
                        .map(|m| m.ty)
                        .is_some_and(|m| (m.can_see_mobile)(w, c, o))
            }) || !owner.is_some_and(|o| mx::in_los(w, c, o));
            if !stabled && far {
                if abandon == DateTime::MIN {
                    cd_mut(w, c).expect("creature").owner_abandon_time = now;
                } else {
                    let delay = ctype(w, c)
                        .map_or(TimeSpan::from_days(1), |t| (t.bonding_abandon_delay)(w, c));
                    if abandon + delay <= now {
                        to_remove.push(c);
                    }
                }
            } else {
                cd_mut(w, c).expect("creature").owner_abandon_time = DateTime::MIN;
            }
        } else if controlled && commandable {
            cd_mut(w, c).expect("creature").owner_abandon_time = DateTime::MIN;
            if mx::map(w, c) != Some(MapId::INTERNAL) {
                let loyalty = cd(w, c).map_or(0, |d| d.loyalty) - MAX_LOYALTY / 10;
                set_loyalty(w, c, loyalty);
                let loyalty = cd(w, c).map_or(0, |d| d.loyalty);
                if loyalty < MAX_LOYALTY / 10 {
                    let name = base::name(w, c).unwrap_or_default();
                    muo_server::mobile::part2::messages::say_number(w, c, 1043270, &name); // * ~1_NAME~ looks around desperately *
                    let snd = w
                        .mobile(c)
                        .map(|m| m.ty)
                        .map_or(0, |m| (m.get_idle_sound)(w, c));
                    mx::play_sound(w, c, snd);
                }
                if loyalty <= 0 {
                    to_release.push(c);
                }
            }
        }
        // Wild creatures squatting in houses are removed outright.
        let d = cd(w, c).expect("creature");
        let in_house = crate::hooks::world_hooks(w)
            .and_then(|h| h.in_house_region)
            .is_some_and(|f| f(w, c))
            && super::overrides::can_be_damaged(w, c);
        if !d.controlled
            && !d.is_stabled
            && (in_house || d.remove_if_untamed && d.spawner_is_none(w, c))
        {
            let step = d.remove_step + 1;
            cd_mut(w, c).expect("creature").remove_step = step;
            if step >= 20 {
                to_remove.push(c);
            }
        } else {
            cd_mut(w, c).expect("creature").remove_step = 0;
        }
    }
    for c in to_release {
        let name = base::name(w, c).unwrap_or_default();
        muo_server::mobile::part2::messages::say_number(w, c, 1043255, &name); // ~1_NAME~ appears to have decided that is better off without a master!
        set_loyalty(w, c, MAX_LOYALTY);
        set_control_order_raw(w, c, OrderType::Release);
    }
    for c in to_remove {
        if let Some(ty) = w.mobile(c).map(|m| m.ty) {
            (ty.delete)(w, c);
        }
    }
}

impl super::data::CreatureData {
    /// `Spawner == null`.
    fn spawner_is_none(&self, w: &World, id: MobileId) -> bool {
        w.mobile(id).is_none_or(|m| m.spawner.is_none())
    }
}

#[allow(dead_code)]
fn unused() {
    let _ = (rnd::double, hooks);
}
