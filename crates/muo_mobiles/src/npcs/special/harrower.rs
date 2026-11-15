//! The Harrower and its tentacles.
//!
//! `Harrower` is `[SerializationGenerator(1, false)]` (version 0 had the same three fields with a count-int
//! tentacle list); `HarrowerTentacles` is `(0, false)` and writes its harrower.

use indexmap::IndexMap;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, cd};
use muo_content::hooks::rnd;
use muo_content::spells::helper::{Point3dRef, turn};
use muo_content::{loot, mx};
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::effect_item;
use muo_server::map::MapId;
use muo_server::mobile::part1::{lifecycle, props_a, props_b};
use muo_server::mobile::part2::{effects, items, messages};
use muo_server::mobile::{MOBILE_BASE, MobileType};
use muo_server::poison::PoisonId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::champions::{goodies_timer, is_player_mobile, justice_reward, rights_holders};
use super::hooks;
use crate::common::*;

const LETHAL: PoisonId = PoisonId(4);

/// `m_Entries`: where the harrower appears and the dungeon entrance its gate opens to.
const ENTRIES: [([i32; 3], [i32; 3]); 15] = [
    ([5242, 945, -40], [1176, 2638, 0]),  // Destard
    ([5225, 798, 0], [1176, 2638, 0]),    // Destard
    ([5556, 886, 30], [1298, 1080, 0]),   // Despise
    ([5187, 615, 0], [4111, 432, 5]),     // Deceit
    ([5319, 583, 0], [4111, 432, 5]),     // Deceit
    ([5713, 1334, -1], [2923, 3407, 8]),  // Fire
    ([5860, 1460, -2], [2923, 3407, 8]),  // Fire
    ([5328, 1620, 0], [5451, 3143, -60]), // Terathan Keep
    ([5690, 538, 0], [2042, 224, 14]),    // Wrong
    ([5609, 195, 0], [514, 1561, 0]),     // Shame
    ([5475, 187, 0], [514, 1561, 0]),     // Shame
    ([6085, 179, 0], [4721, 3822, 0]),    // Hythloth
    ([6084, 66, 0], [4721, 3822, 0]),     // Hythloth
    ([5499, 2003, 0], [2499, 919, 0]),    // Covetous
    ([5579, 1858, 0], [2499, 919, 0]),    // Covetous
];

fn p3(a: [i32; 3]) -> Point3D {
    Point3D::new(a[0], a[1], a[2])
}

/// `Harrower.Instances`.
#[derive(Default)]
pub struct HarrowerStatics {
    pub instances: Vec<MobileId>,
}

/// `Harrower.CanSpawn`.
pub fn can_spawn(w: &World) -> bool {
    w.resources
        .get::<HarrowerStatics>()
        .is_none_or(|s| s.instances.is_empty())
}

/// What `Harrower` adds to a mobile: its fields and the teleport timer.
#[derive(Default)]
pub struct HarrowerData {
    pub true_form: bool,
    pub gate_item: Option<ItemId>,
    /// `None` until the first tentacle is made.
    pub tentacles: Option<Vec<MobileId>>,
    /// `m_DamageEntries`.
    damage_entries: IndexMap<MobileId, i32>,
    timer: TimerHandle,
}

pub fn data(w: &World, id: MobileId) -> Option<&HarrowerData> {
    w.mobile(id)?.ext.get::<HarrowerData>()
}

fn data_mut(w: &mut World, id: MobileId) -> Option<&mut HarrowerData> {
    w.mobile_mut(id)?.ext.get_mut::<HarrowerData>()
}

fn start_teleport_timer(w: &mut World, id: MobileId) {
    let five = TimeSpan::from_secs(5);
    let h = timer::start(w, five, five, 0, move |w| teleport_tick(w, id));
    if let Some(d) = data_mut(w, id) {
        d.timer = h;
    }
}

fn stop_own_timer(w: &mut World, id: MobileId) {
    let mut t = data_mut(w, id)
        .map(|d| std::mem::take(&mut d.timer))
        .unwrap_or_default();
    t.cancel(w);
}

