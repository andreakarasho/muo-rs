//! `PlantBowl`, the plant reward items (`FertileDirt`, `OrangePetals`, `RedLeaves`, `GreenThorns`) and the
//! `GiantIceWorm`.

use std::rc::Rc;

use indexmap::IndexMap;
use muo_content::creature::enums::*;
use muo_content::creature::setters::*;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::hooks::rnd;
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, effect_item, messages, ops};
use muo_server::map::MapId;
use muo_server::mobile::ResistanceType;
use muo_server::packets::message::MessageType;
use muo_server::poison::PoisonId;
use muo_server::skills::SkillName;
use muo_server::target::{self, Target, TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

use crate::ml_quests::npc_support::creature_leaf;

fn local(w: &mut World, from: MobileId, hue: i32, number: i32) {
    deps::local_overhead_message(w, from, MessageType::REGULAR, hue, number);
}

fn in_pack(w: &World, id: ItemId, from: MobileId) -> bool {
    let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    ops::is_child_of(w, id, pack)
}

fn deleted(w: &World, id: ItemId) -> bool {
    w.item(id).is_none_or(|i| i.deleted())
}

fn delete(w: &mut World, id: ItemId) {
    if let Some(ty) = w.item(id).map(|i| i.ty) {
        (ty.delete)(w, id);
    }
}

/// A class that is "parent part + version 0" (`[SerializationGenerator(0, false)]`).
macro_rules! v0_item {
    ($(#[$m:meta])* $name:ident = $full:literal; $($f:ident : $v:expr,)*) => {
        $(#[$m])*
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&ITEM_BASE),
            $($f: $v,)*
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
    };
}

/// `base(itemID); Stackable = true; Hue = hue; Amount = amount`.
fn stack_new(w: &mut World, ty: &'static ItemType, item_id: i32, hue: i32, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    ops::set_stackable(w, id, true);
    if hue != 0 {
        crate::util::hue(w, id, hue);
    }
    ops::set_amount(w, id, amount);
    id
}

// ---- FertileDirt ----

pub fn fertile_dirt_new(w: &mut World, amount: i32) -> ItemId {
    stack_new(w, &FERTILE_DIRT, 0xF81, 0, amount)
}

v0_item! {
    /// `Server.Items.FertileDirt`.
    FERTILE_DIRT = "Server.Items.FertileDirt";
    construct: Some(|w| fertile_dirt_new(w, 1)),
    default_weight: |_, _| 1.0,
}

// ---- PlantBowl ----

static DIRT_PATCH_LAND_TILES: &[i32] = &[
    0x9, 0x15, 0x71, 0x7C, 0x82, 0xA7, 0xDC, 0xE3, 0xE8, 0xEB, 0x141, 0x144, 0x14C, 0x15C, 0x169,
    0x174, 0x1DC, 0x1EF, 0x272, 0x275, 0x27E, 0x281, 0x2D0, 0x2D7, 0x2E5, 0x2FF, 0x303, 0x31F,
    0x32C, 0x32F, 0x33D, 0x340, 0x345, 0x34C, 0x355, 0x358, 0x367, 0x36E, 0x377, 0x37A, 0x38D,
    0x390, 0x395, 0x39C, 0x3A5, 0x3A8, 0x3F6, 0x405, 0x547, 0x54E, 0x553, 0x556, 0x597, 0x59E,
    0x623, 0x63A, 0x6F3, 0x6FA, 0x777, 0x791, 0x79A, 0x7A9, 0x7AE, 0x7B1, 0x98C, 0x99F, 0x9AC,
    0x9BF, 0x5B27, 0x5B3E, 0x71F4, 0x71FB, 0x72C9, 0x72CA,
];

static DIRT_PATCH_STATIC_TILES: [i32; 6] = [0x1B27, 0x1B3E, 0x31F4, 0x31FB, 0x32C9, 0x32CA];

fn in_ranges(tile: i32, ranges: &[i32]) -> bool {
    ranges
        .chunks(2)
        .any(|p| p.len() == 2 && tile >= p[0] && tile <= p[1])
}

/// `PlantBowl.IsDirtPatch(obj)`.
pub fn is_dirt_patch(w: &World, targeted: &Targeted) -> bool {
    match targeted {
        Targeted::Entity(EntityId::Item(i)) => {
            muo_content::factory::item_is(w, *i, "Server.Items.Static")
                && w.item(*i).is_some_and(|it| {
                    !it.movable() && in_ranges(it.item_id, &DIRT_PATCH_STATIC_TILES)
                })
        }
        Targeted::Static(s) => in_ranges(s.item_id, &DIRT_PATCH_STATIC_TILES),
        Targeted::Land(l) => in_ranges(l.tile_id, DIRT_PATCH_LAND_TILES),
        Targeted::Entity(_) => false,
    }
}

fn plant_bowl_on_target(w: &mut World, bowl: ItemId, from: MobileId, targeted: Targeted) {
    use muo_server::mobile::part2::items::place_in_backpack;
    if deleted(w, bowl) {
        return;
    }
    if !in_pack(w, bowl, from) {
        muo_content::mx::msg(w, from, 1042664);
        return;
    }
    let dirt = match targeted {
        Targeted::Entity(EntityId::Item(i))
            if w.item(i).is_some_and(|it| it.ty.is(&FERTILE_DIRT)) =>
        {
            Some(i)
        }
        _ => None,
    };
    if let Some(dirt) = dirt {
        let dirt_needed = if w.expansion() >= muo_data::expansion::Expansion::ML {
            20
        } else {
            40
        };
        if !in_pack(w, dirt, from) {
            muo_content::mx::msg(w, from, 1042664);
        } else if w.item(dirt).map_or(0, |i| i.amount) < dirt_needed {
            local(w, from, 0x3B2, 1061896); // You need more dirt to fill a plant bowl!
        } else {
            let full = super::item::plant_new(w, true);
            if place_in_backpack(w, from, full) {
                base::consume(w, dirt, dirt_needed);
                delete(w, bowl);
                local(w, from, 0x3B2, 1061895); // You fill the bowl with fresh dirt.
            } else {
                delete(w, full);
                local(w, from, 0x3B2, 1061894); // There is no room in your backpack for a bowl full of dirt!
            }
        }
    } else if is_dirt_patch(w, &targeted) {
        let full = super::item::plant_new(w, false);
        if place_in_backpack(w, from, full) {
            delete(w, bowl);
            local(w, from, 0x3B2, 1061895);
        } else {
            delete(w, full);
            local(w, from, 0x3B2, 1061894);
        }
    } else {
        local(w, from, 0x3B2, 1061893); // You'll want to gather fresh dirt in order to raise a healthy plant!
    }
}

v0_item! {
    /// `Server.Engines.Plants.PlantBowl`.
    PLANT_BOWL = "Server.Engines.Plants.PlantBowl";
    construct: Some(|w| ops::new_item_with_id(w, &PLANT_BOWL, 0x15FD)),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1060834, // a plant bowl
    on_double_click: |w, id, from| {
        if !in_pack(w, id, from) {
            muo_content::mx::msg(w, from, 1042664);
            return;
        }
        let mut t = Target::new(w, 3, true, TargetFlags::NONE);
        t.handlers.on_target = Some(Rc::new(move |w: &mut World, from, targeted| plant_bowl_on_target(w, id, from, targeted)));
        t.handlers.on_target_out_of_range = Some(Rc::new(|w: &mut World, from, _| local(w, from, 0x3B2, 502825)));
        let tid = target::add_target(w, t);
        target::set_mobile_target(w, from, Some(tid));
        muo_content::mx::msg(w, from, 1061897); // Choose a patch of dirt to scoop up.
    },
}

// ---- OrangePetals ----

/// `OrangePetals._table`.
#[derive(Default)]
pub struct OrangePetalsTable(IndexMap<MobileId, TimerHandle>);

/// `OrangePetals.UnderEffect(m)`.
pub fn orange_petals_under_effect(w: &World, m: MobileId) -> bool {
    w.resources
        .get::<OrangePetalsTable>()
        .is_some_and(|t| t.0.contains_key(&m))
}

/// `OrangePetals.RemoveEffect(m)`.
pub fn orange_petals_remove_effect(w: &mut World, m: MobileId) {
    if let Some(h) = w
        .resources
        .get_mut::<OrangePetalsTable>()
        .0
        .shift_remove(&m)
    {
        timer::stop(w, h);
    }
}

v0_item! {
    /// `Server.Items.OrangePetals`.
    ORANGE_PETALS = "Server.Items.OrangePetals";
    construct: Some(|w| stack_new(w, &ORANGE_PETALS, 0x1021, 0x2B, 1)),
    label_number: |_, _| 1053122, // orange petals
    default_weight: |_, _| 0.1,
    check_item_use: |w, id, from, item| {
        if item != id {
            return base::check_item_use_base(w, id, from, item);
        }
        if ops::root_parent(w, id) != Some(EntityId::Mobile(from)) {
            muo_content::mx::msg(w, from, 1042038); // You must have the object in your backpack to use it.
            return false;
        }
        base::check_item_use_base(w, id, from, item)
    },
    on_double_click: |w, id, from| {
        if orange_petals_under_effect(w, from) {
            local(w, from, 0x3B2, 1061904); // * You already feel resilient! You decide to save the petal for later *
            return;
        }
        local(w, from, 0x3B2, 1061905); // * You eat the orange petal.  You feel more resilient! *
        muo_content::mx::play_sound(w, from, 0x3B);
        let h = timer::start(w, TimeSpan::from_mins(5), TimeSpan::ZERO, 1, move |w| {
            if !muo_content::mx::deleted(w, from) {
                local(w, from, 0x3F, 1053091); // * You feel the effects of your poison resistance wearing off *
            }
            w.resources.get_mut::<OrangePetalsTable>().0.shift_remove(&from);
        });
        // `_table[from] = timer` replaces (without stopping) an older timer.
        w.resources.get_mut::<OrangePetalsTable>().0.insert(from, h);
        base::consume(w, id, 1);
    },
}

// ---- RedLeaves ----

v0_item! {
    /// `Server.Items.RedLeaves`.
    RED_LEAVES = "Server.Items.RedLeaves";
    construct: Some(|w| stack_new(w, &RED_LEAVES, 0x1E85, 0x21, 1)),
    label_number: |_, _| 1053123, // red leaves
    default_weight: |_, _| 0.1,
    on_double_click: |w, id, from| {
        if !in_pack(w, id, from) {
            muo_content::mx::msg(w, from, 1042664);
            return;
        }
        let mut t = Target::new(w, 3, false, TargetFlags::NONE);
        t.handlers.on_target = Some(Rc::new(move |w: &mut World, from, targeted| red_leaves_on_target(w, id, from, targeted)));
        let tid = target::add_target(w, t);
        target::set_mobile_target(w, from, Some(tid));
        muo_content::mx::msg(w, from, 1061907); // Choose a book you wish to seal with the wax from the red leaf.
    },
}

fn red_leaves_on_target(w: &mut World, leaves: ItemId, from: MobileId, targeted: Targeted) {
    if deleted(w, leaves) {
        return;
    }
    if !in_pack(w, leaves, from) {
        muo_content::mx::msg(w, from, 1042664);
        return;
    }
    let item = match targeted {
        Targeted::Entity(EntityId::Item(i)) if in_pack(w, i, from) => i,
        _ => {
            muo_content::mx::msg(w, from, 1042664);
            return;
        }
    };
    if !muo_items::books::is_book(w, item) {
        ops::label_to_number(w, item, from, 1061911, ""); // You can only use red leaves to seal the ink into book pages!
    } else if !muo_items::books::data(w, item).is_some_and(|d| d.writable) {
        ops::label_to_number(w, item, from, 1061909, ""); // The ink in this book has already been sealed.
    } else {
        base::consume(w, leaves, 1);
        if let Some(d) = muo_items::books::data_mut(w, item) {
            d.writable = false;
        }
        ops::label_to_number(w, item, from, 1061910, ""); // You seal the ink to the page using wax from the red leaf.
    }
}

// ---- GreenThorns ----

const GREEN_THORNS_KEY: &str = "Server.Items.GreenThorns";

v0_item! {
    /// `Server.Items.GreenThorns`.
    GREEN_THORNS = "Server.Items.GreenThorns";
    construct: Some(|w| stack_new(w, &GREEN_THORNS, 0xF42, 0x42, 1)),
    default_weight: |_, _| 1.0,
    label_number: |_, _| 1060837, // green thorns
    on_double_click: |w, id, from| {
        if !in_pack(w, id, from) {
            muo_content::mx::msg(w, from, 1042664);
            return;
        }
        if !muo_server::mobile::part1::combat::can_begin_action(w, from, GREEN_THORNS_KEY) {
            local(w, from, 0x3B2, 1061908); // * You must wait a while before planting another thorn. *
            return;
        }
        let mut t = Target::new(w, 3, true, TargetFlags::NONE);
        t.handlers.on_target = Some(Rc::new(move |w: &mut World, from, targeted| green_thorns_on_target(w, id, from, targeted)));
        t.handlers.on_target_out_of_range = Some(Rc::new(|w: &mut World, from, _| local(w, from, 0x3B2, 502825)));
        let tid = target::add_target(w, t);
        target::set_mobile_target(w, from, Some(tid));
        muo_content::mx::msg(w, from, 1061906); // Choose a spot to plant the thorn.
    },
}

fn green_thorns_on_target(w: &mut World, thorn: ItemId, from: MobileId, targeted: Targeted) {
    use muo_server::mobile::part1::combat::{begin_action, can_begin_action, end_action};
    if deleted(w, thorn) {
        return;
    }
    if !in_pack(w, thorn, from) {
        muo_content::mx::msg(w, from, 1042664);
        return;
    }
    if !can_begin_action(w, from, GREEN_THORNS_KEY) {
        local(w, from, 0x3B2, 1061908);
        return;
    }
    let map = w.mobile(from).and_then(|m| m.map);
    if map != Some(MapId::TRAMMEL) && map != Some(MapId::FELUCCA) {
        muo_server::mobile::part2::messages::local_overhead_message(
            w,
            from,
            muo_primitives::MessageType::REGULAR,
            0x2B2,
            true,
            "No solen lairs exist on this facet.  Try again in Trammel or Felucca.",
        );
        return;
    }
    let Targeted::Land(land) = targeted else {
        local(w, from, 0x3B2, 1061912); // * You cannot plant a green thorn there! *
        return;
    };
    let Some(map) = map else { return };
    match thorns_effect_kind(w, map, land.location, land.tile_id) {
        None => local(w, from, 0x3B2, 1061913), // * You sense it would be useless to plant a green thorn there. *
        Some(kind) => {
            base::consume(w, thorn, 1);
            local_emote(w, from, 1061914); // * You push the strange green thorn into the ground *
            let name = w
                .mobile(from)
                .and_then(|m| (m.ty.name)(w, from))
                .unwrap_or_default();
            muo_server::mobile::part2::messages::nonlocal_overhead_message_number(
                w,
                from,
                muo_primitives::MessageType::EMOTE,
                0x961,
                1061915,
                &name,
            ); // * ~1_PLAYER_NAME~ pushes a strange green thorn into the ground. *
            begin_action(w, from, GREEN_THORNS_KEY);
            timer::start(w, TimeSpan::from_mins(3), TimeSpan::ZERO, 1, move |w| {
                end_action(w, from, GREEN_THORNS_KEY)
            });
            let fx = ThornsEffect {
                kind,
                location: land.location,
                map,
                from,
            };
            timer::start(
                w,
                TimeSpan::from_millis(2500),
                TimeSpan::ZERO,
                1,
                move |w| fx.tick(w, 0),
            );
        }
    }
}

fn local_emote(w: &mut World, from: MobileId, number: i32) {
    muo_server::mobile::part2::messages::local_overhead_message_number(
        w,
        from,
        muo_primitives::MessageType::EMOTE,
        0x961,
        number,
        "",
    );
}

#[derive(Copy, Clone)]
enum ThornsKind {
    Dirt,
    Furrows,
    Swamp,
    Snow,
    Sand,
}

static THORNS_TABLE: [(&[i32], ThornsKind); 5] = [
    (
        &[
            0x71, 0x7C, 0x82, 0xA7, 0xDC, 0xE3, 0xE8, 0xEB, 0x141, 0x144, 0x14C, 0x14F, 0x169,
            0x174, 0x1DC, 0x1E7, 0x1EC, 0x1EF, 0x272, 0x275, 0x27E, 0x281, 0x2D0, 0x2D7, 0x2E5,
            0x2FF, 0x303, 0x31F, 0x32C, 0x32F, 0x33D, 0x340, 0x345, 0x34C, 0x355, 0x358, 0x367,
            0x36E, 0x377, 0x37A, 0x38D, 0x390, 0x395, 0x39C, 0x3A5, 0x3A8, 0x3F6, 0x405, 0x547,
            0x54E, 0x553, 0x556, 0x597, 0x59E, 0x623, 0x63A, 0x6F3, 0x6FA, 0x777, 0x791, 0x79A,
            0x7A9, 0x7AE, 0x7B1,
        ],
        ThornsKind::Dirt,
    ),
    (&[0x9, 0x15, 0x150, 0x15C], ThornsKind::Furrows),
    (
        &[
            0x9C4, 0x9EB, 0x3D65, 0x3D65, 0x3DC0, 0x3DD9, 0x3DDB, 0x3DDC, 0x3DDE, 0x3EF0, 0x3FF6,
            0x3FF6, 0x3FFC, 0x3FFE,
        ],
        ThornsKind::Swamp,
    ),
    (
        &[
            0x10C, 0x10F, 0x114, 0x117, 0x119, 0x11D, 0x179, 0x18A, 0x385, 0x38C, 0x391, 0x394,
            0x39D, 0x3A4, 0x3A9, 0x3AC, 0x5BF, 0x5D6, 0x5DF, 0x5E2, 0x745, 0x748, 0x751, 0x758,
            0x75D, 0x760, 0x76D, 0x773,
        ],
        ThornsKind::Snow,
    ),
    (
        &[
            0x16, 0x3A, 0x44, 0x4B, 0x11E, 0x121, 0x126, 0x12D, 0x192, 0x192, 0x1A8, 0x1AB, 0x1B9,
            0x1D1, 0x282, 0x285, 0x28A, 0x291, 0x335, 0x33C, 0x341, 0x344, 0x34D, 0x354, 0x359,
            0x35C, 0x3B7, 0x3BE, 0x3C7, 0x3CA, 0x5A7, 0x5B2, 0x64B, 0x652, 0x657, 0x65A, 0x663,
            0x66A, 0x66F, 0x672, 0x7BD, 0x7D0,
        ],
        ThornsKind::Sand,
    ),
];

/// `GreenThornsEffect.Create(from, land)`: the effect's class, None when it is "useless to plant a green thorn there".
fn thorns_effect_kind(
    w: &mut World,
    map: MapId,
    location: Point3D,
    tile_id: i32,
) -> Option<ThornsKind> {
    if !muo_server::map::fit::can_spawn_mobile_at(w, map, location.x, location.y, location.z) {
        return None;
    }
    THORNS_TABLE
        .iter()
        .find(|(tiles, _)| in_ranges(tile_id, tiles))
        .map(|(_, k)| *k)
}

/// `GreenThornsEffect`: a timer that plays the next step after the delay the last one returned.
#[derive(Copy, Clone)]
struct ThornsEffect {
    kind: ThornsKind,
    location: Point3D,
    map: MapId,
    from: MobileId,
}

impl ThornsEffect {
    fn tick(self, w: &mut World, step: i32) {
        let next = self.play(w, step);
        if next > TimeSpan::ZERO {
            timer::start(w, next, TimeSpan::ZERO, 1, move |w| self.tick(w, step + 1));
        }
    }

    fn sound(&self, w: &mut World, id: i32) {
        muo_server::effects::play_sound(w, self.location, Some(self.map), id);
    }

    fn puff(&self, w: &mut World) {
        self.sound(w, 0x106);
        let e = effect_item::create_targeted(
            w,
            self.location,
            Some(self.map),
            effect_item::DEFAULT_DURATION,
        );
        muo_server::effects::send_location_particles(w, e, 0x3735, 1, 182, 0xBE3);
    }

    fn dummy(&self, w: &mut World, secs: i64, message: i32) -> ItemId {
        let d = effect_item::create(w, self.location, Some(self.map), TimeSpan::from_secs(secs));
        messages::public_overhead_message_number(w, d, MessageType::REGULAR, 0x3B2, message, "");
        d
    }

    fn play(&self, w: &mut World, step: i32) -> TimeSpan {
        let secs = TimeSpan::from_secs_f64;
        match self.kind {
            ThornsKind::Dirt => match step {
                0 => {
                    self.puff(w);
                    secs(4.0)
                }
                1 => {
                    self.sound(w, 0x222);
                    secs(4.0)
                }
                2 => {
                    self.sound(w, 0x21F);
                    secs(5.0)
                }
                3..=5 => {
                    if step == 3 {
                        self.dummy(w, 20, 1114432); // * The ground erupts with chaotic growth! *
                    }
                    self.sound(w, 0x12D);
                    self.spawn_reagents(w);
                    self.spawn_reagents(w);
                    secs([2.0, 2.0, 3.0][(step - 3) as usize])
                }
                _ => {
                    self.sound(w, 0x12D);
                    self.spawn_reagents(w);
                    self.spawn_reagents(w);
                    TimeSpan::ZERO
                }
            },
            ThornsKind::Furrows => match step {
                0 => {
                    self.puff(w);
                    secs(4.0)
                }
                1 => {
                    let hole = effect_item::create(
                        w,
                        self.location,
                        Some(self.map),
                        TimeSpan::from_secs(10),
                    );
                    if let Some(ty) = w.item(hole).map(|i| i.ty) {
                        (ty.set_item_id)(w, hole, 0x913);
                    }
                    self.sound(w, 0x222);
                    secs(4.0)
                }
                2 => {
                    self.sound(w, 0x21F);
                    secs(4.0)
                }
                _ => {
                    self.dummy(w, 20, 1114428); // * A magical bunny leaps out of its hole, disturbed by the thorn's effect! *
                    self.spawn_creature_named(w, "Server.Mobiles.VorpalBunny");
                    TimeSpan::ZERO
                }
            },
            ThornsKind::Swamp => match step {
                0 => {
                    self.puff(w);
                    secs(4.0)
                }
                1 => {
                    self.sound(w, 0x222);
                    secs(4.0)
                }
                2 => {
                    self.sound(w, 0x21F);
                    secs(1.0)
                }
                _ => {
                    self.dummy(w, 20, 1114429); // * Strange green tendrils rise from the ground, whipping wildly! *
                    self.sound(w, 0x2B0);
                    self.spawn_creature_named(w, "Server.Mobiles.WhippingVine");
                    TimeSpan::ZERO
                }
            },
            ThornsKind::Snow => match step {
                0 => {
                    self.puff(w);
                    secs(4.0)
                }
                1 => {
                    self.sound(w, 0x222);
                    secs(4.0)
                }
                2 => {
                    self.sound(w, 0x21F);
                    secs(4.0)
                }
                _ => {
                    self.dummy(w, 20, 1114430); // * Slithering ice serpents rise to the surface to investigate the disturbance! *
                    self.spawn_creature_named(w, "Server.Mobiles.GiantIceWorm");
                    for _ in 0..3 {
                        self.spawn_creature_named(w, "Server.Mobiles.IceSnake");
                    }
                    TimeSpan::ZERO
                }
            },
            ThornsKind::Sand => match step {
                0 => {
                    self.puff(w);
                    secs(4.0)
                }
                1 => {
                    self.sound(w, 0x222);
                    secs(4.0)
                }
                2 => {
                    self.sound(w, 0x21F);
                    secs(5.0)
                }
                _ => {
                    self.dummy(w, 20, 1114431); // * The sand collapses, revealing a dark hole. *
                    sh_teleporter_create(w, self.location, self.map);
                    TimeSpan::ZERO
                }
            },
        }
    }

    fn spawn_reagents(&self, w: &mut World) {
        let amount = rnd::min_max(w, 10, 25);
        let name = match rnd::below(w, 9) {
            0 => "Server.Items.BlackPearl",
            1 => "Server.Items.Bloodmoss",
            2 => "Server.Items.Garlic",
            3 => "Server.Items.Ginseng",
            4 => "Server.Items.MandrakeRoot",
            5 => "Server.Items.Nightshade",
            6 => "Server.Items.SulfurousAsh",
            7 => "Server.Items.SpidersSilk",
            _ => "Server.Items.FertileDirt",
        };
        let Some(item) =
            muo_content::factory::make_item_args(w, name, &[Value::Int(i64::from(amount))])
        else {
            return;
        };
        if !self.spawn_item(w, item) {
            delete(w, item);
        }
    }

    /// `SpawnItem(item)`.
    fn spawn_item(&self, w: &mut World, item: ItemId) -> bool {
        for _ in 0..5 {
            let x = self.location.x + rnd::min_max(w, -1, 1);
            let y = self.location.y + rnd::min_max(w, -1, 1);
            let z = w.maps.get_average_z(self.map, x, y);
            if muo_server::map::fit::can_fit(
                w,
                self.map,
                x,
                y,
                self.location.z,
                1,
                false,
                true,
                true,
            ) {
                ops::move_to_world_map(
                    w,
                    item,
                    Point3D::new(x, y, self.location.z),
                    Some(self.map),
                );
                return true;
            }
            if muo_server::map::fit::can_fit(w, self.map, x, y, z, 1, false, true, true) {
                ops::move_to_world_map(w, item, Point3D::new(x, y, z), Some(self.map));
                return true;
            }
        }
        false
    }

    fn spawn_creature_named(&self, w: &mut World, name: &str) {
        let Some(c) = muo_content::factory::make_mobile(w, name) else {
            return;
        };
        if !self.spawn_creature(w, c)
            && let Some(ty) = w.mobile(c).map(|m| m.ty)
        {
            (ty.delete)(w, c);
        }
    }

    /// `SpawnCreature(creature)`.
    fn spawn_creature(&self, w: &mut World, creature: MobileId) -> bool {
        for _ in 0..5 {
            let x = self.location.x + rnd::min_max(w, -1, 1);
            let y = self.location.y + rnd::min_max(w, -1, 1);
            let z = w.maps.get_average_z(self.map, x, y);
            let z_use =
                if muo_server::map::fit::can_spawn_mobile_at(w, self.map, x, y, self.location.z) {
                    Some(self.location.z)
                } else if muo_server::map::fit::can_spawn_mobile_at(w, self.map, x, y, z) {
                    Some(z)
                } else {
                    None
                };
            if let Some(zz) = z_use {
                muo_server::mobile::base::move_to_world(
                    w,
                    creature,
                    Point3D::new(x, y, zz),
                    self.map,
                );
                muo_server::mobile::base::set_combatant(w, creature, Some(self.from));
                return true;
            }
        }
        false
    }
}

// ---- GreenThornsSHTeleporter ----

const SH_DESTINATION: Point3D = Point3D::new(5738, 1856, 0);

fn sh_teleporter_create(w: &mut World, location: Point3D, map: MapId) {
    let tele = ops::new_item_with_id(w, &GREEN_THORNS_SH_TELEPORTER, 0x913);
    ops::set_movable(w, tele, false);
    crate::util::hue(w, tele, 1);
    ops::move_to_world_map(w, tele, location, Some(map));
    timer::start(w, TimeSpan::from_mins(1), TimeSpan::ZERO, 1, move |w| {
        delete(w, tele)
    });
}

v0_item! {
    /// `Server.Items.GreenThornsSHTeleporter` (never saved).
    GREEN_THORNS_SH_TELEPORTER = "Server.Items.GreenThornsSHTeleporter";
    construct: None,
    default_name: |_, _| Some("a hole".to_string()),
    skip_serialization: |_, _| true,
    on_double_click: |w, id, from| {
        if muo_content::mx::in_range_point(w, from, ops::get_world_location(w, id), 3) {
            let map = w.item(id).and_then(|i| i.map);
            if let Some(map) = map {
                muo_content::creature::pets::teleport_pets(w, from, SH_DESTINATION, map, false);
            }
            muo_server::mobile::part1::lifecycle::set_location(w, from, SH_DESTINATION);
        } else {
            muo_content::mx::msg(w, from, 1019045); // I can't reach that.
        }
    },
}

// ---- GiantIceWorm ----

fn giant_ice_worm_new(w: &mut World) -> MobileId {
    let id = new_creature(
        w,
        &GIANT_ICE_WORM,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    muo_server::mobile::part1::props_b::set_body(w, id, muo_primitives::Body(89));
    if let Some(m) = w.mobile_mut(id) {
        m.base_sound_id = 0xDC;
    }
    set_str_range(w, id, 216, 245);
    set_dex_range(w, id, 76, 100);
    set_int_range(w, id, 66, 85);
    set_hits_range(w, id, 130, 147);
    set_damage(w, id, 7, 17);
    set_damage_type(w, id, ResistanceType::Physical, 10);
    set_damage_type(w, id, ResistanceType::Cold, 90);
    set_resistance_range(w, id, ResistanceType::Physical, 30, 35);
    set_resistance(w, id, ResistanceType::Fire, 0);
    set_resistance_range(w, id, ResistanceType::Cold, 80, 90);
    set_resistance_range(w, id, ResistanceType::Poison, 15, 25);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 20);
    set_skill_range(w, id, SkillName::Poisoning, 75.1, 95.0);
    set_skill_range(w, id, SkillName::MagicResist, 45.1, 60.0);
    set_skill_range(w, id, SkillName::Tactics, 75.1, 80.0);
    set_skill_range(w, id, SkillName::Wrestling, 60.1, 80.0);
    muo_server::mobile::part1::props_a::set_fame(w, id, 4500);
    muo_server::mobile::part1::props_a::set_karma(w, id, -4500);
    muo_server::mobile::part1::props_a::set_virtual_armor(w, id, 40);
    if let Some(d) = muo_content::creature::data::cd_mut(w, id) {
        d.tamable_raw = true;
        d.control_slots = 1;
        d.min_tame_skill = 71.1;
    }
    id
}

creature_leaf! {
    /// `Server.Mobiles.GiantIceWorm`.
    GIANT_ICE_WORM, GIANT_ICE_WORM_C = "Server.Mobiles.GiantIceWorm" : BASE_CREATURE / BASE_CREATURE_C, ctor giant_ice_worm_new, encoded false;
    c {
        corpse_name: |_w, _id| Some("a giant ice worm corpse"),
        subdue_before_tame: |_w, _id| true,
        default_name: |_w, _id| Some("a giant ice worm"),
        poison_immune: |_w, _id| Some(PoisonId(2)),
        hit_poison: |_w, _id| Some(PoisonId(2)),
        favorite_food: |_w, _id| FoodType::MEAT,
    }
    m {
    }
}

// ---- ctors and registration ----

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));

