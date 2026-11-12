//! `Bandage` (with its target and `BandageTargetRequest`) and
//! `BandageContext`, the healing timer a healer has running on a patient (a world resource, per healer).
//!
//! `EnhancedBandage` (special/anniv2.rs) derives from [`BANDAGE`]. The hooks are `GameHooks::bandage_target_request`,
//! `SpellHooks::consume_bandage` (muo_spells finds a `Server.Items.Bandage` in the pack), the player's
//! `BandageContext.GetContext(m)?.Slip()` and [`BandageHooks::is_free_consume`] (`DuelContext.IsFreeConsume`).

use std::rc::Rc;

use indexmap::IndexMap;
use muo_content::creature;
use muo_content::items::weapon_ability as wa;
use muo_content::props::{Ctor, Kind, Param, Value, add_ctors};
use muo_content::utilities::types::{Numeric, PropType};
use muo_io::{GenericReader, GenericWriter, Result};
use muo_primitives::{Point3D, TimeSpan, utility};
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, ops};
use muo_server::skills::SkillName;
use muo_server::target::{self, Target, TargetFlags, Targeted};
use muo_server::timer::{self, TimerHandle};
use muo_server::type_registry::{Phase, TypeRegistry};

use super::{read_version, write_version};

/// What the bandage asks of other crates.
#[derive(Default)]
pub struct BandageHooks {
    /// `DuelContext.IsFreeConsume(m)`. Unset: false.
    pub is_free_consume: Option<fn(&World, MobileId) -> bool>,
}

/// `Bandage.Range`.
pub fn range(w: &World) -> i32 {
    if muo_content::core::aos(w) { 2 } else { 1 }
}

/// `new Bandage(amount)`.
pub fn new_bandage(w: &mut World, amount: i32) -> ItemId {
    let id = ops::new_item_with_id(w, &BANDAGE, 0xE21);
    ops::set_stackable(w, id, true);
    ops::set_amount(w, id, amount);
    id
}

fn in_reach(w: &mut World, bandage: ItemId, from: MobileId) -> bool {
    let loc = ops::get_world_location(w, bandage);
    let r = range(w);
    deps::mobile_in_range(w, from, loc, r)
}

fn is_innard(w: &World, item: ItemId) -> bool {
    muo_content::factory::item_is(w, item, "Server.Items.PlagueBeastInnard")
}

/// `innard.OnBandage(from)`: only the plague beast's blood answers.
fn innard_on_bandage(w: &mut World, innard: ItemId, from: MobileId) -> bool {
    crate::special::mutation::on_bandage(w, innard, from)
}

/// `InternalTarget.OnTarget`.
fn on_target(w: &mut World, bandage: ItemId, from: MobileId, targeted: Targeted) {
    if w.item(bandage).is_none_or(|i| i.deleted()) {
        return;
    }
    match targeted {
        Targeted::Entity(EntityId::Mobile(mobile)) => {
            if in_reach(w, bandage, from) {
                let started = begin_heal(w, from, mobile);
                let free = w
                    .resources
                    .get::<BandageHooks>()
                    .and_then(|h| h.is_free_consume)
                    .is_some_and(|f| f(w, from));
                if started && !free {
                    base::consume(w, bandage, 1);
                }
            } else {
                deps::send_localized_message(w, from, 500295); // You are too far away to do that.
            }
        }
        Targeted::Entity(EntityId::Item(innard)) if is_innard(w, innard) => {
            if innard_on_bandage(w, innard, from) {
                base::consume(w, bandage, 1);
            }
        }
        _ => deps::send_localized_message(w, from, 500970), // Bandages can not be used on that.
    }
}

