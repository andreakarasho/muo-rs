//! The `Set*` helpers of BaseCreature that leaf constructors call, the constructor (`new BaseCreature(ai, mode,
//! ...)`) and `PackItem`/`PackGold`/`AddLoot`.

use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::items;
use muo_server::mobile::{MobileType, ResistanceType};
use muo_server::skills::{self, SkillName};
use muo_server::world::World;

use super::data::{CreatureData, cd, cd_mut, read};
use super::enums::*;
use super::speeds;
use super::vtable::ctype;
use crate::hooks::rnd;
use crate::loot_pack::LootPack;
use crate::mx;

/// `new BaseCreature(ai, mode, iRangePerception, iRangeFight)` for class `ty`: makes the mobile, its
/// `CreatureData`, sets the speeds from the table, builds the AI and generates loot. Errors when the
/// speed table has no entry.
pub fn new_creature(
    w: &mut World,
    ty: &'static MobileType,
    ai: AIType,
    mode: FightMode,
    range_perception: i32,
    range_fight: i32,
) -> MobileId {
    // Members of the class (or a parent) may read template props: a world that skipped Configure loads them now.
    crate::templates::table(w);
    let id = w.new_mobile(ty);
    let mut d = CreatureData::new(w.tick_count);
    d.loyalty = MAX_LOYALTY;
    d.current_ai = ai;
    d.default_ai = ai;
    d.range_perception = range_perception;
    d.range_fight = range_fight;
    d.fight_mode = mode;
    if let Some(m) = w.mobile_mut(id) {
        m.ext = muo_server::Ext::new(d);
    }
    let (active, passive) = speeds::speeds_of(w, id);
    let (am, pm) = speeds::move_speeds_of(w, id);
    let reacquire = ctype(w, id).map_or(10_000, |c| {
        (c.reacquire_delay)(w, id).total_milliseconds() as i64
    });
    let tick = w.tick_count;
    if let Some(d) = cd_mut(w, id) {
        d.active_move_speed = am;
        d.passive_move_speed = pm;
        d.team = 0;
        d.debug = false;
        d.controlled = false;
        d.control_target = None;
        d.control_order = OrderType::None;
        d.tamable_raw = false;
        d.owners = Vec::new();
        d.next_reacquire_time = tick + reacquire;
    }
    if active <= 0.0 || passive <= 0.0 {
        // A 0-delay creature would spin its AI timer; only construction refuses. The world
        // survives with the fallback pace and the error is logged.
        tracing::error!(
            "{} constructed without speeds - is Data/npc-speeds.json missing?",
            super::type_name(w, id)
        );
    } else if let Some(d) = cd_mut(w, id) {
        d.active_speed = active;
        d.passive_speed = passive;
        d.current_speed = passive;
    }
    let current = read(w, id, ai, |d| d.current_ai);
    crate::ai::change_ai_type(w, id, current);

    if let Some(sp) = ctype(w, id).and_then(|c| (c.speech_type)(w, id)) {
        sp.on_construct(w, id);
    }
    let invulnerable = ctype(w, id).is_some_and(|c| (c.is_invulnerable)(w, id));
    if invulnerable
        && !props_a::core_aos(w)
        && let Some(m) = w.mobile_mut(id)
    {
        m.name_hue = 0x35;
    }
    generate_loot_spawning(w, id, true);
    id
}

// ---- stats ---------------------------------------------------------------------------------------------

/// SetDamage
pub fn set_damage(w: &mut World, id: MobileId, min: i32, max: i32) {
    if let Some(d) = cd_mut(w, id) {
        d.damage_min = min;
        d.damage_max = max;
    }
}

fn hits_max(w: &World, id: MobileId) -> i32 {
    w.mobile(id).map_or(0, |m| (m.ty.hits_max)(w, id))
}

/// SetHits(val)
pub fn set_hits(w: &mut World, id: MobileId, mut val: i32) {
    if val < 1000 && !props_a::core_aos(w) {
        val = val * 100 / 60;
    }
    if let Some(d) = cd_mut(w, id) {
        d.hits_max_seed = val;
    }
    let max = hits_max(w, id);
    props_b::set_hits(w, id, max);
}

/// SetHits(min, max)
pub fn set_hits_range(w: &mut World, id: MobileId, mut min: i32, mut max: i32) {
    if min < 1000 && !props_a::core_aos(w) {
        min = min * 100 / 60;
        max = max * 100 / 60;
    }
    let v = rnd::min_max(w, min, max);
    if let Some(d) = cd_mut(w, id) {
        d.hits_max_seed = v;
    }
    let m = hits_max(w, id);
    props_b::set_hits(w, id, m);
}

pub fn set_stam(w: &mut World, id: MobileId, val: i32) {
    if let Some(d) = cd_mut(w, id) {
        d.stam_max_seed = val;
    }
    let m = w.mobile(id).map_or(0, |m| (m.ty.stam_max)(w, id));
    props_b::set_stam(w, id, m);
}

