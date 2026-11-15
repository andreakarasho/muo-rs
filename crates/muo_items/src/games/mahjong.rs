//! Mahjong: `MahjongGame` and what it owns (`MahjongTile`, `MahjongDealerIndicator`,
//! `MahjongWallBreakIndicator`, `MahjongDices`, `MahjongPlayers`), the piece geometry (`MahjongPieceDim`), the
//! tile generator, and the 0xDA packets (`MahjongPackets`).
//!
//! The sub-objects are plain structs inside one [`GameData`] in the item's `Ext`; their `Serialize`/`Deserialize`
//! are the nested records of the generated `MahjongGame` (a version int each).

use muo_content::props::{Kind, PropDef, Value};
use muo_content::utilities::types::PropType;
use muo_io::{GenericReader, GenericWriter, SpanReader};
use muo_primitives::random::Rng;
use muo_primitives::{DateTime, Point2D, Serial, TimeSpan, utility};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::packets::Pk;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

use crate::misc::item_leaf_raw;

pub const MAX_PLAYERS: usize = 4;
pub const BASE_SCORE: i32 = 30000;

// ---- MahjongPieceDim ----

/// `MahjongPieceDim`.
#[derive(Copy, Clone, Debug)]
pub struct PieceDim {
    pub pos: Point2D,
    pub width: i32,
    pub height: i32,
}

impl PieceDim {
    pub fn is_valid(&self) -> bool {
        self.pos.x >= 0
            && self.pos.y >= 0
            && self.pos.x + self.width <= 670
            && self.pos.y + self.height <= 670
    }

    pub fn is_overlapping(&self, d: &PieceDim) -> bool {
        self.pos.x < d.pos.x + d.width
            && self.pos.y < d.pos.y + d.height
            && self.pos.x + self.width > d.pos.x
            && self.pos.y + self.height > d.pos.y
    }

    pub fn hand_area(&self) -> i32 {
        let (p, w, h) = (self.pos, self.width, self.height);
        if p.x + w > 150 && p.x < 520 && p.y < 35 {
            return 0;
        }
        if p.x + w > 635 && p.y + h > 150 && p.y < 520 {
            return 1;
        }
        if p.x + w > 150 && p.x < 520 && p.y + h > 635 {
            return 2;
        }
        if p.x < 35 && p.y + h > 150 && p.y < 520 {
            return 3;
        }
        -1
    }
}

// ---- pieces ----

/// `MahjongTile`.
#[derive(Clone, Debug, Default)]
pub struct Tile {
    pub number: i32,
    /// `MahjongTileType` (1 to 34).
    pub value: i32,
    pub position: Point2D,
    pub stack_level: i32,
    /// `MahjongPieceDirection`: Up, Left, Down, Right.
    pub direction: i32,
    pub flipped: bool,
}

impl Tile {
    pub fn dimensions(&self) -> PieceDim {
        tile_dimensions(self.position, self.direction)
    }
}

fn tile_dimensions(position: Point2D, direction: i32) -> PieceDim {
    if direction == 0 || direction == 2 {
        PieceDim {
            pos: position,
            width: 20,
            height: 30,
        }
    } else {
        PieceDim {
            pos: position,
            width: 30,
            height: 20,
        }
    }
}

/// `MahjongDealerIndicator`.
#[derive(Clone, Debug, Default)]
pub struct DealerIndicator {
    pub position: Point2D,
    pub direction: i32,
    /// `MahjongWind`: North, East, South, West.
    pub wind: i32,
}

/// `MahjongPlayers`.
#[derive(Clone, Debug, Default)]
pub struct Players {
    pub players: Vec<Option<MobileId>>,
    pub in_game: Vec<bool>,
    pub public_hand: Vec<bool>,
    pub scores: Vec<i32>,
    pub dealer_position: i32,
    pub spectators: Vec<MobileId>,
}

/// The state of a `MahjongGame`.
#[derive(Clone, Debug, Default)]
pub struct GameData {
    pub level: i32,
    pub tiles: Vec<Tile>,
    pub dealer_indicator: DealerIndicator,
    pub wall_break: Point2D,
    pub dices: (i32, i32),
    pub players: Players,
    pub show_scores: bool,
    pub spectator_vision: bool,
    pub last_reset: DateTime,
}

fn game(w: &World, id: ItemId) -> Option<&GameData> {
    w.item(id)?.ext.get::<GameData>()
}

fn game_mut(w: &mut World, id: ItemId) -> Option<&mut GameData> {
    w.item_mut(id)?.ext.get_mut::<GameData>()
}

fn g(w: &World, id: ItemId) -> GameData {
    game(w, id).cloned().unwrap_or_default()
}

pub fn is_game(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&MAHJONG_GAME))
}

// ---- MahjongTileTypeGenerator, walls ----

/// `new MahjongTileTypeGenerator()`: 4 of each of the 34 types, shuffled four times.
fn tile_types(w: &mut World) -> Vec<i32> {
    let mut v = Vec::with_capacity(136);
    for t in 1..=34 {
        v.extend([t; 4]);
    }
    for _ in 0..4 {
        w.rng.shuffle(&mut v);
    }
    v
}

#[allow(clippy::too_many_arguments)]
fn build_wall(
    tiles: &mut [Tile],
    index: &mut usize,
    x: i32,
    y: i32,
    horizontal: bool,
    stack: i32,
    dir: i32,
    types: &mut std::slice::Iter<i32>,
) {
    for i in 0..17 {
        let position = if horizontal {
            Point2D::new(x + i * 20, y)
        } else {
            Point2D::new(x, y + i * 20)
        };
        tiles[*index + i as usize] = Tile {
            number: *index as i32 + i,
            value: *types.next().unwrap_or(&1),
            position,
            stack_level: stack,
            direction: dir,
            flipped: false,
        };
    }
    *index += 17;
}

