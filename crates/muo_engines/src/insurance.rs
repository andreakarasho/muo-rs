//! Item insurance: `Insurance` (the death hooks, `CanInsure`, toggling), `ItemInsuranceMenuGump`,
//! `ItemInsuranceMenuConfirmGump` and `CancelRenewInventoryInsuranceGump`.
//!
//! The menu gump's state, which the confirm and cancel gumps point back to, is
//! [`InsuranceMenus`] (per player: the items, the selections, the page) and each gump is a snapshot of it.

use indexmap::IndexMap;
use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::layout::GumpButtonType;
use muo_content::gumps::{DynamicGumpBuilder, StaticGumpBuilder, send_gump_to_mobile};
use muo_content::player_mobile::{self, PlayerFlag, hooks as phooks};
use muo_content::vendor::banker;
use muo_primitives::Serial;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::ops;
use muo_server::layer::Layer;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part1::props_b;
use muo_server::mobile::part2::messages::send_localized_message;
use muo_server::net_state::NetStateId;
use muo_server::target::{self, TargetFlags, Targeted};

/// `InsuranceContext`.
#[derive(Default)]
struct InsuranceContext {
    insurance_reward: Option<MobileId>,
    insurance_bonus: i32,
    missed_auto_renewal: bool,
}

/// `Insurance._insuranceContexts` and the open `ItemInsuranceMenuGump`s.
#[derive(Default)]
pub struct InsuranceState {
    contexts: IndexMap<MobileId, InsuranceContext>,
    menus: IndexMap<MobileId, MenuState>,
}

/// The fields of `ItemInsuranceMenuGump`.
struct MenuState {
    items: Vec<ItemId>,
    insure: Vec<bool>,
    page: usize,
}

fn st(w: &mut World) -> &mut InsuranceState {
    w.resources.get_mut::<InsuranceState>()
}

/// `Insurance.Enabled`.
pub fn enabled(w: &World) -> bool {
    w.core.features.insurance_enabled
}

/// `Insurance.Configure`: the legacy `insurance.enable` setting.
pub fn configure(w: &mut World) {
    let aos = muo_content::core::aos(w);
    w.core.features.insurance_enabled = w.config.get_setting("insurance.enable", aos);
    let h = w.resources.get_mut::<phooks::SystemHooks>();
    h.insurance_enabled = Some(enabled);
    h.insurance_before_death = Some(check_insurance_before_death);
    h.insurance_item_on_death = Some(check_item_insurance_on_death);
    h.insurance_on_death = Some(check_insurance_on_death);
    h.insurance_toggle = Some(toggle_item_insurance);
    h.insurance_open_menu = Some(open_item_insurance_menu);
    h.insurance_cancel_renew = Some(cancel_renew_inventory_insurance);
    h.insurance_auto_renew = Some(auto_renew_inventory_insurance);
    h.insurance_feature_enabled = Some(enabled);
}

/// `Insurance.GetInsuranceCost`.
pub fn get_insurance_cost(_w: &World, _from: MobileId, _item: ItemId) -> i32 {
    600
}

fn msg(w: &mut World, to: MobileId, number: i32, args: &str, hue: i32) {
    send_localized_message(w, to, number, args, hue);
}

fn is_player(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.ty.is(&player_mobile::PLAYER_MOBILE))
}

fn auto_renew(w: &World, m: MobileId) -> bool {
    player_mobile::data(w, m).is_some_and(|d| d.flag(PlayerFlag::AUTO_RENEW_INSURANCE))
}

fn set_auto_renew(w: &mut World, m: MobileId, v: bool) {
    if let Some(d) = player_mobile::data_mut(w, m) {
        d.set_flag(PlayerFlag::AUTO_RENEW_INSURANCE, v);
    }
}

/// `CheckInsuranceBeforeDeath(from)`.
pub fn check_insurance_before_death(w: &mut World, from: MobileId) {
    if !enabled(w) {
        return;
    }
    let Some(mut damager) =
        muo_server::mobile::part2::damage::find_most_recent_damager(w, from, false)
    else {
        return;
    };
    if muo_content::creature::data::cd(w, damager).is_some() {
        match muo_content::creature::control::get_master(w, damager) {
            Some(m) => damager = m,
            None => return, // `creature.GetMaster()` null: `null is PlayerMobile` is false
        }
    }
    if damager != from && is_player(w, damager) {
        st(w).contexts.insert(
            from,
            InsuranceContext {
                insurance_reward: Some(damager),
                ..Default::default()
            },
        );
    }
}