fn harrower_new(w: &mut World) -> MobileId {
    let args = muo_content::templates::table(w).roll_args(w, HARROWER.full_name);
    let id = new_creature(
        w,
        &HARROWER,
        args.ai,
        args.fight_mode,
        args.range_perception,
        args.range_fight,
    );
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(HarrowerData::default());
    }
    w.resources.get_mut::<HarrowerStatics>().instances.push(id);
    muo_content::templates::apply_mobile(w, id);
    start_teleport_timer(w, id);
    id
}

/// `Harrower.Spawn(platLoc, platMap)`.
pub fn spawn(w: &mut World, plat_loc: Point3D, plat_map: Option<MapId>) -> Option<MobileId> {
    if !can_spawn(w) {
        return None;
    }
    let (loc, entrance) = *muo_primitives::utility::random_element(&mut w.rng, &ENTRIES)?;
    let harrower = harrower_new(w);
    muo_server::mobile::base::move_to_world(w, harrower, p3(loc), MapId::FELUCCA);
    let gate = hooks::get(w).and_then(|h| h.harrower_gate);
    let gate = gate.and_then(|f| {
        f(
            w,
            harrower,
            plat_loc,
            plat_map,
            p3(entrance),
            MapId::FELUCCA,
        )
    });
    if let Some(d) = data_mut(w, harrower) {
        d.gate_item = gate;
    }
    Some(harrower)
}

/// `Morph()`.
pub fn morph(w: &mut World, id: MobileId) {
    if data(w, id).is_none_or(|d| d.true_form) {
        return;
    }
    if let Some(d) = data_mut(w, id) {
        d.true_form = true;
    }
    name(w, id, Some("the true harrower".to_owned()));
    body(w, id, 780);
    hue(w, id, 0x497);
    let ty = w.mobile(id).map(|m| m.ty).unwrap_or(&MOBILE_BASE);
    let (hm, sm, mm) = (
        (ty.hits_max)(w, id),
        (ty.stam_max)(w, id),
        (ty.mana_max)(w, id),
    );
    props_b::set_hits(w, id, hm);
    props_b::set_stam(w, id, sm);
    props_b::set_mana(w, id, mm);
    muo_server::mobile::base::process_delta(w, id);
    messages::say_number(w, id, 1049499, ""); // Behold my true form!

    let Some(map) = mx::map(w, id) else { return };
    let loc = mx::location(w, id);
    for i in 0..9 {
        let angle = f64::from(i * 40) / 180.0 * std::f64::consts::PI;
        let (rx, ry) = (angle.cos(), angle.sin());
        let mut dist = 0;
        let mut ok = false;
        let (mut x, mut y, mut z) = (0, 0, 0);
        while !ok && dist < 10 {
            let rdist = 10 + dist;
            x = loc.x + (rx * f64::from(rdist)) as i32;
            y = loc.y + (ry * f64::from(rdist)) as i32;
            z = w.maps.get_average_z(map, x, y);
            ok = muo_server::map::fit::can_fit(w, map, x, y, loc.z, 16, false, false, true);
            if !ok {
                ok = muo_server::map::fit::can_fit(w, map, x, y, z, 16, false, false, true);
            }
            dist = if dist >= 0 { -(dist + 1) } else { -(dist - 1) };
        }
        if !ok {
            continue;
        }
        let spawn = tentacles_new(w, Some(id));
        let team = cd(w, id).map_or(0, |d| d.team);
        if let Some(d) = muo_content::creature::data::cd_mut(w, spawn) {
            d.team = team;
        }
        muo_server::mobile::base::move_to_world(w, spawn, Point3D::new(x, y, z), map);
        if let Some(d) = data_mut(w, id) {
            d.tentacles.get_or_insert_with(Vec::new).push(spawn);
        }
    }
}

