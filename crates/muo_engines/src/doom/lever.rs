//! The Doom lever puzzle: `LeverPuzzleController`, `LampRoomBox`, `LeverPuzzleStatue`,
//! `LeverPuzzleLever`, `LampRoomTeleporter` and the two regions.
//!
//! The `WandererOfTheVoid` the lamp room box summons is made by name (a creature of muo_mobiles).

use std::cell::Cell;
use std::rc::Rc;

use muo_content::creature::cd;
use muo_content::factory;
use muo_content::hooks::{AosDamage, rnd};
use muo_content::player_mobile as pmob;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::effects::{self, Targeted as FxTarget};
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::packets::message::MessageType;
use muo_server::poison::PoisonId;
use muo_server::region::{self, RegionId, RegionType};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use crate::util::hue;

const MALAS: MapId = MapId::MALAS;

/// `TA`: 3D coords for the levers, 2D coords for the standing regions, then teleporter/statue/box rows.
static LEVER_COORDS: [[i32; 3]; 4] = [[316, 64, 5], [323, 58, 5], [332, 63, 5], [323, 71, 5]];
static TILE_COORDS: [[i32; 2]; 5] = [[324, 64], [316, 65], [324, 58], [332, 64], [323, 72]];
/// 3D coord, then item id and hue for the lamp room teleporters.
static TELES: [([i32; 3], [i32; 2]); 3] = [
    ([468, 92, -1], [0x181D, 0x482]),
    ([469, 92, -1], [0x1821, 0x3fd]),
    ([470, 92, -1], [0x1825, 0x66d]),
];
/// 3D coord and item id for the statues.
static STATUES: [([i32; 3], i32); 2] = [([319, 70, 18], 0x12d8), ([329, 60, 18], 0x12d9)];
static BOX_COORD: [i32; 3] = [469, 96, 6];

/// `Msgs` and `MsgParams` (hue, font).
static MSGS: [(&str, i32, i32); 3] = [
    (
        "You are pinned down by the weight of the boulder!!!",
        0x66d,
        3,
    ),
    ("A speeding rock hits you in the head!", 0x66d, 3),
    ("OUCH!", 0x34, 3),
];
/// `Statue_Msg`.
static STATUE_MSG: [i32; 4] = [1050009, 1050007, 1050008, 1050008];

pub const LR_EXIT: Point3D = Point3D::new(353, 172, -1);
pub const LR_ENTER: Point3D = Point3D::new(467, 96, -1);
pub const LP_CENTER: Point3D = Point3D::new(324, 64, -1);
pub const LR_RECT: Rectangle2D = Rectangle2D::new(465, 92, 10, 10);

/// `PA`: poison message cliloc, its hue and the effect hue.
static PA: [(i32, i32, i32); 6] = [
    (0, 0, 0xA6),
    (1050001, 0x485, 0xAA),
    (1050003, 0x485, 0xAC),
    (1050056, 0x485, 0xA8),
    (1050057, 0x485, 0xA4),
    (1062091, 0x23F3, 0xAC),
];
/// `PA2`: Lesser, Regular, Greater, Deadly, Lethal, Lethal.
static PA2: [PoisonId; 6] = [
    PoisonId(0),
    PoisonId(1),
    PoisonId(2),
    PoisonId(3),
    PoisonId(4),
    PoisonId(4),
];

static FS: [i32; 2] = [0x144, 0x154];
static MS: [i32; 2] = [0x144, 0x14B];
static FS2: [i32; 2] = [0x13F, 0x154];
static MS2: [i32; 2] = [0x13F, 0x14B];
static CS1: [i32; 1] = [0x244];
static EXP: [i32; 1] = [0x307];

/// `installed`.
#[derive(Default)]
struct Installed(bool);

/// What `LeverPuzzleController` holds.
#[derive(Default)]
pub struct ControllerData {
    pub levers: Vec<Option<ItemId>>,
    pub statues: Vec<Option<ItemId>>,
    pub teles: Vec<Option<ItemId>>,
    pub box_: Option<ItemId>,
    // not serialized
    pub tiles: Vec<RegionId>,
    pub lamp_room: Option<RegionId>,
    pub my_key: u16,
    pub their_key: u16,
    pub enabled: bool,
    pub successful: Option<MobileId>,
    pub reset_timer: TimerHandle,
    pub lamp_timer: TimerHandle,
}

pub fn data(w: &World, id: ItemId) -> Option<&ControllerData> {
    w.item(id)?.ext.get::<ControllerData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut ControllerData> {
    w.item_mut(id)?.ext.get_mut::<ControllerData>()
}

fn deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.delete)(w, id);
    }
}

fn mobile_deleted(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_none_or(|x| x.deleted)
}

fn point(c: &[i32]) -> Point3D {
    Point3D::new(c[0], c[1], c[2])
}

