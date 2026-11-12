//! Legacy gumps: the entry-list `Gump` the older gumps are written with.
//!
//! A legacy gump keeps its entries and compiles them on every send. Identical strings share one table slot
//! (`OrderedSet<string>`), unlike the new builders. The class state (what the subclass kept besides entries) is
//! its [`super::base::LegacyGumpLogic`]; `Gump.Add`/`Remove` and the entries' `Parent` link are just the `Vec`.

use indexmap::IndexSet;
use muo_primitives::Serial;
use muo_primitives::html::{TextAlignment, build as build_html};

use super::layout::GumpButtonType;

/// `GumpHtmlLocalizedType`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocalizedKind {
    Plain,
    Color(i32),
    Args { args: String, color: i32 },
}

/// A `GumpEntry`.
#[derive(Clone, Debug, PartialEq)]
pub enum GumpEntry {
    Page(i32),
    AlphaRegion {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    Background {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        gump_id: i32,
    },
    Button {
        x: i32,
        y: i32,
        normal_id: i32,
        pressed_id: i32,
        button_id: i32,
        kind: GumpButtonType,
        param: i32,
    },
    Check {
        x: i32,
        y: i32,
        inactive_id: i32,
        active_id: i32,
        initial_state: bool,
        switch_id: i32,
    },
    EcHandleInput,
    Group(i32),
    Html {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        text: String,
        background: bool,
        scrollbar: bool,
    },
    HtmlLocalized {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        number: i32,
        background: bool,
        scrollbar: bool,
        kind: LocalizedKind,
    },
    Image {
        x: i32,
        y: i32,
        gump_id: i32,
        hue: i32,
        class: Option<String>,
    },
    ImageTileButton {
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
    },
    ImageTiled {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        gump_id: i32,
    },
    Item {
        x: i32,
        y: i32,
        item_id: i32,
        hue: i32,
    },
    ItemProperty(Serial),
    Label {
        x: i32,
        y: i32,
        hue: i32,
        text: String,
    },
    LabelCropped {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
        text: String,
    },
    MasterGump(i32),
    Radio {
        x: i32,
        y: i32,
        inactive_id: i32,
        active_id: i32,
        initial_state: bool,
        switch_id: i32,
    },
    SpriteImage {
        x: i32,
        y: i32,
        gump_id: i32,
        width: i32,
        height: i32,
        sx: i32,
        sy: i32,
    },
    TextEntry {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
        entry_id: i32,
        initial_text: String,
    },
    TextEntryLimited {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        hue: i32,
        entry_id: i32,
        initial_text: String,
        size: i32,
    },
    Tooltip {
        number: i32,
        args: Option<String>,
    },
}

/// What `Gump.Compile` produced besides the packet.
pub struct Compiled {
    pub layout: Vec<u8>,
    pub strings: Vec<u8>,
    pub strings_count: usize,
    pub switches: i32,
    pub text_entries: i32,
    pub has_visual_elements: bool,
}

/// `Gump`'s fields: the entries and the flags.
#[derive(Clone, Debug)]
pub struct LegacyLayout {
    pub entries: Vec<GumpEntry>,
    pub disposable: bool,
    pub resizable: bool,
    pub draggable: bool,
    pub closable: bool,
}

impl Default for LegacyLayout {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            disposable: true,
            resizable: true,
            draggable: true,
            closable: true,
        }
    }
}

fn b(v: bool) -> &'static str {
    if v { "1" } else { "0" }
}

/// `WriteAscii`: a non-ASCII character is `?`; `WriteLatin1`: past 0xFF is `?`.
fn ascii(s: &str, out: &mut Vec<u8>) {
    out.extend(s.chars().map(|c| if c.is_ascii() { c as u8 } else { b'?' }));
}

fn latin1(s: &str, out: &mut Vec<u8>) {
    out.extend(
        s.chars()
            .map(|c| if (c as u32) < 0x100 { c as u8 } else { b'?' }),
    );
}

impl LegacyLayout {
    pub fn add(&mut self, e: GumpEntry) {
        self.entries.push(e);
    }

