//! Advanced search tests: the property test grammar, the result order, the paging
//! and the filtering of `DoSearch`.
//!
//! A class lists its properties (`muo_content::props`), so `Subject` here is an item class with a property table of
//! its own. Where a test needs a type with no property kind, the stand-in is named at the test.

use muo_content::props::{self, Kind, PropDef, Value};
use muo_content::utilities::types::{Numeric, Parsed, PropType};
use muo_primitives::{Point3D, Serial};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::layer::Layer;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;

use super::*;

const GM: AccessLevel = AccessLevel::GameMaster;

fn parse_text(s: &str) -> Option<Parsed> {
    Some(Parsed::Str(s.to_string()))
}

#[derive(Clone, Copy, Default)]
struct SubjectData {
    child: Option<ItemId>,
    maybe: Option<i32>,
    venom: Option<muo_server::poison::PoisonId>,
}

fn data(w: &World, e: EntityId) -> SubjectData {
    match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|i| i.ext.get::<SubjectData>())
            .copied()
            .unwrap_or_default(),
        EntityId::Mobile(_) => SubjectData::default(),
    }
}

static SUBJECT_PROPS: [PropDef; 11] = [
    // `decimal Price`: only double and float take the epsilon path, a decimal compares exactly
    PropDef {
        name: "Price",
        kind: Kind::Plain(PropType::Decimal),
        read: GM,
        write: GM,
        get: |_, _| Value::Float(0.300_000_000_01),
        set: None,
    },
    PropDef {
        name: "Weight",
        kind: Kind::Plain(PropType::Double),
        read: GM,
        write: GM,
        get: |_, _| Value::Float(0.1 + 0.2),
        set: None,
    },
    // `float Ratio = 0.1f`
    PropDef {
        name: "Ratio",
        kind: Kind::Plain(PropType::Float),
        read: GM,
        write: GM,
        get: |_, _| Value::Float(f64::from(0.1f32)),
        set: None,
    },
    PropDef {
        name: "Delay",
        kind: Kind::TimeSpan,
        read: GM,
        write: GM,
        get: |_, _| Value::TimeSpan(muo_primitives::TimeSpan::from_mins_f64(5.0)),
        set: None,
    },
    // `Poison Venom`
    PropDef {
        name: "Venom",
        kind: Kind::Poison,
        read: GM,
        write: GM,
        get: |w, e| Value::Poison(data(w, e).venom),
        set: None,
    },
    // `Guid Id`: a value type with no hot path, compared as its text
    PropDef {
        name: "Id",
        kind: Kind::Plain(PropType::Parsable(parse_text)),
        read: GM,
        write: GM,
        get: |_, _| Value::Str("00000000-0000-0000-0000-000000000001".into()),
        set: None,
    },
    // `LegacyParseType Legacy`: only a static Parse(string)
    PropDef {
        name: "Legacy",
        kind: Kind::Plain(PropType::Parsable(parse_text)),
        read: GM,
        write: GM,
        get: |_, _| Value::Str("alpha".into()),
        set: None,
    },
    // `object Reference`: no CompareTo; the stand-in is a point, which has none either
    PropDef {
        name: "Reference",
        kind: Kind::Plain(PropType::IPoint2D),
        read: GM,
        write: GM,
        get: |_, _| Value::Point3D(Point3D::new(1, 2, 0)),
        set: None,
    },
    PropDef {
        name: "Child",
        kind: Kind::Plain(PropType::Entity),
        read: GM,
        write: GM,
        get: |w, e| Value::Entity(data(w, e).child.map(EntityId::Item)),
        set: None,
    },
    // `int? Maybe`
    PropDef {
        name: "Maybe",
        kind: Kind::Nullable(&Kind::Plain(PropType::Numeric(Numeric::Int))),
        read: GM,
        write: GM,
        get: |w, e| {
            data(w, e)
                .maybe
                .map_or(Value::Null, |m| Value::Int(i64::from(m)))
        },
        set: None,
    },
    PropDef {
        name: "Hue",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Int(i64::from(w.item(i).map_or(0, |i| i.hue))),
            EntityId::Mobile(_) => Value::Null,
        },
        set: None,
    },
];

// Inner.Value (7): the item's amount
static INNER_PROPS: [PropDef; 1] = [PropDef {
    name: "Value",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(i64::from(w.item(i).map_or(0, |i| i.amount))),
        EntityId::Mobile(_) => Value::Null,
    },
    set: None,
}];

