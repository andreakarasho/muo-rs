//! The `Mobile` data, its enums, and (in submodules) the virtual surface.
//!
//! One `Mobile` holds what the class `Mobile` holds. What a subclass adds (`BaseCreature`, `PlayerMobile`)
//! goes in `ext`. The virtual members are fn pointers in [`MobileType`] (`mobile_type.rs`), defaulting to the
//! base behavior in [`base`].
//!
//! Statics of `Mobile` (`DragEffects`, `ActionDelay`, `MaxPlayerResistance`, `DefaultWeapon`, the delta queue,
//! the speech scratch lists) belong to the `World`, not here.
//!

pub mod base;
pub mod mobile_type;
pub mod part1;
pub mod part2;

use bitflags::bitflags;
use muo_primitives::{Body, DateTime, Direction, Point3D, Serial};

use crate::aggressor_info::AggressorInfo;
use crate::ext::Ext;
use crate::guild::GuildId;
use crate::iaccount::AccountId;
use crate::ids::{ItemId, MobileId};
use crate::interfaces::{PartyId, SpawnerId, SpellId, WeaponRef};
use crate::map::MapId;
use crate::mods::{ResistanceMod, SkillMod, StatMod};
use crate::net_state::NetStateId;
use crate::poison::PoisonId;
use crate::prompt::PromptId;
use crate::property_list::PropertyList;
use crate::race::RaceId;
use crate::region::RegionId;
use crate::skills::Skills;
use crate::target::TargetId;
use crate::timer::TimerHandle;

pub use mobile_type::{MOBILE_BASE, MobileType};

bitflags! {
    /// A stat or several (`Str | Dex`).
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct StatType: u8 {
        const STR = 1;
        const DEX = 2;
        const INT = 4;
        const ALL = 7;
    }
}

/// Which way a stat is allowed to move (`StatLockType`).
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum StatLockType {
    #[default]
    Up = 0,
    Down = 1,
    Locked = 2,
}

