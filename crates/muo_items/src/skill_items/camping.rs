//! `Bedroll`, `Kindling` and `Campfire`.
//!
//! The statics of `Campfire` (`_table`, each fire's `_entries` and `_timerToken`) are [`CampState`] in the world's
//! resources: nothing here is saved (`Campfire.SkipSerialization`).

use indexmap::IndexMap;
use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::gumps::{close_gump, has_gump};
use muo_content::player_mobile;
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_primitives::{DateTime, Direction, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::EntityId;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::light_type::LightType;
use muo_server::net_state::NetStateId;
use muo_server::packets::message::MessageType;
use muo_server::skills::SkillName;
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{read_version, write_version};

/// `Campfire.SecureRange`.
const SECURE_RANGE: i32 = 7;

/// `CampfireStatus`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CampfireStatus {
    Burning,
    Extinguishing,
    Off,
}

/// `CampfireEntry` (the player is the key of the table).
#[derive(Clone, Copy, Debug)]
pub struct CampfireEntry {
    pub fire: ItemId,
    pub start: DateTime,
    safe: bool,
}

#[derive(Default)]
struct Fire {
    timer: TimerHandle,
    entries: Vec<MobileId>,
}

/// `Campfire._table` and the per-fire state.
#[derive(Default)]
pub struct CampState {
    table: IndexMap<MobileId, CampfireEntry>,
    fires: IndexMap<ItemId, Fire>,
}

fn state(w: &mut World) -> &mut CampState {
    w.resources.get_mut::<CampState>()
}

/// `Campfire.GetEntry(player)`.
pub fn get_entry(w: &World, player: MobileId) -> Option<CampfireEntry> {
    w.resources.get::<CampState>()?.table.get(&player).copied()
}

/// `CampfireEntry.Valid`.
fn entry_valid(w: &World, player: MobileId, e: &CampfireEntry) -> bool {
    let Some(f) = w.item(e.fire).filter(|f| !f.deleted()) else {
        return false;
    };
    status_of(f.item_id) != CampfireStatus::Off
        && w.mobile(player).is_some_and(|p| p.map == f.map)
        && deps::mobile_in_range(w, player, f.location, SECURE_RANGE)
}

/// `CampfireEntry.Safe`.
pub fn entry_safe(w: &World, player: MobileId, e: &CampfireEntry) -> bool {
    entry_valid(w, player, e) && e.safe
}

/// `Campfire.RemoveEntry(entry)`.
pub fn remove_entry(w: &mut World, player: MobileId) {
    let s = state(w);
    if let Some(e) = s.table.shift_remove(&player)
        && let Some(f) = s.fires.get_mut(&e.fire)
    {
        f.entries.retain(|m| *m != player);
    }
}

fn status_of(item_id: i32) -> CampfireStatus {
    match item_id {
        0xDE3 => CampfireStatus::Burning,
        0xDE9 => CampfireStatus::Extinguishing,
        _ => CampfireStatus::Off,
    }
}

/// `Campfire.Status`.
pub fn status(w: &World, fire: ItemId) -> CampfireStatus {
    w.item(fire)
        .map_or(CampfireStatus::Off, |i| status_of(i.item_id))
}

/// `Status = value`.
pub fn set_status(w: &mut World, fire: ItemId, value: CampfireStatus) {
    if status(w, fire) == value {
        return;
    }
    let Some(ty) = w.item(fire).map(|i| i.ty) else {
        return;
    };
    match value {
        CampfireStatus::Burning => {
            (ty.set_item_id)(w, fire, 0xDE3);
            ops::set_light(w, fire, LightType::Circle300);
        }
        CampfireStatus::Extinguishing => {
            (ty.set_item_id)(w, fire, 0xDE9);
            ops::set_light(w, fire, LightType::Circle150);
        }
        CampfireStatus::Off => {
            (ty.set_item_id)(w, fire, 0xDEA);
            ops::set_light(w, fire, LightType::ArchedWindowEast);
            clear_entries(w, fire);
        }
    }
}

