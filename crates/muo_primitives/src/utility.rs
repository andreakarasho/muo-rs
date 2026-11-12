//! The entity-free math, string and random helpers of `Utility`.
//!
//! What stays out: everything touching `Mobile`/`Map`/`Core`
//! (`AssignRandomHair`, `GetValidLocation*`, `RandomPointIn`'s z, `InUpdateRange`), the reflection and
//! interning helpers, and the null-collection helpers (`Utility.Add(ref list, ..)` is `Option<Vec<T>>`).
//! Random helpers take the generator explicitly instead of the static `BuiltInRng`.

use std::fmt::{self, Write};

use crate::direction::Direction;
use crate::geometry::{Point2D, Point3D, Rectangle2D};
use crate::random::{Rng, WeightedValue};
use crate::timespan::TimeSpan;

/// `Utility.GetDirection`: the facing from one point toward another (a 3:1 slope decides cardinal vs diagonal).
pub fn get_direction(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Direction {
    let dx = to_x.wrapping_sub(from_x);
    let dy = to_y.wrapping_sub(from_y);
    let (adx, ady) = (dx.wrapping_abs(), dy.wrapping_abs());

    if adx >= ady.wrapping_mul(3) {
        return if dx > 0 {
            Direction::EAST
        } else {
            Direction::WEST
        };
    }
    if ady >= adx.wrapping_mul(3) {
        return if dy > 0 {
            Direction::SOUTH
        } else {
            Direction::NORTH
        };
    }
    if dx > 0 {
        return if dy > 0 {
            Direction::DOWN
        } else {
            Direction::RIGHT
        };
    }
    if dy > 0 {
        Direction::LEFT
    } else {
        Direction::UP
    }
}

pub fn get_direction_2d(from: Point2D, to: Point2D) -> Direction {
    get_direction(from.x, from.y, to.x, to.y)
}

pub fn get_direction_3d(from: Point3D, to: Point3D) -> Direction {
    get_direction(from.x, from.y, to.x, to.y)
}

/// `Utility.GetArrayCap`: the element at `index` clamped into the slice, or `None` when it is empty.
pub fn get_array_cap<T>(items: &[T], index: i32) -> Option<&T> {
    let last = items.len().checked_sub(1)?;
    items.get(usize::try_from(index).map_or(0, |i| i.min(last)))
}

/// `Utility.FixPoints`: orders the corners so `top` holds the minimum of each axis.
pub fn fix_points(top: &mut Point3D, bottom: &mut Point3D) {
    if bottom.x < top.x {
        std::mem::swap(&mut top.x, &mut bottom.x);
    }
    if bottom.y < top.y {
        std::mem::swap(&mut top.y, &mut bottom.y);
    }
    if bottom.z < top.z {
        std::mem::swap(&mut top.z, &mut bottom.z);
    }
}

pub fn number_between(num: f64, bound1: i32, bound2: i32, allowance: f64) -> bool {
    let (lo, hi) = if bound1 > bound2 {
        (bound2, bound1)
    } else {
        (bound1, bound2)
    };
    num < f64::from(hi) + allowance && num > f64::from(lo) - allowance
}

/// `Utility.InRange`: a square range (Chebyshev distance) test.
pub fn in_range(p1x: i32, p1y: i32, p2x: i32, p2y: i32, range: i32) -> bool {
    p1x >= p2x.wrapping_sub(range)
        && p1x <= p2x.wrapping_add(range)
        && p1y >= p2y.wrapping_sub(range)
        && p1y <= p2y.wrapping_add(range)
}

pub fn in_range_2d(p1: Point2D, p2: Point2D, range: i32) -> bool {
    in_range(p1.x, p1.y, p2.x, p2.y, range)
}

pub fn in_range_3d(p1: Point3D, p2: Point3D, range: i32) -> bool {
    in_range(p1.x, p1.y, p2.x, p2.y, range)
}

/// `GetDistanceToSqrt` for two points: the straight-line xy distance (the name says sqrt, the value is the root).
pub fn distance_xy(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> f64 {
    let dx = from_x.wrapping_sub(to_x);
    let dy = from_y.wrapping_sub(to_y);
    f64::from(dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy))).sqrt()
}

