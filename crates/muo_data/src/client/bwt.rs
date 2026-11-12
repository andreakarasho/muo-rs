//! The Burrows-Wheeler + move-to-front scheme of compressed
//! `cliloc.*` files (the 7.0.x client's "Huffman-less" localization format).

use crate::error::{DataError, Result};

/// Largest output accepted: a corrupt count must not allocate gigabytes. Real clilocs are a few MB.
const MAX_OUTPUT: i64 = 1 << 28;

/// `BwtDecompress.Decompress(stream, length)` over the bytes after the 4-byte file header.
/// The final input byte is read but never used.
pub fn decompress(input: &[u8]) -> Result<Vec<u8>> {
    let bad = |why: &str| DataError::invalid("bwt stream", why);
    let first_char = input.first().copied().unwrap_or(0xFF);
    let mut first = first_char;

    let mut table = build_table(first);
    let mut output = vec![0u8; input.len()];
    // One iteration per byte read after the first; the byte read by the last iteration is unused.
    for (i, slot) in output
        .iter_mut()
        .enumerate()
        .take(input.len().saturating_sub(1))
    {
        let mut cv = usize::from(first);
        let value = table[cv];
        while cv > 0 {
            table[cv] = table[cv - 1];
            cv -= 1;
        }
        table[0] = value;
        *slot = value as u8;
        first = input[i + 1];
    }
    internal_decompress(&output).ok_or_else(|| bad("corrupt data"))
}

fn build_table(start: u8) -> Vec<u16> {
    let mut table = Vec::with_capacity(256 * 256);
    let (mut first, mut second) = (start, 0u8);
    for _ in 0..256 * 256 {
        table.push(u16::from(first) | (u16::from(second) << 8));
        first = first.wrapping_add(1);
        if first == 0 {
            second = second.wrapping_add(1);
        }
    }
    table.sort_unstable();
    table
}

fn internal_decompress(input: &[u8]) -> Option<Vec<u8>> {
    if input.len() < 1024 {
        return None;
    }
    let mut symbol_table: [u8; 256] = std::array::from_fn(|i| i as u8);
    let mut partial = [0i32; 256 * 3];
    for (i, c) in input[..1024].as_chunks::<4>().0.iter().enumerate() {
        partial[i] = i32::from_le_bytes(*c);
    }
    let sum: i64 = partial[..256].iter().map(|&v| i64::from(v)).sum();
    if !(0..=MAX_OUTPUT).contains(&sum) {
        return None;
    }
    let sum = sum as usize;

    let mut non_zero_count = 256 - partial[..256].iter().filter(|&&v| v == 0).count() as i32;
    let frequency = frequency(&partial[..256]);

    let at = |index: i64| -> Option<u8> { input.get(usize::try_from(index + 1024).ok()?).copied() };

    let mut m: i64 = 0;
    for &freq in frequency.iter().take(non_zero_count as usize) {
        let freq = usize::from(freq);
        symbol_table[usize::from(at(m)?)] = freq as u8;
        partial[freq + 256] = (m + 1) as i32;
        m += i64::from(partial[freq]);
        partial[freq + 512] = i32::try_from(m).ok()?;
    }

    let mut val = symbol_table[0];
    let mut output = Vec::with_capacity(sum);
    while output.len() < sum {
        let v = usize::from(val);
        output.push(val);
        if partial[v + 256] >= partial[v + 512] {
            let before = non_zero_count;
            non_zero_count -= 1;
            if before > 0 {
                shift_left(&mut symbol_table, non_zero_count as usize);
                val = symbol_table[0];
            }
        } else {
            let idx = usize::from(at(i64::from(partial[v + 256]))?);
            partial[v + 256] += 1;
            if idx != 0 {
                shift_left(&mut symbol_table, idx);
                symbol_table[idx] = val;
                val = symbol_table[0];
            }
        }
    }
    Some(output)
}

/// Symbols by descending count (first index wins ties).
fn frequency(counts: &[i32]) -> [u8; 256] {
    let mut tmp = counts.to_vec();
    let mut out = [0u8; 256];
    for slot in &mut out {
        let mut value = 0i64;
        let mut index = 0usize;
        for (j, &c) in tmp.iter().enumerate() {
            if i64::from(c) > value {
                index = j;
                value = i64::from(c);
            }
        }
        if value == 0 {
            break;
        }
        *slot = index as u8;
        tmp[index] = 0;
    }
    out
}

fn shift_left(table: &mut [u8; 256], max: usize) {
    table.copy_within(1..=max.min(255), 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_or_corrupt_input_is_an_error() {
        assert!(decompress(&[]).is_err());
        assert!(decompress(&[1, 2, 3]).is_err());
        // a header claiming more bytes than there are
        let mut raw = vec![0u8; 1100];
        raw[0] = 0xFF;
        raw[1] = 0xFF;
        raw[2] = 0xFF;
        raw[3] = 0x7F;
        assert!(decompress(&raw).is_err());
    }

    #[test]
    fn table_is_sorted_permutation_start() {
        let t = build_table(5);
        assert_eq!(t.len(), 65536);
        assert!(t.windows(2).all(|w| w[0] <= w[1]));
    }
}