/// `AddLeverPuzzlePart(loc, newitem)`.
fn add_part(w: &mut World, loc: &[i32], item: ItemId) -> ItemId {
    if deleted(w, item) {
        w.resources.get_mut::<Installed>().0 = false;
    } else {
        ops::move_to_world_map(w, item, point(loc), Some(MALAS));
    }
    item
}

// ---- the controller ----

/// `LeverPuzzleController()`.
pub fn controller_new(w: &mut World) -> ItemId {
    let id = ops::new_item_with_id(w, &LEVER_PUZZLE_CONTROLLER, 0x1822);
    ops::set_movable(w, id, false);
    hue(w, id, 0x4c);
    w.resources.get_mut::<Installed>().0 = true;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ControllerData::default());
    }

    let mut levers = Vec::new();
    for (i, loc) in LEVER_COORDS.iter().enumerate() {
        // codes are 0x1 shifted left x # of bits, easily handled here
        let lever = lever_new(w, 1 << i, id);
        levers.push(Some(add_part(w, loc, lever)));
    }
    let mut tiles = Vec::new();
    for loc in &TILE_COORDS {
        tiles.push(new_tile_region(w, loc));
    }
    let mut teles = Vec::new();
    for (loc, dat) in &TELES {
        let t = teleporter_new(w, dat);
        teles.push(Some(add_part(w, loc, t)));
    }
    let mut statues = Vec::new();
    for (loc, graphic) in &STATUES {
        let s = statue_new(w, *graphic, id);
        statues.push(Some(add_part(w, loc, s)));
    }
    if !w.resources.get_mut::<Installed>().0 {
        delete_item(w, id);
    } else if let Some(d) = data_mut(w, id) {
        d.enabled = true;
    }
    let bx = box_new(w, id);
    let bx = add_part(w, &BOX_COORD, bx);
    let lamp = new_lamp_room_region(w, id);
    if let Some(d) = data_mut(w, id) {
        d.levers = levers;
        d.statues = statues;
        d.teles = teles;
        d.tiles = tiles;
        d.box_ = Some(bx);
        d.lamp_room = Some(lamp);
    }
    gen_key(w, id);
    id
}

/// `GenKey()`: shuffles `{ 1, 2, 4, 8 }` and appends each as a nibble.
fn gen_key(w: &mut World, id: ItemId) {
    let mut ca: [u16; 4] = [1, 2, 4, 8];
    // `Span<ushort>.Shuffle()`: `Random.Shuffle(span)`, forward Fisher-Yates.
    muo_primitives::random::Rng::shuffle(&mut w.rng, &mut ca[..]);
    if let Some(d) = data_mut(w, id) {
        for c in ca {
            d.my_key = c | (d.my_key << 4);
        }
    }
}

pub fn enabled(w: &World, id: ItemId) -> bool {
    data(w, id).is_some_and(|d| d.enabled)
}

/// `GetOccupant(index)`.
fn get_occupant(w: &World, id: ItemId, index: usize) -> Option<MobileId> {
    let r = *data(w, id)?.tiles.get(index)?;
    let o = w.region(r)?.ext.get::<Occupant>()?.0?;
    (muo_content::mx::alive(w, o)).then_some(o)
}

fn circle_complete(w: &World, id: ItemId) -> bool {
    (0..5).all(|i| get_occupant(w, id, i).is_some())
}

fn get_statue(w: &World, id: ItemId, index: usize) -> Option<ItemId> {
    data(w, id)?
        .statues
        .get(index)
        .copied()
        .flatten()
        .filter(|s| !deleted(w, *s))
}

fn get_lever(w: &World, id: ItemId, index: usize) -> Option<ItemId> {
    data(w, id)?
        .levers
        .get(index)
        .copied()
        .flatten()
        .filter(|s| !deleted(w, *s))
}

/// `PuzzleStatus(message, fstring)`.
fn puzzle_status(w: &mut World, id: ItemId, message: i32, fstring: Option<&str>) {
    for i in 0..2 {
        if let Some(s) = get_statue(w, id, i) {
            muo_server::item::messages::public_overhead_message_number(
                w,
                s,
                MessageType::REGULAR,
                0x3B2,
                message,
                fstring.unwrap_or(""),
            );
        }
    }
}

fn reset_puzzle(w: &mut World, id: ItemId) {
    puzzle_status(w, id, 1062053, None);
    reset_levers(w, id);
}

fn reset_levers(w: &mut World, id: ItemId) {
    for i in 0..4 {
        if let Some(l) = get_lever(w, id, i) {
            if let Some(t) = w.item(l).map(|x| x.ty) {
                (t.set_item_id)(w, l, 0x108E);
            }
            let (p, m) = w
                .item(l)
                .map_or((Point3D::ZERO, None), |x| (x.location, x.map));
            effects::play_sound(w, p, m, 0x3E8);
        }
    }
    if let Some(d) = data_mut(w, id) {
        d.their_key ^= d.their_key;
    }
}

