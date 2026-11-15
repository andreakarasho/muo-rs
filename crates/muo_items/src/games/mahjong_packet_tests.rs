//! Mahjong packet tests: the expected packets are written out as byte builders and compared with the server's packet
//! writer.

use muo_server::mobile::MOBILE_BASE;
use muo_server::net_state::{NetOut, NetStateData};

use super::*;

fn net_world() -> (World, NetStateId) {
    let mut w = crate::test_world();
    let ns = NetStateId(1);
    w.net_states.insert(ns, NetStateData::new(0));
    (w, ns)
}

fn sent(w: &mut World) -> Vec<u8> {
    let mut all: Vec<Vec<u8>> = std::mem::take(&mut w.net_out)
        .into_iter()
        .filter_map(|o| match o {
            NetOut::Send(_, b) => Some(b),
            _ => None,
        })
        .collect();
    assert_eq!(all.len(), 1, "exactly one packet");
    all.remove(0)
}

/// A legacy variable `Packet(0xDA)`.
struct Packet(Vec<u8>);

impl Packet {
    fn new() -> Packet {
        Packet(vec![0xDA, 0, 0])
    }
    fn u8(&mut self, v: u8) -> &mut Self {
        self.0.push(v);
        self
    }
    fn i16(&mut self, v: i16) -> &mut Self {
        self.0.extend_from_slice(&v.to_be_bytes());
        self
    }
    fn i32(&mut self, v: i32) -> &mut Self {
        self.0.extend_from_slice(&v.to_be_bytes());
        self
    }
    fn serial(&mut self, s: Serial) -> &mut Self {
        self.0.extend_from_slice(&s.value().to_be_bytes());
        self
    }
    fn ascii_fixed(&mut self, s: &str, n: usize) -> &mut Self {
        let mut b = s.as_bytes().to_vec();
        b.resize(n, 0);
        self.0.extend_from_slice(&b);
        self
    }
    fn compile(mut self) -> Vec<u8> {
        let n = self.0.len() as u16;
        self.0[1..3].copy_from_slice(&n.to_be_bytes());
        self.0
    }
}

/// A game with `m` seated, `ShowScores`/`SpectatorVision` as given.
fn game_with(
    w: &mut World,
    show_scores: bool,
    spectator_vision: bool,
    join_m: bool,
) -> (ItemId, MobileId) {
    let m = w.new_mobile(&MOBILE_BASE);
    w.mobile_mut(m).unwrap().name = Some("Kamron".to_owned());
    let game = mahjong_game_new(w, &MAHJONG_GAME);
    {
        let d = game_mut(w, game).unwrap();
        d.show_scores = show_scores;
        d.spectator_vision = spectator_vision;
    }
    if join_m {
        join(w, game, m);
    }
    (game, m)
}

fn serial_of(w: &World, game: ItemId) -> Serial {
    w.item(game).unwrap().serial
}

#[test]
fn test_mahjong_join_game() {
    let (mut w, ns) = net_world();
    let game = Serial::new(0x1024);
    let mut p = Packet::new();
    p.serial(game).u8(0).u8(0x19);
    send_join_game(&mut w, Some(ns), game);
    assert_eq!(sent(&mut w), p.compile());
}

#[test]
fn test_mahjong_relieve() {
    let (mut w, ns) = net_world();
    let game = Serial::new(0x1024);
    let mut p = Packet::new();
    p.serial(game).u8(0).u8(0x1A);
    send_relieve(&mut w, Some(ns), game);
    assert_eq!(sent(&mut w), p.compile());
}

