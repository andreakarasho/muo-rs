//! The mage AI. Spells are made by class name through the spell registry
//! (`new_spell_by_name`), so a class no crate has registered yet simply isn't cast.

use muo_primitives::{Direction, Point3D};
use muo_server::ids::{EntityId, MobileId};
use muo_server::interfaces::SpellId;
use muo_server::skills::{SkillName, value};
use muo_server::target::{self, LandTarget, TargetCancelType, TargetFlags, Targeted};
use muo_server::world::World;

use super::base::{self, BASE_AI, acquire_focus_mob, debug_say};
use super::movement::{do_move, move_to};
use super::{ActionType, AiClass, ai_mut, set_action};
use crate::abilities::{self, MonsterAbilityTrigger};
use crate::creature::data::{cd_mut, read};
use crate::creature::vtable::ctype;
use crate::hooks::rnd;
use crate::mx;
use crate::spells::registry::new_spell_by_name;
use crate::spells::spell;

const HEAL_CHANCE: f64 = 0.10; // 10% chance to heal at gm magery
const TELEPORT_CHANCE: f64 = 0.05; // 5% chance to teleport at gm magery
const DISPEL_CHANCE: f64 = 0.75; // 75% chance to dispel at gm magery
const INVIS_CHANCE: f64 = 0.50; // 50% chance to invis at gm magery

#[rustfmt::skip]
const OFFSETS: [(i32, i32); 24] = [
    (-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1),
    (-2, -2), (-2, -1), (-2, 0), (-2, 1), (-2, 2), (-1, -2), (-1, 2), (0, -2),
    (0, 2), (1, -2), (1, 2), (2, -2), (2, -1), (2, 0), (2, 1), (2, 2),
];

/// MageAI's fields, in the AI's `Ext`.
struct MageState {
    combo: i32,
    last_target: Option<MobileId>,
    last_target_loc: Point3D,
    next_cast_time: i64,
    next_heal_time: i64,
    reveal_target: Option<LandTarget>,
}

impl Default for MageState {
    fn default() -> Self {
        MageState {
            combo: -1,
            last_target: None,
            last_target_loc: Point3D::ZERO,
            next_cast_time: 0,
            next_heal_time: 0,
            reveal_target: None,
        }
    }
}

fn st(w: &mut World, id: MobileId) -> Option<&mut MageState> {
    let a = ai_mut(w, id)?;
    if a.ext.get::<MageState>().is_none() {
        a.ext.insert(MageState::default());
    }
    a.ext.get_mut::<MageState>()
}

fn rst<T>(w: &mut World, id: MobileId, f: impl FnOnce(&mut MageState) -> T) -> Option<T> {
    st(w, id).map(f)
}

pub static MAGE_AI: AiClass = AiClass {
    name: "MageAI",
    flee_health_threshold: 0.2, // 20% is default
    flee_chance: 0.1,           // 10% is default
    think,
    do_action_wander,
    do_action_combat,
    do_action_guard,
    ..BASE_AI
};

fn name(w: &World, id: MobileId) -> String {
    muo_server::mobile::base::name_base(w, id).unwrap_or_default()
}

fn smart_ai(w: &World, id: MobileId) -> bool {
    use crate::factory::mobile_is;
    mobile_is(w, id, "Server.Mobiles.BaseVendor")
        || mobile_is(w, id, "Server.Mobiles.BaseEscortable")
        || mobile_is(w, id, "Server.Mobiles.Changeling")
}

fn is_necromancer(w: &mut World, id: MobileId) -> bool {
    crate::core::aos(w) && value(w, id, SkillName::Necromancy) > 50.0
}

fn scale_by_skill(w: &mut World, id: MobileId, v: f64, skill: SkillName) -> f64 {
    v * value(w, id, skill) / 100.0
}

fn tick(w: &World) -> i64 {
    w.tick_count
}

fn focus(w: &World, id: MobileId) -> Option<MobileId> {
    read(w, id, None, |d| d.focus_mob)
}

fn set_focus(w: &mut World, id: MobileId, f: Option<MobileId>) {
    if let Some(d) = cd_mut(w, id) {
        d.focus_mob = f;
    }
}

fn acquire(w: &mut World, id: MobileId) -> bool {
    let (r, fm) = (
        read(w, id, 0, |d| d.range_perception),
        read(w, id, crate::creature::enums::FightMode::None, |d| {
            d.fight_mode
        }),
    );
    acquire_focus_mob(w, id, r, fm, false, false, true)
}

fn think(w: &mut World, id: MobileId) -> bool {
    if mx::deleted(w, id) {
        return false;
    }
    process_target(w, id) || base::think(w, id)
}

fn is_casting(w: &World, id: MobileId) -> bool {
    w.mobile(id)
        .and_then(|m| m.spell)
        .is_some_and(|sp| spell::is_casting(w, sp))
}

