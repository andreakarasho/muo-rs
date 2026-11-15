//! `FukiyaDarts`, `Shuriken` and `Fukiya`: the `INinjaAmmo` stacks and the fukiya's
//! state (`UsesRemaining`, `Poison`, `PoisonCharges`), plus the `SkillItemFns` for poisoning a throwing weapon.
//!
//! Also `NinjaWeapon` ( shooting, loading and unloading of the fukiya and the ninja belt),
//! `EggBomb`, `SmokeBomb` and `LeatherNinjaBelt`.

use muo_content::items::equip_hooks::CraftArgs;
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::layer::Layer;
use muo_server::poison::PoisonId;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;
use muo_server::type_registry::{Phase, TypeRegistry};

use super::write_version;

/// The three fields every ninja item here has.
#[derive(Clone, Copy, Debug, Default)]
pub struct NinjaData {
    pub uses_remaining: i32,
    pub poison: Option<PoisonId>,
    pub poison_charges: i32,
}

pub fn data(w: &World, id: ItemId) -> Option<&NinjaData> {
    w.item(id)?.ext.get::<NinjaData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut NinjaData> {
    w.item_mut(id)?.ext.get_mut::<NinjaData>()
}

/// `UsesRemaining = value` (`[InvalidateProperties]`).
pub fn set_uses_remaining(w: &mut World, id: ItemId, v: i32) {
    if let Some(d) = data_mut(w, id)
        && d.uses_remaining != v
    {
        d.uses_remaining = v;
        ops::invalidate_properties(w, id);
    }
}

fn new_ninja(w: &mut World, ty: &'static ItemType, item_id: i32, uses: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(NinjaData {
            uses_remaining: uses,
            poison: None,
            poison_charges: 0,
        });
    }
    id
}

/// `new FukiyaDarts(amount)`.
pub fn new_fukiya_darts(w: &mut World, amount: i32) -> ItemId {
    new_ninja(w, &FUKIYA_DARTS, 0x2806, amount)
}

/// `new Shuriken(amount)`.
pub fn new_shuriken(w: &mut World, amount: i32) -> ItemId {
    new_ninja(w, &SHURIKEN, 0x27AC, amount)
}

/// `new Fukiya()`.
pub fn new_fukiya(w: &mut World) -> ItemId {
    let id = new_ninja(w, &FUKIYA, 0x27AA, 0);
    (FUKIYA.set_layer)(w, id, Layer::OneHanded);
    id
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    let Some(d) = data(w, id) else { return };
    list.add_int(1060584, d.uses_remaining); // uses remaining: ~1_val~
    if let Some(p) = d.poison
        && d.poison_charges > 0
    {
        list.add_int(1062412 + muo_server::poison::level(w, p), d.poison_charges);
    }
}

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    write_version(out, false);
    let d = data(w, id).copied().unwrap_or_default();
    out.write_i32(d.uses_remaining);
    out.write_poison(d.poison);
    out.write_i32(d.poison_charges);
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let _version = r.read_i32()?;
    let d = NinjaData {
        uses_remaining: r.read_i32()?,
        poison: r.read_poison()?,
        poison_charges: r.read_i32()?,
    };
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    Ok(())
}

/// `OnCraft(..)` of the ammo: an exceptional craft doubles the uses.
fn on_craft(w: &mut World, id: ItemId, a: &CraftArgs) -> i32 {
    if a.quality == 2 {
        let u = data(w, id).map_or(0, |d| d.uses_remaining);
        set_uses_remaining(w, id, u.wrapping_mul(2));
    }
    a.quality
}

fn craftable(w: &mut World, id: ItemId, a: &CraftArgs) -> Option<i32> {
    w.item(id)
        .is_some_and(|i| i.ty.is(&FUKIYA_DARTS) || i.ty.is(&SHURIKEN))
        .then(|| on_craft(w, id, a))
}

macro_rules! ninja {
    ($($name:ident = $full:literal, $ctor:expr, weight $weight:expr;)*) => {$(
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: Some($ctor),
            default_weight: |_, _| $weight,
            get_properties,
            serialize,
            deserialize,
            ..ITEM_BASE
        };
    )*};
}

