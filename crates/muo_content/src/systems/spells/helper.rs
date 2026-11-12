//! `DefensiveSpell`, `SpellHelper` (travel rules, damage, stat buffs, summons,
//! reflection), `TransformationSpellHelper` and its contexts.

use indexmap::IndexMap;
use muo_primitives::{Point3D, Rectangle2D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::ops as item_ops;
use muo_server::map::MapId;
use muo_server::mobile::part1::{combat, props_b};
use muo_server::mobile::part2::{effects as fx, messages as mm};
use muo_server::mobile::{StatType, base as mb};
use muo_server::mods::StatMod;
use muo_server::region;
use muo_server::skills::{self, SkillName};
use muo_server::target::Targeted;
use muo_server::timer::{self, TimerHandle};

use super::hooks::hooks;
use super::spell::{self, SpellId, spell_type};
use super::types::TravelCheckType;
use crate::core;
use crate::creature::{self, ctype};
use crate::misc::aos_helpers::{self, DamageOptions, DamageSplit};
use crate::misc::notoriety;
use muo_server::notoriety::INNOCENT;

const AOS_DAMAGE_DELAY: TimeSpan = TimeSpan::from_millis(1250);
const OLD_DAMAGE_DELAY: TimeSpan = TimeSpan::from_millis(500);
const COMBAT_HEAT_DELAY: TimeSpan = TimeSpan::from_secs(30);
const RESTRICT_TRAVEL_COMBAT: bool = true;

const OFFSETS: [i32; 16] = [-1, -1, -1, 0, -1, 1, 0, -1, 0, 1, 1, -1, 1, 0, 1, 1];

fn send(w: &mut World, to: MobileId, number: i32) {
    mm::send_localized_message(w, to, number, "", 0x3B2);
}

// ---- DefensiveSpell ---------------------------------------------------------------------------------------------

/// `DefensiveSpell.Nullify(from)`: for a minute the mobile cannot begin another defensive action.
pub fn defensive_spell_nullify(w: &mut World, from: MobileId) {
    if !combat::can_begin_action(w, from, "DefensiveSpell") {
        timer::start(w, TimeSpan::from_mins(1), TimeSpan::ZERO, 1, move |w| {
            combat::end_action(w, from, "DefensiveSpell")
        });
    }
}

// ---- positions --------------------------------------------------------------------------------------------------

/// An `IPoint3D`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Point3dRef {
    Entity(EntityId),
    Point(Point3D),
}

impl From<Targeted> for Point3dRef {
    fn from(t: Targeted) -> Self {
        match t {
            Targeted::Entity(e) => Self::Entity(e),
            Targeted::Land(l) => Self::Point(l.location),
            Targeted::Static(s) => Self::Point(s.location),
        }
    }
}

/// `new Point3D(ip)` / `ip.X, ip.Y, ip.Z`.
pub fn point_of(w: &World, p: Point3dRef) -> Point3D {
    match p {
        Point3dRef::Point(p) => p,
        Point3dRef::Entity(EntityId::Mobile(m)) => {
            w.mobile(m).map(|m| m.location).unwrap_or_default()
        }
        Point3dRef::Entity(EntityId::Item(i)) => w.item(i).map(|i| i.location).unwrap_or_default(),
    }
}

/// `GetDamageDelayForSpell(sp)`.
pub fn get_damage_delay_for_spell(w: &World, sp: SpellId) -> TimeSpan {
    if !spell_type(w, sp).is_some_and(|t| t.delayed_damage_in(core::aos(w))) {
        TimeSpan::ZERO
    } else if core::aos(w) {
        AOS_DAMAGE_DELAY
    } else {
        OLD_DAMAGE_DELAY
    }
}

/// `CheckMulti(p, map, houses = true, housingrange = 0)`.
pub fn check_multi(
    w: &mut World,
    p: Point3D,
    map: Option<MapId>,
    houses: bool,
    housing_range: i32,
) -> bool {
    let Some(map) = map.filter(|&m| m != MapId::INTERNAL) else {
        return false;
    };
    let multis: Vec<ItemId> =
        muo_server::map::enumerate::multis_in_sector(w, Some(map), p.x, p.y).collect();
    for multi in multis {
        let is_house = item_is(w, multi, "Server.Multis.BaseHouse");
        if is_house {
            let f = hooks(w).and_then(|h| h.house_check);
            if let Some(f) = f
                && ((houses && f(w, multi, p, true, 16))
                    || (housing_range > 0 && f(w, multi, p, false, housing_range)))
            {
                return true;
            }
        } else if hooks(w)
            .and_then(|h| h.multi_contains)
            .is_some_and(|f| f(w, multi, p))
        {
            return true;
        }
    }
    false
}

/// `Turn(from, to)`: face the target (an item's top-level carrier if it has one).
pub fn turn(w: &mut World, from: MobileId, to: Option<Point3dRef>) {
    let Some(mut to) = to else { return };
    if to == Point3dRef::Entity(EntityId::Mobile(from)) {
        return;
    }
    if let Point3dRef::Entity(EntityId::Item(item)) = to
        && let Some(root) = item_ops::root_parent(w, item)
    {
        if root == EntityId::Mobile(from) {
            return;
        }
        to = Point3dRef::Entity(root);
    }
    let p = point_of(w, to);
    let d = mm_direction_to(w, from, p);
    props_b::set_direction(w, from, d);
}

fn mm_direction_to(w: &World, from: MobileId, p: Point3D) -> muo_primitives::Direction {
    fx::get_direction_to(w, from, p.x, p.y, false)
}

/// `CheckCombat(m)`: in recent combat with a player.
pub fn check_combat(w: &World, m: MobileId) -> bool {
    if !RESTRICT_TRAVEL_COMBAT {
        return false;
    }
    let Some(mob) = w.mobile(m) else { return false };
    let now = w.now;
    for info in &mob.aggressed {
        if w.mobile(info.defender).is_some_and(|d| d.player)
            && now - info.last_combat_time < COMBAT_HEAT_DELAY
        {
            return true;
        }
    }
    if core::aos(w) {
        for info in &mob.aggressors {
            if w.mobile(info.attacker).is_some_and(|a| a.player)
                && info.attacker == m
                && now - info.last_combat_time < COMBAT_HEAT_DELAY
            {
                return true;
            }
        }
    }
    false
}

/// `AdjustField(ref p, map, height, mobsBlock)`.
pub fn adjust_field(
    w: &mut World,
    p: &mut Point3D,
    map: Option<MapId>,
    height: i32,
    mobs_block: bool,
) -> bool {
    let Some(map) = map else { return false };
    for offset in 0..10 {
        let loc = Point3D::new(p.x, p.y, p.z - offset);
        if muo_server::map::fit::can_fit(
            w, map, loc.x, loc.y, loc.z, height, true, mobs_block, true,
        ) {
            *p = loc;
            return true;
        }
    }
    false
}

