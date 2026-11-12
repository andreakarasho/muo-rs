//! `ExpansionInfo` and `MapSelectionFlags`: the expansion
//! table of `Data/expansions.json` and the chosen expansion's `Configuration/expansion.json`.

use crate::client::{ClientFlags, ClientVersion};
use crate::error::{DataError, Result};
use crate::json::{self, converters};
use crate::json_flags;
use bitflags::bitflags;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Path, PathBuf};

pub const EXPANSION_CONFIGURATION_PATH: &str = "Configuration/expansion.json";

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(i32)]
pub enum Expansion {
    #[default]
    None,
    T2A,
    UOR,
    UOTD,
    LBR,
    AOS,
    SE,
    ML,
    SA,
    HS,
    TOL,
    EJ,
}

const EXPANSIONS: [Expansion; 12] = [
    Expansion::None,
    Expansion::T2A,
    Expansion::UOR,
    Expansion::UOTD,
    Expansion::LBR,
    Expansion::AOS,
    Expansion::SE,
    Expansion::ML,
    Expansion::SA,
    Expansion::HS,
    Expansion::TOL,
    Expansion::EJ,
];

impl Expansion {
    pub fn from_index(i: i32) -> Option<Expansion> {
        usize::try_from(i)
            .ok()
            .and_then(|i| EXPANSIONS.get(i).copied())
    }

    pub fn index(self) -> i32 {
        self as i32
    }

    pub fn name(self) -> &'static str {
        const NAMES: [&str; 12] = [
            "None", "T2A", "UOR", "UOTD", "LBR", "AOS", "SE", "ML", "SA", "HS", "TOL", "EJ",
        ];
        NAMES[self as usize]
    }

    /// `Enum.TryParse<Expansion>`: a name (case-sensitive) or a number.
    pub fn parse(s: &str) -> Option<Expansion> {
        let s = s.trim();
        EXPANSIONS
            .iter()
            .copied()
            .find(|e| e.name() == s)
            .or_else(|| s.parse().ok().and_then(Expansion::from_index))
    }
}

impl std::fmt::Display for Expansion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct FeatureFlags: u32 {
        const T2A = 0x1;
        /// In later clients, the T2A/UOR flags are negative feature flags to disable body replacement of Pre-AOS graphics.
        const UOR = 0x2;
        const UOTD = 0x4;
        const LBR = 0x8;
        const AOS = 0x10;
        const SIXTH_CHARACTER_SLOT = 0x20;
        const SE = 0x40;
        const ML = 0x80;
        const EIGHTH_AGE = 0x100;
        /// Crystal/Shadow Custom House Tiles
        const NINTH_AGE = 0x200;
        const TENTH_AGE = 0x400;
        /// Increased Housing/Bank Storage
        const INCREASED_STORAGE = 0x800;
        const SEVENTH_CHARACTER_SLOT = 0x1000;
        const ROLEPLAY_FACES = 0x2000;
        const TRIAL_ACCOUNT = 0x4000;
        const LIVE_ACCOUNT = 0x8000;
        const SA = 0x10000;
        const HS = 0x20000;
        const GOTHIC = 0x40000;
        const RUSTIC = 0x80000;
        const JUNGLE = 0x100000;
        const SHADOWGUARD = 0x200000;
        const TOL = 0x400000;
        const EJ = 0x800000;

