//! Every `SpellScroll` subclass.

use muo_content::props::Ctor;
use muo_server::ids::EntityId;
use muo_server::item::ItemType;

use super::scrolls::*;

scrolls! {
    ARCANE_CIRCLE_SCROLL, ARCANE_CIRCLE_SCROLL_CTOR = "Server.Items.ArcaneCircleScroll", 600, 0x2D51, 0x8FD, [];
    GIFT_OF_RENEWAL_SCROLL, GIFT_OF_RENEWAL_SCROLL_CTOR = "Server.Items.GiftOfRenewalScroll", 601, 0x2D52, 0x8FD, [];
    IMMOLATING_WEAPON_SCROLL, IMMOLATING_WEAPON_SCROLL_CTOR = "Server.Items.ImmolatingWeaponScroll", 602, 0x2D53, 0x8FD, [];
    ATTUNE_WEAPON_SCROLL, ATTUNE_WEAPON_SCROLL_CTOR = "Server.Items.AttuneWeaponScroll", 603, 0x2D54, 0x8FD, [];
    THUNDERSTORM_SCROLL, THUNDERSTORM_SCROLL_CTOR = "Server.Items.ThunderstormScroll", 604, 0x2D55, 0x8FD, [];
    NATURE_FURY_SCROLL, NATURE_FURY_SCROLL_CTOR = "Server.Items.NatureFuryScroll", 605, 0x2D56, 0x8FD, [];
    SUMMON_FEY_SCROLL, SUMMON_FEY_SCROLL_CTOR = "Server.Items.SummonFeyScroll", 606, 0x2D57, 0x8FD, [];
    SUMMON_FIEND_SCROLL, SUMMON_FIEND_SCROLL_CTOR = "Server.Items.SummonFiendScroll", 607, 0x2D58, 0x8FD, [];
    REAPER_FORM_SCROLL, REAPER_FORM_SCROLL_CTOR = "Server.Items.ReaperFormScroll", 608, 0x2D59, 0x8FD, [];
    WILDFIRE_SCROLL, WILDFIRE_SCROLL_CTOR = "Server.Items.WildfireScroll", 609, 0x2D5A, 0x8FD, [];
    ESSENCE_OF_WIND_SCROLL, ESSENCE_OF_WIND_SCROLL_CTOR = "Server.Items.EssenceOfWindScroll", 610, 0x2D5B, 0x8FD, [];
    DRYAD_ALLURE_SCROLL, DRYAD_ALLURE_SCROLL_CTOR = "Server.Items.DryadAllureScroll", 611, 0x2D5C, 0x8FD, [];
    ETHEREAL_VOYAGE_SCROLL, ETHEREAL_VOYAGE_SCROLL_CTOR = "Server.Items.EtherealVoyageScroll", 612, 0x2D5D, 0x8FD, [];
    WORD_OF_DEATH_SCROLL, WORD_OF_DEATH_SCROLL_CTOR = "Server.Items.WordOfDeathScroll", 613, 0x2D5E, 0x8FD, [];
    GIFT_OF_LIFE_SCROLL, GIFT_OF_LIFE_SCROLL_CTOR = "Server.Items.GiftOfLifeScroll", 614, 0x2D5F, 0x8FD, [];
    ARCANE_EMPOWERMENT_SCROLL, ARCANE_EMPOWERMENT_SCROLL_CTOR = "Server.Items.ArcaneEmpowermentScroll", 615, 0x2D60, 0x8FD, [];
    EARTHQUAKE_SCROLL, EARTHQUAKE_SCROLL_CTOR = "Server.Items.EarthquakeScroll", 56, 0x1F65, 0, [];
    ENERGY_VORTEX_SCROLL, ENERGY_VORTEX_SCROLL_CTOR = "Server.Items.EnergyVortexScroll", 57, 0x1F66, 0, [];
    RESURRECTION_SCROLL, RESURRECTION_SCROLL_CTOR = "Server.Items.ResurrectionScroll", 58, 0x1F67, 0, [];
    SUMMON_AIR_ELEMENTAL_SCROLL, SUMMON_AIR_ELEMENTAL_SCROLL_CTOR = "Server.Items.SummonAirElementalScroll", 59, 0x1F68, 0, [];
    SUMMON_DAEMON_SCROLL, SUMMON_DAEMON_SCROLL_CTOR = "Server.Items.SummonDaemonScroll", 60, 0x1F69, 0, [];
    SUMMON_EARTH_ELEMENTAL_SCROLL, SUMMON_EARTH_ELEMENTAL_SCROLL_CTOR = "Server.Items.SummonEarthElementalScroll", 61, 0x1F6A, 0, [];
    SUMMON_FIRE_ELEMENTAL_SCROLL, SUMMON_FIRE_ELEMENTAL_SCROLL_CTOR = "Server.Items.SummonFireElementalScroll", 62, 0x1F6B, 0, [];
    SUMMON_WATER_ELEMENTAL_SCROLL, SUMMON_WATER_ELEMENTAL_SCROLL_CTOR = "Server.Items.SummonWaterElementalScroll", 63, 0x1F6C, 0, [];
    BLADE_SPIRITS_SCROLL, BLADE_SPIRITS_SCROLL_CTOR = "Server.Items.BladeSpiritsScroll", 32, 0x1F4D, 0, [];
    DISPEL_FIELD_SCROLL, DISPEL_FIELD_SCROLL_CTOR = "Server.Items.DispelFieldScroll", 33, 0x1F4E, 0, [];
    INCOGNITO_SCROLL, INCOGNITO_SCROLL_CTOR = "Server.Items.IncognitoScroll", 34, 0x1F4F, 0, [];
    MAGIC_REFLECT_SCROLL, MAGIC_REFLECT_SCROLL_CTOR = "Server.Items.MagicReflectScroll", 35, 0x1F50, 0, [];
    MIND_BLAST_SCROLL, MIND_BLAST_SCROLL_CTOR = "Server.Items.MindBlastScroll", 36, 0x1F51, 0, [];
    PARALYZE_SCROLL, PARALYZE_SCROLL_CTOR = "Server.Items.ParalyzeScroll", 37, 0x1F52, 0, [];
    POISON_FIELD_SCROLL, POISON_FIELD_SCROLL_CTOR = "Server.Items.PoisonFieldScroll", 38, 0x1F53, 0, [];
    SUMMON_CREATURE_SCROLL, SUMMON_CREATURE_SCROLL_CTOR = "Server.Items.SummonCreatureScroll", 39, 0x1F54, 0, [];
    CLUMSY_SCROLL, CLUMSY_SCROLL_CTOR = "Server.Items.ClumsyScroll", 0, 0x1F2E, 0, [];
    CREATE_FOOD_SCROLL, CREATE_FOOD_SCROLL_CTOR = "Server.Items.CreateFoodScroll", 1, 0x1F2F, 0, [];
    FEEBLEMIND_SCROLL, FEEBLEMIND_SCROLL_CTOR = "Server.Items.FeeblemindScroll", 2, 0x1F30, 0, [];
    HEAL_SCROLL, HEAL_SCROLL_CTOR = "Server.Items.HealScroll", 3, 0x1F31, 0, [];
    MAGIC_ARROW_SCROLL, MAGIC_ARROW_SCROLL_CTOR = "Server.Items.MagicArrowScroll", 4, 0x1F32, 0, [];
    NIGHT_SIGHT_SCROLL, NIGHT_SIGHT_SCROLL_CTOR = "Server.Items.NightSightScroll", 5, 0x1F33, 0, [];
    REACTIVE_ARMOR_SCROLL, REACTIVE_ARMOR_SCROLL_CTOR = "Server.Items.ReactiveArmorScroll", 6, 0x1F2D, 0, [];
    WEAKEN_SCROLL, WEAKEN_SCROLL_CTOR = "Server.Items.WeakenScroll", 7, 0x1F34, 0, [];
    ARCH_PROTECTION_SCROLL, ARCH_PROTECTION_SCROLL_CTOR = "Server.Items.ArchProtectionScroll", 25, 0x1F46, 0, [];
    ARCH_CURE_SCROLL, ARCH_CURE_SCROLL_CTOR = "Server.Items.ArchCureScroll", 24, 0x1F45, 0, [];
    CURSE_SCROLL, CURSE_SCROLL_CTOR = "Server.Items.CurseScroll", 26, 0x1F47, 0, [];
    FIRE_FIELD_SCROLL, FIRE_FIELD_SCROLL_CTOR = "Server.Items.FireFieldScroll", 27, 0x1F48, 0, [];
    GREATER_HEAL_SCROLL, GREATER_HEAL_SCROLL_CTOR = "Server.Items.GreaterHealScroll", 28, 0x1F49, 0, [];
    LIGHTNING_SCROLL, LIGHTNING_SCROLL_CTOR = "Server.Items.LightningScroll", 29, 0x1F4A, 0, [];
    MANA_DRAIN_SCROLL, MANA_DRAIN_SCROLL_CTOR = "Server.Items.ManaDrainScroll", 30, 0x1F4B, 0, [];
    RECALL_SCROLL, RECALL_SCROLL_CTOR = "Server.Items.RecallScroll", 31, 0x1F4C, 0, [];
    ANIMATED_WEAPON_SCROLL, ANIMATED_WEAPON_SCROLL_CTOR = "Server.Items.AnimatedWeaponScroll", 683, 0x2DA4, 0, [];
    BOMBARD_SCROLL, BOMBARD_SCROLL_CTOR = "Server.Items.BombardScroll", 688, 0x2DA9, 0, [];
    CLEANSING_WINDS_SCROLL, CLEANSING_WINDS_SCROLL_CTOR = "Server.Items.CleansingWindsScroll", 687, 0x2DA8, 0, [];
    EAGLE_STRIKE_SCROLL, EAGLE_STRIKE_SCROLL_CTOR = "Server.Items.EagleStrikeScroll", 682, 0x2DA3, 0, [];
    ENCHANT_SCROLL, ENCHANT_SCROLL_CTOR = "Server.Items.EnchantScroll", 680, 0x2DA1, 0, [];
    HAIL_STORM_SCROLL, HAIL_STORM_SCROLL_CTOR = "Server.Items.HailStormScroll", 690, 0x2DAB, 0, [];
    HEALING_STONE_SCROLL, HEALING_STONE_SCROLL_CTOR = "Server.Items.HealingStoneScroll", 678, 0x2D9F, 0, [];
    MASS_SLEEP_SCROLL, MASS_SLEEP_SCROLL_CTOR = "Server.Items.MassSleepScroll", 686, 0x2DA7, 0, [];
    NETHER_BOLT_SCROLL, NETHER_BOLT_SCROLL_CTOR = "Server.Items.NetherBoltScroll", 677, 0x2D9E, 0, [];
    NETHER_CYCLONE_SCROLL, NETHER_CYCLONE_SCROLL_CTOR = "Server.Items.NetherCycloneScroll", 691, 0x2DAC, 0, [];
    PURGE_MAGIC_SCROLL, PURGE_MAGIC_SCROLL_CTOR = "Server.Items.PurgeMagicScroll", 679, 0x2DA0, 0, [];
    RISING_COLOSSUS_SCROLL, RISING_COLOSSUS_SCROLL_CTOR = "Server.Items.RisingColossusScroll", 692, 0x2DAD, 0, [];
    SLEEP_SCROLL, SLEEP_SCROLL_CTOR = "Server.Items.SleepScroll", 681, 0x2DA2, 0, [];
    SPELL_PLAGUE_SCROLL, SPELL_PLAGUE_SCROLL_CTOR = "Server.Items.SpellPlagueScroll", 689, 0x2DAA, 0, [];
    SPELL_TRIGGER_SCROLL, SPELL_TRIGGER_SCROLL_CTOR = "Server.Items.SpellTriggerScroll", 685, 0x2DA6, 0, [];
    STONE_FORM_SCROLL, STONE_FORM_SCROLL_CTOR = "Server.Items.StoneFormScroll", 684, 0x2DA5, 0, [];
    ANIMATE_DEAD_SCROLL, ANIMATE_DEAD_SCROLL_CTOR = "Server.Items.AnimateDeadScroll", 100, 0x2260, 0, [];
    BLOOD_OATH_SCROLL, BLOOD_OATH_SCROLL_CTOR = "Server.Items.BloodOathScroll", 101, 0x2261, 0, [];
    CORPSE_SKIN_SCROLL, CORPSE_SKIN_SCROLL_CTOR = "Server.Items.CorpseSkinScroll", 102, 0x2262, 0, [];
    CURSE_WEAPON_SCROLL, CURSE_WEAPON_SCROLL_CTOR = "Server.Items.CurseWeaponScroll", 103, 0x2263, 0, [];
    EVIL_OMEN_SCROLL, EVIL_OMEN_SCROLL_CTOR = "Server.Items.EvilOmenScroll", 104, 0x2264, 0, [];
    EXORCISM_SCROLL, EXORCISM_SCROLL_CTOR = "Server.Items.ExorcismScroll", 116, 0x2270, 0, [];
    HORRIFIC_BEAST_SCROLL, HORRIFIC_BEAST_SCROLL_CTOR = "Server.Items.HorrificBeastScroll", 105, 0x2265, 0, [];
    LICH_FORM_SCROLL, LICH_FORM_SCROLL_CTOR = "Server.Items.LichFormScroll", 106, 0x2266, 0, [];
    MIND_ROT_SCROLL, MIND_ROT_SCROLL_CTOR = "Server.Items.MindRotScroll", 107, 0x2267, 0, [];
    PAIN_SPIKE_SCROLL, PAIN_SPIKE_SCROLL_CTOR = "Server.Items.PainSpikeScroll", 108, 0x2268, 0, [];
    POISON_STRIKE_SCROLL, POISON_STRIKE_SCROLL_CTOR = "Server.Items.PoisonStrikeScroll", 109, 0x2269, 0, [];
    STRANGLE_SCROLL, STRANGLE_SCROLL_CTOR = "Server.Items.StrangleScroll", 110, 0x226A, 0, [];
    SUMMON_FAMILIAR_SCROLL, SUMMON_FAMILIAR_SCROLL_CTOR = "Server.Items.SummonFamiliarScroll", 111, 0x226B, 0, [];
    VAMPIRIC_EMBRACE_SCROLL, VAMPIRIC_EMBRACE_SCROLL_CTOR = "Server.Items.VampiricEmbraceScroll", 112, 0x226C, 0, [];
    VENGEFUL_SPIRIT_SCROLL, VENGEFUL_SPIRIT_SCROLL_CTOR = "Server.Items.VengefulSpiritScroll", 113, 0x226D, 0, [];
    WITHER_SCROLL, WITHER_SCROLL_CTOR = "Server.Items.WitherScroll", 114, 0x226E, 0, [];
    WRAITH_FORM_SCROLL, WRAITH_FORM_SCROLL_CTOR = "Server.Items.WraithFormScroll", 115, 0x226F, 0, [];
    AGILITY_SCROLL, AGILITY_SCROLL_CTOR = "Server.Items.AgilityScroll", 8, 0x1F35, 0, [];
    CUNNING_SCROLL, CUNNING_SCROLL_CTOR = "Server.Items.CunningScroll", 9, 0x1F36, 0, [];
    CURE_SCROLL, CURE_SCROLL_CTOR = "Server.Items.CureScroll", 10, 0x1F37, 0, [];
    HARM_SCROLL, HARM_SCROLL_CTOR = "Server.Items.HarmScroll", 11, 0x1F38, 0, [];
    MAGIC_TRAP_SCROLL, MAGIC_TRAP_SCROLL_CTOR = "Server.Items.MagicTrapScroll", 12, 0x1F39, 0, [];
    MAGIC_UN_TRAP_SCROLL, MAGIC_UN_TRAP_SCROLL_CTOR = "Server.Items.MagicUnTrapScroll", 13, 0x1F3A, 0, [];
    PROTECTION_SCROLL, PROTECTION_SCROLL_CTOR = "Server.Items.ProtectionScroll", 14, 0x1F3B, 0, [];
    STRENGTH_SCROLL, STRENGTH_SCROLL_CTOR = "Server.Items.StrengthScroll", 15, 0x1F3C, 0, [];
    CHAIN_LIGHTNING_SCROLL, CHAIN_LIGHTNING_SCROLL_CTOR = "Server.Items.ChainLightningScroll", 48, 0x1F5D, 0, [];
    ENERGY_FIELD_SCROLL, ENERGY_FIELD_SCROLL_CTOR = "Server.Items.EnergyFieldScroll", 49, 0x1F5E, 0, [];
    FLAMESTRIKE_SCROLL, FLAMESTRIKE_SCROLL_CTOR = "Server.Items.FlamestrikeScroll", 50, 0x1F5F, 0, [];
    GATE_TRAVEL_SCROLL, GATE_TRAVEL_SCROLL_CTOR = "Server.Items.GateTravelScroll", 51, 0x1F60, 0, [];
    MANA_VAMPIRE_SCROLL, MANA_VAMPIRE_SCROLL_CTOR = "Server.Items.ManaVampireScroll", 52, 0x1F61, 0, [];
    MASS_DISPEL_SCROLL, MASS_DISPEL_SCROLL_CTOR = "Server.Items.MassDispelScroll", 53, 0x1F62, 0, [];
    METEOR_SWARM_SCROLL, METEOR_SWARM_SCROLL_CTOR = "Server.Items.MeteorSwarmScroll", 54, 0x1F63, 0, [];
    POLYMORPH_SCROLL, POLYMORPH_SCROLL_CTOR = "Server.Items.PolymorphScroll", 55, 0x1F64, 0, [];
    DISPEL_SCROLL, DISPEL_SCROLL_CTOR = "Server.Items.DispelScroll", 40, 0x1F55, 0, [];
    ENERGY_BOLT_SCROLL, ENERGY_BOLT_SCROLL_CTOR = "Server.Items.EnergyBoltScroll", 41, 0x1F56, 0, [];
    EXPLOSION_SCROLL, EXPLOSION_SCROLL_CTOR = "Server.Items.ExplosionScroll", 42, 0x1F57, 0, [];
    INVISIBILITY_SCROLL, INVISIBILITY_SCROLL_CTOR = "Server.Items.InvisibilityScroll", 43, 0x1F58, 0, [];
    MARK_SCROLL, MARK_SCROLL_CTOR = "Server.Items.MarkScroll", 44, 0x1F59, 0, [];
    MASS_CURSE_SCROLL, MASS_CURSE_SCROLL_CTOR = "Server.Items.MassCurseScroll", 45, 0x1F5A, 0, [];
    PARALYZE_FIELD_SCROLL, PARALYZE_FIELD_SCROLL_CTOR = "Server.Items.ParalyzeFieldScroll", 46, 0x1F5B, 0, [];
    REVEAL_SCROLL, REVEAL_SCROLL_CTOR = "Server.Items.RevealScroll", 47, 0x1F5C, 0, [];
    BLESS_SCROLL, BLESS_SCROLL_CTOR = "Server.Items.BlessScroll", 16, 0x1F3D, 0, [];
    FIREBALL_SCROLL, FIREBALL_SCROLL_CTOR = "Server.Items.FireballScroll", 17, 0x1F3E, 0, [];
    MAGIC_LOCK_SCROLL, MAGIC_LOCK_SCROLL_CTOR = "Server.Items.MagicLockScroll", 18, 0x1F3F, 0, [];
    POISON_SCROLL, POISON_SCROLL_CTOR = "Server.Items.PoisonScroll", 19, 0x1F40, 0, [];
    TELEKINESIS_SCROLL, TELEKINESIS_SCROLL_CTOR = "Server.Items.TelekinesisScroll", 20, 0x1F41, 0, ["Server.Items.TelekinisisScroll"];
    TELEPORT_SCROLL, TELEPORT_SCROLL_CTOR = "Server.Items.TeleportScroll", 21, 0x1F42, 0, [];
    UNLOCK_SCROLL, UNLOCK_SCROLL_CTOR = "Server.Items.UnlockScroll", 22, 0x1F43, 0, [];
    WALL_OF_STONE_SCROLL, WALL_OF_STONE_SCROLL_CTOR = "Server.Items.WallOfStoneScroll", 23, 0x1F44, 0, [];
}

