//! `DyeTub` and its eleven subclasses, `Dyes` and `Scissors`.
//! State: [`TubData`] in `Item::ext`; what a tub class overrides is its [`TubClass`]. [`configure`] sets
//! `EquipHooks::dye_tub` (the tub's `DyedHue` and `FailMessage` that `IDyable.Dye` reads).
//!
//! `CustomHuePicker`/`MetallicHuePicker` are `hue_pickers.rs`. `FurnitureAttribute.Check(item)` is [`DyeHooks::is_furniture`] (unset: nothing is furniture, so only a
//! PotionKeg can be painted by a furniture tub). The scissors target has no PlagueBeast branch.

use std::rc::Rc;

use muo_content::items::armor;
use muo_content::items::equip_hooks;
use muo_content::items::interfaces;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::utility;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::hue_picker::HuePicker;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, base, deps, ops};
use muo_server::mobile::part2::effects;
use muo_server::property_list::PropertyList;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::hue_pickers::{self, CustomHuePicker};
use super::{hue, read_version, write_version};

/// `DyeTub`' fields.
#[derive(Clone, Copy, Debug)]
pub struct TubData {
    pub level: i32,
    pub redyable: bool,
    pub dyed_hue: i32,
    /// `IRewardItem.IsRewardItem` of the subclasses that have it.
    pub is_reward: bool,
}

/// The virtuals of a tub class.
#[derive(Copy, Clone)]
pub struct TubClass {
    pub allow_runebooks: bool,
    pub allow_furniture: bool,
    pub allow_statuettes: bool,
    pub allow_leather: bool,
    pub allow_dyables: bool,
    pub metallic: bool,
    pub target_message: i32,
    pub fail_message: i32,
    /// `Redyable` is overridden to `false` (the white tubs).
    pub never_redyable: bool,
    /// The reward year cliloc of `GetProperties` (ML and later), 0 for none.
    pub year: i32,
    /// The class has `IRewardItem` (its own version-1 record).
    pub reward: bool,
    /// `CustomHuePicker`.
    pub picker: Option<&'static CustomHuePicker>,
}

pub static DYE_TUB_CLASS: TubClass = TubClass {
    allow_runebooks: false,
    allow_furniture: false,
    allow_statuettes: false,
    allow_leather: false,
    allow_dyables: true,
    metallic: false,
    target_message: 500859, // Select the clothing to dye.
    fail_message: 1042083,  // You can not dye that.
    never_redyable: false,
    year: 0,
    reward: false,
    picker: None,
};

/// What other crates answer for `FurnitureAttribute.Check`.
#[derive(Default)]
pub struct DyeHooks {
    pub is_furniture: Option<fn(&World, ItemId) -> bool>,
}

pub fn class(w: &World, id: ItemId) -> &'static TubClass {
    w.item(id)
        .and_then(|i| i.ty.family::<TubClass>())
        .unwrap_or(&DYE_TUB_CLASS)
}

pub fn data(w: &World, id: ItemId) -> Option<&TubData> {
    w.item(id)?.ext.get::<TubData>()
}

fn data_mut(w: &mut World, id: ItemId) -> Option<&mut TubData> {
    w.item_mut(id)?.ext.get_mut::<TubData>()
}

/// `item is DyeTub`.
pub fn is_tub(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&DYE_TUB))
}

/// `Redyable`.
pub fn redyable(w: &World, id: ItemId) -> bool {
    !class(w, id).never_redyable && data(w, id).is_some_and(|d| d.redyable)
}

/// `DyedHue = value`: ignored while not redyable; sets `Hue`.
pub fn set_dyed_hue(w: &mut World, id: ItemId, value: i32) {
    if !data(w, id).is_some_and(|d| d.redyable) {
        return;
    }
    if let Some(d) = data_mut(w, id) {
        d.dyed_hue = value;
    }
    hue(w, id, value);
}