/// `GetEastToWest(from, target)`.
pub fn get_east_to_west(from: Point3D, target: Point3D) -> bool {
    let dx = from.x - target.x;
    let dy = from.y - target.y;
    let rx = (dx - dy) * 44;
    let ry = (dx + dy) * 44;
    (rx >= 0 && ry < 0) || (ry >= 0 && rx < 0)
}

/// `CanRevealCaster(m)`: an uncontrolled creature.
pub fn can_reveal_caster(w: &World, m: MobileId) -> bool {
    creature::cd(w, m).is_some_and(|c| !c.controlled)
}

/// `GetSurfaceTop(ref ip)`.
pub fn get_surface_top(w: &World, p: Point3dRef) -> Point3dRef {
    match p {
        Point3dRef::Entity(EntityId::Item(i)) => match item_ops::get_surface_top(w, i) {
            item_ops::SurfaceTop::Point(p) => Point3dRef::Point(p),
            item_ops::SurfaceTop::Item(i) => Point3dRef::Entity(EntityId::Item(i)),
        },
        other => other,
    }
}

/// `GetSurfaceTop` of a `StaticTarget`: its z, less the tile's height when the tile isn't a surface.
pub fn static_target_surface_top(w: &World, target: muo_server::target::StaticTarget) -> Point3D {
    let item = w.maps.tile_data.item(target.item_id as usize);
    let (surface, calc_height) = (
        item.flags.contains(muo_data::tile_data::TileFlag::SURFACE),
        item.calc_height(),
    );
    let mut z = target.location.z;
    if !surface {
        z -= calc_height;
    }
    Point3D::new(target.location.x, target.location.y, z)
}

// ---- stat buffs and curses --------------------------------------------------------------------------------------

fn stat_name(t: StatType) -> &'static str {
    if t == StatType::STR {
        "Str"
    } else if t == StatType::DEX {
        "Dex"
    } else if t == StatType::INT {
        "Int"
    } else {
        "All"
    }
}

/// `AddStatOffset(m, type, offset, duration)`.
pub fn add_stat_offset(
    w: &mut World,
    m: MobileId,
    t: StatType,
    offset: i32,
    duration: TimeSpan,
) -> bool {
    if offset > 0 {
        add_stat_bonus_value(w, m, m, t, offset, duration, false)
    } else {
        offset >= 0 || add_stat_curse_value(w, m, m, t, -offset, duration, false)
    }
}

/// `AddStatBonus(caster, target, type, duration, skillCheck = true)`.
pub fn add_stat_bonus(
    w: &mut World,
    caster: MobileId,
    target: MobileId,
    t: StatType,
    duration: TimeSpan,
    skill_check: bool,
) -> bool {
    let bonus = get_offset(w, caster, target, t, false);
    add_stat_bonus_value(w, caster, target, t, bonus, duration, skill_check)
}

/// `AddStatBonus(caster, target, type, bonus, duration, skillCheck = false)`.
pub fn add_stat_bonus_value(
    w: &mut World,
    caster: MobileId,
    target: MobileId,
    t: StatType,
    bonus: i32,
    duration: TimeSpan,
    skill_check: bool,
) -> bool {
    let name = format!("[Magic] {} Buff", stat_name(t));

    if let Some(index) = mb::get_stat_mod(w, target, &name, false) {
        let (offset, id) = {
            let m = &w.mobile(target).expect("mod found").stat_mods[index];
            (m.offset, m.id)
        };
        if offset >= bonus {
            return false;
        }
        mb::remove_stat_mod(w, target, id);
    }

    if skill_check {
        muo_server::mobile::part2::stats::check_skill(w, caster, SkillName::EvalInt, 0.0, 120.0);
    }

    let m = StatMod::new(w, t, Some(&name), bonus, duration, None);
    mb::add_stat_mod(w, target, m);
    true
}

/// `GetCurse(caster, target, type)`.
pub fn get_curse(w: &World, _caster: MobileId, target: MobileId, t: StatType) -> i32 {
    let name = format!("[Magic] {} Curse", stat_name(t));
    mb::get_stat_mod(w, target, &name, false)
        .and_then(|i| w.mobile(target).map(|m| m.stat_mods[i].offset))
        .unwrap_or(0)
}

/// `AddStatCurse(caster, target, type, duration, skillCheck = true)`.
pub fn add_stat_curse(
    w: &mut World,
    caster: MobileId,
    target: MobileId,
    t: StatType,
    duration: TimeSpan,
    skill_check: bool,
) -> bool {
    let curse = get_offset(w, caster, target, t, true);
    add_stat_curse_value(w, caster, target, t, curse, duration, skill_check)
}

/// `AddStatCurse(caster, target, type, curse, duration, skillCheck = false)`.
pub fn add_stat_curse_value(
    w: &mut World,
    caster: MobileId,
    target: MobileId,
    t: StatType,
    curse: i32,
    duration: TimeSpan,
    skill_check: bool,
) -> bool {
    let malus = -curse;
    let name = format!("[Magic] {} Curse", stat_name(t));

    if let Some(index) = mb::get_stat_mod(w, target, &name, false) {
        let (offset, id) = {
            let m = &w.mobile(target).expect("mod found").stat_mods[index];
            (m.offset, m.id)
        };
        if offset <= malus {
            return false;
        }
        mb::remove_stat_mod(w, target, id);
    }

    if skill_check {
        let stats = muo_server::mobile::part2::stats::check_skill;
        stats(w, caster, SkillName::EvalInt, 0.0, 120.0);
        stats(w, target, SkillName::MagicResist, 0.0, 120.0);
    }

    let m = StatMod::new(w, t, Some(&name), malus, duration, None);
    mb::add_stat_mod(w, target, m);
    true
}

/// `GetDuration(caster, target)`.
pub fn get_duration(w: &mut World, caster: MobileId, _target: MobileId) -> TimeSpan {
    // TODO: Is this accurate for Curse? Sources say it is magery. Should confirm at least for newest era.
    let skill = if core::aos(w) {
        SkillName::EvalInt
    } else {
        SkillName::Magery
    };
    TimeSpan::from_secs_f64(6.0 * skills::value(w, caster, skill) / 5.0)
}

/// `GetOffsetScalar(caster, target, curse)`.
pub fn get_offset_scalar(w: &mut World, caster: MobileId, target: MobileId, curse: bool) -> f64 {
    let ei = skills::value(w, caster, SkillName::EvalInt);
    let mut percent = if curse {
        8.0 + (ei - skills::value(w, target, SkillName::MagicResist)) / 10.0
    } else {
        1.0 + ei / 10.0
    };
    percent *= 0.01;
    percent.max(0.0)
}

