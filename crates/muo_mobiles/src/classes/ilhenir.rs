//! `Ilhenir` and `Meraktus` (the two champions of the Corrupt and Tormented spawns) and the
//! `StainedOoze` item that Ilhenir and the Interred Grizzle drop.

use muo_content::creature::cd;
use muo_content::factory;
use muo_content::misc::aos_helpers::{self, DamageSplit};
use muo_content::mx;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{MessageType, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base as ibase, ops};
use muo_server::map::MapId;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{enumerators, messages};
use muo_server::packets::movement::{SpeedControlSetting, send_speed_control};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use crate::common::*;
use crate::hand::si;
use crate::leaf::creature_leaf;
use crate::npcs::special::champions::{BASE_CHAMPION, BASE_CHAMPION_C};

// ---- StainedOoze ----

#[derive(Clone, Copy, Default)]
pub struct OozeData {
    pub corrosive: bool,
    pub ticks: i32,
    pub timer: TimerHandle,
}

fn ooze(w: &World, id: ItemId) -> OozeData {
    w.item(id)
        .and_then(|i| i.ext.get::<OozeData>())
        .copied()
        .unwrap_or_default()
}
fn set_ooze(w: &mut World, id: ItemId, d: OozeData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

fn start_ooze_timer(w: &mut World, id: ItemId) {
    let h = timer::start(w, TimeSpan::ZERO, TimeSpan::from_secs(1), 0, move |w| {
        ooze_tick(w, id)
    });
    let mut d = ooze(w, id);
    d.timer = h;
    set_ooze(w, id, d);
}

/// `new StainedOoze(corrosive)`.
pub fn new_stained_ooze(w: &mut World, corrosive: bool) -> ItemId {
    let id = ops::new_item_with_id(w, &STAINED_OOZE, 0x122A);
    ops::set_movable(w, id, false);
    ibase::set_hue(w, id, 0x95);
    set_ooze(
        w,
        id,
        OozeData {
            corrosive,
            ticks: 0,
            timer: TimerHandle::default(),
        },
    );
    start_ooze_timer(w, id);
    id
}

/// The `IDurability` of an armor, weapon or clothing item: its `HitPoints`.
fn hit_points(w: &World, id: ItemId) -> Option<i32> {
    use muo_content::items::{armor, clothing, weapon};
    let i = w.item(id)?;
    if let Some(d) = i.ext.get::<weapon::WeaponData>() {
        Some(d.hit_points)
    } else if let Some(d) = i.ext.get::<clothing::ClothingData>() {
        Some(d.hit_points)
    } else {
        i.ext.get::<armor::ArmorData>().map(|d| d.hit_points)
    }
}

fn set_hit_points(w: &mut World, id: ItemId, v: i32) {
    use muo_content::items::{armor, clothing, weapon};
    let Some(i) = w.item(id) else { return };
    if i.ext.get::<weapon::WeaponData>().is_some() {
        weapon::set_hit_points(w, id, v);
    } else if i.ext.get::<clothing::ClothingData>().is_some() {
        clothing::set_hit_points(w, id, v);
    } else {
        armor::set_hit_points(w, id, v);
    }
}

fn ooze_damage(w: &mut World, id: ItemId, m: MobileId) {
    if ooze(w, id).corrosive {
        let items = w.mobile(m).map(|x| x.items.clone()).unwrap_or_default();
        let mut damaged = false;
        for i in items {
            let Some(hp) = hit_points(w, i) else { continue };
            if hp >= 10 && rnd::double(w) < 0.25 {
                let loss = if hp == 10 { rnd::min_max(w, 1, 5) } else { 10 };
                set_hit_points(w, i, hp - loss);
                damaged = true;
            }
        }
        if damaged {
            // The infernal ooze scorches you, setting you and your equipment ablaze!
            messages::local_overhead_message_number(w, m, MessageType::REGULAR, 0x21, 1072070, "");
            return;
        }
    }
    aos_helpers::damage(
        w,
        m,
        None,
        40,
        DamageSplit {
            phys: 0,
            fire: 0,
            cold: 0,
            pois: 100,
            nrgy: 0,
        },
        Default::default(),
    );
}

fn ooze_tick(w: &mut World, id: ItemId) {
    let Some((loc, map)) = w.item(id).map(|i| (i.location, i.map)) else {
        return;
    };
    let victims: Vec<MobileId> = muo_server::map::enumerate::mobiles_at(w, map, loc.x, loc.y)
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|&m| {
            let pet = cd(w, m).is_some_and(|c| c.controlled || c.summoned);
            (mx::player(w, m) || pet)
                && mx::alive(w, m)
                && !muo_server::mobile::base::is_dead_bonded_pet(w, m)
                && muo_server::mobile::base::can_be_damaged(w, m)
        })
        .collect();
    for m in victims {
        ooze_damage(w, id, m);
    }
    let mut d = ooze(w, id);
    d.ticks += 1;
    let ticks = d.ticks;
    set_ooze(w, id, d);
    if ticks >= 35 {
        ibase::delete(w, id);
    } else if ticks == 30 {
        ibase::set_item_id(w, id, 0x122B);
    }
}

