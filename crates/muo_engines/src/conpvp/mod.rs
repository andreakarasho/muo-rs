//! The dueling system.
//!
//! `mod.rs` holds the data (`DuelContext`, `Participant`, `DuelPlayer`), the questions the rest of the server asks
//! about a duel (the Allow*/Check* rules of `DuelContext`, `IsAlly`, `IsFreeConsume`) and the hooks that carry
//! them (`PlayerMobile`, the weapons, notoriety, spells). The challenge/arena flow is in `flow`, the arenas in
//! `arena`, the ladder and preferences in `ladder`, the region in `safezone`, the gumps in `gumps`.
//!
//! A context is addressed by its id (`u32`); `PlayerMobile.DuelContext`/`DuelPlayer` is `DuelState::of_mobile`.
//!
//! Tournaments are in `tournament` (+ `tournament_gumps`), the arena moongate list in `arena_gump`.
//!
//! The event games are in `games` (the shared `EventController`, scores, scoreboard, awards), `ctf`, `dd`, `koth` and
//! `bombing`; the spectator `ArenaMoongate` is `spectator`. The context carries `tournament`/`event_game` as the flags
//! the rules ask about.

pub mod arena;
pub mod arena_gump;
pub mod bombing;
pub mod ctf;
pub mod dd;
#[cfg(test)]
mod duel_tests;
pub mod flow;
#[cfg(test)]
mod flow_tests;
pub mod games;
pub mod gumps;
pub mod koth;
pub mod ladder;
pub mod ladder_gump;
pub mod props;
mod ruleset;
pub mod safezone;
pub mod spectator;
pub mod speech;
pub mod teleporter_addon;
pub mod tournament;
pub mod tournament_gumps;
pub mod trophy;

use std::collections::HashMap;

use indexmap::IndexMap;
use muo_primitives::Point3D;
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part2::messages as mm;
use muo_server::skills::{SkillName, SkillTable};
use muo_server::timer::TimerHandle;

use muo_content::items::{armor, weapon};

pub use ruleset::{Ruleset, RulesetLayout};

/// `DuelPlayer`.
#[derive(Copy, Clone, Debug)]
pub struct DuelPlayer {
    pub mobile: MobileId,
    pub ready: bool,
    pub eliminated: bool,
}

impl DuelPlayer {
    pub fn new(mobile: MobileId) -> Self {
        DuelPlayer {
            mobile,
            ready: false,
            eliminated: false,
        }
    }
}

/// `Participant`: a side of the duel. A slot is `None` while nobody holds it.
#[derive(Clone, Debug, Default)]
pub struct Participant {
    /// Which participant it is (participants are compared by it); see [`DuelState::new_participant`].
    pub uid: u32,
    pub players: Vec<Option<DuelPlayer>>,
    /// `TourneyPart`: an index into the tournament's participants.
    pub tourney_part: Option<usize>,
}

impl Participant {
    /// `new Participant(context, count)`.
    pub fn new(count: usize) -> Self {
        Participant {
            uid: 0,
            players: vec![None; count],
            tourney_part: None,
        }
    }

    /// `Count`.
    pub fn count(&self) -> usize {
        self.players.len()
    }

    /// `FilledSlots`.
    pub fn filled_slots(&self) -> usize {
        self.players.iter().filter(|p| p.is_some()).count()
    }

    /// `HasOpenSlot`.
    pub fn has_open_slot(&self) -> bool {
        self.players.iter().any(|p| p.is_none())
    }

    /// `Eliminated`: nobody left who is not eliminated.
    pub fn eliminated(&self) -> bool {
        !self
            .players
            .iter()
            .any(|p| p.is_some_and(|p| !p.eliminated))
    }

    /// `Find(mob)`.
    pub fn find(&self, m: MobileId) -> Option<&DuelPlayer> {
        self.players.iter().flatten().find(|p| p.mobile == m)
    }

    /// `Contains(mob)`.
    pub fn contains(&self, m: MobileId) -> bool {
        self.find(m).is_some()
    }

    /// `Resize(count)`: the filled slots move to the front, up to `count` of them.
    pub fn resize(&mut self, count: usize) {
        let old = std::mem::take(&mut self.players);
        self.players = vec![None; count];
        let mut ct = 0;
        for p in old.into_iter().flatten() {
            if ct < count {
                self.players[ct] = Some(p);
                ct += 1;
            }
        }
    }

    /// `Nullify(player)`.
    pub fn nullify(&mut self, m: MobileId) {
        if let Some(s) = self
            .players
            .iter_mut()
            .find(|s| s.is_some_and(|p| p.mobile == m))
        {
            *s = None;
        }
    }
}

