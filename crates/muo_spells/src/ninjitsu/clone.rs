//! The `Clone : BaseCreature` and `CloneAI : BaseAI` of Mirror Image: the copy of the caster that follows it and vanishes when it is hurt.

use muo_content::ai::AiClass;
use muo_content::ai::base::BASE_AI;
use muo_content::ai::movement::{walk_mobile_range, walk_random};
use muo_content::creature::control;
use muo_content::creature::data::cd_mut;
use muo_content::creature::enums::{AIType, FightMode, OrderType};
use muo_content::creature::setters::new_creature;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_content::spells::hooks::hooks_mut;
use muo_content::spells::unsummon_timer;
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::item::{ITEM_BASE, base as item_base, ops};
use muo_server::map::MapId;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::items;
use muo_server::mobile::{MobileType, base as mb};
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::skills::{self, SkillName};
use muo_server::type_registry::TypeRegistry;

use super::mirror_image::{add_clone, remove_clone};

const DEFAULT_RANGE_PERCEPTION: i32 = 16;

/// `Clone._caster`.
#[derive(Clone, Copy, Debug, Default)]
struct CloneData {
    caster: Option<MobileId>,
}

fn caster_of(w: &World, id: MobileId) -> Option<MobileId> {
    w.mobile(id)
        .and_then(|m| m.ext.get::<CloneData>())
        .and_then(|d| d.caster)
}

/// `CloneAI.Think()`: clones only follow their owners.
fn think(w: &mut World, id: MobileId) -> bool {
    let master = control::summon_master(w, id);
    let range = muo_content::creature::data::read(w, id, 0, |d| d.range_perception);
    let map = w.mobile(id).and_then(|m| m.map);
    let near = master.is_some_and(|m| {
        w.mobile(m).and_then(|x| x.map) == map && muo_content::mx::in_range(w, m, id, range)
    });
    match master {
        Some(m) if near => {
            walk_mobile_range(w, id, m, 2, 0, 1);
        }
        _ => walk_random(w, id, 2, 2, 1),
    }
    true
}

/// `CloneAI`.
pub static CLONE_AI: AiClass = AiClass {
    name: "CloneAI",
    think,
    can_detect_hidden: |_, _| false,
    ..BASE_AI
};

/// `Clone(Mobile caster)`: a copy of the caster's looks, skills and worn items, summoned and following it.
pub fn new_clone(w: &mut World, caster: MobileId) -> MobileId {
    let id = new_creature(
        w,
        &CLONE,
        AIType::AiMelee,
        FightMode::None,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(CloneData {
            caster: Some(caster),
        });
    }
    // CloneAI(m): m.SetCurrentSpeedToActive()
    muo_content::creature::speeds::set_current_speed_to_active(w, id);

    let body = props_b::body(w, caster);
    props_b::set_body(w, id, body);
    let hue = mb::hue(w, caster);
    mb::set_hue(w, id, hue);

    let (female, name, name_hue, title, kills, hair, hair_hue, facial, facial_hue) = {
        let c = w.mobile(caster).expect("the caster exists");
        (
            c.female,
            c.name.clone(),
            c.name_hue,
            c.title.clone(),
            c.kills,
            c.hair_item_id,
            c.hair_hue,
            c.facial_hair_item_id,
            c.facial_hair_hue,
        )
    };
    props_b::set_female(w, id, female);
    if let Some(f) = w.mobile(id).map(|m| m.ty.set_name) {
        f(w, id, name);
    }
    if let Some(m) = w.mobile_mut(id) {
        m.name_hue = name_hue;
    }
    props_a::set_title(w, id, title);
    props_b::set_kills(w, id, kills);
    props_b::set_hair_item_id(w, id, hair);
    props_b::set_hair_hue(w, id, hair_hue);
    props_b::set_facial_hair_item_id(w, id, facial);
    props_b::set_facial_hair_hue(w, id, facial_hue);

    for i in 0..SkillName::COUNT as i32 {
        if let Some(s) = SkillName::from_i32(i) {
            let base = skills::base_fixed(w, caster, s);
            let cap = skills::cap_fixed(w, caster, s);
            skills::set_base_fixed(w, id, s, base);
            skills::set_cap_fixed(w, id, s, cap);
        }
    }

    let worn: Vec<_> = w
        .mobile(caster)
        .map(|m| m.items.clone())
        .unwrap_or_default();
    for item in worn {
        let copy = clone_item(w, item);
        items::add_item(w, id, copy);
    }

    props_b::set_warmode(w, id, true);

    control::set_summoned(w, id, true);
    control::set_master(w, id, Some(caster));

    control::set_control_order_raw(w, id, OrderType::Follow);
    if let Some(d) = cd_mut(w, id) {
        d.control_target = Some(caster);
    }

    add_clone_timer(w, id);
    id
}

