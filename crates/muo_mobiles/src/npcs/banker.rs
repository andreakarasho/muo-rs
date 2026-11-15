//! `Banker` (the class; the statics are `muo_content::vendor::banker`), `Minter` and `GypsyBanker`.

use muo_content::items::gold::n0;
use muo_content::mx;
use muo_content::vendor::banker::{get_balance, withdraw};
use muo_content::vendor::{
    BASE_VENDOR, BASE_VENDOR_C, VendorClass, init_outfit,
};
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::events::SpeechEventArgs;
use muo_server::iaccount;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{container, ops};
use muo_server::layer::Layer;
use muo_server::mobile::MobileType;
use muo_server::mobile::part2::messages;
use muo_server::packets::message::AffixType;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

use super::vendors::hand::{add_gypsy_hat, rehue_layer_bright};

fn say(w: &mut World, id: MobileId, n: i32) {
    messages::say_number(w, id, n, "");
}

fn criminal(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| x.criminal)
}

/// `OnSpeech(e)`: *withdraw*, *balance*, *bank* and *check*.
fn on_speech(w: &mut World, id: MobileId, e: &mut SpeechEventArgs) {
    let from = e.mobile;
    if !e.handled && mx::in_range_point(w, from, mx::location(w, id), 12) {
        for &keyword in &e.keywords {
            match keyword {
                0x0000 => {
                    // *withdraw*
                    e.handled = true;
                    if criminal(w, from) {
                        say(w, id, 500389); // I will not do business with a criminal!
                        continue; // next keyword
                    }
                    let split: Vec<&str> = e.speech.split(' ').collect();
                    if split.len() >= 2 {
                        let pack = w.mobile(from).and_then(|m| m.backpack);
                        let Ok(amount) = split[1].parse::<i32>() else {
                            continue; // next keyword
                        };
                        let ml = w.core_ml();
                        if (!ml && amount > 5000) || (ml && amount > 60000) {
                            say(w, id, 500381); // Thou canst not withdraw so much at one time!
                        } else if pack.is_none_or(|p| {
                            w.item(p).is_none_or(|i| i.deleted())
                                || !((ops::get_total(w, p, muo_server::item::TotalType::Weight)
                                    as i32)
                                    < (w.item(p).map_or(0, |i| (i.ty.max_weight)(w, p))))
                                || !(ops::get_total(w, p, muo_server::item::TotalType::Items)
                                    < container::max_items(w, p))
                        }) {
                            say(w, id, 1048147); // Your backpack can't hold anything else.
                        } else if amount > 0 {
                            if !withdraw(w, from, amount) {
                                say(w, id, 500384); // Ah, art thou trying to fool me? Thou hast not so much gold!
                            } else {
                                if let (Some(pack), Some(g)) = (
                                    pack,
                                    muo_content::factory::make_item(w, "Server.Items.Gold"),
                                ) {
                                    ops::set_amount(w, g, amount);
                                    container::drop_item_default(w, pack, g);
                                }
                                say(w, id, 1010005); // Thou hast withdrawn gold from thy account.
                            }
                        }
                    }
                    continue; // next keyword
                }
                0x0001 => {
                    // *balance*
                    e.handled = true;
                    if criminal(w, from) {
                        say(w, id, 500389);
                        continue; // next keyword
                    }
                    let totals = if iaccount::gold_enabled(w) {
                        iaccount::account_totals(w, from)
                    } else {
                        None
                    };
                    if let Some((gold, plat)) = totals {
                        // Thy current bank balance is ~1_AMOUNT~ platinum and ~2_AMOUNT~ gold.
                        let args = format!("{}\t{}", n0(i64::from(plat)), n0(i64::from(gold)));
                        messages::say_number(w, id, 1155855, &args);
                    } else {
                        // Thy current bank balance is ~1_AMOUNT~ gold.
                        let bal = get_balance(w, from);
                        messages::say_number(w, id, 1042759, &n0(i64::from(bal)));
                    }
                    continue; // next keyword
                }
                0x0002 => {
                    // *bank*
                    e.handled = true;
                    if criminal(w, from) {
                        say(w, id, 500378); // Thou art a criminal and cannot access thy bank box.
                        continue; // next keyword
                    }
                    if let Some(bank) = muo_server::mobile::part1::props_b::bank_box(w, from) {
                        muo_server::item::containers::open(w, bank);
                    }
                    continue; // next keyword
                }
                0x0003 => {
                    // *check*
                    e.handled = true;
                    if iaccount::gold_enabled(w) {
                        continue; // next keyword
                    }
                    if criminal(w, from) {
                        say(w, id, 500389);
                        continue; // next keyword
                    }
                    let split: Vec<&str> = e.speech.split(' ').collect();
                    if split.len() >= 2 {
                        let Ok(amount) = split[1].parse::<i32>() else {
                            continue; // next keyword
                        };
                        if amount < 5000 {
                            say(w, id, 1010006); // We cannot create checks for such a paltry amount of gold!
                        } else if amount > 1_000_000 {
                            say(w, id, 1010007); // Our policies prevent us from creating checks worth that much!
                        } else {
                            let check = muo_content::items::gold::new_bank_check(w, amount);
                            let Some(bank) = muo_server::mobile::part1::props_b::bank_box(w, from)
                            else {
                                continue; // next keyword
                            };
                            if !container::try_drop_item(w, bank, Some(from), check, false) {
                                say(w, id, 500386); // There's not enough room in your bankbox for the check!
                                delete_item(w, check);
                            } else if !container::consume_total(
                                w,
                                bank,
                                &muo_content::items::gold::GOLD,
                                amount,
                                true,
                                None,
                            ) {
                                say(w, id, 500384); // Ah, art thou trying to fool me? Thou hast not so much gold!
                                delete_item(w, check);
                            } else {
                                // Into your bank box I have placed a check in the amount of:
                                messages::say_affix(
                                    w,
                                    id,
                                    1042673,
                                    AffixType::APPEND,
                                    &amount.to_string(),
                                    "",
                                );
                            }
                        }
                    }
                    continue; // next keyword
                }
                _ => {}
            }
        }
    }
    (BASE_VENDOR.on_speech)(w, id, e);
}

