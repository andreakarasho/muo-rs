//! `Mobile` properties, second part (`Luck` .. `PropertyList`): the non-virtual properties with
//! logic and the statics declared there. The virtual ones (`Luck`, `Hue`, `Name`, `Weapon`, `Criminal`,
//! `PropertyTitle`, `Str`/`Dex`/`Int`, `GetProperties`, ..) have their base bodies in `mobile::base`. See `part1.rs`.
//!
//! Statics declared in this range live in [`PropsBStatics`] (`w.resources`). Those `part2::statics::Part2Statics`
//! already holds (`DisableDismountInWarmode`, `ExpireCriminalDelay`, the regen rate handlers and default rates, the
//! `DisableHiddenSelfClick`/`AsciiClickMessage`/`GuildClickMessage` flags, the skill check handlers) are read from
//! there so the world keeps one copy; `AllowBeneficialHandler`/`AllowHarmfulHandler` are in `MapsSettings`.
//!
//! The content types the `Weapon`/`Mount` getters test (`is IWeapon`, `IMountItem`) are hooks in [`PropsBStatics`]
//! that content sets.

use muo_primitives::serial::{MAX_ITEM_SERIAL, MAX_VIRTUAL_SERIAL};
use muo_primitives::{Body, DateTime, Direction, Serial, TimeSpan};

use crate::events;
use crate::guild::GuildId;
use crate::ids::{EntityId, ItemId, MobileId};
use crate::interfaces::WeaponRef;
use crate::layer::Layer;
use crate::map::MapId;
use crate::mobile::part2;
use crate::mobile::{AccessLevel, MobileDelta, StatType};
use crate::net_state::NetStateId;
use crate::poison::PoisonId;
use crate::prompt::PromptId;
use crate::property_list::PropertyList;
use crate::region::RegionId;
use crate::timer::{self, TimerHandle};
use crate::world::World;

/// `Mobile.AOSStatusHandler(from, index)`.
pub type AosStatusHandler = fn(&World, MobileId, i32) -> i32;

/// The `Mobile` statics of this part that nothing else holds, plus the content hooks.
pub struct PropsBStatics {
    /// `BodyWeight` (11 + 3 for the backpack).
    pub body_weight: i32,
    /// `OldPropertyTitles`.
    pub old_property_title: bool,
    /// `AOSStatusHandler`.
    pub aos_status_handler: Option<AosStatusHandler>,
    /// `World._nextVirtualSerial`.
    pub next_virtual_serial: u32,
    /// `item is IWeapon`.
    pub item_is_weapon: Option<fn(&World, ItemId) -> bool>,
    /// `item is IMountItem mountItem` then `mountItem.Mount`: the mount mobile an item stands for.
    pub mount_of_item: Option<fn(&World, ItemId) -> Option<MobileId>>,
    /// `new BankBox(this)`: makes the bank box item (not yet added) for the mobile.
    pub create_bank_box: Option<fn(&mut World, MobileId) -> Option<ItemId>>,
    /// `Weapon.GetStatusDamage(this, out min, out max)` of `Mobile.Weapon` (fists when nothing is wielded).
    #[allow(clippy::type_complexity)]
    pub weapon_status_damage: Option<fn(&mut World, MobileId) -> (i32, i32)>,
    /// `weapon.OnBeforeSwing`, `RevealingAction`, `NextCombatTime = ..OnSwing` at the end of `CheckCombatTime`.
    pub combat_swing: Option<fn(&mut World, MobileId, MobileId)>,
}

impl Default for PropsBStatics {
    fn default() -> Self {
        PropsBStatics {
            body_weight: 11,
            old_property_title: false,
            aos_status_handler: None,
            next_virtual_serial: MAX_ITEM_SERIAL,
            item_is_weapon: None,
            mount_of_item: None,
            create_bank_box: Some(|w, m| Some(crate::item::containers::new_bank_box(w, Some(m)))),
            weapon_status_damage: None,
            combat_swing: None,
        }
    }
}

/// `Mobile.BodyWeight`.
pub fn body_weight(w: &World) -> i32 {
    w.resources
        .get::<PropsBStatics>()
        .map_or(11, |s| s.body_weight)
}

/// `Mobile.BodyWeight = value`.
pub fn set_body_weight(w: &mut World, value: i32) {
    w.resources.get_mut::<PropsBStatics>().body_weight = value;
}

/// `Mobile.OldPropertyTitles`.
pub fn old_property_titles(w: &World) -> bool {
    w.resources
        .get::<PropsBStatics>()
        .is_some_and(|s| s.old_property_title)
}

/// `Mobile.OldPropertyTitles = value`.
pub fn set_old_property_titles(w: &mut World, value: bool) {
    w.resources.get_mut::<PropsBStatics>().old_property_title = value;
}

/// `Mobile.AOSStatusHandler`.
pub fn aos_status_handler(w: &World) -> Option<AosStatusHandler> {
    w.resources
        .get::<PropsBStatics>()
        .and_then(|s| s.aos_status_handler)
}

