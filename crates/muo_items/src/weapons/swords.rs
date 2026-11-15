//! Weapon classes of swords; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.AdventurersMachete`.
    ADVENTURERS_MACHETE, ADVENTURERS_MACHETE_CLASS, adventurers_machete_new = "Server.Items.AdventurersMachete" : ELVEN_MACHETE / ELVEN_MACHETE_CLASS, via elven_machete_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::Luck as u32, 20);
    };
    label_number: 1073533,
}

weapon_leaf! {
    /// `Server.Items.BloodBlade`.
    BLOOD_BLADE, BLOOD_BLADE_CLASS, blood_blade_new = "Server.Items.BloodBlade" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x8FE, enc;
    @item {
        required_races: |_, _| ALLOW_GARGOYLES_ONLY,
    }
    flippable: Some((0x8FE, 0x4072)),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::ParalyzingBlow),
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.BoneHarvester`.
    BONE_HARVESTER, BONE_HARVESTER_CLASS, bone_harvester_new = "Server.Items.BoneHarvester" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x26BB, v0;
    flippable: Some((0x26BB, 0x26C5)),
    default_weight: Some(3.0),
    primary_ability: Some(Ability::ParalyzingBlow),
    secondary_ability: Some(Ability::MortalStrike),
}

weapon_leaf! {
    /// `Server.Items.BoneMachete`.
    BONE_MACHETE, BONE_MACHETE_CLASS, bone_machete_new = "Server.Items.BoneMachete" : ELVEN_MACHETE / ELVEN_MACHETE_CLASS, via elven_machete_new, enc;
    @init |w, id| {
        ctor::set_item_id(w, id, 0x20E);
    };
    @item {
        physical_resistance: |_, _| 1,
        fire_resistance: |_, _| 1,
        cold_resistance: |_, _| 1,
        poison_resistance: |_, _| 1,
        energy_resistance: |_, _| 1,
    }
    primary_ability: None,
    secondary_ability: None,
}

weapon_leaf! {
    /// `Server.Items.Broadsword`.
    BROADSWORD, BROADSWORD_CLASS, broadsword_new = "Server.Items.Broadsword" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0xF5E, v0;
    flippable: Some((0xF5E, 0xF5F)),
    default_weight: Some(6.0),
    primary_ability: Some(Ability::CrushingBlow),
    secondary_ability: Some(Ability::ArmorIgnore),
}

weapon_leaf! {
    /// `Server.Items.ChargedAssassinSpike`.
    CHARGED_ASSASSIN_SPIKE, CHARGED_ASSASSIN_SPIKE_CLASS, charged_assassin_spike_new = "Server.Items.ChargedAssassinSpike" : ASSASSIN_SPIKE / ASSASSIN_SPIKE_CLASS, via assassin_spike_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLightning as u32, 10);
    };
    label_number: 1073518,
}

weapon_leaf! {
    /// `Server.Items.CorruptedRuneBlade`.
    CORRUPTED_RUNE_BLADE, CORRUPTED_RUNE_BLADE_CLASS, corrupted_rune_blade_new = "Server.Items.CorruptedRuneBlade" : RUNE_BLADE / RUNE_BLADE_CLASS, via rune_blade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::ResistPhysicalBonus as u32, -5);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::ResistPoisonBonus as u32, 12);
    };
    label_number: 1073540,
}

weapon_leaf! {
    /// `Server.Items.CrescentBlade`.
    CRESCENT_BLADE, CRESCENT_BLADE_CLASS, crescent_blade_new = "Server.Items.CrescentBlade" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x26C1, v0;
    flippable: Some((0x26C1, 0x26CB)),
    default_weight: Some(1.0),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::MortalStrike),
}

weapon_leaf! {
    /// `Server.Items.Cutlass`.
    CUTLASS, CUTLASS_CLASS, cutlass_new = "Server.Items.Cutlass" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x1441, v0;
    flippable: Some((0x1441, 0x1440)),
    default_weight: Some(8.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::ShadowStrike),
}

weapon_leaf! {
    /// `Server.Items.DarkglowScimitar`.
    DARKGLOW_SCIMITAR, DARKGLOW_SCIMITAR_CLASS, darkglow_scimitar_new = "Server.Items.DarkglowScimitar" : RADIANT_SCIMITAR / RADIANT_SCIMITAR_CLASS, via radiant_scimitar_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitDispel as u32, 10);
    };
    label_number: 1073542,
}

weapon_leaf! {
    /// `Server.Items.DiseasedMachete`.
    DISEASED_MACHETE, DISEASED_MACHETE_CLASS, diseased_machete_new = "Server.Items.DiseasedMachete" : ELVEN_MACHETE / ELVEN_MACHETE_CLASS, via elven_machete_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitPoisonArea as u32, 25);
    };
    label_number: 1073536,
}

weapon_leaf! {
    /// `Server.Items.DreadSword`.
    DREAD_SWORD, DREAD_SWORD_CLASS, dread_sword_new = "Server.Items.DreadSword" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x90B, enc;
    @item {
        required_races: |_, _| ALLOW_GARGOYLES_ONLY,
    }
    flippable: Some((0x90B, 0x4074)),
    primary_ability: Some(Ability::CrushingBlow),
    secondary_ability: Some(Ability::ConcussionBlow),
}

weapon_leaf! {
    /// `Server.Items.FierySpellblade`.
    FIERY_SPELLBLADE, FIERY_SPELLBLADE_CLASS, fiery_spellblade_new = "Server.Items.FierySpellblade" : ELVEN_SPELLBLADE / ELVEN_SPELLBLADE_CLASS, via elven_spellblade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::ResistFireBonus as u32, 5);
    };
    label_number: 1073515,
}

weapon_leaf! {
    /// `Server.Items.GargishTalwar`.
    GARGISH_TALWAR, GARGISH_TALWAR_CLASS, gargish_talwar_new = "Server.Items.GargishTalwar" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x908, enc;
    @item {
        required_races: |_, _| ALLOW_GARGOYLES_ONLY,
    }
    flippable: Some((0x908, 0x4075)),
    primary_ability: Some(Ability::WhirlwindAttack),
    secondary_ability: Some(Ability::Dismount),
}

weapon_leaf! {
    /// `Server.Items.GlassSword`.
    GLASS_SWORD, GLASS_SWORD_CLASS, glass_sword_new = "Server.Items.GlassSword" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x90C, enc;
    @item {
        required_races: |_, _| ALLOW_GARGOYLES_ONLY,
    }
    flippable: Some((0x90C, 0x4073)),
    default_weight: Some(6.0),
    primary_ability: Some(Ability::BleedAttack),
    secondary_ability: Some(Ability::MortalStrike),
}

weapon_leaf! {
    /// `Server.Items.IcyScimitar`.
    ICY_SCIMITAR, ICY_SCIMITAR_CLASS, icy_scimitar_new = "Server.Items.IcyScimitar" : RADIANT_SCIMITAR / RADIANT_SCIMITAR_CLASS, via radiant_scimitar_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitHarm as u32, 15);
    };
    label_number: 1073543,
}

weapon_leaf! {
    /// `Server.Items.IcySpellblade`.
    ICY_SPELLBLADE, ICY_SPELLBLADE_CLASS, icy_spellblade_new = "Server.Items.IcySpellblade" : ELVEN_SPELLBLADE / ELVEN_SPELLBLADE_CLASS, via elven_spellblade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::ResistColdBonus as u32, 5);
    };
    label_number: 1073514,
}

weapon_leaf! {
    /// `Server.Items.KnightsWarCleaver`.
    KNIGHTS_WAR_CLEAVER, KNIGHTS_WAR_CLEAVER_CLASS, knights_war_cleaver_new = "Server.Items.KnightsWarCleaver" : WAR_CLEAVER / WAR_CLEAVER_CLASS, via war_cleaver_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenHits as u32, 3);
    };
    label_number: 1073525,
}

weapon_leaf! {
    /// `Server.Items.Kryss`.
    KRYSS, KRYSS_CLASS, kryss_new = "Server.Items.Kryss" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x1401, v0;
    flippable: Some((0x1401, 0x1400)),
    default_weight: Some(2.0),
    primary_ability: Some(Ability::ArmorIgnore),
    secondary_ability: Some(Ability::InfectiousStrike),
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.Lance`.
    LANCE, LANCE_CLASS, lance_new = "Server.Items.Lance" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x26C0, v0;
    flippable: Some((0x26C0, 0x26CA)),
    default_weight: Some(12.0),
    primary_ability: Some(Ability::Dismount),
    secondary_ability: Some(Ability::ConcussionBlow),
    def_skill: SkillName::Fencing,
    def_type: WeaponType::Piercing,
    def_animation: WeaponAnimation::Pierce1H,
}

