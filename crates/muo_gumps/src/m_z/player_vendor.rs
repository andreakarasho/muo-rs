//! The player vendor gumps. The vendor itself is behind
//! [`PlayerVendorOps`](super::ops::PlayerVendorOps); dressing it uses the server's mobile and item functions.

use std::rc::Rc;

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{DynamicGumpBuilder, GumpStrings, StaticGumpBuilder};
use muo_server::World;
use muo_server::hue_picker::{HuePicker, send_to};
use muo_server::ids::{ItemId, MobileId};
use muo_server::layer::Layer;
use muo_server::net_state::NetStateId;

use super::common::{mobile_of, send_singleton};
use super::hooks::ops;
use super::ops::{PlayerVendorOps, PlayerVendorView, VendorItemView};

fn vendor_ops(w: &World) -> Option<Rc<dyn PlayerVendorOps>> {
    ops(w).player_vendors
}

// ---- PlayerVendorBuyGump ----

pub struct PlayerVendorBuyGump {
    vendor: MobileId,
    vi: VendorItemView,
    /// `Localization.GetText(1019072) ?? "an item without a description"`, read when the gump is built.
    no_description: String,
}

/// `PlayerVendorBuyGump.DisplayTo(from, vendor, vi)`; the `description` is `vi.Description`.
pub fn buy_display_to(w: &mut World, from: MobileId, vendor: MobileId, vi: VendorItemView) {
    // 1019072: an item without a description
    let no_description = muo_content::items::equip_hooks::hooks(w)
        .and_then(|h| h.localized_text)
        .and_then(|f| f(w, 1019072))
        .unwrap_or_else(|| "an item without a description".to_owned());
    let g = StaticGumpObj::new(
        100,
        200,
        PlayerVendorBuyGump {
            vendor,
            vi,
            no_description,
        },
    );
    send_singleton(w, from, g);
}

impl StaticGumpLogic for PlayerVendorBuyGump {
    const TYPE_NAME: &'static str = "Server.Gumps.PlayerVendorBuyGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);

        b.add_background(100, 10, 300, 150, 5054);

        b.add_html_localized(125, 20, 250, 24, 1019070, false, false); // You have agreed to purchase:

        b.add_html_placeholder(125, 45, 250, 24, "description", false, false);

        b.add_html_localized(125, 70, 250, 24, 1019071, false, false); // for the amount of:
        b.add_label_placeholder(125, 95, 0, "price");

        b.add_button(250, 130, 4005, 4007, 0);
        b.add_html_localized(282, 130, 100, 24, 1011012, false, false); // CANCEL

        b.add_button(120, 130, 4005, 4007, 1);
        b.add_html_localized(152, 130, 100, 24, 1011036, false, false); // OKAY
    }

    fn build_strings(&self, strings: &mut GumpStrings) {
        let description = if self.vi.description.is_empty() {
            self.no_description.clone()
        } else {
            self.vi.description.clone()
        };

        strings.set_html_text("description", &description, "", -1);

        strings.set_string_slot("price", &self.vi.price.to_string());
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(o) = vendor_ops(w) else { return };
        let vendor = self.vendor;

        if !o.can_interact_with(w, vendor, from, false) {
            return;
        }

        if o.is_owner(w, vendor, from) {
            o.say_to(w, vendor, from, 503212); // You own this shop, just take what you want.
            return;
        }

        if info.button_id == 1 {
            let name = super::common::name_of(w, from);
            o.say(w, vendor, &name);

            if !self.vi.valid || !o.vendor_item_valid_in_pack(w, vendor, self.vi.item) {
                o.say_to(w, vendor, from, 503216); // You can't buy that.
                return;
            }

            let mut total_gold = 0;

            let pack = muo_server::mobile::part1::props_b::backpack(w, from);
            if let Some(p) = pack {
                total_gold += muo_server::item::container::get_amount(
                    w,
                    p,
                    &muo_content::items::gold::GOLD,
                    true,
                );
            }

            total_gold += muo_content::vendor::banker::get_balance(w, from);

            let price = self.vi.price;
            if total_gold < price {
                o.say_to(w, vendor, from, 503205); // You cannot afford this item.
            } else if !muo_server::mobile::part2::items::place_in_backpack(w, from, self.vi.item) {
                o.say_to(w, vendor, from, 503204); // You do not have room in your backpack for this.
            } else {
                let mut left_price = price;

                if let Some(p) = pack {
                    left_price -= muo_server::item::container::consume_up_to(
                        w,
                        p,
                        &muo_content::items::gold::GOLD,
                        left_price,
                        true,
                    );
                }

                if left_price > 0 {
                    muo_content::vendor::banker::withdraw(w, from, left_price);
                }

                o.add_hold_gold(w, vendor, price);

                muo_content::mx::msg(w, from, 503201); // You take the item.
            }
        } else {
            muo_content::mx::msg(w, from, 503207); // Cancelled purchase.
        }
    }
}

// ---- PlayerVendorOwnerGump ----

pub struct PlayerVendorOwnerGump {
    vendor: MobileId,
    view: PlayerVendorView,
}

