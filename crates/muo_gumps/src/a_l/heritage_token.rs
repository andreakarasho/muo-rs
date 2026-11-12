//! `HeritageTokenGump`.

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::net_state::NetStateId;

use super::common::{mobile_of, send};
use super::confirm_heritage;

/// `ButtonLookup`, in the dictionary's insertion order: (button id, item types, cliloc, tooltip, item id, hue,
/// width, height).
#[rustfmt::skip]
#[allow(clippy::type_complexity)] // the button's item-type list and layout, spelled out
const BUTTON_LOOKUP: [(i32, &[&str], i32, i32, i32, i32, i32, i32); 67] = [
    (0x64, &["Server.Items.LeggingsOfEmbers"], 1078147, 1062912, 0x1411, 0x2C, 18, 8),
    (0x65, &["Server.Items.RoseOfTrinsic"], 1062913, 1062914, 0x234D, 0x0, 18, 12),
    (0x66, &["Server.Items.ShaminoCrossbow"], 1062915, 1062916, 0x26C3, 0x504, 18, 8),
    (0x67, &["Server.Items.TapestryOfSosaria"], 1062917, 1062918, 0x3F1D, 0x0, 18, 8),
    (0x68, &["Server.Items.HearthOfHomeFireDeed"], 1062919, 1062920, 0x3F14, 0x0, 18, 8),
    (0x69, &["Server.Items.HolySword"], 1062921, 1062922, 0xF60, 0x482, -1, 10),
    (0x6A, &["Server.Items.SamuraiHelm"], 1062923, 1062924, 0x236C, 0x0, 18, 6),
    (0x6B, &["Server.Items.SpiritualityHelm"], 1075188, 1075223, 0x2B10, 0x226, 18, 11),
    (0x6C, &["Server.Items.ValorGauntlets"], 1075192, 1075224, 0x2B0C, 0x226, 18, 15),
    (0x6D, &["Server.Items.DupresShield"], 1075196, 1075225, 0x2B01, 0x0, 18, 9),
    (0x6E, &["Server.Items.FountainOfLifeDeed"], 1075197, 1075226, 0x2AC6, 0x0, 29, 0),
    (0x6F, &["Server.Items.DawnsMusicBox"], 1075198, 1075227, 0x2AF9, 0x0, -4, -5),
    (0x70, &["Server.Items.OssianGrimoire"], 1078148, 1075228, 0x2253, 0x0, 18, 12),
    (0x71, &["Server.Items.FerretFormTalisman"], 1078142, 1078527, 0x2D98, 0x0, 19, 13),
    (0x72, &["Server.Items.SquirrelFormTalisman"], 1078143, 1078528, 0x2D97, 0x0, 19, 13),
    (0x73, &["Server.Items.CuSidheFormTalisman"], 1078144, 1078529, 0x2D96, 0x0, 19, 8),
    (0x74, &["Server.Items.ReptalonFormTalisman"], 1078145, 1078530, 0x2D95, 0x0, -4, 2),
    (0x75, &["Server.Items.QuiverOfInfinity"], 1075201, 1078526, 0x2B02, 0x0, -2, 9),
    (0x76, &["Server.Items.BoneThroneDeed", "Server.Items.BoneCouchDeed", "Server.Items.BoneTableDeed"], 1074797, 1075986, 0x2A91, 0x0, 25, 5),
    (0x77, &["Server.Items.CreepyPortraitDeed", "Server.Items.DisturbingPortraitDeed", "Server.Items.UnsettlingPortraitDeed"], 1078146, 1075987, 0x2A99, 0x0, 18, 1),
    (0x78, &["Server.Items.MountedPixieBlueDeed", "Server.Items.MountedPixieGreenDeed", "Server.Items.MountedPixieLimeDeed", "Server.Items.MountedPixieOrangeDeed", "Server.Items.MountedPixieWhiteDeed"], 1074799, 1075988, 0x2A71, 0x0, 13, 5),
    (0x79, &["Server.Items.HaunterMirrorDeed"], 1074800, 1075990, 0x2A98, 0x0, 26, 1),
    (0x7A, &["Server.Items.BedOfNailsDeed"], 1074801, 1075989, 0x2A92, 0x0, 18, 1),
    (0x7B, &["Server.Items.SacrificialAltarDeed"], 1074818, 1075991, 0x2AB8, 0x0, 18, 1),
    (0x7C, &["Server.Items.BrokenCoveredChairDeed"], 1076257, 1076610, 0x3F26, 0x0, 18, 8),
    (0x7D, &["Server.Items.BrokenBookcaseDeed"], 1076258, 1076610, 0x3F22, 0x0, 18, 8),
    (0x7E, &["Server.Items.StandingBrokenChairDeed"], 1076259, 1076610, 0x3F24, 0x0, 18, 8),
    (0x7F, &["Server.Items.BrokenVanityDeed"], 1076260, 1076610, 0x3F25, 0x0, 18, 8),
    (0x80, &["Server.Items.BrokenChestOfDrawersDeed"], 1076261, 1076610, 0x3F23, 0x0, 18, 8),
    (0x81, &["Server.Items.BrokenArmoireDeed"], 1076262, 1076610, 0x3F21, 0x0, 18, 8),
    (0x82, &["Server.Items.BrokenBedDeed"], 1076263, 1076610, 0x3F0B, 0x0, 18, 8),
    (0x83, &["Server.Items.BrokenFallenChairDeed"], 1076264, 1076610, 0xC19, 0x0, 13, 8),
    (0x84, &["Server.Items.SuitOfGoldArmorDeed"], 1076265, 1076611, 0x3DAA, 0x0, 20, -3),
    (0x85, &["Server.Items.SuitOfSilverArmorDeed"], 1076266, 1076612, 0x151C, 0x0, -20, -3),
    (0x86, &["Server.Items.BoilingCauldronDeed"], 1076267, 1076613, 0x3DB1, 0x0, 18, 8),
    (0x87, &["Server.Items.GuillotineDeed"], 1024656, 1076614, 0x3F27, 0x0, 18, 8),
    (0x88, &["Server.Items.CherryBlossomTreeDeed"], 1076268, 1076615, 0x3F0C, 0x0, 18, 8),
    (0x89, &["Server.Items.AppleTreeDeed"], 1076269, 1076616, 0x3F07, 0x0, 18, 8),
    (0x8A, &["Server.Items.PeachTreeDeed"], 1076270, 1076617, 0x3F16, 0x0, 18, 8),
    (0x8B, &["Server.Items.HangingAxesDeed"], 1076271, 1076618, 0x3F12, 0x0, 18, 8),
    (0x8C, &["Server.Items.HangingSwordsDeed"], 1076272, 1076619, 0x3F13, 0x0, 18, 8),
    (0x8D, &["Server.Items.BlueFancyRugDeed"], 1076273, 1076620, 0x3F09, 0x0, 18, 8),
    (0x8E, &["Server.Items.WoodenCoffinDeed"], 1076274, 1076621, 0x3F0E, 0x0, 18, 8),
    (0x8F, &["Server.Items.VanityDeed"], 1074027, 1076623, 0x3F1F, 0x0, 18, 8),
    (0x90, &["Server.Items.TableWithPurpleClothDeed"], 1076635, 1076624, 0x118B, 0x0, -4, -9),
    (0x91, &["Server.Items.TableWithBlueClothDeed"], 1076636, 1076624, 0x118C, 0x0, -4, -9),
    (0x92, &["Server.Items.TableWithRedClothDeed"], 1076637, 1076624, 0x118D, 0x0, -4, -9),
    (0x93, &["Server.Items.TableWithOrangeClothDeed"], 1076638, 1076624, 0x118E, 0x0, -4, -9),
    (0x94, &["Server.Items.UnmadeBedDeed"], 1076279, 1076625, 0x3F1E, 0x0, 18, 8),
    (0x95, &["Server.Items.CurtainsDeed"], 1076280, 1076626, 0x3F0F, 0x0, 18, 8),
    (0x96, &["Server.Items.ScarecrowDeed"], 1076281, 1076627, 0x1E34, 0x0, 18, -17),
    (0x97, &["Server.Items.WallTorchDeed"], 1076282, 1076628, 0xA0C, 0x0, 18, 8),
    (0x98, &["Server.Items.FountainDeed"], 1076283, 1076629, 0x3F10, 0x0, 18, 9),
    (0x99, &["Server.Items.StoneStatueDeed"], 1076284, 1076630, 0x3F19, 0x0, 18, 8),
    (0x9A, &["Server.Items.LargeFishingNetDeed"], 1076285, 1076631, 0x1EA5, 0x0, 5, -25),
    (0x9B, &["Server.Items.SmallFishingNetDeed"], 1076286, 1076632, 0x1EA3, 0x0, 18, -27),
    (0x9C, &["Server.Items.HouseLadderDeed"], 1076287, 1076633, 0x2FDF, 0x0, 18, -36),
    (0x9D, &["Server.Items.IronMaidenDeed"], 1076288, 1076622, 0x3F15, 0x0, 18, 8),
    (0x9E, &["Server.Items.BluePlainRugDeed"], 1076585, 1076620, 0x3F0A, 0x0, 18, 8),
    (0x9F, &["Server.Items.GoldenDecorativeRugDeed"], 1076586, 1076620, 0x3F11, 0x0, 18, 8),
    (0xA0, &["Server.Items.CinnamonFancyRugDeed"], 1076587, 1076620, 0x3F0D, 0x0, 18, 8),
    (0xA1, &["Server.Items.RedPlainRugDeed"], 1076588, 1076620, 0x3F18, 0x0, 18, 8),
    (0xA2, &["Server.Items.BlueDecorativeRugDeed"], 1076589, 1076620, 0x3F08, 0x0, 18, 8),
    (0xA3, &["Server.Items.PinkFancyRugDeed"], 1076590, 1076620, 0x3F17, 0x0, 18, 8),
    (0xA4, &["Server.Items.CherryBlossomTrunkDeed"], 1076784, 1076615, 0x312A, 0x0, 18, 8),
    (0xA5, &["Server.Items.AppleTrunkDeed"], 1076785, 1076616, 0x3128, 0x0, 18, 8),
    (0xA6, &["Server.Items.PeachTrunkDeed"], 1076786, 1076617, 0x3129, 0x0, 18, 8),
];

