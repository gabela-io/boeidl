//! Byte-level helpers for ISO-8859-1 fixed-position records.
//!
//! Generated marshal/unmarshal code calls these. All positions are 1-indexed
//! to match the `.boe` DSL.

use super::errors::AeatError;

/// Write `s` into `buf` starting at 1-indexed position `at`, encoding each
/// char as its ISO-8859-1 byte (`c as u32` must be < 256).
pub fn write_field(
    buf: &mut [u8],
    at: usize,
    width: usize,
    s: &str,
    field: &str,
) -> Result<(), AeatError> {
    let start = at - 1;
    for (n, c) in s.chars().enumerate() {
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

/// Parse an unsigned amount field from the buffer. All chars are digits
/// (possibly with leading zeros/spaces).
pub fn parse_unsigned_amount(data: &[u8], at: usize, len: usize) -> Result<i64, AeatError> {
    let start = at - 1;
    let end = start + len;
    if end > data.len() {
        return Err(AeatError::ShortRecord {
            expected: end,
            got: data.len(),
        });
    }
    let slice = &data[start..end];
    let s = std::str::from_utf8(slice).map_err(|_| AeatError::InvalidEncoding)?;
    let trimmed = s.trim_start_matches(['0', ' ']);
    if trimmed.is_empty() {
        return Ok(0);
    }
    trimmed.parse::<i64>().map_err(|_| AeatError::InvalidValue {
        field: format!("parse_unsigned_amount@{at}"),
        value: s.to_string(),
    })
}

/// Append `s` to `buf`, one ISO-8859-1 byte per char (each `c as u32` < 256).
/// Used by the envelope marshaler for header/trailer templates (sequential,
/// unlike `write_field`'s fixed positions).
pub fn append_latin1(buf: &mut Vec<u8>, s: &str) -> Result<(), AeatError> {
    for c in s.chars() {
        let cp = c as u32;
        if cp >= 256 {
            return Err(AeatError::InvalidValue {
                field: "<template>".to_string(),
                value: s.to_string(),
            });
        }
        buf.push(cp as u8);
    }
    Ok(())
}

/// Verify that the fixed literal `expected` sits at 0-indexed byte offset `at`
/// in `data`. `context` names the segment for the error message.
pub fn verify_literal(
    data: &[u8],
    at: usize,
    expected: &str,
    context: &str,
) -> Result<(), AeatError> {
    let want: Vec<u8> = expected.chars().map(|c| c as u8).collect();
    let end = at + want.len();
    if data.len() < end {
        return Err(AeatError::ShortRecord {
            expected: end,
            got: data.len(),
        });
    }
    if data[at..end] != want[..] {
        let got: String = data[at..end].iter().map(|&b| char::from(b)).collect();
        return Err(AeatError::InvalidDelimiter {
            context: context.to_string(),
            expected: expected.to_string(),
            got,
        });
    }
    Ok(())
}

/// Parse an envelope-style (`numN`) signed amount: `'N'`-prefixed → negative,
/// otherwise all digits → positive. Mirror of `encode_signed_amount_n`.
pub fn parse_signed_amount_n(buf: &[u8], at: usize, width: usize) -> Result<i64, AeatError> {
    let s = read_field(buf, at, width);
    let (sign, rest): (i64, String) = if s.starts_with('N') {
        (-1, s.chars().skip(1).collect())
    } else {
        (1, s.clone())
    };
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

    #[test]
    fn parse_unsigned_basic() {
        assert_eq!(
            parse_unsigned_amount(b"00000000001000000", 1, 17).unwrap(),
            1_000_000
        );
    }

    #[test]
    fn append_latin1_pushes_bytes() {
        let mut buf = Vec::new();
        append_latin1(&mut buf, "<AUX>").unwrap();
        assert_eq!(buf, b"<AUX>");
        append_latin1(&mut buf, "PEÑA").unwrap();
        assert_eq!(buf[5], b'P');
        assert_eq!(buf[7], 0xD1); // Ñ
    }

    #[test]
    fn append_latin1_rejects_non_latin1() {
        let mut buf = Vec::new();
        assert!(append_latin1(&mut buf, "€").is_err());
    }

    #[test]
    fn verify_literal_ok_and_mismatch() {
        let data = b"<T13002026";
        verify_literal(data, 0, "<T1300", "header").unwrap();
        let err = verify_literal(data, 0, "<AUX>", "header").unwrap_err();
        matches!(err, AeatError::InvalidDelimiter { .. });
    }

    #[test]
    fn verify_literal_short_data() {
        let err = verify_literal(b"<T", 0, "<T1300", "header").unwrap_err();
        matches!(err, AeatError::ShortRecord { .. });
    }

    #[test]
    fn parse_signed_n_roundtrip() {
        let mut buf = vec![b' '; 20];
        write_field(&mut buf, 1, 17, "00000000001128041", "x").unwrap();
        assert_eq!(parse_signed_amount_n(&buf, 1, 17).unwrap(), 1_128_041);
        let mut buf = vec![b' '; 20];
        write_field(&mut buf, 1, 17, "N0000000000018695", "x").unwrap();
        assert_eq!(parse_signed_amount_n(&buf, 1, 17).unwrap(), -18_695);
    }
}
