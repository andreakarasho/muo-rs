//! The behavior of `BaseCreature`'s virtual members and helpers (everything between the declarations and the
//! `Set*`/`Pack*` helpers).

use muo_primitives::{BodyTable, DateTime, MessageType, TimeSpan, utility};
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{effects, enumerators, messages};
use muo_server::mobile::{AccessLevel, base};
use muo_server::timer;
use muo_server::world::World;

use super::control::{control_master, get_master, summon_master};
use super::data::{cd, cd_mut, read};
use super::enums::*;
use super::vtable::ctype;
use crate::abilities::{self, MonsterAbilityTrigger};
use crate::factory::mobile_is;
use crate::hooks::{ContentHooks, hooks, rnd, world_hooks};
use crate::mx;

pub use crate::mx::deleted;

// ---- statics (BaseCreature.Configure) -------------------------------------------------------------------

/// `BondingEnabled` and `PetsStandDownOnCommand`.
#[derive(Default)]
pub struct CreatureStatics {
    pub bonding_enabled: bool,
    pub pets_stand_down_on_command: bool,
}

pub fn bonding_enabled(w: &World) -> bool {
    w.resources
        .get::<CreatureStatics>()
        .is_some_and(|s| s.bonding_enabled)
}

pub fn pets_stand_down_on_command(w: &World) -> bool {
    w.resources
        .get::<CreatureStatics>()
        .is_some_and(|s| s.pets_stand_down_on_command)
}

/// 4310 `BaseCreature.Configure`.
pub fn configure(w: &mut World) {
    // `item is IMountItem`: MountItem (its mount) and VirtualMountItem (its rider) are the mount items here.
    w.resources
        .get_mut::<muo_server::mobile::part1::props_b::PropsBStatics>()
        .mount_of_item = Some(|w, item| {
        let it = w.item(item)?;
        if it.ty.is(&super::mount::MOUNT_ITEM) {
            return super::mount::mount_of_item(w, item);
        }
        if it.ty.is(&super::virtual_mount::VIRTUAL_MOUNT_ITEM) {
            return super::virtual_mount::rider(w, item);
        }
        if crate::items::is_named(it.ty, "Server.Mobiles.EtherealMount") {
            return crate::hooks::hooks(w)
                .and_then(|h| h.ethereal_mount_rider)
                .and_then(|f| f(w, item));
        }
        None
    });
    super::mount::configure(w);
    {
        use crate::misc::stamina_system::StaminaHooks;
        let h = w.resources.get_mut::<StaminaHooks>();
        h.mount_of = Some(|w, rider| {
            let item = w.mobile(rider)?.mount_item?;
            match super::mount::mount_of_item(w, item) {
                Some(m) => Some(muo_server::ids::EntityId::Mobile(m)),
                None => Some(muo_server::ids::EntityId::Item(item)),
            }
        });
        h.step_traits = Some(|w, e| match e {
            muo_server::ids::EntityId::Mobile(m) => super::mount::step_traits(w, m),
            // EtherealMount: StepsMax 3840 ("should be same as horse"), one step per second of idling
            muo_server::ids::EntityId::Item(i)
                if w.item(i).is_some_and(|x| {
                    crate::items::is_named(x.ty, "Server.Mobiles.EtherealMount")
                }) =>
            {
                Some(crate::misc::stamina_system::StepTraits {
                    steps_max: 3840,
                    steps_gained_per_idle_time: 1,
                    idle_time_per_steps_gain: TimeSpan::from_secs(1),
                })
            }
            _ => None,
        });
    }
    let lbr = w.expansion() >= muo_data::expansion::Expansion::LBR;
    let ml = w.core_ml();
    let bonding = w.config.get_setting("taming.enableBonding", lbr);
    let stand_down = w.config.get_setting("taming.petsStandDownOnCommand", ml);
    let s = w.resources.get_mut::<CreatureStatics>();
    s.bonding_enabled = bonding;
    s.pets_stand_down_on_command = stand_down;
    crate::hooks::hooks_mut(w).aos_damage = Some(crate::misc::aos_helpers::damage_hook);
    w.resources
        .get_mut::<crate::systems::context_menus::ContextMenuHooks>()
        .creature_teach = Some(super::teach::teach_hook);
    {
        let h = w.resources.get_mut::<crate::hooks::WorldHooks>();
        h.compute_kill_awards = Some(|w, id| {
            let map = muo_server::mobile::part1::lifecycle::map(w, id);
            crate::misc::titles::compute_kill_awards(w, id, map)
        });
        h.award_fame_karma = Some(|w, m, fame, karma| {
            crate::misc::titles::award_fame(w, m, fame, true);
            crate::misc::titles::award_karma(w, m, karma, true);
        });
    }
}

// ---- body helpers ---------------------------------------------------------------------------------------

fn body_is(w: &World, id: MobileId, f: impl Fn(&BodyTable, muo_primitives::Body) -> bool) -> bool {
    match (w.resources.get::<BodyTable>(), w.mobile(id)) {
        (Some(t), Some(_)) => f(t, muo_server::mobile::part1::props_b::body(w, id)),
        _ => false,
    }
}

pub fn body_is_human(w: &World, id: MobileId) -> bool {
    body_is(w, id, |t, b| t.is_human(b))
}
pub fn body_is_animal(w: &World, id: MobileId) -> bool {
    body_is(w, id, |t, b| t.is_animal(b))
}
pub fn body_is_monster(w: &World, id: MobileId) -> bool {
    body_is(w, id, |t, b| t.is_monster(b))
}
pub fn body_is_sea(w: &World, id: MobileId) -> bool {
    body_is(w, id, |t, b| t.is_sea(b))
}

/// `GetDistanceToSqrt(other)`.
pub fn distance_sqrt(w: &World, a: MobileId, b: MobileId) -> f64 {
    match (w.mobile(a), w.mobile(b)) {
        (Some(x), Some(y)) => {
            utility::distance_xy(x.location.x, x.location.y, y.location.x, y.location.y)
        }
        _ => f64::MAX,
    }
}

// ---- simple virtuals ------------------------------------------------------------------------------------

/// `IsAnimatedDead`: summoned and one of the Animate Dead classes.
pub fn is_animated_dead(w: &World, id: MobileId) -> bool {
    const TYPES: [&str; 15] = [
        "MoundOfMaggots",
        "HellSteed",
        "SkeletalMount",
        "WailingBanshee",
        "Wraith",
        "SkeletalDragon",
        "LichLord",
        "FleshGolem",
        "Lich",
        "SkeletalKnight",
        "BoneKnight",
        "Mummy",
        "SkeletalMage",
        "BoneMagi",
        "PatchworkSkeleton",
    ];
    // `type == m_AnimateDeadTypes[i]`: the exact class, not a subclass.
    read(w, id, false, |d| d.summoned) && TYPES.contains(&super::type_name(w, id))
}

/// `CanOpenDoors`
pub fn can_open_doors(w: &World, id: MobileId) -> bool {
    !body_is_animal(w, id) && !body_is_sea(w, id)
}

/// `CanMoveOverObstacles`
pub fn can_move_over_obstacles(w: &World, id: MobileId) -> bool {
    props_a::core_aos(w) || body_is_monster(w, id)
}

