//! Pieces several spell classes share.

use muo_content::player_mobile::buff::{BuffIcon, BuffInfo, add_buff};
use muo_content::spells::helper::{self, Point3dRef};
use muo_content::spells::hooks::hooks;
use muo_content::spells::spell::{self, SpellId};
use muo_io::text::TextDefinition;
use muo_server::World;
use muo_server::effects::EffectLayer;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::mobile::StatType;
use muo_server::mobile::part2::{effects, messages as mm};
use muo_server::target::Targeted;

/// The mobile a `Targeted` is.
pub fn mobile_of(t: Targeted) -> Option<MobileId> {
    match t {
        Targeted::Entity(EntityId::Mobile(m)) => Some(m),
        _ => None,
    }
}

/// The item a `Targeted` is.
pub fn item_of(t: Targeted) -> Option<ItemId> {
    match t {
        Targeted::Entity(EntityId::Item(i)) => Some(i),
        _ => None,
    }
}

/// `m is PlayerMobile`.
pub fn is_player_mobile(w: &World, m: MobileId) -> bool {
    w.mobile(m)
        .is_some_and(|x| x.ty.is(&muo_content::player_mobile::PLAYER_MOBILE))
}

/// `SpellHelper.Turn(Caster, m)`.
pub fn turn_to(w: &mut World, caster: MobileId, m: MobileId) {
    helper::turn(w, caster, Some(Point3dRef::Entity(EntityId::Mobile(m))));
}

/// `DuelContext.CheckSuddenDeath(Caster)` with its message: true if the spell must not be cast.
pub fn sudden_death_blocks(w: &mut World, sp: SpellId) -> bool {
    if let Some(caster) = spell::caster(w, sp)
        && hooks(w)
            .and_then(|h| h.duel_check_sudden_death)
            .is_some_and(|f| f(w, caster))
    {
        mm::send_message_hue(
            w,
            caster,
            0x22,
            "You cannot cast this spell when in sudden death.",
        );
        return true;
    }
    false
}

/// `CheckCast()` of the spells that refuse in sudden death, else `base.CheckCast()`.
pub fn check_cast_sudden_death(w: &mut World, sp: SpellId) -> bool {
    !sudden_death_blocks(w, sp) && spell::base_check_cast(w, sp)
}

/// Agility, Cunning, Strength: `CheckBSequence`, a stat bonus for the spell's duration and a buff icon.
#[allow(clippy::too_many_arguments)]
pub fn stat_bonus_spell(
    w: &mut World,
    sp: SpellId,
    targeted: Targeted,
    stat: StatType,
    particles: (i32, i32, i32, i32, EffectLayer),
    sound: i32,
    icon: (BuffIcon, i32),
) {
    let Some(m) = mobile_of(targeted) else { return };
    let Some(caster) = spell::caster(w, sp) else {
        return;
    };
    if spell::check_b_sequence(w, sp, m, false) {
        turn_to(w, caster, m);
        let length = helper::get_duration(w, caster, m);
        helper::add_stat_bonus(w, caster, m, stat, length, false);
        effects::fixed_particles(
            w,
            m,
            particles.0,
            particles.1,
            particles.2,
            particles.3,
            particles.4,
        );
        effects::play_sound(w, m, sound);
        let percentage = (helper::get_offset_scalar(w, caster, m, false) * 100.0) as i32;
        if is_player_mobile(w, m) {
            add_buff(
                w,
                m,
                BuffInfo::new(icon.0, icon.1)
                    .duration(length)
                    .args(TextDefinition::of_text(percentage.to_string())),
            );
        }
    }
}

/// `SpellHelper.Turn(Caster, entity)`.
pub fn turn_to_entity(w: &mut World, caster: MobileId, e: EntityId) {
    helper::turn(w, caster, Some(Point3dRef::Entity(e)));
}

/// `m.Poison?.Level`.
pub fn poison_level(w: &World, m: MobileId) -> Option<i32> {
    let p = w.mobile(m)?.poison?;
    w.resources
        .get::<muo_server::poison::Poisons>()?
        .poisons
        .iter()
        .find(|x| x.id() == p)
        .map(|x| x.level)
}

