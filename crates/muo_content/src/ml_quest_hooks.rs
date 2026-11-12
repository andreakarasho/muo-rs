//! Calls `BaseCreature` makes into the ML quest system , set by `muo_engines`. Unset = the system
//! is not enabled (`MLQuestSystem.Enabled` defaults to `Core.ML`, setting `questSystem.enableMLQuests`; the
//! engine's `configure` sets these hooks whatever it reads).

use muo_primitives::Point3D;
use muo_server::ids::MobileId;
use muo_server::world::World;

#[derive(Default)]
pub struct MlQuestCreatureHooks {
    /// `MLQuestSystem.Enabled && CanGiveMLQuest`.
    pub can_give_quest: Option<fn(&World, MobileId) -> bool>,
    /// `MLQuestSystem.OnDoubleClick(this, pm)` when `from is PlayerMobile`.
    pub double_click: Option<fn(&mut World, MobileId, MobileId)>,
    /// `MLQuestSystem.Enabled && CanGiveMLQuest && from is PlayerMobile`: tells the player to mark the quest items;
    /// true when it did (`OnDragDrop` then returns false).
    pub drag_drop: Option<fn(&mut World, MobileId, MobileId) -> bool>,
    /// `MLQuestSystem.HandleDeletion(this)`.
    pub after_delete: Option<fn(&mut World, MobileId)>,
    /// `MLQuestSystem.Enabled && CanShout && m is PlayerMobile` then `CheckShout(pm, oldLocation)`.
    pub movement: Option<fn(&mut World, MobileId, MobileId, Point3D)>,
    /// `MLQuestSystem.HandleKill(pm, creature)`.
    pub handle_kill: Option<fn(&mut World, MobileId, MobileId)>,
}

pub fn ml_quest_creature_hooks(w: &World) -> Option<&MlQuestCreatureHooks> {
    w.resources.get::<MlQuestCreatureHooks>()
}
