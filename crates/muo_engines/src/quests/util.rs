//! Small helpers the quest code shares.

use muo_server::World;
use muo_server::ids::{ItemId, MobileId};

/// `creature is T` for a mobile class given by its full type name.
pub fn mobile_is(w: &World, m: MobileId, full_name: &str) -> bool {
    muo_content::factory::mobile_is(w, m, full_name)
}

/// `item is T`.
pub fn item_is(w: &World, i: ItemId, full_name: &str) -> bool {
    muo_content::factory::item_is(w, i, full_name)
}

/// `SendLocalizedMessage(number, "", hue)`.
pub fn msg_hue(w: &mut World, to: MobileId, number: i32, hue: i32) {
    muo_server::mobile::part2::messages::send_localized_message(w, to, number, "", hue);
}

/// `var item = new T(amount); if (player.PlaceInBackpack(item)) true else { item.Delete(); false }`.
/// `amount` is for a stackable class whose `(int amount)` constructor only sets `Amount` (Gold, PowderOfTranslocation).
pub fn give_to_backpack(w: &mut World, to: MobileId, full_name: &str, amount: Option<i32>) -> bool {
    let Some(item) = muo_content::factory::make_item(w, full_name) else {
        return false;
    };
    if let Some(a) = amount {
        muo_server::item::ops::set_amount(w, item, a);
    }
    if muo_server::mobile::part2::items::place_in_backpack(w, to, item) {
        true
    } else {
        muo_server::item::base::delete(w, item);
        false
    }
}

/// `Utility.RandomMinMax(min, max)`.
pub fn random_min_max(w: &mut World, min: i32, max: i32) -> i32 {
    muo_content::hooks::rnd::min_max(w, min, max)
}

/// `LocalOverheadMessage(MessageType.Regular, hue, number)`.
pub fn local_overhead_number(w: &mut World, m: MobileId, hue: i32, number: i32) {
    muo_server::mobile::part2::messages::local_overhead_message_number(
        w,
        m,
        muo_primitives::MessageType::REGULAR,
        hue,
        number,
        "",
    );
}

/// `ns.SendDisplayHelpTopic(topic, display)`: the 0xBF/0x17 packet.
pub fn send_display_help_topic(w: &mut World, to: MobileId, topic: i32, display: bool) {
    let Some(ns) = muo_server::mobile::part1::props_b::net_state(w, to) else {
        return;
    };
    if w.cannot_send_packets(ns) {
        return;
    }
    let mut p = vec![0xBF, 0, 11, 0, 0x17, 1];
    p.extend_from_slice(&topic.to_be_bytes());
    p.push(u8::from(display));
    w.send(ns, &p);
}

/// `Body = value`.
pub fn set_body(w: &mut World, id: MobileId, body: i32) {
    muo_server::mobile::part1::props_b::set_body(w, id, muo_primitives::Body(body));
}

/// `SayTo(to, number)`.
pub fn say_to_number(w: &mut World, speaker: MobileId, to: MobileId, number: i32) {
    muo_server::mobile::part2::speech::say_to_number(w, speaker, to, number, "");
}

/// `PublicOverheadMessage(MessageType.Regular, hue, number, args)`.
pub fn public_overhead(w: &mut World, speaker: MobileId, hue: i32, number: i32, args: &str) {
    muo_server::mobile::part2::messages::public_overhead_message_number(
        w,
        speaker,
        muo_primitives::MessageType::REGULAR,
        hue,
        number,
        args,
        false,
    );
}

/// `new T(args...)` made by its class name and added to `m`'s equipment (`AddItem`).
pub fn add_item_named(w: &mut World, m: MobileId, full_name: &str) -> Option<ItemId> {
    let it = muo_content::factory::make_item(w, full_name)?;
    muo_server::mobile::part2::items::add_item(w, m, it);
    Some(it)
}

/// `AddItem(new T { Hue = hue })` of a class made by name.
pub fn add_item_hued(w: &mut World, m: MobileId, full_name: &str, hue: i32) -> Option<ItemId> {
    let it = muo_content::factory::make_item(w, full_name)?;
    super::quester::set_hue(w, it, hue);
    muo_server::mobile::part2::items::add_item(w, m, it);
    Some(it)
}

