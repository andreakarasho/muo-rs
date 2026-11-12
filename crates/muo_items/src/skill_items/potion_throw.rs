//! The thrown potions of Items/Skill Items/Magical/Potions: `BaseExplosionPotion`, `BaseConfusionBlastPotion` and
//! `BaseConflagrationPotion` (their `ThrowTarget`s, the explosion's `DetonateTimer`, the use delays).
//!
//! `ThrowTarget`s are tracked by `TargetId` in [`ThrowRt`] (`from.Target as ThrowTarget`). The fire field of a
//! conflagration potion is `BaseConflagrationPotion.InternalItem` ([`FIRE_FIELD`]).

use std::collections::HashMap;

use muo_content::misc::aos_helpers::{DamageOptions, DamageSplit};
use muo_content::spells::helper::{self, Point3dRef};
use muo_primitives::{DateTime, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{base, deps, ops};
use muo_server::map::MapId;
use muo_server::packets::message::MessageType;
use muo_server::target::{self, TargetFlags, TargetId, Targeted};
use muo_server::timer::{self, TimerHandle};

use super::potions::{self, Kind};

/// The runtime state of the thrown potions.
#[derive(Default)]
pub struct ThrowRt {
    /// The `ThrowTarget` of a mobile, and its potion.
    targets: HashMap<TargetId, ItemId>,
    /// `_users` of each potion.
    users: HashMap<ItemId, Vec<MobileId>>,
    /// `_timer` of each explosion potion: the handle and the time it fires next.
    detonate: HashMap<ItemId, (TimerHandle, i32, DateTime)>,
    /// `_delay`: the next use of a conflagration (0) or confusion blast (1) potion, per mobile.
    delay: HashMap<(u8, MobileId), (TimerHandle, DateTime)>,
}

fn rt(w: &mut World) -> &mut ThrowRt {
    w.resources.get_mut::<ThrowRt>()
}

/// `(from.Target as ThrowTarget)?.Potion`.
fn target_potion(w: &mut World, m: MobileId) -> Option<ItemId> {
    let t = w.mobile(m)?.target?;
    rt(w).targets.get(&t).copied()
}

pub fn can_drink(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let kind = potions::class(w, id).kind;
    let aos = muo_content::core::aos(w);
    let casting = w
        .mobile(from)
        .and_then(|m| m.spell)
        .is_some_and(|s| muo_content::spells::spell::is_casting(w, s));
    let frozen =
        w.mobile(from).is_some_and(|m| m.frozen) || muo_server::mobile::base::paralyzed(w, from);
    if aos && (frozen || casting) {
        // You can not use a purple potion while paralyzed. / You can not use that potion while paralyzed.
        deps::send_localized_message(w, from, 1062725);
        return false;
    }
    if let Kind::Conflagration(..) | Kind::Confusion(_) = kind {
        let which = u8::from(matches!(kind, Kind::Confusion(_)));
        let delay = get_delay(w, which, from);
        if delay > 0 {
            // You cannot use that for another ~1_NUM~ ~2_TIMEUNITS~
            let unit = if delay > 1 { "seconds." } else { "second." };
            muo_server::mobile::part2::messages::send_localized_message(
                w,
                from,
                1072529,
                &format!("{delay}\t{unit}"),
                0x3B2,
            );
            return false;
        }
    }
    target_potion(w, from) != Some(id)
}

pub fn drink(w: &mut World, id: ItemId, from: MobileId) {
    let kind = potions::class(w, id).kind;
    if let Kind::Explosion { .. } = kind {
        // Scavenged explosion potions won't stack with those ones in backpack, and still will explode.
        ops::set_stackable(w, id, false);
    }
    muo_server::mobile::base::revealing_action(w, from);
    rt(w).users.entry(id).or_default().push(from);
    begin_throw(w, id, from, kind);
    if let Kind::Explosion { .. } = kind {
        let running = rt(w).detonate.contains_key(&id);
        if !running {
            deps::send_localized_message(w, from, 500236); // You should throw it now!
            let (delay, interval, count) = if muo_content::core::ml(w) {
                (1.0, 1.25, 5)
            } else {
                (0.75, 1.0, 4)
            };
            start_detonate(w, id, from, delay, interval, count);
        }
    }
}

fn begin_throw(w: &mut World, potion: ItemId, from: MobileId, kind: Kind) {
    let range = match kind {
        Kind::Explosion { .. } if muo_content::core::ml(w) => 12,
        Kind::Explosion { .. } => 10,
        _ => 12,
    };
    let tid = target::begin_target(
        w,
        from,
        range,
        true,
        TargetFlags::NONE,
        move |w, from, t| on_throw_target(w, potion, from, t, kind),
    );
    if let Some(t) = w.targets.get_mut(&tid) {
        t.handlers.on_target_finish = Some(std::rc::Rc::new(move |w, from| {
            if let Some(u) = rt(w).users.get_mut(&potion) {
                u.retain(|&m| m != from);
            }
        }));
    }
    rt(w).targets.insert(tid, potion);
}

fn point_of(w: &World, p: Point3dRef) -> Point3D {
    match p {
        Point3dRef::Point(p) => p,
        Point3dRef::Entity(EntityId::Item(i)) => ops::get_world_location(w, i),
        Point3dRef::Entity(EntityId::Mobile(m)) => {
            w.mobile(m).map_or_else(Point3D::default, |x| x.location)
        }
    }
}

fn on_throw_target(w: &mut World, potion: ItemId, from: MobileId, t: Targeted, kind: Kind) {
    let gone = w
        .item(potion)
        .is_none_or(|i| i.deleted() || i.map == Some(MapId::INTERNAL));
    if gone {
        return;
    }
    let Some(map) = w.mobile(from).and_then(|m| m.map) else {
        return;
    };
    let which = u8::from(matches!(kind, Kind::Confusion(_)));
    if !matches!(kind, Kind::Explosion { .. }) {
        add_delay(w, which, from, if which == 0 { 30 } else { 60 });
    }
    let p = helper::get_surface_top(w, Point3dRef::from(t));
    let mut loc = point_of(w, p);
    muo_server::mobile::base::revealing_action(w, from);
    let hue = w.item(potion).map_or(0, |i| i.hue);
    let gid = if matches!(kind, Kind::Explosion { .. }) {
        w.item(potion).map_or(0xF0D, |i| i.item_id)
    } else {
        0xF0D
    };
    let mobile_target = match p {
        Point3dRef::Entity(EntityId::Mobile(m)) => Some(m),
        _ => None,
    };
    // RelativeLocation is false: the explosion location is the mobile's current location.
    if let Some(m) = mobile_target {
        loc = w.mobile(m).map_or(loc, |x| x.location);
    }
    match mobile_target {
        Some(m) if !matches!(kind, Kind::Explosion { .. }) => {
            muo_server::effects::send_moving_effect(w, from, m, gid, 7, 0, false, false, hue, 0);
        }
        _ => muo_server::effects::send_moving_effect_to_point(
            w, from, loc, gid, 7, 0, false, false, hue, 0,
        ),
    }
    match kind {
        Kind::Explosion { .. } => {
            if w.item(potion).is_some_and(|i| i.amount > 1) {
                muo_server::mobile::part2::items::lift_item_dupe(w, potion, 1);
            }
            ops::internalize(w, potion);
            let from_loc = w.mobile(from).map_or(loc, |m| m.location);
            let delay = TimeSpan::from_secs_f64(0.1 * utility::distance_to_sqrt_3d(from_loc, loc));
            // If the potion is about to explode, stop the timer so it doesn't explode on you, while it is mid-air
            let now = w.now;
            let stop = rt(w)
                .detonate
                .get(&potion)
                .is_some_and(|&(_, remaining, next)| remaining <= 1 && next <= now + delay);
            if stop && let Some((mut h, ..)) = rt(w).detonate.remove(&potion) {
                h.cancel(w);
            }
            timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
                reposition(w, potion, from, loc, map)
            });
        }
        Kind::Conflagration(..) => {
            timer::start(
                w,
                TimeSpan::from_secs_f64(1.5),
                TimeSpan::ZERO,
                1,
                move |w| conflagration_explode(w, potion, from, loc, map),
            );
        }
        Kind::Confusion(radius) => {
            timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
                confusion_explode(w, potion, from, loc, map, radius)
            });
        }
        _ => {}
    }
}

