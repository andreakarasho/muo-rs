//! Name verification (and the `SearchValues` it shares with profanity protection).
//!
//! `SearchValues<char>` is a `&[char]` set here and `SearchValues<string>` (ordinal, ignore case) is a word
//! list: `IndexOfAny` is the lowest index at which any word starts.

use std::rc::Rc;

use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages;

use super::profanity_protection;

pub const ALPHA_NUMERIC: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z',
];

pub const ALPHABETIC: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

pub const NUMERIC: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

pub const SPACE_DASH_PERIOD_QUOTE: &[char] = &[' ', '-', '.', '\''];

pub const START_DISALLOWED: &[&str] = &["seer", "counselor", "gm", "admin", "lady", "lord"];

/// `NameVerification.Disallowed`: `ProfanityProtection.Disallowed` then these.
pub const EXTRA_DISALLOWED: &[&str] = &[
    "tailor",
    "smith",
    "scholar",
    "rogue",
    "novice",
    "neophyte",
    "merchant",
    "medium",
    "master",
    "mage",
    "lb",
    "journeyman",
    "grandmaster",
    "fisherman",
    "expert",
    "chef",
    "carpenter",
    "british",
    "blackthorne",
    "blackthorn",
    "beggar",
    "archer",
    "apprentice",
    "adept",
    "gamemaster",
    "frozen",
    "squelched",
    "invulnerable",
    "osi",
    "origin",
];

/// `NameVerification.Disallowed` in full.
pub fn disallowed() -> Vec<&'static str> {
    profanity_protection::DISALLOWED
        .iter()
        .chain(EXTRA_DISALLOWED)
        .copied()
        .collect()
}

fn upper(c: char) -> char {
    muo_primitives::dotnet::simple_upper(c)
}

/// `a.InsensitiveEquals(b)` over chars (ordinal, ignore case).
fn equals_ignore_case(a: &[char], b: &str) -> bool {
    let mut it = b.chars();
    a.iter()
        .all(|c| it.next().is_some_and(|d| upper(*c) == upper(d)))
        && it.next().is_none()
}

/// `name.IndexOfAny(SearchValues<string>)`: the lowest index where any word matches.
pub(crate) fn index_of_any_word(name: &[char], words: &[&str]) -> Option<usize> {
    (0..name.len()).find(|i| {
        words.iter().any(|word| {
            let n = word.chars().count();
            n > 0 && *i + n <= name.len() && equals_ignore_case(&name[*i..*i + n], word)
        })
    })
}

fn index_of_any(name: &[char], set: &[char]) -> Option<usize> {
    name.iter().position(|c| set.contains(c))
}

/// `ValidatePlayerName`.
pub fn validate_player_name(name: &str) -> bool {
    validate(
        name,
        2,
        16,
        true,
        false,
        true,
        1,
        Some(SPACE_DASH_PERIOD_QUOTE),
    )
}

/// `ValidatePetName`.
pub fn validate_pet_name(name: &str) -> bool {
    validate(name, 1, 16, true, false, true, 0, None)
}

/// `ValidateVendorName`.
pub fn validate_vendor_name(name: &str) -> bool {
    validate(name, 1, 20, true, true, true, 0, None)
}

/// `Validate(name, minLength, maxLength, allowLetters, allowDigits, noExceptionsAtStart, maxExceptions,
/// exceptions)`: with the standard disallowed words and start-disallowed words.
#[allow(clippy::too_many_arguments)]
pub fn validate(
    name: &str,
    min_length: i32,
    max_length: i32,
    allow_letters: bool,
    allow_digits: bool,
    no_exceptions_at_start: bool,
    max_exceptions: i32,
    exceptions: Option<&[char]>,
) -> bool {
    let words = disallowed();
    validate_with(
        name,
        min_length,
        max_length,
        allow_letters,
        allow_digits,
        no_exceptions_at_start,
        max_exceptions,
        exceptions,
        &words,
        Some(START_DISALLOWED),
    )
}

