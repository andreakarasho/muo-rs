//! The base classes of item families (`Items/Weapons`, `Items/Armor`, ..) and what they share.
//!
//! How a family is built: one state struct per family in `Item::ext`
//! ([`weapon::WeaponData`], [`armor::ArmorData`], [`clothing::ClothingData`], [`jewel::JewelData`]), one `static
//! ItemType` per class, one family table of the constants a class's `override`s returned
//! ([`weapon::WeaponClass`], [`armor::ArmorClass`], [`clothing::ClothingClass`], [`jewel::JewelClass`]).

use muo_io::{IoError, Result};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::ops;
use muo_server::item::{Item, ItemType};
use muo_server::mobile::{MobileDelta, StatType};
use muo_server::mods::{SkillMod, StatMod};

use crate::aos::{AosArmorAttribute, AosWeaponAttribute, AttrKind, Attrs};
use crate::core;

/// Declares an `enum` stored as an int: `from_i32` is `None` for a number without a name (a
/// save with an unnamed value is reported as a bad record).
macro_rules! int_enum {
    ($(#[$m:meta])* $v:vis enum $name:ident { $($var:ident = $n:expr),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        #[repr(i32)]
        $v enum $name { $($var = $n),+ }
        impl $name {
            pub const fn from_i32(v: i32) -> Option<Self> {
                match v { $($n => Some(Self::$var),)+ _ => None }
            }
        }
    };
}

pub mod addon;
pub mod addon_container;
pub mod ammo_recovery;
pub mod armor;
pub mod clothing;
pub mod common;
pub mod cont;
pub mod corpse;
pub mod equip_hooks;
pub mod ext_bags;
pub mod flip;
pub mod flip_table;
pub mod food;
pub mod gold;
pub mod house_deps;
pub mod interfaces;
pub mod jewel;
pub mod leaves;
pub mod prop_bags;
pub mod prop_tables;
pub mod resource;
pub mod runic;
pub mod slayer;
pub mod slayer_data;
pub mod tool;
pub mod weapon;
pub mod weapon_ability;
pub mod weapon_bases;
pub mod weapon_combat;
pub mod weapon_hooks;
pub mod weapon_migrate;
pub mod weapon_text;

/// Reads an `enum` written with `WriteEnum` (an encoded int).
pub fn read_enum<T>(
    r: &mut dyn muo_io::GenericReader,
    from: fn(i32) -> Option<T>,
    what: &str,
) -> Result<T> {
    let v = r.read_enum(4)? as i32;
    from(v).ok_or_else(|| IoError::InvalidData(format!("{what}: no value {v}")))
}

/// `Utility.RandomMinMax(min, max)` on the world's generator.
pub fn random_min_max(w: &mut World, min: i32, max: i32) -> i32 {
    muo_primitives::utility::random_min_max(&mut w.rng, min, max)
}

/// `Core.UOR`.
pub fn core_uor(w: &World) -> bool {
    core::expansion(w) >= muo_data::expansion::Expansion::UOR
}

/// `Core.UOTD`.
pub fn core_uotd(w: &World) -> bool {
    core::expansion(w) >= muo_data::expansion::Expansion::UOTD
}

/// `item.ty` or any ancestor is called `full_name`; for classes of crates this one doesn't depend on.
pub fn is_named(ty: &'static ItemType, full_name: &str) -> bool {
    let mut t = Some(ty);
    while let Some(c) = t {
        if c.full_name == full_name {
            return true;
        }
        t = c.parent;
    }
    false
}

/// The attribute bag `kind` of a weapon, armor, clothing or jewel, if the item has it.
pub fn attrs(item: &Item, kind: AttrKind) -> Option<&Attrs> {
    if let Some(d) = item.ext.get::<weapon::WeaponData>() {
        return Some(match kind {
            AttrKind::Attributes => &d.attributes,
            AttrKind::Weapon => &d.weapon_attributes,
            AttrKind::SkillBonuses => &d.skill_bonuses,
            AttrKind::Element => &d.element_damages,
            AttrKind::Armor => return None,
        });
    }
    if let Some(d) = item.ext.get::<armor::ArmorData>() {
        return Some(match kind {
            AttrKind::Attributes => &d.attributes,
            AttrKind::Armor => &d.armor_attributes,
            AttrKind::SkillBonuses => &d.skill_bonuses,
            AttrKind::Weapon => &d.weapon_attributes,
            AttrKind::Element => return None,
        });
    }
    if let Some(d) = item.ext.get::<clothing::ClothingData>() {
        return Some(match kind {
            AttrKind::Attributes => &d.attributes,
            AttrKind::Armor => &d.clothing_attributes,
            AttrKind::SkillBonuses => &d.skill_bonuses,
            AttrKind::Element => &d.resistances,
            AttrKind::Weapon => return None,
        });
    }
    if let Some(d) = item.ext.get::<ext_bags::QuiverData>() {
        return (kind == AttrKind::Attributes).then_some(&d.attributes);
    }
    if let Some(d) = item.ext.get::<ext_bags::TalismanData>() {
        return match kind {
            AttrKind::Attributes => Some(&d.attributes),
            AttrKind::SkillBonuses => Some(&d.skill_bonuses),
            _ => None,
        };
    }
    if let Some(d) = item.ext.get::<ext_bags::SpellbookData>() {
        return match kind {
            AttrKind::Attributes => Some(&d.attributes),
            AttrKind::SkillBonuses => Some(&d.skill_bonuses),
            _ => None,
        };
    }
    let d = item.ext.get::<jewel::JewelData>()?;
    Some(match kind {
        AttrKind::Attributes => &d.attributes,
        AttrKind::SkillBonuses => &d.skill_bonuses,
        AttrKind::Element => &d.resistances,
        AttrKind::Weapon | AttrKind::Armor => return None,
    })
}

fn attrs_mut(item: &mut Item, kind: AttrKind) -> Option<&mut Attrs> {
    if item.ext.get::<weapon::WeaponData>().is_some() {
        let d = item.ext.get_mut::<weapon::WeaponData>()?;
        return Some(match kind {
            AttrKind::Attributes => &mut d.attributes,
            AttrKind::Weapon => &mut d.weapon_attributes,
            AttrKind::SkillBonuses => &mut d.skill_bonuses,
            AttrKind::Element => &mut d.element_damages,
            AttrKind::Armor => return None,
        });
    }
    if item.ext.get::<armor::ArmorData>().is_some() {
        let d = item.ext.get_mut::<armor::ArmorData>()?;
        return Some(match kind {
            AttrKind::Attributes => &mut d.attributes,
            AttrKind::Armor => &mut d.armor_attributes,
            AttrKind::SkillBonuses => &mut d.skill_bonuses,
            AttrKind::Weapon => &mut d.weapon_attributes,
            AttrKind::Element => return None,
        });
    }
    if item.ext.get::<clothing::ClothingData>().is_some() {
        let d = item.ext.get_mut::<clothing::ClothingData>()?;
        return Some(match kind {
            AttrKind::Attributes => &mut d.attributes,
            AttrKind::Armor => &mut d.clothing_attributes,
            AttrKind::SkillBonuses => &mut d.skill_bonuses,
            AttrKind::Element => &mut d.resistances,
            AttrKind::Weapon => return None,
        });
    }
    if item.ext.get::<ext_bags::QuiverData>().is_some() {
        let d = item.ext.get_mut::<ext_bags::QuiverData>()?;
        return (kind == AttrKind::Attributes).then_some(&mut d.attributes);
    }
    if item.ext.get::<ext_bags::TalismanData>().is_some() {
        let d = item.ext.get_mut::<ext_bags::TalismanData>()?;
        return match kind {
            AttrKind::Attributes => Some(&mut d.attributes),
            AttrKind::SkillBonuses => Some(&mut d.skill_bonuses),
            _ => None,
        };
    }
    if item.ext.get::<ext_bags::SpellbookData>().is_some() {
        let d = item.ext.get_mut::<ext_bags::SpellbookData>()?;
        return match kind {
            AttrKind::Attributes => Some(&mut d.attributes),
            AttrKind::SkillBonuses => Some(&mut d.skill_bonuses),
            _ => None,
        };
    }
    let d = item.ext.get_mut::<jewel::JewelData>()?;
    Some(match kind {
        AttrKind::Attributes => &mut d.attributes,
        AttrKind::SkillBonuses => &mut d.skill_bonuses,
        AttrKind::Element => &mut d.resistances,
        AttrKind::Weapon | AttrKind::Armor => return None,
    })
}

/// `bag[attribute]` (`BaseAttributes.GetValue`): 0 for a missing bag and before AOS.
pub fn attr(w: &World, id: ItemId, kind: AttrKind, mask: u32) -> i32 {
    let aos = core::aos(w);
    w.item(id)
        .and_then(|i| attrs(i, kind))
        .map_or(0, |a| a.get(aos, mask))
}

/// `bag[attribute] = value` (`BaseAttributes.SetValue`): a changed `DurabilityBonus` unscales and rescales the
/// owner's hit points around the change; a worn owner's stat timers, resistances and skill mods are refreshed
/// and the owner's properties are invalidated.
pub fn set_attr(w: &mut World, id: ItemId, kind: AttrKind, mask: u32, value: i32) {
    let durability = (kind == AttrKind::Weapon
        && mask == AosWeaponAttribute::DurabilityBonus as u32)
        || (kind == AttrKind::Armor && mask == AosArmorAttribute::DurabilityBonus as u32);
    if durability {
        unscale_durability(w, id);
    }
    if let Some(a) = w.item_mut(id).and_then(|i| attrs_mut(i, kind)) {
        a.set_raw(mask, value);
    }
    if durability {
        scale_durability(w, id);
    }
    if let Some(m) = parent_mobile(w, id)
        && let Some(ty) = w.mobile(m).map(|m| m.ty)
    {
        (ty.check_stat_timers)(w, m);
        (ty.update_resistances)(w, m);
        (ty.delta)(
            w,
            m,
            MobileDelta::STAT
                | MobileDelta::WEAPON_DAMAGE
                | MobileDelta::HITS
                | MobileDelta::STAM
                | MobileDelta::MANA,
        );
        if kind == AttrKind::SkillBonuses {
            skill_bonuses_remove(w, id);
            skill_bonuses_add_to(w, id, m);
        }
    }
    ops::invalidate_properties(w, id);
}

/// `Parent as Mobile` of an item.
pub fn parent_mobile(w: &World, id: ItemId) -> Option<MobileId> {
    match w.item(id)?.parent {
        Some(EntityId::Mobile(m)) if w.mobile(m).is_some() => Some(m),
        _ => None,
    }
}

/// `AosSkillBonuses.AddTo(m)` for the item's skill bonus bag: one `DefaultSkillMod` (relative, obeying the cap)
/// per slot with a bonus.
pub fn skill_bonuses_add_to(w: &mut World, id: ItemId, m: MobileId) {
    skill_bonuses_remove(w, id);
    let aos = core::aos(w);
    let Some(item) = w.item(id) else { return };
    let serial = item.serial;
    let Some(bag) = attrs(item, AttrKind::SkillBonuses) else {
        return;
    };
    let slots: Vec<(muo_server::skills::SkillName, f64)> =
        (0..5).filter_map(|i| bag.skill_bonus(aos, i)).collect();
    let Some(mty) = w.mobile(m).map(|m| m.ty) else {
        return;
    };
    for (skill, bonus) in slots {
        let name = format!("{serial}{skill:?}");
        let mut sk = SkillMod::new_default(w, skill, Some(&name), true, bonus);
        sk.obey_cap = true;
        let mod_id = sk.id;
        (mty.add_skill_mod)(w, m, sk);
        if let Some(bag) = w
            .item_mut(id)
            .and_then(|i| attrs_mut(i, AttrKind::SkillBonuses))
        {
            bag.mods.push((mod_id, m));
        }
    }
}

/// `AosSkillBonuses.Remove()`: takes its skill mods off whoever holds them (and, from ML on, lets spells that
/// need the lost skill end).
pub fn skill_bonuses_remove(w: &mut World, id: ItemId) {
    let Some(mods) = w
        .item_mut(id)
        .and_then(|i| attrs_mut(i, AttrKind::SkillBonuses))
        .map(|b| std::mem::take(&mut b.mods))
    else {
        return;
    };
    let ml = core::ml(w);
    for (mod_id, owner) in mods {
        // A mod the owner no longer has was removed already (its owner is null).
        let Some(ty) = w
            .mobile(owner)
            .filter(|m| m.skill_mods.iter().any(|s| s.id == mod_id))
            .map(|m| m.ty)
        else {
            continue;
        };
        (ty.remove_skill_mod)(w, owner, mod_id);
        if ml && let Some(f) = equip_hooks::hooks(w).and_then(|h| h.check_cancel_morph) {
            f(w, owner);
        }
    }
}

/// `m.AddStatMod(new StatMod(type, $"{serial}Str", bonus, TimeSpan.Zero))` for each bonus that is not 0.
pub fn add_stat_mods(w: &mut World, id: ItemId, m: MobileId, str: i32, dex: i32, int: i32) {
    let Some(serial) = w.item(id).map(|i| i.serial) else {
        return;
    };
    let Some(ty) = w.mobile(m).map(|m| m.ty) else {
        return;
    };
    for (stat, suffix, bonus) in [
        (StatType::STR, "Str", str),
        (StatType::DEX, "Dex", dex),
        (StatType::INT, "Int", int),
    ] {
        if bonus != 0 {
            let name = format!("{serial}{suffix}");
            let sm = StatMod::new(w, stat, Some(&name), bonus, TimeSpan::ZERO, None);
            (ty.add_stat_mod)(w, m, sm);
        }
    }
}

/// `m.RemoveStatMod($"{serial}Str")` and the Dex and Int ones.
pub fn remove_stat_mods(w: &mut World, id: ItemId, m: MobileId) {
    let Some(serial) = w.item(id).map(|i| i.serial) else {
        return;
    };
    let Some(ty) = w.mobile(m).map(|m| m.ty) else {
        return;
    };
    for suffix in ["Str", "Dex", "Int"] {
        (ty.remove_stat_mod_by_name)(w, m, &format!("{serial}{suffix}"));
    }
}

/// `UnscaleDurability()` of whichever family the item is.
pub fn unscale_durability(w: &mut World, id: ItemId) {
    let Some(i) = w.item(id) else { return };
    if i.ext.get::<weapon::WeaponData>().is_some() {
        weapon::unscale_durability(w, id);
    } else if i.ext.get::<clothing::ClothingData>().is_some() {
        clothing::unscale_durability(w, id);
    } else if i.ext.get::<armor::ArmorData>().is_some() {
        armor::unscale_durability(w, id);
    }
}

/// `ScaleDurability()` of whichever family the item is.
pub fn scale_durability(w: &mut World, id: ItemId) {
    let Some(i) = w.item(id) else { return };
    if i.ext.get::<weapon::WeaponData>().is_some() {
        weapon::scale_durability(w, id);
    } else if i.ext.get::<clothing::ClothingData>().is_some() {
        clothing::scale_durability(w, id);
    } else if i.ext.get::<armor::ArmorData>().is_some() {
        armor::scale_durability(w, id);
    }
}

/// How an equipment piece is worn down, the tail every `OnHit`/`TryLowerDurability` of armor, shields and clothing
/// spells out: the family's hit point accessors, whose setters carry that family's side effects.
pub(crate) struct Wear {
    pub hit_points: fn(&World, ItemId) -> i32,
    pub max_hit_points: fn(&World, ItemId) -> i32,
    pub set_hit_points: fn(&mut World, ItemId, i32),
    pub set_max_hit_points: fn(&mut World, ItemId, i32),
}

impl Wear {
    /// `if (wear > 0 && _maxHitPoints > 0) { .. }`: the hit points go first, then the maximum, and a piece worn
    /// past its maximum is deleted.
    pub(crate) fn apply(&self, w: &mut World, id: ItemId, mut wear: i32) {
        if wear <= 0 || (self.max_hit_points)(w, id) <= 0 {
            return;
        }
        let hp = (self.hit_points)(w, id);
        if hp >= wear {
            (self.set_hit_points)(w, id, hp - wear);
            wear = 0;
        } else {
            wear -= hp;
            (self.set_hit_points)(w, id, 0);
        }
        if wear > 0 {
            let max = (self.max_hit_points)(w, id);
            if max > wear {
                (self.set_max_hit_points)(w, id, max - wear);
                if let Some(m) = parent_mobile(w, id) {
                    // Your equipment is severely damaged.
                    muo_server::mobile::part2::messages::local_overhead_message_number(
                        w,
                        m,
                        muo_primitives::MessageType::REGULAR,
                        0x3B2,
                        1_061_121,
                        "",
                    );
                }
            } else if let Some(ty) = w.item(id).map(|i| i.ty) {
                (ty.delete)(w, id);
            }
        }
    }
}

/// A statistic that differs from `Core.ML` on (`Core.ML ? ml : pre`), as the weapons' `AosMinDamage` do.
#[derive(Copy, Clone, Debug)]
pub struct MlInt {
    pub pre: i32,
    pub ml: i32,
}

impl MlInt {
    pub fn get(self, ml: bool) -> i32 {
        if ml { self.ml } else { self.pre }
    }
}

/// `v` before and after ML.
pub const fn same(v: i32) -> MlInt {
    MlInt { pre: v, ml: v }
}

/// `Core.ML ? ml : pre`.
pub const fn ml(pre: i32, ml: i32) -> MlInt {
    MlInt { pre, ml }
}

#[cfg(test)]
mod equip_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod weapon_tests;

/// The gender half of `CanEquip`/`ValidateMobile` of armor and clothing: tells `m` why and returns true when the
/// piece is not for their gender.
pub(crate) fn wrong_gender(
    w: &mut World,
    m: MobileId,
    allow_male: bool,
    allow_female: bool,
) -> bool {
    let female = w.mobile(m).is_some_and(|m| m.female);
    if !allow_male && !female {
        if allow_female {
            muo_server::item::deps::send_localized_message(w, m, 1010388); // Only females can wear this.
        } else {
            muo_server::item::deps::send_message(w, m, "You may not wear this.");
        }
        return true;
    }
    if !allow_female && female {
        if allow_male {
            muo_server::item::deps::send_localized_message(w, m, 1063343); // Only males can wear this.
        } else {
            muo_server::item::deps::send_message(w, m, "You may not wear this.");
        }
        return true;
    }
    false
}

/// The layer half of `CheckPropertyConflict(m)` of armor and clothing: pants conflict with inner legs, a shirt
/// with an inner torso.
pub(crate) fn property_conflict_layer(w: &World, id: ItemId, m: MobileId) -> bool {
    use muo_server::layer::Layer;
    let on = |l| muo_server::item::deps::find_item_on_layer(w, m, l).is_some();
    match w.item(id).map(|i| i.layer) {
        Some(Layer::Pants) => on(Layer::InnerLegs),
        Some(Layer::Shirt) => on(Layer::InnerTorso),
        _ => false,
    }
}

/// `Timer.DelayCall((item, crafter) => item._crafter = crafter?.RawName, this, crafter)`: the old saves keep the
/// crafter as a mobile, whose name is only readable once everything is loaded, so it is taken on the next tick.
pub(crate) fn defer_crafter_name(
    w: &mut World,
    id: ItemId,
    crafter: Option<MobileId>,
    set: fn(&mut World, ItemId, Option<String>),
) {
    muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        let name = crafter.and_then(|m| muo_server::mobile::part1::props_b::raw_name(w, m));
        set(w, id, name);
    });
}