/// `GetOffset(caster, target, type, curse)`.
pub fn get_offset(
    w: &mut World,
    caster: MobileId,
    target: MobileId,
    t: StatType,
    curse: bool,
) -> i32 {
    if !core::aos(w) {
        return 1 + (skills::value(w, caster, SkillName::Magery) * 0.1) as i32;
    }
    let percent = get_offset_scalar(w, caster, target, curse);
    let raw = w
        .mobile(target)
        .map_or((0, 0, 0), |m| (m.str, m.dex, m.int));
    let v = if t == StatType::STR {
        raw.0
    } else if t == StatType::DEX {
        raw.1
    } else {
        raw.2
    };
    (f64::from(v) * percent) as i32
}

// ---- targets ----------------------------------------------------------------------------------------------------

/// `GetGuildFor(m)`: the mobile's guild, or its master's.
pub fn get_guild_for(w: &World, m: MobileId) -> Option<u32> {
    let f = hooks(w).and_then(|h| h.guild_of)?;
    f(w, m).or_else(|| creature::control::get_master(w, m).and_then(|master| f(w, master)))
}

/// `ValidIndirectTarget(from, to)`.
pub fn valid_indirect_target(w: &mut World, from: MobileId, to: MobileId) -> bool {
    if from == to {
        return true;
    }

    let (to_hidden, to_level, from_level) = match (w.mobile(to), w.mobile(from)) {
        (Some(t), Some(f)) => (t.hidden, t.access_level, f.access_level),
        _ => return false,
    };
    if to_hidden && to_level > from_level {
        return false;
    }

    let summoned = |w: &World, m: MobileId| creature::cd(w, m).is_some_and(|c| c.summoned);
    let owner = |w: &World, m: MobileId| {
        if summoned(w, m) {
            creature::control::summon_master(w, m).unwrap_or(m)
        } else {
            m
        }
    };
    let (pm_from, pm_targ) = (owner(w, from), owner(w, to));

    if let Some(opponents) = hooks(w)
        .and_then(|h| h.duel_opponents)
        .and_then(|f| f(w, pm_from, pm_targ))
    {
        return opponents;
    }

    let from_guild = get_guild_for(w, from);
    let to_guild = get_guild_for(w, to);
    if let (Some(a), Some(b)) = (from_guild, to_guild)
        && (a == b
            || hooks(w)
                .and_then(|h| h.guild_is_ally)
                .is_some_and(|f| f(w, a, b)))
    {
        return false;
    }

    let party_has = |w: &World, owner: MobileId, m: MobileId| {
        hooks(w)
            .and_then(|h| h.party_contains)
            .is_some_and(|f| f(w, owner, m))
    };

    if party_has(w, from, to) {
        return false;
    }

    if let Some(targ_master) = creature::control::get_master(w, to)
        && (targ_master == from || party_has(w, from, targ_master))
    {
        return false;
    }

    if let Some(from_master) = creature::control::get_master(w, from)
        && (from_master == to || party_has(w, to, from_master))
    {
        return false;
    }

    let targ_creature = creature::cd(w, to).map(|c| c.controlled);
    let initial_innocent = ctype(w, to).is_some_and(|c| (c.initial_innocent)(w, to));
    (targ_creature == Some(false) && initial_innocent)
        || notoriety::mobile_notoriety(w, from, to) != INNOCENT
        || mb::murderer(w, from)
}

// ---- summons ----------------------------------------------------------------------------------------------------

/// `Summon(creature, caster, sound, duration, scaleDuration, scaleStats)`.
pub fn summon(
    w: &mut World,
    creature_id: MobileId,
    caster: MobileId,
    sound: i32,
    mut duration: TimeSpan,
    scale_duration: bool,
    scale_stats: bool,
) {
    let Some(map) = w.mobile(caster).and_then(|m| m.map) else {
        return;
    };

    let scale = 1.0 + (skills::value(w, caster, SkillName::Magery) - 100.0) / 200.0;

    if scale_duration {
        duration = TimeSpan::from_secs_f64(duration.total_seconds() * scale);
    }

    if scale_stats {
        let (s, d, i) = w
            .mobile(creature_id)
            .map_or((0, 0, 0), |m| (m.str, m.dex, m.int));
        props_b::set_raw_str(w, creature_id, (f64::from(s) * scale) as i32);
        let hp = mb::hits_max(w, creature_id);
        props_b::set_hits(w, creature_id, hp);

        props_b::set_raw_dex(w, creature_id, (f64::from(d) * scale) as i32);
        let sm = mb::stam_max(w, creature_id);
        props_b::set_stam(w, creature_id, sm);

        props_b::set_raw_int(w, creature_id, (f64::from(i) * scale) as i32);
        let mn = mb::mana_max(w, creature_id);
        props_b::set_mana(w, creature_id, mn);
    }

    let mut p = w.mobile(caster).map(|m| m.location).unwrap_or_default();
    if find_valid_spawn_location(w, Some(map), &mut p, true) {
        creature::control::summon(w, creature_id, true, caster, p, sound, duration, None);
        return;
    }

    if let Some(ty) = w.mobile(creature_id).map(|m| m.ty) {
        (ty.delete)(w, creature_id);
    }
    send(w, caster, 501942); // That location is blocked.
}

/// `FindValidSpawnLocation(map, ref p, surroundingsOnly)`.
pub fn find_valid_spawn_location(
    w: &mut World,
    map: Option<MapId>,
    p: &mut Point3D,
    surroundings_only: bool,
) -> bool {
    let Some(map) = map else { return false }; // sanity

    if !surroundings_only {
        if can_spawn(w, map, p.x, p.y, p.z) {
            return true; // p's fine.
        }
        let z = w.maps.get_average_z(map, p.x, p.y);
        if can_spawn(w, map, p.x, p.y, z) {
            *p = Point3D::new(p.x, p.y, z);
            return true;
        }
    }

    let offset = utility::random(&mut w.rng, 8) * 2;

    let mut i = 0;
    while i < OFFSETS.len() {
        let x = p.x + OFFSETS[(offset as usize + i) % OFFSETS.len()];
        let y = p.y + OFFSETS[(offset as usize + i + 1) % OFFSETS.len()];

        if can_spawn(w, map, x, y, p.z) {
            *p = Point3D::new(x, y, p.z);
            return true;
        }

        let z = w.maps.get_average_z(map, x, y);

        if can_spawn(w, map, x, y, z) {
            *p = Point3D::new(x, y, z);
            return true;
        }
        i += 2;
    }

    false
}

fn can_spawn(w: &mut World, map: MapId, x: i32, y: i32, z: i32) -> bool {
    muo_server::map::fit::can_spawn_mobile_at(w, map, x, y, z)
}

// ---- travel -----------------------------------------------------------------------------------------------------

