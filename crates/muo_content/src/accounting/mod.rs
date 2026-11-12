//! Accounts: `Account`, its comments and tags, and the `Accounts` store.
//!
//! An `Account` is a record in the [`Accounts`] resource, referred to by [`AccountId`] (its serial). A mobile's
//! `account` field is that id; a connection's account is the username (`NetStateData::account`).
//!
//! What a `PlayerMobile` knows (`SessionStart`, `GameTime`, `Young`) and the houses of a character are not
//! accounting's: they are [`AccountHooks`], set by the owners of those systems. The defaults act like an account
//! that has no `PlayerMobile`.
//!
//! The pre-binary `Accounts/accounts.xml` loader is `xml.rs`.

pub mod handler;
pub mod password_worker;
pub mod persistence;
pub mod security;
pub mod xml;

use muo_primitives::dotnet::IgnoreCase;
use std::net::IpAddr;

use indexmap::IndexMap;
use muo_io::{GenericReader, GenericWriter, Result as IoResult};
use muo_primitives::{DateTime, Serial, TimeSpan};
use muo_server::World;
use muo_server::iaccount::AccountId;
use muo_server::ids::MobileId;
use muo_server::mobile::AccessLevel;
use muo_server::mobile::part2::messages;
use muo_server::serialization_ext::{ReadEntityExt, WriteEntityExt};
use muo_server::timer::{self, TimerHandle};

use security::PasswordProtectionAlgorithm;

pub const YOUNG_DURATION: TimeSpan = TimeSpan::from_hours(40);
pub const INACTIVE_DURATION: TimeSpan = TimeSpan::from_days(180);
pub const EMPTY_INACTIVE_DURATION: TimeSpan = TimeSpan::from_days(30);

/// `Account.SerializationVersion`.
pub const SERIALIZATION_VERSION: i32 = 6;

/// `AccountComment`.
#[derive(Clone, Debug)]
pub struct AccountComment {
    pub added_by: String,
    content: String,
    pub last_modified: DateTime,
}

