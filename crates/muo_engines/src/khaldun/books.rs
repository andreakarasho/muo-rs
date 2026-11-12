//! The Khaldun books (GrimmochJournal, LysanderNotebook, TavarasJournal) with a
//! fixed `BookContent` and `base(Utility.Random(0xFF1, 2), false)`.

use muo_items::books::{BASE_BOOK, BookClass, BookContent, new_book};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ItemType;
use muo_server::type_registry::TypeRegistry;

/// `GrimmochJournal1.Content`.
pub static GRIMMOCH_JOURNAL1_CONTENT: BookContent = BookContent {
    title: "The daily journal of Grimmoch Drummel",
    author: "Grimmoch",
    pages: &[
        &[
            "Day One :",
            "",
            "'Tis a grand sight, this",
            "primeval tomb, I agree",
            "with Tavara on that.",
            "And we've a good crew",
            "here, they've strong",
            "backs and a good",
        ],
        &[
            "attitude.  I'm a bit",
            "concerned by those",
            "that worked as guides",
            "for us, however.  All",
            "seemed well enough",
            "until we revealed the",
            "immense stone doors",
            "of the tomb structure",
        ],
        &[
            "itself.  Seemed to send",
            "a shiver up their",
            "spines and get them all",
            "stirred up with",
            "whispering.  I'll",
            "watch the lot of them",
            "with a close eye, but",
            "I'm confident we won't",
        ],
        &[
            "have any real",
            "problems on the dig.",
            "I'm especially proud to",
            "see Thomas standing",
            "out - he was a good",
            "hire, despite the",
            "warnings from his",
            "previous employers.",
        ],
        &[
            "He's drummed up the",
            "workers into a",
            "furious pace - we've",
            "nearly halved the",
            "estimate on the",
            "timeline for",
            "excavating the Tomb's",
            "entrance.",
        ],
    ],
};

static GRIMMOCH_JOURNAL1_CLASS: BookClass = BookClass {
    default_content: Some(&GRIMMOCH_JOURNAL1_CONTENT),
};

/// `GrimmochJournal1()`.
pub fn grimmoch_journal1_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &GRIMMOCH_JOURNAL1, graphic, None, None, 0, false)
}

/// `Server.Items.GrimmochJournal1` `[SerializationGenerator(0)]`.
pub static GRIMMOCH_JOURNAL1: ItemType = ItemType {
    full_name: "Server.Items.GrimmochJournal1",
    parent: Some(&BASE_BOOK),
    construct: Some(grimmoch_journal1_new),
    family: Some(&GRIMMOCH_JOURNAL1_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `GrimmochJournal2.Content`.
pub static GRIMMOCH_JOURNAL2_CONTENT: BookContent = BookContent {
    title: "The daily journal of Grimmoch Drummel",
    author: "Grimmoch",
    pages: &[
        &[
            "Day Two :",
            "",
            "We managed to dig out",
            "the last of the",
            "remaining rubble",
            "today, revealing the",
            "entirety of the giant",
            "stone doors that sealed",
        ],
        &[
            "ol' Khal Ankur and",
            "his folk up ages ago.",
            "Actually getting them",
            "open was another",
            "matter altogether,",
            "however.  As the",
            "workers set to the",
            "task with picks and",
        ],
        &[
            "crowbars, I could have",
            "sworn I saw Lysander",
            "Gathenwale fiddling",
            "with something in that",
            "musty old tome of his.",
            " I've no great",
            "knowledge of things",
            "magical, but the way",
        ],
        &[
            "his hand moved over",
            "that book, and the look",
            "of concentration on his",
            "face as he whispered",
            "something to himself",
            "looked like every",
            "description of an",
            "incantation I've ever",
        ],
        &[
            "heard.  The strange",
            "thing is, this set of",
            "doors that an entire",
            "crew of excavators",
            "was laboring over for",
            "hours, right when",
            "Gathenwale finishes",
            "with his mumbling...",
        ],
        &[
            "well, I swore the doors",
            "just gave open at the",
            "exact moment he",
            "spoke his last bit of",
            "whisper and shut the",
            "tome tight in his",
            "hands.  When he",
            "looked up, it was",
        ],
        &[
            "almost as if he was",
            "expecting the doors to",
            "be open, rather than",
            "shocked that they'd",
            "finally given way.",
        ],
    ],
};

static GRIMMOCH_JOURNAL2_CLASS: BookClass = BookClass {
    default_content: Some(&GRIMMOCH_JOURNAL2_CONTENT),
};

/// `GrimmochJournal2()`.
pub fn grimmoch_journal2_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &GRIMMOCH_JOURNAL2, graphic, None, None, 0, false)
}

/// `Server.Items.GrimmochJournal2` `[SerializationGenerator(0)]`.
pub static GRIMMOCH_JOURNAL2: ItemType = ItemType {
    full_name: "Server.Items.GrimmochJournal2",
    parent: Some(&BASE_BOOK),
    construct: Some(grimmoch_journal2_new),
    family: Some(&GRIMMOCH_JOURNAL2_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `GrimmochJournal3.Content`.
pub static GRIMMOCH_JOURNAL3_CONTENT: BookContent = BookContent {
    title: "The daily journal of Grimmoch Drummel",
    author: "Grimmoch",
    pages: &[
        &[
            "Day Three - Day Five:",
            "",
            "I might have",
            "written too hastily in",
            "my first entry - this",
            "place doesn't seem too",
            "bent on giving up any",
            "secrets.   Though the",
        ],
        &[
            "main antechamber is",
            "open to us, the main",
            "exit hall is blocked by",
            "yet another pile of",
            "rubble.  Doesn't look a",
            "bit like anything",
            "caused by a quake or",
            "instability in the",
        ],
        &[
            "stonework... I swear it",
            "looks as if someone",
            "actually piled the",
            "stones up themselves,",
            "some time after the",
            "tomb was built.  The",
            "stones aren't of the",
            "same set nor quality",
        ],
        &[
            "of the carved work",
            "that surrounds them",
            "- if anything, they",
            "resemble the grade of",
            "common rock we saw",
            "in great quantities on",
            "the trip here.  Which",
            "makes it feel all the",
        ],
        &[
            "more like someone",
            "hauled them in and",
            "deliberately covered",
            "this passage.  But then",
            "why not decorate them",
            "in the same ornate",
            "manner as the rest of",
            "the stone in this",
        ],
        &[
            "place?  Lysander",
            "wouldn't hear a word",
            "of what I had to say -",
            "to him, it was a quake",
            "some time in the",
            "history of the tomb,",
            "and that was it, shut",
            "up and move on.  So I",
        ],
        &["shut up, and got back", "to work."],
    ],
};

static GRIMMOCH_JOURNAL3_CLASS: BookClass = BookClass {
    default_content: Some(&GRIMMOCH_JOURNAL3_CONTENT),
};

/// `GrimmochJournal3()`.
pub fn grimmoch_journal3_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &GRIMMOCH_JOURNAL3, graphic, None, None, 0, false)
}

