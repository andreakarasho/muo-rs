//! `BaseTrap` and the eight traps.
//!
//! One [`TrapData`] in `Item::ext` holds the fields of the whole family (`BaseTrap`'s two trigger times, the fire
//! column's damage range, the flame spurt's `Static` and timer, the gas trap's poison); [`TrapClass`] is the
//! `virtual`s (`PassivelyTriggered`, `PassiveTriggerDelay`, `PassiveTriggerRange`, `ResetDelay`, `OnTrigger`).
//!
//! The `[CommandProperty]` members of the traps (`WarningFlame`, `MinDamage`, `MaxDamage`, `Poison`, `Type`,
//! `Extended`, `Breathing`) are the props tables before `configure`.

use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::spells::helper::{SpellDamageSplit, damage_aos, damage_full};
use muo_content::utilities::types::{Numeric, PropType};
use muo_primitives::{DateTime, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, deps, effect_item, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::packets::message::MessageType;
use muo_server::poison::PoisonId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use crate::simple::item_id;

/// The state of every trap class.
#[derive(Clone, Debug)]
pub struct TrapData {
    pub next_passive: DateTime,
    pub next_active: DateTime,
    // FireColumnTrap
    pub warning_flame: bool,
    pub min_damage: i32,
    pub max_damage: i32,
    // FlameSpurtTrap
    pub spurt: Option<ItemId>,
    pub timer: Option<TimerHandle>,
    // GasTrap
    pub poison: Option<PoisonId>,
}

impl Default for TrapData {
    fn default() -> Self {
        TrapData {
            next_passive: DateTime::MIN,
            next_active: DateTime::MIN,
            warning_flame: false,
            min_damage: 0,
            max_damage: 0,
            spurt: None,
            timer: None,
            poison: None,
        }
    }
}

fn data(w: &World, id: ItemId) -> Option<&TrapData> {
    w.item(id)?.ext.get::<TrapData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut TrapData> {
    w.item_mut(id)?.ext.get_mut::<TrapData>()
}

/// The `virtual`s of `BaseTrap`.
#[derive(Copy, Clone)]
pub struct TrapClass {
    pub passively_triggered: bool,
    pub passive_trigger_delay: TimeSpan,
    pub passive_trigger_range: i32,
    pub reset_delay: TimeSpan,
    pub on_trigger: fn(&mut World, ItemId, MobileId),
}

static BASE_TRAP_CLASS: TrapClass = TrapClass {
    passively_triggered: false,
    passive_trigger_delay: TimeSpan::ZERO,
    passive_trigger_range: -1,
    reset_delay: TimeSpan::ZERO,
    on_trigger: |_, _, _| {},
};

fn class(w: &World, id: ItemId) -> &'static TrapClass {
    w.item(id)
        .and_then(|i| i.ty.family::<TrapClass>())
        .unwrap_or(&BASE_TRAP_CLASS)
}

fn new_trap(w: &mut World, ty: &'static ItemType, gid: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, gid);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(TrapData::default());
    }
    ops::set_movable(w, id, false);
    id
}

/// `GetEffectHue()`.
pub fn effect_hue(w: &World, id: ItemId) -> i32 {
    let hue = w.item(id).map_or(0, |i| (i.ty.hue)(w, id)) & 0x3FFF;
    if hue < 2 { 0 } else { hue - 1 }
}

/// `CheckRange(loc, range)`.
pub fn check_range(w: &World, id: ItemId, loc: Point3D, range: i32) -> bool {
    let z = w.item(id).map_or(0, |i| i.location.z);
    let me = ops::get_world_location(w, id);
    z + 8 >= loc.z && loc.z + 16 > z && utility::in_range(me.x, me.y, loc.x, loc.y, range)
}

fn on_movement(w: &mut World, id: ItemId, m: MobileId, old: Point3D) {
    (ITEM_BASE.on_movement)(w, id, m, old);
    let Some(loc) = w.mobile(m).map(|x| x.location) else {
        return;
    };
    if loc == old {
        return;
    }
    let c = class(w, id);
    let (next_active, now) = (data(w, id).map_or(DateTime::MIN, |d| d.next_active), w.now);
    let next_passive = data(w, id).map_or(DateTime::MIN, |d| d.next_passive);
    let entered = check_range(w, id, loc, 0) && !check_range(w, id, old, 0);
    if entered && now >= next_active {
        let at = now + c.reset_delay;
        if let Some(d) = data_mut(w, id) {
            d.next_active = at;
            d.next_passive = at;
        }
        (c.on_trigger)(w, id, m);
    } else if c.passively_triggered
        && check_range(w, id, loc, c.passive_trigger_range)
        && !check_range(w, id, old, c.passive_trigger_range)
        && now >= next_passive
    {
        if let Some(d) = data_mut(w, id) {
            d.next_passive = now + c.passive_trigger_delay;
        }
        (c.on_trigger)(w, id, m);
    }
}

/// `Server.Items.BaseTrap` (abstract).
pub static BASE_TRAP: ItemType = ItemType {
    full_name: "Server.Items.BaseTrap",
    construct: None,
    parent: Some(&ITEM_BASE),
    family: Some(&BASE_TRAP_CLASS),
    handles_on_movement: |_, _| true,
    on_movement,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(TrapData::default());
        }
        Ok(())
    },
    ..ITEM_BASE
};

