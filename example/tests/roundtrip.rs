//! End-to-end test: compile mod130.boe → compile the generated Rust →
//! populate a struct → marshal → unmarshal → assert round-trip.

// Field-by-field assignment reads clearer than a struct literal for these
// many-field fixtures; the default()-then-assign pattern is deliberate.
#![allow(clippy::field_reassign_with_default)]

use boeidl_example::generated::{Mod130, MODEL_NUMBER, MODEL_VERSION, RECORD_LENGTH};

fn make_valid() -> Mod130 {
    let mut m = Mod130::default();
    m.tipo_declaracion = "I".to_string();
    m.nif = "12345678Z".to_string();
    m.comienzo_apellido = "GARC".to_string();
    m.apellidos = "GARCIA LOPEZ".to_string();
    m.nombre = "JUAN".to_string();
    m.ejercicio = "2025".to_string();
    m.periodo = "1T".to_string();
    m.c01_ingresos = 1_000_000;
    m.c02_gastos = 300_000;
    m.persona_contacto = "JUAN GARCIA".to_string();
    m.telefono = "600000000".to_string();
    m.compute_derived();
    m
}

#[test]
fn constants_match_spec() {
    assert_eq!(MODEL_NUMBER, "130");
    assert_eq!(MODEL_VERSION, "2015-v11");
    assert_eq!(RECORD_LENGTH, 878);
}

#[test]
fn marshal_produces_record_of_expected_length() {
    let m = make_valid();
    let bytes = m.marshal().expect("marshal");
    assert_eq!(bytes.len(), RECORD_LENGTH);
}

#[test]
fn marshal_writes_fixed_modelo_at_position_1() {
    let m = make_valid();
    let bytes = m.marshal().expect("marshal");
    assert_eq!(&bytes[0..3], b"130");
}

#[test]
fn marshal_writes_nif_at_position_13() {
    let m = make_valid();
    let bytes = m.marshal().expect("marshal");
    assert_eq!(&bytes[12..21], b"12345678Z");
}

#[test]
fn derive_computes_rendimiento() {
    let m = make_valid();
    // c03 = c01 - c02 = 700000
    assert_eq!(m.c03_rendimiento_neto, 700_000);
    // c04 = max(c03, 0) * 20 / 100 = 140000
    assert_eq!(m.c04_20pct, 140_000);
}

#[test]
fn signed_amount_layout_positive() {
    let m = make_valid();
    let bytes = m.marshal().expect("marshal");
    // c01_ingresos at 77..90 (0-indexed 76..89), value 1000000, decimals 2
    // Expected: ' ' + 12 digits = " 000001000000"
    let s = std::str::from_utf8(&bytes[76..89]).unwrap();
    assert_eq!(s, " 000001000000");
}

#[test]
fn signed_amount_layout_negative() {
    let mut m = make_valid();
    m.c02_gastos = 2_000_000; // forces negative c03
    m.compute_derived();
    assert_eq!(m.c03_rendimiento_neto, -1_000_000);
    let bytes = m.marshal().unwrap();
    // c03 at 103..116 (0-indexed 102..115)
    let s = std::str::from_utf8(&bytes[102..115]).unwrap();
    assert_eq!(s, "N000001000000");
}

#[test]
fn round_trip_preserves_amounts() {
    let m = make_valid();
    let bytes = m.marshal().unwrap();
    let m2 = Mod130::unmarshal(&bytes).expect("unmarshal");
    assert_eq!(m2.c01_ingresos, m.c01_ingresos);
    assert_eq!(m2.c02_gastos, m.c02_gastos);
    assert_eq!(m2.c03_rendimiento_neto, m.c03_rendimiento_neto);
    assert_eq!(m2.c04_20pct, m.c04_20pct);
    assert_eq!(m2.nif, m.nif);
    assert_eq!(m2.ejercicio, m.ejercicio);
    assert_eq!(m2.periodo, m.periodo);
}

#[test]
fn unmarshal_rejects_short_buffer() {
    let err = Mod130::unmarshal(b"too short").unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("too short"), "got: {msg}");
}

#[test]
fn validate_passes_on_consistent_record() {
    let m = make_valid();
    let diags = m.validate();
    // E301, E302, E303 should all pass. W001 may warn only if tipo='N' and c19>0.
    assert!(diags.is_empty(), "expected no diagnostics, got: {diags:?}");
}

#[test]
fn validate_detects_broken_rendimiento() {
    let mut m = make_valid();
    m.c03_rendimiento_neto = 999; // inconsistent with c01 - c02
    let diags = m.validate();
    assert!(
        diags.iter().any(|d| d.code == "E301"),
        "expected E301, got: {diags:?}"
    );
}
