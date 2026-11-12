//! Command tests: how a `where` clause compiles and compares, and how text becomes a
//! property value.
//!
//! A class here is an item type with a property table, so each `Subject` below is a fixture class with the
//! property shapes under test (a
//! `TextDefinition` with `.Number`/`.String`, `Nullable<T>`, a `Rectangle2D`).

use muo_content::props::{self, Kind, PropDef, Value};
use muo_content::utilities::types::{Numeric, ParseContext, Parsed, PropType, try_parse};
use muo_io::TextDefinition;
use muo_primitives::{Point3D, Serial};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::target::Targeted;

use crate::generic::conditional::ObjectConditional;
use crate::properties;

const GM: AccessLevel = AccessLevel::GameMaster;

fn args(s: &[&str]) -> Vec<String> {
    s.iter().map(|s| s.to_string()).collect()
}

fn base_world() -> (World, MobileId) {
    let mut w = muo_server::map::test_support::world();
    props::register_core(&mut w);
    let from = w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE);
    w.mobile_mut(from).unwrap().access_level = AccessLevel::Developer;
    (w, from)
}

// ---- emitter robustness ----

static EMITTER_PROPS: [PropDef; 6] = [
    // `byte Tiny = 5`, `sbyte Signed = -5`, `short Small = -300`, `ushort Key = 40000`, `uint Big = 3_000_000_000`,
    // `ulong Huge = 18_000_000_000_000_000_000`
    PropDef {
        name: "Tiny",
        kind: Kind::Plain(PropType::Numeric(Numeric::Byte)),
        read: GM,
        write: GM,
        get: |_, _| Value::Int(5),
        set: None,
    },
    PropDef {
        name: "Signed",
        kind: Kind::Plain(PropType::Numeric(Numeric::SByte)),
        read: GM,
        write: GM,
        get: |_, _| Value::Int(-5),
        set: None,
    },
    PropDef {
        name: "Small",
        kind: Kind::Plain(PropType::Numeric(Numeric::Short)),
        read: GM,
        write: GM,
        get: |_, _| Value::Int(-300),
        set: None,
    },
    PropDef {
        name: "Key",
        kind: Kind::Plain(PropType::Numeric(Numeric::UShort)),
        read: GM,
        write: GM,
        get: |_, _| Value::Int(40000),
        set: None,
    },
    PropDef {
        name: "Big",
        kind: Kind::Plain(PropType::Numeric(Numeric::UInt)),
        read: GM,
        write: GM,
        get: |_, _| Value::UInt(3_000_000_000),
        set: None,
    },
    PropDef {
        name: "Huge",
        kind: Kind::Plain(PropType::Numeric(Numeric::ULong)),
        read: GM,
        write: GM,
        get: |_, _| Value::UInt(18_000_000_000_000_000_000),
        set: None,
    },
];
static EMITTER_SUBJECT: ItemType = ItemType {
    full_name: "Server.Tests.EmitterSubject",
    parent: Some(&ITEM_BASE),
    ..ITEM_BASE
};

fn emitter_check(binding: &str, op: &str, value: &str) -> bool {
    let (mut w, from) = base_world();
    w.types.register_item(&EMITTER_SUBJECT, &[]);
    props::add(&mut w, "Server.Tests.EmitterSubject", &EMITTER_PROPS);
    let level = w.mobile(from).unwrap().access_level;
    let id = w.new_item(&EMITTER_SUBJECT);
    ObjectConditional::parse_direct(&w, level, &args(&["EmitterSubject", binding, op, value]))
        .expect("the condition compiles")
        .check(&mut w, Targeted::Entity(EntityId::Item(id)))
}

#[test]
fn narrow_and_unsigned_integers_compare_by_equality() {
    for (binding, value) in [
        ("Tiny", "5"),
        ("Signed", "-5"),
        ("Small", "-300"),
        ("Key", "40000"),
        ("Big", "3000000000"),
        ("Huge", "18000000000000000000"),
    ] {
        assert!(emitter_check(binding, "=", value), "{binding} = {value}");
        assert!(!emitter_check(binding, "!=", value), "{binding} != {value}");
    }
}

#[test]
fn narrow_and_unsigned_integers_compare_relationally() {
    for (binding, value) in [
        ("Tiny", "4"),
        ("Signed", "-6"),
        ("Small", "-301"),
        ("Key", "39999"),
        ("Big", "2999999999"),
        ("Huge", "17999999999999999999"),
    ] {
        assert!(emitter_check(binding, ">", value), "{binding} > {value}");
        assert!(!emitter_check(binding, "<", value), "{binding} < {value}");
    }
}

// Unsigned values above the signed range must not wrap into a negative comparison.
#[test]
fn unsigned_comparisons_do_not_wrap_through_signed_math() {
    assert!(emitter_check("Big", ">", "2147483647"));
    assert!(emitter_check("Huge", ">", "9223372036854775807"));
}

// ---- where constant parsing ----

#[derive(Clone, Copy, Default)]
struct WhereData {
    owner: Option<EntityId>,
    thing: Option<EntityId>,
    label: bool,
}

fn wd(w: &World, e: EntityId) -> WhereData {
    match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|i| i.ext.get::<WhereData>())
            .copied()
            .unwrap_or_default(),
        EntityId::Mobile(_) => WhereData::default(),
    }
}

const SKILLS: &[(&str, i64)] = &[("Alchemy", 0), ("Anatomy", 1), ("Magery", 25)];