/// `new T(Mobile)`; `None` when the class isn't registered.
fn mk(w: &mut World, id: MobileId, class: &str) -> Option<SpellId> {
    new_spell_by_name(w, class, id, None)
}

fn cast(w: &mut World, sp: Option<SpellId>) -> bool {
    sp.is_some_and(|sp| spell::cast(w, sp))
}

fn spell_class(w: &World, sp: SpellId) -> &'static str {
    spell::spell_type(w, sp).map_or("", |t| t.full_name.rsplit('.').next().unwrap_or(""))
}

fn do_action_wander(w: &mut World, id: MobileId) -> bool {
    if acquire(w, id) {
        if let Some(f) = focus(w, id) {
            debug_say(w, id, &format!("I am going to attack {}", name(w, f)));
            mx::set_combatant(w, id, Some(f));
        }
        set_action(w, id, ActionType::Combat);
        let t = tick(w);
        rst(w, id, |s| s.next_cast_time = t);
    } else if smart_ai(w, id)
        && w.mobile(id)
            .is_some_and(|m| m.mana < (m.ty.mana_max)(w, id) && !m.meditating)
    {
        debug_say(w, id, "I am going to meditate");
        mx::use_skill(w, id, SkillName::Meditation);
    } else {
        debug_say(w, id, "I am wandering");
        mx::set_warmode(w, id, false);
        base::do_action_wander(w, id);
        if rnd::double(w) < 0.05 {
            let sp = check_cast_healing_spell(w, id);
            cast(w, sp);
        }
    }
    true
}

fn check_cast_healing_spell(w: &mut World, id: MobileId) -> Option<SpellId> {
    // If I'm poisoned, always attempt to cure.
    if w.mobile(id).is_some_and(|m| m.poison.is_some()) {
        return mk(w, id, "CureSpell");
    }
    let next_heal = st(w, id).map_or(0, |s| s.next_heal_time);
    let (summoned, controlled) = read(w, id, (false, false), |d| (d.summoned, d.controlled));
    // Summoned creatures never heal themselves.
    if summoned || (controlled && tick(w).wrapping_sub(next_heal) < 0) {
        return None;
    }
    let smart = smart_ai(w, id);
    if !smart {
        if HEAL_CHANCE > 0.0
            && scale_by_skill(w, id, HEAL_CHANCE, SkillName::Magery) < rnd::double(w)
        {
            return None;
        }
    } else {
        let (hits, max) = (mx::hits(w, id), mx::hits_max(w, id));
        let bound = 4 + if hits == 0 { max } else { max / hits };
        if rnd::from(w, 0, bound) < 3 {
            return None;
        }
    }
    let mut sp = None;
    let (hits, max) = (mx::hits(w, id), mx::hits_max(w, id));
    if hits < max - 50 {
        if use_necromancy(w, id) {
            mx::use_skill(w, id, SkillName::SpiritSpeak);
        } else {
            sp = mk(w, id, "GreaterHealSpell");
        }
    } else if hits < max - 10 {
        sp = mk(w, id, "HealSpell");
    }
    let int = w.mobile(id).map_or(0, |m| (m.ty.int)(w, id));
    let delay = if int >= 500 {
        f64::from(rnd::min_max(w, 7, 10))
    } else {
        f64::from(600 - int).sqrt()
    };
    let t = tick(w) + (delay * 1000.0) as i64;
    rst(w, id, |s| s.next_heal_time = t);
    sp
}

fn paralyzed_or_frozen(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.frozen || (x.ty.paralyzed)(w, m))
}

fn run_to(w: &mut World, id: MobileId, m: MobileId) {
    let range_fight = read(w, id, 1, |d| d.range_fight);
    if !smart_ai(w, id) {
        if !move_to(w, id, m, range_fight) {
            on_failed_move(w, id);
        }
        return;
    }
    if paralyzed_or_frozen(w, m) {
        if mx::in_range(w, id, m, 1) {
            run_from(w, id, m);
        } else if !mx::in_range(w, id, m, range_fight.max(2)) && !move_to(w, id, m, 1) {
            on_failed_move(w, id);
        }
    } else if !mx::in_range(w, id, m, range_fight) {
        if !move_to(w, id, m, 1) {
            on_failed_move(w, id);
        }
    } else if mx::in_range(w, id, m, range_fight - 1) {
        run_from(w, id, m);
    }
}

fn run_from(w: &mut World, id: MobileId, m: MobileId) {
    let d = mx::get_direction_to(w, id, m);
    run(
        w,
        id,
        Direction::from_bits_retain(d.bits().wrapping_sub(4) & Direction::MASK.bits()),
    );
}

fn disallow_all_moves(w: &World, id: MobileId) -> bool {
    ctype(w, id).is_some_and(|c| (c.disallow_all_moves)(w, id))
}

