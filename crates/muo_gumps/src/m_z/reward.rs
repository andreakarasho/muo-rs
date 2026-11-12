//! `RewardGump`: a generic version of the EA Clean Up Britannia reward gump.

use std::rc::Rc;

use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::layout::GumpButtonType;
use muo_content::text::add_html_text_builder;
use muo_io::text::TextDefinition;
use muo_primitives::Rectangle2D;
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::net_state::NetStateId;

use super::common::{draw_builder, has_net_state, mobile_of, send_singleton};

/// `IRewardEntry`, with the entry's `ItemBounds.Bounds[ItemID]` taken when the list is made.
#[derive(Clone, Debug)]
pub struct RewardEntry {
    pub price: i32,
    pub item_id: i32,
    pub hue: i32,
    pub tooltip: i32,
    pub description: Option<TextDefinition>,
    pub bounds: Rectangle2D,
}

impl RewardEntry {
    pub fn new(
        w: &World,
        price: i32,
        item_id: i32,
        hue: i32,
        tooltip: i32,
        description: Option<TextDefinition>,
    ) -> Self {
        Self {
            price,
            item_id,
            hue,
            tooltip,
            description,
            bounds: muo_server::item::bounds::bounds(w, usize::try_from(item_id).unwrap_or(0)),
        }
    }
}

/// `RewardPickedHandler(Mobile from, int index)`.
pub type RewardPickedHandler = Rc<dyn Fn(&mut World, MobileId, usize)>;

pub struct RewardGump {
    title: TextDefinition,
    rewards: Rc<Vec<RewardEntry>>,
    points: i32,
    on_picked: RewardPickedHandler,
}

/// `RewardGump.DisplayTo(from, title, rewards, points, onPicked)`.
pub fn display_to(
    w: &mut World,
    from: MobileId,
    title: TextDefinition,
    rewards: Rc<Vec<RewardEntry>>,
    points: i32,
    on_picked: Option<RewardPickedHandler>,
) {
    if has_net_state(w, from)
        && !rewards.is_empty()
        && let Some(on_picked) = on_picked
    {
        let gump = DynamicGumpObj::new(
            250,
            50,
            RewardGump {
                title,
                rewards,
                points,
                on_picked,
            },
        );
        send_singleton(w, from, gump);
    }
}

impl DynamicGumpLogic for RewardGump {
    const TYPE_NAME: &'static str = "Server.Gumps.RewardGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);

        b.add_image(0, 0, 0x1F40, 0);
        b.add_image_tiled(20, 37, 300, 308, 0x1F42);
        b.add_image(20, 325, 0x1F43, 0);
        b.add_image(35, 8, 0x39, 0);
        b.add_image_tiled(65, 8, 257, 10, 0x3A);
        b.add_image(290, 8, 0x3B, 0);
        b.add_image(32, 33, 0x2635, 0);
        b.add_image_tiled(70, 55, 230, 2, 0x23C5);

        draw_builder(
            b,
            add_html_text_builder(Some(&self.title), false, false, 1, -1),
            70,
            35,
            270,
            20,
        );

        b.add_html_localized_color(50, 65, 150, 20, 1072843, 1, false, false); // Your Reward Points:
        b.add_label(230, 65, 0x64, &self.points.to_string());
        b.add_image_tiled(35, 85, 270, 2, 0x23C5);
        b.add_html_localized_color(35, 90, 270, 20, 1072844, 1, false, false); // Please Choose a Reward:

        b.add_page(1);

        let mut offset = 110;
        let mut page = 1;