pub static ALL: &[(&muo_server::item::ItemType, &[&str])] = &[
    (&ARCANE_CIRCLE_SCROLL, &[]),
    (&GIFT_OF_RENEWAL_SCROLL, &[]),
    (&IMMOLATING_WEAPON_SCROLL, &[]),
    (&ATTUNE_WEAPON_SCROLL, &[]),
    (&THUNDERSTORM_SCROLL, &[]),
    (&NATURE_FURY_SCROLL, &[]),
    (&SUMMON_FEY_SCROLL, &[]),
    (&SUMMON_FIEND_SCROLL, &[]),
    (&REAPER_FORM_SCROLL, &[]),
    (&WILDFIRE_SCROLL, &[]),
    (&ESSENCE_OF_WIND_SCROLL, &[]),
    (&DRYAD_ALLURE_SCROLL, &[]),
    (&ETHEREAL_VOYAGE_SCROLL, &[]),
    (&WORD_OF_DEATH_SCROLL, &[]),
    (&GIFT_OF_LIFE_SCROLL, &[]),
    (&ARCANE_EMPOWERMENT_SCROLL, &[]),
    (&EARTHQUAKE_SCROLL, &[]),
    (&ENERGY_VORTEX_SCROLL, &[]),
    (&RESURRECTION_SCROLL, &[]),
    (&SUMMON_AIR_ELEMENTAL_SCROLL, &[]),
    (&SUMMON_DAEMON_SCROLL, &[]),
    (&SUMMON_EARTH_ELEMENTAL_SCROLL, &[]),
    (&SUMMON_FIRE_ELEMENTAL_SCROLL, &[]),
    (&SUMMON_WATER_ELEMENTAL_SCROLL, &[]),
    (&BLADE_SPIRITS_SCROLL, &[]),
    (&DISPEL_FIELD_SCROLL, &[]),
    (&INCOGNITO_SCROLL, &[]),
    (&MAGIC_REFLECT_SCROLL, &[]),
    (&MIND_BLAST_SCROLL, &[]),
    (&PARALYZE_SCROLL, &[]),
    (&POISON_FIELD_SCROLL, &[]),
    (&SUMMON_CREATURE_SCROLL, &[]),
    (&CLUMSY_SCROLL, &[]),
    (&CREATE_FOOD_SCROLL, &[]),
    (&FEEBLEMIND_SCROLL, &[]),
    (&HEAL_SCROLL, &[]),
    (&MAGIC_ARROW_SCROLL, &[]),
    (&NIGHT_SIGHT_SCROLL, &[]),
    (&REACTIVE_ARMOR_SCROLL, &[]),
    (&WEAKEN_SCROLL, &[]),
    (&ARCH_PROTECTION_SCROLL, &[]),
    (&ARCH_CURE_SCROLL, &[]),
    (&CURSE_SCROLL, &[]),
    (&FIRE_FIELD_SCROLL, &[]),
    (&GREATER_HEAL_SCROLL, &[]),
    (&LIGHTNING_SCROLL, &[]),
    (&MANA_DRAIN_SCROLL, &[]),
    (&RECALL_SCROLL, &[]),
    (&ANIMATED_WEAPON_SCROLL, &[]),
    (&BOMBARD_SCROLL, &[]),
    (&CLEANSING_WINDS_SCROLL, &[]),
    (&EAGLE_STRIKE_SCROLL, &[]),
    (&ENCHANT_SCROLL, &[]),
    (&HAIL_STORM_SCROLL, &[]),
    (&HEALING_STONE_SCROLL, &[]),
    (&MASS_SLEEP_SCROLL, &[]),
    (&NETHER_BOLT_SCROLL, &[]),
    (&NETHER_CYCLONE_SCROLL, &[]),
    (&PURGE_MAGIC_SCROLL, &[]),
    (&RISING_COLOSSUS_SCROLL, &[]),
    (&SLEEP_SCROLL, &[]),
    (&SPELL_PLAGUE_SCROLL, &[]),
    (&SPELL_TRIGGER_SCROLL, &[]),
    (&STONE_FORM_SCROLL, &[]),
    (&ANIMATE_DEAD_SCROLL, &[]),
    (&BLOOD_OATH_SCROLL, &[]),
    (&CORPSE_SKIN_SCROLL, &[]),
    (&CURSE_WEAPON_SCROLL, &[]),
    (&EVIL_OMEN_SCROLL, &[]),
    (&EXORCISM_SCROLL, &[]),
    (&HORRIFIC_BEAST_SCROLL, &[]),
    (&LICH_FORM_SCROLL, &[]),
    (&MIND_ROT_SCROLL, &[]),
    (&PAIN_SPIKE_SCROLL, &[]),
    (&POISON_STRIKE_SCROLL, &[]),
    (&STRANGLE_SCROLL, &[]),
    (&SUMMON_FAMILIAR_SCROLL, &[]),
    (&VAMPIRIC_EMBRACE_SCROLL, &[]),
    (&VENGEFUL_SPIRIT_SCROLL, &[]),
    (&WITHER_SCROLL, &[]),
    (&WRAITH_FORM_SCROLL, &[]),
    (&AGILITY_SCROLL, &[]),
    (&CUNNING_SCROLL, &[]),
    (&CURE_SCROLL, &[]),
    (&HARM_SCROLL, &[]),
    (&MAGIC_TRAP_SCROLL, &[]),
    (&MAGIC_UN_TRAP_SCROLL, &[]),
    (&PROTECTION_SCROLL, &[]),
    (&STRENGTH_SCROLL, &[]),
    (&CHAIN_LIGHTNING_SCROLL, &[]),
    (&ENERGY_FIELD_SCROLL, &[]),
    (&FLAMESTRIKE_SCROLL, &[]),
    (&GATE_TRAVEL_SCROLL, &[]),
    (&MANA_VAMPIRE_SCROLL, &[]),
    (&MASS_DISPEL_SCROLL, &[]),
    (&METEOR_SWARM_SCROLL, &[]),
    (&POLYMORPH_SCROLL, &[]),
    (&DISPEL_SCROLL, &[]),
    (&ENERGY_BOLT_SCROLL, &[]),
    (&EXPLOSION_SCROLL, &[]),
    (&INVISIBILITY_SCROLL, &[]),
    (&MARK_SCROLL, &[]),
    (&MASS_CURSE_SCROLL, &[]),
    (&PARALYZE_FIELD_SCROLL, &[]),
    (&REVEAL_SCROLL, &[]),
    (&BLESS_SCROLL, &[]),
    (&FIREBALL_SCROLL, &[]),
    (&MAGIC_LOCK_SCROLL, &[]),
    (&POISON_SCROLL, &[]),
    (&TELEKINESIS_SCROLL, &["Server.Items.TelekinisisScroll"]),
    (&TELEPORT_SCROLL, &[]),
    (&UNLOCK_SCROLL, &[]),
    (&WALL_OF_STONE_SCROLL, &[]),
];