/// `Mobile.AOSStatusHandler = handler`.
pub fn set_aos_status_handler(w: &mut World, handler: Option<AosStatusHandler>) {
    w.resources.get_mut::<PropsBStatics>().aos_status_handler = handler;
}

/// `Mobile.ExpireCriminalDelay`.
pub fn expire_criminal_delay(w: &World) -> TimeSpan {
    part2::statics::read(w, |s| s.expire_criminal_delay)
}

/// `Mobile.DisableDismountInWarmode`.
pub fn disable_dismount_in_warmode(w: &World) -> bool {
    part2::statics::read(w, |s| s.disable_dismount_in_warmode)
}

/// `Mobile.DefaultHitsRate`.
pub fn default_hits_rate(w: &World) -> TimeSpan {
    part2::statics::read(w, |s| s.default_hits_rate)
}

/// `World.NewVirtual`: the next serial of the virtual range, which restarts at `ResetVirtualSerial`.
pub fn new_virtual_serial(w: &mut World) -> Serial {
    let s = w.resources.get_mut::<PropsBStatics>();
    let value = if s.next_virtual_serial > MAX_VIRTUAL_SERIAL {
        MAX_ITEM_SERIAL
    } else {
        s.next_virtual_serial
    };
    s.next_virtual_serial = value + 1;
    Serial::new(value)
}

/// `Delta(flag)`, which is virtual.
pub(crate) fn delta(w: &mut World, id: MobileId, flag: MobileDelta) {
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.delta)(w, id, flag);
    }
}

pub(crate) fn invalidate_properties(w: &mut World, id: MobileId) {
    part2::properties::invalidate_properties(w, id);
}

pub(crate) fn start_hits_timer(w: &mut World, id: MobileId) {
    part2::stats::start_regen_timer(w, id, part2::stats::Regen::Hits);
}

pub(crate) fn start_stam_timer(w: &mut World, id: MobileId) {
    part2::stats::start_regen_timer(w, id, part2::stats::Regen::Stam);
}

pub(crate) fn start_mana_timer(w: &mut World, id: MobileId) {
    part2::stats::start_regen_timer(w, id, part2::stats::Regen::Mana);
}

pub(crate) fn get_stat_offset(w: &World, id: MobileId, ty: StatType) -> i32 {
    part2::stats::get_stat_offset(w, id, ty)
}

fn parent_is(w: &World, item: ItemId, id: MobileId) -> bool {
    w.item(item)
        .is_some_and(|i| !i.deleted() && i.parent == Some(EntityId::Mobile(id)))
}

pub fn set_hue_mod(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.hue_mod != value {
        m.hue_mod = value;
        delta(w, id, MobileDelta::HUE);
    }
}

pub fn set_direction(w: &mut World, id: MobileId, value: Direction) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.direction != value {
        m.direction = value;
        delta(w, id, MobileDelta::DIRECTION);
    }
}

pub fn set_female(w: &mut World, id: MobileId, value: bool) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.female != value {
        m.female = value;
        let ty = m.ty;
        delta(w, id, MobileDelta::FLAGS);
        (ty.on_gender_changed)(w, id, !value);
    }
}

pub fn set_flying(w: &mut World, id: MobileId, value: bool) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.flying != value {
        m.flying = value;
        delta(w, id, MobileDelta::FLAGS);
    }
}

pub fn set_warmode(w: &mut World, id: MobileId, value: bool) {
    let Some(m) = w.mobile(id) else { return };
    if m.deleted {
        return;
    }

    if m.warmode != value {
        let ty = m.ty;
        let mut t = m.auto_manifest_timer;
        t.cancel(w);
        let Some(m) = w.mobile_mut(id) else { return };
        m.auto_manifest_timer = t;
        m.warmode = value;
        let ns = m.net_state;
        delta(w, id, MobileDelta::FLAGS);

        if let Some(ns) = ns {
            crate::packets::combat::send_set_war_mode(w, ns, value);
        }

        if !value {
            (ty.set_combatant)(w, id, None);
        }

        if !(ty.alive)(w, id) {
            if value {
                delta(w, id, MobileDelta::GHOST_UPDATE);
            } else {
                part2::effects::send_remove_packet(w, id, false);
            }
        }

        (ty.on_warmode_changed)(w, id);
    }
}

pub fn set_hidden(w: &mut World, id: MobileId, value: bool) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.hidden != value {
        m.hidden = value;
        let ty = m.ty;
        // No `Delta(MobileDelta.Flags)` here.
        (ty.on_hidden_changed)(w, id);
    }
}

/// The connection when it is connected, else `None`.
pub fn net_state(w: &World, id: MobileId) -> Option<NetStateId> {
    let ns = w.mobile(id)?.net_state?;
    (!w.cannot_send_packets(ns)).then_some(ns)
}

