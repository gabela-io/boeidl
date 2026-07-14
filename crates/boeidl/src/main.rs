//! boeidl CLI: compile / check / inspect

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use boeidl::ast::{BoeFile, Field, FieldType, Usage};
use boeidl::codegen::rust::generate;
use boeidl::validator::{validate, DiagLevel};

#[derive(Parser)]
#[command(
    name = "boeidl",
    about = "DSL compiler for AEAT fixed-position file formats",
    version
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Parse and semantically validate a .boe file, then emit a Rust module.
    Compile {
        /// Input .boe file
        input: PathBuf,
        /// Output directory. A file named `mod<number>.rs` will be written.
        #[arg(long)]
        out: PathBuf,
    },
    /// Parse and validate a .boe file without generating code.
    Check { input: PathBuf },
    /// Print a human-readable table of fields, positions, and validations.
    Inspect { input: PathBuf },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.cmd {
        Cmd::Compile { input, out } => cmd_compile(&input, &out),
        Cmd::Check { input } => cmd_check(&input),
        Cmd::Inspect { input } => cmd_inspect(&input),
    }
}

fn load(input: &Path) -> Result<BoeFile, String> {
    let src =
        std::fs::read_to_string(input).map_err(|e| format!("reading {}: {e}", input.display()))?;
    let file = boeidl::parse(&src).map_err(|e| format!("parse: {e}"))?;
    let diags = validate(&file);
    let errors: Vec<_> = diags
        .iter()
        .filter(|d| d.level == DiagLevel::Error)
        .collect();
    if !errors.is_empty() {
        let mut msg = String::from("semantic errors:\n");
        for d in &errors {
            msg.push_str(&format!("  - {}\n", d.message));
        }
        return Err(msg);
    }
    // Surface warnings on stderr but don't fail.
    for d in diags.iter().filter(|d| d.level == DiagLevel::Warning) {
        eprintln!("warning: {}", d.message);
    }
    Ok(file)
}

fn cmd_compile(input: &Path, out_dir: &Path) -> Result<(), String> {
    let file = load(input)?;
    let src = generate(&file);
    std::fs::create_dir_all(out_dir).map_err(|e| format!("creating {}: {e}", out_dir.display()))?;
    let out_path = out_dir.join(format!("mod{}.rs", file.model.number));
    std::fs::write(&out_path, src).map_err(|e| format!("writing {}: {e}", out_path.display()))?;
    eprintln!("wrote {}", out_path.display());
    Ok(())
}

fn cmd_check(input: &Path) -> Result<(), String> {
    let _ = load(input)?;
    println!("ok: {} is valid", input.display());
    Ok(())
}

fn cmd_inspect(input: &Path) -> Result<(), String> {
    let file = load(input)?;
    println!(
        "Modelo {} (version {})",
        file.model.number, file.model.version
    );
    println!("Encoding: {:?}", file.model.encoding);
    println!("Record length: {}", file.model.record_length);
    println!();
    println!(
        "{:<4} {:<32} {:<6} {:<6} {:<14} {:<14} Usage",
        "#", "Name", "At", "Len", "Type", "Fixed"
    );
    println!("{}", "─".repeat(100));
    let all_fields: Vec<&Field> = file.records.iter().flat_map(|r| r.fields.iter()).collect();
    for (i, f) in all_fields.iter().enumerate() {
        println!(
            "{:<4} {:<32} {:<6} {:<6} {:<14} {:<14} {}",
            i + 1,
            truncate(&f.name, 32),
            f.at,
            f.length,
            field_type_name(f),
            f.fixed
                .as_deref()
                .map(|s| truncate(s, 14))
                .unwrap_or_default(),
            usage_name(f.usage),
        );
    }
    println!();
    let derives: usize = file.records.iter().map(|r| r.derives.len()).sum();
    let checks: usize = file.records.iter().map(|r| r.checks.len()).sum();
    println!("Derives: {}", derives);
    println!("Checks: {}", checks);
    Ok(())
}

fn field_type_name(f: &Field) -> String {
    match f.ty {
        FieldType::Alpha => "alpha".to_string(),
        FieldType::Alphanumeric => "alphanumeric".to_string(),
        FieldType::Number => "number".to_string(),
        FieldType::SignedAmount => match f.decimals {
            Some(d) => format!("amount({d})"),
            None => "amount".to_string(),
        },
        FieldType::UnsignedAmount => match f.decimals {
            Some(d) => format!("uamount({d})"),
            None => "uamount".to_string(),
        },
    }
}

fn usage_name(u: Usage) -> &'static str {
    match u {
        Usage::All => "all",
        Usage::PrintOnly => "print_only",
        Usage::InternetOnly => "internet_only",
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(n - 1).collect();
        t.push('…');
        t
    }
}
