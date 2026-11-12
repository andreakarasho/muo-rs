//! Layout tests: the pet-resurrection gump in the four gump styles.

use muo_primitives::Serial;
use muo_primitives::html::TextAlignment;

use super::base::*;
use super::builder::{DynamicGumpBuilder, GumpBuilder, GumpStrings, HtmlStyle, StaticGumpBuilder};

const WILT: &str = "<div align=center>Wilt thou sanctify the resurrection of:</div>";

fn centered() -> HtmlStyle<'static> {
    HtmlStyle {
        align: TextAlignment::Center,
        ..HtmlStyle::NONE
    }
}

const ITEMS: [(i32, i32, i32); 5] = [
    (205, 40, 0x4),
    (227, 40, 0x5),
    (180, 78, 0xCAE),
    (195, 90, 0xCAD),
    (218, 95, 0xCB0),
];

/// The `BuildLayout` the dynamic and static test gumps share; no `pet_name` is the placeholder of `StaticLayoutTestGump`.
fn build_pet_layout(b: &mut GumpBuilder, pet_name: Option<&str>) {
    b.add_page(0);
    b.add_background(10, 10, 265, 140, 0x242C);
    for (x, y, id) in ITEMS {
        b.add_item(x, y, id, 0);
    }
    b.add_html(30, 30, 150, 75, WILT, false, false);
    match pet_name {
        Some(n) => b.add_html_styled(30, 70, 150, 25, n, centered(), true, false),
        None => b.add_html_placeholder(30, 70, 150, 25, "petName", true, false),
    }
    b.add_button(40, 105, 0x81A, 0x81B, 0x1);
    b.add_button(110, 105, 0x819, 0x818, 0x2);
}

struct DynamicTest(String);
impl DynamicGumpLogic for DynamicTest {
    const TYPE_NAME: &'static str = "Server.Tests.Gumps.DynamicTestGump";
    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        build_pet_layout(b, Some(&self.0));
    }
}

struct StaticTest;
impl StaticGumpLogic for StaticTest {
    const TYPE_NAME: &'static str = "Server.Tests.Gumps.StaticTestGump";
    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        build_pet_layout(b, Some("Test"));
    }
}

struct StaticLayoutTest(String);
impl StaticGumpLogic for StaticLayoutTest {
    const TYPE_NAME: &'static str = "Server.Tests.Gumps.StaticLayoutTestGump";
    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        build_pet_layout(b, None);
    }
    fn build_strings(&self, s: &mut GumpStrings) {
        s.set_html_text_styled("petName", &self.0, centered());
    }
}

struct LegacyTest;
impl LegacyGumpLogic for LegacyTest {
    const TYPE_NAME: &'static str = "Server.Tests.Gumps.LegacyTestGump";
}

fn legacy_pet(pet_name: &str) -> LegacyGumpObj<LegacyTest> {
    let mut g = LegacyGumpObj::new(50, 50, LegacyTest);
    g.core.serial = Serial::new(0x123);
    g.core.type_id = 0x5345;
    let l = &mut g.layout;
    l.add_page(0);
    l.add_background(10, 10, 265, 140, 0x242C);
    for (x, y, id) in ITEMS {
        l.add_item(x, y, id, 0);
    }
    l.add_html(30, 30, 150, 75, WILT, false, false);
    l.add_html(
        30,
        70,
        150,
        25,
        &format!("<CENTER>{pet_name}</CENTER>"),
        true,
        false,
    );
    l.add_button(40, 105, 0x81A, 0x81B, 0x1);
    l.add_button(110, 105, 0x819, 0x818, 0x2);
    g
}

#[test]
fn test_dynamic_gump_packet() {
    let expected = legacy_pet("Test").compile();
    let mut g = DynamicGumpObj::new(50, 50, DynamicTest("Test".into()));
    g.core.serial = Serial::new(0x123);
    g.core.type_id = 0x5345;
    assert_eq!(g.compile(), expected);
}

