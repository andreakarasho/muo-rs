//! The monster ability singletons. An ability class is a `static MonsterAbility`
//! whose fn pointers default to the base behavior (`..BASE`); `base.Trigger(..)` is [`base_trigger`]. Per-creature
//! state (cooldowns) is in `CreatureData`; per-ability state (`MagicalBarrier._inactiveField`, the DoT tables, the
//! group's scratch list) is in [`AbilityRuntime`] in `w.resources`, keyed by the ability's address.

use indexmap::{IndexMap, IndexSet};
use muo_primitives::WeightedValue;
use muo_primitives::utility;
use muo_primitives::{Direction, Point3D, TimeSpan};
use muo_server::ids::MobileId;
use muo_server::mobile::part1::props_a;
use muo_server::mobile::{ResistanceType, StatType};
use muo_server::mods::{ResistanceMod, StatMod};
use muo_server::timer::{self, TimerHandle};
use muo_server::world::World;

use crate::creature::data::{cd, cd_mut, read};
use crate::creature::enums::*;
use crate::creature::vtable::ctype;
use crate::hooks::{AosDamage, hooks, rnd};
use crate::mx;

/// `MonsterAbilityTrigger : ulong`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct MonsterAbilityTrigger(pub u64);

impl MonsterAbilityTrigger {
    pub const NONE: Self = Self(0);
    pub const THINK: Self = Self(1);
    pub const TAKE_MELEE_DAMAGE: Self = Self(2);
    pub const GIVE_MELEE_DAMAGE: Self = Self(4);
    pub const TAKE_SPELL_DAMAGE: Self = Self(8);
    pub const GIVE_SPELL_DAMAGE: Self = Self(0x10);
    pub const COMBAT_ACTION: Self = Self(0x20);
    pub const DEATH: Self = Self(0x40);
    pub const MOVEMENT: Self = Self(0x80);
    pub const GIVE_DAMAGE: Self = Self(4 | 0x10);
    pub const TAKE_DAMAGE: Self = Self(2 | 8);

    pub const fn or(self, o: Self) -> Self {
        Self(self.0 | o.0)
    }
    pub const fn intersects(self, o: Self) -> bool {
        self.0 & o.0 != 0
    }
    pub const fn without(self, o: Self) -> Self {
        Self(self.0 & !o.0)
    }
}

/// `MonsterAbilityType`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MonsterAbilityType {
    Generic,
    FireBreath,
    GraspingClaw,
    SummonCounter,
    Stun,
    Poison,
    MassPoison,
    ThrowWeapon,
    DestroyEquipment,
    DeathExplosion,
    DrainLife,
    MassDrainLife,
    MagicalBarrier,
    EnergyBoltCounter,
    ReflectPhysicalDamage,
    FanningFire,
    RuneCorruption,
    FanThrow,
    BloodBath,
    TailSwipe,
}

pub type Weighted = WeightedValue<&'static MonsterAbility>;

/// `Array.Empty<MonsterAbility>()`.
pub const NONE: &[&MonsterAbility] = &[];

type Src = MobileId;
type Tgt = Option<MobileId>;

