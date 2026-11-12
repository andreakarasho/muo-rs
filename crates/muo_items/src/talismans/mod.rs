//! Talismans: `BaseTalisman` (+ Migrations), `RandomTalisman`, `TalismanAttribute`,
//! `TalismanSlayer`, the five items in `Items/` and the summoned creatures (`summons.rs`).
//!
//! State is `muo_content::items::ext_bags::TalismanData` in `Item::ext`, so `items::attr` reaches the attribute and
//! skill bonus bags. The virtuals subclasses override are [`TalismanClass`]. [`configure`] sets the hooks the combat
//! code calls (`talisman_killer_bonus`, `talisman_slays`, `talisman_protection`). The spells a removal ends are
//! behind [`TalismanHooks`].

use muo_content::items::ext_bags::{TalismanAttr, TalismanData};
use muo_content::items::weapon_hooks::WeaponHooks;
use muo_content::items::{skill_bonuses_add_to, skill_bonuses_remove};
use muo_content::player_mobile::hooks::SystemHooks;
use muo_io::text::TextDefinition;
use muo_io::{GenericReader, GenericWriter, IoError, Result};
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, base, deps, effect_item, ops};
use muo_server::layer::Layer;
use muo_server::mobile::MobileType;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{EntityType, Phase, TypeRegistry};

pub mod data;
pub mod items;
pub mod summons;

use data::{ITEM_IDS, KILLER_LABELS, KILLERS, SKILLS, SLAYER_TABLE, SUMMON_LABELS, SUMMONS};

/// `TalismanRemoval`.
pub mod removal {
    pub const NONE: i32 = 0;
    pub const WARD: i32 = 390;
    pub const DAMAGE: i32 = 404;
    pub const CURSE: i32 = 407;
    pub const WILDFIRE: i32 = 2843;

    pub(super) fn is_known(v: i32) -> bool {
        matches!(v, NONE | WARD | DAMAGE | CURSE | WILDFIRE)
    }
}

/// `TalismanSlayerName`.
pub mod slayer_name {
    pub const NONE: i32 = 0;
    pub const BOVINE: i32 = 9;
}

// ---- hooks ----

/// What a talisman's removal calls into (the spells it ends). `kind` names the call: "EvilOmenSpell.EndEffect",
/// "StrangleSpell.RemoveCurse", "CorpseSkinSpell.RemoveCurse", "CurseSpell.RemoveEffect", "BleedAttack.EndBleed",
/// "MortalStrike.EndWound", "MagicReflectSpell.EndReflect", "ReactiveArmorSpell.EndArmor",
/// "ProtectionSpell.EndProtection". Unset: nothing (no such effect is running).
#[derive(Default)]
pub struct TalismanHooks {
    pub end_effect: Option<fn(&mut World, &'static str, MobileId)>,
}

fn end_effect(w: &mut World, kind: &'static str, target: MobileId) {
    if let Some(f) = w
        .resources
        .get::<TalismanHooks>()
        .and_then(|h| h.end_effect)
    {
        f(w, kind, target);
    }
}

// ---- the class ----

/// The virtuals of the talisman classes that are not `Item` virtuals.
#[derive(Copy, Clone)]
pub struct TalismanClass {
    /// `ForceShowName`.
    pub force_show_name: bool,
    /// `GetSummoner()`: the class (full name) summoned when the talisman has no summoner of its own.
    pub get_summoner: fn(&mut World, ItemId) -> Option<&'static str>,
}

/// `BaseTalisman`' defaults.
pub static BASE_TALISMAN_CLASS: TalismanClass = TalismanClass {
    force_show_name: false,
    get_summoner: |_, _| None,
};

pub fn data(w: &World, id: ItemId) -> Option<&TalismanData> {
    w.item(id)?.ext.get::<TalismanData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut TalismanData> {
    w.item_mut(id)?.ext.get_mut::<TalismanData>()
}

fn class(w: &World, id: ItemId) -> &'static TalismanClass {
    w.item(id)
        .and_then(|i| i.ty.family::<TalismanClass>())
        .unwrap_or(&BASE_TALISMAN_CLASS)
}

/// `item is BaseTalisman`.
pub fn is_talisman(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_TALISMAN))
}

/// The talisman a mobile wears (`m.Talisman as BaseTalisman`).
pub fn talisman_of(w: &World, m: MobileId) -> Option<ItemId> {
    muo_server::mobile::part1::props_b::talisman(w, m).filter(|&t| is_talisman(w, t))
}

// ---- setters (`[InvalidateProperties]` fields) ----

