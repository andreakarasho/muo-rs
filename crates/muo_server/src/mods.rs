//! Mobile mods: `MobileMod`, `StatMod`, `ResistanceMod`, `SkillMod`, `DefaultSkillMod`, `EquippedSkillMod`,
//! `TimedSkillMod`.
//!
//! A mod is a value in its owner's list (`Mobile::stat_mods` ...) and is named by its [`ModId`]; code that holds
//! a mod to remove it later holds the id. `owner` is set by `Mobile.Add*Mod`. A mod's setters that
//! tell the owner (`ResistanceMod.Type`, `SkillMod.Value`...) are the functions here taking the world.
//!
//! // shortcut: the mods have no Serialize/Deserialize: `Mobile.Serialize`
//! never writes them, so nothing reads those bytes.

use muo_primitives::{DateTime, TimeSpan};

use crate::ids::{ItemId, MobileId};
use crate::mobile::{ResistanceType, StatType};
use crate::skills::SkillName;
use crate::timer::TimerHandle;
use crate::world::World;

/// Identity of one mod object (reference equality).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ModId(pub u32);

/// The counter behind [`ModId`]s; lives in `World::resources`.
#[derive(Default)]
struct ModCounter(u32);

fn next_mod_id(w: &mut World) -> ModId {
    let c = w.resources.get_mut::<ModCounter>();
    c.0 += 1;
    ModId(c.0)
}

/// `StatMod`.
#[derive(Clone, Debug)]
pub struct StatMod {
    pub id: ModId,
    /// `MobileMod.Owner`.
    pub owner: Option<MobileId>,
    /// `MobileMod.Name`.
    pub name: Option<String>,
    pub added: DateTime,
    pub duration: TimeSpan,
    pub ty: StatType,
    pub offset: i32,
    /// `_timerToken`.
    pub timer: TimerHandle,
}

impl StatMod {
    /// `new StatMod(type, name, offset, duration, owner)`: starts the expiry timer when `duration > 0`.
    pub fn new(
        w: &mut World,
        ty: StatType,
        name: Option<&str>,
        offset: i32,
        duration: TimeSpan,
        owner: Option<MobileId>,
    ) -> StatMod {
        let id = next_mod_id(w);
        let timer = if duration > TimeSpan::ZERO {
            start_expiry(w, id, duration)
        } else {
            TimerHandle::default()
        };
        StatMod {
            id,
            owner,
            name: name.map(str::to_owned),
            added: w.now,
            duration,
            ty,
            offset,
            timer,
        }
    }

    /// `HasElapsed`.
    pub fn has_elapsed(&self, now: DateTime) -> bool {
        self.duration > TimeSpan::ZERO && now - self.added >= self.duration
    }

    /// `StatMod.Remove`, called by `Mobile.RemoveStatMod`: cancels the expiry timer.
    pub fn remove(&mut self, w: &mut World) {
        self.timer.cancel(w);
    }

    /// `AfterDeserialization`: restarts the expiry timer of a mod that has time left.
    pub fn after_deserialization(&mut self, w: &mut World) {
        if self.duration > TimeSpan::ZERO && w.now - self.added < self.duration {
            self.timer = start_expiry(w, self.id, self.duration);
        }
    }
}

/// `Timer.StartTimer(duration, RemoveFromOwner)`: `Owner?.RemoveStatMod(this)`.
fn start_expiry(w: &mut World, id: ModId, duration: TimeSpan) -> TimerHandle {
    crate::timer::start(w, duration, TimeSpan::ZERO, 1, move |w| {
        // shortcut: the owner is found by scanning mobiles, as the closure can't borrow the mod; fine while
        // timed stat mods are few, index them by id if that changes.
        let owner = w
            .mobiles
            .values()
            .find(|m| m.stat_mods.iter().any(|s| s.id == id))
            .map(|m| (m.id(), m.ty));
        if let Some((owner, ty)) = owner {
            (ty.remove_stat_mod)(w, owner, id);
        }
    })
}

