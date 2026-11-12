//! The system's local time, in pure Rust (`chrono`'s `Local`, which reads the OS time zone: `localtime_r` on
//! Unix, the Win32 time zone API on Windows): what `DateTime.Now` and `Core.Now.ToSystemLocalTime()` answer.
//! [`install`] points the diagnostic logs at it ([`muo_net::logs::set_clock`]); [`configure`] makes
//! `AutoSave` follow it ([`LocalTimeOffset`]) and reads `system.localTimeZone` (`TimeZoneHandler.Configure`).
//!
//! A named zone of that setting is an IANA id ("Europe/Paris", from `chrono-tz`) or a Windows id ("Pacific Standard
//! Time", through the CLDR table below), both are accepted; an unknown one stops the startup.

use std::sync::RwLock;

use chrono::{Datelike, Local, Offset, TimeZone, Timelike};
use chrono_tz::Tz;
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;

use super::auto_save::{LocalTimeOffset, TrackSystemTimeZone};

/// `TimeZoneHandler.SystemTimeZone` when the setting names a zone other than the machine's.
static ZONE: RwLock<Option<Tz>> = RwLock::new(None);

fn zone() -> Option<Tz> {
    ZONE.read().ok().and_then(|z| *z)
}

fn fields<T: Datelike + Timelike>(n: &T) -> (i32, u32, u32, u32, u32, u32, u32) {
    (
        n.year(),
        n.month(),
        n.day(),
        n.hour(),
        n.minute(),
        n.second(),
        n.nanosecond().min(999_999_999) / 100,
    )
}

/// The local wall clock for [`muo_net::logs::set_clock`]: year, month, day, hour, minute, second and 100ns ticks.
pub fn local_fields() -> (i32, u32, u32, u32, u32, u32, u32) {
    match zone() {
        Some(z) => fields(&chrono::Utc::now().with_timezone(&z)),
        None => fields(&Local::now()),
    }
}

/// The local time's offset from UTC at the UTC instant `utc` (daylight saving included).
pub fn offset_at(utc: DateTime) -> TimeSpan {
    // Ticks since 0001-01-01 to seconds since 1970-01-01.
    const UNIX_EPOCH_TICKS: i64 = 621_355_968_000_000_000;
    let secs = (utc.ticks() - UNIX_EPOCH_TICKS).div_euclid(10_000_000);
    let offset = match zone() {
        Some(z) => z
            .timestamp_opt(secs, 0)
            .earliest()
            .map_or(0, |t| t.offset().fix().local_minus_utc()),
        None => Local
            .timestamp_opt(secs, 0)
            .earliest()
            .map_or(0, |t| t.offset().fix().local_minus_utc()),
    };
    TimeSpan::from_secs(i64::from(offset))
}

/// Points the log timestamps at the local clock. Called before anything logs.
pub fn install() {
    muo_net::logs::set_clock(local_fields);
}

