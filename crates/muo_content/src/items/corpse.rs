//! `Corpse` (with its migrations), `DecayedCorpse`, `LegacyHairInfo`, the corpse packets and
//! `NotorietyHandlers.CorpseNotoriety`.
//!
//! State is [`CorpseData`] in `Item::ext`; the container half is `muo_server::item::container`. The save layout
//! is version 19; versions 13-18 (`MigrateFrom(V13Content..V18Content)`) and everything older
//! (`Deserialize(reader, version)`) all load.
//!
//! `MilitiaFighterCorpse` is made through `CorpseClassHooks::quest_corpse`, the Uzeraan and Summoning quests of
//! `Open` through `CommonItemHooks::corpse_quest_open`, the head of `Carve` through `corpse_new_head`.

use indexmap::{IndexMap, IndexSet};
use muo_io::{GenericReader, GenericWriter, IoError, Result};
use muo_primitives::{Body, BodyTable, DateTime, Point3D, Serial, TimeSpan, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ext::Ext;
use muo_server::guild::GuildId;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::container::{self, CONTAINER};
use muo_server::item::{ImplFlag, ItemType, LootType, deps, ops};
use muo_server::layer::Layer;
use muo_server::lr_reason::LRReason;
use muo_server::map::{MapId, MapRules};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::notoriety as noto;
use muo_server::packets::Pk;
use muo_server::packets::message::MessageType;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::common::hooks;
use crate::core;
use crate::misc::notoriety::{self as cn, GuildRelation};

/// `CorpseFlag`.
pub mod flag {
    pub const CARVED: u32 = 0x1;
    pub const NO_BONES: u32 = 0x2;
    pub const IS_BONES: u32 = 0x4;
    pub const VISITED_BY_TAXIDERMIST: u32 = 0x8;
    pub const CHANNELED: u32 = 0x10;
    pub const CRIMINAL: u32 = 0x20;
    pub const ANIMATED: u32 = 0x40;
    pub const SELF_LOOTED: u32 = 0x80;
    pub const MURDERER: u32 = 0x100;
    pub const OWNER_WAS_BASE_CREATURE: u32 = 0x200;
    pub const OWNER_WAS_SUMMONED: u32 = 0x400;
    pub const OWNER_WAS_ANIMATED_DEAD: u32 = 0x800;
}

/// `Corpse.MonsterLootRightSacrifice`: 2 minutes.
pub const MONSTER_LOOT_RIGHT_SACRIFICE: TimeSpan = TimeSpan::from_mins(2);
const INSTANCED_CORPSE_TIME: TimeSpan = TimeSpan::from_mins(3);
const DEFAULT_DECAY_TIME: TimeSpan = TimeSpan::from_mins(7);
const BONE_DECAY_TIME: TimeSpan = TimeSpan::from_mins(7);

const SERIALIZATION_VERSION: i32 = 19;

/// `InstancedItemInfo`.
#[derive(Clone, Debug)]
pub struct InstancedItemInfo {
    pub item: ItemId,
    pub mobile: MobileId,
    pub perpetual: bool,
}

/// What `Corpse` adds to a `Container`.
#[derive(Clone, Debug)]
pub struct CorpseData {
    /// `_restoreEquip` (null is `None`).
    pub restore_equip: Option<Vec<ItemId>>,
    pub flags: u32,
    pub time_of_death: DateTime,
    /// `_restoreTable` (null is `None`).
    pub restore_table: Option<IndexMap<ItemId, Point3D>>,
    pub decay_timer: TimerHandle,
    pub looters: IndexSet<MobileId>,
    pub killer: Option<MobileId>,
    pub aggressors: Vec<MobileId>,
    pub owner: Option<MobileId>,
    /// Not serialized: what `OnCarve` reads of a creature owner, which is deleted at death.
    pub carve: Option<crate::creature::misc::CarveSnapshot>,
    pub corpse_name: Option<String>,
    pub access_level: AccessLevel,
    pub guild: Option<GuildId>,
    pub equip_items: Vec<ItemId>,
    pub hair_item_id: i32,
    pub hair_hue: i32,
    pub facial_hair_item_id: i32,
    pub facial_hair_hue: i32,
    // not serialized
    pub hair_serial: Serial,
    pub facial_hair_serial: Serial,
    pub devourer: Option<MobileId>,
    pub instanced_items: Option<IndexMap<ItemId, InstancedItemInfo>>,
    pub lifting_worn_item: Option<ItemId>,
    /// `Forensicist`.
    pub forensicist: Option<String>,
}

impl Default for CorpseData {
    fn default() -> Self {
        CorpseData {
            restore_equip: None,
            flags: 0,
            time_of_death: DateTime::MIN,
            restore_table: None,
            decay_timer: TimerHandle::default(),
            looters: IndexSet::new(),
            killer: None,
            aggressors: Vec::new(),
            owner: None,
            carve: None,
            corpse_name: None,
            access_level: AccessLevel::Player,
            guild: None,
            equip_items: Vec::new(),
            hair_item_id: 0,
            hair_hue: 0,
            facial_hair_item_id: 0,
            facial_hair_hue: 0,
            hair_serial: Serial::ZERO,
            facial_hair_serial: Serial::ZERO,
            devourer: None,
            instanced_items: None,
            lifting_worn_item: None,
            forensicist: None,
        }
    }
}

pub fn data(w: &World, id: ItemId) -> Option<&CorpseData> {
    w.item(id)?.ext.get::<CorpseData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut CorpseData> {
    w.item_mut(id)?.ext.get_mut::<CorpseData>()
}

/// `item is Corpse`.
pub fn is_corpse(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&CORPSE))
}

fn read<T>(w: &World, id: ItemId, default: T, f: impl FnOnce(&CorpseData) -> T) -> T {
    data(w, id).map_or(default, f)
}

/// `GetFlag(flag)`.
pub fn get_flag(w: &World, id: ItemId, f: u32) -> bool {
    read(w, id, false, |d| d.flags & f != 0)
}

/// `SetFlag(flag, value)` (the `Flags` setter marks the corpse dirty).
pub fn set_flag(w: &mut World, id: ItemId, f: u32, value: bool) {
    if let Some(d) = data_mut(w, id) {
        if value {
            d.flags |= f;
        } else {
            d.flags &= !f;
        }
    }
}

pub fn owner(w: &World, id: ItemId) -> Option<MobileId> {
    read(w, id, None, |d| d.owner)
}

pub fn killer(w: &World, id: ItemId) -> Option<MobileId> {
    read(w, id, None, |d| d.killer)
}

pub fn aggressors(w: &World, id: ItemId) -> Vec<MobileId> {
    read(w, id, Vec::new(), |d| d.aggressors.clone())
}

pub fn time_of_death(w: &World, id: ItemId) -> DateTime {
    read(w, id, DateTime::MIN, |d| d.time_of_death)
}

pub fn equip_items(w: &World, id: ItemId) -> Vec<ItemId> {
    read(w, id, Vec::new(), |d| d.equip_items.clone())
}

pub fn carved(w: &World, id: ItemId) -> bool {
    get_flag(w, id, flag::CARVED)
}

pub fn animated(w: &World, id: ItemId) -> bool {
    get_flag(w, id, flag::ANIMATED)
}

/// `InstancedCorpse`: `Core.SE && Core.Now < TimeOfDeath + InstancedCorpseTime`.
pub fn instanced_corpse(w: &World, id: ItemId) -> bool {
    core::se(w) && w.now.ticks() < time_of_death(w, id).ticks() + INSTANCED_CORPSE_TIME.ticks()
}

fn body_is_human(w: &World, body: i32) -> bool {
    w.resources
        .get::<BodyTable>()
        .is_some_and(|t| t.is_human(Body(body)))
}

fn body_is_monster(w: &World, body: i32) -> bool {
    w.resources
        .get::<BodyTable>()
        .is_some_and(|t| t.is_monster(Body(body)))
}

/// `ShowsWornGear`: `((Body)Amount).IsHuman && ItemID == 0x2006`.
fn shows_worn_gear(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| i.item_id == 0x2006 && body_is_human(w, i.amount))
}

/// `HairSerial`.
pub fn hair_serial(w: &mut World, id: ItemId) -> Serial {
    let Some(d) = data(w, id) else {
        return Serial::ZERO;
    };
    if d.hair_item_id > 0 && d.hair_serial == Serial::ZERO {
        let s = w.new_virtual_serial();
        if let Some(d) = data_mut(w, id) {
            d.hair_serial = s;
        }
    }
    read(w, id, Serial::ZERO, |d| d.hair_serial)
}

/// `FacialHairSerial`.
pub fn facial_hair_serial(w: &mut World, id: ItemId) -> Serial {
    let Some(d) = data(w, id) else {
        return Serial::ZERO;
    };
    if d.facial_hair_item_id > 0 && d.facial_hair_serial == Serial::ZERO {
        let s = w.new_virtual_serial();
        if let Some(d) = data_mut(w, id) {
            d.facial_hair_serial = s;
        }
    }
    read(w, id, Serial::ZERO, |d| d.facial_hair_serial)
}

// ---- construction ----

/// `GetCorpseName(owner)`: `bc.CorpseNameOverride ?? bc.CorpseName` for a creature.
fn corpse_name_of(w: &World, owner: MobileId) -> Option<String> {
    let d = crate::creature::data::cd(w, owner)?;
    if let Some(n) = &d.corpse_name_override {
        return Some(n.clone());
    }
    let ty = w.mobile(owner)?.ty;
    let f = ty.family::<crate::creature::vtable::CreatureType>()?;
    (f.corpse_name)(w, owner).map(str::to_string)
}