/// The `EventGame` of a context: its controller and kind (`games` has the rules), and `FreeConsume`.
#[derive(Copy, Clone, Debug)]
pub struct EventGameFlags {
    pub free_consume: bool,
    pub controller: ItemId,
    pub kind: games::GameKind,
}

/// What is counting down: `StartCountdown(count, cb)`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CountdownKind {
    /// `SendReadyGump(int)`.
    Ready,
    /// `SendBeginGump(int)`.
    Begin,
}

/// `DuelContext`.
#[derive(Clone, Debug)]
pub struct DuelContext {
    pub initiator: MobileId,
    pub participants: Vec<Participant>,
    pub ruleset: Ruleset,
    /// `Arena`: the `ArenaController` item holding it.
    pub arena: Option<ItemId>,
    /// `m_OverrideArena`.
    pub override_arena: Option<ItemId>,
    pub rematch: bool,
    pub ready_wait: bool,
    pub ready_count: i32,
    pub registered: bool,
    pub started: bool,
    pub finished: bool,
    pub started_begin_countdown: bool,
    pub started_ready_countdown: bool,
    pub tied: bool,
    pub is_sudden_death: bool,
    /// `m_Yielding`.
    pub yielding: bool,
    /// `m_Tournament != null`, and its `IsNotoRestricted`.
    pub tournament: Option<bool>,
    /// The `TournamentController` item of `m_Tournament`.
    pub tournament_ctrl: Option<ItemId>,
    /// `m_Match`: (level, index) in the tournament's pyramid.
    pub tmatch: Option<(usize, usize)>,
    /// `m_EventGame`, and its `FreeConsume` / `CantDoAnything`: an event game with the flags it reports.
    pub event_game: Option<EventGameFlags>,
    /// `m_Walls`.
    pub walls: Vec<ItemId>,
    /// `m_GatePoint` / `m_GateFacet`.
    pub gate_point: Point3D,
    pub gate_facet: Option<muo_server::map::MapId>,
    pub auto_tie_timer: TimerHandle,
    pub countdown_timer: TimerHandle,
    pub sd_warn_timer: TimerHandle,
    pub sd_activate_timer: TimerHandle,
}

impl DuelContext {
    /// `new DuelContext(initiator, layout, addNew: false)`; the caller fills the participants.
    pub fn empty(initiator: MobileId, ruleset: Ruleset) -> Self {
        DuelContext {
            initiator,
            participants: Vec::new(),
            ruleset,
            arena: None,
            override_arena: None,
            rematch: false,
            ready_wait: false,
            ready_count: 0,
            registered: true,
            started: false,
            finished: false,
            started_begin_countdown: false,
            started_ready_countdown: false,
            tied: false,
            is_sudden_death: false,
            yielding: false,
            tournament: None,
            tournament_ctrl: None,
            tmatch: None,
            event_game: None,
            walls: Vec::new(),
            gate_point: Point3D::default(),
            gate_facet: None,
            auto_tie_timer: TimerHandle::default(),
            countdown_timer: TimerHandle::default(),
            sd_warn_timer: TimerHandle::default(),
            sd_activate_timer: TimerHandle::default(),
        }
    }
}

impl Default for DuelContext {
    fn default() -> Self {
        DuelContext::empty(MobileId(Default::default()), Ruleset::default())
    }
}

impl DuelContext {
    /// `Find(mob)`: the player's `DuelPlayer` in this context.
    pub fn find(&self, m: MobileId) -> Option<&DuelPlayer> {
        self.participants.iter().find_map(|p| p.find(m))
    }

    pub fn find_mut(&mut self, m: MobileId) -> Option<&mut DuelPlayer> {
        self.participants
            .iter_mut()
            .flat_map(|p| p.players.iter_mut())
            .flatten()
            .find(|p| p.mobile == m)
    }

    /// The index of the participant with this `uid`.
    pub fn participant_index(&self, uid: u32) -> Option<usize> {
        self.participants.iter().position(|p| p.uid == uid)
    }

    /// The index of the participant `m` belongs to.
    pub fn participant_of(&self, m: MobileId) -> Option<usize> {
        self.participants.iter().position(|p| p.contains(m))
    }

    /// `IsAlly(m1, m2)`: same participant.
    pub fn is_ally(&self, a: MobileId, b: MobileId) -> bool {
        match (self.participant_of(a), self.participant_of(b)) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    }

    /// `IsOneVsOne`.
    pub fn is_one_vs_one(&self) -> bool {
        self.participants.len() == 2
            && self.participants[0].count() == 1
            && self.participants[1].count() == 1
    }

