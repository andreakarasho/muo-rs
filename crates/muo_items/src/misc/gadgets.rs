//! `ArcaneGem`, `TribalPaint`, `OilCloth`, `OilFlask`, `FishOilFlask`, `Bola`, `ClockworkAssembly`, `DeceitBrazier`,
//! `UnholyBone`, `WarningItem`, `HintItem`, the wind chimes (`BaseWindChimes`, `WindChimes`, `FancyWindChimes`), the
//! gump toggle items and `SerpentPillar`.
//!
//! `OilCloth`'s `IDyable`/`IScissorable` are the free functions [`oil_cloth_dye`] / [`oil_cloth_scissor`] (the dye
//! tub and scissors targets reach classes through their own tables); `ClockworkAssembly` makes the `Golem` through
//! its constructor table.

use muo_primitives::dotnet::InvariantCase;
use std::collections::HashMap;

use muo_content::gumps::base::{Gump, RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::{armor, clothing, weapon};
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_io::{GenericReader, GenericWriter, TextDefinition};
use muo_primitives::{MessageType, Point3D, Rectangle2D, TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, messages, ops};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::skills::SkillName;
use muo_server::target::{self, TargetFlags, TargetId, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::{hue, read_version, write_version};
use muo_server::packets::message::MessageType as ItemMessage;

fn in_pack(w: &World, id: ItemId, from: MobileId) -> bool {
    let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    ops::is_child_of(w, id, pack)
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

fn msg(w: &mut World, to: MobileId, number: i32) {
    deps::send_localized_message(w, to, number);
}

fn text(w: &mut World, to: MobileId, s: &str) {
    deps::send_message(w, to, s);
}

fn local_overhead(w: &mut World, to: MobileId, hue: i32, number: i32) {
    muo_server::mobile::part2::messages::local_overhead_message_number(
        w,
        to,
        MessageType::REGULAR,
        hue,
        number,
        "",
    );
}

fn skill(w: &World, m: MobileId, s: SkillName) -> f64 {
    muo_server::skills::value_ref(w, m, s)
}

fn is_a(w: &World, id: ItemId, ty: &'static ItemType) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(ty))
}

/// `Consume()`.
fn consume(w: &mut World, id: ItemId) {
    muo_server::item::base::consume(w, id, 1);
}

/// Which `Target` an item started, so `from.Target is BolaTarget` / `(user.Target as ThrowTarget)?.Bomb` can be
/// asked (a target is a closure).
#[derive(Default)]
struct TargetTags(HashMap<TargetId, (&'static str, ItemId)>);

#[allow(clippy::too_many_arguments)] // one parameter per value the call site passes
fn begin_tagged_target(
    w: &mut World,
    from: MobileId,
    kind: &'static str,
    item: ItemId,
    range: i32,
    allow_ground: bool,
    flags: TargetFlags,
    on_target: impl Fn(&mut World, MobileId, Targeted) + 'static,
    on_finish: Option<fn(&mut World, MobileId, ItemId)>,
) {
    let mut t = target::Target::new(w, range, allow_ground, flags);
    t.handlers.on_target = Some(std::rc::Rc::new(on_target));
    t.handlers.on_target_finish = Some(std::rc::Rc::new(move |w: &mut World, m: MobileId| {
        if let Some(f) = on_finish {
            f(w, m, item);
        }
    }));
    let tid = target::add_target(w, t);
    w.resources
        .get_mut::<TargetTags>()
        .0
        .insert(tid, (kind, item));
    target::set_mobile_target(w, from, Some(tid));
}

/// The kind and item of the target `m` currently has.
fn current_tag(w: &World, m: MobileId) -> Option<(&'static str, ItemId)> {
    let tid = w.mobile(m)?.target?;
    w.resources.get::<TargetTags>()?.0.get(&tid).copied()
}

// ---- IArcaneEquip ----

/// `item as IArcaneEquip`: the robes, cloaks and thigh boots (clothing) and the two arcane gloves (armor).
#[derive(Copy, Clone)]
enum Arcane {
    Clothing,
    Gloves(i32),
}

fn arcane_of(w: &World, id: ItemId) -> Option<(Arcane, i32, i32)> {
    if let Some((cur, max)) = crate::clothing::special::arcane_equip(w, id) {
        return Some((Arcane::Clothing, cur, max));
    }
    let normal = if is_a(w, id, &crate::armor::special::LEATHER_GLOVES) {
        0x13C6
    } else if is_a(w, id, &crate::armor::special::LEAF_GLOVES) {
        0x2FC6
    } else {
        return None;
    };
    let (cur, max) = armor::data(w, id)?.arcane_charges;
    Some((Arcane::Gloves(normal), cur, max))
}

fn set_cur_arcane(w: &mut World, id: ItemId, kind: Arcane, value: i32) {
    match kind {
        Arcane::Clothing => crate::clothing::special::set_cur_arcane_charges(w, id, value),
        Arcane::Gloves(n) => crate::armor::special::set_cur_arcane_charges(w, id, value, n),
    }
}

fn set_max_arcane(w: &mut World, id: ItemId, kind: Arcane, value: i32) {
    match kind {
        Arcane::Clothing => crate::clothing::special::set_max_arcane_charges(w, id, value),
        Arcane::Gloves(n) => crate::armor::special::set_max_arcane_charges(w, id, value, n),
    }
}

// ---- ArcaneGem ----

/// `ArcaneGem.DefaultArcaneHue`.
pub const DEFAULT_ARCANE_HUE: i32 = 2117;

/// `ArcaneGem()`.
pub fn arcane_gem_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1EA7);
    let ml = deps::core_ml(w);
    ops::set_stackable(w, id, ml);
    id
}

/// `ArcaneGem.GetChargesFor(m)`.
pub fn arcane_gem_charges_for(w: &World, m: MobileId) -> i32 {
    ((skill(w, m, SkillName::Tailoring) / 5.0) as i32).clamp(16, 24)
}

const ARCANE_ONLY: &str =
    "You can only use this on exceptionally crafted robes, thigh boots, cloaks, or leather gloves.";

fn arcane_gem_target(w: &mut World, gem: ItemId, from: MobileId, obj: Targeted) {
    if !in_pack(w, gem, from) {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let item = match obj {
        Targeted::Entity(EntityId::Item(i)) => i,
        _ => {
            text(w, from, ARCANE_ONLY);
            return;
        }
    };
    let Some((kind, cur, max)) = arcane_of(w, item) else {
        text(w, from, ARCANE_ONLY);
        return;
    };
    let is_clothing = clothing::is_clothing(w, item);
    let is_armor = armor::is_armor(w, item);
    let is_weapon = weapon::is_weapon(w, item);
    let resource = clothing::data(w, item)
        .map(|d| d.resource)
        .or_else(|| armor::data(w, item).map(|d| d.resource))
        .or_else(|| weapon::data(w, item).map(|d| d.resource))
        .unwrap_or(muo_content::craft::CraftResource::None);

    if !in_pack(w, item, from) {
        msg(w, from, 1042001);
        return;
    }
    if w.item(item)
        .is_some_and(|i| i.loot_type == LootType::Blessed)
    {
        text(w, from, ARCANE_ONLY);
        return;
    }
    if resource != muo_content::craft::CraftResource::None
        && resource != muo_content::craft::CraftResource::RegularLeather
    {
        msg(w, from, 1049690); // Arcane gems can not be used on that type of leather.
        return;
    }

    let charges = arcane_gem_charges_for(w, from);
    let is_arcane = max > 0 && cur >= 0;
    let use_one = |w: &mut World| {
        if w.item(gem).is_some_and(|i| i.amount <= 1) {
            delete_item(w, gem);
        } else {
            let a = w.item(gem).map_or(1, |i| i.amount);
            ops::set_amount(w, gem, a - 1);
        }
    };
    if is_arcane {
        if cur >= max {
            text(w, from, "That item is already fully charged.");
        } else {
            if cur <= 0 {
                hue(w, item, DEFAULT_ARCANE_HUE);
            }
            if cur + charges > max {
                set_cur_arcane(w, item, kind, max);
            } else {
                set_cur_arcane(w, item, kind, cur + charges);
            }
            text(w, from, "You recharge the item.");
            use_one(w);
        }
    } else if skill(w, from, SkillName::Tailoring) >= 80.0 {
        use muo_content::items::armor::ArmorQuality;
        use muo_content::items::clothing::ClothingQuality;
        use muo_content::items::weapon::WeaponQuality;
        let exceptional = (is_clothing
            && clothing::data(w, item).is_some_and(|d| d.quality == ClothingQuality::Exceptional))
            || (is_armor
                && armor::data(w, item).is_some_and(|d| d.quality == ArmorQuality::Exceptional))
            || (is_weapon
                && weapon::data(w, item).is_some_and(|d| d.quality == WeaponQuality::Exceptional));
        if exceptional {
            let crafter = muo_server::mobile::part1::props_b::raw_name(w, from);
            if is_clothing {
                clothing::set_quality(w, item, ClothingQuality::Regular);
                clothing::set_crafter(w, item, crafter);
            } else if is_armor {
                armor::set_quality(w, item, ArmorQuality::Regular);
                armor::set_crafter(w, item, crafter);
                // Is there a method to remove bonuses?
                if let Some(d) = armor::data_mut(w, item) {
                    let changed = d.physical_bonus != 0
                        || d.fire_bonus != 0
                        || d.cold_bonus != 0
                        || d.poison_bonus != 0
                        || d.energy_bonus != 0;
                    d.physical_bonus = 0;
                    d.fire_bonus = 0;
                    d.cold_bonus = 0;
                    d.poison_bonus = 0;
                    d.energy_bonus = 0;
                    if changed {
                        ops::invalidate_properties(w, item);
                    }
                }
            } else {
                weapon::set_quality(w, item, WeaponQuality::Regular);
                if let Some(d) = weapon::data_mut(w, item) {
                    d.crafter = crafter;
                }
                ops::invalidate_properties(w, item);
            }
            // eq.CurArcaneCharges = eq.MaxArcaneCharges = charges;
            set_max_arcane(w, item, kind, charges);
            set_cur_arcane(w, item, kind, charges);
            hue(w, item, DEFAULT_ARCANE_HUE);
            text(w, from, "You enhance the item with your gem.");
            use_one(w);
        } else {
            text(
                w,
                from,
                "Only exceptional items can be enhanced with the gem.",
            );
        }
    } else {
        text(
            w,
            from,
            "You do not have enough skill in tailoring to enhance the item.",
        );
    }
}

/// `ArcaneGem.ConsumeCharges(from, amount)`.
pub fn arcane_gem_consume_charges(w: &mut World, from: MobileId, mut amount: i32) -> bool {
    let items = w.mobile(from).map(|m| m.items.clone()).unwrap_or_default();
    let mut avail = 0;
    for &i in &items {
        if let Some((_, cur, max)) = arcane_of(w, i)
            && max > 0
            && cur >= 0
        {
            avail += cur;
        }
    }
    if avail < amount {
        return false;
    }
    for &i in &items {
        if let Some((kind, cur, max)) = arcane_of(w, i)
            && max > 0
            && cur >= 0
        {
            if cur > amount {
                set_cur_arcane(w, i, kind, cur - amount);
                break;
            }
            amount -= cur;
            set_cur_arcane(w, i, kind, 0);
        }
    }
    true
}

item_leaf! {
    /// `Server.Items.ArcaneGem`.
    ARCANE_GEM = "Server.Items.ArcaneGem" : ITEM_BASE, |w| arcane_gem_new(w, &ARCANE_GEM), encoded false;
    default_weight: |_, _| 1.0,
    default_name: |_, _| Some("arcane gem".to_string()),
    on_double_click: |w, id, from| {
        if !in_pack(w, id, from) {
            msg(w, from, 1042001); // That must be in your pack for you to use it.
        } else {
            target::begin_target(w, from, 2, false, TargetFlags::NONE, move |w, from, t| {
                arcane_gem_target(w, id, from, t)
            });
            text(w, from, "What do you wish to use the gem on?");
        }
    },
}

// ---- TribalPaint ----

/// `TribalPaint()`.
pub fn tribal_paint_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x9EC);
    hue(w, id, 2101);
    let ml = deps::core_ml(w);
    ops::set_stackable(w, id, ml);
    id
}

item_leaf! {
    /// `Server.Items.TribalPaint`.
    TRIBAL_PAINT = "Server.Items.TribalPaint" : ITEM_BASE, |w| tribal_paint_new(w, &TRIBAL_PAINT), encoded false;
    default_weight: |_, _| 2.0,
    label_number: |_, _| 1040000, // savage kin paint
    on_double_click: |w, id, from| {
        use muo_content::spells::helper as sh;
        if !in_pack(w, id, from) {
            msg(w, from, 1042001); // That must be in your pack for you to use it.
            return;
        }
        let sigil = muo_content::spells::hooks::hooks(w)
            .and_then(|h| h.sigil_exists_on)
            .is_some_and(|f| f(w, from));
        if sigil {
            msg(w, from, 1010465); // You cannot disguise yourself while holding a sigil.
        } else if !muo_server::mobile::part1::combat::can_begin_action(w, from, "Server.Spells.Fifth.IncognitoSpell") {
            msg(w, from, 501698); // You cannot disguise yourself while incognitoed.
        } else if !muo_server::mobile::part1::combat::can_begin_action(w, from, "Server.Spells.Seventh.PolymorphSpell")
            || sh::under_transformation(w, from)
        {
            msg(w, from, 501699); // You cannot disguise yourself while polymorphed.
        } else if muo_content::player_mobile::hooks::animal_form_under(w, from) {
            msg(w, from, 1061634); // You cannot disguise yourself while in that form.
        } else if muo_server::mobile::part1::props_b::is_body_mod(w, from)
            || deps::find_item_on_layer(w, from, muo_server::layer::Layer::Helm)
                .is_some_and(|h| w.item(h).is_some_and(|i| i.ty.full_name == "Server.Items.OrcishKinMask"))
        {
            msg(w, from, 501605); // You are already disguised.
        } else {
            let female = w.mobile(from).is_some_and(|m| m.female);
            muo_server::mobile::part1::props_b::set_body_mod(
                w,
                from,
                muo_primitives::Body::new(if female { 184 } else { 183 }),
            );
            muo_server::mobile::part1::props_b::set_hue_mod(w, from, 0);
            if w.mobile(from).is_some_and(|m| m.player) {
                muo_content::player_mobile::set_savage_paint_expiration(w, from, TimeSpan::from_days(7));
            }
            // You now bear the markings of the savage tribe.  Your body paint will last about a week or you can remove it with an oil cloth.
            msg(w, from, 1042537);
            consume(w, id);
        }
    },
}

// ---- OilCloth ----

/// `OilCloth()`.
pub fn oil_cloth_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x175D);
    hue(w, id, 2001);
    id
}

