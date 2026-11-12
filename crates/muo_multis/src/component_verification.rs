//! `ComponentVerification`: which house pieces and stair multis a player may place, read from
//! `housing.bin` (inside MultiCollection.uop) or the client's component sheets.

use std::io::{Read, Seek, SeekFrom};

use muo_data::DataDirectories;
use muo_data::expansion::HousingFlags;
use muo_data::tile_data::TileData;
use muo_server::World;

const NOT_A_COMPONENT: i32 = -1;
const NO_FEATURE_REQUIRED: i32 = 0;
/// housing.bin tags pre-AOS pieces with client feature bits `HousingFlags` does not model: keep only the tier bits.
const HOUSING_TIER_MASK: u32 = HousingFlags::HOUSING_EJ.bits();

/// The two tables, filled on first use (`EnsureLoaded`).
#[derive(Default)]
pub struct Tables {
    loaded: bool,
    items: Vec<i32>,
    multis: Vec<i32>,
}

fn check_validity(w: &World, val: i32) -> bool {
    let flags = w
        .core_expansion_info()
        .map_or(0, |e| e.housing_flags.bits());
    val != NOT_A_COMPONENT && (val == NO_FEATURE_REQUIRED || (flags & val as u32) != 0)
}

fn ensure_loaded(w: &mut World) {
    if w.resources.get::<Tables>().is_some_and(|t| t.loaded) {
        return;
    }
    // Set before loading so a bad file is not re-read on each placement attempt.
    let mut t = Tables {
        loaded: true,
        items: vec![NOT_A_COMPONENT; TileData::MAX_ITEM_VALUE],
        multis: vec![NOT_A_COMPONENT; 0x4000],
    };
    let from_bin = (|| {
        let path = w.maps.multi_data.housing_uop_path.clone()?;
        let entry = w.maps.multi_data.housing_entry?;
        let data = read_uop_entry(&path, &entry)?;
        load_from_housing_bin(&mut t, &data)
    })();
    if from_bin.is_none() {
        // A half-read housing.bin must not leave pieces registered
        t.items.fill(NOT_A_COMPONENT);
        t.multis.fill(NOT_A_COMPONENT);
        load_from_txt_files(w, &mut t);
    }
    *w.resources.get_mut::<Tables>() = t;
}

/// `IsItemValid(itemID)`.
pub fn is_item_valid(w: &mut World, item_id: i32) -> bool {
    ensure_loaded(w);
    let v = w.resources.get::<Tables>().and_then(|t| {
        (item_id > 0)
            .then(|| t.items.get(item_id as usize).copied())
            .flatten()
    });
    v.is_some_and(|v| check_validity(w, v))
}

/// `IsMultiValid(multiID)`.
pub fn is_multi_valid(w: &mut World, multi_id: i32) -> bool {
    ensure_loaded(w);
    let v = w.resources.get::<Tables>().and_then(|t| {
        (multi_id > 0)
            .then(|| t.multis.get(multi_id as usize).copied())
            .flatten()
    });
    v.is_some_and(|v| check_validity(w, v))
}

fn read_uop_entry(
    path: &std::path::Path,
    entry: &muo_data::client::uop::UopEntry,
) -> Option<Vec<u8>> {
    let mut f = std::fs::File::open(path).ok()?;
    f.seek(SeekFrom::Start(u64::try_from(entry.offset).ok()?))
        .ok()?;
    if entry.compressed {
        let mut packed = vec![0u8; entry.compressed_size as usize];
        f.read_exact(&mut packed).ok()?;
        let mut out = vec![0u8; entry.size as usize];
        let n = muo_io::compression::STANDARD
            .unpack(&mut out, &packed)
            .ok()?;
        return (n == entry.size as usize).then_some(out);
    }
    let mut data = vec![0u8; entry.size as usize];
    f.read_exact(&mut data).ok()?;
    Some(data)
}

struct Cur<'a>(&'a [u8]);
impl Cur<'_> {
    fn u32(&mut self) -> Option<u32> {
        let (h, t) = self.0.split_first_chunk::<4>()?;
        self.0 = t;
        Some(u32::from_le_bytes(*h))
    }
}

