//! `BaseFamiliar` and the five necromancer familiars.

use muo_content::ai::familiar::{FAMILIAR_AI, FamiliarFlags, assists_master};
use muo_content::creature::control::control_master;
use muo_content::creature::setters::new_creature;
use muo_content::creature::speeds;
use muo_content::creature::{BASE_CREATURE, BASE_CREATURE_C, CreatureType, cd};
use muo_content::hooks::rnd;
use muo_content::mx;
use muo_primitives::{DateTime, Point3D, TimeSpan};
use muo_server::context_menu_entry::ContextMenuEntry;
use muo_server::effects::send_location_particles_full;
use muo_server::ids::{EntityId, ItemId, MobileId};
use muo_server::item::effect_item::{self, DEFAULT_DURATION};
use muo_server::item::{enumerable, ops};
use muo_server::mobile::part2::{items, stats};
use muo_server::mobile::{MobileType, ResistanceType};
use muo_server::poison::PoisonId;
use muo_server::type_registry::TypeRegistry;
use muo_server::world::World;

use crate::common::*;

/// What a familiar class does when released; `HordeMinionFamiliar` overrides it.
pub struct FamiliarClass {
    /// `BeginRelease(from)`.
    pub begin_release: fn(&mut World, MobileId, MobileId),
    /// `AssistsMaster`; false is kept in `FamiliarFlags` for the AI.
    pub assists_master: bool,
}

fn fclass(w: &World, id: MobileId) -> &'static FamiliarClass {
    muo_content::creature::ctype(w, id)
        .and_then(|c| c.subclass)
        .and_then(|s| s.downcast_ref::<FamiliarClass>())
        .unwrap_or(&BASE_FAMILIAR_CLASS)
}

static BASE_FAMILIAR_CLASS: FamiliarClass = FamiliarClass {
    begin_release,
    assists_master: true,
};
static NO_ASSIST_CLASS: FamiliarClass = FamiliarClass {
    assists_master: false,
    ..BASE_FAMILIAR_CLASS
};

fn deleted(w: &World, id: MobileId) -> bool {
    mx::deleted(w, id)
}

fn delete_mobile(w: &mut World, id: MobileId) {
    if let Some(t) = w.mobile(id).map(|m| m.ty) {
        (t.delete)(w, id);
    }
}

fn controlled(w: &World, id: MobileId) -> bool {
    cd(w, id).is_some_and(|d| d.controlled)
}

/// `new BaseFamiliar()`: `BaseCreature(AIType.AI_Melee)` and the speed.
pub fn new_familiar(w: &mut World, ty: &'static MobileType) -> MobileId {
    let id = new_creature(
        w,
        ty,
        AIType::AiMelee,
        FightMode::Closest,
        DEFAULT_RANGE_PERCEPTION,
        1,
    );
    speeds::set_speed(w, id, 0.1, 0.1, true);
    apply_class_flags(w, id);
    id
}

/// A class whose `AssistsMaster` is false keeps it in `FamiliarFlags` for the AI (constructor and load).
fn apply_class_flags(w: &mut World, id: MobileId) {
    if !fclass(w, id).assists_master
        && let Some(m) = w.mobile_mut(id)
    {
        m.ext.insert(FamiliarFlags {
            assists_master: false,
        });
    }
}

// ---- BaseFamiliar ----

fn master_hidden(w: &World, id: MobileId) -> bool {
    control_master(w, id).is_some_and(|m| w.mobile(m).is_some_and(|x| x.hidden))
}

/// `Combatant` setter: the one choke point for "never fights" / "not while the caster is hidden".
fn set_combatant(w: &mut World, id: MobileId, value: Option<MobileId>) {
    if value.is_some() && (!assists_master(w, id) || master_hidden(w, id)) {
        return;
    }
    (BASE_CREATURE.set_combatant)(w, id, value);
}

/// `OnThink()`.
fn on_think(w: &mut World, id: MobileId) {
    (BASE_CREATURE_C.on_think)(w, id);

    if deleted(w, id) {
        return;
    }
    let master = control_master(w, id);
    let Some(master) = master.filter(|&m| !deleted(w, m)) else {
        drop_pack_contents(w, id);
        delete_mobile(w, id);
        return;
    };

    // Compare our own state: Mobile.OnMove reveals a stepping NPC.
    let mh = w.mobile(master).is_some_and(|m| m.hidden);
    if w.mobile(id).is_some_and(|m| m.hidden) != mh {
        muo_server::mobile::part1::props_b::set_hidden(w, id, mh);
        if mh {
            muo_server::mobile::part1::props_b::set_warmode(w, id, false); // nulls Combatant
        }
    }
}

