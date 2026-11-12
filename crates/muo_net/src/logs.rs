//! The diagnostic text files the transport appends to (`network-disconnects.log`, `network-errors.log`,
//! `unhandled-packets.log`, per-connection `packets.log`) and `Utility.FormatBuffer`.
//!
//! Timestamps are UTC until the server swaps the clock with
//! [`set_clock`] (the content crate does at startup, with the system time zone).

use std::fmt::Write as _;
use std::io::Write as _;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// The platform line ending the log files use.
pub const NL: &str = if cfg!(windows) { "\r\n" } else { "\n" };

/// Local wall-clock fields (year, month, day, hour, minute, second, ticks of 100ns within the second).
pub type Clock = fn() -> (i32, u32, u32, u32, u32, u32, u32);

static DIRECTORY: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);
static CLOCK: Mutex<Option<Clock>> = Mutex::new(None);

/// Where the log files go (default: the working directory).
pub fn set_directory(dir: std::path::PathBuf) {
    if let Ok(mut d) = DIRECTORY.lock() {
        *d = Some(dir);
    }
}

/// Replaces the UTC clock used for log timestamps.
pub fn set_clock(clock: Clock) {
    if let Ok(mut c) = CLOCK.lock() {
        *c = Some(clock);
    }
}

fn utc_fields() -> (i32, u32, u32, u32, u32, u32, u32) {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));

    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;

    (
        year,
        month,
        day,
        (rem / 3600) as u32,
        (rem / 60 % 60) as u32,
        (rem % 60) as u32,
        d.subsec_nanos() / 100,
    )
}

fn fields() -> (i32, u32, u32, u32, u32, u32, u32) {
    match CLOCK.lock().ok().and_then(|c| *c) {
        Some(f) => f(),
        None => utc_fields(),
    }
}

/// `Core.Now` as written after `# `: `MM/dd/yyyy HH:mm:ss`.
pub fn now_string() -> String {
    let (y, mo, d, h, mi, s, _) = fields();
    format!("{mo:02}/{d:02}/{y:04} {h:02}:{mi:02}:{s:02}")
}

/// `{Core.Now:HH:mm:ss.ffff}`
pub fn time_of_day_string() -> String {
    let (_, _, _, h, mi, s, ticks) = fields();
    format!("{h:02}:{mi:02}:{s:02}.{:04}", ticks / 1000)
}

/// `Utility.FormatBuffer`: a 16-column hex dump.
pub fn format_buffer(data: &[u8]) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "        0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F{NL}"
    );
    let _ = write!(
        out,
        "       -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --{NL}"
    );

    if data.is_empty() {
        let _ = write!(out, "0000   {NL}");
        return out;
    }

    for (i, line) in data.chunks(16).enumerate() {
        let hex: Vec<String> = line.iter().map(|b| format!("{b:02X}")).collect();
        let _ = write!(out, "{:04X}   {}{NL}", i * 16, hex.join(" "));
    }
    out
}

fn append(name: &str, text: &str) -> std::io::Result<()> {
    let dir = DIRECTORY
        .lock()
        .ok()
        .and_then(|d| d.clone())
        .unwrap_or_default();
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(name))?
        .write_all(text.as_bytes())
}

/// `NetState.TraceDisconnect`: records why a connection ended; blank reasons are not worth a line.
pub fn trace_disconnect(reason: &str, ip: &str) {
    if reason.trim().is_empty() {
        return;
    }

    let text = format!(
        "# {}{NL}NetState: {ip}{NL}{reason}{NL}{NL}{NL}",
        now_string()
    );
    if let Err(e) = append("network-disconnects.log", &text) {
        trace_exception(&e.to_string());
    }
}

/// `NetState.TraceException`.
pub fn trace_exception(description: &str) {
    let text = format!("# {}{NL}{description}{NL}{NL}{NL}", now_string());
    let _ = append("network-errors.log", &text);
    eprintln!("{description}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_buffer_layout() {
        let dump = format_buffer(&(0..18u8).collect::<Vec<_>>());
        let lines: Vec<&str> = dump.split(NL).collect();
        assert_eq!(
            lines[0],
            "        0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F"
        );
        assert_eq!(
            lines[1],
            "       -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --"
        );
        assert_eq!(
            lines[2],
            "0000   00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F"
        );
        assert_eq!(lines[3], "0010   10 11");
        assert!(format_buffer(&[]).ends_with(&format!("0000   {NL}")));
    }

    #[test]
    fn clock_strings_have_the_expected_shape() {
        let now = now_string();
        assert_eq!(now.len(), 19);
        assert_eq!(time_of_day_string().len(), 13);
        set_clock(|| (2026, 11, 5, 9, 8, 7, 1_234_567));
        assert_eq!(now_string(), "11/05/2026 09:08:07");
        assert_eq!(time_of_day_string(), "09:08:07.1234");
        *CLOCK.lock().unwrap() = None;
    }
}
