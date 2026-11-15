//! Camps: `BaseCamp` and the seven camps (Brigand, Lizardmen, Rat, Orc, Healer, Banker, Mage), and
//! the `LockableBarrel` the camps use.
//!
//! The mobiles, chests, potions and signs a camp makes are made by name (`muo_content::factory`); one that no crate
//! registers is simply left out of the camp.

use muo_content::creature;
use muo_content::factory::{make_item, make_mobile};
use muo_content::items::cont::{self, LOCKABLE_CONTAINER, TrapType};
use muo_content::items::gold::new_gold;
use muo_content::items::house_deps::{DoorFacing, HouseDoorKind, deps};
use muo_primitives::{DateTime, Direction, Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

/// What `BaseCamp` (and the prisoner of the spawn camps) adds.
#[derive(Clone, Debug, Default)]
pub struct CampData {
    pub items: Vec<ItemId>,
    pub mobiles: Vec<MobileId>,
    pub decay_time: DateTime,
    pub decay_delay: TimeSpan,
    pub decay_timer: Option<TimerHandle>,
    pub init_timer: Option<TimerHandle>,
    pub prisoner: Option<MobileId>,
}

fn data(w: &World, id: ItemId) -> CampData {
    w.item(id)
        .and_then(|i| i.ext.get::<CampData>())
        .cloned()
        .unwrap_or_default()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut CampData> {
    w.item_mut(id)?.ext.get_mut::<CampData>()
}

/// A camp class' own virtuals.
pub struct CampClass {
    pub add_components: fn(&mut World, ItemId),
    pub on_enter: fn(&mut World, ItemId, MobileId),
    pub on_exit: fn(&mut World, ItemId, MobileId),
    pub add_item: fn(&mut World, ItemId, ItemId, i32, i32, i32),
    /// `[SerializationGenerator]` has a `_prisoner` field.
    pub has_prisoner: bool,
}

fn class(w: &World, id: ItemId) -> &'static CampClass {
    w.item(id)
        .and_then(|i| i.ty.family::<CampClass>())
        .unwrap_or(&BASE_CAMP_CLASS)
}

fn deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// `new BaseCamp(multiID)` of class `ty`.
pub fn new_base_camp(w: &mut World, ty: &'static ItemType, multi_id: i32) -> ItemId {
    let id = muo_server::item::multi::new_base_multi(w, ty, multi_id);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(CampData {
            decay_delay: TimeSpan::from_mins(30),
            ..Default::default()
        });
    }
    refresh_decay(w, id, true);
    let t = timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        check_add_components(w, id)
    });
    if let Some(d) = data_mut(w, id) {
        d.init_timer = Some(t);
    }
    id
}

/// `CheckAddComponents()`.
fn check_add_components(w: &mut World, id: ItemId) {
    if let Some(d) = data_mut(w, id) {
        d.init_timer = None;
    }
    if deleted(w, id) {
        return;
    }
    (class(w, id).add_components)(w, id);
}

/// `DecayDelay` setter.
pub fn set_decay_delay(w: &mut World, id: ItemId, delay: TimeSpan) {
    if let Some(d) = data_mut(w, id) {
        d.decay_delay = delay;
    }
    refresh_decay(w, id, true);
}

/// `RefreshDecay(setDecayTime)`.
pub fn refresh_decay(w: &mut World, id: ItemId, set_decay_time: bool) {
    if deleted(w, id) {
        return;
    }
    let delay = data(w, id).decay_delay;
    if set_decay_time {
        let t = w.now + delay;
        if let Some(d) = data_mut(w, id) {
            d.decay_time = t;
        }
    }
    if let Some(mut t) = data_mut(w, id).and_then(|d| d.decay_timer.take()) {
        t.cancel(w);
    }
    let t = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| delete_item(w, id));
    if let Some(d) = data_mut(w, id) {
        d.decay_timer = Some(t);
    }
}