ninja! {
    FUKIYA_DARTS = "Server.Items.FukiyaDarts", |w| new_fukiya_darts(w, 1), weight 1.0;
    SHURIKEN = "Server.Items.Shuriken", |w| new_shuriken(w, 1), weight 1.0;
}

// ---- NinjaWeapon ----

const MAX_USES: i32 = 10;

/// What `INinjaWeapon` makes a class answer.
struct WeaponInfo {
    wrong_ammo: i32,
    no_free_hand: i32,
    empty_weapon: i32,
    recently_used: i32,
    full_weapon: i32,
    min_range: i32,
    max_range: i32,
    damage: (i32, i32),
    ammo_type: &'static ItemType,
    load_entry: i32,
    unload_entry: i32,
    /// `AttackAnimation(from, to)`.
    attack_animation: fn(&mut World, MobileId, MobileId),
}

fn human_body(w: &World, m: MobileId) -> bool {
    muo_content::creature::misc::body_is_human(w, m)
}

fn fukiya_animation(w: &mut World, from: MobileId, to: MobileId) {
    let mounted = muo_server::mobile::part1::props_b::mounted(w, from);
    if human_body(w, from) && !mounted {
        muo_server::mobile::base::animate(w, from, 33, 2, 1, true, true, 0);
    }
    muo_server::mobile::part2::effects::play_sound(w, from, 0x223);
    muo_server::mobile::part2::effects::moving_effect(
        w,
        from,
        EntityId::Mobile(to),
        0x2804,
        5,
        0,
        false,
        false,
        0,
        0,
    );
}

fn belt_animation(w: &mut World, from: MobileId, to: MobileId) {
    if human_body(w, from) {
        let mounted = muo_server::mobile::part1::props_b::mounted(w, from);
        muo_server::mobile::base::animate(
            w,
            from,
            if mounted { 26 } else { 9 },
            7,
            1,
            true,
            false,
            0,
        );
    }
    muo_server::mobile::part2::effects::play_sound(w, from, 0x23A);
    muo_server::mobile::part2::effects::moving_effect(
        w,
        from,
        EntityId::Mobile(to),
        0x27AC,
        1,
        0,
        false,
        false,
        0,
        0,
    );
}

static FUKIYA_INFO: WeaponInfo = WeaponInfo {
    wrong_ammo: 1063329,    // You can only load fukiya darts
    no_free_hand: 1063327,  // You must have a free hand to use a fukiya.
    empty_weapon: 1063325,  // You have no fukiya darts!
    recently_used: 1063326, // You are already using that fukiya.
    full_weapon: 1063330,   // You can only load fukiya darts
    min_range: 0,
    max_range: 6,
    damage: (4, 6),
    ammo_type: &FUKIYA_DARTS,
    load_entry: 6224,
    unload_entry: 6225,
    attack_animation: fukiya_animation,
};

static BELT_INFO: WeaponInfo = WeaponInfo {
    wrong_ammo: 1063301,    // You can only place shuriken in a ninja belt.
    no_free_hand: 1063299,  // You must have a free hand to throw shuriken.
    empty_weapon: 1063297,  // You have no shuriken in your ninja belt!
    recently_used: 1063298, // You cannot throw another shuriken yet.
    full_weapon: 1063302,   // You cannot add any more shuriken.
    min_range: 2,
    max_range: 10,
    damage: (3, 5),
    ammo_type: &SHURIKEN,
    load_entry: 6222,
    unload_entry: 6223,
    attack_animation: belt_animation,
};

/// `item is INinjaWeapon`.
fn weapon_info(w: &World, id: ItemId) -> Option<&'static WeaponInfo> {
    let ty = w.item(id)?.ty;
    if ty.is(&FUKIYA) {
        Some(&FUKIYA_INFO)
    } else if ty.is(&LEATHER_NINJA_BELT) {
        Some(&BELT_INFO)
    } else {
        None
    }
}

fn uses(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.uses_remaining)
}

fn poison_of(w: &World, id: ItemId) -> Option<PoisonId> {
    data(w, id).and_then(|d| d.poison)
}

fn charges_of(w: &World, id: ItemId) -> i32 {
    data(w, id).map_or(0, |d| d.poison_charges)
}

