//! Template keys of the non-equipment items (`muo_content::templates::keys`).

use muo_content::templates::keys::{check_bool, check_int, check_name, int, item_key};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::type_registry::TypeRegistry;

use crate::lights;
use crate::skill_items::potions;

/// A check for an array of `N` integers.
fn check_ints<const N: usize>(v: &serde_json::Value) -> Result<(), String> {
    match v.as_array() {
        Some(a) if a.len() == N => a.iter().try_for_each(check_int),
        _ => Err(format!("expected an array of {N} integers, got {v}")),
    }
}

/// The integers of a checked [`check_ints`] value.
fn ints<const N: usize>(v: &serde_json::Value) -> [i32; N] {
    let mut out = [0; N];
    if let Some(a) = v.as_array() {
        for (o, x) in out.iter_mut().zip(a) {
            *o = int(x);
        }
    }
    out
}

/// The names of a `(name, value)` table.
fn names<T>(table: &[(&'static str, T)]) -> Vec<&'static str> {
    table.iter().map(|e| e.0).collect()
}

/// The family state `T` of an item, to set one field of it.
fn ext_mut<T: 'static>(w: &mut World, id: ItemId) -> Option<&mut T> {
    w.item_mut(id).and_then(|i| i.ext.get_mut::<T>())
}

pub(crate) fn register(types: &mut TypeRegistry) {
    // `SpellScroll.SpellID`.
    item_key(types, "spellId", check_int, |w, id, v| {
        crate::skill_items::scrolls::set_spell_id(w, id, int(v));
    });
    // `Food.FillFactor`.
    item_key(types, "foodFillFactor", check_int, |w, id, v| {
        muo_content::items::food::set_fill_factor(w, id, int(v));
    });
    // `BasePotion.PotionEffect`, by name.
    item_key(
        types,
        "potionEffect",
        |v| check_name(v, &names(potions::EFFECT_NAMES)),
        |w, id, v| {
            let n = v.as_str().and_then(|s| potions::EFFECT_NAMES.iter().find(|e| e.0 == s));
            if let Some(&(_, e)) = n {
                potions::set_effect(w, id, e);
            }
        },
    );
    // `DyeTub.DyedHue` (the field only; the template's `hue` sets the hue).
    item_key(types, "dyedHue", check_int, |w, id, v| {
        crate::skill_items::dyes::set_dyed_hue_raw(w, id, int(v));
    });
    // `DyeTub.Redyable`.
    item_key(types, "redyable", check_bool, |w, id, v| {
        crate::skill_items::dyes::set_redyable(w, id, v.as_bool().unwrap_or(false));
    });
    // `MapItem.SetDisplay(x1, y1, x2, y2, w, h)`.
    item_key(types, "mapDisplay", check_ints::<6>, |w, id, v| {
        if let [x1, y1, x2, y2, wd, h] = ints::<6>(v) {
            crate::maps::set_display(w, id, x1, y1, x2, y2, wd, h);
        }
    });
    // `BaseLight.Duration = Burnout ? TimeSpan.FromMinutes(n) : TimeSpan.Zero`.
    item_key(types, "burnoutMinutes", check_int, |w, id, v| {
        let d = if lights::BURNOUT {
            muo_primitives::TimeSpan::from_mins(i64::from(int(v)))
        } else {
            muo_primitives::TimeSpan::ZERO
        };
        lights::set_duration(w, id, d);
    });
    // `BaseLight.Burning`.
    item_key(types, "burning", check_bool, |w, id, v| {
        lights::set_burning(w, id, v.as_bool().unwrap_or(false));
    });
    // `BaseLight.Protected`.
    item_key(types, "protected", check_bool, |w, id, v| {
        lights::set_protected(w, id, v.as_bool().unwrap_or(false));
    });
    // `BaseTool.UsesRemaining`.
    item_key(types, "toolUses", check_int, |w, id, v| {
        muo_content::items::tool::set_uses_remaining(w, id, int(v));
    });
    // `CookableFood.CookingLevel`.
    item_key(types, "cookingLevel", check_int, |w, id, v| {
        if let Some(d) = ext_mut::<crate::food::cooking::CookData>(w, id) {
            d.level = int(v);
        }
    });
}
