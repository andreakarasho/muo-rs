//! The faction traps: `BaseFactionTrap` (with `BaseTrap`'s movement logic, which no other
//! class here needs yet), the four traps, `BaseFactionTrapDeed` and its four deeds, `FactionTrapRemovalKit`, and
//! the `RemoveTrap` / `DetectHidden` faction branches (`SkillItemFns`).
//!
//! `BaseFactionTrapDeed.OnCraft` (`ICraftable`) is [`craft_deed`], registered with the craftables in `configure`.

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, MessageType, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, base, ops};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages as mm;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use muo_content::hooks::rnd;

use super::state::{read_faction_ref, write_faction_ref};
use super::tables::FACTION_DEFS;
use super::{find, find_full, st, st_mut, town};

/// `AllowedPlacing`.
#[derive(Copy, Clone, Eq, PartialEq)]
enum Placing {
    AnyFactionTown,
    ControlledFactionTown,
    FactionStronghold,
}

/// What distinguishes the four traps.
struct Kind {
    item_id: i32,
    label: i32,
    attack_message: i32,
    disarm_message: i32,
    sound: i32,
    hue: i32,
    placing: Placing,
    deed_item_id: i32,
    deed_label: i32,
}

static EXPLOSION: Kind = Kind {
    item_id: 0x11C1,
    label: 1044599,
    attack_message: 1010543,
    disarm_message: 1010539,
    sound: 0x307,
    hue: 0x78,
    placing: Placing::AnyFactionTown,
    deed_item_id: 0x36D2,
    deed_label: 1044603,
};
static GAS: Kind = Kind {
    item_id: 0x113C,
    label: 1044598,
    attack_message: 1010542,
    disarm_message: 502376,
    sound: 0x230,
    hue: 0x44,
    placing: Placing::FactionStronghold,
    deed_item_id: 0x11AB,
    deed_label: 1044602,
};
static SAW: Kind = Kind {
    item_id: 0x11AC,
    label: 1041047,
    attack_message: 1010544,
    disarm_message: 1010540,
    sound: 0x218,
    hue: 0x5A,
    placing: Placing::ControlledFactionTown,
    deed_item_id: 0x1107,
    deed_label: 1044604,
};
static SPIKE: Kind = Kind {
    item_id: 0x11A0,
    label: 1044601,
    attack_message: 1010545,
    disarm_message: 1010541,
    sound: 0x22E,
    hue: 0x5A,
    placing: Placing::ControlledFactionTown,
    deed_item_id: 0x11A5,
    deed_label: 1044605,
};

/// The fields of `BaseFactionTrap` and `BaseTrap`.
pub struct TrapData {
    pub faction: Option<usize>,
    pub placer: Option<MobileId>,
    pub time_of_placement: DateTime,
    next_active: DateTime,
    conceal: TimerHandle,
    kind: &'static Kind,
    which: u8,
}

fn td(w: &World, id: ItemId) -> Option<&TrapData> {
    w.item(id)?.ext.get::<TrapData>()
}

fn td_mut(w: &mut World, id: ItemId) -> Option<&mut TrapData> {
    w.item_mut(id)?.ext.get_mut::<TrapData>()
}

/// `item is BaseFactionTrap`.
pub fn is_trap(w: &World, id: ItemId) -> bool {
    td(w, id).is_some()
}

fn kind(w: &World, id: ItemId) -> &'static Kind {
    td(w, id).map_or(&SPIKE, |d| d.kind)
}

fn rd(r: &mut dyn GenericReader) -> muo_io::Result<DateTime> {
    Ok(DateTime::from_ticks(r.read_date_time_ticks()?))
}

/// `new FactionXTrap(faction, placer)`.
fn new_trap(
    w: &mut World,
    ty: &'static ItemType,
    kind: &'static Kind,
    which: u8,
    f: Option<usize>,
    m: Option<MobileId>,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, kind.item_id);
    ops::set_movable(w, id, false); // BaseTrap
    ops::set_visible(w, id, false);
    let now = w.now;
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(TrapData {
            faction: f,
            placer: m,
            time_of_placement: now,
            next_active: DateTime::MIN,
            conceal: TimerHandle::default(),
            kind,
            which,
        });
    }
    id
}

fn decay_period(w: &World) -> Option<TimeSpan> {
    // `Core.AOS ? 1 day : TimeSpan.MaxValue`
    w.core_aos().then_some(TimeSpan::from_days(1))
}

