//! Notoriety (`NotorietyHandlers`).
//!
//! The handlers read BaseCreature, guild, faction, duel, house and region state that other modules own; each is a
//! `fn` in [`NotorietyHooks`] that the owning crate sets, and an unset one answers as when the system
//! has nothing to say (no creature, no guild, no faction, no duel, no house).
//!
//! `CorpseNotoriety` is `items::corpse::corpse_notoriety`, next to `Corpse`.

use muo_primitives::{Point3D, Serial};
use muo_server::World;
use muo_server::guild::GuildId;
use muo_server::ids::MobileId;
use muo_server::map::{MapId, MapRules};
use muo_server::mobile::AccessLevel;
use muo_server::notoriety::{
    self as noto, ALLY, CAN_BE_ATTACKED, CRIMINAL, ENEMY, INNOCENT, INVULNERABLE, MURDERER,
    Notoriety,
};

use crate::core;
use crate::player_mobile;

/// What `MobileNotoriety` asks about a `BaseCreature` (all false/None for a mobile that is not one).
#[derive(Copy, Clone, Debug, Default)]
pub struct CreatureFacts {
    pub is_creature: bool,
    pub controlled: bool,
    pub control_master: Option<MobileId>,
    /// `ControlOrder == OrderType.Guard`.
    pub guarding: bool,
    pub control_target: Option<MobileId>,
    pub summoned: bool,
    pub summon_master: Option<MobileId>,
    pub always_murderer: bool,
    pub is_invulnerable: bool,
    pub always_attackable: bool,
    pub initial_innocent: bool,
    pub is_animated_dead: bool,
    /// `GetMaster()`: the control master, else the summon master.
    pub master: Option<MobileId>,
    /// `target is BaseFamiliar or ArcaneFey or Golem`: the summoned monsters that stay blue.
    pub exempt_from_summoned_red: bool,
    /// `target is PlayerVendor or TownCrier`.
    pub is_vendor_or_crier: bool,
}

/// A guild-to-guild relation as `Guild.IsAlly` / `IsEnemy` answer.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GuildRelation {
    None,
    Same,
    Ally,
    Enemy,
}

/// The state a duel puts on a pair of players.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DuelNoto {
    Ally,
    Enemy,
}

#[derive(Default)]
#[allow(clippy::type_complexity)]
pub struct NotorietyHooks {
    pub creature: Option<fn(&World, MobileId) -> CreatureFacts>,
    /// `GetGuildFor(m.Guild as Guild, m)`.
    pub guild_for: Option<fn(&mut World, MobileId) -> Option<GuildId>>,
    pub guild_relation: Option<fn(&World, GuildId, GuildId) -> GuildRelation>,
    /// `((Guild)g).Enemies.Count == 0 && g.Type == GuildType.Regular`.
    pub guild_peaceful: Option<fn(&World, GuildId) -> bool>,
    /// `Faction.Find(m, true, true)` as a comparable id.
    pub faction: Option<fn(&World, MobileId) -> Option<u32>>,
    /// `Faction.Find(m, true)`: the same without creature allegiances (the harmful check).
    pub faction_inherit: Option<fn(&World, MobileId) -> Option<u32>>,
    /// `Faction.Facet`.
    pub faction_facet: Option<MapId>,
    /// `pmFrom.DuelContext?.StartedBeginCountdown == true && !Finished && same context`: the pair's standing.
    pub duel_standing: Option<fn(&World, MobileId, MobileId) -> Option<DuelNoto>>,
    /// The whole `DuelContext` block of Mobile_AllowBeneficial/AllowHarmful: `Some(answer)` ends the check.
    pub duel_allow: Option<fn(&World, MobileId, MobileId, bool) -> Option<bool>>,
    /// `pmFrom?.DuelContext?.Started == true || pmTarg?.DuelContext?.Started == true`.
    pub in_started_duel: Option<fn(&World, MobileId) -> bool>,
    /// `BaseHouse.FindHouseAt(p, map, 16)` rules of `CheckHouseFlag(from, m, p, map)`.
    pub check_house_flag:
        Option<fn(&World, MobileId, Option<MobileId>, Point3D, Option<MapId>) -> bool>,
    /// `Region.IsPartOf<SafeZone>()`.
    pub in_safe_zone: Option<fn(&World, MobileId) -> bool>,
    /// `Stealing.ClassicMode`.
    pub stealing_classic_mode: bool,
    /// `target.CanBeginAction<PolymorphSpell>()`.
    pub can_begin_polymorph: Option<fn(&World, MobileId) -> bool>,
    /// `(target as PlayerMobile)?.CheckYoungProtection(from)`.
    pub check_young_protection: Option<fn(&World, MobileId, MobileId) -> bool>,
}