        const EXPANSION_NONE = 0;
        const EXPANSION_T2A = Self::T2A.bits();
        const EXPANSION_UOR = Self::EXPANSION_T2A.bits() | Self::UOR.bits();
        const EXPANSION_UOTD = Self::EXPANSION_UOR.bits() | Self::UOTD.bits();
        const EXPANSION_LBR = Self::EXPANSION_UOTD.bits() | Self::LBR.bits();
        const EXPANSION_AOS = Self::LBR.bits() | Self::AOS.bits() | Self::LIVE_ACCOUNT.bits();
        const EXPANSION_SE = Self::EXPANSION_AOS.bits() | Self::SE.bits();
        const EXPANSION_ML = Self::EXPANSION_SE.bits() | Self::ML.bits() | Self::NINTH_AGE.bits();
        const EXPANSION_SA = Self::EXPANSION_ML.bits() | Self::SA.bits() | Self::GOTHIC.bits() | Self::RUSTIC.bits();
        const EXPANSION_HS = Self::EXPANSION_SA.bits() | Self::HS.bits();
        const EXPANSION_TOL = Self::EXPANSION_HS.bits() | Self::TOL.bits() | Self::JUNGLE.bits() | Self::SHADOWGUARD.bits();
        const EXPANSION_EJ = Self::EXPANSION_TOL.bits() | Self::EJ.bits();
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct CharacterListFlags: u32 {
        const UNK1 = 0x1;
        const OVERWRITE_CONFIG_BUTTON = 0x2;
        const ONE_CHARACTER_SLOT = 0x4;
        const CONTEXT_MENUS = 0x8;
        const SLOT_LIMIT = 0x10;
        const AOS = 0x20;
        const SIXTH_CHARACTER_SLOT = 0x40;
        const SE = 0x80;
        const ML = 0x100;
        const KR = 0x200;
        const UO3D_CLIENT_TYPE = 0x400;
        const UNK3 = 0x800;
        const SEVENTH_CHARACTER_SLOT = 0x1000;
        const UNK4 = 0x2000;
        /// Doesn't seem to be used on OSI
        const NEW_MOVEMENT_SYSTEM = 0x4000;
        const NEW_FELUCCA_AREAS = 0x8000;

        const EXPANSION_NONE = Self::CONTEXT_MENUS.bits();
        const EXPANSION_T2A = Self::CONTEXT_MENUS.bits();
        const EXPANSION_UOR = Self::CONTEXT_MENUS.bits();
        const EXPANSION_UOTD = Self::CONTEXT_MENUS.bits();
        const EXPANSION_LBR = Self::CONTEXT_MENUS.bits();
        const EXPANSION_AOS = Self::CONTEXT_MENUS.bits() | Self::AOS.bits();
        const EXPANSION_SE = Self::EXPANSION_AOS.bits() | Self::SE.bits();
        const EXPANSION_ML = Self::EXPANSION_SE.bits() | Self::ML.bits();
        const EXPANSION_SA = Self::EXPANSION_ML.bits();
        const EXPANSION_HS = Self::EXPANSION_SA.bits();
        const EXPANSION_TOL = Self::EXPANSION_HS.bits();
        const EXPANSION_EJ = Self::EXPANSION_TOL.bits();
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct HousingFlags: u32 {
        const AOS = 0x10;
        const SE = 0x40;
        const ML = 0x80;
        const CRYSTAL = 0x200;
        const SA = 0x10000;
        const HS = 0x20000;
        const GOTHIC = 0x40000;
        const RUSTIC = 0x80000;
        const JUNGLE = 0x100000;
        const SHADOWGUARD = 0x200000;
        const TOL = 0x400000;
        const EJ = 0x800000;

        const HOUSING_AOS = Self::AOS.bits();
        const HOUSING_SE = Self::HOUSING_AOS.bits() | Self::SE.bits();
        const HOUSING_ML = Self::HOUSING_SE.bits() | Self::ML.bits() | Self::CRYSTAL.bits();
        const HOUSING_SA = Self::HOUSING_ML.bits() | Self::SA.bits() | Self::GOTHIC.bits() | Self::RUSTIC.bits();
        const HOUSING_HS = Self::HOUSING_SA.bits() | Self::HS.bits();
        const HOUSING_TOL = Self::HOUSING_HS.bits() | Self::TOL.bits() | Self::JUNGLE.bits() | Self::SHADOWGUARD.bits();
        const HOUSING_EJ = Self::HOUSING_TOL.bits() | Self::EJ.bits();
    }
}

bitflags! {
    /// Which maps an expansion offers.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct MapSelectionFlags: u32 {
        const FELUCCA = 0x01;
        const TRAMMEL = 0x02;
        const ILSHENAR = 0x04;
        const MALAS = 0x08;
        const TOKUNO = 0x10;
        const TER_MUR = 0x20;
    }
}