/// `ReturnsToHome`
pub fn returns_to_home(w: &World, id: MobileId) -> bool {
    read(w, id, false, |d| {
        d.seeks_home
            && d.home != muo_primitives::Point3D::ZERO
            && !d.return_queued
            && !d.controlled
            && !d.summoned
    })
}

/// `IsMonster`
pub fn is_monster(w: &World, id: MobileId) -> bool {
    let controlled = read(w, id, false, |d| d.controlled);
    if !controlled {
        return true;
    }
    match get_master(w, id) {
        Some(m) if cd(w, m).is_some() => ctype(w, m).is_some_and(|c| (c.is_monster)(w, m)),
        _ => false,
    }
}

/// `IsHumanInTown`
pub fn is_human_in_town(w: &World, id: MobileId) -> bool {
    body_is_human(w, id)
        && world_hooks(w)
            .and_then(|h| h.in_guarded_region)
            .is_some_and(|f| f(w, id))
}

pub fn get_faction_allegiance(w: &World, id: MobileId, mob: MobileId) -> Allegiance {
    hooks(w)
        .and_then(|h| h.faction_allegiance)
        .map_or(Allegiance::None, |f| f(w, id, mob))
}

pub fn get_ethic_allegiance(w: &World, id: MobileId, mob: MobileId) -> Allegiance {
    hooks(w)
        .and_then(|h| h.ethic_allegiance)
        .map_or(Allegiance::None, |f| f(w, id, mob))
}

// ---- delete timer ---------------------------------------------------------------------------------------

/// `new DeleteTimer(this, delay).Start()` stored as `_pendingDeleteTimer`.
pub fn start_delete_timer(w: &mut World, id: MobileId, delay: TimeSpan) {
    let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        if let Some(ty) = w.mobile(id).map(|m| m.ty) {
            (ty.delete)(w, id);
        }
    });
    if let Some(d) = cd_mut(w, id) {
        d.pending_delete_timer = h;
    }
}

// ---- IsEnemy ---------------------------------------------------------------------------------------------

pub fn is_enemy(w: &mut World, id: MobileId, m: MobileId) -> bool {
    let group = ctype(w, id).and_then(|c| (c.opposition_group)(w, id));
    if group.is_some_and(|g| g.is_enemy(w, id, m)) {
        return true;
    }
    if mobile_is(w, m, "Server.Mobiles.BaseGuard") {
        return false;
    }
    if ctype(w, id).is_some_and(|c| (c.get_faction_allegiance)(w, id, m) == Allegiance::Ally) {
        return false;
    }
    let c_data = cd(w, m).is_some();
    let c_master = if c_data { get_master(w, m) } else { None };

    let our_shielded_target = c_master.unwrap_or(m);
    let our_ethic = ctype(w, id).and_then(|c| (c.ethic_allegiance)(w, id));
    if hooks(w)
        .and_then(|h: &ContentHooks| h.ethic_shielded)
        .is_some_and(|f| f(w, our_shielded_target, our_ethic))
    {
        return false;
    }
    // Player-only protections; they must veto before the non-creature early return below.
    if hooks(w)
        .and_then(|h| h.enemy_protection)
        .is_some_and(|f| f(w, m))
    {
        return false;
    }
    if !c_data || mobile_is(w, m, "Server.Engines.Quests.Haven.MilitiaFighter") {
        return true;
    }
    let (our_team, our_mode) = read(w, id, (0, FightMode::None), |d| (d.team, d.fight_mode));
    let (their_team, their_mode) = read(w, m, (0, FightMode::None), |d| (d.team, d.fight_mode));
    let (our_karma, their_karma) = (
        w.mobile(id).map_or(0, |x| x.karma),
        w.mobile(m).map_or(0, |x| x.karma),
    );
    if our_team != their_team
        || our_mode == FightMode::Evil && their_karma < 0
        || their_mode == FightMode::Evil && our_karma < 0
    {
        return true;
    }
    let Some(master) = get_master(w, id) else {
        // Non-summons will attack summons of non-NPCs
        return c_master.is_some_and(|cm| cd(w, cm).is_none());
    };
    // Summons will attack others summons, if they are enemies with their master
    // Pets will attack non-summons, but not other summons (legacy logic)
    if cd(w, master).is_some() {
        let target = c_master.unwrap_or(m);
        ctype(w, master).is_some_and(|c| (c.is_enemy)(w, master, target))
    } else {
        c_master.is_none()
    }
}

// ---- combat hooks ---------------------------------------------------------------------------------------

/// `OnGotMeleeAttack`
pub fn on_got_melee_attack(w: &mut World, id: MobileId, attacker: MobileId, _damage: i32) {
    let Some(c) = ctype(w, id) else { return };
    if (c.auto_dispel)(w, id)
        && cd(w, attacker).is_some()
        && ctype(w, attacker).is_some_and(|a| (a.is_dispellable)(w, attacker))
        && (c.auto_dispel_chance)(w, id) > rnd::double(w)
    {
        (c.dispel)(w, id, attacker);
    }
    abilities::trigger_ability(
        w,
        id,
        MonsterAbilityTrigger::TAKE_MELEE_DAMAGE,
        Some(attacker),
    );
}

/// `Dispel`
pub fn dispel(w: &mut World, _id: MobileId, m: MobileId) {
    let (loc, map) = (mx::location(w, m), mx::map(w, m));
    let item = muo_server::item::effect_item::create(
        w,
        loc,
        map,
        muo_server::item::effect_item::DEFAULT_DURATION,
    );
    muo_server::effects::send_location_particles(w, item, 0x3728, 8, 20, 5042);
    muo_server::effects::play_sound_entity(w, mx::entity(m), 0x201);
    if let Some(ty) = w.mobile(m).map(|x| x.ty) {
        (ty.delete)(w, m);
    }
}

/// `OnGaveMeleeAttack`
pub fn on_gave_melee_attack(w: &mut World, id: MobileId, defender: MobileId, _damage: i32) {
    let Some(c) = ctype(w, id) else { return };
    let mut p = (c.hit_poison)(w, id);
    if read(w, id, false, |d| d.is_paragon)
        && let Some(pp) = w.resources.get::<muo_server::poison::Poisons>()
    {
        p = pp.increase_level(p);
    }
    if let Some(p) = p
        && (c.hit_poison_chance)(w, id) >= rnd::double(w)
    {
        if let Some(ty) = w.mobile(defender).map(|m| m.ty) {
            (ty.apply_poison)(w, defender, Some(id), p);
        }
        if read(w, id, false, |d| d.controlled) {
            let cap = muo_server::skills::cap(w, id, muo_server::skills::SkillName::Poisoning);
            mx::check_skill(w, id, muo_server::skills::SkillName::Poisoning, 0.0, cap);
        }
    }
    if (c.auto_dispel)(w, id)
        && cd(w, defender).is_some()
        && ctype(w, defender).is_some_and(|a| (a.is_dispellable)(w, defender))
        && (c.auto_dispel_chance)(w, id) > rnd::double(w)
    {
        (c.dispel)(w, id, defender);
    }
    abilities::trigger_ability(
        w,
        id,
        MonsterAbilityTrigger::GIVE_MELEE_DAMAGE,
        Some(defender),
    );
}

