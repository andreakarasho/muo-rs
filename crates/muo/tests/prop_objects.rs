//! `[get` / `[set` through object-valued properties (`Attributes.Luck`, `Skills.Magery.Base`,
//! `Tournament.SuddenDeathRounds`, `Message.Number`) and the real TimeSpan / Poison kinds, on the whole registered
//! program.

use muo_server::World;
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::type_registry::{Phase, invoke_phase};

/// `SkillInfo.Table` with every skill (the test host loads Data/skills.json).
fn skill_table(w: &mut World) {
    let rows: Vec<String> = (0..58)
        .map(|i| format!(r#"{{"SkillID":{i},"Name":"Skill{i}","Title":"T","StrScale":0,"DexScale":0,"IntScale":0,"StatTotal":0,"StrGain":0,"DexGain":0,"IntGain":0,"GainFactor":1.0,"PrimaryStat":"Str","SecondaryStat":"Dex"}}"#))
        .collect();
    w.resources.get_mut::<muo_server::skills::SkillTable>().info =
        muo_data::skill_info::SkillInfo::from_json(&format!("[{}]", rows.join(","))).unwrap();
}

fn world() -> World {
    let mut w = World::new();
    muo::register_content(&mut w.types);
    skill_table(&mut w);
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

fn staff(w: &mut World) -> muo_server::ids::MobileId {
    let from = w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE);
    w.mobile_mut(from).unwrap().access_level = AccessLevel::Developer;
    from
}

fn set(
    w: &mut World,
    from: muo_server::ids::MobileId,
    e: EntityId,
    name: &str,
    value: &str,
) -> String {
    muo_commands::properties::set_value(w, from, e, name, value)
}

fn get(w: &mut World, from: muo_server::ids::MobileId, e: EntityId, name: &str) -> String {
    muo_commands::properties::get_value(w, from, e, name)
}

#[test]
fn aos_bags_are_objects_with_sub_properties() {
    let mut w = world();
    let from = staff(&mut w);
    let sword =
        EntityId::Item(muo_content::factory::make_item(&mut w, "Server.Items.Longsword").unwrap());
    assert_eq!(
        set(&mut w, from, sword, "Attributes.Luck", "100"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, sword, "Attributes.Luck"),
        "Attributes.Luck = 100 (0x64)"
    );
    assert_eq!(
        set(&mut w, from, sword, "WeaponAttributes.HitHarm", "25"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, sword, "weaponattributes.hitharm"),
        "WeaponAttributes.HitHarm = 25 (0x19)"
    );
    assert_eq!(
        set(&mut w, from, sword, "SkillBonuses.Skill_1_Name", "Magery"),
        "Property has been set."
    );
    assert_eq!(
        set(&mut w, from, sword, "SkillBonuses.Skill_1_Value", "5"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, sword, "SkillBonuses.Skill_1_Name"),
        "SkillBonuses.Skill_1_Name = Magery"
    );
    assert_eq!(
        get(&mut w, from, sword, "SkillBonuses.Skill_1_Value"),
        "SkillBonuses.Skill_1_Value = 5"
    );
    assert!(get(&mut w, from, sword, "Attributes.NoSuchThing").contains("not found"));
}

#[test]
fn skills_are_objects_of_a_mobile() {
    let mut w = world();
    let from = staff(&mut w);
    let m = EntityId::Mobile(w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE));
    assert_eq!(
        set(&mut w, from, m, "Skills.Magery.Base", "50"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, m, "Skills.Magery.Base"),
        "Skills.Magery.Base = 50"
    );
    assert_eq!(
        set(&mut w, from, m, "Skills.Magery.Cap", "120"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, m, "Skills.Magery.Cap"),
        "Skills.Magery.Cap = 120"
    );
    // the lock has a private setter
    assert!(set(&mut w, from, m, "Skills.Magery.Lock", "Locked").contains("read only"));
}

#[test]
fn text_definitions_have_read_only_sub_properties() {
    let mut w = world();
    let from = staff(&mut w);
    let tp = EntityId::Item(
        muo_content::factory::make_item(&mut w, "Server.Items.SkillTeleporter").unwrap(),
    );
    assert_eq!(get(&mut w, from, tp, "Message"), "Message = null");
    assert_eq!(
        set(&mut w, from, tp, "Message", "1060847"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, tp, "Message"),
        "Message = 1060847 (0x102FEF)"
    );
    assert_eq!(
        get(&mut w, from, tp, "Message.Number"),
        "Message.Number = 1060847 (0x102FEF)"
    );
    assert!(set(&mut w, from, tp, "Message.Number", "5").contains("read only"));
    assert_eq!(
        set(&mut w, from, tp, "Message", "Hail, traveller."),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, tp, "Message.String"),
        "Message.String = \"Hail, traveller.\""
    );
    assert_eq!(
        set(&mut w, from, tp, "Message", "(-null-)"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, tp, "Message"), "Message = null");
    assert_eq!(
        get(&mut w, from, tp, "Message.Number"),
        "Property 'Message' is null."
    );
}