static SUBJECT: ItemType = ItemType {
    full_name: "Server.Tests.Subject",
    parent: Some(&ITEM_BASE),
    ..ITEM_BASE
};
static DERIVED: ItemType = ItemType {
    full_name: "Server.Tests.Derived",
    parent: Some(&SUBJECT),
    ..ITEM_BASE
};
static INNER: ItemType = ItemType {
    full_name: "Server.Tests.Inner",
    parent: Some(&ITEM_BASE),
    ..ITEM_BASE
};

fn world() -> World {
    let mut w = muo_server::map::test_support::world();
    muo_content::props::register_core(&mut w);
    props::add(&mut w, "Server.Tests.Subject", &SUBJECT_PROPS);
    props::add(&mut w, "Server.Tests.Inner", &INNER_PROPS);
    w
}

/// `new Subject()`: Hue 5, Name "Moongate", Movable, not Visible, Layer OneHanded, a Child with Value 7.
fn subject(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = w.new_item(ty);
    let inner = w.new_item(&INNER);
    w.item_mut(inner).unwrap().amount = 7;
    let i = w.item_mut(id).unwrap();
    i.hue = 5;
    i.name = Some("Moongate".into());
    i.layer = Layer::OneHanded;
    i.ext = Ext::new(SubjectData {
        child: Some(inner),
        maybe: None,
        venom: None,
    });
    ops::set_movable(w, id, true);
    ops::set_visible(w, id, false);
    id
}

fn set_data(w: &mut World, id: ItemId, f: impl FnOnce(&mut SubjectData)) {
    let i = w.item_mut(id).unwrap();
    let mut d = i.ext.get::<SubjectData>().copied().unwrap_or_default();
    f(&mut d);
    i.ext = Ext::new(d);
}

/// `AdvancedSearchConditions.Compile(typeof(Subject), test)(subject)`.
fn check_on(w: &mut World, id: ItemId, test: &str) -> bool {
    let e = EntityId::Item(id);
    let mut chain = props::class_chain(w, e);
    chain.reverse();
    let class = ClassInfo {
        full_name: chain.last().copied().unwrap(),
        chain,
        is_item: true,
    };
    build_predicate(w, &class, test)
        .is_some_and(|groups| groups.iter().any(|g| g.iter().all(|c| c.matches(w, e))))
}

fn check(test: &str) -> bool {
    let mut w = world();
    let s = subject(&mut w, &SUBJECT);
    check_on(&mut w, s, test)
}

#[test]
fn unusable_leaf_is_no_match_and_does_not_throw() {
    for test in [
        "Hue=abc",         // not a number
        "Hue=99999999999", // overflows int
        "Hue=0xZZ",        // bad hex
        "Layer=Bogus",     // not an enum member
        "Delay=notaspan",
        "Bogus=1", // no such property
        "Hue=",    // no value
        "Hue",     // no operator
    ] {
        assert!(!check(test), "{test}");
    }
}

#[test]
fn unusable_leaf_stays_no_match_under_negation() {
    assert!(!check("~Hue=abc"));
    assert!(!check("~Bogus=1"));
}

#[test]
fn numeric_operators() {
    assert!(check("Hue=5"));
    assert!(check("Hue==5"));
    assert!(check("Hue=0x5"));
    assert!(!check("Hue!=5"));
    assert!(check("Hue!4"));
    assert!(check("Hue>4"));
    assert!(check("Hue<6"));
    assert!(check("Hue>=5"));
    assert!(check("Hue<=5"));
    assert!(!check("Hue~5"));
}

#[test]
fn negation_prefix() {
    assert!(!check("~Hue=5"));
    assert!(check("~Hue=4"));
}

#[test]
fn precedence() {
    // '|' binds looser than '@'.
    for (test, expected) in [
        ("Movable=1", true),
        ("Visible=1", false),
        ("Visible=1@Visible=1|Movable=1", true), // (F&&F)||T
        ("Movable=1|Visible=1@Visible=1", true), // T||(F&&F)
        ("Movable=1@Visible=1", false),
        ("Movable=1@Movable=1", true),
        ("Visible=1|Visible=1", false),
    ] {
        assert_eq!(check(test), expected, "{test}");
    }
}

#[test]
fn string_operators() {
    for (test, expected) in [
        ("Name=Moongate", true),
        ("Name=moongate", false),
        ("Name!Moongate", false),
        ("Name>Moon", true),
        ("Name<gate", true),
        ("Name~ong", true),
        ("Name~>moon", true),
        ("Name~<GATE", true),
        ("Name~~ONG", true),
        ("Name~=MOONGATE", true),
        ("Name~!MOONGATE", false),
        ("Name>=Moon", false), // no such string operator
    ] {
        assert_eq!(check(test), expected, "{test}");
    }
}