/// `PlayerVendorOwnerGump.DisplayTo(from, vendor)`.
pub fn owner_display_to(w: &mut World, from: MobileId, vendor: MobileId) {
    let Some(o) = vendor_ops(w) else { return };
    if let Some(view) = o.view(w, vendor) {
        let g = StaticGumpObj::new(50, 200, PlayerVendorOwnerGump { vendor, view });
        super::common::send(w, from, g);
    }
}

impl StaticGumpLogic for PlayerVendorOwnerGump {
    const TYPE_NAME: &'static str = "Server.Gumps.PlayerVendorOwnerGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(25, 10, 530, 140, 5054);

        b.add_html_localized(425, 25, 120, 20, 1019068, false, false); // See goods
        b.add_button(390, 25, 4005, 4007, 1);
        b.add_html_localized(425, 48, 120, 20, 1019069, false, false); // Customize
        b.add_button(390, 48, 4005, 4007, 2);
        b.add_html_localized(425, 72, 120, 20, 1011012, false, false); // CANCEL
        b.add_button(390, 71, 4005, 4007, 0);

        b.add_html_localized(40, 72, 260, 20, 1038321, false, false); // Gold held for you:
        b.add_label_placeholder(300, 72, 0, "holdGold");
        b.add_html_localized(40, 96, 260, 20, 1038322, false, false); // Gold held in my account:
        b.add_label_placeholder(300, 96, 0, "bankAccount");

        // Client 1038324 replaced:
        b.add_html(40, 120, 260, 20, "My charge per day is:", false, false);
        b.add_label_placeholder(300, 120, 0, "perDay");

        b.add_html_localized(40, 25, 260, 20, 1038318, false, false); // Amount of days I can work:
        b.add_label_placeholder(300, 25, 0, "days");
        b.add_html_localized(40, 48, 260, 20, 1038319, false, false); // Earth days:
        b.add_label_placeholder(300, 48, 0, "earthDays");
    }

    fn build_strings(&self, strings: &mut GumpStrings) {
        let v = &self.view;
        let per_day = v.charge_per_day;
        let days = f64::from(v.hold_gold + v.bank_account) / f64::from(per_day);

        strings.set_string_slot("holdGold", &v.hold_gold.to_string());
        strings.set_string_slot("bankAccount", &v.bank_account.to_string());
        strings.set_string_slot("perDay", &per_day.to_string());
        strings.set_string_slot("days", &(days as i32).to_string());
        strings.set_string_slot("earthDays", &((days / 12.0) as i32).to_string());
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(o) = vendor_ops(w) else { return };
        let vendor = self.vendor;

        if !o.can_interact_with(w, vendor, from, true) {
            return;
        }

        match info.button_id {
            1 => o.open_backpack(w, vendor, from),
            2 => customize_display_to(w, from, vendor),
            _ => {}
        }
    }
}

// ---- NewPlayerVendorOwnerGump ----

pub struct NewPlayerVendorOwnerGump {
    vendor: MobileId,
    view: PlayerVendorView,
}

/// `NewPlayerVendorOwnerGump.DisplayTo(from, vendor)`.
pub fn new_owner_display_to(w: &mut World, from: MobileId, vendor: MobileId) {
    let Some(o) = vendor_ops(w) else { return };
    if let Some(view) = o.view(w, vendor) {
        // `goldHeld / perRealWorldDay` threw a DivideByZeroException in the layout: the gump was never sent.
        if view.charge_per_real_world_day == 0 {
            return;
        }
        let g = DynamicGumpObj::new(50, 200, NewPlayerVendorOwnerGump { vendor, view });
        send_singleton(w, from, g);
    }
}

impl DynamicGumpLogic for NewPlayerVendorOwnerGump {
    const TYPE_NAME: &'static str = "Server.Gumps.NewPlayerVendorOwnerGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let v = &self.view;
        let per_real_world_day = v.charge_per_real_world_day;
        let gold_held = v.hold_gold;

        b.add_page(0);

        b.add_background(25, 10, 530, 180, 0x13BE);

        b.add_image_tiled(35, 20, 510, 160, 0xA40);
        b.add_alpha_region(35, 20, 510, 160);

        b.add_image(10, 0, 0x28DC, 0);
        b.add_image(537, 175, 0x28DC, 0);
        b.add_image(10, 175, 0x28DC, 0);
        b.add_image(537, 0, 0x28DC, 0);

        if gold_held < per_real_world_day {
            let gold_needed = per_real_world_day - gold_held;

            b.add_html_localized_color(40, 35, 260, 20, 1038320, 0x7FFF, false, false); // Gold needed for 1 day of vendor salary:
            b.add_label(300, 35, 0x1F, &gold_needed.to_string());
        } else {
            let days = gold_held / per_real_world_day;

            b.add_html_localized_color(40, 35, 260, 20, 1038318, 0x7FFF, false, false); // # of days Vendor salary is paid for:
            b.add_label(300, 35, 0x480, &days.to_string());
        }

        b.add_html_localized_color(40, 58, 260, 20, 1038324, 0x7FFF, false, false); // My charge per real world day is:
        b.add_label(300, 58, 0x480, &per_real_world_day.to_string());

        b.add_html_localized_color(40, 82, 260, 20, 1038322, 0x7FFF, false, false); // Gold held in my account:
        b.add_label(300, 82, 0x480, &gold_held.to_string());

