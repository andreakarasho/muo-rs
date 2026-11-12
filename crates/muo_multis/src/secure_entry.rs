//! `SetSecureLevelEntry` of `BaseHouse`: the "Set Security" context-menu entry.

use std::rc::Rc;

use muo_content::items::house_deps::deps;
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};

use crate::house::{self, access, storage};

/// `GetSecurable(from, item)` != null: the house the item's security can be set in.
pub fn securable_house(w: &World, from: MobileId, item: ItemId) -> Option<ItemId> {
    let house = access::find_house_at_item(w, item)?;
    if !access::is_owner(w, house, from) || !house::is_aos_rules(w, house) {
        return None;
    }
    let securable = muo_content::items::house_deps::securable_level(w, item).is_some();
    if securable {
        let is_door = muo_content::factory::item_is(w, item, "Server.Items.BaseDoor");
        let mut owned = is_door && house::d(w, house).doors.contains(&item);
        if !owned {
            owned = deps(w)
                .and_then(|x| x.foundation_is_fixture)
                .is_some_and(|f| f(w, house, item));
        }
        if !owned {
            owned = storage::has_locked_down_item(w, house, item);
        }
        owned.then_some(house)
    } else {
        house::d(w, house)
            .secures
            .iter()
            .any(|s| s.item == item)
            .then_some(house)
    }
}

/// `SetSecureLevelEntry.AddTo(from, item, ref list)`: the entry, or None.
pub fn add_to(w: &World, from: MobileId, item: ItemId) -> Option<ContextMenuEntry> {
    securable_house(w, from, item)?;
    let mut e = ContextMenuEntry::new(6203, 6);
    e.on_click = Some(Rc::new(
        |w: &mut World, from: MobileId, target: EntityId| {
            let EntityId::Item(item) = target else { return };
            if let Some(house) = securable_house(w, from, item)
                && let Some(f) = deps(w).and_then(|x| x.set_secure_level_gump)
            {
                f(w, from, house, item);
            }
        },
    ));
    Some(e)
}
