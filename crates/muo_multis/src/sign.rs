//! `HouseSign`.
//!
//! Note: its `LabelNumber => 1061638` does not `override` (it hides `Item.LabelNumber`), so the sign
//! keeps the item's label and the number is not used.

use std::rc::Rc;

use muo_content::items::house_deps::{HouseGumpPage, WarningGump, deps};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::mobile::AccessLevel;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

use crate::house::access::{self, msg};
use crate::house::decay::{self, decay_enabled};
use crate::house::{self, DecayLevel};

/// What `HouseSign` adds.
#[derive(Clone, Debug, Default)]
pub struct HouseSignData {
    pub owner: Option<ItemId>,
    pub original_owner: Option<MobileId>,
    /// `GettingProperties`: set while `GetProperties` reads the house's decay level (a `Cell`: properties are built
    /// from `&World`).
    pub getting_properties: std::cell::Cell<bool>,
}

/// `Sign.GettingProperties`.
pub fn getting_properties(w: &World, sign: ItemId) -> bool {
    w.item(sign)
        .and_then(|i| i.ext.get::<HouseSignData>())
        .is_some_and(|d| d.getting_properties.get())
}

fn sign_data(w: &World, id: ItemId) -> HouseSignData {
    w.item(id)
        .and_then(|i| i.ext.get::<HouseSignData>())
        .cloned()
        .unwrap_or_default()
}

/// `HouseSign.Owner`.
pub fn house_of(w: &World, sign: ItemId) -> Option<ItemId> {
    sign_data(w, sign).owner
}

/// `new HouseSign(owner)`.
pub fn new_house_sign(w: &mut World, house: ItemId) -> ItemId {
    let id = ops::new_item_with_id(w, &HOUSE_SIGN, 0xBD2);
    let original = house::owner(w, house);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(HouseSignData {
            owner: Some(house),
            original_owner: original,
            getting_properties: Default::default(),
        });
    }
    ops::set_movable(w, id, false);
    id
}

/// `GetName()`.
pub fn get_name(w: &World, sign: ItemId) -> String {
    ops::name(w, sign).unwrap_or_else(|| "An Unnamed House".to_string())
}

/// `RestrictDecay` getter and setter live on the house; the sign forwards.
pub fn restrict_decay(w: &World, sign: ItemId) -> bool {
    house_of(w, sign).is_some_and(|h| house::d(w, h).restrict_decay)
}

fn set_getting_properties(w: &World, sign: ItemId, v: bool) {
    if let Some(d) = w.item(sign).and_then(|i| i.ext.get::<HouseSignData>()) {
        d.getting_properties.set(v);
    }
}

fn get_properties(w: &World, id: ItemId, list: &mut muo_server::property_list::PropertyList) {
    (ITEM_BASE.get_properties)(w, id, list);
    let name = muo_primitives::utility::fix_html(&get_name(w, id));
    list.add_str(1061639, &name); // Name: ~1_NAME~
    let owner = house_of(w, id)
        .and_then(|h| decay::owner_name(w, h))
        .unwrap_or_else(|| "nobody".to_string());
    list.add_str(1061640, &owner); // Owner: ~1_OWNER~

    if let Some(h) = house_of(w, id) {
        // This House is Open to the Public : This is a Private Home
        list.add(if house::d(w, h).public {
            1061641
        } else {
            1061642
        });
        set_getting_properties(w, id, true);
        let mut level = decay::decay_level(w, h);
        set_getting_properties(w, id, false);
        if level == DecayLevel::DemolitionPending {
            list.add(1062497); // Demolition Pending
        } else if level != DecayLevel::Ageless {
            if level == DecayLevel::Collapsed {
                level = DecayLevel::IDOC;
            }
            list.add_localized_number(1062028, 1043009 + level as i32); // Condition: This structure is ...
        }
    }
}

fn on_single_click(w: &mut World, id: ItemId, from: MobileId) {
    if let Some(h) = house_of(w, id).filter(|_| decay_enabled(w))
        && house::decay_period(w, h).ticks() != 0
    {
        let message = match decay::decay_level(w, h) {
            DecayLevel::Ageless => "ageless",
            DecayLevel::Fairly => "fairly worn",
            DecayLevel::Greatly => "greatly worn",
            DecayLevel::LikeNew => "like new",
            DecayLevel::Slightly => "slightly worn",
            DecayLevel::Somewhat => "somewhat worn",
            _ => "in danger of collapsing",
        };
        ops::label_to(w, id, from, &format!("This house is {message}."));
    }
    (ITEM_BASE.on_single_click)(w, id, from);
}

/// `ShowSign(m)`.
pub fn show_sign(w: &mut World, sign: ItemId, m: MobileId) {
    let Some(h) = house_of(w, sign) else { return };
    let gm = w
        .mobile(m)
        .is_some_and(|x| x.access_level >= AccessLevel::GameMaster);
    if access::is_friend(w, h, m) && !gm {
        if (muo_content::core::ml(w) && access::is_owner(w, h, m)) || !muo_content::core::ml(w) {
            decay::refresh_decay(w, h);
        }
        if !muo_content::core::aos(w) {
            msg(w, m, 501293); // Welcome back to the house, friend!
        }
    }
    if let Some(f) = deps(w).and_then(|x| x.display_house_gump) {
        f(w, m, h, HouseGumpPage::Information);
    }
}

