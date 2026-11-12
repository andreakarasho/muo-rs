//! Feature flags: `ContentFeatureFlags`, `FeatureFlagSettings`, `FeatureFlagManager` (flags, gump, item,
//! skill and spell blocks, saved as JSON under `Configuration/FeatureFlags`), `FeatureFlagCommands` and
//! `FeatureFlagAdminGump`.
//!
//! The blocks live in [`FeatureFlagManager`] (a `World` resource). The hooks other crates call (item use/equip/container,
//! skill, spell, gump, vendors, house placement, young players, passive detect hidden) are set in [`configure`].
//!
//! shortcut: a gump class is found among `gump_types::ALL` (the gumps' `TYPE_NAME`s; a new gump must be added
//! there) and the gumps open on any connection, not among every type; a stored gump block is kept as it is. `LastModified` is shown as the ISO text of the stored value.

use muo_primitives::dotnet::{IgnoreCase, InvariantCase};
use std::rc::Rc;

use indexmap::IndexMap;
use muo_content::gumps::base::text_colors as tc;
use muo_content::gumps::system::{GumpBlocks, gumps_of};
use muo_content::gumps::{
    DynamicGumpBuilder, DynamicGumpLogic, DynamicGumpObj, RelayInfo, send_gump_to_mobile,
};
use muo_content::mx;
use muo_content::spells::registry::{CAPACITY as SPELL_CAPACITY, SpellRegistry};
use muo_primitives::html;
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::ids::{EntityId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages::send_message_hue;
use muo_server::net_state::NetStateId;
use muo_server::skills::SkillName;
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::EntityType;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

pub const DEFAULT_GUMP_BLOCKED_MESSAGE: &str = "This feature is temporarily disabled.";
pub const DEFAULT_ITEM_USE_BLOCKED_MESSAGE: &str = "This item cannot be used at this time.";
pub const DEFAULT_ITEM_EQUIP_BLOCKED_MESSAGE: &str = "This item cannot be equipped at this time.";
pub const DEFAULT_CONTAINER_BLOCKED_MESSAGE: &str = "This container cannot be opened at this time.";
pub const DEFAULT_SKILL_DISABLED_MESSAGE: &str = "This skill is temporarily disabled.";
pub const DEFAULT_SPELL_DISABLED_MESSAGE: &str = "This spell is temporarily disabled.";

/// `FeatureFlagSettings` (the statics; `RequiredAccessLevel` is mirrored into `GumpBlocks`).
pub struct Settings {
    pub required_access_level: AccessLevel,
    pub log_changes: bool,
    pub broadcast_changes_to_staff: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            required_access_level: AccessLevel::Administrator,
            log_changes: true,
            broadcast_changes_to_staff: true,
        }
    }
}

/// `ContentFeatureFlags`.
#[derive(Copy, Clone, Debug)]
pub struct ContentFeatureFlags {
    pub vendor_purchase: bool,
    pub vendor_sell: bool,
    pub player_vendors: bool,
    pub house_placement: bool,
    pub boat_placement: bool,
    pub bulk_orders: bool,
    pub passive_detect_hidden: bool,
    pub young_player_system: bool,
    pub bitmap_pathfinding_cache: bool,
}

impl Default for ContentFeatureFlags {
    fn default() -> Self {
        Self {
            vendor_purchase: true,
            vendor_sell: true,
            player_vendors: true,
            house_placement: true,
            boat_placement: true,
            bulk_orders: true,
            passive_detect_hidden: true,
            young_player_system: true,
            bitmap_pathfinding_cache: true,
        }
    }
}

/// `FeatureFlag`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FeatureFlag {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub default_enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub last_modified: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_by: Option<String>,
}

/// The members of `FeatureFlagBlockEntry`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BlockBase {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

/// `ItemBlockEntry`.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ItemBlockEntry {
    #[serde(flatten)]
    pub base: BlockBase,
    #[serde(default)]
    pub block_use: bool,
    #[serde(default)]
    pub block_equip: bool,
    #[serde(default)]
    pub block_container_access: bool,
}

/// `SkillBlockEntry`: the skill is written by name (`JsonStringEnumConverter`).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SkillBlockEntry {
    #[serde(flatten)]
    pub base: BlockBase,
    #[serde(default)]
    pub skill: serde_json::Value,
}

/// `FeatureFlagManager`'s state.
pub struct FeatureFlagManager {
    flags: IndexMap<String, FeatureFlag>,
    gump_blocks: IndexMap<String, BlockBase>,
    item_blocks: IndexMap<String, ItemBlockEntry>,
    skill_blocks: Vec<Option<SkillBlockEntry>>,
    spell_blocks: Vec<Option<BlockBase>>,
    initialized: bool,
    pub settings: Settings,
}

impl Default for FeatureFlagManager {
    fn default() -> Self {
        Self {
            flags: IndexMap::new(),
            gump_blocks: IndexMap::new(),
            item_blocks: IndexMap::new(),
            skill_blocks: vec![None; SkillName::COUNT],
            spell_blocks: vec![None; SPELL_CAPACITY],
            initialized: false,
            settings: Settings::default(),
        }
    }
}

fn mgr(w: &World) -> Option<&FeatureFlagManager> {
    w.resources.get::<FeatureFlagManager>()
}

fn mgr_mut(w: &mut World) -> &mut FeatureFlagManager {
    w.resources.get_mut::<FeatureFlagManager>()
}

/// `Type.Name` of a full name.
fn short(full: &str) -> &str {
    full.rsplit('.').next().unwrap_or(full)
}

fn skill_index(name: &SkillName) -> usize {
    *name as i32 as usize
}

/// `Enum.TryParse<SkillName>(text, true, ..)`: a name, or a number.
fn parse_skill(text: &str) -> Option<i32> {
    let t = text.trim();
    if let Ok(n) = t.parse::<i32>() {
        return Some(n);
    }
    (0..SkillName::COUNT as i32)
        .find(|&i| SkillName::from_i32(i).is_some_and(|s| format!("{s:?}").eq_ic(t)))
}

fn skill_name(i: i32) -> String {
    SkillName::from_i32(i).map_or_else(|| i.to_string(), |s| format!("{s:?}"))
}

fn entry_skill(e: &SkillBlockEntry) -> Option<i32> {
    match &e.skill {
        serde_json::Value::String(s) => parse_skill(s),
        serde_json::Value::Number(n) => n.as_i64().and_then(|n| i32::try_from(n).ok()),
        _ => None,
    }
}

// ---- messages ----------------------------------------------------------------------------------------------------

fn log_changes(w: &World) -> bool {
    mgr(w).is_none_or(|m| m.settings.log_changes)
}

fn announce(w: &mut World, text: String) {
    if mgr(w).is_none_or(|m| m.settings.broadcast_changes_to_staff) {
        w.broadcast_staff(0x35, false, &text);
    }
}

// ---- Initialize / Load / Save ------------------------------------------------------------------------------------

fn save_path(w: &World) -> std::path::PathBuf {
    w.core
        .base_directory
        .join("Configuration")
        .join("FeatureFlags")
}

/// `FeatureFlagManager.Initialize`.
pub fn initialize(w: &mut World) {
    if mgr(w).is_some_and(|m| m.initialized) {
        return;
    }
    let _ = std::fs::create_dir_all(save_path(w));
    load(w);
    load_default_flags(w);
    mgr_mut(w).initialized = true;
    let m = mgr(w).expect("just created");
    info!(
        "Feature Flag system initialized with {} flags, {} gump blocks, {} item blocks, {} skill blocks, {} spell blocks",
        m.flags.len(),
        m.gump_blocks.len(),
        m.item_blocks.len(),
        m.skill_blocks.iter().flatten().count(),
        m.spell_blocks.iter().flatten().count()
    );
}

fn save_file<T: Serialize + ?Sized>(w: &World, name: &str, v: &T, what: &str) {
    if let Err(e) = muo_data::json::serialize_file(&save_path(w).join(name), v) {
        error!("Failed to save {what}: {e}");
    }
}

pub fn save(w: &mut World) {
    save_flags(w);
    save_gump_blocks(w);
    save_item_blocks(w);
    save_skill_blocks(w);
    save_spell_blocks(w);
}

fn save_flags(w: &World) {
    if let Some(m) = mgr(w) {
        save_file(
            w,
            "flags.json",
            &m.flags.values().collect::<Vec<_>>(),
            "flags",
        );
    }
}
fn save_gump_blocks(w: &World) {
    if let Some(m) = mgr(w) {
        save_file(
            w,
            "gump-blocks.json",
            &m.gump_blocks.values().collect::<Vec<_>>(),
            "gump blocks",
        );
    }
}
fn save_item_blocks(w: &World) {
    if let Some(m) = mgr(w) {
        save_file(
            w,
            "item-blocks.json",
            &m.item_blocks.values().collect::<Vec<_>>(),
            "item blocks",
        );
    }
}
fn save_skill_blocks(w: &World) {
    if let Some(m) = mgr(w) {
        save_file(
            w,
            "skill-blocks.json",
            &m.skill_blocks.iter().flatten().collect::<Vec<_>>(),
            "skill blocks",
        );
    }
}
fn save_spell_blocks(w: &World) {
    if let Some(m) = mgr(w) {
        save_file(
            w,
            "spell-blocks.json",
            &m.spell_blocks.iter().flatten().collect::<Vec<_>>(),
            "spell blocks",
        );
    }
}

/// `TypeConverter.Read`: the full name of a registered class, or None (with the warning).
fn resolve_stored_type(w: &World, name: &str) -> Option<String> {
    let r = w
        .types
        .find_type_by_name(name, false, true)
        .or_else(|| w.types.find_type_by_name(name, true, true))
        .map(EntityType::full_name);
    if r.is_none() {
        warn!("Attempted to deserialize type {name} which does not exist.");
    }
    r.map(str::to_owned)
}

/// A gump class name is not checked against a registry (there is none): a stored full name is kept as it is.
fn resolve_stored_gump(name: &str) -> Option<String> {
    Some(name.to_owned())
}

