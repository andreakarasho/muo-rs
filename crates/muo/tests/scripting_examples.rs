//! The example scripts and templates in the repository's `Data/` load against the whole registered program.

use muo_server::World;
use muo_server::commands::CommandSystem;
use muo_server::mobile::MOBILE_BASE;
use muo_server::script_hooks::{self, ScriptArg, ScriptHook};

#[test]
fn examples_load_bind_and_run() {
    let mut w = World::new();
    muo::register_content(&mut w.types);
    w.core.base_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    muo_content::templates::reload(&mut w).unwrap();
    let errors = muo_scripting::reload_scripts(&mut w);
    assert!(errors.is_empty(), "{errors:?}");
    assert!(
        w.resources
            .get_mut::<CommandSystem>()
            .entries
            .contains_key("HELLO")
    );

    let rat = muo_content::factory::make_mobile(&mut w, "Examples.ShoutingRat").unwrap();
    let r = w.mobile(rat).unwrap();
    assert_eq!(r.name.as_deref(), Some("a shouting rat"));
    assert_eq!(r.hue, 0x21, "on_spawn ran");
    assert!(script_hooks::mobile_has(&w, rat, ScriptHook::Death));

    let player = w.new_mobile(&MOBILE_BASE);
    (MOBILE_BASE.set_str)(&mut w, player, 100);
    muo_server::mobile::part1::props_b::set_hits(&mut w, player, 50);
    let stone = muo_content::factory::make_item(&mut w, "Examples.HealingStone").unwrap();
    assert_eq!(w.item(stone).unwrap().hue, 1153);
    let used = [ScriptArg::Mobile(Some(player))];
    assert!(script_hooks::item(&mut w, stone, ScriptHook::Use, &used));
    assert!(w.mobile(player).unwrap().hits >= 60);
    assert_eq!(w.item(stone).unwrap().hue, 0x3B2, "recharging");
}
