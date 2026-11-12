//! Changeling and its paragons Guile, Irk and Spite (monsters5): copies the look of the player it fights.

use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, cd};
use muo_content::mx;
use muo_io::{GenericReader, GenericWriter};
use muo_primitives::{Body, DateTime, TimeSpan};
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base as ibase, ops};
use muo_server::layer::Layer;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{effects, items};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::type_registry::TypeRegistry;

use crate::common::*;

/// `m_FireNorth` / `m_FireEast`.
const FIRE_NORTH: [i32; 8] = [-1, -1, 1, -1, -1, 2, 1, 2];
const FIRE_EAST: [i32; 4] = [-1, 0, 2, 0];

/// `_morphedInto`, `m_LastMorph`, `m_NextFireRing`.
#[derive(Clone, Copy)]
pub struct ChangelingData {
    pub morphed_into: Option<MobileId>,
    pub last_morph: DateTime,
    pub next_fire_ring: DateTime,
}
impl Default for ChangelingData {
    fn default() -> Self {
        ChangelingData {
            morphed_into: None,
            last_morph: DateTime::MIN,
            next_fire_ring: DateTime::MIN,
        }
    }
}

fn data(w: &World, id: MobileId) -> ChangelingData {
    w.mobile(id)
        .and_then(|m| m.ext.get::<ChangelingData>())
        .copied()
        .unwrap_or_default()
}
fn set_data(w: &mut World, id: MobileId, d: ChangelingData) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(d);
    }
}

fn default_hue_of(w: &World, id: MobileId) -> i32 {
    muo_content::creature::ctype(w, id).map_or(0, |c| (c.default_hue)(w, id))
}

/// `Hue = DefaultHue;`
pub fn changeling_ctor1(w: &mut World, id: MobileId) {
    set_data(w, id, ChangelingData::default());
    let h = default_hue_of(w, id);
    hue(w, id, h);
}

pub fn changeling_serialize(w: &World, id: MobileId, out: &mut dyn GenericWriter) {
    out.write_i32(0);
    out.write_mobile(w, data(w, id).morphed_into);
}

/// `[AfterDeserialization(false)]`: a changeling that was disguised looks like itself again.
pub fn changeling_deserialize(
    w: &mut World,
    id: MobileId,
    r: &mut dyn GenericReader,
) -> muo_io::Result<()> {
    let _version = r.read_i32()?;
    let morphed_into = r.read_mobile(w)?;
    set_data(
        w,
        id,
        ChangelingData {
            morphed_into,
            ..ChangelingData::default()
        },
    );
    if morphed_into.is_some() {
        changeling_revert(w, id);
    }
    Ok(())
}

pub fn changeling_initial_innocent(w: &World, id: MobileId) -> bool {
    data(w, id).morphed_into.is_some()
}

/// `MorphedInto` set.
pub fn changeling_set_morphed_into(w: &mut World, id: MobileId, mut value: Option<MobileId>) {
    if value == Some(id) {
        value = None;
    }
    if data(w, id).morphed_into != value {
        changeling_revert(w, id);
        if let Some(v) = value {
            changeling_morph(w, id, v);
            let now = w.now;
            let mut d = data(w, id);
            d.last_morph = now;
            set_data(w, id, d);
        }
        let mut d = data(w, id);
        d.morphed_into = value;
        set_data(w, id, d);
        // Delta(MobileDelta.Noto)
        muo_server::mobile::base::delta(w, id, muo_server::mobile::MobileDelta::NOTO);
    }
}

pub fn changeling_on_think(w: &mut World, id: MobileId) {
    (BASE_CREATURE_C.on_think)(w, id);
    if let Some(c) = mx::combatant(w, id) {
        if data(w, id).next_fire_ring <= w.now && rnd::double(w) < 0.02 {
            changeling_fire_ring(w, id);
            let next = w.now + TimeSpan::from_mins(2);
            let mut d = data(w, id);
            d.next_fire_ring = next;
            set_data(w, id, d);
        }
        if mx::player(w, c) && data(w, id).morphed_into != Some(c) && rnd::double(w) < 0.05 {
            changeling_set_morphed_into(w, id, Some(c));
        }
    }
}

pub fn changeling_check_idle(w: &mut World, id: MobileId) -> bool {
    let idle = (BASE_CREATURE_C.check_idle)(w, id);
    let d = data(w, id);
    if idle && d.morphed_into.is_some() && w.now - d.last_morph > TimeSpan::from_secs(30) {
        changeling_set_morphed_into(w, id, None);
    }
    idle
}

fn fire_effects(w: &mut World, id: MobileId, item_id: i32, offsets: &[i32]) {
    for pair in offsets.chunks(2) {
        let mut p = mx::location(w, id);
        p.x += pair[0];
        p.y += pair[1];
        let map = mx::map(w, id);
        if muo_content::spells::helper::adjust_field(w, &mut p, map, 12, false) {
            muo_server::effects::send_location_effect(w, p, map, item_id, 50, 10, 0, 0);
        }
    }
}

/// `FireRing()`.
pub fn changeling_fire_ring(w: &mut World, id: MobileId) {
    fire_effects(w, id, 0x3E27, &FIRE_NORTH);
    fire_effects(w, id, 0x3E31, &FIRE_EAST);
}

