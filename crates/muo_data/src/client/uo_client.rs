//! `UOClient`: which client version the data files belong to.

use super::ClientVersion;
use crate::configuration::ServerConfiguration;
use crate::data_directories::DataDirectories;
use crate::error::{DataError, Result};
use crate::json;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// ClassicUO's `settings.json`, the two properties read.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CuoSettings {
    #[serde(
        rename = "clientversion",
        with = "crate::json::converters::client_version::option",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub client_version: Option<ClientVersion>,
    #[serde(
        rename = "ultimaonlinedirectory",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub ultima_online_directory: Option<String>,
}

/// `UOClient`'s state: the server's client version and the limits protocol code sets.
#[derive(Clone, Debug, Default)]
pub struct UoClient {
    pub cuo_settings: Option<CuoSettings>,
    pub server_client_version: Option<ClientVersion>,
    pub min_required: Option<ClientVersion>,
    pub max_required: Option<ClientVersion>,
    automatically_detected: bool,
}

impl UoClient {
    /// `Load`: `clientData.clientVersion` if configured, else a ClassicUO `settings.json` in the data directories,
    /// else the version resource of `client.exe`.
    pub fn load(&mut self, config: &ServerConfiguration) -> Result<()> {
        self.server_client_version =
            config.get_setting_opt::<ClientVersion>("clientData.clientVersion");
        if self.server_client_version.is_none() {
            let dirs = config.data_directories();
            let (cuo, version) = detect_cuo_client(dirs)?;
            self.cuo_settings = cuo;
            self.server_client_version = match version {
                Some(v) => Some(v),
                None => detect_classic_client(dirs)?,
            };
            self.automatically_detected = true;
        }
        Ok(())
    }

    /// `Configure`: reports what was chosen.
    pub fn configure(&self) {
        let Some(v) = &self.server_client_version else {
            tracing::warn!(
                "Could not detect client version. This may cause data files to load improperly."
            );
            return;
        };
        if self.automatically_detected {
            let from_cuo = self
                .cuo_settings
                .as_ref()
                .and_then(|c| c.client_version.as_ref())
                == Some(v);
            if from_cuo {
                tracing::info!("Automatically detected client version {v} from CUO settings.");
            } else {
                tracing::info!("Automatically detected client version {v}");
            }
            return;
        }
        tracing::info!("Manually configured to use client version {v}");
    }
}

/// `DetectCUOClient`: `settings.json` of a ClassicUO install. Returns its settings (only when its
/// `ultimaonlinedirectory` exists, resolved against the file's folder) and its `clientversion`.
pub fn detect_cuo_client(
    dirs: &DataDirectories,
) -> Result<(Option<CuoSettings>, Option<ClientVersion>)> {
    let Some(path) = dirs.find("settings.json") else {
        return Ok((None, None));
    };
    let Some(mut settings) = json::deserialize_file::<CuoSettings>(&path)? else {
        return Ok((None, None));
    };
    let version = settings.client_version.clone();
    let mut kept = None;
    if let Some(dir) = &settings.ultima_online_directory {
        let base = path.parent().unwrap_or(Path::new("."));
        let full = muo_primitives::path_utility::get_full_path(dir, base);
        if full.is_dir() {
            settings.ultima_online_directory = Some(full.to_string_lossy().into_owned());
            kept = Some(settings);
        }
    }
    Ok((kept, version))
}

/// `DetectClassicClient`: the file version of `client.exe`, read from its `VS_VERSION_INFO` resource.
pub fn detect_classic_client(dirs: &DataDirectories) -> Result<Option<ClientVersion>> {
    let Some(path) = dirs.find("client.exe") else {
        return Ok(None);
    };
    let buffer = std::fs::read(&path).map_err(|e| DataError::io(&path, e))?;
    Ok(version_from_exe(&buffer))
}

/// The four 16-bit version parts after the UTF-16 `VS_VERSION_INFO` marker (30 + 12 bytes in).
pub fn version_from_exe(buffer: &[u8]) -> Option<ClientVersion> {
    const MARKER: &[u8] = b"V\0S\0_\0V\0E\0R\0S\0I\0O\0N\0_\0I\0N\0F\0O\0";
    let index = buffer.windows(MARKER.len()).position(|w| w == MARKER)?;
    let at = index + 42;
    let part = |o: usize| {
        buffer
            .get(at + o..at + o + 2)
            .map(|b| i32::from(u16::from_le_bytes([b[0], b[1]])))
    };
    let (minor, major, private, build) = (part(0)?, part(2)?, part(4)?, part(6)?);
    Some(ClientVersion::new(
        major,
        minor,
        build,
        private,
        super::ClientType::Classic,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exe_version_resource() {
        let mut exe = vec![0u8; 100];
        exe.extend_from_slice(b"V\0S\0_\0V\0E\0R\0S\0I\0O\0N\0_\0I\0N\0F\0O\0");
        exe.extend_from_slice(&[0u8; 12]);
        // minor, major, private, build = 0, 7, 65, 4 => 7.0.4.65
        for v in [0u16, 7, 65, 4] {
            exe.extend_from_slice(&v.to_le_bytes());
        }
        let v = version_from_exe(&exe).unwrap();
        assert_eq!((v.major, v.minor, v.revision, v.patch), (7, 0, 4, 65));
        assert!(version_from_exe(&exe[..120]).is_none());
        assert!(version_from_exe(b"MZ").is_none());
    }

    #[test]
    fn cuo_settings_json() {
        let s: CuoSettings = json::from_str(
            r#"{"clientversion": "7.0.9.0", "ultimaonlinedirectory": "C:\\UO", "other": 1}"#,
        )
        .unwrap();
        assert_eq!(s.client_version, Some(ClientVersion::v7090()));
        assert_eq!(s.ultima_online_directory.as_deref(), Some("C:\\UO"));
    }
}
