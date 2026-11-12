//! Runtime library behavior that Rust's std does differently where the difference is observable: case mapping
//! (`OrdinalIgnoreCase`, `ToUpperInvariant`), `IPAddress.Parse`, `List<T>.Sort`/`Array.Sort` (introsort, unstable
//! but deterministic), `double.ToString()`, narrowing `(short)double` casts. Every expected value in the tests
//! is a recorded reference value.

use std::cmp::Ordering;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// ---- case mapping --------------------------------------------------------------------------------

/// Code points the std tables map (a newer Unicode, or a mapping the invariant tables leave out) that must map to themselves.
fn unknown_to_net(c: u32) -> bool {
    matches!(c, 0x131 | 0x19B | 0x264 | 0x1C89 | 0x1C8A | 0xA7CB..=0xA7CF | 0xA7D2..=0xA7D5 | 0xA7DA..=0xA7DC
        | 0x10D50..=0x10D85 | 0x16EA0..=0x16ED3)
}

/// `char.ToUpperInvariant` over a scalar: the simple (one char) mapping, which is what `OrdinalIgnoreCase` compares.
pub fn simple_upper(c: char) -> char {
    let u = c as u32;
    if unknown_to_net(u) {
        return c;
    }
    // titlecase letters whose full uppercase is two chars but whose simple uppercase is one
    let t = match u {
        0x1F80..=0x1F87 | 0x1F90..=0x1F97 | 0x1FA0..=0x1FA7 => u + 8,
        0x1FB3 => 0x1FBC,
        0x1FC3 => 0x1FCC,
        0x1FF3 => 0x1FFC,
        _ => 0,
    };
    if t != 0 {
        return char::from_u32(t).unwrap_or(c);
    }
    let mut it = c.to_uppercase();
    match (it.next(), it.next()) {
        (Some(x), None) => x,
        _ => c,
    }
}

/// `char.ToLowerInvariant` over a scalar (simple mapping).
pub fn simple_lower(c: char) -> char {
    if unknown_to_net(c as u32) {
        return c;
    }
    let mut it = c.to_lowercase();
    match (it.next(), it.next()) {
        (Some(x), None) => x,
        _ => c,
    }
}

/// `string.ToUpperInvariant()`. Also the key of a `Dictionary<string, _>(StringComparer.OrdinalIgnoreCase)`.
pub fn to_upper_invariant(s: &str) -> String {
    s.chars().map(simple_upper).collect()
}

/// `string.ToLowerInvariant()`.
pub fn to_lower_invariant(s: &str) -> String {
    s.chars().map(simple_lower).collect()
}

/// `string.Equals(a, b, StringComparison.OrdinalIgnoreCase)`.
pub fn eq_ignore_case(a: &str, b: &str) -> bool {
    a == b || a.chars().map(simple_upper).eq(b.chars().map(simple_upper))
}

/// `a.StartsWith(prefix, StringComparison.OrdinalIgnoreCase)`.
pub fn starts_with_ignore_case(a: &str, prefix: &str) -> bool {
    let mut s = a.chars().map(simple_upper);
    prefix
        .chars()
        .map(simple_upper)
        .all(|x| s.next() == Some(x))
}

/// `a.EndsWith(suffix, StringComparison.OrdinalIgnoreCase)`.
pub fn ends_with_ignore_case(a: &str, suffix: &str) -> bool {
    to_upper_invariant(a).ends_with(&to_upper_invariant(suffix))
}

/// `a.Contains(needle, StringComparison.OrdinalIgnoreCase)`.
pub fn contains_ignore_case(a: &str, needle: &str) -> bool {
    to_upper_invariant(a).contains(&to_upper_invariant(needle))
}

/// `a.IndexOf(needle, StringComparison.OrdinalIgnoreCase)` as a char index into `a`.
pub fn index_of_ignore_case(a: &str, needle: &str) -> Option<usize> {
    let n: Vec<char> = needle.chars().map(simple_upper).collect();
    if n.is_empty() {
        return Some(0);
    }
    let h: Vec<char> = a.chars().map(simple_upper).collect();
    h.windows(n.len()).position(|w| w == n.as_slice())
}

/// `string.Compare(a, b, StringComparison.OrdinalIgnoreCase)`: UTF-16 units of the uppercased strings.
pub fn cmp_ignore_case(a: &str, b: &str) -> Ordering {
    to_upper_invariant(a)
        .encode_utf16()
        .cmp(to_upper_invariant(b).encode_utf16())
}

// ---- culture-sensitive string comparison ---------------------------------------------------------

#[path = "string_compare_data.rs"]
mod collation_data;

use std::collections::HashMap;
use std::sync::OnceLock;

struct Collation {
    decomp: HashMap<u32, &'static [u16]>,
    primary: HashMap<u32, &'static [u8]>,
    expand: HashMap<u32, &'static [char]>,
    marked: HashMap<u32, (u32, usize)>,
    mark_rank: HashMap<u16, u32>,
}

fn collation() -> &'static Collation {
    static C: OnceLock<Collation> = OnceLock::new();
    C.get_or_init(|| Collation {
        decomp: collation_data::DECOMP.iter().copied().collect(),
        primary: collation_data::PRIMARY.iter().copied().collect(),
        expand: collation_data::EXPAND.iter().copied().collect(),
        marked: collation_data::MARKED
            .iter()
            .map(|&(c, b, n)| (c, (b, n)))
            .collect(),
        mark_rank: collation_data::MARKS
            .iter()
            .enumerate()
            .map(|(i, &m)| (m, 10 * (i as u32 + 1)))
            .collect(),
    })
}

/// A string's collation weights: primary bytes, secondary and tertiary elements (the three levels of the ICU
/// root collation that `CompareInfo` uses under the invariant culture, with punctuation not ignorable).
#[derive(Default)]
struct SortKey {
    p: Vec<u8>,
    s: Vec<u32>,
    t: Vec<u8>,
}