/// A spell's registry full name, from a full or short name.
fn resolve_spell(w: &World, name: &str) -> Option<String> {
    let reg = w.resources.get::<SpellRegistry>()?;
    (0..SPELL_CAPACITY as i32)
        .filter_map(|i| reg.type_of(i))
        .map(|r| r.full_name())
        .find(|f| f.eq_ic(name) || short(f).eq_ic(name))
        .map(str::to_owned)
}

/// `FeatureFlagManager.Load`.
pub fn load(w: &mut World) {
    let dir = save_path(w);
    let read =
        |name: &str| muo_data::json::deserialize_file::<Vec<serde_json::Value>>(&dir.join(name));
    let result = (|| -> Result<(), muo_data::error::DataError> {
        if let Some(list) = read("flags.json")? {
            for v in list {
                if let Ok(f) = serde_json::from_value::<FeatureFlag>(v) {
                    mgr_mut(w).flags.insert(f.key.ic_key(), f);
                }
            }
        }
        if let Some(list) = read("gump-blocks.json")? {
            for v in list {
                if let Ok(e) = serde_json::from_value::<BlockBase>(v)
                    && let Some(t) = e.resolved_type.as_deref().and_then(resolve_stored_gump)
                {
                    mgr_mut(w).gump_blocks.insert(t, e);
                }
            }
        }
        if let Some(list) = read("item-blocks.json")? {
            for v in list {
                if let Ok(e) = serde_json::from_value::<ItemBlockEntry>(v)
                    && let Some(t) = e
                        .base
                        .resolved_type
                        .as_deref()
                        .and_then(|n| resolve_stored_type(w, n))
                {
                    mgr_mut(w).item_blocks.insert(t, e);
                }
            }
        }
        if let Some(list) = read("skill-blocks.json")? {
            for v in list {
                if let Ok(e) = serde_json::from_value::<SkillBlockEntry>(v)
                    && let Some(i) = entry_skill(&e)
                        .and_then(|i| usize::try_from(i).ok())
                        .filter(|&i| i < SkillName::COUNT)
                {
                    mgr_mut(w).skill_blocks[i] = Some(e);
                }
            }
        }
        if let Some(list) = read("spell-blocks.json")? {
            for v in list {
                let Ok(e) = serde_json::from_value::<BlockBase>(v) else {
                    continue;
                };
                let Some(name) = e.resolved_type.clone() else {
                    continue;
                };
                let Some(full) = resolve_spell(w, &name) else {
                    warn!("Spell type '{name}' has no registered spell ID, skipping");
                    continue;
                };
                let id = w
                    .resources
                    .get::<SpellRegistry>()
                    .map_or(-1, |r| r.registry_number(&full));
                if let Some(slot) = usize::try_from(id).ok().filter(|&i| i < SPELL_CAPACITY) {
                    mgr_mut(w).spell_blocks[slot] = Some(BlockBase {
                        resolved_type: Some(full),
                        ..e
                    });
                }
            }
        }
        Ok(())
    })();
    if let Err(e) = result {
        error!("Failed to load feature flags: {e}");
    }
    update_gump_blocks(w);
    sync_all_static_flags(w);
    debug!("Feature flags loaded successfully");
}

fn load_default_flags(w: &mut World) {
    let s = w.core.features;
    let c = *w.resources.get_mut::<ContentFeatureFlags>();
    let defaults: [(&str, &str, &str, bool); 14] = [
        (
            "player_trading",
            "Economy",
            "Allow secure trades between players",
            s.player_trading,
        ),
        (
            "pvp_combat",
            "Combat",
            "Allow player vs player combat",
            s.pvp_combat,
        ),
        (
            "bank_access",
            "Economy",
            "Allow players to access their bank boxes",
            s.bank_access,
        ),
        (
            "speedhack_detection",
            "System",
            "Enable speedhack detection",
            s.speedhack_detection,
        ),
        (
            "insurance",
            "Economy",
            "Enable item insurance",
            s.insurance_enabled,
        ),
        (
            "vendor_purchase",
            "Economy",
            "Allow purchasing from NPC vendors",
            c.vendor_purchase,
        ),
        (
            "vendor_sell",
            "Economy",
            "Allow selling to NPC vendors",
            c.vendor_sell,
        ),
        (
            "player_vendors",
            "Economy",
            "Allow player vendor interactions",
            c.player_vendors,
        ),
        (
            "house_placement",
            "Housing",
            "Allow new house placements",
            c.house_placement,
        ),
        (
            "boat_placement",
            "Housing",
            "Allow new boat placements",
            c.boat_placement,
        ),
        (
            "bulk_orders",
            "Crafting",
            "Allow bulk order deeds",
            c.bulk_orders,
        ),
        (
            "passive_detect_hidden",
            "System",
            "Enable passive detect hidden",
            c.passive_detect_hidden,
        ),
        (
            "young_player_system",
            "System",
            "Enable the young player system",
            c.young_player_system,
        ),
        (
            "bitmap_pathfinding_cache",
            "Performance",
            "Enable the bitmap pathfinding cache",
            c.bitmap_pathfinding_cache,
        ),
    ];
    let now = w.now.to_json_string();
    let mut added = 0;
    for (key, category, description, enabled) in defaults {
        let m = mgr_mut(w);
        if m.flags.contains_key(key) {
            continue;
        }
        m.flags.insert(
            key.to_owned(),
            FeatureFlag {
                key: key.to_owned(),
                description: Some(description.to_owned()),
                enabled,
                default_enabled: enabled,
                category: Some(category.to_owned()),
                last_modified: now.clone(),
                last_modified_by: Some("System".to_owned()),
            },
        );
        added += 1;
    }
    if added > 0 {
        save(w);
        info!("Seeded {added} default feature flag(s)");
    }
}

// ---- flags -------------------------------------------------------------------------------------------------------

pub fn is_enabled(w: &World, key: &str) -> bool {
    mgr(w).is_some_and(|m| m.flags.get(&key.ic_key()).is_some_and(|f| f.enabled))
}

pub fn get_flag(w: &World, key: &str) -> Option<FeatureFlag> {
    mgr(w)?.flags.get(&key.ic_key()).cloned()
}

pub fn all_flags(w: &World) -> Vec<FeatureFlag> {
    mgr(w)
        .map(|m| m.flags.values().cloned().collect())
        .unwrap_or_default()
}

pub fn set_flag(w: &mut World, key: &str, enabled: bool, modified_by: &str) -> bool {
    let now = w.now.to_json_string();
    let Some(flag) = mgr_mut(w).flags.get_mut(&key.ic_key()) else {
        return false;
    };
    let previous = flag.enabled;
    flag.enabled = enabled;
    flag.last_modified = now;
    flag.last_modified_by = Some(modified_by.to_owned());
    sync_static_flag(w, key, enabled);
    if log_changes(w) {
        info!("Feature flag '{key}' changed from {previous} to {enabled} by {modified_by}");
    }
    announce(
        w,
        format!(
            "[Feature Flag] '{key}' {} by {modified_by}",
            if enabled { "ENABLED" } else { "DISABLED" }
        ),
    );
    save_flags(w);
    true
}

pub fn create_or_update_flag(
    w: &mut World,
    key: &str,
    description: &str,
    category: &str,
    default_enabled: bool,
    created_by: &str,
) {
    let flag = FeatureFlag {
        key: key.to_owned(),
        description: Some(description.to_owned()),
        enabled: default_enabled,
        default_enabled,
        category: Some(category.to_owned()),
        last_modified: w.now.to_json_string(),
        last_modified_by: Some(created_by.to_owned()),
    };
    mgr_mut(w).flags.insert(key.ic_key(), flag);
    sync_static_flag(w, key, default_enabled);
    if log_changes(w) {
        info!("Feature flag '{key}' created/updated by {created_by}");
    }
    save_flags(w);
}

pub fn remove_flag(w: &mut World, key: &str, removed_by: &str) -> bool {
    let Some(flag) = mgr_mut(w).flags.shift_remove(&key.ic_key()) else {
        return false;
    };
    sync_static_flag(w, key, flag.default_enabled);
    if log_changes(w) {
        info!("Feature flag '{key}' removed by {removed_by}");
    }
    save_flags(w);
    true
}

/// `SyncStaticFlag`: mirrors a flag into the statics the rest of the server reads.
fn sync_static_flag(w: &mut World, key: &str, enabled: bool) {
    match key.lower_inv().as_str() {
        "player_trading" => w.core.features.player_trading = enabled,
        "pvp_combat" => w.core.features.pvp_combat = enabled,
        "bank_access" => w.core.features.bank_access = enabled,
        "speedhack_detection" => w.core.features.speedhack_detection = enabled,
        "insurance" => w.core.features.insurance_enabled = enabled,
        "vendor_purchase" => w.resources.get_mut::<ContentFeatureFlags>().vendor_purchase = enabled,
        "vendor_sell" => w.resources.get_mut::<ContentFeatureFlags>().vendor_sell = enabled,
        "player_vendors" => w.resources.get_mut::<ContentFeatureFlags>().player_vendors = enabled,
        "house_placement" => w.resources.get_mut::<ContentFeatureFlags>().house_placement = enabled,
        "boat_placement" => w.resources.get_mut::<ContentFeatureFlags>().boat_placement = enabled,
        "bulk_orders" => w.resources.get_mut::<ContentFeatureFlags>().bulk_orders = enabled,
        "passive_detect_hidden" => {
            w.resources
                .get_mut::<ContentFeatureFlags>()
                .passive_detect_hidden = enabled;
            w.resources
                .get_mut::<muo_content::skills::misc_skills::DetectHiddenSettings>()
                .passive_detect_hidden = enabled;
        }
        "young_player_system" => {
            w.resources
                .get_mut::<ContentFeatureFlags>()
                .young_player_system = enabled;
            w.resources
                .get_mut::<muo_content::accounting::AccountHooks>()
                .young_player_system = enabled;
        }
        "bitmap_pathfinding_cache" => {
            w.resources
                .get_mut::<ContentFeatureFlags>()
                .bitmap_pathfinding_cache = enabled
        }
        _ => {} // Custom flags have no static to mirror
    }
}

