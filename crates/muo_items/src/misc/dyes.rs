//! `HairDye`, `SpecialBeardDye`, `SpecialHairDye`, `PromotionalToken`, `ContractOfEmployment` and
//! `MelisandesHairDye`.
//!
//! `ContractOfEmployment` makes the `PlayerVendor` through the constructor table of that mobile and
//! `BaseHouse.IsThereVendor` through [`PlayerVendorHooks`]; `MelisandesFermentedWine` (a `GreaterExplosionPotion`)
//! waits for the potions.

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{DynamicGumpBuilder, StaticGumpBuilder};
use muo_content::gumps::send_gump_to_mobile;
use muo_content::props::Value;
use muo_io::TextDefinition;
use muo_primitives::{MessageType, Point3D};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::net_state::NetStateId;
use muo_server::type_registry::TypeRegistry;

use super::{hue, random_min_max};

fn in_pack(w: &World, id: ItemId, from: MobileId) -> bool {
    let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    ops::is_child_of(w, id, pack)
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

fn msg(w: &mut World, to: MobileId, number: i32) {
    deps::send_localized_message(w, to, number);
}

fn mobile_of(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_states.get(&ns).and_then(|n| n.mobile)
}

fn cant_reach(w: &mut World, from: MobileId) {
    // I can't reach that.
    muo_server::mobile::part2::messages::local_overhead_message_number(
        w,
        from,
        MessageType::REGULAR,
        906,
        1019045,
        "",
    );
}

fn in_range_1(w: &World, id: ItemId, from: MobileId) -> bool {
    let loc = ops::get_world_location(w, id);
    deps::mobile_in_range(w, from, loc, 1)
}

/// `MondainsLegacy.CheckML(from, message)`.
pub fn check_ml(w: &mut World, from: MobileId, message: bool) -> bool {
    use muo_data::expansion::Expansion;
    let ns = w.mobile(from).and_then(|m| m.net_state);
    let ok = ns.is_some_and(|ns| {
        let (Some(n), Some(table)) = (w.net_state(ns), w.expansion_table()) else {
            return false;
        };
        n.caps.supports_expansion(
            table,
            table.get_info(Expansion::ML as i32),
            w.expansion() as i32,
            true,
        )
    });
    if ns.is_none() {
        return false;
    }
    if !ok && message {
        msg(w, from, 1072791); // You must upgrade to Mondain's Legacy in order to use that item.
    }
    ok
}

/// What the five dye gumps share: where the hue list is, which hair a response dyes.
#[derive(Copy, Clone)]
enum Dye {
    Hair,
    SpecialBeard,
    SpecialHair,
}

const HAIR_ENTRIES: [(i32, i32); 12] = [
    (1602, 26),
    (1628, 27),
    (1502, 32),
    (1302, 32),
    (1402, 32),
    (1202, 24),
    (2402, 29),
    (2213, 6),
    (1102, 8),
    (1110, 8),
    (1118, 16),
    (1134, 16),
];

const SPECIAL_ENTRIES: [(i32, i32); 8] = [
    (12, 10),
    (32, 5),
    (38, 8),
    (54, 3),
    (62, 10),
    (81, 2),
    (89, 2),
    (1153, 2),
];

pub struct DyeGump {
    dye: Dye,
    item: ItemId,
}

impl DyeGump {
    fn entries(&self) -> &'static [(i32, i32)] {
        match self.dye {
            Dye::Hair => &HAIR_ENTRIES,
            _ => &SPECIAL_ENTRIES,
        }
    }
}

