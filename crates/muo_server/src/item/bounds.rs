//! The graphic size and bounding box of every item art, from
//! `Data/Items/ItemBounds.bin` (generated from art.mul/artidx.mul or artLegacyMUL.uop when missing).
//!
//! The table is loaded or generated on first use by [`ensure_loaded`] (call
//! it before reading from a `&mut World` context; [`bounds`] on an unloaded table gives an empty rectangle).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use muo_data::client::art_data::ArtData;
use muo_primitives::Rectangle2D;

use crate::commands::{CommandEventArgs, CommandSystem};
use crate::mobile::AccessLevel;
use crate::world::World;

const BOUNDS_FILE_NAME: &str = "ItemBounds.bin";

/// The statics of `ItemBounds`.
#[derive(Default)]
pub struct ItemBounds {
    /// `Sizes`: `(width, height)` per item id.
    pub sizes: Vec<(u16, u16)>,
    /// `Bounds`.
    pub bounds: Vec<Rectangle2D>,
    loaded: bool,
    is_generating: bool,
}

fn folder(w: &World) -> PathBuf {
    w.core.base_directory.join("Data").join("Items")
}

/// `Configure()`: registers `GenBounds`.
pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "GenBounds",
        AccessLevel::Developer,
        &[],
        Rc::new(gen_bounds_on_command),
    );
}

/// `[Usage("GenBounds")]`: generates the ItemBounds.bin file from art.mul/artidx.mul or artLegacyMUL.uop to
/// determine graphic sizes and boundaries.
// shortcut: this runs inline on the
// game loop (it is a rare developer command), upgrade with a worker when the art archive loads slowly.
fn gen_bounds_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if w.resources.get_mut::<ItemBounds>().is_generating {
        crate::commands::send_message(w, from, "Bounds file is already being generated.");
        return;
    }
    w.resources.get_mut::<ItemBounds>().is_generating = true;
    tracing::info!("Generating {BOUNDS_FILE_NAME}...");
    crate::commands::send_message(w, from, "Generating bounds file...");

    let result = match w.resources.get::<muo_data::DataDirectories>() {
        Some(dirs) => ArtData::open(dirs)
            .map_err(|e| e.to_string())
            .and_then(|mut art| {
                generate_bounds_file(&mut art, &folder(w)).map_err(|e| e.to_string())
            }),
        None => Err("no data directories".to_string()),
    };

    match result {
        Ok((sizes, bounds)) => {
            let s = w.resources.get_mut::<ItemBounds>();
            s.sizes = sizes;
            s.bounds = bounds;
            s.loaded = true;
            crate::commands::send_message(
                w,
                from,
                &format!("Bounds file saved to Data/Items/{BOUNDS_FILE_NAME}."),
            );
            tracing::info!("Generated {BOUNDS_FILE_NAME} successfully.");
        }
        Err(msg) => {
            crate::commands::send_message(w, from, "Failed to generate bounds file:");
            crate::commands::send_message(w, from, &msg);
            tracing::error!("Failed to generate {BOUNDS_FILE_NAME}: {msg}");
        }
    }
    w.resources.get_mut::<ItemBounds>().is_generating = false;
}

/// The static constructor: load `ItemBounds.bin`, or generate it when missing.
pub fn ensure_loaded(w: &mut World) {
    if w.resources.get_mut::<ItemBounds>().loaded {
        return;
    }
    w.resources.get_mut::<ItemBounds>().loaded = true;
    let table_len = muo_data::tile_data::ITEM_TABLE_LENGTH;
    let dir = folder(w);
    let path = dir.join(BOUNDS_FILE_NAME);

    if !path.exists() {
        tracing::info!("Generating {BOUNDS_FILE_NAME}...");
        let generated = match w.resources.get::<muo_data::DataDirectories>() {
            Some(dirs) => ArtData::open(dirs)
                .map_err(|e| e.to_string())
                .and_then(|mut art| {
                    generate_bounds_file(&mut art, &dir).map_err(|e| e.to_string())
                }),
            None => Err("no data directories".to_string()),
        };
        match generated {
            Ok((sizes, bounds)) => {
                let s = w.resources.get_mut::<ItemBounds>();
                s.sizes = sizes;
                s.bounds = bounds;
                tracing::info!("Generated {BOUNDS_FILE_NAME} successfully.");
            }
            Err(e) => tracing::error!("Failed to generate {BOUNDS_FILE_NAME}: {e}"),
        }
        return;
    }

    match read_table(&path, table_len) {
        Ok((sizes, bounds)) => {
            let s = w.resources.get_mut::<ItemBounds>();
            s.sizes = sizes;
            s.bounds = bounds;
        }
        Err(e) => tracing::error!("Failed to read {BOUNDS_FILE_NAME}: {e}"),
    }
}