/// `CheckInsuranceOnDeath(from)`.
pub fn check_insurance_on_death(w: &mut World, from: MobileId) {
    if !enabled(w) {
        return;
    }
    let Some(ctx) = st(w).contexts.shift_remove(&from) else {
        return;
    };
    if ctx.missed_auto_renewal {
        msg(w, from, 1061115, "", 0x3B2); // You do not have the gold to automatically reinsure all your items.
    }
    if let Some(reward) = ctx.insurance_reward
        && ctx.insurance_bonus > 0
        && banker::deposit(w, reward, ctx.insurance_bonus, false)
    {
        // ~1_AMOUNT~ gold has been deposited into your bank box.
        msg(w, reward, 1060397, &ctx.insurance_bonus.to_string(), 0x3B2);
    }
}

/// `CheckItemInsuranceOnDeath(from, item)`.
pub fn check_item_insurance_on_death(w: &mut World, from: MobileId, item: ItemId) -> bool {
    if !enabled(w) || !w.item(item).is_some_and(|i| i.insured()) {
        return false;
    }
    if let Some(d) = phooks::duel_state(w, from)
        && d.registered
        && d.started
        && !(d.has_player && d.eliminated)
    {
        return true;
    }
    st(w).contexts.entry(from).or_default();
    if auto_renew(w, from) {
        let mut cost = get_insurance_cost(w, from, item);
        if st(w)
            .contexts
            .get(&from)
            .is_some_and(|c| c.insurance_reward.is_some())
        {
            cost /= 2;
        }
        if banker::withdraw(w, from, cost) {
            ops::set_paid_insurance(w, item, true);
            msg(w, from, 1060398, &cost.to_string(), 0x3B2); // ~1_AMOUNT~ gold has been withdrawn from your bank box.
        } else {
            ops::set_paid_insurance(w, item, false);
            ops::set_insured(w, item, false);
            if let Some(c) = st(w).contexts.get_mut(&from) {
                c.missed_auto_renewal = true;
            }
        }
    } else {
        ops::set_paid_insurance(w, item, false);
        ops::set_insured(w, item, false);
    }
    if let Some(c) = st(w).contexts.get_mut(&from) {
        c.insurance_bonus += 300;
    }
    true
}

fn item_is(w: &World, item: ItemId, names: &[&str]) -> bool {
    let mut t = w.item(item).map(|i| i.ty);
    while let Some(c) = t {
        if names.contains(&c.full_name) {
            return true;
        }
        t = c.parent;
    }
    false
}

/// `Insurance.CanInsure(from, item)`.
pub fn can_insure(w: &World, from: MobileId, item: ItemId) -> bool {
    if !enabled(w) {
        return false;
    }
    let Some(i) = w.item(item) else { return false };
    let is_container = i.ty.is(&muo_server::item::container::CONTAINER);
    if is_container && !item_is(w, item, &["Server.Items.BaseQuiver"])
        || item_is(
            w,
            item,
            &[
                "Server.Items.BagOfSending",
                "Server.Items.KeyRing",
                "Server.Items.PotionKeg",
                "Server.Factions.Sigil",
            ],
        )
    {
        return false;
    }
    if i.stackable() {
        return false;
    }
    if i.loot_type == muo_server::item::LootType::Cursed {
        return false;
    }
    if i.item_id == 0x204E {
        return false; // death shroud
    }
    if i.layer == Layer::Mount {
        return false;
    }
    i.loot_type != muo_server::item::LootType::Blessed
        && i.loot_type != muo_server::item::LootType::Newbied
        && i.blessed_for != Some(from)
}