macro_rules! dye_gump_logic {
    ($name:ident, $tn:literal, $dye:expr) => {
        pub struct $name(DyeGump);
        impl StaticGumpLogic for $name {
            const TYPE_NAME: &'static str = $tn;
            fn singleton(&self) -> bool {
                true
            }
            fn build_layout(&self, b: &mut StaticGumpBuilder) {
                self.0.build(b);
            }
            fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
                self.0.respond(w, sender, info);
            }
        }
    };
}
dye_gump_logic!(HairDyeGump, "Server.Items.HairDyeGump", Dye::Hair);
dye_gump_logic!(
    SpecialBeardDyeGump,
    "Server.Items.SpecialBeardDyeGump",
    Dye::SpecialBeard
);
dye_gump_logic!(
    SpecialHairDyeGump,
    "Server.Items.SpecialHairDyeGump",
    Dye::SpecialHair
);

impl DyeGump {
    fn build(&self, b: &mut StaticGumpBuilder) {
        let entries = self.entries();
        b.add_page(0);
        match self.dye {
            Dye::Hair => {
                b.add_background(100, 10, 350, 355, 2600);
                b.add_background(120, 54, 110, 270, 5100);
                b.add_html_localized(70, 25, 400, 35, 1011013, false, false); // <center>Hair Color Selection Menu</center>
                b.add_button(149, 328, 4005, 4007, 1);
                b.add_html_localized(185, 329, 250, 35, 1011014, false, false); // Dye my hair this color!
                for (i, e) in entries.iter().enumerate() {
                    let y = 59 + i as i32 * 22;
                    b.add_label(130, y, e.0 - 1, "*****");
                    b.add_button_type(
                        207,
                        y + 1,
                        5224,
                        5224,
                        0,
                        GumpButtonType::Page,
                        i as i32 + 1,
                    );
                }
                for (i, &(start, count)) in entries.iter().enumerate() {
                    b.add_page(i as i32 + 1);
                    let switch_id = i as i32 * 100;
                    for j in 0..count {
                        let (page, row) = (j / 16, j % 16);
                        let (x, y) = (260 + page * 80, 52 + row * 17);
                        b.add_radio(x, y, 210, 211, false, switch_id + j);
                        b.add_label(x + 18, y, start + j - 1, "*****");
                    }
                }
            }
            Dye::SpecialBeard | Dye::SpecialHair => {
                b.add_background(150, 60, 350, 358, 2600);
                b.add_background(170, 104, 110, 270, 5100);
                b.add_html_localized(230, 75, 200, 20, 1011013, false, false); // Hair Color Selection Menu
                if matches!(self.dye, Dye::SpecialBeard) {
                    b.add_html_localized(235, 380, 300, 20, 1013007, false, false); // Dye my beard this color!
                } else {
                    b.add_html_localized(235, 380, 300, 20, 1011014, false, false); // Dye my hair this color!
                }
                b.add_button(200, 380, 0xFA5, 0xFA7, 1); // DYE HAIR
                for (i, e) in entries.iter().enumerate() {
                    b.add_label(180, 109 + i as i32 * 22, e.0 - 1, "*****");
                    b.add_button_type(
                        257,
                        110 + i as i32 * 22,
                        5224,
                        5224,
                        0,
                        GumpButtonType::Page,
                        i as i32 + 1,
                    );
                }
                for (i, &(start, count)) in entries.iter().enumerate() {
                    b.add_page(i as i32 + 1);
                    for j in 0..count {
                        if matches!(self.dye, Dye::SpecialBeard) {
                            let (page, row) = (j / 16, j % 16);
                            let (x, y) = (310 + page * 80, 102 + row * 17);
                            b.add_radio(x, y, 210, 211, false, i as i32 * 100 + j);
                            b.add_label(x + 18, y, start + j - 1, "*****");
                        } else {
                            b.add_label(
                                328 + j / 16 * 80,
                                102 + j % 16 * 17,
                                start + j - 1,
                                "*****",
                            );
                            b.add_radio(
                                310 + j / 16 * 80,
                                102 + j % 16 * 17,
                                210,
                                211,
                                false,
                                i as i32 * 100 + j,
                            );
                        }
                    }
                }
            }
        }
    }