bitflags! {
    /// What changed on a mobile since the last `ProcessDelta` (`MobileDelta`).
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct MobileDelta: i32 {
        const NONE = 0x0000_0000;
        const NAME = 0x0000_0001;
        const FLAGS = 0x0000_0002;
        const HITS = 0x0000_0004;
        const MANA = 0x0000_0008;
        const STAM = 0x0000_0010;
        const STAT = 0x0000_0020;
        const NOTO = 0x0000_0040;
        const GOLD = 0x0000_0080;
        const WEIGHT = 0x0000_0100;
        const DIRECTION = 0x0000_0200;
        const HUE = 0x0000_0400;
        const BODY = 0x0000_0800;
        const ARMOR = 0x0000_1000;
        const STAT_CAP = 0x0000_2000;
        const GHOST_UPDATE = 0x0000_4000;
        const FOLLOWERS = 0x0000_8000;
        const PROPERTIES = 0x0001_0000;
        const TITHING_POINTS = 0x0002_0000;
        const RESISTANCES = 0x0004_0000;
        const WEAPON_DAMAGE = 0x0008_0000;
        const HAIR = 0x0010_0000;
        const FACIAL_HAIR = 0x0020_0000;
        const RACE = 0x0040_0000;
        const HEALTHBAR_YELLOW = 0x0080_0000;
        const HEALTHBAR_POISON = 0x0100_0000;

        const ATTRIBUTES = 0x0000_001C;
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum Healthbar {
    #[default]
    Normal = 0,
    Poison = 1,
    Yellow = 2,
}

/// Staff rank. Compare with `<`/`>`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(i32)]
pub enum AccessLevel {
    #[default]
    Player = 0,
    Counselor = 1,
    GameMaster = 2,
    Seer = 3,
    Administrator = 4,
    Developer = 5,
    Owner = 6,
}

impl AccessLevel {
    /// The level for a raw value (any int is accepted; a value without a name is `None`).
    pub const fn from_i32(v: i32) -> Option<AccessLevel> {
        Some(match v {
            0 => AccessLevel::Player,
            1 => AccessLevel::Counselor,
            2 => AccessLevel::GameMaster,
            3 => AccessLevel::Seer,
            4 => AccessLevel::Administrator,
            5 => AccessLevel::Developer,
            6 => AccessLevel::Owner,
            _ => return None,
        })
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum VisibleDamageType {
    #[default]
    None = 0,
    Related = 1,
    Everyone = 2,
    Selective = 3,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum ResistanceType {
    #[default]
    Physical = 0,
    Fire = 1,
    Cold = 2,
    Poison = 3,
    Energy = 4,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum ApplyPoisonResult {
    #[default]
    Poisoned = 0,
    Immune = 1,
    HigherPoisonActive = 2,
    Cured = 3,
}

/// The `context` object `MutateSpeech`/`CheckHearsMutatedSpeech` pass around. The only value is the static
/// `m_GhostMutateContext` sentinel.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum SpeechContext {
    #[default]
    None,
    /// `Mobile.m_GhostMutateContext`.
    Ghost,
}

/// `DamageEntry`. The intrusive links (`Next`, `Previous`, `OnLinkList`) are `Mobile::damage_entries` being a `Vec`.
#[derive(Clone, Debug)]
pub struct DamageEntry {
    pub damager: MobileId,
    pub damage_given: i32,
    pub last_damage: DateTime,
    /// Sub-entries; they never join a mobile's list.
    pub responsible: Vec<DamageEntry>,
}

/// A `Mobile` and everything its subclasses inherit; the subclass is `ty`, what it adds is `ext`.
pub struct Mobile {
    pub serial: Serial,
    /// The mobile's class.
    pub ty: &'static MobileType,
    /// Family state for classes that add fields; `Ext::NONE` otherwise.
    pub ext: Ext,

    /// `Created`.
    pub created: DateTime,
    /// `Deleted`.
    pub deleted: bool,
    /// Sector links (`Next`, `Previous`, `OnLinkList`).
    pub next: Option<MobileId>,
    pub previous: Option<MobileId>,
    pub on_link_list: bool,

    // Identity.
    /// `m_Name`.
    pub name: Option<String>,
    /// `m_NameMod`.
    pub name_mod: Option<String>,
    /// `m_Title`.
    pub title: Option<String>,
    /// `m_GuildTitle`.
    pub guild_title: Option<String>,
    /// `m_Language`.
    pub language: Option<String>,
    /// `Profile`.
    pub profile: Option<String>,
    pub profile_locked: bool,
    /// `m_Player`.
    pub player: bool,
    pub female: bool,
    /// `m_Race`; `None` is no race, which reads as `Race.DefaultRace`.
    pub race: Option<RaceId>,
    pub access_level: AccessLevel,
    /// `Account`.
    pub account: Option<AccountId>,

    // Appearance.
    pub body: Body,
    pub body_mod: Body,
    pub hue: i32,
    /// `m_HueMod`; -1 when unset.
    pub hue_mod: i32,
    /// `m_SolidHueOverride`; -1 when unset.
    pub solid_hue_override: i32,
    pub direction: Direction,
    pub light_level: i32,
    pub hair_item_id: i32,
    pub hair_hue: i32,
    pub hair_serial: Serial,
    pub facial_hair_item_id: i32,
    pub facial_hair_hue: i32,
    pub facial_hair_serial: Serial,
    /// `NameHue`; -1 when unset.
    pub name_hue: i32,
    pub speech_hue: i32,
    pub emote_hue: i32,
    pub whisper_hue: i32,
    pub yell_hue: i32,
    /// `BaseSoundID`.
    pub base_sound_id: i32,

    // Position.
    pub location: Point3D,
    /// `m_Map`; `None` is no map.
    pub map: Option<MapId>,
    /// `m_Region`.
    pub region: RegionId,
    /// `WalkRegion`.
    pub walk_region: Option<RegionId>,
    pub logout_location: Point3D,
    pub logout_map: Option<MapId>,

    // Stats.
    pub str: i32,
    pub dex: i32,
    pub int: i32,
    pub str_lock: StatLockType,
    pub dex_lock: StatLockType,
    pub int_lock: StatLockType,
    pub stat_cap: i32,
    pub hits: i32,
    pub stam: i32,
    pub mana: i32,
    pub hunger: i32,
    pub thirst: i32,
    /// `BAC`.
    pub bac: i32,
    pub fame: i32,
    pub karma: i32,
    pub kills: i32,
    pub tithing_points: i32,
    pub virtual_armor: i32,
    pub virtual_armor_mod: i32,
    pub followers: i32,
    pub followers_max: i32,
    pub total_gold: i32,
    pub total_items: i32,
    pub total_weight: i32,
    /// `Resistances` (`int[]`, empty until computed).
    pub resistances: Vec<i32>,
    pub melee_damage_absorb: i32,
    pub magic_damage_absorb: i32,
    pub allowed_stealth_steps: i32,
    /// `Skills`; `None` until `DefaultMobileInit` or the loader makes it.
    pub skills: Option<Skills>,
    /// `_statMods`.
    pub stat_mods: Vec<StatMod>,
    /// `_resistanceMods`.
    pub resistance_mods: Vec<ResistanceMod>,
    /// `_skillMods`.
    pub skill_mods: Vec<SkillMod>,
    pub last_str_gain: DateTime,
    pub last_int_gain: DateTime,
    pub last_dex_gain: DateTime,

    // Status.
    pub warmode: bool,
    pub hidden: bool,
    pub blessed: bool,
    pub flying: bool,
    pub frozen: bool,
    pub paralyzed: bool,
    pub criminal: bool,
    pub can_hear_ghosts: bool,
    pub display_guild_title: bool,
    pub ignore_mobiles: bool,
    pub yellow_healthbar: bool,
    pub meditating: bool,
    pub can_swim: bool,
    pub cant_walk: bool,
    pub squelched: bool,
    pub pushing: bool,
    pub target_locked: bool,
    pub auto_page_notify: bool,
    pub disarm_ready: bool,
    pub stun_ready: bool,
    /// `NoMoveHS`.
    pub no_move_hs: bool,
    /// `m_InDeltaQueue`.
    pub in_delta_queue: bool,
    pub delta_flags: MobileDelta,
    /// `m_Poison`.
    pub poison: Option<PoisonId>,

    // Relations.
    pub combatant: Option<MobileId>,
    /// `m_ChangingCombatant`.
    pub changing_combatant: i32,
    pub last_killer: Option<MobileId>,
    pub guild_fealty: Option<MobileId>,
    pub guild: Option<GuildId>,
    pub party: Option<PartyId>,
    /// `Corpse`.
    pub corpse: Option<ItemId>,
    pub backpack: Option<ItemId>,
    pub bank_box: Option<ItemId>,
    pub holding: Option<ItemId>,
    pub mount_item: Option<ItemId>,
    /// `m_Weapon`: the cached weapon.
    pub weapon: Option<WeaponRef>,
    /// `Items`: the equipped items.
    pub items: Vec<ItemId>,
    pub aggressors: Vec<AggressorInfo>,
    pub aggressed: Vec<AggressorInfo>,
    /// `_damageEntries`.
    pub damage_entries: Vec<DamageEntry>,
    pub spawner: Option<SpawnerId>,
    pub spell: Option<SpellId>,
    pub target: Option<TargetId>,
    pub prompt: Option<PromptId>,
    pub net_state: Option<NetStateId>,
    /// `m_PropertyList`: the cached property list, `None` until built.
    pub property_list: Option<PropertyList>,
    /// `_actions`: the `BeginAction(key)` locks, keyed by the type name or object a caller used.
    pub actions: Vec<&'static str>,

    // Times (tick counts are `Core.TickCount`, compare by wrapping subtraction).
    pub next_combat_time: i64,
    pub last_move_time: i64,
    pub next_action_time: i64,
    pub next_action_message: i64,
    pub next_skill_time: i64,
    pub next_spell_time: i64,
    pub next_warmode_change: DateTime,
    pub warmode_changes: i32,
    /// `_warmodeSpamValue`.
    pub warmode_spam_value: bool,

    // Timers (default handle = not running).
    pub auto_manifest_timer: TimerHandle,
    pub combat_timer: TimerHandle,
    pub expire_aggr_timer: TimerHandle,
    pub expire_combatant_timer: TimerHandle,
    pub expire_criminal_timer: TimerHandle,
    pub frozen_timer: TimerHandle,
    pub logout_timer: TimerHandle,
    pub para_timer: TimerHandle,
    pub mana_timer: TimerHandle,
    pub hits_timer: TimerHandle,
    pub stam_timer: TimerHandle,
    /// `PoisonTimer`.
    pub poison_timer: TimerHandle,
}

impl Mobile {
    /// `Body` (get): `IsBodyMod ? m_BodyMod : m_Body`. Read this, not the `body` field (`m_Body`).
    pub fn cur_body(&self) -> Body {
        if self.body_mod.body_id() != 0 {
            self.body_mod
        } else {
            self.body
        }
    }

    /// `new Mobile()` (so `DefaultMobileInit`): on `Map.Internal`, a skills table, stat cap 225, 5 followers.
    /// `created` and `next_skill_time` should be `Core.Now`/`Core.TickCount`; this leaves them zero, use
    /// [`Mobile::new_at`] (or `World::new_mobile`) to stamp them.
    pub fn new(serial: Serial, ty: &'static MobileType) -> Mobile {
        Mobile::new_at(serial, ty, DateTime::MIN, 0)
    }

    /// `new Mobile()` at `now` (`Core.Now`) and `tick_count` (`Core.TickCount`).
    pub fn new_at(
        serial: Serial,
        ty: &'static MobileType,
        now: DateTime,
        tick_count: i64,
    ) -> Mobile {
        Mobile {
            created: now,
            stat_cap: 225,
            followers_max: 5,
            skills: Some(Skills::default()),
            map: Some(MapId::INTERNAL),
            auto_page_notify: true,
            ..Mobile::new_for_load(serial, ty, tick_count)
        }
    }

    /// `new Mobile(Serial)`, the constructor the loader uses before `Deserialize`: no `DefaultMobileInit`
    /// (no skills, no map, stat cap 0). `Created` should be `Core.Now`; stamp it with `created = now`.
    pub fn new_for_load(serial: Serial, ty: &'static MobileType, tick_count: i64) -> Mobile {
        Mobile {
            serial,
            ty,
            ext: Ext::NONE,
            created: DateTime::MIN,
            deleted: false,
            next: None,
            previous: None,
            on_link_list: false,
            name: None,
            name_mod: None,
            title: None,
            guild_title: None,
            language: None,
            profile: None,
            profile_locked: false,
            player: false,
            female: false,
            race: None,
            access_level: AccessLevel::Player,
            account: None,
            body: Body::default(),
            body_mod: Body::default(),
            hue: 0,
            hue_mod: -1,
            solid_hue_override: -1,
            direction: Direction::NORTH,
            light_level: 0,
            hair_item_id: 0,
            hair_hue: 0,
            hair_serial: Serial::ZERO,
            facial_hair_item_id: 0,
            facial_hair_hue: 0,
            facial_hair_serial: Serial::ZERO,
            name_hue: -1,
            speech_hue: 0,
            emote_hue: 0,
            whisper_hue: 0,
            yell_hue: 0,
            base_sound_id: 0,
            location: Point3D::default(),
            map: None,
            // `World::new_mobile` sets Map.Internal.DefaultRegion
            region: RegionId::default(),
            walk_region: None,
            logout_location: Point3D::default(),
            logout_map: None,
            str: 0,
            dex: 0,
            int: 0,
            str_lock: StatLockType::Up,
            dex_lock: StatLockType::Up,
            int_lock: StatLockType::Up,
            stat_cap: 0,
            hits: 0,
            stam: 0,
            mana: 0,
            hunger: 0,
            thirst: 0,
            bac: 0,
            fame: 0,
            karma: 0,
            kills: 0,
            tithing_points: 0,
            virtual_armor: 0,
            virtual_armor_mod: 0,
            followers: 0,
            followers_max: 0,
            total_gold: 0,
            total_items: 0,
            total_weight: 0,
            resistances: Vec::new(),
            melee_damage_absorb: 0,
            magic_damage_absorb: 0,
            allowed_stealth_steps: 0,
            skills: None,
            stat_mods: Vec::new(),
            resistance_mods: Vec::new(),
            skill_mods: Vec::new(),
            last_str_gain: DateTime::MIN,
            last_int_gain: DateTime::MIN,
            last_dex_gain: DateTime::MIN,
            warmode: false,
            hidden: false,
            blessed: false,
            flying: false,
            frozen: false,
            paralyzed: false,
            criminal: false,
            can_hear_ghosts: false,
            display_guild_title: false,
            ignore_mobiles: false,
            yellow_healthbar: false,
            meditating: false,
            can_swim: false,
            cant_walk: false,
            squelched: false,
            pushing: false,
            target_locked: false,
            auto_page_notify: false,
            disarm_ready: false,
            stun_ready: false,
            no_move_hs: false,
            in_delta_queue: false,
            delta_flags: MobileDelta::NONE,
            poison: None,
            combatant: None,
            changing_combatant: 0,
            last_killer: None,
            guild_fealty: None,
            guild: None,
            party: None,
            corpse: None,
            backpack: None,
            bank_box: None,
            holding: None,
            mount_item: None,
            weapon: None,
            items: Vec::new(),
            aggressors: Vec::new(),
            aggressed: Vec::new(),
            damage_entries: Vec::new(),
            spawner: None,
            spell: None,
            target: None,
            prompt: None,
            net_state: None,
            property_list: None,
            actions: Vec::new(),
            next_combat_time: 0,
            last_move_time: 0,
            next_action_time: 0,
            next_action_message: 0,
            next_skill_time: tick_count,
            next_spell_time: 0,
            next_warmode_change: DateTime::MIN,
            warmode_changes: 0,
            warmode_spam_value: false,
            auto_manifest_timer: TimerHandle::default(),
            combat_timer: TimerHandle::default(),
            expire_aggr_timer: TimerHandle::default(),
            expire_combatant_timer: TimerHandle::default(),
            expire_criminal_timer: TimerHandle::default(),
            frozen_timer: TimerHandle::default(),
            logout_timer: TimerHandle::default(),
            para_timer: TimerHandle::default(),
            mana_timer: TimerHandle::default(),
            hits_timer: TimerHandle::default(),
            stam_timer: TimerHandle::default(),
            poison_timer: TimerHandle::default(),
        }
    }

    pub fn id(&self) -> MobileId {
        MobileId(self.serial)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    static HUMAN: MobileType = MobileType {
        full_name: "Server.Mobiles.Human",
        parent: Some(&MOBILE_BASE),
        luck: |_, _| 100,
        ..MOBILE_BASE
    };

    static ELDER: MobileType = MobileType {
        full_name: "Server.Mobiles.Elder",
        parent: Some(&HUMAN),
        armor_rating: |_, _| 12.5,
        ..HUMAN
    };

    #[test]
    fn inherits_through_parent() {
        let mut w = World::new();
        let id = w.new_mobile(&ELDER);
        assert_eq!((ELDER.luck)(&w, id), 100, "from Human");
        assert_eq!((ELDER.armor_rating)(&w, id), 12.5);
        assert_eq!((HUMAN.armor_rating)(&w, id), 0.0);
        assert_eq!((MOBILE_BASE.luck)(&w, id), 0);
        assert!(ELDER.is(&HUMAN) && ELDER.is(&MOBILE_BASE) && !HUMAN.is(&ELDER));
    }

    #[test]
    fn constructor_defaults() {
        let m = Mobile::new_at(Serial::new(1), &MOBILE_BASE, DateTime::MIN, 77);
        assert_eq!(
            (m.stat_cap, m.followers_max, m.map, m.next_skill_time),
            (225, 5, Some(MapId::INTERNAL), 77)
        );
        assert!(m.skills.is_some() && m.auto_page_notify);
        assert_eq!((m.hue_mod, m.solid_hue_override, m.name_hue), (-1, -1, -1));
        let loaded = Mobile::new_for_load(Serial::new(2), &MOBILE_BASE, 5);
        assert_eq!(
            (loaded.stat_cap, loaded.map, loaded.skills.is_none()),
            (0, None, true)
        );
    }

    #[test]
    fn derived_defaults() {
        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        w.mobile_mut(id).unwrap().str = 100;
        assert_eq!((MOBILE_BASE.hits_max)(&w, id), 100);
        assert!((MOBILE_BASE.alive)(&w, id));
        w.mobile_mut(id).unwrap().deleted = true;
        assert!(!(MOBILE_BASE.alive)(&w, id));
    }

    #[test]
    fn enum_values_are_fixed() {
        assert_eq!(AccessLevel::Owner as i32, 6);
        assert!(AccessLevel::GameMaster > AccessLevel::Counselor);
        assert_eq!(MobileDelta::HEALTHBAR_POISON.bits(), 0x0100_0000);
        assert_eq!(MobileDelta::ATTRIBUTES.bits(), 0x1C);
        assert_eq!(StatType::ALL, StatType::STR | StatType::DEX | StatType::INT);
        assert_eq!(StatLockType::Locked as u8, 2);
        assert_eq!(ResistanceType::Energy as i32, 4);
    }
}
