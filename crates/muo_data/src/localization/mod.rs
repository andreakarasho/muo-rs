//! `Localization`: cliloc tables by language.
//!
//! A `Localization` value owns the tables. Looking up a language that is not loaded reads
//! `cliloc.<lang>` from the data directories then, so lookups take `&mut self`.
//!
//! // shortcut: `Configure()` with `_loadLocalizationOnStartup = false` (a constant) loaded nothing; it is left
//! out. `load_cliloc_file` is public for a host that wants to preload.

mod entry;

pub use entry::{FormatArg, LocalizationEntry};

use crate::client::bwt;
use crate::data_directories::DataDirectories;
use crate::error::{DataError, Result};
use indexmap::IndexMap;
use muo_primitives::dotnet::InvariantCase;
use std::collections::HashMap;
use std::path::Path;

pub const FALLBACK_LANGUAGE: &str = "enu";

pub type Entries = IndexMap<i32, LocalizationEntry>;

#[derive(Debug, Default)]
pub struct Localization {
    dirs: DataDirectories,
    localizations: HashMap<String, Entries>,
}

fn lang_key(lang: Option<&str>) -> String {
    lang.map_or_else(|| FALLBACK_LANGUAGE.to_string(), |s| s.lower_inv())
}

impl Localization {
    pub fn new(dirs: DataDirectories) -> Self {
        Localization {
            dirs,
            localizations: HashMap::new(),
        }
    }

    /// `Add`: errors if the number already exists in that language (like `Dictionary.Add`).
    pub fn add(&mut self, lang: Option<&str>, number: i32, text: &str) -> Result<()> {
        let lang = lang_key(lang);
        let entry = LocalizationEntry::new(lang.clone(), number, text);
        let entries = self.localizations.entry(lang).or_default();
        if entries.contains_key(&number) {
            return Err(DataError::invalid(
                "localization",
                format!("An item with the same key ({number}) has already been added."),
            ));
        }
        entries.insert(number, entry);
        Ok(())
    }

    /// `Remove`. The fallback table stays registered even when empty, so a later `add` is visible to fallback lookups.
    pub fn remove(&mut self, lang: Option<&str>, number: i32) -> bool {
        let lang = lang_key(lang);
        let Some(entries) = self.localizations.get_mut(&lang) else {
            return false;
        };
        if entries.shift_remove(&number).is_none() {
            return false;
        }
        if entries.is_empty() && lang != FALLBACK_LANGUAGE {
            self.localizations.remove(&lang);
        }
        true
    }

    pub fn clear(&mut self) {
        self.localizations.clear();
    }

    /// `LoadClilocs(lang)`: reads `cliloc.<lang>` (a language without a file loads as an empty table) and replaces
    /// any table already there.
    pub fn load_cliloc(&mut self, lang: Option<&str>) -> Result<&Entries> {
        let lang = lang_key(lang);
        let entries = match self.dirs.find(&format!("cliloc.{lang}")) {
            Some(path) => load_cliloc_file(&lang, &path)?,
            None => Entries::new(),
        };
        Ok(self
            .localizations
            .entry(lang)
            .insert_entry(entries)
            .into_mut())
    }

    /// `TryGetLocalization(lang, number)`: the language's entry, else the fallback language's.
    pub fn try_get_localization(
        &mut self,
        lang: Option<&str>,
        number: i32,
    ) -> Result<Option<&LocalizationEntry>> {
        let lang = lang_key(lang);
        if lang != FALLBACK_LANGUAGE {
            if !self.localizations.contains_key(&lang) {
                self.load_cliloc(Some(&lang))?;
            }
            if self.localizations[&lang].contains_key(&number) {
                return Ok(self.localizations[&lang].get(&number));
            }
        }
        if !self.localizations.contains_key(FALLBACK_LANGUAGE) {
            self.load_cliloc(Some(FALLBACK_LANGUAGE))?;
        }
        Ok(self.localizations[FALLBACK_LANGUAGE].get(&number))
    }

