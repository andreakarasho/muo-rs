//! OnDeath, OnBeforeDeath, OnDamage, OnGaveMeleeAttack, AlterMeleeDamageTo and other event overrides of monsters
//! and animals.

use muo_content::creature::enums::{FightMode, OrderType};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, cd, control, mount};
use muo_content::factory;
use muo_content::items::weapon_hooks;
use muo_content::misc::aos_helpers::{self, DamageSplit};
use muo_content::mx;
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::container::drop_item_default;
use muo_server::item::ops;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::{effects, items, messages};

use super::ctors::si;
use crate::common::rnd;

fn summoned(w: &World, id: MobileId) -> bool {
    cd(w, id).is_some_and(|c| c.summoned)
}
fn no_kill_awards(w: &World, id: MobileId) -> bool {
    cd(w, id).is_some_and(|c| c.no_kill_awards)
}

// ---- the DemonKnight family: `base.OnDeath(c); if (!Summoned && !NoKillAwards && CheckArtifactChance(this)) ..` ----

macro_rules! artifact_on_death {
    ($name:ident) => {
        pub fn $name(w: &mut World, id: MobileId, c: Option<ItemId>) {
            (BASE_CREATURE.on_death)(w, id, c);
            if !summoned(w, id)
                && !no_kill_awards(w, id)
                && crate::classes::demon_knight::check_artifact_chance(w, id)
            {
                crate::classes::demon_knight::distribute_artifact_from(w, id);
            }
        }
    };
}
artifact_on_death!(abysmal_horror_on_death);
artifact_on_death!(darknight_creeper_on_death);
artifact_on_death!(flesh_renderer_on_death);
artifact_on_death!(impaler_on_death);

/// `base.OnDeath(c); c.DropItem(new GamanHorns(Utility.RandomBool() ? 1 : 2));`
pub fn gaman_on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    (BASE_CREATURE.on_death)(w, id, c);
    let amount = if rnd::bool(w) { 1 } else { 2 };
    if let (Some(c), Some(i)) = (c, factory::make_item_amount(w, &si("GamanHorns"), amount)) {
        drop_item_default(w, c, i);
    }
}

/// `base.OnDeath(c);` then 5% of a DawnsMusicGear (75% common, else uncommon; a paragon's is rare).
pub fn juggernaut_on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    super::batch4::dawns_gear_on_death(w, id, c);
}

// ---- OnBeforeDeath ----

/// `var mount = Mount; if (mount != null) mount.Rider = null;`
fn unmount(w: &mut World, id: MobileId) -> Option<MobileId> {
    let m = props_b::mount(w, id);
    if let Some(m) = m {
        mount::set_rider(w, m, None);
    }
    m
}

/// `mount.Rider = null; if (mount is Mobile mobile) mobile.Delete(); return base.OnBeforeDeath();`
pub fn savage_rider_on_before_death(w: &mut World, id: MobileId) -> bool {
    if let Some(m) = unmount(w, id) {
        muo_server::mobile::base::delete(w, m);
    }
    (BASE_CREATURE.on_before_death)(w, id)
}
pub fn chaos_dragoon_on_before_death(w: &mut World, id: MobileId) -> bool {
    unmount(w, id);
    (BASE_CREATURE.on_before_death)(w, id)
}
pub fn chaos_dragoon_elite_on_before_death(w: &mut World, id: MobileId) -> bool {
    if let Some(m) = props_b::mount(w, id) {
        if factory::mobile_is(w, m, "Server.Mobiles.SwampDragon") {
            crate::classes::swamp_dragon::set_has_barding(w, m, false);
        }
        mount::set_rider(w, m, None);
    }
    (BASE_CREATURE.on_before_death)(w, id)
}

fn spark(w: &mut World, id: MobileId) {
    muo_server::effects::send_location_effect(
        w,
        mx::location(w, id),
        mx::map(w, id),
        0x376A,
        10,
        1,
        0,
        0,
    );
}

