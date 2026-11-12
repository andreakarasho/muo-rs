//! Strandedness: a player who logs in standing on water is moved to the nearest dry
//! place of a list, per facet.

use muo_content::network::events::{PlayerLogin, subscribe_ranked};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::{
    MapId, fit,
    surface::{self, Surface},
};

const FELUCCA: [(i32, i32); 93] = [
    (2528, 3568),
    (2376, 3400),
    (2528, 3896),
    (2168, 3904),
    (1136, 3416),
    (1432, 3648),
    (1416, 4000),
    (4512, 3936),
    (4440, 3120),
    (4192, 3672),
    (4720, 3472),
    (3744, 2768),
    (3480, 2432),
    (3560, 2136),
    (3792, 2112),
    (2800, 2296),
    (2736, 2016),
    (4576, 1456),
    (4680, 1152),
    (4304, 1104),
    (4496, 984),
    (4248, 696),
    (4040, 616),
    (3896, 248),
    (4176, 384),
    (3672, 1104),
    (3520, 1152),
    (3720, 1360),
    (2184, 2152),
    (1952, 2088),
    (2056, 1936),
    (1720, 1992),
    (472, 2064),
    (656, 2096),
    (3008, 3592),
    (2784, 3472),
    (5456, 2400),
    (5976, 2424),
    (5328, 3112),
    (5792, 3152),
    (2120, 3616),
    (2136, 3128),
    (1632, 3528),
    (1328, 3160),
    (1072, 3136),
    (1128, 2976),
    (960, 2576),
    (752, 1832),
    (184, 1488),
    (592, 1440),
    (368, 1216),
    (232, 752),
    (696, 744),
    (304, 1000),
    (840, 376),
    (1192, 624),
    (1200, 192),
    (1512, 240),
    (1336, 456),
    (1536, 648),
    (1104, 952),
    (1864, 264),
    (2136, 200),
    (2160, 528),
    (1904, 512),
    (2240, 784),
    (2536, 776),
    (2488, 216),
    (2336, 72),
    (2648, 288),
    (2680, 576),
    (2896, 88),
    (2840, 344),
    (3136, 72),
    (2968, 520),
    (3192, 328),
    (3448, 208),
    (3432, 608),
    (3184, 752),
    (2800, 704),
    (2768, 1016),
    (2448, 1232),
    (2272, 920),
    (2072, 1080),
    (2048, 1264),
    (1808, 1528),
    (1496, 1880),
    (1656, 2168),
    (2096, 2320),
    (1816, 2528),
    (1840, 2640),
    (1928, 2952),
    (2120, 2712),
];

const ILSHENAR: [(i32, i32); 18] = [
    (1252, 1180),
    (1562, 1090),
    (1444, 1016),
    (1324, 968),
    (1418, 806),
    (1722, 874),
    (1456, 684),
    (1036, 866),
    (612, 476),
    (1476, 372),
    (762, 472),
    (812, 1162),
    (1422, 1144),
    (1254, 1066),
    (1598, 870),
    (1358, 866),
    (510, 302),
    (510, 392),
];

const TOKUNO: [(i32, i32); 72] = [
    (837, 1351),
    (941, 1241),
    (959, 1185),
    (923, 1091),
    (904, 983),
    (845, 944),
    (829, 896),
    (794, 852),
    (766, 821),
    (695, 814),
    (576, 835),
    (518, 840),
    (519, 902),
    (502, 950),
    (503, 1045),
    (547, 1131),
    (518, 1204),
    (506, 1243),
    (526, 1271),
    (562, 1295),
    (616, 1335),
    (789, 1347),
    (712, 1359),
    (202, 498),
    (116, 600),
    (107, 699),
    (162, 799),
    (158, 889),
    (169, 989),
    (194, 1101),
    (250, 1163),
    (295, 1176),
    (280, 1194),
    (286, 1102),
    (250, 1000),
    (260, 906),
    (360, 838),
    (389, 763),
    (415, 662),
    (500, 597),
    (570, 572),
    (631, 577),
    (692, 500),
    (723, 445),
    (672, 379),
    (626, 332),
    (494, 291),
    (371, 336),
    (324, 334),
    (270, 362),
    (1240, 1076),
    (1189, 1115),
    (1046, 1039),
    (1025, 885),
    (907, 809),
    (840, 506),
    (799, 396),
    (720, 258),
    (744, 158),
    (904, 37),
    (974, 91),
    (1020, 187),
    (1035, 288),
    (1104, 395),
    (1215, 462),
    (1275, 488),
    (1348, 611),
    (1363, 739),
    (1364, 765),
    (1364, 876),
    (1300, 936),
    (1240, 1003),
];

/// `m_Trammel = m_Felucca`.
const TRAMMEL: &[(i32, i32)] = &FELUCCA;

fn is_stranded(w: &mut World, from: MobileId) -> bool {
    let Some((map, loc)) = w
        .mobile(from)
        .and_then(|m| m.map.map(|map| (map, m.location)))
    else {
        return false;
    };
    match surface::get_top_surface(w, map, loc) {
        Some(Surface::Land(t)) => (168..=171).contains(&t.id) || (310..=311).contains(&t.id),
        Some(Surface::Static(t)) => (0x1796..=0x17B2).contains(&t.id),
        _ => false,
    }
}

/// `OnLogin(from)`.
pub fn on_login(w: &mut World, from: MobileId) {
    if !is_stranded(w, from) {
        return;
    }
    let Some((map, loc)) = w
        .mobile(from)
        .and_then(|m| m.map.map(|map| (map, m.location)))
    else {
        return;
    };
    let list: &[(i32, i32)] = if map == MapId::FELUCCA {
        &FELUCCA
    } else if map == MapId::TRAMMEL {
        TRAMMEL
    } else if map == MapId::ILSHENAR {
        &ILSHENAR
    } else if map == MapId::TOKUNO {
        &TOKUNO
    } else {
        return;
    };

    let (mut p, mut pdist) = ((0, 0), f64::MAX);
    for &(x, y) in list {
        let (dx, dy) = (f64::from(loc.x - x), f64::from(loc.y - y));
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < pdist {
            p = (x, y);
            pdist = dist;
        }
    }

    let (mut x, mut y) = p;
    let mut z = w.maps.get_average_z(map, x, y);
    let mut can_fit = fit::can_spawn_mobile_at(w, map, x, y, z);
    let mut i = 1;
    while !can_fit && i <= 40 {
        for xo in -1..=1 {
            for yo in -1..=1 {
                if can_fit || (xo == 0 && yo == 0) {
                    continue;
                }
                x = p.0 + xo * i;
                y = p.1 + yo * i;
                z = w.maps.get_average_z(map, x, y);
                can_fit = fit::can_spawn_mobile_at(w, map, x, y, z);
            }
        }
        i += 2;
    }
    if can_fit {
        muo_server::mobile::part1::lifecycle::set_location(w, from, Point3D::new(x, y, z));
    }
}

/// `[OnEvent(nameof(PlayerMobile.PlayerLoginEvent))]`.
pub fn initialize(w: &mut World) {
    subscribe_ranked::<PlayerLogin>(w, 17, |w, e| on_login(w, e.0));
}
