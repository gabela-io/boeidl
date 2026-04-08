use boeidl::parse;
use boeidl::validator::{validate, DiagLevel};

const MOD130: &str = include_str!("../../../models/mod130.boe");

fn errors_of(src: &str) -> Vec<String> {
    let file = parse(src).expect("parse");
    validate(&file)
        .into_iter()
        .filter(|d| d.level == DiagLevel::Error)
        .map(|d| d.message)
        .collect()
}

#[test]
fn mod130_has_no_errors() {
    let errors = errors_of(MOD130);
    assert!(
        errors.is_empty(),
        "unexpected errors in mod130.boe: {errors:#?}"
    );
}

#[test]
fn detects_overlap() {
    let src = r#"
model "X" version "1" { encoding = "ISO-8859-1" line_ending = "CRLF" record_length = 10 }
field a { at = 1 length = 5 type = alphanumeric }
field b { at = 3 length = 5 type = alphanumeric }
"#;
    let errors = errors_of(src);
    assert!(
        errors.iter().any(|m| m.contains("overlaps")),
        "expected overlap error, got: {errors:?}"
    );
}

#[test]
fn detects_exceeds_record_length() {
    let src = r#"
model "X" version "1" { encoding = "ISO-8859-1" line_ending = "CRLF" record_length = 10 }
field a { at = 1 length = 15 type = alphanumeric }
"#;
    let errors = errors_of(src);
    assert!(errors.iter().any(|m| m.contains("exceeds record_length")));
}

#[test]
fn detects_duplicate_field() {
    let src = r#"
model "X" version "1" { encoding = "ISO-8859-1" line_ending = "CRLF" record_length = 10 }
field a { at = 1 length = 5 type = alphanumeric }
field a { at = 6 length = 5 type = alphanumeric }
"#;
    let errors = errors_of(src);
    assert!(errors.iter().any(|m| m.contains("duplicate field name")));
}

#[test]
fn detects_fixed_length_mismatch() {
    let src = r#"
model "X" version "1" { encoding = "ISO-8859-1" line_ending = "CRLF" record_length = 10 }
field a { at = 1 length = 3 type = number fixed = "12" }
"#;
    let errors = errors_of(src);
    assert!(errors.iter().any(|m| m.contains("fixed value length")));
}

#[test]
fn detects_unknown_derive_field() {
    let src = r#"
model "X" version "1" { encoding = "ISO-8859-1" line_ending = "CRLF" record_length = 30 }
field a { at = 1 length = 10 type = signed_amount }
field b { at = 11 length = 10 type = signed_amount }
field c { at = 21 length = 10 type = signed_amount }
derive c = a - nonexistent
"#;
    let errors = errors_of(src);
    assert!(
        errors
            .iter()
            .any(|m| m.contains("unknown field `nonexistent`")),
        "got: {errors:?}"
    );
}

#[test]
fn detects_forward_reference_in_derive() {
    // b is declared after a, so `derive a = b` is a forward reference.
    let src = r#"
model "X" version "1" { encoding = "ISO-8859-1" line_ending = "CRLF" record_length = 20 }
field a { at = 1 length = 10 type = signed_amount }
field b { at = 11 length = 10 type = signed_amount }
derive a = b
"#;
    let errors = errors_of(src);
    assert!(
        errors.iter().any(|m| m.contains("forward reference")),
        "got: {errors:?}"
    );
}

#[test]
fn detects_unknown_check_field() {
    let src = r#"
model "X" version "1" { encoding = "ISO-8859-1" line_ending = "CRLF" record_length = 10 }
field a { at = 1 length = 10 type = number }
check E001 {
    rule = a == missing
    severity = error
    message = "x"
}
"#;
    let errors = errors_of(src);
    assert!(
        errors.iter().any(|m| m.contains("unknown field `missing`")),
        "got: {errors:?}"
    );
}

#[test]
fn decimals_only_on_signed_amount() {
    let src = r#"
model "X" version "1" { encoding = "ISO-8859-1" line_ending = "CRLF" record_length = 10 }
field a { at = 1 length = 10 type = number decimals = 2 }
"#;
    let errors = errors_of(src);
    assert!(errors.iter().any(|m| m.contains("decimals")));
}
