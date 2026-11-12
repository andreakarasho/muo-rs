//! Everything the second half of `Mobile` calls that another module owns, behind one name each. Where the owner
//! has the real function it is called here; a `fn` pointer table (`SpellDeps`, `SpawnerDeps`, ..) is filled by the crate that owns it.

use muo_primitives::Point3D;

use crate::events::SpeechEventArgs;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::interfaces::{PartyId, SpawnerId, SpellId};
use crate::item::{ItemDelta, ops as io};
use crate::map::MapId;
use crate::map::los::LosObject;
use crate::mobile::part1::{combat, lifecycle, props_a, props_b};
use crate::net_state::NetStateId;
use crate::region::{RegionId, RegionType};
use crate::target::Targeted;
use crate::world::World;

// ---------- what a mobile sees ----------

/// `viewer.CanSee(target)`.
pub fn can_see_mobile(w: &mut World, viewer: MobileId, target: MobileId) -> bool {
    w.mobile(viewer)
        .map(|v| v.ty)
        .is_some_and(|ty| (ty.can_see_mobile)(w, viewer, target))
}

/// `viewer.CanSee(object)`: an absent entity is visible.
pub fn can_see_entity(w: &mut World, viewer: MobileId, e: Option<EntityId>) -> bool {
    w.mobile(viewer)
        .map(|v| v.ty)
        .is_some_and(|ty| (ty.can_see_object)(w, viewer, e))
}

/// `viewer.InLOS(target)`.
pub fn in_los_mobile(w: &mut World, viewer: MobileId, target: MobileId) -> bool {
    combat::in_los(w, viewer, target)
}

/// The `InLOS(object)` argument for an item.
pub fn los_item(item: ItemId) -> LosObject {
    LosObject::Item(item)
}

/// `map.GetClientsInRange(p)` with each client's mobile: connections that have no mobile are skipped.
pub fn clients_in_range(w: &World, map: MapId, p: Point3D) -> Vec<(NetStateId, MobileId)> {
    let range = w.maps.global_max_update_range;
    crate::map::enumerate::clients_in_range(w, Some(map), p.x, p.y, range)
        .filter_map(|ns| w.net_state(ns).and_then(|d| d.mobile).map(|m| (ns, m)))
        .collect()
}

// ---------- the mobile's own setters (part1) ----------

pub fn set_hits(w: &mut World, id: MobileId, v: i32) {
    props_b::set_hits(w, id, v);
}

pub fn set_stam(w: &mut World, id: MobileId, v: i32) {
    props_b::set_stam(w, id, v);
}

pub fn set_mana(w: &mut World, id: MobileId, v: i32) {
    props_b::set_mana(w, id, v);
}

pub fn set_map(w: &mut World, id: MobileId, map: Option<MapId>) {
    lifecycle::set_map(w, id, map);
}

pub fn set_holding(w: &mut World, id: MobileId, item: Option<ItemId>) {
    props_a::set_holding(w, id, item);
}

// ---------- items ----------

pub fn item_send_remove_packet(w: &mut World, item: ItemId) {
    io::send_remove_packet(w, item);
}

pub fn item_set_map(w: &mut World, item: ItemId, map: Option<MapId>) {
    io::set_map(w, item, map);
}

/// `item.PileWeight`.
pub fn item_pile_weight(w: &World, item: ItemId) -> i32 {
    io::pile_weight(w, item)
}

pub fn item_delta(w: &mut World, item: ItemId, flags: ItemDelta) {
    io::delta(w, item, flags);
}

pub fn item_clear_bounce(w: &mut World, item: ItemId) {
    io::clear_bounce(w, item);
}

pub fn item_move_to_world(w: &mut World, item: ItemId, loc: Point3D, map: Option<MapId>) {
    io::move_to_world_map(w, item, loc, map);
}

pub fn item_delete(w: &mut World, item: ItemId) {
    if let Some(ty) = w.item(item).map(|i| i.ty) {
        (ty.delete)(w, item);
    }
}

