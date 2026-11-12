//! Tests of the property table and the `where` / `order by` / `limit` modifiers (generic-commands.md).

use muo_content::items::leaves::LONGSWORD;
use muo_content::props;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::target::Targeted;

use crate::generic::conditional::{Extensions, ObjectConditional};
use crate::properties;

fn world() -> (World, MobileId) {
    let mut w = World::new();
    w.types.register_item(&LONGSWORD, &[]);
    props::register_core(&mut w);
    let gm = w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE);
    w.mobile_mut(gm).unwrap().access_level = AccessLevel::GameMaster;
    (w, gm)
}

fn sword(w: &mut World, hue: i32, name: Option<&str>) -> ItemId {
    let id = (LONGSWORD.construct.unwrap())(w);
    w.item_mut(id).unwrap().hue = hue;
    w.item_mut(id).unwrap().name = name.map(str::to_string);
    id
}

fn args(s: &str) -> Vec<String> {
    muo_server::commands::split(s)
}

#[test]
fn get_and_set_go_through_the_table_with_access_checks() {
    let (mut w, gm) = world();
    let s = sword(&mut w, 0x21, None);
    let e = EntityId::Item(s);
    assert_eq!(
        properties::get_value(&mut w, gm, e, "hue"),
        "Hue = 33 (0x21)"
    );
    assert_eq!(
        properties::set_value(&mut w, gm, e, "Hue", "0x30"),
        "Property has been set."
    );
    assert_eq!(w.item(s).unwrap().hue, 0x30);
    assert_eq!(
        properties::set_value(&mut w, gm, e, "Serial", "5"),
        "Property 'Serial' is read only."
    );
    assert_eq!(
        properties::get_value(&mut w, gm, e, "Nope"),
        "Property 'Nope' not found."
    );

    w.mobile_mut(gm).unwrap().access_level = AccessLevel::Counselor;
    assert_eq!(
        properties::set_value(&mut w, gm, e, "Hue", "1"),
        "You must be at least Game Master to set the property 'Hue'."
    );
    assert!(properties::get_value(&mut w, gm, e, "Serial").starts_with("Serial = "));
}

#[test]
fn a_string_property_shows_quoted_and_null_is_the_four_letter_string_only_when_quoted() {
    let (mut w, gm) = world();
    let s = sword(&mut w, 0, Some("null"));
    let e = EntityId::Item(s);
    assert_eq!(
        properties::get_value(&mut w, gm, e, "Name"),
        "Name = @\"null\""
    );
    assert_eq!(
        properties::set_value(&mut w, gm, e, "Name", "(-null-)"),
        "Property has been set."
    );
    assert_eq!(w.item(s).unwrap().name, None);
}

#[test]
fn increase_adds_and_reports() {
    let (mut w, gm) = world();
    let s = sword(&mut w, 10, None);
    let r = properties::increase_value(&mut w, gm, EntityId::Item(s), &args("Hue 5"));
    assert_eq!(r, "The property has been increased.");
    assert_eq!(w.item(s).unwrap().hue, 15);
    assert_eq!(
        properties::increase_value(&mut w, gm, EntityId::Item(s), &args("Hue 0")),
        "Zero is not a valid value to offset."
    );
}

fn matching(w: &mut World, gm: MobileId, text: &str, items: &[ItemId]) -> Vec<ItemId> {
    let mut a = args(text);
    let level = w.mobile(gm).unwrap().access_level;
    let ext = Extensions::parse(w, level, &mut a).unwrap();
    let mut list: Vec<Targeted> = items
        .iter()
        .map(|i| Targeted::Entity(EntityId::Item(*i)))
        .filter(|o| ext.is_valid(w, *o))
        .collect();
    ext.filter(w, &mut list);
    list.into_iter()
        .map(|o| match o {
            Targeted::Entity(EntityId::Item(i)) => i,
            _ => unreachable!(),
        })
        .collect()
}

#[test]
fn where_combines_with_and_or_not_and_string_operators() {
    let (mut w, gm) = world();
    let a = sword(&mut w, 0, Some("Bob's Blade"));
    let b = sword(&mut w, 1, Some("Alice"));
    let c = sword(&mut w, 2, None);
    let all = [a, b, c];
    assert_eq!(matching(&mut w, gm, "where Item Hue = 0", &all), vec![a]);
    assert_eq!(
        matching(&mut w, gm, "where Item Hue = 0 or Hue = 1", &all),
        vec![a, b]
    );
    assert_eq!(
        matching(&mut w, gm, "where Item not Hue = 0", &all),
        vec![b, c]
    );
    assert_eq!(
        matching(&mut w, gm, "where Item Hue >= 1", &all),
        vec![b, c]
    );
    assert_eq!(
        matching(&mut w, gm, "where Item Name starts~ bob", &all),
        vec![a]
    );
    assert_eq!(
        matching(&mut w, gm, "where Item Name = null", &all),
        vec![c]
    );
    // A relational operator on a name is by string order; contains needs a string property.
    assert!(
        Extensions::parse(
            &w,
            AccessLevel::GameMaster,
            &mut args("where Item Hue contains 1")
        )
        .is_err()
    );
    // `where Mobile` selects no item.
    assert_eq!(
        matching(&mut w, gm, "where Mobile Hits > 0", &all),
        Vec::<ItemId>::new()
    );
}