/// Cancels the `ThrowTarget`s of the other users of `potion` (`Target.Cancel(user)`).
fn cancel_other_users(w: &mut World, potion: ItemId) {
    let users = std::mem::take(rt(w).users.entry(potion).or_default());
    for u in users {
        if let Some(t) = w.mobile(u).and_then(|m| m.target)
            && rt(w).targets.get(&t) == Some(&potion)
        {
            target::cancel_mobile_target(w, u); // `Target.Cancel(user)` also sends the cancel-target packet
        }
    }
}

// ---- explosion ----

fn start_detonate(
    w: &mut World,
    potion: ItemId,
    from: MobileId,
    delay: f64,
    interval: f64,
    count: i32,
) {
    let mut left = count;
    let next = w.now + TimeSpan::from_secs_f64(delay);
    let h = timer::start(
        w,
        TimeSpan::from_secs_f64(delay),
        TimeSpan::from_secs_f64(interval),
        count,
        move |w| {
            let remaining_before = left;
            left -= 1;
            let next = w.now + TimeSpan::from_secs_f64(interval);
            if let Some(e) = rt(w).detonate.get_mut(&potion) {
                e.1 = left;
                e.2 = next;
            }
            detonate_tick(w, potion, from, remaining_before - 1);
        },
    );
    rt(w).detonate.insert(potion, (h, count, next));
}

