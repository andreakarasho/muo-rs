//! The buffer and file writers, anchored times (reader/writer part) and the entity persistence type
//! table, plus golden bytes for every primitive of the save format.

use muo_io::serialization::index::TypeTable;
use muo_io::serialization::reader::{DATE_TIME_MAX_TICKS, DATE_TIME_MIN_TICKS, Guid, TypeRef};
use muo_io::{
    GenericReader, GenericWriter, IoError, SaveReader, SaveWriter, SeekOrigin, TextDefinition,
};
use muo_primitives::{Point2D, Point3D, Rectangle2D, Rectangle3D, Serial};

fn writer() -> SaveWriter {
    SaveWriter::new(true)
}

#[test]
fn primitives_are_little_endian_at_expected_offsets() {
    let mut w = writer();
    w.write_u8(0xAB);
    w.write_i8(-5);
    w.write_bool(true);
    w.write_bool(false);
    w.write_i16(-12345);
    w.write_u16(54321);
    w.write_i32(-123456789);
    w.write_u32(3123456789);
    w.write_i64(-1234567890123456789);
    w.write_u64(12345678901234567890);
    w.write_f64(1234.5678);
    w.write_f32(56.75);
    w.write_serial(Serial::new(0x40000001));
    assert_eq!(w.position(), 48);
    let b = w.as_slice();
    assert_eq!(&b[..4], [0xAB, 0xFB, 1, 0]);
    assert_eq!(&b[4..6], (-12345i16).to_le_bytes());
    assert_eq!(&b[6..8], 54321u16.to_le_bytes());
    assert_eq!(&b[44..48], [0x01, 0x00, 0x00, 0x40]);

    let mut r = SaveReader::new(b);
    assert_eq!(r.read_u8().unwrap(), 0xAB);
    assert_eq!(r.read_i8().unwrap(), -5);
    assert!(r.read_bool().unwrap());
    assert!(!r.read_bool().unwrap());
    assert_eq!(r.read_i16().unwrap(), -12345);
    assert_eq!(r.read_u16().unwrap(), 54321);
    assert_eq!(r.read_i32().unwrap(), -123456789);
    assert_eq!(r.read_u32().unwrap(), 3123456789);
    assert_eq!(r.read_i64().unwrap(), -1234567890123456789);
    assert_eq!(r.read_u64().unwrap(), 12345678901234567890);
    assert_eq!(r.read_f64().unwrap(), 1234.5678);
    assert_eq!(r.read_f32().unwrap(), 56.75);
    assert_eq!(r.read_serial().unwrap(), Serial::new(0x40000001));
    assert!(matches!(r.read_u8(), Err(IoError::EndOfStream)));
}

#[test]
fn seek_end_uses_high_water_mark() {
    let mut w = writer();
    w.write_i64(1);
    w.write_i64(2);
    w.write_i64(3);
    w.seek(4, SeekOrigin::Begin).unwrap();
    w.write_i32(99);
    assert_eq!(w.seek(0, SeekOrigin::End).unwrap(), 24);
    assert_eq!(w.position(), 24);
}

#[test]
fn seek_current_and_begin() {
    let mut w = writer();
    w.write_u32(0xDEADBEEF);
    assert_eq!(w.seek(2, SeekOrigin::Begin).unwrap(), 2);
    assert_eq!(w.seek(1, SeekOrigin::Current).unwrap(), 3);
    w.write_u8(0x77);
    assert_eq!(w.as_slice()[3], 0x77);
    assert!(w.seek(10, SeekOrigin::Begin).is_err());
}

#[test]
fn growth_and_span_writes() {
    let mut w = SaveWriter::with_capacity(16, true);
    for i in 0..100i64 {
        w.write_i64(i);
    }
    assert_eq!(w.position(), 800);
    let mut r = SaveReader::new(w.as_slice());
    for i in 0..100i64 {
        assert_eq!(r.read_i64().unwrap(), i);
    }

    let payload: Vec<u8> = (1..=64).collect();
    let mut w = writer();
    w.write_u16(7);
    w.write_bytes(&payload);
    assert_eq!(w.position(), 66);
    assert_eq!(&w.as_slice()[2..], payload);
}