/// `new InternalTarget(bandage)`.
fn internal_target(w: &mut World, bandage: ItemId) -> Target {
    let r = range(w);
    let mut t = Target::new(w, r, false, TargetFlags::BENEFICIAL);
    t.handlers.on_target = Some(Rc::new(move |w, from, targeted| {
        on_target(w, bandage, from, targeted)
    }));
    // OnNonlocalTarget: an innard on a creature is bandaged; anything else is the base message.
    t.handlers.on_nonlocal_target = Some(Rc::new(move |w, from, targeted| match targeted {
        Targeted::Entity(EntityId::Item(innard)) if is_innard(w, innard) => {
            if innard_on_bandage(w, innard, from) {
                base::consume(w, bandage, 1);
            }
        }
        _ => deps::send_localized_message(w, from, 500447), // That is not accessible.
    }));
    t
}

fn double_click(w: &mut World, id: ItemId, from: MobileId) {
    if in_reach(w, id, from) {
        muo_server::mobile::base::revealing_action(w, from);
        deps::send_localized_message(w, from, 500948); // Who will you use the bandages on?
        let t = internal_target(w, id);
        let tid = target::add_target(w, t);
        target::set_mobile_target(w, from, Some(tid));
    } else {
        deps::send_localized_message(w, from, 500295); // You are too far away to do that.
    }
}

/// `Bandage.BandageTargetRequest(from, item, target)` (the client's bandage macro packet).
pub fn bandage_target_request(w: &mut World, from: MobileId, item: ItemId, patient: MobileId) {
    if !w
        .item(item)
        .is_some_and(|i| !i.deleted() && i.ty.is(&BANDAGE))
    {
        return;
    }
    if !in_reach(w, item, from) {
        deps::send_localized_message(w, from, 500295); // You are too far away to do that.
        return;
    }
    if let Some(t) = w.mobile(from).and_then(|m| m.target) {
        target::cancel_mobile_target(w, from);
        let _ = t;
        target::clear_target(w, from);
    }
    muo_server::mobile::base::revealing_action(w, from);
    deps::send_localized_message(w, from, 500948); // Who will you use the bandages on?
    let t = internal_target(w, item);
    let tid = target::add_target(w, t);
    target::invoke(w, tid, from, Targeted::Entity(EntityId::Mobile(patient)));
}

fn dye(w: &mut World, id: ItemId, _from: MobileId, tub: ItemId) -> bool {
    if w.item(id).is_none_or(|i| i.deleted()) {
        return false;
    }
    let h = super::dyes::data(w, tub).map_or(0, |d| d.dyed_hue);
    super::hue(w, id, h);
    true
}

/// `Server.Items.Bandage`.
pub static BANDAGE: ItemType = ItemType {
    full_name: "Server.Items.Bandage",
    parent: Some(&ITEM_BASE),
    construct: Some(|w| new_bandage(w, 1)),
    default_weight: |_, _| 0.1,
    on_double_click: double_click,
    serialize: |w, id, out: &mut dyn GenericWriter| {
        (ITEM_BASE.serialize)(w, id, out);
        write_version(out, false);
    },
    deserialize: |w, id, r: &mut dyn GenericReader| -> Result<()> {
        (ITEM_BASE.deserialize)(w, id, r)?;
        read_version(r, false)?;
        Ok(())
    },
    ..ITEM_BASE
};

// ---- BandageContext ----

struct Context {
    patient: MobileId,
    slips: i32,
    timer: TimerHandle,
    /// `Timer.Delay`.
    delay: TimeSpan,
}

/// `BandageContext._table`.
#[derive(Default)]
pub struct BandageTable {
    table: IndexMap<MobileId, Context>,
}

fn table(w: &mut World) -> &mut BandageTable {
    w.resources.get_mut::<BandageTable>()
}

/// Whether `healer` has a `BandageContext` running (`GetContext(healer) != null`).
pub fn has_context(w: &World, healer: MobileId) -> bool {
    w.resources
        .get::<BandageTable>()
        .is_some_and(|t| t.table.contains_key(&healer))
}