impl SortKey {
    fn letter(&mut self, c: &Collation, cp: u32, s: u32, t: u8) {
        // a code point with no primary weight is ignorable
        if let Some(p) = c.primary.get(&cp) {
            self.p.extend_from_slice(p);
            self.s.push(s);
            self.t.push(t);
        }
    }

    fn add(&mut self, c: &Collation, ch: char) {
        let cp = ch as u32;
        let upper = u8::from(ch.is_uppercase());
        if let Some(parts) = c.decomp.get(&cp) {
            let mut it = parts.iter();
            if let Some(&b) = it.next() {
                self.letter(c, u32::from(b), 0, upper);
            }
            for m in it {
                self.s.push(c.mark_rank.get(m).copied().unwrap_or(0));
                self.t.push(0);
            }
        } else if let Some(&w) = u16::try_from(cp).ok().and_then(|m| c.mark_rank.get(&m)) {
            self.s.push(w);
            self.t.push(0);
        } else if let Some(letters) = c.expand.get(&cp) {
            for (i, &l) in letters.iter().enumerate() {
                self.letter(c, u32::from(l), u32::from(i == 0), upper);
            }
        } else if let Some(&(base, n)) = c.marked.get(&cp) {
            self.letter(c, base, 10 * n as u32 + 5, upper);
        } else if let Some(p) = c.primary.get(&cp) {
            self.p.extend_from_slice(p);
            self.s.push(0);
            // U+00A0 is a space with a compatibility tertiary weight
            self.t.push(if cp == 0xA0 { 2 } else { upper });
        } else if (cp < 0x250
            || (0x370..0x500).contains(&cp)
            || (0x1E00..0x1F00).contains(&cp)
            || (0x2000..0x2070).contains(&cp))
            && !ch.is_alphanumeric()
        {
            // a control or format character in a covered range: ignorable
        } else {
            // outside the table: after every scripted letter, in code point order (what Han and the unassigned do)
            self.p
                .extend_from_slice(&[0xFE, (cp >> 16) as u8, (cp >> 8) as u8, cp as u8]);
            self.s.push(0);
            self.t.push(0);
        }
    }

    fn of(s: &str) -> SortKey {
        let c = collation();
        let mut k = SortKey::default();
        for ch in s.chars() {
            k.add(c, ch);
        }
        k
    }
}

fn compare_keys(a: &str, b: &str, ignore_case: bool) -> Ordering {
    let (ka, kb) = (SortKey::of(a), SortKey::of(b));
    ka.p.cmp(&kb.p).then_with(|| ka.s.cmp(&kb.s)).then_with(|| {
        if ignore_case {
            Ordering::Equal
        } else {
            ka.t.cmp(&kb.t)
        }
    })
}

/// `string.CompareTo(string)`, `string.Compare(a, b)` and `Comparer<string>.Default`: a culture-sensitive comparison
/// under the invariant culture (`CultureInfo.DefaultThreadCurrentCulture`), which is ICU's root collation:
/// `"apple"` sorts before `"Zed"`, lower case before upper case, `"a-b"` before `"ab"`, `"é"` after `"e"` and before
/// `"f"`. Covers Latin, Greek, Cyrillic and general punctuation; other scripts sort after them by code point (the
/// order within Han and Arabic is the collation's, not that). Every expected value in the tests is a recorded
/// reference value (the pair table in `string_compare_table.txt`).
pub fn string_compare(a: &str, b: &str) -> Ordering {
    compare_keys(a, b, false)
}

/// `string.Compare(a, b, true)` / `StringComparison.CurrentCultureIgnoreCase` / `InvariantCultureIgnoreCase`.
pub fn string_compare_ignore_case(a: &str, b: &str) -> Ordering {
    compare_keys(a, b, true)
}

// ---- IPAddress.Parse -------------------------------------------------------------------------------