#[test]
fn encoded_int_matches_format() {
    let cases: [(i32, &[u8]); 11] = [
        (0, &[0x00]),
        (127, &[0x7F]),
        (128, &[0x80, 0x01]),
        (0x3FFF, &[0xFF, 0x7F]),
        (0x4000, &[0x80, 0x80, 0x01]),
        (0x1F_FFFF, &[0xFF, 0xFF, 0x7F]),
        (0x20_0000, &[0x80, 0x80, 0x80, 0x01]),
        (0xFFF_FFFF, &[0xFF, 0xFF, 0xFF, 0x7F]),
        (0x1000_0000, &[0x80, 0x80, 0x80, 0x80, 0x01]),
        (i32::MAX, &[0xFF, 0xFF, 0xFF, 0xFF, 0x07]),
        (-1, &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]),
    ];
    for (value, expected) in cases {
        let mut w = writer();
        w.write_encoded_int(value);
        assert_eq!(w.as_slice(), expected, "{value}");
        assert_eq!(SaveReader::new(expected).read_encoded_int().unwrap(), value);
    }
    // a non-minimal encoding reads the same
    assert_eq!(
        SaveReader::new(&[0xE4, 0x00]).read_encoded_int().unwrap(),
        100
    );
}

#[test]
fn prefixed_strings_golden() {
    let text = "h\u{e9}llo \u{3a9}orld";
    let mut w = writer();
    w.write_string(text);
    let utf8 = text.as_bytes();
    assert_eq!(w.as_slice()[0], 1);
    assert_eq!(w.as_slice()[1] as usize, utf8.len());
    assert_eq!(&w.as_slice()[2..], utf8);
    w.write_opt_string(None);
    assert_eq!(*w.as_slice().last().unwrap(), 0);

    let mut r = SaveReader::new(w.as_slice());
    assert_eq!(r.read_string().unwrap().as_deref(), Some(text));
    assert_eq!(r.read_string().unwrap(), None);

    // empty string: flag, length 0
    let mut w = writer();
    w.write_string("");
    assert_eq!(w.as_slice(), [1, 0]);
}

#[test]
fn long_string_prefix_is_zero_padded_to_worst_case_width() {
    // 100 ASCII chars: worst case 300 bytes -> 2 byte prefix, written non-minimally as (100 | 0x80), 0x00
    let value = "a".repeat(100);
    let mut w = SaveWriter::new(false);
    w.write_string_raw(&value);
    assert_eq!(&w.as_slice()[..2], [100 | 0x80, 0]);
    assert_eq!(w.position(), 102);
    assert_eq!(
        SaveReader::new(w.as_slice()).read_string_raw().unwrap(),
        value
    );

    // 42 chars: worst case 126 < 128, canonical single byte
    let mut w = SaveWriter::new(false);
    w.write_string_raw(&"a".repeat(42));
    assert_eq!(w.as_slice()[0], 42);

    // 43 chars: worst case 129, so already padded
    let mut w = SaveWriter::new(false);
    w.write_string_raw(&"a".repeat(43));
    assert_eq!(&w.as_slice()[..2], [43 | 0x80, 0]);
}

#[test]
fn multibyte_strings_round_trip() {
    for chars in [42usize, 100, 10_000] {
        let value = "\u{4e8c}".repeat(chars);
        let mut w = SaveWriter::with_capacity(16, true);
        w.write_string(&value);
        assert_eq!(
            SaveReader::new(w.as_slice())
                .read_string()
                .unwrap()
                .as_deref(),
            Some(value.as_str())
        );
    }
    // surrogate pairs: 2 UTF-16 units encode to 4 bytes, under the 3 bytes per unit reservation
    let value = "\u{1F600}".repeat(60);
    let mut w = SaveWriter::new(false);
    w.write_string_raw(&value);
    assert_eq!(
        SaveReader::new(w.as_slice()).read_string_raw().unwrap(),
        value
    );
}

#[test]
fn unprefixed_reader_reads_raw_strings() {
    let mut w = SaveWriter::new(false);
    w.write_string("abc");
    assert_eq!(w.as_slice(), [3, b'a', b'b', b'c']);
    assert_eq!(
        SaveReader::without_prefixes(w.as_slice())
            .read_string()
            .unwrap()
            .as_deref(),
        Some("abc")
    );
}

