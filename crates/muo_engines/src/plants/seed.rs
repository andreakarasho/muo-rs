//! `Seed`: a plant seed of a type and hue.

use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::PropType;
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, ops};
use muo_server::target::{TargetFlags, Targeted};
use muo_server::type_registry::{DEFAULT_CALL_PRIORITY, Phase, TypeRegistry};

use super::{
    PlantHooks, PlantHueInfo, PlantTypeInfo, category, hue, random_bonsai,
    random_first_generation_hue, random_first_generation_type, random_peculiar,
};

/// What `Seed` holds.
#[derive(Clone, Copy, Debug)]
pub struct SeedData {
    pub plant_type: i32,
    pub plant_hue: i32,
    pub show_type: bool,
}

pub fn data(w: &World, id: ItemId) -> Option<&SeedData> {
    w.item(id)?.ext.get::<SeedData>()
}

/// `Seed(plantType, plantHue, showType)`: `base(0xDCF)`, `Stackable = Core.SA`, `Hue = PlantHueInfo.GetInfo(hue).Hue`.
pub fn seed_new(w: &mut World, plant_type: i32, plant_hue: i32, show_type: bool) -> Option<ItemId> {
    let id = ops::new_item_with_id(w, &SEED, 0xDCF);
    let sa = w.expansion() >= muo_data::expansion::Expansion::SA;
    ops::set_stackable(w, id, sa);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(SeedData {
            plant_type,
            plant_hue,
            show_type,
        });
    }
    crate::util::hue(w, id, PlantHueInfo::get(plant_hue).hue);
    Some(id)
}

/// `Seed()`: `this(PlantTypeInfo.RandomFirstGeneration(), PlantHueInfo.RandomFirstGeneration())`.
pub fn seed_new_random(w: &mut World) -> ItemId {
    let t = random_first_generation_type(w);
    let h = random_first_generation_hue(w);
    seed_new(w, t, h, false).expect("a seed is always made")
}

/// `PlantHue = value` (`OnPlantHueChanged`: `Hue = PlantHueInfo.GetInfo(newValue).Hue`).
pub fn set_plant_hue(w: &mut World, id: ItemId, value: i32) {
    let changed = w
        .item_mut(id)
        .and_then(|i| i.ext.get_mut::<SeedData>())
        .is_some_and(|d| {
            let c = d.plant_hue != value;
            d.plant_hue = value;
            c
        });
    if changed {
        crate::util::hue(w, id, PlantHueInfo::get(value).hue);
        ops::invalidate_properties(w, id);
    }
}

/// `Seed.RandomBonsaiSeed(increaseRatio)`.
pub fn random_bonsai_seed(w: &mut World, increase_ratio: f64) -> Option<ItemId> {
    let t = random_bonsai(w, increase_ratio);
    seed_new(w, t, hue::PLAIN, false)
}

/// `Seed.RandomPeculiarSeed(group)`.
pub fn random_peculiar_seed(w: &mut World, group: i32) -> Option<ItemId> {
    let t = random_peculiar(w, group);
    seed_new(w, t, hue::PLAIN, false)
}

/// `GetLabel(out args)`.
fn get_label(w: &World, id: ItemId) -> (i32, String) {
    let d = data(w, id).copied().unwrap_or(SeedData {
        plant_type: 0,
        plant_hue: hue::PLAIN,
        show_type: false,
    });
    let amount = w.item(id).map_or(1, |i| i.amount);
    let type_info = PlantTypeInfo::get(d.plant_type);
    let hue_info = PlantHueInfo::get(d.plant_hue);
    let title = if d.show_type || type_info.plant_category == category::DEFAULT {
        hue_info.name
    } else {
        type_info.plant_category
    };
    let bright = hue_info.is_bright();
    if amount == 1 {
        if d.show_type {
            return (
                type_info.seed_label(bright),
                format!("#{title}\t#{}", type_info.name()),
            );
        }
        return (if bright { 1060839 } else { 1060838 }, format!("#{title}")); // [bright] ~1_val~ seed
    }
    if d.show_type {
        return (
            type_info.seed_label_plural(bright),
            format!("{amount}\t#{title}\t#{}", type_info.name()),
        );
    }
    (
        if bright { 1113491 } else { 1113490 },
        format!("{amount}\t#{title}"),
    ) // ~1_amount~ [bright] ~2_val~ seeds
}

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
    if !ops::is_child_of(w, id, pack) {
        muo_content::mx::msg(w, from, 1042664); // You must have the object in your backpack to use it.
        return;
    }
    let mut tg = muo_server::target::Target::new(w, -1, false, TargetFlags::NONE);
    tg.check_los = false; // `CheckLOS = false`
    tg.handlers.on_target = Some(std::rc::Rc::new(
        move |w: &mut World, from: MobileId, t: Targeted| {
            if ops::is_deleted(w, id) {
                return;
            }
            let pack = w.mobile(from).and_then(|m| m.backpack).map(EntityId::Item);
            if !ops::is_child_of(w, id, pack) {
                muo_content::mx::msg(w, from, 1042664);
                return;
            }
            let plant_seed = w.resources.get::<PlantHooks>().and_then(|h| h.plant_seed);
            match t {
                Targeted::Entity(EntityId::Item(item)) => {
                    if !plant_seed.is_some_and(|f| f(w, from, item, id)) {
                        ops::label_to_number(w, item, from, 1061919, ""); // You must use a seed on a bowl of dirt!
                    }
                }
                _ => muo_content::mx::msg(w, from, 1061919),
            }
        },
    ));
    let tid = muo_server::target::add_target(w, tg);
    muo_server::target::set_mobile_target(w, from, Some(tid));
    ops::label_to_number(w, id, from, 1061916, ""); // Choose a bowl of dirt to plant this seed in.
}

