//! Autosave: the once-a-second timer that saves the world on the configured
//! schedule, with an optional warning broadcast.
//!
//! "Local midnight" uses the system time zone: `local_time::configure` sets [`LocalTimeOffset`] at startup and
//! [`TrackSystemTimeZone`] makes each `SetNextSave` refresh it (daylight saving); without them the offset is zero.

use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::timer::{self, TimerHandle};

/// `Core.Now.ToSystemLocalTime()`'s offset from UTC.
#[derive(Default)]
pub struct LocalTimeOffset(pub TimeSpan);

/// Whether `SetNextSave` reads the system time zone each time (set by `local_time::configure`).
#[derive(Default)]
pub struct TrackSystemTimeZone(pub bool);

/// `AutoRestart.Restarting`, set by `login::small` (Misc/AutoRestart) when it begins a restart.
pub use crate::login::small::AutoRestartState;

/// The `AutoSave` statics.
#[derive(Default)]
pub struct AutoSave {
    enabled: bool,
    next_save: DateTime,
    delay: TimeSpan,
    warning: TimeSpan,
    timer: Option<TimerHandle>,
}

impl AutoSave {
    pub fn delay(w: &mut World) -> TimeSpan {
        w.resources.get_mut::<AutoSave>().delay
    }

    pub fn warning(w: &mut World) -> TimeSpan {
        w.resources.get_mut::<AutoSave>().warning
    }

    pub fn saves_enabled(w: &mut World) -> bool {
        w.resources.get_mut::<AutoSave>().enabled
    }
}

/// `SavesEnabled = value`.
pub fn set_saves_enabled(w: &mut World, value: bool) {
    w.resources.get_mut::<AutoSave>().enabled = value;
    if value {
        set_next_save(w);
        let running = w
            .resources
            .get_mut::<AutoSave>()
            .timer
            .is_some_and(|t| t.running(w));
        if !running {
            let h = timer::start(
                w,
                TimeSpan::from_secs(1),
                TimeSpan::from_secs(1),
                0,
                on_tick,
            );
            w.resources.get_mut::<AutoSave>().timer = Some(h);
        }
    } else if let Some(mut t) = w.resources.get_mut::<AutoSave>().timer.take() {
        t.cancel(w);
    }
}

/// `AutoSave.Configure()`.
pub fn configure(w: &mut World) {
    let delay = w
        .config
        .get_or_update_setting("autosave.saveDelay", TimeSpan::from_mins(5))
        .unwrap_or(TimeSpan::from_mins(5));
    let warning = w
        .config
        .get_or_update_setting("autosave.warningDelay", TimeSpan::ZERO)
        .unwrap_or(TimeSpan::ZERO);
    let enabled = w
        .config
        .get_or_update_setting("autosave.enabled", true)
        .unwrap_or(true);
    let s = w.resources.get_mut::<AutoSave>();
    s.delay = delay;
    s.warning = warning;
    set_saves_enabled(w, enabled);
}

/// `ResetAutoSave(saveDelay, warningDelay)`.
pub fn reset_auto_save(w: &mut World, save_delay: TimeSpan, warning_delay: TimeSpan) {
    let s = w.resources.get_mut::<AutoSave>();
    if save_delay != s.delay || warning_delay != s.warning {
        s.delay = save_delay;
        s.warning = warning_delay;
        let (d, wn) = (s.delay, s.warning);
        if let Err(e) = w.config.set_setting("autosave.saveDelay", &d) {
            tracing::error!("Saving the configuration failed: {e}");
        }
        if let Err(e) = w.config.set_setting("autosave.warningDelay", &wn) {
            tracing::error!("Saving the configuration failed: {e}");
        }
    }
    set_saves_enabled(w, true);
}

/// `SetNextSave(now)`: the next multiple of the delay since local midnight, less the warning.
fn set_next_save(w: &mut World) {
    let now = w.now;
    if w.resources.get_mut::<TrackSystemTimeZone>().0 {
        w.resources.get_mut::<LocalTimeOffset>().0 = super::local_time::offset_at(now);
    }
    let offset = w.resources.get_mut::<LocalTimeOffset>().0;
    let s = w.resources.get_mut::<AutoSave>();
    let delay = s.delay.ticks();
    if delay <= 0 {
        // A zero or negative delay saves every tick.
        s.next_save = now;
        return;
    }
    let time_of_day = (now + offset).time_of_day().ticks();
    s.next_save = now + TimeSpan::from_ticks(delay - time_of_day % delay) - s.warning;
}

fn on_tick(w: &mut World) {
    let (next, enabled, delay, warning) = {
        let s = w.resources.get_mut::<AutoSave>();
        (s.next_save, s.enabled, s.delay, s.warning)
    };
    if next > w.now {
        return;
    }
    let restarting = w.resources.get_mut::<AutoRestartState>().restarting;
    if enabled && !restarting && !w.core.closing() && !w.saving() {
        if warning > TimeSpan::ZERO {
            broadcast_warning(w, warning);
            timer::start(w, warning, TimeSpan::ZERO, 1, save);
        } else {
            save(w);
        }
    }
    w.resources.get_mut::<AutoSave>().next_save = next + delay;
}

fn plural(n: i32) -> &'static str {
    if n != 1 { "s" } else { "" }
}

fn broadcast_warning(w: &mut World, warning: TimeSpan) {
    let total = warning.total_seconds().round_ties_even() as i32;
    let (m, s) = (total / 60, total % 60);
    let text = if m > 0 && s > 0 {
        format!(
            "The world will save in {m} minute{} and {s} second{}.",
            plural(m),
            plural(s)
        )
    } else if m > 0 {
        format!("The world will save in {m} minute{}.", plural(m))
    } else {
        format!("The world will save in {s} second{}.", plural(s))
    };
    w.broadcast(0x35, true, &text);
}

/// `AutoSave.Save()`.
pub fn save(w: &mut World) {
    let restarting = w.resources.get_mut::<AutoRestartState>().restarting;
    if !restarting && w.running() {
        w.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_save_is_the_next_multiple_of_the_delay_less_the_warning() {
        let mut w = World::new();
        w.now = DateTime::from_ymd_hms(2024, 5, 6, 7, 3, 20).unwrap();
        {
            let s = w.resources.get_mut::<AutoSave>();
            s.delay = TimeSpan::from_mins(5);
            s.warning = TimeSpan::from_secs(30);
        }
        set_next_save(&mut w);
        assert_eq!(
            w.resources.get_mut::<AutoSave>().next_save,
            DateTime::from_ymd_hms(2024, 5, 6, 7, 4, 30).unwrap()
        );
    }

    #[test]
    fn warning_text() {
        // The text is built in `broadcast_warning`; check the pluralizing it relies on.
        assert_eq!(plural(1), "");
        assert_eq!(plural(2), "s");
        assert_eq!(plural(0), "s");
    }

    #[test]
    fn enabling_starts_one_timer_and_disabling_stops_it() {
        let mut w = World::new();
        w.resources.get_mut::<AutoSave>().delay = TimeSpan::from_mins(5);
        set_saves_enabled(&mut w, true);
        let t = w.resources.get_mut::<AutoSave>().timer.unwrap();
        assert!(t.running(&w));
        set_saves_enabled(&mut w, true);
        assert_eq!(w.resources.get_mut::<AutoSave>().timer, Some(t));
        set_saves_enabled(&mut w, false);
        assert!(!t.running(&w));
        assert!(w.resources.get_mut::<AutoSave>().timer.is_none());
    }
}
