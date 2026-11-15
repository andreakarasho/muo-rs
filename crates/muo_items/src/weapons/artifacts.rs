//! Weapon classes of artifacts; see weapons/mod.rs.

use super::*;

weapon_leaf! {
    /// `Server.Items.AxeOfTheHeavens`.
    AXE_OF_THE_HEAVENS, AXE_OF_THE_HEAVENS_CLASS, axe_of_the_heavens_new = "Server.Items.AxeOfTheHeavens" : DOUBLE_AXE / DOUBLE_AXE_CLASS, via double_axe_new, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.BladeOfInsanity`.
    BLADE_OF_INSANITY, BLADE_OF_INSANITY_CLASS, blade_of_insanity_new = "Server.Items.BladeOfInsanity" : KATANA / KATANA_CLASS, via katana_new, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.BladeOfTheRighteous`.
    BLADE_OF_THE_RIGHTEOUS, BLADE_OF_THE_RIGHTEOUS_CLASS, blade_of_the_righteous_new = "Server.Items.BladeOfTheRighteous" : LONGSWORD / LONGSWORD_CLASS, via longsword_new, v0;
    @init |w, id| {
        ctor::set_hue(w, id, 0x47E);
        ctor::set_slayer(w, id, SlayerName::Exorcism);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechHits as u32, 50);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::UseBestSkill as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusHits as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 50);
    };
}

weapon_leaf! {
    /// `Server.Items.BoneCrusher`.
    BONE_CRUSHER, BONE_CRUSHER_CLASS, bone_crusher_new = "Server.Items.BoneCrusher" : WAR_MACE / WAR_MACE_CLASS, via war_mace_new, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.BreathOfTheDead`.
    BREATH_OF_THE_DEAD, BREATH_OF_THE_DEAD_CLASS, breath_of_the_dead_new = "Server.Items.BreathOfTheDead" : BONE_HARVESTER / BONE_HARVESTER_CLASS, via bone_harvester_new, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.Frostbringer`.
    FROSTBRINGER, FROSTBRINGER_CLASS, frostbringer_new = "Server.Items.Frostbringer" : BOW / BOW_CLASS, via bow_new, v0;
    @json;
    get_damage_types: |_, _, _| DamageTypes { phys: 0, fire: 0, cold: 100, pois: 0, nrgy: 0, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.LegacyOfTheDreadLord`.
    LEGACY_OF_THE_DREAD_LORD, LEGACY_OF_THE_DREAD_LORD_CLASS, legacy_of_the_dread_lord_new = "Server.Items.LegacyOfTheDreadLord" : BARDICHE / BARDICHE_CLASS, via bardiche_new, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.SerpentsFang`.
    SERPENTS_FANG, SERPENTS_FANG_CLASS, serpents_fang_new = "Server.Items.SerpentsFang" : KRYSS / KRYSS_CLASS, via kryss_new, v0;
    @json;
    get_damage_types: |_, _, _| DamageTypes { phys: 25, fire: 0, cold: 0, pois: 75, nrgy: 0, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.StaffOfTheMagi`.
    STAFF_OF_THE_MAGI, STAFF_OF_THE_MAGI_CLASS, staff_of_the_magi_new = "Server.Items.StaffOfTheMagi" : BLACK_STAFF / BLACK_STAFF_CLASS, via black_staff_new, v0;
    @json;
    get_damage_types: |_, _, _| DamageTypes { phys: 0, fire: 0, cold: 0, pois: 0, nrgy: 100, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.TheBeserkersMaul`.
    THE_BESERKERS_MAUL, THE_BESERKERS_MAUL_CLASS, the_beserkers_maul_new = "Server.Items.TheBeserkersMaul" : MAUL / MAUL_CLASS, via maul_new, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.TheDragonSlayer`.
    THE_DRAGON_SLAYER, THE_DRAGON_SLAYER_CLASS, the_dragon_slayer_new = "Server.Items.TheDragonSlayer" : LANCE / LANCE_CLASS, via lance_new, v0;
    @init |w, id| {
        ctor::set_hue(w, id, 0x530);
        ctor::set_slayer(w, id, SlayerName::DragonSlaying);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::Luck as u32, 110);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 50);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::ResistFireBonus as u32, 20);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::UseBestSkill as u32, 1);
    };
    get_damage_types: |_, _, _| DamageTypes { phys: 0, fire: 0, cold: 0, pois: 0, nrgy: 100, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.TheDryadBow`.
    THE_DRYAD_BOW, THE_DRYAD_BOW_CLASS, the_dryad_bow_new = "Server.Items.TheDryadBow" : BOW / BOW_CLASS, via bow_new, v0;
    @init |w, id| {
        ctor::set_item_id(w, id, 0x13B1);
        ctor::set_hue(w, id, 0x48F);
        const POSSIBLE: [SkillName; 6] = [SkillName::Archery, SkillName::Healing, SkillName::MagicResist, SkillName::Peacemaking, SkillName::Chivalry, SkillName::Ninjitsu]; let skill = *random_element(&mut w.rng, &POSSIBLE).unwrap(); let bonus = if random(&mut w.rng, 4) == 0 { 10.0 } else { 5.0 }; ctor::skill_bonus(w, id, 0, skill, bonus);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::SelfRepair as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 50);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 35);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::ResistPoisonBonus as u32, 15);
    };
}

