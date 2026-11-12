//! `LocalizationEntry`: one cliloc.
//!
//! [`LocalizationEntry::format`] fills the `~1_NAME~` slots of a cliloc from a list of [`FormatArg`].

/// One argument of a cliloc format. `{n:#}` (a number that is itself a cliloc to look up) is `Cliloc`; a
/// `Text` starting with `#` followed by a number is also looked up (`TryAppendClilocByNumericString`).
#[derive(Clone, Debug, PartialEq)]
pub enum FormatArg {
    Text(String),
    Number(i64),
    Cliloc(i64),
}

impl From<&str> for FormatArg {
    fn from(s: &str) -> Self {
        FormatArg::Text(s.to_string())
    }
}

impl From<String> for FormatArg {
    fn from(s: String) -> Self {
        FormatArg::Text(s)
    }
}

impl From<i32> for FormatArg {
    fn from(n: i32) -> Self {
        FormatArg::Number(i64::from(n))
    }
}

impl From<i64> for FormatArg {
    fn from(n: i64) -> Self {
        FormatArg::Number(n)
    }
}

#[derive(Clone, Debug)]
pub struct LocalizationEntry {
    pub language: String,
    pub number: i32,
    pub text: String,
    /// The text cut at its `~n_name~` placeholders: `None` is a placeholder, `Some` a literal run.
    pub text_slices: Vec<Option<String>>,
    /// The text with placeholders as `{n-1}`; `None` when it has none.
    pub string_formatter: Option<String>,
}

fn is_word(u: u16) -> bool {
    char::from_u32(u32::from(u)).is_some_and(|c| c == '_' || c.is_alphanumeric())
}

/// The matches of `~(\d+)[_\w]+~` as `(start, end, group 1)` over UTF-16 units. The regex backtracks: an all-digit
/// body `~12~` matches with group 1 `1`.
fn placeholders(units: &[u16]) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < units.len() {
        if units[i] != b'~' as u16 {
            i += 1;
            continue;
        }
        let body = units[i + 1..].iter().take_while(|&&u| is_word(u)).count();
        let end = i + 1 + body;
        let digits = units[i + 1..end]
            .iter()
            .take_while(|&&u| (b'0' as u16..=b'9' as u16).contains(&u))
            .count();
        if body >= 2 && digits >= 1 && units.get(end) == Some(&(b'~' as u16)) {
            let group = if digits == body { digits - 1 } else { digits };
            out.push((
                i,
                end + 1,
                String::from_utf16_lossy(&units[i + 1..i + 1 + group]),
            ));
            i = end + 1;
        } else {
            i += 1;
        }
    }
    out
}

impl LocalizationEntry {
    pub fn new(language: impl Into<String>, number: i32, text: impl Into<String>) -> Self {
        let text = text.into();
        let (text_slices, string_formatter) = parse_text(&text);
        LocalizationEntry {
            language: language.into(),
            number,
            text,
            text_slices,
            string_formatter,
        }
    }

    /// Fills the placeholders with `args` in order. `resolve(number)` gives the text of another cliloc in this
    /// entry's language. Extra arguments are ignored; missing ones leave their slot (and the text after it) out.
    pub fn format(
        &self,
        args: &[FormatArg],
        resolve: &mut dyn FnMut(i32) -> Option<String>,
    ) -> String {
        let slices = &self.text_slices;
        let mut out = String::new();
        let mut next = 0;
        for arg in args {
            // ReadyToAppend
            let Some(first) = slices.get(next) else { break };
            next += 1;
            if let Some(literal) = first {
                out.push_str(literal);
                if slices.get(next).is_none() {
                    break;
                }
                next += 1;
            }
            match arg {
                FormatArg::Number(n) => out.push_str(&n.to_string()),
                FormatArg::Cliloc(n) => match resolve(*n as i32) {
                    Some(t) => out.push_str(&t),
                    None => out.push_str(&n.to_string()),
                },
                FormatArg::Text(s) => {
                    let cliloc = s
                        .strip_prefix('#')
                        .and_then(|n| n.trim().parse::<i64>().ok())
                        .and_then(|n| resolve(n as i32));
                    out.push_str(cliloc.as_deref().unwrap_or(s));
                }
            }
        }
        if let Some(Some(tail)) = slices.get(next) {
            out.push_str(tail);
        }
        out
    }
}

fn parse_text(text: &str) -> (Vec<Option<String>>, Option<String>) {
    let units: Vec<u16> = text.encode_utf16().collect();
    let sub = |a: usize, b: usize| String::from_utf16_lossy(&units[a..b]);
    let mut slices = Vec::new();
    let mut formatter = String::new();
    let mut has_match = false;
    let mut prev = 0;
    for (start, end, group) in placeholders(&units) {
        if prev < start {
            let s = sub(prev, start);
            formatter.push_str(&s);
            slices.push(Some(s));
        }
        slices.push(None);
        has_match = true;
        formatter.push_str(&format!("{{{}}}", group.parse::<i64>().unwrap_or(0) - 1));
        prev = end;
    }
    // The last character alone is not kept as a trailing run (`prevIndex < text.Length - 1`).
    if prev + 1 < units.len() {
        let s = if prev == 0 {
            text.to_string()
        } else {
            sub(prev, units.len())
        };
        formatter.push_str(&s);
        slices.push(Some(s));
    }
    (slices, has_match.then_some(formatter))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slices_and_formatter() {
        let e = LocalizationEntry::new("enu", 1, "You have ~1_COUNT~ of ~2_MAX~ items.");
        assert_eq!(
            e.text_slices,
            vec![
                Some("You have ".into()),
                None,
                Some(" of ".into()),
                None,
                Some(" items.".into())
            ]
        );
        assert_eq!(
            e.string_formatter.as_deref(),
            Some("You have {0} of {1} items.")
        );
        let plain = LocalizationEntry::new("enu", 2, "plain");
        assert_eq!(plain.text_slices, vec![Some("plain".to_string())]);
        assert_eq!(plain.string_formatter, None);
        // a lone trailing character is not a slice, and a one character text has none at all
        assert_eq!(
            LocalizationEntry::new("enu", 3, "~1_A~!").text_slices,
            vec![None]
        );
        assert!(LocalizationEntry::new("enu", 3, "x").text_slices.is_empty());
        // the regex backtracks: an all-digit body gives up its last digit
        assert_eq!(
            LocalizationEntry::new("enu", 4, "~12~")
                .string_formatter
                .as_deref(),
            Some("{0}")
        );
        assert_eq!(
            LocalizationEntry::new("enu", 5, "~a~1_B~x y").text_slices,
            vec![Some("~a".into()), None, Some("x y".into())]
        );
    }

    #[test]
    fn format_fills_slots_in_order() {
        let e = LocalizationEntry::new("enu", 1, "You have ~1_COUNT~ of ~2_MAX~ items.");
        let mut none = |_: i32| None;
        assert_eq!(
            e.format(&[3.into(), "10".into()], &mut none),
            "You have 3 of 10 items."
        );
        // missing arguments stop at the first empty slot, keeping the literal that follows the last filled one
        assert_eq!(e.format(&[3.into()], &mut none), "You have 3 of ");
        assert_eq!(e.format(&[], &mut none), "You have ");
        let mut table = |n: i32| (n == 500003).then(|| "clilocs".to_string());
        assert_eq!(
            e.format(&[FormatArg::Cliloc(500003), "#500003".into()], &mut table),
            "You have clilocs of clilocs items."
        );
        assert_eq!(
            e.format(&[FormatArg::Cliloc(9)], &mut none),
            "You have 9 of "
        );
    }
}