/// `Poison = value; PoisonCharges = charges` (both `[InvalidateProperties]`).
fn set_poison(w: &mut World, id: ItemId, poison: Option<PoisonId>, charges: i32) {
    if let Some(d) = data_mut(w, id) {
        d.poison = poison;
        d.poison_charges = charges;
    }
    ops::invalidate_properties(w, id);
}

fn set_charges(w: &mut World, id: ItemId, charges: i32) {
    if let Some(d) = data_mut(w, id) {
        d.poison_charges = charges;
    }
    ops::invalidate_properties(w, id);
}

/// `NinjaWeapon.WeaponIsValid(weapon, from)`: `!item.Deleted && item.RootParent == from`.
fn weapon_is_valid(w: &World, id: ItemId, from: MobileId) -> bool {
    w.item(id).is_some_and(|i| !i.deleted())
        && ops::root_parent(w, id) == Some(EntityId::Mobile(from))
}

/// `NinjaWeapon.AttemptShoot(from, weapon)`.
pub fn attempt_shoot(w: &mut World, from: MobileId, weapon: ItemId) {
    if !w.mobile(from).is_some_and(|m| m.player) {
        return; // only a PlayerMobile
    }
    let Some(info) = weapon_info(w, weapon) else {
        return;
    };
    if can_use_weapon(w, from, weapon) {
        begin_ninja_target(w, from, weapon, info.max_range);
    }
}

fn begin_ninja_target(w: &mut World, from: MobileId, weapon: ItemId, range: i32) {
    muo_server::target::begin_target(
        w,
        from,
        range,
        false,
        muo_server::target::TargetFlags::HARMFUL,
        move |w, from, t| on_target(w, from, t, weapon),
    );
}

fn can_use_weapon(w: &mut World, from: MobileId, weapon: ItemId) -> bool {
    let Some(info) = weapon_info(w, weapon) else {
        return false;
    };
    if weapon_is_valid(w, weapon, from) {
        if uses(w, weapon) > 0 {
            let cooldown =
                muo_content::player_mobile::data(w, from).is_some_and(|d| d.ninja_wep_cooldown);
            if !cooldown {
                if super::potions::has_free_hand(w, from) {
                    return true;
                }
                deps::send_localized_message(w, from, info.no_free_hand);
            } else {
                deps::send_localized_message(w, from, info.recently_used);
            }
        } else {
            deps::send_localized_message(w, from, info.empty_weapon);
        }
    }
    false
}

fn set_cooldown(w: &mut World, from: MobileId, v: bool) {
    if let Some(d) = muo_content::player_mobile::data_mut(w, from) {
        d.ninja_wep_cooldown = v;
    }
}

fn shoot(w: &mut World, from: MobileId, target: MobileId, weapon: ItemId) {
    let Some(info) = weapon_info(w, weapon) else {
        return;
    };
    if from == target
        || !can_use_weapon(w, from, weapon)
        || !muo_server::mobile::base::can_be_harmful(w, from, target)
    {
        return;
    }
    let tloc = w
        .mobile(target)
        .map_or_else(Default::default, |m| m.location);
    if info.min_range == 0 || !deps::mobile_in_range(w, from, tloc, info.min_range) {
        set_cooldown(w, from, true);
        let d =
            muo_server::mobile::part2::effects::get_direction_to(w, from, tloc.x, tloc.y, false);
        muo_server::mobile::part1::props_b::set_direction(w, from, d);
        muo_server::mobile::base::revealing_action(w, from);
        (info.attack_animation)(w, from, target);
        consume_use(w, weapon);
        if combat_check(w, from, target) {
            muo_server::timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
                on_hit(w, from, target, weapon)
            });
        }
        muo_server::timer::start(
            w,
            TimeSpan::from_millis(2500),
            TimeSpan::ZERO,
            1,
            move |w| set_cooldown(w, from, false),
        );
    } else {
        deps::send_localized_message(w, from, 1063303); // Your target is too close!
    }
}

/// `Unload(from, weapon)`.
fn unload(w: &mut World, from: MobileId, weapon: ItemId) {
    let Some(info) = weapon_info(w, weapon) else {
        return;
    };
    let n = uses(w, weapon);
    if n > 0 {
        let ammo = if std::ptr::eq(info.ammo_type, &FUKIYA_DARTS) {
            new_fukiya_darts(w, n)
        } else {
            new_shuriken(w, n)
        };
        let (p, c) = (poison_of(w, weapon), charges_of(w, weapon));
        if let Some(d) = data_mut(w, ammo) {
            d.poison = p;
            d.poison_charges = c;
        }
        muo_server::mobile::part2::items::add_to_backpack(w, from, ammo);
        set_uses_remaining(w, weapon, 0);
        set_poison(w, weapon, None, 0);
    }
}