static WHERE_PROPS: [PropDef; 8] = [
    muo_content::text_def_prop!(
        "Message",
        GM,
        GM,
        |w: &World, e: EntityId| msg_of(w, e),
        None
    ),
    // `Type Kind = typeof(Static)`: the stand-in class is the fixture itself
    PropDef {
        name: "Kind",
        kind: Kind::Plain(PropType::Type),
        read: GM,
        write: GM,
        get: |_, _| Value::Str("Server.Tests.WhereSubject".into()),
        set: None,
    },
    PropDef {
        name: "Owner",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| Value::Entity(wd(w, e).owner),
        set: None,
    },
    PropDef {
        name: "Thing",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| Value::Entity(wd(w, e).thing),
        set: None,
    },
    PropDef {
        name: "Label",
        kind: Kind::Plain(PropType::Str),
        read: GM,
        write: GM,
        get: |w, e| {
            if wd(w, e).label {
                Value::Str("set".into())
            } else {
                Value::Null
            }
        },
        set: None,
    },
    PropDef {
        name: "Facet",
        kind: Kind::Map,
        read: GM,
        write: GM,
        get: |_, _| Value::Map(Some(MapId::FELUCCA)),
        set: None,
    },
    PropDef {
        name: "Count",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |_, _| Value::Int(0x1F13),
        set: None,
    },
    PropDef {
        name: "Craft",
        kind: Kind::Plain(PropType::Enum(SKILLS)),
        read: GM,
        write: GM,
        get: |_, _| Value::Enum(25),
        set: None,
    },
];
static WHERE_SUBJECT: ItemType = ItemType {
    full_name: "Server.Tests.WhereSubject",
    parent: Some(&ITEM_BASE),
    ..ITEM_BASE
};

fn where_world() -> (World, MobileId, ItemId) {
    let (mut w, from) = base_world();
    w.types.register_item(&WHERE_SUBJECT, &[]);
    w.types.register_item(&EMITTER_SUBJECT, &[]);
    props::add(&mut w, "Server.Tests.WhereSubject", &WHERE_PROPS);
    let s = w.new_item(&WHERE_SUBJECT);
    (w, from, s)
}

fn where_check(
    w: &mut World,
    from: MobileId,
    s: ItemId,
    binding: &str,
    value: &str,
) -> Result<bool, String> {
    let level = w.mobile(from).unwrap().access_level;
    let c =
        ObjectConditional::parse_direct(w, level, &args(&["WhereSubject", binding, "=", value]))?;
    Ok(c.check(w, Targeted::Entity(EntityId::Item(s))))
}

#[test]
fn type_valued_properties_resolve_by_name() {
    let (mut w, from, s) = where_world();
    // a type value compares with the class name ("Static", "Container"); the registered stand-ins here
    assert_eq!(
        where_check(&mut w, from, s, "Kind", "WhereSubject"),
        Ok(true)
    );
    assert_eq!(
        where_check(&mut w, from, s, "Kind", "EmitterSubject"),
        Ok(false)
    );
}

#[test]
fn entity_valued_properties_resolve_by_serial() {
    let (mut w, from, s) = where_world();
    let mob = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let item = w.new_item(&ITEM_BASE);
    w.item_mut(s).unwrap().ext = Ext::new(WhereData {
        owner: Some(EntityId::Mobile(mob)),
        thing: Some(EntityId::Item(item)),
        label: false,
    });
    let serial = |e: Serial| e.value().to_string();
    assert_eq!(
        where_check(&mut w, from, s, "Owner", &serial(mob.0)),
        Ok(true)
    );
    assert_eq!(
        where_check(&mut w, from, s, "Thing", &serial(item.0)),
        Ok(true)
    );
    assert_eq!(
        where_check(&mut w, from, s, "Thing", &(item.0.value() + 1).to_string()),
        Ok(false)
    );
}

// `where` spells null as a bare `null`, not [set's (-null-); every existing clause relies on it.
#[test]
fn bare_null_still_means_null() {
    let (mut w, from, s) = where_world();
    assert_eq!(where_check(&mut w, from, s, "Label", "null"), Ok(true));
    assert_eq!(where_check(&mut w, from, s, "Owner", "null"), Ok(true));

    let mob = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    w.item_mut(s).unwrap().ext = Ext::new(WhereData {
        owner: Some(EntityId::Mobile(mob)),
        thing: None,
        label: true,
    });
    assert_eq!(where_check(&mut w, from, s, "Label", "null"), Ok(false));
    assert_eq!(where_check(&mut w, from, s, "Owner", "null"), Ok(false));
}

#[test]
fn quoted_null_still_means_the_literal_string() {
    let (mut w, from, s) = where_world();
    // `Label = "null"`: the literal four-letter string; here Label holds "set", so set the stand-in text
    static LITERAL: PropDef = PropDef {
        name: "Literal",
        kind: Kind::Plain(PropType::Str),
        read: GM,
        write: GM,
        get: |_, _| Value::Str("null".into()),
        set: None,
    };
    props::add(
        &mut w,
        "Server.Tests.WhereSubject",
        std::slice::from_ref(&LITERAL),
    );
    assert_eq!(
        where_check(&mut w, from, s, "Literal", "@\"null\""),
        Ok(true)
    );
    assert_eq!(where_check(&mut w, from, s, "Literal", "null"), Ok(false));
}

#[test]
fn existing_constant_forms_keep_working() {
    let (mut w, from, s) = where_world();
    // [InlineData(binding, value, expected)]
    for (binding, value, expected) in [
        ("Facet", "Felucca", true),
        ("Facet", "Trammel", false),
        ("Count", "0x1F13", true),
        ("Count", "7955", true),
        ("Count", "0x1F14", false),
        ("Craft", "Magery", true),
        ("Craft", "Anatomy", false),
    ] {
        assert_eq!(
            where_check(&mut w, from, s, binding, value),
            Ok(expected),
            "{binding} = {value}"
        );
    }
}