/// `new Corpse(owner, hairItemId, hairHue, facialHairItemId, facialHairHue, equipItems)` of class `ty`.
pub fn new_corpse_of(
    w: &mut World,
    ty: &'static ItemType,
    owner: MobileId,
    hair: (i32, i32),
    facial_hair: (i32, i32),
    equip_items: Vec<ItemId>,
) -> ItemId {
    let id = container::new_container(w, ty, 0x2006);
    let (body, hue, direction, name, access_level, guild, criminal, is_player) = {
        let Some(m) = w.mobile(owner) else { return id };
        (
            muo_server::mobile::part1::props_b::body(w, owner).0,
            (m.ty.hue)(w, owner),
            m.direction,
            (m.ty.name)(w, owner),
            m.access_level,
            m.guild,
            m.criminal,
            m.player,
        )
    };
    let murderer = muo_server::mobile::base::murderer(w, owner);

    let mut d = CorpseData::default();
    // To suppress console warnings, stackable must be true
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, body); // protocol defines that for itemid 0x2006, amount=body
    ops::set_stackable(w, id, false);

    ops::set_movable(w, id, false);
    (ty.set_hue)(w, id, hue);
    ops::set_direction(w, id, direction);
    ops::set_name(w, id, name);

    d.owner = Some(owner);
    d.carve = crate::creature::misc::carve_snapshot(w, owner);
    d.corpse_name = corpse_name_of(w, owner);
    d.time_of_death = w.now;
    d.access_level = access_level;
    d.guild = guild;
    let mut f = 0;
    if murderer {
        f |= flag::MURDERER;
    }
    if criminal {
        f |= flag::CRIMINAL;
    }

    // Snapshot owner type & state. BaseCreatures are deleted after death, so the owner mobile reference
    // becomes null after restart - notoriety must be derivable without it.
    let creature = crate::creature::data::is_creature(w, owner);
    if creature {
        f |= flag::OWNER_WAS_BASE_CREATURE;
        if crate::creature::data::read(w, owner, false, |c| c.summoned) {
            f |= flag::OWNER_WAS_SUMMONED;
        }
        if crate::creature::misc::is_animated_dead(w, owner) {
            f |= flag::OWNER_WAS_ANIMATED_DEAD;
        }
    }
    if hair.0 > 0 {
        d.hair_item_id = hair.0;
        d.hair_hue = hair.1;
    }
    if facial_hair.0 > 0 {
        d.facial_hair_item_id = facial_hair.0;
        d.facial_hair_hue = facial_hair.1;
    }
    // This corpse does not turn to bones if: the owner is not a player
    if !is_player {
        f |= flag::NO_BONES;
    }
    d.flags = f;
    d.equip_items = equip_items;

    let (aggressors, aggressed) = w
        .mobile(owner)
        .map(|m| (m.aggressors.clone(), m.aggressed.clone()))
        .unwrap_or_default();
    let mut last_time = TimeSpan::MAX;
    let now = w.now;
    for info in &aggressors {
        let since = TimeSpan::from_ticks(now.ticks() - info.last_combat_time.ticks());
        if since < last_time {
            d.killer = Some(info.attacker);
            last_time = since;
        }
        if !creature && !info.criminal_aggression {
            d.aggressors.push(info.attacker);
        }
    }
    for info in &aggressed {
        let since = TimeSpan::from_ticks(now.ticks() - info.last_combat_time.ticks());
        if since < last_time {
            d.killer = Some(info.defender);
            last_time = since;
        }
        if !creature {
            d.aggressors.push(info.defender);
        }
    }
    if creature {
        if let Some(master) = crate::creature::control::get_master(w, owner) {
            d.aggressors.push(master);
        }
        let hits_max = muo_server::mobile::base::hits_max(w, owner);
        for ds in crate::creature::death::get_looting_rights(w, owner, hits_max) {
            if ds.has_right {
                d.aggressors.push(ds.mobile);
            }
        }
    }

    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    begin_decay(w, id, DEFAULT_DECAY_TIME);
    devour_corpse(w, id);
    id
}

/// `new Corpse(owner, equipItems)`.
pub fn new_corpse(w: &mut World, owner: MobileId, equip_items: Vec<ItemId>) -> ItemId {
    new_corpse_of(w, &CORPSE, owner, (0, 0), (0, 0), equip_items)
}

// ---- decay ----

/// `BeginDecay(delay)`.
pub fn begin_decay(w: &mut World, id: ItemId, delay: TimeSpan) {
    let Some(mut h) = data(w, id).map(|d| d.decay_timer) else {
        return;
    };
    // `_decayTimer?.Stop()`
    h.cancel(w);
    let delay = if delay < TimeSpan::ZERO {
        TimeSpan::ZERO
    } else {
        delay
    };
    let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| decay_tick(w, id));
    if let Some(d) = data_mut(w, id) {
        d.decay_timer = h;
    }
}

fn decay_tick(w: &mut World, id: ItemId) {
    if !get_flag(w, id, flag::NO_BONES) {
        turn_to_bones(w, id);
    } else if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// `TurnToBones()`.
pub fn turn_to_bones(w: &mut World, id: ItemId) {
    if ops::is_deleted(w, id) {
        return;
    }
    become_bones(w, id);
    set_flag(w, id, flag::NO_BONES, true);
    set_flag(w, id, flag::IS_BONES, true);
    begin_decay(w, id, BONE_DECAY_TIME);
}

/// `ProcessDelta(); SendRemovePacket(); ItemID = Utility.Random(0xECA, 9); Hue = 0; ProcessDelta();`
fn become_bones(w: &mut World, id: ItemId) {
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    (ty.process_delta)(w, id);
    ops::send_remove_packet(w, id);
    let g = utility::random_from(&mut w.rng, 0xECA, 9);
    (ty.set_item_id)(w, id, g);
    (ty.set_hue)(w, id, 0);
    (ty.process_delta)(w, id);
}

fn on_after_delete(w: &mut World, id: ItemId) {
    if let Some(d) = data_mut(w, id) {
        let mut h = d.decay_timer;
        d.decay_timer = TimerHandle::default();
        d.lifting_worn_item = None;
        h.cancel(w);
    }
}

/// `Devoured`.
pub fn devoured(w: &World, id: ItemId) -> bool {
    read(w, id, false, |d| d.devourer.is_some())
}

/// `DevourCorpse()`.
pub fn devour_corpse(w: &mut World, id: ItemId) -> bool {
    let Some(d) = data(w, id) else { return false };
    let Some(killer) = d.killer else { return false };
    if d.devourer.is_some() || ops::is_deleted(w, id) {
        return false;
    }
    let owner_gone = d.owner.is_none_or(|o| deps::mobile_deleted(w, o));
    let alive = w
        .mobile(killer)
        .is_some_and(|m| !m.deleted && (m.ty.alive)(w, killer));
    if !alive || owner_gone {
        return false;
    }
    let Some(f) = hooks(w).and_then(|h| h.devour) else {
        return false;
    };
    // `Killer is not IDevourer devourer`: only PlagueBeast implements it.
    if !crate::factory::mobile_is(w, killer, "Server.Mobiles.PlagueBeast") {
        return false;
    }
    if let Some(d) = data_mut(w, id) {
        d.devourer = Some(killer);
    }
    f(w, killer, id)
}

// ---- restore table ----

/// `GetRestoreInfo(item, ref loc)`.
pub fn get_restore_info(w: &World, id: ItemId, item: ItemId) -> Option<Point3D> {
    data(w, id)?.restore_table.as_ref()?.get(&item).copied()
}

/// `SetRestoreInfo(item, loc)`.
pub fn set_restore_info(w: &mut World, id: ItemId, item: ItemId, loc: Point3D) {
    if let Some(d) = data_mut(w, id) {
        d.restore_table
            .get_or_insert_with(IndexMap::new)
            .insert(item, loc);
    }
}

/// `ClearRestoreInfo(item)`.
pub fn clear_restore_info(w: &mut World, id: ItemId, item: ItemId) {
    if let Some(d) = data_mut(w, id)
        && let Some(t) = d.restore_table.as_mut()
    {
        t.shift_remove(&item);
        if t.is_empty() {
            d.restore_table = None;
        }
    }
}

// ---- notoriety and looting ----

fn rules_harmful(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.map)
        .and_then(|m| w.maps.get(m))
        .is_some_and(|m| m.rules.contains(MapRules::HARMFUL_RESTRICTIONS))
}

/// `NotorietyHandlers.CorpseNotoriety(source, target)`.
pub fn corpse_notoriety(w: &World, source: MobileId, corpse: ItemId) -> i32 {
    let Some(d) = data(w, corpse) else {
        return noto::INNOCENT;
    };
    if d.access_level > AccessLevel::Player {
        return noto::CAN_BE_ATTACKED;
    }
    // BaseCreatures are deleted on death, so the owner is null after a restart; the flag is the persisted snapshot.
    let creature = d
        .owner
        .filter(|o| crate::creature::data::is_creature(w, *o));
    let owner_was_creature = d.flags & flag::OWNER_WAS_BASE_CREATURE != 0 || creature.is_some();

    if !owner_was_creature {
        if d.flags & flag::MURDERER != 0 {
            return noto::MURDERER;
        }
        // `(target.Map?.Rules & HarmfulRestrictions) == 0` is false for no map (null != 0)
        let has_map = w.item(corpse).and_then(|i| i.map).is_some();
        if d.flags & flag::CRIMINAL != 0 && has_map && !rules_harmful(w, corpse) {
            return noto::CRIMINAL;
        }
    }

    let source_guild = cn::guild_for_read(w, source);
    // `GetGuildFor(target.Guild, target.Owner)`: a controlled creature takes its master's guild (or none), any
    // other owner leaves the corpse's own guild.
    let target_guild = match d.owner.map(|o| cn::resolve_guild_for(w, o).set_guild) {
        Some(Some(g)) => g,
        _ => d.guild,
    };
    let src_faction = cn::faction(w, source);
    let trg_faction = d.owner.and_then(|o| cn::faction(w, o));
    let list = &d.aggressors;

    if let (Some(sg), Some(tg)) = (source_guild, target_guild) {
        match cn::relation(w, sg, tg) {
            GuildRelation::Same | GuildRelation::Ally => return noto::ALLY,
            GuildRelation::Enemy => return noto::ENEMY,
            GuildRelation::None => {}
        }
    }

    let corpse_map = w.item(corpse).and_then(|i| i.map);
    let facet = cn::facet(w);
    let source_on_facet = facet.is_some() && w.mobile(source).and_then(|m| m.map) == facet;
    let loc = w.item(corpse).map_or(Point3D::new(0, 0, 0), |i| i.location);
    let different_factions = matches!((src_faction, trg_faction), (Some(a), Some(b)) if a != b);

    if owner_was_creature {
        if different_factions && source_on_facet {
            return noto::ENEMY;
        }
        if let Some(c) = creature
            && cn::check_house_flag(w, source, Some(c), loc, corpse_map)
        {
            return noto::CAN_BE_ATTACKED;
        }

        let mut actual = noto::CAN_BE_ATTACKED;
        let amount = w.item(corpse).map_or(0, |i| i.amount);
        if d.flags & flag::MURDERER != 0
            || (body_is_monster(w, amount) && d.flags & flag::OWNER_WAS_SUMMONED != 0)
            || d.flags & flag::OWNER_WAS_ANIMATED_DEAD != 0
        {
            actual = noto::MURDERER;
        }

        if w.now.ticks() >= d.time_of_death.ticks() + MONSTER_LOOT_RIGHT_SACRIFICE.ticks() {
            return actual;
        }

        let same_party = hooks(w).and_then(|h| h.same_party);
        for &a in list {
            if a == source || same_party.is_some_and(|f| f(w, a, source)) {
                return actual;
            }
        }
        return noto::INNOCENT;
    }

    if different_factions
        && source_on_facet
        && list.iter().any(|&a| {
            a == source || crate::factory::mobile_is(w, a, "Server.Factions.BaseFactionGuard")
        })
    {
        return noto::ENEMY;
    }

    if cn::check_house_flag(w, source, d.owner, loc, corpse_map) {
        return noto::CAN_BE_ATTACKED;
    }
    // `target.Owner is not PlayerMobile`
    let owner_is_pm = d.owner.is_some_and(|o| {
        w.mobile(o)
            .is_some_and(|m| m.ext.get::<crate::player_mobile::PlayerData>().is_some())
    });
    if !owner_is_pm {
        return noto::CAN_BE_ATTACKED;
    }
    if list.contains(&source) {
        return noto::CAN_BE_ATTACKED;
    }
    noto::INNOCENT
}

