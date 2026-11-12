//! The player mobile: `PlayerFlag`, `NpcGuild`, `SolenFriendship`, `BlockMountType`, the
//! [`PlayerData`] family struct, the vtable [`PLAYER_MOBILE`] and the save layout (version 34).
//!
//! `PlayerMobile` adds fields to `Mobile`; here they live in [`PlayerData`] under `Mobile::ext`. A mobile made
//! by the loader (`Mobile::new_for_load`) has no `ext` yet, so `deserialize` installs it.
//!
//! What PlayerMobile calls in systems outside this module (quests, virtues, factions, ethics, duels, house design,
//! insurance, the special moves, animal form, guilds, regions, pets) is a hook in [`hooks`], set by the owning
//! system's `register`. Save bytes: fields are read and written in the fixed order existing save files use. Data a record carries for
//! another system (legacy virtue/champion/murder fields, a quest in progress) is kept in that system's table
//! ([`tables`], [`quest_record`]) so that a resave keeps it.

use indexmap::IndexSet;
use muo_io::{GenericReader, GenericWriter, IoError, Result};
use muo_primitives::{DateTime, Direction, Point3D, TimeSpan};
use muo_server::ids::MobileId;
use muo_server::mobile::{self, AccessLevel, MOBILE_BASE, MobileType, ResistanceType, StatType};
use muo_server::race::RaceId;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::SkillName;
use muo_server::{Ext, World};

pub mod buff;
pub mod buff_icon;
pub mod hooks;
pub mod members;
pub mod quest_record;
pub mod quest_tables;
pub mod tables;

use crate::aos::AosAttribute;
use crate::misc::aos_helpers::attribute_value;

bitflags::bitflags! {
    /// `PlayerFlag`. The first 16 bits are reserved for the default distribution.
    #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
    pub struct PlayerFlag: i32 {
        const GLASSBLOWING = 0x0000_0001;
        const MASONRY = 0x0000_0002;
        const SAND_MINING = 0x0000_0004;
        const STONE_MINING = 0x0000_0008;
        const TOGGLE_MINING_STONE = 0x0000_0010;
        const KARMA_LOCKED = 0x0000_0020;
        const AUTO_RENEW_INSURANCE = 0x0000_0040;
        const USE_OWN_FILTER = 0x0000_0080;
        const PAGING_SQUELCHED = 0x0000_0200;
        const YOUNG = 0x0000_0400;
        const ACCEPT_GUILD_INVITES = 0x0000_0800;
        const DISPLAY_CHAMPION_TITLE = 0x0000_1000;
        const HAS_STAT_REWARD = 0x0000_2000;
        const REFUSE_TRADES = 0x0000_4000;
        // Custom flags start at 0x00010000; unknown bits are kept so a save round-trips.
        const _ = !0;
    }
}

/// `NpcGuild`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum NpcGuild {
    #[default]
    None = 0,
    MagesGuild,
    WarriorsGuild,
    ThievesGuild,
    RangersGuild,
    HealersGuild,
    MinersGuild,
    MerchantsGuild,
    TinkersGuild,
    TailorsGuild,
    FishermensGuild,
    BardsGuild,
    BlacksmithsGuild,
}

/// `SolenFriendship`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum SolenFriendship {
    #[default]
    None = 0,
    Red = 1,
    Black = 2,
}

/// `BlockMountType`.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum BlockMountType {
    #[default]
    None = -1,
    /// You are still too dazed from being knocked off your mount to ride!
    Dazed = 1040024,
    /// You cannot mount while recovering from a bola throw.
    BolaRecovery = 1062910,
    /// You cannot mount while recovering from a dismount special maneuver.
    DismountRecovery = 1070859,
}

/// `Race.Human`, `Race.Elf`, `Race.Gargoyle`: the `RaceIndex` `RaceDefinitions` gives them.
pub const RACE_HUMAN: RaceId = RaceId(0);
pub const RACE_ELF: RaceId = RaceId(1);
pub const RACE_GARGOYLE: RaceId = RaceId(2);

/// `BOBFilter`, serialized by the generator at version 2.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BobFilter {
    pub ty: i32,
    pub quality: i32,
    pub material: i32,
    pub quantity: i32,
}

impl BobFilter {
    pub fn is_default(&self) -> bool {
        *self == BobFilter::default()
    }

    pub fn clear(&mut self) {
        *self = BobFilter::default();
    }

    /// `BOBFilter.Serialize` (generated): version, save flags, then each non-zero field.
    pub fn serialize(&self, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(2);
        let flags = i32::from(self.ty != 0)
            | i32::from(self.quality != 0) << 1
            | i32::from(self.material != 0) << 2
            | i32::from(self.quantity != 0) << 3;
        writer.write_enum(4, i64::from(flags));
        for (bit, v) in [
            (1, self.ty),
            (2, self.quality),
            (4, self.material),
            (8, self.quantity),
        ] {
            if flags & bit != 0 {
                writer.write_i32(v);
            }
        }
    }

    /// `BOBFilter.Deserialize` (generated), with the version 1 migration (four encoded ints).
    pub fn deserialize(reader: &mut dyn GenericReader) -> Result<BobFilter> {
        let version = reader.read_encoded_int()?;
        let mut f = BobFilter::default();
        if version < 2 {
            if version == 1 {
                f.ty = reader.read_encoded_int()?;
                f.quality = reader.read_encoded_int()?;
                f.material = reader.read_encoded_int()?;
                f.quantity = reader.read_encoded_int()?;
            }
            return Ok(f);
        }
        let flags = reader.read_enum(4)? as i32;
        if flags & 1 != 0 {
            f.ty = reader.read_i32()?;
        }
        if flags & 2 != 0 {
            f.quality = reader.read_i32()?;
        }
        if flags & 4 != 0 {
            f.material = reader.read_i32()?;
        }
        if flags & 8 != 0 {
            f.quantity = reader.read_i32()?;
        }
        Ok(f)
    }
}

/// Everything `PlayerMobile` adds to `Mobile`.
pub struct PlayerData {
    /// `_acquiredRecipes` (insertion ordered; `None` is null).
    pub acquired_recipes: Option<IndexSet<i32>>,
    /// `_allFollowers`: not saved.
    pub all_followers: Option<IndexSet<MobileId>>,
    /// `m_BeardModID`, `m_BeardModHue`, `m_HairModID`, `m_HairModHue`.
    pub beard_mod_id: i32,
    pub beard_mod_hue: i32,
    pub hair_mod_id: i32,
    pub hair_mod_hue: i32,
    /// `m_GameTime`.
    pub game_time: TimeSpan,
    /// `m_GuildRank.Rank`: an index into `RankDefinition.Ranks` (0 Ronin .. 4 Leader).
    pub guild_rank: i32,
    /// `_honorTime`.
    pub honor_time: DateTime,
    pub last_global_light: i32,
    pub last_personal_light: i32,
    pub last_protected_message: bool,
    pub last_young_heal: DateTime,
    pub last_young_message: DateTime,
    pub next_just_award: DateTime,
    pub next_protection_check: i32,
    /// `m_NextSmithBulkOrder`, `m_NextTailorBulkOrder`, `m_SavagePaintExpiration`: absolute times.
    pub next_smith_bulk_order: DateTime,
    pub next_tailor_bulk_order: DateTime,
    pub savage_paint_expiration: DateTime,
    pub no_delta_recursion: bool,
    /// `m_StuckMenuUses`.
    pub stuck_menu_uses: Option<Vec<DateTime>>,
    pub ankh_next_use: DateTime,
    pub peaced_until: DateTime,
    pub last_escort_time: DateTime,
    pub last_pet_ball_time: DateTime,
    pub session_start: DateTime,
    /// `VisibilityList`, `PermaFlags`.
    pub visibility_list: Vec<MobileId>,
    pub perma_flags: Vec<MobileId>,
    pub bob_filter: BobFilter,
    /// `Stabled`, `AutoStabled` (null when never used).
    pub stabled: Option<IndexSet<MobileId>>,
    pub auto_stabled: Option<IndexSet<MobileId>>,
    pub guild_message_hue: i32,
    pub alliance_message_hue: i32,
    pub profession: i32,
    pub is_stealthing: bool,
    pub npc_guild: NpcGuild,
    pub npc_guild_join_time: DateTime,
    pub npc_guild_game_time: TimeSpan,
    pub next_bod_turn_in_time: DateTime,
    pub last_online: DateTime,
    pub tot_items_turned_in: i32,
    pub tot_total_monster_fame: i32,
    pub executes_lightning_strike: i32,
    pub flags: PlayerFlag,
    pub accelerated_start: DateTime,
    pub accelerated_skill: SkillName,
    pub solen_friendship: SolenFriendship,
    /// `EnemyOfOneType`: the class a Ranger's Enemy of One targets (runtime only, not saved).
    pub enemy_of_one_type: Option<&'static MobileType>,
    pub waiting_for_enemy: bool,
    pub ninja_wep_cooldown: bool,
    pub bedroll_logout: bool,
    /// `Learning` ((SkillName)(-1) is none).
    pub learning: Option<SkillName>,
    /// `DoneQuests` (QuestRestartInfo).
    pub done_quests: Vec<(QuestTypeRef, DateTime)>,
    /// `Quest`: the bytes `QuestSerializer.Serialize` wrote after the leading `1` (see [`quest_record`]).
    pub quest: Option<Vec<u8>>,
    /// `m_BuffTable` (runtime only).
    pub buff_table: buff::BuffTable,
    /// `_mountBlock` (runtime only).
    pub mount_block: Option<members::MountBlock>,
    /// `EquipSnapshot` (runtime only).
    pub equip_snapshot: Option<Vec<muo_server::ids::ItemId>>,
    /// `SpeechLog` (runtime only).
    pub speech_log: Option<std::collections::VecDeque<members::SpeechLogEntry>>,
    // DuelPlayer/DuelContext, DesignContext, EthicPlayer, FactionPlayerState, QuestArrow, SentHonorContext and
    // ReceivedHonorContext are runtime-only (never saved) and belong to their systems; [`hooks`] reads them.
}

