//! `[set` / `[props` / `where` reach the generated `[CommandProperty]` tables and the hand tables of the families
//! (factions, boats, houses, corpses, vendors, quivers, talismans, spellbooks..), on the whole registered program: the
//! setter that runs is the server's own, so its side effects (deltas, hue, name, invalidation) happen.

use std::collections::BTreeSet;

use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::AccessLevel;
use muo_server::type_registry::{Phase, invoke_phase};

fn world() -> World {
    let mut w = World::new();
    muo_server::server_setup::register(&mut w.types);
    muo::register_content(&mut w.types);
    let rows: Vec<String> = (0..58)
        .map(|i| format!(r#"{{"SkillID":{i},"Name":"Skill{i}","Title":"T","StrScale":0,"DexScale":0,"IntScale":0,"StatTotal":0,"StrGain":0,"DexGain":0,"IntGain":0,"GainFactor":1.0,"PrimaryStat":"Str","SecondaryStat":"Dex"}}"#))
        .collect();
    w.resources.get_mut::<muo_server::skills::SkillTable>().info =
        muo_data::skill_info::SkillInfo::from_json(&format!("[{}]", rows.join(","))).unwrap();
    invoke_phase(&mut w, Phase::Configure);
    for (i, name) in [(0, "Felucca"), (1, "Trammel")] {
        w.maps.add(muo_server::map::Map::new(
            i,
            i,
            i,
            7168,
            4096,
            0,
            name,
            Default::default(),
        ));
    }
    w
}

fn staff(w: &mut World) -> MobileId {
    let from = w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE);
    w.mobile_mut(from).unwrap().access_level = AccessLevel::Developer;
    from
}

fn item(w: &mut World, class: &str) -> EntityId {
    EntityId::Item(
        muo_content::factory::make_item(w, class).unwrap_or_else(|| panic!("no item {class}")),
    )
}

fn set(w: &mut World, from: MobileId, e: EntityId, name: &str, value: &str) -> String {
    muo_commands::properties::set_value(w, from, e, name, value)
}

fn get(w: &mut World, from: MobileId, e: EntityId, name: &str) -> String {
    muo_commands::properties::get_value(w, from, e, name)
}

fn it(e: EntityId) -> ItemId {
    match e {
        EntityId::Item(i) => i,
        EntityId::Mobile(_) => panic!("not an item"),
    }
}

/// A table added under a class name no registered type has would never be read: every table class is an ancestor-or-self
/// of some registered item or mobile type.
#[test]
fn every_table_names_a_registered_class() {
    let w = world();
    let mut known: BTreeSet<&str> = BTreeSet::new();
    for t in w.types.item_types() {
        let mut c = Some(*t);
        while let Some(x) = c {
            known.insert(x.full_name);
            c = x.parent;
        }
    }
    for t in w.types.mobile_types() {
        let mut c = Some(*t);
        while let Some(x) = c {
            known.insert(x.full_name);
            c = x.parent;
        }
    }
    let bad: Vec<_> = muo_content::props::declared_names(&w)
        .into_iter()
        .map(|(c, _)| c)
        // `VirtualCheck` is made by code (a trade) and is registered nowhere
        .filter(|c| {
            c.starts_with("Server.")
                && !known.contains(c)
                && *c != "Server.Items.VirtualCheck"
                // `Spawner : BaseSpawner` is one Rust class with two property groups
                && *c != "Server.Engines.Spawners.BaseSpawner"
        })
        .collect();
    assert!(
        bad.is_empty(),
        "tables for classes no registered type has: {bad:#?}"
    );
}

#[test]
fn faction_items_set_through_the_ports_own_setters() {
    let mut w = world();
    let from = staff(&mut w);
    let mono = EntityId::Item(muo_engines::factions::items::new_monolith(
        &mut w, false, None, None,
    ));
    assert_eq!(
        set(&mut w, from, mono, "Faction", "Minax"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, mono, "Faction"), "Faction = Minax");
    // `BaseMonolith.Faction` sets the hue of the faction (`HuePrimary`): a blind field write would leave it 0
    assert_eq!(
        w.item(it(mono)).unwrap().hue,
        muo_engines::factions::tables::MINAX.hue_primary
    );
    assert_eq!(
        set(&mut w, from, mono, "Town", "Skara Brae"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, mono, "Town"), "Town = Skara Brae");
    assert_eq!(
        set(&mut w, from, mono, "Faction", "null"),
        "Property has been set."
    );
    assert_eq!(w.item(it(mono)).unwrap().hue, 0);
    let stone = item(&mut w, "Server.Factions.JoinStone");
    assert_eq!(
        set(&mut w, from, stone, "Faction", "Shadowlords"),
        "Property has been set."
    );
    assert_eq!(
        w.item(it(stone)).unwrap().hue,
        muo_engines::factions::tables::SHADOWLORDS.hue_join
    );
}

#[test]
fn serialized_fields_keep_their_setter_side_effects() {
    let mut w = world();
    let from = staff(&mut w);
    // SOS.Level: `AllowLevelChange` clamps to 1..=4 and `OnLevelChanged` updates the hue
    let sos = item(&mut w, "Server.Items.SOS");
    assert_eq!(
        set(&mut w, from, sos, "Level", "9"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, sos, "Level"), "Level = 4 (0x4)");
    assert_eq!(w.item(it(sos)).unwrap().hue, 0x481);
    assert_eq!(
        set(&mut w, from, sos, "Level", "2"),
        "Property has been set."
    );
    assert_eq!(w.item(it(sos)).unwrap().hue, 0);
    // a plain int field
    let horn = item(&mut w, "Server.Engines.Quests.HornOfRetreat");
    assert_eq!(
        set(&mut w, from, horn, "Charges", "7"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, horn, "Charges"), "Charges = 7 (0x7)");
    // `Marked` and `TargetMap` recolour the rune (`CalculateHue`): a blind field write would leave it 0
    let rune = item(&mut w, "Server.Items.RecallRune");
    assert_eq!(
        set(&mut w, from, rune, "TargetMap", "Trammel"),
        "Property has been set."
    );
    assert_eq!(w.item(it(rune)).unwrap().hue, 0);
    assert_eq!(
        set(&mut w, from, rune, "Marked", "true"),
        "Property has been set."
    );
    assert_eq!(w.item(it(rune)).unwrap().hue, 50);
}

#[test]
fn staff_cannot_write_above_their_level() {
    let mut w = world();
    let from = w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE);
    w.mobile_mut(from).unwrap().access_level = AccessLevel::GameMaster;
    let mono = EntityId::Item(muo_engines::factions::items::new_monolith(
        &mut w, false, None, None,
    ));
    // `[CommandProperty(AccessLevel.Counselor, AccessLevel.Administrator)]`
    assert_ne!(
        set(&mut w, from, mono, "Faction", "Minax"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, mono, "Faction"), "Faction = null");
}

