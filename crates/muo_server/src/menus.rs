//! Menus (`BaseMenu`, `IMenu`, `ItemListMenu`, `QuestionMenu`) and the menu half of
//! `NetState` (`AddMenu`, `RemoveMenu`, `ClearMenus`).
//!
//! A menu is owned by the `NetStateData::menus` list from `SendTo` until the response or cancel packet takes it
//! out (`remove_menu_at`), so there is no menu store in the world. A subclass is a [`Menu`] with `on_response` /
//! `on_cancel` closures (they capture what a subclass would keep in fields).

use std::fmt;
use std::rc::Rc;

use crate::net_state::{MENU_CAP, NetStateId};
use crate::world::World;

/// `ItemListEntry`.
#[derive(Clone, Debug)]
pub struct ItemListEntry {
    pub name: String,
    pub item_id: i32,
    pub hue: i32,
    pub craft_index: i32,
}

impl ItemListEntry {
    pub fn new(name: &str, item_id: i32, hue: i32, craft_index: i32) -> ItemListEntry {
        ItemListEntry {
            name: name.trim().to_string(),
            item_id,
            hue,
            craft_index,
        }
    }
}

#[derive(Clone, Debug)]
pub enum MenuKind {
    ItemList {
        question: String,
        entries: Vec<ItemListEntry>,
    },
    Question {
        question: String,
        answers: Vec<String>,
    },
}

pub type MenuCancel = Rc<dyn Fn(&mut World, NetStateId)>;
pub type MenuResponse = Rc<dyn Fn(&mut World, NetStateId, i32)>;

/// `IMenu` (`BaseMenu` with `ItemListMenu` or `QuestionMenu`).
#[derive(Clone)]
pub struct Menu {
    pub serial: i32,
    pub kind: MenuKind,
    pub on_cancel: Option<MenuCancel>,
    pub on_response: Option<MenuResponse>,
}

impl fmt::Debug for Menu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Menu")
            .field("serial", &self.serial)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

/// `BaseMenu()`: the next serial, wrapping back to 1 when it stops being positive.
fn next_serial(w: &mut World) -> i32 {
    let mut serial = w.ui_serials.menu.wrapping_add(1);
    if serial <= 0 {
        serial = 1;
    }
    w.ui_serials.menu = serial;
    serial
}

impl Menu {
    /// `new ItemListMenu(question, entries)`.
    pub fn item_list(w: &mut World, question: &str, entries: Vec<ItemListEntry>) -> Menu {
        Menu {
            serial: next_serial(w),
            kind: MenuKind::ItemList {
                question: question.trim().to_string(),
                entries,
            },
            on_cancel: None,
            on_response: None,
        }
    }

    /// `new QuestionMenu(question, answers)`.
    pub fn question(w: &mut World, question: &str, answers: Vec<String>) -> Menu {
        Menu {
            serial: next_serial(w),
            kind: MenuKind::Question {
                question: question.trim().to_string(),
                answers,
            },
            on_cancel: None,
            on_response: None,
        }
    }

    /// `EntryLength`.
    pub fn entry_length(&self) -> usize {
        match &self.kind {
            MenuKind::ItemList { entries, .. } => entries.len(),
            MenuKind::Question { answers, .. } => answers.len(),
        }
    }

    /// `OnCancel(state)`: the base method does nothing.
    pub fn cancel(&self, w: &mut World, ns: NetStateId) {
        if let Some(f) = self.on_cancel.clone() {
            f(w, ns);
        }
    }

    /// `OnResponse(state, index)`: the base method does nothing.
    pub fn respond(&self, w: &mut World, ns: NetStateId, index: i32) {
        if let Some(f) = self.on_response.clone() {
            f(w, ns, index);
        }
    }
}

