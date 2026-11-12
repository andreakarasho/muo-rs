//! Guilds: names are unique, each class constructs and round-trips; `Guildstone` is version 4 with the name, the
//! abbreviation and the guild, and loads the
//! older layout.

use std::collections::HashSet;

use muo_io::{GenericWriter, SaveReader, SaveWriter};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ITEM_BASE, Item, ItemType};
use muo_server::serialization_ext::WriteEntityExt;

use super::*;

fn save(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn reload(w: &mut World, t: &'static ItemType, bytes: &[u8]) -> ItemId {
    let id = w.add_item(Item::new_for_load(
        muo_primitives::Serial::new(0x4000_0600),
        t,
    ));
    let mut r = SaveReader::new(bytes);
    (t.deserialize)(w, id, &mut r).unwrap_or_else(|e| panic!("{}: {e}", t.full_name));
    assert_eq!(r.remaining(), 0, "{}", t.full_name);
    id
}

#[test]
fn names_are_unique_and_every_class_round_trips() {
    let mut seen = HashSet::new();
    for t in ALL {
        assert!(seen.insert(t.full_name));
        let mut w = World::new();
        let id = if t.full_name == "Server.Items.Guildstone" {
            guildstone_new(&mut w, t, None, None, None)
        } else {
            crate::misc::tests::construct_any(&mut w, t)
        };
        let bytes = save(&w, id);
        let again = reload(&mut w, t, &bytes);
        assert_eq!(save(&w, again), bytes, "{}", t.full_name);
    }
}

#[test]
fn guildstone_record_and_old_layout() {
    let mut w = World::new();
    let id = guildstone_new(
        &mut w,
        &GUILDSTONE,
        None,
        Some("The Order".into()),
        Some("TO".into()),
    );
    let mut expected = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, id, &mut expected);
    expected.write_i32(4);
    expected.write_opt_string(Some("The Order"));
    expected.write_opt_string(Some("TO"));
    expected.write_guild(&w, None);
    assert_eq!(save(&w, id), expected.into_vec());

    // version 3: a bool ("before change over") first
    let mut old = SaveWriter::new(true);
    (ITEM_BASE.serialize)(&w, id, &mut old);
    old.write_i32(3);
    old.write_bool(true);
    old.write_opt_string(Some("Old"));
    old.write_opt_string(Some("O"));
    old.write_guild(&w, None);
    let loaded = reload(&mut w, &GUILDSTONE, &old.into_vec());
    let s = stone(&w, loaded);
    assert_eq!(
        (s.guild_name.as_deref(), s.guild_abbrev.as_deref()),
        (Some("Old"), Some("O"))
    );
}

#[test]
fn teleporter_keeps_its_stone() {
    let mut w = World::new();
    let stone_item = guildstone_new(&mut w, &GUILDSTONE, None, None, None);
    let t = guild_teleporter_new(&mut w, &GUILD_TELEPORTER, Some(stone_item));
    assert_eq!(teleporter_stone(&w, t), Some(stone_item));
    let bytes = save(&w, t);
    let again = reload(&mut w, &GUILD_TELEPORTER, &bytes);
    assert_eq!(teleporter_stone(&w, again), Some(stone_item));
}

#[test]
fn guild_deed_prompt_cancel_says_the_placement_was_cancelled() {
    let mut w = World::new();
    let m = w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE);
    let ns = muo_server::net_state::NetStateId(1);
    let mut d = muo_server::net_state::NetStateData::new(0);
    d.mobile = Some(m);
    w.net_states.insert(ns, d);
    w.mobile_mut(m).unwrap().net_state = Some(ns);
    let deed = muo_server::item::ops::new_item_with_id(&mut w, &GUILD_DEED, 0x14F0);
    let pid = muo_server::prompt::add_prompt(&mut w, guild_deed_prompt(deed));
    muo_server::prompt::on_cancel(&mut w, pid, m);
    let clilocs: Vec<u32> = w
        .net_out
        .iter()
        .filter_map(|o| match o {
            muo_server::net_state::NetOut::Send(_, b) if b[0] == 0xC1 => {
                Some(u32::from_be_bytes(b[14..18].try_into().unwrap()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(clilocs, vec![501145]);
}