/// `CheckDecay()`.
/// The test of `CheckDecay()`, without its side effect.
pub fn has_decayed(w: &World, id: ItemId) -> bool {
    let Some(period) = decay_period(w) else {
        return false;
    };
    let placed = td(w, id).map_or(DateTime::MIN, |d| d.time_of_placement);
    placed + period < w.now
}

fn check_decay(w: &mut World, id: ItemId) -> bool {
    if has_decayed(w, id) {
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            if let Some(t) = w.item(id).map(|i| i.ty) {
                (t.delete)(w, id);
            }
        });
        return true;
    }
    false
}

/// `Conceal()`.
fn conceal(w: &mut World, id: ItemId) {
    if let Some(mut h) = td_mut(w, id).map(|d| std::mem::take(&mut d.conceal)) {
        h.cancel(w);
    }
    if !ops::is_deleted(w, id) {
        ops::set_visible(w, id, false);
    }
}

/// `BeginConceal()`.
pub fn begin_conceal(w: &mut World, id: ItemId) {
    if let Some(mut h) = td_mut(w, id).map(|d| std::mem::take(&mut d.conceal)) {
        h.cancel(w);
    }
    let period = TimeSpan::from_mins(1); // ConcealPeriod
    let h = timer::start(w, period, TimeSpan::ZERO, 1, move |w| conceal(w, id));
    if let Some(d) = td_mut(w, id) {
        d.conceal = h;
    }
}

fn check_range(w: &World, id: ItemId, loc: Point3D, range: i32) -> bool {
    let Some(z) = w.item(id).map(|i| i.location.z) else {
        return false;
    };
    let me = ops::get_world_location(w, id);
    z + 8 >= loc.z
        && loc.z + 16 > z
        && (me.x - loc.x).abs() <= range
        && (me.y - loc.y).abs() <= range
}

/// `IsEnemy(mob)`.
fn is_enemy(w: &mut World, id: ItemId, mob: MobileId) -> bool {
    let Some(x) = w.mobile(mob) else { return false };
    if x.hidden && x.access_level > AccessLevel::Player {
        return false;
    }
    if !muo_server::mobile::base::alive(w, mob)
        || w.mobile(mob)
            .is_some_and(|m| (m.ty.is_dead_bonded_pet)(w, mob))
    {
        return false;
    }
    let mut faction = find_full(w, mob, true, false);
    if faction.is_none() {
        faction = super::mobiles::guard_faction(w, mob);
    }
    faction.is_some_and(|f| Some(f) != td(w, id).and_then(|d| d.faction))
}

/// `OnTrigger(from)`.
fn on_trigger(w: &mut World, id: ItemId, from: MobileId) {
    if !is_enemy(w, id, from) {
        return;
    }
    conceal(w, id);
    let k = kind(w, id);
    let which = td(w, id).map_or(0, |d| d.which);
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    // DoVisibleEffect
    match which {
        0 => muo_server::effects::send_location_effect(w, loc, map, 0x36BD, 15, 10, 0, 0),
        1 => muo_server::effects::send_location_effect(
            w,
            w.item(id).map(|i| i.location).unwrap_or(loc),
            map,
            0x3709,
            28,
            10,
            0x1D3,
            5,
        ),
        2 => muo_server::effects::send_location_effect(
            w,
            w.item(id).map(|i| i.location).unwrap_or(loc),
            map,
            0x11AD,
            25,
            10,
            0,
            0,
        ),
        _ => muo_server::effects::send_location_effect(
            w,
            w.item(id).map(|i| i.location).unwrap_or(loc),
            map,
            0x11A4,
            12,
            6,
            0,
            0,
        ),
    }
    muo_server::effects::play_sound(
        w,
        w.item(id).map(|i| i.location).unwrap_or(loc),
        map,
        k.sound,
    );
    // DoAttackEffect
    if which == 1 {
        if let Some(p) = muo_content::misc::poison_kinds::by_name(w, "Lethal") {
            muo_server::mobile::base::apply_poison(w, from, Some(from), p);
        }
    } else {
        let amount = utility::dice(&mut w.rng, 6, 10, 40);
        muo_server::mobile::part2::damage::damage(w, from, amount, Some(from), true, false);
    }

    let silver_to_award = if muo_server::mobile::base::alive(w, from) {
        20
    } else {
        40
    };
    let (placer, faction) = td(w, id).map_or((None, None), |d| (d.placer, d.faction));
    if let (Some(placer), Some(f)) = (placer, faction) {
        let victim = super::player_state(w, from).map(|(_, p)| p.kill_points);
        let can = super::can_give_silver_pub(w, from, placer);
        if can && victim.is_some_and(|k| k > 0) {
            let given = super::award_silver(w, f, placer, silver_to_award);
            if given > 0 {
                let name = muo_server::mobile::base::name(w, from).unwrap_or_default();
                if muo_server::mobile::base::alive(w, from) {
                    mm::send_message(
                        w,
                        placer,
                        &format!(
                            "You have earned {given} silver pieces because {name} fell for your trap."
                        ),
                    );
                } else {
                    // You have earned ~1_SILVER_AMOUNT~ pieces for vanquishing ~2_PLAYER_NAME~!
                    mm::send_localized_message(
                        w,
                        placer,
                        1042736,
                        &format!("{given} silver\t{name}"),
                        0x3B2,
                    );
                }
            }
            super::given_silver_to_pub(w, from, placer);
        }
    }
    mm::local_overhead_message_number(w, from, MessageType::REGULAR, k.hue, k.attack_message, "");
}

