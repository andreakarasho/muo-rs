//! Vendor classes with logic of their own (not tables, see `npc.rs`): the bulk order
//! vendors, outfits that edit the worn items, and the classes with their own members.

use muo_content::hooks::rnd;
use muo_content::player_mobile::{self};
use muo_content::vendor::VendorClass;
use muo_primitives::TimeSpan;
use muo_server::ids::{ItemId, MobileId};
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

use super::npc::{TEMPLATE_VENDOR_V, new_template_vendor};
use crate::common::*;

// ---- shared pieces ----

/// `from.Skills.X.Base`.
pub fn skill_base_ref(w: &World, id: MobileId, skill: SkillName) -> f64 {
    w.mobile(id)
        .and_then(|m| m.skills.as_ref())
        .and_then(|s| s.peek(skill as usize))
        .map_or(0.0, |s| f64::from(s.base) / 10.0)
}

/// `from is PlayerMobile`.
pub fn is_player(w: &World, m: MobileId) -> bool {
    player_mobile::data(w, m).is_some()
}

// ---- bulk order vendors (Blacksmith, Weaponsmith, Tailor, Weaver) ----

struct Bod {
    skill: SkillName,
    /// `SmallXBOD.CreateRandomFor(from)` is the `create_small_bod` hook with this kind (0 smith, 1 tailor).
    kind: i32,
    large: &'static str,
    small: &'static str,
    get: fn(&World, MobileId) -> TimeSpan,
    set: fn(&mut World, MobileId, TimeSpan),
}

static SMITH: Bod = Bod {
    skill: SkillName::Blacksmith,
    kind: 0,
    large: "Server.Engines.BulkOrders.LargeSmithBOD",
    small: "Server.Engines.BulkOrders.SmallSmithBOD",
    get: player_mobile::next_smith_bulk_order,
    set: player_mobile::set_next_smith_bulk_order,
};
static TAILOR: Bod = Bod {
    skill: SkillName::Tailoring,
    kind: 1,
    large: "Server.Engines.BulkOrders.LargeTailorBOD",
    small: "Server.Engines.BulkOrders.SmallTailorBOD",
    get: player_mobile::next_tailor_bulk_order,
    set: player_mobile::set_next_tailor_bulk_order,
};

/// `CreateBulkOrder(from, fromContextMenu)`.
fn create_bulk_order(
    w: &mut World,
    b: &Bod,
    from: MobileId,
    from_context_menu: bool,
) -> Option<ItemId> {
    if is_player(w, from)
        && (b.get)(w, from) == TimeSpan::ZERO
        && (from_context_menu || rnd::double(w) < 0.2)
    {
        let their_skill = muo_server::skills::base(w, from, b.skill);

        if their_skill >= 70.1 {
            (b.set)(w, from, TimeSpan::from_hours(6));
        } else if their_skill >= 50.1 {
            (b.set)(w, from, TimeSpan::from_hours(2));
        } else {
            (b.set)(w, from, TimeSpan::from_hours(1));
        }

        if their_skill >= 70.1 && (their_skill - 40.0) / 300.0 > rnd::double(w) {
            return muo_content::factory::make_item(w, b.large);
        }
        return muo_content::vendor::hooks(w)
            .and_then(|h| h.create_small_bod)
            .and_then(|f| f(w, b.kind, from));
    }
    None
}

macro_rules! bod_fns {
    ($m:ident, $bod:ident) => {
        mod $m {
            use super::*;
            pub fn create(w: &mut World, _v: MobileId, from: MobileId, cm: bool) -> Option<ItemId> {
                create_bulk_order(w, &$bod, from, cm)
            }
            pub fn is_valid(w: &World, _v: MobileId, item: ItemId) -> bool {
                muo_content::factory::item_is(w, item, $bod.small)
                    || muo_content::factory::item_is(w, item, $bod.large)
            }
            pub fn next(w: &mut World, _v: MobileId, from: MobileId) -> TimeSpan {
                if is_player(w, from) {
                    ($bod.get)(w, from)
                } else {
                    TimeSpan::ZERO
                }
            }
            pub fn received(w: &mut World, _v: MobileId, from: MobileId) {
                if w.core_se() && is_player(w, from) {
                    ($bod.set)(w, from, TimeSpan::ZERO);
                }
            }
        }
    };
}
bod_fns!(smith_bod, SMITH);
bod_fns!(tailor_bod, TAILOR);