pub fn set_net_state(w: &mut World, id: MobileId, value: Option<NetStateId>) {
    let Some(m) = w.mobile(id) else { return };
    if m.net_state == value {
        return;
    }
    let ty = m.ty;

    let (map, old_ns, target) = (m.map, m.net_state, m.target);
    if let Some(map) = map {
        crate::map::sector::on_client_change(w, map, old_ns, value, id);
    }
    if let Some(t) = target {
        crate::target::cancel(w, t, id, crate::target::TargetCancelType::Disconnected);
    }
    part2::deps::spell_on_connection_changed(w, id);
    if let Some(ns) = old_ns {
        part2::deps::cancel_all_trades(w, ns);
    }
    part2::items::close_bank_box(w, id);

    (ty.on_before_disconnected)(w, id);
    events::invoke(w, |e| &mut e.before_disconnected, &mut { id });

    let Some(m) = w.mobile_mut(id) else { return };
    m.net_state = value;
    let mut logout = m.logout_timer;
    logout.cancel(w);
    if let Some(m) = w.mobile_mut(id) {
        m.logout_timer = logout;
    }

    if value.is_none() {
        (ty.on_disconnected)(w, id);
        events::invoke(w, |e| &mut e.disconnected, &mut { id });

        // Disconnected, start the logout timer
        let delay = (ty.get_logout_delay)(w, id);
        let h = timer::start(w, delay, TimeSpan::ZERO, 1, move |w| {
            part2::stats::logout(w, id);
        });
        if let Some(m) = w.mobile_mut(id) {
            m.logout_timer = h;
        }
    } else {
        (ty.on_connected)(w, id);
        events::invoke(w, |e| &mut e.connected, &mut { id });

        let (internal, logout_map, logout_location) = w
            .mobile(id)
            .map(|m| {
                (
                    m.map == Some(MapId::INTERNAL),
                    m.logout_map,
                    m.logout_location,
                )
            })
            .unwrap_or_default();
        if internal && let Some(map) = logout_map {
            crate::mobile::part1::lifecycle::set_map(w, id, Some(map));
            crate::mobile::part1::lifecycle::set_location(w, id, logout_location);
        }
    }

    let mut i = w.mobile(id).map_or(0, |m| m.items.len());
    while i > 0 {
        i -= 1;
        let Some(item) = w.mobile(id).and_then(|m| m.items.get(i).copied()) else {
            continue;
        };
        if !w.item(item).is_some_and(|it| {
            it.ty
                .is(&crate::item::secure_trade_container::SECURE_TRADE_CONTAINER)
        }) {
            continue;
        }
        let mut j = w.item(item).map_or(0, |it| it.items.len());
        while j > 0 {
            j -= 1;
            let Some(inner) = w.item(item).and_then(|it| it.items.get(j).copied()) else {
                continue;
            };
            if let Some(ity) = w.item(inner).map(|it| it.ty) {
                (ity.on_secure_trade)(w, inner, id, id, id, false);
            }
            part2::items::add_to_backpack(w, id, inner);
        }
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            if let Some(ty) = w.item(item).map(|it| it.ty) {
                (ty.delete)(w, item);
            }
        });
    }

    part2::items::drop_holding(w, id);
    (ty.on_net_state_changed)(w, id);
}

pub fn set_guild_title(w: &mut World, id: MobileId, value: Option<String>) {
    let Some(m) = w.mobile_mut(id) else { return };
    let old = m.guild_title.clone();

    if old != value {
        m.guild_title = value.clone();
        let guild = m.guild;
        let ty = m.ty;

        if guild.is_some_and(|g| crate::guild::guild(w, g).is_some_and(|g| !g.disbanded))
            && let Some(title) = value.as_deref()
        {
            // Your guild title has changed :
            part2::messages::send_localized_message_affix(w, id, 1018026, true, title, "", 0x3B2);
        }

        invalidate_properties(w, id);

        (ty.on_guild_title_change)(w, id, old);
    }
}

pub fn set_display_guild_title(w: &mut World, id: MobileId, value: bool) {
    if let Some(m) = w.mobile_mut(id) {
        m.display_guild_title = value;
    }
    invalidate_properties(w, id);
}

pub fn set_name_mod(w: &mut World, id: MobileId, value: Option<String>) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.name_mod != value {
        m.name_mod = value;
        delta(w, id, MobileDelta::NAME);
        invalidate_properties(w, id);
    }
}

pub fn set_yellow_healthbar(w: &mut World, id: MobileId, value: bool) {
    if let Some(m) = w.mobile_mut(id) {
        m.yellow_healthbar = value;
    }
    delta(w, id, MobileDelta::HEALTHBAR_YELLOW);
}

pub fn raw_name(w: &World, id: MobileId) -> Option<String> {
    w.mobile(id).and_then(|m| m.name.clone())
}

/// `Name.get` (virtual: NameMod, a creature's DefaultName).
pub fn name(w: &World, id: MobileId) -> Option<String> {
    w.mobile(id).and_then(|m| (m.ty.name)(w, id))
}

/// `RawName.set` (`Name = value`, virtual).
pub fn set_raw_name(w: &mut World, id: MobileId, value: Option<String>) {
    if let Some(ty) = w.mobile(id).map(|m| m.ty) {
        (ty.set_name)(w, id, value);
    }
}