macro_rules! setter {
    ($(#[$m:meta])* $name:ident, $field:ident: $t:ty) => {
        $(#[$m])*
        pub fn $name(w: &mut World, id: ItemId, value: $t) {
            if let Some(d) = data_mut(w, id)
                && d.$field != value
            {
                d.$field = value;
                ops::invalidate_properties(w, id);
            }
        }
    };
}

setter!(
    /// `Removal` setter.
    set_removal, removal: i32
);
setter!(
    /// `Skill` setter.
    set_skill, skill: i32
);
setter!(
    /// `SuccessBonus` setter.
    set_success_bonus, success_bonus: i32
);
setter!(
    /// `ExceptionalBonus` setter.
    set_exceptional_bonus, exceptional_bonus: i32
);
setter!(
    /// `MaxCharges` setter.
    set_max_charges, max_charges: i32
);
setter!(
    /// `MaxChargeTime` setter.
    set_max_charge_time, max_charge_time: i32
);
setter!(
    /// `ChargeTime` setter.
    set_charge_time, charge_time: i32
);
setter!(
    /// `Blessed` setter.
    set_blessed, blessed: bool
);
setter!(
    /// `Slayer` setter.
    set_slayer, slayer: i32
);

/// `Charges` setter: `[InvalidateProperties]` and `OnChargesChanged` (starts the recharge timer while a charge
/// time is pending).
pub fn set_charges(w: &mut World, id: ItemId, value: i32) {
    let Some(d) = data_mut(w, id) else { return };
    if d.charges == value {
        return;
    }
    d.charges = value;
    let charge_time = d.charge_time;
    ops::invalidate_properties(w, id);
    if charge_time > 0 {
        start_timer(w, id);
    }
}

/// `Protection`/`Killer`/`Summoner` setters (`[InvalidateProperties]`).
pub fn set_protection(w: &mut World, id: ItemId, value: TalismanAttr) {
    if let Some(d) = data_mut(w, id) {
        d.protection = value;
        ops::invalidate_properties(w, id);
    }
}

pub fn set_killer(w: &mut World, id: ItemId, value: TalismanAttr) {
    if let Some(d) = data_mut(w, id) {
        d.killer = value;
        ops::invalidate_properties(w, id);
    }
}

pub fn set_summoner(w: &mut World, id: ItemId, value: TalismanAttr) {
    if let Some(d) = data_mut(w, id) {
        d.summoner = value;
        ops::invalidate_properties(w, id);
    }
}

/// `new TalismanAttribute(owner, type, name, amount)`.
pub fn attribute(type_name: &str, name: TextDefinition, amount: i32) -> TalismanAttr {
    TalismanAttr {
        type_name: Some(type_name.to_string()),
        name: Some(name),
        amount,
    }
}

/// `TalismanAttribute.IsItem => _type.IsAssignableTo(typeof(Item))`.
pub fn attr_is_item(w: &World, a: &TalismanAttr) -> bool {
    a.type_name.as_deref().is_some_and(|n| {
        matches!(
            w.types.find_type_by_full_name(n, false),
            Some(EntityType::Item(_))
        )
    })
}

/// `SetSummoner(type, name)` (virtual; not overridden).
pub fn set_summoner_type(w: &mut World, id: ItemId, type_name: &str, name: TextDefinition) {
    set_summoner(w, id, attribute(type_name, name, 0));
}

/// `SetProtection(type, name, amount)`.
pub fn set_protection_type(
    w: &mut World,
    id: ItemId,
    type_name: &str,
    name: TextDefinition,
    amount: i32,
) {
    set_protection(w, id, attribute(type_name, name, amount));
}

/// `SetKiller(type, name, amount)`.
pub fn set_killer_type(
    w: &mut World,
    id: ItemId,
    type_name: &str,
    name: TextDefinition,
    amount: i32,
) {
    set_killer(w, id, attribute(type_name, name, amount));
}

// ---- constructors ----

/// `GetRandomItemID()`.
pub fn random_item_id(w: &mut World) -> i32 {
    let i = utility::random(&mut w.rng, ITEM_IDS.len() as i32) as usize;
    ITEM_IDS[i]
}

/// `new BaseTalisman(itemID)` for class `ty`: `Layer = Layer.Talisman` and empty attribute bags.
pub fn new_talisman(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    (ty.set_layer)(w, id, Layer::Talisman);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(TalismanData::default());
    }
    id
}

/// `new BaseTalisman()`: `this(GetRandomItemID())`.
pub fn new_talisman_random(w: &mut World, ty: &'static ItemType) -> ItemId {
    let gid = random_item_id(w);
    new_talisman(w, ty, gid)
}

// ---- the recharge timer ----

/// `StartTimer()`: every 10 seconds, `Slice()`.
pub fn start_timer(w: &mut World, id: ItemId) {
    let running =
        data(w, id).is_some_and(|d| d.timer != TimerHandle::default() && d.timer.running(w));
    if running {
        return;
    }
    let h = timer::start(
        w,
        TimeSpan::from_secs(10),
        TimeSpan::from_secs(10),
        0,
        move |w| slice(w, id),
    );
    if let Some(d) = data_mut(w, id) {
        d.timer = h;
    }
}

/// `StopTimer()`.
pub fn stop_timer(w: &mut World, id: ItemId) {
    if let Some(d) = data_mut(w, id) {
        let mut h = d.timer;
        d.timer = TimerHandle::default();
        h.cancel(w);
    }
}

/// `Slice()`.
pub fn slice(w: &mut World, id: ItemId) {
    if ops::is_deleted(w, id) {
        stop_timer(w, id);
        return;
    }
    let Some(ct) = data(w, id).map(|d| d.charge_time) else {
        return;
    };
    if ct - 10 > 0 {
        if let Some(d) = data_mut(w, id) {
            d.charge_time = ct - 10;
        }
    } else {
        if let Some(d) = data_mut(w, id) {
            d.charge_time = 0;
        }
        stop_timer(w, id);
    }
    ops::invalidate_properties(w, id);
}

/// `OnAfterUse(m)`.
pub fn on_after_use(w: &mut World, id: ItemId) {
    let Some(d) = data_mut(w, id) else { return };
    d.charge_time = d.max_charge_time;
    if d.charges > 0 && d.max_charges > 0 {
        d.charges -= 1;
    }
    let charge_time = d.charge_time;
    if charge_time > 0 {
        start_timer(w, id);
    }
    ops::invalidate_properties(w, id);
}

// ---- stat bonuses of the AosAttributes bag ----

fn add_stat_bonuses(w: &mut World, id: ItemId, to: MobileId) {
    crate::quivers::add_stat_bonuses(w, id, to);
}

fn remove_stat_bonuses(w: &mut World, id: ItemId, from: MobileId) {
    crate::quivers::remove_stat_bonuses(w, id, from);
}

// ---- virtuals ----

fn blessed_for(w: &World, id: ItemId) -> Option<MobileId> {
    w.item(id).and_then(|i| i.blessed_for)
}

fn can_equip(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if let Some(owner) = blessed_for(w, id)
        && owner != from
    {
        deps::send_localized_message(w, from, 1010437); // You are not the owner.
        return false;
    }
    (ITEM_BASE.can_equip)(w, id, from)
}

fn on_added(w: &mut World, id: ItemId, parent: EntityId) {
    if let EntityId::Mobile(from) = parent {
        skill_bonuses_add_to(w, id, from);
        add_stat_bonuses(w, id, from);
        let blessed = data(w, id).is_some_and(|d| d.blessed);
        if blessed && blessed_for(w, id).is_none() {
            if let Some(i) = w.item_mut(id) {
                i.blessed_for = Some(from);
            }
            ops::set_loot_type(w, id, LootType::Blessed);
        }
        if let Some(d) = data_mut(w, id)
            && d.charge_time > 0
        {
            d.charge_time = d.max_charge_time;
            start_timer(w, id);
        }
    }
    ops::invalidate_properties(w, id);
}

fn on_removed(w: &mut World, id: ItemId, parent: EntityId) {
    if let EntityId::Mobile(from) = parent {
        skill_bonuses_remove(w, id);
        remove_stat_bonuses(w, id, from);
        let creature = data(w, id).and_then(|d| d.creature);
        if let Some(c) = creature.filter(|&c| w.mobile(c).is_some_and(|m| !m.deleted)) {
            dismiss_creature(w, c);
        }
        stop_timer(w, id);
    }
    ops::invalidate_properties(w, id);
}

/// The particles and sound at the creature, then `_creature.Delete()`.
fn dismiss_creature(w: &mut World, c: MobileId) {
    let Some((loc, map)) = w.mobile(c).map(|m| (m.location, m.map)) else {
        return;
    };
    let fx = effect_item::create(w, loc, map, effect_item::DEFAULT_DURATION);
    muo_server::effects::send_location_particles(w, fx, 0x3728, 8, 20, 5042);
    muo_server::effects::play_sound_entity(w, c, 0x201);
    if let Some(ty) = w.mobile(c).map(|m| m.ty) {
        (ty.delete)(w, c);
    }
}

fn on_delete(w: &mut World, id: ItemId) {
    // `_creature?.Delete(); // Will stop the unsummon timer on OnAfterDelete()`
    if let Some(c) = data(w, id).and_then(|d| d.creature)
        && let Some(ty) = w.mobile(c).map(|m| m.ty)
    {
        (ty.delete)(w, c);
    }
}

fn on_after_duped(w: &mut World, id: ItemId, new_item: ItemId) {
    let Some(src) = data(w, id).cloned() else {
        return;
    };
    if let Some(d) = data_mut(w, new_item) {
        d.summoner = src.summoner;
        d.protection = src.protection;
        d.killer = src.killer;
        d.attributes = src.attributes;
        d.skill_bonuses = src.skill_bonuses;
    }
}

fn add_name_property(w: &World, id: ItemId, list: &mut PropertyList) {
    let Some(d) = data(w, id) else { return };
    if class(w, id).force_show_name {
        base::add_name_property_base(w, id, list);
    } else if !d.summoner.is_empty() {
        match d.summoner.name.as_ref() {
            Some(n) if n.number > 0 => list.add_localized_number(1072400, n.number), // Talisman of ~1_name~ Summoning
            n => list.add_str(
                1072400,
                n.and_then(|n| n.string.as_deref()).unwrap_or("Unknown"),
            ),
        }
    } else if d.removal != removal::NONE {
        list.add_localized_number(1072389, 1072000 + d.removal); // Talisman of ~1_name~
    } else {
        base::add_name_property_base(w, id, list);
    }
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    let Some(d) = data(w, id) else { return };
    if d.blessed {
        match blessed_for(w, id) {
            Some(m) => {
                let name = w
                    .mobile(m)
                    .and_then(|mm| (mm.ty.name)(w, m))
                    .filter(|n| !n.is_empty());
                list.add_str(1072304, name.as_deref().unwrap_or("Unnamed Warrior")); // Owned by ~1_name~
            }
            None => list.add_str(1072304, "Nobody"),
        }
    }
    if matches!(w.item(id).and_then(|i| i.parent), Some(EntityId::Mobile(_)))
        && d.max_charge_time > 0
    {
        if d.charge_time > 0 {
            list.add_int(1074884, d.charge_time); // Charge time left: ~1_val~
        } else {
            list.add(1074883); // Fully Charged
        }
    }
    list.add(1075085); // Requirement: Mondain's Legacy
    if !d.killer.is_empty() && d.killer.amount > 0 {
        // ~1_NAME~ Killer: +~2_val~%
        list.add_str(
            1072388,
            &format!("{}\t{}", attr_name(&d.killer), d.killer.amount),
        );
    }
    if !d.protection.is_empty() && d.protection.amount > 0 {
        // ~1_NAME~ Protection: +~2_val~%
        list.add_str(
            1072387,
            &format!("{}\t{}", attr_name(&d.protection), d.protection.amount),
        );
    }
    let skill = SkillName::from_i32(d.skill).map_or(0, muo_content::aos::skill_label);
    if d.exceptional_bonus != 0 {
        // ~1_NAME~ Exceptional Bonus: ~2_val~%
        list.add_str(1072395, &format!("#{skill}\t{}", d.exceptional_bonus));
    }
    if d.success_bonus != 0 {
        // ~1_NAME~ Bonus: ~2_val~%
        list.add_str(1072394, &format!("#{skill}\t{}", d.success_bonus));
    }
    muo_content::aos::add_skill_bonus_properties(&d.skill_bonuses, list, muo_content::core::aos(w));
    muo_content::aos::add_attribute_properties(
        &d.attributes,
        list,
        muo_content::core::aos(w),
        muo_content::core::ml(w),
        0,
        0,
        0,
    );
    if d.max_charges > 0 {
        list.add_int(1060741, d.charges); // charges: ~1_val~
    }
    if d.slayer != slayer_name::NONE {
        list.add(1072503 + d.slayer);
    }
}

/// `{_killer.Name ?? "Unknown"}`: a `TextDefinition` prints its number as `#n` or its string.
fn attr_name(a: &TalismanAttr) -> String {
    match &a.name {
        None => "Unknown".to_string(),
        Some(n) if n.number > 0 => format!("#{}", n.number),
        Some(n) => n.string.clone().unwrap_or_default(),
    }
}

/// `[AfterDeserialization]`.
fn after_deserialization(w: &mut World, id: ItemId) {
    if let Some(EntityId::Mobile(m)) = w.item(id).and_then(|i| i.parent) {
        add_stat_bonuses(w, id, m);
        skill_bonuses_add_to(w, id, m);
        if data(w, id).is_some_and(|d| d.charge_time > 0) {
            start_timer(w, id);
        }
    }
}

// ---- double click ----

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if muo_server::mobile::part1::props_b::talisman(w, from) != Some(id) {
        deps::send_localized_message(w, from, 502641); // You must equip this item to use it.
        return;
    }
    let Some(d) = data(w, id).cloned() else {
        return;
    };
    if d.charge_time > 0 {
        // You must wait ~1_val~ seconds for this to recharge.
        muo_server::mobile::part2::messages::send_localized_message(
            w,
            from,
            1074882,
            &d.charge_time.to_string(),
            0x3B2,
        );
        return;
    }
    if d.charges == 0 && d.max_charges > 0 {
        deps::send_localized_message(w, from, 1042544); // This item is out of charges.
        return;
    }

    let mut type_name = (class(w, id).get_summoner)(w, id).map(str::to_string);
    if !d.summoner.is_empty() {
        type_name = d.summoner.type_name.clone();
    }
    if let Some(type_name) = type_name {
        // The early `return`s (no room, too many followers) skip `OnAfterUse` and the removal target too.
        if !summon_or_give(w, id, from, &type_name, &d) {
            return;
        }
        on_after_use(w, id);
    }

    if d.removal != removal::NONE {
        let mut t = target::Target::new(w, 12, false, TargetFlags::BENEFICIAL);
        t.handlers.on_target = Some(std::rc::Rc::new(move |w: &mut World, from, targeted| {
            talisman_on_target(w, id, from, targeted)
        }));
        let tid = target::add_target(w, t);
        target::set_mobile_target(w, from, Some(tid));
    }
}

