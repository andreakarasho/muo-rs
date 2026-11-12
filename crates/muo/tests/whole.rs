//! Checks over the whole registered program, which no single crate's tests can see.

use std::panic::{AssertUnwindSafe, catch_unwind};

use muo_io::{SaveReader, SaveWriter};
use muo_primitives::Serial;
use muo_server::{Item, ItemId, Mobile, MobileId, World};

fn world() -> World {
    let mut w = World::new();
    muo::register_content(&mut w.types);
    w
}

fn item_bytes(w: &World, id: ItemId) -> Vec<u8> {
    let ty = w.item(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn mobile_bytes(w: &World, id: MobileId) -> Vec<u8> {
    let ty = w.mobile(id).unwrap().ty;
    let mut out = SaveWriter::new(true);
    (ty.serialize)(w, id, &mut out);
    out.into_vec()
}

fn fail_report(what: &str, bad: &[String]) {
    assert!(
        bad.is_empty(),
        "{what}: {} failed:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

/// Classes that write a different record after a reload (observable in real saves), kept for save compatibility.
/// Anything else that differs is a bug here. An entry that now round-trips fails the test, so the list stays honest.
const CHANGES_ON_RELOAD: &[(&str, &str)] = &[
    // BaseClothing: the loader's `_resource` is 0 when the record has no Resource flag (no `else` in the generated
    // reader, the serialization constructor skips `_resource = DefaultResource`), so the next save writes the flag
    // because 0 != DefaultResource (RegularLeather).
    ("Server.Items.Boots", "BaseClothing resource"),
    ("Server.Items.Shoes", "BaseClothing resource"),
    ("Server.Items.Sandals", "BaseClothing resource"),
    ("Server.Items.ElvenBoots", "BaseClothing resource"),
    ("Server.Items.ThighBoots", "BaseClothing resource"),
    ("Server.Items.DetectiveBoots", "BaseClothing resource"),
    ("Server.Items.LeatherNinjaBelt", "BaseClothing resource"),
    // FillableContainer: `[AfterDeserialization]` starts the respawn timer (random delay), which a new one has not
    // (it is saved as `RespawnTimer?.Next`).
    (
        "Server.Items.FillableLargeCrate",
        "FillableContainer respawn timer",
    ),
    (
        "Server.Items.FillableSmallCrate",
        "FillableContainer respawn timer",
    ),
    (
        "Server.Items.FillableWoodenBox",
        "FillableContainer respawn timer",
    ),
    (
        "Server.Items.FillableMetalBox",
        "FillableContainer respawn timer",
    ),
    (
        "Server.Items.FillableBarrel",
        "FillableContainer respawn timer",
    ),
    (
        "Server.Items.FillableMetalChest",
        "FillableContainer respawn timer",
    ),
    (
        "Server.Items.FillableMetalGoldenChest",
        "FillableContainer respawn timer",
    ),
    (
        "Server.Items.FillableWoodenChest",
        "FillableContainer respawn timer",
    ),
    (
        "Server.Items.LibraryBookcase",
        "FillableContainer respawn timer",
    ),
    // BaseFruitTreeAddon: a tree made with 0 fruits gets `Utility.RandomMinMax(1, 4)` in `[AfterDeserialization]`.
    ("Server.Items.AppleTreeAddon", "fruit respawn on load"),
    ("Server.Items.PeachTreeAddon", "fruit respawn on load"),
    // Neira: `[AfterDeserialization]` runs `CheckSpeedBoost`, which sets `CurrentSpeed = ActiveSpeed`.
    ("Server.Mobiles.Neira", "CurrentSpeed set on load"),
];

/// An entry of [`CHANGES_ON_RELOAD`] when the record differs, else (stale entry) an error.
fn differs(name: &str, same: bool, msg: impl FnOnce() -> String) -> Result<(), String> {
    match (CHANGES_ON_RELOAD.iter().find(|(n, _)| *n == name), same) {
        (None, true) | (Some(_), false) => Ok(()),
        (None, false) => Err(msg()),
        (Some(_), true) => Err(
            "allow-listed as changing on reload, but it round-trips: remove the entry".to_string(),
        ),
    }
}

/// Saving, loading through the empty entity the loader makes, and saving again gives the same record. A class that
/// reads into state the loader hasn't made (see `new_for_load`) loses data here.
#[test]
fn every_constructible_item_survives_a_load() {
    let mut w = world();
    let mut bad = vec![];
    for (n, ty) in w.types.item_types().to_vec().into_iter().enumerate() {
        let Some(construct) = ty.construct else {
            continue;
        };
        let name = ty.full_name;
        let result = catch_unwind(AssertUnwindSafe(|| {
            let id = construct(&mut w);
            let first = item_bytes(&w, id);
            let copy = w.add_item(Item::new_for_load(Serial::new(0x7000_0000 + n as u32), ty));
            let mut r = SaveReader::new(&first);
            (ty.deserialize)(&mut w, copy, &mut r).map_err(|e| format!("load: {e:?}"))?;
            if r.remaining() != 0 {
                return Err(format!("{} bytes left unread", r.remaining()));
            }
            let second = item_bytes(&w, copy);
            differs(name, first == second, || {
                format!("record differs: {first:02x?} then {second:02x?}")
            })
        }));
        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => bad.push(format!("{name}: {e}")),
            Err(_) => bad.push(format!("{name}: panicked")),
        }
    }
    fail_report("items", &bad);
}

#[test]
fn every_constructible_mobile_survives_a_load() {
    let mut w = world();
    let mut bad = vec![];
    for (n, ty) in w.types.mobile_types().to_vec().into_iter().enumerate() {
        let Some(construct) = ty.construct else {
            continue;
        };
        let name = ty.full_name;
        let result = catch_unwind(AssertUnwindSafe(|| {
            let id = construct(&mut w);
            let first = mobile_bytes(&w, id);
            let copy = w.add_mobile(Mobile::new_for_load(
                Serial::new(0x0030_0000 + n as u32),
                ty,
                w.tick_count,
            ));
            let mut r = SaveReader::new(&first);
            (ty.deserialize)(&mut w, copy, &mut r).map_err(|e| format!("load: {e:?}"))?;
            if r.remaining() != 0 {
                return Err(format!("{} bytes left unread", r.remaining()));
            }
            let second = mobile_bytes(&w, copy);
            differs(name, first == second, || {
                format!("record differs: {first:02x?} then {second:02x?}")
            })
        }));
        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => bad.push(format!("{name}: {e}")),
            Err(_) => bad.push(format!("{name}: panicked")),
        }
    }
    fail_report("mobiles", &bad);
}

/// Every type name in the Items.idx/Mobiles.idx of the real saves (the `MUO_GOLDEN_SAVE` folder, and any save or
/// backup under the server's Distribution folder, `MUO_DISTRIBUTION`) must resolve to a registered class or alias, or loading drops its objects.
#[test]
fn every_type_in_real_saves_is_registered() {
    use muo_io::serialization::index::read_idx;
    let mut files = vec![];
    let dist = std::env::var_os("MUO_DISTRIBUTION").map(std::path::PathBuf::from);
    let mut stack: Vec<std::path::PathBuf> = std::env::var_os("MUO_GOLDEN_SAVE")
        .map(Into::into)
        .into_iter()
        .chain(
            dist.iter()
                .flat_map(|d| [d.join("Saves"), d.join("Backups")]),
        )
        .collect();
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if matches!(
                p.file_name().and_then(|n| n.to_str()),
                Some("Items.idx" | "Mobiles.idx")
            ) {
                files.push(p);
            }
        }
    }
    if files.is_empty() {
        eprintln!("no real saves here; skipped");
        return;
    }
    let w = world();
    let mut missing = std::collections::BTreeSet::new();
    for f in &files {
        let Ok(idx) = read_idx(&std::fs::read(f).unwrap()) else {
            continue;
        };
        for name in &idx.type_table {
            if w.types.find_type_by_full_name(name, false).is_none() {
                missing.insert(name.clone());
            }
        }
    }
    assert!(
        missing.is_empty(),
        "types of real saves not registered:
{}",
        missing.into_iter().collect::<Vec<_>>().join(
            "
"
        )
    );
}