/// `FindParent(from)`.
fn find_parent(w: &World, potion: ItemId, from: MobileId) -> Option<EntityId> {
    let i = w.item(potion)?;
    if let Some(h) = i.held_by
        && w.mobile(h).is_some_and(|m| m.holding == Some(potion))
    {
        return Some(EntityId::Mobile(h));
    }
    if let Some(r) = ops::root_parent(w, potion) {
        return Some(r);
    }
    if i.map == Some(MapId::INTERNAL) {
        return Some(EntityId::Mobile(from));
    }
    Some(EntityId::Item(potion))
}

fn detonate_tick(w: &mut World, potion: ItemId, from: MobileId, remaining_count: i32) {
    if w.item(potion).is_none_or(|i| i.deleted()) {
        return;
    }
    let parent = find_parent(w, potion, from);
    if remaining_count > 3 {
        return;
    }
    // Remaining decrements after OnTick so it is off by 1
    if remaining_count <= 0 {
        let at = match parent {
            Some(EntityId::Item(i)) => w.item(i).map(|x| (ops::get_world_location(w, i), x.map)),
            Some(EntityId::Mobile(m)) => w.mobile(m).map(|x| (x.location, x.map)),
            None => None,
        };
        if let Some((loc, map)) = at {
            explode(w, potion, Some(from), true, loc, map);
        }
    } else {
        let text = remaining_count.to_string();
        match parent {
            Some(EntityId::Item(i)) => muo_server::item::messages::public_overhead_message(
                w,
                i,
                MessageType::REGULAR,
                0x22,
                false,
                &text,
            ),
            Some(EntityId::Mobile(m)) => {
                muo_server::mobile::part2::messages::public_overhead_message(
                    w,
                    m,
                    muo_primitives::MessageType::REGULAR,
                    0x22,
                    false,
                    &text,
                    true,
                    muo_server::mobile::AccessLevel::Player,
                )
            }
            None => {}
        }
    }
}

fn reposition(w: &mut World, potion: ItemId, from: MobileId, loc: Point3D, map: MapId) {
    if w.item(potion).is_none_or(|i| i.deleted()) {
        return;
    }
    let running = rt(w).detonate.contains_key(&potion);
    // InstantExplosion is false.
    if !running {
        explode(w, potion, Some(from), true, loc, Some(map));
    } else {
        ops::move_to_world_map(w, potion, loc, Some(map));
    }
}