pub struct HeritageTokenGump {
    token: ItemId,
}

/// `DisplayTo(from, token)`.
pub fn display_to(w: &mut World, from: MobileId, token: ItemId) {
    if w.mobile(from).and_then(|m| m.net_state).is_none()
        || w.item(token).is_none_or(|i| i.deleted())
    {
        return;
    }
    send(
        w,
        from,
        StaticGumpObj::new(60, 36, HeritageTokenGump { token }),
    );
}

#[cfg(test)]
impl HeritageTokenGump {
    pub fn for_test() -> Self {
        Self {
            token: ItemId(muo_primitives::Serial::new(1)),
        }
    }
}

impl StaticGumpLogic for HeritageTokenGump {
    const TYPE_NAME: &'static str = "Server.Gumps.HeritageTokenGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);

        // Standard static framing
        b.add_background(0, 0, 520, 404, 0x13BE);
        b.add_image_tiled(10, 10, 500, 20, 0xA40);
        b.add_image_tiled(10, 40, 500, 324, 0xA40);
        b.add_image_tiled(10, 374, 500, 20, 0xA40);
        b.add_alpha_region(10, 10, 500, 384);
        b.add_button(10, 374, 0xFB1, 0xFB2, 0);
        b.add_html_localized_color(45, 376, 450, 20, 1060051, 0x7FFF, false, false); // CANCEL
        b.add_html_localized_color(14, 12, 500, 20, 1075576, 0x7FFF, false, false); // Choose your item from the following pages

