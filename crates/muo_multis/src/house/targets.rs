//! House targets (`LockdownTarget`, `SecureTarget`, `HouseKickTarget`, `HouseBanTarget`,
//! `HouseAccessTarget`, `CoOwnerTarget`, `HouseFriendTarget`, `HouseOwnerTarget`) as closures over the house.

use std::rc::Rc;

use muo_content::factory::item_is;
use muo_content::items::house_deps::deps;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::target::{Target, TargetFlags, Targeted, add_target, set_mobile_target};

use super::access::{self, msg};
use super::storage::{self, mm_local_overhead};
use super::{deleted, transfer};

type Handler = Rc<dyn Fn(&mut World, MobileId, Targeted)>;

/// `Mobile.Target = target` for a target with `CheckLOS = false`; `not_accessible` also routes
/// `OnTargetNotAccessible` to the same handler.
fn begin(w: &mut World, from: MobileId, range: i32, not_accessible: bool, handler: Handler) {
    let mut t = Target::new(w, range, false, TargetFlags::NONE);
    t.check_los = false;
    t.handlers.on_target = Some(handler.clone());
    if not_accessible {
        t.handlers.on_target_not_accessible = Some(handler);
    }
    let id = add_target(w, t);
    set_mobile_target(w, from, Some(id));
}

fn alive(w: &World, m: MobileId) -> bool {
    muo_server::mobile::base::alive(w, m)
}

/// The addon of an `AddonContainerComponent`: `Some(None)` is a component without one.
fn container_component_addon(w: &World, item: ItemId) -> Option<Option<ItemId>> {
    deps(w)
        .and_then(|x| x.container_component_addon)
        .and_then(|f| f(w, item))
}

/// `new LockdownTarget(release, house)`.
pub fn lockdown_target(w: &mut World, from: MobileId, release: bool, house: ItemId) {
    begin(
        w,
        from,
        12,
        true,
        Rc::new(move |w, from, targeted| {
            if !alive(w, from) || deleted(w, house) || !access::is_co_owner(w, house, from) {
                return;
            }
            match targeted {
                Targeted::Entity(EntityId::Item(item)) => {
                    if release {
                        match container_component_addon(w, item) {
                            Some(Some(addon)) => storage::release(w, house, from, addon),
                            Some(None) => {}
                            None => storage::release(w, house, from, item),
                        }
                    } else if item_is(w, item, "Server.Items.VendorRentalContract") {
                        // You must double click the contract in your pack to lock it down.
                        mm_local_overhead(w, from, 0x3B2, 1062392);
                        mm_local_overhead(w, from, 0x3B2, 501732); // I cannot lock this down!
                    } else if item_is(w, item, "Server.Items.AddonComponent") {
                        mm_local_overhead(w, from, 0x3E9, 501727); // You cannot lock that down!
                        mm_local_overhead(w, from, 0x3E9, 501732); // I cannot lock this down!
                    } else {
                        match container_component_addon(w, item) {
                            Some(Some(addon)) => {
                                storage::lock_down(w, house, from, addon, true);
                            }
                            Some(None) => {}
                            None => {
                                storage::lock_down(w, house, from, item, true);
                            }
                        }
                    }
                }
                Targeted::Static(_) => {}
                _ => msg(w, from, 1005377), // You cannot lock that down
            }
        }),
    );
}

/// `new SecureTarget(release, house)`.
pub fn secure_target(w: &mut World, from: MobileId, release: bool, house: ItemId) {
    begin(
        w,
        from,
        12,
        true,
        Rc::new(move |w, from, targeted| {
            if !alive(w, from) || deleted(w, house) || !access::is_co_owner(w, house, from) {
                return;
            }
            match targeted {
                Targeted::Entity(EntityId::Item(item)) => {
                    if release {
                        match container_component_addon(w, item) {
                            Some(Some(addon)) => storage::release_secure(w, house, from, addon),
                            Some(None) => {}
                            None => storage::release_secure(w, house, from, item),
                        }
                    } else if item_is(w, item, "Server.Items.VendorRentalContract") {
                        mm_local_overhead(w, from, 0x3B2, 1062392);
                        mm_local_overhead(w, from, 0x3B2, 501732); // I cannot lock this down!
                    } else {
                        match container_component_addon(w, item) {
                            Some(Some(addon)) => storage::add_secure(w, house, from, addon),
                            Some(None) => {}
                            None => storage::add_secure(w, house, from, item),
                        }
                    }
                }
                _ => msg(w, from, 1010424), // You cannot secure this
            }
        }),
    );
}

