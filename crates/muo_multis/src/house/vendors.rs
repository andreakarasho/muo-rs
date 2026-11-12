//! House vendor queries: `IsThereVendor`, `AvailableVendorsFor`, `AreThereAvailableVendorsFor`, and
//! `HasAddonContainers`.

use muo_content::factory::{item_is, mobile_is};
use muo_content::items::house_deps::deps;
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::{MapId, enumerate};

use super::d;

/// `IsThereVendor(location, map, out vendor, out rentalContract)`.
pub fn is_there_vendor(w: &World, location: Point3D, map: Option<MapId>) -> (bool, bool) {
    for m in enumerate::mobiles_at(w, map, location.x, location.y) {
        let z = w.mobile(m).map_or(0, |x| x.location.z);
        if (location.z - z).abs() <= 16
            && (mobile_is(w, m, "Server.Mobiles.PlayerVendor")
                || mobile_is(w, m, "Server.Mobiles.PlayerBarkeeper"))
        {
            return (true, false);
        }
    }
    for i in enumerate::items_at(w, map, location.x, location.y) {
        let z = w.item(i).map_or(0, |x| x.location.z);
        if (location.z - z).abs() <= 16 {
            if item_is(w, i, "Server.Mobiles.PlayerVendorPlaceholder") {
                return (true, false);
            }
            if item_is(w, i, "Server.Items.VendorRentalContract") {
                return (false, true);
            }
        }
    }
    (false, false)
}

fn can_interact(w: &mut World, vendor: MobileId, m: MobileId) -> bool {
    deps(w)
        .and_then(|x| x.vendor_can_interact)
        .is_some_and(|f| f(w, vendor, m))
}

/// `AvailableVendorsFor(m)`.
pub fn available_vendors_for(w: &mut World, house: ItemId, m: MobileId) -> Vec<MobileId> {
    d(w, house)
        .player_vendors
        .into_iter()
        .filter(|&v| can_interact(w, v, m))
        .collect()
}

/// `AreThereAvailableVendorsFor(m)`.
pub fn are_there_available_vendors_for(w: &mut World, house: ItemId, m: MobileId) -> bool {
    d(w, house)
        .player_vendors
        .into_iter()
        .any(|v| can_interact(w, v, m))
}

/// `HasAddonContainers`.
pub fn has_addon_containers(w: &World, house: ItemId) -> bool {
    d(w, house)
        .addons
        .into_iter()
        .any(|i| item_is(w, i, "Server.Items.BaseAddonContainer"))
}
