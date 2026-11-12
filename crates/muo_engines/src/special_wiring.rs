//! Connects Items/Special (muo_items) to the gumps (muo_gumps) and the faction items: the hook fields each side waits
//! for are set here, because this crate sees both.

use muo_content::gumps::send_gump_to_mobile;
use muo_server::World;

/// Configure phase.
pub fn configure(w: &mut World) {
    configure_npcs(w);
    {
        let f = &mut w.resources.get_mut::<muo_content::skills::SkillHooks>().fns;
        f.is_sigil = Some(crate::factions::items::is_sigil);
        f.steal_sigil = Some(crate::factions::items::steal_sigil);
        f.stop_invisibility_timer = Some(muo_spells::sixth::invisibility::stop_timer);
        f.display_container_to = Some(|w, item, from| {
            if let Some(ty) = w.item(item).map(|i| i.ty) {
                (ty.display_to)(w, item, from);
            }
        });
    }
    muo_content::spells::hooks::hooks_mut(w).sigil_exists_on =
        Some(crate::factions::items::exists_on);
    w.resources
        .get_mut::<crate::canned_evil::CannedEvilHooks>()
        .properties_gump = Some(crate::special_gumps::open_properties);
    w.resources
        .get_mut::<muo_content::login::shard_poller::ShardPollerHooks>()
        .properties_gump = Some(|w, from, item| {
        crate::special_gumps::open_properties(w, from, muo_server::ids::EntityId::Item(item))
    });
    let h = muo_items::special::hooks::hooks_mut(w);
    h.sigil_exists_on = Some(crate::factions::items::exists_on);
    h.heritage_token_gump = Some(muo_gumps::a_l::heritage_token::display_to);
    h.aquarium_gump = Some(crate::special_gumps::display_aquarium);
    h.aquarium_close_gump = Some(crate::special_gumps::close_aquarium);
    h.raffle_management_gump = Some(crate::special_gumps::display_management);
    h.writ_of_lease_gump = Some(crate::special_gumps::display_writ);
    h.raffle_ticket_gump = Some(crate::special_gumps::display_ticket_gump);
    h.properties_gump = Some(crate::special_gumps::open_properties);
    h.break_crystal_gump = Some(|w, from, item| {
        send_gump_to_mobile(
            w,
            from,
            Box::new(muo_gumps::a_l::confirm_break_crystal::new(item)),
            false,
        );
    });
    h.reward_option_gump = Some(|w, from, deed| {
        use muo_items::special::vet;
        let Some(options) = vet::reward_options(w, deed) else {
            return;
        };
        crate::rewards::gumps::display_option(
            w,
            from,
            options.to_vec(),
            -1,
            std::rc::Rc::new(move |w, from, o| vet::reward_option_selected(w, deed, from, o)),
        );
    });
    h.reward_demolition_gump = Some(crate::rewards::gumps::display_demolition);
    h.ankh_resurrect_gump = Some(|w, m| {
        use muo_gumps::m_z::resurrect::{ResurrectMessage, new_with_prelude};
        let g = new_with_prelude(
            m,
            ResurrectMessage::VirtueShrine,
            muo_items::special::vet3::ankh_response_prelude,
        );
        send_gump_to_mobile(w, m, Box::new(g), false);
    });
    let a = muo_gumps::a_l::hooks::hooks_mut(w);
    a.imprisoned_summon = Some(muo_items::special::statuette::summon);
    a.imprisoned_release = Some(muo_items::special::statuette::release);
    a.new_guild_teleporter = Some(|w, stone| {
        Some(muo_items::guilds::guild_teleporter_new(
            w,
            &muo_items::guilds::GUILD_TELEPORTER,
            Some(stone),
        ))
    });
    a.set_vendor_house = Some(|w, v, house| {
        use muo_mobiles::npcs::vendors::{barkeeper, player_vendor};
        if player_vendor::is_player_vendor(w, v) {
            player_vendor::set_house(w, v, house);
        } else if barkeeper::is_barkeeper(w, v) {
            barkeeper::set_house(w, v, house);
        }
    });
    a.faction_is_leaving =
        Some(|w, m| crate::factions::player_state(w, m).is_some_and(|(_, ps)| ps.is_leaving()));
    a.faction_find = Some(|w, m| {
        crate::factions::find(w, m).map(|f| {
            (
                f as u32,
                crate::factions::tables::FACTION_DEFS[f]
                    .friendly_name
                    .to_owned(),
            )
        })
    });
}