/// `BuildWalls()`.
fn build_walls(w: &mut World) -> Vec<Tile> {
    let types = tile_types(w);
    let mut it = types.iter();
    let mut tiles = vec![Tile::default(); 136];
    let mut i = 0;
    build_wall(&mut tiles, &mut i, 165, 110, true, 0, 0, &mut it);
    build_wall(&mut tiles, &mut i, 165, 115, true, 1, 0, &mut it);
    build_wall(&mut tiles, &mut i, 530, 165, false, 0, 1, &mut it);
    build_wall(&mut tiles, &mut i, 525, 165, false, 1, 1, &mut it);
    build_wall(&mut tiles, &mut i, 165, 530, true, 0, 2, &mut it);
    build_wall(&mut tiles, &mut i, 165, 525, true, 1, 2, &mut it);
    build_wall(&mut tiles, &mut i, 110, 165, false, 0, 3, &mut it);
    build_wall(&mut tiles, &mut i, 115, 165, false, 1, 3, &mut it);
    tiles
}

fn new_players() -> Players {
    Players {
        players: vec![None; MAX_PLAYERS],
        in_game: vec![false; MAX_PLAYERS],
        public_hand: vec![false; MAX_PLAYERS],
        scores: vec![BASE_SCORE; MAX_PLAYERS],
        dealer_position: 0,
        spectators: Vec::new(),
    }
}

/// `MahjongGame()`.
pub fn mahjong_game_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = crate::simple::from_template(w, ty, "Server.Engines.Mahjong.MahjongGame");
    let tiles = build_walls(w);
    let first = utility::random_from(&mut w.rng, 1, 6);
    let second = utility::random_from(&mut w.rng, 1, 6);
    let now = w.now;
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(GameData {
            level: 1, // SecureLevel.CoOwners
            tiles,
            dealer_indicator: DealerIndicator {
                position: Point2D::new(300, 300),
                direction: 0,
                wind: 0,
            },
            wall_break: Point2D::new(335, 335),
            dices: (first, second),
            players: new_players(),
            show_scores: false,
            spectator_vision: false,
            last_reset: now,
        });
    }
    id
}

// ---- MahjongPlayers ----

fn in_game_player(p: &Players, index: i32) -> bool {
    index >= 0
        && (index as usize) < p.players.len()
        && p.players[index as usize].is_some()
        && p.in_game[index as usize]
}

fn player_index(p: &Players, m: MobileId) -> i32 {
    p.players
        .iter()
        .position(|x| *x == Some(m))
        .map_or(-1, |i| i as i32)
}

fn is_in_game_player_mobile(p: &Players, m: MobileId) -> bool {
    in_game_player(p, player_index(p, m))
}

fn dealer(p: &Players) -> Option<MobileId> {
    p.players.get(p.dealer_position as usize).copied().flatten()
}

fn is_in_game_dealer(p: &Players, m: MobileId) -> bool {
    dealer(p) == Some(m)
        && p.in_game
            .get(p.dealer_position as usize)
            .copied()
            .unwrap_or(false)
}

fn in_game_mobiles(p: &Players, players: bool, spectators: bool) -> Vec<MobileId> {
    let mut list = Vec::new();
    if players {
        for i in 0..p.players.len() {
            if in_game_player(p, i as i32) {
                list.extend(p.players[i]);
            }
        }
    }
    if spectators {
        list.extend(p.spectators.iter().copied());
    }
    list
}

fn name_of(w: &World, m: MobileId) -> String {
    w.mobile(m)
        .and_then(|x| (x.ty.name)(w, m))
        .unwrap_or_default()
}

fn send_players_localized(w: &mut World, id: ItemId, number: i32, args: &str) {
    for m in in_game_mobiles(&g(w, id).players, true, true) {
        muo_server::mobile::part2::messages::send_localized_message(w, m, number, args, 0x3B2);
    }
}

fn update_dealer(w: &mut World, id: ItemId, message: bool) {
    let p = g(w, id).players;
    if in_game_player(&p, p.dealer_position) {
        return;
    }
    let n = p.players.len() as i32;
    let found = (p.dealer_position + 1..n)
        .chain(0..p.dealer_position)
        .find(|i| in_game_player(&p, *i));
    if let Some(i) = found {
        if let Some(d) = game_mut(w, id) {
            d.players.dealer_position = i;
        }
        if message {
            send_dealer_changed(w, id);
        }
    }
}

fn send_dealer_changed(w: &mut World, id: ItemId) {
    if let Some(d) = dealer(&g(w, id).players) {
        let n = name_of(w, d);
        send_players_localized(w, id, 1062698, &n); // ~1_name~ is assigned the dealer.
    }
}

fn send_player_exit(w: &mut World, id: ItemId, who: MobileId) {
    let n = name_of(w, who);
    send_players_localized(w, id, 1062762, &n); // ~1_name~ has left the game.
}

fn next_seat(p: &Players) -> i32 {
    let n = p.players.len() as i32;
    (p.dealer_position..n)
        .chain(0..p.dealer_position)
        .find(|i| p.players[*i as usize].is_none())
        .unwrap_or(-1)
}

fn accessible(w: &mut World, id: ItemId, m: MobileId) -> bool {
    match w.item(id).map(|i| i.ty) {
        Some(ty) => (ty.is_accessible_to)(w, id, m),
        None => false,
    }
}

fn near_game(w: &World, id: ItemId, m: MobileId) -> bool {
    let loc = ops::get_world_location(w, id);
    w.mobile(m).map(|x| x.map) == w.item(id).map(|i| i.map) && deps::mobile_in_range(w, m, loc, 5)
}