#[test]
fn string_null_is_the_null_string_for_equality_and_text_otherwise() {
    let mut w = world();
    let unnamed = subject(&mut w, &SUBJECT);
    w.item_mut(unnamed).unwrap().name = None;
    let named = subject(&mut w, &SUBJECT);
    let nullable = subject(&mut w, &SUBJECT);
    w.item_mut(nullable).unwrap().name = Some("nullable".into());

    assert!(check_on(&mut w, unnamed, "Name=null"));
    assert!(!check_on(&mut w, named, "Name=null"));
    assert!(!check_on(&mut w, unnamed, "Name~null"));
    assert!(check_on(&mut w, nullable, "Name~null"));
}

#[test]
fn boolean_accepts_switch_words_and_only_equality() {
    assert!(check("Movable=true"));
    assert!(check("Movable=1"));
    assert!(check("Movable=on"));
    assert!(check("Movable=Enabled"));
    assert!(check("Visible=off"));
    assert!(!check("Movable=maybe"));
    assert!(!check("Movable>0"));
}

#[test]
fn enum_ignores_case_and_orders_by_value() {
    assert!(check("Layer=onehanded"));
    assert!(check("Layer>Invalid"));
    assert!(!check("Layer=TwoHanded"));
}

// Floating point compares to a tolerance derived from the typed value.
#[test]
fn floating_point_uses_epsilon() {
    assert!(check("Weight=0.3"));
    assert!(!check("Weight=0.31"));
    assert!(check("Weight>0.2"));
    assert!(check("Weight<=0.3"));
    assert!(check("Ratio=0.1"));
    assert!(!check("Ratio=0.2"));
}

// `AdvancedSearchConditions.Leaf` sends only double and float to `EpsilonLeaf`; a decimal is an exact comparison.
#[test]
fn decimal_compares_exactly() {
    assert!(check("Price=0.30000000001"));
    assert!(!check("Price=0.3"));
    assert!(check("Price>0.3"));
}

#[test]
fn time_span_parses_and_compares() {
    assert!(check("Delay=00:05:00"));
    assert!(!check("Delay=00:10:00"));
    assert!(check("Delay>00:01:00"));
}

#[test]
fn value_type_without_hot_path_parses_through_types() {
    assert!(check("Id=00000000-0000-0000-0000-000000000001"));
    assert!(!check("Id=00000000-0000-0000-0000-000000000002"));
}

#[test]
fn legacy_parse_string_parses_through_types() {
    assert!(check("Legacy=alpha"));
    assert!(!check("Legacy=beta"));
}

// A reference type with no CompareTo answers equality only; ordering is no match, not a throw.
#[test]
fn reference_type_ordering_is_no_match_and_does_not_throw() {
    assert!(!check("Reference>whatever"));
}

#[test]
fn dotted_name_walks_into_the_property() {
    let mut w = world();
    let s = subject(&mut w, &SUBJECT);
    let orphan = subject(&mut w, &SUBJECT);
    set_data(&mut w, orphan, |d| d.child = None);
    assert!(check_on(&mut w, s, "Child.Value=7"));
    assert!(!check_on(&mut w, s, "Child.Value=8"));
    assert!(!check_on(&mut w, orphan, "Child.Value=7"));
    assert!(!check_on(&mut w, orphan, "~Child.Value=7"));
}

#[test]
fn nullable_compares() {
    let mut w = world();
    let unset = subject(&mut w, &SUBJECT);
    let five = subject(&mut w, &SUBJECT);
    set_data(&mut w, five, |d| d.maybe = Some(5));
    assert!(check_on(&mut w, unset, "Maybe=null"));
    assert!(!check_on(&mut w, unset, "Maybe=5"));
    assert!(check_on(&mut w, five, "Maybe=5"));
    assert!(check_on(&mut w, five, "Maybe>4"));
}

// ---- types ----

// Poison is a reference type that parses through the shared converter (`Poison.Parse`: the registered singleton), so
// "= Lethal" matches the held poison.
#[test]
fn poison_reference_type_parsed_via_types() {
    let mut w = world();
    muo_content::misc::poison_kinds::configure(&mut w);
    let lethal = w
        .resources
        .get::<muo_server::poison::Poisons>()
        .unwrap()
        .get_poison("Lethal")
        .unwrap()
        .id();
    let s = subject(&mut w, &SUBJECT);
    set_data(&mut w, s, |d| d.venom = Some(lethal));
    assert!(check_on(&mut w, s, "Venom=Lethal"));
    assert!(!check_on(&mut w, s, "Venom=Lesser"));
    assert!(!check_on(&mut w, s, "Venom=notapoison"));
}

