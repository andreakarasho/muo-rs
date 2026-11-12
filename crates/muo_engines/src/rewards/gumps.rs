//! The veteran reward gumps: RewardChoiceGump, RewardConfirmGump, RewardOptionGump and RewardDemolitionGump.

use std::rc::Rc;

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::addon;
use muo_content::props::Value;
use muo_primitives::MessageType;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::net_state::NetStateId;

use super::{
    CATEGORIES, Entry, LISTS, RewardHooks, compute_reward_info, consume_reward_point,
    display_notice, has_access_category, has_access_entry,
};

fn mobile_of(w: &World, ns: NetStateId) -> Option<MobileId> {
    w.net_states.get(&ns).and_then(|n| n.mobile)
}

fn has_net_state(w: &World, m: MobileId) -> bool {
    w.mobile(m).and_then(|x| x.net_state).is_some()
}

/// The entries of a category as `(list index, entry)`, in the order `RewardCategory.Entries` got them (lists in
/// order, then the order of the list).
fn category_entries(category: usize) -> Vec<(usize, &'static Entry)> {
    let mut v = Vec::new();
    for (i, list) in LISTS.iter().enumerate() {
        for e in list.iter() {
            if e.category == category {
                v.push((i, e));
            }
        }
    }
    v
}

// ---- RewardChoiceGump ----

pub struct RewardChoiceGump {
    from: MobileId,
    /// `RewardSystem.RewardInterval` in days, `HasAccess` per entry, and the numbers on the first page.
    interval_days: f64,
    cur: i32,
    max: i32,
    access: Vec<Vec<bool>>,
    category_access: Vec<bool>,
}

/// `RewardChoiceGump.DisplayTo(from)`.
pub fn display_choice(w: &mut World, from: MobileId) {
    if !has_net_state(w, from) {
        return;
    }
    let (cur, max, _) = compute_reward_info(w, from);
    let access: Vec<Vec<bool>> = (0..CATEGORIES.len())
        .map(|c| {
            category_entries(c)
                .iter()
                .map(|(i, e)| has_access_entry(w, from, *i, e))
                .collect()
        })
        .collect();
    let category_access = (0..CATEGORIES.len())
        .map(|c| has_access_category(w, from, c))
        .collect();
    let g = RewardChoiceGump {
        from,
        interval_days: super::state_interval_days(w),
        cur,
        max,
        access,
        category_access,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(0, 0, g)), true);
}

fn get_button_id(ty: usize, index: usize) -> i32 {
    2 + (index * 20 + ty) as i32
}

impl RewardChoiceGump {
    fn pages_per_category(&self, category: usize) -> usize {
        let n = self.access[category].iter().filter(|a| **a).count();
        n.div_ceil(24)
    }

    fn render_category(&self, b: &mut DynamicGumpBuilder, category: usize, page: &mut i32) {
        b.add_page(*page);
        let mut i = 0usize;
        for (j, (_, entry)) in category_entries(category).into_iter().enumerate() {
            if !self.access[category][j] {
                continue;
            }
            if i == 24 {
                *page += 1;
                b.add_button_type(305, 415, 0xFA5, 0xFA7, 0, GumpButtonType::Page, *page);
                b.add_html_localized(340, 415, 200, 20, 1011066, false, false); // Next page
                b.add_page(*page);
                b.add_button_type(270, 415, 0xFAE, 0xFB0, 0, GumpButtonType::Page, *page - 1);
                b.add_html_localized(185, 415, 200, 20, 1011067, false, false); // Previous page
                i = 0;
            }
            let (x, y) = ((i / 12 * 250) as i32, (i % 12 * 25) as i32);
            b.add_button(55 + x, 80 + y, 5540, 5541, get_button_id(category, j));
            b.add_html_localized(80 + x, 80 + y, 250, 20, entry.name, false, false);
            i += 1;
        }
        *page += 1;
    }
}

