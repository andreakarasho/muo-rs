//! Weather: the `Weather` class (static and moving storm areas per facet, a 30 second tick that
//! sends the weather packet to the clients standing inside). `WeatherMap` (a `MapItem`) belongs with the map items;
//! it needs only [`pin_positions`].

use std::f64::consts::PI;

use muo_primitives::{Rectangle2D, TimeSpan};
use muo_server::World;
use muo_server::map::MapId;
use muo_server::timer;

/// One `Weather` instance.
#[derive(Clone, Debug)]
pub struct Weather {
    pub facet: MapId,
    pub area: Vec<Rectangle2D>,
    pub temperature: i32,
    pub chance_of_precipitation: i32,
    pub chance_of_extreme_temperature: i32,
    /// Dynamic weather: the area it roams.
    pub bounds: Rectangle2D,
    pub move_speed: i32,
    pub move_angle_x: i32,
    pub move_angle_y: i32,
    active: bool,
    extreme_temperature: bool,
    stage: i32,
}

/// `Weather.m_Facets` and `m_WeatherByFacet` (a list per facet, in creation order).
#[derive(Default)]
pub struct WeatherState {
    facets: Vec<MapId>,
    by_facet: Vec<(MapId, Vec<Weather>)>,
}

fn state(w: &mut World) -> &mut WeatherState {
    w.resources.get_mut::<WeatherState>()
}

/// `Weather.GetWeatherList(facet)`.
pub fn weather_list(w: &World, facet: MapId) -> &[Weather] {
    w.resources
        .get::<WeatherState>()
        .and_then(|s| s.by_facet.iter().find(|(f, _)| *f == facet))
        .map_or(&[], |(_, l)| l.as_slice())
}

fn list_mut(w: &mut World, facet: MapId) -> &mut Vec<Weather> {
    let s = state(w);
    if let Some(i) = s.by_facet.iter().position(|(f, _)| *f == facet) {
        return &mut s.by_facet[i].1;
    }
    s.by_facet.push((facet, Vec::new()));
    &mut s.by_facet.last_mut().expect("just pushed").1
}

/// `new Weather(facet, area, ...)`: adds it to its facet's list and starts its tick. Returns its index.
#[allow(clippy::too_many_arguments)] // new Weather(facet, area, temperature, chanceOfPercipitation, chanceOfExtremeTemperature, moveSpeed, ...)
fn new_weather(
    w: &mut World,
    facet: MapId,
    area: Vec<Rectangle2D>,
    temperature: i32,
    chance_of_precipitation: i32,
    chance_of_extreme_temperature: i32,
    interval: TimeSpan,
    dynamic: Option<(Rectangle2D, i32)>,
) {
    let (bounds, move_speed) = dynamic.unwrap_or((Rectangle2D::EMPTY, 0));
    let list = list_mut(w, facet);
    list.push(Weather {
        facet,
        area,
        temperature,
        chance_of_precipitation,
        chance_of_extreme_temperature,
        bounds,
        move_speed,
        move_angle_x: 0,
        move_angle_y: 0,
        active: false,
        extreme_temperature: false,
        stage: 0,
    });
    let index = list.len() - 1;
    let delay =
        (0.2 + muo_primitives::utility::random_double(&mut w.rng) * 0.8) * interval.total_seconds();
    timer::start(w, TimeSpan::from_secs_f64(delay), interval, 0, move |w| {
        on_tick(w, facet, index)
    });
}

/// `Weather.CheckIntersection`.
pub fn check_intersection(r1: Rectangle2D, r2: Rectangle2D) -> bool {
    r1.x() < r2.x() + r2.width()
        && r2.x() < r1.x() + r1.width()
        && r1.y() < r2.y() + r2.height()
        && r2.y() < r1.y() + r1.height()
}

/// `Weather.CheckContains`.
pub fn check_contains(big: Rectangle2D, small: Rectangle2D) -> bool {
    small.x() >= big.x()
        && small.y() >= big.y()
        && small.x() + small.width() <= big.x() + big.width()
        && small.y() + small.height() <= big.y() + big.height()
}

/// `Weather.IntersectsWith(area)`.
fn intersects_with(weather: &Weather, area: Rectangle2D) -> bool {
    weather.area.iter().any(|a| check_intersection(area, *a))
}