fn configured_world() -> World {
    let mut w = world();
    muo_server::type_registry::invoke_phase(&mut w, muo_server::type_registry::Phase::Configure);
    w
}

/// A kill by a player gives fame and karma: `Titles.ComputeKillAwards` = fame / 100, -karma / 100 of the dead creature
/// (no Felucca bonus off-map), applied by `AwardFame`/`AwardKarma`. Both hooks were once left unassigned.
#[test]
fn a_kill_awards_fame_and_karma() {
    let mut w = configured_world();
    let orc = muo_content::factory::make_mobile(&mut w, "Server.Mobiles.Orc").unwrap();
    let pm = muo_content::player_mobile::new_player_mobile(&mut w);
    w.mobile_mut(pm).unwrap().player = true; // set when a client is attached
    let (fame, karma) = {
        let m = w.mobile(orc).unwrap();
        (m.fame, m.karma)
    };
    assert!(
        fame > 0 && karma < 0,
        "the Orc has fame and bad karma ({fame}, {karma})"
    );
    let now = w.now;
    w.mobile_mut(orc)
        .unwrap()
        .damage_entries
        .push(muo_server::mobile::DamageEntry {
            damager: pm,
            damage_given: 10_000,
            last_damage: now,
            responsible: vec![],
        });
    muo_content::creature::death::on_death(&mut w, orc, None);
    let p = w.mobile(pm).unwrap();
    assert_eq!((p.fame, p.karma), (fame / 100, -karma / 100));
}