    /// `Gump.Remove`.
    pub fn remove(&mut self, e: &GumpEntry) -> bool {
        match self.entries.iter().position(|x| x == e) {
            Some(i) => {
                self.entries.remove(i);
                true
            }
            None => false,
        }
    }

    pub fn reset(&mut self) {
        self.entries.clear();
    }

    pub fn add_page(&mut self, page: i32) {
        self.add(GumpEntry::Page(page));
    }

    pub fn add_alpha_region(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.add(GumpEntry::AlphaRegion {
            x,
            y,
            width,
            height,
        });
    }

    pub fn add_background(&mut self, x: i32, y: i32, width: i32, height: i32, gump_id: i32) {
        self.add(GumpEntry::Background {
            x,
            y,
            width,
            height,
            gump_id,
        });
    }

    /// `AddButton(x, y, normalID, pressedID, buttonID)`: a reply button.
    pub fn add_button(&mut self, x: i32, y: i32, normal_id: i32, pressed_id: i32, button_id: i32) {
        self.add_button_type(
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
        self.add(GumpEntry::Button {
            x,
            y,
            normal_id,
            pressed_id,
            button_id,
            kind,
            param,
        });
    }

    /// `AddCheck`.
    pub fn add_check(
        &mut self,
        x: i32,
        y: i32,
        inactive_id: i32,
        active_id: i32,
        initial_state: bool,
        switch_id: i32,
    ) {
        self.add(GumpEntry::Check {
            x,
            y,
            inactive_id,
            active_id,
            initial_state,
            switch_id,
        });
    }

    pub fn add_group(&mut self, group: i32) {
        self.add(GumpEntry::Group(group));
    }

    pub fn add_tooltip(&mut self, number: i32, args: Option<&str>) {
        self.add(GumpEntry::Tooltip {
            number,
            args: args.map(str::to_owned),
        });
    }

    #[allow(clippy::too_many_arguments)] // one parameter per gump entry field
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
        self.add(GumpEntry::Html {
            x,
            y,
            width,
            height,
            text: text.to_owned(),
            background,
            scrollbar,
        });
    }

