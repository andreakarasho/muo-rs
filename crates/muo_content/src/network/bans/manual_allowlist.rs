//! The manual allowlist and its configuration: addresses an
//! operator carved out by hand in `Configuration/ip-allowlist*.txt`, which suppress ban contributions and blocklist
//! hits. The files are re-read when their stamp changes.

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::UNIX_EPOCH;

use muo_primitives::TimeSpan;
use muo_server::World;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::snapshot::BlocklistSnapshot;
use super::{StopFlag, resolve_path, stop_flag};

const PATH: &str = "Configuration/ip-allowlist.json";

/// `ManualAllowlistSettings`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ManualAllowlistSettings {
    pub enabled: bool,
    pub files: Vec<String>,
    #[serde(
        rename = "reloadInterval",
        with = "muo_data::json::converters::timespan"
    )]
    pub reload_interval: TimeSpan,
}

impl Default for ManualAllowlistSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            files: vec!["Configuration/ip-allowlist*.txt".to_owned()],
            reload_interval: TimeSpan::from_secs(60),
        }
    }
}

/// What the accept path and the pollers share.
#[derive(Default)]
struct Shared {
    snapshot: RwLock<BlocklistSnapshot>,
    last_stamp: AtomicI64,
    enabled: AtomicBool,
}

/// The shared read side: `ManualAllowlist.Contains`.
#[derive(Clone, Default)]
pub struct ManualAllowlist {
    shared: Arc<Shared>,
}

impl ManualAllowlist {
    /// `Count`.
    pub fn count(&self) -> usize {
        self.shared.snapshot.read().map_or(0, |s| s.count())
    }

    /// `Contains(address)`.
    pub fn contains(&self, address: IpAddr) -> bool {
        self.shared
            .snapshot
            .read()
            .is_ok_and(|s| s.is_banned(address))
    }

    /// `Enabled`.
    pub fn enabled(&self) -> bool {
        self.shared.enabled.load(Ordering::SeqCst)
    }

    /// `LoadForTesting(snapshot)`.
    pub fn load_for_testing(&self, snapshot: BlocklistSnapshot) {
        if let Ok(mut s) = self.shared.snapshot.write() {
            *s = snapshot;
        }
    }
}

/// `*` and `?` against a file name, as the directory listing of `Directory.GetFiles(dir, pattern)` matches.
fn wildcard(pattern: &[char], name: &[char]) -> bool {
    match pattern.split_first() {
        None => name.is_empty(),
        Some(('*', rest)) => (0..=name.len()).any(|i| wildcard(rest, &name[i..])),
        Some((p, rest)) => name.first().is_some_and(|n| {
            (*p == '?' || n.eq_ignore_ascii_case(p)) && wildcard(rest, &name[1..])
        }),
    }
}

/// `ExpandPaths()`: every file the patterns name now.
fn expand_paths(patterns: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for pattern in patterns {
        let Some(name) = pattern
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
        else {
            continue;
        };
        if !name.contains('*') && !name.contains('?') {
            if pattern.is_file() {
                files.push(pattern.clone());
            }
            continue;
        }
        let Some(dir) = pattern.parent().filter(|d| d.is_dir()) else {
            continue;
        };
        let Ok(read) = std::fs::read_dir(dir) else {
            continue; // unreadable directory; the next poll retries
        };
        let pat: Vec<char> = name.chars().collect();
        let mut matches: Vec<PathBuf> = read
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.file_name().is_some_and(|n| {
                        wildcard(&pat, &n.to_string_lossy().chars().collect::<Vec<_>>())
                    })
            })
            .collect();
        matches.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
        // ".txt" can pull in the generator's ".txt.tmp" mid-swap on Windows: check the real extension
        files.extend(
            matches
                .into_iter()
                .filter(|m| m.extension().is_some_and(|e| e.eq_ignore_ascii_case("txt"))),
        );
    }
    files
}

/// `Stamp()`: a fingerprint of the files' modification times and sizes.
fn stamp(patterns: &[PathBuf]) -> i64 {
    let mut stamp = 0i64;
    for p in expand_paths(patterns) {
        if let Ok(meta) = std::fs::metadata(&p) {
            let ticks = meta
                .modified()
                .ok()
                .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |d| (d.as_nanos() / 100) as i64);
            stamp = stamp
                .wrapping_mul(31)
                .wrapping_add(ticks)
                .wrapping_add(meta.len() as i64);
        }
    }
    stamp
}

