//! The AOS attribute bags (`BaseAttributes` and its five subclasses).
//!
//! All five are the same thing, a sorted sparse list of `(bit, value)` pairs, so one [`Attrs`] holds the data
//! and the enums below name the bits. Which bag an item has is told by the field it sits in (`WeaponData::
//! attributes`, `weapon_attributes`, ..). Reading honours `Core.AOS` ([`Attrs::get`] takes it as an argument);
//! writing through [`crate::items::set_attr`] also runs the side effects `BaseAttributes.SetValue` had.
//!
//! The mobile side of `AosSkillBonuses.AddTo/Remove` (skill mods) is `items::skill_bonuses_add_to`/`skill_bonuses_remove` (`CheckCancelMorph` is `equip_hooks`).

use muo_io::{GenericReader, GenericWriter, Result};
use muo_server::ids::MobileId;
use muo_server::mods::ModId;
use muo_server::property_list::PropertyList;
use muo_server::skills::SkillName;

/// `AosAttribute`: bits of `AosAttributes`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AosAttribute {
    RegenHits = 0x1,
    RegenStam = 0x2,
    RegenMana = 0x4,
    DefendChance = 0x8,
    AttackChance = 0x10,
    BonusStr = 0x20,
    BonusDex = 0x40,
    BonusInt = 0x80,
    BonusHits = 0x100,
    BonusStam = 0x200,
    BonusMana = 0x400,
    WeaponDamage = 0x800,
    WeaponSpeed = 0x1000,
    SpellDamage = 0x2000,
    CastRecovery = 0x4000,
    CastSpeed = 0x8000,
    LowerManaCost = 0x1_0000,
    LowerRegCost = 0x2_0000,
    ReflectPhysical = 0x4_0000,
    EnhancePotions = 0x8_0000,
    Luck = 0x10_0000,
    SpellChanneling = 0x20_0000,
    NightSight = 0x40_0000,
    IncreasedKarmaLoss = 0x80_0000,
}

/// `AosWeaponAttribute`: bits of `AosWeaponAttributes`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AosWeaponAttribute {
    LowerStatReq = 0x1,
    SelfRepair = 0x2,
    HitLeechHits = 0x4,
    HitLeechStam = 0x8,
    HitLeechMana = 0x10,
    HitLowerAttack = 0x20,
    HitLowerDefend = 0x40,
    HitMagicArrow = 0x80,
    HitHarm = 0x100,
    HitFireball = 0x200,
    HitLightning = 0x400,
    HitDispel = 0x800,
    HitColdArea = 0x1000,
    HitFireArea = 0x2000,
    HitPoisonArea = 0x4000,
    HitEnergyArea = 0x8000,
    HitPhysicalArea = 0x1_0000,
    ResistPhysicalBonus = 0x2_0000,
    ResistFireBonus = 0x4_0000,
    ResistColdBonus = 0x8_0000,
    ResistPoisonBonus = 0x10_0000,
    ResistEnergyBonus = 0x20_0000,
    UseBestSkill = 0x40_0000,
    MageWeapon = 0x80_0000,
    DurabilityBonus = 0x100_0000,
}

/// `AosArmorAttribute`: bits of `AosArmorAttributes`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AosArmorAttribute {
    LowerStatReq = 0x1,
    SelfRepair = 0x2,
    MageArmor = 0x4,
    DurabilityBonus = 0x8,
}

/// `AosElementAttribute`: bits of `AosElementAttributes`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AosElementAttribute {
    Physical = 0x1,
    Fire = 0x2,
    Cold = 0x4,
    Poison = 0x8,
    Energy = 0x10,
    Chaos = 0x20,
    Direct = 0x40,
}

/// Which of an item's bags (they are all `BaseAttributes`).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AttrKind {
    /// `AosAttributes`.
    Attributes,
    /// `AosWeaponAttributes`.
    Weapon,
    /// `AosArmorAttributes`.
    Armor,
    /// `AosSkillBonuses`.
    SkillBonuses,
    /// `AosElementAttributes`.
    Element,
}

/// `BaseAttributes`: `_names` (which bits are set) and `_values` (one per set bit, in bit order).
///
/// `mods` is `AosSkillBonuses.m_Mods` (the skill mods this bag put on a wearer, with the wearer): only a skill
/// bonus bag has any, and a copy (`new AosSkillBonuses(owner, other)`) starts without them.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Attrs {
    names: u32,
    values: Vec<i32>,
    pub mods: Vec<(ModId, MobileId)>,
}

impl Clone for Attrs {
    /// `new BaseAttributes(owner, other)`: the names and values, never the live skill mods.
    fn clone(&self) -> Attrs {
        Attrs {
            names: self.names,
            values: self.values.clone(),
            mods: Vec::new(),
        }
    }
}

impl Attrs {
    /// `IsEmpty`.
    pub fn is_empty(&self) -> bool {
        self.names == 0
    }

    /// `GetValue(bitmask)` without the `Core.AOS` check; callers go through [`Attrs::get`].
    fn raw(&self, mask: u32) -> i32 {
        if self.names & mask == 0 {
            return 0;
        }
        self.values.get(self.index(mask)).copied().unwrap_or(0)
    }