/// `new DyeTub { DyedHue = hue, Redyable = false }` (the Test Center's tub).
pub fn new_unredyable_tub(w: &mut World, hue_value: i32) -> ItemId {
    let id = new_tub(w, &DYE_TUB);
    set_dyed_hue(w, id, hue_value);
    if let Some(d) = data_mut(w, id) {
        d.redyable = false;
    }
    id
}

/// `new DyeTub()` of class `ty`.
pub fn new_tub(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Items.DyeTub");
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(TubData {
            level: 1,
            redyable: true,
            dyed_hue: 0,
            is_reward: false,
        });
    }
    id
}

fn get_context_menu_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<muo_server::context_menu_entry::ContextMenuEntry>,
) {
    base::get_context_menu_entries_base(w, id, from, list);
    if let Some(e) = muo_multis::secure_entry::add_to(w, from, id) {
        list.push(e);
    }
}

fn near(w: &World, a: ItemId, b: ItemId, from: MobileId) -> bool {
    let (la, lb) = (ops::get_world_location(w, a), ops::get_world_location(w, b));
    deps::mobile_in_range(w, from, la, 1) && deps::mobile_in_range(w, from, lb, 1)
}

fn on_double_click(w: &mut World, tub: ItemId, from: MobileId) {
    let c = class(w, tub);
    if c.reward
        && data(w, tub).is_some_and(|d| d.is_reward)
        && let Some(f) = w
            .resources
            .get::<crate::clothing::special::RewardHooks>()
            .and_then(|h| h.check_is_usable_by)
        && !f(w, from, tub)
    {
        return;
    }
    let loc = ops::get_world_location(w, tub);
    if deps::mobile_in_range(w, from, loc, 1) {
        deps::send_localized_message(w, from, c.target_message);
        target::begin_target(w, from, 1, false, TargetFlags::NONE, move |w, from, t| {
            on_target(w, tub, from, t)
        });
    } else {
        deps::send_localized_message(w, from, 500446); // That is too far away.
    }
}

fn paint(w: &mut World, tub: ItemId, item: ItemId, from: MobileId) {
    let h = data(w, tub).map_or(0, |d| d.dyed_hue);
    hue(w, item, h);
    effects::play_sound(w, from, 0x23E);
}