    fn respond(&self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        use muo_server::mobile::part1::props_b as pb;
        if ops::is_deleted(w, self.item) {
            return;
        }
        let Some(m) = mobile_of(w, sender) else {
            return;
        };
        let entries = self.entries();
        if !in_pack(w, self.item, m) {
            msg(w, m, 1042010); // You must have the object in your backpack to use it.
            return;
        }
        if info.button_id == 0 || info.switches.is_empty() {
            msg(w, m, 501200); // You decide not to dye your hair
            return;
        }
        let (hair, facial) = w
            .mobile(m)
            .map_or((0, 0), |x| (x.hair_item_id, x.facial_hair_item_id));
        let no_hair = match self.dye {
            Dye::Hair => hair == 0 && facial == 0,
            Dye::SpecialBeard => facial == 0,
            Dye::SpecialHair => hair == 0,
        };
        if no_hair {
            msg(w, m, 502623); // You have no hair to dye and cannot use this
            return;
        }
        // To prevent this from being exploited, the hue is abstracted into an internal list
        let (index, offset) = (info.switches[0] / 100, info.switches[0] % 100);
        if index < 0 || index as usize >= entries.len() {
            return;
        }
        let (start, count) = entries[index as usize];
        if offset >= 0 && offset < count {
            let h = start + offset;
            match self.dye {
                Dye::Hair => {
                    pb::set_hair_hue(w, m, h);
                    pb::set_facial_hair_hue(w, m, h);
                }
                Dye::SpecialBeard => pb::set_facial_hair_hue(w, m, h),
                Dye::SpecialHair => pb::set_hair_hue(w, m, h),
            }
            if matches!(self.dye, Dye::SpecialHair) {
                // Delete, then the hue
            }
            msg(w, m, 501199); // You dye your hair
            delete_item(w, self.item);
            muo_content::mx::play_sound(w, m, 0x4E);
        }
    }
}

fn send_dye_gump(w: &mut World, to: MobileId, dye: Dye, item: ItemId) {
    let d = DyeGump { dye, item };
    match dye {
        Dye::Hair => send_gump_to_mobile(
            w,
            to,
            Box::new(StaticGumpObj::new(50, 50, HairDyeGump(d))),
            false,
        ),
        Dye::SpecialBeard => send_gump_to_mobile(
            w,
            to,
            Box::new(StaticGumpObj::new(0, 0, SpecialBeardDyeGump(d))),
            false,
        ),
        Dye::SpecialHair => send_gump_to_mobile(
            w,
            to,
            Box::new(StaticGumpObj::new(0, 0, SpecialHairDyeGump(d))),
            false,
        ),
    }
}

item_leaf! {
    /// `Server.Items.HairDye`.
    HAIR_DYE = "Server.Items.HairDye" : ITEM_BASE, |w| ops::new_item_with_id(w, &HAIR_DYE, 0xEFF), encoded false;
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1041060, // Hair Dye
    on_double_click: |w, id, from| {
        if in_range_1(w, id, from) {
            send_dye_gump(w, from, Dye::Hair, id);
        } else {
            cant_reach(w, from);
        }
    },
}

/// `SpecialBeardDye()`.
pub fn special_beard_dye_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    crate::simple::from_template(w, ty, "Server.Items.SpecialBeardDye")
}

item_leaf! {
    /// `Server.Items.SpecialBeardDye`.
    SPECIAL_BEARD_DYE = "Server.Items.SpecialBeardDye" : ITEM_BASE, |w| special_beard_dye_new(w, &SPECIAL_BEARD_DYE), encoded false;
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1041087, // Special Beard Dye
    on_double_click: |w, id, from| {
        if in_range_1(w, id, from) {
            send_dye_gump(w, from, Dye::SpecialBeard, id);
        } else {
            cant_reach(w, from);
        }
    },
}

