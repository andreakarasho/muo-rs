//! The creature constructor statements a mobile template can hold. Each step calls the same setter the Rust
//! constructors call (`creature::setters`, `creature::init_ops`, `creature::packing`).
//!
//! A value written `n` is a fixed value; `[min, max]` is a random value in that range (the `*_range` setter).

use serde_json::Value;

use muo_server::ids::MobileId;
use muo_server::mobile::ResistanceType;
use muo_server::skills::SkillName;
use muo_server::world::World;

use super::keys::{KeyDef, Keys};
use super::{Flow, choose, parse_enum, parse_flow};
use crate::creature::{init_ops as ops, packing, setters};
use crate::hooks::rnd;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IRange {
    Fixed(i32),
    Range(i32, i32),
}

impl IRange {
    /// The value: a fixed one, or `rnd::min_max` drawn now.
    pub fn roll(self, w: &mut World) -> i32 {
        match self {
            Self::Fixed(v) => v,
            Self::Range(a, b) => rnd::min_max(w, a, b),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FRange {
    Fixed(f64),
    Range(f64, f64),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PackWhat {
    /// `PackItem(new T())`, `new T(amount)`, `{ ItemID = item_id }`, or `AddItem` (equipped) with an optional hue.
    Item {
        name: String,
        amount: Option<IRange>,
        hue: Option<i32>,
        item_id: Option<i32>,
        equip: bool,
    },
    Gold(IRange),
    Reg(IRange),
    Gem(IRange),
    NecroReg(IRange),
    ArcaneScroll(IRange),
    Scroll(IRange),
    Statue(IRange),
    Potion,
    /// `PackSlayer(chance)`.
    Slayer(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PackEntry {
    /// `if (Utility.RandomDouble() < chance)` around the statement.
    pub chance: Option<f64>,
    pub what: PackWhat,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    Name(String),
    NameRandom(String),
    Title(String),
    Body(i32),
    Bodies(Vec<i32>),
    Hue(i32),
    BaseSoundId(i32),
    SpeechHue(i32),
    Str(IRange),
    Dex(IRange),
    Int(IRange),
    Hits(IRange),
    Stam(IRange),
    Mana(IRange),
    Damage(i32, i32),
    DamageTypes(Vec<(ResistanceType, IRange)>),
    Resists(Vec<(ResistanceType, IRange)>),
    Skills(Vec<(SkillName, FRange)>),
    SkillBase(Vec<(SkillName, f64)>),
    Fame(i32),
    Karma(i32),
    VirtualArmor(i32),
    Tamable(bool),
    ControlSlots(i32),
    MinTameSkill(f64),
    Female(bool),
    CanSwim(bool),
    CantWalk(bool),
    Paragon(bool),
    RandomHair,
    Pack(Vec<PackEntry>),
    Flow(Box<Flow<Step>>),
    /// A key a content crate registered (`templates::keys`).
    Ext(&'static str, KeyDef<MobileId>, Value),
}

fn int(v: &Value) -> Result<i32, String> {
    v.as_i64()
        .and_then(|n| i32::try_from(n).ok())
        .ok_or_else(|| format!("expected an integer, got {v}"))
}

fn float(v: &Value) -> Result<f64, String> {
    v.as_f64()
        .ok_or_else(|| format!("expected a number, got {v}"))
}

fn boolean(v: &Value) -> Result<bool, String> {
    v.as_bool()
        .ok_or_else(|| format!("expected true or false, got {v}"))
}

fn string(v: &Value) -> Result<String, String> {
    v.as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("expected a string, got {v}"))
}

fn pair<T>(v: &Value, f: impl Fn(&Value) -> Result<T, String>) -> Option<Result<(T, T), String>> {
    let a = v.as_array()?;
    if a.len() != 2 {
        return Some(Err(format!("expected [min, max], got {v}")));
    }
    Some(f(&a[0]).and_then(|x| Ok((x, f(&a[1])?))))
}

pub fn irange(v: &Value) -> Result<IRange, String> {
    match pair(v, int) {
        Some(r) => r.map(|(a, b)| IRange::Range(a, b)),
        None => int(v).map(IRange::Fixed),
    }
}

fn frange(v: &Value) -> Result<FRange, String> {
    match pair(v, float) {
        Some(r) => r.map(|(a, b)| FRange::Range(a, b)),
        None => float(v).map(FRange::Fixed),
    }
}

pub fn resistance(s: &str) -> Result<ResistanceType, String> {
    match s {
        "Physical" => Ok(ResistanceType::Physical),
        "Fire" => Ok(ResistanceType::Fire),
        "Cold" => Ok(ResistanceType::Cold),
        "Poison" => Ok(ResistanceType::Poison),
        "Energy" => Ok(ResistanceType::Energy),
        _ => Err(format!("unknown resistance {s}")),
    }
}

pub fn skill(s: &str) -> Result<SkillName, String> {
    parse_enum(s, SkillName::from_i32).ok_or_else(|| format!("unknown skill {s}"))
}

fn map<K, V>(
    v: &Value,
    k: impl Fn(&str) -> Result<K, String>,
    f: impl Fn(&Value) -> Result<V, String>,
) -> Result<Vec<(K, V)>, String> {
    let o = v
        .as_object()
        .ok_or_else(|| format!("expected an object, got {v}"))?;
    o.iter().map(|(key, val)| Ok((k(key)?, f(val)?))).collect()
}

fn pack_entry(v: &Value) -> Result<PackEntry, String> {
    let o = v
        .as_object()
        .ok_or_else(|| format!("expected a pack object, got {v}"))?;
    let mut chance = None;
    let mut what = None;
    let (mut amount, mut hue, mut item_id, mut equip) = (None, None, None, false);
    let mut set = |w: PackWhat| {
        if what.replace(w).is_some() {
            return Err("a pack entry names more than one thing".to_owned());
        }
        Ok(())
    };
    for (k, val) in o {
        match k.as_str() {
            "chance" => chance = Some(float(val)?),
            "amount" => amount = Some(irange(val)?),
            "hue" => hue = Some(int(val)?),
            "itemId" => item_id = Some(int(val)?),
            "equip" => equip = boolean(val)?,
            "item" => set(PackWhat::Item {
                name: string(val)?,
                amount: None,
                hue: None,
                item_id: None,
                equip: false,
            })?,
            "gold" => set(PackWhat::Gold(irange(val)?))?,
            "reg" => set(PackWhat::Reg(irange(val)?))?,
            "gem" => set(PackWhat::Gem(irange(val)?))?,
            "necroReg" => set(PackWhat::NecroReg(irange(val)?))?,
            "arcaneScroll" => set(PackWhat::ArcaneScroll(irange(val)?))?,
            "scroll" => set(PackWhat::Scroll(irange(val)?))?,
            "statue" => set(PackWhat::Statue(irange(val)?))?,
            "potion" => set(PackWhat::Potion)?,
            "slayer" => set(PackWhat::Slayer(float(val)?))?,
            _ => return Err(format!("unknown pack key \"{k}\"")),
        }
    }
    let mut what = what.ok_or("a pack entry needs item, gold, reg, gem, necroReg, arcaneScroll, scroll, statue, potion or slayer")?;
    match &mut what {
        PackWhat::Item {
            amount: a,
            hue: h,
            item_id: i,
            equip: e,
            ..
        } => (*a, *h, *i, *e) = (amount, hue, item_id, equip),
        _ if amount.is_some() || hue.is_some() || item_id.is_some() || equip => {
            return Err("amount, hue, itemId and equip only go with \"item\"".to_owned());
        }
        _ => {}
    }
    Ok(PackEntry { chance, what })
}

/// One `"key": value` of a mobile template.
pub fn parse(key: &str, v: &Value, keys: &Keys<'_>) -> Result<Step, String> {
    if let Some(f) = parse_flow(key, v, &|k: &str, v: &Value| parse(k, v, keys)) {
        return f.map(|f| Step::Flow(Box::new(f)));
    }
    Ok(match key {
        "name" => Step::Name(string(v)?),
        "nameRandom" => Step::NameRandom(string(v)?),
        "title" => Step::Title(string(v)?),
        "body" => Step::Body(int(v)?),
        "bodies" => Step::Bodies(
            v.as_array()
                .ok_or("expected an array of bodies")?
                .iter()
                .map(int)
                .collect::<Result<Vec<_>, _>>()
                .and_then(|b| {
                    if b.is_empty() {
                        Err("an empty body list".into())
                    } else {
                        Ok(b)
                    }
                })?,
        ),
        "hue" => Step::Hue(int(v)?),
        "baseSoundId" => Step::BaseSoundId(int(v)?),
        "speechHue" => Step::SpeechHue(int(v)?),
        "str" => Step::Str(irange(v)?),
        "dex" => Step::Dex(irange(v)?),
        "int" => Step::Int(irange(v)?),
        "hits" => Step::Hits(irange(v)?),
        "stam" => Step::Stam(irange(v)?),
        "mana" => Step::Mana(irange(v)?),
        "damage" => {
            let (a, b) = pair(v, int).ok_or("expected [min, max]")??;
            Step::Damage(a, b)
        }
        "damageTypes" => Step::DamageTypes(map(v, resistance, irange)?),
        "resists" => Step::Resists(map(v, resistance, irange)?),
        "skills" => Step::Skills(map(v, skill, frange)?),
        "skillBase" => Step::SkillBase(map(v, skill, float)?),
        "fame" => Step::Fame(int(v)?),
        "karma" => Step::Karma(int(v)?),
        "virtualArmor" => Step::VirtualArmor(int(v)?),
        "tamable" => Step::Tamable(boolean(v)?),
        "controlSlots" => Step::ControlSlots(int(v)?),
        "minTameSkill" => Step::MinTameSkill(float(v)?),
        "female" => Step::Female(boolean(v)?),
        "canSwim" => Step::CanSwim(boolean(v)?),
        "cantWalk" => Step::CantWalk(boolean(v)?),
        "paragon" => Step::Paragon(boolean(v)?),
        "randomHair" => {
            if !boolean(v)? {
                return Err("only true is meaningful".into());
            }
            Step::RandomHair
        }
        "pack" => Step::Pack(
            v.as_array()
                .ok_or("expected an array of pack entries")?
                .iter()
                .map(pack_entry)
                .collect::<Result<_, _>>()?,
        ),
        _ => match keys.mobiles.and_then(|k| k.0.get_key_value(key)) {
            Some((name, def)) => {
                (def.check)(v)?;
                Step::Ext(name, *def, v.clone())
            }
            None if keys.lenient => Step::Ext("<unregistered>", Keys::skipped(), Value::Null),
            None => return Err("unknown key".into()),
        },
    })
}

fn stat(
    w: &mut World,
    id: MobileId,
    r: IRange,
    fixed: fn(&mut World, MobileId, i32),
    range: fn(&mut World, MobileId, i32, i32),
) {
    match r {
        IRange::Fixed(v) => fixed(w, id, v),
        IRange::Range(a, b) => range(w, id, a, b),
    }
}

fn pack(w: &mut World, id: MobileId, e: &PackEntry) {
    if let Some(c) = e.chance
        && rnd::double(w) >= c
    {
        return;
    }
    match &e.what {
        PackWhat::Item {
            name,
            amount,
            hue,
            item_id,
            equip,
        } => {
            let amount = amount.map(|a| a.roll(w));
            let Some(it) = crate::factory::make_item(w, name) else {
                tracing::warn!("Templates: unknown item class {name}");
                return;
            };
            if let Some(a) = amount {
                muo_server::item::ops::set_amount(w, it, a);
            }
            if let Some(i) = item_id {
                muo_server::item::base::set_item_id(w, it, *i);
            }
            if let Some(h) = hue {
                muo_server::item::base::set_hue(w, it, *h);
            }
            if *equip {
                muo_server::mobile::part2::items::add_item(w, id, it);
            } else {
                setters::pack_item(w, id, Some(it));
            }
        }
        PackWhat::Gold(r) => stat(w, id, *r, setters::pack_gold, setters::pack_gold_range),
        PackWhat::Reg(r) => stat(w, id, *r, packing::pack_reg, packing::pack_reg_range),
        PackWhat::Gem(r) => stat(w, id, *r, packing::pack_gem, packing::pack_gem_range),
        PackWhat::NecroReg(r) => stat(
            w,
            id,
            *r,
            packing::pack_necro_reg_n,
            packing::pack_necro_reg_range,
        ),
        PackWhat::ArcaneScroll(r) => stat(
            w,
            id,
            *r,
            packing::pack_arcane_scroll_n,
            packing::pack_arcane_scroll_range,
        ),
        PackWhat::Scroll(r) => stat(w, id, *r, packing::pack_scroll, packing::pack_scroll_range),
        PackWhat::Statue(r) => stat(
            w,
            id,
            *r,
            packing::pack_statue_n,
            packing::pack_statue_range,
        ),
        PackWhat::Potion => packing::pack_potion(w, id),
        PackWhat::Slayer(c) => {
            packing::pack_slayer(w, id, *c);
        }
    }
}

pub fn apply(w: &mut World, id: MobileId, s: &Step) {
    match s {
        Step::Name(n) => ops::name(w, id, Some(n.clone())),
        Step::NameRandom(k) => ops::name_random(w, id, k),
        Step::Title(t) => ops::title(w, id, t),
        Step::Body(b) => ops::body(w, id, *b),
        Step::Bodies(l) => ops::body_list(w, id, l),
        Step::Hue(h) => ops::hue(w, id, *h),
        Step::BaseSoundId(v) => ops::base_sound_id(w, id, *v),
        Step::SpeechHue(v) => ops::speech_hue(w, id, *v),
        Step::Str(r) => stat(w, id, *r, setters::set_str, setters::set_str_range),
        Step::Dex(r) => stat(w, id, *r, setters::set_dex, setters::set_dex_range),
        Step::Int(r) => stat(w, id, *r, setters::set_int, setters::set_int_range),
        Step::Hits(r) => stat(w, id, *r, setters::set_hits, setters::set_hits_range),
        Step::Stam(r) => stat(w, id, *r, setters::set_stam, setters::set_stam_range),
        Step::Mana(r) => stat(w, id, *r, setters::set_mana, setters::set_mana_range),
        Step::Damage(a, b) => setters::set_damage(w, id, *a, *b),
        Step::DamageTypes(l) => {
            for (t, r) in l {
                match *r {
                    IRange::Fixed(v) => setters::set_damage_type(w, id, *t, v),
                    IRange::Range(a, b) => setters::set_damage_type_range(w, id, *t, a, b),
                }
            }
        }
        Step::Resists(l) => {
            for (t, r) in l {
                match *r {
                    IRange::Fixed(v) => setters::set_resistance(w, id, *t, v),
                    IRange::Range(a, b) => setters::set_resistance_range(w, id, *t, a, b),
                }
            }
        }
        Step::Skills(l) => {
            for (n, r) in l {
                match *r {
                    FRange::Fixed(v) => setters::set_skill(w, id, *n, v),
                    FRange::Range(a, b) => setters::set_skill_range(w, id, *n, a, b),
                }
            }
        }
        Step::SkillBase(l) => {
            for (n, v) in l {
                ops::skill_base(w, id, *n, *v);
            }
        }
        Step::Fame(v) => ops::fame(w, id, *v),
        Step::Karma(v) => ops::karma(w, id, *v),
        Step::VirtualArmor(v) => ops::virtual_armor(w, id, *v),
        Step::Tamable(v) => ops::tamable(w, id, *v),
        Step::ControlSlots(v) => ops::control_slots(w, id, *v),
        Step::MinTameSkill(v) => ops::min_tame_skill(w, id, *v),
        Step::Female(v) => ops::female(w, id, *v),
        Step::CanSwim(v) => ops::can_swim(w, id, *v),
        Step::CantWalk(v) => ops::cant_walk(w, id, *v),
        Step::Paragon(v) => ops::is_paragon(w, id, *v),
        Step::RandomHair => ops::assign_random_hair(w, id),
        Step::Pack(l) => {
            for e in l {
                pack(w, id, e);
            }
        }
        Step::Flow(f) => {
            for s in choose(w, f) {
                apply(w, id, s);
            }
        }
        Step::Ext(_, def, v) => (def.apply)(w, id, v),
    }
}

/// Every built-in key (the step functions scripts get, one per key).
pub const KEYS: &[&str] = &[
    "name",
    "nameRandom",
    "title",
    "body",
    "bodies",
    "hue",
    "baseSoundId",
    "speechHue",
    "str",
    "dex",
    "int",
    "hits",
    "stam",
    "mana",
    "damage",
    "damageTypes",
    "resists",
    "skills",
    "skillBase",
    "fame",
    "karma",
    "virtualArmor",
    "tamable",
    "controlSlots",
    "minTameSkill",
    "female",
    "canSwim",
    "cantWalk",
    "paragon",
    "randomHair",
    "pack",
];
