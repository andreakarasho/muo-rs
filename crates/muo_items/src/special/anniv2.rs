//! Dawn's music box and `FountainOfLife` (with `EnhancedBandage`).
//!
//! `EnhancedBandage` derives from `Bandage` (skill_items/bandage.rs)
//! (its save bytes are those of a `Bandage` child: one extra `Bandage` version int) and `item is Bandage` is the class
//! name test; `GetTracks(rarity).Shuffle()` shuffles a copy (shuffling the shared array in place would only
//! change the order later `RandomElement` picks from).

use muo_content::gumps::GumpButtonType;
use muo_content::gumps::base::{DynamicGumpLogic, DynamicGumpObj, RelayInfo};
use muo_content::gumps::builder::DynamicGumpBuilder;
use muo_content::gumps::send_gump_to_mobile;
use muo_content::items::addon_container::{
    self, BASE_ADDON_CONTAINER, BASE_ADDON_CONTAINER_CLASS, BASE_ADDON_CONTAINER_DEED,
    ContainerAddonClass, ContainerDeedClass,
};
use muo_content::items::house_deps;
use muo_content::props::{self, Ctor, Kind, Param, PropDef, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};

use muo_primitives::random::Rng;
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::World;
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, LootType, container, ops};
use muo_server::mobile::AccessLevel;
use muo_server::net_state::NetStateId;
use muo_server::region::{MUSIC_NAMES, MusicName};
use muo_server::target::{self, TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::TypeRegistry;

use super::util::{access_level, delete, deleted, in_pack, msg};

// ---- DawnsMusicInfo ----

/// `DawnsMusicRarity`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DawnsMusicRarity {
    Common,
    Uncommon,
    Rare,
}

use DawnsMusicRarity as R;
use MusicName as M;

/// `DawnsMusicBox._info`: `(track, DawnsMusicInfo.Name, DawnsMusicInfo.Rarity)`.
const INFO: [(MusicName, i32, DawnsMusicRarity); 55] = [
    (M::Samlethe, 1075152, R::Common),
    (M::Sailing, 1075163, R::Common),
    (M::Britain2, 1075145, R::Common),
    (M::Britain1, 1075144, R::Common),
    (M::Bucsden, 1075146, R::Common),
    (M::ForestA, 1075161, R::Common),
    (M::Cove, 1075176, R::Common),
    (M::Death, 1075171, R::Common),
    (M::Dungeon9, 1075160, R::Common),
    (M::Dungeon2, 1075175, R::Common),
    (M::Cave01, 1075159, R::Common),
    (M::Combat3, 1075170, R::Common),
    (M::Combat1, 1075168, R::Common),
    (M::Combat2, 1075169, R::Common),
    (M::Jhelom, 1075147, R::Common),
    (M::Linelle, 1075185, R::Common),
    (M::LBCastle, 1075148, R::Common),
    (M::Minoc, 1075150, R::Common),
    (M::Moonglow, 1075177, R::Common),
    (M::Magincia, 1075149, R::Common),
    (M::Nujelm, 1075174, R::Common),
    (M::BTCastle, 1075173, R::Common),
    (M::Tavern04, 1075167, R::Common),
    (M::Skarabra, 1075154, R::Common),
    (M::Stones2, 1075143, R::Common),
    (M::Serpents, 1075153, R::Common),
    (M::Taiko, 1075180, R::Common),
    (M::Tavern01, 1075164, R::Common),
    (M::Tavern02, 1075165, R::Common),
    (M::Tavern03, 1075166, R::Common),
    (M::TokunoDungeon, 1075179, R::Common),
    (M::Trinsic, 1075155, R::Common),
    (M::OldUlt01, 1075142, R::Common),
    (M::Ocllo, 1075151, R::Common),
    (M::Vesper, 1075156, R::Common),
    (M::Victory, 1075172, R::Common),
    (M::MountnA, 1075162, R::Common),
    (M::Wind, 1075157, R::Common),
    (M::Yew, 1075158, R::Common),
    (M::Zento, 1075178, R::Common),
    (M::GwennoConversation, 1075131, R::Uncommon),
    (M::DreadHornArea, 1075181, R::Uncommon),
    (M::ElfCity, 1075182, R::Uncommon),
    (M::GoodEndGame, 1075132, R::Uncommon),
    (M::GoodVsEvil, 1075133, R::Uncommon),
    (M::GreatEarthSerpents, 1075134, R::Uncommon),
    (M::GrizzleDungeon, 1075186, R::Uncommon),
    (M::HumanoidsU9, 1075135, R::Uncommon),
    (M::MelisandesLair, 1075183, R::Uncommon),
    (M::MinocNegative, 1075136, R::Uncommon),
    (M::ParoxysmusLair, 1075184, R::Uncommon),
    (M::Paws, 1075137, R::Uncommon),
    (M::SelimsBar, 1075138, R::Rare),
    (M::SerpentIsleCombatU7, 1075139, R::Rare),
    (M::ValoriaShips, 1075140, R::Rare),
];

