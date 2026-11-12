//! The ethics system (`Ethic`, `EthicsEntity`, `EthicsSystem`, `Player`, `Power`) and the two ethics
//! (Hero, Evil) with their definitions.
//!
//! The system keeps a static `Ethics => [null, null]`, a property that returns a new array on every read, so
//! `Ethic.Hero` and `Ethic.Evil` are always null: nobody can join (the speech handler loops over nulls), item
//! checks never find an ethic (`Find(item)` answers null whatever the flags), and `Hero`/`Evil` used inside a power
//! throw. Those bugs stay: [`hero`] and [`evil`] answer `None`. A `Player` only has an effect when it was attached,
//! which only the pre-version-1 `Ethic` record does on load (`Timer.StartTimer(pl.CheckAttach)`).
//!
//! The entities are saved by `EthicsSystem`, a `GenericEntityPersistence<EthicsEntity>("Ethics", 3, 0x1,
//! 0x7FFFFFFF)`: [`persist`].
//!
//! `HolySteed` and `UnholySteed` are `BaseMount`s ([`mobiles`]); the powers summon them for an hour.

pub mod legacy;
pub mod mobiles;
pub mod persist;
pub mod powers;

use muo_primitives::dotnet::IgnoreCase;
use std::collections::HashMap;

use indexmap::IndexMap;
use muo_primitives::{DateTime, Serial, TimeSpan};
use muo_server::World;
use muo_server::ids::{ItemId, MobileId};
use muo_server::mobile::part2::messages as mm;

use crate::factions;
use powers::Power;

/// Which ethic a record is.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EthicKind {
    Hero,
    Evil,
}

impl EthicKind {
    pub fn type_name(self) -> &'static str {
        match self {
            EthicKind::Hero => "Server.Ethics.Hero.HeroEthic",
            EthicKind::Evil => "Server.Ethics.Evil.EvilEthic",
        }
    }

    /// `EthicDefinition.PrimaryHue`.
    pub fn primary_hue(self) -> i32 {
        match self {
            EthicKind::Hero => 0x482,
            EthicKind::Evil => 0x455,
        }
    }

    /// `EthicDefinition.Title`.
    pub fn title(self) -> &'static str {
        match self {
            EthicKind::Hero => "Hero",
            EthicKind::Evil => "Evil",
        }
    }

    /// `EthicDefinition.Adjunct.String`.
    pub fn adjunct(self) -> &'static str {
        match self {
            EthicKind::Hero => "(Hero)",
            EthicKind::Evil => "(Evil)",
        }
    }

    /// `EthicDefinition.JoinPhrase.String`.
    pub fn join_phrase(self) -> &'static str {
        match self {
            EthicKind::Hero => "I will defend the virtues",
            EthicKind::Evil => "I am evil incarnate",
        }
    }

    /// `EthicDefinition.Powers`, in order.
    pub fn powers(self) -> &'static [Power] {
        match self {
            EthicKind::Hero => &powers::HERO_POWERS,
            EthicKind::Evil => &powers::EVIL_POWERS,
        }
    }

    /// `Ethic.IsEligible(mob)`.
    pub fn is_eligible(self, w: &World, mob: MobileId) -> bool {
        let f = factions::find(w, mob);
        match self {
            EthicKind::Hero => {
                !muo_server::mobile::base::murderer(w, mob)
                    && matches!(
                        f,
                        Some(factions::TRUE_BRITANNIANS | factions::COUNCIL_OF_MAGES)
                    )
            }
            EthicKind::Evil => matches!(f, Some(factions::MINAX | factions::SHADOWLORDS)),
        }
    }
}

/// `Ethic.Hero` (always null, see the module comment).
pub fn hero() -> Option<Serial> {
    None
}

/// `Ethic.Evil` (always null).
pub fn evil() -> Option<Serial> {
    None
}

/// An `Ethic` entity.
#[derive(Clone, Debug)]
pub struct EthicData {
    pub kind: EthicKind,
    pub created: DateTime,
    /// `Players`: the attached ones.
    pub players: Vec<Serial>,
}

/// An `Ethics.Player` entity.
#[derive(Clone, Debug)]
pub struct PlayerData {
    pub created: DateTime,
    pub mobile: Option<MobileId>,
    pub power: i32,
    pub history: i32,
    pub steed: Option<MobileId>,
    pub familiar: Option<MobileId>,
    pub shield: DateTime,
    pub ethic: Option<Serial>,
}