fn is_staff(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.access_level > AccessLevel::Player)
}

fn alive(w: &World, m: MobileId) -> bool {
    muo_server::mobile::base::alive(w, m)
}

fn damage(w: &mut World, ticks_delay: TimeSpan, target: MobileId, amount: i32) {
    damage_full(
        w,
        None,
        ticks_delay,
        target,
        Some(target),
        f64::from(amount),
    );
}

fn at(w: &World, id: ItemId) -> (Point3D, Option<MapId>) {
    w.item(id)
        .map_or((Point3D::new(0, 0, 0), None), |i| (i.location, i.map))
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// The deleted-or-gone test of a timer's item.
fn gone(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

fn mobiles_in_range(w: &World, id: ItemId, range: i32) -> Vec<MobileId> {
    let (p, map) = at(w, id);
    muo_server::map::enumerate::mobiles_in_range(w, map, p.x, p.y, range).collect()
}

/// `IsDeadBondedPet`.
fn dead_bonded_pet(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| (x.ty.is_dead_bonded_pet)(w, m))
}

// ---- FireColumnTrap ----

fn fire_effect(w: &mut World, id: ItemId) {
    let (p, map) = at(w, id);
    let fx = effect_item::create(w, p, map, effect_item::DEFAULT_DURATION);
    muo_server::effects::send_location_particles(w, fx, 0x3709, 10, 30, 5052);
    muo_server::effects::play_sound(w, p, map, 0x225);
}

fn fire_trigger(w: &mut World, id: ItemId, from: MobileId) {
    if is_staff(w, from) {
        return;
    }
    let d = data(w, id).cloned().unwrap_or_default();
    if d.warning_flame {
        fire_effect(w, id);
    }
    let loc = w.mobile(from).map_or(Point3D::new(0, 0, 0), |m| m.location);
    if alive(w, from) && check_range(w, id, loc, 0) {
        let amount = utility::random_min_max(&mut w.rng, d.min_damage, d.max_damage);
        let split = SpellDamageSplit {
            fire: 100,
            ..SpellDamageSplit::default()
        };
        damage_aos(
            w,
            None,
            TimeSpan::from_secs_f64(0.5),
            from,
            Some(from),
            f64::from(amount),
            split,
        );
        if !d.warning_flame {
            fire_effect(w, id);
        }
    }
}

static FIRE_COLUMN_CLASS: TrapClass = TrapClass {
    passively_triggered: true,
    passive_trigger_delay: TimeSpan::from_secs(2),
    passive_trigger_range: 3,
    reset_delay: TimeSpan::from_millis(500),
    on_trigger: fire_trigger,
};

/// `Server.Items.FireColumnTrap`.
pub static FIRE_COLUMN_TRAP: ItemType = ItemType {
    full_name: "Server.Items.FireColumnTrap",
    parent: Some(&BASE_TRAP),
    construct: Some(|w| {
        let id = new_trap(w, &FIRE_COLUMN_TRAP, 0);
        crate::simple::own_template(w, id, "Server.Items.FireColumnTrap");
        if let Some(d) = data_mut(w, id) {
            d.min_damage = 10;
            d.max_damage = 40;
            d.warning_flame = true;
        }
        id
    }),
    family: Some(&FIRE_COLUMN_CLASS),
    serialize: |w, id, out| {
        (BASE_TRAP.serialize)(w, id, out);
        let d = data(w, id).cloned().unwrap_or_default();
        out.write_i32(0);
        out.write_bool(d.warning_flame);
        out.write_i32(d.min_damage);
        out.write_i32(d.max_damage);
    },
    deserialize: |w, id, r| {
        (BASE_TRAP.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let (wf, min, max) = (r.read_bool()?, r.read_i32()?, r.read_i32()?);
        if let Some(d) = data_mut(w, id) {
            d.warning_flame = wf;
            d.min_damage = min;
            d.max_damage = max;
        }
        Ok(())
    },
    ..BASE_TRAP
};

// ---- FlameSpurtTrap ----

fn spurt_start_timer(w: &mut World, id: ItemId) {
    if data(w, id).is_some_and(|d| d.timer.is_some()) {
        return;
    }
    let t = timer::start(
        w,
        TimeSpan::from_secs(1),
        TimeSpan::from_secs(1),
        0,
        move |w| spurt_refresh(w, id),
    );
    if let Some(d) = data_mut(w, id) {
        d.timer = Some(t);
    }
}

fn spurt_stop_timer(w: &mut World, id: ItemId) {
    if let Some(mut t) = data_mut(w, id).and_then(|d| d.timer.take()) {
        t.cancel(w);
    }
}

fn spurt_check_timer(w: &mut World, id: ItemId) {
    let (p, map) = (
        ops::get_world_location(w, id),
        w.item(id).and_then(|i| i.map),
    );
    let active = map
        .and_then(|m| w.maps.get(m))
        .is_some_and(|m| m.sector_at(p.x, p.y).active());
    if active {
        spurt_start_timer(w, id);
    } else {
        spurt_stop_timer(w, id);
    }
}

fn spurt_refresh(w: &mut World, id: ItemId) {
    if gone(w, id) {
        return;
    }
    let z = w.item(id).map_or(0, |i| i.location.z);
    for mob in mobiles_in_range(w, id, 3) {
        let Some(m) = w.mobile(mob) else { continue };
        if m.player
            && alive(w, mob)
            && m.access_level <= AccessLevel::Player
            && z + 8 >= m.location.z
            && m.location.z + 16 > z
        {
            let spurt = data(w, id).and_then(|d| d.spurt);
            if spurt.is_none_or(|s| gone(w, s)) {
                let s = (crate::misc::basic::STATIC.make.expect("set"))(w);
                item_id(w, s, 0x3709);
                let (loc, map) = at(w, id);
                ops::move_to_world_map(w, s, loc, map);
                if let Some(d) = data_mut(w, id) {
                    d.spurt = Some(s);
                }
                let (wl, wmap) = (
                    ops::get_world_location(w, id),
                    w.item(id).and_then(|i| i.map),
                );
                muo_server::effects::play_sound(w, wl, wmap, 0x309);
            }
            return;
        }
    }
    if let Some(s) = data_mut(w, id).and_then(|d| d.spurt.take()) {
        delete(w, s);
    }
}

fn hurt_sound(w: &mut World, m: MobileId) {
    let female = w.mobile(m).is_some_and(|x| x.female);
    muo_server::mobile::part2::effects::play_sound(w, m, if female { 0x327 } else { 0x437 });
}

fn spurt_on_move_over(w: &mut World, id: ItemId, m: MobileId) -> bool {
    if is_staff(w, m) {
        return true;
    }
    if !(w.mobile(m).is_some_and(|x| x.player) && alive(w, m)) {
        return false;
    }
    spurt_check_timer(w, id);
    let amount = utility::random_min_max(&mut w.rng, 1, 30);
    damage(w, TimeSpan::from_ticks(1), m, amount);
    hurt_sound(w, m);
    false
}

fn spurt_on_movement(w: &mut World, id: ItemId, m: MobileId, old: Point3D) {
    on_movement(w, id, m, old);
    let Some(mob) = w.mobile(m) else { return };
    let loc = mob.location;
    if loc == old
        || !mob.player
        || !alive(w, m)
        || mob.access_level > AccessLevel::Player
        || !check_range_pair(w, id, loc, old, 1)
    {
        return;
    }
    spurt_check_timer(w, id);
    let amount = utility::random_min_max(&mut w.rng, 1, 10);
    damage(w, TimeSpan::from_ticks(1), m, amount);
    hurt_sound(w, m);
    let human = w.mobile(m).is_some_and(|_| {
        w.resources
            .get::<muo_primitives::BodyTable>()
            .is_some_and(|t| t.is_human(muo_server::mobile::part1::props_b::body(w, m)))
    });
    if human {
        muo_server::mobile::base::animate(w, m, 20, 1, 1, true, false, 0);
    }
}

/// `CheckRange(loc, oldLoc, range)`.
fn check_range_pair(w: &World, id: ItemId, loc: Point3D, old: Point3D, range: i32) -> bool {
    check_range(w, id, loc, range) && !check_range(w, id, old, range)
}

/// `Server.Items.FlameSpurtTrap`.
pub static FLAME_SPURT_TRAP: ItemType = ItemType {
    full_name: "Server.Items.FlameSpurtTrap",
    parent: Some(&BASE_TRAP),
    construct: Some(|w| {
        let id = new_trap(w, &FLAME_SPURT_TRAP, 0);
        crate::simple::own_template(w, id, "Server.Items.FlameSpurtTrap")
    }),
    on_movement: spurt_on_movement,
    on_move_over: spurt_on_move_over,
    on_location_change: |w, id, old| {
        (BASE_TRAP.on_location_change)(w, id, old);
        spurt_check_timer(w, id);
    },
    on_map_change: |w, id| {
        (BASE_TRAP.on_map_change)(w, id);
        spurt_check_timer(w, id);
    },
    on_sector_activate: |w, id| {
        (BASE_TRAP.on_sector_activate)(w, id);
        spurt_start_timer(w, id);
    },
    on_sector_deactivate: |w, id| {
        (BASE_TRAP.on_sector_deactivate)(w, id);
        spurt_stop_timer(w, id);
    },
    on_delete: |w, id| {
        (BASE_TRAP.on_delete)(w, id);
        if let Some(s) = data(w, id).and_then(|d| d.spurt) {
            delete(w, s);
        }
    },
    serialize: |w, id, out| {
        (BASE_TRAP.serialize)(w, id, out);
        out.write_i32(0);
        out.write_item(w, data(w, id).and_then(|d| d.spurt));
    },
    deserialize: |w, id, r| {
        (BASE_TRAP.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let spurt = r.read_item(w)?;
        if let Some(d) = data_mut(w, id) {
            d.spurt = spurt;
        }
        // [AfterDeserialization(false)] -> Timer.DelayCall
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            if let Some(s) = data(w, id).and_then(|d| d.spurt) {
                delete(w, s);
            }
            spurt_check_timer(w, id);
        });
        Ok(())
    },
    ..BASE_TRAP
};

// ---- GasTrap ----

/// `GasTrap.GetBaseID(type)`.
pub fn gas_base_id(kind: i32) -> i32 {
    match kind {
        0 => 0x113C, // NorthWall
        1 => 0x1147, // WestWall
        2 => 0x11A8, // Floor
        _ => 0,
    }
}

/// `GasTrap.Type`.
fn gas_type(w: &World, id: ItemId) -> i32 {
    match w.item(id).map_or(0, |i| i.item_id) {
        0x113C => 0,
        0x1147 => 1,
        0x11A8 => 2,
        _ => 1,
    }
}

fn gas_trigger(w: &mut World, id: ItemId, from: MobileId) {
    let Some(poison) = data(w, id).and_then(|d| d.poison) else {
        return;
    };
    if !w.mobile(from).is_some_and(|m| m.player) || !alive(w, from) || is_staff(w, from) {
        return;
    }
    let (p, map) = at(w, id);
    let hue = effect_hue(w, id);
    let base = gas_base_id(gas_type(w, id));
    muo_server::effects::send_location_effect(w, p, map, base - 2, 16, 3, hue, 0);
    muo_server::effects::play_sound(w, p, map, 0x231);
    muo_server::mobile::base::apply_poison(w, from, Some(from), poison);
    deps::local_overhead_message(w, from, MessageType::REGULAR, 0x22, 500855); // You are enveloped by a noxious gas cloud!
}

static GAS_CLASS: TrapClass = TrapClass {
    passive_trigger_range: 0,
    on_trigger: gas_trigger,
    ..BASE_TRAP_CLASS
};

/// `new GasTrap(type, poison)`.
pub fn new_gas_trap(w: &mut World, kind: i32, poison: Option<PoisonId>) -> ItemId {
    let id = new_trap(w, &GAS_TRAP, gas_base_id(kind));
    if let Some(d) = data_mut(w, id) {
        d.poison = poison;
    }
    id
}

/// `Poison.Lesser`.
const LESSER: PoisonId = PoisonId(0);

/// `Server.Items.GasTrap`.
pub static GAS_TRAP: ItemType = ItemType {
    full_name: "Server.Items.GasTrap",
    parent: Some(&BASE_TRAP),
    construct: Some(|w| new_gas_trap(w, 2, Some(LESSER))),
    family: Some(&GAS_CLASS),
    serialize: |w, id, out| {
        (BASE_TRAP.serialize)(w, id, out);
        out.write_i32(0);
        out.write_poison(data(w, id).and_then(|d| d.poison));
    },
    deserialize: |w, id, r| {
        (BASE_TRAP.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let p = r.read_poison()?;
        if let Some(d) = data_mut(w, id) {
            d.poison = p;
        }
        Ok(())
    },
    ..BASE_TRAP
};

// ---- GiantSpikeTrap ----

static GIANT_SPIKE_CLASS: TrapClass = TrapClass {
    passively_triggered: true,
    passive_trigger_range: 3,
    on_trigger: |w, id, from| {
        if is_staff(w, from) {
            return;
        }
        let (p, map) = at(w, id);
        let hue = effect_hue(w, id);
        muo_server::effects::send_location_effect(w, p, map, 0x1D99, 48, 2, hue, 0);
        let loc = w.mobile(from).map_or(Point3D::new(0, 0, 0), |m| m.location);
        if alive(w, from) && check_range(w, id, loc, 0) {
            let amount = utility::dice(&mut w.rng, 10, 7, 0);
            damage(w, TimeSpan::from_ticks(1), from, amount);
        }
    },
    ..BASE_TRAP_CLASS
};

/// `Server.Items.GiantSpikeTrap`.
pub static GIANT_SPIKE_TRAP: ItemType = ItemType {
    full_name: "Server.Items.GiantSpikeTrap",
    parent: Some(&BASE_TRAP),
    construct: Some(|w| {
        let id = new_trap(w, &GIANT_SPIKE_TRAP, 0);
        crate::simple::own_template(w, id, "Server.Items.GiantSpikeTrap")
    }),
    family: Some(&GIANT_SPIKE_CLASS),
    serialize: |w, id, out| {
        (BASE_TRAP.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_TRAP.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_TRAP
};

// ---- MushroomTrap ----

fn mushroom_reset(w: &mut World, id: ItemId) {
    let (p, map) = at(w, id);
    let region = muo_server::region::find(w, p, map);
    if muo_content::regions::is_part_of_dungeon(w, region) {
        item_id(w, id, 0x1125); // reset
    } else {
        delete(w, id);
    }
}

static MUSHROOM_CLASS: TrapClass = TrapClass {
    passively_triggered: true,
    passive_trigger_range: 2,
    on_trigger: |w, id, from| {
        if !alive(w, from) || w.item(id).map_or(0, |i| i.item_id) != 0x1125 || is_staff(w, from) {
            return;
        }
        item_id(w, id, 0x1126);
        let (p, map) = at(w, id);
        muo_server::effects::play_sound(w, p, map, 0x306);
        let amount = utility::dice(&mut w.rng, 2, 4, 0);
        damage(w, TimeSpan::from_secs_f64(0.5), from, amount);
        timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
            mushroom_reset(w, id)
        });
    },
    ..BASE_TRAP_CLASS
};

/// `Server.Items.MushroomTrap`.
pub static MUSHROOM_TRAP: ItemType = ItemType {
    full_name: "Server.Items.MushroomTrap",
    parent: Some(&BASE_TRAP),
    construct: Some(|w| {
        let id = new_trap(w, &MUSHROOM_TRAP, 0);
        crate::simple::own_template(w, id, "Server.Items.MushroomTrap")
    }),
    family: Some(&MUSHROOM_CLASS),
    serialize: |w, id, out| {
        (BASE_TRAP.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_TRAP.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        // [AfterDeserialization(false)]
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            if w.item(id).is_some_and(|i| i.item_id == 0x1126) {
                mushroom_reset(w, id);
            }
        });
        Ok(())
    },
    ..BASE_TRAP
};

// ---- SawTrap ----

/// `SawTrap.GetBaseID(type)`: WestWall, NorthWall, WestFloor, NorthFloor.
pub fn saw_base_id(kind: i32) -> i32 {
    match kind {
        1 => 0x1103,
        0 => 0x1116,
        3 => 0x11AC,
        2 => 0x11B1,
        _ => 0,
    }
}

fn saw_type(w: &World, id: ItemId) -> i32 {
    match w.item(id).map_or(0, |i| i.item_id) {
        0x1103 => 1,
        0x1116 => 0,
        0x11AC => 3,
        0x11B1 => 2,
        _ => 1,
    }
}

static SAW_CLASS: TrapClass = TrapClass {
    passive_trigger_range: 0,
    on_trigger: |w, id, from| {
        if !alive(w, from) || is_staff(w, from) {
            return;
        }
        let (p, map) = at(w, id);
        let hue = effect_hue(w, id);
        let base = saw_base_id(saw_type(w, id));
        muo_server::effects::send_location_effect(w, p, map, base + 1, 6, 3, hue, 0);
        muo_server::effects::play_sound(w, p, map, 0x21C);
        let amount = utility::random_min_max(&mut w.rng, 5, 15);
        damage(w, TimeSpan::from_ticks(1), from, amount);
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x22, 500853); // You stepped onto a blade trap!
    },
    ..BASE_TRAP_CLASS
};

