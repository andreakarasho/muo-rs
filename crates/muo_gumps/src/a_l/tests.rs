//! Tests of the A to L gumps: layouts compiled to the 0xDD packet and read back.

use muo_content::gumps::base::{Gump, RelayInfo};
use muo_primitives::Serial;

use super::*;

/// The layout section of a compiled 0xDD packet.
fn layout_of(g: &mut dyn Gump) -> String {
    g.core_mut().serial = Serial::new(1);
    let p = g.compile();
    let len = i32::from_be_bytes(p[19..23].try_into().unwrap());
    assert!(len > 0);
    let raw_len = i32::from_be_bytes(p[23..27].try_into().unwrap()) as usize;
    let mut out = vec![0u8; raw_len];
    muo_io::compression::STANDARD
        .unpack(&mut out, &p[27..19 + 4 + len as usize])
        .unwrap();
    String::from_utf8_lossy(&out)
        .trim_end_matches('\0')
        .to_owned()
}

#[test]
fn add_door_first_page_lists_the_standard_doors() {
    let mut g = add_door::new();
    let l = layout_of(&mut g);
    // AddBlueBack(20 + (6 + 1) * 50, 165)
    assert!(l.starts_with("{ page 0 }{ resizepic 0 0 3600 370 165 }{ resizepic 8 5 83 354 154 }"));
    assert!(l.contains("{ button 80 20 9764 9765 1 0 10 }"));
    assert!(l.contains("{ tilepic 65 30 1653 }"));
    assert!(l.contains("{ button 330 20 9764 9765 1 0 15 }"));
}

#[test]
fn base_confirm_layout_and_response() {
    struct Crystal;
    impl muo_content::gumps::base::LegacyGumpLogic for Crystal {
        const TYPE_NAME: &'static str = "Server.Gumps.Test";
        fn on_response(
            &mut self,
            _: &mut muo_server::World,
            _: muo_server::net_state::NetStateId,
            _: &RelayInfo,
        ) {
        }
    }
    impl base_confirm::ConfirmHandler for Crystal {}
    let mut g = base_confirm::build(Crystal);
    let l = layout_of(&mut g);
    assert!(l.contains("{ checkertrans 0 0 348 262 }"));
    assert!(l.contains("{ radio 25 175 9720 9723 1 1 }"));
    assert!(l.contains("{ radio 25 210 9720 9723 0 0 }"));
    assert!(l.contains("{ button 265 220 247 248 1 0 2 }"));
    // Closable = false: { noclose } leads the layout
    assert!(l.starts_with("{ noclose }"), "{l}");
}

#[test]
fn heritage_token_pages_ten_items_each() {
    let mut g = muo_content::gumps::StaticGumpObj::new(
        60,
        36,
        heritage_token::HeritageTokenGump::for_test(),
    );
    let l = layout_of(&mut g);
    assert!(l.contains("{ page 7 }"));
    assert!(!l.contains("{ page 8 }"));
    // first entry
    assert!(l.contains("{ buttontileart 14 44 2328 2329 1 0 100 5137 44 18 8 }"));
    // the first page of a 67 item list has Next but no Back
    assert!(l.contains("{ button 400 374 4005 4007 0 2 0 }"), "{l}");
}

#[test]
fn image_tile_buttons_response_maps_ids() {
    assert_eq!(base_image_tile_buttons::adjusted_button(101, 5), Some(1));
}

#[test]
fn base_grid_sizes_its_page() {
    use muo_content::gumps::legacy::LegacyLayout;
    let mut l = LegacyLayout::default();
    let mut grid = muo_content::gumps::base_grid::BaseGrid::default();
    grid.add_new_page(&mut l);
    grid.add_entry_label(&mut l, 100, "a");
    grid.add_new_line();
    grid.add_entry_label(&mut l, 60, "b");
    grid.finish_page(&mut l);
    // the background grew to the widest line plus the border, the height to the last line plus the border
    match &l.entries[1] {
        muo_content::gumps::legacy::GumpEntry::Background { width, height, .. } => {
            assert_eq!(*width, 11 + 100 + 1 + 10);
            assert_eq!(*height, 11 + 21 + 20 + 1 + 10);
        }
        e => panic!("{e:?}"),
    }
}