/// `ClearEntries()`.
fn clear_entries(w: &mut World, fire: ItemId) {
    let s = state(w);
    let Some(f) = s.fires.get_mut(&fire) else {
        return;
    };
    let gone = std::mem::take(&mut f.entries);
    for m in gone {
        s.table.shift_remove(&m);
    }
}

/// `new Campfire()`.
pub fn new_campfire(w: &mut World) -> ItemId {
    let id = ops::new_item_with_id(w, &CAMPFIRE, 0xDE3);
    ops::set_movable(w, id, false);
    ops::set_light(w, id, LightType::Circle300);
    let timer = timer::start(
        w,
        TimeSpan::from_secs(1),
        TimeSpan::from_secs(1),
        0,
        move |w| on_tick(w, id),
    );
    state(w).fires.insert(
        id,
        Fire {
            timer,
            entries: Vec::new(),
        },
    );
    id
}

fn on_tick(w: &mut World, fire: ItemId) {
    let Some(created) = w.item(fire).map(|i| i.created) else {
        return;
    };
    let now = w.now;
    let age = (now - created).total_seconds();
    if age >= 100.0 {
        base::delete(w, fire);
    } else if age >= 90.0 {
        set_status(w, fire, CampfireStatus::Off);
    } else if age >= 60.0 {
        set_status(w, fire, CampfireStatus::Extinguishing);
    }

    if status(w, fire) == CampfireStatus::Off || w.item(fire).is_none_or(|i| i.deleted()) {
        return;
    }

    let entries: Vec<MobileId> = state(w)
        .fires
        .get(&fire)
        .map(|f| f.entries.clone())
        .unwrap_or_default();
    for player in entries.into_iter().rev() {
        let Some(e) = get_entry(w, player) else {
            continue;
        };
        let has_state = w.mobile(player).is_some_and(|p| p.net_state.is_some());
        if !entry_valid(w, player, &e) || !has_state {
            remove_entry(w, player);
        } else if !entry_safe(w, player, &e) && (now - e.start).total_seconds() >= 30.0 {
            if let Some(x) = state(w).table.get_mut(&player) {
                x.safe = true;
            }
            deps::send_localized_message(w, player, 500621); // The camp is now secure.
        }
    }

    let (map, loc) = {
        let i = w.item(fire).expect("checked above");
        (i.map, i.location)
    };
    let Some(map) = map else { return };
    for (_, m) in deps::clients_in_range(w, map, loc, SECURE_RANGE) {
        if w.mobile(m).is_some_and(|p| p.player) && get_entry(w, m).is_none() {
            let entry = CampfireEntry {
                fire,
                start: now,
                safe: false,
            };
            let s = state(w);
            s.table.insert(m, entry);
            if let Some(f) = s.fires.get_mut(&fire) {
                f.entries.push(m);
            }
            deps::send_localized_message(w, m, 500620); // You feel it would take a few moments to secure your camp.
        }
    }
}

/// `Campfire.OnAfterDelete()`.
fn campfire_after_delete(w: &mut World, id: ItemId) {
    base::on_after_delete_base(w, id);
    clear_entries(w, id);
    if let Some(mut f) = state(w).fires.shift_remove(&id) {
        f.timer.cancel(w);
    }
}

