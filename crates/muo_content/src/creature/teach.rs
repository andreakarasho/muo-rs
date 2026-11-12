//! The teaching members of BaseCreature: `CheckTeachSkills`, `Teach`, the Teach context-menu
//! entries, and the "train" speech request of a town NPC (`HandleTraining`).

use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::MobileId;
use muo_server::mobile::part2::messages::{self, AffixType};
use muo_server::skills::{self, SkillLock, SkillName};
use muo_server::world::World;

use super::data::{cd_mut, read};
use super::enums::TeachResult;
use super::vtable::ctype;
use crate::hooks::pet_player_hooks;
use crate::mx;

/// The stored skill without the first-touch creation (`Skills[name]` for a read).
pub fn rd(w: &World, id: MobileId, s: SkillName) -> skills::Skill {
    w.mobile(id)
        .and_then(|m| m.skills.as_ref())
        .and_then(|k| k.peek(s as usize))
        .copied()
        .unwrap_or_default()
}

fn skill_count() -> i32 {
    (0..)
        .take_while(|&i| SkillName::from_i32(i).is_some())
        .count() as i32
}

fn lock_of(w: &mut World, id: MobileId, s: SkillName) -> SkillLock {
    skills::lock(w, id, s)
}

/// 4718 `CheckTeachSkills(skill, m, maxPointsToLearn, ref pointsToLearn, doTeach)`.
pub fn check_teach_skills(
    w: &mut World,
    id: MobileId,
    skill: SkillName,
    m: MobileId,
    max_points_to_learn: i32,
    points_to_learn: &mut i32,
    do_teach: bool,
) -> TeachResult {
    let c = ctype(w, id).expect("creature");
    if !(c.check_teach)(w, id, skill as i32, m)
        || !muo_server::mobile::part2::stats::check_alive(w, m, true)
    {
        return TeachResult::Failure;
    }
    let our_base = skills::base_fixed(w, id, skill);
    let their_base = skills::base_fixed(w, m, skill);
    let their_cap = skills::cap_fixed(w, m, skill);
    let mut base_to_set = our_base / 3;
    if base_to_set > 420 {
        base_to_set = 420;
    } else if base_to_set < 200 {
        return TeachResult::Failure;
    }
    if base_to_set > their_cap {
        base_to_set = their_cap;
    }
    *points_to_learn = base_to_set - their_base;
    if max_points_to_learn > 0 && *points_to_learn > max_points_to_learn {
        *points_to_learn = max_points_to_learn;
        base_to_set = their_base + *points_to_learn;
    }
    if *points_to_learn < 0 {
        return TeachResult::KnowsMoreThanMe;
    }
    if *points_to_learn == 0 {
        return TeachResult::KnowsWhatIKnow;
    }
    if lock_of(w, m, skill) != SkillLock::Up {
        return TeachResult::SkillNotRaisable;
    }
    let (cap, total) = w
        .mobile(m)
        .and_then(|x| x.skills.as_ref())
        .map_or((0, 0), |s| (s.cap, s.total));
    let free_points = (cap - total).max(0);
    let mut freeable_points = 0;
    let n = skill_count();
    let mut i = 0;
    while free_points + freeable_points < *points_to_learn && i < n {
        if let Some(sk) = SkillName::from_i32(i).filter(|&sk| sk != skill)
            && lock_of(w, m, sk) == SkillLock::Down
        {
            freeable_points += skills::base_fixed(w, m, sk);
        }
        i += 1;
    }
    if free_points + freeable_points == 0 {
        return TeachResult::NotEnoughFreePoints;
    }
    if free_points + freeable_points < *points_to_learn {
        *points_to_learn = free_points + freeable_points;
        base_to_set = their_base + *points_to_learn;
    }
    if do_teach {
        let mut need = *points_to_learn - free_points;
        let mut i = 0;
        while need > 0 && i < n {
            if let Some(sk) = SkillName::from_i32(i).filter(|&sk| sk != skill)
                && lock_of(w, m, sk) == SkillLock::Down
            {
                let b = skills::base_fixed(w, m, sk);
                if b < need {
                    need -= b;
                    skills::set_base_fixed(w, m, sk, 0);
                } else {
                    skills::set_base_fixed(w, m, sk, b - need);
                    need = 0;
                }
            }
            i += 1;
        }
        let total = skills::total(w, m);
        if base_to_set > their_cap || total - skills::base_fixed(w, m, skill) + base_to_set > cap {
            return TeachResult::NotEnoughFreePoints;
        }
        skills::set_base_fixed(w, m, skill, base_to_set);
    }
    TeachResult::Success
}

