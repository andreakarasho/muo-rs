//! `PotionKeg`. State: [`KegData`] (`Type`, `Held`).

use muo_content::props::{Ctor, add_ctors};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, TotalType, base, container, deps, ops};
use muo_server::mobile::part2::effects;
use muo_server::packets::message::MessageType;
use muo_server::property_list::PropertyList;
use muo_server::type_registry::TypeRegistry;

use super::potions::{self, fx};

/// `PotionKeg._type` and `_held`.
#[derive(Clone, Copy, Debug, Default)]
pub struct KegData {
    pub effect: i32,
    pub held: i32,
}

fn data(w: &World, id: ItemId) -> KegData {
    w.item(id)
        .and_then(|i| i.ext.get::<KegData>().copied())
        .unwrap_or_default()
}

/// `Held`.
pub fn held(w: &World, id: ItemId) -> i32 {
    data(w, id).held
}

/// `Type`.
pub fn keg_type(w: &World, id: ItemId) -> i32 {
    data(w, id).effect
}

/// `Held = value`: the pile weight is moved up the containers.
pub fn set_held(w: &mut World, id: ItemId, value: i32) {
    let old = held(w, id);
    if old == value {
        return;
    }
    let old_pile = ops::pile_weight(w, id);
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<KegData>()) {
        d.held = value;
    }
    let new_pile = ops::pile_weight(w, id);
    ops::update_total(w, id, id, TotalType::Weight, new_pile - old_pile);
    ops::invalidate_properties(w, id);
}

fn set_type(w: &mut World, id: ItemId, effect: i32) {
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<KegData>())
        && d.effect != effect
    {
        d.effect = effect;
        ops::invalidate_properties(w, id);
    }
}

fn label_number(w: &World, id: ItemId) -> i32 {
    let d = data(w, id);
    if d.held <= 0 {
        return 1041641;
    }
    match d.effect {
        e if !(fx::NIGHTSIGHT..=fx::FLINTS_PUNGENT_BREW).contains(&e) => 1041619,
        fx::FLINTS_PUNGENT_BREW => 1113608,
        e if e >= fx::PARASITIC => 1080069 + e - fx::PARASITIC,
        fx::INVISIBILITY => 1080071,
        e if e >= fx::CONFLAGRATION => 1072658 + e - fx::CONFLAGRATION,
        e => 1041620 + e,
    }
}

fn keg_label(held: i32) -> i32 {
    match held {
        ..=0 => 502246,   // The keg is empty.
        1..=4 => 502248,  // The keg is nearly empty.
        5..=19 => 502249, // The keg is not very full.
        20..=29 => 502250,
        30..=39 => 502251,
        40..=46 => 502252,
        47..=53 => 502254,
        54..=69 => 502253,
        70..=79 => 502255,
        80..=95 => 502256,
        96..=99 => 502257,
        _ => 502258, // The keg is completely full.
    }
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    list.add(keg_label(held(w, id)));
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    base::on_single_click_base(w, id, from);
    ops::label_to_number(w, id, from, keg_label(held(w, id)), "");
}