item_leaf! {
    /// `Server.Items.SpecialHairDye`.
    SPECIAL_HAIR_DYE = "Server.Items.SpecialHairDye" : ITEM_BASE, |w| special_beard_dye_new(w, &SPECIAL_HAIR_DYE), encoded false;
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1074402,
    on_double_click: |w, id, from| {
        if in_range_1(w, id, from) {
            send_dye_gump(w, from, Dye::SpecialHair, id);
        } else {
            cant_reach(w, from);
        }
    },
}

// ---- MelisandesHairDye ----

/// `MelisandesHairDye()`.
pub fn melisandes_hair_dye_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0xEFF);
    let h = random_min_max(w, 0x47E, 0x499);
    hue(w, id, h);
    id
}

/// `MelisandesHairDye.ConfirmGump` (a `BaseConfirmGump`: its layout with these numbers).
pub struct MelisandesConfirmGump {
    item: ItemId,
}

impl StaticGumpLogic for MelisandesConfirmGump {
    const TYPE_NAME: &'static str = "Server.Items.MelisandesHairDye+ConfirmGump";

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        build_confirm_layout(b, 1074395, 1074396); // Use Permanent Hair Dye / This special hair dye is made of ..
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id != 2 {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if !info.is_switched(1) {
            msg(w, from, 502620); // You decide not to dye your hair.
            return;
        }
        if !ops::is_deleted(w, self.item) && in_pack(w, self.item, from) {
            if w.mobile(from).is_some_and(|m| m.hair_item_id != 0) {
                let h = w.item(self.item).map_or(0, |i| i.hue);
                muo_server::mobile::part1::props_b::set_hair_hue(w, from, h);
                muo_content::mx::play_sound(w, from, 0x240);
                msg(w, from, 502622); // You dye your hair.
                delete_item(w, self.item);
            } else {
                msg(w, from, 502623); // You have no hair to dye and you cannot use this.
            }
        } else {
            msg(w, from, 1073461); // You don't have enough dye.
        }
    }
}

/// The body of `BaseConfirmGump`: the frame, the title and label, the two radios and the OK button (button 2, the
/// confirm radio is switch 1).
fn build_confirm_layout(b: &mut StaticGumpBuilder, title: i32, label: i32) {
    b.add_page(0);
    b.add_image_tiled(0, 0, 348, 262, 0xA8E);
    b.add_alpha_region(0, 0, 348, 262);
    b.add_image(0, 15, 0x27A8, 0);
    b.add_image_tiled(0, 30, 17, 200, 0x27A7);
    b.add_image(0, 230, 0x27AA, 0);
    b.add_image(15, 230, 0x280C, 0);
    b.add_image_tiled(30, 0, 300, 17, 0x280A);
    b.add_image(315, 0, 0x280E, 0);
    b.add_image(15, 244, 0x280C, 0);
    b.add_image_tiled(30, 244, 300, 17, 0x280A);
    b.add_image(315, 244, 0x280E, 0);
    b.add_image(330, 15, 0x27A8, 0);
    b.add_image_tiled(330, 30, 17, 200, 0x27A7);
    b.add_image(330, 230, 0x27AA, 0);
    b.add_image(333, 2, 0x2716, 0);
    b.add_image(315, 248, 0x2716, 0);
    b.add_image(2, 248, 0x2716, 0);
    b.add_image(2, 2, 0x2716, 0);
    b.add_html_localized_color(25, 25, 200, 20, title, 0x7D00, false, false);
    b.add_image(25, 40, 0xBBF, 0);
    b.add_html_localized_color(25, 55, 300, 120, label, 0x7FFF, false, false);
    b.add_radio(25, 175, 0x25F8, 0x25FB, true, 1);
    b.add_radio(25, 210, 0x25F8, 0x25FB, false, 0);
    b.add_html_localized_color(60, 180, 280, 20, 1074976, 0x7FFF, false, false);
    b.add_html_localized_color(60, 215, 280, 20, 1074977, 0x7FFF, false, false);
    b.add_button(265, 220, 0xF7, 0xF8, 2);
}

