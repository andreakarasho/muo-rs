//! Small misc systems: Animations, AttackMessage, FoodDecay, Geometry, NameList.

use muo_primitives::MessageType;
use muo_primitives::dotnet::InvariantCase;
use muo_primitives::{DateTime, Point2D, Point3D, TimeSpan};
use muo_server::World;
use muo_server::ids::MobileId;
use muo_server::map::MapId;
use muo_server::mobile::base;
use muo_server::mobile::part1::props_a;
use muo_server::mobile::part2::messages;
use muo_server::timer;

// ---- Animations ----

/// `Animations.AnimateRequest(from, actionName)`: the client's `bow` and `salute`.
pub fn animate_request(w: &mut World, from: MobileId, action_name: &str) {
    let action = match action_name {
        "bow" => 32,
        "salute" => 33,
        _ => 0,
    };
    if action <= 0 {
        return;
    }
    let Some(m) = w.mobile(from) else { return };
    let (ty, body) = (m.ty, muo_server::mobile::part1::props_b::body(w, from));
    let human = w
        .resources
        .get::<muo_primitives::BodyTable>()
        .is_some_and(|t| t.is_human(body));
    if (ty.alive)(w, from) && !muo_server::mobile::part1::props_b::mounted(w, from) && human {
        base::animate(w, from, action, 5, 1, true, false, 0);
    }
}

// ---- AttackMessage ----

const ATTACK_HUE: i32 = 0x22;

/// `AttackMessage.CheckAggressions(m1, m2)`: whether either already attacked the other within the last minute.
pub fn check_aggressions(w: &World, m1: MobileId, m2: MobileId) -> bool {
    let delay = TimeSpan::from_mins(1);
    let recent = |owner: MobileId, other: MobileId| {
        w.mobile(owner).is_some_and(|m| {
            m.aggressors
                .iter()
                .any(|i| i.attacker == other && w.now < i.last_combat_time + delay)
        })
    };
    recent(m1, m2) || recent(m2, m1)
}

/// `AttackMessage.EventSink_AggressiveAction`.
pub fn on_aggressive_action(w: &mut World, aggressor: MobileId, aggressed: MobileId) {
    let player = |w: &World, id| w.mobile(id).is_some_and(|m| m.player);
    if !player(w, aggressor) || !player(w, aggressed) {
        return;
    }
    if !check_aggressions(w, aggressor, aggressed) {
        let name_of = |w: &World, id: MobileId| {
            w.mobile(id)
                .and_then(|m| (m.ty.name)(w, id))
                .unwrap_or_default()
        };
        let (a, d) = (name_of(w, aggressor), name_of(w, aggressed));
        messages::local_overhead_message(
            w,
            aggressor,
            MessageType::REGULAR,
            ATTACK_HUE,
            true,
            &format!("You are attacking {d}!"),
        );
        messages::local_overhead_message(
            w,
            aggressed,
            MessageType::REGULAR,
            ATTACK_HUE,
            true,
            &format!("{a} is attacking you!"),
        );
    }
}

/// `AttackMessage.Initialize`.
pub fn attack_message_initialize(w: &mut World) {
    w.events
        .aggressive_action
        .subscribe(|w, e| on_aggressive_action(w, e.aggressor, e.aggressed));
}

// ---- FoodDecay ----

/// `FoodDecayTimer.HungerDecay`.
pub fn hunger_decay(w: &mut World, m: MobileId) {
    if let Some(h) = w.mobile(m).map(|x| x.hunger).filter(|h| *h >= 1) {
        props_a::set_hunger(w, m, h - 1);
    }
}

/// `FoodDecayTimer.ThirstDecay`.
pub fn thirst_decay(w: &mut World, m: MobileId) {
    if let Some(t) = w.mobile(m).map(|x| x.thirst).filter(|t| *t >= 1) {
        props_a::set_thirst(w, m, t - 1);
    }
}

/// `FoodDecayTimer.FoodDecay`: every connected mobile loses a point of each.
pub fn food_decay(w: &mut World) {
    let mobiles: Vec<MobileId> = w.net_states.values().filter_map(|d| d.mobile).collect();
    for m in mobiles {
        hunger_decay(w, m);
        thirst_decay(w, m);
    }
}

