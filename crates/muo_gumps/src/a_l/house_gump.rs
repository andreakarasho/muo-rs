//! `HouseGump` (`HouseListGump`, `HouseRemoveGump`, `HouseGump`, `RenamePrompt`) and
//! `HouseGumpAOS`.
//!
//! Lists kept by reference (`_house.CoOwners`) are named by [`ListKind`] and read from the house when the gump is
//! built. The names shown are those at that moment, which is when it is sent.
//!
//! HouseFoundation (customize, sign hanger, signpost, foundation style) and `HasAddonContainers` go through
//! `AlHooks` (set in `hooks::configure`);
//! `ConvertHouse_Callback` moves the lists, the moving crate, the vendor inventories and the internalized vendors
//! (`vendor.House` through `AlHooks::set_vendor_house`) to the new house.

use std::rc::Rc;

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::system::close_gump;
use muo_multis::house::{self, DecayLevel, DecayType, access, doors, storage};

use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::prompt::{Prompt, add_prompt};

use super::common::{mobile_of, name_of, send};
use super::hooks::hooks;
use super::houses::confirm_demolish;
use crate::m_z::warning;

fn msg(w: &mut World, to: MobileId, number: i32) {
    muo_content::mx::msg(w, to, number);
}

fn ns_of(w: &World, m: MobileId) -> Option<NetStateId> {
    w.mobile(m).and_then(|x| x.net_state)
}

/// Which of the house's mobile lists a gump shows.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ListKind {
    CoOwners,
    Friends,
    Bans,
    Access,
}

fn live_list(w: &World, h: ItemId, k: ListKind) -> Vec<MobileId> {
    house::data(w, h).map_or_else(Vec::new, |d| match k {
        ListKind::CoOwners => d.co_owners.clone(),
        ListKind::Friends => d.friends.clone(),
        ListKind::Bans => d.bans.clone(),
        ListKind::Access => d.access.clone(),
    })
}

fn live_list_mut(w: &mut World, h: ItemId, k: ListKind) -> Option<&mut Vec<MobileId>> {
    house::data_mut(w, h).map(|d| match k {
        ListKind::CoOwners => &mut d.co_owners,
        ListKind::Friends => &mut d.friends,
        ListKind::Bans => &mut d.bans,
        ListKind::Access => &mut d.access,
    })
}

/// `name = m.Name; name = name.Trim(); name.Length > 0`, with "Account of " where asked for.
fn display_name(w: &World, m: MobileId, account_of: bool) -> Option<String> {
    let mob = w.mobile(m)?;
    let name = (mob.ty.name)(w, m)?.trim().to_owned();
    if name.is_empty() {
        return None;
    }
    Some(
        if account_of && muo_content::mx::player(w, m) && mob.account.is_some() {
            format!("Account of {name}")
        } else {
            name
        },
    )
}

fn deleted(w: &World, h: ItemId) -> bool {
    house::deleted(w, h)
}

/// The roles of `from` in `h`, `isCombatRestricted` applied: (owner, co-owner, friend).
fn roles(w: &World, h: ItemId, from: MobileId) -> (bool, bool, bool) {
    let restricted = access::is_combat_restricted(w, h, from);
    let owner = access::is_owner(w, h, from);
    let co = owner || access::is_co_owner(w, h, from);
    let friend = co || access::is_friend(w, h, from);
    if restricted {
        (false, false, false)
    } else {
        (owner, co, friend)
    }
}

fn near_sign(w: &World, h: ItemId, from: MobileId) -> bool {
    let Some(sign) = house::sign(w, h) else {
        return false;
    };
    let sign_map = w.item(sign).and_then(|i| i.map);
    muo_content::mx::map(w, from) == sign_map
        && muo_content::mx::in_range_point(
            w,
            from,
            muo_server::item::ops::get_world_location(w, sign),
            18,
        )
}

fn set_prompt(w: &mut World, from: MobileId, p: Prompt) {
    let id = add_prompt(w, p);
    muo_server::mobile::part1::props_b::set_prompt(w, from, Some(id));
}

/// `new RenamePrompt(house)`.
fn rename_prompt(w: &mut World, from: MobileId, h: ItemId) {
    set_prompt(
        w,
        from,
        Prompt {
            on_cancel: None,
            on_response: Some(Rc::new(move |w: &mut World, from, text: &str| {
                if access::is_friend(w, h, from) {
                    if let Some(sign) = house::sign(w, h) {
                        muo_server::item::ops::set_name(w, sign, Some(text.to_owned()));
                    }
                    muo_content::mx::send_message(w, from, "Sign changed.");
                }
            })),
        },
    );
}

// ---- HouseListGump ----

pub struct HouseListGump {
    house: ItemId,
    number: i32,
    /// None for a skipped entry; the display text otherwise.
    entries: Vec<Option<String>>,
}

/// `new HouseListGump(number, list, house, accountOf)`.
pub fn list_gump(
    w: &World,
    number: i32,
    kind: ListKind,
    house: ItemId,
    account_of: bool,
) -> DynamicGumpObj<HouseListGump> {
    let entries = live_list(w, house, kind)
        .into_iter()
        .map(|m| display_name(w, m, account_of))
        .collect();
    DynamicGumpObj::new(
        20,
        30,
        HouseListGump {
            house,
            number,
            entries,
        },
    )
}

impl DynamicGumpLogic for HouseListGump {
    const TYPE_NAME: &'static str = "Server.Gumps.HouseListGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 420, 430, 5054);
        b.add_background(10, 10, 400, 410, 3000);
        b.add_button(20, 388, 4005, 4007, 0);
        b.add_html_localized(55, 388, 300, 20, 1011104, false, false); // Return to previous menu
        b.add_html_localized(20, 20, 350, 20, self.number, false, false);
        for (i, name) in self.entries.iter().enumerate() {
            let i = i as i32;
            if i % 16 == 0 {
                if i != 0 {
                    b.add_button_type(370, 20, 4005, 4007, 0, GumpButtonType::Page, i / 16 + 1);
                }
                b.add_page(i / 16 + 1);
                if i != 0 {
                    b.add_button_type(340, 20, 4014, 4016, 0, GumpButtonType::Page, i / 16);
                }
            }
            if let Some(name) = name {
                b.add_label(55, 55 + i % 16 * 20, 0, name);
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, _info: &RelayInfo) {
        if deleted(w, self.house) {
            return;
        }
        if let Some(from) = mobile_of(w, sender) {
            display_to(w, from, self.house);
        }
    }
}

// ---- HouseRemoveGump ----

pub struct HouseRemoveGump {
    house: ItemId,
    number: i32,
    kind: ListKind,
    account_of: bool,
    /// `_copy`: the list as it was when the gump was made.
    copy: Vec<MobileId>,
    entries: Vec<Option<String>>,
}

/// `new HouseRemoveGump(number, list, house, accountOf)`.
pub fn remove_gump(
    w: &World,
    number: i32,
    kind: ListKind,
    house: ItemId,
    account_of: bool,
) -> DynamicGumpObj<HouseRemoveGump> {
    let copy = live_list(w, house, kind);
    let entries = copy
        .iter()
        .map(|&m| display_name(w, m, account_of))
        .collect();
    DynamicGumpObj::new(
        20,
        30,
        HouseRemoveGump {
            house,
            number,
            kind,
            account_of,
            copy,
            entries,
        },
    )
}

impl DynamicGumpLogic for HouseRemoveGump {
    const TYPE_NAME: &'static str = "Server.Gumps.HouseRemoveGump";

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 420, 430, 5054);
        b.add_background(10, 10, 400, 410, 3000);
        b.add_button(20, 388, 4005, 4007, 0);
        b.add_html_localized(55, 388, 300, 20, 1011104, false, false); // Return to previous menu
        b.add_button(20, 365, 4005, 4007, 1);
        b.add_html_localized(55, 365, 300, 20, 1011270, false, false); // Remove now!
        b.add_html_localized(20, 20, 350, 20, self.number, false, false);
        for (i, name) in self.entries.iter().enumerate() {
            let i = i as i32;
            if i % 15 == 0 {
                if i != 0 {
                    b.add_button_type(370, 20, 4005, 4007, 0, GumpButtonType::Page, i / 15 + 1);
                }
                b.add_page(i / 15 + 1);
                if i != 0 {
                    b.add_button_type(340, 20, 4014, 4016, 0, GumpButtonType::Page, i / 15);
                }
            }
            if let Some(name) = name {
                b.add_checkbox(34, 52 + i % 15 * 20, 0xD2, 0xD3, false, i);
                b.add_label(55, 52 + i % 15 * 20, 0, name);
            }
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if deleted(w, self.house) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        if info.button_id == 1 {
            // Remove now
            if !info.switches.is_empty() {
                for &index in &info.switches {
                    if index >= 0 && (index as usize) < self.copy.len() {
                        let m = self.copy[index as usize];
                        if let Some(l) = live_list_mut(w, self.house, self.kind)
                            && let Some(p) = l.iter().position(|x| *x == m)
                        {
                            l.remove(p);
                        }
                    }
                }
                if !live_list(w, self.house, self.kind).is_empty() {
                    if let Some(ns) = ns_of(w, from) {
                        close_gump::<HouseGump>(w, ns);
                        close_gump::<HouseListGump>(w, ns);
                        close_gump::<HouseRemoveGump>(w, ns);
                    }
                    let g = remove_gump(w, self.number, self.kind, self.house, self.account_of);
                    send(w, from, g);
                    return;
                }
            }
        }
        display_to(w, from, self.house);
    }
}

