//! Animals with their own state or a few overrides: Cow, Sheep, WaterElemental, Dolphin, the beetles, the hiryus,
//! Kappa.

use muo_content::craft::CraftResource;
use muo_content::creature::data::cd_mut;
use muo_content::creature::mount::{BASE_MOUNT, BASE_MOUNT_C};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, cd, speeds};
use muo_content::factory;
use muo_content::mx;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{DateTime, MessageType, TimeSpan};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::DeathMoveResult;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::messages;

use super::ctors::{pack, si};
use super::pack_animal::{check_access, combine_backpacks, context_menu_entries};
use crate::common::rnd;

// ---- Cow ----

/// `_milkedOn` and `_milk`.
#[derive(Clone, Copy, Debug)]
pub struct CowData {
    pub milked_on: DateTime,
    pub milk: i32,
}

pub fn cow_init(w: &mut World, id: MobileId) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(CowData {
            milked_on: DateTime::MIN,
            milk: 0,
        });
    }
}

pub fn cow_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
    let d = w
        .mobile(id)
        .and_then(|m| m.ext.get::<CowData>())
        .copied()
        .unwrap_or(CowData {
            milked_on: DateTime::MIN,
            milk: 0,
        });
    out.write_date_time_ticks(d.milked_on.ticks());
    out.write_i32(d.milk);
}

pub fn cow_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    let milked_on = DateTime::from_ticks(r.read_date_time_ticks()?);
    let milk = r.read_i32()?;
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(CowData { milked_on, milk });
    }
    Ok(())
}

/// `if (Core.AOS && Utility.Random(1000) == 0) FightMode = FightMode.Closest;` (0.1% chance to have mad cows)
pub fn cow_ctor1(w: &mut World, id: MobileId) {
    if w.core_aos() && rnd::below(w, 1000) == 0 {
        super::ctors::set_fight_mode(w, id, muo_content::creature::enums::FightMode::Closest);
    }
}

/// `Tip()`.
fn cow_tip(w: &mut World, id: MobileId) {
    mx::play_sound(w, id, 121);
    mx::animate(w, id, 8, 0, 3, true, false, 0);
}

pub fn cow_on_double_click(w: &mut World, id: MobileId, from: MobileId) {
    (BASE_CREATURE.on_double_click)(w, id, from);
    let random = rnd::below(w, 100);
    if random < 5 {
        cow_tip(w, id);
    } else if random < 20 {
        mx::play_sound(w, id, 120);
    } else if random < 40 {
        mx::play_sound(w, id, 121);
    }
}

/// `Cow.TryMilk(from)`; None when the mobile is no cow. The messages fall through to the milking.
pub fn cow_try_milk(w: &mut World, cow: MobileId, from: MobileId) -> Option<bool> {
    w.mobile(cow)?.ext.get::<CowData>()?;
    if !mx::in_los(w, from, cow) || !mx::in_range_point(w, from, mx::location(w, cow), 2) {
        mx::msg(w, from, 1080400); // You can not milk the cow from this location.
    }
    let (controlled, master) = cd(w, cow).map_or((false, None), |d| (d.controlled, d.master));
    if controlled && master != Some(from) {
        mx::msg(w, from, 1071182); // The cow nimbly escapes your attempts to milk it.
    }
    let now = w.now;
    let d = *w.mobile(cow)?.ext.get::<CowData>()?;
    if d.milk == 0 && d.milked_on + TimeSpan::from_days(1) > now {
        mx::msg(w, from, 1080198); // This cow can not be milked now. Please wait for some time.
        return Some(false);
    }
    let mut milk = d.milk;
    if milk == 0 {
        milk = 4;
    }
    milk -= 1;
    if let Some(c) = w.mobile_mut(cow).and_then(|m| m.ext.get_mut::<CowData>()) {
        c.milked_on = now;
        c.milk = milk;
    }
    Some(true)
}

// ---- Sheep ----

/// `_nextWoolTime`.
#[derive(Clone, Copy, Debug)]
pub struct SheepData {
    pub next_wool: DateTime,
}

pub fn sheep_init(w: &mut World, id: MobileId) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(SheepData {
            next_wool: DateTime::MIN,
        });
    }
}

fn next_wool(w: &World, id: MobileId) -> DateTime {
    w.mobile(id)
        .and_then(|m| m.ext.get::<SheepData>())
        .map_or(DateTime::MIN, |d| d.next_wool)
}

