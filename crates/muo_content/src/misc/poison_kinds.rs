//! Poisons: `PoisonImpl` and its timer, and the poison kinds (the registrations and lookups).
//!
//! `muo_server::poison` holds the abstract `Poison` and the registry; this registers the concrete poisons and
//! gives each its timer.

use indexmap::IndexMap;
use muo_primitives::{MessageType, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::messages;
use muo_server::poison::{Poison, PoisonFamily, PoisonId, Poisons};
use muo_server::timer::{self, TimerHandle};

use super::aos_helpers::{self, DamageOptions, DamageSplit};

/// The constants of one `PoisonImpl`.
#[derive(Copy, Clone, Debug)]
pub struct PoisonParams {
    pub minimum: i32,
    pub maximum: i32,
    /// `percent * 0.01`.
    pub scalar: f64,
    pub delay: f64,
    pub interval: f64,
    pub count: i32,
    pub message_interval: i32,
}

/// `PoisonTimer.From` per poisoned mobile; `PlayerMobile.ApplyPoison` replaces it after the base applied the poison.
#[derive(Default)]
pub struct PoisonSources(pub IndexMap<MobileId, Option<MobileId>>);

/// `PoisonTimer`'s `_index` and `_lastDamage`, which live as long as the timer.
#[derive(Default)]
struct TickState {
    index: i32,
    last_damage: i32,
}

/// What the timer asks of other systems: `TransformationSpellHelper.UnderTransformation(m,
/// VampiricEmbraceSpell)`, `OrangePetals.UnderEffect(m)` and `AnimalForm.UnderTransformation(m, Unicorn)`.
#[derive(Default)]
pub struct PoisonHooks {
    pub vampiric_embrace: Option<fn(&World, MobileId) -> bool>,
    pub orange_petals: Option<fn(&World, MobileId) -> bool>,
    pub unicorn_form: Option<fn(&World, MobileId) -> bool>,
}

fn params(w: &World, id: PoisonId) -> Option<PoisonParams> {
    w.resources.get::<PoisonTable>()?.0.get(&id.0).copied()
}

/// `PoisonImpl` constants by poison index.
#[derive(Default)]
struct PoisonTable(IndexMap<u8, PoisonParams>);

/// `PoisonImpl.ConstructTimer`: makes (does not start) the timer; delay and interval from the constants.
fn construct_timer(w: &mut World, m: MobileId) -> TimerHandle {
    let Some(poison) = w.mobile(m).and_then(|x| x.poison) else {
        return TimerHandle::default();
    };
    let Some(p) = params(w, poison) else {
        return TimerHandle::default();
    };
    w.resources.get_mut::<PoisonSources>().0.insert(m, Some(m)); // From = m
    let mut state = TickState::default();
    timer::create(
        w,
        TimeSpan::from_secs_f64(p.delay),
        TimeSpan::from_secs_f64(p.interval),
        0,
        move |w| on_tick(w, m, poison, p, &mut state),
    )
}

fn family(w: &World, id: PoisonId) -> PoisonFamily {
    muo_server::poison::find(w, id).map_or(PoisonFamily::Standard, |p| p.family)
}

fn hook(
    w: &World,
    f: impl Fn(&PoisonHooks) -> Option<fn(&World, MobileId) -> bool>,
    m: MobileId,
) -> bool {
    w.resources
        .get::<PoisonHooks>()
        .and_then(f)
        .is_some_and(|f| f(w, m))
}

/// `PoisonImpl.PoisonTimer.OnTick`.
fn on_tick(w: &mut World, mobile: MobileId, poison: PoisonId, p: PoisonParams, st: &mut TickState) {
    let stop_self = |w: &mut World| {
        if let Some(h) = timer::current(w) {
            timer::stop(w, h);
        }
    };
    let Some(mob) = w.mobile(mobile) else {
        stop_self(w);
        return;
    };
    let (ty, name) = (mob.ty, (mob.ty.name)(w, mobile).unwrap_or_default());
    let level = muo_server::poison::level(w, poison);
    let from = w
        .resources
        .get::<PoisonSources>()
        .and_then(|s| s.0.get(&mobile).copied())
        .flatten();

    let resists = (crate::core::aos(w) && level < 4 && hook(w, |h| h.vampiric_embrace, mobile))
        || (level < 3 && hook(w, |h| h.orange_petals, mobile))
        || hook(w, |h| h.unicorn_form, mobile);
    if resists && (ty.cure_poison)(w, mobile, Some(mobile)) {
        if crate::core::sa(w) {
            // * You feel yourself resisting the effects of the poison *
            messages::local_overhead_message_number(
                w,
                mobile,
                MessageType::EMOTE,
                0x3F,
                1114441,
                "",
            );
            // * ~1_NAME~ seems resistant to the poison *
            messages::nonlocal_overhead_message_number(
                w,
                mobile,
                MessageType::EMOTE,
                0x3F,
                1114442,
                &name,
            );
        } else {
            messages::local_overhead_message(
                w,
                mobile,
                MessageType::EMOTE,
                0x3F,
                true,
                "* You feel yourself resisting the effects of the poison *",
            );
            messages::local_overhead_message(
                w,
                mobile,
                MessageType::EMOTE,
                0x3F,
                true,
                &format!("* {name} seems resistant to the poison *"),
            );
        }
        stop_self(w);
        return;
    }

    let idx = st.index;
    st.index += 1;
    if idx == p.count {
        messages::send_localized_message(w, mobile, 502136, "", 0x3B2); // The poison seems to have worn off.
        muo_server::mobile::part1::props_b::set_poison(w, mobile, None);
        stop_self(w);
        return;
    }

    let mut damage;
    if !crate::core::aos(w)
        && st.last_damage != 0
        && muo_primitives::utility::random_bool(&mut w.rng)
    {
        damage = st.last_damage;
    } else {
        let hits = w.mobile(mobile).map_or(0, |m| m.hits);
        damage = 1 + (f64::from(hits) * p.scalar) as i32;
        damage = damage.clamp(p.minimum, p.maximum);
        st.last_damage = damage;
    }

    let fam = family(w, poison);
    let in_range_1 = |w: &World, from: MobileId| {
        let (Some(a), Some(b)) = (w.mobile(from), w.mobile(mobile)) else {
            return false;
        };
        a.map == b.map
            && (a.location.x - b.location.x).abs() <= 1
            && (a.location.y - b.location.y).abs() <= 1
    };
    let same_map = |w: &World, from: MobileId| matches!((w.mobile(from), w.mobile(mobile)), (Some(a), Some(b)) if a.map == b.map);

    // Darkglow: 10% damage boost when attacker is more than 1 tile away
    if fam == PoisonFamily::Darkglow
        && let Some(f) = from
        && same_map(w, f)
        && !in_range_1(w, f)
    {
        damage = (f64::from(damage) * 1.1) as i32;
        messages::send_localized_message(w, f, 1072850, "", 0x3B2); // Darkglow poison increases your damage!
    }

    if let Some(f) = from {
        muo_server::mobile::base::do_harmful(w, f, mobile, true);
    }
    crate::player_mobile::hooks::honor_on_target_poisoned(w, mobile); // `ReceivedHonorContext?.OnTargetPoisoned()`
    aos_helpers::damage(
        w,
        mobile,
        from,
        damage,
        DamageSplit {
            pois: 100,
            ..Default::default()
        },
        DamageOptions::default(),
    );

    // Parasitic: heals attacker for damage dealt when within 1 tile
    if fam == PoisonFamily::Parasitic
        && let Some(f) = from
        && same_map(w, f)
        && in_range_1(w, f)
    {
        muo_server::mobile::part2::damage::heal(w, f, damage);
        // You have had ~1_HEALED_AMOUNT~ hit points healed.
        messages::send_localized_message(w, f, 1060203, &damage.to_string(), 0x3B2);
    }

    // OSI: randomly revealed between first and third damage tick, guessing 60% chance
    if muo_primitives::utility::random_double(&mut w.rng) < 0.40 {
        (ty.revealing_action)(w, mobile);
    }

    if st.index % p.message_interval == 0 {
        (ty.on_poisoned)(w, mobile, from, poison, Some(poison));
    }
}

/// `timer.From = from` of PlayerMobile.ApplyPoison.
pub fn set_source(w: &mut World, victim: MobileId, from: MobileId) {
    if w.mobile(victim)
        .is_some_and(|m| m.poison_timer != TimerHandle::default())
    {
        w.resources
            .get_mut::<PoisonSources>()
            .0
            .insert(victim, Some(from));
    }
}

#[allow(clippy::too_many_arguments)] // one parameter per poison property
fn register_one(
    w: &mut World,
    name: &'static str,
    index: i32,
    level: i32,
    (min, max): (i32, i32),
    percent: f64,
    (delay, interval): (f64, f64),
    count: i32,
    message_interval: i32,
    family: PoisonFamily,
) {
    let reg = Poison {
        index,
        name,
        level,
        family,
        construct_timer,
    };
    if w.resources.get_mut::<Poisons>().register(reg).is_ok() {
        w.resources.get_mut::<PoisonTable>().0.insert(
            index as u8,
            PoisonParams {
                minimum: min,
                maximum: max,
                scalar: percent * 0.01,
                delay,
                interval,
                count,
                message_interval,
            },
        );
    }
}

/// `PoisonKinds.Configure` (`[CallPriority(10)]`).
pub fn configure(w: &mut World) {
    use PoisonFamily::{Darkglow, Parasitic, Standard};
    let r = register_one;
    if crate::core::aos(w) {
        r(
            w,
            "Lesser",
            0,
            0,
            (4, 16),
            7.5,
            (3.0, 2.25),
            10,
            4,
            Standard,
        );
        r(
            w,
            "Regular",
            1,
            1,
            (8, 18),
            10.0,
            (3.0, 3.25),
            10,
            3,
            Standard,
        );
        r(
            w,
            "Greater",
            2,
            2,
            (12, 20),
            15.0,
            (3.0, 4.25),
            10,
            2,
            Standard,
        );
        r(
            w,
            "Deadly",
            3,
            3,
            (16, 30),
            30.0,
            (3.0, 5.25),
            15,
            2,
            Standard,
        );
        r(
            w,
            "Lethal",
            4,
            4,
            (20, 50),
            35.0,
            (3.0, 5.25),
            20,
            2,
            Standard,
        );
    } else {
        r(w, "Lesser", 0, 0, (4, 26), 2.5, (3.5, 3.0), 10, 2, Standard);
        r(
            w,
            "Regular",
            1,
            1,
            (5, 26),
            3.125,
            (3.5, 3.0),
            10,
            2,
            Standard,
        );
        r(
            w,
            "Greater",
            2,
            2,
            (6, 26),
            6.25,
            (3.5, 3.0),
            10,
            2,
            Standard,
        );
        r(
            w,
            "Deadly",
            3,
            3,
            (7, 26),
            12.5,
            (3.5, 4.0),
            10,
            2,
            Standard,
        );
        r(
            w,
            "Lethal",
            4,
            4,
            (9, 26),
            25.0,
            (3.5, 5.0),
            10,
            2,
            Standard,
        );
    }
    if crate::core::ml(w) {
        r(
            w,
            "LesserDarkglow",
            10,
            0,
            (4, 16),
            7.5,
            (3.0, 2.25),
            10,
            4,
            Darkglow,
        );
        r(
            w,
            "RegularDarkglow",
            11,
            1,
            (8, 18),
            10.0,
            (3.0, 3.25),
            10,
            3,
            Darkglow,
        );
        r(
            w,
            "GreaterDarkglow",
            12,
            2,
            (12, 20),
            15.0,
            (3.0, 4.25),
            10,
            2,
            Darkglow,
        );
        r(
            w,
            "DeadlyDarkglow",
            13,
            3,
            (16, 30),
            30.0,
            (3.0, 5.25),
            15,
            2,
            Darkglow,
        );
        r(
            w,
            "LesserParasitic",
            20,
            0,
            (4, 16),
            7.5,
            (3.0, 2.25),
            10,
            4,
            Parasitic,
        );
        r(
            w,
            "RegularParasitic",
            21,
            1,
            (8, 18),
            10.0,
            (3.0, 3.25),
            10,
            3,
            Parasitic,
        );
        r(
            w,
            "GreaterParasitic",
            22,
            2,
            (12, 20),
            15.0,
            (3.0, 4.25),
            10,
            2,
            Parasitic,
        );
        r(
            w,
            "DeadlyParasitic",
            23,
            3,
            (16, 30),
            30.0,
            (3.0, 5.25),
            15,
            2,
            Parasitic,
        );
        r(
            w,
            "LethalParasitic",
            24,
            4,
            (20, 50),
            35.0,
            (3.0, 5.25),
            20,
            2,
            Parasitic,
        );
    }
}

/// `Poison.GetPoison(level)` of PoisonKinds: the standard poison of that level.
pub fn get_poison_by_level(w: &World, level: i32) -> Option<PoisonId> {
    get_poison_by_family_and_level(w, PoisonFamily::Standard, level)
}

/// `Poison.GetPoisonByFamilyAndLevel`.
pub fn get_poison_by_family_and_level(
    w: &World,
    family: PoisonFamily,
    level: i32,
) -> Option<PoisonId> {
    w.resources
        .get::<Poisons>()?
        .poisons
        .iter()
        .find(|p| p.family == family && p.level == level)
        .map(Poison::id)
}

/// `Poison.Lesser` .. `Poison.Lethal`, `Poison.LesserDarkglow` ..: a poison by its registered name.
pub fn by_name(w: &World, name: &str) -> Option<PoisonId> {
    w.resources
        .get::<Poisons>()?
        .get_poison(name)
        .map(Poison::id)
}

/// `Poison.IsDarkglow`.
pub fn is_darkglow(w: &World, id: PoisonId) -> bool {
    family(w, id) == PoisonFamily::Darkglow
}

/// `Poison.IsParasitic`.
pub fn is_parasitic(w: &World, id: PoisonId) -> bool {
    family(w, id) == PoisonFamily::Parasitic
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_aos_poisons_and_lookups() {
        let mut w = World::new();
        configure(&mut w);
        let lethal = by_name(&w, "lethal").unwrap();
        assert_eq!(muo_server::poison::level(&w, lethal), 4);
        assert_eq!(get_poison_by_level(&w, 2), by_name(&w, "Greater"));
        assert_eq!(
            get_poison_by_family_and_level(&w, PoisonFamily::Parasitic, 4),
            by_name(&w, "LethalParasitic")
        );
        assert!(is_darkglow(&w, by_name(&w, "DeadlyDarkglow").unwrap()));
        let p = params(&w, lethal).unwrap();
        assert_eq!((p.minimum, p.maximum, p.count), (20, 50, 20));
    }
}
