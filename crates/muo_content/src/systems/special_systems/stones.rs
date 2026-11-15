//! Special stones: `ResGate`, `GamblingStone` and the supply stones.
//!
//! All of them are `[SerializationGenerator(0, false)]` leaves: the parent part, then the version int 0 (the
//! gambling stone then its pot).

use muo_primitives::MessageType;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{self, ITEM_BASE, ItemType, base, container, ops};
use muo_server::mobile::part2::{effects, items as mitems, messages as mm};
use muo_server::property_list::PropertyList;

use crate::items::gold::{GOLD, new_bank_check};

/// Systems the stones reach that the Gumps crate owns.
#[derive(Default)]
pub struct StoneHooks {
    /// `m.SendGump(new ResurrectGump(m))`.
    pub show_resurrect_gump: Option<fn(&mut World, MobileId)>,
}

fn send(w: &mut World, to: MobileId, number: i32) {
    mm::send_localized_message(w, to, number, "", 0x3B2);
}

fn make_item(w: &mut World, full_name: &str) -> Option<ItemId> {
    match w.types.find_type_by_full_name(full_name, false)? {
        muo_server::type_registry::EntityType::Item(t) => Some((t.construct?)(w)),
        muo_server::type_registry::EntityType::Mobile(_) => None,
    }
}

// ---- ResGate ----------------------------------------------------------------------------------------------------