/// `Server.Items.GrimmochJournal3` `[SerializationGenerator(0)]`.
pub static GRIMMOCH_JOURNAL3: ItemType = ItemType {
    full_name: "Server.Items.GrimmochJournal3",
    parent: Some(&BASE_BOOK),
    construct: Some(grimmoch_journal3_new),
    family: Some(&GRIMMOCH_JOURNAL3_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `GrimmochJournal6.Content`.
pub static GRIMMOCH_JOURNAL6_CONTENT: BookContent = BookContent {
    title: "The daily journal of Grimmoch Drummel",
    author: "Grimmoch",
    pages: &[
        &[
            "Day Six :",
            "",
            "The camp was",
            "attacked last night by",
            "a pack of, well, I don't",
            "have a clue.  I've never",
            "seen the like of these",
            "beasts anywhere.",
        ],
        &[
            "Huge things, with",
            "fangs the size of your",
            "forefinger, covered in",
            "hair and with the",
            "strangest arched back",
            "I've ever seen.  And so",
            "many of them.  We",
            "were forced back into",
        ],
        &[
            "the Tomb for the",
            "night, just to keep our",
            "hides on us.  And",
            "today Gathenwale",
            "practically orders us",
            "all to move the entire",
            "exterior camp into the",
            "Tomb.  Now, I don't",
        ],
        &[
            "disagree that we'd be",
            "well off to use the",
            "place as a point of",
            "fortification... but I",
            "don't like it one bit, in",
            "any case.  I don't like",
            "the look of this place,",
            "nor the sound of it.",
        ],
        &[
            "The way the wind",
            "gets into the",
            "passageways,",
            "whistling up the",
            "strangest noises.",
            "Deep, sustained echoes",
            "of the wind, not so",
            "much flute-like as...",
        ],
        &[
            "well, it sounds",
            "ridiculous.  In any",
            "case, we've set to work",
            "moving the bulk of the",
            "exterior camp into the",
            "main antechamber, so",
            "there's no use moaning",
            "about it now.",
        ],
    ],
};

static GRIMMOCH_JOURNAL6_CLASS: BookClass = BookClass {
    default_content: Some(&GRIMMOCH_JOURNAL6_CONTENT),
};

/// `GrimmochJournal6()`.
pub fn grimmoch_journal6_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &GRIMMOCH_JOURNAL6, graphic, None, None, 0, false)
}

/// `Server.Items.GrimmochJournal6` `[SerializationGenerator(0)]`.
pub static GRIMMOCH_JOURNAL6: ItemType = ItemType {
    full_name: "Server.Items.GrimmochJournal6",
    parent: Some(&BASE_BOOK),
    construct: Some(grimmoch_journal6_new),
    family: Some(&GRIMMOCH_JOURNAL6_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `GrimmochJournal7.Content`.
pub static GRIMMOCH_JOURNAL7_CONTENT: BookContent = BookContent {
    title: "The daily journal of Grimmoch Drummel",
    author: "Grimmoch",
    pages: &[
        &[
            "Day Seven - Day Ten:",
            "",
            "I cannot stand this",
            "place, I cannot bear it.",
            "I've got to get out.",
            "Something evil lurks",
            "in this ancient place,",
            "something best left",
        ],
        &[
            "alone.  I hear them,",
            "yet none of the others",
            "do.  And yet they",
            "must.  Hands, claws,",
            "scratching at stone,",
            "the awful scratching",
            "and the piteous cries",
            "that sound almost like",
        ],
        &[
            "laughter.  I can hear",
            "them above even the",
            "cracks of the",
            "workmen's picks, and",
            "at night they are all I",
            "can hear.  And yet the",
            "others hear nothing.",
            "We must leave this",
        ],
        &[
            "place, we must.",
            "Three workers have",
            "gone missing - Tavara",
            "expects they've",
            "abandoned us - and I",
            "count them lucky if",
            "they have.  I don't care",
            "what the others say,",
        ],
        &[
            "we must leave this",
            "place.  We must do as",
            "those before and pile",
            "up the stones, block all",
            "access to this primeval",
            "crypt, seal it up again",
            "for all eternity.",
        ],
    ],
};

static GRIMMOCH_JOURNAL7_CLASS: BookClass = BookClass {
    default_content: Some(&GRIMMOCH_JOURNAL7_CONTENT),
};

/// `GrimmochJournal7()`.
pub fn grimmoch_journal7_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &GRIMMOCH_JOURNAL7, graphic, None, None, 0, false)
}

/// `Server.Items.GrimmochJournal7` `[SerializationGenerator(0)]`.
pub static GRIMMOCH_JOURNAL7: ItemType = ItemType {
    full_name: "Server.Items.GrimmochJournal7",
    parent: Some(&BASE_BOOK),
    construct: Some(grimmoch_journal7_new),
    family: Some(&GRIMMOCH_JOURNAL7_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `GrimmochJournal11.Content`.
pub static GRIMMOCH_JOURNAL11_CONTENT: BookContent = BookContent {
    title: "The daily journal of Grimmoch Drummel",
    author: "Grimmoch",
    pages: &[
        &[
            "Day Eleven - Day",
            "Thirteen :",
            "",
            "Lysander is gone, and",
            "two more workers",
            "with him.  Good",
            "riddance to the first.",
            "He knows something.",
        ],
        &[
            "He heard them too, I",
            "know he did - and yet",
            "he scowled at me",
            "when I mentioned",
            "them.  I cannot stop",
            "the noise in my head,",
            "the scratching, the",
            "clawing tears at my",
        ],
        &[
            "senses.  What is it?",
            "What does Lysander",
            "seek that I can only",
            "turn from?  Where",
            "has he gone?  The",
            "only answer to my",
            "questions comes as",
            "laughter from behind",
        ],
        &["the stones."],
    ],
};

static GRIMMOCH_JOURNAL11_CLASS: BookClass = BookClass {
    default_content: Some(&GRIMMOCH_JOURNAL11_CONTENT),
};

/// `GrimmochJournal11()`.
pub fn grimmoch_journal11_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &GRIMMOCH_JOURNAL11, graphic, None, None, 0, false)
}

/// `Server.Items.GrimmochJournal11` `[SerializationGenerator(0)]`.
pub static GRIMMOCH_JOURNAL11: ItemType = ItemType {
    full_name: "Server.Items.GrimmochJournal11",
    parent: Some(&BASE_BOOK),
    construct: Some(grimmoch_journal11_new),
    family: Some(&GRIMMOCH_JOURNAL11_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `GrimmochJournal14.Content`.
pub static GRIMMOCH_JOURNAL14_CONTENT: BookContent = BookContent {
    title: "The daily journal of Grimmoch Drummel",
    author: "Grimmoch",
    pages: &[
        &[
            "Day Fourteen - Day",
            "Sixteen :",
            "",
            "We are lost... we are",
            "lost... all is lost.  The",
            "dead are piled up at",
            "my feet.  Bergen and I",
            "somehow managed in",
        ],
        &[
            "the madness to piece",
            "together a barricade,",
            "barring access to the",
            "camp antechamber.",
            "He knows as well as I",
            "that we cannot hold it",
            "forever.  The dead",
            "come.  They took",
        ],
        &[
            "Lysander before our",
            "eyes.  I pity the soul",
            "of even such a",
            "madman - no one",
            "should die in such a",
            "manner.  And yet so",
            "many have.  We're",
            "trapped here in this",
        ],
        &[
            "horror.  So many have",
            "died, and for what?",
            "What curse have we",
            "stumbled upon?  I",
            "cannot bear it, the",
            "moaning, wailing cries",
            "of the dead.  Poor",
            "Thomas, cut to pieces",
        ],
        &[
            "by their blades.  We",
            "had only an hour to",
            "properly bury those",
            "we could, before the",
            "undead legions struck",
            "again.  I cannot go on...",
            "I cannot go on.",
        ],
    ],
};

static GRIMMOCH_JOURNAL14_CLASS: BookClass = BookClass {
    default_content: Some(&GRIMMOCH_JOURNAL14_CONTENT),
};

/// `GrimmochJournal14()`.
pub fn grimmoch_journal14_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &GRIMMOCH_JOURNAL14, graphic, None, None, 0, false)
}