/// The latest of the three gains.
pub fn last_stat_gain(w: &World, id: MobileId) -> DateTime {
    let Some(m) = w.mobile(id) else {
        return DateTime::MIN;
    };
    m.last_str_gain.max(m.last_int_gain).max(m.last_dex_gain)
}

pub fn set_last_stat_gain(w: &mut World, id: MobileId, value: DateTime) {
    if let Some(m) = w.mobile_mut(id) {
        m.last_str_gain = value;
        m.last_int_gain = value;
        m.last_dex_gain = value;
    }
}

pub fn set_guild(w: &mut World, id: MobileId, value: Option<GuildId>) {
    let Some(m) = w.mobile(id) else { return };
    let old = m.guild;

    if old != value {
        if value.is_none() {
            set_guild_title(w, id, None);
        }

        let Some(m) = w.mobile_mut(id) else { return };
        m.guild = value;
        let ty = m.ty;

        delta(w, id, MobileDelta::NOTO);
        invalidate_properties(w, id);

        (ty.on_guild_change)(w, id, old);
    }
}

pub fn poisoned(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.poison.is_some())
}

pub fn is_body_mod(w: &World, id: MobileId) -> bool {
    w.mobile(id).is_some_and(|m| m.body_mod.body_id() != 0)
}

pub fn set_body_mod(w: &mut World, id: MobileId, value: Body) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.body_mod != value {
        m.body_mod = value;
        let ty = m.ty;

        delta(w, id, MobileDelta::BODY);
        invalidate_properties(w, id);

        (ty.check_stat_timers)(w, id);
    }
}

pub fn body(w: &World, id: MobileId) -> Body {
    w.mobile(id).map_or(Body::default(), |m| {
        if m.body_mod.body_id() != 0 {
            m.body_mod
        } else {
            m.body
        }
    })
}

pub fn set_body(w: &mut World, id: MobileId, value: Body) {
    let Some(m) = w.mobile(id) else { return };
    if m.body != value && m.body_mod.body_id() == 0 {
        let ty = m.ty;
        let safe = (ty.safe_body)(w, id, i32::from(value));
        if let Some(m) = w.mobile_mut(id) {
            m.body = Body::from(safe);
        }

        delta(w, id, MobileDelta::BODY);
        invalidate_properties(w, id);

        (ty.check_stat_timers)(w, id);
    }
}

/// `m_Region ?? (Map == null ? Map.Internal.DefaultRegion : Map.DefaultRegion)`; a null
/// `m_Region` is `RegionId(0)`.
pub fn region(w: &World, id: MobileId) -> RegionId {
    let Some(m) = w.mobile(id) else {
        return RegionId::default();
    };
    if m.region != RegionId::default() {
        return m.region;
    }
    crate::region::default_region(w, Some(m.map.unwrap_or(crate::map::MapId::INTERNAL)))
}

pub fn set_solid_hue_override(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.solid_hue_override == value {
        return;
    }
    m.solid_hue_override = value;
    delta(w, id, MobileDelta::HUE | MobileDelta::BODY);
}

/// The bank box, made (and equipped) when there is none.
pub fn bank_box(w: &mut World, id: MobileId) -> Option<ItemId> {
    // `m_BankBox?.Deleted == false && m_BankBox.Parent == this` else `FindItemOnLayer<BankBox>(Layer.Bank)`
    if let Some(b) = part2::items::find_bank_no_create(w, id) {
        return Some(b);
    }

    let make = w
        .resources
        .get::<PropsBStatics>()
        .and_then(|s| s.create_bank_box);
    if let Some(make) = make
        && let Some(b) = make(w, id)
    {
        if let Some(m) = w.mobile_mut(id) {
            m.bank_box = Some(b);
        }
        part2::items::add_item(w, id, b);
    }

    w.mobile(id)?.bank_box
}

pub fn backpack(w: &mut World, id: MobileId) -> Option<ItemId> {
    let cur = w.mobile(id)?.backpack;
    if !cur.is_some_and(|b| parent_is(w, b, id)) {
        // FindItemOnLayer<Container>: any item on the backpack layer is taken to be the container.
        let found = part2::items::find_item_on_layer(w, id, Layer::Backpack);
        if let Some(m) = w.mobile_mut(id) {
            m.backpack = found;
        }
    }
    w.mobile(id)?.backpack
}

pub fn has_trade(w: &World, id: MobileId) -> bool {
    w.mobile(id)
        .and_then(|m| m.net_state)
        .and_then(|ns| w.net_state(ns))
        .and_then(|d| d.trades.as_ref())
        .is_some_and(|t| !t.is_empty())
}

pub fn set_kills(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    let old_value = m.kills;

    if m.kills != value {
        m.kills = value.max(0);
        let ty = m.ty;

        if (old_value >= 5) != (ty.murderer)(w, id) {
            delta(w, id, MobileDelta::NOTO);
            invalidate_properties(w, id);
        }

        (ty.on_kills_change)(w, id, old_value);
    }
}

