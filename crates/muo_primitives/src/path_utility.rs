//! Path helpers. `Core.BaseDirectory` is an explicit `base` argument.

use std::io;
use std::path::{Component, Path, PathBuf};

use crate::random::Rng;

/// Lexically resolves `.` and `..` (what `Path.GetFullPath` does; it never touches the disk).
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            c => out.push(c.as_os_str()),
        }
    }
    out
}

/// `GetFullPath(path, basePath)`: empty is the base, a rooted path is kept as is, a relative one is
/// resolved against the base.
pub fn get_full_path(relative_or_absolute: &str, base: &Path) -> PathBuf {
    if relative_or_absolute.is_empty() {
        return base.to_path_buf();
    }
    let p = Path::new(relative_or_absolute);
    if p.has_root() {
        p.to_path_buf()
    } else {
        normalize(&base.join(p))
    }
}

/// `EnsureDirectory(dir)`: the full path, created if missing.
pub fn ensure_directory(dir: &str, base: &Path) -> io::Result<PathBuf> {
    let path = get_full_path(dir, base);
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

/// `EnsureDirectory(this FileInfo)`: creates the file's parent directory.
pub fn ensure_parent_directory(file: &Path, base: &Path) -> io::Result<()> {
    match file.parent().and_then(Path::to_str) {
        Some(dir) if !dir.is_empty() => std::fs::create_dir_all(get_full_path(dir, base)),
        _ => Ok(()),
    }
}

/// `EnsureRandomPath`: `base_path/<16 hex digits>`, created.
pub fn ensure_random_path(
    base_path: &Path,
    base: &Path,
    rng: &mut impl Rng,
) -> io::Result<PathBuf> {
    let mut bytes = [0u8; 8];
    rng.fill_bytes(&mut bytes);
    let name: String = bytes.iter().map(|b| format!("{b:02X}")).collect();
    ensure_directory(&base_path.join(name).to_string_lossy(), base)
}

/// `CopyDirectoryContents`: overwrites files that exist.
pub fn copy_directory_contents(source: &Path, dest: &Path, recursive: bool) -> io::Result<()> {
    if !source.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source directory not found: {}", source.display()),
        ));
    }
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            if recursive {
                copy_directory_contents(&entry.path(), &target, true)?;
            }
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// `MoveDirectoryContents`: moves files (failing if one exists, like `File.MoveTo`), then removes the
/// source directory, ignoring a failure to do so.
pub fn move_directory_contents(source: &Path, dest: &Path, recursive: bool) -> io::Result<()> {
    if !source.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source directory not found: {}", source.display()),
        ));
    }
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            if recursive {
                move_directory_contents(&entry.path(), &target, true)?;
            }
        } else {
            if target.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("{} exists", target.display()),
                ));
            }
            // rename fails across volumes; copy and delete then
            if std::fs::rename(entry.path(), &target).is_err() {
                std::fs::copy(entry.path(), &target)?;
                std::fs::remove_file(entry.path())?;
            }
        }
    }
    let _ = std::fs::remove_dir_all(source);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::BuiltInRng;

    fn tmp(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("muo_primitives_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn full_paths() {
        let base = Path::new("base").join("dir");
        assert_eq!(get_full_path("", &base), base);
        assert_eq!(get_full_path("a/./b/../c", &base), base.join("a").join("c"));
        let abs = std::env::temp_dir();
        assert_eq!(get_full_path(&abs.to_string_lossy(), &base), abs);
    }

    #[test]
    fn create_copy_move() {
        let root = tmp("fs");
        let made = ensure_directory("one/two", &root).unwrap();
        assert!(made.is_dir());
        std::fs::write(made.join("f.txt"), "hi").unwrap();
        std::fs::create_dir_all(made.join("sub")).unwrap();
        std::fs::write(made.join("sub").join("g.txt"), "yo").unwrap();

        let copy = root.join("copy");
        copy_directory_contents(&made, &copy, true).unwrap();
        assert_eq!(
            std::fs::read_to_string(copy.join("sub").join("g.txt")).unwrap(),
            "yo"
        );

        let moved = root.join("moved");
        move_directory_contents(&copy, &moved, true).unwrap();
        assert!(moved.join("f.txt").is_file() && !copy.exists());
        assert!(move_directory_contents(&copy, &moved, true).is_err());

        let rnd =
            ensure_random_path(&root.join("rnd"), &root, &mut BuiltInRng::from_seed(5)).unwrap();
        assert!(rnd.is_dir() && rnd.file_name().unwrap().len() == 16);
        ensure_parent_directory(&root.join("p").join("q").join("file.bin"), &root).unwrap();
        assert!(root.join("p").join("q").is_dir());
        let _ = std::fs::remove_dir_all(&root);
    }
}