    /// Every player of every participant, in order.
    pub fn all_players(&self) -> Vec<MobileId> {
        self.participants
            .iter()
            .flat_map(|p| p.players.iter().flatten().map(|d| d.mobile))
            .collect()
    }
}

/// The duels, and who is in which: `PlayerMobile.DuelPlayer` / `DuelContext`.
#[derive(Default)]
pub struct DuelState {
    pub contexts: IndexMap<u32, DuelContext>,
    pub of_mobile: HashMap<MobileId, u32>,
    pub last: u32,
    next_participant: u32,
}

impl DuelState {
    /// `new Participant(context, count)`.
    pub fn new_participant(&mut self, count: usize) -> Participant {
        self.next_participant += 1;
        Participant {
            uid: self.next_participant,
            ..Participant::new(count)
        }
    }

    /// Registers a context and the players in it.
    pub fn add(&mut self, c: DuelContext) -> u32 {
        self.last += 1;
        for m in c.all_players() {
            self.of_mobile.insert(m, self.last);
        }
        self.contexts.insert(self.last, c);
        self.last
    }
}

pub(crate) fn st(w: &World) -> Option<&DuelState> {
    w.resources.get::<DuelState>()
}

pub(crate) fn st_mut(w: &mut World) -> &mut DuelState {
    w.resources.get_mut::<DuelState>()
}

/// `PlayerMobile.DuelContext`: the id of the context `m` is in.
pub fn duel_context_of(w: &World, m: MobileId) -> Option<u32> {
    st(w)?.of_mobile.get(&m).copied()
}

/// A context by id.
pub fn ctx(w: &World, id: u32) -> Option<&DuelContext> {
    st(w)?.contexts.get(&id)
}

/// A context by id, mutable.
pub fn ctx_mut(w: &mut World, id: u32) -> Option<&mut DuelContext> {
    st_mut(w).contexts.get_mut(&id)
}

fn context(w: &World, m: MobileId) -> Option<&DuelContext> {
    ctx(w, duel_context_of(w, m)?)
}

/// `pm.DuelPlayer = new DuelPlayer(...)`: puts `m` in the slot (a slot is `None` while nobody holds it).
pub fn set_slot(w: &mut World, id: u32, participant: usize, slot: usize, m: MobileId) {
    let Some(c) = ctx_mut(w, id) else { return };
    if let Some(s) = c
        .participants
        .get_mut(participant)
        .and_then(|p| p.players.get_mut(slot))
    {
        *s = Some(DuelPlayer::new(m));
        let was = in_tourney(w, m);
        st_mut(w).of_mobile.insert(m, id);
        resend_if_tourney_changed(w, m, was);
    }
}

/// `DuelContext?.Finished == false && DuelContext.m_Tournament != null`.
fn in_tourney(w: &World, m: MobileId) -> bool {
    context(w, m).is_some_and(|c| !c.finished && c.tournament.is_some())
}

/// The `PlayerMobile.DuelPlayer` setter: `SendEverything()` when the player entered or left a running tournament match.
fn resend_if_tourney_changed(w: &mut World, m: MobileId, was: bool) {
    if muo_content::player_mobile::data(w, m).is_some() && was != in_tourney(w, m) {
        muo_server::mobile::part2::effects::send_everything(w, m);
    }
}

/// `Participant.Add(mob)`: the first open slot, or one more.
pub fn participant_add(w: &mut World, id: u32, participant: usize, m: MobileId) {
    let Some(p) = ctx(w, id).and_then(|c| c.participants.get(participant)) else {
        return;
    };
    if p.contains(m) {
        return;
    }
    let slot = match p.players.iter().position(|s| s.is_none()) {
        Some(i) => i,
        None => {
            let n = p.count();
            if let Some(c) = ctx_mut(w, id) {
                c.participants[participant].resize(n + 1);
            }
            n
        }
    };
    set_slot(w, id, participant, slot, m);
}

/// `pm.DuelPlayer = null`.
pub fn release_mobile(w: &mut World, m: MobileId) {
    let was = in_tourney(w, m);
    st_mut(w).of_mobile.remove(&m);
    resend_if_tourney_changed(w, m, was);
}

/// `DuelPlayer.Eliminated = true`: a tournament match is told and the player resent.
pub fn set_eliminated(w: &mut World, id: u32, m: MobileId) {
    let in_tournament = {
        let Some(c) = ctx_mut(w, id) else { return };
        if let Some(p) = c.find_mut(m) {
            p.eliminated = true;
        }
        c.tournament.is_some()
    };
    if in_tournament {
        if let Some(pi) = ctx(w, id).and_then(|c| c.participant_of(m)) {
            tournament::on_eliminated(w, id, pi);
        }
        muo_server::mobile::part2::effects::send_everything(w, m);
    }
}