fn notoriety_of(w: &World, from: MobileId, id: ItemId) -> i32 {
    match hooks(w).and_then(|h| h.corpse_notoriety) {
        Some(f) => f(w, from, id),
        None => corpse_notoriety(w, from, id),
    }
}

/// `IsCriminalAction(from)`.
pub fn is_criminal_action(w: &World, id: ItemId, from: MobileId) -> bool {
    let owner = owner(w, id);
    if Some(from) == owner
        || w.mobile(from)
            .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
    {
        return false;
    }
    if let Some(o) = owner
        && hooks(w)
            .and_then(|h| h.party_can_loot)
            .is_some_and(|f| f(w, o, from))
    {
        return false;
    }
    notoriety_of(w, from, id) == noto::INNOCENT
}

/// `CanLoot(from, item)`.
pub fn can_loot(w: &World, id: ItemId, from: MobileId) -> bool {
    !is_criminal_action(w, id, from) || !rules_harmful(w, id)
}

fn owner_is_player(w: &World, id: ItemId) -> bool {
    owner(w, id).is_some_and(|o| w.mobile(o).is_some_and(|m| m.player))
}

/// `CheckLoot(from, item)`.
pub fn check_loot(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !can_loot(w, id, from) {
        if !owner_is_player(w, id) {
            deps::send_localized_message(w, from, 1005035); // You did not earn the right to loot this creature!
        } else {
            deps::send_localized_message(w, from, 1010049); // You may not loot this corpse.
        }
        return false;
    }
    if is_criminal_action(w, id, from) {
        if !owner_is_player(w, id) {
            deps::send_localized_message(w, from, 1005036); // Looting this monster corpse will be a criminal act!
        } else {
            deps::send_localized_message(w, from, 1005038); // Looting this corpse will be a criminal act!
        }
    }
    true
}

fn add_to_looters(w: &mut World, id: ItemId, from: MobileId) {
    if let Some(d) = data_mut(w, id) {
        d.looters.insert(from);
    }
}

// ---- instanced loot ----

fn info_is_owner(w: &World, info: &InstancedItemInfo, m: MobileId) -> bool {
    // Cursed Items are part of everyone's instanced corpse... (?)
    if w.item(info.item)
        .is_some_and(|i| i.loot_type == LootType::Cursed)
    {
        return true;
    }
    if info.mobile == m {
        return true;
    }
    hooks(w)
        .and_then(|h| h.same_party)
        .is_some_and(|f| f(w, info.mobile, m))
}

fn is_child_visible_to(w: &World, id: ItemId, m: MobileId, child: ItemId) -> bool {
    let Some(mob) = w.mobile(m) else { return true };
    if !mob.player || mob.access_level > AccessLevel::Player {
        return true;
    }
    let Some(d) = data(w, id) else { return true };
    let Some(map) = &d.instanced_items else {
        return true;
    };
    let Some(info) = map.get(&child) else {
        return true;
    };
    (!instanced_corpse(w, id) && !info.perpetual) || info_is_owner(w, info, m)
}

/// `AssignInstancedLoot()`.
pub fn assign_instanced_loot(w: &mut World, id: ItemId) {
    let aggressors = aggressors(w, id);
    let items = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
    if aggressors.is_empty() || items.is_empty() {
        return;
    }
    if let Some(d) = data_mut(w, id)
        && d.instanced_items.is_none()
    {
        d.instanced_items = Some(IndexMap::new());
    }

    let mut stackables: std::collections::VecDeque<ItemId> = Default::default();
    let mut unstackables: std::collections::VecDeque<ItemId> = Default::default();
    for item in items {
        let Some(i) = w.item(item) else { continue };
        if i.loot_type != LootType::Cursed {
            // Don't have cursed items take up someone's item spot.. (?)
            if i.flags.contains(ImplFlag::STACKABLE) {
                stackables.push_back(item);
            } else {
                unstackables.push_back(item);
            }
        }
    }

    let mut attackers = aggressors;
    shuffle(w, &mut attackers);

    // stackables first, for the remaining stackables, have those be randomly added after
    while let Some(item) = stackables.pop_front() {
        let amount = w.item(item).map_or(0, |i| i.amount);
        if amount as usize >= attackers.len() {
            let n = attackers.len() as i32;
            let (per, remainder) = (amount / n, amount % n);
            let count = if remainder == 0 {
                attackers.len() - 1
            } else {
                attackers.len()
            };
            for a in attackers.iter().take(count) {
                // LiftItemDupe automagically adds it as a child item to the corpse
                let cur = w.item(item).map_or(0, |i| i.amount);
                if let Some(split) =
                    muo_server::mobile::part2::items::lift_item_dupe(w, item, cur - per)
                {
                    add_instanced(w, id, split, *a);
                }
                // What happens to the remaining portion?  TEMP FOR NOW UNTIL OSI VERIFICATION:  Treat as Single Item.
            }
            if remainder == 0 {
                add_instanced(w, id, item, attackers[attackers.len() - 1]);
            } else {
                unstackables.push_back(item);
            }
        } else {
            // What happens in this case?  TEMP FOR NOW UNTIL OSI VERIFICATION:  Treat as Single Item.
            unstackables.push_back(item);
        }
    }

    let mut index = 0;
    while let Some(item) = unstackables.pop_front() {
        let m = attackers[index % attackers.len()];
        index += 1;
        add_instanced(w, id, item, m);
    }
}

fn add_instanced(w: &mut World, id: ItemId, item: ItemId, m: MobileId) {
    if let Some(d) = data_mut(w, id) {
        d.instanced_items.get_or_insert_with(IndexMap::new).insert(
            item,
            InstancedItemInfo {
                item,
                mobile: m,
                perpetual: false,
            },
        );
    }
}

/// `PooledRefList.Shuffle()`: `Random.Shuffle(span)`, forward Fisher-Yates.
fn shuffle(w: &mut World, v: &mut [MobileId]) {
    muo_primitives::random::Rng::shuffle(&mut w.rng, v);
}

/// `AddCarvedItem(carved, carver)`.
pub fn add_carved_item(w: &mut World, id: ItemId, carved: ItemId, carver: MobileId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.drop_item)(w, id, carved);
    }
    if instanced_corpse(w, id) {
        add_instanced(w, id, carved, carver);
    }
}

// ---- carve ----

/// `Carve(from, item)`.
pub fn carve(w: &mut World, id: ItemId, from: MobileId, item: Option<ItemId>) {
    if is_criminal_action(w, id, from) && rules_harmful(w, id) {
        if !owner_is_player(w, id) {
            deps::send_localized_message(w, from, 1005035); // You did not earn the right to loot this creature!
        } else {
            deps::send_localized_message(w, from, 1010049); // You may not loot this corpse.
        }
        return;
    }
    let dead = owner(w, id);
    if get_flag(w, id, flag::CARVED) || dead.is_none() {
        deps::send_localized_message(w, from, 500485); // You see nothing useful to carve from the corpse.
    } else if shows_worn_gear(w, id) {
        let (loc, map) = w
            .item(id)
            .map_or((Point3D::new(0, 0, 0), None), |i| (i.location, i.map));
        let mut parts = Vec::new();
        parts.push(crate::factory::make_item_args(
            w,
            "Server.Items.Blood",
            &[crate::props::Value::Int(0x122D)],
        ));
        for n in ["Torso", "LeftLeg", "LeftArm", "RightLeg", "RightArm"] {
            parts.push(crate::factory::make_item(w, &format!("Server.Items.{n}")));
        }
        parts.push(
            hooks(w)
                .and_then(|h| h.corpse_new_head)
                .and_then(|f| f(w, dead.unwrap())),
        );
        for it in parts.into_iter().flatten() {
            ops::move_to_world_map(w, it, loc, map);
        }
        set_flag(w, id, flag::CARVED, true);
        become_bones(w, id);
        if is_criminal_action(w, id, from) {
            if let Some(f) = hooks(w).and_then(|h| h.criminal_action) {
                f(w, from, true);
            }
            if let Some(f) = hooks(w).and_then(|h| h.award_karma) {
                f(w, from, -70, true);
            }
        } else if let Some(f) = hooks(w).and_then(|h| h.award_karma) {
            f(w, from, -20, true);
        }
    } else if let Some(dead) = dead.filter(|d| crate::creature::data::is_creature(w, *d)) {
        if let Some(f) = hooks(w).and_then(|h| h.creature_on_carve) {
            f(w, dead, from, id, item);
        }
    } else if let Some(snap) = read(w, id, None, |d| d.carve.clone()) {
        // the creature was deleted when it died: its carve snapshot stands in for it
        crate::creature::misc::on_carve_snapshot(w, &snap, from, id);
    } else {
        deps::send_localized_message(w, from, 500485);
    }
}

