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

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub name: String,
    pub record_length: usize,
    pub fields: Vec<Field>,
    pub derives: Vec<Derive>,
    pub checks: Vec<Check>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoeFile {
    pub model: Model,
    pub records: Vec<Record>,
    pub envelope: Option<Envelope>,
}

/// Una plantilla de texto de longitud fija: literal + interpolaciones `${ident}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplatePart {
    Lit(String),
    /// Referencia a un `param` del envelope por nombre.
    Field(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Template(pub Vec<TemplatePart>);

/// Un valor compartido a nivel fichero, interpolable en header/trailer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub length: usize,
    pub ty: FieldType,
    pub required: bool,
    pub description: Option<String>,
}

/// El "sobre" AEAT: header/trailer de longitud fija que envuelven una
/// secuencia de `record`s (`contains`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub params: Vec<Param>,
    pub header: Template,
    pub trailer: Template,
    /// Nombres de `record` en orden de concatenación.
    pub contains: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub number: String,
    pub version: String,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    /// On single-record files this is authoritative. When the file contains explicit record blocks, each record's own record_length wins and this field holds the primary record's length (first declared).
    pub record_length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    Alpha,
    Alphanumeric,
    Number,
    SignedAmount,
    /// Importe con signo estilo sobre AEAT (`numN`): positivo = dígitos a lo
    /// ancho completo (sin espacio), negativo = `'N'` + dígitos.
    SignedAmountN,
    UnsignedAmount,
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