fn kill_timers(w: &mut World, id: ItemId) {
    let (mut a, mut b) = data(w, id)
        .map(|d| (d.reset_timer, d.lamp_timer))
        .unwrap_or_default();
    a.cancel(w);
    b.cancel(w);
}

fn remove_successful(w: &mut World, id: ItemId) {
    if let Some(d) = data_mut(w, id) {
        d.successful = None;
    }
}

/// `LeverPulled(code)`.
pub fn lever_pulled(w: &mut World, id: ItemId, code: u16) {
    kill_timers(w, id);

    // if one bit in each of the four nibbles is set, this is false
    let their = {
        let Some(d) = data_mut(w, id) else { return };
        d.their_key = code | (d.their_key << 4);
        d.their_key
    };
    if their < 0x0FFF {
        let h = timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
            reset_puzzle(w, id)
        });
        if let Some(d) = data_mut(w, id) {
            d.reset_timer = h;
        }
        return;
    }

    let my = data(w, id).map_or(0, |d| d.my_key);
    if !circle_complete(w, id) {
        puzzle_status(w, id, 1050004, None); // The circle is the key...
    } else if their == my {
        gen_key(w, id);
        let succ = get_occupant(w, id, 0);
        if let Some(d) = data_mut(w, id) {
            d.successful = succ;
        }
        if let Some(s) = succ {
            send_location_effect(w, LP_CENTER, 0x1153, 0, 60, 1);
            play_sounds(w, LP_CENTER, &CS1);
            effects::send_bolt_effect(w, s, true, 0);
            muo_server::mobile::base::move_to_world(w, s, LR_ENTER, MALAS);
            start_lamp_timer(w, id);
            if let Some(d) = data_mut(w, id) {
                d.enabled = false;
            }
        }
    } else {
        let mut correct = 0usize;
        for i in 0..16 {
            if (my >> i) & 1 == 1 && (their >> i) & 1 == 1 {
                correct += 1;
            }
        }
        let arg = (correct > 0).then(|| correct.to_string());
        puzzle_status(w, id, STATUE_MSG[correct.min(3)], arg.as_deref());
        for i in 0..5 {
            if let Some(p) = get_occupant(w, id, i) {
                start_rock_timer(w, p);
            }
        }
    }
    reset_levers(w, id);
}

fn on_after_delete(w: &mut World, id: ItemId) {
    let Some(d) = data(w, id) else { return };
    let (lists, regions, lamp, bx) = (
        [d.teles.clone(), d.statues.clone(), d.levers.clone()],
        d.tiles.clone(),
        d.lamp_room,
        d.box_,
    );
    for list in lists {
        for i in list.into_iter().flatten() {
            if !deleted(w, i) {
                delete_item(w, i);
            }
        }
    }
    if let Some(r) = lamp {
        region::unregister(w, r);
    }
    for r in regions {
        region::unregister(w, r);
    }
    if let Some(b) = bx.filter(|b| !deleted(w, *b)) {
        delete_item(w, b);
    }
}

/// Every entry is written (a deleted one as serial -1), so the count and the indexes survive.
fn write_list(w: &World, out: &mut dyn GenericWriter, list: &[Option<ItemId>]) {
    out.write_encoded_int(list.len() as i32);
    for &i in list {
        out.write_item(w, i);
    }
}

fn read_list(w: &World, r: &mut dyn GenericReader) -> Result<Vec<Option<ItemId>>> {
    let n = r.read_encoded_int()?;
    let mut v = Vec::new();
    for _ in 0..n.max(0) {
        v.push(r.read_item(w)?);
    }
    Ok(v)
}

/// `[SerializationGenerator(0, false)] LeverPuzzleController`.
pub static LEVER_PUZZLE_CONTROLLER: ItemType = ItemType {
    full_name: "Server.Engines.Doom.LeverPuzzleController",
    parent: Some(&ITEM_BASE),
    construct: None,
    make: Some(controller_new),
    on_delete: |w, id| {
        kill_timers(w, id);
        (ITEM_BASE.on_delete)(w, id);
    },
    on_after_delete,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        let Some(d) = data(w, id) else { return };
        out.write_i32(0);
        write_list(w, out, &d.levers);
        write_list(w, out, &d.statues);
        write_list(w, out, &d.teles);
        out.write_item(w, d.box_);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let d = ControllerData {
            levers: read_list(w, r)?,
            statues: read_list(w, r)?,
            teles: read_list(w, r)?,
            box_: r.read_item(w)?,
            ..Default::default()
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        // `[AfterDeserialization]`: the tiles, the lamp room, and fresh keys.
        let mut tiles = Vec::new();
        for loc in &TILE_COORDS {
            tiles.push(new_tile_region(w, loc));
        }
        let lamp = new_lamp_room_region(w, id);
        if let Some(d) = data_mut(w, id) {
            d.tiles = tiles;
            d.lamp_room = Some(lamp);
            d.enabled = true;
            d.their_key = 0;
            d.my_key = 0;
        }
        gen_key(w, id);
        Ok(())
    },
    ..ITEM_BASE
};