fn hooks(w: &World) -> Option<&NotorietyHooks> {
    w.resources.get::<NotorietyHooks>()
}

fn facts(w: &World, m: MobileId) -> CreatureFacts {
    hooks(w)
        .and_then(|h| h.creature)
        .map_or_else(CreatureFacts::default, |f| f(w, m))
}

/// What `GetGuildFor(def, m)` answers and writes: a controlled creature takes its master's guild.
pub struct GuildFor {
    pub guild: Option<GuildId>,
    /// `c.DisplayGuildTitle = false`.
    pub clear_title: bool,
    /// `c.Guild = ..`.
    pub set_guild: Option<Option<GuildId>>,
}

/// `GetGuildFor(m.Guild as Guild, m)` without its writes (see [`GuildFor`]).
pub fn resolve_guild_for(w: &World, m: MobileId) -> GuildFor {
    use crate::creature::enums::OrderType;
    let own = w.mobile(m).and_then(|x| x.guild);
    let plain = GuildFor {
        guild: own,
        clear_title: false,
        set_guild: None,
    };
    let Some(master) = crate::creature::control::control_master(w, m) else {
        return plain;
    };
    let Some(map) = w.mobile(m).map(|x| x.map) else {
        return plain;
    };
    let Some(order) = crate::creature::data::cd(w, m).map(|d| d.control_order) else {
        return plain;
    };
    let on_map = map.is_some_and(|x| x != muo_server::map::MapId::INTERNAL);
    let master_guild = w.mobile(master).and_then(|x| x.guild);
    let done = |g| GuildFor {
        guild: g,
        clear_title: true,
        set_guild: Some(g),
    };
    if on_map
        && (core::aos(w)
            || super::guild::new_guild_system(w)
            || matches!(order, OrderType::Attack | OrderType::Guard))
    {
        return done(master_guild);
    }
    if !on_map || master_guild.is_none() {
        return done(None);
    }
    GuildFor {
        guild: own,
        clear_title: true,
        set_guild: None,
    }
}

/// `GetGuildFor` for a caller that only reads (`&World`): the writes to the pet are left out.
pub fn guild_for_read(w: &World, m: MobileId) -> Option<GuildId> {
    match hooks(w).and_then(|h| h.guild_for) {
        Some(_) => resolve_guild_for(w, m).guild,
        None => w.mobile(m).and_then(|x| x.guild),
    }
}

pub(crate) fn guild_for(w: &mut World, m: MobileId) -> Option<GuildId> {
    match hooks(w).and_then(|h| h.guild_for) {
        Some(f) => f(w, m),
        None => w.mobile(m).and_then(|x| x.guild),
    }
}

pub(crate) fn relation(w: &World, a: GuildId, b: GuildId) -> GuildRelation {
    if a == b {
        return GuildRelation::Same;
    }
    hooks(w)
        .and_then(|h| h.guild_relation)
        .map_or(GuildRelation::None, |f| f(w, a, b))
}

pub(crate) fn faction(w: &World, m: MobileId) -> Option<u32> {
    hooks(w).and_then(|h| h.faction).and_then(|f| f(w, m))
}

pub(crate) fn faction_inherit(w: &World, m: MobileId) -> Option<u32> {
    hooks(w)
        .and_then(|h| h.faction_inherit)
        .and_then(|f| f(w, m))
}