#[test]
fn time_spans_and_poisons_parse() {
    let mut w = world();
    let from = staff(&mut w);
    let tp = EntityId::Item(
        muo_content::factory::make_item(&mut w, "Server.Items.SkillTeleporter").unwrap(),
    );
    assert_eq!(
        set(&mut w, from, tp, "Delay", "00:00:05"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, tp, "Delay"), "Delay = 00:00:05");
    assert_ne!(
        set(&mut w, from, tp, "Delay", "notaspan"),
        "Property has been set."
    );

    let sword =
        EntityId::Item(muo_content::factory::make_item(&mut w, "Server.Items.Longsword").unwrap());
    assert_eq!(
        set(&mut w, from, sword, "Poison", "Lethal"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, sword, "Poison"), "Poison = Lethal");
    assert_eq!(
        set(&mut w, from, sword, "Poison", "(-null-)"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, sword, "Poison"), "Poison = null");

    let m = EntityId::Mobile(w.new_mobile(&muo_content::player_mobile::PLAYER_MOBILE));
    assert_eq!(
        set(&mut w, from, m, "Poison", "Greater"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, m, "Poison"), "Poison = Greater");
}

#[test]
fn detective_boots_level_clamps() {
    let mut w = world();
    let from = staff(&mut w);
    let boots = EntityId::Item(
        muo_content::factory::make_item(&mut w, "Server.Items.DetectiveBoots").unwrap(),
    );
    assert_eq!(
        set(&mut w, from, boots, "Level", "7"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, boots, "Level"), "Level = 2 (0x2)");
    assert_eq!(
        get(&mut w, from, boots, "Attributes.BonusInt"),
        "Attributes.BonusInt = 4 (0x4)"
    );
}

#[test]
fn the_flattened_conpvp_properties_are_nested() {
    let mut w = world();
    let from = staff(&mut w);
    let c = EntityId::Item(
        muo_content::factory::make_item(&mut w, "Server.Engines.ConPVP.TournamentController")
            .unwrap(),
    );
    assert_eq!(
        set(&mut w, from, c, "Tournament.SuddenDeathRounds", "3"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, c, "Tournament.SuddenDeathRounds"),
        "Tournament.SuddenDeathRounds = 3 (0x3)"
    );
    assert!(get(&mut w, from, c, "SuddenDeathRounds").contains("not found"));
    assert_eq!(
        set(&mut w, from, c, "Tournament.SuddenDeath", "00:10:00"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, c, "Tournament.SuddenDeath"),
        "Tournament.SuddenDeath = 00:10:00"
    );

    let a = EntityId::Item(
        muo_content::factory::make_item(&mut w, "Server.Engines.ConPVP.ArenaController").unwrap(),
    );
    assert_eq!(
        set(&mut w, from, a, "Arena.Points.EdgeWest", "(1, 2, 3)"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, a, "Arena.Points.EdgeWest"),
        "Arena.Points.EdgeWest = (1, 2, 3)"
    );
    assert_eq!(
        set(&mut w, from, a, "Arena.Bounds", "(1, 2)+(30, 40)"),
        "Property has been set."
    );
    assert_eq!(
        get(&mut w, from, a, "Arena.Bounds"),
        "Arena.Bounds = (1, 2)+(29, 38)"
    );
}

#[test]
fn spawner_members_need_developer_and_parse() {
    let mut w = world();
    let from = staff(&mut w);
    let sp = EntityId::Item(
        muo_content::factory::make_item(&mut w, "Server.Engines.Spawners.Spawner").unwrap(),
    );
    assert_eq!(
        set(&mut w, from, sp, "MinDelay", "00:00:30"),
        "Property has been set."
    );
    assert_eq!(get(&mut w, from, sp, "MinDelay"), "MinDelay = 00:00:30");
    assert_eq!(set(&mut w, from, sp, "Team", "4"), "Property has been set.");
    assert_eq!(get(&mut w, from, sp, "Team"), "Team = 4 (0x4)");
    assert!(get(&mut w, from, sp, "Running").starts_with("Running = "));
    assert_eq!(
        set(&mut w, from, sp, "SpawnBounds", "(1, 2, 0)+(10, 20, 5)"),
        "Property has been set."
    );
    assert!(get(&mut w, from, sp, "SpawnBounds").starts_with("SpawnBounds = "));
    w.mobile_mut(from).unwrap().access_level = AccessLevel::GameMaster;
    assert!(get(&mut w, from, sp, "Team").contains("must be at least Developer"));
}
