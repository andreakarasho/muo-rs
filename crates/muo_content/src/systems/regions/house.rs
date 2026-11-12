//! The house region: the region of a player house, which keeps out the banned and the uninvited and
//! answers the house speech commands.
//!
//! `BaseHouse` and `HouseFoundation` (Multis) live in the multis crate: the region reaches the house through
//! [`HouseOps`], a table of fn pointers the multis crate sets in [`HouseRegionHooks`]. Without it a house region
//! lets everyone in.
//!
//! `OnLogin(pm)` (`PlayerMobile.PlayerLoginEvent`) is [`on_login`], which `systems::register` subscribes.

use muo_primitives::dotnet::IgnoreCase;
use muo_primitives::{DateTime, Direction, Point3D, Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::events::SpeechEventArgs;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::messages as im;
use muo_server::map::MapId;
use muo_server::mobile::part2::messages as mm;
use muo_server::region::{
    self, DEFAULT_PRIORITY, REGION_BASE, RegionId, RegionType, convert_to_3d,
};
use muo_server::target::Targeted;

use super::base_region::BASE_REGION;
use crate::core;

/// `HousePriority`.
pub const HOUSE_PRIORITY: i32 = DEFAULT_PRIORITY + 1;

/// `CombatHeatDelay`.
pub const COMBAT_HEAT_DELAY: TimeSpan = TimeSpan::from_secs(30);

/// `House.CheckSecureAccess` result.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SecureAccessResult {
    Insecure,
    Accessible,
    Inaccessible,
}

/// The house target the speech commands begin.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum HouseTarget {
    /// `HouseKickTarget(house)`.
    Kick,
    /// `HouseBanTarget(true, house)`.
    Ban,
    /// `LockdownTarget(release, house)`.
    Lockdown { release: bool },
    /// `SecureTarget(unsecure, house)`.
    Secure { unsecure: bool },
}

/// What the region needs of `BaseHouse`.
pub trait HouseOps: Sync {
    fn location(&self, w: &World, house: ItemId) -> Point3D;
    fn map(&self, w: &World, house: ItemId) -> Option<MapId>;
    /// `House.Area`: rectangles relative to the house's location.
    fn area(&self, w: &World, house: ItemId) -> Vec<Rectangle2D>;
    fn relative_ban_location(&self, w: &World, house: ItemId) -> Point3D;
    fn ban_location(&self, w: &World, house: ItemId) -> Point3D;
    fn public(&self, w: &World, house: ItemId) -> bool;
    fn is_aos_rules(&self, w: &World, house: ItemId) -> bool;
    fn is_active(&self, w: &World, house: ItemId) -> bool;
    fn is_banned(&self, w: &World, house: ItemId, m: MobileId) -> bool;
    fn has_access(&self, w: &World, house: ItemId, m: MobileId) -> bool;
    fn is_combat_restricted(&self, w: &World, house: ItemId, m: MobileId) -> bool;
    fn is_owner(&self, w: &World, house: ItemId, m: MobileId) -> bool;
    fn is_co_owner(&self, w: &World, house: ItemId, m: MobileId) -> bool;
    fn is_friend(&self, w: &World, house: ItemId, m: MobileId) -> bool;
    /// `House.IsInside(m)`.
    fn is_inside_mobile(&self, w: &World, house: ItemId, m: MobileId) -> bool;
    /// `House.IsInside(item)`.
    fn is_inside_item(&self, w: &World, house: ItemId, item: ItemId) -> bool;
    /// `House.IsInside(p, height)`.
    fn is_inside_point(&self, w: &World, house: ItemId, p: Point3D, height: i32) -> bool;
    /// `House is HouseFoundation foundation && foundation.Customizer`.
    fn foundation_customizer(&self, w: &World, house: ItemId) -> Option<MobileId>;
    /// `House.InternalizedVendors.Count`.
    fn internalized_vendors(&self, w: &World, house: ItemId) -> usize;
    fn check_accessibility(
        &self,
        w: &mut World,
        house: ItemId,
        item: ItemId,
        from: MobileId,
    ) -> bool;
    fn has_locked_down_item(&self, w: &World, house: ItemId, item: ItemId) -> bool;
    fn has_secure_item(&self, w: &World, house: ItemId, item: ItemId) -> bool;
    fn check_secure_access(
        &self,
        w: &mut World,
        house: ItemId,
        from: MobileId,
        container: ItemId,
    ) -> SecureAccessResult;
    fn sign(&self, w: &World, house: ItemId) -> Option<ItemId>;
    fn built_on(&self, w: &World, house: ItemId) -> DateTime;
    fn add_strong_box(&self, w: &mut World, house: ItemId, from: MobileId);
    fn add_trash_barrel(&self, w: &mut World, house: ItemId, from: MobileId);
    /// `from.Target = new XxxTarget(house)`.
    fn begin_target(&self, w: &mut World, house: ItemId, from: MobileId, target: HouseTarget);
    /// `gumps.Close<HouseGumpAOS>(); gumps.Send(new ConfirmResizeHouseGump(House))`.
    fn begin_resize(&self, w: &mut World, house: ItemId, from: MobileId);
    /// `BaseHouse.FindHouseAt(m)`.
    fn find_house_at(&self, w: &World, m: MobileId) -> Option<ItemId>;
    /// `BaseHouse.FindHouseAt(loc, map, height)`. Default: none.
    fn find_house_at_point(
        &self,
        _w: &World,
        _loc: Point3D,
        _map: Option<MapId>,
        _height: i32,
    ) -> Option<ItemId> {
        None
    }
    /// `m.HasGump<RecentlyCustomizedWarningGump>()` then `m.SendGump(..)`: shows it unless already open.
    fn show_recently_customized_warning(&self, w: &mut World, m: MobileId);
    fn has_recently_customized_warning(&self, w: &World, m: MobileId) -> bool;
    /// `bc?.NoHouseRestrictions`, `bc?.IsHouseSummonable`, `BaseCreature.Summoning`, `bc?.Controlled`: what the
    /// creature rules of the region ask. Unknown for a non-creature.
    fn creature_flags(&self, w: &World, m: MobileId) -> Option<CreatureHouseFlags>;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct CreatureHouseFlags {
    pub no_house_restrictions: bool,
    pub is_house_summonable: bool,
    pub controlled: bool,
}

/// The `fn` table the multis crate sets.
#[derive(Default)]
pub struct HouseRegionHooks {
    pub ops: Option<&'static dyn HouseOps>,
    /// `BaseCreature.Summoning`.
    pub summoning: Option<fn(&World) -> bool>,
}

/// The state of a house region.
#[derive(Copy, Clone, Debug)]
pub struct HouseRegionData {
    pub house: ItemId,
    /// `m_Recursion`.
    pub recursion: bool,
}

fn ops(w: &World) -> Option<&'static dyn HouseOps> {
    w.resources.get::<HouseRegionHooks>()?.ops
}