/// `InvalidTravelMessage(type)`: a cliloc number.
pub fn invalid_travel_message(t: TravelCheckType) -> i32 {
    match t {
        TravelCheckType::RecallTo | TravelCheckType::GateTo => 1019004, // You are not allowed to travel there.
        TravelCheckType::RecallFrom | TravelCheckType::TeleportFrom => 502361, // You cannot teleport into that area from here.
        TravelCheckType::TeleportTo => 501035, // You cannot teleport from here to the destination.
        _ => 501802,                           // Thy spell doth not appear to work...
    }
}

/// The validators, in `m_Validators` order.
const VALIDATOR_COUNT: usize = 19;

/// `m_Rules[type, validator]`: true where the travel is allowed despite the validator matching.
const RULES: [[bool; VALIDATOR_COUNT]; 7] = [
    /* Recall From */
    [
        false, false, true, true, false, false, true, false, false, false, false, true, true,
        false, true, false, false, false, false,
    ],
    /* Recall To */
    [false; VALIDATOR_COUNT],
    /* Gate From */
    [false; VALIDATOR_COUNT],
    /* Gate To */
    [false; VALIDATOR_COUNT],
    /* Mark In */
    [false; VALIDATOR_COUNT],
    /* Tele From */
    [
        true, true, true, true, true, true, true, true, false, true, true, true, false, true, true,
        true, true, false, true,
    ],
    /* Tele To */
    [
        true, true, true, true, true, true, true, true, false, true, false, false, false, true,
        true, true, true, false, false,
    ],
];

/// The result of `CheckTravel`: allowed, or refused with the message (`None` is "use the default message", which
/// the caller reads from [`invalid_travel_message`]).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TravelCheck {
    pub allowed: bool,
    pub message: Option<muo_io::text::TextDefinition>,
}

impl TravelCheck {
    fn ok() -> Self {
        Self {
            allowed: true,
            message: None,
        }
    }

    fn refused(t: TravelCheckType) -> Self {
        Self {
            allowed: false,
            message: Some(muo_io::text::TextDefinition::of_number(
                invalid_travel_message(t),
            )),
        }
    }
}

/// `CheckTravel(caster, type, out message)`.
pub fn check_travel(w: &mut World, caster: MobileId, t: TravelCheckType) -> TravelCheck {
    let (map, loc) = w
        .mobile(caster)
        .map_or((None, Point3D::default()), |m| (m.map, m.location));
    check_travel_at(w, Some(caster), map, loc, t)
}

/// `CheckTravel(map, loc, type, out message)`: no caster.
pub fn check_travel_map(
    w: &mut World,
    map: Option<MapId>,
    loc: Point3D,
    t: TravelCheckType,
) -> TravelCheck {
    check_travel_at(w, None, map, loc, t)
}

/// `CheckTravel(caster, map, loc, type, out message)`.
pub fn check_travel_at(
    w: &mut World,
    caster: Option<MobileId>,
    map: Option<MapId>,
    loc: Point3D,
    t: TravelCheckType,
) -> TravelCheck {
    if is_invalid(w, map, loc) {
        // null, internal, out of bounds
        return TravelCheck::refused(t);
    }
    let map = map.expect("a valid location has a map");

    // Always allow monsters to teleport
    if matches!(
        t,
        TravelCheckType::TeleportTo | TravelCheckType::TeleportFrom
    ) && caster.is_some_and(|c| creature::cd(w, c).is_some_and(|d| !d.controlled && !d.summoned))
    {
        return TravelCheck::ok();
    }

    let v = t as usize;

    if let Some(c) = caster {
        let (cmap, cloc) = w
            .mobile(c)
            .map_or((None, Point3D::default()), |m| (m.map, m.location));
        let destination = region::find(w, loc, Some(map));
        let current = region::find(w, cloc, cmap);

        // `Region.Find(..) as BaseRegion`: only BaseRegion classes check travel.
        for r in [destination, current] {
            if let Some(res) = super::super::regions::check_travel(w, r, c, loc, t)
                && !res.allowed
            {
                return TravelCheck {
                    allowed: false,
                    message: Some(res.message.unwrap_or_else(|| {
                        muo_io::text::TextDefinition::of_number(invalid_travel_message(t))
                    })),
                };
            }
        }
    }

    for (i, rule) in RULES[v].iter().enumerate() {
        let is_valid = *rule || !validator(w, i, map, loc, caster, t);
        if !is_valid {
            return TravelCheck::refused(t);
        }
    }

    TravelCheck::ok()
}

fn validator(
    w: &mut World,
    i: usize,
    map: MapId,
    loc: Point3D,
    caster: Option<MobileId>,
    t: TravelCheckType,
) -> bool {
    match i {
        0 => is_felucca_t2a(map, loc),
        1 => is_khaldun(w, map, loc),
        2 => is_ilshenar(map),
        3 => is_trammel_wind(map, loc),
        4 => is_felucca_wind(map, loc),
        5 => is_felucca_dungeon(w, map, loc),
        6 => is_trammel_solen_hive(map, loc),
        7 => is_felucca_solen_hive(map, loc),
        8 => is_crystal_cave(map, loc),
        9 => is_doom_gauntlet(map, loc),
        10 => is_doom_ferry(map, loc),
        11 => is_safe_zone(w, map, loc, caster, t),
        12 => hooks(w)
            .and_then(|h| h.is_faction_stronghold)
            .is_some_and(|f| f(w, map, loc)),
        13 => hooks(w)
            .and_then(|h| h.is_champion_spawn)
            .is_some_and(|f| f(w, map, loc)),
        14 => is_tokuno_dungeon(map, loc),
        15 => is_lamp_room(map, loc),
        16 => is_guardian_room(map, loc),
        17 => is_heartwood(map, loc),
        18 => hooks(w)
            .and_then(|h| h.is_ml_dungeon)
            .is_some_and(|f| f(w, map, loc)),
        _ => false,
    }
}

pub fn is_wind_loc(loc: Point3D) -> bool {
    let (x, y) = (loc.x, loc.y);
    x >= 5120 && y >= 0 && x < 5376 && y < 256
}

pub fn is_felucca_wind(map: MapId, loc: Point3D) -> bool {
    map == MapId::FELUCCA && is_wind_loc(loc)
}

pub fn is_trammel_wind(map: MapId, loc: Point3D) -> bool {
    map == MapId::TRAMMEL && is_wind_loc(loc)
}

pub fn is_ilshenar(map: MapId) -> bool {
    map == MapId::ILSHENAR
}

pub fn is_solen_hive_loc(loc: Point3D) -> bool {
    let (x, y) = (loc.x, loc.y);
    x >= 5640 && y >= 1776 && x < 5935 && y < 2039
}

pub fn is_trammel_solen_hive(map: MapId, loc: Point3D) -> bool {
    map == MapId::TRAMMEL && is_solen_hive_loc(loc)
}

