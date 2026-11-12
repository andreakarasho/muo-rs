//! Layout tests: the `{ .. }` text each gump compiles to, read against the gumps' `Add*` calls.

use std::rc::Rc;

use muo_content::gumps::Gump;
use muo_io::text::TextDefinition;
use muo_server::World;
use muo_server::ids::MobileId;

/// The unpacked layout and strings sections of a compiled 0xDD packet.
pub fn sections(g: &mut impl Gump) -> (String, Vec<u8>) {
    let p = g.compile();
    assert_eq!(p[0], 0xDD);
    assert_eq!(u16::from_be_bytes([p[1], p[2]]) as usize, p.len());
    let mut at = 19;
    let read = |at: &mut usize| {
        let len = i32::from_be_bytes(p[*at..*at + 4].try_into().unwrap()) as usize;
        if len == 0 {
            *at += 4;
            return Vec::new();
        }
        let raw = i32::from_be_bytes(p[*at + 4..*at + 8].try_into().unwrap()) as usize;
        let mut out = vec![0u8; raw];
        muo_io::compression::STANDARD
            .unpack(&mut out, &p[*at + 8..*at + 4 + len])
            .unwrap();
        *at += 4 + len;
        out
    };
    let layout = read(&mut at);
    at += 4; // the strings count
    let strings = read(&mut at);
    (
        String::from_utf8_lossy(&layout)
            .trim_end_matches('\0')
            .to_owned(),
        strings,
    )
}

pub fn layout_of(g: &mut impl Gump) -> String {
    sections(g).0
}

/// The text of a string table of `u16 length + UTF-16BE` entries.
pub fn strings_of(raw: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 2 <= raw.len() {
        let n = u16::from_be_bytes([raw[i], raw[i + 1]]) as usize;
        i += 2;
        let units: Vec<u16> = (0..n)
            .map(|k| u16::from_be_bytes([raw[i + 2 * k], raw[i + 2 * k + 1]]))
            .collect();
        i += 2 * n;
        out.push(String::from_utf16_lossy(&units));
    }
    out
}

#[test]
fn young_gumps_lay_out_as_expected() {
    let l = layout_of(&mut super::simple::new_young_dungeon_warning());
    assert_eq!(
        l,
        "{ resizepic 0 0 302 250 170 }{ resizepic 10 10 5120 230 150 }\
         { xmfhtmlgumpcolor 20 25 210 70 1018030 0 0 32767 }{ button 163 125 248 249 1 0 0 }"
    );

    let l = layout_of(&mut super::simple::new_young_death_notice());
    assert!(
        l.starts_with(
            "{ noclose }{ resizepic 25 10 302 425 444 }{ resizepic 35 20 5120 405 424 }{ checkertrans 25 10 425 444 }"
        ),
        "{l}"
    );
    assert!(
        l.contains("{ xmfhtmlgumpcolor 190 34 120 20 1046287 0 0 32000 }"),
        "{l}"
    ); // 0x7D00
    assert!(l.ends_with("{ button 363 409 248 249 1 0 0 }"), "{l}");

    let l = layout_of(&mut super::simple::new_renounce_young());
    assert_eq!(
        l,
        "{ resizepic 0 0 302 450 330 }{ resizepic 10 10 5120 430 310 }\
         { xmfhtmlgumpcolor 0 24 450 20 1013004 0 0 32767 }{ xmfhtmlgumpcolor 25 50 380 210 1013005 0 0 32767 }\
         { button 363 285 248 249 1 0 1 }{ button 283 285 241 242 1 0 0 }"
    );
}

#[test]
fn polymorph_has_a_radio_per_entry_numbered_by_category_and_slot() {
    let l = layout_of(&mut super::simple::new_polymorph(None));
    assert!(
        l.starts_with("{ page 0 }{ resizepic 0 0 5054 585 393 }{ resizepic 195 36 3000 387 275 }"),
        "{l}"
    );
    assert!(l.contains("{ button 25 355 4005 4007 1 1 1 }{ xmfhtmlgump 320 355 150 18 1011012 0 0 }{ button 285 355 4005 4007 1 2 0 }"), "{l}");
    // two category buttons that turn the page
    assert!(
        l.contains("{ xmfhtmlgump 5 35 150 25 1015235 1 0 }{ button 155 35 4005 4007 0 1 0 }"),
        "{l}"
    );
    assert!(
        l.contains("{ xmfhtmlgump 5 60 150 25 1015245 1 0 }{ button 155 60 4005 4007 0 2 0 }"),
        "{l}"
    );
    // switch ids are (slot << 8) + category
    assert!(l.contains("{ radio 198 58 210 211 0 0 }"), "{l}");
    assert!(l.contains("{ radio 327 58 210 211 0 256 }"), "{l}");
    assert!(l.contains("{ radio 456 192 210 211 0 2048 }"), "{l}");
    assert!(l.contains("{ radio 198 58 210 211 0 1 }"), "{l}");
    assert_eq!(l.matches("{ radio").count(), 18, "{l}");
}

