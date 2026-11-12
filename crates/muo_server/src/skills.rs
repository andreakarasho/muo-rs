//! `SkillName`, `SkillLock`, `Skill`, `Skills` (`SkillInfo` is in `muo_data`).
//!
//! `Skills` and `Skill` are plain data owned by a `Mobile`; everything that reached back to the owner
//! (`Skill.Value`, `BaseFixedPoint.set`, `Skills.OnSkillChange`, `UseSkill`) is a function here taking the world,
//! the mobile and the [`SkillName`].
//!

use muo_data::skill_info::SkillInfo;
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::TimeSpan;

use crate::ids::MobileId;
use crate::world::World;

/// `SkillUseCallback`: the skill's use; returns the delay before the next skill.
pub type SkillUseCallback = fn(&mut World, MobileId) -> TimeSpan;

/// `SkillInfo.Table` plus each entry's `Callback`; lives in `World::resources`.
#[derive(Default)]
pub struct SkillTable {
    pub info: Vec<SkillInfo>,
    /// A missing or `None` entry is a skill without a callback.
    pub callbacks: Vec<Option<SkillUseCallback>>,
}

impl SkillTable {
    pub fn callback(&self, skill_id: usize) -> Option<SkillUseCallback> {
        self.callbacks.get(skill_id).copied().flatten()
    }
}

/// `SkillLock`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(u8)]
pub enum SkillLock {
    #[default]
    Up = 0,
    Down = 1,
    Locked = 2,
}

impl SkillLock {
    /// A byte cast; a value past `Locked` has no name (the constructor resets it to `Up`).
    pub const fn from_u8(v: u8) -> Option<SkillLock> {
        match v {
            0 => Some(SkillLock::Up),
            1 => Some(SkillLock::Down),
            2 => Some(SkillLock::Locked),
            _ => None,
        }
    }
}

pub use muo_data::skill_info::Stat;

/// One skill's stored values (`Skill`'s `m_Base`, `m_Cap`, `Lock`). Fixed point: tenths of a point.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Skill {
    pub base: u16,
    pub cap: u16,
    pub lock: SkillLock,
}

impl Default for Skill {
    /// `new Skill(owner, info, 0, 1000, SkillLock.Up)`, what `Skills[id]` makes on first touch.
    fn default() -> Self {
        Skill {
            base: 0,
            cap: 1000,
            lock: SkillLock::Up,
        }
    }
}

impl Skill {
    /// `Skill(owner, info, reader)`.
    pub fn read(reader: &mut dyn GenericReader) -> Result<Skill> {
        let mut s = Skill {
            base: 0,
            cap: 0,
            lock: SkillLock::Up,
        };
        let version = reader.read_u8()?;
        match version {
            0 => {
                s.base = reader.read_u16()?;
                s.cap = reader.read_u16()?;
                s.lock = SkillLock::from_u8(reader.read_u8()?).unwrap_or(SkillLock::Up);
            }
            0xFF => s.cap = 1000,
            v => {
                if v & 0xC0 == 0 {
                    if v & 0x1 != 0 {
                        s.base = reader.read_u16()?;
                    }
                    s.cap = if v & 0x2 != 0 {
                        reader.read_u16()?
                    } else {
                        1000
                    };
                    if v & 0x4 != 0 {
                        s.lock = SkillLock::from_u8(reader.read_u8()?).unwrap_or(SkillLock::Up);
                    }
                }
            }
        }
        Ok(s)
    }

    /// `Skill.Serialize`.
    pub fn write(&self, writer: &mut dyn GenericWriter) {
        if self.base == 0 && self.cap == 1000 && self.lock == SkillLock::Up {
            writer.write_u8(0xFF);
        } else {
            let mut flags = 0u8;
            if self.base != 0 {
                flags |= 0x1;
            }
            if self.cap != 1000 {
                flags |= 0x2;
            }
            if self.lock != SkillLock::Up {
                flags |= 0x4;
            }
            writer.write_u8(flags);
            if self.base != 0 {
                writer.write_i16(self.base as i16);
            }
            if self.cap != 1000 {
                writer.write_i16(self.cap as i16);
            }
            if self.lock != SkillLock::Up {
                writer.write_u8(self.lock as u8);
            }
        }
    }
}