/// `FoodDecayTimer.Initialize`: every five minutes.
pub fn food_decay_initialize(w: &mut World) {
    timer::start(
        w,
        TimeSpan::from_mins(5),
        TimeSpan::from_mins(5),
        0,
        food_decay,
    );
}

// ---- Geometry ----

pub fn radians_to_degrees(angle: f64) -> f64 {
    angle * (180.0 / std::f64::consts::PI)
}

pub fn degrees_to_radians(angle: f64) -> f64 {
    angle * (std::f64::consts::PI / 180.0)
}

/// `Geometry.ArcPoint`.
pub fn arc_point(loc: Point3D, radius: i32, angle: i32) -> Point2D {
    let angle = angle.clamp(0, 90);
    // Math.Round is banker's rounding
    let side_a =
        (f64::from(radius) * degrees_to_radians(f64::from(angle)).sin()).round_ties_even() as i32;
    let side_b =
        (f64::from(radius) * degrees_to_radians(f64::from(angle)).cos()).round_ties_even() as i32;
    Point2D {
        x: loc.x - side_b,
        y: loc.y - side_a,
    }
}

/// `Geometry.CirclePoint`.
#[derive(Copy, Clone, Debug)]
pub struct CirclePoint {
    pub point: Point2D,
    pub angle: i32,
    pub quadrant: i32,
}

/// `Geometry.Circle2D(loc, map, radius, effect)`.
pub fn circle_2d(loc: Point3D, map: MapId, radius: i32, effect: &mut dyn FnMut(Point3D, MapId)) {
    circle_2d_arc(loc, map, radius, effect, 0, 360);
}

/// `Geometry.Circle2D(loc, map, radius, effect, angleStart, angleEnd)`.
pub fn circle_2d_arc(
    loc: Point3D,
    map: MapId,
    radius: i32,
    effect: &mut dyn FnMut(Point3D, MapId),
    mut angle_start: i32,
    mut angle_end: i32,
) {
    if !(0..=360).contains(&angle_start) {
        angle_start = 0;
    }
    if !(0..=360).contains(&angle_end) {
        angle_end = 360;
    }
    if angle_start == angle_end {
        return;
    }
    let opposite = angle_start > angle_end;
    let mut start_quadrant = angle_start / 90;
    let mut end_quadrant = angle_end / 90;
    let mut start = arc_point(loc, radius, angle_start % 90);
    let mut end = arc_point(loc, radius, angle_end % 90);
    if opposite {
        std::mem::swap(&mut start, &mut end);
        std::mem::swap(&mut start_quadrant, &mut end_quadrant);
    }
    let start_point = CirclePoint {
        point: start,
        angle: angle_start,
        quadrant: start_quadrant,
    };
    let end_point = CirclePoint {
        point: end,
        angle: angle_end,
        quadrant: end_quadrant,
    };

    let mut error = -radius;
    let mut x = radius;
    let mut y = 0;
    while x > y {
        plot4points(loc, map, x, y, start_point, end_point, effect, opposite);
        plot4points(loc, map, y, x, start_point, end_point, effect, opposite);
        error += y * 2 + 1;
        y += 1;
        if error >= 0 {
            x -= 1;
            error -= x * 2;
        }
    }
    plot4points(loc, map, x, y, start_point, end_point, effect, opposite);
}

