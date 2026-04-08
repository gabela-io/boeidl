//! Error types returned by generated marshal/unmarshal/validate code.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AeatError {
    /// The serialized value does not fit in the declared field width.
    FieldOverflow {
        field: String,
        width: usize,
        got: usize,
    },
    /// A required field was empty/missing at marshal time.
    MissingRequired { field: String },
    /// An invalid value for a field (e.g. not in `domain`).
    InvalidValue { field: String, value: String },
    /// The incoming byte buffer is shorter than `record_length`.
    ShortRecord { expected: usize, got: usize },
    /// The incoming byte buffer contains bytes that are not valid ISO-8859-1.
    /// (All bytes are valid ISO-8859-1, so this is never raised — kept for API stability.)
    InvalidEncoding,
}

impl std::fmt::Display for AeatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FieldOverflow { field, width, got } => write!(
                f,
                "field `{field}`: value has {got} chars but field width is {width}"
            ),
            Self::MissingRequired { field } => {
                write!(f, "field `{field}`: required value is empty")
            }
            Self::InvalidValue { field, value } => {
                write!(f, "field `{field}`: invalid value `{value}`")
            }
            Self::ShortRecord { expected, got } => {
                write!(f, "record too short: expected {expected} bytes, got {got}")
            }
            Self::InvalidEncoding => f.write_str("invalid ISO-8859-1 encoding"),
        }
    }
}

impl std::error::Error for AeatError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AeatDiagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub field: Option<String>,
}