/// `InternalTestStaticGump`: the layout at offset 19, then the strings count 3 and the strings.
fn check_static_gump(expected_layout: &str, mut g: impl Gump, strings: [&str; 2]) {
    g.core_mut().serial = Serial::new(0x123);
    g.core_mut().type_id = 0x5345;
    let packet = g.compile();

    let mut layout = Vec::new();
    write_packed(&mut layout, expected_layout.as_bytes());
    assert_eq!(&packet[19..19 + layout.len()], layout.as_slice());
    assert_eq!(&packet[19 + layout.len()..23 + layout.len()], &[0, 0, 0, 3]);

    let mut table = vec![0u8, 0]; // the empty string
    for s in strings {
        table.extend((s.len() as u16).to_be_bytes());
        table.extend(s.encode_utf16().flat_map(u16::to_be_bytes));
    }
    let mut want = Vec::new();
    write_packed(&mut want, &table);
    assert_eq!(&packet[23 + layout.len()..], want.as_slice());
}

#[test]
fn test_static_layout_gump_packet() {
    let layout = "{ page 0 }{ resizepic 10 10 9260 265 140 }{ tilepic 205 40 4 }{ tilepic 227 40 5 }{ tilepic 180 78 3246 }{ tilepic 195 90 3245 }{ tilepic 218 95 3248 }{ htmlgump 30 30 150 75 1 0 0 }{ htmlgump 30 70 150 25 00002 1 0 }{ button 40 105 2074 2075 1 0 1 }{ button 110 105 2073 2072 1 0 2 }\0";
    check_static_gump(
        layout,
        StaticGumpObj::new(50, 50, StaticLayoutTest("Test".into())),
        [WILT, "<CENTER>Test</CENTER>"],
    );
}

#[test]
fn test_static_gump_packet() {
    let layout = "{ page 0 }{ resizepic 10 10 9260 265 140 }{ tilepic 205 40 4 }{ tilepic 227 40 5 }{ tilepic 180 78 3246 }{ tilepic 195 90 3245 }{ tilepic 218 95 3248 }{ htmlgump 30 30 150 75 1 0 0 }{ htmlgump 30 70 150 25 2 1 0 }{ button 40 105 2074 2075 1 0 1 }{ button 110 105 2073 2072 1 0 2 }\0";
    check_static_gump(
        layout,
        StaticGumpObj::new(50, 50, StaticTest),
        [WILT, "<CENTER>Test</CENTER>"],
    );
}

// A static gump rebuilds its layout on every compile (there is no per-type layout cache), so only the output of the
// second compile is checked.
#[test]
fn test_static_gump_is_cached_output_is_stable() {
    let mut g = StaticGumpObj::new(50, 50, StaticTest);
    g.core.serial = Serial::new(0x124);
    g.core.type_id = 0x5346;
    let first = g.compile();
    assert_eq!(g.compile(), first);
}

struct EmptyLegacy;
impl LegacyGumpLogic for EmptyLegacy {
    const TYPE_NAME: &'static str = "Server.Tests.Gumps.EmptyLegacyTestGump";
}
struct EmptyDynamic;
impl DynamicGumpLogic for EmptyDynamic {
    const TYPE_NAME: &'static str = "Server.Tests.Gumps.EmptyDynamicTestGump";
    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
    }
}
struct EmptyStatic;
impl StaticGumpLogic for EmptyStatic {
    const TYPE_NAME: &'static str = "Server.Tests.Gumps.EmptyStaticTestGump";
    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.set_no_close();
    }
}

#[test]
fn test_empty_gumps_have_no_visual_elements() {
    let mut g = LegacyGumpObj::new(0, 0, EmptyLegacy);
    g.compile();
    assert!(!g.core.has_visual_elements);
    let mut g = DynamicGumpObj::new(0, 0, EmptyDynamic);
    g.compile();
    assert!(!g.core.has_visual_elements);
    let mut g = StaticGumpObj::new(0, 0, EmptyStatic);
    g.compile();
    assert!(!g.core.has_visual_elements);
    g.compile();
    assert!(!g.core.has_visual_elements);
}

#[test]
fn test_visible_gumps_have_visual_elements() {
    let mut g = legacy_pet("Test");
    g.compile();
    assert!(g.core.has_visual_elements);
    let mut g = DynamicGumpObj::new(50, 50, DynamicTest("Test".into()));
    g.compile();
    assert!(g.core.has_visual_elements);
    let mut g = StaticGumpObj::new(50, 50, StaticTest);
    g.compile();
    assert!(g.core.has_visual_elements);
    g.compile();
    assert!(g.core.has_visual_elements);
}
