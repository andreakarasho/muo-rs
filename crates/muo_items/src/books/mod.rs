//! Books: `BaseBook` and its colored subclasses, `BookContent`, `BookPageInfo`
//! and `BookPackets`. The books with a fixed text are in `defined.rs`.
//!
//! `BaseBook` is an `ISecurable`: its level is pushed on `HouseDeps` and its context menu offers
//! `SetSecureLevelEntry` through `CommonItemHooks`; `Level` is saved and loaded as the raw `SecureLevel` int.
//! // shortcut: a null line of a page (`ReadString` returning null) loads as "" (a null line only
//! appears in a hand-edited save); a book is never saved with a null line.

use muo_io::{GenericReader, GenericWriter, IoError, Result, SeekOrigin, SpanReader};
use muo_primitives::{Serial, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::net_state::NetStateId;
use muo_server::packets::Pk;
use muo_server::type_registry::{Phase, TypeRegistry};

pub mod defined;

#[cfg(test)]
mod tests;

/// `BookContent`: the fixed text of a defined book, the `"bookContent"` prop of its class template
/// (`{"title": .., "author": .., "pages": [[line, ..], ..]}`).
#[derive(Clone, Copy)]
pub struct BookContent<'a> {
    pub title: &'a str,
    pub author: &'a str,
    pub pages: &'a [serde_json::Value],
}

/// The lines of one page of a `"bookContent"` prop.
fn page_lines(page: &serde_json::Value) -> &[serde_json::Value] {
    page.as_array().map_or(&[], Vec::as_slice)
}

impl BookContent<'_> {
    /// `Copy()`.
    pub fn copy(&self) -> Vec<Vec<String>> {
        self.pages
            .iter()
            .map(|p| {
                page_lines(p)
                    .iter()
                    .map(|l| l.as_str().unwrap_or_default().to_string())
                    .collect()
            })
            .collect()
    }

    /// `IsMatch(cmp)`.
    pub fn is_match(&self, cmp: &[Vec<String>]) -> bool {
        cmp.len() == self.pages.len()
            && self.pages.iter().zip(cmp).all(|(a, b)| {
                let a = page_lines(a);
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.as_str() == Some(y))
            })
    }
}

/// What `BaseBook` adds to an `Item`: `[SerializableField]` 0..4.
#[derive(Clone, Debug, Default)]
pub struct BookData {
    /// `SecureLevel` (`Owner` = 0).
    pub level: i32,
    pub title: Option<String>,
    pub author: Option<String>,
    pub writable: bool,
    /// `BookPageInfo[]`: the lines of each page.
    pub pages: Vec<Vec<String>>,
}

pub fn data(w: &World, id: ItemId) -> Option<&BookData> {
    w.item(id)?.ext.get::<BookData>()
}

pub fn data_mut(w: &mut World, id: ItemId) -> Option<&mut BookData> {
    w.item_mut(id)?.ext.get_mut::<BookData>()
}

/// `item is BaseBook`.
pub fn is_book(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_BOOK))
}

/// `DefaultContent` of the item's class: its `"bookContent"` prop.
pub fn default_content(w: &World, id: ItemId) -> Option<BookContent<'_>> {
    let v = muo_content::templates::props::item(w, id, "bookContent")?;
    Some(BookContent {
        title: v.get("title")?.as_str()?,
        author: v.get("author")?.as_str()?,
        pages: v.get("pages")?.as_array()?,
    })
}

/// `BaseBook(itemID, title, author, pageCount, writable)` of class `ty`; `BaseBook(itemID, pageCount = 20,
/// writable = true)` passes null title and author, `BaseBook(itemID, writable)` pageCount 0.
pub fn new_book(
    w: &mut World,
    ty: &'static ItemType,
    item_id: i32,
    title: Option<&str>,
    author: Option<&str>,
    page_count: i32,
    writable: bool,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    muo_content::templates::table(w); // a test world loads the templates lazily: the content is a prop
    let content = default_content(w, id);
    let pages = match content {
        None => vec![Vec::new(); page_count.max(0) as usize],
        Some(c) => c.copy(),
    };
    let d = BookData {
        level: 0,
        title: title
            .map(str::to_string)
            .or_else(|| content.map(|c| c.title.to_string())),
        author: author
            .map(str::to_string)
            .or_else(|| content.map(|c| c.author.to_string())),
        writable,
        pages,
    };
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    id
}