#[test]
fn unresolvable_constants_still_report_an_error() {
    let (mut w, from, s) = where_world();
    assert!(where_check(&mut w, from, s, "Kind", "NoSuchTypeAnywhere").is_err());
}

#[test]
fn text_definition_markers_reach_the_where_clause() {
    let (mut w, from, s) = where_world();
    w.item_mut(s)
        .unwrap()
        .ext
        .insert(Msg(Some(TextDefinition::of_number(1060847))));
    for (value, expected) in [("#1060847", true), ("#1060848", false)] {
        assert_eq!(
            where_check(&mut w, from, s, "Message", value),
            Ok(expected),
            "{value}"
        );
    }
}

#[test]
fn quoted_literals_reach_text_definitions_in_where_too() {
    let (mut w, from, s) = where_world();
    w.item_mut(s)
        .unwrap()
        .ext
        .insert(Msg(Some(TextDefinition::of_text("1060847"))));
    assert_eq!(
        where_check(&mut w, from, s, "Message", "@\"1060847\""),
        Ok(true)
    );
    assert_eq!(
        where_check(&mut w, from, s, "Message", "1060847"),
        Ok(false)
    );
}

// ---- TextDefinition properties ----

fn parse_td(s: &str) -> TextDefinition {
    TextDefinition::parse(s)
}

#[test]
fn clilocs_parse_to_number() {
    for (entered, expected) in [("1060847", 1060847), ("#1060847", 1060847)] {
        let td = parse_td(entered);
        assert_eq!(td.number, expected);
        assert_eq!(td.string, None);
    }
}

// ToString() already emits '#', so this closes the round trip.
#[test]
fn cliloc_marker_round_trips_through_to_string() {
    let td = TextDefinition::of_number(1060847);
    assert_eq!(td.to_string(), "#1060847");
    assert_eq!(td, parse_td(&td.to_string()));
}

#[test]
fn quoted_literals_force_a_string() {
    for (entered, expected) in [
        ("hello", "hello"),
        ("@\"0\"", "0"),
        ("@\"1060847\"", "1060847"),
        ("@\"null\"", "null"),
        ("@\"#5\"", "#5"),
    ] {
        let td = parse_td(entered);
        assert_eq!(td.number, 0, "{entered}");
        assert_eq!(td.string.as_deref(), Some(expected), "{entered}");
    }
}

// Bare integers stay clilocs, so existing content is unaffected.
#[test]
fn bare_zero_stays_an_empty_cliloc() {
    let td = parse_td("0");
    assert_eq!(td.number, 0);
    assert_eq!(td.string, None);
}

// GetValue() fills the props-gump edit box; an ambiguous string must come out quoted.
#[test]
fn ambiguous_strings_are_emitted_quoted_so_they_read_back() {
    for text in ["1060847", "0", "#5"] {
        let round = parse_td(&TextDefinition::of_text(text).get_value());
        assert_eq!(round.number, 0, "{text}");
        assert_eq!(round.string.as_deref(), Some(text), "{text}");
    }
}

#[test]
fn unambiguous_values_are_emitted_plain() {
    assert_eq!(
        TextDefinition::of_text("Hail, traveller.").get_value(),
        "Hail, traveller."
    );
    assert_eq!(TextDefinition::of_number(1060847).get_value(), "1060847");
}

#[test]
fn null_sentinel_clears_the_value() {
    let (w, _) = base_world();
    assert_eq!(
        props::parse_kind(&w, &Kind::TextDef(&[]), "(-null-)"),
        Ok(Value::Null)
    );
}

fn parse_str(value: &str) -> Parsed {
    let find_entity = |_: Serial| false;
    let find_type = |_: &str| None;
    let ctx = ParseContext {
        find_entity: &find_entity,
        find_type: &find_type,
    };
    try_parse(&PropType::Str, Some(value), &ctx).expect("a string parses")
}

// [get emits @"null" for the literal string "null"; [set has to decode it to round trip.
#[test]
fn quoted_null_sets_the_literal_string_null() {
    assert_eq!(parse_str("@\"null\""), Parsed::Str("null".into()));
}

#[test]
fn bare_null_sentinel_still_clears_the_value() {
    assert_eq!(parse_str("(-null-)"), Parsed::Null);
}

#[test]
fn get_output_pastes_back_into_set() {
    let (mut w, from) = base_world();
    let item = w.new_item(&ITEM_BASE);
    w.item_mut(item).unwrap().item_id = 0x1F13;
    w.item_mut(item).unwrap().name = Some("null".into());
    let e = EntityId::Item(item);

    let shown = properties::get_value(&mut w, from, e, "Name");
    // "Name = @"null"" -> the value half is what a GM copies.
    let value = &shown[shown.find('=').unwrap() + 2..];
    assert_eq!(value, "@\"null\"");

    w.item_mut(item).unwrap().name = Some("changed".into());
    properties::set_value(&mut w, from, e, "Name", value);

    assert_eq!(w.item(item).unwrap().name.as_deref(), Some("null"));
}

// ---- object conditionals, chained binding sort, distinct comparer, conditional compiler edges ----

/// `SkillTeleporter.Message`: a `TextDefinition` that may be null, kept in the item's `ext`.
#[derive(Clone, Default)]
struct Msg(Option<TextDefinition>);