/// `Server.Items.SawTrap`.
pub static SAW_TRAP: ItemType = ItemType {
    full_name: "Server.Items.SawTrap",
    parent: Some(&BASE_TRAP),
    construct: Some(|w| new_trap(w, &SAW_TRAP, saw_base_id(3))),
    family: Some(&SAW_CLASS),
    serialize: |w, id, out| {
        (BASE_TRAP.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_TRAP.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_TRAP
};

// ---- SpikeTrap ----

/// `SpikeTrap.GetBaseID(type)`: WestWall, NorthWall, WestFloor, NorthFloor.
pub fn spike_base_id(kind: i32) -> i32 {
    [4360, 4379, 4506, 4512]
        .get(kind as usize)
        .copied()
        .unwrap_or(0)
}

fn spike_extended_offset(kind: i32) -> i32 {
    [6, 6, 5, 5].get(kind as usize).copied().unwrap_or(0)
}

fn spike_extended_id(kind: i32) -> i32 {
    spike_base_id(kind) + spike_extended_offset(kind)
}

fn spike_type(w: &World, id: ItemId) -> i32 {
    match w.item(id).map_or(0, |i| i.item_id) {
        4360 | 4361 | 4366 => 0,
        4379 | 4380 | 4385 => 1,
        4506 | 4507 | 4511 => 2,
        4512 | 4513 | 4517 => 3,
        _ => 0,
    }
}

fn spike_set_extended(w: &mut World, id: ItemId, value: bool) {
    let k = spike_type(w, id);
    item_id(
        w,
        id,
        if value {
            spike_extended_id(k)
        } else {
            spike_base_id(k)
        },
    );
}

fn spike_trigger(w: &mut World, id: ItemId, from: MobileId) {
    if !alive(w, from) || is_staff(w, from) {
        return;
    }
    let (p, map) = at(w, id);
    let hue = effect_hue(w, id);
    let k = spike_type(w, id);
    muo_server::effects::send_location_effect(w, p, map, spike_base_id(k) + 1, 18, 3, hue, 0);
    muo_server::effects::play_sound(w, p, map, 0x22C);
    let victims: Vec<MobileId> = muo_server::map::enumerate::mobiles_at(w, map, p.x, p.y)
        .filter(|&m| {
            alive(w, m)
                && !dead_bonded_pet(w, m)
                && w.mobile(m)
                    .is_some_and(|x| x.access_level == AccessLevel::Player)
        })
        .collect();
    for mob in victims {
        let amount = utility::random_min_max(&mut w.rng, 1, 6) * 6;
        damage(w, TimeSpan::from_ticks(1), mob, amount);
    }
    timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
        spike_extended(w, id)
    });
    deps::local_overhead_message(w, from, MessageType::REGULAR, 0x22, 500852); // You stepped onto a spike trap!
}

