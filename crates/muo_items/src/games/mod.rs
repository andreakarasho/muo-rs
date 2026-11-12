//! Board games: `BaseBoard` and the boards (`Backgammon`, `CheckerBoard`, `Chessboard`), `BasePiece` and
//! its 14 pieces, and `Dices`. Mahjong is in `mahjong.rs`.
//!
//! `BaseBoard` is an `ISecurable`: [`secure_level`] / [`set_secure_level`] are pushed on `HouseDeps` so the house
//! system's `SetSecureLevelEntry` offers it.

use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::container::{self, CONTAINER};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::mobile::AccessLevel;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

use crate::misc::{read_version, write_version};

pub mod mahjong;

#[cfg(test)]
mod tests;

// ---- BaseBoard ----

/// What `BaseBoard` adds to a `Container`.
#[derive(Clone, Debug, Default)]
pub struct BoardData {
    /// `SecureLevel` (`Owner` = 0), `[IgnoreDupe]`.
    pub level: i32,
}

/// `CreatePieces()`, the abstract member of `BaseBoard`.
pub struct BoardClass {
    pub create_pieces: fn(&mut World, ItemId),
}

pub fn board_level(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<BoardData>())
        .map_or(0, |d| d.level)
}

/// `BaseBoard(itemID)`: `CreatePieces()` runs at the end of the constructor, with the derived class's table.
pub fn board_new(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = container::new_container(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BoardData::default());
    }
    create_pieces(w, id);
    id
}

fn create_pieces(w: &mut World, id: ItemId) {
    if let Some(c) = w.item(id).and_then(|i| i.ty.family::<BoardClass>()) {
        (c.create_pieces)(w, id);
    }
}

/// `BaseBoard.CreatePiece(piece, x, y)`.
pub fn create_piece(w: &mut World, board: ItemId, piece: ItemId, x: i32, y: i32) {
    if let Some(ty) = w.item(board).map(|i| i.ty) {
        (ty.add_item)(w, board, piece);
    }
    ops::set_location(w, piece, Point3D::new(x, y, 0));
}

/// `BaseBoard.Reset()`.
pub fn reset(w: &mut World, board: ItemId) {
    let items = w.item(board).map(|i| i.items.clone()).unwrap_or_default();
    for item in items.into_iter().rev() {
        if let Some(ty) = w.item(item).map(|i| i.ty) {
            (ty.delete)(w, item);
        }
    }
    create_pieces(w, board);
}

/// `BaseBoard.ValidateDefault(from, board)`.
pub fn validate_default(w: &World, from: MobileId, board: ItemId) -> bool {
    if ops::is_deleted(w, board) {
        return false;
    }
    let Some(m) = w.mobile(from) else {
        return false;
    };
    if m.access_level >= AccessLevel::GameMaster {
        return true;
    }
    if !muo_server::mobile::base::alive(w, from) {
        return false;
    }
    // board.IsChildOf(from.Backpack) || RootParent is not Mobile && same map && in range && house owner
    let in_pack = match m.backpack {
        Some(p) => ops::is_child_of(w, board, Some(EntityId::Item(p))),
        None => ops::is_child_of(w, board, None),
    };
    if in_pack {
        return true;
    }
    let root_is_mobile = matches!(ops::root_parent(w, board), Some(EntityId::Mobile(_)));
    let loc = ops::get_world_location(w, board);
    !root_is_mobile
        && w.item(board).map(|i| i.map) == Some(m.map)
        && deps::mobile_in_range(w, from, loc, 1)
        && muo_multis::house::access::find_house_at_item(w, board)
            .is_some_and(|h| muo_multis::house::access::is_owner(w, h, from))
}

fn is_piece_of(w: &World, piece: ItemId, board: ItemId) -> bool {
    w.item(piece).is_some_and(|i| i.ty.is(&BASE_PIECE)) && piece_board(w, piece) == Some(board)
}

fn board_get_context_menu_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    base::get_context_menu_entries_base(w, id, from, list);
    if validate_default(w, from, id) {
        let gm = w
            .mobile(from)
            .is_some_and(|m| m.access_level >= AccessLevel::GameMaster);
        let mut e = ContextMenuEntry::new(6162, if gm { -1 } else { 1 });
        e.on_click = Some(std::rc::Rc::new(|w, from, target| {
            if let EntityId::Item(board) = target
                && w.item(board).is_some_and(|i| i.ty.is(&BASE_BOARD))
                && validate_default(w, from, board)
            {
                reset(w, board);
            }
        }));
        list.push(e);
    }
    if let Some(f) = muo_content::items::common::hooks(w).and_then(|h| h.secure_level_entry)
        && let Some(e) = f(w, from, id)
    {
        list.push(e);
    }
}