// ---- regions ----

/// `LeverPuzzleRegion.m_Occupant`.
#[derive(Default)]
pub struct Occupant(pub Option<MobileId>);

/// `LampRoomRegion.m_Controller`.
pub struct LampController(pub ItemId);

fn new_tile_region(w: &mut World, loc: &[i32; 2]) -> RegionId {
    let parent = region::find(w, LR_ENTER, Some(MALAS));
    let area = Rectangle2D::new(loc[0], loc[1], 1, 1);
    let r = region::new_region(
        w,
        &LEVER_PUZZLE_REGION,
        None,
        MALAS,
        Some(parent),
        None,
        region::convert_all_to_3d(&[area]),
    );
    muo_content::regions::base_region::init_data(w, r);
    if let Some(reg) = w.region_mut(r) {
        reg.ext.insert(Occupant::default());
    }
    region::register(w, r);
    r
}

fn new_lamp_room_region(w: &mut World, controller: ItemId) -> RegionId {
    let parent = region::find(w, LR_ENTER, Some(MALAS));
    let r = region::new_region(
        w,
        &LAMP_ROOM_REGION,
        None,
        MALAS,
        Some(parent),
        None,
        region::convert_all_to_3d(&[LR_RECT]),
    );
    muo_content::regions::base_region::init_data(w, r);
    if let Some(reg) = w.region_mut(r) {
        reg.ext.insert(LampController(controller));
    }
    region::register(w, r);
    r
}

/// `LeverPuzzleRegion`.
pub static LEVER_PUZZLE_REGION: RegionType = RegionType {
    name: "LeverPuzzleRegion",
    parent: Some(&muo_content::regions::base_region::BASE_REGION),
    on_enter: |w, r, m| {
        let player_alive = pmob::data(w, m).is_some() && muo_content::mx::alive(w, m);
        if let Some(o) = w.region_mut(r).and_then(|x| x.ext.get_mut::<Occupant>())
            && o.0.is_none()
            && player_alive
        {
            o.0 = Some(m);
        }
    },
    on_exit: |w, r, m| {
        if let Some(o) = w.region_mut(r).and_then(|x| x.ext.get_mut::<Occupant>())
            && o.0 == Some(m)
        {
            o.0 = None;
        }
    },
    ..muo_content::regions::base_region::BASE_REGION
};

fn controller_of(w: &World, r: RegionId) -> Option<ItemId> {
    w.region(r)?.ext.get::<LampController>().map(|c| c.0)
}

fn is_wanderer(w: &World, m: MobileId) -> bool {
    factory::mobile_is(w, m, "Server.Mobiles.WandererOfTheVoid")
}

/// `LampRoomRegion`.
pub static LAMP_ROOM_REGION: RegionType = RegionType {
    name: "LampRoomRegion",
    parent: Some(&muo_content::regions::base_region::BASE_REGION),
    on_enter: |w, r, m| {
        if is_wanderer(w, m) || muo_content::mx::access_level(w, m) > AccessLevel::Player {
            return;
        }
        let Some(c) = controller_of(w, r) else { return };
        if let Some(s) = data(w, c).and_then(|d| d.successful) {
            if pmob::data(w, m).is_some() {
                if m == s {
                    return;
                }
            } else if cd(w, m).is_some_and(|d| d.master == Some(s) || d.summoned) {
                return;
            }
        }
        start_kick_timer(w, m);
    },
    on_exit: |w, r, m| {
        if let Some(c) = controller_of(w, r)
            && data(w, c).and_then(|d| d.successful) == Some(m)
        {
            remove_successful(w, c);
        }
    },
    on_death: |w, _r, m| {
        if mobile_deleted(w, m) || is_wanderer(w, m) {
            return;
        }
        start_kick_timer(w, m);
    },
    // just in case
    on_skill_use: |w, r, m, _skill| {
        let Some(c) = controller_of(w, r) else {
            return true;
        };
        data(w, c).is_some_and(|d| d.successful.is_some())
            && (muo_content::mx::access_level(w, m) != AccessLevel::Player
                || data(w, c).and_then(|d| d.successful) == Some(m))
    },
    ..muo_content::regions::base_region::BASE_REGION
};

/// `LampRoomRegion.OnLogin(pm)`: a player who logged out inside the lamp room is kicked out.
pub fn on_login(w: &mut World, pm: MobileId) {
    let Some(m) = w.mobile(pm) else { return };
    let (l, map) = (m.location, m.map);
    let rect = LR_RECT;
    if l.x >= rect.x()
        && l.x <= rect.x() + 10
        && l.y >= rect.y()
        && l.y <= rect.y() + 10
        && map == Some(MapId::INTERNAL)
    {
        start_kick_timer(w, pm);
    }
}

// ---- timers ----

/// `LampRoomKickTimer`.
fn start_kick_timer(w: &mut World, m: MobileId) {
    timer::start(w, TimeSpan::from_millis(250), TimeSpan::ZERO, 1, move |w| {
        move_mobile_out(w, m)
    });
}

