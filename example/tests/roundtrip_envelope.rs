//! Golden byte-exact del Modelo 130 con envelope (946 bytes) + round-trip.
//! Cifras reales 1T 2026 (ver ~/.../test/modelo130.test.ts).

// Field-by-field assignment reads clearer than a struct literal for these
// many-field fixtures; the default()-then-assign pattern is deliberate.
#![allow(clippy::field_reassign_with_default)]

use boeidl_example::generated::envelope::{Mod130Aux, Mod130Envelope, Mod130Pagina};

fn make() -> Mod130Envelope {
    let mut pag = Mod130Pagina::default();
    pag.tipo_declaracion = "I".to_string();
    pag.nif = "09030055W".to_string();
    pag.apellidos = "BORRERO GONZALEZ".to_string();
    pag.nombre = "ALDO ISMAEL".to_string();
    pag.ejercicio = "2026".to_string();
    pag.periodo = "1T".to_string();
    pag.c01_ingresos = 5_880_919; // 58809.19
    pag.c02_gastos = 240_715; //     2407.15
    pag.compute_derived(); // [03]=5640204, [04]=1128041, [07]=[12]=[14]=[17]=[19]=1128041

    Mod130Envelope {
        ejercicio: "2026".to_string(),
        periodo: "1T".to_string(),
        aux: Mod130Aux::default(),
        pagina: pag,
    }
}

#[test]
fn file_is_946_bytes() {
    assert_eq!(make().marshal().unwrap().len(), 946);
}

#[test]
fn envelope_header_and_trailer() {
    let b = make().marshal().unwrap();
    assert_eq!(&b[0..17], b"<T130020261T0000>");
    assert_eq!(&b[17..22], b"<AUX>");
    assert_eq!(&b[322..328], b"</AUX>");
    assert_eq!(&b[928..946], b"</T130020261T0000>");
}

#[test]
fn page_tag_and_identity() {
    let b = make().marshal().unwrap();
    let at = |c: usize| 328 + c - 1; // pos página 1-indexada → offset fichero
    assert_eq!(&b[328..339], b"<T13001000>"); // apertura página (offset 328)
    assert_eq!(&b[at(14)..at(14) + 9], b"09030055W"); // NIF pos 14
    assert_eq!(&b[at(103)..at(103) + 4], b"2026"); // ejercicio pos 103
    assert_eq!(&b[at(589)..at(589) + 12], b"</T13001000>"); // cierre página pos 589
}

#[test]
fn casillas_encoding() {
    let b = make().marshal().unwrap();
    let at = |c: usize| 328 + c - 1; // pos página 1-indexada → offset fichero
    assert_eq!(&b[at(109)..at(109) + 17], b"00000000005880919"); // [01] num
    assert_eq!(&b[at(143)..at(143) + 17], b"00000000005640204"); // [03] numN +
    assert_eq!(&b[at(211)..at(211) + 17], b"00000000001128041"); // [07]
    assert_eq!(&b[at(415)..at(415) + 17], b"00000000001128041"); // [19] = [07]
}

#[test]
fn round_trip() {
    let f = make();
    let b = f.marshal().unwrap();
    let f2 = Mod130Envelope::unmarshal(&b).unwrap();
    assert_eq!(f2.ejercicio, "2026");
    assert_eq!(f2.periodo, "1T");
    assert_eq!(f2.pagina.nif, "09030055W");
    assert_eq!(f2.pagina.c01_ingresos, 5_880_919);
    assert_eq!(f2.pagina.c19_resultado, 1_128_041);
    assert_eq!(f2.marshal().unwrap(), b); // re-serialize idéntico
}

#[test]
fn unmarshal_rejects_bad_header() {
    let mut b = make().marshal().unwrap();
    b[0] = b'X'; // rompe "<T1300…"
    assert!(Mod130Envelope::unmarshal(&b).is_err());
}

#[test]
fn unmarshal_rejects_corrupt_inner_tag() {
    // El tag interno de página "<T13001000>" empieza en el offset 328
    // (header 17 + aux 311). Un byte corrupto ahí debe fallar, no "curarse".
    let mut b = make().marshal().unwrap();
    assert_eq!(&b[328..339], b"<T13001000>");
    b[328] = b'X';
    assert!(Mod130Envelope::unmarshal(&b).is_err());
}

#[test]
fn unmarshal_rejects_divergent_trailer() {
    // El ejercicio del trailer (offset 935..939) debe coincidir con el del
    // header; si diverge, unmarshal falla en vez de dejar ganar al trailer.
    let mut b = make().marshal().unwrap();
    assert_eq!(&b[935..939], b"2026");
    b[935] = b'3'; // "2026" -> "3026"
    assert!(Mod130Envelope::unmarshal(&b).is_err());
}

#[test]
fn unmarshal_rejects_short_buffer_without_panic() {
    // Buffers demasiado cortos deben devolver Err, NO hacer panic (el read_field
    // del header hacía slice sin comprobar límites). Cubre el rango 6..17 y otros.
    let b = make().marshal().unwrap();
    for len in [0usize, 6, 9, 17, 328, 945] {
        assert!(
            Mod130Envelope::unmarshal(&b[..len]).is_err(),
            "len {len} debería ser Err"
        );
    }
}

#[test]
fn unmarshal_rejects_trailing_data() {
    // Un fichero de 946 bytes válido + basura al final debe rechazarse.
    let mut b = make().marshal().unwrap();
    b.extend_from_slice(&[b'Z'; 32]);
    assert!(Mod130Envelope::unmarshal(&b).is_err());
}