impl DynamicGumpLogic for RewardChoiceGump {
    const TYPE_NAME: &'static str = "Server.Engines.VeteranRewards.RewardChoiceGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        // RenderBackground
        b.add_page(0);
        b.add_background(10, 10, 600, 450, 2600);
        b.add_button(530, 415, 4017, 4019, 0);
        b.add_button_type(60, 415, 4014, 4016, 0, GumpButtonType::Page, 1);
        b.add_html_localized(95, 415, 200, 20, 1049755, false, false); // Main Menu

        // RenderCategories
        let days = self.interval_days;
        let interval = if days == 30.0 {
            "month".to_owned()
        } else if days == 60.0 {
            "two months".to_owned()
        } else if days == 90.0 {
            "three months".to_owned()
        } else if days == 365.0 {
            "year".to_owned()
        } else {
            format!("{days} day{}", if days == 1.0 { "" } else { "s" })
        };
        b.add_page(1);
        b.add_html(
            60,
            35,
            500,
            70,
            &format!(
                "<B>Ultima Online Rewards Program</B><BR>Thank you for being a part of the Ultima Online community for a full {interval}. As a token of our appreciation, you may select from the following in-game reward items listed below. The gift items will be attributed to the character you have logged-in with on the shard you are on when you chose the item(s). The number of rewards you are entitled to are listed below and are for your entire account. To read more about these rewards before making a selection, feel free to visit the uo.com site at <A HREF=\"https://uo.com/wiki/ultima-online-wiki/items/veteran-rewards\">https://uo.com/wiki/ultima-online-wiki/items/veteran-rewards</A>."
            ),
            true,
            true,
        );
        b.add_html_localized(60, 105, 300, 35, 1006006, false, false); // Your current total of rewards to choose:
        b.add_label(370, 107, 50, &format!("{}", self.max - self.cur));
        b.add_html_localized(60, 140, 300, 35, 1006007, false, false); // You have already chosen:
        b.add_label(370, 142, 50, &format!("{}", self.cur));

        let mut page = 2;
        for (i, &name) in CATEGORIES.iter().enumerate() {
            if !self.category_access[i] {
                page += 1;
                continue;
            }
            b.add_button_type(
                100,
                180 + i as i32 * 40,
                4005,
                4005,
                0,
                GumpButtonType::Page,
                page,
            );
            page += self.pages_per_category(i) as i32;
            b.add_html_localized(135, 180 + i as i32 * 40, 300, 20, name, false, false);
        }
        page = 2;
        for i in 0..CATEGORIES.len() {
            self.render_category(b, i, &mut page);
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let mut button_id = info.button_id - 1;
        if button_id == 0 {
            let (cur, max, _) = compute_reward_info(w, self.from);
            if cur < max {
                display_notice(w, self.from);
            }
        } else {
            button_id -= 1;
            let (ty, index) = (button_id % 20, button_id / 20);
            if (0..CATEGORIES.len() as i32).contains(&ty) {
                let entries = category_entries(ty as usize);
                if index >= 0 && (index as usize) < entries.len() {
                    let (list, entry) = entries[index as usize];
                    if !has_access_entry(w, self.from, list, entry) {
                        return;
                    }
                    display_confirm(w, self.from, list, entry);
                }
            }
        }
    }
}

// ---- RewardConfirmGump ----

pub struct RewardConfirmGump {
    from: MobileId,
    list: usize,
    entry: &'static Entry,
    from_name: String,
}

/// `RewardConfirmGump.DisplayTo(from, entry)`.
pub fn display_confirm(w: &mut World, from: MobileId, list: usize, entry: &'static Entry) {
    if !has_net_state(w, from) {
        return;
    }
    let from_name = w
        .mobile(from)
        .and_then(|m| m.name.clone())
        .unwrap_or_default();
    let g = RewardConfirmGump {
        from,
        list,
        entry,
        from_name,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(0, 0, g)), true);
}