/// `MoveMobileOut(m)`.
pub fn move_mobile_out(w: &mut World, m: MobileId) {
    if w.mobile(m).is_none() {
        return;
    }
    if pmob::data(w, m).is_some()
        && !muo_content::mx::alive(w, m)
        && let Some(c) = w
            .mobile(m)
            .and_then(|x| x.corpse)
            .filter(|c| !deleted(w, *c))
    {
        ops::move_to_world_map(w, c, LR_EXIT, Some(MALAS));
    }
    muo_content::creature::pets::teleport_pets(w, m, LR_EXIT, MALAS, false);
    muo_server::mobile::part1::lifecycle::set_location(w, m, LR_EXIT);
    muo_server::mobile::base::process_delta(w, m);
}

/// `IsValidDamagable(m)`.
fn is_valid_damagable(w: &World, m: MobileId) -> bool {
    if mobile_deleted(w, m) {
        return false;
    }
    (muo_content::mx::player(w, m) && muo_content::mx::alive(w, m))
        || cd(w, m).is_some_and(|d| {
            (d.controlled || d.summoned) && !muo_server::mobile::base::is_dead_bonded_pet(w, m)
        })
}

/// `AniSafe(m)`.
fn ani_safe(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| x.body_mod.body_id() == 0)
        && muo_content::mx::alive(w, m)
        && !muo_content::spells::helper::under_transformation(w, m)
}

/// `DoDamage(m, min, max, poison)`.
fn do_damage(w: &mut World, m: MobileId, min: i32, max: i32, poison: bool) {
    if !mobile_deleted(w, m) && muo_content::mx::alive(w, m) {
        let damage = rnd::from(w, min, max);
        let f = muo_content::hooks::hooks(w).and_then(|h| h.aos_damage);
        let (phys, pois) = if poison { (0, 100) } else { (100, 0) };
        if let Some(f) = f {
            f(
                w,
                m,
                None,
                AosDamage {
                    amount: damage,
                    phys,
                    pois,
                    ..Default::default()
                },
            );
        }
    }
}

/// `RandomPointIn(x, y, x2, y2, z)`.
fn random_point_in(w: &mut World, x: i32, y: i32, x2: i32, y2: i32, z: i32) -> Point3D {
    Point3D::new(rnd::from(w, x, x2), rnd::from(w, y, y2), z)
}

fn play_sounds(w: &mut World, loc: Point3D, sounds: &[i32]) {
    for &s in sounds {
        effects::play_sound(w, loc, Some(MALAS), s);
    }
}

fn play_effect(
    w: &mut World,
    from: FxTarget,
    to: FxTarget,
    item_id: i32,
    speed: i32,
    explodes: bool,
) {
    effects::send_moving_particles(w, from, to, item_id, speed, 0, true, explodes, 2, 0, 0, 0);
}

/// `SendLocationEffect(p, itemID, speed, duration, hue)`: the arguments are passed on as `Effects.SendLocationEffect`'s
/// `(duration, speed, hue)`.
fn send_location_effect(
    w: &mut World,
    p: Point3D,
    item_id: i32,
    speed: i32,
    duration: i32,
    hue: i32,
) {
    effects::send_location_effect(w, p, Some(MALAS), item_id, speed, duration, hue, 0);
}

/// `PlayerSendASCII(player, index)`.
fn player_send_ascii(w: &mut World, player: MobileId, index: usize) {
    let Some(ns) = w.mobile(player).and_then(|m| m.net_state) else {
        return;
    };
    let (text, h, font) = MSGS[index];
    muo_server::packets::message::send_message(
        w,
        ns,
        muo_primitives::Serial::MINUS_ONE,
        0xFFFF,
        MessageType::LABEL,
        h,
        font,
        true,
        None,
        "",
        text,
    );
}

/// `POHMessage(from, index)`: speech in a font of its own, to every client in range.
fn poh_message(w: &mut World, from: MobileId, index: usize) {
    let Some(m) = w.mobile(from) else { return };
    let (serial, body, name, loc, map) = (
        m.serial,
        m.cur_body().body_id(),
        (m.ty.name)(w, from).unwrap_or_default(),
        m.location,
        m.map,
    );
    let Some(map) = map else { return };
    let (text, h, font) = MSGS[index];
    for ns in effects::clients_in_range(w, map, loc, None) {
        muo_server::packets::message::send_message(
            w,
            ns,
            serial,
            body,
            MessageType::REGULAR,
            h,
            font,
            true,
            None,
            &name,
            text,
        );
    }
}

/// `RockTimer`.
fn start_rock_timer(w: &mut World, player: MobileId) {
    let count = Rc::new(Cell::new(0));
    let stopped = Rc::new(Cell::new(false));
    timer::start(w, TimeSpan::ZERO, TimeSpan::from_millis(250), 3, {
        let (count, stopped) = (count, stopped);
        move |w| {
            if stopped.get() {
                return;
            }
            rock_tick(w, player, &count, &stopped)
        }
    });
}

