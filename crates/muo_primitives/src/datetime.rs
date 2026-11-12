//! `DateTime` over ticks: 100 ns units since 0001-01-01T00:00:00, proleptic Gregorian.
//!
//! // shortcut: no `DateTimeKind` and no time zones. The server stores and compares UTC; local-time
//! conversion belongs to whoever has a time zone database.
//! `+`/`-` with a [`TimeSpan`] saturate at `MIN`/`MAX` instead of failing; use `checked_add` to tell.

use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::timespan::{
    TICKS_PER_DAY, TICKS_PER_HOUR, TICKS_PER_MILLISECOND, TICKS_PER_MINUTE, TICKS_PER_SECOND,
    TimeSpan,
};

/// Ticks of 1970-01-01T00:00:00 (`DateTime.UnixEpoch`).
pub const UNIX_EPOCH_TICKS: i64 = 621_355_968_000_000_000;
/// Ticks of 9999-12-31T23:59:59.9999999.
pub const MAX_TICKS: i64 = 3_155_378_975_999_999_999;
const DAYS_TO_UNIX_EPOCH: i64 = 719_162;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default, Debug)]
pub struct DateTime(i64);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum DayOfWeek {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

pub const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// `DateTime.DaysInMonth`; 0 for a month outside 1..=12.
pub const fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

// Howard Hinnant's civil-date algorithms, on days since 1970-01-01.
const fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

const fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + if m <= 2 { 1 } else { 0 }, m, d)
}

impl DateTime {
    pub const MIN: DateTime = DateTime(0);
    pub const MAX: DateTime = DateTime(MAX_TICKS);
    pub const UNIX_EPOCH: DateTime = DateTime(UNIX_EPOCH_TICKS);

    /// `None` when `ticks` is outside 0..=`MAX_TICKS`.
    pub const fn try_from_ticks(ticks: i64) -> Option<DateTime> {
        if ticks >= 0 && ticks <= MAX_TICKS {
            Some(DateTime(ticks))
        } else {
            None
        }
    }

    /// Clamps into range.
    pub const fn from_ticks(ticks: i64) -> DateTime {
        if ticks < 0 {
            DateTime::MIN
        } else if ticks > MAX_TICKS {
            DateTime::MAX
        } else {
            DateTime(ticks)
        }
    }

    pub const fn ticks(self) -> i64 {
        self.0
    }

    /// `new DateTime(y, m, d)`; `None` for an invalid date.
    pub const fn from_ymd(year: i32, month: u32, day: u32) -> Option<DateTime> {
        DateTime::from_ymd_hms_milli(year, month, day, 0, 0, 0, 0)
    }

    pub const fn from_ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> Option<DateTime> {
        DateTime::from_ymd_hms_milli(year, month, day, hour, min, sec, 0)
    }

    pub const fn from_ymd_hms_milli(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        milli: u32,
    ) -> Option<DateTime> {
        if year < 1
            || year > 9999
            || month < 1
            || month > 12
            || day < 1
            || day > days_in_month(year, month)
        {
            return None;
        }
        if hour > 23 || min > 59 || sec > 59 || milli > 999 {
            return None;
        }
        let days = days_from_civil(year as i64, month as i64, day as i64) + DAYS_TO_UNIX_EPOCH;
        Some(DateTime(
            days * TICKS_PER_DAY
                + hour as i64 * TICKS_PER_HOUR
                + min as i64 * TICKS_PER_MINUTE
                + sec as i64 * TICKS_PER_SECOND
                + milli as i64 * TICKS_PER_MILLISECOND,
        ))
    }

    /// `DateTime.UtcNow`
    pub fn utc_now() -> DateTime {
        let d = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        DateTime::from_ticks(UNIX_EPOCH_TICKS.saturating_add(TimeSpan::from_duration(d).ticks()))
    }

    /// `DateTimeOffset.FromUnixTimeSeconds(..).UtcDateTime`; `None` outside year 1..9999.
    pub const fn from_unix_seconds(secs: i64) -> Option<DateTime> {
        match secs.checked_mul(TICKS_PER_SECOND) {
            Some(t) => match t.checked_add(UNIX_EPOCH_TICKS) {
                Some(t) => DateTime::try_from_ticks(t),
                None => None,
            },
            None => None,
        }
    }

