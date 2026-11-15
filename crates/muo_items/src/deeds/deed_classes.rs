//! `CommodityDeed`, `NameChangeDeed`, `ClothingBlessDeed` and `NewPlayerTicket`.
//!
//! `CommodityDeedBox` is `special::boxes`; a deed is filled and redeemed through the bank box or such a box.
//! `HairRestylingDeed`, `BarkeepContract` and `DragonBardingDeed` are in `last::deeds`, `HolidayTreeDeed` in
//! `special::holiday3`.
//!
//! `VendorRentalContract` is `muo_mobiles::npcs::vendors::rental_contract`.

use muo_content::gumps::base::{RelayInfo, StaticGumpLogic, StaticGumpObj};
use muo_content::gumps::builder::StaticGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::misc::name_verification::validate_player_name;
use muo_io::GenericWriter;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, deps, ops};
use muo_server::net_state::NetStateId;
use muo_server::property_list::PropertyList;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::TypeRegistry;

use crate::commodity;

fn pack_has(w: &mut World, id: ItemId, from: MobileId) -> bool {
    let pack = muo_server::mobile::part1::props_b::backpack(w, from);
    ops::is_child_of(w, id, pack.map(EntityId::Item))
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

fn v0_write(w: &World, id: ItemId, out: &mut dyn GenericWriter, parent: &ItemType) {
    (parent.serialize)(w, id, out);
    out.write_i32(0);
}

// ---- CommodityDeed ----

/// `CommodityDeed._commodity`.
#[derive(Clone, Copy, Debug, Default)]
pub struct CommodityData {
    pub commodity: Option<ItemId>,
}

pub fn commodity_of(w: &World, id: ItemId) -> Option<ItemId> {
    w.item(id)?.ext.get::<CommodityData>()?.commodity
}

pub fn set_commodity_field(w: &mut World, id: ItemId, c: Option<ItemId>) {
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(CommodityData { commodity: c });
    }
}

/// `new CommodityDeed(commodity)`.
pub fn new_commodity_deed(w: &mut World, c: Option<ItemId>) -> ItemId {
    let id = crate::simple::from_template(w, &COMMODITY_DEED, "Server.Items.CommodityDeed");
    set_commodity_field(w, id, c);
    ops::set_loot_type(w, id, LootType::Blessed);
    id
}

/// `SetCommodity(item)`.
pub fn set_commodity(w: &mut World, id: ItemId, item: ItemId) -> bool {
    ops::invalidate_properties(w, id);
    if commodity_of(w, id).is_none() && commodity::deedable(w, item) == Some(true) {
        set_commodity_field(w, id, Some(item));
        ops::internalize(w, item);
        ops::invalidate_properties(w, id);
        return true;
    }
    false
}

fn amount_of(w: &World, item: ItemId) -> i32 {
    w.item(item).map_or(0, |i| i.amount)
}

fn commodity_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let ml = muo_content::core::ml(w);
    let bank = muo_server::mobile::part2::items::find_bank_no_create(w, from);
    let in_bank = bank.is_some_and(|b| ops::is_child_of(w, id, Some(EntityId::Item(b))));
    let cox = crate::special::boxes::commodity_deed_box_find(w, id);
    let cox_secure = cox.is_some_and(|c| ops::is_secure(w, c));
    // Veteran Rewards mods
    let number = if let Some(c) = commodity_of(w, id) {
        if in_bank {
            if let Some(b) = bank
                && let Some(ty) = w.item(b).map(|i| i.ty)
            {
                (ty.drop_item)(w, b, c);
            }
            set_commodity_field(w, id, None);
            delete(w, id);
            1047031 // The commodity has been redeemed.
        } else if let Some(cox) = cox {
            if cox_secure {
                if let Some(ty) = w.item(cox).map(|i| i.ty) {
                    (ty.drop_item)(w, cox, c);
                }
                set_commodity_field(w, id, None);
                delete(w, id);
                1047031 // The commodity has been redeemed.
            } else {
                1080525 // The commodity deed box must be secured before you can use it.
            }
        } else if ml {
            1080526 // That must be in your bank box or commodity deed box to use it.
        } else {
            1047024 // To claim the resources ....
        }
    } else if cox.is_some() && !cox_secure {
        1080525 // The commodity deed box must be secured before you can use it.
    } else if !in_bank && cox.is_none() {
        if ml {
            1080526 // That must be in your bank box or commodity deed box to use it.
        } else {
            1047026 // That must be in your bank box to use it.
        }
    } else {
        target::begin_target(w, from, 3, false, TargetFlags::NONE, move |w, from, t| {
            commodity_target(w, id, from, t)
        });
        1047029 // Target the commodity to fill this deed with.
    };
    deps::send_localized_message(w, from, number);
}