weapon_leaf! {
    /// `Server.Items.TheTaskmaster`.
    THE_TASKMASTER, THE_TASKMASTER_CLASS, the_taskmaster_new = "Server.Items.TheTaskmaster" : WAR_FORK / WAR_FORK_CLASS, via war_fork_new, v0;
    @json;
    get_damage_types: |_, _, _| DamageTypes { phys: 0, fire: 0, cold: 0, pois: 100, nrgy: 0, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.TitansHammer`.
    TITANS_HAMMER, TITANS_HAMMER_CLASS, titans_hammer_new = "Server.Items.TitansHammer" : WAR_HAMMER / WAR_HAMMER_CLASS, via war_hammer_new, v0;
    @json;
}

weapon_leaf! {
    /// `Server.Items.ZyronicClaw`.
    ZYRONIC_CLAW, ZYRONIC_CLAW_CLASS, zyronic_claw_new = "Server.Items.ZyronicClaw" : EXECUTIONERS_AXE / EXECUTIONERS_AXE_CLASS, via executioners_axe_new, v0;
    @init |w, id| {
        ctor::set_hue(w, id, 0x485);
        ctor::set_slayer(w, id, SlayerName::ElementalBan);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLeechMana as u32, 50);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 30);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 50);
    };
    get_damage_types: |_, _, _| DamageTypes { phys: 20, fire: 20, cold: 20, pois: 20, nrgy: 20, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.BlightGrippedLongbow`.
    BLIGHT_GRIPPED_LONGBOW, BLIGHT_GRIPPED_LONGBOW_CLASS, blight_gripped_longbow_new = "Server.Items.BlightGrippedLongbow" : ELVEN_COMPOSITE_LONGBOW / ELVEN_COMPOSITE_LONGBOW_CLASS, via elven_composite_longbow_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.ColdForgedBlade`.
    COLD_FORGED_BLADE, COLD_FORGED_BLADE_CLASS, cold_forged_blade_new = "Server.Items.ColdForgedBlade" : ELVEN_SPELLBLADE / ELVEN_SPELLBLADE_CLASS, via elven_spellblade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitHarm as u32, 40);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::SpellChanneling as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::NightSight as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 25);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 50);
        let h = elemental_damage_hue(w, id); ctor::set_hue(w, id, h);
    };
    get_damage_types: |_, _, _| DamageTypes { phys: 0, fire: 0, cold: 100, pois: 0, nrgy: 0, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.FaerieFire`.
    FAERIE_FIRE, FAERIE_FIRE_CLASS, faerie_fire_new = "Server.Items.FaerieFire" : ELVEN_COMPOSITE_LONGBOW / ELVEN_COMPOSITE_LONGBOW_CLASS, via elven_composite_longbow_new, enc;
    @init |w, id| {
        ctor::set_hue(w, id, 0x489);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitFireball as u32, 25);
        ctor::set_balanced(w, id, true);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::BonusDex as u32, 3);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 20);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 60);
    };
    get_damage_types: |_, _, _| DamageTypes { phys: 0, fire: 100, cold: 0, pois: 0, nrgy: 0, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.LuminousRuneBlade`.
    LUMINOUS_RUNE_BLADE, LUMINOUS_RUNE_BLADE_CLASS, luminous_rune_blade_new = "Server.Items.LuminousRuneBlade" : RUNE_BLADE / RUNE_BLADE_CLASS, via rune_blade_new, enc;
    @init |w, id| {
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::HitLightning as u32, 40);
        set_attr(w, id, AttrKind::Weapon, AosWeaponAttribute::SelfRepair as u32, 5);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::NightSight as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 25);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 55);
        let h = elemental_damage_hue(w, id); ctor::set_hue(w, id, h);
    };
    get_damage_types: |_, _, _| DamageTypes { phys: 0, fire: 0, cold: 0, pois: 0, nrgy: 100, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.MischiefMaker`.
    MISCHIEF_MAKER, MISCHIEF_MAKER_CLASS, mischief_maker_new = "Server.Items.MischiefMaker" : MAGICAL_SHORTBOW / MAGICAL_SHORTBOW_CLASS, via magical_shortbow_new, enc;
    @init |w, id| {
        ctor::set_hue(w, id, 0x8AB);
        ctor::set_slayer(w, id, SlayerName::Silver);
        ctor::set_balanced(w, id, true);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 35);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 45);
    };
    get_damage_types: |_, _, _| DamageTypes { phys: 0, fire: 0, cold: 100, pois: 0, nrgy: 0, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.OverseerSunderedBlade`.
    OVERSEER_SUNDERED_BLADE, OVERSEER_SUNDERED_BLADE_CLASS, overseer_sundered_blade_new = "Server.Items.OverseerSunderedBlade" : RADIANT_SCIMITAR / RADIANT_SCIMITAR_CLASS, via radiant_scimitar_new, enc;
    @init |w, id| {
        ctor::set_item_id(w, id, 0x2D27);
        ctor::set_hue(w, id, 0x485);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::RegenStam as u32, 2);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::AttackChance as u32, 10);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 35);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 45);
        let h = elemental_damage_hue(w, id); ctor::set_hue(w, id, h);
    };
    get_damage_types: |_, _, _| DamageTypes { phys: 0, fire: 100, cold: 0, pois: 0, nrgy: 0, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.PhantomStaff`.
    PHANTOM_STAFF, PHANTOM_STAFF_CLASS, phantom_staff_new = "Server.Items.PhantomStaff" : WILD_STAFF / WILD_STAFF_CLASS, via wild_staff_new, enc;
    @json;
    get_damage_types: |_, _, _| DamageTypes { phys: 0, fire: 0, cold: 50, pois: 50, nrgy: 0, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.RuneCarvingKnife`.
    RUNE_CARVING_KNIFE, RUNE_CARVING_KNIFE_CLASS, rune_carving_knife_new = "Server.Items.RuneCarvingKnife" : ASSASSIN_SPIKE / ASSASSIN_SPIKE_CLASS, via assassin_spike_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.ShardThrasher`.
    SHARD_THRASHER, SHARD_THRASHER_CLASS, shard_thrasher_new = "Server.Items.ShardThrasher" : DIAMOND_MACE / DIAMOND_MACE_CLASS, via diamond_mace_new, enc;
    @json;
}

weapon_leaf! {
    /// `Server.Items.SilvanisFeywoodBow`.
    SILVANIS_FEYWOOD_BOW, SILVANIS_FEYWOOD_BOW_CLASS, silvanis_feywood_bow_new = "Server.Items.SilvanisFeywoodBow" : ELVEN_COMPOSITE_LONGBOW / ELVEN_COMPOSITE_LONGBOW_CLASS, via elven_composite_longbow_new, enc;
    @json;
    get_damage_types: |_, _, _| DamageTypes { phys: 0, fire: 0, cold: 0, pois: 0, nrgy: 100, chaos: 0, direct: 0 },
}

weapon_leaf! {
    /// `Server.Items.TheNightReaper`.
    THE_NIGHT_REAPER, THE_NIGHT_REAPER_CLASS, the_night_reaper_new = "Server.Items.TheNightReaper" : REPEATING_CROSSBOW / REPEATING_CROSSBOW_CLASS, via repeating_crossbow_new, v0;
    @init |w, id| {
        ctor::set_item_id(w, id, 0x26CD);
        ctor::set_hue(w, id, 0x41C);
        ctor::set_slayer(w, id, SlayerName::Exorcism);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::NightSight as u32, 1);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponSpeed as u32, 25);
        set_attr(w, id, AttrKind::Attributes, AosAttribute::WeaponDamage as u32, 55);
    };
}

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&AXE_OF_THE_HEAVENS, &[]);
    types.register_item(&BLADE_OF_INSANITY, &[]);
    types.register_item(&BLADE_OF_THE_RIGHTEOUS, &[]);
    types.register_item(&BONE_CRUSHER, &[]);
    types.register_item(&BREATH_OF_THE_DEAD, &[]);
    types.register_item(&FROSTBRINGER, &[]);
    types.register_item(&LEGACY_OF_THE_DREAD_LORD, &[]);
    types.register_item(&SERPENTS_FANG, &[]);
    types.register_item(&STAFF_OF_THE_MAGI, &[]);
    types.register_item(&THE_BESERKERS_MAUL, &[]);
    types.register_item(&THE_DRAGON_SLAYER, &[]);
    types.register_item(&THE_DRYAD_BOW, &[]);
    types.register_item(&THE_TASKMASTER, &[]);
    types.register_item(&TITANS_HAMMER, &[]);
    types.register_item(&ZYRONIC_CLAW, &[]);
    types.register_item(&BLIGHT_GRIPPED_LONGBOW, &[]);
    types.register_item(&COLD_FORGED_BLADE, &[]);
    types.register_item(&FAERIE_FIRE, &[]);
    types.register_item(&LUMINOUS_RUNE_BLADE, &[]);
    types.register_item(&MISCHIEF_MAKER, &[]);
    types.register_item(&OVERSEER_SUNDERED_BLADE, &[]);
    types.register_item(&PHANTOM_STAFF, &[]);
    types.register_item(&RUNE_CARVING_KNIFE, &[]);
    types.register_item(&SHARD_THRASHER, &[]);
    types.register_item(&SILVANIS_FEYWOOD_BOW, &[]);
    types.register_item(&THE_NIGHT_REAPER, &[]);
}