fn base_add_item(w: &mut World, id: ItemId, item: ItemId, x: i32, y: i32, z: i32) {
    if let Some(d) = data_mut(w, id) {
        d.items.push(item);
    }
    let (l, map) = w.item(id).map(|i| (i.location, i.map)).unwrap_or_default();
    let zavg = map.map_or(0, |m| w.maps.get_average_z(m, l.x + x, l.y + y));
    ops::move_to_world_map(w, item, Point3D::new(l.x + x, l.y + y, zavg + z), map);
}

/// `AddItem(item, xOffset, yOffset, zOffset)` (virtual).
pub fn add_item(w: &mut World, id: ItemId, item: ItemId, x: i32, y: i32, z: i32) {
    (class(w, id).add_item)(w, id, item, x, y, z);
}

/// `AddMobile(m, wanderRange, xOffset, yOffset, zOffset)`.
pub fn add_mobile(
    w: &mut World,
    id: ItemId,
    m: MobileId,
    wander_range: i32,
    x: i32,
    y: i32,
    z: i32,
) {
    if let Some(d) = data_mut(w, id) {
        d.mobiles.push(m);
    }
    let (l, map) = w.item(id).map(|i| (i.location, i.map)).unwrap_or_default();
    let zavg = map.map_or(0, |mp| w.maps.get_average_z(mp, l.x + x, l.y + y));
    let loc = Point3D::new(l.x + x, l.y + y, zavg + z);
    creature::edit(w, m, |d| {
        d.range_home = wander_range;
        d.home = loc;
    });
    if muo_content::factory::mobile_is(w, m, "Server.Mobiles.BaseVendor") {
        muo_server::mobile::part1::props_b::set_direction(w, m, Direction::SOUTH);
    }
    if let Some(map) = map {
        muo_server::mobile::base::move_to_world(w, m, loc, map);
    }
}

fn on_movement(w: &mut World, id: ItemId, m: MobileId, old: Point3D) {
    let loc = w.item(id).map_or(Point3D::ZERO, |i| i.location);
    let new = w.mobile(m).map_or(Point3D::ZERO, |x| x.location);
    let range = 10; // EventRange
    let in_old = (old.x - loc.x).abs().max((old.y - loc.y).abs()) <= range;
    let in_new = (new.x - loc.x).abs().max((new.y - loc.y).abs()) <= range;
    if in_new && !in_old {
        (class(w, id).on_enter)(w, id, m);
    } else if in_old && !in_new {
        (class(w, id).on_exit)(w, id, m);
    }
}

fn on_after_delete(w: &mut World, id: ItemId) {
    (muo_server::item::multi::BASE_MULTI.on_after_delete)(w, id);
    let d = data(w, id);
    for i in d.items {
        if w.item(i).is_some() {
            delete_item(w, i);
        }
    }
    for m in d.mobiles {
        let Some(mob) = w.mobile(m) else { continue };
        let prisoner_free = creature::cd(w, m).is_some_and(|c| !c.is_prisoner);
        if mob.cant_walk || prisoner_free {
            muo_server::mobile::base::delete(w, m);
        }
    }
    if let Some(d) = data_mut(w, id) {
        d.items.clear();
        d.mobiles.clear();
    }
    for t in [data(w, id).decay_timer, data(w, id).init_timer]
        .into_iter()
        .flatten()
    {
        let mut t = t;
        t.cancel(w);
    }
    if let Some(d) = data_mut(w, id) {
        d.decay_timer = None;
        d.init_timer = None;
    }
}

fn tidy_items(w: &World, l: &[ItemId]) -> Vec<ItemId> {
    l.iter()
        .copied()
        .filter(|&i| w.item(i).is_some_and(|x| !x.deleted()))
        .collect()
}

fn tidy_mobiles(w: &World, l: &[MobileId]) -> Vec<MobileId> {
    l.iter()
        .copied()
        .filter(|&m| w.mobile(m).is_some_and(|x| !x.deleted))
        .collect()
}