/// The mount of the mount item.
pub fn mount(w: &mut World, id: MobileId) -> Option<MobileId> {
    let cached = w.mobile(id)?.mount_item;
    let item = match cached {
        Some(mi) if parent_is(w, mi, id) => mi,
        _ => part2::items::find_item_on_layer(w, id, Layer::Mount)?,
    };

    let f = w.resources.get::<PropsBStatics>()?.mount_of_item?;
    let mount = f(w, item)?;
    if let Some(m) = w.mobile_mut(id) {
        m.mount_item = Some(item);
    }
    Some(mount)
}

pub fn mounted(w: &mut World, id: MobileId) -> bool {
    mount(w, id).is_some()
}

pub fn set_stat_cap(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.stat_cap != value {
        m.stat_cap = value;
        delta(w, id, MobileDelta::STAT_CAP);
    }
}

pub fn can_hear_ghosts(w: &World, id: MobileId) -> bool {
    w.mobile(id)
        .is_some_and(|m| m.can_hear_ghosts || m.access_level >= AccessLevel::Counselor)
}

pub fn raw_stat_total(w: &World, id: MobileId) -> i32 {
    w.mobile(id).map_or(0, |m| m.str + m.dex + m.int)
}

pub fn set_prompt(w: &mut World, id: MobileId, value: Option<PromptId>) {
    let Some(m) = w.mobile_mut(id) else { return };
    let old = m.prompt;
    if old == value {
        return;
    }
    m.prompt = None;
    let ns = m.net_state;

    // TODO: Cancel the prompt anyway?
    if value.is_some()
        && let Some(old) = old
    {
        crate::prompt::on_cancel(w, old, id);
        crate::prompt::remove_prompt(w, old);
    }

    if let Some(m) = w.mobile_mut(id) {
        m.prompt = value;
    }
    if let Some(ns) = ns {
        crate::packets::message::send_prompt(
            w,
            ns,
            value.map(|p| muo_primitives::Serial::new(p.0)),
        );
    }
}

pub fn set_raw_str(w: &mut World, id: MobileId, value: i32) {
    let value = value.clamp(1, 65000);
    let Some(m) = w.mobile_mut(id) else { return };
    if m.str != value {
        let old_value = m.str;
        m.str = value;
        let ty = m.ty;
        delta(w, id, MobileDelta::STAT | MobileDelta::HITS);

        let hits = w.mobile(id).map_or(0, |m| m.hits);
        let max = (ty.hits_max)(w, id);
        if hits < max {
            start_hits_timer(w, id);
        } else if hits > max {
            set_hits(w, id, max);
        }

        (ty.on_raw_str_change)(w, id, old_value);
        (ty.on_raw_stat_change)(w, id, StatType::STR, old_value);
    }
}

pub fn set_raw_dex(w: &mut World, id: MobileId, value: i32) {
    let value = value.clamp(1, 65000);
    let Some(m) = w.mobile_mut(id) else { return };
    if m.dex != value {
        let old_value = m.dex;
        m.dex = value;
        let ty = m.ty;
        delta(w, id, MobileDelta::STAT | MobileDelta::STAM);

        let stam = w.mobile(id).map_or(0, |m| m.stam);
        let max = (ty.stam_max)(w, id);
        if stam < max {
            start_stam_timer(w, id);
        } else if stam > max {
            set_stam(w, id, max);
        }

        (ty.on_raw_dex_change)(w, id, old_value);
        (ty.on_raw_stat_change)(w, id, StatType::DEX, old_value);
    }
}

pub fn set_raw_int(w: &mut World, id: MobileId, value: i32) {
    let value = value.clamp(1, 65000);
    let Some(m) = w.mobile_mut(id) else { return };
    if m.int != value {
        let old_value = m.int;
        m.int = value;
        let ty = m.ty;
        delta(w, id, MobileDelta::STAT | MobileDelta::MANA);

        let mana = w.mobile(id).map_or(0, |m| m.mana);
        let max = (ty.mana_max)(w, id);
        if mana < max {
            start_mana_timer(w, id);
        } else if mana > max {
            set_mana(w, id, max);
        }

        (ty.on_raw_int_change)(w, id, old_value);
        (ty.on_raw_stat_change)(w, id, StatType::INT, old_value);
    }
}

pub fn set_hits(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile(id) else { return };
    if m.deleted {
        return;
    }
    let ty = m.ty;

    let max = (ty.hits_max)(w, id);
    let value = value.clamp(0, max);

    if value == max {
        part2::stats::stop_regen_timer(w, id, part2::stats::Regen::Hits);

        if let Some(m) = w.mobile_mut(id) {
            for a in &mut m.aggressors {
                // reset reports on full HP
                a.can_report_murder = false;
            }
        }
        // reset damage entries on full HP
        part2::damage::clear_damage_entries(w, id);
    } else if (ty.can_regen_hits)(w, id) {
        start_hits_timer(w, id);
    } else {
        part2::stats::stop_regen_timer(w, id, part2::stats::Regen::Hits);
    }

    let Some(m) = w.mobile_mut(id) else { return };
    if m.hits != value {
        let old_value = m.hits;
        m.hits = value;
        delta(w, id, MobileDelta::HITS);
        (ty.on_hits_change)(w, id, old_value);
    }
}

