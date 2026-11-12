//! BaseCreature's serialization: version 24 as `[SerializationGenerator(24, false)]` generated it (a `ulong`
//! `SaveFlag` mask, then each present field in index order), the version 23 migration (`V23Content`) and the
//! hand-written pre-codegen reader (versions 0-22). The base `Mobile` part is written/read first.

use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{DateTime, Point3D};
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::MOBILE_BASE;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer;
use muo_server::world::World;

use super::data::{CreatureData, cd, cd_mut};
use super::enums::*;
use super::speeds;
use super::vtable::ctype;

/// The `SaveFlag` bit positions of version 24, in field order.
const V24: [&str; 52] = [
    "CurrentAI",
    "RangePerception",
    "RangeFight",
    "RangeHome",
    "Team",
    "FightMode",
    "ActiveSpeed",
    "PassiveSpeed",
    "CurrentSpeed",
    "ActiveMoveSpeed",
    "PassiveMoveSpeed",
    "Home",
    "HomeMap",
    "Controlled",
    "Master",
    "ControlTarget",
    "ControlDest",
    "ControlOrder",
    "MinTameSkill",
    "Tamable",
    "Summoned",
    "SummonEnd",
    "ControlSlots",
    "Loyalty",
    "CurrentWayPoint",
    "HitsMaxSeed",
    "StamMaxSeed",
    "ManaMaxSeed",
    "DamageMin",
    "DamageMax",
    "PhysicalResistanceSeed",
    "FireResistSeed",
    "ColdResistSeed",
    "PoisonResistSeed",
    "EnergyResistSeed",
    "PhysicalDamage",
    "FireDamage",
    "ColdDamage",
    "PoisonDamage",
    "EnergyDamage",
    "Owners",
    "IsDeadPet",
    "IsBonded",
    "BondingBegin",
    "OwnerAbandonTime",
    "HasGeneratedLoot",
    "IsParagon",
    "Friends",
    "RemoveIfUntamed",
    "RemoveStep",
    "PendingDeleteTimer",
    "CorpseNameOverride",
];

/// Version 23's order: `ControlMaster` where 24 has `Master`, and a `SummonMaster` after `SummonEnd`.
const V23: [&str; 53] = [
    "CurrentAI",
    "RangePerception",
    "RangeFight",
    "RangeHome",
    "Team",
    "FightMode",
    "ActiveSpeed",
    "PassiveSpeed",
    "CurrentSpeed",
    "ActiveMoveSpeed",
    "PassiveMoveSpeed",
    "Home",
    "HomeMap",
    "Controlled",
    "Master",
    "ControlTarget",
    "ControlDest",
    "ControlOrder",
    "MinTameSkill",
    "Tamable",
    "Summoned",
    "SummonEnd",
    "SummonMaster",
    "ControlSlots",
    "Loyalty",
    "CurrentWayPoint",
    "HitsMaxSeed",
    "StamMaxSeed",
    "ManaMaxSeed",
    "DamageMin",
    "DamageMax",
    "PhysicalResistanceSeed",
    "FireResistSeed",
    "ColdResistSeed",
    "PoisonResistSeed",
    "EnergyResistSeed",
    "PhysicalDamage",
    "FireDamage",
    "ColdDamage",
    "PoisonDamage",
    "EnergyDamage",
    "Owners",
    "IsDeadPet",
    "IsBonded",
    "BondingBegin",
    "OwnerAbandonTime",
    "HasGeneratedLoot",
    "IsParagon",
    "Friends",
    "RemoveIfUntamed",
    "RemoveStep",
    "PendingDeleteTimer",
    "CorpseNameOverride",
];

fn bit(order: &[&str], name: &str) -> u64 {
    order
        .iter()
        .position(|n| *n == name)
        .map_or(0, |i| 1u64 << i)
}

