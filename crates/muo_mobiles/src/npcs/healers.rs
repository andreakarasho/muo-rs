//! Healers: `BaseHealer` (a `BaseVendor` that resurrects, version 3 with a price) and its six
//! classes.

use muo_content::creature;
use muo_content::creature::enums::{AIType, FightMode};
use muo_content::hooks::rnd;
use muo_content::vendor::{
    self, BASE_VENDOR, BASE_VENDOR_C, BASE_VENDOR_CLASS, VendorClass, new_vendor,
};
use muo_primitives::{DateTime, Point3D, TimeSpan, utility};
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::MobileType;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

use crate::common::*;

/// What `BaseHealer` adds to a `BaseVendor`.
pub struct HealerData {
    /// `_price`.
    pub price: i32,
    /// `_nextResurrect`.
    pub next_resurrect: DateTime,
}

pub fn data(w: &World, id: MobileId) -> Option<&HealerData> {
    w.mobile(id)?.ext.get::<HealerData>()
}

fn data_mut(w: &mut World, id: MobileId) -> Option<&mut HealerData> {
    w.mobile_mut(id)?.ext.get_mut::<HealerData>()
}

/// `Price = value` (`BaseHealer.Price`).
pub fn set_price(w: &mut World, id: MobileId, value: i32) {
    if let Some(d) = data_mut(w, id) {
        d.price = value;
    }
}

/// The virtuals `BaseHealer` adds.
pub struct HealerClass {
    /// `HealsYoungPlayers`.
    pub heals_young_players: fn(&World, MobileId) -> bool,
    /// `GetRobeColor()`.
    pub get_robe_color: fn(&mut World, MobileId) -> i32,
    /// `CheckResurrect(m)`.
    pub check_resurrect: fn(&mut World, MobileId, MobileId) -> bool,
}

fn hclass(w: &World, id: MobileId) -> &'static HealerClass {
    vendor::subclass::<HealerClass>(w, id).unwrap_or(&BASE_HEALER_CLASS)
}

pub static BASE_HEALER_CLASS: HealerClass = HealerClass {
    heals_young_players: |_, _| true,
    get_robe_color: |w, _| utility::random_yellow_hue(&mut w.rng),
    check_resurrect: |_, _, _| true,
};

/// `AI = ai` (the property setter).
pub fn set_ai(w: &mut World, id: MobileId, ai: AIType) {
    creature::edit(w, id, |d| {
        d.current_ai = if ai == AIType::AiUseDefault {
            d.default_ai
        } else {
            ai
        };
    });
    let cur = creature::read(w, id, ai, |d| d.current_ai);
    muo_content::ai::change_ai_type(w, id, cur);
}

fn is_invulnerable(w: &World, id: MobileId) -> bool {
    creature::ctype(w, id).is_some_and(|c| (c.is_invulnerable)(w, id))
}

/// The `!IsInvulnerable` block of the constructor and of `AfterDeserialization`.
fn mage_ai(w: &mut World, id: MobileId) {
    if !is_invulnerable(w, id) {
        set_ai(w, id, AIType::AiMage);
        muo_content::creature::speeds::set_speed(w, id, 0.2, 0.8, true);
        creature::edit(w, id, |d| {
            d.range_perception = DEFAULT_RANGE_PERCEPTION;
            d.fight_mode = FightMode::Aggressor;
        });
    }
}

/// `BaseHealer(price)` after `BaseVendor(null)`: the class data and the AI; its statements are the
/// `Server.Mobiles.BaseHealer` template every healer template `inherits`.
fn base_healer_init(w: &mut World, id: MobileId, price: i32) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(HealerData {
            price,
            next_resurrect: DateTime::MIN,
        });
    }
    mage_ai(w, id);
}