fn spike_extended(w: &mut World, id: ItemId) {
    spike_set_extended(w, id, true);
    timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
        spike_retracted(w, id)
    });
}

fn spike_retracted(w: &mut World, id: ItemId) {
    spike_set_extended(w, id, false);
    let (p, map) = at(w, id);
    let hue = effect_hue(w, id);
    let k = spike_type(w, id);
    muo_server::effects::send_location_effect(w, p, map, spike_extended_id(k) - 1, 6, 3, hue, 0);
}

static SPIKE_CLASS: TrapClass = TrapClass {
    passive_trigger_range: 0,
    reset_delay: TimeSpan::from_secs(6),
    on_trigger: spike_trigger,
    ..BASE_TRAP_CLASS
};

/// `Server.Items.SpikeTrap`.
pub static SPIKE_TRAP: ItemType = ItemType {
    full_name: "Server.Items.SpikeTrap",
    parent: Some(&BASE_TRAP),
    construct: Some(|w| new_trap(w, &SPIKE_TRAP, spike_base_id(2))),
    family: Some(&SPIKE_CLASS),
    serialize: |w, id, out| {
        (BASE_TRAP.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_TRAP.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        spike_set_extended(w, id, false); // AfterDeserialization
        Ok(())
    },
    ..BASE_TRAP
};

// ---- StoneFaceTrap ----

/// `StoneFaceTrap.GetBaseID(type)`: NorthWestWall, NorthWall, WestWall.
pub fn stone_base_id(kind: i32) -> i32 {
    [0x10F5, 0x10FC, 0x110F]
        .get(kind as usize)
        .copied()
        .unwrap_or(0)
}

fn stone_fire_id(kind: i32) -> i32 {
    [0x10F7, 0x10FE, 0x1111]
        .get(kind as usize)
        .copied()
        .unwrap_or(0)
}

fn stone_type(w: &World, id: ItemId) -> i32 {
    match w.item(id).map_or(0, |i| i.item_id) {
        0x10F5..=0x10F7 => 0,
        0x10FC..=0x10FE => 1,
        0x110F..=0x1111 => 2,
        _ => 0,
    }
}

fn stone_set_breathing(w: &mut World, id: ItemId, value: bool) {
    let k = stone_type(w, id);
    item_id(
        w,
        id,
        if value {
            stone_fire_id(k)
        } else {
            stone_base_id(k)
        },
    );
}

fn stone_trigger(w: &mut World, id: ItemId, from: MobileId) {
    if !alive(w, from) || is_staff(w, from) {
        return;
    }
    let (p, map) = at(w, id);
    muo_server::effects::play_sound(w, p, map, 0x359);
    stone_set_breathing(w, id, true);
    timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
        stone_set_breathing(w, id, false)
    });
    timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
        let damaging = w
            .item(id)
            .is_some_and(|i| i.ty.is(&STONE_FACE_TRAP) && !i.ty.is(&STONE_FACE_TRAP_NO_DAMAGE));
        if damaging {
            stone_trigger_damage(w, id);
        }
    });
}