fn on_movement(w: &mut World, id: ItemId, m: MobileId, old: Point3D) {
    (ITEM_BASE.on_movement)(w, id, m, old);
    // BaseTrap.OnMovement
    let loc = w.mobile(m).map(|x| x.location).unwrap_or_default();
    if loc == old {
        return;
    }
    let now = w.now;
    if check_range(w, id, loc, 0)
        && !check_range(w, id, old, 0)
        && td(w, id).is_some_and(|d| now >= d.next_active)
    {
        if let Some(d) = td_mut(w, id) {
            d.next_active = now; // + ResetDelay (zero)
        }
        on_trigger(w, id, m);
    }
    // BaseFactionTrap.OnMovement
    if !check_decay(w, id) && check_range(w, id, loc, 6) && !check_range(w, id, old, 6) {
        let detect = muo_server::skills::value(w, m, muo_server::skills::SkillName::DetectHidden);
        if find(w, m).is_some() && (detect - 80.0) / 20.0 > rnd::double(w) {
            let k = kind(w, id);
            if let Some(ns) = w.mobile(m).and_then(|x| x.net_state) {
                let (serial, graphic) = w
                    .item(id)
                    .map(|i| (i.serial, i.item_id))
                    .unwrap_or_default();
                muo_server::packets::message::send_message_localized(
                    w,
                    ns,
                    serial,
                    graphic,
                    muo_server::packets::message::MessageType::REGULAR,
                    k.hue,
                    3,
                    1010154,
                    "",
                    "",
                ); // [Faction Trap]
            }
        }
    }
}

fn trap_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(0); // BaseTrap version
    out.write_i32(0); // BaseFactionTrap version
    if let Some(d) = td(w, id) {
        write_faction_ref(out, d.faction);
        out.write_mobile(w, d.placer);
        out.write_date_time_ticks(d.time_of_placement.ticks());
    }
    out.write_i32(0); // generated version
}

fn trap_deserialize(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
    k: &'static Kind,
    which: u8,
) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _v = r.read_i32()?;
    let _v = r.read_i32()?;
    let faction = read_faction_ref(r)?;
    let placer = r.read_mobile(w)?;
    let placed = rd(r)?;
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(TrapData {
            faction,
            placer,
            time_of_placement: placed,
            next_active: DateTime::MIN,
            conceal: TimerHandle::default(),
            kind: k,
            which,
        });
    }
    let _v = r.read_i32()?;
    if w.item(id).is_some_and(|i| i.visible()) {
        begin_conceal(w, id);
    }
    check_decay(w, id);
    Ok(())
}

fn trap_on_delete(w: &mut World, id: ItemId) {
    if let Some(f) = td(w, id).and_then(|d| d.faction) {
        st_mut(w).factions[f].traps.retain(|&t| t != id);
    }
    (ITEM_BASE.on_delete)(w, id);
}

macro_rules! trap {
    ($name:ident, $kind:ident, $which:literal, $ctor:ident, $full:literal) => {
        fn $ctor(w: &mut World) -> ItemId {
            new_trap(w, &$name, &$kind, $which, None, None)
        }
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: Some($ctor),
            label_number: |_, _| $kind.label,
            handles_on_movement: |_, _| true,
            on_movement,
            on_delete: trap_on_delete,
            serialize: trap_serialize,
            deserialize: |w, id, r| trap_deserialize(w, id, r, &$kind, $which),
            ..ITEM_BASE
        };
    };
}