pub fn is_felucca_solen_hive(map: MapId, loc: Point3D) -> bool {
    map == MapId::FELUCCA && is_solen_hive_loc(loc)
}

pub fn is_felucca_t2a(map: MapId, loc: Point3D) -> bool {
    let (x, y) = (loc.x, loc.y);
    map == MapId::FELUCCA && x >= 5120 && y >= 2304 && x < 6144 && y < 4096
}

pub fn is_any_t2a(map: MapId, loc: Point3D) -> bool {
    let (x, y) = (loc.x, loc.y);
    (map == MapId::TRAMMEL || map == MapId::FELUCCA)
        && x >= 5120
        && y >= 2304
        && x < 6144
        && y < 4096
}

pub fn is_felucca_dungeon(w: &World, map: MapId, loc: Point3D) -> bool {
    let r = region::find(w, loc, Some(map));
    super::super::regions::is_part_of_dungeon(w, r)
        && w.region(r).is_some_and(|r| r.map == MapId::FELUCCA)
}

pub fn is_khaldun(w: &World, map: MapId, loc: Point3D) -> bool {
    let r = region::find(w, loc, Some(map));
    w.region(r).and_then(|r| r.name.as_deref()) == Some("Khaldun")
}

pub fn is_crystal_cave(map: MapId, loc: Point3D) -> bool {
    if map != MapId::MALAS || loc.z >= -80 {
        return false;
    }
    let (x, y) = (loc.x, loc.y);
    (x >= 1182 && y >= 437 && x < 1211 && y < 470)
        || (x >= 1156 && y >= 470 && x < 1211 && y < 503)
        || (x >= 1176 && y >= 503 && x < 1208 && y < 509)
        || (x >= 1188 && y >= 509 && x < 1201 && y < 513)
}

pub fn is_safe_zone(
    w: &World,
    map: MapId,
    loc: Point3D,
    caster: Option<MobileId>,
    t: TravelCheckType,
) -> bool {
    matches!(
        t,
        TravelCheckType::TeleportTo | TravelCheckType::TeleportFrom
    ) && hooks(w)
        .and_then(|h| h.safe_zone_blocks_travel)
        .is_some_and(|f| f(w, map, loc, caster))
}

pub fn is_doom_ferry(map: MapId, loc: Point3D) -> bool {
    if map != MapId::MALAS {
        return false;
    }
    let (x, y) = (loc.x, loc.y);
    (x >= 426 && y >= 314 && x <= 430 && y <= 331) || (x >= 406 && y >= 247 && x <= 410 && y <= 264)
}

pub fn is_tokuno_dungeon(map: MapId, loc: Point3D) -> bool {
    // The tokuno dungeons are really inside malas
    if map != MapId::MALAS {
        return false;
    }
    let (x, y) = (loc.x, loc.y);
    let r1 = x >= 0 && y >= 0 && x <= 128 && y <= 128;
    let r2 = x >= 45 && y >= 320 && x < 195 && y < 710;
    r1 || r2
}

pub fn is_doom_gauntlet(map: MapId, loc: Point3D) -> bool {
    if map != MapId::MALAS {
        return false;
    }
    let (x, y) = (loc.x - 256, loc.y - 304);
    x >= 0 && y >= 0 && x < 256 && y < 256
}

pub fn is_lamp_room(map: MapId, loc: Point3D) -> bool {
    if map != MapId::MALAS {
        return false;
    }
    let (x, y) = (loc.x, loc.y);
    x >= 465 && y >= 92 && x < 474 && y < 102
}

pub fn is_guardian_room(map: MapId, loc: Point3D) -> bool {
    if map != MapId::MALAS {
        return false;
    }
    let (x, y) = (loc.x, loc.y);
    x >= 356 && y >= 5 && x < 375 && y < 25
}

pub fn is_heartwood(map: MapId, loc: Point3D) -> bool {
    let (x, y) = (loc.x, loc.y);
    (map == MapId::TRAMMEL || map == MapId::FELUCCA) && x >= 6911 && y >= 254 && x < 7167 && y < 511
}

/// `IsInvalid(map, loc)`: null, internal, out of bounds.
pub fn is_invalid(w: &World, map: Option<MapId>, loc: Point3D) -> bool {
    let Some(map) = map.filter(|&m| m != MapId::INTERNAL) else {
        return true;
    };
    let Some(m) = w.maps.get(map) else {
        return true;
    };
    loc.x < 0 || loc.y < 0 || loc.x >= m.width || loc.y >= m.height
}

/// `IsTown(loc, caster)`.
pub fn is_town(w: &mut World, loc: Point3D, caster: MobileId) -> bool {
    let Some(map) = w.mobile(caster).and_then(|m| m.map) else {
        return false;
    };

    let r = region::find(w, loc, Some(map));
    if super::super::regions::get_safe_zone(w, r).is_some() {
        let pm = w
            .mobile(caster)
            .is_some_and(|m| m.ty.is(&crate::player_mobile::PLAYER_MOBILE));
        if pm
            && !hooks(w)
                .and_then(|h| h.duel_in_progress_for)
                .is_some_and(|f| f(w, caster))
        {
            return true;
        }
    }

    match super::super::regions::get_guarded(w, r) {
        Some(g) => !super::super::regions::guarded::is_disabled(w, g),
        None => false,
    }
}

/// `CheckTown(ip, caster)`.
pub fn check_town(w: &mut World, ip: Point3dRef, caster: MobileId) -> bool {
    let loc = match ip {
        Point3dRef::Entity(EntityId::Item(i)) => item_ops::get_world_location(w, i),
        other => point_of(w, other),
    };
    check_town_at(w, loc, caster)
}

/// `CheckTown(loc, caster)`.
pub fn check_town_at(w: &mut World, loc: Point3D, caster: MobileId) -> bool {
    if is_town(w, loc, caster) {
        send(w, caster, 500946); // You cannot cast this in town!
        return false;
    }
    true
}

// ---- reflection and damage --------------------------------------------------------------------------------------

/// `CheckReflect(circle, caster, ref target)`: true if reflected, `caster` and `target` swapped. `Core.UOR` is
/// `Expansion >= UOR`.
pub fn check_reflect(
    w: &mut World,
    circle: i32,
    caster: &mut MobileId,
    target: &mut MobileId,
) -> bool {
    let mut circle = circle;
    let mut reflect = false;
    let absorb = w.mobile(*target).map_or(0, |m| m.magic_damage_absorb);

    if absorb > 0 {
        if w.core
            .expansion
            .unwrap_or(muo_data::expansion::Expansion::ML)
            < muo_data::expansion::Expansion::UOR
        {
            // T2A: single-use reflection, consumed immediately
            set_magic_absorb(w, *target, 0);
            reflect = true;
        } else {
            circle += 1;

            let remaining = absorb - circle;
            set_magic_absorb(w, *target, remaining);

            // This order isn't very intuitive, but you have to nullify reflect before target gets switched
            reflect = remaining >= 0;
            if remaining <= 0 {
                set_magic_absorb(w, *target, 0);
                defensive_spell_nullify(w, *target);
            }
        }
    }

    if let Some(c) = ctype(w, *target) {
        (c.check_reflect)(w, *target, *caster, &mut reflect);
    }

    if reflect {
        fx::fixed_effect(w, *target, 0x37B9, 10, 5);
        std::mem::swap(caster, target);
    }

    reflect
}

