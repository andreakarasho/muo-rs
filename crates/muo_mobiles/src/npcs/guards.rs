//! Guards: `BaseGuard` (a plain `Mobile` with a focus and three timers), `WarriorGuard` and
//! `ArcherGuard`. The region hooks that call `BaseGuard.Spawn` and `GuardType.CreateInstance` are set in
//! [`initialize`].

use muo_content::hooks::rnd;
use muo_content::mx;
use muo_primitives::{Direction, Point3D, Serial, TimeSpan};
use muo_server::effects::send_location_particles;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::base as item_base;
use muo_server::item::effect_item::{self, DEFAULT_DURATION};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::part1::{lifecycle, props_b};
use muo_server::mobile::part2::{items, messages};
use muo_server::mobile::{MOBILE_BASE, MobileType};
use muo_server::skills::SkillName;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};
use muo_server::world::World;

/// `BaseGuard.GuardsInstantKill` (`guards.instantKill`).
#[derive(Default)]
pub struct GuardStatics {
    pub instant_kill: bool,
}

/// What a guard class adds to a `Mobile`: `_focus` and the timers.
#[derive(Default)]
pub struct GuardData {
    pub focus: Option<MobileId>,
    pub idle_timer: TimerHandle,
    pub attack_timer: TimerHandle,
    /// `GuardIdleTimer.m_Stage`.
    pub idle_stage: i32,
    /// ArcherGuard `_shooting`.
    pub shooting: bool,
}

/// The virtuals of `BaseGuard`.
pub struct GuardClass {
    /// `NonLethalAttack(target)`.
    pub non_lethal_attack: fn(&mut World, MobileId, MobileId),
}

pub fn data(w: &World, id: MobileId) -> Option<&GuardData> {
    w.mobile(id)?.ext.get::<GuardData>()
}

fn data_mut(w: &mut World, id: MobileId) -> Option<&mut GuardData> {
    w.mobile_mut(id)?.ext.get_mut::<GuardData>()
}

fn deleted(w: &World, id: MobileId) -> bool {
    mx::deleted(w, id)
}

/// `m is BaseGuard`.
pub fn is_guard(w: &World, id: MobileId) -> bool {
    data(w, id).is_some()
}

/// `BaseGuard.Configure`.
pub fn configure(w: &mut World) {
    let v = w.config.get_setting("guards.instantKill", true);
    w.resources.get_mut::<GuardStatics>().instant_kill = v;
}

/// `Focus` getter.
pub fn focus(w: &World, id: MobileId) -> Option<MobileId> {
    data(w, id)?.focus
}

fn delete_mobile(w: &mut World, id: MobileId) {
    if let Some(t) = w.mobile(id).map(|m| m.ty) {
        (t.delete)(w, id);
    }
}

fn particles(w: &mut World, loc: Point3D, map: Option<MapId>, effect: i32) {
    let e = effect_item::create(w, loc, map, DEFAULT_DURATION);
    send_location_particles(w, e, 0x3728, 10, 10, effect);
}

/// `BaseGuard.TeleportTo(source, to)`.
pub fn teleport_to(w: &mut World, source: MobileId, to: Point3D) {
    let (loc, map) = w
        .mobile(source)
        .map_or_else(Default::default, |m| (m.location, m.map));
    particles(w, loc, map, 2023);
    lifecycle::set_location(w, source, to);
    particles(w, to, map, 5023);
    mx::play_sound(w, source, 0x1FE);
}

// ---- timers ----

/// `AttackTimer = value`: stops the old one and starts the new one.
fn set_attack_timer(w: &mut World, id: MobileId, start: bool) {
    let mut t = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.attack_timer))
        .unwrap_or_default();
    t.cancel(w);
    if start {
        let h = timer::start(
            w,
            TimeSpan::from_millis(250),
            TimeSpan::from_millis(100),
            0,
            move |w| {
                attack_tick(w, id);
            },
        );
        if let Some(d) = data_mut(w, id) {
            d.attack_timer = h;
        }
    }
}