pub fn set_stam(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile(id) else { return };
    if m.deleted {
        return;
    }
    let ty = m.ty;

    let max = (ty.stam_max)(w, id);
    let value = value.clamp(0, max);

    if (ty.can_regen_stam)(w, id) && value < max {
        start_stam_timer(w, id);
    } else {
        part2::stats::stop_regen_timer(w, id, part2::stats::Regen::Stam);
    }

    let Some(m) = w.mobile_mut(id) else { return };
    if m.stam != value {
        let old_value = m.stam;
        m.stam = value;
        delta(w, id, MobileDelta::STAM);
        (ty.on_stam_change)(w, id, old_value);
    }
}

pub fn set_mana(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile(id) else { return };
    if m.deleted {
        return;
    }
    let ty = m.ty;

    let max = (ty.mana_max)(w, id);
    let value = value.clamp(0, max);

    if value == max {
        part2::stats::stop_regen_timer(w, id, part2::stats::Regen::Mana);

        if let Some(m) = w.mobile_mut(id)
            && m.meditating
        {
            m.meditating = false;
            // You are at peace.
            part2::messages::send_localized_message(w, id, 501846, "", 0x3B2);
        }
    } else if (ty.can_regen_mana)(w, id) {
        start_mana_timer(w, id);
    } else {
        part2::stats::stop_regen_timer(w, id, part2::stats::Regen::Mana);
    }

    let Some(m) = w.mobile_mut(id) else { return };
    if m.mana != value {
        let old_value = m.mana;
        m.mana = value;
        delta(w, id, MobileDelta::MANA);
        (ty.on_mana_change)(w, id, old_value);
    }
}

pub fn set_poison(w: &mut World, id: MobileId, value: Option<PoisonId>) {
    let Some(m) = w.mobile_mut(id) else { return };
    m.poison = value;
    let ty = m.ty;
    let old_timer = std::mem::take(&mut m.poison_timer);
    delta(w, id, MobileDelta::HEALTHBAR_POISON);

    if old_timer != TimerHandle::default() {
        // PoisonTimer.Stop(); PoisonTimer = null
        timer::stop(w, old_timer);
        timer::release(w, old_timer);
    }

    if let Some(p) = value {
        let construct = crate::poison::find(w, p).map(|p| p.construct_timer);
        if let Some(construct) = construct {
            let h = construct(w, id);
            if let Some(m) = w.mobile_mut(id) {
                m.poison_timer = h;
            }
            timer::begin(w, h);
        }
    }

    (ty.check_stat_timers)(w, id);
}

pub fn set_hair_item_id(w: &mut World, id: MobileId, value: i32) {
    // An item id of 0 is the "no hair" state; clear the hue with it, but keep
    // _hairSerial so a pending removal packet references the equipped serial.
    if let Some(m) = w.mobile_mut(id) {
        m.hair_item_id = value.max(0);
        if m.hair_item_id == 0 {
            m.hair_hue = 0;
        }
    }
    delta(w, id, MobileDelta::HAIR);
}

pub fn set_facial_hair_item_id(w: &mut World, id: MobileId, value: i32) {
    if let Some(m) = w.mobile_mut(id) {
        m.facial_hair_item_id = value.max(0);
        if m.facial_hair_item_id == 0 {
            m.facial_hair_hue = 0;
        }
    }
    delta(w, id, MobileDelta::FACIAL_HAIR);
}

pub fn set_hair_hue(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.hair_item_id > 0 {
        m.hair_hue = value;
        delta(w, id, MobileDelta::HAIR);
    }
}

pub fn set_facial_hair_hue(w: &mut World, id: MobileId, value: i32) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.facial_hair_item_id > 0 {
        m.facial_hair_hue = value;
        delta(w, id, MobileDelta::FACIAL_HAIR);
    }
}

/// A virtual serial once hair exists.
pub fn hair_serial(w: &mut World, id: MobileId) -> Serial {
    let need = w
        .mobile(id)
        .is_some_and(|m| m.hair_item_id > 0 && m.hair_serial == Serial::ZERO);
    if need {
        let s = new_virtual_serial(w);
        if let Some(m) = w.mobile_mut(id) {
            m.hair_serial = s;
        }
    }
    w.mobile(id).map_or(Serial::ZERO, |m| m.hair_serial)
}

pub fn facial_hair_serial(w: &mut World, id: MobileId) -> Serial {
    let need = w
        .mobile(id)
        .is_some_and(|m| m.facial_hair_item_id > 0 && m.facial_hair_serial == Serial::ZERO);
    if need {
        let s = new_virtual_serial(w);
        if let Some(m) = w.mobile_mut(id) {
            m.facial_hair_serial = s;
        }
    }
    w.mobile(id).map_or(Serial::ZERO, |m| m.facial_hair_serial)
}