weapon_leaf! {
    /// `Server.Items.LeafbladeOfEase`.
    LEAFBLADE_OF_EASE, LEAFBLADE_OF_EASE_CLASS, leafblade_of_ease_new = "Server.Items.LeafbladeOfEase" : LEAFBLADE / LEAFBLADE_CLASS, via leafblade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::UseBestSkill as u32, 1);
    };
    label_number: 1073524,
}

weapon_leaf! {
    /// `Server.Items.Luckblade`.
    LUCKBLADE, LUCKBLADE_CLASS, luckblade_new = "Server.Items.Luckblade" : LEAFBLADE / LEAFBLADE_CLASS, via leafblade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::Luck as u32, 20);
    };
    label_number: 1073522,
}

weapon_leaf! {
    /// `Server.Items.MacheteOfDefense`.
    MACHETE_OF_DEFENSE, MACHETE_OF_DEFENSE_CLASS, machete_of_defense_new = "Server.Items.MacheteOfDefense" : ELVEN_MACHETE / ELVEN_MACHETE_CLASS, via elven_machete_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 5);
    };
    label_number: 1073535,
}

weapon_leaf! {
    /// `Server.Items.MagekillerAssassinSpike`.
    MAGEKILLER_ASSASSIN_SPIKE, MAGEKILLER_ASSASSIN_SPIKE_CLASS, magekiller_assassin_spike_new = "Server.Items.MagekillerAssassinSpike" : ASSASSIN_SPIKE / ASSASSIN_SPIKE_CLASS, via assassin_spike_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechMana as u32, 16);
    };
    label_number: 1073519,
}