/// `ReadEntity<Mobile>()` into `_owners` / `_friends`: the generated reader adds the result even when it is null (a
/// mobile that no longer exists), so the list keeps its count until `Tidy` at the next save. A serial that doesn't
/// resolve is that null: a `MobileId` nothing answers to.
fn read_kept_mobile(r: &mut dyn GenericReader) -> muo_io::Result<MobileId> {
    Ok(MobileId(r.read_serial()?))
}

fn tidy(w: &World, list: &[MobileId]) -> Vec<MobileId> {
    list.iter()
        .copied()
        .filter(|&m| w.mobile(m).is_some_and(|m| !m.deleted))
        .collect()
}

/// Version 24 `ShouldSerialize*` predicates, in flag order.
fn flags_of(w: &World, id: MobileId, d: &CreatureData) -> u64 {
    let (ta, tp) = speeds::speeds_of(w, id);
    let (tam, tpm) = speeds::move_speeds_of(w, id);
    let pending =
        d.pending_delete_timer.running(w) && !d.is_stabled && !(d.controlled && d.master.is_some());
    let checks = [
        d.current_ai != d.default_ai,
        d.range_perception != DEFAULT_RANGE_PERCEPTION,
        d.range_fight != 1,
        d.range_home != 10,
        d.team != 0,
        d.fight_mode != FightMode::Closest,
        d.active_speed != ta,
        d.passive_speed != tp,
        d.current_speed != d.passive_speed,
        d.active_move_speed != tam,
        d.passive_move_speed != tpm,
        d.home != Point3D::ZERO,
        d.home_map.is_some(),
        d.controlled,
        d.master.is_some(),
        d.control_target.is_some(),
        d.control_dest != Point3D::ZERO,
        d.control_order != OrderType::None,
        d.min_tame_skill != 0.0,
        d.tamable_raw,
        d.summoned,
        d.summoned, // ShouldSerializeSummonEnd() => _summoned
        d.control_slots != 1,
        d.loyalty != MAX_LOYALTY,
        d.current_way_point.is_some(),
        d.hits_max_seed != -1,
        d.stam_max_seed != -1,
        d.mana_max_seed != -1,
        d.damage_min != -1,
        d.damage_max != -1,
        d.physical_resistance_seed != 0,
        d.fire_resist_seed != 0,
        d.cold_resist_seed != 0,
        d.poison_resist_seed != 0,
        d.energy_resist_seed != 0,
        d.physical_damage != 100,
        d.fire_damage != 0,
        d.cold_damage != 0,
        d.poison_damage != 0,
        d.energy_damage != 0,
        !tidy(w, &d.owners).is_empty(),
        d.is_dead_pet,
        d.is_bonded,
        d.bonding_begin != DateTime::MIN,
        d.owner_abandon_time != DateTime::MIN,
        d.has_generated_loot,
        d.is_paragon,
        d.friends.as_ref().is_some_and(|f| !tidy(w, f).is_empty()),
        d.remove_if_untamed,
        d.remove_step != 0,
        pending,
        d.corpse_name_override.is_some(),
    ];
    checks.iter().enumerate().fold(
        0u64,
        |acc, (i, &on)| if on { acc | (1u64 << i) } else { acc },
    )
}