/// `DawnsMusicBox.GetInfo(name)`.
pub fn get_info(track: i32) -> Option<(i32, DawnsMusicRarity)> {
    INFO.iter()
        .find(|r| r.0 as i32 == track)
        .map(|r| (r.1, r.2))
}

const COMMON_TRACKS: [MusicName; 40] = [
    M::Samlethe,
    M::Sailing,
    M::Britain2,
    M::Britain1,
    M::Bucsden,
    M::ForestA,
    M::Cove,
    M::Death,
    M::Dungeon9,
    M::Dungeon2,
    M::Cave01,
    M::Combat3,
    M::Combat1,
    M::Combat2,
    M::Jhelom,
    M::Linelle,
    M::LBCastle,
    M::Minoc,
    M::Moonglow,
    M::Magincia,
    M::Nujelm,
    M::BTCastle,
    M::Tavern04,
    M::Skarabra,
    M::Stones2,
    M::Serpents,
    M::Taiko,
    M::Tavern01,
    M::Tavern02,
    M::Tavern03,
    M::TokunoDungeon,
    M::Trinsic,
    M::OldUlt01,
    M::Ocllo,
    M::Vesper,
    M::Victory,
    M::MountnA,
    M::Wind,
    M::Yew,
    M::Zento,
];

const UNCOMMON_TRACKS: [MusicName; 12] = [
    M::GwennoConversation,
    M::DreadHornArea,
    M::ElfCity,
    M::GoodEndGame,
    M::GoodVsEvil,
    M::GreatEarthSerpents,
    M::GrizzleDungeon,
    M::HumanoidsU9,
    M::MelisandesLair,
    M::MinocNegative,
    M::ParoxysmusLair,
    M::Paws,
];

const RARE_TRACKS: [MusicName; 3] = [M::SelimsBar, M::SerpentIsleCombatU7, M::ValoriaShips];

/// `DawnsMusicBox.GetTracks(rarity)`.
pub fn get_tracks(rarity: DawnsMusicRarity) -> &'static [MusicName] {
    match rarity {
        R::Common => &COMMON_TRACKS,
        R::Uncommon => &UNCOMMON_TRACKS,
        R::Rare => &RARE_TRACKS,
    }
}

/// `DawnsMusicBox.RandomTrack(rarity)`.
pub fn random_track(w: &mut World, rarity: DawnsMusicRarity) -> MusicName {
    let t = get_tracks(rarity);
    t[muo_primitives::utility::random(&mut w.rng, t.len() as i32) as usize]
}

// ---- DawnsMusicBox ----

/// The fields of `DawnsMusicBox`; `tracks` and `level` are `[SerializedIgnoreDupe]`.
#[derive(Clone, Debug, Default)]
struct BoxData {
    tracks: Vec<i32>,
    level: i32,
    item_id: i32,
    count: i32,
    timer: Option<TimerHandle>,
}

fn bx(w: &World, id: ItemId) -> BoxData {
    w.item(id)
        .and_then(|i| i.ext.get::<BoxData>())
        .cloned()
        .unwrap_or_default()
}

fn bx_mut(w: &mut World, id: ItemId) -> Option<&mut BoxData> {
    w.item_mut(id)?.ext.get_mut::<BoxData>()
}

fn set_bx(w: &mut World, id: ItemId, d: BoxData) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(d);
    }
}

/// `DawnsMusicBox()`.
pub fn dawns_music_box_new(w: &mut World) -> ItemId {
    let id = crate::simple::from_template(w, &DAWNS_MUSIC_BOX, "Server.Items.DawnsMusicBox");
    let mut shuffled = COMMON_TRACKS;
    w.rng.shuffle(&mut shuffled);
    set_bx(
        w,
        id,
        BoxData {
            tracks: shuffled[..4].iter().map(|&t| t as i32).collect(),
            ..BoxData::default()
        },
    );
    id
}

