//! Helpers the skill handlers share: targets, messages, class tests.

use muo_primitives::MessageType;
use muo_server::World;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::part2::messages as mm;
use muo_server::target::{self, Target, TargetCancelType, TargetFlags, Targeted};

/// `SendLocalizedMessage(number)`.
pub fn msg(w: &mut World, to: MobileId, number: i32) {
    mm::send_localized_message(w, to, number, "", 0x3B2);
}

/// `SendLocalizedMessage(number, args)`.
pub fn msg_args(w: &mut World, to: MobileId, number: i32, args: &str) {
    mm::send_localized_message(w, to, number, args, 0x3B2);
}

/// `m.PrivateOverheadMessage(MessageType.Regular, 0x3B2, number, to.NetState)`.
pub fn private_msg(w: &mut World, speaker: MobileId, to: MobileId, number: i32) {
    let ns = w.mobile(to).and_then(|m| m.net_state);
    mm::private_overhead_message_number(w, speaker, MessageType::REGULAR, 0x3B2, number, "", ns);
}

/// `LocalOverheadMessage(MessageType.Regular, hue, number)`.
pub fn local_msg(w: &mut World, m: MobileId, hue: i32, number: i32) {
    mm::local_overhead_message_number(w, m, MessageType::REGULAR, hue, number, "");
}

/// `item.SendLocalizedMessageTo(from, number)`.
pub fn item_msg(w: &mut World, item: ItemId, to: MobileId, number: i32) {
    muo_server::item::messages::send_localized_message_to(w, item, to, number, "");
}

/// `x is T`: the mobile's class, or an ancestor, has this full name.
pub fn mobile_is(w: &World, m: MobileId, full_name: &str) -> bool {
    let mut t = w.mobile(m).map(|m| m.ty);
    while let Some(ty) = t {
        if ty.full_name == full_name {
            return true;
        }
        t = ty.parent;
    }
    false
}

/// `x is T` for an item.
pub fn item_is(w: &World, i: ItemId, full_name: &str) -> bool {
    let mut t = w.item(i).map(|i| i.ty);
    while let Some(ty) = t {
        if ty.full_name == full_name {
            return true;
        }
        t = ty.parent;
    }
    false
}

/// `BaseVendor { IsInvulnerable: true }`.
pub fn invulnerable_vendor(w: &World, m: MobileId) -> bool {
    mobile_is(w, m, "Server.Mobiles.BaseVendor")
        && crate::creature::ctype(w, m).is_some_and(|c| (c.is_invulnerable)(w, m))
}

/// `BodyType.Human` of the mobile's body.
pub fn is_human(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|_| {
        w.resources
            .get::<muo_primitives::BodyTable>()
            .is_some_and(|t| t.is_human(muo_server::mobile::part1::props_b::body(w, m)))
    })
}

type OnFinishFn = std::rc::Rc<dyn Fn(&mut World, MobileId)>;
/// The options of a skill's `InternalTarget`.
#[derive(Default)]
pub struct TargetOptions {
    pub allow_nonlocal: bool,
    /// `[PlayerVendorTarget]` on the target class.
    pub player_vendor_target: bool,
    pub on_cancel: Option<fn(&mut World, MobileId, TargetCancelType)>,
    pub on_out_of_range: Option<fn(&mut World, MobileId, Targeted)>,
    /// `OnTargetFinish`.
    pub on_finish: Option<OnFinishFn>,
    /// `BeginTimeout(from, ms)`.
    pub timeout_ms: Option<i64>,
}

/// `from.Target = new InternalTarget()`: a target of `range` (no ground, no flags) calling `on_target`.
pub fn set_target(
    w: &mut World,
    from: MobileId,
    range: i32,
    opts: TargetOptions,
    on_target: impl Fn(&mut World, MobileId, Targeted) + 'static,
) {
    let mut t = Target::new(w, range, false, TargetFlags::NONE);
    t.allow_nonlocal = opts.allow_nonlocal;
    t.player_vendor_target = opts.player_vendor_target;
    t.handlers.on_target = Some(std::rc::Rc::new(on_target));
    if let Some(c) = opts.on_cancel {
        t.handlers.on_target_cancel = Some(std::rc::Rc::new(
            move |w: &mut World, from: MobileId, k: TargetCancelType| c(w, from, k),
        ));
    }
    if let Some(f) = opts.on_out_of_range {
        t.handlers.on_target_out_of_range = Some(std::rc::Rc::new(
            move |w: &mut World, from: MobileId, t: Targeted| f(w, from, t),
        ));
    }
    t.handlers.on_target_finish = opts.on_finish;
    let id = target::add_target(w, t);
    target::set_mobile_target(w, from, Some(id));
    if let Some(ms) = opts.timeout_ms {
        target::begin_timeout(w, id, from, ms);
    }
}

/// The entity a target resolved to.
pub fn entity(t: Targeted) -> Option<EntityId> {
    match t {
        Targeted::Entity(e) => Some(e),
        _ => None,
    }
}

/// `from.CheckTargetSkill(skill, target, min, max)`.
pub fn check_target_skill(
    w: &mut World,
    from: MobileId,
    skill: muo_server::skills::SkillName,
    target: Targeted,
    min: f64,
    max: f64,
) -> bool {
    muo_server::mobile::part2::stats::check_target_skill(w, from, skill, target, min, max)
}

/// `from.CheckSkill(skill, min, max)`.
pub fn check_skill(
    w: &mut World,
    from: MobileId,
    skill: muo_server::skills::SkillName,
    min: f64,
    max: f64,
) -> bool {
    muo_server::mobile::part2::stats::check_skill(w, from, skill, min, max)
}

/// `m.Region.IsPartOf<SafeZone>()`.
pub fn in_safe_zone(w: &World, m: MobileId) -> bool {
    let ty = w
        .resources
        .get::<crate::regions::base_region::RegionHooks>()
        .and_then(|h| h.safe_zone);
    match (ty, w.mobile(m)) {
        (Some(t), Some(x)) => muo_server::region::is_part_of_type(w, x.region, t),
        _ => false,
    }
}