// ---- HouseGump ----

pub struct HouseGump {
    from: MobileId,
    house: ItemId,
    sign_lines: Option<Vec<String>>,
    owner_name: String,
    lock_down_count: i32,
    max_lock_downs: i32,
    secure_count: i32,
    max_secures: i32,
    public: bool,
    visits: i32,
    flags: (bool, bool, bool),
}

fn owner_name(w: &World, h: ItemId) -> String {
    let Some(m) = house::owner(w, h) else {
        return "(unowned)".to_owned();
    };
    let name = w
        .mobile(m)
        .and_then(|x| (x.ty.name)(w, m))
        .map(|n| n.trim().to_owned())
        .unwrap_or_default();
    if name.is_empty() {
        "(no name)".to_owned()
    } else {
        name
    }
}

/// `HouseGump.DisplayTo(from, house)`.
pub fn display_to(w: &mut World, from: MobileId, h: ItemId) {
    if ns_of(w, from).is_none() || deleted(w, h) {
        return;
    }
    if let Some(ns) = ns_of(w, from) {
        close_gump::<HouseListGump>(w, ns);
        close_gump::<HouseRemoveGump>(w, ns);
    }
    let d = house::d(w, h);
    let sign_lines = house::sign(w, h)
        .and_then(|s| muo_io::text::helpers::wrap(&muo_multis::sign::get_name(w, s), 10, 6));
    let g = HouseGump {
        from,
        house: h,
        sign_lines,
        owner_name: owner_name(w, h),
        lock_down_count: storage::lock_down_count(w, h),
        max_lock_downs: d.max_lock_downs,
        secure_count: storage::secure_count(w, h),
        max_secures: d.max_secures,
        public: d.public,
        visits: d.visits,
        flags: roles(w, h, from),
    };
    send(w, from, DynamicGumpObj::new(20, 30, g));
}

impl DynamicGumpLogic for HouseGump {
    const TYPE_NAME: &'static str = "Server.Gumps.HouseGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let is_friend = self.flags.2;
        b.add_page(0);
        if is_friend {
            b.add_background(0, 0, 420, 430, 5054);
            b.add_background(10, 10, 400, 410, 3000);
        }
        b.add_image(130, 0, 100, 0);
        if let Some(lines) = &self.sign_lines {
            let mut y = (101 - lines.len() as i32 * 14) / 2;
            for s in lines {
                b.add_label(130 + (143 - s.chars().count() as i32 * 8) / 2, y, 0, s);
                y += 14;
            }
        }
        if !is_friend {
            return;
        }
        b.add_html_localized(55, 103, 75, 20, 1011233, false, false); // INFO
        b.add_button_type(20, 103, 4005, 4007, 0, GumpButtonType::Page, 1);
        b.add_html_localized(170, 103, 75, 20, 1011234, false, false); // FRIENDS
        b.add_button_type(135, 103, 4005, 4007, 0, GumpButtonType::Page, 2);
        b.add_html_localized(295, 103, 75, 20, 1011235, false, false); // OPTIONS
        b.add_button_type(260, 103, 4005, 4007, 0, GumpButtonType::Page, 3);
        b.add_html_localized(295, 390, 75, 20, 1011441, false, false); // EXIT
        b.add_button(260, 390, 4005, 4007, 0);
        b.add_html_localized(55, 390, 200, 20, 1011236, false, false); // Change this house's name!
        b.add_button(20, 390, 4005, 4007, 1);

        // Info page
        b.add_page(1);
        b.add_html_localized(20, 135, 100, 20, 1011242, false, false); // Owned by:
        b.add_html(120, 135, 100, 20, &self.owner_name, false, false);
        b.add_html_localized(20, 170, 275, 20, 1011237, false, false); // Number of locked down items:
        b.add_html(
            320,
            170,
            50,
            20,
            &self.lock_down_count.to_string(),
            false,
            false,
        );
        b.add_html_localized(20, 190, 275, 20, 1011238, false, false); // Maximum locked down items:
        b.add_html(
            320,
            190,
            50,
            20,
            &self.max_lock_downs.to_string(),
            false,
            false,
        );
        b.add_html_localized(20, 210, 275, 20, 1011239, false, false); // Number of secure containers:
        b.add_html(
            320,
            210,
            50,
            20,
            &self.secure_count.to_string(),
            false,
            false,
        );
        b.add_html_localized(20, 230, 275, 20, 1011240, false, false); // Maximum number of secure containers:
        b.add_html(
            320,
            230,
            50,
            20,
            &self.max_secures.to_string(),
            false,
            false,
        );
        b.add_html_localized(20, 260, 400, 20, 1018032, false, false); // This house is properly placed.
        b.add_html_localized(20, 280, 400, 20, 1018035, false, false); // This house is of modern design.
        if self.public {
            b.add_html_localized(20, 305, 275, 20, 1011241, false, false); // Number of visits this building has had
            b.add_html(320, 305, 50, 20, &self.visits.to_string(), false, false);
        }

        // Friends page
        b.add_page(2);
        for (x, y, n, id) in [
            (45, 130, 1011266, 2),
            (45, 150, 1011267, 3),
            (45, 170, 1018036, 4),
            (45, 190, 1011268, 5),
            (225, 130, 1011243, 6),
            (225, 150, 1011244, 7),
            (225, 170, 1018037, 8),
            (225, 190, 1011245, 9),
        ] {
            let w = if x == 45 { 150 } else { 155 };
            b.add_html_localized(x, y, w, 20, n, false, false);
            b.add_button(x - 25, y, 2714, 2715, id);
        }
        for (y, n, id) in [
            (215, 1011258, 10),
            (235, 1011259, 11),
            (255, 1011260, 12),
            (275, 1011261, 13),
        ] {
            b.add_html_localized(120, y, 280, 20, n, false, false);
            b.add_button(95, y, 2714, 2715, id);
        }

        // Options page
        b.add_page(3);
        b.add_html_localized(45, 150, 355, 30, 1011248, false, false); // Transfer ownership of the house
        b.add_button(20, 150, 2714, 2715, 14);
        b.add_html_localized(45, 180, 355, 30, 1011249, false, false); // Demolish house and get deed back
        b.add_button(20, 180, 2714, 2715, 15);
        if !self.public {
            b.add_html_localized(45, 210, 355, 30, 1011247, false, false); // Change the house locks
            b.add_button(20, 210, 2714, 2715, 16);
            b.add_html_localized(45, 240, 350, 90, 1011253, false, false); // Declare this building to be public.
            b.add_button(20, 240, 2714, 2715, 17);
        } else {
            b.add_html_localized(45, 210, 350, 30, 1011250, false, false); // Change the sign type
            b.add_button_type(20, 210, 2714, 2715, 0, GumpButtonType::Page, 4);
            b.add_html_localized(45, 240, 350, 30, 1011252, false, false); // Declare this building to be private.
            b.add_button(20, 240, 2714, 2715, 17);
        }

        // Change the sign type
        b.add_page(4);
        for i in 0..24 {
            b.add_radio(53 + i / 4 * 50, 137 + i % 4 * 35, 210, 211, false, i + 1);
            b.add_item(60 + i / 4 * 50, 130 + i % 4 * 35, 2980 + i * 2, 0);
        }
        b.add_html_localized(200, 305, 129, 20, 1011254, false, false); // Guild sign choices
        b.add_button_type(350, 305, 252, 253, 0, GumpButtonType::Page, 5);
        b.add_html_localized(200, 340, 355, 30, 1011277, false, false); // Okay that is fine.
        b.add_button(350, 340, 4005, 4007, 18);

