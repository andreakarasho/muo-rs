//! The `[Constructible]` overloads of `RedBook`, `BlueBook`, `BrownBook` and `TanBook`:
//! `()`, `(int pageCount, bool writable)` and `(string title, string author, int pageCount, bool writable)`.
//! Character creation makes `new RedBook("a book", name, 20, true)` through `factory::make_item_args`.

use muo_content::props::{Ctor, Kind, Param, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_server::World;
use muo_server::ids::EntityId;

use super::{BLUE_BOOK, BROWN_BOOK, RED_BOOK, TAN_BOOK, new_book};

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
const BOOL: Kind = Kind::Plain(PropType::Bool);
const STR: Kind = Kind::Plain(PropType::Str);

static PAGES_WRITABLE: [Param; 2] = [
    Param {
        name: "pageCount",
        kind: INT,
        default: None,
        ty: "",
    },
    Param {
        name: "writable",
        kind: BOOL,
        default: None,
        ty: "",
    },
];
static TITLED: [Param; 4] = [
    Param {
        name: "title",
        kind: STR,
        default: None,
        ty: "",
    },
    Param {
        name: "author",
        kind: STR,
        default: None,
        ty: "",
    },
    Param {
        name: "pageCount",
        kind: INT,
        default: None,
        ty: "",
    },
    Param {
        name: "writable",
        kind: BOOL,
        default: None,
        ty: "",
    },
];

macro_rules! book_ctors {
    ($($table:ident, $ty:ident, $gid:expr, $pages:expr;)*) => {$(
        static $table: [Ctor; 3] = [
            Ctor {
                access: None,
                params: &[],
                make: |w, _| EntityId::Item(new_book(w, &$ty, $gid, None, None, $pages, true)),
            },
            Ctor {
                access: None,
                params: &PAGES_WRITABLE,
                make: |w, v| EntityId::Item(new_book(w, &$ty, $gid, None, None, v[0].i32(), v[1].bool())),
            },
            Ctor {
                access: None,
                params: &TITLED,
                make: |w, v| {
                    let (t, a) = (v[0].string(), v[1].string());
                    EntityId::Item(new_book(w, &$ty, $gid, t.as_deref(), a.as_deref(), v[2].i32(), v[3].bool()))
                },
            },
        ];
    )*};
}

book_ctors! {
    C_RED, RED_BOOK, 0xFF1, 20;
    C_BLUE, BLUE_BOOK, 0xFF2, 40;
    C_BROWN, BROWN_BOOK, 0xFEF, 20;
    C_TAN, TAN_BOOK, 0xFF0, 20;
}

/// Adds the overload tables.
pub fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.RedBook", &C_RED);
    add_ctors(w, "Server.Items.BlueBook", &C_BLUE);
    add_ctors(w, "Server.Items.BrownBook", &C_BROWN);
    add_ctors(w, "Server.Items.TanBook", &C_TAN);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_content::props::Value;

    #[test]
    fn character_creation_red_book_has_title_author_and_pages() {
        let mut w = World::new();
        configure(&mut w);
        let args = [
            Value::Str("a book".into()),
            Value::Str("Zed".into()),
            Value::Int(20),
            Value::Bool(true),
        ];
        let id = muo_content::factory::make_item_args(&mut w, "Server.Items.RedBook", &args)
            .expect("the four-argument overload");
        let d = super::super::data(&w, id).unwrap();
        assert_eq!(
            (
                d.title.as_deref(),
                d.author.as_deref(),
                d.pages.len(),
                d.writable
            ),
            (Some("a book"), Some("Zed"), 20, true)
        );
    }
}