#[test]
fn go_data_files_parse_and_link_parents() {
    use muo_server::map::MapId;
    let dir = std::path::PathBuf::from(std::env::var_os("MUO_DISTRIBUTION").unwrap_or_default())
        .join("Data/Locations");
    for (file, map) in [
        ("felucca", MapId::FELUCCA),
        ("ilshenar", MapId::ILSHENAR),
        ("termur", MapId::TER_MUR),
    ] {
        let Ok(text) = std::fs::read_to_string(dir.join(format!("{file}.json"))) else {
            return;
        };
        let t = go::parse_tree(map, &text).unwrap();
        let root = t.root.unwrap();
        assert_eq!(t.nodes[root].parent, None);
        for (i, n) in t.nodes.iter().enumerate() {
            for c in &n.categories {
                assert_eq!(t.nodes[*c].parent, Some(i));
            }
        }
        assert!(!t.nodes[root].categories.is_empty());
    }
}

#[test]
fn go_gump_shows_back_and_one_row_for_a_child_node() {
    use muo_server::map::MapId;
    let json = r#"{"name":"Root","categories":[{"name":"Towns","locations":[{"name":"Britain","location":[1,2,3]}]}],
        "locations":[{"name":"A","location":[1,1,1]}]}"#;
    let t = std::rc::Rc::new(go::parse_tree(MapId::FELUCCA, json).unwrap());
    let mut g = muo_content::gumps::DynamicGumpObj::new(
        50,
        50,
        go::GoGump::for_test(t.clone(), t.nodes[0].categories[0], 0),
    );
    let l = layout_of(&mut g);
    // the header's back button (id 1) and the row's set button (index 0 + 4)
    assert!(
        l.contains(" 5603 5607 1 0 1 }") || l.contains(" 5603 5607 1"),
        "{l}"
    );
    assert!(
        l.contains(" 5601 5605 1 0 4 }") || l.contains(" 5601 5605 4"),
        "{l}"
    );
}

#[test]
fn ban_duration_spans() {
    // `new TimeSpan(1, 2, 3, 4)`
    assert_eq!(
        ban_duration::time_span_for_test(1, 2, 3, 4)
            .unwrap()
            .ticks(),
        (86400 + 7200 + 180 + 4) * 10_000_000
    );
}

#[test]
fn house_gumps_layouts() {
    let mut g = muo_content::gumps::StaticGumpObj::new(
        110,
        100,
        houses::ConfirmDemolishHouseGump::for_test(),
    );
    let l = layout_of(&mut g);
    assert!(l.starts_with("{ noclose }{ page 0 }{ resizepic 0 0 5054 420 280 }"));
    assert!(l.contains("{ button 10 250 4005 4007 1 0 1 }"));
}

fn some_mobile() -> muo_server::ids::MobileId {
    muo_server::ids::MobileId(Serial::new(0x1000))
}

// Golden layouts: each expected string is derived by hand from the gump's layout calls (constants resolved: 0x24AE = 9390,
// 0x2486 = 9350, 0x481 = 1153, 0x845/0x846 = 2117/2118, ...), with the builder's calls in source order.

