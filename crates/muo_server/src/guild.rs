//! Guilds (`BaseGuild`, `GuildType`).
//!
//! `BaseGuild` is abstract; content's `Guild` is the one subclass. The abstract members (`Abbreviation`,
//! `Name`, `Type`, `Disbanded`) are plain fields here and the rest is the [`GuildVt`] a subclass points at.
//! `World.Guilds` and `World.NewGuild` belong to the world; [`find_by_name`], [`find_by_abbrev`] and
//! [`search`] take the guilds as an iterator.

use indexmap::IndexSet;
use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use muo_primitives::{DateTime, Serial};

use crate::ext::Ext;
use crate::ids::MobileId;
use crate::world::World;
use muo_io::{GenericReader, GenericWriter};

/// A `BaseGuild` reference.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GuildId(pub u32);

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum GuildType {
    #[default]
    Regular = 0,
    Chaos = 1,
    Order = 2,
}

/// The abstract members of `BaseGuild` that are behavior.
pub struct GuildVt {
    pub delete: fn(&mut World, GuildId),
    pub serialize: fn(&World, GuildId, &mut dyn GenericWriter),
    pub deserialize: fn(&mut World, GuildId, &mut dyn GenericReader) -> muo_io::Result<()>,
    pub on_delete: fn(&mut World, GuildId, MobileId),
}

pub struct BaseGuild {
    pub serial: Serial,
    pub created: DateTime,
    pub abbreviation: String,
    pub name: String,
    pub ty: GuildType,
    /// `Disbanded`; `Deleted` is the same.
    pub disbanded: bool,
    pub vt: &'static GuildVt,
    pub ext: Ext,
}

impl BaseGuild {
    pub fn id(&self) -> GuildId {
        GuildId(self.serial.value())
    }

    /// `Deleted => Disbanded`.
    pub fn deleted(&self) -> bool {
        self.disbanded
    }

    /// `ToString`: `{Serial} "{Name} [{Abbreviation}]"`.
    pub fn display(&self) -> String {
        format!("{} \"{} [{}]\"", self.serial, self.name, self.abbreviation)
    }
}

/// `World.Guilds` (`GenericEntityPersistence<BaseGuild>("Guilds", 3, 1, 0x7FFFFFFF)`): the guilds by serial, in
/// load order. The persistence itself belongs to the guild class's crate, which fills this in.
#[derive(Default)]
pub struct Guilds {
    by_serial: indexmap::IndexMap<Serial, BaseGuild>,
    last_serial: u32,
}

const MAX_GUILD_SERIAL: u32 = 0x7FFF_FFFF;

impl Guilds {
    /// `GenericEntityPersistence.NewEntity`: the next free serial, wrapping to 1 past the maximum.
    fn new_serial(&mut self) -> Serial {
        let mut last = self.last_serial;
        for _ in 0..MAX_GUILD_SERIAL {
            last += 1;
            if last > MAX_GUILD_SERIAL {
                last = 1;
            }
            if !self.by_serial.contains_key(&Serial::new(last)) {
                self.last_serial = last;
                return Serial::new(last);
            }
        }
        Serial::MINUS_ONE
    }
}

/// `World.NewGuild`.
pub fn new_guild_serial(w: &mut World) -> Serial {
    w.resources.get_mut::<Guilds>().new_serial()
}

/// `World.AddGuild` (and the loader's insert of a guild read from the idx).
pub fn add_guild(w: &mut World, g: BaseGuild) -> GuildId {
    let gs = w.resources.get_mut::<Guilds>();
    gs.last_serial = gs.last_serial.max(g.serial.value());
    let id = g.id();
    gs.by_serial.insert(g.serial, g);
    id
}

/// `protected BaseGuild()`: takes `World.NewGuild`'s serial, stamps `Created = Core.Now`, and `World.AddGuild`s it.
/// The subclass fills `abbreviation`/`name`/`ty` through [`guild_mut`] (they are its abstract members).
pub fn create_guild(w: &mut World, vt: &'static GuildVt) -> GuildId {
    let serial = new_guild_serial(w);
    let g = BaseGuild {
        serial,
        created: w.now,
        abbreviation: String::new(),
        name: String::new(),
        ty: GuildType::Regular,
        disbanded: false,
        vt,
        ext: Ext::default(),
    };
    add_guild(w, g)
}

