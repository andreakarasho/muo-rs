//! `RunebookGump`. The book is behind [`RunebookOps`](super::ops::RunebookOps).

use std::rc::Rc;

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_primitives::utility::fix_html;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::prompt::{Prompt, add_prompt};

use super::common::{mobile_of, send_singleton};
use super::hooks::ops;
use super::ops::{RuneSpell, RunebookOps, RunebookView, SextantView};

pub struct RunebookGump {
    book: RunebookView,
    /// `Core.AOS`.
    aos: bool,
}

/// `new RunebookGump(book)` sent to `to` (`book.SendGumpTo`).
pub fn display_to(w: &mut World, to: MobileId, book: RunebookView) {
    let aos = muo_content::core::aos(w);
    let g = DynamicGumpObj::new(150, 200, RunebookGump { book, aos });
    send_singleton(w, to, g);
}

/// `toCheck.FindGump<RunebookGump>()?.Book`: the book of the runebook gump a mobile has open.
pub fn open_book(w: &World, of: MobileId) -> Option<muo_server::ids::ItemId> {
    let ns = w.mobile(of)?.net_state?;
    muo_content::gumps::system::find_gump::<RunebookGump>(w, ns).map(|g| g.book.book)
}

/// `from.HasGump<RunebookGump>()`.
pub fn has_open_gump(w: &World, of: MobileId) -> bool {
    w.mobile(of)
        .and_then(|m| m.net_state)
        .is_some_and(|ns| muo_content::gumps::system::has_gump::<RunebookGump>(w, ns))
}

/// `m.CloseGump<RunebookGump>()`.
pub fn close_open_gump(w: &mut World, of: MobileId) {
    if let Some(ns) = w.mobile(of).and_then(|m| m.net_state) {
        muo_content::gumps::system::close_gump::<RunebookGump>(w, ns);
    }
}

/// `RunebookGump.GetMapHue(map)` for the map's index (`None` is a null map).
pub fn get_map_hue(map_index: Option<i32>) -> i32 {
    match map_index {
        Some(1) => 10,   // Trammel
        Some(0) => 81,   // Felucca
        Some(2) => 1102, // Ilshenar
        Some(3) => 1102, // Malas
        Some(4) => 1154, // Tokuno
        _ => 0,
    }
}

/// `RunebookGump.GetName(name)`.
pub fn get_name(name: &str) -> String {
    if name.is_empty() {
        "(indescript)".to_owned()
    } else {
        name.trim().to_owned()
    }
}

fn hue_of(e: &super::ops::RunebookEntryView) -> i32 {
    get_map_hue(e.map.map(|_| e.map_index))
}

impl DynamicGumpLogic for RunebookGump {
    const TYPE_NAME: &'static str = "Server.Gumps.RunebookGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        self.add_background(b);
        self.add_index(b);