#[test]
fn modifiers_apply_in_the_documented_order_whatever_the_typing_order() {
    let (mut w, gm) = world();
    let a = sword(&mut w, 5, Some("a"));
    let b = sword(&mut w, 7, Some("b"));
    let c = sword(&mut w, 5, Some("c"));
    let d = sword(&mut w, 9, Some("d"));
    let all = [a, b, c, d];
    assert_eq!(
        matching(&mut w, gm, "limit 2 order by Hue desc where Item", &all),
        vec![d, b]
    );
    assert_eq!(
        matching(&mut w, gm, "where Item distinct Hue order by Hue", &all),
        vec![a, b, d]
    );
    // `-` belongs to the key before it: Hue ascending, ties by Name descending.
    assert_eq!(
        matching(&mut w, gm, "where Item order by Hue Name -", &all),
        vec![c, a, b, d]
    );
}

#[test]
fn a_where_clause_needs_read_access_and_a_known_type() {
    let (w, _) = world();
    assert_eq!(
        ObjectConditional::parse_direct(&w, AccessLevel::Player, &args("Item Hue = 1"))
            .err()
            .unwrap(),
        "You must be at least Game Master to read this property."
    );
    assert_eq!(
        ObjectConditional::parse_direct(&w, AccessLevel::Owner, &args("Nothing Hue = 1"))
            .err()
            .unwrap(),
        "No type with that name (Nothing) was found."
    );
}

#[test]
fn a_dotted_binding_through_a_null_link_is_no_match() {
    let (mut w, gm) = world();
    let a = sword(&mut w, 0, Some("blade"));
    // `Parent` is null for an item on the ground; `Parent.Name` therefore never matches, even under `not`.
    assert_eq!(
        matching(&mut w, gm, "where Item Parent.Name = x", &[a]),
        Vec::<ItemId>::new()
    );
    assert_eq!(
        matching(&mut w, gm, "where Item not Parent.Name = x", &[a]),
        Vec::<ItemId>::new()
    );
}

#[test]
fn creature_and_player_tables_are_registered_for_their_classes() {
    let (mut w, gm) = world();
    let names: Vec<_> = props::props_of(&w, EntityId::Mobile(gm))
        .iter()
        .map(|p| p.name)
        .collect();
    assert!(
        names.contains(&"Hits") && names.contains(&"AnkhNextUse"),
        "{names:?}"
    );
    assert!(!names.contains(&"Team"));
    let _ = &mut w;
}

#[test]
fn constructor_arguments_parse_by_parameter_type_and_defaults_fill_in() {
    use muo_content::props::{Kind, Param, Value, parse_ctor_args};
    use muo_content::utilities::types::{Numeric, PropType};
    let (w, _) = world();
    let int = || Kind::Plain(PropType::Numeric(Numeric::Int));
    let params = [
        Param {
            name: "amount",
            kind: int(),
            default: None,
            ty: "",
        },
        Param {
            name: "hue",
            kind: int(),
            default: Some(Value::Int(7)),
            ty: "",
        },
    ];
    let a = |s: &str| args(s);
    assert_eq!(
        parse_ctor_args(&w, &params, &a("5")),
        Some(vec![Value::Int(5), Value::Int(7)])
    );
    assert_eq!(
        parse_ctor_args(&w, &params, &a("5 0x10")),
        Some(vec![Value::Int(5), Value::Int(16)])
    );
    assert_eq!(parse_ctor_args(&w, &params, &a("x")), None);
    assert_eq!(
        parse_ctor_args(&w, &params, &a("5 x")),
        Some(vec![Value::Int(5), Value::Int(7)])
    );
}

// ---- regression fixes: dotted where, order by, log file names ------------------------------------------------

#[test]
fn a_dotted_where_checks_every_link_when_it_is_parsed() {
    let (w, _) = world();
    let err = |text: &str| {
        ObjectConditional::parse_direct(&w, AccessLevel::Owner, &args(text))
            .err()
            .unwrap()
    };
    assert_eq!(err("Item Parent.Nope = 1"), "Property 'Nope' not found.");
    assert_eq!(err("Item Nope.Name = 1"), "Property 'Nope' not found.");
    // `Backpack` is a Container: its links are looked up on the Mobile's declared type and checked for access.
    assert_eq!(
        err("Mobile Backpack.Nope = 1"),
        "Property 'Nope' not found."
    );
    assert!(
        ObjectConditional::parse_direct(&w, AccessLevel::Owner, &args("Mobile Backpack.Name = x"))
            .is_ok()
    );
    // A link the viewer may not read fails at parse time, whatever the first link allows.
    assert_eq!(
        ObjectConditional::parse_direct(&w, AccessLevel::Counselor, &args("Item Parent.Hue = 1"))
            .err()
            .unwrap(),
        "You must be at least Game Master to read this property."
    );
}