/// `Weather.CheckWeatherConflict(facet, exclude, area)`; `exclude` is an index into the facet's list.
pub fn check_weather_conflict(
    w: &World,
    facet: MapId,
    exclude: Option<usize>,
    area: Rectangle2D,
) -> bool {
    weather_list(w, facet)
        .iter()
        .enumerate()
        .any(|(i, x)| Some(i) != exclude && intersects_with(x, area))
}

/// `Weather.AddDynamicWeather`.
#[allow(clippy::too_many_arguments)] // Weather.AddDynamicWeather(temperature, chanceOfPercipitation, chanceOfExtremeTemperature, moveSpeed, width, height, bounds, facet)
pub fn add_dynamic_weather(
    w: &mut World,
    temperature: i32,
    chance_of_precipitation: i32,
    chance_of_extreme_temperature: i32,
    move_speed: i32,
    width: i32,
    height: i32,
    bounds: Rectangle2D,
) {
    let facets = state(w).facets.clone();
    for facet in facets {
        let mut area = Rectangle2D::EMPTY;
        let mut is_valid = false;
        for _ in 0..10 {
            let x =
                bounds.x() + muo_primitives::utility::random(&mut w.rng, bounds.width() - width);
            let y =
                bounds.y() + muo_primitives::utility::random(&mut w.rng, bounds.height() - height);
            area = Rectangle2D::new(x, y, width, height);
            if !check_weather_conflict(w, facet, None, area) {
                is_valid = true;
            }
            if is_valid {
                break;
            }
        }
        if !is_valid {
            continue;
        }
        new_weather(
            w,
            facet,
            vec![area],
            temperature,
            chance_of_precipitation,
            chance_of_extreme_temperature,
            TimeSpan::from_secs(30),
            Some((bounds, move_speed)),
        );
    }
}

/// `Weather.AddWeather`.
pub fn add_weather(
    w: &mut World,
    temperature: i32,
    chance_of_precipitation: i32,
    chance_of_extreme_temperature: i32,
    area: &[Rectangle2D],
) {
    let facets = state(w).facets.clone();
    for facet in facets {
        new_weather(
            w,
            facet,
            area.to_vec(),
            temperature,
            chance_of_precipitation,
            chance_of_extreme_temperature,
            TimeSpan::from_secs(30),
            None,
        );
    }
}

/// `Weather.Initialize`.
pub fn initialize(w: &mut World) {
    state(w).facets = vec![MapId::FELUCCA, MapId::TRAMMEL];

    // ice island
    add_weather(
        w,
        -15,
        100,
        5,
        &[
            Rectangle2D::new(3850, 160, 390, 320),
            Rectangle2D::new(3900, 480, 380, 180),
            Rectangle2D::new(4160, 660, 150, 110),
        ],
    );
    // covetous entrance, around vesper and minoc
    add_weather(w, 15, 50, 5, &[Rectangle2D::new(2425, 725, 250, 250)]);
    // despise entrance, north of britain
    add_weather(w, 15, 50, 5, &[Rectangle2D::new(1245, 1045, 250, 250)]);

    for _ in 0..15 {
        add_dynamic_weather(
            w,
            15,
            100,
            5,
            8,
            400,
            400,
            Rectangle2D::new(0, 0, 5120, 4096),
        );
    }
}

/// `Weather.Reposition`.
fn reposition(w: &mut World, facet: MapId, index: usize) {
    let Some(me) = weather_list(w, facet).get(index) else {
        return;
    };
    let Some(first) = me.area.first() else { return };
    let (width, height, bounds) = (first.width(), first.height(), me.bounds);
    let mut area = Rectangle2D::EMPTY;
    let mut is_valid = false;
    for _ in 0..10 {
        let x = bounds.x() + muo_primitives::utility::random(&mut w.rng, bounds.width() - width);
        let y = bounds.y() + muo_primitives::utility::random(&mut w.rng, bounds.height() - height);
        area = Rectangle2D::new(x, y, width, height);
        if !check_weather_conflict(w, facet, Some(index), area) {
            is_valid = true;
        }
        if is_valid {
            break;
        }
    }
    if !is_valid {
        return;
    }
    list_mut(w, facet)[index].area[0] = area;
}

/// `Weather.RecalculateMovementAngle`.
fn recalculate_movement_angle(w: &mut World, facet: MapId, index: usize) {
    let angle = muo_primitives::utility::random_double(&mut w.rng) * PI * 2.0;
    let me = &mut list_mut(w, facet)[index];
    me.move_angle_x = (100.0 * angle.cos()) as i32;
    me.move_angle_y = (100.0 * angle.sin()) as i32;
}

