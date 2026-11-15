//! Script functions added for weapons, armor, clothing, jewelry and the artifact folders (constructors moved to scripts).

use muo_server::world::World;
use rhai::{Engine, EvalAltResult, NativeCallContext};

use crate::api::{Ctx, Item};

/// `api::with_world` (private there): the world of the script call in progress.
fn with_world<T>(
    ctx: &NativeCallContext,
    f: impl FnOnce(&mut World) -> T,
) -> Result<T, Box<EvalAltResult>> {
    let ptr = ctx
        .tag()
        .and_then(|t| t.read_lock::<Ctx>())
        .map(|c| c.world);
    let Some(ptr) = ptr else {
        return Err("no world: only callable from a server hook".into());
    };
    // SAFETY: as `api::with_world`: the pointer is the only path to the world for the call's duration.
    Ok(f(unsafe { &mut *ptr }))
}

pub(crate) fn register(e: &mut Engine) {
    // `Slayer = SlayerName.<name>` of a weapon; an unknown name does nothing.
    e.register_fn(
        "set_slayer",
        |ctx: NativeCallContext, i: Item, name: &str| {
            use muo_content::items::slayer::SlayerName;
            let Some(s) = (0..64)
                .filter_map(SlayerName::from_i32)
                .find(|s| format!("{s:?}") == name)
            else {
                return Ok(());
            };
            with_world(&ctx, |w| {
                if let Some(d) = muo_content::items::weapon::data_mut(w, i.0)
                    && d.slayer != s as i32
                {
                    d.slayer = s as i32;
                    muo_server::item::ops::invalidate_properties(w, i.0);
                }
            })
        },
    );
    // `Slayer2 = (SlayerName)n` of a weapon (SlayerLongbow rolls it).
    e.register_fn("set_slayer2", |ctx: NativeCallContext, i: Item, n: i64| {
        with_world(&ctx, |w| {
            let n = n.clamp(0, i64::from(i32::MAX)) as i32;
            if let Some(d) = muo_content::items::weapon::data_mut(w, i.0)
                && d.slayer2 != n
            {
                d.slayer2 = n;
                muo_server::item::ops::invalidate_properties(w, i.0);
            }
        })
    });
}
