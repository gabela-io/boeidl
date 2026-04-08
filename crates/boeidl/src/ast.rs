//! AST types for the .boe DSL.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Encoding {
    Iso8859_1,
    Ascii,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineEnding {
    Crlf,
    Lf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoeFile {
    pub model: Model,
    pub fields: Vec<Field>,
    pub derives: Vec<Derive>,
    pub checks: Vec<Check>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub number: String,
    pub version: String,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    pub record_length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Alpha,
    Alphanumeric,
    Number,
    SignedAmount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pad {
    Space,
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Usage {
    All,
    PrintOnly,
    InternetOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub at: usize,
    pub length: usize,
    pub ty: FieldType,
    pub decimals: Option<usize>,
    pub align: Option<Align>,
    pub pad: Option<Pad>,
    pub required: bool,
    pub fixed: Option<String>,
    pub domain: Option<String>,
    pub usage: Usage,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Derive {
    pub target: String,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub code: String,
    pub rule: BoolExpr,
    pub severity: Severity,
    pub message: String,
}

// ── expressions ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// Integer literal.
    Int(i64),
    /// String literal (for comparisons against `domain`-style values).
    Str(String),
    /// Reference to a field by name.
    Ident(String),
    /// `a op b`.
    Bin(Box<Expr>, BinOp, Box<Expr>),
    /// `max(a, b)` / `min(a, b)`.
    Call(BuiltinFn, Vec<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinFn {
    Max,
    Min,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoolExpr {
    /// A single comparison `a <op> b`.
    Cmp(Expr, CmpOp, Expr),
    /// `a implies b`.
    Implies(Box<BoolExpr>, Box<BoolExpr>),
}
