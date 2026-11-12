//! `BaseImageTileButtonsGump`. A subclass keeps its `ImageTileButtonInfo` list, builds the layout
//! with [`build`] (passing its `XItems`/`YItems` overrides) and, in `on_response`, asks [`adjusted_button`] which
//! button was pressed: `Some(i)` is `HandleButtonResponse(sender, i, buttons[i])`, `None` is `HandleCancel`.

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::legacy::LegacyLayout;
use muo_content::text::add_html_text_gump;
use muo_io::text::TextDefinition;

use super::common::draw_legacy;

/// `ImageTileButtonInfo`.
#[derive(Clone, Debug)]
pub struct ImageTileButtonInfo {
    pub item_id: i32,
    pub hue: i32,
    pub label: TextDefinition,
    pub localized_tooltip: i32,
}

impl ImageTileButtonInfo {
    /// `new ImageTileButtonInfo(itemID, hue, label, localizedTooltip = -1)`.
    pub fn new(item_id: i32, hue: i32, label: TextDefinition) -> Self {
        Self {
            item_id,
            hue,
            label,
            localized_tooltip: -1,
        }
    }
}

/// The constructor. Coords are 0, 0 on OSI, intentional difference: the gump is at (10, 10).
pub fn build(
    l: &mut LegacyLayout,
    header: &TextDefinition,
    buttons: &[ImageTileButtonInfo],
    x_items: i32,
    y_items: i32,
) {
    l.add_page(0);

    let x = x_items * 250;
    let y = y_items * 64;

    l.add_background(0, 0, x + 20, y + 84, 0x13BE);
    l.add_image_tiled(10, 10, x, 20, 0xA40);
    l.add_image_tiled(10, 40, x, y + 4, 0xA40);
    l.add_image_tiled(10, y + 54, x, 20, 0xA40);
    l.add_alpha_region(10, 10, x, y + 64);

    l.add_button(10, y + 54, 0xFB1, 0xFB2, 0); // Cancel Button
    l.add_html_localized_color(45, y + 56, x - 50, 20, 1060051, 0x7FFF, false, false); // CANCEL
    draw_legacy(
        l,
        add_html_text_gump(Some(header), false, false, 0x7FFF, 0xFFFFFF),
        14,
        12,
        x,
        20,
    );

    l.add_page(1);

    let items_per_page = x_items * y_items;

    for (i, b) in buttons.iter().enumerate() {
        let i = i as i32;
        let position = i % items_per_page;

        let inner_x = position % x_items * 250 + 14;
        let inner_y = position / x_items * 64 + 44;

        let page_num = i / items_per_page + 1;

        if position == 0 && i != 0 {
            l.add_button_type(
                x - 100,
                y + 54,
                0xFA5,
                0xFA7,
                0,
                GumpButtonType::Page,
                page_num,
            );
            l.add_html_localized_color(x - 60, y + 56, 60, 20, 1043353, 0x7FFF, false, false); // Next

            l.add_page(page_num);

            l.add_button_type(
                x - 200,
                y + 54,
                0xFAE,
                0xFB0,
                0,
                GumpButtonType::Page,
                page_num - 1,
            );
            l.add_html_localized_color(x - 160, y + 56, 60, 20, 1011393, 0x7FFF, false, false); // Back
        }

        l.add_image_tiled_button(
            inner_x,
            inner_y,
            0x918,
            0x919,
            100 + i,
            GumpButtonType::Reply,
            0,
            b.item_id,
            b.hue,
            15,
            10,
        );
        l.add_tooltip(b.localized_tooltip, None);
        draw_legacy(
            l,
            add_html_text_gump(Some(&b.label), false, false, 0x7FFF, 0xFFFFFF),
            inner_x + 84,
            inner_y,
            250,
            60,
        );
    }
}

/// The index into the buttons of a response, or None for a cancel (`HandleCancel`).
pub fn adjusted_button(button_id: i32, count: usize) -> Option<usize> {
    let adjusted = button_id - 100;
    (adjusted >= 0 && (adjusted as usize) < count).then_some(adjusted as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_below_100_cancel() {
        assert_eq!(adjusted_button(0, 3), None);
        assert_eq!(adjusted_button(100, 3), Some(0));
        assert_eq!(adjusted_button(102, 3), Some(2));
        assert_eq!(adjusted_button(103, 3), None);
    }
}
