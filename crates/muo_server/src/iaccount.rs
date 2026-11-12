//! Accounts as the core sees them: `AccountGold`, `IGoldAccount`, `IAccount`.
//!
//! The account objects live in the accounting module; `Mobile::account` holds an [`AccountId`].

use crate::ids::MobileId;
use crate::mobile::AccessLevel;

/// An `IAccount` reference.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct AccountId(pub u32);

/// `AccountGold` statics, held in the world's resources.
pub struct AccountGold {
    pub enabled: bool,
    pub currency_threshold: i32,
    pub convert_on_bank: bool,
    pub convert_on_trade: bool,
}

impl Default for AccountGold {
    fn default() -> Self {
        AccountGold {
            enabled: false,
            currency_threshold: 1_000_000_000,
            convert_on_bank: false,
            convert_on_trade: false,
        }
    }
}

impl AccountGold {
    /// `AccountGold.Configure`; `tol` is `Core.TOL`, `get` is `ServerConfiguration.GetSetting(key, default)`.
    pub fn configure(&mut self, tol: bool, get: impl Fn(&str, bool) -> bool) {
        self.enabled = get("accountGold.enable", tol);
        self.convert_on_bank = get("accountGold.convertOnBank", true);
        self.convert_on_trade = get("accountGold.convertOnTrade", false);
    }
}

/// `IGoldAccount`.
pub trait GoldAccount {
    fn total_gold(&self) -> i32;
    fn total_plat(&self) -> i32;
    fn deposit_gold(&mut self, amount: i32) -> bool;
    fn deposit_plat(&mut self, amount: i32) -> bool;
    fn withdraw_gold(&mut self, amount: i32) -> bool;
    fn withdraw_plat(&mut self, amount: i32) -> bool;
    fn get_total_gold(&self) -> i64;
}

/// `IAccount` (minus `ISerializable` and `IComparable`, which the accounting system adds).
pub trait Account: GoldAccount {
    fn username(&self) -> &str;
    fn email(&self) -> &str;
    fn set_email(&mut self, email: String);
    fn access_level(&self) -> AccessLevel;
    fn set_access_level(&mut self, level: AccessLevel);
    /// `Length`.
    fn length(&self) -> i32;
    /// `Limit`.
    fn limit(&self) -> i32;
    /// `Count`.
    fn count(&self) -> i32;
    /// The indexer getter.
    fn mobile(&self, index: i32) -> Option<MobileId>;
    /// The indexer setter.
    fn set_mobile(&mut self, index: i32, m: Option<MobileId>);
    fn delete(&mut self);
    fn try_set_username(&mut self, username: &str) -> bool;
    fn set_password(&mut self, password: &str);
    fn check_password(&self, password: &str) -> bool;
}

/// What `Mobile.Account` answers for gold (`IGoldAccount` through the mobile's account). The accounts are
/// content's accounting system; it fills these in on registration.
// Unset (as in a world without accounting) a mobile has no account; muo_content's accounting fills these in.
#[derive(Default, Clone, Copy)]
pub struct AccountHooks {
    /// `(Account.TotalGold, Account.TotalPlat)`, `None` when the mobile has no account.
    #[allow(clippy::type_complexity)]
    pub totals: Option<fn(&crate::World, MobileId) -> Option<(i32, i32)>>,
    pub withdraw_gold: Option<fn(&mut crate::World, MobileId, i32) -> bool>,
    pub deposit_gold: Option<fn(&mut crate::World, MobileId, i32) -> bool>,
    pub withdraw_plat: Option<fn(&mut crate::World, MobileId, i32) -> bool>,
    pub deposit_plat: Option<fn(&mut crate::World, MobileId, i32) -> bool>,
}

fn hooks(w: &crate::World) -> AccountHooks {
    w.resources
        .get::<AccountHooks>()
        .copied()
        .unwrap_or_default()
}

/// `AccountGold.Enabled`.
pub fn gold_enabled(w: &crate::World) -> bool {
    w.resources.get::<AccountGold>().is_some_and(|a| a.enabled)
}

/// `(m.Account.TotalGold, m.Account.TotalPlat)` when the mobile has an account.
pub fn account_totals(w: &crate::World, m: MobileId) -> Option<(i32, i32)> {
    hooks(w).totals.and_then(|f| f(w, m))
}

/// `Account.WithdrawGold(n)` and its three siblings; false without an account.
pub fn account_withdraw_gold(w: &mut crate::World, m: MobileId, n: i32) -> bool {
    hooks(w).withdraw_gold.is_some_and(|f| f(w, m, n))
}
pub fn account_deposit_gold(w: &mut crate::World, m: MobileId, n: i32) -> bool {
    hooks(w).deposit_gold.is_some_and(|f| f(w, m, n))
}
pub fn account_withdraw_plat(w: &mut crate::World, m: MobileId, n: i32) -> bool {
    hooks(w).withdraw_plat.is_some_and(|f| f(w, m, n))
}
pub fn account_deposit_plat(w: &mut crate::World, m: MobileId, n: i32) -> bool {
    hooks(w).deposit_plat.is_some_and(|f| f(w, m, n))
}