        b.add_html_localized_color(40, 108, 260, 20, 1062509, 0x7FFF, false, false); // Shop Name:
        b.add_label(140, 106, 0x66D, &v.shop_name);

        if let Some((days, hours)) = v.rental_expires {
            b.add_label(
                38,
                132,
                0x480,
                &format!(
                    "Location rental will expire in {days} day{} and {hours} hour{}.",
                    if days != 1 { "s" } else { "" },
                    if hours != 1 { "s" } else { "" }
                ),
            );
        }

        b.add_button(390, 24, 0x15E1, 0x15E5, 1);
        b.add_html_localized_color(408, 21, 120, 20, 1019068, 0x7FFF, false, false); // See goods

        b.add_button(390, 44, 0x15E1, 0x15E5, 2);
        b.add_html_localized_color(408, 41, 120, 20, 1019069, 0x7FFF, false, false); // Customize

        b.add_button(390, 64, 0x15E1, 0x15E5, 3);
        b.add_html_localized_color(408, 61, 120, 20, 1062434, 0x7FFF, false, false); // Rename Shop

        b.add_button(390, 84, 0x15E1, 0x15E5, 4);
        b.add_html_localized_color(408, 81, 120, 20, 3006217, 0x7FFF, false, false); // Rename Vendor

        b.add_button(390, 104, 0x15E1, 0x15E5, 5);
        b.add_html_localized_color(408, 101, 120, 20, 3006123, 0x7FFF, false, false); // Open Paperdoll

        b.add_button(390, 124, 0x15E1, 0x15E5, 6);
        b.add_label(408, 121, 0x480, "Collect Gold");

        b.add_button(390, 144, 0x15E1, 0x15E5, 7);
        b.add_label(408, 141, 0x480, "Dismiss Vendor");

        b.add_button(390, 162, 0x15E1, 0x15E5, 0);
        b.add_html_localized_color(408, 161, 120, 20, 1011012, 0x7FFF, false, false); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(o) = vendor_ops(w) else { return };
        let vendor = self.vendor;

        if matches!(info.button_id, 1 | 2) {
            // See goods or Customize
            o.check_teleport(w, vendor, from);
        }

        if !o.can_interact_with(w, vendor, from, true) {
            return;
        }

        match info.button_id {
            1 => o.open_backpack(w, vendor, from),
            2 => new_customize_display_to(w, from, vendor),
            3 => o.rename_shop(w, vendor, from),
            4 => o.rename(w, vendor, from),
            5 => o.display_paperdoll_to(w, vendor, from),
            6 => o.collect_gold(w, vendor, from),
            7 => o.dismiss(w, vendor, from),
            _ => {}
        }
    }
}

// ---- PlayerVendorCustomizeGump ----

struct CustomItem {
    /// The class (`None` for hair and beards).
    type_name: Option<&'static str>,
    item_id: i32,
    loc_number: i32,
    art_number: i32,
    long_text: bool,
}

const fn hair(item_id: i32, loc: i32, long_text: bool) -> CustomItem {
    CustomItem {
        type_name: None,
        item_id,
        loc_number: loc,
        art_number: 0,
        long_text,
    }
}

const fn item(type_name: &'static str, loc: i32, art: i32) -> CustomItem {
    CustomItem {
        type_name: Some(type_name),
        item_id: 0,
        loc_number: loc,
        art_number: art,
        long_text: false,
    }
}

struct CustomCategory {
    layer: Layer,
    loc_number: i32,
    can_dye: bool,
    entries: &'static [CustomItem],
}

