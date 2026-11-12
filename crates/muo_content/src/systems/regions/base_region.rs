//! `BaseRegion`.
//!
//! `BaseRegion`'s state ([`RegionData`], held in the region's `Ext`) also carries what its subclasses add
//! (`Entrance`, the guard fields), so one `Ext` type serves every class of `Regions/`. Its own virtuals are the
//! [`BaseRegionType`] family (the six capability properties, `CheckTravel`, and the `GuardedRegion` virtuals that
//! subclasses elsewhere override).

use indexmap::IndexMap;
use muo_io::text::TextDefinition;
use muo_primitives::dotnet::IgnoreCase;
use muo_primitives::{Point3D, Rectangle3D};
use muo_server::World;
use muo_server::ext::Ext;
use muo_server::ids::MobileId;
use muo_server::region::{REGION_BASE, RegionId, RegionJson, RegionType};
use muo_server::timer::TimerHandle;
use serde_json::Value;

use crate::spells::TravelCheckType;

/// What a `CheckTravel` answers: allowed, or refused with the message (`None` is the caller's default message).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TravelResult {
    pub allowed: bool,
    pub message: Option<TextDefinition>,
}

impl TravelResult {
    pub fn allow() -> Self {
        Self {
            allowed: true,
            message: None,
        }
    }

    /// `message = null; return false` ("use default message").
    pub fn deny() -> Self {
        Self {
            allowed: false,
            message: None,
        }
    }
}

/// The state of a `BaseRegion` (and `DungeonRegion`/`TownRegion`/`GuardedRegion`) instance.
#[derive(Default)]
pub struct RegionData {
    pub exclude_from_parent_spawns: bool,
    /// `Rectangles`, `RectangleWeights`, `TotalWeight`: set by `init_rectangles`.
    pub rectangles: Option<Vec<Rectangle3D>>,
    pub rectangle_weights: Vec<i32>,
    pub total_weight: i32,
    pub rune_name: Option<String>,
    pub no_logout_delay: bool,
    /// `DungeonRegion.Entrance` / `TownRegion.Entrance`.
    pub entrance: Point3D,
    /// `GuardedRegion.GuardType`: the guard class (full name); `None` is `DefaultGuardType`.
    pub guard_type: Option<String>,
    pub guards_disabled: bool,
    /// `m_GuardCandidates`: each candidate's 15 second timer.
    pub guard_candidates: IndexMap<MobileId, TimerHandle>,
}

/// `BaseRegion`'s virtuals, and `GuardedRegion`'s that classes outside `Regions/` override.
pub struct BaseRegionType {
    pub young_protected: bool,
    pub young_may_enter: bool,
    pub mounts_allowed: bool,
    pub dead_may_enter: bool,
    pub resurrection_allowed: bool,
    pub logout_allowed: bool,
    /// `CheckTravel(m, newLocation, travelType, out message)`.
    pub check_travel: fn(&mut World, RegionId, MobileId, Point3D, TravelCheckType) -> TravelResult,
    /// `GuardedRegion.AllowReds`.
    pub allow_reds: fn(&World, RegionId) -> bool,
    /// `GuardedRegion.DefaultGuardType`.
    pub default_guard_type: fn(&World, RegionId) -> &'static str,
    /// `GuardedRegion.IsDisabled()`.
    pub is_disabled: fn(&World, RegionId) -> bool,
}

/// `region.Ext` as [`RegionData`].
pub fn data(w: &World, id: RegionId) -> Option<&RegionData> {
    w.region(id)?.ext.get()
}

pub fn data_mut(w: &mut World, id: RegionId) -> Option<&mut RegionData> {
    w.region_mut(id)?.ext.get_mut()
}

/// Gives a region the state of a BaseRegion class (what its constructor did).
pub fn init_data(w: &mut World, id: RegionId) {
    if let Some(r) = w.region_mut(id)
        && r.ext.get::<RegionData>().is_none()
    {
        r.ext = Ext::new(RegionData::default());
    }
}

/// `region.ty.family::<BaseRegionType>()`.
pub fn family(w: &World, id: RegionId) -> Option<&'static BaseRegionType> {
    w.region(id)?.ty.family()
}

fn get<'a>(o: &'a RegionJson, key: &str) -> Option<&'a Value> {
    o.iter().find(|(k, _)| k.eq_ic(key)).map(|(_, v)| v)
}