/// `CheckPlayers()`.
pub fn check_players(w: &mut World, id: ItemId) {
    let Some(serial) = w.item(id).map(|i| i.serial) else {
        return;
    };
    let mut removed = false;
    let seats = g(w, id).players.players.len();
    for i in 0..seats {
        let p = g(w, id).players;
        let Some(player) = p.players[i] else { continue };
        let deleted = w.mobile(player).is_none_or(|m| m.deleted);
        if deleted {
            if let Some(d) = game_mut(w, id) {
                d.players.players[i] = None;
            }
            send_player_exit(w, id, player);
            update_dealer(w, id, true);
            removed = true;
        } else if p.in_game[i] {
            let ns = w.mobile(player).and_then(|m| m.net_state);
            if ns.is_none() {
                if let Some(d) = game_mut(w, id) {
                    d.players.in_game[i] = false;
                }
                send_player_exit(w, id, player);
                update_dealer(w, id, true);
                removed = true;
            } else if !accessible(w, id, player) || !near_game(w, id, player) {
                if let Some(d) = game_mut(w, id) {
                    d.players.in_game[i] = false;
                }
                send_relieve(w, ns, serial);
                send_player_exit(w, id, player);
                update_dealer(w, id, true);
                removed = true;
            }
        }
    }
    let mut i = 0;
    loop {
        let specs = g(w, id).players.spectators;
        let Some(&m) = specs.get(i) else { break };
        let ns = w.mobile(m).and_then(|x| x.net_state);
        if ns.is_none() || w.mobile(m).is_none_or(|x| x.deleted) {
            remove_spectator_at(w, id, i);
        } else if !accessible(w, id, m) || !near_game(w, id, m) {
            remove_spectator_at(w, id, i);
            send_relieve(w, ns, serial);
        } else {
            i += 1;
        }
    }
    if removed && !update_spectators(w, id) {
        send_players_packet(w, id, true, true);
    }
}

fn remove_spectator_at(w: &mut World, id: ItemId, i: usize) {
    if let Some(d) = game_mut(w, id) {
        d.players.spectators.remove(i);
    }
}

fn update_spectators(w: &mut World, id: ItemId) -> bool {
    let p = g(w, id).players;
    if p.spectators.is_empty() {
        return false;
    }
    let seat = next_seat(&p);
    if seat >= 0 {
        let new_player = p.spectators[0];
        remove_spectator_at(w, id, 0);
        add_player(w, id, new_player, seat as usize, false);
        update_spectators(w, id);
        return true;
    }
    false
}

fn add_player(w: &mut World, id: ItemId, player: MobileId, index: usize, send_join: bool) {
    if let Some(d) = game_mut(w, id) {
        d.players.players[index] = Some(player);
        d.players.in_game[index] = true;
    }
    update_dealer(w, id, false);
    let ns = w.mobile(player).and_then(|m| m.net_state);
    let serial = w.item(id).map_or(Serial::ZERO, |i| i.serial);
    if send_join {
        send_join_game(w, ns, serial);
    }
    send_players_packet(w, id, true, true);
    send_general_info(w, ns, id);
    send_tiles_info(w, ns, id, player);
    let n = name_of(w, player);
    if g(w, id).players.dealer_position == index as i32 {
        send_players_localized(w, id, 1062773, &n); // ~1_name~ has entered the game as the dealer.
    } else {
        send_players_localized(w, id, 1062772, &n); // ~1_name~ has entered the game as a player.
    }
}

fn add_spectator(w: &mut World, id: ItemId, m: MobileId) {
    if !g(w, id).players.spectators.contains(&m)
        && let Some(d) = game_mut(w, id)
    {
        d.players.spectators.push(m);
    }
    let ns = w.mobile(m).and_then(|x| x.net_state);
    let serial = w.item(id).map_or(Serial::ZERO, |i| i.serial);
    send_join_game(w, ns, serial);
    send_players_info(w, ns, id, m);
    send_general_info(w, ns, id);
    send_tiles_info(w, ns, id, m);
}

/// `Players.Join(mobile)`.
pub fn join(w: &mut World, id: ItemId, m: MobileId) {
    let p = g(w, id).players;
    let index = player_index(&p, m);
    if index >= 0 {
        add_player(w, id, m, index as usize, true);
        return;
    }
    let seat = next_seat(&p);
    if seat >= 0 {
        add_player(w, id, m, seat as usize, true);
    } else {
        add_spectator(w, id, m);
    }
}

fn leave_game(w: &mut World, id: ItemId, player: MobileId) {
    let p = g(w, id).players;
    let index = player_index(&p, player);
    if index >= 0 {
        if let Some(d) = game_mut(w, id) {
            d.players.in_game[index as usize] = false;
        }
        send_player_exit(w, id, player);
        update_dealer(w, id, true);
        send_players_packet(w, id, true, true);
    } else if let Some(d) = game_mut(w, id) {
        d.players.spectators.retain(|s| *s != player);
    }
}

fn set_public(w: &mut World, id: ItemId, index: i32, value: bool) {
    let d = g(w, id);
    if index < 0
        || index as usize >= d.players.public_hand.len()
        || d.players.public_hand[index as usize] == value
    {
        return;
    }
    if let Some(dm) = game_mut(w, id) {
        dm.players.public_hand[index as usize] = value;
    }
    send_tiles_packet(w, id, true, !d.spectator_vision);
    if in_game_player(&d.players, index)
        && let Some(p) = d.players.players[index as usize]
    {
        muo_server::mobile::part2::messages::send_localized_message(
            w,
            p,
            if value { 1062775 } else { 1062776 },
            "",
            0x3B2,
        );
    }
}

fn reset_scores(w: &mut World, id: ItemId, value: i32) {
    if let Some(d) = game_mut(w, id) {
        d.players.scores.fill(value);
    }
    let show = g(w, id).show_scores;
    send_players_packet(w, id, true, show);
    send_players_localized(w, id, 1062697, ""); // The dealer redistributes the score sticks evenly.
}