impl AccountComment {
    pub fn new(w: &World, added_by: &str, content: &str) -> Self {
        Self {
            added_by: added_by.to_owned(),
            content: content.to_owned(),
            last_modified: w.now,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    /// Setting the body resets `LastModified`.
    pub fn set_content(&mut self, now: DateTime, content: &str) {
        self.content = content.to_owned();
        self.last_modified = now;
    }

    fn deserialize(r: &mut dyn GenericReader) -> IoResult<Self> {
        Ok(Self {
            added_by: r.read_string()?.unwrap_or_default(),
            last_modified: DateTime::from_ticks(r.read_date_time_ticks()?),
            content: r.read_string()?.unwrap_or_default(),
        })
    }

    fn serialize(&self, writer: &mut dyn GenericWriter) {
        writer.write_string(&self.added_by);
        writer.write_date_time_ticks(self.last_modified.ticks());
        writer.write_string(&self.content);
    }
}

/// `AccountTag`.
#[derive(Clone, Debug)]
pub struct AccountTag {
    pub name: String,
    pub value: Option<String>,
}

impl AccountTag {
    fn deserialize(r: &mut dyn GenericReader) -> IoResult<Self> {
        Ok(Self {
            name: r.read_string()?.unwrap_or_default(),
            value: r.read_string()?,
        })
    }

    fn serialize(&self, writer: &mut dyn GenericWriter) {
        writer.write_string(&self.name);
        writer.write_opt_string(self.value.as_deref());
    }
}

/// `Server.HardwareInfo`: what the client reported in packet 0xD9, kept on the account
/// (not saved).
#[derive(Clone, Debug, Default)]
pub struct HardwareInfo {
    pub cpu_model: i32,
    pub cpu_clock_speed: i32,
    pub cpu_quantity: i32,
    pub os_major: i32,
    pub os_minor: i32,
    pub os_revision: i32,
    pub instance_id: i32,
    pub screen_width: i32,
    pub screen_height: i32,
    pub screen_depth: i32,
    pub physical_memory: i32,
    pub cpu_manufacturer: i32,
    pub cpu_family: i32,
    pub vc_vendor_id: i32,
    pub vc_device_id: i32,
    pub vc_memory: i32,
    pub dx_major: i32,
    pub dx_minor: i32,
    pub vc_description: String,
    pub language: String,
    pub distribution: i32,
    pub clients_running: i32,
    pub clients_installed: i32,
    pub partial_installed: i32,
    pub unknown: String,
    pub time_received: DateTime,
}

/// `Account`.
pub struct Account {
    pub serial: Serial,
    pub username: String,
    pub password_algorithm: PasswordProtectionAlgorithm,
    pub password: Option<String>,
    pub access_level: AccessLevel,
    pub flags: i32,
    pub last_login: DateTime,
    pub total_gold: i32,
    pub total_plat: i32,
    /// `_mobiles`: always 7 slots (`Length`).
    pub mobiles: Vec<Option<MobileId>>,
    /// Empty means none.
    pub comments: Vec<AccountComment>,
    pub tags: Vec<AccountTag>,
    pub login_ips: Vec<IpAddr>,
    /// `_totalGameTime` (without the running session).
    pub total_game_time: TimeSpan,
    pub email: Option<String>,
    pub created: DateTime,
    pub deleted: bool,
    /// `HardwareInfo`: the last client hardware report (not saved).
    pub hardware_info: Option<HardwareInfo>,
    young_timer: Option<TimerHandle>,
}

impl Account {
    /// `Account(Serial)`: the empty shell the loader fills.
    pub(crate) fn blank(serial: Serial, now: DateTime) -> Self {
        Account {
            serial,
            username: String::new(),
            password_algorithm: PasswordProtectionAlgorithm::None,
            password: None,
            access_level: AccessLevel::Player,
            flags: 0,
            last_login: DateTime::MIN,
            total_gold: 0,
            total_plat: 0,
            mobiles: Vec::new(),
            comments: Vec::new(),
            tags: Vec::new(),
            login_ips: Vec::new(),
            total_game_time: TimeSpan::ZERO,
            email: None,
            created: now,
            deleted: false,
            hardware_info: None,
            young_timer: None,
        }
    }

    pub fn id(&self) -> AccountId {
        AccountId(self.serial.value())
    }

    /// `GetFlag(index)`.
    pub fn get_flag(&self, index: i32) -> bool {
        self.flags & (1 << index) != 0
    }

    /// `SetFlag(index, value)`.
    pub fn set_flag(&mut self, index: i32, value: bool) {
        if value {
            self.flags |= 1 << index;
        } else {
            self.flags &= !(1 << index);
        }
    }

    /// `GetTag(name)`.
    pub fn get_tag(&self, name: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|t| t.name == name)
            .and_then(|t| t.value.as_deref())
    }

    /// `AddTag`: no duplicate check.
    pub fn add_tag(&mut self, name: &str, value: &str) {
        self.tags.push(AccountTag {
            name: name.to_owned(),
            value: Some(value.to_owned()),
        });
    }

    /// `RemoveTag`: every tag with the name.
    pub fn remove_tag(&mut self, name: &str) {
        self.tags.retain(|t| t.name != name);
    }

    /// `SetTag`: modifies the first tag with the name, else adds one.
    pub fn set_tag(&mut self, name: &str, value: &str) {
        match self.tags.iter_mut().find(|t| t.name == name) {
            Some(t) => t.value = Some(value.to_owned()),
            None => self.add_tag(name, value),
        }
    }

    /// `SetBanTags(from, banTime, banDuration)`; `from` is `from.ToString()` of the banning mobile.
    pub fn set_ban_tags(&mut self, from: Option<&str>, ban_time: DateTime, ban_duration: TimeSpan) {
        match from {
            None => self.remove_tag("BanDealer"),
            Some(f) => self.set_tag("BanDealer", f),
        }
        if ban_time == DateTime::MIN {
            self.remove_tag("BanTime");
        } else {
            self.set_tag("BanTime", &ban_time.to_json_string());
        }
        if ban_duration == TimeSpan::ZERO {
            self.remove_tag("BanDuration");
        } else {
            self.set_tag("BanDuration", &ban_duration.to_string());
        }
    }

    /// `SetUnspecifiedBan(null)`.
    pub fn set_unspecified_ban(&mut self, from: Option<&str>) {
        self.set_ban_tags(from, DateTime::MIN, TimeSpan::ZERO);
    }

    /// `GetBanTags(out banTime, out banDuration)`.
    pub fn get_ban_tags(&self) -> (bool, DateTime, TimeSpan) {
        let ban_time = self
            .get_tag("BanTime")
            .and_then(DateTime::parse_iso)
            .unwrap_or(DateTime::MIN);
        let ban_duration = match self.get_tag("BanDuration") {
            Some("Infinite") => TimeSpan::MAX,
            Some(d) => TimeSpan::try_parse(d).unwrap_or(TimeSpan::ZERO),
            None => TimeSpan::ZERO,
        };
        (
            ban_time != DateTime::MIN && ban_duration != TimeSpan::ZERO,
            ban_time,
            ban_duration,
        )
    }

    /// `Length`.
    pub fn length(&self) -> i32 {
        self.mobiles.len() as i32
    }

    /// `ToString()`.
    pub fn name(&self) -> &str {
        &self.username
    }
}

impl Account {
    /// `DepositGold`: whole `CurrencyThreshold`s become platinum.
    pub fn deposit_gold(&mut self, amount: i32, threshold: i32) -> bool {
        if amount <= 0 {
            return false;
        }
        // `TotalPlat += plat` wraps on overflow.
        self.total_plat = self.total_plat.wrapping_add(amount / threshold);
        self.total_gold = self.total_gold.wrapping_add(amount % threshold);
        true
    }

