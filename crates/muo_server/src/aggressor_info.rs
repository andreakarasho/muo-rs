//! Who attacked whom and when: one entry per aggressor/aggressed pair.
//!
//! There is no object pool (`m_Pool`, `m_Queued`, `Free`, `DumpAccess`): infos are plain values in
//! `Mobile::aggressors`/`aggressed`, and `Free` has nothing to do.

use muo_primitives::{DateTime, TimeSpan};

use crate::ids::MobileId;
use crate::world::World;

#[derive(Clone, Debug)]
pub struct AggressorInfo {
    pub attacker: MobileId,
    pub defender: MobileId,
    pub can_report_murder: bool,
    pub criminal_aggression: bool,
    pub last_combat_time: DateTime,
    pub reported: bool,
}

/// `AggressorInfo.ExpireDelay`, a settable static (default 2 minutes); kept in `World::resources`.
pub struct AggressorExpireDelay(pub TimeSpan);

impl Default for AggressorExpireDelay {
    fn default() -> Self {
        AggressorExpireDelay(TimeSpan::from_mins(2))
    }
}

impl AggressorInfo {
    /// `AggressorInfo.Create(attacker, defender, criminal)`.
    pub fn create(
        w: &World,
        attacker: MobileId,
        defender: MobileId,
        criminal: bool,
    ) -> AggressorInfo {
        AggressorInfo {
            attacker,
            defender,
            can_report_murder: criminal,
            criminal_aggression: criminal,
            last_combat_time: w.now,
            reported: false,
        }
    }

    /// `Refresh()`.
    pub fn refresh(&mut self, now: DateTime) {
        self.last_combat_time = now;
        self.reported = false;
    }

    /// `AggressorInfo.ExpireDelay`.
    pub fn expire_delay(w: &World) -> TimeSpan {
        w.resources
            .get::<AggressorExpireDelay>()
            .map_or(AggressorExpireDelay::default().0, |d| d.0)
    }

    /// `Expired`: either side is gone or deleted, or the last combat is older than `ExpireDelay`.
    pub fn expired(&self, w: &World) -> bool {
        let gone = |id| w.mobile(id).is_none_or(|m| m.deleted);
        gone(self.attacker)
            || gone(self.defender)
            || w.now >= self.last_combat_time + Self::expire_delay(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::MOBILE_BASE;

    #[test]
    fn expires_after_delay() {
        let mut w = World::new();
        let a = w.new_mobile(&MOBILE_BASE);
        let d = w.new_mobile(&MOBILE_BASE);
        let mut info = AggressorInfo::create(&w, a, d, true);
        assert!(info.can_report_murder && info.criminal_aggression && !info.expired(&w));
        w.now += TimeSpan::from_mins(2);
        assert!(info.expired(&w));
        info.refresh(w.now);
        assert!(!info.expired(&w));
        w.mobile_mut(d).unwrap().deleted = true;
        assert!(info.expired(&w));
    }
}