fn commodity_target(w: &mut World, deed: ItemId, from: MobileId, t: Targeted) {
    if w.item(deed).is_none_or(|i| i.deleted()) {
        return;
    }
    let number = if commodity_of(w, deed).is_some() {
        1047028 // The commodity deed has already been filled.
    } else if let Targeted::Entity(EntityId::Item(item)) = t {
        let bank = muo_server::mobile::part2::items::find_bank_no_create(w, from);
        let cox = crate::special::boxes::commodity_deed_box_find(w, deed);
        // Veteran Rewards mods: `cox?.IsSecure != true && item.IsChildOf(cox)` (kept as is)
        let ok = bank.is_some_and(|b| {
            ops::is_child_of(w, deed, Some(EntityId::Item(b)))
                && ops::is_child_of(w, item, Some(EntityId::Item(b)))
        }) || (!cox.is_some_and(|c| ops::is_secure(w, c))
            && cox.is_some_and(|c| ops::is_child_of(w, item, Some(EntityId::Item(c)))));
        if ok {
            if set_commodity(w, deed, item) {
                crate::simple::hue(w, deed, 0x592);
                1047030 // The commodity deed has been filled.
            } else {
                1047027 // That is not a commodity the bankers will fill a commodity deed with.
            }
        } else if muo_content::core::ml(w) {
            1080526 // That must be in your bank box or commodity deed box to use it.
        } else {
            1047026 // That must be in your bank box to use it.
        }
    } else {
        1047027 // That is not a commodity the bankers will fill a commodity deed with.
    };
    deps::send_localized_message(w, from, number);
}

/// `(Commodity as ICommodity)?.DescriptionNumber ?? Commodity.LabelNumber`: the label, except for the wood a Board
/// or Log is made of.
fn description_number(w: &World, c: ItemId) -> i32 {
    use muo_content::craft::{CraftResource as R, is_standard};
    use muo_content::items::resource::{BOARD, LOG, resource};
    let Some(ty) = w.item(c).map(|i| i.ty) else {
        return 0;
    };
    let label = (ty.label_number)(w, c);
    if ty.is(&BOARD) {
        let r = resource(w, c);
        return match r {
            R::OakWood | R::AshWood | R::YewWood => 1075052 + (r as i32 - R::OakWood as i32),
            R::Bloodwood => 1075055,
            R::Frostwood => 1075056,
            R::Heartwood => 1075062, // WHY Osi.  Why?
            _ => label,
        };
    }
    if ty.is(&LOG) {
        let r = resource(w, c);
        return if is_standard(r) {
            label
        } else {
            1075062 + (r as i32 - R::RegularWood as i32)
        };
    }
    label
}

