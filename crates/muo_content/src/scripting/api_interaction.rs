//! Script functions for targeting cursors, text prompts and simple dialogs.
//!
//! Each takes the name of a script function to call with the player's answer, looked up when the answer comes (in
//! the calling script first, else any script), so a `[ReloadScripts` in between applies:
//!
//! ```rhai
//! fn command_smite(from, args) { target(from, "smite"); }
//! fn smite(from, what) { if type_of(what) == "Mobile" { what.lightning(); } }
//! ```

use muo_server::ids::MobileId;
use muo_server::net_state::NetStateId;
use muo_server::target::{TargetFlags, Targeted};
use muo_server::world::World;
use rhai::{Array, Dynamic, Engine, Map, NativeCallContext};

use super::api::{Ctx, Mobile, R, live_m, with_world};
use crate::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use crate::gumps::builder::DynamicGumpBuilder;

/// The script a call came from (where a callback is looked up first).
fn calling_script(ctx: &NativeCallContext) -> String {
    ctx.tag()
        .and_then(|t| t.read_lock::<Ctx>().map(|c| c.script.to_string()))
        .unwrap_or_default()
}

/// What a target cursor picked: a mobile or item handle, else `#{ x, y, z, item_id }` for land or a static.
fn targeted(t: Targeted) -> Dynamic {
    match t {
        Targeted::Entity(e) => super::api::entity(Some(e)),
        Targeted::Land(l) => {
            let mut m = Map::new();
            m.insert("x".into(), i64::from(l.location.x).into());
            m.insert("y".into(), i64::from(l.location.y).into());
            m.insert("z".into(), i64::from(l.location.z).into());
            m.insert("item_id".into(), Dynamic::from_int(0));
            Dynamic::from_map(m)
        }
        Targeted::Static(s) => {
            let mut m = Map::new();
            m.insert("x".into(), i64::from(s.location.x).into());
            m.insert("y".into(), i64::from(s.location.y).into());
            m.insert("z".into(), i64::from(s.location.z).into());
            m.insert("item_id".into(), i64::from(s.item_id).into());
            Dynamic::from_map(m)
        }
    }
}

fn begin_target(
    ctx: &NativeCallContext,
    from: Mobile,
    callback: &str,
    range: i64,
    ground: bool,
) -> R<()> {
    let (script, callback) = (calling_script(ctx), callback.to_owned());
    with_world(ctx, |w| {
        if live_m(w, from).is_none() {
            return;
        }
        let range = i32::try_from(range).unwrap_or(-1);
        muo_server::target::begin_target(
            w,
            from.0,
            range,
            ground,
            TargetFlags::NONE,
            move |w, m, t| {
                let args = vec![super::api::mobile(m), targeted(t)];
                if !super::call_named(w, &script, &callback, args) {
                    tracing::warn!(
                        "Script {script}: target: no function {callback}(from, targeted)"
                    );
                }
            },
        );
    })
}

/// A dialog: a title, some text and a row of buttons; answering calls `callback(from, button)` (1 for the first
/// button, 0 when closed).
struct ScriptDialog {
    title: String,
    text: String,
    buttons: Vec<String>,
    script: String,
    callback: String,
}

impl DynamicGumpLogic for ScriptDialog {
    const TYPE_NAME: &'static str = "Server.Scripting.ScriptDialog";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let height = 150 + 30 * self.buttons.len() as i32;
        b.add_background(0, 0, 400, height, 9270);
        b.add_alpha_region(10, 10, 380, height - 20);
        b.add_html(
            20,
            20,
            360,
            25,
            &format!("<center>{}</center>", self.title),
            false,
            false,
        );
        b.add_html(20, 50, 360, 80, &self.text, false, true);
        for (i, label) in self.buttons.iter().enumerate() {
            let y = 140 + 30 * i as i32;
            b.add_button(20, y, 4005, 4007, i as i32 + 1);
            b.add_label(55, y + 2, 1152, label);
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(m) = w.net_mobile(sender) else {
            return;
        };
        let button = i64::from(info.button_id.clamp(0, self.buttons.len() as i32));
        let args = vec![super::api::mobile(m), button.into()];
        if !super::call_named(w, &self.script, &self.callback, args) {
            tracing::warn!(
                "Script {}: dialog: no function {}(from, button)",
                self.script,
                self.callback
            );
        }
    }
}