/// `Find(from)?.Eliminated != false`: no player, or eliminated.
fn eliminated_or_none(c: &DuelContext, m: MobileId) -> bool {
    c.find(m).is_none_or(|p| p.eliminated)
}

/// `CantDoAnything(mob)`: only event games restrict.
fn cant_do_anything(w: &World, c: &DuelContext, m: MobileId) -> bool {
    c.event_game.is_some()
        && duel_context_of(w, m).is_some_and(|id| games::cant_do_anything(w, id, m))
}

fn send(w: &mut World, m: MobileId, text: &str) {
    mm::send_message(w, m, text);
}

/// `IsFreeConsume(mob)`.
pub fn is_free_consume(w: &World, m: MobileId) -> bool {
    context(w, m)
        .and_then(|c| c.event_game)
        .is_some_and(|g| g.free_consume)
}

/// `CheckItemEquip(from, item)`.
pub fn check_item_equip(w: &World, c: &DuelContext, item: ItemId) -> bool {
    let Some(it) = w.item(item) else { return true };
    let ty = it.ty;
    if muo_content::factory::item_is(w, item, "Server.Items.Fists") {
        return c.ruleset.get_option("Weapons", "Wrestling");
    }
    if ty.is(&armor::BASE_ARMOR) {
        let Some(d) = armor::data(w, item) else {
            return true;
        };
        if d.protection_level as i32 > armor::ArmorProtectionLevel::Regular as i32
            && !c.ruleset.get_option("Armor", "Magical")
        {
            return false;
        }
        if !muo_content::core::aos(w)
            && d.resource != armor::class(w, item).default_resource
            && !c.ruleset.get_option("Armor", "Colored")
        {
            return false;
        }
        if ty.is(&armor::BASE_SHIELD) && !c.ruleset.get_option("Armor", "Shields") {
            return false;
        }
    } else if ty.is(&weapon::BASE_WEAPON) {
        let Some(d) = weapon::data(w, item) else {
            return true;
        };
        if (d.damage_level as i32 > weapon::WeaponDamageLevel::Regular as i32
            || d.accuracy_level as i32 > weapon::WeaponAccuracyLevel::Regular as i32)
            && !c.ruleset.get_option("Weapons", "Magical")
        {
            return false;
        }
        if !muo_content::core::aos(w)
            && d.resource != muo_content::craft::CraftResource::Iron
            && d.resource != muo_content::craft::CraftResource::None
            && !c.ruleset.get_option("Weapons", "Runics")
        {
            return false;
        }
        let ranged = ty.is(&weapon::BASE_RANGED);
        if ranged && !c.ruleset.get_option("Weapons", "Ranged") {
            return false;
        }
        if !ranged && !c.ruleset.get_option("Weapons", "Melee") {
            return false;
        }
        if d.poison_charges > 0
            && d.poison.is_some()
            && !c.ruleset.get_option("Weapons", "Poisoned")
        {
            return false;
        }
        if muo_content::factory::item_is(w, item, "Server.Items.BaseWand")
            && !c.ruleset.get_option("Items", "Wands")
        {
            return false;
        }
    }
    true
}

/// `AllowItemEquip(from, item)`.
pub fn allow_item_equip(w: &mut World, from: MobileId, item: ItemId) -> bool {
    let Some(c) = context(w, from) else {
        return true;
    };
    if !c.started_begin_countdown || eliminated_or_none(c, from) {
        return true;
    }
    if muo_content::factory::item_is(w, item, "Server.Items.Dagger") || check_item_equip(w, c, item)
    {
        return true;
    }
    send(
        w,
        from,
        "The dueling ruleset prevents you from equipping this item.",
    );
    false
}

/// `AllowSkillUse(from, skill)`.
pub fn allow_skill_use(w: &mut World, from: MobileId, skill: SkillName) -> bool {
    let Some(c) = context(w, from) else {
        return true;
    };
    if !c.started_begin_countdown || eliminated_or_none(c, from) {
        return true;
    }
    if cant_do_anything(w, c, from) {
        return false;
    }
    let id = skill as usize;
    let name = w
        .resources
        .get::<SkillTable>()
        .and_then(|t| t.info.get(id))
        .map(|s| s.name.clone());
    if name.is_some_and(|n| c.ruleset.get_option("Skills", &n)) {
        return true;
    }
    send(
        w,
        from,
        "The dueling ruleset prevents you from using this skill.",
    );
    false
}