fn transfer_score(w: &mut World, id: ItemId, from: MobileId, to_position: i32, amount: i32) {
    let d = g(w, id);
    let from_pos = player_index(&d.players, from);
    let to = d
        .players
        .players
        .get(to_position.max(0) as usize)
        .copied()
        .flatten()
        .filter(|_| to_position >= 0);
    if from_pos < 0 || to.is_none() || d.players.scores[from_pos as usize] < amount {
        return;
    }
    let to = to.expect("checked");
    if let Some(dm) = game_mut(w, id) {
        dm.players.scores[from_pos as usize] =
            dm.players.scores[from_pos as usize].wrapping_sub(amount);
        dm.players.scores[to_position as usize] =
            dm.players.scores[to_position as usize].wrapping_add(amount);
    }
    if d.show_scores {
        send_players_packet(w, id, true, true);
    } else {
        let (fns, tns) = (
            w.mobile(from).and_then(|m| m.net_state),
            w.mobile(to).and_then(|m| m.net_state),
        );
        send_players_info(w, fns, id, from);
        send_players_info(w, tns, id, to);
    }
    // ~1_giver~ gives ~2_receiver~ ~3_number~ points.
    let args = format!("{}\t{}\t{}", name_of(w, from), name_of(w, to), amount);
    send_players_localized(w, id, 1062774, &args);
}

fn open_seat(w: &mut World, id: ItemId, index: i32) {
    let d = g(w, id);
    let Some(player) = d
        .players
        .players
        .get(index.max(0) as usize)
        .copied()
        .flatten()
        .filter(|_| index >= 0)
    else {
        return;
    };
    if d.players.in_game[index as usize] {
        let ns = w.mobile(player).and_then(|m| m.net_state);
        send_relieve(w, ns, w.item(id).map_or(Serial::ZERO, |i| i.serial));
    }
    if let Some(dm) = game_mut(w, id) {
        dm.players.players[index as usize] = None;
    }
    let n = name_of(w, player);
    send_players_localized(w, id, 1062699, &n); // ~1_name~ is relieved from the game by the dealer.
    update_dealer(w, id, true);
    if !update_spectators(w, id) {
        send_players_packet(w, id, true, true);
    }
}

fn assign_dealer(w: &mut World, id: ItemId, index: i32) {
    let d = g(w, id);
    let to = d
        .players
        .players
        .get(index.max(0) as usize)
        .copied()
        .flatten()
        .filter(|_| index >= 0);
    let Some(to) = to else { return };
    if !d.players.in_game[index as usize] {
        return;
    }
    let old = d.players.dealer_position;
    if let Some(dm) = game_mut(w, id) {
        dm.players.dealer_position = index;
    }
    if in_game_player(&d.players, old)
        && let Some(op) = d.players.players[old as usize]
    {
        let ns = w.mobile(op).and_then(|m| m.net_state);
        send_players_info(w, ns, id, op);
    }
    let tns = w.mobile(to).and_then(|m| m.net_state);
    send_players_info(w, tns, id, to);
    send_dealer_changed(w, id);
}

// ---- game operations ----

/// `ResetGame(from)`.
pub fn reset_game(w: &mut World, id: ItemId, from: Option<MobileId>) {
    if w.now - g(w, id).last_reset < TimeSpan::from_secs(5) {
        return;
    }
    let now = w.now;
    if let Some(d) = game_mut(w, id) {
        d.last_reset = now;
    }
    if let Some(f) = from {
        let n = name_of(w, f);
        send_players_localized(w, id, 1062771, &n); // ~1_name~ has reset the game.
    }
    send_relieve_packet(w, id, true, true);
    let tiles = build_walls(w);
    if let Some(d) = game_mut(w, id) {
        d.tiles = tiles;
        d.dealer_indicator = DealerIndicator {
            position: Point2D::new(300, 300),
            direction: 0,
            wind: 0,
        };
        d.wall_break = Point2D::new(335, 335);
        d.players = new_players();
    }
}

/// `ResetWalls(from)`.
pub fn reset_walls(w: &mut World, id: ItemId, from: Option<MobileId>) {
    if w.now - g(w, id).last_reset < TimeSpan::from_secs(5) {
        return;
    }
    let now = w.now;
    let tiles = build_walls(w);
    if let Some(d) = game_mut(w, id) {
        d.last_reset = now;
        d.tiles = tiles;
    }
    send_tiles_packet(w, id, true, true);
    if from.is_some() {
        send_players_localized(w, id, 1062696, ""); // The dealer rebuilds the wall.
    }
}

/// `GetStackLevel(dim)`.
pub fn get_stack_level(d: &GameData, dim: &PieceDim) -> i32 {
    let mut level = -1;
    for t in &d.tiles {
        if t.stack_level > level && dim.is_overlapping(&t.dimensions()) {
            level = t.stack_level;
        }
    }
    level
}

/// `MahjongTile.Move(position, direction, flip, validHandArea)`.
fn move_tile(
    w: &mut World,
    id: ItemId,
    number: usize,
    position: Point2D,
    direction: i32,
    flip: bool,
    valid_hand_area: i32,
) {
    let d = g(w, id);
    let Some(tile) = d.tiles.get(number) else {
        return;
    };
    let dim = tile_dimensions(position, direction);
    let cur = tile.dimensions().hand_area();
    let new = dim.hand_area();
    let movable = get_stack_level(&d, &tile.dimensions()) <= tile.stack_level;
    if !movable
        || !dim.is_valid()
        || (valid_hand_area >= 0
            && ((cur >= 0 && cur != valid_hand_area) || (new >= 0 && new != valid_hand_area)))
    {
        return;
    }
    // StackLevel = -1 first, so the tile does not interfere with itself
    if let Some(dm) = game_mut(w, id) {
        dm.tiles[number].position = position;
        dm.tiles[number].direction = direction;
        dm.tiles[number].stack_level = -1;
    }
    let d2 = g(w, id);
    let level = get_stack_level(&d2, &dim) + 1;
    if let Some(dm) = game_mut(w, id) {
        dm.tiles[number].stack_level = level;
        dm.tiles[number].flipped = flip;
    }
    send_tile_packet(w, id, number, true, true);
}

