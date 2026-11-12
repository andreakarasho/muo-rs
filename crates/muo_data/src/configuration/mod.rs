//! `ServerConfiguration`: the key/value settings of `Configuration/muo.json`.
//!
//! A `ServerConfiguration` is a value the host owns. Settings are strings in the file;
//! a typed read parses and falls back to the default when the key is missing or does not parse. `get_or_update_*`
//! writes a missing default back and saves.

pub mod prompts;
mod settings;

pub use prompts::{PromptIo, StdioPrompt};
pub use settings::ServerSettings;

use crate::client::{ClientVersion, UoClient};
use crate::data_directories::DataDirectories;
use crate::error::{DataError, Result};
use crate::expansion::{EXPANSION_CONFIGURATION_PATH, Expansion, ExpansionTable};
use crate::json;
use muo_primitives::TimeSpan;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

pub const CONFIGURATION_FILE_PATH: &str = "Configuration/muo.json";

/// A type that can live in a settings string. Implement it for your own enums (`Enum.TryParse` / `ToString`).
pub trait SettingValue: Sized {
    /// `None` when the text is not a valid value (the default is used then).
    fn parse_setting(s: &str) -> Option<Self>;
    fn to_setting(&self) -> String;
}

impl SettingValue for i32 {
    fn parse_setting(s: &str) -> Option<Self> {
        s.trim().parse().ok()
    }

    fn to_setting(&self) -> String {
        self.to_string()
    }
}

impl SettingValue for i64 {
    fn parse_setting(s: &str) -> Option<Self> {
        s.trim().parse().ok()
    }

    fn to_setting(&self) -> String {
        self.to_string()
    }
}

/// `bool.TryParse`: "true"/"false" in any case, whitespace allowed. Written as `True`/`False`.
impl SettingValue for bool {
    fn parse_setting(s: &str) -> Option<Self> {
        let t = s.trim();
        if t.eq_ignore_ascii_case("true") {
            Some(true)
        } else if t.eq_ignore_ascii_case("false") {
            Some(false)
        } else {
            None
        }
    }

    fn to_setting(&self) -> String {
        if *self { "True" } else { "False" }.to_string()
    }
}

impl SettingValue for f64 {
    fn parse_setting(s: &str) -> Option<Self> {
        s.trim().parse().ok()
    }

    /// `double.ToString()`: the shortest round-trip digits, in `E+17`/`E-05` style once the decimal exponent leaves
    /// -4..=16 (1e-4 and 1e15..1e16 print in full, 9e-5 and 1.2345678901234568e17 do not).
    fn to_setting(&self) -> String {
        let v = *self;
        if v.is_nan() {
            return "NaN".into();
        }
        if v.is_infinite() {
            return if v > 0.0 { "Infinity" } else { "-Infinity" }.into();
        }
        let s = format!("{v:e}");
        let (mantissa, exp) = s.split_once('e').unwrap_or((&s, "0"));
        let exp: i32 = exp.parse().unwrap_or(0);
        if v == 0.0 || (-4..=16).contains(&exp) {
            return format!("{v}");
        }
        format!(
            "{mantissa}E{}{:02}",
            if exp < 0 { '-' } else { '+' },
            exp.abs()
        )
    }
}

impl SettingValue for TimeSpan {
    fn parse_setting(s: &str) -> Option<Self> {
        TimeSpan::try_parse(s)
    }

    fn to_setting(&self) -> String {
        self.to_string()
    }
}

impl SettingValue for String {
    fn parse_setting(s: &str) -> Option<Self> {
        Some(s.to_string())
    }

    fn to_setting(&self) -> String {
        self.clone()
    }
}

/// `GetSetting(key, ClientVersion)`: any text is a version (an unparsable one is 0.0.0.0).
impl SettingValue for ClientVersion {
    fn parse_setting(s: &str) -> Option<Self> {
        Some(ClientVersion::parse(s))
    }

    fn to_setting(&self) -> String {
        self.to_string()
    }
}

impl SettingValue for Expansion {
    fn parse_setting(s: &str) -> Option<Self> {
        Expansion::parse(s)
    }

    fn to_setting(&self) -> String {
        self.name().to_string()
    }
}

#[derive(Debug)]
pub struct ServerConfiguration {
    settings: ServerSettings,
    file_path: PathBuf,
    mocked: bool,
}