/// `IdleTimer = value`.
fn set_idle_timer(w: &mut World, id: MobileId, start: bool) {
    let mut t = data_mut(w, id)
        .map(|d| {
            d.idle_stage = 0;
            std::mem::take(&mut d.idle_timer)
        })
        .unwrap_or_default();
    t.cancel(w);
    if start {
        let h = timer::start(
            w,
            TimeSpan::from_secs(2),
            TimeSpan::from_millis(2500),
            0,
            move |w| {
                idle_tick(w, id);
            },
        );
        if let Some(d) = data_mut(w, id) {
            d.idle_timer = h;
        }
    }
}

fn stop_attack(w: &mut World, id: MobileId) {
    let mut t = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.attack_timer))
        .unwrap_or_default();
    t.cancel(w);
}

fn stop_idle(w: &mut World, id: MobileId) {
    let mut t = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.idle_timer))
        .unwrap_or_default();
    t.cancel(w);
}

/// `Stop()` from inside `GuardIdleTimer.OnTick`: the `IdleTimer` field keeps its (stopped) timer, so it isn't null.
fn halt_idle(w: &mut World, id: MobileId) {
    if let Some(h) = data(w, id).map(|d| d.idle_timer) {
        timer::stop(w, h);
    }
}

/// `GuardIdleTimer.OnTick`.
fn idle_tick(w: &mut World, id: MobileId) {
    if deleted(w, id) {
        halt_idle(w, id);
        return;
    }
    let stage = data(w, id).map_or(0, |d| d.idle_stage);
    if let Some(d) = data_mut(w, id) {
        d.idle_stage += 1;
    }
    let dir = w.mobile(id).map_or(Direction::NORTH, |m| m.direction);
    if stage % 4 == 0 || !muo_server::mobile::base::move_dir(w, id, dir) {
        let d = Direction::from_bits_retain(rnd::below(w, 8) as u8);
        muo_server::mobile::part1::props_b::set_direction(w, id, d);
    }
    if stage + 1 > 16 {
        let (loc, map) = w
            .mobile(id)
            .map_or_else(Default::default, |m| (m.location, m.map));
        particles(w, loc, map, 2023);
        mx::play_sound(w, id, 0x1FE);
        let spawner = w.mobile(id).and_then(|m| m.spawner);
        match spawner {
            None => delete_mobile(w, id),
            Some(s) => {
                let at = w
                    .find_item(Serial::new(s.0))
                    .and_then(|i| w.item(i))
                    .map(|i| i.location);
                match at {
                    Some(p) => teleport_to(w, id, p),
                    None => teleport_to(w, id, loc),
                }
            }
        }
        halt_idle(w, id);
    }
}

/// `GuardAttackTimer.OnTick`.
fn attack_tick(w: &mut World, id: MobileId) {
    if deleted(w, id) {
        stop_attack(w, id);
        return;
    }
    muo_server::mobile::base::set_criminal(w, id, false);
    props_b::set_kills(w, id, 0);
    let stam_max = w.mobile(id).map_or(0, |m| (m.ty.stam_max)(w, id));
    props_b::set_stam(w, id, stam_max);

    let target = focus(w, id);

    if let Some(t) = target {
        let bad = deleted(w, t) || !mx::alive(w, t) || !mx::can_be_harmful(w, id, t);
        if bad {
            set_focus(w, id, None);
            stop_attack(w, id);
            return;
        }
    }
    if let Some(t) = target
        && mx::combatant(w, id) != Some(t)
    {
        mx::set_combatant(w, id, Some(t));
    }

    match target {
        None => stop_attack(w, id),
        Some(t) => {
            let instant = w
                .resources
                .get::<GuardStatics>()
                .is_some_and(|s| s.instant_kill);
            if instant {
                let loc = mx::location(w, t);
                teleport_to(w, id, loc);
                muo_server::mobile::part2::effects::bolt_effect(w, t, 0);
                if muo_content::creature::is_creature(w, t) {
                    muo_content::creature::edit(w, t, |d| d.no_kill_awards = true);
                }
                let max = mx::hits_max(w, t);
                mx::damage(w, t, max, Some(id));
                muo_server::mobile::base::kill(w, t); // just in case, maybe Damage is overridden on some shard
                let corpse = w.mobile(t).and_then(|m| m.corpse);
                if let Some(c) = corpse
                    && !w.mobile(t).is_some_and(|m| m.player)
                    && let Some(ty) = w.item(c).map(|i| i.ty)
                {
                    (ty.delete)(w, c);
                }
                set_focus(w, id, None);
                stop_attack(w, id);
            } else if let Some(c) = class(w, id) {
                (c.non_lethal_attack)(w, id, t);
            }
        }
    }
}