fn roll_dices(w: &mut World, id: ItemId, from: Option<MobileId>) {
    let first = utility::random_from(&mut w.rng, 1, 6);
    let second = utility::random_from(&mut w.rng, 1, 6);
    if let Some(d) = game_mut(w, id) {
        d.dices = (first, second);
    }
    send_general_packet(w, id, true, true);
    if let Some(f) = from {
        // ~1_name~ rolls the dice and gets a ~2_number~ and a ~3_number~!
        let args = format!("{}\t{first}\t{second}", name_of(w, f));
        send_players_localized(w, id, 1062695, &args);
    }
}

fn set_show_scores(w: &mut World, id: ItemId, value: bool) {
    if g(w, id).show_scores == value {
        return;
    }
    if let Some(d) = game_mut(w, id) {
        d.show_scores = value;
    }
    if value {
        send_players_packet(w, id, true, true);
    }
    send_general_packet(w, id, true, true);
    send_players_localized(w, id, if value { 1062777 } else { 1062778 }, ""); // The dealer has enabled/disabled score display.
}

fn set_spectator_vision(w: &mut World, id: ItemId, value: bool) {
    if g(w, id).spectator_vision == value {
        return;
    }
    if let Some(d) = game_mut(w, id) {
        d.spectator_vision = value;
    }
    ops::invalidate_properties(w, id);
    let p = g(w, id).players;
    if in_game_player(&p, p.dealer_position)
        && let Some(dm) = dealer(&p)
    {
        let ns = w.mobile(dm).and_then(|m| m.net_state);
        send_general_info(w, ns, id);
    }
    send_tiles_packet(w, id, false, true);
    send_players_localized(w, id, if value { 1062715 } else { 1062716 }, ""); // The dealer has enabled/disabled Spectator Vision.
}

// ---- packets ----

fn cannot_send(w: &World, ns: Option<NetStateId>) -> Option<NetStateId> {
    let ns = ns?;
    (!w.cannot_send_packets(ns)).then_some(ns)
}

fn send_join_game(w: &mut World, ns: Option<NetStateId>, serial: Serial) {
    let Some(ns) = cannot_send(w, ns) else { return };
    let mut p = Pk::new(9);
    p.u8(0xDA).u16(9).serial(serial).u16(0x19);
    let b = p.finish();
    w.send(ns, &b);
}

fn send_relieve(w: &mut World, ns: Option<NetStateId>, serial: Serial) {
    let Some(ns) = cannot_send(w, ns) else { return };
    let mut p = Pk::new(9);
    p.u8(0xDA).u16(9).serial(serial).u16(0x1A);
    let b = p.finish();
    w.send(ns, &b);
}

fn send_players_info(w: &mut World, ns: Option<NetStateId>, id: ItemId, to: MobileId) {
    let Some(ns) = cannot_send(w, ns) else { return };
    let (d, serial) = (g(w, id), w.item(id).map_or(Serial::ZERO, |i| i.serial));
    let seats = d.players.players.len();
    let mut p = Pk::new(11 + seats * 45);
    p.u8(0xDA).u16(0).serial(serial).u16(0x02).u16(0); // length and seat count patched below
    let mut count = 0u16;
    for i in 0..seats {
        let m = d.players.players[i];
        if m.is_none() && !d.show_scores {
            continue;
        }
        p.serial(
            m.and_then(|m| w.mobile(m))
                .map_or(Serial::ZERO, |x| x.serial),
        );
        p.u8(if m.is_some() && d.players.dealer_position == i as i32 {
            1
        } else {
            2
        });
        p.u8(i as u8);
        if d.show_scores || m == Some(to) {
            p.i32(d.players.scores[i]);
        } else {
            p.i32(0);
        }
        p.i16(0);
        p.u8(0);
        p.bool(d.players.public_hand[i]);
        let name = m.map(|m| name_of(w, m)).unwrap_or_default();
        let _ = p.write_latin1_fixed(&name, 30);
        p.bool(m.is_none() || !in_game_player(&d.players, i as i32));
        count += 1;
    }
    let mut out = p.finish_with_length();
    out[9..11].copy_from_slice(&count.to_be_bytes());
    w.send(ns, &out);
}

fn tile_visible_value(d: &GameData, t: &Tile, to: MobileId) -> u8 {
    if t.flipped {
        let hand = t.dimensions().hand_area();
        let player_at = if hand >= 0 {
            d.players.players.get(hand as usize).copied().flatten()
        } else {
            None
        };
        if hand < 0
            || d.players
                .public_hand
                .get(hand as usize)
                .copied()
                .unwrap_or(false)
            || player_at == Some(to)
            || (d.spectator_vision && d.players.spectators.contains(&to))
        {
            return t.value as u8;
        }
    }
    0
}

fn write_tile(p: &mut Pk, d: &GameData, t: &Tile, to: MobileId) {
    p.u8(t.number as u8);
    p.u8(tile_visible_value(d, t, to));
    p.i16(t.position.y as i16);
    p.i16(t.position.x as i16);
    p.u8(t.stack_level as u8);
    p.u8(t.direction as u8);
    p.u8(if t.flipped { 0x10 } else { 0 });
}

fn send_tile_info(w: &mut World, ns: Option<NetStateId>, id: ItemId, number: usize, to: MobileId) {
    let Some(ns) = cannot_send(w, ns) else { return };
    let (d, serial) = (g(w, id), w.item(id).map_or(Serial::ZERO, |i| i.serial));
    let Some(t) = d.tiles.get(number) else { return };
    let mut p = Pk::new(18);
    p.u8(0xDA).u16(18).serial(serial).u16(0x03);
    write_tile(&mut p, &d, t, to);
    let b = p.finish();
    w.send(ns, &b);
}