/// `Title = value`.
pub fn set_title(w: &mut World, id: ItemId, value: Option<String>) {
    if let Some(d) = data_mut(w, id)
        && d.title != value
    {
        d.title = value;
        ops::invalidate_properties(w, id);
    }
}

/// `Author = value`.
pub fn set_author(w: &mut World, id: ItemId, value: Option<String>) {
    if let Some(d) = data_mut(w, id)
        && d.author != value
    {
        d.author = value;
        ops::invalidate_properties(w, id);
    }
}

/// `ContentAsString`: every line, each followed by a new line.
pub fn content_as_string(w: &World, id: ItemId) -> String {
    let mut s = String::new();
    for line in data(w, id).iter().flat_map(|d| d.pages.iter().flatten()) {
        s.push_str(line);
        s.push_str(if cfg!(windows) { "\r\n" } else { "\n" });
    }
    s
}

/// `ContentAsStringArray`.
pub fn content_as_string_array(w: &World, id: ItemId) -> Vec<String> {
    data(w, id)
        .map(|d| d.pages.iter().flatten().cloned().collect())
        .unwrap_or_default()
}

mod save_flag {
    pub const TITLE: i32 = 0x1;
    pub const AUTHOR: i32 = 0x2;
    pub const WRITABLE: i32 = 0x4;
    pub const PAGES: i32 = 0x8;
}

const SERIALIZATION_VERSION: i32 = 5;

fn serialize_book(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(SERIALIZATION_VERSION);
    let d = data(w, id).cloned().unwrap_or_default();
    let content = default_content(w, id);
    let mut flags = 0;
    if d.title.as_deref() != content.map(|c| c.title) {
        flags |= save_flag::TITLE;
    }
    if d.author.as_deref() != content.map(|c| c.author) {
        flags |= save_flag::AUTHOR;
    }
    if d.writable {
        flags |= save_flag::WRITABLE;
    }
    if !content.is_some_and(|c| c.is_match(&d.pages)) {
        flags |= save_flag::PAGES;
    }
    out.write_enum(4, i64::from(flags));
    out.write_enum(4, i64::from(d.level));
    if flags & save_flag::TITLE != 0 {
        out.write_opt_string(d.title.as_deref());
    }
    if flags & save_flag::AUTHOR != 0 {
        out.write_opt_string(d.author.as_deref());
    }
    if flags & save_flag::PAGES != 0 {
        out.write_encoded_int(d.pages.len() as i32);
        for page in &d.pages {
            write_page(out, page);
        }
    }
}

/// `BookPageInfo.Serialize`.
fn write_page(out: &mut dyn GenericWriter, lines: &[String]) {
    out.write_i32(lines.len() as i32);
    for l in lines {
        out.write_opt_string(Some(l));
    }
}

/// `new BookPageInfo(reader)`.
fn read_page(r: &mut dyn GenericReader) -> Result<Vec<String>> {
    let n = r.read_i32()?;
    if n < 0 {
        return Err(IoError::InvalidData(format!("BookPageInfo: {n} lines")));
    }
    let mut lines = Vec::new();
    for _ in 0..n {
        lines.push(r.read_string()?.unwrap_or_default());
    }
    Ok(lines)
}

fn read_pages(r: &mut dyn GenericReader) -> Result<Vec<Vec<String>>> {
    let n = r.read_encoded_int()?;
    if n < 0 {
        return Err(IoError::InvalidData(format!("BaseBook pages: {n}")));
    }
    let mut pages = Vec::new();
    for _ in 0..n {
        pages.push(read_page(r)?);
    }
    Ok(pages)
}

fn deserialize_book(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let content = default_content(w, id);
    let mut d = BookData::default();
    if version < SERIALIZATION_VERSION {
        // private void Deserialize(IGenericReader reader, int version)
        d.level = r.read_i32()?;
        let flags = r.read_encoded_int()?;
        if flags & 0x01 != 0 {
            d.title = r.read_string()?;
        } else if let Some(c) = content {
            d.title = Some(c.title.to_string());
        }
        if flags & 0x02 != 0 {
            d.author = r.read_string()?;
        } else if let Some(c) = content {
            d.author = Some(c.author.to_string());
        }
        d.writable = flags & 0x04 != 0;
        d.pages = if flags & 0x08 != 0 {
            read_pages(r)?
        } else {
            content.map(|c| c.copy()).unwrap_or_default()
        };
    } else {
        let flags = r.read_enum(4)? as i32;
        d.level = r.read_enum(4)? as i32;
        d.title = if flags & save_flag::TITLE != 0 {
            r.read_string()?
        } else {
            content.map(|c| c.title.to_string())
        };
        d.author = if flags & save_flag::AUTHOR != 0 {
            r.read_string()?
        } else {
            content.map(|c| c.author.to_string())
        };
        d.writable = flags & save_flag::WRITABLE != 0;
        d.pages = if flags & save_flag::PAGES != 0 {
            read_pages(r)?
        } else {
            content.map(|c| c.copy()).unwrap_or_default()
        };
    }
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    Ok(())
}