    pub const fn from_unix_millis(ms: i64) -> Option<DateTime> {
        match ms.checked_mul(TICKS_PER_MILLISECOND) {
            Some(t) => match t.checked_add(UNIX_EPOCH_TICKS) {
                Some(t) => DateTime::try_from_ticks(t),
                None => None,
            },
            None => None,
        }
    }

    /// `ToUnixTimeSeconds()`: floors for times before 1970.
    pub const fn to_unix_seconds(self) -> i64 {
        (self.0 - UNIX_EPOCH_TICKS).div_euclid(TICKS_PER_SECOND)
    }

    pub const fn to_unix_millis(self) -> i64 {
        (self.0 - UNIX_EPOCH_TICKS).div_euclid(TICKS_PER_MILLISECOND)
    }

    const fn total_days(self) -> i64 {
        self.0 / TICKS_PER_DAY
    }

    /// (year, month, day)
    pub const fn ymd(self) -> (i32, u32, u32) {
        let (y, m, d) = civil_from_days(self.total_days() - DAYS_TO_UNIX_EPOCH);
        (y as i32, m as u32, d as u32)
    }

    pub const fn year(self) -> i32 {
        self.ymd().0
    }

    pub const fn month(self) -> u32 {
        self.ymd().1
    }

    pub const fn day(self) -> u32 {
        self.ymd().2
    }

    pub const fn hour(self) -> u32 {
        (self.0 / TICKS_PER_HOUR % 24) as u32
    }

    pub const fn minute(self) -> u32 {
        (self.0 / TICKS_PER_MINUTE % 60) as u32
    }

    pub const fn second(self) -> u32 {
        (self.0 / TICKS_PER_SECOND % 60) as u32
    }

    pub const fn millisecond(self) -> u32 {
        (self.0 / TICKS_PER_MILLISECOND % 1000) as u32
    }

    pub const fn day_of_week(self) -> DayOfWeek {
        // 0001-01-01 was a Monday
        match (self.total_days() + 1) % 7 {
            0 => DayOfWeek::Sunday,
            1 => DayOfWeek::Monday,
            2 => DayOfWeek::Tuesday,
            3 => DayOfWeek::Wednesday,
            4 => DayOfWeek::Thursday,
            5 => DayOfWeek::Friday,
            _ => DayOfWeek::Saturday,
        }
    }

    pub const fn day_of_year(self) -> u32 {
        let (y, _, _) = self.ymd();
        (self.total_days() - (DateTime::from_ymd_unchecked_jan1(y).total_days()) + 1) as u32
    }

    const fn from_ymd_unchecked_jan1(y: i32) -> DateTime {
        DateTime((days_from_civil(y as i64, 1, 1) + DAYS_TO_UNIX_EPOCH) * TICKS_PER_DAY)
    }

    /// `.Date`: midnight of the same day.
    pub const fn date(self) -> DateTime {
        DateTime(self.0 - self.0 % TICKS_PER_DAY)
    }

    pub const fn time_of_day(self) -> TimeSpan {
        TimeSpan::from_ticks(self.0 % TICKS_PER_DAY)
    }

    pub const fn checked_add_ticks(self, ticks: i64) -> Option<DateTime> {
        match self.0.checked_add(ticks) {
            Some(t) => DateTime::try_from_ticks(t),
            None => None,
        }
    }

    pub const fn checked_add(self, span: TimeSpan) -> Option<DateTime> {
        self.checked_add_ticks(span.ticks())
    }

    /// `AddDays(double)` etc. truncate toward zero to ticks.
    // shortcut: sub-millisecond fractions are truncated, not rounded; callers pass whole units.
    pub fn add_days(self, d: f64) -> DateTime {
        self + TimeSpan::from_days_f64(d)
    }

    pub fn add_hours(self, h: f64) -> DateTime {
        self + TimeSpan::from_hours_f64(h)
    }

    pub fn add_minutes(self, m: f64) -> DateTime {
        self + TimeSpan::from_mins_f64(m)
    }