/// `OfferResurrection(m)`.
fn offer_resurrection(w: &mut World, healer: MobileId, m: MobileId) {
    let d = muo_content::mx::get_direction_to(w, healer, m);
    muo_server::mobile::part1::props_b::set_direction(w, healer, d);
    muo_content::mx::play_sound(w, m, 0x1F2);
    muo_content::mx::fixed_effect(w, m, 0x376A, 10, 16);
    let price = data(w, healer).map_or(0, |d| d.price);
    if let Some(f) = vendor::hooks(w).and_then(|h| h.resurrect_gump) {
        f(w, m, healer, price);
    }
}

/// `OfferHeal(m)`.
fn offer_heal(w: &mut World, healer: MobileId, m: MobileId) {
    let d = muo_content::mx::get_direction_to(w, healer, m);
    muo_server::mobile::part1::props_b::set_direction(w, healer, d);
    if muo_content::player_mobile::check_young_heal_time(w, m) {
        muo_server::mobile::part2::messages::say_number(w, healer, 501229, ""); // You look like you need some healing my child.
        muo_content::mx::play_sound(w, m, 0x1F2);
        muo_content::mx::fixed_effect(w, m, 0x376A, 9, 32);
        let max = muo_content::mx::hits_max(w, m);
        muo_server::mobile::part1::props_b::set_hits(w, m, max);
    } else {
        muo_server::mobile::part2::messages::say_number(w, healer, 501228, ""); // I can do no more for you at this time.
    }
}

/// `OnMovement(m, oldLocation)`.
fn on_movement(w: &mut World, healer: MobileId, m: MobileId, old: Point3D) {
    let frozen = w.mobile(m).is_some_and(|x| x.frozen);
    let next = data(w, healer).map_or(DateTime::MIN, |d| d.next_resurrect);
    if !frozen
        && w.now >= next
        && muo_content::mx::in_range(w, healer, m, 4)
        && !muo_content::mx::in_range_point(w, healer, old, 4)
        && muo_content::mx::in_los(w, healer, m)
    {
        if !muo_content::mx::alive(w, m) {
            let now = w.now;
            if let Some(d) = data_mut(w, healer) {
                d.next_resurrect = now + TimeSpan::from_secs(2);
            }
            let (loc, map) = w
                .mobile(m)
                .map_or_else(Default::default, |x| (x.location, x.map));
            let fits = map.is_some_and(|map| {
                muo_server::map::fit::can_fit(w, map, loc.x, loc.y, loc.z, 16, false, false, true)
            });
            if !fits {
                muo_content::mx::msg(w, m, 502391); // Thou can not be resurrected there!
            } else if (hclass(w, healer).check_resurrect)(w, healer, m) {
                offer_resurrection(w, healer, m);
            }
        } else if (hclass(w, healer).heals_young_players)(w, healer)
            && muo_content::mx::hits(w, m) < muo_content::mx::hits_max(w, m)
            && muo_content::player_mobile::data(w, m).is_some()
            && muo_content::player_mobile::young(w, m)
        {
            offer_heal(w, healer, m);
        }
    }
}

static BASE_HEALER_V: VendorClass = VendorClass {
    init_sb_info: super::vendors::sb::from_props,
    is_active_vendor: |_, _| false,
    shoe_type: crate::template_keys_npcs::shoe_type,
    init_outfit: |w, id| {
        vendor::init_outfit(w, id);
        let h = (hclass(w, id).get_robe_color)(w, id);
        vendor::add_hued(w, id, "Server.Items.Robe", h);
    },
    subclass: Some(&BASE_HEALER_CLASS),
    ..BASE_VENDOR_CLASS
};

static BASE_HEALER_C: muo_content::creature::CreatureType = muo_content::creature::CreatureType {
    is_invulnerable: |_, _| false,
    subclass: Some(&BASE_HEALER_V),
    ..BASE_VENDOR_C
};