/// `ToggleItemInsurance(from)`: asks for a target.
pub fn toggle_item_insurance(w: &mut World, from: MobileId) {
    if !muo_server::mobile::part2::stats::check_alive(w, from, true) {
        return;
    }
    begin_toggle_target(w, from);
    msg(w, from, 1060868, "", 0x3B2); // Target the item you wish to toggle insurance status on <ESC> to cancel
}

fn begin_toggle_target(w: &mut World, from: MobileId) {
    target::begin_target(w, from, -1, false, TargetFlags::NONE, |w, from, t| {
        if !muo_server::mobile::part2::stats::check_alive(w, from, true) {
            return;
        }
        let item = match t {
            Targeted::Entity(EntityId::Item(i)) => Some(i),
            _ => None,
        };
        toggle_item_insurance_item(w, from, item, true);
    });
}

/// `ToggleItemInsurance(from, item, target)`.
pub fn toggle_item_insurance_item(
    w: &mut World,
    from: MobileId,
    item: Option<ItemId>,
    target: bool,
) {
    let child = item.is_some_and(|i| ops::is_child_of(w, i, Some(EntityId::Mobile(from))));
    let Some(item) = item.filter(|_| child) else {
        if target {
            begin_toggle_target(w, from);
        }
        // You can only insure items that you have equipped or that are in your backpack
        msg(w, from, 1060871, "", 0x23);
        return;
    };
    if w.item(item).is_some_and(|i| i.insured()) {
        ops::set_insured(w, item, false);
        msg(w, from, 1060874, "", 0x35); // You cancel the insurance on the item
        if target {
            begin_toggle_target(w, from);
            msg(w, from, 1060868, "", 0x23);
        }
    } else if !can_insure(w, from, item) {
        if target {
            begin_toggle_target(w, from);
        }
        msg(w, from, 1060869, "", 0x23); // You cannot insure that
    } else {
        if !w.item(item).is_some_and(|i| i.paid_insurance()) {
            let cost = get_insurance_cost(w, from, item);
            if banker::withdraw(w, from, cost) {
                msg(w, from, 1060398, &cost.to_string(), 0x3B2);
                ops::set_paid_insurance(w, item, true);
            } else {
                msg(w, from, 1061079, "", 0x23); // You lack the funds to purchase the insurance
                return;
            }
        }
        ops::set_insured(w, item, true);
        msg(w, from, 1060873, "", 0x23); // You have insured the item
        if target {
            begin_toggle_target(w, from);
            msg(w, from, 1060868, "", 0x23);
        }
    }
}

/// `AutoRenewInventoryInsurance(from)`.
pub fn auto_renew_inventory_insurance(w: &mut World, from: MobileId) {
    if !muo_server::mobile::part2::stats::check_alive(w, from, true) {
        return;
    }
    // You have selected to automatically reinsure all insured items upon death
    msg(w, from, 1060881, "", 0x23);
    set_auto_renew(w, from, true);
}

/// `CancelRenewInventoryInsurance(from)`.
pub fn cancel_renew_inventory_insurance(w: &mut World, from: MobileId) {
    if !muo_server::mobile::part2::stats::check_alive(w, from, true) {
        return;
    }
    if muo_content::core::se(w) {
        send_cancel_renew_gump(w, from, false);
    } else {
        // You have cancelled automatically reinsuring all insured items upon death
        msg(w, from, 1061075, "", 0x23);
        set_auto_renew(w, from, false);
    }
}

fn display_in_gump(w: &World, from: MobileId, item: ItemId) -> bool {
    let Some(i) = w.item(item) else { return false };
    let level = w
        .mobile(from)
        .map_or(AccessLevel::Player, |m| m.access_level);
    (i.visible() || level >= AccessLevel::GameMaster) && (i.insured() || can_insure(w, from, item))
}