/// `from.Target = new Target(-1, false, TargetFlags.None) { OnTarget, OnTargetCancel }` with `CheckLOS` as given.
pub fn begin_target(
    w: &mut World,
    from: MobileId,
    check_los: bool,
    on_target: impl Fn(&mut World, MobileId, muo_server::target::Targeted) + 'static,
    on_cancel: Option<fn(&mut World, MobileId)>,
) {
    use muo_server::target::{Target, TargetFlags, add_target, set_mobile_target};
    let mut t = Target::new(w, -1, false, TargetFlags::NONE);
    t.check_los = check_los;
    t.handlers.on_target = Some(std::rc::Rc::new(on_target));
    if let Some(c) = on_cancel {
        t.handlers.on_target_cancel =
            Some(std::rc::Rc::new(move |w: &mut World, from, _| c(w, from)));
    }
    let id = add_target(w, t);
    set_mobile_target(w, from, Some(id));
}

/// `item.IsChildOf(from.Backpack)`.
pub fn in_backpack(w: &World, item: ItemId, from: MobileId) -> bool {
    let pack = w.mobile(from).and_then(|m| m.backpack);
    pack.is_some_and(|p| {
        muo_server::item::ops::is_child_of(w, item, Some(muo_server::ids::EntityId::Item(p)))
    })
}

/// `Utility.AssignRandomHair(m, false)`: a random hair item, the hue stays.
pub fn assign_hair_item(w: &mut World, m: MobileId) {
    let hue = w.mobile(m).map_or(0, |x| x.hair_hue);
    muo_content::vendor::assign_random_hair(w, m, hue);
}

/// `Utility.AssignRandomFacialHair(m, false)`: a random facial hair item, the hue stays.
pub fn assign_facial_item(w: &mut World, m: MobileId) {
    let hue = w.mobile(m).map_or(0, |x| x.facial_hair_hue);
    muo_content::vendor::assign_random_facial_hair(w, m, hue);
}

/// `Utility.AssignRandomFacialHair(m)`: the item, then `Race.RandomHairHue()`.
pub fn assign_facial_random_hue(w: &mut World, m: MobileId) {
    assign_facial_item(w, m);
    let hue = muo_content::vendor::race_random_hair_hue(w, m);
    muo_server::mobile::part1::props_b::set_facial_hair_hue(w, m, hue);
}

/// `item.Amount`.
pub fn amount_of(w: &World, item: ItemId) -> i32 {
    w.item(item).map_or(0, |i| i.amount)
}

/// `item.Deleted` (a gone item counts as deleted).
pub fn deleted_item(w: &World, item: ItemId) -> bool {
    w.item(item).is_none_or(|i| i.deleted())
}

/// `EquipItem(new T())`: equipped, or deleted when it does not fit.
pub fn equip_new(w: &mut World, m: MobileId, full_name: &str) -> Option<ItemId> {
    let it = muo_content::factory::make_item(w, full_name)?;
    if !muo_server::mobile::base::equip_item(w, m, it) {
        muo_server::item::base::delete(w, it);
        return None;
    }
    Some(it)
}

/// `BaseSoundID = value`.
pub fn base_sound_id(w: &mut World, id: MobileId, value: i32) {
    if let Some(m) = w.mobile_mut(id) {
        m.base_sound_id = value;
    }
}

/// `CantWalk = value`.
pub fn cant_walk(w: &mut World, id: MobileId, value: bool) {
    if let Some(m) = w.mobile_mut(id) {
        m.cant_walk = value;
    }
}

/// `Fame = fame; Karma = karma`.
pub fn fame_karma(w: &mut World, id: MobileId, fame: i32, karma: i32) {
    muo_server::mobile::part1::props_a::set_fame(w, id, fame);
    muo_server::mobile::part1::props_a::set_karma(w, id, karma);
}

