use boeidl::ast::{Encoding, LineEnding};
use boeidl::parse;
use pretty_assertions::assert_eq;

#[test]
fn parses_mod130_model_block() {
    let src = r#"
# Modelo 130 — Pago fraccionado IRPF
model "130" version "2015-v11" {
    encoding = "ISO-8859-1"
    line_ending = "CRLF"
    record_length = 878
}
"#;

    let file = parse(src).expect("should parse");
    assert_eq!(file.model.number, "130");
    assert_eq!(file.model.version, "2015-v11");
    assert_eq!(file.model.encoding, Encoding::Iso8859_1);
    assert_eq!(file.model.line_ending, LineEnding::Crlf);
    assert_eq!(file.model.record_length, 878);
}

#[test]
fn rejects_missing_record_length() {
    let src = r#"
model "130" version "2015-v11" {
    encoding = "ISO-8859-1"
    line_ending = "CRLF"
}
"#;
    let err = parse(src).unwrap_err();
    assert!(
        err.to_string().contains("record_length"),
        "unexpected error: {err}"
    );
}

#[test]
fn rejects_unknown_encoding() {
    let src = r#"
model "130" version "2015-v11" {
    encoding = "UTF-8"
    line_ending = "CRLF"
    record_length = 878
}
"#;
    let err = parse(src).unwrap_err();
    assert!(err.to_string().contains("encoding"), "unexpected error: {err}");
}
