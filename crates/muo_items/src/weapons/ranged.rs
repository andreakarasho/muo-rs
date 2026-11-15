//! Weapon classes of ranged; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.AssassinsShortbow`.
    ASSASSINS_SHORTBOW, ASSASSINS_SHORTBOW_CLASS, assassins_shortbow_new = "Server.Items.AssassinsShortbow" : MAGICAL_SHORTBOW / MAGICAL_SHORTBOW_CLASS, via magical_shortbow_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.BarbedLongbow`.
    BARBED_LONGBOW, BARBED_LONGBOW_CLASS, barbed_longbow_new = "Server.Items.BarbedLongbow" : ELVEN_COMPOSITE_LONGBOW / ELVEN_COMPOSITE_LONGBOW_CLASS, via elven_composite_longbow_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.CompositeBow`.
    COMPOSITE_BOW, COMPOSITE_BOW_CLASS, composite_bow_new = "Server.Items.CompositeBow" : BASE_RANGED / BASE_RANGED_CLASS, graphic 0x26C2, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Crossbow`.
    CROSSBOW, CROSSBOW_CLASS, crossbow_new = "Server.Items.Crossbow" : BASE_RANGED / BASE_RANGED_CLASS, graphic 0xF50, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.FrozenLongbow`.
    FROZEN_LONGBOW, FROZEN_LONGBOW_CLASS, frozen_longbow_new = "Server.Items.FrozenLongbow" : ELVEN_COMPOSITE_LONGBOW / ELVEN_COMPOSITE_LONGBOW_CLASS, via elven_composite_longbow_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.HeavyCrossbow`.
    HEAVY_CROSSBOW, HEAVY_CROSSBOW_CLASS, heavy_crossbow_new = "Server.Items.HeavyCrossbow" : BASE_RANGED / BASE_RANGED_CLASS, graphic 0x13FD, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.JukaBow`.
    JUKA_BOW, JUKA_BOW_CLASS, juka_bow_new = "Server.Items.JukaBow" : BOW / BOW_CLASS, via bow_new, v0;
    @json;
    @item {
        on_double_click: juka_bow_on_double_click,
    }
}

weapon_leaf! {
    /// `Server.Items.LightweightShortbow`.
    LIGHTWEIGHT_SHORTBOW, LIGHTWEIGHT_SHORTBOW_CLASS, lightweight_shortbow_new = "Server.Items.LightweightShortbow" : MAGICAL_SHORTBOW / MAGICAL_SHORTBOW_CLASS, via magical_shortbow_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.LongbowOfMight`.
    LONGBOW_OF_MIGHT, LONGBOW_OF_MIGHT_CLASS, longbow_of_might_new = "Server.Items.LongbowOfMight" : ELVEN_COMPOSITE_LONGBOW / ELVEN_COMPOSITE_LONGBOW_CLASS, via elven_composite_longbow_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.MysticalShortbow`.
    MYSTICAL_SHORTBOW, MYSTICAL_SHORTBOW_CLASS, mystical_shortbow_new = "Server.Items.MysticalShortbow" : MAGICAL_SHORTBOW / MAGICAL_SHORTBOW_CLASS, via magical_shortbow_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.RangersShortbow`.
    RANGERS_SHORTBOW, RANGERS_SHORTBOW_CLASS, rangers_shortbow_new = "Server.Items.RangersShortbow" : MAGICAL_SHORTBOW / MAGICAL_SHORTBOW_CLASS, via magical_shortbow_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.RepeatingCrossbow`.
    REPEATING_CROSSBOW, REPEATING_CROSSBOW_CLASS, repeating_crossbow_new = "Server.Items.RepeatingCrossbow" : BASE_RANGED / BASE_RANGED_CLASS, graphic 0x26C3, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.SlayerLongbow`.
    SLAYER_LONGBOW, SLAYER_LONGBOW_CLASS, slayer_longbow_new = "Server.Items.SlayerLongbow" : ELVEN_COMPOSITE_LONGBOW / ELVEN_COMPOSITE_LONGBOW_CLASS, via elven_composite_longbow_new, enc;

}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ASSASSINS_SHORTBOW, &[]);
    types.register_item(&BARBED_LONGBOW, &[]);
    types.register_item(&COMPOSITE_BOW, &[]);
    types.register_item(&CROSSBOW, &[]);
    types.register_item(&FROZEN_LONGBOW, &[]);
    types.register_item(&HEAVY_CROSSBOW, &[]);
    types.register_item(&JUKA_BOW, &[]);
    types.register_item(&LIGHTWEIGHT_SHORTBOW, &[]);
    types.register_item(&LONGBOW_OF_MIGHT, &[]);
    types.register_item(&MYSTICAL_SHORTBOW, &[]);
    types.register_item(&RANGERS_SHORTBOW, &[]);
    types.register_item(&REPEATING_CROSSBOW, &[]);
    types.register_item(&SLAYER_LONGBOW, &[]);
}
