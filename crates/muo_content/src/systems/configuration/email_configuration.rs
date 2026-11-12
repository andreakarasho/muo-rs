//! Email configuration: `Configuration/email-settings.json`.
//!
//! shortcut: a mailbox is its name and address (no MIME mailbox type), upgrade if address
//! validation is ever needed.

use muo_server::World;
use serde::{Deserialize, Serialize};

const REL_PATH: &str = "Configuration/email-settings.json";

/// `MailboxAddress(name, address)`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MailboxAddress {
    pub name: String,
    pub address: String,
}

/// `EmailConfiguration.Settings`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub enabled: bool,
    #[serde(rename = "fromAddress")]
    pub from_address: String,
    #[serde(rename = "fromName")]
    pub from_name: String,
    #[serde(rename = "crashAddress")]
    pub crash_address: String,
    #[serde(rename = "crashName")]
    pub crash_name: String,
    #[serde(rename = "speechLogPageAddress")]
    pub speech_log_page_address: String,
    #[serde(rename = "speechLogPageName")]
    pub speech_log_page_name: String,
    #[serde(rename = "emailServer")]
    pub email_server: String,
    #[serde(rename = "emailPort")]
    pub email_port: i32,
    #[serde(rename = "emailUsername")]
    pub email_username: String,
    #[serde(rename = "emailPassword")]
    pub email_password: String,
    #[serde(rename = "emailSendRetryCount")]
    pub email_send_retry_count: i32,
    #[serde(rename = "emailSendRetryDelay")]
    pub email_send_retry_delay: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            from_address: "".into(),
            from_name: "".into(),
            crash_address: "".into(),
            crash_name: "Crash Log".into(),
            speech_log_page_address: "".into(),
            speech_log_page_name: "GM Support Conversation".into(),
            email_server: "smtp.gmail.com".into(),
            email_port: 465,
            email_username: "".into(),
            email_password: "Some Password 123".into(),
            email_send_retry_count: 5,
            email_send_retry_delay: 3,
        }
    }
}

/// The `EmailConfiguration` statics.
#[derive(Default)]
pub struct EmailConfiguration {
    pub crash_address: MailboxAddress,
    pub email_enabled: bool,
    pub email_port: i32,
    /// Seconds.
    pub email_send_retry_count: i32,
    /// Seconds.
    pub email_send_retry_delay: i32,
    pub email_server: String,
    pub email_server_username: String,
    pub email_server_password: String,
    pub from_address: MailboxAddress,
    pub speech_log_page_address: MailboxAddress,
}

impl EmailConfiguration {
    fn apply(&mut self, s: Settings) {
        self.email_enabled = s.enabled;
        self.from_address = MailboxAddress {
            name: s.from_name,
            address: s.from_address,
        };
        self.crash_address = MailboxAddress {
            name: s.crash_name,
            address: s.crash_address,
        };
        self.speech_log_page_address = MailboxAddress {
            name: s.speech_log_page_name,
            address: s.speech_log_page_address,
        };
        self.email_server = s.email_server;
        self.email_port = s.email_port;
        self.email_server_username = s.email_username;
        self.email_server_password = s.email_password;
        self.email_send_retry_count = s.email_send_retry_count;
        self.email_send_retry_delay = s.email_send_retry_delay;
    }
}

/// `EmailConfiguration.Configure()`. A file that doesn't read is logged and the defaults are kept: there is no
/// error channel in a phase function.
pub fn configure(w: &mut World) {
    let path = w.core.base_directory.join(REL_PATH);
    let settings = if path.exists() {
        match muo_data::json::deserialize_file::<Settings>(&path) {
            Ok(Some(s)) => {
                tracing::info!("Email configuration read from {REL_PATH}");
                s
            }
            _ => {
                tracing::error!("Failed reading email configuration from {REL_PATH}");
                Settings::default()
            }
        }
    } else {
        let s = Settings::default();
        if let Err(e) = muo_data::json::serialize_file(&path, &s) {
            tracing::error!("Saving the email configuration failed: {e}");
        }
        tracing::info!("Email configuration saved to {REL_PATH}.");
        s
    };
    w.resources.get_mut::<EmailConfiguration>().apply(settings);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_written_with_defaults_and_read_back() {
        let dir = std::env::temp_dir().join(format!("muo_email_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("Configuration")).unwrap();
        let mut w = World::new();
        w.core.base_directory = dir.clone();
        configure(&mut w);
        let c = w.resources.get_mut::<EmailConfiguration>();
        assert!(!c.email_enabled);
        assert_eq!(c.email_port, 465);
        assert_eq!(
            c.crash_address,
            MailboxAddress {
                name: "Crash Log".into(),
                address: "".into()
            }
        );
        let text = std::fs::read_to_string(dir.join(REL_PATH)).unwrap();
        assert!(
            text.contains("\"emailSendRetryDelay\": 3") && text.contains("\"fromName\": \"\""),
            "{text}"
        );
        // An edited file wins on the next start.
        std::fs::write(
            dir.join(REL_PATH),
            r#"{ "enabled": true, "emailPort": 25 }"#,
        )
        .unwrap();
        configure(&mut w);
        let c = w.resources.get_mut::<EmailConfiguration>();
        assert!(c.email_enabled && c.email_port == 25 && c.email_server == "smtp.gmail.com");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
