//! boeidl — DSL compiler for AEAT fixed-position file formats.

pub mod ast;
pub mod parser;

pub use parser::{parse, ParseError};
