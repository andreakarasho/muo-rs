//! Hook fields whose two ends sit in different crates, set where this crate sees both. Each
//! assignment names the call it stands for.

use muo_content::gumps::send_gump_to_mobile;
use muo_server::World;
use muo_server::ids::EntityId;

fn send(w: &mut World, to: muo_server::ids::MobileId, g: impl muo_content::gumps::Gump + 'static) {
    send_gump_to_mobile(w, to, Box::new(g), false);
}

/// Configure phase.
pub fn configure(w: &mut World) {
    commands(w);
    game(w);
    gumps(w);
    spells(w);
    engines(w);
    items(w);
    quests_and_harvest(w);
    more_gumps(w);
    houses_and_tables(w);
}

/// `[Furniture]` classes (`FurnitureAttribute.Check(item)`: the exact type, the attribute is not inherited).
const FURNITURE: &[&str] = &[
    "Armoire",
    "BambooChair",
    "BambooScreen",
    "CherryArmoire",
    "ChestOfHeirlooms",
    "CommodityDeedBox",
    "DecorativeBox",
    "Drawer",
    "Easel",
    "ElegantArmoire",
    "ElegantLowTable",
    "ElvenPodium",
    "EmptyBookcase",
    "FancyArmoire",
    "FancyDrawer",
    "FancyElvenArmoire",
    "FancyWoodenChairCushion",
    "FinishedWoodenChest",
    "FootStool",
    "FullBookcase",
    "GiantReplicaAcorn",
    "GiftBox",
    "GildedWoodenChest",
    "GreenStocking",
    "LargeCrate",
    "LargeTable",
    "MapleArmoire",
    "MediumCrate",
    "Nightstand",
    "OrnateWoodenChest",
    "PlainLowTable",
    "PlainWoodenChest",
    "RarewoodChest",
    "RedArmoire",
    "RedStocking",
    "ShojiScreen",
    "ShortCabinet",
    "ShortMusicStand",
    "SimpleElvenArmoire",
    "SmallCrate",
    "Stool",
    "TallCabinet",
    "TallMusicStand",
    "Throne",
    "WoodenBench",
    "WoodenBox",
    "WoodenChair",
    "WoodenChairCushion",
    "WoodenChest",
    "WoodenFootLocker",
    "WoodenThrone",
    "WritingTable",
    "YewWoodTable",
];

fn is_furniture(w: &World, item: muo_server::ids::ItemId) -> bool {
    w.item(item).is_some_and(|i| {
        i.ty.full_name
            .strip_prefix("Server.Items.")
            .is_some_and(|n| FURNITURE.contains(&n))
    })
}

/// The cliloc table (`Localization`), kept in the world behind a `RefCell` because `GetText` loads a language on
/// first use and the callers only hold a `&World`.
#[derive(Default)]
struct LocalizationTable(std::cell::RefCell<Option<muo_data::localization::Localization>>);