pub static STAINED_OOZE: ItemType = ItemType {
    full_name: "Server.Mobiles.StainedOoze",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_stained_ooze(w, false)),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_bool(ooze(w, id).corrosive);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let corrosive = r.read_bool()?;
        set_ooze(
            w,
            id,
            OozeData {
                corrosive,
                ticks: 0,
                timer: TimerHandle::default(),
            },
        );
        // [AfterDeserialization]
        start_ooze_timer(w, id);
        let ticks = if w.item(id).is_some_and(|i| i.item_id == 0x122A) {
            0
        } else {
            30
        };
        let mut d = ooze(w, id);
        d.ticks = ticks;
        set_ooze(w, id, d);
        Ok(())
    },
    on_after_delete: |w, id| {
        (ITEM_BASE.on_after_delete)(w, id);
        let mut d = ooze(w, id);
        d.timer.cancel(w);
        set_ooze(w, id, d);
    },
    ..ITEM_BASE
};

// ---- DropOoze (Ilhenir and InterredGrizzle) ----

/// `GetSpawnPosition(range)`: two random draws around the creature, on the ground there.
fn spawn_position(w: &mut World, id: MobileId) -> Point3D {
    let here = mx::location(w, id);
    let Some(map) = mx::map(w, id) else {
        return here;
    };
    let x = here.x + rnd::min_max(w, -2, 2);
    let y = here.y + rnd::min_max(w, -2, 2);
    let z = w.maps.get_average_z(map, x, y);
    Point3D::new(x, y, z)
}

/// `DropOoze()`: one to three pools of ooze near it, all corrosive or all poisonous.
pub fn drop_ooze(w: &mut World, id: MobileId) {
    let amount = rnd::min_max(w, 1, 3);
    let corrosive = rnd::bool(w);
    for _ in 0..amount {
        let o = new_stained_ooze(w, corrosive);
        let mut p = mx::location(w, id);
        for _ in 0..5 {
            p = spawn_position(w, id);
            let map = mx::map(w, id);
            let at_location = muo_server::map::enumerate::items_at(w, map, p.x, p.y)
                .collect::<Vec<_>>()
                .into_iter()
                .any(|i| factory::item_is(w, i, "Server.Mobiles.StainedOoze"));
            if !at_location {
                break;
            }
        }
        if let Some(map) = mx::map(w, id) {
            ops::move_to_world_map(w, o, p, Some(map));
        }
    }
    if let Some(c) = mx::combatant(w, id) {
        if corrosive {
            mx::msg(w, c, 1072071); // A corrosive gas seeps out of your enemy's skin!
        } else {
            mx::msg(w, c, 1072072); // A poisonous gas seeps out of your enemy's skin!
        }
    }
}

pub fn interred_grizzle_on_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    if rnd::double(w) < 0.1 {
        drop_ooze(w, id);
    }
    (muo_content::creature::BASE_CREATURE.on_damage)(w, id, amount, from, will_kill);
}

// ---- Ilhenir ----

pub fn under_cacophonic_attack(w: &mut World, m: MobileId) -> bool {
    tables(w).cacophonic.contains(&m)
}

fn ilhenir_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &ILHENIR)
}

fn ilhenir_on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    use muo_server::item::container::drop_item_default;
    (BASE_CHAMPION.on_death)(w, id, c);
    if w.core_ml() {
        let Some(c) = c else { return };
        if let Some(i) = factory::make_item(w, &si("GrizzledBones")) {
            drop_item_default(w, c, i);
        }
        if rnd::double(w) < 0.05
            && let Some(i) = factory::make_item(w, &si("GrizzledMareStatuette"))
        {
            drop_item_default(w, c, i);
        }
        if rnd::double(w) < 0.025
            && let Some(i) = factory::make_item(w, &si("CrimsonCincture"))
        {
            drop_item_default(w, c, i);
        }
    }
}

fn ilhenir_on_gave_melee_attack(w: &mut World, id: MobileId, defender: MobileId, damage: i32) {
    (BASE_CHAMPION_C.on_gave_melee_attack)(w, id, defender, damage);
    if rnd::double(w) < 0.25 {
        cacophonic_attack(w, defender);
    }
}

