//! `PropsLayout`, `PropsLayoutExtensions` (for the
//! legacy `Gump`) and `PropsLayoutBuilderExtensions` (for the dynamic builder).
//!
//! The two extension sets are one set of functions here over [`PropsSurface`]; the surface decides the few places where
//! the legacy and builder versions differ (how a title is drawn).

use crate::gumps::legacy::LegacyLayout;
use crate::gumps::{DynamicGumpBuilder, GumpButtonType, HtmlStyle};
use muo_primitives::html::TextAlignment;

use super::props_config::*;

/// `PropsLayout.NextRow`.
pub fn next_row(x: &mut i32, y: &mut i32) {
    next_row_h(x, y, ENTRY_HEIGHT);
}

pub fn next_row_h(x: &mut i32, y: &mut i32, entry_height: i32) {
    *x = BORDER_SIZE + OFFSET_SIZE;
    *y += entry_height + OFFSET_SIZE;
}

/// What a props layout needs of a gump: the legacy `Gump` or the `DynamicGumpBuilder`.
pub trait PropsSurface {
    fn background(&mut self, x: i32, y: i32, w: i32, h: i32, id: i32);
    fn image_tiled(&mut self, x: i32, y: i32, w: i32, h: i32, id: i32);
    fn label_cropped(&mut self, x: i32, y: i32, w: i32, h: i32, hue: i32, text: &str);
    #[allow(clippy::too_many_arguments)]
    fn button(
        &mut self,
        x: i32,
        y: i32,
        normal: i32,
        pressed: i32,
        id: i32,
        kind: GumpButtonType,
        param: i32,
    );
    #[allow(clippy::too_many_arguments)]
    fn text_entry(&mut self, x: i32, y: i32, w: i32, h: i32, hue: i32, id: i32, text: &str);
    /// The title of `AddPropsHeader`, given the cell's raw origin and size.
    fn header_title(&mut self, x: i32, y: i32, w: i32, h: i32, title: Option<&str>);
    /// The title of `AddPropsHeaderWithBack`.
    fn back_title(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        title: Option<&str>,
        entry_height: i32,
    );
}

impl PropsSurface for LegacyLayout {
    fn background(&mut self, x: i32, y: i32, w: i32, h: i32, id: i32) {
        self.add_background(x, y, w, h, id);
    }
    fn image_tiled(&mut self, x: i32, y: i32, w: i32, h: i32, id: i32) {
        LegacyLayout::add_image_tiled(self, x, y, w, h, id);
    }
    fn label_cropped(&mut self, x: i32, y: i32, w: i32, h: i32, hue: i32, text: &str) {
        LegacyLayout::add_label_cropped(self, x, y, w, h, hue, text);
    }
    fn button(
        &mut self,
        x: i32,
        y: i32,
        normal: i32,
        pressed: i32,
        id: i32,
        kind: GumpButtonType,
        param: i32,
    ) {
        self.add_button_type(x, y, normal, pressed, id, kind, param);
    }
    fn text_entry(&mut self, x: i32, y: i32, w: i32, h: i32, hue: i32, id: i32, text: &str) {
        LegacyLayout::add_text_entry(self, x, y, w, h, hue, id, text);
    }
    fn header_title(&mut self, x: i32, y: i32, w: i32, h: i32, title: Option<&str>) {
        if let Some(t) = title {
            self.add_html(x, y, w, h, t, false, false);
        }
    }
    fn back_title(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        title: Option<&str>,
        entry_height: i32,
    ) {
        if let Some(t) = title {
            self.add_html(
                x + TEXT_OFFSET_X,
                y + (entry_height - ENTRY_HEIGHT) / 2,
                w - TEXT_OFFSET_X,
                h,
                &format!("<center>{t}</center>"),
                false,
                false,
            );
        }
    }
}