/// A quest type as `QuestSerializer.Write` stores it: a reference-table index or a full name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuestTypeRef {
    Indexed(i32),
    Named(String),
    None,
}

impl Default for PlayerData {
    /// `new PlayerMobile()`'s field initializers and constructor body.
    fn default() -> Self {
        PlayerData {
            acquired_recipes: None,
            all_followers: None,
            beard_mod_id: -1,
            beard_mod_hue: 0,
            hair_mod_id: -1,
            hair_mod_hue: 0,
            game_time: TimeSpan::ZERO,
            guild_rank: 0, // RankDefinition.Lowest
            honor_time: DateTime::MIN,
            last_global_light: -1,
            last_personal_light: -1,
            last_protected_message: false,
            last_young_heal: DateTime::MIN,
            last_young_message: DateTime::MIN,
            next_just_award: DateTime::MIN,
            next_protection_check: 10,
            next_smith_bulk_order: DateTime::MIN,
            next_tailor_bulk_order: DateTime::MIN,
            savage_paint_expiration: DateTime::MIN,
            no_delta_recursion: false,
            stuck_menu_uses: None,
            ankh_next_use: DateTime::MIN,
            peaced_until: DateTime::MIN,
            last_escort_time: DateTime::MIN,
            last_pet_ball_time: DateTime::MIN,
            session_start: DateTime::MIN,
            visibility_list: Vec::new(),
            perma_flags: Vec::new(),
            bob_filter: BobFilter::default(),
            stabled: None,
            auto_stabled: None,
            guild_message_hue: 0,
            alliance_message_hue: 0,
            profession: 0,
            is_stealthing: false,
            npc_guild: NpcGuild::None,
            npc_guild_join_time: DateTime::MIN,
            npc_guild_game_time: TimeSpan::ZERO,
            next_bod_turn_in_time: DateTime::MIN,
            last_online: DateTime::MIN,
            tot_items_turned_in: 0,
            tot_total_monster_fame: 0,
            executes_lightning_strike: 0,
            flags: PlayerFlag::empty(),
            accelerated_start: DateTime::MIN,
            accelerated_skill: SkillName::Alchemy,
            solen_friendship: SolenFriendship::None,
            enemy_of_one_type: None,
            waiting_for_enemy: false,
            ninja_wep_cooldown: false,
            bedroll_logout: false,
            learning: None,
            done_quests: Vec::new(),
            quest: None,
            buff_table: buff::BuffTable::default(),
            mount_block: None,
            equip_snapshot: None,
            speech_log: None,
        }
    }
}

impl PlayerData {
    /// `GetFlag`.
    pub fn flag(&self, flag: PlayerFlag) -> bool {
        self.flags.intersects(flag)
    }

    /// `SetFlag`.
    pub fn set_flag(&mut self, flag: PlayerFlag, value: bool) {
        self.flags.set(flag, value);
    }
}

/// The player state of `id`, if it is a PlayerMobile that has it.
pub fn data(w: &World, id: MobileId) -> Option<&PlayerData> {
    w.mobile(id)?.ext.get::<PlayerData>()
}

/// As [`data`], mutable; a PlayerMobile without state (made by hand) gets the defaults.
pub fn data_mut(w: &mut World, id: MobileId) -> Option<&mut PlayerData> {
    let m = w.mobile_mut(id)?;
    if m.ext.get::<PlayerData>().is_none() {
        if !m.ty.is(&PLAYER_MOBILE) {
            return None;
        }
        m.ext = Ext::new(PlayerData::default());
    }
    m.ext.get_mut::<PlayerData>()
}

/// `new PlayerMobile()`: a mobile of class `PlayerMobile` with its state, added to the world.
pub fn new_player_mobile(w: &mut World) -> MobileId {
    let id = w.new_mobile(&PLAYER_MOBILE);
    if let Some(m) = w.mobile_mut(id) {
        m.ext = Ext::new(PlayerData::default());
    }
    id
}

/// `Mobile.Race`'s index: null reads as `Race.DefaultRace`, which is Human (`Races[0]`).
fn race_of(w: &World, id: MobileId) -> RaceId {
    w.mobile(id).and_then(|m| m.race).unwrap_or(RACE_HUMAN)
}

// ---- the virtuals PlayerMobile overrides ----

/// `DuelContext?.Started == true && !DuelContext.Finished`.
fn duel_in_progress(w: &World, id: MobileId) -> bool {
    hooks::duel_state(w, id).is_some_and(|d| d.started && !d.finished)
}

fn max_weight(w: &World, id: MobileId) -> i32 {
    let str = w.mobile(id).map_or(0, |m| (m.ty.str)(w, id));
    let base = if crate::core::ml(w) && race_of(w, id) == RACE_HUMAN {
        100
    } else {
        40
    };
    base + (3.5 * f64::from(str)) as i32
}

fn racial_skill_bonus(w: &World, id: MobileId) -> f64 {
    if crate::core::ml(w) && race_of(w, id) == RACE_HUMAN {
        20.0
    } else {
        0.0
    }
}

fn add_armor_rating(w: &World, rating: &mut f64, armor: Option<muo_server::ids::ItemId>) {
    use crate::aos::{AosArmorAttribute, AttrKind};
    let Some(a) = armor else { return };
    if crate::items::armor::is_armor(w, a)
        && (!crate::core::aos(w)
            || crate::items::attr(w, a, AttrKind::Armor, AosArmorAttribute::MageArmor as u32) == 0)
    {
        *rating += crate::items::armor::armor_rating_scaled(w, a);
    }
}

fn armor_rating(w: &World, id: MobileId) -> f64 {
    use mobile::part1::props_b as p;
    let mut rating = 0.0;
    for armor in [
        p::neck_armor(w, id),
        p::hand_armor(w, id),
        p::head_armor(w, id),
        p::arms_armor(w, id),
        p::legs_armor(w, id),
        p::chest_armor(w, id),
        p::shield_armor(w, id),
    ] {
        add_armor_rating(w, &mut rating, armor);
    }
    w.mobile(id).map_or(0.0, |m| {
        f64::from(m.virtual_armor) + f64::from(m.virtual_armor_mod) + rating
    })
}

fn luck(w: &World, id: MobileId) -> i32 {
    attribute_value(w, id, AosAttribute::Luck)
}

/// `NewGuildDisplay`: `Guilds.Guild.NewGuildSystem` is `Core.SE`.
fn new_guild_display(w: &World, _id: MobileId) -> bool {
    crate::core::se(w)
}

fn murderer(w: &World, id: MobileId) -> bool {
    // PingPongEnabled is `Core.T2A && !Core.LBR`.
    let ping_pong = crate::core::t2a(w)
        && !crate::core::lbr(w)
        && w.resources
            .get::<tables::MurderTable>()
            .and_then(|t| t.0.get(&id))
            .is_some_and(|c| c.ping_pongs >= 5);
    ping_pong || (MOBILE_BASE.murderer)(w, id)
}

fn set_paralyzed(w: &mut World, id: MobileId, value: bool) {
    (MOBILE_BASE.set_paralyzed)(w, id, value);
    if value {
        // Paralyze/You are frozen and can not move
        buff::add_buff(
            w,
            id,
            buff::BuffInfo::new(buff::BuffIcon::Paralyze, 1075827),
        );
    } else {
        buff::remove_buff(w, id, buff::BuffIcon::Paralyze);
    }
}

fn capped_stat(w: &World, id: MobileId, base: i32) -> i32 {
    if crate::core::ml(w)
        && w.mobile(id)
            .is_some_and(|m| m.access_level == AccessLevel::Player)
    {
        base.min(150)
    } else {
        base
    }
}

fn str(w: &World, id: MobileId) -> i32 {
    capped_stat(w, id, mobile::base::str_base(w, id))
}

fn int(w: &World, id: MobileId) -> i32 {
    capped_stat(w, id, mobile::base::int_base(w, id))
}

fn dex(w: &World, id: MobileId) -> i32 {
    capped_stat(w, id, mobile::base::dex_base(w, id))
}

fn hits_max(w: &World, id: MobileId) -> i32 {
    let Some(m) = w.mobile(id) else { return 0 };
    let (str_base, str_offs);
    if crate::core::aos(w) {
        str_base = (m.ty.str)(w, id); // Str already includes GetStatOffset
        let mut offs = attribute_value(w, id, AosAttribute::BonusHits);
        if crate::core::ml(w) && offs > 25 && m.access_level <= AccessLevel::Player {
            offs = 25;
        }
        if hooks::animal_form_under_type(w, id, "BakeKitsune")
            || hooks::animal_form_under_type(w, id, "GreyWolf")
        {
            offs += 20;
        }
        str_offs = offs;
    } else {
        str_base = m.str; // RawStr
        str_offs = mobile::part2::stats::get_stat_offset(w, id, StatType::STR);
    }
    str_base / 2 + 50 + str_offs
}

fn stam_max(w: &World, id: MobileId) -> i32 {
    mobile::base::stam_max_base(w, id) + attribute_value(w, id, AosAttribute::BonusStam)
}

fn mana_max(w: &World, id: MobileId) -> i32 {
    mobile::base::mana_max_base(w, id)
        + attribute_value(w, id, AosAttribute::BonusMana)
        + if crate::core::ml(w) && race_of(w, id) == RACE_ELF {
            20
        } else {
            0
        }
}