#[test]
fn golden_war_declaration_gump() {
    let w = muo_server::World::new();
    let mut g = new_guilds::war_declaration(
        &w,
        some_mobile(),
        muo_server::guild::GuildId(5),
        muo_server::guild::GuildId(6),
    );
    let expected = "{ page 0 }{ resizepic 0 0 9390 500 340 }{ resizepic 65 50 9350 370 30 }\
{ xmfhtmlgumpcolor 75 55 370 26 1062979 0 0 15360 }{ gumppic 410 45 9004 }\
{ xmfhtmlgumpcolor 65 95 200 20 1063009 0 0 5295 }{ xmfhtmlgumpcolor 65 120 400 20 1063010 0 0 0 }\
{ resizepic 65 150 9350 40 30 }{ textentry 70 154 50 30 1153 10 0 }\
{ xmfhtmlgumpcolor 65 195 200 20 1063011 0 0 5295 }{ xmfhtmlgumpcolor 65 220 400 20 1063012 0 0 0 }\
{ resizepic 65 250 9350 40 30 }{ textentry 70 254 50 30 1153 11 1 }\
{ resizepic 190 270 9350 130 26 }{ button 195 275 2117 2118 1 0 0 }{ xmfhtmlgumpcolor 220 273 90 26 1006045 0 0 0 }\
{ resizepic 330 270 9350 130 26 }{ button 335 275 2117 2118 1 0 1 }{ xmfhtmlgumpcolor 360 273 90 26 1062989 0 0 20480 }";
    assert_eq!(layout_of(&mut g), expected);
}

#[test]
fn golden_create_guild_gump() {
    let mut w = muo_server::World::new();
    let mut g = new_guilds::create_guild_gump(&mut w, some_mobile(), "Guild Name", "");
    let expected = "{ page 0 }{ resizepic 0 0 9250 500 300 }{ xmfhtmlgumpcolor 25 20 450 25 1062939 1 0 0 }\
{ xmfhtmlgumpcolor 25 60 450 60 1062940 0 0 0 }{ xmfhtmlgumpcolor 25 135 120 25 1062941 0 0 0 }\
{ text 155 135 1153 0 }{ xmfhtmlgumpcolor 25 165 120 25 1011140 0 0 0 }{ resizepic 155 160 3000 320 26 }\
{ textentry 160 163 315 21 1153 5 1 }{ xmfhtmlgumpcolor 25 191 120 26 1063035 0 0 0 }\
{ resizepic 155 186 3000 320 26 }{ textentry 160 189 315 21 1153 6 2 }\
{ button 415 217 247 248 1 0 1 }{ button 345 217 242 241 1 0 0 }{ button 20 260 211 210 1 0 2 }\
{ xmfhtmlgumpcolor 45 260 200 30 1062943 0 0 0 }";
    assert_eq!(layout_of(&mut g), expected);
}