/// `NetState.AddMenu(menu)`.
pub fn add_menu(w: &mut World, ns: NetStateId, menu: Menu) {
    let Some(d) = w.net_state_mut(ns) else { return };
    if d.menus.len() < MENU_CAP {
        d.menus.push(menu);
    } else {
        tracing::info!("Client: {ns:?}: Exceeded menu cap, disconnecting...");
        w.disconnect(ns, "Exceeded menu cap.");
    }
}

/// `NetState.RemoveMenu(menu)`: by serial, as a menu is its own identity.
pub fn remove_menu(w: &mut World, ns: NetStateId, serial: i32) {
    if let Some(d) = w.net_state_mut(ns)
        && let Some(i) = d.menus.iter().position(|m| m.serial == serial)
    {
        d.menus.remove(i);
    }
}

/// `NetState.RemoveMenu(index)`: the removed menu, `None` for an index out of range.
pub fn remove_menu_at(w: &mut World, ns: NetStateId, index: usize) -> Option<Menu> {
    let d = w.net_state_mut(ns)?;
    (index < d.menus.len()).then(|| d.menus.remove(index))
}

/// `NetState.ClearMenus()`.
pub fn clear_menus(w: &mut World, ns: NetStateId) {
    if let Some(d) = w.net_state_mut(ns) {
        d.menus.clear();
    }
}

/// `IMenu.SendTo(state)`: `AddMenu`, then the display packet (dropped when the cap just disconnected it).
pub fn send_to(w: &mut World, ns: NetStateId, menu: Menu) {
    let bytes = crate::packets::menu::create_display_menu(&menu);
    add_menu(w, ns, menu);
    if !w.cannot_send_packets(ns) {
        w.send(ns, &bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_state::{NetOut, NetStateData};

    fn world_with_ns() -> (World, NetStateId) {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        (w, ns)
    }

    #[test]
    fn serials_count_from_one_and_wrap() {
        let mut w = World::new();
        assert_eq!(Menu::question(&mut w, " q ", vec![]).serial, 1);
        assert_eq!(Menu::question(&mut w, "q", vec![]).serial, 2);
        w.ui_serials.menu = i32::MAX;
        assert_eq!(Menu::question(&mut w, "q", vec![]).serial, 1);
    }

    #[test]
    fn send_to_adds_and_sends() {
        let (mut w, ns) = world_with_ns();
        let m = Menu::question(&mut w, "q", vec!["a".into()]);
        send_to(&mut w, ns, m);
        assert_eq!(w.net_state(ns).unwrap().menus.len(), 1);
        assert!(matches!(w.net_out.last(), Some(NetOut::Send(_, b)) if b[0] == 0x7C));
    }

    #[test]
    fn response_runs_the_callback_with_the_index() {
        let (mut w, ns) = world_with_ns();
        let seen = Rc::new(std::cell::Cell::new(-1));
        let s = seen.clone();
        let mut m = Menu::question(&mut w, "q", vec!["a".into()]);
        m.on_response = Some(Rc::new(move |_, _, i| s.set(i)));
        send_to(&mut w, ns, m);
        let m = remove_menu_at(&mut w, ns, 0).unwrap();
        m.respond(&mut w, ns, 3);
        assert_eq!(seen.get(), 3);
        assert!(remove_menu_at(&mut w, ns, 0).is_none());
    }

    #[test]
    fn exceeding_the_cap_disconnects_and_still_sends() {
        let (mut w, ns) = world_with_ns();
        for _ in 0..MENU_CAP {
            let m = Menu::question(&mut w, "q", vec![]);
            add_menu(&mut w, ns, m);
        }
        let m = Menu::question(&mut w, "q", vec![]);
        send_to(&mut w, ns, m);
        assert_eq!(w.net_state(ns).unwrap().menus.len(), MENU_CAP);
        // `SendTo` is AddMenu then the display packet; the queued disconnect does not stop that send
        assert!(matches!(
            w.net_out.as_slice(),
            [NetOut::Disconnect(_, r), NetOut::Send(..)] if r == "Exceeded menu cap."
        ));
    }
}
