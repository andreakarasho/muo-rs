//! Prompts (handle) and `SimplePrompt`.
//!
//! `Mobile::prompt` holds a [`PromptId`]; the `Mobile.Prompt` setter (cancel, `SendPrompt`) is in `props_b`.
//!
//! `SimpleStatePrompt<T>` is a [`SimplePrompt`] whose closure captured the state: a Rust closure already does
//! what the `T state` parameter was for.

use std::rc::Rc;

use crate::ids::MobileId;
use crate::world::World;

/// A `Prompt` reference.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PromptId(pub u32);

/// `PromptCallback(Mobile from, string text)`.
pub type PromptCallback = Rc<dyn Fn(&mut World, MobileId, &str)>;

/// `Prompt.OnCancel(from)`.
pub type PromptCancel = Rc<dyn Fn(&mut World, MobileId)>;

/// `Prompt`: a stored object (`World::prompts`) with the `OnResponse`/`OnCancel` virtuals as closures; unset is
/// the base method (nothing). `Mobile.Prompt` is a plain property, so the code that clears it also calls
/// [`remove_prompt`] (nothing else holds a prompt).
#[derive(Clone, Default)]
pub struct Prompt {
    pub on_cancel: Option<PromptCancel>,
    pub on_response: Option<PromptCallback>,
}

/// `new Prompt()`: takes the next serial (skipping 0) and stores the prompt; the id is its serial.
pub fn add_prompt(w: &mut World, prompt: Prompt) -> PromptId {
    let serial = loop {
        w.ui_serials.prompt = w.ui_serials.prompt.wrapping_add(1);
        if w.ui_serials.prompt != 0 {
            break w.ui_serials.prompt;
        }
    };
    let id = PromptId(serial as u32);
    w.prompts.insert(id, prompt);
    id
}

/// Drops a prompt nothing refers to any more.
pub fn remove_prompt(w: &mut World, id: PromptId) -> Option<Prompt> {
    w.prompts.shift_remove(&id)
}

/// `Prompt.OnResponse(from, text)`.
pub fn on_response(w: &mut World, id: PromptId, from: MobileId, text: &str) {
    if let Some(f) = w.prompts.get(&id).and_then(|p| p.on_response.clone()) {
        f(w, from, text);
    }
}

/// `Prompt.OnCancel(from)`.
pub fn on_cancel(w: &mut World, id: PromptId, from: MobileId) {
    if let Some(f) = w.prompts.get(&id).and_then(|p| p.on_cancel.clone()) {
        f(w, from);
    }
}

/// `SimplePrompt`.
#[derive(Clone)]
pub struct SimplePrompt {
    callback: Option<PromptCallback>,
    callback_handles_cancel: bool,
    cancel_callback: Option<PromptCallback>,
}

impl SimplePrompt {
    /// `SimplePrompt(callback, cancelCallback)`.
    pub fn with_cancel(
        callback: Option<PromptCallback>,
        cancel_callback: Option<PromptCallback>,
    ) -> Self {
        SimplePrompt {
            callback,
            callback_handles_cancel: false,
            cancel_callback,
        }
    }

    /// `SimplePrompt(callback, callbackHandlesCancel = false)`.
    pub fn new(callback: Option<PromptCallback>, callback_handles_cancel: bool) -> Self {
        SimplePrompt {
            callback,
            callback_handles_cancel,
            cancel_callback: None,
        }
    }

    /// The stored [`Prompt`] of this `SimplePrompt` (`SimplePrompt : Prompt`).
    pub fn into_prompt(self) -> Prompt {
        let (a, b) = (self.clone(), self);
        Prompt {
            on_response: Some(Rc::new(move |w, from, text| a.on_response(w, from, text))),
            on_cancel: Some(Rc::new(move |w, from| b.on_cancel(w, from))),
        }
    }

    /// `OnResponse`.
    pub fn on_response(&self, w: &mut World, from: MobileId, text: &str) {
        if let Some(cb) = &self.callback {
            cb(w, from, text);
        }
    }

    /// `OnCancel`.
    pub fn on_cancel(&self, w: &mut World, from: MobileId) {
        if self.callback_handles_cancel
            && let Some(cb) = &self.callback
        {
            cb(w, from, "");
        } else if let Some(cb) = &self.cancel_callback {
            cb(w, from, "");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use muo_primitives::Serial;

    use super::*;

    #[test]
    fn stored_prompt_runs_its_callbacks() {
        let log: Rc<RefCell<Vec<String>>> = Rc::default();
        let (l1, l2) = (log.clone(), log.clone());
        let mut w = World::new();
        let m = MobileId(Serial::new(1));
        let id = add_prompt(
            &mut w,
            Prompt {
                on_cancel: Some(Rc::new(move |_, _| l1.borrow_mut().push("cancel".into()))),
                on_response: Some(Rc::new(move |_, _, t| l2.borrow_mut().push(t.into()))),
            },
        );
        assert_eq!(id, PromptId(1));
        on_response(&mut w, id, m, "hi");
        on_cancel(&mut w, id, m);
        assert!(remove_prompt(&mut w, id).is_some());
        on_response(&mut w, id, m, "gone");
        assert_eq!(*log.borrow(), ["hi", "cancel"]);
    }

    #[test]
    fn cancel_routing() {
        let log: Rc<RefCell<Vec<String>>> = Rc::default();
        let (l1, l2) = (log.clone(), log.clone());
        let cb: PromptCallback = Rc::new(move |_, _, t| l1.borrow_mut().push(format!("cb:{t}")));
        let cancel: PromptCallback =
            Rc::new(move |_, _, t| l2.borrow_mut().push(format!("cancel:{t}")));
        let mut w = World::new();
        let m = MobileId(Serial::new(1));

        let p = SimplePrompt::with_cancel(Some(cb.clone()), Some(cancel));
        p.on_response(&mut w, m, "hi");
        p.on_cancel(&mut w, m);
        let p = SimplePrompt::new(Some(cb), true);
        p.on_cancel(&mut w, m);
        assert_eq!(*log.borrow(), ["cb:hi", "cancel:", "cb:"]);
    }
}
