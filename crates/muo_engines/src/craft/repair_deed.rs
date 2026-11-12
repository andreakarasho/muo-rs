//! `RepairDeed`: the repair service contract a craft shop sells and `Repair.Do`
//! (repair.rs) consumes. Sets the `repair_deed_*` hooks of `muo_content::craft::CraftHooks`.

use muo_content::props::{Ctor, Kind, Param, PropDef, Value};
use muo_content::utilities::types::PropType;
use muo_primitives::TimeSpan;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::layer::Layer;
use muo_server::mobile::AccessLevel;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::ReadEntityExt;
use muo_server::type_registry::{Phase, TypeRegistry};

/// `RepairDeed.RepairSkillType` by index: Smithing, Tailoring, Tinkering, Carpentry, Fletching.
struct SkillInfo {
    system: &'static str,
    nearby: &'static str,
    not_nearby_message: i32,
    name: i32,
}

const TABLE: [SkillInfo; 5] = [
    SkillInfo {
        system: "Blacksmithy",
        nearby: "Server.Mobiles.Blacksmith",
        not_nearby_message: 1047013,
        name: 1023015,
    },
    SkillInfo {
        system: "Tailoring",
        nearby: "Server.Mobiles.Tailor",
        not_nearby_message: 1061132,
        name: 1022981,
    },
    SkillInfo {
        system: "Tinkering",
        nearby: "Server.Mobiles.Tinker",
        not_nearby_message: 1061166,
        name: 1022983,
    },
    SkillInfo {
        system: "Carpentry",
        nearby: "Server.Mobiles.Carpenter",
        not_nearby_message: 1061135,
        name: 1060774,
    },
    SkillInfo {
        system: "BowFletching",
        nearby: "Server.Mobiles.Bowyer",
        not_nearby_message: 1061134,
        name: 1023005,
    },
];

/// `RepairSkillInfo.GetInfo(type)`: an index out of range is Smithing.
fn info(skill: i32) -> &'static SkillInfo {
    &TABLE[usize::try_from(skill)
        .ok()
        .filter(|&v| v < TABLE.len())
        .unwrap_or(0)]
}

/// `RepairDeed`'s fields.
#[derive(Clone, Debug, Default)]
pub struct RepairDeedData {
    pub skill: i32,
    pub skill_level: f64,
    pub crafter: Option<String>,
}

pub fn data(w: &World, id: ItemId) -> Option<&RepairDeedData> {
    w.item(id)?.ext.get::<RepairDeedData>()
}

/// `RepairDeed(skill, level, crafter, normalizeLevel)`.
pub fn new_repair_deed(
    w: &mut World,
    skill: i32,
    level: f64,
    crafter: Option<MobileId>,
    normalize: bool,
) -> ItemId {
    let id = ops::new_item_with_id(w, &REPAIR_DEED, 0x14F0);
    let level = if normalize {
        f64::from((level / 10.0) as i32 * 10)
    } else {
        level
    };
    let crafter = crafter.and_then(|c| muo_server::mobile::part1::props_b::raw_name(w, c));
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(RepairDeedData {
            skill,
            skill_level: level.clamp(0.0, 120.0),
            crafter,
        });
    }
    (REPAIR_DEED.set_hue)(w, id, 0x1BC);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

/// `GetSkillTitle(skillLevel)`: a cliloc number as `#n`, or text.
fn skill_title(level: f64) -> String {
    let skill = (level / 10.0) as i32;
    if skill >= 11 {
        return format!("#{}", 1062008 + skill - 11);
    }
    if skill >= 5 {
        return format!("#{}", 1061123 + skill - 5);
    }
    match skill {
        4 => "a Novice",
        3 => "a Neophyte",
        _ => "a Newbie",
    }
    .to_owned()
}

fn contract_args(w: &World, id: ItemId) -> String {
    let d = data(w, id).cloned().unwrap_or_default();
    format!("{}\t#{}", skill_title(d.skill_level), info(d.skill).name)
}

fn add_name_property(w: &World, id: ItemId, list: &mut PropertyList) {
    // A repair service contract from ~1_SKILL_TITLE~ ~2_SKILL_NAME~.
    list.add_str(1061133, &contract_args(w, id));
}

fn get_properties(w: &World, id: ItemId, list: &mut PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    if let Some(c) = data(w, id).and_then(|d| d.crafter.clone()) {
        list.add_str(1050043, &c); // crafted by ~1_NAME~
    }
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    if ops::is_deleted(w, id) || !deps::can_see(w, from, id) {
        return;
    }
    ops::label_to_number(w, id, from, 1061133, &contract_args(w, id));
    if let Some(c) = data(w, id).and_then(|d| d.crafter.clone()) {
        ops::label_to_number(w, id, from, 1050043, &c);
    }
}

