//! The per-player tables of the engines a `PlayerMobile` record migrates into: `PlayerMurderSystem` (MurderContext),
//! `VirtueSystem` (VirtueContext) and `ChampionTitleSystem` (ChampionTitleContext). Their behavior lives elsewhere;
//! the data, its persistence files (`PlayerMurders`, `Virtues`, `ChampionTitles`) and the migrations
//! `PlayerMobile.Deserialize` runs for versions below 31, 32 and 33 live here, so that loading an old save and
//! saving again keeps what the old record carried. The layouts are the serialization generator's
//! (`Server.Engines.*.Serialization`).

use indexmap::IndexMap;
use muo_io::{GenericReader, GenericWriter, IoError, Result};
use muo_primitives::{DateTime, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::persistence::{GenericPayload, GenericPersistence};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};

fn date(t: i64) -> DateTime {
    DateTime::from_ticks(t.clamp(0, muo_primitives::datetime::MAX_TICKS))
}

// ---- PlayerMurderSystem ----

/// `MurderContext`.
#[derive(Clone, Debug, Default)]
pub struct MurderContext {
    pub short_term_elapse: TimeSpan,
    pub long_term_elapse: TimeSpan,
    pub short_term_murders: i32,
    pub ping_pongs: i32,
    pub bounty: i32,
    pub last_murder_time: DateTime,
}

impl MurderContext {
    fn serialize(&self, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(2);
        writer.write_time_span_ticks(self.short_term_elapse.ticks());
        writer.write_time_span_ticks(self.long_term_elapse.ticks());
        writer.write_i32(self.short_term_murders);
        writer.write_i32(self.ping_pongs);
        writer.write_i32(self.bounty);
        writer.write_date_time_ticks(self.last_murder_time.ticks());
    }

    /// `Deserialize` with the V0 and V1 migrations (`kills` is the player's `Kills`, `now` is `Core.Now`).
    fn deserialize(
        reader: &mut dyn GenericReader,
        kills: i32,
        now: DateTime,
    ) -> Result<MurderContext> {
        let version = reader.read_encoded_int()?;
        let mut c = MurderContext::default();
        if !(0..3).contains(&version) {
            return Err(IoError::InvalidData(format!(
                "MurderContext: version {version}"
            )));
        }
        c.short_term_elapse = TimeSpan::from_ticks(reader.read_time_span_ticks()?);
        c.long_term_elapse = TimeSpan::from_ticks(reader.read_time_span_ticks()?);
        c.short_term_murders = reader.read_i32()?;
        match version {
            0 => c.ping_pongs = i32::from(kills >= 5), // players already at >= 5 kills crossed the threshold
            1 => {
                c.ping_pongs = reader.read_i32()?;
                c.last_murder_time = now;
            }
            _ => {
                c.ping_pongs = reader.read_i32()?;
                c.bounty = reader.read_i32()?;
                c.last_murder_time = date(reader.read_date_time_ticks()?);
            }
        }
        Ok(c)
    }
}

/// `PlayerMurderSystem._murderContexts`.
#[derive(Default)]
pub struct MurderTable(pub IndexMap<MobileId, MurderContext>);

struct MurderPayload;

impl GenericPayload for MurderPayload {
    fn name(&self) -> &str {
        "PlayerMurders"
    }

    fn serialize(&mut self, w: &World, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(0);
        let empty = MurderTable::default();
        let t = w.resources.get::<MurderTable>().unwrap_or(&empty);
        writer.write_encoded_int(t.0.len() as i32);
        for (m, c) in &t.0 {
            writer.write_mobile(w, Some(*m));
            c.serialize(writer);
        }
    }

    fn deserialize(&mut self, w: &mut World, reader: &mut dyn GenericReader) -> Result<()> {
        reader.read_encoded_int()?;
        let count = reader.read_encoded_int()?;
        for _ in 0..count.max(0) {
            let player = reader.read_mobile(w)?;
            let kills = player.and_then(|p| w.mobile(p)).map_or(0, |m| m.kills);
            let c = MurderContext::deserialize(reader, kills, w.now)?;
            if let Some(p) = player {
                w.resources.get_mut::<MurderTable>().0.insert(p, c);
            }
        }
        Ok(())
    }
}