#[test]
fn new_polymorph_pages_by_ten() {
    let l = layout_of(&mut super::simple::new_new_polymorph(None));
    assert_eq!(l.matches("{ page ").count(), 3, "{l}"); // 0, 1, 2
    assert_eq!(l.matches("{ buttontileart").count(), 18, "{l}");
    assert!(l.contains("{ button 400 374 4005 4007 0 2 0 }"), "{l}");
    assert!(
        l.contains("{ page 2 }{ button 300 374 4014 4016 0 1 0 }"),
        "{l}"
    );
    assert!(
        l.contains("{ buttontileart 14 44 2328 2329 1 0 1 8401 0 15 10 }"),
        "{l}"
    );
    assert!(
        l.contains("{ buttontileart 264 44 2328 2329 1 0 2 8405 0 17 10 }"),
        "{l}"
    );
}

#[test]
fn tot_admin_has_a_button_per_era_and_drop_or_reward() {
    let l = layout_of(&mut super::simple::new_tot_admin(1, 3));
    assert!(
        l.starts_with("{ noresize }{ page 0 }{ resizepic 0 0 9200 320 150 }"),
        "{l}"
    );
    assert!(l.contains("{ button 75 50 2361 2361 1 0 1 }"), "{l}"); // activated
    assert!(l.contains("{ button 75 75 2361 2361 1 0 2 }"), "{l}"); // drop era 1 is active
    assert!(l.contains("{ button 180 75 2360 2360 1 0 3 }"), "{l}");
    assert!(l.contains("{ button 180 125 2361 2361 1 0 7 }"), "{l}"); // reward era 3 is active
    assert!(l.contains("{ button 285 120 4005 4006 0 4 2 }"), "{l}");
    assert_eq!(l.matches("{ htmlgump 330 10 180 130").count(), 4, "{l}");
}

#[test]
fn warning_gump_is_centered_and_has_okay_and_cancel() {
    let mut g = super::warning::new(TextDefinition::of_number(1060001), 300, 200, None, true);
    let l = layout_of(&mut g);
    assert_eq!((g.core.x, g.core.y), (170, 140));
    assert_eq!(
        l,
        "{ noclose }{ page 0 }{ resizepic 0 0 5054 300 200 }{ gumppictiled 10 10 280 20 2624 }\
         { checkertrans 10 10 280 20 }{ xmfhtmlgumpcolor 10 10 280 20 1060635 0 0 30720 }\
         { gumppictiled 10 40 280 120 2624 }{ checkertrans 10 40 280 120 }\
         { xmfhtmlgumpcolor 10 40 280 120 1060001 0 1 16760832 }{ gumppictiled 10 170 280 20 2624 }\
         { checkertrans 10 170 280 20 }{ button 10 170 4005 4007 1 0 1 }\
         { xmfhtmlgumpcolor 40 170 170 20 1011036 0 0 32767 }{ button 150 170 4005 4007 1 0 0 }\
         { xmfhtmlgumpcolor 180 170 170 20 1011012 0 0 32767 }"
    );
}

#[test]
fn notice_gump_has_no_cancel_and_its_own_type_id() {
    let mut n = super::warning::new_notice(TextDefinition::of_text("hi"), 300, 200, None);
    let mut w = super::warning::new(TextDefinition::of_text("hi"), 300, 200, None, false);
    let l = layout_of(&mut n);
    assert!(l.contains("1060637"), "{l}");
    assert!(!l.contains("4005 4007 1 0 0 }"), "{l}");
    assert_ne!(n.core.type_id, w.core.type_id);
    assert_eq!(layout_of(&mut w).replace("1060635", "1060637"), l);
}