/// `CacophonicAttack(to)`: walking speed for thirty seconds.
fn cacophonic_attack(w: &mut World, to: MobileId) {
    if mx::alive(w, to) && mx::player(w, to) && !under_cacophonic_attack(w, to) {
        if let Some(ns) = w.mobile(to).and_then(|m| m.net_state) {
            send_speed_control(w, ns, SpeedControlSetting::Walk);
        }
        mx::msg(w, to, 1072069); // A cacophonic sound lambastes you, suppressing your ability to move.
        mx::play_sound(w, to, 0x584);
        tables(w).cacophonic.insert(to);
        timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
            tables(w).cacophonic.remove(&to);
            if let Some(ns) = w.mobile(to).and_then(|m| m.net_state) {
                send_speed_control(w, ns, SpeedControlSetting::Disable);
            }
        });
    }
}

fn ilhenir_on_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    if rnd::double(w) < 0.1 {
        drop_ooze(w, id);
    }
    (BASE_CHAMPION.on_damage)(w, id, amount, from, will_kill);
}

creature_leaf! {
    /// `Server.Mobiles.Ilhenir`.
    ILHENIR, ILHENIR_C = "Server.Mobiles.Ilhenir" : BASE_CHAMPION / BASE_CHAMPION_C, ctor Some(ilhenir_new), encoded false;
    c {
        corpse_name: muo_content::templates::creature_props::corpse_name,
        default_name: muo_content::templates::creature_props::default_name,
        unprovokable: crate::cprops::unprovokable,
        uncalmable: crate::cprops::uncalmable,
        poison_immune: muo_content::templates::creature_props::poison_immune,
        treasure_map_level: muo_content::templates::creature_props::treasure_map_level,
        generate_loot: muo_content::templates::creature_props::generate_loot,
        on_gave_melee_attack: ilhenir_on_gave_melee_attack,
    }
    m {
        get_anger_sound: crate::cprops::get_anger_sound,
        get_idle_sound: crate::cprops::get_idle_sound,
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
        on_death: ilhenir_on_death,
        on_damage: ilhenir_on_damage,
    }
}

// ---- Meraktus ----

/// `_followers` (null until the first time it is on a map).
#[derive(Clone, Default)]
struct Followers(Option<Vec<MobileId>>);

fn followers(w: &World, id: MobileId) -> Option<Vec<MobileId>> {
    w.mobile(id)
        .and_then(|m| m.ext.get::<Followers>())
        .and_then(|f| f.0.clone())
}
fn set_followers(w: &mut World, id: MobileId, f: Option<Vec<MobileId>>) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(Followers(f));
    }
}

pub(crate) fn meraktus_no_followers(w: &mut World, id: MobileId) {
    set_followers(w, id, None);
}

fn meraktus_new(w: &mut World) -> MobileId {
    crate::cprops::new_creature(w, &MERAKTUS)
}

pub fn meraktus_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(1);
    let list = followers(w, id).unwrap_or_default();
    out.write_encoded_int(list.len() as i32);
    for m in list {
        out.write_mobile(w, Some(m));
    }
}

pub fn meraktus_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let version = r.read_i32()?;
    if version == 0 {
        // MigrateFrom(V0Content): nothing was saved
        return Ok(());
    }
    // The array's slots are read as they are: a gone follower stays its slot (written back as -1).
    let list = r.read_mobile_list_kept(true)?;
    set_followers(w, id, Some(list));
    Ok(())
}

/// `OnMapChange(oldMap)`: no `base` call. The first time on a map four tormented minotaurs come along; after that
/// the living ones follow it.
fn meraktus_on_map_change(w: &mut World, id: MobileId, old_map: Option<MapId>) {
    let (loc, map) = (mx::location(w, id), mx::map(w, id));
    match followers(w, id) {
        None => {
            if map.is_some_and(|m| m != MapId::INTERNAL)
                && old_map.is_none_or(|m| m == MapId::INTERNAL)
            {
                let mut list = vec![];
                for _ in 0..4 {
                    if let Some(f) = factory::make_mobile(w, "Server.Mobiles.TormentedMinotaur") {
                        list.push(f);
                        if let Some(map) = map {
                            muo_server::mobile::base::move_to_world(w, f, loc, map);
                        }
                    }
                }
                set_followers(w, id, Some(list));
            }
        }
        Some(list) => {
            for f in list {
                if !mx::deleted(w, f)
                    && let Some(map) = map
                {
                    muo_server::mobile::base::move_to_world(w, f, loc, map);
                }
            }
        }
    }
}