/// `CloneItem(item)`: a plain `Item` with the graphic, hue and layer.
fn clone_item(w: &mut World, item: muo_server::ids::ItemId) -> muo_server::ids::ItemId {
    let (item_id, hue, layer) = w
        .item(item)
        .map_or((0, 0, Default::default()), |i| (i.item_id, i.hue, i.layer));
    let copy = ops::new_item_with_id(w, &ITEM_BASE, item_id);
    item_base::set_hue(w, copy, hue);
    item_base::set_layer(w, copy, layer);
    copy
}

/// `AddClone()`: the unsummon timer, `SummonEnd` and the caster's clone count.
fn add_clone_timer(w: &mut World, id: MobileId) {
    let Some(caster) = caster_of(w, id) else {
        return;
    };
    let ninjitsu = skills::value(w, caster, SkillName::Ninjitsu);
    let duration = TimeSpan::from_secs_f64(30.0 + ninjitsu / 4.0);

    unsummon_timer::start(w, id, duration, None);
    let end = w.now + duration;
    if let Some(d) = cd_mut(w, id) {
        d.summon_end = end;
    }

    add_clone(w, caster);
}

fn clone_on_damage(
    w: &mut World,
    id: MobileId,
    _amount: i32,
    _from: Option<MobileId>,
    _will_kill: bool,
) {
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.delete)(w, id);
    }
}

fn clone_on_delete(w: &mut World, id: MobileId) {
    let (loc, map) = w
        .mobile(id)
        .map_or((Point3D::default(), None), |m| (m.location, m.map));
    let e = muo_server::item::effect_item::create_targeted(
        w,
        loc,
        map,
        muo_server::item::effect_item::DEFAULT_DURATION,
    );
    muo_server::effects::send_location_particles(w, e, 0x3728, 10, 15, 5042);

    (BASE_CREATURE.on_delete)(w, id);
}

fn clone_on_after_delete(w: &mut World, id: MobileId) {
    if let Some(c) = caster_of(w, id) {
        remove_clone(w, c);
    }
    (BASE_CREATURE.on_after_delete)(w, id);
}

static CLONE_C: CreatureType = CreatureType {
    delete_corpse_on_death: |_, _| true,
    is_dispellable: |_, _| false,
    commandable: |_, _| false,
    is_human_in_town: |_, _| false,
    forced_ai: |_, _| Some(&CLONE_AI),
    ..BASE_CREATURE_C
};

/// `Server.Mobiles.Clone` `[SerializationGenerator(0)]`: no parameterless constructor.
pub static CLONE: MobileType = MobileType {
    full_name: "Server.Mobiles.Clone",
    parent: Some(&BASE_CREATURE),
    construct: None,
    family: Some(&CLONE_C),
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_encoded_int(0);
        out.write_mobile(w, caster_of(w, id));
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        let _version = r.read_encoded_int()?;
        let caster = r.read_mobile(w)?;
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(CloneData { caster });
        }
        // [AfterDeserialization] AddClone()
        add_clone_timer(w, id);
        Ok(())
    },
    on_damage: clone_on_damage,
    on_delete: clone_on_delete,
    on_after_delete: clone_on_after_delete,
    ..BASE_CREATURE
};

/// `new Clone(caster).MoveToWorld(caster.Location, caster.Map)`.
fn make_clone(w: &mut World, caster: MobileId) {
    let (loc, map) = w
        .mobile(caster)
        .map_or((Point3D::default(), None), |m| (m.location, m.map));
    let clone = new_clone(w, caster);
    mb::move_to_world(w, clone, loc, map.unwrap_or(MapId::INTERNAL));
}

/// Registers the class.
pub fn register(types: &mut TypeRegistry) {
    types.register_mobile(&CLONE, &[]);
}

/// Sets the hook the Mirror Image spell calls.
pub fn init(w: &mut World) {
    hooks_mut(w).new_mirror_image_clone = Some(make_clone);
}