fn msg_of(w: &World, e: EntityId) -> Option<TextDefinition> {
    match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|i| i.ext.get::<Msg>())
            .and_then(|m| m.0.clone()),
        EntityId::Mobile(_) => None,
    }
}

fn set_msg(w: &mut World, i: ItemId, m: Option<TextDefinition>) {
    w.item_mut(i).unwrap().ext.insert(Msg(m));
}

static TELEPORTER_PROPS: [PropDef; 2] = [
    PropDef {
        name: "Skill",
        kind: Kind::Plain(PropType::Enum(SKILLS)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Enum(w.item(i).map_or(0, |i| i.amount.into())),
            EntityId::Mobile(_) => Value::Null,
        },
        set: None,
    },
    muo_content::text_def_prop!(
        "Message",
        GM,
        GM,
        |w: &World, e: EntityId| msg_of(w, e),
        None
    ),
];
static SKILL_TELEPORTER: ItemType = ItemType {
    full_name: "Server.Items.SkillTeleporter",
    parent: Some(&ITEM_BASE),
    ..ITEM_BASE
};

/// A `SkillTeleporter` fixture: Hue and Name from the item, Skill (the enum) from its amount field, Message.
fn teleporter(w: &mut World, message: Option<TextDefinition>) -> ItemId {
    w.types.register_item(&SKILL_TELEPORTER, &[]);
    props::add(w, "Server.Items.SkillTeleporter", &TELEPORTER_PROPS);
    let id = w.new_item(&SKILL_TELEPORTER);
    w.item_mut(id).unwrap().amount = 25; // Magery
    set_msg(w, id, message);
    id
}

fn tp_check(w: &mut World, from: MobileId, tp: ItemId, condition: &[&str]) -> bool {
    let mut a = vec!["SkillTeleporter"];
    a.extend_from_slice(condition);
    let level = w.mobile(from).unwrap().access_level;
    ObjectConditional::parse_direct(w, level, &args(&a))
        .expect("compiles")
        .check(w, Targeted::Entity(EntityId::Item(tp)))
}

fn number(n: i32) -> Option<TextDefinition> {
    Some(TextDefinition::of_number(n))
}

fn text(s: &str) -> Option<TextDefinition> {
    Some(TextDefinition::of_text(s))
}

#[test]
fn equality_uses_value_semantics_not_reference_identity() {
    let (mut w, from) = base_world();
    let tp = teleporter(&mut w, number(1060847));
    assert!(tp_check(&mut w, from, tp, &["Message", "=", "1060847"]));
}

#[test]
fn inequality_uses_value_semantics_not_reference_identity() {
    let (mut w, from) = base_world();
    let tp = teleporter(&mut w, number(1060847));
    assert!(!tp_check(&mut w, from, tp, &["Message", "!=", "1060847"]));
    assert!(tp_check(&mut w, from, tp, &["Message", "!=", "1234567"]));
}

#[test]
fn string_valued_definitions_compare_by_value() {
    let (mut w, from) = base_world();
    let tp = teleporter(&mut w, text("Hail, traveller."));
    assert!(tp_check(
        &mut w,
        from,
        tp,
        &["Message", "=", "Hail, traveller."]
    ));
    assert!(!tp_check(&mut w, from, tp, &["Message", "=", "Farewell."]));
}

#[test]
fn null_comparison_still_works() {
    let (mut w, from) = base_world();
    let blank = teleporter(&mut w, None);
    let set = teleporter(&mut w, number(1060847));
    assert!(tp_check(&mut w, from, blank, &["Message", "=", "null"]));
    assert!(!tp_check(&mut w, from, set, &["Message", "=", "null"]));
}

// The sweep case: [global where SkillTeleporter Message.Number = X hits teleporters whose Message was never set long
// before it hits one that matches.
#[test]
fn chained_binding_on_a_null_intermediate_is_false_not_an_exception() {
    let (mut w, from) = base_world();
    let blank = teleporter(&mut w, None);
    assert!(!tp_check(
        &mut w,
        from,
        blank,
        &["Message.Number", "=", "1060847"]
    ));
}

#[test]
fn chained_binding_still_matches_when_intermediate_is_present() {
    let (mut w, from) = base_world();
    let tp = teleporter(&mut w, number(1060847));
    assert!(tp_check(
        &mut w,
        from,
        tp,
        &["Message.Number", "=", "1060847"]
    ));
    assert!(!tp_check(
        &mut w,
        from,
        tp,
        &["Message.Number", "=", "1234567"]
    ));
}

#[test]
fn chained_binding_on_a_null_intermediate_is_false_under_negation_too() {
    let (mut w, from) = base_world();
    let blank = teleporter(&mut w, None);
    assert!(!tp_check(
        &mut w,
        from,
        blank,
        &["not", "Message.Number", "=", "1060847"]
    ));
}

#[test]
fn string_operators_on_a_null_intermediate_are_false_not_an_exception() {
    let (mut w, from) = base_world();
    let blank = teleporter(&mut w, None);
    for oper in ["contains", "contains~", "starts", "ends", "=~", "!=~"] {
        assert!(
            !tp_check(&mut w, from, blank, &["Message.String", oper, "gate"]),
            "{oper}"
        );
    }
}

#[test]
fn string_operators_on_a_null_intermediate_are_false_under_negation_too() {
    let (mut w, from) = base_world();
    let blank = teleporter(&mut w, None);
    assert!(!tp_check(
        &mut w,
        from,
        blank,
        &["not", "Message.String", "contains", "gate"]
    ));
}