fn set_learning(w: &mut World, m: MobileId, value: i32) {
    if crate::factory::mobile_is(w, m, "Server.Mobiles.PlayerMobile")
        && let Some(f) = pet_player_hooks(w).and_then(|h| h.set_learning)
    {
        f(w, m, value);
    }
}

/// 4853 `Teach(skill, m, maxPointsToLearn, doTeach)`.
pub fn teach(
    w: &mut World,
    id: MobileId,
    skill: SkillName,
    m: MobileId,
    max_points_to_learn: i32,
    do_teach: bool,
) -> bool {
    let mut points = 0;
    match check_teach_skills(w, id, skill, m, max_points_to_learn, &mut points, do_teach) {
        TeachResult::KnowsMoreThanMe => messages::say_number(w, id, 501508, ""), // I cannot teach thee, for thou knowest more than I!
        TeachResult::KnowsWhatIKnow => messages::say_number(w, id, 501509, ""), // I cannot teach thee, for thou knowest all I can teach!
        TeachResult::NotEnoughFreePoints | TeachResult::SkillNotRaisable => {
            // Make sure this skill is marked to raise. If you are near the skill cap (700 points) you may need to lose
            // some points in another skill first.
            messages::send_localized_message(w, m, 501510, "", 0x22);
        }
        TeachResult::Success => {
            if do_teach {
                messages::say_number(w, id, 501539, ""); // Let me show thee something of how this is done.
                mx::msg(w, m, 501540); // Your skill level increases.
                if let Some(d) = cd_mut(w, id) {
                    d.teaching = -1;
                }
                set_learning(w, m, -1);
            } else {
                // I will teach thee all I know, if paid the amount in full.  The price is:
                messages::say_affix(w, id, 1019077, AffixType::APPEND, &format!(" {points}"), "");
                messages::say_number(w, id, 1043108, ""); // For less I shall teach thee less.
                if let Some(d) = cd_mut(w, id) {
                    d.teaching = skill as i32;
                }
                set_learning(w, m, skill as i32);
            }
            return true;
        }
        _ => {}
    }
    false
}

/// `BaseCreature.GetContextMenuEntries`' loop: a `TeachEntry` per skill the creature can teach `from`.
pub fn add_teach_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    let c = ctype(w, id).expect("creature");
    if !(c.can_teach)(w, id) || !mx::alive(w, from) {
        return;
    }
    for i in 0..skill_count() {
        let Some(sk) = SkillName::from_i32(i) else {
            continue;
        };
        if f64::from(rd(w, id, sk).base) / 10.0 >= 60.0 && (c.check_teach)(w, id, i, from) {
            let to_teach = (i32::from(rd(w, id, sk).base) / 3).min(420);
            let enabled = to_teach > i32::from(rd(w, from, sk).base);
            list.push(crate::systems::context_menus::entries::teach(sk, enabled));
        }
    }
}

/// `HandleTraining(from)` of a town NPC asked to "train".
pub fn handle_training(w: &mut World, id: MobileId, from: MobileId) {
    let c = ctype(w, id).expect("creature");
    let mut found = false;
    for i in 0..skill_count() {
        let Some(sk) = SkillName::from_i32(i) else {
            continue;
        };
        let base = f64::from(rd(w, id, sk).base) / 10.0;
        if base < 60.0 || !(c.check_teach)(w, id, i, from) {
            continue;
        }
        let to_teach = (base / 3.0).min(42.0);
        if to_teach <= f64::from(rd(w, from, sk).base) / 10.0 {
            continue;
        }
        let number = 1043059 + i; // alchemy
        if number > 1043107 {
            continue; // disarming traps
        }
        if !found {
            messages::say_number(w, id, 1043058, ""); // I can train the following:
            found = true;
        }
        messages::say_number(w, id, number, "");
    }
    if !found {
        messages::say_number(w, id, 501505, ""); // Alas, I cannot teach thee anything.
    }
}

/// `ContextMenuHooks::creature_teach`.
pub fn teach_hook(w: &mut World, bc: MobileId, skill: SkillName, from: MobileId) -> bool {
    if !super::is_creature(w, bc) {
        return false;
    }
    teach(w, bc, skill, from, 0, false);
    true
}

/// `OnGoldGiven`'s first branch: `Teach(m_Teaching, from, dropped.Amount, true)`.
pub fn teach_for_gold(w: &mut World, id: MobileId, from: MobileId, amount: i32) -> bool {
    let teaching = read(w, id, -1, |d| d.teaching);
    SkillName::from_i32(teaching).is_some_and(|s| teach(w, id, s, from, amount, true))
}