fn on_skill_invalidated(w: &mut World, id: MobileId, skill: SkillName) {
    if crate::core::aos(w)
        && skill == SkillName::MagicResist
        && let Some(ty) = w.mobile(id).map(|m| m.ty)
    {
        (ty.update_resistances)(w, id);
    }
}

fn get_max_resistance(w: &World, id: MobileId, ty: ResistanceType) -> i32 {
    let Some(m) = w.mobile(id) else { return 0 };
    if m.access_level > AccessLevel::Player {
        return 100;
    }
    let mut max = mobile::base::get_max_resistance_base(w, id, ty);
    if ty != ResistanceType::Physical && hooks::curse_under_effect(w, id) {
        max -= 10;
    }
    if crate::core::ml(w) && race_of(w, id) == RACE_ELF && ty == ResistanceType::Energy {
        max += 5; // intended to go after the 60 max from curse
    }
    max
}

fn get_min_resistance(w: &World, id: MobileId, ty: ResistanceType) -> i32 {
    let magic_resist = (muo_server::skills::value_ref(w, id, SkillName::MagicResist) * 10.0) as i32;
    let min = if magic_resist >= 1000 {
        40 + (magic_resist - 1000) / 50
    } else if magic_resist >= 400 {
        (magic_resist - 400) / 15
    } else {
        i32::MIN
    };
    let lo = mobile::base::get_min_resistance_base(w, id, ty);
    let hi = mobile::part1::props_a::max_player_resistance(w);
    min.clamp(lo.min(hi), hi)
}

fn on_race_change(w: &mut World, id: MobileId, _old: RaceId) {
    members::validate_equipment(w, id);
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.update_resistances)(w, id);
    }
}

fn on_net_state_changed(w: &mut World, id: MobileId) {
    if let Some(d) = data_mut(w, id) {
        d.last_global_light = -1;
        d.last_personal_light = -1;
    }
}

fn compute_base_light_levels(w: &World, id: MobileId) -> (i32, i32) {
    let global = crate::misc::light_cycle::compute_level_for(w, id);
    let racial_night_sight = crate::core::ml(w) && race_of(w, id) == RACE_ELF;
    let light_level = w.mobile(id).map_or(0, |m| m.light_level);
    let personal = if light_level < 21
        && (attribute_value(w, id, AosAttribute::NightSight) > 0 || racial_night_sight)
    {
        21
    } else {
        light_level
    };
    (global, personal)
}

fn check_light_levels(w: &mut World, id: MobileId, force_resend: bool) {
    let Some(m) = w.mobile(id) else { return };
    let (Some(ns), ty, serial) = (m.net_state, m.ty, m.serial) else {
        return;
    };
    let (global, personal) = (ty.compute_light_levels)(w, id);
    let Some(d) = data_mut(w, id) else { return };
    let force = force_resend || global != d.last_global_light || personal != d.last_personal_light;
    if !force {
        return;
    }
    d.last_global_light = global;
    d.last_personal_light = personal;
    muo_server::packets::light::send_global_light_level(w, ns, global);
    muo_server::packets::light::send_personal_light_level(w, ns, serial, personal);
}

fn on_mana_change(w: &mut World, id: MobileId, old: i32) {
    (MOBILE_BASE.on_mana_change)(w, id, old);
    let (limit, mana, meditating) = (
        data(w, id).map_or(0, |d| d.executes_lightning_strike),
        w.mobile(id).map_or(0, |m| m.mana),
        w.mobile(id).is_some_and(|m| m.meditating),
    );
    if limit > 0 && mana < limit {
        hooks::special_move_clear_current(w, id);
    }
    if !meditating {
        buff::remove_buff(w, id, buff::BuffIcon::ActiveMeditation);
    }
}

fn apply_name_suffix(w: &World, id: MobileId, suffix: &str) -> String {
    let mut suffix = suffix.to_owned();
    if young(w, id) {
        suffix = if suffix.is_empty() {
            "(Young)".to_owned()
        } else {
            format!("{suffix} (Young)")
        };
    }
    if let Some(adjunct) = hooks::ethic_adjunct(w, id) {
        suffix = if suffix.is_empty() {
            adjunct
        } else {
            format!("{suffix} {adjunct}")
        };
    }
    if crate::core::ml(w)
        && let Some(facet) = hooks::faction_facet(w)
        && w.mobile(id).is_some_and(|m| m.map == Some(facet))
        && let Some(abbr) = hooks::faction_abbreviation(w, id)
    {
        let adjunct = abbr; // the hook returns it already bracketed
        suffix = if suffix.is_empty() {
            adjunct
        } else {
            format!("{suffix} {adjunct}")
        };
    }
    (MOBILE_BASE.apply_name_suffix)(w, id, &suffix)
}

fn get_logout_delay(w: &World, id: MobileId) -> TimeSpan {
    if young(w, id)
        || data(w, id).is_some_and(|d| d.bedroll_logout)
        || hooks::test_center_enabled(w)
    {
        return TimeSpan::ZERO;
    }
    (MOBILE_BASE.get_logout_delay)(w, id)
}

fn compute_movement_speed(w: &World, id: MobileId, dir: Direction, check_turning: bool) -> i32 {
    let facing = w.mobile(id).map_or(Direction::NORTH, |m| m.direction);
    let delays = w.resources.get::<muo_server::movement::Movement>();
    if check_turning && dir.facing() != facing.facing() {
        return delays.map_or(0, |m| m.turn_delay); // a direction change, not a step
    }
    if hooks::transformation_under(w, id, "ReaperFormSpell") {
        return delays.map_or(0, |m| m.walk_foot_delay);
    }
    let running = dir.is_running();
    let on_horse = w.mobile(id).is_some_and(|m| m.mount_item.is_some());
    if on_horse || hooks::animal_form_speed_boost(w, id) {
        return delays.map_or(0, |m| {
            if running {
                m.run_mount_delay
            } else {
                m.walk_mount_delay
            }
        });
    }
    delays.map_or(0, |m| {
        if running {
            m.run_foot_delay
        } else {
            m.walk_foot_delay
        }
    })
}

fn apply_poison(
    w: &mut World,
    id: MobileId,
    from: Option<MobileId>,
    mut poison: muo_server::poison::PoisonId,
) -> mobile::ApplyPoisonResult {
    let alive = w.mobile(id).is_some_and(|m| (m.ty.alive)(w, id));
    if !alive {
        return mobile::ApplyPoisonResult::Immune;
    }
    let evil_omen = crate::hooks::hooks(w)
        .and_then(|h| h.evil_omen_end_effect)
        .is_some_and(|f| f(w, id));
    if evil_omen
        && let Some(p) = w
            .resources
            .get::<muo_server::poison::Poisons>()
            .and_then(|p| p.increase_level(Some(poison)))
    {
        poison = p;
    }
    let result = (MOBILE_BASE.apply_poison)(w, id, from, poison);
    if let Some(f) = from
        && result == mobile::ApplyPoisonResult::Poisoned
    {
        crate::misc::poison_kinds::set_source(w, id, f); // timer.From = from
    }
    result
}

/// The death-time move result shared by parent and inventory moves.
fn death_move_result(
    w: &mut World,
    id: MobileId,
    item: muo_server::ids::ItemId,
    base: fn(&mut World, MobileId, muo_server::ids::ItemId) -> muo_server::item::DeathMoveResult,
) -> muo_server::item::DeathMoveResult {
    use muo_server::item::DeathMoveResult;
    // It seems all items are unmarked on death, even blessed/insured ones
    if w.item(item).is_some_and(|i| i.quest_item()) {
        muo_server::item::ops::set_quest_item(w, item, false);
    }
    if hooks::insurance_item_on_death(w, id, item) {
        return DeathMoveResult::MoveToBackpack;
    }
    let res = base(w, id, item);
    if res == DeathMoveResult::MoveToCorpse
        && w.item(item).is_some_and(|i| i.movable())
        && young(w, id)
    {
        DeathMoveResult::MoveToBackpack
    } else {
        res
    }
}

fn get_parent_move_result_for(
    w: &mut World,
    id: MobileId,
    item: muo_server::ids::ItemId,
) -> muo_server::item::DeathMoveResult {
    death_move_result(w, id, item, MOBILE_BASE.get_parent_move_result_for)
}

fn get_inventory_move_result_for(
    w: &mut World,
    id: MobileId,
    item: muo_server::ids::ItemId,
) -> muo_server::item::DeathMoveResult {
    death_move_result(w, id, item, MOBILE_BASE.get_inventory_move_result_for)
}

fn check_shove(w: &mut World, id: MobileId, shoved: MobileId) -> bool {
    let ignore = |w: &World, m| w.mobile(m).is_some_and(|x| x.ignore_mobiles);
    ignore(w, id)
        || ignore(w, shoved)
        || hooks::transformation_under(w, shoved, "WraithFormSpell")
        || (MOBILE_BASE.check_shove)(w, id, shoved)
}

fn check_poison_immunity(
    w: &World,
    id: MobileId,
    from: Option<MobileId>,
    poison: muo_server::poison::PoisonId,
) -> bool {
    (young(w, id) && !duel_in_progress(w, id))
        || (MOBILE_BASE.check_poison_immunity)(w, id, from, poison)
}

fn on_poison_immunity(
    w: &mut World,
    id: MobileId,
    from: Option<MobileId>,
    poison: muo_server::poison::PoisonId,
) {
    if young(w, id) && !duel_in_progress(w, id) {
        // You would have been poisoned, were you not new to the land of Britannia. Be careful in the future.
        mobile::part2::messages::send_localized_message(w, id, 502808, "", 0x3B2);
    } else {
        (MOBILE_BASE.on_poison_immunity)(w, id, from, poison);
    }
}