/// `FillBottle(effect)`.
pub fn fill_bottle(w: &mut World, which: i32) -> ItemId {
    use potions::{
        new_agility, new_conflagration, new_confusion_blast, new_cure, new_darkglow,
        new_deadly_poison, new_explosion, new_greater_agility, new_greater_conflagration,
        new_greater_confusion_blast, new_greater_cure, new_greater_explosion, new_greater_heal,
        new_greater_poison, new_greater_strength, new_heal, new_invisibility, new_lesser_cure,
        new_lesser_explosion, new_lesser_heal, new_lesser_poison, new_night_sight, new_parasitic,
        new_poison, new_refresh, new_strength, new_total_refresh,
    };
    let make: fn(&mut World) -> ItemId = match which {
        fx::CURE_LESSER => new_lesser_cure,
        fx::CURE => new_cure,
        fx::CURE_GREATER => new_greater_cure,
        fx::AGILITY => new_agility,
        fx::AGILITY_GREATER => new_greater_agility,
        fx::STRENGTH => new_strength,
        fx::STRENGTH_GREATER => new_greater_strength,
        fx::POISON_LESSER => new_lesser_poison,
        fx::POISON => new_poison,
        fx::POISON_GREATER => new_greater_poison,
        fx::POISON_DEADLY => new_deadly_poison,
        fx::REFRESH => new_refresh,
        fx::REFRESH_TOTAL => new_total_refresh,
        fx::HEAL_LESSER => new_lesser_heal,
        fx::HEAL => new_heal,
        fx::HEAL_GREATER => new_greater_heal,
        fx::EXPLOSION_LESSER => new_lesser_explosion,
        fx::EXPLOSION => new_explosion,
        fx::EXPLOSION_GREATER => new_greater_explosion,
        fx::CONFLAGRATION => new_conflagration,
        fx::CONFLAGRATION_GREATER => new_greater_conflagration,
        fx::CONFUSION_BLAST => new_confusion_blast,
        fx::CONFUSION_BLAST_GREATER => new_greater_confusion_blast,
        fx::INVISIBILITY => new_invisibility,
        fx::PARASITIC => new_parasitic,
        fx::DARKGLOW => new_darkglow,
        _ => new_night_sight,
    };
    make(w)
}

fn give_bottle(w: &mut World, m: MobileId, amount: i32) -> bool {
    let bottle = super::plain::new_bottle(w, amount);
    let pack = w.mobile(m).and_then(|x| x.backpack);
    if !pack.is_some_and(|p| container::try_drop_item(w, p, Some(m), bottle, false)) {
        base::delete(w, bottle);
        return false;
    }
    true
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let loc = ops::get_world_location(w, id);
    if !deps::mobile_in_range(w, from, loc, 2) {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x3B2, 1019045); // I can't reach that.
        return;
    }
    if held(w, id) <= 0 {
        deps::send_localized_message(w, from, 502246); // The keg is empty.
        return;
    }
    let Some(pack) = w.mobile(from).and_then(|m| m.backpack) else {
        return;
    };
    if container::consume_total(w, pack, &super::plain::BOTTLE, 1, true, None) {
        deps::send_localized_message(w, from, 502242); // You pour some of the keg's contents into an empty bottle...
        let effect = keg_type(w, id);
        let pot = fill_bottle(w, effect);
        if container::try_drop_item(w, pack, Some(from), pot, false) {
            deps::send_localized_message(w, from, 502243); // ...and place it into your backpack.
            effects::play_sound(w, from, 0x240);
            let h = held(w, id) - 1;
            set_held(w, id, h);
            if h == 0 {
                deps::send_localized_message(w, from, 502245); // The keg is now empty.
            }
        } else {
            deps::send_localized_message(w, from, 502244); // ...but there is no room for the bottle in your backpack.
            base::delete(w, pot);
        }
    }
}

fn on_drag_drop(w: &mut World, id: ItemId, from: MobileId, item: ItemId) -> bool {
    if !potions::is_potion(w, item) {
        deps::send_localized_message(w, from, 502232); // The keg is not designed to hold that type of object.
        return false;
    }
    let pot_effect = potions::effect(w, item).unwrap_or(0);
    let amount = w.item(item).map_or(0, |i| i.amount);
    let to_hold = (100 - held(w, id)).min(amount);
    if to_hold <= 0 {
        deps::send_localized_message(w, from, 502233); // The keg will not hold any more!
        return false;
    }
    if held(w, id) == 0 {
        if pot_effect >= fx::INVISIBILITY {
            deps::send_localized_message(w, from, 502232); // The keg is not designed to hold that type of object.
            return false;
        }
        if give_bottle(w, from, to_hold) {
            set_type(w, id, pot_effect);
            set_held(w, id, to_hold);
            finish_pour(w, from, item, to_hold);
            return true;
        }
        deps::send_localized_message(w, from, 502238); // You don't have room for the empty bottle in your backpack.
        return false;
    }
    if pot_effect != keg_type(w, id) {
        deps::send_localized_message(w, from, 502236); // You decide that it would be a bad idea to mix different types of potions.
        return false;
    }
    if give_bottle(w, from, to_hold) {
        let h = held(w, id) + to_hold;
        set_held(w, id, h);
        finish_pour(w, from, item, to_hold);
        return true;
    }
    deps::send_localized_message(w, from, 502238); // You don't have room for the empty bottle in your backpack.
    false
}