impl PropsSurface for DynamicGumpBuilder {
    fn background(&mut self, x: i32, y: i32, w: i32, h: i32, id: i32) {
        self.add_background(x, y, w, h, id);
    }
    fn image_tiled(&mut self, x: i32, y: i32, w: i32, h: i32, id: i32) {
        DynamicGumpBuilder::add_image_tiled(self, x, y, w, h, id);
    }
    fn label_cropped(&mut self, x: i32, y: i32, w: i32, h: i32, hue: i32, text: &str) {
        DynamicGumpBuilder::add_label_cropped(self, x, y, w, h, hue, text);
    }
    fn button(
        &mut self,
        x: i32,
        y: i32,
        normal: i32,
        pressed: i32,
        id: i32,
        kind: GumpButtonType,
        param: i32,
    ) {
        self.add_button_type(x, y, normal, pressed, id, kind, param);
    }
    fn text_entry(&mut self, x: i32, y: i32, w: i32, h: i32, hue: i32, id: i32, text: &str) {
        DynamicGumpBuilder::add_text_entry(self, x, y, w, h, hue, id, text);
    }
    fn header_title(&mut self, x: i32, y: i32, w: i32, h: i32, title: Option<&str>) {
        if let Some(t) = title.filter(|t| !t.is_empty()) {
            let style = HtmlStyle {
                align: TextAlignment::Center,
                ..HtmlStyle::NONE
            };
            self.add_html_styled(
                x + TEXT_OFFSET_X,
                y,
                w - TEXT_OFFSET_X,
                h,
                t,
                style,
                false,
                false,
            );
        }
    }
    fn back_title(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        title: Option<&str>,
        entry_height: i32,
    ) {
        if let Some(t) = title.filter(|t| !t.is_empty()) {
            let style = HtmlStyle {
                align: TextAlignment::Center,
                ..HtmlStyle::NONE
            };
            self.add_html_styled(
                x + TEXT_OFFSET_X,
                y + (entry_height - ENTRY_HEIGHT) / 2,
                w - TEXT_OFFSET_X,
                h,
                t,
                style,
                false,
                false,
            );
        }
    }
}

/// `AddPropsFrame`: the outer background and the inner offset region; returns the origin of the first row.
pub fn add_props_frame<S: PropsSurface>(g: &mut S, total_width: i32, row_count: i32) -> (i32, i32) {
    add_props_frame_h(g, total_width, row_count, ENTRY_HEIGHT)
}

pub fn add_props_frame_h<S: PropsSurface>(
    g: &mut S,
    total_width: i32,
    row_count: i32,
    entry_height: i32,
) -> (i32, i32) {
    let total_height = OFFSET_SIZE + (entry_height + OFFSET_SIZE) * row_count;
    g.background(
        0,
        0,
        BORDER_SIZE + total_width + BORDER_SIZE,
        BORDER_SIZE + total_height + BORDER_SIZE,
        BACK_GUMP_ID,
    );
    g.image_tiled(
        BORDER_SIZE,
        BORDER_SIZE,
        total_width,
        total_height,
        OFFSET_GUMP_ID,
    );
    (BORDER_SIZE + OFFSET_SIZE, BORDER_SIZE + OFFSET_SIZE)
}

/// A paging button's type and parameter.
#[derive(Copy, Clone)]
pub struct Nav {
    pub kind: GumpButtonType,
    pub param: i32,
}

impl Nav {
    pub const REPLY: Nav = Nav {
        kind: GumpButtonType::Reply,
        param: 0,
    };
    pub fn page(param: i32) -> Nav {
        Nav {
            kind: GumpButtonType::Page,
            param,
        }
    }
}

/// `AddPropsHeader`: `[Prev] [Title] [Next]`, each with its button id when shown.
pub struct Header<'a> {
    pub title: Option<&'a str>,
    pub prev: Option<i32>,
    pub next: Option<i32>,
    pub prev_nav: Nav,
    pub next_nav: Nav,
}

impl<'a> Header<'a> {
    pub fn new(title: Option<&'a str>, prev: Option<i32>, next: Option<i32>) -> Self {
        Self {
            title,
            prev,
            next,
            prev_nav: Nav::REPLY,
            next_nav: Nav::REPLY,
        }
    }
}

pub fn add_props_header<S: PropsSurface>(
    g: &mut S,
    total_width: i32,
    x: &mut i32,
    y: &mut i32,
    h: &Header<'_>,
) {
    add_props_header_h(g, total_width, x, y, h, ENTRY_HEIGHT);
}