/// `DawnsMusicBox.HasAccess(m)`.
pub fn box_has_access(w: &World, id: ItemId, m: MobileId) -> bool {
    access_level(w, m) >= AccessLevel::GameMaster
        || muo_multis::house::access::find_house_at_item(w, id)
            .is_some_and(|h| muo_multis::house::access::has_access(w, h, m))
}

fn box_in_pack(w: &mut World, id: ItemId, m: MobileId) -> bool {
    in_pack(w, id, m)
}

/// `DawnsMusicBox.PlayMusic(m, music)`.
fn play_music(w: &mut World, id: ItemId, m: MobileId, music: i32) {
    let running = bx(w, id).timer.is_some_and(|t| t.running(w));
    if running {
        end_music(w, id, m);
    } else {
        // `_itemID = ItemID`: the generated `ItemID` property of this class hides `Item.ItemID`, so every `ItemID` in
        // DawnsMusicBox's graphic is `_itemID` (the graphic never changes): a no-op.
    }
    if let (Some(ns), Some(name)) = (
        w.mobile(m).and_then(|x| x.net_state),
        MusicName::from_i32(music),
    ) {
        muo_server::packets::player::send_play_music(w, ns, name);
    }
    let h = timer::start(
        w,
        TimeSpan::from_millis(500),
        TimeSpan::from_millis(500),
        4,
        move |w| animate(w, id),
    );
    if let Some(d) = bx_mut(w, id) {
        d.timer = Some(h);
    }
}

/// `DawnsMusicBox.EndMusic(m)`.
fn end_music(w: &mut World, id: ItemId, m: MobileId) {
    if let Some(mut h) = bx(w, id).timer {
        h.cancel(w);
    }
    if let Some(ns) = w.mobile(m).and_then(|x| x.net_state) {
        muo_server::packets::player::send_stop_music(w, ns);
    }
    // `if (_count > 0) ItemID = _itemID` is `_itemID = _itemID` (see `play_music`).
    if let Some(d) = bx_mut(w, id) {
        d.count = 0;
    }
}

/// `DawnsMusicBox.Animate()`.
fn animate(w: &mut World, id: ItemId) {
    let d = bx(w, id);
    let count = d.count + 1;
    if count >= 4 {
        if let Some(d) = bx_mut(w, id) {
            d.count = 0;
        }
    } else if let Some(d) = bx_mut(w, id) {
        d.count = count;
        d.item_id += 1; // `ItemID++` on the hiding property: only `_itemID` moves
    }
}

/// `DawnsMusicBoxGump`.
struct MusicBoxGump {
    box_id: ItemId,
    tracks: Vec<i32>,
}

fn music_box_gump(w: &mut World, from: MobileId, box_id: ItemId) {
    if super::util::net_state(w, from).is_none() || deleted(w, box_id) {
        return;
    }
    let tracks = bx(w, box_id).tracks;
    send_gump_to_mobile(
        w,
        from,
        Box::new(DynamicGumpObj::new(60, 36, MusicBoxGump { box_id, tracks })),
        false,
    );
}

impl DynamicGumpLogic for MusicBoxGump {
    const TYPE_NAME: &'static str = "Server.Gumps.DawnsMusicBoxGump";

    fn singleton(&self) -> bool {
        true
    }

    fn build_layout(&self, b: &mut DynamicGumpBuilder) {
        b.add_page(0);
        b.add_background(0, 0, 273, 324, 0x13BE);
        b.add_image_tiled(10, 10, 253, 20, 0xA40);
        b.add_image_tiled(10, 40, 253, 244, 0xA40);
        b.add_image_tiled(10, 294, 253, 20, 0xA40);
        b.add_alpha_region(10, 10, 253, 304);
        b.add_button(10, 294, 0xFB1, 0xFB2, 0);
        b.add_html_localized_color(45, 296, 450, 20, 1060051, 0x7FFF, false, false); // CANCEL
        b.add_html_localized_color(14, 12, 273, 20, 1075130, 0x7FFF, false, false); // Choose a track to play

        let mut page = 1;
        let mut y = 49;
        b.add_page(page);
        let mut i = 0;
        while i < self.tracks.len() {
            let info = get_info(self.tracks[i]);
            if i > 0 && i % 10 == 0 {
                b.add_button_type(228, 294, 0xFA5, 0xFA6, 0, GumpButtonType::Page, page + 1);
                b.add_page(page + 1);
                y = 49;
                b.add_button_type(193, 294, 0xFAE, 0xFAF, 0, GumpButtonType::Page, page);
                page += 1;
            }
            if let Some((name, _)) = info {
                b.add_button(19, y, 0x845, 0x846, 100 + i as i32);
                b.add_html_localized_color(44, y - 2, 213, 20, name, 0x7FFF, false, false);
            }
            i += 1;
            y += 24;
        }
        if i % 10 == 0 {
            b.add_button_type(228, 294, 0xFA5, 0xFA6, 0, GumpButtonType::Page, page + 1);
            b.add_page(page + 1);
            y = 49;
            b.add_button_type(193, 294, 0xFAE, 0xFAF, 0, GumpButtonType::Page, page);
        }
        b.add_button(19, y, 0x845, 0x846, 1);
        b.add_html_localized_color(44, y - 2, 213, 20, 1075207, 0x7FFF, false, false); // Stop Song
    }