#[test]
fn golden_admin_gump_server_page() {
    let mut w = muo_server::World::new();
    let mut g = admin::new(
        &mut w,
        some_mobile(),
        admin::AdminGumpPage::AdministerServer,
        0,
        None,
        None,
        admin::AdminState::None,
    );
    let expected = "{ page 0 }{ resizepic 0 0 5054 420 480 }\
{ gumppictiled 10 10 170 100 2624 }{ checkertrans 10 10 170 100 }\
{ gumppictiled 190 10 220 100 2624 }{ checkertrans 190 10 220 100 }\
{ gumppictiled 10 120 400 300 2624 }{ checkertrans 10 120 400 300 }\
{ gumppictiled 10 430 400 40 2624 }{ checkertrans 10 430 400 40 }\
{ button 10 9 4005 4007 1 0 1 }{ htmlgump 45 10 200 20 0 0 0 }\
{ button 10 29 4006 4007 1 0 12 }{ htmlgump 45 30 200 20 1 0 0 }\
{ button 10 49 4005 4007 1 0 23 }{ htmlgump 45 50 200 20 2 0 0 }\
{ button 10 69 4005 4007 1 0 34 }{ htmlgump 45 70 200 20 3 0 0 }\
{ button 10 89 4005 4007 1 0 45 }{ htmlgump 45 90 200 20 4 0 0 }\
{ htmlgump 10 125 400 20 5 0 0 }\
{ button 20 149 4005 4007 1 0 2204 }{ htmlgump 55 150 240 20 6 0 0 }\
{ button 20 179 4005 4007 1 0 2215 }{ htmlgump 55 180 240 20 7 0 0 }\
{ button 20 199 4005 4007 1 0 2226 }{ htmlgump 55 200 240 20 8 0 0 }\
{ button 20 229 4005 4007 1 0 2237 }{ htmlgump 55 230 240 20 9 0 0 }\
{ button 20 249 4005 4007 1 0 2248 }{ htmlgump 55 250 240 20 10 0 0 }\
{ button 20 269 4005 4007 1 0 2259 }{ htmlgump 55 270 240 20 11 0 0 }\
{ htmlgump 10 295 400 20 12 0 0 }{ resizepic 18 318 9350 384 24 }{ textentry 22 322 376 16 0 0 13 }\
{ button 20 349 4005 4007 1 0 2314 }{ htmlgump 55 350 240 20 14 0 0 }\
{ button 220 349 4005 4007 1 0 2325 }{ htmlgump 255 350 240 20 15 0 0 }\
{ button 200 19 4005 4007 1 0 4 }{ htmlgump 235 20 200 20 16 0 0 }\
{ button 200 39 4006 4007 1 0 15 }{ htmlgump 235 40 200 20 17 0 0 }\
{ button 200 59 4005 4007 1 0 26 }{ htmlgump 235 60 200 20 18 0 0 }\
{ button 200 79 4005 4007 1 0 37 }{ htmlgump 235 80 200 20 19 0 0 }";
    assert_eq!(layout_of(&mut g), expected);
}

#[test]
fn admin_button_ids_and_formats() {
    assert_eq!(admin::get_button_id(3, 200), 2204);
    assert_eq!(
        admin::format_access_level(muo_server::mobile::AccessLevel::GameMaster),
        "Game Master"
    );
    assert_eq!(admin::format_byte_amount(2048), "2.0 KB");
    assert_eq!(admin::format_byte_amount(500), "500 Bytes");
}

#[test]
fn interface_gump_pages_and_marks_deleted_objects() {
    use muo_server::ids::{EntityId, ItemId, MobileId};
    let mut w = muo_server::World::new();
    // 17 objects that no longer exist: two pages, the first lists 15 "(deleted)" rows and a next arrow
    let list = (0..17)
        .map(|i| EntityId::Item(ItemId(Serial::new(0x4000_0000 + i))))
        .collect();
    let mut g =
        interface::new_list_for_test(&mut w, MobileId(Serial::new(1)), list, vec!["Name".into()]);
    let l = layout_of(&mut g);
    // text goes to the string table; the layout shows the next arrow (page 1 has no previous) and 15 rows of 20
    assert!(l.contains("{ button 315 13 5601 5605 1 0 2 }"), "{l}");
    assert!(!l.contains("5603"), "{l}");
    assert_eq!(l.matches("{ htmlgump").count(), 1 + 2 + 15 * 2); // page text, two column titles
    assert!(l.contains("{ gumppictiled 313 347 20 20 3604 }"), "{l}");
    w.mobiles.clear();
}

/// A mobile of `level` for the response tests, without a connection.
fn staff(
    w: &mut muo_server::World,
    level: muo_server::mobile::AccessLevel,
) -> muo_server::ids::MobileId {
    let m = w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE);
    w.mobile_mut(m).unwrap().access_level = level;
    m
}

