//! House decay (`DecayType`, `DecayLevel`, `UpdateDecay`, `CheckDecay`, `Decay_Sandbox`) and `DynamicDecay`.
//!
//! `Sign.GettingProperties` is a `Cell` on the sign (`crate::sign::getting_properties`): `UpdateDecay` does not
//! invalidate the sign's properties while the sign is building them.

use muo_content::core;
use muo_content::regions::RegionData;
use muo_primitives::{DateTime, Point3D, Rectangle2D, TimeSpan, utility};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::{ItemId, MobileId};
use muo_server::map::MapId;
use muo_server::mobile::AccessLevel;
use muo_server::region::{self, RegionId, RegionType};
use muo_server::timer;

use super::{
    DecayLevel, DecayType, access, d, data, data_mut, decay_period, deleted, is_aos_rules, owner,
    sign, statics_ref,
};

/// `DynamicDecay.Enabled`.
pub fn dynamic_enabled(w: &World) -> bool {
    core::ml(w)
}

/// `DynamicDecay.Decays(level)`: the stages with a registered duration.
pub fn dynamic_decays(level: DecayLevel) -> bool {
    stage_range(level).is_some()
}

fn stage_range(level: DecayLevel) -> Option<(TimeSpan, TimeSpan)> {
    Some(match level {
        DecayLevel::LikeNew => (TimeSpan::from_hours(1), TimeSpan::from_hours(1)),
        DecayLevel::Slightly | DecayLevel::Somewhat | DecayLevel::Fairly | DecayLevel::Greatly => {
            (TimeSpan::from_days(1), TimeSpan::from_days(2))
        }
        DecayLevel::IDOC => (TimeSpan::from_hours(12), TimeSpan::from_hours(24)),
        _ => return None,
    })
}

/// `DynamicDecay.GetRandomDuration(level)`.
pub fn dynamic_random_duration(w: &mut World, level: DecayLevel) -> TimeSpan {
    let Some((min, max)) = stage_range(level) else {
        return TimeSpan::ZERO;
    };
    let (min, max) = (min.ticks(), max.ticks());
    TimeSpan::from_ticks(min + (utility::random_double(&mut w.rng) * (max - min) as f64) as i64)
}

/// `BaseHouse.DecayEnabled`.
pub fn decay_enabled(w: &World) -> bool {
    statics_ref(w).is_some_and(|s| s.decay_enabled)
}

/// `Account.Inactive`.
fn account_inactive(w: &World, m: MobileId) -> bool {
    let Some(a) = w.mobile(m).and_then(|x| x.account) else {
        return false;
    };
    let Some(acct) = muo_content::accounting::account(w, a) else {
        return false;
    };
    if acct.access_level != AccessLevel::Player {
        return false;
    }
    let empty = access::account_mobiles(w, m).is_empty();
    w.now - acct.last_login
        > if empty {
            muo_content::accounting::EMPTY_INACTIVE_DURATION
        } else {
            muo_content::accounting::INACTIVE_DURATION
        }
}

/// `IsNewer(check, house)`.
pub fn is_newer(w: &World, check: ItemId, house: ItemId) -> bool {
    let (c, h) = (d(w, check), d(w, house));
    let check_time = if c.last_traded > c.built_on {
        c.last_traded
    } else {
        c.built_on
    };
    let house_time = if h.last_traded > h.built_on {
        h.last_traded
    } else {
        h.built_on
    };
    check_time > house_time
}

