//! boeidl — DSL compiler for AEAT fixed-position file formats.

pub mod ast;
pub mod parser;
pub mod runtime;
pub mod validator;

pub use parser::{parse, ParseError};
pub use validator::{validate, DiagLevel, Diagnostic};