// ---- open ----

/// `Open(from, checkSelfLoot)`.
pub fn open(w: &mut World, id: ItemId, from: MobileId, check_self_loot: bool) {
    let Some(is_player_level) = w
        .mobile(from)
        .map(|m| m.access_level <= AccessLevel::Player)
    else {
        return;
    };
    let loc = ops::get_world_location(w, id);
    if is_player_level && !deps::mobile_in_range(w, from, loc, 2) {
        deps::send_localized_message(w, from, 500446); // That is too far away.
        return;
    }

    let has_items = w.item(id).is_some_and(|i| !i.items.is_empty());
    if check_self_loot
        && Some(from) == owner(w, id)
        && !get_flag(w, id, flag::SELF_LOOTED)
        && has_items
    {
        if let Some(robe) = deps::find_item_on_layer(w, from, Layer::OuterTorso)
            && w.item(robe)
                .is_some_and(|i| super::is_named(i.ty, "Server.Items.DeathRobe"))
        {
            let (at, map) = w.mobile(from).map_or((loc, None), |m| (m.location, m.map));
            if let Some(map) = map.filter(|m| *m != MapId::INTERNAL) {
                ops::move_to_world_map(w, robe, at, Some(map));
            }
        }

        let pack = muo_server::mobile::part1::props_b::backpack(w, from);
        let restore_equip = data(w, id).and_then(|d| d.restore_equip.clone());
        if let (Some(re), Some(pack)) = (&restore_equip, pack) {
            let packed = w.item(pack).map(|i| i.items.clone()).unwrap_or_default();
            // Only items in the top-level pack are re-equipped
            for pi in packed {
                if re.contains(&pi)
                    && w.item(pi)
                        .is_some_and(|i| i.flags.contains(ImplFlag::MOVABLE))
                {
                    muo_server::mobile::base::equip_item(w, from, pi);
                }
            }
        }

        let mut didnt_fit = false;
        let items = w.item(id).map(|i| i.items.clone()).unwrap_or_default();
        for item in items {
            if didnt_fit {
                break;
            }
            let Some(it) = w.item(item) else { continue };
            let (layer, movable) = (it.layer, it.flags.contains(ImplFlag::MOVABLE));
            if matches!(layer, Layer::Hair | Layer::FacialHair) || !movable {
                continue;
            }
            let Some(rloc) = get_restore_info(w, id, item) else {
                continue;
            };
            if let Some(pack) =
                pack.filter(|p| container::check_hold(w, *p, Some(from), item, false))
            {
                ops::set_location(w, item, rloc);
                if let Some(pty) = w.item(pack).map(|i| i.ty) {
                    (pty.add_item)(w, pack, item);
                }
                if restore_equip.as_ref().is_some_and(|r| r.contains(&item)) {
                    muo_server::mobile::base::equip_item(w, from, item);
                }
            } else {
                didnt_fit = true;
            }
        }

        muo_server::mobile::part2::effects::play_sound(w, from, 0x3E3);

        if w.item(id).is_some_and(|i| !i.items.is_empty()) {
            deps::send_localized_message(w, from, 1062472); // You gather some of your belongings. The rest remain on the corpse.
        } else {
            set_flag(w, id, flag::CARVED, true);
            if w.item(id).is_some_and(|i| i.item_id == 0x2006) {
                become_bones(w, id);
            }
            deps::send_localized_message(w, from, 1062471); // You quickly gather all of your belongings.
        }
        set_flag(w, id, flag::SELF_LOOTED, true);
    }

    if !check_loot(w, id, from) {
        return;
    }

    if !w
        .mobile(from)
        .is_some_and(|m| m.ext.get::<crate::player_mobile::PlayerData>().is_some())
    {
        return;
    }
    if hooks(w)
        .and_then(|h| h.corpse_quest_open)
        .is_some_and(|f| f(w, from, id))
    {
        return;
    }

    (CONTAINER.on_double_click)(w, id, from);
}

// ---- virtuals ----

fn is_child_public(w: &World, id: ItemId, child: ItemId) -> bool {
    shows_worn_gear(w, id)
        && read(w, id, false, |d| {
            d.lifting_worn_item == Some(child) || d.equip_items.contains(&child)
        })
}

fn check_item_use(w: &mut World, id: ItemId, from: MobileId, item: ItemId) -> bool {
    (CONTAINER.check_item_use)(w, id, from, item) && (item == id || can_loot(w, id, from))
}

fn check_lift(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    item: ItemId,
    reject: &mut LRReason,
) -> bool {
    (CONTAINER.check_lift)(w, id, from, item, reject) && can_loot(w, id, from)
}

fn on_item_used(w: &mut World, id: ItemId, from: MobileId, item: ItemId) {
    (CONTAINER.on_item_used)(w, id, from, item);
    if w.item(item)
        .is_some_and(|i| super::is_named(i.ty, "Server.Items.Food"))
    {
        muo_server::mobile::base::revealing_action(w, from);
    }
    if item != id
        && is_criminal_action(w, id, from)
        && let Some(f) = hooks(w).and_then(|h| h.criminal_action)
    {
        f(w, from, true);
    }
    add_to_looters(w, id, from);
    if let Some(d) = data_mut(w, id)
        && let Some(m) = d.instanced_items.as_mut()
    {
        m.shift_remove(&item);
    }
}

fn on_item_lifted(w: &mut World, id: ItemId, from: MobileId, item: ItemId) {
    (CONTAINER.on_item_lifted)(w, id, from, item);
    if item != id && Some(from) != owner(w, id) {
        muo_server::mobile::base::revealing_action(w, from);
    }
    if item != id
        && is_criminal_action(w, id, from)
        && let Some(f) = hooks(w).and_then(|h| h.criminal_action)
    {
        f(w, from, true);
    }
    add_to_looters(w, id, from);
    if let Some(d) = data_mut(w, id) {
        if let Some(m) = d.instanced_items.as_mut() {
            m.shift_remove(&item);
        }
        // Lifted gear is loot from then on. Clients only learn worn gear from 0x89 on first sight, so a
        // piece put back would show worn after a relog but loose to everyone already watching.
        if let Some(p) = d.equip_items.iter().position(|x| *x == item) {
            d.equip_items.remove(p);
            d.lifting_worn_item = Some(item);
        }
    }
}

fn on_item_removed(w: &mut World, id: ItemId, item: ItemId) {
    (CONTAINER.on_item_removed)(w, id, item);
    if let Some(d) = data_mut(w, id)
        && d.lifting_worn_item == Some(item)
    {
        d.lifting_worn_item = None;
    }
}

fn get_context_menu_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    (CONTAINER.get_context_menu_entries)(w, id, from, list);
    let alive = w.mobile(from).is_some_and(|m| (m.ty.alive)(w, from));
    if core::aos(w) && owner(w, id) == Some(from) && alive {
        let mut e = ContextMenuEntry::new(6215, 2);
        e.on_click = Some(std::rc::Rc::new(|w: &mut World, from, target| {
            if let EntityId::Item(c) = target
                && muo_server::mobile::part2::stats::check_alive(w, from, true)
                && is_corpse(w, c)
            {
                open(w, c, from, false);
            }
        }));
        list.push(e);
    }
}

fn add_name_property(w: &World, id: ItemId, list: &mut muo_server::property_list::PropertyList) {
    let name = w.item(id).and_then(|i| i.name.clone()).unwrap_or_default();
    let corpse_form = w.item(id).is_some_and(|i| i.item_id == 0x2006);
    match (corpse_form, read(w, id, None, |d| d.corpse_name.clone())) {
        (true, Some(cn)) => list.add_text(&cn),
        _ => list.add_str(1046414, &name), // the remains of ~1_NAME~
    }
}

fn on_aos_single_click(w: &mut World, id: ItemId, from: MobileId) {
    let hue = noto::get_hue(w, notoriety_of(w, from, id));
    let Some((header, args)) = ops::property_list(w, id).map(|p| (p.header, p.header_args.clone()))
    else {
        return;
    };
    if header > 0
        && let Some(ns) = deps::net_state_of(w, from)
    {
        let (graphic, name) = w.item(id).map_or((0, String::new()), |i| {
            (i.item_id, i.name.clone().unwrap_or_default())
        });
        muo_server::packets::message::send_message_localized(
            w,
            ns,
            id.0,
            graphic,
            MessageType::LABEL,
            hue,
            3,
            header,
            &name,
            args.as_deref().unwrap_or(""),
        );
    }
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    let hue = noto::get_hue(w, notoriety_of(w, from, id));
    let Some(ns) = deps::net_state_of(w, from) else {
        return;
    };
    let (graphic, name) = w.item(id).map_or((0, String::new()), |i| {
        (i.item_id, i.name.clone().unwrap_or_default())
    });
    let corpse_name = read(w, id, None, |d| d.corpse_name.clone());
    if graphic == 0x2006
        && let Some(cn) = corpse_name
    {
        muo_server::packets::message::send_message(
            w,
            ns,
            id.0,
            graphic,
            MessageType::LABEL,
            hue,
            3,
            true,
            None,
            "",
            &cn,
        );
    } else {
        // the remains of ~1_NAME~ (the name rides in the `args` slot)
        muo_server::packets::message::send_message_localized(
            w,
            ns,
            id.0,
            graphic,
            MessageType::LABEL,
            hue,
            3,
            1046414,
            "",
            &name,
        );
    }
}

fn send_info_to(w: &mut World, id: ItemId, ns: NetStateId, world_packet: &[u8]) {
    (CONTAINER.send_info_to)(w, id, ns, world_packet);
    if shows_worn_gear(w, id)
        && let Some(m) = w.net_mobile(ns)
    {
        send_corpse_content(w, ns, m, id);
        send_corpse_equip(w, ns, m, id);
    }
}