// ---- packets ----

/// `SendBookContent`.
pub fn send_book_content(w: &mut World, ns: NetStateId, book: ItemId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let (Some(serial), Some(d)) = (w.item(book).map(|i| i.serial), data(w, book).cloned()) else {
        return;
    };
    let mut p = Pk::new(9);
    p.u8(0x66); // Packet ID
    p.u16(0); // length, patched
    p.serial(serial);
    p.u16(d.pages.len() as u16);
    for (i, page) in d.pages.iter().enumerate() {
        p.u16((i + 1) as u16);
        p.u16(page.len() as u16);
        for line in page {
            let _ = p.write_utf8_null(line);
        }
    }
    let bytes = p.finish_with_length();
    w.send(ns, &bytes);
}

/// `SendBookCover`.
pub fn send_book_cover(w: &mut World, ns: NetStateId, from: MobileId, book: ItemId) {
    if w.cannot_send_packets(ns) {
        return;
    }
    let (Some(serial), Some(d)) = (w.item(book).map(|i| i.serial), data(w, book).cloned()) else {
        return;
    };
    let title = d.title.clone().unwrap_or_default();
    let author = d.author.clone().unwrap_or_default();
    let loc = ops::get_world_location(w, book);
    let near = deps::mobile_in_range(w, from, loc, 1);
    let length = 17 + title.len() + author.len();
    let mut p = Pk::new(length);
    p.u8(0xD4); // Packet ID
    p.u16(length as u16);
    p.serial(serial);
    p.u8(0x1); // Flag on
    p.bool(d.writable && near);
    p.u16(d.pages.len() as u16);
    p.u16((title.len() + 1) as u16);
    let _ = p.write_utf8_null(&title);
    p.u16((author.len() + 1) as u16);
    let _ = p.write_utf8_null(&author);
    let bytes = p.finish();
    w.send(ns, &bytes);
}

/// The book a header or content packet names, when the player may change it.
fn writable_book(w: &mut World, from: MobileId, r: &mut SpanReader) -> Option<ItemId> {
    let book = w.find_item(Serial::new(r.read_u32().ok()?))?;
    if !is_book(w, book) || !data(w, book)?.writable {
        return None;
    }
    let loc = ops::get_world_location(w, book);
    if !deps::mobile_in_range(w, from, loc, 1) {
        return None;
    }
    let ty = w.item(book)?.ty;
    (ty.is_accessible_to)(w, book, from).then_some(book)
}

fn packet_mobile(w: &mut World, ns: &muo_net::net_state::NetState) -> Option<MobileId> {
    let id = muo_server::net_glue::ensure_net_state(w, ns);
    w.net_state(id).and_then(|d| d.mobile)
}

/// `BookPackets.OldHeaderChange` (0x93).
fn old_header_change(w: &mut World, ns: &mut muo_net::net_state::NetState, body: &[u8]) {
    let Some(from) = packet_mobile(w, ns) else {
        return;
    };
    let mut r = SpanReader::new(body);
    let Some(book) = writable_book(w, from, &mut r) else {
        return;
    };
    let read = (|| {
        r.seek(4, SeekOrigin::Current).ok()?; // Skip flags and page count
        let title = r.read_latin1_safe(Some(60)).ok()?;
        let author = r.read_latin1_safe(Some(30)).ok()?;
        Some((title, author))
    })();
    let Some((title, author)) = read else { return };
    set_title(w, book, Some(utility::fix_html(&title)));
    set_author(w, book, Some(utility::fix_html(&author)));
}