fn send_tiles_info(w: &mut World, ns: Option<NetStateId>, id: ItemId, to: MobileId) {
    let Some(ns) = cannot_send(w, ns) else { return };
    let (d, serial) = (g(w, id), w.item(id).map_or(Serial::ZERO, |i| i.serial));
    let length = 11 + d.tiles.len() * 9;
    let mut p = Pk::new(length);
    p.u8(0xDA)
        .u16(length as u16)
        .serial(serial)
        .u16(0x04)
        .i16(d.tiles.len() as i16);
    for t in &d.tiles {
        write_tile(&mut p, &d, t, to);
    }
    let b = p.finish();
    w.send(ns, &b);
}

fn general_info_bytes(w: &World, id: ItemId) -> Vec<u8> {
    let (d, serial) = (g(w, id), w.item(id).map_or(Serial::ZERO, |i| i.serial));
    let mut p = Pk::new(25);
    p.u8(0xDA).u16(25).serial(serial).u16(0x05).i16(0).u8(0);
    p.u8(u8::from(d.show_scores) | if d.spectator_vision { 2 } else { 0 });
    p.u8(d.dices.0 as u8)
        .u8(d.dices.1 as u8)
        .u8(d.dealer_indicator.wind as u8);
    p.i16(d.dealer_indicator.position.y as i16)
        .i16(d.dealer_indicator.position.x as i16);
    p.u8(d.dealer_indicator.direction as u8);
    p.i16(d.wall_break.y as i16).i16(d.wall_break.x as i16);
    p.finish()
}

fn send_general_info(w: &mut World, ns: Option<NetStateId>, id: ItemId) {
    let Some(ns) = cannot_send(w, ns) else { return };
    let b = general_info_bytes(w, id);
    w.send(ns, &b);
}

fn send_players_packet(w: &mut World, id: ItemId, players: bool, spectators: bool) {
    for m in in_game_mobiles(&g(w, id).players, players, spectators) {
        let ns = w.mobile(m).and_then(|x| x.net_state);
        send_players_info(w, ns, id, m);
    }
}

fn send_general_packet(w: &mut World, id: ItemId, players: bool, spectators: bool) {
    for m in in_game_mobiles(&g(w, id).players, players, spectators) {
        let ns = w.mobile(m).and_then(|x| x.net_state);
        if let Some(ns) = ns {
            let b = general_info_bytes(w, id);
            w.send(ns, &b);
        }
    }
}

fn send_tiles_packet(w: &mut World, id: ItemId, players: bool, spectators: bool) {
    for m in in_game_mobiles(&g(w, id).players, players, spectators) {
        let ns = w.mobile(m).and_then(|x| x.net_state);
        send_tiles_info(w, ns, id, m);
    }
}

fn send_tile_packet(w: &mut World, id: ItemId, number: usize, players: bool, spectators: bool) {
    for m in in_game_mobiles(&g(w, id).players, players, spectators) {
        let ns = w.mobile(m).and_then(|x| x.net_state);
        send_tile_info(w, ns, id, number, m);
    }
}

fn send_relieve_packet(w: &mut World, id: ItemId, players: bool, spectators: bool) {
    let serial = w.item(id).map_or(Serial::ZERO, |i| i.serial);
    for m in in_game_mobiles(&g(w, id).players, players, spectators) {
        let ns = w.mobile(m).and_then(|x| x.net_state);
        send_relieve(w, ns, serial);
    }
}

// ---- incoming (0xDA) ----

fn direction_of(v: u8) -> i32 {
    match v {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => 3,
    }
}

fn wind_of(v: u8) -> i32 {
    match v {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => 3,
    }
}

fn on_packet(w: &mut World, ns: &mut muo_net::net_state::NetState, body: &[u8]) {
    let nsid = muo_server::net_glue::ensure_net_state(w, ns);
    let Some(from) = w.net_state(nsid).and_then(|d| d.mobile) else {
        return;
    };
    let mut r = SpanReader::new(body);
    let Ok(serial) = r.read_u32() else { return };
    let game = w.find_item(Serial::new(serial)).filter(|i| is_game(w, *i));
    if let Some(gm) = game {
        check_players(w, gm);
    }
    let (Ok(_), Ok(cmd)) = (r.read_byte(), r.read_byte()) else {
        return;
    };
    let _ = sub_command(w, game, from, cmd, &mut r);
}