/// `pack.TryDropItem(from, item, sendFullMessage)`.
pub fn container_try_drop_item(
    w: &mut World,
    pack: ItemId,
    from: MobileId,
    item: ItemId,
    send_full_message: bool,
) -> bool {
    crate::item::container::try_drop_item(w, pack, Some(from), item, send_full_message)
}

/// `item is BankBox`.
pub fn item_is_bank_box(w: &World, item: ItemId) -> bool {
    crate::item::containers::is_bank_box(w, item)
}

/// `BankBox.Owner`.
pub fn bank_box_owner(w: &World, item: ItemId) -> Option<MobileId> {
    crate::item::containers::owner(w, item)
}

/// `BankBox.Opened`.
pub fn bank_box_opened(w: &World, item: ItemId) -> bool {
    crate::item::containers::bank_box_opened(w, item)
}

/// `BankBox.Close()`.
pub fn bank_box_close(w: &mut World, item: ItemId) {
    crate::item::containers::bank_box_close(w, item);
}

// ---------- regions ----------

fn region_ty(w: &World, id: MobileId) -> Option<(RegionId, &'static RegionType)> {
    let rid = props_b::region(w, id);
    w.region(rid).map(|r| (rid, r.ty))
}

/// `Region.Find(p, map)`.
pub fn region_find(w: &World, p: Point3D, map: Option<MapId>) -> RegionId {
    crate::region::find(w, p, map)
}

/// `Region.OnRegionChange(m, old, new)`.
pub fn region_on_region_change(
    w: &mut World,
    id: MobileId,
    old: Option<RegionId>,
    new: Option<RegionId>,
) {
    if let (Some(old), Some(new)) = (old, new) {
        crate::region::on_region_change(w, id, old, new);
    }
}

pub fn region_on_criminal_action(w: &mut World, id: MobileId, message: bool) {
    if let Some((rid, ty)) = region_ty(w, id) {
        (ty.on_criminal_action)(w, rid, id, message);
    }
}

pub fn region_on_resurrect(w: &mut World, id: MobileId) -> bool {
    region_ty(w, id).is_none_or(|(rid, ty)| (ty.on_resurrect)(w, rid, id))
}

pub fn region_on_before_death(w: &mut World, id: MobileId) -> bool {
    region_ty(w, id).is_none_or(|(rid, ty)| (ty.on_before_death)(w, rid, id))
}

pub fn region_on_death(w: &mut World, id: MobileId) {
    if let Some((rid, ty)) = region_ty(w, id) {
        (ty.on_death)(w, rid, id);
    }
}

pub fn region_on_damage(w: &mut World, id: MobileId, amount: &mut i32) -> bool {
    region_ty(w, id).is_none_or(|(rid, ty)| (ty.on_damage)(w, rid, id, amount))
}

pub fn region_on_heal(w: &mut World, id: MobileId, amount: &mut i32) -> bool {
    region_ty(w, id).is_none_or(|(rid, ty)| (ty.on_heal)(w, rid, id, amount))
}

pub fn region_on_double_click(w: &mut World, id: MobileId, target: Targeted) -> bool {
    region_ty(w, id).is_none_or(|(rid, ty)| (ty.on_double_click)(w, rid, id, &target))
}

pub fn region_on_speech(w: &mut World, id: MobileId, args: &mut SpeechEventArgs) {
    if let Some((rid, ty)) = region_ty(w, id) {
        (ty.on_speech)(w, rid, args);
    }
}

pub fn region_allow_beneficial(w: &mut World, id: MobileId, target: MobileId) -> bool {
    region_ty(w, id).is_none_or(|(rid, ty)| (ty.allow_beneficial)(w, rid, id, target))
}

pub fn region_allow_harmful(w: &mut World, id: MobileId, target: MobileId) -> bool {
    region_ty(w, id).is_none_or(|(rid, ty)| (ty.allow_harmful)(w, rid, id, target))
}