pub fn add_props_header_h<S: PropsSurface>(
    g: &mut S,
    total_width: i32,
    x: &mut i32,
    y: &mut i32,
    h: &Header<'_>,
    entry_height: i32,
) {
    let empty_width = total_width - PREV_WIDTH - NEXT_WIDTH - OFFSET_SIZE * 4;

    g.image_tiled(*x, *y, PREV_WIDTH, entry_height, HEADER_GUMP_ID);
    if let Some(id) = h.prev {
        g.button(
            *x + PREV_OFFSET_X,
            *y + PREV_OFFSET_Y,
            PREV_BUTTON_ID1,
            PREV_BUTTON_ID2,
            id,
            h.prev_nav.kind,
            h.prev_nav.param,
        );
    }

    *x += PREV_WIDTH + OFFSET_SIZE;

    g.image_tiled(*x, *y, empty_width, entry_height, HEADER_GUMP_ID);
    g.header_title(*x, *y, empty_width, entry_height, h.title);

    *x += empty_width + OFFSET_SIZE;

    g.image_tiled(*x, *y, NEXT_WIDTH, entry_height, HEADER_GUMP_ID);
    if let Some(id) = h.next {
        g.button(
            *x + NEXT_OFFSET_X,
            *y + NEXT_OFFSET_Y,
            NEXT_BUTTON_ID1,
            NEXT_BUTTON_ID2,
            id,
            h.next_nav.kind,
            h.next_nav.param,
        );
    }
}

/// `AddPropsHeaderWithBack`: `[Back] [Title] [Prev] [Next]`.
#[allow(clippy::too_many_arguments)]
pub fn add_props_header_with_back<S: PropsSurface>(
    g: &mut S,
    total_width: i32,
    x: &mut i32,
    y: &mut i32,
    title: Option<&str>,
    back: Option<i32>,
    prev: Option<i32>,
    next: Option<i32>,
    next_nav: Nav,
    entry_height: i32,
) {
    let empty_width = total_width - PREV_WIDTH * 2 - NEXT_WIDTH - OFFSET_SIZE * 5;

    g.image_tiled(*x, *y, PREV_WIDTH, entry_height, HEADER_GUMP_ID);
    if let Some(id) = back {
        g.button(
            *x + PREV_OFFSET_X,
            *y + PREV_OFFSET_Y,
            PREV_BUTTON_ID1,
            PREV_BUTTON_ID2,
            id,
            GumpButtonType::Reply,
            0,
        );
    }

    *x += PREV_WIDTH + OFFSET_SIZE;

    g.image_tiled(*x, *y, empty_width, entry_height, ENTRY_GUMP_ID);
    g.back_title(*x, *y, empty_width, entry_height, title, entry_height);

    *x += empty_width + OFFSET_SIZE;

    g.image_tiled(*x, *y, PREV_WIDTH, entry_height, HEADER_GUMP_ID);
    if let Some(id) = prev {
        g.button(
            *x + PREV_OFFSET_X,
            *y + PREV_OFFSET_Y,
            PREV_BUTTON_ID1,
            PREV_BUTTON_ID2,
            id,
            GumpButtonType::Reply,
            0,
        );
    }

    *x += PREV_WIDTH + OFFSET_SIZE;

    g.image_tiled(*x, *y, NEXT_WIDTH, entry_height, HEADER_GUMP_ID);
    if let Some(id) = next {
        g.button(
            *x + NEXT_OFFSET_X,
            *y + NEXT_OFFSET_Y,
            NEXT_BUTTON_ID1,
            NEXT_BUTTON_ID2,
            id,
            next_nav.kind,
            next_nav.param,
        );
    }
}

/// The set cell every entry row ends with.
fn set_cell<S: PropsSurface>(g: &mut S, x: i32, y: i32, entry_height: i32, button: Option<i32>) {
    if SET_GUMP_ID != 0 {
        g.image_tiled(x, y, SET_WIDTH, entry_height, SET_GUMP_ID);
    }
    if let Some(id) = button {
        g.button(
            x + SET_OFFSET_X,
            y + SET_OFFSET_Y,
            SET_BUTTON_ID1,
            SET_BUTTON_ID2,
            id,
            GumpButtonType::Reply,
            0,
        );
    }
}