/// `LoadFromHousingBin`; None when the data ends early.
fn load_from_housing_bin(t: &mut Tables, data: &[u8]) -> Option<()> {
    let mut r = Cur(data);
    let file_types = r.u32()?;
    for _ in 0..file_types {
        let file_type = r.u32()?;
        let entries = r.u32()?;
        let is_walls = file_type == 5;
        let is_stairs = file_type == 1;
        for _ in 0..entries {
            r.u32()?; // category_id
            r.u32()?; // subcategory_id
            let mask = (r.u32()? & HOUSING_TIER_MASK) as i32;
            r.u32()?; // cliloc_id
            for _ in 0..r.u32()? {
                r.u32()?; // direction
                let id = r.u32()? as i32;
                if id > 0 && (id as usize) < t.items.len() {
                    t.items[id as usize] = mask;
                }
            }
            if is_walls {
                r.u32()?;
            }
            for _ in 0..r.u32()? {
                r.u32()?; // direction
                let id = r.u32()? as i32;
                let table = if is_stairs {
                    &mut t.multis
                } else {
                    &mut t.items
                };
                if id > 0 && (id as usize) < table.len() {
                    table[id as usize] = mask;
                }
            }
            if !is_walls {
                r.u32()?;
            }
        }
    }
    Some(())
}

fn load_from_txt_files(w: &World, t: &mut Tables) {
    let sheets: [(&str, &[&str]); 7] = [
        (
            "walls.txt",
            &[
                "South1",
                "South2",
                "South3",
                "Corner",
                "East1",
                "East2",
                "East3",
                "Post",
                "WindowS",
                "AltWindowS",
                "WindowE",
                "AltWindowE",
                "SecondAltWindowS",
                "SecondAltWindowE",
            ],
        ),
        (
            "teleprts.txt",
            &[
                "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "F13",
                "F14", "F15", "F16",
            ],
        ),
        (
            "stairs.txt",
            &[
                "Block", "North", "East", "South", "West", "Squared1", "Squared2", "Rounded1",
                "Rounded2",
            ],
        ),
        (
            "roof.txt",
            &[
                "North",
                "East",
                "South",
                "West",
                "NSCrosspiece",
                "EWCrosspiece",
                "NDent",
                "EDent",
                "SDent",
                "WDent",
                "NTPiece",
                "ETPiece",
                "STPiece",
                "WTPiece",
                "XPiece",
                "Extra Piece",
            ],
        ),
        (
            "floors.txt",
            &[
                "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "F13",
                "F14", "F15", "F16",
            ],
        ),
        (
            "misc.txt",
            &[
                "Piece1", "Piece2", "Piece3", "Piece4", "Piece5", "Piece6", "Piece7", "Piece8",
            ],
        ),
        (
            "doors.txt",
            &[
                "Piece1", "Piece2", "Piece3", "Piece4", "Piece5", "Piece6", "Piece7", "Piece8",
            ],
        ),
    ];
    for (file, cols) in sheets {
        load_sheet(w, &mut t.items, file, cols);
    }
    load_sheet(
        w,
        &mut t.multis,
        "stairs.txt",
        &["MultiNorth", "MultiEast", "MultiSouth", "MultiWest"],
    );
}

fn resolve_txt(w: &World, file: &str) -> Option<std::path::PathBuf> {
    if let Some(p) = w
        .resources
        .get::<DataDirectories>()
        .and_then(|d| d.find(file))
    {
        return Some(p);
    }
    let bundled = std::path::Path::new("Data").join("Components").join(file);
    bundled.is_file().then_some(bundled)
}

/// `Spreadsheet`: two header rows (types, names) of tab separated fields, then records. A blank line is skipped.
fn read_sheet(path: &std::path::Path) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut lines = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split('\t').map(str::to_string).collect::<Vec<_>>());
    let types = lines.next()?;
    let names = lines.next()?;
    let mut cols = vec![String::new(); types.len()];
    for (i, n) in names.into_iter().enumerate().take(types.len()) {
        cols[i] = n;
    }
    Some((cols, lines.collect()))
}

/// `LoadSpreadsheet`.
fn load_sheet(w: &World, table: &mut [i32], file: &str, tile_columns: &[&str]) {
    let Some(path) = resolve_txt(w, file) else {
        return;
    };
    let Some((names, records)) = read_sheet(&path) else {
        tracing::error!("Could not read house components from {}", path.display());
        return;
    };
    let cid = |n: &str| names.iter().position(|x| x == n);
    let Some(feature) = cid("FeatureMask") else {
        tracing::error!(
            "House component sheet {} has no FeatureMask column",
            path.display()
        );
        return;
    };
    let tiles: Vec<Option<usize>> = tile_columns.iter().map(|c| cid(c)).collect();
    if tiles.iter().all(Option::is_none) {
        tracing::error!(
            "House component sheet {} has none of its expected tile columns",
            path.display()
        );
        return;
    }
    // `Utility.ToInt32`: decimal or 0x hex, 0 when it does not parse; a missing field is empty
    let int = |rec: &Vec<String>, i: usize| rec.get(i).map_or(0, |s| parse_int(s));
    for rec in &records {
        let fid = int(rec, feature);
        for c in tiles.iter().flatten() {
            let id = int(rec, *c);
            if id > 0 && (id as usize) < table.len() {
                table[id as usize] = fid;
            }
        }
    }
}