impl ServerConfiguration {
    /// `Load`: reads `<base_dir>/Configuration/muo.json`, or starts empty when there is none. `mocked` (tests)
    /// never writes the file and skips the first-boot prompts.
    pub fn load(base_dir: &Path, mocked: bool) -> Result<ServerConfiguration> {
        let file_path = base_dir.join(CONFIGURATION_FILE_PATH);
        let settings = if file_path.exists() {
            tracing::info!("Reading server configuration from {CONFIGURATION_FILE_PATH}...");
            let s = json::deserialize_file::<ServerSettings>(&file_path)?.ok_or_else(|| {
                tracing::error!("Reading server configuration failed");
                DataError::NotFound(format!("Failed to deserialize {}.", file_path.display()))
            })?;
            tracing::info!("Reading server configuration done");
            s
        } else {
            ServerSettings::default()
        };
        Ok(ServerConfiguration {
            settings,
            file_path,
            mocked,
        })
    }

    /// A configuration that never touches disk (the test fixtures' `Load(mocked: true)` over no file).
    pub fn in_memory(settings: ServerSettings) -> ServerConfiguration {
        ServerConfiguration {
            settings,
            file_path: PathBuf::from(CONFIGURATION_FILE_PATH),
            mocked: true,
        }
    }

    pub fn settings(&self) -> &ServerSettings {
        &self.settings
    }

    pub fn assembly_directories(&self) -> &[String] {
        &self.settings.assembly_directories
    }

    pub fn data_directories(&self) -> &DataDirectories {
        &self.settings.data_directories
    }

    pub fn data_directories_mut(&mut self) -> &mut DataDirectories {
        &mut self.settings.data_directories
    }

    pub fn listeners(&self) -> &[SocketAddr] {
        &self.settings.listeners
    }

    /// `GetSetting`: the configured value, or `default` (not written back).
    pub fn get_setting<T: SettingValue>(&self, key: &str, default: T) -> T {
        self.get_setting_opt(key).unwrap_or(default)
    }

    /// `GetSetting<T>(key)` for enums: `None` when missing or not valid.
    pub fn get_setting_opt<T: SettingValue>(&self, key: &str) -> Option<T> {
        self.settings
            .settings
            .get(key)
            .and_then(|v| v.as_deref())
            .and_then(|v| T::parse_setting(v))
    }

    /// `GetOrUpdateSetting`: the configured value; a missing key is written with `default` and saved. A value that
    /// is present but invalid gives `default` without rewriting it.
    pub fn get_or_update_setting<T: SettingValue>(&mut self, key: &str, default: T) -> Result<T> {
        match self.settings.settings.get(key) {
            Some(v) => Ok(v
                .as_deref()
                .and_then(|v| T::parse_setting(v))
                .unwrap_or(default)),
            None => {
                self.set_setting(key, &default)?;
                Ok(default)
            }
        }
    }

    /// `GetOrUpdateSetting(key, string defaultValue)` with a null default: a missing key is written as null.
    pub fn get_or_update_nullable_setting(
        &mut self,
        key: &str,
        default: Option<&str>,
    ) -> Result<Option<String>> {
        if let Some(v) = self.settings.settings.get(key) {
            return Ok(v.clone());
        }
        self.settings
            .settings
            .insert(key.to_string(), default.map(str::to_string));
        self.save()?;
        Ok(default.map(str::to_string))
    }

    /// `SetSetting`: sets the value and saves.
    pub fn set_setting<T: SettingValue>(&mut self, key: &str, value: &T) -> Result<()> {
        self.settings
            .settings
            .insert(key.to_string(), Some(value.to_setting()));
        self.save()
    }

    /// `Save`: a no-op when mocked.
    pub fn save(&self) -> Result<()> {
        if self.mocked {
            return Ok(());
        }
        json::serialize_file(&self.file_path, &self.settings)
    }