/// `TeleportTimer.OnTick`.
fn teleport_tick(w: &mut World, owner: MobileId) {
    if mx::deleted(w, owner) {
        stop_own_timer(w, owner);
        return;
    }
    let Some(map) = mx::map(w, owner) else { return };
    if rnd::double(w) < 0.75 {
        return;
    }
    let ty = w.mobile(owner).map(|m| m.ty).unwrap_or(&MOBILE_BASE);
    let mut to_teleport = None;
    for m in mx::mobiles_in_range(w, owner, 16) {
        if m != owner
            && mx::player(w, m)
            && mx::can_be_harmful(w, owner, m)
            && (ty.can_see_mobile)(w, owner, m)
        {
            to_teleport = Some(m);
            break;
        }
    }
    let Some(target) = to_teleport else { return };
    const OFFSETS: [i32; 16] = [-1, -1, -1, 0, -1, 1, 0, -1, 0, 1, 1, -1, 1, 0, 1, 1];
    let offset = (rnd::below(w, 8) * 2) as usize;
    let here = mx::location(w, owner);
    let mut to = here;
    for i in (0..OFFSETS.len()).step_by(2) {
        let x = here.x + OFFSETS[(offset + i) % OFFSETS.len()];
        let y = here.y + OFFSETS[(offset + i + 1) % OFFSETS.len()];
        if muo_server::map::fit::can_spawn_mobile_at(w, map, x, y, here.z) {
            to = Point3D::new(x, y, here.z);
            break;
        }
        let z = w.maps.get_average_z(map, x, y);
        if muo_server::map::fit::can_spawn_mobile_at(w, map, x, y, z) {
            to = Point3D::new(x, y, z);
            break;
        }
    }
    let from = mx::location(w, target);
    let tmap = mx::map(w, target);
    lifecycle::set_location(w, target, to);
    turn(w, owner, Some(Point3dRef::Entity(EntityId::Mobile(target))));
    turn(w, target, Some(Point3dRef::Entity(EntityId::Mobile(owner))));
    muo_server::mobile::base::process_delta(w, target);
    for (loc, effect) in [(from, 2023), (to, 5023)] {
        let e = effect_item::create(w, loc, tmap, effect_item::DEFAULT_DURATION);
        muo_server::effects::send_location_particles(w, e, 0x3728, 10, 10, effect);
    }
    mx::play_sound(w, target, 0x1FE);
    mx::set_combatant(w, owner, Some(target));
}

/// `GivePowerScrolls()`.
fn give_power_scrolls(w: &mut World, id: MobileId) {
    let mut to_give = rights_holders(w, id);
    if to_give.is_empty() {
        return;
    }
    muo_primitives::random::Rng::shuffle(&mut w.rng, &mut to_give);
    for i in 0..16 {
        let r = rnd::double(w);
        let level = if r < 0.1 {
            25
        } else if r < 0.25 {
            20
        } else if r < 0.45 {
            15
        } else if r < 0.70 {
            10
        } else {
            5
        };
        let m = to_give[i % to_give.len()];
        mx::msg(w, m, 1049524); // You have received a scroll of power!
        let s = muo_items::special::scrolls::stat_cap_scroll_new(w, 225 + level);
        items::add_to_backpack(w, m, s);
        if !is_player_mobile(w, m) {
            continue;
        }
        if let Some(prot) = justice_reward(w, m) {
            mx::msg(w, prot, 1049368); // You have been rewarded for your dedication to Justice!
            let s = muo_items::special::scrolls::stat_cap_scroll_new(w, 225 + level);
            items::add_to_backpack(w, prot, s);
        }
    }
}

fn on_before_death(w: &mut World, id: MobileId) -> bool {
    if !data(w, id).is_some_and(|d| d.true_form) {
        morph(w, id);
        return false;
    }
    for m in rights_holders(w, id) {
        if is_player_mobile(w, m)
            && let Some(f) = hooks::get(w).and_then(|h| h.award_harrower_title)
        {
            f(w, m);
        }
    }
    if !cd(w, id).is_some_and(|d| d.no_kill_awards) {
        give_power_scrolls(w, id);
        if let Some(map) = mx::map(w, id) {
            let loc = mx::location(w, id);
            for x in -16..=16 {
                for y in -16..=16 {
                    if f64::from(x * x + y * y).sqrt() <= 16.0 {
                        goodies_timer(w, map, loc.x + x, loc.y + y, (750, 1250));
                    }
                }
            }
        }
        if let Some(d) = data_mut(w, id) {
            d.damage_entries = IndexMap::new();
        }
        // No tentacle ever fitting is treated as an empty list.
        let tentacles = data_mut(w, id)
            .and_then(|d| d.tentacles.take())
            .unwrap_or_default();
        if let Some(d) = data_mut(w, id) {
            d.tentacles = Some(Vec::new());
        }
        for m in tentacles {
            if !mx::deleted(w, m) {
                muo_server::mobile::base::kill(w, m);
            }
            register_damage_to(w, id, Some(m));
        }
        register_damage_to(w, id, Some(id));
        let artifact = get_artifact(w);
        award_artifact(w, id, artifact);
        if let Some(g) = data(w, id).and_then(|d| d.gate_item) {
            muo_server::item::base::delete(w, g);
        }
    }
    (BASE_CREATURE.on_before_death)(w, id)
}