fn on_failed_move(w: &mut World, id: MobileId) {
    let smart = smart_ai(w, id);
    let try_teleport = !disallow_all_moves(w, id)
        && if smart {
            rnd::below(w, 4) == 0
        } else {
            TELEPORT_CHANCE > 0.0
                && scale_by_skill(w, id, TELEPORT_CHANCE, SkillName::Magery) > rnd::double(w)
        };
    if try_teleport {
        if let Some(t) = w.mobile(id).and_then(|m| m.target) {
            target::cancel(w, t, id, TargetCancelType::Canceled);
        }
        let sp = mk(w, id, "TeleportSpell");
        cast(w, sp);
        debug_say(w, id, "I am stuck, I'm going to try teleporting away");
    } else if acquire(w, id) {
        if let Some(f) = focus(w, id) {
            debug_say(
                w,
                id,
                &format!("My move is blocked, so I am going to attack {}", name(w, f)),
            );
            mx::set_combatant(w, id, Some(f));
        }
        set_action(w, id, ActionType::Combat);
    } else {
        debug_say(w, id, "I am stuck");
    }
}

fn run(w: &mut World, id: MobileId, d: Direction) {
    if is_casting(w, id) || paralyzed_or_frozen(w, id) || disallow_all_moves(w, id) {
        return;
    }
    let dir = d | Direction::RUNNING;
    muo_server::mobile::part1::props_b::set_direction(w, id, dir);
    if !do_move(w, id, dir, true) {
        on_failed_move(w, id);
    }
}

fn use_necromancy(w: &mut World, id: MobileId) -> bool {
    use muo_server::skills::base_fixed;
    let magery = base_fixed(w, id, SkillName::Magery);
    let necro = base_fixed(w, id, SkillName::Necromancy);
    is_necromancer(w, id) && rnd::below(w, magery + necro) >= magery
}

fn random_damage_spell(w: &mut World, id: MobileId) -> Option<SpellId> {
    // Spellbinder.SpellbinderAI: `GetRandomDamageSpell() => GetRandomCurseSpell()`
    if super::class_of(w, id).is_some_and(|c| c.name == "SpellbinderAI") {
        return random_curse_spell(w, id);
    }
    if use_necromancy(w, id) {
        let bound = if value(w, id, SkillName::Necromancy) >= 100.0 {
            5
        } else {
            3
        };
        let class = match rnd::below(w, bound) {
            0 => "PainSpikeSpell",
            1 => "PoisonStrikeSpell",
            2 => "StrangleSpell",
            3 => "WitherSpell",
            _ => "VengefulSpiritSpell",
        };
        return mk(w, id, class);
    }
    let max_circle =
        (((value(w, id, SkillName::Magery) + 20.0) / (100.0 / 7.0)) as i32).clamp(1, 8);
    let class = match rnd::below(w, max_circle * 2) {
        0 | 1 => "MagicArrowSpell",
        2 | 3 => "HarmSpell",
        4 | 5 => "FireballSpell",
        6 | 7 => "LightningSpell",
        8 | 9 => "MindBlastSpell",
        10 => "EnergyBoltSpell",
        11 => "ExplosionSpell",
        _ => "FlameStrikeSpell",
    };
    mk(w, id, class)
}

fn random_curse_spell(w: &mut World, id: MobileId) -> Option<SpellId> {
    if use_necromancy(w, id) {
        let class = match rnd::below(w, 4) {
            0 => "BloodOathSpell",
            1 => "CorpseSkinSpell",
            2 => "EvilOmenSpell",
            _ => "MindRotSpell",
        };
        return mk(w, id, class);
    }
    if value(w, id, SkillName::Magery) >= 40.0 && rnd::below(w, 4) == 0 {
        return mk(w, id, "CurseSpell");
    }
    let class = match rnd::below(w, 3) {
        0 => "WeakenSpell",
        1 => "ClumsySpell",
        _ => "FeeblemindSpell",
    };
    mk(w, id, class)
}

fn random_mana_drain_spell(w: &mut World, id: MobileId) -> Option<SpellId> {
    if value(w, id, SkillName::Magery) >= 80.0 && rnd::bool(w) {
        return mk(w, id, "ManaVampireSpell");
    }
    mk(w, id, "ManaDrainSpell")
}

fn do_dispel(w: &mut World, id: MobileId, to_dispel: MobileId) -> Option<SpellId> {
    if !smart_ai(w, id) {
        if DISPEL_CHANCE > 0.0
            && scale_by_skill(w, id, DISPEL_CHANCE, SkillName::Magery) > rnd::double(w)
        {
            return mk(w, id, "DispelSpell");
        }
        return None;
    }
    if let Some(sp) = check_cast_healing_spell(w, id) {
        return Some(sp);
    }
    let distance = crate::creature::misc::distance_sqrt(w, id, to_dispel) as i32;
    if !disallow_all_moves(w, id) && distance > 0 && rnd::below(w, distance) == 0 {
        return mk(w, id, "TeleportSpell");
    }
    if rnd::below(w, 3) == 0
        && !mx::in_range(w, id, to_dispel, 3)
        && !paralyzed_or_frozen(w, to_dispel)
    {
        return mk(w, id, "ParalyzeSpell");
    }
    mk(w, id, "DispelSpell")
}

