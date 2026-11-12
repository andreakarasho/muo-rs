//! `VirtualCheckGump`: the editor of a `VirtualCheck` in a secure trade.
//!
//! A gump is built from its state, so [`refresh`] (`Refresh(true)`) closes the open one and sends a new one with the same amounts.
//!
//! `VirtualCheck.GumpActivator` is [`configure`]: it hands `muo_server::item::virtual_check` the calls that open,
//! refresh and close this gump.

use muo_content::gumps::base::{LegacyGumpLogic, LegacyGumpObj, RelayInfo};
use muo_content::gumps::{close_gump, send_gump_to_mobile};
use muo_primitives::dotnet::InvariantCase;
use muo_primitives::html::center_rgb;
use muo_primitives::parse::try_to_i32;
use muo_server::World;
use muo_server::iaccount::account_totals;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ops::is_deleted, virtual_check as vc};
use muo_server::net_state::NetStateId;

use super::common::{mobile_of, n0};

/// `VirtualCheckGump.Buttons`.
#[derive(Copy, Clone, Eq, PartialEq)]
enum Buttons {
    Close = 0,
    Clear = 1,
    Accept = 2,
    AllPlat = 3,
    AllGold = 4,
}

pub struct VirtualCheckGump {
    user: MobileId,
    /// `Check`; `None` once it was found deleted.
    check: Option<ItemId>,
    plat: i32,
    gold: i32,
}

fn check_alive(w: &World, check: Option<ItemId>) -> bool {
    check.is_some_and(|c| !is_deleted(w, c))
}

/// `(User.Account.TotalPlat, User.Account.TotalGold)`.
fn totals(w: &World, user: MobileId) -> (i32, i32) {
    account_totals(w, user).map_or((0, 0), |(gold, plat)| (plat, gold))
}

/// `new VirtualCheckGump(user, check)` with the amounts to show.
pub fn new(
    w: &World,
    user: MobileId,
    check: ItemId,
    plat: i32,
    gold: i32,
) -> LegacyGumpObj<VirtualCheckGump> {
    let mut g = LegacyGumpObj::new(
        50,
        50,
        VirtualCheckGump {
            user,
            check: Some(check),
            plat,
            gold,
        },
    );
    g.layout.closable = true;
    g.layout.disposable = true;
    g.layout.draggable = true;
    g.layout.resizable = false;

    compile_layout(w, &mut g);
    g
}

/// `CompileLayout()`.
fn compile_layout(w: &World, g: &mut LegacyGumpObj<VirtualCheckGump>) {
    if !check_alive(w, g.logic.check) {
        return;
    }
    let (user, plat_amount, gold_amount) = (g.logic.user, g.logic.plat, g.logic.gold);
    let (total_plat, total_gold) = totals(w, user);
    let raw_name = muo_server::mobile::part1::props_b::raw_name(w, user).unwrap_or_default();

    let l = &mut g.layout;
    l.reset();

    l.add_page(0);

    l.add_background(0, 0, 400, 160, 3500);

    // Title
    l.add_image_tiled(25, 35, 350, 3, 96);
    l.add_image(10, 8, 113, 0);
    l.add_image(360, 8, 113, 0);

    l.add_html(
        40,
        15,
        320,
        20,
        &center_rgb(&format!("BANK OF {}", raw_name.upper_inv()), 0x2F4F4F),
        false,
        false,
    );

    // Platinum Row
    l.add_background(15, 60, 175, 20, 9300);
    l.add_background(20, 45, 165, 30, 9350);
    l.add_item(20, 45, 3826, 0); // Plat
    l.add_label(60, 50, 0, &n0(i64::from(total_plat)));

    l.add_button(195, 50, 95, 95, Buttons::AllPlat as i32); // ->

    l.add_background(210, 60, 175, 20, 9300);
    l.add_background(215, 45, 165, 30, 9350);
    l.add_text_entry_limited(
        225,
        50,
        145,
        20,
        0,
        0,
        &plat_amount.to_string(),
        total_plat.to_string().len() as i32,
    );

    // Gold Row
    l.add_background(15, 100, 175, 20, 9300);
    l.add_background(20, 85, 165, 30, 9350);
    l.add_item(20, 85, 3823, 0); // Gold
    l.add_label(60, 90, 0, &n0(i64::from(total_gold)));

    l.add_button(195, 90, 95, 95, Buttons::AllGold as i32); // ->

    l.add_background(210, 100, 175, 20, 9300);
    l.add_background(215, 85, 165, 30, 9350);
    l.add_text_entry_limited(
        225,
        90,
        145,
        20,
        0,
        1,
        &gold_amount.to_string(),
        total_gold.to_string().len() as i32,
    );

    // Buttons
    l.add_button(20, 128, 12006, 12007, Buttons::Close as i32);
    l.add_button(215, 128, 12003, 12004, Buttons::Clear as i32);
    l.add_button(305, 128, 12000, 12002, Buttons::Accept as i32);
}