#[test]
fn date_time_and_time_span_golden() {
    let mut w = writer();
    w.write_date_time_ticks(0x0102030405060708);
    w.write_time_span_ticks(-1);
    assert_eq!(
        w.as_slice(),
        [
            8, 7, 6, 5, 4, 3, 2, 1, 255, 255, 255, 255, 255, 255, 255, 255
        ]
    );
    let mut r = SaveReader::new(w.as_slice());
    assert_eq!(r.read_date_time_ticks().unwrap(), 0x0102030405060708);
    assert_eq!(r.read_time_span_ticks().unwrap(), -1);

    // an out of range DateTime is rejected, not wrapped
    assert!(
        SaveReader::new(&(-1i64).to_le_bytes())
            .read_date_time_ticks()
            .is_err()
    );
    assert!(
        SaveReader::new(&i64::MAX.to_le_bytes())
            .read_date_time_ticks()
            .is_err()
    );
}

#[test]
fn delta_time_round_trips_with_sentinels() {
    let now = 1_000_000_000_000_000_000i64;
    let mut w = writer();
    w.write_delta_time_ticks(DATE_TIME_MIN_TICKS, now);
    w.write_delta_time_ticks(DATE_TIME_MAX_TICKS, now);
    w.write_delta_time_ticks(now + 500, now);
    assert_eq!(&w.as_slice()[..8], i64::MIN.to_le_bytes());
    assert_eq!(&w.as_slice()[8..16], i64::MAX.to_le_bytes());
    assert_eq!(&w.as_slice()[16..], 500i64.to_le_bytes());
    let mut r = SaveReader::new(w.as_slice());
    assert_eq!(r.read_delta_time_ticks(now).unwrap(), DATE_TIME_MIN_TICKS);
    assert_eq!(r.read_delta_time_ticks(now).unwrap(), DATE_TIME_MAX_TICKS);
    assert_eq!(r.read_delta_time_ticks(now + 7).unwrap(), now + 507);
}

const SAMPLE: i64 = 639_000_000_000_000_000;

fn anchored(shift: i64, value: i64) -> i64 {
    let mut w = writer();
    w.write_anchored_time_ticks(value);
    SaveReader::new(w.as_slice())
        .with_anchored_time_shift_ticks(shift)
        .read_anchored_time_ticks()
        .unwrap()
}

#[test]
fn anchored_time() {
    assert_eq!(anchored(0, SAMPLE), SAMPLE);
    let three_hours = 3 * 3600 * 10_000_000;
    assert_eq!(anchored(three_hours, SAMPLE), SAMPLE + three_hours);
    // sentinels pass through unshifted
    assert_eq!(
        anchored(three_hours, DATE_TIME_MIN_TICKS),
        DATE_TIME_MIN_TICKS
    );
    assert_eq!(
        anchored(three_hours, DATE_TIME_MAX_TICKS),
        DATE_TIME_MAX_TICKS
    );
    // saturates instead of overflowing
    assert_eq!(
        anchored(864_000_000_000, DATE_TIME_MAX_TICKS - 600_000_000),
        DATE_TIME_MAX_TICKS
    );
    assert_eq!(anchored(-SAMPLE * 2, SAMPLE), DATE_TIME_MIN_TICKS);
}

#[test]
fn geometry_golden() {
    let mut w = writer();
    w.write_point3d(Point3D::new(100, -200, 30));
    assert_eq!(
        w.as_slice(),
        [100, 0, 0, 0, 0x38, 0xFF, 0xFF, 0xFF, 30, 0, 0, 0]
    );
    w.write_point2d(Point2D::new(1, 2));
    w.write_rect2d(Rectangle2D::from_points(
        Point2D::new(1, 2),
        Point2D::new(3, 4),
    ));
    w.write_rect3d(Rectangle3D::from_points(
        Point3D::new(1, 2, 3),
        Point3D::new(4, 5, 6),
    ));
    assert_eq!(w.position(), 12 + 8 + 16 + 24);

    let mut r = SaveReader::new(w.as_slice());
    assert_eq!(r.read_point3d().unwrap(), Point3D::new(100, -200, 30));
    assert_eq!(r.read_point2d().unwrap(), Point2D::new(1, 2));
    assert_eq!(
        r.read_rect2d().unwrap(),
        Rectangle2D::from_points(Point2D::new(1, 2), Point2D::new(3, 4))
    );
    assert_eq!(
        r.read_rect3d().unwrap(),
        Rectangle3D::from_points(Point3D::new(1, 2, 3), Point3D::new(4, 5, 6))
    );
}

