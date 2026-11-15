//! The equipment classes whose constructors and constant class values moved to `Data/Templates/Items` and
//! `Data/Scripts/content/equipment`: every name a prop holds resolves, every templated graphic is the one a new
//! item gets, and the scripted classes are bound.

use muo_content::items::equip_props::{RESOURCES, SKILLS, races};
use muo_content::items::{armor, weapon};
use muo_content::templates::{self, Kind};
use muo_server::type_registry::TypeRegistry;
use serde_json::Value;

fn registry() -> TypeRegistry {
    let mut types = TypeRegistry::default();
    muo_content::register(&mut types);
    crate::register(&mut types);
    types
}

/// The classes whose constructor statements a script finishes (random or computed values).
const SCRIPTED: &[(&str, &str)] = &[
    (
        "Server.Items.AbyssReaver",
        "content/equipment/abyss_reaver.rhai",
    ),
    (
        "Server.Items.DetectiveBoots",
        "content/equipment/detective_boots.rhai",
    ),
    (
        "Server.Items.GlacialStaff",
        "content/equipment/glacial_staff.rhai",
    ),
    (
        "Server.Items.SlayerLongbow",
        "content/equipment/slayer_longbow.rhai",
    ),
    (
        "Server.Items.TheDryadBow",
        "content/equipment/the_dryad_bow.rhai",
    ),
    (
        "Server.Items.TheMostKnowledgePerson",
        "content/equipment/the_most_knowledge_person.rhai",
    ),
];

#[test]
fn every_equipment_prop_name_resolves() {
    let t = templates::table(&mut crate::test_world());
    let mut checked = 0;
    for (name, tmpl) in &t.items {
        for (key, v) in &tmpl.props {
            let ok = match (key.as_str(), v) {
                (_, Value::Null) => true,
                ("defSkill", Value::String(s)) => SKILLS.get(s).is_some(),
                ("defType", Value::String(s)) => weapon::TYPES.get(s).is_some(),
                ("defAnimation", Value::String(s)) => weapon::ANIMATIONS.get(s).is_some(),
                ("primaryAbility" | "secondaryAbility", Value::String(s)) => {
                    weapon::ABILITIES.get(s).is_some()
                }
                ("materialType", Value::String(s)) => armor::MATERIALS.get(s).is_some(),
                ("defMedAllowance", Value::String(s)) => armor::MED_ALLOWANCES.get(s).is_some(),
                ("defaultResource", Value::String(s)) => RESOURCES.get(s).is_some(),
                ("requiredRaces", Value::String(s)) => races(s).is_some(),
                ("damageTypes", Value::Object(o)) => o.keys().all(|k| {
                    ["phys", "fire", "cold", "pois", "nrgy", "chaos", "direct"]
                        .contains(&k.as_str())
                }),
                (
                    "defSkill" | "defType" | "defAnimation" | "primaryAbility" | "secondaryAbility"
                    | "materialType" | "defMedAllowance" | "defaultResource" | "requiredRaces"
                    | "damageTypes",
                    _,
                ) => false,
                _ => continue,
            };
            assert!(ok, "{name}: \"{key}\": {v}");
            checked += 1;
        }
    }
    assert!(checked > 500, "{checked} props");
}

#[test]
fn templated_graphics_are_the_constructed_item_ids() {
    let types = registry();
    let mut w = crate::test_world();
    let t = templates::table(&mut w);
    let mut checked = 0;
    for ty in types.item_types() {
        let Some(g) = t
            .get(Kind::Item, ty.full_name)
            .and_then(|tm| tm.props.get("graphic"))
            .and_then(Value::as_i64)
        else {
            continue;
        };
        let (Some(ctor), Some(steps)) = (
            ty.construct,
            t.get(Kind::Item, ty.full_name).map(|tm| &tm.item_steps),
        ) else {
            continue;
        };
        // a class whose own steps set "itemId" ends on that one
        if steps.iter().any(|s| {
            matches!(
                s,
                templates::item_steps::Step::ItemId(_) | templates::item_steps::Step::Flow(_)
            )
        }) {
            continue;
        }
        let id = ctor(&mut w);
        assert_eq!(
            i64::from(w.item(id).map_or(0, |i| i.item_id)),
            g,
            "{}",
            ty.full_name
        );
        checked += 1;
    }
    assert!(checked > 300, "{checked} classes");
}

#[test]
fn scripted_classes_name_their_scripts() {
    let t = templates::table(&mut crate::test_world());
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Data/Scripts");
    for (name, script) in SCRIPTED {
        let tmpl = t
            .get(Kind::Item, name)
            .unwrap_or_else(|| panic!("{name}: no template"));
        assert_eq!(tmpl.script.as_deref(), Some(*script), "{name}");
        assert!(base.join(script).is_file(), "{script} is missing");
    }
}