fn on_kills_change(w: &mut World, id: MobileId, old: i32) {
    let kills = w.mobile(id).map_or(0, |m| m.kills);
    if young(w, id)
        && kills > old
        && let Some(a) = w.mobile(id).and_then(|m| m.account)
    {
        crate::accounting::remove_young_status(w, a, 0);
    }
}

fn on_skill_change(w: &mut World, id: MobileId, skill: SkillName, _old_base: f64) {
    if young(w, id)
        && muo_server::skills::total(w, id) >= 4500
        && let Some(a) = w.mobile(id).and_then(|m| m.account)
    {
        // You have successfully obtained a respectable skill level, and have outgrown your status as a young player!
        crate::accounting::remove_young_status(w, a, 1019036);
    }
    if hooks::ml_quest_enabled(w) {
        hooks::ml_quest_skill_gain(w, id, skill);
    }
}

fn on_access_level_changed(w: &mut World, id: MobileId, _old: AccessLevel) {
    let value = w
        .mobile(id)
        .is_some_and(|m| m.access_level != AccessLevel::Player);
    muo_server::mobile::base::set_ignore_mobiles(w, id, value);
}

pub fn young(w: &World, id: MobileId) -> bool {
    hooks::young_player_system(w) && data(w, id).is_some_and(|d| d.flag(PlayerFlag::YOUNG))
}

pub fn set_young(w: &mut World, id: MobileId, value: bool) {
    if let Some(d) = data_mut(w, id) {
        d.set_flag(PlayerFlag::YOUNG, value);
    }
    mobile::part2::properties::invalidate_properties(w, id);
}

// ---- plain members ----

pub fn get_direction4(from: Point3D, to: Point3D) -> Direction {
    let dx = from.x - to.x;
    let dy = from.y - to.y;
    let rx = dx - dy;
    let ry = dx + dy;
    if rx >= 0 && ry >= 0 {
        Direction::WEST
    } else if rx >= 0 && ry < 0 {
        Direction::SOUTH
    } else if rx < 0 && ry < 0 {
        Direction::EAST
    } else {
        Direction::NORTH
    }
}

pub fn savage_paint_expiration(w: &World, id: MobileId) -> TimeSpan {
    remaining(w, data(w, id).map(|d| d.savage_paint_expiration))
}

pub fn set_savage_paint_expiration(w: &mut World, id: MobileId, value: TimeSpan) {
    let now = w.now;
    // An overflowing `now + value` leaves the field as it was.
    if let (Some(d), Some(t)) = (data_mut(w, id), now.checked_add(value)) {
        d.savage_paint_expiration = t;
    }
}

fn remaining(w: &World, until: Option<DateTime>) -> TimeSpan {
    let span = until.map_or(TimeSpan::ZERO, |t| t - w.now);
    if span > TimeSpan::ZERO {
        span
    } else {
        TimeSpan::ZERO
    }
}

pub fn next_smith_bulk_order(w: &World, id: MobileId) -> TimeSpan {
    remaining(w, data(w, id).map(|d| d.next_smith_bulk_order))
}

/// Sets the time until the next smith bulk order; an overflowing value leaves it unchanged.
pub fn set_next_smith_bulk_order(w: &mut World, id: MobileId, value: TimeSpan) {
    let now = w.now;
    if let (Some(d), Some(t)) = (data_mut(w, id), now.checked_add(value)) {
        d.next_smith_bulk_order = t;
    }
}

pub fn next_tailor_bulk_order(w: &World, id: MobileId) -> TimeSpan {
    remaining(w, data(w, id).map(|d| d.next_tailor_bulk_order))
}

pub fn set_next_tailor_bulk_order(w: &mut World, id: MobileId, value: TimeSpan) {
    let now = w.now;
    if let (Some(d), Some(t)) = (data_mut(w, id), now.checked_add(value)) {
        d.next_tailor_bulk_order = t;
    }
}

pub fn game_time(w: &World, id: MobileId) -> TimeSpan {
    let Some(d) = data(w, id) else {
        return TimeSpan::ZERO;
    };
    if w.mobile(id).is_some_and(|m| m.net_state.is_some()) {
        d.game_time + (w.now - d.session_start)
    } else {
        d.game_time
    }
}

pub fn can_use_stuck_menu(w: &World, id: MobileId) -> bool {
    let Some(uses) = data(w, id).and_then(|d| d.stuck_menu_uses.as_ref()) else {
        return true;
    };
    uses.iter().any(|t| w.now - *t > TimeSpan::from_days(1))
}

pub fn used_stuck_menu(w: &mut World, id: MobileId) {
    let now = w.now;
    let Some(d) = data_mut(w, id) else { return };
    let uses = d
        .stuck_menu_uses
        .get_or_insert_with(|| vec![DateTime::MIN; 2]);
    if let Some(slot) = uses.iter_mut().find(|t| now - **t > TimeSpan::from_days(1)) {
        *slot = now;
    }
}

/// `CheckYoungProtection`'s rate limit on its message, `CheckYoungHealTime`.
pub fn check_young_heal_time(w: &mut World, id: MobileId) -> bool {
    let now = w.now;
    let Some(d) = data_mut(w, id) else {
        return false;
    };
    if now - d.last_young_heal > TimeSpan::from_mins(5) {
        d.last_young_heal = now;
        true
    } else {
        false
    }
}

pub fn add_follower(w: &mut World, id: MobileId, m: MobileId) {
    if let Some(d) = data_mut(w, id) {
        d.all_followers.get_or_insert_with(IndexSet::new).insert(m);
    }
}

/// `GuildRank`: a Game Master or better is always the Leader (4).
pub fn guild_rank(w: &World, id: MobileId) -> i32 {
    if w.mobile(id)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
    {
        return 4;
    }
    data(w, id).map_or(0, |d| d.guild_rank)
}

pub fn remove_follower(w: &mut World, id: MobileId, m: MobileId) -> bool {
    let Some(d) = data_mut(w, id) else {
        return false;
    };
    let removed = d.all_followers.as_mut().is_some_and(|s| s.shift_remove(&m));
    if removed && d.all_followers.as_ref().is_some_and(IndexSet::is_empty) {
        d.all_followers = None;
    }
    removed
}

pub fn add_stabled(w: &mut World, id: MobileId, m: MobileId) {
    if let Some(d) = data_mut(w, id) {
        d.stabled.get_or_insert_with(IndexSet::new).insert(m);
    }
}

pub fn remove_stabled(w: &mut World, id: MobileId, m: MobileId) -> bool {
    let Some(d) = data_mut(w, id) else {
        return false;
    };
    let removed = d.stabled.as_mut().is_some_and(|s| s.shift_remove(&m));
    if removed && d.stabled.as_ref().is_some_and(IndexSet::is_empty) {
        d.stabled = None;
    }
    removed
}

pub fn has_recipe(w: &World, id: MobileId, recipe_id: i32) -> bool {
    data(w, id)
        .and_then(|d| d.acquired_recipes.as_ref())
        .is_some_and(|s| s.contains(&recipe_id))
}

pub fn acquire_recipe(w: &mut World, id: MobileId, recipe_id: i32) {
    if let Some(d) = data_mut(w, id) {
        d.acquired_recipes
            .get_or_insert_with(IndexSet::new)
            .insert(recipe_id);
    }
}

pub fn remove_recipe(w: &mut World, id: MobileId, recipe_id: i32) {
    if let Some(s) = data_mut(w, id).and_then(|d| d.acquired_recipes.as_mut()) {
        s.shift_remove(&recipe_id);
    }
}

pub fn reset_recipes(w: &mut World, id: MobileId) {
    if let Some(d) = data_mut(w, id) {
        d.acquired_recipes = None;
    }
}

/// `SetHairMods`: -1 restores, -2 leaves alone, anything else swaps the style.
pub fn set_hair_mods(w: &mut World, id: MobileId, hair_id: i32, beard_id: i32) {
    for (hair, new_id) in [(true, hair_id), (false, beard_id)] {
        if new_id == -2 {
            continue;
        }
        let Some(m) = w.mobile(id) else { return };
        let (cur_id, cur_hue) = if hair {
            (m.hair_item_id, m.hair_hue)
        } else {
            (m.facial_hair_item_id, m.facial_hair_hue)
        };
        let Some(d) = data_mut(w, id) else { return };
        let (store_id, store_hue) = if hair {
            (&mut d.hair_mod_id, &mut d.hair_mod_hue)
        } else {
            (&mut d.beard_mod_id, &mut d.beard_mod_hue)
        };
        let (set_id, set_hue);
        if new_id == -1 {
            // InternalRestoreHair
            if *store_id == -1 {
                continue;
            }
            (set_id, set_hue) = (*store_id, *store_hue);
            *store_id = -1;
            *store_hue = 0;
        } else {
            // InternalChangeHair
            if *store_id == -1 {
                *store_id = cur_id;
                *store_hue = cur_hue;
            }
            (set_id, set_hue) = (new_id, 0);
        }
        // CreateHair: the property setters (they send the hair delta, and a hue only sticks on an existing style).
        use muo_server::mobile::part1::props_b as pb;
        if hair {
            pb::set_hair_item_id(w, id, set_id);
            pb::set_hair_hue(w, id, set_hue);
        } else {
            pb::set_facial_hair_item_id(w, id, set_id);
            pb::set_facial_hair_hue(w, id, set_hue);
        }
    }
}

// ---- events ----

type Handler = std::rc::Rc<dyn Fn(&mut World, MobileId)>;

/// `PlayerMobile.PlayerDeathEvent` and `PlayerDeletedEvent` (the generated events); `PlayerLoginEvent` is
/// `network::events::PlayerLogin`.
#[derive(Default)]
struct PlayerEvents {
    death: Vec<Handler>,
    deleted: Vec<Handler>,
}