/// The full `Validate`. `start_disallowed` is the `SearchValues<string>` of words a name may not start with.
#[allow(clippy::too_many_arguments)]
pub fn validate_with(
    name: &str,
    min_length: i32,
    max_length: i32,
    allow_letters: bool,
    allow_digits: bool,
    no_exceptions_at_start: bool,
    max_exceptions: i32,
    exceptions: Option<&[char]>,
    disallowed: &[&str],
    start_disallowed: Option<&[&str]>,
) -> bool {
    let name: Vec<char> = name
        .encode_utf16()
        .map(|u| char::from_u32(u32::from(u)).unwrap_or('\u{FFFD}'))
        .collect();
    let len = name.len() as i64;
    if name.is_empty() || len < i64::from(min_length) || len > i64::from(max_length) {
        return false;
    }

    match exceptions {
        None => {
            // We don't have exceptions, so we might be limited to letters or numbers
            let allowed: Option<&[char]> = if allow_letters && allow_digits && max_exceptions == 0 {
                // If we don't allow exceptions, then non-alphanumeric is not allowed
                Some(ALPHA_NUMERIC)
            } else if allow_letters && !allow_digits {
                Some(ALPHABETIC)
            } else if !allow_letters && allow_digits {
                Some(NUMERIC)
            } else {
                None // everything has been allowed
            };
            if let Some(allowed) = allowed
                && name.iter().any(|c| !allowed.contains(c))
            {
                return false;
            }
        }
        Some(exceptions) => {
            // We have exceptions, and at least one of the letters/digits flag is false
            let not_allowed: Option<&[char]> = if allow_letters && !allow_digits {
                Some(NUMERIC)
            } else if !allow_letters && allow_digits {
                Some(ALPHABETIC)
            } else {
                None
            };
            if let Some(not_allowed) = not_allowed
                && name.iter().any(|c| not_allowed.contains(c))
            {
                return false;
            }
            if contains_exceptions(&name, exceptions, no_exceptions_at_start, max_exceptions) {
                return false;
            }
        }
    }

    if !disallowed.is_empty() && contains_disallowed_word(&name, disallowed) {
        return false;
    }

    match start_disallowed {
        None => true,
        Some(words) => index_of_any_word(&name, words) != Some(0),
    }
}

/// `ContainsExceptions`.
pub fn contains_exceptions(
    name: &[char],
    exceptions: &[char],
    mut no_exceptions_at_start: bool,
    max_exceptions: i32,
) -> bool {
    if !no_exceptions_at_start && (max_exceptions <= -1 || max_exceptions == i32::MAX) {
        return false;
    }
    let mut name = name;
    let mut index = index_of_any(name, exceptions);
    let mut exception_count = 0;
    while let Some(i) = index {
        if no_exceptions_at_start {
            if i == 0 {
                return true;
            }
            no_exceptions_at_start = false;
        }
        let was = exception_count;
        exception_count += 1;
        if was >= max_exceptions {
            return true;
        }
        if i + 1 < name.len() {
            name = &name[i + 1..];
            index = index_of_any(name, exceptions);
            if index != Some(0) {
                exception_count = 0;
            }
        } else {
            index = None;
        }
    }
    false
}

/// `ContainsDisallowedWord`: a disallowed word bounded by non-alphanumerics on both sides.
pub fn contains_disallowed_word(name: &[char], disallowed: &[&str]) -> bool {
    // `name` is sliced forward, so "the char before" is always inside the full name.
    let full = name;
    let mut offset = 0;
    let mut index = index_of_any_word(full, disallowed);
    while let Some(rel) = index {
        let i = offset + rel;
        let is_start_boundary = i == 0 || !muo_primitives::dotnet::is_letter_or_digit(full[i - 1]);
        if is_start_boundary {
            for word in disallowed {
                let n = word.chars().count();
                if i + n > full.len() || !equals_ignore_case(&full[i..i + n], word) {
                    continue;
                }
                // End boundary
                if i + n == full.len() || !muo_primitives::dotnet::is_letter_or_digit(full[i + n]) {
                    return true;
                }
            }
        }
        if i + 1 < full.len() {
            offset = i + 1;
            index = index_of_any_word(&full[offset..], disallowed);
        } else {
            index = None;
        }
    }
    false
}