        b.add_page(5);
        for i in 0..29 {
            b.add_radio(53 + i / 5 * 50, 137 + i % 5 * 35, 210, 211, false, i + 25);
            b.add_item(60 + i / 5 * 50, 130 + i % 5 * 35, 3028 + i * 2, 0);
        }
        b.add_html_localized(200, 305, 129, 20, 1011255, false, false); // Shop sign choices
        b.add_button_type(350, 305, 250, 251, 0, GumpButtonType::Page, 4);
        b.add_html_localized(200, 340, 355, 30, 1011277, false, false); // Okay that is fine.
        b.add_button(350, 340, 4005, 4007, 18);
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let h = self.house;
        if deleted(w, h) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let _ = self.from;
        let (is_owner, is_co_owner, is_friend) = roles(w, h, from);
        if !is_friend || !muo_server::mobile::base::alive(w, from) {
            return;
        }
        if !near_sign(w, h, from) {
            return;
        }
        let ns = sender;
        let close_all = |w: &mut World| {
            close_gump::<HouseGump>(w, ns);
            close_gump::<HouseRemoveGump>(w, ns);
            close_gump::<HouseListGump>(w, ns);
        };
        match info.button_id {
            1 => {
                rename_prompt(w, from, h);
                msg(w, from, 501302); // What dost thou wish the sign to say?
            }
            2 => {
                close_all(w);
                let g = list_gump(w, 1011275, ListKind::CoOwners, h, false);
                send(w, from, g);
            }
            3 => {
                if is_owner {
                    msg(w, from, 501328);
                    house::targets::co_owner_target(w, from, true, h);
                } else {
                    msg(w, from, 501327); // Only the house owner may add Co-owners.
                }
            }
            4 => {
                if is_owner {
                    close_all(w);
                    let g = remove_gump(w, 1011274, ListKind::CoOwners, h, false);
                    send(w, from, g);
                } else {
                    msg(w, from, 501329);
                }
            }
            5 => {
                if is_owner {
                    if let Some(l) = live_list_mut(w, h, ListKind::CoOwners) {
                        l.clear();
                    }
                    msg(w, from, 501333);
                } else {
                    msg(w, from, 501330);
                }
            }
            6 => {
                close_all(w);
                let g = list_gump(w, 1011273, ListKind::Friends, h, false);
                send(w, from, g);
            }
            7 => {
                if is_co_owner {
                    msg(w, from, 501317);
                    house::targets::friend_target(w, from, true, h);
                } else {
                    msg(w, from, 501316);
                }
            }
            8 => {
                if is_co_owner {
                    close_all(w);
                    let g = remove_gump(w, 1011272, ListKind::Friends, h, false);
                    send(w, from, g);
                } else {
                    msg(w, from, 501318);
                }
            }
            9 => {
                if is_co_owner {
                    if let Some(l) = live_list_mut(w, h, ListKind::Friends) {
                        l.clear();
                    }
                    msg(w, from, 501332);
                } else {
                    msg(w, from, 501319);
                }
            }
            10 => {
                msg(w, from, 501325);
                house::targets::ban_target(w, from, true, h);
            }
            11 => {
                msg(w, from, 501326);
                house::targets::kick_target(w, from, h);
            }
            12 => {
                close_all(w);
                let g = list_gump(w, 1011271, ListKind::Bans, h, true);
                send(w, from, g);
            }
            13 => {
                close_all(w);
                let g = remove_gump(w, 1011269, ListKind::Bans, h, true);
                send(w, from, g);
            }
            14 => {
                if is_owner {
                    msg(w, from, 501309);
                    house::targets::owner_target(w, from, h);
                } else {
                    msg(w, from, 501310);
                }
            }
            15 => {
                if is_owner {
                    if !muo_content::misc::guild::new_guild_system(w)
                        && storage::find_guildstone(w, h).is_some()
                    {
                        msg(w, from, 501389); // You cannot redeed a house with a guildstone inside.
                    } else {
                        send(w, from, confirm_demolish(h));
                    }
                } else {
                    msg(w, from, 501320);
                }
            }
            16 => {
                if house::data(w, h).is_some_and(|d| d.public) {
                    msg(w, from, 501669); // Public houses are always unlocked.
                } else if is_owner {
                    doors::remove_keys(w, h, from);
                    doors::change_locks(w, h, from);
                    msg(w, from, 501306);
                } else {
                    msg(w, from, 501303);
                }
            }
            17 => {
                if is_owner {
                    let d = house::d(w, h);
                    if d.public && !d.player_vendors.is_empty() {
                        msg(w, from, 501887);
                        return;
                    }
                    access::set_public(w, h, !d.public);
                    if d.public {
                        // now private
                        doors::change_locks(w, h, from);
                        msg(w, from, 501888);
                        msg(w, from, 501306);
                    } else {
                        doors::remove_keys(w, h, from);
                        doors::remove_locks(w, h);
                        msg(w, from, 501886);
                    }
                } else {
                    msg(w, from, 501307);
                }
            }
            18 => {
                if is_owner {
                    if house::data(w, h).is_some_and(|d| d.public) && !info.switches.is_empty() {
                        let index = info.switches[0] - 1;
                        if (0..53).contains(&index) {
                            house::doors::change_sign_type(w, h, 2980 + index * 2);
                        }
                    }
                } else {
                    msg(w, from, 501307);
                }
            }
            _ => {}
        }
    }
}

// ---- HouseGumpAOS ----

/// `HouseGumpPageAOS`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Page {
    Information,
    Security,
    Storage,
    Customize,
    Ownership,
    ChangeHanger,
    ChangeFoundation,
    ChangeSign,
    RemoveCoOwner,
    ListCoOwner,
    RemoveFriend,
    ListFriend,
    RemoveBan,
    ListBan,
    RemoveAccess,
    ListAccess,
    ChangePost,
    Vendors,
}

const LABEL_COLOR: i32 = 0x7FFF;
const SELECTED_COLOR: i32 = 0x421F;
const DISABLED_COLOR: i32 = 0x4210;
const WARNING_COLOR: i32 = 0x7E10;
const LABEL_HUE: i32 = 0x481;
const HIGHLIGHTED_LABEL_HUE: i32 = 0x64;

const HANGER_NUMBERS: [i32; 6] = [2968, 2970, 2972, 2974, 2976, 2978];
const POST_NUMBERS: [i32; 28] = [
    9, 29, 54, 90, 147, 169, 177, 204, 251, 257, 263, 298, 347, 424, 441, 466, 514, 600, 601, 602,
    603, 660, 666, 672, 898, 970, 974, 982,
];
/// 2980 to 3086 step 2, then the ML-only Library and Beekeeper.
fn house_signs() -> Vec<i32> {
    let mut v: Vec<i32> = (0..54).map(|i| 2980 + i * 2).collect();
    v.push(2966);
    v.push(3140);
    v
}

fn foundation_numbers(ml: bool) -> Vec<i32> {
    if ml {
        vec![20, 189, 765, 65, 101, 0x2DF7, 0x2DFB, 0x3672, 0x3676]
    } else {
        vec![20, 189, 765, 65, 101]
    }
}

/// One row of an AddList: the name (None when skipped), whether it is highlighted (a vendor of the viewer).
struct Row {
    index: usize,
    text: String,
    hue: i32,
}

pub struct HouseGumpAos {
    house: ItemId,
    page: Page,
    flags: (bool, bool, bool),
    ml: bool,
    list: Vec<MobileId>,
    rows: Vec<Row>,
    /// `AddList`'s button type (-1: none, 0 and below: label at 10).
    list_button: i32,
    d: house::HouseData,
    public: bool,
    is_foundation: bool,
    has_signpost: bool,
    owner_name: String,
    sign_lines: Option<Vec<String>>,
    built_on: String,
    last_traded: String,
    decay_type: DecayType,
    storage: (i32, i32, i32, i32, i32),
    max_secures: i32,
    max_lockdowns: i32,
    cur_lockdowns: i32,
    bonus_storage: i32,
    new_vendor_system: bool,
    max_vendors: i32,
    has_convert_entry: bool,
}

fn fmt_date(d: muo_primitives::DateTime) -> String {
    let (y, mo, day) = d.ymd();
    format!(
        "{y:04}-{mo:02}-{day:02} {:02}:{:02}:{:02}",
        d.hour(),
        d.minute(),
        d.second()
    )
}

fn is_foundation(w: &World, h: ItemId) -> bool {
    house::class(w, h).class_name == "HouseFoundation"
}