#[test]
fn string_operators_still_match_through_a_chain() {
    let (mut w, from) = base_world();
    let tp = teleporter(&mut w, text("Moongate"));
    assert!(tp_check(
        &mut w,
        from,
        tp,
        &["Message.String", "contains", "gate"]
    ));
    assert!(tp_check(
        &mut w,
        from,
        tp,
        &["Message.String", "starts", "Moon"]
    ));
    assert!(tp_check(
        &mut w,
        from,
        tp,
        &["Message.String", "ends", "gate"]
    ));
    assert!(tp_check(
        &mut w,
        from,
        tp,
        &["Message.String", "=~", "moongate"]
    ));
    assert!(!tp_check(
        &mut w,
        from,
        tp,
        &["Message.String", "contains", "portal"]
    ));
}

// The chain resolves here -- Message is set -- but its String is null because the definition holds a cliloc.
#[test]
fn string_operators_handle_a_null_final_value() {
    let (mut w, from) = base_world();
    let tp = teleporter(&mut w, number(1060847));
    assert!(!tp_check(
        &mut w,
        from,
        tp,
        &["Message.String", "contains", "gate"]
    ));
    assert!(!tp_check(
        &mut w,
        from,
        tp,
        &["Message.String", "=~", "gate"]
    ));
}

#[test]
fn unchained_string_operators_still_work() {
    let (mut w, from) = base_world();
    let tp = teleporter(&mut w, None);
    w.item_mut(tp).unwrap().name = Some("Moongate".into());
    assert!(tp_check(&mut w, from, tp, &["Name", "contains", "gate"]));
    assert!(tp_check(&mut w, from, tp, &["Name", "=~", "moongate"]));
    assert!(!tp_check(&mut w, from, tp, &["Name", "contains", "portal"]));
}

#[test]
fn numeric_comparisons_still_work() {
    let (mut w, from) = base_world();
    let tp = teleporter(&mut w, None);
    w.item_mut(tp).unwrap().hue = 42;
    assert!(tp_check(&mut w, from, tp, &["Hue", "=", "42"]));
    assert!(!tp_check(&mut w, from, tp, &["Hue", "=", "43"]));
    assert!(tp_check(&mut w, from, tp, &["Hue", ">", "41"]));
    assert!(tp_check(&mut w, from, tp, &["Hue", "<", "43"]));
    assert!(tp_check(&mut w, from, tp, &["Hue", "!=", "43"]));
}

#[test]
fn string_comparisons_still_work() {
    let (mut w, from) = base_world();
    let tp = teleporter(&mut w, None);
    w.item_mut(tp).unwrap().name = Some("gate".into());
    assert!(tp_check(&mut w, from, tp, &["Name", "=", "gate"]));
    assert!(!tp_check(&mut w, from, tp, &["Name", "=", "portal"]));
}

#[test]
fn enum_comparisons_still_work() {
    let (mut w, from) = base_world();
    let tp = teleporter(&mut w, None);
    assert!(tp_check(&mut w, from, tp, &["Skill", "=", "Magery"]));
    assert!(!tp_check(&mut w, from, tp, &["Skill", "=", "Anatomy"]));
}

// ---- chained binding sort / distinct comparer: `order by` and `distinct` compile the same chains ----

use crate::generic::conditional::{Binding, compare_by};
use muo_content::props::Access;

fn sorter(w: &World, binding: &str) -> Vec<(Binding, bool)> {
    let class = props::find_class(w, "SkillTeleporter").unwrap();
    let b = Binding::bind(w, &class, AccessLevel::Developer, binding, Access::READ).unwrap();
    b.check_comparable().unwrap();
    vec![(b, true)]
}

fn cmp3(w: &mut World, o: &[(Binding, bool)], a: ItemId, b: ItemId) -> i32 {
    compare_by(
        w,
        o,
        Targeted::Entity(EntityId::Item(a)),
        Targeted::Entity(EntityId::Item(b)),
    ) as i32
}

#[test]
fn sorting_on_a_chain_survives_a_null_intermediate() {
    let (mut w, _) = base_world();
    let named = teleporter(&mut w, text("Alpha"));
    let blank = teleporter(&mut w, None);
    let c = sorter(&w, "Message.String");
    // A consistent total order is the contract; which end the blanks land on is not asserted.
    let forward = cmp3(&mut w, &c, named, blank);
    assert_ne!(0, forward);
    assert_eq!(-forward.signum(), cmp3(&mut w, &c, blank, named).signum());
    assert_eq!(0, cmp3(&mut w, &c, blank, blank));
    assert_eq!(0, cmp3(&mut w, &c, named, named));
}

#[test]
fn sorting_on_a_chain_still_orders_readable_values() {
    let (mut w, _) = base_world();
    let alpha = teleporter(&mut w, text("Alpha"));
    let beta = teleporter(&mut w, text("Beta"));
    let c = sorter(&w, "Message.String");
    assert!(cmp3(&mut w, &c, alpha, beta) < 0);
    assert!(cmp3(&mut w, &c, beta, alpha) > 0);
}

#[test]
fn distinct_on_a_chain_survives_a_null_intermediate() {
    let (mut w, _) = base_world();
    let named = teleporter(&mut w, text("Alpha"));
    let blank = teleporter(&mut w, None);
    let also_blank = teleporter(&mut w, None);
    let c = sorter(&w, "Message.String");
    assert_ne!(0, cmp3(&mut w, &c, named, blank));
    assert_eq!(0, cmp3(&mut w, &c, blank, also_blank));
}