json_flags!(
    FeatureFlags,
    [
        ("None", 0),
        ("T2A", 0x1),
        ("ExpansionT2A", 0x1),
        ("UOR", 0x2),
        ("UOTD", 0x4),
        ("LBR", 0x8),
        ("AOS", 0x10),
        ("SixthCharacterSlot", 0x20),
        ("SE", 0x40),
        ("ML", 0x80),
        ("EighthAge", 0x100),
        ("NinthAge", 0x200),
        ("TenthAge", 0x400),
        ("IncreasedStorage", 0x800),
        ("SeventhCharacterSlot", 0x1000),
        ("RoleplayFaces", 0x2000),
        ("TrialAccount", 0x4000),
        ("LiveAccount", 0x8000),
        ("SA", 0x10000),
        ("HS", 0x20000),
        ("Gothic", 0x40000),
        ("Rustic", 0x80000),
        ("Jungle", 0x100000),
        ("Shadowguard", 0x200000),
        ("TOL", 0x400000),
        ("EJ", 0x800000),
        ("ExpansionNone", 0),
        ("ExpansionUOR", 0x3),
        ("ExpansionUOTD", 0x7),
        ("ExpansionLBR", 0xF),
        ("ExpansionAOS", FeatureFlags::EXPANSION_AOS.bits()),
        ("ExpansionSE", FeatureFlags::EXPANSION_SE.bits()),
        ("ExpansionML", FeatureFlags::EXPANSION_ML.bits()),
        ("ExpansionSA", FeatureFlags::EXPANSION_SA.bits()),
        ("ExpansionHS", FeatureFlags::EXPANSION_HS.bits()),
        ("ExpansionTOL", FeatureFlags::EXPANSION_TOL.bits()),
        ("ExpansionEJ", FeatureFlags::EXPANSION_EJ.bits()),
    ]
);

json_flags!(
    CharacterListFlags,
    [
        ("Unk1", 0x1),
        ("OverwriteConfigButton", 0x2),
        ("OneCharacterSlot", 0x4),
        ("ContextMenus", 0x8),
        ("ExpansionNone", 0x8),
        ("ExpansionT2A", 0x8),
        ("ExpansionUOR", 0x8),
        ("ExpansionUOTD", 0x8),
        ("ExpansionLBR", 0x8),
        ("SlotLimit", 0x10),
        ("AOS", 0x20),
        ("SixthCharacterSlot", 0x40),
        ("SE", 0x80),
        ("ML", 0x100),
        ("KR", 0x200),
        ("UO3DClientType", 0x400),
        ("Unk3", 0x800),
        ("SeventhCharacterSlot", 0x1000),
        ("Unk4", 0x2000),
        ("NewMovementSystem", 0x4000),
        ("NewFeluccaAreas", 0x8000),
        ("ExpansionAOS", 0x28),
        ("ExpansionSE", 0xA8),
        ("ExpansionML", 0x1A8),
        ("ExpansionSA", 0x1A8),
        ("ExpansionHS", 0x1A8),
        ("ExpansionTOL", 0x1A8),
        ("ExpansionEJ", 0x1A8),
    ]
);

json_flags!(
    HousingFlags,
    [
        ("AOS", 0x10),
        ("HousingAOS", 0x10),
        ("SE", 0x40),
        ("ML", 0x80),
        ("Crystal", 0x200),
        ("SA", 0x10000),
        ("HS", 0x20000),
        ("Gothic", 0x40000),
        ("Rustic", 0x80000),
        ("Jungle", 0x100000),
        ("Shadowguard", 0x200000),
        ("TOL", 0x400000),
        ("EJ", 0x800000),
        ("HousingSE", HousingFlags::HOUSING_SE.bits()),
        ("HousingML", HousingFlags::HOUSING_ML.bits()),
        ("HousingSA", HousingFlags::HOUSING_SA.bits()),
        ("HousingHS", HousingFlags::HOUSING_HS.bits()),
        ("HousingTOL", HousingFlags::HOUSING_TOL.bits()),
        ("HousingEJ", HousingFlags::HOUSING_EJ.bits()),
    ]
);

json_flags!(
    MapSelectionFlags,
    [
        ("Felucca", 0x01),
        ("Trammel", 0x02),
        ("Ilshenar", 0x04),
        ("Malas", 0x08),
        ("Tokuno", 0x10),
        ("TerMur", 0x20),
    ]
);

impl MapSelectionFlags {
    /// `MapSelectionValues`.
    pub const VALUES: [MapSelectionFlags; 6] = [
        MapSelectionFlags::FELUCCA,
        MapSelectionFlags::TRAMMEL,
        MapSelectionFlags::ILSHENAR,
        MapSelectionFlags::MALAS,
        MapSelectionFlags::TOKUNO,
        MapSelectionFlags::TER_MUR,
    ];