/// `Serialize` (version 24).
pub fn serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    (MOBILE_BASE.serialize)(w, id, out);
    let Some(d) = cd(w, id) else { return };
    out.write_i32(24);
    let flags = flags_of(w, id, d);
    out.write_u64(flags);
    let has = |n: &str| flags & bit(&V24, n) != 0;

    out.write_encoded_int(d.default_ai as i32);
    if has("CurrentAI") {
        out.write_encoded_int(d.current_ai as i32);
    }
    if has("RangePerception") {
        out.write_encoded_int(d.range_perception);
    }
    if has("RangeFight") {
        out.write_encoded_int(d.range_fight);
    }
    if has("RangeHome") {
        out.write_encoded_int(d.range_home);
    }
    if has("Team") {
        out.write_encoded_int(d.team);
    }
    if has("FightMode") {
        out.write_encoded_int(d.fight_mode as i32);
    }
    if has("ActiveSpeed") {
        out.write_f64(d.active_speed);
    }
    if has("PassiveSpeed") {
        out.write_f64(d.passive_speed);
    }
    if has("CurrentSpeed") {
        out.write_f64(d.current_speed);
    }
    if has("ActiveMoveSpeed") {
        out.write_f64(d.active_move_speed);
    }
    if has("PassiveMoveSpeed") {
        out.write_f64(d.passive_move_speed);
    }
    if has("Home") {
        out.write_point3d(d.home);
    }
    if has("HomeMap") {
        out.write_map(d.home_map);
    }
    if has("Master") {
        out.write_mobile(w, d.master);
    }
    if has("ControlTarget") {
        out.write_mobile(w, d.control_target);
    }
    if has("ControlDest") {
        out.write_point3d(d.control_dest);
    }
    if has("ControlOrder") {
        out.write_encoded_int(d.control_order as i32);
    }
    if has("MinTameSkill") {
        out.write_f64(d.min_tame_skill);
    }
    if has("SummonEnd") {
        out.write_anchored_time_ticks(d.summon_end.ticks());
    }
    if has("ControlSlots") {
        out.write_encoded_int(d.control_slots);
    }
    if has("Loyalty") {
        out.write_encoded_int(d.loyalty);
    }
    if has("CurrentWayPoint") {
        out.write_item(w, d.current_way_point);
    }
    for (name, v) in [
        ("HitsMaxSeed", d.hits_max_seed),
        ("StamMaxSeed", d.stam_max_seed),
        ("ManaMaxSeed", d.mana_max_seed),
        ("DamageMin", d.damage_min),
        ("DamageMax", d.damage_max),
        ("PhysicalResistanceSeed", d.physical_resistance_seed),
        ("FireResistSeed", d.fire_resist_seed),
        ("ColdResistSeed", d.cold_resist_seed),
        ("PoisonResistSeed", d.poison_resist_seed),
        ("EnergyResistSeed", d.energy_resist_seed),
        ("PhysicalDamage", d.physical_damage),
        ("FireDamage", d.fire_damage),
        ("ColdDamage", d.cold_damage),
        ("PoisonDamage", d.poison_damage),
        ("EnergyDamage", d.energy_damage),
    ] {
        if has(name) {
            out.write_encoded_int(v);
        }
    }
    if has("Owners") {
        let list = tidy(w, &d.owners);
        out.write_encoded_int(list.len() as i32);
        for m in list {
            out.write_mobile(w, Some(m));
        }
    }
    if has("BondingBegin") {
        out.write_date_time_ticks(d.bonding_begin.ticks());
    }
    if has("OwnerAbandonTime") {
        out.write_date_time_ticks(d.owner_abandon_time.ticks());
    }
    if has("Friends") {
        let list = d.friends.as_deref().map(|f| tidy(w, f)).unwrap_or_default();
        out.write_encoded_int(list.len() as i32);
        for m in list {
            out.write_mobile(w, Some(m));
        }
    }
    if has("RemoveStep") {
        out.write_encoded_int(d.remove_step);
    }
    if has("PendingDeleteTimer") {
        out.write_anchored_time_ticks(d.pending_delete_timer.next(w).ticks());
    }
    if has("CorpseNameOverride") {
        out.write_opt_string(d.corpse_name_override.as_deref());
    }
}

fn date(ticks: i64) -> DateTime {
    DateTime::try_from_ticks(ticks).unwrap_or(DateTime::MIN)
}

/// `Deserialize`: the base `Mobile` part, the version, then the version's layout, then `AfterDeserialization`.
pub fn deserialize(w: &mut World, id: MobileId, r: &mut dyn GenericReader) -> Result<()> {
    (MOBILE_BASE.deserialize)(w, id, r)?;
    let tick = w.tick_count;
    if let Some(m) = w.mobile_mut(id)
        && m.ext.get::<CreatureData>().is_none()
    {
        m.ext = muo_server::Ext::new(CreatureData::new(tick));
    }
    let version = r.read_i32()?;
    if version == 23 {
        read_flagged(w, id, r, &V23)?;
    } else if version < 24 {
        read_legacy(w, id, r, version)?;
    } else {
        read_flagged(w, id, r, &V24)?;
    }
    after_deserialization(w, id);
    Ok(())
}