weapon_leaf! {
    /// `Server.Items.MagekillerLeafblade`.
    MAGEKILLER_LEAFBLADE, MAGEKILLER_LEAFBLADE_CLASS, magekiller_leafblade_new = "Server.Items.MagekillerLeafblade" : LEAFBLADE / LEAFBLADE_CLASS, via leafblade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechMana as u32, 16);
    };
    label_number: 1073523,
}

weapon_leaf! {
    /// `Server.Items.MagesRuneBlade`.
    MAGES_RUNE_BLADE, MAGES_RUNE_BLADE_CLASS, mages_rune_blade_new = "Server.Items.MagesRuneBlade" : RUNE_BLADE / RUNE_BLADE_CLASS, via rune_blade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::CastSpeed as u32, 1);
    };
    label_number: 1073538,
}

weapon_leaf! {
    /// `Server.Items.OrcishMachete`.
    ORCISH_MACHETE, ORCISH_MACHETE_CLASS, orcish_machete_new = "Server.Items.OrcishMachete" : ELVEN_MACHETE / ELVEN_MACHETE_CLASS, via elven_machete_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusInt as u32, -5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 10);
    };
    label_number: 1073534,
}

weapon_leaf! {
    /// `Server.Items.RuneBladeOfKnowledge`.
    RUNE_BLADE_OF_KNOWLEDGE, RUNE_BLADE_OF_KNOWLEDGE_CLASS, rune_blade_of_knowledge_new = "Server.Items.RuneBladeOfKnowledge" : RUNE_BLADE / RUNE_BLADE_CLASS, via rune_blade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellDamage as u32, 5);
    };
    label_number: 1073539,
}