fn poisoned(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| x.poison.is_some())
}

fn choose_spell(w: &mut World, id: MobileId, c: MobileId) -> Option<SpellId> {
    let mut sp;
    if !smart_ai(w, id) {
        if let Some(h) = check_cast_healing_spell(w, id) {
            return Some(h);
        }
        if is_necromancer(w, id) {
            let ps_damage =
                (value(w, id, SkillName::SpiritSpeak) - value(w, c, SkillName::MagicResist)) / 10.0
                    + if mx::player(w, c) { 18.0 } else { 30.0 };
            if ps_damage > f64::from(mx::hits(w, c)) {
                return mk(w, id, "PainSpikeSpell");
            }
        }
        let mut pick = rnd::below(w, 16);
        // `goto default` cases fall to the damage branch
        if matches!(pick, 0 | 1) && poisoned(w, c) {
            pick = 99;
        }
        if pick == 5 && (paralyzed_or_frozen_only(w, c) || value(w, id, SkillName::Magery) <= 50.0)
        {
            pick = 99;
        }
        if pick == 7
            && (INVIS_CHANCE <= 0.0
                || scale_by_skill(w, id, INVIS_CHANCE, SkillName::Magery) <= rnd::double(w))
        {
            pick = 99;
        }
        sp = match pick {
            0 | 1 => {
                debug_say(w, id, "Attempting to poison");
                mk(w, id, "PoisonSpell")
            }
            2 => {
                debug_say(w, id, "Blessing myself");
                mk(w, id, "BlessSpell")
            }
            3 | 4 => {
                debug_say(w, id, "Attempting to curse");
                random_curse_spell(w, id)
            }
            5 => {
                debug_say(w, id, "Attempting to paralyze");
                mk(w, id, "ParalyzeSpell")
            }
            6 => {
                debug_say(w, id, "Attempting to drain mana");
                random_mana_drain_spell(w, id)
            }
            7 => {
                debug_say(w, id, "Attempting to invis myself");
                mk(w, id, "InvisibilitySpell")
            }
            _ => {
                debug_say(w, id, "Just doing damage");
                random_damage_spell(w, id)
            }
        };
    } else {
        sp = check_cast_healing_spell(w, id);
        if sp.is_none() {
            let mut pick = rnd::below(w, 3);
            if pick == 0 && poisoned(w, c) {
                pick = 1;
            }
            match pick {
                0 => sp = mk(w, id, "PoisonSpell"),
                1 => sp = random_damage_spell(w, id),
                _ => {
                    // Set up a combo
                    let mana = w.mobile(id).map_or(0, |m| m.mana);
                    let meditating = w.mobile(id).is_some_and(|m| m.meditating);
                    if mana > 15 && mana < 40 {
                        if paralyzed_or_frozen_only(w, c) && !poisoned(w, c) && !meditating {
                            debug_say(w, id, "I am going to meditate");
                            mx::use_skill(w, id, SkillName::Meditation);
                        } else if !poisoned(w, c) {
                            sp = mk(w, id, "ParalyzeSpell");
                        }
                    } else if mana > 60 {
                        if rnd::bool(w) && !paralyzed_or_frozen(w, c) && !poisoned(w, c) {
                            rst(w, id, |s| s.combo = 0);
                            sp = mk(w, id, "ParalyzeSpell");
                        } else {
                            rst(w, id, |s| s.combo = 1);
                            sp = mk(w, id, "ExplosionSpell");
                        }
                    }
                }
            }
        }
    }
    match sp {
        Some(s) => {
            let n = spell::info(w, s).map_or("", |i| i.name);
            debug_say(w, id, &format!("Casting {n}"));
        }
        None => debug_say(w, id, "I don't have a spell to use!"),
    }
    sp
}

/// `c.Paralyzed` alone (the non-smart branches test it without `Frozen`).
fn paralyzed_or_frozen_only(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| (x.ty.paralyzed)(w, m))
}