/// `OpenItemInsuranceMenu(from)`.
pub fn open_item_insurance_menu(w: &mut World, from: MobileId) {
    if !muo_server::mobile::part2::stats::check_alive(w, from, true)
        || props_b::net_state(w, from).is_none()
    {
        return;
    }
    let mut queue: Vec<ItemId> = Vec::new();
    let worn: Vec<ItemId> = w.mobile(from).map(|m| m.items.clone()).unwrap_or_default();
    for i in worn {
        if display_in_gump(w, from, i) {
            queue.push(i);
        }
    }
    if let Some(pack) = props_b::backpack(w, from) {
        for i in muo_server::item::enumerable::find_items(w, pack, true, |_, _| true) {
            if display_in_gump(w, from, i) {
                queue.push(i);
            }
        }
    }
    if queue.is_empty() {
        msg(w, from, 1114915, "", 0x35); // None of your current items meet the requirements for insurance.
    } else if is_player(w, from) {
        let insure = queue
            .iter()
            .map(|i| w.item(*i).is_some_and(|x| x.insured()))
            .collect();
        st(w).menus.insert(
            from,
            MenuState {
                items: queue,
                insure,
                page: 0,
            },
        );
        send_menu(w, from);
    }
}

// ---- ItemInsuranceMenuGump -----------------------------------------------------------------------------------------------

struct Row {
    serial: Serial,
    item_id: i32,
    hue: i32,
    ox: i32,
    oy: i32,
    insure: bool,
    cost: i32,
}

pub struct ItemInsuranceMenuGump {
    from: MobileId,
    page: usize,
    count: usize,
    auto_renew: bool,
    balance: i32,
    cost: i32,
    rows: Vec<Row>,
}

fn send_menu(w: &mut World, from: MobileId) {
    muo_server::item::bounds::ensure_loaded(w);
    let Some((items, insure, page)) = w
        .resources
        .get::<InsuranceState>()
        .and_then(|s| s.menus.get(&from))
        .map(|m| (m.items.clone(), m.insure.clone(), m.page))
    else {
        return;
    };
    let balance = banker::get_balance(w, from);
    let mut cost = 0;
    for (i, it) in items.iter().enumerate() {
        if insure[i] {
            cost += get_insurance_cost(w, from, *it);
        }
    }
    let mut rows = Vec::new();
    let mut i = page * 4;
    while i < (page + 1) * 4 && i < items.len() {
        let it = items[i];
        if let Some(x) = w.item(it) {
            let b = muo_server::item::bounds::bounds(w, x.item_id as usize);
            rows.push(Row {
                serial: x.serial,
                item_id: x.item_id,
                hue: x.hue,
                ox: 40 - b.width() / 2 - b.x(),
                oy: 30 - b.height() / 2 - b.y(),
                insure: insure[i],
                cost: get_insurance_cost(w, from, it),
            });
        }
        i += 1;
    }
    let g = ItemInsuranceMenuGump {
        from,
        page,
        count: items.len(),
        auto_renew: auto_renew(w, from),
        balance,
        cost,
        rows,
    };
    send_gump_to_mobile(w, from, Box::new(DynamicGumpObj::new(25, 50, g)), false);
}