/// `World.RemoveGuild`.
pub fn remove_guild(w: &mut World, id: GuildId) {
    w.resources
        .get_mut::<Guilds>()
        .by_serial
        .shift_remove(&Serial::new(id.0));
}

/// `World.Guilds.Values`.
pub fn guild_ids(w: &World) -> Vec<GuildId> {
    w.resources.get::<Guilds>().map_or_else(Vec::new, |g| {
        g.by_serial.values().map(BaseGuild::id).collect()
    })
}

pub fn guild(w: &World, id: GuildId) -> Option<&BaseGuild> {
    w.resources
        .get::<Guilds>()?
        .by_serial
        .get(&Serial::new(id.0))
}

pub fn guild_mut(w: &mut World, id: GuildId) -> Option<&mut BaseGuild> {
    w.resources
        .get_mut::<Guilds>()
        .by_serial
        .get_mut(&Serial::new(id.0))
}

/// `World.FindGuild(serial)`.
pub fn find_guild(w: &World, serial: Serial) -> Option<GuildId> {
    w.resources
        .get::<Guilds>()?
        .by_serial
        .contains_key(&serial)
        .then(|| GuildId(serial.value()))
}

/// `BaseGuild.FindByName` over the world's guilds.
pub fn world_find_by_name(w: &World, name: &str) -> Option<GuildId> {
    let gs = w.resources.get::<Guilds>()?;
    find_by_name(gs.by_serial.values(), name).map(BaseGuild::id)
}

/// `BaseGuild.FindByAbbrev` over the world's guilds.
pub fn world_find_by_abbrev(w: &World, abbr: &str) -> Option<GuildId> {
    let gs = w.resources.get::<Guilds>()?;
    find_by_abbrev(gs.by_serial.values(), abbr).map(BaseGuild::id)
}

/// `BaseGuild.FindByName`.
pub fn find_by_name<'a>(
    guilds: impl IntoIterator<Item = &'a BaseGuild>,
    name: &str,
) -> Option<&'a BaseGuild> {
    guilds.into_iter().find(|g| g.name.eq_ic(name))
}

/// `BaseGuild.FindByAbbrev`.
pub fn find_by_abbrev<'a>(
    guilds: impl IntoIterator<Item = &'a BaseGuild>,
    abbr: &str,
) -> Option<&'a BaseGuild> {
    guilds.into_iter().find(|g| g.abbreviation.eq_ic(abbr))
}

/// `BaseGuild.Search`: the guilds whose name contains none of the words of `find`. The loop is inverted: `all` starts
/// true and a word that IS in the name clears it, so a guild matches when it contains none of the words. Kept
/// as is.
pub fn search<'a>(
    guilds: impl IntoIterator<Item = &'a BaseGuild>,
    find: &str,
) -> IndexSet<GuildId> {
    let mut results = IndexSet::new();
    let find = find.trim();
    if find.is_empty() {
        return results;
    }
    for g in guilds {
        let name = g.name.lower_inv();
        let mut all = true;
        for word in find.split(' ') {
            if word.is_empty() {
                continue;
            }
            if name.contains(&word.lower_inv()) {
                all = false;
                break;
            }
        }
        if all {
            results.insert(g.id());
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    static VT: GuildVt = GuildVt {
        delete: |_, _| {},
        serialize: |_, _, _| {},
        deserialize: |_, _, _| Ok(()),
        on_delete: |_, _, _| {},
    };

    fn g(serial: u32, name: &str, abbr: &str) -> BaseGuild {
        BaseGuild {
            serial: Serial::new(serial),
            created: DateTime::MIN,
            abbreviation: abbr.into(),
            name: name.into(),
            ty: GuildType::Regular,
            disbanded: false,
            vt: &VT,
            ext: Ext::NONE,
        }
    }

    #[test]
    fn finders() {
        let gs = [g(1, "Red Dragons", "RD"), g(2, "Blue Moon", "BM")];
        assert_eq!(find_by_name(&gs, "blue moon").unwrap().abbreviation, "BM");
        assert_eq!(find_by_abbrev(&gs, "rd").unwrap().name, "Red Dragons");
        // The inverted loop: a guild matches when it holds none of the words.
        let found = search(&gs, "dragons");
        assert_eq!(found.len(), 1);
        assert!(found.contains(&GuildId(2)));
        assert!(search(&gs, "  ").is_empty());
    }
}
