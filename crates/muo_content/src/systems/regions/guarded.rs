//! Guarded regions: a region watched by guards that come when a criminal is called out, and its
//! `CheckGuarded`/`SetGuarded`/`ToggleGuarded` commands.

use std::rc::Rc;

use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::commands::{CommandEventArgs, CommandSystem};
use muo_server::events::SpeechEventArgs;
use muo_server::ids::MobileId;
use muo_server::interfaces::SpellId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages as mm;
use muo_server::region::{self, RegionId, RegionJson, RegionType};
use muo_server::timer;

use super::base_region::{
    BASE_REGION, BASE_REGION_FAMILY, BaseRegionType, RegionHooks, base_from_json, data, data_mut,
    family, json_bool, json_string,
};
use crate::creature::{self, ctype};
use crate::misc::notoriety;
use crate::spells::spell;
use muo_server::notoriety::{CRIMINAL, MURDERER};

fn send(w: &mut World, to: MobileId, number: i32) {
    mm::send_localized_message(w, to, number, "", 0x3B2);
}

fn mobile_is(w: &World, m: MobileId, full_name: &str) -> bool {
    let mut t = w.mobile(m).map(|m| m.ty);
    while let Some(ty) = t {
        if ty.full_name == full_name {
            return true;
        }
        t = ty.parent;
    }
    false
}

/// `AllowReds`.
pub fn allow_reds(w: &World, id: RegionId) -> bool {
    family(w, id).map_or_else(|| crate::core::aos(w), |f| (f.allow_reds)(w, id))
}

/// `IsDisabled()`.
pub fn is_disabled(w: &World, id: RegionId) -> bool {
    family(w, id).map_or_else(
        || data(w, id).is_some_and(|d| d.guards_disabled),
        |f| (f.is_disabled)(w, id),
    )
}

/// `GuardType`: the configured guard class, else `DefaultGuardType`.
pub fn guard_type(w: &World, id: RegionId) -> String {
    if let Some(t) = data(w, id).and_then(|d| d.guard_type.clone()) {
        return t;
    }
    family(w, id)
        .map_or("Server.Mobiles.WarriorGuard", |f| {
            (f.default_guard_type)(w, id)
        })
        .to_string()
}

fn from_json(w: &mut World, id: RegionId, o: &RegionJson) -> Result<(), String> {
    base_from_json(w, id, o)?;
    let guard = json_string(o, "GuardType")?;
    let disabled = json_bool(o, "GuardsDisabled")?;
    let entrance = super::base_region::json_point(o, "Entrance")?;
    let d = data_mut(w, id).ok_or("region is gone")?;
    if guard.is_some() {
        d.guard_type = guard;
    }
    if let Some(v) = disabled {
        d.guards_disabled = v;
    }
    if let Some(p) = entrance {
        d.entrance = p;
    }
    Ok(())
}

/// `CheckVendorAccess(vendor, from)`.
pub fn check_vendor_access(w: &World, id: RegionId, from: MobileId) -> bool {
    w.mobile(from)
        .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
        || is_disabled(w, id)
        || !muo_server::mobile::base::murderer(w, from)
}

fn on_begin_spell_cast(w: &mut World, id: RegionId, m: MobileId, s: SpellId) -> bool {
    if !is_disabled(w, id)
        && !spell::spell_type(w, s).is_some_and(|t| (t.on_cast_in_town)(w, s, id))
    {
        send(w, m, 500946); // You cannot cast this in town!
        return false;
    }
    (BASE_REGION.on_begin_spell_cast)(w, id, m, s)
}

fn make_guard(w: &mut World, id: RegionId, focus: MobileId) {
    let t = guard_type(w, id);
    if let Some(f) = w.resources.get::<RegionHooks>().and_then(|h| h.make_guard) {
        f(w, &t, focus);
    }
}

fn on_enter(w: &mut World, id: RegionId, m: MobileId) {
    if is_disabled(w, id) {
        return;
    }
    if !allow_reds(w, id) && muo_server::mobile::base::murderer(w, m) {
        check_guard_candidate(w, id, m);
    }
}

fn on_exit(_: &mut World, _: RegionId, _: MobileId) {}