    pub fn name_of(flag: MapSelectionFlags) -> &'static str {
        match flag {
            MapSelectionFlags::FELUCCA => "Felucca",
            MapSelectionFlags::TRAMMEL => "Trammel",
            MapSelectionFlags::ILSHENAR => "Ilshenar",
            MapSelectionFlags::MALAS => "Malas",
            MapSelectionFlags::TOKUNO => "Tokuno",
            MapSelectionFlags::TER_MUR => "TerMur",
            _ => "",
        }
    }

    pub fn includes(self, flag: MapSelectionFlags) -> bool {
        self.contains(flag)
    }

    /// `GetEnumerable`: the single maps set, in order.
    pub fn iter_maps(self) -> impl Iterator<Item = MapSelectionFlags> {
        Self::VALUES.into_iter().filter(move |f| self.contains(*f))
    }

    /// `ToCommaDelimitedString`.
    pub fn to_comma_delimited_string(self) -> String {
        let s = self
            .iter_maps()
            .map(Self::name_of)
            .collect::<Vec<_>>()
            .join(", ");
        if s.is_empty() { "None".to_string() } else { s }
    }

    /// `EnumFromExpansion`.
    pub fn from_expansion(
        table: &ExpansionTable,
        expansion: Expansion,
    ) -> impl Iterator<Item = MapSelectionFlags> {
        table
            .get_info(expansion.index())
            .map_selection_flags
            .iter_maps()
    }

    /// `ToSelectionFlag(map)`: `1 << MapID`.
    pub fn from_map_id(map_id: i32) -> MapSelectionFlags {
        MapSelectionFlags::from_bits_retain(1u32.wrapping_shl(map_id as u32))
    }
}

const CLIENT_FLAG_NAMES: [(&str, u32); 9] = [
    ("Felucca", 0x1),
    ("Trammel", 0x2),
    ("Ilshenar", 0x4),
    ("Malas", 0x8),
    ("Tokuno", 0x10),
    ("TerMur", 0x20),
    ("KR", 0x40),
    ("Unk2", 0x80),
    ("UOTD", 0x100),
];

/// `JsonStringEnumConverter` text of a flags enum: `"None"`, `"Felucca"`, `"Felucca, Trammel"`; unknown bits make
/// the whole value a number string.
pub fn client_flags_to_string(f: ClientFlags) -> String {
    let bits = f.bits();
    let known: u32 = CLIENT_FLAG_NAMES.iter().fold(0, |a, n| a | n.1);
    if bits == 0 {
        return "None".into();
    }
    if bits & !known != 0 {
        return bits.to_string();
    }
    CLIENT_FLAG_NAMES
        .iter()
        .filter(|n| bits & n.1 != 0)
        .map(|n| n.0)
        .collect::<Vec<_>>()
        .join(", ")
}

/// The reverse: comma separated names (case-insensitive) or a number.
pub fn client_flags_from_str(s: &str) -> Option<ClientFlags> {
    if let Ok(n) = s.trim().parse::<u32>() {
        return Some(ClientFlags::from_bits_retain(n));
    }
    let mut bits = 0;
    for part in s.split(',') {
        let part = part.trim();
        if part.eq_ignore_ascii_case("None") {
            continue;
        }
        bits |= CLIENT_FLAG_NAMES
            .iter()
            .find(|n| n.0.eq_ignore_ascii_case(part))?
            .1;
    }
    Some(ClientFlags::from_bits_retain(bits))
}

mod client_flags_json {
    use super::*;

    pub fn serialize<S: Serializer>(v: &ClientFlags, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&client_flags_to_string(*v))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> std::result::Result<ClientFlags, D::Error> {
        use serde::de::Error;
        let s = String::deserialize(d)?;
        client_flags_from_str(&s).ok_or_else(|| {
            D::Error::custom(format!(
                "The JSON value '{s}' could not be converted to ClientFlags"
            ))
        })
    }
}

/// One expansion's entry in `expansions.json` / `expansion.json`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ExpansionInfo {
    pub id: i32,
    pub name: String,
    #[serde(with = "client_flags_json")]
    pub client_flags: ClientFlags,
    #[serde(with = "converters::flags")]
    pub supported_features: FeatureFlags,
    #[serde(with = "converters::flags")]
    pub character_list_flags: CharacterListFlags,
    #[serde(
        with = "converters::client_version::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub required_client: Option<ClientVersion>,
    #[serde(with = "converters::flags")]
    pub housing_flags: HousingFlags,
    pub mobile_status_version: i32,
    #[serde(with = "converters::flags")]
    pub map_selection_flags: MapSelectionFlags,
}