trap!(
    FACTION_EXPLOSION_TRAP,
    EXPLOSION,
    0,
    explosion_new,
    "Server.Factions.FactionExplosionTrap"
);
trap!(
    FACTION_GAS_TRAP,
    GAS,
    1,
    gas_new,
    "Server.Factions.FactionGasTrap"
);
trap!(
    FACTION_SAW_TRAP,
    SAW,
    2,
    saw_new,
    "Server.Factions.FactionSawTrap"
);
trap!(
    FACTION_SPIKE_TRAP,
    SPIKE,
    3,
    spike_new,
    "Server.Factions.FactionSpikeTrap"
);

// ---- IsValidLocation ----

/// `IsValidLocation(p, m)`: 0, or the cliloc number of why not.
fn is_valid_location(
    w: &World,
    f: Option<usize>,
    which: u8,
    p: Point3D,
    map: Option<muo_server::map::MapId>,
) -> i32 {
    let Some(map_id) = map else {
        return 502956; // You cannot place a trap on that.
    };
    if muo_content::core::ml(w) {
        for item in muo_server::map::enumerate::items_at(w, Some(map_id), p.x, p.y) {
            if td(w, item).is_some_and(|d| d.faction == f) {
                return 1075263; // There is already a trap belonging to your faction at this location.
            }
        }
    }
    let region = Some(muo_server::region::find(w, p, Some(map_id)));
    let placing = match which {
        0 => EXPLOSION.placing,
        1 => GAS.placing,
        2 => SAW.placing,
        _ => SPIKE.placing,
    };
    match placing {
        Placing::FactionStronghold => {
            if let Some(region) = region
                && f.is_some_and(|f| super::stronghold_region_faction(w, region) == Some(f))
            {
                return 0;
            }
            1010355 // This trap can only be placed in your stronghold
        }
        Placing::AnyFactionTown => {
            if region.and_then(|r| town::from_region(w, r)).is_some() {
                0
            } else {
                1010356
            }
        }
        Placing::ControlledFactionTown => {
            let t = region.and_then(|r| town::from_region(w, r));
            if t.is_some_and(|t| town::owner(w, t) == f) {
                0
            } else {
                1010357
            }
        }
    }
}

// ---- deeds ----

/// `BaseFactionTrapDeed._faction`.
#[derive(Default)]
pub struct DeedData {
    pub faction: Option<usize>,
    which: u8,
}

pub fn deed_set_faction(w: &mut World, id: ItemId, f: Option<usize>) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<DeedData>()) {
        d.faction = f;
    }
    if let Some(f) = f {
        base::set_hue(w, id, FACTION_DEFS[f].hue_primary);
    }
}

fn new_deed(w: &mut World, ty: &'static ItemType, k: &'static Kind, which: u8) -> ItemId {
    let id = ops::new_item_with_id(w, ty, k.deed_item_id);
    ops::set_loot_type(w, id, LootType::Blessed);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(DeedData {
            faction: None,
            which,
        });
    }
    id
}

/// `BaseFactionTrapDeed.OnCraft`: `ItemID = 0x14F0; Faction = Faction.Find(from)`.
pub fn craft_deed(w: &mut World, deed: ItemId, from: MobileId) {
    base::set_item_id(w, deed, 0x14F0);
    let f = find(w, from);
    deed_set_faction(w, deed, f);
}

fn deed_on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let (own, which) = w
        .item(id)
        .and_then(|i| i.ext.get::<DeedData>())
        .map_or((None, 0), |d| (d.faction, d.which));
    let msg = |w: &mut World, n: i32| mm::send_localized_message(w, from, n, "", 0x23);
    let Some(faction) = find(w, from) else {
        msg(w, 1010353); // Only faction members may place faction traps
        return;
    };
    if Some(faction) != own {
        msg(w, 1010354); // You may only place faction traps created by your faction
    } else if st(w).is_some_and(|s| s.factions[faction].traps.len() >= 15) {
        msg(w, 1010358); // Your faction already has the maximum number of traps placed
    } else {
        let (k, ty): (&'static Kind, &'static ItemType) = match which {
            0 => (&EXPLOSION, &FACTION_EXPLOSION_TRAP),
            1 => (&GAS, &FACTION_GAS_TRAP),
            2 => (&SAW, &FACTION_SAW_TRAP),
            _ => (&SPIKE, &FACTION_SPIKE_TRAP),
        };
        let (loc, map) = w
            .mobile(from)
            .map(|m| (m.location, m.map))
            .unwrap_or_default();
        let message = is_valid_location(w, Some(faction), which, loc, map);
        let trap = new_trap(w, ty, k, which, Some(faction), Some(from));
        if message > 0 {
            msg(w, message);
            if let Some(t) = w.item(trap).map(|i| i.ty) {
                (t.delete)(w, trap);
            }
        } else {
            msg(w, 1010360); // You arm the trap and carefully hide it from view
            ops::move_to_world_map(w, trap, loc, map);
            st_mut(w).factions[faction].traps.push(trap);
            if let Some(t) = w.item(id).map(|i| i.ty) {
                (t.delete)(w, id);
            }
        }
    }
}