/// `if (!base.OnBeforeDeath()) return false; new Gold(240..375).MoveToWorld(Location, Map); effect; return true;`
pub fn spectral_armour_on_before_death(w: &mut World, id: MobileId) -> bool {
    if !(BASE_CREATURE.on_before_death)(w, id) {
        return false;
    }
    let n = rnd::min_max(w, 240, 375);
    if let Some(g) = factory::make_item_amount(w, &si("Gold"), n) {
        ops::move_to_world_map(w, g, mx::location(w, id), mx::map(w, id));
    }
    spark(w, id);
    true
}
/// As SpectralArmour's without the gold.
pub fn corrupted_soul_on_before_death(w: &mut World, id: MobileId) -> bool {
    if !(BASE_CREATURE.on_before_death)(w, id) {
        return false;
    }
    spark(w, id);
    true
}

/// `if (Core.T2A && Utility.RandomDouble() < .05) PackItem(new MessageInABottle(Map == Felucca ? Felucca : Trammel));`
pub fn kraken_on_before_death(w: &mut World, id: MobileId) -> bool {
    use muo_server::map::MapId;
    if w.expansion() >= muo_data::expansion::Expansion::T2A && rnd::double(w) < 0.05 {
        let map = if mx::map(w, id) == Some(MapId::FELUCCA) {
            MapId::FELUCCA
        } else {
            MapId::TRAMMEL
        };
        let it = factory::make_item_args(
            w,
            &si("MessageInABottle"),
            &[muo_content::props::Value::Map(Some(map))],
        );
        crate::common::pack_item(w, id, it);
    }
    (BASE_CREATURE.on_before_death)(w, id)
}

/// The Khaldun bone mages and knights: the creature turns into a `rm` carrying its loot and deletes itself.
fn khaldun_before_death(w: &mut World, id: MobileId, class: &str) -> bool {
    let Some(rm) = factory::make_mobile(w, class) else {
        return false;
    };
    let team = cd(w, id).map_or(0, |d| d.team);
    if let Some(d) = muo_content::creature::data::cd_mut(w, rm) {
        d.team = team;
    }
    mx::set_combatant(w, rm, mx::combatant(w, id));
    if let Some(d) = muo_content::creature::data::cd_mut(w, rm) {
        d.no_kill_awards = true;
    }
    let pack = match items::find_item_on_layer(w, rm, muo_server::layer::Layer::Backpack) {
        Some(p) => p,
        None => {
            let Some(p) = factory::make_item(w, &si("Backpack")) else {
                return false;
            };
            ops::set_movable(w, p, false);
            items::add_item(w, rm, p);
            p
        }
    };
    for _ in 0..2 {
        for spawning in [true, false] {
            let luck = muo_content::loot_pack::get_luck_chance_for_killer(w, id);
            muo_content::loot_pack::filthy_rich(w).generate(w, id, pack, spawning, luck);
        }
    }
    let ty = w.mobile(id).map(|m| m.ty).unwrap();
    let sound = (ty.get_death_sound)(w, id);
    effects::play_sound(w, id, sound);
    muo_server::effects::send_location_effect(
        w,
        mx::location(w, id),
        mx::map(w, id),
        0x3709,
        30,
        10,
        0x835,
        0,
    );
    if let Some(map) = mx::map(w, id) {
        muo_server::mobile::base::move_to_world(w, rm, mx::location(w, id), map);
    }
    muo_server::mobile::base::delete(w, id);
    false
}
pub fn khaldun_summoner_on_before_death(w: &mut World, id: MobileId) -> bool {
    khaldun_before_death(w, id, "Server.Mobiles.BoneMagi")
}
pub fn khaldun_zealot_on_before_death(w: &mut World, id: MobileId) -> bool {
    khaldun_before_death(w, id, "Server.Mobiles.BoneKnight")
}

// ---- AlterMeleeDamageTo ----

const DRAGONS: [&str; 8] = [
    "Server.Mobiles.Dragon",
    "Server.Mobiles.WhiteWyrm",
    "Server.Mobiles.SwampDragon",
    "Server.Mobiles.Drake",
    "Server.Mobiles.Nightmare",
    "Server.Mobiles.Hiryu",
    "Server.Mobiles.LesserHiryu",
    "Server.Mobiles.Daemon",
];

