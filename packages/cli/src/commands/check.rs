// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Check command - verify generated code is up-to-date and analyze sizes

use anyhow::{Context, Result};
use colored::Colorize;
use lumos_core::generators::{rust, typescript};
use lumos_core::parser::parse_lumos_file;
use lumos_core::size_calculator::SizeCalculator;
use lumos_core::transform::transform_to_ir;
use std::fs;
use std::path::Path;

use crate::utils::validate_output_path;

/// Verify generated code is up-to-date
pub fn run(schema_path: &Path, output_dir: Option<&Path>) -> Result<()> {
    let output_dir = output_dir.unwrap_or_else(|| Path::new("."));

    // Validate output directory
    validate_output_path(output_dir)?;

    println!("{:>12} generated code status", "Checking".cyan().bold());

    // Check if output files exist
    let rust_output = output_dir.join("generated.rs");
    let ts_output = output_dir.join("generated.ts");

    let rust_exists = rust_output.exists();
    let ts_exists = ts_output.exists();

    if !rust_exists || !ts_exists {
        eprintln!("{}: Generated files not found", "error".red().bold());
        if !rust_exists {
            eprintln!("  Missing: {}", rust_output.display());
        }
        if !ts_exists {
            eprintln!("  Missing: {}", ts_output.display());
        }
        eprintln!();
        eprintln!("Run: lumos generate {}", schema_path.display());
        std::process::exit(1);
    }

    // Read and parse schema
    let content = fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;

    let ast = parse_lumos_file(&content)
        .with_context(|| format!("Failed to parse schema: {}", schema_path.display()))?;

    let ir = transform_to_ir(ast).with_context(|| "Failed to transform AST to IR")?;

    // Generate fresh code
    let fresh_rust = rust::generate_module(&ir);
    let fresh_ts = typescript::generate_module(&ir);

    // Read existing generated code
    let existing_rust = fs::read_to_string(&rust_output)
        .with_context(|| format!("Failed to read {}", rust_output.display()))?;

    let existing_ts = fs::read_to_string(&ts_output)
        .with_context(|| format!("Failed to read {}", ts_output.display()))?;

    // Compare
    let rust_match = fresh_rust == existing_rust;
    let ts_match = fresh_ts == existing_ts;

    if rust_match && ts_match {
        println!(
            "{:>12} generated code is up-to-date",
            "Success".green().bold()
        );
        Ok(())
    } else {
        eprintln!(
            "{}: Generated code is out-of-date",
            "warning".yellow().bold()
        );
        if !rust_match {
            eprintln!("  {}", rust_output.display());
        }
        if !ts_match {
            eprintln!("  {}", ts_output.display());
        }
        eprintln!();
        eprintln!("Run: lumos generate {}", schema_path.display());
        std::process::exit(1);
    }
}

/// Check account sizes and detect overflow
pub fn run_size(schema_path: &Path, format: &str) -> Result<()> {
    // Read and parse schema
    let content = fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;

    let ast = parse_lumos_file(&content)
        .with_context(|| format!("Failed to parse schema: {}", schema_path.display()))?;

    let ir = transform_to_ir(ast).with_context(|| "Failed to transform AST to IR")?;

    if ir.is_empty() {
        eprintln!(
            "{}: No type definitions found in schema",
            "warning".yellow().bold()
        );
        return Ok(());
    }

    // Calculate sizes
    let mut calculator = SizeCalculator::new(&ir);
    let sizes = calculator.calculate_all();

    // TODO: Implement output formatting
    if format == "json" {
        // JSON output for programmatic use
        // output_json(&sizes)?;
        eprintln!("JSON output not implemented");
    } else {
        // Human-readable text output
        // output_text(&sizes)?;
        eprintln!("Text output not implemented");
    }

    // Exit with error if any account exceeds limits
    let has_errors = sizes.iter().any(|s| !s.warnings.is_empty());
    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}