fn sync_all_static_flags(w: &mut World) {
    let all: Vec<(String, bool)> = mgr(w)
        .map(|m| {
            m.flags
                .values()
                .map(|f| (f.key.clone(), f.enabled))
                .collect()
        })
        .unwrap_or_default();
    for (k, e) in all {
        sync_static_flag(w, &k, e);
    }
}

// ---- gump blocks -------------------------------------------------------------------------------------------------

/// Keeps `GumpBlocks` (read by `SendGump`) at the active blocks: `UpdateGumpBlocksFlag`.
fn update_gump_blocks(w: &mut World) {
    let (active, level) = match mgr(w) {
        Some(m) => (
            m.gump_blocks
                .iter()
                .filter(|(_, e)| e.active)
                .map(|(k, e)| (k.clone(), e.reason.clone()))
                .collect(),
            m.settings.required_access_level,
        ),
        None => (IndexMap::new(), AccessLevel::Administrator),
    };
    let g = w.resources.get_mut::<GumpBlocks>();
    g.blocked = active;
    g.required_access_level = level;
}

/// `typeof(BaseGump).IsAssignableFrom(AssemblyHandler.FindTypeByName(name))`: a full or short name, case-insensitive,
/// of a gump class the crates implement (`gump_types::ALL`) or that is open on any connection.
fn resolve_gump(w: &World, name: &str) -> Option<String> {
    if name.trim().is_empty() {
        return None;
    }
    let matches = |f: &str| {
        if name.contains('.') {
            f.eq_ic(name)
        } else {
            short(f).eq_ic(name)
        }
    };
    let nss: Vec<NetStateId> = w.net_states.keys().copied().collect();
    let open = nss
        .into_iter()
        .flat_map(|ns| gumps_of(w, ns).map(|g| g.type_name()).collect::<Vec<_>>());
    open.chain(muo_content::gumps::gump_types::ALL.iter().copied())
        .find(|f| matches(f))
        .map(str::to_owned)
}

pub fn block_gump_by_name(w: &mut World, name: &str, reason: Option<&str>, by: &str) -> bool {
    let Some(full) = resolve_gump(w, name) else {
        return false;
    };
    let entry = BlockBase {
        resolved_type: Some(full.clone()),
        reason: reason.map(str::to_owned),
        active: true,
        created_at: w.now.to_json_string(),
        created_by: Some(by.to_owned()),
    };
    mgr_mut(w).gump_blocks.insert(full.clone(), entry);
    update_gump_blocks(w);
    if log_changes(w) {
        warn!(
            "Gump '{full}' BLOCKED by {by}. Reason: {}",
            reason.unwrap_or("")
        );
    }
    announce(
        w,
        format!(
            "[Gump Block] '{}' BLOCKED by {by}. Reason: {}",
            short(&full),
            reason.unwrap_or("")
        ),
    );
    save_gump_blocks(w);
    true
}

pub fn unblock_gump(w: &mut World, full: &str, by: &str) -> bool {
    if mgr_mut(w).gump_blocks.shift_remove(full).is_none() {
        return false;
    }
    update_gump_blocks(w);
    if log_changes(w) {
        info!("Gump '{full}' UNBLOCKED by {by}");
    }
    announce(
        w,
        format!("[Gump Block] '{}' UNBLOCKED by {by}", short(full)),
    );
    save_gump_blocks(w);
    true
}

pub fn unblock_gump_by_name(w: &mut World, name: &str, by: &str) -> bool {
    match resolve_gump(w, name) {
        Some(f) => unblock_gump(w, &f, by),
        None => {
            // A blocked gump whose class is no longer open is found among the blocks.
            let found =
                mgr(w).and_then(|m| m.gump_blocks.keys().find(|k| short(k).eq_ic(name)).cloned());
            found.is_some_and(|f| unblock_gump(w, &f, by))
        }
    }
}

pub fn set_gump_block_active(w: &mut World, full: &str, active: bool, by: &str) -> bool {
    let Some(e) = mgr_mut(w).gump_blocks.get_mut(full) else {
        return false;
    };
    e.active = active;
    update_gump_blocks(w);
    let state = if active { "ACTIVE" } else { "INACTIVE" };
    if log_changes(w) {
        info!("Gump block '{full}' set to {state} by {by}");
    }
    announce(
        w,
        format!("[Gump Block] '{}' set to {state} by {by}", short(full)),
    );
    save_gump_blocks(w);
    true
}

// ---- item blocks -------------------------------------------------------------------------------------------------

/// `ResolveItemType`: the full name of an item class.
fn resolve_item_type(w: &World, name: &str) -> Option<String> {
    let t = w
        .types
        .find_type_by_full_name(name, true)
        .or_else(|| w.types.find_type_by_name(name, false, true))?;
    matches!(t, EntityType::Item(_)).then(|| t.full_name().to_owned())
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ItemBlockKind {
    Use,
    Equip,
    Container,
}

impl ItemBlockKind {
    fn label(self) -> &'static str {
        match self {
            Self::Use => "Use",
            Self::Equip => "Equip",
            Self::Container => "Container",
        }
    }
    fn flag(self, e: &mut ItemBlockEntry) -> &mut bool {
        match self {
            Self::Use => &mut e.block_use,
            Self::Equip => &mut e.block_equip,
            Self::Container => &mut e.block_container_access,
        }
    }
}

/// `BlockItemUse/Equip/Container(type, reason, by)`.
pub fn block_item(w: &mut World, kind: ItemBlockKind, full: &str, reason: Option<&str>, by: &str) {
    let now = w.now.to_json_string();
    let m = mgr_mut(w);
    let e = m
        .item_blocks
        .entry(full.to_owned())
        .or_insert_with(|| ItemBlockEntry {
            base: BlockBase {
                resolved_type: Some(full.to_owned()),
                reason: None,
                active: true,
                created_at: now,
                created_by: Some(by.to_owned()),
            },
            ..Default::default()
        });
    *kind.flag(e) = true;
    if let Some(r) = reason {
        e.base.reason = Some(r.to_owned());
    }
    let label = kind.label();
    if log_changes(w) {
        warn!(
            "Item '{full}' {label} BLOCKED by {by}. Reason: {}",
            reason.unwrap_or("")
        );
    }
    announce(
        w,
        format!(
            "[Item Block] '{}' {label} BLOCKED by {by}. Reason: {}",
            short(full),
            reason.unwrap_or("")
        ),
    );
    save_item_blocks(w);
}

pub fn block_item_by_name(
    w: &mut World,
    kind: ItemBlockKind,
    name: &str,
    reason: Option<&str>,
    by: &str,
) -> bool {
    match resolve_item_type(w, name) {
        Some(f) => {
            block_item(w, kind, &f, reason, by);
            true
        }
        None => false,
    }
}

pub fn unblock_item(w: &mut World, kind: ItemBlockKind, full: &str, by: &str) -> bool {
    let m = mgr_mut(w);
    let Some(e) = m.item_blocks.get_mut(full) else {
        return false;
    };
    if !*kind.flag(e) {
        return false;
    }
    *kind.flag(e) = false;
    if !e.block_use && !e.block_equip && !e.block_container_access {
        m.item_blocks.shift_remove(full);
    }
    let label = kind.label();
    if log_changes(w) {
        info!("Item '{full}' {label} UNBLOCKED by {by}");
    }
    announce(
        w,
        format!("[Item Block] '{}' {label} UNBLOCKED by {by}", short(full)),
    );
    save_item_blocks(w);
    true
}

pub fn unblock_item_by_name(w: &mut World, kind: ItemBlockKind, name: &str, by: &str) -> bool {
    resolve_item_type(w, name).is_some_and(|f| unblock_item(w, kind, &f, by))
}

pub fn remove_item_block(w: &mut World, full: &str, by: &str) -> bool {
    if mgr_mut(w).item_blocks.shift_remove(full).is_none() {
        return false;
    }
    if log_changes(w) {
        info!("Item '{full}' block REMOVED by {by}");
    }
    announce(w, format!("[Item Block] '{}' REMOVED by {by}", short(full)));
    save_item_blocks(w);
    true
}

pub fn set_item_block_active(w: &mut World, full: &str, active: bool, by: &str) -> bool {
    let Some(e) = mgr_mut(w).item_blocks.get_mut(full) else {
        return false;
    };
    e.base.active = active;
    let state = if active { "ACTIVE" } else { "INACTIVE" };
    if log_changes(w) {
        info!("Item block '{full}' set to {state} by {by}");
    }
    announce(
        w,
        format!("[Item Block] '{}' set to {state} by {by}", short(full)),
    );
    save_item_blocks(w);
    true
}

fn item_blocked(w: &World, full: &str, kind: ItemBlockKind, default: &str) -> Option<String> {
    let e = mgr(w)?.item_blocks.get(full)?;
    let on = match kind {
        ItemBlockKind::Use => e.block_use,
        ItemBlockKind::Equip => e.block_equip,
        ItemBlockKind::Container => e.block_container_access,
    };
    (e.base.active && on).then(|| e.base.reason.clone().unwrap_or_else(|| default.to_owned()))
}

/// `IsItemUseBlocked(type, out reason)`.
pub fn is_item_use_blocked(w: &World, full: &str) -> Option<String> {
    item_blocked(
        w,
        full,
        ItemBlockKind::Use,
        DEFAULT_ITEM_USE_BLOCKED_MESSAGE,
    )
}
/// `IsItemEquipBlocked(type, out reason)`.
pub fn is_item_equip_blocked(w: &World, full: &str) -> Option<String> {
    item_blocked(
        w,
        full,
        ItemBlockKind::Equip,
        DEFAULT_ITEM_EQUIP_BLOCKED_MESSAGE,
    )
}
/// `IsContainerAccessBlocked(type, out reason)`.
pub fn is_container_access_blocked(w: &World, full: &str) -> Option<String> {
    item_blocked(
        w,
        full,
        ItemBlockKind::Container,
        DEFAULT_CONTAINER_BLOCKED_MESSAGE,
    )
}

