//! `AcceptHouseTransferGump`, `ConfirmDemolishHouseGump`, `ConfirmResizeHouseGump` and `SetSecureLevelGump`. The
//! hooks `muo_multis` calls them through are set in [`configure`].
//!

use muo_content::gumps::base::{
    DynamicGumpLogic, DynamicGumpObj, RelayInfo, StaticGumpLogic, StaticGumpObj,
};
use muo_content::gumps::builder::{DynamicGumpBuilder, StaticGumpBuilder};
use muo_content::items::house_deps::{deps_mut, securable_level};
use muo_content::misc::guild_class as gc;
use muo_multis::house::{self, SecureLevel};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;

use super::common::{mobile_of, send};
use super::hooks::hooks;

/// `Configure`: the hooks the house code opens these gumps through.
pub fn configure(w: &mut World) {
    let d = deps_mut(w);
    d.accept_house_transfer_gump = Some(|w, to, from, house| {
        send(
            w,
            to,
            StaticGumpObj::new(
                110,
                100,
                AcceptHouseTransferGump {
                    transferor: from,
                    house,
                },
            ),
        );
    });
    d.set_secure_level_gump = Some(|w, m, house, item| {
        let level = securable_level(w, item).unwrap_or(0);
        let gump = SetSecureLevelGump::new(w, house, item, SecureLevel::from_u8(level));
        send(w, m, DynamicGumpObj::new(50, 50, gump));
    });
    d.display_house_gump = Some(super::house_gump::display_house_gump);
    d.warning_gump = Some(|w, to, spec, cb| {
        // the callback runs once, on the first answer
        let cb = std::cell::RefCell::new(Some(cb));
        let cb = Some(std::rc::Rc::new(move |w: &mut World, okay: bool| {
            if let Some(f) = cb.borrow_mut().take() {
                f(w, okay);
            }
        }) as crate::m_z::warning::WarningCallback);
        // the static gump's name and size are type-level constants: one marker type per gump class
        macro_rules! go {
            ($n:ty, $w:literal, $h:literal) => {
                send(
                    w,
                    to,
                    crate::m_z::warning::new_static(
                        GenericWarning::<$n, $w, $h>(spec, std::marker::PhantomData),
                        cb,
                    ),
                )
            };
        }
        match spec.name {
            "Server.Items.HousePlacementEntry+CondemnWarningGump" => go!(CondemnName, 420, 280),
            "Server.Items.HousePlacementEntry+HousePlacementTimeoutNoticeGump" => {
                go!(TimeoutName, 320, 180)
            }
            "Server.Multis.BaseHouse+TradeHouseWarningGump" => go!(TradeName, 420, 280),
            "Server.Multis.HouseSign+ClaimHouseWarningGump" => go!(ClaimName, 420, 280),
            // every caller passes one of the four subclasses' names
            other => debug_assert!(false, "unknown house warning gump {other}"),
        }
    });
    d.recently_customized_warning = Some(|w, m| {
        send(
            w,
            m,
            crate::m_z::warning::new_static(RecentlyCustomizedWarningGump, None),
        );
    });
    d.has_recently_customized_warning = Some(|w, m| {
        w.mobile(m).and_then(|x| x.net_state).is_some_and(|ns| {
            muo_content::gumps::has_gump::<
                crate::m_z::warning::StaticWarningGump<RecentlyCustomizedWarningGump>,
            >(w, ns)
        })
    });
    let a = super::hooks::hooks_mut(w);
    a.has_addon_containers = Some(muo_multis::house::vendors::has_addon_containers);
    a.foundation_begin_customize = Some(muo_multis::foundation::begin_customize);
    a.foundation_has_signpost = Some(muo_multis::foundation::has_signpost);
    a.foundation_set_sign_hanger = Some(muo_multis::foundation::set_sign_hanger_graphic);
    a.foundation_set_type = Some(muo_multis::foundation::set_foundation_type);
    a.foundation_set_signpost_graphic = Some(muo_multis::foundation::set_signpost_graphic);
    let d = deps_mut(w);
    d.confirm_resize_house_gump = Some(|w, m, house| {
        if let Some(ns) = w.mobile(m).and_then(|x| x.net_state) {
            super::house_gump::close_aos(w, ns);
        }
        send(
            w,
            m,
            StaticGumpObj::new(110, 100, ConfirmResizeHouseGump { house }),
        );
    });
}