impl std::fmt::Display for ExpansionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

/// `ExpansionInfo.GetEraFolder`: the closest era folder below `expansion` in `parent` (case-insensitive names).
///
/// It decrements before testing (`while (expansion-- >= 0)`), so the expansion itself is never tried: ML looks
/// for `SE`, then `AOS`... and finally the nonexistent `-1`. Kept as is.
pub fn get_era_folder(parent: &Path, expansion: Expansion) -> Option<PathBuf> {
    let folders: Vec<PathBuf> = std::fs::read_dir(parent)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    let mut index = expansion.index();
    while index >= 0 {
        index -= 1;
        let wanted = Expansion::from_index(index).map(Expansion::name);
        if let Some(wanted) = wanted {
            let found = folders.iter().find(|f| {
                f.file_name()
                    .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(wanted))
            });
            if let Some(f) = found {
                return Some(f.clone());
            }
        }
    }
    None
}

/// The expansion table (`ExpansionInfo.Table`).
#[derive(Clone, Debug, PartialEq)]
pub struct ExpansionTable {
    pub table: Vec<ExpansionInfo>,
}

impl ExpansionTable {
    /// Reads `Data/expansions.json` under `base_dir`; a missing file is an error.
    pub fn load(base_dir: &Path) -> Result<ExpansionTable> {
        let path = base_dir.join("Data/expansions.json");
        let table = json::deserialize_file::<Vec<ExpansionInfo>>(&path)?.ok_or_else(|| {
            DataError::NotFound(format!(
                "Expansion file '{}' could not be found.",
                path.display()
            ))
        })?;
        if table.is_empty() {
            return Err(DataError::invalid(path.to_string_lossy(), "no expansions"));
        }
        Ok(ExpansionTable { table })
    }

    /// `GetInfo`: an index outside the table is expansion 0.
    pub fn get_info(&self, ex: i32) -> &ExpansionInfo {
        let v = if ex < 0 || ex as usize >= self.table.len() {
            0
        } else {
            ex as usize
        };
        &self.table[v]
    }

    /// `StoreMapSelection`.
    pub fn store_map_selection(&mut self, selection: MapSelectionFlags, expansion: Expansion) {
        if let Some(info) = self.table.get_mut(expansion.index() as usize) {
            info.map_selection_flags = selection;
        }
    }

    /// `SaveConfiguration`: writes the current expansion's entry to `Configuration/expansion.json`.
    pub fn save_configuration(&self, base_dir: &Path, current: Expansion) -> Result<()> {
        json::serialize_file(
            &base_dir.join(EXPANSION_CONFIGURATION_PATH),
            self.get_info(current.index()),
        )
    }

