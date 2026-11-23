//! Every class the crates make by type name (`factory::make_item`/`make_mobile`, `pack_new`, `si("Name")`, a
//! `format!("Server.Items.{x}")` whose `x` is a `let`/`for` over string literals, ..) must be registered: an unknown name silently produces nothing. The names that are not yet registered are listed in
//! `KNOWN_MISSING`; the test fails for any other and for a listed name that has since been registered.

use std::collections::BTreeSet;
use std::path::Path;

/// Not registered yet when this test was written (owners: items/mobiles tasks).
const KNOWN_MISSING: &[&str] = &[
    // `Seed` is made through the overload table of the plants engine (`hand/seed.rs`), not by the class name
];

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs")
            && !p
                .file_name()
                .is_some_and(|n| n.to_string_lossy().contains("tests"))
        {
            out.push(p);
        }
    }
}

/// The string literals of `line` that follow a call named like a class factory.
fn names_in(line: &str, out: &mut BTreeSet<String>) {
    const CALLS: &[&str] = &[
        "si(",
        "pack(",
        "pack_new(",
        "pack_new_amount(",
        "add_new(",
        "add_new_hue(",
        "add_nondyed(",
        "equip(",
        "make_item(",
        "make_mobile(",
        "make_item_amount(",
        "pack_opt(",
        "pack_new_hue(",
    ];
    let b = line.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'"' {
            let start = i + 1;
            let mut j = start;
            while j < b.len() && b[j] != b'"' {
                j += if b[j] == b'\\' { 2 } else { 1 };
            }
            let lit = &line[start..j.min(line.len())];
            let before = &line[..i];
            // Only a literal in a factory call counts (not a `mobile_is(.., "Server.Mobiles.X")` check or a name list).
            let in_call = CALLS.iter().any(|c| {
                before
                    .rsplit(&[';', '{', '}'][..])
                    .next()
                    .is_some_and(|s| s.contains(c))
            });
            let full = (lit.starts_with("Server.Items.") || lit.starts_with("Server.Mobiles."))
                && lit.len() > "Server.Mobiles.".len()
                && lit
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_');
            let short = lit.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                && lit.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            // fishing's `entry(.., &["Server.Items.MessageInABottle"])` lists what a cast can catch
            let in_call = in_call
                || (full
                    && before
                        .rsplit(&[';', '{', '}'][..])
                        .next()
                        .is_some_and(|s| s.contains("entry(")));
            if in_call && (full || short) {
                out.insert(lit.to_owned());
            }
            i = j + 1;
        } else {
            i += 1;
        }
    }
}

/// The identifier-like string literals of `text`.
fn literals(text: &str) -> Vec<String> {
    let mut out = vec![];
    let mut it = text.split('"');
    it.next();
    while let Some(lit) = it.next() {
        if lit.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && lit.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            out.push(lit.to_owned());
        }
        it.next();
    }
    out
}

/// The statement that binds `ident` (`let ident = ..;` / `for ident in ..{`) last before `at`, as text.
fn binding_of(text: &str, at: usize, ident: &str) -> Option<String> {
    let head = &text[..at];
    let start = [
        format!("let {ident} ="),
        format!("let {ident}:"),
        format!("for {ident} in"),
    ]
    .iter()
    .filter_map(|p| head.rfind(p.as_str()))
    .max()?;
    let rest = &text[start..];
    let mut depth = 0i32;
    for (i, c) in rest.char_indices() {
        match c {
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            ';' if depth <= 0 => return Some(rest[..i].to_owned()),
            _ => {}
        }
        if depth < 0 {
            return Some(rest[..i].to_owned());
        }
    }
    None
}

/// `factory-call(.. format!("Server.Items.{x}") ..)`: one name per literal `x` is bound to in the file.
fn format_names(text: &str, out: &mut BTreeSet<String>) {
    const CALLS: &[&str] = &[
        "si(",
        "pack(",
        "pack_new(",
        "add_new(",
        "equip(",
        "make_item(",
        "make_mobile(",
        "make_item_amount(",
    ];
    let mut from = 0;
    while let Some(p) = text[from..].find("format!(\"") {
        let at = from + p;
        from = at + 9;
        let line_start = text[..at].rfind('\n').map_or(0, |i| i + 1);
        if text[line_start..at].trim_start().starts_with("//")
            || !CALLS.iter().any(|c| text[line_start..at].contains(c))
        {
            continue;
        }
        let Some(end) = text[from..].find('"') else {
            continue;
        };
        let fmt = &text[from..from + end];
        if !(fmt.starts_with("Server.Items.")
            || fmt.starts_with("Server.Mobiles.")
            || fmt.starts_with('{'))
        {
            continue;
        }
        let (Some(open), Some(close)) = (fmt.find('{'), fmt.find('}')) else {
            continue;
        };
        let ident = &fmt[open + 1..close];
        let (prefix, suffix) = (&fmt[..open], &fmt[close + 1..]);
        if let Some(stmt) = binding_of(text, at, ident) {
            for lit in literals(&stmt) {
                out.insert(format!("{prefix}{lit}{suffix}"));
            }
        }
    }
}

#[test]
fn every_name_mobiles_makes_is_registered() {
    let mut w = muo_server::World::new();
    muo::register_content(&mut w.types);
    let registered: BTreeSet<&str> = w
        .types
        .item_types()
        .iter()
        .map(|t| t.full_name)
        .chain(w.types.mobile_types().iter().map(|t| t.full_name))
        .collect();
    let mut files = vec![];
    for krate in std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join(".."))
        .unwrap()
        .flatten()
    {
        // this crate's tests only name classes to look at
        if krate.file_name() != "muo" && krate.path().join("src").is_dir() {
            walk(&krate.path().join("src"), &mut files);
        }
    }
    let mut names = BTreeSet::new();
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap();
        format_names(&text, &mut names);
        // rustfmt splits a call over lines: scan the file as one line, without comments and type tables
        let code: Vec<&str> = text
            .lines()
            .filter(|l| !l.trim_start().starts_with("//") && !l.contains("full_name:"))
            .collect();
        names_in(&code.join(" "), &mut names);
    }
    // the scan sees names made by a literal (Neira's shroud), built with `format!` and in fishing's table
    for n in [
        "Server.Items.HoodedShroudOfShadows",
        "Server.Items.MessageInABottle",
        "Server.Mobiles.Bogling",
    ] {
        assert!(names.contains(n), "the scan misses {n}");
    }
    let known: BTreeSet<&str> = KNOWN_MISSING.iter().copied().collect();
    let mut missing = BTreeSet::new();
    for n in &names {
        let ok = if n.starts_with("Server.") {
            registered.contains(n.as_str())
        } else {
            registered.contains(format!("Server.Items.{n}").as_str())
                || registered.contains(format!("Server.Mobiles.{n}").as_str())
        };
        if !ok {
            missing.insert(n.as_str());
        }
    }
    eprintln!("unregistered names ({}): {missing:#?}", missing.len());
    let new: Vec<_> = missing.difference(&known).collect();
    let fixed: Vec<_> = known.difference(&missing).collect();
    assert!(new.is_empty(), "unregistered classes made by name: {new:?}");
    assert!(
        fixed.is_empty(),
        "now registered, remove from KNOWN_MISSING: {fixed:?}"
    );
}