fn amount_param() -> [Param; 1] {
    [Param {
        name: "amount",
        kind: INT,
        default: Some(Value::Int(1)),
        ty: "",
    }]
}

static AMOUNT_PARAMS: std::sync::LazyLock<[Param; 1]> = std::sync::LazyLock::new(amount_param);
static BOOL_PARAMS: [Param; 1] = [Param {
    name: "fertileDirt",
    kind: Kind::Plain(PropType::Bool),
    default: Some(Value::Bool(false)),
    ty: "",
}];

macro_rules! amount_ctor {
    ($name:ident, $ty:expr, $item_id:expr, $hue:expr) => {
        static $name: std::sync::LazyLock<[Ctor; 1]> = std::sync::LazyLock::new(|| {
            [Ctor {
                access: None,
                params: &*AMOUNT_PARAMS,
                make: |w, v| EntityId::Item(stack_new(w, &$ty, $item_id, $hue, v[0].i32())),
            }]
        });
    };
}

amount_ctor!(FERTILE_DIRT_CTORS, FERTILE_DIRT, 0xF81, 0);
amount_ctor!(ORANGE_PETALS_CTORS, ORANGE_PETALS, 0x1021, 0x2B);
amount_ctor!(RED_LEAVES_CTORS, RED_LEAVES, 0x1E85, 0x21);
amount_ctor!(GREEN_THORNS_CTORS, GREEN_THORNS, 0xF42, 0x42);

