use boeidl::parse;
use boeidl::validator::{validate, DiagLevel};

const MOD303: &str = include_str!("../../../models/mod303.boe");

#[test]
fn mod303_has_no_errors() {
    let file = parse(MOD303).expect("parse");
    let diags = validate(&file);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagLevel::Error)
        .collect();
    assert!(errors.is_empty(), "unexpected errors: {errors:#?}");
}

#[test]
fn mod303_has_five_records() {
    let file = parse(MOD303).expect("parse");
    assert_eq!(file.records.len(), 5);
    let names: Vec<&str> = file.records.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, vec!["page1", "page2", "page3", "page4", "page5"]);
}

#[test]
fn mod303_record_lengths() {
    let file = parse(MOD303).expect("parse");
    let lens: Vec<usize> = file.records.iter().map(|r| r.record_length).collect();
    assert_eq!(lens, vec![1464, 1706, 1079, 986, 1523]);
}