/// `[OnEvent(nameof(PlayerDeathEvent))]`.
pub fn subscribe_death(w: &mut World, h: impl Fn(&mut World, MobileId) + 'static) {
    w.resources
        .get_mut::<PlayerEvents>()
        .death
        .push(std::rc::Rc::new(h));
}

/// `[OnEvent(nameof(PlayerDeletedEvent))]`.
pub fn subscribe_deleted(w: &mut World, h: impl Fn(&mut World, MobileId) + 'static) {
    w.resources
        .get_mut::<PlayerEvents>()
        .deleted
        .push(std::rc::Rc::new(h));
}

fn raise(w: &mut World, pick: fn(&PlayerEvents) -> &Vec<Handler>, m: MobileId) {
    let hs = w
        .resources
        .get::<PlayerEvents>()
        .map(|e| pick(e).clone())
        .unwrap_or_default();
    for h in hs {
        h(w, m);
    }
}

/// (death, deleted) subscriber counts, for the orphan tests.
pub fn event_counts(w: &World) -> (usize, usize) {
    w.resources
        .get::<PlayerEvents>()
        .map_or((0, 0), |e| (e.death.len(), e.deleted.len()))
}

/// `PlayerDeathEvent(pm)`.
pub fn player_death_event(w: &mut World, m: MobileId) {
    raise(w, |e| &e.death, m);
}

/// `PlayerDeletedEvent(pm)`.
pub fn player_deleted_event(w: &mut World, m: MobileId) {
    raise(w, |e| &e.deleted, m);
}

/// `EventSink.Logout += OnLogout`.
fn on_logout(w: &mut World, m: MobileId) {
    if data(w, m).is_some() {
        members::auto_stable_pets(w, m);
    }
}

/// `EventSink.Connected += EventSink_Connected`.
fn on_connected(w: &mut World, m: MobileId) {
    let now = w.now;
    if let Some(d) = data_mut(w, m) {
        d.session_start = now;
        hooks::quest_start_timer(w, m);
        if let Some(d) = data_mut(w, m) {
            d.bedroll_logout = false;
            d.last_online = now;
        }
    }
    hooks::disguise_start_timer(w, m);
    muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        hooks::special_move_clear_all(w, m)
    });
}

/// `EventSink.Disconnected += EventSink_Disconnected`.
fn on_disconnected(w: &mut World, m: MobileId) {
    // The DesignContext branch: remove the context, eject all from the house, restore relocated entities.
    hooks::design_on_disconnect(w, m);
    let now = w.now;
    if data(w, m).is_some() {
        if let Some(d) = data_mut(w, m) {
            d.game_time += now - d.session_start;
        }
        hooks::quest_stop_timer(w, m);
        if let Some(d) = data_mut(w, m) {
            d.speech_log = None;
            d.last_online = now;
        }
        crate::systems::skills::tracking::clear_quest_arrow(w, m); // `ClearQuestArrow()`
    }
    hooks::disguise_stop_timer(w, m);
}

/// `PlayerMobile.Initialize()`.
pub fn initialize(w: &mut World) {
    w.events.logout.subscribe(|w, m| on_logout(w, *m));
    w.events.connected.subscribe(|w, m| on_connected(w, *m));
    w.events
        .disconnected
        .subscribe(|w, m| on_disconnected(w, *m));
    crate::network::events::subscribe_ranked::<crate::network::events::PlayerLogin>(
        w,
        16,
        |w, e| on_login(w, e.0),
    );
    subscribe_deleted(w, tables::forget_player);
    if crate::core::se(w) {
        muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, members::check_pets);
    }
    // StableMigrations: the old Mobile.Stabled sets move to the PlayerMobile.
    let migrations = w
        .resources
        .get::<mobile::part2::statics::Part2Statics>()
        .and_then(|s| s.stable_migrations.clone());
    for (player, stabled) in migrations.into_iter().flatten() {
        if let Some(d) = data_mut(w, player) {
            d.stabled = Some(stabled);
        }
    }
}

/// `PlayerMobile.OnLogin`, a `PlayerLoginEvent` handler.
pub fn on_login(w: &mut World, from: MobileId) {
    let level = crate::accounting::handler::lockdown_level(w);
    if level > AccessLevel::Player {
        let (account, ns, access) = match w.mobile(from) {
            Some(m) => (m.account, m.net_state, m.access_level),
            None => return,
        };
        let address = ns.and_then(|n| w.net_state(n)).and_then(|d| d.address);
        let has_access = match (account, address) {
            (Some(a), Some(ip)) => crate::accounting::has_access(w, a, ip),
            _ => false,
        };
        let notice;
        if !has_access {
            notice = if access == AccessLevel::Player {
                "The server is currently under lockdown. No players are allowed to log in at this time."
            } else {
                "The server is currently under lockdown. You do not have sufficient access level to connect."
            };
            if let Some(ns) = ns {
                muo_server::timer::start(w, TimeSpan::from_secs(1), TimeSpan::ZERO, 1, move |w| {
                    w.disconnect(ns, "Server is locked down");
                });
            }
        } else if access >= AccessLevel::Administrator {
            notice = "The server is currently under lockdown. As you are an administrator, you may change this from the [Admin gump.";
        } else {
            notice = "The server is currently under lockdown. You have sufficient access level to connect.";
        }
        hooks::lockdown_notice_gump(w, from, notice);
        return;
    }

    hooks::virtue_check_atrophies(w, from);
    members::claim_auto_stabled_pets(w, from);
    hooks::animal_form_start_timer(w, from);
    buff::resend_buffs(w, from);
}

// ---- serialization ----

fn write_mobile_set(w: &World, writer: &mut dyn GenericWriter, set: &Option<IndexSet<MobileId>>) {
    match set {
        None => writer.write_i32(0),
        Some(s) => {
            // Tidy(): drop the deleted ones first.
            let live: Vec<MobileId> = s
                .iter()
                .copied()
                .filter(|id| w.mobile(*id).is_some_and(|m| !m.deleted))
                .collect();
            writer.write_mobile_list(w, &live);
        }
    }
}

fn read_mobile_set_opt(
    reader: &mut dyn GenericReader,
    w: &World,
) -> Result<Option<IndexSet<MobileId>>> {
    let count = reader.read_i32()?;
    if count == 0 {
        return Ok(None);
    }
    let mut set = IndexSet::new();
    for _ in 0..count {
        if let Some(m) = reader.read_mobile(w)? {
            set.insert(m);
        }
    }
    Ok(Some(set))
}

/// Writes the player record (version 34).
pub fn serialize(w: &World, id: MobileId, writer: &mut dyn GenericWriter) {
    (MOBILE_BASE.serialize)(w, id, writer);
    let default = PlayerData::default();
    let d = data(w, id).unwrap_or(&default);

    writer.write_i32(34); // version

    write_mobile_set(w, writer, &d.stabled);

    match &d.stuck_menu_uses {
        Some(uses) => {
            writer.write_bool(true);
            writer.write_i32(uses.len() as i32);
            for t in uses {
                writer.write_date_time_ticks(t.ticks());
            }
        }
        None => writer.write_bool(false),
    }

    writer.write_date_time_ticks(d.peaced_until.ticks());
    writer.write_date_time_ticks(d.ankh_next_use.ticks());
    write_mobile_set(w, writer, &d.auto_stabled);

    match &d.acquired_recipes {
        None => writer.write_i32(0),
        Some(set) => {
            writer.write_i32(set.len() as i32);
            for r in set {
                writer.write_i32(*r);
            }
        }
    }

    writer.write_encoded_int(d.tot_items_turned_in);
    writer.write_i32(d.tot_total_monster_fame);

    writer.write_encoded_int(d.alliance_message_hue);
    writer.write_encoded_int(d.guild_message_hue);

    writer.write_encoded_int(d.guild_rank);
    writer.write_date_time_ticks(d.last_online.ticks());

    writer.write_encoded_int(d.solen_friendship as i32);

    // QuestSerializer.Serialize(Quest, writer)
    match &d.quest {
        None => writer.write_encoded_int(0),
        Some(bytes) => {
            writer.write_encoded_int(1);
            writer.write_bytes(bytes);
        }
    }

    if d.done_quests.is_empty() {
        writer.write_encoded_int(0);
    } else {
        writer.write_encoded_int(d.done_quests.len() as i32);
        for (ty, restart) in &d.done_quests {
            // QuestSerializer.Write(type, QuestSystem.QuestTypes, writer)
            match ty {
                QuestTypeRef::None => writer.write_encoded_int(0),
                QuestTypeRef::Indexed(i) => {
                    writer.write_encoded_int(1);
                    writer.write_encoded_int(*i);
                }
                QuestTypeRef::Named(n) => {
                    writer.write_encoded_int(2);
                    writer.write_string(n);
                }
            }
            writer.write_date_time_ticks(restart.ticks());
        }
    }

    writer.write_encoded_int(d.profession);

    d.bob_filter.serialize(writer);

    let use_mods = d.hair_mod_id != -1 || d.beard_mod_id != -1;
    writer.write_bool(use_mods);
    if use_mods {
        writer.write_i32(d.hair_mod_id);
        writer.write_i32(d.hair_mod_hue);
        writer.write_i32(d.beard_mod_id);
        writer.write_i32(d.beard_mod_hue);
    }

    writer.write_time_span_ticks(savage_paint_expiration(w, id).ticks());

    writer.write_i32(d.npc_guild as i32);
    writer.write_date_time_ticks(d.npc_guild_join_time.ticks());
    writer.write_time_span_ticks(d.npc_guild_game_time.ticks());

    let perma: Vec<MobileId> = d
        .perma_flags
        .iter()
        .copied()
        .filter(|m| w.mobile(*m).is_some_and(|m| !m.deleted))
        .collect();
    writer.write_mobile_list(w, &perma);

    writer.write_time_span_ticks(next_tailor_bulk_order(w, id).ticks());
    writer.write_time_span_ticks(next_smith_bulk_order(w, id).ticks());

    writer.write_i32(d.flags.bits());

    writer.write_time_span_ticks(game_time(w, id).ticks());
}