/// `ResistanceMod`.
#[derive(Clone, Debug)]
pub struct ResistanceMod {
    pub id: ModId,
    pub owner: Option<MobileId>,
    pub name: Option<String>,
    pub ty: ResistanceType,
    pub offset: i32,
}

impl ResistanceMod {
    /// `new ResistanceMod(type, name, offset, owner)`.
    pub fn new(
        w: &mut World,
        ty: ResistanceType,
        name: Option<&str>,
        offset: i32,
        owner: Option<MobileId>,
    ) -> ResistanceMod {
        ResistanceMod {
            id: next_mod_id(w),
            owner,
            name: name.map(str::to_owned),
            ty,
            offset,
        }
    }
}

fn find_resistance_mod(w: &mut World, owner: MobileId, id: ModId) -> Option<&mut ResistanceMod> {
    w.mobile_mut(owner)?
        .resistance_mods
        .iter_mut()
        .find(|r| r.id == id)
}

fn update_resistances(w: &mut World, owner: MobileId) {
    if let Some(ty) = w.mobile(owner).map(|m| m.ty) {
        (ty.update_resistances)(w, owner);
    }
}

/// `ResistanceMod.Type = value` on a mod that is in its owner's list: `Owner?.UpdateResistances()`.
pub fn set_resistance_mod_type(w: &mut World, owner: MobileId, id: ModId, ty: ResistanceType) {
    if let Some(m) = find_resistance_mod(w, owner, id) {
        m.ty = ty;
        update_resistances(w, owner);
    }
}

/// `ResistanceMod.Offset = value` on a mod that is in its owner's list.
pub fn set_resistance_mod_offset(w: &mut World, owner: MobileId, id: ModId, offset: i32) {
    if let Some(m) = find_resistance_mod(w, owner, id) {
        m.offset = offset;
        update_resistances(w, owner);
    }
}

/// What `CheckCondition` is, per subclass.
#[derive(Clone, Debug)]
pub enum SkillModKind {
    /// `DefaultSkillMod`: always true.
    Default,
    /// `EquippedSkillMod`: the item is alive and worn by the owner.
    Equipped(ItemId),
    /// `TimedSkillMod`: until `expire`.
    Timed(DateTime),
}

/// `SkillMod` and its three subclasses.
#[derive(Clone, Debug)]
pub struct SkillMod {
    pub id: ModId,
    pub owner: Option<MobileId>,
    pub name: Option<String>,
    pub obey_cap: bool,
    pub skill: SkillName,
    pub relative: bool,
    pub value: f64,
    pub kind: SkillModKind,
}

impl SkillMod {
    fn make(
        w: &mut World,
        skill: SkillName,
        name: Option<&str>,
        relative: bool,
        value: f64,
        owner: Option<MobileId>,
        kind: SkillModKind,
    ) -> SkillMod {
        SkillMod {
            id: next_mod_id(w),
            owner,
            name: name.map(str::to_owned),
            obey_cap: false,
            skill,
            relative,
            value,
            kind,
        }
    }

    /// `new DefaultSkillMod(skill, name, relative, value)`.
    pub fn new_default(
        w: &mut World,
        skill: SkillName,
        name: Option<&str>,
        relative: bool,
        value: f64,
    ) -> SkillMod {
        Self::make(w, skill, name, relative, value, None, SkillModKind::Default)
    }

    /// `new EquippedSkillMod(skill, name, relative, value, item, owner)`.
    pub fn new_equipped(
        w: &mut World,
        skill: SkillName,
        name: Option<&str>,
        relative: bool,
        value: f64,
        item: ItemId,
        owner: MobileId,
    ) -> SkillMod {
        let kind = SkillModKind::Equipped(item);
        Self::make(w, skill, name, relative, value, Some(owner), kind)
    }

    /// `new TimedSkillMod(skill, name, relative, value, delay)`: expires `delay` after `Core.Now`.
    pub fn new_timed(
        w: &mut World,
        skill: SkillName,
        name: Option<&str>,
        relative: bool,
        value: f64,
        delay: TimeSpan,
    ) -> SkillMod {
        let expire = w.now + delay;
        Self::new_timed_until(w, skill, name, relative, value, expire)
    }

