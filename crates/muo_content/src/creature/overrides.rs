//! `BaseCreature`'s `override`s of `Mobile` virtuals, as `MobileType` members of `BASE_CREATURE`.

use muo_primitives::{Direction, MessageType, Point3D, TimeSpan};
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{messages, stats};
use muo_server::mobile::{AccessLevel, ApplyPoisonResult, MOBILE_BASE, StatType, base};
use muo_server::poison::PoisonId;
use muo_server::property_list::PropertyList;
use muo_server::region::RegionId;
use muo_server::speech_event::SpeechEventArgs;
use muo_server::world::World;

use super::control::{control_master, get_master};
use super::data::{cd, cd_mut, read};
use super::enums::*;
use super::vtable::ctype;
use crate::abilities::{self, MonsterAbilityTrigger};
use crate::hooks::{hooks, rnd, world_hooks};
use crate::mx;

fn base_ty(w: &World, id: MobileId) -> Option<&'static muo_server::mobile::MobileType> {
    w.mobile(id).map(|m| m.ty)
}

/// `Name`
pub fn name(w: &World, id: MobileId) -> Option<String> {
    let m = w.mobile(id)?;
    if m.name_mod.is_none() && m.name.is_none() {
        return ctype(w, id)
            .and_then(|c| (c.default_name)(w, id))
            .map(str::to_owned);
    }
    (MOBILE_BASE.name)(w, id)
}

pub fn set_name(w: &mut World, id: MobileId, value: Option<String>) {
    let default = ctype(w, id).and_then(|c| (c.default_name)(w, id));
    let v = if value.as_deref() == default {
        None
    } else {
        value
    };
    (MOBILE_BASE.set_name)(w, id, v);
}

/// `Murderer`
pub fn murderer(w: &World, id: MobileId) -> bool {
    ctype(w, id).is_some_and(|c| (c.always_murderer)(w, id)) || (MOBILE_BASE.murderer)(w, id)
}

fn stat_max(w: &World, id: MobileId, seed: i32, raw: i32, stat: StatType) -> i32 {
    if seed <= 0 {
        raw
    } else {
        (seed + stats::get_stat_offset(w, id, stat)).clamp(1, 65000)
    }
}

/// `HitsMax`
pub fn hits_max(w: &World, id: MobileId) -> i32 {
    let seed = read(w, id, -1, |d| d.hits_max_seed);
    let raw = w.mobile(id).map_or(0, |m| (m.ty.str)(w, id));
    stat_max(w, id, seed, raw, StatType::STR)
}

pub fn stam_max(w: &World, id: MobileId) -> i32 {
    let seed = read(w, id, -1, |d| d.stam_max_seed);
    let raw = w.mobile(id).map_or(0, |m| (m.ty.dex)(w, id));
    stat_max(w, id, seed, raw, StatType::DEX)
}

pub fn mana_max(w: &World, id: MobileId) -> i32 {
    let seed = read(w, id, -1, |d| d.mana_max_seed);
    let raw = w.mobile(id).map_or(0, |m| (m.ty.int)(w, id));
    stat_max(w, id, seed, raw, StatType::INT)
}

pub fn should_check_stat_timers(_w: &World, _id: MobileId) -> bool {
    false
}

pub fn can_regen_hits(w: &World, id: MobileId) -> bool {
    let (dead, summoned) = read(w, id, (false, false), |d| (d.is_dead_pet, d.summoned));
    !dead && !summoned && (MOBILE_BASE.can_regen_hits)(w, id)
}

pub fn can_regen_stam(w: &World, id: MobileId) -> bool {
    let (dead, paragon) = read(w, id, (false, false), |d| (d.is_dead_pet, d.is_paragon));
    !paragon && !dead && (MOBILE_BASE.can_regen_stam)(w, id)
}

pub fn can_regen_mana(w: &World, id: MobileId) -> bool {
    !read(w, id, false, |d| d.is_dead_pet) && (MOBILE_BASE.can_regen_mana)(w, id)
}

pub fn is_dead_bonded_pet(w: &World, id: MobileId) -> bool {
    read(w, id, false, |d| d.is_dead_pet)
}

pub fn base_physical_resistance(w: &World, id: MobileId) -> i32 {
    read(w, id, 0, |d| d.physical_resistance_seed)
}
pub fn base_fire_resistance(w: &World, id: MobileId) -> i32 {
    read(w, id, 0, |d| d.fire_resist_seed)
}
pub fn base_cold_resistance(w: &World, id: MobileId) -> i32 {
    read(w, id, 0, |d| d.cold_resist_seed)
}
pub fn base_poison_resistance(w: &World, id: MobileId) -> i32 {
    read(w, id, 0, |d| d.poison_resist_seed)
}
pub fn base_energy_resistance(w: &World, id: MobileId) -> i32 {
    read(w, id, 0, |d| d.energy_resist_seed)
}