    fn on_response(&mut self, w: &mut World, sender: NetStateId, info: &RelayInfo) {
        let id = self.box_id;
        if deleted(w, id) {
            return;
        }
        let Some(m) = w.net_states.get(&sender).and_then(|n| n.mobile) else {
            return;
        };
        let locked = ops::is_locked_down(w, id);
        if !box_in_pack(w, id, m) && !locked {
            msg(w, m, 1061856); // You must have the item in your backpack or locked down in order to use it.
        } else if locked && !box_has_access(w, id, m) {
            msg(w, m, 502691); // You must be the owner to use this.
        } else if info.button_id == 1 {
            end_music(w, id, m);
        } else if info.button_id >= 100 {
            let tracks = bx(w, id).tracks;
            if let Some(&t) = tracks.get((info.button_id - 100) as usize) {
                play_music(w, id, m, t);
            }
        }
    }
}

fn box_context_menu(w: &mut World, id: ItemId, from: MobileId, list: &mut Vec<ContextMenuEntry>) {
    (ITEM_BASE.get_context_menu_entries)(w, id, from, list);
    if let Some(e) = muo_content::items::common::hooks(w)
        .and_then(|h| h.secure_level_entry)
        .and_then(|f| f(w, from, id))
    {
        list.push(e);
    }
}

/// `[AfterDeserialization]`: `ItemID = _itemID` is `_itemID = _itemID` (the generated property hides `Item.ItemID`).
fn box_after_deserialization(_w: &mut World, _id: ItemId) {}

/// `Server.Items.DawnsMusicBox` (`[Flippable(0x2AF9, 0x2AFD)]`, `ISecurable`).
pub static DAWNS_MUSIC_BOX: ItemType = ItemType {
    full_name: "Server.Items.DawnsMusicBox",
    parent: Some(&ITEM_BASE),
    construct: Some(dawns_music_box_new),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // Dawn's Music Box
    on_after_duped: |w, id, new| {
        (ITEM_BASE.on_after_duped)(w, id, new);
        if w.item(new).is_some_and(|i| i.ty.is(&DAWNS_MUSIC_BOX)) {
            let tracks = bx(w, id).tracks;
            if let Some(d) = bx_mut(w, new) {
                d.tracks = tracks;
            }
        }
    },
    get_properties: |w, id, list| {
        (ITEM_BASE.get_properties)(w, id, list);
        let (mut common, mut uncommon, mut rare) = (0, 0, 0);
        for t in bx(w, id).tracks {
            match get_info(t).map(|i| i.1) {
                Some(R::Common) => common += 1,
                Some(R::Uncommon) => uncommon += 1,
                Some(R::Rare) => rare += 1,
                None => {}
            }
        }
        if common > 0 {
            list.add_int(1075234, common); // ~1_NUMBER~ Common Tracks
        }
        if uncommon > 0 {
            list.add_int(1075235, uncommon); // ~1_NUMBER~ Uncommon Tracks
        }
        if rare > 0 {
            list.add_int(1075236, rare); // ~1_NUMBER~ Rare Tracks
        }
    },
    get_context_menu_entries: box_context_menu,
    on_double_click: |w, id, from| {
        let locked = ops::is_locked_down(w, id);
        if !box_in_pack(w, id, from) && !locked {
            msg(w, from, 1061856); // You must have the item in your backpack or locked down in order to use it.
        } else if locked && !box_has_access(w, id, from) {
            msg(w, from, 502436); // That is not accessible.
        } else {
            music_box_gump(w, from, id);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(1);
        let d = bx(w, id);
        out.write_encoded_int(d.tracks.len() as i32);
        for t in &d.tracks {
            out.write_enum(4, i64::from(*t));
        }
        out.write_enum(4, i64::from(d.level));
        out.write_i32(d.item_id);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let mut d = BoxData::default();
        if version < 1 {
            let count = r.read_i32()?;
            for _ in 0..count {
                d.tracks.push(r.read_i32()?);
            }
            d.level = r.read_i32()?;
        } else {
            let count = r.read_encoded_int()?;
            for _ in 0..count {
                d.tracks.push(r.read_enum(4)? as i32);
            }
            d.level = r.read_enum(4)? as i32;
        }
        d.item_id = r.read_i32()?;
        set_bx(w, id, d);
        box_after_deserialization(w, id);
        Ok(())
    },
    ..ITEM_BASE
};

// ---- DawnsMusicGear ----

/// `DawnsMusicGear._music`.
#[derive(Clone, Copy, Debug, Default)]
struct GearMusic(i32);

fn gear_music(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<GearMusic>())
        .map_or(0, |m| m.0)
}