/// `[SerializationGenerator(3, false)] Seed`.
pub static SEED: ItemType = ItemType {
    full_name: "Server.Engines.Plants.Seed",
    parent: Some(&ITEM_BASE),
    construct: Some(seed_new_random),
    default_weight: |w, id| {
        crate::template_keys::item_float(w, id, "defaultWeight", ITEM_BASE.default_weight)
    },
    label_number: |w, id| {
        crate::template_keys::item_int(w, id, "labelNumber", ITEM_BASE.label_number)
    },
    force_show_properties: |_, _| true,
    add_name_property: |w, id, list| {
        let (number, args) = get_label(w, id);
        list.add_str(number, &args);
    },
    on_single_click: |w, id, from| {
        let (number, args) = get_label(w, id);
        ops::label_to_number(w, id, from, number, &args);
    },
    on_double_click,
    stack_with: |w, id, from, dropped, play_sound| {
        let same = match (data(w, id).copied(), data(w, dropped).copied()) {
            (Some(a), Some(b)) => {
                a.plant_type == b.plant_type
                    && a.plant_hue == b.plant_hue
                    && a.show_type == b.show_type
            }
            _ => false,
        };
        same && (ITEM_BASE.stack_with)(w, id, from, dropped, play_sound)
    },
    on_after_duped: |w, id, new_item| {
        let Some(d) = data(w, id).copied() else {
            return;
        };
        let h = w.item(id).map_or(0, |i| i.hue);
        if data(w, new_item).is_some() {
            if let Some(n) = w
                .item_mut(new_item)
                .and_then(|i| i.ext.get_mut::<SeedData>())
            {
                n.plant_hue = d.plant_hue;
            }
            crate::util::hue(w, new_item, h);
        }
    },
    serialize: |w, id, out| {
        (ITEM_BASE.serialize)(w, id, out);
        let d = data(w, id).copied().unwrap_or(SeedData {
            plant_type: 0,
            plant_hue: hue::PLAIN,
            show_type: false,
        });
        out.write_i32(3);
        out.write_enum(4, i64::from(d.plant_type));
        out.write_enum(4, i64::from(d.plant_hue));
        out.write_bool(d.show_type);
    },
    deserialize: |w, id, r| {
        (ITEM_BASE.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        let d = if version < 3 {
            SeedData {
                plant_type: r.read_i32()?,
                plant_hue: r.read_i32()?,
                show_type: r.read_bool()?,
            }
        } else {
            SeedData {
                plant_type: r.read_enum(4)? as i32,
                plant_hue: r.read_enum(4)? as i32,
                show_type: r.read_bool()?,
            }
        };
        if let Some(i) = w.item_mut(id) {
            i.ext = Ext::new(d);
        }
        Ok(())
    },
    ..ITEM_BASE
};

pub(crate) const TYPE_NAMES: &[(&str, i64)] = &[
    ("CampionFlowers", 0),
    ("Poppies", 1),
    ("Snowdrops", 2),
    ("Bulrushes", 3),
    ("Lilies", 4),
    ("PampasGrass", 5),
    ("Rushes", 6),
    ("ElephantEarPlant", 7),
    ("Fern", 8),
    ("PonytailPalm", 9),
    ("SmallPalm", 10),
    ("CenturyPlant", 11),
    ("WaterPlant", 12),
    ("SnakePlant", 13),
    ("PricklyPearCactus", 14),
    ("BarrelCactus", 15),
    ("TribarrelCactus", 16),
    ("CommonGreenBonsai", 17),
    ("CommonPinkBonsai", 18),
    ("UncommonGreenBonsai", 19),
    ("UncommonPinkBonsai", 20),
    ("RareGreenBonsai", 21),
    ("RarePinkBonsai", 22),
    ("ExceptionalBonsai", 23),
    ("ExoticBonsai", 24),
    ("Cactus", 25),
    ("FlaxFlowers", 26),
    ("FoxgloveFlowers", 27),
    ("HopsEast", 28),
    ("OrfluerFlowers", 29),
    ("CypressTwisted", 30),
    ("HedgeShort", 31),
    ("JuniperBush", 32),
    ("SnowdropPatch", 33),
    ("Cattails", 34),
    ("PoppyPatch", 35),
    ("SpiderTree", 36),
    ("WaterLily", 37),
    ("CypressStraight", 38),
    ("HedgeTall", 39),
    ("HopsSouth", 40),
    ("SugarCanes", 41),
    ("CocoaTree", 42),
];

pub(crate) const HUE_NAMES: &[(&str, i64)] = &[
    ("Plain", hue::PLAIN as i64),
    ("Red", hue::RED as i64),
    ("Blue", hue::BLUE as i64),
    ("Yellow", hue::YELLOW as i64),
    ("BrightRed", hue::BRIGHT_RED as i64),
    ("BrightBlue", hue::BRIGHT_BLUE as i64),
    ("BrightYellow", hue::BRIGHT_YELLOW as i64),
    ("Purple", hue::PURPLE as i64),
    ("Green", hue::GREEN as i64),
    ("Orange", hue::ORANGE as i64),
    ("BrightPurple", hue::BRIGHT_PURPLE as i64),
    ("BrightGreen", hue::BRIGHT_GREEN as i64),
    ("BrightOrange", hue::BRIGHT_ORANGE as i64),
    ("Black", hue::BLACK as i64),
    ("White", hue::WHITE as i64),
    ("Pink", hue::PINK as i64),
    ("Magenta", hue::MAGENTA as i64),
    ("Aqua", hue::AQUA as i64),
    ("FireRed", hue::FIRE_RED as i64),
    ("None", 0),
];

static TYPE_HUE_PARAMS: [Param; 3] = [
    Param {
        name: "plantType",
        kind: Kind::Plain(PropType::Enum(TYPE_NAMES)),
        default: None,
        ty: "PlantType",
    },
    Param {
        name: "plantHue",
        kind: Kind::Plain(PropType::Enum(HUE_NAMES)),
        default: None,
        ty: "PlantHue",
    },
    Param {
        name: "showType",
        kind: Kind::Plain(PropType::Bool),
        default: Some(Value::Bool(false)),
        ty: "",
    },
];

/// `Seed()` and `Seed(plantType, plantHue, showType = false)`, in declaration order.
static SEED_CTORS: [Ctor; 2] = [
    Ctor {
        access: None,
        params: &[],
        make: |w, _| EntityId::Item(seed_new_random(w)),
    },
    Ctor {
        access: None,
        params: &TYPE_HUE_PARAMS,
        make: |w, v| {
            let id = seed_new(w, v[0].i32(), v[1].i32(), v[2].i64() != 0)
                .expect("a seed is always made");
            EntityId::Item(id)
        },
    },
];

pub fn register(types: &mut TypeRegistry) {
    types.register_item(&SEED, &[]);
    types.register_phase(Phase::Configure, "Seed.Ctors", DEFAULT_CALL_PRIORITY, |w| {
        add_ctors(w, "Server.Engines.Plants.Seed", &SEED_CTORS);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;

    /// `InternalTarget`: `base(-1, false, ..)` with `CheckLOS = false` (a seed can be planted past a wall).
    #[test]
    fn seed_target_does_not_check_line_of_sight() {
        let mut w = muo_server::map::test_support::world();
        muo_content::register(&mut w.types);
        muo_items::register(&mut w.types);
        crate::register(&mut w.types);
        let from = w.new_mobile(&MOBILE_BASE);
        let pack = muo_content::factory::make_item(&mut w, "Server.Items.Backpack").unwrap();
        muo_server::mobile::part2::items::add_item(&mut w, from, pack);
        muo_server::mobile::part1::props_b::backpack(&mut w, from);
        let seed = seed_new_random(&mut w);
        assert!(muo_server::mobile::part2::items::place_in_backpack(
            &mut w, from, seed
        ));
        on_double_click(&mut w, seed, from);
        let t = w.mobile(from).unwrap().target.expect("a target was begun");
        let t = &w.targets[&t];
        assert!(!t.check_los);
        assert_eq!((t.range, t.allow_ground), (-1, false));
    }
}
