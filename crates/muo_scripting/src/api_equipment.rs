//! Script functions added for weapons, armor, clothing, jewelry and the artifact folders (constructors moved to scripts).

use rhai::{Engine, NativeCallContext};

use crate::api::{Item, i32_of, with_world};

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
            let n = i32_of(n).max(0);
            if let Some(d) = muo_content::items::weapon::data_mut(w, i.0)
                && d.slayer2 != n
            {
                d.slayer2 = n;
                muo_server::item::ops::invalidate_properties(w, i.0);
            }
        })
    });
}
