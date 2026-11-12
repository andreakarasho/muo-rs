//! `BaseGridGump`: the grid layout helpers of the older admin gumps. The class
//! state is [`BaseGrid`]; a subclass keeps one next to its own fields and its `LegacyLayout`.
use crate::gumps::GumpButtonType;
use crate::gumps::legacy::{GumpEntry, LegacyLayout};

pub const ARROW_LEFT_ID1: i32 = 0x15E3;
pub const ARROW_LEFT_ID2: i32 = 0x15E7;
pub const ARROW_LEFT_WIDTH: i32 = 16;
pub const ARROW_LEFT_HEIGHT: i32 = 16;
pub const ARROW_RIGHT_ID1: i32 = 0x15E1;
pub const ARROW_RIGHT_ID2: i32 = 0x15E5;
pub const ARROW_RIGHT_WIDTH: i32 = 16;
pub const ARROW_RIGHT_HEIGHT: i32 = 16;

/// The virtual properties.
#[derive(Clone, Copy, Debug)]
pub struct GridStyle {
    pub border_size: i32,
    pub offset_size: i32,
    pub entry_height: i32,
    pub offset_gump_id: i32,
    pub header_gump_id: i32,
    pub entry_gump_id: i32,
    pub back_gump_id: i32,
    pub text_hue: i32,
    pub text_offset_x: i32,
}

impl Default for GridStyle {
    fn default() -> Self {
        Self {
            border_size: 10,
            offset_size: 1,
            entry_height: 20,
            offset_gump_id: 0x0A40,
            header_gump_id: 0x0E14,
            entry_gump_id: 0x0BBC,
            back_gump_id: 0x13BE,
            text_hue: 0,
            text_offset_x: 2,
        }
    }
}

/// `BaseGridGump`'s fields. `background`/`offset` are the indexes of the page's two entries in the layout.
#[derive(Clone, Debug, Default)]
pub struct BaseGrid {
    pub style: GridStyle,
    pub current_page: i32,
    pub current_x: i32,
    pub current_y: i32,
    background: Option<usize>,
    offset: Option<usize>,
}

/// `GetButtonID`.
pub fn get_button_id(type_count: i32, ty: i32, index: i32) -> i32 {
    1 + index * type_count + ty
}

/// `SplitButtonID`: (type, index), or None for a button below 1.
pub fn split_button_id(button_id: i32, type_count: i32) -> Option<(i32, i32)> {
    if button_id < 1 {
        return None;
    }
    let b = button_id - 1;
    Some((b % type_count, b / type_count))
}

fn size_of(l: &LegacyLayout, i: Option<usize>) -> Option<(i32, i32)> {
    match i.map(|i| &l.entries[i]) {
        Some(
            GumpEntry::Background { width, height, .. }
            | GumpEntry::ImageTiled { width, height, .. },
        ) => Some((*width, *height)),
        _ => None,
    }
}

fn set_size(l: &mut LegacyLayout, i: Option<usize>, width: Option<i32>, height: Option<i32>) {
    if let Some(
        GumpEntry::Background {
            width: w,
            height: h,
            ..
        }
        | GumpEntry::ImageTiled {
            width: w,
            height: h,
            ..
        },
    ) = i.map(|i| &mut l.entries[i])
    {
        if let Some(v) = width {
            *w = v;
        }
        if let Some(v) = height {
            *h = v;
        }
    }
}

impl BaseGrid {
    pub fn new(style: GridStyle) -> Self {
        Self {
            style,
            ..Self::default()
        }
    }

    /// `FinishPage`.
    pub fn finish_page(&mut self, l: &mut LegacyLayout) {
        let s = self.style;
        set_size(
            l,
            self.background,
            None,
            Some(self.current_y + s.entry_height + s.offset_size + s.border_size),
        );
        set_size(
            l,
            self.offset,
            None,
            Some(self.current_y + s.entry_height + s.offset_size - s.border_size),
        );
    }

    /// `AddNewPage`.
    pub fn add_new_page(&mut self, l: &mut LegacyLayout) {
        self.finish_page(l);
        let s = self.style;
        self.current_x = s.border_size + s.offset_size;
        self.current_y = s.border_size + s.offset_size;
        self.current_page += 1;
        l.add_page(self.current_page);
        l.add_background(0, 0, 100, 100, s.back_gump_id);
        self.background = Some(l.entries.len() - 1);
        l.add_image_tiled(s.border_size, s.border_size, 100, 100, s.offset_gump_id);
        self.offset = Some(l.entries.len() - 1);
    }

    /// `AddNewLine`.
    pub fn add_new_line(&mut self) {
        self.current_y += self.style.entry_height + self.style.offset_size;
        self.current_x = self.style.border_size + self.style.offset_size;
    }

    /// `IncreaseX`.
    pub fn increase_x(&mut self, l: &mut LegacyLayout, width: i32) {
        let s = self.style;
        self.current_x += width + s.offset_size;
        let w = self.current_x + s.border_size;
        if size_of(l, self.background).is_some_and(|(bw, _)| w > bw) {
            set_size(l, self.background, Some(w), None);
        }
        let w = self.current_x - s.border_size;
        if size_of(l, self.offset).is_some_and(|(ow, _)| w > ow) {
            set_size(l, self.offset, Some(w), None);
        }
    }