/// The class name of a house warning gump (`WarningGump::name`).
trait WarnName: 'static {
    const NAME: &'static str;
}

macro_rules! warn_names {
    ($($t:ident = $n:literal),*) => {$(
        struct $t;
        impl WarnName for $t {
            const NAME: &'static str = $n;
        }
    )*};
}
warn_names!(
    CondemnName = "Server.Items.HousePlacementEntry+CondemnWarningGump",
    TimeoutName = "Server.Items.HousePlacementEntry+HousePlacementTimeoutNoticeGump",
    TradeName = "Server.Multis.BaseHouse+TradeHouseWarningGump",
    ClaimName = "Server.Multis.HouseSign+ClaimHouseWarningGump"
);

/// A `StaticWarningGump<T>` whose numbers come from the caller (`HouseDeps::warning_gump`).
struct GenericWarning<N: WarnName, const W: i32, const H: i32>(
    muo_content::items::house_deps::WarningGump,
    std::marker::PhantomData<N>,
);

impl<N: WarnName, const W: i32, const H: i32> crate::m_z::warning::StaticWarning
    for GenericWarning<N, W, H>
{
    const TYPE_NAME: &'static str = N::NAME;
    const WIDTH: i32 = W;
    const HEIGHT: i32 = H;
    fn header(&self) -> i32 {
        self.0.header
    }
    fn header_color(&self) -> i32 {
        self.0.header_color
    }
    fn static_localized_content(&self) -> i32 {
        self.0.content
    }
}

/// `HouseRegion.RecentlyCustomizedWarningGump`.
struct RecentlyCustomizedWarningGump;

impl crate::m_z::warning::StaticWarning for RecentlyCustomizedWarningGump {
    const TYPE_NAME: &'static str = "Server.Regions.HouseRegion+RecentlyCustomizedWarningGump";
    const WIDTH: i32 = 320;
    const HEIGHT: i32 = 180;
    fn cancel_button(&self) -> bool {
        false
    }
    fn static_localized_content(&self) -> i32 {
        1061826
    }
}

// ---- AcceptHouseTransferGump ----

pub struct AcceptHouseTransferGump {
    transferor: MobileId,
    house: ItemId,
}

impl StaticGumpLogic for AcceptHouseTransferGump {
    const TYPE_NAME: &'static str = "Server.Gumps.AcceptHouseTransferGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.set_no_close();
        b.add_page(0);
        b.add_background(0, 0, 420, 280, 5054);
        b.add_image_tiled(10, 10, 400, 20, 2624);
        b.add_alpha_region(10, 10, 400, 20);
        b.add_html_localized_color(10, 10, 400, 20, 1060635, 30720, false, false); // <CENTER>WARNING</CENTER>
        b.add_image_tiled(10, 40, 400, 200, 2624);
        b.add_alpha_region(10, 40, 400, 200);
        // Another player is attempting to initiate a house trade with you. (1062086)
        b.add_html_localized_color(10, 40, 400, 200, 1062086, 32512, false, true);
        b.add_image_tiled(10, 250, 400, 20, 2624);
        b.add_alpha_region(10, 250, 400, 20);
        b.add_button(10, 250, 4005, 4007, 1);
        b.add_html_localized_color(40, 250, 170, 20, 1011036, 32767, false, false); // OKAY
        b.add_button(210, 250, 4005, 4007, 0);
        b.add_html_localized_color(240, 250, 170, 20, 1011012, 32767, false, false); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if info.button_id == 1
            && !house::deleted(w, self.house)
            && let Some(to) = mobile_of(w, sender)
        {
            house::transfer::end_confirm_transfer(w, self.house, self.transferor, to);
        }
    }
}

// ---- SetSecureLevelGump ----

pub struct SetSecureLevelGump {
    house: ItemId,
    /// The `ISecurable`.
    item: ItemId,
    level: SecureLevel,
    owner_name: String,
    new_guild_system: bool,
    /// `houseOwner?.Guild is Guild guild && guild.Leader == houseOwner`.
    owner_leads_guild: bool,
}

