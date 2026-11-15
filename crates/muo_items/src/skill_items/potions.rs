//! `BasePotion` and the potion families, `Bottle` and `BlankScroll`.
//! State: [`PotionData`] (`PotionEffect`) in `Item::ext`; what a potion class does is its [`PotionClass`] (the `Kind`
//! carries the numbers each subclass overrides). The abstract families (`BaseHealPotion`, ..) are item types of
//! their own because other systems test for them by name; the thrown potions are in `potion_throw.rs`.

use muo_content::aos::AosAttribute;
use muo_content::items::equip_hooks::CraftArgs;
use muo_content::skills::hooks::PoisonPotionFacts;
use muo_primitives::{TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::item::{ITEM_BASE, ItemType, base, deps, effect_item, enumerable, ops};
use muo_server::layer::Layer;
use muo_server::mobile::StatType;
use muo_server::mobile::part2::{effects, messages};
use muo_server::packets::message::MessageType;
use muo_server::poison::{PoisonId, Poisons};
use muo_server::type_registry::{Phase, TypeRegistry};

pub use super::plain::new_bottle;
use super::{hue, read_version, write_version};

/// What differs between potion classes: the overrides of `BasePotion`' subclasses.
#[derive(Copy, Clone)]
pub enum Kind {
    /// `InvisibilityPotion`, `NightSightPotion`: their own code.
    Invisibility,
    NightSight,
    /// `BaseHealPotion`: `(min, max, delay seconds)` for AOS and before.
    Heal {
        aos: (i32, i32, f64),
        pre: (i32, i32, f64),
    },
    /// `BaseCurePotion`: `LevelInfo` as (poison name, chance) for AOS and before.
    Cure {
        aos: &'static [(&'static str, f64)],
        pre: &'static [(&'static str, f64)],
    },
    /// `BaseRefreshPotion.Refresh`.
    Refresh(f64),
    /// `BaseAgilityPotion.DexOffset` (`Duration` is 2 minutes in every class).
    Agility(i32),
    /// `BaseStrengthPotion.StrOffset`.
    Strength(i32),
    /// `BasePoisonPotion`: poison name, `MinPoisoningSkill`, `MaxPoisoningSkill`.
    Poison(&'static str, f64, f64),
    /// `BaseExplosionPotion`: `(MinDamage, MaxDamage)` for AOS and before.
    Explosion {
        aos: (i32, i32),
        pre: (i32, i32),
    },
    /// `BaseConflagrationPotion`.
    Conflagration(i32, i32),
    /// `BaseConfusionBlastPotion.Radius`.
    Confusion(i32),
}

/// A potion class' constants.
#[derive(Copy, Clone)]
pub struct PotionClass {
    pub kind: Kind,
    /// `LabelNumber` override; 0 is `1041314 + PotionEffect`.
    pub label: i32,
}

pub static BASE_POTION_CLASS: PotionClass = PotionClass {
    kind: Kind::NightSight,
    label: 0,
};

/// `BasePotion._potionEffect`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PotionData {
    pub effect: i32,
}

pub fn class(w: &World, id: ItemId) -> &'static PotionClass {
    w.item(id)
        .and_then(|i| i.ty.family::<PotionClass>())
        .unwrap_or(&BASE_POTION_CLASS)
}

/// `PotionEffect` of a potion.
pub fn effect(w: &World, id: ItemId) -> Option<i32> {
    w.item(id)?.ext.get::<PotionData>().map(|d| d.effect)
}

/// `item is BasePotion`.
pub fn is_potion(w: &World, id: ItemId) -> bool {
    w.item(id).is_some_and(|i| i.ty.is(&BASE_POTION))
}

/// `PotionEffect` numbers.
pub mod fx {
    pub const NIGHTSIGHT: i32 = 0;
    pub const CURE_LESSER: i32 = 1;
    pub const CURE: i32 = 2;
    pub const CURE_GREATER: i32 = 3;
    pub const AGILITY: i32 = 4;
    pub const AGILITY_GREATER: i32 = 5;
    pub const STRENGTH: i32 = 6;
    pub const STRENGTH_GREATER: i32 = 7;
    pub const POISON_LESSER: i32 = 8;
    pub const POISON: i32 = 9;
    pub const POISON_GREATER: i32 = 10;
    pub const POISON_DEADLY: i32 = 11;
    pub const REFRESH: i32 = 12;
    pub const REFRESH_TOTAL: i32 = 13;
    pub const HEAL_LESSER: i32 = 14;
    pub const HEAL: i32 = 15;
    pub const HEAL_GREATER: i32 = 16;
    pub const EXPLOSION_LESSER: i32 = 17;
    pub const EXPLOSION: i32 = 18;
    pub const EXPLOSION_GREATER: i32 = 19;
    pub const CONFLAGRATION: i32 = 20;
    pub const CONFLAGRATION_GREATER: i32 = 21;
    pub const CONFUSION_BLAST: i32 = 24;
    pub const CONFUSION_BLAST_GREATER: i32 = 25;
    pub const INVISIBILITY: i32 = 26;
    pub const PARASITIC: i32 = 27;
    pub const DARKGLOW: i32 = 28;
    pub const FLINTS_PUNGENT_BREW: i32 = 29;
}