    #[allow(clippy::too_many_arguments)] // one parameter per gump entry field
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
        self.add(GumpEntry::HtmlLocalized {
            x,
            y,
            width,
            height,
            number,
            background,
            scrollbar,
            kind: LocalizedKind::Plain,
        });
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
        self.add(GumpEntry::HtmlLocalized {
            x,
            y,
            width,
            height,
            number,
            background,
            scrollbar,
            kind: LocalizedKind::Color(color),
        });
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
        self.add(GumpEntry::HtmlLocalized {
            x,
            y,
            width,
            height,
            number,
            background,
            scrollbar,
            kind: LocalizedKind::Args {
                args: args.to_owned(),
                color,
            },
        });
    }

    pub fn add_image(&mut self, x: i32, y: i32, gump_id: i32, hue: i32) {
        self.add(GumpEntry::Image {
            x,
            y,
            gump_id,
            hue,
            class: None,
        });
    }

    pub fn add_image_tiled(&mut self, x: i32, y: i32, width: i32, height: i32, gump_id: i32) {
        self.add(GumpEntry::ImageTiled {
            x,
            y,
            width,
            height,
            gump_id,
        });
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
    ) {
        self.add(GumpEntry::ImageTileButton {
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
        });
    }

    pub fn add_item(&mut self, x: i32, y: i32, item_id: i32, hue: i32) {
        self.add(GumpEntry::Item { x, y, item_id, hue });
    }

    pub fn add_label(&mut self, x: i32, y: i32, hue: i32, text: &str) {
        self.add(GumpEntry::Label {
            x,
            y,
            hue,
            text: text.to_owned(),
        });
    }

    /// `AddLabelHtml`: a centered (or left) html block.
    #[allow(clippy::too_many_arguments)] // one parameter per gump entry field
    pub fn add_label_html(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        text: &str,
        hue: &str,
        size: i32,
        center: bool,
    ) {
        let align = if center {
            TextAlignment::Center
        } else {
            TextAlignment::Left
        };
        let built = build_html(text, hue, size, 0, align);
        self.add_html(x, y, width, height, &built, false, false);
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
        self.add(GumpEntry::LabelCropped {
            x,
            y,
            width,
            height,
            hue,
            text: text.to_owned(),
        });
    }

    pub fn add_radio(
        &mut self,
        x: i32,
        y: i32,
        inactive_id: i32,
        active_id: i32,
        initial_state: bool,
        switch_id: i32,
    ) {
        self.add(GumpEntry::Radio {
            x,
            y,
            inactive_id,
            active_id,
            initial_state,
            switch_id,
        });
    }

    #[allow(clippy::too_many_arguments)] // one parameter per gump entry field
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
        self.add(GumpEntry::TextEntry {
            x,
            y,
            width,
            height,
            hue,
            entry_id,
            initial_text: initial_text.to_owned(),
        });
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
        self.add(GumpEntry::TextEntryLimited {
            x,
            y,
            width,
            height,
            hue,
            entry_id,
            initial_text: initial_text.to_owned(),
            size,
        });
    }

    pub fn add_item_property(&mut self, serial: Serial) {
        self.add(GumpEntry::ItemProperty(serial));
    }

    #[allow(clippy::too_many_arguments)] // one parameter per gump entry field
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
        self.add(GumpEntry::SpriteImage {
            x,
            y,
            gump_id,
            width,
            height,
            sx,
            sy,
        });
    }

    pub fn add_ec_handle_input(&mut self) {
        self.add(GumpEntry::EcHandleInput);
    }

    pub fn add_gump_id_override(&mut self, gump_id: i32) {
        self.add(GumpEntry::MasterGump(gump_id));
    }

    /// Does the gump have a button (plain or tile) with this id? (`DisplayGumpResponse` checks the response.)
    pub fn has_button(&self, button_id: i32) -> bool {
        self.entries.iter().any(|e| match e {
            GumpEntry::Button { button_id: id, .. }
            | GumpEntry::ImageTileButton { button_id: id, .. } => *id == button_id,
            _ => false,
        })
    }

    /// `IsVisualEntry`.
    fn is_visual(e: &GumpEntry) -> bool {
        !matches!(
            e,
            GumpEntry::AlphaRegion { .. }
                | GumpEntry::EcHandleInput
                | GumpEntry::Group(_)
                | GumpEntry::ItemProperty(_)
                | GumpEntry::MasterGump(_)
                | GumpEntry::Page(_)
                | GumpEntry::Tooltip { .. }
        )
    }

    /// `Gump.Compile`'s layout text and strings table (without the packet around them).
    pub fn compile(&self) -> Compiled {
        let mut out = Vec::new();
        let mut strings: IndexSet<String> = IndexSet::new();
        let (mut text_entries, mut switches) = (0, 0);
        let mut has_visual = false;

        if !self.draggable {
            out.extend_from_slice(b"{ nomove }");
        }
        if !self.closable {
            out.extend_from_slice(b"{ noclose }");
        }
        if !self.disposable {
            out.extend_from_slice(b"{ nodispose }");
        }
        if !self.resizable {
            out.extend_from_slice(b"{ noresize }");
        }

        let idx = |s: &str, strings: &mut IndexSet<String>| strings.insert_full(s.to_owned()).0;

        for e in &self.entries {
            has_visual |= Self::is_visual(e);
            match e {
                GumpEntry::Page(0) => out.extend_from_slice(b"{ page 0 }"),
                GumpEntry::Page(p) => ascii(&format!("{{ page {p} }}"), &mut out),
                GumpEntry::AlphaRegion {
                    x,
                    y,
                    width,
                    height,
                } => {
                    ascii(
                        &format!("{{ checkertrans {x} {y} {width} {height} }}"),
                        &mut out,
                    );
                }
                GumpEntry::Background {
                    x,
                    y,
                    width,
                    height,
                    gump_id,
                } => {
                    ascii(
                        &format!("{{ resizepic {x} {y} {gump_id} {width} {height} }}"),
                        &mut out,
                    );
                }
                GumpEntry::Button {
                    x,
                    y,
                    normal_id,
                    pressed_id,
                    button_id,
                    kind,
                    param,
                } => ascii(
                    &format!(
                        "{{ button {x} {y} {normal_id} {pressed_id} {} {param} {button_id} }}",
                        *kind as i32
                    ),
                    &mut out,
                ),
                GumpEntry::Check {
                    x,
                    y,
                    inactive_id,
                    active_id,
                    initial_state,
                    switch_id,
                } => {
                    ascii(
                        &format!(
                            "{{ checkbox {x} {y} {inactive_id} {active_id} {} {switch_id} }}",
                            b(*initial_state)
                        ),
                        &mut out,
                    );
                    switches += 1;
                }
                GumpEntry::EcHandleInput => out.extend_from_slice(b"{ echandleinput }"),
                GumpEntry::Group(1) => out.extend_from_slice(b"{ group 1 }"),
                GumpEntry::Group(g) => ascii(&format!("{{ group {g} }}"), &mut out),
                GumpEntry::Html {
                    x,
                    y,
                    width,
                    height,
                    text,
                    background,
                    scrollbar,
                } => {
                    let i = idx(text, &mut strings);
                    ascii(
                        &format!(
                            "{{ htmlgump {x} {y} {width} {height} {i} {} {} }}",
                            b(*background),
                            b(*scrollbar)
                        ),
                        &mut out,
                    );
                }
                GumpEntry::HtmlLocalized {
                    x,
                    y,
                    width,
                    height,
                    number,
                    background,
                    scrollbar,
                    kind,
                } => match kind {
                    LocalizedKind::Plain => ascii(
                        &format!(
                            "{{ xmfhtmlgump {x} {y} {width} {height} {number} {} {} }}",
                            b(*background),
                            b(*scrollbar)
                        ),
                        &mut out,
                    ),
                    LocalizedKind::Color(color) => ascii(
                        &format!(
                            "{{ xmfhtmlgumpcolor {x} {y} {width} {height} {number} {} {} {color} }}",
                            b(*background),
                            b(*scrollbar)
                        ),
                        &mut out,
                    ),
                    LocalizedKind::Args { args, color } => latin1(
                        &format!(
                            "{{ xmfhtmltok {x} {y} {width} {height} {} {} {color} {number} @{args}@ }}",
                            b(*background),
                            b(*scrollbar)
                        ),
                        &mut out,
                    ),
                },
                GumpEntry::Image {
                    x,
                    y,
                    gump_id,
                    hue,
                    class,
                } => {
                    let cls = class.as_deref().filter(|c| !c.is_empty());
                    let s = match (*hue != 0, cls) {
                        (true, Some(c)) => {
                            format!("{{ gumppic {x} {y} {gump_id} hue={hue} class={c} }}")
                        }
                        (true, None) => format!("{{ gumppic {x} {y} {gump_id} hue={hue} }}"),
                        (false, Some(c)) => format!("{{ gumppic {x} {y} {gump_id} class={c} }}"),
                        (false, None) => format!("{{ gumppic {x} {y} {gump_id} }}"),
                    };
                    ascii(&s, &mut out);
                }
                GumpEntry::ImageTileButton {
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
                } => ascii(
                    &format!(
                        "{{ buttontileart {x} {y} {normal_id} {pressed_id} {} {param} {button_id} {item_id} {hue} {width} {height} }}",
                        *kind as i32
                    ),
                    &mut out,
                ),
                GumpEntry::ImageTiled {
                    x,
                    y,
                    width,
                    height,
                    gump_id,
                } => {
                    ascii(
                        &format!("{{ gumppictiled {x} {y} {width} {height} {gump_id} }}"),
                        &mut out,
                    );
                }
                GumpEntry::Item { x, y, item_id, hue } => ascii(
                    &if *hue == 0 {
                        format!("{{ tilepic {x} {y} {item_id} }}")
                    } else {
                        format!("{{ tilepichue {x} {y} {item_id} {hue} }}")
                    },
                    &mut out,
                ),
                GumpEntry::ItemProperty(s) => {
                    ascii(&format!("{{ itemproperty {} }}", s.value()), &mut out)
                }
                GumpEntry::Label { x, y, hue, text } => {
                    let i = idx(text, &mut strings);
                    ascii(&format!("{{ text {x} {y} {hue} {i} }}"), &mut out);
                }
                GumpEntry::LabelCropped {
                    x,
                    y,
                    width,
                    height,
                    hue,
                    text,
                } => {
                    let i = idx(text, &mut strings);
                    ascii(
                        &format!("{{ croppedtext {x} {y} {width} {height} {hue} {i} }}"),
                        &mut out,
                    );
                }
                GumpEntry::MasterGump(id) => ascii(&format!("{{ mastergump {id} }}"), &mut out),
                GumpEntry::Radio {
                    x,
                    y,
                    inactive_id,
                    active_id,
                    initial_state,
                    switch_id,
                } => {
                    ascii(
                        &format!(
                            "{{ radio {x} {y} {inactive_id} {active_id} {} {switch_id} }}",
                            b(*initial_state)
                        ),
                        &mut out,
                    );
                    switches += 1;
                }
                GumpEntry::SpriteImage {
                    x,
                    y,
                    gump_id,
                    width,
                    height,
                    sx,
                    sy,
                } => {
                    ascii(
                        &format!("{{ picinpic {x} {y} {gump_id} {width} {height} {sx} {sy} }}"),
                        &mut out,
                    );
                }
                GumpEntry::TextEntry {
                    x,
                    y,
                    width,
                    height,
                    hue,
                    entry_id,
                    initial_text,
                } => {
                    let i = idx(initial_text, &mut strings);
                    ascii(
                        &format!("{{ textentry {x} {y} {width} {height} {hue} {entry_id} {i} }}"),
                        &mut out,
                    );
                    text_entries += 1;
                }
                GumpEntry::TextEntryLimited {
                    x,
                    y,
                    width,
                    height,
                    hue,
                    entry_id,
                    initial_text,
                    size,
                } => {
                    let i = idx(initial_text, &mut strings);
                    ascii(
                        &format!(
                            "{{ textentrylimited {x} {y} {width} {height} {hue} {entry_id} {i} {size} }}"
                        ),
                        &mut out,
                    );
                    text_entries += 1;
                }
                GumpEntry::Tooltip { number, args } => {
                    match args.as_deref().filter(|a| !a.is_empty()) {
                        None => ascii(&format!("{{ tooltip {number} }}"), &mut out),
                        Some(a) => latin1(&format!("{{ tooltip {number} @{a}@ }}"), &mut out),
                    }
                }
            }
        }

        let mut table = Vec::new();
        for s in &strings {
            let units: Vec<u16> = s.encode_utf16().collect();
            table.extend_from_slice(&(units.len() as u16).to_be_bytes());
            for u in units {
                table.extend_from_slice(&u.to_be_bytes());
            }
        }
        out.push(0); // Layout text terminator

        Compiled {
            layout: out,
            strings: table,
            strings_count: strings.len(),
            switches,
            text_entries,
            has_visual_elements: has_visual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_layout_text_and_shared_strings() {
        let mut g = LegacyLayout {
            draggable: false,
            ..LegacyLayout::default()
        };
        g.add_page(0);
        g.add_background(1, 2, 3, 4, 5);
        g.add_label(10, 20, 30, "same");
        g.add_label(10, 40, 30, "same");
        g.add_text_entry(1, 2, 3, 4, 5, 6, "");
        g.add_check(0, 0, 1, 2, true, 9);
        g.add_button(0, 0, 1, 2, 77);
        g.add_html_localized_args(0, 0, 1, 1, 100, "a", 5, false, true);
        let c = g.compile();
        assert_eq!(
            String::from_utf8_lossy(&c.layout),
            "{ nomove }{ page 0 }{ resizepic 1 2 5 3 4 }{ text 10 20 30 0 }{ text 10 40 30 0 }{ textentry 1 2 3 4 5 6 1 }{ checkbox 0 0 1 2 1 9 }{ button 0 0 1 2 1 0 77 }{ xmfhtmltok 0 0 1 1 0 1 5 100 @a@ }\0"
        );
        assert_eq!(c.strings_count, 2);
        assert_eq!(c.strings, [0, 4, 0, b's', 0, b'a', 0, b'm', 0, b'e', 0, 0]);
        assert_eq!((c.switches, c.text_entries), (1, 1));
        assert!(c.has_visual_elements);
        assert!(g.has_button(77) && !g.has_button(78));
    }
}
