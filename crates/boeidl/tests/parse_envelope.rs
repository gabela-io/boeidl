use boeidl::parse;
use boeidl::validator::{validate, DiagLevel};

fn errors(src: &str) -> Vec<String> {
    let file = parse(src).expect("parse");
    validate(&file)
        .into_iter()
        .filter(|d| d.level == DiagLevel::Error)
        .map(|d| d.message)
        .collect()
}

const OK: &str = r#"
model "130" version "2015-v12" {
    encoding = "ISO-8859-1"
    line_ending = "LF"
    record_length = 600
}
record "pagina" {
    record_length = 4
    field abre { at = 1 length = 4 type = alphanumeric fixed = "AAAA" }
}
envelope {
    param ejercicio { length = 4 type = number }
    param periodo   { length = 2 type = alphanumeric }
    header  = "<H${ejercicio}${periodo}>"
    trailer = "</H${ejercicio}${periodo}>"
    contains = [pagina]
}
"#;

#[test]
fn valid_envelope_has_no_errors() {
    assert!(errors(OK).is_empty(), "{:?}", errors(OK));
}

#[test]
fn contains_unknown_record_is_error() {
    let src = OK.replace("contains = [pagina]", "contains = [nope]");
    assert!(errors(&src).iter().any(|m| m.contains("nope")));
}

#[test]
fn unknown_placeholder_is_error() {
    let src = OK.replace("<H${ejercicio}${periodo}>", "<H${ejercicio}${nope}>");
    assert!(errors(&src).iter().any(|m| m.contains("nope")));
}
