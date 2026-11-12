//! Stat mods' non-virtual half, the regen timers (`HitsTimer`, `StamTimer`, `ManaTimer`),
//! `InitStats`, `DefaultMobileInit`, `CheckAlive`, `Logout` and `ExpireCriminal`.

use muo_primitives::TimeSpan;

use super::{deps, messages, statics};
use crate::events;
use crate::ids::MobileId;
use crate::map::MapId;
use crate::mobile::{MobileDelta, StatLockType, StatType};
use crate::skills::{SkillName, Skills};
use crate::target::Targeted;
use crate::timer::{self, TimerHandle};
use crate::world::World;
use muo_primitives::MessageType;

pub fn get_stat_delta(ty: StatType) -> MobileDelta {
    let mut delta = MobileDelta::NONE;
    if ty.intersects(StatType::STR) {
        delta |= MobileDelta::HITS;
    }
    if ty.intersects(StatType::DEX) {
        delta |= MobileDelta::STAM;
    }
    if ty.intersects(StatType::INT) {
        delta |= MobileDelta::MANA;
    }
    delta
}

/// The sum of the offsets of the mods that touch `ty` and have not elapsed.
pub fn get_stat_offset(w: &World, id: MobileId, ty: StatType) -> i32 {
    let Some(m) = w.mobile(id) else { return 0 };
    m.stat_mods
        .iter()
        .filter(|s| s.ty.intersects(ty) && !s.has_elapsed(w.now))
        .map(|s| s.offset)
        .sum()
}

/// What `new Mobile()` does after the field initializers. `Mobile::new_at`
/// already builds this; the function is for a mobile built with `Mobile::new_for_load`.
pub fn default_mobile_init(w: &mut World, id: MobileId) {
    let tick = w.tick_count;
    let table_len = w
        .resources
        .get::<crate::skills::SkillTable>()
        .map_or(0, |t| t.info.len());
    let Some(m) = w.mobile_mut(id) else { return };
    m.stat_cap = 225;
    m.followers_max = 5;
    let mut skills = Skills::default();
    skills.ensure(table_len);
    m.skills = Some(skills);
    m.items = Vec::new();
    m.auto_page_notify = true;
    m.aggressors = Vec::new();
    m.aggressed = Vec::new();
    m.next_skill_time = tick;
    deps::set_map(w, id, Some(MapId::INTERNAL));
}

pub fn check_alive(w: &mut World, id: MobileId, message: bool) -> bool {
    let Some(ty) = w.mobile(id).map(|m| m.ty) else {
        return false;
    };
    if (ty.alive)(w, id) {
        return true;
    }
    if message {
        // I am dead and cannot do that.
        messages::local_overhead_message_number(w, id, MessageType::REGULAR, 0x3B2, 1019048, "");
    }
    false
}

pub fn init_stats(w: &mut World, id: MobileId, str: i32, dex: i32, intel: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    m.str = str;
    m.dex = dex;
    m.int = intel;
    let ty = m.ty;
    deps::set_hits(w, id, (ty.hits_max)(w, id));
    deps::set_stam(w, id, (ty.stam_max)(w, id));
    deps::set_mana(w, id, (ty.mana_max)(w, id));
    (ty.delta)(
        w,
        id,
        MobileDelta::STAT | MobileDelta::HITS | MobileDelta::STAM | MobileDelta::MANA,
    );
}

// ---------- CheckSkill / CheckTargetSkill ----------

pub fn check_skill(w: &mut World, id: MobileId, skill: SkillName, min: f64, max: f64) -> bool {
    match statics::read(w, |s| s.skill_check_location_handler) {
        Some(h) => h(w, id, skill, min, max),
        None => false,
    }
}

pub fn check_skill_chance(w: &mut World, id: MobileId, skill: SkillName, chance: f64) -> bool {
    match statics::read(w, |s| s.skill_check_direct_location_handler) {
        Some(h) => h(w, id, skill, chance),
        None => false,
    }
}

pub fn check_target_skill(
    w: &mut World,
    id: MobileId,
    skill: SkillName,
    target: Targeted,
    min: f64,
    max: f64,
) -> bool {
    match statics::read(w, |s| s.skill_check_target_handler) {
        Some(h) => h(w, id, skill, target, min, max),
        None => false,
    }
}

pub fn check_target_skill_chance(
    w: &mut World,
    id: MobileId,
    skill: SkillName,
    target: Targeted,
    chance: f64,
) -> bool {
    match statics::read(w, |s| s.skill_check_direct_target_handler) {
        Some(h) => h(w, id, skill, target, chance),
        None => false,
    }
}

