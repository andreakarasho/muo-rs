//! Hue pickers and the hue-picker half of `NetState` (`AddHuePicker`, `RemoveHuePicker`,
//! `ClearHuePickers`).
//!
//! A picker is owned by the `NetStateData::hue_pickers` list from `SendTo` until the response packet takes it out.

use std::fmt;
use std::rc::Rc;

use muo_primitives::Serial;

use crate::net_state::{HUE_PICKER_CAP, NetStateId};
use crate::world::World;

/// `HuePicker.OnResponse(hue)`.
pub type HueResponse = Rc<dyn Fn(&mut World, i32)>;

/// `HuePicker`; a subclass sets `on_response`.
#[derive(Clone)]
pub struct HuePicker {
    pub serial: Serial,
    pub item_id: i32,
    pub on_response: Option<HueResponse>,
}

impl fmt::Debug for HuePicker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HuePicker")
            .field("serial", &self.serial)
            .field("item_id", &self.item_id)
            .finish_non_exhaustive()
    }
}

impl HuePicker {
    /// `new HuePicker(itemID)`: the next serial, skipping 0.
    pub fn new(w: &mut World, item_id: i32) -> HuePicker {
        let serial = loop {
            w.ui_serials.hue_picker = w.ui_serials.hue_picker.wrapping_add(1);
            if w.ui_serials.hue_picker != 0 {
                break w.ui_serials.hue_picker;
            }
        };
        HuePicker {
            serial: Serial::new(serial),
            item_id,
            on_response: None,
        }
    }

    /// `OnResponse(hue)`: the base method does nothing.
    pub fn respond(&self, w: &mut World, hue: i32) {
        if let Some(f) = self.on_response.clone() {
            f(w, hue);
        }
    }
}

/// `SendDisplayHuePicker` (0x95), built by `packets::player`.
pub use crate::packets::player::send_display_hue_picker;

/// `HuePicker.SendTo(state)`: the packet, then `AddHuePicker`.
pub fn send_to(w: &mut World, ns: NetStateId, picker: HuePicker) {
    send_display_hue_picker(w, ns, picker.serial, picker.item_id);
    add_hue_picker(w, ns, picker);
}

/// `NetState.AddHuePicker(huePicker)`.
pub fn add_hue_picker(w: &mut World, ns: NetStateId, picker: HuePicker) {
    let Some(d) = w.net_state_mut(ns) else { return };
    if d.hue_pickers.len() < HUE_PICKER_CAP {
        d.hue_pickers.push(picker);
    } else {
        tracing::info!("Client: {ns:?}: Exceeded hue picker cap, disconnecting...");
        w.disconnect(ns, "Exceeded hue picker cap.");
    }
}

/// `NetState.RemoveHuePicker(huePicker)`.
pub fn remove_hue_picker(w: &mut World, ns: NetStateId, serial: Serial) {
    if let Some(d) = w.net_state_mut(ns)
        && let Some(i) = d.hue_pickers.iter().position(|p| p.serial == serial)
    {
        d.hue_pickers.remove(i);
    }
}

/// `NetState.RemoveHuePicker(index)`: the removed picker, `None` for an index out of range.
pub fn remove_hue_picker_at(w: &mut World, ns: NetStateId, index: usize) -> Option<HuePicker> {
    let d = w.net_state_mut(ns)?;
    (index < d.hue_pickers.len()).then(|| d.hue_pickers.remove(index))
}

/// `NetState.ClearHuePickers()`.
pub fn clear_hue_pickers(w: &mut World, ns: NetStateId) {
    if let Some(d) = w.net_state_mut(ns) {
        d.hue_pickers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net_state::{NetOut, NetStateData};

    #[test]
    fn send_to_sends_then_adds_and_respond_calls_back() {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        let seen = Rc::new(std::cell::Cell::new(0));
        let s = seen.clone();
        let mut p = HuePicker::new(&mut w, 0xEED);
        assert_eq!(p.serial, Serial::new(1));
        p.on_response = Some(Rc::new(move |_, h| s.set(h)));
        send_to(&mut w, ns, p);
        assert!(matches!(w.net_out.last(), Some(NetOut::Send(_, b)) if b[0] == 0x95));
        let p = remove_hue_picker_at(&mut w, ns, 0).unwrap();
        p.respond(&mut w, 42);
        assert_eq!(seen.get(), 42);
    }

    #[test]
    fn exceeding_the_cap_disconnects() {
        let mut w = World::new();
        let ns = NetStateId(1);
        w.net_states.insert(ns, NetStateData::default());
        for _ in 0..=HUE_PICKER_CAP {
            let p = HuePicker::new(&mut w, 1);
            add_hue_picker(&mut w, ns, p);
        }
        assert_eq!(w.net_state(ns).unwrap().hue_pickers.len(), HUE_PICKER_CAP);
        assert!(
            matches!(w.net_out.last(), Some(NetOut::Disconnect(_, r)) if r == "Exceeded hue picker cap.")
        );
    }
}