fn board_on_drag_drop(w: &mut World, id: ItemId, from: MobileId, dropped: ItemId) -> bool {
    is_piece_of(w, dropped, id) && (CONTAINER.on_drag_drop)(w, id, from, dropped)
}

fn board_on_drag_drop_into(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    dropped: ItemId,
    point: Point3D,
) -> bool {
    if !(is_piece_of(w, dropped, id)
        && container::on_drag_drop_into_default(w, id, from, dropped, point))
    {
        return false;
    }
    let loc = ops::get_world_location(w, id);
    if ops::root_parent(w, id) == Some(EntityId::Mobile(from)) {
        muo_server::mobile::part2::effects::send_sound_at(w, from, 0x127, loc);
    } else if let Some(map) = w.item(id).and_then(|i| i.map) {
        for (ns, _) in deps::clients_in_range(w, map, loc, 2) {
            muo_server::packets::effect::send_sound_effect(w, ns, 0x127, loc);
        }
    }
    true
}

const BOARD_VERSION: i32 = 2;

fn serialize_board(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (CONTAINER.serialize)(w, id, out);
    out.write_i32(BOARD_VERSION);
    out.write_enum(4, i64::from(board_level(w, id)));
}

fn deserialize_board(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (CONTAINER.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let level = if version < BOARD_VERSION {
        // Deserialize(reader, version): only version 1 saved Level (an int).
        if version == 1 { r.read_i32()? } else { 0 }
    } else {
        r.read_enum(4)? as i32
    };
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(BoardData { level });
    }
    Ok(())
}

pub static BASE_BOARD_CLASS: BoardClass = BoardClass {
    create_pieces: |_, _| {},
};

/// `Server.Items.BaseBoard` (abstract: `construct` is `None`).
pub static BASE_BOARD: ItemType = ItemType {
    full_name: "Server.Items.BaseBoard",
    parent: Some(&CONTAINER),
    construct: None,
    family: Some(&BASE_BOARD_CLASS),
    contents_decay: |_, _| false, // pieces are part of the board
    default_weight: |_, _| 5.0,
    displays_content: |_, _| false, // Do not display (x items, y stones)
    is_deco_container: |_, _| false,
    decay_time: |_, _| TimeSpan::from_secs_f64(86_400.0),
    on_drag_drop: board_on_drag_drop,
    on_drag_drop_into: board_on_drag_drop_into,
    get_context_menu_entries: board_get_context_menu_entries,
    serialize: serialize_board,
    deserialize: deserialize_board,
    ..CONTAINER
};

macro_rules! board_leaf {
    (
        $(#[$m:meta])*
        $name:ident, $class:ident = $full:literal, graphic $graphic:literal;
        $(label $label:literal;)?
        pieces |$w:ident, $id:ident| $pieces:block
    ) => {
        pub static $class: BoardClass = BoardClass { create_pieces: |$w, $id| $pieces };

        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_BOARD),
            construct: Some(|w| board_new(w, &$name, $graphic)),
            family: Some(&$class),
            $(label_number: |_, _| $label,)?
            serialize: |w, id, out| {
                serialize_board(w, id, out);
                write_version(out, false);
            },
            deserialize: |w, id, r| {
                deserialize_board(w, id, r)?;
                read_version(r, false)?;
                Ok(())
            },
            ..BASE_BOARD
        };
    };
}

/// `new PieceX(this)` placed at `(x, y)`.
fn put(w: &mut World, board: ItemId, piece: &'static ItemType, x: i32, y: i32) {
    let p = piece_new(w, piece, board);
    create_piece(w, board, p, x, y);
}

board_leaf! {
    /// `Server.Items.Backgammon`; `[Flippable(0xE1C, 0xFAD)]`.
    BACKGAMMON, BACKGAMMON_CLASS = "Server.Items.Backgammon", graphic 0xE1C;
    pieces |w, id| {
        for i in 0..5 {
            put(w, id, &PIECE_WHITE_CHECKER, 42, 17 * i + 6);
            put(w, id, &PIECE_BLACK_CHECKER, 42, 17 * i + 119);
            put(w, id, &PIECE_BLACK_CHECKER, 142, 17 * i + 6);
            put(w, id, &PIECE_WHITE_CHECKER, 142, 17 * i + 119);
        }
        for i in 0..3 {
            put(w, id, &PIECE_BLACK_CHECKER, 108, 17 * i + 6);
            put(w, id, &PIECE_WHITE_CHECKER, 108, 17 * i + 153);
        }
        for i in 0..2 {
            put(w, id, &PIECE_WHITE_CHECKER, 223, 17 * i + 6);
            put(w, id, &PIECE_BLACK_CHECKER, 223, 17 * i + 170);
        }
    }
}