/// `PlayerMurderSystem.MigrateContext(player, shortTerm, longTerm)`, from a record below version 31.
pub fn migrate_murder_context(
    w: &mut World,
    player: MobileId,
    short_term: TimeSpan,
    long_term: TimeSpan,
) {
    let short_term_murders = w
        .resources
        .get::<muo_server::mobile::part2::statics::Part2Statics>()
        .and_then(|s| s.murder_migrations.as_ref())
        .and_then(|m| m.get(&player).copied());
    let c = w
        .resources
        .get_mut::<MurderTable>()
        .0
        .entry(player)
        .or_default();
    if let Some(n) = short_term_murders {
        c.short_term_murders = n.max(0);
    }
    c.short_term_elapse = short_term;
    c.long_term_elapse = long_term;
}

// ---- VirtueSystem ----

/// `VirtueContext`. The `Option`s are the save flags as loaded; `None` writes nothing, like an unset flag.
#[derive(Clone, Debug, Default)]
pub struct VirtueContext {
    pub last_sacrifice_gain: DateTime,
    pub last_sacrifice_loss: DateTime,
    pub available_resurrects: i32,
    pub last_justice_loss: DateTime,
    pub last_compassion_loss: DateTime,
    pub next_compassion_day: DateTime,
    pub compassion_gains: i32,
    pub last_valor_loss: DateTime,
    pub last_honor_use: DateTime,
    pub honor_active: bool,
    pub justice_protection: Option<MobileId>,
    /// `JusticeProtectorStatus` (0 None, 1 Protector, 2 Protected).
    pub justice_status: i32,
    pub values: Option<Vec<i32>>,
}

fn after(t: DateTime, span: TimeSpan) -> Option<DateTime> {
    t.checked_add(span)
}

impl VirtueContext {
    /// `GetValue` / `SetValue`.
    pub fn set_value(&mut self, index: usize, value: i32) {
        let v = self.values.get_or_insert_with(|| vec![0; 8]);
        if index < v.len() {
            v[index] = value;
        }
    }

    // The `ShouldSerialize*` predicates (each virtue's `CanGain`/`CanAtrophy`/`CanUse` negated).
    fn flags(&self, now: DateTime) -> i32 {
        let day = TimeSpan::from_days(1);
        let week = TimeSpan::from_days(7);
        let gain_blocked = after(self.last_sacrifice_gain, day).is_none_or(|t| now < t);
        let atrophy_blocked = |t: DateTime| after(t, week).is_none_or(|t| !(t < now));
        let mut f = 0;
        let mut set = |bit: i32, on: bool| {
            if on {
                f |= bit;
            }
        };
        set(0x1, gain_blocked);
        set(0x2, atrophy_blocked(self.last_sacrifice_loss));
        set(0x4, self.available_resurrects > 0);
        set(0x8, atrophy_blocked(self.last_justice_loss));
        set(0x10, atrophy_blocked(self.last_compassion_loss));
        set(0x20, self.next_compassion_day > now);
        set(0x40, self.compassion_gains > 0);
        set(0x80, atrophy_blocked(self.last_valor_loss));
        set(0x100, now - self.last_honor_use < TimeSpan::from_mins(5));
        set(0x200, self.honor_active);
        let prot = self.justice_protection.is_some() && self.justice_status != 0;
        set(0x400, prot);
        set(0x800, prot);
        set(
            0x1000,
            self.values
                .as_ref()
                .is_some_and(|v| v.iter().any(|x| *x > 0)),
        );
        f
    }

    /// `IsUsed()`.
    pub fn is_used(&self, now: DateTime) -> bool {
        self.flags(now) != 0
    }