/// `Damage`
pub fn damage(
    w: &mut World,
    id: MobileId,
    mut amount: i32,
    from: Option<MobileId>,
    inform_mount: bool,
    ignore_evil_omen: bool,
) {
    let old_hits = mx::hits(w, id);
    // Blood oath reflects the original damage the attacker dealt, before other modifiers.
    let has_blood_oath = from
        .is_some_and(|f| hooks(w).and_then(|h| h.blood_oath_of).and_then(|g| g(w, f)) == Some(id));
    let reflected = if has_blood_oath { amount } else { 0 };
    let (summoned, controlled) = read(w, id, (false, false), |d| (d.summoned, d.controlled));
    let c = ctype(w, id).expect("creature");
    if props_a::core_aos(w) && !summoned && controlled && rnd::double(w) < 0.2 {
        amount = (amount as f64 * (c.bonus_pet_damage_scalar)(w, id)) as i32;
    }
    if hooks(w)
        .and_then(|h| h.evil_omen_end_effect)
        .is_some_and(|f| f(w, id))
    {
        amount = (amount as f64 * 1.25) as i32;
    }
    if has_blood_oath {
        amount = (amount as f64 * 1.2) as i32;
    }
    // `base.Damage(amount, from, informMount)`: the evil omen flag is not forwarded.
    let _ = ignore_evil_omen;
    (MOBILE_BASE.damage)(w, id, amount, from, inform_mount, false);

    // If the blood oath caster will die then damage is not reflected back to the attacker.
    if has_blood_oath
        && mx::alive(w, id)
        && !mx::deleted(w, id)
        && !is_dead_bonded_pet(w, id)
        && let Some(from) = from
    {
        let resist = muo_server::skills::value(w, from, muo_server::skills::SkillName::MagicResist);
        let sa = w.expansion() >= muo_data::expansion::Expansion::SA;
        if let Some(f) = hooks(w).and_then(|h| h.blood_oath_reflected) {
            let d = f(reflected, resist, sa);
            mx::damage(w, from, d, Some(id));
        }
    }
    let hmax = mx::hits_max(w, id);
    let controlled_now = read(w, id, false, |d| d.controlled);
    if (c.subdue_before_tame)(w, id)
        && !controlled_now
        && old_hits > hmax / 10
        && mx::hits(w, id) <= hmax / 10
    {
        // * The creature has been beaten into subjugation! *
        messages::public_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            1080057,
            "",
            false,
        );
    }
}

/// `SetLocation`
pub fn set_location(w: &mut World, id: MobileId, loc: Point3D, is_teleport: bool) {
    (MOBILE_BASE.set_location)(w, id, loc, is_teleport);
    if is_teleport {
        crate::ai::on_teleported(w, id);
    }
}

/// `OnBeforeSpawn`
pub fn on_before_spawn(w: &mut World, id: MobileId, location: Point3D, m: MapId) {
    if hooks(w)
        .and_then(|h| h.paragon_check_convert)
        .is_some_and(|f| f(w, id, location, m))
    {
        set_is_paragon(w, id, true);
    }
    (MOBILE_BASE.on_before_spawn)(w, id, location, m);
}

/// `IsParagon` setter: converts or un-converts through the Paragon system.
pub fn set_is_paragon(w: &mut World, id: MobileId, value: bool) {
    if read(w, id, false, |d| d.is_paragon) == value {
        return;
    }
    if let Some(f) = hooks(w).and_then(|h| h.paragon_convert) {
        f(w, id, value);
    }
    if let Some(d) = cd_mut(w, id) {
        d.is_paragon = value;
    }
    muo_server::mobile::part2::properties::invalidate_properties(w, id);
}

/// `ApplyPoison`
pub fn apply_poison(
    w: &mut World,
    id: MobileId,
    from: Option<MobileId>,
    mut poison: PoisonId,
) -> ApplyPoisonResult {
    if !mx::alive(w, id) || is_dead_bonded_pet(w, id) {
        return ApplyPoisonResult::Immune;
    }
    if hooks(w)
        .and_then(|h| h.evil_omen_end_effect)
        .is_some_and(|f| f(w, id))
        && let Some(p) = w
            .resources
            .get::<muo_server::poison::Poisons>()
            .and_then(|p| p.increase_level(Some(poison)))
    {
        poison = p;
    }
    let result = (MOBILE_BASE.apply_poison)(w, id, from, poison);
    if let Some(f) = from
        && result == ApplyPoisonResult::Poisoned
    {
        crate::misc::poison_kinds::set_source(w, id, f); // timer.From = from
    }
    result
}

/// `CheckPoisonImmunity`
pub fn check_poison_immunity(
    w: &World,
    id: MobileId,
    from: Option<MobileId>,
    poison: PoisonId,
) -> bool {
    if (MOBILE_BASE.check_poison_immunity)(w, id, from, poison) {
        return true;
    }
    let immune = ctype(w, id).and_then(|c| (c.poison_immune)(w, id));
    let immune = if read(w, id, false, |d| d.is_paragon) {
        w.resources
            .get::<muo_server::poison::Poisons>()
            .and_then(|p| p.increase_level(immune))
    } else {
        immune
    };
    immune.is_some_and(|i| muo_server::poison::level(w, i) >= muo_server::poison::level(w, poison))
}