    /// `DepositPlat`.
    pub fn deposit_plat(&mut self, amount: i32) -> bool {
        if amount <= 0 {
            return false;
        }
        self.total_plat = self.total_plat.wrapping_add(amount);
        true
    }

    /// `WithdrawGold`: false when the balance is too low.
    pub fn withdraw_gold(&mut self, amount: i32) -> bool {
        if amount <= 0 {
            return true;
        }
        if amount > self.total_gold {
            return false;
        }
        self.total_gold -= amount;
        true
    }

    /// `WithdrawPlat`.
    pub fn withdraw_plat(&mut self, amount: i32) -> bool {
        if amount <= 0 {
            return true;
        }
        if amount > self.total_plat {
            return false;
        }
        self.total_plat -= amount;
        true
    }

    /// `GetTotalGold()`.
    pub fn get_total_gold(&self, threshold: i32) -> i64 {
        // The two ints are multiplied before widening, so the product wraps on overflow.
        self.total_gold as i64 + self.total_plat.wrapping_mul(threshold) as i64
    }
}

/// Fills `iaccount::AccountHooks` with the accounts' real members (`Mobile.Account` as `IGoldAccount`).
pub fn install_gold_hooks(w: &mut World) {
    use muo_server::iaccount::{AccountGold, AccountHooks as Gold};
    fn threshold(w: &World) -> i32 {
        w.resources
            .get::<AccountGold>()
            .map_or(1_000_000_000, |g| g.currency_threshold)
    }
    fn acct(w: &mut World, m: MobileId) -> Option<&mut Account> {
        let id = w.mobile(m)?.account?;
        account_mut(w, id)
    }
    *w.resources.get_mut::<Gold>() = Gold {
        totals: Some(|w, m| {
            let a = account(w, w.mobile(m)?.account?)?;
            Some((a.total_gold, a.total_plat))
        }),
        withdraw_gold: Some(|w, m, n| acct(w, m).is_some_and(|a| a.withdraw_gold(n))),
        deposit_gold: Some(|w, m, n| {
            let t = threshold(w);
            acct(w, m).is_some_and(|a| a.deposit_gold(n, t))
        }),
        withdraw_plat: Some(|w, m, n| acct(w, m).is_some_and(|a| a.withdraw_plat(n))),
        deposit_plat: Some(|w, m, n| acct(w, m).is_some_and(|a| a.deposit_plat(n))),
    };
}

/// What other systems provide about their mobiles, set by their `Configure`/`Initialize`.
pub struct AccountHooks {
    /// `ContentFeatureFlags.YoungPlayerSystem`.
    pub young_player_system: bool,
    /// `m is PlayerMobile pm ? pm.SessionStart : null`.
    pub session_start: fn(&World, MobileId) -> Option<DateTime>,
    /// `m is PlayerMobile pm ? pm.GameTime : null`.
    pub game_time: fn(&World, MobileId) -> Option<TimeSpan>,
    /// `m is PlayerMobile { Young: true }`.
    pub is_young_player: fn(&World, MobileId) -> bool,
    /// `((PlayerMobile)m).Young = value`.
    pub set_young_player: fn(&mut World, MobileId, bool),
    /// `Region.Find(m.LogoutLocation, m.LogoutMap).IsPartOf<JailRegion>()`.
    pub in_jail_region: fn(&World, MobileId) -> bool,
    /// `BaseHouse.GetHouses(m)` then `Delete()` each.
    pub delete_houses: fn(&mut World, MobileId),
}

impl Default for AccountHooks {
    fn default() -> Self {
        use crate::player_mobile as pm;
        AccountHooks {
            young_player_system: true,
            session_start: |w, m| pm::data(w, m).map(|d| d.session_start),
            game_time: |w, m| pm::data(w, m).map(|d| d.game_time),
            is_young_player: |w, m| pm::data(w, m).is_some() && pm::young(w, m),
            set_young_player: pm::set_young,
            // Replaced in `wiring::configure` (JailRegion).
            in_jail_region: |_, _| false,
            // Replaced by muo_multis hooks (BaseHouse).
            delete_houses: |_, _| {},
        }
    }
}

/// `Accounts`' statics.
#[derive(Default)]
pub struct Accounts {
    by_serial: IndexMap<Serial, Account>,
    /// `_accountsByName`, `OrdinalIgnoreCase`: the key is the lower-cased name.
    by_name: IndexMap<String, Serial>,
    last_serial: u32,
    /// The persistence's `_typeTable` stand-in: whether any account was ever added.
    pub(crate) type_registered: bool,
}

fn name_key(name: &str) -> String {
    name.ic_key()
}

impl Accounts {
    pub fn count(&self) -> usize {
        self.by_name.len()
    }

