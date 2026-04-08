//! Encoding helpers used by generated marshal code.

use super::errors::AeatError;

/// Left-pad `s` to `width` with `ch`. If `s` is already at or over width,
/// return it unchanged (caller is responsible for width checks).
pub fn pad_left(s: &str, width: usize, ch: char) -> String {
    let len = s.chars().count();
    if len >= width {
        return s.to_string();
    }
    let mut out = String::with_capacity(width);
    for _ in 0..(width - len) {
        out.push(ch);
    }
    out.push_str(s);
    out
}

/// Right-pad `s` to `width` with `ch`.
pub fn pad_right(s: &str, width: usize, ch: char) -> String {
    let len = s.chars().count();
    if len >= width {
        return s.to_string();
    }
    let mut out = String::with_capacity(width);
    out.push_str(s);
    for _ in 0..(width - len) {
        out.push(ch);
    }
    out
}

/// Encode a non-negative integer as a zero-padded numeric field of `width` chars.
/// Returns `FieldOverflow` if the number does not fit.
pub fn encode_number(field: &str, value: u64, width: usize) -> Result<String, AeatError> {
    let s = value.to_string();
    if s.chars().count() > width {
        return Err(AeatError::FieldOverflow {
            field: field.to_string(),
            width,
            got: s.chars().count(),
        });
    }
    Ok(pad_left(&s, width, '0'))
}

/// Encode a signed amount in AEAT BOE format:
/// - first position: `' '` if `value >= 0`, `'N'` if negative
/// - remaining `width - 1` positions: absolute value zero-padded
/// - if `decimals > 0`, they occupy the last `decimals` positions (no separator)
///
/// `value` is the integer representation including decimal places
/// (e.g. 123.45 with decimals=2 is encoded as `value = 12345`).
pub fn encode_signed_amount(
    field: &str,
    value: i64,
    width: usize,
    decimals: usize,
) -> Result<String, AeatError> {
    if width == 0 {
        return Err(AeatError::FieldOverflow {
            field: field.to_string(),
            width,
            got: 1,
        });
    }
    let sign = if value < 0 { 'N' } else { ' ' };
    let abs = value.unsigned_abs();
    let digits = abs.to_string();
    let digits_width = width - 1;
    if digits.chars().count() > digits_width {
        return Err(AeatError::FieldOverflow {
            field: field.to_string(),
            width,
            got: digits.chars().count() + 1,
        });
    }
    // Decimals are part of the digit run; padding happens to the whole run.
    let _ = decimals;
    let padded = pad_left(&digits, digits_width, '0');
    let mut out = String::with_capacity(width);
    out.push(sign);
    out.push_str(&padded);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pad_left_zero() {
        assert_eq!(pad_left("42", 5, '0'), "00042");
        assert_eq!(pad_left("12345", 5, '0'), "12345");
        assert_eq!(pad_left("123456", 5, '0'), "123456"); // no truncation
    }

    #[test]
    fn pad_right_space() {
        assert_eq!(pad_right("HI", 5, ' '), "HI   ");
    }

    #[test]
    fn encode_number_pads() {
        assert_eq!(encode_number("x", 130, 3).unwrap(), "130");
        assert_eq!(encode_number("x", 1, 5).unwrap(), "00001");
    }

    #[test]
    fn encode_number_overflows() {
        let err = encode_number("x", 1000, 3).unwrap_err();
        match err {
            AeatError::FieldOverflow {
                width: 3, got: 4, ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn encode_signed_positive() {
        // 13 chars total: sign + 12 digits. Value 12345 (= 123.45 with 2 decimals).
        let s = encode_signed_amount("x", 12345, 13, 2).unwrap();
        assert_eq!(s, " 000000012345");
        assert_eq!(s.chars().count(), 13);
    }

    #[test]
    fn encode_signed_negative() {
        let s = encode_signed_amount("x", -12345, 13, 2).unwrap();
        assert_eq!(s, "N000000012345");
    }

    #[test]
    fn encode_signed_zero() {
        let s = encode_signed_amount("x", 0, 13, 2).unwrap();
        assert_eq!(s, " 000000000000");
    }

    #[test]
    fn encode_signed_overflow() {
        // 5 chars total → 4 digits max → 99999 overflows.
        let err = encode_signed_amount("x", 99999, 5, 0).unwrap_err();
        match err {
            AeatError::FieldOverflow {
                width: 5, got: 6, ..
            } => {}
            other => panic!("unexpected: {other:?}"),
        }
    }
}