    pub fn add_seconds(self, s: f64) -> DateTime {
        self + TimeSpan::from_secs_f64(s)
    }

    pub fn add_milliseconds(self, ms: f64) -> DateTime {
        self + TimeSpan::from_millis_f64(ms)
    }

    /// `AddMonths`: the day clamps to the last day of the target month. `None` if out of range.
    pub fn checked_add_months(self, months: i32) -> Option<DateTime> {
        if !(-120_000..=120_000).contains(&months) {
            return None;
        }
        let (y, m, d) = self.ymd();
        let total = i64::from(y) * 12 + i64::from(m) - 1 + i64::from(months);
        let (ny, nm) = (total.div_euclid(12), total.rem_euclid(12) as u32 + 1);
        if !(1..=9999).contains(&ny) {
            return None;
        }
        let ny = ny as i32;
        let nd = d.min(days_in_month(ny, nm));
        let days =
            days_from_civil(i64::from(ny), i64::from(nm), i64::from(nd)) + DAYS_TO_UNIX_EPOCH;
        DateTime::try_from_ticks(days * TICKS_PER_DAY + self.0 % TICKS_PER_DAY)
    }

    /// `AddYears` (Feb 29 clamps to Feb 28 in a non-leap year).
    pub fn checked_add_years(self, years: i32) -> Option<DateTime> {
        if !(-10_000..=10_000).contains(&years) {
            return None;
        }
        self.checked_add_months(years * 12)
    }

    /// `self - earlier`
    pub const fn since(self, earlier: DateTime) -> TimeSpan {
        TimeSpan::from_ticks(self.0 - earlier.0)
    }

    /// `Utility.ToTimeStamp`: `yyyy-MM-dd-HH-mm-ss`.
    pub fn to_timestamp(self) -> String {
        let (y, mo, d) = self.ymd();
        format!(
            "{y:04}-{mo:02}-{d:02}-{:02}-{:02}-{:02}",
            self.hour(),
            self.minute(),
            self.second()
        )
    }

    /// The round-trip ("o") format of a UTC value: `yyyy-MM-ddTHH:mm:ss.fffffffZ`.
    pub fn to_iso_string(self) -> String {
        let (y, mo, d) = self.ymd();
        format!(
            "{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}.{:07}Z",
            self.hour(),
            self.minute(),
            self.second(),
            self.0 % TICKS_PER_SECOND
        )
    }

    /// `XmlConvert.ToString(dt, Utc)` and what System.Text.Json writes for a UTC `DateTime`: `o` with the
    /// trailing zeros of the fraction trimmed (`2026-11-05T09:30:15.25Z`, `2026-11-05T09:30:15Z`).
    pub fn to_json_string(self) -> String {
        let iso = self.to_iso_string(); // yyyy-MM-ddTHH:mm:ss.fffffffZ
        let (head, frac) = iso[..iso.len() - 1].split_at(19);
        let frac = frac.trim_end_matches('0');
        format!("{head}{}Z", if frac == "." { "" } else { frac })
    }

    /// `ToString("yyyy-MM-ddTHH:mm:ssZ")`.
    pub fn to_iso_seconds_string(self) -> String {
        let (y, mo, d) = self.ymd();
        format!(
            "{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}Z",
            self.hour(),
            self.minute(),
            self.second()
        )
    }

    /// `JsonSerializer.Deserialize<DateTime>`: strictly `YYYY-MM-DD[Thh:mm[:ss[.f{0,16}]][Z|+hh[:mm]|-hh[:mm]]]`
    /// (no whitespace, upper case `T`/`Z`, a zone only after a time). Beyond 7 fraction digits are dropped.
    /// A zone offset gives the UTC instant here; to convert it to the local zone, see [`Self::parse_json_with_local`].
    pub fn parse_json(s: &str) -> Option<DateTime> {
        DateTime::parse_json_with_local(s, |_| TimeSpan::ZERO)
    }

