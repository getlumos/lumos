// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Validate command - syntax validation without code generation

use anyhow::{Context, Result};
use colored::Colorize;
use lumos_core::parser::parse_lumos_file;
use lumos_core::transform::transform_to_ir;
use std::fs;
use std::path::Path;

/// Validate schema syntax without generating code
pub fn run(schema_path: &Path) -> Result<()> {
    println!(
        "{:>12} {}",
        "Validating".cyan().bold(),
        schema_path.display()
    );

    let content = fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;

    let ast = parse_lumos_file(&content)
        .with_context(|| format!("Failed to parse schema: {}", schema_path.display()))?;

    let ir = transform_to_ir(ast).with_context(|| "Failed to transform AST to IR")?;

    if ir.is_empty() {
        println!("{}: No type definitions found", "warning".yellow().bold());
    } else {
        println!(
            "{:>12} Found {} valid type definitions",
            "Success".green().bold(),
            ir.len()
        );
    }

    Ok(())
}