/// `Geometry.plot4points`.
#[allow(clippy::too_many_arguments)]
pub fn plot4points(
    loc: Point3D,
    map: MapId,
    x: i32,
    y: i32,
    start: CirclePoint,
    end: CirclePoint,
    effect: &mut dyn FnMut(Point3D, MapId),
    opposite: bool,
) {
    let point_a = Point2D {
        x: loc.x - x,
        y: loc.y - y,
    };
    let point_b = Point2D {
        x: loc.x - y,
        y: loc.y - x,
    };
    let p3 = |x, y| Point3D { x, y, z: loc.z };

    let mut quadrant = 2;
    if x == 0 && start.quadrant == 3 {
        quadrant = 3;
    }
    if within_circle_bounds(
        if quadrant == 3 { point_b } else { point_a },
        quadrant,
        start,
        end,
        opposite,
    ) {
        effect(p3(loc.x + x, loc.y + y), map);
    }

    quadrant = 3;
    if y == 0 && start.quadrant == 0 {
        quadrant = 0;
    }
    if x != 0
        && within_circle_bounds(
            if quadrant == 0 { point_a } else { point_b },
            quadrant,
            start,
            end,
            opposite,
        )
    {
        effect(p3(loc.x - x, loc.y + y), map);
    }
    if y != 0 && within_circle_bounds(point_b, 1, start, end, opposite) {
        effect(p3(loc.x + x, loc.y - y), map);
    }
    if x != 0 && y != 0 && within_circle_bounds(point_a, 0, start, end, opposite) {
        effect(p3(loc.x - x, loc.y - y), map);
    }
}

/// `Geometry.WithinCircleBounds` (the unused `center` argument is dropped).
pub fn within_circle_bounds(
    point_loc: Point2D,
    point_quadrant: i32,
    start: CirclePoint,
    end: CirclePoint,
    opposite: bool,
) -> bool {
    if start.angle == 0 && end.angle == 360 {
        return true;
    }
    let (start_x, start_y) = (start.point.x, start.point.y);
    let (end_x, end_y) = (end.point.x, end.point.y);
    let (x, y) = (point_loc.x, point_loc.y);

    if point_quadrant < start.quadrant || point_quadrant > end.quadrant {
        return opposite;
    }
    if point_quadrant > start.quadrant && point_quadrant < end.quadrant {
        return !opposite;
    }

    let mut within = true;
    if start.quadrant == end.quadrant {
        // a chain of three branches that each clear `within`
        if (start_x == end_x && (x > start_x || y > start_y || y < end_y))
            || (start_y == end_y && (y < start_y || x < start_x || x > end_x))
            || x < start_x
            || x > end_x
            || y > start_y
            || y < end_y
        {
            within = false;
        }
    } else if (point_quadrant == start.quadrant && (x < start_x || y > start_y))
        || (point_quadrant == end.quadrant && (x > end_x || y < end_y))
    {
        within = false;
    }
    if opposite { !within } else { within }
}

/// `Geometry.Line2D`: Bresenham from `start` to `end` at `start.Z`.
pub fn line_2d(start: Point3D, end: Point3D, map: MapId, effect: &mut dyn FnMut(Point3D, MapId)) {
    let steep = (end.y - start.y).abs() > (end.x - start.x).abs();
    let (mut x0, mut x1, mut y0, mut y1) = (start.x, end.x, start.y, end.y);
    if steep {
        std::mem::swap(&mut x0, &mut y0);
        std::mem::swap(&mut x1, &mut y1);
    }
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
        std::mem::swap(&mut y0, &mut y1);
    }
    let deltax = x1 - x0;
    let deltay = (y1 - y0).abs();
    let mut error = deltax / 2;
    let ystep = if y0 < y1 { 1 } else { -1 };
    let mut y = y0;
    for x in x0..=x1 {
        if steep {
            effect(
                Point3D {
                    x: y,
                    y: x,
                    z: start.z,
                },
                map,
            );
        } else {
            effect(Point3D { x, y, z: start.z }, map);
        }
        error -= deltay;
        if error < 0 {
            y += ystep;
            error += deltax;
        }
    }
}

// ---- NameList ----

/// One entry of `Data/names.json`.
#[derive(serde::Deserialize, Clone, Debug, Default)]
pub struct NameList {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "names")]
    pub list: Vec<String>,
}

impl NameList {
    /// `ContainsName`: ordinal, case-sensitive.
    pub fn contains_name(&self, name: &str) -> bool {
        self.list.iter().any(|n| n == name)
    }

    /// `GetRandomName`: `List.RandomElement() ?? ""`.
    pub fn get_random_name(&self, rng: &mut muo_primitives::BuiltInRng) -> String {
        if self.list.is_empty() {
            return String::new();
        }
        self.list[muo_primitives::utility::random(rng, self.list.len() as i32) as usize].clone()
    }
}