/// The installed house operations (set by the multis crate).
pub fn house_ops(w: &World) -> Option<&'static dyn HouseOps> {
    ops(w)
}

fn state(w: &World, id: RegionId) -> Option<HouseRegionData> {
    w.region(id)?.ext.get::<HouseRegionData>().copied()
}

/// `House` of the region.
pub fn house_of(w: &World, id: RegionId) -> Option<ItemId> {
    state(w, id).map(|s| s.house)
}

fn send(w: &mut World, to: MobileId, number: i32) {
    mm::send_localized_message(w, to, number, "", 0x3B2);
}

/// `new HouseRegion(house)`: not registered yet (the house registers it).
pub fn new_house_region(w: &mut World, house: ItemId) -> Option<RegionId> {
    let o = ops(w)?;
    let map = o.map(w, house)?;
    let loc = o.location(w, house);

    // GetArea(house)
    let area: Vec<_> = o
        .area(w, house)
        .iter()
        .map(|r| {
            convert_to_3d(Rectangle2D::new(
                loc.x + r.start.x,
                loc.y + r.start.y,
                r.width(),
                r.height(),
            ))
        })
        .collect();

    let id = region::new_region(
        w,
        &HOUSE_REGION,
        None,
        map,
        None,
        Some(HOUSE_PRIORITY),
        area,
    );
    let ban = o.relative_ban_location(w, house);
    if let Some(r) = w.region_mut(id) {
        r.ext = muo_server::Ext::new(HouseRegionData {
            house,
            recursion: false,
        });
        r.go_location = Point3D::new(loc.x + ban.x, loc.y + ban.y, loc.z + ban.z);
    }
    Some(id)
}

/// `OnLogin(pm)`.
pub fn on_login(w: &mut World, pm: MobileId) {
    let Some(o) = ops(w) else { return };
    if let Some(house) = o.find_house_at(w, pm)
        && !o.public(w, house)
        && !o.is_friend(w, house, pm)
    {
        let to = o.ban_location(w, house);
        muo_server::mobile::base::set_location(w, pm, to, false);
    }
}

