//! `InitializePropertyList`, `ClearProperties` and `InvalidateProperties`.

use crate::ids::MobileId;
use crate::map::MapId;
use crate::mobile::MobileDelta;
use crate::property_list::PropertyList;
use crate::world::World;

/// `InitializePropertyList(list)`: runs `GetProperties` into `list`, terminated; `IsBuilding` is set meanwhile.
pub fn initialize_property_list(w: &World, id: MobileId, list: &mut PropertyList) {
    list.is_building = true;
    if let Some(m) = w.mobile(id) {
        (m.ty.get_properties)(w, id, list);
    }
    list.terminate();
    list.is_building = false;
}

pub fn clear_properties(w: &mut World, id: MobileId) {
    if let Some(m) = w.mobile_mut(id) {
        m.property_list = None;
    }
}

/// Rebuilds the cached list and, when its hash changed, queues
/// `MobileDelta.Properties`. Called from inside the list's own build is refused (always a bug in a property
/// getter) with an error logged; the possibly stale tooltip is kept.
pub fn invalidate_properties(w: &mut World, id: MobileId) {
    if !w.core.opl_enabled {
        return;
    }
    let Some(m) = w.mobile(id) else { return };

    if m.property_list.as_ref().is_some_and(|l| l.is_building) {
        tracing::error!(
            "{:?} called InvalidateProperties() while its property list was being built. Remove the side effect from the property getter, or defer it with Timer.DelayCall.",
            id
        );
        return;
    }

    if m.map.is_some() && m.map != Some(MapId::INTERNAL) && !w.loading() {
        let ty = m.ty;
        // Publish the build in progress so a nested InvalidateProperties is refused above.
        if let Some(l) = w.mobile_mut(id).and_then(|m| m.property_list.as_mut()) {
            l.is_building = true;
        }
        let (old_hash, new_hash) = match w.mobile(id).and_then(|m| m.property_list.clone()) {
            Some(mut list) => {
                let old = list.hash();
                list.reset();
                initialize_property_list(w, id, &mut list);
                let new = list.hash();
                if let Some(m) = w.mobile_mut(id) {
                    m.property_list = Some(list);
                }
                (Some(old), new)
            }
            // `PropertyList.Hash` builds the list, then reads its hash.
            None => {
                super::super::part1::props_b::ensure_property_list(w, id);
                let new = w
                    .mobile(id)
                    .and_then(|m| m.property_list.as_ref())
                    .map_or(0, |l| l.hash());
                (None, new)
            }
        };
        if old_hash != Some(new_hash) {
            (ty.delta)(w, id, MobileDelta::PROPERTIES);
        }
    } else {
        clear_properties(w, id);
    }
}