fn deed_serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(0); // BaseFactionTrapDeed version
    write_faction_ref(
        out,
        w.item(id)
            .and_then(|i| i.ext.get::<DeedData>())
            .and_then(|d| d.faction),
    );
    out.write_i32(0); // generated version
}

fn deed_deserialize(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
    which: u8,
) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _v = r.read_i32()?;
    let f = read_faction_ref(r)?;
    let _v = r.read_i32()?;
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(DeedData { faction: f, which });
    }
    Ok(())
}

macro_rules! deed {
    ($name:ident, $kind:ident, $trap:ident, $which:literal, $ctor:ident, $full:literal) => {
        fn $ctor(w: &mut World) -> ItemId {
            new_deed(w, &$name, &$kind, $which)
        }
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: None, // `FactionTrapDeed` has no parameterless constructor
            make: Some($ctor),
            label_number: |_, _| $kind.deed_label,
            default_weight: |_, _| 1.0,
            on_double_click: deed_on_double_click,
            serialize: deed_serialize,
            deserialize: |w, id, r| deed_deserialize(w, id, r, $which),
            ..ITEM_BASE
        };
    };
}

deed!(
    FACTION_EXPLOSION_TRAP_DEED,
    EXPLOSION,
    FACTION_EXPLOSION_TRAP,
    0,
    explosion_deed_new,
    "Server.Factions.FactionExplosionTrapDeed"
);
deed!(
    FACTION_GAS_TRAP_DEED,
    GAS,
    FACTION_GAS_TRAP,
    1,
    gas_deed_new,
    "Server.Factions.FactionGasTrapDeed"
);
deed!(
    FACTION_SAW_TRAP_DEED,
    SAW,
    FACTION_SAW_TRAP,
    2,
    saw_deed_new,
    "Server.Factions.FactionSawTrapDeed"
);
deed!(
    FACTION_SPIKE_TRAP_DEED,
    SPIKE,
    FACTION_SPIKE_TRAP,
    3,
    spike_deed_new,
    "Server.Factions.FactionSpikeTrapDeed"
);

// ---- FactionTrapRemovalKit ----

/// `FactionTrapRemovalKit._charges`.
pub struct KitData {
    pub charges: i32,
}

fn kit_new(w: &mut World) -> ItemId {
    let id = ops::new_item_with_id(w, &FACTION_TRAP_REMOVAL_KIT, 7867);
    ops::set_loot_type(w, id, LootType::Blessed);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(KitData { charges: 25 });
    }
    id
}

/// `ConsumeCharge(consumer)`.
pub fn kit_consume_charge(w: &mut World, id: ItemId, consumer: MobileId) {
    let left = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<KitData>())
        .map(|d| {
            d.charges -= 1;
            d.charges
        })
        .unwrap_or(0);
    if left <= 0 {
        if let Some(t) = w.item(id).map(|i| i.ty) {
            (t.delete)(w, id);
        }
        mm::send_localized_message(w, consumer, 1042531, "", 0x3B2); // You have used all of the parts in your trap removal kit.
    } else {
        ops::invalidate_properties(w, id);
    }
}

fn kit_get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    let n = w
        .item(id)
        .and_then(|i| i.ext.get::<KitData>())
        .map_or(0, |d| d.charges);
    list.add_int(1060584, n); // uses remaining: ~1_val~
}

pub static FACTION_TRAP_REMOVAL_KIT: ItemType = ItemType {
    full_name: "Server.Factions.FactionTrapRemovalKit",
    parent: Some(&ITEM_BASE),
    construct: Some(kit_new),
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", ITEM_BASE.label_number)
    },
    get_properties: kit_get_properties,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_encoded_int(
            w.item(id)
                .and_then(|i| i.ext.get::<KitData>())
                .map_or(0, |d| d.charges),
        );
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let charges = r.read_encoded_int()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(KitData { charges });
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- skill hooks ----