impl SetSecureLevelGump {
    pub fn new(w: &World, house: ItemId, item: ItemId, level: SecureLevel) -> Self {
        let owner = house::owner(w, house);
        Self {
            house,
            item,
            level,
            owner_name: house::decay::owner_name(w, house).unwrap_or_default(),
            new_guild_system: muo_content::misc::guild::new_guild_system(w),
            owner_leads_guild: owner.is_some_and(|o| {
                w.mobile(o)
                    .and_then(|m| m.guild)
                    .is_some_and(|g| gc::data(w, g).is_some_and(|d| d.leader == Some(o)))
            }),
        }
    }

    fn color(&self, level: SecureLevel) -> i32 {
        if self.level == level { 0x7F18 } else { 0x7FFF }
    }

    fn first_id(&self, level: SecureLevel) -> i32 {
        if self.level == level { 4006 } else { 4005 }
    }
}

impl DynamicGumpLogic for SetSecureLevelGump {
    const TYPE_NAME: &'static str = "Server.Gumps.SetSecureLevelGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);

        let offset = if self.new_guild_system { 20 } else { 0 };
        b.add_background(0, 0, 220, 160 + offset, 5054);
        b.add_image_tiled(10, 10, 200, 20, 5124);
        b.add_image_tiled(10, 40, 200, 20, 5124);
        b.add_image_tiled(10, 70, 200, 80 + offset, 5124);
        b.add_alpha_region(10, 10, 200, 140);

        b.add_html_localized_color(10, 10, 200, 20, 1061276, 32767, false, false); // <CENTER>SET ACCESS</CENTER>
        b.add_html_localized_color(10, 40, 100, 20, 1041474, 32767, false, false); // Owner:
        b.add_label(110, 40, 1152, &self.owner_name);

        b.add_button(10, 70, self.first_id(SecureLevel::Owner), 4007, 1);
        b.add_html_localized_color(
            45,
            70,
            150,
            20,
            1061277,
            self.color(SecureLevel::Owner),
            false,
            false,
        ); // Owner Only

        b.add_button(10, 90, self.first_id(SecureLevel::CoOwners), 4007, 2);
        b.add_html_localized_color(
            45,
            90,
            150,
            20,
            1061278,
            self.color(SecureLevel::CoOwners),
            false,
            false,
        ); // Co-Owners

        b.add_button(10, 110, self.first_id(SecureLevel::Friends), 4007, 3);
        b.add_html_localized_color(
            45,
            110,
            150,
            20,
            1061279,
            self.color(SecureLevel::Friends),
            false,
            false,
        ); // Friends

        // Only the actual House owner AND guild master can set guild secures
        if self.new_guild_system && self.owner_leads_guild {
            b.add_button(10, 130, self.first_id(SecureLevel::Guild), 4007, 5);
            b.add_html_localized_color(
                45,
                130,
                150,
                20,
                1063455,
                self.color(SecureLevel::Guild),
                false,
                false,
            ); // Guild Members
        }

        b.add_button(
            10,
            130 + offset,
            self.first_id(SecureLevel::Anyone),
            4007,
            4,
        );
        b.add_html_localized_color(
            45,
            130 + offset,
            150,
            20,
            1061626,
            self.color(SecureLevel::Anyone),
            false,
            false,
        ); // Anyone
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(from) = mobile_of(w, sender) else {
            return;
        };
        let current = securable_level(w, self.item).map_or(self.level, SecureLevel::from_u8);
        let level = match info.button_id {
            1 => SecureLevel::Owner,
            2 => SecureLevel::CoOwners,
            3 => SecureLevel::Friends,
            4 => SecureLevel::Anyone,
            5 => SecureLevel::Guild,
            _ => current,
        };

        if current == level {
            muo_content::mx::msg(w, from, 1061281); // Access level unchanged.
        } else {
            if let Some(f) = hooks(w).and_then(|h| h.securable_set_level) {
                f(w, self.item, level as u8);
            }
            muo_content::mx::msg(w, from, 1061280); // New access level set.
        }
        let _ = self.house;
    }
}

// ---- ConfirmDemolishHouseGump / ConfirmResizeHouseGump ----

pub struct ConfirmDemolishHouseGump {
    house: ItemId,
}

#[cfg(test)]
impl ConfirmDemolishHouseGump {
    pub fn for_test() -> Self {
        Self {
            house: ItemId(muo_primitives::Serial::new(1)),
        }
    }
}