/// `OnDamage`
pub fn on_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    let hmax = mx::hits_max(w, id);
    let hits = mx::hits(w, id);
    if read(w, id, false, |d| d.bard_pacified) && ((hmax - hits) as f64 * 0.001) > rnd::double(w) {
        unpacify(w, id);
    }
    let threshold = if !crate::core::aos(w) {
        0
    } else if from.and_then(|f| w.mobile(f)).is_some_and(|f| f.player) {
        18
    } else {
        25
    };
    if amount > threshold {
        crate::player_mobile::hooks::bandage_slip(w, id);
    }
    crate::player_mobile::hooks::confidence_stop_regenerating(w, id);
    crate::misc::stamina_system::fatigue_on_damage(w, id, amount);
    if !will_kill && let Some(sp) = ctype(w, id).and_then(|c| (c.speech_type)(w, id)) {
        sp.on_damage(w, id, amount);
    }
    crate::player_mobile::hooks::honor_on_damaged(w, id, from, amount); // `ReceivedHonorContext?.OnTargetDamaged(from, amount)`, from may be null
    (MOBILE_BASE.on_damage)(w, id, amount, from, will_kill);
}

/// `Unpacify`
pub fn unpacify(w: &mut World, id: MobileId) {
    let now = w.now;
    if let Some(d) = cd_mut(w, id) {
        d.bard_end_time = now;
        d.bard_pacified = false;
    }
}

/// `ApplyNameSuffix`
pub fn apply_name_suffix(w: &World, id: MobileId, suffix: &str) -> String {
    let paragon = read(w, id, false, |d| d.is_paragon);
    let ml_art = ctype(w, id).is_some_and(|c| (c.gives_ml_minor_artifact)(w, id));
    let s = if paragon && !ml_art {
        if suffix.is_empty() {
            "(Paragon)".to_owned()
        } else {
            format!("{suffix} (Paragon)")
        }
    } else {
        suffix.to_owned()
    };
    (MOBILE_BASE.apply_name_suffix)(w, id, &s)
}

/// `Move`
pub fn move_dir(w: &mut World, id: MobileId, d: Direction) -> bool {
    if !(MOBILE_BASE.move_dir)(w, id, d) {
        return false;
    }
    abilities::trigger_ability_move(w, id, MonsterAbilityTrigger::MOVEMENT, d);
    true
}

/// `AggressiveAction(aggressor, criminal)`
pub fn aggressive_action(w: &mut World, id: MobileId, aggressor: MobileId, criminal: bool) {
    (MOBILE_BASE.aggressive_action_criminal)(w, id, aggressor, criminal);
    if let Some(master) = control_master(w, id)
        && crate::misc::notoriety::check_aggressor(w, master, aggressor)
    {
        let info = muo_server::aggressor_info::AggressorInfo::create(w, id, aggressor, true);
        if let Some(a) = w.mobile_mut(aggressor) {
            a.aggressors.push(info);
        }
    }
    if !crate::ai::on_aggressive_action(w, id, aggressor) {
        return;
    }
    (ctype(w, id).expect("creature").stop_flee)(w, id);
    (ctype(w, id).expect("creature").force_reacquire)(w, id);
    if !(ctype(w, id).expect("creature").is_enemy)(w, id, aggressor)
        && let Some(f) = crate::hooks::hooks(w).and_then(|h| h.ethic_finish_shield)
    {
        f(w, aggressor);
    }
}

/// `OnMoveOver`
pub fn on_move_over(w: &mut World, id: MobileId, m: MobileId) -> bool {
    if cd(w, m).is_some_and(|c| !c.controlled) {
        let dead_bonded = |w: &World, x: MobileId| is_dead_bonded_pet(w, x);
        let hidden_staff = w
            .mobile(id)
            .is_some_and(|x| x.hidden && x.access_level > AccessLevel::Player);
        return !mx::alive(w, id)
            || !mx::alive(w, m)
            || dead_bonded(w, id)
            || dead_bonded(w, m)
            || hidden_staff;
    }
    // `Region.IsPartOf<SafeZone>() && m is PlayerMobile pm && (pm.DuelContext?.Started != true ||
    // pm.DuelContext.Finished || pm.DuelPlayer?.Eliminated != false)`
    if crate::player_mobile::hooks::in_safe_zone(w, id)
        && crate::player_mobile::data(w, m).is_some()
    {
        let in_duel = crate::player_mobile::hooks::duel_state(w, m)
            .is_some_and(|d| d.started && !d.finished && !d.eliminated);
        if !in_duel {
            return true;
        }
    }
    (MOBILE_BASE.on_move_over)(w, id, m)
}