/// `to is Dragon or WhiteWyrm or SwampDragon or Drake or Nightmare or Hiryu or LesserHiryu or Daemon` triples it.
fn triple_against_dragons(w: &World, to: MobileId, damage: &mut i32) {
    if DRAGONS.iter().any(|d| factory::mobile_is(w, to, d)) {
        *damage *= 3;
    }
}
macro_rules! dragon_slayer {
    ($name:ident) => {
        pub fn $name(w: &mut World, _id: MobileId, to: MobileId, damage: &mut i32) {
            triple_against_dragons(w, to, damage);
        }
    };
}
dragon_slayer!(savage_alter_melee_damage_to);
dragon_slayer!(savage_rider_alter_melee_damage_to);
dragon_slayer!(chaos_dragoon_alter_melee_damage_to);
dragon_slayer!(chaos_dragoon_elite_alter_melee_damage_to);

// ---- OnDamage / melee ----

/// A 5% chance when a player does more than 5 damage: one of four taunts.
pub fn juka_lord_on_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    if !will_kill
        && amount > 5
        && let Some(f) = from.filter(|&f| mx::player(w, f))
        && rnd::below(w, 100) < 5
    {
        let name = (w.mobile(f).map(|m| m.ty).unwrap().name)(w, f).unwrap_or_default();
        let text = match rnd::below(w, 4) {
            0 => format!("{name}!!  You will have to do better than that!"),
            1 => format!("{name}!!  Prepare to meet your doom!"),
            2 => format!("{name}!!  My armies will crush you!"),
            _ => format!("{name}!!  You will pay for that!"),
        };
        messages::say(w, id, true, &text);
    }
    (BASE_CREATURE.on_damage)(w, id, amount, from, will_kill);
}

pub fn juka_warrior_on_gave_melee_attack(
    w: &mut World,
    id: MobileId,
    defender: MobileId,
    damage: i32,
) {
    (BASE_CREATURE_C.on_gave_melee_attack)(w, id, defender, damage);
    if rnd::double(w) < 0.80 {
        return;
    }
    match rnd::below(w, 3) {
        0 => {
            mx::msg(w, defender, 1004014);
            muo_server::mobile::base::freeze(w, defender, TimeSpan::from_secs(4));
        }
        1 => {
            mx::msg(w, defender, 1072221);
            muo_server::mobile::base::freeze(w, defender, TimeSpan::from_secs(3));
        }
        _ => {
            let amount = rnd::from(w, 10, 5);
            aos_helpers::damage(
                w,
                defender,
                Some(id),
                amount,
                DamageSplit {
                    phys: 100,
                    fire: 0,
                    cold: 0,
                    pois: 0,
                    nrgy: 0,
                },
                Default::default(),
            );
            mx::send_message(w, defender, "You have been hit by a critical strike!");
        }
    }
}

macro_rules! yomotsu_paralyze {
    ($name:ident) => {
        /// 10%: `FixedEffect(0x37B9, 10, 5)`, message 1070840 and a 4 second paralysis.
        pub fn $name(w: &mut World, id: MobileId, defender: MobileId, damage: i32) {
            (BASE_CREATURE_C.on_gave_melee_attack)(w, id, defender, damage);
            if rnd::double(w) < 0.1 {
                mx::fixed_effect(w, defender, 0x37B9, 10, 5);
                mx::msg(w, defender, 1070840);
                muo_server::mobile::base::paralyze(w, defender, TimeSpan::from_secs(4));
            }
        }
    };
}
yomotsu_paralyze!(yomotsu_elder_on_gave_melee_attack);
yomotsu_paralyze!(yomotsu_priest_on_gave_melee_attack);
yomotsu_paralyze!(yomotsu_warrior_on_gave_melee_attack);

/// `if (from != null && !willKill && amount > 3 && !InRange(from, 7))`: a thrown spear (`0xF51`) 1 second later.
pub fn meer_warrior_on_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    if let Some(f) = from
        && !will_kill
        && amount > 3
        && !mx::in_range(w, id, f, 7)
    {
        effects::moving_effect_plain(w, id, EntityId::Mobile(f), 0xF51, 10, 0, false, false);
        let dmg = rnd::min_max(w, 30, 40) - if w.core_aos() { 0 } else { 10 };
        weapon_hooks::spell_helper_damage(
            w,
            TimeSpan::from_secs(1),
            f,
            id,
            f64::from(dmg),
            [100, 0, 0, 0, 0],
        );
    }
    (BASE_CREATURE.on_damage)(w, id, amount, from, will_kill);
}

