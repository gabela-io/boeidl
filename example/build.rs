//! Build script: compile ../models/mod130.boe into a Rust module via boeidl.
//!
//! The generated file lands at `$OUT_DIR/mod130.rs` and is `include!`d from
//! src/generated.rs. In a real public-use scenario you would instead invoke
//! the installed CLI (`boeidl compile ...`), but inside this workspace we can
//! use the library directly.

use std::path::PathBuf;

fn main() {
    let input = PathBuf::from("../models/mod130.boe");
    println!("cargo:rerun-if-changed={}", input.display());

    let src = std::fs::read_to_string(&input)
        .unwrap_or_else(|e| panic!("reading {}: {e}", input.display()));
    let file = boeidl::parse(&src).expect("parse mod130.boe");

    let diags = boeidl::validate(&file);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == boeidl::DiagLevel::Error)
        .collect();
    if !errors.is_empty() {
        for d in &errors {
            eprintln!("semantic error: {}", d.message);
        }
        panic!("mod130.boe failed semantic validation");
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));

    let code = boeidl::codegen::rust::generate(&file);
    let out_path = out_dir.join("mod130.rs");
    std::fs::write(&out_path, code).expect("write generated mod130.rs");

    // Modelo 130 con sobre.
    let input_env = PathBuf::from("../models/mod130_sobre.boe");
    println!("cargo:rerun-if-changed={}", input_env.display());
    let src_env = std::fs::read_to_string(&input_env)
        .unwrap_or_else(|e| panic!("reading {}: {e}", input_env.display()));
    let file_env = boeidl::parse(&src_env).expect("parse mod130_sobre.boe");
    let errors_env: Vec<_> = boeidl::validate(&file_env)
        .into_iter()
        .filter(|d| d.level == boeidl::DiagLevel::Error)
        .collect();
    if !errors_env.is_empty() {
        for d in &errors_env {
            eprintln!("semantic error: {}", d.message);
        }
        panic!("mod130_sobre.boe failed semantic validation");
    }
    let code_env = boeidl::codegen::rust::generate(&file_env);
    std::fs::write(out_dir.join("mod130_sobre.rs"), code_env).expect("write mod130_sobre.rs");
}