board_leaf! {
    /// `Server.Items.CheckerBoard`.
    CHECKER_BOARD, CHECKER_BOARD_CLASS = "Server.Items.CheckerBoard", graphic 0xFA6;
    label 1016449; // a checker board
    pieces |w, id| {
        for i in 0..4 {
            put(w, id, &PIECE_WHITE_CHECKER, 50 * i + 45, 25);
            put(w, id, &PIECE_WHITE_CHECKER, 50 * i + 70, 50);
            put(w, id, &PIECE_WHITE_CHECKER, 50 * i + 45, 75);
            put(w, id, &PIECE_BLACK_CHECKER, 50 * i + 70, 150);
            put(w, id, &PIECE_BLACK_CHECKER, 50 * i + 45, 175);
            put(w, id, &PIECE_BLACK_CHECKER, 50 * i + 70, 200);
        }
    }
}

board_leaf! {
    /// `Server.Items.Chessboard`.
    CHESSBOARD, CHESSBOARD_CLASS = "Server.Items.Chessboard", graphic 0xFA6;
    label 1016450; // a chessboard
    pieces |w, id| {
        for i in 0..8 {
            put(w, id, &PIECE_BLACK_PAWN, 67, 25 * i + 17);
            put(w, id, &PIECE_WHITE_PAWN, 192, 25 * i + 17);
        }
        // Rook
        put(w, id, &PIECE_BLACK_ROOK, 42, 5);
        put(w, id, &PIECE_BLACK_ROOK, 42, 180);
        put(w, id, &PIECE_WHITE_ROOK, 216, 5);
        put(w, id, &PIECE_WHITE_ROOK, 216, 180);
        // Knight
        put(w, id, &PIECE_BLACK_KNIGHT, 42, 30);
        put(w, id, &PIECE_BLACK_KNIGHT, 42, 155);
        put(w, id, &PIECE_WHITE_KNIGHT, 216, 30);
        put(w, id, &PIECE_WHITE_KNIGHT, 216, 155);
        // Bishop
        put(w, id, &PIECE_BLACK_BISHOP, 42, 55);
        put(w, id, &PIECE_BLACK_BISHOP, 42, 130);
        put(w, id, &PIECE_WHITE_BISHOP, 216, 55);
        put(w, id, &PIECE_WHITE_BISHOP, 216, 130);
        // Queen
        put(w, id, &PIECE_BLACK_QUEEN, 42, 105);
        put(w, id, &PIECE_WHITE_QUEEN, 216, 105);
        // King
        put(w, id, &PIECE_BLACK_KING, 42, 80);
        put(w, id, &PIECE_WHITE_KING, 216, 80);
    }
}

// ---- BasePiece ----

/// What `BasePiece` adds to an `Item`: `Board`.
#[derive(Clone, Debug, Default)]
pub struct PieceData {
    pub board: Option<ItemId>,
}

pub fn piece_board(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.ext.get::<PieceData>()?.board
}

/// `BasePiece(itemID, board)` of class `ty`.
pub fn piece_new_with_id(
    w: &mut World,
    ty: &'static ItemType,
    item_id: i32,
    board: ItemId,
) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(PieceData { board: Some(board) });
    }
    id
}

/// `new PieceX(board)`: the graphic is the class's `base(..)` argument, kept in [`PIECES`].
pub fn piece_new(w: &mut World, ty: &'static ItemType, board: ItemId) -> ItemId {
    let graphic = PIECES
        .iter()
        .find(|(t, ..)| std::ptr::eq(*t, ty))
        .map_or(0, |(_, g, _)| *g);
    piece_new_with_id(w, ty, graphic, board)
}

fn board_alive(w: &World, id: ItemId) -> Option<ItemId> {
    piece_board(w, id).filter(|b| !ops::is_deleted(w, *b))
}

/// `BasePiece.OnSingleClick`'s and `OnDragLift`'s first steps: delete a piece whose board is gone.
fn piece_orphan(w: &mut World, id: ItemId) -> bool {
    if board_alive(w, id).is_none() {
        if let Some(ty) = w.item(id).map(|i| i.ty) {
            (ty.delete)(w, id);
        }
        return true;
    }
    false
}