/// `OilCloth.Dye(from, sender)` (`IDyable`).
pub fn oil_cloth_dye(w: &mut World, id: ItemId, dyed_hue: i32) -> bool {
    if ops::is_deleted(w, id) {
        return false;
    }
    hue(w, id, dyed_hue);
    true
}

/// `OilCloth.Scissor(from, scissors)` (`IScissorable`).
pub fn oil_cloth_scissor(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if ops::is_deleted(w, id) || !deps::can_see(w, from, id) {
        return false;
    }
    if let Some(bandage) = muo_content::factory::make_item(w, "Server.Items.Bandage")
        && let Some(ty) = w.item(id).map(|i| i.ty)
    {
        (ty.scissor_helper)(w, id, from, bandage, 1, true);
    }
    true
}

fn oil_cloth_target(w: &mut World, cloth: ItemId, from: MobileId, obj: Targeted) {
    // TODO: Need details on how oil cloths should get consumed here
    if !in_pack(w, cloth, from) {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let ent = match obj {
        Targeted::Entity(e) => Some(e),
        _ => None,
    };
    if let Some(EntityId::Item(item)) = ent
        && ops::root_parent(w, item) != Some(EntityId::Mobile(from))
    {
        msg(w, from, 1005425); // You may only wipe down items you are holding or carrying.
    } else if let Some(EntityId::Item(item)) =
        ent.filter(|e| matches!(e, EntityId::Item(i) if weapon::is_weapon(w, *i)))
    {
        let (poison, charges) =
            weapon::data(w, item).map_or((None, 0), |d| (d.poison, d.poison_charges));
        if poison.is_none() || charges <= 0 {
            local_overhead(w, from, 0x3B2, 1005422); // Hmmmm... this does not need to be cleaned.
        } else {
            weapon::set_poison_charges(w, item, if charges < 2 { 0 } else { charges - 2 });
            if weapon::data(w, item).is_some_and(|d| d.poison_charges > 0) {
                msg(w, from, 1005423); // You have removed some of the caustic substance, but not all.
            } else {
                msg(w, from, 1010497); // You have cleaned the item.
            }
        }
    } else if ent == Some(EntityId::Mobile(from)) && w.mobile(from).is_some_and(|m| m.player) {
        let body_mod = w.mobile(from).map_or(0, |m| m.body_mod.body_id());
        if body_mod == 183 || body_mod == 184 {
            muo_content::player_mobile::set_savage_paint_expiration(w, from, TimeSpan::ZERO);
            muo_server::mobile::part1::props_b::set_body_mod(w, from, muo_primitives::Body::new(0));
            muo_server::mobile::part1::props_b::set_hue_mod(w, from, -1);
            msg(w, from, 1040006); // You wipe away all of your body paint.
            consume(w, cloth);
        } else {
            local_overhead(w, from, 0x3B2, 1005422); // Hmmmm... this does not need to be cleaned.
        }
    } else if let Some(EntityId::Item(item)) =
        ent.filter(|e| matches!(e, EntityId::Item(i) if crate::food::beverage::is_beverage(w, *i)))
    {
        if let Some(bomb) = beverage_to_firebomb(w, item) {
            muo_server::item::base::replace_with(w, item, bomb);
            msg(w, from, 1060580); // You prepare a firebomb.
            consume(w, cloth);
        }
    } else if let Some(EntityId::Item(item)) =
        ent.filter(|e| matches!(e, EntityId::Item(i) if is_a(w, *i, &FIREBOMB)))
    {
        let _ = item;
        msg(w, from, 1060579); // That is already a firebomb!
    } else {
        msg(w, from, 1005426); // The cloth will not work on that.
    }
}

/// `beverage.Content == BeverageType.Liquor` then `new Firebomb(beverage.ItemID) { Name = beverage.Name }`.
fn beverage_to_firebomb(w: &mut World, beverage: ItemId) -> Option<ItemId> {
    if crate::food::beverage::content(w, beverage) != crate::food::beverage::BeverageType::Liquor {
        return None;
    }
    let graphic = w.item(beverage)?.item_id;
    let name = ops::name(w, beverage);
    let bomb = firebomb_new(w, &FIREBOMB, graphic);
    ops::set_name(w, bomb, name);
    Some(bomb)
}

item_leaf! {
    /// `Server.Items.OilCloth`.
    OIL_CLOTH = "Server.Items.OilCloth" : ITEM_BASE, |w| oil_cloth_new(w, &OIL_CLOTH), encoded false;
    label_number: |_, _| 1041498, // oil cloth
    default_weight: |_, _| 1.0,
    on_double_click: |w, id, from| {
        if in_pack(w, id, from) {
            target::begin_target(w, from, -1, false, TargetFlags::NONE, move |w, from, t| {
                oil_cloth_target(w, id, from, t)
            });
            msg(w, from, 1005424); // Select the weapon or armor you wish to use the cloth on.
        } else {
            msg(w, from, 1042001); // That must be in your pack for you to use it.
        }
    },
}

// ---- OilFlask, FishOilFlask ----

/// `OilFlask(amount)` of class `ty`.
pub fn oil_flask_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1C18);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