/// `AllowItemUse(from, item)`.
pub fn allow_item_use(w: &mut World, from: MobileId, item: ItemId) -> bool {
    let Some(c) = context(w, from) else {
        return true;
    };
    if !c.started_begin_countdown || eliminated_or_none(c, from) {
        return true;
    }
    let is = |name: &str| muo_content::factory::item_is(w, item, name);
    let refresh = is("Server.Items.BaseRefreshPotion");
    if !refresh && cant_do_anything(w, c, from) {
        return false;
    }
    let potion = is("Server.Items.BasePotion");
    let (mut title, mut option): (Option<&str>, Option<&str>) = (None, None);
    if potion {
        title = Some("Potions");
        option = [
            ("Server.Items.BaseAgilityPotion", "Agility"),
            ("Server.Items.BaseCurePotion", "Cure"),
            ("Server.Items.BaseHealPotion", "Heal"),
            ("Server.Items.NightSightPotion", "Nightsight"),
            ("Server.Items.BasePoisonPotion", "Poison"),
            ("Server.Items.BaseStrengthPotion", "Strength"),
            ("Server.Items.BaseExplosionPotion", "Explosion"),
            ("Server.Items.BaseRefreshPotion", "Refresh"),
        ]
        .into_iter()
        .find(|(n, _)| is(n))
        .map(|(_, o)| o);
    } else if is("Server.Items.Bandage") {
        (title, option) = (Some("Items"), Some("Bandages"));
    } else if is("Server.Items.TrappableContainer") {
        // `TrapType != None`
        if muo_content::player_mobile::hooks::item_is_trapped(w, item) {
            (title, option) = (Some("Items"), Some("Trapped Containers"));
        }
    } else if is("Server.Items.Bola") {
        (title, option) = (Some("Items"), Some("Bolas"));
    } else if is("Server.Items.OrangePetals") {
        (title, option) = (Some("Items"), Some("Orange Petals"));
    } else if is("Server.Mobiles.EtherealMount")
        || w.item(item)
            .is_some_and(|i| i.layer == muo_server::layer::Layer::Mount)
    {
        (title, option) = (Some("Items"), Some("Mounts"));
    } else if is("Server.Items.LeatherNinjaBelt") {
        (title, option) = (Some("Items"), Some("Shurikens"));
    } else if is("Server.Items.Fukiya") {
        (title, option) = (Some("Items"), Some("Fukiya Darts"));
    } else if is("Server.Items.FireHorn") {
        (title, option) = (Some("Items"), Some("Fire Horns"));
    } else if is("Server.Items.BaseWand") {
        (title, option) = (Some("Items"), Some("Wands"));
    }

    if title.is_some() && option.is_some() && c.started_begin_countdown && !c.started {
        send(w, from, "You may not use this item before the duel begins.");
        return false;
    }
    let sudden = c.is_sudden_death;
    if potion && !is("Server.Items.BaseExplosionPotion") && !refresh && sudden {
        mm::send_message_hue(w, from, 0x22, "You may not drink potions in sudden death.");
        return false;
    }
    if is("Server.Items.Bandage") && sudden {
        mm::send_message_hue(w, from, 0x22, "You may not use bandages in sudden death.");
        return false;
    }
    match (title, option) {
        (Some(t), Some(o)) if !c.ruleset.get_option(t, o) => {
            send(
                w,
                from,
                "The dueling ruleset prevents you from using this item.",
            );
            false
        }
        _ => true,
    }
}

/// `AllowSpecialAbility(from, name, message)` (static): `InstAllowSpecialAbility` of the player's context.
pub fn allow_special_ability(w: &mut World, from: MobileId, name: &str, message: bool) -> bool {
    let Some(c) = context(w, from) else {
        return true;
    };
    if !c.started_begin_countdown || eliminated_or_none(c, from) {
        return true;
    }
    if cant_do_anything(w, c, from) {
        return false;
    }
    if c.ruleset.get_option("Combat Abilities", name) {
        return true;
    }
    if message {
        send(
            w,
            from,
            "The dueling ruleset prevents you from using this combat ability.",
        );
    }
    false
}

/// `AllowSpecialMove(from, name, move)`: the title is "Bushido" for a `NinjaMove` and "Ninjitsu" for a `SamuraiMove`
/// (the wrong way round, kept as is; `FindByOption` falls through to the other school's page anyway).
pub fn allow_special_move(
    w: &mut World,
    from: MobileId,
    name: &str,
    ninja_move: Option<bool>,
) -> bool {
    let Some(c) = context(w, from) else {
        return true;
    };
    if !c.started_begin_countdown || eliminated_or_none(c, from) {
        return true;
    }
    if cant_do_anything(w, c, from) {
        return false;
    }
    let title = ninja_move.map(|n| if n { "Bushido" } else { "Ninjitsu" });
    if title.is_none_or(|t| c.ruleset.get_option(t, name)) {
        return true;
    }
    send(
        w,
        from,
        "The dueling ruleset prevents you from using this move.",
    );
    false
}