static PLANT_ITEM_CTORS: [Ctor; 1] = [Ctor {
    access: None,
    params: &BOOL_PARAMS,
    make: |w, v| EntityId::Item(super::item::plant_new(w, v[0].i64() != 0)),
}];

pub fn register(types: &mut TypeRegistry) {
    for (t, aliases) in [
        (&FERTILE_DIRT, &[][..]),
        (&PLANT_BOWL, &[]),
        (&ORANGE_PETALS, &[]),
        (&RED_LEAVES, &[]),
        (&GREEN_THORNS, &[]),
        (&GREEN_THORNS_SH_TELEPORTER, &[]),
    ] {
        types.register_item(t, aliases);
    }
    types.register_mobile(&GIANT_ICE_WORM, &[]);
    types.register_phase(
        Phase::Configure,
        "Plants.MiscCtors",
        DEFAULT_CALL_PRIORITY,
        |w| {
            add_ctors(w, "Server.Items.FertileDirt", &*FERTILE_DIRT_CTORS);
            add_ctors(w, "Server.Items.OrangePetals", &*ORANGE_PETALS_CTORS);
            add_ctors(w, "Server.Items.RedLeaves", &*RED_LEAVES_CTORS);
            add_ctors(w, "Server.Items.GreenThorns", &*GREEN_THORNS_CTORS);
            add_ctors(w, "Server.Engines.Plants.PlantItem", &PLANT_ITEM_CTORS);
        },
    );
}