static CATEGORIES: [CustomCategory; 11] = [
    CustomCategory {
        layer: Layer::InnerTorso,
        loc_number: 1011357,
        can_dye: true,
        entries: &[
            // Upper Torso
            item("Server.Items.Shirt", 1011359, 5399),
            item("Server.Items.FancyShirt", 1011360, 7933),
            item("Server.Items.PlainDress", 1011363, 7937),
            item("Server.Items.FancyDress", 1011364, 7935),
            item("Server.Items.Robe", 1011365, 7939),
        ],
    },
    CustomCategory {
        layer: Layer::MiddleTorso,
        loc_number: 1011371,
        can_dye: true,
        entries: &[
            // Over chest
            item("Server.Items.Doublet", 1011358, 8059),
            item("Server.Items.Tunic", 1011361, 8097),
            item("Server.Items.JesterSuit", 1011366, 8095),
            item("Server.Items.BodySash", 1011372, 5441),
            item("Server.Items.Surcoat", 1011362, 8189),
            item("Server.Items.HalfApron", 1011373, 5435),
            item("Server.Items.FullApron", 1011374, 5437),
        ],
    },
    CustomCategory {
        layer: Layer::Shoes,
        loc_number: 1011388,
        can_dye: true,
        entries: &[
            // Footwear
            item("Server.Items.Sandals", 1011389, 5901),
            item("Server.Items.Shoes", 1011390, 5904),
            item("Server.Items.Boots", 1011391, 5899),
            item("Server.Items.ThighBoots", 1011392, 5906),
        ],
    },
    CustomCategory {
        layer: Layer::Helm,
        loc_number: 1011375,
        can_dye: true,
        entries: &[
            // Hats
            item("Server.Items.SkullCap", 1011376, 5444),
            item("Server.Items.Bandana", 1011377, 5440),
            item("Server.Items.FloppyHat", 1011378, 5907),
            item("Server.Items.WideBrimHat", 1011379, 5908),
            item("Server.Items.Cap", 1011380, 5909),
            item("Server.Items.TallStrawHat", 1011382, 5910),
        ],
    },
    CustomCategory {
        layer: Layer::Helm,
        loc_number: 1015319,
        can_dye: true,
        entries: &[
            // More Hats
            item("Server.Items.StrawHat", 1011382, 5911),
            item("Server.Items.WizardsHat", 1011383, 5912),
            item("Server.Items.Bonnet", 1011384, 5913),
            item("Server.Items.FeatheredHat", 1011385, 5914),
            item("Server.Items.TricorneHat", 1011386, 5915),
            item("Server.Items.JesterHat", 1011387, 5916),
        ],
    },
    CustomCategory {
        layer: Layer::Pants,
        loc_number: 1011367,
        can_dye: true,
        entries: &[
            // Lower Torso
            item("Server.Items.LongPants", 1011368, 5433),
            item("Server.Items.Kilt", 1011369, 5431),
            item("Server.Items.Skirt", 1011370, 5398),
        ],
    },
    CustomCategory {
        layer: Layer::Cloak,
        loc_number: 1011393,
        can_dye: true,
        entries: &[
            // Back
            item("Server.Items.Cloak", 1011394, 5397),
        ],
    },
    CustomCategory {
        layer: Layer::Hair,
        loc_number: 1011395,
        can_dye: true,
        entries: &[
            // Hair
            hair(0x203B, 1011052, false),
            hair(0x203C, 1011053, false),
            hair(0x203D, 1011054, false),
            hair(0x2044, 1011055, false),
            hair(0x2045, 1011047, false),
            hair(0x204A, 1011050, false),
            hair(0x2047, 1011396, false),
            hair(0x2048, 1011048, false),
            hair(0x2049, 1011049, false),
        ],
    },
    CustomCategory {
        layer: Layer::FacialHair,
        loc_number: 1015320,
        can_dye: true,
        entries: &[
            // Facial Hair
            hair(0x2041, 1011062, false),
            hair(0x203F, 1011060, false),
            hair(0x204B, 1015321, true),
            hair(0x203E, 1011061, false),
            hair(0x204C, 1015322, true),
            hair(0x2040, 1015323, false),
            hair(0x204D, 1011401, false),
        ],
    },
    CustomCategory {
        layer: Layer::FIRST_VALID,
        loc_number: 1011397,
        can_dye: false,
        entries: &[
            // Held items
            item("Server.Items.FishingPole", 1011406, 3520),
            item("Server.Items.Pickaxe", 1011407, 3717),
            item("Server.Items.Pitchfork", 1011408, 3720),
            item("Server.Items.Cleaver", 1015324, 3778),
            item("Server.Items.Mace", 1011409, 3933),
            item("Server.Items.Torch", 1011410, 3940),
            item("Server.Items.Hammer", 1011411, 4020),
            item("Server.Items.Longsword", 1011412, 3936),
            item("Server.Items.GnarledStaff", 1011413, 5113),
        ],
    },
    CustomCategory {
        layer: Layer::FIRST_VALID,
        loc_number: 1015325,
        can_dye: false,
        entries: &[
            // More held items
            item("Server.Items.Crossbow", 1011414, 3920),
            item("Server.Items.WarMace", 1011415, 5126),
            item("Server.Items.TwoHandedAxe", 1011416, 5186),
            item("Server.Items.Spear", 1011417, 3939),
            item("Server.Items.Katana", 1011418, 5118),
            item("Server.Items.Spellbook", 1011419, 3834),
        ],
    },
];

/// The categories.
fn categories() -> &'static [CustomCategory] {
    &CATEGORIES
}

pub struct PlayerVendorCustomizeGump {
    vendor: MobileId,
}

/// `PlayerVendorCustomizeGump.DisplayTo(from, vendor)`.
pub fn customize_display_to(w: &mut World, from: MobileId, vendor: MobileId) {
    if !muo_content::mx::deleted(w, vendor) {
        let g = StaticGumpObj::new(30, 40, PlayerVendorCustomizeGump { vendor });
        send_singleton(w, from, g);
    }
}

impl StaticGumpLogic for PlayerVendorCustomizeGump {
    const TYPE_NAME: &'static str = "Server.Gumps.PlayerVendorCustomizeGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 585, 393, 5054);
        b.add_background(195, 36, 387, 275, 3000);
        b.add_html_localized(10, 10, 565, 18, 1011356, false, false); // <center>VENDOR CUSTOMIZATION MENU</center>
        b.add_html_localized(60, 355, 150, 18, 1011036, false, false); // OKAY
        b.add_button(25, 355, 4005, 4007, 1);
        b.add_html_localized(320, 355, 150, 18, 1011012, false, false); // CANCEL
        b.add_button(285, 355, 4005, 4007, 0);

        let mut y = 35;
        for (i, cat) in categories().iter().enumerate() {
            b.add_html_localized(5, y, 150, 25, cat.loc_number, true, false);
            b.add_button_type(155, y, 4005, 4007, 0, GumpButtonType::Page, i as i32 + 1);
            y += 25;
        }