    /// `AddEntryLabel`.
    pub fn add_entry_label(&mut self, l: &mut LegacyLayout, width: i32, text: &str) {
        let s = self.style;
        l.add_image_tiled(
            self.current_x,
            self.current_y,
            width,
            s.entry_height,
            s.entry_gump_id,
        );
        l.add_label_cropped(
            self.current_x + s.text_offset_x,
            self.current_y,
            width - s.text_offset_x,
            s.entry_height,
            s.text_hue,
            text,
        );
        self.increase_x(l, width);
    }

    /// `AddEntryHtml`.
    pub fn add_entry_html(&mut self, l: &mut LegacyLayout, width: i32, text: &str) {
        let s = self.style;
        l.add_image_tiled(
            self.current_x,
            self.current_y,
            width,
            s.entry_height,
            s.entry_gump_id,
        );
        l.add_html(
            self.current_x + s.text_offset_x,
            self.current_y,
            width - s.text_offset_x,
            s.entry_height,
            text,
            false,
            false,
        );
        self.increase_x(l, width);
    }

    /// `AddEntryHeader(width, spannedEntries)`.
    pub fn add_entry_header(&mut self, l: &mut LegacyLayout, width: i32, spanned: i32) {
        let s = self.style;
        l.add_image_tiled(
            self.current_x,
            self.current_y,
            width,
            s.entry_height * spanned + s.offset_size * (spanned - 1),
            s.header_gump_id,
        );
        self.increase_x(l, width);
    }

    /// `AddBlankLine`.
    pub fn add_blank_line(&mut self, l: &mut LegacyLayout) {
        if let Some(GumpEntry::ImageTiled { x, width, .. }) =
            self.offset.map(|i| l.entries[i].clone())
        {
            l.add_image_tiled(
                x,
                self.current_y,
                width,
                self.style.entry_height,
                self.style.back_gump_id + 4,
            );
        }
        self.add_new_line();
    }

    /// `AddEntryButton(width, normalID, pressedID, buttonID, buttonWidth, buttonHeight, spannedEntries)`.
    #[allow(clippy::too_many_arguments)] // one argument per entry option
    pub fn add_entry_button(
        &mut self,
        l: &mut LegacyLayout,
        width: i32,
        normal_id: i32,
        pressed_id: i32,
        button_id: i32,
        button_width: i32,
        button_height: i32,
        spanned: i32,
    ) {
        let s = self.style;
        let h = s.entry_height * spanned + s.offset_size * (spanned - 1);
        l.add_image_tiled(self.current_x, self.current_y, width, h, s.header_gump_id);
        l.add_button(
            self.current_x + (width - button_width) / 2,
            self.current_y + (h - button_height) / 2,
            normal_id,
            pressed_id,
            button_id,
        );
        self.increase_x(l, width);
    }

    /// `AddEntryPageButton`.
    #[allow(clippy::too_many_arguments)] // one argument per entry option
    pub fn add_entry_page_button(
        &mut self,
        l: &mut LegacyLayout,
        width: i32,
        normal_id: i32,
        pressed_id: i32,
        page: i32,
        button_width: i32,
        button_height: i32,
    ) {
        let s = self.style;
        l.add_image_tiled(
            self.current_x,
            self.current_y,
            width,
            s.entry_height,
            s.header_gump_id,
        );
        l.add_button_type(
            self.current_x + (width - button_width) / 2,
            self.current_y + (s.entry_height - button_height) / 2,
            normal_id,
            pressed_id,
            0,
            GumpButtonType::Page,
            page,
        );
        self.increase_x(l, width);
    }

    /// `AddEntryText`.
    pub fn add_entry_text(
        &mut self,
        l: &mut LegacyLayout,
        width: i32,
        entry_id: i32,
        initial_text: &str,
    ) {
        let s = self.style;
        l.add_image_tiled(
            self.current_x,
            self.current_y,
            width,
            s.entry_height,
            s.entry_gump_id,
        );
        l.add_text_entry(
            self.current_x + s.text_offset_x,
            self.current_y,
            width - s.text_offset_x,
            s.entry_height,
            s.text_hue,
            entry_id,
            initial_text,
        );
        self.increase_x(l, width);
    }
}

/// `sender.Mobile`.
pub fn mobile_of(
    w: &muo_server::World,
    ns: muo_server::net_state::NetStateId,
) -> Option<muo_server::ids::MobileId> {
    w.net_states.get(&ns).and_then(|n| n.mobile)
}

/// `to.SendGump(gump)`.
pub fn send(
    w: &mut muo_server::World,
    to: muo_server::ids::MobileId,
    gump: impl crate::gumps::Gump + 'static,
) {
    crate::gumps::send_gump_to_mobile(w, to, Box::new(gump), false);
}