/// `GetContext(healer)`'s `Delay` while its timer `Running`: `None` once it fired or was stopped.
pub fn context_delay(w: &World, healer: MobileId) -> Option<TimeSpan> {
    w.resources
        .get::<BandageTable>()
        .and_then(|t| t.table.get(&healer))
        .map(|c| c.delay)
}

/// `context.Slip()`.
pub fn slip(w: &mut World, healer: MobileId) {
    if has_context(w, healer) {
        deps::send_localized_message(w, healer, 500961); // Your fingers slip!
        if let Some(c) = table(w).table.get_mut(&healer) {
            c.slips += 1;
        }
    }
}

/// `context.StopHeal()`.
pub fn stop_heal(w: &mut World, healer: MobileId) {
    if let Some(mut c) = table(w).table.shift_remove(&healer) {
        c.timer.cancel(w);
    }
}

fn monster_or_animal(w: &World, m: MobileId) -> bool {
    !w.mobile(m).is_some_and(|x| x.player)
        && (creature::misc::body_is_monster(w, m) || creature::misc::body_is_animal(w, m))
}

/// `GetPrimarySkill(m)`.
pub fn primary_skill(w: &World, m: MobileId) -> SkillName {
    if monster_or_animal(w, m) {
        SkillName::Veterinary
    } else {
        SkillName::Healing
    }
}

/// `GetSecondarySkill(m)`.
pub fn secondary_skill(w: &World, m: MobileId) -> SkillName {
    if monster_or_animal(w, m) {
        SkillName::AnimalLore
    } else {
        SkillName::Anatomy
    }
}

fn dead_pet(w: &World, m: MobileId) -> bool {
    creature::cd(w, m).is_some_and(|c| c.is_dead_pet)
}

fn alive(w: &World, m: MobileId) -> bool {
    muo_content::mx::alive(w, m)
}