/// The class of a spell for `AllowSpellCast`: (title, option) of the ruleset; a title of `None` is always allowed.
fn spell_option(
    w: &World,
    spell: muo_server::interfaces::SpellId,
) -> Option<(Option<&'static str>, &'static str)> {
    use muo_content::spells::types::SpellCircle as C;
    use muo_content::spells::{magery, spell as sp};
    let ty = sp::spell_type(w, spell)?;
    let name = ty.info.name;
    let mut t = Some(ty);
    while let Some(x) = t {
        let title = match x.full_name {
            "Server.Spells.Spellweaving.ArcanistSpell" => Some("Spellweaving"),
            "Server.Spells.Chivalry.PaladinSpell" => Some("Chivalry"),
            "Server.Spells.Necromancy.NecromancerSpell" => Some("Necromancy"),
            "Server.Spells.Ninjitsu.NinjaSpell" => Some("Ninjitsu"),
            "Server.Spells.Bushido.SamuraiSpell" => Some("Bushido"),
            "Server.Spells.MagerySpell" => {
                return Some((
                    magery::circle(w, spell).map(|c| match c {
                        C::First => "1st Circle",
                        C::Second => "2nd Circle",
                        C::Third => "3rd Circle",
                        C::Fourth => "4th Circle",
                        C::Fifth => "5th Circle",
                        C::Sixth => "6th Circle",
                        C::Seventh => "7th Circle",
                        C::Eighth => "8th Circle",
                    }),
                    name,
                ));
            }
            _ => None,
        };
        if title.is_some() {
            return Some((title, name));
        }
        t = x.parent;
    }
    Some((Some("Other Spell"), name))
}

/// `AllowSpellCast(from, spell)`.
pub fn allow_spell_cast(
    w: &mut World,
    from: MobileId,
    spell: muo_server::interfaces::SpellId,
) -> bool {
    let Some(c) = context(w, from) else {
        return true;
    };
    if !c.started_begin_countdown || eliminated_or_none(c, from) {
        return true;
    }
    if cant_do_anything(w, c, from) {
        return false;
    }
    let is_recall = muo_content::spells::spell::spell_type(w, spell)
        .is_some_and(|t| t.full_name == "Server.Spells.Fourth.RecallSpell");
    if is_recall {
        send(w, from, "You may not cast this spell.");
    }
    let Some(c) = context(w, from) else {
        return true;
    };
    let allowed = match spell_option(w, spell) {
        Some((Some(title), option)) => c.ruleset.get_option(title, option),
        _ => true,
    };
    if allowed {
        return true;
    }
    send(
        w,
        from,
        "The dueling ruleset prevents you from casting this spell.",
    );
    false
}

/// `OnLocationChanged(mob)` / `OnMapChanged(mob)`: leaving the arena forfeits.
pub fn on_location_changed(w: &mut World, mob: MobileId) {
    let Some(id) = duel_context_of(w, mob) else {
        return;
    };
    let Some(c) = ctx(w, id) else { return };
    if !c.registered || !c.started_begin_countdown || c.finished {
        return;
    }
    let Some(arena_id) = c.arena else { return };
    let Some(a) = arena::data(w, arena_id) else {
        return;
    };
    let (facet, bounds, outside) = (a.facet, a.bounds, a.outside);
    let (map, loc) = w
        .mobile(mob)
        .map_or((None, Point3D::default()), |m| (m.map, m.location));
    if map == facet && bounds.contains_point(&loc) {
        return;
    }
    if eliminated_or_none(c, mob) {
        return;
    }
    if map.is_none() {
        // `mob.Map == Map.Internal` (logged out): the logout spot inside the arena moves outside.
        let (lmap, lloc) = w.mobile(mob).map_or((None, Point3D::default()), |m| {
            (m.logout_map, m.logout_location)
        });
        if lmap == facet
            && bounds.contains_point(&lloc)
            && let Some(m) = w.mobile_mut(mob)
        {
            m.logout_location = outside;
        }
    }
    set_eliminated(w, id, mob);
    mm::local_overhead_message(
        w,
        mob,
        muo_primitives::MessageType::REGULAR,
        0x22,
        false,
        "You have forfeited your position in the duel.",
    );
    let name = flow::name(w, mob);
    mm::nonlocal_overhead_message(
        w,
        mob,
        muo_primitives::MessageType::REGULAR,
        0x22,
        false,
        &format!("{name} has forfeited by leaving the dueling arena."),
    );
    if let Some(winner) = flow::check_completion(w, id) {
        flow::finish(w, id, winner);
    }
}