    /// `ConfigurePrompts`: the first-boot questions (data directories, listeners, server name, expansion and maps),
    /// asked only for what is missing. Returns the current expansion (`Core.Expansion`), or `None` when mocked.
    /// `base_dir` holds `Configuration/`; `uo_client` is (re)loaded once the data directories are known.
    pub fn configure_prompts(
        &mut self,
        io: &mut dyn PromptIo,
        base_dir: &Path,
        expansions: &mut ExpansionTable,
        uo_client: &mut UoClient,
    ) -> Result<Option<Expansion>> {
        if self.mocked {
            return Ok(None);
        }
        let mut updated = false;

        if self.settings.data_directories.is_empty() {
            updated = true;
            for d in prompts::get_data_directories(io) {
                self.settings.data_directories.insert(d);
            }
        }

        uo_client.load(self)?;
        if let Some(dir) = uo_client
            .cuo_settings
            .as_ref()
            .and_then(|c| c.ultima_online_directory.clone())
        {
            self.settings.data_directories.insert(dir);
        }

        if self.settings.listeners.is_empty() {
            updated = true;
            self.settings.listeners.extend(prompts::get_listeners(io));
        }

        if !self
            .settings
            .settings
            .contains_key("serverListing.serverName")
        {
            updated = true;
            let name = prompts::get_server_name(io);
            self.settings
                .settings
                .insert("serverListing.serverName".into(), Some(name));
        }

        // We have a known, current expansion, so we can deserialize it from Configuration
        let current = match expansions.load_configuration(base_dir)? {
            Some(e) => e,
            None => {
                // an `expansion` property of an older muo.json is taken over
                let legacy = self
                    .settings
                    .data
                    .remove("expansion")
                    .and_then(|v| match v {
                        serde_json::Value::String(s) => Expansion::parse(&s),
                        serde_json::Value::Number(n) => {
                            n.as_i64().and_then(|n| Expansion::from_index(n as i32))
                        }
                        _ => None,
                    });
                let current = legacy.unwrap_or_else(|| prompts::get_expansion(io, expansions));
                // We've updated the selected expansion, so choose the maps we want from it, then store our selection
                let selected = prompts::get_selected_maps(io, expansions, current);
                expansions.store_map_selection(selected, current);
                updated = true;
                current
            }
        };

        if updated {
            self.save()?;
            io.write_line(&format!(
                "Server configuration saved to {CONFIGURATION_FILE_PATH}."
            ));
            // Either the expansion.json file has never existed, or it's been deleted, so create it now
            expansions.save_configuration(base_dir, current)?;
            io.write_line(&format!(
                "Expansion configuration saved to {EXPANSION_CONFIGURATION_PATH}."
            ));
        }
        Ok(Some(current))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expansion::MapSelectionFlags;
    use prompts::tests::{Script, table};

    /// The reference `double.ToString()` text for the bit patterns; the first column is `(double)(float)x` of
    /// `movementThrottle.suspiciousRateThreshold`-style settings (`GetOrUpdateSetting(key, double)`).
    #[test]
    fn double_setting_text_matches_reference() {
        let floats: [(u32, &str); 12] = [
            (1065772646, "1.0499999523162842"),
            (1066192077, "1.100000023841858"),
            (1056964608, "0.5"),
            (1065353216, "1"),
            (1036831949, "0.10000000149011612"),
            (925353388, "9.999999747378752E-06"),
            (1206984805, "123456.7890625"),
            (1482907561, "999999986991104"),
            (1510874058, "10000000272564224"),
            (2139095039, "3.4028234663852886E+38"),
            (1050253722, "0.30000001192092896"),
            (872714123, "1.234567861274627E-07"),
        ];
        for (bits, want) in floats {
            assert_eq!(f64::from(f32::from_bits(bits)).to_setting(), want, "{bits}");
        }
        let doubles: [(i64, &str); 14] = [
            (4547007122018943789, "0.0001"),
            (4546269252255995407, "9E-05"),
            (4548686525128629352, "0.00012345"),
            (4532020583610935537, "1E-05"),
            (4831355200913801216, "1000000000000000"),
            (4832797072101665536, "1234567890123456"),
            (4847542438873900484, "12345678901234568"),
            (4862596447618666293, "1.2345678901234568E+17"),
            (9097811302482466869, "1.5E+300"),
            (4906019910204099648, "1E+20"),
            (1, "5E-324"),
            (4562254508917369340, "0.001"),
            (-4392016835940974592, "-1000000000000000"),
            (-4677102784598780401, "-9E-05"),
        ];
        for (bits, want) in doubles {
            assert_eq!(f64::from_bits(bits as u64).to_setting(), want, "{bits}");
        }
    }

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("muo_cfg_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn write_config(dir: &Path, text: &str) {
        std::fs::create_dir_all(dir.join("Configuration")).unwrap();
        std::fs::write(dir.join(CONFIGURATION_FILE_PATH), text).unwrap();
    }

    #[test]
    fn typed_reads_fall_back_to_the_default() {
        let dir = temp("typed");
        write_config(
            &dir,
            r#"{ "settings": { "a.int": "42", "a.bad": "x", "a.bool": "true", "a.span": "00:05:00", "a.d": "1.5",
                 "a.exp": "ML", "a.cv": "7.0.9.0", "a.s": "text" } }"#,
        );
        let c = ServerConfiguration::load(&dir, true).unwrap();
        assert_eq!(c.get_setting("a.int", 1), 42);
        assert_eq!(c.get_setting("a.bad", 7), 7);
        assert_eq!(c.get_setting("a.missing", 7i64), 7);
        assert!(c.get_setting("a.bool", false));
        assert_eq!(
            c.get_setting("a.span", TimeSpan::ZERO),
            TimeSpan::from_mins(5)
        );
        assert_eq!(c.get_setting("a.d", 0.0), 1.5);
        assert_eq!(c.get_setting("a.exp", Expansion::None), Expansion::ML);
        assert_eq!(c.get_setting_opt::<Expansion>("a.missing"), None);
        assert_eq!(
            c.get_setting("a.cv", ClientVersion::v6000()),
            ClientVersion::v7090()
        );
        assert_eq!(c.get_setting("a.s", String::new()), "text");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_or_update_writes_missing_defaults_and_keeps_the_file_shape() {
        let dir = temp("update");
        write_config(
            &dir,
            r#"{ // comment
              "assemblyDirectories": ["./Assemblies"],
              "dataDirectories": ["C:\\UO"],
              "listeners": ["0.0.0.0:2593"],
              "settings": { "b.x": "1" },
              "expansion": "SA",
            }"#,
        );
        let mut c = ServerConfiguration::load(&dir, false).unwrap();
        assert_eq!(c.get_or_update_setting("b.x", 5).unwrap(), 1);
        assert!(c.get_or_update_setting("a.enabled", true).unwrap());
        assert_eq!(
            c.get_or_update_setting("a.delay", TimeSpan::from_mins(5))
                .unwrap(),
            TimeSpan::from_mins(5)
        );
        c.set_setting("a.rate", &0.25).unwrap();