/// `NameList.m_Table`: case-insensitive by type, insertion ordered.
#[derive(Default)]
pub struct NameLists(indexmap::IndexMap<String, NameList>);

/// `NameList.GetNameList(type)`.
pub fn get_name_list<'a>(w: &'a World, kind: &str) -> Option<&'a NameList> {
    w.resources.get::<NameLists>()?.0.get(&kind.lower_inv())
}

/// `NameList.RandomName(type)`.
pub fn random_name(w: &mut World, kind: &str) -> String {
    let Some(list) = get_name_list(w, kind).cloned() else {
        return String::new();
    };
    list.get_random_name(&mut w.rng)
}

/// Fills the table from the text of `Data/names.json`: names trimmed (`FixNames`), a duplicate type is an error
/// (a repeated key is rejected).
pub fn load_name_lists(w: &mut World, json: &str) -> Result<(), String> {
    // `JsonConfig` options: comments and trailing commas allowed
    let lists: Vec<NameList> = muo_data::json::from_str(json).map_err(|e| e.to_string())?;
    let table = &mut w.resources.get_mut::<NameLists>().0;
    for mut l in lists {
        for n in &mut l.list {
            *n = n.trim().to_owned();
        }
        let key = l.kind.lower_inv();
        if table.contains_key(&key) {
            return Err(format!(
                "An item with the same key has already been added. Key: {}",
                l.kind
            ));
        }
        table.insert(key, l);
    }
    Ok(())
}

/// `NameList.Configure`: reads `Data/names.json` of the base directory.
// `Core.BaseDirectory` is the caller's `base_dir`; `load_name_lists` takes the file text.
pub fn configure_name_lists(w: &mut World, base_dir: &std::path::Path) -> Result<(), String> {
    let path = base_dir.join("Data/names.json");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to deserialize {}: {e}.", path.display()))?;
    load_name_lists(w, &text)
}

#[allow(dead_code)]
fn _unused(_: DateTime) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_and_arc() {
        let mut pts = Vec::new();
        line_2d(
            Point3D { x: 0, y: 0, z: 5 },
            Point3D { x: 3, y: 1, z: 9 },
            MapId(0),
            &mut |p, _| pts.push((p.x, p.y, p.z)),
        );
        assert_eq!(pts, [(0, 0, 5), (1, 0, 5), (2, 1, 5), (3, 1, 5)]);
        let loc = Point3D {
            x: 100,
            y: 100,
            z: 0,
        };
        assert_eq!(arc_point(loc, 10, 0), Point2D { x: 90, y: 100 });
        let mut n = 0;
        circle_2d(loc, MapId(0), 3, &mut |_, _| n += 1);
        assert!(n >= 16);
    }

    #[test]
    fn names_load_and_look_up() {
        let mut w = World::new();
        load_name_lists(&mut w, r#"[{"type":"Male","names":[" Bob ","Al"]}]"#).unwrap();
        let l = get_name_list(&w, "MALE").unwrap();
        assert!(l.contains_name("Bob") && !l.contains_name("bob"));
        assert!(["Bob", "Al"].contains(&random_name(&mut w, "male").as_str()));
        assert_eq!(random_name(&mut w, "nope"), "");
    }

    #[test]
    fn names_json_is_read_like_json_config() {
        let mut w = World::new();
        // comments and trailing commas load; a bad file or a repeated type is an error (startup stops)
        load_name_lists(
            &mut w,
            "[ // c
 {\"type\":\"Male\",\"names\":[\"Al\",],}, ]",
        )
        .unwrap();
        assert!(get_name_list(&w, "male").is_some());
        assert!(load_name_lists(&mut w, "[{\"type\":\"MALE\",\"names\":[]}]").is_err());
        assert!(load_name_lists(&mut World::new(), "[{").is_err());
        let dir = std::env::temp_dir().join(format!("muo_names_{}", std::process::id()));
        assert!(configure_name_lists(&mut World::new(), &dir).is_err());
    }
}