        for (i, entry) in self.rewards.iter().enumerate() {
            let bounds = entry.bounds;
            let height = 36.max(bounds.height());

            if offset + height > 320 {
                b.add_html_localized_color(240, 335, 60, 20, 1072854, 1, false, false); // <div align=right>Next</div>
                b.add_button_type(300, 335, 0x15E1, 0x15E5, 51, GumpButtonType::Page, page + 1);

                page += 1;
                b.add_page(page);

                b.add_button_type(150, 335, 0x15E3, 0x15E7, 52, GumpButtonType::Page, page - 1);
                b.add_html_localized_color(170, 335, 60, 20, 1074880, 1, false, false); // Previous

                offset = 110;
            }

            let available = entry.price <= self.points;
            let half = offset + height / 2;

            if available {
                b.add_button(35, half - 6, 0x837, 0x838, 100 + i as i32);
            }

            b.add_item(
                83 - bounds.width() / 2 - bounds.x(),
                half - bounds.height() / 2 - bounds.y(),
                entry.item_id,
                if available { entry.hue } else { 995 },
            );

            if entry.tooltip != 0 {
                b.add_tooltip(entry.tooltip);
            }

            b.add_label(
                133,
                half - 10,
                if available { 0x64 } else { 0x21 },
                &entry.price.to_string(),
            );

            if let Some(d) = &entry.description {
                if let Some(s) = &d.string {
                    b.add_html(190, offset, 114, height, s, false, false);
                } else if d.number != 0 {
                    b.add_html_localized_color(190, offset, 114, height, d.number, 1, false, false);
                }
            }

            offset += height + 10;
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let choice = info.button_id - 100;

        if choice >= 0 && (choice as usize) < self.rewards.len() {
            let entry = self.rewards[choice as usize].clone();

            if entry.price <= self.points
                && let Some(from) = mobile_of(w, sender)
            {
                confirm_display_to(
                    w,
                    from,
                    RewardParent {
                        title: self.title.clone(),
                        rewards: self.rewards.clone(),
                        points: self.points,
                        on_picked: self.on_picked.clone(),
                    },
                    choice as usize,
                    entry,
                );
            }
        }
    }
}

/// The `RewardGump` a `RewardConfirmGump` returns to.
pub struct RewardParent {
    pub title: TextDefinition,
    pub rewards: Rc<Vec<RewardEntry>>,
    pub points: i32,
    pub on_picked: RewardPickedHandler,
}

pub struct RewardConfirmGump {
    index: usize,
    parent: RewardParent,
    entry: RewardEntry,
}

/// `RewardConfirmGump.DisplayTo(from, parent, index, entry)`.
pub fn confirm_display_to(
    w: &mut World,
    from: MobileId,
    parent: RewardParent,
    index: usize,
    entry: RewardEntry,
) {
    if has_net_state(w, from) {
        let gump = DynamicGumpObj::new(
            120,
            50,
            RewardConfirmGump {
                index,
                parent,
                entry,
            },
        );
        send_singleton(w, from, gump);
    }
}

impl DynamicGumpLogic for RewardConfirmGump {
    const TYPE_NAME: &'static str = "Server.Gumps.RewardConfirmGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.set_no_close();

        b.add_page(0);

        b.add_image_tiled(0, 0, 348, 262, 0xA8E);
        b.add_alpha_region(0, 0, 348, 262);
        b.add_image(0, 15, 0x27A8, 0);
        b.add_image_tiled(0, 30, 17, 200, 0x27A7);
        b.add_image(0, 230, 0x27AA, 0);
        b.add_image(15, 0, 0x280C, 0);
        b.add_image_tiled(30, 0, 300, 17, 0x280A);
        b.add_image(315, 0, 0x280E, 0);
        b.add_image(15, 244, 0x280C, 0);
        b.add_image_tiled(30, 244, 300, 17, 0x280A);
        b.add_image(315, 244, 0x280E, 0);
        b.add_image(330, 15, 0x27A8, 0);
        b.add_image_tiled(330, 30, 17, 200, 0x27A7);
        b.add_image(330, 230, 0x27AA, 0);
        b.add_image(333, 2, 0x2716, 0);
        b.add_image(333, 248, 0x2716, 0);
        b.add_image(2, 248, 0x2716, 0);
        b.add_image(2, 2, 0x2716, 0);