fn level_of(w: &World, p: PoisonId) -> i32 {
    muo_server::poison::level(w, p)
}

/// `Reload(from, weapon, ammo)`.
fn reload(w: &mut World, from: MobileId, weapon: ItemId, ammo: ItemId) {
    let Some(info) = weapon_info(w, weapon) else {
        return;
    };
    if uses(w, weapon) >= MAX_USES {
        deps::send_localized_message(w, from, info.full_weapon);
        return;
    }
    let mut need = (MAX_USES - uses(w, weapon)).min(uses(w, ammo));
    if need <= 0 {
        return; // they targeted ammo with 0 uses: undefined behavior
    }
    let (wp, ap) = (poison_of(w, weapon), poison_of(w, ammo));
    if let Some(wp) = wp
        && ap.is_none_or(|ap| level_of(w, wp) > level_of(w, ap))
    {
        deps::send_localized_message(w, from, 1070767); // Loaded projectile is stronger, unload it first
        return;
    }
    if uses(w, weapon) > 0 {
        let differs = match (wp, ap) {
            (None, Some(_)) => true,
            (Some(a), Some(b)) => level_of(w, a) != level_of(w, b),
            _ => false,
        };
        if differs {
            unload(w, from, weapon);
            need = MAX_USES.min(uses(w, ammo));
        }
    }
    let poison_needed = (MAX_USES - charges_of(w, weapon)).min(charges_of(w, ammo));
    set_uses_remaining(w, weapon, uses(w, weapon) + need);
    let charges = charges_of(w, weapon) + poison_needed;
    set_charges(w, weapon, charges);
    if charges > 0 {
        set_poison(w, weapon, ap, charges);
    }
    let ammo_charges = charges_of(w, ammo) - poison_needed;
    set_charges(w, ammo, ammo_charges);
    set_uses_remaining(w, ammo, uses(w, ammo) - need);
    if uses(w, ammo) < 1 {
        base::delete(w, ammo);
    } else if charges_of(w, ammo) < 1 {
        set_poison(w, ammo, None, charges_of(w, ammo));
    }
}

/// `ConsumeUse(weapon)`.
fn consume_use(w: &mut World, weapon: ItemId) {
    let n = uses(w, weapon);
    if n > 0 {
        set_uses_remaining(w, weapon, n - 1);
        if n - 1 < 1 {
            set_poison(w, weapon, None, 0);
        }
    }
}

/// `CombatCheck(attacker, defender)`: modded from `BaseWeapon.CheckHit`, with Ninjitsu as the attack skill.
fn combat_check(w: &mut World, attacker: MobileId, defender: MobileId) -> bool {
    use muo_content::aos::AosAttribute;
    use muo_content::items::{weapon_ability as wa, weapon_hooks as hk};
    let def_weapon = hk::weapon_item(w, defender);
    let at_skill = muo_server::skills::value(w, attacker, SkillName::Ninjitsu);
    let mut def_skill = match def_weapon {
        Some(d) => {
            muo_content::items::weapon_combat::get_defend_skill_value(w, d, attacker, defender)
        }
        None => 0.0,
    };
    if def_skill <= -20.0 {
        def_skill = -19.9;
    }
    let mut attack_value = f64::from(muo_content::misc::aos_helpers::attribute_value(
        w,
        attacker,
        AosAttribute::AttackChance,
    ));
    attack_value += f64::from(hk::divine_fury_attack_bonus(w, attacker));
    if hk::animal_form_under(w, attacker, Some("Server.Mobiles.GreyWolf"))
        || hk::animal_form_under(w, attacker, Some("Server.Mobiles.BakeKitsune"))
    {
        attack_value += 20.0;
    }
    if wa::hit_lower_is_under_attack_effect(w, attacker) {
        attack_value -= 25.0;
    }
    if attack_value > 45.0 {
        attack_value = 45.0;
    }
    attack_value = (at_skill + 20.0) * (100.0 + attack_value);

    let mut defense_value = f64::from(muo_content::misc::aos_helpers::attribute_value(
        w,
        defender,
        AosAttribute::DefendChance,
    ));
    defense_value -= f64::from(hk::divine_fury_defend_malus(w, defender));
    if wa::hit_lower_is_under_defense_effect(w, defender) {
        defense_value -= 25.0;
    }
    if let Some(b) = wa::block_bonus(w, defender) {
        defense_value += f64::from(b);
    }
    if let Some(e) = hk::discordance_effect(w, attacker) {
        defense_value -= f64::from(e);
    }
    if defense_value > 45.0 {
        defense_value = 45.0;
    }
    defense_value = (def_skill + 20.0) * (100.0 + defense_value);

    let mut chance = attack_value / (defense_value * 2.0);
    if chance < 0.02 {
        chance = 0.02;
    }
    muo_server::mobile::part2::stats::check_skill_chance(w, attacker, SkillName::Ninjitsu, chance)
}