    /// `Accounts.NewAccount`: the next free serial in `1..=0x7FFFFFFF`, wrapping.
    pub fn new_account_serial(&mut self) -> Serial {
        let (min, max) = (1u32, 0x7FFF_FFFFu32);
        let mut last = self.last_serial.max(min - 1);
        for _ in 0..max {
            last += 1;
            if last > max {
                last = min;
            }
            if !self.by_serial.contains_key(&Serial::new(last)) {
                self.last_serial = last;
                return Serial::new(last);
            }
        }
        Serial::MINUS_ONE
    }

    /// `Accounts.GetAccounts()`: name order of insertion.
    pub fn iter(&self) -> impl Iterator<Item = &Account> {
        self.by_name.values().filter_map(|s| self.by_serial.get(s))
    }

    pub fn ids(&self) -> Vec<AccountId> {
        self.iter().map(Account::id).collect()
    }

    pub fn get(&self, id: AccountId) -> Option<&Account> {
        self.by_serial.get(&Serial::new(id.0))
    }

    pub fn get_mut(&mut self, id: AccountId) -> Option<&mut Account> {
        self.by_serial.get_mut(&Serial::new(id.0))
    }

    /// `Accounts.GetAccount(username)`.
    pub fn find(&self, username: &str) -> Option<AccountId> {
        self.by_name
            .get(&name_key(username))
            .map(|s| AccountId(s.value()))
    }

    /// `Accounts.Add(a)`.
    pub fn add(&mut self, a: Account) -> AccountId {
        let id = a.id();
        self.by_name.insert(name_key(&a.username), a.serial);
        self.by_serial.insert(a.serial, a);
        self.type_registered = true;
        id
    }

    /// `Accounts.Remove(a)`.
    pub fn remove(&mut self, id: AccountId) -> Option<Account> {
        let a = self.by_serial.shift_remove(&Serial::new(id.0))?;
        self.by_name.shift_remove(&name_key(&a.username));
        Some(a)
    }

    pub(crate) fn raw_insert(&mut self, a: Account) {
        self.by_serial.insert(a.serial, a);
        self.type_registered = true;
    }

    pub(crate) fn raw_serials(&self) -> impl Iterator<Item = Serial> + '_ {
        self.by_serial.keys().copied()
    }

    pub(crate) fn set_last_serial(&mut self, last: u32) {
        self.last_serial = last;
    }

    pub(crate) fn rebuild_names(&mut self) {
        for a in self.by_serial.values() {
            self.by_name.insert(name_key(&a.username), a.serial);
        }
    }
}

/// The resource.
pub fn accounts(w: &mut World) -> &mut Accounts {
    w.resources.get_mut::<Accounts>()
}

pub fn hooks(w: &mut World) -> &mut AccountHooks {
    w.resources.get_mut::<AccountHooks>()
}

/// `Accounts.GetAccount(username)`.
pub fn get_account(w: &World, username: &str) -> Option<AccountId> {
    w.resources.get::<Accounts>()?.find(username)
}

pub fn account(w: &World, id: AccountId) -> Option<&Account> {
    w.resources.get::<Accounts>()?.get(id)
}

pub fn account_mut(w: &mut World, id: AccountId) -> Option<&mut Account> {
    accounts(w).get_mut(id)
}

/// `Account(username, password)`.
pub fn create_account(w: &mut World, username: &str, password: &str) -> AccountId {
    let serial = accounts(w).new_account_serial();
    let mut a = Account::blank(serial, w.now);
    a.username = username.to_owned();
    a.access_level = AccessLevel::Player;
    a.last_login = w.now;
    a.total_game_time = TimeSpan::ZERO;
    a.mobiles = vec![None; 7];
    let id = accounts(w).add(a);
    set_password(w, id, password);
    id
}

/// `Account.SetPassword`.
pub fn set_password(w: &mut World, id: AccountId, plain: &str) {
    let alg = handler::current_algorithm(w);
    let Some(a) = account_mut(w, id) else { return };
    a.password_algorithm = alg;
    let phrase = security::derive_phrase(alg, &a.username, plain);
    a.password = security::encrypt_password(alg, &phrase);
}

/// `Account.CheckPassword`: also moves the stored hash to the configured algorithm.
pub fn check_password(w: &mut World, id: AccountId, plain: &str) -> bool {
    let current = handler::current_algorithm(w);
    let Some(a) = account(w, id) else {
        return false;
    };
    let phrase = security::derive_phrase(a.password_algorithm, &a.username, plain);
    let stored = a.password.clone().unwrap_or_default();
    if !security::validate_password(a.password_algorithm, &stored, &phrase) {
        return false;
    }
    // Upgrade the password protection in case we change the algorithm
    if a.password_algorithm != current || security::needs_rehash(current, &stored) {
        set_password(w, id, plain);
    }
    true
}

/// `Account.Deleted`-aware lookup of a character: the indexer getter. A deleted mobile is cleared from the slot.
pub fn mobile_at(w: &mut World, id: AccountId, index: i32) -> Option<MobileId> {
    let a = account(w, id)?;
    if index < 0 || index as usize >= a.mobiles.len() {
        return None;
    }
    let m = a.mobiles[index as usize]?;
    if w.mobile(m).is_some_and(|mm| !mm.deleted) {
        return Some(m);
    }
    // This is the only place that clears a mobile for garbage collection outside of an entire account deletion.
    if let Some(mm) = w.mobile_mut(m) {
        mm.account = None;
    }
    if let Some(a) = account_mut(w, id) {
        a.mobiles[index as usize] = None;
    }
    None
}

