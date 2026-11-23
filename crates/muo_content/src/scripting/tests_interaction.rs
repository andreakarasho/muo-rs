//! Tests of the script functions for targeting cursors, text prompts and dialogs (`api_interaction.rs`).

use muo_server::ids::EntityId;
use muo_server::target::Targeted;

use super::tests::{call, script_world};

const SCRIPT: &str = r#"
    fn pick(from) { from.move_to(100, 100, 0, 0); target(from, "picked"); }
    fn picked(from, what) { from.title = `picked ${type_of(what)}`; }
    fn ask(from) { prompt(from, "Your name?", "answered"); }
    fn answered(from, text) { from.title = `answer ${text}`; }
    fn show(from) { dialog(from, "Hello", "Pick one", ["A", "B"], "chosen"); }
    fn chosen(from, button) { from.title = `button ${button}`; }
"#;

fn title(w: &muo_server::world::World, m: muo_server::ids::MobileId) -> String {
    w.mobile(m).unwrap().title.clone().unwrap_or_default()
}

#[test]
fn a_target_cursor_calls_back_with_what_was_picked() {
    let mut w = script_world("target", SCRIPT);
    let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let _ = call(&mut w, "pick", vec![super::api::mobile(m)]);
    let t = w.mobile(m).unwrap().target.expect("a cursor is open");
    muo_server::target::invoke(&mut w, t, m, Targeted::Entity(EntityId::Mobile(m)));
    assert_eq!(title(&w, m), "picked Mobile");
}

#[test]
fn a_prompt_calls_back_with_the_answer() {
    let mut w = script_world("prompt", SCRIPT);
    let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let _ = call(&mut w, "ask", vec![super::api::mobile(m)]);
    let p = w.mobile(m).unwrap().prompt.expect("a prompt is open");
    muo_server::prompt::on_response(&mut w, p, m, "Bob");
    assert_eq!(title(&w, m), "answer Bob");
}

#[test]
fn a_dialog_to_a_mobile_without_a_client_is_harmless() {
    let mut w = script_world("dialog", SCRIPT);
    let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    let _ = call(&mut w, "show", vec![super::api::mobile(m)]);
}