/// `new BasePotion(itemID, effect)` of class `ty`.
pub fn new_potion(w: &mut World, ty: &'static ItemType, item_id: i32, effect: i32) -> ItemId {
    let id = ops::new_item_with_id(w, ty, item_id);
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(PotionData { effect });
    }
    let stackable = muo_content::core::ml(w);
    ops::set_stackable(w, id, stackable);
    id
}

// ---- BasePotion statics ----

/// `EnhancePotions(m)`.
pub fn enhance_potions(w: &mut World, m: MobileId) -> i32 {
    let mut ep =
        muo_content::misc::aos_helpers::attribute_value(w, m, AosAttribute::EnhancePotions);
    let alchemy = muo_server::skills::value(w, m, muo_server::skills::SkillName::Alchemy);
    let skill_bonus = (alchemy * 10.0 / 33.0) as i32;
    let player = w
        .mobile(m)
        .is_some_and(|x| x.access_level <= muo_server::mobile::AccessLevel::Player);
    if muo_content::core::ml(w) && ep > 50 && player {
        ep = 50;
    }
    ep + skill_bonus
}

/// `Scale(m, int)`.
pub fn scale(w: &mut World, m: MobileId, v: i32) -> i32 {
    if !muo_content::core::aos(w) {
        return v;
    }
    let ep = enhance_potions(w, m);
    muo_content::misc::aos_helpers::scale(v, 100 + ep)
}

/// `Scale(m, double)`.
pub fn scale_f(w: &mut World, m: MobileId, v: f64) -> f64 {
    if !muo_content::core::aos(w) {
        return v;
    }
    v * (1.0 + 0.01 * f64::from(enhance_potions(w, m)))
}

/// `HasFreeHand(m)`.
pub fn has_free_hand(w: &World, m: MobileId) -> bool {
    let mut hand_one = deps::find_item_on_layer(w, m, Layer::OneHanded);
    let hand_two = deps::find_item_on_layer(w, m, Layer::TwoHanded);
    if let Some(h2) = hand_two
        && muo_content::items::weapon::is_weapon(w, h2)
    {
        hand_one = Some(h2);
    }
    if let Some(h2) = hand_two
        && w.item(h2)
            .is_some_and(|i| i.ty.is(&muo_content::items::weapon::BASE_RANGED))
        && muo_content::items::weapon::data(w, h2).is_some_and(|d| d.balanced)
    {
        return true;
    }
    hand_one.is_none() || hand_two.is_none()
}

fn require_free_hand(w: &World, id: ItemId) -> bool {
    !matches!(
        class(w, id).kind,
        Kind::Explosion { .. } | Kind::Conflagration(..) | Kind::Confusion(_)
    )
}

/// `IsThrowablePotion`.
pub fn is_throwable(w: &World, id: ItemId) -> bool {
    !require_free_hand(w, id)
}

fn base_can_drink(w: &mut World, id: ItemId, from: MobileId) -> bool {
    if !w.item(id).is_some_and(|i| i.movable()) {
        return false;
    }
    let loc = ops::get_world_location(w, id);
    if !deps::mobile_in_range(w, from, loc, 1) {
        deps::send_localized_message(w, from, 502138); // That is too far away for you to use
        return false;
    }
    if require_free_hand(w, id) && !has_free_hand(w, from) {
        deps::send_localized_message(w, from, 502172); // You must have a free hand to drink a potion.
        return false;
    }
    true
}

/// `PlayDrinkEffect(m)`.
pub fn play_drink_effect(w: &mut World, id: ItemId, m: MobileId) {
    muo_server::mobile::base::revealing_action(w, m);
    effects::play_sound(w, m, 0x2D6);
    let free = w
        .resources
        .get::<muo_content::spells::hooks::SpellHooks>()
        .and_then(|h| h.duel_is_free_consume)
        .is_some_and(|f| f(w, m));
    if !free {
        base::consume(w, id, 1);
        if let Some(b) = muo_content::factory::make_item(w, "Server.Items.Bottle") {
            muo_server::mobile::part2::items::add_to_backpack(w, m, b);
        }
    }
    let human = w.mobile(m).is_some_and(|_| {
        w.resources
            .get::<muo_primitives::BodyTable>()
            .is_some_and(|t| t.is_human(muo_server::mobile::part1::props_b::body(w, m)))
    });
    if human && !muo_server::mobile::part1::props_b::mounted(w, m) {
        muo_server::mobile::base::animate(w, m, 34, 5, 1, true, false, 0);
    }
}

