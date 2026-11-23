//! Every `SpellScroll` subclass.

use muo_content::props::Ctor;
use muo_server::ids::EntityId;
use muo_server::item::ItemType;

use super::scrolls::*;

scrolls! {
    ARCANE_CIRCLE_SCROLL, ARCANE_CIRCLE_SCROLL_CTOR = "Server.Items.ArcaneCircleScroll", [];
    GIFT_OF_RENEWAL_SCROLL, GIFT_OF_RENEWAL_SCROLL_CTOR = "Server.Items.GiftOfRenewalScroll", [];
    IMMOLATING_WEAPON_SCROLL, IMMOLATING_WEAPON_SCROLL_CTOR = "Server.Items.ImmolatingWeaponScroll", [];
    ATTUNE_WEAPON_SCROLL, ATTUNE_WEAPON_SCROLL_CTOR = "Server.Items.AttuneWeaponScroll", [];
    THUNDERSTORM_SCROLL, THUNDERSTORM_SCROLL_CTOR = "Server.Items.ThunderstormScroll", [];
    NATURE_FURY_SCROLL, NATURE_FURY_SCROLL_CTOR = "Server.Items.NatureFuryScroll", [];
    SUMMON_FEY_SCROLL, SUMMON_FEY_SCROLL_CTOR = "Server.Items.SummonFeyScroll", [];
    SUMMON_FIEND_SCROLL, SUMMON_FIEND_SCROLL_CTOR = "Server.Items.SummonFiendScroll", [];
    REAPER_FORM_SCROLL, REAPER_FORM_SCROLL_CTOR = "Server.Items.ReaperFormScroll", [];
    WILDFIRE_SCROLL, WILDFIRE_SCROLL_CTOR = "Server.Items.WildfireScroll", [];
    ESSENCE_OF_WIND_SCROLL, ESSENCE_OF_WIND_SCROLL_CTOR = "Server.Items.EssenceOfWindScroll", [];
    DRYAD_ALLURE_SCROLL, DRYAD_ALLURE_SCROLL_CTOR = "Server.Items.DryadAllureScroll", [];
    ETHEREAL_VOYAGE_SCROLL, ETHEREAL_VOYAGE_SCROLL_CTOR = "Server.Items.EtherealVoyageScroll", [];
    WORD_OF_DEATH_SCROLL, WORD_OF_DEATH_SCROLL_CTOR = "Server.Items.WordOfDeathScroll", [];
    GIFT_OF_LIFE_SCROLL, GIFT_OF_LIFE_SCROLL_CTOR = "Server.Items.GiftOfLifeScroll", [];
    ARCANE_EMPOWERMENT_SCROLL, ARCANE_EMPOWERMENT_SCROLL_CTOR = "Server.Items.ArcaneEmpowermentScroll", [];
    EARTHQUAKE_SCROLL, EARTHQUAKE_SCROLL_CTOR = "Server.Items.EarthquakeScroll", [];
    ENERGY_VORTEX_SCROLL, ENERGY_VORTEX_SCROLL_CTOR = "Server.Items.EnergyVortexScroll", [];
    RESURRECTION_SCROLL, RESURRECTION_SCROLL_CTOR = "Server.Items.ResurrectionScroll", [];
    SUMMON_AIR_ELEMENTAL_SCROLL, SUMMON_AIR_ELEMENTAL_SCROLL_CTOR = "Server.Items.SummonAirElementalScroll", [];
    SUMMON_DAEMON_SCROLL, SUMMON_DAEMON_SCROLL_CTOR = "Server.Items.SummonDaemonScroll", [];
    SUMMON_EARTH_ELEMENTAL_SCROLL, SUMMON_EARTH_ELEMENTAL_SCROLL_CTOR = "Server.Items.SummonEarthElementalScroll", [];
    SUMMON_FIRE_ELEMENTAL_SCROLL, SUMMON_FIRE_ELEMENTAL_SCROLL_CTOR = "Server.Items.SummonFireElementalScroll", [];
    SUMMON_WATER_ELEMENTAL_SCROLL, SUMMON_WATER_ELEMENTAL_SCROLL_CTOR = "Server.Items.SummonWaterElementalScroll", [];
    BLADE_SPIRITS_SCROLL, BLADE_SPIRITS_SCROLL_CTOR = "Server.Items.BladeSpiritsScroll", [];
    DISPEL_FIELD_SCROLL, DISPEL_FIELD_SCROLL_CTOR = "Server.Items.DispelFieldScroll", [];
    INCOGNITO_SCROLL, INCOGNITO_SCROLL_CTOR = "Server.Items.IncognitoScroll", [];
    MAGIC_REFLECT_SCROLL, MAGIC_REFLECT_SCROLL_CTOR = "Server.Items.MagicReflectScroll", [];
    MIND_BLAST_SCROLL, MIND_BLAST_SCROLL_CTOR = "Server.Items.MindBlastScroll", [];
    PARALYZE_SCROLL, PARALYZE_SCROLL_CTOR = "Server.Items.ParalyzeScroll", [];
    POISON_FIELD_SCROLL, POISON_FIELD_SCROLL_CTOR = "Server.Items.PoisonFieldScroll", [];
    SUMMON_CREATURE_SCROLL, SUMMON_CREATURE_SCROLL_CTOR = "Server.Items.SummonCreatureScroll", [];
    CLUMSY_SCROLL, CLUMSY_SCROLL_CTOR = "Server.Items.ClumsyScroll", [];
    CREATE_FOOD_SCROLL, CREATE_FOOD_SCROLL_CTOR = "Server.Items.CreateFoodScroll", [];
    FEEBLEMIND_SCROLL, FEEBLEMIND_SCROLL_CTOR = "Server.Items.FeeblemindScroll", [];
    HEAL_SCROLL, HEAL_SCROLL_CTOR = "Server.Items.HealScroll", [];
    MAGIC_ARROW_SCROLL, MAGIC_ARROW_SCROLL_CTOR = "Server.Items.MagicArrowScroll", [];
    NIGHT_SIGHT_SCROLL, NIGHT_SIGHT_SCROLL_CTOR = "Server.Items.NightSightScroll", [];
    REACTIVE_ARMOR_SCROLL, REACTIVE_ARMOR_SCROLL_CTOR = "Server.Items.ReactiveArmorScroll", [];
    WEAKEN_SCROLL, WEAKEN_SCROLL_CTOR = "Server.Items.WeakenScroll", [];
    ARCH_PROTECTION_SCROLL, ARCH_PROTECTION_SCROLL_CTOR = "Server.Items.ArchProtectionScroll", [];
    ARCH_CURE_SCROLL, ARCH_CURE_SCROLL_CTOR = "Server.Items.ArchCureScroll", [];
    CURSE_SCROLL, CURSE_SCROLL_CTOR = "Server.Items.CurseScroll", [];
    FIRE_FIELD_SCROLL, FIRE_FIELD_SCROLL_CTOR = "Server.Items.FireFieldScroll", [];
    GREATER_HEAL_SCROLL, GREATER_HEAL_SCROLL_CTOR = "Server.Items.GreaterHealScroll", [];
    LIGHTNING_SCROLL, LIGHTNING_SCROLL_CTOR = "Server.Items.LightningScroll", [];
    MANA_DRAIN_SCROLL, MANA_DRAIN_SCROLL_CTOR = "Server.Items.ManaDrainScroll", [];
    RECALL_SCROLL, RECALL_SCROLL_CTOR = "Server.Items.RecallScroll", [];
    ANIMATED_WEAPON_SCROLL, ANIMATED_WEAPON_SCROLL_CTOR = "Server.Items.AnimatedWeaponScroll", [];
    BOMBARD_SCROLL, BOMBARD_SCROLL_CTOR = "Server.Items.BombardScroll", [];
    CLEANSING_WINDS_SCROLL, CLEANSING_WINDS_SCROLL_CTOR = "Server.Items.CleansingWindsScroll", [];
    EAGLE_STRIKE_SCROLL, EAGLE_STRIKE_SCROLL_CTOR = "Server.Items.EagleStrikeScroll", [];
    ENCHANT_SCROLL, ENCHANT_SCROLL_CTOR = "Server.Items.EnchantScroll", [];
    HAIL_STORM_SCROLL, HAIL_STORM_SCROLL_CTOR = "Server.Items.HailStormScroll", [];
    HEALING_STONE_SCROLL, HEALING_STONE_SCROLL_CTOR = "Server.Items.HealingStoneScroll", [];
    MASS_SLEEP_SCROLL, MASS_SLEEP_SCROLL_CTOR = "Server.Items.MassSleepScroll", [];
    NETHER_BOLT_SCROLL, NETHER_BOLT_SCROLL_CTOR = "Server.Items.NetherBoltScroll", [];
    NETHER_CYCLONE_SCROLL, NETHER_CYCLONE_SCROLL_CTOR = "Server.Items.NetherCycloneScroll", [];
    PURGE_MAGIC_SCROLL, PURGE_MAGIC_SCROLL_CTOR = "Server.Items.PurgeMagicScroll", [];
    RISING_COLOSSUS_SCROLL, RISING_COLOSSUS_SCROLL_CTOR = "Server.Items.RisingColossusScroll", [];
    SLEEP_SCROLL, SLEEP_SCROLL_CTOR = "Server.Items.SleepScroll", [];
    SPELL_PLAGUE_SCROLL, SPELL_PLAGUE_SCROLL_CTOR = "Server.Items.SpellPlagueScroll", [];
    SPELL_TRIGGER_SCROLL, SPELL_TRIGGER_SCROLL_CTOR = "Server.Items.SpellTriggerScroll", [];
    STONE_FORM_SCROLL, STONE_FORM_SCROLL_CTOR = "Server.Items.StoneFormScroll", [];
    ANIMATE_DEAD_SCROLL, ANIMATE_DEAD_SCROLL_CTOR = "Server.Items.AnimateDeadScroll", [];
    BLOOD_OATH_SCROLL, BLOOD_OATH_SCROLL_CTOR = "Server.Items.BloodOathScroll", [];
    CORPSE_SKIN_SCROLL, CORPSE_SKIN_SCROLL_CTOR = "Server.Items.CorpseSkinScroll", [];
    CURSE_WEAPON_SCROLL, CURSE_WEAPON_SCROLL_CTOR = "Server.Items.CurseWeaponScroll", [];
    EVIL_OMEN_SCROLL, EVIL_OMEN_SCROLL_CTOR = "Server.Items.EvilOmenScroll", [];
    EXORCISM_SCROLL, EXORCISM_SCROLL_CTOR = "Server.Items.ExorcismScroll", [];
    HORRIFIC_BEAST_SCROLL, HORRIFIC_BEAST_SCROLL_CTOR = "Server.Items.HorrificBeastScroll", [];
    LICH_FORM_SCROLL, LICH_FORM_SCROLL_CTOR = "Server.Items.LichFormScroll", [];
    MIND_ROT_SCROLL, MIND_ROT_SCROLL_CTOR = "Server.Items.MindRotScroll", [];
    PAIN_SPIKE_SCROLL, PAIN_SPIKE_SCROLL_CTOR = "Server.Items.PainSpikeScroll", [];
    POISON_STRIKE_SCROLL, POISON_STRIKE_SCROLL_CTOR = "Server.Items.PoisonStrikeScroll", [];
    STRANGLE_SCROLL, STRANGLE_SCROLL_CTOR = "Server.Items.StrangleScroll", [];
    SUMMON_FAMILIAR_SCROLL, SUMMON_FAMILIAR_SCROLL_CTOR = "Server.Items.SummonFamiliarScroll", [];
    VAMPIRIC_EMBRACE_SCROLL, VAMPIRIC_EMBRACE_SCROLL_CTOR = "Server.Items.VampiricEmbraceScroll", [];
    VENGEFUL_SPIRIT_SCROLL, VENGEFUL_SPIRIT_SCROLL_CTOR = "Server.Items.VengefulSpiritScroll", [];
    WITHER_SCROLL, WITHER_SCROLL_CTOR = "Server.Items.WitherScroll", [];
    WRAITH_FORM_SCROLL, WRAITH_FORM_SCROLL_CTOR = "Server.Items.WraithFormScroll", [];
    AGILITY_SCROLL, AGILITY_SCROLL_CTOR = "Server.Items.AgilityScroll", [];
    CUNNING_SCROLL, CUNNING_SCROLL_CTOR = "Server.Items.CunningScroll", [];
    CURE_SCROLL, CURE_SCROLL_CTOR = "Server.Items.CureScroll", [];
    HARM_SCROLL, HARM_SCROLL_CTOR = "Server.Items.HarmScroll", [];
    MAGIC_TRAP_SCROLL, MAGIC_TRAP_SCROLL_CTOR = "Server.Items.MagicTrapScroll", [];
    MAGIC_UN_TRAP_SCROLL, MAGIC_UN_TRAP_SCROLL_CTOR = "Server.Items.MagicUnTrapScroll", [];
    PROTECTION_SCROLL, PROTECTION_SCROLL_CTOR = "Server.Items.ProtectionScroll", [];
    STRENGTH_SCROLL, STRENGTH_SCROLL_CTOR = "Server.Items.StrengthScroll", [];
    CHAIN_LIGHTNING_SCROLL, CHAIN_LIGHTNING_SCROLL_CTOR = "Server.Items.ChainLightningScroll", [];
    ENERGY_FIELD_SCROLL, ENERGY_FIELD_SCROLL_CTOR = "Server.Items.EnergyFieldScroll", [];
    FLAMESTRIKE_SCROLL, FLAMESTRIKE_SCROLL_CTOR = "Server.Items.FlamestrikeScroll", [];
    GATE_TRAVEL_SCROLL, GATE_TRAVEL_SCROLL_CTOR = "Server.Items.GateTravelScroll", [];
    MANA_VAMPIRE_SCROLL, MANA_VAMPIRE_SCROLL_CTOR = "Server.Items.ManaVampireScroll", [];
    MASS_DISPEL_SCROLL, MASS_DISPEL_SCROLL_CTOR = "Server.Items.MassDispelScroll", [];
    METEOR_SWARM_SCROLL, METEOR_SWARM_SCROLL_CTOR = "Server.Items.MeteorSwarmScroll", [];
    POLYMORPH_SCROLL, POLYMORPH_SCROLL_CTOR = "Server.Items.PolymorphScroll", [];
    DISPEL_SCROLL, DISPEL_SCROLL_CTOR = "Server.Items.DispelScroll", [];
    ENERGY_BOLT_SCROLL, ENERGY_BOLT_SCROLL_CTOR = "Server.Items.EnergyBoltScroll", [];
    EXPLOSION_SCROLL, EXPLOSION_SCROLL_CTOR = "Server.Items.ExplosionScroll", [];
    INVISIBILITY_SCROLL, INVISIBILITY_SCROLL_CTOR = "Server.Items.InvisibilityScroll", [];
    MARK_SCROLL, MARK_SCROLL_CTOR = "Server.Items.MarkScroll", [];
    MASS_CURSE_SCROLL, MASS_CURSE_SCROLL_CTOR = "Server.Items.MassCurseScroll", [];
    PARALYZE_FIELD_SCROLL, PARALYZE_FIELD_SCROLL_CTOR = "Server.Items.ParalyzeFieldScroll", [];
    REVEAL_SCROLL, REVEAL_SCROLL_CTOR = "Server.Items.RevealScroll", [];
    BLESS_SCROLL, BLESS_SCROLL_CTOR = "Server.Items.BlessScroll", [];
    FIREBALL_SCROLL, FIREBALL_SCROLL_CTOR = "Server.Items.FireballScroll", [];
    MAGIC_LOCK_SCROLL, MAGIC_LOCK_SCROLL_CTOR = "Server.Items.MagicLockScroll", [];
    POISON_SCROLL, POISON_SCROLL_CTOR = "Server.Items.PoisonScroll", [];
    TELEKINESIS_SCROLL, TELEKINESIS_SCROLL_CTOR = "Server.Items.TelekinesisScroll", ["Server.Items.TelekinisisScroll"];
    TELEPORT_SCROLL, TELEPORT_SCROLL_CTOR = "Server.Items.TeleportScroll", [];
    UNLOCK_SCROLL, UNLOCK_SCROLL_CTOR = "Server.Items.UnlockScroll", [];
    WALL_OF_STONE_SCROLL, WALL_OF_STONE_SCROLL_CTOR = "Server.Items.WallOfStoneScroll", [];
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