/// `Server.Items.GrimmochJournal14` `[SerializationGenerator(0)]`.
pub static GRIMMOCH_JOURNAL14: ItemType = ItemType {
    full_name: "Server.Items.GrimmochJournal14",
    parent: Some(&BASE_BOOK),
    construct: Some(grimmoch_journal14_new),
    family: Some(&GRIMMOCH_JOURNAL14_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `GrimmochJournal17.Content`.
pub static GRIMMOCH_JOURNAL17_CONTENT: BookContent = BookContent {
    title: "The daily journal of Grimmoch Drummel",
    author: "Grimmoch",
    pages: &[
        &[
            "Day Seventeen - Day",
            "Twenty-Two :",
            "",
            "The fighting never",
            "ceases... the blood",
            "never stops flowing,",
            "like a river through",
            "the bloated corpses of",
        ],
        &[
            "the dead.  And yet",
            "there are still more.",
            "Always more, with",
            "the red fire gleaming",
            "in their eyes.  My",
            "arm aches, I've taken",
            "to the sword as my",
            "bow seems to do little",
        ],
        &[
            "good... the dull ache in",
            "my arm... so many",
            "swings, cleaving a",
            "mountain of decaying",
            "flesh.  And Thomas...",
            "he was there, in the",
            "thick of it... Thomas",
            "was beside me...",
        ],
        &[
            "his face cleaved in",
            "twain - and yet beside",
            "me, fighting with us",
            "against the horde until",
            "he was cut down once",
            "again.  And I swear I",
            "see him even now,",
            "there in the dark",
        ],
        &[
            "corner of the",
            "antechamber, his eyes",
            "flickering in the last",
            "dying embers of the",
            "fire... and he stares at",
            "me, and a scream fills",
            "the vault - whether",
            "his or mine, I can no",
        ],
        &["longer tell."],
    ],
};

static GRIMMOCH_JOURNAL17_CLASS: BookClass = BookClass {
    default_content: Some(&GRIMMOCH_JOURNAL17_CONTENT),
};

/// `GrimmochJournal17()`.
pub fn grimmoch_journal17_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &GRIMMOCH_JOURNAL17, graphic, None, None, 0, false)
}

/// `Server.Items.GrimmochJournal17` `[SerializationGenerator(0)]`.
pub static GRIMMOCH_JOURNAL17: ItemType = ItemType {
    full_name: "Server.Items.GrimmochJournal17",
    parent: Some(&BASE_BOOK),
    construct: Some(grimmoch_journal17_new),
    family: Some(&GRIMMOCH_JOURNAL17_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `GrimmochJournal23.Content`.
pub static GRIMMOCH_JOURNAL23_CONTENT: BookContent = BookContent {
    title: "The daily journal of Grimmoch Drummel",
    author: "Grimmoch",
    pages: &[&["Day Twenty-Three :", "", "We no longer bury the", "dead."]],
};

static GRIMMOCH_JOURNAL23_CLASS: BookClass = BookClass {
    default_content: Some(&GRIMMOCH_JOURNAL23_CONTENT),
};

/// `GrimmochJournal23()`.
pub fn grimmoch_journal23_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &GRIMMOCH_JOURNAL23, graphic, None, None, 0, false)
}

/// `Server.Items.GrimmochJournal23` `[SerializationGenerator(0)]`.
pub static GRIMMOCH_JOURNAL23: ItemType = ItemType {
    full_name: "Server.Items.GrimmochJournal23",
    parent: Some(&BASE_BOOK),
    construct: Some(grimmoch_journal23_new),
    family: Some(&GRIMMOCH_JOURNAL23_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `LysanderNotebook1.Content`.
pub static LYSANDER_NOTEBOOK1_CONTENT: BookContent = BookContent {
    title: "Lysander's Notebook",
    author: "L. Gathenwale",
    pages: &[
        &[
            "Day One :",
            "",
            "At last, it stands",
            "before me.  The doors",
            "of Thy Sanctum will",
            "open to me now, after",
            "all these years of",
            "searching.  I give",
        ],
        &[
            "myself unto Thee,",
            "Khal Ankur, I have",
            "come for Thy secrets",
            "and I will kneel",
            "prostrate before Thee.",
            " Blessed are the",
            "Keepers, praise unto",
            "Thee, a thousand",
        ],
        &["fortunes in the night."],
    ],
};

static LYSANDER_NOTEBOOK1_CLASS: BookClass = BookClass {
    default_content: Some(&LYSANDER_NOTEBOOK1_CONTENT),
};

/// `LysanderNotebook1()`.
pub fn lysander_notebook1_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &LYSANDER_NOTEBOOK1, graphic, None, None, 0, false)
}

/// `Server.Items.LysanderNotebook1` `[SerializationGenerator(0)]`.
pub static LYSANDER_NOTEBOOK1: ItemType = ItemType {
    full_name: "Server.Items.LysanderNotebook1",
    parent: Some(&BASE_BOOK),
    construct: Some(lysander_notebook1_new),
    family: Some(&LYSANDER_NOTEBOOK1_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `LysanderNotebook2.Content`.
pub static LYSANDER_NOTEBOOK2_CONTENT: BookContent = BookContent {
    title: "Lysander's Notebook",
    author: "L. Gathenwale",
    pages: &[
        &[
            "Day Two:",
            "",
            "The woman, Tavara",
            "Sewel, is unbearable.",
            "Her entire demeanor",
            "sickens me.  I would",
            "take her life for Thee",
            "now, my Lord.  But I",
        ],
        &[
            "cannot alert the",
            "others.  Progress is",
            "made too slowly, I",
            "cannot stand this",
            "perpetual waiting.",
            "Today I knelt down",
            "with the workers,",
            "tossing stones and dirt",
        ],
        &[
            "aside with my very",
            "hands as they dug at",
            "the last of the rubble",
            "covering the entrance",
            "to Thy Sanctum.  The",
            "Sewel woman was",
            "shocked at my",
            "demeanor, dirtying",
        ],
        &[
            "my robes, on my",
            "knees in the muck as I",
            "clawed at the rocks.",
            "She thought I did this",
            "for those sickly",
            "scholars, or for her,",
            "or for what she",
            "laughably calls 'The",
        ],
        &[
            "Gift of Discovery', of",
            "learning.  As if I did",
            "not know what I went",
            "to find!  I come for",
            "Thee, Master.  Soon",
            "shall I receive Thy",
            "gifts, Thy blessings.",
            "Patience, eternal",
        ],
        &[
            "patience.  I must take",
            "my lessons well.  I",
            "have learned from",
            "Thee, Master, I have.",
        ],
    ],
};

static LYSANDER_NOTEBOOK2_CLASS: BookClass = BookClass {
    default_content: Some(&LYSANDER_NOTEBOOK2_CONTENT),
};

/// `LysanderNotebook2()`.
pub fn lysander_notebook2_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &LYSANDER_NOTEBOOK2, graphic, None, None, 0, false)
}

