//! Scheduled events: `EventScheduler` (a one-second timer over a priority queue of due times),
//! `BaseScheduledEvent`/`ScheduledEvent`/`CallbackScheduledEvent`/`YearlyScheduledEvent`, `MonthDay`, `AllowedDays`,
//! `AllowedMonths` and the common recurrence patterns.
//!
//! A `TimeZoneInfo` is a [`TimeZone`] (UTC offset plus the US daylight-saving rule). An event is one
//! record with a callback (and the optional yearly range), which covers every kind of scheduled event.

use std::collections::BTreeSet;
use std::rc::Rc;

use bitflags::bitflags;
use indexmap::IndexMap;
use muo_primitives::datetime::{DayOfWeek, days_in_month};
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::timer;
use tracing::error;

bitflags! {
    /// `AllowedDays`.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct AllowedDays: u8 {
        const SUNDAY = 1;
        const MONDAY = 1 << 1;
        const TUESDAY = 1 << 2;
        const WEDNESDAY = 1 << 3;
        const THURSDAY = 1 << 4;
        const FRIDAY = 1 << 5;
        const SATURDAY = 1 << 6;
        const ALL = 0x7F;
    }
}

bitflags! {
    /// `AllowedMonths`.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct AllowedMonths: u16 {
        const ALL = 0x0FFF;
    }
}

impl AllowedMonths {
    /// `(AllowedMonths)(1 << (month - 1))`.
    pub fn of_month(month: u32) -> AllowedMonths {
        AllowedMonths::from_bits_retain(1 << (month - 1))
    }
}

/// `DayOfWeek.ToDaysOfWeek()`.
pub fn to_days_of_week(d: DayOfWeek) -> AllowedDays {
    AllowedDays::from_bits_retain(1 << (d as u8))
}

/// `OrdinalDayOccurrence`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrdinalDayOccurrence {
    Last = -1,
    First = 0,
    Second = 1,
    Third = 2,
    Fourth = 3,
    Fifth = 4,
}

/// `MonthDay`: a month and day with no year.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MonthDay {
    pub month: u8,
    pub day: u8,
}

impl MonthDay {
    /// `new MonthDay(year, month, day)`: None for a date out of range.
    pub fn new(year: i32, month: i32, day: i32) -> Option<MonthDay> {
        if !(1..=12).contains(&month) {
            return None;
        }
        if day < 1 || day > days_in_month(year, month as u32) as i32 {
            return None;
        }
        Some(MonthDay {
            month: month as u8,
            day: day as u8,
        })
    }
}

/// `DateTime.IsBetween(MonthDay start, MonthDay end)`.
pub fn is_between(d: DateTime, start: MonthDay, end: MonthDay) -> bool {
    let (check_month, check_day) = (d.month() as u8, d.day() as u8);
    let is_after_start =
        check_month > start.month || check_month == start.month && check_day >= start.day;
    let is_before_end = check_month < end.month || check_month == end.month && check_day <= end.day;
    if start.month < end.month || start.month == end.month && start.day <= end.day {
        return is_after_start && is_before_end;
    }
    // spans the year boundary (e.g. Nov 15 - Feb 15)
    is_after_start || is_before_end
}

/// `TimeOnly` (hours and minutes are all the patterns read).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TimeOnly {
    pub hour: u32,
    pub minute: u32,
}

impl TimeOnly {
    /// `TimeOnly.FromDateTime`.
    pub fn from_date_time(d: DateTime) -> TimeOnly {
        TimeOnly {
            hour: d.hour(),
            minute: d.minute(),
        }
    }
    fn to_time_span(self) -> TimeSpan {
        TimeSpan::from_hours_f64(self.hour as f64) + TimeSpan::from_mins_f64(self.minute as f64)
    }
}

/// `TimeZoneInfo`: a standard UTC offset, with the US daylight-saving rule (second Sunday of March 02:00 to first
/// Sunday of November 02:00 wall time) when `us_dst` is set.
// shortcut: only the US rule, no tz database; add one when an event needs another zone's rules.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TimeZone {
    pub offset_minutes: i32,
    pub us_dst: bool,
}

impl TimeZone {
    pub const UTC: TimeZone = TimeZone {
        offset_minutes: 0,
        us_dst: false,
    };
    /// `TimeZoneInfo.FindSystemTimeZoneById("America/New_York")`.
    pub const NEW_YORK: TimeZone = TimeZone {
        offset_minutes: -300,
        us_dst: true,
    };

    fn nth_sunday(year: i32, month: u32, n: u32) -> DateTime {
        let first = DateTime::from_ymd(year, month, 1).unwrap();
        first.add_days((((7 - first.day_of_week() as i32) % 7) + 7 * (n as i32 - 1)) as f64)
    }
    /// The local wall time DST starts (02:00) and the local wall time it ends (02:00 DST, 01:00 standard).
    fn dst_range(year: i32) -> (DateTime, DateTime) {
        (
            Self::nth_sunday(year, 3, 2).add_hours(2.0),
            Self::nth_sunday(year, 11, 1).add_hours(2.0),
        )
    }