/// Loot weapons, armor and jewelry get magic properties through `LootHooks.apply_aos_attributes` (`BaseRunicTool`).
#[test]
fn runic_attributes_land_on_loot_items() {
    use muo_content::aos::AttrKind;
    let mut w = configured_world();
    let f = muo_content::loot_pack::LootHooks::default().apply_aos_attributes;
    assert!(f.is_none());
    let f = w
        .resources
        .get::<muo_content::loot_pack::LootHooks>()
        .unwrap()
        .apply_aos_attributes
        .expect("set");
    for name in [
        "Server.Items.Katana",
        "Server.Items.PlateChest",
        "Server.Items.GoldRing",
        "Server.Items.Bonnet",
    ] {
        let mut hits = 0;
        for _ in 0..20 {
            let item = muo_content::factory::make_item(&mut w, name).unwrap();
            f(&mut w, item, 0, 4, 40, 100);
            let it = w.item(item).unwrap();
            let any = [
                AttrKind::Attributes,
                AttrKind::Weapon,
                AttrKind::Armor,
                AttrKind::SkillBonuses,
                AttrKind::Element,
            ]
            .iter()
            .any(|k| muo_content::items::attrs(it, *k).is_some_and(|a| !a.is_empty()));
            let bonus = muo_content::items::armor::data(&w, item).is_some_and(|d| {
                d.physical_bonus + d.fire_bonus + d.cold_bonus + d.poison_bonus + d.energy_bonus > 0
            });
            hits += usize::from(any || bonus);
        }
        assert!(hits >= 15, "{name}: only {hits}/20 got properties");
    }
}

