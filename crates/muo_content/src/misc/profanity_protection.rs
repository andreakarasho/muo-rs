//! Profanity protection: what happens to speech with a disallowed word.

use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;

use super::name_verification;

/// `ProfanityAction`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ProfanityAction {
    /// no action taken
    None,
    /// speech is not displayed
    #[default]
    Disallow,
    /// makes the player criminal, not killable by guards
    Criminal,
    /// makes the player criminal, can be killed by guards
    CriminalAction,
    /// player is kicked
    Disconnect,
    /// some other implementation
    Other,
}

impl ProfanityAction {
    /// `Enum.TryParse` of the configuration text: a case-sensitive name or a number.
    pub fn parse(s: &str) -> Option<ProfanityAction> {
        use ProfanityAction::*;
        let all = [None, Disallow, Criminal, CriminalAction, Disconnect, Other];
        let s = s.trim();
        let by_name = all.iter().copied().find(|a| format!("{a:?}") == s);
        by_name.or_else(|| s.parse::<usize>().ok().and_then(|i| all.get(i).copied()))
    }
}

/// `Exceptions`: "Used by the guild system".
pub const EXCEPTIONS: &[char] = &[
    ' ', '-', '.', '\'', '"', ',', '_', '+', '=', '~', '`', '!', '^', '*', '\\', '/', ';', ':',
    '<', '>', '[', ']', '{', '}', '?', '|', '(', ')', '%', '$', '&', '#', '@',
];

pub const DISALLOWED: &[&str] = &[
    "jigaboo",
    "chigaboo",
    "wop",
    "kyke",
    "kike",
    "tit",
    "spic",
    "prick",
    "piss",
    "lezbo",
    "lesbo",
    "felatio",
    "dyke",
    "dildo",
    "chinc",
    "chink",
    "cunnilingus",
    "cum",
    "cocksucker",
    "cock",
    "clitoris",
    "clit",
    "ass",
    "hitler",
    "penis",
    "nigga",
    "nigger",
    "klit",
    "kunt",
    "jiz",
    "jism",
    "jerkoff",
    "jackoff",
    "goddamn",
    "fag",
    "blowjob",
    "bitch",
    "asshole",
    "dick",
    "pussy",
    "snatch",
    "cunt",
    "twat",
    "shit",
    "fuck",
];

/// `Enabled` and `Action`, read by `Configure`.
#[derive(Default)]
struct Settings {
    enabled: bool,
    action: ProfanityAction,
}

/// `ProfanityProtection.Configure`.
pub fn configure(w: &mut World) {
    let enabled = w.config.get_setting("profanityProtection.enabled", false);
    let action = w
        .config
        .get_setting_opt::<String>("profanityProtection.action")
        .and_then(|s| ProfanityAction::parse(&s))
        .unwrap_or(ProfanityAction::Disallow);
    w.resources.get_mut::<Settings>().enabled = enabled;
    w.resources.get_mut::<Settings>().action = action;
    if enabled {
        w.events.speech.subscribe(on_speech);
    }
}

/// `OnProfanityDetected`: whether the speech may go on.
fn on_profanity_detected(w: &mut World, from: MobileId) -> bool {
    let action = w
        .resources
        .get::<Settings>()
        .map_or(ProfanityAction::Disallow, |s| s.action);
    match action {
        ProfanityAction::None => true,
        ProfanityAction::Disallow => false,
        ProfanityAction::Criminal => {
            if let Some(ty) = w.mobile(from).map(|m| m.ty) {
                (ty.set_criminal)(w, from, true);
            }
            true
        }
        ProfanityAction::CriminalAction => {
            muo_server::mobile::base::criminal_action(w, from, false);
            true
        }
        ProfanityAction::Disconnect => {
            if let Some(ns) = w.mobile(from).and_then(|m| m.net_state) {
                w.disconnect(ns, "Using profanity.");
            }
            false
        }
        // TODO: provide a custom implementation if this is chosen
        ProfanityAction::Other => true,
    }
}

/// `span.IndexOfAny(DisallowedSearchValues) != -1`.
pub fn disallowed_in(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    name_verification::index_of_any_word(&chars, DISALLOWED).is_some()
}

/// `ContainsProfanity(speech)`.
pub fn contains_profanity(speech: &str) -> bool {
    !speech.is_empty()
        && !name_verification::validate_with(
            speech,
            1,
            i32::MAX,
            true,
            true,
            true,
            i32::MAX, // allow all non-alphanumeric characters
            None,
            DISALLOWED,
            None,
        )
}

fn on_speech(w: &mut World, e: &mut muo_server::events::SpeechEventArgs) {
    let from = e.mobile;
    if w.mobile(from)
        .is_none_or(|m| m.access_level > AccessLevel::Player)
    {
        return;
    }
    if contains_profanity(&e.speech) {
        e.blocked = !on_profanity_detected(w, from);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_whole_words_only() {
        assert!(contains_profanity("what the fuck"));
        assert!(contains_profanity("SHIT!"));
        assert!(!contains_profanity("assassin"));
        assert!(!contains_profanity("class"));
        assert!(!contains_profanity(""));
    }

    #[test]
    fn action_parse() {
        assert_eq!(
            ProfanityAction::parse("Criminal"),
            Some(ProfanityAction::Criminal)
        );
        assert_eq!(
            ProfanityAction::parse("4"),
            Some(ProfanityAction::Disconnect)
        );
        assert_eq!(ProfanityAction::parse("x"), None);
    }

    #[test]
    fn speech_without_profanity_passes_validation() {
        for s in [
            "Hello world",
            "This is a normal conversation",
            "I would like to trade with you",
            "",
        ] {
            assert!(!contains_profanity(s), "{s}");
        }
    }

    #[test]
    fn speech_with_words_that_look_like_profanity_passes_validation() {
        for s in [
            "I'm going to class tomorrow",
            "This assignment is hard",
            "That's a nice cocktail",
        ] {
            assert!(!contains_profanity(s), "{s}");
        }
    }

    #[test]
    fn speech_with_profanity_fails_validation() {
        for s in ["This is ass", "What the fuck", "You're a bitch"] {
            assert!(contains_profanity(s), "{s}");
        }
    }

    #[test]
    fn contains_disallowed_word_detects_profanity_with_boundaries() {
        for s in ["ass", "an ass joke", "ass."] {
            let chars: Vec<char> = s.chars().collect();
            assert!(
                name_verification::contains_disallowed_word(&chars, DISALLOWED),
                "{s}"
            );
        }
    }
}
