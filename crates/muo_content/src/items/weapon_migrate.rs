//! The loaders of BaseWeapon saves below version 11: the migrations and the version 10 layout (`V10Content`).
//!
//! - version 10 (`[SerializationGenerator(10)]`): the current flag layout with `PlayerConstructed` at 0x04000000
//!   (it moved onto `Item`) and every later flag one bit up; fields are read like the generated code reads them
//!   (`ReadEnum` = encoded int).
//! - versions 0 to 9 (pre-generator): the same flag bits, but plain `ReadInt` enums, a `Mobile` entity as crafter,
//!   `Identified` implied from version 6 on, and `Speed` as an int below version 9.
//!
//! Both end in `AfterDeserialization`, which the caller runs.
//!
//! shortcut: an enum value without a name (the field is a raw int in the save) reads as the default level;
//! no real save has one.

use muo_io::{GenericReader, Result};
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::serialization_ext::ReadEntityExt;

use super::weapon::{
    WeaponAccuracyLevel, WeaponDamageLevel, WeaponData, WeaponDurabilityLevel, WeaponQuality,
};
use crate::aos::Attrs;
use crate::craft::CraftResource;

mod old {
    pub const DAMAGE_LEVEL: u32 = 0x1;
    pub const ACCURACY_LEVEL: u32 = 0x2;
    pub const DURABILITY_LEVEL: u32 = 0x4;
    pub const QUALITY: u32 = 0x8;
    pub const HITS: u32 = 0x10;
    pub const MAX_HITS: u32 = 0x20;
    pub const SLAYER: u32 = 0x40;
    pub const POISON: u32 = 0x80;
    pub const POISON_CHARGES: u32 = 0x100;
    pub const CRAFTER: u32 = 0x200;
    pub const IDENTIFIED: u32 = 0x400;
    pub const STR_REQ: u32 = 0x800;
    pub const DEX_REQ: u32 = 0x1000;
    pub const INT_REQ: u32 = 0x2000;
    pub const MIN_DAMAGE: u32 = 0x4000;
    pub const MAX_DAMAGE: u32 = 0x8000;
    pub const HIT_SOUND: u32 = 0x1_0000;
    pub const MISS_SOUND: u32 = 0x2_0000;
    pub const SPEED: u32 = 0x4_0000;
    pub const MAX_RANGE: u32 = 0x8_0000;
    pub const SKILL: u32 = 0x10_0000;
    pub const TYPE: u32 = 0x20_0000;
    pub const ANIMATION: u32 = 0x40_0000;
    pub const RESOURCE: u32 = 0x80_0000;
    pub const ATTRIBUTES: u32 = 0x100_0000;
    pub const WEAPON_ATTRIBUTES: u32 = 0x200_0000;
    pub const PLAYER_CONSTRUCTED: u32 = 0x400_0000;
    pub const SKILL_BONUSES: u32 = 0x800_0000;
    pub const SLAYER2: u32 = 0x1000_0000;
    pub const ELEMENT_DAMAGES: u32 = 0x2000_0000;
    pub const ENGRAVED_TEXT: u32 = 0x4000_0000;
}

/// What the generated reader (`encoded`: version 10) or the old `Deserialize(reader, version)` (`legacy`) fills in.
#[derive(Copy, Clone)]
enum Layout {
    /// Version 10: `ReadEnum` (encoded int) enums, crafter as a string.
    V10,
    /// Versions 0 to 9.
    Legacy(i32),
}

/// What a migrated record yields besides the weapon state.
pub struct Migrated {
    pub data: WeaponData,
    /// `Item.PlayerConstructed`.
    pub player_constructed: bool,
    /// Legacy crafter entity: `Timer.DelayCall(crafter => _crafter = crafter?.RawName, ..)` names it a tick later.
    pub crafter_entity: Option<Option<MobileId>>,
}

fn int_value(r: &mut dyn GenericReader, layout: Layout) -> Result<i32> {
    match layout {
        Layout::V10 => Ok(r.read_enum(4)? as i32),
        Layout::Legacy(_) => r.read_i32(),
    }
}