/// The indexer setter.
pub fn set_mobile_at(w: &mut World, id: AccountId, index: i32, value: Option<MobileId>) {
    let Some(a) = account(w, id) else { return };
    if index < 0 || index as usize >= a.mobiles.len() {
        return;
    }
    if let Some(old) = a.mobiles[index as usize]
        && let Some(m) = w.mobile_mut(old)
    {
        m.account = None;
    }
    if let Some(a) = account_mut(w, id) {
        a.mobiles[index as usize] = value;
    }
    if let Some(m) = value.and_then(|m| w.mobile_mut(m)) {
        m.account = Some(id);
    }
}

/// `Count`: the characters on the account.
pub fn count(w: &mut World, id: AccountId) -> i32 {
    let len = account(w, id).map_or(0, |a| a.mobiles.len());
    (0..len as i32)
        .filter(|i| mobile_at(w, id, *i).is_some())
        .count() as i32
}

/// `Limit`: `Core.SA ? 7 : Core.AOS ? 6 : 5`.
pub fn limit(w: &World) -> i32 {
    if crate::core::sa(w) {
        7
    } else if crate::core::aos(w) {
        6
    } else {
        5
    }
}

/// `Mobiles` / the enumerator: the characters that are not deleted.
pub fn mobiles(w: &World, id: AccountId) -> Vec<MobileId> {
    account(w, id)
        .map(|a| {
            a.mobiles
                .iter()
                .flatten()
                .copied()
                .filter(|m| w.mobile(*m).is_some_and(|mm| !mm.deleted))
                .collect()
        })
        .unwrap_or_default()
}

/// `Banned` getter: an expired ban clears itself.
pub fn is_banned(w: &mut World, id: AccountId) -> bool {
    let now = w.now;
    let Some(a) = account_mut(w, id) else {
        return false;
    };
    if !a.get_flag(0) {
        return false;
    }
    let (found, ban_time, ban_duration) = a.get_ban_tags();
    if found && ban_duration != TimeSpan::MAX && now >= ban_time + ban_duration {
        a.set_unspecified_ban(None); // clear
        a.set_flag(0, false);
        return false;
    }
    true
}

/// `Banned` setter.
pub fn set_banned(w: &mut World, id: AccountId, banned: bool) {
    if let Some(a) = account_mut(w, id) {
        a.set_flag(0, banned);
    }
}

/// `Young` getter.
pub fn is_young(w: &World, id: AccountId) -> bool {
    let on = w
        .resources
        .get::<AccountHooks>()
        .is_none_or(|h| h.young_player_system);
    on && account(w, id).is_some_and(|a| !a.get_flag(1))
}

/// `Young` setter: also stops the young timer.
pub fn set_young(w: &mut World, id: AccountId, value: bool) {
    let Some(a) = account_mut(w, id) else { return };
    a.set_flag(1, !value);
    if let Some(mut t) = a.young_timer.take() {
        t.cancel(w);
    }
}

/// `Inactive`.
pub fn is_inactive(w: &mut World, id: AccountId) -> bool {
    let Some(a) = account(w, id) else {
        return false;
    };
    if a.access_level != AccessLevel::Player {
        return false;
    }
    let last = a.last_login;
    let inactive = w.now - last;
    let empty = count(w, id) == 0;
    inactive
        > if empty {
            EMPTY_INACTIVE_DURATION
        } else {
            INACTIVE_DURATION
        }
}

/// `TotalGameTime` getter: counts the running session of a connected `PlayerMobile`.
pub fn total_game_time(w: &World, id: AccountId) -> TimeSpan {
    let Some(a) = account(w, id) else {
        return TimeSpan::ZERO;
    };
    let h = w.resources.get::<AccountHooks>();
    for m in a.mobiles.iter().flatten() {
        let connected = w.mobile(*m).is_some_and(|mm| mm.net_state.is_some());
        if connected && let Some(start) = h.and_then(|h| (h.session_start)(w, *m)) {
            return a.total_game_time + (w.now - start);
        }
    }
    a.total_game_time
}

/// `AccountAge`.
pub fn account_age(w: &World, id: AccountId) -> TimeSpan {
    account(w, id).map_or(TimeSpan::ZERO, |a| w.now - a.created)
}