/// The constructor tables, by full name.
pub static CTORS: &[(&str, &[muo_content::props::Ctor])] = &[
    ("Server.Items.ArcaneCircleScroll", &ARCANE_CIRCLE_SCROLL_CTOR),
    ("Server.Items.GiftOfRenewalScroll", &GIFT_OF_RENEWAL_SCROLL_CTOR),
    ("Server.Items.ImmolatingWeaponScroll", &IMMOLATING_WEAPON_SCROLL_CTOR),
    ("Server.Items.AttuneWeaponScroll", &ATTUNE_WEAPON_SCROLL_CTOR),
    ("Server.Items.ThunderstormScroll", &THUNDERSTORM_SCROLL_CTOR),
    ("Server.Items.NatureFuryScroll", &NATURE_FURY_SCROLL_CTOR),
    ("Server.Items.SummonFeyScroll", &SUMMON_FEY_SCROLL_CTOR),
    ("Server.Items.SummonFiendScroll", &SUMMON_FIEND_SCROLL_CTOR),
    ("Server.Items.ReaperFormScroll", &REAPER_FORM_SCROLL_CTOR),
    ("Server.Items.WildfireScroll", &WILDFIRE_SCROLL_CTOR),
    ("Server.Items.EssenceOfWindScroll", &ESSENCE_OF_WIND_SCROLL_CTOR),
    ("Server.Items.DryadAllureScroll", &DRYAD_ALLURE_SCROLL_CTOR),
    ("Server.Items.EtherealVoyageScroll", &ETHEREAL_VOYAGE_SCROLL_CTOR),
    ("Server.Items.WordOfDeathScroll", &WORD_OF_DEATH_SCROLL_CTOR),
    ("Server.Items.GiftOfLifeScroll", &GIFT_OF_LIFE_SCROLL_CTOR),
    ("Server.Items.ArcaneEmpowermentScroll", &ARCANE_EMPOWERMENT_SCROLL_CTOR),
    ("Server.Items.EarthquakeScroll", &EARTHQUAKE_SCROLL_CTOR),
    ("Server.Items.EnergyVortexScroll", &ENERGY_VORTEX_SCROLL_CTOR),
    ("Server.Items.ResurrectionScroll", &RESURRECTION_SCROLL_CTOR),
    ("Server.Items.SummonAirElementalScroll", &SUMMON_AIR_ELEMENTAL_SCROLL_CTOR),
    ("Server.Items.SummonDaemonScroll", &SUMMON_DAEMON_SCROLL_CTOR),
    ("Server.Items.SummonEarthElementalScroll", &SUMMON_EARTH_ELEMENTAL_SCROLL_CTOR),
    ("Server.Items.SummonFireElementalScroll", &SUMMON_FIRE_ELEMENTAL_SCROLL_CTOR),
    ("Server.Items.SummonWaterElementalScroll", &SUMMON_WATER_ELEMENTAL_SCROLL_CTOR),
    ("Server.Items.BladeSpiritsScroll", &BLADE_SPIRITS_SCROLL_CTOR),
    ("Server.Items.DispelFieldScroll", &DISPEL_FIELD_SCROLL_CTOR),
    ("Server.Items.IncognitoScroll", &INCOGNITO_SCROLL_CTOR),
    ("Server.Items.MagicReflectScroll", &MAGIC_REFLECT_SCROLL_CTOR),
    ("Server.Items.MindBlastScroll", &MIND_BLAST_SCROLL_CTOR),
    ("Server.Items.ParalyzeScroll", &PARALYZE_SCROLL_CTOR),
    ("Server.Items.PoisonFieldScroll", &POISON_FIELD_SCROLL_CTOR),
    ("Server.Items.SummonCreatureScroll", &SUMMON_CREATURE_SCROLL_CTOR),
    ("Server.Items.ClumsyScroll", &CLUMSY_SCROLL_CTOR),
    ("Server.Items.CreateFoodScroll", &CREATE_FOOD_SCROLL_CTOR),
    ("Server.Items.FeeblemindScroll", &FEEBLEMIND_SCROLL_CTOR),
    ("Server.Items.HealScroll", &HEAL_SCROLL_CTOR),
    ("Server.Items.MagicArrowScroll", &MAGIC_ARROW_SCROLL_CTOR),
    ("Server.Items.NightSightScroll", &NIGHT_SIGHT_SCROLL_CTOR),
    ("Server.Items.ReactiveArmorScroll", &REACTIVE_ARMOR_SCROLL_CTOR),
    ("Server.Items.WeakenScroll", &WEAKEN_SCROLL_CTOR),
    ("Server.Items.ArchProtectionScroll", &ARCH_PROTECTION_SCROLL_CTOR),
    ("Server.Items.ArchCureScroll", &ARCH_CURE_SCROLL_CTOR),
    ("Server.Items.CurseScroll", &CURSE_SCROLL_CTOR),
    ("Server.Items.FireFieldScroll", &FIRE_FIELD_SCROLL_CTOR),
    ("Server.Items.GreaterHealScroll", &GREATER_HEAL_SCROLL_CTOR),
    ("Server.Items.LightningScroll", &LIGHTNING_SCROLL_CTOR),
    ("Server.Items.ManaDrainScroll", &MANA_DRAIN_SCROLL_CTOR),
    ("Server.Items.RecallScroll", &RECALL_SCROLL_CTOR),
    ("Server.Items.AnimatedWeaponScroll", &ANIMATED_WEAPON_SCROLL_CTOR),
    ("Server.Items.BombardScroll", &BOMBARD_SCROLL_CTOR),
    ("Server.Items.CleansingWindsScroll", &CLEANSING_WINDS_SCROLL_CTOR),
    ("Server.Items.EagleStrikeScroll", &EAGLE_STRIKE_SCROLL_CTOR),
    ("Server.Items.EnchantScroll", &ENCHANT_SCROLL_CTOR),
    ("Server.Items.HailStormScroll", &HAIL_STORM_SCROLL_CTOR),
    ("Server.Items.HealingStoneScroll", &HEALING_STONE_SCROLL_CTOR),
    ("Server.Items.MassSleepScroll", &MASS_SLEEP_SCROLL_CTOR),
    ("Server.Items.NetherBoltScroll", &NETHER_BOLT_SCROLL_CTOR),
    ("Server.Items.NetherCycloneScroll", &NETHER_CYCLONE_SCROLL_CTOR),
    ("Server.Items.PurgeMagicScroll", &PURGE_MAGIC_SCROLL_CTOR),
    ("Server.Items.RisingColossusScroll", &RISING_COLOSSUS_SCROLL_CTOR),
    ("Server.Items.SleepScroll", &SLEEP_SCROLL_CTOR),
    ("Server.Items.SpellPlagueScroll", &SPELL_PLAGUE_SCROLL_CTOR),
    ("Server.Items.SpellTriggerScroll", &SPELL_TRIGGER_SCROLL_CTOR),
    ("Server.Items.StoneFormScroll", &STONE_FORM_SCROLL_CTOR),
    ("Server.Items.AnimateDeadScroll", &ANIMATE_DEAD_SCROLL_CTOR),
    ("Server.Items.BloodOathScroll", &BLOOD_OATH_SCROLL_CTOR),
    ("Server.Items.CorpseSkinScroll", &CORPSE_SKIN_SCROLL_CTOR),
    ("Server.Items.CurseWeaponScroll", &CURSE_WEAPON_SCROLL_CTOR),
    ("Server.Items.EvilOmenScroll", &EVIL_OMEN_SCROLL_CTOR),
    ("Server.Items.ExorcismScroll", &EXORCISM_SCROLL_CTOR),
    ("Server.Items.HorrificBeastScroll", &HORRIFIC_BEAST_SCROLL_CTOR),
    ("Server.Items.LichFormScroll", &LICH_FORM_SCROLL_CTOR),
    ("Server.Items.MindRotScroll", &MIND_ROT_SCROLL_CTOR),
    ("Server.Items.PainSpikeScroll", &PAIN_SPIKE_SCROLL_CTOR),
    ("Server.Items.PoisonStrikeScroll", &POISON_STRIKE_SCROLL_CTOR),
    ("Server.Items.StrangleScroll", &STRANGLE_SCROLL_CTOR),
    ("Server.Items.SummonFamiliarScroll", &SUMMON_FAMILIAR_SCROLL_CTOR),
    ("Server.Items.VampiricEmbraceScroll", &VAMPIRIC_EMBRACE_SCROLL_CTOR),
    ("Server.Items.VengefulSpiritScroll", &VENGEFUL_SPIRIT_SCROLL_CTOR),
    ("Server.Items.WitherScroll", &WITHER_SCROLL_CTOR),
    ("Server.Items.WraithFormScroll", &WRAITH_FORM_SCROLL_CTOR),
    ("Server.Items.AgilityScroll", &AGILITY_SCROLL_CTOR),
    ("Server.Items.CunningScroll", &CUNNING_SCROLL_CTOR),
    ("Server.Items.CureScroll", &CURE_SCROLL_CTOR),
    ("Server.Items.HarmScroll", &HARM_SCROLL_CTOR),
    ("Server.Items.MagicTrapScroll", &MAGIC_TRAP_SCROLL_CTOR),
    ("Server.Items.MagicUnTrapScroll", &MAGIC_UN_TRAP_SCROLL_CTOR),
    ("Server.Items.ProtectionScroll", &PROTECTION_SCROLL_CTOR),
    ("Server.Items.StrengthScroll", &STRENGTH_SCROLL_CTOR),
    ("Server.Items.ChainLightningScroll", &CHAIN_LIGHTNING_SCROLL_CTOR),
    ("Server.Items.EnergyFieldScroll", &ENERGY_FIELD_SCROLL_CTOR),
    ("Server.Items.FlamestrikeScroll", &FLAMESTRIKE_SCROLL_CTOR),
    ("Server.Items.GateTravelScroll", &GATE_TRAVEL_SCROLL_CTOR),
    ("Server.Items.ManaVampireScroll", &MANA_VAMPIRE_SCROLL_CTOR),
    ("Server.Items.MassDispelScroll", &MASS_DISPEL_SCROLL_CTOR),
    ("Server.Items.MeteorSwarmScroll", &METEOR_SWARM_SCROLL_CTOR),
    ("Server.Items.PolymorphScroll", &POLYMORPH_SCROLL_CTOR),
    ("Server.Items.DispelScroll", &DISPEL_SCROLL_CTOR),
    ("Server.Items.EnergyBoltScroll", &ENERGY_BOLT_SCROLL_CTOR),
    ("Server.Items.ExplosionScroll", &EXPLOSION_SCROLL_CTOR),
    ("Server.Items.InvisibilityScroll", &INVISIBILITY_SCROLL_CTOR),
    ("Server.Items.MarkScroll", &MARK_SCROLL_CTOR),
    ("Server.Items.MassCurseScroll", &MASS_CURSE_SCROLL_CTOR),
    ("Server.Items.ParalyzeFieldScroll", &PARALYZE_FIELD_SCROLL_CTOR),
    ("Server.Items.RevealScroll", &REVEAL_SCROLL_CTOR),
    ("Server.Items.BlessScroll", &BLESS_SCROLL_CTOR),
    ("Server.Items.FireballScroll", &FIREBALL_SCROLL_CTOR),
    ("Server.Items.MagicLockScroll", &MAGIC_LOCK_SCROLL_CTOR),
    ("Server.Items.PoisonScroll", &POISON_SCROLL_CTOR),
    ("Server.Items.TelekinesisScroll", &TELEKINESIS_SCROLL_CTOR),
    ("Server.Items.TeleportScroll", &TELEPORT_SCROLL_CTOR),
    ("Server.Items.UnlockScroll", &UNLOCK_SCROLL_CTOR),
    ("Server.Items.WallOfStoneScroll", &WALL_OF_STONE_SCROLL_CTOR),
];