/// `FilledMessageNumber` and `FillMultiplier`.
fn flask_fill(w: &World, id: ItemId) -> (i32, f64) {
    if is_a(w, id, &FISH_OIL_FLASK) {
        (1150865, 2.0)
    } else {
        (1150864, 1.0)
    }
}

fn oil_flask_target(w: &mut World, flask: ItemId, from: MobileId, targeted: Targeted) {
    use crate::lights::base as light;
    let Targeted::Entity(EntityId::Item(lantern)) = targeted else {
        msg(w, from, 502218); // You cannot use that.
        return;
    };
    if !is_a(w, lantern, &crate::lights::LANTERN) {
        msg(w, from, 502218);
        return;
    }
    let movable = w.item(lantern).is_some_and(|i| i.movable());
    let player = w
        .mobile(from)
        .is_some_and(|m| m.access_level == AccessLevel::Player);
    if !movable || (light::protected(w, lantern) && player) {
        msg(w, from, 500685); // You can't use that, it belongs to someone else.
        return;
    }
    let (message, multiplier) = flask_fill(w, flask);
    let was_burning = light::burning(w, lantern);
    light::douse(w, lantern);
    light::set_duration(
        w,
        lantern,
        TimeSpan::from_secs_f64(crate::lights::LANTERN_FULL_DURATION.total_seconds() * multiplier),
    );
    light::set_burnt_out(w, lantern, false);
    if was_burning {
        light::ignite(w, lantern);
    }
    msg(w, from, message);
    let empty = super::small::empty_oil_flask_new(w, &super::small::EMPTY_OIL_FLASK, 1);
    if !muo_server::mobile::part2::items::place_in_backpack(w, from, empty) {
        let mut did_stack = false;
        for i in muo_server::mobile::part2::enumerators::get_items_at(w, from) {
            if muo_server::item::base::stack_with(w, i, Some(from), flask, false) {
                did_stack = true;
                break;
            }
        }
        if !did_stack {
            let (loc, map) = w
                .item(flask)
                .map_or((Point3D::default(), None), |i| (i.location, i.map));
            ops::move_to_world_map(w, empty, loc, map);
        }
    }
    consume(w, flask);
}

fn oil_flask_double_click(w: &mut World, id: ItemId, from: MobileId) {
    msg(w, from, 501947); // Select lantern to refuel.
    target::begin_target(w, from, 2, true, TargetFlags::NONE, move |w, from, t| {
        oil_flask_target(w, id, from, t)
    });
}

item_leaf! {
    /// `Server.Items.OilFlask` `[SerializationGenerator(0)]`.
    OIL_FLASK = "Server.Items.OilFlask" : ITEM_BASE, |w| oil_flask_new(w, &OIL_FLASK, 1), encoded true;
    label_number: |_, _| 1027199, // oil flask
    on_double_click: oil_flask_double_click,
}

item_leaf! {
    /// `Server.Items.FishOilFlask` `[SerializationGenerator(0)]`.
    FISH_OIL_FLASK = "Server.Items.FishOilFlask" : OIL_FLASK, |w| oil_flask_new(w, &FISH_OIL_FLASK, 1), encoded true;
    label_number: |_, _| 1150863, // fish oil flask
}

// ---- Bola ----

/// `Bola(amount)`.
pub fn bola_new(w: &mut World, ty: &'static ItemType, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x26AC);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

fn bola_has_free_hands(w: &mut World, from: MobileId) -> bool {
    use muo_server::layer::Layer;
    let mut one = deps::find_item_on_layer(w, from, Layer::OneHanded);
    let mut two = deps::find_item_on_layer(w, from, Layer::TwoHanded);
    let movable =
        |w: &World, i: Option<ItemId>| i.is_some_and(|i| w.item(i).is_some_and(|x| x.movable()));
    if muo_content::core::se(w) {
        if let Some(pack) = w.mobile(from).and_then(|m| m.backpack) {
            for slot in [&mut one, &mut two] {
                if movable(w, *slot)
                    && let Some(i) = slot.take()
                    && let Some(pty) = w.item(pack).map(|p| p.ty)
                {
                    (pty.drop_item)(w, pack, i);
                }
            }
        }
    } else if muo_content::core::aos(w) {
        for slot in [&mut one, &mut two] {
            if movable(w, *slot)
                && let Some(i) = slot.take()
            {
                deps::add_to_backpack(w, from, i);
            }
        }
    }
    one.is_none() && two.is_none()
}

fn bola_finish_throw(w: &mut World, from: MobileId, to: MobileId) {
    use muo_content::factory::mobile_is;
    if muo_content::core::aos(w) {
        let (loc, map) = w
            .mobile(to)
            .map_or((Point3D::default(), None), |m| (m.location, m.map));
        let b = bola_new(w, &BOLA, 1);
        ops::move_to_world_map(w, b, loc, map);
    }
    let chaos = mobile_is(w, to, "Server.Mobiles.ChaosDragoon")
        || mobile_is(w, to, "Server.Mobiles.ChaosDragoonElite");
    if chaos {
        msg(w, from, 1042047); // You fail to knock the rider from its mount.
    }
    if !chaos {
        // to.Mount?.Rider = null (a BaseMount or an EtherealMount)
        muo_content::creature::mount::dismount(w, to);
    }
    if w.mobile(to).is_some_and(|m| m.player) {
        if muo_content::player_mobile::hooks::animal_form_under(w, to) {
            let name = w
                .mobile(from)
                .and_then(|m| (m.ty.name)(w, from))
                .unwrap_or_default();
            // ~1_NAME~ knocked you out of animal form!
            muo_server::mobile::part2::messages::send_localized_message(
                w, to, 1114066, &name, 0x3B2,
            );
        } else if muo_server::mobile::part1::props_b::mounted(w, to) {
            msg(w, to, 1040023); // You have been knocked off of your mount!
        }
        let secs = if deps::core_ml(w) { 10 } else { 3 };
        muo_content::player_mobile::members::set_mount_block(
            w,
            to,
            muo_content::player_mobile::BlockMountType::Dazed,
            Some(TimeSpan::from_secs(secs)),
            true,
        );
    }
    // only failsafe, attacker should already be dismounted
    if muo_content::core::aos(w) && w.mobile(from).is_some_and(|m| m.player) {
        let secs = if deps::core_ml(w) { 10 } else { 3 };
        muo_content::player_mobile::members::set_mount_block(
            w,
            from,
            muo_content::player_mobile::BlockMountType::BolaRecovery,
            Some(TimeSpan::from_secs(secs)),
            true,
        );
    }
    muo_content::mx::damage(w, to, 1, None);
    timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
        muo_server::mobile::part1::combat::end_action(w, from, "Bola");
    });
}

fn bola_target(w: &mut World, bola: ItemId, from: MobileId, obj: Targeted) {
    use muo_content::{mx, player_mobile::hooks as ph};
    if ops::is_deleted(w, bola) {
        return;
    }
    let Targeted::Entity(EntityId::Mobile(to)) = obj else {
        msg(w, from, 1049629); // You cannot throw a bola at that.
        return;
    };
    if !in_pack(w, bola, from) {
        msg(w, from, 1040019); // The bola must be in your pack to use it.
    } else if !bola_has_free_hands(w, from) {
        msg(w, from, 1040015); // Your hands must be free to use this
    } else if muo_server::mobile::part1::props_b::mounted(w, from) {
        msg(w, from, 1040016); // You cannot use this while riding a mount
    } else if ph::animal_form_under(w, from) {
        msg(w, from, 1070902); // You can't use this while in an animal form!
    } else if !muo_server::mobile::part1::props_b::mounted(w, to) && !ph::animal_form_under(w, to) {
        msg(w, from, 1049628); // You have no reason to throw a bola at that.
    } else if !mx::can_be_harmful(w, from, to) {
    } else if muo_server::mobile::part1::combat::begin_action(w, from, "Bola") {
        // EtherealMount.StopMounting(from)
        if let Some(f) = muo_content::hooks::hooks(w).and_then(|h| h.ethereal_stop_mounting) {
            f(w, from);
        }
        mx::do_harmful(w, from, to);
        consume(w, bola);
        let dir = mx::get_direction_to(w, from, to);
        mx::set_direction(w, from, dir);
        mx::animate(w, from, 11, 5, 1, true, false, 0);
        muo_server::mobile::part2::effects::moving_effect_plain(
            w,
            from,
            EntityId::Mobile(to),
            0x26AC,
            10,
            0,
            false,
            false,
        );
        timer::start(w, TimeSpan::from_millis(500), TimeSpan::ZERO, 1, move |w| {
            bola_finish_throw(w, from, to)
        });
    } else {
        msg(w, from, 1049624); // You have to wait a few moments before you can use another bola!
    }
}

