//! Inhuman speech: the gibberish and the few real words that ratmen, orcs, lizardmen and wisps say.
//! The four speech sets are data ([`RATMAN`], [`ORC`], [`LIZARDMAN`], [`WISP`]); the functions take the set the
//! creature's class names.

use muo_primitives::{Point3D, utility};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::{effects, messages};

/// `IHSFlags`.
pub const ON_DAMAGED: i32 = 0x01;
pub const ON_DEATH: i32 = 0x02;
pub const ON_MOVEMENT: i32 = 0x04;
pub const ON_SPEECH: i32 = 0x08;
/// `IHSFlags.All` (`OnSpeech` is left out: "To enable monster conversations, add `| OnSpeech`").
pub const ALL: i32 = ON_DAMAGED | ON_DEATH | ON_MOVEMENT;

/// One `InhumanSpeech`.
pub struct InhumanSpeech {
    pub hue: i32,
    pub sound: i32,
    pub flags: i32,
    pub keywords: &'static [&'static str],
    pub responses: &'static [&'static str],
    pub syllables: &'static [&'static str],
}

/// `InhumanSpeech.Ratman`.
pub static RATMAN: InhumanSpeech = InhumanSpeech {
    hue: 149,
    sound: 438,
    flags: 7,
    keywords: &[
        "meat",
        "gold",
        "kill",
        "killing",
        "slay",
        "sword",
        "axe",
        "spell",
        "magic",
        "spells",
        "swords",
        "axes",
        "mace",
        "maces",
        "monster",
        "monsters",
        "food",
        "run",
        "escape",
        "away",
        "help",
        "dead",
        "die",
        "dying",
        "lose",
        "losing",
        "life",
        "lives",
        "death",
        "ghost",
        "ghosts",
        "british",
        "blackthorn",
        "guild",
        "guilds",
        "dragon",
        "dragons",
        "game",
        "games",
        "ultima",
        "silly",
        "stupid",
        "dumb",
        "idiot",
        "idiots",
        "cheesy",
        "cheezy",
        "crazy",
        "dork",
        "jerk",
        "fool",
        "foolish",
        "ugly",
        "insult",
        "scum",
    ],
    responses: &[
        "meat",
        "kill",
        "pound",
        "crush",
        "yum yum",
        "crunch",
        "destroy",
        "murder",
        "eat",
        "munch",
        "massacre",
        "food",
        "monster",
        "evil",
        "run",
        "die",
        "lose",
        "dumb",
        "idiot",
        "fool",
        "crazy",
        "dinner",
        "lunch",
        "breakfast",
        "fight",
        "battle",
        "doomed",
        "rip apart",
        "tear apart",
        "smash",
        "edible?",
        "shred",
        "disembowel",
        "ugly",
        "smelly",
        "stupid",
        "hideous",
        "smell",
        "tasty",
        "invader",
        "attack",
        "raid",
        "plunder",
        "pillage",
        "treasure",
        "loser",
        "lose",
        "scum",
    ],
    syllables: &[
        "skrit", "ch", "ch", "it", "ti", "it", "ti", "ak", "ek", "ik", "ok", "uk", "yk", "ka",
        "ke", "ki", "ko", "ku", "ky", "at", "et", "it", "ot", "ut", "yt", "cha", "che", "chi",
        "cho", "chu", "chy", "ach", "ech", "ich", "och", "uch", "ych", "att", "ett", "itt", "ott",
        "utt", "ytt", "tat", "tet", "tit", "tot", "tut", "tyt", "tta", "tte", "tti", "tto", "ttu",
        "tty", "tak", "tek", "tik", "tok", "tuk", "tyk", "ack", "eck", "ick", "ock", "uck", "yck",
        "cka", "cke", "cki", "cko", "cku", "cky", "rak", "rek", "rik", "rok", "ruk", "ryk", "tcha",
        "tche", "tchi", "tcho", "tchu", "tchy", "rach", "rech", "rich", "roch", "ruch", "rych",
        "rrap", "rrep", "rrip", "rrop", "rrup", "rryp", "ccka", "ccke", "ccki", "ccko", "ccku",
        "ccky",
    ],
};