    fn serialize(&self, w: &World, writer: &mut dyn GenericWriter) {
        let f = self.flags(w.now);
        writer.write_encoded_int(1);
        writer.write_enum(4, i64::from(f));
        let anchored = [
            (0x1, self.last_sacrifice_gain),
            (0x2, self.last_sacrifice_loss),
        ];
        for (bit, t) in anchored {
            if f & bit != 0 {
                writer.write_anchored_time_ticks(t.ticks());
            }
        }
        if f & 0x4 != 0 {
            writer.write_i32(self.available_resurrects);
        }
        for (bit, t) in [
            (0x8, self.last_justice_loss),
            (0x10, self.last_compassion_loss),
            (0x20, self.next_compassion_day),
        ] {
            if f & bit != 0 {
                writer.write_anchored_time_ticks(t.ticks());
            }
        }
        if f & 0x40 != 0 {
            writer.write_i32(self.compassion_gains);
        }
        for (bit, t) in [(0x80, self.last_valor_loss), (0x100, self.last_honor_use)] {
            if f & bit != 0 {
                writer.write_anchored_time_ticks(t.ticks());
            }
        }
        if f & 0x400 != 0 {
            writer.write_mobile(w, self.justice_protection);
        }
        if f & 0x800 != 0 {
            writer.write_enum(1, i64::from(self.justice_status)); // a byte enum
        }
        if f & 0x1000 != 0 {
            let v = self.values.as_deref().unwrap_or(&[]);
            writer.write_encoded_int(v.len() as i32);
            for x in v {
                writer.write_i32(*x);
            }
        }
    }

    fn deserialize(w: &World, reader: &mut dyn GenericReader) -> Result<VirtueContext> {
        let version = reader.read_encoded_int()?;
        if !(0..=1).contains(&version) {
            return Err(IoError::InvalidData(format!(
                "VirtueContext: version {version}"
            )));
        }
        let f = reader.read_enum(4)? as i32;
        let utc_now = DateTime::utc_now().ticks();
        // V0 stored these as delta times, V1 as anchored times.
        let time = |reader: &mut dyn GenericReader| -> Result<DateTime> {
            Ok(date(if version == 0 {
                reader.read_delta_time_ticks(utc_now)?
            } else {
                reader.read_anchored_time_ticks()?
            }))
        };
        let mut c = VirtueContext::default();
        if f & 0x1 != 0 {
            c.last_sacrifice_gain = time(reader)?;
        }
        if f & 0x2 != 0 {
            c.last_sacrifice_loss = time(reader)?;
        }
        if f & 0x4 != 0 {
            c.available_resurrects = reader.read_i32()?;
        }
        if f & 0x8 != 0 {
            c.last_justice_loss = time(reader)?;
        }
        if f & 0x10 != 0 {
            c.last_compassion_loss = time(reader)?;
        }
        if f & 0x20 != 0 {
            c.next_compassion_day = time(reader)?;
        }
        if f & 0x40 != 0 {
            c.compassion_gains = reader.read_i32()?;
        }
        if f & 0x80 != 0 {
            c.last_valor_loss = time(reader)?;
        }
        if f & 0x100 != 0 {
            c.last_honor_use = time(reader)?;
        }
        c.honor_active = f & 0x200 != 0;
        if f & 0x400 != 0 {
            c.justice_protection = reader.read_mobile(w)?;
        }
        if f & 0x800 != 0 {
            c.justice_status = reader.read_enum(1)? as i32;
        }
        if f & 0x1000 != 0 {
            let n = reader.read_encoded_int()?;
            let mut v = Vec::with_capacity(n.clamp(0, 64) as usize);
            for _ in 0..n.max(0) {
                v.push(reader.read_i32()?);
            }
            c.values = Some(v);
        }
        Ok(c)
    }
}

/// `VirtueSystem._playerVirtues`.
#[derive(Default)]
pub struct VirtueTable(pub IndexMap<MobileId, VirtueContext>);

struct VirtuePayload;

impl GenericPayload for VirtuePayload {
    fn name(&self) -> &str {
        "Virtues"
    }

    fn serialize(&mut self, w: &World, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(0);
        let empty = VirtueTable::default();
        let t = w.resources.get::<VirtueTable>().unwrap_or(&empty);
        writer.write_encoded_int(t.0.len() as i32);
        for (m, c) in &t.0 {
            writer.write_mobile(w, Some(*m));
            c.serialize(w, writer);
        }
    }

