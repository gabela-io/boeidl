//! pest → AST conversion.

use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;

use crate::ast::*;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct BoeParser;

#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

impl<R: pest::RuleType> From<pest::error::Error<R>> for ParseError {
    fn from(e: pest::error::Error<R>) -> Self {
        ParseError(e.to_string())
    }
}

fn err(msg: impl Into<String>) -> ParseError {
    ParseError(msg.into())
}

pub fn parse(src: &str) -> Result<BoeFile, ParseError> {
    let mut pairs = BoeParser::parse(Rule::file, src)?;
    let file = pairs.next().ok_or_else(|| err("empty file"))?;

    let mut model: Option<Model> = None;
    let mut fields = Vec::new();
    let mut derives = Vec::new();
    let mut checks = Vec::new();
    let mut records: Vec<Record> = Vec::new();
    let mut saw_top_level_item = false;

    for pair in file.into_inner() {
        match pair.as_rule() {
            Rule::model_block => model = Some(parse_model(pair)?),
            Rule::field_block => {
                saw_top_level_item = true;
                fields.push(parse_field(pair)?);
            }
            Rule::derive_stmt => {
                saw_top_level_item = true;
                derives.push(parse_derive(pair)?);
            }
            Rule::check_block => {
                saw_top_level_item = true;
                checks.push(parse_check(pair)?);
            }
            Rule::record_block => records.push(parse_record_block(pair)?),
            Rule::EOI => {}
            r => return Err(err(format!("unexpected rule at top level: {r:?}"))),
        }
    }

    let model = model.ok_or_else(|| err("missing `model` block"))?;

    if !records.is_empty() && saw_top_level_item {
        return Err(err(
            "cannot mix top-level fields with record blocks",
        ));
    }

    let records = if !records.is_empty() {
        records
    } else {
        vec![Record {
            name: format!("mod{}", model.number),
            record_length: model.record_length,
            fields,
            derives,
            checks,
        }]
    };
    Ok(BoeFile { model, records })
}

fn parse_record_block(pair: Pair<Rule>) -> Result<Record, ParseError> {
    let mut inner = pair.into_inner();
    let name = unquote(
        inner
            .next()
            .ok_or_else(|| err("record: missing name"))?
            .as_str(),
    );

    let mut record_length: Option<usize> = None;
    let mut fields = Vec::new();
    let mut derives = Vec::new();
    let mut checks = Vec::new();

    for child in inner {
        match child.as_rule() {
            Rule::record_length_kv => {
                let int_pair = child
                    .into_inner()
                    .next()
                    .ok_or_else(|| err("record_length: missing value"))?;
                record_length = Some(
                    int_pair
                        .as_str()
                        .parse()
                        .map_err(|e| err(format!("record_length: {e}")))?,
                );
            }
            Rule::field_block => fields.push(parse_field(child)?),
            Rule::derive_stmt => derives.push(parse_derive(child)?),
            Rule::check_block => checks.push(parse_check(child)?),
            r => return Err(err(format!("unexpected rule in record block: {r:?}"))),
        }
    }

    Ok(Record {
        name: name.clone(),
        record_length: record_length
            .ok_or_else(|| err(format!("record `{name}`: missing `record_length`")))?,
        fields,
        derives,
        checks,
    })
}

// ── model ──────────────────────────────────────────────────────────────

fn parse_model(pair: Pair<Rule>) -> Result<Model, ParseError> {
    let mut inner = pair.into_inner();
    let number = unquote(
        inner
            .next()
            .ok_or_else(|| err("model: missing number"))?
            .as_str(),
    );
    let version = unquote(
        inner
            .next()
            .ok_or_else(|| err("model: missing version"))?
            .as_str(),
    );

    let mut encoding: Option<Encoding> = None;
    let mut line_ending: Option<LineEnding> = None;
    let mut record_length: Option<usize> = None;

    for attr in inner {
        let (key, value) = key_value(attr)?;
        match key.as_str() {
            "encoding" => {
                let s = unquote(value.as_str());
                encoding = Some(match s.as_str() {
                    "ISO-8859-1" => Encoding::Iso8859_1,
                    "ASCII" => Encoding::Ascii,
                    other => return Err(err(format!("unknown encoding: {other}"))),
                });
            }
            "line_ending" => {
                let s = unquote(value.as_str());
                line_ending = Some(match s.as_str() {
                    "CRLF" => LineEnding::Crlf,
                    "LF" => LineEnding::Lf,
                    other => return Err(err(format!("unknown line_ending: {other}"))),
                });
            }
            "record_length" => {
                record_length = Some(
                    value
                        .as_str()
                        .parse()
                        .map_err(|e| err(format!("invalid record_length: {e}")))?,
                );
            }
            other => return Err(err(format!("unknown model attribute: {other}"))),
        }
    }

    Ok(Model {
        number,
        version,
        encoding: encoding.ok_or_else(|| err("model: missing `encoding`"))?,
        line_ending: line_ending.ok_or_else(|| err("model: missing `line_ending`"))?,
        record_length: record_length.ok_or_else(|| err("model: missing `record_length`"))?,
    })
}