#[test]
fn spellbook_content_recounts_the_spells() {
    let mut w = world();
    let from = staff(&mut w);
    let book = item(&mut w, "Server.Items.Spellbook");
    // `Content` runs `OnContentChanged`: `SpellCount` follows the bits
    assert_eq!(
        set(&mut w, from, book, "Content", "7"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, book, "SpellCount"),
        "SpellCount = 3 (0x3)"
    );
    assert_eq!(
        set(&mut w, from, book, "Attributes.Luck", "50"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, book, "Attributes.Luck"),
        "Attributes.Luck = 50 (0x32)"
    );
}

#[test]
fn quiver_and_talisman_bags_and_attribute_objects() {
    let mut w = world();
    let from = staff(&mut w);
    let q = item(&mut w, "Server.Items.ElvenQuiver");
    assert_eq!(
        set(&mut w, from, q, "LowerAmmoCost", "25"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, q, "LowerAmmoCost"),
        "LowerAmmoCost = 25 (0x19)"
    );
    assert_eq!(
        set(&mut w, from, q, "Attributes.Luck", "10"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, q, "Attributes.Luck"),
        "Attributes.Luck = 10 (0xA)"
    );
    let t = item(&mut w, "Server.Items.RandomTalisman");
    assert_eq!(
        set(&mut w, from, t, "Protection.Amount", "12"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, t, "Protection.Amount"),
        "Protection.Amount = 12 (0xC)"
    );
    assert_eq!(
        set(&mut w, from, t, "MaxCharges", "9"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, t, "MaxCharges"), "MaxCharges = 9 (0x9)");
}

#[test]
fn mobile_account_weapon_and_mount_read_the_engine() {
    let mut w = world();
    let from = staff(&mut w);
    let m = EntityId::Mobile(from);
    assert_eq!(get(&mut w, from, m, "Mounted"), "Mounted = False");
    assert_eq!(
        get(&mut w, from, m, "Weapon"),
        "Weapon = Server.Items.Fists"
    );
    // `Account` is an object hosted by the mobile
    let acct = muo_content::accounting::create_account(&mut w, "tester", "pw");
    w.mobile_mut(from).unwrap().account = Some(acct);
    assert_eq!(
        set(&mut w, from, m, "Account.Email", "a@b.c"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, m, "Account.Email"),
        "Account.Email = \"a@b.c\""
    );
    assert_eq!(
        muo_content::accounting::account(&w, acct)
            .unwrap()
            .email
            .as_deref(),
        Some("a@b.c")
    );
    // `Race` goes through `set_race` (it changes the body)
    assert_eq!(get(&mut w, from, m, "Race"), "Race = Human");
}