fn bola_double_click(w: &mut World, id: ItemId, from: MobileId) {
    use muo_content::player_mobile::hooks as ph;
    if !in_pack(w, id, from) {
        msg(w, from, 1040019); // The bola must be in your pack to use it.
    } else if !muo_server::mobile::part1::combat::can_begin_action(w, from, "Bola") {
        msg(w, from, 1049624); // You have to wait a few moments before you can use another bola!
    } else if current_tag(w, from).is_some_and(|(k, _)| k == "BolaTarget") {
        msg(w, from, 1049631); // This bola is already being used.
    } else if !bola_has_free_hands(w, from) {
        msg(w, from, 1040015); // Your hands must be free to use this
    } else if muo_server::mobile::part1::props_b::mounted(w, from) {
        msg(w, from, 1040016); // You cannot use this while riding a mount
    } else if ph::animal_form_under(w, from) {
        msg(w, from, 1070902); // You can't use this while in an animal form!
    } else {
        // EtherealMount.StopMounting(from)
        if let Some(f) = muo_content::hooks::hooks(w).and_then(|h| h.ethereal_stop_mounting) {
            f(w, from);
        }
        begin_tagged_target(
            w,
            from,
            "BolaTarget",
            id,
            8,
            false,
            TargetFlags::HARMFUL,
            move |w, from, t| bola_target(w, id, from, t),
            None,
        );
        // * You begin to swing the bola...*
        muo_server::mobile::part2::messages::local_overhead_message_number(
            w,
            from,
            MessageType::EMOTE,
            0x3B2,
            1049632,
            "",
        );
        let name = w
            .mobile(from)
            .and_then(|m| (m.ty.name)(w, from))
            .unwrap_or_default();
        // ~1_NAME~ begins to menacingly swing a bola...
        muo_server::mobile::part2::messages::nonlocal_overhead_message_number(
            w,
            from,
            MessageType::EMOTE,
            0x3B2,
            1049633,
            &name,
        );
    }
}

item_leaf! {
    /// `Server.Items.Bola`.
    BOLA = "Server.Items.Bola" : ITEM_BASE, |w| bola_new(w, &BOLA, 1), encoded false;
    default_weight: |_, _| 4.0,
    on_double_click: bola_double_click,
}

// ---- ClockworkAssembly ----

/// `ClockworkAssembly()`.
pub fn clockwork_assembly_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x1EA8);
    hue(w, id, 1102);
    id
}

/// Builds `class` with the constructor of `args.len()` parameters its table lists (`Activator.CreateInstance`).
pub(crate) fn create_with_args_pub(w: &mut World, class: &str, args: &[Value]) -> Option<EntityId> {
    let ctors = muo_content::props::ctors_of(w, class)?;
    let c = ctors.iter().find(|c| c.params.len() == args.len())?;
    Some((c.make)(w, args))
}

fn clockwork_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !in_pack(w, id, from) {
        msg(w, from, 1071944); // The clockwork assembly must be in your backpack to construct a golem.
        return;
    }
    let tinker = skill(w, from, SkillName::Tinkering);
    if tinker < 60.0 {
        msg(w, from, 1071943); // You must be a Journeyman or higher Tinker to construct a golem.
        return;
    }
    let (followers, max) = w
        .mobile(from)
        .map_or((0, 0), |m| (m.followers, m.followers_max));
    if followers + 4 > max {
        msg(w, from, 1049607); // You have too many followers to control that creature.
        return;
    }
    let scalar = if tinker >= 100.0 {
        1.0
    } else if tinker >= 90.0 {
        0.9
    } else if tinker >= 80.0 {
        0.8
    } else if tinker >= 70.0 {
        0.7
    } else {
        0.6
    };
    let Some(pack) = w.mobile(from).and_then(|m| m.backpack) else {
        return;
    };
    let names = [
        "Server.Items.PowerCrystal",
        "Server.Items.Gears",
        "Server.Items.BronzeIngot",
        "Server.Items.IronIngot",
    ];
    let clilocs = [1071945, 1071946, 1071947, 1071948];
    let amounts = [1, 5, 50, 50];
    let mut types: Vec<&'static ItemType> = Vec::new();
    for n in names {
        match w.types.find_type_by_full_name(n, false) {
            Some(muo_server::type_registry::EntityType::Item(t)) => types.push(t),
            _ => return, // a part class that is not registered cannot be counted
        }
    }
    let res =
        muo_server::item::container::consume_total_types(w, pack, &types, &amounts, true, None);
    if res >= 0 {
        msg(w, from, clilocs[res as usize]);
        return;
    }
    let Some(EntityId::Mobile(g)) = create_with_args_pub(
        w,
        "Server.Mobiles.Golem",
        &[Value::Bool(true), Value::Float(scalar)],
    ) else {
        return; // no Golem constructor registered: nothing to build
    };
    if muo_content::creature::control::set_control_master(w, g, Some(from)) {
        delete_item(w, id);
        if let Some((loc, Some(map))) = w.mobile(from).map(|m| (m.location, m.map)) {
            muo_server::mobile::base::move_to_world(w, g, loc, map);
        }
        muo_content::mx::play_sound(w, from, 0x241);
    }
}

item_leaf! {
    /// `Server.Items.ClockworkAssembly`.
    CLOCKWORK_ASSEMBLY = "Server.Items.ClockworkAssembly" : ITEM_BASE, |w| clockwork_assembly_new(w, &CLOCKWORK_ASSEMBLY), encoded false;
    default_weight: |_, _| 5.0,
    label_number: |_, _| 1073426, // Clockwork Assembly
    on_double_click: clockwork_double_click,
}

// ---- Firebomb ----

/// What `Firebomb` keeps in memory (nothing is serialized).
#[derive(Clone, Debug, Default)]
struct BombData {
    lit_by: Option<MobileId>,
    thrown_from: Point3D,
    ticks: i32,
    pub(crate) timer: Option<TimerHandle>,
    users: Vec<MobileId>,
}

fn bomb(w: &World, id: ItemId) -> BombData {
    w.item(id)
        .and_then(|i| i.ext.get::<BombData>())
        .cloned()
        .unwrap_or_default()
}

fn bomb_mut(w: &mut World, id: ItemId) -> Option<&mut BombData> {
    let i = w.item_mut(id)?;
    if i.ext.get::<BombData>().is_none() {
        i.ext = Ext::new(BombData::default());
    }
    i.ext.get_mut::<BombData>()
}

/// `Firebomb(itemID)`.
pub fn firebomb_new(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    hue(w, id, 1260);
    id
}

fn bomb_overhead(w: &mut World, id: ItemId, held_by: Option<MobileId>, ticks: i32) {
    let s = ticks.to_string();
    let root = ops::root_parent(w, id);
    if let Some(m) = held_by {
        muo_server::mobile::part2::messages::public_overhead_message(
            w,
            m,
            MessageType::REGULAR,
            957,
            false,
            &s,
            true,
            AccessLevel::Player,
        );
    } else if root.is_none() {
        messages::public_overhead_message(w, id, ItemMessage::REGULAR, 957, false, &s);
    } else if let Some(EntityId::Mobile(m)) = root {
        muo_server::mobile::part2::messages::public_overhead_message(
            w,
            m,
            MessageType::REGULAR,
            957,
            false,
            &s,
            true,
            AccessLevel::Player,
        );
    }
}

fn firebomb_tick(w: &mut World, id: ItemId) {
    use muo_content::misc::aos_helpers::{DamageOptions, DamageSplit, damage};
    let Some(item) = w.item(id) else {
        return;
    };
    if item.deleted() {
        if let Some(mut t) = bomb(w, id).timer {
            t.cancel(w);
        }
        return;
    }
    let held_by = item.held_by;
    let map = item.map;
    if map == Some(muo_server::map::MapId::INTERNAL) && held_by.is_none() {
        return;
    }
    let ticks = bomb(w, id).ticks;
    if ticks <= 2 {
        let t = ticks + 1;
        if let Some(b) = bomb_mut(w, id) {
            b.ticks = t;
        }
        bomb_overhead(w, id, held_by, t);
        return;
    }
    if let Some(h) = held_by {
        muo_server::mobile::part2::items::drop_holding(w, h);
    }
    let b = bomb(w, id);
    if !b.users.is_empty() {
        let mut queue = Vec::new();
        for &u in &b.users {
            if current_tag(w, u).is_some_and(|(k, bi)| k == "ThrowTarget" && bi == id) {
                queue.push(u);
            }
        }
        if let Some(d) = bomb_mut(w, id) {
            d.users.clear();
        }
        for u in queue {
            target::cancel_mobile_target(w, u);
        }
    }
    let fire = DamageSplit {
        fire: 100,
        ..Default::default()
    };
    match ops::root_parent(w, id) {
        Some(EntityId::Mobile(parent)) => {
            msg(w, parent, 1060583); // The firebomb explodes in your hand!
            let amount = utility::random(&mut w.rng, 3) + 4;
            damage(w, parent, None, amount, fire, DamageOptions::default());
        }
        None => {
            let loc = ops::get_world_location(w, id);
            let lit_by = b.lit_by;
            let mut targets = Vec::new();
            let near: Vec<MobileId> =
                muo_server::map::enumerate::mobiles_in_range(w, map, loc.x, loc.y, 1).collect();
            for m in near {
                let ok = match lit_by {
                    None => true,
                    Some(l) => {
                        muo_content::spells::helper::valid_indirect_target(w, l, m)
                            && muo_server::mobile::base::can_be_harmful_msg(w, l, m, false)
                    }
                };
                if ok {
                    targets.push(m);
                }
            }
            for victim in targets {
                if let Some(l) = lit_by {
                    muo_server::mobile::base::do_harmful(w, l, victim, false);
                }
                let amount = utility::random(&mut w.rng, 3) + 4;
                damage(w, victim, lit_by, amount, fire, DamageOptions::default());
            }
            let from = b.thrown_from;
            let east_to_west = muo_content::spells::helper::get_east_to_west(from, loc);
            muo_server::effects::play_sound(w, from, map, 0x20C);
            let graphic = if east_to_west { 0x398C } else { 0x3996 };
            let make = muo_content::items::common::hooks(w).and_then(|h| h.fire_field_item);
            for i in -2..=2 {
                let at = if east_to_west {
                    Point3D::new(from.x + i, from.y, from.z)
                } else {
                    Point3D::new(from.x, from.y + i, from.z)
                };
                if let Some(make) = make {
                    make(w, graphic, at, lit_by, map, TimeSpan::from_secs(9), i);
                }
            }
        }
        Some(EntityId::Item(_)) => {}
    }
    if let Some(mut t) = bomb(w, id).timer {
        t.cancel(w);
    }
    delete_item(w, id);
}