fn set_magic_absorb(w: &mut World, m: MobileId, v: i32) {
    if let Some(m) = w.mobile_mut(m) {
        m.magic_damage_absorb = v;
    }
}

/// `Damage(spell, target, damage)`.
pub fn damage_spell(w: &mut World, sp: SpellId, target: MobileId, damage: f64) {
    let ts = get_damage_delay_for_spell(w, sp);
    let from = spell::caster(w, sp);
    damage_full(w, Some(sp), ts, target, from, damage);
}

/// `Damage(delay, target, damage)` / `Damage(delay, target, from, damage)` / `Damage(spell, delay, target, from, damage)`.
pub fn damage_full(
    w: &mut World,
    sp: Option<SpellId>,
    delay: TimeSpan,
    target: MobileId,
    from: Option<MobileId>,
    damage: f64,
) {
    let mut damage_given = damage as i32;

    if delay == TimeSpan::ZERO {
        if let (Some(f), Some(c)) = (from, from.and_then(|f| ctype(w, f))) {
            (c.alter_spell_damage_to)(w, f, target, &mut damage_given);
        }
        if let Some(c) = ctype(w, target) {
            (c.alter_spell_damage_from)(w, target, from, &mut damage_given);
        }

        if let Some(ty) = w.mobile(target).map(|m| m.ty) {
            (ty.damage)(w, target, damage_given, from, true, false);
        }

        if let Some(f) = from {
            if let Some(c) = ctype(w, f) {
                (c.on_damage_spell)(w, f, target, damage_given);
            }
            if let Some(c) = ctype(w, target) {
                (c.on_harmful_spell)(w, target, f);
                (c.on_damaged_by_spell)(w, target, f, damage_given);
            }
        }
    } else {
        // SpellDamageTimer
        let mut dmg = damage_given;
        let h = timer::create(w, delay, TimeSpan::ZERO, 1, move |w| {
            if let Some(f) = from
                && let Some(c) = ctype(w, f)
            {
                (c.alter_spell_damage_to)(w, f, target, &mut dmg);
            }
            if let Some(c) = ctype(w, target) {
                (c.alter_spell_damage_from)(w, target, from, &mut dmg);
            }
            if let Some(ty) = w.mobile(target).map(|m| m.ty) {
                (ty.damage)(w, target, dmg, None, true, false);
            }
            if let Some(sp) = sp {
                spell::remove_delayed_damage_context(w, sp, target);
            }
        });
        if let Some(sp) = sp
            && spell_type(w, sp).is_some_and(|t| t.delayed_damage_in(core::aos(w)))
        {
            spell::start_delayed_damage_context(w, sp, target, h);
        }
        timer::begin(w, h);
    }
}

/// The elemental split and algorithm of an AOS spell damage.
#[derive(Copy, Clone, Debug, Default)]
pub struct SpellDamageSplit {
    pub phys: i32,
    pub fire: i32,
    pub cold: i32,
    pub pois: i32,
    pub nrgy: i32,
    pub chaos: i32,
    /// `DFAlgorithm` as its integer value (`Standard` is 0).
    pub dfa: i32,
}

/// `Damage(spell, delay, target, from, damage, phys, fire, cold, pois, nrgy, chaos, dfa)`: the AOS form.
pub fn damage_aos(
    w: &mut World,
    sp: Option<SpellId>,
    delay: TimeSpan,
    target: MobileId,
    from: Option<MobileId>,
    damage: f64,
    split: SpellDamageSplit,
) {
    let mut dmg = damage as i32;

    if delay == TimeSpan::ZERO {
        if let Some(f) = from
            && let Some(c) = ctype(w, f)
        {
            (c.alter_spell_damage_to)(w, f, target, &mut dmg);
        }
        if let Some(c) = ctype(w, target) {
            (c.alter_spell_damage_from)(w, target, from, &mut dmg);
        }

        if let Some(red) = hooks(w)
            .and_then(|h| h.feint_damage_reduction)
            .and_then(|f| f(w, from, target))
        {
            // example: 35 damage * 50 / 100 = 17 damage
            dmg -= dmg * red / 100;
        }

        if let Some(f) = hooks(w).and_then(|h| h.set_dfa) {
            f(w, split.dfa);
        }
        let damage_given = aos_helpers::damage(
            w,
            target,
            from,
            dmg,
            DamageSplit {
                phys: split.phys,
                fire: split.fire,
                cold: split.cold,
                pois: split.pois,
                nrgy: split.nrgy,
            },
            DamageOptions {
                chaos: split.chaos,
                ..Default::default()
            },
        );
        if let Some(f) = hooks(w).and_then(|h| h.spell_plague_on_damaged) {
            f(w, target);
        }
        if let Some(f) = hooks(w).and_then(|h| h.set_dfa) {
            f(w, 0);
        }

        if let Some(f) = from {
            if let Some(c) = ctype(w, f) {
                (c.on_damage_spell)(w, f, target, damage_given);
            }
            if let Some(c) = ctype(w, target) {
                (c.on_harmful_spell)(w, target, f);
                (c.on_damaged_by_spell)(w, target, f, damage_given);
            }
        }
    } else {
        // SpellDamageTimerAOS
        let h = timer::create(w, delay, TimeSpan::ZERO, 1, move |w| {
            let dead = w
                .mobile(target)
                .is_none_or(|m| m.deleted || !(m.ty.alive)(w, target));
            if dead {
                return;
            }
            damage_aos(w, sp, TimeSpan::ZERO, target, from, f64::from(dmg), split);
            if let Some(sp) = sp {
                spell::remove_delayed_damage_context(w, sp, target);
            }
        });
        if let Some(sp) = sp
            && spell_type(w, sp).is_some_and(|t| t.delayed_damage_in(core::aos(w)))
        {
            spell::start_delayed_damage_context(w, sp, target, h);
        }
        timer::begin(w, h);
    }
}

/// `Damage(spell, target, damage, phys, fire, cold, pois, nrgy, chaos, dfa)`: the delay of the spell, its caster.
pub fn damage_aos_spell(
    w: &mut World,
    sp: SpellId,
    target: MobileId,
    damage: f64,
    split: SpellDamageSplit,
) {
    let delay = get_damage_delay_for_spell(w, sp);
    let from = spell::caster(w, sp);
    damage_aos(w, Some(sp), delay, target, from, damage, split);
}