#[test]
fn weapon_and_armor_members_run_the_ports_setters() {
    let mut w = world();
    let from = staff(&mut w);
    let sword = item(&mut w, "Server.Items.Longsword");
    // `StrRequirement` reads the class value until it is set (`-1` means "the class's"), then the written one
    let before = get(&mut w, from, sword, "StrRequirement");
    assert_eq!(
        set(&mut w, from, sword, "StrRequirement", "77"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, sword, "StrRequirement"),
        "StrRequirement = 77 (0x4D)"
    );
    assert_ne!(before, get(&mut w, from, sword, "StrRequirement"));
    // `Quality` runs UnscaleDurability / ScaleDurability: the hit points follow
    let hp = get(&mut w, from, sword, "MaxHitPoints");
    assert_eq!(
        set(&mut w, from, sword, "Quality", "Exceptional"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, sword, "Quality"), "Quality = Exceptional");
    assert_ne!(hp, get(&mut w, from, sword, "MaxHitPoints"));
    let armor = item(&mut w, "Server.Items.PlateChest");
    assert_eq!(
        set(&mut w, from, armor, "BaseArmorRating", "30"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, armor, "BaseArmorRating"),
        "BaseArmorRating = 30 (0x1E)"
    );
    assert!(get(&mut w, from, armor, "BodyPosition").starts_with("BodyPosition = "));
}

#[test]
fn corpse_flags_and_boat_plank_members() {
    let mut w = world();
    let from = staff(&mut w);
    let corpse = EntityId::Item(muo_content::items::corpse::new_corpse(
        &mut w,
        from,
        Vec::new(),
    ));
    assert_eq!(
        set(&mut w, from, corpse, "Carved", "true"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, corpse, "Carved"), "Carved = True");
    assert_eq!(get(&mut w, from, corpse, "IsBones"), "IsBones = False");
    assert_eq!(get(&mut w, from, corpse, "Devoured"), "Devoured = False");
    let boat = muo_content::factory::make_item(&mut w, "Server.Multis.SmallBoat")
        .unwrap_or_else(|| muo_server::ids::ItemId(muo_primitives::Serial::new(0x4000_0001)));
    let plank = EntityId::Item(muo_multis::boats::parts::new_plank(
        &mut w,
        boat,
        muo_multis::boats::parts::PlankSide::Port,
        0,
    ));
    assert_eq!(get(&mut w, from, plank, "Starboard"), "Starboard = False");
    assert_eq!(
        set(&mut w, from, plank, "Side", "Starboard"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, plank, "Starboard"), "Starboard = True");
}

#[test]
fn container_state_lives_in_the_item_not_in_ext() {
    let mut w = world();
    let from = staff(&mut w);
    let pack = item(&mut w, "Server.Items.Backpack");
    assert_eq!(
        set(&mut w, from, pack, "MaxItems", "55"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, pack, "MaxItems"), "MaxItems = 55 (0x37)");
    assert_eq!(
        set(&mut w, from, pack, "GumpID", "60"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, pack, "GumpID"), "GumpID = 60 (0x3C)");
}

/// props3: a `PlayerMobile` hosts the `VirtueContext`, `ChampionTitleContext` and murder members, and the clamps of the
/// `Allow*Change` setters hold (`FlourMill.CurFlour` 0..=2, `ShortTermMurders >= 0`).
#[test]
fn engine_objects_over_a_player_and_clamping_setters() {
    let mut w = world();
    let from = staff(&mut w);
    let pm = EntityId::Mobile(w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE));
    assert_eq!(
        set(&mut w, from, pm, "Virtues.Valor", "7"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, pm, "Virtues.Valor"),
        "Virtues.Valor = 7 (0x7)"
    );
    assert_eq!(
        set(&mut w, from, pm, "ChampionTitles.AbyssValue", "3"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, pm, "ChampionTitles.AbyssValue"),
        "ChampionTitles.AbyssValue = 3 (0x3)"
    );
    assert_eq!(
        set(&mut w, from, pm, "ChampionTitles.AbyssValue", "0"),
        "Property has been set."
    ); // removes the title
    assert_eq!(
        get(&mut w, from, pm, "ChampionTitles.AbyssValue"),
        "ChampionTitles.AbyssValue = 0 (0x0)"
    );
    assert_eq!(
        set(&mut w, from, pm, "ShortTermMurders", "-4"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, pm, "ShortTermMurders"),
        "ShortTermMurders = 0 (0x0)"
    );
    let mill = item(&mut w, "Server.Items.FlourMillEastAddon");
    assert_eq!(
        set(&mut w, from, mill, "CurFlour", "40"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, mill, "CurFlour"), "CurFlour = 2 (0x2)");
    assert_eq!(get(&mut w, from, mill, "IsFull"), "IsFull = True");
}