fn finish_pour(w: &mut World, from: MobileId, pot: ItemId, to_hold: i32) {
    effects::play_sound(w, from, 0x240);
    deps::send_localized_message(w, from, 502237); // You place the empty bottle in your backpack.
    base::consume(w, pot, to_hold);
    if w.item(pot).is_some_and(|i| !i.deleted()) {
        ops::bounce(w, pot, from);
    }
}

/// `new PotionKeg { Held = 100, Type = PotionEffect.<name>, Hue = hue }` (the Test Center's kegs).
pub fn new_test_keg(w: &mut World, effect: &str, hue: i32) -> Option<ItemId> {
    let e = match effect {
        "CureGreater" => fx::CURE_GREATER,
        "HealGreater" => fx::HEAL_GREATER,
        "PoisonDeadly" => fx::POISON_DEADLY,
        "RefreshTotal" => fx::REFRESH_TOTAL,
        "ExplosionGreater" => fx::EXPLOSION_GREATER,
        _ => return None,
    };
    let id = (POTION_KEG.construct.expect("construct"))(w);
    set_held(w, id, 100);
    set_type(w, id, e);
    (w.item(id)?.ty.set_hue)(w, id, hue);
    Some(id)
}

/// `Server.Items.PotionKeg`.
pub static POTION_KEG: ItemType = ItemType {
    full_name: "Server.Items.PotionKeg",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &POTION_KEG, 0x1940);
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(KegData::default());
        }
        id
    }),
    default_weight: |w, id| 20.0 + f64::from(held(w, id).clamp(0, 100)) * 0.8,
    label_number,
    get_properties,
    on_single_click,
    on_double_click,
    on_drag_drop,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        let d = data(w, id);
        out.write_i32(2);
        out.write_enum(4, i64::from(d.effect));
        out.write_i32(d.held);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        // Version 2 writes the enum with `WriteEnum` (an encoded int); the version 0 and 1 layouts a plain int.
        let effect = if version >= 2 {
            r.read_enum(4)? as i32
        } else {
            r.read_i32()?
        };
        let held = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(KegData { effect, held });
        }
        Ok(())
    },
    ..ITEM_BASE
};

static KEG_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[],
    make: |w, _| EntityId::Item((POTION_KEG.construct.expect("set"))(w)),
}];

fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.PotionKeg", &KEG_CTOR);
    let f = &mut w.resources.get_mut::<muo_content::skills::SkillHooks>().fns;
    f.potion_keg = Some(|w, item| {
        w.item(item)
            .is_some_and(|i| i.ty.is(&POTION_KEG))
            .then(|| (held(w, item), label_number(w, item)))
    });
}

/// `PotionKeg.Initialize()`: `TileData.ItemTable[0x1940].Height = 4`.
fn initialize(w: &mut World) {
    if let Some(i) = w.maps.tile_data.items.get_mut(0x1940) {
        i.height = 4;
    }
}

/// Registers `PotionKeg`.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&POTION_KEG, &[]);
    types.register_phase_default(
        muo_server::type_registry::Phase::Initialize,
        "Server.Items.PotionKeg.Initialize",
        initialize,
    );
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.PotionKeg.Configure",
        configure,
    );
}