item_leaf! {
    /// `Server.Items.MelisandesHairDye`.
    MELISANDES_HAIR_DYE = "Server.Items.MelisandesHairDye" : ITEM_BASE, |w| melisandes_hair_dye_new(w, &MELISANDES_HAIR_DYE), encoded false;
    label_number: |_, _| 1041088, // Hair Dye
    get_properties: |w, id, list| {
        muo_server::item::base::get_properties_base(w, id, list);
        list.add(1075085); // Requirement: Mondain's Legacy
    },
    on_double_click: |w, id, from| {
        if in_pack(w, id, from) {
            if check_ml(w, from, true) {
                // `BaseConfirmGump`: x 120, y 50, not closable, disposable, draggable
                let mut g = StaticGumpObj::new(120, 50, MelisandesConfirmGump { item: id });
                let _ = &mut g;
                send_gump_to_mobile(w, from, Box::new(g), false);
            }
        } else {
            msg(w, from, 1042001); // That must be in your pack for you to use it.
        }
    },
}

// ---- PromotionalToken ----

/// What a promotional token class says (`ItemName`, `ItemReceiveMessage`, `ItemGumpName`) and makes.
pub struct TokenClass {
    pub item_name: fn() -> TextDefinition,
    pub receive_message: fn() -> TextDefinition,
    pub gump_name: fn() -> TextDefinition,
    /// `CreateItemFor(from)`.
    pub create_item_for: fn(&mut World, MobileId) -> Option<ItemId>,
}

fn token_class(w: &World, id: ItemId) -> Option<&'static TokenClass> {
    w.item(id)?.ty.family::<TokenClass>()
}

/// `PromotionalToken()`.
pub fn promotional_token_new(w: &mut World, ty: &'static ItemType) -> ItemId {
    let id = ops::new_item_with_id(w, ty, 0x2AAA);
    ops::set_loot_type(w, id, LootType::Blessed);
    ops::set_light(w, id, muo_server::light_type::LightType::Circle300);
    id
}

/// `PromotionalTokenGump`.
pub struct PromotionalTokenGump {
    token: ItemId,
    /// `ItemGumpName`'s cliloc number, read from the token class when the gump is made.
    name: i32,
}

impl DynamicGumpLogic for PromotionalTokenGump {
    const TYPE_NAME: &'static str = "Server.Items.PromotionalToken+PromotionalTokenGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 240, 135, 0x2422);
        // Click "OKAY" to redeem the following promotional item:
        b.add_html_localized_color(15, 15, 210, 75, 1070972, 0, true, false);
        b.add_html_localized(15, 60, 210, 75, self.name, false, false); // ItemGumpName.AddHtmlText
        b.add_button(160, 95, 0xF7, 0xF8, 1); // Okay
        b.add_button(90, 95, 0xF2, 0xF1, 0); // Cancel
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id != 1 {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if !in_pack(w, self.token, from) {
            msg(w, from, 1062334); // This item must be in your backpack to be used.
        } else if let Some(c) = token_class(w, self.token)
            && let Some(i) = (c.create_item_for)(w, from)
        {
            if let Some(bank) = muo_server::mobile::part1::props_b::bank_box(w, from)
                && let Some(bty) = w.item(bank).map(|b| b.ty)
            {
                (bty.add_item)(w, bank, i);
            }
            let m = (c.receive_message)();
            muo_content::text::send_message_to(w, Some(&m), from);
            delete_item(w, self.token);
        }
    }
}

fn promotional_token_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !in_pack(w, id, from) {
        msg(w, from, 1062334); // This item must be in your backpack to be used.
        return;
    }
    if w.mobile(from).and_then(|m| m.net_state).is_none() {
        return;
    }
    // shortcut: the gump name of the three tokens is a cliloc number; a literal string would need a Dynamic gump
    // with HTML text, none of the classes has one.
    let name = token_class(w, id).map_or(0, |c| (c.gump_name)().number);
    send_gump_to_mobile(
        w,
        from,
        Box::new(DynamicGumpObj::new(
            10,
            10,
            PromotionalTokenGump { token: id, name },
        )),
        false,
    );
}

