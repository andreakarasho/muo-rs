//! The custom hairstylist: the vendor (a `BaseVendor`, `[SerializationGenerator(0, false)]`)
//! and its three `DynamicGump`s. A gump reads what it needs from its mobiles (the balance, the sex) when it is
//! made, which is when it is sent.

use muo_content::creature::CreatureType;
use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::system::close_gump;
use muo_content::player_mobile::{self, RACE_ELF};
use muo_content::vendor::banker;
use muo_content::vendor::{BASE_VENDOR, BASE_VENDOR_C, BASE_VENDOR_CLASS, VendorClass};
use muo_primitives::{MessageType, utility};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::MobileType;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::messages;
use muo_server::net_state::NetStateId;
use muo_server::type_registry::TypeRegistry;

use muo_gumps::m_z::common::{mobile_of, send_singleton};

struct BuyInfo {
    title: i32,
    price: i32,
    facial_hair: bool,
    /// `GumpFactoryFn`: which gump the entry opens.
    kind: Kind,
}

enum Kind {
    Style {
        facial: bool,
    },
    /// `(hair, facialHair, bright)`.
    Hue(bool, bool, bool),
}

const SELL_LIST: [BuyInfo; 8] = [
    BuyInfo {
        title: 1018357,
        price: 50000,
        facial_hair: false,
        kind: Kind::Style { facial: false },
    }, // New Hair
    BuyInfo {
        title: 1018358,
        price: 50000,
        facial_hair: true,
        kind: Kind::Style { facial: true },
    }, // New Beard
    BuyInfo {
        title: 1018359,
        price: 50,
        facial_hair: false,
        kind: Kind::Hue(true, true, false),
    }, // Normal Hair Dye
    BuyInfo {
        title: 1018360,
        price: 500000,
        facial_hair: false,
        kind: Kind::Hue(true, true, true),
    }, // Bright Hair Dye
    BuyInfo {
        title: 1018361,
        price: 30000,
        facial_hair: false,
        kind: Kind::Hue(true, false, false),
    }, // Hair Only Dye
    BuyInfo {
        title: 1018362,
        price: 30000,
        facial_hair: true,
        kind: Kind::Hue(false, true, false),
    }, // Beard Only Dye
    BuyInfo {
        title: 1018363,
        price: 500000,
        facial_hair: false,
        kind: Kind::Hue(true, false, true),
    }, // Bright Hair Only Dye
    BuyInfo {
        title: 1018364,
        price: 500000,
        facial_hair: true,
        kind: Kind::Hue(false, true, true),
    }, // Bright Beard Only Dye
];

fn is_female(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|m| m.female || m.cur_body().is_female())
}

fn private(w: &mut World, vendor: MobileId, to: MobileId, number: i32) {
    let ns = w.mobile(to).and_then(|m| m.net_state);
    messages::private_overhead_message_number(
        w,
        vendor,
        MessageType::REGULAR,
        0x3B2,
        number,
        "",
        ns,
    );
}

// ---- the vendor ----

fn custom_hairstylist_new(w: &mut World) -> MobileId {
    super::npc::new_template_vendor(w, &CUSTOM_HAIRSTYLIST)
}

static CUSTOM_HAIRSTYLIST_V: VendorClass = VendorClass {
    init_sb_info: |_, _| Vec::new(),
    is_active_buyer: |_, _| false,
    is_active_seller: |_, _| true,
    shoe_type: crate::template_keys_npcs::shoe_type,
    get_hair_hue: |w, _| utility::random_bright_hue(&mut w.rng),
    on_buy_items: |_, _, _, _| false,
    vendor_buy: |w, id, from| {
        let g = buy_gump(w, from, id);
        send_singleton(w, from, g);
    },
    init_outfit: crate::template_keys_npcs::init_outfit,
    ..BASE_VENDOR_CLASS
};

pub static CUSTOM_HAIRSTYLIST_C: CreatureType = CreatureType {
    subclass: Some(&CUSTOM_HAIRSTYLIST_V),
    ..BASE_VENDOR_C
};