/// `DoLeech(damageGiven, from, target)`: the Wraith Form and Vampiric Embrace leeches.
pub fn do_leech(w: &mut World, damage_given: i32, from: MobileId, target: MobileId) {
    let Some(ctx_type) = transformation_type(w, from) else {
        return;
    };
    if ctx_type == "Server.Spells.Necromancy.WraithFormSpell" {
        // WraithFormSpell.DoWraithLeech (muo_spells sets the hook)
        if let Some(f) = w
            .resources
            .get::<TransformationHooks>()
            .and_then(|h| h.wraith_leech)
        {
            f(w, from, target, damage_given);
        }
    } else if ctx_type == "Server.Spells.Necromancy.VampiricEmbraceSpell" {
        let heal = w
            .mobile(target)
            .map_or(0, |m| m.hits)
            .min(aos_helpers::scale(damage_given, 20));
        let hits = w.mobile(from).map_or(0, |m| m.hits);
        props_b::set_hits(w, from, hits + heal);
        fx::play_sound(w, from, 0x44D);
    }
}

/// `Heal(amount, target, from, message = true)`.
pub fn heal(w: &mut World, amount: i32, target: MobileId, from: Option<MobileId>, message: bool) {
    // TODO: All Healing *spells* go through ArcaneEmpowerment
    muo_server::mobile::part2::damage::heal_from(w, target, amount, from, message);
}

// ---- TransformationSpellHelper ----------------------------------------------------------------------------------

/// `ITransformationSpell`: what a transformation spell class provides.
pub struct TransformationInfo {
    pub body: i32,
    pub hue: i32,
    pub phys_resist_offset: i32,
    pub fire_resist_offset: i32,
    pub cold_resist_offset: i32,
    pub pois_resist_offset: i32,
    pub nrgy_resist_offset: i32,
    pub tick_rate: f64,
    pub on_tick: fn(&mut World, MobileId),
    pub do_effect: fn(&mut World, MobileId),
    pub remove_effect: fn(&mut World, MobileId),
    /// `spell.CastSkill` and the minimum of `spell.GetCastSkills` for `m` (`AosSkillBonuses.CheckCancelMorph`).
    pub cast_skill: muo_server::skills::SkillName,
    pub min_skill: fn(&mut World, MobileId) -> f64,
}

/// `TransformContext`.
#[derive(Clone, Copy)]
pub struct TransformContext {
    pub timer: TimerHandle,
    /// The spell class: its full name.
    pub ty: &'static str,
    pub spell: &'static TransformationInfo,
}

/// `TransformationSpellHelper._table`, and the slots other systems own.
#[derive(Default)]
pub struct TransformationHooks {
    pub table: IndexMap<MobileId, TransformContext>,
    /// `WraithFormSpell.DoWraithLeech(from, target, damage)`.
    pub wraith_leech: Option<fn(&mut World, MobileId, MobileId, i32)>,
}

fn table(w: &mut World) -> &mut IndexMap<MobileId, TransformContext> {
    &mut w.resources.get_mut::<TransformationHooks>().table
}

/// `GetContext(m)`.
pub fn get_context(w: &World, m: MobileId) -> Option<TransformContext> {
    w.resources
        .get::<TransformationHooks>()?
        .table
        .get(&m)
        .copied()
}

fn transformation_type(w: &World, m: MobileId) -> Option<&'static str> {
    get_context(w, m).map(|c| c.ty)
}

/// `UnderTransformation(m)`.
pub fn under_transformation(w: &World, m: MobileId) -> bool {
    get_context(w, m).is_some()
}

/// `UnderTransformation(m, type)`.
pub fn under_transformation_of(w: &World, m: MobileId, type_name: &str) -> bool {
    get_context(w, m).is_some_and(|c| c.ty == type_name)
}

/// `AddContext(m, context)`.
pub fn add_context(w: &mut World, m: MobileId, context: TransformContext) {
    table(w).insert(m, context);
}

/// `RemoveContext(m, resetGraphics)`.
pub fn remove_context(w: &mut World, m: MobileId, reset_graphics: bool) {
    if let Some(c) = get_context(w, m) {
        remove_context_of(w, m, c, reset_graphics);
    }
}

/// `RemoveContext(m, context, resetGraphics)`.
pub fn remove_context_of(
    w: &mut World,
    m: MobileId,
    context: TransformContext,
    reset_graphics: bool,
) {
    if table(w).shift_remove(&m).is_none() {
        return;
    }

    mb::remove_resistance_mod_by_name(w, m, "TransformSpell");

    if reset_graphics {
        props_b::set_hue_mod(w, m, -1);
        props_b::set_body_mod(w, m, muo_primitives::Body(0));
    }

    let mut t = context.timer;
    t.cancel(w);
    (context.spell.remove_effect)(w, m);
}

/// `CheckCast(caster, spell)`.
pub fn transformation_check_cast(w: &mut World, caster: MobileId) -> bool {
    if hooks(w)
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, caster))
    {
        send(w, caster, 1061632); // You can't do that while carrying the sigil.
        return false;
    }
    if !combat::can_begin_action(w, caster, "Server.Spells.Seventh.PolymorphSpell") {
        send(w, caster, 1061628); // You can't do that while polymorphed.
        return false;
    }
    if hooks(w)
        .and_then(|h| h.animal_form_under)
        .is_some_and(|f| f(w, caster))
    {
        send(w, caster, 1061091); // You cannot cast that spell in this form.
        return false;
    }
    true
}