/// `Explode(from, direct, loc, map)`.
pub fn explode(
    w: &mut World,
    potion: ItemId,
    from: Option<MobileId>,
    direct: bool,
    loc: Point3D,
    map: Option<MapId>,
) {
    if w.item(potion).is_none_or(|i| i.deleted()) {
        return;
    }
    base::consume(w, potion, 1);
    cancel_other_users(w, potion);
    let Some(map) = map else { return };
    muo_server::effects::play_sound(w, loc, Some(map), 0x307);
    muo_server::effects::send_location_effect(w, loc, Some(map), 0x36B0, 9, 10, 0, 0);
    let mut alchemy_bonus = 0;
    if direct && let Some(f) = from {
        let alchemy = muo_server::skills::value(w, f, muo_server::skills::SkillName::Alchemy);
        alchemy_bonus = (alchemy / if muo_content::core::aos(w) { 5.0 } else { 10.0 }) as i32;
    }
    let mut to_hit = Vec::new();
    for m in muo_server::map::enumerate::mobiles_in_range(w, Some(map), loc.x, loc.y, 2)
        .collect::<Vec<_>>()
    {
        let ok = match from {
            None => true,
            Some(f) => {
                helper::valid_indirect_target(w, f, m)
                    && muo_content::mx::can_be_harmful_quiet(w, f, m)
            }
        };
        if ok {
            to_hit.push(m);
        }
    }
    let to_damage = to_hit.len() as i32;
    let (aos_r, pre_r) = match potions::class(w, potion).kind {
        Kind::Explosion { aos, pre } => (aos, pre),
        _ => ((0, 0), (0, 0)),
    };
    let (min0, max0) = if muo_content::core::aos(w) {
        aos_r
    } else {
        pre_r
    };
    let user = from.unwrap_or(MobileId(muo_primitives::Serial::new(0)));
    let (min, max) = if from.is_some() {
        (potions::scale(w, user, min0), potions::scale(w, user, max0))
    } else {
        (min0, max0)
    };
    for m in to_hit {
        if let Some(f) = from {
            muo_content::mx::do_harmful(w, f, m);
        }
        let mut damage = utility::random_min_max(&mut w.rng, min, max) + alchemy_bonus;
        if !muo_content::core::aos(w) && damage > 40 {
            damage = 40;
        } else if muo_content::core::aos(w) && to_damage > 2 {
            damage /= to_damage - 1;
        }
        muo_content::misc::aos_helpers::damage(
            w,
            m,
            from,
            damage,
            DamageSplit {
                phys: 0,
                fire: 100,
                cold: 0,
                pois: 0,
                nrgy: 0,
            },
            DamageOptions::default(),
        );
    }
}

// ---- use delays ----

fn add_delay(w: &mut World, which: u8, m: MobileId, seconds: i64) {
    if let Some((mut h, _)) = rt(w).delay.remove(&(which, m)) {
        h.cancel(w);
    }
    let end = w.now + TimeSpan::from_secs(seconds);
    let h = timer::start(
        w,
        TimeSpan::from_secs(seconds),
        TimeSpan::ZERO,
        1,
        move |w| {
            rt(w).delay.remove(&(which, m));
        },
    );
    rt(w).delay.insert((which, m), (h, end));
}

fn get_delay(w: &mut World, which: u8, m: MobileId) -> i32 {
    let now = w.now;
    match rt(w).delay.get(&(which, m)) {
        Some(&(_, next)) if next > now => (next - now).total_seconds().round_ties_even() as i32,
        _ => 0,
    }
}

// ---- conflagration and confusion blast ----

fn conflagration_explode(w: &mut World, potion: ItemId, from: MobileId, loc: Point3D, map: MapId) {
    if w.item(potion).is_none_or(|i| i.deleted()) {
        return;
    }
    base::consume(w, potion, 1);
    cancel_other_users(w, potion);
    muo_server::effects::play_sound(w, loc, Some(map), 0x20C);
    let Kind::Conflagration(min, max) = potions::class(w, potion).kind else {
        return;
    };
    for i in -2..=2 {
        for j in -2..=2 {
            let p = Point3D::new(loc.x + i, loc.y + j, loc.z);
            if muo_server::map::fit::can_fit(w, map, p.x, p.y, p.z, 12, true, false, true)
                && deps::in_los_point(w, from, p)
            {
                new_fire(w, from, p, map, min, max);
            }
        }
    }
}

fn confusion_explode(
    w: &mut World,
    potion: ItemId,
    from: MobileId,
    loc: Point3D,
    map: MapId,
    radius: i32,
) {
    if w.item(potion).is_none_or(|i| i.deleted()) {
        return;
    }
    base::consume(w, potion, 1);
    cancel_other_users(w, potion);
    muo_server::effects::play_sound(w, loc, Some(map), 0x207);
    circle_effect(w, loc, map, radius, 270, 90);
    timer::start(
        w,
        TimeSpan::from_secs_f64(0.3),
        TimeSpan::ZERO,
        1,
        move |w| circle_effect(w, loc, map, radius, 90, 270),
    );
    for m in muo_server::map::enumerate::mobiles_in_range(w, Some(map), loc.x, loc.y, radius)
        .collect::<Vec<_>>()
    {
        let mon = muo_content::creature::cd(w, m).is_some_and(|c| !c.controlled && !c.summoned);
        if mon {
            let end = w.now + TimeSpan::from_secs(5);
            muo_content::creature::pets::pacify(w, m, end); // TODO check
            let _ = from;
        }
    }
}

