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

// ---- custom gumps ----

/// One element of a script gump.
enum El {
    Background(i32, i32, i32, i32, i32),
    Alpha(i32, i32, i32, i32),
    Label(i32, i32, i32, String),
    Html(i32, i32, i32, i32, String, bool, bool),
    Button(i32, i32, i32, i32, i32),
    Page(i32),
    PageButton(i32, i32, i32, i32, i32),
    Check(i32, i32, i32, bool),
    Item(i32, i32, i32, i32),
    Image(i32, i32, i32, i32),
    Tiled(i32, i32, i32, i32, i32),
    NoClose,
}

/// A gump laid out by a script; answering calls `callback(from, button, switches)` (button 0 when closed,
/// `switches` the ids of the ticked checkboxes).
struct ScriptGump {
    els: Vec<El>,
    script: String,
    callback: String,
}

impl DynamicGumpLogic for ScriptGump {
    const TYPE_NAME: &'static str = "Server.Scripting.ScriptGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        for e in &self.els {
            match e {
                El::Background(x, y, w, h, id) => b.add_background(*x, *y, *w, *h, *id),
                El::Alpha(x, y, w, h) => b.add_alpha_region(*x, *y, *w, *h),
                El::Label(x, y, hue, t) => b.add_label(*x, *y, *hue, t),
                El::Html(x, y, w, h, t, bg, scroll) => b.add_html(*x, *y, *w, *h, t, *bg, *scroll),
                El::Button(x, y, normal, pressed, id) => {
                    b.add_button(*x, *y, *normal, *pressed, *id)
                }
                El::Page(p) => b.add_page(*p),
                El::PageButton(x, y, normal, pressed, page) => b.add_button_type(
                    *x,
                    *y,
                    *normal,
                    *pressed,
                    0,
                    crate::gumps::layout::GumpButtonType::Page,
                    *page,
                ),
                El::Check(x, y, id, on) => b.add_checkbox(*x, *y, 210, 211, *on, *id),
                El::Item(x, y, id, hue) => b.add_item(*x, *y, *id, *hue),
                El::Image(x, y, id, hue) => b.add_image(*x, *y, *id, *hue),
                El::Tiled(x, y, w, h, id) => b.add_image_tiled(*x, *y, *w, *h, *id),
                El::NoClose => b.set_no_close(),
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(m) = w.net_mobile(sender) else {
            return;
        };
        let switches: Array = info
            .switches
            .iter()
            .map(|s| Dynamic::from(i64::from(*s)))
            .collect();
        let args = vec![
            super::api::mobile(m),
            i64::from(info.button_id).into(),
            switches.into(),
        ];
        if !super::call_named(w, &self.script, &self.callback, args) {
            tracing::warn!(
                "Script {}: gump: no function {}(from, button, switches)",
                self.script,
                self.callback
            );
        }
    }
}

/// One element from its map: `#{ kind: "label", x: 10, y: 20, text: "Hi" }` (see `register_gump`).
fn element(m: &Map) -> Result<El, String> {
    let int =
        |k: &str, d: i64| -> i32 { m.get(k).and_then(|v| v.as_int().ok()).unwrap_or(d) as i32 };
    let text = |k: &str| m.get(k).map(|v| v.to_string()).unwrap_or_default();
    let flag = |k: &str| m.get(k).and_then(|v| v.as_bool().ok()).unwrap_or(false);
    let (x, y) = (int("x", 0), int("y", 0));
    let kind = text("kind");
    Ok(match kind.as_str() {
        "background" => El::Background(x, y, int("w", 100), int("h", 100), int("id", 9270)),
        "alpha" => El::Alpha(x, y, int("w", 100), int("h", 100)),
        "label" => El::Label(x, y, int("hue", 1152), text("text")),
        "html" => El::Html(
            x,
            y,
            int("w", 200),
            int("h", 20),
            text("text"),
            flag("background"),
            flag("scroll"),
        ),
        "button" => El::Button(
            x,
            y,
            int("normal", 4005),
            int("pressed", 4007),
            int("id", 1),
        ),
        "page" => El::Page(int("page", 0)),
        "page_button" => El::PageButton(
            x,
            y,
            int("normal", 4005),
            int("pressed", 4007),
            int("page", 1),
        ),
        "check" => El::Check(x, y, int("id", 1), flag("checked")),
        "item" => El::Item(x, y, int("item_id", 0), int("hue", 0)),
        "image" => El::Image(x, y, int("id", 0), int("hue", 0)),
        "tiled" => El::Tiled(x, y, int("w", 100), int("h", 100), int("id", 0)),
        "no_close" => El::NoClose,
        other => {
            return Err(format!(
                "gump: unknown element kind '{other}' (background, alpha, label, html, button, page, page_button, check, item, image, tiled, no_close)"
            ));
        }
    })
}

/// `gump(to, [elements], "callback")`: a window laid out element by element.
///
/// Each element is a map with a `kind`: `background` (x, y, w, h, id), `alpha` (x, y, w, h), `label` (x, y, text,
/// hue), `html` (x, y, w, h, text, background, scroll), `button` (x, y, id, normal, pressed), `page` (page: what
/// follows is on that page; 0 is every page), `page_button` (x, y, page), `check` (x, y, id, checked), `item` (x, y,
/// item_id, hue), `image` (x, y, id, hue), `tiled` (x, y, w, h, id: an image repeated over the area), `no_close`
/// (right-click does not close it). Answering calls
/// `callback(to, button, switches)`.
pub(crate) fn register_gump(e: &mut Engine) {
    e.register_fn(
        "gump",
        |ctx: NativeCallContext, to: Mobile, elements: Array, callback: &str| -> R<()> {
            let els = elements
                .iter()
                .map(|v| {
                    v.read_lock::<Map>()
                        .ok_or_else(|| "gump: every element is a map".to_owned())
                        .and_then(|m| element(&m))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let gump = ScriptGump {
                els,
                script: calling_script(&ctx),
                callback: callback.to_owned(),
            };
            with_world(&ctx, |w| {
                if live_m(w, to).is_some() {
                    let g = DynamicGumpObj::new(50, 50, gump);
                    crate::gumps::system::send_gump_to_mobile(w, to.0, Box::new(g), false);
                }
            })
        },
    );
}