/// `SheepBody()`.
pub fn sheep_body(w: &mut World, id: MobileId) {
    let b = if w.now >= next_wool(w, id) {
        0xCF
    } else {
        0xDF
    };
    crate::common::body(w, id, b);
}

pub fn sheep_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(1);
    out.write_anchored_time_ticks(next_wool(w, id).ticks());
}

pub fn sheep_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let version = r.read_i32()?;
    let ticks = if version == 0 {
        r.read_delta_time_ticks(w.now.ticks())?
    } else {
        r.read_anchored_time_ticks()?
    };
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(SheepData {
            next_wool: DateTime::from_ticks(ticks),
        });
    }
    sheep_body(w, id);
    Ok(())
}

/// `Body == 0xCF ? 3 : 0`
pub fn sheep_wool(w: &World, id: MobileId) -> i32 {
    if w.mobile(id).is_some_and(|m| m.cur_body().0 == 0xCF) {
        3
    } else {
        0
    }
}

pub fn sheep_on_think(w: &mut World, id: MobileId) {
    (BASE_CREATURE_C.on_think)(w, id);
    sheep_body(w, id);
}

/// `Sheep.Carve(from, item)`.
pub fn sheep_carve(w: &mut World, id: MobileId, from: MobileId, _item: ItemId) {
    if w.now < next_wool(w, id) {
        // This sheep is not yet ready to be shorn.
        let ns = w.mobile(from).and_then(|m| m.net_state);
        messages::private_overhead_message_number(
            w,
            id,
            MessageType::REGULAR,
            0x3B2,
            500449,
            "",
            ns,
        );
        return;
    }
    mx::msg(w, from, 500452); // You place the gathered wool into your backpack.
    let amount = if mx::map(w, id) == Some(muo_server::map::MapId::FELUCCA) {
        2
    } else {
        1
    };
    if let Some(wool) = factory::make_item_amount(w, &si("Wool"), amount) {
        muo_server::mobile::part2::items::add_to_backpack(w, from, wool);
    }
    let t = w.now + TimeSpan::from_hours(3);
    if let Some(d) = w.mobile_mut(id).and_then(|m| m.ext.get_mut::<SheepData>()) {
        d.next_wool = t;
    }
    sheep_body(w, id);
}

// ---- WaterElemental ----

/// `_hasDecanter`.
pub struct HasDecanter(pub bool);

pub fn water_elemental_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(1);
    out.write_bool(
        w.mobile(id)
            .and_then(|m| m.ext.get::<HasDecanter>())
            .is_some_and(|d| d.0),
    );
}

pub fn water_elemental_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    // Version 0 has no fields (`MigrateFrom` does nothing) and keeps the constructor's `true`.
    let has = if r.read_i32()? == 0 {
        true
    } else {
        r.read_bool()?
    };
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(HasDecanter(has));
    }
    Ok(())
}

// ---- Dolphin ----

fn dolphin_jump(w: &mut World, id: MobileId) {
    if rnd::bool(w) {
        mx::animate(w, id, 3, 16, 1, true, false, 0);
    } else {
        mx::animate(w, id, 4, 20, 1, true, false, 0);
    }
}

pub fn dolphin_on_double_click(w: &mut World, id: MobileId, from: MobileId) {
    if mx::access_level(w, from) >= muo_server::mobile::AccessLevel::GameMaster {
        dolphin_jump(w, id);
    }
}

pub fn dolphin_on_think(w: &mut World, id: MobileId) {
    if rnd::double(w) < 0.005 {
        dolphin_jump(w, id);
    }
    (BASE_CREATURE_C.on_think)(w, id);
}

// ---- the beetles ----