// ---- skill blocks ------------------------------------------------------------------------------------------------

fn skill_slot(skill: i32) -> Option<usize> {
    usize::try_from(skill)
        .ok()
        .filter(|&i| i < SkillName::COUNT)
}

pub fn is_skill_blocked(w: &World, skill: SkillName) -> Option<String> {
    let e = mgr(w)?.skill_blocks.get(skill_index(&skill))?.as_ref()?;
    e.base.active.then(|| {
        e.base
            .reason
            .clone()
            .unwrap_or_else(|| DEFAULT_SKILL_DISABLED_MESSAGE.to_owned())
    })
}

pub fn block_skill(w: &mut World, skill: i32, reason: Option<&str>, by: &str) {
    let Some(i) = skill_slot(skill) else { return };
    let e = SkillBlockEntry {
        base: BlockBase {
            resolved_type: None,
            reason: reason.map(str::to_owned),
            active: true,
            created_at: w.now.to_json_string(),
            created_by: Some(by.to_owned()),
        },
        skill: serde_json::Value::String(skill_name(skill)),
    };
    mgr_mut(w).skill_blocks[i] = Some(e);
    let n = skill_name(skill);
    if log_changes(w) {
        warn!(
            "Skill '{n}' BLOCKED by {by}. Reason: {}",
            reason.unwrap_or("")
        );
    }
    announce(
        w,
        format!(
            "[Skill Block] '{n}' BLOCKED by {by}. Reason: {}",
            reason.unwrap_or("")
        ),
    );
    save_skill_blocks(w);
}

pub fn unblock_skill(w: &mut World, skill: i32, by: &str) -> bool {
    let Some(i) = skill_slot(skill) else {
        return false;
    };
    if mgr_mut(w).skill_blocks[i].take().is_none() {
        return false;
    }
    let n = skill_name(skill);
    if log_changes(w) {
        info!("Skill '{n}' UNBLOCKED by {by}");
    }
    announce(w, format!("[Skill Block] '{n}' UNBLOCKED by {by}"));
    save_skill_blocks(w);
    true
}

pub fn set_skill_block_active(w: &mut World, skill: i32, active: bool, by: &str) -> bool {
    let Some(i) = skill_slot(skill) else {
        return false;
    };
    let Some(e) = mgr_mut(w).skill_blocks[i].as_mut() else {
        return false;
    };
    e.base.active = active;
    let (n, state) = (
        skill_name(skill),
        if active { "ACTIVE" } else { "INACTIVE" },
    );
    if log_changes(w) {
        info!("Skill block '{n}' set to {state} by {by}");
    }
    announce(w, format!("[Skill Block] '{n}' set to {state} by {by}"));
    save_skill_blocks(w);
    true
}

// ---- spell blocks ------------------------------------------------------------------------------------------------

fn spell_id_of(w: &World, full: &str) -> Option<usize> {
    let id = w.resources.get::<SpellRegistry>()?.registry_number(full);
    usize::try_from(id).ok().filter(|&i| i < SPELL_CAPACITY)
}

/// `GetSpellBlockEntry(spellType)` when active: `IsSpellBlocked(Type)` and the entry's reason (None: the default text).
pub fn spell_blocked(w: &World, full: &str) -> Option<Option<String>> {
    let id = spell_id_of(w, full)?;
    let e = mgr(w)?.spell_blocks.get(id)?.as_ref()?;
    e.active.then(|| e.reason.clone())
}

pub fn block_spell_by_name(w: &mut World, name: &str, reason: Option<&str>, by: &str) -> bool {
    let Some(full) = resolve_spell(w, name) else {
        return false;
    };
    if let Some(id) = spell_id_of(w, &full) {
        let e = BlockBase {
            resolved_type: Some(full.clone()),
            reason: reason.map(str::to_owned),
            active: true,
            created_at: w.now.to_json_string(),
            created_by: Some(by.to_owned()),
        };
        mgr_mut(w).spell_blocks[id] = Some(e);
        if log_changes(w) {
            warn!(
                "Spell '{full}' BLOCKED by {by}. Reason: {}",
                reason.unwrap_or("")
            );
        }
        announce(
            w,
            format!(
                "[Spell Block] '{}' BLOCKED by {by}. Reason: {}",
                short(&full),
                reason.unwrap_or("")
            ),
        );
        save_spell_blocks(w);
    }
    true
}

pub fn unblock_spell(w: &mut World, full: &str, by: &str) -> bool {
    let Some(id) = spell_id_of(w, full) else {
        return false;
    };
    if mgr_mut(w).spell_blocks[id].take().is_none() {
        return false;
    }
    if log_changes(w) {
        info!("Spell '{}' UNBLOCKED by {by}", short(full));
    }
    announce(
        w,
        format!("[Spell Block] '{}' UNBLOCKED by {by}", short(full)),
    );
    save_spell_blocks(w);
    true
}

pub fn unblock_spell_by_name(w: &mut World, name: &str, by: &str) -> bool {
    resolve_spell(w, name).is_some_and(|f| unblock_spell(w, &f, by))
}

pub fn set_spell_block_active(w: &mut World, id: usize, active: bool, by: &str) -> bool {
    let Some(e) = mgr_mut(w).spell_blocks.get_mut(id).and_then(Option::as_mut) else {
        return false;
    };
    e.active = active;
    let n = e
        .resolved_type
        .as_deref()
        .map(short)
        .unwrap_or("")
        .to_owned();
    let state = if active { "ACTIVE" } else { "INACTIVE" };
    if log_changes(w) {
        info!("Spell block '{n}' set to {state} by {by}");
    }
    announce(w, format!("[Spell Block] '{n}' set to {state} by {by}"));
    save_spell_blocks(w);
    true
}

// ---- hooks -------------------------------------------------------------------------------------------------------

fn below_required(w: &World, m: MobileId) -> bool {
    let req = mgr(w).map_or(AccessLevel::Administrator, |m| {
        m.settings.required_access_level
    });
    w.mobile(m).is_none_or(|x| x.access_level < req)
}

/// Sets every hook other systems read the flags through; registers the commands.
pub fn configure(w: &mut World) {
    use muo_content::player_mobile::hooks::SystemHooks;
    use muo_content::vendor::{Feature, VendorHooks};

    w.resources.get_mut::<FeatureFlagManager>();
    w.resources.get_mut::<ContentFeatureFlags>();

    {
        let h = w.resources.get_mut::<SystemHooks>();
        h.item_equip_blocked = Some(|w, m, item| {
            if !below_required(w, m) {
                return None;
            }
            let full = w.item(item)?.ty.full_name;
            is_item_equip_blocked(w, full)
        });
        h.item_use_blocked = Some(|w, m, item| {
            if !below_required(w, m) {
                return None;
            }
            let full = w.item(item)?.ty.full_name;
            is_item_use_blocked(w, full)
        });
        h.skill_blocked = Some(|w, m, skill| {
            if below_required(w, m) {
                is_skill_blocked(w, skill)
            } else {
                None
            }
        });
        h.young_player_system = Some(|w| {
            w.resources
                .get::<ContentFeatureFlags>()
                .is_none_or(|c| c.young_player_system)
        });
    }
    {
        let c = muo_content::items::common::hooks_mut(w);
        c.item_use_blocked = Some(is_item_use_blocked);
        c.container_access_blocked = Some(is_container_access_blocked);
    }
    w.resources.get_mut::<VendorHooks>().feature_enabled = Some(|w, f| {
        let c = w
            .resources
            .get::<ContentFeatureFlags>()
            .copied()
            .unwrap_or_default();
        match f {
            Feature::VendorPurchase => c.vendor_purchase,
            Feature::VendorSell => c.vendor_sell,
            Feature::BulkOrders => c.bulk_orders,
        }
    });
    muo_content::items::house_deps::deps_mut(w).house_placement_blocked = Some(|w, from| {
        let off = w
            .resources
            .get::<ContentFeatureFlags>()
            .is_some_and(|c| !c.house_placement);
        off && below_required(w, from)
    });
    muo_content::items::house_deps::deps_mut(w).boat_placement_blocked = Some(|w, from| {
        let off = w
            .resources
            .get::<ContentFeatureFlags>()
            .is_some_and(|c| !c.boat_placement);
        off && below_required(w, from)
    });
    muo_content::spells::hooks::hooks_mut(w).spell_blocked_reason = Some(spell_blocked);

    register_commands(w);
}

// ---- commands ----------------------------------------------------------------------------------------------------

fn reg(
    w: &mut World,
    name: &str,
    level: AccessLevel,
    aliases: &[&str],
    f: fn(&mut World, &mut CommandEventArgs),
) {
    CommandSystem::register(w, name, level, aliases, Rc::new(f));
}

fn register_commands(w: &mut World) {
    use AccessLevel::{Administrator as A, GameMaster as G};
    reg(w, "FeatureFlag", A, &["FF"], feature_flag_command);
    reg(w, "BlockGump", A, &[], block_gump_command);
    reg(w, "UnblockGump", A, &[], unblock_gump_command);
    reg(w, "BlockItemUse", A, &[], |w, e| {
        block_item_command(w, e, ItemBlockKind::Use)
    });
    reg(w, "BlockItemEquip", A, &[], |w, e| {
        block_item_command(w, e, ItemBlockKind::Equip)
    });
    reg(w, "BlockItemContainer", A, &[], |w, e| {
        block_item_command(w, e, ItemBlockKind::Container)
    });
    reg(w, "UnblockItemUse", A, &[], |w, e| {
        unblock_item_command(w, e, ItemBlockKind::Use)
    });
    reg(w, "UnblockItemEquip", A, &[], |w, e| {
        unblock_item_command(w, e, ItemBlockKind::Equip)
    });
    reg(w, "UnblockItemContainer", A, &[], |w, e| {
        unblock_item_command(w, e, ItemBlockKind::Container)
    });
    reg(w, "BlockSkill", A, &[], block_skill_command);
    reg(w, "UnblockSkill", A, &[], unblock_skill_command);
    reg(w, "BlockSpell", A, &[], block_spell_command);
    reg(w, "UnblockSpell", A, &[], unblock_spell_command);
    reg(w, "FeatureList", G, &[], feature_list_command);
    reg(w, "FeatureAdmin", A, &[], |w, e| open_admin(w, e.mobile));
    reg(w, "ListGumps", G, &[], list_gumps_command);
}