fn firebomb_target(w: &mut World, id: ItemId, from: MobileId, obj: Targeted) {
    use muo_content::spells::helper::{Point3dRef, get_surface_top};
    let (deleted, map) = w.item(id).map_or((true, None), |i| (i.deleted(), i.map));
    if deleted || map == Some(muo_server::map::MapId::INTERNAL) || !in_pack(w, id, from) {
        return;
    }
    let p: Point3dRef = obj.into();
    if matches!(
        obj,
        Targeted::Entity(EntityId::Mobile(_))
            | Targeted::Entity(EntityId::Item(_))
            | Targeted::Land(_)
            | Targeted::Static(_)
    ) {
        let p = get_surface_top(w, p);
        let thrown = match p {
            Point3dRef::Point(pt) => pt,
            Point3dRef::Entity(EntityId::Item(i)) => {
                w.item(i).map_or_else(Point3D::default, |x| x.location)
            }
            Point3dRef::Entity(EntityId::Mobile(m)) => {
                w.mobile(m).map_or_else(Point3D::default, |x| x.location)
            }
        };
        if let Some(b) = bomb_mut(w, id) {
            b.thrown_from = thrown;
        }
        muo_server::mobile::base::revealing_action(w, from);
        let (graphic, bhue) = w.item(id).map_or((0, 0), |i| (i.item_id, i.hue));
        match p {
            Point3dRef::Entity(e) => muo_server::effects::send_moving_effect(
                w, from, e, graphic, 7, 0, false, false, bhue, 0,
            ),
            Point3dRef::Point(pt) => muo_server::effects::send_moving_effect_to_point(
                w, from, pt, graphic, 7, 0, false, false, bhue, 0,
            ),
        }
        timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            if ops::is_deleted(w, id) {
                return;
            }
            let to = bomb(w, id).thrown_from;
            ops::move_to_world_map(w, id, to, map);
        });
        ops::internalize(w, id);
    }
}

fn firebomb_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !in_pack(w, id, from) {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
        return;
    }
    let casting = w
        .mobile(from)
        .and_then(|m| m.spell)
        .is_some_and(|s| muo_content::spells::spell::is_casting(w, s));
    if muo_content::core::aos(w)
        && (muo_server::mobile::base::paralyzed(w, from)
            || w.mobile(from).is_some_and(|m| m.frozen)
            || casting)
    {
        msg(w, from, 1075857); // You cannot use that while paralyzed.
        return;
    }
    let running = bomb(w, id).timer.is_some_and(|t| t.running(w));
    if running {
        msg(w, from, 1060581); // You've already lit it!  Better throw it now!
    } else {
        let t = timer::start(
            w,
            TimeSpan::from_secs(1),
            TimeSpan::from_secs(1),
            0,
            move |w| firebomb_tick(w, id),
        );
        if let Some(b) = bomb_mut(w, id) {
            b.timer = Some(t);
            b.lit_by = Some(from);
        }
        msg(w, from, 1060582); // You light the firebomb.  Throw it now!
    }
    if let Some(b) = bomb_mut(w, id)
        && !b.users.contains(&from)
    {
        b.users.push(from);
    }
    begin_tagged_target(
        w,
        from,
        "ThrowTarget",
        id,
        12,
        true,
        TargetFlags::NONE,
        move |w, from, t| firebomb_target(w, id, from, t),
        Some(|w, from, id| {
            if let Some(b) = bomb_mut(w, id) {
                b.users.retain(|u| *u != from);
            }
        }),
    );
}

item_leaf! {
    /// `Server.Items.Firebomb`.
    FIREBOMB = "Server.Items.Firebomb" : ITEM_BASE, |w| firebomb_new(w, &FIREBOMB, 0x99B), encoded false;
    default_weight: |_, _| 2.0,
    on_double_click: firebomb_double_click,
}

// ---- UnholyBone ----

const BONE_SPAWN: [&str; 12] = [
    "Server.Mobiles.Skeleton",
    "Server.Mobiles.Zombie",
    "Server.Mobiles.Wraith",
    "Server.Mobiles.Spectre",
    "Server.Mobiles.Ghoul",
    "Server.Mobiles.Mummy",
    "Server.Mobiles.Bogle",
    "Server.Mobiles.RottingCorpse",
    "Server.Mobiles.BoneKnight",
    "Server.Mobiles.SkeletalKnight",
    "Server.Mobiles.Lich",
    "Server.Mobiles.LichLord",
];

#[derive(Clone, Debug, Default)]
pub(crate) struct BoneData {
    pub(crate) timer: Option<TimerHandle>,
}

/// `new SpawnTimer(item).Start()`.
fn bone_start_timer(w: &mut World, id: ItemId) {
    let delay = super::random_min_max(w, 5, 10);
    let t = timer::start(
        w,
        TimeSpan::from_secs(i64::from(delay)),
        TimeSpan::ZERO,
        1,
        move |w| {
            if w.item(id).is_none_or(|i| i.deleted()) {
                return;
            }
            let pick = utility::random(&mut w.rng, 12) as usize;
            let spawn = muo_content::factory::make_mobile(w, BONE_SPAWN[pick])
                .or_else(|| muo_content::factory::make_mobile(w, BONE_SPAWN[0]));
            if let Some(m) = spawn
                && let Some((loc, Some(map))) = w.item(id).map(|i| (i.location, i.map))
            {
                muo_server::mobile::base::move_to_world(w, m, loc, map);
            }
            delete_item(w, id);
        },
    );
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BoneData { timer: Some(t) });
    }
}

/// `UnholyBone()`.
pub fn unholy_bone_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xF7E);
    ops::set_movable(w, id, false);
    hue(w, id, 0x497);
    bone_start_timer(w, id);
    id
}

/// `UnholyBone.Carve(from, item)`.
pub fn unholy_bone_carve(w: &mut World, id: ItemId, from: MobileId, _blade: ItemId) -> bool {
    if !is_a(w, id, &UNHOLY_BONE) {
        return false;
    }
    let loc = ops::get_world_location(w, id);
    let map = w.item(id).and_then(|i| i.map);
    muo_server::effects::play_sound(w, loc, map, 0x48F);
    muo_server::effects::send_location_effect(w, loc, map, 0x3728, 10, 10, 0, 0);
    if utility::random_double(&mut w.rng) < 0.3 {
        if w.item(id).is_some_and(|i| i.item_id == 0xF7E) {
            text(w, from, "You destroy the bone.");
        } else {
            text(w, from, "You destroy the bone pile.");
        }
        let amount = super::random_min_max(w, 25, 100);
        let gold = muo_content::items::gold::new_gold(w, amount);
        ops::move_to_world_map(w, gold, loc, map);
        delete_item(w, id);
        if let Some(mut t) = w
            .item(id)
            .and_then(|i| i.ext.get::<BoneData>())
            .and_then(|d| d.timer)
        {
            t.cancel(w);
        }
    } else if w.item(id).is_some_and(|i| i.item_id == 0xF7E) {
        text(w, from, "You damage the bone.");
    } else {
        text(w, from, "You damage the bone pile.");
    }
    true
}

item_leaf_raw! {
    /// `Server.Items.UnholyBone`.
    UNHOLY_BONE = "Server.Items.UnholyBone" : ITEM_BASE;
    construct: Some(|w| unholy_bone_new(w, &UNHOLY_BONE)),
    default_name: |_, _| Some("unholy bone".to_string()),
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        crate::misc::write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = crate::misc::read_version(r, false)?;
        bone_start_timer(w, id); // [AfterDeserialization]
        Ok(())
    },
}

// ---- DeceitBrazier ----