/// `OnCast(caster, spell)`: the whole cast of a transformation spell whose class is `ty`.
pub fn transformation_on_cast(
    w: &mut World,
    caster: MobileId,
    sp: SpellId,
    info: &'static TransformationInfo,
) -> bool {
    let Some(ty) = spell_type(w, sp) else {
        return false;
    };

    if hooks(w)
        .and_then(|h| h.sigil_exists_on)
        .is_some_and(|f| f(w, caster))
    {
        send(w, caster, 1061632); // You can't do that while carrying the sigil.
    } else if !combat::can_begin_action(w, caster, "Server.Spells.Seventh.PolymorphSpell") {
        send(w, caster, 1061628); // You can't do that while polymorphed.
    } else if hooks(w)
        .and_then(|h| h.is_disguised)
        .is_some_and(|f| f(w, caster))
    {
        send(w, caster, 1061631); // You can't do that while disguised.
        return false;
    } else if hooks(w)
        .and_then(|h| h.animal_form_under)
        .is_some_and(|f| f(w, caster))
    {
        send(w, caster, 1061091); // You cannot cast that spell in this form.
    } else if !combat::can_begin_action(w, caster, "Server.Spells.Fifth.IncognitoSpell")
        || (w.mobile(caster).is_some_and(|m| m.body_mod.body_id() != 0)
            && get_context(w, caster).is_none())
    {
        (ty.do_fizzle)(w, sp);
    } else if spell::check_sequence(w, sp) {
        let context = get_context(w, caster);
        let our_type = ty.full_name;

        let was_transformed = context.is_some();
        let our_transform = context.is_some_and(|c| c.ty == our_type);

        if let Some(c) = context {
            remove_context_of(w, caster, c, our_transform);

            if our_transform {
                fx::play_sound(w, caster, 0xFA);
                fx::fixed_particles(
                    w,
                    caster,
                    0x3728,
                    1,
                    13,
                    5042,
                    muo_server::effects::EffectLayer::Waist,
                );
            }
        }
        let _ = was_transformed;

        if !our_transform {
            use muo_server::mobile::ResistanceType;
            for (t, offset) in [
                (ResistanceType::Physical, info.phys_resist_offset),
                (ResistanceType::Fire, info.fire_resist_offset),
                (ResistanceType::Cold, info.cold_resist_offset),
                (ResistanceType::Poison, info.pois_resist_offset),
                (ResistanceType::Energy, info.nrgy_resist_offset),
            ] {
                if offset != 0 {
                    let m = muo_server::mods::ResistanceMod::new(
                        w,
                        t,
                        Some("TransformSpell"),
                        offset,
                        None,
                    );
                    mb::add_resistance_mod(w, caster, m);
                }
            }

            let human = w
                .resources
                .get::<muo_primitives::BodyTable>()
                .is_some_and(|t| t.is_human(muo_primitives::Body(info.body)));
            if !human && let Some(mount) = props_b::mount(w, caster) {
                muo_server::mobile::part2::deps::mount_clear_rider(w, mount);
            }

            props_b::set_body_mod(w, caster, muo_primitives::Body(info.body));
            props_b::set_hue_mod(w, caster, info.hue);

            (info.do_effect)(w, caster);

            let rate = TimeSpan::from_secs_f64(info.tick_rate);
            let h_cell = std::rc::Rc::new(std::cell::Cell::new(TimerHandle::default()));
            let h = timer::create(w, rate, rate, 0, {
                let h_cell = h_cell.clone();
                move |w| transform_tick(w, caster, info, h_cell.get())
            });
            h_cell.set(h);
            timer::begin(w, h);

            add_context(
                w,
                caster,
                TransformContext {
                    timer: h,
                    ty: our_type,
                    spell: info,
                },
            );
            return true;
        }
    }

    false
}

/// `TransformTimer.OnTick`.
fn transform_tick(w: &mut World, m: MobileId, info: &'static TransformationInfo, h: TimerHandle) {
    let gone = w.mobile(m).is_none_or(|mob| {
        mob.deleted
            || !(mob.ty.alive)(w, m)
            || props_b::body(w, m).body_id() != info.body
            || mb::hue(w, m) != info.hue
    });
    if gone {
        remove_context(w, m, true);
        timer::stop(w, h); // `Stop()`: the context may already be gone
    } else {
        (info.on_tick)(w, m);
    }
}

/// Dead mobile cleanup is not a rectangle, but the module keeps `Rectangle2D` for the callers of `Turn` using
/// areas.
pub fn rect_contains(r: Rectangle2D, p: Point3D) -> bool {
    r.contains(muo_primitives::Point2D::new(p.x, p.y))
}

/// `item is T`: the item's class, or an ancestor, has this full name.
fn item_is(w: &World, id: ItemId, full_name: &str) -> bool {
    let mut t = w.item(id).map(|i| i.ty);
    while let Some(ty) = t {
        if ty.full_name == full_name {
            return true;
        }
        t = ty.parent;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_table_shape() {
        assert_eq!(RULES.len(), 7);
        // Recall From is allowed out of Ilshenar and the Trammel wind, not out of the Felucca T2A land.
        assert!(RULES[TravelCheckType::RecallFrom as usize][2]);
        assert!(!RULES[TravelCheckType::RecallFrom as usize][0]);
        // Marking is never allowed in a validated region.
        assert!(RULES[TravelCheckType::Mark as usize].iter().all(|&b| !b));
        // Teleporting out of the Malas crystal cave is not allowed, into the stronghold neither.
        assert!(!RULES[TravelCheckType::TeleportFrom as usize][8]);
        assert!(!RULES[TravelCheckType::TeleportTo as usize][12]);
    }

    #[test]
    fn region_geometry() {
        let p = |x, y, z| Point3D::new(x, y, z);
        assert!(is_felucca_t2a(MapId::FELUCCA, p(5120, 2304, 0)));
        assert!(!is_felucca_t2a(MapId::TRAMMEL, p(5120, 2304, 0)));
        assert!(is_any_t2a(MapId::TRAMMEL, p(6143, 4095, 0)));
        assert!(!is_any_t2a(MapId::TRAMMEL, p(6144, 4095, 0)));
        assert!(is_wind_loc(p(5375, 255, 0)) && !is_wind_loc(p(5376, 255, 0)));
        assert!(is_crystal_cave(MapId::MALAS, p(1190, 440, -81)));
        assert!(!is_crystal_cave(MapId::MALAS, p(1190, 440, -80)));
        assert!(
            is_doom_gauntlet(MapId::MALAS, p(256, 304, 0))
                && !is_doom_gauntlet(MapId::MALAS, p(512, 304, 0))
        );
        assert!(is_doom_ferry(MapId::MALAS, p(428, 320, 0)));
        assert!(
            is_tokuno_dungeon(MapId::MALAS, p(100, 100, 0))
                && !is_tokuno_dungeon(MapId::TRAMMEL, p(100, 100, 0))
        );
        assert!(is_heartwood(MapId::FELUCCA, p(6911, 254, 0)));
    }

    #[test]
    fn east_to_west_matches_the_quadrants() {
        let o = Point3D::new(100, 100, 0);
        // Directly north-west of the target: dx=-1, dy=-1 -> rx=0, ry=-88: east to west.
        assert!(get_east_to_west(Point3D::new(99, 99, 0), o));
        assert!(!get_east_to_west(Point3D::new(101, 99, 0), o));
    }

    #[test]
    fn travel_messages() {
        assert_eq!(invalid_travel_message(TravelCheckType::RecallTo), 1019004);
        assert_eq!(invalid_travel_message(TravelCheckType::GateTo), 1019004);
        assert_eq!(invalid_travel_message(TravelCheckType::RecallFrom), 502361);
        assert_eq!(invalid_travel_message(TravelCheckType::TeleportTo), 501035);
        assert_eq!(invalid_travel_message(TravelCheckType::Mark), 501802);
    }
}