        for (i, cat) in categories().iter().enumerate() {
            let i = i as i32;
            b.add_page(i + 1);

            for (c, entry) in cat.entries.iter().enumerate() {
                let c = c as i32;
                let x = 198 + c % 3 * 129;
                let y = 38 + c / 3 * 67;

                b.add_html_localized(
                    x,
                    y,
                    100,
                    if entry.long_text { 36 } else { 18 },
                    entry.loc_number,
                    false,
                    false,
                );

                if entry.art_number != 0 {
                    b.add_item(x + 20, y + 25, entry.art_number, 0);
                }

                b.add_radio(
                    x,
                    y + if entry.long_text { 40 } else { 20 },
                    210,
                    211,
                    false,
                    (c << 8) + i,
                );
            }

            if cat.can_dye {
                b.add_html_localized(327, 239, 100, 18, 1011402, false, false); // Color
                b.add_radio(327, 259, 210, 211, false, 100 + i);
            }

            b.add_html_localized(456, 239, 100, 18, 1011403, false, false); // Remove
            b.add_radio(456, 259, 210, 211, false, 200 + i);
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let vendor = self.vendor;
        if muo_content::mx::deleted(w, vendor) {
            return;
        }

        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(o) = vendor_ops(w) else { return };

        let is_player_vendor = o.is_player_vendor(w, vendor);
        if is_player_vendor && !o.can_interact_with(w, vendor, from, true) {
            return;
        }

        if let Some(false) = o.barkeeper_is_owner(w, vendor, from) {
            return;
        }

        if info.button_id == 0 {
            if is_player_vendor {
                // do nothing for barkeeps
                bow_to(w, vendor, from);
            }
        } else if info.button_id == 1 && !info.switches.is_empty() {
            let cnum = info.switches[0];
            let mut cat = cnum % 256;
            let ent = cnum >> 8;
            let cats = categories();

            if cat >= 0 && (cat as usize) < cats.len() {
                if ent >= 0 && (ent as usize) < cats[cat as usize].entries.len() {
                    let category = &cats[cat as usize];
                    let entry = &category.entries[ent as usize];

                    let mut found = muo_server::mobile::part2::items::find_item_on_layer(
                        w,
                        vendor,
                        category.layer,
                    );
                    if let Some(i) = found {
                        delete_item(w, i);
                    }

                    if found.is_none() {
                        found = find_item_of_category(w, vendor, category);
                    }
                    if let Some(i) = found {
                        delete_item(w, i);
                    }

                    if category.layer == Layer::FacialHair {
                        if w.mobile(vendor).is_some_and(|m| m.female) {
                            muo_content::mx::msg(w, from, 1010639); // You cannot place facial hair on a woman!
                        } else {
                            let hue = w.mobile(vendor).map_or(0, |m| m.facial_hair_hue);

                            muo_server::mobile::part1::props_b::set_facial_hair_item_id(
                                w, vendor, 0,
                            );
                            muo_server::mobile::base::process_delta(w, vendor); // invalidate item ID for clients

                            muo_server::mobile::part1::props_b::set_facial_hair_item_id(
                                w,
                                vendor,
                                entry.item_id,
                            );
                            muo_server::mobile::part1::props_b::set_facial_hair_hue(w, vendor, hue);
                        }
                    } else if category.layer == Layer::Hair {
                        let hue = w.mobile(vendor).map_or(0, |m| m.hair_hue);

                        muo_server::mobile::part1::props_b::set_hair_item_id(w, vendor, 0);
                        muo_server::mobile::base::process_delta(w, vendor); // invalidate item ID for clients

                        muo_server::mobile::part1::props_b::set_hair_item_id(
                            w,
                            vendor,
                            entry.item_id,
                        );
                        muo_server::mobile::part1::props_b::set_hair_hue(w, vendor, hue);
                    } else if let Some(i) = entry
                        .type_name
                        .and_then(|n| muo_content::factory::make_item(w, n))
                    {
                        muo_server::item::base::set_layer(w, i, category.layer);

                        if !muo_server::mobile::base::equip_item(w, vendor, i) {
                            delete_item(w, i);
                        }
                    }

                    customize_display_to(w, from, vendor);
                }
            } else {
                cat -= 100;

                if cat < 100 {
                    if cat >= 0 && (cat as usize) < cats.len() {
                        let category = &cats[cat as usize];

                        if category.layer == Layer::Hair {
                            send_hair_hue_picker(w, sender, vendor, from, false);
                        } else if category.layer == Layer::FacialHair {
                            send_hair_hue_picker(w, sender, vendor, from, true);
                        } else if let Some(i) = find_item_of_category(w, vendor, category) {
                            send_item_hue_picker(w, sender, i, vendor, from);
                        }
                    }
                } else {
                    cat -= 100;

                    if (cat as usize) < cats.len() {
                        let category = &cats[cat as usize];

                        if category.layer == Layer::Hair {
                            muo_server::mobile::part1::props_b::set_hair_item_id(w, vendor, 0);
                        } else if category.layer == Layer::FacialHair {
                            muo_server::mobile::part1::props_b::set_facial_hair_item_id(
                                w, vendor, 0,
                            );
                        } else if let Some(i) = find_item_of_category(w, vendor, category) {
                            delete_item(w, i);
                        }

                        customize_display_to(w, from, vendor);
                    }
                }
            }
        }
    }
}