/// `DawnsMusicGear(music)`.
pub fn dawns_music_gear_new_music(w: &mut World, music: i32) -> ItemId {
    let id = crate::simple::from_template(w, &DAWNS_MUSIC_GEAR, "Server.Items.DawnsMusicGear");
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(GearMusic(music));
    }
    id
}

/// `DawnsMusicGear()`: a random common track.
pub fn dawns_music_gear_new(w: &mut World) -> ItemId {
    let t = random_track(w, R::Common);
    dawns_music_gear_new_music(w, t as i32)
}

/// `DawnsMusicGear.InternalTarget.OnTarget`.
fn gear_on_target(w: &mut World, gear: ItemId, from: MobileId, t: Targeted) {
    if deleted(w, gear) {
        return;
    }
    let Targeted::Entity(EntityId::Item(b)) = t else {
        msg(w, from, 1071964); // Gears can only be put into a musicbox.
        return;
    };
    if !w.item(b).is_some_and(|i| i.ty.is(&DAWNS_MUSIC_BOX)) {
        msg(w, from, 1071964); // Gears can only be put into a musicbox.
        return;
    }
    let music = gear_music(w, gear);
    if !bx(w, b).tracks.contains(&music) {
        if let Some(d) = bx_mut(w, b) {
            d.tracks.push(music);
        }
        msg(w, from, 1071962); // This song track is already in the musicbox.
        return;
    }
    ops::invalidate_properties(w, b);
    delete(w, gear);
    msg(w, from, 1071961); // This song has been added to the musicbox.
}

/// `Server.Items.DawnsMusicGear` (`[Flippable(0x1053, 0x1054)]`).
pub static DAWNS_MUSIC_GEAR: ItemType = ItemType {
    full_name: "Server.Items.DawnsMusicGear",
    parent: Some(&ITEM_BASE),
    construct: Some(dawns_music_gear_new),
    default_weight: crate::class_props::default_weight,
    add_name_property: |w, id, list| {
        let Some((name, rarity)) = get_info(gear_music(w, id)) else {
            (ITEM_BASE.add_name_property)(w, id, list);
            return;
        };
        match rarity {
            R::Common => list.add(1075204), // Gear for Dawn's Music Box (Common)
            R::Uncommon => list.add(1075205), // Gear for Dawn's Music Box (Uncommon)
            R::Rare => list.add(1075206),   // Gear for Dawn's Music Box (Rare)
        }
        list.add(name);
    },
    on_double_click: |w, id, from| {
        target::begin_target(w, from, 2, false, TargetFlags::NONE, move |w, from, t| {
            gear_on_target(w, id, from, t)
        });
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_encoded_int(2);
        out.write_enum(4, i64::from(gear_music(w, id)));
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_encoded_int()?;
        let music = if version < 2 {
            r.read_i32()?
        } else {
            r.read_enum(4)? as i32
        };
        if let Some(i) = w.item_mut(id) {
            i.ext.insert(GearMusic(music));
        }
        Ok(())
    },
    ..ITEM_BASE
};

// ---- EnhancedBandage ----

/// `EnhancedBandage(amount)`.
pub fn enhanced_bandage_new(w: &mut World, amount: i32) -> ItemId {
    let id = crate::simple::from_template(w, &ENHANCED_BANDAGE, "Server.Items.EnhancedBandage");
    ops::set_amount(w, id, amount);
    super::hue(w, id, 0x8A5);
    id
}