pub fn region_on_beneficial_action(w: &mut World, id: MobileId, target: MobileId) {
    if let Some((rid, ty)) = region_ty(w, id) {
        (ty.on_beneficial_action)(w, rid, id, target);
    }
}

/// `target.Region.OnGotBeneficialAction(this, target)`.
pub fn region_on_got_beneficial_action(w: &mut World, id: MobileId, target: MobileId) {
    if let Some((rid, ty)) = region_ty(w, target) {
        (ty.on_got_beneficial_action)(w, rid, id, target);
    }
}

pub fn region_on_did_harmful(w: &mut World, id: MobileId, target: MobileId) {
    if let Some((rid, ty)) = region_ty(w, id) {
        (ty.on_did_harmful)(w, rid, id, target);
    }
}

/// `target.Region.OnGotHarmful(this, target)`.
pub fn region_on_got_harmful(w: &mut World, id: MobileId, target: MobileId) {
    if let Some((rid, ty)) = region_ty(w, target) {
        (ty.on_got_harmful)(w, rid, id, target);
    }
}

/// `Region.OnCombatantChange(this, old, new)`.
pub fn region_on_combatant_change(
    w: &mut World,
    id: MobileId,
    old: Option<MobileId>,
    new: Option<MobileId>,
) -> bool {
    region_ty(w, id).is_none_or(|(rid, ty)| (ty.on_combatant_change)(w, rid, id, old, new))
}

/// `from.Region.OnSkillUse(from, skillID)`.
pub fn region_on_skill_use(w: &mut World, id: MobileId, skill: i32) -> bool {
    region_ty(w, id).is_none_or(|(rid, ty)| (ty.on_skill_use)(w, rid, id, skill))
}

pub fn region_on_location_changed(w: &mut World, id: MobileId, old: Point3D) {
    if let Some((rid, ty)) = region_ty(w, id) {
        (ty.on_location_changed)(w, rid, id, old);
    }
}

// ---------- spells, spawners, parties, mounts ----------

/// The spell system (in muo_content) as `fn` pointers its `Configure` phase sets. Unset: no
/// spell exists, which is what every call below answers when `m_Spell` is null.
#[derive(Default)]
pub struct SpellDeps {
    pub on_caster_hurt: Option<fn(&mut World, MobileId)>,
    pub on_caster_killed: Option<fn(&mut World, MobileId)>,
    pub on_connection_changed: Option<fn(&mut World, MobileId)>,
    pub on_caster_using_object: Option<fn(&mut World, MobileId, EntityId) -> bool>,
    pub on_caster_equipping: Option<fn(&mut World, MobileId, ItemId) -> bool>,
    pub blocks_movement: Option<fn(&World, SpellId) -> bool>,
    /// `m_Spell?.OnCasterMoving(d) == false` negated: true lets the move go on.
    pub on_caster_moving: Option<fn(&mut World, MobileId, muo_primitives::Direction) -> bool>,
}

/// `m_Spell?.OnCasterHurt()`.
pub fn spell_on_caster_hurt(w: &mut World, id: MobileId) {
    if let Some(f) = w
        .resources
        .get::<SpellDeps>()
        .and_then(|d| d.on_caster_hurt)
    {
        f(w, id);
    }
}

/// `m_Spell?.OnConnectionChanged()`.
pub fn spell_on_connection_changed(w: &mut World, id: MobileId) {
    if let Some(f) = w
        .resources
        .get::<SpellDeps>()
        .and_then(|d| d.on_connection_changed)
    {
        f(w, id);
    }
}

/// `m_Spell?.OnCasterKilled()`.
pub fn spell_on_caster_killed(w: &mut World, id: MobileId) {
    if let Some(f) = w
        .resources
        .get::<SpellDeps>()
        .and_then(|d| d.on_caster_killed)
    {
        f(w, id);
    }
}

