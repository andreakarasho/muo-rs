//! `Deserialize`, byte for byte. (`Serialize` is `part1::lifecycle::serialize`.)
//!
//! The versions fall through from newest to oldest; each block below runs when the version is at least the
//! lowest case label of that block, in descending order. A version outside 0..=38 matches no case and reads
//! nothing.

use indexmap::IndexSet;
use muo_io::{GenericReader, Result};
use muo_primitives::{Body, DateTime, Direction};

use super::{migrations, stats};
use crate::guild::GuildId;
use crate::ids::{ItemId, MobileId};
use crate::mobile::AccessLevel;
use crate::mobile::part1::props_a;
use crate::serialization_ext::ReadEntityExt;
use crate::skills::{SkillTable, Skills};
use crate::world::World;

fn date(ticks: i64) -> DateTime {
    DateTime::try_from_ticks(ticks).unwrap_or(DateTime::MIN)
}

/// `reader.ReadEntity<BaseGuild>()`: the guild registry (filled by the guild persistence's indexes, which load
/// before any mobile's body).
fn read_guild(w: &World, r: &mut dyn GenericReader) -> Result<Option<GuildId>> {
    r.read_guild(w)
}

macro_rules! me {
    ($w:expr, $id:expr) => {
        match $w.mobile_mut($id) {
            Some(m) => m,
            None => return Ok(()),
        }
    };
}