/// Nothing reveals a hidden caster's familiar.
fn revealing_action(w: &mut World, id: MobileId) {
    if !master_hidden(w, id) {
        (BASE_CREATURE.revealing_action)(w, id);
    }
}

fn get_context_menu_entries(
    w: &mut World,
    id: MobileId,
    from: MobileId,
    list: &mut Vec<ContextMenuEntry>,
) {
    (BASE_CREATURE.get_context_menu_entries)(w, id, from, list);
    if mx::alive(w, from) && Some(from) == control_master(w, id) && mx::in_range(w, from, id, 14) {
        let mut e = ContextMenuEntry::new(6118, 14);
        e.on_click = Some(std::rc::Rc::new(|w: &mut World, from, target| {
            if let EntityId::Mobile(f) = target
                && stats::check_alive(w, from, true)
                && !deleted(w, f)
                && controlled(w, f)
                && Some(from) == control_master(w, f)
            {
                (fclass(w, f).begin_release)(w, f, from);
            }
        }));
        list.push(e);
    }
}

/// `BeginRelease(from)`.
fn begin_release(w: &mut World, id: MobileId, from: MobileId) {
    if !deleted(w, id)
        && controlled(w, id)
        && Some(from) == control_master(w, id)
        && stats::check_alive(w, from, true)
    {
        end_release(w, id, Some(from));
    }
}

/// `EndRelease(from)`.
pub fn end_release(w: &mut World, id: MobileId, from: Option<MobileId>) {
    let alive_ok = from.is_none_or(|f| stats::check_alive(w, f, true));
    if alive_ok
        && !deleted(w, id)
        && controlled(w, id)
        && from.is_some()
        && from == control_master(w, id)
    {
        let (loc, map) = w
            .mobile(id)
            .map_or_else(Default::default, |m| (m.location, m.map));
        let e = effect_item::create(w, loc, map, DEFAULT_DURATION);
        send_location_particles_full(w, e, 0x3728, 1, 13, 2100, 3, 5042, 0);
        mx::play_sound(w, id, 0x201);
        delete_mobile(w, id);
    }
}

/// `DropPackContents()`.
pub fn drop_pack_contents(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    let (loc, map, pack) = (m.location, m.map, m.backpack);
    let (Some(map), Some(pack)) = (map, pack) else {
        return;
    };
    if map == muo_server::map::MapId::INTERNAL {
        return;
    }
    for item in enumerable::find_items(w, pack, true, |_, _| true) {
        ops::move_to_world_map(w, item, loc, Some(map));
    }
}

fn after_deserialization(w: &mut World, id: MobileId) {
    muo_server::timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        drop_pack_contents(w, id);
        delete_mobile(w, id);
    });
}

pub static BASE_FAMILIAR_C: CreatureType = CreatureType {
    bard_immune: |_, _| true,
    poison_immune: |_, _| Some(PoisonId(4)), // Poison.Lethal
    commandable: |_, _| false,
    player_range_sensitive: |_, _| false,
    stands_down_on_command: |_, _| false,
    reduce_speed_with_damage: |_, _| false,
    forced_ai: |_, _| Some(&FAMILIAR_AI),
    on_think,
    subclass: Some(&BASE_FAMILIAR_CLASS),
    ..BASE_CREATURE_C
};

/// `Server.Mobiles.BaseFamiliar` (abstract).
pub static BASE_FAMILIAR: MobileType = MobileType {
    full_name: "Server.Mobiles.BaseFamiliar",
    parent: Some(&BASE_CREATURE),
    construct: None,
    family: Some(&BASE_FAMILIAR_C),
    ignore_mobiles: |_, _| true,
    set_combatant,
    revealing_action,
    get_context_menu_entries,
    serialize: |w, id, out| {
        (BASE_CREATURE.serialize)(w, id, out);
        out.write_i32(0);
    },
    deserialize: |w, id, r| {
        (BASE_CREATURE.deserialize)(w, id, r)?;
        let _version = r.read_i32()?;
        apply_class_flags(w, id);
        after_deserialization(w, id);
        Ok(())
    },
    ..BASE_CREATURE
};