/// `Server.Items.EnhancedBandage` (a `Bandage`: see the module comment).
pub static ENHANCED_BANDAGE: ItemType = ItemType {
    full_name: "Server.Items.EnhancedBandage",
    parent: Some(&crate::skill_items::bandage::BANDAGE),
    construct: Some(|w| enhanced_bandage_new(w, 1)),
    default_weight: crate::class_props::default_weight,
    label_number: crate::class_props::label_number, // enhanced bandage
    add_name_properties: |w, id, list| {
        (crate::skill_items::bandage::BANDAGE.add_name_properties)(w, id, list);
        list.add(1075216); // these bandages have been enhanced
    },
    serialize: |w, id, out| {
        (crate::skill_items::bandage::BANDAGE.serialize)(w, id, out);
        out.write_encoded_int(0);
    },
    deserialize: |w, id, r| {
        (crate::skill_items::bandage::BANDAGE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        Ok(())
    },
    ..crate::skill_items::bandage::BANDAGE
};

// ---- FountainOfLife ----

pub const MAX_CHARGES: i32 = 10;

/// `FountainOfLife._charges` / `FountainOfLifeDeed._charges`.
#[derive(Clone, Copy, Debug, Default)]
struct Charges(i32);

/// `FountainOfLife._timer`.
#[derive(Clone, Copy, Debug, Default)]
struct FountainTimer(Option<TimerHandle>);

fn charges(w: &World, id: ItemId) -> i32 {
    w.item(id)
        .and_then(|i| i.ext.get::<Charges>())
        .map_or(0, |c| c.0)
}

fn set_charges_raw(w: &mut World, id: ItemId, n: i32) {
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(Charges(n));
    }
}

/// `Charges = n` (`Math.Min(value, MaxCharges)`, `[InvalidateProperties]`).
fn set_charges(w: &mut World, id: ItemId, n: i32) {
    let n = n.min(MAX_CHARGES);
    if charges(w, id) != n {
        set_charges_raw(w, id, n);
        ops::invalidate_properties(w, id);
    }
}

fn fountain_timer(w: &World, id: ItemId) -> Option<TimerHandle> {
    w.item(id)
        .and_then(|i| i.ext.get::<FountainTimer>())
        .and_then(|t| t.0)
}

/// `FountainOfLife.DeserializeTimer(delay)`: `Timer.DelayCall(Max(delay, 0), RechargeTime, Recharge)`.
fn deserialize_timer(w: &mut World, id: ItemId, delay: TimeSpan) {
    let delay = if delay < TimeSpan::ZERO {
        TimeSpan::ZERO
    } else {
        delay
    };
    let h = timer::start(w, delay, TimeSpan::from_days(1), 0, move |w| {
        recharge(w, id)
    });
    if let Some(i) = w.item_mut(id) {
        i.ext.insert(FountainTimer(Some(h)));
    }
}

/// `FountainOfLife.Recharge()`.
pub fn recharge(w: &mut World, id: ItemId) {
    set_charges(w, id, MAX_CHARGES);
    enhance(w, id, None);
}

fn is_bandage(w: &World, item: ItemId) -> bool {
    w.item(item)
        .is_some_and(|i| i.ty.is(&crate::skill_items::bandage::BANDAGE))
}

/// `FountainOfLife.Enhance(from)`.
pub fn enhance(w: &mut World, id: ItemId, from: Option<MobileId>) {
    let mut i = w.item(id).map_or(0, |x| x.items.len()) as i64 - 1;
    while i >= 0 && charges(w, id) > 0 {
        let Some(item) = w.item(id).and_then(|x| x.items.get(i as usize).copied()) else {
            i -= 1;
            continue;
        };
        if w.item(item).is_some_and(|x| x.ty.is(&ENHANCED_BANDAGE)) {
            i -= 1;
            continue;
        }
        if is_bandage(w, item) {
            let c = charges(w, id);
            let amount = w.item(item).map_or(0, |x| x.amount);
            let enhanced;
            if amount > c {
                ops::set_amount(w, item, amount - c);
                enhanced = enhanced_bandage_new(w, c);
                set_charges(w, id, 0);
            } else {
                enhanced = enhanced_bandage_new(w, amount);
                set_charges(w, id, c - amount);
                delete(w, item);
            }
            // try stacking first
            if (from.is_none() || !container::try_drop_item(w, id, from, enhanced, false))
                && let Some(ty) = w.item(id).map(|x| x.ty)
            {
                (ty.drop_item)(w, id, enhanced);
            }
        }
        i -= 1;
    }
    ops::invalidate_properties(w, id);
}

