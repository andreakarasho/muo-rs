//! `ConfirmHeritageGump`.

use muo_content::factory::make_item;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::net_state::NetStateId;

use super::common::{mobile_of, send};
use super::heritage_token;

mod buttons {
    pub const CANCEL: i32 = 0;
    pub const OKAY: i32 = 1;
}

pub struct ConfirmHeritageGump {
    /// The item types' full names.
    selected: &'static [&'static str],
    token: ItemId,
    cliloc: i32,
}

/// `DisplayTo(from, token, selected, cliloc)`.
pub fn display_to(
    w: &mut World,
    from: MobileId,
    token: ItemId,
    selected: &'static [&'static str],
    cliloc: i32,
) {
    if w.mobile(from).and_then(|m| m.net_state).is_none()
        || w.item(token).is_none_or(|i| i.deleted())
        || selected.is_empty()
    {
        return;
    }
    send(
        w,
        from,
        DynamicGumpObj::new(
            60,
            36,
            ConfirmHeritageGump {
                selected,
                token,
                cliloc,
            },
        ),
    );
}

impl DynamicGumpLogic for ConfirmHeritageGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ConfirmHeritageGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 291, 99, 0x13BE);
        b.add_image_tiled(5, 6, 280, 20, 0xA40);
        // Click "OKAY" to redeem the following promotional item:
        b.add_html_localized_color(9, 8, 280, 20, 1070972, 0x7FFF, false, false);
        b.add_image_tiled(5, 31, 280, 40, 0xA40);
        b.add_html_localized_color(9, 35, 272, 40, self.cliloc, 0x7FFF, false, false);
        b.add_button(180, 73, 0xFB7, 0xFB8, buttons::OKAY);
        b.add_html_localized_color(215, 75, 100, 20, 1011036, 0x7FFF, false, false); // OKAY
        b.add_button(5, 73, 0xFB1, 0xFB2, buttons::CANCEL);
        b.add_html_localized_color(40, 75, 100, 20, 1060051, 0x7FFF, false, false); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if w.item(self.token).is_none_or(|i| i.deleted()) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        match info.button_id {
            buttons::OKAY => {
                // `item` outlives the loop: a failed creation re-adds the earlier item
                let mut item = None;
                for ty in self.selected {
                    if let Some(i) = make_item(w, ty) {
                        item = Some(i);
                    } else {
                        tracing::error!("Failed to create heritage item of type {ty}");
                    }
                    if let Some(i) = item {
                        muo_server::item::base::delete(w, self.token);
                        muo_server::item::deps::add_to_backpack(w, from, i);
                    }
                }
            }
            buttons::CANCEL => heritage_token::display_to(w, from, self.token),
            _ => {}
        }
    }
}