/// `Skills` owned by a mobile. Slots are `None` until touched, then `Skill::default()`.
#[derive(Clone, Debug)]
pub struct Skills {
    /// `Cap`.
    pub cap: i32,
    /// `Total`.
    pub total: i32,
    /// `m_Skills`; sized to the table on first use (`Skills::ensure`).
    slots: Vec<Option<Skill>>,
    /// `m_Highest`, a skill id.
    highest: Option<usize>,
}

impl Default for Skills {
    /// `new Skills(owner)`: cap 7000, no skills touched.
    fn default() -> Self {
        Skills {
            cap: 7000,
            total: 0,
            slots: Vec::new(),
            highest: None,
        }
    }
}

impl Skills {
    /// Sizes the slots to the table (`SkillInfo.Table.Length`); never shrinks.
    pub fn ensure(&mut self, len: usize) {
        if self.slots.len() < len {
            self.slots.resize(len, None);
        }
    }

    /// `Length`.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// `this[id]` without the first-touch creation: the stored skill, if any.
    pub fn peek(&self, id: usize) -> Option<&Skill> {
        self.slots.get(id)?.as_ref()
    }

    /// `this[id]`: creates the default skill on first touch; `None` outside the table.
    pub fn get(&mut self, id: usize) -> Option<&mut Skill> {
        Some(self.slots.get_mut(id)?.get_or_insert_with(Skill::default))
    }

    /// `Skills(owner, reader)`; `table_len` is `SkillInfo.Table.Length`.
    pub fn read(reader: &mut dyn GenericReader, table_len: usize) -> Result<Skills> {
        let mut s = Skills {
            cap: 0,
            total: 0,
            slots: Vec::new(),
            highest: None,
        };
        let version = reader.read_i32()?;
        if !(0..=3).contains(&version) {
            return Ok(s);
        }
        if version == 0 {
            reader.read_i32()?;
        }
        if version >= 2 {
            s.cap = reader.read_i32()?;
        } else {
            s.cap = 7000;
        }
        if version < 3 {
            reader.read_i32()?; // m_Total
        }
        s.slots = vec![None; table_len];
        let count = reader.read_i32()?;
        for i in 0..count.max(0) as usize {
            let sk = Skill::read(reader)?;
            // Past the table: "Will be discarded".
            if i < table_len && (sk.base != 0 || sk.cap != 1000 || sk.lock != SkillLock::Up) {
                s.slots[i] = Some(sk);
                s.total += i32::from(sk.base);
            }
        }
        Ok(s)
    }

    /// `Skills.Serialize`: recomputes `Total` as it goes.
    pub fn write(&mut self, writer: &mut dyn GenericWriter) {
        self.total = 0;
        writer.write_i32(3);
        writer.write_i32(self.cap);
        writer.write_i32(self.slots.len() as i32);
        for slot in &self.slots {
            match slot {
                None => writer.write_u8(0xFF),
                Some(sk) => {
                    sk.write(writer);
                    self.total += i32::from(sk.base);
                }
            }
        }
    }

    /// `Highest`: the skill with the greatest base (first wins a tie), or skill 0 when none is set.
    pub fn highest(&mut self) -> Option<usize> {
        if self.highest.is_none() {
            let mut best = None;
            let mut value = i32::MIN;
            for (i, s) in self.slots.iter().enumerate() {
                if let Some(s) = s
                    && i32::from(s.base) > value
                {
                    value = i32::from(s.base);
                    best = Some(i);
                }
            }
            if best.is_none() && !self.slots.is_empty() {
                self.get(0);
                best = Some(0);
            }
            self.highest = best;
        }
        self.highest
    }

    /// The part of `Skills.OnSkillChange` that touches only `Skills`.
    fn invalidate_highest(&mut self, id: usize) {
        if self.highest == Some(id) {
            self.highest = None;
        } else if let Some(h) = self.highest
            && self.peek(id).map_or(0, |s| s.base) > self.peek(h).map_or(0, |s| s.base)
        {
            self.highest = Some(id);
        }
    }
}