pub static CUSTOM_HAIRSTYLIST: MobileType = MobileType {
    full_name: "Server.Mobiles.CustomHairstylist",
    parent: Some(&BASE_VENDOR),
    construct: Some(custom_hairstylist_new),
    family: Some(&CUSTOM_HAIRSTYLIST_C),
    click_title: |_, _| false,
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

// ---- HairstylistBuyGump ----

pub struct HairstylistBuyGump {
    from: MobileId,
    vendor: MobileId,
    female: bool,
    balance: i32,
}

/// `new HairstylistBuyGump(from, vendor, _sellList)`: closes the other two hairstylist gumps.
fn buy_gump(w: &mut World, from: MobileId, vendor: MobileId) -> DynamicGumpObj<HairstylistBuyGump> {
    if let Some(ns) = w.mobile(from).and_then(|m| m.net_state) {
        close_gump::<ChangeHairHueGump>(w, ns);
        close_gump::<ChangeHairstyleGump>(w, ns);
    }
    DynamicGumpObj::new(
        50,
        50,
        HairstylistBuyGump {
            from,
            vendor,
            female: is_female(w, from),
            balance: banker::get_balance(w, from),
        },
    )
}

impl DynamicGumpLogic for HairstylistBuyGump {
    const TYPE_NAME: &'static str = "Server.Mobiles.HairstylistBuyGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let can =
            |info: &BuyInfo| self.balance >= info.price && (!info.facial_hair || !self.female);
        let can_afford = SELL_LIST.iter().filter(|i| can(i)).count() as i32;

        b.add_page(0);
        b.add_background(50, 10, 450, 100 + can_afford * 25, 2600);
        b.add_html_localized(100, 40, 350, 20, 1018356, false, false); // Choose your hairstyle change:

        let mut index = 0;
        for (i, info) in SELL_LIST.iter().enumerate() {
            if can(info) {
                b.add_html_localized(140, 75 + index * 25, 300, 20, info.title, false, false);
                b.add_button(100, 75 + index * 25, 4005, 4007, 1 + i as i32);
                index += 1;
            }
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let index = info.button_id - 1;
        if index < 0 || index as usize >= SELL_LIST.len() {
            return;
        }
        let (from, vendor) = (self.from, self.vendor);
        let balance = banker::get_balance(w, from);
        let female = is_female(w, from);
        let buy = &SELL_LIST[index as usize];
        if buy.facial_hair && female {
            private(w, vendor, from, 1010639);
        } else if balance >= buy.price {
            match buy.kind {
                Kind::Style { facial } => {
                    let g = style_gump(w, from, vendor, buy.price, facial);
                    send_singleton(w, from, g);
                }
                Kind::Hue(hair, facial_hair, bright) => {
                    let entries = if bright {
                        &BRIGHT_ENTRIES[..]
                    } else {
                        &REGULAR_ENTRIES[..]
                    };
                    let g = DynamicGumpObj::new(
                        50,
                        50,
                        ChangeHairHueGump {
                            vendor,
                            price: buy.price,
                            hair,
                            facial_hair,
                            entries,
                        },
                    );
                    send_singleton(w, from, g);
                }
            }
        } else {
            private(w, vendor, from, 1042293);
        }
    }
}

// ---- ChangeHairHueGump ----

/// `ChangeHairHueEntry`: a name and `count` consecutive hues from `start`.
struct HueEntry {
    start: i32,
    count: i32,
}

static BRIGHT_ENTRIES: [HueEntry; 8] = [
    HueEntry {
        start: 12,
        count: 10,
    },
    HueEntry {
        start: 32,
        count: 5,
    },
    HueEntry {
        start: 38,
        count: 8,
    },
    HueEntry {
        start: 54,
        count: 3,
    },
    HueEntry {
        start: 62,
        count: 10,
    },
    HueEntry {
        start: 81,
        count: 2,
    },
    HueEntry {
        start: 89,
        count: 2,
    },
    HueEntry {
        start: 1153,
        count: 2,
    },
];

static REGULAR_ENTRIES: [HueEntry; 12] = [
    HueEntry {
        start: 1602,
        count: 26,
    },
    HueEntry {
        start: 1628,
        count: 27,
    },
    HueEntry {
        start: 1502,
        count: 32,
    },
    HueEntry {
        start: 1302,
        count: 32,
    },
    HueEntry {
        start: 1402,
        count: 32,
    },
    HueEntry {
        start: 1202,
        count: 24,
    },
    HueEntry {
        start: 2402,
        count: 29,
    },
    HueEntry {
        start: 2213,
        count: 6,
    },
    HueEntry {
        start: 1102,
        count: 8,
    },
    HueEntry {
        start: 1110,
        count: 8,
    },
    HueEntry {
        start: 1118,
        count: 16,
    },
    HueEntry {
        start: 1134,
        count: 16,
    },
];

const HUE_NAME: &str = "*****";

pub struct ChangeHairHueGump {
    vendor: MobileId,
    price: i32,
    hair: bool,
    facial_hair: bool,
    entries: &'static [HueEntry],
}

impl DynamicGumpLogic for ChangeHairHueGump {
    const TYPE_NAME: &'static str = "Server.Mobiles.ChangeHairHueGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);

        b.add_background(100, 10, 350, 370, 2600);
        b.add_background(120, 54, 110, 270, 5100);

        b.add_html_localized(155, 25, 240, 30, 1011013, false, false); // <center>Hair Color Selection Menu</center>

        b.add_html_localized(150, 330, 220, 35, 1011014, false, false); // Dye my hair this color!
        b.add_button(380, 330, 4005, 4007, 1);

        for (i, entry) in self.entries.iter().enumerate() {
            let i = i as i32;
            b.add_label(130, 59 + i * 22, entry.start - 1, HUE_NAME);
            b.add_button_type(207, 60 + i * 22, 5224, 5224, 0, GumpButtonType::Page, 1 + i);
        }

        for (i, entry) in self.entries.iter().enumerate() {
            b.add_page(1 + i as i32);
            for j in 0..entry.count {
                let (page, index) = (j / 16, j % 16);
                b.add_label(
                    278 + page * 80,
                    52 + index * 17,
                    entry.start + j - 1,
                    HUE_NAME,
                );
                b.add_radio(
                    260 + page * 80,
                    52 + index * 17,
                    210,
                    211,
                    false,
                    j * self.entries.len() as i32 + i as i32,
                );
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let vendor = self.vendor;
        if info.button_id != 1 || info.switches.is_empty() {
            private(w, vendor, from, 1013009); // You decide not to change your hairstyle.
            return;
        }
        let n = self.entries.len() as i32;
        let (offset, index) = (info.switches[0] / n, info.switches[0] % n);
        if index < 0 || index >= n || offset < 0 || offset >= self.entries[index as usize].count {
            return;
        }
        let (hair_id, facial_id) = w
            .mobile(from)
            .map_or((0, 0), |m| (m.hair_item_id, m.facial_hair_item_id));
        if (!self.hair || hair_id <= 0) && (!self.facial_hair || facial_id <= 0) {
            private(w, vendor, from, 502623); // You have no hair to dye and you cannot use this.
            return;
        }
        if !banker::withdraw(w, from, self.price) {
            private(w, vendor, from, 1042293); // You cannot afford my services for that style.
            return;
        }
        let hue = self.entries[index as usize].start + offset;
        if self.hair {
            props_b::set_hair_hue(w, from, hue);
        }
        if self.facial_hair {
            props_b::set_facial_hair_hue(w, from, hue);
        }
    }
}

// ---- ChangeHairstyleGump ----

/// `ChangeHairstyleEntry(gumpID, x, y, itemID)`.
struct StyleEntry {
    gump_id: i32,
    x: i32,
    y: i32,
    item_id: i32,
}

const fn se(gump_id: i32, x: i32, y: i32, item_id: i32) -> StyleEntry {
    StyleEntry {
        gump_id,
        x,
        y,
        item_id,
    }
}

static HAIR_ENTRIES: [StyleEntry; 10] = [
    se(50700, 70 - 137, 20 - 60, 0x203B),
    se(60710, 193 - 260, 18 - 60, 0x2045),
    se(50703, 316 - 383, 25 - 60, 0x2044),
    se(60708, 70 - 137, 75 - 125, 0x203C),
    se(60900, 193 - 260, 85 - 125, 0x2047),
    se(60713, 320 - 383, 85 - 125, 0x204A),
    se(60702, 70 - 137, 140 - 190, 0x203D),
    se(60707, 193 - 260, 140 - 190, 0x2049),
    se(60901, 315 - 383, 150 - 190, 0x2048),
    se(0, 0, 0, 0),
];

static BEARD_ENTRIES: [StyleEntry; 7] = [
    se(50800, 120 - 187, 30 - 80, 0x2040),
    se(50904, 243 - 310, 33 - 80, 0x204B),
    se(50906, 120 - 187, 100 - 150, 0x204D),
    se(50801, 243 - 310, 95 - 150, 0x203E),
    se(50802, 120 - 187, 173 - 220, 0x203F),
    se(50905, 243 - 310, 165 - 220, 0x204C),
    se(50808, 120 - 187, 242 - 290, 0x2041),
];

pub struct ChangeHairstyleGump {
    from: MobileId,
    vendor: MobileId,
    price: i32,
    facial_hair: bool,
    entries: &'static [StyleEntry],
}

/// `new ChangeHairstyleGump(from, vendor, price, facialHair, entries)`: closes the other two hairstylist gumps.
fn style_gump(
    w: &mut World,
    from: MobileId,
    vendor: MobileId,
    price: i32,
    facial: bool,
) -> DynamicGumpObj<ChangeHairstyleGump> {
    if let Some(ns) = w.mobile(from).and_then(|m| m.net_state) {
        close_gump::<HairstylistBuyGump>(w, ns);
        close_gump::<ChangeHairHueGump>(w, ns);
    }
    let entries: &'static [StyleEntry] = if facial {
        &BEARD_ENTRIES
    } else {
        &HAIR_ENTRIES
    };
    DynamicGumpObj::new(
        50,
        50,
        ChangeHairstyleGump {
            from,
            vendor,
            price,
            facial_hair: facial,
            entries,
        },
    )
}

impl DynamicGumpLogic for ChangeHairstyleGump {
    const TYPE_NAME: &'static str = "Server.Mobiles.ChangeHairstyleGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let facial = self.facial_hair;
        let table_width: i32 = if facial { 2 } else { 3 };
        let table_height =
            (self.entries.len() as i32 + table_width - if facial { 1 } else { 2 }) / table_width;
        const OFFSET_WIDTH: i32 = 123;
        let offset_height = if facial { 70 } else { 65 };
        let table_width_offset = 81 + table_width * OFFSET_WIDTH;
        let table_height_offset = 45 + table_height * offset_height;

        b.add_page(0);

        b.add_background(0, 0, table_width_offset, 60 + table_height_offset, 2600);

        b.add_button(45, table_height_offset, 4005, 4007, 1);
        b.add_html_localized(77, table_height_offset, 90, 35, 1006044, false, false); // Ok

        b.add_button(table_width_offset - 180, table_height_offset, 4005, 4007, 0);
        b.add_html_localized(
            table_width_offset - 148,
            table_height_offset,
            90,
            35,
            1006045,
            false,
            false,
        ); // Cancel

        if !facial {
            b.add_html_localized(50, 15, 350, 20, 1018353, false, false); // <center>New Hairstyle</center>
        } else {
            b.add_html_localized(55, 15, 200, 20, 1018354, false, false); // <center>New Beard</center>
        }

        for (i, entry) in self.entries.iter().enumerate() {
            let i = i as i32;
            let (y_table, x_table) = (i / table_width, i % table_width);
            let x_offset = x_table * OFFSET_WIDTH;
            let y_offset = y_table * offset_height;

            if entry.gump_id != 0 {
                b.add_radio(40 + x_offset, 70 + y_offset, 208, 209, false, i);
                b.add_background(87 + x_offset, 50 + y_offset, 50, 50, 2620);
                b.add_image(
                    87 + x_offset + entry.x,
                    50 + y_offset + entry.y,
                    entry.gump_id,
                    0,
                );
            } else if !facial {
                b.add_radio(40 + (x_table + 1) * OFFSET_WIDTH, 240, 208, 209, false, i);
                b.add_html_localized(
                    60 + (x_table + 1) * OFFSET_WIDTH,
                    240,
                    85,
                    35,
                    1011064,
                    false,
                    false,
                ); // Bald
            } else {
                b.add_radio(40 + x_offset, 70 + y_offset, 208, 209, false, i);
                b.add_html_localized(60 + x_offset, 70 + y_offset, 85, 35, 1011064, false, false); // Bald
            }
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let (from, vendor) = (self.from, self.vendor);
        if self.facial_hair && is_female(w, from) {
            return;
        }
        if w.mobile(from).is_some_and(|m| m.race == Some(RACE_ELF)) {
            muo_server::mobile::part2::messages::send_message(
                w,
                from,
                "This isn't implemented for elves yet.  Sorry!",
            );
            return;
        }
        if info.button_id != 1 || info.switches.is_empty() {
            private(w, vendor, from, 1013009); // You decide not to change your hairstyle.
            return;
        }
        let index = info.switches[0];
        if index < 0 || index as usize >= self.entries.len() {
            return;
        }
        let entry = &self.entries[index as usize];

        if player_mobile::data(w, from).is_some() {
            player_mobile::set_hair_mods(w, from, -1, -1);
        }

        let current = w.mobile(from).map_or(0, |m| {
            if self.facial_hair {
                m.facial_hair_item_id
            } else {
                m.hair_item_id
            }
        });
        if current == entry.item_id {
            return;
        }

        if !banker::withdraw(w, from, self.price) {
            private(w, vendor, from, 1042293); // You cannot afford my services for that style.
        } else if self.facial_hair {
            props_b::set_facial_hair_item_id(w, from, entry.item_id);
        } else {
            props_b::set_hair_item_id(w, from, entry.item_id);
        }
    }
}

pub(crate) fn register(types: &mut TypeRegistry) {
    types.register_mobile(&CUSTOM_HAIRSTYLIST, &[]);
}

#[cfg(test)]
pub(crate) fn all() -> Vec<&'static MobileType> {
    vec![&CUSTOM_HAIRSTYLIST]
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_content::gumps::base::Gump;

    #[test]
    fn the_three_gumps_compile() {
        let mut w = World::new();
        let from = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        let vendor = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        for (balance, female) in [
            (0, false),
            (50, false),
            (10_000_000, false),
            (10_000_000, true),
        ] {
            let mut g = DynamicGumpObj::new(
                50,
                50,
                HairstylistBuyGump {
                    from,
                    vendor,
                    female,
                    balance,
                },
            );
            assert_eq!(g.compile()[0], 0xDD);
        }
        for bright in [false, true] {
            let entries: &'static [HueEntry] = if bright {
                &BRIGHT_ENTRIES
            } else {
                &REGULAR_ENTRIES
            };
            let mut g = DynamicGumpObj::new(
                50,
                50,
                ChangeHairHueGump {
                    vendor,
                    price: 1,
                    hair: true,
                    facial_hair: true,
                    entries,
                },
            );
            assert_eq!(g.compile()[0], 0xDD);
        }
        for facial in [false, true] {
            let mut g = style_gump(&mut w, from, vendor, 50000, facial);
            assert_eq!(g.compile()[0], 0xDD);
        }
    }

    #[test]
    fn it_has_no_sb_info_and_sells_through_its_own_gump() {
        let mut w = World::new();
        let id = (CUSTOM_HAIRSTYLIST.construct.unwrap())(&mut w);
        assert!(
            muo_content::vendor::data(&w, id)
                .unwrap()
                .buy_info
                .is_empty()
        );
        assert!(muo_content::vendor::is_active_seller(&w, id));
        assert!(!muo_content::vendor::is_active_buyer(&w, id));
    }
}