fn on_target(w: &mut World, tub: ItemId, from: MobileId, t: Targeted) {
    let c = class(w, tub);
    let Targeted::Entity(EntityId::Item(item)) = t else {
        deps::send_localized_message(w, from, c.fail_message);
        return;
    };
    let is_named = |w: &World, n: &str| {
        w.item(item)
            .is_some_and(|i| muo_content::items::is_named(i.ty, n))
    };
    if w.item(item).is_some_and(|i| i.quest_item()) {
        deps::send_localized_message(w, from, 1151836); // You may not dye toggled quest items.
    } else if c.allow_dyables && interfaces::is_dyable(w, item) {
        if !near(w, tub, item, from) {
            deps::send_localized_message(w, from, 500446); // That is too far away.
        } else if matches!(
            w.item(item).and_then(|i| i.parent),
            Some(EntityId::Mobile(_))
        ) {
            deps::send_localized_message(w, from, 500861); // Can't Dye clothing that is being worn.
        } else if interfaces::dye(w, item, from, tub) == Some(true) {
            effects::play_sound(w, from, 0x23E);
        }
    } else if c.allow_furniture
        && (is_named(w, "Server.Items.PotionKeg")
            || w.resources
                .get::<DyeHooks>()
                .and_then(|h| h.is_furniture)
                .is_some_and(|f| f(w, item)))
    {
        if !near(w, tub, item, from) {
            deps::send_localized_message(w, from, 500446); // That is too far away.
            return;
        }
        let pack = w.mobile(from).and_then(|m| m.backpack);
        let mut okay = pack.is_some_and(|p| ops::is_child_of(w, item, Some(EntityId::Item(p))));
        if !okay {
            if w.item(item).is_some_and(|i| i.parent.is_none()) {
                let o = muo_content::regions::house::house_ops(w);
                let loc = ops::get_world_location(w, item);
                let map = w.item(item).and_then(|i| i.map);
                let height = ops::item_data(w, item).height as i32;
                let house = o.and_then(|o| o.find_house_at_point(w, loc, map, height));
                match (o, house) {
                    (Some(o), Some(h))
                        if o.has_locked_down_item(w, h, item) || o.has_secure_item(w, h, item) =>
                    {
                        if o.is_co_owner(w, h, from) || o.is_owner(w, h, from) {
                            okay = true;
                        } else {
                            deps::send_localized_message(w, from, 501023); // You must be the owner to use this item.
                        }
                    }
                    _ => deps::send_localized_message(w, from, 501022), // Furniture must be locked down to paint it.
                }
            } else {
                deps::send_localized_message(w, from, 1048135); // The furniture must be in your backpack to be painted.
            }
        }
        if okay {
            paint(w, tub, item, from);
        }
    } else if c.allow_runebooks
        && (is_named(w, "Server.Items.Runebook") || is_named(w, "Server.Items.RecallRune"))
    {
        if !near(w, tub, item, from) {
            deps::send_localized_message(w, from, 500446);
        } else if !w.item(item).is_some_and(|i| i.movable()) {
            deps::send_localized_message(w, from, 1049776); // You cannot dye runes or runebooks that are locked down.
        } else {
            paint(w, tub, item, from);
        }
    } else if c.allow_statuettes && is_named(w, "Server.Items.MonsterStatuette") {
        if !near(w, tub, item, from) {
            deps::send_localized_message(w, from, 500446);
        } else if !w.item(item).is_some_and(|i| i.movable()) {
            deps::send_localized_message(w, from, 1049779); // You cannot dye statuettes that are locked down.
        } else {
            paint(w, tub, item, from);
        }
    } else if c.allow_leather
        && (is_leather(w, item)
            || is_named(w, "Server.Items.ElvenBoots")
            || is_named(w, "Server.Items.WoodlandBelt"))
    {
        if !near(w, tub, item, from) {
            deps::send_localized_message(w, from, 500446);
        } else if !w.item(item).is_some_and(|i| i.movable()) {
            deps::send_localized_message(w, from, 1042419); // You may not dye leather items which are locked down.
        } else if matches!(
            w.item(item).and_then(|i| i.parent),
            Some(EntityId::Mobile(_))
        ) {
            deps::send_localized_message(w, from, 500861); // Can't Dye clothing that is being worn.
        } else {
            paint(w, tub, item, from);
        }
    } else {
        deps::send_localized_message(w, from, c.fail_message);
    }
}

/// `item is BaseArmor { MaterialType: Leather or Studded }`.
fn is_leather(w: &World, item: ItemId) -> bool {
    armor::is_armor(w, item)
        && matches!(
            armor::class(w, item).material_type,
            armor::ArmorMaterialType::Leather | armor::ArmorMaterialType::Studded
        )
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    base::get_properties_base(w, id, list);
    let c = class(w, id);
    if c.year != 0 && muo_content::core::ml(w) && data(w, id).is_some_and(|d| d.is_reward) {
        list.add(c.year);
    }
}

fn serialize(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(2);
    let d = data(w, id).copied().unwrap_or(TubData {
        level: 1,
        redyable: true,
        dyed_hue: 0,
        is_reward: false,
    });
    out.write_enum(4, i64::from(d.level));
    out.write_bool(d.redyable);
    out.write_i32(d.dyed_hue);
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = TubData {
        level: 1,
        redyable: true,
        dyed_hue: 0,
        is_reward: false,
    };
    if version >= 2 {
        d.level = r.read_enum(4)? as i32;
    } else if version == 1 {
        d.level = r.read_i32()?;
    }
    d.redyable = r.read_bool()?;
    d.dyed_hue = r.read_i32()?;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    Ok(())
}