/// `InhumanSpeech.Orc`.
pub static ORC: InhumanSpeech = InhumanSpeech {
    hue: 34,
    sound: 432,
    flags: 7,
    keywords: &[
        "meat",
        "gold",
        "kill",
        "killing",
        "slay",
        "sword",
        "axe",
        "spell",
        "magic",
        "spells",
        "swords",
        "axes",
        "mace",
        "maces",
        "monster",
        "monsters",
        "food",
        "run",
        "escape",
        "away",
        "help",
        "dead",
        "die",
        "dying",
        "lose",
        "losing",
        "life",
        "lives",
        "death",
        "ghost",
        "ghosts",
        "british",
        "blackthorn",
        "guild",
        "guilds",
        "dragon",
        "dragons",
        "game",
        "games",
        "ultima",
        "silly",
        "stupid",
        "dumb",
        "idiot",
        "idiots",
        "cheesy",
        "cheezy",
        "crazy",
        "dork",
        "jerk",
        "fool",
        "foolish",
        "ugly",
        "insult",
        "scum",
    ],
    responses: &[
        "meat",
        "kill",
        "pound",
        "crush",
        "yum yum",
        "crunch",
        "destroy",
        "murder",
        "eat",
        "munch",
        "massacre",
        "food",
        "monster",
        "evil",
        "run",
        "die",
        "lose",
        "dumb",
        "idiot",
        "fool",
        "crazy",
        "dinner",
        "lunch",
        "breakfast",
        "fight",
        "battle",
        "doomed",
        "rip apart",
        "tear apart",
        "smash",
        "edible?",
        "shred",
        "disembowel",
        "ugly",
        "smelly",
        "stupid",
        "hideous",
        "smell",
        "tasty",
        "invader",
        "attack",
        "raid",
        "plunder",
        "pillage",
        "treasure",
        "loser",
        "lose",
        "scum",
    ],
    syllables: &[
        "bu", "du", "fu", "ju", "gu", "ulg", "gug", "gub", "gur", "oog", "gub", "log", "ru", "stu",
        "glu", "ug", "ud", "og", "log", "ro", "flu", "bo", "duf", "fun", "nog", "dun", "bog",
        "dug", "gh", "ghu", "gho", "nug", "ig", "igh", "ihg", "luh", "duh", "bug", "dug", "dru",
        "urd", "gurt", "grut", "grunt", "snarf", "urgle", "igg", "glu", "glug", "foo", "bar",
        "baz", "ghat", "ab", "ad", "gugh", "guk", "ag", "alm", "thu", "log", "bilge", "augh",
        "gha", "gig", "goth", "zug", "pig", "auh", "gan", "azh", "bag", "hig", "oth", "dagh",
        "gulg", "ugh", "ba", "bid", "gug", "bug", "rug", "hat", "brui", "gagh", "buad", "buil",
        "buim", "bum", "hug", "hug", "buo", "ma", "buor", "ghed", "buu", "ca", "guk", "clog",
        "thurg", "car", "cro", "thu", "da", "cuk", "gil", "cur", "dak", "dar", "deak", "der",
        "dil", "dit", "at", "ag", "dor", "gar", "dre", "tk", "dri", "gka", "rim", "eag", "egg",
        "ha", "rod", "eg", "lat", "eichel", "ek", "ep", "ka", "it", "ut", "ewk", "ba", "dagh",
        "faugh", "foz", "fog", "fid", "fruk", "gag", "fub", "fud", "fur", "bog", "fup", "hagh",
        "gaa", "kt", "rekk", "lub", "lug", "tug", "gna", "urg", "l", "gno", "gnu", "gol", "gom",
        "kug", "ukk", "jak", "jek", "rukk", "jja", "akt", "nuk", "hok", "hrol", "olm", "natz", "i",
        "i", "o", "u", "ikk", "ign", "juk", "kh", "kgh", "ka", "hig", "ke", "ki", "klap", "klu",
        "knod", "kod", "knu", "thnu", "krug", "nug", "nar", "nag", "neg", "neh", "oag", "ob",
        "ogh", "oh", "om", "dud", "oo", "pa", "hrak", "qo", "quad", "quil", "ghig", "rur", "sag",
        "sah", "sg",
    ],
};

