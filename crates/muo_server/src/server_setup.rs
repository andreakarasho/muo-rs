//! The `Configure`/`Initialize` functions of the core itself (the content crates register
//! theirs), plus `TileMatrixLoader.LoadTileMatrix` and the post-`Configure` data hookup of `Core.Setup`.

use muo_data::client::multi_data::MultiData;
use muo_data::client::uo_client::UoClient;
use muo_data::tile_data::TileData;
use muo_data::tile_matrix::TileMatrixOptions;

use crate::item::decay::DecayConfig;
use crate::map::TileSource;
use crate::type_registry::{Phase, TypeRegistry};
use crate::world::World;

/// Registers the core's own phase functions. Priorities keep the dependencies: client version
/// first, then the tile and multi data, then the map definitions (`[CallPriority(2)]` there).
pub fn register(types: &mut TypeRegistry) {
    types.register_phase(
        Phase::Configure,
        "UOClient.Configure",
        0,
        uo_client_configure,
    );
    types.register_phase(
        Phase::Configure,
        "TileData.Configure",
        1,
        tile_data_configure,
    );
    types.register_phase(
        Phase::Configure,
        "MultiData.Configure",
        1,
        multi_data_configure,
    );
    types.register_phase(
        Phase::Configure,
        "MapLoader.Configure",
        2,
        map_loader_configure,
    );
    types.register_phase(
        Phase::Configure,
        "BaseMulti.Configure",
        2,
        crate::item::multi::register,
    );
    types.register_phase_default(
        Phase::Configure,
        "DecayScheduler.Configure",
        decay_configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "MovementThrottle.Configure",
        movement_throttle_configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "DumpNetStates.Configure",
        crate::net_glue::dump_net_states_configure,
    );
    types.register_phase_default(Phase::Configure, "Mobile.Configure", mobile_configure);
    types.register_phase_default(
        Phase::Configure,
        "Movement.Configure",
        crate::movement::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "Timer.ConfigureTimerPool",
        timer_pool_configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "ItemBounds.Configure",
        crate::item::bounds::configure,
    );
}

/// `Mobile.Configure`: `spellCasting.disableCastParalyze` (read only).
fn mobile_configure(w: &mut World) {
    let v = w
        .config
        .get_setting("spellCasting.disableCastParalyze", true);
    w.resources
        .get_mut::<crate::mobile::part2::statics::Part2Statics>()
        .disable_cast_paralyze = v;
}

/// `Timer.ConfigureTimerPool`: the pool sizes are written back on first boot; this timer has no pool to size.
fn timer_pool_configure(w: &mut World) {
    let capacity = w
        .config
        .get_or_update_setting("timer.initialPoolCapacity", 1024_i32)
        .unwrap_or(1024);
    let _ = w
        .config
        .get_or_update_setting("timer.maxPoolCapacity", capacity.wrapping_mul(16));
}

fn uo_client_configure(w: &mut World) {
    let mut client = w.resources.get::<UoClient>().cloned().unwrap_or_default();
    if let Err(e) = client.load(&w.config) {
        tracing::warn!("Could not load the client version: {e}");
    }
    client.configure();
    w.resources.insert(client);
}

fn tile_data_configure(w: &mut World) {
    match TileData::load(w.config.data_directories()) {
        Ok(t) => w.maps.tile_data = t,
        Err(e) => tracing::error!("Failed to load tiledata.mul: {e}"),
    }
}

fn multi_data_configure(w: &mut World) {
    let version = w
        .resources
        .get::<UoClient>()
        .and_then(|c| c.server_client_version.clone());
    match MultiData::load(w.config.data_directories(), &w.config, version.as_ref()) {
        Ok(m) => w.maps.multi_data = m,
        Err(e) => tracing::error!("Failed to load the multi data: {e}"),
    }
}

fn map_loader_configure(w: &mut World) {
    let base = w.core.base_directory.clone();
    if let Err(e) = crate::map::loader::configure(w, &base) {
        tracing::error!("Failed to load the map definitions: {e}");
    }
}