/// `GetFightModeRanking`
pub fn get_fight_mode_ranking(
    w: &mut World,
    id: MobileId,
    m: MobileId,
    mode: FightMode,
    player_only: bool,
) -> f64 {
    if player_only && !mx::player(w, m) {
        return f64::MIN;
    }
    match mode {
        // returns strongest mobile
        FightMode::Strongest => {
            let tactics = muo_server::skills::value(w, m, muo_server::skills::SkillName::Tactics);
            tactics + w.mobile(m).map_or(0, |x| (x.ty.str)(w, m)) as f64
        }
        // returns weakest mobile
        FightMode::Weakest => -(mx::hits(w, m) as f64),
        _ => -distance_sqrt(w, id, m),
    }
}

/// `Turn`: negative = left, positive = right.
pub fn turn(w: &mut World, id: MobileId, steps: i32) {
    let Some(v) = w.mobile(id).map(|m| m.direction.bits() as i32) else {
        return;
    };
    let d =
        muo_primitives::Direction::from_bits_retain((((v & 0x7) + steps) & 0x7 | (v & 0x80)) as u8);
    props_b::set_direction(w, id, d);
}

/// `TurnInternal`
pub fn turn_internal(w: &mut World, id: MobileId, steps: i32) {
    let Some(v) = w.mobile(id).map(|m| m.direction.bits() as i32) else {
        return;
    };
    let d =
        muo_primitives::Direction::from_bits_retain((((v & 0x7) + steps) & 0x7 | (v & 0x80)) as u8);
    effects::set_direction(w, id, d);
}

/// `GetTeamSize`
pub fn get_team_size(w: &mut World, id: MobileId, range: i32) -> i32 {
    let team = read(w, id, 0, |d| d.team);
    let mut count = 0;
    for m in enumerators::get_mobiles_in_range(w, id, range) {
        if m != id
            && cd(w, m).is_some_and(|c| c.team == team)
            && !mx::deleted(w, m)
            && w.mobile(id)
                .map(|x| x.ty)
                .is_some_and(|x| (x.can_see_mobile)(w, id, m))
        {
            count += 1;
        }
    }
    count
}

/// `CheckedAnimate`: mounted creatures don't animate.
#[allow(clippy::too_many_arguments)] // one parameter per input the check reads
pub fn checked_animate(
    w: &mut World,
    id: MobileId,
    action: i32,
    frames: i32,
    repeat: i32,
    forward: bool,
    rep: bool,
    delay: i32,
) {
    if !props_b::mounted(w, id) {
        mx::animate(w, id, action, frames, repeat, forward, rep, delay);
    }
}

/// `CheckIdle`: true when the creature is idling (and so shouldn't move).
pub fn check_idle(w: &mut World, id: MobileId) -> bool {
    if mx::combatant(w, id).is_some() {
        return false; // in combat, not idling
    }
    let release = read(w, id, DateTime::MIN, |d| d.idle_release_time);
    if release > DateTime::MIN {
        if w.now >= release {
            if let Some(d) = cd_mut(w, id) {
                d.idle_release_time = DateTime::MIN;
            }
            return false; // idle is over
        }
        return true; // still idling
    }
    if rnd::below(w, 100) < 95 {
        return false; // chose not to enter the idle state
    }
    let secs = super::speeds::random_idle_seconds(w);
    let until = w.now + TimeSpan::from_secs(secs as i64);
    if let Some(d) = cd_mut(w, id) {
        d.idle_release_time = until;
    }
    let c = ctype(w, id).expect("creature");
    if body_is_human(w, id) {
        let a = if rnd::bool(w) { 5 } else { 6 };
        (c.checked_animate)(w, id, a, 5, 1, true, false, 1);
    } else if body_is_animal(w, id) {
        match rnd::below(w, 3) {
            0 => (c.checked_animate)(w, id, 3, 3, 1, true, false, 1),
            1 => (c.checked_animate)(w, id, 9, 5, 1, true, false, 1),
            _ => (c.checked_animate)(w, id, 10, 5, 1, true, false, 1),
        }
    } else if body_is_monster(w, id) {
        let a = if rnd::bool(w) { 17 } else { 18 };
        (c.checked_animate)(w, id, a, 5, 1, true, false, 1);
    }
    let snd = w
        .mobile(id)
        .map(|m| m.ty)
        .map_or(0, |m| (m.get_idle_sound)(w, id));
    mx::play_sound(w, id, snd);
    true // entered idle state
}

// ---- flee -----------------------------------------------------------------------------------------------

pub fn stop_flee(w: &mut World, id: MobileId) {
    if let Some(d) = cd_mut(w, id) {
        d.end_flee_time = DateTime::MIN;
    }
}

pub fn check_flee(w: &mut World, id: MobileId) -> bool {
    let end = read(w, id, DateTime::MIN, |d| d.end_flee_time);
    if end == DateTime::MIN {
        return false;
    }
    if w.now >= end {
        (ctype(w, id).expect("creature").stop_flee)(w, id);
        return false;
    }
    true
}

pub fn begin_flee(w: &mut World, id: MobileId, max: TimeSpan) {
    let end = w.now + max;
    if let Some(d) = cd_mut(w, id) {
        d.end_flee_time = end;
    }
}

// ---- OnThink --------------------------------------------------------------------------------------------