/// `HandlesOnSpeech`
pub fn handles_on_speech(w: &World, id: MobileId, from: MobileId) -> bool {
    let sp = ctype(w, id).and_then(|c| (c.speech_type)(w, id));
    sp.is_some_and(|s| s.flags & crate::misc::inhuman_speech::ON_SPEECH != 0)
        && mx::in_range(w, from, id, 3)
        || crate::ai::handles_on_speech(w, id, from)
            && mx::in_range(w, from, id, read(w, id, 0, |d| d.range_perception))
}

/// `OnSpeech`
pub fn on_speech(w: &mut World, id: MobileId, e: &mut SpeechEventArgs) {
    let rp = read(w, id, 0, |d| d.range_perception);
    if let Some(sp) = ctype(w, id).and_then(|c| (c.speech_type)(w, id))
        && sp.on_speech(w, id, e.mobile, &e.speech)
    {
        e.handled = true;
        return;
    }
    if !e.handled && mx::in_range(w, e.mobile, id, rp) {
        crate::ai::on_speech(w, id, e);
    }
}

/// `IsHarmfulCriminal`
pub fn is_harmful_criminal(w: &mut World, id: MobileId, target: MobileId) -> bool {
    Some(target) != get_master(w, id)
        && cd(w, target).is_none_or(|c| {
            !ctype(w, target).is_some_and(|t| (t.initial_innocent)(w, target)) || c.controlled
        })
        && crate::player_mobile::data(w, target).is_none_or(|d| d.perma_flags.is_empty())
        && (MOBILE_BASE.is_harmful_criminal)(w, id, target)
}

/// `CriminalAction`
pub fn criminal_action(w: &mut World, id: MobileId, message: bool) {
    (MOBILE_BASE.criminal_action)(w, id, message);
    if let Some(master) = get_master(w, id).filter(|&m| mx::player(w, m))
        && let Some(ty) = base_ty(w, master)
    {
        (ty.criminal_action)(w, master, false);
    }
}

/// `DoHarmful`
pub fn do_harmful(w: &mut World, id: MobileId, target: MobileId, indirect: bool) {
    (MOBILE_BASE.do_harmful)(w, id, target, indirect);
    let Some(owner) = get_master(w, id) else {
        return;
    };
    if target == id || target == owner {
        return;
    }
    let attackers: Vec<MobileId> = w
        .mobile(id)
        .map(|m| m.aggressors.iter().map(|a| a.attacker).collect())
        .unwrap_or_default();
    if attackers.contains(&target) {
        return;
    }
    let defenders: Vec<MobileId> = w
        .mobile(id)
        .map(|m| m.aggressed.iter().map(|a| a.defender).collect())
        .unwrap_or_default();
    if defenders.contains(&target)
        && let Some(master) = get_master(w, id)
        && mx::player(w, master)
        && base::can_be_harmful_msg(w, master, target, false)
    {
        base::do_harmful_base(w, master, target, true);
    }
}

/// `OnCombatantChange`
pub fn on_combatant_change(w: &mut World, id: MobileId) {
    (MOBILE_BASE.on_combatant_change)(w, id);
    let alive_target = mx::combatant(w, id).is_some_and(|c| !mx::deleted(w, c) && mx::alive(w, c));
    props_b::set_warmode(w, id, alive_target);
    if ctype(w, id).is_some_and(|c| (c.can_fly)(w, id)) && w.mobile(id).is_some_and(|m| m.warmode) {
        props_b::set_flying(w, id, false);
    }
}

/// OnMapChange / 3099 OnLocationChange: wake the AI if its sector is active.
pub fn on_map_change(w: &mut World, id: MobileId, old: Option<MapId>) {
    crate::ai::check_ai_active(w, id);
    (MOBILE_BASE.on_map_change)(w, id, old);
}

pub fn on_location_change(w: &mut World, id: MobileId, old: Point3D) {
    crate::ai::check_ai_active(w, id);
    (MOBILE_BASE.on_location_change)(w, id, old);
}

/// `OnMovement`
pub fn on_movement(w: &mut World, id: MobileId, m: MobileId, old_location: Point3D) {
    let c = ctype(w, id).expect("creature");
    if should_acquire_on_approach(w, id, m) {
        schedule_acquire_on_approach(w, id);
    } else if (c.reacquire_on_movement)(w, id) {
        (c.force_reacquire)(w, id);
    }
    if let Some(sp) = ctype(w, id).and_then(|c| (c.speech_type)(w, id)) {
        sp.on_movement(w, id, m, old_location);
    }
    // Notice sound
    let (mode, controlled, summoned, pacified) =
        read(w, id, (FightMode::None, false, false, false), |d| {
            (d.fight_mode, d.controlled, d.summoned, d.bard_pacified)
        });
    let (m_hidden, m_access, m_player, m_alive) = w
        .mobile(m)
        .map_or((false, AccessLevel::Player, false, false), |x| {
            (x.hidden, x.access_level, x.player, true)
        });
    if (!m_hidden || m_access == AccessLevel::Player)
        && m_player
        && mode != FightMode::Aggressor
        && mode != FightMode::None
        && mx::combatant(w, id).is_none()
        && !controlled
        && !summoned
        && !pacified
        && mx::in_range_point(w, id, mx::location(w, m), 18)
        && !mx::in_range_point(w, id, old_location, 18)
    {
        if super::misc::body_is_monster(w, id) {
            mx::animate(w, id, 11, 5, 1, true, false, 1);
        }
        let snd = w
            .mobile(id)
            .map(|x| x.ty)
            .map_or(0, |x| (x.get_anger_sound)(w, id));
        mx::play_sound(w, id, snd);
    }
    // MLQuestSystem.Enabled && CanShout && m is PlayerMobile -> CheckShout(pm, oldLocation)
    if (c.can_shout)(w, id)
        && m_player_mobile(w, m)
        && let Some(f) = crate::ml_quest_hooks::ml_quest_creature_hooks(w).and_then(|h| h.movement)
    {
        f(w, id, m, old_location);
    }
    let _ = m_alive;
    call_guards_on_murderer(w, id, m);
}