/// `Server.Items.ResGate`.
pub static RES_GATE: ItemType = ItemType {
    full_name: "Server.Items.ResGate",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &RES_GATE, 0);
        crate::templates::apply_item_as(w, id, RES_GATE.full_name);
        id
    }),
    default_name: |w, id| crate::template_keys::item_name(w, id, ITEM_BASE.default_name),
    on_move_over: |w, _, m| {
        let (alive, map, loc) = w.mobile(m).map_or((true, None, Default::default()), |x| {
            ((x.ty.alive)(w, m), x.map, x.location)
        });
        let fits = map.is_some_and(|map| {
            muo_server::map::fit::can_fit(w, map, loc.x, loc.y, loc.z, 16, false, false, true)
        });
        if !alive && fits {
            effects::play_sound(w, m, 0x214);
            effects::fixed_effect(w, m, 0x376A, 10, 16);

            if let Some(f) = w
                .resources
                .get::<StoneHooks>()
                .and_then(|h| h.show_resurrect_gump)
            {
                f(w, m);
            }
        } else {
            send(w, m, 502391); // Thou can not be resurrected there!
        }

        false
    },
    serialize: |w, id, out| {
        base::serialize_base(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        base::deserialize_base(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- supply stones ----------------------------------------------------------------------------------------------

/// `from.AddToBackpack(bag)`, deleting the bag when it doesn't fit.
fn give_bag(w: &mut World, from: MobileId, bag_class: &str) {
    let Some(bag) = make_item(w, bag_class) else {
        return;
    };
    if !mitems::add_to_backpack(w, from, bag)
        && let Some(ty) = w.item(bag).map(|i| i.ty)
    {
        (ty.delete)(w, bag);
    }
}

macro_rules! supply_stone {
    ($(#[$m:meta])* $name:ident = $full:literal, hue $hue:expr, name $dn:literal, bag $bag:literal) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            construct: Some(|w| {
                let id = ops::new_item_with_id(w, &$name, 0xED4);
                ops::set_movable(w, id, false);
                base::set_hue(w, id, $hue);
                id
            }),
            default_name: |_, _| Some($dn.to_string()),
            on_double_click: |w, _, from| give_bag(w, from, $bag),
            serialize: |w, id, out| {
                base::serialize_base(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                base::deserialize_base(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..ITEM_BASE
        };
    };
}

supply_stone!(
    /// `Server.Items.AlchemyStone`.
    ALCHEMY_STONE = "Server.Items.AlchemyStone", hue 0x250, name "an Alchemist Supply Stone", bag "Server.Items.AlchemyBag"
);
supply_stone!(
    /// `Server.Items.IngotStone`: its bag type is spelled `BagOfingots` (sic), as saves name it.
    INGOT_STONE = "Server.Items.IngotStone", hue 0x480, name "an Ingot stone", bag "Server.Items.BagOfingots"
);
supply_stone!(
    /// `Server.Items.RegStone`.
    REG_STONE = "Server.Items.RegStone", hue 0x2D1, name "a reagent stone", bag "Server.Items.BagOfReagents"
);
supply_stone!(
    /// `Server.Items.ScribeStone`.
    SCRIBE_STONE = "Server.Items.ScribeStone", hue 0x105, name "a Scribe Supply Stone", bag "Server.Items.ScribeBag"
);
supply_stone!(
    /// `Server.Items.SmithStone`.
    SMITH_STONE = "Server.Items.SmithStone", hue 0x476, name "a Blacksmith Supply Stone", bag "Server.Items.SmithBag"
);
supply_stone!(
    /// `Server.Items.TailorStone`.
    TAILOR_STONE = "Server.Items.TailorStone", hue 0x315, name "a Tailor Supply Stone", bag "Server.Items.TailorBag"
);

// ---- GamblingStone ----------------------------------------------------------------------------------------------

/// What `GamblingStone` adds: `_gamblePot`.
#[derive(Clone, Copy, Debug)]
pub struct GamblingStoneData {
    pub gamble_pot: i32,
}

pub fn gamble_pot(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<GamblingStoneData>())
        .map_or(2500, |d| d.gamble_pot)
}

pub fn set_gamble_pot(w: &mut World, id: ItemId, v: i32) {
    if let Some(i) = w.item_mut(id) {
        i.ext = muo_server::Ext::new(GamblingStoneData { gamble_pot: v });
    }
    // [InvalidateProperties]
    base::invalidate_properties(w, id);
}

fn say(w: &mut World, to: MobileId, hue: i32, text: &str) {
    mm::send_message_hue(w, to, hue, text);
}

/// `OnDoubleClick(from)`.
fn gambling_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack);

    let paid = pack.is_some_and(|p| container::consume_total(w, p, &GOLD, 250, true, None));
    if paid {
        let pot = gamble_pot(w, id) + 150;
        set_gamble_pot(w, id, pot);

        let roll = muo_primitives::utility::random(&mut w.rng, 1200);

        if roll == 0 {
            // Jackpot
            const MAX_CHECK: i32 = 1_000_000;

            let mut pot = gamble_pot(w, id);
            say(w, from, 0x35, &format!("You win the {pot}gp jackpot!"));

            while pot > MAX_CHECK {
                let check = new_bank_check(w, MAX_CHECK);
                mitems::add_to_backpack(w, from, check);

                pot -= MAX_CHECK;
            }

            let check = new_bank_check(w, pot);
            mitems::add_to_backpack(w, from, check);

            set_gamble_pot(w, id, 2500);
        } else if roll <= 20 {
            // Chance for a regbag
            say(w, from, 0x35, "You win a bag of reagents!");
            if let Some(bag) = make_item(w, "Server.Items.BagOfReagents") {
                mitems::add_to_backpack(w, from, bag);
            }
        } else if roll <= 40 {
            // Chance for gold
            say(w, from, 0x35, "You win 1500gp!");
            let check = new_bank_check(w, 1500);
            mitems::add_to_backpack(w, from, check);
        } else if roll <= 100 {
            // Another chance for gold
            say(w, from, 0x35, "You win 1000gp!");
            let check = new_bank_check(w, 1000);
            mitems::add_to_backpack(w, from, check);
        } else {
            // Loser!
            say(w, from, 0x22, "You lose!");
        }
    } else {
        say(
            w,
            from,
            0x22,
            "You need at least 250gp in your backpack to use this.",
        );
    }
}

/// `Server.Items.GamblingStone`.
pub static GAMBLING_STONE: ItemType = ItemType {
    full_name: "Server.Items.GamblingStone",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| {
        let id = ops::new_item_with_id(w, &GAMBLING_STONE, 0);
        crate::templates::apply_item_as(w, id, GAMBLING_STONE.full_name);
        if let Some(i) = w.item_mut(id) {
            i.ext = muo_server::Ext::new(GamblingStoneData { gamble_pot: 2500 });
        }
        id
    }),
    default_name: |w, id| crate::template_keys::item_name(w, id, ITEM_BASE.default_name),
    get_properties: |w, id, list: &mut PropertyList| {
        base::get_properties_base(w, id, list);
        list.add_text(&format!("Jackpot: {}gp", gamble_pot(w, id)));
    },
    on_single_click: |w, id, from| {
        base::on_single_click_base(w, id, from);
        ops::label_to(w, id, from, &format!("Jackpot: {}gp", gamble_pot(w, id)));
    },
    on_double_click: gambling_double_click,
    serialize: |w, id, out| {
        base::serialize_base(w, id, out);
        out.write_i32(0); // version
        out.write_i32(gamble_pot(w, id));
    },
    deserialize: |w, id, r| {
        base::deserialize_base(w, id, r)?;
        let _version = r.read_i32()?;
        let pot = r.read_i32()?;
        if let Some(i) = w.item_mut(id) {
            i.ext = muo_server::Ext::new(GamblingStoneData { gamble_pot: pot });
        }
        Ok(())
    },
    ..ITEM_BASE
};