fn mobile_name(w: &World, m: MobileId) -> String {
    w.mobile(m)
        .and_then(|x| (x.ty.name)(w, m))
        .unwrap_or_default()
}

fn args_of(e: &CommandEventArgs) -> Vec<String> {
    (0..e.length() as i32).map(|i| e.get_string(i)).collect()
}

fn rest(a: &[String], from: usize) -> Option<String> {
    (a.len() > from).then(|| a[from..].join(" "))
}

fn msg_hue(w: &mut World, to: MobileId, hue: i32, text: &str) {
    send_message_hue(w, to, hue, text);
}

fn feature_flag_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let a = args_of(e);
    if a.is_empty() {
        open_admin(w, from);
        return;
    }
    let key = a[0].lower_inv();
    let action = a
        .get(1)
        .map_or_else(|| "info".to_owned(), |s| s.lower_inv());
    let by = mobile_name(w, from);
    match action.as_str() {
        "on" | "enable" | "true" | "1" | "off" | "disable" | "false" | "0" => {
            let on = matches!(action.as_str(), "on" | "enable" | "true" | "1");
            if set_flag(w, &key, on, &by) {
                let t = format!(
                    "Feature flag '{key}' has been {}.",
                    if on { "ENABLED" } else { "DISABLED" }
                );
                msg_hue(w, from, 0x35, &t);
            } else {
                msg_hue(w, from, 0x22, &format!("Feature flag '{key}' not found."));
            }
        }
        "toggle" => match get_flag(w, &key) {
            Some(f) => {
                set_flag(w, &key, !f.enabled, &by);
                // `flag` is the live object, so it reads the new state
                let now = !f.enabled;
                msg_hue(
                    w,
                    from,
                    0x35,
                    &format!(
                        "Feature flag '{key}' toggled to {}.",
                        if now { "ENABLED" } else { "DISABLED" }
                    ),
                );
            }
            None => msg_hue(w, from, 0x22, &format!("Feature flag '{key}' not found.")),
        },
        "create" => {
            if a.len() < 4 {
                mx::send_message(
                    w,
                    from,
                    "Usage: [FeatureFlag <key> create <category> <description>",
                );
                return;
            }
            let description = a[3..].join(" ");
            create_or_update_flag(w, &key, &description, &a[2], true, &by);
            msg_hue(w, from, 0x35, &format!("Feature flag '{key}' created."));
        }
        "delete" | "remove" => {
            if remove_flag(w, &key, &by) {
                msg_hue(w, from, 0x35, &format!("Feature flag '{key}' removed."));
            } else {
                msg_hue(w, from, 0x22, &format!("Feature flag '{key}' not found."));
            }
        }
        _ => match get_flag(w, &key) {
            Some(f) => {
                msg_hue(w, from, 0x35, &format!("=== Feature Flag: {} ===", f.key));
                mx::send_message(
                    w,
                    from,
                    &format!("Enabled: {}", if f.enabled { "Yes" } else { "No" }),
                );
                mx::send_message(
                    w,
                    from,
                    &format!("Default: {}", if f.default_enabled { "Yes" } else { "No" }),
                );
                mx::send_message(
                    w,
                    from,
                    &format!("Category: {}", f.category.as_deref().unwrap_or("")),
                );
                mx::send_message(
                    w,
                    from,
                    &format!("Description: {}", f.description.as_deref().unwrap_or("")),
                );
                let when = muo_primitives::DateTime::parse_iso(&f.last_modified)
                    .map_or(f.last_modified.clone(), |d| d.to_string());
                mx::send_message(
                    w,
                    from,
                    &format!(
                        "Last Modified: {when} by {}",
                        f.last_modified_by.as_deref().unwrap_or("")
                    ),
                );
            }
            None => {
                msg_hue(w, from, 0x22, &format!("Feature flag '{key}' not found."));
                mx::send_message(w, from, "Use [FeatureList to see all available flags.");
            }
        },
    }
}

fn block_gump_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let a = args_of(e);
    if a.is_empty() {
        mx::send_message(w, from, "Usage: [BlockGump <typeName> [reason]");
        mx::send_message(
            w,
            from,
            "Example: [BlockGump CraftGump Crafting temporarily disabled",
        );
        return;
    }
    let reason = rest(&a, 1);
    let by = mobile_name(w, from);
    if block_gump_by_name(w, &a[0], reason.as_deref(), &by) {
        msg_hue(w, from, 0x35, &format!("Gump '{}' has been BLOCKED.", a[0]));
        if let Some(r) = reason {
            mx::send_message(w, from, &format!("Reason: {r}"));
        }
    } else {
        msg_hue(
            w,
            from,
            0x22,
            &format!("Could not find gump type '{}'.", a[0]),
        );
        mx::send_message(
            w,
            from,
            "Make sure you're using the correct type name (e.g., CraftGump, HelpGump, etc.)",
        );
    }
}

fn unblock_gump_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let a = args_of(e);
    if a.is_empty() {
        mx::send_message(w, from, "Usage: [UnblockGump <typeName>");
        mx::send_message(w, from, "Use [FeatureList gumps to see blocked gumps.");
        return;
    }
    let by = mobile_name(w, from);
    if unblock_gump_by_name(w, &a[0], &by) {
        msg_hue(
            w,
            from,
            0x35,
            &format!("Gump block for '{}' has been REMOVED.", a[0]),
        );
    } else {
        msg_hue(
            w,
            from,
            0x22,
            &format!("No block found for gump '{}'.", a[0]),
        );
    }
}

fn block_item_command(w: &mut World, e: &mut CommandEventArgs, kind: ItemBlockKind) {
    let from = e.mobile;
    let a = args_of(e);
    let action = kind.label();
    if a.is_empty() {
        mx::send_message(
            w,
            from,
            &format!("Usage: [BlockItem{action} <typeName> [reason]"),
        );
        mx::send_message(
            w,
            from,
            &format!("Or target an item: [BlockItem{action} target [reason]"),
        );
        return;
    }
    let reason = rest(&a, 1);
    if a[0].eq_ic("target") {
        mx::send_message(w, from, "Target the item to block:");
        target::begin_target(w, from, -1, false, TargetFlags::NONE, move |w, from, t| {
            if let Targeted::Entity(EntityId::Item(i)) = t
                && let Some(ty) = w.item(i).map(|x| x.ty)
            {
                let by = mobile_name(w, from);
                block_item(w, kind, ty.full_name, reason.as_deref(), &by);
                msg_hue(
                    w,
                    from,
                    0x35,
                    &format!("Item '{}' {action} has been BLOCKED.", short(ty.full_name)),
                );
                if let Some(r) = &reason {
                    mx::send_message(w, from, &format!("Reason: {r}"));
                }
            } else {
                msg_hue(w, from, 0x22, "That is not an item.");
            }
        });
        return;
    }
    let by = mobile_name(w, from);
    if block_item_by_name(w, kind, &a[0], reason.as_deref(), &by) {
        msg_hue(
            w,
            from,
            0x35,
            &format!("Item '{}' {action} has been BLOCKED.", a[0]),
        );
        if let Some(r) = reason {
            mx::send_message(w, from, &format!("Reason: {r}"));
        }
    } else {
        msg_hue(
            w,
            from,
            0x22,
            &format!("Could not find item type '{}'.", a[0]),
        );
    }
}

fn unblock_item_command(w: &mut World, e: &mut CommandEventArgs, kind: ItemBlockKind) {
    let from = e.mobile;
    let a = args_of(e);
    let action = kind.label();
    if a.is_empty() {
        mx::send_message(w, from, &format!("Usage: [UnblockItem{action} <typeName>"));
        mx::send_message(w, from, "Use [FeatureList items to see blocked items.");
        return;
    }
    let by = mobile_name(w, from);
    if unblock_item_by_name(w, kind, &a[0], &by) {
        msg_hue(
            w,
            from,
            0x35,
            &format!("Item '{}' {action} block has been REMOVED.", a[0]),
        );
    } else {
        msg_hue(
            w,
            from,
            0x22,
            &format!("No {action} block found for item '{}'.", a[0]),
        );
    }
}

fn block_skill_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let a = args_of(e);
    if a.is_empty() {
        mx::send_message(w, from, "Usage: [BlockSkill <SkillName> [reason]");
        mx::send_message(w, from, "Example: [BlockSkill Magery Investigating exploit");
        return;
    }
    let Some(skill) = parse_skill(&a[0]) else {
        msg_hue(w, from, 0x22, &format!("Unknown skill name '{}'.", a[0]));
        mx::send_message(
            w,
            from,
            "Valid skills: Alchemy, Anatomy, Magery, Mining, etc.",
        );
        return;
    };
    let reason = rest(&a, 1);
    let by = mobile_name(w, from);
    block_skill(w, skill, reason.as_deref(), &by);
    msg_hue(
        w,
        from,
        0x35,
        &format!("Skill '{}' has been BLOCKED.", skill_name(skill)),
    );
    if let Some(r) = reason {
        mx::send_message(w, from, &format!("Reason: {r}"));
    }
}