fn stone_trigger_damage(w: &mut World, id: ItemId) {
    let victims: Vec<MobileId> = mobiles_in_range(w, id, 1)
        .into_iter()
        .filter(|&m| {
            alive(w, m)
                && !dead_bonded_pet(w, m)
                && w.mobile(m)
                    .is_some_and(|x| x.access_level == AccessLevel::Player)
        })
        .collect();
    for mob in victims {
        let amount = utility::dice(&mut w.rng, 3, 15, 0);
        damage(w, TimeSpan::from_ticks(1), mob, amount);
    }
}

static STONE_FACE_CLASS: TrapClass = TrapClass {
    passively_triggered: true,
    passive_trigger_range: 2,
    on_trigger: stone_trigger,
    ..BASE_TRAP_CLASS
};

/// `Server.Items.StoneFaceTrap`.
pub static STONE_FACE_TRAP: ItemType = ItemType {
    full_name: "Server.Items.StoneFaceTrap",
    parent: Some(&BASE_TRAP),
    construct: Some(|w| {
        let id = new_trap(w, &STONE_FACE_TRAP, 0);
        crate::simple::own_template(w, id, "Server.Items.StoneFaceTrap")
    }),
    family: Some(&STONE_FACE_CLASS),
    serialize: |w, id, out| {
        (BASE_TRAP.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_TRAP.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        stone_set_breathing(w, id, false); // AfterDeserialization
        Ok(())
    },
    ..BASE_TRAP
};

/// `Server.Items.StoneFaceTrapNoDamage`: `TriggerDamage` does nothing.
pub static STONE_FACE_TRAP_NO_DAMAGE: ItemType = ItemType {
    full_name: "Server.Items.StoneFaceTrapNoDamage",
    parent: Some(&STONE_FACE_TRAP),
    construct: Some(|w| {
        let id = new_trap(w, &STONE_FACE_TRAP_NO_DAMAGE, 0);
        crate::simple::own_template(w, id, "Server.Items.StoneFaceTrapNoDamage")
    }),
    serialize: |w, id, out| {
        (STONE_FACE_TRAP.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (STONE_FACE_TRAP.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..STONE_FACE_TRAP
};

// ---- constructors for `[add` ----

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));

const POISON: Kind = Kind::Poison;
static GAS_TYPE_NAMES: &[(&str, i64)] = &[("NorthWall", 0), ("WestWall", 1), ("Floor", 2)];
static SAW_TYPE_NAMES: &[(&str, i64)] = &[
    ("WestWall", 0),
    ("NorthWall", 1),
    ("WestFloor", 2),
    ("NorthFloor", 3),
];

fn poison_of(v: &Value) -> Option<PoisonId> {
    match v {
        Value::Null => None,
        Value::Poison(p) => *p,
        v => Some(PoisonId(v.i64() as u8)),
    }
}

static GAS_P1: [Param; 1] = [Param {
    name: "poison",
    kind: POISON,
    default: None,
    ty: "Poison",
}];
static GAS_P2: [Param; 2] = [
    Param {
        name: "type",
        kind: Kind::Plain(PropType::Enum(GAS_TYPE_NAMES)),
        default: None,
        ty: "GasTrapType",
    },
    Param {
        name: "poison",
        kind: POISON,
        default: Some(Value::Null),
        ty: "Poison",
    },
];
static C_GAS: [Ctor; 3] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| EntityId::Item(new_gas_trap(w, 2, Some(LESSER))),
    },
    Ctor {
        access: None,
        params: &GAS_P1,
        make: |w, v| EntityId::Item(new_gas_trap(w, 2, poison_of(&v[0]))),
    },
    Ctor {
        access: None,
        params: &GAS_P2,
        make: |w, v| EntityId::Item(new_gas_trap(w, v[0].i32(), poison_of(&v[1]))),
    },
];
static SAW_P: [Param; 1] = [Param {
    name: "type",
    kind: Kind::Plain(PropType::Enum(SAW_TYPE_NAMES)),
    default: Some(Value::Enum(3)),
    ty: "SawTrapType",
}];
static C_SAW: [Ctor; 1] = [Ctor {
    access: None,
    params: &SAW_P,
    make: |w, v| EntityId::Item(new_trap(w, &SAW_TRAP, saw_base_id(v[0].i32()))),
}];
static SPIKE_P: [Param; 1] = [Param {
    name: "type",
    kind: Kind::Plain(PropType::Enum(&[
        ("WestWall", 0),
        ("NorthWall", 1),
        ("WestFloor", 2),
        ("NorthFloor", 3),
    ])),
    default: Some(Value::Enum(2)),
    ty: "SpikeTrapType",
}];
static C_SPIKE: [Ctor; 1] = [Ctor {
    access: None,
    params: &SPIKE_P,
    make: |w, v| EntityId::Item(new_trap(w, &SPIKE_TRAP, spike_base_id(v[0].i32()))),
}];

