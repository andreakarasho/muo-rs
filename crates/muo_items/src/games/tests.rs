//! Games: names are unique, boards place their pieces, saves round-trip and match the generated layout
//! (`BaseBoard` version 2 + `SecureLevel` after the container part; `BasePiece` version 0 + the board).

use std::collections::HashSet;

use muo_io::{SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{Item, ItemType};

use super::*;

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn reload(w: &mut World, t: &'static ItemType, bytes: &[u8]) -> ItemId {
    let id = w.add_item(Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0400),
        t,
    ));
    let mut r = SaveReader::new(bytes);
    (t.deserialize)(w, id, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
    assert_eq!(r.remaining(), 0, "{}", t.full_name);
    id
}

#[test]
fn names_are_unique() {
    let mut seen = HashSet::new();
    for t in [&BASE_BOARD, &BASE_PIECE]
        .into_iter()
        .chain(CONSTRUCTIBLE.iter().copied())
        .chain(PIECES.iter().map(|(t, ..)| *t))
    {
        assert!(seen.insert(t.full_name), "{} twice", t.full_name);
    }
    assert_eq!(seen.len(), 2 + 4 + 14);
}

#[test]
fn boards_create_their_pieces() {
    let expected = [(&BACKGAMMON, 30), (&CHECKER_BOARD, 24), (&CHESSBOARD, 32)];
    for (t, n) in expected {
        let mut w = crate::test_world();
        let id = (t.construct.unwrap())(&mut w);
        let items = w.item(id).unwrap().items.clone();
        assert_eq!(items.len(), n, "{}", t.full_name);
        assert!(items.iter().all(|p| piece_board(&w, *p) == Some(id)));
    }
    // Chessboard: first item is a black pawn at (67, 17).
    let mut w = crate::test_world();
    let id = (CHESSBOARD.construct.unwrap())(&mut w);
    let first = w.item(id).unwrap().items[0];
    assert_eq!(
        w.item(first).unwrap().ty.full_name,
        "Server.Items.PieceBlackPawn"
    );
    assert_eq!(w.item(first).unwrap().item_id, 0x3590);
    let l = w.item(first).unwrap().location;
    assert_eq!((l.x, l.y, l.z), (67, 17, 0));
}

#[test]
fn reset_rebuilds_the_pieces() {
    let mut w = crate::test_world();
    let id = (CHECKER_BOARD.construct.unwrap())(&mut w);
    let first = w.item(id).unwrap().items[0];
    reset(&mut w, id);
    assert_eq!(w.item(id).unwrap().items.len(), 24);
    assert!(w.item(first).is_none_or(|i| i.deleted()));
}

#[test]
fn board_and_piece_records() {
    let mut w = crate::test_world();
    let id = (DICES.construct.unwrap())(&mut w);
    let bytes = save(&w, id);
    assert_eq!(&bytes[bytes.len() - 4..], &[0, 0, 0, 0]);

    let id = (BACKGAMMON.construct.unwrap())(&mut w);
    let bytes = save(&w, id);
    // ..., BaseBoard version 2, SecureLevel.Owner (encoded 0), Backgammon version 0
    assert_eq!(&bytes[bytes.len() - 9..], &[2, 0, 0, 0, 0, 0, 0, 0, 0]);
    let again = reload(&mut w, &BACKGAMMON, &bytes);
    assert_eq!(save(&w, again), bytes);

    let piece = w.item(id).unwrap().items[0];
    let bytes = save(&w, piece);
    // Item part, version 0, board serial
    let n = bytes.len();
    assert_eq!(&bytes[n - 4..], &[0, 0, 0, 0]); // PieceWhiteChecker version
    let serial = w.item(id).unwrap().serial.value().to_le_bytes();
    assert_eq!(&bytes[n - 8..n - 4], &serial);
    assert_eq!(&bytes[n - 12..n - 8], &[0, 0, 0, 0]); // BasePiece version
    let again = reload(&mut w, &PIECE_WHITE_CHECKER, &bytes);
    assert_eq!(piece_board(&w, again), Some(id));
}

#[test]
fn old_board_version_one_reads_level() {
    let mut w = crate::test_world();
    let id = (CHESSBOARD.construct.unwrap())(&mut w);
    let mut out = SaveWriter::new(true);
    (CONTAINER.serialize)(&w, id, &mut out);
    let mut bytes = out.into_vec();
    bytes.extend([1, 0, 0, 0, 3, 0, 0, 0]); // BaseBoard version 1, Level = Anyone (an int)
    bytes.extend([0, 0, 0, 0]); // Chessboard version
    let loaded = reload(&mut w, &CHESSBOARD, &bytes);
    assert_eq!(board_level(&w, loaded), 3);
}