/// `BaseCreature.m_NoDupeGuards`: the one murderer guards were called on lately (static).
#[derive(Default)]
struct NoDupeGuards(Option<MobileId>);

/// The tail of OnMovement: a human calls the guards on a murderer who walks up.
fn call_guards_on_murderer(w: &mut World, id: MobileId, m: MobileId) {
    use crate::systems::regions::guarded as g;
    if w.resources.get::<NoDupeGuards>().and_then(|n| n.0) == Some(m) {
        return;
    }
    let c = ctype(w, id).expect("creature");
    if !super::misc::body_is_human(w, id)
        || muo_server::mobile::base::murderer(w, id)
        || (c.always_attackable)(w, id)
        || !muo_server::mobile::base::murderer(w, m)
        || !mx::in_range_point(w, m, mx::location(w, id), 12)
        || !mx::alive(w, m)
    {
        return;
    }
    let Some(region) = w
        .mobile(id)
        .and_then(|x| muo_server::region::get_region_of_type(w, x.region, &g::GUARDED_REGION))
    else {
        return;
    };
    if !g::is_disabled(w, region)
        && g::is_guard_candidate(w, region, m)
        && muo_server::mobile::part1::combat::begin_action(w, id, "GuardedRegion")
    {
        let n = 1013037 + rnd::below(w, 16);
        muo_server::mobile::part2::messages::say_number(w, id, n, "");
        g::call_guards(w, region, mx::location(w, id));
        muo_server::timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
            muo_server::mobile::part1::combat::end_action(w, id, "GuardedRegion");
        });
        w.resources.get_mut::<NoDupeGuards>().0 = Some(m);
        muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, |w| {
            w.resources.get_mut::<NoDupeGuards>().0 = None;
        });
    }
}

fn should_acquire_on_approach(w: &mut World, id: MobileId, m: MobileId) -> bool {
    let (mode, controlled, summoned, pacified) =
        read(w, id, (FightMode::None, false, false, false), |d| {
            (d.fight_mode, d.controlled, d.summoned, d.bard_pacified)
        });
    let range = ctype(w, id).map_or(10, |c| (c.acquire_on_approach_range)(w, id));
    if !(mx::combatant(w, id).is_none()
        && !controlled
        && !summoned
        && !pacified
        && mode != FightMode::None
        && mode != FightMode::Aggressor
        && mx::in_range_point(w, id, mx::location(w, m), range))
    {
        return false;
    }
    let c = ctype(w, id).expect("creature");
    (c.is_enemy)(w, id, m) && mx::can_be_harmful_quiet(w, id, m)
}

fn schedule_acquire_on_approach(w: &mut World, id: MobileId) {
    let delay = ctype(w, id).map_or(2000, |c| {
        (c.acquire_on_approach_delay)(w, id).total_milliseconds() as i64
    });
    let deadline = w.tick_count + delay;
    if let Some(d) = cd_mut(w, id)
        && deadline - d.next_reacquire_time < 0
    {
        d.next_reacquire_time = deadline;
    }
    if delay <= 0 {
        // Zero: think now, the ranked scan engages within a wheel turn.
        crate::ai::prod(w, id);
    }
}

/// `OnDoubleClick`
pub fn on_double_click(w: &mut World, id: MobileId, from: MobileId) {
    // from.AccessLevel >= GameMaster && !Body.IsHuman -> Backpack?.DisplayTo(from)
    if w.mobile(from)
        .is_some_and(|f| f.access_level >= AccessLevel::GameMaster)
        && !super::misc::body_is_human(w, id)
        && let Some(pack) = props_b::backpack(w, id)
    {
        muo_server::item::container::display_to_default(w, pack, from);
    }
    death_adder_charm(w, id, from);
    // MLQuestSystem.Enabled && CanGiveMLQuest && from is PlayerMobile
    if let Some(f) = crate::ml_quest_hooks::ml_quest_creature_hooks(w).and_then(|h| h.double_click)
    {
        f(w, id, from);
    }
    (MOBILE_BASE.on_double_click)(w, id, from);
}