/// `if (!Controlled) CurrentSpeed = BoostedSpeed (0.1)`.
fn boost(w: &mut World, id: MobileId) {
    if !cd(w, id).is_some_and(|d| d.controlled) {
        speeds::set_current_speed(w, id, 0.1);
    }
}
/// `if (Combatant == null && !Controlled) CurrentSpeed = PassiveSpeed`.
fn unboost(w: &mut World, id: MobileId) {
    if mx::combatant(w, id).is_none() && !cd(w, id).is_some_and(|d| d.controlled) {
        let p = cd(w, id).map_or(0.0, |d| d.passive_speed);
        speeds::set_current_speed(w, id, p);
    }
}
pub fn beetle_on_harmful_spell(w: &mut World, id: MobileId, _from: MobileId) {
    boost(w, id);
}
pub fn beetle_on_combatant_change(w: &mut World, id: MobileId) {
    unboost(w, id);
}
pub fn fire_beetle_on_harmful_spell(w: &mut World, id: MobileId, _from: MobileId) {
    boost(w, id);
}
pub fn fire_beetle_on_combatant_change(w: &mut World, id: MobileId) {
    unboost(w, id);
}
pub fn fire_beetle_get_control_chance(
    _w: &mut World,
    _id: MobileId,
    _m: MobileId,
    _base: bool,
) -> f64 {
    1.0
}
/// `var pack = Backpack; pack?.Delete(); pack = new StrongBackpack(); pack.Movable = false; AddItem(pack);`
pub fn beetle_ctor1(w: &mut World, id: MobileId) {
    super::pack_animal::replace_backpack(w, id);
}

pub fn beetle_on_before_death(w: &mut World, id: MobileId) -> bool {
    if !(BASE_MOUNT.on_before_death)(w, id) {
        return false;
    }
    combine_backpacks(w, id);
    true
}
pub fn beetle_get_inventory_move_result_for(
    _w: &mut World,
    _id: MobileId,
    _item: ItemId,
) -> DeathMoveResult {
    DeathMoveResult::MoveToCorpse
}
pub fn beetle_is_snoop(w: &World, id: MobileId, from: MobileId) -> bool {
    !check_access(w, id, from) && (BASE_MOUNT.is_snoop)(w, id, from)
}
pub fn beetle_on_drag_drop(w: &mut World, id: MobileId, from: MobileId, item: ItemId) -> bool {
    if (BASE_MOUNT_C.check_feed)(w, id, from, item) {
        return true;
    }
    if check_access(w, id, from) {
        muo_server::mobile::part2::items::add_to_backpack(w, id, item);
        return true;
    }
    (BASE_MOUNT.on_drag_drop)(w, id, from, item)
}
pub fn beetle_check_nonlocal_drop(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    _item: ItemId,
    _target: ItemId,
) -> bool {
    check_access(w, id, from)
}
pub fn beetle_check_nonlocal_lift(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    _item: ItemId,
) -> bool {
    check_access(w, id, from)
}
pub fn beetle_get_context_menu_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    (BASE_MOUNT.get_context_menu_entries)(w, id, from, list);
    context_menu_entries(w, id, from, list);
}

/// `FireBeetle.AfterDeserialization`: `Hue = 0x489`; the class saves only its version.
pub fn fire_beetle_serialize(_w: &World, _id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
}
pub fn fire_beetle_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    crate::common::hue(w, id, 0x489);
    Ok(())
}

// ---- the hiryus ----

/// `Hiryu.GetHue()`.
fn hiryu_hue(w: &mut World) -> i32 {
    let r = rnd::below(w, 1075);
    let hue = match r {
        1074 => 0x85C,
        1073 => 0x490,
        1071.. => 0x030,
        1069.. => 0x037,
        1066.. => 0x295,
        1063.. => 0x123,
        1058.. => 0x482,
        1050.. => 0x487,
        1040.. => muo_content::craft::hue(w, CraftResource::Gold),
        1030.. => muo_content::craft::hue(w, CraftResource::Agapite),
        1020.. => 0x495,
        1010.. => 0x48D,
        1000.. => 0x47F,
        _ => 0,
    };
    hue | 0x8000
}
/// `LesserHiryu.GetHue()`.
fn lesser_hiryu_hue(w: &mut World) -> i32 {
    let r = rnd::below(w, 525);
    let hue = match r {
        524 => 0x258,
        523 => muo_content::craft::hue(w, CraftResource::Valorite),
        520.. => 0x7D4,
        510.. => 0x163,
        500.. => 0x295,
        _ => 0,
    };
    hue | 0x8000
}
pub fn hiryu_ctor1(w: &mut World, id: MobileId) {
    let h = hiryu_hue(w);
    crate::common::hue(w, id, h);
}
pub fn hiryu_ctor2(w: &mut World, id: MobileId) {
    super::seed::bonsai_third(w, id);
}
pub fn hiryu_ctor3(w: &mut World, id: MobileId) {
    super::seed::peculiar_third(w, id, 3);
}
pub fn lesser_hiryu_ctor1(w: &mut World, id: MobileId) {
    let h = lesser_hiryu_hue(w);
    crate::common::hue(w, id, h);
}
pub fn lesser_hiryu_ctor2(w: &mut World, id: MobileId) {
    super::seed::bonsai_third(w, id);
}
/// `ControlMaster.Skills.Bushido.Base >= 90.0`
pub fn lesser_hiryu_override_bonding_reqs(w: &World, id: MobileId) -> bool {
    muo_content::creature::control::control_master(w, id)
        .is_some_and(|m| base_ref(w, m, muo_server::skills::SkillName::Bushido) >= 90.0)
}
pub fn lesser_hiryu_get_control_chance(
    w: &mut World,
    id: MobileId,
    m: MobileId,
    use_base_skill: bool,
) -> f64 {
    use muo_server::skills::{self, SkillName};
    let taming = (BASE_MOUNT_C.get_control_chance)(w, id, m, use_base_skill);
    if taming < 0.05 {
        return taming;
    }
    let skill = if use_base_skill {
        skills::base(w, m, SkillName::Bushido)
    } else {
        skills::value(w, m, SkillName::Bushido)
    };
    if skill < 90.0 {
        return taming;
    }
    let mut bushido = (skill - 30.0) / 100.0;
    if skills::base(w, m, SkillName::Bushido) >= 120.0 {
        bushido += 0.05;
    }
    if bushido > taming { bushido } else { taming }
}