/// `HouseGumpAOS.DisplayTo(from, house, page)`.
pub fn display_aos(w: &mut World, from: MobileId, h: ItemId, page: Page) {
    if ns_of(w, from).is_none() || deleted(w, h) {
        return;
    }
    let flags = roles(w, h, from);
    let d = house::d(w, h);
    let ml = muo_content::core::ml(w);
    let (list, list_button, account_of, star): (Vec<MobileId>, i32, bool, bool) = match page {
        Page::Vendors => (
            available_vendors_for(w, &d.player_vendors, from),
            1,
            false,
            false,
        ),
        Page::RemoveCoOwner => (d.co_owners.clone(), 10, false, true),
        Page::ListCoOwner => (d.co_owners.clone(), -1, false, true),
        Page::RemoveFriend => (d.friends.clone(), 11, false, true),
        Page::ListFriend => (d.friends.clone(), -1, false, true),
        Page::RemoveBan => (d.bans.clone(), 12, true, true),
        Page::ListBan => (d.bans.clone(), -1, true, true),
        Page::RemoveAccess => (d.access.clone(), 13, false, true),
        Page::ListAccess => (d.access.clone(), -1, false, true),
        _ => (Vec::new(), -1, false, false),
    };
    let mut rows = Vec::new();
    let ops = crate::m_z::hooks::ops(w).player_vendors;
    for (i, &m) in list.iter().enumerate() {
        let mut hue = LABEL_HUE;
        let name = if let Some(v) = ops.as_ref().and_then(|o| o.view(w, m)) {
            if ops.as_ref().is_some_and(|o| o.is_owner(w, m, from)) {
                hue = HIGHLIGHTED_LABEL_HUE;
            }
            v.shop_name
        } else if let Some(n) = w.mobile(m).and_then(|x| (x.ty.name)(w, m)) {
            n
        } else {
            continue;
        };
        let mut name = name.trim().to_owned();
        if name.is_empty() {
            continue;
        }
        if account_of
            && muo_content::mx::player(w, m)
            && w.mobile(m).is_some_and(|x| x.account.is_some())
        {
            name = format!("Account of {name}");
        }
        if star {
            name = format!("* {name}");
        }
        rows.push(Row {
            index: i,
            text: name,
            hue,
        });
    }
    let sign_lines = house::sign(w, h)
        .and_then(|s| muo_io::text::helpers::wrap(&muo_multis::sign::get_name(w, s), 10, 6));
    let owner = house::owner(w, h).filter(|&m| !muo_content::mx::deleted(w, m));
    let owner_name = match owner {
        None => "(unowned)".to_owned(),
        Some(m) => {
            let n = name_of(w, m).trim().to_owned();
            if n.is_empty() {
                "(no name)".to_owned()
            } else {
                n
            }
        }
    };
    let nvs = house::new_vendor_system(w);
    let g = HouseGumpAos {
        house: h,
        page,
        flags,
        ml,
        list,
        rows,
        list_button,
        public: d.public,
        is_foundation: is_foundation(w, h),
        has_signpost: hooks(w)
            .and_then(|x| x.foundation_has_signpost)
            .is_some_and(|f| f(w, h)),
        owner_name,
        sign_lines,
        built_on: fmt_date(d.built_on),
        last_traded: fmt_date(d.last_traded),
        decay_type: house::decay::decay_type(w, h),
        storage: storage::aos_cur_secures_parts(w, h),
        max_secures: house::aos_max_secures(w, h),
        max_lockdowns: house::aos_max_lockdowns(w, h),
        cur_lockdowns: storage::aos_cur_lockdowns(w, h),
        bonus_storage: (house::bonus_storage_scalar(w) * 100.0 - 100.0) as i32,
        new_vendor_system: nvs,
        max_vendors: house::new_vendor_system_max_vendors(w, h),
        has_convert_entry: house::convert_entry(w, h).is_some(),
        d,
    };
    send(w, from, DynamicGumpObj::new(50, 40, g));
}

/// `AvailableVendorsFor(m)`.
fn available_vendors_for(w: &mut World, vendors: &[MobileId], m: MobileId) -> Vec<MobileId> {
    let Some(ops) = crate::m_z::hooks::ops(w).player_vendors else {
        return Vec::new();
    };
    vendors
        .iter()
        .copied()
        .filter(|&v| ops.can_interact_with(w, v, m, false))
        .collect()
}

fn button_id(kind: i32, index: i32) -> i32 {
    1 + index * 15 + kind
}

fn add_button_labeled(
    b: &mut DynamicGumpBuilder,
    x: i32,
    y: i32,
    id: i32,
    number: i32,
    enabled: bool,
) {
    if enabled {
        b.add_button(x, y, 4005, 4007, id);
    }
    b.add_html_localized_color(
        x + 35,
        y,
        240,
        20,
        number,
        if enabled { LABEL_COLOR } else { DISABLED_COLOR },
        false,
        false,
    );
}

impl HouseGumpAos {
    fn add_page_button(
        &self,
        b: &mut DynamicGumpBuilder,
        x: i32,
        y: i32,
        id: i32,
        number: i32,
        page: Page,
    ) {
        let sel = self.page == page;
        b.add_button(x, y, if sel { 4006 } else { 4005 }, 4007, id);
        b.add_html_localized_color(
            x + 45,
            y,
            200,
            20,
            number,
            if sel { SELECTED_COLOR } else { LABEL_COLOR },
            false,
            false,
        );
    }

    fn add_list(&self, b: &mut DynamicGumpBuilder) {
        let mut last_page = 0;
        // pages go by the running index of the kept rows; the rows here are those
        for (index, row) in self.rows.iter().enumerate() {
            let index = index as i32;
            let xoffset = index % 20 / 10 * 200;
            let yoffset = index % 10 * 20;
            let page = 1 + index / 20;
            if page != last_page {
                if last_page != 0 {
                    b.add_button_type(40, 360, 4005, 4007, 0, GumpButtonType::Page, page);
                }
                b.add_page(page);
                if last_page != 0 {
                    b.add_button_type(10, 360, 4014, 4016, 0, GumpButtonType::Page, last_page);
                }
                last_page = page;
            }
            if self.list_button != -1 {
                b.add_button(
                    10 + xoffset,
                    150 + yoffset,
                    4005,
                    4007,
                    button_id(self.list_button, row.index as i32),
                );
            }
            b.add_label(
                if self.list_button > 0 {
                    45 + xoffset
                } else {
                    10 + xoffset
                },
                150 + yoffset,
                row.hue,
                &row.text,
            );
        }
    }
}

impl DynamicGumpLogic for HouseGumpAos {
    const TYPE_NAME: &'static str = "Server.Gumps.HouseGumpAOS";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        let (is_owner, is_co_owner, is_friend) = self.flags;
        let vendors = self.page == Page::Vendors;
        b.add_page(0);
        if is_friend || vendors {
            b.add_background(0, 0, 420, if !vendors { 440 } else { 420 }, 5054);
            b.add_image_tiled(10, 10, 400, 100, 2624);
            b.add_alpha_region(10, 10, 400, 100);
            b.add_image_tiled(10, 120, 400, 260, 2624);
            b.add_alpha_region(10, 120, 400, 260);
            b.add_image_tiled(10, 390, 400, if !vendors { 40 } else { 20 }, 2624);
            b.add_alpha_region(10, 390, 400, if !vendors { 40 } else { 20 });
            add_button_labeled(b, 250, if !vendors { 410 } else { 390 }, 0, 1060675, true); // CLOSE
            b.add_image(10, 10, 100, 0);
            if let Some(lines) = &self.sign_lines {
                let mut y = (114 - lines.len() as i32 * 14) / 2;
                for s in lines {
                    b.add_label(10 + (160 - s.chars().count() as i32 * 8) / 2, y, 0, s);
                    y += 14;
                }
            }
        }
        if vendors {
            b.add_html_localized_color(10, 120, 400, 20, 1062428, LABEL_COLOR, false, false); // <CENTER>SHOPS</CENTER>
            self.add_list(b);
            return;
        }
        if !is_friend {
            return;
        }
        if self.public {
            add_button_labeled(b, 10, 390, button_id(0, 0), 1060674, true); // Banish
            add_button_labeled(b, 10, 410, button_id(0, 1), 1011261, true); // Lift a Ban
        } else {
            add_button_labeled(b, 10, 390, button_id(0, 2), 1060676, true); // Grant Access
            add_button_labeled(b, 10, 410, button_id(0, 3), 1060677, true); // Revoke Access
        }
        self.add_page_button(b, 150, 10, button_id(1, 0), 1060668, Page::Information);
        self.add_page_button(b, 150, 30, button_id(1, 1), 1060669, Page::Security);
        self.add_page_button(b, 150, 50, button_id(1, 2), 1060670, Page::Storage);
        self.add_page_button(b, 150, 70, button_id(1, 3), 1060671, Page::Customize);
        self.add_page_button(b, 150, 90, button_id(1, 4), 1060672, Page::Ownership);

