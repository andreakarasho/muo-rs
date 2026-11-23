//! Script functions added for muo_engines, muo_content and muo_multis content (constructors moved to scripts).
//! The `mlq_` functions are the statements of the ML quest givers' constructors (`ml_quests/npc_support.rs`).

use muo_primitives::utility;
use muo_server::mobile::part1::props_b;
use muo_server::type_registry::EntityType;
use rhai::{Engine, NativeCallContext};

use super::api::{Mobile, R, i32_of, live_m, with_world};

/// `Utility.Random<Name>Hue()` by the names the templates use (`"randomNeutralHue"`), 0 for an unknown name.
fn random_hue(rng: &mut muo_primitives::BuiltInRng, name: &str) -> i32 {
    match name {
        "randomPinkHue" => utility::random_pink_hue(rng),
        "randomBlueHue" => utility::random_blue_hue(rng),
        "randomGreenHue" => utility::random_green_hue(rng),
        "randomOrangeHue" => utility::random_orange_hue(rng),
        "randomRedHue" => utility::random_red_hue(rng),
        "randomYellowHue" => utility::random_yellow_hue(rng),
        "randomNeutralHue" => utility::random_neutral_hue(rng),
        "randomAnimalHue" => utility::random_animal_hue(rng),
        "randomNondyedHue" => utility::random_nondyed_hue(rng),
        "randomBrightHue" => utility::random_bright_hue(rng),
        _ => 0,
    }
}

/// `AddItem(new T(hue))` with `T` a short class name; `hue` < 0 leaves the item's own hue.
fn equip(ctx: &NativeCallContext, m: Mobile, class: &str, hue: i64) -> R<()> {
    with_world(ctx, |w| {
        live_m(w, m)?;
        let Some(EntityType::Item(t)) = w.types.find_type_by_name(class, false, false) else {
            return None;
        };
        let it = (t.construct?)(w);
        if hue >= 0
            && let Some(ty) = w.item(it).map(|i| i.ty)
        {
            (ty.set_hue)(w, it, i32_of(hue));
        }
        muo_server::mobile::part2::items::add_item(w, m.0, it);
        Some(())
    })
    .map(|_| ())
}

pub(crate) fn register(e: &mut Engine) {
    e.register_fn("random_hue", |ctx: NativeCallContext, name: &str| {
        with_world(&ctx, |w| i64::from(random_hue(&mut w.rng, name)))
    });
    e.register_fn("mlq_hair_hue", |ctx: NativeCallContext, m: Mobile| {
        with_world(&ctx, |w| w.mobile(m.0).map_or(0, |x| i64::from(x.hair_hue)))
    });
    // `Utility.AssignRandomHair(m, hue)`.
    e.register_fn(
        "mlq_random_hair",
        |ctx: NativeCallContext, m: Mobile, hue: i64| {
            with_world(&ctx, |w| {
                if live_m(w, m).is_some() {
                    crate::vendor::assign_random_hair(w, m.0, i32_of(hue));
                }
            })
        },
    );
    // `Utility.AssignRandomFacialHair(m, hue)`.
    e.register_fn(
        "mlq_random_facial_hair",
        |ctx: NativeCallContext, m: Mobile, hue: i64| {
            with_world(&ctx, |w| {
                if live_m(w, m).is_some() {
                    crate::vendor::assign_random_facial_hair(w, m.0, i32_of(hue));
                }
            })
        },
    );
    // `FacialHairItemID = item_id; FacialHairHue = hue`.
    e.register_fn(
        "mlq_facial_hair",
        |ctx: NativeCallContext, m: Mobile, item_id: i64, hue: i64| {
            with_world(&ctx, |w| {
                if live_m(w, m).is_some() {
                    props_b::set_facial_hair_item_id(w, m.0, i32_of(item_id));
                    props_b::set_facial_hair_hue(w, m.0, i32_of(hue));
                }
            })
        },
    );
    // `SetSkill(skill, min, max)`.
    e.register_fn(
        "mlq_skill_range",
        |ctx: NativeCallContext, m: Mobile, skill: &str, min: f64, max: f64| -> R<bool> {
            let Ok(skill) = crate::templates::mobile_steps::skill(skill) else {
                return Ok(false);
            };
            with_world(&ctx, |w| {
                live_m(w, m).is_some_and(|_| {
                    crate::creature::setters::set_skill_range(w, m.0, skill, min, max);
                    true
                })
            })
        },
    );
    e.register_fn(
        "mlq_equip",
        |ctx: NativeCallContext, m: Mobile, class: &str, hue: i64| equip(&ctx, m, class, hue),
    );
    e.register_fn(
        "mlq_equip",
        |ctx: NativeCallContext, m: Mobile, class: &str| equip(&ctx, m, class, -1),
    );
}
