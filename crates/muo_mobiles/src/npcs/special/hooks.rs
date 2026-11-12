//! What the special NPCs and guards call in systems above this crate (muo_engines depends on muo_mobiles): the engines
//! crate sets these in `special_wiring.rs`. An unset hook behaves as if the system were absent.

use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;

type AwardValorFn = fn(&mut World, MobileId, i32) -> Option<bool>;
type HarrowerGateFn =
    fn(&mut World, MobileId, Point3D, Option<MapId>, Point3D, MapId) -> Option<ItemId>;
#[derive(Default)]
pub struct SpecialHooks {
    /// `PlayerMurderSystem.BountiesEnabled`. Unset: false.
    pub bounties_enabled: Option<fn(&World) -> bool>,
    /// `PlayerMurderSystem.GetBounty(player)`.
    pub get_bounty: Option<fn(&World, MobileId) -> i32>,
    /// `PlayerMurderSystem.ClearBounty(player)`.
    pub clear_bounty: Option<fn(&mut World, MobileId)>,
    /// `VirtueSystem.Award(pm, VirtueName.Valor, amount, ref gainedPath)`: `Some(gained_path)` when it was awarded.
    pub award_valor: Option<AwardValorFn>,
    /// `JusticeVirtue.GetProtector(pm)`.
    pub justice_protector: Option<fn(&World, MobileId) -> Option<MobileId>>,
    /// `JusticeVirtue.CheckMapRegion(first, second)`.
    pub justice_check_map_region: Option<fn(&World, MobileId, MobileId) -> bool>,
    /// `VirtueSystem.GetLevel(m, VirtueName.Justice)` as 0 None, 1 Seeker, 2 Follower, 3 Knight.
    pub justice_level: Option<fn(&World, MobileId) -> i32>,
    /// `new ChampionSkull(type)`, the type being `ChampionSkullType`'s number.
    pub skull_new: Option<fn(&mut World, i32) -> ItemId>,
    /// `ChampionTitleSystem.AwardHarrowerTitle(pm)`.
    pub award_harrower_title: Option<fn(&mut World, MobileId)>,
    /// `new HarrowerGate(harrower, platLoc, platMap, entrance, Map.Felucca)`.
    pub harrower_gate: Option<HarrowerGateFn>,
}

pub fn get(w: &World) -> Option<&SpecialHooks> {
    w.resources.get::<SpecialHooks>()
}