fn on_layer(w: &World, id: MobileId, layer: Layer) -> Option<ItemId> {
    part2::items::find_item_on_layer(w, id, layer)
}

pub fn shield_armor(w: &World, id: MobileId) -> Option<ItemId> {
    on_layer(w, id, Layer::TwoHanded)
}

pub fn neck_armor(w: &World, id: MobileId) -> Option<ItemId> {
    on_layer(w, id, Layer::Neck)
}

pub fn hand_armor(w: &World, id: MobileId) -> Option<ItemId> {
    on_layer(w, id, Layer::Gloves)
}

pub fn head_armor(w: &World, id: MobileId) -> Option<ItemId> {
    on_layer(w, id, Layer::Helm)
}

pub fn arms_armor(w: &World, id: MobileId) -> Option<ItemId> {
    on_layer(w, id, Layer::Arms)
}

pub fn legs_armor(w: &World, id: MobileId) -> Option<ItemId> {
    on_layer(w, id, Layer::InnerLegs).or_else(|| on_layer(w, id, Layer::Pants))
}

pub fn chest_armor(w: &World, id: MobileId) -> Option<ItemId> {
    on_layer(w, id, Layer::InnerTorso).or_else(|| on_layer(w, id, Layer::Shirt))
}

pub fn talisman(w: &World, id: MobileId) -> Option<ItemId> {
    on_layer(w, id, Layer::Talisman)
}

/// `other == null ? -1 : Serial.CompareTo(other.Serial)`.
pub fn compare_to(id: MobileId, other: Option<MobileId>) -> std::cmp::Ordering {
    match other {
        None => std::cmp::Ordering::Less,
        Some(o) => id.0.cmp(&o.0),
    }
}

/// Builds `Mobile::property_list` when there is none. The list is built outside
/// the mobile because `get_properties` reads the world (a nested `InvalidateProperties` would otherwise see
/// `IsBuilding`; `get_properties` cannot reach `&mut World` here).
pub fn ensure_property_list(w: &mut World, id: MobileId) {
    let Some(m) = w.mobile(id) else { return };
    if m.property_list.is_some() {
        return;
    }
    let ty = m.ty;
    let mut list = PropertyList::new(m.serial);
    // InitializePropertyList
    list.is_building = true;
    (ty.get_properties)(w, id, &mut list);
    list.terminate();
    list.is_building = false;
    if let Some(m) = w.mobile_mut(id) {
        m.property_list = Some(list);
    }
}

/// `Name.set` (the base setter, `base::set_name_base`).
pub(crate) fn set_name_base(w: &mut World, id: MobileId, value: Option<String>) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.name != value {
        // I'm leaving out the && m_NameMod == null
        let old_name = m.name.clone();
        m.name = value.clone();
        let ty = m.ty;
        (ty.on_after_name_change)(w, id, old_name, value);
        delta(w, id, MobileDelta::NAME);
        invalidate_properties(w, id);
    }
}

/// `Weapon.get` (the base getter, `base::weapon`). It caches in `m_Weapon`, so it takes `&mut World`.
pub(crate) fn weapon_base(w: &mut World, id: MobileId) -> Option<WeaponRef> {
    let (cached, ty) = {
        let m = w.mobile(id)?;
        (m.weapon, m.ty)
    };

    if let Some(WeaponRef::Item(item)) = cached
        && parent_is(w, item, id)
        && (ty.can_see_item)(w, id, item)
    {
        return cached;
    }

    if let Some(m) = w.mobile_mut(id) {
        m.weapon = None;
    }

    let item = on_layer(w, id, Layer::OneHanded).or_else(|| on_layer(w, id, Layer::TwoHanded));

    if let Some(item) = item {
        let is_weapon = w
            .resources
            .get::<PropsBStatics>()
            .and_then(|s| s.item_is_weapon)
            .is_some_and(|f| f(w, item));
        if is_weapon {
            let r = WeaponRef::Item(item);
            if let Some(m) = w.mobile_mut(id) {
                m.weapon = Some(r);
            }
            return Some(r);
        }
    }

    Some((ty.get_default_weapon)(w, id))
}