/// CLDR `windowsZones.xml`, the territory-001 (default) IANA zone of each Windows time zone id: the ids Windows
/// accepts in `FindSystemTimeZoneById`. Checked against the ICU Windows-to-IANA mapping for every row: all agree
/// except "Mountain Standard Time (Mexico)" (now America/Chihuahua) and "Kamchatka Standard Time" (no mapping; kept).
const WINDOWS_ZONES: &[(&str, &str)] = &[
    ("Dateline Standard Time", "Etc/GMT+12"),
    ("UTC-11", "Etc/GMT+11"),
    ("Aleutian Standard Time", "America/Adak"),
    ("Hawaiian Standard Time", "Pacific/Honolulu"),
    ("Marquesas Standard Time", "Pacific/Marquesas"),
    ("Alaskan Standard Time", "America/Anchorage"),
    ("UTC-09", "Etc/GMT+9"),
    ("Pacific Standard Time (Mexico)", "America/Tijuana"),
    ("UTC-08", "Etc/GMT+8"),
    ("Pacific Standard Time", "America/Los_Angeles"),
    ("US Mountain Standard Time", "America/Phoenix"),
    ("Mountain Standard Time (Mexico)", "America/Chihuahua"),
    ("Mountain Standard Time", "America/Denver"),
    ("Yukon Standard Time", "America/Whitehorse"),
    ("Central America Standard Time", "America/Guatemala"),
    ("Central Standard Time", "America/Chicago"),
    ("Easter Island Standard Time", "Pacific/Easter"),
    ("Central Standard Time (Mexico)", "America/Mexico_City"),
    ("Canada Central Standard Time", "America/Regina"),
    ("SA Pacific Standard Time", "America/Bogota"),
    ("Eastern Standard Time (Mexico)", "America/Cancun"),
    ("Eastern Standard Time", "America/New_York"),
    ("Haiti Standard Time", "America/Port-au-Prince"),
    ("Cuba Standard Time", "America/Havana"),
    ("US Eastern Standard Time", "America/Indianapolis"),
    ("Turks And Caicos Standard Time", "America/Grand_Turk"),
    ("Paraguay Standard Time", "America/Asuncion"),
    ("Atlantic Standard Time", "America/Halifax"),
    ("Venezuela Standard Time", "America/Caracas"),
    ("Central Brazilian Standard Time", "America/Cuiaba"),
    ("SA Western Standard Time", "America/La_Paz"),
    ("Pacific SA Standard Time", "America/Santiago"),
    ("Newfoundland Standard Time", "America/St_Johns"),
    ("Tocantins Standard Time", "America/Araguaina"),
    ("E. South America Standard Time", "America/Sao_Paulo"),
    ("SA Eastern Standard Time", "America/Cayenne"),
    ("Argentina Standard Time", "America/Buenos_Aires"),
    ("Greenland Standard Time", "America/Godthab"),
    ("Montevideo Standard Time", "America/Montevideo"),
    ("Magallanes Standard Time", "America/Punta_Arenas"),
    ("Saint Pierre Standard Time", "America/Miquelon"),
    ("Bahia Standard Time", "America/Bahia"),
    ("UTC-02", "Etc/GMT+2"),
    ("Azores Standard Time", "Atlantic/Azores"),
    ("Cape Verde Standard Time", "Atlantic/Cape_Verde"),
    ("UTC", "Etc/UTC"),
    ("GMT Standard Time", "Europe/London"),
    ("Greenwich Standard Time", "Atlantic/Reykjavik"),
    ("Sao Tome Standard Time", "Africa/Sao_Tome"),
    ("Morocco Standard Time", "Africa/Casablanca"),
    ("W. Europe Standard Time", "Europe/Berlin"),
    ("Central Europe Standard Time", "Europe/Budapest"),
    ("Romance Standard Time", "Europe/Paris"),
    ("Central European Standard Time", "Europe/Warsaw"),
    ("W. Central Africa Standard Time", "Africa/Lagos"),
    ("Jordan Standard Time", "Asia/Amman"),
    ("GTB Standard Time", "Europe/Bucharest"),
    ("Middle East Standard Time", "Asia/Beirut"),
    ("Egypt Standard Time", "Africa/Cairo"),
    ("E. Europe Standard Time", "Europe/Chisinau"),
    ("Syria Standard Time", "Asia/Damascus"),
    ("West Bank Standard Time", "Asia/Hebron"),
    ("South Africa Standard Time", "Africa/Johannesburg"),
    ("FLE Standard Time", "Europe/Kiev"),
    ("Israel Standard Time", "Asia/Jerusalem"),
    ("South Sudan Standard Time", "Africa/Juba"),
    ("Kaliningrad Standard Time", "Europe/Kaliningrad"),
    ("Sudan Standard Time", "Africa/Khartoum"),
    ("Libya Standard Time", "Africa/Tripoli"),
    ("Namibia Standard Time", "Africa/Windhoek"),
    ("Arabic Standard Time", "Asia/Baghdad"),
    ("Turkey Standard Time", "Europe/Istanbul"),
    ("Arab Standard Time", "Asia/Riyadh"),
    ("Belarus Standard Time", "Europe/Minsk"),
    ("Russian Standard Time", "Europe/Moscow"),
    ("E. Africa Standard Time", "Africa/Nairobi"),
    ("Iran Standard Time", "Asia/Tehran"),
    ("Arabian Standard Time", "Asia/Dubai"),
    ("Astrakhan Standard Time", "Europe/Astrakhan"),
    ("Azerbaijan Standard Time", "Asia/Baku"),
    ("Russia Time Zone 3", "Europe/Samara"),
    ("Mauritius Standard Time", "Indian/Mauritius"),
    ("Saratov Standard Time", "Europe/Saratov"),
    ("Georgian Standard Time", "Asia/Tbilisi"),
    ("Volgograd Standard Time", "Europe/Volgograd"),
    ("Caucasus Standard Time", "Asia/Yerevan"),
    ("Afghanistan Standard Time", "Asia/Kabul"),
    ("West Asia Standard Time", "Asia/Tashkent"),
    ("Ekaterinburg Standard Time", "Asia/Yekaterinburg"),
    ("Pakistan Standard Time", "Asia/Karachi"),
    ("Qyzylorda Standard Time", "Asia/Qyzylorda"),
    ("India Standard Time", "Asia/Calcutta"),
    ("Sri Lanka Standard Time", "Asia/Colombo"),
    ("Nepal Standard Time", "Asia/Katmandu"),
    ("Central Asia Standard Time", "Asia/Almaty"),
    ("Bangladesh Standard Time", "Asia/Dhaka"),
    ("Omsk Standard Time", "Asia/Omsk"),
    ("Myanmar Standard Time", "Asia/Rangoon"),
    ("SE Asia Standard Time", "Asia/Bangkok"),
    ("Altai Standard Time", "Asia/Barnaul"),
    ("W. Mongolia Standard Time", "Asia/Hovd"),
    ("North Asia Standard Time", "Asia/Krasnoyarsk"),
    ("N. Central Asia Standard Time", "Asia/Novosibirsk"),
    ("Tomsk Standard Time", "Asia/Tomsk"),
    ("China Standard Time", "Asia/Shanghai"),
    ("North Asia East Standard Time", "Asia/Irkutsk"),
    ("Singapore Standard Time", "Asia/Singapore"),
    ("W. Australia Standard Time", "Australia/Perth"),
    ("Taipei Standard Time", "Asia/Taipei"),
    ("Ulaanbaatar Standard Time", "Asia/Ulaanbaatar"),
    ("Aus Central W. Standard Time", "Australia/Eucla"),
    ("Transbaikal Standard Time", "Asia/Chita"),
    ("Tokyo Standard Time", "Asia/Tokyo"),
    ("North Korea Standard Time", "Asia/Pyongyang"),
    ("Korea Standard Time", "Asia/Seoul"),
    ("Yakutsk Standard Time", "Asia/Yakutsk"),
    ("Cen. Australia Standard Time", "Australia/Adelaide"),
    ("AUS Central Standard Time", "Australia/Darwin"),
    ("E. Australia Standard Time", "Australia/Brisbane"),
    ("AUS Eastern Standard Time", "Australia/Sydney"),
    ("West Pacific Standard Time", "Pacific/Port_Moresby"),
    ("Tasmania Standard Time", "Australia/Hobart"),
    ("Vladivostok Standard Time", "Asia/Vladivostok"),
    ("Lord Howe Standard Time", "Australia/Lord_Howe"),
    ("Bougainville Standard Time", "Pacific/Bougainville"),
    ("Russia Time Zone 10", "Asia/Srednekolymsk"),
    ("Magadan Standard Time", "Asia/Magadan"),
    ("Norfolk Standard Time", "Pacific/Norfolk"),
    ("Sakhalin Standard Time", "Asia/Sakhalin"),
    ("Central Pacific Standard Time", "Pacific/Guadalcanal"),
    ("Russia Time Zone 11", "Asia/Kamchatka"),
    ("New Zealand Standard Time", "Pacific/Auckland"),
    ("UTC+12", "Etc/GMT-12"),
    ("Fiji Standard Time", "Pacific/Fiji"),
    ("Kamchatka Standard Time", "Asia/Kamchatka"),
    ("Chatham Islands Standard Time", "Pacific/Chatham"),
    ("UTC+13", "Etc/GMT-13"),
    ("Tonga Standard Time", "Pacific/Tongatapu"),
    ("Samoa Standard Time", "Pacific/Apia"),
    ("Line Islands Standard Time", "Pacific/Kiritimati"),
    // Windows 11 ids the ICU Windows-to-IANA mapping has no entry for (CLDR lists none) but
    // `FindSystemTimeZoneById` finds on Windows: same offsets and rules as these zones. "Mid-Atlantic Standard Time"
    // (UTC-2 with DST) has no IANA zone and stays unknown.
    ("British Columbia Standard Time", "America/Vancouver"),
    ("Alberta Standard Time", "America/Edmonton"),
    ("Manitoba Standard Time", "America/Winnipeg"),
];