/// The `SaveFlag` reader of version 24, and (through `order`) the `V23Content` reader plus `MigrateFrom`.
fn read_flagged(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
    order: &[&str],
) -> Result<()> {
    let flags = r.read_u64()?;
    let has = |n: &str| flags & bit(order, n) != 0;
    let (ta, tp) = speeds::speeds_of(w, id);
    let (tam, tpm) = speeds::move_speeds_of(w, id);
    let mut d = CreatureData::new(w.tick_count);

    d.default_ai = AIType::from_i32(r.read_encoded_int()?).unwrap_or_default();
    d.current_ai = if has("CurrentAI") {
        AIType::from_i32(r.read_encoded_int()?).unwrap_or_default()
    } else {
        d.default_ai
    };
    d.range_perception = if has("RangePerception") {
        r.read_encoded_int()?
    } else {
        DEFAULT_RANGE_PERCEPTION
    };
    d.range_fight = if has("RangeFight") {
        r.read_encoded_int()?
    } else {
        1
    };
    d.range_home = if has("RangeHome") {
        r.read_encoded_int()?
    } else {
        10
    };
    d.team = if has("Team") {
        r.read_encoded_int()?
    } else {
        0
    };
    d.fight_mode = if has("FightMode") {
        FightMode::from_i32(r.read_encoded_int()?).unwrap_or_default()
    } else {
        FightMode::Closest
    };
    d.active_speed = if has("ActiveSpeed") {
        r.read_f64()?
    } else {
        ta
    };
    d.passive_speed = if has("PassiveSpeed") {
        r.read_f64()?
    } else {
        tp
    };
    d.current_speed = if has("CurrentSpeed") {
        r.read_f64()?
    } else {
        d.passive_speed
    };
    d.active_move_speed = if has("ActiveMoveSpeed") {
        r.read_f64()?
    } else {
        tam
    };
    d.passive_move_speed = if has("PassiveMoveSpeed") {
        r.read_f64()?
    } else {
        tpm
    };
    if has("Home") {
        d.home = r.read_point3d()?;
    }
    if has("HomeMap") {
        d.home_map = ReadEntityExt::read_map(r)?;
    }
    d.controlled = has("Controlled");
    let mut master = None;
    if has("Master") {
        master = r.read_mobile(w)?;
    }
    if has("ControlTarget") {
        d.control_target = r.read_mobile(w)?;
    }
    if has("ControlDest") {
        d.control_dest = r.read_point3d()?;
    }
    if has("ControlOrder") {
        d.control_order = OrderType::from_i32(r.read_encoded_int()?).unwrap_or_default();
    }
    if has("MinTameSkill") {
        d.min_tame_skill = r.read_f64()?;
    }
    d.tamable_raw = has("Tamable");
    d.summoned = has("Summoned");
    if has("SummonEnd") {
        d.summon_end = date(r.read_anchored_time_ticks()?);
    }
    let mut summon_master = None;
    if has("SummonMaster") {
        summon_master = r.read_mobile(w)?;
    }
    // A traded summon could still carry its caster as summon master; the owner wins.
    d.master = master.or(summon_master);
    d.control_slots = if has("ControlSlots") {
        r.read_encoded_int()?
    } else {
        1
    };
    d.loyalty = if has("Loyalty") {
        r.read_encoded_int()?
    } else {
        MAX_LOYALTY
    };
    if has("CurrentWayPoint") {
        d.current_way_point = r.read_item(w)?;
    }
    d.hits_max_seed = if has("HitsMaxSeed") {
        r.read_encoded_int()?
    } else {
        -1
    };
    d.stam_max_seed = if has("StamMaxSeed") {
        r.read_encoded_int()?
    } else {
        -1
    };
    d.mana_max_seed = if has("ManaMaxSeed") {
        r.read_encoded_int()?
    } else {
        -1
    };
    d.damage_min = if has("DamageMin") {
        r.read_encoded_int()?
    } else {
        -1
    };
    d.damage_max = if has("DamageMax") {
        r.read_encoded_int()?
    } else {
        -1
    };
    for (name, slot) in [
        ("PhysicalResistanceSeed", &mut d.physical_resistance_seed),
        ("FireResistSeed", &mut d.fire_resist_seed),
        ("ColdResistSeed", &mut d.cold_resist_seed),
        ("PoisonResistSeed", &mut d.poison_resist_seed),
        ("EnergyResistSeed", &mut d.energy_resist_seed),
    ] {
        if has(name) {
            *slot = r.read_encoded_int()?;
        }
    }
    d.physical_damage = if has("PhysicalDamage") {
        r.read_encoded_int()?
    } else {
        100
    };
    for (name, slot) in [
        ("FireDamage", &mut d.fire_damage),
        ("ColdDamage", &mut d.cold_damage),
        ("PoisonDamage", &mut d.poison_damage),
        ("EnergyDamage", &mut d.energy_damage),
    ] {
        if has(name) {
            *slot = r.read_encoded_int()?;
        }
    }
    if has("Owners") {
        let n = r.read_encoded_int()?;
        for _ in 0..n {
            d.owners.push(read_kept_mobile(r)?);
        }
    }
    d.is_dead_pet = has("IsDeadPet");
    d.is_bonded = has("IsBonded");
    if has("BondingBegin") {
        d.bonding_begin = date(r.read_date_time_ticks()?);
    }
    if has("OwnerAbandonTime") {
        d.owner_abandon_time = date(r.read_date_time_ticks()?);
    }
    d.has_generated_loot = has("HasGeneratedLoot");
    d.is_paragon = has("IsParagon");
    if has("Friends") {
        let n = r.read_encoded_int()?;
        let mut list = Vec::new();
        for _ in 0..n {
            list.push(read_kept_mobile(r)?);
        }
        d.friends = Some(list);
    }
    d.remove_if_untamed = has("RemoveIfUntamed");
    if has("RemoveStep") {
        d.remove_step = r.read_encoded_int()?;
    }
    let mut pending: Option<DateTime> = None;
    if has("PendingDeleteTimer") {
        let next = date(r.read_anchored_time_ticks()?);
        if next != DateTime::MIN {
            pending = Some(next);
        }
    }
    if has("CorpseNameOverride") {
        d.corpse_name_override = r.read_string()?;
    }
    install(w, id, d);
    if let Some(next) = pending {
        let delay = next.since(w.now);
        super::misc::start_delete_timer(w, id, delay);
    }
    Ok(())
}

