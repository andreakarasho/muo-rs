//! Assistant configuration: `Configuration/assistants.json`.

use std::path::PathBuf;

use muo_primitives::TimeSpan;
use muo_server::World;
use serde::{Deserialize, Serialize};

use super::assistant_features::AssistantFeatures;

const PATH: &str = "Configuration/assistants.json";

const DEFAULT_WARNING_MESSAGE: &str = "The server was unable to negotiate features with your assistant. \
You must download and run an updated version of <A HREF=\"https://uosteam.com\">UOSteam</A>\
 or <A HREF=\"https://github.com/markdwags/Razor/releases/latest\">Razor</A>.\
<BR><BR>Make sure you've checked the option <B>Negotiate features with server</B>, \
once you have this box checked you may log in and play normally.\
<BR><BR>You will be disconnected shortly.";

/// `AssistantSettings`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssistantSettings {
    #[serde(rename = "warnOnFailure", default)]
    pub warn_on_failure: bool,
    #[serde(rename = "kickOnFailure", default)]
    pub kick_on_failure: bool,
    #[serde(
        rename = "disallowedFeatures",
        with = "muo_data::json::converters::flags",
        default
    )]
    pub disallowed_features: AssistantFeatures,
    #[serde(
        rename = "disconnectDelay",
        with = "muo_data::json::converters::timespan",
        default = "zero"
    )]
    pub disconnect_delay: TimeSpan,
    #[serde(
        rename = "warningMessage",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub warning_message: Option<String>,
}

fn zero() -> TimeSpan {
    TimeSpan::ZERO
}

impl Default for AssistantSettings {
    fn default() -> Self {
        Self {
            warn_on_failure: true,
            kick_on_failure: true,
            disallowed_features: AssistantFeatures::NONE,
            disconnect_delay: TimeSpan::from_secs(15),
            warning_message: Some(DEFAULT_WARNING_MESSAGE.to_string()),
        }
    }
}

/// `AssistantConfiguration.Settings`, held in `w.resources`; defaults until `configure` runs.
#[derive(Default)]
pub struct AssistantConfiguration {
    pub settings: AssistantSettings,
    path: PathBuf,
}

impl AssistantConfiguration {
    fn save(&self) {
        if let Err(e) = muo_data::json::serialize_file(&self.path, &self.settings) {
            tracing::error!("Saving {PATH} failed: {e}");
        }
    }

    /// `SetDisallowed(feature, value)`; saves.
    pub fn set_disallowed(&mut self, feature: AssistantFeatures, value: bool) {
        self.settings.disallowed_features.set(feature, value);
        self.save();
    }

    pub fn disallow_feature(&mut self, feature: AssistantFeatures) {
        self.set_disallowed(feature, true);
    }

    pub fn allow_feature(&mut self, feature: AssistantFeatures) {
        self.set_disallowed(feature, false);
    }
}

/// `AssistantConfiguration.Configure()`.
pub fn configure(w: &mut World) {
    let path = w.core.base_directory.join(PATH);
    let cfg = w.resources.get_mut::<AssistantConfiguration>();
    cfg.path = path.clone();
    if path.exists() {
        match muo_data::json::deserialize_file::<AssistantSettings>(&path) {
            Ok(Some(s)) => cfg.settings = s,
            // A missing or unreadable file leaves the defaults.
            Ok(None) | Err(_) => {
                tracing::error!("Failed reading {PATH}; using the default assistant settings")
            }
        }
    } else {
        cfg.settings = AssistantSettings::default();
        cfg.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_file_shape_round_trips() {
        let s = AssistantSettings::default();
        let json = muo_data::json::to_string(&s).unwrap();
        assert!(
            json.contains("\"warnOnFailure\": true")
                && json.contains("\"disconnectDelay\": \"00:00:15\""),
            "{json}"
        );
        assert!(
            json.contains("\"FilterWeather\": false") && !json.contains("\"All\""),
            "{json}"
        );
        assert_eq!(
            muo_data::json::from_str::<AssistantSettings>(&json).unwrap(),
            s
        );
    }

    #[test]
    fn features_toggle_and_save() {
        let dir = std::env::temp_dir().join(format!("muo_assist_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("Configuration")).unwrap();
        let mut w = World::new();
        w.core.base_directory = dir.clone();
        configure(&mut w);
        let c = w.resources.get_mut::<AssistantConfiguration>();
        c.disallow_feature(AssistantFeatures::FILTER_LIGHT | AssistantFeatures::SMART_TARGET);
        c.allow_feature(AssistantFeatures::SMART_TARGET);
        assert_eq!(
            c.settings.disallowed_features,
            AssistantFeatures::FILTER_LIGHT
        );
        let back: AssistantSettings = muo_data::json::deserialize_file(&dir.join(PATH))
            .unwrap()
            .unwrap();
        assert_eq!(back.disallowed_features, AssistantFeatures::FILTER_LIGHT);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