/// The DeathAdder charm of `OnDoubleClick`: a snake the player's DeathAdder familiar can turn on a target.
fn death_adder_charm(w: &mut World, id: MobileId, from: MobileId) {
    use muo_server::target::{Target, TargetFlags, Targeted};
    let charmable = ctype(w, id).is_some_and(|c| (c.death_adder_charmable)(w, id));
    let familiar = crate::hooks::hooks(w).and_then(|h| h.death_adder_familiar_of);
    let Some(adder) = familiar.and_then(|f| f(w, from)) else {
        return;
    };
    if !charmable
        || !mx::can_be_harmful_quiet(w, from, id)
        || w.mobile(adder).and_then(|a| a.map) != w.mobile(from).and_then(|f| f.map)
    {
        return;
    }
    let v7000 = w
        .mobile(from)
        .and_then(|f| f.net_state)
        .and_then(|ns| w.net_state(ns))
        .is_some_and(|n| {
            n.caps
                .has_protocol_changes(muo_net::protocol_changes::ProtocolChanges::VERSION_7000)
        });
    if v7000 {
        mx::msg(w, from, 1114362); // You charm the snake.  Select a target to attack.
    } else {
        messages::send_ascii_message(w, from, "You charm the snake.  Select a target to attack.");
    }
    let mut t = Target::new(w, -1, false, TargetFlags::HARMFUL);
    t.handlers.on_target = Some(std::rc::Rc::new(
        move |w: &mut World, from: MobileId, targeted: Targeted| {
            let charmable = ctype(w, id).is_some_and(|c| (c.death_adder_charmable)(w, id));
            if !charmable
                || mx::combatant(w, id).is_some()
                || !mx::can_be_harmful_quiet(w, from, id)
            {
                return;
            }
            let has_adder = crate::hooks::hooks(w)
                .and_then(|h| h.death_adder_familiar_of)
                .and_then(|f| f(w, from))
                .is_some();
            if !has_adder {
                return;
            }
            let Targeted::Entity(muo_server::ids::EntityId::Mobile(targ)) = targeted else {
                return;
            };
            if !mx::can_be_harmful_quiet(w, from, targ) {
                return;
            }
            if let Some(ty) = w.mobile(from).map(|m| m.ty) {
                (ty.revealing_action)(w, from);
            }
            muo_server::mobile::base::do_harmful(w, from, targ, true);
            mx::set_combatant(w, id, Some(targ));
            if crate::ai::ai(w, id).is_some() {
                crate::ai::set_action(w, id, crate::ai::ActionType::Combat);
            }
        },
    ));
    let tid = muo_server::target::add_target(w, t);
    muo_server::target::set_mobile_target(w, from, Some(tid));
}

fn m_player_mobile(w: &World, m: MobileId) -> bool {
    crate::player_mobile::data(w, m).is_some()
}

/// `AddNameProperties`
pub fn add_name_properties(w: &World, id: MobileId, list: &mut PropertyList) {
    (MOBILE_BASE.add_name_properties)(w, id, list);
    let Some(d) = cd(w, id) else { return };
    let c = ctype(w, id).expect("creature");
    // MLQuestSystem.Enabled && CanGiveMLQuest
    if crate::ml_quest_hooks::ml_quest_creature_hooks(w)
        .and_then(|h| h.can_give_quest)
        .is_some_and(|f| f(w, id))
    {
        list.add(1072269); // Quest Giver
    }
    if w.core_ml() {
        if (c.display_weight)(w, id) {
            let tw = w.mobile(id).map_or(0, |m| m.total_weight);
            list.add_str(if tw == 1 { 1072788 } else { 1072789 }, &tw.to_string()); // Weight: ~1_WEIGHT~ stones
        }
        if d.control_order == OrderType::Guard {
            list.add(1080078); // guarding
        }
    }
    let clone = crate::factory::mobile_is(w, id, "Server.Mobiles.Clone");
    if d.summoned && !((c.is_animated_dead)(w, id) || (c.is_necro_familiar)(w, id) || clone) {
        list.add(1049646); // (summoned)
    } else if d.controlled && (c.commandable)(w, id) {
        // Deliberate: show only (bonded), never (bonded) and (tame) together.
        list.add(if d.is_bonded { 1049608 } else { 502006 }); // (bonded) / (tame)
    }
}

/// `OnSingleClick`
pub fn on_single_click(w: &mut World, id: MobileId, from: MobileId) {
    let (controlled, summoned, bonded) = read(w, id, (false, false, false), |d| {
        (d.controlled, d.summoned, d.is_bonded)
    });
    if controlled && ctype(w, id).is_some_and(|c| (c.commandable)(w, id)) {
        let number = if summoned {
            1049646 // (summoned)
        } else if bonded {
            1049608 // (bonded)
        } else {
            502006 // (tame)
        };
        let ns = w.mobile(from).and_then(|m| m.net_state);
        messages::private_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            number,
            "",
            ns,
        );
    }
    (MOBILE_BASE.on_single_click)(w, id, from);
}

