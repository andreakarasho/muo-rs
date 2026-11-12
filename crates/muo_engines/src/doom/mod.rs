//! Doom: the gauntlet (`GauntletSpawner`, `GauntletRegion`, `[GenGauntlet`/`[RemGauntlet`)
//! and the lamp room lever puzzle.
//!
//! The priced healer, the variety dealer and the Wanderer of the Void are made by name (classes of muo_mobiles);
//! `PricedHealer(5000)`'s price is set through [`DoomHooks`].

use muo_primitives::{Point3D, Rectangle2D};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::ops;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

pub mod gauntlet;
pub mod lever;

const MALAS: MapId = MapId::MALAS;

fn p(x: i32, y: i32) -> Point3D {
    Point3D::new(x, y, -1)
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.delete)(w, id);
    }
}

/// `RemoveItem<T>(x, y, z = -1)`: the first match at the tile (`item is T && z == -1 || item.Z == z`, as written).
fn remove_item(w: &mut World, full_name: &str, x: i32, y: i32, z: i32) {
    let found = muo_server::map::enumerate::items_at(w, Some(MALAS), x, y).find(|&i| {
        (muo_content::factory::item_is(w, i, full_name) && z == -1)
            || w.item(i).is_some_and(|it| it.location.z == z)
    });
    if let Some(i) = found {
        delete_item(w, i);
    }
}

fn remove_mobile(w: &mut World, full_name: &str, x: i32, y: i32) {
    let found = w.mobiles.iter().find_map(|(&id, m)| {
        let home_ok = muo_content::creature::cd(w, id).is_some_and(|d| d.home == p(x, y));
        (m.map == Some(MALAS) && muo_content::factory::mobile_is(w, id, full_name) && home_ok)
            .then_some(id)
    });
    if let Some(m) = found
        && let Some(t) = w.mobile(m).map(|x| x.ty)
    {
        (t.delete)(w, m);
    }
}

fn remove_teleporter(w: &mut World, x_from: i32, y_from: i32) {
    remove_item(w, "Server.Items.Teleporter", x_from, y_from, -1);
    remove_item(w, "Server.Items.Static", x_from, y_from, -1);
}

fn remove_door_set(w: &mut World, x: i32, y: i32) {
    let found = muo_server::map::enumerate::items_at(w, Some(MALAS), x, y).find(|&i| {
        w.item(i).is_some_and(|it| it.location.z == -1)
            && muo_items::construction::doors::is_door(w, i)
    });
    if let Some(door) = found {
        if let Some(l) = gauntlet::door_link(w, door) {
            delete_item(w, l);
        }
        delete_item(w, door);
    }
}

/// `[RemGauntlet`: "Removes the Gauntlet from Dungeon Doom."
fn remove_gauntlet(w: &mut World, _e: &mut muo_server::commands::CommandEventArgs) {
    remove_all(w);
}

fn remove_all(w: &mut World) {
    remove_mobile(w, "Server.Mobiles.PricedHealer", 387, 400);
    remove_teleporter(w, 390, 407);
    remove_door_set(w, 393, 404);
    remove_item(w, "Server.Items.MorphItem", 433, 371, -1);
    remove_item(w, "Server.Items.MorphItem", 433, 372, -1);
    remove_mobile(w, "Server.Mobiles.VarietyDealer", 492, 369);
    for x in 434..=478 {
        for y in 371..=372 {
            remove_item(w, "Server.Items.Static", x, y, -1);
        }
    }
    for (x, y) in [(471, 428), (462, 494), (403, 502), (357, 476), (361, 433)] {
        remove_teleporter(w, x, y);
    }
    for (x, y) in [
        (491, 456),
        (482, 520),
        (406, 538),
        (335, 512),
        (326, 433),
        (423, 430),
    ] {
        remove_item(w, "Server.Engines.Doom.GauntletSpawner", x, y, -1);
    }
    remove_item(w, "Server.Items.ConfirmationMoongate", 433, 326, 4);
}

