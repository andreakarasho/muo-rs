//! The data half of the map loader: the entries of `Data/map-definitions.json`. Building the `Map`
//! objects from them (`RegisterMap`, the clamp of width and height to the sector size) is muo_server's.

use crate::error::{DataError, Result};
use crate::json;
use bitflags::bitflags;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::Path;

bitflags! {
    /// `MapRules`.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct MapRules: u32 {
        /// Internal map (used for dragging, commodity deeds, etc)
        const INTERNAL = 0x1;
        /// Anyone can move over anyone else without taking stamina loss
        const FREE_MOVEMENT = 0x2;
        /// Disallow performing beneficial actions on criminals/murderers
        const BENEFICIAL_RESTRICTIONS = 0x4;
        /// Disallow performing harmful actions on innocents
        const HARMFUL_RESTRICTIONS = 0x8;
        const TRAMMEL_RULES = 0xE;
    }
}

const RULE_NAMES: [(&str, u32); 7] = [
    ("None", 0),
    ("Internal", 1),
    ("FreeMovement", 2),
    ("BeneficialRestrictions", 4),
    ("HarmfulRestrictions", 8),
    ("TrammelRules", 0xE),
    ("FeluccaRules", 0),
];

impl MapRules {
    /// `JsonStringEnumConverter` reading: comma separated names (case-insensitive) or a number.
    pub fn parse(s: &str) -> Option<MapRules> {
        if let Ok(n) = s.trim().parse::<u32>() {
            return Some(MapRules::from_bits_retain(n));
        }
        let mut bits = 0;
        for part in s.split(',') {
            bits |= RULE_NAMES
                .iter()
                .find(|n| n.0.eq_ignore_ascii_case(part.trim()))?
                .1;
        }
        Some(MapRules::from_bits_retain(bits))
    }
}

impl std::fmt::Display for MapRules {
    /// `Enum.ToString()`: the matching name, else the single bits joined by ", ".
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bits = self.bits();
        if let Some(n) = RULE_NAMES
            .iter()
            .rev()
            .find(|n| n.1 == bits && (bits != 0 || n.0 == "None"))
        {
            return f.write_str(n.0);
        }
        let parts: Vec<&str> = [1u32, 2, 4, 8]
            .iter()
            .filter(|b| bits & **b != 0)
            .filter_map(|b| RULE_NAMES.iter().find(|n| n.1 == *b).map(|n| n.0))
            .collect();
        if parts.is_empty() || bits & !0xF != 0 {
            write!(f, "{bits}")
        } else {
            f.write_str(&parts.join(", "))
        }
    }
}

impl Serialize for MapRules {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for MapRules {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        use serde::de::Error;
        let s = String::deserialize(d)?;
        MapRules::parse(&s).ok_or_else(|| {
            D::Error::custom(format!(
                "The JSON value '{s}' could not be converted to MapRules"
            ))
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapDefinition {
    /// Unique index in `Map.Maps`; 0-31 and 127, 255 are reserved for the core.
    pub index: i32,
    /// Id used in client communications; 0-5 for visible maps.
    pub id: i32,
    /// Map file number (`mapN.mul`); 0-5 for visible maps.
    #[serde(rename = "fileIndex")]
    pub file_index: i32,
    pub name: String,
    /// In tiles; -1 for the internal map.
    pub width: i32,
    pub height: i32,
    /// 0 Spring, 1 Summer, 2 Fall, 3 Winter, 4 Desolation.
    pub season: i32,
    pub rules: MapRules,
    /// How many maps this entry makes (1 unless given): copies take the next indexes and are named
    /// `<name><index>`. Private instances of one place, such as a hideout per player.
    pub count: i32,
}

impl Default for MapDefinition {
    fn default() -> Self {
        MapDefinition {
            index: 0,
            id: 0,
            file_index: 0,
            name: String::new(),
            width: 0,
            height: 0,
            season: 0,
            rules: MapRules::empty(),
            count: 1,
        }
    }
}

/// Reads `Data/map-definitions.json` under `base_dir`.
pub fn load_map_definitions(base_dir: &Path) -> Result<Vec<MapDefinition>> {
    let path = base_dir.join("Data/map-definitions.json");
    json::deserialize_file(&path)?
        .ok_or_else(|| DataError::NotFound(format!("Failed to deserialize {}.", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_names() {
        assert_eq!(
            MapRules::parse("TrammelRules"),
            Some(MapRules::TRAMMEL_RULES)
        );
        assert_eq!(MapRules::parse("feluccarules"), Some(MapRules::empty()));
        assert_eq!(
            MapRules::parse("Internal, FreeMovement"),
            Some(MapRules::from_bits_retain(3))
        );
        assert_eq!(MapRules::parse("Nope"), None);
        assert_eq!(MapRules::TRAMMEL_RULES.to_string(), "TrammelRules");
        assert_eq!(MapRules::empty().to_string(), "None");
        assert_eq!(
            MapRules::from_bits_retain(3).to_string(),
            "Internal, FreeMovement"
        );
    }

    #[test]
    fn the_distribution_file_loads() {
        let root =
            std::path::PathBuf::from(std::env::var_os("MUO_DISTRIBUTION").unwrap_or_default());
        if !root.join("Data/map-definitions.json").exists() {
            return;
        }
        let maps = load_map_definitions(&root).unwrap();
        assert_eq!(maps.len(), 7);
        assert_eq!(
            (maps[1].name.as_str(), maps[1].rules),
            ("Trammel", MapRules::TRAMMEL_RULES)
        );
        assert_eq!(maps[6].index, 127);
        assert_eq!(maps[6].rules, MapRules::INTERNAL);
    }
}