fn do_combo(w: &mut World, id: MobileId, c: MobileId) -> Option<SpellId> {
    let mut sp = None;
    let combo = st(w, id).map_or(-1, |s| s.combo);
    let set = |w: &mut World, v: i32| {
        rst(w, id, |s| s.combo = v);
    };
    let mut combo_now = combo;
    if combo == 0 {
        sp = mk(w, id, "ExplosionSpell");
        combo_now += 1;
    } else if combo == 1 {
        sp = mk(w, id, "WeakenSpell");
        combo_now += 1;
    } else if combo == 2 {
        if !poisoned(w, c) {
            sp = mk(w, id, "PoisonSpell");
        } else if is_necromancer(w, id) {
            sp = mk(w, id, "StrangleSpell");
        }
        combo_now += 1;
    }
    set(w, combo_now);
    if combo_now == 3 && sp.is_none() {
        let n = if is_necromancer(w, id) { 4 } else { 3 };
        match rnd::below(w, n) {
            0 => {
                let (ci, cd) = w
                    .mobile(c)
                    .map_or((0, 0), |m| ((m.ty.int)(w, c), (m.ty.dex)(w, c)));
                sp = mk(
                    w,
                    id,
                    if ci < cd {
                        "FeeblemindSpell"
                    } else {
                        "ClumsySpell"
                    },
                );
                set(w, combo_now + 1);
            }
            1 => {
                sp = mk(w, id, "EnergyBoltSpell");
                set(w, -1);
            }
            2 => {
                sp = mk(w, id, "FlameStrikeSpell");
                set(w, -1);
            }
            _ => {
                sp = mk(w, id, "PainSpikeSpell");
                set(w, -1);
            }
        }
    } else if combo_now == 4 && sp.is_none() {
        sp = mk(w, id, "MindBlastSpell");
        set(w, -1);
    }
    sp
}

fn get_delay_ms(w: &mut World, id: MobileId, sp: Option<SpellId>) -> i64 {
    if smart_ai(w, id) || sp.is_some_and(|s| spell_class(w, s) == "DispelSpell") {
        let active = read(w, id, 0.0, |d| d.active_speed);
        return (active * 1000.0) as i64;
    }
    let del = scale_by_skill(w, id, 3.0, SkillName::Magery);
    let min = 6.0 - del * 1.25;
    let max = 6.0 - del * 0.75;
    ((min + (max - min) * rnd::double(w)) * 1000.0) as i64
}

fn combatant_gone(w: &mut World, id: MobileId, c: Option<MobileId>) -> bool {
    let Some(c) = c else { return true };
    mx::deleted(w, c)
        || !mx::alive(w, c)
        || w.mobile(c).is_some_and(|m| (m.ty.is_dead_bonded_pet)(w, c))
        || !w
            .mobile(id)
            .map(|m| m.ty)
            .is_some_and(|m| (m.can_see_mobile)(w, id, c))
        || !mx::can_be_harmful_quiet(w, id, c)
        || mx::map(w, c) != mx::map(w, id)
}

