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

/// Top-level `.boe` file. For batch 1 this only contains the `model` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoeFile {
    pub model: Model,
}

/// `model "<number>" version "<version>" { ... }` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub number: String,
    pub version: String,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    pub record_length: usize,
}