/// `Server.Mobiles.BaseHealer` (abstract).
pub static BASE_HEALER: MobileType = MobileType {
    full_name: "Server.Mobiles.BaseHealer",
    parent: Some(&BASE_VENDOR),
    construct: None,
    family: Some(&BASE_HEALER_C),
    on_movement,
    serialize: |w, id, out| {
        (BASE_VENDOR.serialize)(w, id, out);
        out.write_i32(3);
        out.write_i32(data(w, id).map_or(0, |d| d.price));
    },
    deserialize: |w, id, r| {
        (BASE_VENDOR.deserialize)(w, id, r)?;
        if let Some(m) = w.mobile_mut(id)
            && m.ext.get::<HealerData>().is_none()
        {
            m.ext.insert(HealerData {
                price: 0,
                next_resurrect: DateTime::MIN,
            });
        }
        let version = r.read_i32()?;
        if version == 2 {
            // MigrateFrom(V2Content): _price = 0
            if let Some(d) = data_mut(w, id) {
                d.price = 0;
            }
        } else if version < 3 {
            // Deserialize(reader, version): a previous RunUO serialization issue; the version was never written.
            r.seek(-4, muo_io::SeekOrigin::Current)?;
        } else {
            let price = r.read_i32()?;
            if let Some(d) = data_mut(w, id) {
                d.price = price;
            }
        }
        mage_ai(w, id);
        Ok(())
    },
    ..BASE_VENDOR
};

// ---- leaf classes ----

/// A `BaseHealer` subclass whose own part is `version int` (nothing else).
macro_rules! healer_leaf {
    ($name:ident, $class:ident, $vclass:ident, $hclass:ident = $full:literal, ver $ver:literal, ctor $ctor:ident;
     c { $($cf:ident : $cv:expr,)* } m { $($mf:ident : $mv:expr,)* }) => {
        static $class: muo_content::creature::CreatureType = muo_content::creature::CreatureType {
            subclass: Some(&$vclass),
            $($cf: $cv,)*
            ..BASE_HEALER_C
        };
        pub static $name: MobileType = MobileType {
            full_name: $full,
            parent: Some(&BASE_HEALER),
            construct: Some($ctor),
            family: Some(&$class),
            serialize: |w, id, out| {
                (BASE_HEALER.serialize)(w, id, out);
                out.write_i32($ver);
            },
            deserialize: |w, id, r| {
                (BASE_HEALER.deserialize)(w, id, r)?;
                let _version = r.read_i32()?;
                Ok(())
            },
            $($mf: $mv,)*
            ..BASE_HEALER
        };
    };
}

fn new_healer_base(w: &mut World, ty: &'static MobileType, price: i32) -> MobileId {
    let id = new_vendor(w, ty, None);
    base_healer_init(w, id, price);
    muo_content::templates::apply_mobile(w, id);
    id
}

/// `CheckTeach(skill, from)` restricted to a skill list.
fn check_teach_in(w: &World, id: MobileId, skill: i32, from: MobileId, list: &[SkillName]) -> bool {
    (BASE_VENDOR_C.check_teach)(w, id, skill, from) && list.iter().any(|s| *s as i32 == skill)
}

fn say(w: &mut World, id: MobileId, number: i32) {
    muo_server::mobile::part2::messages::say_number(w, id, number, "");
}

fn criminal(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| x.criminal)
}

fn healer_check_resurrect(w: &mut World, id: MobileId, m: MobileId) -> bool {
    if criminal(w, m) {
        say(w, id, 501222); // Thou art a criminal.  I shall not resurrect thee.
        return false;
    }
    if muo_server::mobile::base::murderer(w, m) {
        say(w, id, 501223); // Thou'rt not a decent and good person. I shall not resurrect thee.
        return false;
    }
    if w.mobile(m).is_some_and(|x| x.karma < 0) {
        say(w, id, 501224); // Thou hast strayed from the path of virtue, but thou still deservest a second chance.
    }
    true
}

fn evil_check_resurrect(w: &mut World, id: MobileId, m: MobileId) -> bool {
    if w.core_aos() && criminal(w, m) {
        say(w, id, 501222); // Thou art a criminal.  I shall not resurrect thee.
        return false;
    }
    true
}

