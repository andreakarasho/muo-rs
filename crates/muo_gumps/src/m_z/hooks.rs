//! Calls these gumps make into systems that live in other crates (spells, houses, vendors, treasures of Tokuno,
//! the property registry...), as `fn` pointers in `w.resources` set by the owning module's `register`. An unset
//! hook behaves as if the system were absent or inert (noted per field).

use std::rc::Rc;

use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::net_state::NetStateId;

use super::props::host::PropsHost;

type PolymorphCastFn = fn(&mut World, MobileId, Option<ItemId>, i32);
#[derive(Default)]
pub struct MzHooks {
    /// The property registry the props gumps ask (see `props::host`). Unset: they show no properties.
    pub props: Option<Rc<dyn PropsHost>>,

    /// `new PolymorphSpell(caster, scroll, body).Cast()`. Unset: nothing happens.
    pub polymorph_cast: Option<PolymorphCastFn>,
    /// `IDirectionAddonDeed`: `deed.East = east; deed.SendTarget(from)`. Unset: nothing happens.
    pub addon_deed_choose: Option<fn(&mut World, ItemId, bool, MobileId)>,

    /// `PlayerMobile.TithingPoints`. Unset: 0.
    pub tithing_points: Option<fn(&World, MobileId) -> i32>,
    /// `PlayerMobile.TithingPoints += amount`.
    pub add_tithing_points: Option<fn(&mut World, MobileId, i32)>,

    /// `TreasuresOfTokuno.DropEra` / `RewardEra` as the enum's number (0 is `None`). Unset: 0.
    pub tot_drop_era: Option<fn(&World) -> i32>,
    pub tot_reward_era: Option<fn(&World) -> i32>,
    pub tot_set_drop_era: Option<fn(&mut World, i32)>,
    pub tot_set_reward_era: Option<fn(&mut World, i32)>,

    /// `new ClientGump(from, targetState)` sent to `from`. Unset: nothing happens.
    pub open_client_gump: Option<fn(&mut World, MobileId, NetStateId)>,

    /// The owners of houses, runebooks, player vendors and rental contracts (see `ops`). Unset: those gumps do nothing.
    pub ops: super::ops::Ops,
}

pub fn hooks(w: &World) -> Option<&MzHooks> {
    w.resources.get::<MzHooks>()
}

pub fn hooks_mut(w: &mut World) -> &mut MzHooks {
    w.resources.get_mut::<MzHooks>()
}

/// The installed property registry.
pub fn props_host(w: &World) -> Option<Rc<dyn PropsHost>> {
    hooks(w).and_then(|h| h.props.clone())
}

/// The installed `ops` (cheap to clone: reference counts).
pub fn ops(w: &World) -> super::ops::Ops {
    hooks(w).map(|h| h.ops.clone()).unwrap_or_default()
}