pub static BLACKSMITH_V: VendorClass = VendorClass {
    supports_bulk_orders: |w, _v, from| {
        is_player(w, from) && skill_base_ref(w, from, SkillName::Blacksmith) > 0.0
    },
    create_bulk_order: smith_bod::create,
    is_valid_bulk_order: smith_bod::is_valid,
    get_next_bulk_order: smith_bod::next,
    on_successful_bulk_order_receive: smith_bod::received,
    ..TEMPLATE_VENDOR_V
};
vendor_leaf!(
    BLACKSMITH,
    BLACKSMITH_C,
    BLACKSMITH_V = "Server.Mobiles.Blacksmith",
    ctor | w | new_template_vendor(w, &BLACKSMITH)
);

static WEAPONSMITH_V: VendorClass = VendorClass {
    supports_bulk_orders: |w, _v, from| {
        is_player(w, from) && w.core_aos() && skill_base_ref(w, from, SkillName::Blacksmith) > 0.0
    },
    create_bulk_order: smith_bod::create,
    is_valid_bulk_order: smith_bod::is_valid,
    get_next_bulk_order: smith_bod::next,
    on_successful_bulk_order_receive: smith_bod::received,
    ..TEMPLATE_VENDOR_V
};
vendor_leaf!(
    WEAPONSMITH,
    WEAPONSMITH_C,
    WEAPONSMITH_V = "Server.Mobiles.Weaponsmith",
    ctor | w | new_template_vendor(w, &WEAPONSMITH)
);

static TAILOR_V: VendorClass = VendorClass {
    supports_bulk_orders: |w, _v, from| {
        is_player(w, from) && skill_base_ref(w, from, SkillName::Tailoring) > 0.0
    },
    create_bulk_order: tailor_bod::create,
    is_valid_bulk_order: tailor_bod::is_valid,
    get_next_bulk_order: tailor_bod::next,
    on_successful_bulk_order_receive: tailor_bod::received,
    ..TEMPLATE_VENDOR_V
};
vendor_leaf!(
    TAILOR_NPC,
    TAILOR_C,
    TAILOR_V = "Server.Mobiles.Tailor",
    ctor | w | new_template_vendor(w, &TAILOR_NPC)
);

static WEAVER_V: VendorClass = VendorClass {
    supports_bulk_orders: |w, _v, from| {
        is_player(w, from) && skill_base_ref(w, from, SkillName::Tailoring) > 0.0
    },
    create_bulk_order: tailor_bod::create,
    is_valid_bulk_order: tailor_bod::is_valid,
    get_next_bulk_order: tailor_bod::next,
    on_successful_bulk_order_receive: tailor_bod::received,
    ..TEMPLATE_VENDOR_V
};
vendor_leaf!(
    WEAVER,
    WEAVER_C,
    WEAVER_V = "Server.Mobiles.Weaver",
    ctor | w | new_template_vendor(w, &WEAVER)
);

// ---- outfits that edit the worn items (their `npcOutfit` props) ----

static IRON_WORKER_V: VendorClass = VendorClass {
    ..TEMPLATE_VENDOR_V
};
vendor_leaf!(
    IRON_WORKER,
    IRON_WORKER_C,
    IRON_WORKER_V = "Server.Mobiles.IronWorker",
    ctor | w | new_template_vendor(w, &IRON_WORKER)
);

static VAGABOND_V: VendorClass = VendorClass {
    ..TEMPLATE_VENDOR_V
};
vendor_leaf!(
    VAGABOND,
    VAGABOND_C,
    VAGABOND_V = "Server.Mobiles.Vagabond",
    ctor | w | new_template_vendor(w, &VAGABOND)
);