/// Hooks that features silently vanish without: unset after `Configure` is a bug, not a feature. Add a hook here when
/// you set one.
#[test]
fn critical_hooks_are_set_after_configure() {
    use muo_content::hooks::{DeathHooks, WorldHooks};
    let w = configured_world();
    let mut unset: Vec<&str> = vec![];
    macro_rules! need {
        ($h:expr; $($f:ident),* $(,)?) => { $( if $h.$f.is_none() { unset.push(concat!(stringify!($f))); } )* }
    }
    let h = w.resources.get::<WorldHooks>().expect("WorldHooks");
    need!(h; compute_kill_awards, award_fame_karma, mount_rider, in_guarded_region, corpse_animated,
        corpse_set_carved, corpse_add_carved_item, corpse_is_criminal_action, waypoint_next);
    let l = w
        .resources
        .get::<muo_content::loot_pack::LootHooks>()
        .expect("LootHooks");
    need!(l; apply_aos_attributes, apply_old_weapon, apply_old_armor, apply_instrument);
    let p = w
        .resources
        .get::<muo_content::creature::packing::PackHooks>()
        .expect("PackHooks");
    need!(p; apply_aos_attributes, set_armor_levels, set_weapon_levels, set_loot_slayer);
    let d = w.resources.get::<DeathHooks>().expect("DeathHooks");
    need!(d; pack_treasure_map);
    let c = muo_content::items::common::hooks(&w).expect("CommonItemHooks");
    need!(c; corpse_notoriety, same_party, criminal_action, award_karma, creature_on_carve);
    let r = w
        .resources
        .get::<muo_content::misc::regen_rates::RegenHooks>()
        .expect("RegenHooks");
    need!(r; horrific_beast, vampiric_embrace, lich_form, dog_or_cat_form, kirin_form, armor_offset,
        creature_not_animated_dead, paragon_or_leviathan);
    let s = w
        .resources
        .get::<muo_content::player_mobile::hooks::SystemHooks>()
        .expect("SystemHooks");
    need!(s; in_jail_region, guarded_region_protects, region_not_young_protected, dungeon_entrance, pet_info,
        pet_auto_stable, pet_claim, creature_master, stealing_classic_mode);
    assert!(unset.is_empty(), "unset hooks: {unset:?}");
    // Not an `Option`: a jailed character may not be deleted (`AccountHandler`), so the default must be replaced.
    let a = w
        .resources
        .get::<muo_content::accounting::AccountHooks>()
        .expect("AccountHooks");
    let default = muo_content::accounting::AccountHooks::default();
    assert!(
        a.in_jail_region as usize != default.in_jail_region as usize,
        "AccountHooks.in_jail_region is the default"
    );
    assert!(
        a.delete_houses as usize != default.delete_houses as usize,
        "AccountHooks.delete_houses is the default"
    );
}

/// `PlayerMobile.IsHarmfulCriminal` on a target with PermaFlags: false, but an Innocent target gets `Delta(Noto)`.
#[test]
fn harmful_to_a_perma_flagged_player_raises_the_noto_delta() {
    use muo_server::mobile::MobileDelta;
    let mut w = configured_world();
    let a = muo_content::player_mobile::new_player_mobile(&mut w);
    let b = muo_content::player_mobile::new_player_mobile(&mut w);
    let c = muo_content::player_mobile::new_player_mobile(&mut w);
    for id in [a, b] {
        w.mobile_mut(id).unwrap().player = true;
        w.mobile_mut(id).unwrap().map = Some(muo_server::map::MapId::FELUCCA);
    }
    muo_content::misc::notoriety::initialize(&mut w);
    w.resources
        .get_mut::<muo_content::player_mobile::hooks::SystemHooks>()
        .stealing_classic_mode = Some(|_| true);
    muo_content::player_mobile::data_mut(&mut w, b)
        .unwrap()
        .perma_flags
        .push(c);
    assert_eq!(
        muo_server::notoriety::compute(&mut w, a, b),
        muo_server::notoriety::INNOCENT
    );
    let ty = w.mobile(a).unwrap().ty;
    assert!(!(ty.is_harmful_criminal)(&mut w, a, b));
    assert!(w.mobile(b).unwrap().delta_flags.contains(MobileDelta::NOTO));
}