/// Mobiles/{Special,Townfolk,Guards}: what the creatures call into the engines.
/// `AdvancedSearchThreadWorker.HandleValidInternal(entity)`: what a commodity deed, a key ring or a house legitimately
/// holds on the internal map.
fn valid_internal_held(w: &World, e: muo_server::ids::EntityId) -> Vec<muo_server::ids::EntityId> {
    use muo_server::ids::EntityId::Item;
    let Item(id) = e else { return Vec::new() };
    let Some(it) = w.item(id) else {
        return Vec::new();
    };
    if muo_content::items::is_named(it.ty, "Server.Items.CommodityDeed") {
        let on_internal = muo_items::deeds::deed_classes::commodity_of(w, id)
            .and_then(|c| w.item(c))
            .is_some_and(|c| c.map == Some(muo_server::map::MapId::INTERNAL));
        return if on_internal { vec![e] } else { Vec::new() };
    }
    if let Some(k) = it.ext.get::<muo_content::items::gold::KeyRingData>() {
        return k.keys.iter().map(|&k| Item(k)).collect();
    }
    if muo_content::items::is_named(it.ty, "Server.Multis.BaseHouse") {
        let h = muo_multis::house::d(w, id);
        let relocated = h
            .relocated_entities
            .iter()
            .filter_map(|r| matches!(r.entity, Item(_)).then_some(r.entity));
        let inventory = h
            .vendor_inventories
            .iter()
            .flat_map(|v| v.items.iter().map(|&i| Item(i)));
        return relocated.chain(inventory).collect();
    }
    Vec::new()
}

