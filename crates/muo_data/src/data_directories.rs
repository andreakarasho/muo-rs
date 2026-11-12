//! `Core.FindDataFile` / `Core.FindDataFileByPattern` over the `dataDirectories` set.

use indexmap::IndexSet;
use muo_primitives::dotnet::InvariantCase;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The `dataDirectories` setting: a set that enumerates in insertion order (a `HashSet<string>` without removals).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DataDirectories(pub IndexSet<String>);

impl Serialize for DataDirectories {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(&self.0)
    }
}

impl<'de> Deserialize<'de> for DataDirectories {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(DataDirectories(
            Vec::<String>::deserialize(d)?.into_iter().collect(),
        ))
    }
}

impl DataDirectories {
    pub fn new<I: IntoIterator<Item = S>, S: Into<String>>(dirs: I) -> Self {
        DataDirectories(dirs.into_iter().map(Into::into).collect())
    }

    pub fn insert(&mut self, dir: impl Into<String>) -> bool {
        self.0.insert(dir.into())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter()
    }

    /// First directory holding `path`. Case-insensitive on non-Windows, as `Core.IsLinux` did.
    pub fn find(&self, path: &str) -> Option<PathBuf> {
        self.0.iter().find_map(|dir| {
            let full = Path::new(dir).join(path);
            if full.is_file() {
                return Some(full);
            }
            if cfg!(windows) {
                return None;
            }
            let parent = full.parent()?;
            let name = full.file_name()?.to_string_lossy().lower_inv();
            std::fs::read_dir(parent)
                .ok()?
                .flatten()
                .find(|e| e.file_name().to_string_lossy().lower_inv() == name && e.path().is_file())
                .map(|e| e.path())
        })
    }

    /// `FindDataFile(path, throwNotFound: true)`.
    pub fn require(&self, path: &str) -> crate::Result<PathBuf> {
        self.find(path)
            .ok_or_else(|| crate::DataError::NotFound(path.to_string()))
    }

    /// `FindDataFileByPattern("cliloc.*")`: `*` and `?` wildcards, case-insensitive, files only.
    pub fn find_by_pattern(&self, pattern: &str) -> Vec<PathBuf> {
        let pat = pattern.lower_inv();
        let mut out = Vec::new();
        for dir in &self.0 {
            let Ok(rd) = std::fs::read_dir(dir) else {
                continue;
            };
            for e in rd.flatten() {
                if e.path().is_file()
                    && wildcard(&pat, &e.file_name().to_string_lossy().lower_inv())
                {
                    out.push(e.path());
                }
            }
        }
        out
    }
}

fn wildcard(pat: &str, s: &str) -> bool {
    fn go(p: &[char], s: &[char]) -> bool {
        match p.split_first() {
            None => s.is_empty(),
            Some(('*', rest)) => (0..=s.len()).any(|i| go(rest, &s[i..])),
            Some(('?', rest)) => !s.is_empty() && go(rest, &s[1..]),
            Some((c, rest)) => s.first() == Some(c) && go(rest, &s[1..]),
        }
    }
    go(
        &pat.chars().collect::<Vec<_>>(),
        &s.chars().collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcards() {
        assert!(wildcard("cliloc.*", "cliloc.enu"));
        assert!(!wildcard("cliloc.*", "cliloc"));
        assert!(wildcard("a?c", "abc"));
    }
}