fn serialize(w: &World, id: ItemId, out: &mut dyn muo_io::GenericWriter) {
    (muo_server::item::multi::BASE_MULTI.serialize)(w, id, out);
    out.write_i32(2); // version
    let d = data(w, id);
    for list in [tidy_items(w, &d.items)] {
        out.write_encoded_int(list.len() as i32);
        for i in list {
            out.write_item(w, Some(i));
        }
    }
    let ms = tidy_mobiles(w, &d.mobiles);
    out.write_encoded_int(ms.len() as i32);
    for m in ms {
        out.write_mobile(w, Some(m));
    }
    out.write_anchored_time_ticks(d.decay_time.ticks());
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn muo_io::GenericReader) -> muo_io::Result<()> {
    (muo_server::item::multi::BASE_MULTI.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = CampData {
        decay_delay: TimeSpan::from_mins(30),
        ..Default::default()
    };
    if version >= 2 {
        let n = r.read_encoded_int()?;
        for _ in 0..n {
            d.items.extend(r.read_item(w)?);
        }
        let n = r.read_encoded_int()?;
        for _ in 0..n {
            d.mobiles.extend(r.read_mobile(w)?);
        }
        d.decay_time = DateTime::from_ticks(r.read_anchored_time_ticks()?);
    } else {
        // version 1 (encoded counts) and 0 (int counts) both end with a delta time
        let counted = |r: &mut dyn muo_io::GenericReader| {
            if version == 1 {
                r.read_encoded_int()
            } else {
                r.read_i32()
            }
        };
        let n = counted(r)?;
        for _ in 0..n {
            d.items.extend(r.read_item(w)?);
        }
        let n = counted(r)?;
        for _ in 0..n {
            d.mobiles.extend(r.read_mobile(w)?);
        }
        d.decay_time = DateTime::from_ticks(r.read_delta_time_ticks(w.now.ticks())?);
    }
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
    Ok(())
}

/// `AfterDeserialization()`; the subclass fields are read first, so it runs when the camp is complete.
fn after_deserialization(w: &mut World, id: ItemId) {
    let remaining = data(w, id).decay_time - w.now;
    if remaining > TimeSpan::ZERO {
        if let Some(d) = data_mut(w, id) {
            d.decay_delay = remaining;
        }
        refresh_decay(w, id, false);
    } else {
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            delete_item(w, id)
        });
        return;
    }
    let t = timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        check_add_components(w, id)
    });
    if let Some(d) = data_mut(w, id) {
        d.init_timer = Some(t);
    }
}

static BASE_CAMP_CLASS: CampClass = CampClass {
    add_components: |_, _| {},
    on_enter: |w, id, _| refresh_decay(w, id, true),
    on_exit: |w, id, _| refresh_decay(w, id, true),
    add_item: base_add_item,
    has_prisoner: false,
};

/// `Server.Multis.BaseCamp` (abstract).
pub static BASE_CAMP: ItemType = ItemType {
    full_name: "Server.Multis.BaseCamp",
    parent: Some(&muo_server::item::multi::BASE_MULTI),
    construct: None,
    family: Some(&BASE_CAMP_CLASS),
    handles_on_movement: |_, _| true,
    on_movement,
    on_after_delete,
    serialize,
    deserialize,
    ..muo_server::item::multi::BASE_MULTI
};

fn new_static(w: &mut World, graphic: i32) -> Option<ItemId> {
    let s = make_item(w, "Server.Items.Static")?;
    if let Some(ty) = w.item(s).map(|i| i.ty) {
        (ty.set_item_id)(w, s, graphic);
    }
    Some(s)
}

fn new_item(w: &mut World, graphic: i32) -> ItemId {
    ops::new_item_with_id(w, &ITEM_BASE, graphic)
}

fn make_gate(w: &mut World, facing: DoorFacing) -> Option<ItemId> {
    let f = deps(w)?.make_door?;
    f(
        w,
        HouseDoorKind::Named("Server.Items.LightWoodGate", facing),
    )
}

