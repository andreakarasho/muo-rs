//! Script functions added for creatures, animals, mounts and NPCs (constructors moved to scripts): the
//! `BaseCreature` setters a constructor with arguments calls (`Golem(summoned, scalar)`, `BaseEnraged(summoner)`).
//! An unknown resistance or skill name does nothing.

use crate::creature::{control, init_ops, setters};
use crate::templates::mobile_steps::{resistance, skill};
use rhai::{Dynamic, Engine, NativeCallContext};

use super::api::{Mobile, R, i32_of, live_m, with_world};

/// Runs `f` on a live mobile of the script call.
fn on(ctx: &NativeCallContext, m: Mobile, f: impl FnOnce(&mut muo_server::world::World)) -> R<()> {
    with_world(ctx, |w| {
        if live_m(w, m).is_some() {
            f(w);
        }
    })
}

macro_rules! range_fn {
    ($e:ident, $($name:literal => $f:path,)*) => {$(
        $e.register_fn($name, |ctx: NativeCallContext, m: Mobile, a: i64, b: i64| {
            on(&ctx, m, |w| $f(w, m.0, i32_of(a), i32_of(b)))
        });
    )*};
}

pub(crate) fn register(e: &mut Engine) {
    range_fn! { e,
        "set_str_range" => setters::set_str_range,
        "set_dex_range" => setters::set_dex_range,
        "set_int_range" => setters::set_int_range,
        "set_hits_range" => setters::set_hits_range,
        "set_stam_range" => setters::set_stam_range,
        "set_damage" => setters::set_damage,
    }
    e.register_fn("set_hue", |ctx: NativeCallContext, m: Mobile, n: i64| {
        on(&ctx, m, |w| init_ops::hue(w, m.0, i32_of(n)))
    });
    e.register_fn("set_fame", |ctx: NativeCallContext, m: Mobile, n: i64| {
        on(&ctx, m, |w| init_ops::fame(w, m.0, i32_of(n)))
    });
    e.register_fn("set_karma", |ctx: NativeCallContext, m: Mobile, n: i64| {
        on(&ctx, m, |w| init_ops::karma(w, m.0, i32_of(n)))
    });
    // `SetDamageType(ResistanceType.<name>, n)`
    e.register_fn(
        "set_damage_type",
        |ctx: NativeCallContext, m: Mobile, r: &str, n: i64| {
            let Ok(r) = resistance(r) else { return Ok(()) };
            on(&ctx, m, |w| setters::set_damage_type(w, m.0, r, i32_of(n)))
        },
    );
    e.register_fn(
        "set_resistance",
        |ctx: NativeCallContext, m: Mobile, r: &str, n: i64| {
            let Ok(r) = resistance(r) else { return Ok(()) };
            on(&ctx, m, |w| setters::set_resistance(w, m.0, r, i32_of(n)))
        },
    );
    e.register_fn(
        "set_resistance_range",
        |ctx: NativeCallContext, m: Mobile, r: &str, a: i64, b: i64| {
            let Ok(r) = resistance(r) else { return Ok(()) };
            on(&ctx, m, |w| {
                setters::set_resistance_range(w, m.0, r, i32_of(a), i32_of(b));
            })
        },
    );
    e.register_fn(
        "set_skill_range",
        |ctx: NativeCallContext, m: Mobile, s: &str, a: f64, b: f64| {
            let Ok(s) = skill(s) else { return Ok(()) };
            on(&ctx, m, |w| setters::set_skill_range(w, m.0, s, a, b))
        },
    );
    // `PackItem(new T())` / `PackItem(new T(amount))`
    e.register_fn(
        "pack_new",
        |ctx: NativeCallContext, m: Mobile, class: &str| {
            on(&ctx, m, |w| init_ops::pack_new(w, m.0, class))
        },
    );
    e.register_fn(
        "pack_new_amount",
        |ctx: NativeCallContext, m: Mobile, class: &str, n: i64| {
            on(&ctx, m, |w| {
                init_ops::pack_new_amount(w, m.0, class, i32_of(n));
            })
        },
    );
    // `Master = master` (`()` clears it).
    e.register_fn(
        "set_master",
        |ctx: NativeCallContext, m: Mobile, master: Dynamic| {
            let master = master.try_cast::<Mobile>().map(|x| x.0);
            on(&ctx, m, |w| control::set_master(w, m.0, master))
        },
    );
}