fn class(w: &World, id: MobileId) -> Option<&'static GuardClass> {
    w.mobile(id)?.ty.family::<GuardClass>()
}

/// `new GuardAvengeTimer(focus).Start()`: three more guards, 2.5 s in, then every second.
fn start_avenge_timer(w: &mut World, focus: MobileId) {
    timer::start(
        w,
        TimeSpan::from_millis(2500),
        TimeSpan::from_secs(1),
        3,
        move |w| {
            spawn_for(w, focus, focus, 1, true);
        },
    );
}

// ---- Focus ----

/// `Focus` setter (WarriorGuard and ArcherGuard have the same).
pub fn set_focus(w: &mut World, id: MobileId, value: Option<MobileId>) {
    if deleted(w, id) {
        return;
    }
    let old = focus(w, id);
    if old != value {
        if let Some(d) = data_mut(w, id) {
            d.focus = value;
        }
        if let Some(v) = value {
            muo_server::mobile::base::aggressive_action(w, id, v);
        }
        mx::set_combatant(w, id, value);

        if old.is_some_and(|o| !mx::alive(w, o)) {
            messages::say_text(w, id, "Thou hast suffered thy punishment, scoundrel.");
        }
        if value.is_some() {
            messages::say_number(w, id, 500131, ""); // Thou wilt regret thine actions, swine!
        }

        set_attack_timer(w, id, false);
        set_idle_timer(w, id, false);

        if value.is_some() {
            set_attack_timer(w, id, true);
        } else {
            set_idle_timer(w, id, true);
        }
    } else if value.is_none()
        && data(w, id).is_some_and(|d| d.idle_timer == TimerHandle::default())
        && w.mobile(id).is_some_and(|m| m.spawner.is_some())
    {
        set_idle_timer(w, id, true);
    }
}

// ---- spawning ----

/// `BaseGuard.Spawn(region, target, amount, onlyAdditional)`.
pub fn spawn_for(
    w: &mut World,
    caller: MobileId,
    target: MobileId,
    amount: i32,
    only_additional: bool,
) {
    let region = w.mobile(caller).map(|m| m.region);
    if let Some(r) = region {
        spawn(w, r, target, amount, only_additional);
    }
}

pub fn spawn(
    w: &mut World,
    region: muo_server::region::RegionId,
    target: MobileId,
    mut amount: i32,
    only_additional: bool,
) {
    if deleted(w, target) {
        return;
    }
    for g in mx::mobiles_in_range(w, target, 15) {
        if !is_guard(w, g) {
            continue;
        }
        let f = focus(w, g);
        if f.is_none() {
            // idling
            set_focus(w, g, Some(target));
            amount -= 1;
        } else if f == Some(target) && !only_additional {
            amount -= 1;
        }
    }
    while amount > 0 {
        amount -= 1;
        if let Some(t) = w.region(region).map(|r| r.ty) {
            (t.make_guard)(w, region, target);
        }
    }
}

/// `GuardedRegion.GuardType.CreateInstance(focus)`.
fn make_guard(w: &mut World, guard_type: &str, focus: MobileId) {
    match guard_type {
        "WarriorGuard" | "Server.Mobiles.WarriorGuard" => {
            warrior_guard_with(w, Some(focus));
        }
        "ArcherGuard" | "Server.Mobiles.ArcherGuard" => {
            archer_guard_with(w, Some(focus));
        }
        _ => {}
    }
}

