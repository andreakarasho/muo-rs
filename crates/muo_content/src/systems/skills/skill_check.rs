//! The skill gain algorithm behind `Mobile.CheckSkill`/`CheckTargetSkill`, and
//! stat gain.

use muo_primitives::{DateTime, Point2D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::StatLockType;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::statics::Part2Statics;
use muo_server::skills::{self, SkillLock, SkillName, SkillTable, Stat};
use muo_server::target::Targeted;

use super::anti_macro::{self, AntiMacroObject};
use super::skill_events;
use crate::core;
use crate::creature;
use crate::player_mobile;

/// The `SkillCheck` statics.
pub struct SkillCheckSettings {
    pub stat_max: i32,
    pub stat_gain_chance_multiplier: f64,
    pub use_pub45_stat_gain: bool,
    pub primary_stat_gain_chance: f64,
    pub stat_gain_delay: TimeSpan,
    pub pet_stat_gain_delay: TimeSpan,
}

impl Default for SkillCheckSettings {
    fn default() -> Self {
        Self {
            stat_max: 100,
            stat_gain_chance_multiplier: 1.0,
            use_pub45_stat_gain: false,
            primary_stat_gain_chance: 0.75,
            stat_gain_delay: TimeSpan::from_mins(10),
            pet_stat_gain_delay: TimeSpan::from_mins(5),
        }
    }
}

fn settings(w: &mut World) -> &SkillCheckSettings {
    w.resources.get_mut::<SkillCheckSettings>()
}

/// `RollStatIncreaseChance(statGain)`.
fn roll_stat_increase_chance(w: &mut World, stat_gain: f64) -> bool {
    stat_gain / 33.3 * settings(w).stat_gain_chance_multiplier > utility::random_double(&mut w.rng)
}

/// `SkillCheck.Configure()`.
pub fn configure(w: &mut World) {
    let lbr = core::lbr(w);
    let ml = core::ml(w);
    let stat_max = w
        .config
        .get_or_update_setting("stats.statMax", if lbr { 125 } else { 100 })
        .unwrap_or(100);
    let mult = w
        .config
        .get_or_update_setting("stats.gainChanceMultiplier", 1.0)
        .unwrap_or(1.0);
    let primary = w.config.get_setting("stats.primaryStatGainChance", 0.75);
    let delay = w.config.get_setting(
        "stats.gainDelay",
        TimeSpan::from_mins_f64(if ml { 0.05 } else { 10.0 }),
    );
    let pet_delay = w
        .config
        .get_setting("stats.petGainDelay", TimeSpan::from_mins(5));
    // Publish 45 - Preparation for UOKR
    let pub45 = w.config.get_setting("stats.usePub45StatGain", ml);
    *w.resources.get_mut::<SkillCheckSettings>() = SkillCheckSettings {
        stat_max,
        stat_gain_chance_multiplier: mult,
        use_pub45_stat_gain: pub45,
        primary_stat_gain_chance: primary,
        stat_gain_delay: delay,
        pet_stat_gain_delay: pet_delay,
    };
}

/// `SkillCheck.Initialize()`: the four handlers `Mobile.CheckSkill` & co. go through.
pub fn initialize(w: &mut World) {
    let s = w.resources.get_mut::<Part2Statics>();
    s.skill_check_location_handler = Some(mobile_skill_check_location);
    s.skill_check_direct_location_handler = Some(mobile_skill_check_direct_location);
    s.skill_check_target_handler = Some(mobile_skill_check_target);
    s.skill_check_direct_target_handler = Some(mobile_skill_check_direct_target);
}

fn has_skill(w: &World, from: MobileId, skill: SkillName) -> bool {
    w.mobile(from)
        .and_then(|m| m.skills.as_ref())
        .is_some_and(|s| (skill as usize) < s.len().max(SkillName::COUNT))
}

pub fn mobile_skill_check_location(
    w: &mut World,
    from: MobileId,
    skill: SkillName,
    min_skill: f64,
    max_skill: f64,
) -> bool {
    if !has_skill(w, from, skill) {
        return false;
    }
    let success = check_location(w, from, skill, min_skill, max_skill);
    skill_events::invoke_skill_used(w, from, skill, success);
    success
}

fn location_object(w: &mut World, from: MobileId) -> AntiMacroObject {
    let size = anti_macro::settings(w).location_size.max(1);
    let loc = w.mobile(from).map(|m| m.location).unwrap_or_default();
    AntiMacroObject::Location(Point2D::new(loc.x / size, loc.y / size))
}

fn check_location(
    w: &mut World,
    from: MobileId,
    skill: SkillName,
    min_skill: f64,
    max_skill: f64,
) -> bool {
    let value = skills::value(w, from, skill);

    if value < min_skill {
        return false; // Too difficult
    }

    if value >= max_skill || min_skill >= max_skill {
        return true; // No challenge
    }

    let chance = (value - min_skill) / (max_skill - min_skill);

    let obj = location_object(w, from);
    check_skill(w, from, skill, Some(obj), chance)
}

pub fn mobile_skill_check_direct_location(
    w: &mut World,
    from: MobileId,
    skill: SkillName,
    chance: f64,
) -> bool {
    if !has_skill(w, from, skill) {
        return false;
    }
    let success = check_direct_location(w, from, skill, chance);
    skill_events::invoke_skill_used(w, from, skill, success);
    success
}

fn check_direct_location(w: &mut World, from: MobileId, skill: SkillName, chance: f64) -> bool {
    if chance < 0.0 {
        return false; // Too difficult
    }

    if chance >= 1.0 {
        return true; // No challenge
    }

    let obj = location_object(w, from);
    check_skill(w, from, skill, Some(obj), chance)
}

/// `CheckSkill(from, skill, amObj, chance)`.
pub fn check_skill(
    w: &mut World,
    from: MobileId,
    skill: SkillName,
    am_obj: Option<AntiMacroObject>,
    chance: f64,
) -> bool {
    if w.mobile(from)
        .and_then(|m| m.skills.as_ref())
        .is_none_or(|s| s.cap == 0)
    {
        return false;
    }

    let success = chance >= utility::random_double(&mut w.rng);

    let alive = w.mobile(from).is_some_and(|m| (m.ty.alive)(w, from));
    let region = w.mobile(from).map(|m| m.region);
    let allow_region = match region.and_then(|r| w.region(r).map(|reg| (r, reg.ty))) {
        Some((rid, rt)) => (rt.allow_gain)(w, rid, from, skill, am_obj.and_then(|o| o.entity())),
        None => true,
    };
    if alive && allow_region {
        if skills::base(w, from, skill) < 10.0 {
            // Gain regardless of the AllowGain check
            gain(w, from, skill);
        } else if allow_gain(w, from, skill, am_obj) {
            let (cap, total) = w
                .mobile(from)
                .and_then(|m| m.skills.as_ref())
                .map_or((1, 0), |s| (s.cap, s.total));
            let mut gc = f64::from(cap - total) / f64::from(cap);
            let skill_cap = skills::cap(w, from, skill);
            gc += (skill_cap - skills::base(w, from, skill)) / skill_cap;
            gc /= 2.0;

            gc += (1.0 - chance)
                * (if success {
                    0.5
                } else if core::aos(w) {
                    0.0
                } else {
                    0.2
                });
            gc /= 2.0;

            gc *= w
                .resources
                .get::<SkillTable>()
                .and_then(|t| t.info.get(skill as usize))
                .map_or(1.0, |i| i.gain_factor);

            if gc < 0.01 {
                gc = 0.01;
            }

            if creature::cd(w, from).is_some_and(|c| c.controlled) {
                gc *= 2.0;
            }

            if gc >= utility::random_double(&mut w.rng) {
                gain(w, from, skill);
            }
        }

        // Gain one stat per 10 minutes, regardless of whether you gain in the skill
        if !settings(w).use_pub45_stat_gain && success {
            legacy_gain(w, from, skill);
        }
    }

    success
}

pub fn mobile_skill_check_target(
    w: &mut World,
    from: MobileId,
    skill: SkillName,
    target: Targeted,
    min_skill: f64,
    max_skill: f64,
) -> bool {
    if !has_skill(w, from, skill) {
        return false;
    }
    let success = check_target(w, from, skill, target, min_skill, max_skill);
    skill_events::invoke_skill_used(w, from, skill, success);
    success
}

fn check_target(
    w: &mut World,
    from: MobileId,
    skill: SkillName,
    target: Targeted,
    min_skill: f64,
    max_skill: f64,
) -> bool {
    let value = skills::value(w, from, skill);

    if value < min_skill {
        return false; // Too difficult
    }

    if value >= max_skill || min_skill >= max_skill {
        return true; // No challenge
    }

    let chance = (value - min_skill) / (max_skill - min_skill);

    check_skill(w, from, skill, Some(AntiMacroObject::from(target)), chance)
}

pub fn mobile_skill_check_direct_target(
    w: &mut World,
    from: MobileId,
    skill: SkillName,
    target: Targeted,
    chance: f64,
) -> bool {
    if !has_skill(w, from, skill) {
        return false;
    }
    let success = check_direct_target(w, from, skill, target, chance);
    skill_events::invoke_skill_used(w, from, skill, success);
    success
}

fn check_direct_target(
    w: &mut World,
    from: MobileId,
    skill: SkillName,
    target: Targeted,
    chance: f64,
) -> bool {
    if chance < 0.0 {
        return false; // Too difficult
    }

    if chance >= 1.0 {
        return true; // No challenge
    }

    check_skill(w, from, skill, Some(AntiMacroObject::from(target)), chance)
}

fn allow_gain(
    w: &mut World,
    from: MobileId,
    skill: SkillName,
    obj: Option<AntiMacroObject>,
) -> bool {
    // Changed some time between the introduction of AoS and SE.
    if core::aos(w) && super::hooks::ops(w).faction_in_skill_loss(w, from) {
        return false;
    }

    if !w
        .mobile(from)
        .is_some_and(|m| m.ty.is(&player_mobile::PLAYER_MOBILE))
    {
        return true;
    }
    anti_macro::anti_macro_check(w, from, skill, obj)
}

/// `Gain(from, skill)`.
pub fn gain(w: &mut World, from: MobileId, skill: SkillName) {
    if creature::cd(w, from).is_some_and(|c| c.is_dead_pet) {
        return;
    }

    if skill == SkillName::Focus && creature::is_creature(w, from) {
        return;
    }

    let base_fixed = skills::base_fixed(w, from, skill);
    let cap_fixed = skills::cap_fixed(w, from, skill);
    if base_fixed < cap_fixed && skills::lock(w, from, skill) == SkillLock::Up {
        let mut to_gain = 1;

        if skills::base(w, from, skill) <= 10.0 {
            to_gain = utility::random(&mut w.rng, 4) + 1;
        }

        let (total, cap, len) = w
            .mobile(from)
            .and_then(|m| m.skills.as_ref())
            .map_or((0, 1, 0), |s| (s.total, s.cap, s.len()));
        let is_player = w.mobile(from).is_some_and(|m| m.player);

        if is_player && f64::from(total) / f64::from(cap) >= utility::random_double(&mut w.rng) {
            for i in 0..len {
                let Some(to_lower) = SkillName::from_i32(i as i32) else {
                    continue;
                };
                if to_lower != skill
                    && skills::lock(w, from, to_lower) == SkillLock::Down
                    && skills::base_fixed(w, from, to_lower) >= to_gain
                {
                    let b = skills::base_fixed(w, from, to_lower);
                    skills::set_base_fixed(w, from, to_lower, (b - to_gain).max(0));
                    break;
                }
            }
        }

        if let Some(d) = player_mobile::data(w, from)
            && skill == d.accelerated_skill
            && d.accelerated_start > w.now
        {
            to_gain *= utility::random_min_max(&mut w.rng, 2, 5);
        }

        let total = w
            .mobile(from)
            .and_then(|m| m.skills.as_ref())
            .map_or(0, |s| s.total);
        if !is_player || total < cap {
            let b = skills::base_fixed(w, from, skill);
            skills::set_base_fixed(w, from, skill, (b + to_gain).min(cap_fixed));
        }
    }

    if settings(w).use_pub45_stat_gain && skills::lock(w, from, skill) == SkillLock::Up {
        let Some((primary_stat, secondary_stat)) = w
            .resources
            .get::<SkillTable>()
            .and_then(|t| t.info.get(skill as usize))
            .map(|i| (i.primary_stat, i.secondary_stat))
        else {
            return;
        };

        let primary_stat_lock = to_lock(w, primary_stat, from);
        let secondary_stat_lock = to_lock(w, secondary_stat, from);

        if primary_stat_lock != StatLockType::Up && secondary_stat_lock != StatLockType::Up {
            return;
        }

        // Flat 1 in 20 chance to gain anything
        if 0.05 * settings(w).stat_gain_chance_multiplier > utility::random_double(&mut w.rng) {
            // 75% for primary, 25% for secondary - Unless primary is not set to gain.
            let chance = settings(w).primary_stat_gain_chance;
            let stat_to_gain = if primary_stat_lock == StatLockType::Up
                && chance > utility::random_double(&mut w.rng)
            {
                primary_stat
            } else {
                secondary_stat
            };

            gain_stat(w, from, stat_to_gain);
        }
    }
}

/// `LegacyGain(from, info)`.
pub fn legacy_gain(w: &mut World, from: MobileId, skill: SkillName) {
    let Some((str_gain, dex_gain, int_gain)) = w
        .resources
        .get::<SkillTable>()
        .and_then(|t| t.info.get(skill as usize))
        .map(|i| (i.str_gain, i.dex_gain, i.int_gain))
    else {
        return;
    };
    let (str_lock, dex_lock, int_lock) = w.mobile(from).map_or(
        (
            StatLockType::Locked,
            StatLockType::Locked,
            StatLockType::Locked,
        ),
        |m| (m.str_lock, m.dex_lock, m.int_lock),
    );

    if str_gain > 0.0 && str_lock == StatLockType::Up && roll_stat_increase_chance(w, str_gain) {
        gain_stat(w, from, Stat::Str);
    }

    if dex_gain > 0.0 && dex_lock == StatLockType::Up && roll_stat_increase_chance(w, dex_gain) {
        gain_stat(w, from, Stat::Dex);
    }

    if int_gain > 0.0 && int_lock == StatLockType::Up && roll_stat_increase_chance(w, int_gain) {
        gain_stat(w, from, Stat::Int);
    }
}

/// `ToLock(stat, from)`.
pub fn to_lock(w: &World, stat: Stat, from: MobileId) -> StatLockType {
    let Some(m) = w.mobile(from) else {
        return StatLockType::Up;
    };
    match stat {
        Stat::Str => m.str_lock,
        Stat::Dex => m.dex_lock,
        Stat::Int => m.int_lock,
    }
}

fn raw(w: &World, from: MobileId, stat: Stat) -> i32 {
    w.mobile(from).map_or(0, |m| match stat {
        Stat::Str => m.str,
        Stat::Dex => m.dex,
        Stat::Int => m.int,
    })
}

fn set_raw(w: &mut World, from: MobileId, stat: Stat, v: i32) {
    match stat {
        Stat::Str => props_b::set_raw_str(w, from, v),
        Stat::Dex => props_b::set_raw_dex(w, from, v),
        Stat::Int => props_b::set_raw_int(w, from, v),
    }
}

/// `CanLower(from, stat)`.
pub fn can_lower(w: &World, from: MobileId, stat: Stat) -> bool {
    to_lock(w, stat, from) == StatLockType::Down && raw(w, from, stat) > 10
}

/// `CanRaise(from, stat)`.
pub fn can_raise(w: &mut World, from: MobileId, stat: Stat) -> bool {
    let controlled = creature::cd(w, from).is_some_and(|c| c.controlled);
    if !controlled {
        let stat_cap = w.mobile(from).map_or(0, |m| m.stat_cap);
        if props_b::raw_stat_total(w, from) >= stat_cap {
            return false;
        }
    }

    to_lock(w, stat, from) == StatLockType::Up && raw(w, from, stat) < settings(w).stat_max
}

/// `IncreaseStat(from, stat, atrophy)`.
pub fn increase_stat(w: &mut World, from: MobileId, stat: Stat, atrophy: bool) {
    let stat_cap = w.mobile(from).map_or(0, |m| m.stat_cap);
    let atrophy = atrophy || props_b::raw_stat_total(w, from) >= stat_cap;
    let stat_max = settings(w).stat_max;

    // The two other stats, in the order they are tested: (first, second).
    let (first, second) = match stat {
        Stat::Str => (Stat::Dex, Stat::Int),
        Stat::Dex => (Stat::Str, Stat::Int),
        Stat::Int => (Stat::Str, Stat::Dex),
    };

    if atrophy {
        if can_lower(w, from, first)
            && (raw(w, from, first) < raw(w, from, second) || !can_lower(w, from, second))
        {
            let v = raw(w, from, first) - 1;
            set_raw(w, from, first, v);
        } else if can_lower(w, from, second) {
            let v = raw(w, from, second) - 1;
            set_raw(w, from, second, v);
        }
    }

    if can_raise(w, from, stat) {
        let v = (raw(w, from, stat) + 1).min(stat_max);
        set_raw(w, from, stat, v);
    }
}

/// `GainStat(from, stat)`.
pub fn gain_stat(w: &mut World, from: MobileId, stat: Stat) {
    let controlled = creature::cd(w, from).is_some_and(|c| c.controlled);
    let (delay, pet_delay) = {
        let s = settings(w);
        (s.stat_gain_delay, s.pet_stat_gain_delay)
    };
    let now: DateTime = w.now;
    let Some(m) = w.mobile_mut(from) else { return };
    let last = match stat {
        Stat::Str => &mut m.last_str_gain,
        Stat::Dex => &mut m.last_dex_gain,
        Stat::Int => &mut m.last_int_gain,
    };
    let wait = if controlled { pet_delay } else { delay };
    if *last + wait >= now {
        return;
    }
    *last = now;

    let stat_cap = w.mobile(from).map_or(1, |m| m.stat_cap);
    let atrophy = f64::from(props_b::raw_stat_total(w, from)) / f64::from(stat_cap)
        >= utility::random_double(&mut w.rng);

    increase_stat(w, from, stat, atrophy);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;
    use muo_server::skills::Skills;

    fn skilled_world() -> (World, MobileId) {
        let mut w = World::new();
        w.resources.get_mut::<SkillTable>().info = muo_data::skill_info::SkillInfo::from_json(
            r#"[{"SkillID":0,"Name":"Alchemy","Title":"Alchemist","StrScale":0,"DexScale":0,"IntScale":0,"StatTotal":0,"StrGain":0,"DexGain":0,"IntGain":0,"GainFactor":1.0,"PrimaryStat":"Str","SecondaryStat":"Dex"}]"#,
        )
        .unwrap_or_default();
        let m = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(m).unwrap().skills = Some(Skills::default());
        (w, m)
    }

    #[test]
    fn a_check_with_no_challenge_or_too_hard_never_rolls() {
        let (mut w, m) = skilled_world();
        // Too difficult: the value is 0, the minimum is 10.
        assert!(!mobile_skill_check_location(
            &mut w,
            m,
            SkillName::Alchemy,
            10.0,
            50.0
        ));
        // No challenge: min >= max.
        assert!(mobile_skill_check_location(
            &mut w,
            m,
            SkillName::Alchemy,
            0.0,
            0.0
        ));
        assert!(!mobile_skill_check_direct_location(
            &mut w,
            m,
            SkillName::Alchemy,
            -0.1
        ));
        assert!(mobile_skill_check_direct_location(
            &mut w,
            m,
            SkillName::Alchemy,
            1.0
        ));
        let t = Targeted::Entity(muo_server::ids::EntityId::Mobile(m));
        assert!(!mobile_skill_check_direct_target(
            &mut w,
            m,
            SkillName::Alchemy,
            t,
            -1.0
        ));
        assert!(mobile_skill_check_direct_target(
            &mut w,
            m,
            SkillName::Alchemy,
            t,
            2.0
        ));
    }

    #[test]
    fn stat_increase_respects_locks_and_the_cap() {
        let (mut w, m) = skilled_world();
        {
            let mob = w.mobile_mut(m).unwrap();
            mob.str = 50;
            mob.dex = 50;
            mob.int = 50;
            mob.stat_cap = 225;
            mob.str_lock = StatLockType::Up;
            mob.dex_lock = StatLockType::Down;
            mob.int_lock = StatLockType::Down;
        }
        settings(&mut w); // default stat max 100
        increase_stat(&mut w, m, Stat::Str, false);
        assert_eq!(raw(&w, m, Stat::Str), 51);
        // At the cap the raise takes from a stat locked down, the lower one first (dex 50 < int 50 is false, so int? no:
        // `RawDex < RawInt` is false and Int can be lowered, so Dex is not chosen by the first branch).
        w.mobile_mut(m).unwrap().stat_cap = 151;
        increase_stat(&mut w, m, Stat::Str, false);
        assert_eq!(
            raw(&w, m, Stat::Str),
            52,
            "at the cap a stat locked down atrophies first, which makes room"
        );
        assert_eq!(
            raw(&w, m, Stat::Int),
            49,
            "atrophy lowers a stat locked down"
        );
        assert!(
            can_lower(&w, m, Stat::Dex) && !can_raise(&mut w, m, Stat::Dex),
            "dex is locked down, not up"
        );
        w.mobile_mut(m).unwrap().str = 100;
        w.mobile_mut(m).unwrap().stat_cap = 500;
        increase_stat(&mut w, m, Stat::Str, false);
        assert_eq!(raw(&w, m, Stat::Str), 100, "capped at stats.statMax");
    }

    #[test]
    fn gain_stat_waits_out_the_delay() {
        let (mut w, m) = skilled_world();
        {
            let mob = w.mobile_mut(m).unwrap();
            mob.str = 50;
            mob.dex = 50;
            mob.int = 50;
            mob.stat_cap = 225;
        }
        w.now = DateTime::from_ymd_hms(2024, 1, 1, 0, 0, 0).unwrap();
        gain_stat(&mut w, m, Stat::Str);
        let first = w.mobile(m).unwrap().last_str_gain;
        assert_eq!(first, w.now);
        gain_stat(&mut w, m, Stat::Str);
        assert_eq!(w.mobile(m).unwrap().last_str_gain, first);
        w.now += TimeSpan::from_mins(11);
        gain_stat(&mut w, m, Stat::Str);
        assert_eq!(w.mobile(m).unwrap().last_str_gain, w.now);
    }

    use std::sync::Mutex;
    use std::sync::atomic::{AtomicU32, Ordering};

    static CALLS: AtomicU32 = AtomicU32::new(0);
    static LAST: Mutex<Option<(MobileId, SkillName, bool)>> = Mutex::new(None);

    fn record(_: &mut World, from: MobileId, skill: SkillName, success: bool) {
        CALLS.fetch_add(1, Ordering::SeqCst);
        *LAST.lock().unwrap() = Some((from, skill, success));
    }

    /// SkillEventsTests: every handler raises once per attempt, short circuits included, with the outcome returned.
    #[test]
    fn skill_used_is_raised_once_per_attempt_with_the_outcome() {
        let (mut w, m) = skilled_world();
        skill_events::subscribe_skill_used(&mut w, record);
        let last = || LAST.lock().unwrap().unwrap();
        let t = Targeted::Entity(muo_server::ids::EntityId::Mobile(m));

        // A rolled attempt.
        let rolled = mobile_skill_check_direct_target(&mut w, m, SkillName::Alchemy, t, 0.5);
        assert_eq!(CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(last(), (m, SkillName::Alchemy, rolled));

        // The short circuits.
        assert!(mobile_skill_check_direct_location(
            &mut w,
            m,
            SkillName::Alchemy,
            1.0
        ));
        assert_eq!(CALLS.load(Ordering::SeqCst), 2);
        assert!(last().2);

        assert!(!mobile_skill_check_direct_target(
            &mut w,
            m,
            SkillName::Alchemy,
            t,
            -0.1
        ));
        assert_eq!(CALLS.load(Ordering::SeqCst), 3);
        assert!(!last().2);

        assert!(!mobile_skill_check_location(
            &mut w,
            m,
            SkillName::Alchemy,
            50.0,
            100.0
        ));
        assert_eq!(CALLS.load(Ordering::SeqCst), 4);
        assert!(!last().2);

        assert!(mobile_skill_check_target(
            &mut w,
            m,
            SkillName::Alchemy,
            t,
            0.0,
            0.0
        ));
        assert_eq!(CALLS.load(Ordering::SeqCst), 5);
        assert!(last().2);
    }

    // ---- SkillEventsTests, one test each (the recorder is per thread, so each test has its own) ----

    type Recorded = (u32, Option<(MobileId, SkillName, bool)>);

    thread_local! {
        static REC: std::cell::RefCell<Recorded> = const { std::cell::RefCell::new((0, None)) };
    }

    fn rec(_: &mut World, from: MobileId, skill: SkillName, success: bool) {
        REC.with(|r| {
            let n = r.borrow().0 + 1;
            *r.borrow_mut() = (n, Some((from, skill, success)));
        });
    }

    fn calls() -> u32 {
        REC.with(|r| r.borrow().0)
    }

    fn rec_last() -> (MobileId, SkillName, bool) {
        REC.with(|r| r.borrow().1.unwrap())
    }

    #[test]
    fn direct_target_rolled_attempt_raises_once_with_the_returned_outcome() {
        let (mut w, from) = skilled_world();
        skill_events::subscribe_skill_used(&mut w, rec);
        let null_target = Targeted::Entity(muo_server::ids::EntityId::Mobile(from)); // stands in for a null target

        let rolled =
            mobile_skill_check_direct_target(&mut w, from, SkillName::Mining, null_target, 0.5);
        assert_eq!(calls(), 1);
        assert_eq!(rec_last(), (from, SkillName::Mining, rolled));

        assert!(!mobile_skill_check_direct_target(
            &mut w,
            from,
            SkillName::Mining,
            null_target,
            0.0
        ));
        assert_eq!(calls(), 2);
        assert!(!rec_last().2);
    }

    #[test]
    fn short_circuits_still_raise_with_the_handler_outcome() {
        let (mut w, from) = skilled_world();
        skill_events::subscribe_skill_used(&mut w, rec);
        let null_target = Targeted::Entity(muo_server::ids::EntityId::Mobile(from)); // stands in for a null target

        assert!(mobile_skill_check_direct_location(
            &mut w,
            from,
            SkillName::Mining,
            1.0
        ));
        assert_eq!(calls(), 1);
        assert!(rec_last().2);

        assert!(!mobile_skill_check_direct_target(
            &mut w,
            from,
            SkillName::Mining,
            null_target,
            -0.1
        ));
        assert_eq!(calls(), 2);
        assert!(!rec_last().2);

        assert!(!mobile_skill_check_location(
            &mut w,
            from,
            SkillName::Mining,
            50.0,
            100.0
        ));
        assert_eq!(calls(), 3);
        assert!(!rec_last().2);

        assert!(mobile_skill_check_target(
            &mut w,
            from,
            SkillName::Mining,
            null_target,
            0.0,
            0.0
        ));
        assert_eq!(calls(), 4);
        assert!(rec_last().2);
    }
}