/// `Server.Items.Campfire`: never saved.
pub static CAMPFIRE: ItemType = ItemType {
    full_name: "Server.Items.Campfire",
    parent: Some(&ITEM_BASE),
    construct: None,
    skip_serialization: |_, _| true,
    on_after_delete: campfire_after_delete,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Kindling ----

/// `new Kindling(amount)`.
pub fn new_kindling(w: &mut World, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &KINDLING, 0xDE1);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `AddOffsetLocation(from, offsetX, offsetY, list)`.
fn add_offset_location(w: &mut World, from: MobileId, ox: i32, oy: i32, list: &mut Vec<Point3D>) {
    let Some((map, loc)) = w
        .mobile(from)
        .and_then(|m| m.map.map(|map| (map, m.location)))
    else {
        return;
    };
    let (x, y) = (loc.x + ox, loc.y + oy);
    let p = Point3D::new(x, y, loc.z);
    if muo_server::map::fit::can_fit_point(w, map, p, 1) && deps::in_los_point(w, from, p) {
        list.push(p);
    } else {
        let z = w.maps.get_average_z(map, x, y);
        let p = Point3D::new(x, y, z);
        if muo_server::map::fit::can_fit_point(w, map, p, 1) && deps::in_los_point(w, from, p) {
            list.push(p);
        }
    }
}

/// `GetFireLocation(from)`: `None` is `Point3D.Zero`.
fn get_fire_location(w: &mut World, id: ItemId, from: MobileId) -> Option<Point3D> {
    let region = muo_server::mobile::part1::props_b::region(w, from);
    if muo_content::regions::is_part_of_dungeon(w, region) {
        return None;
    }
    let item = w.item(id)?;
    if item.parent.is_none() {
        return Some(item.location);
    }
    let mut list = Vec::with_capacity(4);
    add_offset_location(w, from, 0, -1, &mut list);
    add_offset_location(w, from, -1, 0, &mut list);
    add_offset_location(w, from, 0, 1, &mut list);
    add_offset_location(w, from, 1, 0, &mut list);
    utility::random_element(&mut w.rng, &list).copied()
}

fn kindling_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !base::verify_move(w, id, from) {
        return;
    }
    let loc = ops::get_world_location(w, id);
    if !deps::mobile_in_range(w, from, loc, 2) {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1019045); // I can't reach that.
        return;
    }
    let fire_location = get_fire_location(w, id, from);
    let map = w.mobile(from).and_then(|m| m.map);
    match fire_location {
        None => deps::send_localized_message(w, from, 501695), // There is not a spot nearby to place your campfire.
        Some(_)
            if !muo_server::mobile::part2::stats::check_skill(
                w,
                from,
                SkillName::Camping,
                0.0,
                100.0,
            ) =>
        {
            deps::send_localized_message(w, from, 501696); // You fail to ignite the campfire.
        }
        Some(p) => {
            base::consume(w, id, 1);
            if w.item(id)
                .is_some_and(|i| !i.deleted() && i.parent.is_none())
            {
                muo_server::mobile::part2::items::place_in_backpack(w, from, id);
            }
            let fire = new_campfire(w);
            ops::move_to_world_map(w, fire, p, map);
        }
    }
}

/// `Server.Items.Kindling`.
pub static KINDLING: ItemType = ItemType {
    full_name: "Server.Items.Kindling",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_kindling(w, 1)),
    default_weight: |_, _| 5.0,
    on_double_click: kindling_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Bedroll ----

/// `Bedroll.LogoutGump`.
pub struct LogoutGump {
    player: MobileId,
    bedroll: ItemId,
    timer: TimerHandle,
}

/// `new LogoutGump(entry, bedroll)`: closes itself after ten seconds.
fn new_logout_gump(w: &mut World, player: MobileId, bedroll: ItemId) -> StaticGumpObj<LogoutGump> {
    let timer = timer::start(w, TimeSpan::from_secs(10), TimeSpan::ZERO, 1, move |w| {
        // CloseGump()
        remove_entry(w, player);
        if let Some(ns) = w.mobile(player).and_then(|m| m.net_state) {
            close_gump::<LogoutGump>(w, ns);
        }
    });
    StaticGumpObj::new(
        100,
        0,
        LogoutGump {
            player,
            bedroll,
            timer,
        },
    )
}

impl StaticGumpLogic for LogoutGump {
    const TYPE_NAME: &'static str = "Server.Items.Bedroll+LogoutGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 400, 350, 0xA28);
        b.add_html_localized(100, 20, 200, 35, 1011015, false, false); // <center>Logging out via camping</center>

