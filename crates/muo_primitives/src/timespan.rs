//! `TimeSpan` over ticks (100 ns), plus wrapping tick-count math.

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};
use std::str::FromStr;
use std::time::Duration;

use crate::parse::FormatError;

pub const TICKS_PER_MILLISECOND: i64 = 10_000;
pub const TICKS_PER_SECOND: i64 = 10_000_000;
pub const TICKS_PER_MINUTE: i64 = 600_000_000;
pub const TICKS_PER_HOUR: i64 = 36_000_000_000;
pub const TICKS_PER_DAY: i64 = 864_000_000_000;

/// A signed span of ticks. `+`/`-` saturate instead of overflowing.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, Debug)]
pub struct TimeSpan(i64);

impl TimeSpan {
    pub const ZERO: TimeSpan = TimeSpan(0);
    pub const MAX: TimeSpan = TimeSpan(i64::MAX);
    pub const MIN: TimeSpan = TimeSpan(i64::MIN);

    pub const fn from_ticks(ticks: i64) -> TimeSpan {
        TimeSpan(ticks)
    }

    pub const fn from_millis(ms: i64) -> TimeSpan {
        TimeSpan(ms.saturating_mul(TICKS_PER_MILLISECOND))
    }

    pub const fn from_secs(s: i64) -> TimeSpan {
        TimeSpan(s.saturating_mul(TICKS_PER_SECOND))
    }

    pub const fn from_mins(m: i64) -> TimeSpan {
        TimeSpan(m.saturating_mul(TICKS_PER_MINUTE))
    }

    pub const fn from_hours(h: i64) -> TimeSpan {
        TimeSpan(h.saturating_mul(TICKS_PER_HOUR))
    }

    pub const fn from_days(d: i64) -> TimeSpan {
        TimeSpan(d.saturating_mul(TICKS_PER_DAY))
    }

    // FromX(double) is `(long)(value * ticksPerUnit)`: truncation, not rounding; NaN is 0 here.
    pub fn from_millis_f64(ms: f64) -> TimeSpan {
        TimeSpan((ms * TICKS_PER_MILLISECOND as f64) as i64)
    }

    pub fn from_secs_f64(s: f64) -> TimeSpan {
        TimeSpan((s * TICKS_PER_SECOND as f64) as i64)
    }

    pub fn from_mins_f64(m: f64) -> TimeSpan {
        TimeSpan((m * TICKS_PER_MINUTE as f64) as i64)
    }

    pub fn from_hours_f64(h: f64) -> TimeSpan {
        TimeSpan((h * TICKS_PER_HOUR as f64) as i64)
    }

    pub fn from_days_f64(d: f64) -> TimeSpan {
        TimeSpan((d * TICKS_PER_DAY as f64) as i64)
    }

    pub const fn ticks(self) -> i64 {
        self.0
    }

    pub const fn days(self) -> i32 {
        (self.0 / TICKS_PER_DAY) as i32
    }

    pub const fn hours(self) -> i32 {
        ((self.0 / TICKS_PER_HOUR) % 24) as i32
    }

    pub const fn minutes(self) -> i32 {
        ((self.0 / TICKS_PER_MINUTE) % 60) as i32
    }

    pub const fn seconds(self) -> i32 {
        ((self.0 / TICKS_PER_SECOND) % 60) as i32
    }

    pub const fn milliseconds(self) -> i32 {
        ((self.0 / TICKS_PER_MILLISECOND) % 1000) as i32
    }

    pub fn total_days(self) -> f64 {
        self.0 as f64 / TICKS_PER_DAY as f64
    }

    pub fn total_hours(self) -> f64 {
        self.0 as f64 / TICKS_PER_HOUR as f64
    }

    pub fn total_minutes(self) -> f64 {
        self.0 as f64 / TICKS_PER_MINUTE as f64
    }

    pub fn total_seconds(self) -> f64 {
        self.0 as f64 / TICKS_PER_SECOND as f64
    }

    pub fn total_milliseconds(self) -> f64 {
        // this one clamps into the range of whole milliseconds
        (self.0 as f64 / TICKS_PER_MILLISECOND as f64)
            .clamp(-922_337_203_685_477.0, 922_337_203_685_477.0)
    }