fn send_inaccessible_message(w: &mut World, _: RegionId, item: ItemId, from: MobileId) -> bool {
    let is_container = item_is(w, item, "Server.Items.Container");
    if is_container {
        im::send_localized_message_to(w, item, from, 501647, ""); // That is secure.
    } else {
        im::send_localized_message_to(w, item, from, 1061637, ""); // You are not allowed to access this.
    }
    true
}

fn check_accessibility(w: &mut World, id: RegionId, item: ItemId, from: MobileId) -> bool {
    match (ops(w), house_of(w, id)) {
        (Some(o), Some(h)) => o.check_accessibility(w, h, item, from),
        _ => true,
    }
}

fn set_recursion(w: &mut World, id: RegionId, v: bool) {
    if let Some(d) = w
        .region_mut(id)
        .and_then(|r| r.ext.get_mut::<HouseRegionData>())
    {
        d.recursion = v;
    }
}

fn banned_enter_message(w: &mut World, m: MobileId) {
    if !core::se(w) {
        send(w, m, 501284); // You may not enter.
    }
}

// Use OnLocationChanged instead of OnEnter because it can be that we enter a house region even though we're not
// actually inside the house
#[allow(clippy::if_same_then_else)] // the ban repeats for each condition
fn on_location_changed(w: &mut World, id: RegionId, m: MobileId, old_location: Point3D) {
    let Some(st) = state(w, id) else { return };
    if st.recursion {
        return;
    }

    (REGION_BASE.on_location_changed)(w, id, m, old_location);

    let (Some(o), house) = (ops(w), st.house) else {
        return;
    };
    set_recursion(w, id, true);

    let flags = o.creature_flags(w, m);
    let summoning = w
        .resources
        .get::<HouseRegionHooks>()
        .and_then(|h| h.summoning)
        .is_some_and(|f| f(w));
    let no_restrictions = flags.is_some_and(|f| f.no_house_restrictions);
    let summonable = flags.is_some_and(|f| f.is_house_summonable);

    if !no_restrictions
        && (!summonable || summoning || o.is_inside_point(w, house, old_location, 16))
    {
        let public = o.public(w, house);
        let aos_rules = o.is_aos_rules(w, house);
        let inside = o.is_inside_mobile(w, house, m);
        let customizer = o.foundation_customizer(w, house);

        if (public || !aos_rules) && o.is_banned(w, house, m) && inside {
            let to = o.ban_location(w, house);
            muo_server::mobile::base::set_location(w, m, to, false);
            banned_enter_message(w, m);
        } else if aos_rules && !public && !o.has_access(w, house, m) && inside {
            let to = o.ban_location(w, house);
            muo_server::mobile::base::set_location(w, m, to, false);
            banned_enter_message(w, m);
        } else if o.is_combat_restricted(w, house, m)
            && inside
            && !o.is_inside_point(w, house, old_location, 16)
        {
            let to = o.ban_location(w, house);
            muo_server::mobile::base::set_location(w, m, to, false);
            send(w, m, 1061637); // You are not allowed to access this.
        } else if customizer.is_some_and(|c| c != m) && inside {
            let to = o.ban_location(w, house);
            muo_server::mobile::base::set_location(w, m, to, false);
        }
    }

    if o.internalized_vendors(w, house) > 0
        && o.is_inside_mobile(w, house, m)
        && !o.is_inside_point(w, house, old_location, 16)
        && o.is_owner(w, house, m)
        && w.mobile(m).is_some_and(|x| (x.ty.alive)(w, m))
        && !o.has_recently_customized_warning(w, m)
    {
        o.show_recently_customized_warning(w, m);
    }

    set_recursion(w, id, false);
}