/// `FountainOfLife(charges)`.
pub fn fountain_of_life_new(w: &mut World, charges: i32) -> ItemId {
    let id = addon_container::new_addon_container(w, &FOUNTAIN_OF_LIFE, 0x2AC0);
    set_charges_raw(w, id, charges);
    id
}

fn fountain_deed_of(w: &mut World, a: ItemId) -> ItemId {
    fountain_of_life_deed_new(w, charges(w, a))
}

static FOUNTAIN_CLASS: ContainerAddonClass = ContainerAddonClass {
    deed_of: Some(fountain_deed_of),
    ..BASE_ADDON_CONTAINER_CLASS
};

/// `Server.Items.FountainOfLife` (`[Flippable(0x2AC0, 0x2AC3)]`).
pub static FOUNTAIN_OF_LIFE: ItemType = ItemType {
    full_name: "Server.Items.FountainOfLife",
    parent: Some(&BASE_ADDON_CONTAINER),
    construct: Some(|w| fountain_of_life_new(w, MAX_CHARGES)),
    family: Some(&FOUNTAIN_CLASS),
    label_number: crate::class_props::label_number, // Fountain of Life
    default_gump_id: crate::class_props::default_gump_id,
    default_drop_sound: crate::class_props::default_drop_sound,
    default_max_items: crate::class_props::default_max_items,
    on_drag_lift: |_, _, _| false,
    on_drag_drop: |w, id, from, dropped| {
        if !is_bandage(w, dropped) {
            msg(w, from, 1075209); // Only bandages may be dropped into the fountain.
            return false;
        }
        if (BASE_ADDON_CONTAINER.on_drag_drop)(w, id, from, dropped) {
            enhance(w, id, Some(from));
            return true;
        }
        false
    },
    on_drag_drop_into: |w, id, from, item, p: Point3D| {
        if !is_bandage(w, item) {
            msg(w, from, 1075209); // Only bandages may be dropped into the fountain.
            return false;
        }
        if (BASE_ADDON_CONTAINER.on_drag_drop_into)(w, id, from, item, p) {
            enhance(w, id, Some(from));
            return true;
        }
        false
    },
    add_name_properties: |w, id, list| {
        (BASE_ADDON_CONTAINER.add_name_properties)(w, id, list);
        list.add_int(1075217, charges(w, id)); // ~1_val~ charges remaining
    },
    on_delete: |w, id| {
        if let Some(mut h) = fountain_timer(w, id) {
            h.cancel(w);
        }
        (BASE_ADDON_CONTAINER.on_delete)(w, id);
    },
    serialize: |w, id, out| {
        (BASE_ADDON_CONTAINER.serialize)(w, id, out);
        out.write_encoded_int(1);
        out.write_i32(charges(w, id));
        let next = fountain_timer(w, id).map_or(DateTime::MIN, |h| h.next(w));
        out.write_date_time_ticks(next.ticks());
    },
    deserialize: |w, id, r| {
        (BASE_ADDON_CONTAINER.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let c = r.read_i32()?;
        let next = DateTime::from_ticks(r.read_date_time_ticks()?);
        set_charges_raw(w, id, c);
        if next != DateTime::MIN {
            let delay = next - w.now;
            deserialize_timer(w, id, delay);
        }
        Ok(())
    },
    ..BASE_ADDON_CONTAINER
};

/// `FountainOfLifeDeed(charges)`.
pub fn fountain_of_life_deed_new(w: &mut World, charges: i32) -> ItemId {
    let id = addon_container::new_deed(w, &FOUNTAIN_OF_LIFE_DEED);
    ops::set_loot_type(w, id, LootType::Blessed);
    set_charges_raw(w, id, charges);
    id
}

static FOUNTAIN_DEED_CLASS: ContainerDeedClass = ContainerDeedClass {
    addon: |w, deed| fountain_of_life_new(w, charges(w, deed)),
    select_direction: None,
};

/// `Server.Items.FountainOfLifeDeed`.
pub static FOUNTAIN_OF_LIFE_DEED: ItemType = ItemType {
    full_name: "Server.Items.FountainOfLifeDeed",
    parent: Some(&BASE_ADDON_CONTAINER_DEED),
    construct: Some(|w| fountain_of_life_deed_new(w, MAX_CHARGES)),
    family: Some(&FOUNTAIN_DEED_CLASS),
    label_number: crate::class_props::label_number, // Fountain of Life
    serialize: |w, id, out| {
        (BASE_ADDON_CONTAINER_DEED.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_i32(charges(w, id));
    },
    deserialize: |w, id, r| {
        (BASE_ADDON_CONTAINER_DEED.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let c = r.read_i32()?;
        set_charges_raw(w, id, c);
        Ok(())
    },
    ..BASE_ADDON_CONTAINER_DEED
};

// ---- registration ----

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));

const MUSIC_ENUM_KIND: Kind = Kind::Plain(PropType::Enum(&MUSIC_NAMES));

/// `[CommandProperty(GameMaster)] DawnsMusicGear.Music`.
static GEAR_PROPS: &[PropDef] = &[PropDef {
    name: "Music",
    kind: MUSIC_ENUM_KIND,
    read: AccessLevel::GameMaster,
    write: AccessLevel::GameMaster,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Enum(i64::from(gear_music(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err("not an item".to_string());
        };
        if let Some(it) = w.item_mut(i) {
            it.ext.insert(GearMusic(v.i64() as i32));
        }
        ops::invalidate_properties(w, i);
        Ok(())
    }),
}];

/// `[CommandProperty(GameMaster)] Charges` of the fountain.
static FOUNTAIN_PROPS: &[PropDef] = &[PropDef {
    name: "Charges",
    kind: INT,
    read: AccessLevel::GameMaster,
    write: AccessLevel::GameMaster,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(i64::from(charges(w, i))),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err("not an item".to_string());
        };
        set_charges(w, i, v.i32());
        Ok(())
    }),
}];