/// `if (!Core.SE && Utility.RandomDouble() < 0.2 && attacker is BaseCreature { Controlled: true, ControlMaster: not null })`
pub fn serpentine_dragon_on_got_melee_attack(
    w: &mut World,
    id: MobileId,
    attacker: MobileId,
    damage: i32,
) {
    (BASE_CREATURE_C.on_got_melee_attack)(w, id, attacker, damage);
    if !w.core_se() && rnd::double(w) < 0.2 {
        let master = cd(w, attacker)
            .filter(|c| c.controlled)
            .and_then(|c| c.master);
        if let Some(m) = master {
            control::issue_order(w, attacker, OrderType::Attack, None, Some(m));
        }
    }
}

// ---- aura ----

pub fn corporeal_brume_has_aura(w: &World, id: MobileId) -> bool {
    mx::combatant(w, id).is_some()
}
#[allow(dead_code)] // no caller reaches it yet
pub fn corporeal_brume_aura_base_damage(w: &mut World, _id: MobileId) -> i32 {
    rnd::min_max(w, 25, 35)
}
pub fn corporeal_brume_aura_effect(w: &mut World, _id: MobileId, m: MobileId) {
    effects::fixed_particles_hue(
        w,
        m,
        0x374A,
        10,
        15,
        5038,
        1181,
        2,
        muo_server::effects::EffectLayer::Head,
    );
    effects::play_sound(w, m, 0x213);
}

// ---- summons that thin themselves out ----

/// `Core.AOS || SummonMaster?.Player != true || (Map?.Rules & MapRules.HarmfulRestrictions) != 0`
fn summon_follows_acquire_rules(w: &World, id: MobileId) -> bool {
    if w.core_aos() {
        return true;
    }
    let master = control::summon_master(w, id);
    if !master.is_some_and(|m| mx::player(w, m)) {
        return true;
    }
    mx::map(w, id).and_then(|m| w.maps.get(m)).is_some_and(|m| {
        m.rules
            .contains(muo_server::map::MapRules::HARMFUL_RESTRICTIONS)
    })
}
pub fn blade_spirits_follows_acquire_rules(w: &World, id: MobileId) -> bool {
    summon_follows_acquire_rules(w, id)
}
pub fn energy_vortex_follows_acquire_rules(w: &World, id: MobileId) -> bool {
    summon_follows_acquire_rules(w, id)
}

/// `Math.Max(this.GetDistanceToSqrt(m), 1.0)`.
fn distance_sqrt(w: &World, a: MobileId, b: MobileId) -> f64 {
    let (pa, pb): (Point3D, Point3D) = (mx::location(w, a), mx::location(w, b));
    let (dx, dy) = (f64::from(pa.x - pb.x), f64::from(pa.y - pb.y));
    (dx * dx + dy * dy).sqrt().max(1.0)
}
/// `(m.Str + m.Skills.Tactics.Value) / Math.Max(GetDistanceToSqrt(m), 1.0)`
pub fn blade_spirits_get_fight_mode_ranking(
    w: &mut World,
    id: MobileId,
    m: MobileId,
    _t: FightMode,
    _p: bool,
) -> f64 {
    let s = muo_server::mobile::base::str(w, m);
    let t = muo_server::skills::value(w, m, muo_server::skills::SkillName::Tactics);
    (f64::from(s) + t) / distance_sqrt(w, id, m)
}
/// `(m.Int + m.Skills.Magery.Value) / Math.Max(GetDistanceToSqrt(m), 1.0)`
pub fn energy_vortex_get_fight_mode_ranking(
    w: &mut World,
    id: MobileId,
    m: MobileId,
    _t: FightMode,
    _p: bool,
) -> f64 {
    let s = muo_server::mobile::base::int(w, m);
    let t = muo_server::skills::value(w, m, muo_server::skills::SkillName::Magery);
    (f64::from(s) + t) / distance_sqrt(w, id, m)
}

/// `PooledRefList.Shuffle()`: `Random.Shuffle(span)`, forward Fisher-Yates.
fn shuffle(w: &mut World, v: &mut [MobileId]) {
    muo_primitives::random::Rng::shuffle(&mut w.rng, v);
}