pub fn distance_to_sqrt_2d(from: Point2D, to: Point2D) -> f64 {
    distance_xy(from.x, from.y, to.x, to.y)
}

pub fn distance_to_sqrt_3d(from: Point3D, to: Point3D) -> f64 {
    distance_xy(from.x, from.y, to.x, to.y)
}

// ---- random -----------------------------------------------------------------------------------

/// `Utility.Random(count)`: [0, count), or (-count, 0] for a negative count.
pub fn random(rng: &mut impl Rng, count: i32) -> i32 {
    if count < 0 {
        -rng.next_below(count.wrapping_neg())
    } else {
        rng.next_below(count)
    }
}

/// `Utility.Random(from, count)`: `from + [0, count)`.
pub fn random_from(rng: &mut impl Rng, from: i32, count: i32) -> i32 {
    rng.next_from(from, count)
}

pub fn random_i64(rng: &mut impl Rng, count: i64) -> i64 {
    if count < 0 {
        -rng.next_i64_below(count.wrapping_neg())
    } else {
        rng.next_i64_below(count)
    }
}

pub fn random_i64_from(rng: &mut impl Rng, from: i64, count: i64) -> i64 {
    rng.next_i64_from(from, count)
}

pub fn random_bool(rng: &mut impl Rng) -> bool {
    rng.next_below(2) == 0
}

pub fn random_double(rng: &mut impl Rng) -> f64 {
    rng.next_f64()
}

pub fn random_bytes(rng: &mut impl Rng, buf: &mut [u8]) {
    rng.fill_bytes(buf);
}

/// `Utility.RandomMinMax(int, int)`: inclusive on both ends, arguments in either order.
pub fn random_min_max(rng: &mut impl Rng, min: i32, max: i32) -> i32 {
    let (min, max) = if min > max { (max, min) } else { (min, max) };
    if min == max {
        return min;
    }
    min.wrapping_add(rng.next_below(max.wrapping_sub(min).wrapping_add(1)))
}

pub fn random_min_max_i64(rng: &mut impl Rng, min: i64, max: i64) -> i64 {
    let (min, max) = if min > max { (max, min) } else { (min, max) };
    if min == max {
        return min;
    }
    min.wrapping_add(rng.next_i64_below(max.wrapping_sub(min).wrapping_add(1)))
}

pub fn random_min_max_f64(rng: &mut impl Rng, min: f64, max: f64) -> f64 {
    let (min, max) = if min > max { (max, min) } else { (min, max) };
    if min == max {
        return min;
    }
    min + rng.next_f64() * (max - min)
}

pub fn random_min_max_time(rng: &mut impl Rng, min: TimeSpan, max: TimeSpan) -> TimeSpan {
    TimeSpan::from_ticks(random_min_max_i64(rng, min.ticks(), max.ticks()))
}

/// `RandomElement`: `None` for an empty slice.
pub fn random_element<'a, T>(rng: &mut impl Rng, items: &'a [T]) -> Option<&'a T> {
    if items.is_empty() {
        None
    } else {
        items.get(random(rng, items.len() as i32) as usize)
    }
}

/// `TakeRandomElement`: removes and returns a random element.
pub fn take_random_element<T>(rng: &mut impl Rng, items: &mut Vec<T>) -> Option<T> {
    if items.is_empty() {
        return None;
    }
    let index = random(rng, items.len() as i32) as usize;
    Some(items.remove(index))
}

/// `RandomWeightedElement(values, totalWeight)`. Draws `[0, total)` and takes the first entry whose
/// running weight reaches it, so a draw of 0 selects the first entry even at weight 0.
/// `None` when there is nothing to pick.
pub fn random_weighted_element_total<'a, T>(
    rng: &mut impl Rng,
    values: &'a [WeightedValue<T>],
    total_weight: i32,
) -> Option<&'a WeightedValue<T>> {
    let mut roll = random(rng, total_weight);
    for v in values {
        roll -= v.weight;
        if roll <= 0 {
            return Some(v);
        }
    }
    None
}