/// `CreateDoorSet(xDoor, yDoor, doorEastToWest, hue)`.
fn create_door_set(w: &mut World, x: i32, y: i32, east_to_west: bool, hue: i32) -> ItemId {
    use muo_content::items::house_deps::DoorFacing;
    use muo_items::construction::doors::{METAL_DOOR, facing_door, set_link};
    let (hf, lf) = if east_to_west {
        (DoorFacing::NorthCCW, DoorFacing::SouthCW)
    } else {
        (DoorFacing::WestCW, DoorFacing::EastCCW)
    };
    let hi = facing_door(w, &METAL_DOOR, hf, 0x675, 0xEC, 0xF3);
    let lo = facing_door(w, &METAL_DOOR, lf, 0x675, 0xEC, 0xF3);
    ops::move_to_world_map(w, hi, p(x, y), Some(MALAS));
    let (dx, dy) = if east_to_west { (0, 1) } else { (1, 0) };
    ops::move_to_world_map(w, lo, p(x + dx, y + dy), Some(MALAS));
    set_link(w, hi, Some(lo));
    set_link(w, lo, Some(hi));
    crate::util::hue(w, hi, hue);
    crate::util::hue(w, lo, hue);
    hi
}

/// `CreateTeleporter(xFrom, yFrom, xTo, yTo)`.
fn create_teleporter(w: &mut World, xf: i32, yf: i32, xt: i32, yt: i32) {
    use muo_items::misc::basic::{STATIC, static_new};
    use muo_items::misc::teleporter;
    let pad = static_new(w, &STATIC, 0x1822);
    let tele =
        teleporter::teleporter_new(w, &teleporter::TELEPORTER, p(xt, yt), Some(MALAS), false);
    crate::util::hue(w, pad, 0x482);
    ops::move_to_world_map(w, pad, p(xf, yf), Some(MALAS));
    ops::move_to_world_map(w, tele, p(xf, yf), Some(MALAS));
    if let Some(d) = teleporter::data_mut(w, tele) {
        d.sound_id = 0x1FE;
    }
    teleporter::set_flag(w, tele, teleporter::flags::SOURCE_EFFECT, true);
    teleporter::set_flag(w, tele, teleporter::flags::DEST_EFFECT, true);
}

/// `CreateSpawner(...)`.
#[allow(clippy::too_many_arguments)]
fn create_spawner(
    w: &mut World,
    type_name: &str,
    xs: i32,
    ys: i32,
    xd: i32,
    yd: i32,
    xp: i32,
    yp: i32,
    east_to_west: bool,
    x_start: i32,
    y_start: i32,
    x_width: i32,
    y_height: i32,
) -> ItemId {
    let spawner = gauntlet::new_spawner(w, Some(type_name));
    ops::move_to_world_map(w, spawner, p(xs, ys), Some(MALAS));
    if xd > 0 && yd > 0 {
        let door = create_door_set(w, xd, yd, east_to_west, 0);
        if let Some(d) = gauntlet::data_mut(w, spawner) {
            d.door = Some(door);
        }
    }
    if let Some(d) = gauntlet::data_mut(w, spawner) {
        d.region_bounds = Rectangle2D::new(x_start, y_start, x_width, y_height);
    }
    if xp > 0
        && yp > 0
        && let Some(pent) = muo_content::factory::make_item(w, "Server.Items.PentagramAddon")
    {
        ops::move_to_world_map(w, pent, p(xp, yp), Some(MALAS));
        if let Some(d) = gauntlet::data_mut(w, spawner) {
            d.addon = Some(pent);
        }
    }
    spawner
}

/// `Home = Location; RangeHome = range`.
fn home_here(w: &mut World, m: MobileId, range: i32) {
    let loc = muo_content::mx::location(w, m);
    if let Some(d) = muo_content::creature::cd_mut(w, m) {
        d.home = loc;
        d.range_home = range;
    }
}

