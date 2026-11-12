//! The gump builders: `StaticGumpBuilder`, `DynamicGumpBuilder` and `GumpStringsBuilder`.
//!
//! One [`GumpBuilder`] serves both: the dynamic builder is the static one without placeholders. Text (labels,
//! html, entries) is written into the strings table and the layout refers to it by index; index 0 of a static
//! gump is an empty string kept for slots nobody fills.
//!
//! shortcut: `StaticGump` cached its compressed layout after the first instance; here the layout is built for
//! each instance (same bytes, a little more work per send). Upgrade with a per-class cache if gump sends show up
//! in a profile.

use std::collections::HashMap;

use muo_primitives::Serial;
use muo_primitives::html::{TextAlignment, build as build_html};

use super::layout::{GumpButtonType, LayoutBuilder};

/// Appends `len` (u16, UTF-16 units) and the text in big-endian UTF-16, truncated at 65535 characters.
fn write_string(buf: &mut Vec<u8>, text: &str) {
    let units: Vec<u16> = text.encode_utf16().take(usize::from(u16::MAX)).collect();
    buf.extend_from_slice(&(units.len() as u16).to_be_bytes());
    for u in units {
        buf.extend_from_slice(&u.to_be_bytes());
    }
}

/// Html with the optional wrapper `AddHtml(.., color, size, fontStyle, align, ..)` takes.
#[derive(Clone, Copy, Debug, Default)]
pub struct HtmlStyle<'a> {
    pub color: &'a str,
    /// -1 is no size.
    pub size: i32,
    pub font_style: u8,
    pub align: TextAlignment,
}

impl HtmlStyle<'_> {
    pub const NONE: HtmlStyle<'static> = HtmlStyle {
        color: "",
        size: -1,
        font_style: 0,
        align: TextAlignment::Left,
    };

    fn is_plain(&self) -> bool {
        self.color.is_empty()
            && self.size == -1
            && self.font_style == 0
            && self.align == TextAlignment::Left
    }
}

/// `StaticGumpBuilder` / `DynamicGumpBuilder`.
pub struct GumpBuilder {
    pub layout: LayoutBuilder,
    strings: Vec<u8>,
    pub(crate) strings_count: i32,
    /// Where each placeholder's index goes: (slot key, offset in the layout).
    slot_offsets: Vec<(String, usize)>,
}

pub type StaticGumpBuilder = GumpBuilder;
pub type DynamicGumpBuilder = GumpBuilder;

impl GumpBuilder {
    /// `new StaticGumpBuilder()`: string 0 is reserved empty so an unfilled slot has something to show.
    pub fn new_static() -> Self {
        let mut strings = Vec::new();
        strings.extend_from_slice(&[0, 0]);
        Self {
            layout: LayoutBuilder::new(),
            strings,
            strings_count: 1,
            slot_offsets: Vec::new(),
        }
    }

    /// `new DynamicGumpBuilder()`.
    pub fn new_dynamic() -> Self {
        Self {
            layout: LayoutBuilder::new(),
            strings: Vec::new(),
            strings_count: 0,
            slot_offsets: Vec::new(),
        }
    }

    pub fn switches(&self) -> i32 {
        self.layout.switches
    }

    pub fn text_entries(&self) -> i32 {
        self.layout.text_entries
    }

    pub fn has_visual_elements(&self) -> bool {
        self.layout.has_visual_elements
    }

    pub fn layout_data(&self) -> &[u8] {
        self.layout.data()
    }

    pub fn strings_data(&self) -> &[u8] {
        &self.strings
    }

    pub fn has_slots(&self) -> bool {
        !self.slot_offsets.is_empty()
    }

    fn intern(&mut self, text: &str) -> i32 {
        write_string(&mut self.strings, text);
        let i = self.strings_count;
        self.strings_count += 1;
        i
    }

    pub fn set_no_close(&mut self) {
        self.layout.set_no_close();
    }
    pub fn set_no_move(&mut self) {
        self.layout.set_no_move();
    }
    pub fn set_no_resize(&mut self) {
        self.layout.set_no_resize();
    }
    pub fn set_no_dispose(&mut self) {
        self.layout.set_no_dispose();
    }