// ---- packets ----

/// `SendCorpseEquip(beholder, beheld)`.
pub fn send_corpse_equip(w: &mut World, ns: NetStateId, beholder: MobileId, corpse: ItemId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let list = equip_items(w, corpse);
    let mut p = Pk::new(8 + (list.len() + 2) * 5);
    p.u8(0x89);
    p.zeros(2);
    p.serial(corpse.0);
    for item in list {
        let ok = w
            .item(item)
            .is_some_and(|i| !i.deleted() && i.parent == Some(EntityId::Item(corpse)));
        if ok && deps::can_see(w, beholder, item) {
            let layer = w.item(item).map_or(0, |i| i.layer.0);
            p.u8(layer.wrapping_add(1));
            p.serial(item.0);
        }
    }
    if owner(w, corpse).is_some() {
        if read(w, corpse, 0, |d| d.hair_item_id) > 0 {
            let s = hair_serial(w, corpse);
            p.u8(Layer::Hair.0 + 1);
            p.serial(s);
        }
        if read(w, corpse, 0, |d| d.facial_hair_item_id) > 0 {
            let s = facial_hair_serial(w, corpse);
            p.u8(Layer::FacialHair.0 + 1);
            p.serial(s);
        }
    }
    p.u8(Layer::Invalid.0);
    let bytes = p.finish_with_length();
    w.send(ns, &bytes);
}

/// `SendCorpseContent(beholder, beheld)`.
pub fn send_corpse_content(w: &mut World, ns: NetStateId, beholder: MobileId, corpse: ItemId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let grid = w
        .net_state(ns)
        .is_some_and(|d| d.caps.container_grid_lines());
    let list = equip_items(w, corpse);
    let mut body: Vec<u8> = Vec::new();
    let mut written: u16 = 0;
    let entry = |body: &mut Vec<u8>,
                 serial: Serial,
                 graphic: i32,
                 amount: u16,
                 xy: (i16, i16),
                 hue: i32| {
        body.extend_from_slice(&serial.value().to_be_bytes());
        body.extend_from_slice(&(graphic as u16).to_be_bytes());
        body.push(0); // signed, itemID offset
        body.extend_from_slice(&amount.to_be_bytes());
        body.extend_from_slice(&xy.0.to_be_bytes());
        body.extend_from_slice(&xy.1.to_be_bytes());
        if grid {
            body.push(0); // Grid Location?
        }
        body.extend_from_slice(&corpse.0.value().to_be_bytes());
        body.extend_from_slice(&(hue as u16).to_be_bytes());
    };
    for child in list {
        let ok = w
            .item(child)
            .is_some_and(|c| !c.deleted() && c.parent == Some(EntityId::Item(corpse)))
            && deps::can_see(w, beholder, child);
        if ok && let Some(c) = w.item(child) {
            entry(
                &mut body,
                c.serial,
                c.item_id,
                c.amount as u16,
                (c.location.x as i16, c.location.y as i16),
                c.hue,
            );
            written += 1;
        }
    }
    if owner(w, corpse).is_some() {
        let (hi, hh, fi, fh) = read(w, corpse, (0, 0, 0, 0), |d| {
            (
                d.hair_item_id,
                d.hair_hue,
                d.facial_hair_item_id,
                d.facial_hair_hue,
            )
        });
        if hi > 0 {
            let s = hair_serial(w, corpse);
            entry(&mut body, s, hi, 1, (0, 0), hh);
            written += 1;
        }
        if fi > 0 {
            let s = facial_hair_serial(w, corpse);
            entry(&mut body, s, fi, 1, (0, 0), fh);
            written += 1;
        }
    }
    let mut out = Vec::with_capacity(5 + body.len());
    out.push(0x3C);
    out.extend_from_slice(&((5 + body.len()) as u16).to_be_bytes());
    out.extend_from_slice(&written.to_be_bytes());
    out.extend_from_slice(&body);
    w.send(ns, &out);
}

// ---- serialization ----

fn write_corpse(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    let d = data(w, id).cloned().unwrap_or_default();
    out.write_i32(SERIALIZATION_VERSION);

    let re = d.restore_equip.as_deref().unwrap_or(&[]);
    out.write_encoded_int(re.len() as i32);
    for i in re {
        out.write_item(w, Some(*i));
    }
    out.write_enum(4, i64::from(d.flags));
    out.write_anchored_time_ticks(d.time_of_death.ticks());

    let n = d.restore_table.as_ref().map_or(0, |t| t.len());
    out.write_encoded_int(n as i32);
    for (k, v) in d.restore_table.iter().flatten() {
        out.write_item(w, Some(*k));
        out.write_point3d(*v);
    }

    let next = if d.decay_timer == TimerHandle::default() {
        DateTime::MIN
    } else {
        d.decay_timer.next(w)
    };
    out.write_anchored_time_ticks(next.ticks());

    out.write_encoded_int(d.looters.len() as i32);
    for m in &d.looters {
        out.write_mobile(w, Some(*m));
    }
    out.write_mobile(w, d.killer);
    out.write_encoded_int(d.aggressors.len() as i32);
    for m in &d.aggressors {
        out.write_mobile(w, Some(*m));
    }
    out.write_mobile(w, d.owner);
    out.write_opt_string(d.corpse_name.as_deref());
    out.write_enum(4, d.access_level as i64);
    out.write_serial(d.guild.map_or(Serial::MINUS_ONE, |g| Serial::new(g.0)));
    out.write_encoded_int(d.equip_items.len() as i32);
    for i in &d.equip_items {
        out.write_item(w, Some(*i));
    }
    out.write_i32(d.hair_item_id);
    out.write_i32(d.hair_hue);
    out.write_i32(d.facial_hair_item_id);
    out.write_i32(d.facial_hair_hue);
}

fn read_guild(r: &mut dyn GenericReader) -> Result<Option<GuildId>> {
    let s = r.read_serial()?;
    Ok((s != Serial::MINUS_ONE && s != Serial::ZERO).then(|| GuildId(s.value())))
}

/// `_looters` is a set: the generated reader skips a null entry.
fn read_encoded_mobiles(w: &World, r: &mut dyn GenericReader) -> Result<Vec<MobileId>> {
    let n = r.read_encoded_int()?;
    let mut v = Vec::new();
    for _ in 0..n.max(0) {
        v.extend(r.read_mobile(w)?);
    }
    Ok(v)
}

fn access_from(v: i32) -> Result<AccessLevel> {
    AccessLevel::from_i32(v)
        .ok_or_else(|| IoError::InvalidData(format!("AccessLevel: no value {v}")))
}

fn to_date(ticks: i64) -> DateTime {
    DateTime::try_from_ticks(ticks).unwrap_or(DateTime::MIN)
}

/// Versions 13-19 share their order; what differs is the time encoding (delta before 18, TimeOfDeath until 19), what follows the
/// guild (`Kills` int in 13/14, `Murderer` bool in 15, nothing after) and how hair is stored (none in 13, the
/// legacy blocks in 14-16, four ints from 17).
/// `LegacyHairInfo`: a bool, then `[int version][int itemId][int hue]` when set. (item id, hue).
fn read_legacy_hair(r: &mut dyn GenericReader) -> Result<Option<(i32, i32)>> {
    if !r.read_bool()? {
        return Ok(None);
    }
    r.read_i32()?;
    let item_id = r.read_i32()?;
    Ok(Some((item_id, r.read_i32()?)))
}

