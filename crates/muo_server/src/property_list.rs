//! Object property lists (`IObjectPropertyListEntity`, `IPropertyList`, `ObjectPropertyList`, `OplTextBlock`).
//!
//! `IPropertyList` and `ObjectPropertyList` are one type: `get_properties(.., list)` fills it. The
//! interpolated-string handler overloads (`Add($"..")`, `Add(number, $"..")`) are `add_fmt(format_args!(..))` /
//! `add_number_fmt(number, format_args!(..))`; `{v:#}` on an integer (the cliloc marker "#<v>") is [`cliloc_arg`].
//! Lengths are UTF-16 units, as on the wire.
//!
//! `ObjectPropertyList.Enabled` (a static) is not here: it belongs to the world's settings.
//! `IObjectPropertyListEntity` has no Rust counterpart: `Item` and `Mobile` carry `property_list` and the
//! `get_properties` virtual.

use muo_primitives::Serial;
use muo_primitives::hash::compute_hash64_bytes;

/// Each of these is localized to "~1_NOTHING~" which allows the string argument to be used.
const STRING_NUMBERS: [i32; 5] = [1042971, 1070722, 1114057, 1114778, 1114779];

/// Max characters for a single OPL property argument (the legacy 2D client's buffer).
pub const MAX_ARGUMENT_LENGTH: usize = 504;

/// 0xD6 header: packet id, length, unknown, serial, unknown, hash.
const HEADER_LENGTH: usize = 15;

/// Terminate writes the bare hash into 0xD6, the 0xDC info packet sends `hash()` with bit 30 set.
const HASH_MASK: u64 = 0x3FF_FFFF;

/// `{value:#}` on an integer: the cliloc marker.
pub fn cliloc_arg(value: i64) -> String {
    format!("#{value}")
}

/// `ObjectPropertyList`.
#[derive(Clone, Debug)]
pub struct PropertyList {
    hash: i32,
    string_numbers_index: usize,
    buffer: Vec<u8>,
    buffer_pos: usize,
    /// `Entity.Serial` for the log line (`IEntity? Entity`; zero when null).
    entity: Serial,
    /// `Header`: the first cliloc added.
    pub header: i32,
    /// `HeaderArgs`: null until the first add.
    pub header_args: Option<String>,
    /// `IsBuilding`: true while GetProperties populates this list.
    pub is_building: bool,
}

impl Default for PropertyList {
    fn default() -> Self {
        PropertyList::new(Serial::ZERO)
    }
}

impl PropertyList {
    /// `new ObjectPropertyList(e)`; `Serial::ZERO` for a null entity.
    pub fn new(entity: Serial) -> PropertyList {
        let mut buffer = Vec::with_capacity(64);
        buffer.push(0xD6);
        buffer.extend_from_slice(&[0, 0]); // length, written by terminate
        buffer.extend_from_slice(&1u16.to_be_bytes());
        buffer.extend_from_slice(&entity.value().to_be_bytes());
        buffer.extend_from_slice(&0u16.to_be_bytes());
        buffer.extend_from_slice(&[0; 4]); // hash
        PropertyList {
            hash: 0,
            string_numbers_index: 0,
            buffer,
            buffer_pos: HEADER_LENGTH,
            entity,
            header: 0,
            header_args: None,
            is_building: false,
        }
    }

    /// `Hash`: 0x40000000 + the bare hash, what 0xDC carries.
    pub fn hash(&self) -> i32 {
        0x4000_0000 + self.hash
    }

    /// `Buffer`: the 0xD6 packet once `terminate` has run.
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// `Reset`.
    pub fn reset(&mut self) {
        self.buffer_pos = HEADER_LENGTH;
        self.hash = 0;
        self.string_numbers_index = 0;
        self.header = 0;
        self.header_args = None;
    }

    /// `Terminate`: xxHash3 over the property block, the zero terminator, the hash and the length.
    pub fn terminate(&mut self) {
        let length = self.buffer_pos + 4;
        self.buffer.resize(length, 0);
        self.hash =
            (compute_hash64_bytes(&self.buffer[HEADER_LENGTH..self.buffer_pos]) & HASH_MASK) as i32;
        self.buffer[self.buffer_pos..length].fill(0);
        self.buffer[11..15].copy_from_slice(&self.hash.to_be_bytes());
        // WritePacketLength: BytesWritten is the high-water mark, the whole buffer.
        self.buffer[1..3].copy_from_slice(&(length as u16).to_be_bytes());
    }