fn rock_tick(w: &mut World, player: MobileId, count: &Cell<i32>, stopped: &Cell<bool>) {
    if w.mobile(player).is_none_or(|m| m.map != Some(MALAS)) {
        stopped.set(true);
        return;
    }
    count.set(count.get() + 1);
    let female = w.mobile(player).is_some_and(|m| m.female);
    let ploc = muo_content::mx::location(w, player);
    match count.get() {
        1 => {
            muo_content::mx::paralyze(w, player, TimeSpan::from_secs(2));
            effects::send_target_effect(w, player, 0x11B7, 20, 10, 0, 0);
            player_send_ascii(w, player, 0); // You are pinned down ...
            play_sounds(w, ploc, if !female { &FS } else { &MS });
            let above = FxTarget::point(Point3D::new(ploc.x, ploc.y, ploc.z + 50), Some(MALAS));
            play_effect(w, above, FxTarget::from(player), 0x11B7, 20, false);
        }
        2 => {
            do_damage(w, player, 80, 90, false);
            effects::send_target_effect(w, player, 0x36BD, 20, 10, 0, 0);
            play_sounds(w, ploc, &EXP);
            player_send_ascii(w, player, 1); // A speeding rock ...
            if ani_safe(w, player) {
                muo_content::mx::animate(w, player, 21, 10, 1, true, true, 0);
            }
        }
        3 => {
            stopped.set(true);
            effects::send_target_effect(w, player, 0x36B0, 20, 10, 0, 0);
            player_send_ascii(w, player, 1);
            play_sounds(w, ploc, if !female { &FS2 } else { &MS2 });

            let j = rnd::from(w, 6, 10);
            for _ in 0..j {
                let r = 10;
                let at = random_point_in(w, ploc.x - r, ploc.y - r, r * 2, r * 2, ploc.z);
                let victims: Vec<MobileId> =
                    muo_server::map::enumerate::mobiles_in_range(w, Some(MALAS), at.x, at.y, 2)
                        .collect();
                // Queued because we cannot modify while iterating through mobiles in range.
                for m in victims {
                    if is_valid_damagable(w, m) && m != player {
                        let g = rock(w);
                        play_effect(w, FxTarget::from(player), FxTarget::from(m), g, 8, true);
                        do_damage(w, m, 25, 30, false);
                        if muo_content::mx::player(w, m) {
                            poh_message(w, m, 2); // OUCH!
                        }
                    }
                }
                let g = rock(w);
                play_effect(
                    w,
                    FxTarget::from(player),
                    FxTarget::point(at, Some(MALAS)),
                    g,
                    8,
                    false,
                );
            }
        }
        _ => {}
    }
}

fn rock(w: &mut World) -> i32 {
    0x1363 + rnd::from(w, 0, 11)
}

/// `LampRoomTimer`.
fn start_lamp_timer(w: &mut World, controller: ItemId) {
    let mut level = 0usize;
    let mut ticks = 0;
    let h = timer::start(
        w,
        TimeSpan::from_secs(5),
        TimeSpan::from_secs(5),
        0,
        move |w| {
            ticks += 1;
            let Some(lamp) = data(w, controller).and_then(|d| d.lamp_room) else {
                return;
            };
            let mobiles = region::get_mobiles(w, lamp);
            if ticks >= 71 || region::get_player_count(w, lamp) == 0 {
                for &m in &mobiles {
                    if !mobile_deleted(w, m) && !muo_server::mobile::base::is_dead_bonded_pet(w, m)
                    {
                        muo_server::mobile::base::kill(w, m);
                    }
                }
                if let Some(d) = data_mut(w, controller) {
                    d.enabled = true;
                }
                let mut t = data(w, controller)
                    .map(|d| d.lamp_timer)
                    .unwrap_or_default();
                t.cancel(w);
                return;
            }
            if ticks % 12 == 0 {
                level += 1;
            }
            let level = level.min(5);
            for &m in &mobiles {
                if !is_valid_damagable(w, m) {
                    continue;
                }
                if ticks % 2 == 0 && level == 5 {
                    if muo_content::mx::player(w, m) {
                        muo_server::mobile::part2::messages::say_number(w, m, 1062092, "");
                        if ani_safe(w, m) {
                            muo_content::mx::animate(w, m, 32, 5, 1, true, false, 0);
                        }
                    }
                    do_damage(w, m, 15, 20, true);
                }
                if rnd::from(w, (level & 3) as i32, 3) == 3 {
                    muo_content::creature::overrides::apply_poison(w, m, Some(m), PA2[level]);
                }
                if ticks % 12 == 0 && level > 0 && muo_content::mx::player(w, m) {
                    muo_server::mobile::part2::messages::send_localized_message(
                        w,
                        m,
                        PA[level].0,
                        "",
                        PA[level].1,
                    );
                }
            }
            for _ in 0..=level {
                let p = random_point_in(
                    w,
                    LR_RECT.x(),
                    LR_RECT.y(),
                    LR_RECT.height(),
                    LR_RECT.width(),
                    -1,
                );
                let speed = rnd::from(w, 150, 200);
                send_location_effect(w, p, 0x36B0, speed, 0, PA[level].2);
            }
        },
    );
    if let Some(d) = data_mut(w, controller) {
        d.lamp_timer = h;
    }
}