/// `RegisterDamageTo(m)`.
fn register_damage_to(w: &mut World, id: MobileId, m: Option<MobileId>) {
    let Some(m) = m else { return };
    let entries: Vec<(MobileId, i32)> = props_a::damage_entries(w, m)
        .iter()
        .map(|e| (e.damager, e.damage_given))
        .collect();
    for (damager, given) in entries {
        let ty = w.mobile(damager).map(|x| x.ty).unwrap_or(&MOBILE_BASE);
        let master = (ty.get_damage_master)(w, damager, m);
        register_damage(w, id, master.unwrap_or(damager), given);
    }
}

/// `RegisterDamage(from, amount)`.
fn register_damage(w: &mut World, id: MobileId, from: MobileId, amount: i32) {
    if !mx::player(w, from) {
        return;
    }
    let Some(d) = data_mut(w, id) else { return };
    let total = amount + d.damage_entries.get(&from).copied().unwrap_or(0);
    d.damage_entries.insert(from, total);
    mx::send_message(w, from, &format!("Total Damage: {total}"));
}

/// `AwardArtifact(artifact)`.
fn award_artifact(w: &mut World, id: MobileId, artifact: Option<ItemId>) {
    let Some(artifact) = artifact else { return };
    let entries: Vec<(MobileId, i32)> = data(w, id)
        .map(|d| d.damage_entries.iter().map(|(k, v)| (*k, *v)).collect())
        .unwrap_or_default();
    let mut total = 0;
    let mut valid = Vec::new();
    for (m, v) in entries {
        if is_eligible(w, id, m, artifact) {
            valid.push((m, v));
            total += v;
        }
    }
    let random = rnd::min_max(w, 1, total);
    total = 0;
    for (m, v) in valid {
        total += v;
        if total >= random {
            give_artifact(w, m, artifact);
            return;
        }
    }
    muo_server::item::base::delete(w, artifact);
}

/// `GiveArtifact(to, artifact)`.
fn give_artifact(w: &mut World, to: MobileId, artifact: ItemId) {
    let pack = props_b::backpack(w, to);
    if pack.is_some_and(|p| {
        muo_server::item::container::try_drop_item(w, p, Some(to), artifact, false)
    }) {
        mx::msg(w, to, 1062317); // For your valor in combating the fallen beast, a special artifact has been bestowed on you.
    } else {
        muo_server::item::base::delete(w, artifact);
    }
}

/// `IsEligible(m, artifact)`.
fn is_eligible(w: &mut World, id: MobileId, m: MobileId, artifact: ItemId) -> bool {
    let loc = mx::location(w, id);
    if !(mx::player(w, m) && mx::alive(w, m) && mx::in_range_point(w, m, loc, 32)) {
        return false;
    }
    props_b::backpack(w, m)
        .is_some_and(|p| muo_server::item::container::check_hold(w, p, Some(m), artifact, false))
}

const UNIQUE: [&str; 1] = ["Server.Items.AcidProofRobe"];
const SHARED: [&str; 1] = ["Server.Items.TheRobeOfBritanniaAri"];
const DECORATIVE: [&str; 2] = ["Server.Items.EvilIdolSkull", "Server.Items.SkullPole"];