/// `Server.Items.CommodityDeed`; `[SerializationGenerator(1, false)]`.
pub static COMMODITY_DEED: ItemType = ItemType {
    full_name: "Server.Items.CommodityDeed",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_commodity_deed(w, None)),
    default_weight: crate::class_props::default_weight,
    label_number: |w, id| {
        if commodity_of(w, id).is_none() {
            1047016
        } else {
            1047017
        }
    },
    get_properties: |w, id, list: &mut PropertyList| {
        (ITEM_BASE.get_properties)(w, id, list);
        if let Some(c) = commodity_of(w, id) {
            let amount = amount_of(w, c);
            let item = w.item(c);
            let name = item.and_then(|i| i.name.clone());
            let _ = item;
            // `Commodity is ICommodity ic` shows DescriptionNumber (a given name is ignored); anything else its
            // Name or LabelNumber.
            let args = match name {
                Some(n) if commodity::deedable(w, c).is_none() => format!("{n}\t{amount}"),
                _ => format!("#{}\t{amount}", description_number(w, c)),
            };
            list.add_str(1060658, &args); // ~1_val~: ~2_val~
        } else {
            list.add(1060748); // unfilled
        }
    },
    on_single_click: |w, id, from| {
        (ITEM_BASE.on_single_click)(w, id, from);
        if let Some(c) = commodity_of(w, id) {
            let amount = amount_of(w, c);
            let item = w.item(c);
            let args = match item.and_then(|i| i.name.clone()) {
                Some(n) => format!("{n}\t{amount}"),
                None => format!("#{}\t{amount}", description_number(w, c)),
            };
            ops::label_to_number(w, id, from, 1060658, &args);
        }
    },
    on_double_click: commodity_double_click,
    on_delete: |w, id| {
        if let Some(c) = commodity_of(w, id) {
            delete(w, c);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(1);
        out.write_item(w, commodity_of(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?; // version 0 reads the same item
        let c = r.read_item(w)?;
        set_commodity_field(w, id, c);
        if c.is_some() {
            crate::simple::hue(w, id, 0x592);
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- NameChangeDeed ----

/// `NameChangeDeedGump`.
struct NameChangeGump {
    sender: ItemId,
}

fn add_black_alpha(b: &mut StaticGumpBuilder, x: i32, y: i32, w: i32, h: i32) {
    b.add_image_tiled(x, y, w, h, 2624);
    b.add_alpha_region(x, y, w, h);
}

impl StaticGumpLogic for NameChangeGump {
    const TYPE_NAME: &'static str = "Server.Items.NameChangeDeedGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_page(0);
        add_black_alpha(b, 10, 120, 250, 85);
        b.add_html(
            10,
            125,
            250,
            20,
            "<CENTER><BASEFONT COLOR=#FFFFFF>Name Change Deed</BASEFONT></CENTER>",
            false,
            false,
        );
        b.add_label(73, 15, 1152, "");
        b.add_label(20, 150, 0x480, "New Name:");
        // AddTextField(100, 150, 150, 20, 0)
        b.add_background(98, 148, 154, 24, 0x2486);
        b.add_text_entry(102, 152, 146, 16, 0, 0, "");
        // AddButtonLabeled(75, 180, 1, "Submit")
        b.add_button(75, 179, 4005, 4007, 1);
        b.add_html(
            110,
            180,
            240,
            20,
            "<BASEFONT COLOR=#FFFFFF>Submit</CENTER>",
            false,
            false,
        );
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let Some(m) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let root = ops::root_parent(w, self.sender);
        if w.item(self.sender).is_none_or(|i| i.deleted())
            || info.button_id != 1
            || root != Some(EntityId::Mobile(m))
        {
            return;
        }
        let new_name = info.get_text_entry(0).unwrap_or("").trim().to_string();
        if !validate_player_name(&new_name) {
            deps::send_message(w, m, "That name is unacceptable.");
            return;
        }
        muo_server::mobile::part1::props_b::set_raw_name(w, m, Some(new_name.clone()));
        deps::send_message(w, m, "Your name has been changed!");
        deps::send_message(w, m, &format!("You are now known as {new_name}"));
        delete(w, self.sender);
    }
}

/// `Server.Items.NameChangeDeed`.
pub static NAME_CHANGE_DEED: ItemType = ItemType {
    full_name: "Server.Items.NameChangeDeed",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| crate::simple::from_template(w, &NAME_CHANGE_DEED, "Server.Items.NameChangeDeed")),
    default_name: crate::class_props::default_name,
    on_double_click: |w, id, from| {
        if ops::root_parent(w, id) == Some(EntityId::Mobile(from)) {
            if w.mobile(from).is_some_and(|m| m.net_state.is_some())
                && w.item(id).is_some_and(|i| !i.deleted())
            {
                send_gump_to_mobile(
                    w,
                    from,
                    Box::new(StaticGumpObj::new(50, 50, NameChangeGump { sender: id })),
                    true,
                );
            }
        } else {
            deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        }
    },
    serialize: |w, id, out| v0_write(w, id, out, &ITEM_BASE),
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- ClothingBlessDeed ----

fn clothing_bless_target(w: &mut World, deed: ItemId, from: MobileId, t: Targeted) {
    if w.item(deed).is_none_or(|i| i.deleted())
        || ops::root_parent(w, deed) != Some(EntityId::Mobile(from))
    {
        return;
    }
    let Targeted::Entity(EntityId::Item(item)) = t else {
        deps::send_localized_message(w, from, 500509); // You cannot bless that object
        return;
    };
    if !w
        .item(item)
        .is_some_and(|i| i.ty.is(&muo_content::items::clothing::BASE_CLOTHING))
    {
        deps::send_localized_message(w, from, 500509); // You cannot bless that object
        return;
    }
    if crate::clothing::special::is_arcane(w, item) {
        deps::send_localized_message(w, from, 1005019); // This bless deed is for Clothes only.
        return;
    }
    let (loot, blessed_for, insured) = w.item(item).map_or((LootType::Regular, None, false), |i| {
        (i.loot_type, i.blessed_for, i.insured())
    });
    let insurance = muo_content::player_mobile::hooks::insurance_enabled(w);
    if loot == LootType::Blessed || blessed_for == Some(from) || (insurance && insured) {
        deps::send_localized_message(w, from, 1045113); // That item is already blessed
    } else if loot != LootType::Regular {
        deps::send_localized_message(w, from, 1045114); // You can not bless that item
    } else if ops::root_parent(w, item) != Some(EntityId::Mobile(from)) {
        deps::send_localized_message(w, from, 500509); // You cannot bless that object
    } else {
        ops::set_loot_type(w, item, LootType::Blessed);
        deps::send_localized_message(w, from, 1010026); // You bless the item....
        delete(w, deed); // Delete the bless deed
    }
}

/// `Server.Items.ClothingBlessDeed`.
pub static CLOTHING_BLESS_DEED: ItemType = ItemType {
    full_name: "Server.Items.ClothingBlessDeed",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| crate::simple::from_template(w, &CLOTHING_BLESS_DEED, "Server.Items.ClothingBlessDeed")),
    default_weight: crate::class_props::default_weight,
    default_name: crate::class_props::default_name,
    display_loot_type: |_, _| false,
    on_double_click: |w, id, from| {
        if !pack_has(w, id, from) {
            deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        } else {
            deps::send_localized_message(w, from, 1005018); // What would you like to bless? (Clothes Only)
            target::begin_target(w, from, 1, false, TargetFlags::NONE, move |w, from, t| {
                clothing_bless_target(w, id, from, t)
            });
        }
    },
    serialize: |w, id, out| v0_write(w, id, out, &ITEM_BASE),
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- NewPlayerTicket ----

/// `NewPlayerTicket._owner`.
#[derive(Clone, Copy, Debug, Default)]
struct TicketData {
    owner: Option<MobileId>,
}

fn ticket_owner(w: &World, id: ItemId) -> Option<MobileId> {
    w.item(id)?.ext.get::<TicketData>()?.owner
}

/// `ticket.Owner = owner`.
pub fn set_ticket_owner(w: &mut World, id: ItemId, owner: Option<MobileId>) {
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(TicketData { owner });
    }
}

/// `NewPlayerTicketGump`.
struct TicketGump {
    ticket: ItemId,
}

impl StaticGumpLogic for TicketGump {
    const TYPE_NAME: &'static str = "Server.Items.NewPlayerTicket+NewPlayerTicketGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut StaticGumpBuilder) {
        b.add_background(0, 0, 400, 385, 0xA28);
        // Choose the gift you prefer. WARNING: if you cancel, and your partner does not, you will need to find another matching ticket!
        b.add_html_localized(30, 45, 340, 70, 1013011, true, true);
        for (i, (y, label)) in [
            (128, 1013012),
            (163, 1013013),
            (198, 1013014),
            (233, 1013015),
            (268, 1013016),
            (303, 1013017),
        ]
        .into_iter()
        .enumerate()
        {
            b.add_button(46, y, 0xFA5, 0xFA7, i as i32 + 1);
            b.add_html_localized(80, y + 2, 320, 35, label, false, false);
        }
        b.add_button(120, 340, 0xFA5, 0xFA7, 0);
        b.add_html_localized(154, 342, 100, 35, 1011012, false, false); // CANCEL
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        if w.item(self.ticket).is_none_or(|i| i.deleted()) {
            return;
        }
        let (items, number): (&[(&str, &[i64])], i32) = match info.button_id {
            1 => (&[("Server.Items.Sextant", &[])], 1010494),
            2 => (&[("Server.Items.HairRestylingDeed", &[])], 501933),
            3 => (&[("Server.Items.Spellbook", &[0xFFFF_FFFF])], 1010495),
            4 => (&[("Server.Items.FireworksWand", &[])], 501935),
            5 => (&[("Server.Items.Spyglass", &[])], 501936),
            6 => (
                &[("Server.Items.DyeTub", &[]), ("Server.Items.Dyes", &[])],
                501937,
            ),
            _ => return,
        };
        let Some(from) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let made: Vec<ItemId> = items
            .iter()
            .filter_map(|(name, args)| {
                let args: Vec<muo_content::props::Value> = args
                    .iter()
                    .map(|&a| muo_content::props::Value::UInt(a as u64))
                    .collect();
                muo_content::factory::make_item_args(w, name, &args)
            })
            .collect();
        // `if (item != null)`: the first item decides
        if made.is_empty() || made.len() != items.len() {
            for m in made {
                delete(w, m);
            }
            return;
        }
        delete(w, self.ticket);
        deps::send_localized_message(w, from, number);
        for m in made {
            muo_server::mobile::part2::items::add_to_backpack(w, from, m);
        }
    }
}

fn ticket_target(w: &mut World, mine: ItemId, from: MobileId, t: Targeted) {
    let Targeted::Entity(EntityId::Item(item)) = t else {
        deps::send_localized_message(w, from, 501929); // You will need to select a ticket.
        return;
    };
    if item == mine {
        deps::send_localized_message(w, from, 501928); // You can't target the same ticket!
    } else if w.item(item).is_some_and(|i| i.ty.is(&NEW_PLAYER_TICKET)) {
        let them = ticket_owner(w, item).filter(|&m| w.mobile(m).is_some_and(|x| !x.deleted));
        match them {
            None => deps::send_localized_message(w, from, 501930), // That is not a valid ticket.
            Some(them) => {
                for (m, t) in [(from, mine), (them, item)] {
                    let live = w.mobile(m).is_some_and(|x| x.net_state.is_some())
                        && w.item(t).is_some_and(|i| !i.deleted());
                    if live {
                        send_gump_to_mobile(
                            w,
                            m,
                            Box::new(StaticGumpObj::new(50, 50, TicketGump { ticket: t })),
                            true,
                        );
                    }
                }
            }
        }
    } else if w.item(item).is_some_and(|i| i.item_id == 0x14F0) {
        deps::send_localized_message(w, from, 501931); // You need to find another ticket marked NEW PLAYER.
    } else {
        deps::send_localized_message(w, from, 501929); // You will need to select a ticket.
    }
}

/// `Server.Items.NewPlayerTicket`.
pub static NEW_PLAYER_TICKET: ItemType = ItemType {
    full_name: "Server.Items.NewPlayerTicket",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = crate::simple::from_template(w, &NEW_PLAYER_TICKET, "Server.Items.NewPlayerTicket");
        set_ticket_owner(w, id, None);
        id
    }),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // a young player ticket
    display_loot_type: |_, _| false,
    get_properties: |w, id, list: &mut PropertyList| {
        (ITEM_BASE.get_properties)(w, id, list);
        // This is half a prize ticket! Double-click this ticket and target any other ticket marked NEW PLAYER and get a prize! This ticket will only work for YOU, so don't give it away!
        list.add(1041492);
    },
    on_double_click: |w, id, from| {
        if Some(from) != ticket_owner(w, id) {
            deps::send_localized_message(w, from, 501926); // This isn't your ticket! Shame on you! You have to use YOUR ticket.
        } else if !pack_has(w, id, from) {
            deps::send_localized_message(w, from, 1042001); // That must be in your pack for you to use it.
        } else {
            deps::send_localized_message(w, from, 501927); // Target any other ticket marked NEW PLAYER to win a prize.
            target::begin_target(w, from, 2, false, TargetFlags::NONE, move |w, from, t| {
                ticket_target(w, id, from, t)
            });
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        out.write_mobile(w, ticket_owner(w, id));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let owner = r.read_mobile(w)?;
        set_ticket_owner(w, id, owner);
        Ok(())
    },
    ..ITEM_BASE
};

use muo_content::props::{self, Kind, PropDef, Value};
use muo_content::utilities::types::PropType;
use muo_server::mobile::AccessLevel;

/// `[CommandProperty(AccessLevel.Owner)] NewPlayerTicket.Owner`.
static TICKET_PROPS: &[PropDef] = &[PropDef {
    name: "Owner",
    kind: Kind::Plain(PropType::Entity),
    read: AccessLevel::Owner,
    write: AccessLevel::Owner,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Entity(ticket_owner(w, i).map(EntityId::Mobile)),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err("not an item".to_string());
        };
        let owner = match v.entity() {
            Some(EntityId::Mobile(m)) => Some(m),
            None => None,
            Some(_) => return Err("not a mobile".to_string()),
        };
        set_ticket_owner(w, i, owner);
        Ok(())
    }),
}];