// A null intermediate on a value-typed chain reads as default(int) rather than throwing.
#[test]
fn sorting_on_a_value_type_chain_still_survives_a_null_intermediate() {
    let (mut w, _) = base_world();
    let valued = teleporter(&mut w, number(1000));
    let blank = teleporter(&mut w, None);
    let c = sorter(&w, "Message.Number");
    // A null Message reads as default(int) == 0, so it sorts below cliloc 1000.
    assert!(cmp3(&mut w, &c, blank, valued) < 0);
    assert!(cmp3(&mut w, &c, valued, blank) > 0);
}

#[test]
fn sorting_on_a_chained_value_type_orders_values() {
    let (mut w, _) = base_world();
    let low = teleporter(&mut w, number(1060847));
    let high = teleporter(&mut w, number(1060848));
    let c = sorter(&w, "Message.Number");
    assert!(cmp3(&mut w, &c, low, high) < 0);
    assert!(cmp3(&mut w, &c, high, low) > 0);
    assert_eq!(0, cmp3(&mut w, &c, low, low));
}

#[test]
fn distinct_on_a_chained_value_type_separates_values() {
    let (mut w, _) = base_world();
    let low = teleporter(&mut w, number(1060847));
    let high = teleporter(&mut w, number(1060848));
    let also_low = teleporter(&mut w, number(1060847));
    let c = sorter(&w, "Message.Number");
    assert_ne!(0, cmp3(&mut w, &c, low, high));
    assert_eq!(0, cmp3(&mut w, &c, low, also_low));
}

fn tp_with(
    w: &mut World,
    hue: i32,
    name: &str,
    loc: Point3D,
    message: Option<TextDefinition>,
) -> ItemId {
    let tp = teleporter(w, message);
    let i = w.item_mut(tp).unwrap();
    i.hue = hue;
    i.name = Some(name.into());
    i.location = loc;
    tp
}

// There is no hash here (distinct is a sort over the bindings): equal on every listed property is `Equal`.
#[test]
fn equal_objects_hash_the_same_across_property_shapes() {
    let (mut w, _) = base_world();
    teleporter(&mut w, None);
    let class = props::find_class(&w, "SkillTeleporter").unwrap();
    let c: Vec<(Binding, bool)> = ["Hue", "Name", "Location", "Message.String"]
        .iter()
        .map(|b| {
            (
                Binding::bind(&w, &class, AccessLevel::Developer, b, Access::READ).unwrap(),
                true,
            )
        })
        .collect();
    let p = Point3D::new(1, 2, 3);
    let one = tp_with(&mut w, 7, "Gate", p, text("Alpha"));
    let two = tp_with(&mut w, 7, "Gate", p, text("Alpha"));
    assert_eq!(0, cmp3(&mut w, &c, one, two));
}

#[test]
fn null_references_and_null_chain_links_hash_without_throwing() {
    let (mut w, _) = base_world();
    teleporter(&mut w, None);
    let class = props::find_class(&w, "SkillTeleporter").unwrap();
    let c: Vec<(Binding, bool)> = ["Name", "Message.String"]
        .iter()
        .map(|b| {
            (
                Binding::bind(&w, &class, AccessLevel::Developer, b, Access::READ).unwrap(),
                true,
            )
        })
        .collect();
    let blank = teleporter(&mut w, None);
    let also_blank = teleporter(&mut w, None);
    assert_eq!(0, cmp3(&mut w, &c, blank, also_blank));
}

#[test]
fn differing_objects_are_not_equal() {
    let (mut w, _) = base_world();
    teleporter(&mut w, None);
    let class = props::find_class(&w, "SkillTeleporter").unwrap();
    let c: Vec<(Binding, bool)> = ["Hue", "Location"]
        .iter()
        .map(|b| {
            (
                Binding::bind(&w, &class, AccessLevel::Developer, b, Access::READ).unwrap(),
                true,
            )
        })
        .collect();
    let one = tp_with(&mut w, 1, "", Point3D::ZERO, None);
    let two = tp_with(&mut w, 2, "", Point3D::ZERO, None);
    assert_ne!(0, cmp3(&mut w, &c, one, two));
}

// ---- conditional compiler edges: a struct with no CompareTo ----

static BOUNDS_PROPS: [PropDef; 1] = [PropDef {
    name: "Bounds",
    kind: Kind::Rect2D,
    read: GM,
    write: GM,
    get: |_, _| {
        Value::Rect(muo_primitives::Rectangle2D::from_points(
            muo_primitives::Point2D::new(1, 2),
            muo_primitives::Point2D::new(3, 4),
        ))
    },
    set: None,
}];
static BOUNDS_SUBJECT: ItemType = ItemType {
    full_name: "Server.Tests.BoundsSubject",
    parent: Some(&ITEM_BASE),
    ..ITEM_BASE
};

fn bounds_check(prop: &str, op: &str, value: &str) -> Result<bool, String> {
    let (mut w, from) = base_world();
    w.types.register_item(&BOUNDS_SUBJECT, &[]);
    props::add(&mut w, "Server.Tests.BoundsSubject", &BOUNDS_PROPS);
    let id = w.new_item(&BOUNDS_SUBJECT);
    let level = w.mobile(from).unwrap().access_level;
    let c = ObjectConditional::parse_direct(&w, level, &args(&["BoundsSubject", prop, op, value]))?;
    Ok(c.check(&mut w, Targeted::Entity(EntityId::Item(id))))
}

#[test]
fn non_comparable_struct_compares_by_value_equality() {
    assert_eq!(bounds_check("Bounds", "=", "(1, 2)+(3, 4)"), Ok(true));
    assert_eq!(bounds_check("Bounds", "=", "(1, 2)+(3, 5)"), Ok(false));
    assert_eq!(bounds_check("Bounds", "!=", "(1, 2)+(3, 4)"), Ok(false));
    assert_eq!(bounds_check("Bounds", "!=", "(1, 2)+(3, 5)"), Ok(true));
}

