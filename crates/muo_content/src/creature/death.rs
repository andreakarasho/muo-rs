//! `BaseCreature` death: looting rights, `OnBeforeDeath`, `OnDeath`, `OnDelete`.

use muo_primitives::DateTime;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::base;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::world::World;

use super::control::{
    control_master, get_master, issue_order, set_control_master, set_master, summon_master,
};
use super::data::{cd, cd_mut, read};
use super::enums::OrderType;
use super::events::{creature_death_event, creature_deleted_event};
use super::vtable::ctype;
use crate::abilities::{self, MonsterAbilityTrigger};
use crate::hooks::{hooks, rnd, world_hooks};
use crate::mx;

/// `DamageStore`.
#[derive(Clone, Debug)]
pub struct DamageStore {
    pub mobile: MobileId,
    pub damage: i32,
    pub has_right: bool,
}

/// `ComputeBonusDamage`: damage dealt by `m`'s own pets.
pub fn compute_bonus_damage(w: &mut World, dead: MobileId, m: MobileId) -> i32 {
    let entries = props_a::damage_entries(w, dead).to_vec();
    let mut bonus = 0;
    for de in entries.iter().rev() {
        if de.damager == m || cd(w, de.damager).is_none() {
            continue;
        }
        if get_master(w, de.damager) == Some(m) {
            bonus += de.damage_given;
        }
    }
    bonus
}

/// `GetLootingRights(damageEntries, hitsMax)`: most recent entries first.
pub fn get_looting_rights(w: &mut World, dead: MobileId, hits_max: i32) -> Vec<DamageStore> {
    let entries = props_a::damage_entries(w, dead).to_vec();
    let mut rights: Vec<DamageStore> = Vec::new();
    let mut first_damager: Option<usize> = None;
    let present_player =
        |w: &World, m: MobileId| w.mobile(m).is_some_and(|x| !x.deleted && x.player);

    for de in entries.iter().rev() {
        let mut damage = de.damage_given;
        for sub in &de.responsible {
            let master = sub.damager;
            if !present_player(w, master) {
                continue;
            }
            match rights.iter().position(|r| r.mobile == master) {
                Some(k) => {
                    rights[k].damage += sub.damage_given;
                    first_damager = Some(k);
                }
                None => {
                    rights.push(DamageStore {
                        mobile: master,
                        damage: sub.damage_given,
                        has_right: false,
                    });
                    first_damager = Some(rights.len() - 1);
                }
            }
            damage -= sub.damage_given;
        }
        let m = de.damager;
        if !present_player(w, m) || damage <= 0 {
            continue;
        }
        match rights.iter().position(|r| r.mobile == m) {
            Some(j) => {
                rights[j].damage += damage;
                first_damager = Some(j);
            }
            None => {
                rights.push(DamageStore {
                    mobile: m,
                    damage,
                    has_right: false,
                });
                first_damager = Some(rights.len() - 1);
            }
        }
    }

    // Handle damage rights per Five on Friday: https://www.uoguide.com/Five_on_Friday_-_January_19,_2007
    if !rights.is_empty() {
        let top_index = first_damager;
        if let Some(i) = top_index {
            rights[i].damage = (rights[i].damage as f64 * 1.25) as i32;
        }
        if rights.len() > 1 {
            // `CompareTo` orders by damage descending.
            muo_primitives::dotnet::net_sort(&mut rights, |a, b| b.damage.cmp(&a.damage));
        }
        let top_damage = rights[0].damage;
        let min_damage = if hits_max >= 3000 {
            top_damage / 16
        } else if hits_max >= 1000 {
            top_damage / 8
        } else if hits_max >= 200 {
            top_damage / 4
        } else {
            top_damage / 2
        };
        for ds in &mut rights {
            ds.has_right = ds.damage >= min_damage;
        }
    }
    rights
}