/// Phase `Initialize`: the region system's hooks.
pub fn initialize(w: &mut World) {
    use muo_content::regions::base_region::RegionHooks;
    let h = w.resources.get_mut::<RegionHooks>();
    h.guard_spawn_from_mobile = Some(|w, caller, target| spawn_for(w, caller, target, 1, false));
    h.guard_spawn_from_region = Some(|w, region, target| spawn(w, region, target, 1, false));
    h.make_guard = Some(make_guard);
}

// ---- BaseGuard ----

/// `new T(target)`: `BaseGuard(target)` (the location of the target and the first focus), then the class's template
/// (the title, body, outfit and skills).
fn new_guard(w: &mut World, ty: &'static MobileType, target: Option<MobileId>) -> MobileId {
    let id = w.new_mobile(ty);
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(GuardData::default());
    }
    if let Some(t) = target {
        let (loc, map) = w
            .mobile(t)
            .map_or_else(Default::default, |m| (m.location, m.map));
        lifecycle::set_location(w, id, loc);
        lifecycle::set_map(w, id, map);
        particles(w, loc, map, 5023);
        set_focus(w, id, Some(t));
    }
    muo_content::templates::apply_mobile(w, id);
    id
}

fn on_before_death(w: &mut World, id: MobileId) -> bool {
    // Warrior and Archer: if a guard dies, three more guards will spawn.
    if let Some(f) = focus(w, id)
        && mx::alive(w, f)
    {
        start_avenge_timer(w, f);
    }
    // BaseGuard.OnBeforeDeath
    let (loc, map) = w
        .mobile(id)
        .map_or_else(Default::default, |m| (m.location, m.map));
    particles(w, loc, map, 2023);
    mx::play_sound(w, id, 0x1FE);
    delete_mobile(w, id);
    false
}

/// `OnDragDrop(from, dropped)`: a player's head is turned in for its bounty.
fn on_drag_drop(w: &mut World, id: MobileId, from: MobileId, dropped: ItemId) -> bool {
    use muo_items::body_parts::parts::HeadData;
    let hooks = super::special::hooks::get(w);
    let enabled = hooks.and_then(|h| h.bounties_enabled).is_some_and(|f| f(w));
    let head = w
        .item(dropped)
        .and_then(|i| i.ext.get::<HeadData>())
        .filter(|h| h.player_name.is_some())
        .cloned();
    if enabled && let Some(head) = head {
        let target = head.bounty_target;
        if target.is_none() || w.now - head.carved_time > TimeSpan::from_hours(24) {
            say_non_bounty_head_response(w, id);
            item_base::delete(w, dropped);
            return true;
        }
        messages::say_number(w, id, 500670, ""); // Ah, a head!  Let me check to see if there is a bounty on this.
        item_base::delete(w, dropped);
        if let Some(t) = target {
            claim_bounty(w, id, from, t);
        }
        return true;
    }
    (MOBILE_BASE.on_drag_drop)(w, id, from, dropped)
}

/// `ClaimBounty(from, target)`.
fn claim_bounty(w: &mut World, id: MobileId, from: MobileId, target: MobileId) {
    let bounty = super::special::hooks::get(w)
        .and_then(|h| h.get_bounty)
        .map_or(0, |f| f(w, target));
    let name = w
        .mobile(target)
        .and_then(|m| (m.ty.name)(w, target))
        .unwrap_or_default();
    if bounty > 0 {
        if let Some(f) = super::special::hooks::get(w).and_then(|h| h.clear_bounty) {
            f(w, target);
        }
        muo_content::vendor::banker::deposit(w, from, bounty, true);
        muo_content::misc::titles::award_karma(w, from, 20, true);
        // The bounty on ~1_PLAYER_NAME~ was ~2_AMOUNT~ gold, and has been credited to your account.
        messages::say_number(w, id, 1042855, &format!("{name}	{bounty}"));
    } else {
        messages::say_number(w, id, 1042854, &name); // There was no bounty on ~1_PLAYER_NAME~.
    }
}

/// `SayNonBountyHeadResponse()`.
fn say_non_bounty_head_response(w: &mut World, id: MobileId) {
    let n = if rnd::below(w, 5) == 0 {
        500661 + rnd::below(w, 9) // 500661-500669: silly guard responses
    } else {
        500654 + rnd::below(w, 7) // 500654-500660: normal guard responses
    };
    messages::say_number(w, id, n, "");
}