// ---------- regen timers ----------

pub fn get_hits_regen_rate(w: &World, m: MobileId) -> TimeSpan {
    match statics::read(w, |s| s.hits_regen_rate_handler) {
        Some(h) => h(w, m),
        None => statics::read(w, |s| s.default_hits_rate),
    }
}

pub fn get_stam_regen_rate(w: &World, m: MobileId) -> TimeSpan {
    match statics::read(w, |s| s.stam_regen_rate_handler) {
        Some(h) => h(w, m),
        None => statics::read(w, |s| s.default_stam_rate),
    }
}

pub fn get_mana_regen_rate(w: &World, m: MobileId) -> TimeSpan {
    match statics::read(w, |s| s.mana_regen_rate_handler) {
        Some(h) => h(w, m),
        None => statics::read(w, |s| s.default_mana_rate),
    }
}

/// Which stat a regen timer feeds.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Regen {
    Hits,
    Stam,
    Mana,
}

impl Regen {
    fn handle(self, w: &World, id: MobileId) -> TimerHandle {
        w.mobile(id)
            .map_or_else(TimerHandle::default, |m| match self {
                Regen::Hits => m.hits_timer,
                Regen::Stam => m.stam_timer,
                Regen::Mana => m.mana_timer,
            })
    }

    fn set_handle(self, w: &mut World, id: MobileId, h: TimerHandle) {
        if let Some(m) = w.mobile_mut(id) {
            match self {
                Regen::Hits => m.hits_timer = h,
                Regen::Stam => m.stam_timer = h,
                Regen::Mana => m.mana_timer = h,
            }
        }
    }

    fn rate(self, w: &mut World, id: MobileId) -> TimeSpan {
        if let Some(pre) = statics::read(w, |s| s.regen_pre_rate) {
            pre(w, id, self as i32);
        }
        match self {
            Regen::Hits => get_hits_regen_rate(w, id),
            Regen::Stam => get_stam_regen_rate(w, id),
            Regen::Mana => get_mana_regen_rate(w, id),
        }
    }
}

/// `m_XTimer ??= new XTimer(this); m_XTimer.Start();`: a running timer is left alone. The timer is
/// `Timer(rate, rate)` forever; each tick regenerates one point while the stat can, then re-reads the rate
/// (`Delay = Interval = GetXRegenRate(owner)`).
pub fn start_regen_timer(w: &mut World, id: MobileId, which: Regen) {
    if which.handle(w, id).running(w) {
        return;
    }
    let rate = which.rate(w, id);
    let h = timer::start(w, rate, rate, 0, move |w| regen_tick(w, id, which));
    which.set_handle(w, id, h);
}

/// `m_XTimer?.Stop()`.
pub fn stop_regen_timer(w: &mut World, id: MobileId, which: Regen) {
    let mut h = which.handle(w, id);
    h.cancel(w);
    which.set_handle(w, id, h);
}

fn regen_tick(w: &mut World, id: MobileId, which: Regen) {
    let Some(m) = w.mobile(id) else { return };
    let ty = m.ty;
    match which {
        Regen::Hits if (ty.can_regen_hits)(w, id) => {
            let v = m.hits + 1;
            deps::set_hits(w, id, v);
        }
        Regen::Stam if (ty.can_regen_stam)(w, id) => {
            let v = m.stam + 1;
            deps::set_stam(w, id, v);
        }
        Regen::Mana if (ty.can_regen_mana)(w, id) => {
            let v = m.mana + 1;
            deps::set_mana(w, id, v);
        }
        _ => {}
    }
    let rate = which.rate(w, id);
    if let Some(h) = timer::current(w) {
        timer::set_delay(w, h, rate);
        timer::set_interval(w, h, rate);
    }
}

// ---------- Logout / ExpireCriminal ----------

/// `Logout` (private; the logout timer's tick).
pub fn logout(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    if m.map != Some(MapId::INTERNAL) {
        let mut mid = id;
        events::invoke_logout(w, &mut mid);
        if let Some(m) = w.mobile_mut(id) {
            m.logout_location = m.location;
            m.logout_map = m.map;
        }
        super::items::internalize(w, id);
    }
}

/// `ExpireCriminal` (private; the criminal timer's tick).
pub fn expire_criminal(w: &mut World, id: MobileId) {
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.set_criminal)(w, id, false);
    }
}