/// The body of `OnDoubleClick` once a `type` is known: an item goes to the backpack, a creature is summoned.
fn summon_or_give(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    type_name: &str,
    d: &TalismanData,
) -> bool {
    // `false`: returned before `OnAfterUse`.
    match w.types.find_type_by_full_name(type_name, false) {
        Some(EntityType::Item(_)) => {
            let Some(mut item) = muo_content::factory::make_item(w, type_name) else {
                return true; // `entity = null` after the exception
            };
            let mut count = 1;
            if d.summoner.amount > 1 {
                if w.item(item).is_some_and(|i| i.stackable()) {
                    ops::set_amount(w, item, d.summoner.amount);
                } else {
                    count = d.summoner.amount;
                }
            }
            let pack = muo_server::mobile::part1::props_b::backpack(w, from);
            let room = pack.is_some_and(|p| {
                let weight = ops::weight(w, item);
                f64::from(count) * weight
                    <= f64::from(muo_server::item::container::max_weight(w, p))
                    && w.item(p).map_or(0, |i| i.items.len()) as i32 + count
                        <= muo_server::item::container::max_items(w, p)
            });
            if !room {
                deps::send_localized_message(w, from, 500720); // You don't have enough room in your backpack!
                if let Some(ty) = w.item(item).map(|i| i.ty) {
                    (ty.delete)(w, item);
                }
                return false;
            }
            for i in 0..count {
                muo_server::mobile::part2::items::place_in_backpack(w, from, item);
                if i + 1 < count {
                    match muo_content::factory::make_item(w, type_name) {
                        Some(next) => item = next,
                        None => break,
                    }
                }
            }
            let ty = w.item(item).map(|i| i.ty);
            if ty.is_some_and(|t| muo_content::items::is_named(t, "Server.Items.Board")) {
                deps::send_localized_message(w, from, 1075000); // You have been given some wooden boards.
            } else if ty.is_some_and(|t| muo_content::items::is_named(t, "Server.Items.IronIngot"))
            {
                deps::send_localized_message(w, from, 1075001); // You have been given some ingots.
            } else if ty.is_some_and(|t| muo_content::items::is_named(t, "Server.Items.Bandage")) {
                deps::send_localized_message(w, from, 1075002); // You have been given some clean bandages.
            } else if let Some(name) = &d.summoner.name {
                // You have been given ~1_name~
                let arg = if name.number > 0 {
                    format!("##{}", name.number) // `$"#{_summoner.Name}"`: the TextDefinition prints `#n` itself
                } else {
                    name.string.clone().unwrap_or_default()
                };
                muo_server::mobile::part2::messages::send_localized_message(
                    w, from, 1074853, &arg, 0x3B2,
                );
            }
        }
        Some(EntityType::Mobile(_)) => {
            let Some(mob) = muo_content::factory::make_mobile(w, type_name) else {
                return true;
            };
            let alive = data(w, id)
                .and_then(|d| d.creature)
                .filter(|&c| w.mobile(c).is_some_and(|m| !m.deleted));
            if alive.is_some() {
                deps::send_localized_message(w, from, 1074270); // You have too many followers to summon another one.
                if let Some(ty) = w.mobile(mob).map(|m| m.ty) {
                    (ty.delete)(w, mob);
                }
                return false;
            }
            let (loc, sound) = (
                w.mobile(from).map(|m| m.location).unwrap_or_default(),
                w.mobile(mob).map_or(0, |m| m.base_sound_id),
            );
            if muo_content::creature::control::summon(
                w,
                mob,
                true,
                from,
                loc,
                sound,
                TimeSpan::from_secs(600),
                None,
            ) {
                let (mloc, mmap) = w
                    .mobile(mob)
                    .map(|m| (m.location, m.map))
                    .unwrap_or_default();
                let fx = effect_item::create(w, mloc, mmap, effect_item::DEFAULT_DURATION);
                muo_server::effects::send_location_particles(w, fx, 0x3728, 1, 10, 0x26B6);
                muo_content::creature::control::set_summoned(w, mob, false);
                muo_content::creature::control::issue_order(
                    w,
                    mob,
                    muo_content::creature::enums::OrderType::Follow,
                    None,
                    Some(from),
                );
                if let Some(d) = data_mut(w, id) {
                    d.creature = Some(mob);
                }
            }
        }
        None => {} // `entity?.Delete()` of a null
    }
    true
}