// ---- DarkWolfFamiliar ----

/// `m_NextRestore`, `m_NextFlare`: the next time the familiar's own effect runs.
struct NextEffect(DateTime);

fn dark_wolf_new(w: &mut World) -> MobileId {
    let id = new_familiar(w, &DARK_WOLF_FAMILIAR);
    body(w, id, 99);
    hue(w, id, 0x901);
    base_sound_id(w, id, 0xE5);
    set_str(w, id, 100);
    set_dex(w, id, 90);
    set_int(w, id, 90);
    set_hits(w, id, 60);
    set_stam(w, id, 90);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 40, 50);
    set_resistance_range(w, id, ResistanceType::Fire, 25, 40);
    set_resistance_range(w, id, ResistanceType::Cold, 25, 40);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 40);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 40);
    set_skill_range(w, id, SkillName::Wrestling, 85.1, 90.0);
    set_skill(w, id, SkillName::Tactics, 50.0);
    control_slots(w, id, 1);
    id
}

fn next_effect(w: &World, id: MobileId) -> DateTime {
    w.mobile(id)
        .and_then(|m| m.ext.get::<NextEffect>())
        .map_or(DateTime::MIN, |n| n.0)
}

fn set_next_effect(w: &mut World, id: MobileId, t: DateTime) {
    if let Some(m) = w.mobile_mut(id) {
        m.ext.insert(NextEffect(t));
    }
}

fn dark_wolf_on_think(w: &mut World, id: MobileId) {
    on_think(w, id);
    if w.now < next_effect(w, id) {
        return;
    }
    let t = w.now + TimeSpan::from_secs(2);
    set_next_effect(w, id, t);
    if let Some(caster) = muo_content::creature::control::get_master(w, id) {
        // ++caster.Stam
        let stam = w.mobile(caster).map_or(0, |m| m.stam);
        muo_server::mobile::part1::props_b::set_stam(w, caster, stam + 1);
    }
}

creature_leaf! {
    DARK_WOLF_FAMILIAR, DARK_WOLF_FAMILIAR_C = "Server.Mobiles.DarkWolfFamiliar" : BASE_FAMILIAR / BASE_FAMILIAR_C, ctor None, encoded false;
    c {
        corpse_name: |_w, _| Some("a dark wolf corpse"),
        default_name: |_w, _| Some("a dark wolf"),
        on_think: dark_wolf_on_think,
    }
    m {
        make: Some(dark_wolf_new),
    }
}

// ---- DeathAdder ----

fn death_adder_new(w: &mut World) -> MobileId {
    let id = new_familiar(w, &DEATH_ADDER);
    body(w, id, 0x15);
    hue(w, id, 0x455);
    base_sound_id(w, id, 219);
    set_str(w, id, 70);
    set_dex(w, id, 150);
    set_int(w, id, 100);
    set_hits(w, id, 50);
    set_stam(w, id, 150);
    set_mana(w, id, 0);
    set_damage(w, id, 1, 4);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance(w, id, ResistanceType::Physical, 10);
    set_resistance(w, id, ResistanceType::Poison, 100);
    set_skill(w, id, SkillName::Wrestling, 90.0);
    set_skill(w, id, SkillName::Tactics, 50.0);
    set_skill(w, id, SkillName::MagicResist, 100.0);
    set_skill(w, id, SkillName::Poisoning, 150.0);
    control_slots(w, id, 1);
    id
}

creature_leaf! {
    DEATH_ADDER, DEATH_ADDER_C = "Server.Mobiles.DeathAdder" : BASE_FAMILIAR / BASE_FAMILIAR_C, ctor None, encoded true;
    c {
        corpse_name: |_w, _| Some("a death adder corpse"),
        default_name: |_w, _| Some("a death adder"),
        hit_poison: |w, _| Some(if rnd::double(w) < 0.8 { PoisonId(2) } else { PoisonId(3) }),
        subclass: Some(&NO_ASSIST_CLASS),
    }
    m {
        make: Some(death_adder_new),
    }
}

// ---- ShadowWispFamiliar ----