pub fn random_weighted_element<'a, T>(
    rng: &mut impl Rng,
    values: &'a [WeightedValue<T>],
) -> Option<&'a WeightedValue<T>> {
    let total = values.iter().fold(0i32, |a, v| a.wrapping_add(v.weight));
    random_weighted_element_total(rng, values, total)
}

/// `RandomSample`: `count` distinct random elements (at most all of them) without touching `source`.
// `count` is capped at the length: asking for more than there are returns them all.
pub fn random_sample<T: Clone>(rng: &mut impl Rng, source: &[T], count: usize) -> Vec<T> {
    let count = count.min(source.len());
    let mut taken = vec![false; source.len()];
    let mut out = Vec::with_capacity(count);
    while out.len() < count {
        let i = random(rng, source.len() as i32) as usize;
        if !taken[i] {
            taken[i] = true;
            out.push(source[i].clone());
        }
    }
    out
}

/// `Utility.CoinFlips(amount, maximum)`: heads out of `amount` fair flips, stopping at `maximum`.
pub fn coin_flips_max(rng: &mut impl Rng, mut amount: i32, maximum: i32) -> i32 {
    let mut heads = 0;
    while amount > 0 {
        // a draw below 2^amount, at most 62 bits
        let num = if amount >= 62 {
            rng.next_i64() as u64
        } else {
            rng.next_i64_below(1i64 << amount) as u64
        };
        heads += num.count_ones() as i32;
        if heads >= maximum {
            return maximum;
        }
        amount -= 62;
    }
    heads
}

pub fn coin_flips(rng: &mut impl Rng, mut amount: i32) -> i32 {
    let mut heads = 0;
    while amount > 0 {
        let num = if amount >= 62 {
            rng.next_i64() as u64
        } else {
            rng.next_i64_below(1i64 << amount) as u64
        };
        heads += num.count_ones() as i32;
        amount -= 62;
    }
    heads
}

/// `Utility.Dice(amount, sides, bonus)`
pub fn dice(rng: &mut impl Rng, amount: i32, sides: i32, bonus: i32) -> i32 {
    if amount <= 0 || sides <= 0 {
        return 0;
    }
    let total = if sides == 2 {
        coin_flips(rng, amount)
    } else {
        (0..amount).fold(0i32, |t, _| t.wrapping_add(rng.next_from(1, sides)))
    };
    total.wrapping_add(bonus)
}

/// `Utility.RandomPointIn(rect, map)` without the map: the x and y; the caller asks the map for z.
pub fn random_point_xy_in(rng: &mut impl Rng, rect: Rectangle2D) -> (i32, i32) {
    (
        rng.next_from(rect.x(), rect.width()),
        rng.next_from(rect.y(), rect.height()),
    )
}