fn read(w: &World, r: &mut dyn GenericReader, layout: Layout) -> Result<Migrated> {
    let flags = match layout {
        Layout::V10 => r.read_enum(4)? as u32,
        Layout::Legacy(_) => r.read_i32()? as u32,
    };
    let has = |f: u32| flags & f != 0;
    let mut d = WeaponData::for_load();
    let mut crafter_entity = None;

    if has(old::DAMAGE_LEVEL) {
        d.damage_level = WeaponDamageLevel::from_i32(int_value(r, layout)?)
            .unwrap_or(WeaponDamageLevel::Regular);
    }
    if has(old::ACCURACY_LEVEL) {
        d.accuracy_level = WeaponAccuracyLevel::from_i32(int_value(r, layout)?)
            .unwrap_or(WeaponAccuracyLevel::Regular);
    }
    if has(old::DURABILITY_LEVEL) {
        d.durability_level = WeaponDurabilityLevel::from_i32(int_value(r, layout)?)
            .unwrap_or(WeaponDurabilityLevel::Regular);
    }
    if has(old::QUALITY) {
        d.quality =
            WeaponQuality::from_i32(int_value(r, layout)?).unwrap_or(WeaponQuality::Regular);
    }
    if has(old::HITS) {
        d.hit_points = r.read_i32()?;
    }
    if has(old::MAX_HITS) {
        d.max_hit_points = r.read_i32()?;
    }
    if has(old::SLAYER) {
        d.slayer = int_value(r, layout)?;
    }
    if has(old::POISON) {
        d.poison = r.read_poison()?;
    }
    if has(old::POISON_CHARGES) {
        d.poison_charges = r.read_i32()?;
    }
    if has(old::CRAFTER) {
        match layout {
            Layout::V10 => d.crafter = r.read_string()?,
            Layout::Legacy(_) => crafter_entity = Some(r.read_mobile(w)?),
        }
    }
    if has(old::IDENTIFIED) {
        d.identified = match layout {
            Layout::Legacy(version) => version >= 6 || r.read_bool()?,
            Layout::V10 => true,
        };
    }
    if has(old::STR_REQ) {
        d.str_requirement = r.read_i32()?;
    }
    if has(old::DEX_REQ) {
        d.dex_requirement = r.read_i32()?;
    }
    if has(old::INT_REQ) {
        d.int_requirement = r.read_i32()?;
    }
    if has(old::MIN_DAMAGE) {
        d.min_damage = r.read_i32()?;
    }
    if has(old::MAX_DAMAGE) {
        d.max_damage = r.read_i32()?;
    }
    if has(old::HIT_SOUND) {
        d.hit_sound = r.read_i32()?;
    }
    if has(old::MISS_SOUND) {
        d.miss_sound = r.read_i32()?;
    }
    if has(old::SPEED) {
        d.speed = match layout {
            Layout::Legacy(version) if version < 9 => r.read_i32()? as f32,
            _ => r.read_f32()?,
        };
    }
    if has(old::MAX_RANGE) {
        d.max_range = r.read_i32()?;
    }
    if has(old::SKILL) {
        d.skill = int_value(r, layout)?;
    }
    if has(old::TYPE) {
        d.weapon_type = int_value(r, layout)?;
    }
    if has(old::ANIMATION) {
        d.animation = int_value(r, layout)?;
    }
    if has(old::RESOURCE) {
        d.resource = CraftResource::from_i32(int_value(r, layout)?).unwrap_or(CraftResource::Iron);
    }
    if has(old::ATTRIBUTES) {
        d.attributes = Attrs::deserialize(r)?;
    }
    if has(old::WEAPON_ATTRIBUTES) {
        d.weapon_attributes = Attrs::deserialize(r)?;
    }
    let player_constructed = has(old::PLAYER_CONSTRUCTED);
    if has(old::SKILL_BONUSES) {
        d.skill_bonuses = Attrs::deserialize(r)?;
    }
    if has(old::SLAYER2) {
        d.slayer2 = int_value(r, layout)?;
    }
    if has(old::ELEMENT_DAMAGES) {
        d.element_damages = Attrs::deserialize(r)?;
    }
    if has(old::ENGRAVED_TEXT) {
        d.engraved_text = r.read_string()?;
    }

    Ok(Migrated {
        data: d,
        player_constructed,
        crafter_entity,
    })
}

/// Reads a version 10 record (after the version int).
pub fn read_v10(w: &World, r: &mut dyn GenericReader) -> Result<Migrated> {
    read(w, r, Layout::V10)
}

/// `Deserialize(reader, version)` for versions 0 to 9 (after the version int).
pub fn read_legacy(w: &World, r: &mut dyn GenericReader, version: i32) -> Result<Migrated> {
    read(w, r, Layout::Legacy(version))
}

/// Stores the migrated state on the item; a legacy crafter entity names the crafter on the next timer tick
/// (`Timer.DelayCall(crafter => _crafter = crafter?.RawName, crafter)`).
pub fn apply(w: &mut World, id: ItemId, m: Migrated) {
    let Migrated {
        data,
        player_constructed,
        crafter_entity,
    } = m;
    if let Some(i) = w.item_mut(id) {
        i.ext = muo_server::ext::Ext::new(data);
        i.player_constructed = player_constructed;
    }
    if let Some(crafter) = crafter_entity {
        muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            let name = crafter.and_then(|c| muo_server::mobile::part1::props_b::raw_name(w, c));
            if let Some(d) = super::weapon::data_mut(w, id) {
                d.crafter = name;
            }
        });
    }
}