fn houses_and_tables(w: &mut World) {
    use muo_commands::teleporters::DesignInsertResult as R;
    use muo_server::ids::{EntityId, ItemId};

    // FurnitureAttribute.Check(item)
    w.resources
        .get_mut::<crate::harvest::HarvestHooks>()
        .is_furniture = Some(is_furniture);
    w.resources
        .get_mut::<muo_items::skill_items::dyes::DyeHooks>()
        .is_furniture = Some(is_furniture);

    // Localization.GetText(number) and Localization.Clear()
    muo_content::items::equip_hooks::hooks_mut(w).localized_text = Some(|w, number| {
        let t = w.resources.get::<LocalizationTable>()?;
        let mut t = t.0.borrow_mut();
        let dirs = w
            .resources
            .get::<muo_data::DataDirectories>()
            .cloned()
            .unwrap_or_default();
        let loc = t.get_or_insert_with(|| muo_data::localization::Localization::new(dirs));
        loc.get_text(number, None).ok().flatten().map(str::to_owned)
    });
    w.resources.insert(LocalizationTable::default());
    // `Localization.Format(number, lang, $"{arg}")` (empty when the cliloc does not exist): the SOS message gump.
    w.resources
        .get_mut::<muo_items::skill_items::fishing::FishingHooks>()
        .localized_format = Some(|w, number, lang, arg| {
        let Some(t) = w.resources.get::<LocalizationTable>() else {
            return String::new();
        };
        let mut t = t.0.borrow_mut();
        let dirs = w
            .resources
            .get::<muo_data::DataDirectories>()
            .cloned()
            .unwrap_or_default();
        let loc = t.get_or_insert_with(|| muo_data::localization::Localization::new(dirs));
        loc.format(number, lang, &[arg.into()]).unwrap_or_default()
    });
    let c = w.resources.get_mut::<muo_commands::hooks::CommandHooks>();
    c.clear_localization = Some(|w| {
        if let Some(t) = w.resources.get::<LocalizationTable>()
            && let Some(l) = t.0.borrow_mut().as_mut()
        {
            l.clear();
        }
    });

    // Sextant.ReverseLookup(map, xLong, yLat, xMins, yMins, xEast, ySouth)
    c.sextant_reverse_lookup = Some(muo_items::skill_items::runebook::sextant_reverse_lookup);

    // BaseHouse.AllHouses: the house that has the item locked down or secured; its PropertiesGump
    c.trace_lockdown = Some(|w, from, item| {
        let all = muo_multis::house::statics_ref(w)
            .map(|s| s.all_houses.clone())
            .unwrap_or_default();
        let Some(house) = all.into_iter().find(|&h| {
            muo_multis::house::storage::has_secure_item(w, h, item)
                || muo_multis::house::storage::has_locked_down_item(w, h, item)
        }) else {
            return false;
        };
        let g = muo_gumps::m_z::props::properties::new(
            w,
            from,
            muo_gumps::m_z::props::host::PropObj::Entity(EntityId::Item(house)),
        );
        send(w, from, g);
        true
    });

    // DesignInsertCommand.ProcessInsert after the item checks
    c.design_insert = Some(|w, item: ItemId| {
        let Some(house) = muo_multis::house::access::find_house_at_item(w, item)
            .filter(|&h| muo_multis::foundation::is_foundation(w, h))
        else {
            return (R::NotInHouse, None);
        };
        let (Some(it), Some(h)) = (w.item(item), w.item(house)) else {
            return (R::NotInHouse, None);
        };
        let (item_id, x, y, z) = (
            it.item_id,
            it.location.x - h.location.x,
            it.location.y - h.location.y,
            it.location.z - h.location.z,
        );
        // TryInsertIntoState: the bounds check, `Components.Add`, `OnRevised()`
        let try_insert = |fd: &mut muo_multis::foundation::FoundationData,
                          td: &muo_data::TileData,
                          design: bool| {
            let slot = if design {
                fd.design.take()
            } else {
                fd.current.take()
            };
            let mut s = slot.unwrap_or_default();
            let (mn, mx) = (s.components.min(), s.components.max());
            let ok = !(x < mn.x || y < mn.y || x > mx.x || y > mx.y);
            if ok {
                s.components.add(item_id, x, y, z, td);
                fd.revise(&mut s);
            }
            if design {
                fd.design = Some(s)
            } else {
                fd.current = Some(s)
            }
            ok
        };
        let inside = muo_multis::foundation::edit(w, house, |fd, td| {
            let ok = try_insert(fd, td, false);
            if ok {
                try_insert(fd, td, true);
            }
            ok
        });
        if inside != Some(true) {
            return (R::OutsideHouseBounds, Some(house));
        }
        if let Some(ty) = w.item(item).map(|i| i.ty) {
            (ty.delete)(w, item);
        }
        (R::Valid, Some(house))
    });

    // MapUO: `Party.Get(from)` and `from.Guild as Guild` member lists
    let m = w
        .resources
        .get_mut::<muo_content::network::map_uo::MapUoHooks>();
    m.guild_members = Some(|w, from| {
        let g = w.mobile(from)?.guild?;
        let d = muo_content::misc::guild_class::data(w, g)?;
        Some(d.members.iter().map(|&m| Some(m)).collect())
    });
    m.party_members = Some(|w, from| {
        let p = crate::party::get(w, from)?;
        Some(
            crate::party::state(w, p)?
                .members
                .iter()
                .map(|i| Some(i.mobile))
                .collect(),
        )
    });
}