/// `CanDrink(from)` through the class.
pub fn can_drink(w: &mut World, id: ItemId, from: MobileId) -> bool {
    // `MelisandesFermentedWine.CanDrink`: `MondainsLegacy.CheckML(from) && base.CanDrink(from)`
    if w.item(id)
        .is_some_and(|i| i.ty.full_name == "Server.Items.MelisandesFermentedWine")
        && !crate::misc::dyes::check_ml(w, from, true)
    {
        return false;
    }
    if !base_can_drink(w, id, from) {
        return false;
    }
    match class(w, id).kind {
        Kind::Heal { .. } => heal_can_drink(w, from),
        Kind::Cure { .. } => cure_can_drink(w, from),
        Kind::Refresh(_) => {
            let (stam, max) = (
                w.mobile(from).map_or(0, |m| m.stam),
                muo_server::mobile::base::stam_max(w, from),
            );
            if stam >= max {
                messages::send_message(
                    w,
                    from,
                    "You decide against drinking this potion, as you are already at full stamina.",
                );
                return false;
            }
            true
        }
        Kind::Agility(off) => stat_can_drink(w, from, StatType::DEX, off),
        Kind::Strength(off) => stat_can_drink(w, from, StatType::STR, off),
        Kind::Invisibility => invisibility_can_drink(w, from),
        Kind::NightSight => {
            if !muo_server::mobile::part1::combat::begin_action(w, from, "Server.LightCycle") {
                messages::send_message(w, from, "You already have night sight.");
                return false;
            }
            true
        }
        Kind::Explosion { .. } | Kind::Conflagration(..) | Kind::Confusion(_) => {
            super::potion_throw::can_drink(w, id, from)
        }
        Kind::Poison(..) => true,
    }
}

/// `Drink(from)` through the class.
pub fn drink(w: &mut World, id: ItemId, from: MobileId) {
    match class(w, id).kind {
        Kind::Heal { aos, pre } => {
            let (min, max, delay) = if muo_content::core::aos(w) { aos } else { pre };
            let (min, max) = (scale(w, from, min), scale(w, from, max));
            let amount = utility::random_min_max(&mut w.rng, min, max);
            muo_server::mobile::part2::damage::heal(w, from, amount);
            play_drink_effect(w, id, from);
            muo_server::timer::start(
                w,
                TimeSpan::from_secs_f64(delay),
                TimeSpan::ZERO,
                1,
                move |w| {
                    muo_server::mobile::part1::combat::end_action(
                        w,
                        from,
                        "Server.Items.BaseHealPotion",
                    );
                },
            );
        }
        Kind::Cure { aos, pre } => {
            let info = if muo_content::core::aos(w) { aos } else { pre };
            do_cure(w, from, info);
            play_drink_effect(w, id, from);
            effects::fixed_particles(
                w,
                from,
                0x373A,
                10,
                15,
                5012,
                muo_server::effects::EffectLayer::Waist,
            );
            effects::play_sound(w, from, 0x1E0);
        }
        Kind::Refresh(r) => {
            let stam_max = muo_server::mobile::base::stam_max(w, from);
            let add = scale(w, from, (r * f64::from(stam_max)) as i32);
            let cur = w.mobile(from).map_or(0, |m| m.stam);
            muo_server::mobile::part1::props_b::set_stam(w, from, cur + add);
            play_drink_effect(w, id, from);
        }
        Kind::Agility(_) | Kind::Strength(_) => play_drink_effect(w, id, from),
        Kind::Poison(name, ..) => {
            if let Some(p) = poison_named(w, name) {
                muo_server::mobile::base::apply_poison(w, from, Some(from), p);
            }
            play_drink_effect(w, id, from);
        }
        Kind::Invisibility => invisibility_drink(w, id, from),
        Kind::NightSight => {
            muo_content::misc::light_cycle::start_night_sight_timer(w, from);
            muo_server::mobile::part1::props_a::set_light_level(
                w,
                from,
                muo_content::misc::light_cycle::DUNGEON_LEVEL / 2,
            );
            effects::fixed_particles(
                w,
                from,
                0x376A,
                9,
                32,
                5007,
                muo_server::effects::EffectLayer::Waist,
            );
            effects::play_sound(w, from, 0x1E3);
            play_drink_effect(w, id, from);
        }
        Kind::Explosion { .. } | Kind::Conflagration(..) | Kind::Confusion(_) => {
            super::potion_throw::drink(w, id, from)
        }
    }
}

fn poison_named(w: &World, name: &str) -> Option<PoisonId> {
    w.resources
        .get::<Poisons>()?
        .get_poison(name)
        .map(|p| p.id())
}

// ---- heal ----

