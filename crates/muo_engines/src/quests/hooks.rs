//! Calls the quests make into systems another crate owns (plants, spells, maps, the MLQuest system...),
//! as `fn` pointers in `w.resources`. An unset hook behaves as if the system did nothing.

use muo_primitives::{Point2D, Point3D};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};

type MakeSeedFn = fn(&mut World, i32, i32) -> Option<ItemId>;
type VirtueAwardFn = fn(&mut World, MobileId, i32, i32) -> bool;
type ApplyRunicAttributesFn = fn(&mut World, ItemId, i32, i32, i32);
#[derive(Default)]
pub struct QuestHooks {
    /// `SummonFamiliarSpell.Table.TryGetValue(from, out bc) && bc is HordeMinionFamiliar hmf && hmf.InRange(from, 5)
    /// && hmf.TargetLocation == null`, then `hmf.TargetLocation = target`; true when it did. Unset: false.
    pub horde_minion_fetch: Option<fn(&mut World, MobileId, Point2D) -> bool>,
    /// `new Seed(plantType, plantHue)` (Engines/Plants). Unset: no seed.
    pub make_seed: Option<MakeSeedFn>,
    /// `PMList.Trammel/Felucca/Malas.Entries` locations of a map (the public moongates). Unset: none.
    pub moongates: Option<fn(&World, muo_server::map::MapId) -> Vec<Point3D>>,
    /// `VirtueSystem.Award(player, virtue, points, ref gainedPath)`; `virtue` is the `VirtueName` value (Sacrifice is
    /// 1). True when the virtue was awarded. Unset: false.
    pub virtue_award: Option<VirtueAwardFn>,
    /// `m.SendGump(new ResurrectGump(m, ResurrectMessage.Healer))`. Unset: nothing.
    pub resurrect_gump_healer: Option<fn(&mut World, MobileId)>,
    /// `BaseRunicTool.ApplyAttributesTo(weapon or armor, attributeCount, min, max)`. Unset: nothing.
    pub apply_runic_attributes: Option<ApplyRunicAttributesFn>,
}

pub fn quest_hooks(w: &World) -> Option<&QuestHooks> {
    w.resources.get::<QuestHooks>()
}
