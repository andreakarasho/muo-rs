//! Weapon classes of maces; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.Club`.
    CLUB, CLUB_CLASS, club_new = "Server.Items.Club" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x13B4, v0;
    flippable: Some((0x13B4, 0x13B3)),
    default_weight: Some(9.0),
    primary_ability: Some(Ability::ShadowStrike),
    secondary_ability: Some(Ability::Dismount),
}

weapon_leaf! {
    /// `Server.Items.DiscMace`.
    DISC_MACE, DISC_MACE_CLASS, disc_mace_new = "Server.Items.DiscMace" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x903, enc;
    @item {
        required_races: |_, _| ALLOW_GARGOYLES_ONLY,
    }
    flippable: Some((0x903, 0x406E)),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::Disarm),
}

weapon_leaf! {
    /// `Server.Items.EmeraldMace`.
    EMERALD_MACE, EMERALD_MACE_CLASS, emerald_mace_new = "Server.Items.EmeraldMace" : DIAMOND_MACE / DIAMOND_MACE_CLASS, via diamond_mace_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.HammerPick`.
    HAMMER_PICK, HAMMER_PICK_CLASS, hammer_pick_new = "Server.Items.HammerPick" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x143D, v0;
    @json;
    flippable: Some((0x143D, 0x143C)),
    default_weight: Some(9.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::MortalStrike),
}

weapon_leaf! {
    /// `Server.Items.Mace`.
    MACE, MACE_CLASS, mace_new = "Server.Items.Mace" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0xF5C, v0;
    flippable: Some((0xF5C, 0xF5D)),
    default_weight: Some(14.0),
    primary_ability: Some(Ability::ConcussionBlow),
    secondary_ability: Some(Ability::Disarm),
}

weapon_leaf! {
    /// `Server.Items.MagicWand`.
    MAGIC_WAND, MAGIC_WAND_CLASS, magic_wand_new = "Server.Items.MagicWand" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0xDF2, v0;
    default_weight: Some(1.0),
    primary_ability: Some(Ability::Dismount),
    secondary_ability: Some(Ability::Disarm),
}

weapon_leaf! {
    /// `Server.Items.Maul`.
    MAUL, MAUL_CLASS, maul_new = "Server.Items.Maul" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x143B, v0;
    flippable: Some((0x143B, 0x143A)),
    default_weight: Some(10.0),
    primary_ability: Some(Ability::CrushingBlow),
    secondary_ability: Some(Ability::ConcussionBlow),
}

weapon_leaf! {
    /// `Server.Items.RubyMace`.
    RUBY_MACE, RUBY_MACE_CLASS, ruby_mace_new = "Server.Items.RubyMace" : DIAMOND_MACE / DIAMOND_MACE_CLASS, via diamond_mace_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.SapphireMace`.
    SAPPHIRE_MACE, SAPPHIRE_MACE_CLASS, sapphire_mace_new = "Server.Items.SapphireMace" : DIAMOND_MACE / DIAMOND_MACE_CLASS, via diamond_mace_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Scepter`.
    SCEPTER, SCEPTER_CLASS, scepter_new = "Server.Items.Scepter" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x26BC, v0;
    flippable: Some((0x26BC, 0x26C6)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::CrushingBlow),
    secondary_ability: Some(Ability::MortalStrike),
}

weapon_leaf! {
    /// `Server.Items.SilverEtchedMace`.
    SILVER_ETCHED_MACE, SILVER_ETCHED_MACE_CLASS, silver_etched_mace_new = "Server.Items.SilverEtchedMace" : DIAMOND_MACE / DIAMOND_MACE_CLASS, via diamond_mace_new, enc;
    @init |w, id| {
        ctor::set_slayer(w, id, SlayerName::Exorcism);
    };
}

weapon_leaf! {
    /// `Server.Items.WarHammer`.
    WAR_HAMMER, WAR_HAMMER_CLASS, war_hammer_new = "Server.Items.WarHammer" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x1439, v0;
    @json;
    flippable: Some((0x1439, 0x1438)),
    default_weight: Some(10.0),
    primary_ability: Some(Ability::WhirlwindAttack),
    secondary_ability: Some(Ability::CrushingBlow),
    def_animation: WeaponAnimation::Bash2H,
}

weapon_leaf! {
    /// `Server.Items.WarMace`.
    WAR_MACE, WAR_MACE_CLASS, war_mace_new = "Server.Items.WarMace" : BASE_BASHING / BASE_BASHING_CLASS, graphic 0x1407, v0;
    flippable: Some((0x1407, 0x1406)),
    default_weight: Some(17.0),
    primary_ability: Some(Ability::CrushingBlow),
    secondary_ability: Some(Ability::MortalStrike),
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&CLUB, &[]);
    types.register_item(&DISC_MACE, &[]);
    types.register_item(&EMERALD_MACE, &[]);
    types.register_item(&HAMMER_PICK, &[]);
    types.register_item(&MACE, &[]);
    types.register_item(&MAGIC_WAND, &[]);
    types.register_item(&MAUL, &[]);
    types.register_item(&RUBY_MACE, &[]);
    types.register_item(&SAPPHIRE_MACE, &[]);
    types.register_item(&SCEPTER, &[]);
    types.register_item(&SILVER_ETCHED_MACE, &[]);
    types.register_item(&WAR_HAMMER, &[]);
    types.register_item(&WAR_MACE, &[]);
}
