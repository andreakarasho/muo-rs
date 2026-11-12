//! `GrimmochDrummel`, `LysanderGathenwale`, `MorgBergen` and `TavaraSewel`: the cursed
//! expedition members of the Khaldun pit.

use muo_content::creature::enums::{AIType, DEFAULT_RANGE_PERCEPTION, FightMode};
use muo_content::creature::setters::{
    add_loot_n, new_creature, pack_item, set_damage, set_damage_type, set_dex_range,
    set_hits_range, set_int_range, set_mana, set_mana_range, set_resistance_range, set_skill_range,
    set_stam_range, set_str_range,
};
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType};
use muo_content::{factory, hooks::rnd, loot, loot_pack, mx};
use muo_primitives::Body;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::ops;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::items::add_item;
use muo_server::mobile::{MobileType, ResistanceType};
use muo_server::skills::SkillName;
use muo_server::type_registry::TypeRegistry;

/// `AddItem(new T(hue))` (or without a hue); returns the item to tweak.
fn equip(w: &mut World, name: &str, hue: Option<i32>) -> Option<ItemId> {
    let it = factory::make_item(w, name)?;
    if let Some(h) = hue {
        crate::util::hue(w, it, h);
    }
    Some(it)
}

fn add(w: &mut World, m: MobileId, it: Option<ItemId>) {
    if let Some(it) = it {
        add_item(w, m, it);
    }
}

fn common(w: &mut World, id: MobileId, hue: i32, female: bool) {
    props_a::set_title(w, id, Some("the Cursed".to_owned()));
    muo_server::mobile::base::set_hue(w, id, hue);
    if female {
        props_b::set_female(w, id, true);
    }
    props_b::set_body(w, id, Body(if female { 0x191 } else { 0x190 }));
}

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
    let id = new_creature(
        w,
        &GRIMMOCH_DRUMMEL,
        AIType::AiArcher,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    common(w, id, 0x8596, false);
    muo_server::mobile::part1::props_b::set_hair_item_id(w, id, 0x204A); // Krisna
    if let Some(bow) = equip(w, "Server.Items.Bow", None) {
        ops::set_movable(w, bow, false);
        add_item(w, id, bow);
    }
    let it = equip(w, "Server.Items.Boots", Some(0x8A4));
    add(w, id, it);
    let it = equip(w, "Server.Items.BodySash", Some(0x8A4));
    add(w, id, it);
    if let Some(pack) = equip(w, "Server.Items.Backpack", None) {
        ops::set_movable(w, pack, false);
        add_item(w, id, pack);
    }
    let gloves = equip(w, "Server.Items.LeatherGloves", Some(0x96F));
    let chest = equip(w, "Server.Items.LeatherChest", Some(0x96F));
    add(w, id, gloves);
    add(w, id, chest);

    set_str_range(w, id, 111, 120);
    set_dex_range(w, id, 151, 160);
    set_int_range(w, id, 41, 50);
    set_hits_range(w, id, 180, 207);
    set_mana(w, id, 0);
    set_damage(w, id, 13, 16);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 45, 55);
    set_resistance_range(w, id, ResistanceType::Poison, 30, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 20, 25);
    set_skill_range(w, id, SkillName::Archery, 90.1, 110.0);
    set_skill_range(w, id, SkillName::Swords, 60.1, 70.0);
    set_skill_range(w, id, SkillName::Tactics, 90.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 60.1, 70.0);
    set_skill_range(w, id, SkillName::Anatomy, 90.1, 100.0);
    props_a::set_fame(w, id, 5000);
    props_a::set_karma(w, id, -1000);

    let arrows = factory::make_item_amount(w, "Server.Items.Arrow", 40);
    pack_item(w, id, arrows);
    if rnd::below(w, 100) < 3 {
        let horn = factory::make_item(w, "Server.Items.FireHorn");
        pack_item(w, id, horn);
    }
    if rnd::below(w, 3) < 1 {
        let j = loot::random_grimmoch_journal(w);
        pack_item(w, id, j);
    }
    id
}