    /// `GetText`: the original text, `None` when there is no such entry.
    pub fn get_text(&mut self, number: i32, lang: Option<&str>) -> Result<Option<&str>> {
        Ok(self
            .try_get_localization(lang, number)?
            .map(|e| e.text.as_str()))
    }

    /// `Localization.Format(number, lang, $"...")`: empty when the cliloc does not exist.
    pub fn format(
        &mut self,
        number: i32,
        lang: Option<&str>,
        args: &[FormatArg],
    ) -> Result<String> {
        let lang = lang_key(lang);
        let Some(entry) = self.try_get_localization(Some(&lang), number)?.cloned() else {
            return Ok(String::new());
        };
        // Arguments that are themselves clilocs are looked up in the same language; a failed table load there is
        // treated as a missing entry.
        let mut resolve = |n: i32| {
            self.try_get_localization(Some(&lang), n)
                .ok()
                .flatten()
                .map(|e| e.text.clone())
        };
        Ok(entry.format(args, &mut resolve))
    }
}

/// Reads one `cliloc.<lang>` file. The plain layout is a 6 byte header (`2i32`, `1i16`) and records of
/// `number:i32, flag:u8, length:i16, utf8 text`; the 7.0.x client's files are the same behind a 4 byte header and
/// BWT compression. Corrupt files are errors.
pub fn load_cliloc_file(lang: &str, path: &Path) -> Result<Entries> {
    let bytes = std::fs::read(path).map_err(|e| DataError::io(path, e))?;
    parse_cliloc(lang, &bytes).map_err(|e| match e {
        DataError::InvalidData { reason, .. } => DataError::invalid(path.to_string_lossy(), reason),
        other => other,
    })
}