/// A JSON bool property: `Err` for a value that isn't a bool.
pub fn json_bool(o: &RegionJson, key: &str) -> Result<Option<bool>, String> {
    match get(o, key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(_) => Err(format!("{key} must be a boolean")),
    }
}

/// A JSON string property.
pub fn json_string(o: &RegionJson, key: &str) -> Result<Option<String>, String> {
    match get(o, key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(format!("{key} must be a string")),
    }
}

/// A JSON `Point3D` property.
pub fn json_point(o: &RegionJson, key: &str) -> Result<Option<Point3D>, String> {
    match get(o, key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => muo_data::json::converters::point3d::deserialize(v.clone())
            .map(Some)
            .map_err(|e| e.to_string()),
    }
}

/// `BaseRegion`'s properties: `ExcludeFromParentSpawns`, `RuneName`, `NoLogoutDelay`.
pub fn base_from_json(w: &mut World, id: RegionId, o: &RegionJson) -> Result<(), String> {
    init_data(w, id);
    let exclude = json_bool(o, "ExcludeFromParentSpawns")?;
    let rune = json_string(o, "RuneName")?;
    let no_delay = json_bool(o, "NoLogoutDelay")?;
    let d = data_mut(w, id).ok_or("region is gone")?;
    if let Some(v) = exclude {
        d.exclude_from_parent_spawns = v;
    }
    if rune.is_some() {
        d.rune_name = rune;
    }
    if let Some(v) = no_delay {
        d.no_logout_delay = v;
    }
    Ok(())
}

/// `GetRuneNameFor(region)`: the first rune name up the parent chain.
pub fn get_rune_name_for(w: &World, mut region: Option<RegionId>) -> Option<String> {
    while let Some(r) = region {
        if let Some(name) = data(w, r).and_then(|d| d.rune_name.clone()) {
            return Some(name);
        }
        region = w.region(r).and_then(|r| r.parent);
    }
    None
}

/// `ToString()`: `Name ?? RuneName ?? GetType().Name`.
pub fn to_string(w: &World, id: RegionId) -> String {
    let Some(r) = w.region(id) else {
        return String::new();
    };
    r.name
        .clone()
        .or_else(|| data(w, id).and_then(|d| d.rune_name.clone()))
        .unwrap_or_else(|| r.ty.name.to_string())
}

// ---- the overrides ----------------------------------------------------------------------------------------------

/// `GetLogoutDelay(m)`.
fn get_logout_delay(w: &World, id: RegionId, m: MobileId) -> muo_primitives::TimeSpan {
    let no_delay = data(w, id).is_some_and(|d| d.no_logout_delay);
    let calm = w
        .mobile(m)
        .is_some_and(|m| m.aggressors.is_empty() && m.aggressed.is_empty() && !m.criminal);
    if no_delay && calm {
        muo_primitives::TimeSpan::ZERO
    } else {
        (REGION_BASE.get_logout_delay)(w, id, m)
    }
}

/// `OnEnter(m)`: a young player entering a region that doesn't protect the young gets a warning.
fn on_enter(w: &mut World, id: RegionId, m: MobileId) {
    let protected = family(w, id).is_none_or(|f| f.young_protected);
    let young = w
        .mobile(m)
        .is_some_and(|mob| mob.ty.is(&crate::player_mobile::PLAYER_MOBILE))
        && crate::player_mobile::young(w, m);
    if young
        && !protected
        && let Some(f) = w
            .resources
            .get::<RegionHooks>()
            .and_then(|h| h.young_dungeon_warning)
    {
        f(w, m);
    }
}

/// `AcceptsSpawnsFrom(region)`.
fn accepts_spawns_from(w: &World, id: RegionId, region: RegionId) -> bool {
    let exclude = data(w, id).is_some_and(|d| d.exclude_from_parent_spawns);
    (region == id || !exclude) && (REGION_BASE.accepts_spawns_from)(w, id, region)
}