fn heal_can_drink(w: &mut World, from: MobileId) -> bool {
    let (hits, max) = (
        w.mobile(from).map_or(0, |m| m.hits),
        muo_server::mobile::base::hits_max(w, from),
    );
    if hits >= max {
        deps::send_localized_message(w, from, 1049547); // You decide against drinking this potion, as you are already at full health.
        return false;
    }
    if muo_server::mobile::part1::props_b::poisoned(w, from)
        || muo_content::items::weapon_ability::is_wounded(w, from)
    {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x22, 1005000); // You can not heal yourself in your current state.
        return false;
    }
    if !muo_server::mobile::part1::combat::begin_action(w, from, "Server.Items.BaseHealPotion") {
        deps::local_overhead_message(w, from, MessageType::REGULAR, 0x22, 500235); // You must wait 10 seconds before using another healing potion.
        return false;
    }
    true
}

// ---- cure ----

fn cure_can_drink(w: &mut World, from: MobileId) -> bool {
    if muo_content::spells::helper::under_transformation_of(
        w,
        from,
        "Server.Spells.Necromancy.VampiricEmbraceSpell",
    ) {
        deps::send_localized_message(w, from, 1061652); // The garlic in the potion would surely kill you.
        return false;
    }
    if !muo_server::mobile::part1::props_b::poisoned(w, from) {
        deps::send_localized_message(w, from, 1042000); // You are not poisoned.
        return false;
    }
    true
}

fn do_cure(w: &mut World, from: MobileId, info: &[(&str, f64)]) {
    let mut cure = false;
    for &(name, chance) in info {
        let current = w.mobile(from).and_then(|m| m.poison);
        if poison_named(w, name) == current && current.is_some() {
            let scaled = scale_f(w, from, chance);
            if scaled > utility::random_double(&mut w.rng) {
                cure = true;
                break;
            }
        }
    }
    if cure && muo_server::mobile::base::cure_poison(w, from, Some(from)) {
        deps::send_localized_message(w, from, 500231); // You feel cured of poison!
        effects::fixed_effect(w, from, 0x373A, 10, 15);
        effects::play_sound(w, from, 0x1E0);
    } else if !cure {
        deps::send_localized_message(w, from, 500232); // That potion was not strong enough to cure your ailment!
    }
}

// ---- agility and strength ----

fn stat_can_drink(w: &mut World, from: MobileId, stat: StatType, offset: i32) -> bool {
    // TODO: Verify scaled; is it offset, duration, or both?
    let off = scale(w, from, offset);
    if !muo_content::spells::helper::add_stat_offset(w, from, stat, off, TimeSpan::from_mins(2)) {
        deps::send_localized_message(w, from, 502173); // You are already under a similar effect.
        return false;
    }
    effects::fixed_effect(w, from, 0x375A, 10, 15);
    effects::play_sound(w, from, 0x1E7);
    true
}

// ---- invisibility ----

#[derive(Default)]
struct InvisibilityTable(std::collections::HashMap<MobileId, muo_server::timer::TimerHandle>);

fn has_timer(w: &World, m: MobileId) -> bool {
    w.resources
        .get::<InvisibilityTable>()
        .is_some_and(|t| t.0.contains_key(&m))
}

fn invisibility_can_drink(w: &mut World, from: MobileId) -> bool {
    if w.mobile(from).is_some_and(|m| m.hidden) {
        deps::send_localized_message(w, from, 1073185); // You are already unseen.
        return false;
    }
    if has_timer(w, from) {
        deps::send_localized_message(w, from, 1073186); // An invisibility potion is already taking effect on your person.
        return false;
    }
    true
}

fn invisibility_drink(w: &mut World, id: ItemId, from: MobileId) {
    let h = muo_server::timer::start(w, TimeSpan::from_secs(2), TimeSpan::ZERO, 1, move |w| {
        hide(w, from)
    });
    w.resources.get_mut::<InvisibilityTable>().0.insert(from, h);
    play_drink_effect(w, id, from);
}