fn on_speech(w: &mut World, id: RegionId, args: &mut SpeechEventArgs) {
    (BASE_REGION.on_speech)(w, id, args);

    if is_disabled(w, id) {
        return;
    }

    let from = args.mobile;
    if w.mobile(from).is_some_and(|m| (m.ty.alive)(w, from)) && args.has_keyword(0x0007) {
        // *guards*
        let loc = w.mobile(from).map(|m| m.location).unwrap_or_default();
        call_guards(w, id, loc);
    }
}

fn on_aggressed(
    w: &mut World,
    id: RegionId,
    aggressor: MobileId,
    aggressed: MobileId,
    criminal: bool,
) {
    (BASE_REGION.on_aggressed)(w, id, aggressor, aggressed, criminal);

    if !is_disabled(w, id) && aggressor != aggressed && criminal {
        check_guard_candidate(w, id, aggressor);
    }
}

fn on_got_beneficial_action(w: &mut World, id: RegionId, helper: MobileId, helped: MobileId) {
    (BASE_REGION.on_got_beneficial_action)(w, id, helper, helped);

    if is_disabled(w, id) {
        return;
    }

    let noto = notoriety::mobile_notoriety(w, helper, helped);

    if helper != helped && (noto == CRIMINAL || noto == MURDERER) {
        check_guard_candidate(w, id, helper);
    }
}

fn on_criminal_action(w: &mut World, id: RegionId, m: MobileId, message: bool) {
    (BASE_REGION.on_criminal_action)(w, id, m, message);

    if !is_disabled(w, id) {
        check_guard_candidate(w, id, m);
    }
}

/// `CheckGuardCandidate(m)`.
pub fn check_guard_candidate(w: &mut World, id: RegionId, m: MobileId) {
    if !is_guard_candidate(w, id, m) {
        return;
    }

    let existing = data(w, id).and_then(|d| d.guard_candidates.get(&m).copied());
    match existing {
        None => {
            // GuardTimer: 15 seconds, then the candidate is let off.
            let h = timer::start(w, TimeSpan::from_secs(15), TimeSpan::ZERO, 1, move |w| {
                let removed = data_mut(w, id).and_then(|d| d.guard_candidates.shift_remove(&m));
                if removed.is_some() {
                    send(w, m, 502276); // Guards can no longer be called on you.
                }
            });
            if let Some(d) = data_mut(w, id) {
                d.guard_candidates.insert(m, h);
            }
            send(w, m, 502275); // Guards can now be called on you!

            let Some((map, loc)) = w.mobile(m).map(|mob| (mob.map, mob.location)) else {
                return;
            };
            if map.is_none() {
                return;
            }

            let mut fake_call: Option<MobileId> = None;
            let mut prio = 0.0;

            for v in muo_server::mobile::part2::enumerators::get_mobiles_in_range(w, m, 8) {
                let is_player = w.mobile(v).is_some_and(|x| x.player);
                let human_in_town = if creature::is_creature(w, v) {
                    creature::misc::is_human_in_town(w, v)
                } else {
                    let human = w.mobile(v).is_some_and(|_| {
                        w.resources
                            .get::<muo_primitives::BodyTable>()
                            .is_some_and(|t| {
                                t.is_human(muo_server::mobile::part1::props_b::body(w, v))
                            })
                    });
                    human
                        && w.mobile(v)
                            .is_some_and(|x| region::is_part_of(w, x.region, id))
                };
                if is_player || v == m || is_guard_candidate(w, id, v) || !human_in_town {
                    continue;
                }

                let vloc = w.mobile(v).map(|x| x.location).unwrap_or_default();
                let dist = utility::distance_to_sqrt_3d(loc, vloc);

                if fake_call.is_none() || dist < prio {
                    fake_call = Some(v);
                    prio = dist;
                }
            }

            if let Some(call) = fake_call {
                let line = utility::random_from(&mut w.rng, 1013037, 16);
                mm::say_number(w, call, line, "");

                if let Some(f) = w
                    .resources
                    .get::<RegionHooks>()
                    .and_then(|h| h.guard_spawn_from_mobile)
                {
                    f(w, call, m);
                }
                if let Some(mut t) =
                    data_mut(w, id).and_then(|d| d.guard_candidates.shift_remove(&m))
                {
                    t.cancel(w);
                }

                if !is_always_guard_candidate(w, id, m) {
                    send(w, m, 502276); // Guards can no longer be called on you.
                }
            }
        }
        Some(mut t) => {
            // timer.Stop(); timer.Start();
            t.cancel(w);
            let h = timer::start(w, TimeSpan::from_secs(15), TimeSpan::ZERO, 1, move |w| {
                let removed = data_mut(w, id).and_then(|d| d.guard_candidates.shift_remove(&m));
                if removed.is_some() {
                    send(w, m, 502276);
                }
            });
            if let Some(d) = data_mut(w, id) {
                d.guard_candidates.insert(m, h);
            }
        }
    }
}