pub(crate) fn facet(w: &World) -> Option<MapId> {
    hooks(w).and_then(|h| h.faction_facet)
}

fn rules(w: &World, map: Option<MapId>) -> Option<MapRules> {
    map.and_then(|id| w.maps.get(id)).map(|m| m.rules)
}

/// `CheckAggressor(list, target)`.
pub fn check_aggressor(w: &World, owner: MobileId, target: MobileId) -> bool {
    w.mobile(owner)
        .is_some_and(|m| m.aggressors.iter().any(|i| i.attacker == target))
}

/// `CheckAggressed(list, target)`.
pub fn check_aggressed(w: &World, owner: MobileId, target: MobileId) -> bool {
    w.mobile(owner).is_some_and(|m| {
        m.aggressed
            .iter()
            .any(|i| !i.criminal_aggression && i.defender == target)
    })
}

/// `IsPet(c)`.
pub fn is_pet(c: &CreatureFacts) -> bool {
    c.is_creature && c.controlled
}

/// `IsSummoned(c)`.
pub fn is_summoned(c: &CreatureFacts) -> bool {
    c.is_creature && c.summoned
}

/// `CheckHouseFlag(from, m, p, map)`.
pub fn check_house_flag(
    w: &World,
    from: MobileId,
    m: Option<MobileId>,
    p: Point3D,
    map: Option<MapId>,
) -> bool {
    hooks(w)
        .and_then(|h| h.check_house_flag)
        .is_some_and(|f| f(w, from, m, p, map))
}

fn is_human_or_ghost(w: &World, m: MobileId) -> (bool, bool) {
    let body = muo_server::mobile::part1::props_b::body(w, m);
    w.resources
        .get::<muo_primitives::BodyTable>()
        .map_or((false, false), |t| {
            (
                t.is_human(body),
                matches!(body.0, 402 | 403 | 607 | 608 | 694 | 695 | 970),
            )
        })
}

fn is_monster(w: &World, m: MobileId) -> bool {
    let body = muo_server::mobile::part1::props_b::body(w, m);
    w.resources
        .get::<muo_primitives::BodyTable>()
        .is_some_and(|t| t.is_monster(body))
}