/// `TalismanTarget.OnTarget`.
fn talisman_on_target(w: &mut World, id: ItemId, from: MobileId, o: Targeted) {
    if ops::is_deleted(w, id) {
        return;
    }
    if muo_server::mobile::part1::props_b::talisman(w, from) != Some(id) {
        deps::send_localized_message(w, from, 502641); // You must equip this item to use it.
        return;
    }
    let Targeted::Entity(EntityId::Mobile(target)) = o else {
        deps::send_localized_message(w, from, 1046439); // That is not a valid target.
        return;
    };
    let Some(d) = data(w, id).cloned() else {
        return;
    };
    if d.charge_time > 0 {
        muo_server::mobile::part2::messages::send_localized_message(
            w,
            from,
            1074882,
            &d.charge_time.to_string(),
            0x3B2,
        );
        return;
    }
    if d.charges == 0 && d.max_charges > 0 {
        deps::send_localized_message(w, from, 1042544); // This item is out of charges.
        return;
    }
    match d.removal {
        removal::CURSE => remove_curse(w, from, target),
        removal::DAMAGE => {
            muo_server::mobile::part2::effects::play_sound(w, target, 0x201);
            location_particles_at(w, target, 0x3728, 1, 13, 0x834, 0x13B2);
            end_effect(w, "BleedAttack.EndBleed", target);
            end_effect(w, "MortalStrike.EndWound", target);
            remove_buffs(w, target, &["Bleed", "MortalStrike"]);
            deps::send_localized_message(w, target, 1072405); // Your lasting damage effects have been removed!
            if target != from {
                deps::send_localized_message(w, from, 1072406); // Your Targets lasting damage effects have been removed!
            }
        }
        removal::WARD => {
            muo_server::mobile::part2::effects::play_sound(w, target, 0x201);
            location_particles_at(w, target, 0x3728, 1, 13, 0x834, 0x13B2);
            end_effect(w, "MagicReflectSpell.EndReflect", target);
            end_effect(w, "ReactiveArmorSpell.EndArmor", target);
            end_effect(w, "ProtectionSpell.EndProtection", target);
            deps::send_localized_message(w, target, 1072402); // Your wards have been removed!
            if target != from {
                deps::send_localized_message(w, from, 1072403); // Your target's wards have been removed!
            }
        }
        _ => {} // Wildfire: `// TODO`
    }
    on_after_use(w, id);
}