fn table_len(w: &World) -> usize {
    w.resources.get::<SkillTable>().map_or(0, |t| t.info.len())
}

/// The skill's stored values, created on first touch (`Skills[name]`). `None` when the mobile has no `Skills` or
/// the id is outside the table.
fn skill_mut(w: &mut World, id: MobileId, skill: SkillName) -> Option<&mut Skill> {
    let len = table_len(w);
    let skills = w.mobile_mut(id)?.skills.as_mut()?;
    skills.ensure(len);
    skills.get(skill as usize)
}

/// `Skill.Base` (`BaseFixedPoint / 10.0`); 0 for a missing skill.
pub fn base(w: &mut World, id: MobileId, skill: SkillName) -> f64 {
    f64::from(base_fixed(w, id, skill)) / 10.0
}

/// `Skill.BaseFixedPoint`.
pub fn base_fixed(w: &mut World, id: MobileId, skill: SkillName) -> i32 {
    skill_mut(w, id, skill).map_or(0, |s| i32::from(s.base))
}

/// `Skill.BaseFixedPoint = value`: clamps to a ushort; a change adjusts `Skills.Total`, calls
/// `Skills.OnSkillChange` and then the mobile's `OnSkillChange(name, oldBase)`.
pub fn set_base_fixed(w: &mut World, id: MobileId, skill: SkillName, value: i32) {
    let sv = value.clamp(0, 0xFFFF) as u16;
    let Some(s) = skill_mut(w, id, skill) else {
        return;
    };
    let old = s.base;
    if old != sv {
        s.base = sv;
        if let Some(sk) = w.mobile_mut(id).and_then(|m| m.skills.as_mut()) {
            sk.total = sk.total - i32::from(old) + i32::from(sv);
        }
        on_skill_change(w, id, skill);
        if let Some(ty) = w.mobile(id).map(|m| m.ty) {
            (ty.on_skill_change)(w, id, skill, f64::from(old) / 10.0);
        }
    }
}

/// `Skill.Base = value`.
pub fn set_base(w: &mut World, id: MobileId, skill: SkillName, value: f64) {
    set_base_fixed(w, id, skill, (value * 10.0) as i32);
}

/// `Skill.CapFixedPoint`.
pub fn cap_fixed(w: &mut World, id: MobileId, skill: SkillName) -> i32 {
    skill_mut(w, id, skill).map_or(0, |s| i32::from(s.cap))
}

/// `Skill.Cap` (`CapFixedPoint / 10.0`).
pub fn cap(w: &mut World, id: MobileId, skill: SkillName) -> f64 {
    f64::from(cap_fixed(w, id, skill)) / 10.0
}

/// `Skill.CapFixedPoint = value`.
pub fn set_cap_fixed(w: &mut World, id: MobileId, skill: SkillName, value: i32) {
    let sv = value.clamp(0, 0xFFFF) as u16;
    let Some(s) = skill_mut(w, id, skill) else {
        return;
    };
    if s.cap != sv {
        s.cap = sv;
        on_skill_change(w, id, skill);
    }
}

/// `Skill.Cap = value`.
pub fn set_cap(w: &mut World, id: MobileId, skill: SkillName, value: f64) {
    set_cap_fixed(w, id, skill, (value * 10.0) as i32);
}

/// `Skill.Lock`.
pub fn lock(w: &mut World, id: MobileId, skill: SkillName) -> SkillLock {
    skill_mut(w, id, skill).map_or(SkillLock::Up, |s| s.lock)
}

/// `Skill.SetLockNoRelay`: a value past `Locked` is ignored (the enum can't hold one).
pub fn set_lock_no_relay(w: &mut World, id: MobileId, skill: SkillName, lock: SkillLock) {
    if let Some(s) = skill_mut(w, id, skill) {
        s.lock = lock;
    }
}

/// `Skill.Fixed` (`(int)(Value * 10)`).
pub fn fixed(w: &mut World, id: MobileId, skill: SkillName) -> i32 {
    (value(w, id, skill) * 10.0) as i32
}