        /* Using a bedroll in the safety of a camp will log you out of the game safely.
         * If this is what you wish to do choose CONTINUE and you will be logged out.
         * Otherwise, select the CANCEL button to avoid logging out at this time.
         * The camp will remain secure for 10 seconds at which time this window will close
         * and you not be logged out.
         */
        b.add_html_localized(50, 55, 300, 140, 1011016, true, true);
        b.add_button(45, 298, 0xFA5, 0xFA7, 1);
        b.add_html_localized(80, 300, 110, 35, 1011011, false, false); // CONTINUE
        b.add_button(200, 298, 0xFA5, 0xFA7, 0);
        b.add_html_localized(235, 300, 110, 35, 1011012, false, false); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let (pm, bedroll) = (self.player, self.bedroll);
        let entry = get_entry(w, pm);
        self.timer.cancel(w);
        let Some(entry) = entry else { return };
        // `Campfire.GetEntry(pm) != _entry`: the entry the gump was made for is still the player's.
        if info.button_id == 1
            && entry_safe(w, pm, &entry)
            && w.item(bedroll)
                .is_some_and(|i| !i.deleted() && i.parent.is_none())
            && base::is_accessible_to(w, bedroll, pm)
            && base::verify_move(w, bedroll, pm)
            && w.item(bedroll).map(|i| i.map) == w.mobile(pm).map(|m| m.map)
            && deps::mobile_in_range(w, pm, ops::get_world_location(w, bedroll), 2)
        {
            muo_server::mobile::part2::items::place_in_backpack(w, pm, bedroll);
            if let Some(d) = player_mobile::data_mut(w, pm) {
                d.bedroll_logout = true;
            }
            w.disconnect(sender, "Used a bedroll to log out.");
        }
        remove_entry(w, pm);
    }
}

fn bedroll_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if w.item(id).is_none_or(|i| i.parent.is_some()) || !base::verify_move(w, id, from) {
        return;
    }
    let (loc, item_id) = match w.item(id) {
        Some(i) => (i.location, i.item_id),
        None => return,
    };
    if !deps::mobile_in_range(w, from, loc, 2) {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1019045); // I can't reach that.
        return;
    }
    let Some(ty) = w.item(id).map(|i| i.ty) else {
        return;
    };
    if item_id == 0xA57 {
        // rolled
        let from_loc = w.mobile(from).map_or(Point3D::default(), |m| m.location);
        let dir = player_mobile::get_direction4(from_loc, loc);
        (ty.set_item_id)(
            w,
            id,
            if dir == Direction::NORTH || dir == Direction::SOUTH {
                0xA55
            } else {
                0xA56
            },
        );
    } else {
        // unrolled
        (ty.set_item_id)(w, id, 0xA57);
        let ns = w.mobile(from).and_then(|m| m.net_state);
        if let Some(ns) = ns
            && !has_gump::<LogoutGump>(w, ns)
            && let Some(entry) = get_entry(w, from)
            && entry_safe(w, from, &entry)
        {
            let g = new_logout_gump(w, from, id);
            super::hue_pickers::send(w, from, g);
        }
    }
}

/// `Server.Items.Bedroll`.
pub static BEDROLL: ItemType = ItemType {
    full_name: "Server.Items.Bedroll",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| ops::new_item_with_id(w, &BEDROLL, 0xA57)),
    default_weight: |_, _| 5.0,
    on_double_click: bedroll_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, true);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, true)?;
        Ok(())
    },
    ..ITEM_BASE
};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
static KINDLING_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: INT,
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_kindling(w, v[0].i32())),
}];

fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.Kindling", &KINDLING_CTOR);
}

/// Registers the camping classes.
pub fn register(types: &mut TypeRegistry) {
    for t in [&BEDROLL, &KINDLING, &CAMPFIRE] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.Kindling.Configure",
        configure,
    );
}