fn meraktus_on_after_delete(w: &mut World, id: MobileId) {
    (BASE_CHAMPION.on_after_delete)(w, id);
    for f in followers(w, id).unwrap_or_default() {
        if let Some(ty) = w.mobile(f).map(|m| m.ty) {
            (ty.delete)(w, f);
        }
    }
}

fn meraktus_on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    use muo_server::item::container::drop_item_default;
    (BASE_CHAMPION.on_death)(w, id, c);
    if !w.core_ml() {
        return;
    }
    let Some(c) = c else { return };
    if let Some(i) = factory::make_item(w, &si("MalletAndChisel")) {
        drop_item_default(w, c, i);
    }
    let n = ["MinotaurHedge", "LightYarn", "BonePile"][rnd::below(w, 3) as usize];
    if let Some(i) = factory::make_item(w, &si(n)) {
        drop_item_default(w, c, i);
    }
    if rnd::bool(w)
        && let Some(i) = factory::make_item(w, &si("TormentedChains"))
    {
        drop_item_default(w, c, i);
    }
    if rnd::double(w) < 0.025
        && let Some(i) = factory::make_item(w, &si("CrimsonCincture"))
    {
        drop_item_default(w, c, i);
    }
}

fn meraktus_on_gave_melee_attack(w: &mut World, id: MobileId, defender: MobileId, damage: i32) {
    (BASE_CHAMPION_C.on_gave_melee_attack)(w, id, defender, damage);
    if rnd::double(w) < 0.2 {
        earthquake(w, id);
    }
}

/// `Earthquake()`: the players and hostile pets within 8 tiles are dismounted and hit.
pub fn earthquake(w: &mut World, id: MobileId) {
    let team = cd(w, id).map_or(0, |d| d.team);
    let mut queue = vec![];
    for m in enumerators::get_mobiles_in_range(w, id, 8) {
        if mx::deleted(w, m) || m == id || !mx::can_be_harmful(w, id, m) {
            continue;
        }
        let pet = cd(w, m).is_some_and(|c| c.controlled || c.summoned || c.team != team);
        if mx::player(w, m) || pet {
            queue.push(m);
        }
    }
    for m in queue {
        if muo_content::player_mobile::data(w, m).is_some() && props_b::mounted(w, m) {
            muo_content::creature::mount::dismount(w, m);
        }
        let damage = ((f64::from(mx::hits(w, m)) * 0.6) as i32).clamp(10, 75);
        muo_server::mobile::base::do_harmful(w, id, m, false);
        aos_helpers::damage(
            w,
            m,
            Some(id),
            damage,
            DamageSplit {
                phys: 100,
                fire: 0,
                cold: 0,
                pois: 0,
                nrgy: 0,
            },
            Default::default(),
        );
        let human = muo_content::creature::misc::body_is_human(w, m);
        if mx::alive(w, m) && human && !props_b::mounted(w, m) {
            mx::animate(w, m, 20, 7, 1, true, false, 0); // take hit
        }
    }
}

creature_leaf! {
    /// `Server.Mobiles.Meraktus`.
    MERAKTUS, MERAKTUS_C = "Server.Mobiles.Meraktus" : BASE_CHAMPION / BASE_CHAMPION_C, ctor Some(meraktus_new), state meraktus_serialize, meraktus_deserialize;
    c {
        corpse_name: muo_content::templates::creature_props::corpse_name,
        default_name: muo_content::templates::creature_props::default_name,
        meat: muo_content::templates::creature_props::meat,
        hides: muo_content::templates::creature_props::hides,
        hide_type: crate::cprops::hide_type,
        poison_immune: muo_content::templates::creature_props::poison_immune,
        treasure_map_level: muo_content::templates::creature_props::treasure_map_level,
        bard_immune: crate::cprops::bard_immune,
        unprovokable: crate::cprops::unprovokable,
        uncalmable: crate::cprops::uncalmable,
        get_weapon_ability: crate::cprops::get_weapon_ability,
        generate_loot: crate::cprops::generate_loot,
        on_gave_melee_attack: meraktus_on_gave_melee_attack,
    }
    m {
        get_anger_sound: crate::cprops::get_anger_sound,
        get_idle_sound: crate::cprops::get_idle_sound,
        get_attack_sound: crate::cprops::get_attack_sound,
        get_hurt_sound: crate::cprops::get_hurt_sound,
        get_death_sound: crate::cprops::get_death_sound,
        on_death: meraktus_on_death,
        on_map_change: meraktus_on_map_change,
        on_after_delete: meraktus_on_after_delete,
    }
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&STAINED_OOZE, &[]);
    types.register_mobile(&ILHENIR, &[]);
    types.register_mobile(&MERAKTUS, &[]);
}