/// `m_Spell?.OnCasterUsingObject(o) == false` negated: true lets the use go on.
pub fn spell_on_caster_using_object(w: &mut World, id: MobileId, o: EntityId) -> bool {
    match w
        .resources
        .get::<SpellDeps>()
        .and_then(|d| d.on_caster_using_object)
    {
        Some(f) => f(w, id, o),
        None => true,
    }
}

/// `m_Spell?.OnCasterEquipping(item) == false` negated.
pub fn spell_on_caster_equipping(w: &mut World, id: MobileId, item: ItemId) -> bool {
    match w
        .resources
        .get::<SpellDeps>()
        .and_then(|d| d.on_caster_equipping)
    {
        Some(f) => f(w, id, item),
        None => true,
    }
}

/// `spell.BlocksMovement`.
pub fn spell_blocks_movement(w: &World, spell: SpellId) -> bool {
    w.resources
        .get::<SpellDeps>()
        .and_then(|d| d.blocks_movement)
        .is_some_and(|f| f(w, spell))
}

/// `m_Spell?.OnCasterMoving(d) == false` negated.
pub fn spell_on_caster_moving(w: &mut World, id: MobileId, d: muo_primitives::Direction) -> bool {
    match w
        .resources
        .get::<SpellDeps>()
        .and_then(|x| x.on_caster_moving)
    {
        Some(f) => f(w, id, d),
        None => true,
    }
}

type NotifyDeathFn = fn(&mut World, SpawnerId, MobileId, Option<MobileId>);
type GetSpawnPositionFn = fn(&mut World, SpawnerId, MobileId, MapId) -> Point3D;
/// The spawner engine (muo_engines) as `fn` pointers its `register` sets. `SpawnerId` is the serial of the
/// spawner item. Unset: no spawner exists.
#[derive(Default)]
pub struct SpawnerDeps {
    /// `spawner.Remove(spawnable)`.
    pub remove: Option<fn(&mut World, SpawnerId, EntityId)>,
    /// `BaseSpawner.NotifySpawnedDeath(creature, killer)`.
    pub notify_death: Option<NotifyDeathFn>,
    /// `spawner.UnlinkOnTaming`: true for every spawner.
    pub unlink_on_taming: Option<fn(&World, SpawnerId) -> bool>,
    /// `spawner.ReturnOnDeactivate`.
    pub return_on_deactivate: Option<fn(&World, SpawnerId) -> bool>,
    /// `spawner.IsInSpawnBounds(location)`.
    pub is_in_spawn_bounds: Option<fn(&World, SpawnerId, Point3D) -> bool>,
    /// `spawner.GetSpawnPosition(m, map)`.
    pub get_spawn_position: Option<GetSpawnPositionFn>,
    /// `spawner.WalkingRange`.
    pub walking_range: Option<fn(&World, SpawnerId) -> i32>,
    /// `(spawner as RegionSpawner)?.SpawnRegion`: none for any other class.
    pub region_spawner_region: Option<fn(&World, SpawnerId) -> Option<RegionId>>,
    /// `spawner.Region`.
    pub region: Option<fn(&World, SpawnerId) -> Option<RegionId>>,
}

fn spawner_deps<T: Copy>(w: &World, f: impl FnOnce(&SpawnerDeps) -> Option<T>) -> Option<T> {
    w.resources.get::<SpawnerDeps>().and_then(f)
}

/// `spawner.ReturnOnDeactivate`.
pub fn spawner_return_on_deactivate(w: &World, s: SpawnerId) -> bool {
    spawner_deps(w, |d| d.return_on_deactivate).is_some_and(|f| f(w, s))
}

/// `spawner.IsInSpawnBounds(location)`.
pub fn spawner_is_in_spawn_bounds(w: &World, s: SpawnerId, loc: Point3D) -> bool {
    spawner_deps(w, |d| d.is_in_spawn_bounds).is_some_and(|f| f(w, s, loc))
}

