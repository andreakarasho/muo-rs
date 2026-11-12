//! `SkillInfo` and `Stat`, and the loading of `Data/skills.json`.
//!
//! The `Callback` (`SkillUseCallback`, the skill's use handler) is not data; muo_content binds it by skill id.
//! `Skills`/`Skill` hold a `Mobile`'s values and stay with muo_server.

use crate::error::{DataError, Result};
use crate::json;
use serde::Deserialize;
use std::path::Path;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize)]
pub enum Stat {
    #[serde(alias = "str", alias = "STR")]
    Str,
    #[serde(alias = "dex", alias = "DEX")]
    Dex,
    #[serde(alias = "int", alias = "INT")]
    Int,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkillInfo {
    pub skill_id: i32,
    pub name: String,
    pub title: String,
    /// The JSON value divided by 100.
    pub str_scale: f64,
    pub dex_scale: f64,
    pub int_scale: f64,
    pub stat_total: f64,
    pub str_gain: f64,
    pub dex_gain: f64,
    pub int_gain: f64,
    pub gain_factor: f64,
    pub profession_skill_name: String,
    pub primary_stat: Stat,
    pub secondary_stat: Stat,
}

/// The JSON constructor's parameters plus the settable `StatTotal`.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase", default)]
struct Raw {
    #[serde(rename = "SkillID")]
    skill_id: i32,
    name: String,
    title: String,
    str_scale: f64,
    dex_scale: f64,
    int_scale: f64,
    stat_total: Option<f64>,
    str_gain: f64,
    dex_gain: f64,
    int_gain: f64,
    gain_factor: f64,
    profession_skill_name: Option<String>,
    primary_stat: Stat,
    secondary_stat: Stat,
}

impl Default for Raw {
    fn default() -> Self {
        Raw {
            skill_id: 0,
            name: String::new(),
            title: String::new(),
            str_scale: 0.0,
            dex_scale: 0.0,
            int_scale: 0.0,
            stat_total: None,
            str_gain: 0.0,
            dex_gain: 0.0,
            int_gain: 0.0,
            gain_factor: 0.0,
            profession_skill_name: None,
            primary_stat: Stat::Str,
            secondary_stat: Stat::Str,
        }
    }
}

impl From<Raw> for SkillInfo {
    fn from(r: Raw) -> Self {
        SkillInfo {
            skill_id: r.skill_id,
            // `Name.RemoveOrdinal(" ")`
            profession_skill_name: r
                .profession_skill_name
                .unwrap_or_else(|| r.name.replace(' ', "")),
            title: r.title,
            str_scale: r.str_scale / 100.0,
            dex_scale: r.dex_scale / 100.0,
            int_scale: r.int_scale / 100.0,
            // computed from the raw scales, then overwritten by a "StatTotal" property when the file has one
            stat_total: r
                .stat_total
                .unwrap_or(r.str_scale + r.dex_scale + r.int_scale),
            str_gain: r.str_gain,
            dex_gain: r.dex_gain,
            int_gain: r.int_gain,
            gain_factor: r.gain_factor,
            name: r.name,
            primary_stat: r.primary_stat,
            secondary_stat: r.secondary_stat,
        }
    }
}

impl SkillInfo {
    pub fn from_json(text: &str) -> serde_json::Result<Vec<SkillInfo>> {
        Ok(json::from_str::<Vec<Raw>>(text)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// `SkillInfo.Table = JsonConfig.Deserialize<SkillInfo[]>("Data/skills.json")`.
    pub fn load_table(base_dir: &Path) -> Result<Vec<SkillInfo>> {
        let path = base_dir.join("Data/skills.json");
        let raw = json::deserialize_file::<Vec<Raw>>(&path)?
            .ok_or_else(|| DataError::NotFound(path.display().to_string()))?;
        Ok(raw.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_rules() {
        let t = SkillInfo::from_json(
            r#"[{"SkillID": 0, "Name": "Animal Lore", "Title": "Biologist", "StrScale": 50, "DexScale": 25,
                 "IntScale": 25, "StrGain": 0.1, "GainFactor": 1, "PrimaryStat": "Int", "SecondaryStat": "dex"},
                {"SkillID": 1, "Name": "A", "StatTotal": 7, "ProfessionSkillName": "Other", "PrimaryStat": "Str", "SecondaryStat": "Str"}]"#,
        )
        .unwrap();
        assert_eq!(t[0].profession_skill_name, "AnimalLore");
        assert_eq!((t[0].str_scale, t[0].dex_scale), (0.5, 0.25));
        assert_eq!(t[0].stat_total, 100.0);
        assert_eq!(
            (t[0].primary_stat, t[0].secondary_stat),
            (Stat::Int, Stat::Dex)
        );
        assert_eq!(
            (t[1].stat_total, t[1].profession_skill_name.as_str()),
            (7.0, "Other")
        );
        assert!(SkillInfo::from_json(r#"[{"PrimaryStat": "Wis"}]"#).is_err());
    }

    #[test]
    fn the_distribution_file_loads() {
        let root =
            std::path::PathBuf::from(std::env::var_os("MUO_DISTRIBUTION").unwrap_or_default());
        if !root.join("Data/skills.json").exists() {
            return;
        }
        let t = SkillInfo::load_table(&root).unwrap();
        assert_eq!(t.len(), 58);
        assert_eq!(t[0].name, "Alchemy");
        assert_eq!(t[0].primary_stat, Stat::Int);
        assert_eq!(t[0].stat_total, 10.0);
        assert!(t.iter().enumerate().all(|(i, s)| s.skill_id == i as i32));
    }
}