    /// [`Self::parse_json`] where an explicit `+hh:mm`/`-hh:mm` offset gives a local-kind value (the UTC instant plus
    /// `local_offset(instant)`); `Z` and no zone are left as written.
    pub fn parse_json_with_local(
        s: &str,
        local_offset: impl Fn(DateTime) -> TimeSpan,
    ) -> Option<DateTime> {
        let mut zoned = false;
        let b = s.as_bytes();
        let num = |from: usize, n: usize| -> Option<u32> {
            let d = b.get(from..from + n)?;
            d.iter()
                .all(u8::is_ascii_digit)
                .then(|| d.iter().fold(0, |a, c| a * 10 + u32::from(c - b'0')))
        };
        if b.len() < 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let (y, mo, d) = (num(0, 4)?, num(5, 2)?, num(8, 2)?);
        let (mut h, mut mi, mut sec, mut frac, mut off) = (0, 0, 0, 0i64, 0i64);
        if b.len() > 10 {
            if b[10] != b'T' || b.len() < 16 || b[13] != b':' {
                return None;
            }
            (h, mi) = (num(11, 2)?, num(14, 2)?);
            let mut i = 16;
            if b.get(i) == Some(&b':') {
                sec = num(i + 1, 2)?;
                i += 3;
                if b.get(i) == Some(&b'.') {
                    let digits = b[i + 1..].iter().take_while(|c| c.is_ascii_digit()).count();
                    if digits > 16 || (digits == 0 && i + 1 == b.len()) {
                        return None;
                    }
                    let f7: String = s[i + 1..i + 1 + digits.min(7)].into();
                    frac = f7.parse::<i64>().unwrap_or(0) * 10i64.pow(7 - f7.len() as u32);
                    i += 1 + digits;
                }
            }
            if i < b.len() && &s[i..] != "Z" {
                let sign = match b[i] {
                    b'+' => 1,
                    b'-' => -1,
                    _ => return None,
                };
                let (oh, om) = match b.len() - i {
                    3 => (num(i + 1, 2)?, 0),
                    6 if b[i + 3] == b':' => (num(i + 1, 2)?, num(i + 4, 2)?),
                    _ => return None,
                };
                if oh > 23 || om > 59 {
                    return None;
                }
                off = sign * i64::from(oh * 60 + om);
                zoned = true;
            }
        }
        if mi > 59 || sec > 59 {
            return None;
        }
        let base = DateTime::from_ymd_hms(
            i32::try_from(y).ok().filter(|&y| y >= 1)?,
            mo,
            d,
            h,
            mi,
            sec,
        )?;
        let utc = base.checked_add_ticks(frac - off * TICKS_PER_MINUTE)?;
        if zoned {
            utc.checked_add(local_offset(utc))
        } else {
            Some(utc)
        }
    }

    /// `XmlConvert.ToDateTime(s, Utc)` / ISO 8601: `yyyy-MM-dd[THH:mm:ss[.f+]][Z|+hh:mm|-hh:mm]` as UTC.
    pub fn parse_iso(s: &str) -> Option<DateTime> {
        let s = s.trim();
        let num = |t: &str| -> Option<u32> {
            if t.is_empty() || !t.bytes().all(|b| b.is_ascii_digit()) {
                None
            } else {
                t.parse().ok()
            }
        };
        let (date, rest) = match s.split_once('T') {
            Some((d, r)) => (d, Some(r)),
            None => (s, None),
        };
        let mut dp = date.splitn(3, '-');
        let y = num(dp.next()?)?;
        let mo = num(dp.next()?)?;
        let d = num(dp.next()?)?;
        let (mut h, mut mi, mut sec, mut frac, mut offset_min) = (0, 0, 0, 0i64, 0i64);
        if let Some(rest) = rest {
            let (time, zone) = match rest.find(['Z', '+', '-']) {
                Some(i) => (&rest[..i], &rest[i..]),
                None => (rest, ""),
            };
            let mut tp = time.splitn(3, ':');
            h = num(tp.next()?)?;
            mi = num(tp.next()?)?;
            let sp = tp.next()?;
            let (sw, f) = sp.split_once('.').unwrap_or((sp, ""));
            sec = num(sw)?;
            if !f.is_empty() {
                let f7: String = f.chars().take(7).collect();
                frac = i64::from(num(&f7)?) * 10i64.pow(7 - f7.len() as u32);
            }
            if let Some(z) = zone.strip_prefix(['+', '-']) {
                let (hh, mm) = z.split_once(':')?;
                let off = i64::from(num(hh)?) * 60 + i64::from(num(mm)?);
                offset_min = if zone.starts_with('-') { -off } else { off };
            } else if !zone.is_empty() && zone != "Z" {
                return None;
            }
        }
        let base = DateTime::from_ymd_hms(i32::try_from(y).ok()?, mo, d, h, mi, sec)?;
        base.checked_add_ticks(frac - offset_min * TICKS_PER_MINUTE)
    }
}