impl PlayerData {
    /// `new Player(ethic, mobile)`.
    pub fn new(ethic: Serial, mobile: Option<MobileId>, now: DateTime) -> PlayerData {
        PlayerData {
            created: now,
            mobile,
            power: 5,
            history: 5,
            steed: None,
            familiar: None,
            shield: DateTime::MIN,
            ethic: Some(ethic),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Entity {
    Ethic(EthicData),
    Player(PlayerData),
}

/// What `EthicsSystem` and the statics of `Ethic` / `Player` kept.
#[derive(Default)]
pub struct EthicsState {
    /// `Ethic.Enabled`.
    pub enabled: bool,
    /// `_persistence`'s entities, by serial, in creation order.
    pub entities: IndexMap<Serial, Entity>,
    pub last_serial: u32,
    /// `PlayerMobile.EthicPlayer`.
    pub attached: HashMap<MobileId, Serial>,
}

pub fn st(w: &World) -> Option<&EthicsState> {
    w.resources.get::<EthicsState>()
}

pub fn st_mut(w: &mut World) -> &mut EthicsState {
    w.resources.get_mut::<EthicsState>()
}

impl EthicsState {
    /// `EthicsSystem.NewProfile`: the next serial.
    pub fn new_serial(&mut self) -> Serial {
        self.last_serial += 1;
        Serial::new(self.last_serial)
    }

    pub fn player(&self, s: Serial) -> Option<&PlayerData> {
        match self.entities.get(&s)? {
            Entity::Player(p) => Some(p),
            Entity::Ethic(_) => None,
        }
    }

    pub fn player_mut(&mut self, s: Serial) -> Option<&mut PlayerData> {
        match self.entities.get_mut(&s)? {
            Entity::Player(p) => Some(p),
            Entity::Ethic(_) => None,
        }
    }

    pub fn ethic(&self, s: Serial) -> Option<&EthicData> {
        match self.entities.get(&s)? {
            Entity::Ethic(e) => Some(e),
            Entity::Player(_) => None,
        }
    }
}

/// `Player.Attach()`.
pub fn attach(w: &mut World, pl: Serial) {
    let (mobile, ethic) = match st(w).and_then(|s| s.player(pl)) {
        Some(p) => (p.mobile, p.ethic),
        None => return,
    };
    let is_player = mobile.is_some_and(|m| w.mobile(m).is_some_and(|x| x.player));
    let s = st_mut(w);
    if let (true, Some(m)) = (is_player, mobile) {
        s.attached.insert(m, pl);
    }
    if let Some(Entity::Ethic(e)) = ethic.and_then(|e| s.entities.get_mut(&e)) {
        e.players.push(pl);
    }
}

/// `Player.Detach()`.
pub fn detach(w: &mut World, pl: Serial) {
    let (mobile, ethic) = match st(w).and_then(|s| s.player(pl)) {
        Some(p) => (p.mobile, p.ethic),
        None => return,
    };
    let s = st_mut(w);
    if let Some(m) = mobile {
        s.attached.remove(&m);
    }
    if let Some(Entity::Ethic(e)) = ethic.and_then(|e| s.entities.get_mut(&e)) {
        e.players.retain(|p| *p != pl);
    }
}

/// `Player.CheckAttach()`.
pub fn check_attach(w: &mut World, pl: Serial) {
    let Some((Some(m), Some(ethic))) = st(w)
        .and_then(|s| s.player(pl))
        .map(|p| (p.mobile, p.ethic))
    else {
        return;
    };
    let kind = st(w).and_then(|s| s.ethic(ethic)).map(|e| e.kind);
    if kind.is_some_and(|k| k.is_eligible(w, m)) {
        attach(w, pl);
    }
}

/// `Player.Find(mob, inherit)`.
pub fn find_player_full(w: &mut World, mob: MobileId, inherit: bool) -> Option<Serial> {
    let target = if inherit && muo_content::creature::cd(w, mob).is_some() {
        muo_content::creature::control::get_master(w, mob).unwrap_or(mob)
    } else {
        mob
    };
    if !w.mobile(target).is_some_and(|m| m.player) {
        return None;
    }
    let pl = st(w)?.attached.get(&target).copied()?;
    let (ethic, mobile) = st(w)?.player(pl).map(|p| (p.ethic, p.mobile))?;
    let kind = st(w)?.ethic(ethic?)?.kind;
    if !mobile.is_some_and(|m| kind.is_eligible(w, m)) {
        st_mut(w).attached.remove(&target);
        return None;
    }
    Some(pl)
}

/// `Player.Find(mob)` for a read-only caller: the same answer without dropping an ineligible attachment.
pub fn find_player_ref(w: &World, mob: MobileId) -> Option<Serial> {
    if !w.mobile(mob).is_some_and(|m| m.player) {
        return None;
    }
    let pl = st(w)?.attached.get(&mob).copied()?;
    let p = st(w)?.player(pl)?;
    let kind = st(w)?.ethic(p.ethic?)?.kind;
    kind.is_eligible(w, p.mobile?).then_some(pl)
}

/// `Player.Find(mob)`.
pub fn find_player(w: &mut World, mob: MobileId) -> Option<Serial> {
    find_player_full(w, mob, false)
}

/// `Player.Power`.
pub fn power(w: &World, pl: Serial) -> i32 {
    st(w).and_then(|s| s.player(pl)).map_or(0, |p| p.power)
}

/// `Player.Power += dp; Player.History += dh`.
pub fn add_power(w: &mut World, pl: Serial, dp: i32, dh: i32) {
    if let Some(p) = st_mut(w).player_mut(pl) {
        p.power += dp;
        p.history += dh;
    }
}

/// `Player.IsShielded`: an expired shield is finished on the way.
pub fn is_shielded(w: &mut World, pl: Serial) -> bool {
    let Some(shield) = st(w).and_then(|s| s.player(pl)).map(|p| p.shield) else {
        return false;
    };
    if shield == DateTime::MIN {
        return false;
    }
    if w.now < shield + TimeSpan::from_hours(1) {
        return true;
    }
    finish_shield(w, pl);
    false
}

fn is_shielded_ref(w: &World, pl: Serial) -> bool {
    st(w)
        .and_then(|s| s.player(pl))
        .is_some_and(|p| p.shield != DateTime::MIN && w.now < p.shield + TimeSpan::from_hours(1))
}

/// `Player.BeginShield()`.
pub fn begin_shield(w: &mut World, pl: Serial) {
    let now = w.now;
    if let Some(p) = st_mut(w).player_mut(pl) {
        p.shield = now;
    }
}

/// `Player.FinishShield()`.
pub fn finish_shield(w: &mut World, pl: Serial) {
    if let Some(p) = st_mut(w).player_mut(pl) {
        p.shield = DateTime::MIN;
    }
}

/// `Ethic.Find(mob)`: the ethic of an attached player.
pub fn find_ethic(w: &World, mob: MobileId) -> Option<Serial> {
    let pl = find_player_ref(w, mob)?;
    st(w)?.player(pl)?.ethic
}

/// `Ethic.Find(item)`: with `Hero`/`Evil` null the answer is null for every item (see the module comment).
pub fn find_item_ethic(_w: &World, _item: ItemId) -> Option<Serial> {
    None
}

/// `Ethic.CheckTrade(from, to, newOwner, item)`.
pub fn check_trade(
    w: &mut World,
    from: MobileId,
    to: MobileId,
    new_owner: MobileId,
    item: ItemId,
) -> bool {
    let Some(item_ethic) = find_item_ethic(w, item) else {
        return true;
    };
    if find_ethic(w, new_owner) == Some(item_ethic) {
        return true;
    }
    let who = if from == new_owner { to } else { from };
    // `itemEthic == Hero` / `== Evil` cannot hold (both are null): no message.
    let _ = who;
    false
}

/// `Ethic.CheckEquip(from, item)`.
pub fn check_equip(w: &mut World, from: MobileId, item: ItemId) -> bool {
    let Some(item_ethic) = find_item_ethic(w, item) else {
        return true;
    };
    if find_ethic(w, from) == Some(item_ethic) {
        return true;
    }
    false
}

/// `Ethic.IsImbued(item)`.
pub fn is_imbued(w: &World, item: ItemId) -> bool {
    find_item_ethic(w, item).is_some()
}

/// `Ethic.IsImbued(item, true)`: the item or anything inside it.
pub fn is_imbued_recursive(w: &World, item: ItemId) -> bool {
    is_imbued(w, item)
        || w.item(item)
            .is_some_and(|i| i.items.iter().any(|&c| is_imbued_recursive(w, c)))
}

/// `Ethic.EventSink_Speech`.
fn on_speech(w: &mut World, e: &mut muo_server::events::SpeechEventArgs) {
    if e.blocked || e.handled {
        return;
    }
    let mob = e.mobile;
    let Some(pl) = find_player(w, mob) else {
        // `Ethics` is [null, null]: no ethic to join.
        return;
    };
    if crate::conpvp::duel_context_of(w, mob).is_some() {
        return;
    }
    let Some(kind) = st(w)
        .and_then(|s| s.player(pl))
        .and_then(|p| p.ethic)
        .and_then(|e| st(w).and_then(|s| s.ethic(e)))
        .map(|e| e.kind)
    else {
        return;
    };
    for power in kind.powers() {
        if !power.phrase.eq_ic(&e.speech) {
            continue;
        }
        if !power.check_invoke(w, pl) {
            continue;
        }
        power.begin_invoke(w, pl);
        e.handled = true;
        break;
    }
}

/// Where a `Mobile` message goes for a player's mobile.
pub(crate) fn player_mobile(w: &World, pl: Serial) -> Option<MobileId> {
    st(w).and_then(|s| s.player(pl)).and_then(|p| p.mobile)
}

pub(crate) fn say(w: &mut World, m: MobileId, text: &str) {
    mm::local_overhead_message(
        w,
        m,
        muo_primitives::MessageType::REGULAR,
        0x3B2,
        false,
        text,
    );
}

/// `EthicsSystem.Configure` / `Ethic.Configure` / `Ethic.Initialize`, and the hooks of the ethics.
pub fn configure(w: &mut World) {
    use muo_content::player_mobile::hooks as ph;

    let enabled = w
        .config
        .get_or_update_setting("ethics.enable", false)
        .unwrap_or(false);
    *st_mut(w) = EthicsState {
        enabled,
        last_serial: 0,
        ..EthicsState::default()
    };
    persist::register(w);

    let h = w.resources.get_mut::<ph::SystemHooks>();
    h.ethic_of_mobile = Some(|w, m| find_ethic(w, m).map(|e| e.value() as i32));
    h.ethic_of_item = Some(|w, item| find_item_ethic(w, item).map(|e| e.value() as i32));
    h.ethic_adjunct = Some(|w, m| {
        let e = find_player_ref(w, m).and_then(|pl| st(w)?.player(pl)?.ethic)?;
        st(w)?.ethic(e).map(|e| e.kind.adjunct().to_owned())
    });

    let e = w
        .resources
        .get_mut::<muo_content::items::equip_hooks::EquipHooks>();
    e.ethic_check_equip = Some(check_equip);
    e.ethic_check_trade = Some(check_trade);
    e.ethic_is_imbued = Some(is_imbued);
    w.resources
        .get_mut::<muo_mobiles::npcs::vendors::player_vendor::PlayerVendorHooks>()
        .is_imbued = Some(is_imbued_recursive);

    let c = muo_content::hooks::hooks_mut(w);
    // `GetEthicAllegiance`: the class's `EthicAllegiance` (Hero 0 / Evil 1) against the ethic of `mob` (or its master).
    c.ethic_allegiance = Some(|w, creature, mob| {
        use muo_content::creature::enums::Allegiance;
        let Some(side) = muo_content::creature::ctype(w, creature)
            .and_then(|c| (c.ethic_allegiance)(w, creature))
        else {
            return Allegiance::None;
        };
        if w.mobile(mob).and_then(|m| m.map) != Some(crate::factions::FACET) {
            return Allegiance::None;
        }
        let target = if muo_content::creature::cd(w, mob).is_some() {
            muo_content::creature::control::get_master(w, mob).unwrap_or(mob)
        } else {
            mob
        };
        let Some(kind) = find_ethic(w, target).and_then(|e| st(w)?.ethic(e).map(|e| e.kind)) else {
            return Allegiance::None;
        };
        let own = match kind {
            EthicKind::Hero => 0,
            EthicKind::Evil => 1,
        };
        if own == side {
            Allegiance::Ally
        } else {
            Allegiance::Enemy
        }
    });
    c.ethic_finish_shield = Some(finish_shield_of);
    c.ethic_shielded = Some(|w, m, ours| {
        find_player_ref(w, m).is_some_and(|pl| {
            let side = st(w).and_then(|s| {
                s.player(pl)?
                    .ethic
                    .and_then(|e| s.ethic(e))
                    .map(|e| match e.kind {
                        EthicKind::Hero => 0,
                        EthicKind::Evil => 1,
                    })
            });
            is_shielded_ref(w, pl) && (ours.is_none() || ours == side)
        })
    });
}

/// `Ethic.Initialize`: the speech handler, when the system is enabled.
pub fn initialize(w: &mut World) {
    if st(w).is_some_and(|s| s.enabled) {
        w.events.speech.subscribe(on_speech);
    }
}

/// `BaseCreature.AggressiveAction`'s tail: `Player.Find(aggressor, true)` finishes a shield that is up.
fn finish_shield_of(w: &mut World, aggressor: MobileId) {
    if let Some(pl) = find_player_full(w, aggressor, true)
        && is_shielded(w, pl)
    {
        finish_shield(w, pl);
    }
}
