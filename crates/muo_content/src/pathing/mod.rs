//! Pathing: `MovementImpl` (the walk check), the A* search and `PathFollower`.

pub mod astar;
pub mod follower;
pub mod movement_impl;
pub mod recorder;
#[cfg(test)]
mod route_tests;
#[cfg(test)]
mod tests;

use muo_server::world::World;

pub use follower::PathFollower;

/// `MovementImpl.Configure`, `PathFollower.Configure` and `BitmapAStarAlgorithm.Configure`, plus the hooks the AI
/// and the player's movement call through.
pub fn configure(w: &mut World) {
    w.resources.get_mut::<muo_server::movement::Movement>().imp = Some(&movement_impl::IMPL);
    // `GetOrUpdateSetting`: the default is written to muo.json on first boot
    let enabled = w
        .config
        .get_or_update_setting("pathfinding.enable", true)
        .unwrap_or(true);
    w.resources.get_mut::<follower::PathConfig>().enabled = enabled;
    let max = w
        .config
        .get_or_update_setting("pathfinding.maxSearchNodes", 1000)
        .unwrap_or(1000);
    w.resources.get_mut::<astar::PathScratch>().max_search_nodes = max;

    let h = w.resources.get_mut::<crate::ai::PathHooks>();
    h.follow = Some(follower::follow_hook);
    h.force_repath = Some(follower::force_repath_hook);
    w.resources
        .get_mut::<crate::player_mobile::hooks::SystemHooks>()
        .ignore_movable_impassables = Some(|w, v| {
        w.resources
            .get_mut::<crate::ai::movement::MovementFlags>()
            .ignore_movable_impassables = v;
    });
}

/// `EntityFinalizationTracker.Configure`: `[gc` and `[TrackLeaks`. Rust has no collector to force, so `[gc` is accepted
/// and does nothing; leak tracking is not built in, so it answers `[TrackLeaks` with the "not enabled" message.
fn configure_finalization_tracker(w: &mut World) {
    // `PathCacheCommands.Configure` writes the cache cap to the config on first boot (there is no cache yet).
    let _ = w
        .config
        .get_or_update_setting("pathfinding.maxResidentChunks", 8192);
    use muo_server::commands::{CommandEventArgs, CommandSystem};
    use muo_server::mobile::AccessLevel;
    CommandSystem::register(
        w,
        "gc",
        AccessLevel::Administrator,
        &[],
        std::rc::Rc::new(|_: &mut World, _: &mut CommandEventArgs| {}),
    );
    CommandSystem::register(
        w,
        "TrackLeaks",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(|w: &mut World, e: &mut CommandEventArgs| {
            muo_server::mobile::part2::messages::send_message(
                w,
                e.mobile,
                "Entity leak tracking is not enabled in this build. Rebuild with TRACK_LEAKS defined.",
            );
        }),
    );
}

/// `PathCacheCommands.ConfigurePrompts`: asks once whether to pre-bake the cache and keeps the answer in
/// `pathfinding.prebakeMaps`; skipped when already answered or when input is redirected (there is no cache yet).
fn configure_prompts(w: &mut World) {
    use std::io::IsTerminal;
    const KEY: &str = "pathfinding.prebakeMaps";
    if w.config.get_setting_opt::<String>(KEY).is_some() || !std::io::stdin().is_terminal() {
        return;
    }
    println!();
    println!("Pre-bake the pathfinding cache for your selected maps now?");
    println!("  Bakes each map's .swb so there is zero first-pathfind-after-boot latency.");
    println!("  Takes several minutes and ~tens of MB of disk per facet. You can also do");
    println!("  this later at runtime with [PathBake.");
    print!("Pre-bake now? [y/N] ");
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
    let prebake = line.trim().to_ascii_lowercase().starts_with('y');
    let _ = w.config.set_setting(KEY, &prebake);
}

pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase};
    types.register_phase(
        Phase::ConfigurePrompts,
        "PathCacheCommands.ConfigurePrompts",
        DEFAULT_CALL_PRIORITY,
        configure_prompts,
    );
    types.register_phase(
        Phase::Configure,
        "MovementImpl.Configure",
        DEFAULT_CALL_PRIORITY,
        configure,
    );
    types.register_phase(
        Phase::Configure,
        "Server.Engines.Pathing.PathfindRecorder.Configure",
        DEFAULT_CALL_PRIORITY,
        recorder::configure,
    );
    types.register_phase(
        Phase::Configure,
        "Server.EntityFinalizationTracker.Configure",
        DEFAULT_CALL_PRIORITY,
        configure_finalization_tracker,
    );
}

#[cfg(test)]
mod tracker_tests {
    #[test]
    fn gc_and_trackleaks_are_registered_with_their_levels() {
        use muo_server::commands::CommandSystem;
        use muo_server::mobile::AccessLevel;
        let mut w = muo_server::world::World::new();
        super::configure_finalization_tracker(&mut w);
        let cs = w.resources.get::<CommandSystem>().unwrap();
        let level = |n: &str| {
            cs.entries
                .values()
                .find(|e| e.command == n)
                .unwrap()
                .access_level
        };
        assert_eq!(level("gc"), AccessLevel::Administrator);
        assert_eq!(level("TrackLeaks"), AccessLevel::Developer);
    }
}