        let label = |b: &mut DynamicGumpBuilder, x, y, wd, n, c| {
            b.add_html_localized_color(x, y, wd, 20, n, c, false, false)
        };
        match self.page {
            Page::Information => {
                label(b, 20, 130, 200, 1011242, LABEL_COLOR); // Owned By:
                b.add_label(210, 130, LABEL_HUE, &self.owner_name);
                label(b, 20, 170, 380, 1018032, SELECTED_COLOR);
                label(b, 20, 190, 380, 1018035, SELECTED_COLOR);
                label(
                    b,
                    20,
                    210,
                    380,
                    if self.is_foundation { 1060681 } else { 1060680 },
                    SELECTED_COLOR,
                );
                label(
                    b,
                    20,
                    230,
                    380,
                    if self.public { 1060678 } else { 1060679 },
                    SELECTED_COLOR,
                );
                match self.decay_type {
                    DecayType::Ageless | DecayType::AutoRefresh => {
                        label(b, 20, 250, 380, 1062209, SELECTED_COLOR)
                    }
                    DecayType::ManualRefresh => label(b, 20, 250, 380, 1062208, SELECTED_COLOR),
                    DecayType::Condemned => label(b, 20, 250, 380, 1062207, WARNING_COLOR),
                }
                label(b, 20, 290, 200, 1060692, SELECTED_COLOR); // Built On:
                b.add_label(250, 290, LABEL_HUE, &self.built_on);
                label(b, 20, 310, 200, 1060693, SELECTED_COLOR); // Last Traded:
                b.add_label(250, 310, LABEL_HUE, &self.last_traded);
                label(b, 20, 330, 200, 1061793, SELECTED_COLOR); // House Value
                b.add_label(250, 330, LABEL_HUE, &self.d.price.to_string());
                label(b, 20, 360, 300, 1011241, SELECTED_COLOR); // Number of visits this building has had:
                b.add_label(350, 360, LABEL_HUE, &self.d.visits.to_string());
            }
            Page::Security => {
                add_button_labeled(b, 10, 130, button_id(3, 0), 1011266, is_co_owner);
                add_button_labeled(b, 10, 150, button_id(3, 1), 1011267, is_owner);
                add_button_labeled(b, 10, 170, button_id(3, 2), 1018036, is_owner);
                add_button_labeled(b, 10, 190, button_id(3, 3), 1011268, is_owner);
                add_button_labeled(b, 10, 220, button_id(3, 4), 1011243, true);
                add_button_labeled(b, 10, 240, button_id(3, 5), 1011244, is_co_owner);
                add_button_labeled(b, 10, 260, button_id(3, 6), 1018037, is_co_owner);
                add_button_labeled(b, 10, 280, button_id(3, 7), 1011245, is_co_owner);
                if self.public {
                    add_button_labeled(b, 10, 310, button_id(3, 8), 1011260, true);
                    add_button_labeled(b, 10, 330, button_id(3, 9), 1060698, true);
                    add_button_labeled(b, 210, 130, button_id(3, 12), 1060695, is_owner);
                    label(b, 245, 150, 240, 1060694, SELECTED_COLOR);
                } else {
                    add_button_labeled(b, 10, 310, button_id(3, 10), 1060699, true);
                    add_button_labeled(b, 10, 330, button_id(3, 11), 1060700, true);
                    label(b, 245, 130, 240, 1060695, SELECTED_COLOR);
                    add_button_labeled(b, 210, 150, button_id(3, 13), 1060694, is_owner);
                }
            }
            Page::Storage => {
                label(b, 10, 130, 400, 1060682, LABEL_COLOR); // <CENTER>HOUSE STORAGE SUMMARY</CENTER>
                let (cur_secures, from_secures, from_vendors, from_lockdowns, from_crate) =
                    self.storage;
                let max_secures = self.max_secures;
                let row = |b: &mut DynamicGumpBuilder, y, n, v: String| {
                    label(b, 10, y, 300, n, LABEL_COLOR);
                    b.add_label(310, y, LABEL_HUE, &v);
                };
                if self.bonus_storage > 0 {
                    row(b, 150, 1072519, format!("{}%", self.bonus_storage)); // Increased Storage
                }
                row(b, 170, 1060683, max_secures.to_string()); // Maximum Secure Storage
                row(b, 190, 1060685, from_crate.to_string()); // Used by Moving Crate
                row(b, 210, 1060686, from_lockdowns.to_string()); // Used by Lockdowns
                if self.new_vendor_system {
                    row(b, 230, 1060688, from_secures.to_string()); // Used by Secure Containers
                    row(
                        b,
                        250,
                        1060689,
                        (max_secures - cur_secures).max(0).to_string(),
                    ); // Available Storage
                    row(b, 290, 1060690, self.max_lockdowns.to_string()); // Maximum Lockdowns
                    row(
                        b,
                        310,
                        1060691,
                        (self.max_lockdowns - self.cur_lockdowns).max(0).to_string(),
                    );
                    let vendors =
                        self.d.player_vendors.len() + self.d.vendor_rental_contracts.len();
                    row(b, 350, 1062391, format!("{vendors} / {}", self.max_vendors)); // Vendor Count
                } else {
                    row(b, 230, 1060687, from_vendors.to_string()); // Used by Vendors
                    row(b, 250, 1060688, from_secures.to_string());
                    row(
                        b,
                        270,
                        1060689,
                        (max_secures - cur_secures).max(0).to_string(),
                    );
                    row(b, 330, 1060690, self.max_lockdowns.to_string());
                    row(
                        b,
                        350,
                        1060691,
                        (self.max_lockdowns - self.cur_lockdowns).max(0).to_string(),
                    );
                }
            }
            Page::Customize => {
                let customizable = is_owner && self.is_foundation;
                add_button_labeled(
                    b,
                    10,
                    120,
                    button_id(5, 0),
                    1060759,
                    is_owner && !customizable && self.has_convert_entry,
                );
                add_button_labeled(
                    b,
                    10,
                    160,
                    button_id(5, 1),
                    1060765,
                    is_owner && customizable,
                );
                add_button_labeled(
                    b,
                    10,
                    180,
                    button_id(5, 2),
                    1060760,
                    is_owner && self.d.moving_crate.is_some(),
                );
                add_button_labeled(
                    b,
                    10,
                    210,
                    button_id(5, 3),
                    1060761,
                    is_owner && self.public,
                );
                add_button_labeled(
                    b,
                    10,
                    230,
                    button_id(5, 4),
                    1060762,
                    is_owner && customizable,
                );
                add_button_labeled(
                    b,
                    10,
                    250,
                    button_id(5, 5),
                    1060763,
                    is_owner && customizable && self.has_signpost,
                );
                add_button_labeled(
                    b,
                    10,
                    280,
                    button_id(5, 6),
                    1062004,
                    is_owner && customizable,
                );
                add_button_labeled(b, 10, 310, button_id(5, 7), 1060764, is_co_owner);
            }
            Page::Ownership => {
                add_button_labeled(
                    b,
                    10,
                    130,
                    button_id(6, 0),
                    1061794,
                    is_owner
                        && self.d.moving_crate.is_none()
                        && self.d.internalized_vendors.is_empty(),
                );
                add_button_labeled(b, 10, 150, button_id(6, 1), 1061797, is_owner);
                add_button_labeled(b, 10, 190, button_id(6, 2), 1061798, false);
            }
            Page::ChangeHanger => {
                for (i, n) in HANGER_NUMBERS.iter().enumerate() {
                    let i = i as i32;
                    let (x, y) = (50 + i % 3 * 100, 180 + i / 3 * 80);
                    b.add_button(x, y, 4005, 4007, button_id(7, i));
                    b.add_item(x + 20, y, *n, 0);
                }
            }
            Page::ChangeFoundation => {
                for (i, n) in foundation_numbers(self.ml).iter().enumerate() {
                    let i = i as i32;
                    let (x, y) = (15 + i % 5 * 80, 180 + i / 5 * 100);
                    b.add_button(x, y, 4005, 4007, button_id(8, i));
                    b.add_item(x + 25, y, *n, 0);
                }
            }
            Page::ChangeSign => {
                let signs = house_signs();
                let (per_page, total) = if self.ml { (24, 56) } else { (18, 54) };
                let pages = (f64::from(total) / f64::from(per_page)).ceil() as i32;
                let mut index = 0usize;
                for i in 0..pages {
                    b.add_page(i + 1);
                    b.add_button_type(
                        10,
                        360,
                        4005,
                        4007,
                        0,
                        GumpButtonType::Page,
                        (i + 1) % pages + 1,
                    );
                    let mut j = 0;
                    while j < per_page && total - per_page * i - j > 0 {
                        let (x, y) = (30 + j % 6 * 60, 130 + j / 6 * 60);
                        b.add_button(x, y, 4005, 4007, button_id(9, index as i32));
                        b.add_item(x + 20, y, signs[index], 0);
                        index += 1;
                        j += 1;
                    }
                }
            }
            Page::RemoveCoOwner | Page::ListCoOwner => {
                label(b, 10, 120, 400, 1060730, LABEL_COLOR);
                self.add_list(b);
            }
            Page::RemoveFriend | Page::ListFriend => {
                label(b, 10, 120, 400, 1060731, LABEL_COLOR);
                self.add_list(b);
            }
            Page::RemoveBan | Page::ListBan => {
                label(b, 10, 120, 400, 1060733, LABEL_COLOR);
                self.add_list(b);
            }
            Page::RemoveAccess | Page::ListAccess => {
                label(b, 10, 120, 400, 1060732, LABEL_COLOR);
                self.add_list(b);
            }
            Page::ChangePost => {
                let mut index = 0usize;
                for i in 0..2 {
                    b.add_page(i + 1);
                    b.add_button_type(
                        10,
                        360,
                        4005,
                        4007,
                        0,
                        GumpButtonType::Page,
                        (i + 1) % 2 + 1,
                    );
                    let mut j = 0;
                    while j < 16 && index < POST_NUMBERS.len() {
                        let (x, y) = (15 + j % 8 * 50, 130 + j / 8 * 110);
                        b.add_button(x, y, 4005, 4007, button_id(14, index as i32));
                        b.add_item(x + 10, y, POST_NUMBERS[index], 0);
                        index += 1;
                        j += 1;
                    }
                }
            }
            Page::Vendors => {}
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let h = self.house;
        if deleted(w, h) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let (is_owner, is_co_owner, is_friend) = {
            let (_, _, f) = roles(w, h, from);
            let restricted = access::is_combat_restricted(w, h, from);
            let owner = access::is_owner(w, h, from);
            let co = owner || access::is_co_owner(w, h, from);
            let friend = co || access::is_friend(w, h, from);
            // a restricted viewer keeps `isOwner` here (only co-owner and friend are cleared)
            let _ = f;
            if restricted {
                (owner, false, false)
            } else {
                (owner, co, friend)
            }
        };
        if !muo_server::mobile::part2::stats::check_alive(w, from, true) {
            return;
        }
        if !near_sign(w, h, from) {
            return;
        }
        let foundation = is_foundation(w, h);
        let val = info.button_id - 1;
        if val < 0 {
            return;
        }
        let (kind, index) = (val % 15, val / 15);
        let public = house::data(w, h).is_some_and(|d| d.public);

        if self.page == Page::Vendors {
            if let Some(&vendor) = usize::try_from(index).ok().and_then(|i| self.list.get(i)) {
                let Some(ops) = crate::m_z::hooks::ops(w).player_vendors else {
                    return;
                };
                if !ops.can_interact_with(w, vendor, from, false) {
                    return;
                }
                let sign = house::sign(w, h);
                let near = sign.is_some_and(|s| {
                    let sm = w.item(s).and_then(|i| i.map);
                    muo_content::mx::map(w, from) == sm
                        && muo_content::mx::in_range_point(
                            w,
                            from,
                            muo_server::item::ops::get_world_location(w, s),
                            5,
                        )
                });
                if !near {
                    msg(w, from, 1062429); // You must be within five paces of the house sign to use this option.
                } else if ops.is_owner(w, vendor, from) {
                    if house::new_vendor_system(w) {
                        crate::m_z::player_vendor::new_owner_display_to(w, from, vendor);
                    } else {
                        crate::m_z::player_vendor::owner_display_to(w, from, vendor);
                    }
                } else {
                    ops.open_backpack(w, vendor, from);
                }
            }
            return;
        }
        if !is_friend {
            return;
        }
        let ns = sender;
        let page = |w: &mut World, p: Page| display_aos(w, from, h, p);
        match kind {
            0 => match index {
                0 => {
                    if public {
                        msg(w, from, 501325);
                        house::targets::ban_target(w, from, true, h);
                    }
                }
                1 => {
                    if public {
                        page(w, Page::RemoveBan);
                    }
                }
                2 => {
                    if !public {
                        msg(w, from, 1060711);
                        house::targets::access_target(w, from, h);
                    }
                }
                3 if !public => {
                    page(w, Page::RemoveAccess);
                }
                _ => {}
            },
            1 => {
                let p = match index {
                    0 => Page::Information,
                    1 => Page::Security,
                    2 => Page::Storage,
                    3 => Page::Customize,
                    4 => Page::Ownership,
                    _ => return,
                };
                page(w, p);
            }
            3 => self.security(w, from, h, index, (is_owner, is_co_owner)),
            5 => self.customize(w, from, h, index, (is_owner, is_co_owner), foundation),
            6 => match index {
                0 => {
                    if is_owner
                        && self.d.moving_crate.is_none()
                        && self.d.internalized_vendors.is_empty()
                    {
                        if !muo_content::misc::guild::new_guild_system(w)
                            && storage::find_guildstone(w, h).is_some()
                        {
                            msg(w, from, 501389);
                        } else if muo_content::core::ml(w)
                            && muo_content::mx::access_level(w, from) < AccessLevel::GameMaster
                            && w.now
                                <= house::d(w, h).built_on + muo_primitives::TimeSpan::from_hours(1)
                        {
                            msg(w, from, 1080178); // You must wait one hour between each house demolition.
                        } else {
                            send(w, from, confirm_demolish(h));
                        }
                    }
                }
                1 if is_owner => {
                    if house::new_vendor_system(w) && house::transfer::has_personal_vendors(w, h) {
                        msg(w, from, 1062467);
                    } else if house::decay::decay_level(w, h) == DecayLevel::DemolitionPending {
                        msg(w, from, 1005321);
                    } else {
                        msg(w, from, 501309);
                        house::targets::owner_target(w, from, h);
                    }
                }
                _ => {}
            },
            7 => {
                if is_owner && foundation && (index as usize) < HANGER_NUMBERS.len() {
                    if let Some(f) = hooks(w).and_then(|x| x.foundation_set_sign_hanger) {
                        f(w, h, HANGER_NUMBERS[index as usize]);
                    }
                    page(w, Page::Customize);
                }
            }
            8 => {
                if is_owner && foundation {
                    let new_type = if self.ml && index >= 5 {
                        match index {
                            5 => 5,
                            6 => 6,
                            7 => 7,
                            8 => 8,
                            _ => return,
                        }
                    } else {
                        match index {
                            0 => 1,
                            1 => 2,
                            2 => 3,
                            3 => 4,
                            4 => 0,
                            _ => return,
                        }
                    };
                    if let Some(f) = hooks(w).and_then(|x| x.foundation_set_type) {
                        f(w, h, new_type);
                    }
                    page(w, Page::Customize);
                }
            }
            9 => {
                let signs = house_signs();
                if is_owner && public && (index as usize) < signs.len() {
                    house::doors::change_sign_type(w, h, signs[index as usize]);
                    page(w, Page::Customize);
                }
            }
            10..=13 => {
                let allowed = match kind {
                    10 => is_owner,
                    11 => is_co_owner,
                    _ => true,
                };
                let Some(&target) = usize::try_from(index).ok().and_then(|i| self.list.get(i))
                else {
                    return;
                };
                if !allowed {
                    return;
                }
                let (lk, back) = match kind {
                    10 => {
                        access::remove_co_owner(w, h, from, target);
                        (ListKind::CoOwners, Page::RemoveCoOwner)
                    }
                    11 => {
                        access::remove_friend(w, h, from, target);
                        (ListKind::Friends, Page::RemoveFriend)
                    }
                    12 => {
                        access::remove_ban(w, h, from, target);
                        (ListKind::Bans, Page::RemoveBan)
                    }
                    _ => {
                        access::remove_access(w, h, from, target);
                        (ListKind::Access, Page::RemoveAccess)
                    }
                };
                if !live_list(w, h, lk).is_empty() {
                    page(w, back);
                } else {
                    page(w, Page::Security);
                }
            }
            14 if is_owner && foundation && (index as usize) < POST_NUMBERS.len() => {
                if let Some(f) = hooks(w).and_then(|x| x.foundation_set_signpost_graphic) {
                    f(w, h, POST_NUMBERS[index as usize]);
                }
                page(w, Page::Customize);
            }
            _ => {}
        }
        let _ = ns;
    }
}