/// `OnBeforeDeath`
pub fn on_before_death(w: &mut World, id: MobileId) -> bool {
    abilities::trigger_ability(w, id, MonsterAbilityTrigger::DEATH, None);
    let c = ctype(w, id).expect("creature");
    let mut treasure_level = (c.treasure_map_level)(w, id);
    let map = mx::map(w, id);
    if treasure_level == 1 && map == Some(muo_server::map::MapId::TRAMMEL) {
        let mut killer = w.mobile(id).and_then(|m| m.last_killer);
        if let Some(k) = killer
            && cd(w, k).is_some()
        {
            killer = get_master(w, k);
        }
        let young = killer
            .and_then(|k| {
                crate::hooks::pet_player_hooks(w)
                    .and_then(|h| h.young)
                    .map(|f| f(w, k))
            })
            .unwrap_or(false);
        // TreasureMap.IsInHavenIsland(this)
        let l = mx::location(w, id);
        if young && (3314..=3814).contains(&l.x) && (2345..=3095).contains(&l.y) {
            treasure_level = 0;
        }
    }
    let (summoned, no_awards, bonded, paragon, has_loot) =
        read(w, id, (false, false, false, false, false), |d| {
            (
                d.summoned,
                d.no_kill_awards,
                d.is_bonded,
                d.is_paragon,
                d.has_generated_loot,
            )
        });
    if !summoned && !no_awards && !bonded {
        if treasure_level >= 0 {
            let chest_chance = hooks(w).and_then(|h| h.paragon_chest_chance).unwrap_or(0.0);
            if paragon && chest_chance > rnd::double(w) {
                // PackItem(new ParagonChest(Name, treasureLevel))
                if let Some(f) = crate::hooks::death_hooks(w).and_then(|h| h.pack_paragon_chest) {
                    f(w, id, treasure_level);
                }
            } else if (map == Some(muo_server::map::MapId::FELUCCA)
                || map == Some(muo_server::map::MapId::TRAMMEL))
                && rnd::double(w) < (c.treasure_map_chance)(w, id)
                && let Some(f) = crate::hooks::death_hooks(w).and_then(|h| h.pack_treasure_map)
            {
                f(w, id, treasure_level);
            }
        }
        let choc = hooks(w)
            .and_then(|h| h.paragon_chocolate_chance)
            .unwrap_or(0.0);
        if paragon && choc > rnd::double(w) {
            let name =
                ["CocoaButter", "CocoaLiquor", "SackOfSugar", "Vanilla"][rnd::below(w, 4) as usize];
            let item = crate::factory::make_item(w, &format!("Server.Items.{name}"));
            super::setters::pack_item(w, id, item);
        }
    }
    if !summoned && !no_awards && !has_loot {
        if let Some(d) = cd_mut(w, id) {
            d.has_generated_loot = true;
        }
        super::setters::generate_loot_spawning(w, id, false);
    }
    if !no_awards
        && muo_server::region::is_part_of_name(
            w,
            muo_server::mobile::part1::props_b::region(w, id),
            "Doom",
            false,
        )
    {
        // TheSummoningQuest.GetDaemonBonesFor(this)
        let bones = crate::hooks::death_hooks(w)
            .and_then(|h| h.daemon_bones_for)
            .map_or(0, |f| f(w, id));
        if bones > 0 {
            let item = crate::factory::make_item(w, "Server.Items.DaemonBone");
            if let Some(i) = item {
                if let Some(it) = w.item_mut(i) {
                    it.amount = bones;
                }
                super::setters::pack_item(w, id, Some(i));
            }
        }
    }
    if (c.is_animated_dead)(w, id) {
        let (loc, map) = (mx::location(w, id), mx::map(w, id));
        muo_server::effects::send_location_effect(w, loc, map, 0x3728, 13, 1, 0x461, 4);
    }
    if let Some(sp) = ctype(w, id).and_then(|c| (c.speech_type)(w, id)) {
        sp.on_death(w, id);
    }
    crate::player_mobile::hooks::honor_on_killed(w, id); // `ReceivedHonorContext?.OnTargetKilled()`
    base::on_before_death_base(w, id)
}