/// `AddPropsEntryButton`: `[Label] [Button]`; `button` is the button id when there is one.
pub fn add_props_entry_button<S: PropsSurface>(
    g: &mut S,
    x: &mut i32,
    y: &mut i32,
    entry_width: i32,
    label: &str,
    button: Option<i32>,
    text_hue: i32,
) {
    g.image_tiled(*x, *y, entry_width, ENTRY_HEIGHT, ENTRY_GUMP_ID);
    g.label_cropped(
        *x + TEXT_OFFSET_X,
        *y,
        entry_width - TEXT_OFFSET_X,
        ENTRY_HEIGHT,
        text_hue,
        label,
    );
    *x += entry_width + OFFSET_SIZE;
    set_cell(g, *x, *y, ENTRY_HEIGHT, button);
}

/// `AddPropsEntryNameValue`: `[Name] [Value] [Button]`.
#[allow(clippy::too_many_arguments)]
pub fn add_props_entry_name_value<S: PropsSurface>(
    g: &mut S,
    x: &mut i32,
    y: &mut i32,
    name_width: i32,
    value_width: i32,
    name: &str,
    value: &str,
    button: Option<i32>,
) {
    g.image_tiled(*x, *y, name_width, ENTRY_HEIGHT, ENTRY_GUMP_ID);
    g.label_cropped(
        *x + TEXT_OFFSET_X,
        *y,
        name_width - TEXT_OFFSET_X,
        ENTRY_HEIGHT,
        TEXT_HUE,
        name,
    );
    *x += name_width + OFFSET_SIZE;

    g.image_tiled(*x, *y, value_width, ENTRY_HEIGHT, ENTRY_GUMP_ID);
    g.label_cropped(
        *x + TEXT_OFFSET_X,
        *y,
        value_width - TEXT_OFFSET_X,
        ENTRY_HEIGHT,
        TEXT_HUE,
        value,
    );
    *x += value_width + OFFSET_SIZE;

    set_cell(g, *x, *y, ENTRY_HEIGHT, button);
}

/// `AddPropsEntryTextInput`: `[TextEntry] [Button]`.
#[allow(clippy::too_many_arguments)]
pub fn add_props_entry_text_input<S: PropsSurface>(
    g: &mut S,
    x: &mut i32,
    y: &mut i32,
    entry_width: i32,
    entry_id: i32,
    initial_text: &str,
    button: Option<i32>,
) {
    g.image_tiled(*x, *y, entry_width, ENTRY_HEIGHT, ENTRY_GUMP_ID);
    g.text_entry(
        *x + TEXT_OFFSET_X,
        *y,
        entry_width - TEXT_OFFSET_X,
        ENTRY_HEIGHT,
        TEXT_HUE,
        entry_id,
        initial_text,
    );
    *x += entry_width + OFFSET_SIZE;
    set_cell(g, *x, *y, ENTRY_HEIGHT, button);
}

/// `AddPropsEntryLabel`: `[Label] [Empty]`.
pub fn add_props_entry_label<S: PropsSurface>(
    g: &mut S,
    x: &mut i32,
    y: &mut i32,
    entry_width: i32,
    label: &str,
) {
    g.image_tiled(*x, *y, entry_width, ENTRY_HEIGHT, ENTRY_GUMP_ID);
    g.label_cropped(
        *x + TEXT_OFFSET_X,
        *y,
        entry_width - TEXT_OFFSET_X,
        ENTRY_HEIGHT,
        TEXT_HUE,
        label,
    );
    *x += entry_width + OFFSET_SIZE;
    set_cell(g, *x, *y, ENTRY_HEIGHT, None);
}

/// `AddPropsEntryType`: the type group header, drawn as a label row.
pub fn add_props_entry_type<S: PropsSurface>(
    g: &mut S,
    x: &mut i32,
    y: &mut i32,
    type_width: i32,
    type_name: &str,
) {
    add_props_entry_label(g, x, y, type_width, type_name);
}

/// `AddPropsEntryBlank`.
pub fn add_props_entry_blank<S: PropsSurface>(g: &mut S, x: i32, y: i32, total_width: i32) {
    g.image_tiled(
        x - OFFSET_SIZE,
        y,
        total_width,
        ENTRY_HEIGHT,
        BACK_GUMP_ID + 4,
    );
}