#[test]
fn test_mahjong_players_info() {
    for show_scores in [true, false] {
        let (mut w, ns) = net_world();
        let (game, m) = game_with(&mut w, show_scores, false, true);
        let d = game_ref(&w, game);
        let seats = d.players.players.len();

        // MahjongPlayersInfo
        let mut p = Packet::new();
        p.serial(serial_of(&w, game))
            .u8(0)
            .u8(2)
            .u8(0)
            .u8(seats as u8);
        let mut n = 0;
        for i in 0..seats {
            let mobile = d.players.players[i];
            if let Some(mob) = mobile {
                p.serial(w.mobile(mob).unwrap().serial);
                p.u8(if d.players.dealer_position == i as i32 {
                    1
                } else {
                    2
                });
                p.u8(i as u8);
                p.i32(if d.show_scores || mob == m {
                    d.players.scores[i]
                } else {
                    0
                });
                p.i16(0).u8(0);
                p.u8(u8::from(d.players.public_hand[i]));
                p.ascii_fixed(w.mobile(mob).unwrap().name.as_deref().unwrap(), 30);
                p.u8(u8::from(!(d.players.in_game[i])));
                n += 1;
            } else if d.show_scores {
                p.i32(0).u8(2).u8(i as u8);
                p.i32(d.players.scores[i]);
                p.i16(0).u8(0);
                p.u8(u8::from(d.players.public_hand[i]));
                p.ascii_fixed("", 30);
                p.u8(1);
                n += 1;
            }
        }
        let mut expected = p.compile();
        if n != seats {
            expected[10] = n as u8; // Stream.Seek(10); Write((byte)n)
        }

        send_players_info(&mut w, Some(ns), game, m);
        assert_eq!(sent(&mut w), expected, "show_scores {show_scores}");
    }
}

fn game_ref(w: &World, game: ItemId) -> GameData {
    g(w, game)
}

#[test]
fn test_mahjong_general_info() {
    for (show_scores, spectator_vision) in
        [(true, true), (true, false), (false, true), (false, false)]
    {
        let (mut w, ns) = net_world();
        let (game, _) = game_with(&mut w, show_scores, spectator_vision, false);
        let d = game_ref(&w, game);

        // MahjongGeneralInfo
        let mut p = Packet::new();
        p.serial(serial_of(&w, game)).u8(0).u8(5).i16(0).u8(0);
        p.u8(u8::from(d.show_scores) | if d.spectator_vision { 2 } else { 0 });
        p.u8(d.dices.0 as u8).u8(d.dices.1 as u8);
        p.u8(d.dealer_indicator.wind as u8)
            .i16(d.dealer_indicator.position.y as i16)
            .i16(d.dealer_indicator.position.x as i16);
        p.u8(d.dealer_indicator.direction as u8);
        p.i16(d.wall_break.y as i16).i16(d.wall_break.x as i16);

        send_general_info(&mut w, Some(ns), game);
        assert_eq!(sent(&mut w), p.compile());
    }
}

/// The value byte of a tile for `to` (`MahjongTilesInfo` and `MahjongTileInfo` share it).
fn tile_value(d: &GameData, t: &Tile, to: MobileId) -> u8 {
    if t.flipped {
        let hand = t.dimensions().hand_area();
        let player_at = if hand >= 0 {
            d.players.players.get(hand as usize).copied().flatten()
        } else {
            None
        };
        if hand < 0
            || d.players.public_hand[hand as usize]
            || player_at == Some(to)
            || d.spectator_vision && d.players.spectators.contains(&to)
        {
            return t.value as u8;
        }
    }
    0
}

fn write_tile(p: &mut Packet, d: &GameData, t: &Tile, to: MobileId) {
    p.u8(t.number as u8).u8(tile_value(d, t, to));
    p.i16(t.position.y as i16).i16(t.position.x as i16);
    p.u8(t.stack_level as u8).u8(t.direction as u8);
    p.u8(if t.flipped { 0x10 } else { 0 });
}

#[test]
fn test_mahjong_tiles_info() {
    for spectator_vision in [true, false] {
        let (mut w, ns) = net_world();
        let (game, m) = game_with(&mut w, false, spectator_vision, true);
        let d = game_ref(&w, game);

        let mut p = Packet::new();
        p.serial(serial_of(&w, game))
            .u8(0)
            .u8(4)
            .i16(d.tiles.len() as i16);
        for t in &d.tiles {
            write_tile(&mut p, &d, t, m);
        }

        send_tiles_info(&mut w, Some(ns), game, m);
        assert_eq!(sent(&mut w), p.compile());
    }
}

#[test]
fn test_mahjong_tile_info() {
    for spectator_vision in [true, false] {
        let (mut w, ns) = net_world();
        let (game, m) = game_with(&mut w, false, spectator_vision, true);
        let d = game_ref(&w, game);

        let mut p = Packet::new();
        p.serial(serial_of(&w, game)).u8(0).u8(3);
        write_tile(&mut p, &d, &d.tiles[0], m);

        send_tile_info(&mut w, Some(ns), game, 0, m);
        assert_eq!(sent(&mut w), p.compile());
    }
}