fn notice(
    w: &mut World,
    from: MobileId,
    n: impl warning::StaticNotice,
    cb: impl Fn(&mut World) + 'static,
) {
    let g = warning::new_static_notice(n, Some(Rc::new(cb)));
    send(w, from, g);
}

fn warning_gump(
    w: &mut World,
    from: MobileId,
    spec: impl warning::StaticWarning,
    cb: impl Fn(&mut World, bool) + 'static,
) {
    let g = warning::new_static(spec, Some(Rc::new(cb)));
    send(w, from, g);
}

macro_rules! static_warning {
    ($name:ident, $type_name:expr, $w:expr, $h:expr, $content:expr) => {
        struct $name;
        impl warning::StaticWarning for $name {
            const TYPE_NAME: &'static str = $type_name;
            const WIDTH: i32 = $w;
            const HEIGHT: i32 = $h;
            fn static_localized_content(&self) -> i32 {
                $content
            }
        }
    };
}

macro_rules! static_notice {
    ($name:ident, $type_name:expr, $w:expr, $h:expr, $content:expr) => {
        struct $name;
        impl warning::StaticNotice for $name {
            const TYPE_NAME: &'static str = $type_name;
            const WIDTH: i32 = $w;
            const HEIGHT: i32 = $h;
            fn static_localized_content(&self) -> i32 {
                $content
            }
        }
    };
}