/// `CallGuards(p)`.
pub fn call_guards(w: &mut World, id: RegionId, p: Point3D) {
    let Some(map) = w.region(id).map(|r| r.map) else {
        return;
    };
    let mut queue = Vec::new();

    let nearby: Vec<MobileId> =
        muo_server::map::enumerate::mobiles_in_range(w, Some(map), p.x, p.y, 14).collect();
    for m in nearby {
        if !is_guard_candidate(w, id, m) {
            continue;
        }

        // Dictionary members can be targeted regardless of region (guards reach outside).
        // Non-dictionary permanent candidates (reds) must be in the region.
        let in_table = data(w, id).is_some_and(|d| d.guard_candidates.contains_key(&m));
        let in_region = w
            .mobile(m)
            .is_some_and(|x| region::is_part_of(w, x.region, id));
        if in_table || (in_region && is_always_guard_candidate(w, id, m)) {
            if let Some(mut t) = data_mut(w, id).and_then(|d| d.guard_candidates.shift_remove(&m)) {
                t.cancel(w);
            }
            queue.push(m);
        }
    }

    for m in queue {
        if let Some(f) = w
            .resources
            .get::<RegionHooks>()
            .and_then(|h| h.guard_spawn_from_region)
        {
            f(w, id, m);
        }

        if !is_always_guard_candidate(w, id, m) {
            send(w, m, 502276); // Guards can no longer be called on you.
        }
    }
}

fn is_always_guard_candidate(w: &World, id: RegionId, m: MobileId) -> bool {
    !allow_reds(w, id) && muo_server::mobile::base::murderer(w, m)
}

/// `IsGuardCandidate(m)`.
pub fn is_guard_candidate(w: &World, id: RegionId, m: MobileId) -> bool {
    let Some(mob) = w.mobile(m) else { return false };
    if mobile_is(w, m, "Server.Mobiles.BaseGuard")
        || !(mob.ty.alive)(w, m)
        || mob.access_level > AccessLevel::Player
        || mob.blessed
    {
        return false;
    }

    let invulnerable =
        creature::is_creature(w, m) && ctype(w, m).is_some_and(|c| (c.is_invulnerable)(w, m));
    !invulnerable && (is_always_guard_candidate(w, id, m) || mob.criminal)
}

/// `Disable(reg)`.
pub fn disable(w: &mut World, id: RegionId) -> RegionId {
    if let Some(d) = data_mut(w, id) {
        d.guards_disabled = true;
    }
    id
}

pub static GUARDED_FAMILY: BaseRegionType = BaseRegionType {
    ..BASE_REGION_FAMILY
};

/// `GuardedRegion`.
pub static GUARDED_REGION: RegionType = RegionType {
    name: "GuardedRegion",
    parent: Some(&BASE_REGION),
    from_json,
    on_begin_spell_cast,
    allow_housing: |_, _, _, _| false,
    make_guard,
    on_enter,
    on_exit,
    on_speech,
    on_aggressed,
    on_got_beneficial_action,
    on_criminal_action,
    family: Some(&GUARDED_FAMILY),
    ..BASE_REGION
};

// ---- commands ---------------------------------------------------------------------------------------------------

fn guarded_region_of(w: &World, from: MobileId) -> Option<RegionId> {
    let r = w.mobile(from)?.region;
    region::get_region_of_type(w, r, &GUARDED_REGION)
}

fn msg(w: &mut World, to: MobileId, text: &str) {
    mm::send_message(w, to, text);
}

/// `[Usage("CheckGuarded"), Description("Returns a value indicating if the current region is guarded or not.")]`
fn check_guarded_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    match guarded_region_of(w, from) {
        None => msg(w, from, "You are not in a guardable region."),
        Some(reg) if data(w, reg).is_some_and(|d| d.guards_disabled) => {
            msg(w, from, "The guards in this region have been disabled.")
        }
        Some(_) => msg(w, from, "This region is actively guarded."),
    }
}