/// `Skill.Value`: the non-racial value, or the owner's `RacialSkillBonus` when that is higher.
pub fn value(w: &mut World, id: MobileId, skill: SkillName) -> f64 {
    let v = non_racial_value(w, id, skill);
    let bonus = w
        .mobile(id)
        .map_or(0.0, |m| (m.ty.racial_skill_bonus)(w, id));
    if bonus > v { bonus } else { v }
}

/// `Skill.NonRacialValue` for a caller that only holds `&World` (no first-touch creation of the skill, and a skill mod whose
/// condition has lapsed is skipped instead of being removed first). Same number as [`value`].
pub fn non_racial_ref(w: &World, id: MobileId, skill: SkillName) -> f64 {
    let Some(m) = w.mobile(id) else { return 0.0 };
    let stored = m
        .skills
        .as_ref()
        .and_then(|s| s.peek(skill as usize))
        .copied();
    let s = stored.unwrap_or_default();
    let base = f64::from(s.base) / 10.0;
    let cap = f64::from(s.cap) / 10.0;
    let mut value = base;
    if let Some(i) = w
        .resources
        .get::<SkillTable>()
        .and_then(|t| t.info.get(skill as usize))
    {
        let mut offset = f64::from(m.str) * i.str_scale
            + f64::from(m.dex) * i.dex_scale
            + f64::from(m.int) * i.int_scale;
        let inv = 100.0 - base;
        if inv <= 0.0 {
            offset = 0.0;
        } else {
            offset *= inv;
            if i.stat_total > 0.0 && offset > i.stat_total * inv {
                offset = i.stat_total * inv;
            }
        }
        value += offset;
    }
    let (mut obey, mut not_obey) = (0.0, 0.0);
    for md in m
        .skill_mods
        .iter()
        .filter(|md| md.skill == skill && md.check_condition(w))
    {
        if md.relative {
            if md.obey_cap {
                obey += md.value
            } else {
                not_obey += md.value
            }
        } else {
            obey = 0.0;
            not_obey = 0.0;
            value = md.value;
        }
    }
    value += not_obey;
    if value < cap {
        value += obey;
        if value > cap {
            value = cap;
        }
    }
    value
}

/// `Skill.Value` for a caller that only holds `&World`: the non-racial value, or the racial bonus when higher.
pub fn value_ref(w: &World, id: MobileId, skill: SkillName) -> f64 {
    let value = non_racial_ref(w, id, skill);
    let bonus = w
        .mobile(id)
        .map_or(0.0, |m| (m.ty.racial_skill_bonus)(w, id));
    if bonus > value { bonus } else { value }
}

/// `Skill.NonRacialValue`: base plus the stat contribution, then the owner's skill mods.
pub fn non_racial_value(w: &mut World, id: MobileId, skill: SkillName) -> f64 {
    let base = base(w, id, skill);
    let cap = cap(w, id, skill);
    let (str, dex, int) = match w.mobile(id) {
        Some(m) => (m.str, m.dex, m.int),
        None => return base,
    };
    let Some((str_scale, dex_scale, int_scale, stat_total)) = w
        .resources
        .get::<SkillTable>()
        .and_then(|t| t.info.get(skill as usize))
        .map(|i| (i.str_scale, i.dex_scale, i.int_scale, i.stat_total))
    else {
        return base;
    };
    // `Owner.RawStr` is the stored stat, not `Str`.
    let mut stats_offset =
        f64::from(str) * str_scale + f64::from(dex) * dex_scale + f64::from(int) * int_scale;
    let inv = 100.0 - base;
    if inv <= 0.0 {
        stats_offset = 0.0;
    } else {
        stats_offset *= inv;
        if stat_total > 0.0 {
            let total = stat_total * inv;
            if stats_offset > total {
                stats_offset = total;
            }
        }
    }
    let mut value = base + stats_offset;

    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.validate_skill_mods)(w, id);
    }
    let (mut bonus_obey, mut bonus_not_obey) = (0.0, 0.0);
    if let Some(m) = w.mobile(id) {
        for md in &m.skill_mods {
            if md.skill != skill {
                continue;
            }
            if md.relative {
                if md.obey_cap {
                    bonus_obey += md.value;
                } else {
                    bonus_not_obey += md.value;
                }
            } else {
                bonus_obey = 0.0;
                bonus_not_obey = 0.0;
                value = md.value;
            }
        }
    }
    value += bonus_not_obey;
    if value < cap {
        value += bonus_obey;
        if value > cap {
            value = cap;
        }
    }
    value
}

