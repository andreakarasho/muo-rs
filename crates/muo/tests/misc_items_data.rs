//! The non-equipment item data moved out of Rust: book texts (`"bookContent"`), board piece layouts
//! (`"boardPieces"`) and the `(amountFrom, amountTo)` resource constructors (`content/items/amount_range.rhai`).

use muo_primitives::{BuiltInRng, utility};
use muo_server::World;
use muo_server::item::ItemType;

fn world() -> World {
    let mut w = World::new();
    muo::register_content(&mut w.types);
    w.core.base_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    muo_content::templates::reload(&mut w).unwrap();
    assert!(muo_content::scripting::reload_scripts(&mut w).is_empty());
    w
}

fn book_classes() -> Vec<&'static ItemType> {
    let mut v = muo_items::books::defined::ALL.to_vec();
    v.extend_from_slice(muo_engines::khaldun::books::ALL);
    v
}

#[test]
fn every_defined_book_has_its_text_and_saves_only_its_header() {
    let mut w = world();
    let classes = book_classes();
    assert_eq!(classes.len(), 34 + 28);
    for t in classes {
        let id = (t.construct.unwrap())(&mut w);
        let c = muo_items::books::default_content(&w, id)
            .unwrap_or_else(|| panic!("{}: no bookContent", t.full_name));
        let d = muo_items::books::data(&w, id).unwrap();
        assert!(!c.pages.is_empty(), "{}", t.full_name);
        assert!(c.is_match(&d.pages), "{}", t.full_name);
        assert_eq!(d.title.as_deref(), Some(c.title), "{}", t.full_name);
        assert_eq!(d.author.as_deref(), Some(c.author), "{}", t.full_name);
        assert!(!d.writable, "{}", t.full_name);
    }
}

#[test]
fn boards_lay_out_their_pieces() {
    let mut w = world();
    for (t, n) in [
        (&muo_items::games::BACKGAMMON, 30),
        (&muo_items::games::CHECKER_BOARD, 24),
        (&muo_items::games::CHESSBOARD, 32),
    ] {
        let id = (t.construct.unwrap())(&mut w);
        let pieces = w.item(id).unwrap().items.clone();
        assert_eq!(pieces.len(), n, "{}", t.full_name);
        for p in pieces {
            assert_eq!(muo_items::games::piece_board(&w, p), Some(id));
        }
        // Reset lays the same pieces out again
        muo_items::games::reset(&mut w, id);
        assert_eq!(w.item(id).unwrap().items.len(), n, "{}", t.full_name);
    }
}

/// The old `Foo(amountFrom, amountTo)`: `this(Utility.RandomMinMax(amountFrom, amountTo))`.
#[test]
fn amount_range_constructors_draw_the_amount_as_before() {
    let mut w = world();
    let names = [
        "BarkFragment",
        "BlueDiamond",
        "BrilliantAmber",
        "Corruption",
        "DarkSapphire",
        "DiseasedBark",
        "DreadHornMane",
        "EcruCitrine",
        "EyeOfTheTravesty",
        "FireRuby",
        "GrizzledBones",
        "LardOfParoxysmus",
        "Muculent",
        "ParasiticPlant",
        "PerfectEmerald",
        "Putrefication",
        "Scourge",
        "SwitchItem",
        "Taint",
        "Turquoise",
    ];
    for n in names {
        let full = format!("Server.Items.{n}");
        let ty = muo_content::props::item_ty(&w, &full);
        for (from, to) in [(1, 1), (2, 7), (10, 500)] {
            w.rng = BuiltInRng::from_seed(7);
            let want = utility::random_min_max(&mut w.rng, from, to);
            w.rng = BuiltInRng::from_seed(7);
            let id = muo_items::resources::leaves::amount_range_new(&mut w, ty, from, to);
            let plain = (ty.construct.unwrap())(&mut w);
            let (i, p) = (w.item(id).unwrap(), w.item(plain).unwrap());
            assert_eq!(i.amount, want, "{full} {from}..{to}");
            assert_eq!(
                (i.item_id, i.hue, i.flags),
                (p.item_id, p.hue, p.flags),
                "{full}"
            );
            assert_eq!(
                p.amount, 1,
                "{full}: the plain constructor keeps its amount"
            );
        }
    }
}