fn do_action_combat(w: &mut World, id: MobileId) -> bool {
    let mut c = mx::combatant(w, id);
    mx::set_warmode(w, id, true);
    if combatant_gone(w, id, c) {
        // Our combatant is deleted, dead, hidden, or we cannot hurt them: try to find another.
        if acquire(w, id) {
            let f = focus(w, id);
            mx::set_combatant(w, id, f);
            c = f;
            if let Some(f) = f {
                debug_say(
                    w,
                    id,
                    &format!(
                        "Something happened to my combatant, so I am going to fight {}",
                        name(w, f)
                    ),
                );
            }
            set_focus(w, id, None);
        } else {
            debug_say(
                w,
                id,
                "Something happened to my combatant, and nothing is around. I am on guard.",
            );
            set_action(w, id, ActionType::Guard);
            return true;
        }
    }
    let Some(mut c) = c else { return true };

    if !mx::in_los(w, id, c) {
        debug_say(w, id, "I can't see my target");
        if acquire(w, id) {
            if let Some(f) = focus(w, id) {
                mx::set_combatant(w, id, Some(f));
                c = f;
                debug_say(w, id, &format!("I will switch to {}", name(w, c)));
            }
            set_focus(w, id, None);
        }
    }

    if !crate::core::aos(w)
        && smart_ai(w, id)
        && !w.mobile(id).is_some_and(|m| m.stun_ready)
        && value(w, id, SkillName::Wrestling) >= 80.0
        && value(w, id, SkillName::Anatomy) >= 80.0
    {
        super::classes::stun_request(w, id);
    }

    let rp = read(w, id, 0, |d| d.range_perception);
    if !mx::in_range(w, id, c, rp) {
        // They are somewhat far away, can we find something else?
        if acquire(w, id) {
            let f = focus(w, id);
            mx::set_combatant(w, id, f);
            set_focus(w, id, None);
        } else {
            let leash = ctype(w, id).map_or(0, |t| (t.chase_leash_range)(w, id));
            if !mx::in_range(w, id, c, leash) {
                mx::set_combatant(w, id, None);
            }
        }
        match mx::combatant(w, id) {
            Some(n) => c = n,
            None => {
                debug_say(w, id, "My combatant has fled, so I am on guard");
                set_action(w, id, ActionType::Guard);
                return true;
            }
        }
    }

    // Geometry (not hiding) is blocking the shot: close in until line of sight returns. Poisoned mages still cure.
    if !poisoned(w, id) && !is_casting(w, id) && !mx::in_los(w, id, c) {
        debug_say(
            w,
            id,
            "I cannot see my target, moving to regain line of sight",
        );
        if !move_to(w, id, c, 1) {
            on_failed_move(w, id);
        }
        let loc = mx::location(w, c);
        rst(w, id, |s| {
            s.last_target = Some(c);
            s.last_target_loc = loc;
        });
        return true;
    }

    if abilities::trigger_ability(w, id, MonsterAbilityTrigger::COMBAT_ACTION, Some(c)) {
        debug_say(w, id, "I used my abilities!");
    } else if w.mobile(id).is_some_and(|m| m.spell.is_none())
        && tick(w).wrapping_sub(st(w, id).map_or(0, |s| s.next_cast_time)) >= 0
    {
        if read(w, id, false, |d| d.controlled) && c == id {
            debug_say(w, id, "I should not attack myself!");
            mx::set_combatant(w, id, None);
            set_action(w, id, ActionType::Guard);
            return true;
        }
        // We are ready to cast a spell
        let to_dispel = find_dispel_target(w, id, true);
        let first = if poisoned(w, id) {
            // Top cast priority is cure
            debug_say(w, id, "I am going to cure myself");
            mk(w, id, "CureSpell")
        } else if let Some(d) = to_dispel {
            debug_say(w, id, &format!("I am going to dispel {}", name(w, d)));
            do_dispel(w, id, d) // may return null if dumb AI and doesn't have enough skill
        } else {
            None
        };
        let sp = match first {
            Some(s) => Some(s),
            None => {
                let smart = smart_ai(w, id);
                let combo = st(w, id).map_or(-1, |s| s.combo);
                let their_heal = w
                    .mobile(c)
                    .and_then(|m| m.spell)
                    .map(|s| spell_class(w, s))
                    .is_some_and(|n| n == "HealSpell" || n == "GreaterHealSpell");
                if smart && combo != -1 {
                    do_combo(w, id, c)
                } else if smart && !poisoned(w, c) && their_heal {
                    mk(w, id, "PoisonSpell")
                } else {
                    choose_spell(w, id, c)
                }
            }
        };

        let range = sp
            .and_then(|s| crate::spells::targeting::ranged_spell_range(w, s))
            .unwrap_or(rp); // (spell as IRangedSpell)?.TargetRange ?? Mobile.RangePerception
        // Move first before casting
        if !smart_ai(w, id) || to_dispel.is_none() {
            run_to(w, id, c);
        } else if let Some(d) = to_dispel {
            if mx::in_range(w, id, d, range.min(10)) {
                run_from(w, id, d);
            } else if !mx::in_range(w, id, d, range) {
                run_to(w, id, d);
            }
        }
        // After running, make sure we are still in range
        if sp.is_none() || mx::in_range(w, id, c, range) {
            cast(w, sp);
            // Even if we don't have a spell, delay the next potential cast
            let delay = get_delay_ms(w, id, sp);
            let t = tick(w) + delay;
            rst(w, id, |s| s.next_cast_time = t);
        }
    } else if !is_casting(w, id) {
        run_to(w, id, c);
    }

    let last_move = w.mobile(id).map_or(0, |m| m.last_move_time);
    if mx::in_range(w, id, c, 1) || is_casting(w, id) || tick(w) - last_move > 400 {
        let d = mx::get_direction_to(w, id, c);
        muo_server::mobile::part1::props_b::set_direction(w, id, d);
    }
    let loc = mx::location(w, c);
    rst(w, id, |s| {
        s.last_target = Some(c);
        s.last_target_loc = loc;
    });
    true
}

fn land_target(w: &mut World, map: muo_server::map::MapId, p: Point3D) -> LandTarget {
    let z = w.maps.get_average_z(map, p.x, p.y);
    let tile = w.maps.land_tile(map, p.x, p.y);
    LandTarget {
        location: Point3D::new(p.x, p.y, z),
        tile_id: i32::from(tile.id as u16 & 0x3FFF),
    }
}

