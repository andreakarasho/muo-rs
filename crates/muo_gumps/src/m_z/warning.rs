//! `WarningGump`, `NoticeGump`, `StaticWarningGump` and `StaticNoticeGump`.
//!
//! The subclasses (`NoticeGump : WarningGump`, every `StaticWarningGump<T>` subclass) have their own class name,
//! which the client's gump type id is a hash of: [`WarningKind`] and [`StaticWarning`] carry it.

use std::marker::PhantomData;
use std::rc::Rc;

use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{DynamicGumpBuilder, GumpStrings, StaticGumpBuilder};
use muo_content::text::add_html_text_builder;
use muo_io::text::TextDefinition;
use muo_server::World;
use muo_server::net_state::NetStateId;

use super::common::draw_builder;

/// `Action<bool>`: `true` when OKAY was pressed.
pub type WarningCallback = Rc<dyn Fn(&mut World, bool)>;

/// Names the class an instance of [`WarningGump`] belongs to.
pub trait WarningKind: 'static {
    const TYPE_NAME: &'static str;
}

pub struct IsWarningGump;
impl WarningKind for IsWarningGump {
    const TYPE_NAME: &'static str = "Server.Gumps.WarningGump";
}

pub struct IsNoticeGump;
impl WarningKind for IsNoticeGump {
    const TYPE_NAME: &'static str = "Server.Gumps.NoticeGump";
}

type CallbackFn22 = Rc<dyn Fn(&mut World)>;
type CallbackFn2 = Rc<dyn Fn(&mut World)>;
/// `WarningGump` (and `NoticeGump`, which only fixes the constructor's arguments).
pub struct WarningGump<K: WarningKind = IsWarningGump> {
    header: i32,
    header_color: i32,
    content_color: i32,
    width: i32,
    height: i32,
    cancel_button: bool,
    content: TextDefinition,
    callback: Option<WarningCallback>,
    _kind: PhantomData<K>,
}

pub type NoticeGump = WarningGump<IsNoticeGump>;

/// `new WarningGump(header, headerColor, content, contentColor, width, height, callback, cancelButton)`.
#[allow(clippy::too_many_arguments)]
pub fn new_full(
    header: i32,
    header_color: i32,
    content: TextDefinition,
    content_color: i32,
    width: i32,
    height: i32,
    callback: Option<WarningCallback>,
    cancel_button: bool,
) -> DynamicGumpObj<WarningGump<IsWarningGump>> {
    build(
        header,
        header_color,
        content,
        content_color,
        width,
        height,
        callback,
        cancel_button,
    )
}

/// `new WarningGump(content, width, height, callback, cancelButton)`.
pub fn new(
    content: TextDefinition,
    width: i32,
    height: i32,
    callback: Option<WarningCallback>,
    cancel_button: bool,
) -> DynamicGumpObj<WarningGump<IsWarningGump>> {
    build(
        1060635, // <CENTER>WARNING</CENTER>
        0x7800,
        content,
        0xFFC000,
        width,
        height,
        callback,
        cancel_button,
    )
}

/// `new NoticeGump(content, width, height, callback)`.
pub fn new_notice(
    content: TextDefinition,
    width: i32,
    height: i32,
    callback: Option<CallbackFn22>,
) -> DynamicGumpObj<NoticeGump> {
    new_notice_full(
        1060637, // <CENTER>NOTICE</CENTER>
        0x7800, content, 0xFFC000, width, height, callback,
    )
}

