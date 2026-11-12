//! Races.
//!
//! `Race` is abstract; each concrete race (Human, Elf, Gargoyle in content) is a [`Race`] value whose
//! abstract members are the [`RaceOps`] table it points at. The registry (`Race.Races`, `AllRaces`) is a
//! [`Races`] value in the world's resources.

use muo_data::expansion::Expansion;
use muo_primitives::dotnet::IgnoreCase;
use muo_primitives::{BuiltInRng, parse::FormatError};

use crate::ids::MobileId;
use crate::world::World;

/// A `Race` reference: its `RaceIndex`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct RaceId(pub u8);

pub const ALLOW_ALL_RACES: i32 = 0x7;
pub const ALLOW_HUMAN_OR_ELVES: i32 = 0x3;
pub const ALLOW_ELVES_ONLY: i32 = 0x2;
pub const ALLOW_GARGOYLES_ONLY: i32 = 0x4;

/// The abstract members of `Race`.
pub struct RaceOps {
    pub validate_hair: fn(female: bool, item_id: i32) -> bool,
    pub random_hair: fn(&mut BuiltInRng, female: bool) -> i32,
    pub validate_facial_hair: fn(female: bool, item_id: i32) -> bool,
    pub random_facial_hair: fn(&mut BuiltInRng, female: bool) -> i32,
    pub clip_skin_hue: fn(hue: i32) -> i32,
    pub random_skin_hue: fn(&mut BuiltInRng) -> i32,
    pub clip_hair_hue: fn(hue: i32) -> i32,
    pub random_hair_hue: fn(&mut BuiltInRng) -> i32,
}

pub struct Race {
    pub required_expansion: Expansion,
    pub male_body: i32,
    pub male_ghost_body: i32,
    pub female_body: i32,
    pub female_ghost_body: i32,
    pub race_id: i32,
    pub race_index: i32,
    /// `1 << raceIndex`.
    pub race_flag: i32,
    pub name: String,
    pub plural_name: String,
    pub ops: &'static RaceOps,
}

impl Race {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        race_id: i32,
        race_index: i32,
        name: &str,
        plural_name: &str,
        male_body: i32,
        female_body: i32,
        male_ghost_body: i32,
        female_ghost_body: i32,
        required_expansion: Expansion,
        ops: &'static RaceOps,
    ) -> Race {
        Race {
            required_expansion,
            male_body,
            male_ghost_body,
            female_body,
            female_ghost_body,
            race_id,
            race_index,
            race_flag: 1 << race_index,
            name: name.to_string(),
            plural_name: plural_name.to_string(),
            ops,
        }
    }

    pub fn id(&self) -> RaceId {
        RaceId(self.race_index as u8)
    }

    /// `Race.IsAllowedRace`.
    pub fn is_allowed_race(&self, allowed_race_flags: i32) -> bool {
        allowed_race_flags & self.race_flag != 0
    }

    /// `AliveBody(bool)`.
    pub fn alive_body_for(&self, female: bool) -> i32 {
        if female {
            self.female_body
        } else {
            self.male_body
        }
    }

    /// `GhostBody(bool)`.
    pub fn ghost_body_for(&self, female: bool) -> i32 {
        if female {
            self.female_ghost_body
        } else {
            self.male_ghost_body
        }
    }
}

/// `Race.Races` and `Race.AllRaces`.
pub struct Races {
    pub races: Vec<Option<Race>>,
    pub all_races: Vec<RaceId>,
}

impl Default for Races {
    fn default() -> Self {
        Races {
            races: (0..0x100).map(|_| None).collect(),
            all_races: Vec::new(),
        }
    }
}

impl Races {
    /// `Races[race.RaceIndex] = race` and `AllRaces.Add(race)` (what content's race classes do when they
    /// register).
    pub fn register(&mut self, race: Race) {
        let idx = race.race_index as usize;
        self.all_races.push(race.id());
        self.races[idx] = Some(race);
    }

    pub fn get(&self, id: RaceId) -> Option<&Race> {
        self.races.get(id.0 as usize)?.as_ref()
    }

    /// `Race.DefaultRace` / `Race.Human`: `Races[0]`.
    pub fn default_race(&self) -> Option<&Race> {
        self.races[0].as_ref()
    }

    /// `Race.Parse`: an index, else a trimmed name or plural name, case-insensitively.
    // Null slots of `Races` are skipped while matching names
    // and an unknown name is an Err.
    pub fn parse(&self, s: &str) -> Result<RaceId, FormatError> {
        self.try_parse(s).ok_or(FormatError)
    }

    /// `Race.TryParse`.
    pub fn try_parse(&self, s: &str) -> Option<RaceId> {
        if let Ok(index) = s.trim().parse::<i32>()
            && index >= 0
            && (index as usize) < self.races.len()
        {
            return self.races[index as usize].as_ref().map(Race::id);
        }
        let s = s.trim();
        self.races
            .iter()
            .flatten()
            .find(|r| r.name.eq_ic(s) || r.plural_name.eq_ic(s))
            .map(Race::id)
    }
}

fn female_of(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|m| m.female)
}

/// `Race.Body(Mobile)`.
pub fn body(w: &World, race: &Race, m: MobileId) -> i32 {
    let alive = w.mobile(m).is_some_and(|mm| (mm.ty.alive)(w, m));
    if alive {
        race.alive_body_for(female_of(w, m))
    } else {
        race.ghost_body_for(female_of(w, m))
    }
}

/// `Race.AliveBody(Mobile)`.
pub fn alive_body(w: &World, race: &Race, m: MobileId) -> i32 {
    race.alive_body_for(female_of(w, m))
}

/// `Race.GhostBody(Mobile)`.
pub fn ghost_body(w: &World, race: &Race, m: MobileId) -> i32 {
    race.ghost_body_for(female_of(w, m))
}

/// `Race.ValidateHair(Mobile, itemID)`.
pub fn validate_hair(w: &World, race: &Race, m: MobileId, item_id: i32) -> bool {
    (race.ops.validate_hair)(female_of(w, m), item_id)
}

/// `Race.RandomHair(Mobile)`.
pub fn random_hair(w: &mut World, race: &Race, m: MobileId) -> i32 {
    let f = female_of(w, m);
    (race.ops.random_hair)(&mut w.rng, f)
}

/// `Race.ValidateFacialHair(Mobile, itemID)`.
pub fn validate_facial_hair(w: &World, race: &Race, m: MobileId, item_id: i32) -> bool {
    (race.ops.validate_facial_hair)(female_of(w, m), item_id)
}

/// `Race.RandomFacialHair(Mobile)`.
pub fn random_facial_hair(w: &mut World, race: &Race, m: MobileId) -> i32 {
    let f = female_of(w, m);
    (race.ops.random_facial_hair)(&mut w.rng, f)
}