/// `[Usage("SetGuarded <true|false>"), Description("Enables or disables guards for the current region.")]`
fn set_guarded_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    if e.length() == 1 {
        match guarded_region_of(w, from) {
            None => msg(w, from, "You are not in a guardable region."),
            Some(reg) => {
                let disabled = !e.get_boolean(0);
                if let Some(d) = data_mut(w, reg) {
                    d.guards_disabled = disabled;
                }
                msg(
                    w,
                    from,
                    if disabled {
                        "The guards in this region have been disabled."
                    } else {
                        "The guards in this region have been enabled."
                    },
                );
            }
        }
    } else {
        msg(w, from, "Format: SetGuarded <true|false>");
    }
}

/// `[Usage("ToggleGuarded"), Description("Toggles the state of guards for the current region.")]`
fn toggle_guarded_on_command(w: &mut World, e: &mut CommandEventArgs) {
    let from = e.mobile;
    match guarded_region_of(w, from) {
        None => msg(w, from, "You are not in a guardable region."),
        Some(reg) => {
            let disabled = !data(w, reg).is_some_and(|d| d.guards_disabled);
            if let Some(d) = data_mut(w, reg) {
                d.guards_disabled = disabled;
            }
            msg(
                w,
                from,
                if disabled {
                    "The guards in this region have been disabled."
                } else {
                    "The guards in this region have been enabled."
                },
            );
        }
    }
}

/// `GuardedRegion.Configure()`.
pub fn configure(w: &mut World) {
    CommandSystem::register(
        w,
        "CheckGuarded",
        AccessLevel::GameMaster,
        &[],
        Rc::new(check_guarded_on_command),
    );
    CommandSystem::register(
        w,
        "SetGuarded",
        AccessLevel::Administrator,
        &[],
        Rc::new(set_guarded_on_command),
    );
    CommandSystem::register(
        w,
        "ToggleGuarded",
        AccessLevel::Administrator,
        &[],
        Rc::new(toggle_guarded_on_command),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::map::MapId;
    use muo_server::region::{new_region, register};

    fn world_with_town() -> (World, RegionId) {
        let mut w = muo_server::map::test_support::world();
        let rect = muo_primitives::Rectangle3D::from_points(
            Point3D::new(0, 0, -128),
            Point3D::new(100, 100, 128),
        );
        let id = new_region(
            &mut w,
            &GUARDED_REGION,
            Some("Town"),
            MapId::FELUCCA,
            None,
            Some(50),
            vec![rect],
        );
        super::super::base_region::init_data(&mut w, id);
        register(&mut w, id);
        (w, id)
    }

    #[test]
    fn disabled_guards_and_default_guard_types() {
        let (mut w, id) = world_with_town();
        assert!(!is_disabled(&w, id));
        assert_eq!(guard_type(&w, id), "Server.Mobiles.WarriorGuard");
        disable(&mut w, id);
        assert!(is_disabled(&w, id));
        data_mut(&mut w, id).unwrap().guard_type = Some("Server.Mobiles.ArcherGuard".into());
        assert_eq!(guard_type(&w, id), "Server.Mobiles.ArcherGuard");
    }

    #[test]
    fn json_properties() {
        let (mut w, id) = world_with_town();
        let o: RegionJson = serde_json::from_str(
            r#"{"GuardType":"Server.Mobiles.ArcherGuard","GuardsDisabled":true,"Entrance":{"x":1,"y":2,"z":3},"RuneName":"Trinsic"}"#,
        )
        .unwrap();
        from_json(&mut w, id, &o).unwrap();
        let d = data(&w, id).unwrap();
        assert_eq!(d.guard_type.as_deref(), Some("Server.Mobiles.ArcherGuard"));
        assert!(d.guards_disabled);
        assert_eq!(d.entrance, Point3D::new(1, 2, 3));
        assert_eq!(d.rune_name.as_deref(), Some("Trinsic"));
        let bad: RegionJson = serde_json::from_str(r#"{"GuardsDisabled":"yes"}"#).unwrap();
        assert!(from_json(&mut w, id, &bad).is_err());
    }
}