static_warning!(
    RemoveAllCoOwnersWarningGump,
    "Server.Gumps.HouseGumpAOS+RemoveAllCoOwnersWarningGump",
    420,
    280,
    1060736
);
static_warning!(
    RemoveAllFriendsWarningGump,
    "Server.Gumps.HouseGumpAOS+RemoveAllFriendsWarningGump",
    420,
    280,
    1018039
);
static_warning!(
    LifeAllBansWarningGump,
    "Server.Gumps.HouseGumpAOS+LifeAllBansWarningGump",
    420,
    280,
    1060753
);
static_warning!(
    RevokeAccessWarning,
    "Server.Gumps.HouseGumpAOS+RevokeAccessWarning",
    420,
    280,
    1061842
);
static_warning!(
    ConvertToCustomHouseWarningGump,
    "Server.Gumps.HouseGumpAOS+ConvertToCustomHouseWarningGump",
    420,
    280,
    1060013
);
static_notice!(
    CannotPerformActionVendorsNoticeGump,
    "Server.Gumps.HouseGumpAOS+CannotPerformActionVendorsNoticeGump",
    320,
    180,
    1062395
);
static_notice!(
    CannotPerformActionContractsNoticeGump,
    "Server.Gumps.HouseGumpAOS+CannotPerformActionContractsNoticeGump",
    320,
    180,
    1062351
);
static_notice!(
    CannotCustomizeAddonsNoticeGump,
    "Server.Gumps.HouseGumpAOS+CannotCustomizeAddonsNoticeGump",
    320,
    180,
    1074863
);
static_notice!(
    ReplaceHouseNoticeGump,
    "Server.Gumps.HouseGumpAOS+ReplaceHouseNoticeGump",
    420,
    280,
    1060012
);
static_notice!(
    HouseConvertedPublicNoticeGump,
    "Server.Gumps.HouseGumpAOS+HouseConvertedPublicNoticeGump",
    320,
    180,
    501886
);
static_notice!(
    CannotConvertPrivateNoticeGump,
    "Server.Gumps.HouseGumpAOS+CannotConvertPrivateNoticeGump",
    320,
    180,
    501887
);
static_notice!(
    HouseConvertedPrivateNoticeGump,
    "Server.Gumps.HouseGumpAOS+HouseConvertedPrivateNoticeGump",
    320,
    180,
    501888
);

/// `HouseConvertedPublicOldSystemNoticeGump`: its content is a string.
struct HouseConvertedPublicOldSystemNoticeGump;

impl warning::StaticNotice for HouseConvertedPublicOldSystemNoticeGump {
    const TYPE_NAME: &'static str =
        "Server.Gumps.HouseGumpAOS+HouseConvertedPublicOldSystemNoticeGump";
    const WIDTH: i32 = 320;
    const HEIGHT: i32 = 180;
    fn content(&self) -> Option<String> {
        Some("This house is now public. Friends of the house may now have vendors working out of this building.".into())
    }
}

fn public_private_notice(w: &mut World, from: MobileId, h: ItemId) {
    if !deleted(w, h) {
        display_aos(w, from, h, Page::Security);
    }
}

fn customize_notice(w: &mut World, from: MobileId, h: ItemId) {
    if !deleted(w, h) {
        display_aos(w, from, h, Page::Customize);
    }
}

impl HouseGumpAos {
    fn security(
        &mut self,
        w: &mut World,
        from: MobileId,
        h: ItemId,
        index: i32,
        roles: (bool, bool),
    ) {
        let (is_owner, is_co_owner) = roles;
        let page = |w: &mut World, p: Page| display_aos(w, from, h, p);
        match index {
            0 => {
                if is_co_owner {
                    page(w, Page::ListCoOwner);
                }
            }
            1 => {
                if is_owner {
                    msg(w, from, 501328);
                    house::targets::co_owner_target(w, from, true, h);
                }
            }
            2 => {
                if is_owner {
                    page(w, Page::RemoveCoOwner);
                }
            }
            3 => {
                if is_owner {
                    warning_gump(w, from, RemoveAllCoOwnersWarningGump, move |w, okay| {
                        if deleted(w, h) {
                            return;
                        }
                        if okay && access::is_owner(w, h, from) {
                            if let Some(l) = live_list_mut(w, h, ListKind::CoOwners) {
                                l.clear();
                            }
                            msg(w, from, 501333);
                        }
                        display_aos(w, from, h, Page::Security);
                    });
                }
            }
            4 => page(w, Page::ListFriend),
            5 => {
                if is_co_owner {
                    msg(w, from, 501317);
                    house::targets::friend_target(w, from, true, h);
                }
            }
            6 => {
                if is_co_owner {
                    page(w, Page::RemoveFriend);
                }
            }
            7 => {
                if is_co_owner {
                    warning_gump(w, from, RemoveAllFriendsWarningGump, move |w, okay| {
                        if deleted(w, h) {
                            return;
                        }
                        if okay && access::is_co_owner(w, h, from) {
                            if let Some(l) = live_list_mut(w, h, ListKind::Friends) {
                                l.clear();
                            }
                            msg(w, from, 501332);
                        }
                        display_aos(w, from, h, Page::Security);
                    });
                }
            }
            8 => page(w, Page::ListBan),
            9 => warning_gump(w, from, LifeAllBansWarningGump, move |w, okay| {
                if deleted(w, h) {
                    return;
                }
                if okay && access::is_friend(w, h, from) {
                    if let Some(l) = live_list_mut(w, h, ListKind::Bans) {
                        l.clear();
                    }
                    msg(w, from, 1060754);
                }
                display_aos(w, from, h, Page::Security);
            }),
            10 => page(w, Page::ListAccess),
            11 => warning_gump(w, from, RevokeAccessWarning, move |w, okay| {
                clear_access(w, from, okay, h)
            }),
            12 => {
                if is_owner {
                    let d = house::d(w, h);
                    if !d.player_vendors.is_empty() {
                        notice(w, from, CannotConvertPrivateNoticeGump, move |w| {
                            public_private_notice(w, from, h)
                        });
                        return;
                    }
                    if !d.vendor_rental_contracts.is_empty() {
                        notice(w, from, CannotPerformActionContractsNoticeGump, move |w| {
                            public_private_notice(w, from, h)
                        });
                        return;
                    }
                    access::set_public(w, h, false);
                    doors::change_locks(w, h, from);
                    notice(w, from, HouseConvertedPrivateNoticeGump, move |w| {
                        public_private_notice(w, from, h)
                    });
                    for m in house::storage::get_mobiles(w, h) {
                        if !access::has_access(w, h, m) && access::is_inside_mobile(w, h, m) {
                            let ban = house::ban_location(w, h);
                            muo_server::mobile::part1::lifecycle::set_location(w, m, ban);
                        }
                    }
                }
            }
            13 if is_owner => {
                access::set_public(w, h, true);
                doors::remove_keys(w, h, from);
                doors::remove_locks(w, h);
                if house::new_vendor_system(w) {
                    notice(w, from, HouseConvertedPublicNoticeGump, move |w| {
                        public_private_notice(w, from, h)
                    });
                } else {
                    notice(w, from, HouseConvertedPublicOldSystemNoticeGump, move |w| {
                        public_private_notice(w, from, h)
                    });
                }
                for m in house::storage::get_mobiles(w, h) {
                    if access::is_banned(w, h, m) && access::is_inside_mobile(w, h, m) {
                        let ban = house::ban_location(w, h);
                        muo_server::mobile::part1::lifecycle::set_location(w, m, ban);
                    }
                }
            }
            _ => {}
        }
    }