/// `Effects.SendLocationParticles(EffectItem.Create(target.Location, target.Map, DefaultDuration), item, speed,
/// duration, hue, 0, effect, 0)`.
fn location_particles_at(
    w: &mut World,
    target: MobileId,
    item_id: i32,
    speed: i32,
    duration: i32,
    hue: i32,
    effect: i32,
) {
    let Some((loc, map)) = w.mobile(target).map(|m| (m.location, m.map)) else {
        return;
    };
    let fx = effect_item::create(w, loc, map, effect_item::DEFAULT_DURATION);
    muo_server::effects::send_location_particles_full(
        w, fx, item_id, speed, duration, hue, 0, effect, 0,
    );
}

fn remove_curse(w: &mut World, from: MobileId, target: MobileId) {
    muo_server::mobile::part2::effects::play_sound(w, target, 0xF6);
    muo_server::mobile::part2::effects::play_sound(w, target, 0x1F7);
    muo_server::mobile::part2::effects::fixed_particles_hue(
        w,
        target,
        0x3709,
        1,
        30,
        9963,
        13,
        3,
        muo_server::effects::EffectLayer::Head,
    );
    if let (Some((loc, _)), Some(map)) = (
        w.mobile(target).map(|m| (m.location, ())),
        w.mobile(from).and_then(|m| m.map),
    ) {
        let a = muo_primitives::Point3D::new(loc.x, loc.y, loc.z - 10);
        let b = muo_primitives::Point3D::new(loc.x, loc.y, loc.z + 50);
        muo_server::effects::send_moving_particles_layer(
            w,
            muo_server::effects::Targeted::point(a, Some(map)),
            muo_server::effects::Targeted::point(b, Some(map)),
            0x2255,
            1,
            0,
            false,
            false,
            13,
            3,
            9501,
            1,
            0,
            i32::from(muo_server::effects::EffectLayer::Head.0),
            0x100,
        );
    }
    for name in [
        "[Magic] Str Curse",
        "[Magic] Dex Curse",
        "[Magic] Int Curse",
    ] {
        let curse = w.mobile(target).map(|m| m.ty).and_then(|ty| {
            (ty.get_stat_mod)(w, target, name, false)
                .and_then(|i| w.mobile(target).map(|m| m.stat_mods[i].offset))
        });
        if curse.is_some_and(|o| o < 0)
            && let Some(ty) = w.mobile(target).map(|m| m.ty)
        {
            (ty.remove_stat_mod_by_name)(w, target, name);
        }
    }
    if let Some(ty) = w.mobile(target).map(|m| m.ty) {
        (ty.set_paralyzed)(w, target, false);
    }
    end_effect(w, "EvilOmenSpell.EndEffect", target);
    end_effect(w, "StrangleSpell.RemoveCurse", target);
    end_effect(w, "CorpseSkinSpell.RemoveCurse", target);
    end_effect(w, "CurseSpell.RemoveEffect", target);
    remove_buffs(w, target, &["Clumsy", "FeebleMind", "Weaken", "MassCurse"]);
    deps::send_localized_message(w, target, 1072408); // Any curses on you have been lifted
    if target != from {
        deps::send_localized_message(w, from, 1072409); // Your targets curses have been lifted
    }
}

/// `if (target is PlayerMobile pm) pm.RemoveBuff(icon)` for each icon.
fn remove_buffs(w: &mut World, target: MobileId, icons: &[&str]) {
    if !w.mobile(target).is_some_and(|m| m.player) {
        return;
    }
    for name in icons {
        let icon = match *name {
            "Clumsy" => muo_content::player_mobile::buff_icon::BuffIcon::Clumsy,
            "FeebleMind" => muo_content::player_mobile::buff_icon::BuffIcon::FeebleMind,
            "Weaken" => muo_content::player_mobile::buff_icon::BuffIcon::Weaken,
            "MassCurse" => muo_content::player_mobile::buff_icon::BuffIcon::MassCurse,
            "Bleed" => muo_content::player_mobile::buff_icon::BuffIcon::Bleed,
            "MortalStrike" => muo_content::player_mobile::buff_icon::BuffIcon::MortalStrike,
            _ => continue,
        };
        muo_content::player_mobile::buff::remove_buff(w, target, icon);
    }
}

// ---- slayers and the combat hooks ----

/// A mobile class is `name` or derives from it (`Type.IsAssignableFrom`).
fn mobile_is_named(ty: &'static MobileType, name: &str) -> bool {
    let mut t = Some(ty);
    while let Some(c) = t {
        if c.full_name == name {
            return true;
        }
        t = c.parent;
    }
    false
}

/// `TalismanSlayer.Slays(name, m)`.
pub fn slays(w: &World, name: i32, m: MobileId) -> bool {
    let Some(ty) = w.mobile(m).map(|m| m.ty) else {
        return false;
    };
    let Some((_, types)) = usize::try_from(name - 1)
        .ok()
        .and_then(|i| SLAYER_TABLE.get(i))
    else {
        return false;
    };
    types.iter().any(|t| mobile_is_named(ty, t))
}

/// `TalismanAttribute.DamageBonus(to) => to?.GetType() == _type ? _amount : 0`.
pub fn damage_bonus(w: &World, a: &TalismanAttr, to: MobileId) -> i32 {
    let (Some(t), Some(ty)) = (a.type_name.as_deref(), w.mobile(to).map(|m| m.ty)) else {
        return 0;
    };
    if ty.full_name == t { a.amount } else { 0 }
}

fn killer_bonus(w: &World, attacker: MobileId, defender: MobileId) -> i32 {
    talisman_of(w, attacker)
        .and_then(|t| data(w, t))
        .map_or(0, |d| damage_bonus(w, &d.killer, defender))
}