/// `InhumanSpeech.Lizardman`.
pub static LIZARDMAN: InhumanSpeech = InhumanSpeech {
    hue: 58,
    sound: 418,
    flags: 7,
    keywords: &[
        "meat",
        "gold",
        "kill",
        "killing",
        "slay",
        "sword",
        "axe",
        "spell",
        "magic",
        "spells",
        "swords",
        "axes",
        "mace",
        "maces",
        "monster",
        "monsters",
        "food",
        "run",
        "escape",
        "away",
        "help",
        "dead",
        "die",
        "dying",
        "lose",
        "losing",
        "life",
        "lives",
        "death",
        "ghost",
        "ghosts",
        "british",
        "blackthorn",
        "guild",
        "guilds",
        "dragon",
        "dragons",
        "game",
        "games",
        "ultima",
        "silly",
        "stupid",
        "dumb",
        "idiot",
        "idiots",
        "cheesy",
        "cheezy",
        "crazy",
        "dork",
        "jerk",
        "fool",
        "foolish",
        "ugly",
        "insult",
        "scum",
    ],
    responses: &[
        "meat",
        "kill",
        "pound",
        "crush",
        "yum yum",
        "crunch",
        "destroy",
        "murder",
        "eat",
        "munch",
        "massacre",
        "food",
        "monster",
        "evil",
        "run",
        "die",
        "lose",
        "dumb",
        "idiot",
        "fool",
        "crazy",
        "dinner",
        "lunch",
        "breakfast",
        "fight",
        "battle",
        "doomed",
        "rip apart",
        "tear apart",
        "smash",
        "edible?",
        "shred",
        "disembowel",
        "ugly",
        "smelly",
        "stupid",
        "hideous",
        "smell",
        "tasty",
        "invader",
        "attack",
        "raid",
        "plunder",
        "pillage",
        "treasure",
        "loser",
        "lose",
        "scum",
    ],
    syllables: &[
        "ss", "sth", "iss", "is", "ith", "kth", "sith", "this", "its", "sit", "tis", "tsi", "ssi",
        "sil", "lis", "sis", "lil", "thil", "lith", "sthi", "lish", "shi", "shash", "sal", "miss",
        "ra", "tha", "thes", "ses", "sas", "las", "les", "sath", "sia", "ais", "isa", "asi",
        "asth", "stha", "sthi", "isth", "asa", "ath", "tha", "als", "sla", "thth", "ci", "ce",
        "cy", "yss", "ys", "yth", "syth", "thys", "yts", "syt", "tys", "tsy", "ssy", "syl", "lys",
        "sys", "lyl", "thyl", "lyth", "sthy", "lysh", "shy", "myss", "ysa", "sthy", "ysth",
    ],
};

/// `InhumanSpeech.Wisp`.
pub static WISP: InhumanSpeech = InhumanSpeech {
    hue: 89,
    sound: 466,
    flags: 4,
    keywords: &[],
    responses: &[],
    syllables: &[
        "b", "c", "d", "f", "g", "h", "i", "j", "k", "l", "m", "n", "p", "r", "s", "t", "v", "w",
        "x", "z", "c", "c", "x", "x", "x", "x", "x", "y", "y", "y", "y", "t", "t", "k", "k", "l",
        "l", "m", "m", "m", "m", "z",
    ],
};

fn upper_first(s: &mut String) {
    if let Some(c) = s.chars().next() {
        let up: String = c.to_uppercase().collect();
        s.replace_range(..c.len_utf8(), &up);
    }
}