/// `new NoticeGump(header, headerColor, content, contentColor, width, height, callback)`.
pub fn new_notice_full(
    header: i32,
    header_color: i32,
    content: TextDefinition,
    content_color: i32,
    width: i32,
    height: i32,
    callback: Option<CallbackFn2>,
) -> DynamicGumpObj<NoticeGump> {
    let callback: Option<WarningCallback> = callback.map(|cb| {
        let f: WarningCallback = Rc::new(move |w: &mut World, _| cb(w));
        f
    });
    build(
        header,
        header_color,
        content,
        content_color,
        width,
        height,
        callback,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn build<K: WarningKind>(
    header: i32,
    header_color: i32,
    content: TextDefinition,
    content_color: i32,
    width: i32,
    height: i32,
    callback: Option<WarningCallback>,
    cancel_button: bool,
) -> DynamicGumpObj<WarningGump<K>> {
    DynamicGumpObj::new(
        (640 - width) / 2,
        (480 - height) / 2,
        WarningGump {
            header,
            header_color,
            content_color,
            width,
            height,
            cancel_button,
            content,
            callback,
            _kind: PhantomData,
        },
    )
}

impl<K: WarningKind> DynamicGumpLogic for WarningGump<K> {
    const TYPE_NAME: &'static str = K::TYPE_NAME;

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let (width, height) = (self.width, self.height);
        b.set_no_close();

        b.add_page(0);

        b.add_background(0, 0, width, height, 5054);

        b.add_image_tiled(10, 10, width - 20, 20, 2624);
        b.add_alpha_region(10, 10, width - 20, 20);
        b.add_html_localized_color(
            10,
            10,
            width - 20,
            20,
            self.header,
            self.header_color,
            false,
            false,
        );

        b.add_image_tiled(10, 40, width - 20, height - 80, 2624);
        b.add_alpha_region(10, 40, width - 20, height - 80);

        let text = add_html_text_builder(
            Some(&self.content),
            false,
            true,
            self.content_color,
            self.content_color,
        );
        draw_builder(b, text, 10, 40, width - 20, height - 80);

        b.add_image_tiled(10, height - 30, width - 20, 20, 2624);
        b.add_alpha_region(10, height - 30, width - 20, 20);

        b.add_button(10, height - 30, 4005, 4007, 1);
        b.add_html_localized_color(40, height - 30, 170, 20, 1011036, 0x7FFF, false, false); // OKAY

        if self.cancel_button {
            b.add_button(10 + (width - 20) / 2, height - 30, 4005, 4007, 0);
            b.add_html_localized_color(
                40 + (width - 20) / 2,
                height - 30,
                170,
                20,
                1011012,
                32767,
                false,
                false,
            ); // CANCEL
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if let Some(cb) = self.callback.clone() {
            cb(w, info.button_id == 1);
        }
    }
}

/// What a `StaticWarningGump<T>` subclass overrides. The defaults are the base class's.
pub trait StaticWarning: 'static {
    /// The subclass's full name.
    const TYPE_NAME: &'static str;
    const WIDTH: i32;
    const HEIGHT: i32;

    /// `Header`.
    fn header(&self) -> i32 {
        1060635 // <CENTER>WARNING</CENTER>
    }
    fn header_color(&self) -> i32 {
        0x7800
    }
    fn content_color(&self) -> &str {
        "#FFC000"
    }
    fn cancel_button(&self) -> bool {
        true
    }
    /// If this is non-zero, the content is localized and cached.
    fn static_localized_content(&self) -> i32 {
        0
    }
    fn static_localized_content_color(&self) -> i32 {
        0x7F00
    }
    fn content(&self) -> Option<String> {
        None
    }
}

/// `StaticWarningGump<T>` over a subclass.
pub struct StaticWarningGump<S: StaticWarning> {
    pub spec: S,
    callback: Option<WarningCallback>,
}

/// `new T(callback)`: the gump sits in the middle of a 640x480 screen.
pub fn new_static<S: StaticWarning>(
    spec: S,
    callback: Option<WarningCallback>,
) -> StaticGumpObj<StaticWarningGump<S>> {
    StaticGumpObj::new(
        (640 - S::WIDTH) / 2,
        (480 - S::HEIGHT) / 2,
        StaticWarningGump { spec, callback },
    )
}

impl<S: StaticWarning> StaticGumpLogic for StaticWarningGump<S> {
    const TYPE_NAME: &'static str = S::TYPE_NAME;

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        let (width, height) = (S::WIDTH, S::HEIGHT);
        let s = &self.spec;

        b.set_no_close();

        b.add_page(0);

        b.add_background(0, 0, width, height, 5054);

        b.add_image_tiled(10, 10, width - 20, 20, 2624);
        b.add_alpha_region(10, 10, width - 20, 20);
        b.add_html_localized_color(
            10,
            10,
            width - 20,
            20,
            s.header(),
            s.header_color(),
            false,
            false,
        );

        b.add_image_tiled(10, 40, width - 20, height - 80, 2624);
        b.add_alpha_region(10, 40, width - 20, height - 80);

        if s.static_localized_content() > 0 {
            b.add_html_localized_color(
                10,
                40,
                width - 20,
                height - 80,
                s.static_localized_content(),
                s.static_localized_content_color(),
                false,
                true,
            );
        } else {
            b.add_html_placeholder(10, 40, width - 20, height - 80, "content", false, true);
        }

        b.add_image_tiled(10, height - 30, width - 20, 20, 2624);
        b.add_alpha_region(10, height - 30, width - 20, 20);

        b.add_button(10, height - 30, 4005, 4007, 1);
        b.add_html_localized_color(40, height - 30, 170, 20, 1011036, 32767, false, false); // OKAY

        if s.cancel_button() {
            b.add_button(10 + (width - 20) / 2, height - 30, 4005, 4007, 0);
            b.add_html_localized_color(
                40 + (width - 20) / 2,
                height - 30,
                170,
                20,
                1011012,
                32767,
                false,
                false,
            ); // CANCEL
        }
    }

    fn build_strings(&self, strings: &mut GumpStrings) {
        strings.set_html_text(
            "content",
            self.spec.content().as_deref().unwrap_or(""),
            self.spec.content_color(),
            -1,
        );
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        if let Some(cb) = self.callback.clone() {
            cb(w, info.button_id == 1);
        }
    }
}

