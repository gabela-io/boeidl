//! Golden byte-exact del Modelo 130 con sobre (946 bytes) + round-trip.
//! Cifras reales 1T 2026 (ver ~/.../test/modelo130.test.ts).

use boeidl_example::generated::sobre::{Mod130Aux, Mod130Fichero, Mod130Pagina};

fn make() -> Mod130Fichero {
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

    Mod130Fichero {
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
    let f2 = Mod130Fichero::unmarshal(&b).unwrap();
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
    assert!(Mod130Fichero::unmarshal(&b).is_err());
}