    fn deserialize(&mut self, w: &mut World, reader: &mut dyn GenericReader) -> Result<()> {
        reader.read_encoded_int()?;
        let count = reader.read_encoded_int()?;
        for _ in 0..count.max(0) {
            let player = reader.read_mobile(w)?;
            let c = VirtueContext::deserialize(w, reader)?;
            if let Some(p) = player
                && c.is_used(w.now)
            {
                w.resources.get_mut::<VirtueTable>().0.insert(p, c);
            }
        }
        Ok(())
    }
}

/// `pm.Virtues` (`VirtueSystem.GetOrCreateVirtues`).
pub fn virtues_mut(w: &mut World, pm: MobileId) -> &mut VirtueContext {
    w.resources
        .get_mut::<VirtueTable>()
        .0
        .entry(pm)
        .or_default()
}

/// `JusticeVirtue.AddProtection(protector, protectee)`.
pub fn add_protection(w: &mut World, protector: MobileId, protectee: MobileId) {
    let v = virtues_mut(w, protectee);
    v.justice_status = 2; // Protected
    v.justice_protection = Some(protector);
    let v = virtues_mut(w, protector);
    v.justice_status = 1; // Protector
    v.justice_protection = Some(protectee);
}

/// `VirtueSystem.Initialize`: the `Mobile.VirtueMigrations` values of records below the mobile's own version.
pub fn apply_virtue_migrations(w: &mut World) {
    let migrations = w
        .resources
        .get::<muo_server::mobile::part2::statics::Part2Statics>()
        .and_then(|s| s.virtue_migrations.clone());
    for (m, values) in migrations.into_iter().flatten() {
        if w.mobile(m).is_none_or(|x| !x.ty.is(&super::PLAYER_MOBILE)) {
            continue; // FixVirtue: `m is not PlayerMobile`
        }
        for (i, v) in values.into_iter().enumerate() {
            if v > 0 {
                virtues_mut(w, m).set_value(i, v);
            }
        }
    }
}

// ---- ChampionTitleSystem ----

/// `ChampionTitle`.
#[derive(Copy, Clone, Debug, Default)]
pub struct ChampionTitle {
    pub value: i32,
    pub last_decay: DateTime,
}

/// `ChampionTitleContext`: the titles in field order (Abyss, Arachnid, ColdBlood, ForestLord, VerminHorde,
/// UnholyTerror, SleepingDragon, Corrupt, Glade), which is also the order of the save flags.
#[derive(Clone, Debug, Default)]
pub struct ChampionTitleContext {
    pub harrower: i32,
    pub titles: [Option<ChampionTitle>; 9],
}

/// `ChampionTitleContext.SetValue(type, value)` for the title in `slot` (`titles`' index, the saved field order).
pub fn set_title_value(w: &mut World, pm: MobileId, slot: usize, value: i32) {
    let now = w.now;
    let c = champion_titles_mut(w, pm);
    if c.titles[slot].is_some() && value <= 0 {
        c.titles[slot] = None;
    } else {
        let title = c.titles[slot].get_or_insert_with(ChampionTitle::default);
        title.value = value;
        title.last_decay = now;
    }
    muo_server::mobile::part2::properties::invalidate_properties(w, pm);
}

/// `ChampionTitleContext.SetLastDecay(type, value)`: creates the title like `TryGetOrCreateTitle`.
pub fn set_title_last_decay(w: &mut World, pm: MobileId, slot: usize, value: DateTime) {
    champion_titles_mut(w, pm).titles[slot]
        .get_or_insert_with(ChampionTitle::default)
        .last_decay = value;
}

/// `ChampionSpawnType` (Abyss..Glade, Corrupt) to its slot in [`ChampionTitleContext::titles`].
const SPAWN_TYPE_SLOT: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 8, 7];

impl ChampionTitle {
    fn read(reader: &mut dyn GenericReader) -> Result<ChampionTitle> {
        reader.read_encoded_int()?; // version
        Ok(ChampionTitle {
            value: reader.read_encoded_int()?,
            last_decay: date(reader.read_date_time_ticks()?),
        })
    }

    fn write(&self, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(0);
        writer.write_encoded_int(self.value);
        writer.write_date_time_ticks(self.last_decay.ticks());
    }
}

