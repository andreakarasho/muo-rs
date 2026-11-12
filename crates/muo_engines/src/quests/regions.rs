//! The quest regions: CancelQuestRegion, QuestCompleteObjectiveRegion, QuestNoEntryRegion and QuestOfferRegion.
//!
//! None of them is in `regions.json` (no JSON loader is registered for them), so nothing makes one by itself; code
//! that wants one makes the region and sets [`QuestRegionData`] with [`set_data`].

use muo_content::regions::BASE_REGION;
use muo_primitives::{Direction, Point3D};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::region::{RegionId, RegionType};

use super::core::{ObjClass, QClass, can_offer_quest, has_quest, load, quest_is};
use super::gumps;

/// The properties of the four classes: `Quest`, `Objective`, `MinObjective`, `MaxObjective`, `Message`.
#[derive(Clone, Default)]
pub struct QuestRegionData {
    pub quest: Option<&'static QClass>,
    pub objective: Option<&'static ObjClass>,
    pub min_objective: Option<&'static ObjClass>,
    pub max_objective: Option<&'static ObjClass>,
    pub message: i32,
}

/// Gives a region the state of one of the quest region classes.
pub fn set_data(w: &mut World, id: RegionId, data: QuestRegionData) {
    if let Some(r) = w.region_mut(id) {
        r.ext.insert(data);
    }
}

fn data(w: &World, id: RegionId) -> QuestRegionData {
    w.region(id)
        .and_then(|r| r.ext.get::<QuestRegionData>())
        .cloned()
        .unwrap_or_default()
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
}

fn is_staff(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.access_level > AccessLevel::Player)
}

/// `CancelQuestRegion`.
pub static CANCEL_QUEST_REGION: RegionType = RegionType {
    name: "CancelQuestRegion",
    parent: Some(&BASE_REGION),
    on_move_into: |w, id, m, d: Direction, new: Point3D, old: Point3D| {
        if !(BASE_REGION.on_move_into)(w, id, m, d, new, old) {
            return false;
        }
        if is_staff(w, m) {
            return true;
        }
        let Some(quest) = data(w, id).quest else {
            return true;
        };
        if is_player(w, m) && quest_is(w, m, quest) {
            let ns = muo_server::mobile::part1::props_b::net_state(w, m);
            if !ns.is_some_and(|ns| muo_content::gumps::has_gump::<gumps::CancelLogic>(w, ns))
                && let Some(q) = load(w, m)
            {
                q.begin_cancel_quest(w);
            }
            return false;
        }
        true
    },
    ..BASE_REGION
};

/// `QuestCompleteObjectiveRegion`.
pub static QUEST_COMPLETE_OBJECTIVE_REGION: RegionType = RegionType {
    name: "QuestCompleteObjectiveRegion",
    parent: Some(&BASE_REGION),
    on_enter: |w, id, m| {
        (BASE_REGION.on_enter)(w, id, m);
        let d = data(w, id);
        if let (Some(quest), Some(objective)) = (d.quest, d.objective)
            && is_player(w, m)
        {
            super::core::with_quest_of(w, m, quest, |w, q| {
                if let Some(i) = q
                    .find_objective(objective)
                    .filter(|&i| !q.objectives[i].completed(q))
                {
                    q.complete_objective(w, i);
                }
            });
        }
    },
    ..BASE_REGION
};

/// `QuestNoEntryRegion`.
pub static QUEST_NO_ENTRY_REGION: RegionType = RegionType {
    name: "QuestNoEntryRegion",
    parent: Some(&BASE_REGION),
    on_move_into: |w, id, m, d: Direction, new: Point3D, old: Point3D| {
        if !(BASE_REGION.on_move_into)(w, id, m, d, new, old) {
            return false;
        }
        if is_staff(w, m) {
            return true;
        }
        // m is BaseCreature { Controlled: false, Summoned: false }
        if muo_content::creature::is_creature(w, m) {
            let (controlled, summoned) =
                muo_content::creature::read(w, m, (true, true), |c| (c.controlled, c.summoned));
            if !controlled && !summoned {
                return true;
            }
        }
        let rd = data(w, id);
        let Some(quest) = rd.quest else { return true };
        if is_player(w, m)
            && let Some(q) = load(w, m).filter(|q| q.is(quest))
            && rd.min_objective.is_none_or(|o| q.has_objective(o))
            && rd.max_objective.is_none_or(|o| !q.has_objective(o))
        {
            return true;
        }
        if rd.message != 0 {
            muo_content::mx::msg(w, m, rd.message);
        }
        false
    },
    ..BASE_REGION
};

/// `QuestOfferRegion`.
pub static QUEST_OFFER_REGION: RegionType = RegionType {
    name: "QuestOfferRegion",
    parent: Some(&BASE_REGION),
    on_enter: |w, id, m| {
        (BASE_REGION.on_enter)(w, id, m);
        let Some(quest) = data(w, id).quest else {
            return;
        };
        if is_player(w, m) && !has_quest(w, m) && can_offer_quest(w, m, quest) {
            super::core::Quest::new(w, quest, m).send_offer(w);
        }
    },
    ..BASE_REGION
};

/// The region classes of this folder.
pub static REGIONS: &[&RegionType] = &[
    &CANCEL_QUEST_REGION,
    &QUEST_COMPLETE_OBJECTIVE_REGION,
    &QUEST_NO_ENTRY_REGION,
    &QUEST_OFFER_REGION,
];