    fn customize(
        &mut self,
        w: &mut World,
        from: MobileId,
        h: ItemId,
        index: i32,
        roles: (bool, bool),
        foundation: bool,
    ) {
        let (is_owner, is_co_owner) = roles;
        let page = |w: &mut World, p: Page| display_aos(w, from, h, p);
        match index {
            0 => {
                if is_owner && !foundation {
                    if house::decay::rented_vendors(w, h) {
                        notice(w, from, CannotPerformActionVendorsNoticeGump, move |w| {
                            customize_notice(w, from, h)
                        });
                    } else if house::convert_entry(w, h).is_some() {
                        warning_gump(w, from, ConvertToCustomHouseWarningGump, move |w, okay| {
                            convert_house(w, from, okay, h)
                        });
                    }
                }
            }
            1 => {
                if is_owner && foundation {
                    if house::decay::rented_vendors(w, h) {
                        notice(w, from, CannotPerformActionVendorsNoticeGump, move |w| {
                            customize_notice(w, from, h)
                        });
                    } else if hooks(w)
                        .and_then(|x| x.has_addon_containers)
                        .is_some_and(|f| f(w, h))
                    {
                        notice(w, from, CannotCustomizeAddonsNoticeGump, move |w| {
                            customize_notice(w, from, h)
                        });
                    } else if let Some(f) = hooks(w).and_then(|x| x.foundation_begin_customize) {
                        f(w, h, from);
                    }
                }
            }
            2 => {
                let crate_ = house::d(w, h).moving_crate;
                if let (true, Some(c)) = (is_owner, crate_) {
                    if !access::is_inside_mobile(w, h, from) {
                        msg(w, from, 502092); // You must be in your house to do this.
                    } else {
                        let (loc, map) = (
                            muo_content::mx::location(w, from),
                            muo_content::mx::map(w, from),
                        );
                        muo_server::item::ops::move_to_world_map(w, c, loc, map);
                        muo_multis::moving_crate::restart_timer(w, c);
                    }
                }
            }
            3 => {
                if is_owner && house::data(w, h).is_some_and(|d| d.public) {
                    page(w, Page::ChangeSign);
                }
            }
            4 => {
                if is_owner && foundation {
                    page(w, Page::ChangeHanger);
                }
            }
            5 => {
                if is_owner
                    && foundation
                    && hooks(w)
                        .and_then(|x| x.foundation_has_signpost)
                        .is_some_and(|f| f(w, h))
                {
                    page(w, Page::ChangePost);
                }
            }
            6 => {
                if is_owner && foundation {
                    page(w, Page::ChangeFoundation);
                }
            }
            7 if is_co_owner => {
                rename_prompt(w, from, h);
                msg(w, from, 501302);
            }
            _ => {}
        }
    }
}

fn clear_access(w: &mut World, from: MobileId, okay: bool, h: ItemId) {
    if deleted(w, h) {
        return;
    }
    if okay && access::is_friend(w, h, from) {
        let list = live_list(w, h, ListKind::Access);
        if let Some(l) = live_list_mut(w, h, ListKind::Access) {
            l.clear();
        }
        for m in list {
            if !access::has_access(w, h, m) && access::is_inside_mobile(w, h, m) {
                let ban = house::ban_location(w, h);
                muo_server::mobile::part1::lifecycle::set_location(w, m, ban);
                msg(w, m, 1060734); // Your access to this house has been revoked.
            }
        }
        msg(w, from, 1061843);
    }
    display_aos(w, from, h, Page::Security);
}

/// `ConvertHouse_Callback`.
fn convert_house(w: &mut World, from: MobileId, okay: bool, h: ItemId) {
    if deleted(w, h) {
        return;
    }
    if okay && access::is_owner(w, h, from) && !house::decay::rented_vendors(w, h) {
        let Some(e) = house::convert_entry(w, h) else {
            return;
        };
        let price = house::d(w, h).price;
        let cost = e.cost - price;
        if cost > 0 {
            if muo_content::vendor::banker::withdraw(w, from, cost) {
                muo_content::mx::msg_args(w, from, 1060398, &cost.to_string());
            } else {
                msg(w, from, 1061624);
                return;
            }
        } else if cost < 0 {
            if muo_content::vendor::banker::deposit(w, from, -cost, true) {
                muo_content::mx::msg_args(w, from, 1060397, &(-cost).to_string());
            } else {
                return;
            }
        }
        if let Some(nh) = muo_multis::placement_tool::construct_house(w, e, from) {
            if let Some(d) = house::data_mut(w, nh) {
                d.price = e.cost;
            }
            house::storage::move_all_to_crate(w, h);
            let old = house::d(w, h);
            if let Some(d) = house::data_mut(w, nh) {
                d.friends = old.friends.clone();
                d.co_owners = old.co_owners.clone();
                d.bans = old.bans.clone();
                d.access = old.access.clone();
                d.built_on = old.built_on;
                d.last_traded = old.last_traded;
                d.public = old.public;
                d.vendor_inventories
                    .extend(old.vendor_inventories.iter().cloned());
                d.internalized_vendors
                    .extend(old.internalized_vendors.iter().cloned());
                if old.moving_crate.is_some() {
                    d.moving_crate = old.moving_crate;
                }
            }
            if let Some(d) = house::data_mut(w, h) {
                d.vendor_inventories.clear();
                d.internalized_vendors.clear();
                d.moving_crate = None;
            }
            // `inventory.House = newHouse`, `vendor.House = newHouse`
            muo_multis::vendor_inventory::move_to_house(w, nh);
            if let Some(set) = hooks(w).and_then(|h| h.set_vendor_house) {
                let vendors = house::d(w, nh).internalized_vendors.clone();
                for v in vendors {
                    set(w, v, Some(nh));
                }
            }
            if let Some(c) = old.moving_crate
                && let Some(data) = w
                    .item_mut(c)
                    .and_then(|i| i.ext.get_mut::<muo_multis::moving_crate::MovingCrateData>())
            {
                data.house = Some(nh);
            }
            let items = house::storage::get_items(w, h);
            let mobiles = house::storage::get_mobiles(w, h);
            let loc = house::location(w, h);
            let (ox, oy, oz) = house::class(w, h).convert_offset;
            let map = house::map_of(w, h);
            muo_server::item::ops::move_to_world_map(
                w,
                nh,
                muo_primitives::Point3D::new(loc.x + ox, loc.y + oy, loc.z + oz),
                map,
            );
            muo_server::item::base::delete(w, h);
            let ban = house::ban_location(w, nh);
            for i in items {
                muo_server::item::ops::set_location(w, i, ban);
            }
            for m in mobiles {
                muo_server::mobile::part1::lifecycle::set_location(w, m, ban);
            }
            send_notice_replace(w, from);
            return;
        }
    }
    display_aos(w, from, h, Page::Security);
}

fn send_notice_replace(w: &mut World, from: MobileId) {
    let g = warning::new_static_notice(ReplaceHouseNoticeGump, None);
    send(w, from, g);
}

/// `ConfirmResizeHouseGump`'s `gumps.Close<HouseGumpAOS>()`.
pub fn close_aos(w: &mut World, ns: NetStateId) {
    close_gump::<HouseGumpAos>(w, ns);
}

/// `HouseGump.DisplayTo` / `HouseGumpAOS.DisplayTo(.., page)` picked by `IsAosRules` (the `display_house_gump` hook).
pub fn display_house_gump(
    w: &mut World,
    m: MobileId,
    h: ItemId,
    page: muo_content::items::house_deps::HouseGumpPage,
) {
    use muo_content::items::house_deps::HouseGumpPage as P;
    if house::is_aos_rules(w, h) {
        display_aos(
            w,
            m,
            h,
            match page {
                P::Information => Page::Information,
                P::Vendors => Page::Vendors,
            },
        );
    } else {
        display_to(w, m, h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `RenamePrompt.OnResponse` says "Sign changed." to a friend whether or not the house has a sign.
    #[test]
    fn rename_prompt_answers_a_friend_of_a_house_without_a_sign() {
        let mut w = World::new();
        let m = w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE);
        let ns = NetStateId(1);
        let mut d = muo_server::net_state::NetStateData::new(0);
        d.mobile = Some(m);
        w.net_states.insert(ns, d);
        w.mobile_mut(m).unwrap().net_state = Some(ns);
        let h = muo_multis::house::save::new_base_house(
            &mut w,
            &muo_multis::house::BASE_HOUSE,
            0x64,
            Some(m),
            0,
            0,
        );
        assert!(house::sign(&w, h).is_none());
        rename_prompt(&mut w, m, h);
        let id = w.mobile(m).unwrap().prompt.unwrap();
        muo_server::prompt::on_response(&mut w, id, m, "New name");
        let said = |w: &World| {
            let want: Vec<u8> = "Sign changed."
                .encode_utf16()
                .flat_map(u16::to_be_bytes)
                .collect();
            w.net_out.iter().any(|o| matches!(o,
                muo_server::net_state::NetOut::Send(_, b) if b.windows(want.len()).any(|x| x == want.as_slice())))
        };
        assert!(said(&w));
    }
}