impl ChampionTitleContext {
    /// `ChampionTitleContext.Deserialize(reader, version)`: the layout inside `PlayerMobile` below version 33.
    pub fn deserialize_legacy(reader: &mut dyn GenericReader) -> Result<ChampionTitleContext> {
        let mut c = ChampionTitleContext {
            harrower: reader.read_encoded_int()?,
            ..Default::default()
        };
        let length = reader.read_encoded_int()?;
        for i in 0..length.max(0) as usize {
            let Some(slot) = SPAWN_TYPE_SLOT.get(i) else {
                // An unknown ChampionSpawnType value fails the load.
                return Err(IoError::InvalidData(format!(
                    "ChampionTitles: no ChampionSpawnType value {i}"
                )));
            };
            c.titles[*slot] = Some(ChampionTitle::read(reader)?);
        }
        Ok(c)
    }

    fn serialize(&self, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(1);
        let mut flags = 0i32;
        for (i, t) in self.titles.iter().enumerate() {
            if t.is_some() {
                flags |= 1 << i;
            }
        }
        writer.write_enum(4, i64::from(flags));
        writer.write_i32(self.harrower);
        for t in self.titles.iter().flatten() {
            t.write(writer);
        }
    }

    fn deserialize(reader: &mut dyn GenericReader) -> Result<ChampionTitleContext> {
        let version = reader.read_encoded_int()?;
        if version < 1 {
            return Self::deserialize_legacy(reader);
        }
        let flags = reader.read_enum(4)? as i32;
        let mut c = ChampionTitleContext {
            harrower: reader.read_i32()?,
            ..Default::default()
        };
        for (i, title) in c.titles.iter_mut().enumerate() {
            if flags & (1 << i) != 0 {
                *title = Some(ChampionTitle::read(reader)?);
            }
        }
        Ok(c)
    }
}

/// `ChampionTitleSystem._championTitleContexts`.
#[derive(Default)]
pub struct ChampionTitleTable(pub IndexMap<MobileId, ChampionTitleContext>);

struct ChampionPayload;

impl GenericPayload for ChampionPayload {
    fn name(&self) -> &str {
        "ChampionTitles"
    }

    fn serialize(&mut self, w: &World, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(0);
        let empty = ChampionTitleTable::default();
        let t = w.resources.get::<ChampionTitleTable>().unwrap_or(&empty);
        writer.write_encoded_int(t.0.len() as i32);
        for (m, c) in &t.0 {
            writer.write_mobile(w, Some(*m));
            c.serialize(writer);
        }
    }

    fn deserialize(&mut self, w: &mut World, reader: &mut dyn GenericReader) -> Result<()> {
        reader.read_encoded_int()?;
        let count = reader.read_encoded_int()?;
        for _ in 0..count.max(0) {
            let player = reader.read_mobile(w)?;
            let c = ChampionTitleContext::deserialize(reader)?;
            if let Some(p) = player {
                w.resources.get_mut::<ChampionTitleTable>().0.insert(p, c);
            }
        }
        Ok(())
    }
}

/// `pm.ChampionTitles` (`ChampionTitleSystem.GetOrCreateChampionTitleContext`).
pub fn champion_titles_mut(w: &mut World, pm: MobileId) -> &mut ChampionTitleContext {
    w.resources
        .get_mut::<ChampionTitleTable>()
        .0
        .entry(pm)
        .or_default()
}

/// `ChampionSpawnInfo.Table[i].LevelNames[0]`, by `ChampionSpawnType` (the other two levels are 11 and 22 lower).
const CHAMPION_LEVEL_NAME: [i32; 9] = [
    1113118, 1113117, 1113115, 1113116, 1113119, 1113120, 1113121, 1113123, 1113122,
];

/// `ChampionTitleSystem.GetChampionTitleLabel(player)`: the cliloc of the highest title, or 0.
pub fn champion_title_label(w: &World, pm: MobileId) -> i32 {
    let Some(c) = w
        .resources
        .get::<ChampionTitleTable>()
        .and_then(|t| t.0.get(&pm))
    else {
        return 0;
    };
    if c.harrower > 0 {
        return 1113082 + c.harrower.min(10);
    }
    let (mut highest_value, mut highest_type) = (0, 0);
    for (i, slot) in SPAWN_TYPE_SLOT.iter().enumerate() {
        let Some(t) = c.titles[*slot] else { continue };
        if t.value > highest_value {
            highest_value = t.value;
            highest_type = i;
        }
    }
    let offset = match highest_value {
        v if v > 800 => 3,
        v if v > 300 => v / 300,
        _ => 0,
    };
    if offset > 0 {
        return CHAMPION_LEVEL_NAME[highest_type] - 11 * (offset.min(3) - 1);
    }
    0
}