fn sub_command(
    w: &mut World,
    game: Option<ItemId>,
    from: MobileId,
    cmd: u8,
    r: &mut SpanReader,
) -> Option<()> {
    let id = game;
    let p = |w: &World| id.map(|i| g(w, i).players);
    let in_player = |w: &World| p(w).is_some_and(|p| is_in_game_player_mobile(&p, from));
    let in_dealer = |w: &World| p(w).is_some_and(|p| is_in_game_dealer(&p, from));
    match cmd {
        0x6 => leave_game(w, id?, from), // ExitGame
        0xA => {
            if !in_player(w) {
                return None;
            }
            let to = r.read_byte().ok()?;
            let amount = r.read_i32().ok()?;
            transfer_score(w, id?, from, i32::from(to), amount);
        }
        0xB => {
            if !in_player(w) {
                return None;
            }
            roll_dices(w, id?, Some(from));
        }
        0xC => {
            if !in_dealer(w) {
                return None;
            }
            reset_walls(w, id?, Some(from));
        }
        0xD => {
            if !in_dealer(w) {
                return None;
            }
            reset_scores(w, id?, BASE_SCORE);
        }
        0xF => {
            if !in_dealer(w) {
                return None;
            }
            let position = r.read_byte().ok()?;
            assign_dealer(w, id?, i32::from(position));
        }
        0x10 => {
            if !in_dealer(w) {
                return None;
            }
            let position = i32::from(r.read_byte().ok()?);
            let at = p(w)?.players.get(position as usize).copied().flatten();
            if at == Some(from) {
                return None;
            }
            open_seat(w, id?, position);
        }
        0x11 => {
            if !in_dealer(w) {
                return None;
            }
            r.read_i16().ok()?;
            r.read_byte().ok()?;
            let options = r.read_byte().ok()?;
            set_show_scores(w, id?, options & 1 != 0);
            set_spectator_vision(w, id?, options & 2 != 0);
        }
        0x15 => {
            if !in_dealer(w) {
                return None;
            }
            let y = i32::from(r.read_i16().ok()?);
            let x = i32::from(r.read_i16().ok()?);
            let pos = Point2D::new(x, y);
            if (PieceDim {
                pos,
                width: 20,
                height: 20,
            })
            .is_valid()
            {
                if let Some(d) = game_mut(w, id?) {
                    d.wall_break = pos;
                }
                send_general_packet(w, id?, true, true);
            }
        }
        0x16 => {
            if !in_player(w) {
                return None;
            }
            r.read_i16().ok()?;
            r.read_byte().ok()?;
            let public = r.read_byte().ok()? != 0;
            let index = player_index(&p(w)?, from);
            set_public(w, id?, index, public);
        }
        0x17 => {
            if !in_player(w) {
                return None;
            }
            let number = usize::from(r.read_byte().ok()?);
            if number >= g(w, id?).tiles.len() {
                return None;
            }
            r.read_byte().ok()?; // Current direction
            let direction = direction_of(r.read_byte().ok()?);
            r.read_byte().ok()?;
            let flip = r.read_byte().ok()? != 0;
            r.read_i16().ok()?; // Current Y
            r.read_i16().ok()?; // Current X
            r.read_byte().ok()?;
            let y = i32::from(r.read_i16().ok()?);
            let x = i32::from(r.read_i16().ok()?);
            r.read_byte().ok()?;
            let hand = player_index(&p(w)?, from);
            move_tile(w, id?, number, Point2D::new(x, y), direction, flip, hand);
        }
        0x18 => {
            if !in_dealer(w) {
                return None;
            }
            let direction = direction_of(r.read_byte().ok()?);
            let wind = wind_of(r.read_byte().ok()?);
            let y = i32::from(r.read_i16().ok()?);
            let x = i32::from(r.read_i16().ok()?);
            let pos = Point2D::new(x, y);
            let dim = if direction == 0 || direction == 2 {
                PieceDim {
                    pos,
                    width: 40,
                    height: 20,
                }
            } else {
                PieceDim {
                    pos,
                    width: 20,
                    height: 40,
                }
            };
            if dim.is_valid() {
                if let Some(d) = game_mut(w, id?) {
                    d.dealer_indicator = DealerIndicator {
                        position: pos,
                        direction,
                        wind,
                    };
                }
                send_general_packet(w, id?, true, true);
            }
        }
        _ => {}
    }
    Some(())
}

fn mahjong_configure(w: &mut World) {
    muo_content::network::register_packet(w, 0xDA, 0, true, false, on_packet);
    let d = muo_content::items::house_deps::deps_mut(w);
    d.securables
        .push(|w, id| is_game(w, id).then(|| g(w, id).level as u8));
    d.securable_setters.push(|w, id, level| {
        if !is_game(w, id) {
            return false;
        }
        if let Some(d) = game_mut(w, id) {
            d.level = i32::from(level);
        }
        true
    });
    muo_content::props::add(w, "Server.Engines.Mahjong.MahjongGame", MAHJONG_PROPS);
}

// ---- serialization ----

fn write_players(out: &mut dyn GenericWriter, w: &World, p: &Players) {
    out.write_i32(1);
    out.write_encoded_int(p.players.len() as i32);
    for m in &p.players {
        out.write_mobile(w, *m);
    }
    out.write_encoded_int(p.in_game.len() as i32);
    for b in &p.in_game {
        out.write_bool(*b);
    }
    out.write_encoded_int(p.public_hand.len() as i32);
    for b in &p.public_hand {
        out.write_bool(*b);
    }
    out.write_encoded_int(p.scores.len() as i32);
    for s in &p.scores {
        out.write_i32(*s);
    }
    out.write_i32(p.dealer_position);
}

fn read_players(w: &World, r: &mut dyn GenericReader) -> muo_io::Result<Players> {
    let version = r.read_i32()?;
    let mut p = Players::default();
    if version < 1 {
        let seats = r.read_i32()?.max(0);
        for _ in 0..seats {
            p.players.push(r.read_mobile(w)?);
            p.in_game.push(false);
            p.public_hand.push(r.read_bool()?);
            p.scores.push(r.read_i32()?);
        }
        p.dealer_position = r.read_i32()?;
        return Ok(p);
    }
    for _ in 0..r.read_encoded_int()?.max(0) {
        p.players.push(r.read_mobile(w)?);
    }
    for _ in 0..r.read_encoded_int()?.max(0) {
        p.in_game.push(r.read_bool()?);
    }
    for _ in 0..r.read_encoded_int()?.max(0) {
        p.public_hand.push(r.read_bool()?);
    }
    for _ in 0..r.read_encoded_int()?.max(0) {
        p.scores.push(r.read_i32()?);
    }
    p.dealer_position = r.read_i32()?;
    Ok(p)
}

fn serialize_game(w: &World, id: ItemId, out: &mut dyn GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(1);
    let d = g(w, id);
    out.write_enum(4, i64::from(d.level));
    out.write_encoded_int(d.tiles.len() as i32);
    for t in &d.tiles {
        out.write_i32(1);
        out.write_i32(t.number);
        out.write_enum(4, i64::from(t.value));
        out.write_point2d(t.position);
        out.write_i32(t.stack_level);
        out.write_enum(4, i64::from(t.direction));
        out.write_bool(t.flipped);
    }
    out.write_i32(1);
    out.write_point2d(d.dealer_indicator.position);
    out.write_enum(4, i64::from(d.dealer_indicator.direction));
    out.write_enum(4, i64::from(d.dealer_indicator.wind));
    out.write_i32(0);
    out.write_point2d(d.wall_break);
    out.write_i32(0);
    out.write_i32(d.dices.0);
    out.write_i32(d.dices.1);
    write_players(out, w, &d.players);
    out.write_bool(d.show_scores);
    out.write_bool(d.spectator_vision);
}