/// `Weather.MoveForward`.
fn move_forward(w: &mut World, facet: MapId, index: usize) {
    if weather_list(w, facet)[index].area.is_empty() {
        return;
    }
    for _ in 0..5 {
        // try 5 times to find a valid spot
        let me = &weather_list(w, facet)[index];
        let x_offset = me.move_speed * me.move_angle_x / 100;
        let y_offset = me.move_speed * me.move_angle_y / 100;
        let old = me.area[0];
        let new_area = Rectangle2D::new(
            old.x() + x_offset,
            old.y() + y_offset,
            old.width(),
            old.height(),
        );
        if !check_weather_conflict(w, facet, Some(index), new_area)
            && check_contains(me.bounds, new_area)
        {
            list_mut(w, facet)[index].area[0] = new_area;
            break;
        }
        recalculate_movement_angle(w, facet, index);
    }
}

/// `Weather.OnTick`.
fn on_tick(w: &mut World, facet: MapId, index: usize) {
    let Some(me) = weather_list(w, facet).get(index) else {
        return;
    };
    if me.stage == 0 {
        let (p, e) = (me.chance_of_precipitation, me.chance_of_extreme_temperature);
        let active = p > muo_primitives::utility::random(&mut w.rng, 100);
        let extreme = e > muo_primitives::utility::random(&mut w.rng, 100);
        let me = &mut list_mut(w, facet)[index];
        me.active = active;
        me.extreme_temperature = extreme;
        if me.move_speed > 0 {
            reposition(w, facet, index);
            recalculate_movement_angle(w, facet, index);
        }
    }

    let me = &weather_list(w, facet)[index];
    if me.active {
        let (stage, move_speed) = (me.stage, me.move_speed);
        if stage > 0 && move_speed > 0 {
            move_forward(w, facet, index);
        }
        let me = &weather_list(w, facet)[index];
        let mut temperature = me.temperature;
        if me.extreme_temperature {
            temperature *= -1;
        }
        let density = if me.stage < 15 {
            me.stage * 5
        } else {
            (150 - me.stage * 5).clamp(10, 70)
        };
        let kind = if density == 0 {
            0xFE
        } else if temperature > 0 {
            0
        } else {
            2
        };
        let area = me.area.clone();
        let targets: Vec<_> = w
            .net_states
            .iter()
            .filter_map(|(id, d)| d.mobile.map(|m| (*id, m)))
            .collect();
        for (ns, mob) in targets {
            let Some(m) = w.mobile(mob) else { continue };
            if m.map != Some(facet) {
                continue;
            }
            let contains = area.is_empty() || area.iter().any(|r| r.contains_3d(m.location));
            if !contains {
                continue;
            }
            muo_server::packets::player::send_weather(
                w,
                ns,
                kind as u8,
                density as u8,
                temperature as u8,
            );
        }
    }
    let me = &mut list_mut(w, facet)[index];
    me.stage += 1;
    me.stage %= 30;
}

/// What `WeatherMap.OnDoubleClick` pins: the centre of every weather area of the facet.
pub fn pin_positions(w: &World, facet: MapId) -> Vec<(i32, i32)> {
    weather_list(w, facet)
        .iter()
        .flat_map(|x| {
            x.area
                .iter()
                .map(|a| (a.x() + a.width() / 2, a.y() + a.height() / 2))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangles_intersect_and_contain() {
        let a = Rectangle2D::new(0, 0, 10, 10);
        assert!(check_intersection(a, Rectangle2D::new(9, 9, 5, 5)));
        assert!(
            !check_intersection(a, Rectangle2D::new(10, 0, 5, 5)),
            "touching edges do not intersect"
        );
        assert!(check_contains(a, Rectangle2D::new(2, 2, 8, 8)));
        assert!(!check_contains(a, Rectangle2D::new(2, 2, 9, 8)));
    }

    #[test]
    fn static_weather_is_added_per_facet() {
        let mut w = World::new();
        initialize(&mut w);
        let felucca = weather_list(&w, MapId::FELUCCA);
        // three static areas, then the dynamic ones that found room
        assert!(felucca.len() >= 3);
        assert_eq!(felucca[0].temperature, -15);
        assert_eq!(felucca[0].area.len(), 3);
        assert_eq!(
            weather_list(&w, MapId::TRAMMEL)[1].area[0],
            Rectangle2D::new(2425, 725, 250, 250)
        );
        assert_eq!(pin_positions(&w, MapId::ILSHENAR).len(), 0);
    }
}