fn talisman_slays(w: &World, attacker: MobileId, defender: MobileId) -> bool {
    talisman_of(w, attacker)
        .and_then(|t| data(w, t))
        .is_some_and(|d| slays(w, d.slayer, defender))
}

/// `t.Protection?.Type?.IsInstanceOfType(from)`: the protection percent.
fn talisman_protection(w: &World, m: MobileId, from: MobileId) -> i32 {
    let Some(d) = talisman_of(w, m).and_then(|t| data(w, t)) else {
        return 0;
    };
    let (Some(t), Some(ty)) = (
        d.protection.type_name.as_deref(),
        w.mobile(from).map(|m| m.ty),
    ) else {
        return 0;
    };
    if mobile_is_named(ty, t) {
        d.protection.amount
    } else {
        0
    }
}

fn configure(w: &mut World) {
    use muo_spells::{
        fifth::magic_reflect, first::reactive_armor, fourth::curse, necromancy, second::protection,
    };
    w.resources.get_mut::<TalismanHooks>().end_effect = Some(|w, kind, m| match kind {
        "EvilOmenSpell.EndEffect" => {
            necromancy::evil_omen::end_effect(w, m);
        }
        "StrangleSpell.RemoveCurse" => {
            necromancy::strangle::remove_curse(w, m);
        }
        "CorpseSkinSpell.RemoveCurse" => {
            necromancy::corpse_skin::remove_curse(w, m);
        }
        "CurseSpell.RemoveEffect" => {
            curse::remove_effect(w, m);
        }
        "BleedAttack.EndBleed" => muo_content::items::weapon_ability::end_bleed(w, m, true),
        "MortalStrike.EndWound" => {
            muo_content::items::weapon_ability::end_wound(w, m);
        }
        "MagicReflectSpell.EndReflect" => magic_reflect::end_reflect(w, m),
        "ReactiveArmorSpell.EndArmor" => reactive_armor::end_armor(w, m),
        "ProtectionSpell.EndProtection" => protection::end_protection(w, m),
        _ => {}
    });
    let h = w.resources.get_mut::<WeaponHooks>();
    h.talisman_killer_bonus = Some(killer_bonus);
    h.talisman_slays = Some(talisman_slays);
    h.talisman_slayer_bovine_slays = Some(|w, defender| slays(w, slayer_name::BOVINE, defender));
    w.resources.get_mut::<SystemHooks>().talisman_protection = Some(talisman_protection);
    // Usage: RandomTalisman <count>; Description: Generates random talismans in your backpack.
    muo_server::commands::CommandSystem::register(
        w,
        "RandomTalisman",
        muo_server::mobile::AccessLevel::GameMaster,
        &[],
        std::rc::Rc::new(random_talisman_command),
    );
}

/// `RandomTalisman_OnCommand`: `count` of `Loot.RandomTalisman()` into the backpack.
fn random_talisman_command(w: &mut World, e: &mut muo_server::commands::CommandEventArgs) {
    for _ in 0..e.get_int32(0) {
        if let Some(t) = muo_content::loot::random_talisman(w) {
            muo_server::mobile::part2::items::add_to_backpack(w, e.mobile, t);
        }
    }
}

// ---- random properties ----

/// `GetRandomSummoner(owner)`.
pub fn random_summoner(w: &mut World) -> TalismanAttr {
    if utility::random_double(&mut w.rng) < 0.975 {
        return TalismanAttr::default();
    }
    let num = utility::random(&mut w.rng, SUMMONS.len() as i32) as usize;
    let amount = if num > 14 { 10 } else { 0 };
    attribute(
        SUMMONS[num],
        TextDefinition::of_number(SUMMON_LABELS[num]),
        amount,
    )
}

/// `GetRandomRemoval()`.
pub fn random_removal(w: &mut World) -> i32 {
    if utility::random_double(&mut w.rng) < 0.65 {
        const LIST: [i32; 3] = [390, 404, 407];
        return LIST[utility::random(&mut w.rng, 3) as usize];
    }
    removal::NONE
}

/// `GetRandomKiller(owner, includingNone)`.
pub fn random_killer(w: &mut World, including_none: bool) -> TalismanAttr {
    if including_none && utility::random_bool(&mut w.rng) {
        return TalismanAttr::default();
    }
    let num = utility::random(&mut w.rng, KILLERS.len() as i32) as usize;
    let amount = utility::random_min_max(&mut w.rng, 10, 100);
    attribute(
        KILLERS[num],
        TextDefinition::of_number(KILLER_LABELS[num]),
        amount,
    )
}

/// `GetRandomProtection(owner, includingNone)`.
pub fn random_protection(w: &mut World, including_none: bool) -> TalismanAttr {
    if including_none && utility::random_bool(&mut w.rng) {
        return TalismanAttr::default();
    }
    let num = utility::random(&mut w.rng, KILLERS.len() as i32) as usize;
    let amount = utility::random_min_max(&mut w.rng, 5, 60);
    attribute(
        KILLERS[num],
        TextDefinition::of_number(KILLER_LABELS[num]),
        amount,
    )
}

/// `GetRandomSkill()`.
pub fn random_skill(w: &mut World) -> i32 {
    SKILLS[utility::random(&mut w.rng, SKILLS.len() as i32) as usize] as i32
}

fn random_bonus(w: &mut World, chance: f64) -> i32 {
    if utility::random_double(&mut w.rng) < chance {
        let r = utility::random_min_max(&mut w.rng, 7, 403);
        let num = 40.0 - f64::from(r).ln() * 5.0;
        return num.round_ties_even() as i32; // Math.Round: banker's rounding
    }
    0
}

/// `GetRandomExceptional()`.
pub fn random_exceptional(w: &mut World) -> i32 {
    random_bonus(w, 0.3)
}

/// `GetRandomSuccessful()`.
pub fn random_successful(w: &mut World) -> i32 {
    random_bonus(w, 0.75)
}

/// `GetRandomBlessed()`.
pub fn random_blessed(w: &mut World) -> bool {
    utility::random_double(&mut w.rng) < 0.02
}

/// `GetRandomSlayer()`.
pub fn random_slayer(w: &mut World) -> i32 {
    if utility::random_double(&mut w.rng) < 0.01 {
        utility::random_min_max(&mut w.rng, 1, 9)
    } else {
        slayer_name::NONE
    }
}