/// One `BaseCreature` subclass of the quests: `[SerializationGenerator(0)]` (an encoded version), the constructor
/// `base(AIType.<ai>, FightMode.Aggressor)` followed by `init`.
#[macro_export]
macro_rules! quest_creature {
    // `template`: the constructor is the class's mobile template (`Data/Scripts/classes/mobiles`); the name, corpse name
    // and `AlwaysMurderer` are its props.
    (
        $(#[$m:meta])*
        $name:ident, $cname:ident = $full:literal, template;
        m { $($mf:ident : $mv:expr,)* }
    ) => {
        pub static $cname: muo_content::creature::CreatureType = muo_content::creature::CreatureType {
            default_name: muo_content::templates::creature_props::default_name,
            corpse_name: muo_content::templates::creature_props::corpse_name,
            always_murderer: muo_content::templates::creature_props::always_murderer,
            ..muo_content::creature::BASE_CREATURE_C
        };
        $(#[$m])*
        pub static $name: muo_server::mobile::MobileType = muo_server::mobile::MobileType {
            full_name: $full,
            parent: Some(&muo_content::creature::BASE_CREATURE),
            construct: Some(|w| muo_content::templates::new_creature(w, &$name)),
            family: Some(&$cname),
            serialize: |w, id, out| {
                (muo_content::creature::BASE_CREATURE.serialize)(w, id, out);
                out.write_encoded_int(0);
            },
            deserialize: |w, id, r| {
                (muo_content::creature::BASE_CREATURE.deserialize)(w, id, r)?;
                r.read_encoded_int()?;
                Ok(())
            },
            $($mf: $mv,)*
            ..muo_content::creature::BASE_CREATURE
        };
    };
    (
        $(#[$m:meta])*
        $name:ident, $cname:ident = $full:literal, ai $ai:ident, init |$w:ident, $id:ident| $init:block;
        c { $($cf:ident : $cv:expr,)* }
        m { $($mf:ident : $mv:expr,)* }
    ) => {
        pub static $cname: muo_content::creature::CreatureType =
            muo_content::creature::CreatureType { $($cf: $cv,)* ..muo_content::creature::BASE_CREATURE_C };
        $(#[$m])*
        pub static $name: muo_server::mobile::MobileType = muo_server::mobile::MobileType {
            full_name: $full,
            parent: Some(&muo_content::creature::BASE_CREATURE),
            construct: Some(|$w| {
                let $id = muo_content::creature::setters::new_creature(
                    $w,
                    &$name,
                    muo_content::creature::enums::AIType::$ai,
                    muo_content::creature::enums::FightMode::Aggressor,
                    muo_content::creature::enums::DEFAULT_RANGE_PERCEPTION,
                    1,
                );
                $init
                $id
            }),
            family: Some(&$cname),
            serialize: |w, id, out| {
                (muo_content::creature::BASE_CREATURE.serialize)(w, id, out);
                out.write_encoded_int(0);
            },
            deserialize: |w, id, r| {
                (muo_content::creature::BASE_CREATURE.deserialize)(w, id, r)?;
                r.read_encoded_int()?;
                Ok(())
            },
            $($mf: $mv,)*
            ..muo_content::creature::BASE_CREATURE
        };
    };
}

/// `backpack.GetAmount(typeof(T))` for a class given by its full name.
pub fn amount_in(w: &World, container: Option<ItemId>, full_name: &str) -> i32 {
    let Some(c) = container else { return 0 };
    match w.types.find_type_by_full_name(full_name, false) {
        Some(muo_server::type_registry::EntityType::Item(t)) => {
            muo_server::item::container::get_amount(w, c, t, true)
        }
        _ => 0,
    }
}

/// `new T(amount)` of a stackable class, dropped into `cont`.
pub fn drop_amount(w: &mut World, cont: ItemId, full_name: &str, amount: i32) {
    if let Some(i) = muo_content::factory::make_item(w, full_name) {
        muo_server::item::ops::set_amount(w, i, amount);
        muo_server::item::container::drop_item_default(w, cont, i);
    }
}

/// `new T()` dropped into `cont`.
pub fn drop_new(w: &mut World, cont: ItemId, full_name: &str) -> Option<ItemId> {
    let i = muo_content::factory::make_item(w, full_name)?;
    muo_server::item::container::drop_item_default(w, cont, i);
    Some(i)
}

fn human_ops(w: &World) -> Option<&'static muo_server::race::RaceOps> {
    Some(
        w.resources
            .get::<muo_server::race::Races>()?
            .default_race()?
            .ops,
    )
}

/// `Race.Human.RandomHair(female)`.
pub fn human_random_hair(w: &mut World, female: bool) -> i32 {
    match human_ops(w) {
        Some(o) => (o.random_hair)(&mut w.rng, female),
        None => 0,
    }
}

/// `Race.Human.RandomFacialHair(female)`.
pub fn human_random_facial_hair(w: &mut World, female: bool) -> i32 {
    match human_ops(w) {
        Some(o) => (o.random_facial_hair)(&mut w.rng, female),
        None => 0,
    }
}

/// `Race.Human.RandomHairHue()`.
pub fn human_random_hair_hue(w: &mut World) -> i32 {
    match human_ops(w) {
        Some(o) => (o.random_hair_hue)(&mut w.rng),
        None => 0,
    }
}

/// `Say(number)`.
pub fn say_number(w: &mut World, m: MobileId, number: i32) {
    muo_server::mobile::part2::messages::say_number(w, m, number, "");
}