/// `OnHit(from, target, weapon)`.
fn on_hit(w: &mut World, from: MobileId, target: MobileId, weapon: ItemId) {
    let Some(info) = weapon_info(w, weapon) else {
        return;
    };
    if !muo_server::mobile::base::can_be_harmful(w, from, target) {
        return;
    }
    muo_server::mobile::base::do_harmful(w, from, target, false);
    let dmg = utility::random_min_max(&mut w.rng, info.damage.0, info.damage.1);
    muo_content::misc::aos_helpers::damage(
        w,
        target,
        Some(from),
        dmg,
        muo_content::misc::aos_helpers::DamageSplit {
            phys: 100,
            ..Default::default()
        },
        Default::default(),
    );
    if let Some(p) = poison_of(w, weapon)
        && charges_of(w, weapon) > 0
    {
        let evil = muo_content::hooks::hooks(w)
            .and_then(|h| h.evil_omen_end_effect)
            .is_some_and(|f| f(w, target));
        let poison = if evil {
            w.resources
                .get::<muo_server::poison::Poisons>()
                .and_then(|ps| ps.increase_level(Some(p)))
                .unwrap_or(p)
        } else {
            p
        };
        muo_server::mobile::base::apply_poison(w, target, Some(from), poison);
        let left = charges_of(w, weapon) - 1;
        if left < 1 {
            set_poison(w, weapon, None, left);
        } else {
            set_charges(w, weapon, left);
        }
    }
}

/// `OnTarget(from, targeted, weapon)`.
fn on_target(w: &mut World, from: MobileId, t: muo_server::target::Targeted, weapon: ItemId) {
    use muo_server::target::Targeted;
    if !w.mobile(from).is_some_and(|m| m.player) || !weapon_is_valid(w, weapon, from) {
        return;
    }
    let Some(info) = weapon_info(w, weapon) else {
        return;
    };
    match t {
        Targeted::Entity(EntityId::Mobile(m)) => shoot(w, from, m, weapon),
        Targeted::Entity(EntityId::Item(i))
            if w.item(i)
                .is_some_and(|x| std::ptr::eq(x.ty, info.ammo_type)) =>
        {
            reload(w, from, weapon, i);
        }
        _ => deps::send_localized_message(w, from, info.wrong_ammo),
    }
}

/// The two entries of the context menu of a ninja weapon in the pack (`LoadEntry`, `UnloadEntry`).
fn weapon_menu_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>,
) {
    use muo_server::context_menu_entry::ContextMenuEntry;
    let Some(info) = weapon_info(w, id) else {
        return;
    };
    if ops::is_child_of(w, id, Some(EntityId::Mobile(from))) {
        let mut load = ContextMenuEntry::new(info.load_entry, 0);
        load.on_click = Some(std::rc::Rc::new(|w, from, target| {
            if let EntityId::Item(weapon) = target
                && weapon_is_valid(w, weapon, from)
            {
                begin_ninja_target(w, from, weapon, 10);
            }
        }));
        list.push(load);
        let mut unl = ContextMenuEntry::new(info.unload_entry, 0);
        unl.enabled = uses(w, id) > 0;
        unl.on_click = Some(std::rc::Rc::new(|w, from, target| {
            if let EntityId::Item(weapon) = target
                && weapon_is_valid(w, weapon, from)
            {
                unload(w, from, weapon);
            }
        }));
        list.push(unl);
    }
}