// ---- `[props` ----

use muo_content::props::PropDef;

macro_rules! trap_prop {
    ($name:literal, $kind:expr, $get:expr, $set:expr) => {
        PropDef {
            name: $name,
            kind: $kind,
            read: AccessLevel::GameMaster,
            write: AccessLevel::GameMaster,
            get: |w, e| match e {
                EntityId::Item(i) => $get(w, i),
                _ => Value::Null,
            },
            set: Some(|w, e, v| {
                let EntityId::Item(i) = e else {
                    return Err("not an item".to_string());
                };
                $set(w, i, v);
                Ok(())
            }),
        }
    };
}

static FIRE_COLUMN_PROPS: &[PropDef] = &[
    trap_prop!(
        "WarningFlame",
        Kind::Plain(PropType::Bool),
        |w: &World, i| Value::Bool(data(w, i).is_some_and(|d| d.warning_flame)),
        |w: &mut World, i, v: Value| {
            if let Some(d) = data_mut(w, i) {
                d.warning_flame = v.bool();
            }
        }
    ),
    trap_prop!(
        "MinDamage",
        INT,
        |w: &World, i| Value::Int(i64::from(data(w, i).map_or(0, |d| d.min_damage))),
        |w: &mut World, i, v: Value| {
            if let Some(d) = data_mut(w, i) {
                d.min_damage = v.i32();
            }
        }
    ),
    trap_prop!(
        "MaxDamage",
        INT,
        |w: &World, i| Value::Int(i64::from(data(w, i).map_or(0, |d| d.max_damage))),
        |w: &mut World, i, v: Value| {
            if let Some(d) = data_mut(w, i) {
                d.max_damage = v.i32();
            }
        }
    ),
];