fn key_value(attr: Pair<Rule>) -> Result<(String, Pair<Rule>), ParseError> {
    let mut parts = attr.into_inner();
    let key = parts
        .next()
        .ok_or_else(|| err("attr: missing key"))?
        .as_str()
        .to_string();
    let value = parts.next().ok_or_else(|| err("attr: missing value"))?;
    Ok((key, value))
}

// ── field ──────────────────────────────────────────────────────────────

fn parse_field(pair: Pair<Rule>) -> Result<Field, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .ok_or_else(|| err("field: missing name"))?
        .as_str()
        .to_string();

    let mut at: Option<usize> = None;
    let mut length: Option<usize> = None;
    let mut ty: Option<FieldType> = None;
    let mut decimals = None;
    let mut align = None;
    let mut pad = None;
    let mut required = false;
    let mut fixed = None;
    let mut domain = None;
    let mut usage = Usage::All;
    let mut description = None;

    for attr in inner {
        let (key, value) = key_value(attr)?;
        // field_value = { string | int | ident }
        let v = value
            .into_inner()
            .next()
            .ok_or_else(|| err("field attr: empty value"))?;
        match (key.as_str(), v.as_rule()) {
            ("at", Rule::int) => {
                at = Some(v.as_str().parse().map_err(|e| err(format!("at: {e}")))?)
            }
            ("length", Rule::int) => {
                length = Some(
                    v.as_str()
                        .parse()
                        .map_err(|e| err(format!("length: {e}")))?,
                )
            }
            ("decimals", Rule::int) => {
                decimals = Some(
                    v.as_str()
                        .parse()
                        .map_err(|e| err(format!("decimals: {e}")))?,
                )
            }
            ("type", Rule::ident) => {
                ty = Some(match v.as_str() {
                    "alpha" => FieldType::Alpha,
                    "alphanumeric" => FieldType::Alphanumeric,
                    "number" => FieldType::Number,
                    "signed_amount" => FieldType::SignedAmount,
                    "unsigned_amount" => FieldType::UnsignedAmount,
                    other => return Err(err(format!("unknown field type: {other}"))),
                });
            }
            ("align", Rule::ident) => {
                align = Some(match v.as_str() {
                    "left" => Align::Left,
                    "right" => Align::Right,
                    other => return Err(err(format!("unknown align: {other}"))),
                });
            }
            ("pad", Rule::ident) => {
                pad = Some(match v.as_str() {
                    "space" => Pad::Space,
                    "zero" => Pad::Zero,
                    other => return Err(err(format!("unknown pad: {other}"))),
                });
            }
            ("required", Rule::ident) => {
                required = match v.as_str() {
                    "true" => true,
                    "false" => false,
                    other => return Err(err(format!("required must be true/false, got {other}"))),
                };
            }
            ("fixed", Rule::string) => fixed = Some(unquote(v.as_str())),
            ("domain", Rule::string) => domain = Some(unquote(v.as_str())),
            ("usage", Rule::ident) => {
                usage = match v.as_str() {
                    "all" => Usage::All,
                    "print_only" => Usage::PrintOnly,
                    "internet_only" => Usage::InternetOnly,
                    other => return Err(err(format!("unknown usage: {other}"))),
                };
            }
            ("description", Rule::string) => description = Some(unquote(v.as_str())),
            (k, r) => {
                return Err(err(format!(
                    "field `{name}`: unsupported attribute `{k}` (value rule {r:?})"
                )))
            }
        }
    }

    Ok(Field {
        name,
        at: at.ok_or_else(|| err("field: missing `at`"))?,
        length: length.ok_or_else(|| err("field: missing `length`"))?,
        ty: ty.ok_or_else(|| err("field: missing `type`"))?,
        decimals,
        align,
        pad,
        required,
        fixed,
        domain,
        usage,
        description,
    })
}

// ── derive ─────────────────────────────────────────────────────────────

fn parse_derive(pair: Pair<Rule>) -> Result<Derive, ParseError> {
    let mut inner = pair.into_inner();
    let target = inner
        .next()
        .ok_or_else(|| err("derive: missing target"))?
        .as_str()
        .to_string();
    let expr_pair = inner.next().ok_or_else(|| err("derive: missing expr"))?;
    let expr = parse_expr(expr_pair)?;
    Ok(Derive { target, expr })
}

// ── check ──────────────────────────────────────────────────────────────

