//! Template keys of the add-on, decoration and class-table families (`muo_content::templates::keys`).

use muo_content::items::addon::{self, Comp};
use muo_content::items::addon_container::{self, CComp};
use muo_content::templates::keys::{check_int, int, item_key};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::type_registry::TypeRegistry;
use serde_json::Value;

/// One `addonComponents` row: `[itemId, x, y, z]`, or `[{"itemId": n, "label": l}, x, y, z]` (a localized
/// component), or `[{"itemId": n, "type": "Server.Items.AnvilComponent"}, x, y, z]` (a component class).
fn check_component_row(v: &Value) -> Result<(), String> {
    let Some([c, x, y, z]) = v.as_array().map(Vec::as_slice) else {
        return Err(format!("expected [component, x, y, z], got {v}"));
    };
    [x, y, z].into_iter().try_for_each(check_int)?;
    match c {
        Value::Object(o) => {
            check_int(o.get("itemId").unwrap_or(&Value::Null))?;
            if let Some(l) = o.get("label") {
                check_int(l)?;
            }
            match o.get("type") {
                Some(t) if !t.is_string() => Err(format!("expected a class name, got {t}")),
                Some(_) if o.contains_key("label") => Err("a component has a label or a type".into()),
                _ => Ok(()),
            }
        }
        c => check_int(c),
    }
}

fn check_component_rows(v: &Value) -> Result<(), String> {
    match v.as_array() {
        Some(a) => a.iter().try_for_each(check_component_row),
        None => Err(format!("expected an array of rows, got {v}")),
    }
}

/// `AddComponent(new <component>, x, y, z)` for each row, on a `BaseAddon` or a `BaseAddonContainer`.
pub(crate) fn add_component_rows(w: &mut World, id: ItemId, v: &Value) {
    let container = addon_container::is_container_addon(w, id);
    for row in v.as_array().into_iter().flatten() {
        let Some([c, x, y, z]) = row.as_array().map(Vec::as_slice) else {
            continue;
        };
        let (x, y, z) = (int(x), int(y), int(z));
        let item_id = int(c.get("itemId").unwrap_or(c));
        let label = c.get("label").map(int);
        let ty = c.get("type").and_then(Value::as_str);
        if container {
            let cc = label.map_or(CComp::Plain(item_id), |l| CComp::Localized(item_id, l));
            addon_container::add_components(w, id, &[(cc, x, y, z)]);
            continue;
        }
        let comp = match ty {
            Some(name) => match crate::addons::item_type_named(w, name) {
                Some(t) => addon::new_component(w, t, item_id),
                None => {
                    tracing::warn!("Templates: addonComponents: unknown component class {name}");
                    continue;
                }
            },
            None => label
                .map_or(Comp::Plain(item_id), |l| Comp::Localized(item_id, l))
                .build(w),
        };
        addon::add_component(w, id, comp, x, y, z);
    }
}

pub(crate) fn register(types: &mut TypeRegistry) {
    // `BaseAddon()` / `BaseAddonContainer()` constructor bodies: their `AddComponent` rows.
    item_key(types, "addonComponents", check_component_rows, add_component_rows);
}
