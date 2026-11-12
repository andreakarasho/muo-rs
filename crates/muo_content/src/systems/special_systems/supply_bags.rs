//! The supply bags: AlchemyBag, BagOfIngots, ScribeBag, SmithBag and TailorBag.
//!
//! All five are `[SerializationGenerator(0, false)]` `Bag`s: the parent part, then the version int 0. Each
//! `[Constructible]` constructor takes an `amount`; the type's `construct` is the parameterless call, which uses the
//! default of that parameter.
//!
//! The items they fill the bag with (`BagOfReagents`, `MortarPestle`, `Bottle`, ingots, tools, `Hides`,
//! `BoltOfCloth`, dye tubs, `BlankScroll`, `SewingKit`, `Scissors`) are made by class name. The constructor's
//! `amount` is the table [`configure`] registers for `[add AlchemyBag 100`.

use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::ItemType;
use muo_server::item::ops;

use crate::factory::make_item;
use crate::items::cont::{BAG, new_base_container};
use crate::items::tool;
use crate::props::{Ctor, Kind, Param, Value, add_ctors};
use crate::utilities::types::{Numeric, PropType};
use muo_server::ids::EntityId;

/// What a bag item is made with.
#[derive(Copy, Clone)]
pub enum Fill {
    /// `new T()`.
    One,
    /// `new T(amount)` of a stack: the stack's `Amount`.
    Amount(i32),
    /// `new T(uses)` of a tool: its `UsesRemaining`.
    Uses(i32),
    /// `new T(n)` through the class's `[Constructible]` overload (`BagOfReagents(int amount)` fills each reagent).
    Ctor(i32),
}

/// `new T(..)` of the class `full_name`; `None` while the class is not registered.
pub fn make_filled(w: &mut World, full_name: &str, fill: Fill) -> Option<ItemId> {
    if let Fill::Ctor(n) = fill {
        return crate::factory::make_item_args(w, full_name, &[Value::Int(i64::from(n))])
            .or_else(|| make_item(w, full_name));
    }
    let item = make_item(w, full_name)?;
    match fill {
        Fill::One | Fill::Ctor(_) => {}
        Fill::Amount(n) => ops::set_amount(w, item, n),
        Fill::Uses(n) => {
            if tool::is_tool(w, item) {
                tool::set_uses_remaining(w, item, n);
            }
        }
    }
    Some(item)
}

/// `DropItem(new T(..))` of the class `full_name`.
fn drop_new(w: &mut World, bag: ItemId, full_name: &str, fill: Fill) {
    let Some(item) = make_filled(w, full_name, fill) else {
        return;
    };
    if let Some(ty) = w.item(bag).map(|i| i.ty) {
        (ty.drop_item)(w, bag, item);
    }
}

const INGOTS: [&str; 9] = [
    "Server.Items.DullCopperIngot",
    "Server.Items.ShadowIronIngot",
    "Server.Items.CopperIngot",
    "Server.Items.BronzeIngot",
    "Server.Items.GoldIngot",
    "Server.Items.AgapiteIngot",
    "Server.Items.VeriteIngot",
    "Server.Items.ValoriteIngot",
    "Server.Items.IronIngot",
];

fn alchemy_fill(w: &mut World, id: ItemId, amount: i32) {
    muo_server::item::base::set_hue(w, id, 0x250);
    drop_new(
        w,
        id,
        "Server.Items.MortarPestle",
        Fill::Uses((amount / 1000).max(1)),
    );
    drop_new(w, id, "Server.Items.BagOfReagents", Fill::Ctor(5000));
    drop_new(w, id, "Server.Items.Bottle", Fill::Amount(5000));
}

fn ingots_fill(w: &mut World, id: ItemId, amount: i32, tools_amount: Option<i32>) {
    for ingot in INGOTS {
        drop_new(w, id, ingot, Fill::Amount(amount));
    }
    match tools_amount {
        // BagOfingots: `new Tongs()`, `new TinkerTools()`
        None => {
            drop_new(w, id, "Server.Items.Tongs", Fill::One);
            drop_new(w, id, "Server.Items.TinkerTools", Fill::One);
        }
        // SmithBag: `new Tongs(amount)`, `new TinkerTools(amount)`
        Some(n) => {
            drop_new(w, id, "Server.Items.Tongs", Fill::Uses(n));
            drop_new(w, id, "Server.Items.TinkerTools", Fill::Uses(n));
        }
    }
}

fn scribe_fill(w: &mut World, id: ItemId, amount: i32) {
    muo_server::item::base::set_hue(w, id, 0x105);
    drop_new(w, id, "Server.Items.BagOfReagents", Fill::Ctor(amount));
    drop_new(w, id, "Server.Items.BlankScroll", Fill::Amount(amount));
}

fn tailor_fill(w: &mut World, id: ItemId, amount: i32) {
    muo_server::item::base::set_hue(w, id, 0x315);
    drop_new(
        w,
        id,
        "Server.Items.SewingKit",
        Fill::Uses((amount / 100).max(1)),
    );
    drop_new(w, id, "Server.Items.Scissors", Fill::One);
    drop_new(w, id, "Server.Items.Hides", Fill::Amount(amount));
    drop_new(
        w,
        id,
        "Server.Items.BoltOfCloth",
        Fill::Amount((amount / 25).max(1)),
    );
    drop_new(w, id, "Server.Items.DyeTub", Fill::One);
    drop_new(w, id, "Server.Items.DyeTub", Fill::One);
    drop_new(w, id, "Server.Items.BlackDyeTub", Fill::One);
    drop_new(w, id, "Server.Items.Dyes", Fill::One);
}