    /// `LoadConfiguration`: reads `Configuration/expansion.json` and, if it exists, replaces that expansion's
    /// table entry and returns it.
    pub fn load_configuration(&mut self, base_dir: &Path) -> Result<Option<Expansion>> {
        let Some(info) =
            json::deserialize_file::<ExpansionInfo>(&base_dir.join(EXPANSION_CONFIGURATION_PATH))?
        else {
            return Ok(None);
        };
        let Some(expansion) = Expansion::from_index(info.id) else {
            return Err(DataError::invalid(
                EXPANSION_CONFIGURATION_PATH,
                format!("Id {} is not an expansion", info.id),
            ));
        };
        let index = expansion.index() as usize;
        self.table[index] = info;
        Ok(Some(expansion))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("muo_exp_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn the_distribution_table_loads() {
        let Some(real) = std::env::var_os("MUO_DISTRIBUTION") else {
            return;
        };
        let t = ExpansionTable::load(Path::new(&real)).unwrap();
        assert_eq!(t.table.len(), 12);
        let ml = t.get_info(Expansion::ML.index());
        assert_eq!(ml.name, "Mondain's Legacy");
        assert!(
            ml.supported_features
                .contains(FeatureFlags::ML | FeatureFlags::NINTH_AGE)
        );
        assert!(!ml.supported_features.contains(FeatureFlags::SA));
        assert_eq!(t.get_info(99).id, 0);
        assert_eq!(t.get_info(-1).id, 0);
        // the Distribution file spells the sixth map "TerMer", so it never selects Ter Mur
        assert!(
            !t.get_info(Expansion::EJ.index())
                .map_selection_flags
                .contains(MapSelectionFlags::TER_MUR)
        );
    }

    #[test]
    fn expansion_json_round_trip() {
        let mut info = ExpansionInfo {
            id: 7,
            name: "Mondain's Legacy".into(),
            client_flags: ClientFlags::FELUCCA | ClientFlags::TRAMMEL,
            supported_features: FeatureFlags::EXPANSION_ML,
            character_list_flags: CharacterListFlags::EXPANSION_ML,
            required_client: Some(ClientVersion::parse("7.0.0.0")),
            housing_flags: HousingFlags::HOUSING_ML,
            mobile_status_version: 6,
            map_selection_flags: MapSelectionFlags::FELUCCA | MapSelectionFlags::MALAS,
        };
        let text = json::to_string(&info).unwrap();
        assert!(
            text.contains("\"ClientFlags\": \"Felucca, Trammel\"")
                && text.contains("\"RequiredClient\": \"7.0.0.0\"")
        );
        assert_eq!(json::from_str::<ExpansionInfo>(&text).unwrap(), info);

        info.required_client = None;
        assert!(!json::to_string(&info).unwrap().contains("RequiredClient"));
    }

    #[test]
    fn configuration_files() {
        let dir = temp("cfg");
        let mut t = ExpansionTable {
            table: (0..12)
                .map(|i| ExpansionInfo {
                    id: i,
                    name: format!("e{i}"),
                    ..Default::default()
                })
                .collect(),
        };
        assert_eq!(t.load_configuration(&dir).unwrap(), None);
        t.store_map_selection(MapSelectionFlags::FELUCCA, Expansion::SE);
        t.save_configuration(&dir, Expansion::SE).unwrap();
        let mut fresh = ExpansionTable {
            table: (0..12)
                .map(|i| ExpansionInfo {
                    id: i,
                    ..Default::default()
                })
                .collect(),
        };
        assert_eq!(fresh.load_configuration(&dir).unwrap(), Some(Expansion::SE));
        assert_eq!(
            fresh.get_info(6).map_selection_flags,
            MapSelectionFlags::FELUCCA
        );
        std::fs::write(dir.join(EXPANSION_CONFIGURATION_PATH), "{ \"Id\": 99 }").unwrap();
        assert!(fresh.load_configuration(&dir).is_err());
        std::fs::write(dir.join(EXPANSION_CONFIGURATION_PATH), "{ nope").unwrap();
        assert!(fresh.load_configuration(&dir).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn era_folder_starts_below_the_expansion() {
        let dir = temp("era");
        std::fs::create_dir_all(dir.join("aos")).unwrap();
        std::fs::create_dir_all(dir.join("ML")).unwrap();
        // ML itself is never tried; the nearest lower folder wins
        assert_eq!(
            get_era_folder(&dir, Expansion::ML)
                .unwrap()
                .file_name()
                .unwrap(),
            "aos"
        );
        assert!(get_era_folder(&dir, Expansion::T2A).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn map_selection_helpers() {
        let f = MapSelectionFlags::FELUCCA | MapSelectionFlags::MALAS;
        assert_eq!(f.to_comma_delimited_string(), "Felucca, Malas");
        assert_eq!(
            MapSelectionFlags::empty().to_comma_delimited_string(),
            "None"
        );
        assert_eq!(MapSelectionFlags::from_map_id(3), MapSelectionFlags::MALAS);
        assert_eq!(Expansion::parse("ML"), Some(Expansion::ML));
        assert_eq!(Expansion::parse("7"), Some(Expansion::ML));
        assert_eq!(Expansion::parse("ml"), None);
    }

    #[test]
    fn map_selection_comma_separated_list() {
        assert_eq!(
            MapSelectionFlags::FELUCCA.to_comma_delimited_string(),
            "Felucca"
        );
        assert_eq!(
            (MapSelectionFlags::FELUCCA | MapSelectionFlags::TRAMMEL).to_comma_delimited_string(),
            "Felucca, Trammel"
        );
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestMapConfig {
        #[serde(with = "converters::flags")]
        test_map_flags: MapSelectionFlags,
    }

    #[test]
    fn serialize_and_deserialize_map_selection_flags() {
        let map_config = TestMapConfig {
            test_map_flags: MapSelectionFlags::FELUCCA | MapSelectionFlags::ILSHENAR,
        };
        let serialized = json::to_string(&map_config).unwrap();
        let deserialized: TestMapConfig = json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, map_config);
    }
}
