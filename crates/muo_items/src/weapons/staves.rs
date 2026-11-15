//! Weapon classes of staves; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.BlackStaff`.
    BLACK_STAFF, BLACK_STAFF_CLASS, black_staff_new = "Server.Items.BlackStaff" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0xDF0, v0;
}

// ChylothStaff (namespace Server.Engines.Quests.Doom)
weapon_leaf! {
    /// `Server.Engines.Quests.Doom.ChylothStaff`.
    CHYLOTH_STAFF, CHYLOTH_STAFF_CLASS, chyloth_staff_new = "Server.Engines.Quests.Doom.ChylothStaff" : BLACK_STAFF / BLACK_STAFF_CLASS, via black_staff_new, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.GlacialStaff`.
    GLACIAL_STAFF, GLACIAL_STAFF_CLASS, glacial_staff_new = "Server.Items.GlacialStaff" : BLACK_STAFF / BLACK_STAFF_CLASS, via black_staff_new, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.GlassStaff`.
    GLASS_STAFF, GLASS_STAFF_CLASS, glass_staff_new = "Server.Items.GlassStaff" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0x905, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.GnarledStaff`.
    GNARLED_STAFF, GNARLED_STAFF_CLASS, gnarled_staff_new = "Server.Items.GnarledStaff" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0x13F8, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.QuarterStaff`.
    QUARTER_STAFF, QUARTER_STAFF_CLASS, quarter_staff_new = "Server.Items.QuarterStaff" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0xE89, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.SerpentstoneStaff`.
    SERPENTSTONE_STAFF, SERPENTSTONE_STAFF_CLASS, serpentstone_staff_new = "Server.Items.SerpentstoneStaff" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0x906, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.ShepherdsCrook`.
    SHEPHERDS_CROOK, SHEPHERDS_CROOK_CLASS, shepherds_crook_new = "Server.Items.ShepherdsCrook" : BASE_STAFF / BASE_STAFF_CLASS, graphic 0xE81, v0;
    @json;
    @item {
        on_double_click: shepherds_crook_on_double_click,
    }
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BLACK_STAFF, &[]);
    types.register_item(&GLACIAL_STAFF, &[]);
    types.register_item(&CHYLOTH_STAFF, &[]);
    types.register_item(&GLASS_STAFF, &[]);
    types.register_item(&GNARLED_STAFF, &[]);
    types.register_item(&QUARTER_STAFF, &[]);
    types.register_item(&SERPENTSTONE_STAFF, &["Server.Items.SerpentStoneStaff"]);
    types.register_item(&SHEPHERDS_CROOK, &[]);
}
