//! Books: names are unique, every class constructs and round-trips its save bytes, and the save layout is
//! `BaseBook`'s (version 5, `SaveFlag`, `SecureLevel`, then the
//! title, author and pages that differ from `DefaultContent`).

use std::collections::HashSet;

use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::item::{ITEM_BASE, Item, ItemType};

use super::*;

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn item_part(w: &World, id: ItemId) -> Vec<u8> {
    let mut out = SaveWriter::new(true);
    (ITEM_BASE.serialize)(w, id, &mut out);
    out.into_vec()
}

fn all() -> Vec<&'static ItemType> {
    ALL.iter()
        .chain(defined::ALL.iter())
        .copied()
        .filter(|t| t.construct.is_some())
        .collect()
}

fn reload(w: &mut World, t: &'static ItemType, bytes: &[u8]) -> ItemId {
    let id = w.add_item(Item::new_for_load(Serial::new(0x4000_0200), t));
    let mut r = SaveReader::new(bytes);
    (t.deserialize)(w, id, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
    assert_eq!(r.remaining(), 0, "{}", t.full_name);
    id
}

#[test]
fn names_are_unique_and_every_book_constructs() {
    let mut seen = HashSet::new();
    for t in ALL.iter().chain(defined::ALL.iter()) {
        assert!(seen.insert(t.full_name), "{} twice", t.full_name);
    }
    assert_eq!(seen.len(), 5 + 34);
    for t in all() {
        let mut w = World::new();
        let id = (t.construct.unwrap())(&mut w);
        assert_eq!(w.item(id).unwrap().ty.full_name, t.full_name);
    }
}

#[test]
fn every_book_round_trips() {
    for t in all() {
        let mut w = World::new();
        let id = (t.construct.unwrap())(&mut w);
        let bytes = save(&w, id);
        let again = reload(&mut w, t, &bytes);
        assert_eq!(save(&w, again), bytes, "{}", t.full_name);
        assert_eq!(
            data(&w, again).unwrap().pages,
            data(&w, id).unwrap().pages,
            "{}",
            t.full_name
        );
    }
}

#[test]
fn red_book_record() {
    let mut w = World::new();
    let id = (RED_BOOK.construct.unwrap())(&mut w);
    // BaseBook: version 5, flags (Title|Author|Writable|Pages = 1+2 are set since the title and author are null and
    // the default is null too, so only Writable 4 and Pages 8 since 20 empty pages are not the default (none)),
    // level 0, then 20 pages of 0 lines; RedBook: version 0.
    let mut expected = item_part(&w, id);
    expected.extend([5, 0, 0, 0]); // SerializationVersion
    expected.push(0x04 | 0x08); // SaveFlag, encoded int
    expected.push(0); // SecureLevel.Owner
    expected.push(20); // pages, encoded int
    for _ in 0..20 {
        expected.extend([0, 0, 0, 0]); // BookPageInfo: zero lines
    }
    expected.extend([0, 0, 0, 0]); // RedBook version
    assert_eq!(save(&w, id), expected);
}

#[test]
fn defined_book_writes_only_what_differs() {
    let mut w = World::new();
    let id = (defined::DRAKOVS_JOURNAL.construct.unwrap())(&mut w);
    let d = data(&w, id).unwrap();
    assert_eq!(d.title.as_deref(), Some("Drakov's Journal"));
    assert_eq!(d.author.as_deref(), Some("Drakov"));
    assert!(!d.writable);
    let mut expected = item_part(&w, id);
    expected.extend([5, 0, 0, 0, 0, 0]); // version, no flags, SecureLevel.Owner
    expected.extend([0, 0, 0, 0]); // BlueBook: int version 0
    expected.push(0); // DrakovsJournal: encoded version 0
    assert_eq!(save(&w, id), expected);
    // Editing a line makes the pages differ from the content: they are saved in full.
    data_mut(&mut w, id).unwrap().pages[0][0] = "changed".into();
    let bytes = save(&w, id);
    assert_eq!(bytes[item_part(&w, id).len() + 4], 0x08);
    let again = reload(&mut w, &defined::DRAKOVS_JOURNAL, &bytes);
    assert_eq!(data(&w, again).unwrap().pages[0][0], "changed");
}

#[test]
fn old_saves_load() {
    // version 4 (< 5): SecureLevel int, flags, [title], [author], [pages].
    let mut w = World::new();
    let src = (RED_BOOK.construct.unwrap())(&mut w);
    let mut bytes = item_part(&w, src);
    bytes.extend([4, 0, 0, 0]); // version
    bytes.extend([3, 0, 0, 0]); // Level = Anyone
    bytes.push(0x01 | 0x04 | 0x08); // Title, Writable, Content
    bytes.extend([1, 3, b'a', b'b', b'c']); // title: present flag, length, bytes
    bytes.push(1); // one page
    bytes.extend([1, 0, 0, 0, 1, 2, b'h', b'i']); // one line "hi"
    bytes.extend([0, 0, 0, 0]); // RedBook version
    let id = reload(&mut w, &RED_BOOK, &bytes);
    let d = data(&w, id).unwrap();
    assert_eq!(
        (d.level, d.title.as_deref(), d.author.as_deref(), d.writable),
        (3, Some("abc"), None, true)
    );
    assert_eq!(d.pages, vec![vec!["hi".to_string()]]);
}

#[test]
fn dupe_copies_header_and_pages() {
    let mut w = World::new();
    let a = (BLUE_BOOK.construct.unwrap())(&mut w);
    let b = (BLUE_BOOK.construct.unwrap())(&mut w);
    set_title(&mut w, a, Some("t".into()));
    data_mut(&mut w, a).unwrap().pages[3] = vec!["x".into(), "y".into()];
    data_mut(&mut w, a).unwrap().level = 2;
    (BLUE_BOOK.dupe)(&mut w, a, b);
    let d = data(&w, b).unwrap();
    assert_eq!(d.title.as_deref(), Some("t"));
    assert_eq!(d.pages[3], vec!["x".to_string(), "y".to_string()]);
    assert_eq!(d.pages.len(), 40);
    assert_eq!(d.level, 0, "Level is [IgnoreDupe]");
}

#[test]
fn content_match() {
    let c = &defined::DRAKOVS_JOURNAL_CONTENT;
    assert!(c.is_match(&c.copy()));
    let mut changed = c.copy();
    changed.pop();
    assert!(!c.is_match(&changed));
}
