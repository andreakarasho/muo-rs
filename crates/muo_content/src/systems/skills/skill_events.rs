//! Skill events: raised once per skill attempt with the attempt's outcome, including attempts the
//! handler resolves without a roll (too difficult, no challenge). Not raised when the mobile lacks the skill.

use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::skills::SkillName;

pub type SkillUsedHandler = fn(&mut World, MobileId, SkillName, bool);

/// `SkillEvents.SkillUsed`.
#[derive(Default)]
pub struct SkillEvents {
    handlers: Vec<SkillUsedHandler>,
}

/// `SkillEvents.SkillUsed += handler`.
pub fn subscribe_skill_used(w: &mut World, handler: SkillUsedHandler) {
    w.resources.get_mut::<SkillEvents>().handlers.push(handler);
}

/// `SkillEvents.InvokeSkillUsed(from, skill, success)`.
pub fn invoke_skill_used(w: &mut World, from: MobileId, skill: SkillName, success: bool) {
    let handlers = w.resources.get_mut::<SkillEvents>().handlers.clone();
    for h in handlers {
        h(w, from, skill, success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static SEEN: AtomicU32 = AtomicU32::new(0);

    #[test]
    fn handlers_see_every_attempt() {
        let mut w = World::new();
        let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        subscribe_skill_used(&mut w, |_, _, s, ok| {
            SEEN.fetch_add(if ok { 10 } else { 1 } * (s as u32 + 1), Ordering::SeqCst);
        });
        invoke_skill_used(&mut w, m, SkillName::Alchemy, true);
        invoke_skill_used(&mut w, m, SkillName::Anatomy, false);
        assert_eq!(SEEN.load(Ordering::SeqCst), 10 + 2);
    }
}
