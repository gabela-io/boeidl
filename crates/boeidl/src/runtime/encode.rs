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

/// Encode an unsigned fixed-point amount as `width` ASCII digits, zero-padded.
/// `value` is the integer representation including decimal places. Errors on
/// negative values or overflow. `_decimals` is accepted for symmetry with
/// `encode_signed_amount`.
pub fn encode_unsigned_amount(
    field: &str,
    value: i64,
    width: usize,
    _decimals: usize,
) -> Result<String, AeatError> {
    if value < 0 {
        return Err(AeatError::InvalidValue {
            field: field.to_string(),
            value: value.to_string(),
        });
    }
    let s = value.to_string();
    if s.len() > width {
        return Err(AeatError::FieldOverflow {
            field: field.to_string(),
            width,
            got: s.len(),
        });
    }
    Ok(pad_left(&s, width, '0'))
}

/// Encode a signed amount in the AEAT **envelope** (`numN`) convention:
/// - `value >= 0`: `width` digits, zero-padded, NO sign char.
/// - `value < 0`:  `'N'` + `width - 1` digits, zero-padded.
///
/// Differs from `encode_signed_amount` (flat DR, `' '`-for-positive). `value`
/// is the integer including decimals (12345 == 123.45 with 2 decimals).
pub fn encode_signed_amount_n(
    field: &str,
    value: i64,
    width: usize,
    _decimals: usize,
) -> Result<String, AeatError> {
    if width == 0 {
        return Err(AeatError::FieldOverflow {
            field: field.to_string(),
            width,
            got: 1,
        });
    }
    let digits = value.unsigned_abs().to_string();
    if value < 0 {
        let dw = width - 1;
        if digits.chars().count() > dw {
            return Err(AeatError::FieldOverflow {
                field: field.to_string(),
                width,
                got: digits.chars().count() + 1,
            });
        }
        Ok(format!("N{}", pad_left(&digits, dw, '0')))
    } else {
        if digits.chars().count() > width {
            return Err(AeatError::FieldOverflow {
                field: field.to_string(),
                width,
                got: digits.chars().count(),
            });
        }
        Ok(pad_left(&digits, width, '0'))
    }
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
    fn encode_unsigned_amount_positive() {
        assert_eq!(
            encode_unsigned_amount("x", 1_000_000, 17, 2).unwrap(),
            "00000000001000000"
        );
    }

    #[test]
    fn encode_unsigned_amount_zero() {
        assert_eq!(encode_unsigned_amount("x", 0, 10, 2).unwrap(), "0000000000");
    }

    #[test]
    fn encode_unsigned_amount_rejects_negative() {
        assert!(encode_unsigned_amount("x", -1, 10, 2).is_err());
    }

    #[test]
    fn encode_unsigned_amount_overflow() {
        assert!(encode_unsigned_amount("x", 10_i64.pow(11), 10, 2).is_err());
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

    #[test]
    fn encode_signed_n_positive_is_all_digits() {
        assert_eq!(
            encode_signed_amount_n("x", 1_128_041, 17, 2).unwrap(),
            "00000000001128041"
        );
        assert_eq!(
            encode_signed_amount_n("x", 0, 17, 2).unwrap(),
            "00000000000000000"
        );
    }

    #[test]
    fn encode_signed_n_negative_has_n_prefix() {
        assert_eq!(
            encode_signed_amount_n("x", -18_695, 17, 2).unwrap(),
            "N0000000000018695"
        );
    }

    #[test]
    fn encode_signed_n_overflow() {
        assert!(encode_signed_amount_n("x", 10_i64.pow(17), 17, 2).is_err());
        assert!(encode_signed_amount_n("x", -(10_i64.pow(16)), 17, 2).is_err());
    }
}
