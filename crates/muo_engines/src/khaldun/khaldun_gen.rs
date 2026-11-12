//! `[GenKhaldun`: places the Khaldun decoration.

use muo_items::misc::basic::{MORPH_ITEM, MorphData, morph_item_new};
use muo_items::misc::machines::{
    EFFECT_CONTROLLER, EffectTriggerType, effect_controller_new, effect_controller_set_sound_id,
    effect_controller_set_trigger_range, effect_controller_set_trigger_type,
};
use muo_primitives::{Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::ItemId;
use muo_server::item::{ItemType, ops};
use muo_server::light_type::LightType;
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;

use super::items::{
    DISAPPEARING_RAISE_SWITCH, RAISABLE_ITEM, RAISE_SWITCH, raisable_new, set_raisable_item,
    switch_new,
};

const FELUCCA: MapId = MapId::FELUCCA;

#[derive(Default)]
struct Gen {
    count: i32,
}

fn move_to(w: &mut World, item: ItemId, x: i32, y: i32, z: i32) {
    ops::move_to_world_map(w, item, Point3D::new(x, y, z), Some(FELUCCA));
}

impl Gen {
    fn find_morph_item(
        &self,
        w: &World,
        x: i32,
        y: i32,
        z: i32,
        inactive: i32,
        active: i32,
    ) -> bool {
        muo_server::map::enumerate::items_at(w, Some(FELUCCA), x, y).any(|i| {
            w.item(i).is_some_and(|it| {
                it.ty.is(&MORPH_ITEM)
                    && it.location.z == z
                    && it.ext.get::<MorphData>().is_some_and(|d| {
                        d.inactive_item_id == inactive && d.active_item_id == active
                    })
            })
        })
    }

    fn find_effect_controller(&self, w: &World, x: i32, y: i32, z: i32) -> bool {
        muo_server::map::enumerate::items_at(w, Some(FELUCCA), x, y).any(|i| {
            w.item(i)
                .is_some_and(|it| it.ty.is(&EFFECT_CONTROLLER) && it.location.z == z)
        })
    }

    /// `TryCreateItem(x, y, z, srcItem)`: the item already at (x, y) of the same class, else `src` placed.
    fn try_create_item(
        &mut self,
        w: &mut World,
        x: i32,
        y: i32,
        z: i32,
        src: ItemId,
        ty: &'static ItemType,
    ) -> ItemId {
        let found = muo_server::map::enumerate::items_at(w, Some(FELUCCA), x, y)
            .find(|&i| w.item(i).is_some_and(|it| it.ty.is(ty)));
        if let Some(item) = found {
            if let Some(t) = w.item(src).map(|i| i.ty) {
                (t.delete)(w, src);
            }
            return item;
        }
        move_to(w, src, x, y, z);
        self.count += 1;
        src
    }

    #[allow(clippy::too_many_arguments)] // one parameter per field of the call
    fn create_morph_item(
        &mut self,
        w: &mut World,
        x: i32,
        y: i32,
        z: i32,
        inactive: i32,
        active: i32,
        range: i32,
    ) {
        if self.find_morph_item(w, x, y, z, inactive, active) {
            return;
        }
        let item = morph_item_new(w, &MORPH_ITEM, inactive, active, range, 3);
        move_to(w, item, x, y, z);
        self.count += 1;
    }

    #[allow(clippy::too_many_arguments)] // one parameter per field of the call
    fn create_approach_light(
        &mut self,
        w: &mut World,
        x: i32,
        y: i32,
        z: i32,
        off: i32,
        on: i32,
        light: LightType,
    ) {
        if self.find_morph_item(w, x, y, z, off, on) {
            return;
        }
        let item = morph_item_new(w, &MORPH_ITEM, off, on, 2, 3);
        ops::set_light(w, item, light);
        move_to(w, item, x, y, z);
        self.count += 1;
    }

    fn create_sound_effect(
        &mut self,
        w: &mut World,
        x: i32,
        y: i32,
        z: i32,
        sound: i32,
        range: i32,
    ) {
        if self.find_effect_controller(w, x, y, z) {
            return;
        }
        let item = effect_controller_new(w, &EFFECT_CONTROLLER);
        effect_controller_set_sound_id(w, item, sound);
        effect_controller_set_trigger_type(w, item, EffectTriggerType::InRange);
        effect_controller_set_trigger_range(w, item, range);
        move_to(w, item, x, y, z);
        self.count += 1;
    }

    fn create_big_teleporter_item(&mut self, w: &mut World, x: i32, y: i32, reverse: bool) {
        let (a, b) = if reverse {
            (0x17DC, 0x17EE)
        } else {
            (0x17EE, 0x17DC)
        };
        if self.find_morph_item(w, x, y, 0, a, b) {
            return;
        }
        let item = morph_item_new(w, &MORPH_ITEM, a, b, 1, 3);
        move_to(w, item, x, y, 0);
        self.count += 1;
    }
}

/// `GenKhaldun_OnCommand`: "Generates Khaldun Puzzles".
pub fn gen_khaldun(w: &mut World, e: &mut muo_server::commands::CommandEventArgs) {
    let mut g = Gen::default();

    // Generate Morph Items
    for (x, y, z, a, b, r) in [
        (5459, 1416, 0, 0x1D0, 0x1, 1),
        (5460, 1416, 0, 0x1D0, 0x1, 1),
        (5459, 1416, 0, 0x1, 0x53D, 1),
        (5460, 1416, 0, 0x1, 0x53B, 1),
        (5459, 1425, 0, 0x1, 0x53B, 2),
        (5459, 1426, 0, 0x1, 0x53B, 2),
        (5459, 1427, 0, 0x1, 0x53B, 2),
        (5460, 1425, 0, 0x1, 0x53B, 2),
        (5460, 1426, 0, 0x1, 0x53B, 2),
        (5460, 1427, 0, 0x1, 0x53B, 2),
        (5461, 1427, 0, 0x1, 0x53B, 2),
        (5460, 1422, 0, 0x1, 0x544, 2),
        (5460, 1419, 0, 0x1, 0x545, 2),
        (5460, 1420, 0, 0x1, 0x545, 2),
        (5460, 1423, 0, 0x1, 0x545, 2),
        (5460, 1424, 0, 0x1, 0x545, 2),
        (5461, 1426, 0, 0x1, 0x545, 2),
        (5460, 1417, 0, 0x1, 0x546, 1),
        (5460, 1418, 0, 0x1, 0x546, 2),
        (5460, 1421, 0, 0x1, 0x546, 2),
        (5461, 1425, 0, 0x1, 0x548, 2),
        (5459, 1420, 0, 0x1, 0x54A, 2),
        (5459, 1421, 0, 0x1, 0x54A, 2),
        (5459, 1423, 0, 0x1, 0x54A, 2),
        (5459, 1418, 0, 0x1, 0x54B, 2),
        (5459, 1422, 0, 0x1, 0x54B, 2),
        (5459, 1417, 0, 0x1, 0x54C, 1),
        (5459, 1419, 0, 0x1, 0x54C, 2),
        (5459, 1424, 0, 0x1, 0x54C, 2),
        (5458, 1426, 0, 0x1, 0x1D1, 2),
        (5459, 1427, 0, 0x1, 0x1E3, 2),
        (5458, 1425, 3, 0x1, 0x1E4, 2),
        (5458, 1427, 6, 0x1, 0x1E5, 2),
        (5461, 1427, 0, 0x1, 0x1E8, 2),
        (5460, 1427, 0, 0x1, 0x1E9, 2),
        (5458, 1425, 0, 0x1, 0x1EA, 2),
        (5458, 1427, 0, 0x1, 0x1EA, 2),
        (5458, 1427, 3, 0x1, 0x1EA, 2),
    ] {
        g.create_morph_item(w, x, y, z, a, b, r);
    }

    // Generate Approach Lights
    for (x, y, z, off, on, light) in [
        (5393, 1417, 0, 0x1857, 0x1858, LightType::Circle150),
        (5393, 1420, 0, 0x1857, 0x1858, LightType::Circle150),
        (5395, 1421, 0, 0x1857, 0x1858, LightType::Circle150),
        (5396, 1417, 0, 0x1857, 0x1858, LightType::Circle150),
        (5397, 1419, 0, 0x1857, 0x1858, LightType::Circle150),
        (5441, 1393, 5, 0x1F2B, 0x19BB, LightType::Circle225),
        (5446, 1393, 5, 0x1F2B, 0x19BB, LightType::Circle225),
    ] {
        g.create_approach_light(w, x, y, z, off, on, light);
    }

    // Generate Sound Effects
    for (x, y, z, sound, range) in [
        (5425, 1489, 5, 0x102, 1),
        (5425, 1491, 5, 0x102, 1),
        (5449, 1499, 10, 0xF5, 1),
        (5451, 1499, 10, 0xF5, 1),
        (5453, 1499, 10, 0xF5, 1),
        (5524, 1367, 0, 0x102, 1),
        (5450, 1370, 0, 0x220, 2),
        (5450, 1372, 0, 0x220, 2),
        (5460, 1416, 0, 0x244, 2),
        (5483, 1439, 5, 0x14, 3),
    ] {
        g.create_sound_effect(w, x, y, z, sound, range);
    }

    // Generate Big Teleporter
    g.create_big_teleporter_item(w, 5387, 1325, true);
    g.create_big_teleporter_item(w, 5388, 1326, true);
    g.create_big_teleporter_item(w, 5388, 1325, false);
    g.create_big_teleporter_item(w, 5387, 1326, false);

    // Generate Central Khaldun entrance
    let src = switch_new(w, &DISAPPEARING_RAISE_SWITCH, 0x108F);
    let sw = g.try_create_item(w, 5459, 1426, 10, src, &DISAPPEARING_RAISE_SWITCH);
    let src = switch_new(w, &RAISE_SWITCH, 0x1093);
    let lv = g.try_create_item(w, 5403, 1359, 0, src, &RAISE_SWITCH);

    let src = raisable_new(w, 0x788, 10, 0x477, 0x475, TimeSpan::from_secs_f64(90.0));
    let stone = g.try_create_item(w, 5403, 1360, 0, src, &RAISABLE_ITEM);
    let src = raisable_new(w, 0x1D0, 20, 0x477, 0x475, TimeSpan::from_mins(5));
    let door = g.try_create_item(w, 5524, 1367, 0, src, &RAISABLE_ITEM);

    set_raisable_item(w, sw, Some(stone));
    set_raisable_item(w, lv, Some(door));

    let n = g.count;
    muo_content::mx::send_message(
        w,
        e.mobile,
        &format!(
            "{n} dynamic Khaldun item{} generated.",
            if n == 1 { "" } else { "s" }
        ),
    );
}

pub fn configure(w: &mut World) {
    use muo_server::commands::CommandSystem;
    CommandSystem::register(
        w,
        "GenKhaldun",
        AccessLevel::Developer,
        &[],
        std::rc::Rc::new(gen_khaldun),
    );
}