fn read_versioned(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
    version: i32,
) -> Result<()> {
    let now = w.now;
    // The decay timer is anchored from version 18, TimeOfDeath only from 19 (V18Content still reads it as a delta).
    let time = |r: &mut dyn GenericReader, anchored: bool| -> Result<DateTime> {
        Ok(to_date(if anchored {
            r.read_anchored_time_ticks()?
        } else {
            r.read_delta_time_ticks(now.ticks())?
        }))
    };
    let mut d = CorpseData {
        restore_equip: Some(r.read_item_list_kept(true)?),
        flags: r.read_enum(4)? as u32,
        ..Default::default()
    };
    d.time_of_death = time(r, version >= 19)?;
    let n = r.read_encoded_int()?;
    let mut table = IndexMap::new();
    for _ in 0..n.max(0) {
        let k = r.read_item(w)?;
        let v = r.read_point3d()?;
        if let Some(k) = k {
            table.insert(k, v);
        }
    }
    d.restore_table = Some(table);
    let decay_next = time(r, version >= 18)?;
    let decay_delay = (decay_next != DateTime::MIN)
        .then(|| TimeSpan::from_ticks(decay_next.ticks() - now.ticks()));
    d.looters = read_encoded_mobiles(w, r)?.into_iter().collect();
    d.killer = r.read_mobile(w)?;
    d.aggressors = r.read_mobile_list_kept(true)?;
    d.owner = r.read_mobile(w)?;
    d.corpse_name = r.read_string()?;
    d.access_level = access_from(r.read_enum(4)? as i32)?;
    d.guild = read_guild(r)?;
    match version {
        13 | 14 => {
            if r.read_i32()? >= 5 {
                d.flags |= flag::MURDERER;
            }
        }
        15 if r.read_bool()? => {
            d.flags |= flag::MURDERER;
        }
        _ => {}
    }
    d.equip_items = r.read_item_list_kept(true)?;
    match version {
        13 => {}
        14..=16 => {
            if let Some((item_id, hue)) = read_legacy_hair(r)? {
                d.hair_item_id = item_id;
                d.hair_hue = hue;
            }
            if let Some((item_id, hue)) = read_legacy_hair(r)? {
                d.facial_hair_item_id = item_id;
                d.facial_hair_hue = hue;
            }
        }
        _ => {
            d.hair_item_id = r.read_i32()?;
            d.hair_hue = r.read_i32()?;
            d.facial_hair_item_id = r.read_i32()?;
            d.facial_hair_hue = r.read_i32()?;
        }
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    // 13-16 always built an InternalTimer (even for a MinValue delay); 17+ only when a timer was running.
    match (version, decay_delay) {
        (13..=16, delay) => begin_decay(w, id, delay.unwrap_or(TimeSpan::MIN)),
        (_, Some(delay)) => begin_decay(w, id, delay),
        _ => {}
    }
    Ok(())
}

/// `Deserialize(reader, version)` for versions below 13.
fn read_legacy(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    let now = w.now;
    let mut d = CorpseData::default();
    if r.read_bool()? {
        d.restore_equip = Some(r.read_item_list(w)?);
    }
    d.flags = r.read_i32()? as u32;
    d.time_of_death = to_date(r.read_delta_time_ticks(now.ticks())?);
    let count = r.read_i32()?;
    let mut table: Vec<(Option<ItemId>, Option<Point3D>)> = Vec::new();
    for _ in 0..count.max(0) {
        let item = r.read_item(w)?;
        if r.read_bool()? {
            table.push((item, Some(r.read_point3d()?)));
        } else {
            table.push((item, None));
        }
    }
    let decay = if r.read_bool()? {
        let t = to_date(r.read_delta_time_ticks(now.ticks())?);
        Some(TimeSpan::from_ticks(t.ticks() - now.ticks()))
    } else {
        None
    };
    d.looters = r.read_mobile_set(w)?;
    d.killer = r.read_mobile(w)?;
    d.aggressors = r.read_mobile_list(w)?;
    d.owner = r.read_mobile(w)?;
    d.corpse_name = r.read_string()?;
    d.access_level = access_from(r.read_i32()?)?;
    r.read_i32()?; // guild reserve
    if r.read_i32()? >= 5 {
        d.flags |= flag::MURDERER;
    }
    d.equip_items = r.read_item_list(w)?;
    for (item, loc) in table {
        let Some(item) = item else { continue };
        let at = loc.or_else(|| w.item(item).map(|i| i.location));
        if let Some(at) = at {
            d.restore_table
                .get_or_insert_with(IndexMap::new)
                .insert(item, at);
        }
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    if let Some(delay) = decay {
        begin_decay(w, id, delay);
    }
    Ok(())
}

fn read_corpse(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    let version = r.read_i32()?;
    if (13..=SERIALIZATION_VERSION).contains(&version) {
        return read_versioned(w, id, r, version);
    }
    if version < SERIALIZATION_VERSION {
        return read_legacy(w, id, r);
    }
    Err(IoError::InvalidData(format!(
        "Corpse version {version} is newer than {SERIALIZATION_VERSION}"
    )))
}

/// `Server.Items.Corpse`.
pub static CORPSE: ItemType = ItemType {
    full_name: "Server.Items.Corpse",
    parent: Some(&CONTAINER),
    construct: None,
    display_weight: |_, _| false,
    is_deco_container: |_, _| false,
    displays_content: |_, _| false,
    is_child_public,
    is_child_visible_to,
    check_item_use,
    check_lift,
    on_item_used,
    on_item_lifted,
    on_item_removed,
    get_context_menu_entries,
    on_double_click: |w, id, from| open(w, id, from, core::aos(w)),
    check_content_display: |_, _, _| false,
    add_name_property,
    on_aos_single_click,
    on_single_click,
    send_info_to,
    on_after_delete,
    serialize: |w, id, out| {
        container::serialize(w, id, out);
        write_corpse(w, id, out);
    },
    deserialize: |w, id, r| {
        container::deserialize(w, id, r)?;
        read_corpse(w, id, r)
    },
    ..CONTAINER
};

// ---- DecayedCorpse ----

/// What `DecayedCorpse` adds.
#[derive(Clone, Debug, Default)]
pub struct DecayedData {
    pub decay_timer: TimerHandle,
}

fn decayed_begin(w: &mut World, id: ItemId, delay: TimeSpan) {
    let Some(mut h) = w
        .item(id)
        .and_then(|i| i.ext.get::<DecayedData>())
        .map(|d| d.decay_timer)
    else {
        return;
    };
    h.cancel(w);
    let delay = if delay < TimeSpan::ZERO {
        TimeSpan::ZERO
    } else {
        delay
    };
    let nh = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
    });
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<DecayedData>()) {
        d.decay_timer = nh;
    }
}

/// `new DecayedCorpse(name)`.
pub fn new_decayed_corpse(w: &mut World, name: &str) -> ItemId {
    let g = utility::random_from(&mut w.rng, 0xECA, 9);
    let id = container::new_container(w, &DECAYED_CORPSE, g);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(DecayedData::default());
    }
    ops::set_movable(w, id, false);
    ops::set_name(w, id, Some(name.to_string()));
    decayed_begin(w, id, DEFAULT_DECAY_TIME);
    id
}

/// `Server.Items.DecayedCorpse`.
pub static DECAYED_CORPSE: ItemType = ItemType {
    full_name: "Server.Items.DecayedCorpse",
    parent: Some(&CONTAINER),
    construct: None,
    displays_content: |_, _| false,
    check_content_display: |_, _, _| false,
    add_name_property: |w, id, list| {
        let name = w.item(id).and_then(|i| i.name.clone()).unwrap_or_default();
        list.add_str(1046414, &name);
    },
    on_single_click: |w, id, from| {
        let name = w.item(id).and_then(|i| i.name.clone()).unwrap_or_default();
        ops::label_to_number(w, id, from, 1046414, &name); // the remains of ~1_NAME~
    },
    on_after_delete: |w, id| {
        if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<DecayedData>()) {
            let mut h = d.decay_timer;
            d.decay_timer = TimerHandle::default();
            h.cancel(w);
        }
    },
    serialize: |w, id, out| {
        container::serialize(w, id, out);
        out.write_i32(3);
        let next = w
            .item(id)
            .and_then(|i| i.ext.get::<DecayedData>())
            .filter(|d| d.decay_timer != TimerHandle::default())
            .map_or(DateTime::MIN, |d| d.decay_timer.next(w));
        out.write_anchored_time_ticks(next.ticks());
    },
    deserialize: |w, id, r| {
        container::deserialize(w, id, r)?;
        let version = r.read_i32()?;
        let now = w.now;
        let delay = if version == 2 {
            // MigrateFrom(V2Content): the delta time alone (no bool); DateTime.MinValue is no timer.
            let t = to_date(r.read_delta_time_ticks(now.ticks())?);
            (t != DateTime::MIN).then(|| TimeSpan::from_ticks(t.ticks() - now.ticks()))
        } else if version < 3 {
            // Deserialize(reader, version): a bool, then a delta time.
            if r.read_bool()? {
                let t = to_date(r.read_delta_time_ticks(now.ticks())?);
                Some(TimeSpan::from_ticks(t.ticks() - now.ticks()))
            } else {
                None
            }
        } else {
            let t = to_date(r.read_anchored_time_ticks()?);
            (t != DateTime::MIN).then(|| TimeSpan::from_ticks(t.ticks() - now.ticks()))
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(DecayedData::default());
        }
        if let Some(delay) = delay {
            decayed_begin(w, id, delay);
        }
        Ok(())
    },
    ..CONTAINER
};

/// `Corpse.Initialize`: `Mobile.CreateCorpseHandler += Mobile_CreateCorpseHandler`.
pub fn initialize(w: &mut World) {
    w.resources
        .get_mut::<muo_server::mobile::part2::statics::Part2Statics>()
        .create_corpse_handler = Some(create_corpse_handler);
}

type QuestCorpseFn =
    fn(&mut World, MobileId, (i32, i32), (i32, i32), Vec<ItemId>) -> Option<ItemId>;
/// The corpse classes of other crates: `Some` when `owner` has one of its own (the Haven `MilitiaFighterCorpse`).
#[derive(Default)]
pub struct CorpseClassHooks {
    pub quest_corpse: Option<QuestCorpseFn>,
}

/// `Mobile_CreateCorpseHandler(owner, initialContent, equipItems)`.
pub fn create_corpse_handler(
    w: &mut World,
    owner: MobileId,
    initial_content: &[ItemId],
    equip_items: &[ItemId],
) -> Option<ItemId> {
    let (hair, facial) = {
        let m = w.mobile(owner)?;
        (
            (m.hair_item_id, m.hair_hue),
            (m.facial_hair_item_id, m.facial_hair_hue),
        )
    };
    // `owner is MilitiaFighter ? new MilitiaFighterCorpse(..) : new Corpse(..)`
    let quest = w
        .resources
        .get::<CorpseClassHooks>()
        .and_then(|h| h.quest_corpse)
        .and_then(|f| f(w, owner, hair, facial, equip_items.to_vec()));
    let c = quest
        .unwrap_or_else(|| new_corpse_of(w, &CORPSE, owner, hair, facial, equip_items.to_vec()));
    if let Some(m) = w.mobile_mut(owner) {
        m.corpse = Some(c);
    }
    let player = w.mobile(owner).is_some_and(|m| m.player);
    let backpack = muo_server::mobile::part1::props_b::backpack(w, owner);
    for &item in initial_content {
        let parent_is_pack = w
            .item(item)
            .is_some_and(|i| i.parent.is_some() && i.parent == backpack.map(EntityId::Item));
        let Some(ty) = w.item(c).map(|i| i.ty) else {
            break;
        };
        if core::aos(w) && player && parent_is_pack {
            (ty.add_item)(w, c, item);
        } else {
            (ty.drop_item)(w, c, item);
        }
        if player && core::aos(w) {
            let loc = w.item(item).map_or(Point3D::new(0, 0, 0), |i| i.location);
            set_restore_info(w, c, item, loc);
        }
    }
    if core::se(w) && !player {
        assign_instanced_loot(w, c);
    } else if core::aos(w)
        && let Some(snap) = crate::player_mobile::data(w, owner).map(|d| d.equip_snapshot.clone())
        && let Some(d) = data_mut(w, c)
    {
        d.restore_equip = snap;
    }

    let (mut loc, mut map) = w.mobile(owner).map(|m| (m.location, m.map))?;
    if map.is_none_or(|m| m == MapId::INTERNAL) {
        let m = w.mobile(owner)?;
        loc = m.logout_location;
        map = m.logout_map;
    }
    ops::move_to_world_map(w, c, loc, map);
    Some(c)
}