static GAS_PROPS: &[PropDef] = &[
    trap_prop!(
        "Poison",
        POISON,
        |w: &World, i| Value::Poison(data(w, i).and_then(|d| d.poison)),
        |w: &mut World, i, v: Value| {
            let p = poison_of(&v);
            if let Some(d) = data_mut(w, i) {
                d.poison = p;
            }
        }
    ),
    trap_prop!(
        "Type",
        Kind::Plain(PropType::Enum(GAS_TYPE_NAMES)),
        |w: &World, i| Value::Enum(i64::from(gas_type(w, i))),
        |w: &mut World, i, v: Value| item_id(w, i, gas_base_id(v.i32()))
    ),
];

static SAW_PROPS: &[PropDef] = &[trap_prop!(
    "Type",
    Kind::Plain(PropType::Enum(SAW_TYPE_NAMES)),
    |w: &World, i| Value::Enum(i64::from(saw_type(w, i))),
    |w: &mut World, i, v: Value| item_id(w, i, saw_base_id(v.i32()))
)];

static SPIKE_TYPE_NAMES: &[(&str, i64)] = &[
    ("WestWall", 0),
    ("NorthWall", 1),
    ("WestFloor", 2),
    ("NorthFloor", 3),
];

fn spike_is_extended(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| i.item_id == spike_extended_id(spike_type(w, id)))
}