pub static KEEPER_OF_CHIVALRY_V: VendorClass = VendorClass {
    ..TEMPLATE_VENDOR_V
};
vendor_leaf!(
    KEEPER_OF_CHIVALRY,
    KEEPER_OF_CHIVALRY_C,
    KEEPER_OF_CHIVALRY_V = "Server.Mobiles.KeeperOfChivalry",
    ctor | w | new_template_vendor(w, &KEEPER_OF_CHIVALRY)
);

static GYPSY_MAIDEN_V: VendorClass = VendorClass {
    get_gender: |_, _| true,
    ..TEMPLATE_VENDOR_V
};
vendor_leaf!(
    GYPSY_MAIDEN,
    GYPSY_MAIDEN_C,
    GYPSY_MAIDEN_V = "Server.Mobiles.GypsyMaiden",
    ctor | w | new_template_vendor(w, &GYPSY_MAIDEN)
);

static HOLY_MAGE_V: VendorClass = VendorClass {
    ..TEMPLATE_VENDOR_V
};
vendor_leaf!(
    HOLY_MAGE,
    HOLY_MAGE_C,
    HOLY_MAGE_V = "Server.Mobiles.HolyMage",
    ctor | w | new_template_vendor(w, &HOLY_MAGE)
);

// ---- Scribe ----

/// `m_NextShush`.
struct NextShush(muo_primitives::DateTime);

fn scribe_on_speech(w: &mut World, id: MobileId, e: &mut muo_server::events::SpeechEventArgs) {
    (muo_content::vendor::BASE_VENDOR.on_speech)(w, id, e);
    let next = w
        .mobile(id)
        .and_then(|m| m.ext.get::<NextShush>())
        .map_or(muo_primitives::DateTime::MIN, |n| n.0);
    if !e.handled && next <= w.now && muo_content::mx::in_los(w, id, e.mobile) {
        let d = muo_content::mx::get_direction_to(w, id, e.mobile);
        let female = w.mobile(id).is_some_and(|m| m.female);
        muo_server::mobile::part1::props_b::set_direction(w, id, d);
        muo_content::mx::play_sound(w, id, if female { 0x32F } else { 0x441 });
        muo_server::mobile::part2::messages::public_overhead_message_number(
            w,
            id,
            muo_primitives::MessageType::REGULAR,
            0x3B2,
            1073990, // Shhhh!
            "",
            false,
        );
        let t = w.now + TimeSpan::from_mins(1);
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(NextShush(t));
        }
        e.handled = true;
    }
}

static SCRIBE_V: VendorClass = VendorClass {
    ..TEMPLATE_VENDOR_V
};
vendor_leaf!(SCRIBE, SCRIBE_C, SCRIBE_V = "Server.Mobiles.Scribe", ctor |w| new_template_vendor(w, &SCRIBE);
    m {
        handles_on_speech: |w, _id, from| w.mobile(from).is_some_and(|m| m.player),
        on_speech: scribe_on_speech,
    }
);

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in [
        &BLACKSMITH,
        &WEAPONSMITH,
        &TAILOR_NPC,
        &WEAVER,
        &IRON_WORKER,
        &VAGABOND,
        &KEEPER_OF_CHIVALRY,
        &GYPSY_MAIDEN,
        &HOLY_MAGE,
        &SCRIBE,
    ] {
        types.register_mobile(t, &[]);
    }
}

#[cfg(test)]
pub(crate) fn all() -> Vec<&'static muo_server::mobile::MobileType> {
    vec![
        &BLACKSMITH,
        &WEAPONSMITH,
        &TAILOR_NPC,
        &WEAVER,
        &IRON_WORKER,
        &VAGABOND,
        &KEEPER_OF_CHIVALRY,
        &GYPSY_MAIDEN,
        &HOLY_MAGE,
        &SCRIBE,
    ]
}
