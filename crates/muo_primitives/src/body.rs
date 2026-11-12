//! Body ids and the body type table.
//!
//! The `Data/bodyTable.cfg` table is a value built with
//! [`BodyTable::parse`] (the caller reads the file), and the id-only questions live on [`Body`].

use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
#[repr(u8)]
pub enum BodyType {
    #[default]
    Empty,
    Monster,
    Sea,
    Animal,
    Human,
    Equipment,
}

impl BodyType {
    /// `Enum.TryParse(name, ignoreCase: true)`, which also accepts a number.
    pub fn parse(s: &str) -> Option<BodyType> {
        let s = s.trim();
        const ALL: [(&str, BodyType); 6] = [
            ("empty", BodyType::Empty),
            ("monster", BodyType::Monster),
            ("sea", BodyType::Sea),
            ("animal", BodyType::Animal),
            ("human", BodyType::Human),
            ("equipment", BodyType::Equipment),
        ];
        if let Ok(n) = s.parse::<u8>() {
            return ALL.get(usize::from(n)).map(|e| e.1);
        }
        ALL.iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(s))
            .map(|e| e.1)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, Debug)]
pub struct Body(pub i32);

impl Body {
    pub const fn new(body_id: i32) -> Body {
        Body(body_id)
    }

    pub const fn body_id(self) -> i32 {
        self.0
    }

    pub const fn is_gargoyle(self) -> bool {
        matches!(self.0, 666 | 667 | 694 | 695)
    }

    pub const fn is_male(self) -> bool {
        matches!(self.0, 183 | 185 | 400 | 402 | 605 | 607 | 666 | 694 | 750)
    }

    pub const fn is_female(self) -> bool {
        matches!(self.0, 184 | 186 | 401 | 403 | 606 | 608 | 667 | 695 | 751)
    }

    pub const fn is_ghost(self) -> bool {
        matches!(self.0, 402 | 403 | 607 | 608 | 694 | 695 | 970)
    }
}

impl From<i32> for Body {
    fn from(id: i32) -> Body {
        Body(id)
    }
}

impl From<Body> for i32 {
    fn from(b: Body) -> i32 {
        b.0
    }
}

/// `0xID`
impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}

/// The body id -> type table (`Body.m_Types`).
#[derive(Clone, Debug, Default)]
pub struct BodyTable {
    types: Vec<BodyType>,
}

impl BodyTable {
    const SIZE: usize = 0x1000;

    /// Parses `Data/bodyTable.cfg`: tab separated `id<TAB>type`, `#` comments. Returns the table and the
    /// invalid lines (for the caller to log as "Invalid bodyTable entry").
    pub fn parse(text: &str) -> (BodyTable, Vec<String>) {
        let mut types = vec![BodyType::Empty; BodyTable::SIZE];
        let mut invalid = Vec::new();
        for line in text.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut split = line.split('\t');
            let entry = (|| {
                let id = split.next()?.trim().parse::<i32>().ok()?;
                let ty = BodyType::parse(split.next()?)?;
                usize::try_from(id)
                    .ok()
                    .filter(|i| *i < BodyTable::SIZE)
                    .map(|i| (i, ty))
            })();
            match entry {
                Some((i, ty)) => types[i] = ty,
                None => invalid.push(line.to_string()),
            }
        }
        (BodyTable { types }, invalid)
    }

    /// `Body.Type`; `Empty` for an id outside the table (or with no table loaded).
    pub fn type_of(&self, body: Body) -> BodyType {
        usize::try_from(body.0)
            .ok()
            .and_then(|i| self.types.get(i))
            .copied()
            .unwrap_or(BodyType::Empty)
    }

    fn is(&self, body: Body, ty: BodyType) -> bool {
        usize::try_from(body.0).is_ok_and(|i| i < self.types.len()) && self.type_of(body) == ty
    }

    pub fn is_human(&self, body: Body) -> bool {
        self.is(body, BodyType::Human) && !matches!(body.0, 402 | 403 | 607 | 608 | 694 | 695 | 970)
    }

    pub fn is_monster(&self, body: Body) -> bool {
        self.is(body, BodyType::Monster)
    }

    pub fn is_animal(&self, body: Body) -> bool {
        self.is(body, BodyType::Animal)
    }

    /// True only for an id inside a loaded table whose type is `Empty`.
    pub fn is_empty(&self, body: Body) -> bool {
        self.is(body, BodyType::Empty)
    }

    pub fn is_sea(&self, body: Body) -> bool {
        self.is(body, BodyType::Sea)
    }

    pub fn is_equipment(&self, body: Body) -> bool {
        self.is(body, BodyType::Equipment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_predicates() {
        assert!(
            Body(400).is_male()
                && Body(401).is_female()
                && Body(402).is_ghost()
                && Body(402).is_male()
        );
        assert!(Body(666).is_gargoyle() && !Body(400).is_gargoyle());
        assert_eq!(Body(0x190).to_string(), "0x190");
    }

    #[test]
    fn table() {
        let (t, bad) = BodyTable::parse(
            "# comment\n\n400\tHuman\n402\tHuman\n17\tmonster\nxyz\tHuman\n5000\tSea\n9\tbogus\n",
        );
        assert_eq!(bad, ["xyz\tHuman", "5000\tSea", "9\tbogus"]);
        assert!(t.is_human(Body(400)) && !t.is_human(Body(402)));
        assert!(t.is_monster(Body(17)) && !t.is_animal(Body(17)));
        assert_eq!(t.type_of(Body(-1)), BodyType::Empty);
        assert!(!t.is_empty(Body(-1)) && t.is_empty(Body(3)));
        assert!(!BodyTable::default().is_empty(Body(3)));
    }
}