fn commands(w: &mut World) {
    let c = w.resources.get_mut::<muo_commands::hooks::CommandHooks>();
    // new StaffRobe(accessLevel)
    c.new_staff_robe = Some(|w, level| {
        Some(muo_items::suits::staff_robe_new(
            w,
            &muo_items::suits::STAFF_ROBE,
            level,
        ))
    });
    // new Teleporter(destination, destinationMap)
    c.new_teleporter = Some(|w, dest, map| {
        Some(muo_items::misc::teleporter::teleporter_new(
            w,
            &muo_items::misc::teleporter::TELEPORTER,
            dest,
            map,
            false,
        ))
    });
    // DesignContext.Check(m)
    c.design_context_check = Some(muo_multis::foundation::check_context);
    // HouseSign.Owner.RefreshDecay()
    c.refresh_house = Some(|w, sign| {
        if !w
            .item(sign)
            .is_some_and(|i| i.ty.is(&muo_multis::sign::HOUSE_SIGN))
        {
            return None;
        }
        Some(muo_multis::sign::house_of(w, sign).is_some_and(|h| {
            muo_multis::house::decay::refresh_decay(w, h);
            true
        }))
    });
    // Spawner.Reset(), ISpawner { Running: true }.Respawn(), BaseSpawner.OnDoubleClick(from)
    c.spawner_reset = Some(crate::spawners::core::reset);
    c.spawner_respawn = Some(|w, e| {
        if let EntityId::Item(i) = e
            && crate::spawners::data(w, i).is_some_and(|d| d.running)
        {
            crate::spawners::core::respawn(w, i);
        }
    });
    c.spawner_open = Some(|w, from, spawner| {
        let ty = w.item(spawner).map(|i| i.ty);
        if let Some(ty) = ty {
            (ty.on_double_click)(w, spawner, from);
        }
    });
}

fn game(w: &mut World) {
    let g = w
        .resources
        .get_mut::<muo_content::network::incoming_game::GameHooks>();
    // Fists.StunRequest(from) / Fists.DisarmRequest(from)
    g.fists_stun_request = Some(muo_items::weapons::special::stun_request);
    g.fists_disarm_request = Some(muo_items::weapons::special::disarm_request);
    // Guild.GuildGumpRequest(m)
    g.guild_gump_request = Some(|w, m| {
        if !muo_content::misc::guild::new_guild_system(w) || !w.mobile(m).is_some_and(|x| x.player)
        {
            return;
        }
        match w.mobile(m).and_then(|x| x.guild) {
            None => {
                let g = muo_gumps::a_l::new_guilds::create_guild_default(w, m);
                send(w, m, g);
            }
            Some(guild) => muo_gumps::a_l::new_guilds::open_info_gump(w, m, guild),
        }
    });
}

fn gumps(w: &mut World) {
    // m.SendGump(new YoungDungeonWarningGump())
    w.resources
        .get_mut::<muo_content::regions::RegionHooks>()
        .young_dungeon_warning = Some(|w, m| {
        send(w, m, muo_gumps::m_z::simple::new_young_dungeon_warning());
    });
    // TransformationSpellHelper.UnderTransformation(m, typeof(AnimalForm))
    w.resources
        .get_mut::<muo_content::regions::RegionHooks>()
        .under_animal_form = Some(muo_spells::ninjitsu::animal_form::under_transformation);
}