fn configure_npcs(w: &mut World) {
    {
        use muo_mobiles::npcs::vendors::player_vendor as pv;
        let b = w.resources.get_mut::<crate::bulk_orders::book::BobHooks>();
        // `RootParent as PlayerVendor`
        b.root_vendor = Some(
            |w, book| match muo_server::item::ops::root_parent(w, book) {
                Some(muo_server::ids::EntityId::Mobile(m)) if pv::is_player_vendor(w, m) => Some(m),
                _ => None,
            },
        );
        b.is_for_sale = Some(|w, v, book| pv::get_vendor_item(w, v, book).map(|i| i.is_for_sale()));
        b.add_hold_gold = Some(|w, v, price| {
            let g = pv::hold_gold(w, v);
            pv::set_hold_gold(w, v, g.wrapping_add(price));
        });
    }
    let h = w
        .resources
        .get_mut::<muo_content::player_mobile::hooks::SystemHooks>();
    h.manually_set_ping_pong = Some(crate::murder::manually_set_ping_pong);
    h.manually_set_short_term_murders = Some(crate::murder::manually_set_short_term_murders);
    let c = w.resources.get_mut::<muo_commands::hooks::CommandHooks>();
    c.valid_internal_held = Some(valid_internal_held);
    // `PlayerMurderSystem.BountiesEnabled`: `[Decorate` adds the bounty boards
    c.bounties_enabled = Some(|w| {
        w.resources
            .get::<crate::murder::MurderState>()
            .is_some_and(|s| s.bounties_enabled)
    });
    c.set_short_term_murders = Some(crate::murder::manually_set_short_term_murders);
    c.vendor_has_house =
        Some(|w, m| muo_mobiles::npcs::vendors::player_vendor::house_of(w, m).is_some());
    use muo_mobiles::npcs::special::hooks::SpecialHooks;
    use muo_mobiles::npcs::townfolk::EscortableHooks;

    let s = w.resources.get_mut::<SpecialHooks>();
    s.bounties_enabled = Some(|w| {
        w.resources
            .get::<crate::murder::MurderState>()
            .is_some_and(|s| s.bounties_enabled)
    });
    s.get_bounty = Some(crate::murder::get_bounty);
    s.clear_bounty = Some(crate::murder::clear_bounty);
    s.award_valor = Some(|w, pm, amount| {
        crate::virtues::award(w, pm, crate::virtues::VirtueName::Valor, amount)
    });
    s.justice_protector = Some(crate::virtues::justice_get_protector);
    s.justice_check_map_region = Some(crate::virtues::justice_check_map_region);
    s.justice_level =
        Some(|w, m| crate::virtues::get_level(w, m, crate::virtues::VirtueName::Justice) as i32);
    s.skull_new = Some(|w, ty| {
        crate::canned_evil::items::skull_new(
            w,
            crate::canned_evil::info::ChampionSkullType::from_i32(ty),
        )
    });
    s.award_harrower_title = Some(crate::canned_evil::titles::award_harrower_title);

    muo_content::skills::animal::hooks_mut_taming(w).faction_war_horse_allows =
        Some(|w, from, creature| {
            let horse = w
                .mobile(creature)?
                .ext
                .get::<crate::factions::mobiles::WarHorseData>()?
                .faction;
            let faction = crate::factions::find(w, from);
            Some(faction.is_some() && faction == horse)
        });
    muo_content::items::common::hooks_mut(w).corpse_new_head = Some(|w, dead| {
        let name = (w.mobile(dead)?.ty.name)(w, dead);
        let head = muo_items::body_parts::parts::new_head(w, 0, name);
        let bounty = w
            .resources
            .get::<crate::murder::MurderState>()
            .is_some_and(|s| s.bounties_enabled)
            && muo_content::player_mobile::data(w, dead).is_some();
        if bounty {
            muo_items::body_parts::parts::set_bounty_target(w, head, Some(dead));
        }
        Some(head)
    });

    let c = w.resources.get_mut::<crate::canned_evil::CannedEvilHooks>();
    c.champion_artifact = Some(muo_mobiles::npcs::special::champions::get_artifact);
    c.create_random_power_scroll = Some(|w, min, max| {
        Some(muo_items::special::scrolls::power_scroll_create_random_no_craft(w, min, max))
    });
    c.harrower_can_spawn = Some(muo_mobiles::npcs::special::harrower::can_spawn);
    c.harrower_spawn = Some(muo_mobiles::npcs::special::harrower::spawn);

    // `Type.CreateInstance<IEntity>(arg)` of the vendors' one-integer constructors, and the beverage test of the
    // bartenders' sell list.
    let v = w.resources.get_mut::<muo_content::vendor::VendorHooks>();
    v.construct_with_arg = Some(|w, name, arg| {
        muo_content::factory::make_item_args(
            w,
            name,
            &[muo_content::props::Value::Int(i64::from(arg))],
        )
    });
    // `Town.FromRegion(Region)?.Tax` of `GetPriceScalar`.
    v.town_tax = Some(|w, id| {
        let t = crate::factions::town::from_region(
            w,
            muo_server::mobile::part1::props_b::region(w, id),
        )?;
        crate::factions::town::tax(w, t)
    });
    v.beverage_is_empty_or_milk = Some(|w, item| {
        use muo_items::food::beverage;
        muo_content::factory::item_is(w, item, "Server.Items.BaseBeverage").then(|| {
            beverage::is_empty(w, item)
                || beverage::content(w, item) == beverage::BeverageType::Milk
        })
    });
    w.resources
        .get_mut::<muo_mobiles::npcs::vendors::player_vendor::PlayerVendorHooks>()
        .feature_enabled = Some(|w| {
        w.resources
            .get::<crate::feature_flags::ContentFeatureFlags>()
            .is_none_or(|c| c.player_vendors)
    });

    // BaseEscortable and the ML quest system.
    let e = w.resources.get_mut::<EscortableHooks>();
    e.tell = Some(|w, id, pm, number| {
        crate::ml_quests::quester::tell(
            w,
            crate::ml_quests::quester::Quester::Mobile(id),
            pm,
            number,
        );
    });
    e.quest_usable = Some(|w, short, region| {
        use crate::ml_quests::{engine, model::ObjectiveKind};
        let q = engine::quest(w, engine::find_quest(w, short)?);
        // We're already there!
        let there = q.objectives.iter().any(|o| match &o.kind {
            ObjectiveKind::Escort {
                destination: Some(d),
            } => muo_server::region::is_part_of_name(w, region, d.region, false),
            _ => false,
        });
        Some(!there)
    });
    e.award_compassion = Some(|w, pm, prisoner| {
        use crate::virtues::{VirtueName, award};
        use muo_content::player_mobile::tables::virtues_mut;
        let now = w.now;
        let v = virtues_mut(w, pm);
        if v.compassion_gains > 0 && now > v.next_compassion_day {
            v.next_compassion_day = muo_primitives::DateTime::MIN;
            v.compassion_gains = 0;
        }
        let gains = v.compassion_gains;
        if gains >= 5 {
            // You must wait about a day before you can gain in compassion again.
            muo_content::mx::msg(w, pm, 1053004);
        } else if let Some(gained_path) = award(
            w,
            pm,
            VirtueName::Compassion,
            if prisoner { 400 } else { 200 },
        ) {
            // You have achieved a path in compassion! / You have gained in compassion.
            muo_content::mx::msg(w, pm, if gained_path { 1053005 } else { 1053002 });
            let v = virtues_mut(w, pm);
            // in one day CompassionGains gets reset to 0
            v.next_compassion_day = now + muo_primitives::TimeSpan::from_days(1);
            v.compassion_gains += 1;
            if v.compassion_gains >= 5 {
                muo_content::mx::msg(w, pm, 1053004);
            }
        } else {
            // You have achieved the highest path of compassion and can no longer gain any further.
            muo_content::mx::msg(w, pm, 1053003);
        }
    });
    let h = w
        .resources
        .get_mut::<crate::ml_quests::engine::EscortHooks>();
    h.is_being_deleted = Some(muo_mobiles::npcs::townfolk::is_being_deleted);
    h.is_prisoner = Some(muo_mobiles::npcs::townfolk::is_prisoner);
    h.begin_delete = Some(muo_mobiles::npcs::townfolk::begin_delete);
    w.resources
        .get_mut::<crate::ml_quests::quester::QuesterHooks>()
        .dynamic_quests = Some(|w, m| {
        // `BaseEscortable` builds its own list (`StaticMLQuester` is false); other creatures use the registered one.
        if !muo_mobiles::npcs::townfolk::is_escortable(w, m)
            || muo_mobiles::npcs::townfolk::is_static_ml_quester(w, m)
        {
            return None;
        }
        let full = muo_mobiles::npcs::townfolk::ml_quest_full_name(w, m);
        Some(
            full.and_then(|f| crate::ml_quests::engine::find_quest_by_full_name(w, &f))
                .into_iter()
                .collect(),
        )
    });
}