    /// `GetValue(bitmask)`: 0 before AOS.
    pub fn get(&self, core_aos: bool, mask: u32) -> i32 {
        if core_aos { self.raw(mask) } else { 0 }
    }

    /// The part of `SetValue` that changes the bag. The owner's side effects are in `items::set_attr`.
    pub fn set_raw(&mut self, mask: u32, value: i32) {
        let index = self.index(mask);
        if value != 0 {
            if self.names & mask != 0 {
                if let Some(v) = self.values.get_mut(index) {
                    *v = value;
                }
            } else if index <= self.values.len() {
                self.values.insert(index, value);
                self.names |= mask;
            }
        } else if self.names & mask != 0 && index < self.values.len() {
            self.names &= !mask;
            self.values.remove(index);
        }
    }

    /// `GetIndex`: how many lower bits are set. The (never used) mask 0x80000000 + overflow would give -1;
    /// the loop here ends at the same place for every mask the enums define.
    fn index(&self, mask: u32) -> usize {
        // shortcut: counting the set bits below `mask` equals a bit-by-bit loop for a one-bit mask; a multi-bit or
        // zero mask is never passed by the callers.
        (self.names & mask.wrapping_sub(1)).count_ones() as usize
    }

    /// `ClearValues()`.
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// `Serialize`: version (encoded int 0), `_names`, then the values as encoded ints.
    pub fn serialize(&self, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(0);
        writer.write_u32(self.names);
        writer.write_encoded_int(self.values.len() as i32);
        for v in &self.values {
            writer.write_encoded_int(*v);
        }
    }

    /// `Deserialize`.
    pub fn deserialize(reader: &mut dyn GenericReader) -> Result<Attrs> {
        let _version = reader.read_encoded_int()?;
        let names = reader.read_u32()?;
        let len = reader.read_encoded_int()?;
        // A bag never holds more than 32 values; a larger count is a corrupt save, not an allocation request.
        if !(0..=32).contains(&len) {
            return Err(muo_io::IoError::InvalidData(format!(
                "attribute count {len}"
            )));
        }
        let mut values = Vec::with_capacity(len as usize);
        for _ in 0..len {
            values.push(reader.read_encoded_int()?);
        }
        Ok(Attrs {
            names,
            values,
            mods: Vec::new(),
        })
    }

    /// `AosSkillBonuses.GetValues(index, ..)`: slot `index` (0..5) as a skill and a bonus; false if the bonus is 0.
    pub fn skill_bonus(&self, core_aos: bool, index: u32) -> Option<(SkillName, f64)> {
        let mut v = self.get(core_aos, 1 << index);
        let (mut skill, mut bonus) = (0_i32, 0_i32);
        for _ in 0..16 {
            skill = (skill << 1) | (v & 1);
            v >>= 1;
            bonus = (bonus << 1) | (v & 1);
            v >>= 1;
        }
        let bonus = f64::from(bonus) / 10.0;
        // The stored int may be any value; a skill without a name cannot be held, so it reads as no bonus.
        let skill = SkillName::from_i32(skill)?;
        (bonus != 0.0).then_some((skill, bonus))
    }

    /// `AosSkillBonuses.SetValues(index, skill, bonus)` for the bag; the owner side effects are the caller's.
    pub fn skill_bonus_value(skill: SkillName, bonus: f64) -> i32 {
        let (mut v, mut v_skill, mut v_bonus) = (0_i32, skill as i32, (bonus * 10.0) as i32);
        for _ in 0..16 {
            v = (v << 1) | (v_bonus & 1);
            v_bonus >>= 1;
            v = (v << 1) | (v_skill & 1);
            v_skill >>= 1;
        }
        v
    }
}

fn add_nonzero(list: &mut PropertyList, number: i32, value: i32) {
    if value != 0 {
        list.add_int(number, value);
    }
}