// ---- the items ----

/// `LampRoomBox(controller)`.
#[derive(Default)]
struct BoxData {
    controller: Option<ItemId>,
    wanderer: Option<MobileId>,
}

fn box_data(w: &World, id: ItemId) -> Option<&BoxData> {
    w.item(id)?.ext.get::<BoxData>()
}

fn box_new(w: &mut World, controller: ItemId) -> ItemId {
    let id = ops::new_item_with_id(w, &LAMP_ROOM_BOX, 0xe80);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BoxData {
            controller: Some(controller),
            wanderer: None,
        });
    }
    ops::set_movable(w, id, false);
    id
}

/// Delete the controller when a part goes (`OnAfterDelete` of the box, statue and lever).
fn delete_controller(w: &mut World, c: Option<ItemId>) {
    if let Some(c) = c.filter(|c| !deleted(w, *c)) {
        delete_item(w, c);
    }
}

/// `[SerializationGenerator(0, false)] LampRoomBox`.
pub static LAMP_ROOM_BOX: ItemType = ItemType {
    full_name: "Server.Engines.Doom.LampRoomBox",
    parent: Some(&ITEM_BASE),
    construct: None,
    on_double_click: |w, id, m| {
        let loc = ops::get_world_location(w, id);
        if !muo_content::mx::in_range_point(w, m, loc, 3) {
            return;
        }
        let Some(c) = box_data(w, id).and_then(|d| d.controller) else {
            return;
        };
        if enabled(w, c) {
            return;
        }
        let alive = box_data(w, id)
            .and_then(|d| d.wanderer)
            .is_some_and(|x| muo_content::mx::alive(w, x));
        if !alive && let Some(wm) = factory::make_mobile(w, "Server.Mobiles.WandererOfTheVoid") {
            muo_server::mobile::base::move_to_world(w, wm, LR_ENTER, MALAS);
            muo_server::mobile::part2::messages::public_overhead_message_number(
                w,
                wm,
                muo_primitives::MessageType::REGULAR,
                0x3B2,
                1060002, // I am the guardian of...
                "",
                false,
            );
            if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<BoxData>()) {
                d.wanderer = Some(wm);
            }
            timer::start(w, TimeSpan::from_secs(5), TimeSpan::ZERO, 1, move |w| {
                // You try to pry the box open...
                muo_server::item::messages::public_overhead_message_number(
                    w,
                    id,
                    MessageType::REGULAR,
                    0x3B2,
                    1060003,
                    "",
                );
            });
        }
    },
    on_after_delete: |w, id| {
        let c = box_data(w, id).and_then(|d| d.controller);
        delete_controller(w, c);
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_item(w, box_data(w, id).and_then(|d| d.controller));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let controller = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(BoxData {
                controller,
                wanderer: None,
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// `LeverPuzzleStatue`'s controller (not serialized).
#[derive(Default)]
struct StatueData {
    controller: Option<ItemId>,
}

fn statue_new(w: &mut World, graphic: i32, controller: ItemId) -> ItemId {
    let id = ops::new_item_with_id(w, &LEVER_PUZZLE_STATUE, graphic);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(StatueData {
            controller: Some(controller),
        });
    }
    hue(w, id, 0x44E);
    ops::set_movable(w, id, false);
    id
}

/// `[SerializationGenerator(0, false)] LeverPuzzleStatue`.
pub static LEVER_PUZZLE_STATUE: ItemType = ItemType {
    full_name: "Server.Engines.Doom.LeverPuzzleStatue",
    parent: Some(&ITEM_BASE),
    construct: None,
    on_after_delete: |w, id| {
        let c = w
            .item(id)
            .and_then(|i| i.ext.get::<StatueData>())
            .and_then(|d| d.controller);
        delete_controller(w, c);
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `LeverPuzzleLever`'s fields.
#[derive(Default)]
struct LeverData {
    code: u16,
    controller: Option<ItemId>,
}

fn lever_data(w: &World, id: ItemId) -> Option<&LeverData> {
    w.item(id)?.ext.get::<LeverData>()
}

fn lever_new(w: &mut World, code: u16, controller: ItemId) -> ItemId {
    let id = ops::new_item_with_id(w, &LEVER_PUZZLE_LEVER, 0x108E);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(LeverData {
            code,
            controller: Some(controller),
        });
    }
    hue(w, id, 0x66D);
    ops::set_movable(w, id, false);
    id
}

/// `[SerializationGenerator(0, false)] LeverPuzzleLever`.
pub static LEVER_PUZZLE_LEVER: ItemType = ItemType {
    full_name: "Server.Engines.Doom.LeverPuzzleLever",
    parent: Some(&ITEM_BASE),
    construct: None,
    on_double_click: |w, id, m| {
        let Some((code, c)) = lever_data(w, id).map(|d| (d.code, d.controller)) else {
            return;
        };
        if c.is_some_and(|c| enabled(w, c)) {
            let cur = w.item(id).map_or(0, |i| i.item_id);
            if let Some(t) = w.item(id).map(|i| i.ty) {
                (t.set_item_id)(w, id, cur ^ 2);
            }
            let (p, map) = w
                .item(id)
                .map_or((Point3D::ZERO, None), |i| (i.location, i.map));
            effects::play_sound(w, p, map, 0x3E8);
            if let Some(c) = c {
                lever_pulled(w, c, code);
            }
        } else {
            // You throw the switch, but the mechanism cannot be engaged again so soon.
            muo_content::mx::msg(w, m, 1060001);
        }
    },
    on_after_delete: |w, id| {
        let c = lever_data(w, id).and_then(|d| d.controller);
        delete_controller(w, c);
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let d = lever_data(w, id);
        out.write_u16(d.map_or(0, |d| d.code));
        out.write_item(w, d.and_then(|d| d.controller));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let code = r.read_u16()?;
        let controller = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(LeverData { code, controller });
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// `LampRoomTeleporter(dat)`: `Hue = dat[1]; ItemID = dat[0]; Movable = false`.
fn teleporter_new(w: &mut World, dat: &[i32; 2]) -> ItemId {
    let id = ops::new_item_with_id(w, &LAMP_ROOM_TELEPORTER, 1);
    hue(w, id, dat[1]);
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.set_item_id)(w, id, dat[0]);
    }
    ops::set_movable(w, id, false);
    id
}

/// `[TypeAlias("Server.Engines.Doom.LampRoomTelePorter")] LampRoomTeleporter`.
pub static LAMP_ROOM_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Engines.Doom.LampRoomTeleporter",
    parent: Some(&ITEM_BASE),
    construct: None,
    handles_on_movement: |_, _| true,
    on_move_over: |w, _id, m| {
        if pmob::data(w, m).is_some() {
            if muo_content::spells::helper::check_combat(w, m) {
                muo_server::mobile::part2::messages::send_localized_message(
                    w, m, 1005564, "", 0x22,
                ); // Wouldst thou flee during the heat of battle??
            } else {
                muo_content::creature::pets::teleport_pets(w, m, LR_EXIT, MALAS, false);
                muo_server::mobile::base::move_to_world(w, m, LR_EXIT, MALAS);
                return false;
            }
        }
        true
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- the command ----

/// `[GenLeverPuzzle`: "Generates lamp room and lever puzzle in doom."
pub fn gen_lamp_puzzle(w: &mut World, e: &mut muo_server::commands::CommandEventArgs) {
    let from = e.mobile;
    let exists = muo_server::map::enumerate::items_at(w, Some(MALAS), LP_CENTER.x, LP_CENTER.y)
        .any(|i| w.item(i).is_some_and(|x| x.ty.is(&LEVER_PUZZLE_CONTROLLER)));
    if exists {
        muo_content::mx::send_message(
            w,
            from,
            "Lamp room puzzle already exists: please delete the existing controller first ...",
        );
        return;
    }
    muo_content::mx::send_message(w, from, "Generating Lamp Room puzzle...");
    let c = controller_new(w);
    ops::move_to_world_map(w, c, LP_CENTER, Some(MALAS));
    let installed = w.resources.get_mut::<Installed>().0;
    muo_content::mx::send_message(
        w,
        from,
        if !installed {
            "There was a problem generating the puzzle."
        } else {
            "Lamp room puzzle successfully generated."
        },
    );
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&LEVER_PUZZLE_CONTROLLER, &[]);
    types.register_item(&LAMP_ROOM_BOX, &[]);
    types.register_item(&LEVER_PUZZLE_STATUE, &[]);
    types.register_item(&LEVER_PUZZLE_LEVER, &[]);
    types.register_item(
        &LAMP_ROOM_TELEPORTER,
        &["Server.Engines.Doom.LampRoomTelePorter"],
    );
}

#[allow(dead_code)]
fn _e(_: EntityId) {}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};

    /// A deleted part stays in its list (written as serial -1), so the count and the lever indexes survive a save.
    #[test]
    fn list_keeps_its_null_entries() {
        let w = World::new();
        let mut out = SaveWriter::new(true);
        write_list(&w, &mut out, &[None, None, None]);
        let mut r = SaveReader::new(out.as_slice());
        assert_eq!(read_list(&w, &mut r).unwrap(), vec![None, None, None]);
        assert_eq!(r.remaining(), 0);
    }
}