// Healer
fn healer_new(w: &mut World) -> MobileId {
    new_healer_base(w, &HEALER, 0)
}
static HEALER_H: HealerClass = HealerClass {
    check_resurrect: healer_check_resurrect,
    ..BASE_HEALER_CLASS
};
static HEALER_V: VendorClass = VendorClass {
    is_active_vendor: |_, _| true,
    subclass: Some(&HEALER_H),
    ..BASE_HEALER_V
};
healer_leaf!(HEALER, HEALER_C, HEALER_V, HEALER_H = "Server.Mobiles.Healer", ver 0, ctor healer_new;
    c {
    can_teach: |_, _| true,
    is_invulnerable: |_, _| true,
    check_teach: |w, id, skill, from| check_teach_in(w, id, skill, from,
        &[SkillName::Forensics, SkillName::Healing, SkillName::SpiritSpeak, SkillName::Swords]),}
    m {}
);

// EvilHealer
fn evil_healer_new(w: &mut World) -> MobileId {
    new_healer_base(w, &EVIL_HEALER, 0)
}
static EVIL_HEALER_H: HealerClass = HealerClass {
    check_resurrect: evil_check_resurrect,
    ..BASE_HEALER_CLASS
};
static EVIL_HEALER_V: VendorClass = VendorClass {
    is_active_vendor: |_, _| true,
    subclass: Some(&EVIL_HEALER_H),
    ..BASE_HEALER_V
};
healer_leaf!(EVIL_HEALER, EVIL_HEALER_C, EVIL_HEALER_V, EVIL_HEALER_H = "Server.Mobiles.EvilHealer", ver 0, ctor evil_healer_new;
    c {
    can_teach: |_, _| true,
    always_murderer: |_, _| true,
    check_teach: |w, id, skill, from| check_teach_in(w, id, skill, from,
        &[SkillName::Forensics, SkillName::Healing, SkillName::SpiritSpeak, SkillName::Swords]),}
    m {}
);

// EvilWanderingHealer
fn evil_wandering_healer_new(w: &mut World) -> MobileId {
    new_healer_base(w, &EVIL_WANDERING_HEALER, 0)
}
static EVIL_WANDERING_HEALER_H: HealerClass = HealerClass {
    check_resurrect: evil_check_resurrect,
    ..BASE_HEALER_CLASS
};
static EVIL_WANDERING_HEALER_V: VendorClass = VendorClass {
    subclass: Some(&EVIL_WANDERING_HEALER_H),
    ..BASE_HEALER_V
};
healer_leaf!(EVIL_WANDERING_HEALER, EVIL_WANDERING_HEALER_C, EVIL_WANDERING_HEALER_V, EVIL_WANDERING_HEALER_H = "Server.Mobiles.EvilWanderingHealer", ver 0, ctor evil_wandering_healer_new;
    c {
    can_teach: |_, _| true,
    always_murderer: |_, _| true,
    check_teach: |w, id, skill, from| check_teach_in(w, id, skill, from,
        &[SkillName::Anatomy, SkillName::Camping, SkillName::Forensics, SkillName::Healing, SkillName::SpiritSpeak]),}
    m { on_death: evil_wandering_on_death, click_title: |_, _| false, }
);

// FortuneTeller
fn fortune_teller_new(w: &mut World) -> MobileId {
    new_healer_base(w, &FORTUNE_TELLER, 0)
}
static FORTUNE_TELLER_H: HealerClass = HealerClass {
    get_robe_color: |w, _| utility::random_bright_hue(&mut w.rng),
    ..BASE_HEALER_CLASS
};
static FORTUNE_TELLER_V: VendorClass = VendorClass {
    is_active_vendor: |_, _| true,
    init_outfit: crate::template_keys_npcs::init_outfit,
    subclass: Some(&FORTUNE_TELLER_H),
    ..BASE_HEALER_V
};
healer_leaf!(FORTUNE_TELLER, FORTUNE_TELLER_C, FORTUNE_TELLER_V, FORTUNE_TELLER_H = "Server.Mobiles.FortuneTeller", ver 0, ctor fortune_teller_new;
    c {
    can_teach: |_, _| true,
    is_invulnerable: |_, _| true,
    check_teach: |w, id, skill, from| check_teach_in(w, id, skill, from,
        &[SkillName::Anatomy, SkillName::Healing, SkillName::Forensics, SkillName::SpiritSpeak]),}
    m {}
);