        b.add_item(140, 120, self.entry.item_id, self.entry.hue);

        if self.entry.tooltip != 0 {
            b.add_tooltip(self.entry.tooltip);
        }

        b.add_html_localized_color(25, 22, 200, 20, 1074974, 0x7D00, false, false); // Confirm Selection
        b.add_image(25, 40, 0xBBF, 0);
        b.add_html_localized_color(25, 55, 300, 120, 1074975, 0x7FFF, false, false); // Are you sure you wish to select this?
        b.add_radio(25, 175, 0x25F8, 0x25FB, true, 1);
        b.add_radio(25, 210, 0x25F8, 0x25FB, false, 0);
        b.add_html_localized_color(60, 180, 280, 20, 1074976, 0x7FFF, false, false); // Yes
        b.add_html_localized_color(60, 215, 280, 20, 1074977, 0x7FFF, false, false); // No
        b.add_button(265, 220, 0xF7, 0xF8, 7);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if info.button_id == 7 && info.is_switched(1) {
            (self.parent.on_picked.clone())(w, from, self.index);
        } else {
            let p = &self.parent;
            display_to(
                w,
                from,
                p.title.clone(),
                p.rewards.clone(),
                p.points,
                Some(p.on_picked.clone()),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m_z::tests::layout_of;
    use muo_primitives::Rectangle2D;

    fn entry(
        price: i32,
        item_id: i32,
        hue: i32,
        description: Option<TextDefinition>,
    ) -> RewardEntry {
        RewardEntry {
            price,
            item_id,
            hue,
            tooltip: 0,
            description,
            bounds: Rectangle2D::new(5, 6, 30, 40),
        }
    }

    #[test]
    fn an_affordable_reward_has_a_button_and_a_centered_item() {
        let g = RewardGump {
            title: TextDefinition::of_number(1075000),
            rewards: Rc::new(vec![entry(
                10,
                0xE21,
                0,
                Some(TextDefinition::of_text("a bandage")),
            )]),
            points: 20,
            on_picked: Rc::new(|_, _, _| {}),
        };
        let mut g = DynamicGumpObj::new(250, 50, g);
        let l = layout_of(&mut g);
        // height = max(36, 40) = 40, half = 110 + 20
        assert!(l.contains("{ button 35 124 2103 2104 1 0 100 }"), "{l}");
        assert!(l.contains("{ tilepic 63 104 3617 }"), "{l}");
        assert!(l.contains("{ page 1 }"), "{l}");
    }

    #[test]
    fn an_unaffordable_reward_is_drawn_grey_and_without_a_button() {
        let g = RewardGump {
            title: TextDefinition::of_number(1075000),
            rewards: Rc::new(vec![entry(50, 0xE21, 7, None)]),
            points: 20,
            on_picked: Rc::new(|_, _, _| {}),
        };
        let mut g = DynamicGumpObj::new(250, 50, g);
        let l = layout_of(&mut g);
        assert!(!l.contains(" 100 }"), "{l}");
        assert!(l.contains("{ tilepichue 63 104 3617 995 }"), "{l}");
    }

    #[test]
    fn rewards_that_do_not_fit_go_on_the_next_page() {
        let rewards: Vec<_> = (0..6).map(|_| entry(1, 0xE21, 0, None)).collect();
        let g = RewardGump {
            title: TextDefinition::of_number(1075000),
            rewards: Rc::new(rewards),
            points: 20,
            on_picked: Rc::new(|_, _, _| {}),
        };
        let mut g = DynamicGumpObj::new(250, 50, g);
        let l = layout_of(&mut g);
        // 110 + (40 + 10) per reward: the fifth would end at 350 > 320
        assert!(l.contains("{ button 300 335 5601 5605 0 2 51 }"), "{l}");
        assert!(l.contains("{ page 2 }"), "{l}");
    }
}
