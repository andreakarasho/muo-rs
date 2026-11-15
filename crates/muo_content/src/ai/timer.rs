//! The AI timer: decisions at `CurrentSpeed`, plus move-only wakes at `NextMove` while a
//! pursuit is live. Each tick schedules the earlier of the two deadlines.

use muo_primitives::TimeSpan;
use muo_server::ids::MobileId;
use muo_server::timer::{self, TimerHandle};
use muo_server::world::World;

use super::{ai, ai_mut, base, movement};
use crate::creature::data::{cd, read};
use crate::creature::vtable::ctype;
use crate::hooks::rnd;
use crate::mx;

#[derive(Default)]
pub struct AiTimer {
    pub handle: TimerHandle,
    next_think: i64,
    next_wake: i64,
    in_tick: bool,
    detect_hidden_min_delay: i32,
    detect_hidden_max_delay: i32,
}

impl AiTimer {
    pub fn new(tick_count: i64) -> AiTimer {
        AiTimer {
            next_think: tick_count,
            ..Default::default()
        }
    }
}

fn speed_ms(w: &World, id: MobileId) -> i64 {
    (read(w, id, 0.0, |d| d.current_speed) * 1000.0) as i64
}

/// Creates the wheel node (the `AITimer` constructor). The delay is irrelevant: Activate is the only start path.
pub fn create(w: &mut World, id: MobileId) {
    let interval = TimeSpan::from_secs_f64(read(w, id, 0.0, |d| d.current_speed));
    let h = timer::create(w, TimeSpan::ZERO, interval, 0, move |w| on_tick(w, id));
    let tick = w.tick_count;
    if let Some(a) = ai_mut(w, id) {
        a.timer.handle = h;
        a.timer.next_think = tick;
        a.next_detect_hidden = tick;
    }
}

pub fn running(w: &World, id: MobileId) -> bool {
    ai(w, id).is_some_and(|a| a.timer.handle.running(w))
}

fn handle(w: &World, id: MobileId) -> TimerHandle {
    ai(w, id).map(|a| a.timer.handle).unwrap_or_default()
}

/// `Activate`.
pub fn activate(w: &mut World, id: MobileId) {
    let tick = w.tick_count;
    if let Some(a) = ai_mut(w, id) {
        a.timer.next_think = tick;
    }
    if running(w, id) {
        return;
    }
    // Short random spread: the creature responds within a think while a sector's worth of timers avoids a
    // same-tick burst; the idle think jitter keeps the cohort apart from there.
    let delay = rnd::below(w, 256) as i64;
    let h = handle(w, id);
    timer::set_delay(w, h, TimeSpan::from_millis(delay));
    timer::begin(w, h);
    if let Some(a) = ai_mut(w, id) {
        a.timer.next_wake = tick + delay;
    }
}

/// `Prod`: think now.
pub fn prod(w: &mut World, id: MobileId) {
    let tick = w.tick_count;
    if let Some(a) = ai_mut(w, id) {
        a.timer.next_think = tick;
    } else {
        return;
    }
    if running(w, id) {
        reschedule(w, id);
        return;
    }
    let h = handle(w, id);
    timer::set_delay(w, h, TimeSpan::ZERO);
    timer::begin(w, h);
    if let Some(a) = ai_mut(w, id) {
        a.timer.next_wake = tick;
    }
}

/// `OnSpeedChanged`: a speed-up must not wait out a stale, longer think deadline.
pub fn on_speed_changed(w: &mut World, id: MobileId) {
    let Some(a) = ai(w, id) else { return };
    let candidate = w.tick_count + speed_ms(w, id);
    if candidate - a.timer.next_think < 0 {
        if let Some(a) = ai_mut(w, id) {
            a.timer.next_think = candidate;
        }
        reschedule(w, id);
    }
}

/// Moves the pending wake earlier: Stop, Delay = remaining, Start.
fn reschedule(w: &mut World, id: MobileId) {
    let Some(a) = ai(w, id) else { return };
    if a.timer.in_tick || !running(w, id) {
        return; // ScheduleNext handles it at tick end
    }
    let now = w.tick_count;
    let mut deadline = a.timer.next_think;
    if let Some(next_move) = movement::try_get_move_wake(w, id)
        && next_move - now > 0
        && next_move - deadline < 0
    {
        deadline = next_move;
    }
    if deadline - a.timer.next_wake >= 0 {
        return; // pending wake is already early enough
    }
    let h = handle(w, id);
    timer::stop(w, h);
    let delay = (deadline - now).max(0);
    timer::set_delay(w, h, TimeSpan::from_millis(delay));
    timer::begin(w, h);
    if let Some(a) = ai_mut(w, id) {
        a.timer.next_wake = now + delay;
    }
}

/// `AITimer.Stop()`: pauses the timer; Activate restarts it.
pub fn pause(w: &mut World, id: MobileId) {
    let h = handle(w, id);
    timer::stop(w, h);
}

/// `AIObject?.AITimer.Stop()` before the AI is dropped: frees the wheel node.
pub fn stop(w: &mut World, id: MobileId) {
    let h = handle(w, id);
    timer::stop(w, h);
    timer::release(w, h);
    if let Some(a) = ai_mut(w, id) {
        a.timer.handle = TimerHandle::default();
    }
}