/// `OnDeath`
pub fn on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    if let Some(sp) = w.mobile(id).and_then(|m| m.spawner) {
        let killer = w.mobile(id).and_then(|m| m.last_killer);
        muo_server::mobile::part2::deps::spawner_notify_death(w, sp, id, killer);
    }
    let ty = ctype(w, id).expect("creature");
    if read(w, id, false, |d| d.is_bonded) {
        let snd = match w.mobile(id).map(|m| m.ty) {
            Some(t) => (t.get_death_sound)(w, id),
            None => 0,
        };
        muo_server::effects::play_sound_entity(w, mx::entity(id), snd);
        props_b::set_warmode(w, id, false);
        props_b::set_poison(w, id, None);
        mx::set_combatant(w, id, None);
        props_b::set_hits(w, id, 0);
        props_b::set_stam(w, id, 0);
        props_b::set_mana(w, id, 0);
        if let Some(d) = cd_mut(w, id) {
            d.is_dead_pet = true;
        }
        issue_order(w, id, OrderType::Follow, None, control_master(w, id));
        if let Some(m) = w.mobile(id).map(|m| m.ty) {
            (m.process_delta)(w, id);
        }
        muo_server::mobile::part2::effects::send_incoming_packet(w, id);
        let attackers: Vec<MobileId> = w
            .mobile(id)
            .map(|m| m.aggressors.iter().map(|a| a.attacker).collect())
            .unwrap_or_default();
        for a in attackers {
            if mx::combatant(w, a) == Some(id) {
                mx::set_combatant(w, a, None);
            }
        }
        let defenders: Vec<MobileId> = w
            .mobile(id)
            .map(|m| m.aggressed.iter().map(|a| a.defender).collect())
            .unwrap_or_default();
        for d in defenders {
            if mx::combatant(w, d) == Some(id) {
                mx::set_combatant(w, d, None);
            }
        }
        let owner = control_master(w, id);
        let near = owner.is_some_and(|o| {
            !mx::deleted(w, o)
                && mx::map(w, o) == mx::map(w, id)
                && mx::in_range(w, o, id, 12)
                && w.mobile(id)
                    .map(|m| m.ty)
                    .is_some_and(|m| (m.can_see_mobile)(w, id, o))
        }) && owner.is_some_and(|o| mx::in_los(w, id, o));
        let now = w.now;
        if let Some(d) = cd_mut(w, id) {
            if !near {
                if d.owner_abandon_time == DateTime::MIN {
                    d.owner_abandon_time = now;
                }
            } else {
                d.owner_abandon_time = DateTime::MIN;
            }
        }
        creature_death_event(w, id);
        if let Some(m) = w.mobile(id).map(|m| m.ty) {
            (m.check_stat_timers)(w, id);
        }
        return;
    }

    let (summoned, no_awards) = read(w, id, (false, false), |d| (d.summoned, d.no_kill_awards));
    if !summoned && !no_awards {
        let (total_fame, total_karma) = hooks(w)
            .and(world_hooks(w))
            .and_then(|h| h.compute_kill_awards)
            .map_or((0, 0), |f| f(w, id));
        let hits_max = mx::hits_max(w, id);
        let list = get_looting_rights(w, id, hits_max);
        let mut titles: Vec<(MobileId, i32, i32)> = Vec::new();
        let mut given_quest_kill = false;
        let mut given_faction_kill = false;
        let mut given_tot_kill = false;
        let uor = w.expansion() >= muo_data::expansion::Expansion::UOR;
        for ds in list.iter().filter(|d| d.has_right) {
            if !uor {
                let lk = w.mobile(id).and_then(|m| m.last_killer);
                let killer = match lk {
                    Some(k) if cd(w, k).is_some() => {
                        w.mobile(k).and_then(|m| (m.ty.get_damage_master)(w, k, id))
                    }
                    other => other,
                };
                if Some(ds.mobile) == killer {
                    titles.push((ds.mobile, total_fame, total_karma));
                }
            } else if let Some(members) = world_hooks(w)
                .and_then(|h| h.party_members)
                .map(|f| f(w, ds.mobile))
                .filter(|m| !m.is_empty())
            {
                let n = members.len() as i32;
                let (df, dk) = (total_fame / n, total_karma / n);
                for m in members {
                    match titles.iter_mut().find(|t| t.0 == m) {
                        Some(t) => {
                            t.1 += df;
                            t.2 += dk;
                        }
                        None => titles.push((m, df, dk)),
                    }
                }
            } else {
                titles.push((ds.mobile, total_fame, total_karma));
            }
            (ty.on_killed_by)(w, id, ds.mobile);
            if !given_faction_kill {
                given_faction_kill = true;
                crate::player_mobile::hooks::faction_handle_death(w, id, Some(ds.mobile));
            }
            // `!givenToTKill && (Map == Map.Tokuno || region.IsPartOf("Yomotsu Mines") || ..)`: one roll per death
            if !given_tot_kill {
                let kr = props_b::region(w, ds.mobile);
                if w.mobile(id)
                    .is_some_and(|m| m.map == Some(muo_server::map::MapId::TOKUNO))
                    || muo_server::region::is_part_of_name(w, kr, "Yomotsu Mines", false)
                    || muo_server::region::is_part_of_name(w, kr, "Fan Dancer's Dojo", false)
                {
                    given_tot_kill = true;
                    if let Some(f) =
                        crate::hooks::death_hooks(w).and_then(|h| h.on_death_rights_holder)
                    {
                        f(w, id, ds.mobile, c);
                    }
                }
            }
            // MLQuestSystem.HandleKill(pm, this) for a PlayerMobile.
            if crate::player_mobile::data(w, ds.mobile).is_some()
                && let Some(f) =
                    crate::ml_quest_hooks::ml_quest_creature_hooks(w).and_then(|h| h.handle_kill)
            {
                f(w, ds.mobile, id);
            }
            // `if (ds.m_Mobile is PlayerMobile pm && !givenQuestKill && pm.Quest != null) { qs.OnKill(this, c); }`
            if !given_quest_kill
                && let Some(f) = crate::hooks::death_hooks(w).and_then(|h| h.quest_on_kill)
            {
                given_quest_kill = f(w, ds.mobile, id, c);
            }
        }
        if let Some(f) = world_hooks(w).and_then(|h| h.award_fame_karma) {
            for (m, fame, karma) in titles {
                f(w, m, fame, karma);
            }
        }
    }

    base::on_death_base(w, id, c);
    if (ty.delete_corpse_on_death)(w, id)
        && let Some(c) = c
    {
        muo_server::item::base::delete(w, c);
    }
    creature_death_event(w, id);
}

/// `OnDelete`
pub fn on_delete(w: &mut World, id: MobileId) {
    creature_deleted_event(w, id);
    // Both registries are keyed by the master, which is cleared below.
    let sm = summon_master(w, id);
    let animated = ctype(w, id).is_some_and(|c| (c.is_animated_dead)(w, id));
    if let Some(f) = crate::hooks::hooks(w).and_then(|h| h.creature_unregister_summon) {
        f(w, id, sm, animated);
    }
    let m = control_master(w, id);
    set_control_master(w, id, None);
    set_master(w, id, None);
    crate::player_mobile::hooks::honor_cancel(w, id); // `ReceivedHonorContext?.Cancel()`
    base::on_delete_base(w, id);
    if let Some(m) = m {
        muo_server::mobile::part2::properties::invalidate_properties(w, m);
    }
    let _ = summon_master;
}