/// `Server.Items.LysanderNotebook2` `[SerializationGenerator(0)]`.
pub static LYSANDER_NOTEBOOK2: ItemType = ItemType {
    full_name: "Server.Items.LysanderNotebook2",
    parent: Some(&BASE_BOOK),
    construct: Some(lysander_notebook2_new),
    family: Some(&LYSANDER_NOTEBOOK2_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `LysanderNotebook3.Content`.
pub static LYSANDER_NOTEBOOK3_CONTENT: BookContent = BookContent {
    title: "Lysander's Notebook",
    author: "L. Gathenwale",
    pages: &[
        &[
            "Day Three - Day Six:",
            "",
            "What are these Beasts",
            "that dare to defy our",
            "presence here?  Hast",
            "Thou sent them,",
            "Master?  To tear",
            "apart these foolish",
        ],
        &[
            "ones that accompany",
            "me?  That repugnant",
            "pustule, Drummel, put",
            "forth his absurd little",
            "theories as to the",
            "nature of the Beasts",
            "that attacked our",
            "camp, but I'll have",
        ],
        &[
            "none of his words.  He",
            "asks too many",
            "questions.  He is a",
            "taint upon the grounds",
            "of Thy Sanctum,",
            "Master - I will deal",
            "with him after the",
            "Sewel woman.",
        ],
        &[
            "Speaking of Sewel, I",
            "have convinced that",
            "empty-headed harlot",
            "that we should move",
            "our encampment",
            "within the",
            "antechamber.  She",
            "thinks I worry for",
        ],
        &[
            "her safety.  I come",
            "for thee, Master.  I",
            "make my camp in Thy",
            "chambers.  I sleep",
            "under Thy roof.  I can",
            "feel Thine presence",
            "even now.  Soon,",
            "Master.  Soon.",
        ],
    ],
};

static LYSANDER_NOTEBOOK3_CLASS: BookClass = BookClass {
    default_content: Some(&LYSANDER_NOTEBOOK3_CONTENT),
};

/// `LysanderNotebook3()`.
pub fn lysander_notebook3_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &LYSANDER_NOTEBOOK3, graphic, None, None, 0, false)
}

/// `Server.Items.LysanderNotebook3` `[SerializationGenerator(0)]`.
pub static LYSANDER_NOTEBOOK3: ItemType = ItemType {
    full_name: "Server.Items.LysanderNotebook3",
    parent: Some(&BASE_BOOK),
    construct: Some(lysander_notebook3_new),
    family: Some(&LYSANDER_NOTEBOOK3_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `LysanderNotebook7.Content`.
pub static LYSANDER_NOTEBOOK7_CONTENT: BookContent = BookContent {
    title: "Lysander's Notebook",
    author: "L. Gathenwale",
    pages: &[
        &[
            "Day Seven :",
            "",
            "The Sewel woman",
            "pratters on endlessly.",
            "And she dares to",
            "speak Thy Name,",
            "Master!  I wish so",
            "vehemently to take a",
        ],
        &[
            "knife to that little",
            "neck of hers.  She",
            "struts around the",
            "chambers of Thy",
            "Sanctum with her",
            "repugnant airs, her",
            "scholarly conjecture",
            "on this or that.  That I",
        ],
        &[
            "could peel the skin",
            "from her face and",
            "show her how vile and",
            "ugly she truly is, how",
            "unworthy of entrance",
            "to Thy Sanctum.  I",
            "must take her,",
            "Master.  I must rend",
        ],
        &[
            "that little wench to",
            "pieces.  I ask this gift",
            "of Thee, that I might",
            "cleanse Thy Sanctum",
            "of her presence.  Give",
            "me the Sewel woman",
            "and I shall show you",
            "my mastery of Death,",
        ],
        &[
            "Master.  I shall cut",
            "her to bits and scatter",
            "them before the",
            "others as a warning.",
            "I cannot stand her",
            "presence, I cannot",
            "abide it.  And",
            "Drummel!  He is a",
        ],
        &[
            "pustule that must be",
            "lanced, a sickness that",
            "I must cure by blade",
            "and fire.  Not a trace",
            "of him will be left",
            "when I'm done with",
            "him.  Praises to Thee,",
            "Master.  I shall honor",
        ],
        &["Thee with many", "sacrifices, soon", "enough."],
    ],
};

static LYSANDER_NOTEBOOK7_CLASS: BookClass = BookClass {
    default_content: Some(&LYSANDER_NOTEBOOK7_CONTENT),
};

/// `LysanderNotebook7()`.
pub fn lysander_notebook7_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &LYSANDER_NOTEBOOK7, graphic, None, None, 0, false)
}

/// `Server.Items.LysanderNotebook7` `[SerializationGenerator(0)]`.
pub static LYSANDER_NOTEBOOK7: ItemType = ItemType {
    full_name: "Server.Items.LysanderNotebook7",
    parent: Some(&BASE_BOOK),
    construct: Some(lysander_notebook7_new),
    family: Some(&LYSANDER_NOTEBOOK7_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `LysanderNotebook8.Content`.
pub static LYSANDER_NOTEBOOK8_CONTENT: BookContent = BookContent {
    title: "Lysander's Notebook",
    author: "L. Gathenwale",
    pages: &[
        &[
            "Day Eight - Day Ten :",
            "",
            "Have you taken them,",
            "Master?  They could",
            "not have found a way",
            "past the stones that",
            "block our path!  The",
            "three workers, My",
        ],
        &[
            "Master, where have",
            "they gone?  Curses",
            "upon them!  I'll cut",
            "them all to pieces if",
            "they show their faces",
            "again, then burn the",
            "rest alive upon a pyre,",
            "for all to see, as a",
        ],
        &[
            "warning of Thy",
            "Power.  How could",
            "they have gotten past",
            "me?  I sleep against",
            "the very walls, to",
            "hear Thy Words, to",
            "feel Thy Breath.  I",
            "can find no egress",
        ],
        &[
            "from the chambers",
            "that the Sewel woman",
            "does not know of nor",
            "have men working at",
            "excavating.  Where",
            "have they gone,",
            "Master?  Have you",
            "taken them, or do they",
        ],
        &[
            "truly flee from Thy",
            "Presence?  I will kill",
            "them if they show",
            "their faces again.",
            "Give me Strength, my",
            "Master, to let them",
            "live a while longer,",
            "until they have",
        ],
        &[
            "fulfilled their",
            "purpose and I kneel",
            "before Thee, covered",
            "in their blood.",
        ],
    ],
};

static LYSANDER_NOTEBOOK8_CLASS: BookClass = BookClass {
    default_content: Some(&LYSANDER_NOTEBOOK8_CONTENT),
};

/// `LysanderNotebook8()`.
pub fn lysander_notebook8_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &LYSANDER_NOTEBOOK8, graphic, None, None, 0, false)
}

