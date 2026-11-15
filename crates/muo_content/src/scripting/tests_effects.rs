//! Tests of the script functions for sounds and visual effects (`api_effects.rs`).

use super::tests::{call, script_world};

#[test]
fn effects_run_on_mobiles_items_and_places() {
    let mut w = script_world(
        "effects",
        r#"
        fn run(m) {
            m.move_to(100, 100, 0, 0);
            let torch = spawn_item("Gold", 101, 100, 0, 0);
            m.play_sound(0x1F5); torch.play_sound(0x208);
            play_sound_at(100, 100, 0, 0, 0x307);
            m.lightning(); torch.lightning();
            m.effect(0x3709, 10, 30); m.effect(0x3709, 10, 30, 33);
            m.particles(0x376A, 9, 32, 5030);
            effect_at(100, 100, 0, 0, 0x36BD, 20); effect_at(100, 100, 0, 0, 0x36BD, 20, 33);
            moving_effect(m, m, 0x36E4, 5); missile(m, torch, 0x36E4, 5);
            m.flash("LightFlash");
            true
        }
        fn bad(m) { m.flash("Sparkles"); }
        "#,
    );
    let m = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
    assert_eq!(
        call(&mut w, "run", vec![super::api::mobile(m)]).as_bool(),
        Ok(true)
    );
    assert!(
        super::call_value(&mut w, "bad", vec![super::api::mobile(m)]).is_err(),
        "an unknown screen effect is a script error"
    );
}