pub fn on_think(w: &mut World, id: MobileId) {
    const ENABLE_RUMMAGING: bool = true;
    const CHANCE_TO_RUMMAGE: f64 = 0.5;
    let tc = w.tick_count;
    let c = ctype(w, id).expect("creature");
    let (summoned, controlled, next_rummage) = read(w, id, (false, false, 0), |d| {
        (d.summoned, d.controlled, d.next_rummage_time)
    });
    if ENABLE_RUMMAGING
        && (c.can_rummage_corpses)(w, id)
        && !summoned
        && !controlled
        && tc - next_rummage >= 0
    {
        let (min, max) = if rnd::double(w) < CHANCE_TO_RUMMAGE && (c.rummage)(w, id) {
            (1.0, 4.0)
        } else {
            (0.25, 0.75)
        };
        let delay = min + rnd::double(w) * (max - min);
        if let Some(d) = cd_mut(w, id) {
            d.next_rummage_time = tc + TimeSpan::from_mins_f64(delay).total_milliseconds() as i64;
        }
    }
    // Fire breath, etc.
    let combatant = mx::combatant(w, id);
    abilities::trigger_ability(w, id, MonsterAbilityTrigger::THINK, combatant);

    let bard_pacified = read(w, id, false, |d| d.bard_pacified);
    let healing = read(w, id, false, |d| d.heal_timer.running(w));
    if ((c.can_heal)(w, id) || (c.can_heal_owner)(w, id))
        && mx::alive(w, id)
        && !healing
        && !bard_pacified
    {
        let owner = control_master(w, id);
        let (next_owner, next_self) = read(w, id, (0, 0), |d| {
            (d.next_heal_owner_time, d.next_heal_time)
        });
        // Order matters: the message-sending CanBeBeneficial runs before the map and range tests.
        let owner_ok = owner.is_some_and(|o| {
            (c.can_heal_owner)(w, id)
                && tc - next_owner >= 0
                && can_be_beneficial(w, id, o)
                && mx::map(w, o) == mx::map(w, id)
                && mx::in_range(w, o, id, (c.heal_start_range)(w, id))
                && mx::in_los(w, id, o)
                && (mx::hits(w, o) as f64)
                    < (c.heal_owner_trigger)(w, id) * mx::hits_max(w, o) as f64
        });
        if owner_ok {
            let o = owner.expect("checked");
            (c.heal_start)(w, id, o);
            let next = tc + (c.heal_owner_interval)(w, id).mul_add(1000.0, 0.0) as i64;
            if let Some(d) = cd_mut(w, id) {
                d.next_heal_owner_time = next;
            }
        } else if (c.can_heal)(w, id)
            && tc - next_self >= 0
            && can_be_beneficial(w, id, id)
            && ((mx::hits(w, id) as f64) < (c.heal_trigger)(w, id) * mx::hits_max(w, id) as f64
                || w.mobile(id).is_some_and(|m| m.poison.is_some()))
        {
            (c.heal_start)(w, id, id);
            let next = tc + ((c.heal_interval)(w, id) * 1000.0) as i64;
            if let Some(d) = cd_mut(w, id) {
                d.next_heal_time = next;
            }
        }
    }

    if (c.returns_to_home)(w, id) && is_spawner_bound(w, id) && !in_range_home(w, id) {
        let (combatant, warmode) = (
            mx::combatant(w, id),
            w.mobile(id).is_some_and(|m| m.warmode),
        );
        if combatant.is_none() && !warmode && rnd::double(w) < 0.10 {
            /* some throttling */
            let home = read(w, id, muo_primitives::Point3D::ZERO, |d| d.home);
            let dir = effects::get_direction_to(w, id, home.x, home.y, false);
            let moved = w
                .mobile(id)
                .map(|m| m.ty)
                .is_some_and(|ty| (ty.move_dir)(w, id, dir));
            let fails = read(w, id, 0, |d| d.failed_return_home);
            let fails = if !moved { fails + 1 } else { 0 };
            if let Some(d) = cd_mut(w, id) {
                d.failed_return_home = fails;
            }
            if fails > 5 {
                if let Some(ty) = w.mobile(id).map(|m| m.ty) {
                    (ty.set_location)(w, id, home, true);
                }
                if let Some(d) = cd_mut(w, id) {
                    d.failed_return_home = 0;
                }
            }
        }
    } else if let Some(d) = cd_mut(w, id) {
        d.failed_return_home = 0;
    }

    let next_aura = read(w, id, 0, |d| d.next_aura);
    if (c.has_aura)(w, id) && tc - next_aura >= 0 {
        (c.aura_damage)(w, id);
        let next = tc + (c.aura_interval)(w, id).total_milliseconds() as i64;
        if let Some(d) = cd_mut(w, id) {
            d.next_aura = next;
        }
    }
}

fn in_range_home(w: &World, id: MobileId) -> bool {
    let (home, range) = read(w, id, (muo_primitives::Point3D::ZERO, 0), |d| {
        (d.home, d.range_home)
    });
    mx::in_range_point(w, id, home, range)
}

/// `IsSpawnerBound`
pub fn is_spawner_bound(w: &World, id: MobileId) -> bool {
    let Some(m) = w.mobile(id) else { return false };
    let Some(d) = cd(w, id) else { return false };
    // `(Spawner as Spawner)?.Map == Map`: no spawner reads as not bound.
    let spawner_map = m
        .spawner
        .and_then(|sp| w.item(muo_server::ids::ItemId(muo_primitives::Serial::new(sp.0))))
        .map(|i| i.map);
    m.map.is_some_and(|map| map != MapId::INTERNAL)
        && d.fight_mode != FightMode::None
        && d.range_home >= 0
        && !d.controlled
        && !d.summoned
        && spawner_map == Some(m.map)
}

/// `CanBeBeneficial(target, true, true)`.
fn can_be_beneficial(w: &mut World, id: MobileId, target: MobileId) -> bool {
    match w.mobile(id).map(|m| m.ty) {
        Some(ty) => (ty.can_be_beneficial_full)(w, id, target, true, true),
        None => false,
    }
}

// ---- heal / aura ----------------------------------------------------------------------------------------

/// `HealStart`
pub fn heal_start(w: &mut World, id: MobileId, patient: MobileId) {
    let c = ctype(w, id).expect("creature");
    (c.stop_heal)(w, id);
    let on_self = patient == id;
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.revealing_action)(w, id);
    }
    if !on_self {
        if let Some(ty) = w.mobile(patient).map(|m| m.ty) {
            (ty.revealing_action)(w, patient);
        }
        let name = base::name(w, id).unwrap_or_default();
        // 1008078 is appended after the name: ": Attempting to heal you."
        messages::send_localized_message_affix(w, patient, 1008078, false, &name, "", 0x3B2);
    }
    let seconds = (if on_self {
        (c.heal_delay)(w, id)
    } else {
        (c.heal_owner_delay)(w, id)
    }) + if mx::alive(w, patient) { 0.0 } else { 5.0 };
    let h = timer::start(
        w,
        TimeSpan::from_secs_f64(seconds),
        TimeSpan::ZERO,
        1,
        move |w| {
            if let Some(c) = ctype(w, id) {
                (c.heal)(w, id, patient);
            }
        },
    );
    if let Some(d) = cd_mut(w, id) {
        d.heal_timer = h;
    }
}

/// `Heal`
pub fn heal(w: &mut World, id: MobileId, patient: MobileId) {
    use muo_server::skills::{self, SkillName};
    let c = ctype(w, id).expect("creature");
    if !mx::alive(w, id)
        || mx::map(w, id) == Some(MapId::INTERNAL)
        || !can_be_beneficial(w, id, patient)
        || mx::map(w, patient) != mx::map(w, id)
        || !mx::in_range(w, id, patient, (c.heal_end_range)(w, id))
    {
        (c.stop_heal)(w, id);
        return;
    }
    if !mx::in_range(w, id, patient, (c.heal_start_range)(w, id)) {
        return;
    }
    let on_self = patient == id;
    let poison = w.mobile(patient).and_then(|m| m.poison);
    let healing = skills::value(w, id, SkillName::Healing);
    let anatomy = skills::value(w, id, SkillName::Anatomy);
    if let Some(p) = poison {
        let level = muo_server::poison::level(w, p);
        let chance = (healing - 30.0) / 50.0 - level as f64 * 0.1;
        if healing >= 60.0 && anatomy >= 60.0 && chance > rnd::double(w) {
            let cured = w
                .mobile(patient)
                .map(|m| m.ty)
                .is_some_and(|ty| (ty.cure_poison)(w, patient, Some(id)));
            if cured {
                mx::msg(w, patient, 1010059); // You have been cured of all poisons.
                mx::check_skill(w, id, SkillName::Healing, 0.0, 60.0 + level as f64 * 10.0); //TODO Verify formula
                mx::check_skill(w, id, SkillName::Anatomy, 0.0, 100.0);
            }
        }
    } else if hooks_bleeding(w, patient) {
        mx::msg(w, patient, 1060167); // The bleeding wounds have healed, you are no longer bleeding!
        crate::items::weapon_ability::end_bleed(w, patient, false);
    } else {
        let chance = (healing + 10.0) / 100.0;
        if chance > rnd::double(w) {
            let min = anatomy / 10.0 + healing / 6.0 + 4.0;
            let mut max = anatomy / 8.0 + healing / 3.0 + 4.0;
            if on_self {
                max += 10.0;
            }
            let to_heal = (min + rnd::double(w) * (max - min)) * (c.heal_scalar)(w, id);
            muo_server::mobile::part2::damage::heal(w, patient, to_heal as i32);
            mx::check_skill(w, id, SkillName::Healing, 0.0, 90.0);
            mx::check_skill(w, id, SkillName::Anatomy, 0.0, 100.0);
        }
    }
    (c.heal_effect)(w, id, patient);
    (c.stop_heal)(w, id);
    let (hits, hmax) = (mx::hits(w, id) as f64, mx::hits_max(w, id) as f64);
    let (phits, pmax) = (mx::hits(w, patient) as f64, mx::hits_max(w, patient) as f64);
    if on_self && (c.heal_fully)(w, id) && hits >= (c.heal_trigger)(w, id) * hmax && hits < hmax
        || !on_self
            && (c.heal_owner_fully)(w, id)
            && phits >= (c.heal_owner_trigger)(w, id) * pmax
            && phits < pmax
    {
        (c.heal_start)(w, id, patient);
    }
}