/// `TimeZoneInfo.FindSystemTimeZoneById`: an IANA id, or a Windows id (case-insensitive, as the registry lookup is).
pub fn find_time_zone(id: &str) -> Option<Tz> {
    id.parse::<Tz>().ok().or_else(|| {
        let (_, iana) = WINDOWS_ZONES
            .iter()
            .find(|(win, _)| win.eq_ignore_ascii_case(id))?;
        iana.parse().ok()
    })
}

/// `TimeZoneHandler.Configure`: a `system.localTimeZone` naming a zone replaces the machine's. The default is the
/// machine zone's id, so an unset (empty) setting changes nothing. An unknown id stops the startup: it panics.
fn configure_zone(w: &mut World) {
    let id: String = w.config.get_setting("system.localTimeZone", String::new());
    if id.is_empty() {
        return;
    }
    match find_time_zone(&id) {
        Some(z) => *ZONE.write().expect("zone lock") = Some(z),
        None => panic!(
            "TimeZoneNotFoundException: The time zone ID '{id}' was not found on the local computer."
        ),
    }
}

/// Startup: the same, and `AutoSave` follows the system offset.
pub fn configure(w: &mut World) {
    install();
    configure_zone(w);
    let now = w.now;
    w.resources.get_mut::<LocalTimeOffset>().0 = offset_at(now);
    w.resources.get_mut::<TrackSystemTimeZone>().0 = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_offset_is_a_sane_time_zone_offset() {
        let o = offset_at(DateTime::utc_now());
        assert!(o >= TimeSpan::from_hours(-14) && o <= TimeSpan::from_hours(14));
    }

    #[test]
    fn a_named_zone_replaces_the_machines() {
        *ZONE.write().unwrap() = Some("Asia/Kolkata".parse().unwrap());
        // 1970-01-01 in ticks: India is UTC+5:30 all year.
        assert_eq!(offset_at(DateTime::UNIX_EPOCH), TimeSpan::from_secs(19_800));
        *ZONE.write().unwrap() = None;
    }

    #[test]
    fn every_windows_id_maps_to_a_known_zone() {
        for (win, iana) in WINDOWS_ZONES {
            assert!(iana.parse::<Tz>().is_ok(), "{win} -> {iana}");
        }
    }

    #[test]
    fn a_windows_id_finds_its_iana_zone() {
        assert_eq!(
            find_time_zone("Pacific Standard Time"),
            Some(chrono_tz::America::Los_Angeles)
        );
        assert_eq!(
            find_time_zone("pacific standard time"),
            Some(chrono_tz::America::Los_Angeles)
        );
        assert_eq!(
            find_time_zone("India Standard Time"),
            Some(chrono_tz::Asia::Calcutta)
        );
        assert_eq!(
            find_time_zone("Europe/Paris"),
            Some(chrono_tz::Europe::Paris)
        );
        assert_eq!(
            find_time_zone("Mountain Standard Time (Mexico)"),
            Some(chrono_tz::America::Chihuahua)
        );
        assert_eq!(find_time_zone("Nowhere Standard Time"), None);
    }

    #[test]
    #[should_panic(expected = "TimeZoneNotFoundException")]
    fn an_unknown_zone_id_stops_the_startup() {
        let mut w = World::new();
        w.config
            .set_setting("system.localTimeZone", &"Nowhere Standard Time".to_string())
            .unwrap();
        configure_zone(&mut w);
    }

    #[test]
    fn local_fields_are_a_calendar_date() {
        let (y, mo, d, h, mi, s, t) = local_fields();
        assert!(y >= 2024 && (1..=12).contains(&mo) && (1..=31).contains(&d));
        assert!(h < 24 && mi < 60 && s < 61 && t < 10_000_000);
    }
}