fn unblock_skill_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let a = args_of(e);
    if a.is_empty() {
        mx::send_message(w, from, "Usage: [UnblockSkill <SkillName>");
        mx::send_message(w, from, "Use [FeatureList skills to see blocked skills.");
        return;
    }
    let Some(skill) = parse_skill(&a[0]) else {
        msg_hue(w, from, 0x22, &format!("Unknown skill name '{}'.", a[0]));
        return;
    };
    let by = mobile_name(w, from);
    if unblock_skill(w, skill, &by) {
        msg_hue(
            w,
            from,
            0x35,
            &format!("Skill block for '{}' has been REMOVED.", skill_name(skill)),
        );
    } else {
        msg_hue(
            w,
            from,
            0x22,
            &format!("No block found for skill '{}'.", skill_name(skill)),
        );
    }
}

fn block_spell_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let a = args_of(e);
    if a.is_empty() {
        mx::send_message(w, from, "Usage: [BlockSpell <SpellTypeName> [reason]");
        mx::send_message(
            w,
            from,
            "Example: [BlockSpell RecallSpell Investigating exploit",
        );
        return;
    }
    let reason = rest(&a, 1);
    let by = mobile_name(w, from);
    if block_spell_by_name(w, &a[0], reason.as_deref(), &by) {
        msg_hue(
            w,
            from,
            0x35,
            &format!("Spell '{}' has been BLOCKED.", a[0]),
        );
        if let Some(r) = reason {
            mx::send_message(w, from, &format!("Reason: {r}"));
        }
    } else {
        msg_hue(
            w,
            from,
            0x22,
            &format!("Could not find spell type '{}'.", a[0]),
        );
        mx::send_message(
            w,
            from,
            "Make sure you're using the correct type name (e.g., RecallSpell, GateTravelSpell, etc.)",
        );
    }
}

fn unblock_spell_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let a = args_of(e);
    if a.is_empty() {
        mx::send_message(w, from, "Usage: [UnblockSpell <SpellTypeName>");
        mx::send_message(w, from, "Use [FeatureList spells to see blocked spells.");
        return;
    }
    let by = mobile_name(w, from);
    if unblock_spell_by_name(w, &a[0], &by) {
        msg_hue(
            w,
            from,
            0x35,
            &format!("Spell block for '{}' has been REMOVED.", a[0]),
        );
    } else {
        msg_hue(
            w,
            from,
            0x22,
            &format!("No block found for spell '{}'.", a[0]),
        );
    }
}

fn sorted_flags(w: &World) -> Vec<FeatureFlag> {
    let mut f = all_flags(w);
    muo_primitives::dotnet::net_sort(&mut f, |a, b| {
        let (ca, cb) = (
            a.category.as_deref().unwrap_or(""),
            b.category.as_deref().unwrap_or(""),
        );
        muo_primitives::dotnet::cmp_ignore_case(ca, cb)
            .then_with(|| muo_primitives::dotnet::cmp_ignore_case(&a.key, &b.key))
    });
    f
}

fn feature_list_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    let a = args_of(e);
    let filter = a
        .first()
        .map_or_else(|| "all".to_owned(), |s| s.lower_inv());
    let is = |n: &str| filter == n || filter == "all";
    let status = |active: bool| if active { "[OFF]" } else { "[ON]" };

    if is("flags") {
        let flags = sorted_flags(w);
        msg_hue(
            w,
            from,
            0x35,
            &format!("=== Feature Flags ({}) ===", flags.len()),
        );
        for f in flags {
            let s = if f.enabled { "[ON]" } else { "[OFF]" };
            mx::send_message(
                w,
                from,
                &format!(
                    "  {s} {} ({}): {}",
                    f.key,
                    f.category.as_deref().unwrap_or(""),
                    f.description.as_deref().unwrap_or("")
                ),
            );
        }
    }
    if is("gumps") {
        let rows = block_rows(w, Page::GumpBlocks);
        msg_hue(
            w,
            from,
            0x35,
            &format!("=== Blocked Gumps ({}) ===", rows.len()),
        );
        for r in rows {
            mx::send_message(
                w,
                from,
                &format!(
                    "  {} {}: {}",
                    status(r.active),
                    r.name,
                    r.reason
                        .unwrap_or_else(|| DEFAULT_GUMP_BLOCKED_MESSAGE.to_owned())
                ),
            );
        }
    }
    if is("items") {
        let rows = block_rows(w, Page::ItemBlocks);
        msg_hue(
            w,
            from,
            0x35,
            &format!("=== Blocked Items ({}) ===", rows.len()),
        );
        for r in rows {
            let mut actions = Vec::new();
            if r.block_use {
                actions.push("Use");
            }
            if r.block_equip {
                actions.push("Equip");
            }
            if r.block_container {
                actions.push("Container");
            }
            let acts = if actions.is_empty() {
                "[none]".to_owned()
            } else {
                format!("[{}]", actions.join("]["))
            };
            mx::send_message(
                w,
                from,
                &format!(
                    "  {} {} {acts}: {}",
                    status(r.active),
                    r.name,
                    r.reason
                        .unwrap_or_else(|| DEFAULT_ITEM_USE_BLOCKED_MESSAGE.to_owned())
                ),
            );
        }
    }
    if is("skills") {
        msg_hue(w, from, 0x35, "=== Blocked Skills ===");
        for r in block_rows(w, Page::SkillBlocks) {
            mx::send_message(
                w,
                from,
                &format!(
                    "  {} {}: {}",
                    status(r.active),
                    r.name,
                    r.reason
                        .unwrap_or_else(|| DEFAULT_SKILL_DISABLED_MESSAGE.to_owned())
                ),
            );
        }
    }
    if is("spells") {
        msg_hue(w, from, 0x35, "=== Blocked Spells ===");
        for r in block_rows(w, Page::SpellBlocks) {
            mx::send_message(
                w,
                from,
                &format!(
                    "  {} {}: {}",
                    status(r.active),
                    r.name,
                    r.reason
                        .unwrap_or_else(|| DEFAULT_SPELL_DISABLED_MESSAGE.to_owned())
                ),
            );
        }
    }
    if !matches!(
        filter.as_str(),
        "flags" | "gumps" | "items" | "skills" | "spells" | "all"
    ) {
        mx::send_message(
            w,
            from,
            "Usage: [FeatureList [flags|gumps|items|skills|spells|all]",
        );
    }
}

fn list_gumps_for(w: &mut World, from: MobileId, target: MobileId) {
    let Some(ns) = w.mobile(target).and_then(|m| m.net_state) else {
        msg_hue(w, from, 0x22, "That player is not online.");
        return;
    };
    let list: Vec<&'static str> = gumps_of(w, ns).map(|g| g.type_name()).collect();
    let name = mobile_name(w, target);
    if list.is_empty() {
        msg_hue(w, from, 0x35, &format!("{name} has no gumps open."));
        return;
    }
    msg_hue(
        w,
        from,
        0x35,
        &format!("=== Open Gumps for {name} ({}) ===", list.len()),
    );
    for full in list {
        mx::send_message(w, from, &format!("  \u{2022} {}", short(full)));
        mx::send_message(w, from, &format!("      Full: {full}"));
        msg_hue(
            w,
            from,
            0x3B2,
            &format!("      Block with: [BlockGump {}", short(full)),
        );
    }
}

fn list_gumps_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() > 0 && e.get_string(0).eq_ic("self") {
        list_gumps_for(w, from, from);
    } else {
        mx::send_message(
            w,
            from,
            "Target a player to list their open gumps (or use [ListGumps self):",
        );
        target::begin_target(w, from, -1, false, TargetFlags::NONE, |w, from, t| {
            if let Targeted::Entity(EntityId::Mobile(m)) = t {
                list_gumps_for(w, from, m);
            } else {
                msg_hue(w, from, 0x22, "That is not a player.");
            }
        });
    }
}

// ---- FeatureFlagAdminGump ----------------------------------------------------------------------------------------

const FLAGS_PER_PAGE: usize = 10;
const BLOCKS_PER_PAGE: usize = 7;
const FLAG_ROW_HEIGHT: i32 = 25;
const BLOCK_ROW_HEIGHT: i32 = 45;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Page {
    Flags,
    GumpBlocks,
    ItemBlocks,
    SkillBlocks,
    SpellBlocks,
}

impl Page {
    fn from_button(b: i32) -> Option<Page> {
        Some(match b {
            1 => Page::Flags,
            2 => Page::GumpBlocks,
            3 => Page::ItemBlocks,
            4 => Page::SkillBlocks,
            5 => Page::SpellBlocks,
            _ => return None,
        })
    }
}

/// One row of a block page, with the key the toggle/remove buttons act on.
#[derive(Clone, Debug)]
struct BlockRow {
    /// The full type name, or the skill/spell number.
    key: String,
    name: String,
    reason: Option<String>,
    active: bool,
    block_use: bool,
    block_equip: bool,
    block_container: bool,
}

fn block_rows(w: &World, page: Page) -> Vec<BlockRow> {
    let Some(m) = mgr(w) else { return Vec::new() };
    let base = |key: String, name: String, b: &BlockBase| BlockRow {
        key,
        name,
        reason: b.reason.clone(),
        active: b.active,
        block_use: false,
        block_equip: false,
        block_container: false,
    };
    match page {
        Page::Flags => Vec::new(),
        Page::GumpBlocks => m
            .gump_blocks
            .iter()
            .map(|(k, b)| base(k.clone(), short(k).to_owned(), b))
            .collect(),
        Page::ItemBlocks => m
            .item_blocks
            .iter()
            .map(|(k, e)| BlockRow {
                block_use: e.block_use,
                block_equip: e.block_equip,
                block_container: e.block_container_access,
                ..base(k.clone(), short(k).to_owned(), &e.base)
            })
            .collect(),
        Page::SkillBlocks => m
            .skill_blocks
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                e.as_ref()
                    .map(|e| base(i.to_string(), skill_name(i as i32), &e.base))
            })
            .collect(),
        Page::SpellBlocks => m
            .spell_blocks
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                e.as_ref().map(|e| {
                    base(
                        i.to_string(),
                        e.resolved_type
                            .as_deref()
                            .map(short)
                            .unwrap_or("")
                            .to_owned(),
                        e,
                    )
                })
            })
            .collect(),
    }
}