/// `Server.Items.PromotionalToken` (abstract).
pub static PROMOTIONAL_TOKEN: ItemType = ItemType {
    full_name: "Server.Items.PromotionalToken",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_weight: |_, _| 5.0,
    label_number: |_, _| 1070997, // A promotional token
    get_properties: |w, id, list| {
        muo_server::item::base::get_properties_base(w, id, list);
        if let Some(c) = token_class(w, id) {
            let n = (c.item_name)();
            if n.number > 0 {
                list.add_localized_number(1070998, n.number); // Use this to redeem<br>your ~1_PROMO~
            } else if let Some(s) = &n.string {
                list.add_str(1070998, s);
            }
        }
    },
    on_double_click: promotional_token_double_click,
    on_removed: |w, id, parent| {
        let _ = id;
        let m = match parent {
            EntityId::Item(i) => match ops::root_parent(w, i) {
                Some(EntityId::Mobile(m)) => Some(m),
                _ => None,
            },
            EntityId::Mobile(m) => Some(m),
        };
        if let Some(ns) = m.and_then(|m| w.mobile(m)).and_then(|m| m.net_state) {
            muo_content::gumps::close_gump::<PromotionalTokenGump>(w, ns);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        super::write_version(out, false);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        super::read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

static SOULSTONE_FRAGMENT_TOKEN_CLASS: TokenClass = TokenClass {
    item_name: || TextDefinition::of_number(1071000), // soulstone fragment
    receive_message: || TextDefinition::of_number(1070976), // A soulstone fragment has been created in your bank box.
    gump_name: || TextDefinition::of_number(1070999),       // <center>Soulstone Fragment</center>
    create_item_for: |w, from| {
        // `from?.Account != null ? new SoulstoneFragment(from.Account.ToString()) : null`
        let account = w.mobile(from)?.account?;
        let name = muo_content::accounting::account(w, account)?
            .username
            .clone();
        match super::gadgets::create_with_args_pub(
            w,
            "Server.Items.SoulstoneFragment",
            &[Value::Str(name)],
        )? {
            EntityId::Item(i) => Some(i),
            _ => None,
        }
    },
};

/// `Server.Items.SoulstoneFragmentToken`.
pub static SOULSTONE_FRAGMENT_TOKEN: ItemType = ItemType {
    full_name: "Server.Items.SoulstoneFragmentToken",
    parent: Some(&PROMOTIONAL_TOKEN),
    construct: Some(|w| promotional_token_new(w, &SOULSTONE_FRAGMENT_TOKEN)),
    family: Some(&SOULSTONE_FRAGMENT_TOKEN_CLASS),
    serialize: |w, id, out| {
        (PROMOTIONAL_TOKEN.serialize)(w, id, out);
        out.write_i32(0); // the class's own `[SerializationGenerator(0, false)]` version
    },
    deserialize: |w, id, r| {
        (PROMOTIONAL_TOKEN.deserialize)(w, id, r)?;
        r.read_i32()?;
        Ok(())
    },
    ..PROMOTIONAL_TOKEN
};

// ---- ContractOfEmployment ----

/// `BaseHouse.IsThereVendor(location, map)`: `(vendor, rental contract)`.
pub type IsThereVendor = fn(&World, Point3D, Option<muo_server::map::MapId>) -> (bool, bool);

/// What `ContractOfEmployment` asks of the vendor system: `BaseHouse.IsThereVendor(loc, map, out vendor,
/// out contract)`. Unset: neither is there.
#[derive(Default)]
pub struct PlayerVendorHooks {
    pub is_there_vendor: Option<IsThereVendor>,
}

/// `new PlayerVendor(from, house) { Direction = from.Direction & Direction.Mask }` moved to the owner's place.
fn place_vendor(w: &mut World, from: MobileId, house: Option<ItemId>, contract: ItemId) {
    let args = [
        Value::Entity(Some(EntityId::Mobile(from))),
        Value::Entity(house.map(EntityId::Item)),
    ];
    let Some(EntityId::Mobile(v)) =
        super::gadgets::create_with_args_pub(w, "Server.Mobiles.PlayerVendor", &args)
    else {
        return; // no PlayerVendor constructor registered: nothing to place
    };
    let dir = w.mobile(from).map(|m| m.direction.bits() & 7);
    if let Some(d) = dir {
        muo_server::mobile::part1::props_b::set_direction(
            w,
            v,
            muo_primitives::Direction::from_bits_retain(d),
        );
    }
    if let Some((loc, Some(map))) = w.mobile(from).map(|m| (m.location, m.map)) {
        muo_server::mobile::base::move_to_world(w, v, loc, map);
    }
    // Ah! it feels good to be working again.
    muo_server::mobile::part2::speech::say_to_number(w, v, from, 503246, "");
    delete_item(w, contract);
}

fn contract_double_click(w: &mut World, id: ItemId, from: MobileId) {
    use muo_multis::house::{self, access};
    if !in_pack(w, id, from) {
        msg(w, from, 1042001); // That must be in your pack for you to use it.
    } else if w
        .mobile(from)
        .is_some_and(|m| m.access_level >= muo_server::mobile::AccessLevel::GameMaster)
    {
        msg(w, from, 503248); // Your godly powers allow you to place this vendor whereever you wish.
        let h = access::find_house_at_mobile(w, from);
        place_vendor(w, from, h, id);
    } else {
        let Some(h) = access::find_house_at_mobile(w, from) else {
            msg(w, from, 503240); // Vendors can only be placed in houses.
            return;
        };
        let new_system = house::new_vendor_system(w);
        if !new_system && !access::is_friend(w, h, from) {
            // You must ask the owner of this building to name you a friend of the household in order to place a vendor here.
            msg(w, from, 503242);
        } else if new_system && !access::is_owner(w, h, from) {
            // Only the house owner can directly place vendors.  Please ask the house owner to offer you a vendor contract so that you may place a vendor in this house.
            msg(w, from, 1062423);
        } else if !house::d(w, h).public || !house::storage::can_place_new_vendor(w, h) {
            // You cannot place this vendor or barkeep.  Make sure the house is public and has sufficient storage available.
            msg(w, from, 503241);
        } else {
            let (loc, map) = w
                .mobile(from)
                .map_or((Point3D::default(), None), |m| (m.location, m.map));
            let (vendor, contract) = w
                .resources
                .get::<PlayerVendorHooks>()
                .and_then(|x| x.is_there_vendor)
                .map_or((false, false), |f| f(w, loc, map));
            if vendor {
                msg(w, from, 1062677); // You cannot place a vendor or barkeep at this location.
            } else if contract {
                // You cannot place a vendor or barkeep on top of a rental contract!
                msg(w, from, 1062678);
            } else {
                place_vendor(w, from, Some(h), id);
            }
        }
    }
}

item_leaf! {
    /// `Server.Items.ContractOfEmployment`.
    CONTRACT_OF_EMPLOYMENT = "Server.Items.ContractOfEmployment" : ITEM_BASE, |w| ops::new_item_with_id(w, &CONTRACT_OF_EMPLOYMENT, 0x14F0), encoded false;
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1041243, // a contract of employment
    on_double_click: contract_double_click,
}

/// The classes of this file.
pub static ALL: &[&ItemType] = &[
    &HAIR_DYE,
    &SPECIAL_BEARD_DYE,
    &SPECIAL_HAIR_DYE,
    &MELISANDES_HAIR_DYE,
    &PROMOTIONAL_TOKEN,
    &SOULSTONE_FRAGMENT_TOKEN,
    &CONTRACT_OF_EMPLOYMENT,
];

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
}