/// `Server.Items.LysanderNotebook8` `[SerializationGenerator(0)]`.
pub static LYSANDER_NOTEBOOK8: ItemType = ItemType {
    full_name: "Server.Items.LysanderNotebook8",
    parent: Some(&BASE_BOOK),
    construct: Some(lysander_notebook8_new),
    family: Some(&LYSANDER_NOTEBOOK8_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `LysanderNotebook11.Content`.
pub static LYSANDER_NOTEBOOK11_CONTENT: BookContent = BookContent {
    title: "Lysander's Notebook",
    author: "L. Gathenwale",
    pages: &[
        &[
            "Day Eleven - Day",
            "Thirteen:",
            "",
            "I come for Thee, my",
            "Master.  I come!  The",
            "way is clear, I have",
            "found Thy path and",
            "washed it in the blood",
        ],
        &[
            "of the two workers",
            "that caught sight of",
            "me.  Ah, how sweet it",
            "was to cut them open,",
            "to see the blood pour",
            "out in great torrents, to",
            "stand in it, to revel in",
            "it.  If only I had time",
        ],
        &[
            "for the Sewel woman.",
            "But there will be time",
            "enough for her.  I",
            "have learned Thy",
            "Patience, Master.  I",
            "come for Thee.  I walk",
            "Thy halls in penance,",
            "my last steps in this",
        ],
        &[
            "repulsive living",
            "frame.  I come for",
            "Thee and Thy Gifts,",
            "my Master.  Glory",
            "Unto Thee, Khal",
            "Ankur, Keeper of the",
            "Seventh Death,",
            "Master, Leader of the",
        ],
        &[
            "Chosen, the Khaldun.",
            "Praises in Thy",
            "Name, Master of Life",
            "and Death, Lord of All.",
            " Khal Ankur, Master,",
            "Prophet, I join Thy",
            "ranks this night, a",
            "member of the",
        ],
        &["Khaldun at last!"],
    ],
};

static LYSANDER_NOTEBOOK11_CLASS: BookClass = BookClass {
    default_content: Some(&LYSANDER_NOTEBOOK11_CONTENT),
};

/// `LysanderNotebook11()`.
pub fn lysander_notebook11_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &LYSANDER_NOTEBOOK11, graphic, None, None, 0, false)
}