/// `RefusesGuardTarget`: a guard never fights its master or the master's other pets.
pub fn refuses_guard_target(w: &mut World, id: MobileId, target: MobileId) -> bool {
    let (controlled, order, provoked) = read(w, id, (false, OrderType::None, false), |d| {
        (d.controlled, d.control_order, d.bard_provoked)
    });
    controlled && order == OrderType::Guard && !provoked && crate::ai::is_guard_ally(w, id, target)
}

/// Combatant setter.
pub fn set_combatant(w: &mut World, id: MobileId, value: Option<MobileId>) {
    if let Some(v) = value
        && refuses_guard_target(w, id, v)
    {
        return;
    }
    base::set_combatant_base(w, id, value);
}

/// `CanBeHarmful(target, message, ignoreOurBlessedness)`
pub fn can_be_harmful_full(
    w: &mut World,
    id: MobileId,
    target: MobileId,
    message: bool,
    ignore_blessed: bool,
) -> bool {
    if crate::factory::mobile_is(w, target, "Server.Factions.BaseFactionGuard") {
        return false;
    }
    let invulnerable =
        cd(w, target).is_some() && ctype(w, target).is_some_and(|c| (c.is_invulnerable)(w, target));
    if invulnerable
        || crate::factory::mobile_is(w, target, "Server.Mobiles.PlayerVendor")
        || crate::factory::mobile_is(w, target, "Server.Mobiles.TownCrier")
    {
        if message {
            let n = base::name(w, target).unwrap_or_default();
            let text = match w.mobile(target).and_then(|m| m.title.clone()) {
                None => format!("{n} cannot be harmed."),
                Some(t) => format!("{n} {t} cannot be harmed."),
            };
            mx::send_message(w, id, &text);
        }
        return false;
    }
    (MOBILE_BASE.can_be_harmful_full)(w, id, target, message, ignore_blessed)
}

/// `CanBeRenamedBy`
pub fn can_be_renamed_by(w: &World, id: MobileId, from: MobileId) -> bool {
    let in_jail = w.mobile(from).is_some_and(|f| {
        muo_server::region::is_part_of_type(w, f.region, &crate::systems::jail::JAIL_REGION)
    });
    control_master(w, id) == Some(from) && !in_jail || (MOBILE_BASE.can_be_renamed_by)(w, id, from)
}

/// `OnRegionChange`
pub fn on_region_change(w: &mut World, id: MobileId, old: RegionId, new: RegionId) {
    (MOBILE_BASE.on_region_change)(w, id, old, new);
    use muo_server::mobile::part2::deps as sd;
    if let Some(sp) = w.mobile(id).and_then(|m| m.spawner)
        && cd(w, id).is_some_and(|d| d.controlled)
        && !sd::spawner_unlink_on_taming(w, sp)
    {
        let accepts = sd::spawner_region(w, sp).is_some_and(|r| {
            w.region(new)
                .is_some_and(|reg| (reg.ty.accepts_spawns_from)(w, new, r))
        });
        if !accepts {
            sd::spawner_remove(w, sp, muo_server::ids::EntityId::Mobile(id));
            if let Some(m) = w.mobile_mut(id) {
                m.spawner = None;
            }
        }
    }
}

/// `GetDamageMaster`
pub fn get_damage_master(w: &World, id: MobileId, damagee: MobileId) -> Option<MobileId> {
    let d = cd(w, id)?;
    if d.bard_provoked && Some(damagee) == d.bard_target {
        return d.bard_master;
    }
    get_master(w, id).or_else(|| (MOBILE_BASE.get_damage_master)(w, id, damagee))
}

/// `CanBeDamaged`
pub fn can_be_damaged(w: &World, id: MobileId) -> bool {
    if is_dead_bonded_pet(w, id) || ctype(w, id).is_some_and(|c| (c.is_invulnerable)(w, id)) {
        return false;
    }
    (MOBILE_BASE.can_be_damaged)(w, id)
}

/// `OnSectorDeactivate`
pub fn on_sector_deactivate(w: &mut World, id: MobileId) {
    let c = ctype(w, id).expect("creature");
    let home = read(w, id, Point3D::ZERO, |d| d.home);
    let range = read(w, id, 0, |d| d.range_home);
    if !mx::deleted(w, id)
        && (c.returns_to_home)(w, id)
        && super::misc::is_spawner_bound(w, id)
        && !mx::in_range_point(w, id, home, range + 5)
    {
        let secs = rnd::below(w, 45) + 15;
        muo_server::timer::start(
            w,
            muo_primitives::TimeSpan::from_secs(secs as i64),
            muo_primitives::TimeSpan::ZERO,
            1,
            move |w| go_home_callback(w, id),
        );
        if let Some(d) = cd_mut(w, id) {
            d.return_queued = true;
        }
    } else if (c.player_range_sensitive)(w, id) {
        crate::ai::deactivate(w, id);
    }
    (MOBILE_BASE.on_sector_deactivate)(w, id);
}