/// `spawner.GetSpawnPosition(m, map)`; zero without a spawner engine.
pub fn spawner_get_spawn_position(w: &mut World, s: SpawnerId, m: MobileId, map: MapId) -> Point3D {
    match spawner_deps(w, |d| d.get_spawn_position) {
        Some(f) => f(w, s, m, map),
        None => Point3D::ZERO,
    }
}

/// `spawner.WalkingRange`.
pub fn spawner_walking_range(w: &World, s: SpawnerId) -> i32 {
    spawner_deps(w, |d| d.walking_range).map_or(0, |f| f(w, s))
}

/// `(spawner as RegionSpawner)?.SpawnRegion`.
pub fn spawner_region_spawner_region(w: &World, s: SpawnerId) -> Option<RegionId> {
    spawner_deps(w, |d| d.region_spawner_region).and_then(|f| f(w, s))
}

/// `spawner.Region`.
pub fn spawner_region(w: &World, s: SpawnerId) -> Option<RegionId> {
    spawner_deps(w, |d| d.region).and_then(|f| f(w, s))
}

/// `Spawner.Remove(entity)`.
pub fn spawner_remove(w: &mut World, spawner: SpawnerId, e: EntityId) {
    if let Some(f) = w.resources.get::<SpawnerDeps>().and_then(|d| d.remove) {
        f(w, spawner, e);
    }
}

/// `(Spawner as BaseSpawner)?.NotifySpawnedDeath(m, killer)`.
pub fn spawner_notify_death(
    w: &mut World,
    spawner: SpawnerId,
    m: MobileId,
    killer: Option<MobileId>,
) {
    if let Some(f) = w
        .resources
        .get::<SpawnerDeps>()
        .and_then(|d| d.notify_death)
    {
        f(w, spawner, m, killer);
    }
}

/// `Spawner?.UnlinkOnTaming == true`.
pub fn spawner_unlink_on_taming(w: &World, spawner: SpawnerId) -> bool {
    w.resources
        .get::<SpawnerDeps>()
        .and_then(|d| d.unlink_on_taming)
        .is_some_and(|f| f(w, spawner))
}

/// The party engine (muo_engines) as `fn` pointers its `Configure` sets. Unset: no party exists.
#[derive(Default)]
pub struct PartyDeps {
    pub on_stats_query: Option<fn(&mut World, PartyId, MobileId, MobileId)>,
    pub on_mana_changed: Option<fn(&mut World, PartyId, MobileId)>,
    pub on_stam_changed: Option<fn(&mut World, PartyId, MobileId)>,
}

/// `ip.OnStatsQuery(from, target)`.
pub fn party_on_stats_query(w: &mut World, party: PartyId, from: MobileId, target: MobileId) {
    if let Some(f) = w
        .resources
        .get::<PartyDeps>()
        .and_then(|d| d.on_stats_query)
    {
        f(w, party, from, target);
    }
}

/// `if (Party is IParty ip) { ip.OnManaChanged(this); ip.OnStamChanged(this); }`.
pub fn party_on_mana_stam_changed(w: &mut World, id: MobileId, send_mana: bool, send_stam: bool) {
    let Some(party) = w.mobile(id).and_then(|m| m.party) else {
        return;
    };
    if send_mana
        && let Some(f) = w
            .resources
            .get::<PartyDeps>()
            .and_then(|d| d.on_mana_changed)
    {
        f(w, party, id);
    }
    if send_stam
        && let Some(f) = w
            .resources
            .get::<PartyDeps>()
            .and_then(|d| d.on_stam_changed)
    {
        f(w, party, id);
    }
}