const BRAZIER_CREATURES: &[&str] = &[
    "FireSteed",
    "Skeleton",
    "SkeletalKnight",
    "SkeletalMage",
    "Mummy",
    "BoneKnight",
    "Lich",
    "LichLord",
    "BoneMagi",
    "Wraith",
    "Shade",
    "Spectre",
    "Zombie",
    "RottingCorpse",
    "Ghoul",
    "Balron",
    "Daemon",
    "Imp",
    "GreaterMongbat",
    "Mongbat",
    "IceFiend",
    "Gargoyle",
    "StoneGargoyle",
    "FireGargoyle",
    "HordeMinion",
    "Gazer",
    "ElderGazer",
    "GazerLarva",
    "Harpy",
    "StoneHarpy",
    "HeadlessOne",
    "HellHound",
    "HellCat",
    "Phoenix",
    "LavaLizard",
    "SandVortex",
    "ShadowWisp",
    "SwampTentacle",
    "PredatorHellCat",
    "Wisp",
    "GiantSpider",
    "DreadSpider",
    "FrostSpider",
    "Scorpion",
    "ArcticOgreLord",
    "Cyclops",
    "Ettin",
    "EvilMage",
    "FrostTroll",
    "Ogre",
    "OgreLord",
    "Orc",
    "OrcishLord",
    "OrcishMage",
    "OrcBrute",
    "Ratman",
    "RatmanMage",
    "OrcCaptain",
    "Troll",
    "Titan",
    "EvilMageLord",
    "OrcBomber",
    "RatmanArcher",
    "Dragon",
    "Drake",
    "Snake",
    "GreaterDragon",
    "IceSerpent",
    "GiantSerpent",
    "IceSnake",
    "LavaSerpent",
    "Lizardman",
    "Wyvern",
    "WhiteWyrm",
    "ShadowWyrm",
    "SilverSerpent",
    "LavaSnake",
    "EarthElemental",
    "PoisonElemental",
    "FireElemental",
    "SnowElemental",
    "IceElemental",
    "AcidElemental",
    "WaterElemental",
    "Efreet",
    "AirElemental",
    "Golem",
    "SewerRat",
    "GiantRat",
    "DireWolf",
    "TimberWolf",
    "Cougar",
    "Alligator",
];

#[derive(Clone, Debug)]
pub struct BrazierData {
    pub spawn_range: i32,
    pub next_spawn_delay: TimeSpan,
    pub next_spawn: muo_primitives::DateTime,
    pub timer: Option<TimerHandle>,
}

fn brazier(w: &World, id: ItemId) -> Option<BrazierData> {
    w.item(id)?.ext.get::<BrazierData>().cloned()
}

fn brazier_mut(w: &mut World, id: ItemId) -> Option<&mut BrazierData> {
    w.item_mut(id)?.ext.get_mut::<BrazierData>()
}

/// `DeceitBrazier()`.
pub fn deceit_brazier_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xE31);
    ops::set_movable(w, id, false);
    ops::set_light(w, id, muo_server::light_type::LightType::Circle225);
    let now = w.now;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BrazierData {
            spawn_range: 5,
            next_spawn_delay: TimeSpan::from_mins(15),
            next_spawn: now,
            timer: None,
        });
    }
    id
}

fn brazier_spawn_position(w: &mut World, id: ItemId) -> Point3D {
    let Some((loc, Some(map))) = w.item(id).map(|i| (i.location, i.map)) else {
        return w.item(id).map_or_else(Point3D::default, |i| i.location);
    };
    let range = brazier(w, id).map_or(5, |b| b.spawn_range);
    // Try 10 times to find a Spawnable location.
    for _ in 0..10 {
        let x = loc.x + (utility::random(&mut w.rng, range * 2 + 1) - range);
        let y = loc.y + (utility::random(&mut w.rng, range * 2 + 1) - range);
        let z = w.maps.get_average_z(map, x, y);
        if muo_server::map::fit::can_spawn_mobile_at(w, map, x, y, loc.z) {
            return Point3D::new(x, y, loc.z);
        }
        if muo_server::map::fit::can_spawn_mobile_at(w, map, x, y, z) {
            return Point3D::new(x, y, z);
        }
    }
    loc
}

fn brazier_do_effect(w: &mut World, loc: Point3D, map: Option<muo_server::map::MapId>) {
    let e = muo_server::item::effect_item::create_targeted(
        w,
        loc,
        map,
        muo_server::item::effect_item::DEFAULT_DURATION,
    );
    muo_server::effects::send_location_particles(w, e, 0x3709, 10, 30, 5052);
    muo_server::effects::play_sound(w, loc, map, 0x225);
}

fn brazier_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
    let from_loc = w.mobile(from).map_or_else(Point3D::default, |m| m.location);
    if !utility::in_range_3d(from_loc, loc, 2) {
        msg(w, from, 500446); // That is too far away.
        return;
    }
    let Some(b) = brazier(w, id) else { return };
    if b.next_spawn < w.now {
        let map = w.item(id).and_then(|i| i.map);
        let pick = utility::random(&mut w.rng, BRAZIER_CREATURES.len() as i32) as usize;
        let name = format!("Server.Mobiles.{}", BRAZIER_CREATURES[pick]);
        let Some(bc) = muo_content::factory::make_mobile(w, &name) else {
            return; // the type could not be made: nothing happens
        };
        let spawn_loc = brazier_spawn_position(w, id);
        brazier_do_effect(w, spawn_loc, map);
        timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
            // SummonCreatureToWorld
            let (home, range) = (
                w.item(id).map_or_else(Point3D::default, |i| i.location),
                brazier(w, id).map_or(5, |b| b.spawn_range),
            );
            muo_content::creature::control::set_home(w, bc, home);
            muo_content::creature::data::edit(w, bc, |c| c.range_home = range);
            muo_content::creature::data::edit(w, bc, |c| {
                c.fight_mode = muo_content::creature::enums::FightMode::Closest
            });
            if let Some(map) = map {
                muo_server::mobile::base::move_to_world(w, bc, spawn_loc, map);
            }
            brazier_do_effect(w, spawn_loc, map);
            if let Some(c) = muo_content::creature::vtable::ctype(w, bc) {
                (c.force_reacquire)(w, bc);
            }
        });
        let now = w.now;
        if let Some(d) = brazier_mut(w, id) {
            d.next_spawn = now + d.next_spawn_delay;
        }
    } else {
        // The brazier fizzes and pops, but nothing seems to happen.
        messages::public_overhead_message_number(w, id, ItemMessage::REGULAR, 0x3B2, 500760, "");
    }
}

item_leaf_raw! {
    /// `Server.Items.DeceitBrazier`.
    DECEIT_BRAZIER = "Server.Items.DeceitBrazier" : ITEM_BASE;
    construct: Some(|w| deceit_brazier_new(w, &DECEIT_BRAZIER)),
    label_number: |_, _| 1023633, // Brazier
    handles_on_movement: |_, _| true,
    on_movement: |w, id, m, old| {
        let Some(b) = brazier(w, id) else { return };
        let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
        let Some(mm) = w.mobile(m) else { return };
        let running = b.timer.is_some_and(|t| t.running(w));
        if b.next_spawn < w.now
            && utility::in_range_3d(mm.location, loc, 1)
            && !utility::in_range_3d(old, loc, 1)
            && mm.player
            && !(mm.access_level > AccessLevel::Player || mm.hidden)
            && !running
        {
            let t = timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
                // HeedWarning: Heed this warning well, and use this brazier at your own peril.
                messages::public_overhead_message_number(w, id, ItemMessage::REGULAR, 0x3B2, 500761, "");
                if let Some(mut t) = brazier(w, id).and_then(|b| b.timer) {
                    t.cancel(w);
                }
            });
            if let Some(d) = brazier_mut(w, id) {
                d.timer = Some(t);
            }
        }
    },
    on_double_click: brazier_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
        let b = brazier(w, id);
        out.write_i32(b.as_ref().map_or(5, |b| b.spawn_range));
        out.write_time_span_ticks(b.map_or(TimeSpan::from_mins(15), |b| b.next_spawn_delay).ticks());
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        let spawn_range = r.read_i32()?;
        let next_spawn_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
        let now = w.now; // AfterDeserialization: NextSpawn = Core.Now
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(BrazierData { spawn_range, next_spawn_delay, next_spawn: now, timer: None });
        }
        Ok(())
    },
}

// ---- WarningItem, HintItem ----

#[derive(Clone, Debug, Default)]
pub struct WarningData {
    pub warning: Option<TextDefinition>,
    pub range: i32,
    pub reset_delay: TimeSpan,
    /// `HintItem._hintMessage`.
    pub hint: Option<TextDefinition>,
    pub last_broadcast: muo_primitives::DateTime,
    pub broadcasting: bool,
}

pub fn warning(w: &World, id: ItemId) -> WarningData {
    w.item(id)
        .and_then(|i| i.ext.get::<WarningData>())
        .cloned()
        .unwrap_or_default()
}

pub fn warning_mut(w: &mut World, id: ItemId) -> Option<&mut WarningData> {
    let i = w.item_mut(id)?;
    if i.ext.get::<WarningData>().is_none() {
        i.ext = Ext::new(WarningData::default());
    }
    i.ext.get_mut::<WarningData>()
}

/// `WarningItem(itemID, range, warning)`.
pub fn warning_item_new(
    w: &mut World,
    ty: &'static ItemType,
    item_id: i32,
    range: i32,
    warning: TextDefinition,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    ops::set_movable(w, id, false);
    let now = w.now;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(WarningData {
            warning: Some(warning),
            range: range.min(18),
            last_broadcast: now,
            ..Default::default()
        });
    }
    // `m_LastBroadcast` is default(DateTime): the first broadcast is never inside the reset delay.
    if let Some(d) = warning_mut(w, id) {
        d.last_broadcast = muo_primitives::DateTime::MIN;
    }
    id
}

