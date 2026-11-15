//! The 2007 Valentine's `RedVelvetGiftBox` and `ValentinesCard`, the `EarringBoxSet` of the 11th year promotion and
//! `CommodityDeedBox`.

use std::rc::Rc;

use muo_content::items::cont::{BASE_CONTAINER, new_base_container};
use muo_primitives::{Point3D, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, ops};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::type_registry::TypeRegistry;

use super::util::msg;

/// `AddToBox(item, loc)`: `DropItem(item); item.Location = loc;`.
fn add_to_box(w: &mut World, cont: ItemId, item: ItemId, loc: Point3D) {
    if let Some(ty) = w.item(cont).map(|i| i.ty) {
        (ty.drop_item)(w, cont, item);
    }
    ops::set_location(w, item, loc);
}

/// `RedVelvetGiftBox(fill)`.
pub fn red_velvet_gift_box_new(w: &mut World, ty: &'static ItemType, fill: bool) -> ItemId {
    let id = new_base_container(w, ty, 0);
    crate::simple::own_template(w, id, "Server.Items.RedVelvetGiftBox");
    if fill {
        for i in 0..5 {
            if let Some(c) = value_card(w, "Server.Items.ValentinesCardSouth") {
                add_to_box(w, id, c, Point3D::new(60 + i * 10, 47, 0));
            }
            if let Some(c) = value_card(w, "Server.Items.ValentinesCardEast") {
                add_to_box(w, id, c, Point3D::new(20 + i * 10, 72, 0));
            }
        }
        if let Some(b) = muo_content::factory::make_item(w, "Server.Items.Bacon") {
            add_to_box(w, id, b, Point3D::new(90, 85, 0));
        }
        if let Some(r) = muo_content::factory::make_item(w, "Server.Items.RoseInAVase") {
            add_to_box(w, id, r, Point3D::new(130, 55, 0));
        }
    }
    id
}

fn value_card(w: &mut World, name: &str) -> Option<ItemId> {
    muo_content::factory::make_item(w, name)
}