pub fn parse_cliloc(lang: &str, bytes: &[u8]) -> Result<Entries> {
    let bad = |why: &str| DataError::invalid("cliloc", why);
    let header_ok = |h: &[u8]| {
        h.len() >= 6
            && i32::from_le_bytes([h[0], h[1], h[2], h[3]]) == 2
            && i16::from_le_bytes([h[4], h[5]]) == 1
    };

    let decompressed;
    let data: &[u8] = if header_ok(bytes) {
        &bytes[6..]
    } else {
        // Skip header
        decompressed =
            bwt::decompress(bytes.get(4..).ok_or_else(|| bad("Invalid cliloc header"))?)?;
        if !header_ok(&decompressed) {
            return Err(bad("Invalid cliloc header"));
        }
        &decompressed[6..]
    };

    let mut entries = Entries::new();
    let mut pos = 0;
    while pos < data.len() {
        let head = data
            .get(pos..pos + 7)
            .ok_or_else(|| bad("Unexpected end of file."))?;
        let number = i32::from_le_bytes([head[0], head[1], head[2], head[3]]);
        // head[4] is the flag: Original, Custom, Modified
        let length = i16::from_le_bytes([head[5], head[6]]);
        let length = usize::try_from(length).map_err(|_| bad("Negative text length."))?;
        let text = data
            .get(pos + 7..pos + 7 + length)
            .ok_or_else(|| bad("Unexpected end of file."))?;
        entries.insert(
            number,
            LocalizationEntry::new(lang, number, String::from_utf8_lossy(text)),
        );
        pos += 7 + length;
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORMAT_NUMBER: i32 = 500002;
    const PARAMETER_NUMBER: i32 = 500003;

    #[test]
    fn cliloc_as_parameter() {
        let mut l = Localization::default();
        l.add(
            Some("enu"),
            FORMAT_NUMBER,
            "This tests ~1_NUMBER~ as parameters.",
        )
        .unwrap();
        l.add(Some("enu"), PARAMETER_NUMBER, "clilocs").unwrap();

        let numeric = l
            .format(
                FORMAT_NUMBER,
                Some("enu"),
                &[FormatArg::Cliloc(i64::from(PARAMETER_NUMBER))],
            )
            .unwrap();
        let string_param = l
            .format(FORMAT_NUMBER, Some("enu"), &["#500003".into()])
            .unwrap();

        assert_eq!(numeric, "This tests clilocs as parameters.");
        assert_eq!(string_param, "This tests clilocs as parameters.");
        assert_eq!(l.format(1, None, &[]).unwrap(), "");
    }

    #[test]
    fn removing_the_last_fallback_entry_keeps_later_adds_visible() {
        let mut l = Localization::default();
        l.add(Some("enu"), 500004, "first").unwrap();
        assert!(l.remove(Some("enu"), 500004));

        l.add(Some("enu"), 500005, "second").unwrap();

        assert!(l.try_get_localization(None, 500004).unwrap().is_none());
        assert_eq!(
            l.try_get_localization(None, 500005).unwrap().unwrap().text,
            "second"
        );
        assert!(l.add(Some("ENU"), 500005, "dup").is_err());
    }

    #[test]
    fn other_languages_fall_back_to_enu() {
        let mut l = Localization::default();
        l.add(Some("enu"), 1, "english").unwrap();
        l.add(Some("deu"), 2, "deutsch").unwrap();
        assert_eq!(l.get_text(1, Some("deu")).unwrap(), Some("english"));
        assert_eq!(l.get_text(2, Some("DEU")).unwrap(), Some("deutsch"));
        assert_eq!(l.get_text(2, None).unwrap(), None);
        assert!(l.remove(Some("deu"), 2));
        assert!(!l.remove(Some("deu"), 2));
    }

    fn record(number: i32, text: &str) -> Vec<u8> {
        let mut r = number.to_le_bytes().to_vec();
        r.push(0);
        r.extend_from_slice(&(text.len() as i16).to_le_bytes());
        r.extend_from_slice(text.as_bytes());
        r
    }

    #[test]
    fn parses_the_plain_file_layout_and_rejects_corruption() {
        let mut file = 2i32.to_le_bytes().to_vec();
        file.extend_from_slice(&1i16.to_le_bytes());
        file.extend(record(1000000, "Hello ~1_NAME~"));
        file.extend(record(1000001, "caf\u{e9}"));
        let e = parse_cliloc("enu", &file).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(e[&1000001].text, "caf\u{e9}");
        assert_eq!(e[&1000000].string_formatter.as_deref(), Some("Hello {0}"));

        assert!(parse_cliloc("enu", &file[..file.len() - 1]).is_err()); // truncated text
        assert!(parse_cliloc("enu", &file[..9]).is_err()); // truncated record header
        assert!(parse_cliloc("enu", &[1, 2, 3]).is_err());
        assert!(parse_cliloc("enu", &[]).is_err());
        let mut neg = file[..6].to_vec();
        neg.extend_from_slice(&[0, 0, 0, 0, 0, 0xFF, 0xFF]);
        assert!(parse_cliloc("enu", &neg).is_err());
    }

    #[test]
    fn a_missing_language_file_is_an_empty_table() {
        let mut l = Localization::new(DataDirectories::default());
        assert_eq!(l.get_text(1, Some("zzz")).unwrap(), None);
    }

    /// Needs a real client: `MUO_CLIENT_DIR` is the UO directory holding `cliloc.enu` (plain or compressed).
    #[test]
    #[ignore = "needs the UO client files (set MUO_CLIENT_DIR)"]
    fn real_cliloc_enu_loads() {
        let dir = std::env::var("MUO_CLIENT_DIR").expect("MUO_CLIENT_DIR");
        let mut l = Localization::new(DataDirectories::new([dir]));
        assert_eq!(
            l.get_text(1005000, None).unwrap().map(str::is_empty),
            Some(false)
        );
    }
}