    /// `Add(int number)`.
    pub fn add(&mut self, number: i32) {
        if number == 0 {
            return;
        }
        if self.header == 0 {
            self.header = number;
            self.header_args = Some(String::new());
        }
        self.buffer.truncate(self.buffer_pos);
        self.buffer.extend_from_slice(&number.to_be_bytes());
        self.buffer.extend_from_slice(&0u16.to_be_bytes());
        self.buffer_pos += 6;
    }

    /// `Add(int number, string arguments)`.
    pub fn add_str(&mut self, number: i32, arguments: &str) {
        self.internal_add(number, &utf16(arguments));
    }

    /// `Add(int number, int value)`.
    pub fn add_int(&mut self, number: i32, value: i32) {
        self.add_str(number, &value.to_string());
    }

    /// `AddLocalized(int value)`: `$"{value:#}"` under the next passthrough cliloc.
    pub fn add_localized(&mut self, value: i32) {
        let number = self.next_string_number();
        self.add_str(number, &cliloc_arg(i64::from(value)));
    }

    /// `AddLocalized(int number, int value)`.
    pub fn add_localized_number(&mut self, number: i32, value: i32) {
        self.add_str(number, &cliloc_arg(i64::from(value)));
    }

    /// `Add(ReadOnlySpan<char>)` / `Add(string)`: under the next passthrough cliloc.
    pub fn add_text(&mut self, argument: &str) {
        let number = self.next_string_number();
        self.internal_add(number, &utf16(argument));
    }

    /// `Add($"..")` (interpolation handler): `add_fmt(format_args!(..))`.
    pub fn add_fmt(&mut self, args: std::fmt::Arguments) {
        self.add_text(&args.to_string());
    }

    /// `Add(int number, $"..")`.
    pub fn add_number_fmt(&mut self, number: i32, args: std::fmt::Arguments) {
        self.add_str(number, &args.to_string());
    }

    /// `TextBlock()`: finish with [`OplTextBlock::dispose`].
    pub fn text_block(&self) -> OplTextBlock {
        OplTextBlock::default()
    }

    /// `AddChunked`: newline-joined text over as many properties as needed, breaking only at '\n'.
    pub fn add_chunked(&mut self, text: &str) {
        let t = utf16(text);
        if t.is_empty() {
            return;
        }
        let mut chunk_start = 0;
        let mut search_from = 0;
        loop {
            let nl = t[search_from..].iter().position(|&c| c == 0x0A);
            let line_end = nl.map_or(t.len(), |n| search_from + n);

            // Appending this line would push the chunk past the cap: flush up to the previous line's end.
            if line_end - chunk_start > MAX_ARGUMENT_LENGTH && search_from > chunk_start {
                let number = self.next_string_number();
                self.internal_add(number, &t[chunk_start..search_from - 1]);
                chunk_start = search_from;
                continue;
            }

            if nl.is_none() {
                let number = self.next_string_number();
                self.internal_add(number, &t[chunk_start..]); // a single over-cap line is clamped by the add
                return;
            }
            search_from = line_end + 1;
        }
    }

    fn next_string_number(&mut self) -> i32 {
        let n = STRING_NUMBERS[self.string_numbers_index % STRING_NUMBERS.len()];
        self.string_numbers_index += 1;
        n
    }

    /// `ClampArgument` + `InternalAdd`.
    fn internal_add(&mut self, number: i32, chars: &[u16]) {
        if number == 0 {
            return;
        }
        let chars = if chars.len() <= MAX_ARGUMENT_LENGTH {
            chars
        } else {
            tracing::warn!(
                "OPL property on {:?} (cliloc {number}) is {} chars; truncating to {MAX_ARGUMENT_LENGTH} to avoid legacy 2D-client tooltip-buffer overflow. Use AddChunked for multi-line text.",
                self.entity,
                chars.len()
            );
            &chars[..MAX_ARGUMENT_LENGTH]
        };
        if self.header == 0 {
            self.header = number;
            self.header_args = Some(String::from_utf16_lossy(chars));
        }
        let str_length = chars.len() * 2;
        self.buffer.truncate(self.buffer_pos);
        self.buffer.extend_from_slice(&number.to_be_bytes());
        self.buffer
            .extend_from_slice(&(str_length as u16).to_be_bytes());
        for c in chars {
            self.buffer.extend_from_slice(&c.to_le_bytes());
        }
        self.buffer_pos += 6 + str_length;
    }
}

fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// `OplTextBlock`: accumulates '\n'-joined lines, emitted by [`OplTextBlock::dispose`] through `add_chunked`.
#[derive(Debug, Default)]
pub struct OplTextBlock {
    builder: String,
    any: bool,
}