    pub const fn abs(self) -> TimeSpan {
        TimeSpan(self.0.saturating_abs())
    }

    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }

    pub const fn checked_add(self, o: TimeSpan) -> Option<TimeSpan> {
        match self.0.checked_add(o.0) {
            Some(v) => Some(TimeSpan(v)),
            None => None,
        }
    }

    /// `None` for a negative span (std's `Duration` is unsigned).
    pub fn to_duration(self) -> Option<Duration> {
        u64::try_from(self.0).ok().map(|t| {
            Duration::new(
                t / TICKS_PER_SECOND as u64,
                (t % TICKS_PER_SECOND as u64) as u32 * 100,
            )
        })
    }

    pub fn from_duration(d: Duration) -> TimeSpan {
        let ticks = d
            .as_secs()
            .saturating_mul(TICKS_PER_SECOND as u64)
            .saturating_add(u64::from(d.subsec_nanos() / 100));
        TimeSpan(i64::try_from(ticks).unwrap_or(i64::MAX))
    }

    /// `TimeSpan.TryParse` (invariant culture): `[-][d.]hh:mm[:ss[.fffffff]]`, `d`, or `[-]d.hh:mm:ss`.
    pub fn try_parse(s: &str) -> Option<TimeSpan> {
        let s = s.trim();
        let (neg, s) = match s.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, s),
        };
        if s.is_empty()
            || !s
                .bytes()
                .all(|b| b.is_ascii_digit() || b == b':' || b == b'.')
        {
            return None;
        }
        let num = |t: &str| -> Option<i64> {
            if t.is_empty() || t.len() > 10 {
                None
            } else {
                t.parse().ok()
            }
        };
        let mut ticks: i128 = 0;
        if !s.contains(':') {
            ticks = i128::from(num(s)?) * i128::from(TICKS_PER_DAY);
        } else {
            let (days, rest) = match s.split_once('.') {
                Some((d, r)) if r.contains(':') && !d.contains(':') => (num(d)?, r),
                _ => (0, s),
            };
            let mut parts = rest.split(':');
            let h = num(parts.next()?)?;
            let m = num(parts.next()?)?;
            let sec_part = parts.next();
            if parts.next().is_some() || h > 23 || m > 59 {
                return None;
            }
            let (sec, frac) = match sec_part {
                None => (0, 0),
                Some(sp) => match sp.split_once('.') {
                    Some((a, f)) if !f.is_empty() && f.len() <= 7 && !f.contains('.') => {
                        (num(a)?, num(f)? * 10i64.pow(7 - f.len() as u32))
                    }
                    Some(_) => return None,
                    None => (num(sp)?, 0),
                },
            };
            if sec > 59 {
                return None;
            }
            ticks += i128::from(days) * i128::from(TICKS_PER_DAY);
            ticks += i128::from(
                h * TICKS_PER_HOUR + m * TICKS_PER_MINUTE + sec * TICKS_PER_SECOND + frac,
            );
        }
        if neg {
            ticks = -ticks;
        }
        i64::try_from(ticks).ok().map(TimeSpan)
    }

    /// `Utility.FormatTimeCompact`: `1d 2h 3m`, minutes at least (`0m`), seconds when asked.
    pub fn format_compact(self, show_seconds: bool) -> String {
        let mut sb = String::new();
        let part = |sb: &mut String, v: i32, unit: char, always_if_started: bool| {
            if !sb.is_empty() && always_if_started {
                sb.push(' ');
                sb.push_str(&format!("{v}{unit}"));
            } else if v >= 1 {
                sb.push_str(&format!("{v}{unit}"));
            }
        };
        part(&mut sb, self.days(), 'd', false);
        part(&mut sb, self.hours(), 'h', true);
        part(&mut sb, self.minutes(), 'm', true);
        if show_seconds {
            part(&mut sb, self.seconds(), 's', true);
        } else if sb.is_empty() {
            sb.push_str("0m");
        }
        sb
    }
}

/// The constant ("c") format: `[-][d.]hh:mm:ss[.fffffff]`.
impl fmt::Display for TimeSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = i128::from(self.0).abs();
        if self.0 < 0 {
            f.write_str("-")?;
        }
        let day = t / i128::from(TICKS_PER_DAY);
        if day != 0 {
            write!(f, "{day}.")?;
        }
        let rem = t % i128::from(TICKS_PER_DAY);
        write!(
            f,
            "{:02}:{:02}:{:02}",
            rem / i128::from(TICKS_PER_HOUR),
            rem / i128::from(TICKS_PER_MINUTE) % 60,
            rem / i128::from(TICKS_PER_SECOND) % 60
        )?;
        let frac = rem % i128::from(TICKS_PER_SECOND);
        if frac != 0 {
            write!(f, ".{frac:07}")?;
        }
        Ok(())
    }
}

impl FromStr for TimeSpan {
    type Err = FormatError;
    fn from_str(s: &str) -> Result<TimeSpan, FormatError> {
        TimeSpan::try_parse(s).ok_or(FormatError)
    }
}

impl Add for TimeSpan {
    type Output = TimeSpan;
    fn add(self, o: TimeSpan) -> TimeSpan {
        TimeSpan(self.0.saturating_add(o.0))
    }
}

impl Sub for TimeSpan {
    type Output = TimeSpan;
    fn sub(self, o: TimeSpan) -> TimeSpan {
        TimeSpan(self.0.saturating_sub(o.0))
    }
}

impl AddAssign for TimeSpan {
    fn add_assign(&mut self, o: TimeSpan) {
        *self = *self + o;
    }
}

impl SubAssign for TimeSpan {
    fn sub_assign(&mut self, o: TimeSpan) {
        *self = *self - o;
    }
}

impl Neg for TimeSpan {
    type Output = TimeSpan;
    fn neg(self) -> TimeSpan {
        TimeSpan(self.0.saturating_neg())
    }
}