    pub fn add_alpha_region(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.layout.add_alpha_region(x, y, width, height);
    }

    pub fn add_background(&mut self, x: i32, y: i32, width: i32, height: i32, gump_id: i32) {
        self.layout.add_background(x, y, width, height, gump_id);
    }

    /// `AddButton(.., GumpButtonType.Reply, 0)`.
    pub fn add_button(&mut self, x: i32, y: i32, normal_id: i32, pressed_id: i32, button_id: i32) {
        self.layout.add_button(
            x,
            y,
            normal_id,
            pressed_id,
            button_id,
            GumpButtonType::Reply,
            0,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_button_type(
        &mut self,
        x: i32,
        y: i32,
        normal_id: i32,
        pressed_id: i32,
        button_id: i32,
        kind: GumpButtonType,
        param: i32,
    ) {
        self.layout
            .add_button(x, y, normal_id, pressed_id, button_id, kind, param);
    }

    pub fn add_checkbox(
        &mut self,
        x: i32,
        y: i32,
        inactive_id: i32,
        active_id: i32,
        selected: bool,
        switch_id: i32,
    ) {
        self.layout
            .add_checkbox(x, y, inactive_id, active_id, selected, switch_id);
    }

    pub fn add_group(&mut self, group_id: i32) {
        self.layout.add_group(group_id);
    }

    /// `AddHtmlPlaceholder(x, y, w, h, slotKey, background, scrollbar)`.
    #[allow(clippy::too_many_arguments)] // one argument per gump entry field
    pub fn add_html_placeholder(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        slot_key: &str,
        background: bool,
        scrollbar: bool,
    ) {
        let at = self
            .layout
            .add_html_placeholder(x, y, width, height, background, scrollbar);
        self.slot_offsets.push((slot_key.to_owned(), at));
    }

    /// `AddHtml(x, y, w, h, text, color, size, fontStyle, align, background, scrollbar)`.
    #[allow(clippy::too_many_arguments)]
    pub fn add_html_styled(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        text: &str,
        style: HtmlStyle<'_>,
        background: bool,
        scrollbar: bool,
    ) {
        let index = if style.is_plain() {
            self.intern(text)
        } else {
            let built = build_html(text, style.color, style.size, style.font_style, style.align);
            self.intern(&built)
        };
        self.layout
            .add_html(x, y, width, height, index, background, scrollbar);
    }

    /// `AddHtml(x, y, w, h, text, background, scrollbar)`.
    #[allow(clippy::too_many_arguments)] // one argument per gump entry field
    pub fn add_html(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        text: &str,
        background: bool,
        scrollbar: bool,
    ) {
        self.add_html_styled(
            x,
            y,
            width,
            height,
            text,
            HtmlStyle::NONE,
            background,
            scrollbar,
        );
    }

    #[allow(clippy::too_many_arguments)] // one argument per gump entry field
    pub fn add_html_localized(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        number: i32,
        background: bool,
        scrollbar: bool,
    ) {
        self.layout
            .add_html_localized(x, y, width, height, number, background, scrollbar);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_html_localized_color(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        number: i32,
        color: i32,
        background: bool,
        scrollbar: bool,
    ) {
        self.layout
            .add_html_localized_color(x, y, width, height, number, color, background, scrollbar);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_html_localized_args(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        number: i32,
        args: &str,
        color: i32,
        background: bool,
        scrollbar: bool,
    ) {
        self.layout.add_html_localized_args(
            x, y, width, height, number, args, color, background, scrollbar,
        );
    }

    pub fn add_image(&mut self, x: i32, y: i32, gump_id: i32, hue: i32) {
        self.layout.add_image(x, y, gump_id, hue, "");
    }

    pub fn add_image_class(&mut self, x: i32, y: i32, gump_id: i32, hue: i32, cls: &str) {
        self.layout.add_image(x, y, gump_id, hue, cls);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_image_tiled_button(
        &mut self,
        x: i32,
        y: i32,
        normal_id: i32,
        pressed_id: i32,
        button_id: i32,
        kind: GumpButtonType,
        param: i32,
        item_id: i32,
        hue: i32,
        width: i32,
        height: i32,
        localized_tooltip: i32,
    ) {
        self.layout.add_image_tiled_button(
            x,
            y,
            normal_id,
            pressed_id,
            button_id,
            kind,
            param,
            item_id,
            hue,
            width,
            height,
            localized_tooltip,
        );
    }

    pub fn add_image_tiled(&mut self, x: i32, y: i32, width: i32, height: i32, gump_id: i32) {
        self.layout.add_image_tiled(x, y, width, height, gump_id);
    }

    pub fn add_item(&mut self, x: i32, y: i32, item_id: i32, hue: i32) {
        self.layout.add_item(x, y, item_id, hue);
    }

    pub fn add_item_property(&mut self, serial: Serial) {
        self.layout.add_item_property(serial);
    }

    pub fn add_label_placeholder(&mut self, x: i32, y: i32, hue: i32, slot_key: &str) {
        let at = self.layout.add_label_placeholder(x, y, hue);
        self.slot_offsets.push((slot_key.to_owned(), at));
    }

    pub fn add_label(&mut self, x: i32, y: i32, hue: i32, text: &str) {
        let i = self.intern(text);
        self.layout.add_label(x, y, hue, i);
    }

    pub fn add_label_cropped_placeholder(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
        slot_key: &str,
    ) {
        let at = self
            .layout
            .add_label_cropped_placeholder(x, y, width, height, hue);
        self.slot_offsets.push((slot_key.to_owned(), at));
    }

    pub fn add_label_cropped(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
        text: &str,
    ) {
        let i = self.intern(text);
        self.layout.add_label_cropped(x, y, width, height, hue, i);
    }

    pub fn add_gump_id_override(&mut self, gump_id: i32) {
        self.layout.add_gump_id_override(gump_id);
    }

    pub fn add_page(&mut self, page: i32) {
        self.layout.add_page(page);
    }

    pub fn add_radio(
        &mut self,
        x: i32,
        y: i32,
        inactive_id: i32,
        active_id: i32,
        selected: bool,
        switch_id: i32,
    ) {
        self.layout
            .add_radio(x, y, inactive_id, active_id, selected, switch_id);
    }

    #[allow(clippy::too_many_arguments)] // one argument per gump entry field
    pub fn add_sprite_image(
        &mut self,
        x: i32,
        y: i32,
        gump_id: i32,
        width: i32,
        height: i32,
        sx: i32,
        sy: i32,
    ) {
        self.layout
            .add_sprite_image(x, y, gump_id, width, height, sx, sy);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_text_entry_placeholder(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
        entry_id: i32,
        slot_key: &str,
    ) {
        let at = self
            .layout
            .add_text_entry_placeholder(x, y, width, height, hue, entry_id);
        self.slot_offsets.push((slot_key.to_owned(), at));
    }

    #[allow(clippy::too_many_arguments)] // one argument per gump entry field
    pub fn add_text_entry(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
        entry_id: i32,
        initial_text: &str,
    ) {
        let i = self.intern(initial_text);
        self.layout
            .add_text_entry(x, y, width, height, hue, entry_id, i);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_text_entry_limited_placeholder(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
        entry_id: i32,
        slot_key: &str,
        size: i32,
    ) {
        let at = self
            .layout
            .add_text_entry_limited_placeholder(x, y, width, height, hue, entry_id, size);
        self.slot_offsets.push((slot_key.to_owned(), at));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_text_entry_limited(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
        entry_id: i32,
        initial_text: &str,
        size: i32,
    ) {
        let i = self.intern(initial_text);
        self.layout
            .add_text_entry_limited(x, y, width, height, hue, entry_id, i, size);
    }

    pub fn add_tooltip(&mut self, number: i32) {
        self.layout.add_tooltip(number);
    }

    pub fn add_tooltip_args(&mut self, number: i32, args: &str) {
        self.layout.add_tooltip_args(number, args);
    }

    pub fn finalize_layout(&mut self) {
        self.layout.finalize();
    }

    /// `GumpStringsBuilder.FinalizeStrings(ref builder)`: points every filled placeholder at its string, which
    /// comes after the static strings.
    pub fn finalize_strings(&mut self, dynamic: &GumpStrings) {
        let starting_index = self.strings_count as usize;
        let offsets = std::mem::take(&mut self.slot_offsets);
        for (key, offset) in &offsets {
            if let Some(index) = dynamic.slots.get(key) {
                self.layout
                    .write_slot_index(*offset, index + starting_index);
            }
        }
        self.slot_offsets = offsets;
    }
}

/// `GumpStringsBuilder`: the strings that fill a static gump's slots, in the order they are set.
#[derive(Default)]
pub struct GumpStrings {
    buf: Vec<u8>,
    pub(crate) count: i32,
    /// Slot key -> the string's index among these strings (the last `SetStringSlot` of a key wins).
    slots: HashMap<String, usize>,
}

impl GumpStrings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data(&self) -> &[u8] {
        &self.buf
    }

    /// `SetStringSlot(slotKey, text)`.
    pub fn set_string_slot(&mut self, slot_key: &str, text: &str) {
        write_string(&mut self.buf, text);
        self.slots.insert(slot_key.to_owned(), self.count as usize);
        self.count += 1;
    }

    /// `SetHtmlText(slotKey, value, color, size, fontStyle, align)`.
    pub fn set_html_text_styled(&mut self, slot_key: &str, value: &str, style: HtmlStyle<'_>) {
        let built = build_html(
            value,
            style.color,
            style.size,
            style.font_style,
            style.align,
        );
        self.set_string_slot(slot_key, &built);
    }

    /// `SetHtmlText(slotKey, value, color, size)`.
    pub fn set_html_text(&mut self, slot_key: &str, value: &str, color: &str, size: i32) {
        self.set_html_text_styled(
            slot_key,
            value,
            HtmlStyle {
                color,
                size,
                ..HtmlStyle::NONE
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_builder_numbers_strings_from_zero() {
        let mut b = GumpBuilder::new_dynamic();
        b.add_page(0);
        b.add_label(10, 20, 0x480, "Hi");
        b.add_html(0, 0, 10, 10, "<b>x</b>", false, true);
        b.finalize_layout();
        assert_eq!(
            String::from_utf8_lossy(b.layout_data()),
            "{ page 0 }{ text 10 20 1152 0 }{ htmlgump 0 0 10 10 1 0 1 }\0"
        );
        assert_eq!(b.strings_count, 2);
        assert_eq!(&b.strings_data()[..6], &[0, 2, 0, b'H', 0, b'i']);
    }

    #[test]
    fn static_slots_follow_the_static_strings() {
        let mut b = GumpBuilder::new_static();
        b.add_label(0, 0, 0, "static");
        b.add_html_placeholder(1, 2, 3, 4, "content", false, true);
        b.add_label_placeholder(5, 6, 7, "unfilled");
        b.finalize_layout();
        let mut s = GumpStrings::new();
        s.set_string_slot("other", "x");
        s.set_html_text("content", "hello", "#FFC000", 4);
        // static strings: [empty, "static"] -> the slots start at 2: "other" is 2, "content" is 3
        b.finalize_strings(&s);
        assert_eq!(
            String::from_utf8_lossy(b.layout_data()),
            "{ text 0 0 0 1 }{ htmlgump 1 2 3 4 00003 0 1 }{ text 5 6 7       }\0"
        );
        assert_eq!(b.strings_count, 2);
        assert_eq!(s.count, 2);
        assert!(b.has_slots());
    }

    #[test]
    fn styled_html_is_wrapped() {
        let mut b = GumpBuilder::new_dynamic();
        b.add_html_styled(
            0,
            0,
            1,
            1,
            "t",
            HtmlStyle {
                align: TextAlignment::Center,
                ..HtmlStyle::NONE
            },
            false,
            false,
        );
        let data = b.strings_data().to_vec();
        let units: Vec<u16> = data[2..]
            .chunks(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        assert_eq!(String::from_utf16(&units).unwrap(), "<CENTER>t</CENTER>");
    }
}