/// `Geometry.Circle2D(loc, map, radius, BlastEffect, start, end)`.
fn circle_effect(w: &mut World, loc: Point3D, map: MapId, radius: i32, start: i32, end: i32) {
    let mut points = Vec::new();
    muo_content::misc::small::circle_2d_arc(
        loc,
        map,
        radius,
        &mut |p, _| points.push(p),
        start,
        end,
    );
    for p in points {
        if muo_server::map::fit::can_fit(w, map, p.x, p.y, p.z, 12, true, false, true) {
            muo_server::effects::send_location_effect(w, p, Some(map), 0x376A, 4, 9, 0, 0);
        }
    }
}

pub fn configure(_w: &mut World) {}

// ---- BaseConflagrationPotion.InternalItem: the fire field ----

/// `InternalItem`'s fields: `_from`, `_end`, `_minDamage`, `_maxDamage` (the timer is `FireRt`).
#[derive(Clone, Copy, Debug)]
struct FireData {
    from: Option<MobileId>,
    end: DateTime,
    min_damage: i32,
    max_damage: i32,
}

/// The unsaved `_timer` of each fire.
#[derive(Default)]
struct FireRt {
    timers: HashMap<ItemId, TimerHandle>,
}

fn fire(w: &World, id: ItemId) -> Option<FireData> {
    w.item(id)?.ext.get::<FireData>().copied()
}

/// `GetDamage()`.
fn fire_damage(w: &mut World, id: ItemId) -> i32 {
    let Some(d) = fire(w, id) else { return 0 };
    utility::random_min_max(&mut w.rng, d.min_damage, d.max_damage)
}

/// `InternalItem(from, loc, map, min, max)`.
fn new_fire(w: &mut World, from: MobileId, loc: Point3D, map: MapId, min: i32, max: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &FIRE_FIELD, 0x398C);
    ops::set_movable(w, id, false);
    ops::set_light(w, id, muo_server::light_type::LightType::Circle300);
    ops::move_to_world_map(w, id, loc, Some(map));
    let end = w.now + TimeSpan::from_secs(10);
    // SetDamage(min, max): the alchemy bonus of the thrower.
    let alchemy = muo_server::skills::fixed(w, from, muo_server::skills::SkillName::Alchemy);
    let bonus = alchemy / 125 + alchemy / 250;
    let (min_damage, max_damage) = (
        potions::scale(w, from, min + bonus),
        potions::scale(w, from, max + bonus),
    );
    if let Some(i) = w.item_mut(id) {
        i.ext = muo_server::ext::Ext::new(FireData {
            from: Some(from),
            end,
            min_damage,
            max_damage,
        });
    }
    start_fire_timer(w, id);
    id
}

/// `_timer = new InternalTimer(this, _end); _timer.Start()`.
fn start_fire_timer(w: &mut World, id: ItemId) {
    let h = timer::start(w, TimeSpan::ZERO, TimeSpan::from_secs(1), 0, move |w| {
        fire_tick(w, id)
    });
    w.resources.get_mut::<FireRt>().timers.insert(id, h);
}

fn fire_tick(w: &mut World, id: ItemId) {
    if w.item(id).is_none_or(|i| i.deleted()) {
        return;
    }
    let Some(d) = fire(w, id) else { return };
    if w.now > d.end {
        base::delete(w, id);
        if let Some(mut h) = w.resources.get_mut::<FireRt>().timers.remove(&id) {
            h.cancel(w);
        }
        return;
    }
    let (map, loc) = match w.item(id) {
        Some(i) => (i.map, i.location),
        None => return,
    };
    let Some(from) = d.from.filter(|_| map.is_some()) else {
        return;
    };
    let aos = muo_content::core::aos(w);
    let at: Vec<MobileId> = muo_server::map::enumerate::mobiles_at(w, map, loc.x, loc.y).collect();
    let hit: Vec<MobileId> = at
        .into_iter()
        .filter(|&m| {
            let z = w.mobile(m).map_or(0, |x| x.location.z);
            z + 16 > loc.z
                && loc.z + 12 > z
                && (!aos || m != from)
                && muo_content::spells::helper::valid_indirect_target(w, from, m)
                && muo_content::mx::can_be_harmful_quiet(w, from, m)
        })
        .collect();
    for m in hit {
        muo_content::mx::do_harmful(w, from, m);
        let dmg = fire_damage(w, id);
        muo_content::misc::aos_helpers::damage(
            w,
            m,
            Some(from),
            dmg,
            DamageSplit {
                fire: 100,
                ..Default::default()
            },
            DamageOptions::default(),
        );
        muo_server::mobile::part2::effects::play_sound(w, m, 0x208);
    }
}