/// Systems the regions call into that other crates own.
#[derive(Default)]
pub struct RegionHooks {
    /// Shows `YoungDungeonWarningGump`.
    pub young_dungeon_warning: Option<fn(&mut World, MobileId)>,
    /// `BaseGuard.Spawn(caller, target)`: a guard called by a citizen next to the criminal.
    pub guard_spawn_from_mobile: Option<fn(&mut World, MobileId, MobileId)>,
    /// `BaseGuard.Spawn(region, target)`.
    pub guard_spawn_from_region: Option<fn(&mut World, RegionId, MobileId)>,
    /// `GuardType.CreateInstance(focus)`: makes a guard of the class (full name) with this focus.
    pub make_guard: Option<fn(&mut World, &str, MobileId)>,
    /// `TransformationSpellHelper.UnderTransformation(m, typeof(AnimalForm))`.
    pub under_animal_form: Option<fn(&World, MobileId) -> bool>,
    /// The `SafeZone` region class (ConPVP).
    pub safe_zone: Option<&'static RegionType>,
}

/// `CheckTravel` of a region that is a BaseRegion class: `None` for any other region class.
pub fn check_travel(
    w: &mut World,
    id: RegionId,
    m: MobileId,
    loc: Point3D,
    t: TravelCheckType,
) -> Option<TravelResult> {
    let f = family(w, id)?;
    Some((f.check_travel)(w, id, m, loc, t))
}

/// `BaseRegion.CheckTravel` itself: the parent BaseRegion's answer, else allowed.
fn base_check_travel(
    w: &mut World,
    id: RegionId,
    m: MobileId,
    loc: Point3D,
    t: TravelCheckType,
) -> TravelResult {
    if let Some(parent) = w.region(id).and_then(|r| r.parent)
        && let Some(res) = check_travel(w, parent, m, loc, t)
    {
        return res;
    }
    TravelResult::allow()
}

fn default_allow_reds(w: &World, _: RegionId) -> bool {
    crate::core::aos(w)
}

fn default_guard_type_of(w: &World, id: RegionId) -> &'static str {
    let map = w.region(id).map(|r| r.map);
    if map == Some(muo_server::map::MapId::ILSHENAR) || map == Some(muo_server::map::MapId::MALAS) {
        "Server.Mobiles.ArcherGuard"
    } else {
        "Server.Mobiles.WarriorGuard"
    }
}

fn guards_disabled(w: &World, id: RegionId) -> bool {
    data(w, id).is_some_and(|d| d.guards_disabled)
}

pub static BASE_REGION_FAMILY: BaseRegionType = BaseRegionType {
    young_protected: true,
    young_may_enter: true,
    mounts_allowed: true,
    dead_may_enter: true,
    resurrection_allowed: true,
    logout_allowed: true,
    check_travel: base_check_travel,
    allow_reds: default_allow_reds,
    default_guard_type: default_guard_type_of,
    is_disabled: guards_disabled,
};

/// `BaseRegion`.
pub static BASE_REGION: RegionType = RegionType {
    name: "BaseRegion",
    parent: Some(&REGION_BASE),
    from_json: base_from_json,
    get_logout_delay,
    on_enter,
    accepts_spawns_from,
    family: Some(&BASE_REGION_FAMILY),
    ..REGION_BASE
};

// ---- InitRectangles ---------------------------------------------------------------------------------------------

/// `InitRectangles()`: tests whether the area rectangles overlap and, if so, breaks them into smaller
/// non-overlapping rectangles; then the weights (areas) used to pick a random spot.
pub fn init_rectangles(w: &mut World, id: RegionId) {
    init_data(w, id);
    if data(w, id).is_some_and(|d| d.rectangles.is_some()) {
        return;
    }
    let Some(area) = w.region(id).map(|r| r.area.clone()) else {
        return;
    };
    let rects = split_overlaps(&area);

    let weights: Vec<i32> = rects.iter().map(|r| r.width() * r.height()).collect();
    let total = weights.iter().sum();
    if let Some(d) = data_mut(w, id) {
        d.rectangles = Some(rects);
        d.rectangle_weights = weights;
        d.total_weight = total;
    }
}