/// `VerifyRegion(m)`: in a town and a shop of the kind within 6.
fn verify_region(w: &mut World, id: ItemId, m: MobileId) -> bool {
    let region = muo_server::mobile::part1::props_b::region(w, m);
    if !muo_server::region::is_part_of_type(w, region, &muo_content::regions::simple::TOWN_REGION) {
        return false;
    }
    let want = info(data(w, id).map_or(0, |d| d.skill)).nearby;
    muo_server::mobile::part2::enumerators::get_mobiles_in_range(w, m, 6)
        .into_iter()
        .any(|o| w.mobile(o).is_some_and(|x| mobile_named(x.ty, want)))
}

fn mobile_named(ty: &'static muo_server::mobile::MobileType, name: &str) -> bool {
    let mut t = Some(ty);
    while let Some(c) = t {
        if c.full_name == name {
            return true;
        }
        t = c.parent;
    }
    false
}

/// `Check(from)`.
pub fn check(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let pack = deps::find_item_on_layer(w, from, Layer::Backpack);
    if !pack.is_some_and(|p| ops::is_child_of(w, id, Some(EntityId::Item(p)))) {
        deps::send_localized_message(w, from, 1047012); // The contract must be in your backpack to use it.
        return false;
    }
    if !verify_region(w, id, from) {
        deps::send_localized_message(
            w,
            from,
            info(data(w, id).map_or(0, |d| d.skill)).not_nearby_message,
        );
        return false;
    }
    true
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if check(w, id, from) {
        let skill = data(w, id).map_or(0, |d| d.skill);
        let hook = muo_content::craft::hooks(w).and_then(|h| h.repair_do_deed);
        if let Some(f) = hook {
            // `Repair.Do(from, RepairSkillInfo.GetInfo(_skill).System, this)`: the hook maps the index to a system.
            f(
                w,
                from,
                id,
                match info(skill).system {
                    "Tailoring" => 1,
                    "Tinkering" => 2,
                    "Carpentry" => 3,
                    "BowFletching" => 4,
                    _ => 0,
                },
            );
        }
    }
}

/// `Server.Items.RepairDeed`, `[SerializationGenerator(1, false)]`.
pub static REPAIR_DEED: ItemType = ItemType {
    full_name: "Server.Items.RepairDeed",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_repair_deed(w, 0, 100.0, None, true)),
    display_loot_type: |_, _| false,
    add_name_property,
    get_properties,
    on_single_click,
    on_double_click,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        let d = data(w, id).cloned().unwrap_or_default();
        out.write_i32(1);
        out.write_enum(4, i64::from(d.skill));
        out.write_f64(d.skill_level);
        out.write_opt_string(d.crafter.as_deref());
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let mut d = RepairDeedData::default();
        if version < 1 {
            // `Timer.DelayCall((item, crafter) => item._crafter = crafter?.RawName, this, reader.ReadEntity<Mobile>())`
            d.skill = r.read_i32()?;
            d.skill_level = r.read_f64()?;
            let crafter = r.read_mobile(w)?;
            muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                let name = crafter.and_then(|c| muo_server::mobile::part1::props_b::raw_name(w, c));
                if let Some(d) = w
                    .item_mut(id)
                    .and_then(|i| i.ext.get_mut::<RepairDeedData>())
                {
                    d.crafter = name;
                }
            });
        } else {
            d.skill = r.read_enum(4)? as i32;
            d.skill_level = r.read_f64()?;
            d.crafter = r.read_string()?;
        }
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        Ok(())
    },
    ..ITEM_BASE
};

fn hook_check(w: &mut World, id: ItemId, from: MobileId) -> bool {
    check(w, id, from)
}

fn hook_level(w: &World, id: ItemId) -> f64 {
    data(w, id).map_or(0.0, |d| d.skill_level)
}

fn hook_create(w: &mut World, skill: i32, level: f64, crafter: MobileId) -> Option<ItemId> {
    Some(new_repair_deed(w, skill, level, Some(crafter), true))
}

/// Sets the `repair_deed_*` hooks (called from the craft engine's Configure).
pub fn configure(w: &mut World) {
    let h = muo_content::craft::hooks_mut(w);
    h.repair_deed_check = Some(hook_check);
    h.repair_deed_level = Some(hook_level);
    h.repair_deed_create = Some(hook_create);
}

// ---- `[add RepairDeed ..` and the `[CommandProperty]` members ----

const SKILL_TYPES: &[(&str, i64)] = &[
    ("Smithing", 0),
    ("Tailoring", 1),
    ("Tinkering", 2),
    ("Carpentry", 3),
    ("Fletching", 4),
];
const SKILL_KIND: Kind = Kind::Plain(PropType::Enum(SKILL_TYPES));
const DOUBLE: Kind = Kind::Plain(PropType::Double);
const BOOL: Kind = Kind::Plain(PropType::Bool);
const ENTITY: Kind = Kind::Plain(PropType::Entity);