/// `Server.Items.DyeTub`.
pub static DYE_TUB: ItemType = ItemType {
    full_name: "Server.Items.DyeTub",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_tub(w, &DYE_TUB)),
    family: Some(&DYE_TUB_CLASS),
    default_weight: crate::class_props::default_weight,
    get_context_menu_entries,
    get_properties,
    on_double_click,
    serialize,
    deserialize,
    ..ITEM_BASE
};

macro_rules! tubs {
    ($($name:ident, $init:ident = $full:literal : $parent:ident, label $label:expr, $class:ident { $($f:ident : $v:expr),* $(,)? }, reward $reward:expr, init $initfn:expr;)*) => {$(
        pub static $class: TubClass = TubClass { reward: $reward, $($f: $v,)* ..DYE_TUB_CLASS };
        #[doc = concat!("`new ", $full, "()`.")]
        pub fn $init(w: &mut World) -> ItemId {
            let id = new_tub(w, &$name);
            let init: fn(&mut World, ItemId) = $initfn;
            init(w, id);
            id
        }
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some($init),
            family: Some(&$class),
            label_number: |_, _| $label,
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                // A class under another reward tub (Metallic/White leather) has only its own plain version.
                if $reward && std::ptr::eq(&$parent, &DYE_TUB) {
                    out.write_i32(1);
                    out.write_bool(data(w, id).is_some_and(|d| d.is_reward));
                } else {
                    write_version(out, false);
                }
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                let version = r.read_i32()?;
                if $reward && std::ptr::eq(&$parent, &DYE_TUB) {
                    if version >= 1 {
                        let v = r.read_bool()?;
                        if let Some(d) = data_mut(w, id) {
                            d.is_reward = v;
                        }
                    } else if w.item(id).is_some_and(|i| i.loot_type == LootType::Regular) {
                        ops::set_loot_type(w, id, LootType::Blessed);
                    }
                }
                Ok(())
            },
            ..$parent
        };
    )*};
}

fn set_dyed(w: &mut World, id: ItemId, h: i32) {
    if let Some(d) = data_mut(w, id) {
        d.dyed_hue = h;
    }
    hue(w, id, h);
}
fn blessed(w: &mut World, id: ItemId) {
    ops::set_loot_type(w, id, LootType::Blessed);
}
fn no_redye(w: &mut World, id: ItemId) {
    if let Some(d) = data_mut(w, id) {
        d.redyable = false;
    }
}