impl DynamicGumpLogic for ItemInsuranceMenuGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ItemInsuranceMenuGump";
    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 520, 510, 0x13BE);
        b.add_image_tiled(10, 10, 500, 30, 0xA40);
        b.add_image_tiled(10, 50, 500, 355, 0xA40);
        b.add_image_tiled(10, 415, 500, 80, 0xA40);
        b.add_alpha_region(10, 10, 500, 485);
        b.add_button(15, 470, 0xFB1, 0xFB2, 0);
        b.add_html_localized_color(50, 472, 80, 20, 1011012, 0x7FFF, false, false); // CANCEL
        if self.auto_renew {
            b.add_button(360, 10, 9723, 9724, 1);
        } else {
            b.add_button(360, 10, 9720, 9722, 1);
        }
        b.add_html_localized_color(395, 14, 105, 20, 1114122, 0x7FFF, false, false); // AUTO REINSURE
        b.add_button(395, 470, 0xFA5, 0xFA6, 2);
        b.add_html_localized_color(430, 472, 50, 20, 1006044, 0x7FFF, false, false); // OK
        b.add_html_localized_color(10, 14, 150, 20, 1114121, 0x7FFF, false, false); // <CENTER>ITEM INSURANCE MENU</CENTER>
        b.add_html_localized_color(45, 54, 70, 20, 1062214, 0x7FFF, false, false); // Item
        b.add_html_localized_color(250, 54, 70, 20, 1061038, 0x7FFF, false, false); // Cost
        b.add_html_localized_color(400, 54, 70, 20, 1114311, 0x7FFF, false, false); // Insured
        b.add_html_localized_color(15, 420, 300, 20, 1114310, 0x7FFF, false, false); // GOLD AVAILABLE:
        b.add_label(215, 420, 0x481, &self.balance.to_string());
        b.add_html_localized_color(15, 435, 300, 20, 1114123, 0x7FFF, false, false); // TOTAL COST OF INSURANCE:
        b.add_label(215, 435, 0x481, &self.cost.to_string());
        if self.cost != 0 {
            b.add_html_localized_color(15, 450, 300, 20, 1114125, 0x7FFF, false, false); // NUMBER OF DEATHS PAYABLE:
            b.add_label(215, 450, 0x481, &(self.balance / self.cost).to_string());
        }
        let mut y = 72;
        for (n, r) in self.rows.iter().enumerate() {
            let i = (self.page * 4 + n) as i32;
            b.add_image_tiled_button(
                40,
                y,
                0x918,
                0x918,
                0,
                GumpButtonType::Page,
                0,
                r.item_id,
                r.hue,
                r.ox,
                r.oy,
                0,
            );
            b.add_item_property(r.serial);
            if r.insure {
                b.add_button(400, y, 9723, 9724, 100 + i);
                b.add_label(250, y, 0x481, &r.cost.to_string());
            } else {
                b.add_button(400, y, 9720, 9722, 100 + i);
                b.add_label(250, y, 0x66C, &r.cost.to_string());
            }
            y += 75;
        }
        if self.page >= 1 {
            b.add_button(15, 380, 0xFAE, 0xFAF, 3);
            b.add_html_localized_color(50, 380, 450, 20, 1044044, 0x7FFF, false, false); // PREV PAGE
        }
        if (self.page + 1) * 4 < self.count {
            b.add_button(400, 380, 0xFA5, 0xFA7, 4);
            b.add_html_localized_color(435, 380, 70, 20, 1044045, 0x7FFF, false, false); // NEXT PAGE
        }
    }

    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let from = self.from;
        if info.button_id == 0 || !muo_server::mobile::part2::stats::check_alive(w, from, true) {
            return;
        }
        if !w
            .resources
            .get::<InsuranceState>()
            .is_some_and(|s| s.menus.contains_key(&from))
        {
            return;
        }
        match info.button_id {
            1 => {
                // Auto Reinsure
                if auto_renew(w, from) {
                    send_cancel_renew_gump(w, from, true);
                } else {
                    auto_renew_inventory_insurance(w, from);
                    send_menu(w, from);
                }
            }
            2 => send_confirm_gump(w, from), // OK
            3 => {
                if let Some(m) = st(w).menus.get_mut(&from)
                    && m.page >= 1
                {
                    m.page -= 1;
                    send_menu(w, from);
                }
            }
            4 => {
                if let Some(m) = st(w).menus.get_mut(&from)
                    && (m.page + 1) * 4 < m.items.len()
                {
                    m.page += 1;
                    send_menu(w, from);
                }
            }
            b => {
                let idx = b - 100;
                if idx >= 0
                    && let Some(m) = st(w).menus.get_mut(&from)
                    && (idx as usize) < m.items.len()
                {
                    m.insure[idx as usize] = !m.insure[idx as usize];
                }
                send_menu(w, from);
            }
        }
    }
}

/// `ItemInsuranceMenuGump.ToggleSelected()`.
fn toggle_selected(w: &mut World, from: MobileId) {
    let Some((items, insure)) = w
        .resources
        .get::<InsuranceState>()
        .and_then(|s| s.menus.get(&from))
        .map(|m| (m.items.clone(), m.insure.clone()))
    else {
        return;
    };
    for (i, it) in items.iter().enumerate() {
        let current = w.item(*it).is_some_and(|x| x.insured());
        if current != insure[i] {
            toggle_item_insurance_item(w, from, Some(*it), false);
        }
    }
}

// ---- ItemInsuranceMenuConfirmGump / CancelRenewInventoryInsuranceGump ---------------------------------------------------------

pub struct ItemInsuranceMenuConfirmGump {
    from: MobileId,
}