pub fn register(types: &mut TypeRegistry) {
    for ty in [&CORPSE, &DECAYED_CORPSE] {
        types.register_item(ty, &[]);
    }
    // `Corpse : ICarvable`: the knife's `targeted is ICarvable` finds it (and every subclass) here.
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Corpse.ICarvable",
        |w| {
            crate::items::interfaces::register_carvable(w, &CORPSE, |w, id, from, knife| {
                carve(w, id, from, Some(knife))
            });
        },
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Initialize,
        "Server.Items.Corpse.Initialize",
        initialize,
    );
}

/// Corpse tests: child visibility, hair migration, loot and the corpse packets
/// (with the legacy `CorpseEquip`/`CorpseContent` packets written out as builders).
/// The worn weapon is a Katana where the tests use a VikingSword: both are `BaseSword`s.
#[cfg(test)]
mod behavior_tests {
    use super::*;
    use crate::items::leaves::KATANA;
    use muo_primitives::BodyTable;
    use muo_server::mobile::MOBILE_BASE;
    use muo_server::net_state::{NetOut, NetStateData};

    fn world() -> World {
        let mut w = World::new();
        let (table, invalid) = BodyTable::parse("400\tHuman\n201\tMonster\n");
        assert!(invalid.is_empty());
        w.resources.insert(table);
        w
    }

    fn new_owner(w: &mut World, body: i32) -> MobileId {
        let owner = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(owner).unwrap().body = Body(body);
        owner
    }

    /// `EquipSword`: weapons take their layer from tiledata, which the tests don't load.
    fn equip_sword(w: &mut World, owner: MobileId) -> ItemId {
        let sword = (KATANA.construct.unwrap())(w);
        (w.item(sword).unwrap().ty.set_layer)(w, sword, Layer::OneHanded);
        muo_server::mobile::part2::items::add_item(w, owner, sword);
        assert_eq!(w.item(sword).unwrap().parent, Some(EntityId::Mobile(owner)));
        sword
    }

    fn corpse_of(w: &mut World, owner: MobileId) -> ItemId {
        let items = w.mobile(owner).unwrap().items.clone();
        new_corpse(w, owner, items)
    }

    #[test]
    fn is_child_public_human_corpse_with_worn_gear_returns_true() {
        let mut w = world();
        let owner = new_owner(&mut w, 0x190);
        let worn = equip_sword(&mut w, owner);
        let corpse = corpse_of(&mut w, owner);
        assert_eq!(w.item(corpse).unwrap().item_id, 0x2006);
        assert!(body_is_human(&w, w.item(corpse).unwrap().amount));
        assert!(is_child_public(&w, corpse, worn));
    }

    #[test]
    fn is_child_public_human_corpse_with_loot_item_returns_false() {
        let mut w = world();
        let owner = new_owner(&mut w, 0x190);
        equip_sword(&mut w, owner);
        let corpse = corpse_of(&mut w, owner);
        let loot = crate::items::gold::new_gold(&mut w, 100);
        (w.item(corpse).unwrap().ty.drop_item)(&mut w, corpse, loot);
        assert_eq!(w.item(corpse).unwrap().item_id, 0x2006);
        assert!(body_is_human(&w, w.item(corpse).unwrap().amount));
        assert!(!is_child_public(&w, corpse, loot));
    }

    #[test]
    fn is_child_public_bone_corpse_returns_false() {
        let mut w = world();
        let owner = new_owner(&mut w, 0x190);
        let worn = equip_sword(&mut w, owner);
        let corpse = corpse_of(&mut w, owner);
        (w.item(corpse).unwrap().ty.set_item_id)(&mut w, corpse, 0xECA);
        assert!(body_is_human(&w, w.item(corpse).unwrap().amount));
        assert_ne!(w.item(corpse).unwrap().item_id, 0x2006);
        assert!(!is_child_public(&w, corpse, worn));
    }

    #[test]
    fn is_child_public_non_human_corpse_returns_false() {
        let mut w = world();
        let owner = new_owner(&mut w, 0xC9);
        let worn = equip_sword(&mut w, owner);
        let corpse = corpse_of(&mut w, owner);
        assert_eq!(w.item(corpse).unwrap().item_id, 0x2006);
        assert!(!body_is_human(&w, w.item(corpse).unwrap().amount));
        assert!(!is_child_public(&w, corpse, worn));
    }

    #[test]
    fn decayed_corpse_v2_reads_a_bare_delta_time() {
        let mut w = world();
        let src = new_decayed_corpse(&mut w, "x");
        let mut out = muo_io::SaveWriter::new(true);
        (w.item(src).unwrap().ty.serialize)(&w, src, &mut out);
        let mut bytes = out.into_vec();
        bytes.truncate(bytes.len() - 12); // version 3 + anchored time
        bytes.extend_from_slice(&2i32.to_le_bytes());
        bytes.extend_from_slice(&(60 * 10_000_000i64).to_le_bytes()); // a minute from now, no bool
        let dst = new_decayed_corpse(&mut w, "y");
        let mut r = muo_io::SaveReader::new(&bytes);
        (w.item(dst).unwrap().ty.deserialize)(&mut w, dst, &mut r).unwrap();
        assert_eq!(r.position() as usize, bytes.len());
        let d = w
            .item(dst)
            .unwrap()
            .ext
            .get::<DecayedData>()
            .unwrap()
            .decay_timer;
        assert_eq!(d.next(&w).ticks() - w.now.ticks(), 60 * 10_000_000);
    }

    /// The generated reader keeps a null entry of `_restoreEquip`, `_aggressors` and `_equipItems`, and the writer
    /// writes it as -1: a corpse whose referenced entity is gone keeps its counts and slots across a save.
    #[test]
    fn lists_keep_a_gone_entry_across_load_and_save() {
        use muo_io::GenericWriter;
        let mut w = world();
        let id = crate::items::gold::new_gold(&mut w, 1);
        let live = crate::items::gold::new_gold(&mut w, 1);
        let mob = new_owner(&mut w, 400);
        let (item_s, mob_s) = (w.item(live).unwrap().serial, w.mobile(mob).unwrap().serial);
        let mut out = muo_io::SaveWriter::new(true);
        out.write_i32(19);
        out.write_encoded_int(2); // restore equip: live, gone
        out.write_serial(item_s);
        out.write_serial(muo_primitives::Serial::MINUS_ONE);
        out.write_enum(4, 0);
        out.write_i64(0); // TimeOfDeath
        out.write_encoded_int(0); // restore table
        out.write_i64(0); // DecayTimerNext
        out.write_encoded_int(0); // looters
        out.write_u32(0); // killer
        out.write_encoded_int(3); // aggressors: gone, live, gone
        out.write_serial(muo_primitives::Serial::MINUS_ONE);
        out.write_serial(mob_s);
        out.write_serial(muo_primitives::Serial::new(0x3FFF_FFF0));
        out.write_u32(0); // owner
        out.write_string("");
        out.write_enum(4, 0); // access level
        out.write_u32(0); // guild
        out.write_encoded_int(2); // equip items: gone, live
        out.write_serial(muo_primitives::Serial::MINUS_ONE);
        out.write_serial(item_s);
        for _ in 0..4 {
            out.write_i32(0);
        }
        let bytes = out.into_vec();
        read_corpse(&mut w, id, &mut muo_io::SaveReader::new(&bytes)).unwrap();
        let d = data(&w, id).unwrap();
        assert_eq!(
            (
                d.restore_equip.as_ref().unwrap().len(),
                d.aggressors.len(),
                d.equip_items.len()
            ),
            (2, 3, 2)
        );

        // A deleted referent is written as -1 as well, and the counts stay.
        let ty = w.item(live).unwrap().ty;
        (ty.delete)(&mut w, live);
        let mut again = muo_io::SaveWriter::new(true);
        write_corpse(&w, id, &mut again);
        let saved = again.into_vec();
        let mut expected = muo_io::SaveWriter::new(true);
        expected.write_i32(19);
        expected.write_encoded_int(2);
        expected.write_serial(muo_primitives::Serial::MINUS_ONE);
        expected.write_serial(muo_primitives::Serial::MINUS_ONE);
        let expected = expected.into_vec();
        assert_eq!(
            &saved[..expected.len()],
            &expected[..],
            "restore equip: both entries, both -1"
        );
    }

    #[test]
    fn version_18_reads_time_of_death_as_a_delta() {
        use muo_io::GenericWriter;
        let mut w = world();
        w.now = DateTime::try_from_ticks(630_000_000_000_000_000).unwrap();
        let id = crate::items::gold::new_gold(&mut w, 1);
        let mut out = muo_io::SaveWriter::new(true);
        out.write_i32(18);
        out.write_encoded_int(0); // restore equip
        out.write_enum(4, 0); // flags
        out.write_i64(-60 * 10_000_000); // TimeOfDeath: a delta, a minute ago
        out.write_encoded_int(0); // restore table
        out.write_i64(0); // DecayTimerNext: anchored DateTime.MinValue
        out.write_encoded_int(0); // looters
        out.write_u32(0); // killer
        out.write_encoded_int(0); // aggressors
        out.write_u32(0); // owner
        out.write_string("");
        out.write_enum(4, 0); // access level
        out.write_u32(0); // guild
        out.write_encoded_int(0); // equip items
        for _ in 0..4 {
            out.write_i32(0);
        }
        let bytes = out.into_vec();
        let mut r = muo_io::SaveReader::new(&bytes);
        read_corpse(&mut w, id, &mut r).unwrap();
        assert_eq!(r.position() as usize, bytes.len());
        assert_eq!(
            time_of_death(&w, id).ticks(),
            w.now.ticks() - 60 * 10_000_000
        );
    }

    #[test]
    fn legacy_hair_tail_present_round_trips() {
        for (item_id, hue) in [(0x203B, 1102), (0x2049, 0)] {
            let mut out = muo_io::SaveWriter::new(true);
            out.write_bool(true);
            out.write_i32(0); // legacy VirtualHairInfo serialization version
            out.write_i32(item_id);
            out.write_i32(hue);
            let buffer = out.into_vec();
            let mut r = muo_io::SaveReader::new(&buffer);
            assert_eq!(read_legacy_hair(&mut r).unwrap(), Some((item_id, hue)));
            // The entire block must be consumed - the loader validates exact byte length.
            assert_eq!(r.position() as usize, buffer.len());
        }
    }

