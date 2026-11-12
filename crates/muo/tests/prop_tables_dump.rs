//! `cargo test -p muo --test prop_tables_dump -- --ignored` writes build/prop_tables.tsv (class<TAB>property), every
//! `[CommandProperty]` member, for coverage review.

#[test]
#[ignore = "writes build/prop_tables.tsv"]
fn dump_prop_tables() {
    let mut w = muo_server::World::new();
    muo_server::server_setup::register(&mut w.types);
    muo::register_content(&mut w.types);
    muo_server::type_registry::invoke_phase(&mut w, muo_server::type_registry::Phase::Configure);
    let mut out = String::new();
    for (class, names) in muo_content::props::declared_names(&w) {
        for n in names {
            out.push_str(&format!("{class}\t{n}\n"));
        }
    }
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/prop_tables.tsv");
    std::fs::write(path, out).unwrap();
    // every registered type's full name (a table of an abstract base class is attached to its registered descendants)
    let mut names = std::collections::BTreeSet::new();
    for t in w.types.item_types() {
        let mut c = Some(*t);
        while let Some(x) = c {
            names.insert(x.full_name);
            c = x.parent;
        }
    }
    for t in w.types.mobile_types() {
        let mut c = Some(*t);
        while let Some(x) = c {
            names.insert(x.full_name);
            c = x.parent;
        }
    }
    let types = names
        .into_iter()
        .collect::<Vec<_>>()
        .join(&String::from(char::from(10)));
    std::fs::write(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/types.txt"),
        types,
    )
    .unwrap();
}