/// `Skill.Update()` / `Skills.OnSkillChange(skill)`: refreshes `Highest`, tells the owner
/// (`OnSkillInvalidated`) and its client.
pub fn on_skill_change(w: &mut World, id: MobileId, skill: SkillName) {
    if let Some(sk) = w.mobile_mut(id).and_then(|m| m.skills.as_mut()) {
        sk.invalidate_highest(skill as usize);
    }
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.on_skill_invalidated)(w, id, skill);
    }
    if let Some(ns) = w.mobile(id).and_then(|m| m.net_state) {
        let entry = packet_entry(w, id, skill);
        crate::packets::player::send_skill_change(w, ns, &entry);
    }
}

/// What the skill packets read of one `Skill`.
pub fn packet_entry(
    w: &mut World,
    id: MobileId,
    skill: SkillName,
) -> crate::packets::player::SkillPacketEntry {
    crate::packets::player::SkillPacketEntry {
        skill_id: skill as i32,
        non_racial_value: non_racial_value(w, id, skill),
        base_fixed_point: base_fixed(w, id, skill),
        lock: lock(w, id, skill) as u8,
        cap_fixed_point: cap_fixed(w, id, skill),
    }
}

/// `Skill.Update()`.
pub fn update(w: &mut World, id: MobileId, skill: SkillName) {
    on_skill_change(w, id, skill);
}

/// `Skills.Total`.
pub fn total(w: &World, id: MobileId) -> i32 {
    w.mobile(id)
        .and_then(|m| m.skills.as_ref())
        .map_or(0, |s| s.total)
}

/// `Skills.Highest`.
pub fn highest(w: &mut World, id: MobileId) -> Option<SkillName> {
    let len = table_len(w);
    let skills = w.mobile_mut(id)?.skills.as_mut()?;
    skills.ensure(len);
    skills.highest().and_then(|i| SkillName::from_i32(i as i32))
}

/// `Skills.UseSkill(from, skillID)`.
pub fn use_skill(w: &mut World, from: MobileId, skill_id: i32) -> bool {
    let Some(ty) = w.mobile(from).map(|m| m.ty) else {
        return false;
    };
    if !crate::mobile::part2::stats::check_alive(w, from, true) {
        return false;
    }
    if !crate::mobile::part2::deps::region_on_skill_use(w, from, skill_id) {
        return false;
    }
    let Some(name) = SkillName::from_i32(skill_id) else {
        return false;
    };
    if !(ty.allow_skill_use)(w, from, name) {
        return false;
    }
    if skill_id >= 0 && (skill_id as usize) < table_len(w) {
        let callback = w
            .resources
            .get::<SkillTable>()
            .and_then(|t| t.callback(skill_id as usize));
        if let Some(cb) = callback {
            let (next_skill_time, has_spell) = match w.mobile(from) {
                Some(m) => (m.next_skill_time, m.spell.is_some()),
                None => return false,
            };
            if w.tick_count.wrapping_sub(next_skill_time) >= 0 && !has_spell {
                (ty.disruptive_action)(w, from);
                let delay = cb(w, from);
                let tick = w.tick_count;
                if let Some(m) = w.mobile_mut(from) {
                    m.next_skill_time = tick + i64::from(delay.total_milliseconds() as i32);
                }
                return true;
            }
            (ty.send_skill_message)(w, from);
        } else {
            // That skill cannot be used directly.
            crate::mobile::part2::messages::send_localized_message(w, from, 500014, "", 0x3B2);
        }
    }
    false
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(i32)]
pub enum SkillName {
    #[default]
    Alchemy = 0,
    Anatomy = 1,
    AnimalLore = 2,
    ItemID = 3,
    ArmsLore = 4,
    Parry = 5,
    Begging = 6,
    Blacksmith = 7,
    Fletching = 8,
    Peacemaking = 9,
    Camping = 10,
    Carpentry = 11,
    Cartography = 12,
    Cooking = 13,
    DetectHidden = 14,
    Discordance = 15,
    EvalInt = 16,
    Healing = 17,
    Fishing = 18,
    Forensics = 19,
    Herding = 20,
    Hiding = 21,
    Provocation = 22,
    Inscribe = 23,
    Lockpicking = 24,
    Magery = 25,
    MagicResist = 26,
    Tactics = 27,
    Snooping = 28,
    Musicianship = 29,
    Poisoning = 30,
    Archery = 31,
    SpiritSpeak = 32,
    Stealing = 33,
    Tailoring = 34,
    AnimalTaming = 35,
    TasteID = 36,
    Tinkering = 37,
    Tracking = 38,
    Veterinary = 39,
    Swords = 40,
    Macing = 41,
    Fencing = 42,
    Wrestling = 43,
    Lumberjacking = 44,
    Mining = 45,
    Meditation = 46,
    Stealth = 47,
    RemoveTrap = 48,
    Necromancy = 49,
    Focus = 50,
    Chivalry = 51,
    Bushido = 52,
    Ninjitsu = 53,
    Spellweaving = 54,
    Mysticism = 55,
    Imbuing = 56,
    Throwing = 57,
}