/// Every class of this file, for the type registry.
pub static ALL_ITEMS: [&ItemType; 8] = [
    &RES_GATE,
    &GAMBLING_STONE,
    &ALCHEMY_STONE,
    &INGOT_STONE,
    &REG_STONE,
    &SCRIBE_STONE,
    &SMITH_STONE,
    &TAILOR_STONE,
];

#[allow(unused)]
fn _unused(_: MessageType) {
    let _ = item::ITEM_BASE;
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;

    #[test]
    fn gambling_stone_pot_round_trips_in_the_generated_layout() {
        let mut w = World::new();
        let id = (GAMBLING_STONE.construct.unwrap())(&mut w);
        assert_eq!(gamble_pot(&w, id), 2500);
        assert!(!w.item(id).unwrap().movable());
        assert_eq!(w.item(id).unwrap().hue, 0x56);
        set_gamble_pot(&mut w, id, 4321);
        let mut writer = muo_io::serialization::save_writer::SaveWriter::new(true);
        (GAMBLING_STONE.serialize)(&w, id, &mut writer);
        let bytes = writer.into_vec();
        let tail = &bytes[bytes.len() - 8..];
        assert_eq!(
            tail,
            [0, 0, 0, 0, 0xE1, 0x10, 0, 0],
            "version int 0, then the pot (little endian 4321)"
        );
    }

    #[test]
    fn a_gambler_without_gold_is_told_so() {
        let mut w = World::new();
        let id = (GAMBLING_STONE.construct.unwrap())(&mut w);
        let m = w.new_mobile(&MOBILE_BASE);
        gambling_double_click(&mut w, id, m);
        assert_eq!(gamble_pot(&w, id), 2500, "nothing was paid");
    }

    #[test]
    fn supply_stones_are_immovable_and_named() {
        let mut w = World::new();
        for ty in [
            &ALCHEMY_STONE,
            &INGOT_STONE,
            &REG_STONE,
            &SCRIBE_STONE,
            &SMITH_STONE,
            &TAILOR_STONE,
        ] {
            let id = (ty.construct.unwrap())(&mut w);
            let it = w.item(id).unwrap();
            assert!(!it.movable());
            assert_eq!(it.item_id, 0xED4);
            assert!((ty.default_name)(&w, id).is_some());
        }
        let reg = (REG_STONE.construct.unwrap())(&mut w);
        assert_eq!(w.item(reg).unwrap().hue, 0x2D1);
        let gate = (RES_GATE.construct.unwrap())(&mut w);
        assert_eq!(
            (RES_GATE.default_name)(&w, gate).as_deref(),
            Some("a resurrection gate")
        );
    }
}