// `Rectangle2D` is a `[PropertyObject]`: `where` drills into X, Y, Width, Height of the value the getter returned.
#[test]
fn rectangle_sub_properties_read_the_parent_value() {
    assert_eq!(bounds_check("Bounds.X", "=", "1"), Ok(true));
    assert_eq!(bounds_check("Bounds.Width", "=", "2"), Ok(true));
    assert_eq!(bounds_check("Bounds.Height", ">", "1"), Ok(true));
    assert_eq!(bounds_check("Bounds.Height", ">", "2"), Ok(false));
}

// Only == and != are meaningful without a CompareTo; a relational operator is an error at compile time, not garbage
// at run time.
#[test]
fn non_comparable_struct_rejects_relational_operators() {
    for op in [">", ">=", "<", "<="] {
        assert!(bounds_check("Bounds", op, "(1, 2)+(3, 4)").is_err(), "{op}");
    }
}

// ---- nullable property conditions ----

#[derive(Clone, Default)]
struct NullSubj {
    count: Option<i64>,
    delay: Option<muo_primitives::TimeSpan>,
    skill: Option<i64>,
}

fn ns(w: &World, e: EntityId) -> NullSubj {
    match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|i| i.ext.get::<NullSubj>())
            .cloned()
            .unwrap_or_default(),
        EntityId::Mobile(_) => NullSubj::default(),
    }
}

const NULLABLE_INT: Kind = Kind::Nullable(&Kind::Plain(PropType::Numeric(Numeric::Int)));
static NULLABLE_PROPS: [PropDef; 3] = [
    PropDef {
        name: "Count",
        kind: NULLABLE_INT,
        read: GM,
        write: GM,
        get: |w, e| ns(w, e).count.map_or(Value::Null, Value::Int),
        set: None,
    },
    PropDef {
        name: "Delay",
        kind: Kind::Nullable(&Kind::TimeSpan),
        read: GM,
        write: GM,
        get: |w, e| ns(w, e).delay.map_or(Value::Null, Value::TimeSpan),
        set: None,
    },
    PropDef {
        name: "Skill",
        kind: Kind::Nullable(&Kind::Plain(PropType::Enum(SKILLS))),
        read: GM,
        write: GM,
        get: |w, e| ns(w, e).skill.map_or(Value::Null, Value::Enum),
        set: None,
    },
];
static NULLABLE_SUBJECT: ItemType = ItemType {
    full_name: "Server.Tests.NullableSubject",
    parent: Some(&ITEM_BASE),
    ..ITEM_BASE
};

fn nullable_world() -> (World, MobileId) {
    let (mut w, from) = base_world();
    w.types.register_item(&NULLABLE_SUBJECT, &[]);
    props::add(&mut w, "Server.Tests.NullableSubject", &NULLABLE_PROPS);
    (w, from)
}

fn subject_with(w: &mut World, f: impl FnOnce(&mut NullSubj)) -> ItemId {
    let id = w.new_item(&NULLABLE_SUBJECT);
    let mut d = NullSubj::default();
    f(&mut d);
    w.item_mut(id).unwrap().ext.insert(d);
    id
}

fn nullable_check(
    w: &mut World,
    from: MobileId,
    s: ItemId,
    binding: &str,
    op: &str,
    value: &str,
) -> bool {
    let level = w.mobile(from).unwrap().access_level;
    ObjectConditional::parse_direct(w, level, &args(&["NullableSubject", binding, op, value]))
        .expect("compiles")
        .check(w, Targeted::Entity(EntityId::Item(s)))
}

#[test]
fn set_value_compares_by_equality() {
    let (mut w, from) = nullable_world();
    let s = subject_with(&mut w, |d| d.count = Some(5));
    assert!(nullable_check(&mut w, from, s, "Count", "=", "5"));
    assert!(!nullable_check(&mut w, from, s, "Count", "=", "4"));
    assert!(nullable_check(&mut w, from, s, "Count", "!=", "4"));
    assert!(!nullable_check(&mut w, from, s, "Count", "!=", "5"));
}

#[test]
fn set_value_compares_relationally() {
    let (mut w, from) = nullable_world();
    let s = subject_with(&mut w, |d| d.count = Some(5));
    assert!(nullable_check(&mut w, from, s, "Count", ">", "4"));
    assert!(!nullable_check(&mut w, from, s, "Count", "<", "4"));
    assert!(nullable_check(&mut w, from, s, "Count", ">=", "5"));
    assert!(nullable_check(&mut w, from, s, "Count", "<=", "5"));
}

#[test]
fn unset_value_equals_null() {
    let (mut w, from) = nullable_world();
    let unset = subject_with(&mut w, |_| {});
    let five = subject_with(&mut w, |d| d.count = Some(5));
    assert!(nullable_check(&mut w, from, unset, "Count", "=", "null"));
    assert!(!nullable_check(&mut w, from, five, "Count", "=", "null"));
    assert!(nullable_check(&mut w, from, five, "Count", "!=", "null"));
}

// Lifted semantics: null is never greater, lesser, or equal to a value -- only unequal.
#[test]
fn unset_value_satisfies_no_relation() {
    let (mut w, from) = nullable_world();
    let s = subject_with(&mut w, |_| {});
    assert!(!nullable_check(&mut w, from, s, "Count", "=", "0"));
    assert!(nullable_check(&mut w, from, s, "Count", "!=", "0"));
    for op in [">", ">=", "<", "<="] {
        assert!(!nullable_check(&mut w, from, s, "Count", op, "0"), "{op}");
    }
}