impl Mul<f64> for TimeSpan {
    type Output = TimeSpan;
    fn mul(self, f: f64) -> TimeSpan {
        TimeSpan((self.0 as f64 * f) as i64)
    }
}

impl Div<f64> for TimeSpan {
    type Output = TimeSpan;
    fn div(self, f: f64) -> TimeSpan {
        TimeSpan((self.0 as f64 / f) as i64)
    }
}

impl Sum for TimeSpan {
    fn sum<I: Iterator<Item = TimeSpan>>(iter: I) -> TimeSpan {
        iter.fold(TimeSpan::ZERO, |a, b| a + b)
    }
}

/// Wrapping tick-count math: tick readings are huge or negative, so compare
/// by subtraction, never directly, and keep no zero/sign sentinels.
pub mod tick_count {
    /// `later - earlier`, wrapping: the elapsed ticks between two readings.
    pub const fn elapsed(later: i64, earlier: i64) -> i64 {
        later.wrapping_sub(earlier)
    }

    /// `now - deadline >= 0`: the deadline has been reached (`now >= deadline` that survives wraparound).
    pub const fn reached(now: i64, deadline: i64) -> bool {
        now.wrapping_sub(deadline) >= 0
    }

    /// `a - b < 0`: reading `a` is before `b`.
    pub const fn before(a: i64, b: i64) -> bool {
        a.wrapping_sub(b) < 0
    }

    /// `start + interval`, wrapping.
    pub const fn deadline(start: i64, interval: i64) -> i64 {
        start.wrapping_add(interval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_and_totals() {
        let t = TimeSpan::from_days(1)
            + TimeSpan::from_hours(2)
            + TimeSpan::from_mins(3)
            + TimeSpan::from_secs(4)
            + TimeSpan::from_millis(5);
        assert_eq!(
            (
                t.days(),
                t.hours(),
                t.minutes(),
                t.seconds(),
                t.milliseconds()
            ),
            (1, 2, 3, 4, 5)
        );
        assert_eq!(TimeSpan::from_secs(90).total_minutes(), 1.5);
        assert_eq!(TimeSpan::from_millis_f64(1.5).ticks(), 15_000);
        assert_eq!(TimeSpan::from_secs_f64(-0.5).total_milliseconds(), -500.0);
    }

    #[test]
    fn display_and_parse() {
        assert_eq!(TimeSpan::from_secs(3661).to_string(), "01:01:01");
        assert_eq!(
            (TimeSpan::from_days(2) + TimeSpan::from_millis(1)).to_string(),
            "2.00:00:00.0010000"
        );
        assert_eq!((-TimeSpan::from_mins(5)).to_string(), "-00:05:00");
        for s in ["01:01:01", "2.00:00:00.0010000", "-00:05:00", "00:00:00"] {
            assert_eq!(s.parse::<TimeSpan>().unwrap().to_string(), s);
        }
        assert_eq!("3".parse(), Ok(TimeSpan::from_days(3)));
        assert_eq!("12:30".parse(), Ok(TimeSpan::from_mins(750)));
        assert_eq!("0:0:1.5".parse(), Ok(TimeSpan::from_millis(1500)));
        for bad in ["", "abc", "25:00:00", "1:61", "1:2:3:4", "1:00:00.", ":"] {
            assert!(bad.parse::<TimeSpan>().is_err(), "{bad}");
        }
    }

    #[test]
    fn duration_round_trip() {
        let d = Duration::new(5, 123_456_700);
        assert_eq!(TimeSpan::from_duration(d).to_duration(), Some(d));
        assert_eq!((-TimeSpan::from_secs(1)).to_duration(), None);
    }

    #[test]
    fn compact() {
        let t = TimeSpan::from_days(1) + TimeSpan::from_mins(3) + TimeSpan::from_secs(5);
        assert_eq!(t.format_compact(false), "1d 0h 3m");
        assert_eq!(t.format_compact(true), "1d 0h 3m 5s");
        assert_eq!(TimeSpan::from_secs(30).format_compact(false), "0m");
        assert_eq!(TimeSpan::from_secs(30).format_compact(true), "30s");
        assert_eq!(TimeSpan::from_hours(2).format_compact(false), "2h 0m"); // minutes follow once anything printed
    }

    #[test]
    fn saturating_ops() {
        assert_eq!(TimeSpan::MAX + TimeSpan::from_secs(1), TimeSpan::MAX);
        assert_eq!(TimeSpan::MAX.checked_add(TimeSpan::from_ticks(1)), None);
    }

    #[test]
    fn tick_counts_wrap() {
        let start = i64::MAX - 5;
        let deadline = tick_count::deadline(start, 10); // wraps negative
        assert!(deadline < 0);
        assert!(!tick_count::reached(start, deadline));
        assert!(tick_count::reached(start.wrapping_add(10), deadline));
        assert!(tick_count::before(start, deadline));
        assert_eq!(tick_count::elapsed(deadline, start), 10);
    }
}
