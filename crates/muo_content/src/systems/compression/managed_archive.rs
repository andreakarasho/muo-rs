//! `ManagedArchive`: `.tar.zst` creation and `.tar`/`.tar.gz`/`.tar.zst` extraction.
//!
//! Pure Rust: `tar` for the container, `ruzstd` for zstd, `flate2` (miniz_oxide) for gzip.
//! shortcut: ruzstd's encoder only implements its `Fastest` level (about zstd level 1), so `compression_level` is
//! accepted and ignored: archives are valid `.tar.zst` any zstd reads, only larger than level 3 would make them.
//! Upgrade when ruzstd implements `Default`.
//! shortcut: the tar is written to `<dest>.tmp.tar` and then compressed to `<dest>.tmp` (two passes on disk), as the
//! ruzstd encoder pulls from a `Read` instead of exposing a `Write` stream.
//! shortcut: the `tar` crate adds PAX headers only for names longer than 100 bytes (other writers may write a
//! PAX block per entry); any tar reader accepts both.

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Component, Path, PathBuf};

/// Files under a directory (`EnumerateFiles(.., AllDirectories)`): the directory's files, then each subdirectory.
fn files_of(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    let mut subdirs = Vec::new();
    for e in fs::read_dir(dir)? {
        let e = e?;
        let p = e.path();
        if e.file_type()?.is_dir() {
            subdirs.push(p);
        } else {
            out.push(p);
        }
    }
    for d in subdirs {
        files_of(&d, out)?;
    }
    Ok(())
}

fn with_suffix(p: &Path, suffix: &str) -> PathBuf {
    let mut s = p.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

/// `CreateTarZstd`: one entry per file under each source directory, named relative to `relative_to` with `/`.
/// Writes a temp file first and then renames it over `destination`. The entry count, or -1 on failure.
pub fn create_tar_zstd(
    source_paths: &[PathBuf],
    destination: &Path,
    relative_to: &Path,
    _compression_level: i32,
) -> i32 {
    let temp = with_suffix(destination, ".tmp");
    let tar_temp = with_suffix(destination, ".tmp.tar");
    match create_inner(source_paths, destination, relative_to, &temp, &tar_temp) {
        Ok((count, total)) => {
            tracing::info!(
                "Created archive at {} ({count} entries, {:.1} MB)",
                destination.display(),
                total as f64 / 1_048_576.0
            );
            count
        }
        Err(e) => {
            tracing::error!("Failed to create archive at {}: {e}", destination.display());
            let _ = fs::remove_file(&temp);
            let _ = fs::remove_file(&tar_temp);
            -1
        }
    }
}

fn create_inner(
    sources: &[PathBuf],
    destination: &Path,
    relative_to: &Path,
    temp: &Path,
    tar_temp: &Path,
) -> io::Result<(i32, u64)> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut count = 0;
    let mut total = 0u64;
    {
        let mut builder =
            tar::Builder::new(BufWriter::with_capacity(81920, File::create(tar_temp)?));
        for source in sources {
            if !source.is_dir() {
                tracing::warn!("Source path does not exist, skipping: {}", source.display());
                continue;
            }
            let mut files = Vec::new();
            files_of(source, &mut files)?;
            for file in files {
                let rel = file.strip_prefix(relative_to).unwrap_or(&file);
                let name = rel.to_string_lossy().replace('\\', "/");
                builder.append_path_with_name(&file, name)?;
                count += 1;
                total += fs::metadata(&file)?.len();
                if count % 100 == 0 {
                    tracing::debug!(
                        "Archive progress: {count} entries, {:.1} MB",
                        total as f64 / 1_048_576.0
                    );
                }
            }
        }
        builder.into_inner()?.flush()?;
    }
    {
        let src = BufReader::with_capacity(81920, File::open(tar_temp)?);
        let mut dst = BufWriter::with_capacity(81920, File::create(temp)?);
        ruzstd::encoding::compress(src, &mut dst, ruzstd::encoding::CompressionLevel::Fastest);
        dst.flush()?;
    }
    fs::remove_file(tar_temp)?;
    // Atomic rename on the same filesystem.
    if destination.exists() {
        fs::remove_file(destination)?;
    }
    fs::rename(temp, destination)?;
    Ok((count, total))
}

fn zstd_reader(file: &Path) -> io::Result<Box<dyn Read>> {
    let f = BufReader::with_capacity(81920, File::open(file)?);
    let d = ruzstd::decoding::StreamingDecoder::new(f)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(Box::new(d))
}

/// `ExtractTarZstd`.
pub fn extract_tar_zstd(archive: &Path, out: &Path) -> bool {
    extract_with(archive, out, zstd_reader)
}

