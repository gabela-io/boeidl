//! Byte-level helpers for ISO-8859-1 fixed-position records.
//!
//! Generated marshal/unmarshal code calls these. All positions are 1-indexed
//! to match the `.boe` DSL.

use super::errors::AeatError;

/// Write `s` into `buf` starting at 1-indexed position `at`, encoding each
/// char as its ISO-8859-1 byte (`c as u32` must be < 256).
pub fn write_field(buf: &mut [u8], at: usize, width: usize, s: &str, field: &str) -> Result<(), AeatError> {
    let start = at - 1;
    let mut n = 0;
    for c in s.chars() {
        if n >= width {
            return Err(AeatError::FieldOverflow {
                field: field.to_string(),
                width,
                got: s.chars().count(),
            });
        }
        let cp = c as u32;
        if cp >= 256 {
            return Err(AeatError::InvalidValue {
                field: field.to_string(),
                value: s.to_string(),
            });
        }
        buf[start + n] = cp as u8;
        n += 1;
    }
    // Any remaining positions are left as the caller initialized them.
    Ok(())
}

/// Read a fixed-position field from `buf` as a `String`, decoding each byte
/// as its ISO-8859-1 code point. Never fails — ISO-8859-1 covers all u8.
pub fn read_field(buf: &[u8], at: usize, width: usize) -> String {
    let start = at - 1;
    buf[start..start + width]
        .iter()
        .map(|&b| char::from(b))
        .collect()
}

/// Parse a signed amount field from the buffer.
/// First char is ' ' (or anything non-'N') for positive, 'N' for negative.
/// Remaining chars are digits (may have leading spaces from unused fields).
pub fn parse_signed_amount(buf: &[u8], at: usize, width: usize) -> Result<i64, AeatError> {
    let s = read_field(buf, at, width);
    let mut chars = s.chars();
    let sign_char = chars.next().unwrap_or(' ');
    let sign: i64 = if sign_char == 'N' { -1 } else { 1 };
    let rest: String = chars.collect();
    let trimmed = rest.trim_start_matches(['0', ' ']);
    let n: i64 = if trimmed.is_empty() {
        0
    } else {
        trimmed.parse().map_err(|_| AeatError::InvalidValue {
            field: format!("<at {at}>"),
            value: s.clone(),
        })?
    };
    Ok(sign * n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_ascii_and_latin1() {
        let mut buf = vec![b' '; 10];
        write_field(&mut buf, 1, 5, "HOLA", "x").unwrap();
        assert_eq!(&buf[0..5], b"HOLA ");

        write_field(&mut buf, 6, 5, "PEÑA", "x").unwrap();
        assert_eq!(buf[5], b'P');
        assert_eq!(buf[6], b'E');
        assert_eq!(buf[7], 0xD1); // Ñ in ISO-8859-1
        assert_eq!(buf[8], b'A');
    }

    #[test]
    fn write_rejects_non_latin1() {
        let mut buf = vec![b' '; 10];
        let err = write_field(&mut buf, 1, 5, "€uro", "x").unwrap_err();
        matches!(err, AeatError::InvalidValue { .. });
    }

    #[test]
    fn read_latin1_roundtrip() {
        let mut buf = vec![b' '; 10];
        write_field(&mut buf, 1, 5, "PEÑA ", "x").unwrap();
        assert_eq!(read_field(&buf, 1, 5), "PEÑA ");
    }

    #[test]
    fn parse_signed_positive_and_negative() {
        let mut buf = vec![b' '; 20];
        write_field(&mut buf, 1, 13, " 000000012345", "x").unwrap();
        assert_eq!(parse_signed_amount(&buf, 1, 13).unwrap(), 12345);

        let mut buf = vec![b' '; 20];
        write_field(&mut buf, 1, 13, "N000000012345", "x").unwrap();
        assert_eq!(parse_signed_amount(&buf, 1, 13).unwrap(), -12345);
    }
}