fn send_confirm_gump(w: &mut World, from: MobileId) {
    send_gump_to_mobile(
        w,
        from,
        Box::new(StaticGumpObj::new(
            250,
            200,
            ItemInsuranceMenuConfirmGump { from },
        )),
        false,
    );
}

impl StaticGumpLogic for ItemInsuranceMenuConfirmGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ItemInsuranceMenuConfirmGump";
    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 240, 142, 0x13BE);
        b.add_image_tiled(6, 6, 228, 100, 0xA40);
        b.add_image_tiled(6, 116, 228, 20, 0xA40);
        b.add_alpha_region(6, 6, 228, 142);
        b.add_html_localized_color(8, 8, 228, 100, 1114300, 0x7FFF, false, false); // Do you wish to insure all newly selected items?
        b.add_button(6, 116, 0xFB1, 0xFB2, 0);
        b.add_html_localized_color(40, 118, 450, 20, 1060051, 0x7FFF, false, false); // CANCEL
        b.add_button(114, 116, 0xFA5, 0xFA7, 1);
        b.add_html_localized_color(148, 118, 450, 20, 1073996, 0x7FFF, false, false); // ACCEPT
    }
    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let pm = self.from;
        if !is_player(w, pm) || !muo_server::mobile::part2::stats::check_alive(w, pm, true) {
            return;
        }
        if info.button_id == 1 {
            toggle_selected(w, pm);
        } else {
            msg(w, pm, 1042021, "", 0x3B2); // Cancelled.
            send_menu(w, pm);
        }
    }
}

pub struct CancelRenewInventoryInsuranceGump {
    from: MobileId,
    /// Whether an `ItemInsuranceMenuGump` is to be shown again afterwards.
    from_menu: bool,
}

fn send_cancel_renew_gump(w: &mut World, from: MobileId, from_menu: bool) {
    send_gump_to_mobile(
        w,
        from,
        Box::new(StaticGumpObj::new(
            250,
            200,
            CancelRenewInventoryInsuranceGump { from, from_menu },
        )),
        false,
    );
}

impl StaticGumpLogic for CancelRenewInventoryInsuranceGump {
    const TYPE_NAME: &'static str = "Server.Gumps.CancelRenewInventoryInsuranceGump";
    fn singleton(&self) -> bool {
        true
    }
    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 240, 142, 0x13BE);
        b.add_image_tiled(6, 6, 228, 100, 0xA40);
        b.add_image_tiled(6, 116, 228, 20, 0xA40);
        b.add_alpha_region(6, 6, 228, 142);
        b.add_html_localized_color(8, 8, 228, 100, 1071021, 0x7FFF, false, false); // You are about to disable inventory insurance auto-renewal.
        b.add_button(6, 116, 0xFB1, 0xFB2, 0);
        b.add_html_localized_color(40, 118, 450, 20, 1060051, 0x7FFF, false, false); // CANCEL
        b.add_button(114, 116, 0xFA5, 0xFA7, 1);
        b.add_html_localized_color(148, 118, 450, 20, 1071022, 0x7FFF, false, false); // DISABLE IT!
    }
    fn on_response(&mut self, w: &mut World, _sender: NetStateId, info: &RelayInfo) {
        let pm = self.from;
        if !is_player(w, pm) || !muo_server::mobile::part2::stats::check_alive(w, pm, true) {
            return;
        }
        if info.button_id == 1 {
            // You have cancelled automatically reinsuring all insured items upon death
            msg(w, pm, 1061075, "", 0x23);
            set_auto_renew(w, pm, false);
        } else {
            msg(w, pm, 1042021, "", 0x3B2); // Cancelled.
        }
        if self.from_menu {
            send_menu(w, pm);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cost_is_flat() {
        let w = World::new();
        assert_eq!(
            get_insurance_cost(&w, MobileId(Serial::new(1)), ItemId(Serial::new(2))),
            600
        );
    }

    #[test]
    fn nothing_is_insurable_while_the_feature_is_off() {
        let w = World::new();
        assert!(!enabled(&w) || !can_insure(&w, MobileId(Serial::new(1)), ItemId(Serial::new(2))));
    }
}