/// `FeatureFlagAdminGump`: a `DynamicGump` over a snapshot of what its page lists.
pub struct AdminGump {
    page: Page,
    page_index: usize,
    flags: Vec<FeatureFlag>,
    blocks: Vec<BlockRow>,
}

impl AdminGump {
    fn snapshot(w: &World, page: Page, page_index: usize) -> Self {
        let flags = if page == Page::Flags {
            sorted_flags(w)
        } else {
            Vec::new()
        };
        Self {
            page,
            page_index,
            flags,
            blocks: block_rows(w, page),
        }
    }
}

/// `from.SendGump(new FeatureFlagAdminGump(page, pageIndex))`.
pub fn open_admin(w: &mut World, from: MobileId) {
    send_admin(w, from, Page::Flags, 0);
}

fn send_admin(w: &mut World, to: MobileId, page: Page, page_index: usize) {
    let g = AdminGump::snapshot(w, page, page_index);
    send_gump_to_mobile(w, to, Box::new(DynamicGumpObj::new(50, 50, g)), false);
}

fn pages(count: usize, per: usize) -> usize {
    count.div_ceil(per).max(1)
}

impl AdminGump {
    fn add_pagination(&self, b: &mut DynamicGumpBuilder, total: usize) {
        if total <= 1 {
            return;
        }
        if self.page_index > 0 {
            b.add_button(300, 425, 4014, 4016, 102);
            b.add_html(
                335,
                425,
                50,
                20,
                &html::color("Prev", tc::WHITE),
                false,
                false,
            );
        }
        b.add_html(
            370,
            425,
            60,
            20,
            &html::color(&format!("{}/{}", self.page_index + 1, total), tc::WHITE),
            false,
            false,
        );
        if self.page_index < total - 1 {
            b.add_button(420, 425, 4005, 4007, 103);
            b.add_html(
                455,
                425,
                50,
                20,
                &html::color("Next", tc::WHITE),
                false,
                false,
            );
        }
    }

    fn flags_page(&self, b: &mut DynamicGumpBuilder) {
        let h = |b: &mut DynamicGumpBuilder, x, w, t: &str| {
            b.add_html(x, 80, w, 20, &html::color(t, tc::WHITE), false, false)
        };
        h(b, 20, 150, "Flag");
        h(b, 180, 150, "Category");
        h(b, 275, 350, "Description");
        h(b, 690, 60, "Status");
        let start = self.page_index * FLAGS_PER_PAGE;
        let end = (start + FLAGS_PER_PAGE).min(self.flags.len());
        let mut y = 105;
        for (n, f) in self.flags.get(start..end).unwrap_or(&[]).iter().enumerate() {
            let status = if f.enabled { tc::GREEN } else { tc::RED };
            b.add_button(
                20,
                y,
                if f.enabled { 2154 } else { 2151 },
                if f.enabled { 2151 } else { 2154 },
                1000 + n as i32,
            );
            b.add_html(
                60,
                y + 3,
                130,
                20,
                &html::color(&f.key, tc::WHITE),
                false,
                false,
            );
            b.add_html(
                180,
                y + 3,
                150,
                20,
                &html::color(f.category.as_deref().unwrap_or(""), tc::LIGHT_GRAY),
                false,
                false,
            );
            b.add_html(
                275,
                y + 3,
                350,
                20,
                &html::color(f.description.as_deref().unwrap_or(""), tc::LIGHT_GRAY),
                false,
                false,
            );
            b.add_html(
                690,
                y + 3,
                60,
                20,
                &html::color(if f.enabled { "ON" } else { "OFF" }, status),
                false,
                false,
            );
            y += FLAG_ROW_HEIGHT;
        }
        self.add_pagination(b, pages(self.flags.len(), FLAGS_PER_PAGE));
    }

    fn block_page(&self, b: &mut DynamicGumpBuilder, header: &str, help: &str, items: bool) {
        let h = |b: &mut DynamicGumpBuilder, x, w, t: &str| {
            b.add_html(x, 80, w, 20, &html::color(t, tc::WHITE), false, false)
        };
        if items {
            h(b, 20, 150, "Item Type");
            h(b, 180, 280, "Reason");
            h(b, 530, 50, "Use");
            h(b, 580, 50, "Equip");
            h(b, 640, 50, "Open");
            h(b, 700, 40, "Edit");
            h(b, 750, 60, "Remove");
        } else {
            h(b, 20, 180, header);
            h(b, 200, 400, "Reason");
            h(b, 690, 60, "Status");
            h(b, 750, 60, "Remove");
        }
        let start = self.page_index * BLOCKS_PER_PAGE;
        let mut y = 105;
        for (n, r) in self
            .blocks
            .iter()
            .skip(start)
            .take(BLOCKS_PER_PAGE)
            .enumerate()
        {
            let n = n as i32;
            let reason = html::color(r.reason.as_deref().unwrap_or("(default)"), tc::LIGHT_GRAY);
            b.add_button(
                20,
                y,
                if r.active { 2151 } else { 2154 },
                if r.active { 2154 } else { 2151 },
                2000 + n,
            );
            if items {
                b.add_html(
                    60,
                    y + 3,
                    120,
                    20,
                    &html::color(&r.name, tc::WHITE),
                    false,
                    false,
                );
                b.add_html(180, y + 3, 280, 40, &reason, false, false);
                for (x, on) in [
                    (530, r.block_use),
                    (580, r.block_equip),
                    (640, r.block_container),
                ] {
                    let c = if on { tc::RED } else { tc::GREEN };
                    b.add_html(
                        x,
                        y + 3,
                        50,
                        20,
                        &html::color(if on { "X" } else { "-" }, c),
                        false,
                        false,
                    );
                }
                b.add_button(700, y + 3, 4011, 4013, 4000 + n);
            } else {
                let status = if r.active { tc::RED } else { tc::GREEN };
                b.add_html(
                    60,
                    y + 3,
                    150,
                    20,
                    &html::color(&r.name, tc::WHITE),
                    false,
                    false,
                );
                b.add_html(200, y + 3, 400, 40, &reason, false, false);
                b.add_html(
                    690,
                    y + 3,
                    60,
                    20,
                    &html::color(if r.active { "OFF" } else { "ON" }, status),
                    false,
                    false,
                );
            }
            b.add_button(750, y + 3, 4017, 4019, 3000 + n);
            y += BLOCK_ROW_HEIGHT;
        }
        self.add_pagination(b, pages(self.blocks.len(), BLOCKS_PER_PAGE));
        b.add_html(
            20,
            430,
            400,
            20,
            &html::color(help, tc::LIGHT_GRAY),
            false,
            false,
        );
    }
}

impl DynamicGumpLogic for AdminGump {
    const TYPE_NAME: &'static str = "Server.Systems.FeatureFlags.FeatureFlagAdminGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 820, 500, 9270);
        b.add_html(
            0,
            15,
            820,
            25,
            &html::center_color("Feature Flag Administration", "#00FF00"),
            false,
            false,
        );

        let tabs = [
            (Page::Flags, 20, 55, 80, "Flags"),
            (Page::GumpBlocks, 140, 175, 100, "Gumps"),
            (Page::ItemBlocks, 270, 305, 80, "Items"),
            (Page::SkillBlocks, 390, 425, 80, "Skills"),
            (Page::SpellBlocks, 510, 545, 80, "Spells"),
        ];
        for (i, (p, bx, tx, tw, label)) in tabs.into_iter().enumerate() {
            let color = if self.page == p {
                tc::YELLOW
            } else {
                tc::WHITE
            };
            b.add_button(bx, 45, 4005, 4007, i as i32 + 1);
            b.add_html(tx, 47, tw, 20, &html::color(label, color), false, false);
        }
        b.add_alpha_region(15, 75, 790, 380);

        match self.page {
            Page::Flags => self.flags_page(b),
            Page::GumpBlocks => {
                self.block_page(b, "Gump Type", "Use [BlockGump to add new blocks", false)
            }
            Page::ItemBlocks => self.block_page(b, "", "Use [BlockItem to add new blocks", true),
            Page::SkillBlocks => {
                self.block_page(b, "Skill", "Use [BlockSkill to add new blocks", false)
            }
            Page::SpellBlocks => {
                self.block_page(b, "Spell Type", "Use [BlockSpell to add new blocks", false)
            }
        }