macro_rules! hue_ranges {
    ($($(#[$doc:meta])* $name:ident($start:expr, $count:expr);)*) => {$(
        $(#[$doc])*
        pub fn $name(rng: &mut impl Rng) -> i32 {
            random_from(rng, $start, $count)
        }
    )*};
}

hue_ranges! {
    /// Random hue in the range 1201-1254
    random_pink_hue(1201, 54);
    /// Random hue in the range 1301-1354
    random_blue_hue(1301, 54);
    /// Random hue in the range 1401-1454
    random_green_hue(1401, 54);
    /// Random hue in the range 1501-1554
    random_orange_hue(1501, 54);
    /// Random hue in the range 1601-1654
    random_red_hue(1601, 54);
    /// Random hue in the range 1701-1754
    random_yellow_hue(1701, 54);
    /// Random hue in the range 1801-1908
    random_neutral_hue(1801, 108);
    /// Random hue in the range 2001-2018
    random_snake_hue(2001, 18);
    /// Random hue in the range 2101-2130
    random_bird_hue(2101, 30);
    /// Random hue in the range 2201-2224
    random_slime_hue(2201, 24);
    /// Random hue in the range 2301-2318
    random_animal_hue(2301, 18);
    /// Random hue in the range 2401-2430
    random_metal_hue(2401, 30);
    /// Random hue in the range 2-1001
    random_dyed_hue(2, 1000);
}

/// Random pink, blue, green, orange, red or yellow hue.
pub fn random_nondyed_hue(rng: &mut impl Rng) -> i32 {
    match random(rng, 6) {
        0 => random_pink_hue(rng),
        1 => random_blue_hue(rng),
        2 => random_green_hue(rng),
        3 => random_orange_hue(rng),
        4 => random_red_hue(rng),
        5 => random_yellow_hue(rng),
        _ => 0,
    }
}

pub const fn clip_dyed_hue(hue: i32) -> i32 {
    if hue < 2 {
        2
    } else if hue > 1001 {
        1001
    } else {
        hue
    }
}

/// Random hue from 0x62, 0x71 (10%) or 0x03, 0x0D, 0x13, 0x1C, 0x21, 0x30, 0x37, 0x3A, 0x44, 0x59.
pub fn random_bright_hue(rng: &mut impl Rng) -> i32 {
    const RARE: [i32; 2] = [0x62, 0x71];
    const COMMON: [i32; 10] = [0x03, 0x0D, 0x13, 0x1C, 0x21, 0x30, 0x37, 0x3A, 0x44, 0x59];
    let list: &[i32] = if random_double(rng) < 0.1 {
        &RARE
    } else {
        &COMMON
    };
    list[random(rng, list.len() as i32) as usize]
}

// ---- numbers and colors -----------------------------------------------------------------------

/// `Utility.NumberOfSetBits`
pub const fn number_of_set_bits(i: u64) -> i32 {
    i.count_ones() as i32
}

/// `Utility.Abs`: `int.MinValue` stays negative (the bit trick).
pub const fn abs(value: i32) -> i32 {
    value.wrapping_abs()
}

/// 15-bit color to 24-bit RGB.
pub const fn c16_to_32(c16: i32) -> i32 {
    let c16 = c16 & 0x7FFF;
    let r = ((c16 >> 10) & 0x1F) << 3;
    let g = ((c16 >> 5) & 0x1F) << 3;
    let b = (c16 & 0x1F) << 3;
    (r << 16) | (g << 8) | b
}

pub const fn c16_to_16(c16: i32) -> i32 {
    c16 & 0x7FFF
}

/// 24-bit RGB to 15-bit color.
pub const fn c32_to_16(c32: i32) -> i32 {
    let c32 = c32 & 0xFF_FFFF;
    let r = ((c32 >> 16) & 0xFF) >> 3;
    let g = ((c32 >> 8) & 0xFF) >> 3;
    let b = (c32 & 0xFF) >> 3;
    (r << 10) | (g << 5) | b
}

// ---- text -------------------------------------------------------------------------------------

/// `FixHtml`: the gump HTML reader chokes on `<`, `>` and `#` in user text.
pub fn fix_html(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '<' => '(',
            '>' => ')',
            '#' => '-',
            c => c,
        })
        .collect()
}

/// `GetXMLInt32`: `XmlConvert.ToInt32`, then `int.TryParse`, then the default (neither reads hex).
pub fn get_xml_i32(s: &str, default: i32) -> i32 {
    if is_hex(s) {
        default
    } else {
        crate::parse::try_to_i32(s).unwrap_or(default)
    }
}

/// `GetXMLUInt32`
pub fn get_xml_u32(s: &str, default: u32) -> u32 {
    if is_hex(s) {
        default
    } else {
        crate::parse::try_to_u32(s).unwrap_or(default)
    }
}

fn is_hex(s: &str) -> bool {
    s.trim()
        .get(..2)
        .is_some_and(|h| h.eq_ignore_ascii_case("0x"))
}

/// `GetXMLTimeSpan`: `XmlConvert.ToTimeSpan` is the ISO 8601 duration (`PT1H30M`, `P1DT2S`).
pub fn get_xml_timespan(s: &str, default: TimeSpan) -> TimeSpan {
    parse_iso_duration(s.trim()).unwrap_or(default)
}