#[test]
fn order_by_a_property_without_compare_to_is_an_error() {
    use muo_content::props::{Kind, PropDef, Value};
    use muo_content::utilities::types::PropType;
    static TABLE: &[PropDef] = &[PropDef {
        name: "Species",
        kind: Kind::Plain(PropType::Type),
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |_, _| Value::Null,
        set: None,
    }];
    let (mut w, _) = world();
    props::add(&mut w, "Server.Items.Longsword", TABLE);
    assert_eq!(
        Extensions::parse(
            &w,
            AccessLevel::GameMaster,
            &mut args("where Longsword order by Species")
        )
        .err()
        .unwrap(),
        "Property is not comparable."
    );
    // Entities compare by serial, so ordering by one works.
    assert!(
        Extensions::parse(
            &w,
            AccessLevel::GameMaster,
            &mut args("where Longsword order by Parent")
        )
        .is_ok()
    );
}

#[test]
fn the_command_log_file_is_named_by_to_long_date_string() {
    let t = muo_primitives::DateTime::from_ymd(2026, 11, 5).unwrap();
    assert_eq!(
        crate::logging::long_date_string(t),
        "Thursday, 05 November 2026"
    );
}

// ---- the [add path ---------------------------------------------------------------------------------------------

#[test]
fn add_builds_an_item_by_name_at_a_location() {
    let (mut w, gm) = world();
    // The command log goes under the base directory; keep it out of the source tree.
    w.core.base_directory = std::env::temp_dir().join(format!("muo-add-{}", std::process::id()));
    let before = w.items.len();
    let p = muo_primitives::Point3D::new(100, 100, 0);
    crate::add::invoke_args(
        &mut w,
        gm,
        p,
        p,
        &args("longsword set hue 0x22"),
        None,
        false,
        false,
    );
    assert_eq!(w.items.len(), before + 1);
    let built = w.items.values().last().unwrap();
    assert_eq!(built.ty.full_name, LONGSWORD.full_name);
    assert_eq!(built.hue, 0x22);
    // An unknown name builds nothing.
    crate::add::invoke_args(&mut w, gm, p, p, &args("nosuchthing"), None, false, false);
    assert_eq!(w.items.len(), before + 1);
}

#[test]
fn the_add_menu_finds_classes_by_name_and_loads_the_category_tree() {
    use crate::add_menus::{CagNode, CagTree, match_empty_ctor, open_categorized};
    let (mut w, gm) = world();
    let found = match_empty_ctor(&w, "longsw");
    assert_eq!(found.len(), 1);
    assert!(match_empty_ctor(&w, "   ").is_empty());

    let dir = std::env::temp_dir().join(format!("muo-cag-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("Data")).unwrap();
    std::fs::write(
        dir.join("Data/categorization.json"),
        r#"[{"category":"Items.Weapons","objects":[{"type":"Longsword","gfx":5,"hue":3},{"type":"NoSuchType"}]}]"#,
    )
    .unwrap();
    w.core.base_directory = dir.clone();
    open_categorized(&mut w, gm);
    let tree = w.resources.get::<CagTree>().unwrap();
    // root > Items > Weapons > [Longsword]; the object with an unknown type has no graphic to look up and is dropped.
    assert_eq!(tree.cats[0].title, "Add Menu");
    let CagNode::Category(items) = tree.cats[0].nodes.as_ref().unwrap()[0].clone() else {
        panic!()
    };
    let CagNode::Category(weapons) = tree.cats[items].nodes.as_ref().unwrap()[0].clone() else {
        panic!()
    };
    let nodes = tree.cats[weapons].nodes.as_ref().unwrap();
    assert_eq!(nodes.len(), 1);
    let CagNode::Object(o) = &nodes[0] else {
        panic!()
    };
    assert_eq!(
        (o.title().as_str(), o.item_id, o.hue),
        ("Longsword", Some(5), Some(3))
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_bad_extension_is_reported_before_a_where_that_does_not_compile() {
    let (w, _) = world();
    let err = |text: &str| {
        Extensions::parse(&w, AccessLevel::Owner, &mut args(text))
            .err()
            .unwrap()
    };
    // Parse runs from the last modifier back; `Compile` of the where waits for Optimize.
    assert_eq!(
        err("where Item Hue > nope order"),
        "Invalid ordering syntax."
    );
    assert_eq!(
        err("where Item Hue > nope distinct"),
        "Invalid distinction syntax."
    );
    assert_ne!(err("where Item Hue > nope"), "Invalid ordering syntax.");
}