/// `Server.Items.RedVelvetGiftBox`.
pub static RED_VELVET_GIFT_BOX: ItemType = ItemType {
    full_name: "Server.Items.RedVelvetGiftBox",
    parent: Some(&BASE_CONTAINER),
    construct: Some(|w| red_velvet_gift_box_new(w, &RED_VELVET_GIFT_BOX, false)),
    default_gump_id: crate::class_props::default_gump_id,
    label_number: crate::class_props::label_number, // A Red Velvet Box
    serialize: |w, id, out| {
        (BASE_CONTAINER.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_CONTAINER
};

/// `Server.Items.EarringBoxSet`.
pub static EARRING_BOX_SET: ItemType = ItemType {
    full_name: "Server.Items.EarringBoxSet",
    parent: Some(&RED_VELVET_GIFT_BOX),
    construct: Some(|w| {
        use muo_content::aos::AosElementAttribute as E;
        let id = red_velvet_gift_box_new(w, &EARRING_BOX_SET, false);
        for e in [E::Physical, E::Fire, E::Cold, E::Poison, E::Energy] {
            let item = super::leaves::earrings_of_protection_new(w, e);
            if let Some(ty) = w.item(id).map(|i| i.ty) {
                (ty.drop_item)(w, id, item);
            }
        }
        id
    }),
    serialize: |w, id, out| {
        (RED_VELVET_GIFT_BOX.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (RED_VELVET_GIFT_BOX.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..RED_VELVET_GIFT_BOX
};

// ---- ValentinesCard ----

const UNSIGNED: &str = "___";

#[derive(Clone, Debug, Default)]
pub struct CardData {
    pub label_number: i32,
    pub from: Option<String>,
    pub to: Option<String>,
}

fn card(w: &World, id: ItemId) -> CardData {
    w.item(id)
        .and_then(|i| i.ext.get::<CardData>())
        .cloned()
        .unwrap_or_default()
}

fn card_args(d: &CardData) -> String {
    format!(
        "{}\t{}",
        d.to.as_deref().unwrap_or(UNSIGNED),
        d.from.as_deref().unwrap_or(UNSIGNED)
    )
}

/// `ValentinesCard(itemid)`.
pub fn valentines_card_new(w: &mut World, ty: &'static ItemType, item_id: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    ops::set_loot_type(w, id, LootType::Blessed);
    let hue = if utility::random_double(&mut w.rng) < 0.001 {
        0x47E
    } else {
        0xE8
    };
    super::hue(w, id, hue);
    let label_number = utility::random_from(&mut w.rng, 1077589, 5);
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(CardData {
            label_number,
            from: None,
            to: None,
        });
    }
    id
}

fn card_on_target(w: &mut World, id: ItemId, from: MobileId, targeted: Targeted) {
    if ops::is_deleted(w, id) {
        return;
    }
    let Targeted::Entity(EntityId::Mobile(to)) = targeted else {
        msg(w, from, 1077488); // That's not another player!
        return;
    };
    if !w
        .mobile(to)
        .is_some_and(|m| m.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
    {
        msg(w, from, 1077496); // You can't possibly be THAT lonely!
        return;
    }
    if to == from {
        msg(w, from, 1077495); // You can't give yourself a card, silly!
        return;
    }
    let (f, t) = (
        w.mobile(from).and_then(|m| (m.ty.name)(w, from)),
        w.mobile(to).and_then(|m| (m.ty.name)(w, to)),
    );
    if let Some(d) = w.item_mut(id).and_then(|i| i.ext.get_mut::<CardData>()) {
        d.from = f;
        d.to = t;
    }
    ops::invalidate_properties(w, id);
    msg(w, from, 1077498); // You fill out the card. Hopefully the other person actually likes you...
}

/// `Server.Items.ValentinesCard`.
pub static VALENTINES_CARD: ItemType = ItemType {
    full_name: "Server.Items.ValentinesCard",
    parent: Some(&ITEM_BASE),
    construct: None,
    default_name: crate::class_props::default_name,
    add_name_property: |w, id, list| {
        let d = card(w, id);
        list.add_str(d.label_number, &card_args(&d));
    },
    on_single_click: |w, id, from| {
        (ITEM_BASE.on_single_click)(w, id, from);
        let d = card(w, id);
        ops::label_to_number(w, id, from, d.label_number, &card_args(&d));
    },
    on_double_click: |w, id, from| {
        if card(w, id).to.is_none() {
            let held = ops::is_child_of(w, id, Some(EntityId::Mobile(from)));
            if held {
                let mut t = target::Target::new(w, 10, false, TargetFlags::NONE);
                t.handlers.on_target = Some(Rc::new(move |w: &mut World, from, targeted| {
                    card_on_target(w, id, from, targeted)
                }));
                let tid = target::add_target(w, t);
                target::set_mobile_target(w, from, Some(tid));
                msg(w, from, 1077497); // To whom do you wish to give this card?
            } else {
                msg(w, from, 1080063); // This must be in your backpack to use it.
            }
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
        let d = card(w, id);
        out.write_i32(d.label_number);
        out.write_opt_string(d.from.as_deref());
        out.write_opt_string(d.to.as_deref());
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        let label_number = r.read_i32()?;
        let from = r.read_string()?;
        let to = r.read_string()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(CardData {
                label_number,
                from,
                to,
            });
        }
        Ok(())
    },
    ..ITEM_BASE
};

macro_rules! card_leaf {
    ($(#[$m:meta])* $name:ident = $full:literal, $gid:expr) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&VALENTINES_CARD),
            construct: Some(|w| valentines_card_new(w, &$name, $gid)),
            serialize: |w, id, out| {
                (VALENTINES_CARD.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (VALENTINES_CARD.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..VALENTINES_CARD
        };
    };
}

card_leaf!(
    /// `Server.Items.ValentinesCardSouth`.
    VALENTINES_CARD_SOUTH = "Server.Items.ValentinesCardSouth", 0x0EBD
);
card_leaf!(
    /// `Server.Items.ValentinesCardEast`.
    VALENTINES_CARD_EAST = "Server.Items.ValentinesCardEast", 0x0EBE
);

// ---- CommodityDeedBox ----

pub fn reward_flag(w: &World, id: ItemId) -> bool {
    w.item(id)
        .and_then(|i| i.ext.get::<RewardFlag>())
        .is_some_and(|d| d.0)
}

/// `IsRewardItem` of a reward container.
#[derive(Clone, Copy, Debug, Default)]
pub struct RewardFlag(pub bool);

/// `Server.Items.CommodityDeedBox`.
pub static COMMODITY_DEED_BOX: ItemType = ItemType {
    full_name: "Server.Items.CommodityDeedBox",
    parent: Some(&BASE_CONTAINER),
    construct: Some(|w| {
        let id = new_base_container(w, &COMMODITY_DEED_BOX, 0);
        crate::simple::own_template(w, id, "Server.Items.CommodityDeedBox");
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(RewardFlag(false));
        }
        id
    }),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // Commodity Deed Box
    default_gump_id: crate::class_props::default_gump_id,
    get_properties: |w, id, list| {
        (BASE_CONTAINER.get_properties)(w, id, list);
        if reward_flag(w, id) {
            list.add(1076217); // 1st Year Veteran Reward
        }
    },
    serialize: |w, id, out| {
        (BASE_CONTAINER.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_bool(reward_flag(w, id));
    },
    deserialize: |w, id, r| {
        (BASE_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let flag = r.read_bool()?;
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(RewardFlag(flag));
        }
        Ok(())
    },
    ..BASE_CONTAINER
};

/// `CommodityDeedBox.IsRewardItem = value`.
pub fn set_commodity_box_reward(w: &mut World, id: ItemId, value: bool) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(RewardFlag(value));
    }
    ops::invalidate_properties(w, id);
}

/// `CommodityDeedBox.Find(deed)`: the box a deed is inside of (itself included).
pub fn commodity_deed_box_find(w: &World, deed: ItemId) -> Option<ItemId> {
    let mut cur = Some(deed);
    while let Some(c) = cur {
        let it = w.item(c)?;
        if it.ty.is(&COMMODITY_DEED_BOX) {
            return Some(c);
        }
        cur = match it.parent {
            Some(EntityId::Item(p)) => Some(p),
            _ => None,
        };
    }
    None
}

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &RED_VELVET_GIFT_BOX,
        &EARRING_BOX_SET,
        &VALENTINES_CARD,
        &VALENTINES_CARD_SOUTH,
        &VALENTINES_CARD_EAST,
        &COMMODITY_DEED_BOX,
    ] {
        types.register_item(t, &[]);
    }
}