/// `GenerateBoundsFile`: writes the file and returns the tables.
#[allow(clippy::type_complexity)] // the callback type, spelled out
pub fn generate_bounds_file(
    art: &mut ArtData,
    folder: &Path,
) -> Result<(Vec<(u16, u16)>, Vec<Rectangle2D>), Box<dyn std::error::Error>> {
    if !art.is_initialized() {
        return Err("Unable to load art.mul/artidx.mul or artLegacyMUL.uop".into());
    }
    let n = muo_data::tile_data::ITEM_TABLE_LENGTH;
    let mut sizes = vec![(0u16, 0u16); n];
    let mut bounds = vec![Rectangle2D::EMPTY; n];

    fs::create_dir_all(folder)?;
    let mut out = std::io::BufWriter::new(fs::File::create(folder.join(BOUNDS_FILE_NAME))?);
    for i in 0..n {
        let (w, h, b) = art.get_static_bounds(i as i32)?;
        out.write_all(&w.to_le_bytes())?;
        out.write_all(&h.to_le_bytes())?;
        out.write_all(&(b.x() as i16).to_le_bytes())?;
        out.write_all(&(b.y() as i16).to_le_bytes())?;
        out.write_all(&((b.x() + b.width() + 1) as i16).to_le_bytes())?;
        out.write_all(&((b.y() + b.height() + 1) as i16).to_le_bytes())?;
        bounds[i].set(b.x(), b.y(), b.width(), b.height());
        sizes[i] = (w, h);
    }
    out.flush()?;
    Ok((sizes, bounds))
}

/// `GenerateTable`: reads the file. The entry count is `file length / 8` (an entry is 12 bytes), so a
/// well-formed file of exactly the table's length reads fully and a short one fails at its end.
#[allow(clippy::type_complexity)] // the callback type, spelled out
pub fn read_table(
    path: &Path,
    table_len: usize,
) -> Result<(Vec<(u16, u16)>, Vec<Rectangle2D>), Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let mut bounds = vec![Rectangle2D::EMPTY; table_len];
    let mut sizes = vec![(0u16, 0u16); table_len];
    let count = table_len.min(data.len() / 8);
    let mut pos = 0usize;
    let mut take = |n: usize| -> Result<&[u8], Box<dyn std::error::Error>> {
        let s = data
            .get(pos..pos + n)
            .ok_or("Unable to read beyond the end of the stream.")?;
        pos += n;
        Ok(s)
    };
    for i in 0..count {
        let w = u16::from_le_bytes(take(2)?.try_into()?);
        let h = u16::from_le_bytes(take(2)?.try_into()?);
        let x_min = i16::from_le_bytes(take(2)?.try_into()?) as i32;
        let y_min = i16::from_le_bytes(take(2)?.try_into()?) as i32;
        let x_max = i16::from_le_bytes(take(2)?.try_into()?) as i32;
        let y_max = i16::from_le_bytes(take(2)?.try_into()?) as i32;
        sizes[i] = (w, h);
        bounds[i].set(x_min, y_min, x_max - x_min, y_max - y_min);
    }
    Ok((sizes, bounds))
}

/// `ItemBounds.Bounds[index]`; empty when the table is not loaded.
pub fn bounds(w: &World, index: usize) -> Rectangle2D {
    w.resources
        .get::<ItemBounds>()
        .and_then(|b| b.bounds.get(index).copied())
        .unwrap_or(Rectangle2D::EMPTY)
}

/// `ItemBounds.Sizes[index]`.
pub fn size(w: &World, index: usize) -> (u16, u16) {
    w.resources
        .get::<ItemBounds>()
        .and_then(|b| b.sizes.get(index).copied())
        .unwrap_or((0, 0))
}