/// `HintItem(itemID, range, warning, hint)`.
pub fn hint_item_new(
    w: &mut World,
    item_id: i32,
    range: i32,
    warning: TextDefinition,
    hint: TextDefinition,
) -> ItemId {
    let id = warning_item_new(w, &HINT_ITEM, item_id, range, warning);
    if let Some(d) = warning_mut(w, id) {
        d.hint = Some(hint);
    }
    id
}

/// `WarningMessage = value`.
pub fn set_warning_message(w: &mut World, id: ItemId, value: TextDefinition) {
    if let Some(d) = warning_mut(w, id) {
        d.warning = Some(value);
    }
}

/// `HintMessage = value`.
pub fn set_hint_message(w: &mut World, id: ItemId, value: TextDefinition) {
    if let Some(d) = warning_mut(w, id) {
        d.hint = Some(value);
    }
}

/// `ResetDelay = value`.
pub fn set_reset_delay(w: &mut World, id: ItemId, value: TimeSpan) {
    if let Some(d) = warning_mut(w, id) {
        d.reset_delay = value;
    }
}

fn warning_only_to_triggerer(w: &World, id: ItemId) -> bool {
    is_a(w, id, &HINT_ITEM)
}

fn warning_send(
    w: &mut World,
    id: ItemId,
    triggerer: MobileId,
    only_to_triggerer: bool,
    m: Option<&TextDefinition>,
) {
    if only_to_triggerer {
        muo_content::text::send_message_to(w, m, triggerer);
    } else {
        muo_content::text::public_overhead_message_item(w, m, id, ItemMessage::REGULAR, 0x3B2);
    }
}

fn warning_broadcast(w: &mut World, id: ItemId, triggerer: MobileId) {
    let d = warning(w, id);
    if d.broadcasting || w.now < d.last_broadcast + d.reset_delay {
        return;
    }
    let now = w.now;
    if let Some(d) = warning_mut(w, id) {
        d.last_broadcast = now;
        d.broadcasting = true;
    }
    let only = warning_only_to_triggerer(w, id);
    warning_send(w, id, triggerer, only, d.warning.as_ref());
    // NeighborRange is 5 for both classes
    let (loc, map) = w
        .item(id)
        .map_or((Point3D::default(), None), |i| (i.location, i.map));
    let queue: Vec<ItemId> = muo_server::map::enumerate::items_in_range(w, map, loc.x, loc.y, 5)
        .filter(|&i| i != id && is_a(w, i, &WARNING_ITEM))
        .collect();
    for i in queue {
        warning_broadcast(w, i, triggerer);
    }
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        if let Some(d) = warning_mut(w, id) {
            d.broadcasting = false;
        }
    });
}

fn serialize_warning(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(1);
    let d = warning(w, id);
    out.write_text_definition(d.warning.as_ref());
    out.write_i32(d.range);
    out.write_time_span_ticks(d.reset_delay.ticks());
}

fn deserialize_warning(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<i32> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let (warning, range, reset_delay);
    if version < 1 {
        let s = r.read_string()?;
        let n = r.read_i32()?;
        range = r.read_i32()?;
        reset_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
        warning = Some(if n > 0 {
            TextDefinition::of_number(n)
        } else {
            TextDefinition::of(0, s.as_deref())
        });
    } else {
        warning = r.read_text_definition()?;
        range = r.read_i32()?;
        reset_delay = TimeSpan::from_ticks(r.read_time_span_ticks()?);
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(WarningData {
            warning,
            range,
            reset_delay,
            ..Default::default()
        });
    }
    Ok(version)
}

item_leaf_raw! {
    /// `Server.Items.WarningItem`.
    WARNING_ITEM = "Server.Items.WarningItem" : ITEM_BASE;
    construct: None, // `[add` uses `WARNING_ITEM_CTORS`
    handles_on_movement: |_, _| true,
    on_movement: |w, id, m, old| {
        let d = warning(w, id);
        let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
        let Some(mm) = w.mobile(m) else { return };
        if mm.player && utility::in_range_3d(mm.location, loc, d.range) && !utility::in_range_3d(old, loc, d.range) {
            warning_broadcast(w, id, m);
        }
    },
    serialize: |w, id, out| serialize_warning(w, id, out),
    deserialize: |w, id, r| deserialize_warning(w, id, r).map(|_| ()),
}

item_leaf_raw! {
    /// `Server.Items.HintItem`.
    HINT_ITEM = "Server.Items.HintItem" : WARNING_ITEM;
    on_double_click: |w, id, from| {
        let hint = warning(w, id).hint;
        warning_send(w, id, from, true, hint.as_ref());
    },
    serialize: |w, id, out| {
        serialize_warning(w, id, out);
        out.write_i32(1);
        out.write_text_definition(warning(w, id).hint.as_ref());
    },
    deserialize: |w, id, r| {
        let version = deserialize_warning(w, id, r)?;
        let _ = version;
        let hint_version = r.read_i32()?;
        let hint = if hint_version < 1 {
            let s = r.read_string()?;
            let n = r.read_i32()?;
            Some(if n > 0 { TextDefinition::of_number(n) } else { TextDefinition::of(0, s.as_deref()) })
        } else {
            r.read_text_definition()?
        };
        if let Some(d) = warning_mut(w, id) {
            d.hint = hint;
        }
        Ok(())
    },
}

// ---- GumpToggleItems, BaseWindChimes ----

/// `IGumpToggleItem`: how the gump reads and writes the item.
#[derive(Copy, Clone)]
pub struct ToggleItem {
    pub item: ItemId,
    pub turned_on: fn(&World, ItemId) -> bool,
    pub set_turned_on: fn(&mut World, ItemId, bool),
}

/// `TurnOnGump` / `TurnOffGump`.
pub struct TurnOnOffGump<const ON: bool> {
    item: ToggleItem,
}

impl<const ON: bool> StaticGumpLogic for TurnOnOffGump<ON> {
    const TYPE_NAME: &'static str = if ON {
        "Server.Items.TurnOnGump"
    } else {
        "Server.Items.TurnOffGump"
    };

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 300, 150, 0xA28);
        b.add_button(40, 53, 0xFA5, 0xFA7, 1);
        b.add_html_localized(80, 55, 65, 35, 1011036, false, false); // OKAY
        b.add_button(150, 53, 0xFA5, 0xFA7, 0);
        b.add_html_localized(190, 55, 100, 35, 1011012, false, false); // CANCEL
        if ON {
            b.add_html_localized(45, 20, 300, 35, 1011034, false, false); // Activate this item
        } else {
            b.add_html_localized(45, 20, 300, 35, 1011035, false, false); // Deactivate this item
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        if info.button_id != 1 {
            msg(w, from, 502694); // Cancelled action.
            return;
        }
        let new_value = !(self.item.turned_on)(w, self.item.item);
        (self.item.set_turned_on)(w, self.item.item, new_value);
        if new_value && !ops::is_locked_down(w, self.item.item) {
            msg(w, from, 502693); // Remember, this only works when locked down.
        }
    }
}

/// Sends `TurnOffGump` when the item is on, else `TurnOnGump`.
pub fn send_toggle_gump(w: &mut World, to: MobileId, item: ToggleItem) {
    let gump: Box<dyn Gump> = if (item.turned_on)(w, item.item) {
        Box::new(StaticGumpObj::new(
            150,
            200,
            TurnOnOffGump::<false> { item },
        ))
    } else {
        Box::new(StaticGumpObj::new(150, 200, TurnOnOffGump::<true> { item }))
    };
    send_gump_to_mobile(w, to, gump, false);
}

#[derive(Clone, Debug, Default)]
pub struct ChimesData {
    pub turned_on: bool,
}

fn chimes_on(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.ext.get::<ChimesData>())
        .is_some_and(|d| d.turned_on)
}

pub fn chimes_set_on(w: &mut World, id: ItemId, value: bool) {
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ChimesData { turned_on: value });
    }
    ops::invalidate_properties(w, id);
}

const WIND_CHIME_SOUNDS: [i32; 3] = [0x505, 0x506, 0x507];

fn chimes_new(w: &mut World, ty: &'static ItemType, graphic: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, graphic);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(ChimesData::default());
    }
    id
}

