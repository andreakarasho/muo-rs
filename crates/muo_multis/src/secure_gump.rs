//! `m.SendGump(new SetSecureLevelGump(info, house))` through the gump crate's hook.

use muo_content::items::house_deps::deps;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};

pub fn send(w: &mut World, m: MobileId, house: ItemId, item: ItemId) {
    if let Some(f) = deps(w).and_then(|x| x.set_secure_level_gump) {
        f(w, m, house, item);
    }
}