tubs! {
    BLACK_DYE_TUB, new_black = "Server.Items.BlackDyeTub" : DYE_TUB, label 0, BLACK_CLASS {}, reward false,
        init |w, id| { set_dyed(w, id, 1); no_redye(w, id); };
    BLAZE_DYE_TUB, new_blaze = "Server.Items.BlazeDyeTub" : DYE_TUB, label 0, BLAZE_CLASS {}, reward false,
        init |w, id| { set_dyed(w, id, 0x489); no_redye(w, id); };
    FURNITURE_DYE_TUB, new_furniture = "Server.Items.FurnitureDyeTub" : DYE_TUB, label 1041246, FURNITURE_CLASS {
        allow_dyables: false, allow_furniture: true, target_message: 501019, fail_message: 501021, year: 1076217 }, reward true,
        init |w, id| blessed(w, id);
    LEATHER_DYE_TUB, new_leather = "Server.Items.LeatherDyeTub" : DYE_TUB, label 1041284, LEATHER_CLASS {
        allow_dyables: false, allow_leather: true, target_message: 1042416, fail_message: 1042418, year: 1076218, picker: Some(&hue_pickers::LEATHER_DYE_TUB) }, reward true,
        init |w, id| blessed(w, id);
    METALLIC_CLOTH_DYETUB, new_metallic_cloth = "Server.Items.MetallicClothDyetub" : DYE_TUB, label 1152920, METALLIC_CLOTH_CLASS {
        metallic: true }, reward false,
        init |w, id| blessed(w, id);
    METALLIC_LEATHER_DYE_TUB, new_metallic_leather = "Server.Items.MetallicLeatherDyeTub" : LEATHER_DYE_TUB, label 1153495, METALLIC_LEATHER_CLASS {
        allow_dyables: false, allow_leather: true, target_message: 1042416, fail_message: 1042418, metallic: true, year: 1076221 }, reward true,
        init |w, id| blessed(w, id);
    REWARD_BLACK_DYE_TUB, new_reward_black = "Server.Items.RewardBlackDyeTub" : DYE_TUB, label 1006008, REWARD_BLACK_CLASS {
        year: 1076217 }, reward true,
        init |w, id| { set_dyed(w, id, 1); no_redye(w, id); blessed(w, id); };
    RUNEBOOK_DYE_TUB, new_runebook_tub = "Server.Items.RunebookDyeTub" : DYE_TUB, label 1049740, RUNEBOOK_TUB_CLASS {
        allow_dyables: false, allow_runebooks: true, target_message: 1049774, fail_message: 1049775, year: 1076220, picker: Some(&hue_pickers::LEATHER_DYE_TUB) }, reward true,
        init |w, id| blessed(w, id);
    SPECIAL_DYE_TUB, new_special = "Server.Items.SpecialDyeTub" : DYE_TUB, label 1041285, SPECIAL_CLASS { year: 1076217, picker: Some(&hue_pickers::SPECIAL_DYE_TUB) }, reward true,
        init |w, id| blessed(w, id);
    STATUETTE_DYE_TUB, new_statuette_tub = "Server.Items.StatuetteDyeTub" : DYE_TUB, label 1049741, STATUETTE_TUB_CLASS {
        allow_dyables: false, allow_statuettes: true, target_message: 1049777, fail_message: 1049778, year: 1076221, picker: Some(&hue_pickers::LEATHER_DYE_TUB) }, reward true,
        init |w, id| blessed(w, id);
    WHITE_CLOTH_DYE_TUB, new_white_cloth = "Server.Items.WhiteClothDyeTub" : DYE_TUB, label 1149984, WHITE_CLOTH_CLASS {
        never_redyable: true }, reward false,
        init |w, id| { set_dyed(w, id, 0x9C2); };
    WHITE_LEATHER_DYE_TUB, new_white_leather = "Server.Items.WhiteLeatherDyeTub" : LEATHER_DYE_TUB, label 1149900, WHITE_LEATHER_CLASS {
        allow_dyables: false, allow_leather: true, target_message: 1042416, fail_message: 1042418, never_redyable: true, year: 1076218, picker: Some(&hue_pickers::LEATHER_DYE_TUB) }, reward false,
        init |w, id| { set_dyed(w, id, 0x9C2); blessed(w, id); };
}

/// Every tub class.
pub static ALL: &[&ItemType] = &[
    &DYE_TUB,
    &BLACK_DYE_TUB,
    &BLAZE_DYE_TUB,
    &FURNITURE_DYE_TUB,
    &LEATHER_DYE_TUB,
    &METALLIC_CLOTH_DYETUB,
    &METALLIC_LEATHER_DYE_TUB,
    &REWARD_BLACK_DYE_TUB,
    &RUNEBOOK_DYE_TUB,
    &SPECIAL_DYE_TUB,
    &STATUETTE_DYE_TUB,
    &WHITE_CLOTH_DYE_TUB,
    &WHITE_LEATHER_DYE_TUB,
];

// ---- Dyes ----