fn parse_iso_duration(s: &str) -> Option<TimeSpan> {
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let s = s.strip_prefix('P')?;
    let (date, time) = match s.split_once('T') {
        Some((d, t)) => (d, Some(t)),
        None => (s, None),
    };
    if date.is_empty() && time.is_none_or(str::is_empty) {
        return None;
    }
    let mut total = 0.0f64;
    let mut take = |part: &str, units: &[(char, f64)]| -> Option<()> {
        let mut num = String::new();
        let mut last = usize::MAX;
        for c in part.chars() {
            if c.is_ascii_digit() || c == '.' {
                num.push(c);
            } else {
                let at = units.iter().position(|(u, _)| *u == c)?;
                if num.is_empty() || (last != usize::MAX && at <= last) {
                    return None;
                }
                total += num.parse::<f64>().ok()? * units[at].1;
                num.clear();
                last = at;
            }
        }
        num.is_empty().then_some(())
    };
    take(date, &[('D', 86_400.0)])?;
    if let Some(t) = time {
        take(t, &[('H', 3600.0), ('M', 60.0), ('S', 1.0)])?;
    }
    let span = TimeSpan::from_secs_f64(total);
    Some(if neg { -span } else { span })
}

/// `FormatBuffer`: a hex dump, sixteen bytes a line.
pub fn format_buffer(out: &mut impl Write, data: &[u8]) -> fmt::Result {
    out.write_str("        0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F\n")?;
    out.write_str("       -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --\n")?;
    if data.is_empty() {
        return out.write_str("0000   \n");
    }
    for (i, chunk) in data.chunks(16).enumerate() {
        write!(out, "{:04X}   ", i * 16)?;
        for (j, b) in chunk.iter().enumerate() {
            if j > 0 {
                out.write_char(' ')?;
            }
            write!(out, "{b:02X}")?;
        }
        out.write_char('\n')?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::{BuiltInRng, FixedRng};

    #[test]
    fn directions() {
        assert_eq!(get_direction(0, 0, 10, 0), Direction::EAST);
        assert_eq!(get_direction(0, 0, -10, 1), Direction::WEST);
        assert_eq!(get_direction(0, 0, 0, 5), Direction::SOUTH);
        assert_eq!(get_direction(0, 0, 1, -9), Direction::NORTH);
        assert_eq!(get_direction(0, 0, 3, 3), Direction::DOWN);
        assert_eq!(get_direction(0, 0, 3, -3), Direction::RIGHT);
        assert_eq!(get_direction(0, 0, -3, 3), Direction::LEFT);
        assert_eq!(get_direction(0, 0, -3, -3), Direction::UP);
        assert_eq!(get_direction(5, 5, 5, 5), Direction::WEST); // 0 >= 0: falls into the east/west arm
    }

    #[test]
    fn ranges_and_points() {
        assert!(in_range(10, 10, 12, 8, 2) && !in_range(10, 10, 13, 10, 2));
        let (mut a, mut b) = (Point3D::new(5, 1, 9), Point3D::new(2, 7, 3));
        fix_points(&mut a, &mut b);
        assert_eq!((a, b), (Point3D::new(2, 1, 3), Point3D::new(5, 7, 9)));
        assert_eq!(distance_xy(0, 0, 3, 4), 5.0);
        assert!(number_between(5.5, 6, 2, 0.6) && !number_between(7.0, 2, 6, 0.5));
        assert_eq!(get_array_cap(&[1, 2, 3], 7), Some(&3));
        assert_eq!(get_array_cap(&[1, 2, 3], -1), Some(&1));
        assert_eq!(get_array_cap::<i32>(&[], 0), None);
    }

    #[test]
    fn random_helpers_stay_in_range() {
        let mut r = BuiltInRng::from_seed(1);
        for _ in 0..500 {
            assert!((0..10).contains(&random(&mut r, 10)));
            assert!((-9..=0).contains(&random(&mut r, -10)));
            assert!((5..=8).contains(&random_min_max(&mut r, 8, 5)));
            assert!((1201..1255).contains(&random_pink_hue(&mut r)));
            assert!((2..1002).contains(&random_dyed_hue(&mut r)));
            let d = dice(&mut r, 3, 6, 2);
            assert!((5..=20).contains(&d));
            let c = coin_flips(&mut r, 100);
            assert!((0..=100).contains(&c));
            assert!(coin_flips_max(&mut r, 100, 10) <= 10);
        }
        assert_eq!(random_min_max(&mut r, 4, 4), 4);
        assert_eq!(dice(&mut r, 0, 6, 5), 0);
        let s = random_sample(&mut r, &[1, 2, 3, 4, 5], 3);
        let mut u = s.clone();
        u.sort_unstable();
        u.dedup();
        assert_eq!(u.len(), 3);
        assert_eq!(random_sample(&mut r, &[1, 2], 9).len(), 2);
        let mut v = vec![1, 2, 3];
        assert!(take_random_element(&mut r, &mut v).is_some() && v.len() == 2);
        assert_eq!(random_element::<i32>(&mut r, &[]), None);
    }

    #[test]
    fn weighted_and_fixed() {
        let vals = [
            WeightedValue::new(1, 'a'),
            WeightedValue::new(3, 'b'),
            WeightedValue::new(6, 'c'),
        ];
        // PredictableRandom(0) -> roll 0 -> first entry; high roll -> last
        assert_eq!(
            random_weighted_element(&mut FixedRng(0), &vals).map(|v| v.value),
            Some('a')
        );
        assert_eq!(
            random_weighted_element(&mut FixedRng(2), &vals).map(|v| v.value),
            Some('b')
        );
        assert_eq!(
            random_weighted_element(&mut FixedRng(9), &vals).map(|v| v.value),
            Some('c')
        );
        assert_eq!(random_weighted_element::<char>(&mut FixedRng(0), &[]), None);
        // dice with two sides goes through coin flips: 1 flip, fixed value 1 -> popcount 1
        assert_eq!(dice(&mut FixedRng(1), 1, 2, 0), 1);
    }

    #[test]
    fn colors() {
        assert_eq!(c16_to_32(0x7FFF), 0xF8F8F8);
        assert_eq!(c32_to_16(0xFFFFFF), 0x7FFF);
        assert_eq!(c32_to_16(c16_to_32(0x1234)), 0x1234);
        assert_eq!(c16_to_16(0xFFFF), 0x7FFF);
        assert_eq!(abs(i32::MIN), i32::MIN);
        assert_eq!(number_of_set_bits(u64::MAX), 64);
        assert_eq!(clip_dyed_hue(0), 2);
        assert_eq!(clip_dyed_hue(5000), 1001);
    }

    #[test]
    fn text_helpers() {
        assert_eq!(fix_html("<b>#1</b>"), "(b)-1(/b)");
        assert_eq!(get_xml_i32(" 12 ", 0), 12);
        assert_eq!(get_xml_i32("0x10", 7), 7); // XmlConvert has no hex; neither does int.TryParse
        assert_eq!(get_xml_u32("x", 3), 3);
        assert_eq!(
            get_xml_timespan("PT1H30M", TimeSpan::ZERO),
            TimeSpan::from_mins(90)
        );
        assert_eq!(
            get_xml_timespan("P1DT2S", TimeSpan::ZERO),
            TimeSpan::from_secs(86_402)
        );
        assert_eq!(
            get_xml_timespan("PT0.5S", TimeSpan::ZERO),
            TimeSpan::from_millis(500)
        );
        assert_eq!(
            get_xml_timespan("-PT1M", TimeSpan::ZERO),
            TimeSpan::from_mins(-1)
        );
        assert_eq!(
            get_xml_timespan("1:00:00", TimeSpan::from_secs(9)),
            TimeSpan::from_secs(9)
        );
        assert_eq!(
            get_xml_timespan("P", TimeSpan::from_secs(9)),
            TimeSpan::from_secs(9)
        );
        assert_eq!(
            get_xml_timespan("PT1M1H", TimeSpan::from_secs(9)),
            TimeSpan::from_secs(9)
        );
    }

    #[test]
    fn hex_dump() {
        let mut s = String::new();
        format_buffer(&mut s, &(0u8..17).collect::<Vec<_>>()).unwrap();
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(
            lines[2],
            "0000   00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F"
        );
        assert_eq!(lines[3], "0010   10");
        let mut e = String::new();
        format_buffer(&mut e, &[]).unwrap();
        assert!(e.ends_with("0000   \n"));
    }
}