/// `[GenGauntlet`: "Generates the Gauntlet in Dungeon Doom."
fn gen_gauntlet(w: &mut World, e: &mut muo_server::commands::CommandEventArgs) {
    use muo_content::factory::make_mobile;
    remove_gauntlet(w, e);

    // Begin healer room
    if let Some(healer) = make_mobile(w, "Server.Mobiles.PricedHealer") {
        if let Some(f) = w
            .resources
            .get::<DoomHooks>()
            .and_then(|h| h.set_healer_price)
        {
            f(w, healer, 5000);
        }
        muo_server::mobile::base::move_to_world(w, healer, p(387, 400), MALAS);
        home_here(w, healer, 5);
    }
    create_teleporter(w, 390, 407, 394, 405);
    let healer_door = create_door_set(w, 393, 404, true, 0x44E);
    {
        use muo_items::construction::doors::{set_key_value, set_locked};
        set_locked(w, healer_door, true);
        let k = muo_content::items::gold::random_value(w);
        set_key_value(w, healer_door, k);
        if let Some(l) =
            gauntlet::door_link(w, healer_door).filter(|l| w.item(*l).is_some_and(|i| !i.deleted()))
        {
            set_locked(w, l, true);
            let k = muo_content::items::gold::random_value(w);
            set_key_value(w, l, k);
        }
    }
    // End healer room

    // Begin supply room
    for (y, active) in [(371, 0x116), (372, 0x115)] {
        let item = muo_items::misc::basic::morph_item_new(
            w,
            &muo_items::misc::basic::MORPH_ITEM,
            0x29F,
            active,
            3,
            3,
        );
        crate::util::hue(w, item, 0x44E);
        ops::move_to_world_map(w, item, p(433, y), Some(MALAS));
    }
    if let Some(dealer) = make_mobile(w, "Server.Mobiles.VarietyDealer") {
        outfit_variety_dealer(w, dealer);
        muo_server::mobile::base::move_to_world(w, dealer, p(492, 369), MALAS);
        home_here(w, dealer, 2);
    }
    for x in 434..=478 {
        for y in 371..=372 {
            let item =
                muo_items::misc::basic::static_new(w, &muo_items::misc::basic::STATIC, 0x524);
            crate::util::hue(w, item, 1);
            ops::move_to_world_map(w, item, p(x, y), Some(MALAS));
        }
    }
    // End supply room

    // Begin gauntlet cycle
    for (a, b, c, d) in [
        (471, 428, 474, 428),
        (462, 494, 462, 498),
        (403, 502, 399, 506),
        (357, 476, 356, 480),
        (361, 433, 357, 434),
    ] {
        create_teleporter(w, a, b, c, d);
    }
    let sp1 = create_spawner(
        w,
        "DarknightCreeper",
        491,
        456,
        473,
        432,
        417,
        426,
        true,
        473,
        412,
        39,
        60,
    );
    let sp2 = create_spawner(
        w,
        "FleshRenderer",
        482,
        520,
        468,
        496,
        426,
        422,
        false,
        448,
        496,
        56,
        48,
    );
    let sp3 = create_spawner(
        w, "Impaler", 406, 538, 408, 504, 432, 430, false, 376, 504, 64, 48,
    );
    let sp4 = create_spawner(
        w,
        "ShadowKnight",
        335,
        512,
        360,
        478,
        424,
        439,
        false,
        300,
        478,
        72,
        64,
    );
    let sp5 = create_spawner(
        w,
        "AbysmalHorror",
        326,
        433,
        360,
        429,
        416,
        435,
        true,
        300,
        408,
        60,
        56,
    );
    let sp6 = create_spawner(
        w,
        "DemonKnight",
        423,
        430,
        0,
        0,
        423,
        430,
        true,
        392,
        392,
        72,
        96,
    );
    let chain = [sp1, sp2, sp3, sp4, sp5, sp6, sp1];
    for pair in chain.windows(2) {
        if let Some(d) = gauntlet::data_mut(w, pair[0]) {
            d.sequence = Some(pair[1]);
        }
    }
    gauntlet::set_state(w, sp1, gauntlet::State::InProgress);
    // End gauntlet cycle

    // Begin exit gate
    {
        use muo_items::misc::moongates::{
            CONFIRMATION_MOONGATE, confirmation_edit, confirmation_moongate_new,
        };
        let gate = confirmation_moongate_new(
            w,
            &CONFIRMATION_MOONGATE,
            Point3D::new(2350, 1270, -85),
            Some(MALAS),
        );
        muo_items::misc::moongates::set_dispellable(w, gate, false);
        confirmation_edit(w, gate, |d| {
            d.gump_width = 420;
            d.gump_height = 280;
            d.message_color = 0x7F00;
            d.message = Some(muo_io::text::TextDefinition::of_number(1062109)); // You are about to exit Dungeon Doom.  Do you wish to continue?
            d.title_color = 0x7800;
            d.title_number = 1062108; // Please verify...
        });
        crate::util::hue(w, gate, 0x44E);
        ops::move_to_world_map(w, gate, Point3D::new(433, 326, 4), Some(MALAS));
    }
}