/// `Server.Items.Fukiya`.
pub static FUKIYA: ItemType = ItemType {
    full_name: "Server.Items.Fukiya",
    parent: Some(&ITEM_BASE),
    construct: Some(new_fukiya),
    default_weight: crate::class_props::default_weight,
    get_properties,
    on_double_click: |w, id, from| attempt_shoot(w, from, id),
    get_context_menu_entries: |w, id, from, list| {
        base::get_context_menu_entries_base(w, id, from, list);
        weapon_menu_entries(w, id, from, list);
    },
    serialize,
    deserialize,
    ..ITEM_BASE
};

// ---- LeatherNinjaBelt ----

/// `new LeatherNinjaBelt()`: `base(0x2790)`, `Layer = Layer.Waist`.
pub fn new_leather_ninja_belt(w: &mut World) -> ItemId {
    let id = muo_content::items::clothing::new_clothing(w, &LEATHER_NINJA_BELT, 0x2790, 0);
    (LEATHER_NINJA_BELT.set_layer)(w, id, Layer::Waist);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(NinjaData::default());
    }
    id
}

/// `Server.Items.LeatherNinjaBelt` (a `BaseWaist` that is also an `INinjaWeapon`).
pub static LEATHER_NINJA_BELT: ItemType = ItemType {
    full_name: "Server.Items.LeatherNinjaBelt",
    parent: Some(&muo_content::items::clothing::BASE_WAIST),
    construct: Some(new_leather_ninja_belt),
    family: Some(&BELT_CLASS),
    default_weight: crate::class_props::default_weight,
    get_properties: |w, id, list| {
        (muo_content::items::clothing::BASE_WAIST.get_properties)(w, id, list);
        get_ninja_properties(w, id, list);
    },
    on_equip: |w, id, from| {
        if (muo_content::items::clothing::BASE_WAIST.on_equip)(w, id, from) {
            deps::send_localized_message(w, from, 1070785); // Double click this item each time you wish to throw a shuriken.
            return true;
        }
        false
    },
    on_double_click: |w, id, from| attempt_shoot(w, from, id),
    get_context_menu_entries: |w, id, from, list| {
        (muo_content::items::clothing::BASE_WAIST.get_context_menu_entries)(w, id, from, list);
        weapon_menu_entries(w, id, from, list);
    },
    serialize: |w, id, out| {
        (muo_content::items::clothing::BASE_WAIST.serialize)(w, id, out);
        write_version(out, false);
        let d = data(w, id).copied().unwrap_or_default();
        out.write_i32(d.uses_remaining);
        out.write_poison(d.poison);
        out.write_i32(d.poison_charges);
    },
    deserialize: |w, id, r| {
        (muo_content::items::clothing::BASE_WAIST.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let d = NinjaData {
            uses_remaining: r.read_i32()?,
            poison: r.read_poison()?,
            poison_charges: r.read_i32()?,
        };
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(d);
        }
        Ok(())
    },
    ..muo_content::items::clothing::BASE_WAIST
};

/// `DefaultResource => CraftResource.RegularLeather`.
static BELT_CLASS: muo_content::items::clothing::ClothingClass =
    muo_content::items::clothing::ClothingClass {
        default_resource: muo_content::craft::CraftResource::RegularLeather,
        ..muo_content::items::clothing::BASE_WAIST_CLASS
    };

fn get_ninja_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    let Some(d) = data(w, id) else { return };
    list.add_int(1060584, d.uses_remaining); // uses remaining: ~1_val~
    if let Some(p) = d.poison
        && d.poison_charges > 0
    {
        list.add_int(1062412 + muo_server::poison::level(w, p), d.poison_charges);
    }
}

// ---- EggBomb and SmokeBomb ----