        const ITEMS_PER_PAGE: i32 = 10;
        let total_items = BUTTON_LOOKUP.len() as i32;
        let total_pages = (total_items + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;

        let mut index = 0;
        let mut page = 1;

        for &(button_id, _, cliloc, tooltip, item_id, hue, width, height) in &BUTTON_LOOKUP {
            // When index cleanly divides by 10, we are starting a new page
            if index % ITEMS_PER_PAGE == 0 {
                b.add_page(page);

                // Add Next Button
                if page < total_pages {
                    b.add_button_type(400, 374, 0xFA5, 0xFA7, 0, GumpButtonType::Page, page + 1);
                    b.add_html_localized_color(440, 376, 60, 20, 1043353, 0x7FFF, false, false); // Next
                }

                // Add Back Button
                if page > 1 {
                    b.add_button_type(300, 374, 0xFAE, 0xFB0, 0, GumpButtonType::Page, page - 1);
                    b.add_html_localized_color(340, 376, 60, 20, 1011393, 0x7FFF, false, false); // Back
                }
            }

            // Calculate dynamic grid placement (2 columns, 5 rows)
            let pos_on_page = index % ITEMS_PER_PAGE;
            let col = pos_on_page % 2; // 0 for left, 1 for right
            let row = pos_on_page / 2; // 0 through 4

            let x_base = if col == 0 { 14 } else { 264 };
            let y_base = 44 + row * 64;
            let text_x = if col == 0 { 98 } else { 348 };

            // Render the UI entries
            b.add_image_tiled_button(
                x_base,
                y_base,
                0x918,
                0x919,
                button_id,
                GumpButtonType::Reply,
                0,
                item_id,
                hue,
                width,
                height,
                -1,
            );

            if tooltip > 0 {
                b.add_tooltip(tooltip);
            }

            b.add_html_localized_color(text_x, y_base, 250, 60, cliloc, 0x7FFF, false, false);

            index += 1;

            // Bump to the next page when we fill the current one
            if index % ITEMS_PER_PAGE == 0 {
                page += 1;
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if w.item(self.token).is_none_or(|i| i.deleted()) || info.button_id == 0 {
            return;
        }
        let Some(&(_, types, cliloc, ..)) = BUTTON_LOOKUP.iter().find(|e| e.0 == info.button_id)
        else {
            return;
        };
        let Some(from) = mobile_of(w, sender) else {
            return;
        };

        if !types.is_empty() && cliloc > 0 {
            confirm_heritage::display_to(w, from, self.token, types, cliloc);
        } else {
            // This option is currently disabled, while we evaluate it for game balance.
            muo_content::mx::msg(w, from, 501311);
        }
    }
}