    /// `new TimedSkillMod(skill, name, relative, value, expire)`.
    pub fn new_timed_until(
        w: &mut World,
        skill: SkillName,
        name: Option<&str>,
        relative: bool,
        value: f64,
        expire: DateTime,
    ) -> SkillMod {
        let kind = SkillModKind::Timed(expire);
        Self::make(w, skill, name, relative, value, None, kind)
    }

    /// `Absolute`.
    pub fn absolute(&self) -> bool {
        !self.relative
    }

    /// `CheckCondition()`.
    pub fn check_condition(&self, w: &World) -> bool {
        match self.kind {
            SkillModKind::Default => true,
            SkillModKind::Equipped(item) => {
                // `_item.Deleted` on a freed item reads as deleted.
                let Some(i) = w.item(item) else { return false };
                let owner_live = self
                    .owner
                    .and_then(|o| w.mobile(o))
                    .is_some_and(|m| !m.deleted);
                !i.deleted() && owner_live && i.parent == self.owner.map(Into::into)
            }
            SkillModKind::Timed(expire) => w.now < expire,
        }
    }

    /// `Remove()`: `Owner?.RemoveSkillMod(this)`.
    pub fn remove(&self, w: &mut World) {
        if let Some(owner) = self.owner
            && let Some(ty) = w.mobile(owner).map(|m| m.ty)
        {
            (ty.remove_skill_mod)(w, owner, self.id);
        }
    }
}

fn find_skill_mod(w: &mut World, owner: MobileId, id: ModId) -> Option<&mut SkillMod> {
    w.mobile_mut(owner)?
        .skill_mods
        .iter_mut()
        .find(|s| s.id == id)
}

/// `SkillMod.ObeyCap = value` on a mod in its owner's list.
pub fn set_skill_mod_obey_cap(w: &mut World, owner: MobileId, id: ModId, value: bool) {
    if let Some(m) = find_skill_mod(w, owner, id) {
        m.obey_cap = value;
        let skill = m.skill;
        crate::skills::update(w, owner, skill);
    }
}

/// `SkillMod.Skill = value`: updates the new skill, then the old one.
pub fn set_skill_mod_skill(w: &mut World, owner: MobileId, id: ModId, value: SkillName) {
    if let Some(m) = find_skill_mod(w, owner, id) {
        let old = std::mem::replace(&mut m.skill, value);
        crate::skills::update(w, owner, value);
        crate::skills::update(w, owner, old);
    }
}

/// `SkillMod.Relative = value`.
pub fn set_skill_mod_relative(w: &mut World, owner: MobileId, id: ModId, value: bool) {
    if let Some(m) = find_skill_mod(w, owner, id) {
        m.relative = value;
        let skill = m.skill;
        crate::skills::update(w, owner, skill);
    }
}

/// `SkillMod.Value = value`.
pub fn set_skill_mod_value(w: &mut World, owner: MobileId, id: ModId, value: f64) {
    if let Some(m) = find_skill_mod(w, owner, id) {
        m.value = value;
        let skill = m.skill;
        crate::skills::update(w, owner, skill);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timed_mod_expires() {
        let mut w = World::new();
        let m = SkillMod::new_timed(
            &mut w,
            SkillName::Magery,
            Some("t"),
            true,
            5.0,
            TimeSpan::from_secs(10),
        );
        assert!(m.check_condition(&w));
        w.now += TimeSpan::from_secs(10);
        assert!(!m.check_condition(&w));
    }

    #[test]
    fn stat_mod_elapsed() {
        let mut w = World::new();
        let m = StatMod::new(
            &mut w,
            StatType::STR,
            Some("s"),
            5,
            TimeSpan::from_secs(3),
            None,
        );
        assert!(!m.has_elapsed(w.now));
        assert!(m.has_elapsed(w.now + TimeSpan::from_secs(3)));
        let forever = StatMod::new(&mut w, StatType::DEX, None, 1, TimeSpan::ZERO, None);
        assert!(!forever.has_elapsed(w.now + TimeSpan::from_days(9)));
        assert_ne!(m.id, forever.id);
    }
}