#[test]
fn type_ref_golden() {
    let mut w = writer();
    w.write_type_hash(None);
    w.write_type_hash(Some(0x0102030405060708));
    assert_eq!(w.as_slice(), [0, 2, 8, 7, 6, 5, 4, 3, 2, 1]);
    let mut r = SaveReader::new(w.as_slice());
    assert_eq!(r.read_type().unwrap(), None);
    assert_eq!(
        r.read_type().unwrap(),
        Some(TypeRef::Hash(0x0102030405060708))
    );

    // legacy flag 1: a raw full name
    let mut r = SaveReader::new(&[1, 3, b'A', b'.', b'B']);
    assert_eq!(r.read_type().unwrap(), Some(TypeRef::Name("A.B".into())));
    assert!(SaveReader::new(&[9]).read_type().unwrap().is_none()); // an unknown flag is null
}

#[test]
fn enums_ip_guid_bits_decimal_text_golden() {
    let mut w = writer();
    w.write_enum(1, 0x7F);
    w.write_enum(2, 0x1234);
    w.write_enum(4, 300);
    w.write_enum(8, -2);
    assert_eq!(
        w.as_slice(),
        [
            0x7F, 0x34, 0x12, 0xAC, 0x02, 0xFE, 255, 255, 255, 255, 255, 255, 255
        ]
    );
    let mut r = SaveReader::new(w.as_slice());
    assert_eq!(r.read_enum(1).unwrap(), 0x7F);
    assert_eq!(r.read_enum(2).unwrap(), 0x1234);
    assert_eq!(r.read_enum(4).unwrap(), 300);
    assert_eq!(r.read_enum(8).unwrap(), -2);
    assert!(r.read_enum(3).is_err());

    let mut w = writer();
    w.write_ip_address("127.0.0.1".parse().unwrap());
    w.write_ip_address("::1".parse().unwrap());
    assert_eq!(&w.as_slice()[..5], [4, 127, 0, 0, 1]);
    assert_eq!(w.as_slice()[5], 16);
    let mut r = SaveReader::new(w.as_slice());
    assert_eq!(r.read_ip_address().unwrap().to_string(), "127.0.0.1");
    assert_eq!(r.read_ip_address().unwrap().to_string(), "::1");
    assert!(
        SaveReader::new(&[5, 1, 2, 3, 4, 5])
            .read_ip_address()
            .is_err()
    );

    // the first three Guid fields are laid out little endian
    let guid = Guid([
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
        0xFF,
    ]);
    let mut w = writer();
    w.write_guid(guid);
    assert_eq!(
        w.as_slice(),
        [
            0x33, 0x22, 0x11, 0x00, 0x55, 0x44, 0x77, 0x66, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD,
            0xEE, 0xFF
        ]
    );
    assert_eq!(SaveReader::new(w.as_slice()).read_guid().unwrap(), guid);
    assert_eq!(guid.to_string(), "00112233-4455-6677-8899-aabbccddeeff");

    // bit 0 is the least significant bit of byte 0
    let bits = [true, false, true, true, false, false, false, false, true];
    let mut w = writer();
    w.write_bit_array(&bits);
    assert_eq!(w.as_slice(), [9, 0b0000_1101, 0b0000_0001]);
    assert_eq!(
        SaveReader::new(w.as_slice()).read_bit_array().unwrap(),
        bits
    );
    assert!(
        SaveReader::new(&[0xFF, 0xFF, 0xFF, 0xFF, 0x07])
            .read_bit_array()
            .is_err()
    );

    let mut w = writer();
    w.write_decimal_bits([1, 2, 3, 4]);
    assert_eq!(w.position(), 16);
    assert_eq!(
        SaveReader::new(w.as_slice()).read_decimal_bits().unwrap(),
        [1, 2, 3, 4]
    );

    let mut w = writer();
    w.write_text_definition(None);
    w.write_text_definition(Some(&TextDefinition::of_number(10345)));
    w.write_text_definition(Some(&TextDefinition::of_text("Hi")));
    w.write_text_definition(Some(&TextDefinition::EMPTY));
    assert_eq!(w.as_slice(), [3, 1, 0xE9, 0x50, 2, 1, 2, b'H', b'i', 0]);
    let mut r = SaveReader::new(w.as_slice());
    assert_eq!(r.read_text_definition().unwrap(), None);
    assert_eq!(
        r.read_text_definition().unwrap(),
        Some(TextDefinition::of_number(10345))
    );
    assert_eq!(
        r.read_text_definition().unwrap(),
        Some(TextDefinition::of_text("Hi"))
    );
    assert_eq!(
        r.read_text_definition().unwrap(),
        Some(TextDefinition::EMPTY)
    );
}