macro_rules! supply_bag {
    ($(#[$m:meta])* $name:ident = $full:literal, $fill:expr $(, name $dn:literal)?) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BAG),
            construct: Some(|w| {
                let id = new_base_container(w, &$name, 0xE76);
                let fill: fn(&mut World, ItemId) = $fill;
                fill(w, id);
                id
            }),
            $(default_name: |_, _| Some($dn.to_string()),)?
            serialize: |w, id, out| {
                (BAG.serialize)(w, id, out);
                out.write_i32(0);
            },
            deserialize: |w, id, r| {
                (BAG.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            ..BAG
        };
    };
}

supply_bag!(
    /// `Server.Items.AlchemyBag`.
    ALCHEMY_BAG = "Server.Items.AlchemyBag", |w, id| alchemy_fill(w, id, 5000), name "an Alchemy Kit"
);
supply_bag!(
    /// `Server.Items.BagOfingots` (spelled so in saves).
    BAG_OF_INGOTS = "Server.Items.BagOfingots", |w, id| ingots_fill(w, id, 5000, None)
);
supply_bag!(
    /// `Server.Items.ScribeBag`.
    SCRIBE_BAG = "Server.Items.ScribeBag", |w, id| scribe_fill(w, id, 5000), name "a Scribe Kit"
);
supply_bag!(
    /// `Server.Items.SmithBag`.
    SMITH_BAG = "Server.Items.SmithBag", |w, id| ingots_fill(w, id, 5000, Some(5000))
);
supply_bag!(
    /// `Server.Items.TailorBag`: its parameterless `amount` is 500.
    TAILOR_BAG = "Server.Items.TailorBag", |w, id| tailor_fill(w, id, 500), name "a Tailoring Kit"
);

macro_rules! amount_ctor {
    ($name:ident, $ty:expr, $default:literal, $fill:expr) => {
        static $name: [Ctor; 1] = [Ctor {
            access: None,
            params: &[Param {
                name: "amount",
                kind: INT,
                default: Some(Value::Int($default)),
                ty: "",
            }],
            make: |w, v| {
                let id = new_base_container(w, &$ty, 0xE76);
                let fill: fn(&mut World, ItemId, i32) = $fill;
                fill(w, id, v[0].i32());
                EntityId::Item(id)
            },
        }];
    };
}

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
amount_ctor!(ALCHEMY_CTORS, ALCHEMY_BAG, 5000, |w, id, a| alchemy_fill(
    w, id, a
));
amount_ctor!(INGOTS_CTORS, BAG_OF_INGOTS, 5000, |w, id, a| ingots_fill(
    w, id, a, None
));
amount_ctor!(SCRIBE_CTORS, SCRIBE_BAG, 5000, |w, id, a| scribe_fill(
    w, id, a
));
amount_ctor!(SMITH_CTORS, SMITH_BAG, 5000, |w, id, a| ingots_fill(
    w,
    id,
    a,
    Some(a)
));
amount_ctor!(TAILOR_CTORS, TAILOR_BAG, 500, |w, id, a| tailor_fill(
    w, id, a
));

/// `[Constructible] T(int amount = ..)` of each bag, for `[add AlchemyBag 100`.
pub fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.AlchemyBag", &ALCHEMY_CTORS);
    add_ctors(w, "Server.Items.BagOfingots", &INGOTS_CTORS);
    add_ctors(w, "Server.Items.ScribeBag", &SCRIBE_CTORS);
    add_ctors(w, "Server.Items.SmithBag", &SMITH_CTORS);
    add_ctors(w, "Server.Items.TailorBag", &TAILOR_CTORS);
}

/// Every class of this file, for the type registry.
pub static ALL_ITEMS: [&ItemType; 5] = [
    &ALCHEMY_BAG,
    &BAG_OF_INGOTS,
    &SCRIBE_BAG,
    &SMITH_BAG,
    &TAILOR_BAG,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bags_are_bags_with_the_expected_hue_and_name() {
        let mut w = World::new();
        for (ty, hue, name) in [
            (&ALCHEMY_BAG, 0x250, Some("an Alchemy Kit")),
            (&BAG_OF_INGOTS, 0, None),
            (&SCRIBE_BAG, 0x105, Some("a Scribe Kit")),
            (&SMITH_BAG, 0, None),
            (&TAILOR_BAG, 0x315, Some("a Tailoring Kit")),
        ] {
            let id = (ty.construct.unwrap())(&mut w);
            assert!(ty.is(&BAG));
            assert_eq!(w.item(id).unwrap().hue, hue);
            assert_eq!((ty.default_name)(&w, id).as_deref(), name);
        }
    }

    #[test]
    fn a_bag_round_trips_as_the_parent_plus_version_zero() {
        let mut w = World::new();
        let id = (SMITH_BAG.construct.unwrap())(&mut w);
        let mut writer = muo_io::serialization::save_writer::SaveWriter::new(true);
        (SMITH_BAG.serialize)(&w, id, &mut writer);
        let bytes = writer.into_vec();
        assert_eq!(
            &bytes[bytes.len() - 4..],
            [0, 0, 0, 0],
            "version int 0 ends the record"
        );
    }
}