/// The outfit block of `CreateVarietyDealer`: everything but the shop layers is deleted, then four items.
fn outfit_variety_dealer(w: &mut World, dealer: MobileId) {
    use muo_server::layer::Layer;
    muo_server::mobile::base::set_name(w, dealer, Some("Nix".to_owned()));
    muo_server::mobile::part1::props_a::set_title(w, dealer, Some("the Variety Dealer".to_owned()));
    muo_server::mobile::part1::props_b::set_body(w, dealer, muo_primitives::Body(400));
    muo_server::mobile::part1::props_b::set_female(w, dealer, false);
    muo_server::mobile::base::set_hue(w, dealer, 0x8835);
    {
        use muo_server::mobile::part1::props_b as pb;
        pb::set_hair_item_id(w, dealer, 0x2049); // Pig Tails
        pb::set_hair_hue(w, dealer, 0x482);
        pb::set_facial_hair_item_id(w, dealer, 0x203E);
        pb::set_facial_hair_hue(w, dealer, 0x482);
    }
    let items: Vec<ItemId> = w
        .mobile(dealer)
        .map(|m| m.items.clone())
        .unwrap_or_default();
    for i in items {
        let layer = w.item(i).map(|x| x.layer);
        if !matches!(
            layer,
            Some(Layer::ShopBuy | Layer::ShopResale | Layer::ShopSell)
        ) {
            delete_item(w, i);
        }
    }
    for (name, hue) in [
        ("Server.Items.FloppyHat", Some(1)),
        ("Server.Items.Robe", Some(1)),
        ("Server.Items.LanternOfSouls", None),
        ("Server.Items.Sandals", Some(0x482)),
    ] {
        if let Some(it) = muo_content::factory::make_item(w, name) {
            if let Some(h) = hue {
                crate::util::hue(w, it, h);
            }
            muo_server::mobile::part2::items::add_item(w, dealer, it);
        }
    }
}

/// Calls into creature classes this crate does not have.
#[derive(Default)]
pub struct DoomHooks {
    /// `new PricedHealer(price)`: sets the price of a made `PricedHealer`. Unset: the price stays its default.
    pub set_healer_price: Option<fn(&mut World, MobileId, i32)>,
}

pub fn configure(w: &mut World) {
    use muo_server::commands::CommandSystem;
    w.resources.get_mut::<DoomHooks>().set_healer_price =
        Some(muo_mobiles::npcs::healers::set_price);
    CommandSystem::register(
        w,
        "GenGauntlet",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(gen_gauntlet),
    );
    CommandSystem::register(
        w,
        "RemGauntlet",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(remove_gauntlet),
    );
    CommandSystem::register(
        w,
        "GenLeverPuzzle",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(lever::gen_lamp_puzzle),
    );
    muo_content::network::events::subscribe_ranked::<muo_content::network::events::PlayerLogin>(
        w,
        5,
        |w, e| {
            lever::on_login(w, e.0);
        },
    );
}

pub fn register(types: &mut TypeRegistry) {
    gauntlet::register(types);
    lever::register(types);
    types.register_phase(
        Phase::Configure,
        "Doom.Configure",
        DEFAULT_CALL_PRIORITY,
        configure,
    );
}