/// `InvisibilityPotion.Hide(m)`.
pub fn hide(w: &mut World, m: MobileId) {
    let Some((loc, map)) = w.mobile(m).map(|x| (x.location, x.map)) else {
        return;
    };
    let fx = effect_item::create(
        w,
        muo_primitives::Point3D::new(loc.x, loc.y, loc.z + 16),
        map,
        effect_item::DEFAULT_DURATION,
    );
    muo_server::effects::send_location_particles(w, fx, 0x376A, 10, 15, 5045);
    effects::play_sound(w, m, 0x3C4);
    muo_server::mobile::part1::props_b::set_hidden(w, m, true);
    if w.mobile(m)
        .is_some_and(|x| x.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
    {
        use muo_content::player_mobile::buff::{BuffInfo, add_buff, remove_buff};
        use muo_content::player_mobile::buff_icon::BuffIcon;
        remove_buff(w, m, BuffIcon::HidingAndOrStealth);
        add_buff(w, m, BuffInfo::new(BuffIcon::Invisibility, 1075825)); // Invisibility/Invisible
    }
    remove_timer(w, m, false);
    muo_server::timer::start(w, TimeSpan::from_secs(30), TimeSpan::ZERO, 1, move |w| {
        end_hide(w, m)
    });
}

fn end_hide(w: &mut World, m: MobileId) {
    muo_server::mobile::base::revealing_action(w, m);
    remove_timer(w, m, false);
}

/// `InvisibilityPotion.RemoveTimer(m, interrupted)`.
pub fn remove_timer(w: &mut World, m: MobileId, interrupted: bool) {
    let h = w.resources.get_mut::<InvisibilityTable>().0.remove(&m);
    if let Some(mut h) = h {
        if interrupted {
            deps::send_localized_message(w, m, 1073187); // The invisibility effect is interrupted.
        }
        h.cancel(w);
    }
}

// ---- double click ----

fn on_double_click(w: &mut World, id: ItemId, from: MobileId) {
    if !can_drink(w, id, from) {
        return;
    }
    let mut pot = id;
    if is_throwable(w, id) && w.item(id).is_some_and(|i| i.amount > 1) {
        let Some(ty) = w.item(id).map(|i| i.ty) else {
            return;
        };
        let Some(new) = ty.construct.map(|f| f(w)) else {
            return;
        };
        pot = new;
        let amount = w.item(id).map_or(1, |i| i.amount);
        ops::set_amount(w, id, amount - 1);
        let pack = w
            .mobile(from)
            .and_then(|m| m.backpack)
            .filter(|&p| w.item(p).is_some_and(|i| !i.deleted()));
        if let Some(p) = pack {
            if let Some(pty) = w.item(p).map(|i| i.ty) {
                (pty.drop_item)(w, p, pot);
            }
        } else if let Some((loc, map)) = w.mobile(from).map(|m| (m.location, m.map)) {
            ops::move_to_world_map(w, pot, loc, map);
        }
    }
    drink(w, pot, from);
}

fn stack_with(
    w: &mut World,
    id: ItemId,
    from: Option<MobileId>,
    dropped: ItemId,
    play_sound: bool,
) -> bool {
    is_potion(w, dropped)
        && effect(w, dropped) == effect(w, id)
        && base::stack_with_base(w, id, from, dropped, play_sound)
}

// ---- save ----

fn serialize(w: &World, id: ItemId, out: &mut dyn muo_io::GenericWriter) {
    (ITEM_BASE.serialize)(w, id, out);
    out.write_i32(2);
    out.write_enum(4, i64::from(effect(w, id).unwrap_or(0)));
}

fn deserialize(w: &mut World, id: ItemId, r: &mut dyn muo_io::GenericReader) -> muo_io::Result<()> {
    (ITEM_BASE.deserialize)(w, id, r)?;
    let version = r.read_i32()?;
    let effect = if version < 2 {
        r.read_i32()?
    } else {
        r.read_enum(4)? as i32
    };
    if let Some(i) = w.item_mut(id) {
        i.ext = Ext::new(PotionData { effect });
    }
    Ok(())
}

/// `Server.Items.BasePotion` (abstract).
pub static BASE_POTION: ItemType = ItemType {
    full_name: "Server.Items.BasePotion",
    parent: Some(&ITEM_BASE),
    construct: None,
    family: Some(&BASE_POTION_CLASS),
    default_weight: crate::class_props::default_weight,
    label_number: |w, id| {
        let c = class(w, id);
        if c.label != 0 {
            c.label
        } else {
            1041314 + effect(w, id).unwrap_or(0)
        }
    },
    on_double_click,
    stack_with,
    serialize,
    deserialize,
    ..ITEM_BASE
};

macro_rules! families {
    ($($name:ident = $full:literal;)*) => {$(
        #[doc = concat!("`", $full, "` (abstract).")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&BASE_POTION),
            serialize: |w, id, out| {
                (BASE_POTION.serialize)(w, id, out);
                write_version(out, false);
            },
            deserialize: |w, id, r| {
                (BASE_POTION.deserialize)(w, id, r)?;
                read_version(r, false)?;
                Ok(())
            },
            ..BASE_POTION
        };
    )*};
}

families! {
    BASE_HEAL_POTION = "Server.Items.BaseHealPotion";
    BASE_CURE_POTION = "Server.Items.BaseCurePotion";
    BASE_REFRESH_POTION = "Server.Items.BaseRefreshPotion";
    BASE_AGILITY_POTION = "Server.Items.BaseAgilityPotion";
    BASE_STRENGTH_POTION = "Server.Items.BaseStrengthPotion";
    BASE_POISON_POTION = "Server.Items.BasePoisonPotion";
    BASE_EXPLOSION_POTION = "Server.Items.BaseExplosionPotion";
    BASE_CONFLAGRATION_POTION = "Server.Items.BaseConflagrationPotion";
    BASE_CONFUSION_BLAST_POTION = "Server.Items.BaseConfusionBlastPotion";
}