item_leaf_raw! {
    /// `Server.Items.BaseWindChimes`.
    BASE_WIND_CHIMES = "Server.Items.BaseWindChimes" : ITEM_BASE;
    construct: None,
    handles_on_movement: |w, id| chimes_on(w, id) && ops::is_locked_down(w, id),
    on_movement: |w, id, m, old| {
        let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
        let Some(mm) = w.mobile(m) else { return };
        if chimes_on(w, id)
            && ops::is_locked_down(w, id)
            && (!mm.hidden || mm.access_level == AccessLevel::Player)
            && utility::in_range_3d(mm.location, loc, 2)
            && !utility::in_range_3d(old, loc, 2)
        {
            let map = w.item(id).and_then(|i| i.map);
            let sound = super::random_element(w, &WIND_CHIME_SOUNDS);
            muo_server::effects::play_sound(w, loc, map, sound);
        }
    },
    get_properties: |w, id, list| {
        muo_server::item::base::get_properties_base(w, id, list);
        list.add(if chimes_on(w, id) { 502695 } else { 502696 }); // turned on / turned off
    },
    on_double_click: |w, id, from| {
        let owner = w.mobile(from).is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
            || muo_content::items::addon::find_house_at_item(w, id)
                .is_some_and(|h| muo_content::items::addon::house_is_owner(w, h, from));
        if !owner {
            msg(w, from, 502691); // You must be the owner to use this.
        } else {
            send_toggle_gump(w, from, ToggleItem { item: id, turned_on: chimes_on, set_turned_on: chimes_set_on });
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_bool(chimes_on(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_i32()?;
        let on = r.read_bool()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(ChimesData { turned_on: on });
        }
        Ok(())
    },
}

item_leaf_raw! {
    /// `Server.Items.WindChimes`.
    WIND_CHIMES = "Server.Items.WindChimes" : BASE_WIND_CHIMES;
    construct: Some(|w| chimes_new(w, &WIND_CHIMES, 0x2832)),
    serialize: |w, id, out| {
        (BASE_WIND_CHIMES.serialize)(w, id, out);
        out.write_i32(0); // the class's own `[SerializationGenerator(0, false)]` version
    },
    deserialize: |w, id, r| {
        (BASE_WIND_CHIMES.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
}

item_leaf_raw! {
    /// `Server.Items.FancyWindChimes`.
    FANCY_WIND_CHIMES = "Server.Items.FancyWindChimes" : BASE_WIND_CHIMES;
    construct: Some(|w| chimes_new(w, &FANCY_WIND_CHIMES, 0x2833)),
    serialize: |w, id, out| {
        (BASE_WIND_CHIMES.serialize)(w, id, out);
        out.write_i32(0); // the class's own `[SerializationGenerator(0, false)]` version
    },
    deserialize: |w, id, r| {
        (BASE_WIND_CHIMES.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    label_number: |_, _| 1030291,
}

// ---- SerpentPillar ----

#[derive(Clone, Debug, Default)]
pub struct PillarData {
    pub active: bool,
    pub word: Option<String>,
    pub destination: Rectangle2D,
}

fn pillar(w: &World, id: ItemId) -> PillarData {
    w.item(id)
        .and_then(|i| i.ext.get::<PillarData>())
        .cloned()
        .unwrap_or_default()
}

/// `SerpentPillar(word, destination, active)`.
pub fn serpent_pillar_new(
    w: &mut World,
    ty: &'static ItemType,
    word: Option<String>,
    destination: Rectangle2D,
    active: bool,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x233F);
    ops::set_movable(w, id, false);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(PillarData {
            active,
            word,
            destination,
        });
    }
    id
}

item_leaf_raw! {
    /// `Server.Items.SerpentPillar`.
    SERPENT_PILLAR = "Server.Items.SerpentPillar" : ITEM_BASE;
    construct: Some(|w| serpent_pillar_new(w, &SERPENT_PILLAR, None, Rectangle2D::default(), false)),
    handles_on_speech: |_, _| true,
    on_speech: |w, id, e| {
        let d = pillar(w, id);
        let from = e.mobile;
        let loc = w.item(id).map_or_else(Point3D::default, |i| i.location);
        let Some(fm) = w.mobile(from) else { return };
        if !e.handled && utility::in_range_3d(fm.location, loc, 10) && Some(e.speech.lower_inv()) == d.word {
            use muo_multis::boats;
            let (from_loc, map) = (fm.location, fm.map);
            let Some(boat) = boats::find_boat_at(w, from_loc, map) else { return };
            let tiller = boats::tiller_man(w, boat);
            if !d.active {
                // Ar, Legend has it that these pillars are inactive! No man knows how it might be undone!
                if let Some(t) = tiller {
                    boats::parts::tiller_say(w, t, 502507);
                }
                return;
            }
            let Some(map) = map else { return };
            let boat_gid = w.item(boat).map_or(0, |i| i.item_id);
            for _ in 0..5 {
                // Try 5 times
                let x = utility::random_from(&mut w.rng, d.destination.x(), d.destination.width());
                let y = utility::random_from(&mut w.rng, d.destination.y(), d.destination.height());
                let z = w.maps.get_average_z(map, x, y);
                let dest = Point3D::new(x, y, z);
                if boats::can_fit(w, boat, dest, Some(map), boat_gid) {
                    let bl = w.item(boat).map_or_else(Point3D::default, |i| i.location);
                    boats::teleport(w, boat, x - bl.x, y - bl.y, z - bl.z);
                    return;
                }
            }
            if let Some(t) = tiller {
                boats::parts::tiller_say(w, t, 502508); // Ar, I refuse to take that matey through here!
            }
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(0);
        let d = pillar(w, id);
        out.write_bool(d.active);
        out.write_opt_string(d.word.as_deref());
        out.write_rect2d(d.destination);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        let active = r.read_bool()?;
        let word = r.read_string()?;
        let destination = r.read_rect2d()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(PillarData { active, word, destination });
        }
        Ok(())
    },
}

// ---- tables, registration ----

fn kind_int() -> Kind {
    Kind::Plain(PropType::Numeric(Numeric::Int))
}

const fn int(name: &'static str) -> Param {
    Param {
        name,
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: None,
        ty: "",
    }
}

const fn string(name: &'static str) -> Param {
    Param {
        name,
        kind: Kind::Plain(PropType::Str),
        default: None,
        ty: "",
    }
}

fn text_arg(v: &Value, as_number: bool) -> TextDefinition {
    if as_number {
        TextDefinition::of_number(v.i32())
    } else {
        TextDefinition::of_text(v.string().unwrap_or_default())
    }
}

static WARNING_ITEM_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[int("itemID"), int("range"), int("warning")],
        make: |w, v| {
            EntityId::Item(warning_item_new(
                w,
                &WARNING_ITEM,
                v[0].i32(),
                v[1].i32(),
                text_arg(&v[2], true),
            ))
        },
    },
    Ctor {
        access: None,
        params: &[int("itemID"), int("range"), string("warning")],
        make: |w, v| {
            EntityId::Item(warning_item_new(
                w,
                &WARNING_ITEM,
                v[0].i32(),
                v[1].i32(),
                text_arg(&v[2], false),
            ))
        },
    },
];

static HINT_ITEM_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[int("itemID"), int("range"), int("warning"), int("hint")],
        make: |w, v| {
            EntityId::Item(hint_item_new(
                w,
                v[0].i32(),
                v[1].i32(),
                text_arg(&v[2], true),
                text_arg(&v[3], true),
            ))
        },
    },
    Ctor {
        access: None,
        params: &[
            int("itemID"),
            int("range"),
            string("warning"),
            string("hint"),
        ],
        make: |w, v| {
            EntityId::Item(hint_item_new(
                w,
                v[0].i32(),
                v[1].i32(),
                text_arg(&v[2], false),
                text_arg(&v[3], false),
            ))
        },
    },
];

fn amount_ctor(name: &'static str) -> Param {
    Param {
        name,
        kind: kind_int(),
        default: Some(Value::Int(1)),
        ty: "",
    }
}

static BOLA_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(bola_new(w, &BOLA, v[0].i32())),
}];
static OIL_FLASK_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(oil_flask_new(w, &OIL_FLASK, v[0].i32())),
}];
static FISH_OIL_FLASK_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(oil_flask_new(w, &FISH_OIL_FLASK, v[0].i32())),
}];
static FIREBOMB_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "itemID",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        default: Some(Value::Int(0x99B)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(firebomb_new(w, &FIREBOMB, v[0].i32())),
}];

pub(crate) fn gadgets_configure(w: &mut World) {
    let _ = amount_ctor;
    // `OilCloth : IScissorable, IDyable`.
    muo_content::items::interfaces::register_scissorable(w, &OIL_CLOTH, |w, id, from, _| {
        oil_cloth_scissor(w, id, from)
    });
    muo_content::items::interfaces::register_dyable(w, &OIL_CLOTH, |w, id, _, tub| {
        let h = muo_content::items::clothing::tub_dyed_hue(w, tub);
        oil_cloth_dye(w, id, h)
    });
    for (name, t) in [
        ("Server.Items.WarningItem", &WARNING_ITEM_CTORS[..]),
        ("Server.Items.HintItem", &HINT_ITEM_CTORS[..]),
        ("Server.Items.Bola", &BOLA_CTORS[..]),
        ("Server.Items.OilFlask", &OIL_FLASK_CTORS[..]),
        ("Server.Items.FishOilFlask", &FISH_OIL_FLASK_CTORS[..]),
        ("Server.Items.Firebomb", &FIREBOMB_CTORS[..]),
    ] {
        add_ctors(w, name, t);
    }
    // `UnholyBone : ICarvable`: a bladed item in range reaches it through `ICarvable` (BladedItemTarget, HarvestTarget).
    muo_content::items::interfaces::register_carvable(w, &UNHOLY_BONE, |w, id, from, blade| {
        unholy_bone_carve(w, id, from, blade);
    });
    w.resources
        .get_mut::<muo_content::targets::TargetHooks>()
        .unholy_bone_carve = Some(unholy_bone_carve);
    muo_content::hooks::hooks_mut(w).cancel_bola_target = Some(|w, m| {
        if current_tag(w, m).is_some_and(|(k, _)| k == "BolaTarget") {
            target::cancel_mobile_target(w, m);
        }
    });
}

/// The classes of this file.
pub static ALL: &[&ItemType] = &[
    &ARCANE_GEM,
    &TRIBAL_PAINT,
    &OIL_CLOTH,
    &OIL_FLASK,
    &FISH_OIL_FLASK,
    &BOLA,
    &CLOCKWORK_ASSEMBLY,
    &FIREBOMB,
    &UNHOLY_BONE,
    &DECEIT_BRAZIER,
    &WARNING_ITEM,
    &HINT_ITEM,
    &BASE_WIND_CHIMES,
    &WIND_CHIMES,
    &FANCY_WIND_CHIMES,
    &SERPENT_PILLAR,
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.Gadgets.Configure",
        gadgets_configure,
    );
}