fn should_stop(w: &mut World, id: MobileId) -> bool {
    if mx::deleted(w, id) {
        return true;
    }
    let map = mx::map(w, id);
    let sensitive = ctype(w, id).is_some_and(|c| (c.player_range_sensitive)(w, id));
    let controlled = read(w, id, false, |d| d.controlled);
    if map.is_none()
        || map == Some(muo_server::map::MapId::INTERNAL)
        || sensitive && !controlled && !crate::creature::overrides::sector_active(w, id)
    {
        base::deactivate(w, id);
        return true;
    }
    false
}

fn on_tick(w: &mut World, id: MobileId) {
    let Some(a) = ai_mut(w, id) else { return };
    a.timer.in_tick = true;
    on_tick_core(w, id);
    if let Some(a) = ai_mut(w, id) {
        a.timer.in_tick = false;
    }
}

fn on_tick_core(w: &mut World, id: MobileId) {
    if should_stop(w, id) {
        pause(w, id);
        return;
    }
    let tick = w.tick_count;
    let Some(next_think) = ai(w, id).map(|a| a.timer.next_think) else {
        return;
    };
    if tick - next_think >= 0 {
        if let Some(c) = ctype(w, id) {
            use muo_server::script_hooks::{ScriptHook, mobile as script};
            if !script(w, id, ScriptHook::Think, &[]) {
                (c.on_think)(w, id);
            }
        }
        if should_stop(w, id) {
            pause(w, id);
            return;
        }
        handle_bard_effects(w, id);
        let Some(class) = ai(w, id).map(|a| a.class) else {
            return;
        };
        let controlled = read(w, id, false, |d| d.controlled);
        let acted = if controlled {
            (class.obey)(w, id)
        } else {
            (class.think)(w, id)
        };
        if acted {
            handle_detect_hidden(w, id);
        }
        // Cadence from the post-decision speed (decisions may flip active/passive).
        let period = speed_ms(w, id);
        let mut nt = w.tick_count + period;
        // Idle cadence drifts: a zero-mean jitter random-walks think phases apart, so creatures spawned or woken
        // together cannot stay in lock-step. Engaged cadence stays exact.
        let (cur, passive) = read(w, id, (0.0, 0.0), |d| (d.current_speed, d.passive_speed));
        if cur == passive {
            let jitter = (period >> 3) as i32;
            nt += rnd::min_max(w, -jitter, jitter) as i64;
        }
        if let Some(a) = ai_mut(w, id) {
            a.timer.next_think = nt;
        }
    } else {
        movement::continue_move(w, id);
    }
    schedule_next(w, id);
}

fn schedule_next(w: &mut World, id: MobileId) {
    let now = w.tick_count;
    let Some(next_think) = ai(w, id).map(|a| a.timer.next_think) else {
        return;
    };
    let mut delay = next_think - now;
    if let Some(next_move) = movement::try_get_move_wake(w, id) {
        let move_delay = next_move - now;
        // Only a future budget is a wake: a blocked creature must not spin the timer.
        if move_delay > 0 && move_delay < delay {
            delay = move_delay;
        }
    }
    // The wheel rounds up to its 8ms resolution; a non-positive delay becomes one turn.
    let h = handle(w, id);
    timer::set_interval(w, h, TimeSpan::from_millis(delay));
    if let Some(a) = ai_mut(w, id) {
        a.timer.next_wake = now + delay.max(0);
    }
}

fn handle_bard_effects(w: &mut World, id: MobileId) {
    let (pacified, provoked) = read(w, id, (false, false), |d| {
        (d.bard_pacified, d.bard_provoked)
    });
    if pacified {
        base::do_bard_pacified(w, id);
    } else if provoked {
        base::do_bard_provoked(w, id);
    }
}

fn handle_detect_hidden(w: &mut World, id: MobileId) {
    let Some(class) = ai(w, id).map(|a| a.class) else {
        return;
    };
    let tick = w.tick_count;
    let next = ai(w, id).map_or(0, |a| a.next_detect_hidden);
    if !(class.can_detect_hidden)(w, id) || tick - next < 0 {
        return;
    }
    (class.detect_hidden)(w, id);
    let (mut min, mut max) = ai(w, id).map_or((0, 0), |a| {
        (
            a.timer.detect_hidden_min_delay,
            a.timer.detect_hidden_max_delay,
        )
    });
    if min == 0 || max == 0 {
        let int = w.mobile(id).map_or(1, |m| (m.ty.int)(w, id)).max(1);
        let delay = (30000 / int).min(120);
        min = delay * 900; // 26s to 108s
        max = delay * 1100; // 32s to 132s
    }
    let next = w.tick_count + rnd::min_max(w, min, max) as i64;
    if let Some(a) = ai_mut(w, id) {
        a.timer.detect_hidden_min_delay = min;
        a.timer.detect_hidden_max_delay = max;
        a.next_detect_hidden = next;
    }
}

#[allow(dead_code)]
fn _unused(w: &World, id: MobileId) -> bool {
    cd(w, id).is_some()
}