/// `ClaimGump_Callback(from, okay)`.
fn claim_callback(w: &mut World, sign: ItemId, from: MobileId, okay: bool) {
    if okay
        && let Some(h) = house_of(w, sign)
        && house::owner(w, h).is_none()
        && decay::decay_level(w, h) != DecayLevel::DemolitionPending
    {
        let d = house::d(w, h);
        let can_claim = (!d.co_owners.is_empty() && access::is_co_owner(w, h, from))
            || access::is_friend(w, h, from);
        if can_claim && !access::has_account_house(w, from) {
            access::set_owner(w, h, Some(from));
            let now = w.now;
            if let Some(d) = house::data_mut(w, h) {
                d.last_traded = now;
            }
        }
    }
    show_sign(w, sign, from);
}

fn on_double_click(w: &mut World, id: ItemId, m: MobileId) {
    let Some(h) = house_of(w, id) else { return };
    let gm = w
        .mobile(m)
        .is_some_and(|x| x.access_level >= AccessLevel::GameMaster);
    if !gm
        && house::owner(w, h).is_none()
        && decay::decay_level(w, h) != DecayLevel::DemolitionPending
    {
        let can_claim = access::is_co_owner(w, h, m) || access::is_friend(w, h, m);
        if can_claim
            && !access::has_account_house(w, m)
            && let Some(f) = deps(w).and_then(|x| x.warning_gump)
        {
            /* You do not currently own any house on any shard with this account, and this house currently does
             * not have an owner. If you wish, you may choose to claim this house and become its rightful owner.
             * If you do this, it will become your Primary house and automatically refresh. If you claim this
             * house, you will be unable to place another house or have another house transferred to you for the
             * next 7 days. Do you wish to claim this house? */
            let gump = WarningGump {
                name: "Server.Multis.HouseSign+ClaimHouseWarningGump",
                header: 501036,
                header_color: 0x7F00,
                content: 1049719,
                width: 420,
                height: 280,
            };
            f(
                w,
                m,
                gump,
                Box::new(move |w, okay| claim_callback(w, id, m, okay)),
            );
        }
    }
    show_sign(w, id, m);
}

fn on_after_delete(w: &mut World, id: ItemId) {
    (ITEM_BASE.on_after_delete)(w, id);
    if let Some(h) = house_of(w, id).filter(|&h| !house::deleted(w, h))
        && let Some(ty) = w.item(h).map(|i| i.ty)
    {
        (ty.delete)(w, h);
    }
}

fn get_context_menu_entries(
    w: &mut World,
    id: ItemId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
    let alive = muo_server::mobile::base::alive(w, from);
    let Some(h) = house_of(w, id) else { return };
    if !house::new_vendor_system(w) || !alive || !house::is_aos_rules(w, h) {
        return;
    }
    if crate::house::vendors::are_there_available_vendors_for(w, h, from) {
        let mut e = ContextMenuEntry::new(6211, 5);
        e.on_click = Some(Rc::new(vendors_click));
        list.push(e);
    }
    if !house::d(w, h).vendor_inventories.is_empty() {
        let mut e = ContextMenuEntry::new(6213, 5);
        e.on_click = Some(Rc::new(reclaim_click));
        list.push(e);
    }
}

fn vendors_click(w: &mut World, from: MobileId, target: EntityId) {
    let EntityId::Item(sign) = target else { return };
    if !muo_server::mobile::part2::stats::check_alive(w, from, true) || house::deleted(w, sign) {
        return;
    }
    let Some(h) = house_of(w, sign) else { return };
    if !crate::house::vendors::are_there_available_vendors_for(w, h, from) {
        return;
    }
    if w.mobile(from).map(|m| m.map) != Some(house::map_of(w, sign)) {
        msg(w, from, 1062429); // You must be within five paces of the house sign to use this option.
    } else if let Some(f) = deps(w).and_then(|x| x.display_house_gump) {
        f(w, from, h, HouseGumpPage::Vendors);
    }
}

fn reclaim_click(w: &mut World, from: MobileId, target: EntityId) {
    let EntityId::Item(sign) = target else { return };
    if !muo_server::mobile::part2::stats::check_alive(w, from, true) || house::deleted(w, sign) {
        return;
    }
    let Some(h) = house_of(w, sign) else { return };
    if house::d(w, h).vendor_inventories.is_empty() {
        return;
    }
    if w.mobile(from).map(|m| m.map) != Some(house::map_of(w, sign)) {
        msg(w, from, 1062429); // You must be within five paces of the house sign to use this option.
    } else if let Some(f) = deps(w).and_then(|x| x.display_vendor_inventory_gump) {
        f(w, from, h);
    }
}

/// `Server.Multis.HouseSign`.
pub static HOUSE_SIGN: ItemType = ItemType {
    full_name: "Server.Multis.HouseSign",
    parent: Some(&ITEM_BASE),
    construct: None,
    force_show_properties: |w, _| w.core.opl_enabled,
    get_properties,
    on_single_click,
    on_double_click,
    on_after_delete,
    get_context_menu_entries,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0); // version
        let d = sign_data(w, id);
        out.write_item(w, d.owner);
        out.write_mobile(w, d.original_owner);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let owner = r.read_item(w)?.filter(|&i| house::is_house(w, i));
        let original_owner = r.read_mobile(w)?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(HouseSignData {
                owner,
                original_owner,
                getting_properties: Default::default(),
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&HOUSE_SIGN, &[]);
}
