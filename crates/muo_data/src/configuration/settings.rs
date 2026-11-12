//! `ServerSettings`: the shape of `Configuration/muo.json`.

use crate::data_directories::DataDirectories;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::net::SocketAddr;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerSettings {
    #[serde(rename = "assemblyDirectories")]
    pub assembly_directories: Vec<String>,

    #[serde(rename = "dataDirectories")]
    pub data_directories: DataDirectories,

    #[serde(rename = "listeners", with = "listeners")]
    pub listeners: Vec<SocketAddr>,

    /// Every value is a string (or `null`, kept and written back as is; a reader sees it as the key's default);
    /// sorted by key with the culture comparer, like a `SortedDictionary<string, string>`.
    #[serde(rename = "settings", serialize_with = "serialize_sorted")]
    pub settings: BTreeMap<String, Option<String>>,

    /// `[JsonExtensionData]`: properties this version does not know (such as a legacy `expansion`).
    #[serde(flatten)]
    pub data: Map<String, Value>,
}

fn serialize_sorted<S: Serializer>(
    m: &BTreeMap<String, Option<String>>,
    s: S,
) -> Result<S::Ok, S::Error> {
    let mut entries: Vec<_> = m.iter().collect();
    entries.sort_by(|a, b| muo_primitives::dotnet::string_compare(a.0, b.0));
    s.collect_map(entries)
}

mod listeners {
    use super::*;
    use serde::de::Error;

    pub fn serialize<S: Serializer>(v: &[SocketAddr], s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(v.iter().map(ToString::to_string))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<SocketAddr>, D::Error> {
        Vec::<String>::deserialize(d)?
            .into_iter()
            .map(|s| {
                s.parse()
                    .map_err(|_| D::Error::custom("IPEndPoint must be in the correct format"))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_null_setting_is_written_back_as_null() {
        let s: ServerSettings = serde_json::from_str(r#"{"settings":{"b":"1","a":null}}"#).unwrap();
        assert_eq!(s.settings["a"], None);
        let out = serde_json::to_string(&s).unwrap();
        assert!(out.contains(r#""settings":{"a":null,"b":"1"}"#), "{out}");
    }
}