/// `NotorietyHandlers.MobileNotoriety(source, target)`.
pub fn mobile_notoriety(w: &mut World, source: MobileId, target: MobileId) -> i32 {
    let (Some(src), Some(tgt)) = (w.mobile(source), w.mobile(target)) else {
        return CAN_BE_ATTACKED;
    };
    let bc_targ = facts(w, target);
    if core::aos(w)
        && (tgt.blessed
            || (bc_targ.is_creature && bc_targ.is_invulnerable)
            || bc_targ.is_vendor_or_crier)
    {
        return INVULNERABLE;
    }
    // Moved above AccessLevel check so staff summons are red
    if bc_targ.is_creature && bc_targ.always_murderer {
        return MURDERER;
    }

    let pm_from = player_mobile::data(w, source).is_some();
    let pm_targ = player_mobile::data(w, target).is_some();
    if pm_from && pm_targ {
        match hooks(w)
            .and_then(|h| h.duel_standing)
            .and_then(|f| f(w, source, target))
        {
            Some(DuelNoto::Ally) => return ALLY,
            Some(DuelNoto::Enemy) => return ENEMY,
            None => {}
        }
    }

    if tgt.access_level > AccessLevel::Player {
        return CAN_BE_ATTACKED;
    }

    if src.player && !tgt.player && pm_from && bc_targ.is_creature {
        let master_level = bc_targ
            .master
            .and_then(|m| w.mobile(m))
            .map(|m| m.access_level);
        if master_level.is_some_and(|l| l > AccessLevel::Player) {
            return CAN_BE_ATTACKED;
        }
        let master = if bc_targ.controlled {
            bc_targ.control_master
        } else {
            None
        };
        if core::ml(w)
            && let Some(master) = master
        {
            if (source == master && check_aggressor(w, target, source))
                || check_aggressor(w, source, target)
            {
                return CAN_BE_ATTACKED;
            }
            return mobile_notoriety(w, source, master);
        }
        if !bc_targ.summoned && !bc_targ.controlled {
            let enemy_type = player_mobile::data(w, source).and_then(|d| d.enemy_of_one_type);
            if enemy_type.is_some_and(|t| std::ptr::eq(t, tgt.ty)) {
                return ENEMY;
            }
        }
    }

    if (tgt.ty.murderer)(w, target)
        || (is_monster(w, target) && is_summoned(&bc_targ) && !bc_targ.exempt_from_summoned_red)
        || (bc_targ.is_creature && bc_targ.is_animated_dead)
    {
        return MURDERER;
    }
    if (tgt.ty.criminal)(w, target) {
        return CRIMINAL;
    }

    let (src_map, tgt_location, tgt_map) = (src.map, tgt.location, tgt.map);
    let source_guild = guild_for(w, source);
    let target_guild = guild_for(w, target);
    if let (Some(a), Some(b)) = (source_guild, target_guild) {
        match relation(w, a, b) {
            GuildRelation::Same | GuildRelation::Ally => return ALLY,
            GuildRelation::Enemy => return ENEMY,
            GuildRelation::None => {}
        }
    }

    let (src_faction, trg_faction) = (faction(w, source), faction(w, target));
    if let (Some(a), Some(b)) = (src_faction, trg_faction)
        && a != b
        && src_map == facet(w)
    {
        return ENEMY;
    }

    let hooks_ = hooks(w);
    let classic_perma = crate::systems::skills::stealing::classic_mode(w)
        && player_mobile::data(w, target).is_some_and(|d| d.perma_flags.contains(&source));
    if classic_perma
        || (bc_targ.is_creature && bc_targ.always_attackable)
        || check_house_flag(w, source, Some(target), tgt_location, tgt_map)
    {
        return CAN_BE_ATTACKED;
    }

    if !(bc_targ.is_creature && bc_targ.initial_innocent) {
        let (human, ghost) = is_human_or_ghost(w, target);
        let polymorphed = !core::ml(w)
            && !hooks_
                .and_then(|h| h.can_begin_polymorph)
                .is_none_or(|f| f(w, target));
        if (!human && !ghost && !is_pet(&bc_targ) && !pm_targ) || polymorphed {
            return CAN_BE_ATTACKED;
        }
    }

    if check_aggressor(w, source, target) || check_aggressed(w, source, target) {
        return CAN_BE_ATTACKED;
    }

    if bc_targ.is_creature
        && bc_targ.controlled
        && bc_targ.guarding
        && bc_targ.control_target == Some(source)
    {
        return CAN_BE_ATTACKED;
    }

    let bc_src = facts(w, source);
    if bc_src.is_creature
        && let Some(master) = bc_src.master
        && (check_aggressor(w, master, target)
            || mobile_notoriety(w, master, target) == CAN_BE_ATTACKED
            || bc_targ.is_creature)
    {
        return CAN_BE_ATTACKED;
    }

    INNOCENT
}

/// `GuildStatus`.
#[derive(Copy, Clone, Eq, PartialEq)]
enum GuildStatus {
    None,
    Peaceful,
    Waring,
}

fn guild_status(w: &World, m: MobileId) -> GuildStatus {
    let Some(g) = w.mobile(m).and_then(|x| x.guild) else {
        return GuildStatus::None;
    };
    if hooks(w)
        .and_then(|h| h.guild_peaceful)
        .is_some_and(|f| f(w, g))
    {
        GuildStatus::Peaceful
    } else {
        GuildStatus::Waring
    }
}

/// The player a mobile acts for: `(bc?.GetMaster() ?? m) as PlayerMobile`.
fn acting_player(w: &World, m: MobileId) -> Option<MobileId> {
    let c = facts(w, m);
    let who = if c.is_creature {
        c.master.unwrap_or(m)
    } else {
        m
    };
    player_mobile::data(w, who).is_some().then_some(who)
}

fn in_safe_zone(w: &World, m: MobileId) -> bool {
    hooks(w)
        .and_then(|h| h.in_safe_zone)
        .is_some_and(|f| f(w, m))
}