type OnRiderDamagedFn = fn(&mut World, MobileId, i32, Option<MobileId>, bool);
/// The mounts (`IMount`, content) as `fn` pointers their `register` sets. Unset: no mount exists.
#[derive(Default)]
pub struct MountDeps {
    /// `mount.OnRiderDamaged(amount, from, willKill)`; the id is the mount (a mobile or the item's mount).
    pub on_rider_damaged: Option<OnRiderDamagedFn>,
    /// `mount.Rider = null`.
    pub clear_rider: Option<fn(&mut World, MobileId)>,
}

/// `Mount?.OnRiderDamaged(amount, from, willKill)` of the rider `id`.
pub fn mount_on_rider_damaged(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    let Some(mount) = crate::mobile::part1::props_b::mount(w, id) else {
        return;
    };
    if let Some(f) = w
        .resources
        .get::<MountDeps>()
        .and_then(|d| d.on_rider_damaged)
    {
        f(w, mount, amount, from, will_kill);
    }
}

/// `mount.Rider = null`.
pub fn mount_clear_rider(w: &mut World, mount: MobileId) {
    if let Some(f) = w.resources.get::<MountDeps>().and_then(|d| d.clear_rider) {
        f(w, mount);
    }
}

// ---------- skills, trades, features ----------

/// `Skills.SpiritSpeak.Value`: the base of the skill in points. Mods and racial bonus are not added.
pub fn spirit_speak_value(w: &World, id: MobileId) -> f64 {
    w.mobile(id)
        .and_then(|m| m.skills.as_ref())
        .and_then(|s| s.peek(crate::skills::SkillName::SpiritSpeak as usize))
        .map_or(0.0, |s| f64::from(s.base) / 10.0)
}

/// `m_NetState.SendSkillsUpdate(Skills)`.
pub fn send_skills_update(w: &mut World, ns: NetStateId, id: MobileId) {
    let len = w
        .resources
        .get::<crate::skills::SkillTable>()
        .map_or(0, |t| t.info.len());
    let entries: Vec<_> = (0..len as i32)
        .filter_map(crate::skills::SkillName::from_i32)
        .map(|s| crate::skills::packet_entry(w, id, s))
        .collect();
    crate::packets::player::send_skills_update(w, ns, &entries);
}

/// `ServerFeatureFlags.PlayerTrading`.
pub fn player_trading_enabled(w: &World) -> bool {
    w.core.features.player_trading
}

/// `ServerFeatureFlags.PvPCombat`.
pub fn pvp_combat_enabled(w: &World) -> bool {
    w.core.features.pvp_combat
}

/// `NetState.CancelAllTrades()`.
pub fn cancel_all_trades(w: &mut World, ns: NetStateId) {
    crate::secure_trade::cancel_all_trades(w, ns);
}

/// `NetState.ValidateAllTrades()`.
pub fn validate_all_trades(w: &mut World, ns: NetStateId) {
    crate::secure_trade::validate_all_trades(w, ns);
}

/// `theirState.FindTradeContainer(m)`: the container of the trade with `m`'s client.
pub fn find_trade_container(w: &World, ns: NetStateId, with: MobileId) -> Option<ItemId> {
    crate::secure_trade::find_trade_container(w, ns, with)
}

/// `theirState.AddTrade(ourState)`: starts a trade, returns its container.
pub fn add_trade(w: &mut World, ns: NetStateId, other: NetStateId) -> Option<ItemId> {
    crate::secure_trade::add_trade(w, ns, other)
}

/// `item is SecureTradeContainer { Trade: { } trade } && trade.From.Mobile != viewer && trade.To.Mobile != viewer`.
pub fn trade_hides_container(w: &World, item: ItemId, viewer: MobileId) -> bool {
    if !w.item(item).is_some_and(|i| {
        i.ty.is(&crate::item::secure_trade_container::SECURE_TRADE_CONTAINER)
    }) {
        return false;
    }
    crate::item::secure_trade_container::trade_of(w, item)
        .and_then(|t| w.trades.get(&t))
        .is_some_and(|t| t.from.mobile != viewer && t.to.mobile != viewer)
}