/// `BookPackets.HeaderChange` (0xD4).
fn header_change(w: &mut World, ns: &mut muo_net::net_state::NetState, body: &[u8]) {
    let Some(from) = packet_mobile(w, ns) else {
        return;
    };
    let mut r = SpanReader::new(body);
    let Some(book) = writable_book(w, from, &mut r) else {
        return;
    };
    let read = (|| {
        r.seek(4, SeekOrigin::Current).ok()?; // Skip flags and page count
        let title_length = usize::from(r.read_u16().ok()?);
        if title_length > 60 {
            return None;
        }
        let title = r.read_utf8_safe(Some(title_length)).ok()?;
        let author_length = usize::from(r.read_u16().ok()?);
        if author_length > 30 {
            return None;
        }
        let author = r.read_utf8_safe(Some(author_length)).ok()?;
        Some((title, author))
    })();
    let Some((title, author)) = read else { return };
    set_title(w, book, Some(utility::fix_html(&title)));
    set_author(w, book, Some(utility::fix_html(&author)));
}

/// `BookPackets.ContentChange` (0x66).
fn content_change(w: &mut World, ns: &mut muo_net::net_state::NetState, body: &[u8]) {
    let Some(from) = packet_mobile(w, ns) else {
        return;
    };
    let mut r = SpanReader::new(body);
    let Some(book) = writable_book(w, from, &mut r) else {
        return;
    };
    let count = data(w, book).map_or(0, |d| d.pages.len());
    let Ok(page_count) = r.read_u16() else { return };
    if usize::from(page_count) > count {
        return;
    }
    for _ in 0..page_count {
        let Ok(index) = r.read_u16() else { return };
        let index = usize::from(index);
        if index < 1 || index > count {
            return;
        }
        let Ok(line_count) = r.read_u16() else { return };
        if line_count > 8 {
            return;
        }
        let mut lines = Vec::new();
        for _ in 0..line_count {
            let Ok(line) = r.read_utf8_safe(None) else {
                return;
            };
            // `Length >= 80` counts UTF-16 units
            if line.encode_utf16().count() >= 80 {
                return;
            }
            lines.push(line);
        }
        if let Some(d) = data_mut(w, book) {
            d.pages[index - 1] = lines;
        }
    }
}

/// `BookPackets.Configure`.
fn configure(w: &mut World) {
    use muo_content::network::register_packet as reg;
    reg(w, 0xD4, 0, true, false, header_change);
    reg(w, 0x66, 0, true, false, content_change);
    reg(w, 0x93, 99, true, false, old_header_change);
    colored_ctors::configure(w);
}

pub mod colored_ctors;

// ---- classes ----

fn name_or_base(w: &World, id: ItemId, list: &mut muo_server::property_list::PropertyList) {
    match data(w, id)
        .and_then(|d| d.title.as_deref())
        .filter(|t| !t.is_empty())
    {
        Some(t) => list.add_text(t),
        None => base::add_name_property_base(w, id, list),
    }
}

/// `Server.Items.BaseBook`.
pub static BASE_BOOK: ItemType = ItemType {
    full_name: "Server.Items.BaseBook",
    parent: Some(&ITEM_BASE),
    // shortcut: every `[Constructible]` BaseBook constructor takes an item id, so [add has none to call.
    construct: None,
    add_name_property: name_or_base,
    get_context_menu_entries: |w, id, from, list| {
        base::get_context_menu_entries_base(w, id, from, list);
        if let Some(f) = muo_content::items::common::hooks(w).and_then(|h| h.secure_level_entry)
            && let Some(e) = f(w, from, id)
        {
            list.push(e);
        }
    },
    on_single_click: |w, id, from| {
        let d = data(w, id).cloned().unwrap_or_default();
        ops::label_to(
            w,
            id,
            from,
            &format!(
                "{} by {}",
                d.title.unwrap_or_default(),
                d.author.unwrap_or_default()
            ),
        );
        ops::label_to(w, id, from, &format!("[{} pages]", d.pages.len()));
    },
    on_double_click: |w, id, from| {
        let (no_title, writable) = data(w, id).map_or((false, false), |d| {
            (d.title.is_none() && d.author.is_none(), d.writable)
        });
        if no_title && writable {
            set_title(w, id, Some("a book".to_string()));
            let name = w.mobile(from).and_then(|m| (m.ty.name)(w, from));
            set_author(w, id, name);
        }
        if let Some(ns) = deps::net_state_of(w, from) {
            send_book_cover(w, ns, from, id);
            send_book_content(w, ns, id);
        }
    },
    dupe: |w, id, new_item| {
        // CopyProperties: Title, Author and Writable are copied; Level and Pages are [IgnoreDupe].
        if let (Some(src), true) = (data(w, id).cloned(), is_book(w, new_item))
            && let Some(d) = data_mut(w, new_item)
        {
            d.title = src.title;
            d.author = src.author;
            d.writable = src.writable;
        }
        base::dupe_base(w, id, new_item);
    },
    on_after_duped: |w, id, new_item| {
        if !is_book(w, new_item) {
            return;
        }
        let pages = data(w, id).map(|d| d.pages.clone()).unwrap_or_default();
        if let Some(d) = data_mut(w, new_item) {
            d.pages = pages;
        }
    },
    serialize: serialize_book,
    deserialize: deserialize_book,
    ..ITEM_BASE
};

