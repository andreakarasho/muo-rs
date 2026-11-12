//! What the house region (`HouseRegion`, in `muo_content`) asks of a house, and the other hooks the
//! content crates left for `BaseHouse`.
//!
//! `foundation_customizer` is the foundation's design session (`foundation.rs`); the "recently customized" warning
//! hooks are the gumps crate's (`a_l/houses.rs`).

use muo_content::creature;
use muo_content::items::house_deps::deps;
use muo_content::regions::house::{
    CreatureHouseFlags, HouseOps, HouseRegionHooks, HouseTarget, SecureAccessResult,
};
use muo_primitives::{DateTime, Point3D, Rectangle2D};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;

use super::access;
use super::storage;
use super::{d, targets};

/// `bc` facts for the house region: None for a mobile that is not a `BaseCreature`.
pub fn creature_house_flags(w: &World, m: MobileId) -> Option<CreatureHouseFlags> {
    let ct = creature::ctype(w, m)?;
    let data = creature::cd(w, m)?;
    Some(CreatureHouseFlags {
        no_house_restrictions: (ct.no_house_restrictions)(w, m),
        is_house_summonable: (ct.is_house_summonable)(w, m),
        controlled: data.controlled,
    })
}

/// The operations table.
pub struct Ops;

impl HouseOps for Ops {
    fn location(&self, w: &World, house: ItemId) -> Point3D {
        super::location(w, house)
    }
    fn map(&self, w: &World, house: ItemId) -> Option<MapId> {
        super::map_of(w, house)
    }
    fn area(&self, w: &World, house: ItemId) -> Vec<Rectangle2D> {
        super::area(w, house)
    }
    fn relative_ban_location(&self, w: &World, house: ItemId) -> Point3D {
        d(w, house).relative_ban_location
    }
    fn ban_location(&self, w: &World, house: ItemId) -> Point3D {
        super::ban_location(w, house)
    }
    fn public(&self, w: &World, house: ItemId) -> bool {
        d(w, house).public
    }
    fn is_aos_rules(&self, w: &World, house: ItemId) -> bool {
        super::is_aos_rules(w, house)
    }
    fn is_active(&self, w: &World, house: ItemId) -> bool {
        super::is_active(w, house)
    }
    fn is_banned(&self, w: &World, house: ItemId, m: MobileId) -> bool {
        access::is_banned(w, house, m)
    }
    fn has_access(&self, w: &World, house: ItemId, m: MobileId) -> bool {
        access::has_access(w, house, m)
    }
    fn is_combat_restricted(&self, w: &World, house: ItemId, m: MobileId) -> bool {
        access::is_combat_restricted(w, house, m)
    }
    fn is_owner(&self, w: &World, house: ItemId, m: MobileId) -> bool {
        access::is_owner(w, house, m)
    }
    fn is_co_owner(&self, w: &World, house: ItemId, m: MobileId) -> bool {
        access::is_co_owner(w, house, m)
    }
    fn is_friend(&self, w: &World, house: ItemId, m: MobileId) -> bool {
        access::is_friend(w, house, m)
    }
    fn is_inside_mobile(&self, w: &World, house: ItemId, m: MobileId) -> bool {
        access::is_inside_mobile(w, house, m)
    }
    fn is_inside_item(&self, w: &World, house: ItemId, item: ItemId) -> bool {
        access::is_inside_item(w, house, item)
    }
    fn is_inside_point(&self, w: &World, house: ItemId, p: Point3D, height: i32) -> bool {
        storage::is_inside_point(w, house, p, height)
    }
    fn foundation_customizer(&self, w: &World, house: ItemId) -> Option<MobileId> {
        deps(w)
            .and_then(|x| x.foundation_customizer)
            .and_then(|f| f(w, house))
    }
    fn internalized_vendors(&self, w: &World, house: ItemId) -> usize {
        d(w, house).internalized_vendors.len()
    }
    fn check_accessibility(
        &self,
        w: &mut World,
        house: ItemId,
        item: ItemId,
        from: MobileId,
    ) -> bool {
        storage::check_accessibility(w, house, item, from)
    }
    fn has_locked_down_item(&self, w: &World, house: ItemId, item: ItemId) -> bool {
        storage::has_locked_down_item(w, house, item)
    }
    fn has_secure_item(&self, w: &World, house: ItemId, item: ItemId) -> bool {
        storage::has_secure_item(w, house, item)
    }
    fn check_secure_access(
        &self,
        w: &mut World,
        house: ItemId,
        from: MobileId,
        container: ItemId,
    ) -> SecureAccessResult {
        storage::check_secure_access(w, house, from, container)
    }
    fn sign(&self, w: &World, house: ItemId) -> Option<ItemId> {
        super::sign(w, house)
    }
    fn built_on(&self, w: &World, house: ItemId) -> DateTime {
        d(w, house).built_on
    }
    fn add_strong_box(&self, w: &mut World, house: ItemId, from: MobileId) {
        storage::add_strong_box(w, house, from);
    }
    fn add_trash_barrel(&self, w: &mut World, house: ItemId, from: MobileId) {
        super::doors::add_trash_barrel(w, house, from);
    }
    fn begin_target(&self, w: &mut World, house: ItemId, from: MobileId, target: HouseTarget) {
        match target {
            HouseTarget::Kick => targets::kick_target(w, from, house),
            HouseTarget::Ban => targets::ban_target(w, from, true, house),
            HouseTarget::Lockdown { release } => targets::lockdown_target(w, from, release, house),
            HouseTarget::Secure { unsecure } => targets::secure_target(w, from, unsecure, house),
        }
    }
    fn begin_resize(&self, w: &mut World, house: ItemId, from: MobileId) {
        if let Some(f) = deps(w).and_then(|x| x.confirm_resize_house_gump) {
            f(w, from, house);
        }
    }
    fn find_house_at(&self, w: &World, m: MobileId) -> Option<ItemId> {
        access::find_house_at_mobile(w, m)
    }
    fn find_house_at_point(
        &self,
        w: &World,
        loc: Point3D,
        map: Option<MapId>,
        height: i32,
    ) -> Option<ItemId> {
        access::find_house_at_point(w, loc, map, height)
    }
    fn show_recently_customized_warning(&self, w: &mut World, m: MobileId) {
        if let Some(f) = deps(w).and_then(|x| x.recently_customized_warning) {
            f(w, m);
        }
    }
    fn has_recently_customized_warning(&self, w: &World, m: MobileId) -> bool {
        deps(w)
            .and_then(|x| x.has_recently_customized_warning)
            .is_some_and(|f| f(w, m))
    }
    fn creature_flags(&self, w: &World, m: MobileId) -> Option<CreatureHouseFlags> {
        creature_house_flags(w, m)
    }
}

static OPS: Ops = Ops;

/// Installs `HouseRegionHooks` (the region's view of houses).
pub fn install_region_hooks(w: &mut World) {
    let h = w.resources.get_mut::<HouseRegionHooks>();
    h.ops = Some(&OPS);
    h.summoning = Some(|w| {
        w.resources
            .get::<creature::control::SummonState>()
            .is_some_and(|s| s.summoning)
    });
}
