//! `GrimmochDrummel`, `LysanderGathenwale`, `MorgBergen` and `TavaraSewel`: the cursed
//! expedition members of the Khaldun pit.

use muo_content::creature::setters::add_loot_n;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_content::{factory, hooks::rnd, loot, loot_pack, mx};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::ops;
use muo_server::mobile::MobileType;
use muo_server::type_registry::TypeRegistry;

/// `new Gold(Utility.RandomMinMax(190, 230)).MoveToWorld(Location, Map)`.
fn drop_gold(w: &mut World, id: MobileId) {
    let n = rnd::min_max(w, 190, 230);
    if let Some(g) = factory::make_item_amount(w, "Server.Items.Gold", n) {
        let (l, m) = (mx::location(w, id), mx::map(w, id));
        ops::move_to_world_map(w, g, l, m);
    }
}

fn pit_effect(w: &mut World, id: MobileId) {
    muo_server::effects::send_location_effect(
        w,
        mx::location(w, id),
        mx::map(w, id),
        0x376A,
        10,
        1,
        0,
        0,
    );
}

/// Moves a random journal/notebook (`Loot.RandomXxx()`) to the creature's feet.
fn drop_book(w: &mut World, id: MobileId, make: fn(&mut World) -> Option<ItemId>) {
    if let Some(b) = make(w) {
        let (l, m) = (mx::location(w, id), mx::map(w, id));
        ops::move_to_world_map(w, b, l, m);
    }
}

// ---- GrimmochDrummel ----

fn grimmoch_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &GRIMMOCH_DRUMMEL)
}

static GRIMMOCH_DRUMMEL_C: CreatureType = CreatureType {
    delete_corpse_on_death: |_, _| true,
    always_murderer: muo_content::templates::creature_props::always_murderer,
    default_name: muo_content::templates::creature_props::default_name,
    ..BASE_CREATURE_C
};

/// `Server.Mobiles.GrimmochDrummel`.
pub static GRIMMOCH_DRUMMEL: MobileType = MobileType {
    full_name: "Server.Mobiles.GrimmochDrummel",
    parent: Some(&BASE_CREATURE),
    construct: Some(grimmoch_new),
    family: Some(&GRIMMOCH_DRUMMEL_C),
    click_title: |_, _| false,
    show_fame_title: |_, _| false,
    get_idle_sound: |_, _| 0x178,
    get_anger_sound: |_, _| 0x1AC,
    get_death_sound: |_, _| 0x27E,
    get_hurt_sound: |_, _| 0x177,
    on_before_death: |w, id| {
        drop_gold(w, id);
        if let Some(pack) = w.mobile(id).and_then(|m| m.backpack) {
            ops::set_movable(w, pack, true);
            let (l, m) = (mx::location(w, id), mx::map(w, id));
            ops::move_to_world_map(w, pack, l, m);
        }
        pit_effect(w, id);
        true
    },
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_CREATURE
};

// ---- LysanderGathenwale ----

fn lysander_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &LYSANDER_GATHENWALE)
}

static LYSANDER_C: CreatureType = CreatureType {
    delete_corpse_on_death: |_, _| true,
    always_murderer: muo_content::templates::creature_props::always_murderer,
    default_name: muo_content::templates::creature_props::default_name,
    generate_loot: |w, id| add_loot_n(w, id, &loot_pack::MED_SCROLLS, 2),
    ..BASE_CREATURE_C
};

/// `Server.Mobiles.LysanderGathenwale`.
pub static LYSANDER_GATHENWALE: MobileType = MobileType {
    full_name: "Server.Mobiles.LysanderGathenwale",
    parent: Some(&BASE_CREATURE),
    construct: Some(lysander_new),
    family: Some(&LYSANDER_C),
    click_title: |_, _| false,
    show_fame_title: |_, _| false,
    get_idle_sound: |_, _| 0x1CE,
    get_anger_sound: |_, _| 0x1AC,
    get_death_sound: |_, _| 0x182,
    get_hurt_sound: |_, _| 0x28D,
    on_before_death: |w, id| {
        if !(BASE_CREATURE.on_before_death)(w, id) {
            return false;
        }
        // `Backpack?.Destroy()`
        if let Some(pack) = w.mobile(id).and_then(|m| m.backpack)
            && let Some(t) = w.item(pack).map(|i| i.ty)
        {
            (t.delete)(w, pack);
        }
        if rnd::below(w, 3) == 0 {
            drop_book(w, id, loot::random_lysander_notebook);
        }
        pit_effect(w, id);
        true
    },
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_CREATURE
};

// ---- MorgBergen ----

fn morg_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &MORG_BERGEN)
}

static MORG_C: CreatureType = CreatureType {
    delete_corpse_on_death: |_, _| true,
    always_murderer: muo_content::templates::creature_props::always_murderer,
    default_name: muo_content::templates::creature_props::default_name,
    ..BASE_CREATURE_C
};

/// `Server.Mobiles.MorgBergen`.
pub static MORG_BERGEN: MobileType = MobileType {
    full_name: "Server.Mobiles.MorgBergen",
    parent: Some(&BASE_CREATURE),
    construct: Some(morg_new),
    family: Some(&MORG_C),
    show_fame_title: |_, _| false,
    get_idle_sound: |_, _| 0x1CE,
    get_anger_sound: |_, _| 0x263,
    get_death_sound: |_, _| 0x1D1,
    get_hurt_sound: |_, _| 0x25E,
    on_before_death: |w, id| {
        drop_gold(w, id);
        pit_effect(w, id);
        true
    },
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_CREATURE
};

// ---- TavaraSewel ----

fn tavara_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &TAVARA_SEWEL)
}

static TAVARA_C: CreatureType = CreatureType {
    delete_corpse_on_death: |_, _| true,
    always_murderer: muo_content::templates::creature_props::always_murderer,
    default_name: muo_content::templates::creature_props::default_name,
    ..BASE_CREATURE_C
};

/// `Server.Mobiles.TavaraSewel`.
pub static TAVARA_SEWEL: MobileType = MobileType {
    full_name: "Server.Mobiles.TavaraSewel",
    parent: Some(&BASE_CREATURE),
    construct: Some(tavara_new),
    family: Some(&TAVARA_C),
    show_fame_title: |_, _| false,
    get_idle_sound: |_, _| 0x27F,
    get_anger_sound: |_, _| 0x258,
    get_death_sound: |_, _| 0x25B,
    get_hurt_sound: |_, _| 0x257,
    on_before_death: |w, id| {
        drop_gold(w, id);
        if rnd::below(w, 3) == 0 {
            drop_book(w, id, loot::random_tavaras_journal);
        }
        pit_effect(w, id);
        true
    },
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    ..BASE_CREATURE
};

pub fn register(types: &mut TypeRegistry) {
    for t in [
        &GRIMMOCH_DRUMMEL,
        &LYSANDER_GATHENWALE,
        &MORG_BERGEN,
        &TAVARA_SEWEL,
    ] {
        types.register_mobile(t, &[]);
    }
}