fn on_move_into(
    w: &mut World,
    id: RegionId,
    from: MobileId,
    d: Direction,
    new_location: Point3D,
    old_location: Point3D,
) -> bool {
    if !(REGION_BASE.on_move_into)(w, id, from, d, new_location, old_location) {
        return false;
    }
    let (Some(o), Some(house)) = (ops(w), house_of(w, id)) else {
        return true;
    };

    let flags = o.creature_flags(w, from);
    let summoning = w
        .resources
        .get::<HouseRegionHooks>()
        .and_then(|h| h.summoning)
        .is_some_and(|f| f(w));

    if !flags.is_some_and(|f| f.no_house_restrictions) {
        let summonable = flags.is_some_and(|f| f.is_house_summonable);
        if summonable && !(summoning || o.is_inside_point(w, house, old_location, 16)) {
            return false;
        }

        // Untamed creatures cannot enter private houses
        let aos_rules = o.is_aos_rules(w, house);
        let public = o.public(w, house);
        if aos_rules && !public && flags.is_some_and(|f| !f.controlled) {
            return false;
        }

        if (public || !aos_rules)
            && o.is_banned(w, house, from)
            && o.is_inside_point(w, house, new_location, 16)
        {
            let to = o.ban_location(w, house);
            muo_server::mobile::base::set_location(w, from, to, false);
            banned_enter_message(w, from);
            return false;
        }

        if aos_rules
            && !public
            && !o.has_access(w, house, from)
            && o.is_inside_point(w, house, new_location, 16)
        {
            banned_enter_message(w, from);
            return false;
        }

        if o.is_combat_restricted(w, house, from)
            && !o.is_inside_point(w, house, old_location, 16)
            && o.is_inside_point(w, house, new_location, 16)
        {
            send(w, from, 1061637); // You are not allowed to access this.
            return false;
        }

        if o.foundation_customizer(w, house).is_some_and(|c| c != from)
            && o.is_inside_point(w, house, new_location, 16)
        {
            return false;
        }
    }

    if o.internalized_vendors(w, house) > 0
        && o.is_inside_mobile(w, house, from)
        && !o.is_inside_point(w, house, old_location, 16)
        && o.is_owner(w, house, from)
        && w.mobile(from).is_some_and(|x| (x.ty.alive)(w, from))
        && !o.has_recently_customized_warning(w, from)
    {
        o.show_recently_customized_warning(w, from);
    }

    true
}

fn on_decay(w: &mut World, id: RegionId, item: ItemId) -> bool {
    let (Some(o), Some(house)) = (ops(w), house_of(w, id)) else {
        return (REGION_BASE.on_decay)(w, id, item);
    };
    ((!o.has_locked_down_item(w, house, item) && !o.has_secure_item(w, house, item))
        || !o.is_inside_item(w, house, item))
        && (REGION_BASE.on_decay)(w, id, item)
}

fn get_logout_delay(w: &World, id: RegionId, m: MobileId) -> TimeSpan {
    let base = || (BASE_REGION.get_logout_delay)(w, id, m);
    let (Some(o), Some(house)) = (ops(w), house_of(w, id)) else {
        return base();
    };
    if !o.is_friend(w, house, m) || !o.is_inside_mobile(w, house, m) {
        return base();
    }

    if let Some(mob) = w.mobile(m) {
        for info in &mob.aggressed {
            if w.mobile(info.defender).is_some_and(|d| d.player)
                && w.now - info.last_combat_time < COMBAT_HEAT_DELAY
            {
                return base();
            }
        }
    }

    TimeSpan::ZERO
}