/// `Server.Items.LysanderNotebook11` `[SerializationGenerator(0)]`.
pub static LYSANDER_NOTEBOOK11: ItemType = ItemType {
    full_name: "Server.Items.LysanderNotebook11",
    parent: Some(&BASE_BOOK),
    construct: Some(lysander_notebook11_new),
    family: Some(&LYSANDER_NOTEBOOK11_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal1.Content`.
pub static TAVARAS_JOURNAL1_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day One:",
            "",
            "The workers continue",
            "tirelessly in their",
            "efforts to unload our",
            "supplies even as light",
            "fades.  I feel I should",
            "lend a hand in the",
        ],
        &[
            "effort, and yet I",
            "cannot bear to take my",
            "attention away from",
            "the magnificent stone",
            "doors of the tomb.",
            "Every inch of their",
            "massive frame is",
            "covered with",
        ],
        &[
            "intricately carved",
            "design work - 'tis",
            "truly a sight to see.",
            "I've spent the day",
            "sketching and",
            "cataloging what I can",
            "of them while my",
            "companions set up our",
        ],
        &[
            "camp and make",
            "preparations for",
            "tomorrow's work.",
            "Though the stonework",
            "symbols inspire me to",
            "new flights of fancy,",
            "some of the workers",
            "seem strangely",
        ],
        &[
            "fearful of them.  I",
            "cannot wait 'til the",
            "morrow when those",
            "ancient works of stone",
            "shall swing open and",
            "deliver unto me",
            "everything I have",
            "dreamed of for the",
        ],
        &["last ten years of my", "life."],
    ],
};

static TAVARAS_JOURNAL1_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL1_CONTENT),
};

/// `TavarasJournal1()`.
pub fn tavaras_journal1_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL1, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal1` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL1: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal1",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal1_new),
    family: Some(&TAVARAS_JOURNAL1_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal2.Content`.
pub static TAVARAS_JOURNAL2_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day Two:",
            "",
            "Everything we'd",
            "heard and read of the",
            "tomb has proved",
            "correct - and yet,",
            "nothing could prepare",
            "me for the sight of it",
        ],
        &[
            "with my own eyes.",
            "The Tomb of Khal",
            "Ankur has given up",
            "its secrets at last!  The",
            "intricate stonework",
            "that covered the tomb",
            "doors seems to",
            "continue throughout",
        ],
        &[
            "the entirety of the",
            "catacombs, each",
            "hallway and room",
            "yielding a seemingly",
            "endless amount of",
            "information for my",
            "companions and I to",
            "record.  It will take",
        ],
        &[
            "years to catalogue the",
            "entirety of the Tomb,",
            "if those legends of its",
            "massive size prove",
            "true.  Sadly, a good",
            "deal of the Tomb's",
            "interior has been",
            "damaged or utterly",
        ],
        &[
            "destroyed, whether",
            "by seismic activity in",
            "the surrounding",
            "mountainside or",
            "merely the slow",
            "efforts of Time",
            "itself, I do not know.",
            "A good deal of the",
        ],
        &[
            "stonework has been",
            "cracked or collapsed",
            "entirely, especially",
            "near the entrance",
            "supports of the main",
            "hall.  Our passage has",
            "indeed already been",
            "entirely blocked in the",
        ],
        &[
            "first major room",
            "we've discovered, a",
            "massive pile of",
            "boulders and stones",
            "blocking any exit",
            "from the",
            "antechamber.  What",
            "could have caused",
        ],
        &[
            "such a localized",
            "disruption of the",
            "support structures,",
            "one can only guess -",
            "but it will surely take",
            "an entire afternoon's",
            "effort to remove even",
            "a fraction of it.  I look",
        ],
        &[
            "forward to more",
            "progress tomorrow",
            "once the workers have",
            "set to excavating the",
            "hall.",
        ],
    ],
};

static TAVARAS_JOURNAL2_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL2_CONTENT),
};

/// `TavarasJournal2()`.
pub fn tavaras_journal2_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL2, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal2` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL2: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal2",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal2_new),
    family: Some(&TAVARAS_JOURNAL2_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal3.Content`.
pub static TAVARAS_JOURNAL3_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day Three - Day Five:",
            "",
            "I do not understand",
            "this place... not as I",
            "once thought I did.",
            "Something palatable",
            "seems to hinder our",
            "every attempt to",
        ],
        &[
            "investigate this",
            "ancient site.",
            "Excavation work on",
            "the first major",
            "hallway finished only",
            "yesterday - the",
            "amount of stone and",
            "rubble blocking the",
        ],
        &[
            "egress was",
            "astounding, it stands",
            "in immense piles",
            "outside the Tomb's",
            "entrance, as if we",
            "were digging the",
            "tunnels of this",
            "abhorred place",
        ],
        &[
            "ourselves!  The",
            "satisfaction of",
            "completing our efforts",
            "was quickly thwarted,",
            "however, as we",
            "discovered the end of",
            "the hallway we had",
            "just revealed was",
        ],
        &[
            "blocked by yet another",
            "colossal pile of stone.",
            "I've had a few of the",
            "workers set up",
            "primitive scaffolding",
            "in the main",
            "antechamber so that I",
            "can spend my time",
        ],
        &[
            "pouring over the detail",
            "work on the stone",
            "carvings while the",
            "rest of our crew",
            "continue excavating",
            "the inner halls.",
        ],
    ],
};

static TAVARAS_JOURNAL3_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL3_CONTENT),
};

/// `TavarasJournal3()`.
pub fn tavaras_journal3_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL3, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal3` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL3: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal3",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal3_new),
    family: Some(&TAVARAS_JOURNAL3_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal6.Content`.
pub static TAVARAS_JOURNAL6_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day Six:",
            "",
            "Late last night our",
            "camp was set upon by",
            "a pack of wild beasts",
            "- behemoth creatures",
            "with a speed and",
            "viciousness I'd n'ere",
        ],
        &[
            "before seen.  Even",
            "Grimmoch, well",
            "versed in all manner",
            "of wildlife, was",
            "unsure as to their",
            "nature - though I lay",
            "blame upon the",
            "darkness covering",
        ],
        &[
            "their movements",
            "rather than on his",
            "skill as a huntsman.",
            "The attacks did not let",
            "up the entire night,",
            "and we were",
            "eventually forced to",
            "flee into the Tomb",
        ],
        &[
            "itself to take refuge",
            "from the ravenous",
            "creatures - e'en",
            "Lysander's spells",
            "could not keep the foul",
            "things from attacking",
            "in great numbers.",
            "The Tomb performed",
        ],
        &[
            "well as an impromptu",
            "fortress, and we",
            "managed to spend the",
            "night unscathed.",
            "Morning's light",
            "seemed to have",
            "scattered the beasts,",
            "as not a single one of",
        ],
        &[
            "them was to be seen as",
            "we exited the Tomb -",
            "not even a carcass of",
            "the few that were",
            "slain a'fore we fled.",
            "Lysander set the crew",
            "to work, moving our",
            "supplies and gear into",
        ],
        &[
            "the Tomb, in case the",
            "creatures did opt to",
            "return.  Such savage",
            "fury had the beasts -",
            "and not a single one",
            "ever turned to run,",
            "even in the face of",
            "certain death.",
        ],
    ],
};

static TAVARAS_JOURNAL6_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL6_CONTENT),
};

/// `TavarasJournal6()`.
pub fn tavaras_journal6_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL6, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal6` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL6: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal6",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal6_new),
    family: Some(&TAVARAS_JOURNAL6_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal7.Content`.
pub static TAVARAS_JOURNAL7_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day Seven:",
            "",
            "T'was written that,",
            "upon his death, Khal",
            "Ankur's followers,",
            "those known as the",
            "Keepers of the",
            "Seventh Death, sealed",
        ],
        &[
            "themselves within the",
            "Sanctum they had",
            "carved from the",
            "mountains in his",
            "honor.  The Zealots of",
            "his order entombed",
            "the lesser followers",
            "alive, then, when all",
        ],
        &[
            "but two remained, slit",
            "their throats and",
            "joined Khal Ankur in",
            "death.  Surely this is",
            "not surprising for a",
            "Cult that worshipped",
            "death and sacrifice so",
            "vehemently as it is",
        ],
        &[
            "said that the Keepers",
            "did - and yet, to be in",
            "this Tomb, to know",
            "that somewhere in its",
            "depths hundreds upon",
            "hundreds of bodies",
            "lay, sealed alive at",
            "their own behest...",
        ],
        &[
            "I must confess that",
            "the very thought of it",
            "troubles my dreams at",
            "night.  I've asked",
            "Lysander if we might",
            "reestablish the camp",
            "outside the Tomb,",
            "setting up night",
        ],
        &[
            "watches and some sort",
            "of fortification, but",
            "he'll have none of it.  I",
            "did not press the",
            "issue, as I suddenly",
            "felt foolish even at",
            "my askance.",
        ],
    ],
};

static TAVARAS_JOURNAL7_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL7_CONTENT),
};

/// `TavarasJournal7()`.
pub fn tavaras_journal7_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL7, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal7` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL7: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal7",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal7_new),
    family: Some(&TAVARAS_JOURNAL7_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal8.Content`.
pub static TAVARAS_JOURNAL8_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day Eight :",
            "",
            "Astounding progress",
            "was made today, and",
            "my very head spins",
            "with the excitement",
            "of it.  Upon full",
            "excavation of the far",
        ],
        &[
            "western hall, another",
            "large antechamber",
            "was revealed.  By the",
            "larger, mosaic style of",
            "the wall carvings and",
            "their framing, as well",
            "as the numerous",
            "vellum scrolls and",
        ],
        &[
            "tomes held within, the",
            "room appears to have",
            "been a great museum",
            "or library of sorts.",
            "The sheer amount of",
            "written information",
            "encased within this",
            "room would surely",
        ],
        &[
            "take me decades to",
            "study e'en if I could",
            "immediately decipher",
            "the strange text with",
            "which it was written.",
            "My sheer joy at the",
            "discovery was quickly",
            "noted by the brute",
        ],
        &[
            "known as Morg",
            "Bergen, who, even in",
            "his simple way,",
            "seemed just as",
            "delighted as I that",
            "some progress had",
            "been made.  I must",
            "confess, upon his",
        ],
        &[
            "inclusion in our party",
            "at the beginning of",
            "this journey I was",
            "somewhat suspect of",
            "his nature, but he has",
            "a startlingly quick wit",
            "about him for such a",
            "massive, calloused",
        ],
        &[
            "warrior.  While",
            "Lysander and e'en",
            "Grimmoch always",
            "seem to investigate the",
            "tomb with a scowling",
            "determination, Bergen",
            "seems to feel the same",
            "thrill of discovery as",
        ],
        &[
            "I.  I am proud to now",
            "count him as a friend,",
            "and am thankful for",
            "his laughter as well",
            "as his strength.",
        ],
    ],
};

static TAVARAS_JOURNAL8_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL8_CONTENT),
};

/// `TavarasJournal8()`.
pub fn tavaras_journal8_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL8, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal8` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL8: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal8",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal8_new),
    family: Some(&TAVARAS_JOURNAL8_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal9.Content`.
pub static TAVARAS_JOURNAL9_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day Nine - Day Ten:",
            "",
            "The excavation of the",
            "next set of tunnels",
            "has ceased, as three",
            "of the workers have",
            "gone missing in the",
            "night.  Bergen voiced",
        ],
        &[
            "the opinion that they",
            "had most likely",
            "abandoned our group",
            "altogether and headed",
            "back, as they were of",
            "the number that",
            "seemed especially",
            "disturbed by the",
        ],
        &[
            "Tomb.  Lysander had",
            "other ideas, however.",
            "In the middle of our",
            "discussion on the",
            "matter, he went into a",
            "wild tirade on the",
            "possibility that they",
            "had somehow",
        ],
        &[
            "infiltrated the tomb's",
            "interior without us.",
            "The pure, hateful",
            "venom in his voice",
            "when he spoke of the",
            "workers shocked me,",
            "as I had always",
            "thought him to be a",
        ],
        &[
            "levelheaded man of",
            "great learning.  As we",
            "are still at work",
            "digging out the rubble",
            "that blocks all access",
            "to the inner chambers,",
            "I cannot help but",
            "believe the workers",
        ],
        &["must have fled the", "site altogether, as", "Bergen said."],
    ],
};

static TAVARAS_JOURNAL9_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL9_CONTENT),
};

/// `TavarasJournal9()`.
pub fn tavaras_journal9_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL9, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal9` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL9: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal9",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal9_new),
    family: Some(&TAVARAS_JOURNAL9_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal11.Content`.
pub static TAVARAS_JOURNAL11_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day Eleven - Day",
            "Thirteen:",
            "",
            "Two more workers",
            "have gone missing.",
            "Even more disturbing",
            "is the fact that",
            "Lysander has joined",
        ],
        &[
            "them. Late last night",
            "the workers finished",
            "excavating the next",
            "main hall, and we",
            "retired to the main",
            "antechamber and our",
            "camp to rest up for",
            "exploration on the",
        ],
        &[
            "'morrow.  In the",
            "middle of the night we",
            "woke to a strange",
            "howling sound, and as",
            "the men prepared",
            "themselves for",
            "another onslaught of",
            "the beasts that had",
        ],
        &[
            "troubled our outer",
            "camp, it was noticed",
            "that Lysander was",
            "nowhere in our",
            "number.  I cannot",
            "fathom where he has",
            "gone - the newly",
            "revealed chamber",
        ],
        &[
            "holds no immediate",
            "egress, blocked again",
            "by piles of stone and",
            "rubble, and I cannot",
            "believe that Lysander,",
            "of all people, would",
            "have fled this site -",
            "indeed, he had lately",
        ],
        &[
            "grown almost fanatical",
            "in his work to",
            "discover more of the",
            "secrets barred to us",
            "by the consistently",
            "slow progress of",
            "excavating each new",
            "hallway.  The men are",
        ],
        &[
            "at work even now, and",
            "as the ceaseless",
            "thumps and cracks of",
            "their picks",
            "reverberate",
            "throughout the",
            "entirety of the tomb,",
            "the dust continues to",
        ],
        &[
            "pour down from the",
            "ancient stonework",
            "above us like some",
            "horrible, eldritch",
            "curse upon us all.",
        ],
    ],
};

static TAVARAS_JOURNAL11_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL11_CONTENT),
};

/// `TavarasJournal11()`.
pub fn tavaras_journal11_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL11, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal11` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL11: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal11",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal11_new),
    family: Some(&TAVARAS_JOURNAL11_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal14.Content`.
pub static TAVARAS_JOURNAL14_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day Fourteen - Day",
            "Fifteen:",
            "",
            "Lysander has",
            "returned... and yet,",
            "how can I describe the",
            "horror of it?  He",
            "stands across the",
        ],
        &[
            "chamber from me",
            "even now, a changed",
            "man.  His hair hangs",
            "in grimy knots across",
            "his face, his clothes",
            "filthy and torn in",
            "places... and the blood",
            "- covered in blood, his",
        ],
        &[
            "skin shining in",
            "scarlet reflections of",
            "the torchlight.  He",
            "will let no one",
            "approach; a thick,",
            "rusted dagger in his",
            "hand warding off any",
            "attempts to overcome",
        ],
        &[
            "him.  And the blood,",
            "which runs down in",
            "great rivulets from",
            "his arms and hands -",
            "it is not his own, and",
            "this is enough to keep",
            "us at a wary distance.",
            "Morg Bergen wishes",
        ],
        &[
            "to subdue him",
            "quickly, but there is",
            "something in",
            "Lysander's eyes - and",
            "I remember the power",
            "of his spells, even as",
            "he swings the jagged",
            "dagger back and forth",
        ],
        &[
            "in a wide swath",
            "before him.",
            "Something about the",
            "sight of it makes my",
            "stomach churn.",
            "Something has",
            "happened, something",
            "that changes",
        ],
        &[
            "everything.  Lysander",
            "has lost his sanity to",
            "this tomb... or to",
            "something within it.",
            "Do we dare approach?",
            "We must make a",
            "decision soon.",
        ],
    ],
};

static TAVARAS_JOURNAL14_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL14_CONTENT),
};

/// `TavarasJournal14()`.
pub fn tavaras_journal14_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL14, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal14` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL14: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal14",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal14_new),
    family: Some(&TAVARAS_JOURNAL14_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal16.Content`.
pub static TAVARAS_JOURNAL16_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day Sixteen:",
            "",
            "Why do I write?  I",
            "must... not so much",
            "because there must be",
            "some record of",
            "this... what's",
            "happened here... as",
        ],
        &[
            "for my own sanity.",
            "The act of putting pen",
            "to paper calms me,",
            "focuses me, even in",
            "this madness.",
            "Lysander is dead.  So",
            "many are dead.  And",
            "we're trapped here,",
        ],
        &[
            "trapped forever in",
            "this nightmare.  He",
            "would not let us pass,",
            "wild in his psychosis,",
            "furious, spitting,",
            "covered in blood, he",
            "swung the ancient",
            "dagger at any who",
        ],
        &[
            "approached.  He",
            "babbled incoherently,",
            "cursed at us, the most",
            "hateful curses,",
            "prophecy, doom upon",
            "us.  Bergen would",
            "have none of it.",
            "Finally, he leapt at",
        ],
        &[
            "Lysander, his",
            "massive axe at his",
            "side.  But he would not",
            "be the end of the mad",
            "mage... no... they",
            "were... those hands,",
            "covered in the dirt of",
            "the grave, maggots,",
        ],
        &[
            "filth.  They rose up",
            "behind Lysander.",
            "That look of curiosity",
            "on the mage's face as",
            "Bergen skidded to a",
            "halt... t'was almost a",
            "moment of sanity for",
            "him, surely, to",
        ],
        &[
            "attempt to comprehend",
            "what could have",
            "stopped the warrior in",
            "his tracks.  And then",
            "they were upon him.",
            "Skeletal hands, arms",
            "and faces with loose,",
            "corrupted flesh",
        ],
        &[
            "hanging from yellow",
            "bone.  Inhuman, yet",
            "once human,",
            "staggering towards us",
            "as their companions",
            "tore at Lysander,",
            "coming towards us in",
            "droves.",
        ],
    ],
};

static TAVARAS_JOURNAL16_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL16_CONTENT),
};

/// `TavarasJournal16()`.
pub fn tavaras_journal16_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL16, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal16` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL16: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal16",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal16_new),
    family: Some(&TAVARAS_JOURNAL16_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal16b.Content`.
pub static TAVARAS_JOURNAL16B_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day Sixteen, Later :",
            "",
            "We ran. What could",
            "we do?  We ran back",
            "towards the entrance,",
            "cutting at them when",
            "we could.  T'was a",
            "nightmare, and yet",
        ],
        &[
            "nothing to prepare us",
            "for what would come.",
            "We were almost there,",
            "the entrance to this",
            "abhorred crypt in",
            "sight.  Then the earth",
            "shook with such a",
            "force that we were",
        ],
        &[
            "dropped to our hands",
            "and knees, stumbling",
            "in the darkness with",
            "those... those things",
            "surely behind us.",
            "The noise of falling",
            "rock and crumbling",
            "stone drowned out our",
        ],
        &[
            "piteous cries.  No sign",
            "of the entrance",
            "remained.",
            "We owe our lives to",
            "Bergen, whose wits",
            "returned quickly.",
            "That he could make us",
            "hurry back into the",
        ],
        &[
            "main antechamber...",
            "actually run back",
            "towards those eldritch",
            "dead that stalked us.",
            "But we did, the",
            "strength of his",
            "convictions enough for",
            "us in the moment.",
        ],
        &[
            "And at our campsite",
            "we erected our last",
            "defense, a pitiable",
            "wall of wood and",
            "stone, anything at",
            "hand that might block",
            "the tide of those",
            "nightmare creatures.",
        ],
        &[
            "And I sit against it",
            "even now.  I can hear",
            "their moans, their",
            "wailing cries in the",
            "distance - they'll be",
            "here soon, even at the",
            "unhurried pace of the",
            "shuffling dead.",
        ],
    ],
};

static TAVARAS_JOURNAL16B_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL16B_CONTENT),
};

/// `TavarasJournal16b()`.
pub fn tavaras_journal16b_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL16B, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal16b` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL16B: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal16b",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal16b_new),
    family: Some(&TAVARAS_JOURNAL16B_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal17.Content`.
pub static TAVARAS_JOURNAL17_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day Seventeen - Day",
            "Eighteen :",
            "",
            "I cannot go on much",
            "longer.  I know now",
            "t'was no work of the",
            "earth that trapped us",
            "here - I can feel His",
        ],
        &[
            "force in it.  It was His",
            "will, His power that",
            "has sealed us here in",
            "this nightmare.  The",
            "barricade will not be",
            "enough.  So many of",
            "them. They come like",
            "unto the ocean's waves",
        ],
        &[
            "- ceaseless,",
            "neverending.  For",
            "every five we strike",
            "down, another ten rise",
            "up against us.  And",
            "like the sands we",
            "cannot help but be",
            "brought down, wasted",
        ],
        &["away in this ocean of", "blood."],
    ],
};

static TAVARAS_JOURNAL17_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL17_CONTENT),
};

/// `TavarasJournal17()`.
pub fn tavaras_journal17_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL17, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal17` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL17: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal17",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal17_new),
    family: Some(&TAVARAS_JOURNAL17_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

/// `TavarasJournal19.Content`.
pub static TAVARAS_JOURNAL19_CONTENT: BookContent = BookContent {
    title: "Journal: Discovery of the Tomb",
    author: "Tavara Sewel",
    pages: &[
        &[
            "Day Nineteen - Day",
            "Twenty-One :",
            "",
            "The barricade won't",
            "hold - never, and",
            "they'll come, they",
            "come even now.  I",
            "would tear the last of",
        ],
        &[
            "it down, let them in to",
            "devour us all, if only",
            "to stop the screaming",
            "- the awful, wailing",
            "cries that fill the tomb",
            "with their presence.",
            "May my ancestors",
            "forgive me, but it",
        ],
        &["must be done. I must", "end this."],
    ],
};

static TAVARAS_JOURNAL19_CLASS: BookClass = BookClass {
    default_content: Some(&TAVARAS_JOURNAL19_CONTENT),
};

/// `TavarasJournal19()`.
pub fn tavaras_journal19_new(w: &mut World) -> ItemId {
    let graphic = muo_content::hooks::rnd::from(w, 0xFF1, 2);
    new_book(w, &TAVARAS_JOURNAL19, graphic, None, None, 0, false)
}

/// `Server.Items.TavarasJournal19` `[SerializationGenerator(0)]`.
pub static TAVARAS_JOURNAL19: ItemType = ItemType {
    full_name: "Server.Items.TavarasJournal19",
    parent: Some(&BASE_BOOK),
    construct: Some(tavaras_journal19_new),
    family: Some(&TAVARAS_JOURNAL19_CLASS),
    serialize: |w, id, out| {
        (BASE_BOOK.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (BASE_BOOK.deserialize)(w, id, r)?;
        r.read_encoded_int()?;
        Ok(())
    },
    ..BASE_BOOK
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&GRIMMOCH_JOURNAL1, &[]);
    types.register_item(&GRIMMOCH_JOURNAL2, &[]);
    types.register_item(&GRIMMOCH_JOURNAL3, &[]);
    types.register_item(&GRIMMOCH_JOURNAL6, &[]);
    types.register_item(&GRIMMOCH_JOURNAL7, &[]);
    types.register_item(&GRIMMOCH_JOURNAL11, &[]);
    types.register_item(&GRIMMOCH_JOURNAL14, &[]);
    types.register_item(&GRIMMOCH_JOURNAL17, &[]);
    types.register_item(&GRIMMOCH_JOURNAL23, &[]);
    types.register_item(&LYSANDER_NOTEBOOK1, &[]);
    types.register_item(&LYSANDER_NOTEBOOK2, &[]);
    types.register_item(&LYSANDER_NOTEBOOK3, &[]);
    types.register_item(&LYSANDER_NOTEBOOK7, &[]);
    types.register_item(&LYSANDER_NOTEBOOK8, &[]);
    types.register_item(&LYSANDER_NOTEBOOK11, &[]);
    types.register_item(&TAVARAS_JOURNAL1, &[]);
    types.register_item(&TAVARAS_JOURNAL2, &[]);
    types.register_item(&TAVARAS_JOURNAL3, &[]);
    types.register_item(&TAVARAS_JOURNAL6, &[]);
    types.register_item(&TAVARAS_JOURNAL7, &[]);
    types.register_item(&TAVARAS_JOURNAL8, &[]);
    types.register_item(&TAVARAS_JOURNAL9, &[]);
    types.register_item(&TAVARAS_JOURNAL11, &[]);
    types.register_item(&TAVARAS_JOURNAL14, &[]);
    types.register_item(&TAVARAS_JOURNAL16, &[]);
    types.register_item(&TAVARAS_JOURNAL16B, &[]);
    types.register_item(&TAVARAS_JOURNAL17, &[]);
    types.register_item(&TAVARAS_JOURNAL19, &[]);
}
