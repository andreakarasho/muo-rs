//! `BaseConfirmGump`: the layout and response every confirmation gump of this family shares. A
//! subclass is a [`ConfirmHandler`] and its own `LegacyGumpLogic` that forwards `on_response` to [`on_response`].

use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_content::gumps::legacy::LegacyLayout;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::net_state::NetStateId;

use super::common::mobile_of;

mod buttons {
    pub const BREAK: i32 = 1;
    pub const CONFIRM: i32 = 2;
}

/// The virtual members of `BaseConfirmGump`.
pub trait ConfirmHandler {
    /// `TitleNumber`: `<center>Warning!</center>`.
    fn title_number(&self) -> i32 {
        1075083
    }
    /// `LabelNumber`: Are you sure you wish to select this?
    fn label_number(&self) -> i32 {
        1074975
    }
    fn confirm(&mut self, _w: &mut World, _from: MobileId) {}
    fn refuse(&mut self, _w: &mut World, _from: MobileId) {}
}

/// `new BaseConfirmGump()` for a subclass: the constructor fills the layout with its virtual numbers.
pub fn build<T: ConfirmHandler + LegacyGumpLogic>(logic: T) -> LegacyGumpObj<T> {
    let (title, label) = (logic.title_number(), logic.label_number());
    let mut g = LegacyGumpObj::new(120, 50, logic);
    g.layout.closable = false;
    g.layout.disposable = true;
    g.layout.draggable = true;
    g.layout.resizable = false;
    let l: &mut LegacyLayout = &mut g.layout;

    l.add_page(0);
    l.add_image_tiled(0, 0, 348, 262, 0xA8E);
    l.add_alpha_region(0, 0, 348, 262);
    l.add_image(0, 15, 0x27A8, 0);
    l.add_image_tiled(0, 30, 17, 200, 0x27A7);
    l.add_image(0, 230, 0x27AA, 0);
    l.add_image(15, 230, 0x280C, 0);
    l.add_image_tiled(30, 0, 300, 17, 0x280A);
    l.add_image(315, 0, 0x280E, 0);
    l.add_image(15, 244, 0x280C, 0);
    l.add_image_tiled(30, 244, 300, 17, 0x280A);
    l.add_image(315, 244, 0x280E, 0);
    l.add_image(330, 15, 0x27A8, 0);
    l.add_image_tiled(330, 30, 17, 200, 0x27A7);
    l.add_image(330, 230, 0x27AA, 0);
    l.add_image(333, 2, 0x2716, 0);
    l.add_image(315, 248, 0x2716, 0);
    l.add_image(2, 248, 0x2716, 0);
    l.add_image(2, 2, 0x2716, 0);
    l.add_html_localized_color(25, 25, 200, 20, title, 0x7D00, false, false);
    l.add_image(25, 40, 0xBBF, 0);
    l.add_html_localized_color(25, 55, 300, 120, label, 0x7FFF, false, false);

    l.add_radio(25, 175, 0x25F8, 0x25FB, true, buttons::BREAK);
    l.add_radio(25, 210, 0x25F8, 0x25FB, false, 0);

    l.add_html_localized_color(60, 180, 280, 20, 1074976, 0x7FFF, false, false);
    l.add_html_localized_color(60, 215, 280, 20, 1074977, 0x7FFF, false, false);

    l.add_button(265, 220, 0xF7, 0xF8, buttons::CONFIRM);
    g
}

/// `OnResponse`.
pub fn on_response<T: ConfirmHandler>(
    t: &mut T,
    w: &mut World,
    sender: NetStateId,
    info: &RelayInfo,
) {
    if info.button_id == buttons::CONFIRM {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if info.is_switched(buttons::BREAK) {
            t.confirm(w, from);
        } else {
            t.refuse(w, from);
        }
    }
}