/// One ability class. Virtual members are fn pointers; `data` holds the numeric `virtual` properties.
#[allow(clippy::type_complexity)]
pub struct MonsterAbility {
    pub name: &'static str,
    pub ability_type: MonsterAbilityType,
    pub trigger: MonsterAbilityTrigger,
    pub chance: f64,
    pub min_cooldown: TimeSpan,
    pub max_cooldown: TimeSpan,
    /// `MonsterAbilityGroup`'s members; empty for any other ability.
    pub group: &'static [Weighted],
    /// `AreaEffectMonsterAbility.AreaRange`.
    pub area_range: i32,
    /// `MonsterAbilitySingleTargetDoT.MinDelay` / `MaxDelay`.
    pub min_delay: TimeSpan,
    pub max_delay: TimeSpan,
    pub can_trigger: fn(&mut World, &'static MonsterAbility, Src, MonsterAbilityTrigger) -> bool,
    pub do_trigger: fn(&mut World, &'static MonsterAbility, MonsterAbilityTrigger, Src, Tgt),
    pub alter_melee_from: fn(&mut World, &'static MonsterAbility, Src, MobileId, &mut i32),
    pub alter_melee_to: fn(&mut World, &'static MonsterAbility, Src, MobileId, &mut i32),
    pub alter_spell_from: fn(&mut World, &'static MonsterAbility, Src, MobileId, &mut i32),
    pub alter_spell_to: fn(&mut World, &'static MonsterAbility, Src, MobileId, &mut i32),
    pub alter_scalar_from: fn(&mut World, &'static MonsterAbility, Src, MobileId, &mut f64),
    pub alter_scalar_to: fn(&mut World, &'static MonsterAbility, Src, MobileId, &mut f64),
    pub do_move: fn(&mut World, &'static MonsterAbility, Src, Direction),
    // Single-target family.
    pub can_effect_target:
        fn(&mut World, &'static MonsterAbility, MonsterAbilityTrigger, Src, MobileId) -> bool,
    pub on_before_target:
        fn(&mut World, &'static MonsterAbility, MonsterAbilityTrigger, Src, MobileId),
    pub on_target: fn(&mut World, &'static MonsterAbility, MonsterAbilityTrigger, Src, MobileId),
    // Area family.
    pub do_effect_target: fn(&mut World, &'static MonsterAbility, Src, MobileId),
    // DoT family.
    pub get_count: fn(&World, &'static MonsterAbility, Src, MobileId) -> i32,
    pub effect_tick: fn(&mut World, &'static MonsterAbility, Src, MobileId, &mut TimeSpan),
    pub end_effect: fn(&mut World, &'static MonsterAbility, Src, MobileId),
    pub on_effect_expired: fn(&mut World, &'static MonsterAbility, Src, MobileId),
}

impl MonsterAbility {
    fn addr(&'static self) -> usize {
        self as *const MonsterAbility as usize
    }
}

pub fn same(a: &'static MonsterAbility, b: &'static MonsterAbility) -> bool {
    std::ptr::eq(a, b)
}

/// State kept per ability rather than per creature.
#[derive(Default)]
pub struct AbilityRuntime {
    /// `MonsterAbilityGroup._availableToTrigger`: indexes into `group`.
    group_available: IndexMap<usize, Vec<usize>>,
    /// DoT `_table`: (ability, defender) -> timer.
    dot: IndexMap<(usize, MobileId), TimerHandle>,
    /// `StunAttack._stunning`.
    stunning: IndexSet<MobileId>,
    /// `DeathExplosion._chainDepth`.
    chain_depth: i32,
    /// `MagicalBarrier._inactiveField`.
    inactive_field: IndexMap<usize, IndexSet<MobileId>>,
}

fn rt(w: &mut World) -> &mut AbilityRuntime {
    w.resources.get_mut::<AbilityRuntime>()
}

// ---- base behavior ------------------------------------------------------------------------------------

/// `MonsterAbility.CanTrigger`.
pub fn base_can_trigger(
    w: &mut World,
    a: &'static MonsterAbility,
    source: Src,
    trigger: MonsterAbilityTrigger,
) -> bool {
    if !a.trigger.intersects(trigger) || !mx::alive_and_present(w, source) {
        return false;
    }
    if is_ability_on_cooldown(w, source, a) {
        return false;
    }
    let c = a.chance;
    c >= 1.0 || (c > 0.0 && c > rnd::double(w))
}

/// `MonsterAbility.Trigger`: tracks the cooldown.
pub fn base_trigger(
    w: &mut World,
    a: &'static MonsterAbility,
    _t: MonsterAbilityTrigger,
    source: Src,
    _target: Tgt,
) {
    if a.min_cooldown <= TimeSpan::ZERO && a.max_cooldown <= TimeSpan::ZERO {
        return;
    }
    let amount = if a.min_cooldown == a.max_cooldown {
        a.min_cooldown
    } else {
        utility::random_min_max_time(&mut w.rng, a.min_cooldown, a.max_cooldown)
    };
    set_ability_cooldown(w, source, a, amount);
}

pub static BASE: MonsterAbility = MonsterAbility {
    name: "MonsterAbility",
    ability_type: MonsterAbilityType::Generic,
    trigger: MonsterAbilityTrigger::NONE,
    chance: 1.0,
    min_cooldown: TimeSpan::ZERO,
    max_cooldown: TimeSpan::ZERO,
    group: &[],
    area_range: 3,
    min_delay: TimeSpan::from_secs(10),
    max_delay: TimeSpan::from_secs(10),
    can_trigger: base_can_trigger,
    do_trigger: base_trigger,
    alter_melee_from: |_, _, _, _, _| {},
    alter_melee_to: |_, _, _, _, _| {},
    alter_spell_from: |_, _, _, _, _| {},
    alter_spell_to: |_, _, _, _, _| {},
    alter_scalar_from: |_, _, _, _, _| {},
    alter_scalar_to: |_, _, _, _, _| {},
    do_move: |_, _, _, _| {},
    can_effect_target: single_can_effect_target,
    on_before_target: |_, _, _, _, _| {},
    on_target: |_, _, _, _, _| {},
    do_effect_target: |_, _, _, _| {},
    get_count: |_, _, _, _| 1,
    effect_tick: |_, _, _, _, _| {},
    end_effect: |_, _, _, _| {},
    on_effect_expired: |_, _, _, _| {},
};

// ---- creature-side API -------------------------------------------------------------------------------------

/// `IsAbilityOnCooldown`
pub fn is_ability_on_cooldown(w: &mut World, source: Src, a: &'static MonsterAbility) -> bool {
    let tc = w.tick_count;
    let Some(d) = cd_mut(w, source) else {
        return false;
    };
    let Some(pos) = d.ability_cooldowns.iter().position(|(k, _)| *k == a.addr()) else {
        return false;
    };
    if d.ability_cooldowns[pos].1 - tc > 0 {
        return true;
    }
    d.ability_cooldowns.remove(pos);
    false
}

/// `SetAbilityCooldown`
pub fn set_ability_cooldown(
    w: &mut World,
    source: Src,
    a: &'static MonsterAbility,
    cooldown: TimeSpan,
) {
    let deadline = w.tick_count + cooldown.total_milliseconds() as i64;
    if let Some(d) = cd_mut(w, source) {
        match d.ability_cooldowns.iter_mut().find(|(k, _)| *k == a.addr()) {
            Some(e) => e.1 = deadline,
            None => d.ability_cooldowns.push((a.addr(), deadline)),
        }
    }
}

/// `ClearAbilityCooldowns`
pub fn clear_ability_cooldowns(w: &mut World, source: Src) {
    if let Some(d) = cd_mut(w, source) {
        d.ability_cooldowns.clear();
    }
}

fn abilities_of(w: &World, source: Src) -> &'static [&'static MonsterAbility] {
    ctype(w, source).map_or(NONE, |c| (c.get_monster_abilities)(w, source))
}

fn can_use(w: &World, source: Src, a: &'static MonsterAbility) -> bool {
    ctype(w, source).is_none_or(|c| (c.can_use_ability)(w, source, a))
}

/// `MonsterAbilityGroup.GetAbilityWithType`.
fn group_with_type(
    g: &'static MonsterAbility,
    ty: MonsterAbilityType,
) -> Option<&'static MonsterAbility> {
    g.group
        .iter()
        .map(|x| x.value)
        .find(|a| a.ability_type == ty)
}

/// `GetAbility`
pub fn get_ability(
    w: &World,
    source: Src,
    ty: MonsterAbilityType,
) -> Option<&'static MonsterAbility> {
    for &a in abilities_of(w, source) {
        if !can_use(w, source, a) {
            continue;
        }
        if !a.group.is_empty() {
            if let Some(found) = group_with_type(a, ty) {
                return Some(found);
            }
        } else if a.ability_type == ty {
            return Some(a);
        }
    }
    None
}

/// `HasAbility`
pub fn has_ability(w: &World, source: Src, ability: &'static MonsterAbility) -> bool {
    abilities_of(w, source)
        .iter()
        .any(|&a| same(a, ability) || ability.group.iter().any(|x| same(x.value, ability))) // a group matches by the argument, not the element
}

/// `TriggerAbility`: true if any ability triggered. Re-entrant calls with the same trigger do nothing.
pub fn trigger_ability(
    w: &mut World,
    source: Src,
    trigger: MonsterAbilityTrigger,
    defender: Tgt,
) -> bool {
    if read(w, source, trigger, |d| d.active_triggers).intersects(trigger)
        && cd(w, source).is_some_and(|d| d.active_triggers.intersects(trigger))
    {
        return false;
    }
    let abilities = abilities_of(w, source);
    if abilities.is_empty() {
        return false;
    }
    if let Some(d) = cd_mut(w, source) {
        d.active_triggers = d.active_triggers.or(trigger);
    }
    let mut triggered = false;
    for &a in abilities {
        if can_use(w, source, a) && (a.can_trigger)(w, a, source, trigger) {
            (a.do_trigger)(w, a, trigger, source, defender);
            triggered = true;
        }
    }
    if let Some(d) = cd_mut(w, source) {
        d.active_triggers = d.active_triggers.without(trigger);
    }
    triggered
}

/// `TriggerAbilityMove`
pub fn trigger_ability_move(
    w: &mut World,
    source: Src,
    trigger: MonsterAbilityTrigger,
    d: Direction,
) {
    for &a in abilities_of(w, source) {
        if can_use(w, source, a) && (a.can_trigger)(w, a, source, trigger) {
            (a.do_move)(w, a, source, d);
        }
    }
}

/// `TriggerAbilityAlterDamage`
pub fn trigger_ability_alter_damage(
    w: &mut World,
    source: Src,
    trigger: MonsterAbilityTrigger,
    defender: MobileId,
    damage: &mut i32,
) {
    for &a in abilities_of(w, source) {
        if can_use(w, source, a) && (a.can_trigger)(w, a, source, trigger) {
            if trigger.intersects(MonsterAbilityTrigger::GIVE_MELEE_DAMAGE) {
                (a.alter_melee_to)(w, a, source, defender, damage);
            }
            if trigger.intersects(MonsterAbilityTrigger::TAKE_MELEE_DAMAGE) {
                (a.alter_melee_from)(w, a, source, defender, damage);
            }
            if trigger.intersects(MonsterAbilityTrigger::GIVE_SPELL_DAMAGE) {
                (a.alter_spell_to)(w, a, source, defender, damage);
            }
            if trigger.intersects(MonsterAbilityTrigger::TAKE_SPELL_DAMAGE) {
                (a.alter_spell_from)(w, a, source, defender, damage);
            }
        }
    }
}

/// `TriggerAbilityAlterDamageScalar`
pub fn trigger_ability_alter_damage_scalar(
    w: &mut World,
    source: Src,
    trigger: MonsterAbilityTrigger,
    defender: MobileId,
    scalar: &mut f64,
) {
    for &a in abilities_of(w, source) {
        if can_use(w, source, a) && (a.can_trigger)(w, a, source, trigger) {
            if trigger.intersects(MonsterAbilityTrigger::GIVE_SPELL_DAMAGE) {
                (a.alter_scalar_to)(w, a, source, defender, scalar);
            }
            if trigger.intersects(MonsterAbilityTrigger::TAKE_SPELL_DAMAGE) {
                (a.alter_scalar_from)(w, a, source, defender, scalar);
            }
        }
    }
}

/// `[OnEvent(CreatureDeathEvent)] MonsterAbility.ResetCooldownsOnDeath`.
pub fn reset_cooldowns_on_death(w: &mut World, source: MobileId) {
    clear_ability_cooldowns(w, source);
}

// ---- MonsterAbilityGroup -------------------------------------------------------------------------------

fn group_can_trigger(
    w: &mut World,
    g: &'static MonsterAbility,
    source: Src,
    trigger: MonsterAbilityTrigger,
) -> bool {
    if !base_can_trigger(w, g, source, trigger) {
        return false;
    }
    let mut available = Vec::new();
    for (i, wa) in g.group.iter().enumerate() {
        let a = wa.value;
        if (a.can_trigger)(w, a, source, trigger) {
            available.push(i);
        }
    }
    let any = !available.is_empty();
    rt(w).group_available.insert(g.addr(), available);
    any
}

fn group_trigger(
    w: &mut World,
    g: &'static MonsterAbility,
    trigger: MonsterAbilityTrigger,
    source: Src,
    target: Tgt,
) {
    let available = rt(w)
        .group_available
        .get(&g.addr())
        .cloned()
        .unwrap_or_default();
    if available.is_empty() {
        return;
    }
    let slice: Vec<Weighted> = available.iter().map(|&i| g.group[i]).collect();
    let Some(chosen) = utility::random_weighted_element(&mut w.rng, &slice).map(|x| x.value) else {
        return;
    };
    base_trigger(w, g, trigger, source, target);
    (chosen.do_trigger)(w, chosen, trigger, source, target);
    rt(w).group_available.insert(g.addr(), Vec::new());
}

/// `new MonsterAbilityGroup(...)`: the trigger mask is the union of the members'.
pub const fn group(
    members: &'static [Weighted],
    triggers: MonsterAbilityTrigger,
) -> MonsterAbility {
    MonsterAbility {
        name: "MonsterAbilityGroup",
        trigger: triggers,
        group: members,
        can_trigger: group_can_trigger,
        do_trigger: group_trigger,
        alter_melee_from: |w, g, s, t, d| {
            g.group
                .iter()
                .for_each(|x| (x.value.alter_melee_from)(w, x.value, s, t, d))
        },
        alter_melee_to: |w, g, s, t, d| {
            g.group
                .iter()
                .for_each(|x| (x.value.alter_melee_to)(w, x.value, s, t, d))
        },
        alter_spell_from: |w, g, s, t, d| {
            g.group
                .iter()
                .for_each(|x| (x.value.alter_spell_from)(w, x.value, s, t, d))
        },
        alter_spell_to: |w, g, s, t, d| {
            g.group
                .iter()
                .for_each(|x| (x.value.alter_spell_to)(w, x.value, s, t, d))
        },
        alter_scalar_from: |w, g, s, t, d| {
            g.group
                .iter()
                .for_each(|x| (x.value.alter_scalar_from)(w, x.value, s, t, d))
        },
        alter_scalar_to: |w, g, s, t, d| {
            g.group
                .iter()
                .for_each(|x| (x.value.alter_scalar_to)(w, x.value, s, t, d))
        },
        do_move: |w, g, s, d| {
            g.group
                .iter()
                .for_each(|x| (x.value.do_move)(w, x.value, s, d))
        },
        ..BASE
    }
}

// ---- MonsterAbilitySingleTarget -------------------------------------------------------------------------

fn single_can_effect_target(
    w: &mut World,
    _a: &'static MonsterAbility,
    _t: MonsterAbilityTrigger,
    source: Src,
    defender: MobileId,
) -> bool {
    mx::alive(w, defender) && mx::can_be_harmful(w, source, defender)
}

fn single_trigger(
    w: &mut World,
    a: &'static MonsterAbility,
    trigger: MonsterAbilityTrigger,
    source: Src,
    target: Tgt,
) {
    // A null target (Think/Death triggers) does nothing.
    if let Some(t) = target
        && (a.can_effect_target)(w, a, trigger, source, t)
    {
        (a.on_before_target)(w, a, trigger, source, t);
        (a.on_target)(w, a, trigger, source, t);
    }
    base_trigger(w, a, trigger, source, target);
}

// ---- AreaEffectMonsterAbility ---------------------------------------------------------------------------

fn area_can_effect_target(
    w: &mut World,
    _a: &'static MonsterAbility,
    _t: MonsterAbilityTrigger,
    source: Src,
    defender: MobileId,
) -> bool {
    if source == defender || !mx::alive(w, defender) || !mx::can_be_harmful(w, source, defender) {
        return false;
    }
    let ok = mx::player(w, defender)
        || cd(w, defender).is_some_and(|dd| {
            let st = cd(w, source).map_or(0, |s| s.team);
            dd.team != st || dd.controlled || dd.summoned
        });
    ok && (!props_a::core_aos(w) || mx::in_los(w, source, defender))
}

fn area_trigger(
    w: &mut World,
    a: &'static MonsterAbility,
    trigger: MonsterAbilityTrigger,
    source: Src,
    target: Tgt,
) {
    let mut queue = Vec::new();
    for m in mx::mobiles_in_range(w, source, a.area_range) {
        if (a.can_effect_target)(w, a, trigger, source, m) {
            queue.push(m);
        }
    }
    for m in queue {
        (a.do_effect_target)(w, a, source, m);
    }
    base_trigger(w, a, trigger, source, target);
}

// ---- MonsterAbilitySingleTargetDoT ----------------------------------------------------------------------

/// `IsUnderEffect`.
pub fn is_under_effect(w: &mut World, a: &'static MonsterAbility, defender: MobileId) -> bool {
    rt(w).dot.contains_key(&(a.addr(), defender))
}

/// `RemoveEffect`: true if the defender was under the effect.
pub fn remove_effect(
    w: &mut World,
    a: &'static MonsterAbility,
    source: Src,
    defender: MobileId,
) -> bool {
    if let Some(h) = rt(w).dot.shift_remove(&(a.addr(), defender)) {
        timer::stop(w, h);
        timer::release(w, h);
        (a.end_effect)(w, a, source, defender);
        true
    } else {
        false
    }
}

fn dot_on_target(
    w: &mut World,
    a: &'static MonsterAbility,
    _t: MonsterAbilityTrigger,
    source: Src,
    defender: MobileId,
) {
    let duration = utility::random_min_max_time(&mut w.rng, a.min_delay, a.max_delay);
    let count = (a.get_count)(w, a, source, defender);
    let h = timer::create(w, duration, duration, count, move |w| {
        dot_tick(w, a, source, defender)
    });
    // `_table[defender] = timer` replaces an entry without stopping it.
    rt(w).dot.insert((a.addr(), defender), h);
    timer::begin(w, h);
}

fn dot_tick(w: &mut World, a: &'static MonsterAbility, source: Src, defender: MobileId) {
    let handle = timer::current(w);
    let remaining = handle.map_or(0, |h| h.remaining_count(w));
    let mut delay = if remaining <= 1 {
        TimeSpan::MIN
    } else {
        utility::random_min_max_time(&mut w.rng, a.min_delay, a.max_delay)
    };
    (a.effect_tick)(w, a, source, defender, &mut delay);
    if let Some(h) = handle {
        timer::set_interval(w, h, delay);
    }
    // Remaining decrements after OnTick so it is off by 1
    if handle.map_or(0, |h| h.remaining_count(w)) <= 1 {
        remove_effect(w, a, source, defender);
        (a.on_effect_expired)(w, a, source, defender);
    }
}

/// The base of a DoT ability: single-target dispatch whose `on_target` starts the expire timer.
const DOT: MonsterAbility = MonsterAbility {
    do_trigger: single_trigger,
    on_target: dot_on_target,
    ..BASE
};
const SINGLE: MonsterAbility = MonsterAbility {
    do_trigger: single_trigger,
    ..BASE
};
const AREA: MonsterAbility = MonsterAbility {
    do_trigger: area_trigger,
    can_effect_target: area_can_effect_target,
    ..BASE
};

fn aos_damage(w: &mut World, to: MobileId, from: Src, d: AosDamage) -> i32 {
    match hooks(w).and_then(|h| h.aos_damage) {
        Some(f) => f(w, to, Some(from), d),
        // Unset only before `BaseCreature.Configure` has run.
        None => {
            mx::damage(w, to, d.amount, Some(from));
            d.amount
        }
    }
}

const fn sc(n: i32) -> TimeSpan {
    TimeSpan::from_secs(n as i64)
}

// ---- the abilities --------------------------------------------------------------------------------------

macro_rules! ranged_counter_check {
    () => {
        |w, a, t, s, d| {
            single_can_effect_target(w, a, t, s, d)
                && hooks(w)
                    .and_then(|h| h.uses_ranged_weapon)
                    .is_some_and(|f| f(w, d))
        }
    };
}

pub static THROW_HATCHET_COUNTER: MonsterAbility = MonsterAbility {
    name: "ThrowHatchetCounter",
    ability_type: MonsterAbilityType::ThrowWeapon,
    trigger: MonsterAbilityTrigger::TAKE_DAMAGE,
    chance: 0.4,
    on_target: |w, _, _, s, d| {
        let dl = mx::entity(d);
        muo_server::mobile::part2::effects::moving_effect(
            w, s, dl, 0xF43, 10, 0, false, false, 0, 0,
        );
        mx::do_harmful(w, s, d);
        aos_damage(
            w,
            d,
            s,
            AosDamage {
                amount: 50,
                phys: 100,
                ..Default::default()
            },
        );
    },
    can_effect_target: ranged_counter_check!(),
    ..SINGLE
};

pub static ENERGY_BOLT_COUNTER: MonsterAbility = MonsterAbility {
    name: "EnergyBoltCounter",
    ability_type: MonsterAbilityType::EnergyBoltCounter,
    trigger: MonsterAbilityTrigger::TAKE_DAMAGE,
    chance: 0.4,
    on_target: |w, _, _, s, d| {
        muo_server::mobile::part2::effects::moving_particles(
            w,
            s,
            mx::entity(d),
            0x379F,
            7,
            0,
            false,
            true,
            0xBE3,
            0xFCB,
            0x211,
            0,
        );
        mx::play_sound(w, d, 0x229);
        mx::do_harmful(w, s, d);
        aos_damage(
            w,
            d,
            s,
            AosDamage {
                amount: 50,
                nrgy: 100,
                ..Default::default()
            },
        );
    },
    can_effect_target: |w, a, t, s, d| {
        single_can_effect_target(w, a, t, s, d)
            && (t == MonsterAbilityTrigger::TAKE_SPELL_DAMAGE
                || hooks(w)
                    .and_then(|h| h.uses_ranged_weapon)
                    .is_some_and(|f| f(w, d)))
    },
    ..SINGLE
};

pub static FAN_THROW_COUNTER: MonsterAbility = MonsterAbility {
    name: "FanThrowCounter",
    ability_type: MonsterAbilityType::FanThrow,
    trigger: MonsterAbilityTrigger::TAKE_DAMAGE,
    chance: 0.2,
    on_target: |w, _, _, s, d| {
        // No moving effect is sent.
        mx::do_harmful(w, s, d);
        let amount = rnd::min_max(w, 50, 65);
        aos_damage(
            w,
            d,
            s,
            AosDamage {
                amount,
                phys: 100,
                ..Default::default()
            },
        );
    },
    can_effect_target: |w, a, t, s, d| {
        single_can_effect_target(w, a, t, s, d) && !mx::in_range(w, d, s, 1)
    },
    ..SINGLE
};

pub static POISON_GAS_COUNTER: MonsterAbility = MonsterAbility {
    name: "PoisonGasCounter",
    ability_type: MonsterAbilityType::Poison,
    trigger: MonsterAbilityTrigger::TAKE_DAMAGE,
    chance: 0.05,
    can_effect_target: |w, _, _, s, d| {
        mx::in_range(w, s, d, 1) && !cd(w, d).is_some_and(|c| c.bard_provoked)
    },
    on_target: |w, _, _, s, d| {
        mx::animate(w, s, 10, 4, 1, true, false, 0);
        mx::do_harmful(w, s, d);
        aos_damage(
            w,
            d,
            s,
            AosDamage {
                amount: 50,
                phys: 100,
                ..Default::default()
            },
        );
        muo_server::mobile::part2::effects::fixed_particles_hue(
            w,
            d,
            0x36BD,
            1,
            10,
            0x1F78,
            0xA6,
            0,
            muo_server::effects::EffectLayer::None,
        );
        apply_named_poison(w, d, s, "Deadly");
    },
    ..SINGLE
};

fn apply_named_poison(w: &mut World, to: MobileId, from: Src, name: &str) {
    let id = w
        .resources
        .get::<muo_server::poison::Poisons>()
        .and_then(|p| p.get_poison(name))
        .map(|p| p.id());
    if let (Some(p), Some(ty)) = (id, w.mobile(to).map(|m| m.ty)) {
        (ty.apply_poison)(w, to, Some(from), p);
    }
}

pub static POISON_GAS_AREA_ATTACK: MonsterAbility = MonsterAbility {
    name: "PoisonGasAreaAttack",
    ability_type: MonsterAbilityType::MassPoison,
    trigger: MonsterAbilityTrigger::COMBAT_ACTION,
    chance: 0.5,
    min_cooldown: sc(5),
    max_cooldown: sc(30),
    can_trigger: |w, a, source, trigger| {
        if !base_can_trigger(w, a, source, trigger) {
            return false;
        }
        // Only do the mass ability as a side-effect of combating a specific target
        let Some(c) = mx::combatant(w, source) else {
            return false;
        };
        !mx::deleted(w, c)
            && mx::map(w, c) == mx::map(w, source)
            && mx::in_range_point(w, source, mx::location(w, c), 3)
            && mx::can_be_harmful(w, source, c)
            && mx::in_los(w, source, c)
    },
    do_trigger: |w, a, t, s, target| {
        muo_server::mobile::part2::effects::fixed_particles(
            w,
            s,
            0x376A,
            9,
            32,
            0x2539,
            muo_server::effects::EffectLayer::LeftHand,
        );
        mx::play_sound(w, s, 0x1DE);
        area_trigger(w, a, t, s, target);
    },
    do_effect_target: |w, _, s, d| apply_named_poison(w, d, s, "Deadly"),
    ..AREA
};

pub static DRAIN_LIFE_AREA_ATTACK: MonsterAbility = MonsterAbility {
    name: "DrainLifeAreaAttack",
    ability_type: MonsterAbilityType::MassDrainLife,
    trigger: MonsterAbilityTrigger::GIVE_MELEE_DAMAGE.or(MonsterAbilityTrigger::TAKE_MELEE_DAMAGE),
    chance: 0.1,
    area_range: 2,
    do_effect_target: |w, _, s, d| {
        muo_server::mobile::part2::effects::fixed_particles_hue(
            w,
            d,
            0x374A,
            10,
            15,
            5013,
            0x496,
            0,
            muo_server::effects::EffectLayer::Waist,
        );
        mx::play_sound(w, d, 0x231);
        mx::send_message(w, d, "You feel the life drain out of you!");
        // DrainLife: MinDamage 10, MaxDamage 40
        let to_drain = rnd::min_max(w, 10, 40);
        if let Some(m) = w.mobile(s).map(|m| m.hits) {
            muo_server::mobile::part1::props_b::set_hits(w, s, m + to_drain);
        }
        mx::do_harmful(w, s, d);
        mx::damage(w, d, to_drain, Some(s));
    },
    ..AREA
};

pub static DEATH_EXPLOSION: MonsterAbility = MonsterAbility {
    name: "DeathExplosion",
    ability_type: MonsterAbilityType::DeathExplosion,
    trigger: MonsterAbilityTrigger::DEATH,
    chance: 1.0,
    can_trigger: |_, _, _, _| true,
    do_trigger: |w, a, t, s, target| {
        if t != MonsterAbilityTrigger::DEATH || rt(w).chain_depth > 0 {
            return;
        }
        muo_server::mobile::part2::effects::fixed_particles(
            w,
            s,
            0x36BD,
            20,
            10,
            5044,
            muo_server::effects::EffectLayer::Head,
        );
        mx::play_sound(w, s, 0x307);
        rt(w).chain_depth += 1;
        area_trigger(w, a, t, s, target);
        rt(w).chain_depth -= 1;
    },
    do_effect_target: |w, _, s, d| {
        mx::do_harmful(w, s, d);
        // TODO: is the damage type correct?
        let amount = mx::hits_max(w, d) / 5;
        aos_damage(
            w,
            d,
            s,
            AosDamage {
                amount,
                ignore_armor: true,
                phys: 50,
                fire: 50,
                ..Default::default()
            },
        );
    },
    ..AREA
};

pub static REFLECT_PHYSICAL_DAMAGE: MonsterAbility = MonsterAbility {
    name: "ReflectPhysicalDamage",
    ability_type: MonsterAbilityType::ReflectPhysicalDamage,
    // This is handled manually in AOS.Damage()
    trigger: MonsterAbilityTrigger::COMBAT_ACTION,
    do_trigger: |_, _, _, _, _| {},
    ..BASE
};

pub static DESTROY_EQUIPMENT: MonsterAbility = MonsterAbility {
    name: "DestroyEquipment",
    ability_type: MonsterAbilityType::DestroyEquipment,
    trigger: MonsterAbilityTrigger::GIVE_MELEE_DAMAGE,
    chance: 0.05,
    can_effect_target: |w, _, _, s, d| {
        mx::player(w, d)
            && mx::access_level(w, d) == muo_server::mobile::AccessLevel::Player
            && mx::in_range(w, s, d, 1)
    },
    on_target: |w, _, _, s, d| {
        let Some(items) = w.mobile(d).map(|m| m.items.clone()) else {
            return;
        };
        let candidates: Vec<_> = items
            .into_iter()
            .filter(|&i| w.item(i).is_some_and(|it| !it.deleted()))
            .filter(|&i| {
                hooks(w)
                    .and_then(|h| h.destroy_equipment_candidate)
                    .is_some_and(|f| f(w, i))
            })
            .collect();
        if candidates.is_empty() {
            return;
        }
        mx::do_harmful(w, s, d);
        let pick = candidates[rnd::below(w, candidates.len() as i32) as usize];
        let name = muo_server::item::ops::name(w, pick)
            .map(|n| n.trim().to_owned())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| muo_server::item::ops::item_data(w, pick).name.clone());
        muo_server::item::base::delete(w, pick);
        // Their ~1_NAME~ is destroyed by the attack.
        muo_server::mobile::part2::messages::nonlocal_overhead_message_number(
            w,
            d,
            muo_primitives::MessageType::REGULAR,
            0x3B2,
            1080034,
            &name,
        );
        // Your ~1_NAME~ is destroyed by the attack.
        muo_server::mobile::part2::messages::local_overhead_message_number(
            w,
            d,
            muo_primitives::MessageType::REGULAR,
            0x3B2,
            1080035,
            &name,
        );
    },
    ..SINGLE
};

pub static MAGICAL_BARRIER: MonsterAbility = MonsterAbility {
    name: "MagicalBarrier",
    ability_type: MonsterAbilityType::MagicalBarrier,
    trigger: MonsterAbilityTrigger::TAKE_DAMAGE
        .or(MonsterAbilityTrigger::GIVE_DAMAGE)
        .or(MonsterAbilityTrigger::THINK),
    min_cooldown: sc(10),
    max_cooldown: sc(10),
    // Regeneration is subject to the cooldown, the rest are not.
    can_trigger: |w, a, s, t| t != MonsterAbilityTrigger::THINK || base_can_trigger(w, a, s, t),
    do_trigger: |w, a, t, s, target| {
        let can_use_field = |w: &World| mx::hits(w, s) >= mx::hits_max(w, s) * 9 / 10;
        if t == MonsterAbilityTrigger::THINK {
            if can_use_field(w)
                && rt(w)
                    .inactive_field
                    .get_mut(&a.addr())
                    .is_some_and(|f| f.shift_remove(&s))
            {
                // Field going up!
                muo_server::mobile::part2::effects::fixed_particles(
                    w,
                    s,
                    0,
                    10,
                    0,
                    0x2530,
                    muo_server::effects::EffectLayer::Waist,
                );
                if rt(w)
                    .inactive_field
                    .get(&a.addr())
                    .is_some_and(|f| f.is_empty())
                {
                    rt(w).inactive_field.shift_remove(&a.addr());
                }
            }
        } else if t == MonsterAbilityTrigger::TAKE_SPELL_DAMAGE && !can_use_field(w) {
            let added = rt(w).inactive_field.entry(a.addr()).or_default().insert(s);
            if added {
                muo_server::mobile::part2::effects::fixed_particles(
                    w,
                    s,
                    0x3735,
                    1,
                    30,
                    0x251F,
                    muo_server::effects::EffectLayer::Waist,
                );
            }
        }
        base_trigger(w, a, t, s, target);
    },
    alter_melee_from: |w, a, s, target, damage| {
        if has_field(w, a, s) {
            *damage = 0; // no melee damage when the field is up
            muo_server::mobile::part2::effects::fixed_particles(
                w,
                s,
                0x376A,
                20,
                10,
                0x2530,
                muo_server::effects::EffectLayer::Waist,
            );
            mx::play_sound(w, s, 0x2F4);
            mx::msg(w, target, 1114360); // Your weapon cannot penetrate the creature's magical barrier.
        }
    },
    alter_spell_from: |w, a, s, target, damage| {
        if !has_field(w, a, s) {
            *damage = 0; // no spell damage when the field is down
            muo_server::mobile::part2::effects::fixed_particles(
                w,
                s,
                0,
                10,
                0,
                0x2522,
                muo_server::effects::EffectLayer::Waist,
            );
            mx::msg(w, target, 1114359); // Your attack has no effect on the creature's armor.
        }
    },
    do_move: |w, a, s, _| {
        if has_field(w, a, s) && mx::combatant(w, s).is_some() {
            muo_server::mobile::part2::effects::fixed_particles(
                w,
                s,
                0,
                10,
                0,
                0x2530,
                muo_server::effects::EffectLayer::Waist,
            );
        }
    },
    ..BASE
};

/// `MagicalBarrier.HasField`.
pub fn has_field(w: &mut World, a: &'static MonsterAbility, source: Src) -> bool {
    !rt(w)
        .inactive_field
        .get(&a.addr())
        .is_some_and(|f| f.contains(&source))
}

fn stun_can_trigger(
    w: &mut World,
    a: &'static MonsterAbility,
    s: Src,
    t: MonsterAbilityTrigger,
) -> bool {
    !rt(w).stunning.contains(&s) && base_can_trigger(w, a, s, t)
}

fn stun_trigger(
    w: &mut World,
    a: &'static MonsterAbility,
    t: MonsterAbilityTrigger,
    s: Src,
    target: Tgt,
) {
    let Some(tg) = target else { return };
    rt(w).stunning.insert(s);
    if let Some(f) = hooks(w).and_then(|h| h.weapon_on_hit) {
        f(w, s, tg);
    }
    rt(w).stunning.shift_remove(&s);
    if mx::alive(w, tg) {
        mx::set_frozen(w, tg, true);
        let a2 = a;
        timer::start(w, sc(5), TimeSpan::ZERO, 1, move |w| {
            (a2.on_effect_expired)(w, a2, s, tg)
        });
    }
    base_trigger(w, a, t, s, target);
}

/// `ColossalBlow` (a `StunAttack`).
pub static COLOSSAL_BLOW: MonsterAbility = MonsterAbility {
    name: "ColossalBlow",
    ability_type: MonsterAbilityType::Stun,
    trigger: MonsterAbilityTrigger::GIVE_MELEE_DAMAGE,
    chance: 0.3,
    can_trigger: stun_can_trigger,
    do_trigger: |w, a, t, s, target| {
        if let Some(tg) = target {
            mx::animate(w, tg, 21, 6, 1, true, false, 0);
            mx::play_sound(w, s, 0xEE);
            mx::local_overhead(w, tg, 0x3B2, "You have been stunned by a colossal blow!");
        }
        stun_trigger(w, a, t, s, target);
    },
    // StunAttack.Recover + ColossalBlow.Recover (the stun's `Timer.DelayCall(StunDuration, Recover, target)`).
    on_effect_expired: |w, _, _, d| {
        mx::set_frozen(w, d, false);
        mx::set_combatant(w, d, None);
        mx::local_overhead(w, d, 0x3B2, "You recover your senses.");
    },
    ..BASE
};

pub static TAIL_SWIPE: MonsterAbility = MonsterAbility {
    name: "TailSwipe",
    ability_type: MonsterAbilityType::TailSwipe,
    trigger: MonsterAbilityTrigger::GIVE_MELEE_DAMAGE,
    chance: 0.3,
    // ServUO's SpecialAbility base cooldown; TailSwipe does not override it.
    min_cooldown: sc(30),
    max_cooldown: sc(30),
    on_target: |w, _, _, s, d| {
        mx::do_harmful(w, s, d);
        mx::play_sound(w, d, 0x204);
        mx::fixed_effect(w, d, 0x376A, 6, 1);
        // OSI/Stratics reports roughly a 90% stun / 10% confuse split.
        if rnd::double(w) < 0.10 {
            mx::msg(w, d, 1112555); // You're left confused as the creature's tail catches you right in the face!
            for (ty, name) in [
                (StatType::DEX, "[TailSwipe] Dex"),
                (StatType::INT, "[TailSwipe] Int"),
            ] {
                let m = StatMod::new(w, ty, Some(name), -20, sc(5), None);
                muo_server::mobile::base::add_stat_mod(w, d, m);
            }
        } else {
            mx::msg(w, d, 1112554); // You're stunned as the creature's tail knocks the wind out of you.
            mx::paralyze(w, d, sc(3));
        }
    },
    ..SINGLE
};

pub static SUMMON_PIXIES_COUNTER: MonsterAbility = MonsterAbility {
    name: "SummonPixiesCounter",
    ability_type: MonsterAbilityType::SummonCounter,
    trigger: MonsterAbilityTrigger::TAKE_DAMAGE,
    chance: 0.1,
    do_trigger: |w, a, t, s, target| {
        let amount = rnd::min_max(w, 3, 6);
        for _ in 0..amount {
            if !crate::creature::control::spawn_minion_by_name(
                w,
                s,
                "Server.Mobiles.Pixie",
                target,
                Some(FightMode::Closest),
            ) {
                break;
            }
        }
        base_trigger(w, a, t, s, target);
    },
    ..BASE
};

/// `SummonUndeadCounter`'s virtual properties are constants; `summon` is `CreateSummon(source)`.
fn summon_undead(
    w: &mut World,
    a: &'static MonsterAbility,
    t: MonsterAbilityTrigger,
    s: Src,
    target: Tgt,
    create: fn(&mut World) -> &'static str,
) {
    const AMOUNT_TO_SUMMON: i32 = 4;
    const SUMMON_RANGE: i32 = 4;
    const CHANCE_TO_POLYMORPH: f64 = 0.25;
    let followers = w.mobile(s).map_or(0, |m| m.followers);
    let amount = AMOUNT_TO_SUMMON - followers;
    let paralyzed = w.mobile(s).is_some_and(|m| (m.ty.paralyzed)(w, s));
    let will_transform =
        if !paralyzed && CHANCE_TO_POLYMORPH > 0.0 && CHANCE_TO_POLYMORPH > rnd::double(w) {
            rnd::below(w, amount)
        } else {
            -1
        };
    let map = mx::map(w, s);
    for i in 0..amount {
        let loc = crate::creature::control::valid_location_in_los(w, map, s, SUMMON_RANGE);
        let summon = if i == will_transform {
            let body = if rnd::bool(w) { 50 } else { 56 };
            muo_server::mobile::part1::props_b::set_body_mod(w, s, muo_primitives::Body::new(body));
            muo_server::mobile::part1::props_b::set_hue_mod(w, s, 0);
            // Name is not changed
            timer::start(w, TimeSpan::from_mins(5), TimeSpan::ZERO, 1, move |w| {
                muo_server::mobile::part1::props_b::set_body_mod(
                    w,
                    s,
                    muo_primitives::Body::new(0),
                );
                muo_server::mobile::part1::props_b::set_hue_mod(w, s, -1);
            });
            Some(s)
        } else {
            let name = create(w);
            let m = crate::factory::make_mobile(w, name);
            if let Some(m) = m {
                let (team, home, range_home) =
                    cd(w, s).map_or((0, Point3D::ZERO, 10), |c| (c.team, c.home, c.range_home));
                // Team, Summoned, Master, FightMode, Combatant, Home, RangeHome: the setters' effects, in order.
                crate::creature::control::set_team(w, m, team);
                crate::creature::control::set_summoned(w, m, true);
                crate::creature::control::set_master(w, m, Some(s));
                crate::creature::edit(w, m, |d| d.fight_mode = FightMode::Closest);
                if let Some(tg) = target {
                    mx::set_combatant(w, m, Some(tg));
                }
                crate::creature::control::set_home(w, m, home);
                crate::creature::edit(w, m, |d| d.range_home = range_home);
            }
            m
        };
        let Some(summon) = summon else { continue };
        if let Some(map) = map {
            muo_server::mobile::base::move_to_world(w, summon, loc, map);
            muo_server::effects::send_location_effect(w, loc, Some(map), 0x3728, 10, 10, 0, 0);
        }
        mx::play_sound(w, summon, 0x48F);
        let snd = w
            .mobile(summon)
            .map(|m| m.ty)
            .map_or(0, |m| (m.get_attack_sound)(w, summon));
        mx::play_sound(w, summon, snd);
    }
    base_trigger(w, a, t, s, target);
    let _ = SUMMON_RANGE;
}

const SUMMON_UNDEAD: MonsterAbility = MonsterAbility {
    ability_type: MonsterAbilityType::SummonCounter,
    trigger: MonsterAbilityTrigger::TAKE_DAMAGE,
    chance: 0.05,
    min_cooldown: TimeSpan::from_mins(5),
    max_cooldown: TimeSpan::from_mins(5),
    can_trigger: |w, a, s, t| {
        w.mobile(s).is_some_and(|m| m.followers < 4) && base_can_trigger(w, a, s, t)
    },
    ..BASE
};

pub static SUMMON_SKELETONS_COUNTER: MonsterAbility = MonsterAbility {
    name: "SummonSkeletonsCounter",
    do_trigger: |w, a, t, s, target| {
        summon_undead(w, a, t, s, target, |_| "Server.Mobiles.Skeleton")
    },
    ..SUMMON_UNDEAD
};

pub static SUMMON_LESSER_UNDEAD_COUNTER: MonsterAbility = MonsterAbility {
    name: "SummonLesserUndeadCounter",
    do_trigger: |w, a, t, s, target| {
        // `_group = Utility.Random(2)` once per trigger, then one pick per summon.
        let group = rnd::below(w, 2);
        summon_undead_group(w, a, t, s, target, group);
    },
    ..SUMMON_UNDEAD
};

fn summon_undead_group(
    w: &mut World,
    a: &'static MonsterAbility,
    t: MonsterAbilityTrigger,
    s: Src,
    target: Tgt,
    group: i32,
) {
    // A fn pointer cannot capture `group`; pick through a resource-free path: the group is re-derived per summon
    // from a one-shot slot.
    w.resources.get_mut::<LesserGroup>().0 = group;
    summon_undead(w, a, t, s, target, |w| {
        let group = w.resources.get_mut::<LesserGroup>().0;
        if group == 0 {
            [
                "Server.Mobiles.Skeleton",
                "Server.Mobiles.SkeletalMage",
                "Server.Mobiles.Mummy",
                "Server.Mobiles.Zombie",
            ][rnd::below(w, 4) as usize]
        } else {
            [
                "Server.Mobiles.Ghoul",
                "Server.Mobiles.Spectre",
                "Server.Mobiles.Shade",
                "Server.Mobiles.Bogle",
            ][rnd::below(w, 4) as usize]
        }
    });
}

/// `_group` carried between the two calls of one trigger.
#[derive(Default)]
struct LesserGroup(i32);

// ---- DoT abilities --------------------------------------------------------------------------------------

fn resist_mod(w: &mut World, defender: MobileId, ty: ResistanceType, name: &str, offset: i32) {
    let m = ResistanceMod::new(w, ty, Some(name), offset, None);
    muo_server::mobile::base::add_resistance_mod(w, defender, m);
}

fn resistance_of(w: &mut World, id: MobileId, ty: ResistanceType) -> i32 {
    w.mobile(id).map(|m| m.ty).map_or(0, |t| match ty {
        ResistanceType::Physical => (t.physical_resistance)(w, id),
        ResistanceType::Fire => (t.fire_resistance)(w, id),
        ResistanceType::Cold => (t.cold_resistance)(w, id),
        ResistanceType::Poison => (t.poison_resistance)(w, id),
        ResistanceType::Energy => (t.energy_resistance)(w, id),
    })
}

fn remove_resist_mod(w: &mut World, id: MobileId, name: &str) {
    muo_server::mobile::base::remove_resistance_mod_by_name(w, id, name);
}

pub static GRASPING_CLAW: MonsterAbility = MonsterAbility {
    name: "GraspingClaw",
    ability_type: MonsterAbilityType::GraspingClaw,
    trigger: MonsterAbilityTrigger::GIVE_MELEE_DAMAGE,
    chance: 0.10,
    on_before_target: |w, a, _, s, d| {
        if remove_effect(w, a, s, d) {
            mx::msg(w, d, 1070837); // The creature lands another blow in your weakened state.
        } else {
            mx::msg(w, d, 1070836); // The blow from the creature's claws has made you more susceptible to physical attacks.
        }
    },
    on_target: |w, a, t, s, d| {
        dot_on_target(w, a, t, s, d);
        mx::do_harmful(w, s, d);
        let phys = resistance_of(w, d, ResistanceType::Physical);
        resist_mod(
            w,
            d,
            ResistanceType::Physical,
            "GraspingClaw",
            -(phys * 15 / 100),
        );
        mx::fixed_effect(w, d, 0x37B9, 10, 5);
    },
    end_effect: |w, _, _, d| remove_resist_mod(w, d, "GraspingClaw"),
    on_effect_expired: |w, _, _, d| mx::msg(w, d, 1070838), // Your resistance to physical attacks has returned.
    ..DOT
};

pub static RUNE_CORRUPTION: MonsterAbility = MonsterAbility {
    name: "RuneCorruption",
    ability_type: MonsterAbilityType::RuneCorruption,
    trigger: MonsterAbilityTrigger::GIVE_MELEE_DAMAGE,
    chance: 0.05,
    on_before_target: |w, a, _, s, d| {
        if remove_effect(w, a, s, d) {
            mx::msg(w, d, 1070845); // The creature continues to corrupt your armor!
        } else {
            mx::msg(w, d, 1070846); // The creature magically corrupts your armor!
        }
    },
    on_target: |w, a, t, s, d| {
        dot_on_target(w, a, t, s, d);
        mx::do_harmful(w, s, d);
        let ml = w.core_ml();
        for ty in [
            ResistanceType::Physical,
            ResistanceType::Fire,
            ResistanceType::Cold,
            ResistanceType::Poison,
            ResistanceType::Energy,
        ] {
            let r = resistance_of(w, d, ty);
            if r > 0 {
                let offset = if ml {
                    -(r / 2)
                } else if r > 70 {
                    -70
                } else {
                    -r
                };
                resist_mod(w, d, ty, "RuneCorruption", offset);
            }
        }
        mx::fixed_effect(w, d, 0x37B9, 10, 5);
    },
    end_effect: |w, _, _, d| remove_resist_mod(w, d, "RuneCorruption"),
    on_effect_expired: |w, _, _, d| mx::msg(w, d, 1071967), // The corruption of your armor has worn off
    ..DOT
};

/// `new FanningFire(0.05, -10, 35, 45)`, the only configuration `MonsterAbilities` creates.
pub static FANNING_FIRE: MonsterAbility = MonsterAbility {
    name: "FanningFire",
    ability_type: MonsterAbilityType::FanningFire,
    trigger: MonsterAbilityTrigger::COMBAT_ACTION,
    chance: 0.05,
    on_before_target: |w, a, _, s, d| {
        // Renew the effects
        remove_effect(w, a, s, d);
        mx::msg(w, d, 1070833); // The creature fans you with fire, reducing your resistance to fire attacks.
    },
    on_target: |w, a, t, s, d| {
        dot_on_target(w, a, t, s, d);
        mx::do_harmful(w, s, d);
        resist_mod(w, d, ResistanceType::Fire, "FanningFire", -10);
        muo_server::mobile::part2::effects::fixed_particles(
            w,
            d,
            0x3709,
            10,
            30,
            0x34,
            muo_server::effects::EffectLayer::RightFoot,
        );
        mx::play_sound(w, d, 0x208);
        let amount = rnd::min_max(w, 35, 45);
        aos_damage(
            w,
            d,
            s,
            AosDamage {
                amount,
                fire: 100,
                ..Default::default()
            },
        );
    },
    end_effect: |w, _, _, d| remove_resist_mod(w, d, "FanningFire"),
    on_effect_expired: |w, _, _, d| mx::msg(w, d, 1070834), // Your resistance to fire attacks has returned.
    ..DOT
};

pub static DRAIN_LIFE_ATTACK: MonsterAbility = MonsterAbility {
    name: "DrainLifeAttack",
    ability_type: MonsterAbilityType::DrainLife,
    trigger: MonsterAbilityTrigger::GIVE_MELEE_DAMAGE,
    chance: 0.5,
    min_delay: sc(1),
    max_delay: sc(1),
    get_count: |_, _, _, _| 5,
    can_effect_target: |w, a, t, s, d| {
        single_can_effect_target(w, a, t, s, d) && w.mobile(d).is_some_and(|m| m.mana > 14)
    },
    on_before_target: |w, a, _, s, d| {
        if remove_effect(w, a, s, d) {
            mx::msg(w, d, 1070847); // The creature continues to steal your life force!
        } else {
            mx::msg(w, d, 1070848); // You feel your life force being stolen away.
        }
    },
    on_target: |w, a, t, s, d| {
        dot_on_target(w, a, t, s, d);
        drain_life(w, &DRAIN_LIFE_ATTACK, s, d);
    },
    effect_tick: |w, a, s, d, _| drain_life(w, a, s, d),
    on_effect_expired: |w, _, _, d| mx::msg(w, d, 1070849), // The drain on your life force is gone.
    ..DOT
};

fn drain_life(w: &mut World, a: &'static MonsterAbility, source: Src, defender: MobileId) {
    mx::do_harmful(w, source, defender);
    if let Some(m) = w.mobile(defender).map(|m| m.mana) {
        muo_server::mobile::part1::props_b::set_mana(w, defender, m - 15);
    }
    if mx::alive(w, defender) {
        let given = aos_damage(
            w,
            defender,
            source,
            AosDamage {
                amount: 5,
                nrgy: 100,
                ..Default::default()
            },
        );
        let h = mx::hits(w, source);
        muo_server::mobile::part1::props_b::set_hits(w, source, h + given);
    } else {
        remove_effect(w, a, source, defender);
        mx::msg(w, defender, 1070849); // The drain on your life force is gone.
    }
}

pub static BLOOD_BATH_ATTACK: MonsterAbility = MonsterAbility {
    name: "BloodBathAttack",
    ability_type: MonsterAbilityType::BloodBath,
    trigger: MonsterAbilityTrigger::GIVE_MELEE_DAMAGE,
    chance: 0.1,
    min_delay: sc(1),
    max_delay: sc(1),
    get_count: |_, _, _, _| 5,
    effect_tick: |w, a, s, d, _| {
        if mx::alive(w, d) {
            mx::damage(w, d, 2, Some(s));
        } else {
            remove_effect(w, a, s, d);
        }
    },
    on_before_target: |w, a, _, s, d| {
        if remove_effect(w, a, s, d) {
            mx::msg(w, d, 1070825); // The creature continues to rage!
        } else {
            mx::msg(w, d, 1070826); // The creature goes into a rage, inflicting heavy damage!
        }
    },
    on_effect_expired: |w, _, _, d| mx::msg(w, d, 1070824), // The creature's rage subsides.
    ..DOT
};

// ---- Fire Breath ----------------------------------------------------------------------------------------

/// `FireBreath`'s `virtual` numbers. `ColdBreath` and `ChaosBreath` override a few.
#[derive(Copy, Clone)]
pub struct Breath {
    pub chaos_damage: i32,
    pub physical_damage: i32,
    pub fire_damage: i32,
    pub cold_damage: i32,
    pub poison_damage: i32,
    pub energy_damage: i32,
    pub effect_hue: i32,
}

const FIRE: Breath = Breath {
    chaos_damage: 0,
    physical_damage: 0,
    fire_damage: 100,
    cold_damage: 0,
    poison_damage: 0,
    energy_damage: 0,
    effect_hue: 0,
};

fn breath_of(a: &'static MonsterAbility) -> Breath {
    match a.name {
        "ColdBreath" => Breath {
            cold_damage: 100,
            fire_damage: 0,
            effect_hue: 0x480,
            ..FIRE
        },
        "ChaosBreath" => Breath {
            chaos_damage: 100,
            fire_damage: 0,
            ..FIRE
        },
        // Leviathan's `LeviathanBreath`: a private subclass of FireBreath
        "LeviathanBreath" => Breath {
            physical_damage: 70,
            cold_damage: 30,
            fire_damage: 0,
            effect_hue: 0x1ED,
            ..FIRE
        },
        _ => FIRE,
    }
}

const FIRE_BREATH_BASE: MonsterAbility = MonsterAbility {
    ability_type: MonsterAbilityType::FireBreath,
    trigger: MonsterAbilityTrigger::COMBAT_ACTION,
    chance: 0.5,
    // Min/max seconds until next breath
    min_cooldown: sc(30),
    max_cooldown: sc(45),
    can_trigger: |w, a, s, t| !read(w, s, false, |d| d.summoned) && base_can_trigger(w, a, s, t),
    do_trigger: breath_trigger,
    ..BASE
};

pub static FIRE_BREATH: MonsterAbility = MonsterAbility {
    name: "FireBreath",
    ..FIRE_BREATH_BASE
};
pub static COLD_BREATH: MonsterAbility = MonsterAbility {
    name: "ColdBreath",
    ..FIRE_BREATH_BASE
};
pub static CHAOS_BREATH: MonsterAbility = MonsterAbility {
    name: "ChaosBreath",
    ..FIRE_BREATH_BASE
};

const BREATH_STALL_TIME: f64 = 1.0;
const BREATH_EFFECT_DELAY: f64 = 1.3;
const BREATH_DAMAGE_DELAY: f64 = 1.0;

/// `CanFireBreathTarget`.
pub fn can_fire_breath_target(w: &mut World, source: Src, target: MobileId) -> bool {
    let range = read(w, source, 0, |d| d.range_perception);
    let dead_bonded = |w: &World, id: MobileId| {
        w.mobile(id)
            .is_some_and(|m| (m.ty.is_dead_bonded_pet)(w, id))
    };
    !dead_bonded(w, source)
        && !read(w, source, false, |d| d.bard_pacified)
        && mx::alive(w, target)
        && !dead_bonded(w, target)
        && mx::map(w, target) == mx::map(w, source)
        && mx::can_be_harmful(w, source, target)
        && mx::in_range(w, target, source, range)
        && mx::in_los(w, source, target)
}

fn breath_trigger(
    w: &mut World,
    a: &'static MonsterAbility,
    t: MonsterAbilityTrigger,
    source: Src,
    target: Tgt,
) {
    let Some(tg) = target else { return };
    if !can_fire_breath_target(w, source, tg) {
        return;
    }
    // BreathStallMovement
    let stall = w.tick_count + (BREATH_STALL_TIME * 1000.0) as i64;
    crate::ai::set_next_move(w, source, stall);
    // BreathPlayAngerSound / Animation
    let snd = w
        .mobile(source)
        .map(|m| m.ty)
        .map_or(0, |m| (m.get_anger_sound)(w, source));
    mx::play_sound(w, source, snd);
    mx::animate(w, source, 12, 5, 1, true, false, 0);
    let dir = mx::get_direction_to(w, source, tg);
    mx::set_direction(w, source, dir);
    timer::start(
        w,
        TimeSpan::from_millis_f64(BREATH_EFFECT_DELAY * 1000.0),
        TimeSpan::ZERO,
        1,
        move |w| breath_effect(w, a, source, tg),
    );
    base_trigger(w, a, t, source, target);
}

fn breath_effect(w: &mut World, a: &'static MonsterAbility, source: Src, target: MobileId) {
    if !mx::alive(w, target) || !mx::can_be_harmful(w, source, target) {
        return;
    }
    let b = breath_of(a);
    mx::play_sound(w, source, 0x227);
    muo_server::mobile::part2::effects::moving_effect(
        w,
        source,
        mx::entity(target),
        0x36D4,
        5,
        0,
        false,
        false,
        b.effect_hue,
        0,
    );
    timer::start(
        w,
        TimeSpan::from_millis_f64(BREATH_DAMAGE_DELAY * 1000.0),
        TimeSpan::ZERO,
        1,
        move |w| breath_damage(w, a, source, target),
    );
}

fn breath_damage(w: &mut World, a: &'static MonsterAbility, source: Src, target: MobileId) {
    if ctype(w, target).is_some_and(|c| (c.breath_immune)(w, target)) {
        return;
    }
    if mx::can_be_harmful(w, source, target) {
        mx::do_harmful(w, source, target);
        breath_deal_damage(w, a, source, target);
    }
}

fn breath_deal_damage(w: &mut World, a: &'static MonsterAbility, source: Src, target: MobileId) {
    if hooks(w)
        .and_then(|h| h.check_spell_evasion)
        .is_some_and(|f| f(w, target))
    {
        return;
    }
    let b = breath_of(a);
    let (mut phys, mut fire, mut cold, mut pois, mut nrgy) = (
        b.physical_damage,
        b.fire_damage,
        b.cold_damage,
        b.poison_damage,
        b.energy_damage,
    );
    if b.chaos_damage > 0 {
        match rnd::below(w, 5) {
            0 => phys += b.chaos_damage,
            1 => fire += b.chaos_damage,
            2 => cold += b.chaos_damage,
            3 => pois += b.chaos_damage,
            _ => nrgy += b.chaos_damage,
        }
    }
    let amount = if a.name == "LeviathanBreath" {
        breath_compute_damage_scalar(w, source, 0.05) // BreathDamageScalar
    } else {
        breath_compute_damage(w, source)
    };
    if phys == 0 && fire == 0 && cold == 0 && pois == 0 && nrgy == 0 {
        mx::damage(w, target, amount, Some(source)); // Unresistable damage even in AOS
    } else {
        aos_damage(
            w,
            target,
            source,
            AosDamage {
                amount,
                phys,
                fire,
                cold,
                pois,
                nrgy,
                ..Default::default()
            },
        );
    }
}

/// `BreathComputeDamage`.
pub fn breath_compute_damage(w: &World, source: Src) -> i32 {
    breath_compute_damage_scalar(w, source, if props_a::core_aos(w) { 0.16 } else { 0.05 })
}

/// `BreathComputeDamage` with the ability's `BreathDamageScalar`.
pub fn breath_compute_damage_scalar(w: &World, source: Src, scalar: f64) -> i32 {
    let mut damage = (mx::hits(w, source) as f64 * scalar) as i32;
    if read(w, source, false, |d| d.is_paragon) {
        let buff = hooks(w).and_then(|h| h.paragon_hits_buff).unwrap_or(5.0);
        damage = (damage as f64 / buff) as i32;
    }
    damage.min(200)
}

// `MonsterAbilities.*`: each read of a property would make a new instance, so two reads would be
// two cooldown keys; here each ability is one static and a creature class lists the statics it uses. The only
// parameterized one, `new FanningFire(0.05, -10, 35, 45)`, is `FANNING_FIRE`.