/// Starts the criminal-expiry timer (`Timer.StartTimer(ExpireCriminalDelay, ExpireCriminal, out token)`).
pub fn start_expire_criminal_timer(w: &mut World, id: MobileId) {
    let delay = statics::read(w, |s| s.expire_criminal_delay);
    let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| expire_criminal(w, id));
    if let Some(m) = w.mobile_mut(id) {
        m.expire_criminal_timer = h;
    }
}

/// `StatLockType` from a save byte; a value without a name is `Up`. The three names are
/// the only ones a save holds.
pub fn stat_lock_from_u8(v: u8) -> StatLockType {
    match v {
        1 => StatLockType::Down,
        2 => StatLockType::Locked,
        _ => StatLockType::Up,
    }
}

pub fn has_queued_deltas(w: &World) -> bool {
    statics::read(w, |s| !s.delta_queue.is_empty())
}

/// Processes at most the mobiles queued when it started.
pub fn process_delta_queue(w: &mut World) {
    let mut limit = statics::read(w, |s| s.delta_queue.len()) as i64;
    loop {
        limit -= 1;
        if limit < 0 {
            break;
        }
        let Some(mob) = w
            .resources
            .get_mut::<statics::Part2Statics>()
            .delta_queue
            .pop_front()
        else {
            break;
        };
        let Some(m) = w.mobile_mut(mob) else { continue };
        m.in_delta_queue = false;
        let ty = m.ty;
        (ty.process_delta)(w, mob);
    }

    let left = statics::read(w, |s| s.delta_queue.len());
    if left > 0 {
        tracing::warn!("{left} mobiles left in delta queue after processing.");
    }
}

pub fn launch_browser(w: &mut World, id: MobileId, url: &str) {
    if let Some(ns) = w.mobile(id).and_then(|m| m.net_state) {
        // NetState.LaunchBrowser: the "launching web browser" label first
        crate::packets::message::send_message_localized(
            w,
            ns,
            muo_primitives::Serial::MINUS_ONE,
            -1,
            crate::packets::message::MessageType::LABEL,
            0x35,
            3,
            501231,
            "",
            "",
        );
        crate::packets::player::send_launch_browser(w, ns, url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::MOBILE_BASE;
    use crate::mods::StatMod;

    #[test]
    fn stat_delta_and_offset() {
        assert_eq!(
            get_stat_delta(StatType::STR | StatType::INT),
            MobileDelta::HITS | MobileDelta::MANA
        );
        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        let a = StatMod::new(
            &mut w,
            StatType::STR,
            Some("a"),
            10,
            TimeSpan::ZERO,
            Some(id),
        );
        let b = StatMod::new(
            &mut w,
            StatType::STR | StatType::DEX,
            Some("b"),
            5,
            TimeSpan::ZERO,
            Some(id),
        );
        let c = StatMod::new(
            &mut w,
            StatType::INT,
            Some("c"),
            7,
            TimeSpan::ZERO,
            Some(id),
        );
        w.mobile_mut(id).unwrap().stat_mods.extend([a, b, c]);
        assert_eq!(get_stat_offset(&w, id, StatType::STR), 15);
        assert_eq!(get_stat_offset(&w, id, StatType::DEX), 5);
        assert_eq!(get_stat_offset(&w, id, StatType::INT), 7);
        assert_eq!(get_stat_offset(&w, id, StatType::ALL), 22);
    }

    #[test]
    fn check_alive_and_names() {
        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        assert!(check_alive(&mut w, id, true));
        w.mobile_mut(id).unwrap().deleted = true;
        assert!(!check_alive(&mut w, id, false));
        assert_eq!(
            statics::ACCESS_LEVEL_NAMES[crate::mobile::AccessLevel::GameMaster as usize],
            "a game master"
        );
        assert_eq!(stat_lock_from_u8(2), StatLockType::Locked);
    }

    #[test]
    fn regen_timer_start_is_idempotent_and_stop_clears() {
        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        start_regen_timer(&mut w, id, Regen::Hits);
        let h = w.mobile(id).unwrap().hits_timer;
        assert!(h.running(&w));
        start_regen_timer(&mut w, id, Regen::Hits);
        assert_eq!(w.mobile(id).unwrap().hits_timer, h);
        stop_regen_timer(&mut w, id, Regen::Hits);
        assert!(!w.mobile(id).unwrap().hits_timer.running(&w));
    }
}