pub fn set_stam_range(w: &mut World, id: MobileId, min: i32, max: i32) {
    let v = rnd::min_max(w, min, max);
    set_stam(w, id, v);
}

pub fn set_mana(w: &mut World, id: MobileId, val: i32) {
    if let Some(d) = cd_mut(w, id) {
        d.mana_max_seed = val;
    }
    let m = w.mobile(id).map_or(0, |m| (m.ty.mana_max)(w, id));
    props_b::set_mana(w, id, m);
}

pub fn set_mana_range(w: &mut World, id: MobileId, min: i32, max: i32) {
    let v = rnd::min_max(w, min, max);
    set_mana(w, id, v);
}

pub fn set_str(w: &mut World, id: MobileId, val: i32) {
    props_b::set_raw_str(w, id, val);
    let m = hits_max(w, id);
    props_b::set_hits(w, id, m);
}

pub fn set_str_range(w: &mut World, id: MobileId, min: i32, max: i32) {
    let v = rnd::min_max(w, min, max);
    set_str(w, id, v);
}

pub fn set_dex(w: &mut World, id: MobileId, val: i32) {
    props_b::set_raw_dex(w, id, val);
    let m = w.mobile(id).map_or(0, |m| (m.ty.stam_max)(w, id));
    props_b::set_stam(w, id, m);
}

pub fn set_dex_range(w: &mut World, id: MobileId, min: i32, max: i32) {
    let v = rnd::min_max(w, min, max);
    set_dex(w, id, v);
}

pub fn set_int(w: &mut World, id: MobileId, val: i32) {
    props_b::set_raw_int(w, id, val);
    let m = w.mobile(id).map_or(0, |m| (m.ty.mana_max)(w, id));
    props_b::set_mana(w, id, m);
}

pub fn set_int_range(w: &mut World, id: MobileId, min: i32, max: i32) {
    let v = rnd::min_max(w, min, max);
    set_int(w, id, v);
}

/// SetDamageType(type, val)
pub fn set_damage_type(w: &mut World, id: MobileId, ty: ResistanceType, val: i32) {
    if let Some(d) = cd_mut(w, id) {
        match ty {
            ResistanceType::Physical => d.physical_damage = val,
            ResistanceType::Fire => d.fire_damage = val,
            ResistanceType::Cold => d.cold_damage = val,
            ResistanceType::Poison => d.poison_damage = val,
            ResistanceType::Energy => d.energy_damage = val,
        }
    }
}

pub fn set_damage_type_range(w: &mut World, id: MobileId, ty: ResistanceType, min: i32, max: i32) {
    let v = rnd::min_max(w, min, max);
    set_damage_type(w, id, ty, v);
}

/// SetResistance(type, val)
pub fn set_resistance(w: &mut World, id: MobileId, ty: ResistanceType, val: i32) {
    if let Some(d) = cd_mut(w, id) {
        match ty {
            ResistanceType::Physical => d.physical_resistance_seed = val,
            ResistanceType::Fire => d.fire_resist_seed = val,
            ResistanceType::Cold => d.cold_resist_seed = val,
            ResistanceType::Poison => d.poison_resist_seed = val,
            ResistanceType::Energy => d.energy_resist_seed = val,
        }
    }
    super::control::update_resistances(w, id);
}

pub fn set_resistance_range(w: &mut World, id: MobileId, ty: ResistanceType, min: i32, max: i32) {
    let v = rnd::min_max(w, min, max);
    set_resistance(w, id, ty, v);
}

/// SetSkill(name, val)
pub fn set_skill(w: &mut World, id: MobileId, name: SkillName, val: f64) {
    set_skill_fixed(w, id, name, (val * 10.0) as i32);
}

/// SetSkill(name, min, max)
pub fn set_skill_range(w: &mut World, id: MobileId, name: SkillName, min: f64, max: f64) {
    let v = rnd::min_max(w, (min * 10.0) as i32, (max * 10.0) as i32);
    set_skill_fixed(w, id, name, v);
}

fn set_skill_fixed(w: &mut World, id: MobileId, name: SkillName, fixed: i32) {
    skills::set_base_fixed(w, id, name, fixed);
    if skills::base(w, id, name) > skills::cap(w, id, name) {
        if props_a::core_se(w) {
            let delta = skills::base_fixed(w, id, name) - skills::cap_fixed(w, id, name);
            let cap = props_a::skills_cap(w, id);
            props_a::set_skills_cap(w, id, cap + delta);
        }
        let b = skills::base(w, id, name);
        skills::set_cap(w, id, name, b);
    }
}

/// SetFameLevel
pub fn set_fame_level(w: &mut World, id: MobileId, level: i32) {
    let fame = match level {
        1 => rnd::min_max(w, 0, 1249),
        2 => rnd::min_max(w, 1250, 2499),
        3 => rnd::min_max(w, 2500, 4999),
        4 => rnd::min_max(w, 5000, 9999),
        5 => rnd::min_max(w, 10000, 10000),
        _ => w.mobile(id).map_or(0, |m| m.fame),
    };
    props_a::set_fame(w, id, fame);
}