fn on_after_delete(w: &mut World, id: MobileId) {
    stop_attack(w, id);
    stop_idle(w, id);
    (MOBILE_BASE.on_after_delete)(w, id);
}

/// `[AfterDeserialization]`.
fn after_deserialization(w: &mut World, id: MobileId) {
    if focus(w, id).is_some() {
        set_attack_timer(w, id, true);
    } else {
        set_idle_timer(w, id, true);
    }
}

fn base_serialize(w: &World, id: MobileId, out: &mut dyn muo_io::GenericWriter) {
    (MOBILE_BASE.serialize)(w, id, out);
    out.write_encoded_int(0);
}

fn guard_serialize(w: &World, id: MobileId, out: &mut dyn muo_io::GenericWriter) {
    base_serialize(w, id, out);
    out.write_i32(0);
    use muo_server::serialization_ext::WriteEntityExt;
    out.write_mobile(w, focus(w, id));
}

fn guard_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn muo_io::GenericReader,
) -> muo_io::Result<()> {
    use muo_server::serialization_ext::ReadEntityExt;
    (MOBILE_BASE.deserialize)(w, id, r)?;
    let _base_version = r.read_encoded_int()?;
    let _version = r.read_i32()?;
    let f = r.read_mobile(w)?;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(GuardData {
            focus: f,
            ..GuardData::default()
        });
    }
    after_deserialization(w, id);
    Ok(())
}

// ---- construction shared by both classes ----

// ---- WarriorGuard ----

fn warrior_guard_new(w: &mut World) -> MobileId {
    warrior_guard_with(w, None)
}

/// `new WarriorGuard(target)`.
pub fn warrior_guard_with(w: &mut World, target: Option<MobileId>) -> MobileId {
    new_guard(w, &WARRIOR_GUARD, target)
}

fn warrior_non_lethal_attack(w: &mut World, id: MobileId, target: MobileId) {
    if !mx::in_range(w, id, target, 20) {
        set_focus(w, id, None);
    } else if !mx::in_range(w, id, target, 10) || !mx::in_los(w, id, target) {
        let l = mx::location(w, target);
        teleport_to(w, id, l);
    } else if !mx::in_range(w, id, target, 1) {
        let d = mx::get_direction_to(w, id, target) | Direction::RUNNING;
        if !muo_server::mobile::base::move_dir(w, id, d) {
            let l = mx::location(w, target);
            teleport_to(w, id, l);
        }
    } else if !can_see(w, id, target)
        && muo_server::mobile::base::use_skill(w, id, SkillName::DetectHidden)
    {
        messages::say_text(w, id, "Reveal!");
    }
}

fn can_see(w: &mut World, id: MobileId, target: MobileId) -> bool {
    muo_server::mobile::base::can_see_mobile(w, id, target)
}

static WARRIOR_GUARD_CLASS: GuardClass = GuardClass {
    non_lethal_attack: warrior_non_lethal_attack,
};

/// `Server.Mobiles.WarriorGuard`.
pub static WARRIOR_GUARD: MobileType = MobileType {
    full_name: "Server.Mobiles.WarriorGuard",
    parent: Some(&MOBILE_BASE),
    construct: Some(warrior_guard_new),
    family: Some(&WARRIOR_GUARD_CLASS),
    on_before_death,
    on_after_delete,
    on_drag_drop,
    serialize: guard_serialize,
    deserialize: guard_deserialize,
    ..MOBILE_BASE
};

// ---- ArcherGuard ----

fn archer_guard_new(w: &mut World) -> MobileId {
    archer_guard_with(w, None)
}

/// `new ArcherGuard(target)`.
pub fn archer_guard_with(w: &mut World, target: Option<MobileId>) -> MobileId {
    new_guard(w, &ARCHER_GUARD, target)
}

