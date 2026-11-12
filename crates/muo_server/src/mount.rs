//! `IMount` and `IHasSteps`: the interfaces content mounts implement.

use muo_primitives::TimeSpan;

use crate::ids::{ItemId, MobileId};
use crate::world::World;

/// `IHasSteps`.
pub trait HasSteps {
    fn steps_max(&self, w: &World, id: MobileId) -> i32;
    fn steps_gained_per_idle_time(&self, w: &World, id: MobileId) -> i32;
    fn idle_time_per_steps_gain(&self, w: &World, id: MobileId) -> TimeSpan;
}

/// `IMount`.
pub trait Mount: HasSteps {
    fn rider(&self, w: &World, id: MobileId) -> Option<MobileId>;
    fn set_rider(&self, w: &mut World, id: MobileId, rider: Option<MobileId>);
    fn on_rider_damaged(
        &self,
        w: &mut World,
        id: MobileId,
        amount: i32,
        from: Option<MobileId>,
        will_kill: bool,
    );
}

/// `IMountItem`: the mount an item stands for.
pub trait MountItem {
    fn mount(&self, w: &World, item: ItemId) -> Option<MobileId>;
}