/// The overlap splitting of `InitRectangles`.
pub fn split_overlaps(area: &[Rectangle3D]) -> Vec<Rectangle3D> {
    let mut buffer1: Vec<Rectangle3D> = Vec::new();
    let mut buffer2: Vec<Rectangle3D> = Vec::new();

    for a in area {
        buffer2.push(*a);

        let mut j = 0;
        while j < buffer1.len() && !buffer2.is_empty() {
            let comp = buffer1[j];

            let mut k = buffer2.len() as isize - 1;
            while k >= 0 {
                let rect = buffer2[k as usize];

                let (l1, r1, t1, b1) = (rect.start.x, rect.end.x, rect.start.y, rect.end.y);
                let (l2, r2, t2, b2) = (comp.start.x, comp.end.x, comp.start.y, comp.end.y);

                if l1 < r2 && r1 > l2 && t1 < b2 && b1 > t2 {
                    buffer2.remove(k as usize);

                    let sz = rect.start.z;
                    let ez = rect.end.z;

                    let r3 = |x1, y1, x2, y2| {
                        Rectangle3D::from_points(Point3D::new(x1, y1, sz), Point3D::new(x2, y2, ez))
                    };

                    if l1 < l2 {
                        buffer2.push(r3(l1, t1, l2, b1));
                    }

                    if r1 > r2 {
                        buffer2.push(r3(r2, t1, r1, b1));
                    }

                    if t1 < t2 {
                        buffer2.push(r3(l1.max(l2), t1, r1.min(r2), t2));
                    }

                    if b1 > b2 {
                        buffer2.push(r3(l1.max(l2), b2, r1.min(r2), b1));
                    }
                }
                k -= 1;
            }
            j += 1;
        }

        buffer1.append(&mut buffer2);
    }

    buffer1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x1: i32, y1: i32, x2: i32, y2: i32) -> Rectangle3D {
        Rectangle3D::from_points(Point3D::new(x1, y1, -128), Point3D::new(x2, y2, 128))
    }

    #[test]
    fn disjoint_rectangles_are_kept() {
        let out = split_overlaps(&[r(0, 0, 10, 10), r(20, 20, 30, 30)]);
        assert_eq!(out, vec![r(0, 0, 10, 10), r(20, 20, 30, 30)]);
    }

    #[test]
    fn overlapping_rectangles_are_cut_into_non_overlapping_pieces() {
        let out = split_overlaps(&[r(0, 0, 10, 10), r(5, 5, 15, 15)]);
        // The union of the two squares: 100 + 100 - 25.
        let area: i32 = out.iter().map(|r| r.width() * r.height()).sum();
        assert_eq!(area, 175);
        for (i, a) in out.iter().enumerate() {
            for b in &out[i + 1..] {
                let overlap = a.start.x < b.end.x
                    && a.end.x > b.start.x
                    && a.start.y < b.end.y
                    && a.end.y > b.start.y;
                assert!(!overlap, "{a:?} overlaps {b:?}");
            }
        }
    }

    #[test]
    fn init_rectangles_weights() {
        let mut w = World::new();
        let id = muo_server::region::new_region(
            &mut w,
            &BASE_REGION,
            Some("x"),
            muo_server::map::MapId::FELUCCA,
            None,
            Some(50),
            vec![r(0, 0, 10, 10), r(5, 5, 15, 15)],
        );
        init_rectangles(&mut w, id);
        let d = data(&w, id).unwrap();
        assert_eq!(d.total_weight, d.rectangle_weights.iter().sum::<i32>());
        assert_eq!(
            d.rectangles.as_ref().unwrap().len(),
            d.rectangle_weights.len()
        );
        let before = d.total_weight;
        init_rectangles(&mut w, id); // idempotent
        assert_eq!(data(&w, id).unwrap().total_weight, before);
    }

    #[test]
    fn rune_names_come_from_the_nearest_ancestor_and_to_string_falls_back() {
        let mut w = World::new();
        let map = muo_server::map::MapId::FELUCCA;
        let parent =
            muo_server::region::new_region(&mut w, &BASE_REGION, None, map, None, Some(50), vec![]);
        init_data(&mut w, parent);
        data_mut(&mut w, parent).unwrap().rune_name = Some("Somewhere".into());
        let child = muo_server::region::new_region(
            &mut w,
            &BASE_REGION,
            Some("Inn"),
            map,
            Some(parent),
            None,
            vec![],
        );
        init_data(&mut w, child);
        assert_eq!(
            get_rune_name_for(&w, Some(child)).as_deref(),
            Some("Somewhere")
        );
        assert_eq!(get_rune_name_for(&w, None), None);
        assert_eq!(to_string(&w, child), "Inn");
        assert_eq!(to_string(&w, parent), "Somewhere");
        data_mut(&mut w, parent).unwrap().rune_name = None;
        assert_eq!(to_string(&w, parent), "BaseRegion");
    }
}