/// The supply bags fill their `BagOfReagents(amount)` through its constructor (5000 of each reagent), not the default 50,
/// and every class they drop is registered.
#[test]
fn supply_bags_hold_their_full_kits() {
    let mut w = configured_world();
    for (bag, min_items) in [
        ("Server.Items.AlchemyBag", 3),
        ("Server.Items.ScribeBag", 2),
        ("Server.Items.SmithBag", 11),
        ("Server.Items.BagOfingots", 11),
        ("Server.Items.TailorBag", 8),
    ] {
        let id = muo_content::factory::make_item(&mut w, bag).unwrap();
        assert_eq!(w.item(id).unwrap().items.len(), min_items, "{bag}");
    }
    for bag in ["Server.Items.AlchemyBag", "Server.Items.ScribeBag"] {
        let id = muo_content::factory::make_item(&mut w, bag).unwrap();
        let regs = w
            .item(id)
            .unwrap()
            .items
            .iter()
            .copied()
            .find(|c| w.item(*c).unwrap().ty.full_name == "Server.Items.BagOfReagents")
            .unwrap();
        for r in w.item(regs).unwrap().items.clone() {
            let amount = w.item(r).unwrap().amount;
            assert_eq!(amount, 5000, "{bag}");
        }
    }
}

/// `Victoria.InitSBInfo` adds `SBMage`: the sorceress of the Summoning quest has a buy list.
#[test]
fn victoria_sells_what_the_mage_vendor_sells() {
    let mut w = configured_world();
    let v =
        muo_content::factory::make_mobile(&mut w, "Server.Engines.Quests.Doom.Victoria").unwrap();
    let d = muo_content::vendor::data(&w, v).expect("a vendor");
    assert!(!d.buy_info.is_empty());
}

/// Settings read in a `Configure` take effect in the production phase list (`AccountGold`, `Movement`, `Mobile` were
/// once never configured), and a corpse is `ICarvable` for the knife (it was missing from the interface registry).
#[test]
fn configure_settings_take_effect() {
    let mut w = World::new();
    muo_server::server_setup::register(&mut w.types); // as `setup_with` does before the content
    muo::register_content(&mut w.types);
    let c = &mut w.config;
    let _ = c.set_setting("accountGold.enable", &true);
    let _ = c.set_setting("accountGold.convertOnTrade", &true);
    let _ = c.set_setting("movement.delay.walkFoot", &123_i32);
    let _ = c.set_setting("movement.delay.runMount", &45_i32);
    let _ = c.set_setting("spellCasting.disableCastParalyze", &false);
    let _ = c.set_setting("ethics.enable", &true);
    muo_server::type_registry::invoke_phase(&mut w, muo_server::type_registry::Phase::Configure);

    let g = w
        .resources
        .get::<muo_server::iaccount::AccountGold>()
        .unwrap();
    assert!(g.enabled && g.convert_on_trade && g.convert_on_bank);
    let m = w.resources.get::<muo_server::movement::Movement>().unwrap();
    assert_eq!(
        (m.walk_foot_delay, m.run_mount_delay, m.run_foot_delay),
        (123, 45, 200)
    );
    assert!(
        !w.resources
            .get::<muo_server::mobile::part2::statics::Part2Statics>()
            .unwrap()
            .disable_cast_paralyze
    );

    let corpse = w.add_item(Item::new_for_load(
        Serial::new(0x7000_0001),
        &muo_content::items::corpse::CORPSE,
    ));
    assert!(muo_content::items::interfaces::is_carvable(&w, corpse));
}

/// `Corpse.Carve` reads the dead creature (`dead is BaseCreature`), which was deleted when it died: the knife on a
/// cow's corpse still carves it, once.
#[test]
fn a_knife_carves_the_corpse_of_a_deleted_creature() {
    let mut w = configured_world();
    let cow = muo_content::factory::make_mobile(&mut w, "Server.Mobiles.Cow").unwrap();
    let pm = muo_content::player_mobile::new_player_mobile(&mut w);
    let corpse = muo_content::items::corpse::new_corpse(&mut w, cow, vec![]);
    muo_server::mobile::base::delete(&mut w, cow);
    assert!(w.mobile(cow).is_none(), "the cow is gone from the world");
    muo_content::items::corpse::carve(&mut w, corpse, pm, None);
    assert!(muo_content::items::corpse::carved(&w, corpse), "carved");
}