/// `_vendor.Direction = _vendor.GetDirectionTo(from); Animate(32, 5, 1, true, false, 0); SayTo(from, 1043310 + Random(12))`.
fn bow_to(w: &mut World, vendor: MobileId, from: MobileId) {
    let o = vendor_ops(w);
    let d = muo_content::mx::get_direction_to(w, vendor, from);
    muo_content::mx::set_direction(w, vendor, d);
    muo_content::mx::animate(w, vendor, 32, 5, 1, true, false, 0); // bow
    let number = 1043310 + muo_content::hooks::rnd::below(w, 12); // a little random speech
    if let Some(o) = o {
        o.say_to(w, vendor, from, number);
    }
}

fn delete_item(w: &mut World, i: ItemId) {
    if let Some(ty) = w.item(i).map(|i| i.ty) {
        (ty.delete)(w, i);
    }
}

/// The first of the vendor's items (in order) whose class is one of the category's.
fn find_item_of_category(w: &World, vendor: MobileId, category: &CustomCategory) -> Option<ItemId> {
    let items = w.mobile(vendor)?.items.clone();
    for i in items {
        let name = w.item(i).map(|it| it.ty.full_name)?;
        if category.entries.iter().any(|e| e.type_name == Some(name)) {
            return Some(i);
        }
    }
    None
}

/// `new PVHairHuePicker(facialHair, vendor, from).SendTo(state)`.
fn send_hair_hue_picker(
    w: &mut World,
    ns: NetStateId,
    vendor: MobileId,
    from: MobileId,
    facial: bool,
) {
    let mut picker = HuePicker::new(w, 0xFAB);
    picker.on_response = Some(Rc::new(move |w: &mut World, hue: i32| {
        if muo_content::mx::deleted(w, vendor) {
            return;
        }
        let Some(o) = vendor_ops(w) else { return };
        if o.is_player_vendor(w, vendor) && !o.can_interact_with(w, vendor, from, true) {
            return;
        }
        if let Some(false) = o.barkeeper_is_owner(w, vendor, from) {
            return;
        }
        if facial {
            muo_server::mobile::part1::props_b::set_facial_hair_hue(w, vendor, hue);
        } else {
            muo_server::mobile::part1::props_b::set_hair_hue(w, vendor, hue);
        }
        customize_display_to(w, from, vendor);
    }));
    send_to(w, ns, picker);
}

/// `new PVHuePicker(item, vendor, from).SendTo(state)`.
fn send_item_hue_picker(
    w: &mut World,
    ns: NetStateId,
    item: ItemId,
    vendor: MobileId,
    from: MobileId,
) {
    let item_id = w.item(item).map_or(0, |i| i.item_id);
    let mut picker = HuePicker::new(w, item_id);
    picker.on_response = Some(Rc::new(move |w: &mut World, hue: i32| {
        if muo_server::item::ops::is_deleted(w, item) {
            return;
        }
        let Some(o) = vendor_ops(w) else { return };
        if o.is_player_vendor(w, vendor) && !o.can_interact_with(w, vendor, from, true) {
            return;
        }
        if let Some(false) = o.barkeeper_is_owner(w, vendor, from) {
            return;
        }
        muo_server::item::base::set_hue(w, item, hue); // Item.Hue.set: the delta
        customize_display_to(w, from, vendor);
    }));
    send_to(w, ns, picker);
}

// ---- NewPlayerVendorCustomizeGump ----

/// `(itemID, name)` of the hair styles and beards.
const HAIR_STYLES: [(i32, i32); 9] = [
    (0x203B, 1011052), // Short
    (0x203C, 1011053), // Long
    (0x203D, 1011054), // Ponytail
    (0x2044, 1011055), // Mohawk
    (0x2045, 1011047), // Pageboy
    (0x204A, 1011050), // Topknot
    (0x2047, 1011396), // Curly
    (0x2048, 1011048), // Receding
    (0x2049, 1011049), // 2-tails
];

const BEARD_STYLES: [(i32, i32); 7] = [
    (0x2041, 1011062), // Mustache
    (0x203F, 1011060), // Short beard
    (0x204B, 1015321), // Short Beard & Moustache
    (0x203E, 1011061), // Long beard
    (0x204C, 1015322), // Long Beard & Moustache
    (0x2040, 1015323), // Goatee
    (0x204D, 1011401), // Vandyke
];

pub struct NewPlayerVendorCustomizeGump {
    vendor: MobileId,
    female: bool,
}

/// `NewPlayerVendorCustomizeGump.DisplayTo(from, vendor)`.
pub fn new_customize_display_to(w: &mut World, from: MobileId, vendor: MobileId) {
    if !muo_content::mx::deleted(w, vendor) {
        let female = w.mobile(vendor).is_some_and(|m| m.female);
        let g = DynamicGumpObj::new(50, 50, NewPlayerVendorCustomizeGump { vendor, female });
        send_singleton(w, from, g);
    }
}

impl DynamicGumpLogic for NewPlayerVendorCustomizeGump {
    const TYPE_NAME: &'static str = "Server.Gumps.NewPlayerVendorCustomizeGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);

        b.add_background(0, 0, 370, 370, 0x13BE);