fn shadow_wisp_new(w: &mut World) -> MobileId {
    let id = new_familiar(w, &SHADOW_WISP_FAMILIAR);
    body(w, id, 165);
    hue(w, id, 0x901);
    base_sound_id(w, id, 466);
    set_str(w, id, 50);
    set_dex(w, id, 60);
    set_int(w, id, 100);
    set_hits(w, id, 50);
    set_stam(w, id, 60);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Energy, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 10, 15);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 15);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 15);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 15);
    set_resistance(w, id, ResistanceType::Energy, 99);
    set_skill(w, id, SkillName::Wrestling, 40.0);
    set_skill(w, id, SkillName::Tactics, 40.0);
    control_slots(w, id, 1);
    id
}

fn shadow_wisp_on_think(w: &mut World, id: MobileId) {
    on_think(w, id);
    if w.now < next_effect(w, id) {
        return;
    }
    let t = w.now + TimeSpan::from_secs_f64(5.0 + 25.0 * rnd::double(w));
    set_next_effect(w, id, t);

    muo_server::mobile::part2::effects::fixed_effect_hue(w, id, 0x37C4, 1, 12, 1109, 6);
    mx::play_sound(w, id, 0x1D3);

    muo_server::timer::start(w, TimeSpan::from_millis(500), TimeSpan::ZERO, 1, move |w| {
        flare(w, id)
    });
}

/// `Flare()`.
fn flare(w: &mut World, id: MobileId) {
    let Some(caster) = muo_content::creature::control::get_master(w, id) else {
        return;
    };
    for m in mx::mobiles_in_range(w, id, 5) {
        let Some(mm) = w.mobile(m) else { continue };
        if !mm.player
            || !mx::alive(w, m)
            || muo_content::creature::is_creature(w, m)
                && w.mobile(m).is_some_and(|x| (x.ty.is_dead_bonded_pet)(w, m))
            || mm.karma > 0
            || mm.access_level >= muo_server::mobile::AccessLevel::Counselor
        {
            continue;
        }
        let friendly = w.mobile(caster).is_none_or(|c| {
            !c.aggressors.iter().any(|a| a.attacker == m)
                && !c.aggressed.iter().any(|a| a.defender == m)
        });
        if friendly {
            muo_server::mobile::part2::effects::fixed_effect_hue(w, m, 0x37C4, 1, 12, 1109, 3); // At player
            let karma = w.mobile(m).map_or(0, |x| x.karma);
            let mana = w.mobile(m).map_or(0, |x| x.mana);
            muo_server::mobile::part1::props_b::set_mana(w, m, mana + 1 - karma / 1000);
        }
    }
}

creature_leaf! {
    SHADOW_WISP_FAMILIAR, SHADOW_WISP_FAMILIAR_C = "Server.Mobiles.ShadowWispFamiliar" : BASE_FAMILIAR / BASE_FAMILIAR_C, ctor None, encoded false;
    c {
        corpse_name: |_w, _| Some("a shadow wisp corpse"),
        default_name: |_w, _| Some("a shadow wisp"),
        on_think: shadow_wisp_on_think,
        subclass: Some(&NO_ASSIST_CLASS),
    }
    m {
        make: Some(shadow_wisp_new),
    }
}

// ---- VampireBatFamiliar ----

fn vampire_bat_new(w: &mut World) -> MobileId {
    let id = new_familiar(w, &VAMPIRE_BAT_FAMILIAR);
    body(w, id, 317);
    base_sound_id(w, id, 0x270);
    set_str(w, id, 120);
    set_dex(w, id, 120);
    set_int(w, id, 100);
    set_hits(w, id, 90);
    set_stam(w, id, 120);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 10, 15);
    set_resistance_range(w, id, ResistanceType::Fire, 10, 15);
    set_resistance_range(w, id, ResistanceType::Cold, 10, 15);
    set_resistance_range(w, id, ResistanceType::Poison, 10, 15);
    set_resistance_range(w, id, ResistanceType::Energy, 10, 15);
    set_skill_range(w, id, SkillName::Wrestling, 95.1, 100.0);
    set_skill(w, id, SkillName::Tactics, 50.0);
    control_slots(w, id, 1);
    id
}

