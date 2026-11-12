//! `IharaSoko`: the Imperial Minister of Trade, who takes the minor artifacts.

use muo_content::creature::CreatureType;
use muo_content::player_mobile as pmob;
use muo_content::vendor::{
    BASE_VENDOR, BASE_VENDOR_C, BASE_VENDOR_CLASS, VendorClass, add_hued, new_vendor,
};
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;
use muo_server::type_registry::TypeRegistry;

use super::ITEMS_PER_REWARD;
use super::gumps::{
    close_redeem, close_turn_in, find_redeemable_items, has_redeem, has_turn_in, send_redeem,
    turn_in_gump,
};

/// `IharaSoko()`: `base("the Imperial Minister of Trade")`, `Female = false; Body = 0x190; Hue = 0x8403`.
pub fn ihara_soko_new(w: &mut World) -> MobileId {
    let id = new_vendor(w, &IHARA_SOKO, Some("the Imperial Minister of Trade"));
    muo_server::mobile::part1::props_b::set_female(w, id, false);
    muo_server::mobile::part1::props_b::set_body(w, id, muo_primitives::Body(0x190));
    muo_server::mobile::base::set_hue(w, id, 0x8403);
    id
}

fn init_outfit(w: &mut World, id: MobileId) {
    add_hued(w, id, "Server.Items.Waraji", 0x711);
    if let Some(pack) = muo_content::factory::make_item(w, "Server.Items.Backpack") {
        muo_server::mobile::part2::items::add_item(w, id, pack);
    }
    add_hued(w, id, "Server.Items.Kamishimo", 0x483);
    add_hued(w, id, "Server.Items.LightPlateJingasa", 0x711);
}

/// `OnMovement(m, oldLocation)`.
fn on_movement(w: &mut World, id: MobileId, m: MobileId, old_location: Point3D) {
    if !muo_content::mx::alive(w, m) || pmob::data(w, m).is_none() {
        return;
    }
    let (z, pz) = (
        muo_content::mx::location(w, id).z,
        muo_content::mx::location(w, m).z,
    );
    if muo_content::mx::alive(w, m)
        && (z - pz).abs() < 16
        && muo_content::mx::in_range(w, id, m, 3)
        && !muo_server::mobile::base::in_range_point3d(w, id, old_location, 3)
    {
        let turned = pmob::data(w, m).map_or(0, |d| d.tot_items_turned_in);
        if turned >= ITEMS_PER_REWARD {
            // Congratulations! You have turned in enough minor treasures to earn a greater reward.
            muo_server::mobile::part2::speech::say_to_number(w, id, m, 1070980, "");
            close_turn_in(w, m); // Sanity
            if !has_redeem(w, m) {
                send_redeem(w, m, id, false);
            }
        } else {
            if turned == 0 {
                // Bring me 10 of the lost treasures of Tokuno and I will reward you with a valuable item.
                muo_server::mobile::part2::speech::say_to_number(w, id, m, 1071013, "");
            } else {
                // You have turned in ~1_COUNT~ minor artifacts. Turn in ~2_NUM~ to receive a reward.
                muo_server::mobile::part2::speech::say_to_number(
                    w,
                    id,
                    m,
                    1070981,
                    &format!("{turned}\t{ITEMS_PER_REWARD}"),
                );
            }
            let buttons = find_redeemable_items(w, m);
            if let Some(b) = buttons.filter(|b| !b.is_empty())
                && !has_turn_in(w, m)
            {
                let g = turn_in_gump(w, id, b);
                muo_content::gumps::send_gump_to_mobile(w, m, Box::new(g), false);
            }
        }
    }

    const LEAVE_RANGE: i32 = 7;
    if !muo_content::mx::in_range(w, id, m, LEAVE_RANGE)
        && muo_server::mobile::base::in_range_point3d(w, id, old_location, LEAVE_RANGE)
    {
        close_redeem(w, m);
        close_turn_in(w, m);
    }
}

static IHARA_SOKO_V: VendorClass = VendorClass {
    is_active_vendor: |_, _| false,
    init_outfit,
    // `TurnToTokuno() { }`: keeps the name "Ihara Soko".
    turn_to_tokuno: |_, _| {},
    ..BASE_VENDOR_CLASS
};

static IHARA_SOKO_C: CreatureType = CreatureType {
    is_invulnerable: |_, _| true,
    disallow_all_moves: |_, _| true,
    can_teach: |_, _| false,
    default_name: |_, _| Some("Ihara Soko"),
    subclass: Some(&IHARA_SOKO_V),
    ..BASE_VENDOR_C
};

/// `Server.Mobiles.IharaSoko`.
pub static IHARA_SOKO: MobileType = MobileType {
    full_name: "Server.Mobiles.IharaSoko",
    parent: Some(&BASE_VENDOR),
    construct: None,
    make: Some(ihara_soko_new),
    family: Some(&IHARA_SOKO_C),
    click_title: |_, _| true,
    can_be_damaged: |_, _| false,
    on_movement,
    serialize: |w, id, out| {
        (BASE_VENDOR.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_VENDOR.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_VENDOR
};

pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&IHARA_SOKO, &[]);
}