fn delete_item(w: &mut World, id: ItemId) {
    if let Some(t) = w.item(id).map(|i| i.ty) {
        (t.delete)(w, id);
    }
}

fn add_custom_context_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    if mx::alive(w, from) {
        list.push(muo_content::context_menus::entries::open_bank());
    }
    (BASE_VENDOR_C.add_custom_context_entries)(w, id, from, list);
}

// ---- Banker ----

static BANKER_V: VendorClass = VendorClass {
    ..super::vendors::npc::TEMPLATE_VENDOR_V
};
static BANKER_C: muo_content::creature::CreatureType = muo_content::creature::CreatureType {
    add_custom_context_entries,
    subclass: Some(&BANKER_V),
    ..BASE_VENDOR_C
};
fn handles_on_speech(w: &World, id: MobileId, from: MobileId) -> bool {
    mx::in_range_point(w, from, mx::location(w, id), 12)
        || (BASE_VENDOR.handles_on_speech)(w, id, from)
}
/// `Server.Mobiles.Banker`.
pub static BANKER: MobileType = MobileType {
    full_name: "Server.Mobiles.Banker",
    parent: Some(&BASE_VENDOR),
    construct: Some(|w| super::vendors::npc::new_template_vendor(w, &BANKER)),
    family: Some(&BANKER_C),
    handles_on_speech,
    on_speech,
    serialize: |w, id, out| {
        (BASE_VENDOR.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_VENDOR.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_VENDOR
};

// ---- Minter ----

static MINTER_V: VendorClass = VendorClass {
    ..BANKER_V
};
static MINTER_C: muo_content::creature::CreatureType = muo_content::creature::CreatureType {
    subclass: Some(&MINTER_V),
    ..BANKER_C
};
/// `Server.Mobiles.Minter`.
pub static MINTER: MobileType = MobileType {
    full_name: "Server.Mobiles.Minter",
    parent: Some(&BANKER),
    construct: Some(|w| super::vendors::npc::new_template_vendor(w, &MINTER)),
    family: Some(&MINTER_C),
    serialize: |w, id, out| {
        (BANKER.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BANKER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BANKER
};

// ---- GypsyBanker ----

static GYPSY_BANKER_V: VendorClass = VendorClass {
    is_active_vendor: |_, _| false,
    init_outfit: |w, id| {
        init_outfit(w, id);
        add_gypsy_hat(w, id);
        for layer in [
            Layer::Pants,
            Layer::Shoes,
            Layer::OuterLegs,
            Layer::InnerLegs,
            Layer::OuterTorso,
            Layer::InnerTorso,
            Layer::Shirt,
        ] {
            rehue_layer_bright(w, id, layer);
        }
    },
    ..BANKER_V
};
static GYPSY_BANKER_C: muo_content::creature::CreatureType = muo_content::creature::CreatureType {
    subclass: Some(&GYPSY_BANKER_V),
    ..BANKER_C
};
/// `Server.Mobiles.GypsyBanker`.
pub static GYPSY_BANKER: MobileType = MobileType {
    full_name: "Server.Mobiles.GypsyBanker",
    parent: Some(&BANKER),
    construct: Some(|w| super::vendors::npc::new_template_vendor(w, &GYPSY_BANKER)),
    family: Some(&GYPSY_BANKER_C),
    click_title: |_, _| false,
    serialize: |w, id, out| {
        (BANKER.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BANKER.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BANKER
};

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in [&BANKER, &MINTER, &GYPSY_BANKER] {
        types.register_mobile(t, &[]);
    }
}

#[cfg(test)]
pub(crate) fn all() -> Vec<&'static MobileType> {
    vec![&BANKER, &MINTER, &GYPSY_BANKER]
}