/// What a `StaticNoticeGump<T>` subclass overrides: a [`StaticWarning`] whose header is NOTICE and which has no
/// CANCEL button.
pub trait StaticNotice: 'static {
    const TYPE_NAME: &'static str;
    const WIDTH: i32;
    const HEIGHT: i32;

    fn header(&self) -> i32 {
        1060637 // <CENTER>NOTICE</CENTER>
    }
    fn header_color(&self) -> i32 {
        0x7800
    }
    fn content_color(&self) -> &str {
        "#FFC000"
    }
    fn static_localized_content(&self) -> i32 {
        0
    }
    fn static_localized_content_color(&self) -> i32 {
        0x7F00
    }
    fn content(&self) -> Option<String> {
        None
    }
}

type CallbackFn = Rc<dyn Fn(&mut World)>;
/// `StaticNoticeGump<T>`.
pub struct StaticNoticeAdapter<N: StaticNotice>(pub N);

impl<N: StaticNotice> StaticWarning for StaticNoticeAdapter<N> {
    const TYPE_NAME: &'static str = N::TYPE_NAME;
    const WIDTH: i32 = N::WIDTH;
    const HEIGHT: i32 = N::HEIGHT;

    fn header(&self) -> i32 {
        self.0.header()
    }
    fn header_color(&self) -> i32 {
        self.0.header_color()
    }
    fn content_color(&self) -> &str {
        self.0.content_color()
    }
    fn cancel_button(&self) -> bool {
        false
    }
    fn static_localized_content(&self) -> i32 {
        self.0.static_localized_content()
    }
    fn static_localized_content_color(&self) -> i32 {
        self.0.static_localized_content_color()
    }
    fn content(&self) -> Option<String> {
        self.0.content()
    }
}