macro_rules! potions {
    ($($name:ident, $new:ident = $full:literal : $parent:ident, gid $gid:expr, effect $fx:expr, hue $hue:expr, label $label:expr, $kind:expr;)*) => {$(
        #[doc = concat!("`new ", $full, "()`.")]
        pub fn $new(w: &mut World) -> ItemId {
            let id = new_potion(w, &$name, $gid, $fx);
            if $hue != 0 {
                hue(w, id, $hue);
            }
            id
        }
        #[doc = concat!("`", $full, "`.")]
        pub static $name: ItemType = ItemType {
            full_name: $full,
            parent: Some(&$parent),
            construct: Some($new),
            family: Some(&PotionClass { kind: $kind, label: $label }),
            serialize: |w, id, out| {
                ($parent.serialize)(w, id, out);
                write_version(out, false);
            },
            deserialize: |w, id, r| {
                ($parent.deserialize)(w, id, r)?;
                read_version(r, false)?;
                Ok(())
            },
            ..$parent
        };
    )*};
}

const LESSER_OLD: &[(&str, f64)] = &[("Lesser", 0.75), ("Regular", 0.50), ("Greater", 0.15)];
const LESSER_AOS: &[(&str, f64)] = &[("Lesser", 0.75), ("Regular", 0.50), ("Greater", 0.25)];
const CURE_OLD: &[(&str, f64)] = &[
    ("Lesser", 1.00),
    ("Regular", 0.75),
    ("Greater", 0.50),
    ("Deadly", 0.15),
];
const CURE_AOS: &[(&str, f64)] = &[
    ("Lesser", 1.00),
    ("Regular", 0.95),
    ("Greater", 0.75),
    ("Deadly", 0.50),
    ("Lethal", 0.25),
];
const GREATER_OLD: &[(&str, f64)] = &[
    ("Lesser", 1.00),
    ("Regular", 1.00),
    ("Greater", 1.00),
    ("Deadly", 0.75),
    ("Lethal", 0.25),
];
const GREATER_AOS: &[(&str, f64)] = &[
    ("Lesser", 1.00),
    ("Regular", 1.00),
    ("Greater", 1.00),
    ("Deadly", 0.95),
    ("Lethal", 0.75),
];