impl OplTextBlock {
    /// `Add(ReadOnlySpan<char>)`: an empty line is skipped.
    pub fn add(&mut self, line: &str) {
        if line.is_empty() {
            return;
        }
        self.builder.push_str(line);
        self.builder.push('\n');
        self.any = true;
    }

    /// The interpolated `Add($"..")`: `block.add_fmt(format_args!(..))`.
    /// The handler's `_wrote` is true once any part ran (a hole, even an empty one); only `$""` writes nothing.
    pub fn add_fmt(&mut self, args: std::fmt::Arguments) {
        let s = args.to_string();
        if args.as_str() != Some("") {
            self.builder.push_str(&s);
            self.builder.push('\n');
            self.any = true;
        }
    }

    /// `Dispose`: strips the trailing '\n' and chunks the text into the list.
    pub fn dispose(self, list: &mut PropertyList) {
        if self.any {
            list.add_chunked(&self.builder[..self.builder.len() - 1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_hash(props: &[(i32, Option<&str>)]) -> i32 {
        let mut opl = PropertyList::new(Serial::ZERO);
        for &(cliloc, arg) in props {
            match arg {
                None => opl.add(cliloc),
                Some(a) => opl.add_str(cliloc, a),
            }
        }
        opl.terminate();
        opl.hash()
    }

    fn decode(opl: &mut PropertyList) -> Vec<(i32, String)> {
        opl.terminate();
        let b = opl.buffer();
        let mut pos = 15;
        let mut out = vec![];
        loop {
            let cliloc = i32::from_be_bytes(b[pos..pos + 4].try_into().unwrap());
            pos += 4;
            if cliloc == 0 {
                break;
            }
            let len = u16::from_be_bytes([b[pos], b[pos + 1]]) as usize;
            pos += 2;
            let units: Vec<u16> = b[pos..pos + len]
                .chunks(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            out.push((cliloc, String::from_utf16_lossy(&units)));
            pos += len;
        }
        out
    }

    #[test]
    fn repeated_argument_does_not_cancel_out() {
        let ten = build_hash(&[
            (1063752, Some("10")),
            (1063737, Some("10")),
            (1063740, None),
        ]);
        let five = build_hash(&[(1063752, Some("5")), (1063737, Some("5")), (1063740, None)]);
        assert_ne!(ten, five);
    }

    #[test]
    fn duplicate_order_and_swap_change_hash() {
        assert_ne!(
            build_hash(&[(1060658, None)]),
            build_hash(&[(1060658, None), (1060658, None)])
        );
        assert_ne!(
            build_hash(&[(1060658, Some("Alpha")), (1060659, Some("Beta"))]),
            build_hash(&[(1060659, Some("Beta")), (1060658, Some("Alpha"))])
        );
        assert_ne!(
            build_hash(&[(1063752, Some("10")), (1063737, Some("5"))]),
            build_hash(&[(1063752, Some("5")), (1063737, Some("10"))])
        );
        let p = [(1063752, Some("10")), (1063737, Some("5")), (1063740, None)];
        assert_eq!(build_hash(&p), build_hash(&p));
    }

    #[test]
    fn hash_stays_within_the_revision_mask() {
        let mut opl = PropertyList::new(Serial::ZERO);
        opl.add_str(
            1063752,
            "A rather long argument that pushes the buffer past its initial size",
        );
        opl.add_str(1063737, "12345");
        opl.add(1063740);
        opl.terminate();
        assert_eq!(opl.hash() & !0x3FF_FFFF, 0x4000_0000);
    }

    #[test]
    fn short_blocks_are_well_distributed() {
        let count = 5000;
        let mut with_arg = std::collections::HashSet::new();
        let mut without = std::collections::HashSet::new();
        let mut previous = 0;
        for i in 0..count {
            let cur = build_hash(&[(1060584, Some(&i.to_string()))]);
            if i > 0 {
                assert_ne!(previous, cur);
            }
            previous = cur;
            with_arg.insert(cur);
            without.insert(build_hash(&[(1060000 + i, None)]));
        }
        assert!(with_arg.len() >= count as usize - 5);
        assert!(without.len() >= count as usize - 5);
    }

    #[test]
    fn empty_list_is_nonzero_and_distinct() {
        let mut empty = PropertyList::new(Serial::ZERO);
        empty.terminate();
        assert_ne!(empty.hash(), 0);
        assert_eq!(empty.hash() & !0x3FF_FFFF, 0x4000_0000);
        assert_ne!(empty.hash(), build_hash(&[(1060658, None)]));
    }

    // Golden: the bare hash is the low 26 bits of xxHash3-64 over the property block; the block and the packet
    // around it are literal bytes (1070722 = 0x00105682).
    #[test]
    fn hash_is_xxh3_of_the_property_block() {
        let mut opl = PropertyList::new(Serial::new(0x4000_0001));
        opl.add_str(1070722, "Hi");
        opl.terminate();
        let block = [0x00, 0x10, 0x56, 0x82, 0x00, 0x04, b'H', 0, b'i', 0];
        assert_eq!(&opl.buffer()[15..25], &block);
        let expected = (compute_hash64_bytes(&block) & 0x3FF_FFFF) as i32;
        assert_eq!(opl.hash(), 0x4000_0000 + expected);
        let b = opl.buffer();
        assert_eq!(&b[..11], &[0xD6, 0, 29, 0, 1, 0x40, 0, 0, 1, 0, 0]);
        assert_eq!(&b[11..15], &expected.to_be_bytes());
        assert_eq!(&b[25..29], &[0; 4]);
    }

    #[test]
    fn span_add_matches_string_and_numbers_cycle() {
        let mut a = PropertyList::new(Serial::ZERO);
        a.add_str(1070722, "Hello World");
        let mut b = PropertyList::new(Serial::ZERO);
        b.add_number_fmt(1070722, format_args!("Hello {}", "World"));
        assert_eq!(decode(&mut a), decode(&mut b));
        assert_eq!(a.hash(), b.hash());
        let mut c = PropertyList::new(Serial::ZERO);
        for _ in 0..6 {
            c.add_text("x");
        }
        let nums: Vec<i32> = decode(&mut c).iter().map(|e| e.0).collect();
        assert_eq!(nums, [1042971, 1070722, 1114057, 1114778, 1114779, 1042971]);
    }

    #[test]
    fn hash_format_marks_integers() {
        let mut l = PropertyList::new(Serial::ZERO);
        l.add_str(1062028, &cliloc_arg(1043009));
        assert_eq!(decode(&mut l)[0], (1062028, "#1043009".to_string()));
        let mut l = PropertyList::new(Serial::ZERO);
        l.add_localized(5);
        assert_eq!(decode(&mut l)[0], (1042971, "#5".to_string()));
    }

    #[test]
    fn add_truncates_over_max_length() {
        let mut l = PropertyList::new(Serial::ZERO);
        l.add_str(1070722, &"x".repeat(MAX_ARGUMENT_LENGTH + 50));
        let e = decode(&mut l);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].1.len(), MAX_ARGUMENT_LENGTH);
        assert_eq!(l.header_args.as_ref().unwrap().len(), MAX_ARGUMENT_LENGTH);
    }

    #[test]
    fn add_chunked_splits_at_newlines() {
        let lines: Vec<String> = (0..10u8)
            .map(|i| ((b'a' + i) as char).to_string().repeat(100))
            .collect();
        let text = lines.join("\n");
        let mut l = PropertyList::new(Serial::ZERO);
        l.add_chunked(&text);
        let e = decode(&mut l);
        assert!(e.len() > 1);
        assert!(e.iter().all(|x| x.1.len() <= MAX_ARGUMENT_LENGTH));
        let rejoined: Vec<&str> = e.iter().map(|x| x.1.as_str()).collect();
        assert_eq!(rejoined.join("\n"), text);
    }

    #[test]
    fn text_block() {
        let mut l = PropertyList::new(Serial::ZERO);
        let mut b = l.text_block();
        b.add("Line One");
        b.add("Line Two");
        b.add("");
        b.dispose(&mut l);
        assert_eq!(
            decode(&mut l),
            [(1042971, "Line One\nLine Two".to_string())]
        );

        let mut l = PropertyList::new(Serial::ZERO);
        let mut b = l.text_block();
        b.add_fmt(format_args!("Luck Bonus: +{}%", 5));
        b.dispose(&mut l);
        assert_eq!(decode(&mut l)[0], (1042971, "Luck Bonus: +5%".to_string()));

        let mut l = PropertyList::new(Serial::ZERO);
        l.text_block().dispose(&mut l);
        assert!(decode(&mut l).is_empty());
    }

    #[test]
    fn reset_keeps_header_and_restarts() {
        let mut l = PropertyList::new(Serial::new(5));
        l.add_str(1070722, "abc");
        assert_eq!((l.header, l.header_args.as_deref()), (1070722, Some("abc")));
        l.reset();
        assert_eq!((l.header, l.header_args.as_deref()), (0, None));
        l.terminate();
        assert_eq!(l.buffer().len(), 19);
        assert_eq!(&l.buffer()[..9], &[0xD6, 0, 19, 0, 1, 0, 0, 0, 5]);
    }
}