/// `GetRandomCharges()`.
pub fn random_charges(w: &mut World) -> i32 {
    if utility::random_bool(&mut w.rng) {
        utility::random_min_max(&mut w.rng, 10, 50)
    } else {
        0
    }
}

// ---- save layout (BaseTalisman version 1) ----

mod save_flag {
    pub const ATTRIBUTES: u32 = 0x1;
    pub const SKILL_BONUSES: u32 = 0x2;
    pub const PROTECTION: u32 = 0x4;
    pub const KILLER: u32 = 0x8;
    pub const SUMMONER: u32 = 0x10;
    pub const REMOVAL: u32 = 0x20;
    pub const SKILL: u32 = 0x40;
    pub const SUCCESS_BONUS: u32 = 0x80;
    pub const EXCEPTIONAL_BONUS: u32 = 0x100;
    pub const MAX_CHARGES: u32 = 0x200;
    pub const CHARGES: u32 = 0x400;
    pub const MAX_CHARGE_TIME: u32 = 0x800;
    pub const CHARGE_TIME: u32 = 0x1000;
    pub const BLESSED: u32 = 0x2000;
    pub const SLAYER: u32 = 0x4000;
}

/// `TalismanAttribute.Serialize` (`[SerializationGenerator(1, false)]`).
fn write_attr(a: &TalismanAttr, out: &mut dyn GenericWriter) {
    out.write_i32(1);
    out.write_type(a.type_name.as_deref());
    out.write_text_definition(a.name.as_ref());
    out.write_i32(a.amount);
}

/// `TalismanAttribute.Deserialize` (version 1; older records: flags, then each present field, see the
/// migration).
fn read_attr(w: &World, r: &mut dyn GenericReader) -> Result<TalismanAttr> {
    let version = r.read_i32()?;
    let mut a = TalismanAttr::default();
    if version < 1 {
        let flags = r.read_encoded_int()?;
        if flags & 0x1 != 0 {
            let n = r.read_string()?.unwrap_or_default();
            a.type_name = w
                .types
                .find_type_by_full_name(&n, false)
                .map(|t| t.full_name().to_string());
        }
        if flags & 0x2 != 0 {
            a.name = r.read_text_definition()?;
        }
        if flags & 0x4 != 0 {
            a.amount = r.read_encoded_int()?;
        }
    } else {
        a.type_name = ReadEntityExt::read_type(r, w)?.map(|t| t.full_name().to_string());
        a.name = r.read_text_definition()?;
        a.amount = r.read_i32()?;
    }
    Ok(a)
}

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(1);
    let Some(d) = data(w, id) else {
        out.write_enum(4, 0);
        return;
    };
    let mut flags = 0;
    for (on, f) in [
        (!d.attributes.is_empty(), save_flag::ATTRIBUTES),
        (!d.skill_bonuses.is_empty(), save_flag::SKILL_BONUSES),
        (!d.protection.is_empty(), save_flag::PROTECTION),
        (!d.killer.is_empty(), save_flag::KILLER),
        (!d.summoner.is_empty(), save_flag::SUMMONER),
        (d.removal != removal::NONE, save_flag::REMOVAL),
        (d.skill != 0, save_flag::SKILL),
        (d.success_bonus != 0, save_flag::SUCCESS_BONUS),
        (d.exceptional_bonus != 0, save_flag::EXCEPTIONAL_BONUS),
        (d.max_charges != 0, save_flag::MAX_CHARGES),
        (d.charges != 0, save_flag::CHARGES),
        (d.max_charge_time != 0, save_flag::MAX_CHARGE_TIME),
        (d.charge_time != 0, save_flag::CHARGE_TIME),
        (d.blessed, save_flag::BLESSED),
        (d.slayer != slayer_name::NONE, save_flag::SLAYER),
    ] {
        if on {
            flags |= f;
        }
    }
    out.write_enum(4, i64::from(flags));
    if flags & save_flag::ATTRIBUTES != 0 {
        d.attributes.serialize(out);
    }
    if flags & save_flag::SKILL_BONUSES != 0 {
        d.skill_bonuses.serialize(out);
    }
    if flags & save_flag::PROTECTION != 0 {
        write_attr(&d.protection, out);
    }
    if flags & save_flag::KILLER != 0 {
        write_attr(&d.killer, out);
    }
    if flags & save_flag::SUMMONER != 0 {
        write_attr(&d.summoner, out);
    }
    if flags & save_flag::REMOVAL != 0 {
        out.write_enum(4, i64::from(d.removal));
    }
    if flags & save_flag::SKILL != 0 {
        out.write_enum(4, i64::from(d.skill));
    }
    if flags & save_flag::SUCCESS_BONUS != 0 {
        out.write_encoded_int(d.success_bonus);
    }
    if flags & save_flag::EXCEPTIONAL_BONUS != 0 {
        out.write_encoded_int(d.exceptional_bonus);
    }
    if flags & save_flag::MAX_CHARGES != 0 {
        out.write_encoded_int(d.max_charges);
    }
    if flags & save_flag::CHARGES != 0 {
        out.write_i32(d.charges);
    }
    if flags & save_flag::MAX_CHARGE_TIME != 0 {
        out.write_encoded_int(d.max_charge_time);
    }
    if flags & save_flag::CHARGE_TIME != 0 {
        out.write_encoded_int(d.charge_time);
    }
    if flags & save_flag::SLAYER != 0 {
        out.write_enum(4, i64::from(d.slayer));
    }
}