static SPIKE_PROPS: &[PropDef] = &[
    trap_prop!(
        "Type",
        Kind::Plain(PropType::Enum(SPIKE_TYPE_NAMES)),
        |w: &World, i| Value::Enum(i64::from(spike_type(w, i))),
        |w: &mut World, i, v: Value| {
            let k = v.i32();
            let gid = if spike_is_extended(w, i) {
                spike_extended_id(k)
            } else {
                spike_base_id(k)
            };
            item_id(w, i, gid);
        }
    ),
    trap_prop!(
        "Extended",
        Kind::Plain(PropType::Bool),
        |w: &World, i| Value::Bool(spike_is_extended(w, i)),
        |w: &mut World, i, v: Value| spike_set_extended(w, i, v.bool())
    ),
];

static STONE_TYPE_NAMES: &[(&str, i64)] =
    &[("NorthWestWall", 0), ("NorthWall", 1), ("WestWall", 2)];

fn stone_is_breathing(w: &World, id: ItemId) -> bool {
    w.item(id)
        .is_some_and(|i| i.item_id == stone_fire_id(stone_type(w, id)))
}

static STONE_FACE_PROPS: &[PropDef] = &[
    trap_prop!(
        "Type",
        Kind::Plain(PropType::Enum(STONE_TYPE_NAMES)),
        |w: &World, i| Value::Enum(i64::from(stone_type(w, i))),
        |w: &mut World, i, v: Value| {
            let k = v.i32();
            let gid = if stone_is_breathing(w, i) {
                stone_fire_id(k)
            } else {
                stone_base_id(k)
            };
            item_id(w, i, gid);
        }
    ),
    trap_prop!(
        "Breathing",
        Kind::Plain(PropType::Bool),
        |w: &World, i| Value::Bool(stone_is_breathing(w, i)),
        |w: &mut World, i, v: Value| stone_set_breathing(w, i, v.bool())
    ),
];

fn configure(w: &mut World) {
    let _ = INT;
    muo_content::props::add(w, "Server.Items.FireColumnTrap", FIRE_COLUMN_PROPS);
    muo_content::props::add(w, "Server.Items.GasTrap", GAS_PROPS);
    muo_content::props::add(w, "Server.Items.SawTrap", SAW_PROPS);
    muo_content::props::add(w, "Server.Items.SpikeTrap", SPIKE_PROPS);
    muo_content::props::add(w, "Server.Items.StoneFaceTrap", STONE_FACE_PROPS);
    add_ctors(w, "Server.Items.GasTrap", &C_GAS);
    add_ctors(w, "Server.Items.SawTrap", &C_SAW);
    add_ctors(w, "Server.Items.SpikeTrap", &C_SPIKE);
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &BASE_TRAP,
        &FIRE_COLUMN_TRAP,
        &FLAME_SPURT_TRAP,
        &GAS_TRAP,
        &GIANT_SPIKE_TRAP,
        &MUSHROOM_TRAP,
        &SAW_TRAP,
        &SPIKE_TRAP,
        &STONE_FACE_TRAP,
        &STONE_FACE_TRAP_NO_DAMAGE,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BaseTrap.Configure",
        configure,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::GenericWriter;
    use muo_io::{SaveReader, SaveWriter};

    fn save(w: &World, id: ItemId) -> Vec<u8> {
        let mut out = SaveWriter::new(true);
        (w.item(id).unwrap().ty.serialize)(w, id, &mut out);
        out.into_vec()
    }

    #[test]
    fn fire_column_trap_saves_the_generated_layout() {
        let mut w = crate::test_world();
        let id = (FIRE_COLUMN_TRAP.construct.unwrap())(&mut w);
        assert_eq!(
            (w.item(id).unwrap().item_id, w.item(id).unwrap().movable()),
            (0x1B71, false)
        );
        let mut want = SaveWriter::new(true);
        (ITEM_BASE.serialize)(&w, id, &mut want);
        want.write_i32(0); // BaseTrap
        want.write_i32(0); // FireColumnTrap version
        want.write_bool(true);
        want.write_i32(10);
        want.write_i32(40);
        let want = want.into_vec();
        assert_eq!(save(&w, id), want);
        let id2 = w.add_item(muo_server::item::Item::new_for_load(
            muo_primitives::Serial::new(0x4000_0100),
            &FIRE_COLUMN_TRAP,
        ));
        let mut r = SaveReader::new(&want);
        (FIRE_COLUMN_TRAP.deserialize)(&mut w, id2, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        assert_eq!(save(&w, id2), want);
    }

    #[test]
    fn trap_graphics() {
        let mut w = crate::test_world();
        let id = (STONE_FACE_TRAP_NO_DAMAGE.construct.unwrap())(&mut w);
        assert_eq!(w.item(id).unwrap().item_id, 0x10FC);
        assert_eq!(stone_type(&w, id), 1);
        stone_set_breathing(&mut w, id, true);
        assert_eq!(w.item(id).unwrap().item_id, 0x10FE);
        let gas = new_gas_trap(&mut w, 0, None);
        assert_eq!(w.item(gas).unwrap().item_id, 0x113C);
        assert_eq!(effect_hue(&w, gas), 0);
        assert!(check_range(&w, gas, Point3D::new(0, 0, 0), 0));
    }
}
