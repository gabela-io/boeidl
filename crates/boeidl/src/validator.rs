//! Semantic validation of a parsed `BoeFile`.
//!
//! Checks (all reported as diagnostics; caller decides what to do):
//! - duplicate field names
//! - `at` / `length` fit inside `record_length`
//! - `fixed` value byte-length matches field length
//! - field positions don't overlap
//! - gaps between fields (warning)
//! - `derive` target must be a declared field, only references previously-declared fields
//! - `check.rule` references only declared fields

use std::collections::{HashMap, HashSet};

use crate::ast::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: DiagLevel,
    pub message: String,
}

impl Diagnostic {
    fn error(msg: impl Into<String>) -> Self {
        Self {
            level: DiagLevel::Error,
            message: msg.into(),
        }
    }
    fn warning(msg: impl Into<String>) -> Self {
        Self {
            level: DiagLevel::Warning,
            message: msg.into(),
        }
    }
}

pub fn validate(file: &BoeFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // ── fields ──────────────────────────────────────────────────────────
    // Track declaration order for forward-reference check in `derive`.
    let mut order: HashMap<&str, usize> = HashMap::new();
    let mut seen_names: HashSet<&str> = HashSet::new();
    for (i, f) in file.fields.iter().enumerate() {
        if !seen_names.insert(f.name.as_str()) {
            diags.push(Diagnostic::error(format!(
                "duplicate field name `{}`",
                f.name
            )));
        }
        order.insert(f.name.as_str(), i);

        // at/length sanity
        if f.at == 0 {
            diags.push(Diagnostic::error(format!(
                "field `{}`: `at` is 1-indexed, got 0",
                f.name
            )));
        }
        if f.length == 0 {
            diags.push(Diagnostic::error(format!(
                "field `{}`: `length` must be > 0",
                f.name
            )));
        }
        let end = f.at.saturating_add(f.length).saturating_sub(1);
        if end > file.model.record_length {
            diags.push(Diagnostic::error(format!(
                "field `{}`: end position {} exceeds record_length {}",
                f.name, end, file.model.record_length
            )));
        }

        // fixed length vs field length (byte-oriented: ISO-8859-1 is 1 byte/char,
        // but we use .chars().count() here because Ñ/Ç are single source chars
        // and the .boe file is UTF-8; the byte encoding check happens in codegen).
        if let Some(fixed) = &f.fixed {
            let fixed_len = fixed.chars().count();
            if fixed_len != f.length {
                diags.push(Diagnostic::error(format!(
                    "field `{}`: fixed value length {} does not match declared length {}",
                    f.name, fixed_len, f.length
                )));
            }
        }

        // decimals only on signed_amount
        if f.decimals.is_some() && f.ty != FieldType::SignedAmount {
            diags.push(Diagnostic::error(format!(
                "field `{}`: `decimals` only allowed on signed_amount",
                f.name
            )));
        }
    }

    // ── overlaps & gaps (sort by `at`) ──────────────────────────────────
    let mut sorted: Vec<&Field> = file.fields.iter().collect();
    sorted.sort_by_key(|f| f.at);

    let mut cursor: usize = 1;
    for f in &sorted {
        if f.at < cursor {
            diags.push(Diagnostic::error(format!(
                "field `{}` at position {} overlaps previous field (expected >= {})",
                f.name, f.at, cursor
            )));
        } else if f.at > cursor {
            diags.push(Diagnostic::warning(format!(
                "gap between positions {} and {} (before field `{}`)",
                cursor,
                f.at - 1,
                f.name
            )));
        }
        cursor = f.at + f.length;
    }
    // Trailing coverage: cursor should be record_length + 1 after the last field.
    let expected = file.model.record_length + 1;
    if !sorted.is_empty() && cursor < expected {
        diags.push(Diagnostic::warning(format!(
            "trailing gap: positions {}..{} are not covered",
            cursor, file.model.record_length
        )));
    }

    // ── derive ──────────────────────────────────────────────────────────
    for d in &file.derives {
        let Some(&target_idx) = order.get(d.target.as_str()) else {
            diags.push(Diagnostic::error(format!(
                "derive: unknown target field `{}`",
                d.target
            )));
            continue;
        };
        for name in collect_idents(&d.expr) {
            match order.get(name.as_str()) {
                None => diags.push(Diagnostic::error(format!(
                    "derive `{}`: unknown field `{}`",
                    d.target, name
                ))),
                Some(&idx) if idx >= target_idx => diags.push(Diagnostic::error(format!(
                    "derive `{}`: forward reference to `{}` (must be declared before target)",
                    d.target, name
                ))),
                _ => {}
            }
        }
    }

    // ── checks ──────────────────────────────────────────────────────────
    for c in &file.checks {
        for name in collect_bool_idents(&c.rule) {
            if !order.contains_key(name.as_str()) {
                diags.push(Diagnostic::error(format!(
                    "check `{}`: unknown field `{}`",
                    c.code, name
                )));
            }
        }
    }

    diags
}

fn collect_idents(e: &Expr) -> Vec<String> {
    let mut out = Vec::new();
    walk_expr(e, &mut out);
    out
}

fn walk_expr(e: &Expr, out: &mut Vec<String>) {
    match e {
        Expr::Ident(n) => out.push(n.clone()),
        Expr::Int(_) | Expr::Str(_) => {}
        Expr::Bin(a, _, b) => {
            walk_expr(a, out);
            walk_expr(b, out);
        }
        Expr::Call(_, args) => {
            for a in args {
                walk_expr(a, out);
            }
        }
    }
}

fn collect_bool_idents(b: &BoolExpr) -> Vec<String> {
    let mut out = Vec::new();
    walk_bool(b, &mut out);
    out
}

fn walk_bool(b: &BoolExpr, out: &mut Vec<String>) {
    match b {
        BoolExpr::Cmp(a, _, b) => {
            walk_expr(a, out);
            walk_expr(b, out);
        }
        BoolExpr::Implies(l, r) => {
            walk_bool(l, out);
            walk_bool(r, out);
        }
    }
}
