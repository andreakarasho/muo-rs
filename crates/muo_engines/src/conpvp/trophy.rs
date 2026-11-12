//! The `Trophy` item (title, rank, owner, date), flippable 5020/4647.

use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::PropType;
use muo_primitives::DateTime;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

/// `TrophyRank`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum TrophyRank {
    Bronze = 0,
    Silver = 1,
    Gold = 2,
}

impl TrophyRank {
    pub fn from_i32(v: i32) -> TrophyRank {
        match v {
            1 => TrophyRank::Silver,
            2 => TrophyRank::Gold,
            _ => TrophyRank::Bronze,
        }
    }

    /// `GetRankName(rank)`: anything but silver and gold is bronze.
    fn name(self) -> &'static str {
        match self {
            TrophyRank::Silver => "silver",
            TrophyRank::Gold => "gold",
            TrophyRank::Bronze => "bronze",
        }
    }

    /// `UpdateStyle()`'s hue.
    pub fn hue(self) -> i32 {
        match self {
            TrophyRank::Gold => 2213,
            TrophyRank::Silver => 0,
            TrophyRank::Bronze => 2206,
        }
    }
}

/// The fields of a `Trophy`.
#[derive(Clone, Debug)]
pub struct TrophyData {
    pub title: Option<String>,
    pub rank: TrophyRank,
    pub owner: Option<MobileId>,
    pub date: DateTime,
}

fn data(w: &World, id: ItemId) -> Option<&TrophyData> {
    w.item(id)?.ext.get::<TrophyData>()
}

/// `RawName` of a mobile.
fn raw_name(w: &World, m: MobileId) -> String {
    w.mobile(m).and_then(|x| x.name.clone()).unwrap_or_default()
}

/// `Date.ToString("d")`: the short date, month/day/year.
fn short_date(d: DateTime) -> String {
    format!("{}/{}/{}", d.month(), d.day(), d.year())
}

/// The lines of `GetProperties` / `OnSingleClick`: the owner line or the title, then the date.
fn lines(w: &World, id: ItemId) -> Vec<String> {
    let Some(d) = data(w, id) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let title = d.title.clone().unwrap_or_default();
    if let Some(o) = d.owner {
        out.push(format!("{title} -- {}", raw_name(w, o)));
    } else if d.title.is_some() {
        out.push(title);
    }
    if d.date != DateTime::MIN {
        out.push(short_date(d.date));
    }
    out
}

/// `new Trophy(title, rank)`.
pub fn new_trophy(w: &mut World, title: Option<String>, rank: TrophyRank) -> ItemId {
    let id = ops::new_item_with_id(w, &TROPHY, 5020);
    let now = w.now;
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(TrophyData {
            title,
            rank,
            owner: None,
            date: now,
        });
    }
    ops::set_loot_type(w, id, LootType::Blessed);
    (TROPHY.set_hue)(w, id, rank.hue()); // UpdateStyle()
    id
}

/// `Trophy`.
pub static TROPHY: ItemType = ItemType {
    full_name: "Server.Items.Trophy",
    construct: None,
    parent: Some(&ITEM_BASE),
    default_name: |w, id| {
        let rank = data(w, id).map_or(TrophyRank::Bronze, |d| d.rank);
        Some(format!("{} trophy", rank.name()))
    },
    on_added: |w, id, parent| {
        (ITEM_BASE.on_added)(w, id, parent);
        // `Owner ??= RootParent as Mobile`
        if data(w, id).is_some_and(|d| d.owner.is_none())
            && let Some(EntityId::Mobile(m)) = ops::root_parent(w, id)
            && let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<TrophyData>())
        {
            d.owner = Some(m);
        }
    },
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        for l in lines(w, id) {
            list.add_text(&l);
        }
    },
    on_single_click: |w, id, from| {
        (ITEM_BASE.on_single_click)(w, id, from);
        for l in lines(w, id) {
            ops::label_to(w, id, from, &l);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(2);
        let d = data(w, id);
        out.write_opt_string(d.and_then(|d| d.title.as_deref()));
        out.write_i32(d.map_or(0, |d| d.rank as i32));
        out.write_mobile(w, d.and_then(|d| d.owner));
        out.write_date_time_ticks(d.map_or(DateTime::MIN, |d| d.date).ticks());
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?; // versions 0 and 1 are read by the same migration, with the same layout
        let title = r.read_string()?;
        let rank = TrophyRank::from_i32(r.read_i32()?);
        let owner = r.read_mobile(w)?;
        let date = DateTime::from_ticks(r.read_date_time_ticks()?);
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(TrophyData {
                title,
                rank,
                owner,
                date,
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

static RANK_NAMES: &[(&str, i64)] = &[("Bronze", 0), ("Silver", 1), ("Gold", 2)];

static TROPHY_PARAMS: [Param; 2] = [
    Param {
        name: "title",
        kind: Kind::Plain(PropType::Str),
        default: None,
        ty: "",
    },
    Param {
        name: "rank",
        kind: Kind::Plain(PropType::Enum(RANK_NAMES)),
        default: None,
        ty: "TrophyRank",
    },
];

/// `Trophy(string title, TrophyRank rank)`.
static TROPHY_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &TROPHY_PARAMS,
    make: |w, v| {
        let title = match &v[0] {
            Value::Str(s) => Some(s.clone()),
            _ => None,
        };
        EntityId::Item(new_trophy(
            w,
            title,
            TrophyRank::from_i32(v[1].i64() as i32),
        ))
    },
}];

/// Registers the item class.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&TROPHY, &[]);
}

/// The `[Constructible]` overload.
pub fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.Trophy", &TROPHY_CTORS);
}