/// `NotorietyHandlers.Mobile_AllowBeneficial`.
pub fn allow_beneficial(w: &mut World, from: MobileId, target: MobileId) -> bool {
    let (Some(f), Some(t)) = (w.mobile(from), w.mobile(target)) else {
        return true;
    };
    if f.access_level > AccessLevel::Player || t.access_level > AccessLevel::Player {
        return true;
    }
    if in_safe_zone(w, from) || in_safe_zone(w, target) {
        return false;
    }
    let bc_targ = facts(w, target);
    let pm_from = acting_player(w, from);
    let pm_targ = acting_player(w, target);

    if let (Some(a), Some(b)) = (pm_from, pm_targ)
        && let Some(answer) = hooks(w)
            .and_then(|h| h.duel_allow)
            .and_then(|d| d(w, a, b, true))
    {
        return answer;
    }
    let started = |m: Option<MobileId>| {
        m.is_some_and(|m| {
            hooks(w)
                .and_then(|h| h.in_started_duel)
                .is_some_and(|d| d(w, m))
        })
    };
    if started(pm_from) || started(pm_targ) {
        return false;
    }

    let map = f.map;
    let target_faction = faction_inherit(w, target);
    if (!core::ml(w) || map == facet(w))
        && target_faction.is_some()
        && faction_inherit(w, from) != target_faction
    {
        return false;
    }
    if rules(w, map).is_some_and(|r| !r.contains(MapRules::BENEFICIAL_RESTRICTIONS)) {
        return true; // In felucca, anything goes
    }
    let from_staff = pm_from
        .and_then(|p| w.mobile(p))
        .is_some_and(|p| p.access_level != AccessLevel::Player);
    if !f.player && (pm_from.is_none() || from_staff) {
        return true; // NPCs have no restrictions
    }
    if bc_targ.is_creature && !bc_targ.controlled {
        return false; // Players cannot heal uncontrolled mobiles
    }
    if let Some(p) = pm_from
        && player_mobile::young(w, p)
        && pm_targ.is_some_and(|t| !player_mobile::young(w, t))
    // `pmTarg?.Young == false`: null is not false
    {
        return false; // Young players cannot perform beneficial actions towards non-young players or pets
    }
    let (fg, tg) = (guild_for(w, from), guild_for(w, target));
    if let (Some(a), Some(b)) = (fg, tg)
        && matches!(relation(w, a, b), GuildRelation::Same | GuildRelation::Ally)
    {
        return true; // Guild members can be beneficial
    }
    let (a, b) = (guild_status(w, from), guild_status(w, target));
    a != GuildStatus::Waring && b != GuildStatus::Waring
}