    /// `TimeZoneInfo.ConvertTimeFromUtc`.
    pub fn from_utc(self, utc: DateTime) -> DateTime {
        let std = utc.add_minutes(self.offset_minutes as f64);
        if self.us_dst {
            let (start, end) = Self::dst_range(std.year());
            if utc >= start.add_minutes(-self.offset_minutes as f64)
                && utc < end.add_minutes(-(self.offset_minutes + 60) as f64)
            {
                return std.add_hours(1.0);
            }
        }
        std
    }
    /// `TimeZoneInfo.IsInvalidTime`: a wall time skipped by the spring-forward.
    pub fn is_invalid_time(self, local: DateTime) -> bool {
        if !self.us_dst {
            return false;
        }
        let (start, _) = Self::dst_range(local.year());
        local >= start && local < start.add_hours(1.0)
    }
    /// `LocalToUtc`: a time in the skipped hour takes the standard offset, an ambiguous one the daylight offset.
    pub fn to_utc(self, local: DateTime) -> DateTime {
        let mut off = self.offset_minutes;
        if self.us_dst {
            let (start, end) = Self::dst_range(local.year());
            if local >= start.add_hours(1.0) && local < end {
                off += 60;
            }
        }
        local.add_minutes(-off as f64)
    }
}

/// A day at 00:00 plus a time, in the same way `new DateTime(y, m, d, h, mi, 0)` builds it.
fn at(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> Option<DateTime> {
    DateTime::from_ymd_hms(y, mo, d, h, mi, 0)
}

/// `IRecurrencePattern`.
#[derive(Copy, Clone, Debug)]
pub enum Recurrence {
    Hourly {
        interval_hours: i32,
    },
    Daily {
        interval_days: i32,
    },
    Weekly {
        interval_weeks: i32,
        allowed_months: AllowedMonths,
        allowed_days: AllowedDays,
    },
    /// `day_of_month` -1 is "the day of `after`".
    Monthly {
        day_of_month: i32,
        interval_months: i32,
    },
    MonthlyOrdinal {
        ordinal: OrdinalDayOccurrence,
        day_of_week: DayOfWeek,
        interval_months: i32,
    },
    /// Any other `IRecurrencePattern` implementation.
    Custom(fn(DateTime, TimeOnly, TimeZone) -> DateTime),
}

impl Recurrence {
    pub const HOURLY: Recurrence = Recurrence::Hourly { interval_hours: 1 };
    pub const DAILY: Recurrence = Recurrence::Daily { interval_days: 1 };
    pub const WEEKLY: Recurrence = Self::weekly(1);
    pub const BIWEEKLY: Recurrence = Self::weekly(2);
    pub const MONTHLY: Recurrence = Recurrence::Monthly {
        day_of_month: -1,
        interval_months: 1,
    };
    pub const YEARLY: Recurrence = Recurrence::Monthly {
        day_of_month: -1,
        interval_months: 12,
    };

    /// `new WeeklyRecurrencePattern(intervalWeeks)`.
    pub const fn weekly(interval_weeks: i32) -> Recurrence {
        Recurrence::Weekly {
            interval_weeks: if interval_weeks < 1 {
                1
            } else {
                interval_weeks
            },
            allowed_months: AllowedMonths::ALL,
            allowed_days: AllowedDays::ALL,
        }
    }

    /// `new WeeklyRecurrencePattern(intervalWeeks, months, days)`: `None` flags mean all.
    pub fn weekly_on(interval_weeks: i32, months: AllowedMonths, days: AllowedDays) -> Recurrence {
        Recurrence::Weekly {
            interval_weeks: interval_weeks.max(1),
            allowed_months: if months.is_empty() {
                AllowedMonths::ALL
            } else {
                months
            },
            allowed_days: if days.is_empty() {
                AllowedDays::ALL
            } else {
                days
            },
        }
    }

    /// `EventScheduler.GetMonthlyRecurrence(dayOfMonth)`.
    pub fn monthly_on(day_of_month: i32) -> Recurrence {
        Recurrence::Monthly {
            day_of_month: day_of_month.clamp(1, 31),
            interval_months: 1,
        }
    }

    /// `GetNextOccurrence(afterUtc, time, timeZone)`: UTC, or `DateTime.MaxValue`.
    pub fn next_occurrence(self, after_utc: DateTime, time: TimeOnly, tz: TimeZone) -> DateTime {
        let local = tz.from_utc(after_utc);
        match self {
            Recurrence::Custom(f) => f(after_utc, time, tz),
            Recurrence::Hourly { interval_hours } => {
                let h = interval_hours.max(1);
                at(
                    local.year(),
                    local.month(),
                    local.day(),
                    local.hour(),
                    time.minute,
                )
                .map(|d| tz.to_utc(d).add_hours(h as f64))
                .unwrap_or(DateTime::MAX)
            }
            Recurrence::Daily { interval_days } => {
                let d = interval_days.max(1);
                at(
                    local.year(),
                    local.month(),
                    local.day(),
                    time.hour,
                    time.minute,
                )
                .map(|x| tz.to_utc(x).add_days(d as f64))
                .unwrap_or(DateTime::MAX)
            }
            Recurrence::Weekly {
                interval_weeks,
                allowed_months,
                allowed_days,
            } => weekly_next(
                local,
                time,
                tz,
                interval_weeks.max(1),
                allowed_months,
                allowed_days,
            ),
            Recurrence::Monthly {
                day_of_month,
                interval_months,
            } => monthly_next(local, time, tz, day_of_month, interval_months.max(1)),
            Recurrence::MonthlyOrdinal {
                ordinal,
                day_of_week,
                interval_months,
            } => ordinal_next(
                local,
                time,
                tz,
                ordinal,
                day_of_week,
                interval_months.max(1),
            ),
        }
    }
}

fn weekly_next(
    local: DateTime,
    time: TimeOnly,
    tz: TimeZone,
    interval_weeks: i32,
    allowed_months: AllowedMonths,
    allowed_days: AllowedDays,
) -> DateTime {
    let days_of_week = if allowed_days.is_empty() {
        to_days_of_week(local.day_of_week())
    } else {
        allowed_days
    };
    let week_start = local.date().add_days(-(local.day_of_week() as i32 as f64));
    for week in 0..=52 {
        let next_week_start = week_start.add_days(7.0 * interval_weeks as f64 * week as f64);
        let week_end = next_week_start.add_days(6.0);
        let both = AllowedMonths::of_month(next_week_start.month())
            | AllowedMonths::of_month(week_end.month());
        // Skip the entire week if the start and end months are not in the allowed months
        if (allowed_months & both).is_empty() {
            continue;
        }
        for i in 0..7 {
            let day = next_week_start.add_days(i as f64);
            if (allowed_months & AllowedMonths::of_month(day.month())).is_empty() {
                continue;
            }
            if !(days_of_week & to_days_of_week(day.day_of_week())).is_empty()
                && let Some(candidate) =
                    at(day.year(), day.month(), day.day(), time.hour, time.minute)
                && candidate > local
                && !tz.is_invalid_time(candidate)
            {
                return tz.to_utc(candidate);
            }
        }
    }
    DateTime::MAX
}

fn monthly_next(
    local: DateTime,
    time: TimeOnly,
    tz: TimeZone,
    day_of_month: i32,
    interval: i32,
) -> DateTime {
    let day = if day_of_month == -1 {
        local.day() as i32
    } else {
        day_of_month.clamp(1, 31)
    };
    let (year, month) = (local.year(), local.month() as i32);
    for i in 0..100 {
        let next_month = month + interval * i;
        let candidate = DateTime::from_ymd(year, 1, 1)
            .and_then(|d| d.checked_add_months(next_month - 1))
            .and_then(|d| d.checked_add_ticks((day as i64 - 1) * 864_000_000_000))
            .and_then(|d| d.checked_add(time.to_time_span()));
        // Some months may not have that day of the month, if not, we skip to the next interval
        match candidate {
            Some(c) if c > local && c.day() as i32 == day && !tz.is_invalid_time(c) => {
                return tz.to_utc(c);
            }
            Some(_) => {}
            None => return DateTime::MAX, // no date past year 9999
        }
    }
    DateTime::MAX
}

fn ordinal_next(
    local: DateTime,
    time: TimeOnly,
    tz: TimeZone,
    ordinal: OrdinalDayOccurrence,
    day_of_week: DayOfWeek,
    interval: i32,
) -> DateTime {
    let (year, month) = (local.year(), local.month() as i32);
    for i in 0..100 {
        let next_month = month + interval * i;
        let candidate_year = year + (next_month - 1) / 12;
        let candidate_month = ((next_month - 1) % 12 + 1) as u32;
        let candidate = if ordinal as i32 >= OrdinalDayOccurrence::First as i32 {
            let Some(first) = at(candidate_year, candidate_month, 1, time.hour, time.minute) else {
                return DateTime::MAX;
            };
            let offset = (day_of_week as i32 - first.day_of_week() as i32 + 7) % 7;
            let c = first.add_days((offset + 7 * ordinal as i32) as f64);
            // If candidate is not in the same month, skip
            if c.month() != candidate_month {
                continue;
            }
            c
        } else {
            let dim = days_in_month(candidate_year, candidate_month);
            let Some(last) = at(candidate_year, candidate_month, dim, time.hour, time.minute)
            else {
                return DateTime::MAX;
            };
            let mut offset = last.day_of_week() as i32 - day_of_week as i32 + 7;
            if offset >= 7 {
                offset -= 7;
            }
            last.add_days(-offset as f64)
        };
        if candidate > local && !tz.is_invalid_time(candidate) {
            return tz.to_utc(candidate);
        }
    }
    DateTime::MAX
}

/// A scheduled event's handle.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct EventId(u32);

/// `BaseScheduledEvent` + `ScheduledEvent` + `YearlyScheduledEvent` with a callback.
pub struct ScheduledEvent {
    pub time_zone: TimeZone,
    pub next_occurrence: DateTime,
    pub cancelled: bool,
    pub recurrence: Option<Recurrence>,
    pub time: TimeOnly,
    pub end_date: DateTime,
    /// `YearlyStart`, `YearlyEnd` of a `YearlyScheduledEvent`.
    pub yearly: Option<(MonthDay, MonthDay)>,
    callback: Rc<dyn Fn(&mut World)>,
}

/// `EventScheduler`: the shared instance is a `World` resource.
#[derive(Default)]
pub struct EventScheduler {
    events: IndexMap<u32, ScheduledEvent>,
    /// The `PriorityQueue<BaseScheduledEvent, DateTime>`: (due, sequence, id).
    queue: BTreeSet<(DateTime, u64, u32)>,
    seq: u64,
    next_id: u32,
    started: bool,
}

fn sched(w: &mut World) -> &mut EventScheduler {
    w.resources.get_mut::<EventScheduler>()
}

/// `EventScheduler.Configure`: starts the one-second timer.
pub fn configure(w: &mut World) {
    if std::mem::replace(&mut sched(w).started, true) {
        return;
    }
    timer::start(w, TimeSpan::ZERO, TimeSpan::from_secs_f64(1.0), 0, on_tick);
}

/// The start of a schedule: `startAfter` is UTC, or a local time of the event's zone.
#[derive(Copy, Clone, Debug)]
pub enum Start {
    Utc(DateTime),
    Local(DateTime),
}

/// `ScheduleEvent(startOn, callback, recurrence, timeZone)`.
pub fn schedule_event(
    w: &mut World,
    start_on: Start,
    callback: impl Fn(&mut World) + 'static,
    recurrence: Option<Recurrence>,
    time_zone: Option<TimeZone>,
) -> EventId {
    let d = match start_on {
        Start::Utc(d) | Start::Local(d) => d,
    };
    schedule_event_at(
        w,
        start_on,
        TimeOnly::from_date_time(d),
        callback,
        recurrence,
        time_zone,
    )
}

/// `ScheduleEvent(after, time, callback, recurrence, timeZone)`.
pub fn schedule_event_at(
    w: &mut World,
    after: Start,
    time: TimeOnly,
    callback: impl Fn(&mut World) + 'static,
    recurrence: Option<Recurrence>,
    time_zone: Option<TimeZone>,
) -> EventId {
    let ev = ScheduledEvent {
        time_zone: TimeZone::UTC,
        next_occurrence: DateTime::MAX,
        cancelled: false,
        recurrence,
        time,
        end_date: DateTime::MAX,
        yearly: None,
        callback: Rc::new(callback),
    };
    add_event(w, ev, after, time_zone)
}

/// A `ScheduledEvent` with `endOn`: the occurrences stop at `end_on_utc`.
pub fn schedule_event_ending(
    w: &mut World,
    after: Start,
    time: TimeOnly,
    end_on_utc: DateTime,
    callback: impl Fn(&mut World) + 'static,
    recurrence: Option<Recurrence>,
) -> EventId {
    let ev = ScheduledEvent {
        time_zone: TimeZone::UTC,
        next_occurrence: DateTime::MAX,
        cancelled: false,
        recurrence,
        time,
        end_date: end_on_utc,
        yearly: None,
        callback: Rc::new(callback),
    };
    add_event(w, ev, after, None)
}

/// A `YearlyCallbackScheduledEvent`: `Schedule(scheduler, after, tz)` of the new event.
#[allow(clippy::too_many_arguments)] // one parameter per field of the call
pub fn schedule_yearly(
    w: &mut World,
    after: Start,
    time: TimeOnly,
    yearly_start: MonthDay,
    yearly_end: MonthDay,
    end_on: Option<Start>,
    callback: impl Fn(&mut World) + 'static,
    recurrence: Recurrence,
    time_zone: Option<TimeZone>,
) -> EventId {
    let tz = time_zone.unwrap_or(TimeZone::UTC);
    let end_date = match end_on {
        None => DateTime::MAX,
        Some(Start::Utc(d)) => d,
        // the constructor converts with the event's own (still default, UTC) TimeZone
        Some(Start::Local(d)) => TimeZone::UTC.to_utc(d),
    };
    let _ = tz;
    let ev = ScheduledEvent {
        time_zone: TimeZone::UTC,
        next_occurrence: DateTime::MAX,
        cancelled: false,
        recurrence: Some(recurrence),
        time,
        end_date,
        yearly: Some((yearly_start, yearly_end)),
        callback: Rc::new(callback),
    };
    add_event(w, ev, after, time_zone)
}

fn add_event(w: &mut World, ev: ScheduledEvent, after: Start, tz: Option<TimeZone>) -> EventId {
    let s = sched(w);
    s.next_id += 1;
    let id = s.next_id;
    s.events.insert(id, ev);
    let eid = EventId(id);
    schedule(w, eid, after, tz);
    eid
}

/// `BaseScheduledEvent.Schedule(scheduler, startAfter, timeZone)`.
pub fn schedule(w: &mut World, e: EventId, start_after: Start, tz: Option<TimeZone>) {
    cancel(w, e);
    let tz = tz.unwrap_or(TimeZone::UTC);
    if let Some(ev) = sched(w).events.get_mut(&e.0) {
        ev.time_zone = tz;
    }
    schedule_inner(w, e, start_after, true);
}

fn schedule_inner(w: &mut World, e: EventId, start_after: Start, is_first: bool) {
    let Some(tz) = sched(w).events.get(&e.0).map(|x| x.time_zone) else {
        return;
    };
    let after_utc = match start_after {
        Start::Utc(d) => d,
        Start::Local(d) => tz.to_utc(d),
    };
    let next = get_next_occurrence(sched(w), e.0, after_utc);
    // For the first occurrence, we should set it to the startAfter date if we have no recurrence.
    let at = if next == DateTime::MAX && is_first {
        after_utc
    } else {
        next
    };
    let s = sched(w);
    if let Some(ev) = s.events.get_mut(&e.0) {
        ev.cancelled = false;
        ev.next_occurrence = at;
    }
    if at != DateTime::MAX {
        enqueue(s, e.0);
    }
}

fn enqueue(s: &mut EventScheduler, id: u32) {
    if let Some(ev) = s.events.get(&id)
        && ev.next_occurrence < DateTime::MAX
    {
        s.seq += 1;
        let key = (ev.next_occurrence, s.seq, id);
        s.queue.insert(key);
    }
}

/// `ScheduledEvent.GetNextOccurrence` and the override of `YearlyScheduledEvent`.
fn get_next_occurrence(s: &EventScheduler, id: u32, after: DateTime) -> DateTime {
    let Some(ev) = s.events.get(&id) else {
        return DateTime::MAX;
    };
    let base = |after: DateTime| {
        let next = ev.recurrence.map_or(DateTime::MAX, |r| {
            r.next_occurrence(after, ev.time, ev.time_zone)
        });
        if next >= ev.end_date {
            DateTime::MAX
        } else {
            next
        }
    };
    let next = base(after);
    let Some((ys, ye)) = ev.yearly else {
        return next;
    };
    if next == DateTime::MAX {
        return DateTime::MAX;
    }
    let local_next = ev.time_zone.from_utc(next);
    if is_between(local_next, ys, ye) {
        return next;
    }
    // If we're after the end of this year's range but before the start of next year's range
    let year = if ys > ye && local_next.month() as u8 > ye.month {
        local_next.year()
    } else {
        local_next.year() + 1
    };
    let Some(start) = DateTime::from_ymd(year, ys.month as u32, ys.day as u32) else {
        return DateTime::MAX;
    };
    let start_utc = ev.time_zone.to_utc(start);
    match ev.recurrence {
        Some(r) => r.next_occurrence(start_utc, ev.time, ev.time_zone),
        None => DateTime::MAX,
    }
}

/// `BaseScheduledEvent.Cancel`.
pub fn cancel(w: &mut World, e: EventId) {
    let s = sched(w);
    if let Some(ev) = s.events.get_mut(&e.0) {
        ev.cancelled = true;
    }
    unschedule(s, e.0);
}

/// `UnscheduleEvent`: out of the queue.
fn unschedule(s: &mut EventScheduler, id: u32) {
    s.queue.retain(|&(_, _, i)| i != id);
}

/// `NextOccurrence` of an event.
pub fn next_occurrence(w: &World, e: EventId) -> Option<DateTime> {
    w.resources
        .get::<EventScheduler>()?
        .events
        .get(&e.0)
        .map(|x| x.next_occurrence)
}

/// `Cancelled`.
pub fn is_cancelled(w: &World, e: EventId) -> bool {
    w.resources
        .get::<EventScheduler>()
        .and_then(|s| s.events.get(&e.0))
        .is_none_or(|x| x.cancelled)
}

/// `EventScheduler.OnTick`.
fn on_tick(w: &mut World) {
    let now = w.now;
    loop {
        let s = sched(w);
        let Some(&(due, seq, id)) = s.queue.first() else {
            break;
        };
        let cancelled = s.events.get(&id).is_none_or(|e| e.cancelled);
        if !cancelled && due > now {
            break;
        }
        s.queue.remove(&(due, seq, id));
        if cancelled {
            continue;
        }
        advance(w, id);
    }
}

/// `BaseScheduledEvent.Advance`: runs the event, then schedules the next occurrence.
fn advance(w: &mut World, id: u32) {
    let Some((cb, next)) = sched(w)
        .events
        .get(&id)
        .map(|e| (e.callback.clone(), e.next_occurrence))
    else {
        return;
    };
    // A panic in the callback is caught: logged, and the event still reschedules.
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| cb(w))).is_err() {
        error!("OnEvent failed to execute.");
    }
    schedule_inner(w, EventId(id), Start::Utc(next), false);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime {
        DateTime::from_ymd_hms(y, mo, d, h, mi, 0).unwrap()
    }
    const T: TimeOnly = TimeOnly {
        hour: 12,
        minute: 30,
    };

    #[test]
    fn daily_is_the_next_day_at_the_time() {
        // 12:30 today + 1 day, whatever `after` is (the interval is added to today's slot)
        let n = Recurrence::DAILY.next_occurrence(dt(2024, 3, 10, 9, 0), T, TimeZone::UTC);
        assert_eq!(n, dt(2024, 3, 11, 12, 30));
    }

    #[test]
    fn weekly_finds_the_next_allowed_day() {
        let r = Recurrence::weekly_on(
            1,
            AllowedMonths::ALL,
            AllowedDays::FRIDAY | AllowedDays::MONDAY,
        );
        // 2024-03-10 is a Sunday: the next Monday is the 11th
        let n = r.next_occurrence(dt(2024, 3, 10, 13, 0), T, TimeZone::UTC);
        assert_eq!(n, dt(2024, 3, 11, 12, 30));
    }

    #[test]
    fn monthly_skips_months_without_the_day() {
        let r = Recurrence::monthly_on(31);
        let n = r.next_occurrence(dt(2024, 4, 1, 0, 0), T, TimeZone::UTC);
        assert_eq!(n, dt(2024, 5, 31, 12, 30));
    }

    #[test]
    fn ordinal_last_friday() {
        let r = Recurrence::MonthlyOrdinal {
            ordinal: OrdinalDayOccurrence::Last,
            day_of_week: DayOfWeek::Friday,
            interval_months: 1,
        };
        let n = r.next_occurrence(dt(2024, 3, 1, 0, 0), T, TimeZone::UTC);
        assert_eq!(n, dt(2024, 3, 29, 12, 30));
    }

    #[test]
    fn month_day_range_wraps_the_year() {
        let (s, e) = (
            MonthDay::new(2024, 11, 15).unwrap(),
            MonthDay::new(2024, 2, 15).unwrap(),
        );
        assert!(is_between(dt(2024, 12, 25, 0, 0), s, e));
        assert!(is_between(dt(2024, 1, 5, 0, 0), s, e));
        assert!(!is_between(dt(2024, 6, 1, 0, 0), s, e));
        assert!(MonthDay::new(2023, 2, 29).is_none());
    }

    #[test]
    fn scheduler_runs_due_events_and_reschedules() {
        let mut w = World::new();
        w.now = dt(2024, 3, 10, 12, 0);
        let hits = Rc::new(std::cell::Cell::new(0));
        let h = hits.clone();
        let e = schedule_event_at(
            &mut w,
            Start::Utc(dt(2024, 3, 10, 12, 0)),
            T,
            move |_| h.set(h.get() + 1),
            Some(Recurrence::DAILY),
            None,
        );
        assert_eq!(next_occurrence(&w, e), Some(dt(2024, 3, 11, 12, 30)));
        w.now = dt(2024, 3, 11, 12, 31);
        on_tick(&mut w);
        assert_eq!(hits.get(), 1);
        assert_eq!(next_occurrence(&w, e), Some(dt(2024, 3, 12, 12, 30)));
        cancel(&mut w, e);
        w.now = dt(2024, 3, 13, 0, 0);
        on_tick(&mut w);
        assert_eq!(hits.get(), 1);
    }
}

