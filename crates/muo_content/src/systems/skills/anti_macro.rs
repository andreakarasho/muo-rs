//! The anti-macro system: skill gain is refused when a player keeps using a skill on the same
//! location or target. `Configuration/antimacro.json`.

use indexmap::IndexMap;
use muo_primitives::{DateTime, Point2D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::skills::SkillName;
use muo_server::target::Targeted;
use muo_server::timer::{self, TimerHandle};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// *** NOTE ***: Modifying these values will not change an already created antimacro.json file!
const ANTI_MACRO_SKILL_DEFAULTS: [bool; 58] = [
    false, // Alchemy = 0,
    true,  // Anatomy = 1,
    true,  // AnimalLore = 2,
    true,  // ItemID = 3,
    true,  // ArmsLore = 4,
    false, // Parry = 5,
    true,  // Begging = 6,
    false, // Blacksmith = 7,
    false, // Fletching = 8,
    true,  // Peacemaking = 9,
    true,  // Camping = 10,
    false, // Carpentry = 11,
    false, // Cartography = 12,
    false, // Cooking = 13,
    true,  // DetectHidden = 14,
    true,  // Discordance = 15,
    true,  // EvalInt = 16,
    true,  // Healing = 17,
    true,  // Fishing = 18,
    true,  // Forensics = 19,
    true,  // Herding = 20,
    true,  // Hiding = 21,
    true,  // Provocation = 22,
    false, // Inscribe = 23,
    true,  // Lockpicking = 24,
    true,  // Magery = 25,
    true,  // MagicResist = 26,
    false, // Tactics = 27,
    true,  // Snooping = 28,
    true,  // Musicianship = 29,
    true,  // Poisoning = 30,
    false, // Archery = 31,
    true,  // SpiritSpeak = 32,
    true,  // Stealing = 33,
    false, // Tailoring = 34,
    true,  // AnimalTaming = 35,
    true,  // TasteID = 36,
    false, // Tinkering = 37,
    true,  // Tracking = 38,
    true,  // Veterinary = 39,
    false, // Swords = 40,
    false, // Macing = 41,
    false, // Fencing = 42,
    false, // Wrestling = 43,
    true,  // Lumberjacking = 44,
    true,  // Mining = 45,
    true,  // Meditation = 46,
    true,  // Stealth = 47,
    true,  // RemoveTrap = 48,
    true,  // Necromancy = 49,
    false, // Focus = 50,
    true,  // Chivalry = 51
    true,  // Bushido = 52
    true,  // Ninjitsu = 53
    true,  // Spellweaving
    true,  // Mysticism = 55
    true,  // Imbuing = 56
    false, // Throwing = 57
];

const ANTI_MACRO_PATH: &str = "Configuration/antimacro.json";

/// `AntiMacroSettings`.
#[derive(Clone, Debug, PartialEq)]
pub struct AntiMacroSettings {
    /// How many times may we use the same location/target for gain.
    pub allowance: i32,
    /// The size of each location, make this smaller so players dont have to move as far.
    pub location_size: i32,
    pub enabled: bool,
    /// How long do we remember targets/locations?
    pub expire: TimeSpan,
    /// `SkillsThatUseAntiMacro`.
    pub skills_that_use_anti_macro: Vec<bool>,
}

impl Default for AntiMacroSettings {
    fn default() -> Self {
        Self {
            allowance: 3,
            location_size: 5,
            enabled: false,
            expire: TimeSpan::from_mins(5),
            skills_that_use_anti_macro: ANTI_MACRO_SKILL_DEFAULTS.to_vec(),
        }
    }
}

/// `(name, index)` of every `SkillName`, for the `BitArrayEnumIndexConverter`.
fn skill_names() -> Vec<(String, usize)> {
    (0..SkillName::COUNT as i32)
        .filter_map(SkillName::from_i32)
        .map(|s| (format!("{s:?}"), s as usize))
        .collect()
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SettingsJson {
    #[serde(default)]
    allowance: i32,
    #[serde(default)]
    location_size: i32,
    #[serde(default)]
    enabled: bool,
    #[serde(with = "muo_data::json::converters::timespan", default = "zero")]
    expire: TimeSpan,
    skills_that_use_anti_macro: Value,
}

fn zero() -> TimeSpan {
    TimeSpan::ZERO
}

impl AntiMacroSettings {
    fn to_json(&self) -> SettingsJson {
        let names = skill_names();
        let names: Vec<(&str, usize)> = names.iter().map(|(n, i)| (n.as_str(), *i)).collect();
        SettingsJson {
            allowance: self.allowance,
            location_size: self.location_size,
            enabled: self.enabled,
            expire: self.expire,
            skills_that_use_anti_macro: muo_data::json::converters::bit_array::to_value(
                &names,
                &self.skills_that_use_anti_macro,
            ),
        }
    }

    fn from_json(j: SettingsJson) -> Result<Self, String> {
        let names = skill_names();
        let names: Vec<(&str, usize)> = names.iter().map(|(n, i)| (n.as_str(), *i)).collect();
        let bits = muo_data::json::converters::bit_array::from_value::<serde_json::Error>(
            &names,
            &j.skills_that_use_anti_macro,
        )
        .map_err(|e| e.to_string())?;
        Ok(Self {
            allowance: j.allowance,
            location_size: j.location_size,
            enabled: j.enabled,
            expire: j.expire,
            skills_that_use_anti_macro: bits,
        })
    }
}

/// What a skill was used on: a location bucket or an entity. A `LandTarget`/`StaticTarget` is a fresh object each
/// time (reference equality), so it never matches an earlier one and is not remembered.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum AntiMacroObject {
    Location(Point2D),
    Entity(EntityId),
    /// A land or static target.
    Fresh,
}

impl From<Targeted> for AntiMacroObject {
    fn from(t: Targeted) -> Self {
        match t {
            Targeted::Entity(e) => Self::Entity(e),
            Targeted::Land(_) | Targeted::Static(_) => Self::Fresh,
        }
    }
}

impl AntiMacroObject {
    /// The entity, for `Region.AllowGain(m, skill, object)`.
    pub fn entity(self) -> Option<EntityId> {
        match self {
            Self::Entity(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Default)]
struct CountAndTimeStamp {
    count: i32,
    expiration: DateTime,
}

#[derive(Default)]
struct PlayerAntiMacro {
    // This can get quite large. If a player is logged in for a while it grows until the next world save.
    tracking: IndexMap<(SkillName, AntiMacroObject), CountAndTimeStamp>,
    last_expiration: DateTime,
}

/// The statics.
#[derive(Default)]
pub struct AntiMacro {
    settings: AntiMacroSettings,
    table: IndexMap<MobileId, PlayerAntiMacro>,
    logout_cleanup: IndexMap<MobileId, TimerHandle>,
}

pub fn settings(w: &mut World) -> &AntiMacroSettings {
    &w.resources.get_mut::<AntiMacro>().settings
}

fn state(w: &mut World) -> &mut AntiMacro {
    w.resources.get_mut::<AntiMacro>()
}

/// `AntiMacroSystem.Configure()`.
pub fn configure(w: &mut World) {
    let path = w.core.base_directory.join(ANTI_MACRO_PATH);
    let s = if path.exists() {
        match muo_data::json::deserialize_file::<SettingsJson>(&path)
            .map_err(|e| e.to_string())
            .and_then(|j| {
                j.ok_or_else(|| "empty".to_string())
                    .and_then(AntiMacroSettings::from_json)
            }) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed reading {ANTI_MACRO_PATH}: {e}; using the defaults");
                AntiMacroSettings::default()
            }
        }
    } else {
        let s = AntiMacroSettings::default();
        if let Err(e) = muo_data::json::serialize_file(&path, &s.to_json()) {
            tracing::error!("Saving {ANTI_MACRO_PATH} failed: {e}");
        }
        s
    };
    state(w).settings = s;
}

/// `AntiMacroSystem.Initialize()`.
pub fn initialize(w: &mut World) {
    w.events.world_save.subscribe(|w, _| on_world_save(w));
    w.events.logout.subscribe(|w, m| on_logout(w, *m));
}

fn on_world_save(w: &mut World) {
    let now = w.now;
    let table = &mut state(w).table;
    table.retain(|_, anti_macro| {
        if anti_macro.last_expiration <= now {
            false
        } else {
            // CleanExpired
            anti_macro
                .tracking
                .retain(|_, c| !(c.count <= 0 || c.expiration <= now));
            true
        }
    });
}

/// `OnLogin(pm)`: stop the clear out timer.
pub fn on_login(w: &mut World, pm: MobileId) {
    if let Some(mut t) = state(w).logout_cleanup.shift_remove(&pm) {
        t.cancel(w);
    }
}

fn on_logout(w: &mut World, m: MobileId) {
    let now = w.now;
    let Some(last) = state(w).table.get(&m).map(|a| a.last_expiration) else {
        return;
    };

    if last < now {
        state(w).table.shift_remove(&m);
        return;
    }

    let expire = state(w).settings.expire;
    if let Some(mut t) = state(w).logout_cleanup.shift_remove(&m) {
        // timer.Stop(); then timer.Start() below: restarts with the original delay
        t.cancel(w);
    }
    let t = timer::start(w, expire, TimeSpan::ZERO, 1, move |w| cleanup_player(w, m));
    state(w).logout_cleanup.insert(m, t);
}

/// `CleanupPlayer(pm)`.
pub fn cleanup_player(w: &mut World, pm: MobileId) {
    state(w).table.shift_remove(&pm);
    if let Some(mut t) = state(w).logout_cleanup.shift_remove(&pm) {
        t.cancel(w);
    }
}

/// `UseAntiMacro(skillId)`.
pub fn use_anti_macro(w: &mut World, skill_id: i32) -> bool {
    usize::try_from(skill_id)
        .ok()
        .and_then(|i| settings(w).skills_that_use_anti_macro.get(i).copied())
        .unwrap_or(false)
}

/// `AntiMacroCheck(pm, skill, obj)`: false refuses the gain.
pub fn anti_macro_check(
    w: &mut World,
    pm: MobileId,
    skill: SkillName,
    obj: Option<AntiMacroObject>,
) -> bool {
    let Some(obj) = obj.filter(|o| *o != AntiMacroObject::Fresh) else {
        return true;
    };
    let is_player_level = w
        .mobile(pm)
        .is_some_and(|m| m.access_level == AccessLevel::Player);
    if !settings(w).enabled || !is_player_level || !use_anti_macro(w, skill as i32) {
        return true;
    }

    let now = w.now;
    let (allowance, expire) = {
        let s = settings(w);
        (s.allowance, s.expire)
    };
    let anti = state(w).table.entry(pm).or_default();
    let existed = anti.tracking.contains_key(&(skill, obj));
    let c = anti.tracking.entry((skill, obj)).or_default();
    c.count += 1;

    if !existed || c.expiration <= now || c.count < allowance {
        c.expiration = now + expire;
        anti.last_expiration = now + expire;
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;

    #[test]
    fn defaults_and_the_json_file() {
        let s = AntiMacroSettings::default();
        assert_eq!(s.skills_that_use_anti_macro.len(), 58);
        assert!(s.skills_that_use_anti_macro[SkillName::Anatomy as usize]);
        assert!(!s.skills_that_use_anti_macro[SkillName::Alchemy as usize]);
        let text = muo_data::json::to_string(&s.to_json()).unwrap();
        assert!(
            text.contains("\"Allowance\": 3") && text.contains("\"Expire\": \"00:05:00\""),
            "{text}"
        );
        assert!(
            text.contains("\"Anatomy\": true") && text.contains("\"Alchemy\": false"),
            "{text}"
        );
        let back = AntiMacroSettings::from_json(muo_data::json::from_str(&text).unwrap()).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn the_same_target_stops_counting_after_the_allowance() {
        let mut w = World::new();
        state(&mut w).settings.enabled = true;
        let m = w.new_mobile(&MOBILE_BASE);
        let other = w.new_mobile(&MOBILE_BASE);
        w.now = DateTime::from_ymd_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let target = Some(AntiMacroObject::Entity(EntityId::Mobile(other)));
        // Allowance 3: the first two uses (count < 3) and the first (not existing) pass, the third is refused.
        assert!(anti_macro_check(&mut w, m, SkillName::Anatomy, target));
        assert!(anti_macro_check(&mut w, m, SkillName::Anatomy, target));
        assert!(!anti_macro_check(&mut w, m, SkillName::Anatomy, target));
        // A different skill, a different target, or a skill that doesn't use it are fresh.
        assert!(anti_macro_check(&mut w, m, SkillName::EvalInt, target));
        assert!(anti_macro_check(
            &mut w,
            m,
            SkillName::Anatomy,
            Some(AntiMacroObject::Location(Point2D::new(1, 1)))
        ));
        assert!(anti_macro_check(&mut w, m, SkillName::Alchemy, target));
        assert!(anti_macro_check(&mut w, m, SkillName::Anatomy, None));
        assert!(anti_macro_check(
            &mut w,
            m,
            SkillName::Anatomy,
            Some(AntiMacroObject::Fresh)
        ));
        // After the entry expires the same target is allowed again.
        w.now += TimeSpan::from_mins(6);
        assert!(anti_macro_check(&mut w, m, SkillName::Anatomy, target));
        // Staff are exempt.
        w.mobile_mut(m).unwrap().access_level = AccessLevel::GameMaster;
        for _ in 0..5 {
            assert!(anti_macro_check(&mut w, m, SkillName::Anatomy, target));
        }
    }

    #[test]
    fn world_save_drops_expired_players() {
        let mut w = World::new();
        state(&mut w).settings.enabled = true;
        let m = w.new_mobile(&MOBILE_BASE);
        w.now = DateTime::from_ymd_hms(2024, 1, 1, 0, 0, 0).unwrap();
        anti_macro_check(
            &mut w,
            m,
            SkillName::Anatomy,
            Some(AntiMacroObject::Location(Point2D::new(1, 1))),
        );
        on_world_save(&mut w);
        assert_eq!(state(&mut w).table.len(), 1);
        w.now += TimeSpan::from_mins(6);
        on_world_save(&mut w);
        assert!(state(&mut w).table.is_empty());
    }
}