fn spells(w: &mut World) {
    use muo_server::ids::MobileId;
    let h = muo_content::spells::hooks::hooks_mut(w);
    // ArcaneGem.ConsumeCharges(caster, count)
    h.arcane_gem_consume_charges = Some(muo_items::misc::gadgets::arcane_gem_consume_charges);
    // WeaponAbility.ClearCurrentAbility(m)
    h.clear_current_weapon_ability =
        Some(muo_content::items::weapon_ability::clear_current_ability);
    // Feint.GetDamageReduction(from, target, out reduction)
    h.feint_damage_reduction = Some(|w, from, target| {
        from.and_then(|f| muo_content::items::weapon_ability::feint_damage_reduction(w, f, target))
    });
    // item is Moongate { Dispellable: false }
    h.moongate_not_dispellable = Some(muo_items::misc::moongates::not_dispellable);
    // item is RecallRune, rune.Mark(from)
    h.is_recall_rune = Some(|w, i| {
        w.item(i)
            .is_some_and(|x| x.ty.is(&muo_items::skill_items::runebook::RECALL_RUNE))
    });
    h.rune_mark = Some(muo_items::skill_items::runebook::mark);
    // BaseFormTalisman.EntryEnabled(m, type)
    h.form_talisman_entry_enabled = Some(muo_items::special::anniv::entry_enabled);
    // AnimateDeadSpell.Target: MaabusCoffinComponent and the Dark Tides quest
    h.animate_maabus_coffin = Some(crate::quests::dark_tides_mobs::animate_maabus_coffin);
    // Faction.Find(m) == Faction.Find(caster) on the faction facet (Felucca)
    h.faction_same_as_caster = Some(|w, m, caster| {
        w.mobile(m)
            .is_some_and(|x| x.map == Some(muo_server::map::MapId::FELUCCA))
            && crate::factions::find(w, m)
                .is_some_and(|f| Some(f) == crate::factions::find(w, caster))
    });
    // `from.NetState?.SupportsExpansion(Expansion.ML) == true`: no NetState is false (the PlayerMobile test is the caller's).
    h.net_supports_ml = Some(|w, m| {
        crate::craft::item::supports_expansion(w, m, muo_data::expansion::Expansion::ML)
    });
    // GiftOfLife: the master's PetResurrectGump if in range, else the first friend's in range
    h.pet_resurrect_gump = Some(|w, pet, hits_scalar| {
        let at = |w: &World, m: MobileId| {
            w.mobile(m)
                .filter(|x| x.net_state.is_some())
                .map(|x| x.location)
        };
        let pet_loc = w.mobile(pet).map(|x| x.location).unwrap_or_default();
        let in_range = |w: &World, m: MobileId| {
            at(w, m).is_some_and(|l| muo_server::mobile::part1::combat::in_update_range(pet_loc, l))
        };
        if let Some(master) =
            muo_content::creature::control::get_master(w, pet).filter(|&m| in_range(w, m))
        {
            let g = muo_gumps::m_z::simple::new_pet_resurrect(w, pet, hits_scalar);
            send(w, master, g);
            return;
        }
        let friends = muo_content::creature::data::cd(w, pet)
            .and_then(|d| d.friends.clone())
            .unwrap_or_default();
        if let Some(f) = friends.into_iter().find(|&f| in_range(w, f)) {
            let g = muo_gumps::m_z::simple::new_pet_resurrect(w, pet, 0.0);
            send(w, f, g);
        }
    });
}

fn engines(w: &mut World) {
    use crate::canned_evil::spawn;
    // ValorVirtue: the spawn of an IdolOfTheChampion
    let c = w.resources.get_mut::<crate::virtues::ChampionHooks>();
    c.idol_spawn = Some(|w, idol| {
        let s = spawn::idol_spawn(w, idol)?;
        let d = spawn::data(w, s)?;
        Some(crate::virtues::IdolSpawn {
            has_been_advanced: d.has_been_advanced,
            active: d.active,
            champion_spawned: d.champion.is_some(),
            sub_level: spawn::get_sub_level(w, s),
        })
    });
    // idol.Spawn.HasBeenAdvanced = true; idol.Spawn.AdvanceLevel()
    c.advance_level = Some(|w, idol| {
        if let Some(s) = spawn::idol_spawn(w, idol) {
            if let Some(d) = spawn::data_mut(w, s) {
                d.has_been_advanced = true;
            }
            spawn::advance_level(w, s);
        }
    });
    // idol.Spawn.Start(); idol.Spawn.ActivatedByValor = true
    c.start_by_valor = Some(|w, idol| {
        if let Some(s) = spawn::idol_spawn(w, idol) {
            spawn::start(w, s);
            if let Some(d) = spawn::data_mut(w, s) {
                d.activated_by_valor = true;
            }
        }
    });
    let c = w.resources.get_mut::<crate::canned_evil::CannedEvilHooks>();
    // PowerScroll.CreateRandomNoCraft(min, max)
    c.create_random_power_scroll = Some(|w, min, max| {
        Some(muo_items::special::scrolls::power_scroll_create_random(
            w, min, max,
        ))
    });
    // (champion as BaseChampion)?.GetArtifact()
    c.champion_artifact = Some(muo_mobiles::npcs::special::champions::get_artifact);
    // `mob.Guild as Guild`: the old guild class's leader, type, enemies, alliance and members
    w.resources
        .get_mut::<crate::factions::flow::GuildHooks>()
        .guild_of = Some(|w, m| {
        use muo_content::misc::guild_class as gc;
        let id = w.mobile(m)?.guild?;
        let d = gc::data(w, id)?;
        Some(crate::factions::flow::GuildInfo {
            id,
            leader: d.leader,
            ty: gc::guild_type(w, id),
            enemies: d.enemies.len(),
            has_alliance: gc::alliance(w, id).is_some(),
            members: d.members.clone(),
        })
    });
    // guild.RemoveMember(mob)
    w.resources
        .get_mut::<crate::factions::flow::GuildHooks>()
        .remove_member =
        Some(|w, g, m| muo_content::misc::guild_class::remove_member(w, g, m, 1018028));
}

