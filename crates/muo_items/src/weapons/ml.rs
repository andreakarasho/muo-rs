//! Weapon classes of ml; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.AssassinSpike`.
    ASSASSIN_SPIKE, ASSASSIN_SPIKE_CLASS, assassin_spike_new = "Server.Items.AssassinSpike" : BASE_KNIFE / BASE_KNIFE_CLASS, json (), enc;
}

weapon_leaf! {
    /// `Server.Items.ButchersWarCleaver`.
    BUTCHERS_WAR_CLEAVER, BUTCHERS_WAR_CLEAVER_CLASS, butchers_war_cleaver_new = "Server.Items.ButchersWarCleaver" : WAR_CLEAVER / WAR_CLEAVER_CLASS, via war_cleaver_new, enc;
    @item {
        append_child_name_properties: butchers_war_cleaver_append_child_name_properties,
    }
}

weapon_leaf! {
    /// `Server.Items.DiamondMace`.
    DIAMOND_MACE, DIAMOND_MACE_CLASS, diamond_mace_new = "Server.Items.DiamondMace" : BASE_BASHING / BASE_BASHING_CLASS, json (), enc;
}

weapon_leaf! {
    /// `Server.Items.ElvenCompositeLongbow`.
    ELVEN_COMPOSITE_LONGBOW, ELVEN_COMPOSITE_LONGBOW_CLASS, elven_composite_longbow_new = "Server.Items.ElvenCompositeLongbow" : BASE_RANGED / BASE_RANGED_CLASS, json (), enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.ElvenMachete`.
    ELVEN_MACHETE, ELVEN_MACHETE_CLASS, elven_machete_new = "Server.Items.ElvenMachete" : BASE_SWORD / BASE_SWORD_CLASS, json (), enc;
}

weapon_leaf! {
    /// `Server.Items.ElvenSpellblade`.
    ELVEN_SPELLBLADE, ELVEN_SPELLBLADE_CLASS, elven_spellblade_new = "Server.Items.ElvenSpellblade" : BASE_KNIFE / BASE_KNIFE_CLASS, json (), enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Leafblade`.
    LEAFBLADE, LEAFBLADE_CLASS, leafblade_new = "Server.Items.Leafblade" : BASE_KNIFE / BASE_KNIFE_CLASS, json (), enc;
}

weapon_leaf! {
    /// `Server.Items.MagicalShortbow`.
    MAGICAL_SHORTBOW, MAGICAL_SHORTBOW_CLASS, magical_shortbow_new = "Server.Items.MagicalShortbow" : BASE_RANGED / BASE_RANGED_CLASS, json (), enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.OrnateAxe`.
    ORNATE_AXE, ORNATE_AXE_CLASS, ornate_axe_new = "Server.Items.OrnateAxe" : BASE_AXE / BASE_AXE_CLASS, json (), enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.RadiantScimitar`.
    RADIANT_SCIMITAR, RADIANT_SCIMITAR_CLASS, radiant_scimitar_new = "Server.Items.RadiantScimitar" : BASE_SWORD / BASE_SWORD_CLASS, json (), enc;
}

weapon_leaf! {
    /// `Server.Items.RuneBlade`.
    RUNE_BLADE, RUNE_BLADE_CLASS, rune_blade_new = "Server.Items.RuneBlade" : BASE_SWORD / BASE_SWORD_CLASS, json (), enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.WarCleaver`.
    WAR_CLEAVER, WAR_CLEAVER_CLASS, war_cleaver_new = "Server.Items.WarCleaver" : BASE_KNIFE / BASE_KNIFE_CLASS, json (), enc;
}

weapon_leaf! {
    /// `Server.Items.WildStaff`.
    WILD_STAFF, WILD_STAFF_CLASS, wild_staff_new = "Server.Items.WildStaff" : BASE_STAFF / BASE_STAFF_CLASS, json (), enc;
    @json;
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ASSASSIN_SPIKE, &[]);
    types.register_item(&BUTCHERS_WAR_CLEAVER, &[]);
    types.register_item(&DIAMOND_MACE, &[]);
    types.register_item(&ELVEN_COMPOSITE_LONGBOW, &[]);
    types.register_item(&ELVEN_MACHETE, &[]);
    types.register_item(&ELVEN_SPELLBLADE, &[]);
    types.register_item(&LEAFBLADE, &[]);
    types.register_item(&MAGICAL_SHORTBOW, &[]);
    types.register_item(&ORNATE_AXE, &[]);
    types.register_item(&RADIANT_SCIMITAR, &[]);
    types.register_item(&RUNE_BLADE, &[]);
    types.register_item(&WAR_CLEAVER, &[]);
    types.register_item(&WILD_STAFF, &[]);
}