fn piece_is_child_of_board(w: &World, id: ItemId, board: ItemId) -> bool {
    ops::is_child_of(w, id, Some(EntityId::Item(board)))
}

fn piece_drop_back(w: &mut World, board: ItemId, id: ItemId) {
    if let Some(ty) = w.item(board).map(|i| i.ty) {
        (ty.drop_item)(w, board, id);
    }
}

/// `Server.Items.BasePiece`.
pub static BASE_PIECE: ItemType = ItemType {
    full_name: "Server.Items.BasePiece",
    parent: Some(&ITEM_BASE),
    // `BasePiece(itemID, board)` is the only constructor and is not [Constructible].
    construct: None,
    is_virtual_item: |_, _| true,
    can_target: |_, _| false,
    on_single_click: |w, id, from| {
        if piece_orphan(w, id) {
            return;
        }
        let board = piece_board(w, id).unwrap_or(id);
        if !piece_is_child_of_board(w, id, board) {
            piece_drop_back(w, board, id);
        } else {
            base::on_single_click_base(w, id, from);
        }
    },
    on_drag_lift: |w, id, _from| {
        if piece_orphan(w, id) {
            return false;
        }
        let board = piece_board(w, id).unwrap_or(id);
        if !piece_is_child_of_board(w, id, board) {
            piece_drop_back(w, board, id);
            return false;
        }
        true
    },
    drop_to_mobile: |_, _, _, _, _| false,
    drop_to_item: |w, id, from, target, p| {
        piece_board(w, id) == Some(target)
            && p.x != -1
            && p.y != -1
            && base::drop_to_item_base(w, id, from, target, p)
    },
    drop_to_world: |_, _, _, _| false,
    get_lift_sound: |_, _, _| -1,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
        out.write_item(w, piece_board(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        let board = r.read_item(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(PieceData { board });
        }
        // [AfterDeserialization(false)] -> Timer.DelayCall(AfterDeserialization)
        muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            let orphan =
                piece_board(w, id).is_none() || w.item(id).is_some_and(|i| i.parent.is_none());
            if orphan && let Some(ty) = w.item(id).map(|i| i.ty) {
                (ty.delete)(w, id);
            }
        });
        Ok(())
    },
    ..ITEM_BASE
};

macro_rules! pieces {
    ($($name:ident = $full:literal, $graphic:literal, $label:literal;)*) => {
        $(
            #[doc = concat!("`", $full, "`.")]
            pub static $name: ItemType = ItemType {
                full_name: $full,
                parent: Some(&BASE_PIECE),
                // `Piece..(BaseBoard board)` is not [Constructible]: only a board makes pieces.
                construct: None,
                default_name: |_, _| Some($label.to_string()),
                serialize: |w, id, out| {
                    (BASE_PIECE.serialize)(w, id, out);
                    write_version(out, false);
                },
                deserialize: |w, id, r| {
                    (BASE_PIECE.deserialize)(w, id, r)?;
                    read_version(r, false)?;
                    Ok(())
                },
                ..BASE_PIECE
            };
        )*

        /// Every piece with its graphic and name.
        pub static PIECES: &[(&ItemType, i32, &str)] = &[$((&$name, $graphic, $label),)*];
    };
}

pieces! {
    PIECE_WHITE_KING = "Server.Items.PieceWhiteKing", 0x3587, "white king";
    PIECE_BLACK_KING = "Server.Items.PieceBlackKing", 0x358E, "black king";
    PIECE_WHITE_QUEEN = "Server.Items.PieceWhiteQueen", 0x358A, "white queen";
    PIECE_BLACK_QUEEN = "Server.Items.PieceBlackQueen", 0x3591, "black queen";
    PIECE_WHITE_ROOK = "Server.Items.PieceWhiteRook", 0x3586, "white rook";
    PIECE_BLACK_ROOK = "Server.Items.PieceBlackRook", 0x358D, "black rook";
    PIECE_WHITE_BISHOP = "Server.Items.PieceWhiteBishop", 0x3585, "white bishop";
    PIECE_BLACK_BISHOP = "Server.Items.PieceBlackBishop", 0x358C, "black bishop";
    PIECE_WHITE_KNIGHT = "Server.Items.PieceWhiteKnight", 0x3588, "white knight";
    PIECE_BLACK_KNIGHT = "Server.Items.PieceBlackKnight", 0x358F, "black knight";
    PIECE_WHITE_PAWN = "Server.Items.PieceWhitePawn", 0x3589, "white pawn";
    PIECE_BLACK_PAWN = "Server.Items.PieceBlackPawn", 0x3590, "black pawn";
    PIECE_WHITE_CHECKER = "Server.Items.PieceWhiteChecker", 0x3584, "white checker";
    PIECE_BLACK_CHECKER = "Server.Items.PieceBlackChecker", 0x358B, "black checker";
}