impl SkillName {
    pub const COUNT: usize = 58;

    /// The skill for a raw value (any int is accepted; values outside the table have no name).
    pub const fn from_i32(v: i32) -> Option<SkillName> {
        Some(match v {
            0 => SkillName::Alchemy,
            1 => SkillName::Anatomy,
            2 => SkillName::AnimalLore,
            3 => SkillName::ItemID,
            4 => SkillName::ArmsLore,
            5 => SkillName::Parry,
            6 => SkillName::Begging,
            7 => SkillName::Blacksmith,
            8 => SkillName::Fletching,
            9 => SkillName::Peacemaking,
            10 => SkillName::Camping,
            11 => SkillName::Carpentry,
            12 => SkillName::Cartography,
            13 => SkillName::Cooking,
            14 => SkillName::DetectHidden,
            15 => SkillName::Discordance,
            16 => SkillName::EvalInt,
            17 => SkillName::Healing,
            18 => SkillName::Fishing,
            19 => SkillName::Forensics,
            20 => SkillName::Herding,
            21 => SkillName::Hiding,
            22 => SkillName::Provocation,
            23 => SkillName::Inscribe,
            24 => SkillName::Lockpicking,
            25 => SkillName::Magery,
            26 => SkillName::MagicResist,
            27 => SkillName::Tactics,
            28 => SkillName::Snooping,
            29 => SkillName::Musicianship,
            30 => SkillName::Poisoning,
            31 => SkillName::Archery,
            32 => SkillName::SpiritSpeak,
            33 => SkillName::Stealing,
            34 => SkillName::Tailoring,
            35 => SkillName::AnimalTaming,
            36 => SkillName::TasteID,
            37 => SkillName::Tinkering,
            38 => SkillName::Tracking,
            39 => SkillName::Veterinary,
            40 => SkillName::Swords,
            41 => SkillName::Macing,
            42 => SkillName::Fencing,
            43 => SkillName::Wrestling,
            44 => SkillName::Lumberjacking,
            45 => SkillName::Mining,
            46 => SkillName::Meditation,
            47 => SkillName::Stealth,
            48 => SkillName::RemoveTrap,
            49 => SkillName::Necromancy,
            50 => SkillName::Focus,
            51 => SkillName::Chivalry,
            52 => SkillName::Bushido,
            53 => SkillName::Ninjitsu,
            54 => SkillName::Spellweaving,
            55 => SkillName::Mysticism,
            56 => SkillName::Imbuing,
            57 => SkillName::Throwing,
            _ => return None,
        })
    }
}