/// `Morph(m)`: the body, name, hair and worn items of `m`.
pub fn changeling_morph(w: &mut World, id: MobileId, m: MobileId) {
    let Some(src) = w.mobile(m) else { return };
    let (b, h, female, nm, name_hue, title, kills) = (
        props_b::body(w, m),
        (src.ty.hue)(w, m),
        src.female,
        (src.ty.name)(w, m),
        src.name_hue,
        src.title.clone(),
        src.kills,
    );
    let (hair, hair_hue, facial, facial_hue) = (
        src.hair_item_id,
        src.hair_hue,
        src.facial_hair_item_id,
        src.facial_hair_hue,
    );
    let worn = src.items.clone();
    props_b::set_body(w, id, b);
    muo_server::mobile::base::set_hue(w, id, h);
    props_b::set_female(w, id, female);
    name(w, id, nm);
    if let Some(x) = w.mobile_mut(id) {
        x.name_hue = name_hue;
    }
    props_a::set_title(w, id, title);
    props_b::set_kills(w, id, kills);
    props_b::set_hair_item_id(w, id, hair);
    props_b::set_hair_hue(w, id, hair_hue);
    props_b::set_facial_hair_item_id(w, id, facial);
    props_b::set_facial_hair_hue(w, id, facial_hue);
    for item in worn {
        let layer = ibase::layer(w, item);
        if layer != Layer::Backpack && layer != Layer::Mount && layer != Layer::Bank {
            let c = new_cloned_item(w, item);
            items::add_item(w, id, c);
        }
    }
    mx::play_sound(w, id, 0x511);
    effects::fixed_particles(w, id, 0x376A, 1, 14, 5045, EffectLayer::Waist);
}

/// `Revert()`.
pub fn changeling_revert(w: &mut World, id: MobileId) {
    props_b::set_body(w, id, Body(264));
    let paragon = cd(w, id).is_some_and(|d| d.is_paragon);
    let default = default_hue_of(w, id);
    let h = if paragon && default == 0 {
        PARAGON_HUE
    } else {
        default
    };
    muo_server::mobile::base::set_hue(w, id, h);
    props_b::set_female(w, id, false);
    name(w, id, None);
    if let Some(x) = w.mobile_mut(id) {
        x.name_hue = -1;
    }
    props_a::set_title(w, id, None);
    props_b::set_kills(w, id, 0);
    props_b::set_hair_item_id(w, id, 0);
    props_b::set_hair_hue(w, id, 0);
    props_b::set_facial_hair_item_id(w, id, 0);
    props_b::set_facial_hair_hue(w, id, 0);
    changeling_delete_cloned_items(w, id);
    mx::play_sound(w, id, 0x511);
    effects::fixed_particles(w, id, 0x376A, 1, 14, 5045, EffectLayer::Waist);
}

/// `Paragon.Hue`.
const PARAGON_HUE: i32 = 0x501;

/// `DeleteClonedItems()`: the worn ones and the ones in the pack.
pub fn changeling_delete_cloned_items(w: &mut World, id: MobileId) {
    let is_cloned = |w: &World, i: ItemId| w.item(i).is_some_and(|x| x.ty.is(&CLONED_ITEM));
    let worn: Vec<ItemId> = w.mobile(id).map(|m| m.items.clone()).unwrap_or_default();
    for i in worn.into_iter().rev() {
        if is_cloned(w, i) {
            ibase::delete(w, i);
        }
    }
    if let Some(pack) = props_b::backpack(w, id) {
        let inside: Vec<ItemId> = w.item(pack).map(|p| p.items.clone()).unwrap_or_default();
        for i in inside.into_iter().rev() {
            if is_cloned(w, i) {
                ibase::delete(w, i);
            }
        }
    }
}

pub fn changeling_on_after_delete(w: &mut World, id: MobileId) {
    changeling_delete_cloned_items(w, id);
    (BASE_CREATURE.on_after_delete)(w, id);
}

/// `ClearHands()`: nothing (a changeling keeps what it holds).
pub fn changeling_clear_hands(_w: &mut World, _id: MobileId) {}

// ---- Changeling.ClonedItem ----

/// `new ClonedItem(item)`: the look of `item`, not movable.
pub fn new_cloned_item(w: &mut World, src: ItemId) -> ItemId {
    let id = ops::new_item_with_id(w, &CLONED_ITEM, ibase::item_id(w, src));
    let nm = ops::name(w, src);
    ops::set_name(w, id, nm);
    let weight = ops::weight(w, src);
    ops::set_weight(w, id, weight);
    let h = ibase::hue(w, src);
    ibase::set_hue(w, id, h);
    let layer = ibase::layer(w, src);
    ibase::set_layer(w, id, layer);
    ops::set_movable(w, id, false);
    id
}

pub static CLONED_ITEM: ItemType = ItemType {
    full_name: "Server.Mobiles.Changeling+ClonedItem",
    parent: Some(&ITEM_BASE),
    construct: None,
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..ITEM_BASE
};

pub fn register_cloned_item(types: &mut TypeRegistry) {
    types.register_item(&CLONED_ITEM, &[]);
}