/// `new T(callback)` for a notice.
pub fn new_static_notice<N: StaticNotice>(
    notice: N,
    callback: Option<CallbackFn>,
) -> StaticGumpObj<StaticWarningGump<StaticNoticeAdapter<N>>> {
    let callback: Option<WarningCallback> = callback.map(|cb| {
        let f: WarningCallback = Rc::new(move |w: &mut World, _| cb(w));
        f
    });
    new_static(StaticNoticeAdapter(notice), callback)
}

/// A `StaticWarningGump<T>` subclass whose `Content` is plain text: `K` picks the class (its name is what the client's
/// gump type id hashes), `W`x`H` its size.
struct TextWarning<const K: u8, const W: i32, const H: i32> {
    content: String,
    localized: i32,
    cancel: bool,
}

impl<const K: u8, const W: i32, const H: i32> StaticWarning for TextWarning<K, W, H> {
    const TYPE_NAME: &'static str = match K {
        0 => "Server.Commands.ClearCommands+DeleteObjectsNoticeGump",
        1 => "Server.Commands.Generic.DeleteCommand+DeleteObjectsNoticeGump",
        2 => "Server.Commands.Generic.OpenBrowserCommand+OpenBrowserWarningGump",
        3 => "Server.Commands.Generic.DesignInsertCommand+InsertObjectsNoticeGump",
        _ => "Server.Assistants.AssistantHandler+AssistantFailureWarningGump",
    };
    const WIDTH: i32 = W;
    const HEIGHT: i32 = H;
    fn cancel_button(&self) -> bool {
        self.cancel
    }
    fn static_localized_content(&self) -> i32 {
        self.localized
    }
    fn content(&self) -> Option<String> {
        Some(self.content.clone())
    }
}

fn send_text_warning<const K: u8, const W: i32, const H: i32>(
    w: &mut World,
    to: muo_server::ids::MobileId,
    spec: TextWarning<K, W, H>,
    cb: Option<WarningCallback>,
) {
    super::common::send(w, to, new_static(spec, cb));
}

/// `CommandHooks::warning_gump` (the commands' confirmation gumps, picked by their size) and
/// `AssistantHooks::warning_gump`.
pub fn configure(w: &mut World) {
    w.resources
        .get_mut::<muo_commands::hooks::CommandHooks>()
        .warning_gump = Some(|w, to, content, (width, height), cb| {
        let cb: Option<WarningCallback> = Some(Rc::new({
            let cb = std::cell::RefCell::new(Some(cb));
            move |w: &mut World, okay: bool| {
                if let Some(f) = cb.borrow_mut().take() {
                    f(w, okay);
                }
            }
        }));
        let content = content.to_string();
        match (width, height) {
            (360, 260) => send_text_warning(
                w,
                to,
                TextWarning::<0, 360, 260> {
                    content,
                    localized: 0,
                    cancel: true,
                },
                cb,
            ),
            (320, 240) => send_text_warning(
                w,
                to,
                TextWarning::<2, 320, 240> {
                    content,
                    localized: 0,
                    cancel: true,
                },
                cb,
            ),
            // DeleteCommand's and DesignInsertCommand's gumps are both 420x280; the delete one says "deleting".
            _ if content.starts_with("You are about to insert") => send_text_warning(
                w,
                to,
                TextWarning::<3, 420, 280> {
                    content,
                    localized: 0,
                    cancel: true,
                },
                cb,
            ),
            _ => send_text_warning(
                w,
                to,
                TextWarning::<1, 420, 280> {
                    content,
                    localized: 0,
                    cancel: true,
                },
                cb,
            ),
        }
    });
    w.resources
        .get_mut::<muo_content::assistants::assistant_handler::AssistantHooks>()
        .warning_gump = Some(|w, to, message| {
        let spec = TextWarning::<4, 420, 250> {
            content: message.string.clone().unwrap_or_default(),
            localized: message.number,
            cancel: false,
        };
        send_text_warning(w, to, spec, None);
    });
}