fn items(w: &mut World) {
    // `targ is PlantItem item` then `item.Pour(from, this)` of a beverage
    w.resources
        .get_mut::<muo_items::food::beverage::BeverageHooks>()
        .plant_pour = Some(|w, item, from, bev| {
        let plant = crate::plants::item::is_plant(w, item);
        if plant {
            crate::plants::item::pour(w, item, from, bev);
        }
        plant
    });
    // Chaos / Order shields: Guild.Type
    w.resources
        .get_mut::<muo_items::armor::special::ShieldGuildHooks>()
        .guild_type = Some(|w, g| Some(muo_content::misc::guild_class::guild_type(w, g)));
    let a = w
        .resources
        .get_mut::<muo_items::construction::paired::AnkhHooks>();
    // new ResurrectGump(m, ResurrectMessage.VirtueShrine) / new TithingGump(from)
    a.resurrect_gump = Some(|w, m| {
        send(
            w,
            m,
            muo_gumps::m_z::resurrect::new(
                Some(m),
                muo_gumps::m_z::resurrect::ResurrectMessage::VirtueShrine,
                false,
                0.0,
                0,
            ),
        );
    });
    a.tithing_gump = Some(|w, m| {
        let g = muo_gumps::m_z::simple::new_tithing(w, m, 0);
        send(w, m, g);
    });
    // new ResurrectGump(m) of the stones, new ResurrectGump(m, ResurrectMessage.Healer) of the quests
    w.resources
        .get_mut::<muo_content::special_systems::stones::StoneHooks>()
        .show_resurrect_gump = Some(|w, m| {
        send(
            w,
            m,
            muo_gumps::m_z::resurrect::new(
                Some(m),
                muo_gumps::m_z::resurrect::ResurrectMessage::Generic,
                false,
                0.0,
                0,
            ),
        )
    });
    w.resources
        .get_mut::<crate::quests::hooks::QuestHooks>()
        .resurrect_gump_healer = Some(|w, m| {
        send(
            w,
            m,
            muo_gumps::m_z::resurrect::new(
                Some(m),
                muo_gumps::m_z::resurrect::ResurrectMessage::Healer,
                false,
                0.0,
                0,
            ),
        );
    });
    // BaseVendor healers: ResurrectGump(healer, price) when price > 0, else (m, Healer)
    w.resources
        .get_mut::<muo_content::vendor::VendorHooks>()
        .resurrect_gump = Some(|w, m, healer, price| {
        let g = if price > 0 {
            muo_gumps::m_z::resurrect::new_with_price(Some(healer), price)
        } else {
            muo_gumps::m_z::resurrect::new(
                Some(m),
                muo_gumps::m_z::resurrect::ResurrectMessage::Healer,
                false,
                0.0,
                0,
            )
        };
        send(w, m, g);
    });
}

