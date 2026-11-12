//! Assistant feature flags.

use muo_data::json_flags;

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct AssistantFeatures: u64 {
        const NONE = 0;
        /// Weather Filter
        const FILTER_WEATHER = 1 << 0;
        /// Light Filter
        const FILTER_LIGHT = 1 << 1;
        /// Smart Last Target
        const SMART_TARGET = 1 << 2;
        /// Range Check Last Target
        const RANGED_TARGET = 1 << 3;
        /// Automatically Open Doors
        const AUTO_OPEN_DOORS = 1 << 4;
        /// Unequip Weapon on spell cast
        const DEQUIP_ON_CAST = 1 << 5;
        /// Un/Re-equip weapon on potion use
        const AUTO_POTION_EQUIP = 1 << 6;
        /// Block heal If poisoned/Macro If Poisoned condition/Heal or Cure self
        const POISONED_CHECKS = 1 << 7;
        /// Disallow Looping macros, For loops, and macros that call other macros
        const LOOPED_MACROS = 1 << 8;
        /// The use once agent
        const USE_ONCE_AGENT = 1 << 9;
        /// The restock agent
        const RESTOCK_AGENT = 1 << 10;
        /// The sell agent
        const SELL_AGENT = 1 << 11;
        /// The buy agent
        const BUY_AGENT = 1 << 12;
        /// All potion hotkeys
        const POTION_HOTKEYS = 1 << 13;
        /// All random target hotkeys (not target next, last target, target self)
        const RANDOM_TARGETS = 1 << 14;
        /// All closest target hotkeys
        const CLOSEST_TARGETS = 1 << 15;
        /// Health and Mana/Stam messages shown over player's heads
        const OVERHEAD_HEALTH = 1 << 16;
        // AssistUO Only
        /// The autoloot agent
        const AUTOLOOT_AGENT = 1 << 17;
        /// The bone cutter agent
        const BONE_CUTTER_AGENT = 1 << 18;
        /// Javascript macro engine
        const JSCRIPT_MACROS = 1 << 19;
        /// Auto remount after dismount
        const AUTO_REMOUNT = 1 << 20;
        /// Auto bandage friends, self, last and mount option
        const AUTO_BANDAGE = 1 << 21;
        /// Enemy target share on guild, party or alliance chat
        const ENEMY_TARGET_SHARE = 1 << 22;
        /// Season Filter
        const FILTER_SEASON = 1 << 23;
        /// Spell target share on guild, party or alliance chat
        const SPELL_TARGET_SHARE = 1 << 24;
        /// Every feature possible
        const ALL = !0;
    }
}

json_flags!(
    AssistantFeatures,
    [
        ("None", 0),
        ("FilterWeather", 1u64 << 0),
        ("FilterLight", 1u64 << 1),
        ("SmartTarget", 1u64 << 2),
        ("RangedTarget", 1u64 << 3),
        ("AutoOpenDoors", 1u64 << 4),
        ("DequipOnCast", 1u64 << 5),
        ("AutoPotionEquip", 1u64 << 6),
        ("PoisonedChecks", 1u64 << 7),
        ("LoopedMacros", 1u64 << 8),
        ("UseOnceAgent", 1u64 << 9),
        ("RestockAgent", 1u64 << 10),
        ("SellAgent", 1u64 << 11),
        ("BuyAgent", 1u64 << 12),
        ("PotionHotkeys", 1u64 << 13),
        ("RandomTargets", 1u64 << 14),
        ("ClosestTargets", 1u64 << 15),
        ("OverheadHealth", 1u64 << 16),
        ("AutolootAgent", 1u64 << 17),
        ("BoneCutterAgent", 1u64 << 18),
        ("JScriptMacros", 1u64 << 19),
        ("AutoRemount", 1u64 << 20),
        ("AutoBandage", 1u64 << 21),
        ("EnemyTargetShare", 1u64 << 22),
        ("FilterSeason", 1u64 << 23),
        ("SpellTargetShare", 1u64 << 24),
        ("All", !0u64),
    ]
);
