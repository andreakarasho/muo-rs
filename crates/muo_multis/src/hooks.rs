//! Sets the hooks the content crates left for houses (grep `BaseHouse` in `muo_content`): the common items, the
//! region, accounts, the player's context menu, spells, notoriety, and `BaseHouse`'s own Configure/Initialize.

use muo_content::accounting::AccountHooks;
use muo_content::factory::{find_item_by_type, item_is};
use muo_content::items::common::hooks_mut as common_hooks;
use muo_content::player_mobile::hooks::{HouseInfo, SystemHooks};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::item::multi;
use muo_server::map::MapId;
use muo_server::timer;

use crate::house::{self, access, decay, region_ops, storage};

/// `BaseHouse.Configure()`: `houseDecay.enable`, `LockedDownFlag = 1`, `SecureFlag = 2`.
pub fn configure(w: &mut World) {
    // A boat's plank is an `ILockable` (doors register theirs in muo_items).
    muo_content::items::common::add_lockable(
        w,
        muo_content::items::common::Lockable {
            key_value: crate::boats::parts::lockable_key_value,
            toggle: crate::boats::parts::lockable_toggle,
        },
    );
    let enabled = w
        .config
        .get_or_update_setting("houseDecay.enable", true)
        .unwrap_or(true);
    house::statics(w).decay_enabled = enabled;
    let s = muo_server::item::ops::statics(w);
    s.locked_down_flag = 1;
    s.secure_flag = 2;
    // The hooks are needed while the world loads (a house builds its region when it is read), which is before
    // `Initialize`.
    install(w);
}

/// `BaseHouse.Initialize()` and the hook installation.
pub fn initialize(w: &mut World) {
    timer::start(
        w,
        TimeSpan::from_mins(1),
        TimeSpan::from_mins(1),
        0,
        decay::decay_on_tick,
    );
    decay::initialize_treasure_regions(w);
}

/// `[OnEvent(nameof(PlayerMobile.PlayerDeletedEvent))] HandleDeletion(mob)`.
pub fn handle_deletion(w: &mut World, mob: MobileId) {
    let houses = access::get_houses(w, mob);
    if houses.is_empty() {
        return;
    }
    let mut trans = None;
    if let Some(a) = w.mobile(mob).and_then(|m| m.account) {
        let chars: Vec<Option<MobileId>> = muo_content::accounting::account(w, a)
            .map(|x| x.mobiles.clone())
            .unwrap_or_default();
        for c in chars.into_iter().flatten() {
            if c != mob && w.mobile(c).is_some_and(|x| !x.deleted) {
                trans = Some(c);
            }
        }
    }
    for h in houses {
        if trans.is_none() && house::d(w, h).co_owners.is_empty() {
            timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
                if let Some(ty) = w.item(h).map(|i| i.ty) {
                    (ty.delete)(w, h);
                }
            });
        } else {
            access::set_owner(w, h, trans);
        }
    }
}

fn delete_houses(w: &mut World, m: MobileId) {
    for h in access::get_houses(w, m) {
        if let Some(ty) = w.item(h).map(|i| i.ty) {
            (ty.delete)(w, h);
        }
    }
}

fn house_info(w: &World, m: MobileId, from: MobileId) -> Option<HouseInfo> {
    let h = access::find_house_at_mobile(w, m)?;
    Some(HouseInfo {
        internalized_vendors: house::d(w, h).internalized_vendors.len() as i32,
        is_owner: access::is_owner(w, h, from),
        is_aos_rules: house::is_aos_rules(w, h),
        is_friend: access::is_friend(w, h, from),
        ban_location: house::ban_location(w, h),
    })
}

/// `CheckHouseFlag(from, m, p, map)` of notoriety.
fn check_house_flag(
    w: &World,
    from: MobileId,
    m: Option<MobileId>,
    p: Point3D,
    map: Option<MapId>,
) -> bool {
    let Some(h) = access::find_house_at_point(w, p, map, 16) else {
        return false;
    };
    if house::d(w, h).public || !access::is_friend(w, h, from) {
        return false;
    }
    if let Some(m) = m {
        if access::is_friend(w, h, m) {
            return false;
        }
        if muo_content::creature::is_creature(w, m)
            && !w.mobile(m).is_some_and(|x| x.deleted)
            && muo_content::creature::cd(w, m).is_some_and(|d| d.controlled)
            && let Some(master) = muo_content::creature::control::control_master(w, m)
        {
            return !access::is_friend(w, h, master);
        }
    }
    true
}

fn eject_from_house(w: &mut World, from: MobileId, target: MobileId) {
    let Some(h) = access::find_house_at_mobile(w, target) else {
        return;
    };
    if !muo_server::mobile::base::alive(w, from)
        || house::deleted(w, h)
        || !access::is_friend(w, h, from)
    {
        return;
    }
    access::kick(w, h, from, target);
}

fn reclaim_vendor_gump(w: &mut World, m: MobileId) {
    if let Some(h) = access::find_house_at_mobile(w, m)
        && let Some(f) =
            muo_content::items::house_deps::deps(w).and_then(|x| x.display_vendor_inventory_gump)
    {
        f(w, m, h);
    }
}