        for page in 0..8 {
            b.add_page(2 + page);

            b.add_button_type(125, 14, 2205, 2205, 0, GumpButtonType::Page, 1 + page);

            if page < 7 {
                b.add_button_type(393, 14, 2206, 2206, 0, GumpButtonType::Page, 3 + page);
            }

            for half in 0..2 {
                self.add_details(b, (page * 2 + half) as usize, half);
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let Some(o) = ops(w).runebooks else { return };
        let book = self.book.book;

        o.remove_opener(w, book, from);

        let range = if muo_content::core::ml(w) { 3 } else { 1 };
        let loc = o.world_location(w, book);
        if o.deleted(w, book)
            || !muo_content::mx::in_range_point(w, from, loc, range)
            || !o.design_context_check(w, from)
        {
            return;
        }

        let access = muo_content::mx::access_level(w, from);
        let mut button_id = info.button_id;

        if button_id == 1 {
            // Rename book
            if !o.is_locked_down(w, book) || access >= AccessLevel::GameMaster {
                muo_content::mx::msg(w, from, 502414); // Please enter a title for the runebook:
                begin_rename_prompt(w, from, o.clone(), book);
            } else {
                // That cannot be done while the book is locked down.
                muo_server::mobile::part2::messages::send_localized_message(
                    w, from, 502413, "", 0x35,
                );
            }

            return;
        }

        button_id -= 2;

        let (index, kind) = (button_id / 6, button_id % 6);

        if index < 0 || index as usize >= o.entry_count(w, book) {
            return;
        }
        let index = index as usize;

        match kind {
            0 => {
                // Use charges
                if o.cur_charges(w, book) <= 0 {
                    muo_content::mx::msg(w, from, 502412); // There are no charges left on that item.
                } else {
                    send_target_coordinates(w, &*o, book, index, from);

                    o.on_travel(w, book);
                    o.cast(w, book, from, index, RuneSpell::RecallCharge);
                }
            }
            1 => {
                // Drop rune
                if !o.is_locked_down(w, book) || access >= AccessLevel::GameMaster {
                    o.drop_rune(w, book, from, index);

                    if !muo_content::core::ml(w) {
                        o.send_gump_to(w, book, from);
                    }
                } else {
                    // That cannot be done while the book is locked down.
                    muo_server::mobile::part2::messages::send_localized_message(w, from, 502413, "", 0x35);
                }
            }
            2 => {
                // Set default
                if o.check_access(w, book, from) {
                    o.set_default(w, book, index);

                    // Resend the runebook gump
                    o.send_gump_to(w, book, from);

                    muo_content::mx::msg(w, from, 502417); // New default location set.
                }
            }
            3 => {
                // Recall
                if !o.has_spell(w, from, 31) {
                    muo_content::mx::msg(w, from, 500015); // You do not have that spell!
                    return;
                }

                send_target_coordinates(w, &*o, book, index, from);

                o.on_travel(w, book);
                o.cast(w, book, from, index, RuneSpell::Recall);
            }
            4 => {
                // Gate
                if !o.has_spell(w, from, 51) {
                    muo_content::mx::msg(w, from, 500015); // You do not have that spell!
                    return;
                }

                send_target_coordinates(w, &*o, book, index, from);

                o.on_travel(w, book);
                o.cast(w, book, from, index, RuneSpell::Gate);
            }
            5
                // Sacred Journey
                if muo_content::core::aos(w) => {
                    if !o.has_spell(w, from, 209) {
                        muo_content::mx::msg(w, from, 500015); // You do not have that spell!
                        o.remove_opener(w, book, from);
                        return;
                    }

                    send_target_coordinates(w, &*o, book, index, from);

                    o.on_travel(w, book);
                    o.cast(w, book, from, index, RuneSpell::SacredJourney);
                }
            _ => {}
        }
    }
}

impl RunebookGump {
    fn add_background(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);

        // Background image
        b.add_image(100, 10, 2200, 0);

        // Two separators
        for i in 0..2 {
            let mut x_offset = 125 + i * 165;

            b.add_image(x_offset, 50, 57, 0);
            x_offset += 20;

            for _ in 0..6 {
                b.add_image(x_offset, 50, 58, 0);
                x_offset += 15;
            }

            b.add_image(x_offset - 5, 50, 59, 0);
        }

        // First four page buttons
        let (mut x_offset, mut gump_id) = (130, 2225);
        for i in 0..4 {
            b.add_button_type(
                x_offset,
                187,
                gump_id,
                gump_id,
                0,
                GumpButtonType::Page,
                2 + i,
            );
            x_offset += 35;
            gump_id += 1;
        }

        // Next four page buttons
        let (mut x_offset, mut gump_id) = (300, 2229);
        for i in 0..4 {
            b.add_button_type(
                x_offset,
                187,
                gump_id,
                gump_id,
                0,
                GumpButtonType::Page,
                6 + i,
            );
            x_offset += 35;
            gump_id += 1;
        }

        // Charges
        b.add_html_localized(140, 40, 80, 18, 1011296, false, false); // Charges:
        b.add_html(
            220,
            40,
            30,
            18,
            &self.book.cur_charges.to_string(),
            false,
            false,
        );

        // Max charges
        b.add_html_localized(300, 40, 100, 18, 1011297, false, false); // Max Charges:
        b.add_html(
            400,
            40,
            30,
            18,
            &self.book.max_charges.to_string(),
            false,
            false,
        );
    }