/// `[CommandProperty(GameMaster)] Level` of the music box.
static BOX_PROPS: &[PropDef] = &[PropDef {
    name: "Level",
    kind: INT,
    read: AccessLevel::GameMaster,
    write: AccessLevel::GameMaster,
    get: |w, e| match e {
        EntityId::Item(i) => Value::Int(i64::from(bx(w, i).level)),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err("not an item".to_string());
        };
        if let Some(d) = bx_mut(w, i) {
            d.level = v.i32();
        }
        Ok(())
    }),
}];

static GEAR_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| EntityId::Item(dawns_music_gear_new(w)),
    },
    Ctor {
        access: None,
        params: &[Param {
            name: "music",
            kind: MUSIC_ENUM_KIND,
            default: None,
            ty: "MusicName",
        }],
        make: |w, v| EntityId::Item(dawns_music_gear_new_music(w, v[0].i64() as i32)),
    },
];

static FOUNTAIN_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "charges",
        kind: INT,
        default: Some(Value::Int(MAX_CHARGES as i64)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(fountain_of_life_new(w, v[0].i32())),
}];

static FOUNTAIN_DEED_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "charges",
        kind: INT,
        default: Some(Value::Int(MAX_CHARGES as i64)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(fountain_of_life_deed_new(w, v[0].i32())),
}];

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &DAWNS_MUSIC_BOX,
        &DAWNS_MUSIC_GEAR,
        &ENHANCED_BANDAGE,
        &FOUNTAIN_OF_LIFE,
        &FOUNTAIN_OF_LIFE_DEED,
    ] {
        types.register_item(t, &[]);
    }
}

pub(super) fn configure(w: &mut World) {
    props::add(w, "Server.Items.DawnsMusicGear", GEAR_PROPS);
    props::add(w, "Server.Items.DawnsMusicBox", BOX_PROPS);
    props::add(w, "Server.Items.FountainOfLife", FOUNTAIN_PROPS);
    props::add(w, "Server.Items.FountainOfLifeDeed", FOUNTAIN_PROPS);
    add_ctors(w, "Server.Items.DawnsMusicGear", &GEAR_CTORS);
    add_ctors(w, "Server.Items.FountainOfLife", &FOUNTAIN_CTORS);
    add_ctors(w, "Server.Items.FountainOfLifeDeed", &FOUNTAIN_DEED_CTORS);
    muo_content::items::interfaces::register_dyable(w, &ENHANCED_BANDAGE, |_, _, _, _| false);
    house_deps::deps_mut(w).securables.push(|w, id| {
        w.item(id)
            .filter(|i| i.ty.is(&DAWNS_MUSIC_BOX))
            .map(|_| bx(w, id).level as u8)
    });
    house_deps::deps_mut(w)
        .securable_setters
        .push(|w, id, level| {
            if !w.item(id).is_some_and(|i| i.ty.is(&DAWNS_MUSIC_BOX)) {
                return false;
            }
            if let Some(d) = bx_mut(w, id) {
                d.level = i32::from(level);
            }
            true
        });
}