static GRIMMOCH_DRUMMEL_C: CreatureType = CreatureType {
    delete_corpse_on_death: |_, _| true,
    always_murderer: |_, _| true,
    default_name: |_, _| Some("Grimmoch Drummel"),
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
    let id = new_creature(
        w,
        &LYSANDER_GATHENWALE,
        AIType::AiMage,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    common(w, id, 0x8838, false);
    let it = equip(w, "Server.Items.Boots", Some(0x599));
    add(w, id, it);
    let it = equip(w, "Server.Items.Cloak", Some(0x96F));
    add(w, id, it);
    let book = equip(w, "Server.Items.Spellbook", Some(0x599));
    let gloves = equip(w, "Server.Items.RingmailGloves", Some(0x599));
    let chest = equip(w, "Server.Items.StuddedChest", Some(0x96F));
    let arms = equip(w, "Server.Items.PlateArms", Some(0x599));
    for it in [book, gloves, chest, arms] {
        add(w, id, it);
    }

    set_str_range(w, id, 111, 120);
    set_dex_range(w, id, 71, 80);
    set_int_range(w, id, 121, 130);
    set_hits_range(w, id, 180, 207);
    set_mana_range(w, id, 227, 265);
    set_damage(w, id, 5, 13);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 60);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::Wrestling, 80.1, 90.0);
    set_skill_range(w, id, SkillName::Tactics, 90.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 80.1, 90.0);
    set_skill_range(w, id, SkillName::Magery, 90.1, 100.0);
    set_skill_range(w, id, SkillName::EvalInt, 95.1, 100.0);
    set_skill_range(w, id, SkillName::Meditation, 90.1, 100.0);
    props_a::set_fame(w, id, 5000);
    props_a::set_karma(w, id, -10000);

    if let Some(reags) = loot::random_reagent(w) {
        ops::set_amount(w, reags, 30);
        pack_item(w, id, Some(reags));
    }
    id
}

static LYSANDER_C: CreatureType = CreatureType {
    delete_corpse_on_death: |_, _| true,
    always_murderer: |_, _| true,
    default_name: |_, _| Some("Lysander Gatherwale"),
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
    let id = new_creature(
        w,
        &MORG_BERGEN,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    common(w, id, 0x8596, false);
    let it = equip(w, "Server.Items.ShortPants", Some(0x59C));
    add(w, id, it);
    let bardiche = equip(w, "Server.Items.Bardiche", Some(0x96F));
    if let Some(b) = bardiche {
        ops::set_movable(w, b, false);
    }
    let gloves = equip(w, "Server.Items.LeatherGloves", Some(0x96F));
    let arms = equip(w, "Server.Items.LeatherArms", Some(0x96F));
    for it in [bardiche, gloves, arms] {
        add(w, id, it);
    }

    set_str_range(w, id, 111, 120);
    set_dex_range(w, id, 111, 120);
    set_int_range(w, id, 51, 60);
    set_hits_range(w, id, 180, 207);
    set_mana(w, id, 0);
    set_damage(w, id, 9, 17);
    set_damage_type(w, id, ResistanceType::Physical, 40);
    set_damage_type(w, id, ResistanceType::Cold, 60);
    set_resistance_range(w, id, ResistanceType::Physical, 35, 45);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 60);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::Swords, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 90.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 80.1, 90.0);
    set_skill_range(w, id, SkillName::Anatomy, 90.1, 100.0);
    props_a::set_fame(w, id, 5000);
    props_a::set_karma(w, id, -1000);
    id
}

static MORG_C: CreatureType = CreatureType {
    delete_corpse_on_death: |_, _| true,
    always_murderer: |_, _| true,
    default_name: |_, _| Some("Morg Bergen"),
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
    let id = new_creature(
        w,
        &TAVARA_SEWEL,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    common(w, id, 0x8838, true);
    let it = equip(w, "Server.Items.Kilt", Some(0x59C));
    add(w, id, it);
    let it = equip(w, "Server.Items.Sandals", Some(0x599));
    add(w, id, it);
    let kryss = equip(w, "Server.Items.Kryss", Some(0x96F));
    let buckler = equip(w, "Server.Items.Buckler", Some(0x96F));
    for it in [kryss, buckler].into_iter().flatten() {
        ops::set_movable(w, it, false);
    }
    let gloves = equip(w, "Server.Items.RingmailGloves", Some(0x599));
    let chest = equip(w, "Server.Items.FemalePlateChest", Some(0x96F));
    for it in [kryss, buckler, gloves, chest] {
        add(w, id, it);
    }

    set_str_range(w, id, 111, 120);
    set_dex_range(w, id, 111, 120);
    set_int_range(w, id, 111, 120);
    set_hits_range(w, id, 180, 207);
    set_stam_range(w, id, 126, 150);
    set_mana(w, id, 0);
    set_damage(w, id, 13, 16);
    set_resistance_range(w, id, ResistanceType::Physical, 25, 30);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 30);
    set_resistance_range(w, id, ResistanceType::Cold, 50, 60);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 35);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 35);
    set_skill_range(w, id, SkillName::Fencing, 90.1, 100.0);
    set_skill_range(w, id, SkillName::Tactics, 90.1, 100.0);
    set_skill_range(w, id, SkillName::MagicResist, 80.1, 90.0);
    set_skill_range(w, id, SkillName::Anatomy, 90.1, 100.0);
    props_a::set_fame(w, id, 5000);
    props_a::set_karma(w, id, -1000);
    id
}

static TAVARA_C: CreatureType = CreatureType {
    delete_corpse_on_death: |_, _| true,
    always_murderer: |_, _| true,
    default_name: |_, _| Some("Tavara Sewel"),
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