/// Behavior tests of the event scheduler.
#[cfg(test)]
mod behavior_tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    fn dt(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime {
        DateTime::from_ymd_hms(y, mo, d, h, mi, 0).unwrap()
    }
    fn t(h: u32, m: u32) -> TimeOnly {
        TimeOnly { hour: h, minute: m }
    }
    fn secs(n: f64) -> TimeSpan {
        TimeSpan::from_secs_f64(n)
    }

    #[test]
    fn hourly_recurrence_pattern_get_next_occurrence_returns_correct_time() {
        for (s, h, e) in [
            (dt(2024, 5, 15, 12, 30), 1, dt(2024, 5, 15, 13, 30)),
            (dt(2024, 5, 15, 23, 30), 2, dt(2024, 5, 16, 1, 30)),
        ] {
            let r = Recurrence::Hourly { interval_hours: h };
            assert_eq!(r.next_occurrence(s, t(0, e.minute()), TimeZone::UTC), e);
        }
    }

    #[test]
    fn daily_recurrence_pattern_get_next_occurrence_returns_correct_day() {
        for (s, d, e) in [
            (dt(2024, 5, 15, 12, 30), 1, dt(2024, 5, 16, 12, 30)),
            (dt(2024, 5, 31, 12, 30), 2, dt(2024, 6, 2, 12, 30)),
        ] {
            let r = Recurrence::Daily { interval_days: d };
            assert_eq!(
                r.next_occurrence(s, t(e.hour(), e.minute()), TimeZone::UTC),
                e
            );
        }
    }

    fn weekly(interval: i32, months: AllowedMonths, days: AllowedDays, s: DateTime, e: DateTime) {
        let r = Recurrence::weekly_on(interval, months, days);
        assert_eq!(
            r.next_occurrence(s, t(e.hour(), e.minute()), TimeZone::UTC),
            e,
            "{s:?} {days:?}"
        );
    }

    #[test]
    fn weekly_recurrence_pattern_basic_patterns_returns_correct_day() {
        let all = AllowedMonths::ALL;
        weekly(
            1,
            all,
            AllowedDays::ALL,
            dt(2024, 5, 15, 12, 30),
            dt(2024, 5, 16, 12, 30),
        );
        weekly(
            1,
            all,
            AllowedDays::MONDAY,
            dt(2024, 5, 15, 12, 30),
            dt(2024, 5, 20, 12, 30),
        );
        weekly(
            2,
            all,
            AllowedDays::WEDNESDAY,
            dt(2024, 5, 15, 12, 30),
            dt(2024, 5, 29, 12, 30),
        );
    }

    #[test]
    fn weekly_recurrence_pattern_cross_boundaries_returns_correct_day() {
        let all = AllowedMonths::ALL;
        weekly(
            1,
            all,
            AllowedDays::MONDAY,
            dt(2024, 12, 30, 12, 30),
            dt(2025, 1, 6, 12, 30),
        );
        weekly(
            1,
            all,
            AllowedDays::SATURDAY,
            dt(2024, 5, 31, 12, 30),
            dt(2024, 6, 1, 12, 30),
        );
    }

    fn months(list: &[u32]) -> AllowedMonths {
        list.iter().fold(AllowedMonths::empty(), |a, m| {
            a | AllowedMonths::of_month(*m)
        })
    }

    #[test]
    fn weekly_recurrence_pattern_month_filtering_returns_correct_day() {
        let w = AllowedDays::WEDNESDAY;
        weekly(
            1,
            months(&[1, 2]),
            w,
            dt(2024, 5, 15, 12, 30),
            dt(2025, 1, 1, 12, 30),
        );
        weekly(
            1,
            months(&[1, 12]),
            w,
            dt(2024, 12, 15, 12, 30),
            dt(2024, 12, 18, 12, 30),
        );
        weekly(
            1,
            months(&[2, 4]),
            AllowedDays::SATURDAY,
            dt(2024, 1, 31, 12, 30),
            dt(2024, 2, 3, 12, 30),
        );
    }

    #[test]
    fn weekly_recurrence_pattern_dst_transitions_handles_correctly() {
        let ny = TimeZone::NEW_YORK;
        let sun = AllowedDays::SUNDAY;
        for (s, e) in [
            (dt(2024, 3, 9, 12, 0), dt(2024, 3, 10, 12, 0)),
            (dt(2024, 3, 10, 2, 30), dt(2024, 3, 17, 2, 30)), // during spring forward (invalid time)
            (dt(2024, 11, 2, 12, 0), dt(2024, 11, 3, 12, 0)),
            (dt(2024, 11, 3, 1, 30), dt(2024, 11, 10, 1, 30)), // during fall back (ambiguous time)
        ] {
            let r = Recurrence::weekly_on(1, AllowedMonths::ALL, sun);
            let after_utc = ny.to_utc(s);
            let result = r.next_occurrence(after_utc, t(e.hour(), e.minute()), ny);
            assert_eq!(ny.from_utc(result), e, "{s:?}");
        }
    }

    #[test]
    fn weekly_recurrence_pattern_days_of_week_handling_returns_correct_day() {
        let all = AllowedMonths::ALL;
        // AllowedDays.None: the constructor makes it All, so the next day at the time is the same day
        weekly(
            1,
            all,
            AllowedDays::empty(),
            dt(2024, 5, 29, 12, 0),
            dt(2024, 5, 29, 12, 30),
        );
        weekly(
            1,
            all,
            AllowedDays::WEDNESDAY | AllowedDays::FRIDAY,
            dt(2024, 5, 29, 12, 0),
            dt(2024, 5, 29, 12, 30),
        );
        weekly(
            1,
            all,
            AllowedDays::MONDAY | AllowedDays::FRIDAY,
            dt(2024, 5, 29, 12, 0),
            dt(2024, 5, 31, 12, 30),
        );
    }

    #[test]
    fn weekly_recurrence_pattern_week_spanning_months_returns_correct_day() {
        let m = months(&[1, 3]);
        for (s, e, next) in [
            (
                dt(2024, 1, 29, 12, 0),
                dt(2024, 1, 29, 12, 30),
                dt(2024, 1, 30, 12, 30),
            ),
            (
                dt(2024, 1, 31, 12, 0),
                dt(2024, 1, 31, 12, 30),
                dt(2024, 3, 1, 12, 30),
            ),
        ] {
            let r = Recurrence::weekly_on(1, m, AllowedDays::empty());
            let time = t(e.hour(), e.minute());
            let result = r.next_occurrence(s, time, TimeZone::UTC);
            assert_eq!(result, e);
            assert_eq!(r.next_occurrence(result, time, TimeZone::UTC), next);
        }
    }

    #[test]
    fn monthly_ordinal_recurrence_pattern_get_next_occurrence_returns_correct_date() {
        use OrdinalDayOccurrence::*;
        let ny = TimeZone::NEW_YORK;
        for (s, ord, e) in [
            (dt(2024, 11, 3, 1, 30), First, dt(2024, 11, 3, 1, 30)),
            (dt(2024, 3, 31, 0, 0), Last, dt(2024, 3, 31, 0, 0)),
            (dt(2024, 3, 31, 0, 0), Fifth, dt(2024, 3, 31, 0, 0)),
            (dt(2024, 4, 3, 0, 0), Fifth, dt(2024, 6, 30, 0, 0)), // June has 5 Sundays
        ] {
            let r = Recurrence::MonthlyOrdinal {
                ordinal: ord,
                day_of_week: DayOfWeek::Sunday,
                interval_months: 1,
            };
            let before_utc = ny.to_utc(s.add_days(-1.0));
            let result = r.next_occurrence(before_utc, t(s.hour(), s.minute()), ny);
            assert_eq!(ny.from_utc(result), e, "{s:?} {ord:?}");
        }
    }

    #[test]
    fn monthly_recurrence_pattern_dst_transitions_handles_correctly() {
        let ny = TimeZone::NEW_YORK;
        // 2:30 AM does not exist on the spring-forward day, so the occurrence moves to the next month.
        // 1:30 AM happens twice on the fall-back day; the first (EDT) pass.
        for (y, mo, d, h, mi, e) in [
            (2024, 3, 10, 2, 30, dt(2024, 4, 10, 6, 30)),
            (2024, 11, 3, 1, 30, dt(2024, 11, 3, 5, 30)),
        ] {
            let r = Recurrence::monthly_on(d as i32);
            let before_utc = ny.to_utc(dt(y, mo, d, 0, 0));
            assert_eq!(r.next_occurrence(before_utc, t(h, mi), ny), e);
        }
    }

    #[test]
    fn monthly_recurrence_month_without_day_skips_to_next_valid_month() {
        let r = Recurrence::monthly_on(31);
        assert_eq!(
            r.next_occurrence(dt(2024, 2, 1, 12, 0), t(12, 0), TimeZone::UTC),
            dt(2024, 3, 31, 12, 0)
        );
    }

    #[test]
    fn local_to_utc_ambiguous_time_handles_consistently() {
        let ny = TimeZone::NEW_YORK;
        let local = dt(2024, 11, 3, 1, 30);
        assert_eq!(ny.from_utc(ny.to_utc(local)), local);
    }

    // The test pattern: interval seconds, at most `max` occurrences, counting its calls.
    thread_local! { static CALLS: Cell<i32> = const { Cell::new(0) }; static MAX: Cell<i32> = const { Cell::new(i32::MAX) }; }
    fn test_pattern(after: DateTime, _: TimeOnly, _: TimeZone) -> DateTime {
        CALLS.with(|c| c.set(c.get() + 1));
        if CALLS.get() > MAX.get() {
            DateTime::MAX
        } else {
            after + secs(10.0)
        }
    }
    fn setup(max: i32) -> World {
        CALLS.set(0);
        MAX.set(max);
        let mut w = World::new();
        w.now = dt(2024, 1, 1, 12, 0);
        configure(&mut w);
        w
    }
    fn slice_to(w: &mut World, now: DateTime) {
        w.now = now;
        for _ in 0..2 {
            w.tick_count += 1000;
            muo_server::timer::slice(w);
        }
    }

    #[test]
    fn schedule_event_executes_callback_happy_path() {
        let mut w = setup(0);
        let called = Rc::new(Cell::new(false));
        let c = called.clone();
        let now = w.now;
        let e = schedule_event(&mut w, Start::Utc(now), move |_| c.set(true), None, None);
        assert_eq!(next_occurrence(&w, e), Some(now));
        slice_to(&mut w, now);
        assert!(called.get());
        assert_eq!(next_occurrence(&w, e), Some(DateTime::MAX));
        cancel(&mut w, e);
    }

    #[test]
    fn scheduler_orders_events_by_next_occurrence() {
        let mut w = setup(0);
        let order = Rc::new(RefCell::new(Vec::new()));
        let now = w.now;
        // added out of order
        for n in [3, 1, 2] {
            let (o, start) = (order.clone(), now.add_seconds(10.0 * n as f64));
            schedule_event_at(
                &mut w,
                Start::Utc(start),
                TimeOnly::from_date_time(start),
                move |_| o.borrow_mut().push(n),
                None,
                None,
            );
        }
        slice_to(&mut w, now.add_seconds(40.0));
        assert_eq!(*order.borrow(), vec![1, 2, 3]);
    }

    #[test]
    fn scheduler_handles_recurring_events() {
        let mut w = setup(3);
        let count = Rc::new(Cell::new(0));
        let c = count.clone();
        let now = w.now;
        let e = schedule_event_at(
            &mut w,
            Start::Utc(now),
            TimeOnly::from_date_time(now),
            move |_| c.set(c.get() + 1),
            Some(Recurrence::Custom(test_pattern)),
            None,
        );
        slice_to(&mut w, now.add_seconds(50.0));
        // one call per occurrence the pattern yields
        assert_eq!(count.get(), 3);
        cancel(&mut w, e);
    }

    #[test]
    fn scheduler_handles_exceptions_in_events() {
        let mut w = setup(0);
        let (failed, later) = (Rc::new(Cell::new(false)), Rc::new(Cell::new(false)));
        let now = w.now;
        let f = failed.clone();
        schedule_event(
            &mut w,
            Start::Utc(now.add_seconds(10.0)),
            move |_| {
                f.set(true);
                panic!("Test exception")
            },
            None,
            None,
        );
        let l = later.clone();
        schedule_event(
            &mut w,
            Start::Utc(now.add_seconds(20.0)),
            move |_| l.set(true),
            None,
            None,
        );
        slice_to(&mut w, now.add_seconds(30.0));
        assert!(failed.get() && later.get());
    }

    #[test]
    fn scheduler_removes_events_correctly() {
        let mut w = setup(0);
        let called = Rc::new(Cell::new(false));
        let c = called.clone();
        let now = w.now;
        let e = schedule_event(
            &mut w,
            Start::Utc(now.add_seconds(10.0)),
            move |_| c.set(true),
            None,
            None,
        );
        cancel(&mut w, e);
        slice_to(&mut w, now.add_seconds(20.0));
        assert!(!called.get());
    }

    #[test]
    fn scheduler_handles_events_with_end_dates() {
        let mut w = setup(i32::MAX);
        let count = Rc::new(Cell::new(0));
        let c = count.clone();
        let now = w.now;
        schedule_event_ending(
            &mut w,
            Start::Utc(now),
            TimeOnly::from_date_time(now),
            now.add_seconds(36.0),
            move |_| c.set(c.get() + 1),
            Some(Recurrence::Custom(test_pattern)),
        );
        slice_to(&mut w, now.add_seconds(50.0));
        // initial + 2 within the timeframe
        assert_eq!(count.get(), 3);
    }
}