fn parse_int(s: &str) -> i32 {
    let s = s.trim();
    match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        Some(h) => i32::from_str_radix(h, 16).unwrap_or(0),
        None => s.parse().unwrap_or(0),
    }
}

/// Spreadsheet tests against `read_sheet` (a sheet that cannot be read is `None`, for `LoadSpreadsheet` to log).
#[cfg(test)]
mod spreadsheet_tests {
    use super::*;

    // misc.txt's shape: 13 columns, the last being the Comment the client never reads.
    const TYPES: &str = "int\tint\tint\tint\tint\tint\tint\tint\tint\tint\tint\tint\tstring";
    const NAMES: &str = "Category\tStyle\tTID\tPiece1\tPiece2\tPiece3\tPiece4\tPiece5\tPiece6\tPiece7\tPiece8\tFeatureMask\tComment";

    struct Sheet(std::path::PathBuf);

    impl Sheet {
        fn new(name: &str, lines: &[&str]) -> Sheet {
            let path =
                std::env::temp_dir().join(format!("muo-sheet-{}-{name}.txt", std::process::id()));
            std::fs::write(&path, lines.join("\n") + "\n").unwrap();
            Sheet(path)
        }

        fn read(&self) -> Option<(Vec<String>, Vec<Vec<String>>)> {
            read_sheet(&self.0)
        }
    }

    impl Drop for Sheet {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    /// `GetColumnID`: -1 when no column has the name.
    fn column_id(cols: &[String], name: &str) -> i32 {
        cols.iter().position(|c| c == name).map_or(-1, |i| i as i32)
    }

    /// `record.GetInt32(id)`: a missing field is empty, which `Utility.ToInt32` reads as 0.
    fn get_int32(record: &[String], id: i32) -> i32 {
        record.get(id as usize).map_or(0, |s| parse_int(s))
    }

    #[test]
    fn row_missing_trailing_comment_is_still_read() {
        // An empty Comment written without a trailing tab leaves the row one field short.
        let s = Sheet::new(
            "missing-comment",
            &[
                TYPES,
                NAMES,
                "0\t0\t1060056\t44\t0\t41\t40\t42\t0\t43\t29\t8",
            ],
        );
        let (cols, records) = s.read().unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(get_int32(&records[0], column_id(&cols, "Piece1")), 44);
        assert_eq!(get_int32(&records[0], column_id(&cols, "Piece8")), 29);
        assert_eq!(get_int32(&records[0], column_id(&cols, "FeatureMask")), 8);
    }

    #[test]
    fn short_row_does_not_drop_later_rows() {
        let s = Sheet::new(
            "short-row",
            &[
                TYPES,
                NAMES,
                "0\t0\t1060056\t44\t0\t41\t40\t42\t0\t43\t29\t0",
                "0\t1\t1060057\t45\t0\t0\t0\t0\t0\t0\t0\t0\tFieldstone Arches",
            ],
        );
        let (cols, records) = s.read().unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(get_int32(&records[0], column_id(&cols, "Piece1")), 44);
        assert_eq!(get_int32(&records[1], column_id(&cols, "Piece1")), 45);
    }

    #[test]
    fn tab_only_separator_lines_are_not_mistaken_for_header_rows() {
        // The retail client's doors.txt separates its header rows this way.
        let separator = "\t".repeat(12);
        let s = Sheet::new(
            "separators",
            &[
                TYPES,
                &separator,
                NAMES,
                &separator,
                "0\t0\t1060056\t44\t0\t41\t40\t42\t0\t43\t29\t0\tFieldstone Archways",
            ],
        );
        let (cols, records) = s.read().unwrap();

        assert_eq!(column_id(&cols, "Piece1"), 3);
        assert_eq!(column_id(&cols, "FeatureMask"), 11);

        assert_eq!(records.len(), 1);
        assert_eq!(get_int32(&records[0], column_id(&cols, "Piece1")), 44);
    }

    #[test]
    fn missing_header_rows_is_an_error_instead_of_a_null_reference() {
        let s = Sheet::new("no-headers", &[TYPES]);
        assert!(s.read().is_none());
    }

    #[test]
    fn header_with_fewer_names_than_types_is_tolerated() {
        let s = Sheet::new(
            "short-names",
            &[
                TYPES,
                "Category\tStyle\tTID\tPiece1",
                "0\t0\t1060056\t44\t0\t41\t40\t42\t0\t43\t29\t0\tFieldstone Archways",
            ],
        );
        let (cols, records) = s.read().unwrap();

        assert_eq!(get_int32(&records[0], column_id(&cols, "Piece1")), 44);
        assert_eq!(column_id(&cols, "FeatureMask"), -1);
    }
}