/// `RemoveTrap`'s `BaseFactionTrap` branch.
fn disarm(w: &mut World, from: MobileId, trap: ItemId) {
    let msg = |w: &mut World, n: i32| mm::send_localized_message(w, from, n, "", 0x3B2);
    let faction = find(w, from);
    let kit = w.mobile(from).and_then(|m| m.backpack).and_then(|p| {
        muo_server::item::enumerable::find_item_by_type(w, p, true, &FACTION_TRAP_REMOVAL_KIT)
    });
    let (placer, tf) = td(w, trap).map_or((None, None), |d| (d.placer, d.faction));
    let is_owner = placer == Some(from) || tf.is_some_and(|f| super::is_commander(w, f, from));
    let Some(faction) = faction else {
        msg(w, 1010538); // You may not disarm faction traps unless you are in an opposing faction
        return;
    };
    if tf == Some(faction) && !is_owner {
        msg(w, 1010537); // You may not disarm traps set by your own faction!
    } else if !is_owner && kit.is_none() {
        msg(w, 1042530); // You must have a trap removal kit at the base level of your pack to disarm a faction trap.
    } else {
        use muo_server::skills::SkillName;
        let target = muo_server::target::Targeted::Entity(EntityId::Item(trap));
        let ok = (muo_content::core::ml(w) && is_owner)
            || (muo_content::skills::common::check_target_skill(
                w,
                from,
                SkillName::RemoveTrap,
                target,
                80.0,
                100.0,
            ) && muo_content::skills::common::check_target_skill(
                w,
                from,
                SkillName::Tinkering,
                target,
                80.0,
                100.0,
            ));
        if ok {
            let k = kind(w, trap);
            let ns = w.mobile(from).and_then(|m| m.net_state);
            mm::private_overhead_message_number(
                w,
                from,
                MessageType::REGULAR,
                k.hue,
                k.disarm_message,
                "",
                ns,
            );
            if !is_owner {
                let silver = super::award_silver(w, faction, from, 100);
                if silver > 0 {
                    // You have been granted faction silver for removing the enemy trap :
                    mm::send_localized_message(
                        w,
                        from,
                        1008113,
                        &muo_content::items::gold::n0(silver as i64),
                        0x3B2,
                    );
                }
            }
            if let Some(t) = w.item(trap).map(|i| i.ty) {
                (t.delete)(w, trap);
            }
        } else {
            msg(w, 502372); // You fail to disarm the trap... but you don't set it off
        }
        if !is_owner && let Some(kit) = kit {
            kit_consume_charge(w, kit, from);
        }
    }
}

/// `DetectHidden`'s reveal of a faction trap.
fn detect(w: &mut World, src: MobileId, trap: ItemId) -> bool {
    use muo_server::skills::SkillName;
    let target = muo_server::target::Targeted::Entity(EntityId::Item(trap));
    if find(w, src).is_some()
        && muo_content::skills::common::check_target_skill(
            w,
            src,
            SkillName::DetectHidden,
            target,
            80.0,
            100.0,
        )
    {
        let name = td(w, trap)
            .and_then(|d| d.faction)
            .map_or("", |f| FACTION_DEFS[f].friendly_name);
        // You reveal a trap placed by a faction:
        mm::send_localized_message_affix(w, src, 1042712, true, &format!(" {name}"), "", 0x3B2);
        ops::set_visible(w, trap, true);
        begin_conceal(w, trap);
        return true;
    }
    false
}

pub fn configure(w: &mut World) {
    let f = &mut w.resources.get_mut::<muo_content::skills::SkillHooks>().fns;
    f.is_faction_trap = Some(is_trap);
    f.detect_faction_trap = Some(detect);
    f.disarm_faction_trap = Some(disarm);
    f.faction_in_skill_loss = Some(super::in_skill_loss);
    muo_content::craft::hooks_mut(w)
        .craftables
        .push(|w, id, a| {
            w.item(id)?.ext.get::<DeedData>()?;
            craft_deed(w, id, a.from);
            Some(a.quality)
        });
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &FACTION_EXPLOSION_TRAP,
        &FACTION_GAS_TRAP,
        &FACTION_SAW_TRAP,
        &FACTION_SPIKE_TRAP,
        &FACTION_EXPLOSION_TRAP_DEED,
        &FACTION_GAS_TRAP_DEED,
        &FACTION_SAW_TRAP_DEED,
        &FACTION_SPIKE_TRAP_DEED,
        &FACTION_TRAP_REMOVAL_KIT,
    ] {
        types.register_item(t, &[]);
    }
}