/// `Delete()`: the account, all characters of the account, and all houses of those characters.
pub fn delete_account(w: &mut World, id: AccountId) {
    let len = account(w, id).map_or(0, |a| a.mobiles.len());
    for i in 0..len as i32 {
        let Some(m) = mobile_at(w, id, i) else {
            continue;
        };
        let hooks = w.resources.get::<AccountHooks>().map(|h| h.delete_houses);
        if let Some(f) = hooks {
            f(w, m);
        }
        if let Some(ty) = w.mobile(m).map(|mm| mm.ty) {
            (ty.delete)(w, m);
        }
        if let Some(mm) = w.mobile_mut(m) {
            mm.account = None;
        }
        if let Some(a) = account_mut(w, id) {
            a.mobiles[i as usize] = None;
        }
    }
    if let Some(first) = account(w, id).and_then(|a| a.login_ips.first().copied()) {
        let table = handler::ip_table(w);
        if let Some(n) = table.get_mut(&first) {
            *n -= 1;
        }
    }
    if let Some(a) = account_mut(w, id) {
        a.deleted = true;
    }
    accounts(w).remove(id);
}

/// `TrySetUsername`.
pub fn try_set_username(w: &mut World, id: AccountId, username: &str) -> bool {
    let Some(a) = account(w, id) else {
        return false;
    };
    if username == a.username || get_account(w, username).is_some() {
        return false;
    }
    let acc = accounts(w);
    let mut a = acc.remove(id).expect("checked above");
    a.username = username.to_owned();
    acc.add(a);
    true
}

/// `HasAccess(ipAddress)`.
pub fn has_access(w: &mut World, id: AccountId, _address: IpAddr) -> bool {
    let level = handler::lockdown_level(w);
    let Some(a) = account(w, id) else {
        return false;
    };
    if level <= AccessLevel::Player || a.access_level >= level {
        return true;
    }
    let len = a.mobiles.len() as i32;
    for i in 0..len {
        if let Some(m) = mobile_at(w, id, i)
            && w.mobile(m).is_some_and(|mm| mm.access_level >= level)
        {
            return true;
        }
    }
    false
}

/// `LogAccess(ipAddress)`.
pub fn log_access(w: &mut World, id: AccountId, address: IpAddr) {
    let empty = account(w, id).is_some_and(|a| a.login_ips.is_empty());
    if empty {
        *handler::ip_table(w).entry(address).or_insert(0) += 1;
    }
    if let Some(a) = account_mut(w, id)
        && !a.login_ips.contains(&address)
    {
        a.login_ips.push(address);
    }
}

/// `CheckAccess(ipAddress)`: if allowed, the address is added to the address list.
pub fn check_access(w: &mut World, id: AccountId, address: IpAddr) -> bool {
    let ok = has_access(w, id, address);
    if ok {
        log_access(w, id, address);
    }
    ok
}

/// `RemoveYoungStatus(message)`.
pub fn remove_young_status(w: &mut World, id: AccountId, message: i32) {
    set_young(w, id, false);
    let Some(a) = account(w, id) else { return };
    let list: Vec<MobileId> = a.mobiles.iter().flatten().copied().collect();
    for m in list {
        let (is_young, set_young_player) = {
            let h = w.resources.get::<AccountHooks>();
            (h.map(|h| h.is_young_player), h.map(|h| h.set_young_player))
        };
        if !is_young.is_some_and(|f| f(w, m)) {
            continue;
        }
        if let Some(f) = set_young_player {
            f(w, m, false);
        }
        if w.mobile(m).is_some_and(|mm| mm.net_state.is_some()) {
            if message > 0 {
                messages::send_localized_message(w, m, message, "", 0x3B2);
            }
            // You are no longer considered a young player of Ultima Online,
            // and are no longer subject to the limitations and benefits of being in that caste.
            messages::send_localized_message(w, m, 1019039, "", 0x3B2);
        }
    }
}

/// `CheckYoung`.
pub fn check_young(w: &mut World, id: AccountId) {
    if total_game_time(w, id) >= YOUNG_DURATION {
        // You are old enough to be considered an adult, and have outgrown your status as a young player!
        remove_young_status(w, id, 1019038);
    }
}

/// `AfterDeserialization`.
fn after_deserialization(w: &mut World, id: AccountId) {
    let Some(a) = account(w, id) else { return };
    let mobs: Vec<MobileId> = a.mobiles.iter().flatten().copied().collect();
    for m in &mobs {
        if let Some(mm) = w.mobile_mut(*m) {
            mm.account = Some(id);
        }
    }
    let zero = account(w, id).is_some_and(|a| a.total_game_time == TimeSpan::ZERO);
    if zero {
        let f = w.resources.get::<AccountHooks>().map(|h| h.game_time);
        let mut sum = TimeSpan::ZERO;
        if let Some(f) = f {
            for m in &mobs {
                if let Some(t) = f(w, *m) {
                    sum += t;
                }
            }
        }
        if let Some(a) = account_mut(w, id) {
            a.total_game_time += sum;
        }
    }
    if is_young(w, id) {
        check_young(w, id);
    }
}

