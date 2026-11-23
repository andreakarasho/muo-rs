//! Map setup: `Configure` and `RegisterMap`, over the definitions muo_data reads.

use std::path::Path;

use muo_data::map_definitions::{MapDefinition, load_map_definitions};

use super::{Map, MapId, SECTOR_SIZE};
use crate::World;

/// `MapLoader.RegisterMap`: the map goes in `Maps[def.index]` and `AllMaps`; width and height are at least one sector.
/// Its default region is made with it.
pub fn register_map(w: &mut World, def: &MapDefinition) -> Result<MapId, String> {
    let index = u8::try_from(def.index)
        .map_err(|_| format!("Index was outside the bounds of the array: {}", def.index))?;
    let map = Map::new(
        def.id,
        i32::from(index),
        def.file_index,
        def.width.max(SECTOR_SIZE),
        def.height.max(SECTOR_SIZE),
        def.season,
        &def.name,
        def.rules,
    );
    let id = w.maps.add(map);
    crate::region::ensure_default_region(w, id);
    Ok(id)
}

/// `MapLoader.Configure`: reads `Data/map-definitions.json` under `base_dir`. Definitions that fail are
/// skipped and reported; the error is the file itself failing to load.
pub fn configure(w: &mut World, base_dir: &Path) -> Result<usize, String> {
    tracing::info!("Loading Map Definitions");
    let started = std::time::Instant::now();

    let defs = load_map_definitions(base_dir).map_err(|e| e.to_string())?;

    let mut failures = Vec::new();
    let mut count = 0;
    // an entry with a `count` is that many maps, on the indexes from its own, each named after its index
    fn copies(def: &MapDefinition) -> impl Iterator<Item = MapDefinition> + '_ {
        let n = def.count.max(1);
        (0..n).map(move |i| MapDefinition {
            index: def.index + i,
            name: if n > 1 {
                format!("{}{}", def.name, def.index + i)
            } else {
                def.name.clone()
            },
            count: 1,
            ..def.clone()
        })
    }
    for def in defs.iter().flat_map(copies) {
        let def = &def;
        match register_map(w, def) {
            Ok(_) => count += 1,
            Err(e) => {
                tracing::debug!(
                    "Failed to load map definition {} ({}): {e}",
                    def.name,
                    def.id
                );
                failures.push(format!(
                    "\tInvalid map definition {} ({})",
                    def.name, def.id
                ));
            }
        }
    }

    let secs = started.elapsed().as_secs_f64();
    if failures.is_empty() {
        tracing::info!(
            "Loading map definitions done ({count} maps, 0 failures) ({secs:.2} seconds)"
        );
    } else {
        tracing::warn!(
            "Loading map definitions failed ({count} maps, {} failures) ({secs:.2} seconds)\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
    Ok(count)
}