/// `Reload()`: reads every file, newline-joined, into one snapshot.
fn reload(shared: &Shared, patterns: &[PathBuf]) {
    // Fingerprint BEFORE parsing, so it describes the version being read
    let st = stamp(patterns);
    let mut combined = Vec::new();
    let mut files = 0;
    for p in expand_paths(patterns) {
        match std::fs::read(&p) {
            Ok(bytes) => {
                combined.extend_from_slice(&bytes);
                combined.push(b'\n');
                files += 1;
            }
            // Fail open per file: losing one entry beats refusing to load the rest.
            Err(e) => warn!("Could not read allowlist \"{}\": {e}", p.display()),
        }
    }
    let next = if combined.is_empty() {
        BlocklistSnapshot::empty()
    } else {
        BlocklistSnapshot::build(&combined).0
    };
    let count = next.count();
    if let Ok(mut s) = shared.snapshot.write() {
        *s = next;
    }
    shared.last_stamp.store(st, Ordering::SeqCst);
    info!("Manual allowlist loaded {count} range(s) from {files} file(s)");
}

/// `ManualAllowlist.Initialize`.
pub fn initialize(w: &mut World) {
    let settings: ManualAllowlistSettings =
        super::load_settings(w, PATH, ManualAllowlistSettings::default());
    let mut patterns: Vec<PathBuf> = settings
        .files
        .iter()
        .filter(|f| !f.trim().is_empty())
        .map(|f| resolve_path(w, f))
        .collect();
    let interval = if settings.reload_interval <= TimeSpan::ZERO {
        std::time::Duration::from_secs(60)
    } else {
        std::time::Duration::from_secs_f64(settings.reload_interval.total_seconds())
    };

    if !settings.enabled {
        // The generator still subtracts these files, so a carve-out an operator already wrote looks like it works
        // right up until a behavioural detection contributes the address anyway.
        let present = expand_paths(&patterns).len();
        patterns.clear();
        if present > 0 {
            warn!(
                "Manual allowlist is off (\"enabled\" false in ip-allowlist.json) but {present} allowlist file(s) are present; those carve-outs will not suppress ban contributions"
            );
        } else {
            info!("Manual allowlist disabled (\"enabled\" false in ip-allowlist.json)");
        }
        return;
    }
    if patterns.is_empty() {
        info!("Manual allowlist disabled (\"files\" empty in ip-allowlist.json)");
        return;
    }

    let handle = handle(w);
    handle.shared.enabled.store(true, Ordering::SeqCst);
    reload(&handle.shared, &patterns);
    let (shared, stop, state) = (handle.shared.clone(), stop_flag(w), w.state_mirror.clone());
    std::thread::spawn(move || poll_loop(&shared, &patterns, interval, &stop, &state));
}

fn poll_loop(
    shared: &Shared,
    patterns: &[PathBuf],
    interval: std::time::Duration,
    stop: &StopFlag,
    world_state: &std::sync::atomic::AtomicU8,
) {
    while !stop.sleep(interval) {
        if stamp(patterns) != shared.last_stamp.load(Ordering::SeqCst) {
            // A save owns the disk and nothing here is urgent.
            if stop.wait_while_saving(world_state) {
                return;
            }
            reload(shared, patterns);
        }
    }
}

/// The handle the filters and `Initialize` share (made empty and disabled on first use).
pub fn handle(w: &mut World) -> ManualAllowlist {
    w.resources.get_mut::<ManualAllowlist>().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcards_match_like_a_directory_listing() {
        let m = |p: &str, n: &str| {
            wildcard(
                &p.chars().collect::<Vec<_>>(),
                &n.chars().collect::<Vec<_>>(),
            )
        };
        assert!(m("ip-allowlist*.txt", "ip-allowlist.txt"));
        assert!(m("ip-allowlist*.txt", "ip-allowlist-office.txt"));
        assert!(!m("ip-allowlist*.txt", "ip-allowlist.txt.tmp"));
        assert!(m("a?c", "abc") && !m("a?c", "ac"));
    }

    #[test]
    fn settings_default_is_off_with_the_shipped_pattern() {
        let s = ManualAllowlistSettings::default();
        assert!(!s.enabled);
        assert_eq!(s.files, ["Configuration/ip-allowlist*.txt"]);
        let json = muo_data::json::to_string(&s).unwrap();
        assert!(json.contains("\"reloadInterval\": \"00:01:00\""), "{json}");
    }

    #[test]
    fn contains_uses_the_loaded_snapshot() {
        let h = ManualAllowlist::default();
        assert!(!h.contains("1.2.3.4".parse().unwrap()));
        h.load_for_testing(BlocklistSnapshot::build(b"1.2.3.0/24").0);
        assert!(h.contains("1.2.3.4".parse().unwrap()));
    }
}