fn read_checked_enum(r: &mut dyn GenericReader, ok: fn(i32) -> bool, what: &str) -> Result<i32> {
    let v = r.read_enum(4)? as i32;
    if ok(v) {
        Ok(v)
    } else {
        Err(IoError::InvalidData(format!("{what}: no value {v}")))
    }
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = TalismanData::default();
    if version < 1 {
        read_old(w, id, r, &mut d)?;
    } else {
        let flags = r.read_enum(4)? as u32;
        if flags & save_flag::ATTRIBUTES != 0 {
            d.attributes = muo_content::aos::Attrs::deserialize(r)?;
        }
        if flags & save_flag::SKILL_BONUSES != 0 {
            d.skill_bonuses = muo_content::aos::Attrs::deserialize(r)?;
        }
        if flags & save_flag::PROTECTION != 0 {
            d.protection = read_attr(w, r)?;
        }
        if flags & save_flag::KILLER != 0 {
            d.killer = read_attr(w, r)?;
        }
        if flags & save_flag::SUMMONER != 0 {
            d.summoner = read_attr(w, r)?;
        }
        if flags & save_flag::REMOVAL != 0 {
            d.removal = read_checked_enum(r, removal::is_known, "TalismanRemoval")?;
        }
        if flags & save_flag::SKILL != 0 {
            d.skill = read_checked_enum(r, |v| SkillName::from_i32(v).is_some(), "SkillName")?;
        }
        if flags & save_flag::SUCCESS_BONUS != 0 {
            d.success_bonus = r.read_encoded_int()?;
        }
        if flags & save_flag::EXCEPTIONAL_BONUS != 0 {
            d.exceptional_bonus = r.read_encoded_int()?;
        }
        if flags & save_flag::MAX_CHARGES != 0 {
            d.max_charges = r.read_encoded_int()?;
        }
        if flags & save_flag::CHARGES != 0 {
            d.charges = r.read_i32()?;
        }
        if flags & save_flag::MAX_CHARGE_TIME != 0 {
            d.max_charge_time = r.read_encoded_int()?;
        }
        if flags & save_flag::CHARGE_TIME != 0 {
            d.charge_time = r.read_encoded_int()?;
        }
        d.blessed = flags & save_flag::BLESSED != 0;
        if flags & save_flag::SLAYER != 0 {
            d.slayer = read_checked_enum(r, |v| (0..=9).contains(&v), "TalismanSlayerName")?;
        }
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    after_deserialization(w, id);
    Ok(())
}

/// The legacy `Deserialize(reader, version)` of `BaseTalisman`: the pre-codegen layout (`OldSaveFlag`).
fn read_old(
    w: &mut World,
    id: ItemId,
    r: &mut dyn GenericReader,
    d: &mut TalismanData,
) -> Result<()> {
    let flags = r.read_encoded_int()? as u32;
    if flags & 0x1 != 0 {
        d.attributes = muo_content::aos::Attrs::deserialize(r)?;
    }
    if flags & 0x2 != 0 {
        d.skill_bonuses = muo_content::aos::Attrs::deserialize(r)?;
    }
    if flags & 0x4 != 0 {
        // Owner: `BlessedFor = reader.ReadEntity<Mobile>()`
        let owner = r.read_mobile(w)?;
        if let Some(i) = w.item_mut(id) {
            i.blessed_for = owner;
        }
    }
    if flags & 0x8 != 0 {
        d.protection = read_attr(w, r)?;
    }
    if flags & 0x10 != 0 {
        d.killer = read_attr(w, r)?;
    }
    if flags & 0x20 != 0 {
        d.summoner = read_attr(w, r)?;
    }
    if flags & 0x40 != 0 {
        d.removal = r.read_encoded_int()?;
    }
    if flags & 0x80 != 0 {
        // OldKarmaLoss: `Attributes.IncreasedKarmaLoss = reader.ReadEncodedInt()`
        let v = r.read_encoded_int()?;
        d.attributes
            .set_raw(muo_content::aos::AosAttribute::IncreasedKarmaLoss as u32, v);
    }
    if flags & 0x100 != 0 {
        d.skill = r.read_encoded_int()?;
    }
    if flags & 0x200 != 0 {
        d.success_bonus = r.read_encoded_int()?;
    }
    if flags & 0x400 != 0 {
        d.exceptional_bonus = r.read_encoded_int()?;
    }
    if flags & 0x800 != 0 {
        d.max_charges = r.read_encoded_int()?;
    }
    if flags & 0x1000 != 0 {
        d.charges = r.read_encoded_int()?;
    }
    if flags & 0x2000 != 0 {
        d.max_charge_time = r.read_encoded_int()?;
    }
    if flags & 0x4000 != 0 {
        d.charge_time = r.read_encoded_int()?;
    }
    if flags & 0x1_0000 != 0 {
        d.slayer = r.read_encoded_int()?;
    }
    d.blessed = flags & 0x8000 != 0;
    Ok(())
}

/// `Server.Items.BaseTalisman` (`[SerializationGenerator(1, false)]`; no `[Constructible]` constructor).
pub static BASE_TALISMAN: ItemType = ItemType {
    full_name: "Server.Items.BaseTalisman",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_TALISMAN_CLASS),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1071023, // Talisman
    on_after_duped,
    can_equip,
    on_added,
    on_removed,
    on_double_click,
    add_name_property,
    get_properties,
    on_delete,
    serialize,
    deserialize,
    ..ITEM_BASE
};

/// `Server.Items.RandomTalisman` (`[SerializationGenerator(0, false)]`).
pub static RANDOM_TALISMAN: ItemType = ItemType {
    full_name: "Server.Items.RandomTalisman",
    parent: Some(&BASE_TALISMAN),
    construct: Some(|w| random_talisman_new(w, &RANDOM_TALISMAN)),
    serialize: |w, id, out| {
        (BASE_TALISMAN.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_TALISMAN.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_TALISMAN
};

/// `RandomTalisman()`.
pub fn random_talisman_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let gid = random_item_id(w);
    let id = new_talisman(w, ty, gid);
    let summoner = random_summoner(w);
    let summoner_empty = summoner.is_empty();
    let is_item = attr_is_item(w, &summoner);
    set_summoner(w, id, summoner);
    if summoner_empty {
        let r = random_removal(w);
        set_removal(w, id, r);
        if r != removal::NONE {
            let c = random_charges(w);
            set_max_charges(w, id, c);
            set_max_charge_time(w, id, 1200);
        }
    } else {
        let c = utility::random_min_max(&mut w.rng, 10, 50);
        set_max_charges(w, id, c);
        set_max_charge_time(w, id, if is_item { 60 } else { 1800 });
    }
    let blessed = random_blessed(w);
    set_blessed(w, id, blessed);
    let slayer = random_slayer(w);
    set_slayer(w, id, slayer);
    let p = random_protection(w, true);
    set_protection(w, id, p);
    let k = random_killer(w, true);
    set_killer(w, id, k);
    let s = random_skill(w);
    set_skill(w, id, s);
    let e = random_exceptional(w);
    set_exceptional_bonus(w, id, e);
    let sb = random_successful(w);
    set_success_bonus(w, id, sb);
    let max = data(w, id).map_or(0, |d| d.max_charges);
    set_charges(w, id, max);
    id
}

/// Registers the classes of this directory.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BASE_TALISMAN, &[]);
    types.register_item(&RANDOM_TALISMAN, &[]);
    items::register(types);
    summons::register(types);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BaseTalisman.Configure",
        configure,
    );
}