/// `NotorietyHandlers.Mobile_AllowHarmful`.
pub fn allow_harmful(w: &mut World, from: MobileId, target: MobileId) -> bool {
    let (Some(f), Some(t)) = (w.mobile(from), w.mobile(target)) else {
        return true;
    };
    if f.access_level > AccessLevel::Player || t.access_level > AccessLevel::Player {
        return true;
    }
    if in_safe_zone(w, from) || in_safe_zone(w, target) {
        return false;
    }
    let bc_targ = facts(w, target);
    let pm_from = acting_player(w, from);
    let pm_targ = acting_player(w, target);

    if let (Some(a), Some(b)) = (pm_from, pm_targ)
        && let Some(answer) = hooks(w)
            .and_then(|h| h.duel_allow)
            .and_then(|d| d(w, a, b, false))
    {
        return answer;
    }
    let started = |m: Option<MobileId>| {
        m.is_some_and(|m| {
            hooks(w)
                .and_then(|h| h.in_started_duel)
                .is_some_and(|d| d(w, m))
        })
    };
    if started(pm_from) || started(pm_targ) {
        return false;
    }

    if rules(w, f.map).is_some_and(|r| !r.contains(MapRules::HARMFUL_RESTRICTIONS)) {
        return true; // In felucca, anything goes
    }
    let from_staff = pm_from
        .and_then(|p| w.mobile(p))
        .is_some_and(|p| p.access_level != AccessLevel::Player);
    if !f.player && (pm_from.is_none() || from_staff) {
        // Uncontrolled NPCs are only restricted by the young system
        let young_protected = hooks(w)
            .and_then(|h| h.check_young_protection)
            .is_some_and(|c| c(w, target, from));
        return check_aggressor(w, from, target)
            || check_aggressed(w, from, target)
            || !young_protected;
    }

    let from_serial = f.serial;
    let (fg, tg) = (guild_for(w, from), guild_for(w, target));
    if let (Some(a), Some(b)) = (fg, tg)
        && relation(w, a, b) != GuildRelation::None
    {
        return true; // Guild allies or enemies can be harmful
    }
    let Some(t) = w.mobile(target) else {
        return true;
    };
    let summoner_is_player = bc_targ
        .summon_master
        .and_then(|s| w.mobile(s))
        .is_some_and(|s| s.player && Some(s.serial) != Some(from_serial));
    if (bc_targ.is_creature && bc_targ.controlled) || (bc_targ.is_creature && summoner_is_player) {
        return false; // Cannot harm other controlled mobiles from players
    }
    if t.player {
        return false; // Cannot harm other players
    }
    (bc_targ.is_creature && bc_targ.initial_innocent) || noto::compute(w, from, target) != INNOCENT
}

/// `NotorietyHandlers.Initialize`: hues, the handler and the two `Mobile` allow-handlers.
pub fn initialize(w: &mut World) {
    let n = w.resources.get_mut::<Notoriety>();
    n.hues.resize(8, 0);
    n.hues[INNOCENT as usize] = 0x59;
    n.hues[ALLY as usize] = 0x3F;
    n.hues[CAN_BE_ATTACKED as usize] = 0x3B2;
    n.hues[CRIMINAL as usize] = 0x3B2;
    n.hues[ENEMY as usize] = 0x90;
    n.hues[MURDERER as usize] = 0x22;
    n.hues[INVULNERABLE as usize] = 0x35;
    n.handler = Some(mobile_notoriety);
    w.maps.settings.allow_beneficial_handler = Some(allow_beneficial);
    w.maps.settings.allow_harmful_handler = Some(allow_harmful);
}

#[allow(dead_code)]
fn _serial(_: Serial) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_mobiles_follow_the_checks_in_order() {
        let mut w = World::new();
        let a = player_mobile::new_player_mobile(&mut w);
        let b = player_mobile::new_player_mobile(&mut w);
        w.mobile_mut(a).unwrap().player = true;
        w.mobile_mut(b).unwrap().player = true;
        // two players, no body table: nothing marks b, so it is innocent
        assert_eq!(mobile_notoriety(&mut w, a, b), INNOCENT);
        w.mobile_mut(b).unwrap().criminal = true;
        assert_eq!(mobile_notoriety(&mut w, a, b), CRIMINAL);
        w.mobile_mut(b).unwrap().access_level = AccessLevel::GameMaster;
        assert_eq!(mobile_notoriety(&mut w, a, b), CAN_BE_ATTACKED);
        w.mobile_mut(b).unwrap().access_level = AccessLevel::Player;
        w.mobile_mut(b).unwrap().blessed = true;
        assert_eq!(
            mobile_notoriety(&mut w, a, b),
            INVULNERABLE,
            "blessed counts from AOS on"
        );
    }

    #[test]
    fn young_player_may_help_a_target_that_is_no_player() {
        // `pmFrom?.Young == true && pmTarg?.Young == false`: a null pmTarg doesn't match.
        let mut w = World::new();
        let a = player_mobile::new_player_mobile(&mut w);
        let npc = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        w.mobile_mut(a).unwrap().player = true;
        player_mobile::set_young(&mut w, a, true);
        assert!(allow_beneficial(&mut w, a, npc));
        let b = player_mobile::new_player_mobile(&mut w);
        w.mobile_mut(b).unwrap().player = true;
        assert!(!allow_beneficial(&mut w, a, b));
    }
}
