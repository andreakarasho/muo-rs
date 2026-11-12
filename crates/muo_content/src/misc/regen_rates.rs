//! Hit, stamina and mana regeneration rates.
//!
//! The regen handlers of `Mobile` take `&World` (muo_server `RegenRateHandler`), but the rates also call
//! `CheckBonusSkill` -> `Mobile.CheckSkill` from inside them, which rolls a skill gain and needs `&mut World`.
//! `Part2Statics::regen_pre_rate` ([`pre_rate`]) runs those checks right before the regen timers ask a handler
//! for the rate: stamina checks Focus, mana checks Meditation (when not meditating) and, under AOS, Focus.

use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::statics::Part2Statics;
use muo_server::skills::SkillName;

use crate::aos::AosAttribute;
use crate::misc::aos_helpers::attribute_value;
use crate::player_mobile::{PLAYER_MOBILE, RACE_HUMAN};

/// `Skill.Value` read from a `&World` (`skills::value_ref`).
pub fn skill_value(w: &World, id: MobileId, skill: SkillName) -> f64 {
    muo_server::skills::value_ref(w, id, skill)
}

fn is_player_mobile(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.ty.is(&PLAYER_MOBILE))
}

/// What the rates ask of systems in other crates: `TransformationSpellHelper.UnderTransformation` for
/// HorrificBeast, VampiricEmbrace and LichForm, `AnimalForm.UnderTransformation` for Dog, Cat and Kirin, and the
/// creature checks (`IsAnimatedDead`, `IsParagon`, `Leviathan`) and armor meditation penalty.
#[derive(Default)]
pub struct RegenHooks {
    pub horrific_beast: Option<fn(&World, MobileId) -> bool>,
    pub vampiric_embrace: Option<fn(&World, MobileId) -> bool>,
    pub lich_form: Option<fn(&World, MobileId) -> bool>,
    pub dog_or_cat_form: Option<fn(&World, MobileId) -> bool>,
    pub kirin_form: Option<fn(&World, MobileId) -> bool>,
    /// `bc?.IsAnimatedDead == false`: a BaseCreature that isn't animated dead.
    pub creature_not_animated_dead: Option<fn(&World, MobileId) -> bool>,
    /// `bc.IsParagon || from is Leviathan`.
    pub paragon_or_leviathan: Option<fn(&World, MobileId) -> bool>,
    /// `RegenRates.GetArmorOffset(from)`.
    pub armor_offset: Option<fn(&World, MobileId) -> f64>,
}

fn hook(
    w: &World,
    f: impl Fn(&RegenHooks) -> Option<fn(&World, MobileId) -> bool>,
    m: MobileId,
) -> bool {
    w.resources
        .get::<RegenHooks>()
        .and_then(f)
        .is_some_and(|f| f(w, m))
}

/// `CheckBonusSkill(m, cur, max, skill)`: the chance rises with how full the pool is.
pub fn check_bonus_skill(w: &mut World, m: MobileId, cur: i32, max: i32, skill: SkillName) {
    let alive = w.mobile(m).is_some_and(|x| (x.ty.alive)(w, m));
    if !alive {
        return;
    }
    let mut n = f64::from(cur) / f64::from(max);
    let v = (skill_value(w, m, skill) * 0.005).sqrt();
    n *= 1.0 - v;
    n += v;
    muo_server::mobile::part2::stats::check_skill_chance(w, m, skill, n);
}

/// The `CheckBonusSkill` calls of `Mobile_StamRegenRate` and `Mobile_ManaRegenRate` (`which`: 1 stamina, 2 mana).
pub fn pre_rate(w: &mut World, from: MobileId, which: i32) {
    let Some(m) = w.mobile(from) else { return };
    if m.skills.is_none() {
        return;
    }
    match which {
        1 => {
            let (cur, max) = (m.stam, (m.ty.stam_max)(w, from));
            check_bonus_skill(w, from, cur, max, SkillName::Focus);
        }
        2 => {
            let (cur, max, meditating) = (m.mana, (m.ty.mana_max)(w, from), m.meditating);
            if !meditating {
                check_bonus_skill(w, from, cur, max, SkillName::Meditation);
            }
            if crate::core::aos(w) {
                check_bonus_skill(w, from, cur, max, SkillName::Focus);
            }
        }
        _ => {}
    }
}

/// `Mobile_HitsRegenRate`.
fn hits_regen_rate(w: &World, from: MobileId) -> TimeSpan {
    let mut points = attribute_value(w, from, AosAttribute::RegenHits);
    if hook(w, |h| h.creature_not_animated_dead, from) {
        points += 4;
    }
    if hook(w, |h| h.paragon_or_leviathan, from) {
        points += 40;
    }
    if crate::core::ml(w) && is_player_mobile(w, from) {
        // does racial bonus go before/after?
        if w.mobile(from).and_then(|m| m.race).unwrap_or(RACE_HUMAN) == RACE_HUMAN {
            points += 2;
        }
        points = points.min(18);
    }
    if points < 0 {
        points = 0;
    }
    if hook(w, |h| h.horrific_beast, from) {
        points += 20;
    }
    if hook(w, |h| h.dog_or_cat_form, from) {
        points += (skill_value(w, from, SkillName::Ninjitsu) / 3.0) as i32;
    }
    TimeSpan::from_secs_f64(10.0 / f64::from(1 + points))
}