// Two runtime types resolving the same declared property share one compiled predicate: the Rust predicate is
// compiled per class chain, and a subclass's chain resolves the same declared `Hue`.
#[test]
fn subclasses_share_the_predicate_compiled_for_the_declaring_type() {
    let mut w = world();
    let base = subject(&mut w, &SUBJECT);
    let derived = subject(&mut w, &DERIVED);
    assert!(check_on(&mut w, base, "Hue=5"));
    assert!(check_on(&mut w, derived, "Hue=5"));
}

// ---- paging ----

#[test]
fn visible_count_is_correct() {
    // [InlineData(total, from, max, expected)]
    for (total, from, max, expected) in [
        (20, 0, 18, 18),
        (20, 18, 18, 2),
        (5, 0, 18, 5),
        (0, 0, 18, 0),
    ] {
        assert_eq!(
            visible_count(total, from, max),
            expected,
            "{total} {from} {max}"
        );
    }
}

// ---- result order ----

fn result(w: &mut World, name: &str, map: Option<MapId>) -> SearchResult {
    let item = w.new_item(&ITEM_BASE);
    w.item_mut(item).unwrap().item_id = 0x1;
    SearchResult {
        name: name.to_string(),
        type_name: "Item",
        location: Point3D::ZERO,
        map: map.or(Some(MapId::FELUCCA)),
        parent: None,
        entity: Some(EntityId::Item(item)),
        selected: false,
    }
}

fn six_equal_keyed(w: &mut World) -> Vec<SearchResult> {
    (0..6).map(|_| result(w, "same", None)).collect()
}

fn serials(v: &[SearchResult]) -> Vec<u32> {
    v.iter().map(serial_of).collect()
}

fn shuffled(in_order: &[SearchResult], permutation: [usize; 6]) -> Vec<SearchResult> {
    permutation.iter().map(|i| in_order[*i].clone()).collect()
}

fn sort_with(w: &mut World, from: MobileId, key: SortKey, reverse: bool, v: &mut [SearchResult]) {
    muo_primitives::dotnet::net_sort(v, |a, b| compare(w, from, key, reverse, a, b));
}

fn viewer(w: &mut World) -> MobileId {
    muo_server::map::test_support::place_mobile(w, MapId::TRAMMEL, Point3D::new(1000, 1000, 0))
}

#[test]
fn equal_keys_order_by_serial_from_any_arrival_order() {
    let mut w = world();
    let from = viewer(&mut w);
    for (key, reverse) in [
        (SortKey::Type, false),
        (SortKey::Type, true),
        (SortKey::Name, false),
        (SortKey::Name, true),
        (SortKey::Map, false),
        (SortKey::Map, true),
        (SortKey::Selected, false),
        (SortKey::Selected, true),
    ] {
        let in_order = six_equal_keyed(&mut w);
        let mut first = shuffled(&in_order, [3, 0, 5, 1, 4, 2]);
        let mut second = shuffled(&in_order, [5, 4, 3, 2, 1, 0]);
        sort_with(&mut w, from, key, reverse, &mut first);
        sort_with(&mut w, from, key, reverse, &mut second);
        assert_eq!(serials(&first), serials(&second));
        assert_eq!(serials(&first), serials(&in_order));
    }
    // AdvancedSearchResultSerialComparer: by serial alone
    let in_order = six_equal_keyed(&mut w);
    let mut v = shuffled(&in_order, [3, 0, 5, 1, 4, 2]);
    v.sort_by_key(serial_of);
    assert_eq!(serials(&v), serials(&in_order));
}

// Off-map results compare equal on range.
#[test]
fn range_comparer_orders_off_map_results_by_serial() {
    let mut w = world();
    let from = viewer(&mut w);
    for reverse in [false, true] {
        let in_order = six_equal_keyed(&mut w);
        let mut first = shuffled(&in_order, [3, 0, 5, 1, 4, 2]);
        let mut second = shuffled(&in_order, [5, 4, 3, 2, 1, 0]);
        sort_with(&mut w, from, SortKey::Range, reverse, &mut first);
        sort_with(&mut w, from, SortKey::Range, reverse, &mut second);
        assert_eq!(serials(&first), serials(&second));
    }
}