/// `Deserialize` (the base behavior of the virtual).
pub fn deserialize(w: &mut World, id: MobileId, reader: &mut dyn GenericReader) -> Result<()> {
    let version = reader.read_i32()?;
    if !(0..=38).contains(&version) {
        return Ok(());
    }
    let utc_now = DateTime::utc_now().ticks();

    // 38..31
    if version >= 31 {
        // 38: stat-gain stamps moved from delta time to anchored time; 37: hair inline; 36: virtues moved to
        // VirtueSystem; 35: short term murders moved to PlayerMurderSystem; 34: Stabled moved to PlayerMobile;
        // 33: removed Created; 32: removed StuckMenu.
        let (str_gain, int_gain, dex_gain) = if version >= 38 {
            (
                reader.read_anchored_time_ticks()?,
                reader.read_anchored_time_ticks()?,
                reader.read_anchored_time_ticks()?,
            )
        } else {
            (
                reader.read_delta_time_ticks(utc_now)?,
                reader.read_delta_time_ticks(utc_now)?,
                reader.read_delta_time_ticks(utc_now)?,
            )
        };
        let m = me!(w, id);
        m.last_str_gain = date(str_gain);
        m.last_int_gain = date(int_gain);
        m.last_dex_gain = date(dex_gain);
    }

    // 30
    if version >= 30 {
        // Before v37 each hair was a VirtualHairInfo whose Serialize wrote a leading version int ahead of the
        // item id and hue.
        let hairflag = reader.read_u8()?;
        if hairflag & 0x01 != 0 {
            if version < 37 {
                reader.read_i32()?; // legacy VirtualHairInfo version
            }
            let (item_id, hue) = (reader.read_i32()?, reader.read_i32()?);
            let m = me!(w, id);
            m.hair_item_id = item_id;
            m.hair_hue = hue;
        }
        if hairflag & 0x02 != 0 {
            if version < 37 {
                reader.read_i32()?; // legacy VirtualHairInfo version
            }
            let (item_id, hue) = (reader.read_i32()?, reader.read_i32()?);
            let m = me!(w, id);
            m.facial_hair_item_id = item_id;
            m.facial_hair_hue = hue;
        }
    }

    // 29
    if version >= 29 {
        let race = reader.read_race()?;
        me!(w, id).race = race;
    }

    // 28
    if (28..=30).contains(&version) {
        let t = reader.read_delta_time_ticks(utc_now)?;
        crate::mobile::part1::props_b::set_last_stat_gain(w, id, date(t));
    }

    // 27
    if version >= 27 {
        me!(w, id).tithing_points = reader.read_i32()?;
    }

    // 26, 25, 24
    if version >= 24 {
        let corpse = reader.read_item(w)?;
        me!(w, id).corpse = corpse;
    }

    // 23
    if (23..33).contains(&version) {
        let created = reader.read_date_time_ticks()?;
        me!(w, id).created = date(created);
    }

    // 22, 21
    if (21..34).contains(&version) {
        // Migrated to PlayerMobile
        let stabled: IndexSet<MobileId> = reader.read_mobile_set(w)?;
        migrations::add_to_stabled_migration(w, id, stabled);
    }

    // 20
    if version >= 20 {
        me!(w, id).cant_walk = reader.read_bool()?;
    }

    // 19, 18
    if (18..36).contains(&version) {
        reader.read_u8()?; // VirtueInfo version
        let mask = reader.read_u8()?;
        if mask != 0 {
            let mut values = [0i32; 8];
            for (i, v) in values.iter_mut().enumerate() {
                if mask & (1 << i) != 0 {
                    *v = reader.read_i32()?;
                }
            }
            migrations::add_to_virtue_migration(w, id, values);
        }
    }

    // 17
    if version >= 17 {
        let (thirst, bac) = (reader.read_i32()?, reader.read_i32()?);
        props_a::set_thirst(w, id, thirst);
        me!(w, id).bac = bac;
    }

    // 16
    if version >= 16 {
        if version < 35 {
            // Migrated to PlayerMurderSystem
            let murders = reader.read_i32()?;
            migrations::add_to_murder_migration(w, id, murders);
        }
        if version <= 24 {
            reader.read_date_time_ticks()?;
            reader.read_date_time_ticks()?;
        }
    }

    // 15
    if version >= 15 {
        if version < 22 {
            reader.read_i32()?; // followers
        }
        me!(w, id).followers_max = reader.read_i32()?;
    }

    // 14
    if version >= 14 {
        me!(w, id).magic_damage_absorb = reader.read_i32()?;
    }

    // 13
    if version >= 13 {
        let fealty = reader.read_mobile(w)?;
        me!(w, id).guild_fealty = fealty;
    }

    // 12
    if version >= 12 {
        let guild = read_guild(w, reader)?;
        me!(w, id).guild = guild;
    }

    // 11
    if version >= 11 {
        me!(w, id).display_guild_title = reader.read_bool()?;
    }

    // 10
    if version >= 10 {
        me!(w, id).can_swim = reader.read_bool()?;
    }

    // 9
    if version >= 9 {
        me!(w, id).squelched = reader.read_bool()?;
    }

    // 8
    if version >= 8 {
        let holding: Option<ItemId> = reader.read_item(w)?;
        me!(w, id).holding = holding;
    }

    // 7
    if version >= 7 {
        me!(w, id).virtual_armor = reader.read_i32()?;
    }

    // 6
    if version >= 6 {
        me!(w, id).base_sound_id = reader.read_i32()?;
    }

    // 5
    if version >= 5 {
        let (disarm, stun) = (reader.read_bool()?, reader.read_bool()?);
        let m = me!(w, id);
        m.disarm_ready = disarm;
        m.stun_ready = stun;
    }

    // 4
    if (4..=25).contains(&version) {
        reader.read_poison()?;
    }

    // 3
    if version >= 3 {
        me!(w, id).stat_cap = reader.read_i32()?;
    }

    // 2
    if version >= 2 {
        me!(w, id).name_hue = reader.read_i32()?;
    }

    // 1
    if version >= 1 {
        me!(w, id).hunger = reader.read_i32()?;
    }

    // 0
    {
        {
            let m = me!(w, id);
            if version < 11 {
                m.display_guild_title = true;
            }
            if version < 3 {
                m.stat_cap = 225;
            }
            if version < 15 {
                m.followers = 0;
                m.followers_max = 5;
            }
        }

        let location = reader.read_point3d()?;
        let body = reader.read_i32()?;
        let name = reader.read_string()?;
        let guild_title = reader.read_string()?;
        let criminal = reader.read_bool()?;
        let kills = reader.read_i32()?;
        let speech_hue = reader.read_i32()?;
        let emote_hue = reader.read_i32()?;
        let whisper_hue = reader.read_i32()?;
        let yell_hue = reader.read_i32()?;
        let language = reader.read_string()?;
        let female = reader.read_bool()?;
        let warmode = reader.read_bool()?;
        let hidden = reader.read_bool()?;
        let direction = reader.read_u8()?;
        let hue = reader.read_i32()?;
        let (str, dex, int) = (reader.read_i32()?, reader.read_i32()?, reader.read_i32()?);
        let (hits, stam, mana) = (reader.read_i32()?, reader.read_i32()?, reader.read_i32()?);
        let map = reader.read_map()?;
        let blessed = reader.read_bool()?;
        let fame = reader.read_i32()?;
        let karma = reader.read_i32()?;
        let access_level = reader.read_u8()?;
        {
            let m = me!(w, id);
            m.location = location;
            m.body = Body::new(body);
            m.name = name;
            m.guild_title = guild_title;
            m.criminal = criminal;
            m.kills = kills;
            m.speech_hue = speech_hue;
            m.emote_hue = emote_hue;
            m.whisper_hue = whisper_hue;
            m.yell_hue = yell_hue;
            m.language = language;
            m.female = female;
            m.warmode = warmode;
            m.hidden = hidden;
            m.direction = Direction::from_bits_retain(direction);
            m.hue = hue;
            m.str = str;
            m.dex = dex;
            m.int = int;
            m.hits = hits;
            m.stam = stam;
            m.mana = mana;
            m.map = map;
            m.blessed = blessed;
            m.fame = fame;
            m.karma = karma;
            // shortcut: a byte without an AccessLevel name reads as Player.
            m.access_level =
                AccessLevel::from_i32(i32::from(access_level)).unwrap_or(AccessLevel::Player);
        }

        let table_len = w.resources.get::<SkillTable>().map_or(0, |t| t.info.len());
        let skills = Skills::read(reader, table_len)?;
        let item_list = reader.read_item_list(w)?;
        let player = reader.read_bool()?;
        let title = reader.read_string()?;
        let profile = reader.read_string()?;
        let profile_locked = reader.read_bool()?;
        {
            let m = me!(w, id);
            m.skills = Some(skills);
            m.items = item_list;
            m.player = player;
            m.title = title;
            m.profile = profile;
            m.profile_locked = profile_locked;
        }

        if version <= 18 {
            reader.read_i32()?;
            reader.read_i32()?;
            reader.read_i32()?;
        }

        let auto_page_notify = reader.read_bool()?;
        let logout_location = reader.read_point3d()?;
        let logout_map = reader.read_map()?;
        let (str_lock, dex_lock, int_lock) =
            (reader.read_u8()?, reader.read_u8()?, reader.read_u8()?);
        {
            let m = me!(w, id);
            m.auto_page_notify = auto_page_notify;
            m.logout_location = logout_location;
            m.logout_map = logout_map;
            m.str_lock = stats::stat_lock_from_u8(str_lock);
            m.dex_lock = stats::stat_lock_from_u8(dex_lock);
            m.int_lock = stats::stat_lock_from_u8(int_lock);
        }

        if version < 32 && reader.read_bool()? {
            let count = reader.read_i32()?;
            for _ in 0..count.max(0) {
                reader.read_date_time_ticks()?;
            }
        }

        {
            let m = me!(w, id);
            if m.player && m.map != Some(crate::map::MapId::INTERNAL) {
                m.logout_location = m.location;
                m.logout_map = m.map;
                m.map = Some(crate::map::MapId::INTERNAL);
            }
        }

        let (map, loc, criminal, ty) = {
            let m = me!(w, id);
            (m.map, m.location, m.criminal, m.ty)
        };
        if let Some(map) = map {
            crate::map::sector::on_enter_mobile(w, map, loc, id);
        }

        if criminal {
            stats::start_expire_criminal_timer(w, id);
        }

        if (ty.should_check_stat_timers)(w, id) {
            (ty.check_stat_timers)(w, id);
        }

        super::effects::update_region(w, id);

        (ty.update_resistances)(w, id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use muo_io::serialization::{SaveReader, SaveWriter};
    use muo_primitives::Point3D;

    use super::*;
    use crate::mobile::part1::lifecycle::serialize;
    use crate::mobile::{MOBILE_BASE, StatLockType};

    /// Serialize with part1's writer, read back with this reader into a fresh mobile: every field survives.
    #[test]
    fn round_trips_through_serialize() {
        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        {
            let m = w.mobile_mut(id).unwrap();
            m.name = Some("Bob".into());
            m.title = Some("the Brave".into());
            m.language = Some("ENU".into());
            m.location = Point3D::new(1, 2, 3);
            m.map = Some(crate::map::MapId(1));
            m.body = Body(400);
            m.direction = Direction::EAST;
            m.hair_item_id = 0x203B;
            m.hair_hue = 7;
            m.facial_hair_item_id = 0x204D;
            m.facial_hair_hue = 9;
            m.str = 50;
            m.dex = 40;
            m.int = 30;
            m.hits = 20;
            m.stam = 21;
            m.mana = 22;
            m.str_lock = StatLockType::Locked;
            m.dex_lock = StatLockType::Down;
            m.fame = 100;
            m.karma = -100;
            m.kills = 3;
            m.access_level = AccessLevel::Seer;
            m.followers_max = 7;
            m.stat_cap = 240;
            m.hunger = 11;
            m.thirst = 12;
            m.bac = 2;
            m.name_hue = 55;
            m.speech_hue = 1;
            m.emote_hue = 2;
            m.whisper_hue = 3;
            m.yell_hue = 4;
            m.female = true;
            m.hidden = true;
            m.warmode = true;
            m.blessed = true;
            m.display_guild_title = true;
            m.can_swim = true;
            m.cant_walk = true;
            m.squelched = true;
            m.virtual_armor = 5;
            m.base_sound_id = 0x1B0;
            m.magic_damage_absorb = 4;
            m.tithing_points = 9;
            m.profile = Some("hello".into());
            m.profile_locked = true;
            m.auto_page_notify = false;
            m.logout_location = Point3D::new(4, 5, 6);
            m.logout_map = Some(crate::map::MapId(2));
            m.disarm_ready = true;
        }
        let mut out = SaveWriter::new(true);
        serialize(&w, id, &mut out);
        let bytes = out.into_vec();

        let mut w2 = World::new();
        let id2 = w2.new_mobile(&MOBILE_BASE);
        let mut r = SaveReader::new(&bytes);
        deserialize(&mut w2, id2, &mut r).unwrap();
        assert_eq!(r.remaining(), 0, "reader consumed the whole record");

        let (a, b) = (w.mobile(id).unwrap(), w2.mobile(id2).unwrap());
        assert_eq!(a.name, b.name);
        assert_eq!(a.title, b.title);
        assert_eq!(a.language, b.language);
        assert_eq!(a.location, b.location);
        assert_eq!(a.map, b.map);
        assert_eq!(a.body, b.body);
        assert_eq!(a.direction, b.direction);
        assert_eq!((a.hair_item_id, a.hair_hue), (b.hair_item_id, b.hair_hue));
        assert_eq!(
            (a.facial_hair_item_id, a.facial_hair_hue),
            (b.facial_hair_item_id, b.facial_hair_hue)
        );
        assert_eq!((a.str, a.dex, a.int), (b.str, b.dex, b.int));
        assert_eq!((a.hits, a.stam, a.mana), (b.hits, b.stam, b.mana));
        assert_eq!(
            (a.str_lock, a.dex_lock, a.int_lock),
            (b.str_lock, b.dex_lock, b.int_lock)
        );
        assert_eq!((a.fame, a.karma, a.kills), (b.fame, b.karma, b.kills));
        assert_eq!(a.access_level, b.access_level);
        assert_eq!((a.followers_max, a.stat_cap), (b.followers_max, b.stat_cap));
        assert_eq!((a.hunger, a.thirst, a.bac), (b.hunger, b.thirst, b.bac));
        assert_eq!(a.name_hue, b.name_hue);
        assert_eq!(
            (a.speech_hue, a.emote_hue, a.whisper_hue, a.yell_hue),
            (b.speech_hue, b.emote_hue, b.whisper_hue, b.yell_hue)
        );
        assert_eq!(
            (a.female, a.hidden, a.warmode, a.blessed),
            (b.female, b.hidden, b.warmode, b.blessed)
        );
        assert_eq!(
            (a.display_guild_title, a.can_swim, a.cant_walk, a.squelched),
            (b.display_guild_title, b.can_swim, b.cant_walk, b.squelched)
        );
        assert_eq!(
            (a.virtual_armor, a.base_sound_id),
            (b.virtual_armor, b.base_sound_id)
        );
        assert_eq!(
            (a.magic_damage_absorb, a.tithing_points),
            (b.magic_damage_absorb, b.tithing_points)
        );
        assert_eq!(
            (&a.profile, a.profile_locked),
            (&b.profile, b.profile_locked)
        );
        assert_eq!(a.auto_page_notify, b.auto_page_notify);
        assert_eq!(
            (a.logout_location, a.logout_map),
            (b.logout_location, b.logout_map)
        );
        assert!(b.disarm_ready && !b.stun_ready);
    }

    /// An old record: version 0 has only the base block.
    #[test]
    fn version_zero_applies_the_defaults() {
        use muo_io::GenericWriter;
        let mut out = SaveWriter::new(true);
        out.write_i32(0);
        out.write_point3d(Point3D::new(5, 6, 7));
        out.write_i32(401);
        out.write_opt_string(Some("Ann"));
        out.write_opt_string(None);
        out.write_bool(false); // criminal
        for _ in 0..5 {
            out.write_i32(0); // kills, speech, emote, whisper, yell
        }
        out.write_opt_string(None); // language
        out.write_bool(false); // female
        out.write_bool(false); // warmode
        out.write_bool(false); // hidden
        out.write_u8(0); // direction
        for _ in 0..7 {
            out.write_i32(0); // hue, str, dex, int, hits, stam, mana
        }
        out.write_map_index(Some(0x7F)); // map
        out.write_bool(false); // blessed
        out.write_i32(0); // fame
        out.write_i32(0); // karma
        out.write_u8(0); // access level
        // Skills: version 3, cap, no skills.
        out.write_i32(3);
        out.write_i32(7000);
        out.write_i32(0);
        out.write_i32(0); // items
        out.write_bool(false); // player
        out.write_opt_string(None); // title
        out.write_opt_string(None); // profile
        out.write_bool(false); // profile locked
        for _ in 0..3 {
            out.write_i32(0); // version <= 18: three unused ints
        }
        out.write_bool(true); // auto page notify
        out.write_point3d(Point3D::ZERO);
        out.write_map_index(None);
        out.write_u8(0);
        out.write_u8(0);
        out.write_u8(0);
        out.write_bool(false); // the version < 32 StuckMenu flag
        let bytes = out.into_vec();

        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        let mut r = SaveReader::new(&bytes);
        deserialize(&mut w, id, &mut r).unwrap();
        assert_eq!(r.remaining(), 0);
        let m = w.mobile(id).unwrap();
        assert_eq!((m.stat_cap, m.followers_max, m.followers), (225, 5, 0));
        assert!(m.display_guild_title);
        assert_eq!(m.name.as_deref(), Some("Ann"));
        assert_eq!(m.body, Body(401));
    }

    #[test]
    fn an_unknown_version_reads_nothing_more() {
        use muo_io::GenericWriter;
        let mut out = SaveWriter::new(true);
        out.write_i32(99);
        out.write_i32(1234);
        let bytes = out.into_vec();
        let mut w = World::new();
        let id = w.new_mobile(&MOBILE_BASE);
        let mut r = SaveReader::new(&bytes);
        deserialize(&mut w, id, &mut r).unwrap();
        assert_eq!(r.remaining(), 4);
    }
}