/// `MovementThrottle.Configure`: the `movementThrottle.*` settings, each written back with its default on first boot.
fn movement_throttle_configure(w: &mut World) {
    let d = w.movement_throttle.config.clone();
    let c = &mut w.config;
    // a failed save of the settings file is not fatal: the value in memory is still the configured one
    let mut int =
        |key: &str, default: i32| c.get_or_update_setting(key, default).unwrap_or(default);
    let max_credit = int("movementThrottle.maxCredit", d.max_credit);
    let max_rtt_bonus = int("movementThrottle.maxRttBonus", d.max_rtt_bonus);
    let hard_queue_limit = int("movementThrottle.hardQueueLimit", d.hard_queue_limit as i32);
    let movement_history_size = int(
        "movementThrottle.movementHistorySize",
        d.movement_history_size as i32,
    );
    let min_samples_for_rate = int("movementThrottle.minSamplesForRate", d.min_samples_for_rate);
    let max_chain_gap = int("movementThrottle.maxChainGap", d.max_chain_gap);
    let cooldown = int(
        "movementThrottle.speedHackNotificationCooldown",
        d.speed_hack_notification_cooldown as i32,
    );
    let mut float = |key: &str, default: f32| {
        c.get_or_update_setting(key, f64::from(default))
            .unwrap_or(f64::from(default)) as f32
    };
    let suspicious = float(
        "movementThrottle.suspiciousRateThreshold",
        d.suspicious_rate_threshold,
    );
    let definite = float(
        "movementThrottle.definiteRateThreshold",
        d.definite_rate_threshold,
    );
    let config = &mut w.movement_throttle.config;
    config.max_credit = max_credit;
    config.max_rtt_bonus = max_rtt_bonus;
    config.hard_queue_limit = hard_queue_limit.max(0) as usize;
    config.movement_history_size = movement_history_size.max(0) as usize;
    config.min_samples_for_rate = min_samples_for_rate;
    config.max_chain_gap = max_chain_gap;
    config.speed_hack_notification_cooldown = i64::from(cooldown);
    config.suspicious_rate_threshold = suspicious;
    config.definite_rate_threshold = definite;
}

fn decay_configure(w: &mut World) {
    let d = DecayConfig::default();
    let config = DecayConfig {
        max_items_per_tick: w
            .config
            .get_setting("decay.maxItemsPerTick", d.max_items_per_tick),
        tick_interval: w.config.get_setting("decay.tickInterval", d.tick_interval),
        bucket_interval: w
            .config
            .get_setting("decay.bucketInterval", d.bucket_interval),
        jitter_max_milliseconds: w
            .config
            .get_setting("decay.jitterMaxMs", d.jitter_max_milliseconds),
    };
    crate::item::decay::configure(w, config);
}

/// `TileMatrixLoader.LoadTileMatrix`: the maps open their files lazily from this source.
pub fn load_tile_matrix(w: &mut World) {
    tracing::info!("Loading maps");
    let version = w
        .resources
        .get::<UoClient>()
        .and_then(|c| c.server_client_version.clone());
    let options = TileMatrixOptions::from_config(&w.config, version.as_ref());
    w.maps.tile_source = Some(TileSource {
        dirs: w.config.data_directories().clone(),
        options,
    });
    tracing::info!("Loading maps done");
}

/// `Body`'s static constructor: reads `Data/bodyTable.cfg` (the table is a world resource here).
pub fn load_body_table(w: &mut World) {
    let path = w.core.base_directory.join("Data/bodyTable.cfg");
    let Ok(text) = std::fs::read_to_string(&path) else {
        tracing::error!("Data/bodyTable.cfg does not exist.");
        return;
    };
    let (table, invalid) = muo_primitives::BodyTable::parse(&text);
    for entry in invalid {
        tracing::warn!("Invalid bodyTable entry: {entry}");
    }
    w.resources.insert(table);
}