/// `GetArtifact()`.
fn get_artifact(w: &mut World) -> Option<ItemId> {
    let r = rnd::double(w);
    let list: &[&str] = if r < 0.05 {
        &UNIQUE
    } else if r < 0.15 {
        &SHARED
    } else if r < 0.30 {
        &DECORATIVE
    } else {
        return None;
    };
    let name = *muo_primitives::utility::random_element(&mut w.rng, list)?;
    loot::construct(w, name)
}

fn on_after_delete(w: &mut World, id: MobileId) {
    w.resources
        .get_mut::<HarrowerStatics>()
        .instances
        .retain(|&m| m != id);
    (BASE_CREATURE.on_after_delete)(w, id);
}

fn harrower_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(1);
    let d = data(w, id);
    out.write_bool(d.is_some_and(|d| d.true_form));
    out.write_item(w, d.and_then(|d| d.gate_item));
    let list = d.and_then(|d| d.tentacles.as_deref()).unwrap_or(&[]);
    out.write_encoded_int(list.len() as i32);
    for &t in list {
        out.write_mobile(w, Some(t));
    }
}

fn harrower_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let version = r.read_i32()?;
    let true_form = r.read_bool()?;
    let gate_item = r.read_item(w)?;
    let tentacles = if version < 1 {
        r.read_mobile_list(w)?
    } else {
        r.read_mobile_list_kept(true)?
    };
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(HarrowerData {
            true_form,
            gate_item,
            tentacles: Some(tentacles),
            ..HarrowerData::default()
        });
    }
    w.resources.get_mut::<HarrowerStatics>().instances.push(id);
    // [AfterDeserialization]
    start_teleport_timer(w, id);
    Ok(())
}

creature_leaf! {
    /// `Server.Mobiles.Harrower`.
    HARROWER, HARROWER_C = "Server.Mobiles.Harrower" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(harrower_new), state harrower_serialize, harrower_deserialize;
    c {
        default_name: |_w, _id| Some("the harrower"),
        auto_dispel: |_w, _id| true,
        unprovokable: |_w, _id| true,
        poison_immune: |_w, _id| Some(LETHAL),
        disallow_all_moves: |w, id| data(w, id).is_some_and(|d| d.true_form),
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::super_boss(w), 2);
            add_loot(w, id, loot_pack::meager(w));
        },
    }
    m {
        hits_max: |w, id| if data(w, id).is_some_and(|d| d.true_form) { 65000 } else { 30000 },
        mana_max: |_w, _id| 5000,
        on_before_death: on_before_death,
        on_after_delete: on_after_delete,
    }
}

// ---- HarrowerTentacles ----

/// What `HarrowerTentacles` adds: `_harrower` and the drain timer.
#[derive(Default)]
pub struct TentaclesData {
    pub harrower: Option<MobileId>,
    timer: TimerHandle,
}

fn tdata(w: &World, id: MobileId) -> Option<&TentaclesData> {
    w.mobile(id)?.ext.get::<TentaclesData>()
}

fn tdata_mut(w: &mut World, id: MobileId) -> Option<&mut TentaclesData> {
    w.mobile_mut(id)?.ext.get_mut::<TentaclesData>()
}

fn start_drain_timer(w: &mut World, id: MobileId) {
    let five = TimeSpan::from_secs(5);
    let h = timer::start(w, five, five, 0, move |w| drain_tick(w, id));
    if let Some(d) = tdata_mut(w, id) {
        d.timer = h;
    }
}

fn stop_drain_timer(w: &mut World, id: MobileId) {
    let mut t = tdata_mut(w, id)
        .map(|d| std::mem::take(&mut d.timer))
        .unwrap_or_default();
    t.cancel(w);
}

/// `new HarrowerTentacles(harrower)`.
pub fn tentacles_new(w: &mut World, harrower: Option<MobileId>) -> MobileId {
    let args = muo_content::templates::table(w).roll_args(w, HARROWER_TENTACLES.full_name);
    let id = new_creature(
        w,
        &HARROWER_TENTACLES,
        args.ai,
        args.fight_mode,
        args.range_perception,
        args.range_fight,
    );
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(TentaclesData {
            harrower,
            ..TentaclesData::default()
        });
    }
    muo_content::templates::apply_mobile(w, id);
    start_drain_timer(w, id);
    id
}