        b.add_button(770, 460, 4017, 4019, 0);
        b.add_html(
            720,
            462,
            50,
            20,
            &html::color("Close", tc::WHITE),
            false,
            false,
        );
        b.add_button(20, 460, 4023, 4025, 100);
        b.add_html(
            55,
            462,
            50,
            20,
            &html::color("Save", tc::WHITE),
            false,
            false,
        );
        b.add_button(120, 460, 4014, 4016, 101);
        b.add_html(
            155,
            462,
            60,
            20,
            &html::color("Refresh", tc::WHITE),
            false,
            false,
        );
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let b = info.button_id;
        if b == 0 {
            return;
        }
        let name = mobile_name(w, from);
        if let Some(p) = Page::from_button(b) {
            send_admin(w, from, p, 0);
            return;
        }
        match b {
            100 => {
                save(w);
                msg_hue(w, from, 0x35, "Feature flags saved to disk.");
                send_admin(w, from, self.page, self.page_index);
                return;
            }
            101 => {
                send_admin(w, from, self.page, self.page_index);
                return;
            }
            102 => {
                // `_pageIndex - 1` can only come from a Prev button, which is drawn only above page 0
                send_admin(w, from, self.page, self.page_index.saturating_sub(1));
                return;
            }
            103 => {
                send_admin(w, from, self.page, self.page_index + 1);
                return;
            }
            _ => {}
        }
        let slot = |base: i32, per: usize| -> Option<usize> {
            let i = usize::try_from(b - base).ok().filter(|&i| i < per)?;
            Some(i)
        };
        if let Some(i) = slot(1000, FLAGS_PER_PAGE) {
            let at = self.page_index * FLAGS_PER_PAGE + i;
            if self.page == Page::Flags
                && let Some(f) = self.flags.get(at)
            {
                set_flag(w, &f.key.clone(), !f.enabled, &name);
            }
        } else if let Some(i) = slot(2000, BLOCKS_PER_PAGE) {
            if let Some(r) = self.blocks.get(self.page_index * BLOCKS_PER_PAGE + i) {
                let (key, now) = (r.key.clone(), !r.active);
                match self.page {
                    Page::GumpBlocks => drop(set_gump_block_active(w, &key, now, &name)),
                    Page::ItemBlocks => drop(set_item_block_active(w, &key, now, &name)),
                    Page::SkillBlocks => drop(set_skill_block_active(
                        w,
                        key.parse().unwrap_or(-1),
                        now,
                        &name,
                    )),
                    Page::SpellBlocks => drop(set_spell_block_active(
                        w,
                        key.parse().unwrap_or(usize::MAX),
                        now,
                        &name,
                    )),
                    Page::Flags => {}
                }
            }
        } else if let Some(i) = slot(3000, BLOCKS_PER_PAGE)
            && let Some(r) = self.blocks.get(self.page_index * BLOCKS_PER_PAGE + i)
        {
            let key = r.key.clone();
            match self.page {
                Page::GumpBlocks => drop(unblock_gump(w, &key, &name)),
                Page::ItemBlocks => drop(remove_item_block(w, &key, &name)),
                Page::SkillBlocks => drop(unblock_skill(w, key.parse().unwrap_or(-1), &name)),
                Page::SpellBlocks => {
                    let full = mgr(w).and_then(|m| {
                        m.spell_blocks
                            .get(key.parse::<usize>().unwrap_or(usize::MAX))?
                            .as_ref()?
                            .resolved_type
                            .clone()
                    });
                    if let Some(f) = full {
                        unblock_spell(w, &f, &name);
                    }
                }
                Page::Flags => {}
            }
        } else if let Some(i) = slot(4000, BLOCKS_PER_PAGE)
            && self.page == Page::ItemBlocks
            && let Some(r) = self.blocks.get(self.page_index * BLOCKS_PER_PAGE + i)
        {
            // Edit item block: `new PropertiesGump(from, _displayedBlocks[index])`
            open_item_block_props(w, from, &r.key.clone());
        }
        send_admin(w, from, self.page, self.page_index);
    }
}

// ---- the Edit button: a PropertiesGump over an ItemBlockEntry ----

/// The item block each viewer's open props gump edits (its key): the gump's owner is the viewer, whose entry the
/// getters and setters below read.
#[derive(Default)]
struct EditingBlocks(std::collections::HashMap<MobileId, String>);

fn edited_block(w: &World, e: EntityId) -> Option<&ItemBlockEntry> {
    let EntityId::Mobile(m) = e else { return None };
    let key = w.resources.get::<EditingBlocks>()?.0.get(&m)?;
    mgr(w)?.item_blocks.get(key)
}

fn edited_block_mut(w: &mut World, e: EntityId) -> Option<&mut ItemBlockEntry> {
    let EntityId::Mobile(m) = e else { return None };
    let key = w.resources.get::<EditingBlocks>()?.0.get(&m)?.clone();
    mgr_mut(w).item_blocks.get_mut(&key)
}

macro_rules! block_prop {
    ($name:literal, $kind:expr, $get:expr, $set:expr) => {
        muo_content::props::PropDef {
            name: $name,
            kind: $kind,
            read: AccessLevel::Administrator,
            write: AccessLevel::Administrator,
            get: |w, e| edited_block(w, e).map_or(muo_content::props::Value::Null, $get),
            set: $set,
        }
    };
}

fn text_or_null(s: &Option<String>) -> muo_content::props::Value {
    s.clone().map_or(
        muo_content::props::Value::Null,
        muo_content::props::Value::Str,
    )
}

/// `FeatureFlagBlockEntry` and `ItemBlockEntry`'s `[CommandProperty]` members.
static ITEM_BLOCK_PROPS: &[muo_content::props::PropDef] = &[
    block_prop!(
        "ResolvedType",
        muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Type),
        |b| text_or_null(&b.base.resolved_type),
        Some(|w, e, v| {
            if let Some(b) = edited_block_mut(w, e) {
                b.base.resolved_type = v.string();
            }
            Ok(())
        })
    ),
    block_prop!(
        "Reason",
        muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Str),
        |b| text_or_null(&b.base.reason),
        Some(|w, e, v| {
            if let Some(b) = edited_block_mut(w, e) {
                b.base.reason = v.string();
            }
            Ok(())
        })
    ),
    block_prop!(
        "Active",
        muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Bool),
        |b| muo_content::props::Value::Bool(b.base.active),
        Some(|w, e, v| {
            if let Some(b) = edited_block_mut(w, e) {
                b.base.active = v.bool();
            }
            Ok(())
        })
    ),
    block_prop!(
        "CreatedAt",
        muo_content::props::Kind::DateTime,
        |b| muo_primitives::DateTime::parse_iso(&b.base.created_at).map_or(
            muo_content::props::Value::Null,
            muo_content::props::Value::DateTime
        ),
        None
    ),
    block_prop!(
        "CreatedBy",
        muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Str),
        |b| text_or_null(&b.base.created_by),
        None
    ),
    block_prop!(
        "BlockUse",
        muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Bool),
        |b| muo_content::props::Value::Bool(b.block_use),
        Some(|w, e, v| {
            if let Some(b) = edited_block_mut(w, e) {
                b.block_use = v.bool();
            }
            Ok(())
        })
    ),
    block_prop!(
        "BlockEquip",
        muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Bool),
        |b| muo_content::props::Value::Bool(b.block_equip),
        Some(|w, e, v| {
            if let Some(b) = edited_block_mut(w, e) {
                b.block_equip = v.bool();
            }
            Ok(())
        })
    ),
    block_prop!(
        "BlockContainerAccess",
        muo_content::props::Kind::Plain(muo_content::utilities::types::PropType::Bool),
        |b| muo_content::props::Value::Bool(b.block_container_access),
        Some(|w, e, v| {
            if let Some(b) = edited_block_mut(w, e) {
                b.block_container_access = v.bool();
            }
            Ok(())
        })
    ),
];

/// `from.SendGump(new PropertiesGump(from, block))`.
fn open_item_block_props(w: &mut World, from: MobileId, key: &str) {
    if mgr(w).is_none_or(|m| !m.item_blocks.contains_key(key)) {
        return;
    }
    w.resources
        .get_mut::<EditingBlocks>()
        .0
        .insert(from, key.to_owned());
    let obj = muo_gumps::m_z::props::registry::SubObj::object(
        EntityId::Mobile(from),
        ITEM_BLOCK_PROPS,
        "ItemBlockEntry",
    );
    let g = muo_gumps::m_z::props::properties::new(w, from, obj);
    muo_gumps::m_z::common::send(w, from, g);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gump_names_resolve_without_the_gump_being_open() {
        let w = World::new();
        assert_eq!(
            resolve_gump(&w, "addgump").as_deref(),
            Some("Server.Gumps.AddGump")
        );
        assert_eq!(
            resolve_gump(&w, "server.gumps.addgump").as_deref(),
            Some("Server.Gumps.AddGump")
        );
        assert_eq!(resolve_gump(&w, "NoSuchGump"), None);
        assert_eq!(resolve_gump(&w, "  "), None);
    }

    #[test]
    fn item_block_props_read_and_write_the_edited_entry() {
        let mut w = World::new();
        let from = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        let e = ItemBlockEntry {
            base: BlockBase {
                resolved_type: Some("Server.Items.Gold".into()),
                reason: Some("r".into()),
                ..Default::default()
            },
            block_use: true,
            ..Default::default()
        };
        mgr_mut(&mut w)
            .item_blocks
            .insert("Server.Items.Gold".into(), e);
        w.resources
            .get_mut::<EditingBlocks>()
            .0
            .insert(from, "Server.Items.Gold".into());
        let owner = EntityId::Mobile(from);
        let prop = |n: &str| ITEM_BLOCK_PROPS.iter().find(|p| p.name == n).unwrap();
        assert!((prop("BlockUse").get)(&mut w, owner).bool());
        assert_eq!(
            (prop("Reason").get)(&mut w, owner).string().as_deref(),
            Some("r")
        );
        (prop("BlockEquip").set.unwrap())(&mut w, owner, muo_content::props::Value::Bool(true))
            .unwrap();
        assert!(mgr(&w).unwrap().item_blocks["Server.Items.Gold"].block_equip);
        assert!(prop("CreatedBy").set.is_none());
    }

    #[test]
    fn skills_parse_by_name_or_number() {
        assert_eq!(parse_skill("magery"), Some(25));
        assert_eq!(parse_skill("25"), Some(25));
        assert_eq!(parse_skill("nope"), None);
    }

    #[test]
    fn flag_json_matches_the_distribution_file() {
        let text = r#"[{"Key":"pvp_combat","Description":"d","Enabled":true,"DefaultEnabled":true,"Category":"Combat","LastModified":"2026-10-25T23:33:04.2906569Z","LastModifiedBy":"System"}]"#;
        let v: Vec<FeatureFlag> = serde_json::from_str(text).unwrap();
        assert!(v[0].enabled && v[0].key == "pvp_combat");
        let back = serde_json::to_string(&v).unwrap();
        assert_eq!(back, text);
    }

    #[test]
    fn item_block_entry_round_trips_flattened() {
        let e = ItemBlockEntry {
            base: BlockBase {
                resolved_type: Some("Server.Items.Gold".into()),
                active: true,
                ..Default::default()
            },
            block_use: true,
            ..Default::default()
        };
        let s = serde_json::to_string(&e).unwrap();
        let back: ItemBlockEntry = serde_json::from_str(&s).unwrap();
        assert!(back.block_use && back.base.active && !back.block_equip);
    }

    #[test]
    fn pages_round_up() {
        assert_eq!(pages(0, 10), 1);
        assert_eq!(pages(11, 10), 2);
    }
}