fn new_sign(w: &mut World, sign_type: i32, west: bool) -> Option<ItemId> {
    let s = make_item(w, "Server.Items.Sign")?;
    let graphic = 0xB95 + 2 * sign_type + i32::from(west);
    if let Some(ty) = w.item(s).map(|i| i.ty) {
        (ty.set_item_id)(w, s, graphic);
    }
    Some(s)
}

fn add_opt_item(
    w: &mut World,
    camp: ItemId,
    item: Option<ItemId>,
    x: i32,
    y: i32,
    z: i32,
) -> Option<ItemId> {
    let item = item?;
    add_item(w, camp, item, x, y, z);
    Some(item)
}

fn add_opt_mobile(
    w: &mut World,
    camp: ItemId,
    m: Option<MobileId>,
    range: i32,
    x: i32,
    y: i32,
    z: i32,
) -> Option<MobileId> {
    let m = m?;
    add_mobile(w, camp, m, range, x, y, z);
    Some(m)
}

/// The gate pair, sign and two NPCs of the town camps (`HealerCamp`, `BankerCamp`, `MageCamp`).
fn town_camp(w: &mut World, camp: ItemId, sign_type: i32, npc: &str) {
    let west = make_gate(w, DoorFacing::WestCW);
    let east = make_gate(w, DoorFacing::EastCCW);
    add_opt_item(w, camp, west, -4, 4, 7);
    add_opt_item(w, camp, east, -3, 4, 7);
    if let (Some(wd), Some(ed), Some(f)) = (west, east, deps(w).and_then(|x| x.door_set_link)) {
        f(w, wd, ed);
        f(w, ed, wd);
    }
    let sign = new_sign(w, sign_type, true);
    add_opt_item(w, camp, sign, -5, 5, -4);
    let a = make_mobile(w, npc);
    add_opt_mobile(w, camp, a, 4, -4, 3, 7);
    let b = make_mobile(w, npc);
    add_opt_mobile(w, camp, b, 5, 4, -2, 0);
}

/// What differs between the four spawn camps.
struct SpawnCamp {
    mobile: &'static str,
    count: i32,
    captain: Option<&'static str>,
    /// The Brigand camp's `Utility.Random(5)` potion (a fifth outcome is nothing); the others roll 80% first.
    brigand_potions: bool,
    fire_y: i32,
    standard: Option<(i32, i32, i32, i32)>,
    chest: (i32, i32),
    crates: (i32, i32),
}

const BRIGAND: SpawnCamp = SpawnCamp {
    mobile: "Server.Mobiles.Brigand",
    count: 4,
    captain: None,
    brigand_potions: true,
    fire_y: 7,
    standard: None,
    chest: (-2, -2),
    crates: (2, 2),
};
const LIZARDMEN: SpawnCamp = SpawnCamp {
    mobile: "Server.Mobiles.Lizardman",
    count: 4,
    captain: None,
    brigand_potions: false,
    fire_y: 7,
    standard: Some((0x41F, 4, 4, 0)),
    chest: (2, -2),
    crates: (-2, 2),
};
const RAT: SpawnCamp = SpawnCamp {
    mobile: "Server.Mobiles.Ratman",
    count: 4,
    captain: None,
    brigand_potions: false,
    fire_y: 6,
    standard: Some((0x41F, 5, 5, 0)),
    chest: (-2, -2),
    crates: (2, 2),
};
const ORC: SpawnCamp = SpawnCamp {
    mobile: "Server.Mobiles.Orc",
    count: 3,
    captain: Some("Server.Mobiles.OrcCaptain"),
    brigand_potions: false,
    fire_y: 7,
    standard: Some((0x428, -5, -4, 0)),
    chest: (-2, 2),
    crates: (2, -2),
};