    fn add_index(&self, b: &mut DynamicGumpBuilder) {
        // Index
        b.add_page(1);

        // Rename button
        b.add_button(125, 15, 2472, 2473, 1);
        b.add_html_localized(158, 22, 100, 18, 1011299, false, false); // Rename book

        // List of entries
        let entries = &self.book.entries;

        for i in 0..16usize {
            let (col, row) = ((i / 8) as i32, (i % 8) as i32);
            // Use charge button
            b.add_button(130 + col * 160, 65 + row * 15, 2103, 2104, 2 + i as i32 * 6);

            // Description label
            if i < entries.len() {
                let entry = &entries[i];

                b.add_label_cropped(
                    145 + col * 160,
                    60 + row * 15,
                    115,
                    17,
                    hue_of(entry),
                    &get_name(&entry.description),
                );
            } else {
                b.add_label_cropped(145 + col * 160, 60 + row * 15, 115, 17, 0, "Empty");
            }
        }

        // Turn page button
        b.add_button_type(393, 14, 2206, 2206, 0, GumpButtonType::Page, 2);
    }

    fn add_details(&self, b: &mut DynamicGumpBuilder, index: usize, half: i32) {
        let base = 2 + index as i32 * 6;
        // Use charge button
        b.add_button(130 + half * 160, 65, 2103, 2104, base);

        if index < self.book.entries.len() {
            let e = &self.book.entries[index];

            // Location labels
            if let Some(s) = &e.sextant {
                b.add_label(
                    135 + half * 160,
                    80,
                    0,
                    &format!(
                        "{}° {}'{}",
                        s.y_lat,
                        s.y_mins,
                        if s.y_south { "S" } else { "N" }
                    ),
                );
                b.add_label(
                    135 + half * 160,
                    95,
                    0,
                    &format!(
                        "{}° {}'{}",
                        s.x_long,
                        s.x_mins,
                        if s.x_east { "E" } else { "W" }
                    ),
                );
            }

            // Drop rune button
            b.add_button(135 + half * 160, 115, 2437, 2438, base + 1);
            b.add_html_localized(150 + half * 160, 115, 100, 18, 1011298, false, false); // Drop rune

            // Set as default button
            let def_button_id = if !e.is_default { 2361 } else { 2360 };

            b.add_button(160 + half * 140, 20, def_button_id, def_button_id, base + 2);
            b.add_html_localized(175 + half * 140, 15, 100, 18, 1011300, false, false); // Set default

            if self.aos {
                b.add_button(135 + half * 160, 140, 2103, 2104, base + 3);
                b.add_html_localized(150 + half * 160, 136, 110, 20, 1062722, false, false); // Recall

                b.add_button(135 + half * 160, 158, 2103, 2104, base + 4);
                b.add_html_localized(150 + half * 160, 154, 110, 20, 1062723, false, false); // Gate Travel

                b.add_button(135 + half * 160, 176, 2103, 2104, base + 5);
                b.add_html_localized(150 + half * 160, 172, 110, 20, 1062724, false, false); // Sacred Journey
            } else {
                // Recall button
                b.add_button(135 + half * 160, 140, 2271, 2271, base + 3);

                // Gate button
                b.add_button(205 + half * 160, 140, 2291, 2291, base + 4);
            }

            // Description label
            b.add_label_cropped(
                145 + half * 160,
                60,
                115,
                17,
                hue_of(e),
                &get_name(&e.description),
            );
        } else {
            // Description label
            b.add_label_cropped(145 + half * 160, 60, 115, 17, 0, "Empty");
        }
    }
}

/// `SendTargetCoordinates(to, location, map)`.
fn send_target_coordinates(
    w: &mut World,
    o: &dyn RunebookOps,
    book: muo_server::ids::ItemId,
    index: usize,
    to: MobileId,
) {
    let Some((_, _, Some(s))) = o.entry_location(w, book, index) else {
        return;
    };
    let SextantView {
        x_long,
        y_lat,
        x_mins,
        y_mins,
        x_east,
        y_south,
    } = s;
    let text = format!(
        "{y_lat}° {y_mins}'{}, {x_long}' {x_mins}'{}",
        if y_south { "S" } else { "N" },
        if x_east { "E" } else { "W" }
    );
    muo_content::mx::send_message(w, to, &text);
}