impl InhumanSpeech {
    /// `GetRandomSyllable`.
    pub fn random_syllable(&self, w: &mut World) -> &'static str {
        self.syllables[utility::random(&mut w.rng, self.syllables.len() as i32) as usize]
    }

    /// `ConstructWord(syllableCount)`.
    pub fn construct_word(&self, w: &mut World, syllable_count: i32) -> String {
        let mut word = String::new();
        for _ in 0..syllable_count {
            word.push_str(self.random_syllable(w));
        }
        word
    }

    /// `ConstructSentence(wordCount)`.
    pub fn construct_sentence(&self, w: &mut World, word_count: i32) -> String {
        let mut sentence = String::new();
        let mut need_upper_case = true;
        for i in 0..word_count {
            if i > 0 {
                // not first word
                let random = utility::random_min_max(&mut w.rng, 1, 15);
                if random < 11 {
                    sentence.push(' ');
                } else {
                    need_upper_case = true;
                    sentence.push_str(if random > 13 { "! " } else { ". " });
                }
            }
            let count = if utility::random(&mut w.rng, 100) < 30 {
                5
            } else {
                3
            };
            let syllable_count = utility::random_from(&mut w.rng, 1, count);
            let mut word = self.construct_word(w, syllable_count);
            if need_upper_case {
                upper_first(&mut word);
            }
            sentence.push_str(&word);
            need_upper_case = false;
        }
        sentence.push(if utility::random_min_max(&mut w.rng, 1, 5) == 1 {
            '!'
        } else {
            '.'
        });
        sentence
    }

    /// `SaySentance(mob, wordCount)`.
    pub fn say_sentence(&self, w: &mut World, mob: MobileId, word_count: i32) {
        let s = self.construct_sentence(w, word_count);
        messages::say_text(w, mob, &s);
        effects::play_sound(w, mob, self.sound);
    }

    /// `SayRandomTranslate(mob, sentencesInEnglish)`.
    pub fn say_random_translate(&self, w: &mut World, mob: MobileId, sentences: &[&str]) {
        let words = utility::random_min_max(&mut w.rng, 2, 3);
        self.say_sentence(w, mob, words);
        let pick = utility::random(&mut w.rng, sentences.len() as i32) as usize;
        messages::say_text(w, mob, sentences[pick]);
    }

    /// `GetRandomResponseWord(keywordsFound)`.
    fn random_response_word(&self, w: &mut World, found: &[&'static str]) -> &'static str {
        let random =
            utility::random(&mut w.rng, (found.len() + self.responses.len()) as i32) as usize;
        if random < found.len() {
            found[random]
        } else {
            self.responses[random - found.len()]
        }
    }

    /// `OnSpeech(mob, speaker, text)`: whether the creature answered.
    pub fn on_speech(&self, w: &mut World, mob: MobileId, speaker: MobileId, text: &str) -> bool {
        if self.flags & ON_SPEECH == 0 || self.keywords.is_empty() || self.responses.is_empty() {
            return false; // not enabled
        }
        let (Some(s), Some(m)) = (w.mobile(speaker), w.mobile(mob)) else {
            return false;
        };
        if !(s.ty.alive)(w, speaker) {
            return false;
        }
        let (sloc, mloc, sdir, mdir) = (s.location, m.location, s.direction, m.direction);
        if !muo_server::mobile::base::in_range_point3d(w, speaker, mloc, 3) {
            return false;
        }
        if sdir.facing() != effects::get_direction_to(w, speaker, mloc.x, mloc.y, false) {
            return false;
        }
        if mdir.facing() != effects::get_direction_to(w, mob, sloc.x, sloc.y, false) {
            return false;
        }

        let mut found: Vec<&'static str> = Vec::new();
        for word in text.split(' ') {
            let lower = word.to_lowercase();
            if let Some(k) = self.keywords.iter().find(|k| **k == lower) {
                found.push(k);
            }
        }
        if found.is_empty() {
            return false;
        }
        let response_word = if utility::random_bool(&mut w.rng) {
            self.random_response_word(w, &found)
        } else {
            found[utility::random(&mut w.rng, found.len() as i32) as usize]
        };
        let second = self.random_response_word(w, &found);
        let (mut a, mut b) = (response_word.to_owned(), second.to_owned());
        upper_first(&mut a);
        upper_first(&mut b);
        let response = match utility::random(&mut w.rng, 6) {
            1 => format!("{a} thee!"),
            2 => format!("{a}?"),
            3 => format!("{a}! {b}."),
            4 => format!("{a}."),
            5 => format!("{a}? {b}."),
            _ => format!("Me {response_word}?"),
        };
        let split_len = text.split(' ').count() as i32;
        let max_words = (split_len / 2 + 1).clamp(2, 6);
        let words = utility::random_min_max(&mut w.rng, 2, max_words);
        self.say_sentence(w, mob, words);
        messages::say_text(w, mob, &response);
        true
    }

    /// `OnDeath(mob)`.
    pub fn on_death(&self, w: &mut World, mob: MobileId) {
        if self.flags & ON_DEATH == 0 {
            return; // not enabled
        }
        if utility::random(&mut w.rng, 100) < 90 {
            return; // 90% chance to do nothing; 10% chance to talk
        }
        self.say_random_translate(
            w,
            mob,
            &[
                "Revenge!",
                "NOOooo!",
                "I... I...",
                "Me no die!",
                "Me die!",
                "Must... not die...",
                "Oooh, me hurt...",
                "Me dying?",
            ],
        );
    }

    /// `OnMovement(mob, mover, oldLocation)`.
    pub fn on_movement(
        &self,
        w: &mut World,
        mob: MobileId,
        mover: MobileId,
        old_location: Point3D,
    ) {
        if self.flags & ON_MOVEMENT == 0 {
            return; // not enabled
        }
        let Some(mv) = w.mobile(mover) else { return };
        if !mv.player || (mv.hidden && mv.access_level > AccessLevel::Player) {
            return;
        }
        let mover_loc = mv.location;
        let in_range =
            |w: &World, p: Point3D| muo_server::mobile::base::in_range_point3d(w, mob, p, 5);
        if !in_range(w, mover_loc) || in_range(w, old_location) {
            return; // only talk when they enter 5 tile range
        }
        if utility::random(&mut w.rng, 100) < 90 {
            return; // 90% chance to do nothing; 10% chance to talk
        }
        self.say_sentence(w, mob, 6);
    }

    /// `OnDamage(mob, amount)`.
    pub fn on_damage(&self, w: &mut World, mob: MobileId, amount: i32) {
        if self.flags & ON_DAMAGED == 0 {
            return; // not enabled
        }
        if utility::random(&mut w.rng, 100) < 90 {
            return; // 90% chance to do nothing; 10% chance to talk
        }
        if amount < 5 {
            self.say_random_translate(
                w,
                mob,
                &[
                    "Ouch!",
                    "Me not hurt bad!",
                    "Thou fight bad.",
                    "Thy blows soft!",
                    "You bad with weapon!",
                ],
            );
        } else {
            self.say_random_translate(
                w,
                mob,
                &[
                    "Ouch! Me hurt!",
                    "No, kill me not!",
                    "Me hurt!",
                    "Away with thee!",
                    "Oof! That hurt!",
                    "Aaah! That hurt...",
                    "Good blow!",
                ],
            );
        }
    }

    /// `OnConstruct(mob)`.
    pub fn on_construct(&self, w: &mut World, mob: MobileId) {
        if let Some(m) = w.mobile_mut(mob) {
            m.speech_hue = self.hue;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentences_start_upper_case_and_end_in_punctuation() {
        let mut w = World::new();
        for count in 1..8 {
            let s = RATMAN.construct_sentence(&mut w, count);
            assert!(s.chars().next().unwrap().is_uppercase(), "{s}");
            assert!(s.ends_with('.') || s.ends_with('!'), "{s}");
        }
    }

    #[test]
    fn the_sets_have_the_expected_flags() {
        assert_eq!((RATMAN.hue, RATMAN.sound, RATMAN.flags), (149, 438, 7));
        assert_eq!(
            (WISP.hue, WISP.flags, WISP.keywords.len()),
            (89, ON_MOVEMENT, 0)
        );
        assert_eq!(RATMAN.keywords.len(), 55);
    }
}