fn spawn_camp_components(w: &mut World, camp: ItemId, c: &SpawnCamp) {
    ops::set_visible(w, camp, false);
    set_decay_delay(w, camp, TimeSpan::from_mins(5));

    let s = new_static(w, 0x10ee);
    add_opt_item(w, camp, s, 0, 0, 0);
    let s = new_static(w, 0xfac);
    add_opt_item(w, camp, s, 0, c.fire_y, 0);

    match utility::random(&mut w.rng, 3) {
        0 => {
            let i = new_item(w, 0xDE3); // Campfire
            add_item(w, camp, i, 0, c.fire_y, 0);
            let i = new_item(w, 0x974); // Cauldron
            add_item(w, camp, i, 0, c.fire_y, 1);
        }
        1 => {
            let i = new_item(w, 0x1E95); // Rabbit on a spit
            add_item(w, camp, i, 0, c.fire_y, 1);
        }
        _ => {
            let i = new_item(w, 0x1E94); // Chicken on a spit
            add_item(w, camp, i, 0, c.fire_y, 1);
        }
    }

    if let Some((graphic, x, y, z)) = c.standard {
        let i = new_item(w, graphic); // Gruesome Standart
        add_item(w, camp, i, x, y, z);
    }

    add_camp_chests(w, camp, c);

    for _ in 0..c.count {
        let m = make_mobile(w, c.mobile);
        let (x, y) = (
            utility::random_min_max(&mut w.rng, -7, 7),
            utility::random_min_max(&mut w.rng, -7, 7),
        );
        add_opt_mobile(w, camp, m, 6, x, y, 0);
    }
    if let Some(captain) = c.captain {
        let m = make_mobile(w, captain);
        let (x, y) = (
            utility::random_min_max(&mut w.rng, -7, 7),
            utility::random_min_max(&mut w.rng, -7, 7),
        );
        add_opt_mobile(w, camp, m, 2, x, y, 0);
    }

    let kind = if utility::random(&mut w.rng, 2) == 0 {
        "Server.Mobiles.Noble"
    } else {
        "Server.Mobiles.SeekerOfAdventure"
    };
    let prisoner = make_mobile(w, kind);
    if let Some(p) = prisoner {
        creature::edit(w, p, |d| d.is_prisoner = true);
        if let Some(m) = w.mobile_mut(p) {
            m.cant_walk = true;
        }
        if let Some(d) = data_mut(w, camp) {
            d.prisoner = Some(p);
        }
        let hue = [0x57, 0x67, 0x77, 0x87, 0x117][utility::random(&mut w.rng, 5) as usize];
        if let Some(m) = w.mobile_mut(p) {
            m.yell_hue = hue;
        }
        let (x, y) = (
            utility::random_min_max(&mut w.rng, -2, 2),
            utility::random_min_max(&mut w.rng, -2, 2),
        );
        add_mobile(w, camp, p, 2, x, y, 0);
    }
}

