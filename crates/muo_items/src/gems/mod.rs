//! The nine gem stacks, and `BaseJewel.GetGemType`/`GetGemItemType`, which need them. Each gem is `Stackable = true; Amount = amount` on a plain `Item`.

use muo_content::items::jewel::GemType;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, container, ops};
use muo_server::type_registry::{Phase, TypeRegistry};

/// `Gem(int amount = 1)` of class `ty`: `base(graphic)`, `Stackable = true; Amount = amount`.
pub fn new_gem(w: &mut World, ty: &'static ItemType, graphic: i32, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, graphic);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

macro_rules! gem_leaves {
    ($($(#[$m:meta])* $name:ident = $full:literal, graphic $gid:expr;)*) => {
        $(
            $(#[$m])*
            pub static $name: ItemType = ItemType {
                full_name: $full,
                parent: Some(&ITEM_BASE),
                construct: Some(|w| new_gem(w, &$name, $gid, 1)),
                default_weight: |_, _| 0.1,
                serialize: |w, id, out| {
                    (ITEM_BASE.serialize)(w, id, out);
                    out.write_i32(0); // [SerializationGenerator(0, false)]
                },
                deserialize: |w, id, r| {
                    (ITEM_BASE.deserialize)(w, id, r)?;
                    let _version = r.read_i32()?;
                    Ok(())
                },
                ..ITEM_BASE
            };
        )*

        /// Every gem class, for `register` and the tests.
        pub static ALL: &[&ItemType] = &[$(&$name),*];
    };
}

gem_leaves! {
    /// `Server.Items.Amber`.
    AMBER = "Server.Items.Amber", graphic 0xF25;
    /// `Server.Items.Amethyst`.
    AMETHYST = "Server.Items.Amethyst", graphic 0xF16;
    /// `Server.Items.Citrine`.
    CITRINE = "Server.Items.Citrine", graphic 0xF15;
    /// `Server.Items.Diamond`.
    DIAMOND = "Server.Items.Diamond", graphic 0xF26;
    /// `Server.Items.Emerald`.
    EMERALD = "Server.Items.Emerald", graphic 0xF10;
    /// `Server.Items.Ruby`.
    RUBY = "Server.Items.Ruby", graphic 0xF13;
    /// `Server.Items.Sapphire`.
    SAPPHIRE = "Server.Items.Sapphire", graphic 0xF19;
    /// `Server.Items.StarSapphire`.
    STAR_SAPPHIRE = "Server.Items.StarSapphire", graphic 0xF21;
    /// `Server.Items.Tourmaline`.
    TOURMALINE = "Server.Items.Tourmaline", graphic 0xF2D;
}

/// `BaseJewel.GetGemType(item)`: the gem an item of this class is (a gem class matches itself exactly, like the
/// type pattern on the sealed-in-practice gem classes; a subclass of one also matches, as `is` does).
pub fn gem_type_of(ty: &'static ItemType) -> GemType {
    for (g, t) in [
        (GemType::StarSapphire, &STAR_SAPPHIRE),
        (GemType::Emerald, &EMERALD),
        (GemType::Sapphire, &SAPPHIRE),
        (GemType::Ruby, &RUBY),
        (GemType::Citrine, &CITRINE),
        (GemType::Amethyst, &AMETHYST),
        (GemType::Tourmaline, &TOURMALINE),
        (GemType::Amber, &AMBER),
        (GemType::Diamond, &DIAMOND),
    ] {
        if ty.is(t) {
            return g;
        }
    }
    GemType::None
}

/// `BaseJewel.GetGemType(item)` for an item.
pub fn gem_type(w: &World, id: ItemId) -> GemType {
    w.item(id).map_or(GemType::None, |i| gem_type_of(i.ty))
}

/// `BaseJewel.GetGemItemType(type)`: the gem class, `None` for `GemType.None`.
pub fn gem_item_type(g: GemType) -> Option<&'static ItemType> {
    Some(match g {
        GemType::StarSapphire => &STAR_SAPPHIRE,
        GemType::Emerald => &EMERALD,
        GemType::Sapphire => &SAPPHIRE,
        GemType::Ruby => &RUBY,
        GemType::Citrine => &CITRINE,
        GemType::Amethyst => &AMETHYST,
        GemType::Tourmaline => &TOURMALINE,
        GemType::Amber => &AMBER,
        GemType::Diamond => &DIAMOND,
        GemType::None => return None,
    })
}

/// `from.Backpack?.ConsumeTotal(GetGemItemType(gem), count) == true`: the `EquipHooks::consume_gems` hook.
pub fn consume_gems(w: &mut World, from: MobileId, gem: GemType, count: i32) -> bool {
    let Some(ty) = gem_item_type(gem) else {
        return false;
    };
    let Some(pack) = muo_server::mobile::part1::props_b::backpack(w, from) else {
        return false;
    };
    container::consume_total(w, pack, ty, count, true, None)
}

/// Sets `EquipHooks::consume_gems` (T2A jewelry takes the gems from the crafter's backpack).
fn configure(w: &mut World) {
    muo_content::items::equip_hooks::hooks_mut(w).consume_gems = Some(consume_gems);
}

/// Registers the classes of this file.
pub fn register(types: &mut TypeRegistry) {
    for t in ALL {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BaseJewel.Configure",
        configure,
    );
}