#[test]
fn map_and_race_index() {
    let mut w = writer();
    w.write_map_index(Some(1));
    w.write_map_index(None);
    w.write_race_index(Some(2));
    assert_eq!(w.as_slice(), [1, 0xFF, 2]);
    let mut r = SaveReader::new(w.as_slice());
    assert_eq!(r.read_map_index().unwrap(), Some(1));
    assert_eq!(r.read_map_index().unwrap(), None);
    assert_eq!(r.read_race_index().unwrap(), Some(2));
}

#[test]
fn reader_seek_and_read() {
    let data = [1, 2, 3, 4];
    let mut r = SaveReader::new(&data);
    assert_eq!(r.seek(-1, SeekOrigin::End).unwrap(), 3);
    assert_eq!(r.read_u8().unwrap(), 4);
    assert_eq!(r.seek(-5, SeekOrigin::Current).unwrap(), 0); // clamped to 0
    assert!(r.seek(5, SeekOrigin::Begin).is_err());
    let mut buf = [0u8; 5];
    assert!(matches!(r.read(&mut buf), Err(IoError::EndOfStream)));
    assert_eq!(r.read(&mut buf[..2]).unwrap(), 2);
    assert_eq!(r.position(), 2);
}

#[test]
fn bad_data_is_an_error_not_a_panic() {
    // string claiming more bytes than exist
    assert!(SaveReader::new(&[1, 0x7F, b'a']).read_string().is_err());
    // a negative length reads as empty
    assert_eq!(
        SaveReader::new(&[1, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F])
            .read_string()
            .unwrap()
            .as_deref(),
        Some("")
    );
    // an unterminated 7-bit int
    assert!(SaveReader::new(&[0x80, 0x80]).read_encoded_int().is_err());
    assert!(SaveReader::new(&[]).read_string().is_err());
}

#[test]
fn file_writer_patches_backwards() {
    let dir = std::env::temp_dir().join(format!("muo_io_fbw_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test.idx");

    let mut w = SaveWriter::with_capacity(64, false);
    w.write_i32(3);
    let count_position = w.position();
    w.write_i32(0);
    for i in 0..1000u64 {
        w.write_u64(i.wrapping_mul(0x9E3779B97F4A7C15));
        w.write_i32(i as i32);
    }
    let end = w.position();
    w.seek(count_position as i64, SeekOrigin::Begin).unwrap();
    w.write_i32(1000);
    w.seek(0, SeekOrigin::End).unwrap();
    assert_eq!(w.position(), end);
    w.write_to_file(&path).unwrap();

    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(bytes.len(), 4 + 4 + 1000 * 12);
    let mut r = SaveReader::new(&bytes);
    assert_eq!(r.read_i32().unwrap(), 3);
    assert_eq!(r.read_i32().unwrap(), 1000);
    for i in 0..1000u64 {
        assert_eq!(r.read_u64().unwrap(), i.wrapping_mul(0x9E3779B97F4A7C15));
        assert_eq!(r.read_i32().unwrap(), i as i32);
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn type_table_assigns_stable_insertion_ordered_indexes() {
    let mut t = TypeTable::new();
    assert_eq!(t.register("RoundTripEntity").unwrap(), 0);
    assert_eq!(t.register("System.String").unwrap(), 1);
    assert_eq!(t.register("RoundTripEntity").unwrap(), 0); // duplicate is a no-op
    assert_eq!(t.names(), ["RoundTripEntity", "System.String"]);
    assert_eq!(t.index_of("System.String"), Some(1));
    assert_eq!(t.index_of("System.Int32"), None);

    for i in 2..=u16::MAX {
        t.register(&format!("T{i}")).unwrap();
    }
    assert!(t.register("OneTooMany").is_err());
}