    #[test]
    fn lifted_worn_gear_put_back_stays_loot() {
        let mut w = world();
        let owner = new_owner(&mut w, 0x190);
        let sword = equip_sword(&mut w, owner);
        let corpse = new_corpse(&mut w, owner, vec![sword]);
        let drop = w.item(corpse).unwrap().ty.drop_item;
        drop(&mut w, corpse, sword);
        assert!(equip_items(&w, corpse).contains(&sword));

        on_item_lifted(&mut w, corpse, owner, sword);
        ops::internalize(&mut w, sword);
        drop(&mut w, corpse, sword);

        assert_eq!(w.item(sword).unwrap().parent, Some(EntityId::Item(corpse)));
        assert!(!equip_items(&w, corpse).contains(&sword));
    }

    // ---- the removal broadcasts of CorpseChildPublicTests ----

    /// `NewBystander(ns, location)`: a logged-in mobile with a net state, on Felucca of the map kit.
    fn bystander(w: &mut World, x: i32, y: i32) -> (NetStateId, MobileId) {
        let ns = NetStateId(w.net_states.len() as u32 + 1);
        let m = w.new_mobile(&MOBILE_BASE);
        muo_server::mobile::part2::stats::default_mobile_init(w, m);
        w.net_states.insert(
            ns,
            NetStateData {
                mobile: Some(m),
                ..Default::default()
            },
        );
        w.mobile_mut(m).unwrap().net_state = Some(ns);
        muo_server::mobile::part1::lifecycle::move_to_world(
            w,
            m,
            Point3D::new(x, y, 0),
            MapId::FELUCCA,
        );
        (ns, m)
    }

    /// `ReceivedRemove(ns, serial)`: the queued bytes of the connection carry a RemoveEntity of `serial`.
    fn received_remove(w: &World, ns: NetStateId, serial: Serial) -> bool {
        let needle = muo_server::packets::entity::create_remove_entity(serial);
        let mut bytes = Vec::new();
        for o in &w.net_out {
            if let NetOut::Send(n, b) = o
                && *n == ns
            {
                bytes.extend_from_slice(b);
            }
        }
        bytes.windows(needle.len()).any(|c| c == needle.as_slice())
    }

    fn map_world() -> World {
        let mut w = muo_server::map::test_support::world();
        let (table, invalid) = BodyTable::parse(
            "400	Human
201	Monster
",
        );
        assert!(invalid.is_empty());
        w.resources.insert(table);
        w
    }

    #[test]
    fn human_corpse_worn_gear_removal_broadcasts_loot_removal_does_not() {
        let mut w = map_world();
        let corpse_loc = Point3D::new(1500, 1500, 0);
        let (ns, _) = bystander(&mut w, corpse_loc.x + 1, corpse_loc.y);
        let owner = new_owner(&mut w, 0x190);

        let sword = (KATANA.construct.unwrap())(&mut w);
        let corpse = new_corpse(&mut w, owner, vec![sword]);
        muo_server::item::base::add_item(&mut w, corpse, sword);

        let loot = crate::items::gold::new_gold(&mut w, 100);
        let drop = w.item(corpse).unwrap().ty.drop_item;
        drop(&mut w, corpse, loot);

        ops::move_to_world_map(&mut w, corpse, corpse_loc, Some(MapId::FELUCCA));

        // A container not in the world is a valid destination; only the removal from the corpse matters here.
        let pouch = container::new_container(&mut w, &CONTAINER, 0xE75);
        muo_server::item::base::add_item(&mut w, pouch, sword);

        assert!(received_remove(&w, ns, w.item(sword).unwrap().serial));

        let loot_serial = w.item(loot).unwrap().serial;
        muo_server::item::base::delete(&mut w, loot);
        assert!(!received_remove(&w, ns, loot_serial));
    }

    #[test]
    fn human_corpse_lifting_worn_gear_broadcasts_removal() {
        let mut w = map_world();
        let corpse_loc = Point3D::new(1500, 1500, 0);
        let (ns, _) = bystander(&mut w, corpse_loc.x + 1, corpse_loc.y);
        let owner = new_owner(&mut w, 0x190);
        let sword = (KATANA.construct.unwrap())(&mut w);
        (w.item(sword).unwrap().ty.set_layer)(&mut w, sword, Layer::OneHanded);

        muo_server::mobile::part1::lifecycle::move_to_world(
            &mut w,
            owner,
            Point3D::new(corpse_loc.x - 1, corpse_loc.y, corpse_loc.z),
            MapId::FELUCCA,
        );
        let corpse = new_corpse(&mut w, owner, vec![sword]);
        muo_server::item::base::add_item(&mut w, corpse, sword);
        ops::move_to_world_map(&mut w, corpse, corpse_loc, Some(MapId::FELUCCA));

        // Lifting takes the piece off the worn list before the remove is sent; the remove must
        // still reach everyone who was shown it worn.
        let amount = w.item(sword).unwrap().amount;
        let (rejected, _) = muo_server::mobile::base::lift(&mut w, owner, sword, amount);

        assert!(!rejected);
        assert!(!equip_items(&w, corpse).contains(&sword));
        assert!(received_remove(&w, ns, w.item(sword).unwrap().serial));
    }

    // ---- CorpsePacketTests ----

    fn sent(w: &mut World) -> Vec<u8> {
        let mut all: Vec<Vec<u8>> = std::mem::take(&mut w.net_out)
            .into_iter()
            .filter_map(|o| match o {
                NetOut::Send(_, b) => Some(b),
                _ => None,
            })
            .collect();
        assert_eq!(all.len(), 1);
        all.remove(0)
    }

    #[test]
    fn test_corpse_equip_packet() {
        let mut w = world();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::new(0));
        let m = new_owner(&mut w, 0x190);
        let weapon = equip_sword(&mut w, m);
        let c = corpse_of(&mut w, m);

        // CorpseEquip: only items parented to the corpse are listed; the sword still hangs on the mobile.
        let mut expected = vec![0x89, 0, 0];
        expected.extend_from_slice(&c.0.value().to_be_bytes());
        let list = equip_items(&w, c);
        assert_eq!(list, vec![weapon]);
        for item in list {
            let ok = w
                .item(item)
                .is_some_and(|i| !i.deleted() && i.parent == Some(EntityId::Item(c)));
            if ok && deps::can_see(&mut w, m, item) {
                expected.push(w.item(item).unwrap().layer.0 + 1);
                expected.extend_from_slice(&item.0.value().to_be_bytes());
            }
        }
        expected.push(Layer::Invalid.0);
        let n = expected.len() as u16;
        expected[1..3].copy_from_slice(&n.to_be_bytes());

        send_corpse_equip(&mut w, ns, m, c);
        assert_eq!(sent(&mut w), expected);
    }

    #[test]
    fn test_corpse_container_packet() {
        for grid in [false, true] {
            let mut w = world();
            let ns = NetStateId(1);
            w.net_states.insert(ns, NetStateData::new(0));
            if grid {
                w.net_states.get_mut(&ns).unwrap().caps.protocol_changes =
                    muo_net::protocol_changes::ProtocolChanges::CONTAINER_GRID_LINES;
            }
            let m = new_owner(&mut w, 0x190);
            equip_sword(&mut w, m);
            let c = corpse_of(&mut w, m);

            // CorpseContent / CorpseContent6017
            let mut body = Vec::new();
            let mut written = 0u16;
            for child in equip_items(&w, c) {
                let ok = w
                    .item(child)
                    .is_some_and(|i| !i.deleted() && i.parent == Some(EntityId::Item(c)));
                if ok && deps::can_see(&mut w, m, child) {
                    let i = w.item(child).unwrap();
                    body.extend_from_slice(&i.serial.value().to_be_bytes());
                    body.extend_from_slice(&(i.item_id as u16).to_be_bytes());
                    body.push(0);
                    body.extend_from_slice(&(i.amount as u16).to_be_bytes());
                    body.extend_from_slice(&(i.location.x as i16).to_be_bytes());
                    body.extend_from_slice(&(i.location.y as i16).to_be_bytes());
                    if grid {
                        body.push(0);
                    }
                    body.extend_from_slice(&c.0.value().to_be_bytes());
                    body.extend_from_slice(&(i.hue as u16).to_be_bytes());
                    written += 1;
                }
            }
            let mut expected = vec![0x3C];
            expected.extend_from_slice(&((5 + body.len()) as u16).to_be_bytes());
            expected.extend_from_slice(&written.to_be_bytes());
            expected.extend_from_slice(&body);

            send_corpse_content(&mut w, ns, m, c);
            assert_eq!(sent(&mut w), expected, "grid {grid}");
        }
    }

    // ---- Misc/CorpseNotorietyTests ----

    #[test]
    fn player_corpse_notoriety_ignores_transformed_body() {
        for (body_mod, expected, kills) in [
            (0, noto::INNOCENT, 0),
            (746, noto::INNOCENT, 0), // Horrific Beast, a monster body
            (0, noto::MURDERER, 5),
            (746, noto::MURDERER, 5),
        ] {
            let mut w = world();
            let victim = crate::player_mobile::new_player_mobile(&mut w);
            let looter = crate::player_mobile::new_player_mobile(&mut w);
            {
                let m = w.mobile_mut(victim).unwrap();
                m.kills = kills;
                m.body_mod = Body(body_mod);
            }
            let corpse = new_corpse(&mut w, victim, vec![]);
            assert_eq!(
                corpse_notoriety(&w, looter, corpse),
                expected,
                "bodyMod {body_mod} kills {kills}"
            );
        }
    }
    #[test]
    fn a_killer_that_is_no_devourer_does_not_devour() {
        let mut w = world();
        super::super::common::hooks_mut(&mut w).devour = Some(|_, _, _| true);
        let owner = new_owner(&mut w, 0x190);
        let killer = new_owner(&mut w, 0x190);
        let corpse = corpse_of(&mut w, owner);
        data_mut(&mut w, corpse).unwrap().killer = Some(killer);
        assert!(!devour_corpse(&mut w, corpse));
        assert!(!devoured(&w, corpse));
    }
}
