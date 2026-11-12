//! `[add <type> <args>`: the overload tables of the `[Constructible]` constructors.

use muo_server::World;
use muo_server::type_registry::{Phase, invoke_phase};

fn world() -> World {
    let mut w = World::new();
    muo::register_content(&mut w.types);
    invoke_phase(&mut w, Phase::Configure);
    for (i, name) in [(0, "Felucca"), (1, "Trammel")] {
        w.maps.add(muo_server::map::Map::new(
            i,
            i,
            i,
            7168,
            4096,
            0,
            name,
            Default::default(),
        ));
    }
    w
}

/// Writes target/ctors-registered.txt (`full name<TAB>arity,arity..` in table order).
#[test]
fn dump_registered_ctor_tables() {
    let w = world();
    let mut rows: Vec<String> = muo_content::props::all_ctors(&w)
        .into_iter()
        .map(|(n, t)| {
            let a: Vec<String> = t.iter().map(|c| c.params.len().to_string()).collect();
            format!("{n}\t{}", a.join(","))
        })
        .collect();
    rows.sort();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    std::fs::write(dir.join("ctors-registered.txt"), rows.join("\n")).unwrap();
    assert!(rows.len() > 50);
}

/// Writes target/ctors-types.txt (`full name<TAB>name:Type,..` per constructor, as `[add` prints it) for review.
#[test]
fn dump_registered_ctor_param_types() {
    let w = world();
    let mut rows = Vec::new();
    for (n, t) in muo_content::props::all_ctors(&w) {
        for c in t {
            let ps: Vec<String> = c
                .params
                .iter()
                .map(|p| format!("{}:{}", p.name, muo_commands::add::param_type(p)))
                .collect();
            rows.push(format!("{n}	{}", ps.join(",")));
        }
    }
    rows.sort();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    std::fs::write(
        dir.join("ctors-types.txt"),
        rows.join(
            "
",
        ),
    )
    .unwrap();
    assert!(rows.len() > 50);
}

/// The classes of a generated `ctor_tables.rs` that have a constructor with arguments, with the arguments for the
/// last such constructor whose parameters are all plain numbers, bools or strings.
fn samples(src: &str, n: usize) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    // Class names come from the `add_generated_ctors(w, "Full", &C_X)` lines, the constructors from the `C_X` statics.
    let mut classes: Vec<(String, String)> = Vec::new();
    for line in src.lines() {
        if let Some(rest) = line.trim().strip_prefix("add_generated_ctors(w, \"") {
            let full = rest.split('"').next().unwrap().to_string();
            let ident = rest
                .split("&C_")
                .nth(1)
                .unwrap()
                .trim_end_matches(");")
                .to_string();
            classes.push((full, format!("C_{ident}")));
        }
    }
    for (full, ident) in classes {
        let Some(body) = src.split(&format!("static {ident}: ")).nth(1) else {
            continue;
        };
        let body = body
            .split(
                "
];",
            )
            .next()
            .unwrap();
        let mut best = None;
        for ctor in body.split("    Ctor {").skip(1) {
            let params = ctor
                .split("params: &[")
                .nth(1)
                .unwrap()
                .split(
                    "],
",
                )
                .next()
                .unwrap();
            if params.trim().is_empty() {
                continue;
            }
            let mut args = Vec::new();
            let mut ok = true;
            for p in params.split("Param {").skip(1) {
                let kind = p
                    .split("kind: ")
                    .nth(1)
                    .unwrap()
                    .split(',')
                    .next()
                    .unwrap()
                    .trim();
                match kind {
                    "INT" | "UINT" | "LONG" | "ULONG" | "SHORT" | "USHORT" | "BYTE" | "SBYTE" => {
                        args.push("7".to_string())
                    }
                    "BOOL" => args.push("true".to_string()),
                    "DOUBLE" => args.push("1.5".to_string()),
                    "STR" => args.push("abc".to_string()),
                    _ => ok = false,
                }
            }
            if ok {
                best = Some(args);
            }
        }
        if let Some(args) = best {
            out.push((full, args));
        }
    }
    // Spread the sample over the file instead of taking ten neighbours.
    let step = (out.len() / n).max(1);
    out.into_iter().step_by(step).take(n).collect()
}

/// `[add <type> <args>` builds objects through the overload tables: ten classes with arguments from each generated
/// crate (fewer where a crate has fewer), made by `invoke_args` as a Game Master types them.
#[test]
fn add_builds_a_sample_of_ten_per_crate_through_invoke_args() {
    let crates: [(&str, &str); 5] = [
        (
            "muo_content",
            include_str!("../../muo_content/src/ctor_tables.rs"),
        ),
        (
            "muo_items",
            include_str!("../../muo_items/src/ctor_tables.rs"),
        ),
        (
            "muo_engines",
            include_str!("../../muo_engines/src/ctor_tables.rs"),
        ),
        (
            "muo_mobiles",
            include_str!("../../muo_mobiles/src/ctor_tables.rs"),
        ),
        (
            "muo_spells",
            include_str!("../../muo_spells/src/ctor_tables.rs"),
        ),
    ];
    let mut w = world();
    w.core.base_directory = std::env::temp_dir().join(format!("muo-ctors-{}", std::process::id()));
    let gm = w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE);
    w.mobile_mut(gm).unwrap().access_level = muo_server::mobile::AccessLevel::Owner;
    let p = muo_primitives::Point3D::new(100, 100, 0);
    let mut failed = Vec::new();
    for (crate_name, src) in crates {
        let picked = samples(src, 10);
        if picked.is_empty() {
            // muo_spells has one table, of a constructor taking an enum.
            assert_eq!(crate_name, "muo_spells");
            continue;
        }
        for (full, args) in picked {
            let short = full.rsplit('.').next().unwrap();
            let line = format!("{short} {}", args.join(" "));
            let before = w.items.len() + w.mobiles.len();
            muo_commands::add::invoke_args(
                &mut w,
                gm,
                p,
                p,
                &muo_server::commands::split(&line),
                None,
                false,
                false,
            );
            if w.items.len() + w.mobiles.len() == before {
                failed.push(format!("{crate_name}: [add {line}"));
            }
        }
    }
    assert!(
        failed.is_empty(),
        "{}",
        failed.join(
            "
"
        )
    );
}