/// `new ConfirmDemolishHouseGump(house)`.
pub fn confirm_demolish(house: ItemId) -> StaticGumpObj<ConfirmDemolishHouseGump> {
    StaticGumpObj::new(110, 100, ConfirmDemolishHouseGump { house })
}

pub struct ConfirmResizeHouseGump {
    house: ItemId,
}

/// `#,0`.
fn thousands(n: i32) -> String {
    let s = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// The checks both gumps make before refunding. `resize` picks the messages and their order. Returns true
/// when the house may be demolished.
fn vendor_checks(w: &mut World, from: MobileId, h: ItemId, resize: bool) -> bool {
    let d = house::data(w, h);
    let crate_or_vendors =
        d.is_some_and(|d| d.moving_crate.is_some() || !d.internalized_vendors.is_empty());
    if crate_or_vendors {
        if resize {
            // You can not resize your house at this time. Please remove all items fom the moving crate and try again.
            muo_content::mx::msg(w, from, 1080455);
        }
        return false;
    }
    if !muo_content::misc::guild::new_guild_system(w)
        && house::storage::find_guildstone(w, h).is_some()
    {
        muo_content::mx::msg(w, from, 501389); // You cannot redeed a house with a guildstone inside.
        return false;
    }
    let rented = house::decay::rented_vendors(w, h);
    let inventories = house::data(w, h).is_some_and(|d| !d.vendor_inventories.is_empty());
    if rented && inventories {
        // You cannot do that that while you still have contract vendors or unclaimed contract vendor inventory in your house.
        muo_content::mx::msg(w, from, 1062679);
        return false;
    }
    if rented {
        muo_content::mx::msg(w, from, 1062680); // You cannot do that that while you still have contract vendors in your house.
        return false;
    }
    if inventories {
        // You cannot do that that while you still have unclaimed contract vendor inventory in your house.
        muo_content::mx::msg(w, from, 1062681);
        return false;
    }
    true
}

fn price(w: &World, h: ItemId) -> i32 {
    house::data(w, h).map_or(0, |d| d.price)
}

/// Drops the deed in the bank box; false (message sent) when it doesn't fit.
fn give_deed(w: &mut World, from: MobileId, deed: ItemId) -> bool {
    let bank = muo_server::mobile::part1::props_b::bank_box(w, from);
    let ok = bank
        .is_some_and(|b| muo_server::item::container::try_drop_item(w, b, Some(from), deed, false));
    if !ok {
        muo_server::item::base::delete(w, deed);
        muo_content::mx::msg(w, from, 500390); // Your bank box is full.
    }
    ok
}

impl StaticGumpLogic for ConfirmDemolishHouseGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ConfirmDemolishHouseGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.set_no_close();
        b.add_page(0);
        b.add_background(0, 0, 420, 280, 5054);
        b.add_image_tiled(10, 10, 400, 20, 2624);
        b.add_alpha_region(10, 10, 400, 20);
        b.add_html_localized_color(10, 10, 400, 20, 1060635, 30720, false, false); // <CENTER>WARNING</CENTER>
        b.add_image_tiled(10, 40, 400, 200, 2624);
        b.add_alpha_region(10, 40, 400, 200);
        // You are about to demolish your house. (1061795)
        b.add_html_localized_color(10, 40, 400, 200, 1061795, 32512, false, true);
        b.add_image_tiled(10, 250, 400, 20, 2624);
        b.add_alpha_region(10, 250, 400, 20);
        b.add_button(10, 250, 4005, 4007, 1);
        b.add_html_localized_color(40, 250, 170, 20, 1011036, 32767, false, false); // OKAY
        b.add_button(210, 250, 4005, 4007, 0);
        b.add_html_localized_color(240, 250, 170, 20, 1011012, 32767, false, false); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let h = self.house;
        if info.button_id != 1 || house::deleted(w, h) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };

        if !house::access::is_owner(w, h, from) {
            muo_content::mx::msg(w, from, 501320); // Only the house owner may do this.
            return;
        }
        if !vendor_checks(w, from, h, false) {
            return;
        }

        if muo_content::mx::access_level(w, from) >= AccessLevel::GameMaster {
            muo_content::mx::send_message(
                w,
                from,
                "You do not get a refund for your house as you are not a player",
            );
        } else if house::is_aos_rules(w, h) && price(w, h) > 0 {
            let amount = price(w, h);
            if muo_content::vendor::banker::deposit(w, from, amount, true) {
                // ~1_AMOUNT~ gold has been deposited into your bank box.
                muo_content::mx::msg_args(w, from, 1060397, &thousands(amount));
            } else {
                muo_content::mx::msg(w, from, 500390); // Your bank box is full.
                return;
            }
        } else {
            let get_deed = house::class(w, h).get_deed;
            let Some(deed) = get_deed(w, h) else {
                muo_content::mx::send_message(w, from, "Unable to refund house.");
                return;
            };
            if !give_deed(w, from, deed) {
                return;
            }
        }

        house::doors::remove_keys(w, h, from);
        muo_server::item::base::delete(w, h);
    }
}