fn hooks_bleeding(w: &World, patient: MobileId) -> bool {
    crate::items::weapon_ability::is_bleeding(w, patient)
}

/// `StopHeal`
pub fn stop_heal(w: &mut World, id: MobileId) {
    let h = cd(w, id).map(|d| d.heal_timer);
    if let Some(mut h) = h {
        h.cancel(w);
        if let Some(d) = cd_mut(w, id) {
            d.heal_timer = h;
        }
    }
}

/// `AuraDamage`
pub fn aura_damage(w: &mut World, id: MobileId) {
    let c = ctype(w, id).expect("creature");
    let dead_bonded = w
        .mobile(id)
        .is_some_and(|m| (m.ty.is_dead_bonded_pet)(w, id));
    if !mx::alive(w, id) || dead_bonded {
        return;
    }
    let team = read(w, id, 0, |d| d.team);
    let aos = props_a::core_aos(w);
    let mut queue = Vec::new();
    for m in enumerators::get_mobiles_in_range(w, id, (c.aura_range)(w, id)) {
        if m != id
            && base::can_be_harmful_msg(w, id, m, false)
            && (aos || mx::in_los(w, id, m))
            && (cd(w, m).is_some_and(|b| b.controlled || b.summoned || b.team != team)
                || mx::player(w, m))
        {
            queue.push(m);
        }
    }
    for m in queue {
        let d = crate::hooks::AosDamage {
            amount: (c.aura_base_damage)(w, id),
            ignore_armor: false,
            phys: (c.aura_physical_damage)(w, id),
            fire: (c.aura_fire_damage)(w, id),
            cold: (c.aura_cold_damage)(w, id),
            pois: (c.aura_poison_damage)(w, id),
            nrgy: (c.aura_energy_damage)(w, id),
            chaos: (c.aura_chaos_damage)(w, id),
            direct: 0,
        };
        match hooks(w).and_then(|h| h.aos_damage) {
            Some(f) => {
                f(w, m, Some(id), d);
            }
            None => mx::damage(w, m, d.amount, Some(id)),
        }
        (c.aura_effect)(w, id, m);
    }
}

// ---- remaining item/system dependent members -------------------------------------------------------------

/// What `OnCarve` reads of the dead creature. A creature's corpse could outlive its (deleted) object; here the
/// mobile is gone from the world, so the corpse keeps these values from the time it was made.
#[derive(Clone, Debug)]
pub struct CarveSnapshot {
    feathers: i32,
    wool: i32,
    meat: i32,
    hides: i32,
    scales: i32,
    meat_type: MeatType,
    hide_type: HideType,
    /// `ScaleType` may roll a random number: asked at carve time, with the dead creature's id.
    scale_type: fn(&mut World, MobileId) -> ScaleType,
    owner: MobileId,
    summoned: bool,
    bonded: bool,
}

/// The `Feathers`/`Wool`/`Meat`/`Hides`/`Scales` and types of creature `id`, `None` for a mobile that is not one.
pub fn carve_snapshot(w: &World, id: MobileId) -> Option<CarveSnapshot> {
    let c = ctype(w, id)?;
    let (summoned, bonded) = read(w, id, (false, false), |d| (d.summoned, d.is_bonded));
    Some(CarveSnapshot {
        feathers: (c.feathers)(w, id),
        wool: (c.wool)(w, id),
        meat: (c.meat)(w, id),
        hides: (c.hides)(w, id),
        scales: (c.scales)(w, id),
        meat_type: (c.meat_type)(w, id),
        hide_type: (c.hide_type)(w, id),
        scale_type: c.scale_type,
        owner: id,
        summoned,
        bonded,
    })
}

/// `OnCarve`
pub fn on_carve(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    corpse: ItemId,
    _with: Option<ItemId>,
) {
    if let Some(snap) = carve_snapshot(w, id) {
        on_carve_snapshot(w, &snap, from, corpse);
    }
}