/// `ExtractTarGz`.
pub fn extract_tar_gz(archive: &Path, out: &Path) -> bool {
    extract_with(archive, out, |p| {
        Ok(Box::new(flate2::read::GzDecoder::new(
            BufReader::with_capacity(81920, File::open(p)?),
        )))
    })
}

/// `ExtractTar`.
pub fn extract_tar(archive: &Path, out: &Path) -> bool {
    extract_with(archive, out, |p| {
        Ok(Box::new(BufReader::with_capacity(81920, File::open(p)?)))
    })
}

fn extract_with(
    archive: &Path,
    out: &Path,
    open: impl Fn(&Path) -> io::Result<Box<dyn Read>>,
) -> bool {
    let run = || -> io::Result<()> {
        fs::create_dir_all(out)?;
        extract_from(open(archive)?, out)
    };
    match run() {
        Ok(()) => true,
        Err(e) => {
            tracing::error!("Failed to extract archive {}: {e}", archive.display());
            false
        }
    }
}

/// `ExtractTarFromStream`: regular files only; an entry resolving outside `out` is an error.
fn extract_from(stream: impl Read, out: &Path) -> io::Result<()> {
    let mut ar = tar::Archive::new(stream);
    for entry in ar.entries()? {
        let mut entry = entry?;
        let name = entry.path()?.into_owned();
        // Rooted names and ".." segments would otherwise let the archive write anywhere the process can.
        let escapes = name.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        });
        if escapes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Archive entry '{}' resolves outside '{}'",
                    name.display(),
                    out.display()
                ),
            ));
        }
        let dest = out.join(&name);
        if let Some(dir) = dest.parent() {
            fs::create_dir_all(dir)?;
        }
        if entry.header().entry_type().is_file() {
            let mut f = File::create(&dest)?;
            io::copy(&mut entry, &mut f)?;
        }
    }
    Ok(())
}