/// `OnTick()`.
fn on_tick(w: &mut World, healer: MobileId) {
    let Some(patient) = table(w).table.get(&healer).map(|c| c.patient) else {
        return;
    };
    let slips = table(w).table.get(&healer).map_or(0, |c| c.slips);
    stop_heal(w, healer);

    let primary = primary_skill(w, patient);
    let secondary = secondary_skill(w, patient);
    let pet = creature::cd(w, patient).is_some();
    let mut healer_number: i32;
    let patient_number: i32;
    let mut play_sound = true;
    let check_skills: bool;
    let say = |w: &mut World, to: MobileId, n: i32| deps::send_localized_message(w, to, n);

    if !alive(w, healer) {
        say(w, healer, 500962); // You were unable to finish your work before you died.
        return;
    }
    let r = range(w);
    let ploc = w
        .mobile(patient)
        .map_or_else(Point3D::default, |m| m.location);
    if !deps::mobile_in_range(w, healer, ploc, r) {
        say(w, healer, 500963); // You did not stay close enough to heal your target.
        return;
    }

    let poisoned = muo_server::mobile::part1::props_b::poisoned(w, patient);
    if !alive(w, patient) || (pet && dead_pet(w, patient)) {
        let map = w.mobile(patient).and_then(|m| m.map);
        let fits = map.is_some_and(|m| {
            muo_server::map::fit::can_fit(w, m, ploc.x, ploc.y, ploc.z, 16, false, false, true)
        });
        if !fits {
            say(w, healer, 501042); // Target can not be resurrected at that location.
            say(w, patient, 502391); // Thou can not be resurrected there!
            return;
        }
        let region = muo_server::mobile::part1::props_b::region(w, patient);
        if muo_server::region::is_part_of_name(w, region, "Khaldun", false) {
            // The veil of death in this area is too strong and resists thy efforts to restore life.
            say(w, healer, 1010395);
            return;
        }
        let healing = muo_server::skills::value(w, healer, primary);
        let anatomy = muo_server::skills::value(w, healer, secondary);
        let chance = (healing - 68.0) / 50.0 - f64::from(slips) * 0.02;
        check_skills = healing >= 80.0 && anatomy >= 80.0;

        let war_horse =
            muo_content::factory::mobile_is(w, patient, "Server.Factions.FactionWarHorse");
        let pet_master = creature::control::control_master(w, patient);
        let roll = check_skills && chance > utility::random_double(&mut w.rng);
        if !roll && (!muo_content::core::se(w) || !pet || !war_horse || pet_master != Some(healer))
        {
            if pet && dead_pet(w, patient) {
                say(w, healer, 503256); // You fail to resurrect the creature.
            } else {
                say(w, healer, 500966); // You are unable to resurrect your patient.
            }
            return;
        }

        healer_number = 500965; // You are able to resurrect your patient.
        patient_number = -1;
        muo_server::mobile::part2::effects::play_sound(w, patient, 0x214);
        muo_server::mobile::part2::effects::fixed_effect(w, patient, 0x376A, 10, 16);

        if pet && dead_pet(w, patient) {
            match pet_master {
                Some(master) if master == healer => {
                    creature::misc::resurrect_pet(w, patient);
                    for i in 0..SkillName::COUNT as i32 {
                        if let Some(s) = SkillName::from_i32(i) {
                            let b = muo_server::skills::base(w, patient, s);
                            muo_server::skills::set_base(w, patient, s, b - 0.1);
                        }
                    }
                }
                Some(master) if mobile_near(w, master, patient, 3) => {
                    healer_number = 503255; // You are able to resurrect the creature.
                    send_pet_gump(w, master, patient);
                }
                _ => {
                    let mut found = false;
                    let friends = creature::cd(w, patient)
                        .and_then(|c| c.friends.clone())
                        .unwrap_or_default();
                    for friend in friends {
                        if mobile_near(w, friend, patient, 3) {
                            healer_number = 503255; // You are able to resurrect the creature.
                            send_pet_gump(w, friend, patient);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        healer_number = 1049670; // The pet's owner must be nearby to attempt resurrection.
                    }
                }
            }
        } else {
            let g = muo_gumps::m_z::resurrect::new(
                Some(healer),
                muo_gumps::m_z::resurrect::ResurrectMessage::Generic,
                false,
                0.0,
                0,
            );
            muo_content::gumps::send_gump_to_mobile(w, patient, Box::new(g), false);
        }
    } else if poisoned {
        say(w, healer, 500969); // You finish applying the bandages.
        let healing = muo_server::skills::value(w, healer, primary);
        let anatomy = muo_server::skills::value(w, healer, secondary);
        let level = w
            .mobile(patient)
            .and_then(|m| m.poison)
            .map_or(0, |p| muo_server::poison::level(w, p));
        let chance = (healing - 30.0) / 50.0 - f64::from(level) * 0.1 - f64::from(slips) * 0.02;
        check_skills = healing >= 60.0 && anatomy >= 60.0;
        if !(check_skills && chance > utility::random_double(&mut w.rng)) {
            say(w, healer, 1010060); // You have failed to cure your target!
            return;
        }
        if muo_server::mobile::base::cure_poison(w, patient, Some(healer)) {
            healer_number = if healer == patient { -1 } else { 1010058 }; // You have cured the target of all poisons.
            patient_number = 1010059; // You have been cured of all poisons.
        } else {
            healer_number = -1;
            patient_number = -1;
        }
    } else {
        if wa::is_bleeding(w, patient) {
            say(w, healer, 1060088); // You bind the wound and stop the bleeding
            say(w, patient, 1060167); // The bleeding wounds have healed, you are no longer bleeding!
            wa::end_bleed(w, patient, false);
            return;
        }
        if wa::is_wounded(w, patient) {
            if healer == patient {
                say(w, healer, 1005000); // You cannot heal yourself in your current state.
            } else {
                say(w, healer, 1010398); // You cannot heal that target in their current state.
            }
            return;
        }
        let (hits, hits_max) = (
            muo_content::mx::hits(w, patient),
            muo_content::mx::hits_max(w, patient),
        );
        if hits == hits_max {
            say(w, healer, 1010395); // You heal what little damage your patient had.
            return;
        }

        check_skills = true;
        patient_number = -1;
        let healing = muo_server::skills::value(w, healer, primary);
        let anatomy = muo_server::skills::value(w, healer, secondary);
        let chance = (healing + 10.0) / 100.0 - f64::from(slips) * 0.02;

        if chance > utility::random_double(&mut w.rng) {
            healer_number = 500969; // You finish applying the bandages.
            let aos = muo_content::core::aos(w);
            let (min, max) = if aos {
                (
                    anatomy / 8.0 + healing / 5.0 + 4.0,
                    anatomy / 6.0 + healing / 2.5 + 4.0,
                )
            } else {
                (
                    anatomy / 5.0 + healing / 5.0 + 3.0,
                    anatomy / 5.0 + healing / 2.0 + 10.0,
                )
            };
            let mut to_heal = min + utility::random_double(&mut w.rng) * (max - min);
            if creature::misc::body_is_monster(w, patient)
                || creature::misc::body_is_animal(w, patient)
            {
                to_heal += f64::from(hits_max) / 100.0;
            }
            if aos {
                to_heal -= to_heal * f64::from(slips) * 0.35; // TODO: Verify algorithm
            } else {
                to_heal -= f64::from(slips * 4);
            }
            if to_heal < 1.0 {
                to_heal = 1.0;
                healer_number = 500968; // You apply the bandages, but they barely help.
            }
            muo_server::mobile::part2::damage::heal_from(
                w,
                patient,
                to_heal as i32,
                Some(healer),
                false,
            );
        } else {
            healer_number = 500968; // You apply the bandages, but they barely help.
            play_sound = false;
        }
    }

    if healer_number != -1 {
        say(w, healer, healer_number);
    }
    if patient_number != -1 {
        say(w, patient, patient_number);
    }
    if play_sound {
        muo_server::mobile::part2::effects::play_sound(w, patient, 0x57);
    }
    if check_skills {
        muo_content::mx::check_skill(w, healer, secondary, 0.0, 120.0);
        muo_content::mx::check_skill(w, healer, primary, 0.0, 120.0);
    }
    let karma = w.mobile(patient).map_or(0, |m| m.karma);
    if healer != patient && karma > 0 {
        muo_content::misc::titles::award_karma(w, healer, karma / 5, true);
    }
}

fn mobile_near(w: &World, a: MobileId, b: MobileId, range: i32) -> bool {
    muo_content::mx::in_range(w, a, b, range)
}

/// `to.SendGump(new PetResurrectGump(healer, pet))`.
fn send_pet_gump(w: &mut World, to: MobileId, pet: MobileId) {
    let g = muo_gumps::m_z::simple::new_pet_resurrect(w, pet, 0.0);
    muo_content::gumps::send_gump_to_mobile(w, to, Box::new(g), false);
}

/// `BandageContext.BeginHeal(healer, patient)`: whether a context was made.
pub fn begin_heal(w: &mut World, healer: MobileId, patient: MobileId) -> bool {
    let creature_data = creature::cd(w, patient).map(|c| c.is_dead_pet);
    let is_dead_pet = creature_data == Some(true);
    let say = |w: &mut World, n: i32| deps::send_localized_message(w, healer, n);
    let (hits, hits_max) = (
        muo_content::mx::hits(w, patient),
        muo_content::mx::hits_max(w, patient),
    );
    let patient_alive = alive(w, patient);
    let ploc = w
        .mobile(patient)
        .map_or_else(Point3D::default, |m| m.location);
    let pmap = w.mobile(patient).and_then(|m| m.map);

    if muo_content::factory::mobile_is(w, patient, "Server.Mobiles.Golem") {
        say(w, 500970); // Bandages cannot be used on that.
    } else if creature_data.is_some() && creature::misc::is_animated_dead(w, patient) {
        say(w, 500951); // You cannot heal that.
    } else if !muo_server::mobile::part1::props_b::poisoned(w, patient)
        && hits == hits_max
        && !wa::is_bleeding(w, patient)
        && !is_dead_pet
    {
        say(w, 500955); // That being is not damaged!
    } else if !patient_alive
        && !pmap.is_some_and(|m| {
            muo_server::map::fit::can_fit(w, m, ploc.x, ploc.y, ploc.z, 16, false, false, true)
        })
    {
        say(w, 501042); // Target cannot be resurrected at that location.
    } else if muo_content::player_mobile::data(w, healer).is_some()
        && muo_content::player_mobile::young(w, healer)
        && muo_content::player_mobile::data(w, patient).is_some()
        && !muo_content::player_mobile::young(w, patient)
    {
        say(w, 500952); // As a young player, you may not use beneficial skills on older players.
    } else if muo_server::mobile::base::can_be_beneficial_full(w, healer, patient, true, true) {
        muo_server::mobile::base::do_beneficial(w, healer, patient);

        let on_self = healer == patient;
        let dex = w.mobile(healer).map_or(0, |m| (m.ty.dex)(w, healer));
        let aos = muo_content::core::aos(w);
        let res_delay = if patient_alive { 0.0 } else { 5.0 };
        let seconds = if on_self {
            if aos {
                5.0 + 0.5 * (f64::from(120 - dex) / 10.0) // TODO: Verify algorithm
            } else {
                9.4 + 0.6 * (f64::from(120 - dex) / 10.0)
            }
        } else if aos && primary_skill(w, patient) == SkillName::Veterinary {
            2.0
        } else if aos {
            if dex < 204 {
                3.2 - (f64::from(dex) / 130.0).sin() * 2.5 + res_delay
            } else {
                0.7 + res_delay
            }
        } else if dex >= 100 {
            3.0 + res_delay
        } else if dex >= 40 {
            4.0 + res_delay
        } else {
            5.0 + res_delay
        };

        stop_heal(w, healer);
        let delay = TimeSpan::from_millis_f64(seconds * 1000.0);
        let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| on_tick(w, healer));
        table(w).table.insert(
            healer,
            Context {
                patient,
                slips: 0,
                timer: h,
                delay,
            },
        );

        if !on_self {
            let name = w
                .mobile(healer)
                .and_then(|m| (m.ty.name)(w, healer))
                .unwrap_or_default();
            muo_server::mobile::part2::messages::send_localized_message(
                w, patient, 1008078, &name, 0x3B2,
            ); // : Attempting to heal you.
        }
        say(w, 500956); // You begin applying the bandages.
        return true;
    }
    false
}

const INT: Kind = Kind::Plain(PropType::Numeric(Numeric::Int));
static BANDAGE_CTOR: [Ctor; 1] = [Ctor {
    access: None,
    params: &[Param {
        name: "amount",
        kind: INT,
        default: Some(Value::Int(1)),
        ty: "",
    }],
    make: |w, v| EntityId::Item(new_bandage(w, v[0].i32())),
}];

fn configure(w: &mut World) {
    add_ctors(w, "Server.Items.Bandage", &BANDAGE_CTOR);
    muo_content::items::interfaces::register_dyable(w, &BANDAGE, dye);
    w.resources
        .get_mut::<muo_content::network::incoming_game::GameHooks>()
        .bandage_target_request = Some(bandage_target_request);
    w.resources
        .get_mut::<muo_content::player_mobile::hooks::SystemHooks>()
        .bandage_slip = Some(slip);
}

/// Registers `Bandage`.
pub fn register(types: &mut TypeRegistry) {
    types.register_item(&BANDAGE, &[]);
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.Bandage.Configure",
        configure,
    );
}
