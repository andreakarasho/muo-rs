//! `GumpLayoutBuilder`, `GumpFlags` and `GumpButtonType`: the text a gump's layout packet is made of
//! (`{ resizepic 0 0 5054 400 300 }`).
//!
//! The builder owns a `Vec<u8>`.

use muo_primitives::Serial;

bitflags::bitflags! {
    /// `GumpFlags`.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
    pub struct GumpFlags: i32 {
        const NO_DISPOSE = 0x1;
        const NO_RESIZE = 0x2;
        const NO_MOVE = 0x4;
        const NO_CLOSE = 0x8;
    }
}

/// `GumpButtonType`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum GumpButtonType {
    Page = 0,
    #[default]
    Reply = 1,
}

/// `GumpLayoutBuilder`.
#[derive(Default)]
pub struct LayoutBuilder {
    buf: Vec<u8>,
    pub flags: GumpFlags,
    pub switches: i32,
    pub text_entries: i32,
    pub has_visual_elements: bool,
}

/// `GetBytesLatin1`: a character past 0xFF is `?`.
fn latin1(s: &str, out: &mut Vec<u8>) {
    out.extend(
        s.chars()
            .map(|c| if (c as u32) < 0x100 { c as u8 } else { b'?' }),
    );
}

impl LayoutBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// `LayoutData`.
    pub fn data(&self) -> &[u8] {
        &self.buf
    }

    /// The offset the next byte goes to.
    pub fn position(&self) -> usize {
        self.buf.len()
    }

    fn write(&mut self, s: &str) {
        self.buf.extend_from_slice(s.as_bytes());
    }

    fn start(&mut self, name: &str) {
        self.write("{ ");
        self.write(name);
        self.write(" ");
    }

    fn end(&mut self) {
        self.write("}");
    }

    fn int(&mut self, v: i64) {
        self.write(&v.to_string());
        self.write(" ");
    }

    fn ints(&mut self, vs: &[i32]) {
        for v in vs {
            self.int(i64::from(*v));
        }
    }

    fn boolean(&mut self, v: bool) {
        self.write(if v { "1 " } else { "0 " });
    }

    fn text(&mut self, s: &str) {
        latin1(s, &mut self.buf);
    }

    pub fn set_no_close(&mut self) {
        if !self.flags.contains(GumpFlags::NO_CLOSE) {
            self.write("{ noclose }");
            self.flags |= GumpFlags::NO_CLOSE;
        }
    }

    pub fn set_no_move(&mut self) {
        if !self.flags.contains(GumpFlags::NO_MOVE) {
            self.write("{ nomove }");
            self.flags |= GumpFlags::NO_MOVE;
        }
    }

    pub fn set_no_resize(&mut self) {
        if !self.flags.contains(GumpFlags::NO_RESIZE) {
            self.write("{ noresize }");
            self.flags |= GumpFlags::NO_RESIZE;
        }
    }

    pub fn set_no_dispose(&mut self) {
        if !self.flags.contains(GumpFlags::NO_DISPOSE) {
            self.write("{ nodispose }");
            self.flags |= GumpFlags::NO_DISPOSE;
        }
    }

    pub fn add_alpha_region(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.start("checkertrans");
        self.ints(&[x, y, width, height]);
        self.end();
    }

    pub fn add_background(&mut self, x: i32, y: i32, width: i32, height: i32, gump_id: i32) {
        self.has_visual_elements = true;
        self.start("resizepic");
        self.ints(&[x, y, gump_id, width, height]);
        self.end();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_button(
        &mut self,
        x: i32,
        y: i32,
        normal_id: i32,
        pressed_id: i32,
        button_id: i32,
        kind: GumpButtonType,
        param: i32,
    ) {
        self.has_visual_elements = true;
        self.start("button");
        self.ints(&[x, y, normal_id, pressed_id]);
        self.boolean(kind == GumpButtonType::Reply);
        self.ints(&[param, button_id]);
        self.end();
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
        self.has_visual_elements = true;
        self.start("checkbox");
        self.ints(&[x, y, inactive_id, active_id]);
        self.boolean(selected);
        self.ints(&[switch_id]);
        self.end();
        self.switches += 1;
    }

    pub fn add_group(&mut self, group_id: i32) {
        if group_id == 1 {
            self.write("{ group 1 }");
            return;
        }
        self.start("group");
        self.ints(&[group_id]);
        self.end();
    }

    /// `AddHtmlPlaceholder`: the offset of the six blanks the string index is written into.
    pub fn add_html_placeholder(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        background: bool,
        scrollbar: bool,
    ) -> usize {
        self.has_visual_elements = true;
        self.start("htmlgump");
        self.ints(&[x, y, width, height]);
        let position = self.position();
        self.write("      ");
        self.boolean(background);
        self.boolean(scrollbar);
        self.end();
        position
    }

    #[allow(clippy::too_many_arguments)] // one argument per layout parameter
    pub fn add_html(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        text: i32,
        background: bool,
        scrollbar: bool,
    ) {
        self.has_visual_elements = true;
        self.start("htmlgump");
        self.ints(&[x, y, width, height, text]);
        self.boolean(background);
        self.boolean(scrollbar);
        self.end();
    }

    #[allow(clippy::too_many_arguments)] // one argument per layout parameter
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
        self.has_visual_elements = true;
        self.start("xmfhtmlgump");
        self.ints(&[x, y, width, height, number]);
        self.boolean(background);
        self.boolean(scrollbar);
        self.end();
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
        self.has_visual_elements = true;
        self.start("xmfhtmlgumpcolor");
        self.ints(&[x, y, width, height, number]);
        self.boolean(background);
        self.boolean(scrollbar);
        self.ints(&[color]);
        self.end();
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
        self.has_visual_elements = true;
        self.start("xmfhtmltok");
        self.ints(&[x, y, width, height]);
        self.boolean(background);
        self.boolean(scrollbar);
        self.ints(&[color, number]);
        if !args.is_empty() {
            self.write("@");
            self.text(args);
            self.write("@ ");
        }
        self.end();
    }

    /// `AddImage(x, y, gumpId, hue, cls)`.
    pub fn add_image(&mut self, x: i32, y: i32, gump_id: i32, hue: i32, cls: &str) {
        self.has_visual_elements = true;
        self.start("gumppic");
        self.ints(&[x, y, gump_id]);
        if hue != 0 {
            self.write("hue=");
            self.ints(&[hue]);
        }
        if !cls.is_empty() {
            self.write("class=");
            self.text(cls);
            self.write(" ");
        }
        self.end();
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
        self.has_visual_elements = true;
        self.start("buttontileart");
        self.ints(&[x, y, normal_id, pressed_id]);
        self.boolean(kind == GumpButtonType::Reply);
        self.ints(&[param, button_id, item_id, hue, width, height]);
        self.end();
        if localized_tooltip > 0 {
            self.add_tooltip(localized_tooltip);
        }
    }

    pub fn add_image_tiled(&mut self, x: i32, y: i32, width: i32, height: i32, gump_id: i32) {
        self.has_visual_elements = true;
        self.start("gumppictiled");
        self.ints(&[x, y, width, height, gump_id]);
        self.end();
    }

    pub fn add_item(&mut self, x: i32, y: i32, item_id: i32, hue: i32) {
        self.has_visual_elements = true;
        self.start(if hue == 0 { "tilepic" } else { "tilepichue" });
        self.ints(&[x, y, item_id]);
        if hue != 0 {
            self.ints(&[hue]);
        }
        self.end();
    }

    pub fn add_item_property(&mut self, serial: Serial) {
        self.start("itemproperty");
        self.int(i64::from(serial.value()));
        self.end();
    }

    pub fn add_label_placeholder(&mut self, x: i32, y: i32, hue: i32) -> usize {
        self.has_visual_elements = true;
        self.start("text");
        self.ints(&[x, y, hue]);
        let position = self.position();
        self.write("      ");
        self.end();
        position
    }

    pub fn add_label(&mut self, x: i32, y: i32, hue: i32, text: i32) {
        self.has_visual_elements = true;
        self.start("text");
        self.ints(&[x, y, hue, text]);
        self.end();
    }

    pub fn add_label_cropped_placeholder(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
    ) -> usize {
        self.has_visual_elements = true;
        self.start("croppedtext");
        self.ints(&[x, y, width, height, hue]);
        let position = self.position();
        self.write("      ");
        self.end();
        position
    }

    pub fn add_label_cropped(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
        text: i32,
    ) {
        self.has_visual_elements = true;
        self.start("croppedtext");
        self.ints(&[x, y, width, height, hue, text]);
        self.end();
    }

    pub fn add_gump_id_override(&mut self, gump_id: i32) {
        self.start("mastergump");
        self.ints(&[gump_id]);
        self.end();
    }

    pub fn add_page(&mut self, page: i32) {
        if page == 0 {
            self.write("{ page 0 }");
            return;
        }
        self.start("page");
        self.ints(&[page]);
        self.end();
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
        self.has_visual_elements = true;
        self.start("radio");
        self.ints(&[x, y, inactive_id, active_id]);
        self.boolean(selected);
        self.ints(&[switch_id]);
        self.end();
        self.switches += 1;
    }

    #[allow(clippy::too_many_arguments)] // one argument per layout parameter
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
        self.has_visual_elements = true;
        self.start("picinpic");
        self.ints(&[x, y, gump_id, width, height, sx, sy]);
        self.end();
    }

    pub fn add_text_entry_placeholder(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
        entry_id: i32,
    ) -> usize {
        self.has_visual_elements = true;
        self.start("textentry");
        self.ints(&[x, y, width, height, hue, entry_id]);
        let position = self.position();
        self.write("      ");
        self.end();
        self.text_entries += 1;
        position
    }

    #[allow(clippy::too_many_arguments)] // one argument per layout parameter
    pub fn add_text_entry(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
        entry_id: i32,
        initial_text: i32,
    ) {
        self.has_visual_elements = true;
        self.start("textentry");
        self.ints(&[x, y, width, height, hue, entry_id, initial_text]);
        self.end();
        self.text_entries += 1;
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
        size: i32,
    ) -> usize {
        self.has_visual_elements = true;
        self.start("textentrylimited");
        self.ints(&[x, y, width, height, hue, entry_id]);
        let position = self.position();
        self.write("      ");
        self.ints(&[size]);
        self.end();
        self.text_entries += 1;
        position
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
        initial_text: i32,
        size: i32,
    ) {
        self.has_visual_elements = true;
        self.start("textentrylimited");
        self.ints(&[x, y, width, height, hue, entry_id, initial_text, size]);
        self.end();
        self.text_entries += 1;
    }

    pub fn add_tooltip(&mut self, number: i32) {
        self.start("tooltip");
        self.ints(&[number]);
        self.end();
    }

    pub fn add_tooltip_args(&mut self, number: i32, args: &str) {
        self.start("tooltip");
        self.ints(&[number]);
        if !args.is_empty() {
            self.write("@");
            self.text(args);
            self.write("@ ");
        }
        self.end();
    }

    /// `FinalizeLayout`: the NUL that ends the layout text.
    pub fn finalize(&mut self) {
        self.buf.push(0);
    }

    /// Writes a slot's string index over its six blanks as five zero-padded digits (`WriteSlotIndex`).
    pub fn write_slot_index(&mut self, offset: usize, index: usize) {
        let digits = format!("{index:05}");
        // an index past 99999 spills over the blank, as "00000" formatting does
        let n = digits.len().min(6);
        self.buf[offset..offset + n].copy_from_slice(&digits.as_bytes()[..n]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(b: &LayoutBuilder) -> String {
        String::from_utf8_lossy(b.data()).into_owned()
    }

    #[test]
    fn elements_have_the_client_syntax() {
        let mut b = LayoutBuilder::new();
        b.add_page(0);
        b.add_background(0, 0, 400, 300, 5054);
        b.add_alpha_region(10, 10, 380, 280);
        b.add_button(100, 265, 4005, 4007, 1, GumpButtonType::Reply, 0);
        b.add_group(1);
        b.add_group(2);
        b.add_page(3);
        b.add_item(1, 2, 3, 0);
        b.add_item(1, 2, 3, 9);
        b.add_image(5, 6, 7, 0, "");
        b.add_image(5, 6, 7, 8, "cls");
        b.add_tooltip_args(1010, "a\u{20ac}b");
        b.finalize();
        assert_eq!(
            text(&b),
            "{ page 0 }{ resizepic 0 0 5054 400 300 }{ checkertrans 10 10 380 280 }{ button 100 265 4005 4007 1 0 1 }{ group 1 }{ group 2 }{ page 3 }{ tilepic 1 2 3 }{ tilepichue 1 2 3 9 }{ gumppic 5 6 7 }{ gumppic 5 6 7 hue=8 class=cls }{ tooltip 1010 @a?b@ }\0"
        );
        assert!(b.has_visual_elements);
    }

    #[test]
    fn slot_index_overwrites_the_blanks() {
        let mut b = LayoutBuilder::new();
        let at = b.add_html_placeholder(1, 2, 3, 4, false, true);
        b.write_slot_index(at, 12);
        assert_eq!(text(&b), "{ htmlgump 1 2 3 4 00012 0 1 }");
    }

    #[test]
    fn switches_and_entries_are_counted() {
        let mut b = LayoutBuilder::new();
        b.add_checkbox(0, 0, 1, 2, true, 7);
        b.add_radio(0, 0, 1, 2, false, 8);
        b.add_text_entry(0, 0, 10, 10, 0, 1, 0);
        assert_eq!((b.switches, b.text_entries), (2, 1));
        assert_eq!(
            text(&b),
            "{ checkbox 0 0 1 2 1 7 }{ radio 0 0 1 2 0 8 }{ textentry 0 0 10 10 0 1 0 }"
        );
    }
}