fn install(w: &mut World, id: MobileId, mut d: CreatureData) {
    if let Some(old) = cd(w, id) {
        // Keep the live (non-serialized) state a constructor may have set; the loader constructs fresh ones.
        d.seeks_home = old.seeks_home;
    }
    if let Some(m) = w.mobile_mut(id) {
        m.ext = muo_server::Ext::new(d);
    }
}

/// Pre-codegen reader (`Deserialize(IGenericReader, int version)`), versions 0-22.
fn read_legacy(w: &mut World, id: MobileId, r: &mut dyn GenericReader, version: i32) -> Result<()> {
    let mut d = CreatureData::new(w.tick_count);
    let mut control_master: Option<MobileId> = None;

    d.current_ai = AIType::from_i32(r.read_i32()?).unwrap_or_default();
    d.default_ai = AIType::from_i32(r.read_i32()?).unwrap_or_default();
    d.range_perception = r.read_i32()?;
    d.range_fight = r.read_i32()?;
    d.team = r.read_i32()?;
    d.active_speed = r.read_f64()?;
    d.passive_speed = r.read_f64()?;
    d.current_speed = r.read_f64()?;
    d.home = Point3D::new(r.read_i32()?, r.read_i32()?, r.read_i32()?);

    if version >= 1 {
        d.range_home = r.read_i32()?;
        if version < 20 {
            for _ in 0..2 {
                // Spell attacks, then spell defenses: a count of type names.
                let n = r.read_i32()?;
                for _ in 0..n {
                    r.read_string()?;
                }
            }
        }
    } else {
        d.range_home = 0;
    }

    if version >= 2 {
        d.fight_mode = FightMode::from_i32(r.read_i32()?).unwrap_or_default();
        d.controlled = r.read_bool()?;
        control_master = r.read_mobile(w)?;
        d.control_target = r.read_mobile(w)?;
        d.control_dest = r.read_point3d()?;
        d.control_order = OrderType::from_i32(r.read_i32()?).unwrap_or_default();
        d.min_tame_skill = r.read_f64()?;
        if version < 9 {
            r.read_f64()?;
        }
        d.tamable_raw = r.read_bool()?;
        d.summoned = r.read_bool()?;
        if d.summoned {
            // The UnsummonTimer is restarted in AfterDeserialization.
            let ticks = if version >= 21 {
                r.read_anchored_time_ticks()?
            } else {
                r.read_delta_time_ticks(DateTime::utc_now().ticks())?
            };
            d.summon_end = date(ticks);
        }
        d.control_slots = r.read_i32()?;
    } else {
        d.fight_mode = FightMode::Closest;
        d.controlled = false;
        d.control_target = None;
        d.control_order = OrderType::None;
    }

    d.loyalty = if version >= 3 {
        r.read_i32()?
    } else {
        MAX_LOYALTY
    };
    if version >= 4 {
        d.current_way_point = r.read_item(w)?;
    }
    let mut master = None;
    if version >= 5 {
        master = r.read_mobile(w)?;
    }
    // A transferred summon kept its caster as summon master; the owner wins.
    d.master = control_master.or(master);

    if version >= 6 {
        d.hits_max_seed = r.read_i32()?;
        d.stam_max_seed = r.read_i32()?;
        d.mana_max_seed = r.read_i32()?;
        d.damage_min = r.read_i32()?;
        d.damage_max = r.read_i32()?;
    }
    if version >= 7 {
        d.physical_resistance_seed = r.read_i32()?;
        d.physical_damage = r.read_i32()?;
        d.fire_resist_seed = r.read_i32()?;
        d.fire_damage = r.read_i32()?;
        d.cold_resist_seed = r.read_i32()?;
        d.cold_damage = r.read_i32()?;
        d.poison_resist_seed = r.read_i32()?;
        d.poison_damage = r.read_i32()?;
        d.energy_resist_seed = r.read_i32()?;
        d.energy_damage = r.read_i32()?;
    }
    if version >= 8 {
        d.owners = r.read_mobile_list(w)?;
    }
    if version >= 10 {
        d.is_dead_pet = r.read_bool()?;
        d.is_bonded = r.read_bool()?;
        d.bonding_begin = date(r.read_date_time_ticks()?);
        d.owner_abandon_time = date(r.read_date_time_ticks()?);
    }
    d.has_generated_loot = version < 11 || r.read_bool()?;
    d.is_paragon = version >= 12 && r.read_bool()?;
    if version >= 13 && r.read_bool()? {
        d.friends = Some(r.read_mobile_list(w)?);
    } else if version < 13 && d.control_order >= OrderType::Unfriend {
        d.control_order =
            OrderType::from_i32(d.control_order as i32 + 1).unwrap_or(d.control_order);
    }
    if version < 16 && d.loyalty != MAX_LOYALTY {
        d.loyalty *= 10;
    }
    if version >= 14 {
        d.remove_if_untamed = r.read_bool()?;
        d.remove_step = r.read_i32()?;
    }
    let mut delete_time = muo_primitives::TimeSpan::ZERO;
    if version >= 17 {
        delete_time = muo_primitives::TimeSpan::from_ticks(r.read_time_span_ticks()?);
    }
    let last_owner = d.owners.last().copied();
    let abandoned = last_owner.is_some() && !d.controlled; // && !IsStabled (never serialized: false)
    if delete_time > muo_primitives::TimeSpan::ZERO || abandoned {
        if delete_time == muo_primitives::TimeSpan::ZERO {
            delete_time = muo_primitives::TimeSpan::from_days(3);
        }
    } else {
        delete_time = muo_primitives::TimeSpan::ZERO;
    }
    if version >= 18 {
        d.corpse_name_override = r.read_string()?;
    }
    if version >= 19 {
        d.home_map = ReadEntityExt::read_map(r)?;
    }
    install(w, id, d);
    if version >= 22 {
        let (a, p) = (r.read_f64()?, r.read_f64()?);
        if let Some(d) = cd_mut(w, id) {
            d.active_move_speed = a;
            d.passive_move_speed = p;
        }
    } else {
        speeds::migrate_move_speeds(w, id);
    }
    if version <= 14 && cd(w, id).is_some_and(|d| d.is_paragon) {
        let hue = w.mobile(id).map_or(0, |m| (m.ty.hue)(w, id));
        if hue == 0x31 {
            // Paragon hue fixed, should now be 0x501 (`Paragon.Hue`).
            if let Some(m) = w.mobile_mut(id) {
                m.hue = 0x501;
            }
        }
    }
    if delete_time > muo_primitives::TimeSpan::ZERO {
        super::misc::start_delete_timer(w, id, delete_time);
    }
    Ok(())
}