/// `NameVerification.Configure`: `[ValidateName]`.
pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "ValidateName",
        AccessLevel::Administrator,
        &[],
        Rc::new(|w: &mut World, e: &mut CommandEventArgs| {
            // Usage: ValidateName; Description: Checks the result of NameValidation on the specified name.
            if validate(
                &e.arg_string,
                2,
                16,
                true,
                false,
                true,
                1,
                Some(SPACE_DASH_PERIOD_QUOTE),
            ) {
                messages::send_message_hue(w, e.mobile, 0x59, "That name is considered valid.");
            } else {
                messages::send_message_hue(w, e.mobile, 0x22, "That name is considered invalid.");
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_names() {
        assert!(
            validate_player_name("Bob Smithy"),
            "smith only counts as a whole word"
        );
        assert!(validate_player_name("Bobsmith"));
        assert!(
            !validate_player_name("Bob Smith"),
            "a disallowed word between boundaries"
        );
        assert!(validate_player_name("Al-Bob"));
        assert!(!validate_player_name("-Bob"), "no exceptions at the start");
        assert!(
            validate_player_name("A B C"),
            "separated exceptions don't add up"
        );
        assert!(
            !validate_player_name("A  B"),
            "two in a row exceed the maximum of 1"
        );
        assert!(
            !validate_player_name("GM Bob"),
            "starts with a reserved word"
        );
        assert!(!validate_player_name("B"), "too short");
        assert!(!validate_player_name("Bob1"), "digits are not allowed");
    }

    #[test]
    fn pet_and_vendor_names() {
        assert!(validate_pet_name("Rex"));
        assert!(!validate_pet_name("Re x"));
        assert!(validate_vendor_name("Bob2"));
        assert!(!validate_vendor_name("fuck"));
    }

    const EXC: &[char] = &[' ', '-', '.'];
    fn v(name: &str, min: i32, max: i32, letters: bool, digits: bool) -> bool {
        validate(name, min, max, letters, digits, true, 0, None)
    }

    #[test]
    fn validate_player_name_valid_names_return_true() {
        for n in ["John", "Mary Ann", "Bob-Jones", "O'Malley"] {
            assert!(validate_player_name(n), "{n}");
        }
    }

    #[test]
    fn validate_pet_name_invalid_names_return_false() {
        for n in ["Mrs-Smith", "Mr. Whiskers", "Dog123"] {
            assert!(!validate_pet_name(n), "{n}");
        }
    }

    #[test]
    fn validate_vendor_name_valid_names_return_true() {
        for n in ["Blacksmith12", "Baker", "Innkeeper"] {
            assert!(validate_vendor_name(n), "{n}");
        }
    }

    #[test]
    fn validate_vendor_name_disallowed_characters_return_false() {
        for n in ["Blacksmith 123", "Baker-Smith", "*Innkeeper*"] {
            assert!(!validate_vendor_name(n), "{n}");
        }
    }

    #[test]
    fn validate_length_bounds_and_exceptions() {
        assert!(v("Ab", 2, 16, true, false));
        assert!(validate(
            "AbcdefghijklmnopqrsT",
            1,
            20,
            true,
            true,
            true,
            0,
            None
        ));
        assert!(validate("A-B.C D", 2, 16, true, false, false, 3, Some(EXC)));
        // "ass" is disallowed, but "class" has proper boundaries
        assert!(validate_player_name("Class"));
        assert!(!validate("", 1, 20, true, true, true, 0, None));
        assert!(!v("A", 2, 16, true, false));
        assert!(!v("AbcdefghijklmnopqrstuvwxyzABCDEF", 2, 16, true, false));
        assert!(!validate(
            "A-B.C  D-E",
            2,
            16,
            true,
            false,
            false,
            1,
            Some(EXC)
        ));
        assert!(!validate("-John", 2, 16, true, false, true, 1, Some(EXC)));
    }

    #[test]
    fn validate_disallowed_words_return_false() {
        for n in ["ass", "GodDamn Fine", "Fuck"] {
            assert!(!validate_player_name(n), "{n}");
        }
    }

    #[test]
    fn validate_disallowed_prefixes_return_false() {
        for n in ["GMJohn", "LordBob", "SeerMagic"] {
            assert!(!validate_player_name(n), "{n}");
        }
    }
}