fn send_dialog(w: &mut World, to: MobileId, dialog: ScriptDialog) {
    let gump = DynamicGumpObj::new(100, 100, dialog);
    crate::gumps::system::send_gump_to_mobile(w, to, Box::new(gump), false);
}

pub(crate) fn register(e: &mut Engine) {
    // `target(from, "callback")`: a cursor; picking something calls `callback(from, targeted)` with a mobile, an
    // item, or `#{ x, y, z, item_id }` for land and statics. `target(from, "callback", range)`: within `range` tiles.
    e.register_fn(
        "target",
        |ctx: NativeCallContext, from: Mobile, callback: &str| {
            begin_target(&ctx, from, callback, -1, true)
        },
    );
    e.register_fn(
        "target",
        |ctx: NativeCallContext, from: Mobile, callback: &str, range: i64| {
            begin_target(&ctx, from, callback, range, true)
        },
    );
    // `prompt(from, text, "callback")`: asks for a line of text; `callback(from, answer)` (`""` when cancelled)
    e.register_fn(
        "prompt",
        |ctx: NativeCallContext, from: Mobile, text: &str, callback: &str| {
            let (script, callback) = (calling_script(&ctx), callback.to_owned());
            let text = text.to_owned();
            with_world(&ctx, |w| {
                if live_m(w, from).is_none() {
                    return;
                }
                muo_server::mobile::part2::messages::send_message(w, from.0, &text);
                let (s2, c2) = (script.clone(), callback.clone());
                let answer: muo_server::prompt::PromptCallback =
                    std::rc::Rc::new(move |w, m, t| {
                        let args = vec![super::api::mobile(m), t.to_owned().into()];
                        if !super::call_named(w, &script, &callback, args) {
                            tracing::warn!(
                                "Script {script}: prompt: no function {callback}(from, text)"
                            );
                        }
                    });
                let cancel: muo_server::prompt::PromptCallback =
                    std::rc::Rc::new(move |w, m, _| {
                        super::call_named(w, &s2, &c2, vec![super::api::mobile(m), "".into()]);
                    });
                let p = muo_server::prompt::SimplePrompt::with_cancel(Some(answer), Some(cancel));
                muo_server::mobile::part2::speech::begin_prompt(w, from.0, p);
            })
        },
    );
    // `dialog(to, title, text, ["Yes", "No"], "callback")`: a window with buttons; `callback(to, button)` gets 1
    // for the first button, 2 for the second, .., 0 when closed. Text may hold simple HTML (`<br>`, `<b>`).
    e.register_fn(
        "dialog",
        |ctx: NativeCallContext,
         to: Mobile,
         title: &str,
         text: &str,
         buttons: Array,
         callback: &str| {
            let dialog = ScriptDialog {
                title: title.to_owned(),
                text: text.to_owned(),
                buttons: buttons
                    .into_iter()
                    .map(|b| b.to_string())
                    .take(10)
                    .collect(),
                script: calling_script(&ctx),
                callback: callback.to_owned(),
            };
            with_world(&ctx, |w| {
                if live_m(w, to).is_some() {
                    send_dialog(w, to.0, dialog);
                }
            })
        },
    );
    // `cancel_target(from)`: closes an open cursor
    e.register_fn("cancel_target", |ctx: NativeCallContext, from: Mobile| {
        with_world(&ctx, |w| {
            muo_server::target::cancel_mobile_target(w, from.0)
        })
    });
}
