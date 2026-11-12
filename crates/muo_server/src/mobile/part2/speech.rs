//! `SayTo`, `BeginPrompt`, `SendHuePicker`, `SendMenu` and the speech-collection helper of `DoSpeech`.

use muo_primitives::MessageType;

use crate::hue_picker::{self, HuePicker};
use crate::ids::{EntityId, ItemId, MobileId};
use crate::item::ops as item_ops;
use crate::menus::{self, Menu};
use crate::packets::message as pk;
use crate::prompt::{self, PromptId, SimplePrompt};
use crate::world::World;

/// `PrivateOverheadMessage(Regular, SpeechHue, ascii, text, to.NetState)`.
pub fn say_to_ascii(w: &mut World, id: MobileId, to: MobileId, ascii: bool, text: &str) {
    let hue = w.mobile(id).map_or(0, |m| m.speech_hue);
    let ns = w.mobile(to).and_then(|m| m.net_state);
    super::messages::private_overhead_message(w, id, MessageType::REGULAR, hue, ascii, text, ns);
}

pub fn say_to(w: &mut World, id: MobileId, to: MobileId, text: &str) {
    say_to_ascii(w, id, to, false, text);
}

/// A localized message from us in our speech hue.
pub fn say_to_number(w: &mut World, id: MobileId, to: MobileId, number: i32, args: &str) {
    let hue = w.mobile(id).map_or(0, |m| m.speech_hue);
    say_to_number_hue(w, id, to, number, hue, args);
}

/// `SayTo(to, number, hue, args = "")`. Nothing is sent to a null `to.NetState`.
pub fn say_to_number_hue(
    w: &mut World,
    id: MobileId,
    to: MobileId,
    number: i32,
    hue: i32,
    args: &str,
) {
    let Some(m) = w.mobile(id) else { return };
    let (serial, body, name) = (
        m.serial,
        m.cur_body().body_id(),
        (m.ty.name)(w, id).unwrap_or_default(),
    );
    if let Some(ns) = w.mobile(to).and_then(|m| m.net_state) {
        pk::send_message_localized(
            w,
            ns,
            serial,
            body,
            pk::MessageType::REGULAR,
            hue,
            3,
            number,
            &name,
            args,
        );
    }
}

/// `Prompt = new SimplePrompt(..)`. The state overloads
/// (`SimpleStatePrompt<T>`) are the same call with a closure that captured the state.
pub fn begin_prompt(w: &mut World, id: MobileId, simple: SimplePrompt) -> PromptId {
    let pid = prompt::add_prompt(w, simple.into_prompt());
    crate::mobile::part1::props_b::set_prompt(w, id, Some(pid));
    pid
}

pub fn send_hue_picker(w: &mut World, id: MobileId, p: HuePicker) -> bool {
    match w.mobile(id).and_then(|m| m.net_state) {
        Some(ns) => {
            hue_picker::send_to(w, ns, p);
            true
        }
        None => false,
    }
}

pub fn send_menu(w: &mut World, id: MobileId, m: Menu) -> bool {
    match w.mobile(id).and_then(|m| m.net_state) {
        Some(ns) => {
            menus::send_to(w, ns, m);
            true
        }
        None => false,
    }
}

/// The items in `cont`, containers included, that handle speech.
pub fn add_speech_items_from(w: &World, list: &mut Vec<EntityId>, cont: ItemId) {
    let Some(c) = w.item(cont) else { return };
    for &item in &c.items {
        let Some(it) = w.item(item) else { continue };
        if (it.ty.handles_on_speech)(w, item) {
            list.push(EntityId::Item(item));
        }
        if item_ops::is_container(w, item) {
            add_speech_items_from(w, list, item);
        }
    }
}