/// `DecayType`.
pub fn decay_type(w: &World, house: ItemId) -> DecayType {
    let h = d(w, house);
    if h.restrict_decay || !decay_enabled(w) || decay_period(w, house) == TimeSpan::ZERO {
        return DecayType::Ageless;
    }
    let condemned = if core::aos(w) {
        DecayType::Condemned
    } else {
        DecayType::ManualRefresh
    };
    let Some(o) = h.owner else { return condemned };
    let Some(acct) = w
        .mobile(o)
        .and_then(|m| m.account)
        .and_then(|a| muo_content::accounting::account(w, a))
    else {
        return condemned;
    };
    if acct.access_level >= AccessLevel::GameMaster {
        return DecayType::Ageless;
    }
    let chars = access::account_mobiles(w, o);
    if chars.iter().any(|&c| {
        w.mobile(c)
            .is_some_and(|m| m.access_level >= AccessLevel::GameMaster)
    }) {
        return DecayType::Ageless;
    }
    if !core::aos(w) {
        return DecayType::ManualRefresh;
    }
    if account_inactive(w, o) {
        return DecayType::Condemned;
    }
    let mut newest: Option<ItemId> = None;
    for c in chars {
        for check in access::get_houses(w, c) {
            if newest.is_none_or(|n| is_newer(w, check, n)) {
                newest = Some(check);
            }
        }
    }
    if newest == Some(house) {
        DecayType::AutoRefresh
    } else {
        DecayType::ManualRefresh
    }
}

/// `CanDecay`.
pub fn can_decay(w: &World, house: ItemId) -> bool {
    matches!(
        decay_type(w, house),
        DecayType::Condemned | DecayType::ManualRefresh
    )
}

fn has_rented_vendors(w: &World, house: ItemId) -> bool {
    let h = d(w, house);
    let f = muo_content::items::house_deps::deps(w).and_then(|x| x.vendor_is_rented);
    h.player_vendors.iter().any(|&v| f.is_some_and(|f| f(w, v)))
}

/// `HasRentedVendors`.
pub fn rented_vendors(w: &World, house: ItemId) -> bool {
    has_rented_vendors(w, house)
}

/// `GetOldDecayLevel()`.
pub fn old_decay_level(w: &World, house: ItemId) -> DecayLevel {
    let h = d(w, house);
    let after = w.now - h.last_refreshed;
    let period = decay_period(w, house).ticks();
    let percent = if period == 0 {
        i64::MAX
    } else {
        after.ticks().saturating_mul(1000) / period
    };

    if percent >= 1000 {
        // 100.0%
        return if has_rented_vendors(w, house) || !h.vendor_inventories.is_empty() {
            DecayLevel::DemolitionPending
        } else {
            DecayLevel::Collapsed
        };
    }
    if percent >= 950 {
        return DecayLevel::IDOC;
    }
    if percent >= 750 {
        return DecayLevel::Greatly;
    }
    if percent >= 500 {
        return DecayLevel::Fairly;
    }
    if percent >= 250 {
        return DecayLevel::Somewhat;
    }
    if percent >= 5 {
        return DecayLevel::Slightly;
    }
    DecayLevel::LikeNew
}

/// `DecayLevel` getter.
pub fn decay_level(w: &World, house: ItemId) -> DecayLevel {
    if !can_decay(w, house) {
        return DecayLevel::Ageless;
    }
    if dynamic_enabled(w) {
        let h = d(w, house);
        let stage = h.current_stage;
        if stage == DecayLevel::Collapsed
            && (has_rented_vendors(w, house) || !h.vendor_inventories.is_empty())
        {
            return DecayLevel::DemolitionPending;
        }
        return stage;
    }
    old_decay_level(w, house)
}

/// `ResetDynamicDecay()`.
pub fn reset_dynamic_decay(w: &mut World, house: ItemId) {
    if let Some(h) = data_mut(w, house) {
        h.current_stage = DecayLevel::Ageless;
        h.next_decay_stage = DateTime::MIN;
    }
}

/// `SetDynamicDecay(level)`.
pub fn set_dynamic_decay(w: &mut World, house: ItemId, level: DecayLevel) {
    let next = if dynamic_decays(level) {
        w.now + dynamic_random_duration(w, level)
    } else {
        DateTime::MIN
    };
    if let Some(h) = data_mut(w, house) {
        h.current_stage = level;
        h.next_decay_stage = next;
    }
}