weapon_leaf! {
    /// `Server.Items.Runesabre`.
    RUNESABRE, RUNESABRE_CLASS, runesabre_new = "Server.Items.Runesabre" : RUNE_BLADE / RUNE_BLADE_CLASS, via rune_blade_new, enc;
    @init |w, id| {
        ctor::skill_bonus(w, id, 0, SkillName::MagicResist, 5.0);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::MageWeapon as u32, -29);
    };
    label_number: 1073537,
}

weapon_leaf! {
    /// `Server.Items.Scimitar`.
    SCIMITAR, SCIMITAR_CLASS, scimitar_new = "Server.Items.Scimitar" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x13B6, v0;
    flippable: Some((0x13B6, 0x13B5)),
    default_weight: Some(5.0),
    primary_ability: Some(Ability::DoubleStrike),
    secondary_ability: Some(Ability::ParalyzingBlow),
}

weapon_leaf! {
    /// `Server.Items.SerratedWarCleaver`.
    SERRATED_WAR_CLEAVER, SERRATED_WAR_CLEAVER_CLASS, serrated_war_cleaver_new = "Server.Items.SerratedWarCleaver" : WAR_CLEAVER / WAR_CLEAVER_CLASS, via war_cleaver_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 7);
    };
    label_number: 1073527,
}

weapon_leaf! {
    /// `Server.Items.SpellbladeOfDefense`.
    SPELLBLADE_OF_DEFENSE, SPELLBLADE_OF_DEFENSE_CLASS, spellblade_of_defense_new = "Server.Items.SpellbladeOfDefense" : ELVEN_SPELLBLADE / ELVEN_SPELLBLADE_CLASS, via elven_spellblade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 5);
    };
    label_number: 1073516,
}

weapon_leaf! {
    /// `Server.Items.ThinLongsword`.
    THIN_LONGSWORD, THIN_LONGSWORD_CLASS, thin_longsword_new = "Server.Items.ThinLongsword" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x13B8, v0;
    flippable: Some((0x13B8, 0x13B7)),
    default_weight: Some(1.0),
}

weapon_leaf! {
    /// `Server.Items.TrueAssassinSpike`.
    TRUE_ASSASSIN_SPIKE, TRUE_ASSASSIN_SPIKE_CLASS, true_assassin_spike_new = "Server.Items.TrueAssassinSpike" : ASSASSIN_SPIKE / ASSASSIN_SPIKE_CLASS, via assassin_spike_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 4);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 4);
    };
    label_number: 1073517,
}

weapon_leaf! {
    /// `Server.Items.TrueLeafblade`.
    TRUE_LEAFBLADE, TRUE_LEAFBLADE_CLASS, true_leafblade_new = "Server.Items.TrueLeafblade" : LEAFBLADE / LEAFBLADE_CLASS, via leafblade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::ResistPoisonBonus as u32, 5);
    };
    label_number: 1073521,
}

weapon_leaf! {
    /// `Server.Items.TrueRadiantScimitar`.
    TRUE_RADIANT_SCIMITAR, TRUE_RADIANT_SCIMITAR_CLASS, true_radiant_scimitar_new = "Server.Items.TrueRadiantScimitar" : RADIANT_SCIMITAR / RADIANT_SCIMITAR_CLASS, via radiant_scimitar_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::NightSight as u32, 1);
    };
    label_number: 1073541,
}

weapon_leaf! {
    /// `Server.Items.TrueSpellblade`.
    TRUE_SPELLBLADE, TRUE_SPELLBLADE_CLASS, true_spellblade_new = "Server.Items.TrueSpellblade" : ELVEN_SPELLBLADE / ELVEN_SPELLBLADE_CLASS, via elven_spellblade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellChanneling as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::CastSpeed as u32, -1);
    };
    label_number: 1073513,
}

weapon_leaf! {
    /// `Server.Items.TrueWarCleaver`.
    TRUE_WAR_CLEAVER, TRUE_WAR_CLEAVER_CLASS, true_war_cleaver_new = "Server.Items.TrueWarCleaver" : WAR_CLEAVER / WAR_CLEAVER_CLASS, via war_cleaver_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 4);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenHits as u32, 2);
    };
    label_number: 1073528,
}