/// `from.Prompt = new InternalPrompt(book)`.
fn begin_rename_prompt(
    w: &mut World,
    from: MobileId,
    o: Rc<dyn RunebookOps>,
    book: muo_server::ids::ItemId,
) {
    let (o1, o2) = (o.clone(), o);
    let prompt = Prompt {
        on_response: Some(Rc::new(move |w: &mut World, from: MobileId, text: &str| {
            let range = if muo_content::core::ml(w) { 3 } else { 1 };
            let loc = o1.world_location(w, book);
            if o1.deleted(w, book) || !muo_content::mx::in_range_point(w, from, loc, range) {
                return;
            }

            if o1.check_access(w, book, from) {
                o1.set_description(w, book, &fix_html(text.trim()));
                o1.send_gump_to(w, book, from);

                muo_content::mx::send_message(w, from, "The book's title has been changed.");
            } else {
                muo_content::mx::msg(w, from, 502416); // That cannot be done while the book is locked down.
            }
        })),
        on_cancel: Some(Rc::new(move |w: &mut World, from: MobileId| {
            muo_content::mx::msg(w, from, 502415); // Request cancelled.

            let range = if muo_content::core::ml(w) { 3 } else { 1 };
            let loc = o2.world_location(w, book);
            if !o2.deleted(w, book) && muo_content::mx::in_range_point(w, from, loc, range) {
                o2.send_gump_to(w, book, from);
            }
        })),
    };
    let id = add_prompt(w, prompt);
    muo_server::mobile::part1::props_b::set_prompt(w, from, Some(id));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m_z::ops::RunebookEntryView;
    use crate::m_z::tests::{layout_of, sections, strings_of};
    use muo_primitives::{Point3D, Serial};
    use muo_server::ids::ItemId;

    fn gump(aos: bool) -> DynamicGumpObj<RunebookGump> {
        let entry = RunebookEntryView {
            description: "  Home ".to_owned(),
            map: Some(muo_server::map::MapId(1)),
            map_index: 1,
            location: Point3D::new(1, 2, 3),
            sextant: Some(SextantView {
                x_long: 10,
                y_lat: 20,
                x_mins: 3,
                y_mins: 4,
                x_east: true,
                y_south: false,
            }),
            is_default: true,
        };
        let book = RunebookView {
            book: ItemId(Serial::new(0x4000_0001)),
            cur_charges: 5,
            max_charges: 10,
            entries: vec![entry],
        };
        DynamicGumpObj::new(150, 200, RunebookGump { book, aos })
    }

    #[test]
    fn the_index_lists_each_rune_with_its_map_hue() {
        let mut g = gump(true);
        let (l, strings) = sections(&mut g);
        assert!(l.contains("{ button 125 15 2472 2473 1 0 1 }"), "{l}");
        assert!(l.contains("{ button 130 65 2103 2104 1 0 2 }"), "{l}"); // use charge, entry 0
        assert!(l.contains("{ button 290 65 2103 2104 1 0 8 }"), "{l}"); // entry 1 (empty) in the second column
        let s = strings_of(&strings);
        assert!(
            s.contains(&"Home".to_owned()) && s.contains(&"Empty".to_owned()),
            "{s:?}"
        );
        assert!(
            s.contains(&"20° 4'N".to_owned()) && s.contains(&"10° 3'E".to_owned()),
            "{s:?}"
        );
        // eight turnable pages of two entries
        assert_eq!(l.matches("{ page ").count(), 10, "{l}");
    }

    #[test]
    fn aos_has_three_travel_buttons_and_pre_aos_two() {
        let aos = layout_of(&mut gump(true));
        assert!(aos.contains("{ button 135 140 2103 2104 1 0 5 }"), "{aos}");
        assert!(aos.contains("{ button 135 176 2103 2104 1 0 7 }"), "{aos}");
        let old = layout_of(&mut gump(false));
        assert!(old.contains("{ button 135 140 2271 2271 1 0 5 }"), "{old}");
        assert!(old.contains("{ button 205 140 2291 2291 1 0 6 }"), "{old}");
        assert!(
            !old.contains(" 2 7 }") && !old.contains("2103 2104 1 0 7 }"),
            "{old}"
        );
    }

    #[test]
    fn map_hues() {
        assert_eq!(
            [
                get_map_hue(Some(1)),
                get_map_hue(Some(0)),
                get_map_hue(Some(2)),
                get_map_hue(Some(4)),
                get_map_hue(None)
            ],
            [10, 81, 1102, 1154, 0]
        );
        assert_eq!(get_name(""), "(indescript)");
    }
}
