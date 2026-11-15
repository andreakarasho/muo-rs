//! `BaseShieldGuard`, `ChaosGuard` and `OrderGuard`: the guards that give a guild member the
//! faction shield. All three are `[SerializationGenerator(0, false)]` (parent part + version int 0).

use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C};
use muo_content::hooks::rnd;
use muo_content::{factory, mx};
use muo_io::{GenericReader, GenericWriter};
use muo_server::World;
use muo_server::guild::GuildType;
use muo_server::ids::MobileId;
use muo_server::layer::Layer;
use muo_server::mobile::MobileType;
use muo_server::mobile::part1::{props_a, props_b};
use muo_server::mobile::part2::{items, messages};
use muo_server::type_registry::TypeRegistry;

/// The abstract members of `BaseShieldGuard`.
struct ShieldGuard {
    keyword: i32,
    shield: &'static str,
    signup_number: i32,
    ty: GuildType,
}

const CHAOS: ShieldGuard = ShieldGuard {
    keyword: 0x22, // *chaos shield*
    shield: "Server.Items.ChaosShield",
    signup_number: 1007140, // Sign up with a guild of chaos if thou art interested.
    ty: GuildType::Chaos,
};

const ORDER: ShieldGuard = ShieldGuard {
    keyword: 0x21, // *order shield*
    shield: "Server.Items.OrderShield",
    signup_number: 1007141, // Sign up with a guild of order if thou art interested.
    ty: GuildType::Order,
};

fn guard_of(w: &World, id: MobileId) -> &'static ShieldGuard {
    if factory::mobile_is(w, id, "Server.Mobiles.ChaosGuard") {
        &CHAOS
    } else {
        &ORDER
    }
}

fn chaos_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &CHAOS_GUARD)
}

fn order_new(w: &mut World) -> MobileId {
    muo_content::templates::new_creature(w, &ORDER_GUARD)
}

fn handles_on_speech(w: &World, id: MobileId, from: MobileId) -> bool {
    mx::in_range_point(w, from, mx::location(w, id), 2)
        || (BASE_CREATURE.handles_on_speech)(w, id, from)
}

fn on_speech(w: &mut World, id: MobileId, e: &mut muo_server::events::SpeechEventArgs) {
    let g = guard_of(w, id);
    if !e.handled
        && e.has_keyword(g.keyword)
        && mx::in_range_point(w, e.mobile, mx::location(w, id), 2)
    {
        e.handled = true;
        let from = e.mobile;
        let guild_type = w
            .mobile(from)
            .and_then(|m| m.guild)
            .map(|gid| muo_content::misc::guild_class::guild_type(w, gid));
        // `from.Guild is not Guild g || g.Type != Type`
        if guild_type != Some(g.ty) {
            messages::say_number(w, id, g.signup_number, "");
        } else {
            let pack = props_b::backpack(w, from);
            let shield = factory::make_item(w, g.shield);
            let two_handed = items::find_item_on_layer(w, from, Layer::TwoHanded);
            let has_one = pack
                .is_some_and(|p| factory::find_item_by_type(w, p, g.shield).is_some())
                || two_handed.is_some_and(|t| factory::item_is(w, t, g.shield));
            if has_one {
                messages::say_number(w, id, 1007110, ""); // Why dost thou ask about virtue guards when thou art one?
                if let Some(s) = shield {
                    muo_server::item::base::delete(w, s);
                }
            } else if shield.is_some_and(|s| items::place_in_backpack(w, from, s)) {
                let n = rnd::from(w, 1007101, 5);
                messages::say_number(w, id, n, "");
                messages::say_number(w, id, 1007139, ""); // I see you are in need of our shield, Here you go.
                if let Some(s) = shield {
                    items::add_to_backpack(w, from, s);
                }
            } else {
                mx::msg(w, from, 502868); // Your backpack is too full.
                if let Some(s) = shield {
                    muo_server::item::base::delete(w, s);
                }
            }
        }
    }
    (BASE_CREATURE.on_speech)(w, id, e);
}

/// `BaseShieldGuard` (abstract): the speech handling.
pub static BASE_SHIELD_GUARD: MobileType = MobileType {
    full_name: "Server.Mobiles.BaseShieldGuard",
    parent: Some(&BASE_CREATURE),
    construct: None,
    family: Some(&BASE_CREATURE_C),
    serialize: |w, id, out: &mut dyn GenericWriter| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r: &mut dyn GenericReader| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        Ok(())
    },
    handles_on_speech,
    on_speech,
    ..BASE_CREATURE
};

creature_leaf! {
    /// `Server.Mobiles.ChaosGuard`.
    CHAOS_GUARD, CHAOS_GUARD_C = "Server.Mobiles.ChaosGuard" : BASE_SHIELD_GUARD / BASE_CREATURE_C, ctor Some(chaos_new), encoded false;
    c {
        bard_immune: |_w, _id| true,
    }
    m {
    }
}

creature_leaf! {
    /// `Server.Mobiles.OrderGuard`.
    ORDER_GUARD, ORDER_GUARD_C = "Server.Mobiles.OrderGuard" : BASE_SHIELD_GUARD / BASE_CREATURE_C, ctor Some(order_new), encoded false;
    c {
        bard_immune: |_w, _id| true,
    }
    m {
    }
}

pub(crate) fn all() -> Vec<&'static MobileType> {
    vec![&CHAOS_GUARD, &ORDER_GUARD]
}

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in all() {
        types.register_mobile(t, &[]);
    }
}

#[allow(dead_code)]
fn _unused(w: &mut World, id: MobileId) {
    props_a::set_title(w, id, None);
}