fn tentacles_ctor(w: &mut World) -> MobileId {
    tentacles_new(w, None)
}

/// `DrainTimer.OnTick`.
fn drain_tick(w: &mut World, owner: MobileId) {
    if mx::deleted(w, owner) {
        stop_drain_timer(w, owner);
        return;
    }
    let team = cd(w, owner).map_or(0, |d| d.team);
    let mut queue = Vec::new();
    for m in mx::mobiles_in_range(w, owner, 9) {
        if m == owner || !mx::can_be_harmful(w, owner, m) {
            continue;
        }
        if mx::player(w, m) && !mx::alive(w, m) {
            continue;
        }
        if let Some(d) = cd(w, m) {
            let animated =
                muo_content::creature::ctype(w, m).is_some_and(|c| (c.is_animated_dead)(w, m));
            if !(d.controlled || animated || d.summoned || d.team != team) {
                continue;
            }
        }
        queue.push(m);
    }
    for m in queue {
        mx::do_harmful(w, owner, m);
        effects::fixed_particles_hue(w, m, 0x374A, 10, 15, 5013, 0x455, 0, EffectLayer::Waist);
        mx::play_sound(w, m, 0x1F1);
        let drain = rnd::min_max(w, 14, 30);
        let hits = mx::hits(w, owner);
        props_b::set_hits(w, owner, hits + drain);
        if let Some(h) = tdata(w, owner).and_then(|d| d.harrower) {
            let hits = mx::hits(w, h);
            props_b::set_hits(w, h, hits + drain);
        }
        mx::damage(w, m, drain, Some(owner));
    }
}

fn tentacles_after_delete(w: &mut World, id: MobileId) {
    stop_drain_timer(w, id);
    (BASE_CREATURE.on_after_delete)(w, id);
    // `Harrower?.RemoveFromTentacles(this)`
    if let Some(h) = tdata(w, id).and_then(|d| d.harrower)
        && let Some(list) = data_mut(w, h).and_then(|d| d.tentacles.as_mut())
    {
        list.retain(|&t| t != id);
    }
}

fn tentacles_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
    out.write_mobile(w, tdata(w, id).and_then(|d| d.harrower));
}

fn tentacles_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    let harrower = r.read_mobile(w)?;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(TentaclesData {
            harrower,
            ..TentaclesData::default()
        });
    }
    // [AfterDeserialization]
    start_drain_timer(w, id);
    Ok(())
}

creature_leaf! {
    /// `Server.Mobiles.HarrowerTentacles`.
    HARROWER_TENTACLES, HARROWER_TENTACLES_C = "Server.Mobiles.HarrowerTentacles" : BASE_CREATURE / BASE_CREATURE_C, ctor Some(tentacles_ctor), state tentacles_serialize, tentacles_deserialize;
    c {
        corpse_name: |_w, _id| Some("a tentacles corpse"),
        default_name: |_w, _id| Some("tentacles of the harrower"),
        auto_dispel: |_w, _id| true,
        unprovokable: |_w, _id| true,
        poison_immune: |_w, _id| Some(LETHAL),
        disallow_all_moves: |_w, _id| true,
        check_reflect: |_w, _id, _caster, reflect| *reflect = true,
        generate_loot: |w, id| {
            add_loot_n(w, id, loot_pack::filthy_rich(w), 2);
            add_loot_n(w, id, &loot_pack::MED_SCROLLS, 3);
            add_loot_n(w, id, &loot_pack::HIGH_SCROLLS, 2);
        },
    }
    m {
        get_idle_sound: |_w, _id| 0x101,
        get_anger_sound: |_w, _id| 0x5E,
        get_death_sound: |_w, _id| 0x1C2,
        get_attack_sound: |_w, _id| -1,
        get_hurt_sound: |_w, _id| 0x289,
        on_after_delete: tentacles_after_delete,
    }
}

pub(crate) fn all() -> Vec<&'static MobileType> {
    vec![&HARROWER, &HARROWER_TENTACLES]
}

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in all() {
        types.register_mobile(t, &[]);
    }
}
