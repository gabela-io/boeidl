use boeidl::ast::*;
use boeidl::parse;
use pretty_assertions::assert_eq;

const MOD130: &str = include_str!("../../../models/mod130.boe");

#[test]
fn parses_full_mod130() {
    let file = parse(MOD130).expect("mod130.boe should parse");

    // model
    assert_eq!(file.model.number, "130");
    assert_eq!(file.model.version, "2015-v11");
    assert_eq!(file.model.record_length, 878);

    // 46 fields per the DR130e15v11 design
    assert_eq!(file.records[0].fields.len(), 46, "expected 46 fields");

    // spot-check a few
    let by_name = |n: &str| file.records[0].fields.iter().find(|f| f.name == n).unwrap();

    let modelo = by_name("modelo");
    assert_eq!(modelo.at, 1);
    assert_eq!(modelo.length, 3);
    assert_eq!(modelo.ty, FieldType::Number);
    assert_eq!(modelo.fixed.as_deref(), Some("130"));
    assert!(modelo.required);

    let c01 = by_name("c01_ingresos");
    assert_eq!(c01.at, 77);
    assert_eq!(c01.ty, FieldType::SignedAmount);
    assert_eq!(c01.decimals, Some(2));

    let firma_localidad = by_name("firma_localidad");
    assert_eq!(firma_localidad.usage, Usage::PrintOnly);

    let tipo_declaracion = by_name("tipo_declaracion");
    assert_eq!(tipo_declaracion.usage, Usage::InternetOnly);
    assert_eq!(tipo_declaracion.domain.as_deref(), Some("B|G|I|N|U"));

    // 9 derives
    assert_eq!(file.records[0].derives.len(), 9);
    assert_eq!(file.records[0].derives[0].target, "c03_rendimiento_neto");

    // 4 checks (E301, E302, E303, W001)
    assert_eq!(file.records[0].checks.len(), 4);
    let codes: Vec<_> = file.records[0].checks.iter().map(|c| c.code.as_str()).collect();
    assert_eq!(codes, vec!["E301", "E302", "E303", "W001"]);

    // Severity distribution
    assert_eq!(file.records[0].checks[0].severity, Severity::Error);
    assert_eq!(file.records[0].checks[3].severity, Severity::Warning);
}

#[test]
fn implies_rule_shape() {
    let file = parse(MOD130).unwrap();
    let w001 = file.records[0].checks.iter().find(|c| c.code == "W001").unwrap();
    match &w001.rule {
        BoolExpr::Implies(lhs, rhs) => {
            // lhs: tipo_declaracion == "N"
            match lhs.as_ref() {
                BoolExpr::Cmp(Expr::Ident(n), CmpOp::Eq, Expr::Str(s)) => {
                    assert_eq!(n, "tipo_declaracion");
                    assert_eq!(s, "N");
                }
                other => panic!("unexpected lhs: {other:?}"),
            }
            // rhs: c19_resultado <= 0
            match rhs.as_ref() {
                BoolExpr::Cmp(Expr::Ident(n), CmpOp::Le, Expr::Int(0)) => {
                    assert_eq!(n, "c19_resultado");
                }
                other => panic!("unexpected rhs: {other:?}"),
            }
        }
        other => panic!("expected Implies, got {other:?}"),
    }
}

#[test]
fn derive_expr_shape() {
    let file = parse(MOD130).unwrap();
    // c04_20pct = max(c03_rendimiento_neto, 0) * 20 / 100
    let d = file.records[0]
        .derives
        .iter()
        .find(|d| d.target == "c04_20pct")
        .unwrap();
    // Outer should be a Bin(... Div ...) since * and / are left-associative.
    match &d.expr {
        Expr::Bin(_, BinOp::Div, rhs) => match rhs.as_ref() {
            Expr::Int(100) => {}
            other => panic!("expected /100, got {other:?}"),
        },
        other => panic!("expected top-level Div, got {other:?}"),
    }
}
