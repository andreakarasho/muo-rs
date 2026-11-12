//! The string helpers that content uses.
//!
//! Not provided: `Remove(span, span, comparison)` (std `replace`/`split` cover it), `ToPooledArray`,
//! `AppendSpaceWithArticle` (ValueStringBuilder), `ReplaceAny`, `IndexOfTerminator` (private to the cursors).

/// `DefaultIfNullOrEmpty`: the default for null, empty or all-white-space text.
pub fn default_if_blank<'a>(value: Option<&'a str>, default: &'a str) -> &'a str {
    match value {
        Some(v) if !v.trim().is_empty() => v,
        _ => default,
    }
}

fn to_upper_invariant(c: char) -> char {
    let mut up = c.to_uppercase();
    match (up.next(), up.next()) {
        (Some(u), None) => u,
        _ => c,
    }
}

/// `Capitalize`: upper-cases the first letter of every word, skipping a "the " in front of a word.
pub fn capitalize(value: &str) -> String {
    let mut chars: Vec<char> = value.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let rest = &chars[index..];
        if rest.len() >= 4
            && rest[..4]
                .iter()
                .collect::<String>()
                .eq_ignore_ascii_case("the ")
        {
            index += 4;
            continue;
        }
        let space = rest.iter().position(|&c| c == ' ');
        let last = rest.len() - 1;
        chars[index] = to_upper_invariant(chars[index]);
        match space {
            Some(i) if i != last => index += i + 1,
            _ => break,
        }
    }
    chars.into_iter().collect()
}

/// `TrimMultiline`: trims every line.
pub fn trim_multiline(s: &str, line_separator: &str) -> String {
    s.split(line_separator)
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(line_separator)
}

/// `IndentMultiline`: prefixes every line.
pub fn indent_multiline(s: &str, indent: &str, line_separator: &str) -> String {
    s.split(line_separator)
        .map(|l| format!("{indent}{l}"))
        .collect::<Vec<_>>()
        .join(line_separator)
}

/// `Wrap`: word-wraps into at most `max_lines` lines of `per_line` characters; `None` for blank text.
pub fn wrap(value: &str, per_line: usize, max_lines: usize) -> Option<Vec<String>> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let all: Vec<char> = value.chars().collect();
    let mut span: &[char] = &all;
    let mut list: Vec<String> = Vec::with_capacity(max_lines);
    let text = |s: &[char]| s.iter().collect::<String>();
    let mut line_length = 0usize;

    while !span.is_empty() {
        let space_index = span[line_length..]
            .iter()
            .position(|&c| c == ' ')
            .unwrap_or(span.len() - line_length);
        let new_line_length = line_length + space_index;

        if new_line_length == per_line
            || (new_line_length < per_line && new_line_length == span.len())
        {
            list.push(text(&span[..new_line_length]));
            if list.len() == max_lines || new_line_length == span.len() {
                break;
            }
            span = &span[new_line_length + 1..];
            line_length = 0;
        } else if new_line_length < per_line {
            line_length = new_line_length + 1;
        } else if line_length > 0 && line_length <= per_line {
            list.push(text(&span[..line_length - 1]));
            if list.len() == max_lines {
                break;
            }
            span = &span[line_length..];
            line_length = 0;
        } else {
            // A single word longer than a line: break it up forcibly.
            let mut remaining = new_line_length as isize;
            let mut index = 0usize;
            while (index as isize) < remaining {
                remaining -= per_line as isize;
                let length = per_line - usize::from(span.get(index) == Some(&' '));
                let end = (index + length).min(span.len());
                list.push(text(&span[index.min(end)..end]));
                if list.len() == max_lines {
                    break;
                }
                index += per_line;
            }
            let skip = (new_line_length as isize - remaining).max(0) as usize;
            span = &span[skip.min(span.len())..];
            line_length = 0;
        }
    }

    Some(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitalizes_words() {
        assert_eq!(capitalize("a bag of stuff"), "A Bag Of Stuff");
        assert_eq!(capitalize("the rat"), "the Rat");
        assert_eq!(capitalize("The rat"), "The Rat");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("the "), "the ");
        assert_eq!(capitalize("x "), "X ");
    }

    #[test]
    fn multiline() {
        assert_eq!(trim_multiline(" a \n b ", "\n"), "a\nb");
        assert_eq!(indent_multiline("a\nb", "\t", "\n"), "\ta\n\tb");
        assert_eq!(default_if_blank(Some("  "), "d"), "d");
        assert_eq!(default_if_blank(Some("x"), "d"), "x");
    }

    #[test]
    fn wraps() {
        assert_eq!(wrap("  ", 5, 3), None);
        assert_eq!(
            wrap("hello world foo", 11, 5).unwrap(),
            ["hello world", "foo"]
        );
        assert_eq!(wrap("aa bb cc", 5, 5).unwrap(), ["aa bb", "cc"]);
        assert_eq!(wrap("abcdefgh", 3, 5).unwrap(), ["abc", "def", "gh"]);
        assert_eq!(wrap("aa bb cc dd", 5, 2).unwrap(), ["aa bb", "cc dd"]);
    }

    // The new Wrap against the old HouseGump wrap, plus the literal case.
    fn old_wrap(value: &str, start_index: usize, max_lines: usize) -> Option<Vec<String>> {
        let value = value.trim();
        if value.is_empty() {
            return None;
        }
        let mut list: Vec<String> = Vec::new();
        let mut current = String::new();
        for val in value.split(' ') {
            let mut v = if current.is_empty() {
                val.to_string()
            } else {
                format!("{current} {val}")
            };
            let vlen = v.chars().count();
            if vlen < start_index {
                current = v;
            } else if vlen == start_index {
                list.push(v);
                if list.len() == max_lines {
                    return Some(list);
                }
                current = String::new();
            } else if val.chars().count() <= start_index {
                list.push(current);
                if list.len() == max_lines {
                    return Some(list);
                }
                current = val.to_string();
            } else {
                while v.chars().count() >= start_index {
                    list.push(v.chars().take(start_index).collect());
                    if list.len() == max_lines {
                        return Some(list);
                    }
                    v = v.chars().skip(start_index).collect();
                }
                current = v;
            }
        }
        if !current.is_empty() {
            list.push(current);
        }
        Some(list)
    }

    #[test]
    fn wrap_matches_the_old_house_gump_wrap() {
        for s in [
            "this is a sentence that will probably wrap around a few times because it is long",
            "An Unnamed House",
            "Batville",
            "Bald's Shop",
        ] {
            assert_eq!(wrap(s, 10, 6), old_wrap(s, 10, 6), "{s}");
        }
        assert_eq!(
            wrap("Something ThatIsVeryLongAndShouldBe broken up", 10, 6).unwrap(),
            [
                "Something",
                "ThatIsVery",
                "LongAndSho",
                "uldBe",
                "broken up"
            ]
        );
    }
}