#[test]
fn admin_gump_account_access_change_needs_a_level_below_the_admin() {
    use muo_server::mobile::AccessLevel as L;
    let mut w = muo_server::World::new();
    let admin_m = staff(&mut w, L::Administrator);
    let a = muo_content::accounting::create_account(&mut w, "target", "pw");
    let ns = muo_server::net_state::NetStateId(1);
    // button = index 11-based: type 5 (accounts), index 22 = GameMaster, 24 = Administrator
    let press = |w: &mut muo_server::World, index: i32| {
        let mut g = admin::new(
            w,
            admin_m,
            admin::AdminGumpPage::AccountDetailsChangeAccess,
            0,
            None,
            None,
            admin::AdminState::Account(a),
        );
        g.on_response(
            w,
            ns,
            &RelayInfo::new(admin::get_button_id(5, index), vec![], vec![]),
        );
    };
    press(&mut w, 24); // not below Administrator: nothing happens
    assert_eq!(
        muo_content::accounting::account(&w, a)
            .unwrap()
            .access_level,
        L::Player
    );
    press(&mut w, 22);
    assert_eq!(
        muo_content::accounting::account(&w, a)
            .unwrap()
            .access_level,
        L::GameMaster
    );
}

/// A mobile whose connection is `NetStateId(1)`, so sent gumps land in the gump system.
fn connected(
    w: &mut muo_server::World,
    m: muo_server::ids::MobileId,
) -> muo_server::net_state::NetStateId {
    let ns = muo_server::net_state::NetStateId(1);
    let mut d = muo_server::net_state::NetStateData::new(0);
    d.mobile = Some(m);
    w.net_states.insert(ns, d);
    w.mobile_mut(m).unwrap().net_state = Some(ns);
    ns
}

#[test]
fn admin_add_tag_value_prompt_sends_the_gump_only_with_an_account() {
    use muo_server::mobile::AccessLevel as L;
    let mut w = muo_server::World::new();
    let m = staff(&mut w, L::Administrator);
    let ns = connected(&mut w, m);
    let a = muo_content::accounting::create_account(&mut w, "tagged", "pw");
    let respond = |w: &mut muo_server::World, acct| {
        admin::add_tag_value_prompt(w, m, acct, "k".into());
        let id = w.mobile(m).unwrap().prompt.unwrap();
        muo_server::prompt::on_response(w, id, m, "v");
    };
    respond(&mut w, None); // `m_Account != null` fails: nothing is sent
    assert_eq!(muo_content::gumps::system::gumps_of(&w, ns).count(), 0);
    respond(&mut w, Some(a));
    assert_eq!(muo_content::gumps::system::gumps_of(&w, ns).count(), 1);
}

/// The cliloc numbers of the `SendLocalizedMessage` packets (0xC1) sent so far.
fn clilocs_sent(w: &muo_server::World) -> Vec<u32> {
    w.net_out
        .iter()
        .filter_map(|o| match o {
            muo_server::net_state::NetOut::Send(_, b) if b[0] == 0xC1 => {
                Some(u32::from_be_bytes(b[14..18].try_into().unwrap()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn war_declaration_says_the_proposal_was_sent_also_when_it_was_updated() {
    use muo_server::mobile::AccessLevel as L;
    let mut w = muo_server::World::new();
    let pm = staff(&mut w, L::Player);
    w.mobile_mut(pm).unwrap().player = true;
    muo_content::player_mobile::data_mut(&mut w, pm); // the PlayerData the guild rank lives in
    let ns = connected(&mut w, pm);
    let other_leader = staff(&mut w, L::Player);
    let g = muo_content::misc::guild_class::new_guild(&mut w, pm, "Alpha", "AAA");
    let other = muo_content::misc::guild_class::new_guild(&mut w, other_leader, "Beta", "BBB");
    let press = |w: &mut muo_server::World| {
        let mut gump = new_guilds::war_declaration(w, pm, g, other);
        gump.on_response(
            w,
            ns,
            &RelayInfo::new(1, vec![], vec![(10, "1".into()), (11, "5".into())]),
        );
    };
    press(&mut w);
    assert_eq!(clilocs_sent(&w), vec![1070751]); // a new proposal: the guild is told, then the sender
    press(&mut w);
    assert_eq!(clilocs_sent(&w), vec![1070751, 1070752, 1070751]); // updated, and still "sent to"
}