/// `UpdateDecay()`: advances the decay bookkeeping; runs from the decay tick, never from a read.
pub fn update_decay(w: &mut World, house: ItemId) {
    if !can_decay(w, house) {
        let stage = d(w, house).current_stage;
        if dynamic_enabled(w) && stage != DecayLevel::Ageless {
            reset_dynamic_decay(w, house);
        }
        if let Some(h) = data_mut(w, house) {
            h.was_undecayable = true;
        }
    } else {
        if d(w, house).was_undecayable {
            // Leaving the undecayable state counts as a refresh, which the old getter guaranteed by restamping
            // LastRefreshed on every read while undecayable.
            let now = w.now;
            if let Some(h) = data_mut(w, house) {
                h.was_undecayable = false;
                h.last_refreshed = now;
            }
            if dynamic_enabled(w) {
                reset_dynamic_decay(w, house);
            }
        }
        if dynamic_enabled(w) {
            let h = d(w, house);
            let stage = h.current_stage;
            if stage == DecayLevel::Ageless
                || (dynamic_decays(stage) && h.next_decay_stage <= w.now)
            {
                set_dynamic_decay(w, house, DecayLevel::from_i32(stage as i32 + 1));
            }
        }
    }

    let level = decay_level(w, house);
    if level != d(w, house).last_decay_level {
        if let Some(h) = data_mut(w, house) {
            h.last_decay_level = level;
        }
        if let Some(s) = sign(w, house).filter(|&s| !crate::sign::getting_properties(w, s)) {
            muo_server::item::ops::invalidate_properties(w, s);
        }
    }
}

/// `RefreshDecay()`.
pub fn refresh_decay(w: &mut World, house: ItemId) -> bool {
    if decay_type(w, house) == DecayType::Condemned {
        return false;
    }
    update_decay(w, house);
    let old = decay_level(w, house);
    let now = w.now;
    if let Some(h) = data_mut(w, house) {
        h.last_refreshed = now;
    }
    if dynamic_enabled(w) {
        reset_dynamic_decay(w, house);
    }
    if let Some(s) = sign(w, house) {
        muo_server::item::ops::invalidate_properties(w, s);
    }
    old > DecayLevel::LikeNew
}

/// `CheckDecay()`.
pub fn check_decay(w: &mut World, house: ItemId) -> bool {
    update_decay(w, house);
    if !deleted(w, house) && decay_level(w, house) == DecayLevel::Collapsed {
        timer::start(w, TimeSpan::ZERO, TimeSpan::ZERO, 1, move |w| {
            decay_sandbox(w, house)
        });
        return true;
    }
    false
}

/// `Decay_OnTick()`: every house, once a minute.
pub fn decay_on_tick(w: &mut World) {
    let all = statics_ref(w)
        .map(|s| s.all_houses.clone())
        .unwrap_or_default();
    for h in all {
        if !deleted(w, h) || data(w, h).is_some() {
            check_decay(w, h);
        }
    }
}

/// `Decay_Sandbox()`.
pub fn decay_sandbox(w: &mut World, house: ItemId) {
    if deleted(w, house) {
        return;
    }
    if core::ml(w) {
        new_temp_no_housing_region(w, house, None);
    }
    super::storage::kill_vendors(w, house);
    let ty = w.item(house).map(|i| i.ty);
    if let Some(ty) = ty {
        (ty.delete)(w, house);
    }
}

// ---- TreasureRegion ----

/// `TreasureRegion`: a base region around a treasure location (a plain `BaseRegion` of its own type).
pub static TREASURE_REGION: RegionType = RegionType {
    name: "TreasureRegion",
    parent: Some(&muo_content::regions::BASE_REGION),
    allow_housing: |_, _, _, _| false,
    on_enter: |w, _, m| {
        if w.mobile(m)
            .is_some_and(|x| x.access_level > AccessLevel::Player)
        {
            muo_server::mobile::part2::messages::send_message(
                w,
                m,
                "You have entered a protected treasure map area.",
            );
        }
    },
    on_exit: |w, _, m| {
        if w.mobile(m)
            .is_some_and(|x| x.access_level > AccessLevel::Player)
        {
            muo_server::mobile::part2::messages::send_message(
                w,
                m,
                "You have left a protected treasure map area.",
            );
        }
    },
    ..muo_content::regions::BASE_REGION
};