creature_leaf! {
    VAMPIRE_BAT_FAMILIAR, VAMPIRE_BAT_FAMILIAR_C = "Server.Mobiles.VampireBatFamiliar" : BASE_FAMILIAR / BASE_FAMILIAR_C, ctor None, encoded false;
    c {
        corpse_name: |_w, _| Some("a vampire bat corpse"),
        default_name: |_w, _| Some("a vampire bat"),
    }
    m {
        make: Some(vampire_bat_new),
    }
}

// ---- HordeMinionFamiliar ----

type ContextMenuEntriesFn = fn(&mut World, MobileId, MobileId, &mut Vec<ContextMenuEntry>);
/// `PackAnimal.*` lives with the animals (`hand::pack_animal`). Until it sets these, the horde
/// minion's pack behaves as the base class does without it.
#[derive(Default)]
pub struct PackAnimalHooks {
    /// `PackAnimal.CombineBackpacks(animal)` (OnBeforeDeath).
    pub combine_backpacks: Option<fn(&mut World, MobileId)>,
    /// `PackAnimal.CheckAccess(animal, from)`.
    pub check_access: Option<fn(&World, MobileId, MobileId) -> bool>,
    /// `PackAnimal.TryPackOpen(animal, from)`.
    pub try_pack_open: Option<fn(&mut World, MobileId, MobileId)>,
    /// `PackAnimal.GetContextMenuEntries(animal, from, ref list)`.
    pub context_menu_entries: Option<ContextMenuEntriesFn>,
    /// `from.SendGump(new ReleaseFamiliarWarningGump(callback))` with the answer applied by [`end_release`].
    pub release_warning_gump: Option<fn(&mut World, MobileId, MobileId)>,
}

fn pa(w: &World) -> Option<&PackAnimalHooks> {
    w.resources.get::<PackAnimalHooks>()
}

fn pack_access(w: &World, id: MobileId, from: MobileId) -> bool {
    pa(w)
        .and_then(|h| h.check_access)
        .is_some_and(|f| f(w, id, from))
}

fn horde_minion_new(w: &mut World) -> MobileId {
    let id = new_familiar(w, &HORDE_MINION_FAMILIAR);
    body(w, id, 776);
    base_sound_id(w, id, 0x39D);
    set_str(w, id, 100);
    set_dex(w, id, 110);
    set_int(w, id, 100);
    set_hits(w, id, 70);
    set_stam(w, id, 110);
    set_mana(w, id, 0);
    set_damage(w, id, 5, 10);
    set_damage_type(w, id, ResistanceType::Physical, 100);
    set_resistance_range(w, id, ResistanceType::Physical, 50, 60);
    set_resistance_range(w, id, ResistanceType::Fire, 50, 55);
    set_resistance_range(w, id, ResistanceType::Poison, 25, 30);
    set_resistance_range(w, id, ResistanceType::Energy, 25, 30);
    set_skill_range(w, id, SkillName::Wrestling, 70.1, 75.0);
    set_skill(w, id, SkillName::Tactics, 50.0);
    control_slots(w, id, 1);

    if let Some(old) = w.mobile(id).and_then(|m| m.backpack)
        && let Some(t) = w.item(old).map(|i| i.ty)
    {
        (t.delete)(w, old);
    }
    if let Some(pack) = muo_content::factory::make_item(w, "Server.Items.Backpack") {
        ops::set_movable(w, pack, false);
        ops::set_weight(w, pack, 13.0);
        items::add_item(w, id, pack);
    }
    id
}

/// `_nextPickup`.
struct NextPickup(i64);

fn horde_minion_on_think(w: &mut World, id: MobileId) {
    on_think(w, id);

    let next = w
        .mobile(id)
        .and_then(|m| m.ext.get::<NextPickup>())
        .map_or(0, |n| n.0);
    if w.tick_count.wrapping_sub(next) <= 0 {
        return;
    }
    let mut picked_up = 3;
    'pickup: {
        let Some(pack) = w.mobile(id).and_then(|m| m.backpack) else {
            break 'pickup;
        };
        let mut queue: Vec<ItemId> =
            muo_server::mobile::part2::enumerators::get_items_in_range(w, id, 2)
                .into_iter()
                .filter(|&i| w.item(i).is_some_and(|it| it.movable() && it.stackable()))
                .collect();
        queue.reverse();
        while picked_up > 0 {
            let Some(item) = queue.pop() else { break };
            if !muo_server::item::container::check_hold(w, pack, Some(id), item, false) {
                break 'pickup;
            }
            let tick = w.tick_count;
            if let Some(m) = w.mobile_mut(id) {
                m.next_action_time = tick;
            }
            let amount = w.item(item).map_or(0, |i| i.amount);
            let (rejected, _) = muo_server::mobile::base::lift(w, id, item, amount);
            if rejected {
                continue;
            }
            muo_server::mobile::base::drop_mobile(w, id, id, Point3D::ZERO);
            picked_up -= 1;
        }
    }
    if picked_up < 3 {
        // 5-10s
        let t = w.tick_count + i64::from(rnd::from(w, 5000, 5000));
        if let Some(m) = w.mobile_mut(id) {
            m.ext.insert(NextPickup(t));
        }
    }
}