/// `PlayerMurderSystem`, `VirtueSystem` and `ChampionTitleSystem`'s `Configure`: each constructs its persistence.
pub fn register_persistence(w: &mut World) {
    w.persistence
        .register(10, Box::new(GenericPersistence::new(MurderPayload)));
    w.persistence
        .register(10, Box::new(GenericPersistence::new(VirtuePayload)));
    w.persistence
        .register(10, Box::new(GenericPersistence::new(ChampionPayload)));
}

/// `PlayerMobile.PlayerDeletedEvent` for the three systems.
pub fn forget_player(w: &mut World, pm: MobileId) {
    w.resources.get_mut::<MurderTable>().0.shift_remove(&pm);
    w.resources.get_mut::<VirtueTable>().0.shift_remove(&pm);
    w.resources
        .get_mut::<ChampionTitleTable>()
        .0
        .shift_remove(&pm);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::{SaveReader, SaveWriter};

    fn round_trip(
        w: &World,
        p: &mut dyn GenericPayload,
        w2: &mut World,
        p2: &mut dyn GenericPayload,
    ) -> Vec<u8> {
        let mut out = SaveWriter::new(true);
        p.serialize(w, &mut out);
        let bytes = out.into_vec();
        p2.deserialize(w2, &mut SaveReader::new(&bytes)).unwrap();
        bytes
    }

    #[test]
    fn the_three_tables_round_trip() {
        let mut w = World::new();
        let id = crate::player_mobile::new_player_mobile(&mut w);
        virtues_mut(&mut w, id).available_resurrects = 2;
        virtues_mut(&mut w, id).set_value(3, 40);
        w.resources.get_mut::<MurderTable>().0.insert(
            id,
            MurderContext {
                short_term_murders: 2,
                bounty: 9,
                ..Default::default()
            },
        );
        champion_titles_mut(&mut w, id).harrower = 4;
        champion_titles_mut(&mut w, id).titles[8] = Some(ChampionTitle {
            value: 350,
            last_decay: DateTime::from_ticks(5_000_000_000),
        });

        let mut w2 = World::new();
        let id2 = crate::player_mobile::new_player_mobile(&mut w2);
        assert_eq!(id, id2, "both worlds hand out the first mobile serial");
        let a = round_trip(&w, &mut MurderPayload, &mut w2, &mut MurderPayload);
        let b = round_trip(&w, &mut VirtuePayload, &mut w2, &mut VirtuePayload);
        let c = round_trip(&w, &mut ChampionPayload, &mut w2, &mut ChampionPayload);
        // MurderContext version 2: a 6 field record per player, after the system's version and count.
        assert_eq!(a[..2], [0, 1]);
        assert_eq!(w2.resources.get::<MurderTable>().unwrap().0[&id2].bounty, 9);
        let v = &w2.resources.get::<VirtueTable>().unwrap().0[&id2];
        assert_eq!(
            (v.available_resurrects, v.values.as_ref().unwrap()[3]),
            (2, 40)
        );
        let t = &w2.resources.get::<ChampionTitleTable>().unwrap().0[&id2];
        assert_eq!((t.harrower, t.titles[8].unwrap().value), (4, 350));
        // Saving what was loaded writes the same bytes.
        assert_eq!(
            b,
            round_trip(
                &w2,
                &mut VirtuePayload,
                &mut World::new(),
                &mut VirtuePayload
            )
        );
        assert_eq!(
            c,
            round_trip(
                &w2,
                &mut ChampionPayload,
                &mut World::new(),
                &mut ChampionPayload
            )
        );
        // A Harrower title (4) outranks the spawn titles.
        assert_eq!(champion_title_label(&w, id), 1113082 + 4);
    }
}