/// `new TreasureRegion(x, y, map)`: no house may be placed within 5 tiles of the treasure.
pub fn new_treasure_region(w: &mut World, x: i32, y: i32, map: MapId) -> RegionId {
    const RANGE: i32 = 5;
    let area = region::convert_all_to_3d(&[Rectangle2D::new(
        x - RANGE,
        y - RANGE,
        1 + RANGE * 2,
        1 + RANGE * 2,
    )]);
    let id = region::new_region(
        w,
        &TREASURE_REGION,
        None,
        map,
        None,
        Some(region::DEFAULT_PRIORITY),
        area,
    );
    let z = w.maps.get_average_z(map, x, y);
    if let Some(r) = w.region_mut(id) {
        r.ext = Ext::new(RegionData::default());
        r.go_location = Point3D::new(x, y, z);
    }
    region::register(w, id);
    id
}

/// `TreasureRegion.Initialize()`: `Data/treasure.cfg` (`x y` per line), a region in Felucca and Trammel each.
pub fn initialize_treasure_regions(w: &mut World) {
    let path = w.core.base_directory.join("Data/treasure.cfg");
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    for line in text.lines() {
        let mut split = line.split(' ');
        let (Some(x), Some(y)) = (
            split.next().and_then(|x| x.parse::<i32>().ok()),
            split.next().and_then(|y| y.parse::<i32>().ok()),
        ) else {
            continue; // "Warning: Error in Line ..."
        };
        new_treasure_region(w, x, y, MapId::FELUCCA);
        new_treasure_region(w, x, y, MapId::TRAMMEL);
    }
}

// ---- TempNoHousingRegion ----

struct TempNoHousingOwner(Option<MobileId>);

/// `TempNoHousingRegion`: houses cannot be placed on a collapsed house' plot for an hour, except by its owner.
pub static TEMP_NO_HOUSING_REGION: RegionType = RegionType {
    name: "TempNoHousingRegion",
    parent: Some(&muo_content::regions::BASE_REGION),
    allow_housing: |w, id, from, _| {
        let owner = w
            .region(id)
            .and_then(|r| r.ext.get::<TempNoHousingOwner>())
            .and_then(|o| o.0);
        owner == Some(from) || access::check_account(w, from, owner)
    },
    ..muo_content::regions::BASE_REGION
};

/// `new TempNoHousingRegion(house, regionOwner)`.
pub fn new_temp_no_housing_region(
    w: &mut World,
    house: ItemId,
    region_owner: Option<MobileId>,
) -> Option<RegionId> {
    let map = super::map_of(w, house)?;
    let house_region = super::region(w, house)?;
    let area = w.region(house_region)?.area.clone();
    let id = region::new_region(
        w,
        &TEMP_NO_HOUSING_REGION,
        None,
        map,
        None,
        Some(region::DEFAULT_PRIORITY),
        area,
    );
    if let Some(r) = w.region_mut(id) {
        r.ext = Ext::new(RegionData::default());
        r.ext.insert(TempNoHousingOwner(region_owner));
    }
    region::register(w, id);
    timer::start(
        w,
        super::restricted_placing_time(),
        TimeSpan::ZERO,
        1,
        move |w| region::unregister(w, id),
    );
    Some(id)
}

/// Whether the house rules apply to the item (used by the sign).
pub fn is_aos(w: &World, house: ItemId) -> bool {
    is_aos_rules(w, house)
}

/// `Owner?.Name`.
pub fn owner_name(w: &World, house: ItemId) -> Option<String> {
    owner(w, house).and_then(|o| muo_server::mobile::base::name(w, o))
}