/// The double click of both bombs.
fn bomb_double_click(w: &mut World, id: ItemId, from: MobileId) {
    use muo_content::skills::stealth::HidingSettings;
    let pack = w.mobile(from).and_then(|m| m.backpack);
    if !ops::is_child_of(w, id, pack.map(EntityId::Item)) {
        deps::send_localized_message(w, from, 1060640); // The item must be in your backpack to use it.
    } else if muo_server::skills::value(w, from, SkillName::Ninjitsu) < 50.0 {
        // You need at least ~1_SKILL_REQUIREMENT~ ~2_SKILL_NAME~ skill to use that ability.
        muo_server::mobile::part2::messages::send_localized_message(
            w,
            from,
            1063013,
            "50\tNinjitsu",
            0x3B2,
        );
    } else if w.tick_count - w.mobile(from).map_or(0, |m| m.next_skill_time) < 0 {
        deps::send_localized_message(w, from, 1070772); // You must wait a few seconds before you can use that item.
    } else if w.mobile(from).map_or(0, |m| m.mana) < 10 {
        deps::send_localized_message(w, from, 1049456); // You don't have enough mana to do that.
    } else {
        w.resources.get_mut::<HidingSettings>().combat_override = true;
        if muo_server::mobile::base::use_skill(w, from, SkillName::Hiding) {
            let mana = w.mobile(from).map_or(0, |m| m.mana);
            muo_server::mobile::part1::props_b::set_mana(w, from, mana - 10);
            muo_server::mobile::part2::effects::fixed_particles_hue(
                w,
                from,
                0x3709,
                1,
                30,
                9904,
                1108,
                6,
                muo_server::effects::EffectLayer::RightFoot,
            );
            muo_server::mobile::part2::effects::play_sound(w, from, 0x22F);
            base::consume(w, id, 1);
        }
        w.resources.get_mut::<HidingSettings>().combat_override = false;
    }
}

/// `new EggBomb()` / `new SmokeBomb()`: `base(0x2808)`, `Stackable = Core.ML`.
fn new_bomb(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.EggBomb");
    let ml = muo_content::core::ml(w);
    ops::set_stackable(w, id, ml);
    id
}

/// `new SmokeBomb()`.
pub fn new_smoke_bomb(w: &mut World) -> ItemId {
    new_bomb(w, &SMOKE_BOMB)
}

/// `new EggBomb()`.
pub fn new_egg_bomb(w: &mut World) -> ItemId {
    new_bomb(w, &EGG_BOMB)
}

/// `Server.Items.SmokeBomb`.
pub static SMOKE_BOMB: ItemType = ItemType {
    full_name: "Server.Items.SmokeBomb",
    parent: Some(&ITEM_BASE),
    construct: Some(new_smoke_bomb),
    default_weight: crate::class_props::default_weight,
    on_double_click: bomb_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `Server.Items.EggBomb`.
pub static EGG_BOMB: ItemType = ItemType {
    full_name: "Server.Items.EggBomb",
    construct: Some(new_egg_bomb),
    label_number: crate::class_props::label_number,
    ..SMOKE_BOMB
};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
static DARTS_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: INT,
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_fukiya_darts(w, v[0].i32())),
}];
static SHURIKEN_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: INT,
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_shuriken(w, v[0].i32())),
}];

fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.FukiyaDarts", &DARTS_CTOR);
    add_ctors(w, "Server.Items.Shuriken", &SHURIKEN_CTOR);
    muo_content::craft::hooks_mut(w).craftables.push(craftable);
    let f = &mut w.resources.get_mut::<muo_content::skills::SkillHooks>().fns;
    // `item is FukiyaDarts or Shuriken`: its `UsesRemaining`.
    f.poisonable_throwing = Some(|w, item| {
        w.item(item)
            .filter(|i| i.ty.is(&FUKIYA_DARTS) || i.ty.is(&SHURIKEN))
            .and_then(|_| data(w, item).map(|d| d.uses_remaining))
    });
    f.poison_throwing = Some(|w, item, poison, charges| {
        if let Some(d) = data_mut(w, item) {
            d.poison = Some(poison);
            d.poison_charges = charges;
        }
        ops::invalidate_properties(w, item);
    });
}

/// Registers the ninja ammo and the fukiya.
pub fn register(types: &mut TypeRegistry) {
    for t in [
        &FUKIYA_DARTS,
        &SHURIKEN,
        &FUKIYA,
        &LEATHER_NINJA_BELT,
        &SMOKE_BOMB,
        &EGG_BOMB,
    ] {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.FukiyaDarts.Configure",
        configure,
    );
}