#[test]
fn reverse_flips_the_key_not_the_tie_break() {
    let mut w = world();
    let from = viewer(&mut w);
    let a1 = result(&mut w, "alpha", None);
    let a2 = result(&mut w, "alpha", None);
    let b1 = result(&mut w, "beta", None);

    let mut results = vec![b1.clone(), a2.clone(), a1.clone()];
    sort_with(&mut w, from, SortKey::Name, false, &mut results);
    assert_eq!(
        serials(&results),
        serials(&[a1.clone(), a2.clone(), b1.clone()])
    );

    sort_with(&mut w, from, SortKey::Name, true, &mut results);
    assert_eq!(serials(&results), serials(&[b1, a1, a2]));
}

// ---- the worker: `do_search` filters in one pass on the loop ----

fn run_search(w: &mut World, from: MobileId, filter: Filter) -> Vec<EntityId> {
    w.resources.get_mut::<Sessions>().0.insert(
        from,
        Session {
            filter,
            ..Session::default()
        },
    );
    do_search(w, from);
    w.resources.get::<Sessions>().unwrap().0[&from]
        .results
        .iter()
        .flatten()
        .filter_map(|r| r.entity)
        .collect()
}

#[test]
fn worker_property_test_filters_items_and_mobiles() {
    let mut w = world();
    let from = viewer(&mut w);
    let plain = ops::new_item_with_id(&mut w, &ITEM_BASE, 0x1);
    let hued = ops::new_item_with_id(&mut w, &ITEM_BASE, 0x1);
    w.item_mut(hued).unwrap().hue = 42;
    // `new Gold(1)`: any other item class stands in here
    let hued_too = ops::new_item_with_id(&mut w, &DERIVED, 0xEED);
    w.item_mut(hued_too).unwrap().hue = 7;
    let mobile = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    w.mobile_mut(mobile).unwrap().hue = 1002;

    let filter = Filter {
        filter_property_test: true,
        property_test: Some("Hue > 0".into()),
        ..Filter::default()
    };
    let matched = run_search(&mut w, from, filter);

    // `from` (hue 0) is not among them: the viewer is not pushed
    assert_eq!(matched.len(), 3);
    assert!(matched.contains(&EntityId::Item(hued)));
    assert!(matched.contains(&EntityId::Item(hued_too)));
    assert!(matched.contains(&EntityId::Mobile(mobile)));
    assert!(!matched.contains(&EntityId::Item(plain)));
}

// The map boxes are independent checks: an entity on any ticked map passes.
#[test]
fn worker_several_maps_ticked_matches_any_of_them() {
    let mut w = world();
    // Malas (map 3) is not among the test world's maps
    let def = muo_data::map_definitions::MapDefinition {
        index: 3,
        id: 3,
        file_index: 3,
        name: "Malas".to_string(),
        width: 2560,
        height: 2048,
        season: 0,
        rules: muo_server::map::MapRules::empty(),
        count: 1,
    };
    muo_server::map::loader::register_map(&mut w, &def).expect("malas");
    let from = viewer(&mut w);
    let at = |w: &mut World, map: Option<MapId>| {
        let id = ops::new_item_with_id(w, &ITEM_BASE, 0x1);
        if let Some(map) = map {
            ops::move_to_world_map(w, id, Point3D::new(1000, 1000, 0), Some(map));
        }
        id
    };
    let fel = at(&mut w, Some(MapId::FELUCCA));
    let tram = at(&mut w, Some(MapId::TRAMMEL));
    let internal = at(&mut w, None); // starts on Map.Internal
    let malas = at(&mut w, Some(MapId(3)));

    let filter = Filter {
        filter_felucca: true,
        filter_trammel: true,
        filter_internal_map: true,
        hide_valid_internal_map: false,
        ..Filter::default()
    };
    let matched = run_search(&mut w, from, filter);
    assert!(matched.contains(&EntityId::Item(fel)));
    assert!(matched.contains(&EntityId::Item(tram)));
    assert!(matched.contains(&EntityId::Item(internal)));
    assert!(!matched.contains(&EntityId::Item(malas)));
}

#[test]
fn worker_deleted_entity_is_skipped() {
    let mut w = world();
    let from = viewer(&mut w);
    let item = ops::new_item_with_id(&mut w, &ITEM_BASE, 0x1);
    muo_server::map::test_support::delete_item(&mut w, item);
    // no filters: everything matches
    let matched = run_search(&mut w, from, Filter::default());
    assert!(!matched.contains(&EntityId::Item(item)));
    let _ = Serial::ZERO;
}
