//! The repository's `Data/Templates` and `Data/Scripts` load the way the server loads them at startup: strictly, with
//! every content crate's template keys registered. A typo or a key nobody registers fails here, not on a live server.

use muo_server::World;

#[test]
fn shipped_templates_and_scripts_load_strictly() {
    let mut w = World::new();
    muo::register_content(&mut w.types);
    w.core.base_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    muo_content::templates::reload(&mut w).unwrap();
    let errors = muo_scripting::reload_scripts(&mut w);
    assert!(errors.is_empty(), "{errors:?}");
    let t = muo_content::templates::table(&mut w);
    assert!(
        t.mobiles.len() > 400 && t.items.len() > 500,
        "{} {}",
        t.mobiles.len(),
        t.items.len()
    );
}