// A struct that compares through CompareTo rather than a primitive operator.
#[test]
fn nullable_struct_compares_through_compare_to() {
    let (mut w, from) = nullable_world();
    let set = subject_with(&mut w, |d| {
        d.delay = Some(muo_primitives::TimeSpan::from_secs_f64(5.0))
    });
    assert!(nullable_check(&mut w, from, set, "Delay", "=", "00:00:05"));
    assert!(nullable_check(&mut w, from, set, "Delay", ">", "00:00:04"));
    assert!(!nullable_check(&mut w, from, set, "Delay", "<", "00:00:04"));
    assert!(!nullable_check(&mut w, from, set, "Delay", "=", "null"));

    let unset = subject_with(&mut w, |_| {});
    assert!(nullable_check(&mut w, from, unset, "Delay", "=", "null"));
    assert!(!nullable_check(
        &mut w, from, unset, "Delay", ">", "00:00:04"
    ));
    assert!(!nullable_check(
        &mut w, from, unset, "Delay", "<", "00:00:04"
    ));
}

#[test]
fn nullable_enum_compares_by_name_and_order() {
    let (mut w, from) = nullable_world();
    let set = subject_with(&mut w, |d| d.skill = Some(25));
    assert!(nullable_check(&mut w, from, set, "Skill", "=", "Magery"));
    assert!(!nullable_check(&mut w, from, set, "Skill", "=", "Anatomy"));
    assert!(nullable_check(&mut w, from, set, "Skill", ">", "Alchemy"));
    assert!(!nullable_check(&mut w, from, set, "Skill", "=", "null"));

    let unset = subject_with(&mut w, |_| {});
    assert!(nullable_check(&mut w, from, unset, "Skill", "=", "null"));
    assert!(!nullable_check(&mut w, from, unset, "Skill", "=", "Magery"));
    assert!(!nullable_check(
        &mut w, from, unset, "Skill", ">", "Alchemy"
    ));
}

// `sort by` on a nullable: values order by the underlying type and an unset value takes a consistent place at one
// end, the same convention a null reference already had.
#[test]
fn sorting_on_a_nullable_orders_values_and_places_unset_consistently() {
    let (mut w, _) = nullable_world();
    let class = props::find_class(&w, "NullableSubject").unwrap();
    let b = Binding::bind(&w, &class, AccessLevel::Developer, "Count", Access::READ).unwrap();
    let c = vec![(b, true)];
    let low = subject_with(&mut w, |d| d.count = Some(1));
    let high = subject_with(&mut w, |d| d.count = Some(2));
    let unset = subject_with(&mut w, |_| {});
    assert!(cmp3(&mut w, &c, low, high) < 0);
    assert!(cmp3(&mut w, &c, high, low) > 0);
    assert_eq!(0, cmp3(&mut w, &c, low, low));
    let forward = cmp3(&mut w, &c, high, unset);
    assert_ne!(0, forward);
    assert_eq!(-forward.signum(), cmp3(&mut w, &c, unset, high).signum());
    let other_unset = subject_with(&mut w, |_| {});
    assert_eq!(0, cmp3(&mut w, &c, unset, other_unset));
}

// DistinctCompiler orders through SortCompiler.Build: a property with no CompareTo is "Property is not comparable."
#[test]
fn distinct_rejects_a_non_comparable_property() {
    use crate::generic::conditional::Extensions;
    let (mut w, from) = base_world();
    w.types.register_item(&BOUNDS_SUBJECT, &[]);
    props::add(&mut w, "Server.Tests.BoundsSubject", &BOUNDS_PROPS);
    let level = w.mobile(from).unwrap().access_level;
    let mut with_where = args(&[
        "where",
        "BoundsSubject",
        "Bounds",
        "!=",
        "(1, 2)+(3, 5)",
        "distinct",
        "Bounds",
    ]);
    let r = Extensions::parse(&w, level, &mut with_where);
    assert_eq!(r.err().as_deref(), Some("Property is not comparable."));
}

// ---- AdvancedSearchConditions: every public property resolves, not only [CommandProperty] ones; @"text" is literal ----

#[test]
fn advanced_search_leaf_reads_public_properties_and_quoted_text() {
    use muo_content::props::ClassInfo;
    let (mut w, _) = base_world();
    w.types.register_item(&EMITTER_SUBJECT, &[]);
    let class = ClassInfo {
        full_name: "Server.Tests.EmitterSubject",
        chain: vec!["Server.Item", "Server.Tests.EmitterSubject"],
        is_item: true,
    };
    let id = w.new_item(&EMITTER_SUBJECT);
    let e = EntityId::Item(id);
    // Deleted is a public property, not a [CommandProperty]
    let deleted =
        crate::generic::conditional::search_leaf(&w, &class, false, "=", "deleted", "false")
            .expect("Deleted resolves");
    assert!(deleted.matches(&mut w, e));
    let negated =
        crate::generic::conditional::search_leaf(&w, &class, true, "=", "Deleted", "false")
            .unwrap();
    assert!(!negated.matches(&mut w, e));
    assert!(
        crate::generic::conditional::search_leaf(&w, &class, false, "=", "NoSuchProperty", "1")
            .is_none()
    );
    // DefaultName is null for a plain item: "~" with null is the four-letter word, @"..." is the text inside
    let named = crate::generic::conditional::search_leaf(
        &w,
        &class,
        false,
        "~",
        "DefaultName",
        "@\"null\"",
    )
    .unwrap();
    assert!(!named.matches(&mut w, e));
}
