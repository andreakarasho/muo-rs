//! Poisons.
//!
//! `Poison` is abstract; each concrete poison (content's `Poison.Lesser` .. `Lethal`, `Darkglow`,
//! `Parasitic`) is a [`Poison`] value registered in [`Poisons`], which lives in the world's resources.

use muo_primitives::dotnet::IgnoreCase;
use muo_primitives::parse::FormatError;

use crate::ids::MobileId;
use crate::timer::TimerHandle;
use crate::world::World;

/// A `Poison` reference: its `Index` in `Poison.Poisons`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PoisonId(pub u8);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PoisonFamily {
    Standard,
    Darkglow,
    Parasitic,
}

pub struct Poison {
    pub index: i32,
    pub name: &'static str,
    pub level: i32,
    pub family: PoisonFamily,
    /// `ConstructTimer(Mobile)`: makes (does not start) the poison timer for `m`.
    pub construct_timer: fn(&mut World, MobileId) -> TimerHandle,
}

impl Poison {
    pub fn id(&self) -> PoisonId {
        PoisonId(self.index as u8)
    }
}

/// `Poison.Poisons` and `PoisonsByName` (case-insensitive).
#[derive(Default)]
pub struct Poisons {
    pub poisons: Vec<Poison>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum RegisterError {
    IndexExists,
    NameExists,
}

impl Poisons {
    /// `Poison.Register`. The name is checked only inside the loop over the existing poisons, so the first
    /// registration never checks it.
    pub fn register(&mut self, reg: Poison) -> Result<(), RegisterError> {
        for poison in &self.poisons {
            if reg.index == poison.index {
                return Err(RegisterError::IndexExists);
            }
            if self.get_poison(reg.name).is_some() {
                return Err(RegisterError::NameExists);
            }
        }
        self.poisons.push(reg);
        Ok(())
    }

    /// `Poison.GetPoisonByIndex`.
    pub fn get_poison_by_index(&self, index: i32) -> Option<&Poison> {
        usize::try_from(index)
            .ok()
            .and_then(|i| self.poisons.get(i))
    }

    /// `Poison.IncreaseLevel`: the next index, or the same poison at the top.
    pub fn increase_level(&self, old: Option<PoisonId>) -> Option<PoisonId> {
        let old = old?;
        Some(
            self.get_poison_by_index(old.0 as i32 + 1)
                .map_or(old, Poison::id),
        )
    }

    /// `Poison.GetPoison(name)`.
    pub fn get_poison(&self, name: &str) -> Option<&Poison> {
        self.poisons.iter().find(|p| p.name.eq_ic(name))
    }

    /// `Poison.Parse`.
    pub fn parse(&self, s: &str) -> Result<PoisonId, FormatError> {
        self.try_parse(s).ok_or(FormatError)
    }

    /// `Poison.TryParse`: an index, else a trimmed name.
    pub fn try_parse(&self, s: &str) -> Option<PoisonId> {
        if let Ok(index) = s.trim().parse::<i32>()
            && let Some(p) = self.get_poison_by_index(index)
        {
            return Some(p.id());
        }
        self.get_poison(s.trim()).map(Poison::id)
    }
}

/// The poison a [`PoisonId`] names.
pub fn find(w: &World, id: PoisonId) -> Option<&Poison> {
    // By `Index`, not by list position: content registers Darkglow at 10.. and Parasitic at 20.., and a reference
    // to a poison is its `Index`, so a saved/passed `Index` always means that poison.
    w.resources
        .get::<Poisons>()?
        .poisons
        .iter()
        .find(|p| p.index == i32::from(id.0))
}

/// `Poison.Level` of `id`, or -1 when unregistered.
pub fn level(w: &World, id: PoisonId) -> i32 {
    find(w, id).map_or(-1, |p| p.level)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(index: i32, name: &'static str) -> Poison {
        Poison {
            index,
            name,
            level: index,
            family: PoisonFamily::Standard,
            construct_timer: |_, _| TimerHandle::default(),
        }
    }

    #[test]
    fn register_and_parse() {
        let mut ps = Poisons::default();
        ps.register(p(0, "Lesser")).unwrap();
        ps.register(p(1, "Regular")).unwrap();
        assert_eq!(ps.register(p(1, "Other")), Err(RegisterError::IndexExists));
        assert_eq!(ps.register(p(2, "lesser")), Err(RegisterError::NameExists));
        assert_eq!(ps.try_parse(" 1 "), Some(PoisonId(1)));
        assert_eq!(ps.try_parse("REGULAR"), Some(PoisonId(1)));
        assert_eq!(ps.try_parse("nope"), None);
        assert_eq!(ps.increase_level(Some(PoisonId(1))), Some(PoisonId(1)));
        assert_eq!(ps.increase_level(Some(PoisonId(0))), Some(PoisonId(1)));
        assert_eq!(ps.increase_level(None), None);
    }
}