/// `item is T` by full class name, through the base classes.
pub fn item_is(w: &World, id: ItemId, full_name: &str) -> bool {
    let mut t = w.item(id).map(|i| i.ty);
    while let Some(ty) = t {
        if ty.full_name == full_name {
            return true;
        }
        t = ty.parent;
    }
    false
}

/// `a.GetDistanceToSqrt(b)`.
pub fn distance_sqrt(w: &World, a: MobileId, b: MobileId) -> f64 {
    let (Some(a), Some(b)) = (w.mobile(a), w.mobile(b)) else {
        return 0.0;
    };
    let dx = f64::from(a.location.x - b.location.x);
    let dy = f64::from(a.location.y - b.location.y);
    (dx * dx + dy * dy).sqrt()
}

/// `Poison.GetPoison(level)`: the first poison of that level.
pub fn poison_by_level(w: &World, level: i32) -> Option<muo_server::poison::PoisonId> {
    w.resources
        .get::<muo_server::poison::Poisons>()?
        .poisons
        .iter()
        .find(|p| p.family == muo_server::poison::PoisonFamily::Standard && p.level == level)
        .map(|p| p.id())
}

/// `GetSurfaceTop(ref p)` then `new Point3D(p)`.
pub fn surface_top(w: &World, t: Targeted) -> muo_primitives::Point3D {
    match t {
        Targeted::Land(l) => l.location,
        Targeted::Static(s) => helper::static_target_surface_top(w, s),
        Targeted::Entity(e) => {
            helper::point_of(w, helper::get_surface_top(w, Point3dRef::Entity(e)))
        }
    }
}

/// `map.GetMobilesInRange(loc, range)`.
pub fn mobiles_in_range(
    w: &World,
    map: Option<muo_server::map::MapId>,
    loc: muo_primitives::Point3D,
    range: i32,
) -> Vec<MobileId> {
    muo_server::map::enumerate::mobiles_in_range(w, map, loc.x, loc.y, range).collect()
}

/// `(m as BaseCreature)?.OnHarmfulSpell(caster)`.
pub fn creature_on_harmful_spell(w: &mut World, m: MobileId, caster: MobileId) {
    if let Some(c) = muo_content::creature::ctype(w, m) {
        (c.on_harmful_spell)(w, m, caster);
    }
}

/// `caster.RevealingAction()`.
pub fn revealing_action(w: &mut World, m: MobileId) {
    if let Some(ty) = w.mobile(m).map(|x| x.ty) {
        (ty.revealing_action)(w, m);
    }
}

/// `m.Alive`.
pub fn alive(w: &World, m: MobileId) -> bool {
    w.mobile(m).is_some_and(|x| (x.ty.alive)(w, m))
}

/// `if (SpellHelper.CanRevealCaster(m)) caster.RevealingAction()`.
pub fn reveal_caster_if(w: &mut World, m: MobileId, caster: MobileId) {
    if helper::can_reveal_caster(w, m) {
        revealing_action(w, caster);
    }
}

/// `m is T` by full class name, through the base classes.
pub fn mobile_is(w: &World, m: MobileId, full_name: &str) -> bool {
    let mut t = w.mobile(m).map(|x| x.ty);
    while let Some(ty) = t {
        if ty.full_name == full_name {
            return true;
        }
        t = ty.parent;
    }
    false
}

/// `m.Race.RandomSkinHue()`.
pub fn random_skin_hue(w: &mut World, m: MobileId) -> Option<i32> {
    let race = w.mobile(m)?.race?;
    let ops = w
        .resources
        .get::<muo_server::race::Races>()?
        .get(race)
        .map(|r| r.ops)?;
    Some((ops.random_skin_hue)(&mut w.rng))
}

/// `m.GetDistanceToSqrt(p)`.
pub fn distance_sqrt_to(w: &World, m: MobileId, p: muo_primitives::Point3D) -> f64 {
    let Some(a) = w.mobile(m) else { return 0.0 };
    let dx = f64::from(a.location.x - p.x);
    let dy = f64::from(a.location.y - p.y);
    (dx * dx + dy * dy).sqrt()
}