fn dyes_double_click(w: &mut World, _id: ItemId, from: MobileId) {
    deps::send_localized_message(w, from, 500856); // Select the dye tub to use the dyes on.
    target::begin_target(w, from, 1, false, TargetFlags::NONE, |w, from, t| {
        let Targeted::Entity(EntityId::Item(tub)) = t else {
            deps::send_localized_message(w, from, 500857); // Use this on a dye tub.
            return;
        };
        if !is_tub(w, tub) {
            deps::send_localized_message(w, from, 500857); // Use this on a dye tub.
        } else if redyable(w, tub) {
            let c = class(w, tub);
            let set: hue_pickers::HueCallback = Rc::new(move |w, _from, h| set_dyed_hue(w, tub, h));
            if c.metallic {
                // OSI has three metallic tubs now
                hue_pickers::send(w, from, hue_pickers::new_metallic(from, set));
            } else if let Some(def) = c.picker {
                hue_pickers::send(w, from, hue_pickers::new_custom(from, def, set));
            } else {
                let item_id = w.item(tub).map_or(0, |i| i.item_id);
                let mut picker = HuePicker::new(w, item_id);
                picker.on_response = Some(Rc::new(move |w: &mut World, h: i32| {
                    set_dyed_hue(w, tub, h)
                }));
                muo_server::mobile::part2::speech::send_hue_picker(w, from, picker);
            }
        } else if w.item(tub).is_some_and(|i| i.ty.is(&BLACK_DYE_TUB)) {
            deps::send_localized_message(w, from, 1010092); // You can not use this on a black dye tub.
        } else {
            muo_server::mobile::part2::messages::send_message(
                w,
                from,
                "That dye tub may not be redyed.",
            );
        }
    });
}

/// `Server.Items.Dyes`.
pub static DYES: ItemType = ItemType {
    full_name: "Server.Items.Dyes",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| crate::simple::from_template(w, &DYES, "Server.Items.Dyes")),
    default_weight: crate::class_props::default_weight,
    on_double_click: dyes_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- Scissors ----

/// `Scissors.CanScissor(from, obj)`.
pub fn can_scissor(w: &mut World, from: MobileId, item: ItemId) -> bool {
    if w.item(item)
        .is_some_and(|i| (i.ty.nontransferable)(w, item))
    {
        deps::send_localized_message(w, from, 502440); // Scissors can not be used on that to produce anything.
        return false;
    }
    true
}

/// `targeted is PlagueBeastInnard or PlagueBeastMutationCore`.
fn is_plague_beast_part(w: &World, item: ItemId) -> bool {
    w.item(item).is_some_and(|i| {
        muo_content::items::is_named(i.ty, "Server.Items.PlagueBeastInnard")
            || muo_content::items::is_named(i.ty, "Server.Items.PlagueBeastMutationCore")
    })
}

fn scissors_double_click(w: &mut World, id: ItemId, from: MobileId) {
    deps::send_localized_message(w, from, 502434); // What should I use these scissors on?
    let mut t = target::Target::new(w, 2, false, TargetFlags::NONE);
    t.handlers.on_target = Some(Rc::new(move |w, from, t| {
        if w.item(id).is_none_or(|i| i.deleted()) {
            return;
        }
        let aos = muo_content::core::aos(w);
        if aos && matches!(t, Targeted::Entity(EntityId::Mobile(m)) if m == from) {
            // "That doesn't seem like the smartest thing to do." / "That was an encounter you don't wish to repeat." / "Ha! You missed!"
            let n = utility::random(&mut w.rng, 3);
            deps::send_localized_message(w, from, 1062845 + n);
            return;
        }
        if muo_content::core::se(w)
            && utility::random_double(&mut w.rng) < 0.80
            && w.mobile(from).is_some_and(|m| m.direction.is_running())
            && w.tick_count - w.mobile(from).map_or(0, |m| m.last_move_time)
                < muo_server::mobile::part1::combat::compute_movement_speed_current(w, from) as i64
        {
            // Didn't your parents ever tell you not to run with scissors in your hand?!
            deps::send_localized_message(w, from, 1063305);
            return;
        }
        match t {
            Targeted::Entity(EntityId::Item(item))
                if interfaces::is_scissorable(w, item)
                    && (is_plague_beast_part(w, item)
                        || w.item(item).is_some_and(|i| i.movable())) =>
            {
                if can_scissor(w, from, item)
                    && interfaces::scissor(w, item, from, id) == Some(true)
                {
                    effects::play_sound(w, from, 0x248);
                }
            }
            _ => deps::send_localized_message(w, from, 502440), // Scissors can not be used on that to produce anything.
        }
    }));
    // OnNonlocalTarget: a plague beast's innards are cut from the beast; everything else is the base message.
    t.handlers.on_nonlocal_target = Some(Rc::new(move |w, from, t| {
        match t {
            Targeted::Entity(EntityId::Item(item)) if is_plague_beast_part(w, item) => {
                if can_scissor(w, from, item)
                    && interfaces::scissor(w, item, from, id) == Some(true)
                {
                    effects::play_sound(w, from, 0x248);
                }
            }
            _ => deps::send_localized_message(w, from, 500447), // That is not accessible.
        }
    }));
    let tid = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(tid));
}