/// `CountEntries` of a `.tar.zst`: -1 on failure.
pub fn count_entries(archive: &Path) -> i32 {
    let run = || -> io::Result<i32> {
        let mut ar = tar::Archive::new(zstd_reader(archive)?);
        let mut n = 0;
        for e in ar.entries()? {
            e?;
            n += 1;
        }
        Ok(n)
    };
    run().unwrap_or_else(|e| {
        tracing::error!(
            "Failed to count entries in archive {}: {e}",
            archive.display()
        );
        -1
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct Dir(PathBuf);

    impl Dir {
        fn new() -> Dir {
            static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let d = std::env::temp_dir().join(format!(
                "muo_ma_{}_{}",
                std::process::id(),
                N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            ));
            let _ = fs::remove_dir_all(&d);
            fs::create_dir_all(&d).unwrap();
            Dir(d)
        }

        fn make(&self, name: &str, files: &[(&str, &str)]) -> PathBuf {
            let dir = self.0.join(name);
            fs::create_dir_all(&dir).unwrap();
            for (f, c) in files {
                let p = dir.join(f);
                fs::create_dir_all(p.parent().unwrap()).unwrap();
                fs::write(p, c).unwrap();
            }
            dir
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn create_multiple_directories_includes_all() {
        let t = Dir::new();
        let d1 = t.make(
            "backup1",
            &[("items.bin", "items data"), ("items.idx", "items index")],
        );
        let d2 = t.make(
            "backup2",
            &[
                ("mobiles.bin", "mobiles data"),
                ("mobiles.idx", "mobiles index"),
            ],
        );
        let archive = t.0.join("multi.tar.zst");
        assert_eq!(create_tar_zstd(&[d1, d2], &archive, &t.0, 1), 4);
    }

    #[test]
    fn extract_recreates_files() {
        let t = Dir::new();
        let src = t.make(
            "source",
            &[
                ("data.txt", "Test data content"),
                ("config/settings.json", "{\"key\":\"value\"}"),
            ],
        );
        let archive = t.0.join("extract-test.tar.zst");
        create_tar_zstd(&[src], &archive, &t.0, 1);
        let out = t.0.join("extracted");
        assert!(extract_tar_zstd(&archive, &out));
        assert_eq!(
            fs::read_to_string(out.join("source/data.txt")).unwrap(),
            "Test data content"
        );
        assert_eq!(
            fs::read_to_string(out.join("source/config/settings.json")).unwrap(),
            "{\"key\":\"value\"}"
        );
    }

    #[test]
    fn count_entries_matches_create_count() {
        let t = Dir::new();
        let src = t.make(
            "source",
            &[
                ("a.txt", "a"),
                ("b.txt", "b"),
                ("c/d.txt", "d"),
                ("c/e.txt", "e"),
            ],
        );
        let archive = t.0.join("count-test.tar.zst");
        let created = create_tar_zstd(&[src], &archive, &t.0, 1);
        assert_eq!(created, 4);
        assert_eq!(count_entries(&archive), created);
    }

    #[test]
    fn empty_and_nonexistent_sources_give_zero() {
        let t = Dir::new();
        let empty = t.make("empty", &[]);
        assert_eq!(
            create_tar_zstd(&[empty], &t.0.join("empty.tar.zst"), &t.0, 1),
            0
        );
        assert_eq!(
            create_tar_zstd(
                &[t.0.join("does-not-exist")],
                &t.0.join("nonexistent.tar.zst"),
                &t.0,
                1
            ),
            0
        );
    }

    #[test]
    fn temp_file_cleaned_up() {
        let t = Dir::new();
        let src = t.make("source", &[("file.txt", "content")]);
        let archive = t.0.join("cleanup-test.tar.zst");
        create_tar_zstd(&[src], &archive, &t.0, 1);
        assert!(!with_suffix(&archive, ".tmp").exists());
        assert!(!with_suffix(&archive, ".tmp.tar").exists());
        assert!(archive.exists());
    }

    #[test]
    fn invalid_files_fail() {
        let t = Dir::new();
        let bad = t.0.join("bad.tar.zst");
        fs::write(&bad, "this is not a valid archive").unwrap();
        assert!(!extract_tar_zstd(&bad, &t.0.join("extract-bad")));
        assert_eq!(count_entries(&bad), -1);
        let bad_gz = t.0.join("bad.tar.gz");
        fs::write(&bad_gz, "not a valid gzip archive").unwrap();
        assert!(!extract_tar_gz(&bad_gz, &t.0.join("extract-bad-gz")));
    }

    fn tar_of(files: &[(&str, &str)]) -> Vec<u8> {
        let mut b = tar::Builder::new(Vec::new());
        for (name, content) in files {
            let mut h = tar::Header::new_gnu();
            h.set_size(content.len() as u64);
            h.set_entry_type(tar::EntryType::Regular);
            h.set_mode(0o644);
            // `set_path` refuses `..` and rooted names; the bytes are written as an attacker would.
            let raw = h.as_old_mut();
            raw.name = [0; 100];
            raw.name[..name.len()].copy_from_slice(name.as_bytes());
            h.set_cksum();
            b.append(&h, Cursor::new(content.as_bytes())).unwrap();
        }
        b.into_inner().unwrap()
    }

    #[test]
    fn extract_tar_gz_round_trips() {
        let t = Dir::new();
        let tar = tar_of(&[
            ("data.txt", "GZip test data"),
            ("sub/nested.txt", "Nested content"),
        ]);
        let archive = t.0.join("test.tar.gz");
        let mut gz = flate2::write::GzEncoder::new(
            File::create(&archive).unwrap(),
            flate2::Compression::fast(),
        );
        gz.write_all(&tar).unwrap();
        gz.finish().unwrap();
        let out = t.0.join("gz-extracted");
        assert!(extract_tar_gz(&archive, &out));
        assert_eq!(
            fs::read_to_string(out.join("data.txt")).unwrap(),
            "GZip test data"
        );
        assert_eq!(
            fs::read_to_string(out.join("sub/nested.txt")).unwrap(),
            "Nested content"
        );
    }

    #[test]
    fn extract_plain_tar_round_trips() {
        let t = Dir::new();
        let archive = t.0.join("test.tar");
        fs::write(&archive, tar_of(&[("plain.txt", "Plain tar test data")])).unwrap();
        let out = t.0.join("tar-extracted");
        assert!(extract_tar(&archive, &out));
        assert_eq!(
            fs::read_to_string(out.join("plain.txt")).unwrap(),
            "Plain tar test data"
        );
    }

    #[test]
    fn entries_outside_the_output_directory_are_rejected() {
        for name in ["../escaped.txt", "sub/../../escaped.txt"] {
            let t = Dir::new();
            let archive = t.0.join("traversal.tar");
            fs::write(&archive, tar_of(&[(name, "escaped")])).unwrap();
            let out = t.0.join("out").join("extract");
            assert!(!extract_tar(&archive, &out));
            assert!(!t.0.join("out").join("escaped.txt").exists());
            assert!(!t.0.join("escaped.txt").exists());
        }
    }

    #[test]
    fn rooted_entry_is_rejected() {
        let t = Dir::new();
        let target = t.0.join("rooted-escaped.txt");
        let archive = t.0.join("rooted.tar");
        fs::write(&archive, tar_of(&[(&target.to_string_lossy(), "escaped")])).unwrap();
        assert!(!extract_tar(&archive, &t.0.join("rooted-extract")));
        assert!(!target.exists());
    }
}