/// `new HouseKickTarget(house)`.
pub fn kick_target(w: &mut World, from: MobileId, house: ItemId) {
    begin(
        w,
        from,
        -1,
        false,
        Rc::new(move |w, from, targeted| {
            if !alive(w, from) || deleted(w, house) || !access::is_friend(w, house, from) {
                return;
            }
            match targeted {
                Targeted::Entity(EntityId::Mobile(m)) => access::kick(w, house, from, m),
                _ => msg(w, from, 501347), // You cannot eject that from the house!
            }
        }),
    );
}

/// `new HouseBanTarget(ban, house)`.
pub fn ban_target(w: &mut World, from: MobileId, ban: bool, house: ItemId) {
    begin(
        w,
        from,
        -1,
        false,
        Rc::new(move |w, from, targeted| {
            if !alive(w, from) || deleted(w, house) || !access::is_friend(w, house, from) {
                return;
            }
            match targeted {
                Targeted::Entity(EntityId::Mobile(m)) => {
                    if ban {
                        access::ban(w, house, from, m);
                    } else {
                        access::remove_ban(w, house, from, m);
                    }
                }
                _ => msg(w, from, 501347), // You cannot eject that from the house!
            }
        }),
    );
}

/// `new HouseAccessTarget(house)`.
pub fn access_target(w: &mut World, from: MobileId, house: ItemId) {
    begin(
        w,
        from,
        -1,
        false,
        Rc::new(move |w, from, targeted| {
            if !alive(w, from) || deleted(w, house) || !access::is_friend(w, house, from) {
                return;
            }
            match targeted {
                Targeted::Entity(EntityId::Mobile(m)) => access::grant_access(w, house, from, m),
                _ => msg(w, from, 1060712), // That is not a player.
            }
        }),
    );
}

/// `new CoOwnerTarget(add, house)`.
pub fn co_owner_target(w: &mut World, from: MobileId, add: bool, house: ItemId) {
    begin(
        w,
        from,
        12,
        false,
        Rc::new(move |w, from, targeted| {
            if !alive(w, from) || deleted(w, house) || !access::is_owner(w, house, from) {
                return;
            }
            match targeted {
                Targeted::Entity(EntityId::Mobile(m)) => {
                    if add {
                        access::add_co_owner(w, house, from, m);
                    } else {
                        access::remove_co_owner(w, house, from, m);
                    }
                }
                _ => msg(w, from, 501362), // That can't be a coowner
            }
        }),
    );
}

/// `new HouseFriendTarget(add, house)`.
pub fn friend_target(w: &mut World, from: MobileId, add: bool, house: ItemId) {
    begin(
        w,
        from,
        12,
        false,
        Rc::new(move |w, from, targeted| {
            if !alive(w, from) || deleted(w, house) || !access::is_co_owner(w, house, from) {
                return;
            }
            match targeted {
                Targeted::Entity(EntityId::Mobile(m)) => {
                    if add {
                        access::add_friend(w, house, from, m);
                    } else {
                        access::remove_friend(w, house, from, m);
                    }
                }
                _ => msg(w, from, 501371), // That can't be a friend
            }
        }),
    );
}

/// `new HouseOwnerTarget(house)`.
pub fn owner_target(w: &mut World, from: MobileId, house: ItemId) {
    begin(
        w,
        from,
        12,
        false,
        Rc::new(move |w, from, targeted| match targeted {
            Targeted::Entity(EntityId::Mobile(m)) if w.mobile(m).is_some_and(|x| x.player) => {
                transfer::begin_confirm_transfer(w, house, from, m);
            }
            _ => msg(w, from, 501384), // Only a player can own a house!
        }),
    );
}
