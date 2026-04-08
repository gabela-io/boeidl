//! pest → AST conversion.

use pest::Parser;
use pest_derive::Parser;

use crate::ast::{BoeFile, Encoding, LineEnding, Model};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct BoeParser;

/// Parse error with a human-readable message (positions from pest).
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

pub fn parse(src: &str) -> Result<BoeFile, ParseError> {
    let mut pairs = BoeParser::parse(Rule::file, src)?;
    let file = pairs.next().ok_or_else(|| ParseError("empty file".into()))?;

    let mut model: Option<Model> = None;
    for pair in file.into_inner() {
        match pair.as_rule() {
            Rule::model_block => model = Some(parse_model(pair)?),
            Rule::EOI => {}
            r => {
                return Err(ParseError(format!("unexpected rule at top level: {r:?}")));
            }
        }
    }

    let model = model.ok_or_else(|| ParseError("missing `model` block".into()))?;
    Ok(BoeFile { model })
}

fn parse_model(pair: pest::iterators::Pair<Rule>) -> Result<Model, ParseError> {
    let mut inner = pair.into_inner();

    let number = unquote(
        inner
            .next()
            .ok_or_else(|| ParseError("model: missing number".into()))?
            .as_str(),
    );
    let version = unquote(
        inner
            .next()
            .ok_or_else(|| ParseError("model: missing version".into()))?
            .as_str(),
    );

    let mut encoding: Option<Encoding> = None;
    let mut line_ending: Option<LineEnding> = None;
    let mut record_length: Option<usize> = None;

    for attr in inner {
        // attr = { ident ~ "=" ~ (string | int) }
        let mut parts = attr.into_inner();
        let key = parts
            .next()
            .ok_or_else(|| ParseError("model attr: missing key".into()))?
            .as_str()
            .to_string();
        let value = parts
            .next()
            .ok_or_else(|| ParseError("model attr: missing value".into()))?;

        match key.as_str() {
            "encoding" => {
                let s = unquote(value.as_str());
                encoding = Some(match s.as_str() {
                    "ISO-8859-1" => Encoding::Iso8859_1,
                    "ASCII" => Encoding::Ascii,
                    other => {
                        return Err(ParseError(format!("unknown encoding: {other}")));
                    }
                });
            }
            "line_ending" => {
                let s = unquote(value.as_str());
                line_ending = Some(match s.as_str() {
                    "CRLF" => LineEnding::Crlf,
                    "LF" => LineEnding::Lf,
                    other => {
                        return Err(ParseError(format!("unknown line_ending: {other}")));
                    }
                });
            }
            "record_length" => {
                record_length = Some(value.as_str().parse().map_err(|e| {
                    ParseError(format!("invalid record_length: {e}"))
                })?);
            }
            other => {
                return Err(ParseError(format!("unknown model attribute: {other}")));
            }
        }
    }

    Ok(Model {
        number,
        version,
        encoding: encoding
            .ok_or_else(|| ParseError("model: missing `encoding`".into()))?,
        line_ending: line_ending
            .ok_or_else(|| ParseError("model: missing `line_ending`".into()))?,
        record_length: record_length
            .ok_or_else(|| ParseError("model: missing `record_length`".into()))?,
    })
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}