/// With Core.SE a summon sees to it that no more than 6 EnergyVortex/BladeSpirits summons are in range 5.
fn summon_on_think(w: &mut World, id: MobileId) {
    if w.core_se() && summoned(w, id) {
        let mut list: Vec<MobileId> = mx::mobiles_in_range(w, id, 5)
            .into_iter()
            .filter(|&m| {
                (factory::mobile_is(w, m, "Server.Mobiles.EnergyVortex")
                    || factory::mobile_is(w, m, "Server.Mobiles.BladeSpirits"))
                    && summoned(w, m)
            })
            .collect();
        let mut amount = list.len() as i32 - 6;
        if amount > 0 {
            shuffle(w, &mut list);
            while amount > 0 {
                let target = list[amount as usize];
                amount -= 1;
                (BASE_CREATURE_C.dispel)(w, id, target);
            }
        }
    }
    (BASE_CREATURE_C.on_think)(w, id);
}
pub fn blade_spirits_on_think(w: &mut World, id: MobileId) {
    summon_on_think(w, id);
}
pub fn energy_vortex_on_think(w: &mut World, id: MobileId) {
    summon_on_think(w, id);
}

// ---- weapon abilities ----

/// `Utility.RandomBool() ? ParalyzingBlow : BleedAttack`
pub fn pyre_get_weapon_ability(
    w: &mut World,
    _id: MobileId,
) -> Option<muo_content::items::weapon::Ability> {
    use muo_content::items::weapon::Ability;
    Some(if rnd::bool(w) {
        Ability::ParalyzingBlow
    } else {
        Ability::BleedAttack
    })
}
pub fn rend_get_weapon_ability(
    w: &mut World,
    id: MobileId,
) -> Option<muo_content::items::weapon::Ability> {
    pyre_get_weapon_ability(w, id)
}

// ---- CuSidhe, RestlessSoul ----

/// A non-elf player master must wear PadsOfTheCuSidhe to ride it.
pub fn cu_sidhe_on_double_click(w: &mut World, id: MobileId, from: MobileId) {
    let elf = w
        .mobile(from)
        .is_some_and(|m| m.race.is_some() && m.race == super::ctors::race_named(w, "Elf"));
    let master = control::control_master(w, id);
    if !elf
        && master == Some(from)
        && mx::access_level(w, from) == muo_server::mobile::AccessLevel::Player
    {
        let pads = items::find_item_on_layer(w, from, muo_server::layer::Layer::Shoes);
        if pads.is_some_and(|p| factory::item_is(w, p, "Server.Items.PadsOfTheCuSidhe")) {
            mx::msg(w, from, 1071981);
        } else {
            mx::msg(w, from, 1072203);
            return;
        }
    }
    (BASE_CREATURE.on_double_click)(w, id, from);
}

/// `public override void DisplayPaperdollTo(Mobile to) { }`
pub fn restless_soul_display_paperdoll_to(_w: &mut World, _id: MobileId, _to: MobileId) {}

/// Removes the `PaperdollEntry` (6123) the base adds.
pub fn restless_soul_get_context_menu_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>,
) {
    (BASE_CREATURE.get_context_menu_entries)(w, id, from, list);
    let paperdoll = muo_server::context_menu_entry::ContextMenuEntry::new(6123, 18).number;
    list.retain(|e| e.number != paperdoll);
}

/// A player on Trammel in the Haven box (5199-5271, 1812-1865) with the Schmendrick objective in progress is no enemy.
pub fn restless_soul_is_enemy(w: &mut World, id: MobileId, m: MobileId) -> bool {
    use muo_server::map::MapId;
    if muo_content::player_mobile::data(w, m).is_some() {
        let p = mx::location(w, id);
        if mx::map(w, id) == Some(MapId::TRAMMEL)
            && (5199..=5271).contains(&p.x)
            && (1812..=1865).contains(&p.y)
            && muo_content::player_mobile::hooks::quest_haven_schmendrick_in_progress(w, m)
        {
            return false;
        }
    }
    (BASE_CREATURE_C.is_enemy)(w, id, m)
}