fn horde_minion_begin_release(w: &mut World, id: MobileId, from: MobileId) {
    let has_items = w
        .mobile(id)
        .and_then(|m| m.backpack)
        .and_then(|p| w.item(p))
        .is_some_and(|p| !p.items.is_empty());
    if has_items {
        if let Some(f) = pa(w).and_then(|h| h.release_warning_gump) {
            f(w, id, from);
        }
    } else {
        end_release(w, id, Some(from));
    }
}

static HORDE_MINION_CLASS: FamiliarClass = FamiliarClass {
    begin_release: horde_minion_begin_release,
    ..BASE_FAMILIAR_CLASS
};

fn horde_minion_on_before_death(w: &mut World, id: MobileId) -> bool {
    if !(BASE_FAMILIAR.on_before_death)(w, id) {
        return false;
    }
    if let Some(f) = pa(w).and_then(|h| h.combine_backpacks) {
        f(w, id);
    }
    true
}

creature_leaf! {
    HORDE_MINION_FAMILIAR, HORDE_MINION_FAMILIAR_C = "Server.Mobiles.HordeMinionFamiliar" : BASE_FAMILIAR / BASE_FAMILIAR_C, ctor None, encoded false;
    c {
        corpse_name: |_w, _| Some("a horde minion corpse"),
        default_name: |_w, _| Some("a horde minion"),
        display_weight: |_w, _| true,
        on_think: horde_minion_on_think,
        subclass: Some(&HORDE_MINION_CLASS),
    }
    m {
        make: Some(horde_minion_new),
        on_before_death: horde_minion_on_before_death,
        get_inventory_move_result_for: |_, _, _| muo_server::item::DeathMoveResult::MoveToCorpse,
        is_snoop: |w, id, from| !pack_access(w, id, from) && (BASE_FAMILIAR.is_snoop)(w, id, from),
        check_nonlocal_drop: |w, id, from, _, _| pack_access(w, id, from),
        check_nonlocal_lift: |w, id, from, _| pack_access(w, id, from),
        on_drag_drop: |w, id, from, item| {
            if (BASE_CREATURE_C.check_feed)(w, id, from, item) {
                return true;
            }
            if pack_access(w, id, from) {
                muo_server::mobile::part2::items::add_to_backpack(w, id, item);
                return true;
            }
            (BASE_FAMILIAR.on_drag_drop)(w, id, from, item)
        },
        on_double_click: |w, id, from| {
            if let Some(f) = pa(w).and_then(|h| h.try_pack_open) {
                f(w, id, from);
            }
        },
        get_context_menu_entries: |w, id, from, list| {
            get_context_menu_entries(w, id, from, list);
            if let Some(f) = pa(w).and_then(|h| h.context_menu_entries) {
                f(w, id, from, list);
            }
        },
    }
}

pub(crate) fn register(types: &mut TypeRegistry) {
    for t in [
        &DARK_WOLF_FAMILIAR,
        &DEATH_ADDER,
        &SHADOW_WISP_FAMILIAR,
        &VAMPIRE_BAT_FAMILIAR,
        &HORDE_MINION_FAMILIAR,
    ] {
        types.register_mobile(t, &[]);
    }
}

#[cfg(test)]
pub(crate) fn all() -> Vec<&'static MobileType> {
    vec![
        &DARK_WOLF_FAMILIAR,
        &DEATH_ADDER,
        &SHADOW_WISP_FAMILIAR,
        &VAMPIRE_BAT_FAMILIAR,
        &HORDE_MINION_FAMILIAR,
    ]
}