/// `AosAttributes.GetProperties(list, damageBonus, hitChanceBonus, luckBonus)`.
pub fn add_attribute_properties(
    a: &Attrs,
    list: &mut PropertyList,
    core_aos: bool,
    core_ml: bool,
    damage_bonus: i32,
    hit_chance_bonus: i32,
    luck_bonus: i32,
) {
    use AosAttribute as A;
    let g = |x: AosAttribute| a.get(core_aos, x as u32);
    add_nonzero(list, 1060401, g(A::WeaponDamage) + damage_bonus); // damage increase ~1_val~%
    add_nonzero(list, 1060408, g(A::DefendChance)); // defense chance increase ~1_val~%
    add_nonzero(list, 1060409, g(A::BonusDex)); // dexterity bonus ~1_val~
    add_nonzero(list, 1060411, g(A::EnhancePotions)); // enhance potions ~1_val~%
    add_nonzero(list, 1060412, g(A::CastRecovery)); // faster cast recovery ~1_val~
    add_nonzero(list, 1060413, g(A::CastSpeed)); // faster casting ~1_val~
    add_nonzero(list, 1060415, g(A::AttackChance) + hit_chance_bonus); // hit chance increase ~1_val~%
    add_nonzero(list, 1060431, g(A::BonusHits)); // hit point increase ~1_val~
    add_nonzero(list, 1060432, g(A::BonusInt)); // intelligence bonus ~1_val~
    add_nonzero(list, 1060433, g(A::LowerManaCost)); // lower mana cost ~1_val~%
    add_nonzero(list, 1060434, g(A::LowerRegCost)); // lower reagent cost ~1_val~%
    add_nonzero(list, 1060436, g(A::Luck) + luck_bonus); // luck ~1_val~
    add_nonzero(list, 1060439, g(A::BonusMana)); // mana increase ~1_val~
    add_nonzero(list, 1060440, g(A::RegenMana)); // mana regeneration ~1_val~
    if g(A::NightSight) != 0 {
        list.add(1060441); // night sight
    }
    add_nonzero(list, 1060442, g(A::ReflectPhysical)); // reflect physical damage ~1_val~%
    add_nonzero(list, 1060443, g(A::RegenStam)); // stamina regeneration ~1_val~
    add_nonzero(list, 1060444, g(A::RegenHits)); // hit point regeneration ~1_val~
    if g(A::SpellChanneling) != 0 {
        list.add(1060482); // spell channeling
    }
    add_nonzero(list, 1060483, g(A::SpellDamage)); // spell damage increase ~1_val~%
    add_nonzero(list, 1060484, g(A::BonusStam)); // stamina increase ~1_val~
    add_nonzero(list, 1060485, g(A::BonusStr)); // strength bonus ~1_val~
    add_nonzero(list, 1060486, g(A::WeaponSpeed)); // swing speed increase ~1_val~%
    if core_ml {
        add_nonzero(list, 1075210, g(A::IncreasedKarmaLoss)); // Increased Karma Loss ~1val~%
    }
}

/// `AosArmorAttributes.GetProperties(list)`.
pub fn add_armor_attribute_properties(a: &Attrs, list: &mut PropertyList, core_aos: bool) {
    if a.get(core_aos, AosArmorAttribute::MageArmor as u32) != 0 {
        list.add(1060437); // mage armor
    }
    let prop = a.get(core_aos, AosArmorAttribute::SelfRepair as u32);
    if prop != 0 {
        list.add_int(1060450, prop); // self repair ~1_val~
    }
}

/// `AosSkillBonuses.GetProperties(list)`.
pub fn add_skill_bonus_properties(a: &Attrs, list: &mut PropertyList, core_aos: bool) {
    for i in 0..5 {
        if let Some((skill, bonus)) = a.skill_bonus(core_aos, i) {
            // `$"{GetLabel(skill):#}	{bonus}"`
            list.add_number_fmt(
                1060451 + i as i32,
                format_args!("#{}	{}", skill_label(skill), bonus),
            );
        }
    }
}

/// `AosSkillBonuses.GetLabel`: the cliloc number naming a skill in "+N skill" lines.
pub fn skill_label(skill: SkillName) -> i32 {
    match skill {
        SkillName::EvalInt => 1_002_070,
        SkillName::Forensics => 1_002_078,
        SkillName::Lockpicking => 1_002_097,
        _ => 1_044_060 + skill as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::SaveWriter;

    #[test]
    fn set_keeps_bit_order_and_removes_on_zero() {
        let mut a = Attrs::default();
        a.set_raw(AosAttribute::Luck as u32, 50);
        a.set_raw(AosAttribute::RegenHits as u32, 2);
        a.set_raw(AosAttribute::WeaponDamage as u32, 35);
        assert_eq!(a.values, vec![2, 35, 50]);
        assert_eq!(a.names, 0x1 | 0x800 | 0x10_0000);
        a.set_raw(AosAttribute::WeaponDamage as u32, 0);
        assert_eq!(a.values, vec![2, 50]);
        assert_eq!(a.get(true, AosAttribute::Luck as u32), 50);
        assert_eq!(
            a.get(false, AosAttribute::Luck as u32),
            0,
            "no attributes before AOS"
        );
        a.set_raw(AosAttribute::Luck as u32, 0);
        a.set_raw(AosAttribute::RegenHits as u32, 0);
        assert!(a.is_empty());
    }

    #[test]
    fn serialized_bytes() {
        let mut a = Attrs::default();
        a.set_raw(AosAttribute::WeaponDamage as u32, 35);
        let mut w = SaveWriter::new(true);
        a.serialize(&mut w);
        // version 0, names 0x00000800 (u32 LE), one value, 35
        assert_eq!(w.into_vec(), [0, 0x00, 0x08, 0, 0, 1, 35]);
    }

    #[test]
    fn skill_bonus_round_trip() {
        let v = Attrs::skill_bonus_value(SkillName::Swords, 7.5);
        let mut a = Attrs::default();
        a.set_raw(1, v);
        assert_eq!(a.skill_bonus(true, 0), Some((SkillName::Swords, 7.5)));
    }
}