fn default_rate(w: &World, f: impl Fn(&Part2Statics) -> TimeSpan) -> TimeSpan {
    w.resources.get::<Part2Statics>().map_or(TimeSpan::ZERO, f)
}

/// `Mobile_StamRegenRate`.
fn stam_regen_rate(w: &World, from: MobileId) -> TimeSpan {
    if w.mobile(from).is_none_or(|m| m.skills.is_none()) {
        return default_rate(w, |s| s.default_stam_rate);
    }
    // CheckBonusSkill(Focus): `pre_rate`.
    let mut points = (skill_value(w, from, SkillName::Focus) * 0.1) as i32;
    if hook(w, |h| h.paragon_or_leviathan, from) {
        points += 40;
    }
    let mut capped = attribute_value(w, from, AosAttribute::RegenStam);
    if hook(w, |h| h.vampiric_embrace, from) {
        capped += 15;
    }
    if hook(w, |h| h.kirin_form, from) {
        capped += 20;
    }
    if crate::core::ml(w) && is_player_mobile(w, from) {
        capped = capped.min(24);
    }
    points += capped;
    if points < -1 {
        points = -1;
    }
    TimeSpan::from_secs_f64(1.0 / (0.1 * f64::from(2 + points)))
}

/// `Mobile_ManaRegenRate`.
fn mana_regen_rate(w: &World, from: MobileId) -> TimeSpan {
    let Some(m) = w.mobile(from) else {
        return TimeSpan::ZERO;
    };
    if m.skills.is_none() {
        return default_rate(w, |s| s.default_mana_rate);
    }
    // CheckBonusSkill(Meditation) when not meditating, and Focus under AOS: `pre_rate`.
    let meditating = m.meditating;
    let int = f64::from((m.ty.int)(w, from));
    let meditation = skill_value(w, from, SkillName::Meditation);
    let armor_penalty = w
        .resources
        .get::<RegenHooks>()
        .and_then(|h| h.armor_offset)
        .map_or(0.0, |f| f(w, from));

    let rate = if crate::core::aos(w) {
        let mut med_points = int + meditation * 3.0;
        med_points *= if meditation < 100.0 { 0.025 } else { 0.0275 };
        let focus_points = skill_value(w, from, SkillName::Focus) * 0.05;
        if armor_penalty > 0.0 {
            med_points = 0.0; // In AOS, wearing any meditation-blocking armor completely removes meditation bonus
        }
        let mut total = focus_points
            + med_points
            + if meditating {
                if med_points > 13.0 { 13.0 } else { med_points }
            } else {
                0.0
            };
        if hook(w, |h| h.paragon_or_leviathan, from) {
            total += 40.0;
        }
        let mut capped = attribute_value(w, from, AosAttribute::RegenMana);
        if hook(w, |h| h.vampiric_embrace, from) {
            capped += 3;
        } else if hook(w, |h| h.lich_form, from) {
            capped += 13;
        }
        if crate::core::ml(w) && is_player_mobile(w, from) {
            capped = capped.min(18);
        }
        total += f64::from(capped);
        if total < -1.0 {
            total = -1.0;
        }
        if crate::core::ml(w) {
            total = total.floor();
        }
        1.0 / (0.1 * (2.0 + total))
    } else {
        let med_points = (int + meditation) * 0.5;
        let mut r = if med_points <= 0.0 {
            7.0
        } else if med_points <= 100.0 {
            7.0 - 239.0 * med_points / 2400.0 + 19.0 * med_points * med_points / 48000.0
        } else if med_points < 120.0 {
            1.0
        } else {
            0.75
        };
        r += armor_penalty;
        if meditating {
            r *= 0.5;
        }
        r.clamp(0.5, 7.0)
    };
    TimeSpan::from_secs_f64(rate)
}

/// `RegenRates.Configure` (`[CallPriority(10)]`).
pub fn configure(w: &mut World) {
    let aos = crate::core::aos(w);
    let s = w.resources.get_mut::<Part2Statics>();
    s.default_hits_rate = TimeSpan::from_secs_f64(11.0);
    s.default_stam_rate = TimeSpan::from_secs_f64(7.0);
    s.default_mana_rate = TimeSpan::from_secs_f64(7.0);
    s.mana_regen_rate_handler = Some(mana_regen_rate);
    s.regen_pre_rate = Some(pre_rate);
    if aos {
        s.stam_regen_rate_handler = Some(stam_regen_rate);
        s.hits_regen_rate_handler = Some(hits_regen_rate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configure_sets_defaults_and_aos_handlers() {
        let mut w = World::new();
        configure(&mut w);
        let s = w.resources.get::<Part2Statics>().unwrap();
        assert_eq!(s.default_hits_rate, TimeSpan::from_secs(11));
        assert!(s.mana_regen_rate_handler.is_some() && s.hits_regen_rate_handler.is_some());
    }

    #[test]
    fn hits_rate_is_ten_over_one_plus_points() {
        let mut w = World::new();
        let id = crate::player_mobile::new_player_mobile(&mut w);
        // ML player, human: 0 + 2 points -> 10/3 seconds
        assert_eq!(hits_regen_rate(&w, id), TimeSpan::from_secs_f64(10.0 / 3.0));
    }
}