/// `Skills[name].Base` of a mobile that may have no skills loaded yet.
fn base_ref(w: &World, id: MobileId, skill: muo_server::skills::SkillName) -> f64 {
    let s = w
        .mobile(id)
        .and_then(|m| m.skills.as_ref())
        .and_then(|s| s.peek(skill as usize))
        .copied()
        .unwrap_or_default();
    f64::from(s.base) / 10.0
}

// ---- Kappa ----

/// `new Acid(TimeSpan.FromSeconds(10), 5, 10) { Name = "slime" }` (tick rate 1 second).
pub fn kappa_new_harmful_item(w: &mut World, _id: MobileId) -> Option<ItemId> {
    let acid = muo_items::misc::small::acid_new(
        w,
        &muo_items::misc::small::ACID,
        TimeSpan::from_secs_f64(10.0),
        TimeSpan::from_secs_f64(1.0),
        5,
        10,
    );
    muo_server::item::ops::set_name(w, acid, Some("slime".to_owned()));
    Some(acid)
}
pub fn kappa_ctor1(w: &mut World, id: MobileId) {
    crate::common::pack_new_amount(w, id, &si("RawFishSteak"), 3);
}
pub fn kappa_ctor2(w: &mut World, id: MobileId) {
    for _ in 0..2 {
        match rnd::below(w, 6) {
            0 => pack(w, id, "Gears"),
            1 => pack(w, id, "Hinge"),
            2 => pack(w, id, "Axle"),
            _ => {}
        }
    }
}
pub fn kappa_ctor3(w: &mut World, id: MobileId) {
    if w.core_ml() && rnd::below(w, 3) == 0 {
        super::seed::pack_peculiar(w, id, 4);
    }
}
/// The acid blood ability.
pub fn kappa_on_damage(
    w: &mut World,
    id: MobileId,
    amount: i32,
    from: Option<MobileId>,
    will_kill: bool,
) {
    if let Some(f) = from.filter(|&f| mx::map(w, f).is_some()) {
        let mut amt = 0;
        let mut target = id;
        let rand = rnd::from(w, 1, 100);
        if will_kill {
            amt = ((rand % 5) >> 2) + 3;
        }
        if mx::hits(w, id) < 100 && rand < 21 {
            target = if rand % 2 < 1 { id } else { f };
            amt += 1;
        }
        if amt > 0 {
            muo_content::creature::pets::spill_acid(w, id, Some(target), amt);
            mx::msg(w, f, 1070820); // The creature spills a pool of acidic slime!
            let mana = w.mobile(id).map_or(0, |m| m.mana);
            if mana > 14 {
                props_b::set_mana(w, id, mana - 15);
            }
        }
    }
    (BASE_CREATURE.on_damage)(w, id, amount, from, will_kill);
}

#[allow(dead_code)]
fn _unused(w: &mut World, id: MobileId) {
    let _ = cd_mut(w, id);
}

/// `_hasDecanter = true;`
pub fn water_elemental_ctor1(w: &mut World, id: MobileId) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(HasDecanter(true));
    }
}