fn do_action_guard(w: &mut World, id: MobileId) -> bool {
    let (last, last_loc) = st(w, id).map_or((None, Point3D::ZERO), |s| {
        (s.last_target, s.last_target_loc)
    });
    if last.is_some_and(|l| w.mobile(l).is_some_and(|m| m.hidden)) {
        let map = mx::map(w, id);
        let rp = read(w, id, 0, |d| d.range_perception);
        if map.is_none() || !mx::in_range_point(w, id, last_loc, rp) {
            rst(w, id, |s| s.last_target = None);
        } else if w.mobile(id).is_some_and(|m| m.spell.is_none())
            && tick(w).wrapping_sub(st(w, id).map_or(0, |s| s.next_cast_time)) >= 0
        {
            debug_say(w, id, "I am going to reveal my last target");
            if let Some(map) = map {
                let lt = land_target(w, map, last_loc);
                rst(w, id, |s| s.reveal_target = Some(lt));
            }
            let sp = mk(w, id, "RevealSpell");
            if cast(w, sp) {
                rst(w, id, |s| s.last_target = None); // only do it once
            }
            let delay = get_delay_ms(w, id, sp);
            let t = tick(w) + delay;
            rst(w, id, |s| s.next_cast_time = t);
        }
    }
    if acquire(w, id) {
        if let Some(f) = focus(w, id) {
            debug_say(w, id, &format!("I am going to attack {}", name(w, f)));
            mx::set_combatant(w, id, Some(f));
        }
        set_action(w, id, ActionType::Combat);
    } else {
        if !read(w, id, false, |d| d.controlled) {
            process_target(w, id);
            let sp = check_cast_healing_spell(w, id);
            cast(w, sp);
        }
        base::do_action_guard(w, id);
    }
    true
}

fn find_dispel_target(w: &mut World, id: MobileId, active_only: bool) -> Option<MobileId> {
    let int = w.mobile(id).map_or(0, |m| (m.ty.int)(w, id));
    let auto_dispel = ctype(w, id).is_some_and(|c| (c.auto_dispel)(w, id));
    if mx::deleted(w, id) || int < 95 || can_dispel(w, id, id) || auto_dispel {
        return None;
    }
    let rp = read(w, id, 0, |d| d.range_perception);
    let comb = mx::combatant(w, id);
    let usable = |w: &World, m: MobileId| {
        !mx::deleted(w, m)
            && mx::alive(w, m)
            && !w.mobile(m).is_some_and(|x| (x.ty.is_dead_bonded_pet)(w, m))
    };
    let dist = |w: &World, m: MobileId| crate::creature::misc::distance_sqrt(w, id, m);
    if active_only {
        let mut active: Option<MobileId> = None;
        let mut active_prio = 0.0;
        if let Some(c) =
            comb.filter(|&c| usable(w, c) && mx::in_range(w, id, c, rp) && can_dispel(w, id, c))
        {
            active = Some(c);
            active_prio = dist(w, c);
            if active_prio <= 2.0 {
                return active;
            }
        }
        let (aggressed, aggressors): (Vec<MobileId>, Vec<MobileId>) =
            w.mobile(id).map_or((vec![], vec![]), |m| {
                (
                    m.aggressed.iter().map(|i| i.defender).collect(),
                    m.aggressors.iter().map(|i| i.attacker).collect(),
                )
            });
        for m in aggressed.into_iter().chain(aggressors) {
            if Some(m) != comb
                && mx::combatant(w, m) == Some(id)
                && mx::in_range(w, id, m, rp)
                && can_dispel(w, id, m)
            {
                let prio = dist(w, m);
                if active.is_none() || prio < active_prio {
                    active = Some(m);
                    active_prio = prio;
                    if active_prio <= 2.0 {
                        return active;
                    }
                }
            }
        }
        return active;
    }
    mx::map(w, id)?;
    let (mut active, mut inactive): (Option<MobileId>, Option<MobileId>) = (None, None);
    let (mut act_prio, mut inact_prio) = (0.0, 0.0);
    if let Some(c) = comb.filter(|&c| usable(w, c) && can_dispel(w, id, c)) {
        active = Some(c);
        inactive = Some(c);
        act_prio = dist(w, c);
        inact_prio = act_prio;
    }
    for m in mx::mobiles_in_range(w, id, rp) {
        if m != id && can_dispel(w, id, m) {
            let prio = dist(w, m);
            if inactive.is_none() || prio < inact_prio {
                inactive = Some(m);
                inact_prio = prio;
            }
            if (comb == Some(m) || mx::combatant(w, m) == Some(id))
                && (active.is_none() || prio < act_prio)
            {
                active = Some(m);
                act_prio = prio;
            }
        }
    }
    active.or(inactive)
}

fn can_dispel(w: &mut World, id: MobileId, m: MobileId) -> bool {
    crate::creature::is_creature(w, m)
        && read(w, m, false, |d| d.summoned)
        && crate::creature::misc::summoner(w, m) != Some(id)
        && mx::can_be_harmful_quiet(w, id, m)
        && !crate::creature::misc::is_animated_dead(w, m)
}