/// `Server.Items.BaseConflagrationPotion+InternalItem`; `[SerializationGenerator(0, false)]`.
pub static FIRE_FIELD: muo_server::item::ItemType = muo_server::item::ItemType {
    full_name: "Server.Items.BaseConflagrationPotion+InternalItem",
    parent: Some(&muo_server::item::ITEM_BASE),
    construct: None,
    blocks_fit: |_, _| true,
    on_after_delete: |w, id| {
        base::on_after_delete_base(w, id);
        if let Some(mut h) = w.resources.get_mut::<FireRt>().timers.remove(&id) {
            h.cancel(w);
        }
    },
    on_move_over: |w, id, m| {
        let Some(d) = fire(w, id) else { return true };
        let aos = muo_content::core::aos(w);
        let visible = w.item(id).is_some_and(|i| i.visible());
        if let Some(from) = d.from
            && visible
            && (!aos || m != from)
            && muo_content::spells::helper::valid_indirect_target(w, from, m)
            && muo_content::mx::can_be_harmful_quiet(w, from, m)
        {
            muo_content::mx::do_harmful(w, from, m);
            let dmg = fire_damage(w, id);
            muo_content::misc::aos_helpers::damage(
                w,
                m,
                Some(from),
                dmg,
                DamageSplit {
                    fire: 100,
                    ..Default::default()
                },
                DamageOptions::default(),
            );
            muo_server::mobile::part2::effects::play_sound(w, m, 0x208);
        }
        true
    },
    serialize: |w, id, out| {
        use muo_server::serialization_ext::WriteEntityExt;
        (muo_server::item::ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let d = fire(w, id).unwrap_or(FireData {
            from: None,
            end: DateTime::MIN,
            min_damage: 0,
            max_damage: 0,
        });
        out.write_mobile(w, d.from);
        out.write_date_time_ticks(d.end.ticks());
        out.write_i32(d.min_damage);
        out.write_i32(d.max_damage);
    },
    deserialize: |w, id, r| {
        use muo_server::serialization_ext::ReadEntityExt;
        (muo_server::item::ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let from = r.read_mobile(w)?;
        let end = DateTime::from_ticks(r.read_date_time_ticks()?);
        let (min_damage, max_damage) = (r.read_i32()?, r.read_i32()?);
        if let Some(i) = w.item_mut(id) {
            i.ext = muo_server::ext::Ext::new(FireData {
                from,
                end,
                min_damage,
                max_damage,
            });
        }
        start_fire_timer(w, id); // [AfterDeserialization]
        Ok(())
    },
    ..muo_server::item::ITEM_BASE
};

/// The `InternalItem` class of the conflagration potions.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    types.register_item(&FIRE_FIELD, &[]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;
    use muo_server::net_state::{NetOut, NetStateData, NetStateId};

    /// `Target.Cancel(user)` (the static one) sends the cancel-target packet before it cancels the target.
    #[test]
    fn thrown_potion_blast_cancel_tells_the_client_to_drop_the_cursor() {
        let mut w = muo_server::map::test_support::world();
        muo_content::register(&mut w.types);
        muo_items_register(&mut w);
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::new(0));
        let user = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(user).unwrap().net_state = Some(ns);
        let potion = super::super::potions::new_confusion_blast(&mut w);
        drink(&mut w, potion, user);
        assert!(target_potion(&mut w, user) == Some(potion));
        w.net_out.clear();
        cancel_other_users(&mut w, potion);
        assert!(w.mobile(user).unwrap().target.is_none());
        let want = muo_server::packets::target::create_cancel_target();
        assert!(
            w.net_out
                .iter()
                .any(|o| matches!(o, NetOut::Send(n, b) if *n == ns && *b == want)),
            "the client is told to drop its cursor"
        );
    }

    fn muo_items_register(w: &mut World) {
        crate::register(&mut w.types);
    }
}
