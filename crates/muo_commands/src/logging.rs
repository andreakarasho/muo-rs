//! `CommandLogging`, the per-command audit log under `Logs/Commands`.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use muo_server::ids::{EntityId, MobileId};
use muo_server::world::World;

use muo_content::props::Value;

/// `CommandLogging.Enabled` and the shared output file.
pub struct CommandLogging {
    pub enabled: bool,
    output: Option<PathBuf>,
}

impl Default for CommandLogging {
    fn default() -> Self {
        Self {
            enabled: true,
            output: None,
        }
    }
}

/// `CommandLogging.Initialize`: opens `Logs/Commands/<date>.log` and logs every command through the event.
pub fn initialize(w: &mut World) {
    let dir = w.core.base_directory.join("Logs").join("Commands");
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(format!("{}.log", long_date_string(w.now)));
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "##############################");
        let _ = writeln!(f, "Log started on {}", w.now);
        let _ = writeln!(f);
        w.resources.get_mut::<CommandLogging>().output = Some(path);
    }
    w.events.command.subscribe(|w, e| {
        let from = e.mobile;
        let text = format!(
            "{} {} used command '{} {}'",
            access_name(w, from),
            format_entity(w, EntityId::Mobile(from)),
            e.command,
            e.arg_string
        );
        write_line(w, from, &text);
    });
}

/// `DateTime.ToLongDateString()` under the invariant culture the server runs with (`dddd, dd MMMM yyyy`).
pub fn long_date_string(t: muo_primitives::DateTime) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let (y, m, d) = t.ymd();
    format!(
        "{:?}, {d:02} {} {y:04}",
        t.day_of_week(),
        MONTHS[(m as usize).saturating_sub(1).min(11)]
    )
}

fn access_name(w: &World, m: MobileId) -> String {
    w.mobile(m)
        .map_or(String::new(), |m| format!("{:?}", m.access_level))
}

fn account_name(w: &World, m: MobileId) -> Option<String> {
    let acct = w.mobile(m)?.account?;
    muo_content::accounting::account(w, acct).map(|a| a.username.clone())
}

/// `CommandLogging.Format`: a mobile as `0x.. "Name" ('account')`, an item as `0x.. (Type)`.
pub fn format_entity(w: &World, e: EntityId) -> String {
    match e {
        EntityId::Mobile(m) => {
            let name = w
                .mobile(m)
                .and_then(|x| (x.ty.name)(w, m))
                .unwrap_or_default();
            let head = format!("0x{:08X} \"{name}\"", m.0.value());
            match account_name(w, m) {
                Some(a) => format!("{head} ('{a}')"),
                None => format!("{head} (no account)"),
            }
        }
        EntityId::Item(i) => {
            let t = w
                .item(i)
                .map_or("", |x| x.ty.full_name.rsplit('.').next().unwrap_or(""));
            format!("0x{:08X} ({t})", i.0.value())
        }
    }
}

/// `CommandLogging.WriteLine(from, text)`: to the shared log and to `Logs/Commands/<access level>/<name>.log`.
pub fn write_line(w: &mut World, from: MobileId, text: &str) {
    let log = w.resources.get_mut::<CommandLogging>();
    if !log.enabled {
        return;
    }
    let shared = log.output.clone();
    let ns = w
        .mobile(from)
        .and_then(|m| m.net_state)
        .and_then(|ns| w.net_state(ns))
        .map(|d| d.description.clone())
        .unwrap_or_default();
    let line = format!("{}: {}: {}", w.now, ns, text);
    if let Some(p) = shared
        && let Ok(mut f) = OpenOptions::new().append(true).open(p)
    {
        let _ = writeln!(f, "{line}");
    }
    let name = account_name(w, from)
        .or_else(|| w.mobile(from).and_then(|m| (m.ty.name)(w, from)))
        .unwrap_or_default();
    let dir = w
        .core
        .base_directory
        .join("Logs")
        .join("Commands")
        .join(access_name(w, from));
    if fs::create_dir_all(&dir).is_ok()
        && let Ok(mut f) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join(format!("{name}.log")))
    {
        let _ = writeln!(f, "{line}");
    }
}

/// `CommandLogging.Safe`: a file name from text, the characters Windows forbids replaced by `_`.
pub fn safe(ip: Option<&str>) -> String {
    let Some(ip) = ip else { return "null".into() };
    let ip = ip.trim();
    if ip.is_empty() {
        return "empty".into();
    }
    ip.chars()
        .map(|c| if "\\/:*?\"<>|".contains(c) { '_' } else { c })
        .collect()
}

/// `CommandLogging.LogChangeProperty`.
pub fn log_change_property(w: &mut World, from: MobileId, o: EntityId, name: &str, value: &str) {
    let text = format!(
        "{} {} set property '{name}' of {} to '{value}'",
        access_name(w, from),
        format_entity(w, EntityId::Mobile(from)),
        format_entity(w, o)
    );
    write_line(w, from, &text);
}

/// `CommandLogging.WriteLine(from, $"{from.AccessLevel} {Format(from)} {text}")`.
pub fn log_text(w: &mut World, from: MobileId, text: &str) {
    let line = format!(
        "{} {} {text}",
        access_name(w, from),
        format_entity(w, EntityId::Mobile(from))
    );
    write_line(w, from, &line);
}

/// Shorthand for the `CommandLogging.WriteLine(from, $"{from.AccessLevel} {Format(from)} verbing {Format(o)}")`
/// lines most commands start with.
pub fn log_action(w: &mut World, from: MobileId, verb: &str, target: EntityId) {
    let text = format!(
        "{} {} {verb} {}",
        access_name(w, from),
        format_entity(w, EntityId::Mobile(from)),
        format_entity(w, target)
    );
    write_line(w, from, &text);
}

/// Display text of a `Value` for the change-property log (`toSet?.ToString() ?? "(-null-)"`).
pub fn value_text(w: &World, v: &Value) -> String {
    match v {
        Value::Null => "(-null-)".into(),
        other => muo_content::props::display(w, &muo_content::props::Kind::Text, other),
    }
}