/// `Criminal.set` (the base setter, `base::set_criminal`).
pub(crate) fn set_criminal_base(w: &mut World, id: MobileId, value: bool) {
    let Some(m) = w.mobile_mut(id) else { return };
    if m.criminal != value {
        m.criminal = value;
        delta(w, id, MobileDelta::NOTO);
        invalidate_properties(w, id);
    }

    let mut t = w
        .mobile(id)
        .map_or_else(TimerHandle::default, |m| m.expire_criminal_timer);
    t.cancel(w);
    if let Some(m) = w.mobile_mut(id) {
        m.expire_criminal_timer = t;
    }
    if value {
        part2::stats::start_expire_criminal_timer(w, id);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::mobile::MobileType;
    use crate::mobile::mobile_type::MOBILE_BASE;
    use crate::mods::StatMod;

    thread_local! {
        static DELTAS: RefCell<Vec<MobileDelta>> = const { RefCell::new(Vec::new()) };
    }

    static RECORDING: MobileType = MobileType {
        full_name: "Test.Recording",
        parent: Some(&MOBILE_BASE),
        delta: |_, _, flag| DELTAS.with(|d| d.borrow_mut().push(flag)),
        ..MOBILE_BASE
    };

    fn deltas() -> Vec<MobileDelta> {
        DELTAS.with(|d| std::mem::take(&mut *d.borrow_mut()))
    }

    fn new_mobile() -> (World, MobileId) {
        let mut w = World::new();
        let id = w.new_mobile(&RECORDING);
        deltas();
        (w, id)
    }

    #[test]
    fn raw_stat_clamps_and_reports() {
        let (mut w, id) = new_mobile();
        set_raw_str(&mut w, id, 70000);
        assert_eq!(w.mobile(id).unwrap().str, 65000);
        assert!(deltas().contains(&(MobileDelta::STAT | MobileDelta::HITS)));
        set_raw_str(&mut w, id, 0);
        assert_eq!(w.mobile(id).unwrap().str, 1);
        deltas();
        set_raw_str(&mut w, id, 1);
        assert!(deltas().is_empty(), "no change, no delta");
        set_raw_dex(&mut w, id, 50);
        assert!(deltas().contains(&(MobileDelta::STAT | MobileDelta::STAM)));
    }

    #[test]
    fn stat_setters_ignore_changes_under_a_mod() {
        let (mut w, id) = new_mobile();
        set_raw_str(&mut w, id, 50);
        let m = StatMod::new(
            &mut w,
            StatType::STR,
            Some("x"),
            10,
            TimeSpan::ZERO,
            Some(id),
        );
        w.mobile_mut(id).unwrap().stat_mods.push(m);
        (RECORDING.set_str)(&mut w, id, 99);
        assert_eq!(w.mobile(id).unwrap().str, 50);
        assert_eq!((RECORDING.str)(&w, id), 60);
    }

    #[test]
    fn hits_clamp_to_max_and_report() {
        let (mut w, id) = new_mobile();
        set_raw_str(&mut w, id, 100);
        deltas();
        set_hits(&mut w, id, 1000);
        assert_eq!(w.mobile(id).unwrap().hits, 100);
        assert_eq!(deltas(), vec![MobileDelta::HITS]);
        set_hits(&mut w, id, 100);
        assert!(deltas().is_empty(), "no change, no delta");
        set_hits(&mut w, id, -5);
        assert_eq!(w.mobile(id).unwrap().hits, 0);
    }

    #[test]
    fn name_setter_reports_only_a_change() {
        let (mut w, id) = new_mobile();
        (RECORDING.set_name)(&mut w, id, Some("Bob".into()));
        assert_eq!((RECORDING.name)(&w, id).as_deref(), Some("Bob"));
        assert!(deltas().contains(&MobileDelta::NAME));
        (RECORDING.set_name)(&mut w, id, Some("Bob".into()));
        assert!(deltas().is_empty());
        set_name_mod(&mut w, id, Some("Mod".into()));
        assert_eq!((RECORDING.name)(&w, id).as_deref(), Some("Mod"));
        assert_eq!(raw_name(&w, id).as_deref(), Some("Bob"));
    }

    #[test]
    fn kills_floor_at_zero_and_hair_serial_is_virtual() {
        let (mut w, id) = new_mobile();
        set_kills(&mut w, id, -3);
        assert_eq!(w.mobile(id).unwrap().kills, 0);
        assert_eq!(hair_serial(&mut w, id), Serial::ZERO);
        set_hair_item_id(&mut w, id, 0x203B);
        let s = hair_serial(&mut w, id);
        assert_eq!(s.value(), MAX_ITEM_SERIAL);
        assert_eq!(hair_serial(&mut w, id), s);
        set_hair_item_id(&mut w, id, 0);
        assert_eq!(w.mobile(id).unwrap().hair_hue, 0);
    }

    // Virtual serial tests.
    #[test]
    fn new_virtual_gets_and_rollover() {
        // walking up from the reset takes ~17.9 million draws; start just short of the wrap
        const DRAWS_BEFORE_MAX: u32 = 1000;
        let mut w = World::new();
        w.resources.get_mut::<PropsBStatics>().next_virtual_serial =
            MAX_VIRTUAL_SERIAL - DRAWS_BEFORE_MAX;

        let mut last = new_virtual_serial(&mut w).value();
        loop {
            let virtual_serial = new_virtual_serial(&mut w).value();
            assert_eq!(last + 1, virtual_serial);
            last = virtual_serial;
            if last == MAX_VIRTUAL_SERIAL {
                break;
            }
        }

        // the next one is MaxItemSerial (the first virtual serial) due to rollover
        assert_eq!(new_virtual_serial(&mut w).value(), MAX_ITEM_SERIAL);
    }
}