potions! {
    INVISIBILITY_POTION, new_invisibility = "Server.Items.InvisibilityPotion" : BASE_POTION, gid 0xF0A, effect fx::INVISIBILITY, hue 0x48D, label 1072941, Kind::Invisibility;
    NIGHT_SIGHT_POTION, new_night_sight = "Server.Items.NightSightPotion" : BASE_POTION, gid 0xF06, effect fx::NIGHTSIGHT, hue 0, label 0, Kind::NightSight;
    LESSER_HEAL_POTION, new_lesser_heal = "Server.Items.LesserHealPotion" : BASE_HEAL_POTION, gid 0xF0C, effect fx::HEAL_LESSER, hue 0, label 0, Kind::Heal { aos: (6, 8, 3.0), pre: (3, 10, 10.0) };
    HEAL_POTION, new_heal = "Server.Items.HealPotion" : BASE_HEAL_POTION, gid 0xF0C, effect fx::HEAL, hue 0, label 0, Kind::Heal { aos: (13, 16, 8.0), pre: (6, 20, 10.0) };
    GREATER_HEAL_POTION, new_greater_heal = "Server.Items.GreaterHealPotion" : BASE_HEAL_POTION, gid 0xF0C, effect fx::HEAL_GREATER, hue 0, label 0, Kind::Heal { aos: (20, 25, 10.0), pre: (9, 30, 10.0) };
    LESSER_CURE_POTION, new_lesser_cure = "Server.Items.LesserCurePotion" : BASE_CURE_POTION, gid 0xF07, effect fx::CURE_LESSER, hue 0, label 0, Kind::Cure { aos: LESSER_AOS, pre: LESSER_OLD };
    CURE_POTION, new_cure = "Server.Items.CurePotion" : BASE_CURE_POTION, gid 0xF07, effect fx::CURE, hue 0, label 0, Kind::Cure { aos: CURE_AOS, pre: CURE_OLD };
    GREATER_CURE_POTION, new_greater_cure = "Server.Items.GreaterCurePotion" : BASE_CURE_POTION, gid 0xF07, effect fx::CURE_GREATER, hue 0, label 0, Kind::Cure { aos: GREATER_AOS, pre: GREATER_OLD };
    REFRESH_POTION, new_refresh = "Server.Items.RefreshPotion" : BASE_REFRESH_POTION, gid 0xF0B, effect fx::REFRESH, hue 0, label 0, Kind::Refresh(0.25);
    TOTAL_REFRESH_POTION, new_total_refresh = "Server.Items.TotalRefreshPotion" : BASE_REFRESH_POTION, gid 0xF0B, effect fx::REFRESH_TOTAL, hue 0, label 0, Kind::Refresh(1.0);
    AGILITY_POTION, new_agility = "Server.Items.AgilityPotion" : BASE_AGILITY_POTION, gid 0xF08, effect fx::AGILITY, hue 0, label 0, Kind::Agility(10);
    GREATER_AGILITY_POTION, new_greater_agility = "Server.Items.GreaterAgilityPotion" : BASE_AGILITY_POTION, gid 0xF08, effect fx::AGILITY_GREATER, hue 0, label 0, Kind::Agility(20);
    STRENGTH_POTION, new_strength = "Server.Items.StrengthPotion" : BASE_STRENGTH_POTION, gid 0xF09, effect fx::STRENGTH, hue 0, label 0, Kind::Strength(10);
    GREATER_STRENGTH_POTION, new_greater_strength = "Server.Items.GreaterStrengthPotion" : BASE_STRENGTH_POTION, gid 0xF09, effect fx::STRENGTH_GREATER, hue 0, label 0, Kind::Strength(20);
    LESSER_POISON_POTION, new_lesser_poison = "Server.Items.LesserPoisonPotion" : BASE_POISON_POTION, gid 0xF0A, effect fx::POISON_LESSER, hue 0, label 0, Kind::Poison("Lesser", 0.0, 60.0);
    POISON_POTION, new_poison = "Server.Items.PoisonPotion" : BASE_POISON_POTION, gid 0xF0A, effect fx::POISON, hue 0, label 0, Kind::Poison("Regular", 30.0, 70.0);
    GREATER_POISON_POTION, new_greater_poison = "Server.Items.GreaterPoisonPotion" : BASE_POISON_POTION, gid 0xF0A, effect fx::POISON_GREATER, hue 0, label 0, Kind::Poison("Greater", 60.0, 100.0);
    DEADLY_POISON_POTION, new_deadly_poison = "Server.Items.DeadlyPoisonPotion" : BASE_POISON_POTION, gid 0xF0A, effect fx::POISON_DEADLY, hue 0, label 0, Kind::Poison("Deadly", 95.0, 100.0);
    DARKGLOW_POTION, new_darkglow = "Server.Items.DarkglowPotion" : BASE_POISON_POTION, gid 0xF0A, effect fx::DARKGLOW, hue 0x96, label 1072849, Kind::Poison("GreaterDarkglow", 95.0, 100.0);
    PARASITIC_POTION, new_parasitic = "Server.Items.ParasiticPotion" : BASE_POISON_POTION, gid 0xF0A, effect fx::PARASITIC, hue 0x17C, label 1072848, Kind::Poison("DeadlyParasitic", 95.0, 100.0);
    LESSER_EXPLOSION_POTION, new_lesser_explosion = "Server.Items.LesserExplosionPotion" : BASE_EXPLOSION_POTION, gid 0xF0D, effect fx::EXPLOSION_LESSER, hue 0, label 0, Kind::Explosion { aos: (5, 10), pre: (5, 10) };
    EXPLOSION_POTION, new_explosion = "Server.Items.ExplosionPotion" : BASE_EXPLOSION_POTION, gid 0xF0D, effect fx::EXPLOSION, hue 0, label 0, Kind::Explosion { aos: (10, 20), pre: (10, 20) };
    GREATER_EXPLOSION_POTION, new_greater_explosion = "Server.Items.GreaterExplosionPotion" : BASE_EXPLOSION_POTION, gid 0xF0D, effect fx::EXPLOSION_GREATER, hue 0, label 0, Kind::Explosion { aos: (20, 40), pre: (15, 30) };
    CONFLAGRATION_POTION, new_conflagration = "Server.Items.ConflagrationPotion" : BASE_CONFLAGRATION_POTION, gid 0xF06, effect fx::CONFLAGRATION, hue 0x489, label 1072095, Kind::Conflagration(2, 4);
    GREATER_CONFLAGRATION_POTION, new_greater_conflagration = "Server.Items.GreaterConflagrationPotion" : BASE_CONFLAGRATION_POTION, gid 0xF06, effect fx::CONFLAGRATION_GREATER, hue 0x489, label 1072098, Kind::Conflagration(4, 8);
    CONFUSION_BLAST_POTION, new_confusion_blast = "Server.Items.ConfusionBlastPotion" : BASE_CONFUSION_BLAST_POTION, gid 0xF06, effect fx::CONFUSION_BLAST, hue 0x48D, label 1072105, Kind::Confusion(5);
    GREATER_CONFUSION_BLAST_POTION, new_greater_confusion_blast = "Server.Items.GreaterConfusionBlastPotion" : BASE_CONFUSION_BLAST_POTION, gid 0xF06, effect fx::CONFUSION_BLAST_GREATER, hue 0x48D, label 1072108, Kind::Confusion(7);
}