/// `EventSink_Connected`.
fn on_connected(w: &mut World, m: MobileId) {
    let Some(id) = w.mobile(m).and_then(|mm| mm.account) else {
        return;
    };
    if is_young(w, id) && account(w, id).is_some_and(|a| a.young_timer.is_none()) {
        let h = timer::start(
            w,
            TimeSpan::from_mins(1),
            TimeSpan::from_mins(1),
            0,
            move |w| check_young(w, id),
        );
        if let Some(a) = account_mut(w, id) {
            a.young_timer = Some(h);
        }
    }
}

/// `EventSink_Disconnected`.
fn on_disconnected(w: &mut World, m: MobileId) {
    let Some(id) = w.mobile(m).and_then(|mm| mm.account) else {
        return;
    };
    if let Some(mut t) = account_mut(w, id).and_then(|a| a.young_timer.take()) {
        t.cancel(w);
    }
    let start = w.resources.get::<AccountHooks>().map(|h| h.session_start);
    let Some(start) = start.and_then(|f| f(w, m)) else {
        return;
    };
    let now = w.now;
    if let Some(a) = account_mut(w, id) {
        a.total_game_time += now - start;
    }
}

/// `Account.OnLogin(pm)`, subscribed to `PlayerMobile.PlayerLoginEvent`.
fn on_login(w: &mut World, e: &mut crate::network::events::PlayerLogin) {
    let pm = e.0;
    let Some(id) = w.mobile(pm).and_then(|m| m.account) else {
        return;
    };
    let player_young = crate::player_mobile::young(w, pm);
    if !player_young || !is_young(w, id) {
        return;
    }
    let ts = YOUNG_DURATION - total_game_time(w, id);
    let hours = (ts.total_hours() as i32).max(0);
    let text = if hours == 1 {
        format!(
            "You will enjoy the benefits and relatively safe status of a young player for {hours} more hour."
        )
    } else {
        format!(
            "You will enjoy the benefits and relatively safe status of a young player for {hours} more hours."
        )
    };
    messages::send_ascii_message(w, pm, &text);
}

/// `Account.Initialize`.
pub fn initialize(w: &mut World) {
    w.events.connected.subscribe(|w, m| on_connected(w, *m));
    w.events
        .disconnected
        .subscribe(|w, m| on_disconnected(w, *m));
    crate::network::events::subscribe_ranked::<crate::network::events::PlayerLogin>(w, 1, on_login);
}

// ---- serialization (the generator's output for version 6, and the migrations) ----

impl Account {
    /// `Serialize(writer)`.
    pub fn serialize(&self, w: &World, writer: &mut dyn GenericWriter) {
        writer.write_encoded_int(SERIALIZATION_VERSION);
        writer.write_string(&self.username);
        writer.write_enum(4, self.password_algorithm as i64);
        writer.write_opt_string(self.password.as_deref());
        writer.write_enum(4, self.access_level as i64);
        writer.write_i32(self.flags);
        writer.write_date_time_ticks(self.last_login.ticks());
        writer.write_i32(self.total_gold);
        writer.write_i32(self.total_plat);
        writer.write_encoded_int(self.mobiles.len() as i32);
        for m in &self.mobiles {
            writer.write_mobile(w, *m);
        }
        writer.write_encoded_int(self.comments.len() as i32);
        for c in &self.comments {
            c.serialize(writer);
        }
        writer.write_encoded_int(self.tags.len() as i32);
        for t in &self.tags {
            t.serialize(writer);
        }
        writer.write_encoded_int(self.login_ips.len() as i32);
        for ip in &self.login_ips {
            writer.write_ip_address(*ip);
        }
        writer.write_time_span_ticks(self.total_game_time.ticks());
        writer.write_opt_string(self.email.as_deref());
    }

    /// `Deserialize(reader)` for every version. `reader` is at the start of the record.
    pub fn deserialize(&mut self, w: &World, r: &mut dyn GenericReader) -> IoResult<()> {
        let version = r.read_encoded_int()?;
        if (3..=5).contains(&version) {
            // V3Content, V4Content, V5Content then MigrateFrom: the same fields in order, V3 has Created,
            // V3/V4/V5 carry the deleted IP restrictions.
            self.username = r.read_string()?.unwrap_or_default();
            self.read_common_head(r, version == 3)?;
            self.read_tail(w, r, true)?;
            return Ok(());
        }
        if version < SERIALIZATION_VERSION {
            return self.deserialize_legacy(w, r, version);
        }
        self.username = r.read_string()?.unwrap_or_default();
        self.read_common_head(r, false)?;
        self.read_tail(w, r, false)
    }

    /// Password algorithm to `TotalPlat` (V3 also has `Created` between `Flags` and `LastLogin`).
    fn read_common_head(&mut self, r: &mut dyn GenericReader, with_created: bool) -> IoResult<()> {
        self.password_algorithm = PasswordProtectionAlgorithm::from_i32(r.read_enum(4)? as i32);
        self.password = r.read_string()?;
        self.access_level = AccessLevel::from_i32(r.read_enum(4)? as i32).unwrap_or_default();
        self.flags = r.read_i32()?;
        if with_created {
            self.created = DateTime::from_ticks(r.read_date_time_ticks()?);
        }
        self.last_login = DateTime::from_ticks(r.read_date_time_ticks()?);
        self.total_gold = r.read_i32()?;
        self.total_plat = r.read_i32()?;
        Ok(())
    }