// WanderingHealer
fn wandering_healer_new(w: &mut World) -> MobileId {
    new_healer_base(w, &WANDERING_HEALER, 0)
}
static WANDERING_HEALER_H: HealerClass = HealerClass {
    check_resurrect: healer_check_resurrect,
    ..BASE_HEALER_CLASS
};
static WANDERING_HEALER_V: VendorClass = VendorClass {
    subclass: Some(&WANDERING_HEALER_H),
    ..BASE_HEALER_V
};
healer_leaf!(WANDERING_HEALER, WANDERING_HEALER_C, WANDERING_HEALER_V, WANDERING_HEALER_H = "Server.Mobiles.WanderingHealer", ver 0, ctor wandering_healer_new;
    c {
    can_teach: |_, _| true,
    check_teach: |w, id, skill, from| check_teach_in(w, id, skill, from,
        &[SkillName::Anatomy, SkillName::Camping, SkillName::Forensics, SkillName::Healing, SkillName::SpiritSpeak]),}
    m { click_title: |_, _| false, }
);

// PricedHealer: [Constructible] PricedHealer(int price = 5000)
fn priced_healer_new(w: &mut World) -> MobileId {
    priced_healer_with(w, 5000)
}

/// `new PricedHealer(price)`.
pub fn priced_healer_with(w: &mut World, price: i32) -> MobileId {
    new_healer_base(w, &PRICED_HEALER, price)
}
static PRICED_HEALER_H: HealerClass = HealerClass {
    heals_young_players: |_, _| false,
    ..BASE_HEALER_CLASS
};
static PRICED_HEALER_V: VendorClass = VendorClass {
    subclass: Some(&PRICED_HEALER_H),
    ..BASE_HEALER_V
};
static PRICED_HEALER_C: muo_content::creature::CreatureType = muo_content::creature::CreatureType {
    is_invulnerable: |_, _| true,
    subclass: Some(&PRICED_HEALER_V),
    ..BASE_HEALER_C
};
pub static PRICED_HEALER: MobileType = MobileType {
    full_name: "Server.Mobiles.PricedHealer",
    parent: Some(&BASE_HEALER),
    construct: Some(priced_healer_new),
    family: Some(&PRICED_HEALER_C),
    serialize: |w, id, out| {
        (BASE_HEALER.serialize)(w, id, out);
        out.write_i32(1);
    },
    deserialize: |w, id, r| {
        (BASE_HEALER.deserialize)(w, id, r)?;
        let version = r.read_i32()?;
        if version == 0 {
            // MigrateFrom(V0Content): Price = content.Price
            let price = r.read_i32()?;
            if let Some(d) = data_mut(w, id) {
                d.price = price;
            }
        }
        Ok(())
    },
    ..BASE_HEALER
};

// OnDeath of EvilWanderingHealer: base, then a map fragment in half of the corpses.
fn evil_wandering_on_death(w: &mut World, id: MobileId, c: Option<ItemId>) {
    (BASE_HEALER.on_death)(w, id, c);
    if rnd::bool(w)
        && let (Some(c), Some(f)) = (
            c,
            muo_content::factory::make_item(w, "Server.Items.FragmentOfAMap"),
        )
    {
        muo_server::item::container::drop_item_default(w, c, f);
    }
}

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in [
        &HEALER,
        &EVIL_HEALER,
        &EVIL_WANDERING_HEALER,
        &FORTUNE_TELLER,
        &WANDERING_HEALER,
        &PRICED_HEALER,
    ] {
        types.register_mobile(t, &[]);
    }
}

#[cfg(test)]
pub(crate) fn all() -> Vec<&'static MobileType> {
    vec![
        &HEALER,
        &EVIL_HEALER,
        &EVIL_WANDERING_HEALER,
        &FORTUNE_TELLER,
        &WANDERING_HEALER,
        &PRICED_HEALER,
    ]
}
