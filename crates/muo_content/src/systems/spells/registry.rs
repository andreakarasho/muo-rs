//! The spell registry: spell ids to spell classes.

use indexmap::IndexMap;
use muo_primitives::dotnet::InvariantCase;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};

use super::special_move::SpecialMoveType;
use super::spell::{SpellId, SpellType, new_spell};

/// `m_Types` holds 700 entries.
pub const CAPACITY: usize = 700;

const CIRCLE_NAMES: [&str; 13] = [
    "First",
    "Second",
    "Third",
    "Fourth",
    "Fifth",
    "Sixth",
    "Seventh",
    "Eighth",
    "Necromancy",
    "Chivalry",
    "Bushido",
    "Ninjitsu",
    "Spellweaving",
];

/// A registered class: a spell, or a special move (both are in the one table, by id).
#[derive(Copy, Clone)]
pub enum Registered {
    Spell(&'static SpellType),
    Special(&'static SpecialMoveType),
}

impl Registered {
    pub fn full_name(self) -> &'static str {
        match self {
            Self::Spell(t) => t.full_name,
            Self::Special(t) => t.full_name,
        }
    }
}

/// The `SpellRegistry` statics.
pub struct SpellRegistry {
    types: Vec<Option<Registered>>,
    ids_from_types: IndexMap<&'static str, i32>,
    /// `SpecialMoves`.
    special_moves: IndexMap<i32, &'static SpecialMoveType>,
    /// The classes by lower-cased full name, for `NewSpell(name, ..)` (`AssemblyHandler.FindTypeByFullName`).
    by_name: IndexMap<String, Registered>,
}

impl Default for SpellRegistry {
    fn default() -> Self {
        Self {
            types: vec![None; CAPACITY],
            ids_from_types: IndexMap::new(),
            special_moves: IndexMap::new(),
            by_name: IndexMap::new(),
        }
    }
}

impl SpellRegistry {
    /// `Count`: how many ids are registered.
    pub fn count(&self) -> usize {
        self.types.iter().flatten().count()
    }

    /// `Types[id]`.
    pub fn type_of(&self, id: i32) -> Option<Registered> {
        usize::try_from(id)
            .ok()
            .and_then(|i| self.types.get(i).copied().flatten())
    }

    /// `GetRegistryNumber(type)`: -1 when unregistered.
    pub fn registry_number(&self, full_name: &str) -> i32 {
        self.ids_from_types.get(full_name).copied().unwrap_or(-1)
    }

    pub fn registry_number_of_special(&self, mv: &'static SpecialMoveType) -> i32 {
        self.registry_number(mv.full_name)
    }

    /// `SpecialMoves.Keys`.
    pub fn special_move_ids(&self) -> Vec<i32> {
        self.special_moves.keys().copied().collect()
    }

    /// `Register(spellID, type)`.
    pub fn register(&mut self, spell_id: i32, t: Registered) {
        let Some(slot) = usize::try_from(spell_id).ok().filter(|&i| i < CAPACITY) else {
            return;
        };
        self.types[slot] = Some(t);
        self.ids_from_types.entry(t.full_name()).or_insert(spell_id);
        self.by_name.insert(t.full_name().lower_inv(), t);
        if let Registered::Special(m) = t {
            self.special_moves.entry(spell_id).or_insert(m);
        }
    }

    /// `GetSpecialMove(spellID)`.
    pub fn special_move(&self, spell_id: i32) -> Option<&'static SpecialMoveType> {
        match self.type_of(spell_id)? {
            Registered::Special(_) => self.special_moves.get(&spell_id).copied(),
            Registered::Spell(_) => None,
        }
    }

    /// The class `NewSpell(string name, ..)` finds: `Server.Spells.<circle>.<name>Spell` (or `<name>` when it ends
    /// in "spell"), blanks removed, first circle that has it.
    pub fn find_by_name(&self, name: &str) -> Option<&'static SpellType> {
        let name = name.replace(' ', "");
        let ends_with_spell = name.lower_inv().ends_with("spell");
        for circle in CIRCLE_NAMES {
            let full = if ends_with_spell {
                format!("Server.Spells.{circle}.{name}")
            } else {
                format!("Server.Spells.{circle}.{name}Spell")
            };
            if let Some(Registered::Spell(t)) = self.by_name.get(&full.lower_inv()).copied() {
                return Some(t);
            }
        }
        None
    }
}

pub fn registry(w: &mut World) -> &mut SpellRegistry {
    w.resources.get_mut::<SpellRegistry>()
}

/// `GetRegistryNumber(ISpell s)`.
pub fn registry_number_of_spell(w: &mut World, s: SpellId) -> i32 {
    let Some(name) = super::spell::spell_type(w, s).map(|t| t.full_name) else {
        return -1;
    };
    registry(w).registry_number(name)
}

/// `NewSpell(spellID, caster, scroll)`: `None` when the id isn't a spell.
pub fn new_spell_by_id(
    w: &mut World,
    spell_id: i32,
    caster: MobileId,
    scroll: Option<ItemId>,
) -> Option<SpellId> {
    match registry(w).type_of(spell_id)? {
        Registered::Spell(t) => Some(new_spell(w, t, caster, scroll)),
        Registered::Special(_) => None,
    }
}

/// `NewSpell(name, caster, scroll)`.
pub fn new_spell_by_name(
    w: &mut World,
    name: &str,
    caster: MobileId,
    scroll: Option<ItemId>,
) -> Option<SpellId> {
    let t = registry(w).find_by_name(name)?;
    Some(new_spell(w, t, caster, scroll))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spells::spell::SPELL_BASE;

    static CLUMSY: SpellType = SpellType {
        full_name: "Server.Spells.First.ClumsySpell",
        parent: Some(&SPELL_BASE),
        ..SPELL_BASE
    };

    #[test]
    fn register_and_lookup() {
        let mut r = SpellRegistry::default();
        assert_eq!(r.registry_number("Server.Spells.First.ClumsySpell"), -1);
        r.register(0, Registered::Spell(&CLUMSY));
        r.register(-1, Registered::Spell(&CLUMSY));
        r.register(700, Registered::Spell(&CLUMSY));
        assert_eq!(r.count(), 1);
        assert_eq!(r.registry_number("Server.Spells.First.ClumsySpell"), 0);
        assert!(r.find_by_name("Clumsy").is_some());
        assert!(r.find_by_name("clumsy spell").is_some());
        assert!(r.find_by_name("ClumsySpell").is_some());
        assert!(r.find_by_name("Fireball").is_none());
        assert!(r.special_move(0).is_none());
    }

    #[test]
    fn new_spell_makes_an_instance() {
        let mut w = World::new();
        registry(&mut w).register(0, Registered::Spell(&CLUMSY));
        let caster = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        let s = new_spell_by_id(&mut w, 0, caster, None).unwrap();
        assert_eq!(super::super::spell::caster(&w, s), Some(caster));
        assert_eq!(registry_number_of_spell(&mut w, s), 0);
        assert!(new_spell_by_id(&mut w, 1, caster, None).is_none());
    }
}