fn invalid(msg: &str) -> IoError {
    IoError::InvalidData(msg.to_owned())
}

/// Reads the player record.
pub fn deserialize(w: &mut World, id: MobileId, reader: &mut dyn GenericReader) -> Result<()> {
    (MOBILE_BASE.deserialize)(w, id, reader)?;
    let version = reader.read_i32()?;
    let mut d = PlayerData::default();
    if version < 32 {
        tables::virtues_mut(w, id); // `var virtues = version < 32 ? Virtues : null` creates the context
    }
    let utc_now = DateTime::utc_now().ticks();
    let date = |t: i64| DateTime::from_ticks(t.clamp(0, muo_primitives::datetime::MAX_TICKS));
    let now = w.now;

    // Versions are cumulative: a file of version N has every field of versions N down to 0, so reads run in this order.
    if version >= 30 {
        d.stabled = read_mobile_set_opt(reader, w)?;
    }
    if version >= 29 && reader.read_bool()? {
        let n = reader.read_i32()?;
        let mut uses = Vec::with_capacity(n.clamp(0, 16) as usize);
        for _ in 0..n {
            uses.push(date(reader.read_date_time_ticks()?));
        }
        d.stuck_menu_uses = Some(uses);
    }
    if version >= 28 {
        d.peaced_until = date(reader.read_date_time_ticks()?);
    }
    if version >= 27 {
        d.ankh_next_use = date(reader.read_date_time_ticks()?);
    }
    if version >= 26 {
        d.auto_stabled = read_mobile_set_opt(reader, w)?;
    }
    if version >= 25 {
        let count = reader.read_i32()?;
        if count > 0 {
            let mut set = IndexSet::new();
            for _ in 0..count {
                let r = reader.read_i32()?;
                // Don't add in recipes which we haven't gotten or have been removed
                if version > 33 || reader.read_bool()? {
                    set.insert(r);
                }
            }
            d.acquired_recipes = Some(set);
        }
    }
    if (24..32).contains(&version) {
        reader.read_delta_time_ticks(utc_now)?; // LastHonorLoss, not even used
    }
    if (23..33).contains(&version) {
        // ChampionTitles.Deserialize(reader): the context moves to the ChampionTitleSystem table.
        let c = tables::ChampionTitleContext::deserialize_legacy(reader)?;
        *tables::champion_titles_mut(w, id) = c;
    }
    if (22..32).contains(&version) {
        tables::virtues_mut(w, id).last_valor_loss = date(reader.read_date_time_ticks()?);
    }
    if version >= 21 {
        d.tot_items_turned_in = reader.read_encoded_int()?;
        d.tot_total_monster_fame = reader.read_i32()?;
    }
    if version >= 20 {
        d.alliance_message_hue = reader.read_encoded_int()?;
        d.guild_message_hue = reader.read_encoded_int()?;
    }
    let mut guild_rank_read = false;
    let mut savage_paint = false;
    if version >= 19 {
        // RankDefinition.Ranks.Length - 1 == 4
        d.guild_rank = reader.read_encoded_int()?.min(4);
        d.last_online = date(reader.read_date_time_ticks()?);
        guild_rank_read = true;
    }
    if version >= 18 {
        d.solen_friendship = match reader.read_encoded_int()? {
            1 => SolenFriendship::Red,
            2 => SolenFriendship::Black,
            _ => SolenFriendship::None,
        };
    }
    if version >= 16 {
        // QuestSerializer.DeserializeQuest
        if reader.read_encoded_int()? == 1 {
            // QuestSerializer.DeserializeQuest: kept as the bytes it wrote, for the quest system.
            d.quest = quest_record::scan(reader)?;
        }
        let count = reader.read_encoded_int()?;
        for _ in 0..count.max(0) {
            let ty = match reader.read_encoded_int()? {
                1 => QuestTypeRef::Indexed(reader.read_encoded_int()?),
                2 => reader
                    .read_string()?
                    .map_or(QuestTypeRef::None, QuestTypeRef::Named),
                _ => QuestTypeRef::None,
            };
            let restart = if version < 17 {
                DateTime::MAX
            } else {
                date(reader.read_date_time_ticks()?)
            };
            d.done_quests.push((ty, restart));
        }
        d.profession = reader.read_encoded_int()?;
    }
    if (15..32).contains(&version) {
        tables::virtues_mut(w, id).last_compassion_loss =
            date(reader.read_delta_time_ticks(utc_now)?);
    }
    if (14..32).contains(&version) {
        let gains = reader.read_encoded_int()?; // virtues.CompassionGains
        tables::virtues_mut(w, id).compassion_gains = gains;
        if gains > 0 {
            tables::virtues_mut(w, id).next_compassion_day =
                date(reader.read_delta_time_ticks(utc_now)?);
        }
    }
    if version >= 12 {
        d.bob_filter = BobFilter::deserialize(reader)?;
    }
    if (11..13).contains(&version) {
        for item in reader.read_item_list(w)? {
            muo_server::item::ops::set_paid_insurance(w, item, true);
        }
    }
    if version >= 10 && reader.read_bool()? {
        d.hair_mod_id = reader.read_i32()?;
        d.hair_mod_hue = reader.read_i32()?;
        d.beard_mod_id = reader.read_i32()?;
        d.beard_mod_hue = reader.read_i32()?;
    }
    if version >= 9 {
        let span = TimeSpan::from_ticks(reader.read_time_span_ticks()?);
        // SavagePaintExpiration = value
        if let Some(t) = now.checked_add(span) {
            d.savage_paint_expiration = t;
        }
        savage_paint = remaining_from(now, d.savage_paint_expiration) > TimeSpan::ZERO;
    }
    if version >= 8 {
        d.npc_guild = npc_guild_from(reader.read_i32()?);
        d.npc_guild_join_time = date(reader.read_date_time_ticks()?);
        d.npc_guild_game_time = TimeSpan::from_ticks(reader.read_time_span_ticks()?);
    }
    if version >= 7 {
        d.perma_flags = reader.read_mobile_list(w)?;
    }
    if version >= 6 {
        let span = TimeSpan::from_ticks(reader.read_time_span_ticks()?);
        if let Some(t) = now.checked_add(span) {
            d.next_tailor_bulk_order = t;
        }
    }
    if version >= 5 {
        let span = TimeSpan::from_ticks(reader.read_time_span_ticks()?);
        if let Some(t) = now.checked_add(span) {
            d.next_smith_bulk_order = t;
        }
    }
    if (4..32).contains(&version) {
        tables::virtues_mut(w, id).last_justice_loss = date(reader.read_delta_time_ticks(utc_now)?);
        let protectors = reader.read_mobile_list(w)?; // always a list of 0, or 1
        if let Some(protector) = protectors.first() {
            tables::add_protection(w, *protector, id); // JusticeVirtue.AddProtection
        }
    }
    if (3..32).contains(&version) {
        let v = tables::virtues_mut(w, id);
        v.last_sacrifice_gain = date(reader.read_delta_time_ticks(utc_now)?);
        v.last_sacrifice_loss = date(reader.read_delta_time_ticks(utc_now)?);
        v.available_resurrects = reader.read_i32()?;
    }
    if version >= 2 {
        d.flags = PlayerFlag::from_bits_retain(reader.read_i32()?);
    }
    if version >= 1 {
        if version < 31 {
            let long_term = TimeSpan::from_ticks(reader.read_time_span_ticks()?);
            let short_term = TimeSpan::from_ticks(reader.read_time_span_ticks()?);
            tables::migrate_murder_context(w, id, short_term, long_term);
        }
        d.game_time = TimeSpan::from_ticks(reader.read_time_span_ticks()?);
    }

    // The profession is validated against the loaded table.
    if !hooks::verify_profession(w, d.profession) {
        d.profession = 0;
    }
    if !guild_rank_read {
        d.guild_rank = 1; // RankDefinition.Member: "going from older version to new version"
    }

    let (access, hidden, account) = {
        let m = w
            .mobile(id)
            .ok_or_else(|| invalid("mobile vanished while loading"))?;
        (m.access_level, m.hidden, m.account)
    };
    if d.last_online == DateTime::MIN
        && let Some(t) = account
            .and_then(|a| crate::accounting::account(w, a))
            .map(|a| a.last_login)
    {
        d.last_online = t;
    }
    let stabled: Vec<MobileId> = d.stabled.iter().flatten().copied().collect();
    if let Some(m) = w.mobile_mut(id) {
        m.ext = Ext::new(d);
        if access > AccessLevel::Player {
            m.ignore_mobiles = true;
        }
    }
    if savage_paint {
        // BodyMod/HueMod setters (delta, properties, stat timers), not field writes.
        let female = w.mobile(id).is_some_and(|m| m.female);
        mobile::part1::props_b::set_body_mod(
            w,
            id,
            muo_primitives::Body(if female { 184 } else { 183 }),
        );
        mobile::part1::props_b::set_hue_mod(w, id, 0);
    }
    for pet in stabled {
        hooks::pet_mark_stabled(w, pet, id); // `if (stabled is BaseCreature bc)`: the hook checks the class
    }
    if hidden {
        // Hiding is the only buff where it has an effect that's serialized.
        buff::add_buff(
            w,
            id,
            buff::BuffInfo::new(buff::BuffIcon::HidingAndOrStealth, 1075655),
        );
    }
    Ok(())
}