fn parse_check(pair: Pair<Rule>) -> Result<Check, ParseError> {
    let mut inner = pair.into_inner();
    let code = inner
        .next()
        .ok_or_else(|| err("check: missing code"))?
        .as_str()
        .to_string();

    let mut rule: Option<BoolExpr> = None;
    let mut severity: Option<Severity> = None;
    let mut message: Option<String> = None;

    for attr in inner {
        let (key, value) = key_value(attr)?;
        let v = value
            .into_inner()
            .next()
            .ok_or_else(|| err("check attr: empty value"))?;
        match (key.as_str(), v.as_rule()) {
            ("rule", Rule::bool_expr) => rule = Some(parse_bool_expr(v)?),
            ("severity", Rule::ident) => {
                severity = Some(match v.as_str() {
                    "error" => Severity::Error,
                    "warning" => Severity::Warning,
                    other => return Err(err(format!("unknown severity: {other}"))),
                });
            }
            ("message", Rule::string) => message = Some(unquote(v.as_str())),
            (k, r) => {
                return Err(err(format!(
                    "check `{code}`: unsupported attr `{k}` (rule {r:?})"
                )))
            }
        }
    }

    Ok(Check {
        code,
        rule: rule.ok_or_else(|| err("check: missing `rule`"))?,
        severity: severity.ok_or_else(|| err("check: missing `severity`"))?,
        message: message.ok_or_else(|| err("check: missing `message`"))?,
    })
}

// ── expressions ────────────────────────────────────────────────────────

fn parse_bool_expr(pair: Pair<Rule>) -> Result<BoolExpr, ParseError> {
    // bool_expr = { cmp_expr ~ ("implies" ~ cmp_expr)* }
    // Right-associative: `a implies b implies c` == `a implies (b implies c)`.
    let mut cmps: Vec<BoolExpr> = Vec::new();
    for child in pair.into_inner() {
        if child.as_rule() == Rule::cmp_expr {
            cmps.push(parse_cmp_expr(child)?);
        }
    }
    let mut iter = cmps.into_iter().rev();
    let mut acc = iter.next().ok_or_else(|| err("bool_expr: empty"))?;
    for lhs in iter {
        acc = BoolExpr::Implies(Box::new(lhs), Box::new(acc));
    }
    Ok(acc)
}

fn parse_cmp_expr(pair: Pair<Rule>) -> Result<BoolExpr, ParseError> {
    // cmp_expr = { expr ~ (cmp_op ~ expr)? }
    let mut inner = pair.into_inner();
    let lhs = parse_expr(inner.next().ok_or_else(|| err("cmp_expr: missing lhs"))?)?;
    let op_pair = inner.next().ok_or_else(|| err("cmp_expr: missing op"))?;
    let op = match op_pair.as_str() {
        "==" => CmpOp::Eq,
        "!=" => CmpOp::Ne,
        ">=" => CmpOp::Ge,
        "<=" => CmpOp::Le,
        ">" => CmpOp::Gt,
        "<" => CmpOp::Lt,
        other => return Err(err(format!("unknown cmp op: {other}"))),
    };
    let rhs = parse_expr(inner.next().ok_or_else(|| err("cmp_expr: missing rhs"))?)?;
    Ok(BoolExpr::Cmp(lhs, op, rhs))
}

fn parse_expr(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    match pair.as_rule() {
        Rule::expr => parse_expr(pair.into_inner().next().ok_or_else(|| err("empty expr"))?),
        Rule::add_expr => parse_binary(pair.into_inner(), |s| match s {
            "+" => Some(BinOp::Add),
            "-" => Some(BinOp::Sub),
            _ => None,
        }),
        Rule::mul_expr => parse_binary(pair.into_inner(), |s| match s {
            "*" => Some(BinOp::Mul),
            "/" => Some(BinOp::Div),
            _ => None,
        }),
        Rule::atom => parse_atom(pair),
        Rule::int => Ok(Expr::Int(
            pair.as_str()
                .parse()
                .map_err(|e| err(format!("int: {e}")))?,
        )),
        Rule::ident => Ok(Expr::Ident(pair.as_str().to_string())),
        Rule::string => Ok(Expr::Str(unquote(pair.as_str()))),
        Rule::func_call => parse_func_call(pair),
        r => Err(err(format!("parse_expr: unexpected rule {r:?}"))),
    }
}

fn parse_binary<F>(mut inner: Pairs<Rule>, map_op: F) -> Result<Expr, ParseError>
where
    F: Fn(&str) -> Option<BinOp>,
{
    let first = inner.next().ok_or_else(|| err("binary: empty"))?;
    let mut acc = parse_expr(first)?;
    while let Some(op_pair) = inner.next() {
        // op_pair should be add_op / mul_op
        let op = map_op(op_pair.as_str())
            .ok_or_else(|| err(format!("unknown op: {}", op_pair.as_str())))?;
        let rhs_pair = inner.next().ok_or_else(|| err("binary: missing rhs"))?;
        let rhs = parse_expr(rhs_pair)?;
        acc = Expr::Bin(Box::new(acc), op, Box::new(rhs));
    }
    Ok(acc)
}

fn parse_atom(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let inner = pair.into_inner().next().ok_or_else(|| err("atom: empty"))?;
    parse_expr(inner)
}

fn parse_func_call(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .ok_or_else(|| err("func_call: missing name"))?
        .as_str();
    let func = match name {
        "max" => BuiltinFn::Max,
        "min" => BuiltinFn::Min,
        other => return Err(err(format!("unknown function: {other}"))),
    };
    let args = inner.map(parse_expr).collect::<Result<Vec<_>, _>>()?;
    Ok(Expr::Call(func, args))
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}