fn add_camp_chests(w: &mut World, camp: ItemId, c: &SpawnCamp) {
    let name = match utility::random(&mut w.rng, 3) {
        0 => "Server.Items.MetalChest",
        1 => "Server.Items.MetalGoldenChest",
        _ => "Server.Items.WoodenChest",
    };
    if let Some(chest) = make_item(w, name) {
        set_lift_override(w, chest);
        if let Some(f) = deps(w).and_then(|x| x.treasure_map_chest_fill) {
            f(w, chest, 1);
        }
        add_item(w, camp, chest, c.chest.0, c.chest.1, 0);
    }

    let name = match utility::random(&mut w.rng, 4) {
        0 => "Server.Items.SmallCrate",
        1 => "Server.Items.MediumCrate",
        2 => "Server.Items.LargeCrate",
        _ => "Server.Multis.LockableBarrel",
    };
    let crates = make_item(w, name);
    let power = utility::random_min_max(&mut w.rng, 30, 40);
    if let Some(cr) = crates
        && let Some(e) = cont::ext_mut(w, cr)
    {
        e.trap_type = TrapType::ExplosionTrap;
        e.trap_power = power;
        e.trap_level = 2;
        e.required_skill = 76;
        e.lock_level = 66;
        e.max_lock_level = 116;
        e.locked = true;
    }
    let amount = utility::random_min_max(&mut w.rng, 100, 400);
    let gold = new_gold(w, amount);
    drop_into(w, crates, Some(gold));
    for name in ["Server.Items.Arrow", "Server.Items.Bolt"] {
        let a = make_item(w, name);
        if let Some(a) = a {
            ops::set_amount(w, a, 10);
        }
        drop_into(w, crates, a);
    }
    if let Some(cr) = crates {
        set_lift_override(w, cr);
    }

    let potion = if c.brigand_potions {
        match utility::random(&mut w.rng, 5) {
            0 => Some("LesserCurePotion"),
            1 => Some("LesserExplosionPotion"),
            2 => Some("LesserHealPotion"),
            3 => Some("LesserPoisonPotion"),
            _ => None,
        }
    } else if utility::random_double(&mut w.rng) < 0.8 {
        Some(match utility::random(&mut w.rng, 4) {
            0 => "LesserCurePotion",
            1 => "LesserExplosionPotion",
            2 => "LesserHealPotion",
            _ => "LesserPoisonPotion",
        })
    } else {
        None
    };
    if let Some(p) = potion {
        let item = make_item(w, &format!("Server.Items.{p}"));
        drop_into(w, crates, item);
    }
    if let Some(cr) = crates {
        add_item(w, camp, cr, c.crates.0, c.crates.1, 0);
    }
}

fn drop_into(w: &mut World, cont: Option<ItemId>, item: Option<ItemId>) {
    if let (Some(c), Some(i)) = (cont, item)
        && let Some(ty) = w.item(c).map(|x| x.ty)
    {
        (ty.drop_item)(w, c, i);
    }
}

fn set_lift_override(w: &mut World, c: ItemId) {
    if let Some(i) = w.item_mut(c) {
        i.container_mut().lift_override = true;
    }
}

/// `OnEnter(m)` of the spawn camps: the prisoner begs. Doesn't refresh the decay timer.
fn spawn_on_enter(w: &mut World, id: ItemId, m: MobileId) {
    let player = w.mobile(m).is_some_and(|x| x.player);
    let prisoner = data(w, id)
        .prisoner
        .filter(|&p| w.mobile(p).is_some_and(|x| x.cant_walk));
    if player && let Some(p) = prisoner {
        let number = 502261 + utility::random(&mut w.rng, 8);
        muo_server::mobile::part2::messages::yell_number(w, p, number, "");
    }
}

/// `AddItem` of the spawn camps: nothing moves.
fn spawn_add_item(w: &mut World, id: ItemId, item: ItemId, x: i32, y: i32, z: i32) {
    ops::set_movable(w, item, false);
    base_add_item(w, id, item, x, y, z);
}

macro_rules! camp {
    ($name:ident, $cls:ident, $full:literal, $id:expr, $new:ident, $comp:expr, $enter:expr, $exit:expr, $add:expr, $prisoner:literal) => {
        pub static $cls: CampClass = CampClass {
            add_components: $comp,
            on_enter: $enter,
            on_exit: $exit,
            add_item: $add,
            has_prisoner: $prisoner,
        };

        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_CAMP),
            construct: Some($new),
            family: Some(&$cls),
            serialize: |w, id, out| {
                (BASE_CAMP.serialize)(w, id, out);
                out.write_i32(0);
                if $prisoner {
                    out.write_mobile(w, data(w, id).prisoner);
                }
            },
            deserialize: |w, id, r| {
                (BASE_CAMP.deserialize)(w, id, r)?;
                r.read_i32()?;
                if $prisoner {
                    let p = r.read_mobile(w)?;
                    if let Some(d) = data_mut(w, id) {
                        d.prisoner = p;
                    }
                }
                after_deserialization(w, id);
                Ok(())
            },
            ..BASE_CAMP
        };

        fn $new(w: &mut World) -> ItemId {
            new_base_camp(w, &$name, $id)
        }
    };
}