/// `OnCarve` of a creature whose object is gone: the values kept by its corpse.
pub fn on_carve_snapshot(w: &mut World, c: &CarveSnapshot, from: MobileId, corpse: ItemId) {
    use crate::factory::make_item;
    let (mut feathers, mut wool, mut meat, mut hides, mut scales) =
        (c.feathers, c.wool, c.meat, c.hides, c.scales);
    let (summoned, bonded) = (c.summoned, c.bonded);
    let animated = world_hooks(w)
        .and_then(|h| h.corpse_animated)
        .is_some_and(|f| f(w, corpse));
    if feathers == 0 && wool == 0 && meat == 0 && hides == 0 && scales == 0
        || summoned
        || bonded
        || animated
    {
        if animated {
            // Use this on corpses to carve away meat and hide
            muo_server::item::messages::send_localized_message_to(w, corpse, from, 500464, "");
        } else {
            mx::msg(w, from, 500485); // You see nothing useful to carve from the corpse.
        }
        return;
    }
    let ml = w.core_ml();
    if ml && w.mobile(from).and_then(|m| m.race).is_none_or(|r| r.0 == 0) {
        hides = (hides as f64 * 1.1).ceil() as i32; // 10% bonus only applies to hides, ore & logs
    }
    let corpse_map = w.item(corpse).and_then(|i| i.map);
    if corpse_map == Some(MapId::FELUCCA) {
        feathers *= 2;
        wool *= 2;
        hides *= 2;
        if ml {
            meat *= 2;
            scales *= 2;
        }
    }
    let loc = w
        .item(corpse)
        .map_or(muo_primitives::Point3D::ZERO, |i| i.location);
    if let (Some(blood), Some(map)) = (make_item(w, "Server.Items.Blood"), corpse_map) {
        // `new Blood(0x122D)`
        muo_server::item::base::set_item_id(w, blood, 0x122D);
        muo_server::item::ops::move_to_world_map(w, blood, loc, Some(map));
    }
    let add = |w: &mut World, name: &str, amount: i32| {
        if let Some(it) = make_item(w, name) {
            muo_server::item::ops::set_amount(w, it, amount);
            if let Some(f) = world_hooks(w).and_then(|h| h.corpse_add_carved_item) {
                f(w, corpse, it, from);
            }
        }
    };
    if feathers != 0 {
        add(w, "Server.Items.Feather", feathers);
        mx::msg(w, from, 500479); // You pluck the bird. The feathers are now on the corpse.
    }
    if wool != 0 {
        add(w, "Server.Items.TaintedWool", wool);
        mx::msg(w, from, 500483); // You shear it, and the wool is now on the corpse.
    }
    if meat != 0 {
        match c.meat_type {
            MeatType::Ribs => add(w, "Server.Items.RawRibs", meat),
            MeatType::Bird => add(w, "Server.Items.RawBird", meat),
            MeatType::LambLeg => add(w, "Server.Items.RawLambLeg", meat),
        }
        mx::msg(w, from, 500467); // You carve some meat, which remains on the corpse.
    }
    if hides != 0 {
        let hide_type = c.hide_type;
        let knife = crate::core::aos(w)
            && matches!(muo_server::mobile::base::weapon(w, from), Some(muo_server::interfaces::WeaponRef::Item(i))
                if crate::factory::item_is(w, i, "Server.Items.SkinningKnife"));
        let name = match (knife, hide_type) {
            (true, HideType::Regular) => "Server.Items.Leather",
            (true, HideType::Spined) => "Server.Items.SpinedLeather",
            (true, HideType::Horned) => "Server.Items.HornedLeather",
            (true, HideType::Barbed) => "Server.Items.BarbedLeather",
            (false, HideType::Regular) => "Server.Items.Hides",
            (false, HideType::Spined) => "Server.Items.SpinedHides",
            (false, HideType::Horned) => "Server.Items.HornedHides",
            (false, HideType::Barbed) => "Server.Items.BarbedHides",
        };
        if let Some(it) = make_item(w, name) {
            muo_server::item::ops::set_amount(w, it, hides);
            if knife && muo_server::mobile::part2::items::place_in_backpack(w, from, it) {
                // You skin it and place the cut-up hides in your backpack.
                mx::msg(w, from, 1073555);
            } else {
                muo_server::item::container::drop_item_default(w, corpse, it);
                mx::msg(w, from, 500471); // You skin it, and the hides are now in the corpse.
            }
        } else {
            mx::msg(w, from, 500471);
        }
    }
    if scales != 0 {
        let names: &[&str] = match (c.scale_type)(w, c.owner) {
            ScaleType::Red => &["Server.Items.RedScales"],
            ScaleType::Yellow => &["Server.Items.YellowScales"],
            ScaleType::Black => &["Server.Items.BlackScales"],
            ScaleType::Green => &["Server.Items.GreenScales"],
            ScaleType::White => &["Server.Items.WhiteScales"],
            ScaleType::Blue => &["Server.Items.BlueScales"],
            ScaleType::All => &[
                "Server.Items.RedScales",
                "Server.Items.YellowScales",
                "Server.Items.BlackScales",
                "Server.Items.GreenScales",
                "Server.Items.WhiteScales",
                "Server.Items.BlueScales",
            ],
        };
        for n in names {
            add(w, n, scales);
        }
        mx::msg(w, from, 1079284); // You cut away some scales, but they remain on the corpse.
    }
    if let Some(f) = world_hooks(w).and_then(|h| h.corpse_set_carved) {
        f(w, corpse);
    }
    if world_hooks(w)
        .and_then(|h| h.corpse_is_criminal_action)
        .is_some_and(|f| f(w, corpse, from))
        && let Some(ty) = w.mobile(from).map(|m| m.ty)
    {
        (ty.criminal_action)(w, from, true);
    }
}

/// `CheckGold`
pub fn check_gold(w: &mut World, id: MobileId, from: MobileId, dropped: ItemId) -> bool {
    crate::factory::item_is(w, dropped, "Server.Items.Gold")
        && (ctype(w, id).expect("creature").on_gold_given)(w, id, from, dropped)
}

/// `OnGoldGiven`
pub fn on_gold_given(w: &mut World, id: MobileId, from: MobileId, dropped: ItemId) -> bool {
    let amount = w.item(dropped).map_or(0, |i| i.amount);
    let c = ctype(w, id).expect("creature");
    if (c.check_teaching_match)(w, id, from) {
        if super::teach::teach_for_gold(w, id, from, amount) {
            muo_server::item::base::delete(w, dropped);
            return true;
        }
    } else if (c.is_human_in_town)(w, id) {
        let dir = mx::get_direction_to(w, id, from);
        mx::set_direction(w, id, dir);
        let old = w.mobile(id).map_or(0, |m| m.speech_hue);
        if let Some(m) = w.mobile_mut(id) {
            m.speech_hue = 0x23F;
        }
        mx::say_to(w, id, from, "Thou art giving me gold?");
        mx::say_to(
            w,
            id,
            from,
            if amount >= 400 {
                "'Tis a noble gift."
            } else {
                "Money is always welcome."
            },
        );
        if let Some(m) = w.mobile_mut(id) {
            m.speech_hue = 0x3B2;
        }
        muo_server::mobile::part2::speech::say_to_number(w, id, from, 501548, ""); // I thank thee.
        if let Some(m) = w.mobile_mut(id) {
            m.speech_hue = old;
        }
        muo_server::item::base::delete(w, dropped);
        return true;
    }
    false
}

/// `OnKilledBy`
pub fn on_killed_by(w: &mut World, id: MobileId, mob: MobileId) {
    // MondainsLegacy/Paragon artifact chances (the engines task) run through this hook.
    if let Some(f) = world_hooks(w).and_then(|h| h.on_killed_by_rights_holder) {
        f(w, id, mob);
    }
}

/// `Rummage`
pub fn rummage(w: &mut World, id: MobileId) -> bool {
    if props_b::backpack(w, id).is_none() {
        return false;
    }
    let corpse = muo_server::mobile::part2::enumerators::get_items_in_range(w, id, 2)
        .into_iter()
        .find(|&c| {
            crate::factory::item_is(w, c, "Server.Items.Corpse")
                && w.item(c).is_some_and(|i| !i.items.is_empty())
        });
    let Some(corpse) = corpse else { return false };
    let count = w.item(corpse).map_or(0, |i| i.items.len());
    for _ in 0..count {
        let items = w.item(corpse).map(|i| i.items.clone()).unwrap_or_default();
        if items.is_empty() {
            break;
        }
        let item = items[rnd::below(w, items.len() as i32) as usize];
        let amount = w.item(item).map_or(1, |i| i.amount);
        let (rejected, _) = muo_server::mobile::base::lift(w, id, item, amount);
        if !rejected
            && muo_server::mobile::base::drop_mobile(
                w,
                id,
                id,
                muo_primitives::Point3D::new(-1, -1, 0),
            )
        {
            // *rummages through a corpse and takes an item*
            messages::public_overhead_message_number(
                w,
                id,
                MessageType::EMOTE,
                0x3B2,
                1008086,
                "",
                false,
            );
            //TODO Instance rummaged loot
            return true;
        }
    }
    false
}