weapon_leaf! {
    /// `Server.Items.TwinklingScimitar`.
    TWINKLING_SCIMITAR, TWINKLING_SCIMITAR_CLASS, twinkling_scimitar_new = "Server.Items.TwinklingScimitar" : RADIANT_SCIMITAR / RADIANT_SCIMITAR_CLASS, via radiant_scimitar_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Attributes, AosAttribute::DefendChance as u32, 6);
    };
    label_number: 1073544,
}

weapon_leaf! {
    /// `Server.Items.VikingSword`.
    VIKING_SWORD, VIKING_SWORD_CLASS, viking_sword_new = "Server.Items.VikingSword" : BASE_SWORD / BASE_SWORD_CLASS, graphic 0x13B9, v0;
    flippable: Some((0x13B9, 0x13BA)),
    default_weight: Some(6.0),
    primary_ability: Some(Ability::CrushingBlow),
    secondary_ability: Some(Ability::ParalyzingBlow),
}

weapon_leaf! {
    /// `Server.Items.WoundingAssassinSpike`.
    WOUNDING_ASSASSIN_SPIKE, WOUNDING_ASSASSIN_SPIKE_CLASS, wounding_assassin_spike_new = "Server.Items.WoundingAssassinSpike" : ASSASSIN_SPIKE / ASSASSIN_SPIKE_CLASS, via assassin_spike_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitHarm as u32, 15);
    };
    label_number: 1073520,
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&ADVENTURERS_MACHETE, &[]);
    types.register_item(&BLOOD_BLADE, &[]);
    types.register_item(&BONE_HARVESTER, &[]);
    types.register_item(&BONE_MACHETE, &[]);
    types.register_item(&BROADSWORD, &[]);
    types.register_item(&CHARGED_ASSASSIN_SPIKE, &[]);
    types.register_item(&CORRUPTED_RUNE_BLADE, &[]);
    types.register_item(&CRESCENT_BLADE, &[]);
    types.register_item(&CUTLASS, &[]);
    types.register_item(&DARKGLOW_SCIMITAR, &[]);
    types.register_item(&DISEASED_MACHETE, &[]);
    types.register_item(&DREAD_SWORD, &[]);
    types.register_item(&FIERY_SPELLBLADE, &[]);
    types.register_item(&GARGISH_TALWAR, &[]);
    types.register_item(&GLASS_SWORD, &[]);
    types.register_item(&ICY_SCIMITAR, &[]);
    types.register_item(&ICY_SPELLBLADE, &[]);
    types.register_item(&KNIGHTS_WAR_CLEAVER, &[]);
    types.register_item(&KRYSS, &[]);
    types.register_item(&LANCE, &[]);
    types.register_item(&LEAFBLADE_OF_EASE, &[]);
    types.register_item(&LUCKBLADE, &[]);
    types.register_item(&MACHETE_OF_DEFENSE, &[]);
    types.register_item(&MAGEKILLER_ASSASSIN_SPIKE, &[]);
    types.register_item(&MAGEKILLER_LEAFBLADE, &[]);
    types.register_item(&MAGES_RUNE_BLADE, &[]);
    types.register_item(&ORCISH_MACHETE, &[]);
    types.register_item(&RUNE_BLADE_OF_KNOWLEDGE, &[]);
    types.register_item(&RUNESABRE, &[]);
    types.register_item(&SCIMITAR, &[]);
    types.register_item(&SERRATED_WAR_CLEAVER, &[]);
    types.register_item(&SPELLBLADE_OF_DEFENSE, &[]);
    types.register_item(&THIN_LONGSWORD, &[]);
    types.register_item(&TRUE_ASSASSIN_SPIKE, &[]);
    types.register_item(&TRUE_LEAFBLADE, &[]);
    types.register_item(&TRUE_RADIANT_SCIMITAR, &[]);
    types.register_item(&TRUE_SPELLBLADE, &[]);
    types.register_item(&TRUE_WAR_CLEAVER, &[]);
    types.register_item(&TWINKLING_SCIMITAR, &[]);
    types.register_item(&VIKING_SWORD, &[]);
    types.register_item(&WOUNDING_ASSASSIN_SPIKE, &[]);
}