/// `Configure`: sets the hooks the duel rules answer.
pub fn configure(w: &mut World) {
    use muo_content::items::weapon_hooks::WeaponHooks;
    use muo_content::misc::notoriety::{DuelNoto, NotorietyHooks};
    use muo_content::player_mobile::hooks::{DuelState as PlayerDuelState, SystemHooks};
    use muo_content::spells::{hooks as spell_hooks, special_move::SpecialMoves};

    w.resources.get_mut::<DuelState>();
    ruleset::root(w);

    let h = w.resources.get_mut::<SystemHooks>();
    h.duel_state = Some(|w, m| {
        let c = context(w, m)?;
        let p = c.find(m);
        Some(PlayerDuelState {
            registered: c.registered,
            started: c.started,
            finished: c.finished,
            tournament: c.tournament.is_some(),
            eliminated: p.is_none_or(|p| p.eliminated),
            has_player: p.is_some(),
        })
    });
    h.duel_allow_item_use = Some(allow_item_use);
    h.duel_allow_item_equip = Some(allow_item_equip);
    h.duel_allow_skill_use = Some(allow_skill_use);
    h.duel_on_location_changed = Some(on_location_changed);
    h.duel_on_map_changed = Some(on_location_changed);
    h.in_safe_zone = Some(safezone::mobile_in_safe_zone);
    h.duel_same_context = Some(|w, a, b| duel_context_of(w, a) == duel_context_of(w, b));

    let wh = w.resources.get_mut::<WeaponHooks>();
    wh.duel_check_item_equip =
        Some(|w, m, item| context(w, m).is_none_or(|c| check_item_equip(w, c, item)));
    wh.duel_allow_special_ability = Some(allow_special_ability);

    let n = w.resources.get_mut::<NotorietyHooks>();
    n.in_safe_zone = Some(safezone::mobile_in_safe_zone);
    n.duel_standing = Some(|w, from, target| {
        let c = context(w, from)?;
        let same = duel_context_of(w, from) == duel_context_of(w, target);
        (c.started_begin_countdown && !c.finished && same).then(|| {
            if c.is_ally(from, target) {
                DuelNoto::Ally
            } else {
                DuelNoto::Enemy
            }
        })
    });
    n.in_started_duel = Some(|w, m| context(w, m).is_some_and(|c| c.started));
    n.duel_allow = Some(|w, from, target, beneficial| {
        let (cf, ct) = (duel_context_of(w, from), duel_context_of(w, target));
        let (a, b) = (context(w, from), context(w, target));
        if cf != ct && (a.is_some_and(|c| c.started) || b.is_some_and(|c| c.started)) {
            return Some(false);
        }
        let shared = cf.is_some() && cf == ct;
        if let (Some(a), true) = (a, shared) {
            let elim = |m| a.find(m).is_none_or(|p| p.eliminated);
            if (a.started_ready_countdown && !a.started) || a.tied || elim(from) || elim(target) {
                return Some(false);
            }
        }
        // Only `Mobile_AllowBeneficial` has the sudden death rule.
        if beneficial
            && a.is_some_and(|c| c.find(from).is_some_and(|p| !p.eliminated) && c.is_sudden_death)
        {
            return Some(false);
        }
        if let (Some(a), true) = (a, shared)
            && a.tournament == Some(true)
            && let (Some(pf), Some(pt)) = (a.participant_of(from), a.participant_of(target))
            && (pf != pt) == beneficial
        {
            return Some(false);
        }
        if a.is_some_and(|c| c.started) && shared {
            return Some(true);
        }
        None
    });

    let s = spell_hooks::hooks_mut(w);
    // `SpellHelper.IsSafeZone`: in a SafeZone and `(caster as PlayerMobile)?.DuelPlayer?.Eliminated == false`
    s.safe_zone_blocks_travel = Some(|w, map, loc, caster| {
        let r = muo_server::region::find(w, loc, Some(map));
        muo_server::region::is_part_of_type(w, r, &safezone::SAFE_ZONE)
            && caster.is_some_and(|c| {
                muo_content::player_mobile::data(w, c).is_some()
                    && context(w, c)
                        .and_then(|x| x.find(c))
                        .is_some_and(|p| !p.eliminated)
            })
    });
    s.duel_is_free_consume = Some(is_free_consume);
    s.duel_allow_spell_cast = Some(allow_spell_cast);
    s.duel_check_sudden_death = Some(|w, m| {
        // `CheckSuddenDeath(mob)`: the duel is in sudden death and `mob` is still in it.
        context(w, m).is_some_and(|c| c.is_sudden_death && c.find(m).is_some_and(|p| !p.eliminated))
    });
    s.duel_allows_skill = Some(|w, m, skill| {
        context(w, m)
            .is_none_or(|c| !c.started || c.finished || c.ruleset.get_option("Skills", skill))
    });
    s.duel_opponents = Some(|w, a, b| {
        let shared =
            duel_context_of(w, a).is_some() && duel_context_of(w, a) == duel_context_of(w, b);
        let c = context(w, a)?;
        if !shared || !c.started {
            return None;
        }
        Some(c.participant_of(a)? != c.participant_of(b)?)
    });
    s.duel_in_progress_for = Some(|w, m| {
        context(w, m).is_some_and(|c| c.started && c.find(m).is_some_and(|p| !p.eliminated))
    });

    w.resources
        .get_mut::<SpecialMoves>()
        .duel_allow_special_move = Some(allow_special_move);

    muo_content::player_mobile::subscribe_death(w, flow::on_player_death);
    speech::configure(w);
    arena::configure(w);
    trophy::configure(w);
    teleporter_addon::configure(w);
    props::configure(w);
}