    fn read_tail(
        &mut self,
        w: &World,
        r: &mut dyn GenericReader,
        ip_restrictions: bool,
    ) -> IoResult<()> {
        let n = r.read_encoded_int()?.max(0) as usize;
        self.mobiles = Vec::with_capacity(n);
        for _ in 0..n {
            self.mobiles.push(r.read_mobile(w)?);
        }
        let n = r.read_encoded_int()?.max(0);
        self.comments = Vec::new();
        for _ in 0..n {
            self.comments.push(AccountComment::deserialize(r)?);
        }
        let n = r.read_encoded_int()?.max(0);
        self.tags = Vec::new();
        for _ in 0..n {
            self.tags.push(AccountTag::deserialize(r)?);
        }
        let n = r.read_encoded_int()?.max(0);
        self.login_ips = Vec::new();
        for _ in 0..n {
            self.login_ips.push(r.read_ip_address()?);
        }
        if ip_restrictions {
            let n = r.read_encoded_int()?.max(0);
            for _ in 0..n {
                r.read_string()?;
            }
        }
        self.total_game_time = TimeSpan::from_ticks(r.read_time_span_ticks()?);
        self.email = r.read_string()?;
        Ok(())
    }

    /// `Deserialize(reader, version)` of the legacy layouts, versions 0 to 2.
    fn deserialize_legacy(
        &mut self,
        w: &World,
        r: &mut dyn GenericReader,
        version: i32,
    ) -> IoResult<()> {
        if version != 2 {
            // Due to a bug where we were not versioning at all, reset so we don't have an issue deserializing
            r.seek(0, muo_io::SeekOrigin::Begin)?;
        }
        self.username = r.read_string()?.unwrap_or_default();
        self.password_algorithm = PasswordProtectionAlgorithm::from_i32(if version < 2 {
            r.read_i32()?
        } else {
            r.read_enum(4)? as i32
        });
        self.password = r.read_string()?;
        let level = if version < 2 {
            r.read_i32()?
        } else {
            r.read_enum(4)? as i32
        };
        self.access_level = AccessLevel::from_i32(level).unwrap_or_default();
        self.flags = r.read_i32()?;
        self.created = DateTime::from_ticks(r.read_date_time_ticks()?);
        self.last_login = DateTime::from_ticks(r.read_date_time_ticks()?);
        self.total_gold = r.read_i32()?;
        self.total_plat = r.read_i32()?;

        let n = r.read_i32()?.max(0);
        self.mobiles = Vec::new();
        for _ in 0..n {
            self.mobiles.push(r.read_mobile(w)?);
        }
        let n = r.read_i32()?.max(0);
        self.comments = Vec::new();
        for _ in 0..n {
            self.comments.push(AccountComment::deserialize(r)?);
        }
        let n = r.read_i32()?.max(0);
        self.tags = Vec::new();
        for _ in 0..n {
            self.tags.push(AccountTag::deserialize(r)?);
        }
        let n = r.read_i32()?.max(0);
        self.login_ips = Vec::new();
        for _ in 0..n {
            if version < 2 {
                if let Some(ip) = r.read_string()?.and_then(|s| s.parse().ok()) {
                    self.login_ips.push(ip);
                }
            } else {
                self.login_ips.push(r.read_ip_address()?);
            }
        }
        let n = r.read_i32()?.max(0);
        for _ in 0..n {
            r.read_string()?; // IP Restrictions
        }
        self.total_game_time = TimeSpan::from_ticks(r.read_time_span_ticks()?);
        if version > 1 {
            self.email = r.read_string()?;
        }
        Ok(())
    }
}

/// `Timer.DelayCall(AfterDeserialization)` for a freshly loaded account.
pub(crate) fn schedule_after_deserialization(w: &mut World, id: AccountId) {
    timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
        after_deserialization(w, id)
    });
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod gold_tests {
    use super::*;

    #[test]
    fn gold_and_plat_follow_the_currency_rules() {
        let mut a = Account::blank(Serial::new(1), DateTime::MIN);
        assert!(!a.deposit_gold(0, 1_000_000_000));
        assert!(a.deposit_gold(1_250_000_000, 1_000_000_000));
        assert_eq!((a.total_plat, a.total_gold), (1, 250_000_000));
        assert!(a.withdraw_gold(0)); // nothing to withdraw is success
        assert!(!a.withdraw_gold(250_000_001));
        assert!(a.withdraw_gold(250_000_000));
        assert!(!a.withdraw_plat(2));
        assert!(a.deposit_plat(3) && a.withdraw_plat(4));
        assert_eq!(a.get_total_gold(1_000_000_000), 0);
    }
}