fn weapon_max_range(w: &mut World, id: MobileId) -> i32 {
    if let Some(f) = muo_content::hooks::hooks(w).and_then(|h| h.weapon_max_range) {
        return f(w, id);
    }
    for layer in [Layer::TwoHanded, Layer::OneHanded] {
        if let Some(it) = items::find_item_on_layer(w, id, layer)
            && w.item(it)
                .is_some_and(|i| i.ty.is(&muo_content::items::weapon::BASE_WEAPON))
        {
            return muo_content::items::weapon::max_range(w, it);
        }
    }
    1
}

fn set_shooting(w: &mut World, id: MobileId, v: bool) {
    if let Some(d) = data_mut(w, id) {
        d.shooting = v;
    }
}

fn archer_non_lethal_attack(w: &mut World, id: MobileId, target: MobileId) {
    let out_of_max = |w: &mut World| {
        let r = weapon_max_range(w, id);
        !mx::in_range(w, id, target, r)
    };
    if !mx::in_range(w, id, target, 20) {
        set_shooting(w, id, false);
        set_focus(w, id, None);
        return;
    }
    if !mx::in_los(w, id, target) {
        set_shooting(w, id, false);
        let l = mx::location(w, target);
        teleport_to(w, id, l);
        return;
    }
    if !can_see(w, id, target) {
        set_shooting(w, id, false);
        if mx::in_range(w, id, target, 2) {
            if muo_server::mobile::base::use_skill(w, id, SkillName::DetectHidden) {
                messages::say_text(w, id, "Reveal!");
            }
        } else {
            let d = mx::get_direction_to(w, id, target) | Direction::RUNNING;
            if !muo_server::mobile::base::move_dir(w, id, d) && out_of_max(w) {
                let l = mx::location(w, target);
                teleport_to(w, id, l);
            }
        }
        return;
    }
    if data(w, id).is_some_and(|d| d.shooting) {
        let spare = w
            .mobile(id)
            .is_some_and(|m| m.next_combat_time.wrapping_sub(w.tick_count) > 1000);
        if spare || out_of_max(w) {
            set_shooting(w, id, false);
        }
        return;
    }
    if mx::in_range(w, id, target, 1) {
        // Move(GetDirectionTo(target) - 4 | Direction.Running)
        let back =
            Direction::from_bits_retain(mx::get_direction_to(w, id, target).bits().wrapping_sub(4))
                | Direction::RUNNING;
        if !muo_server::mobile::base::move_dir(w, id, back) && out_of_max(w) {
            let l = mx::location(w, target);
            teleport_to(w, id, l);
            return;
        }
    }
    if mx::in_range(w, id, target, 4) {
        set_shooting(w, id, true);
    }
}

static ARCHER_GUARD_CLASS: GuardClass = GuardClass {
    non_lethal_attack: archer_non_lethal_attack,
};

/// `Server.Mobiles.ArcherGuard`.
pub static ARCHER_GUARD: MobileType = MobileType {
    full_name: "Server.Mobiles.ArcherGuard",
    parent: Some(&MOBILE_BASE),
    construct: Some(archer_guard_new),
    family: Some(&ARCHER_GUARD_CLASS),
    on_before_death,
    on_after_delete,
    on_drag_drop,
    serialize: guard_serialize,
    deserialize: guard_deserialize,
    ..MOBILE_BASE
};

pub(crate) fn register(types: &mut TypeRegistry) {
    types.register_mobile(&WARRIOR_GUARD, &[]);
    types.register_mobile(&ARCHER_GUARD, &[]);
    types.register_phase(
        Phase::Configure,
        "BaseGuard.Configure",
        DEFAULT_CALL_PRIORITY,
        configure,
    );
    types.register_phase(
        Phase::Initialize,
        "BaseGuard.Initialize",
        DEFAULT_CALL_PRIORITY,
        initialize,
    );
}

#[cfg(test)]
pub(crate) fn all() -> Vec<&'static MobileType> {
    vec![&WARRIOR_GUARD, &ARCHER_GUARD]
}

#[allow(dead_code)]
fn _unused(_: EntityId, _: &dyn Fn()) {
    let _ = item_base::set_hue;
}

#[cfg(test)]
pub(crate) fn data_mut_for_test(w: &mut World, id: MobileId) -> &mut GuardData {
    data_mut(w, id).unwrap()
}