#[test]
fn who_gump_with_nobody_is_a_frame_and_a_page_label() {
    let w = World::new();
    let mut g = super::who::new(&w, Rc::new(Vec::<MobileId>::new()), 0);
    let (l, strings) = sections(&mut g);
    assert_eq!(
        l,
        "{ page 0 }{ resizepic 0 0 5054 223 42 }{ gumppictiled 10 10 203 22 2624 }\
         { gumppictiled 11 11 159 20 3004 }{ text 13 11 0 0 }{ gumppictiled 171 11 20 20 3604 }\
         { gumppictiled 192 11 20 20 3604 }"
    );
    assert_eq!(strings_of(&strings), ["Page 1 of 0 (0)"]);
}

#[test]
fn props_frame_matches_the_expected_arithmetic() {
    use muo_content::gumps::props_layout::*;
    let mut l = muo_content::gumps::legacy::LegacyLayout::default();
    let (x, y) = add_props_frame(&mut l, 100, 3);
    assert_eq!((x, y), (11, 11));
    // totalHeight = 1 + (20 + 1) * 3 = 64
    let text = String::from_utf8_lossy(&l.compile().layout).into_owned();
    assert!(
        text.starts_with("{ resizepic 0 0 5054 120 84 }{ gumppictiled 10 10 100 64 2624 }"),
        "{text}"
    );
}

#[test]
fn the_legacy_and_builder_headers_differ_only_in_the_title() {
    use muo_content::gumps::props_layout::*;
    let mut legacy = muo_content::gumps::legacy::LegacyLayout::default();
    let (mut x, mut y) = (11, 11);
    add_props_header(
        &mut legacy,
        203,
        &mut x,
        &mut y,
        &Header::new(Some("T"), Some(1), Some(2)),
    );
    let l = String::from_utf8_lossy(&legacy.compile().layout).into_owned();
    // [Prev] [Title:159 wide, html at the cell's origin] [Next]
    assert!(l.contains("{ button 13 13 5603 5607 1 0 1 }"), "{l}");
    assert!(l.contains("{ htmlgump 32 11 159 20 0 0 0 }"), "{l}");
    assert!(l.contains("{ button 194 13 5601 5605 1 0 2 }"), "{l}");

    let mut b = muo_content::gumps::DynamicGumpBuilder::new_dynamic();
    let (mut x, mut y) = (11, 11);
    add_props_header(
        &mut b,
        203,
        &mut x,
        &mut y,
        &Header::new(Some("T"), Some(1), Some(2)),
    );
    b.finalize_layout();
    let l = String::from_utf8_lossy(b.layout_data()).into_owned();
    // the builder's title is centered by html and starts two pixels right
    assert!(l.contains("{ htmlgump 34 11 157 20 0 0 0 }"), "{l}");
    let strings = strings_of(b.strings_data());
    assert_eq!(strings, ["<CENTER>T</CENTER>"], "{strings:?}");
}

#[test]
fn track_what_gump_lays_out_as_expected() {
    let l = layout_of(&mut super::tracking::new_what());
    assert!(l.starts_with("{ page 0 }{ resizepic 0 0 5054 440 135 }{ resizepic 10 10 2620 420 75 }{ resizepic 10 85 3000 420 25 }{ tilepic 20 20 9682 }{ button 20 110 4005 4007 1 0 1 }"), "{l}");
    assert!(l.contains("{ button 320 110 4005 4007 1 0 4 }"), "{l}");
}

#[test]
fn jail_record_gump_fills_its_slots() {
    let w = World::new();
    let mut g = super::jail::new(
        &w,
        MobileId(muo_primitives::Serial::new(1)),
        &muo_content::jail::JailRecord::default(),
    );
    let (l, s) = sections(&mut g);
    assert!(
        l.starts_with("{ noresize }{ page 0 }{ resizepic 0 0 9200 330 300 }"),
        "{l}"
    );
    let strs = strings_of(&s);
    assert!(strs.iter().any(|t| t.contains("JAIL RECORD")), "{strs:?}");
    assert!(
        strs.iter().any(|t| t.contains("Last Jailed: Never")),
        "{strs:?}"
    );
    assert!(
        strs.iter().any(|t| t.contains("Status: Not Jailed")),
        "{strs:?}"
    );
}