/// One book class that is "parent part + version": `NAME = "Server.Items.Foo" : PARENT, |w| ctor, encoded <bool>;`
/// or, for the books of `defined.rs`, `content X [, label "text"]`.
macro_rules! book_leaf {
    (
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident, |$w:ident| $ctor:expr, encoded $enc:literal;
    ) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|$w| $ctor),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                $crate::misc::write_version(out, $enc);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                $crate::misc::read_version(r, $enc)?;
                Ok(())
            },
            ..$parent
        };
    };
}

/// `new <Book>()` of a book with a fixed text: `base(0, false)` of its parent then the class template.
pub fn defined_book_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = new_book(w, ty, 0, None, None, 0, false);
    muo_content::templates::apply_item_as(w, id, ty.full_name);
    id
}

/// The `"nameText"` prop of a defined book: the name it shows instead of its title.
fn name_text(w: &World, id: ItemId) -> Option<&'static str> {
    use muo_content::templates::props;
    props::as_static_str(w, props::item(w, id, "nameText")).flatten()
}

/// `AddNameProperty` of a defined book: its `"nameText"`, else `BaseBook`'s.
pub fn defined_add_name_property(
    w: &World,
    id: ItemId,
    list: &mut muo_server::property_list::PropertyList,
) {
    match name_text(w, id) {
        Some(t) => list.add_text(t),
        None => (BASE_BOOK.add_name_property)(w, id, list),
    }
}

/// `OnSingleClick` of a defined book: its `"nameText"`, else `BaseBook`'s.
pub fn defined_on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    match name_text(w, id) {
        Some(t) => ops::label_to(w, id, from, t),
        None => (BASE_BOOK.on_single_click)(w, id, from),
    }
}

/// A book of `defined.rs`: `NAME = "Server.Items.Foo" : PARENT;` (encoded version 0).
macro_rules! defined_book {
    (
        $(#[$m:meta])*
        $name:ident = $full:literal : $parent:ident;
    ) => {
        $(#[$m])*
        pub static $name: muo_server::item::ItemType = muo_server::item::ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some(|w| $crate::books::defined_book_new(w, &$name)),
            add_name_property: $crate::books::defined_add_name_property,
            on_single_click: $crate::books::defined_on_single_click,
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                $crate::misc::write_version(out, true);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                $crate::misc::read_version(r, true)?;
                Ok(())
            },
            ..$parent
        };
    };
}
pub(crate) use defined_book;

/// `RedBook()`: `base(0xFF1)`; the other colors differ in the graphic and `BlueBook`'s 40 pages.
macro_rules! colored_book {
    ($name:ident, $full:literal, $graphic:literal, $pages:literal) => {
        book_leaf! {
            #[doc = concat!("`", $full, "`.")]
            $name = $full : BASE_BOOK, |w| new_book(w, &$name, $graphic, None, None, $pages, true), encoded false;
        }
    };
}

colored_book!(RED_BOOK, "Server.Items.RedBook", 0xFF1, 20);
colored_book!(BLUE_BOOK, "Server.Items.BlueBook", 0xFF2, 40);
colored_book!(BROWN_BOOK, "Server.Items.BrownBook", 0xFEF, 20);
colored_book!(TAN_BOOK, "Server.Items.TanBook", 0xFF0, 20);

/// The classes of this file.
pub static ALL: &[&ItemType] = &[&BASE_BOOK, &RED_BOOK, &BLUE_BOOK, &BROWN_BOOK, &TAN_BOOK];

/// Registers the classes of this folder.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
    defined::register(types);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BookPackets.Configure",
        configure,
    );
    types.register_phase_default(Phase::Configure, "Server.Items.BaseBook.Securable", |w| {
        let d = muo_content::items::house_deps::deps_mut(w);
        d.securables
            .push(|w, id| is_book(w, id).then(|| data(w, id).map_or(0, |d| d.level) as u8));
        d.securable_setters.push(|w, id, level| {
            if !is_book(w, id) {
                return false;
            }
            if let Some(d) = data_mut(w, id) {
                d.level = i32::from(level);
            }
            true
        });
    });
}