/// `GoHome_Callback`
pub fn go_home_callback(w: &mut World, id: MobileId) {
    let queued = read(w, id, false, |d| d.return_queued);
    if queued && super::misc::is_spawner_bound(w, id) && !sector_active(w, id) {
        let home = read(w, id, Point3D::ZERO, |d| d.home);
        if let Some(ty) = base_ty(w, id) {
            (ty.set_location)(w, id, home, true);
        }
        if ctype(w, id).is_some_and(|c| (c.player_range_sensitive)(w, id)) && !sector_active(w, id)
        {
            crate::ai::deactivate(w, id);
        }
    }
    if let Some(d) = cd_mut(w, id) {
        d.return_queued = false;
    }
}

pub fn sector_active(w: &World, id: MobileId) -> bool {
    let Some(m) = w.mobile(id) else { return false };
    m.map
        .and_then(|map| w.maps.get(map))
        .is_some_and(|map| map.sector_at(m.location.x, m.location.y).active())
}

/// `OnSectorActivate`
pub fn on_sector_activate(w: &mut World, id: MobileId) {
    if ctype(w, id).is_some_and(|c| (c.player_range_sensitive)(w, id)) {
        crate::ai::activate(w, id);
    }
    (MOBILE_BASE.on_sector_activate)(w, id);
}

/// `OnAfterDelete`
pub fn on_after_delete(w: &mut World, id: MobileId) {
    crate::ai::stop_timer(w, id);
    if let Some(d) = cd_mut(w, id) {
        d.ai = None;
    }
    super::control::stop_delete_timer(w, id);
    if let Some(d) = cd_mut(w, id) {
        d.focus_mob = None;
    }
    super::control::stop_unsummon_timer(w, id);
    if let Some(f) = crate::ml_quest_hooks::ml_quest_creature_hooks(w).and_then(|h| h.after_delete)
    {
        f(w, id);
    }
    crate::misc::stamina_system::remove_entry(
        w,
        crate::misc::stamina_system::StepsKey::Entity(muo_server::ids::EntityId::Mobile(id)),
    );
    (MOBILE_BASE.on_after_delete)(w, id);
}

/// `OnDragDrop`
pub fn on_drag_drop(w: &mut World, id: MobileId, from: MobileId, dropped: ItemId) -> bool {
    let c = ctype(w, id).expect("creature");
    if (c.check_feed)(w, id, from, dropped) {
        return true;
    }
    if (c.check_gold)(w, id, from, dropped) {
        return true;
    }
    // Happens for all questers, even those with nothing to offer right now.
    if let Some(f) = crate::ml_quest_hooks::ml_quest_creature_hooks(w).and_then(|h| h.drag_drop)
        && f(w, id, from)
    {
        return false;
    }
    (MOBILE_BASE.on_drag_drop)(w, id, from, dropped)
}

/// `RevealingAction`
pub fn revealing_action(w: &mut World, id: MobileId) {
    if let Some(f) = crate::hooks::hooks(w).and_then(|h| h.invisibility_stop_timer) {
        f(w, id); // InvisibilitySpell.StopTimer(this)
    }
    (MOBILE_BASE.revealing_action)(w, id);
}

#[allow(dead_code)]
fn _unused(w: &World) -> bool {
    world_hooks(w).is_some()
}

/// `GetContextMenuEntries`
pub fn get_context_menu_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>,
) {
    use muo_server::context_menu_entry::ContextMenuEntry;
    use muo_server::ids::EntityId;
    (MOBILE_BASE.get_context_menu_entries)(w, id, from, list);
    let Some(c) = ctype(w, id) else { return };
    if (c.commandable)(w, id) {
        crate::ai::context_menu::get_context_menu_entries(w, id, from, list);
    }
    let (tamable, controlled) = read(w, id, (false, false), |d| (d.tamable_raw, d.controlled));
    if tamable && !controlled && mx::alive(w, from) {
        // TameEntry(enabled): number 6130, range 6.
        let female = w.mobile(from).is_some_and(|m| m.female);
        let mut e = ContextMenuEntry::new(6130, 6);
        e.enabled = if female {
            (c.allow_female_tamer)(w, id)
        } else {
            (c.allow_male_tamer)(w, id)
        };
        e.on_click = Some(std::rc::Rc::new(
            |w: &mut World, from: MobileId, target: EntityId| {
                if let EntityId::Mobile(bc) = target {
                    if !muo_server::mobile::part2::stats::check_alive(w, from, true)
                        || !crate::creature::is_creature(w, bc)
                    {
                        return;
                    }
                    // `from.TargetLocked`, `AnimalTaming.DisableMessage`, `UseSkill(AnimalTaming)` then
                    // `from.Target.Invoke(from, bc)`: `skills::animal::tame_from_context_menu`.
                    if let Some(f) = hooks(w).and_then(|h| h.tame_from_context_menu) {
                        f(w, from, bc);
                    }
                }
            },
        ));
        list.push(e);
    }
    (c.add_custom_context_entries)(w, id, from, list);
    super::teach::add_teach_entries(w, id, from, list);
}