        b.add_image_tiled(10, 10, 350, 20, 0xA40);
        b.add_image_tiled(10, 40, 350, 20, 0xA40);
        b.add_image_tiled(10, 70, 350, 260, 0xA40);
        b.add_image_tiled(10, 340, 350, 20, 0xA40);

        b.add_alpha_region(10, 10, 350, 350);

        b.add_html_localized_color(10, 12, 350, 18, 1011356, 0x7FFF, false, false); // <center>VENDOR CUSTOMIZATION MENU</center>

        b.add_html_localized_color(10, 42, 150, 18, 1062459, 0x421F, false, false); // <CENTER>HAIR</CENTER>

        for (i, (_, name)) in HAIR_STYLES.iter().enumerate() {
            let i = i as i32;
            b.add_button(10, 70 + i * 20, 0xFA5, 0xFA7, 0x100 | i);
            b.add_html_localized_color(45, 72 + i * 20, 110, 18, *name, 0x7FFF, false, false);
        }

        let hl = HAIR_STYLES.len() as i32;
        b.add_button(10, 70 + hl * 20, 0xFB1, 0xFB3, 2);
        b.add_html_localized_color(45, 72 + hl * 20, 110, 18, 1011403, 0x7FFF, false, false); // Remove

        b.add_button(10, 70 + (hl + 1) * 20, 0xFA5, 0xFA7, 3);
        b.add_html_localized_color(
            45,
            72 + (hl + 1) * 20,
            110,
            18,
            1011402,
            0x7FFF,
            false,
            false,
        ); // Color

        if self.female {
            b.add_button(160, 290, 0xFA5, 0xFA7, 1);
            b.add_html_localized_color(195, 292, 160, 18, 1015327, 0x7FFF, false, false); // Male

            b.add_html_localized_color(195, 312, 160, 18, 1015328, 0x421F, false, false); // Female
        } else {
            b.add_html_localized_color(160, 42, 210, 18, 1062460, 0x421F, false, false); // <CENTER>BEARD</CENTER>

            for (i, (_, name)) in BEARD_STYLES.iter().enumerate() {
                let i = i as i32;
                b.add_button(160, 70 + i * 20, 0xFA5, 0xFA7, 0x200 | i);
                b.add_html_localized_color(195, 72 + i * 20, 160, 18, *name, 0x7FFF, false, false);
            }

            let bl = BEARD_STYLES.len() as i32;
            b.add_button(160, 70 + bl * 20, 0xFB1, 0xFB3, 4);
            b.add_html_localized_color(195, 72 + bl * 20, 160, 18, 1011403, 0x7FFF, false, false); // Remove

            b.add_button(160, 70 + (bl + 1) * 20, 0xFA5, 0xFA7, 5);
            b.add_html_localized_color(
                195,
                72 + (bl + 1) * 20,
                160,
                18,
                1011402,
                0x7FFF,
                false,
                false,
            ); // Color

            b.add_html_localized_color(195, 292, 160, 18, 1015327, 0x421F, false, false); // Male

            b.add_button(160, 310, 0xFA5, 0xFA7, 1);
            b.add_html_localized_color(195, 312, 160, 18, 1015328, 0x7FFF, false, false); // Female
        }

        b.add_button(10, 340, 0xFA5, 0xFA7, 0);
        b.add_html_localized_color(45, 342, 305, 18, 1060675, 0x7FFF, false, false); // CLOSE
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(o) = vendor_ops(w) else { return };
        let vendor = self.vendor;

        if !o.can_interact_with(w, vendor, from, true) {
            return;
        }

        use muo_server::mobile::part1::props_b as pb;
        let (female, hair_id, facial_id) = w.mobile(vendor).map_or((false, 0, 0), |m| {
            (m.female, m.hair_item_id, m.facial_hair_item_id)
        });

        match info.button_id {
            0 => bow_to(w, vendor, from), // CLOSE
            1 => {
                // Female/Male
                if female {
                    pb::set_body(w, vendor, muo_primitives::Body::new(400));
                    pb::set_female(w, vendor, false);
                } else {
                    pb::set_body(w, vendor, muo_primitives::Body::new(401));
                    pb::set_female(w, vendor, true);

                    pb::set_facial_hair_item_id(w, vendor, 0);
                }

                new_customize_display_to(w, from, vendor);
            }
            2 => {
                // Remove hair
                pb::set_hair_item_id(w, vendor, 0);

                new_customize_display_to(w, from, vendor);
            }
            3 => {
                // Color hair
                if hair_id > 0 {
                    send_new_hue_picker(w, sender, vendor, false, from);
                } else {
                    new_customize_display_to(w, from, vendor);
                }
            }
            4 => {
                // Remove beard
                pb::set_facial_hair_item_id(w, vendor, 0);

                new_customize_display_to(w, from, vendor);
            }
            5 => {
                // Color beard
                if facial_id > 0 {
                    send_new_hue_picker(w, sender, vendor, true, from);
                } else {
                    new_customize_display_to(w, from, vendor);
                }
            }
            button => {
                if (button & 0x100) != 0 {
                    // Hair style selected
                    let index = (button & 0xFF) as usize;

                    if index >= HAIR_STYLES.len() {
                        return;
                    }

                    let hair_hue = w.mobile(vendor).map_or(0, |m| m.hair_hue);

                    pb::set_hair_item_id(w, vendor, 0);
                    muo_server::mobile::base::process_delta(w, vendor);

                    pb::set_hair_item_id(w, vendor, HAIR_STYLES[index].0);

                    pb::set_hair_hue(w, vendor, hair_hue);

                    new_customize_display_to(w, from, vendor);
                } else if (button & 0x200) != 0 {
                    // Beard style selected
                    if female {
                        return;
                    }

                    let index = (button & 0xFF) as usize;

                    if index >= BEARD_STYLES.len() {
                        return;
                    }

                    let hair_hue = w.mobile(vendor).map_or(0, |m| m.facial_hair_hue);

                    pb::set_facial_hair_item_id(w, vendor, 0);
                    muo_server::mobile::base::process_delta(w, vendor);

                    pb::set_facial_hair_item_id(w, vendor, BEARD_STYLES[index].0);

                    pb::set_facial_hair_hue(w, vendor, hair_hue);

                    new_customize_display_to(w, from, vendor);
                }
            }
        }
    }
}