/// `ProcessTarget`: answers the target a spell opened. True when there was one.
fn process_target(w: &mut World, id: MobileId) -> bool {
    use muo_server::map::fit::can_spawn_mobile_at;
    let Some(tid) = w.mobile(id).and_then(|m| m.target) else {
        return false;
    };
    let spell_id = w.targets.get(&tid).and_then(|t| t.spell);
    let class = spell_id.map(|s| spell_class(w, s)).unwrap_or("");
    let (is_reveal, is_dispel) = (class == "RevealSpell", class == "DispelSpell");
    let (is_paralyze, is_teleport, is_invisible) = (
        class == "ParalyzeSpell",
        class == "TeleportSpell",
        class == "InvisibilitySpell",
    );
    let mut teleport_away = false;
    let smart = smart_ai(w, id);
    let to_target: Option<MobileId>;

    if is_invisible {
        to_target = Some(id);
    } else if is_dispel {
        to_target = find_dispel_target(w, id, false);
        if !smart && let Some(t) = to_target {
            run_to(w, id, t);
        } else if let Some(t) = to_target.filter(|&t| mx::in_range(w, id, t, 10)) {
            run_from(w, id, t);
        }
    } else if smart && (is_paralyze || is_teleport) {
        let t = find_dispel_target(w, id, true);
        match t {
            None => {
                to_target = mx::combatant(w, id);
                if let Some(c) = to_target {
                    run_to(w, id, c);
                }
            }
            Some(t) => {
                to_target = Some(t);
                if mx::in_range(w, id, t, 10) {
                    run_from(w, id, t);
                }
                teleport_away = true;
            }
        }
    } else {
        to_target = mx::combatant(w, id);
        if let Some(t) = to_target {
            // Without line of sight the stand-off is pointless: close in so the held target can be invoked.
            if !mx::in_los(w, id, t) {
                move_to(w, id, t, 1);
            } else {
                run_to(w, id, t);
            }
        }
    }

    let (flags, range) = w
        .targets
        .get(&tid)
        .map_or((TargetFlags::NONE, -1), |t| (t.flags, t.range));
    let mob_target = |m| Targeted::Entity(EntityId::Mobile(m));
    if let (true, Some(t)) = (flags.contains(TargetFlags::HARMFUL), to_target) {
        let can_see = w
            .mobile(id)
            .map(|m| m.ty)
            .is_some_and(|m| (m.can_see_mobile)(w, id, t));
        if (range == -1 || mx::in_range(w, id, t, range)) && can_see && mx::in_los(w, id, t) {
            target::invoke(w, tid, id, mob_target(t));
        } else if is_dispel {
            target::cancel(w, tid, id, TargetCancelType::Canceled);
        }
    } else if flags.contains(TargetFlags::BENEFICIAL) {
        target::invoke(w, tid, id, mob_target(id));
    } else if is_reveal && st(w, id).is_some_and(|s| s.reveal_target.is_some()) {
        if let Some(lt) = st(w, id).and_then(|s| s.reveal_target) {
            target::invoke(w, tid, id, Targeted::Land(lt));
        }
    } else {
        let map = mx::map(w, id);
        if let (Some(map), true, Some(t)) = (map, is_teleport, to_target) {
            let tele_range = if range >= 0 {
                range
            } else if crate::core::ml(w) {
                11
            } else {
                12
            };
            let (me, them) = (mx::location(w, id), mx::location(w, t));
            let (px, py) = if teleport_away {
                let (rx, ry) = (me.x - them.x, me.y - them.y);
                let d = crate::creature::misc::distance_sqrt(w, id, t);
                (
                    them.x + (f64::from(rx) * (10.0 / d)) as i32,
                    them.y + (f64::from(ry) * (10.0 / d)) as i32,
                )
            } else {
                (them.x, them.y)
            };
            for (x, y) in OFFSETS {
                let p = Point3D::new(px + x, py + y, 0);
                let lt = land_target(w, map, p);
                if (range == -1 || mx::in_range_point(w, id, p, range))
                    && muo_server::mobile::part1::combat::in_los_object(
                        w,
                        id,
                        muo_server::map::los::LosObject::Land(lt.location),
                    )
                    && can_spawn_mobile_at(w, map, px + x, py + y, lt.location.z)
                    && !crate::spells::helper::check_multi(w, p, Some(map), true, 0)
                {
                    target::invoke(w, tid, id, Targeted::Land(lt));
                    return true;
                }
            }
            for _ in 0..10 {
                let rx = me.x - tele_range + rnd::below(w, tele_range * 2 + 1);
                let ry = me.y - tele_range + rnd::below(w, tele_range * 2 + 1);
                let p = Point3D::new(rx, ry, 0);
                let lt = land_target(w, map, p);
                if muo_server::mobile::part1::combat::in_los_object(
                    w,
                    id,
                    muo_server::map::los::LosObject::Land(lt.location),
                ) && can_spawn_mobile_at(w, map, lt.location.x, lt.location.y, lt.location.z)
                    && !crate::spells::helper::check_multi(w, p, Some(map), true, 0)
                {
                    target::invoke(w, tid, id, Targeted::Land(lt));
                    return true;
                }
            }
        }
        target::cancel(w, tid, id, TargetCancelType::Canceled);
    }
    true
}