/// `ResurrectPet`
pub fn resurrect_pet(w: &mut World, id: MobileId) {
    if !read(w, id, false, |d| d.is_dead_pet) {
        return;
    }
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return;
    };
    (ty.on_before_resurrect)(w, id);
    props_b::set_poison(w, id, None);
    props_b::set_warmode(w, id, false);
    props_b::set_hits(w, id, 10);
    let stam_max = (ty.stam_max)(w, id);
    props_b::set_stam(w, id, stam_max);
    props_b::set_mana(w, id, 0);
    (ty.process_delta)(w, id);
    if let Some(d) = cd_mut(w, id) {
        d.is_dead_pet = false;
    }
    let (serial, loc, map) = w
        .mobile(id)
        .map_or((Default::default(), Default::default(), None), |m| {
            (m.serial, m.location, m.map)
        });
    let bonded = muo_server::packets::mobile::create_bonded_status(serial, false);
    muo_server::effects::send_packet(w, loc, map, &bonded);
    muo_server::mobile::part2::effects::send_incoming_packet(w, id);
    (ty.on_after_resurrect)(w, id);
    crate::ai::activate(w, id);
    let owner = control_master(w, id);
    let near = owner.is_some_and(|o| {
        !mx::deleted(w, o)
            && mx::map(w, o) == mx::map(w, id)
            && mx::in_range(w, o, id, 12)
            && w.mobile(id)
                .map(|m| m.ty)
                .is_some_and(|m| (m.can_see_mobile)(w, id, o))
    }) && owner.is_some_and(|o| mx::in_los(w, id, o));
    let now = w.now;
    if let Some(d) = cd_mut(w, id) {
        if near {
            d.owner_abandon_time = DateTime::MIN;
        } else if d.owner_abandon_time == DateTime::MIN {
            d.owner_abandon_time = now;
        }
    }
    (ty.check_stat_timers)(w, id);
}

/// `NewHarmfulItem`: `new Acid(TimeSpan.FromSeconds(10), 30, 30)`.
pub fn new_harmful_item(w: &mut World, _id: MobileId) -> Option<ItemId> {
    let make = crate::hooks::hooks(w).and_then(|h| h.new_acid);
    match make {
        Some(f) => Some(f(
            w,
            TimeSpan::from_secs(10),
            TimeSpan::from_secs(1),
            30,
            30,
        )),
        None => crate::factory::make_item(w, "Server.Items.Acid"),
    }
}

/// `CheckFoodPreference`
pub fn check_food_preference(w: &World, id: MobileId, f: ItemId) -> bool {
    let favorite = ctype(w, id).map_or(FoodType::empty(), |c| (c.favorite_food)(w, id));
    for ty in [
        FoodType::EGGS,
        FoodType::FISH,
        FoodType::GRAINS_AND_HAY,
        FoodType::MEAT,
        FoodType::FRUITS_AND_VEGGIES,
        FoodType::GOLD,
        FoodType::METAL,
        FoodType::LEATHER,
    ] {
        if favorite.contains(ty) && food_matches(w, f, ty) {
            return true;
        }
    }
    false
}

fn food_matches(w: &World, f: ItemId, ty: FoodType) -> bool {
    use crate::factory::item_is;
    let in_list = |list: &[&str]| list.iter().any(|n| item_is(w, f, n));
    use crate::craft::{CraftResourceType, resource_type};
    use crate::items::armor::{self, ArmorMaterialType as M};
    let item_ty = w.item(f).map(|i| i.ty);
    let is = |t: &'static muo_server::item::ItemType| item_ty.is_some_and(|x| x.is(t));
    // `BaseArmor.MaterialType`, then the `Resource` of a weapon or clothing; None when it is neither.
    let armor_material = || is(&armor::BASE_ARMOR).then(|| armor::class(w, f).material_type);
    let resource_kind = || {
        if is(&crate::items::weapon::BASE_WEAPON) {
            crate::items::weapon::data(w, f).map(|d| resource_type(d.resource))
        } else if is(&crate::items::clothing::BASE_CLOTHING) {
            crate::items::clothing::data(w, f).map(|d| resource_type(d.resource))
        } else {
            None
        }
    };
    if ty == FoodType::LEATHER {
        // Goat special case
        if in_list(&[
            "Server.Items.BaseLeather",
            "Server.Items.Bag",
            "Server.Items.Pouch",
            "Server.Items.Backpack",
            "Server.Items.StrongBackpack",
        ]) {
            return true;
        }
        if let Some(m) = armor_material() {
            return matches!(
                m,
                M::Leather | M::Studded | M::Spined | M::Horned | M::Barbed
            );
        }
        return resource_kind() == Some(CraftResourceType::Leather);
    }
    if ty == FoodType::METAL {
        if let Some(m) = armor_material() {
            return matches!(m, M::Ringmail | M::Chainmail | M::Plate);
        }
        if let Some(kind) = resource_kind() {
            return kind == CraftResourceType::Metal;
        }
        return in_list(&[
            "Server.Items.BaseIngot",
            "Server.Items.BaseOre",
            "Server.Items.MetalChest",
            "Server.Items.MetalGoldenChest",
            "Server.Items.MetalBox",
            "Server.Items.Gears",
            "Server.Items.Saw",
            "Server.Items.Axle",
            "Server.Items.AxleGears",
            "Server.Items.ClockParts",
            "Server.Items.Hinge",
            "Server.Items.Springs",
            "Server.Items.Spyglass",
            "Server.Items.Fork",
            "Server.Items.ForkLeft",
            "Server.Items.ForkRight",
            "Server.Items.Spoon",
            "Server.Items.SpoonLeft",
            "Server.Items.SpoonRight",
            "Server.Items.Knife",
            "Server.Items.KnifeLeft",
            "Server.Items.KnifeRight",
            "Server.Items.DrawKnife",
            "Server.Items.Hammer",
            "Server.Items.Froe",
            "Server.Items.Inshave",
            "Server.Items.Nails",
            "Server.Items.RunicDovetailSaw",
            "Server.Items.RunicHammer",
            "Server.Items.Skillet",
            "Server.Items.SledgeHammer",
            "Server.Items.Tongs",
            "Server.Items.TinkerTools",
            "Server.Items.SmithHammer",
            "Server.Items.AncientSmithyHammer",
            "Server.Items.Scorp",
        ]);
    }
    if ty == FoodType::EGGS {
        in_list(&["Server.Items.FriedEggs", "Server.Items.Eggs"])
    } else if ty == FoodType::FISH {
        in_list(&["Server.Items.FishSteak", "Server.Items.RawFishSteak"])
    } else if ty == FoodType::GRAINS_AND_HAY {
        in_list(&[
            "Server.Items.BreadLoaf",
            "Server.Items.FrenchBread",
            "Server.Items.SheafOfHay",
        ])
    } else if ty == FoodType::MEAT {
        in_list(&[
            "Server.Items.Bacon",
            "Server.Items.CookedBird",
            "Server.Items.Sausage",
            "Server.Items.Ham",
            "Server.Items.Ribs",
            "Server.Items.LambLeg",
            "Server.Items.ChickenLeg",
            "Server.Items.RawBird",
            "Server.Items.RawRibs",
            "Server.Items.RawLambLeg",
            "Server.Items.RawChickenLeg",
            "Server.Items.Head",
            "Server.Items.LeftArm",
            "Server.Items.LeftLeg",
            "Server.Items.Torso",
            "Server.Items.RightArm",
            "Server.Items.RightLeg",
        ])
    } else if ty == FoodType::FRUITS_AND_VEGGIES {
        in_list(&[
            "Server.Items.HoneydewMelon",
            "Server.Items.YellowGourd",
            "Server.Items.GreenGourd",
            "Server.Items.Banana",
            "Server.Items.Bananas",
            "Server.Items.Lemon",
            "Server.Items.Lime",
            "Server.Items.Dates",
            "Server.Items.Grapes",
            "Server.Items.Peach",
            "Server.Items.Pear",
            "Server.Items.Apple",
            "Server.Items.Watermelon",
            "Server.Items.Squash",
            "Server.Items.Cantaloupe",
            "Server.Items.Carrot",
            "Server.Items.Cabbage",
            "Server.Items.Onion",
            "Server.Items.Lettuce",
            "Server.Items.Pumpkin",
        ])
    } else {
        // White wyrms eat gold.
        in_list(&["Server.Items.Gold"])
    }
}