/// One part of an `inet_aton` address: `0x` hex, leading-`0` octal, else decimal. No sign, no spaces.
fn aton_part(p: &str) -> Option<u64> {
    let (digits, radix) = if let Some(h) = p.strip_prefix("0x").or_else(|| p.strip_prefix("0X")) {
        (h, 16)
    } else if p.len() > 1 && p.starts_with('0') {
        (&p[1..], 8)
    } else {
        (p, 10)
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    u64::from_str_radix(digits, radix)
        .ok()
        .filter(|&v| v <= u64::from(u32::MAX))
}

/// `IPAddress.TryParse`: IPv4 in the legacy forms (`127.1`; `010.0.0.1` is 8.0.0.1; `0x7f.1`; `2130706433`),
/// IPv6 with an optional `[..]` and `%scope` (the scope is dropped). Whitespace is not trimmed.
pub fn parse_ip_address(s: &str) -> Option<IpAddr> {
    if s.contains(':') {
        let t = s
            .strip_prefix('[')
            .and_then(|r| r.strip_suffix(']'))
            .unwrap_or(s);
        let t = t.split('%').next().unwrap_or(t);
        return t.parse::<Ipv6Addr>().ok().map(IpAddr::V6);
    }
    let parts: Vec<u64> = s.split('.').map(aton_part).collect::<Option<_>>()?;
    let (last, head) = parts.split_last()?;
    if parts.len() > 4 || head.iter().any(|&p| p > 255) || *last >= 1u64 << (8 * (5 - parts.len()))
    {
        return None;
    }
    let mut v = *last;
    for (i, &p) in head.iter().enumerate() {
        v |= p << (8 * (3 - i));
    }
    Some(IpAddr::V4(Ipv4Addr::from(v as u32)))
}

// ---- List<T>.Sort / Array.Sort -----------------------------------------------------------------------

/// `List<T>.Sort(Comparison)`, `Array.Sort(array, comparison)` and `Array.Sort(keys, items)`: introsort
/// (insertion sort up to 16, median-of-three quicksort, heap sort past the depth limit). Unstable, but the same
/// input always gives the same order. A stable LINQ `OrderBy` is `sort_by` instead.
pub fn net_sort<T>(v: &mut [T], mut cmp: impl FnMut(&T, &T) -> Ordering) {
    if v.len() > 1 {
        let depth = 2 * (v.len().ilog2() as usize + 1);
        intro_sort(v, depth, &mut cmp);
    }
}

fn swap_if_greater<T>(k: &mut [T], i: usize, j: usize, c: &mut impl FnMut(&T, &T) -> Ordering) {
    if i != j && c(&k[i], &k[j]) == Ordering::Greater {
        k.swap(i, j);
    }
}

fn intro_sort<T>(mut k: &mut [T], mut depth: usize, c: &mut impl FnMut(&T, &T) -> Ordering) {
    loop {
        let n = k.len();
        if n <= 1 {
            return;
        }
        if n <= 16 {
            match n {
                2 => swap_if_greater(k, 0, 1, c),
                3 => {
                    swap_if_greater(k, 0, 1, c);
                    swap_if_greater(k, 0, 2, c);
                    swap_if_greater(k, 1, 2, c);
                }
                _ => insertion_sort(k, c),
            }
            return;
        }
        if depth == 0 {
            heap_sort(k, c);
            return;
        }
        depth -= 1;
        let p = pick_pivot_and_partition(k, c);
        let (lo, hi) = std::mem::take(&mut k).split_at_mut(p);
        intro_sort(&mut hi[1..], depth, c);
        k = lo;
    }
}

fn pick_pivot_and_partition<T>(k: &mut [T], c: &mut impl FnMut(&T, &T) -> Ordering) -> usize {
    let hi = k.len() - 1;
    let middle = hi >> 1;
    swap_if_greater(k, 0, middle, c);
    swap_if_greater(k, 0, hi, c);
    swap_if_greater(k, middle, hi, c);
    k.swap(middle, hi - 1); // the pivot sits at hi - 1 until the end
    let (mut left, mut right) = (0, hi - 1);
    while left < right {
        loop {
            left += 1;
            if c(&k[left], &k[hi - 1]) != Ordering::Less {
                break;
            }
        }
        loop {
            right -= 1;
            if c(&k[hi - 1], &k[right]) != Ordering::Less {
                break;
            }
        }
        if left >= right {
            break;
        }
        k.swap(left, right);
    }
    if left != hi - 1 {
        k.swap(left, hi - 1);
    }
    left
}

fn heap_sort<T>(k: &mut [T], c: &mut impl FnMut(&T, &T) -> Ordering) {
    let n = k.len();
    for i in (1..=n >> 1).rev() {
        down_heap(k, i, n, c);
    }
    for i in (2..=n).rev() {
        k.swap(0, i - 1);
        down_heap(k, 1, i - 1, c);
    }
}

fn down_heap<T>(k: &mut [T], mut i: usize, n: usize, c: &mut impl FnMut(&T, &T) -> Ordering) {
    // The reference sort holds k[i - 1] aside and fills holes; moving it down by swaps leaves the same array
    while i <= n >> 1 {
        let mut child = 2 * i;
        if child < n && c(&k[child - 1], &k[child]) == Ordering::Less {
            child += 1;
        }
        if c(&k[i - 1], &k[child - 1]) != Ordering::Less {
            break;
        }
        k.swap(i - 1, child - 1);
        i = child;
    }
}

fn insertion_sort<T>(k: &mut [T], c: &mut impl FnMut(&T, &T) -> Ordering) {
    for i in 0..k.len() - 1 {
        let mut j = i + 1;
        while j > 0 && c(&k[j], &k[j - 1]) == Ordering::Less {
            k.swap(j, j - 1);
            j -= 1;
        }
    }
}

// ---- numbers ---------------------------------------------------------------------------------------------

/// `double.ToString()` (invariant culture): the shortest round-trip digits, scientific (`1E+17`, `1.5E-05`)
/// past 17 integer digits or below 1E-4. Rust's `{}` never uses an exponent.
pub fn double_to_string(x: f64) -> String {
    if !x.is_finite() {
        return non_finite(x);
    }
    sci_or_fixed(&format!("{x:e}"), 17)
}

/// `float.ToString()`: the same with 9 digits.
pub fn single_to_string(x: f32) -> String {
    if !x.is_finite() {
        return non_finite(f64::from(x));
    }
    sci_or_fixed(&format!("{x:e}"), 9)
}

fn non_finite(x: f64) -> String {
    match (x.is_nan(), x > 0.0) {
        (true, _) => "NaN",
        (_, true) => "Infinity",
        _ => "-Infinity",
    }
    .into()
}

/// `e` is Rust's `{:e}`: `[-]d[.ddd]e<exp>`.
fn sci_or_fixed(e: &str, max_scale: i32) -> String {
    let (sign, e) = e.strip_prefix('-').map_or(("", e), |r| ("-", r));
    let (mant, exp) = e.split_once('e').unwrap_or((e, "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    let scale = exp + 1; // digits before the decimal point
    if digits == "0" {
        return format!("{sign}0");
    }
    if scale > max_scale || scale < -3 {
        let frac = if digits.len() > 1 {
            format!(".{}", &digits[1..])
        } else {
            String::new()
        };
        return format!(
            "{sign}{}{frac}E{}{:02}",
            &digits[..1],
            if exp < 0 { '-' } else { '+' },
            exp.abs()
        );
    }
    if scale <= 0 {
        format!("{sign}0.{}{digits}", "0".repeat((-scale) as usize))
    } else if digits.len() as i32 <= scale {
        format!(
            "{sign}{digits}{}",
            "0".repeat((scale - digits.len() as i32) as usize)
        )
    } else {
        format!(
            "{sign}{}.{}",
            &digits[..scale as usize],
            &digits[scale as usize..]
        )
    }
}

/// `(short)d` in an unchecked context: converts to `int` (saturating, NaN is 0) and truncates the bits.
/// Rust's `d as i16` saturates at the narrow type instead.
pub fn f64_to_i16(d: f64) -> i16 {
    d as i32 as i16
}

/// `(ushort)d`, see [`f64_to_i16`].
pub fn f64_to_u16(d: f64) -> u16 {
    d as i32 as u16
}

/// `(sbyte)d`, see [`f64_to_i16`].
pub fn f64_to_i8(d: f64) -> i8 {
    d as i32 as i8
}

/// `(byte)d`, see [`f64_to_i16`].
pub fn f64_to_u8(d: f64) -> u8 {
    d as i32 as u8
}

/// The `Insensitive*` string helpers (`OrdinalIgnoreCase`) as methods, so `a.eq_ignore_ascii_case(b)` becomes `a.eq_ic(b)`.
pub trait IgnoreCase {
    /// `InsensitiveEquals`.
    fn eq_ic(&self, other: &str) -> bool;
    /// `InsensitiveStartsWith`.
    fn starts_with_ic(&self, prefix: &str) -> bool;
    /// `InsensitiveContains`.
    fn contains_ic(&self, needle: &str) -> bool;
    /// The key of an `OrdinalIgnoreCase` dictionary: `ToUpperInvariant()`.
    fn ic_key(&self) -> String;
}

impl IgnoreCase for str {
    fn eq_ic(&self, other: &str) -> bool {
        eq_ignore_case(self, other)
    }
    fn starts_with_ic(&self, prefix: &str) -> bool {
        starts_with_ignore_case(self, prefix)
    }
    fn contains_ic(&self, needle: &str) -> bool {
        contains_ignore_case(self, needle)
    }
    fn ic_key(&self) -> String {
        to_upper_invariant(self)
    }
}

/// `ToLowerInvariant()` / `ToUpperInvariant()` as methods, replacing `str::to_lowercase` / `to_uppercase`
/// (those use the full mapping: `İ` becomes two chars, `ß` becomes `SS`).
pub trait InvariantCase {
    fn lower_inv(&self) -> String;
    fn upper_inv(&self) -> String;
}

impl InvariantCase for str {
    fn lower_inv(&self) -> String {
        to_lower_invariant(self)
    }
    fn upper_inv(&self) -> String {
        to_upper_invariant(self)
    }
}

#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::*;

    #[test]
    fn case_mapping_matches_net() {
        // ToUpperInvariant: dotless i and ß stay, long s goes to S, titlecase Greek to its capital
        assert_eq!(to_upper_invariant("ıßſ"), "ıßS");
        assert_eq!(to_upper_invariant("\u{1F80}\u{1FB3}"), "\u{1F88}\u{1FBC}");
        assert_eq!(to_upper_invariant("привет"), "ПРИВЕТ");
        assert_eq!(to_lower_invariant("İK"), "İk");
        assert!(eq_ignore_case("Ärger", "äRGER"));
        assert!(!eq_ignore_case("i", "ı"));
        assert!(!eq_ignore_case("\u{212A}", "k")); // Kelvin sign
        assert!(eq_ignore_case("\u{10428}", "\u{10400}")); // Deseret, a surrogate pair
        assert!(starts_with_ignore_case("Orders now", "ORDERS"));
        assert!(contains_ignore_case("A Fine Day", "fine"));
        assert_eq!(index_of_ignore_case("abcDEF", "def"), Some(3));
        assert_eq!(cmp_ignore_case("a", "B"), Ordering::Less);
        assert_eq!(cmp_ignore_case("_", "a"), Ordering::Greater);
    }

    fn ip(s: &str) -> Option<String> {
        parse_ip_address(s).map(|a| a.to_string())
    }

    #[test]
    fn ip_address_parse_matches_net() {
        for (s, want) in [
            ("127.1", Some("127.0.0.1")),
            ("127.0.1", Some("127.0.0.1")),
            ("010.0.0.1", Some("8.0.0.1")),
            ("0x7f.1", Some("127.0.0.1")),
            ("0x7f000001", Some("127.0.0.1")),
            ("2130706433", Some("127.0.0.1")),
            ("1.2.3.4.5", None),
            ("1.2.3.", None),
            ("1..3", None),
            ("256.1.1.1", None),
            ("1.2.65536", None),
            ("1.65535.65535", None),
            ("1.16777215", Some("1.255.255.255")),
            ("4294967295", Some("255.255.255.255")),
            ("4294967296", None),
            (" 1.2.3.4", None),
            ("1.2.3.4 ", None),
            ("", None),
            ("0x", None),
            ("09.1.1.1", None),
            ("08", None),
            ("1.2.3.0x4", Some("1.2.3.4")),
            ("0X1F.1", Some("31.0.0.1")),
            ("1.2.3.-4", None),
            ("+1.2.3.4", None),
            ("1.2.3.4/24", None),
            ("01.02.03.04", Some("1.2.3.4")),
            ("00000001.1.1.1", Some("1.1.1.1")),
            ("12345678901234567890", None),
            ("::1", Some("::1")),
            ("[::1]", Some("::1")),
            ("::ffff:1.2.3.4", Some("::ffff:1.2.3.4")),
            ("fe80::1%5", Some("fe80::1")),
            ("::ffff:0102:0304", Some("::ffff:1.2.3.4")),
            ("1:0:0:0:0:0:0:1", Some("1::1")),
            ("2001:db8:0:0:1:0:0:1", Some("2001:db8::1:0:0:1")),
        ] {
            assert_eq!(ip(s).as_deref(), want, "{s:?}");
        }
    }

    /// The `(key, index)` list the reference probe sorted: key = (lcg >> 16) % k, lcg = lcg * 1103515245 + 12345.
    fn probe(n: usize, k: i64, seed: i64) -> Vec<(i64, usize)> {
        let mut x = seed;
        (0..n)
            .map(|i| {
                x = (x.wrapping_mul(1103515245).wrapping_add(12345)) & 0x7fff_ffff;
                ((x >> 16) % k, i)
            })
            .collect()
    }

    fn sorted_idx(n: usize, k: i64, seed: i64) -> Vec<usize> {
        let mut v = probe(n, k, seed);
        net_sort(&mut v, |a, b| a.0.cmp(&b.0));
        v.into_iter().map(|t| t.1).collect()
    }

    #[test]
    fn net_sort_matches_net_list_sort() {
        assert_eq!(sorted_idx(2, 2, 1), [0, 1]);
        assert_eq!(
            sorted_idx(16, 3, 3),
            [1, 2, 4, 6, 7, 12, 13, 5, 9, 11, 14, 15, 0, 3, 8, 10]
        );
        assert_eq!(
            sorted_idx(17, 3, 4),
            [8, 2, 3, 10, 9, 15, 14, 13, 12, 16, 1, 6, 11, 5, 4, 7, 0]
        );
        assert_eq!(
            sorted_idx(17, 100, 5),
            [6, 2, 10, 14, 16, 0, 1, 9, 11, 5, 7, 13, 8, 3, 12, 15, 4]
        );
        assert_eq!(
            sorted_idx(40, 4, 6),
            [
                9, 21, 17, 3, 34, 16, 33, 13, 29, 22, 27, 28, 23, 18, 7, 6, 5, 10, 26, 25, 32, 37,
                35, 0, 39, 20, 15, 12, 4, 2, 38, 14, 30, 31, 11, 8, 36, 1, 24, 19
            ]
        );
        let h = sorted_idx(300, 3, 9);
        assert_eq!(
            h[..12],
            [0, 140, 141, 143, 144, 145, 146, 298, 139, 151, 154, 159]
        );
        assert_eq!(h[295..], [183, 36, 186, 176, 299][..]);
        let h = sorted_idx(1000, 2, 10);
        assert_eq!(h[..10], [332, 788, 789, 422, 421, 420, 417, 416, 415, 412]);
        assert_eq!(h[994..], [449, 450, 451, 453, 455, 999]);
    }

    #[test]
    fn double_to_string_matches_net() {
        for (x, want) in [
            (0.0, "0"),
            (1.5, "1.5"),
            (0.1, "0.1"),
            (100.0, "100"),
            (1e15, "1000000000000000"),
            (1e16, "10000000000000000"),
            (12345678901234567.8, "12345678901234568"),
            (1e17, "1E+17"),
            (1.5e17, "1.5E+17"),
            (123456789012345678.0, "1.2345678901234568E+17"),
            (1e-5, "1E-05"),
            (1.2e-5, "1.2E-05"),
            (0.0001, "0.0001"),
            (0.00012, "0.00012"),
            (5e-324, "5E-324"),
            (1.7976931348623157e308, "1.7976931348623157E+308"),
            (0.30000000000000004, "0.30000000000000004"),
            (1234567.125, "1234567.125"),
            (100.0 / 3.0, "33.333333333333336"),
            (-1e-10, "-1E-10"),
            (-0.0, "-0"),
            (f64::NAN, "NaN"),
            (f64::NEG_INFINITY, "-Infinity"),
        ] {
            assert_eq!(double_to_string(x), want, "{x:e}");
        }
        for (x, want) in [
            (1e8_f32, "100000000"),
            (1e10, "1E+10"),
            (3.1415927, "3.1415927"),
            (1e-5, "1E-05"),
            (16777216.0, "16777216"),
        ] {
            assert_eq!(single_to_string(x), want);
        }
    }

    #[test]
    fn fixed_formats_agree_with_net() {
        // F/N formats round the exact binary value half to even, like Rust's {:.n}
        for (x, f0, f1, f2) in [
            (0.25, "0", "0.2", "0.25"),
            (0.75, "1", "0.8", "0.75"),
            (2.5, "2", "2.5", "2.50"),
            (3.5, "4", "3.5", "3.50"),
            (0.125, "0", "0.1", "0.12"),
            (0.375, "0", "0.4", "0.38"),
            (0.15, "0", "0.1", "0.15"),
            (1.005, "1", "1.0", "1.00"),
            (-0.25, "-0", "-0.2", "-0.25"),
            (-0.04, "-0", "-0.0", "-0.04"),
        ] {
            assert_eq!(format!("{x:.0}"), f0);
            assert_eq!(format!("{x:.1}"), f1);
            assert_eq!(format!("{x:.2}"), f2);
        }
    }

    #[test]
    fn narrow_casts_match_net() {
        assert_eq!(f64_to_u8(-3.9), 253);
        assert_eq!(f64_to_u8(2147483647.5), 255);
        assert_eq!(f64_to_i16(2147483647.5), -1);
        assert_eq!(f64_to_i16(-2147483649.0), 0);
        assert_eq!(f64_to_u16(-3.9), 65533);
        assert_eq!(f64_to_i8(f64::NAN), 0);
        assert_eq!(f64_to_u8(300.0), 44);
    }

    /// The recorded reference matrix: every pair of 134 strings, four comparisons each
    /// (`CompareTo`, `Compare(a, b, true)`, `Ordinal`, `InvariantCultureIgnoreCase`).
    #[test]
    fn string_compare_matches_net() {
        let text = include_str!("string_compare_table.txt");
        let mut lines = text.lines();
        let words: Vec<String> = lines
            .next()
            .unwrap()
            .split('\u{1}')
            .map(|w| {
                let units: Vec<u16> = w
                    .split("\\u")
                    .skip(1)
                    .map(|h| u16::from_str_radix(h, 16).unwrap())
                    .collect();
                String::from_utf16_lossy(&units)
            })
            .collect();
        let n = words.len();
        let rows: Vec<&str> = lines.take(4 * n).collect();
        let mut bad = Vec::new();
        for mode in [0usize, 1, 3] {
            for (i, a) in words.iter().enumerate() {
                let row: Vec<char> = rows[mode * n + i].chars().collect();
                for (j, b) in words.iter().enumerate() {
                    let want = match row[j] {
                        '<' => Ordering::Less,
                        '>' => Ordering::Greater,
                        _ => Ordering::Equal,
                    };
                    let got = if mode == 0 {
                        string_compare(a, b)
                    } else {
                        string_compare_ignore_case(a, b)
                    };
                    if got != want {
                        bad.push(format!(
                            "mode {mode}: {a:?} vs {b:?}: want {want:?}, got {got:?}"
                        ));
                    }
                }
            }
        }
        assert!(
            bad.is_empty(),
            "{} mismatches, first: {:#?}",
            bad.len(),
            &bad[..bad.len().min(25)]
        );
    }

    #[test]
    fn string_compare_basics() {
        assert_eq!(string_compare("apple", "Zed"), Ordering::Less);
        assert_eq!(string_compare("a", "A"), Ordering::Less);
        assert_eq!(string_compare_ignore_case("a", "A"), Ordering::Equal);
    }
}

// ---- char.IsLetterOrDigit -------------------------------------------------------------------------

/// `char.IsLetterOrDigit` on a UTF-16 unit: a letter (Lu, Ll, Lt, Lm, Lo) or a decimal digit (Nd), unlike std's
/// `is_alphanumeric` (which adds Nl, No and the alphabetic marks). A character above U+FFFF is two surrogates in
/// UTF-16, neither a letter, so it is false here. Table: Unicode 16.0.0.
pub fn is_letter_or_digit(c: char) -> bool {
    let Ok(u) = u16::try_from(u32::from(c)) else {
        return false;
    };
    LETTER_OR_DIGIT
        .binary_search_by(|&(lo, hi)| {
            if u < lo {
                Ordering::Greater
            } else if u > hi {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .is_ok()
}

const LETTER_OR_DIGIT: &[(u16, u16)] = &[
    (0x0030, 0x0039),
    (0x0041, 0x005A),
    (0x0061, 0x007A),
    (0x00AA, 0x00AA),
    (0x00B5, 0x00B5),
    (0x00BA, 0x00BA),
    (0x00C0, 0x00D6),
    (0x00D8, 0x00F6),
    (0x00F8, 0x02C1),
    (0x02C6, 0x02D1),
    (0x02E0, 0x02E4),
    (0x02EC, 0x02EC),
    (0x02EE, 0x02EE),
    (0x0370, 0x0374),
    (0x0376, 0x0377),
    (0x037A, 0x037D),
    (0x037F, 0x037F),
    (0x0386, 0x0386),
    (0x0388, 0x038A),
    (0x038C, 0x038C),
    (0x038E, 0x03A1),
    (0x03A3, 0x03F5),
    (0x03F7, 0x0481),
    (0x048A, 0x052F),
    (0x0531, 0x0556),
    (0x0559, 0x0559),
    (0x0560, 0x0588),
    (0x05D0, 0x05EA),
    (0x05EF, 0x05F2),
    (0x0620, 0x064A),
    (0x0660, 0x0669),
    (0x066E, 0x066F),
    (0x0671, 0x06D3),
    (0x06D5, 0x06D5),
    (0x06E5, 0x06E6),
    (0x06EE, 0x06FC),
    (0x06FF, 0x06FF),
    (0x0710, 0x0710),
    (0x0712, 0x072F),
    (0x074D, 0x07A5),
    (0x07B1, 0x07B1),
    (0x07C0, 0x07EA),
    (0x07F4, 0x07F5),
    (0x07FA, 0x07FA),
    (0x0800, 0x0815),
    (0x081A, 0x081A),
    (0x0824, 0x0824),
    (0x0828, 0x0828),
    (0x0840, 0x0858),
    (0x0860, 0x086A),
    (0x0870, 0x0887),
    (0x0889, 0x088E),
    (0x08A0, 0x08C9),
    (0x0904, 0x0939),
    (0x093D, 0x093D),
    (0x0950, 0x0950),
    (0x0958, 0x0961),
    (0x0966, 0x096F),
    (0x0971, 0x0980),
    (0x0985, 0x098C),
    (0x098F, 0x0990),
    (0x0993, 0x09A8),
    (0x09AA, 0x09B0),
    (0x09B2, 0x09B2),
    (0x09B6, 0x09B9),
    (0x09BD, 0x09BD),
    (0x09CE, 0x09CE),
    (0x09DC, 0x09DD),
    (0x09DF, 0x09E1),
    (0x09E6, 0x09F1),
    (0x09FC, 0x09FC),
    (0x0A05, 0x0A0A),
    (0x0A0F, 0x0A10),
    (0x0A13, 0x0A28),
    (0x0A2A, 0x0A30),
    (0x0A32, 0x0A33),
    (0x0A35, 0x0A36),
    (0x0A38, 0x0A39),
    (0x0A59, 0x0A5C),
    (0x0A5E, 0x0A5E),
    (0x0A66, 0x0A6F),
    (0x0A72, 0x0A74),
    (0x0A85, 0x0A8D),
    (0x0A8F, 0x0A91),
    (0x0A93, 0x0AA8),
    (0x0AAA, 0x0AB0),
    (0x0AB2, 0x0AB3),
    (0x0AB5, 0x0AB9),
    (0x0ABD, 0x0ABD),
    (0x0AD0, 0x0AD0),
    (0x0AE0, 0x0AE1),
    (0x0AE6, 0x0AEF),
    (0x0AF9, 0x0AF9),
    (0x0B05, 0x0B0C),
    (0x0B0F, 0x0B10),
    (0x0B13, 0x0B28),
    (0x0B2A, 0x0B30),
    (0x0B32, 0x0B33),
    (0x0B35, 0x0B39),
    (0x0B3D, 0x0B3D),
    (0x0B5C, 0x0B5D),
    (0x0B5F, 0x0B61),
    (0x0B66, 0x0B6F),
    (0x0B71, 0x0B71),
    (0x0B83, 0x0B83),
    (0x0B85, 0x0B8A),
    (0x0B8E, 0x0B90),
    (0x0B92, 0x0B95),
    (0x0B99, 0x0B9A),
    (0x0B9C, 0x0B9C),
    (0x0B9E, 0x0B9F),
    (0x0BA3, 0x0BA4),
    (0x0BA8, 0x0BAA),
    (0x0BAE, 0x0BB9),
    (0x0BD0, 0x0BD0),
    (0x0BE6, 0x0BEF),
    (0x0C05, 0x0C0C),
    (0x0C0E, 0x0C10),
    (0x0C12, 0x0C28),
    (0x0C2A, 0x0C39),
    (0x0C3D, 0x0C3D),
    (0x0C58, 0x0C5A),
    (0x0C5D, 0x0C5D),
    (0x0C60, 0x0C61),
    (0x0C66, 0x0C6F),
    (0x0C80, 0x0C80),
    (0x0C85, 0x0C8C),
    (0x0C8E, 0x0C90),
    (0x0C92, 0x0CA8),
    (0x0CAA, 0x0CB3),
    (0x0CB5, 0x0CB9),
    (0x0CBD, 0x0CBD),
    (0x0CDD, 0x0CDE),
    (0x0CE0, 0x0CE1),
    (0x0CE6, 0x0CEF),
    (0x0CF1, 0x0CF2),
    (0x0D04, 0x0D0C),
    (0x0D0E, 0x0D10),
    (0x0D12, 0x0D3A),
    (0x0D3D, 0x0D3D),
    (0x0D4E, 0x0D4E),
    (0x0D54, 0x0D56),
    (0x0D5F, 0x0D61),
    (0x0D66, 0x0D6F),
    (0x0D7A, 0x0D7F),
    (0x0D85, 0x0D96),
    (0x0D9A, 0x0DB1),
    (0x0DB3, 0x0DBB),
    (0x0DBD, 0x0DBD),
    (0x0DC0, 0x0DC6),
    (0x0DE6, 0x0DEF),
    (0x0E01, 0x0E30),
    (0x0E32, 0x0E33),
    (0x0E40, 0x0E46),
    (0x0E50, 0x0E59),
    (0x0E81, 0x0E82),
    (0x0E84, 0x0E84),
    (0x0E86, 0x0E8A),
    (0x0E8C, 0x0EA3),
    (0x0EA5, 0x0EA5),
    (0x0EA7, 0x0EB0),
    (0x0EB2, 0x0EB3),
    (0x0EBD, 0x0EBD),
    (0x0EC0, 0x0EC4),
    (0x0EC6, 0x0EC6),
    (0x0ED0, 0x0ED9),
    (0x0EDC, 0x0EDF),
    (0x0F00, 0x0F00),
    (0x0F20, 0x0F29),
    (0x0F40, 0x0F47),
    (0x0F49, 0x0F6C),
    (0x0F88, 0x0F8C),
    (0x1000, 0x102A),
    (0x103F, 0x1049),
    (0x1050, 0x1055),
    (0x105A, 0x105D),
    (0x1061, 0x1061),
    (0x1065, 0x1066),
    (0x106E, 0x1070),
    (0x1075, 0x1081),
    (0x108E, 0x108E),
    (0x1090, 0x1099),
    (0x10A0, 0x10C5),
    (0x10C7, 0x10C7),
    (0x10CD, 0x10CD),
    (0x10D0, 0x10FA),
    (0x10FC, 0x1248),
    (0x124A, 0x124D),
    (0x1250, 0x1256),
    (0x1258, 0x1258),
    (0x125A, 0x125D),
    (0x1260, 0x1288),
    (0x128A, 0x128D),
    (0x1290, 0x12B0),
    (0x12B2, 0x12B5),
    (0x12B8, 0x12BE),
    (0x12C0, 0x12C0),
    (0x12C2, 0x12C5),
    (0x12C8, 0x12D6),
    (0x12D8, 0x1310),
    (0x1312, 0x1315),
    (0x1318, 0x135A),
    (0x1380, 0x138F),
    (0x13A0, 0x13F5),
    (0x13F8, 0x13FD),
    (0x1401, 0x166C),
    (0x166F, 0x167F),
    (0x1681, 0x169A),
    (0x16A0, 0x16EA),
    (0x16F1, 0x16F8),
    (0x1700, 0x1711),
    (0x171F, 0x1731),
    (0x1740, 0x1751),
    (0x1760, 0x176C),
    (0x176E, 0x1770),
    (0x1780, 0x17B3),
    (0x17D7, 0x17D7),
    (0x17DC, 0x17DC),
    (0x17E0, 0x17E9),
    (0x1810, 0x1819),
    (0x1820, 0x1878),
    (0x1880, 0x1884),
    (0x1887, 0x18A8),
    (0x18AA, 0x18AA),
    (0x18B0, 0x18F5),
    (0x1900, 0x191E),
    (0x1946, 0x196D),
    (0x1970, 0x1974),
    (0x1980, 0x19AB),
    (0x19B0, 0x19C9),
    (0x19D0, 0x19D9),
    (0x1A00, 0x1A16),
    (0x1A20, 0x1A54),
    (0x1A80, 0x1A89),
    (0x1A90, 0x1A99),
    (0x1AA7, 0x1AA7),
    (0x1B05, 0x1B33),
    (0x1B45, 0x1B4C),
    (0x1B50, 0x1B59),
    (0x1B83, 0x1BA0),
    (0x1BAE, 0x1BE5),
    (0x1C00, 0x1C23),
    (0x1C40, 0x1C49),
    (0x1C4D, 0x1C7D),
    (0x1C80, 0x1C8A),
    (0x1C90, 0x1CBA),
    (0x1CBD, 0x1CBF),
    (0x1CE9, 0x1CEC),
    (0x1CEE, 0x1CF3),
    (0x1CF5, 0x1CF6),
    (0x1CFA, 0x1CFA),
    (0x1D00, 0x1DBF),
    (0x1E00, 0x1F15),
    (0x1F18, 0x1F1D),
    (0x1F20, 0x1F45),
    (0x1F48, 0x1F4D),
    (0x1F50, 0x1F57),
    (0x1F59, 0x1F59),
    (0x1F5B, 0x1F5B),
    (0x1F5D, 0x1F5D),
    (0x1F5F, 0x1F7D),
    (0x1F80, 0x1FB4),
    (0x1FB6, 0x1FBC),
    (0x1FBE, 0x1FBE),
    (0x1FC2, 0x1FC4),
    (0x1FC6, 0x1FCC),
    (0x1FD0, 0x1FD3),
    (0x1FD6, 0x1FDB),
    (0x1FE0, 0x1FEC),
    (0x1FF2, 0x1FF4),
    (0x1FF6, 0x1FFC),
    (0x2071, 0x2071),
    (0x207F, 0x207F),
    (0x2090, 0x209C),
    (0x2102, 0x2102),
    (0x2107, 0x2107),
    (0x210A, 0x2113),
    (0x2115, 0x2115),
    (0x2119, 0x211D),
    (0x2124, 0x2124),
    (0x2126, 0x2126),
    (0x2128, 0x2128),
    (0x212A, 0x212D),
    (0x212F, 0x2139),
    (0x213C, 0x213F),
    (0x2145, 0x2149),
    (0x214E, 0x214E),
    (0x2183, 0x2184),
    (0x2C00, 0x2CE4),
    (0x2CEB, 0x2CEE),
    (0x2CF2, 0x2CF3),
    (0x2D00, 0x2D25),
    (0x2D27, 0x2D27),
    (0x2D2D, 0x2D2D),
    (0x2D30, 0x2D67),
    (0x2D6F, 0x2D6F),
    (0x2D80, 0x2D96),
    (0x2DA0, 0x2DA6),
    (0x2DA8, 0x2DAE),
    (0x2DB0, 0x2DB6),
    (0x2DB8, 0x2DBE),
    (0x2DC0, 0x2DC6),
    (0x2DC8, 0x2DCE),
    (0x2DD0, 0x2DD6),
    (0x2DD8, 0x2DDE),
    (0x2E2F, 0x2E2F),
    (0x3005, 0x3006),
    (0x3031, 0x3035),
    (0x303B, 0x303C),
    (0x3041, 0x3096),
    (0x309D, 0x309F),
    (0x30A1, 0x30FA),
    (0x30FC, 0x30FF),
    (0x3105, 0x312F),
    (0x3131, 0x318E),
    (0x31A0, 0x31BF),
    (0x31F0, 0x31FF),
    (0x3400, 0x4DBF),
    (0x4E00, 0xA48C),
    (0xA4D0, 0xA4FD),
    (0xA500, 0xA60C),
    (0xA610, 0xA62B),
    (0xA640, 0xA66E),
    (0xA67F, 0xA69D),
    (0xA6A0, 0xA6E5),
    (0xA717, 0xA71F),
    (0xA722, 0xA788),
    (0xA78B, 0xA7CD),
    (0xA7D0, 0xA7D1),
    (0xA7D3, 0xA7D3),
    (0xA7D5, 0xA7DC),
    (0xA7F2, 0xA801),
    (0xA803, 0xA805),
    (0xA807, 0xA80A),
    (0xA80C, 0xA822),
    (0xA840, 0xA873),
    (0xA882, 0xA8B3),
    (0xA8D0, 0xA8D9),
    (0xA8F2, 0xA8F7),
    (0xA8FB, 0xA8FB),
    (0xA8FD, 0xA8FE),
    (0xA900, 0xA925),
    (0xA930, 0xA946),
    (0xA960, 0xA97C),
    (0xA984, 0xA9B2),
    (0xA9CF, 0xA9D9),
    (0xA9E0, 0xA9E4),
    (0xA9E6, 0xA9FE),
    (0xAA00, 0xAA28),
    (0xAA40, 0xAA42),
    (0xAA44, 0xAA4B),
    (0xAA50, 0xAA59),
    (0xAA60, 0xAA76),
    (0xAA7A, 0xAA7A),
    (0xAA7E, 0xAAAF),
    (0xAAB1, 0xAAB1),
    (0xAAB5, 0xAAB6),
    (0xAAB9, 0xAABD),
    (0xAAC0, 0xAAC0),
    (0xAAC2, 0xAAC2),
    (0xAADB, 0xAADD),
    (0xAAE0, 0xAAEA),
    (0xAAF2, 0xAAF4),
    (0xAB01, 0xAB06),
    (0xAB09, 0xAB0E),
    (0xAB11, 0xAB16),
    (0xAB20, 0xAB26),
    (0xAB28, 0xAB2E),
    (0xAB30, 0xAB5A),
    (0xAB5C, 0xAB69),
    (0xAB70, 0xABE2),
    (0xABF0, 0xABF9),
    (0xAC00, 0xD7A3),
    (0xD7B0, 0xD7C6),
    (0xD7CB, 0xD7FB),
    (0xF900, 0xFA6D),
    (0xFA70, 0xFAD9),
    (0xFB00, 0xFB06),
    (0xFB13, 0xFB17),
    (0xFB1D, 0xFB1D),
    (0xFB1F, 0xFB28),
    (0xFB2A, 0xFB36),
    (0xFB38, 0xFB3C),
    (0xFB3E, 0xFB3E),
    (0xFB40, 0xFB41),
    (0xFB43, 0xFB44),
    (0xFB46, 0xFBB1),
    (0xFBD3, 0xFD3D),
    (0xFD50, 0xFD8F),
    (0xFD92, 0xFDC7),
    (0xFDF0, 0xFDFB),
    (0xFE70, 0xFE74),
    (0xFE76, 0xFEFC),
    (0xFF10, 0xFF19),
    (0xFF21, 0xFF3A),
    (0xFF41, 0xFF5A),
    (0xFF66, 0xFFBE),
    (0xFFC2, 0xFFC7),
    (0xFFCA, 0xFFCF),
    (0xFFD2, 0xFFD7),
    (0xFFDA, 0xFFDC),
];

#[cfg(test)]
mod letter_or_digit_tests {
    use super::is_letter_or_digit;

    #[test]
    fn matches_the_utf16_categories() {
        assert!(is_letter_or_digit('a') && is_letter_or_digit('Z') && is_letter_or_digit('7'));
        assert!(is_letter_or_digit('\u{E9}') && is_letter_or_digit('\u{0663}')); // Ll, Nd
        assert!(!is_letter_or_digit('\u{BD}')); // No: std is_alphanumeric says true
        assert!(!is_letter_or_digit('\u{2160}')); // Nl: same
        assert!(!is_letter_or_digit('\u{0301}')); // Mn
        assert!(!is_letter_or_digit('_') && !is_letter_or_digit(' '));
        assert!(!is_letter_or_digit('\u{1D400}')); // astral: surrogates in UTF-16
    }
}