/// Registers the item classes of the system.
pub fn register(types: &mut muo_server::type_registry::TypeRegistry) {
    arena::register(types);
    teleporter_addon::register(types);
    ladder::register(types);
    types.register_item(&ladder_gump::LADDER_ITEM, &[]);
    trophy::register(types);
    tournament::register(types);
    arena_gump::register(types);
    spectator::register(types);
    ctf::register(types);
    dd::register(types);
    koth::register(types);
    bombing::register(types);
}

#[cfg(test)]
mod tests {
    use super::*;
    use muo_server::mobile::MOBILE_BASE;

    fn pl(m: MobileId) -> Option<DuelPlayer> {
        Some(DuelPlayer {
            mobile: m,
            ready: true,
            eliminated: false,
        })
    }

    #[test]
    fn pvp_no_duel_means_everything_allowed() {
        let mut w = World::new();
        let m = w.new_mobile(&MOBILE_BASE);
        assert!(duel_context_of(&w, m).is_none());
        assert!(allow_skill_use(&mut w, m, SkillName::Magery));
        assert!(allow_special_ability(&mut w, m, "Disarm", true));
        assert!(!is_free_consume(&w, m));
    }

    #[test]
    fn pvp_started_duel_sides_and_ruleset() {
        let mut w = World::new();
        let (a, b, c) = (
            w.new_mobile(&MOBILE_BASE),
            w.new_mobile(&MOBILE_BASE),
            w.new_mobile(&MOBILE_BASE),
        );
        let mut ctx = DuelContext {
            started_begin_countdown: true,
            started: true,
            ..DuelContext::default()
        };
        ctx.participants = vec![
            Participant {
                uid: 1,
                players: vec![pl(a)],
                tourney_part: None,
            },
            Participant {
                uid: 2,
                players: vec![pl(b)],
                tourney_part: None,
            },
        ];
        ctx.ruleset.set_option("Combat Abilities", "Disarm", false);
        w.resources.get_mut::<DuelState>().add(ctx);

        assert!(
            !allow_special_ability(&mut w, a, "Disarm", false),
            "the ruleset forbids it"
        );
        assert!(
            allow_special_ability(&mut w, a, "No Such Ability", false),
            "an unlisted option is allowed"
        );
        assert!(
            !allow_special_ability(&mut w, a, "Stun", false),
            "Stun is listed now and a fresh ruleset has every option off"
        );
        assert!(
            allow_special_ability(&mut w, c, "Disarm", false),
            "not in the duel"
        );
        let ctx = context(&w, a).unwrap();
        assert!(!ctx.is_ally(a, b));
        assert!(ctx.is_ally(a, a));
    }

    #[test]
    fn pvp_participant_slots() {
        let mut p = Participant::new(3);
        assert!(p.has_open_slot() && p.eliminated());
        let mut w = World::new();
        let m = w.new_mobile(&MOBILE_BASE);
        p.players[2] = pl(m);
        p.resize(2);
        assert_eq!(p.filled_slots(), 1);
        assert!(p.players[0].is_some(), "filled slots move to the front");
        p.nullify(m);
        assert_eq!(p.filled_slots(), 0);
    }
}
