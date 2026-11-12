//! JSON config reading and writing.
//!
//! `JsonConfig.DefaultOptions` read with comments skipped and trailing commas allowed, and wrote indented with null
//! properties omitted. serde_json is strict, so `from_str` strips comments and trailing commas first; omitting nulls
//! is `#[serde(skip_serializing_if = "Option::is_none")]` on the fields of the config struct. Type-specific shapes
//! (`Point3D`, `TimeSpan`, flags...) are the `converters` modules, used with `#[serde(with = "...")]`.
//!
//! // shortcut: `JsonSerializerOptions` customization (`GetOptions(extra converters)`) has no counterpart; custom
//! shapes are `#[serde(with)]` on the field. Indented output uses LF (not `Environment.NewLine`) and lowercase
//! `\u00xx` escapes.

pub mod converters;
mod utilities;

pub use utilities::sort_by_property_name;

use crate::error::{DataError, Result};
use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;

/// Removes `//` and `/* */` comments and commas before `]`/`}`, outside strings.
pub fn strip_jsonc(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let (mut i, mut in_str) = (0, false);
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            out.push(c);
            if c == '\\' && i + 1 < chars.len() {
                out.push(chars[i + 1]);
                i += 1;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
            out.push(c);
        } else if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        } else if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            out.push(' ');
            continue;
        } else {
            out.push(c);
        }
        i += 1;
    }
    strip_trailing_commas(&out)
}

fn strip_trailing_commas(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let (mut i, mut in_str) = (0, false);
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            out.push(c);
            if c == '\\' && i + 1 < chars.len() {
                out.push(chars[i + 1]);
                i += 1;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
            out.push(c);
        } else if c == ','
            && matches!(
                chars[i + 1..].iter().find(|c| !c.is_whitespace()),
                Some(']' | '}')
            )
        {
            // dropped
        } else {
            out.push(c);
        }
        i += 1;
    }
    out
}

/// `JsonSerializer.Deserialize<T>(text, DefaultOptions)`.
pub fn from_str<T: DeserializeOwned>(text: &str) -> serde_json::Result<T> {
    serde_json::from_str(&strip_jsonc(text))
}

/// `JsonConfig.Serialize(value)`: indented.
pub fn to_string<T: Serialize + ?Sized>(value: &T) -> serde_json::Result<String> {
    serde_json::to_string_pretty(value)
}

/// `JsonConfig.Deserialize<T>(filePath)`: `None` when the file does not exist.
pub fn deserialize_file<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path).map_err(|e| DataError::io(path, e))?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    from_str(text).map(Some).map_err(|source| DataError::Json {
        path: path.to_path_buf(),
        source,
    })
}

/// `JsonConfig.Serialize(filePath, value)`: replaces the file, creating its directory.
pub fn serialize_file<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    let contents = to_string(value).map_err(|source| DataError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| DataError::io(dir, e))?;
    }
    std::fs::write(path, contents).map_err(|e| DataError::io(path, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct T {
        a: i32,
        b: Vec<String>,
    }

    #[test]
    fn comments_and_trailing_commas_are_accepted() {
        let t: T =
            from_str("{ // x\n \"a\": 1, /* y */ \"b\": [\"//no\", \"a,]\",\n ],\n }").unwrap();
        assert_eq!(
            t,
            T {
                a: 1,
                b: vec!["//no".into(), "a,]".into()]
            }
        );
        let t: T = from_str("{\"a\": 1, \"b\": [] , // tail\n}").unwrap();
        assert_eq!(t.a, 1);
    }

    #[test]
    fn missing_file_is_none_and_garbage_is_an_error() {
        let dir = std::env::temp_dir().join(format!("muo_json_{}", std::process::id()));
        let p = dir.join("sub").join("c.json");
        assert!(deserialize_file::<T>(&p).unwrap().is_none());
        serialize_file(&p, &serde_json::json!({"a": 2, "b": []})).unwrap();
        assert_eq!(deserialize_file::<T>(&p).unwrap().unwrap().a, 2);
        std::fs::write(&p, "{ nope").unwrap();
        assert!(deserialize_file::<T>(&p).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