/// `RewardEntry.Construct()`: `ItemType.CreateInstance<Item>(Args)`, then `IsRewardItem = true`.
pub fn construct(w: &mut World, entry: &Entry) -> Option<ItemId> {
    let args: Vec<Value> = entry.args.iter().map(|a| Value::Int(*a)).collect();
    let item = muo_content::factory::make_item_args(w, entry.item_type, &args)?;
    if let Some(f) = w
        .resources
        .get::<RewardHooks>()
        .and_then(|h| h.set_is_reward_item)
    {
        f(w, item);
    } else if let Some(d) = muo_content::items::clothing::data_mut(w, item) {
        d.is_reward_item = true;
    }
    Some(item)
}

impl DynamicGumpLogic for RewardConfirmGump {
    const TYPE_NAME: &'static str = "Server.Engines.VeteranRewards.RewardConfirmGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(10, 10, 500, 300, 2600);
        b.add_html_localized(30, 55, 300, 35, 1006000, false, false); // You have selected:
        b.add_html_localized(335, 55, 150, 35, self.entry.name, false, false);
        b.add_html_localized(30, 95, 300, 35, 1006001, false, false); // This will be assigned to this character:
        b.add_label(335, 95, 0, &self.from_name);
        // Are you sure you wish to select this reward for this character? ...
        b.add_html_localized(35, 160, 450, 90, 1006002, true, true);
        b.add_button(60, 265, 4005, 4007, 1);
        b.add_html_localized(95, 266, 150, 35, 1006044, false, false); // Ok
        b.add_button(295, 265, 4017, 4019, 0);
        b.add_html_localized(330, 266, 150, 35, 1006045, false, false); // Cancel
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        if info.button_id == 1 {
            if !has_access_entry(w, from, self.list, self.entry) {
                return;
            }
            if let Some(item) = construct(w, self.entry) {
                if self.entry.item_type == "Server.Items.RedSoulstone" {
                    // `soulstone.Account = _from.Account.Username`
                    let username = w
                        .mobile(from)
                        .and_then(|m| m.account)
                        .and_then(|a| muo_content::accounting::account(w, a))
                        .map(|a| a.username.clone());
                    if let Some(f) = w
                        .resources
                        .get::<RewardHooks>()
                        .and_then(|h| h.set_soulstone_account)
                    {
                        f(w, item, username.unwrap_or_default());
                    }
                }
                if consume_reward_point(w, from) {
                    muo_server::mobile::part2::items::add_to_backpack(w, from, item);
                } else if let Some(t) = w.item(item).map(|i| i.ty) {
                    (t.delete)(w, item);
                }
            }
        }
        let (cur, max, _) = compute_reward_info(w, from);
        if cur < max {
            display_notice(w, from);
        }
    }
}

// ---- RewardOptionGump ----

/// `IRewardOption.OnOptionSelected(from, choice)`.
pub type OptionSelected = Rc<dyn Fn(&mut World, MobileId, i32)>;

pub struct RewardOptionGump {
    /// `RewardOptionList`: `(ID, Cliloc)`.
    options: Vec<(i32, i32)>,
    title: i32,
    on_selected: OptionSelected,
}

/// `RewardOptionGump.DisplayTo(from, option, title)`; `options` is what `option.GetOptions(list)` added.
pub fn display_option(
    w: &mut World,
    from: MobileId,
    options: Vec<(i32, i32)>,
    title: i32,
    on_selected: OptionSelected,
) {
    if !has_net_state(w, from) {
        return;
    }
    let g = RewardOptionGump {
        options,
        title,
        on_selected,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(60, 36, g)), true);
}