fn remaining_from(now: DateTime, until: DateTime) -> TimeSpan {
    let span = until - now;
    if span > TimeSpan::ZERO {
        span
    } else {
        TimeSpan::ZERO
    }
}

fn npc_guild_from(v: i32) -> NpcGuild {
    use NpcGuild::*;
    [
        None,
        MagesGuild,
        WarriorsGuild,
        ThievesGuild,
        RangersGuild,
        HealersGuild,
        MinersGuild,
        MerchantsGuild,
        TinkersGuild,
        TailorsGuild,
        FishermensGuild,
        BardsGuild,
        BlacksmithsGuild,
    ]
    .get(v as usize)
    .copied()
    .unwrap_or(None)
}

/// `Server.Mobiles.PlayerMobile`.
pub static PLAYER_MOBILE: MobileType = MobileType {
    full_name: "Server.Mobiles.PlayerMobile",
    parent: Some(&MOBILE_BASE),
    construct: None, // not [Constructible]; `new_player_mobile` is the constructor
    racial_skill_bonus,
    max_weight,
    armor_rating,
    luck,
    new_guild_display,
    murderer,
    set_paralyzed,
    str,
    int,
    dex,
    hits_max,
    stam_max,
    mana_max,
    on_skill_invalidated,
    get_max_resistance,
    get_min_resistance,
    on_race_change,
    on_net_state_changed,
    compute_base_light_levels,
    check_light_levels,
    on_mana_change,
    apply_name_suffix,
    get_logout_delay,
    compute_movement_speed,
    apply_poison,
    get_parent_move_result_for,
    get_inventory_move_result_for,
    check_shove,
    check_poison_immunity,
    on_poison_immunity,
    serialize,
    deserialize,
    toggle_flying: members::toggle_flying,
    on_dropped_item_to_world: members::on_dropped_item_to_world,
    delta: members::delta,
    revealing_action: members::revealing_action,
    on_hidden_changed: members::on_hidden_changed,
    on_sub_item_added: members::on_sub_item_added,
    can_be_harmful_full: members::can_be_harmful_full,
    can_be_beneficial_full: members::can_be_beneficial_full,
    check_context_menu_display: members::check_context_menu_display,
    on_item_added: members::on_item_added,
    on_item_removed: members::on_item_removed,
    move_dir: members::move_dir,
    check_movement: members::check_movement,
    allow_item_use: members::allow_item_use,
    allow_skill_use: members::allow_skill_use,
    move_to_world: members::move_to_world,
    set_location: members::set_location,
    get_context_menu_entries: members::get_context_menu_entries,
    disruptive_action: members::disruptive_action,
    on_double_click: members::on_double_click,
    display_paperdoll_to: members::display_paperdoll_to,
    check_equip: members::check_equip,
    check_trade: members::check_trade,
    check_nonlocal_drop: members::check_nonlocal_drop,
    on_location_change: members::on_location_change,
    on_move_over: members::on_move_over,
    on_map_change: members::on_map_change,
    on_beneficial_action: members::on_beneficial_action,
    on_damage: members::on_damage,
    damage: members::damage,
    resurrect: members::resurrect,
    on_before_death: members::on_before_death,
    on_death: members::on_death,
    mutate_speech: members::mutate_speech,
    do_speech: members::do_speech,
    is_harmful_criminal: members::is_harmful_criminal,
    can_see_mobile: members::can_see_mobile,
    can_see_item: members::can_see_item,
    on_after_delete: members::on_after_delete,
    get_properties: members::get_properties,
    on_single_click: members::on_single_click,
    on_move: members::on_move,
    on_movement: members::on_movement,
    on_delete: members::on_delete,
    on_speech: members::on_speech,
    on_kills_change,
    on_skill_change,
    on_access_level_changed,
    ..MOBILE_BASE
};

/// Registers `PlayerMobile` and the phase functions of the misc systems this module holds.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    use crate::misc::{
        light_cycle, name_verification, poison_kinds, profanity_protection, race_definitions, small,
    };
    use muo_server::type_registry::Phase;

    types.register_mobile(&PLAYER_MOBILE, &[]);

    types.register_phase_default(
        Phase::Configure,
        "RaceDefinitions.Configure",
        race_definitions::configure,
    );
    types.register_phase(
        Phase::Configure,
        "PoisonKinds.Configure",
        10,
        poison_kinds::configure,
    );
    types.register_phase(
        Phase::Configure,
        "RegenRates.Configure",
        10,
        crate::misc::regen_rates::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "LightCycle.Configure",
        light_cycle::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "NameVerification.Configure",
        name_verification::configure,
    );
    types.register_phase_default(
        Phase::Configure,
        "ProfanityProtection.Configure",
        profanity_protection::configure,
    );
    types.register_phase_default(
        Phase::Initialize,
        "Broadcasts.Initialize",
        crate::misc::broadcasts::initialize,
    );
    types.register_phase_default(Phase::Configure, "NameList.Configure", |w| {
        let base = w.core.base_directory.clone();
        if let Err(e) = small::configure_name_lists(w, &base) {
            // A failed configure stops startup
            w.core.startup_error = Some(format!("NameList.Configure: {e}"));
        }
    });
    types.register_phase_default(Phase::Configure, "BuffInfo.Configure", buff::configure);
    types.register_phase_default(
        Phase::Configure,
        "PlayerMurderSystem.Configure",
        tables::register_persistence,
    );
    types.register_phase_default(Phase::Configure, "PlayerMobile.Configure", |w| {
        // What BaseCreature, spells and skills ask of a PlayerMobile (crate::hooks::PetPlayerHooks).
        let h = w.resources.get_mut::<crate::hooks::PetPlayerHooks>();
        h.add_follower = Some(add_follower);
        h.remove_follower = Some(|w, pm, c| {
            remove_follower(w, pm, c);
            if let Some(d) = data_mut(w, pm) {
                d.auto_stabled.as_mut().map(|s| s.shift_remove(&c));
            }
        });
        h.learning = Some(|w, pm| {
            data(w, pm)
                .and_then(|d| d.learning)
                .map_or(-1, |s| s as i32)
        });
        h.set_learning = Some(|w, pm, v| {
            if let Some(d) = data_mut(w, pm) {
                d.learning = SkillName::from_i32(v);
            }
        });
        h.young = Some(young);
        h.all_followers = Some(|w, pm| {
            data(w, pm)
                .and_then(|d| d.all_followers.as_ref())
                .map_or_else(Vec::new, |f| f.iter().copied().collect())
        });
    });
    types.register_phase_default(Phase::Configure, "NotorietyHandlers.Configure", |w| {
        // What the notoriety handlers read of a player (crate::misc::notoriety::NotorietyHooks).
        let h = w
            .resources
            .get_mut::<crate::misc::notoriety::NotorietyHooks>();
        h.in_safe_zone = Some(hooks::in_safe_zone);
        h.faction = Some(|w, m| hooks::faction_of(w, m).map(|f| f as u32));
        // `CheckYoungProtection(from)` also sends the throttled message; the handlers take `&World`, so they
        // get the verdict alone (the message goes out from `members::check_young_protection`).
        h.check_young_protection = Some(members::young_protects_from);
    });
    types.register_phase_default(Phase::Configure, "AOS.Configure", |w| {
        // `AOS.Damage` for the creature and spell code (crate::hooks::ContentHooks::aos_damage).
        crate::hooks::hooks_mut(w).aos_damage = Some(aos_damage);
    });
    types.register_phase_default(
        Phase::Initialize,
        "VirtueSystem.Initialize",
        tables::apply_virtue_migrations,
    );
    types.register_phase_default(
        Phase::Configure,
        "StaminaSystem.Configure",
        crate::misc::stamina_system::configure,
    );

    types.register_phase_default(
        Phase::Initialize,
        "NotorietyHandlers.Initialize",
        crate::misc::notoriety::initialize,
    );
    types.register_phase_default(
        Phase::Initialize,
        "StaminaSystem.Initialize",
        crate::misc::stamina_system::initialize,
    );
    types.register_phase_default(Phase::Initialize, "PlayerMobile.Initialize", initialize);
    types.register_phase_default(
        Phase::Initialize,
        "Weather.Initialize",
        crate::misc::weather::initialize,
    );
    types.register_phase_default(
        Phase::Initialize,
        "LightCycle.Initialize",
        light_cycle::initialize,
    );
    types.register_phase_default(
        Phase::Initialize,
        "AttackMessage.Initialize",
        small::attack_message_initialize,
    );
    types.register_phase_default(
        Phase::Initialize,
        "FoodDecayTimer.Initialize",
        small::food_decay_initialize,
    );
}

