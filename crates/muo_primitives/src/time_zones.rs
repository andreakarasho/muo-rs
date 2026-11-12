//! Time zones with their region names.
//!
//! `TimeZoneHandler` (a `TimeZoneInfo` chosen by the `system.localTimeZone` setting) needs a time zone
//! database, which this crate doesn't carry.

use serde::{Deserialize, Serialize};

/// One entry of the time zone/regions JSON: `{"timezone": "...", "regions": ["...", ...]}`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeZoneWithRegions {
    #[serde(rename = "timezone", default)]
    pub time_zone: String,
    #[serde(default)]
    pub regions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_names() {
        let z: TimeZoneWithRegions =
            serde_json::from_str(r#"{"timezone":"Pacific Standard Time","regions":["Britain"]}"#)
                .unwrap();
        assert_eq!(z.time_zone, "Pacific Standard Time");
        assert_eq!(z.regions, ["Britain"]);
        assert!(serde_json::to_string(&z).unwrap().contains("\"timezone\""));
    }
}