/// `RepairDeed(skill, level, normalizeLevel)` then `RepairDeed(skill = Smithing, level = 100.0, crafter = null,
/// normalizeLevel = true)`, in declaration order.
static CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[
            Param {
                name: "skill",
                kind: SKILL_KIND,
                default: None,
                ty: "RepairSkillType",
            },
            Param {
                name: "level",
                kind: DOUBLE,
                default: None,
                ty: "",
            },
            Param {
                name: "normalizeLevel",
                kind: BOOL,
                default: None,
                ty: "",
            },
        ],
        make: |w, v| {
            EntityId::Item(new_repair_deed(
                w,
                v[0].i32(),
                v[1].f64(),
                None,
                v[2].bool(),
            ))
        },
    },
    Ctor {
        access: None,
        params: &[
            Param {
                name: "skill",
                kind: SKILL_KIND,
                default: Some(Value::Enum(0)),
                ty: "RepairSkillType",
            },
            Param {
                name: "level",
                kind: DOUBLE,
                default: Some(Value::Float(100.0)),
                ty: "",
            },
            Param {
                name: "crafter",
                kind: ENTITY,
                default: Some(Value::Null),
                ty: "Mobile",
            },
            Param {
                name: "normalizeLevel",
                kind: BOOL,
                default: Some(Value::Bool(true)),
                ty: "",
            },
        ],
        make: |w, v| {
            EntityId::Item(new_repair_deed(
                w,
                v[0].i32(),
                v[1].f64(),
                v[2].mobile(),
                v[3].bool(),
            ))
        },
    },
];

fn update(w: &mut World, e: EntityId, f: impl FnOnce(&mut RepairDeedData)) -> Result<(), String> {
    let EntityId::Item(id) = e else {
        return Err("not an item".to_string());
    };
    let d = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<RepairDeedData>())
        .ok_or("not a repair deed")?;
    f(d);
    ops::invalidate_properties(w, id); // [InvalidateProperties]
    Ok(())
}

static PROPS: &[PropDef] = &[
    PropDef {
        name: "Skill",
        kind: SKILL_KIND,
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Enum(i64::from(data(w, i).map_or(0, |d| d.skill))),
            _ => Value::Null,
        },
        set: Some(|w, e, v| update(w, e, |d| d.skill = v.i32())),
    },
    PropDef {
        name: "Crafter",
        kind: Kind::Plain(PropType::Str),
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => data(w, i)
                .and_then(|d| d.crafter.clone())
                .map_or(Value::Null, Value::Str),
            _ => Value::Null,
        },
        set: Some(|w, e, v| update(w, e, |d| d.crafter = v.string())),
    },
    PropDef {
        name: "SkillLevel",
        kind: DOUBLE,
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Float(data(w, i).map_or(0.0, |d| d.skill_level)),
            _ => Value::Null,
        },
        // `AllowSkillLevelChange`: `Math.Clamp(value, 0, 120.0)`
        set: Some(|w, e, v| update(w, e, |d| d.skill_level = v.f64().clamp(0.0, 120.0))),
    },
];

fn configure_props(w: &mut World) {
    muo_content::props::add(w, "Server.Items.RepairDeed", PROPS);
    muo_content::props::add_ctors(w, "Server.Items.RepairDeed", &CTORS);
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&REPAIR_DEED, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.RepairDeed.Configure",
        configure_props,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctor_tables_and_properties() {
        let mut w = World::new();
        configure_props(&mut w);
        // the two overloads, in declaration order; the second one's defaults
        assert_eq!(CTORS[0].params.len(), 3);
        assert_eq!(
            CTORS[1]
                .params
                .iter()
                .filter(|p| p.default.is_some())
                .count(),
            4
        );
        let EntityId::Item(id) = (CTORS[0].make)(
            &mut w,
            &[Value::Enum(2), Value::Float(105.0), Value::Bool(true)],
        ) else {
            panic!("an item")
        };
        let d = data(&w, id).unwrap();
        assert_eq!((d.skill, d.skill_level), (2, 100.0));
        let p = |n: &str| PROPS.iter().find(|p| p.name == n).unwrap();
        (p("SkillLevel").set.unwrap())(&mut w, EntityId::Item(id), Value::Float(500.0)).unwrap();
        assert_eq!(data(&w, id).unwrap().skill_level, 120.0);
        (p("Crafter").set.unwrap())(&mut w, EntityId::Item(id), Value::Str("Bob".into())).unwrap();
        assert!(
            matches!((p("Crafter").get)(&mut w, EntityId::Item(id)), Value::Str(s) if s == "Bob")
        );
    }
}