fn quests_and_harvest(w: &mut World) {
    // VirtueSystem.Award(player, virtue, points, ref gainedPath): true when awarded
    w.resources
        .get_mut::<crate::quests::hooks::QuestHooks>()
        .virtue_award = Some(|w, m, virtue, points| {
        crate::virtues::VirtueName::from_i32(virtue)
            .is_some_and(|v| crate::virtues::award(w, m, v, points).is_some())
    });
    // TreasureMap.OnBeginDig(from)
    w.resources
        .get_mut::<crate::harvest::HarvestHooks>()
        .treasure_map_begin_dig = Some(muo_items::maps::treasure::on_begin_dig);
}

fn more_gumps(w: &mut World) {
    // new ClientGump(from, targetState); PlayerMobile.TithingPoints += amount
    let m = muo_gumps::m_z::hooks::hooks_mut(w);
    m.open_client_gump = Some(|w, from, state| {
        let g = muo_gumps::a_l::client::new(w, from, Some(state), "");
        send(w, from, g);
    });
    m.tithing_points = Some(|w, pm| w.mobile(pm).map_or(0, |x| x.tithing_points));
    m.add_tithing_points = Some(|w, pm, amount| {
        let cur = w.mobile(pm).map_or(0, |x| x.tithing_points);
        muo_server::mobile::part1::props_a::set_tithing_points(w, pm, cur + amount);
    });
    // new PresetMap(PresetMapEntry.Table[index]) and the table's names
    let v = w.resources.get_mut::<muo_content::vendor::VendorHooks>();
    v.make_preset_map = Some(|w, i| {
        Some(muo_items::maps::preset_map_new(
            w,
            &muo_items::maps::PRESET_MAP,
            i as i32,
        ))
    });
    v.preset_map_names = Some(|_| {
        muo_items::maps::PRESET_MAP_TABLE
            .iter()
            .map(|e| e.name)
            .collect()
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configure_sets_the_cross_crate_hooks() {
        let mut w = World::new();
        configure(&mut w);
        let g = w
            .resources
            .get::<muo_content::network::incoming_game::GameHooks>()
            .unwrap();
        // Fists.StunRequest / DisarmRequest (the client's stun and disarm toggles)
        assert!(
            g.fists_stun_request.is_some()
                && g.fists_disarm_request.is_some()
                && g.guild_gump_request.is_some()
        );
        let c = w
            .resources
            .get::<muo_commands::hooks::CommandHooks>()
            .unwrap();
        assert!(
            c.new_staff_robe.is_some() && c.new_teleporter.is_some() && c.spawner_reset.is_some()
        );
        assert!(
            muo_content::spells::hooks::hooks(&w)
                .unwrap()
                .arcane_gem_consume_charges
                .is_some()
        );
        assert!(
            w.resources
                .get::<muo_items::armor::special::ShieldGuildHooks>()
                .unwrap()
                .guild_type
                .is_some()
        );
    }

    /// A foundation (a 3x3 footprint of multi 0x20) standing at 1000,1000 on Felucca.
    fn foundation_world() -> (World, muo_server::ids::MobileId, muo_server::ids::ItemId) {
        use muo_data::client::multi_data::{MultiComponentList, MultiTileEntry};
        use muo_server::item::ops;
        let mut w = World::new();
        configure(&mut w);
        muo_server::item::multi::register(&mut w);
        w.maps.add(muo_server::map::Map::new(
            0,
            0,
            0,
            7168,
            4096,
            0,
            "Felucca",
            Default::default(),
        ));
        let entries: Vec<MultiTileEntry> = (-1..=1)
            .flat_map(|x| {
                (-1..=1).map(move |y| MultiTileEntry::new(0x31F4, x, y, 0, Default::default()))
            })
            .collect();
        w.maps
            .multi_data
            .set_components(0x20, MultiComponentList::from_entries(&entries));
        let owner = w.new_mobile(&muo_server::mobile::MOBILE_BASE);
        let house = muo_multis::foundation::new_foundation(&mut w, owner, 0x20, 100, 10);
        ops::move_to_world_map(
            &mut w,
            house,
            muo_primitives::Point3D::new(1000, 1000, 0),
            Some(muo_server::map::MapId::FELUCCA),
        );
        (w, owner, house)
    }

    fn loose_item(w: &mut World, at: muo_primitives::Point3D) -> muo_server::ids::ItemId {
        use muo_server::item::{ITEM_BASE, ops};
        let it = ops::new_item_with_id(w, &ITEM_BASE, 0x1234);
        ops::move_to_world_map(w, it, at, Some(muo_server::map::MapId::FELUCCA));
        it
    }

    /// `DesignInsertCommand.ProcessInsert`: an item standing in the house goes into both the current and the design
    /// state (one more tile each, at z 5 so `Add` replaces no floor tile; a new revision) and is deleted.
    #[test]
    fn design_insert_moves_an_item_in_the_house_into_both_states() {
        use muo_commands::teleporters::{DesignInsertResult as R, process_insert};
        let (mut w, _, house) = foundation_world();
        let min = muo_multis::foundation::current_state(&mut w, house)
            .components
            .min();
        let it = loose_item(
            &mut w,
            muo_primitives::Point3D::new(1000 + min.x, 1000 + min.y, 5),
        );
        let before = muo_multis::foundation::fd(&w, house).unwrap().clone();
        let (cur, des) = (before.current.unwrap(), before.design.unwrap());

        assert_eq!(
            process_insert(&mut w, Some(it), false),
            (R::Valid, Some(house))
        );

        let after = muo_multis::foundation::fd(&w, house).unwrap();
        let (acur, ades) = (
            after.current.as_ref().unwrap(),
            after.design.as_ref().unwrap(),
        );
        assert_eq!(acur.components.list.len(), cur.components.list.len() + 1);
        assert_eq!(ades.components.list.len(), des.components.list.len() + 1);
        assert!(acur.components.list.iter().any(|e| e.item_id == 0x1234
            && (e.offset_x, e.offset_y, e.offset_z) == (min.x as i16, min.y as i16, 5)));
        assert!(
            acur.revision > cur.revision && ades.revision > des.revision,
            "OnRevised"
        );
        assert!(w.item(it).is_none_or(|i| i.deleted()), "item.Delete()");
    }

    #[test]
    fn design_insert_refuses_invalid_inserts() {
        use muo_commands::teleporters::{DesignInsertResult as R, process_insert};
        let (mut w, _, house) = foundation_world();
        // no item
        assert_eq!(process_insert(&mut w, None, false), (R::InvalidItem, None));
        // `staticsOnly && item is not Static`
        let min = muo_multis::foundation::current_state(&mut w, house)
            .components
            .min();
        let it = loose_item(
            &mut w,
            muo_primitives::Point3D::new(1000 + min.x, 1000 + min.y, 0),
        );
        assert_eq!(
            process_insert(&mut w, Some(it), true),
            (R::InvalidItem, None)
        );
        assert!(
            !w.item(it).unwrap().deleted(),
            "nothing is inserted or deleted"
        );
        // an item outside every house
        let far = loose_item(&mut w, muo_primitives::Point3D::new(3000, 3000, 0));
        assert_eq!(
            process_insert(&mut w, Some(far), false),
            (R::NotInHouse, None)
        );
        assert!(!w.item(far).unwrap().deleted());
    }

    /// `TraceLockdownCommand`: the first of `BaseHouse.AllHouses` with the item in `HasSecureItem`/`HasLockedDownItem`
    /// opens its PropertiesGump; with none, `No house was found.`
    #[test]
    fn trace_lockdown_finds_the_house_that_holds_the_item() {
        let (mut w, owner, house) = foundation_world();
        let trace = w
            .resources
            .get::<muo_commands::hooks::CommandHooks>()
            .unwrap()
            .trace_lockdown
            .unwrap();
        let it = loose_item(&mut w, muo_primitives::Point3D::new(1000, 1000, 0));
        assert!(!trace(&mut w, owner, it), "not locked down in any house");
        muo_multis::house::storage::set_lockdown(&mut w, house, it, true, false);
        assert!(muo_multis::house::storage::has_locked_down_item(
            &w, house, it
        ));
        assert!(trace(&mut w, owner, it), "locked down in the house");
        muo_multis::house::storage::set_lockdown(&mut w, house, it, false, false);
        assert!(!trace(&mut w, owner, it), "released again");
    }
}