/// `[AfterDeserialization] AfterDeserialization`.
pub fn after_deserialization(w: &mut World, id: MobileId) {
    let tick = w.tick_count;
    let now = w.now;
    let (active, passive) = cd(w, id).map_or((1.0, 1.0), |d| (d.active_speed, d.passive_speed));
    let type_name = super::type_name(w, id);
    if let Some(d) = cd_mut(w, id) {
        d.next_reacquire_time = tick;
        if active <= 0.0 || passive <= 0.0 {
            tracing::error!(
                "{type_name} loaded without speeds - is Data/npc-speeds.json missing or changed? Pacing at {FALLBACK_ACTIVE_SPEED}/{FALLBACK_PASSIVE_SPEED}.",
            );
            d.active_speed = FALLBACK_ACTIVE_SPEED;
            d.passive_speed = FALLBACK_PASSIVE_SPEED;
            d.current_speed = d.passive_speed;
        }
    }
    if w.core_aos()
        && w.mobile(id).is_some_and(|m| m.name_hue == 0x35)
        && let Some(m) = w.mobile_mut(id)
    {
        m.name_hue = -1;
    }
    let (summoned, summon_end) = cd(w, id).map_or((false, now), |d| (d.summoned, d.summon_end));
    if summoned {
        super::control::start_unsummon_timer(w, id, summon_end.since(now), None);
    }
    // An abandoned pet with no persisted countdown still despawns.
    let abandon = cd(w, id).is_some_and(|d| {
        !d.pending_delete_timer.running(w)
            && d.owners.last().is_some_and(|&o| w.mobile(o).is_some()) // LastOwner != null
            && !d.controlled
            && !d.is_stabled
    });
    if abandon {
        super::misc::start_delete_timer(w, id, muo_primitives::TimeSpan::from_days(3));
    }
    if let Some(m) = w.mobile(id) {
        let ty = m.ty;
        (ty.check_stat_timers)(w, id);
    }
    let current = cd(w, id).map_or(AIType::AiUseDefault, |d| d.current_ai);
    crate::ai::change_ai_type(w, id, current);
    super::control::add_followers(w, id);
    if ctype(w, id).is_some_and(|c| (c.is_animated_dead)(w, id))
        && let Some(f) = crate::hooks::hooks(w).and_then(|h| h.animate_dead_register)
    {
        let master = super::control::summon_master(w, id);
        f(w, master, id); // AnimateDeadSpell.Register(SummonMaster, this)
    }
    let _ = timer::stop; // keep the timer import honest for the delete timers started above
}

#[allow(dead_code)]
fn unused(_: ItemId) {}
