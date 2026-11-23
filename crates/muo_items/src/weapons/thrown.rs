//! Weapon classes of thrown; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.AbyssReaver`.
    ABYSS_REAVER, ABYSS_REAVER_CLASS, abyss_reaver_new = "Server.Items.AbyssReaver" : CYCLONE / CYCLONE_CLASS, via cyclone_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.BansheesCall`.
    BANSHEES_CALL, BANSHEES_CALL_CLASS, banshees_call_new = "Server.Items.BansheesCall" : CYCLONE / CYCLONE_CLASS, via cyclone_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Boomerang`.
    BOOMERANG, BOOMERANG_CLASS, boomerang_new = "Server.Items.Boomerang" : BASE_THROWN / BASE_THROWN_CLASS, json (), enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Cyclone`.
    CYCLONE, CYCLONE_CLASS, cyclone_new = "Server.Items.Cyclone" : BASE_THROWN / BASE_THROWN_CLASS, json (), enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.RaptorClaw`.
    RAPTOR_CLAW, RAPTOR_CLAW_CLASS, raptor_claw_new = "Server.Items.RaptorClaw" : BOOMERANG / BOOMERANG_CLASS, via boomerang_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.SoulGlaive`.
    SOUL_GLAIVE, SOUL_GLAIVE_CLASS, soul_glaive_new = "Server.Items.SoulGlaive" : BASE_THROWN / BASE_THROWN_CLASS, json (), enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.StoneSlithClaw`.
    STONE_SLITH_CLAW, STONE_SLITH_CLAW_CLASS, stone_slith_claw_new = "Server.Items.StoneSlithClaw" : CYCLONE / CYCLONE_CLASS, via cyclone_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.StormCaller`.
    STORM_CALLER, STORM_CALLER_CLASS, storm_caller_new = "Server.Items.StormCaller" : BOOMERANG / BOOMERANG_CLASS, via boomerang_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.ValkyriesGlaive`.
    VALKYRIES_GLAIVE, VALKYRIES_GLAIVE_CLASS, valkyries_glaive_new = "Server.Items.ValkyriesGlaive" : SOUL_GLAIVE / SOUL_GLAIVE_CLASS, via soul_glaive_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.WindOfCorruption`.
    WIND_OF_CORRUPTION, WIND_OF_CORRUPTION_CLASS, wind_of_corruption_new = "Server.Items.WindOfCorruption" : CYCLONE / CYCLONE_CLASS, via cyclone_new, enc;
    @json;
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ABYSS_REAVER, &[]);
    types.register_item(&BANSHEES_CALL, &[]);
    types.register_item(&BOOMERANG, &[]);
    types.register_item(&CYCLONE, &[]);
    types.register_item(&RAPTOR_CLAW, &[]);
    types.register_item(&SOUL_GLAIVE, &[]);
    types.register_item(&STONE_SLITH_CLAW, &[]);
    types.register_item(&STORM_CALLER, &[]);
    types.register_item(&VALKYRIES_GLAIVE, &[]);
    types.register_item(&WIND_OF_CORRUPTION, &[]);
}