// ---- Dices ----

/// `Dices.Roll(from)`.
pub fn roll(w: &mut World, id: ItemId, from: MobileId) {
    let a = utility::random_from(&mut w.rng, 1, 6);
    let b = utility::random_from(&mut w.rng, 1, 6);
    let name = w
        .mobile(from)
        .and_then(|m| (m.ty.name)(w, from))
        .unwrap_or_default();
    muo_server::item::messages::public_overhead_message(
        w,
        id,
        muo_server::packets::message::MessageType::REGULAR,
        0,
        false,
        &format!("*{name} rolls {a}, {b}*"),
    );
}

/// `Server.Items.Dices`.
pub static DICES: ItemType = ItemType {
    full_name: "Server.Items.Dices",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| ops::new_item_with_id(w, &DICES, 0xFA7)),
    default_weight: |_, _| 1.0,
    on_double_click: |w, id, from| {
        let loc = ops::get_world_location(w, id);
        if !deps::mobile_in_range(w, from, loc, 2) {
            return;
        }
        roll(w, id, from);
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

/// `[Flippable]` data: the classes and their item ids.
pub static FLIPPABLE: &[(&ItemType, &[i32])] = &[(&BACKGAMMON, &[0xE1C, 0xFAD])];

/// The classes of this folder that can be constructed.
pub static CONSTRUCTIBLE: &[&ItemType] = &[&BACKGAMMON, &CHECKER_BOARD, &CHESSBOARD, &DICES];

/// `Dices.OnTelekinesis(from)` (`ITelekinesisable`): true when `id` is a pair of dice.
fn dices_on_telekinesis(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !w.item(id).is_some_and(|i| i.ty.is(&DICES)) {
        return false;
    }
    let (loc, map) = w
        .item(id)
        .map_or((Point3D::default(), None), |i| (i.location, i.map));
    let e = muo_server::item::effect_item::create_targeted(
        w,
        loc,
        map,
        muo_server::item::effect_item::DEFAULT_DURATION,
    );
    muo_server::effects::send_location_particles(w, e, 0x376A, 9, 32, 5022);
    muo_server::effects::play_sound(w, loc, map, 0x1F5);
    roll(w, id, from);
    true
}

/// `(item as ISecurable)?.Level` of a board.
pub fn secure_level(w: &World, id: ItemId) -> Option<u8> {
    w.item(id)?
        .ty
        .is(&BASE_BOARD)
        .then(|| board_level(w, id) as u8)
}

/// `board.Level = level`.
pub fn set_secure_level(w: &mut World, id: ItemId, level: u8) -> bool {
    if !w.item(id).is_some_and(|i| i.ty.is(&BASE_BOARD)) {
        return false;
    }
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<BoardData>()) {
        d.level = i32::from(level);
    }
    true
}

fn games_configure(w: &mut World) {
    let d = muo_content::items::house_deps::deps_mut(w);
    d.securables.push(secure_level);
    d.securable_setters.push(set_secure_level);
    let h = muo_content::spells::hooks::hooks_mut(w);
    if h.item_on_telekinesis.is_none() {
        // BaseDoor and Dices are the `ITelekinesisable` classes that are not containers.
        h.item_is_telekinesisable = Some(|w, id| {
            crate::construction::doors::is_door(w, id)
                || w.item(id).is_some_and(|i| i.ty.is(&DICES))
        });
        h.item_on_telekinesis = Some(|w, id, from| {
            crate::construction::doors::on_telekinesis(w, id, from)
                || dices_on_telekinesis(w, id, from)
        });
    }
}

/// Registers the classes of this folder.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.BaseBoard.Configure",
        games_configure,
    );
    mahjong::register(types);
    types.register_item(&BASE_BOARD, &[]);
    for t in CONSTRUCTIBLE {
        types.register_item(t, &[]);
    }
    types.register_item(&BASE_PIECE, &[]);
    for (t, ..) in PIECES {
        types.register_item(t, &[]);
    }
}