/// SetKarmaLevel
pub fn set_karma_level(w: &mut World, id: MobileId, level: i32) {
    let karma = match level {
        0 => -rnd::min_max(w, 0, 624),
        1 => -rnd::min_max(w, 625, 1249),
        2 => -rnd::min_max(w, 1250, 2499),
        3 => -rnd::min_max(w, 2500, 4999),
        4 => -rnd::min_max(w, 5000, 9999),
        5 => -rnd::min_max(w, 10000, 10000),
        _ => w.mobile(id).map_or(0, |m| m.karma),
    };
    props_a::set_karma(w, id, karma);
}

// ---- loot ---------------------------------------------------------------------

/// GenerateLoot(bool spawning), through the class's override.
pub fn generate_loot_spawning(w: &mut World, id: MobileId, spawning: bool) {
    match ctype(w, id) {
        Some(c) => (c.generate_loot_spawning)(w, id, spawning),
        None => base_generate_loot_spawning(w, id, spawning),
    }
}

/// GenerateLoot(bool spawning), `base.GenerateLoot(spawning)`.
pub fn base_generate_loot_spawning(w: &mut World, id: MobileId, spawning: bool) {
    if let Some(d) = cd_mut(w, id) {
        d.spawning = spawning;
    }
    if !spawning {
        let luck = crate::loot_pack::get_luck_chance_for_killer(w, id);
        if let Some(d) = cd_mut(w, id) {
            d.killers_luck = luck;
        }
    }
    if let Some(c) = ctype(w, id) {
        (c.generate_loot)(w, id);
    }
    if read(w, id, false, |d| d.is_paragon) {
        let fame = w.mobile(id).map_or(0, |m| m.fame);
        let pack = if fame < 1250 {
            crate::loot_pack::meager(w)
        } else if fame < 2500 {
            crate::loot_pack::average(w)
        } else if fame < 5000 {
            crate::loot_pack::rich(w)
        } else if fame < 10000 {
            crate::loot_pack::filthy_rich(w)
        } else {
            crate::loot_pack::ultra_rich(w)
        };
        add_loot(w, id, pack);
    }
    if let Some(d) = cd_mut(w, id) {
        d.spawning = false;
        d.killers_luck = 0;
    }
}

/// AddLoot(pack, amount)
pub fn add_loot_n(w: &mut World, id: MobileId, pack: &'static LootPack, amount: i32) {
    for _ in 0..amount {
        add_loot(w, id, pack);
    }
}

/// AddLoot(pack)
pub fn add_loot(w: &mut World, id: MobileId, pack: &'static LootPack) {
    if read(w, id, false, |d| d.summoned) {
        return;
    }
    let Some(backpack) = ensure_backpack(w, id) else {
        return;
    };
    let (spawning, luck) = read(w, id, (false, 0), |d| (d.spawning, d.killers_luck));
    pack.generate(w, id, backpack, spawning, luck);
}

/// `Backpack ?? new Backpack { Movable = false }`, equipped with `AddItem`.
pub fn ensure_backpack(w: &mut World, id: MobileId) -> Option<ItemId> {
    if let Some(b) = props_b::backpack(w, id) {
        return Some(b);
    }
    let b = crate::factory::make_item(w, "Server.Items.Backpack")?;
    muo_server::item::ops::set_movable(w, b, false);
    items::add_item(w, id, b);
    Some(b)
}

/// PackGold(amount)
pub fn pack_gold(w: &mut World, id: MobileId, amount: i32) {
    if amount > 0
        && let Some(g) = crate::factory::make_item(w, "Server.Items.Gold")
    {
        muo_server::item::ops::set_amount(w, g, amount);
        pack_item(w, id, Some(g));
    }
}

/// PackGold(min, max)
pub fn pack_gold_range(w: &mut World, id: MobileId, min: i32, max: i32) {
    let v = rnd::min_max(w, min, max);
    pack_gold(w, id, v);
}

/// PackItem
pub fn pack_item(w: &mut World, id: MobileId, item: Option<ItemId>) {
    let Some(item) = item else { return };
    if read(w, id, false, |d| d.summoned) {
        muo_server::item::base::delete(w, item);
        return;
    }
    let Some(pack) = ensure_backpack(w, id) else {
        return;
    };
    let stackable = w.item(item).is_some_and(|i| i.stackable());
    // `if (!item.Stackable || !pack.TryDropItem(this, item, false)) pack.DropItem(item)`
    if !stackable || !muo_server::item::container::try_drop_item(w, pack, Some(id), item, false) {
        muo_server::item::container::drop_item_default(w, pack, item);
    }
}

#[allow(dead_code)]
fn _unused(w: &World, id: MobileId) -> bool {
    cd(w, id).is_some() && mx::alive(w, id)
}