/// `Server.Items.Scissors`.
pub static SCISSORS: ItemType = ItemType {
    full_name: "Server.Items.Scissors",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| crate::simple::from_template(w, &SCISSORS, "Server.Items.Scissors")),
    default_weight: crate::class_props::default_weight,
    on_double_click: scissors_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

const GM: muo_server::mobile::AccessLevel = muo_server::mobile::AccessLevel::GameMaster;

/// `[SerializedCommandProperty(GameMaster)]` `DyedHue` (`AllowDyedHueChange`: only while redyable) and `Redyable`.
static TUB_PROPS: &[muo_content::props::PropDef] = &[
    muo_content::props::PropDef {
        name: "DyedHue",
        kind: muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Numeric(
            muo_content::utilities::types::Numeric::Int,
        )),
        read: GM,
        write: GM,
        get: |w, e| match e {
            muo_server::ids::EntityId::Item(i) => {
                muo_content::props::Value::Int(i64::from(data(w, i).map_or(0, |d| d.dyed_hue)))
            }
            _ => muo_content::props::Value::Null,
        },
        set: Some(|w, e, v| {
            if let muo_server::ids::EntityId::Item(i) = e {
                set_dyed_hue(w, i, v.i32());
            }
            Ok(())
        }),
    },
    muo_content::props::PropDef {
        name: "Redyable",
        kind: muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Bool),
        read: GM,
        write: GM,
        get: |w, e| match e {
            muo_server::ids::EntityId::Item(i) => muo_content::props::Value::Bool(redyable(w, i)),
            _ => muo_content::props::Value::Null,
        },
        set: Some(|w, e, v| {
            if let muo_server::ids::EntityId::Item(i) = e
                && let Some(d) = data_mut(w, i)
            {
                d.redyable = v.bool();
            }
            Ok(())
        }),
    },
];

fn configure(w: &mut World) {
    muo_content::props::add(w, "Server.Items.DyeTub", TUB_PROPS);
    // `DyeTub : ISecurable`: the house's secure menu reads and sets its `Level`.
    let d = muo_content::items::house_deps::deps_mut(w);
    d.securables
        .push(|w, id| is_tub(w, id).then(|| data(w, id).map_or(0, |d| d.level) as u8));
    d.securable_setters.push(|w, id, level| {
        if !is_tub(w, id) {
            return false;
        }
        if let Some(d) = data_mut(w, id) {
            d.level = i32::from(level);
        }
        true
    });
    equip_hooks::hooks_mut(w).dye_tub = Some(|w, tub| {
        let c = class(w, tub);
        (data(w, tub).map_or(0, |d| d.dyed_hue), c.fail_message)
    });
}

/// Registers the tubs, dyes and scissors.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
    types.register_item(&DYES, &[]);
    types.register_item(&SCISSORS, &[]);
    types.register_phase_default(Phase::Configure, "Server.Items.DyeTub.Configure", configure);
}