/// Invariant culture general format: `MM/dd/yyyy HH:mm:ss`.
impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (y, mo, d) = self.ymd();
        write!(
            f,
            "{mo:02}/{d:02}/{y:04} {:02}:{:02}:{:02}",
            self.hour(),
            self.minute(),
            self.second()
        )
    }
}

impl Add<TimeSpan> for DateTime {
    type Output = DateTime;
    fn add(self, span: TimeSpan) -> DateTime {
        DateTime::from_ticks(self.0.saturating_add(span.ticks()))
    }
}

impl Sub<TimeSpan> for DateTime {
    type Output = DateTime;
    fn sub(self, span: TimeSpan) -> DateTime {
        DateTime::from_ticks(self.0.saturating_sub(span.ticks()))
    }
}

impl Sub for DateTime {
    type Output = TimeSpan;
    fn sub(self, o: DateTime) -> TimeSpan {
        self.since(o)
    }
}

impl AddAssign<TimeSpan> for DateTime {
    fn add_assign(&mut self, span: TimeSpan) {
        *self = *self + span;
    }
}

impl SubAssign<TimeSpan> for DateTime {
    fn sub_assign(&mut self, span: TimeSpan) {
        *self = *self - span;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(y: i32, m: u32, d: u32) -> DateTime {
        DateTime::from_ymd(y, m, d).unwrap()
    }

    #[test]
    fn known_ticks() {
        assert_eq!(dt(1, 1, 1).ticks(), 0);
        assert_eq!(dt(1970, 1, 1).ticks(), UNIX_EPOCH_TICKS);
        assert_eq!(
            DateTime::from_ymd_hms_milli(9999, 12, 31, 23, 59, 59, 999)
                .unwrap()
                .ticks(),
            MAX_TICKS - 9999
        );
        assert_eq!(dt(2000, 1, 1).ticks(), 630_822_816_000_000_000);
        assert_eq!(dt(2024, 2, 29).to_unix_seconds(), 1_709_164_800);
    }

    #[test]
    fn fields() {
        let t = DateTime::from_ymd_hms_milli(2026, 11, 5, 9, 30, 15, 250).unwrap();
        assert_eq!(t.ymd(), (2026, 11, 5));
        assert_eq!(
            (t.hour(), t.minute(), t.second(), t.millisecond()),
            (9, 30, 15, 250)
        );
        assert_eq!(t.day_of_week(), DayOfWeek::Thursday);
        assert_eq!(t.day_of_year(), 309);
        assert_eq!(dt(1, 1, 1).day_of_week(), DayOfWeek::Monday);
        assert_eq!(dt(1970, 1, 1).day_of_week(), DayOfWeek::Thursday);
        assert_eq!(t.date(), dt(2026, 11, 5));
        assert_eq!(
            t.time_of_day().total_seconds(),
            9.0 * 3600.0 + 30.0 * 60.0 + 15.25
        );
        assert_eq!(t.to_timestamp(), "2026-11-05-09-30-15");
        assert_eq!(t.to_string(), "11/05/2026 09:30:15");
        assert_eq!(t.to_iso_string(), "2026-11-05T09:30:15.2500000Z");
    }

    #[test]
    fn roundtrip_every_day_of_a_few_years() {
        for y in [1, 4, 100, 400, 1900, 1969, 1970, 2000, 2024, 2100, 9999] {
            for m in 1..=12 {
                for d in 1..=days_in_month(y, m) {
                    let t = dt(y, m, d);
                    assert_eq!(t.ymd(), (y, m, d));
                }
            }
        }
        assert!(DateTime::from_ymd(2023, 2, 29).is_none() && DateTime::from_ymd(0, 1, 1).is_none());
    }

    #[test]
    fn unix_conversion() {
        let t = DateTime::from_unix_seconds(1_700_000_000).unwrap();
        assert_eq!(t.ymd(), (2023, 11, 14));
        assert_eq!(t.to_unix_seconds(), 1_700_000_000);
        assert_eq!(
            DateTime::from_unix_millis(-1).unwrap().to_unix_seconds(),
            -1
        );
        assert_eq!(DateTime::from_unix_millis(-1).unwrap().to_unix_millis(), -1);
        assert_eq!(DateTime::from_unix_seconds(i64::MAX), None);
    }

    #[test]
    fn add_months_and_years_clamp() {
        assert_eq!(dt(2024, 1, 31).checked_add_months(1), Some(dt(2024, 2, 29)));
        assert_eq!(dt(2023, 1, 31).checked_add_months(1), Some(dt(2023, 2, 28)));
        assert_eq!(
            dt(2023, 3, 31).checked_add_months(-1),
            Some(dt(2023, 2, 28))
        );
        assert_eq!(
            dt(2023, 12, 15).checked_add_months(2),
            Some(dt(2024, 2, 15))
        );
        assert_eq!(dt(2024, 2, 29).checked_add_years(1), Some(dt(2025, 2, 28)));
        assert_eq!(dt(2024, 2, 29).checked_add_years(4), Some(dt(2028, 2, 29)));
        assert_eq!(dt(9999, 12, 1).checked_add_months(1), None);
        assert_eq!(dt(1, 1, 1).checked_add_months(-1), None);
        let t = DateTime::from_ymd_hms(2024, 1, 31, 13, 14, 15).unwrap();
        assert_eq!(
            t.checked_add_months(1).unwrap().time_of_day(),
            t.time_of_day()
        );
    }

    #[test]
    fn arithmetic() {
        let a = dt(2026, 1, 1);
        assert_eq!(a + TimeSpan::from_days(31), dt(2026, 2, 1));
        assert_eq!(dt(2026, 2, 1) - a, TimeSpan::from_days(31));
        assert_eq!(a.add_days(1.5).hour(), 12);
        assert_eq!(DateTime::MAX + TimeSpan::from_days(1), DateTime::MAX);
        assert_eq!(a.checked_add(TimeSpan::MAX), None);
        assert_eq!(DateTime::MIN - TimeSpan::from_days(1), DateTime::MIN);
    }

    #[test]
    fn parse_iso() {
        assert_eq!(DateTime::parse_iso("2026-11-05"), Some(dt(2026, 11, 5)));
        assert_eq!(
            DateTime::parse_iso("2026-11-05T09:30:15.25Z"),
            DateTime::from_ymd_hms_milli(2026, 11, 5, 9, 30, 15, 250)
        );
        assert_eq!(
            DateTime::parse_iso("2026-11-05T09:30:00+02:00"),
            DateTime::from_ymd_hms(2026, 11, 5, 7, 30, 0)
        );
        assert_eq!(
            DateTime::parse_iso("2026-11-05T09:30:00-01:30"),
            DateTime::from_ymd_hms(2026, 11, 5, 11, 0, 0)
        );
        assert_eq!(DateTime::parse_iso("garbage"), None);
        assert_eq!(DateTime::parse_iso("2026-13-05"), None);
    }
}

#[cfg(test)]
mod json_tests {
    use super::*;