        let saved = std::fs::read_to_string(dir.join(CONFIGURATION_FILE_PATH)).unwrap();
        let v: serde_json::Value = serde_json::from_str(&saved).unwrap();
        assert_eq!(v["settings"]["a.enabled"], "True");
        assert_eq!(v["settings"]["a.delay"], "00:05:00");
        assert_eq!(v["settings"]["a.rate"], "0.25");
        assert_eq!(v["listeners"][0], "0.0.0.0:2593");
        assert_eq!(v["dataDirectories"][0], "C:\\UO");
        assert_eq!(v["expansion"], "SA"); // extension data survives
        let keys: Vec<&str> = v["settings"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["a.delay", "a.enabled", "a.rate", "b.x"]); // sorted

        // reloading gives the same
        let again = ServerConfiguration::load(&dir, true).unwrap();
        assert_eq!(again.settings(), c.settings());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mocked_never_writes() {
        let dir = temp("mock");
        let mut c = ServerConfiguration::load(&dir, true).unwrap();
        c.set_setting("a", &1).unwrap();
        assert!(!dir.join(CONFIGURATION_FILE_PATH).exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_corrupt_file_is_an_error() {
        let dir = temp("corrupt");
        write_config(&dir, "{ nope");
        assert!(ServerConfiguration::load(&dir, true).is_err());
        write_config(&dir, "null");
        assert!(ServerConfiguration::load(&dir, true).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn double_text_matches_reference() {
        assert_eq!(1.1f64.to_setting(), "1.1");
        assert_eq!(1.0f64.to_setting(), "1");
        assert_eq!(1.100000023841858f64.to_setting(), "1.100000023841858");
        assert_eq!(1e15f64.to_setting(), "1000000000000000"); // switches at 1E+17
        assert_eq!(1e17f64.to_setting(), "1E+17");
        assert_eq!(1.5e-7f64.to_setting(), "1.5E-07");
        assert_eq!(f64::NAN.to_setting(), "NaN");
    }

    #[test]
    fn first_boot_prompts_ask_only_for_what_is_missing() {
        let dir = temp("boot");
        let here = std::env::temp_dir().to_string_lossy().to_string();
        let mut c = ServerConfiguration::load(&dir, false).unwrap();
        let mut table = table();
        let mut client = UoClient::default();
        // data dir, finish; listener default, finish; shard name; expansion 7 (ML); accept maps
        let mut io = Script::new(&[&here, "", "", "", "Test Shard", "7", ""]);
        let current = c
            .configure_prompts(&mut io, &dir, &mut table, &mut client)
            .unwrap();
        assert_eq!(current, Some(Expansion::ML));
        assert_eq!(c.listeners(), ["0.0.0.0:2593".parse().unwrap()]);
        assert_eq!(
            c.get_setting("serverListing.serverName", String::new()),
            "Test Shard"
        );
        assert!(
            dir.join(CONFIGURATION_FILE_PATH).exists()
                && dir.join(EXPANSION_CONFIGURATION_PATH).exists()
        );

        // second boot: everything is known, nothing is asked
        let mut c2 = ServerConfiguration::load(&dir, false).unwrap();
        let mut table2 = crate::configuration::prompts::tests::table();
        let mut io2 = Script::default();
        let current = c2
            .configure_prompts(&mut io2, &dir, &mut table2, &mut client)
            .unwrap();
        assert_eq!(current, Some(Expansion::ML));
        assert!(io2.output.is_empty());
        assert_eq!(
            table2.get_info(7).map_selection_flags,
            MapSelectionFlags::FELUCCA | MapSelectionFlags::TRAMMEL
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