impl DynamicGumpLogic for RewardOptionGump {
    const TYPE_NAME: &'static str = "Server.Gumps.RewardOptionGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 273, 324, 0x13BE);
        b.add_image_tiled(10, 10, 253, 20, 0xA40);
        b.add_image_tiled(10, 40, 253, 244, 0xA40);
        b.add_image_tiled(10, 294, 253, 20, 0xA40);
        b.add_alpha_region(10, 10, 253, 304);
        b.add_button(10, 294, 0xFB1, 0xFB2, 0);
        b.add_html_localized_color(45, 296, 450, 20, 1060051, 0x7FFF, false, false); // CANCEL
        if self.title > 0 {
            b.add_html_localized_color(14, 12, 273, 20, self.title, 0x7FFF, false, false);
        } else {
            // Select your choice from the menu below.
            b.add_html_localized_color(14, 12, 273, 20, 1080392, 0x7FFF, false, false);
        }
        b.add_page(1);
        for (i, (id, cliloc)) in self.options.iter().enumerate() {
            b.add_button(19, 49 + i as i32 * 24, 0x845, 0x846, *id);
            b.add_html_localized_color(
                44,
                47 + i as i32 * 24,
                213,
                20,
                *cliloc,
                0x7FFF,
                false,
                false,
            );
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if self.options.iter().any(|(id, _)| *id == info.button_id)
            && let Some(m) = mobile_of(w, sender)
        {
            let f = self.on_selected.clone();
            f(w, m, info.button_id);
        }
    }
}

// ---- RewardDemolitionGump ----

pub struct RewardDemolitionGump {
    addon: ItemId,
    question: i32,
}

/// `RewardDemolitionGump.DisplayTo(from, addon, question)`.
pub fn display_demolition(w: &mut World, from: MobileId, addon_item: ItemId, question: i32) {
    if !has_net_state(w, from) || w.item(addon_item).is_none_or(|i| i.deleted()) {
        return;
    }
    let g = RewardDemolitionGump {
        addon: addon_item,
        question,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(150, 50, g)), true);
}

impl DynamicGumpLogic for RewardDemolitionGump {
    const TYPE_NAME: &'static str = "Server.Gumps.RewardDemolitionGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 220, 170, 0x13BE);
        b.add_background(10, 10, 200, 150, 0xBB8);
        b.add_html_localized(20, 30, 180, 60, self.question, false, false); // Do you wish to re-deed this decoration?
        b.add_html_localized(55, 100, 150, 25, 1011011, false, false); // CONTINUE
        b.add_button(20, 100, 0xFA5, 0xFA7, 1);
        b.add_html_localized(55, 125, 150, 25, 1011012, false, false); // CANCEL
        b.add_button(20, 125, 0xFA5, 0xFA7, 0);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let item = self.addon;
        if w.item(item).is_none_or(|i| i.deleted()) || info.button_id != 1 {
            return;
        }
        let Some(m) = mobile_of(w, sender) else {
            return;
        };
        // `BaseHouse.FindHouseAt(m)`: the house at the mobile's location, height 16.
        let at = w.mobile(m).and_then(|x| x.map.map(|map| (x.location, map)));
        let finder = addon::hooks_mut(w).find_house_at_point;
        let house = at.zip(finder).and_then(|((p, map), f)| f(w, p, map, 16));
        if !house.is_some_and(|h| addon::house_is_owner(w, h, m)) {
            // You can only re-deed this decoration if you are the house owner or originally placed the decoration.
            muo_content::mx::msg(w, m, 1049784);
            return;
        }
        let loc = w.item(item).map_or_else(Default::default, |i| i.location);
        if muo_content::mx::in_range_point(w, m, loc, 2) {
            let class = addon::addon_class(w, item);
            let deed = match (class.deed_of, class.deed) {
                (Some(f), _) => Some(f(w, item)),
                (None, Some(f)) => Some(f(w)),
                _ => None,
            };
            if let (Some(deed), Some(house)) = (deed, house) {
                muo_server::mobile::part2::items::add_to_backpack(w, m, deed);
                addon::house_addons_remove(w, house, item);
                if let Some(t) = w.item(item).map(|i| i.ty) {
                    (t.delete)(w, item);
                }
            }
        } else {
            muo_server::mobile::part2::messages::local_overhead_message_number(
                w,
                m,
                MessageType::REGULAR,
                0x3B2,
                1019045,
                "",
            ); // I can't reach that.
        }
    }
}