impl StaticGumpLogic for ConfirmResizeHouseGump {
    const TYPE_NAME: &'static str = "Server.Gumps.ConfirmResizeHouseGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.set_no_close();
        b.add_page(0);
        b.add_background(0, 0, 420, 280, 0x13BE);
        b.add_image_tiled(10, 10, 400, 20, 0xA40);
        b.add_alpha_region(10, 10, 400, 20);
        b.add_html_localized_color(10, 10, 400, 20, 1060635, 0x7800, false, false); // <CENTER>WARNING</CENTER>
        b.add_image_tiled(10, 40, 400, 200, 0xA40);
        b.add_alpha_region(10, 40, 400, 200);
        // You are attempting to resize your house. (1080196)
        b.add_html_localized_color(10, 40, 400, 200, 1080196, 0x7F00, false, true);
        b.add_image_tiled(10, 250, 400, 20, 0xA40);
        b.add_alpha_region(10, 250, 400, 20);
        b.add_button(10, 250, 0xFA5, 0xFA7, 1);
        b.add_button(210, 250, 0xFA5, 0xFA7, 0);
        b.add_html_localized_color(40, 250, 170, 20, 1011036, 0x7FFF, false, false); // OKAY
        b.add_html_localized_color(240, 250, 170, 20, 1011012, 0x7FFF, false, false); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let h = self.house;
        if info.button_id > 1 || house::deleted(w, h) {
            return;
        }
        let Some(from) = mobile_of(w, sender) else {
            return;
        };

        if info.button_id == 0 {
            super::house_gump::display_aos(w, from, h, super::house_gump::Page::Customize);
            return;
        }

        if !house::access::is_owner(w, h, from) {
            muo_content::mx::msg(w, from, 501320); // Only the house owner may do this.
            return;
        }
        if !vendor_checks(w, from, h, true) {
            return;
        }

        if muo_content::mx::access_level(w, from) > AccessLevel::Player {
            muo_content::mx::send_message(
                w,
                from,
                "You do not get a refund for your house as you are not a player.",
            );
        } else {
            let amount = price(w, h);
            let to_give = if !house::is_aos_rules(w, h) || amount <= 0 {
                let get_deed = house::class(w, h).get_deed;
                get_deed(w, h)
            } else {
                None
            };
            if let Some(deed) = to_give
                && !give_deed(w, from, deed)
            {
                return;
            }
            if amount <= 0 || !muo_content::vendor::banker::deposit(w, from, amount, true) {
                muo_content::mx::send_message(w, from, "Unable to refund house.");
                return;
            }
            // ~1_AMOUNT~ gold has been deposited into your bank box.
            muo_content::mx::msg_args(w, from, 1060397, &amount.to_string());
        }

        house::doors::remove_keys(w, h, from);
        house::decay::new_temp_no_housing_region(w, h, Some(from));
        muo_server::item::base::delete(w, h);
    }
}

#[cfg(test)]
mod warning_name_tests {
    use super::*;
    use crate::m_z::warning::StaticWarning;

    #[test]
    fn house_warning_gumps_carry_their_class_names() {
        // The name is hashed into the gump type id of the 0xDD packet.
        type G = GenericWarning<CondemnName, 420, 280>;
        assert_eq!(
            <G as StaticWarning>::TYPE_NAME,
            "Server.Items.HousePlacementEntry+CondemnWarningGump"
        );
        assert_eq!(<G as StaticWarning>::WIDTH, 420);
        assert_eq!(
            <GenericWarning<TimeoutName, 320, 180> as StaticWarning>::HEIGHT,
            180
        );
    }
}
