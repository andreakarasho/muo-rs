//! `TextDefinition`: a cliloc number or a literal string.

use muo_primitives::parse::try_to_i32;
use std::fmt;

#[derive(Clone, Debug, Default)]
pub struct TextDefinition {
    pub number: i32,
    pub string: Option<String>,
}

impl TextDefinition {
    pub const EMPTY: Self = Self {
        number: 0,
        string: None,
    };

    pub fn of_number(number: i32) -> Self {
        Self {
            number,
            string: None,
        }
    }

    pub fn of_text(text: impl Into<String>) -> Self {
        Self {
            number: 0,
            string: Some(text.into()),
        }
    }

    pub fn of(number: i32, text: Option<&str>) -> Self {
        Self {
            number,
            string: text.map(str::to_owned),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.number <= 0 && self.string.is_none()
    }

    /// `Format()`: `"1234 (0x4D2)"`, `"\"text\""` or `None`.
    pub fn format(&self) -> Option<String> {
        if self.number > 0 {
            Some(format!("{} (0x{:X})", self.number, self.number))
        } else {
            self.string.as_ref().map(|s| format!("\"{s}\""))
        }
    }

    /// `GetValue()`: the editable text form; quotes a string that `parse` would read as something else.
    pub fn get_value(&self) -> String {
        if self.number > 0 {
            return self.number.to_string();
        }
        match &self.string {
            None => String::new(),
            Some(s) => {
                let p = Self::parse(s);
                if p.number == 0 && p.string.as_deref() == Some(s.as_str()) {
                    s.clone()
                } else {
                    format!("@\"{s}\"")
                }
            }
        }
    }

    /// `#1234` or a bare `1234`/`0x4D2` is a cliloc; `@"1234"` is the literal text. Always succeeds: anything that
    /// is not a cliloc is a string. No trimming.
    pub fn parse(s: &str) -> Self {
        if s.len() >= 3 && s.starts_with("@\"") && s.ends_with('"') {
            return Self::of_text(&s[2..s.len() - 1]);
        }
        if let Some(n) = s
            .strip_prefix('#')
            .filter(|r| !r.is_empty())
            .and_then(try_to_i32)
        {
            return Self::of_number(n);
        }
        if let Some(n) = try_to_i32(s) {
            return Self::of_number(n);
        }
        Self::of_text(s)
    }
}

impl PartialEq for TextDefinition {
    fn eq(&self, other: &Self) -> bool {
        if self.number > 0 || other.number > 0 {
            self.number == other.number
        } else {
            self.string == other.string
        }
    }
}

impl Eq for TextDefinition {}

impl fmt::Display for TextDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.number > 0 {
            write!(f, "#{}", self.number)
        } else {
            f.write_str(self.string.as_deref().unwrap_or(""))
        }
    }
}

impl From<i32> for TextDefinition {
    fn from(n: i32) -> Self {
        Self::of_number(n)
    }
}

impl From<&str> for TextDefinition {
    fn from(s: &str) -> Self {
        Self::of_text(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        for (n, s, expected) in [
            (10345, "", "#10345"),
            (0, "Hello", "Hello"),
            (10345, "Hello", "#10345"),
        ] {
            assert_eq!(TextDefinition::of(n, Some(s)).to_string(), expected);
        }
    }

    #[test]
    fn equality() {
        let cases = [
            (10345, "", 10345, "", true),
            (10345, "", 0, "Hello", false),
            (0, "Hello", 0, "Hello", true),
            (0, "Hello", 0, "hello", false),
            (10345, "Hello", 10345, "Goodbye", true),
            (10345, "Hello", 10510, "Hello", false),
        ];
        for (a1, a2, b1, b2, eq) in cases {
            assert_eq!(
                TextDefinition::of(a1, Some(a2)) == TextDefinition::of(b1, Some(b2)),
                eq
            );
        }
    }

    #[test]
    fn parse_and_value() {
        assert_eq!(TextDefinition::parse("#1234").number, 1234);
        assert_eq!(TextDefinition::parse("0x4D2").number, 1234);
        assert_eq!(
            TextDefinition::parse("@\"1234\"").string.as_deref(),
            Some("1234")
        );
        assert_eq!(
            TextDefinition::parse("hello").string.as_deref(),
            Some("hello")
        );
        assert_eq!(TextDefinition::of_text("1234").get_value(), "@\"1234\"");
        assert_eq!(TextDefinition::of_text("hi").get_value(), "hi");
        assert_eq!(
            TextDefinition::of_number(7).format().as_deref(),
            Some("7 (0x7)")
        );
    }

    #[test]
    fn text_definition_to_string() {
        for (n, s, expected) in [
            (10345, "", "#10345"),
            (0, "Hello", "Hello"),
            (10345, "Hello", "#10345"),
        ] {
            assert_eq!(TextDefinition::of(n, Some(s)).to_string(), expected);
        }
    }

    #[test]
    fn text_definition_is_equal() {
        for (a1, a2, b1, b2, eq) in [
            (10345, "", 10345, "", true),
            (10345, "", 0, "Hello", false),
            (0, "Hello", 0, "Hello", true),
            (0, "Hello", 0, "hello", false),
            (10345, "Hello", 10345, "Goodbye", true),
            (10345, "Hello", 10510, "Hello", false),
        ] {
            let a = TextDefinition::of(a1, Some(a2));
            let b = TextDefinition::of(b1, Some(b2));
            assert_eq!(a == b, eq, "{a1} {a2} {b1} {b2}");
        }
    }
}