/// `CheckFeed`
pub fn check_feed(w: &mut World, id: MobileId, from: MobileId, dropped: ItemId) -> bool {
    use muo_server::skills::{self, SkillName};
    let c = ctype(w, id).expect("creature");
    let (dead, controlled) = read(w, id, (true, false), |d| (d.is_dead_pet, d.controlled));
    if dead || !controlled || control_master(w, id) != Some(from) && !(c.is_pet_friend)(w, id, from)
    {
        return false;
    }
    if !(c.check_food_preference)(w, id, dropped) {
        let ns = w.mobile(from).and_then(|m| m.net_state);
        messages::private_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            1043257,
            "",
            ns,
        ); // The animal shies away.
        return false;
    }
    let amount = w.item(dropped).map_or(0, |i| i.amount);
    if amount <= 0 {
        return false;
    }
    let is_gold = crate::factory::item_is(w, dropped, "Server.Items.Gold");
    let stam_gain = if is_gold {
        amount - 50
    } else {
        amount * 15 - 50
    };
    if stam_gain > 0 {
        let s = w.mobile(id).map_or(0, |m| m.stam);
        props_b::set_stam(w, id, s + stam_gain);
        crate::misc::stamina_system::regen_steps(
            w,
            crate::misc::stamina_system::StepsKey::Entity(muo_server::ids::EntityId::Mobile(id)),
            stam_gain * 4,
            true,
        );
    }
    if props_a::core_se(w) {
        super::control::set_loyalty(w, id, MAX_LOYALTY);
    } else if read(w, id, MAX_LOYALTY, |d| d.loyalty) < MAX_LOYALTY {
        let inc = utility::coin_flips_max(&mut w.rng, amount, MAX_LOYALTY_INCREASE) * 10;
        if inc > 0 {
            let l = read(w, id, 0, |d| d.loyalty);
            super::control::set_loyalty(w, id, MAX_LOYALTY.min(l + inc));
            muo_server::mobile::part2::speech::say_to_number(w, id, from, 502060, ""); // Your pet looks happier.
        }
    }
    if body_is_animal(w, id) {
        mx::animate(w, id, 3, 5, 1, true, false, 0);
    } else if body_is_monster(w, id) {
        mx::animate(w, id, 17, 5, 1, true, false, 0);
    }
    let (bondable, bonded) = ((c.is_bondable)(w, id), read(w, id, false, |d| d.is_bonded));
    if bondable && !bonded {
        let master = control_master(w, id);
        if master == Some(from) {
            // So friends can't start the bonding process
            let min_tame = read(w, id, 0.0, |d| d.min_tame_skill);
            let base_taming = skills::base(w, from, SkillName::AnimalTaming);
            let value_taming = skills::value(w, from, SkillName::AnimalTaming);
            if min_tame <= 29.1
                || base_taming >= min_tame
                || (c.override_bonding_reqs)(w, id)
                || w.core_ml() && value_taming >= min_tame
            {
                let begin = read(w, id, DateTime::MIN, |d| d.bonding_begin);
                if begin == DateTime::MIN {
                    let now = w.now;
                    if let Some(d) = cd_mut(w, id) {
                        d.bonding_begin = now;
                    }
                } else if begin + (c.bonding_delay)(w, id) <= w.now {
                    super::control::set_is_bonded(w, id, true);
                    if let Some(d) = cd_mut(w, id) {
                        d.bonding_begin = DateTime::MIN;
                    }
                    mx::msg(w, from, 1049666); // Your pet has bonded with you!
                }
            } else if w.core_ml() {
                // Your pet cannot form a bond with you until your animal taming ability has risen.
                mx::msg(w, from, 1075268);
            }
        }
    }
    muo_server::item::base::delete(w, dropped);
    true
}

/// `CheckTeach`
pub fn check_teach(w: &World, id: MobileId, skill: i32, from: MobileId) -> bool {
    use muo_server::skills::SkillName;
    let base = |s| f64::from(super::teach::rd(w, from, s).base) / 10.0;
    if !ctype(w, id).is_some_and(|c| (c.can_teach)(w, id)) {
        return false;
    }
    if skill == SkillName::Stealth as i32
        && base(SkillName::Hiding) < crate::systems::skills::stealth::hiding_requirement(w)
    {
        return false;
    }
    if skill == SkillName::RemoveTrap as i32
        && (base(SkillName::Lockpicking) < 50.0 || base(SkillName::DetectHidden) < 50.0)
    {
        return false;
    }
    props_a::core_aos(w)
        || skill != SkillName::Focus as i32
            && skill != SkillName::Chivalry as i32
            && skill != SkillName::Necromancy as i32
}

/// `CheckTeachingMatch`
pub fn check_teaching_match(w: &World, id: MobileId, m: MobileId) -> bool {
    let teaching = read(w, id, -1, |d| d.teaching);
    if teaching == -1 {
        return false;
    }
    if mobile_is(w, m, "Server.Mobiles.PlayerMobile") {
        return crate::hooks::pet_player_hooks(w)
            .and_then(|h| h.learning)
            .is_some_and(|f| f(w, m) == teaching);
    }
    true
}

/// `DropBackpack`
pub fn drop_backpack(w: &mut World, id: MobileId) {
    let Some(pack) = props_b::backpack(w, id) else {
        return;
    };
    if w.item(pack).is_none_or(|i| i.items.is_empty()) {
        return;
    }
    // `backpack.EnumerateItems()`: every nested item too, breadth first
    let contents = muo_server::item::enumerable::find_items(w, pack, true, |_, _| true);
    if contents.is_empty() {
        return;
    }
    let Some(b) = crate::factory::make_item(w, "Server.Items.CreatureBackpack") else {
        return;
    };
    let name = muo_server::mobile::base::name(w, id);
    muo_server::item::ops::set_name(w, b, name);
    for it in contents {
        if let Some(ty) = w.item(b).map(|i| i.ty) {
            (ty.drop_item)(w, b, it);
        }
    }
    let house = world_hooks(w)
        .and_then(|h| h.house_ban_location)
        .and_then(|f| f(w, id));
    let (loc, map) = match house {
        Some((l, m)) => (l, Some(m)),
        None => (mx::location(w, id), mx::map(w, id)),
    };
    muo_server::item::ops::move_to_world_map(w, b, loc, map);
}

/// Used by the vtable defaults that read the summoner.
pub fn summoner(w: &World, id: MobileId) -> Option<MobileId> {
    summon_master(w, id)
}

#[allow(dead_code)]
fn _unused(_: AccessLevel) {}