fn on_speech(w: &mut World, id: RegionId, e: &mut SpeechEventArgs) {
    (BASE_REGION.on_speech)(w, id, e);

    let (Some(o), Some(house)) = (ops(w), house_of(w, id)) else {
        return;
    };
    let from = e.mobile;
    let sign = o.sign(w, house);

    let is_owner = o.is_owner(w, house, from);
    let is_co_owner = is_owner || o.is_co_owner(w, house, from);
    let is_friend = is_co_owner || o.is_friend(w, house, from);

    if !is_friend {
        return;
    }

    if !w.mobile(from).is_some_and(|m| (m.ty.alive)(w, from)) {
        return;
    }

    if core::ml(w) && e.speech.eq_ic("I wish to resize my house") {
        let far = match sign {
            Some(sign) => {
                let sign_map = w.item(sign).and_then(|i| i.map);
                let from_map = w.mobile(from).and_then(|m| m.map);
                let sign_loc = muo_server::item::ops::get_world_location(w, sign);
                from_map != sign_map
                    || !muo_server::mobile::base::in_range_point3d(w, from, sign_loc, 0)
            }
            None => true,
        };
        if far {
            send(w, from, 500295); // you are too far away to do that.
        } else if w.now <= o.built_on(w, house).add_hours(1.0) {
            send(w, from, 1080178); // You must wait one hour between each house demolition.
        } else if is_owner {
            o.begin_resize(w, house, from);
        } else {
            send(w, from, 501320); // Only the house owner may do this.
        }
    }

    if !o.is_inside_mobile(w, house, from) || !o.is_active(w, house) {
        return;
    }

    if e.has_keyword(0x33) {
        // remove thyself
        send(w, from, 501326); // Target the individual to eject from this house.
        o.begin_target(w, house, from, HouseTarget::Kick);
    } else if e.has_keyword(0x34) {
        // I ban thee
        if !o.public(w, house) && o.is_aos_rules(w, house) {
            send(w, from, 1062521); // You cannot ban someone from a private house.  Revoke their access instead.
        } else {
            send(w, from, 501325); // Target the individual to ban from this house.
            o.begin_target(w, house, from, HouseTarget::Ban);
        }
    } else if e.has_keyword(0x23) {
        // I wish to lock this down
        if is_co_owner {
            send(w, from, 502097); // Lock what down?
            o.begin_target(w, house, from, HouseTarget::Lockdown { release: false });
        } else {
            send(w, from, 1010587); // You are not a co-owner of this house.
        }
    } else if e.has_keyword(0x24) {
        // I wish to release this
        if is_co_owner {
            send(w, from, 502100); // Choose the item you wish to release
            o.begin_target(w, house, from, HouseTarget::Lockdown { release: true });
        } else {
            send(w, from, 1010587); // You are not a co-owner of this house.
        }
    } else if e.has_keyword(0x25) {
        // I wish to secure this
        if is_owner {
            send(w, from, 502103); // Choose the item you wish to secure
            o.begin_target(w, house, from, HouseTarget::Secure { unsecure: false });
        } else {
            send(w, from, 502094); // You must be in your house to do this.
        }
    } else if e.has_keyword(0x26) {
        // I wish to unsecure this
        if is_owner {
            send(w, from, 502106); // Choose the item you wish to unsecure
            o.begin_target(w, house, from, HouseTarget::Secure { unsecure: true });
        } else {
            send(w, from, 502094); // You must be in your house to do this.
        }
    } else if e.has_keyword(0x27) {
        // I wish to place a strongbox
        if is_owner {
            send(w, from, 502109); // Owners do not get a strongbox of their own.
        } else if is_co_owner {
            o.add_strong_box(w, house, from);
        } else {
            send(w, from, 1010587); // You are not a co-owner of this house.
        }
    } else if e.has_keyword(0x28) {
        // trash barrel
        if is_co_owner {
            o.add_trash_barrel(w, house, from);
        } else {
            send(w, from, 1010587); // You are not a co-owner of this house.
        }
    }
}

fn on_double_click(w: &mut World, id: RegionId, from: MobileId, o: &Targeted) -> bool {
    if let (Targeted::Entity(EntityId::Item(item)), Some(h), Some(ops)) =
        (o, house_of(w, id), ops(w))
    {
        let is_container = item_is(w, *item, "Server.Items.Container");
        if is_container {
            match ops.check_secure_access(w, h, from, *item) {
                SecureAccessResult::Accessible => return true,
                SecureAccessResult::Inaccessible => {
                    im::send_localized_message_to(w, *item, from, 1010563, "");
                    return false;
                }
                SecureAccessResult::Insecure => {}
            }
        }
    }
    (BASE_REGION.on_double_click)(w, id, from, o)
}

fn on_single_click(w: &mut World, id: RegionId, from: MobileId, o: &Targeted) -> bool {
    if let (Targeted::Entity(EntityId::Item(item)), Some(h), Some(ops)) =
        (o, house_of(w, id), ops(w))
    {
        if ops.has_locked_down_item(w, h, *item) {
            muo_server::item::ops::label_to_number(w, *item, from, 501643, ""); // [locked down]
        } else if ops.has_secure_item(w, h, *item) {
            muo_server::item::ops::label_to_number(w, *item, from, 501644, ""); // [locked down & secure]
        }
    }
    (BASE_REGION.on_single_click)(w, id, from, o)
}

/// `HouseRegion`.
pub static HOUSE_REGION: RegionType = RegionType {
    name: "HouseRegion",
    parent: Some(&BASE_REGION),
    allow_housing: |_, _, _, _| false,
    send_inaccessible_message,
    check_accessibility,
    on_location_changed,
    on_move_into,
    on_decay,
    get_logout_delay,
    on_speech,
    on_double_click,
    on_single_click,
    ..BASE_REGION
};

/// `item is T`: the item's class, or an ancestor, has this full name.
fn item_is(w: &World, id: ItemId, full_name: &str) -> bool {
    let mut t = w.item(id).map(|i| i.ty);
    while let Some(ty) = t {
        if ty.full_name == full_name {
            return true;
        }
        t = ty.parent;
    }
    false
}