/// Every concrete potion class.
pub static ALL: &[&ItemType] = &[
    &INVISIBILITY_POTION,
    &NIGHT_SIGHT_POTION,
    &LESSER_HEAL_POTION,
    &HEAL_POTION,
    &GREATER_HEAL_POTION,
    &LESSER_CURE_POTION,
    &CURE_POTION,
    &GREATER_CURE_POTION,
    &REFRESH_POTION,
    &TOTAL_REFRESH_POTION,
    &AGILITY_POTION,
    &GREATER_AGILITY_POTION,
    &STRENGTH_POTION,
    &GREATER_STRENGTH_POTION,
    &LESSER_POISON_POTION,
    &POISON_POTION,
    &GREATER_POISON_POTION,
    &DEADLY_POISON_POTION,
    &DARKGLOW_POTION,
    &PARASITIC_POTION,
    &LESSER_EXPLOSION_POTION,
    &EXPLOSION_POTION,
    &GREATER_EXPLOSION_POTION,
    &CONFLAGRATION_POTION,
    &GREATER_CONFLAGRATION_POTION,
    &CONFUSION_BLAST_POTION,
    &GREATER_CONFUSION_BLAST_POTION,
];

static ABSTRACT: &[&ItemType] = &[
    &BASE_POTION,
    &BASE_HEAL_POTION,
    &BASE_CURE_POTION,
    &BASE_REFRESH_POTION,
    &BASE_AGILITY_POTION,
    &BASE_STRENGTH_POTION,
    &BASE_POISON_POTION,
    &BASE_EXPLOSION_POTION,
    &BASE_CONFLAGRATION_POTION,
    &BASE_CONFUSION_BLAST_POTION,
];

// ---- hooks ----

/// `BasePotion.OnCraft(..)` (alchemy: a potion goes into a keg of its kind when the crafter has one).
fn craftable(w: &mut World, id: ItemId, a: &CraftArgs) -> Option<i32> {
    if !is_potion(w, id) {
        return None;
    }
    let from = a.from;
    let pack = w
        .mobile(from)
        .and_then(|m| m.backpack)
        .filter(|_| a.craft_system == "Alchemy"); // craftSystem is DefAlchemy
    if let Some(pack) = pack {
        if effect(w, id).unwrap_or(0) >= fx::INVISIBILITY {
            return Some(1);
        }
        let kegs = enumerable::find_items_by_type(
            w,
            pack,
            true,
            &super::potion_keg::POTION_KEG,
            |_, _| true,
        );
        for keg in kegs {
            let held = super::potion_keg::held(w, keg);
            if held <= 0 || held >= 100 {
                continue;
            }
            if super::potion_keg::keg_type(w, keg) != effect(w, id).unwrap_or(0) {
                continue;
            }
            super::potion_keg::set_held(w, keg, held + 1);
            base::consume(w, id, 1);
            let bottle = new_bottle(w, 1);
            muo_server::mobile::part2::items::add_to_backpack(w, from, bottle);
            return Some(-1); // signal placed in keg
        }
    }
    Some(1)
}

fn poison_potion(w: &World, item: ItemId) -> Option<PoisonPotionFacts> {
    let c = class(w, item);
    let Kind::Poison(name, min, max) = c.kind else {
        return None;
    };
    is_potion(w, item).then_some(())?;
    Some(PoisonPotionFacts {
        poison: poison_named(w, name)?,
        min_skill: min,
        max_skill: max,
    })
}

fn configure(w: &mut World) {
    crate::commodity::register(w, &BASE_POTION, muo_content::core::ml);
    muo_content::craft::hooks_mut(w).craftables.push(craftable);
    {
        let f = &mut w.resources.get_mut::<muo_content::skills::SkillHooks>().fns;
        f.potion_label = Some(|w, item| {
            let ty = w.item(item).map(|i| i.ty).filter(|t| t.is(&BASE_POTION))?;
            Some((ty.label_number)(w, item))
        });
        f.poison_potion = Some(poison_potion);
        f.is_blank_scroll = Some(|w, item| {
            w.item(item)
                .is_some_and(|i| i.ty.is(&super::plain::BLANK_SCROLL))
        });
    }
    super::potion_throw::configure(w);
}

/// Registers the potion classes.
pub fn register(types: &mut TypeRegistry) {
    for t in ABSTRACT {
        types.register_item(t, &[]);
    }
    for t in ALL {
        types.register_item(t, &[]);
    }
    types.register_phase_default(
        Phase::Configure,
        "Server.Items.BasePotion.Configure",
        configure,
    );
}