fn nothing(_: &mut World, _: ItemId, _: MobileId) {}

camp!(
    BRIGAND_CAMP,
    BRIGAND_CAMP_CLASS,
    "Server.Multis.BrigandCamp",
    0x10EE,
    new_brigand_camp,
    |w, id| spawn_camp_components(w, id, &BRIGAND),
    spawn_on_enter,
    nothing,
    spawn_add_item,
    true
);
camp!(
    LIZARDMEN_CAMP,
    LIZARDMEN_CAMP_CLASS,
    "Server.Multis.LizardmenCamp",
    0x10EE,
    new_lizardmen_camp,
    |w, id| spawn_camp_components(w, id, &LIZARDMEN),
    spawn_on_enter,
    nothing,
    spawn_add_item,
    true
);
camp!(
    RAT_CAMP,
    RAT_CAMP_CLASS,
    "Server.Multis.RatCamp",
    0x10EE,
    new_rat_camp,
    |w, id| spawn_camp_components(w, id, &RAT),
    spawn_on_enter,
    nothing,
    spawn_add_item,
    true
);
camp!(
    ORC_CAMP,
    ORC_CAMP_CLASS,
    "Server.Multis.OrcCamp",
    0x10EE,
    new_orc_camp,
    |w, id| spawn_camp_components(w, id, &ORC),
    spawn_on_enter,
    nothing,
    spawn_add_item,
    true
);
camp!(
    HEALER_CAMP,
    HEALER_CAMP_CLASS,
    "Server.Multis.HealerCamp",
    0x1F4,
    new_healer_camp,
    |w, id| town_camp(w, id, 11, "Server.Mobiles.Healer"),
    |w, id, _| refresh_decay(w, id, true),
    |w, id, _| refresh_decay(w, id, true),
    base_add_item,
    false
);
camp!(
    BANKER_CAMP,
    BANKER_CAMP_CLASS,
    "Server.Multis.BankerCamp",
    0x1F6,
    new_banker_camp,
    |w, id| town_camp(w, id, 59, "Server.Mobiles.Banker"),
    |w, id, _| refresh_decay(w, id, true),
    |w, id, _| refresh_decay(w, id, true),
    base_add_item,
    false
);
camp!(
    MAGE_CAMP,
    MAGE_CAMP_CLASS,
    "Server.Multis.MageCamp",
    0x1F5,
    new_mage_camp,
    |w, id| town_camp(w, id, 12, "Server.Mobiles.Mage"),
    |w, id, _| refresh_decay(w, id, true),
    |w, id, _| refresh_decay(w, id, true),
    base_add_item,
    false
);

/// `Server.Multis.LockableBarrel`.
pub static LOCKABLE_BARREL: ItemType = ItemType {
    full_name: "Server.Multis.LockableBarrel",
    parent: Some(&LOCKABLE_CONTAINER),
    construct: Some(|w| cont::new_lockable(w, &LOCKABLE_BARREL, 0xE77)),
    default_weight: |w, id| {
        crate::template_props::item_float(w, id, "defaultWeight", LOCKABLE_CONTAINER.default_weight)
    },
    serialize: |w, id, out| {
        (LOCKABLE_CONTAINER.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (LOCKABLE_CONTAINER.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..LOCKABLE_CONTAINER
};

pub fn register(types: &mut TypeRegistry) {
    for ty in [
        &BRIGAND_CAMP,
        &LIZARDMEN_CAMP,
        &RAT_CAMP,
        &ORC_CAMP,
        &HEALER_CAMP,
        &BANKER_CAMP,
        &MAGE_CAMP,
        &LOCKABLE_BARREL,
    ] {
        types.register_item(ty, &[]);
    }
}