/// `Send()`: shows the gump, or closes it if the check is gone.
pub fn send(w: &mut World, user: MobileId, check: ItemId, plat: i32, gold: i32) {
    if is_deleted(w, check) {
        close(w, user, Some(check));
    } else {
        let g = new(w, user, check, plat, gold);
        send_gump_to_mobile(w, user, Box::new(g), true);
    }
}

/// `Close()`; the check is `None` where the gump already forgot it.
pub fn close(w: &mut World, user: MobileId, check: Option<ItemId>) {
    if let Some(ns) = w.mobile(user).and_then(|m| m.net_state) {
        close_gump::<VirtualCheckGump>(w, ns);
    }

    if let Some(c) = check.filter(|&c| !is_deleted(w, c)) {
        vc::update_trade(w, c, user);
    }
}

/// `Refresh(recompile)`: the layout is always made anew here.
pub fn refresh(w: &mut World, user: MobileId, check: ItemId, plat: i32, gold: i32) {
    if is_deleted(w, check) {
        close(w, user, Some(check));
        return;
    }

    close(w, user, Some(check));
    send(w, user, check, plat, gold);
}

impl LegacyGumpLogic for VirtualCheckGump {
    const TYPE_NAME: &'static str = "Server.Gumps.VirtualCheckGump";
    fn singleton(&self) -> bool {
        true
    }

    fn on_server_close(&mut self, w: &mut World, _owner: NetStateId) {
        if let Some(c) = self.check.filter(|&c| !is_deleted(w, c)) {
            vc::update_trade(w, c, self.user);
        }
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let user = self.user;
        let sender_mobile = mobile_of(w, sender);
        let Some(check) = self.check.filter(|&c| !is_deleted(w, c)) else {
            close(w, user, None);
            return;
        };
        if sender_mobile != Some(user) {
            close(w, user, Some(check));
            return;
        }

        let mut refresh_it = false;
        let mut updated = false;

        match info.button_id {
            x if x == Buttons::Clear as i32 => {
                self.plat = 0;
                self.gold = 0;
                refresh_it = true;
            }
            x if x == Buttons::Accept as i32 => {
                let plat_text = info.get_text_entry(0);
                let gold_text = info.get_text_entry(1);

                // `int.TryParse(.., out _plat)` leaves 0 behind when it fails.
                match plat_text.and_then(try_to_i32) {
                    None => {
                        self.plat = 0;
                        muo_content::mx::send_message(
                            w,
                            user,
                            "That is not a valid amount of platinum.",
                        );
                        refresh_it = true;
                    }
                    Some(p) => {
                        self.plat = p;
                        match gold_text.and_then(try_to_i32) {
                            None => {
                                self.gold = 0;
                                muo_content::mx::send_message(
                                    w,
                                    user,
                                    "That is not a valid amount of gold.",
                                );
                                refresh_it = true;
                            }
                            Some(gld) => {
                                self.gold = gld;
                                let (total_plat, total_gold) = totals(w, user);

                                if total_plat < self.plat || total_gold < self.gold {
                                    self.plat = total_plat;
                                    self.gold = total_gold;
                                    muo_content::mx::send_message(
                                        w,
                                        user,
                                        "You do not have that much currency.",
                                    );
                                    refresh_it = true;
                                } else {
                                    vc::set_plat(w, check, self.plat);
                                    vc::set_gold(w, check, self.gold);
                                    updated = true;
                                }
                            }
                        }
                    }
                }
            }
            x if x == Buttons::AllPlat as i32 => {
                self.plat = totals(w, user).0;
                refresh_it = true;
            }
            x if x == Buttons::AllGold as i32 => {
                self.gold = totals(w, user).1;
                refresh_it = true;
            }
            _ => {}
        }

        if updated {
            muo_content::mx::send_message(w, user, "Your offer has been updated.");
        }

        if refresh_it && !is_deleted(w, check) {
            refresh(w, user, check, self.plat, self.gold);
            return;
        }

        close(w, user, Some(check));
    }
}

/// `VirtualCheck.GumpActivator`: the check's double click opens this gump.
pub fn configure(w: &mut World) {
    use vc::VirtualCheckGumpHooks;
    w.resources
        .get_mut::<VirtualCheckGumpHooks>()
        .clone_from(&VirtualCheckGumpHooks {
            send: Some(|w, user, check| {
                let (p, g) = (vc::plat(w, check), vc::gold(w, check));
                send(w, user, check, p, g);
            }),
            close: Some(|w, user, check| close(w, user, Some(check))),
            refresh: Some(|w, user, check| {
                let (p, g) = (vc::plat(w, check), vc::gold(w, check));
                refresh(w, user, check, p, g);
            }),
        });
}