fn configure(w: &mut World) {
    props::add(w, "Server.Items.NewPlayerTicket", TICKET_PROPS);
}

/// Registers the classes.
pub fn register(types: &mut TypeRegistry) {
    types.register_phase_default(
        muo_server::type_registry::Phase::Configure,
        "Server.Items.NewPlayerTicket.Configure",
        configure,
    );
    for t in [
        &COMMODITY_DEED,
        &NAME_CHANGE_DEED,
        &CLOTHING_BLESS_DEED,
        &NEW_PLAYER_TICKET,
    ] {
        types.register_item(t, &[]);
    }
}

#[cfg(test)]
mod description_tests {
    use super::*;
    use muo_content::craft::CraftResource as R;
    use muo_content::items::resource::{BOARD, LOG, set_resource};

    #[test]
    fn board_and_log_commodities_are_named_for_their_wood() {
        let mut w = World::new();
        let board = (BOARD.construct.unwrap())(&mut w);
        let log = (LOG.construct.unwrap())(&mut w);
        let plain = description_number(&w, board);
        assert_eq!(plain, (BOARD.label_number)(&w, board));
        for (r, board_n, log_n) in [
            (R::OakWood, 1075052, 1075063),
            (R::YewWood, 1075054, 1075065),
            (R::Heartwood, 1075062, 1075066),
            (R::Bloodwood, 1075055, 1075067),
            (R::Frostwood, 1075056, 1075068),
        ] {
            set_resource(&mut w, board, r);
            set_resource(&mut w, log, r);
            assert_eq!(description_number(&w, board), board_n, "{r:?}");
            assert_eq!(description_number(&w, log), log_n, "{r:?}");
        }
    }
}