fn install(w: &mut World) {
    region_ops::install_region_hooks(w);

    let c = common_hooks(w);
    c.house_check_accessible = Some(storage::check_accessible);
    c.house_check_hold = Some(
        |w, m, cont, item, message, _check_items, plus_items, _plus_weight| {
            storage::check_hold(w, m, cont, item, message, plus_items)
        },
    );
    c.house_drop_check = Some(|w, cont, from, dropped| {
        let Some(h) = access::find_house_at_item(w, cont) else {
            return true;
        };
        if !storage::has_locked_down_item(w, h, cont) {
            return true;
        }
        if item_is(w, dropped, "Server.Items.VendorRentalContract")
            || (muo_server::item::ops::is_container(w, dropped)
                && find_item_by_type(w, dropped, "Server.Items.VendorRentalContract").is_some())
        {
            access::msg(w, from, 1062492); // You cannot place a rental contract in a locked down container.
            return false;
        }
        storage::lock_down(w, h, from, dropped, false)
    });
    c.secure_level_entry = Some(crate::secure_entry::add_to);
    c.root_is_player_vendor = Some(|w, item| {
        matches!(muo_server::item::ops::root_parent(w, item), Some(muo_server::ids::EntityId::Mobile(m))
            if muo_content::factory::mobile_is(w, m, "Server.Mobiles.PlayerVendor"))
    });

    w.resources.get_mut::<AccountHooks>().delete_houses = delete_houses;

    let s = w.resources.get_mut::<SystemHooks>();
    s.house_at = Some(house_info);
    s.house_reclaim_vendor_gump = Some(reclaim_vendor_gump);

    w.resources
        .get_mut::<muo_content::hooks::WorldHooks>()
        .in_house_region = Some(|w, m| {
        let Some(mob) = w.mobile(m) else { return false };
        muo_server::region::get_region_of_type(
            w,
            mob.region,
            &muo_content::regions::house::HOUSE_REGION,
        )
        .is_some()
    });
    // `BaseHouse.FindHouseAt(m)`'s `(BanLocation, Map)` (BaseCreature.Delete moves the backpack contents there)
    w.resources
        .get_mut::<muo_content::hooks::WorldHooks>()
        .house_ban_location = Some(|w, m| {
        let h = access::find_house_at_mobile(w, m)?;
        Some((house::ban_location(w, h), house::map_of(w, h)?))
    });
    w.resources
        .get_mut::<muo_content::misc::notoriety::NotorietyHooks>()
        .check_house_flag = Some(check_house_flag);
    w.resources
        .get_mut::<muo_content::context_menus::ContextMenuHooks>()
        .eject_from_house = Some(eject_from_house);

    let sp = muo_content::spells::hooks::hooks_mut(w);
    sp.house_check = Some(|w, house, p, inside, range| {
        if inside {
            storage::is_inside_point(w, house, p, 16)
        } else {
            w.item(house)
                .is_some_and(|i| (i.ty.in_range_point3d)(w, house, p, range))
        }
    });
    sp.multi_contains = Some(multi::contains_point3d);
    // Nature's Fury: `Region.Find(p, map).GetRegion<HouseRegion>()?.House?.IsFriend(caster) == false`
    sp.house_blocks_nature_fury = Some(|w, caster, p, map| {
        let r = muo_server::region::find(w, p, Some(map));
        muo_server::region::get_region_of_type(w, r, &muo_content::regions::house::HOUSE_REGION)
            .and_then(|r| muo_content::regions::house::house_of(w, r))
            .is_some_and(|h| !access::is_friend(w, h, caster))
    });
    sp.house_locked_down_or_secured = Some(storage::check_locked_down_or_secured);
    sp.house_secured = Some(storage::check_secured);

    muo_content::items::house_deps::deps_mut(w)
        .securables
        .push(crate::teleporter::securable_level);
    // `Guild.IsEnemy` (HasAccess' guild rule)
    muo_content::items::house_deps::deps_mut(w).guild_is_enemy = Some(|w, a, b| {
        muo_content::misc::guild_class::relation_hook(w, a, b)
            == muo_content::misc::notoriety::GuildRelation::Enemy
    });

    // One table of fields shared with the other owners (food, weapons..): only the house fields are set here.
    let f = &mut w.resources.get_mut::<muo_content::skills::SkillHooks>().fns;
    f.find_house_at = Some(access::find_house_at_mobile);
    f.find_house_at_point = Some(access::find_house_at_point);
    f.house_is_friend = Some(access::is_friend);
    f.is_inside_mobile = Some(access::is_inside_mobile);

    // What the addons ask of a house.
    let a = muo_content::items::addon::hooks_mut(w);
    a.find_house_at_point = Some(|w, p, map, h| access::find_house_at_point(w, p, Some(map), h));
    a.find_house_at_item = Some(|w, item| access::find_house_at_item(w, item));
    a.house_is_owner = Some(|w, h, m| access::is_owner(w, h, m));
    a.item_house_co_owner = Some(|w, item, m| {
        access::find_house_at_item(w, item).is_some_and(|h| access::is_co_owner(w, h, m))
    });
    a.house_addons_contains = Some(|w, h, addon| house::d(w, h).addons.contains(&addon));
    a.house_addons_add = Some(|w, h, addon| {
        if let Some(d) = house::data_mut(w, h) {
            d.addons.push(addon);
        }
    });
    a.house_addons_remove = Some(|w, h, addon| {
        if let Some(d) = house::data_mut(w, h) {
            d.addons.retain(|&x| x != addon);
        }
    });
    a.house_doors = Some(|w, h| house::d(w, h).doors);
    a.door_open = Some(|w, d| {
        muo_content::items::house_deps::deps(w)
            .and_then(|x| x.door_open_offset)
            .is_some_and(|f| f(w, d).is_some())
    });
    a.house_add_secure = Some(storage::add_secure);

    crate::foundation::install(w);
    muo_content::items::house_deps::deps_mut(w)
        .securable_setters
        .push(crate::teleporter::set_securable_level);
    muo_content::player_mobile::subscribe_deleted(w, handle_deletion);
}