/// `new PVHuePicker(vendor, facialHair, from).SendTo(from.NetState)`.
fn send_new_hue_picker(
    w: &mut World,
    ns: NetStateId,
    vendor: MobileId,
    facial: bool,
    from: MobileId,
) {
    let mut picker = HuePicker::new(w, 0xFAB);
    picker.on_response = Some(Rc::new(move |w: &mut World, hue: i32| {
        let Some(o) = vendor_ops(w) else { return };
        if !o.can_interact_with(w, vendor, from, true) {
            return;
        }

        if facial {
            muo_server::mobile::part1::props_b::set_facial_hair_hue(w, vendor, hue);
        } else {
            muo_server::mobile::part1::props_b::set_hair_hue(w, vendor, hue);
        }

        new_customize_display_to(w, from, vendor);
    }));
    send_to(w, ns, picker);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::item::{ITEM_BASE, ItemDelta};
    use muo_server::map::MapId;
    use muo_server::mobile::MOBILE_BASE;
    use muo_server::net_state::NetStateData;

    /// A barkeeper owned by `owner`: only `is_player_vendor` and `barkeeper_is_owner` matter to the hue pickers.
    struct Barkeeper(MobileId);

    impl PlayerVendorOps for Barkeeper {
        fn view(&self, _: &World, _: MobileId) -> Option<PlayerVendorView> {
            None
        }
        fn can_interact_with(&self, _: &mut World, _: MobileId, _: MobileId, _: bool) -> bool {
            false
        }
        fn is_owner(&self, _: &World, _: MobileId, _: MobileId) -> bool {
            false
        }
        fn barkeeper_is_owner(&self, _: &World, _: MobileId, from: MobileId) -> Option<bool> {
            Some(from == self.0)
        }
        fn is_player_vendor(&self, _: &World, _: MobileId) -> bool {
            false
        }
        fn say(&self, _: &mut World, _: MobileId, _: &str) {}
        fn say_to(&self, _: &mut World, _: MobileId, _: MobileId, _: i32) {}
        fn add_hold_gold(&self, _: &mut World, _: MobileId, _: i32) {}
        fn vendor_item_valid_in_pack(&self, _: &World, _: MobileId, _: ItemId) -> bool {
            true
        }
        fn open_backpack(&self, _: &mut World, _: MobileId, _: MobileId) {}
        fn check_teleport(&self, _: &mut World, _: MobileId, _: MobileId) {}
        fn rename_shop(&self, _: &mut World, _: MobileId, _: MobileId) {}
        fn rename(&self, _: &mut World, _: MobileId, _: MobileId) {}
        fn display_paperdoll_to(&self, _: &mut World, _: MobileId, _: MobileId) {}
        fn collect_gold(&self, _: &mut World, _: MobileId, _: MobileId) {}
        fn dismiss(&self, _: &mut World, _: MobileId, _: MobileId) {}
    }

    /// `PVHuePicker.OnResponse` sets `m_Item.Hue`, a property: the client is told (`ItemDelta.Update`), and only
    /// the owner's picker works.
    #[test]
    fn the_item_hue_picker_sets_the_hue_through_the_property() {
        let mut w = World::new();
        let (owner, stranger, vendor) = (
            w.new_mobile(&MOBILE_BASE),
            w.new_mobile(&MOBILE_BASE),
            w.new_mobile(&MOBILE_BASE),
        );
        let item = w.new_item(&ITEM_BASE);
        w.item_mut(item).unwrap().map = Some(MapId::FELUCCA);
        super::super::hooks::hooks_mut(&mut w).ops.player_vendors = Some(Rc::new(Barkeeper(owner)));
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        for (who, hue, want) in [(stranger, 33, 0), (owner, 44, 44)] {
            send_item_hue_picker(&mut w, ns, item, vendor, who);
            let picker = muo_server::hue_picker::remove_hue_picker_at(&mut w, ns, 0).unwrap();
            picker.respond(&mut w, hue);
            assert_eq!(w.item(item).unwrap().hue, want);
        }
        assert!(
            w.item(item)
                .unwrap()
                .delta_flags
                .contains(ItemDelta::UPDATE)
        );
    }
}