fn read_tile(r: &mut dyn GenericReader) -> muo_io::Result<Tile> {
    let version = r.read_i32()?;
    if version < 1 {
        return Ok(Tile {
            number: r.read_i32()?,
            value: r.read_i32()?,
            position: r.read_point2d()?,
            stack_level: r.read_i32()?,
            direction: r.read_i32()?,
            flipped: r.read_bool()?,
        });
    }
    Ok(Tile {
        number: r.read_i32()?,
        value: r.read_enum(4)? as i32,
        position: r.read_point2d()?,
        stack_level: r.read_i32()?,
        direction: r.read_enum(4)? as i32,
        flipped: r.read_bool()?,
    })
}

fn read_dealer_indicator(r: &mut dyn GenericReader) -> muo_io::Result<DealerIndicator> {
    let version = r.read_i32()?;
    if version < 1 {
        return Ok(DealerIndicator {
            position: r.read_point2d()?,
            direction: r.read_i32()?,
            wind: r.read_i32()?,
        });
    }
    Ok(DealerIndicator {
        position: r.read_point2d()?,
        direction: r.read_enum(4)? as i32,
        wind: r.read_enum(4)? as i32,
    })
}

fn deserialize_game(w: &mut World, id: ItemId, r: &mut dyn GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let mut d = GameData::default();
    if version < 1 {
        d.level = r.read_i32()?;
        let n = r.read_i32()?.max(0);
        for _ in 0..n {
            d.tiles.push(read_tile(r)?);
        }
    } else {
        d.level = r.read_enum(4)? as i32;
        let n = r.read_encoded_int()?.max(0);
        for _ in 0..n {
            d.tiles.push(read_tile(r)?);
        }
    }
    d.dealer_indicator = read_dealer_indicator(r)?;
    r.read_i32()?;
    d.wall_break = r.read_point2d()?;
    r.read_i32()?;
    d.dices = (r.read_i32()?, r.read_i32()?);
    d.players = read_players(w, r)?;
    d.show_scores = r.read_bool()?;
    d.spectator_vision = r.read_bool()?;
    d.last_reset = w.now; // AfterDeserialization
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(d);
    }
    Ok(())
}

// ---- the item ----

item_leaf_raw! {
    /// `Server.Engines.Mahjong.MahjongGame` `[SerializationGenerator(1, false)]`.
    MAHJONG_GAME = "Server.Engines.Mahjong.MahjongGame" : ITEM_BASE;
    construct: Some(|w| mahjong_game_new(w, &MAHJONG_GAME)),
    default_weight: crate::class_props::default_weight,
    get_properties: |w, id, list| {
        base::get_properties_base(w, id, list);
        if g(w, id).spectator_vision {
            list.add(1062717); // Spectator Vision Enabled
        } else {
            list.add(1062718); // Spectator Vision Disabled
        }
    },
    get_context_menu_entries: |w, id, from, list| {
        base::get_context_menu_entries_base(w, id, from, list);
        check_players(w, id);
        let alive = muo_server::mobile::base::alive(w, from);
        let house_ok = accessible(w, id, from);
        if alive && house_ok && in_game_mobiles(&g(w, id).players, true, false).is_empty() {
            let mut e = ContextMenuEntry::new(6162, -1);
            e.on_click = Some(std::rc::Rc::new(|w, from, target| {
                if let EntityId::Item(gm) = target
                    && is_game(w, gm)
                    && !ops::is_deleted(w, gm)
                    && muo_server::mobile::part2::stats::check_alive(w, from, true)
                    && accessible(w, gm, from)
                    && in_game_mobiles(&g(w, gm).players, true, false).is_empty()
                {
                    reset_game(w, gm, Some(from));
                }
            }));
            list.push(e);
        }
        if let Some(f) = muo_content::items::common::hooks(w).and_then(|h| h.secure_level_entry)
            && let Some(e) = f(w, from, id) {
                list.push(e);
            }
    },
    on_double_click: |w, id, from| {
        check_players(w, id);
        join(w, id, from);
    },
    serialize: |w, id, out| serialize_game(w, id, out),
    deserialize: |w, id, r| deserialize_game(w, id, r),
}

const SECURE_LEVELS: &[(&str, i64)] = &[
    ("Owner", 0),
    ("CoOwners", 1),
    ("Friends", 2),
    ("Anyone", 3),
    ("Guild", 4),
];

static MAHJONG_PROPS: &[PropDef] = &[
    PropDef {
        name: "Level",
        kind: Kind::Plain(PropType::Enum(SECURE_LEVELS)),
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Enum(i64::from(g(w, i).level)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            if let EntityId::Item(i) = e
                && let Some(d) = game_mut(w, i)
            {
                d.level = v.i32();
            }
            Ok(())
        }),
    },
    PropDef {
        name: "ShowScores",
        kind: Kind::Plain(PropType::Bool),
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(g(w, i).show_scores),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            if let EntityId::Item(i) = e {
                set_show_scores(w, i, v.bool());
            }
            Ok(())
        }),
    },
    PropDef {
        name: "SpectatorVision",
        kind: Kind::Plain(PropType::Bool),
        read: AccessLevel::GameMaster,
        write: AccessLevel::GameMaster,
        get: |w, e| match e {
            EntityId::Item(i) => Value::Bool(g(w, i).spectator_vision),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            if let EntityId::Item(i) = e {
                set_spectator_vision(w, i, v.bool());
            }
            Ok(())
        }),
    },
];

/// Registers `MahjongGame` and its 0xDA handler.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&MAHJONG_GAME, &[]);
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Engines.Mahjong.MahjongPackets.Configure",
        mahjong_configure,
    );
}

#[cfg(test)]
#[path = "mahjong_packet_tests.rs"]
mod packet_tests;