    /// Expected UTC ticks of `JsonSerializer.Deserialize<DateTime>` (None = rejected).
    #[test]
    fn json_offset_becomes_a_local_kind_value() {
        let plus2 = |_: DateTime| TimeSpan::from_secs(7200);
        let t = |s: &str| DateTime::parse_json_with_local(s, plus2).map(DateTime::ticks);
        // 10:00+01:00 is 09:00 UTC, which is 11:00 local at +2
        assert_eq!(
            t("2024-05-06T10:00:00+01:00"),
            DateTime::parse_json("2024-05-06T11:00:00").map(DateTime::ticks)
        );
        // Z and no zone stay as written
        assert_eq!(
            t("2024-05-06T10:00:00Z"),
            DateTime::parse_json("2024-05-06T10:00:00Z").map(DateTime::ticks)
        );
        assert_eq!(
            t("2024-05-06T10:00:00"),
            DateTime::parse_json("2024-05-06T10:00:00").map(DateTime::ticks)
        );
    }

    #[test]
    fn parse_json_matches_net() {
        let t = |s: &str| DateTime::parse_json(s).map(DateTime::ticks);
        for (s, want) in [
            ("2026-11-05T09:30:15Z", Some(639294678150000000)),
            ("2026-11-05T09:30:15", Some(639294678150000000)),
            ("2026-11-05", Some(639294336000000000)),
            ("2026-11-05T09:30:15.1Z", Some(639294678151000000)),
            ("2026-11-05T09:30:15.12345678Z", Some(639294678151234567)),
            (
                "2026-11-05T09:30:15.1234567890123456Z",
                Some(639294678151234567),
            ),
            ("2026-11-05T09:30:15.99999999Z", Some(639294678159999999)),
            ("2026-11-05T09:30:15.12345678901234567Z", None),
            ("2026-11-05T09:30Z", Some(639294678000000000)),
            ("2026-11-05T09:30:15.Z", Some(639294678150000000)),
            ("2026-11-05T09:30:15.", None),
            ("2026-11-05T09:30:15+02:00", Some(639294606150000000)),
            ("2026-11-05T09:30:15+02", Some(639294606150000000)),
            ("2026-11-05T09:30:15-00:00", Some(639294678150000000)),
            ("2026-11-05T09:30:15+14:00", Some(639294174150000000)),
            ("2026-11-05T09:30:15+24:00", None),
            ("2026-11-05T09:30:15+02:60", None),
            ("2026-11-05T09:30:15+0200", None),
            ("2026-11-05T09:30:15+2:00", None),
            (" 2026-11-05T09:30:15Z", None),
            ("2026-11-05T09:30:15Z ", None),
            ("2026-11-05t09:30:15Z", None),
            ("2026-11-05T09:30:15z", None),
            ("2026-11-05T24:00:00Z", None),
            ("2026-11-05T09:30:60Z", None),
            ("2026-11-05T09:30:15,5Z", None),
            ("2026-02-30T09:30:15Z", None),
            ("2025-02-29", None),
            ("2024-02-29", Some(638447616000000000)),
            ("0001-01-01T00:00:00Z", Some(0)),
            ("0000-01-01", None),
            ("9999-12-31T23:59:59.9999999Z", Some(3155378975999999999)),
            ("2026-11-05 09:30:15Z", None),
            ("2026-11-05T9:30:15Z", None),
            ("2026-11-5T09:30:15Z", None),
            ("2026-11-05T09", None),
            ("2026-11-05T09:30:1", None),
            ("2026-11-05Z", None),
            ("2026-11-05+02:00", None),
            ("20261105T093015Z", None),
            ("", None),
        ] {
            assert_eq!(t(s), want, "{s:?}");
        }
    }

    #[test]
    fn json_string_trims_the_fraction() {
        let d = DateTime::from_ymd_hms_milli(2026, 11, 5, 9, 30, 15, 250).unwrap();
        assert_eq!(d.to_json_string(), "2026-11-05T09:30:15.25Z");
        assert_eq!(d.to_iso_string(), "2026-11-05T09:30:15.2500000Z");
        let s = DateTime::from_ymd_hms(2026, 11, 5, 9, 30, 15).unwrap();
        assert_eq!(s.to_json_string(), "2026-11-05T09:30:15Z");
        assert_eq!(s.to_iso_seconds_string(), "2026-11-05T09:30:15Z");
        assert_eq!(
            DateTime::from_ticks(s.ticks() + 1).to_json_string(),
            "2026-11-05T09:30:15.0000001Z"
        );
        assert_eq!(DateTime::parse_json(&d.to_json_string()), Some(d));
    }
}
