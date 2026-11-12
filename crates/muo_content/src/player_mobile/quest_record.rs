//! A `PlayerMobile.Quest` in progress (`QuestSerializer`, `QuestSystem`, `QuestObjective`, `QuestConversation`), kept
//! as the bytes `QuestSerializer.Serialize` wrote so that a resave writes the same
//! record back. The quest engine does not read it; the record is walked only far enough to find where it
//! ends: the base layouts here, the per-class `ChildDeserialize` layouts in [`super::quest_tables`].

use muo_io::{GenericReader, IoError, Result};

use super::quest_tables::{CLASSES, SYSTEMS};

/// One read of a `ChildDeserialize`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tok {
    /// `ReadEncodedInt`.
    E,
    /// `ReadBool`.
    B,
    /// `ReadInt`.
    I,
    /// `ReadString`.
    S,
    /// `ReadEntity<T>`: a serial.
    Ser,
    /// `ReadPoint3D`.
    P3,
    /// An encoded count, then that many encoded ints.
    ListE,
    /// An encoded count, then that many (encoded int, bool) pairs.
    ListEB,
    /// `Hag.FindApprenticeObjective`: a version, a `Point3D` from version 1, then a serial.
    FindApprentice,
}

fn bad(what: &str) -> IoError {
    IoError::InvalidData(format!("quest: {what}"))
}

fn run(reader: &mut dyn GenericReader, prog: &[Tok]) -> Result<()> {
    for t in prog {
        match t {
            Tok::E => {
                reader.read_encoded_int()?;
            }
            Tok::B => {
                reader.read_bool()?;
            }
            Tok::I | Tok::Ser => {
                reader.read_i32()?;
            }
            Tok::S => {
                reader.read_string()?;
            }
            Tok::P3 => {
                reader.read_point3d()?;
            }
            Tok::ListE => {
                let n = reader.read_encoded_int()?;
                for _ in 0..n.max(0) {
                    reader.read_encoded_int()?;
                }
            }
            Tok::ListEB => {
                let n = reader.read_encoded_int()?;
                for _ in 0..n.max(0) {
                    reader.read_encoded_int()?;
                    reader.read_bool()?;
                }
            }
            Tok::FindApprentice => {
                let version = reader.read_encoded_int()?;
                if version >= 1 {
                    reader.read_point3d()?;
                }
                if version >= 0 {
                    reader.read_i32()?;
                }
            }
        }
    }
    Ok(())
}

/// `QuestSerializer.ReadType(referenceTable, reader)`: the full name of the class, or None for a null type.
fn read_type(reader: &mut dyn GenericReader, table: &[&str]) -> Result<Option<String>> {
    match reader.read_encoded_int()? {
        1 => {
            let index = reader.read_encoded_int()?;
            Ok(usize::try_from(index)
                .ok()
                .and_then(|i| table.get(i))
                .map(|s| (*s).to_owned()))
        }
        2 => Ok(reader.read_string()?),
        _ => Ok(None),
    }
}

fn class_layout(name: &str) -> Option<&'static [Tok]> {
    CLASSES.iter().find(|(n, _)| *n == name).map(|(_, p)| *p)
}

/// `QuestObjective.BaseDeserialize` and the class's `ChildDeserialize`.
fn objective(reader: &mut dyn GenericReader, table: &[&str]) -> Result<()> {
    if reader.read_encoded_int()? != 1 {
        return Ok(()); // null
    }
    let name = read_type(reader, table)?.ok_or_else(|| bad("unknown objective type"))?;
    let layout = class_layout(&name).ok_or_else(|| bad(&format!("unknown objective {name}")))?;
    let version = reader.read_encoded_int()?;
    if version == 1 {
        reader.read_bool()?; // HasBeenRead
    }
    if version == 0 || version == 1 {
        reader.read_encoded_int()?; // CurProgress
        reader.read_bool()?; // HasCompleted
    }
    run(reader, layout)
}

/// `QuestConversation.BaseDeserialize` and the class's `ChildDeserialize`.
fn conversation(reader: &mut dyn GenericReader, table: &[&str]) -> Result<()> {
    if reader.read_encoded_int()? != 1 {
        return Ok(()); // null
    }
    let name = read_type(reader, table)?.ok_or_else(|| bad("unknown conversation type"))?;
    let layout = class_layout(&name).ok_or_else(|| bad(&format!("unknown conversation {name}")))?;
    if reader.read_encoded_int()? == 0 {
        reader.read_bool()?; // HasBeenRead
    }
    run(reader, layout)
}

/// Reads one `QuestSerializer.DeserializeQuest` body (after its leading `1`) and returns its bytes. A type that is
/// null or unknown gives None with only the type read: no quest is built and nothing more of it is read.
pub fn scan(reader: &mut dyn GenericReader) -> Result<Option<Vec<u8>>> {
    let start = reader.position();
    let names: Vec<&str> = SYSTEMS.iter().map(|s| s.name).collect();
    let Some(system) =
        read_type(reader, &names)?.and_then(|name| SYSTEMS.iter().find(|s| s.name == name))
    else {
        return Ok(None);
    };

    // QuestSystem.BaseDeserialize
    if reader.read_encoded_int()? == 0 {
        for _ in 0..reader.read_encoded_int()?.max(0) {
            objective(reader, system.table)?;
        }
        for _ in 0..reader.read_encoded_int()?.max(0) {
            conversation(reader, system.table)?;
        }
    }
    run(reader, system.child)?;

    let end = reader.position();
    reader.seek(start as i64, muo_io::SeekOrigin::Begin)?;
    Ok(Some(reader.read_vec((end - start) as usize)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_io::SaveReader;

    #[test]
    fn walks_a_summoning_quest_and_keeps_its_bytes() {
        // type indexed 0 (TheSummoningQuest); version 0; no objectives, no conversations; child: version,
        // a serial, a bool. Followed by a byte that belongs to the next field.
        let bytes = [1, 0, 0, 0, 0, 0, 7, 0, 0, 0, 1, 0xAA];
        let mut r = SaveReader::new(&bytes);
        let got = scan(&mut r).unwrap().unwrap();
        assert_eq!(got, &bytes[..11]);
        assert_eq!(r.position(), 11);
    }

    #[test]
    fn walks_an_objective_by_table_index() {
        // Doom.VanquishDaemonObjective is index 3 of The Summoning's table.
        let mut b = vec![1, 0, 0]; // quest 0, version 0
        b.extend([1]); // one objective
        b.extend([1, 1, 3]); // present, indexed, 3
        b.extend([0, 5, 1]); // objective version 0, progress 5, completed
        b.extend([0, 1, 0, 0, 0, 2, 0, 0, 0]); // child: version, two serials
        b.extend([0]); // no conversations
        b.extend([0, 0, 0, 0, 0, 0]); // system child: version, serial, bool
        let n = b.len();
        b.push(9);
        let mut r = SaveReader::new(&b);
        assert_eq!(scan(&mut r).unwrap().unwrap().len(), n);
    }

    #[test]
    fn an_unknown_quest_type_builds_no_quest_and_reads_only_the_type() {
        // by name, an unknown class; then bytes that belong to the next field
        use muo_io::{GenericWriter, SaveWriter};
        let mut o = SaveWriter::new(true);
        o.write_encoded_int(2);
        o.write_string("Nope");
        let mut b = o.as_slice().to_vec();
        let n = b.len();
        b.extend([0xAA, 0xBB]);
        let mut r = SaveReader::new(&b);
        assert_eq!(scan(&mut r).unwrap(), None);
        assert_eq!(r.position(), n as u64);
    }
}