/// `ContentHooks::aos_damage`: `AOS.Damage(m, from, amount, ignoreArmor, phys, fire, cold, pois, nrgy, chaos,
/// direct)`.
fn aos_damage(
    w: &mut World,
    m: MobileId,
    from: Option<MobileId>,
    d: crate::hooks::AosDamage,
) -> i32 {
    use crate::misc::aos_helpers::{DamageOptions, DamageSplit, damage};
    damage(
        w,
        m,
        from,
        d.amount,
        DamageSplit {
            phys: d.phys,
            fire: d.fire,
            cold: d.cold,
            pois: d.pois,
            nrgy: d.nrgy,
        },
        DamageOptions {
            ignore_armor: d.ignore_armor,
            chaos: d.chaos,
            direct: d.direct,
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};

    #[test]
    fn faction_suffix_is_bracketed_once() {
        let mut w = World::new();
        let id = new_player_mobile(&mut w);
        let h = w.resources.get_mut::<hooks::SystemHooks>();
        h.faction_facet = Some(|_| Some(muo_server::map::MapId::FELUCCA));
        h.faction_abbreviation = Some(|_, _| Some("[CoM]".to_owned()));
        w.mobile_mut(id).unwrap().map = Some(muo_server::map::MapId::FELUCCA);
        assert_eq!(apply_name_suffix(&w, id, ""), "[CoM]");
    }

    #[test]
    fn direction4_quadrants() {
        let p = |x, y| Point3D { x, y, z: 0 };
        assert_eq!(get_direction4(p(10, 10), p(0, 0)), Direction::WEST);
        assert_eq!(get_direction4(p(0, 10), p(10, 0)), Direction::NORTH);
    }

    #[test]
    fn bob_filter_round_trips_and_matches_generator_bytes() {
        let f = BobFilter {
            ty: 3,
            quality: 0,
            material: 7,
            quantity: 0,
        };
        let mut out = SaveWriter::new(true);
        f.serialize(&mut out);
        let bytes = out.into_vec();
        // version 2, flags Type|Material (0b101), then the two ints
        assert_eq!(bytes, [2, 5, 3, 0, 0, 0, 7, 0, 0, 0]);
        let mut r = SaveReader::new(&bytes);
        assert_eq!(BobFilter::deserialize(&mut r).unwrap(), f);
    }

    #[test]
    fn save_round_trips_and_has_the_expected_layout() {
        let mut w = World::new();
        let id = new_player_mobile(&mut w);
        {
            let d = data_mut(&mut w, id).unwrap();
            d.profession = 4;
            d.guild_message_hue = 9;
            d.flags = PlayerFlag::YOUNG | PlayerFlag::from_bits_retain(0x10000);
            d.acquired_recipes = Some([5, 9].into_iter().collect());
            d.game_time = TimeSpan::from_mins(3);
            d.hair_mod_id = 0x203B;
            d.hair_mod_hue = 12;
        }
        let mut out = SaveWriter::new(true);
        serialize(&w, id, &mut out);
        let bytes = out.into_vec();

        let mut w2 = World::new();
        let id2 = new_player_mobile(&mut w2);
        let mut r = SaveReader::new(&bytes);
        deserialize(&mut w2, id2, &mut r).unwrap();
        let (a, b) = (data(&w, id).unwrap(), data(&w2, id2).unwrap());
        assert_eq!(
            (a.profession, a.guild_message_hue, a.flags),
            (b.profession, b.guild_message_hue, b.flags)
        );
        assert_eq!(a.acquired_recipes, b.acquired_recipes);
        assert_eq!(
            (a.hair_mod_id, a.hair_mod_hue, a.beard_mod_id),
            (b.hair_mod_id, b.hair_mod_hue, b.beard_mod_id)
        );
        assert_eq!(a.game_time, b.game_time);

        let mut out2 = SaveWriter::new(true);
        serialize(&w2, id2, &mut out2);
        assert_eq!(bytes, out2.into_vec());
    }

    /// A version-30 `PlayerMobile` record after the base `Mobile` part, with every legacy field set.
    fn legacy_record(w: &World, id: MobileId, quest: &[u8]) -> Vec<u8> {
        let mut out = SaveWriter::new(true);
        (MOBILE_BASE.serialize)(w, id, &mut out);
        out.write_i32(30); // version
        out.write_i32(0); // Stabled (an empty set)
        out.write_bool(false); // stuck menu uses
        out.write_date_time_ticks(0); // PeacedUntil
        out.write_date_time_ticks(0); // AnkhNextUse
        out.write_i32(0); // AutoStabled
        out.write_i32(0); // recipes
        out.write_i64(i64::MIN); // LastHonorLoss
        // ChampionTitles: harrower 7, two titles (Abyss 11, Arachnid 22)
        out.write_encoded_int(7);
        out.write_encoded_int(2);
        for (value, ticks) in [(11, 1_000_000_000i64), (22, 2_000_000_000)] {
            out.write_encoded_int(0);
            out.write_encoded_int(value);
            out.write_date_time_ticks(ticks);
        }
        out.write_date_time_ticks(3_000_000_000); // virtues.LastValorLoss
        out.write_encoded_int(5); // ToTItemsTurnedIn
        out.write_i32(6);
        out.write_encoded_int(1); // AllianceMessageHue
        out.write_encoded_int(2); // GuildMessageHue
        out.write_encoded_int(3); // rank
        out.write_date_time_ticks(0); // LastOnline
        out.write_encoded_int(0); // SolenFriendship
        if quest.is_empty() {
            out.write_encoded_int(0);
        } else {
            out.write_encoded_int(1);
            out.write_bytes(quest);
        }
        out.write_encoded_int(0); // DoneQuests
        out.write_encoded_int(4); // Profession
        out.write_i64(i64::MIN); // virtues.LastCompassionLoss
        out.write_encoded_int(2); // CompassionGains
        out.write_i64(i64::MAX); // virtues.NextCompassionDay
        BobFilter::default().serialize(&mut out);
        out.write_bool(false); // hair mods
        out.write_time_span_ticks(0); // SavagePaintExpiration
        out.write_i32(0); // NpcGuild
        out.write_date_time_ticks(0);
        out.write_time_span_ticks(0);
        out.write_i32(0); // PermaFlags
        out.write_time_span_ticks(0); // tailor
        out.write_time_span_ticks(0); // smith
        out.write_i64(i64::MIN); // virtues.LastJusticeLoss
        out.write_i32(0); // protectors
        out.write_i64(i64::MIN); // virtues.LastSacrificeGain
        out.write_i64(i64::MIN); // virtues.LastSacrificeLoss
        out.write_i32(3); // virtues.AvailableResurrects
        out.write_i32(PlayerFlag::YOUNG.bits()); // Flags
        out.write_time_span_ticks(900_000_000_000); // longTermElapse
        out.write_time_span_ticks(100_000_000_000); // shortTermElapse
        out.write_time_span_ticks(5_000_000_000); // GameTime
        out.into_vec()
    }

    #[test]
    fn set_hair_mods_swaps_and_restores_the_style_through_the_setters() {
        let mut w = World::new();
        let id = new_player_mobile(&mut w);
        muo_server::mobile::part1::props_b::set_hair_item_id(&mut w, id, 0x203B);
        muo_server::mobile::part1::props_b::set_hair_hue(&mut w, id, 0x44);
        set_hair_mods(&mut w, id, 0x2044, -2);
        let m = w.mobile(id).unwrap();
        assert_eq!((m.hair_item_id, m.hair_hue), (0x2044, 0));
        set_hair_mods(&mut w, id, -1, -2);
        let m = w.mobile(id).unwrap();
        assert_eq!((m.hair_item_id, m.hair_hue), (0x203B, 0x44));
    }

    #[test]
    fn a_legacy_record_moves_its_data_to_the_owning_tables() {
        let mut w = World::new();
        let id = new_player_mobile(&mut w);
        let bytes = legacy_record(&w, id, &[]);
        let mut w2 = World::new();
        let id2 = new_player_mobile(&mut w2);
        deserialize(&mut w2, id2, &mut SaveReader::new(&bytes)).unwrap();

        let c = &w2.resources.get::<tables::ChampionTitleTable>().unwrap().0[&id2];
        assert_eq!(c.harrower, 7);
        assert_eq!(c.titles[0].unwrap().value, 11);
        assert_eq!(c.titles[1].unwrap().value, 22);
        let v = &w2.resources.get::<tables::VirtueTable>().unwrap().0[&id2];
        assert_eq!((v.available_resurrects, v.compassion_gains), (3, 2));
        assert_eq!(v.last_valor_loss.ticks(), 3_000_000_000);
        let m = &w2.resources.get::<tables::MurderTable>().unwrap().0[&id2];
        assert_eq!(m.short_term_elapse.ticks(), 100_000_000_000);
        assert_eq!(m.long_term_elapse.ticks(), 900_000_000_000);
        let d = data(&w2, id2).unwrap();
        assert_eq!(
            (d.profession, d.tot_items_turned_in, d.guild_rank),
            (4, 5, 3)
        );
        assert_eq!(d.game_time.ticks(), 5_000_000_000);
        assert!(d.flag(PlayerFlag::YOUNG));
    }

    #[test]
    fn a_quest_in_progress_survives_a_resave() {
        // The Summoning, version 0, no objectives or conversations; child: version, a serial, a bool.
        let quest = [1, 0, 0, 0, 0, 0, 7, 0, 0, 0, 1];
        let mut w = World::new();
        let id = new_player_mobile(&mut w);
        let bytes = legacy_record(&w, id, &quest);
        let mut w2 = World::new();
        let id2 = new_player_mobile(&mut w2);
        deserialize(&mut w2, id2, &mut SaveReader::new(&bytes)).unwrap();
        assert_eq!(data(&w2, id2).unwrap().quest.as_deref(), Some(&quest[..]));
        assert_eq!(data(&w2, id2).unwrap().profession, 4); // the fields after the quest were found
        let mut out = SaveWriter::new(true);
        serialize(&w2, id2, &mut out);
        let resaved = out.into_vec();
        let mut w3 = World::new();
        let id3 = new_player_mobile(&mut w3);
        deserialize(&mut w3, id3, &mut SaveReader::new(&resaved)).unwrap();
        assert_eq!(data(&w3, id3).unwrap().quest.as_deref(), Some(&quest[..]));
    }

    #[test]
    fn caps_and_weights() {
        let mut w = World::new();
        let id = new_player_mobile(&mut w);
        w.mobile_mut(id).unwrap().str = 200;
        assert_eq!(
            (PLAYER_MOBILE.str)(&w, id),
            150,
            "a player is capped at 150 in ML"
        );
        assert_eq!((PLAYER_MOBILE.max_weight)(&w, id), 100 + 525);
        w.mobile_mut(id).unwrap().access_level = AccessLevel::GameMaster;
        assert_eq!((PLAYER_MOBILE.str)(&w, id), 200);
    }
}
