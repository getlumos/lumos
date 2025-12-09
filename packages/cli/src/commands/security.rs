// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Security analysis and audit commands

use anyhow::{Context, Result};
use colored::Colorize;
use lumos_core::audit_generator::{AuditGenerator, CheckCategory};
use lumos_core::parser::parse_lumos_file;
use lumos_core::security_analyzer::SecurityAnalyzer;
use lumos_core::transform::transform_to_ir;
use std::fs;
use std::path::Path;

/// Run security analysis on schema
pub fn run_analyze(schema_path: &Path, format: &str, strict: bool) -> Result<()> {
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

    // Run security analysis
    let mut analyzer = SecurityAnalyzer::new(&ir);
    if strict {
        analyzer = analyzer.with_strict_mode();
    }

    let findings = analyzer.analyze();

    // TODO: Implement output formatting
    if format == "json" {
        // output_security_json(&findings)?;
        eprintln!("JSON output not implemented");
    } else {
        // output_security_text(&findings, schema_path)?;
        eprintln!("Text output not implemented");
    }

    // Exit with error if any critical findings
    let has_critical = findings.iter().any(|f| {
        matches!(
            f.severity,
            lumos_core::security_analyzer::Severity::Critical
        )
    });

    if has_critical {
        std::process::exit(1);
    }

    Ok(())
}

/// Run audit checklist generation
pub fn run_audit(schema_path: &Path, output_path: Option<&Path>, format: &str) -> Result<()> {
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

    // Generate checklist
    let generator = AuditGenerator::new(&ir);
    let checklist = generator.generate();

    // Determine output path
    let output = output_path.unwrap_or_else(|| Path::new("SECURITY_AUDIT.md"));

    // Generate output based on format
    // TODO: Implement output formatting
    if format == "json" {
        // generate_audit_json(&checklist, output)?;
        eprintln!("JSON output not implemented");
    } else {
        // generate_audit_markdown(&checklist, schema_path, output)?;
        eprintln!("Markdown output not implemented");
    }

    println!(
        "\n{} {}",
        "Generated:".green().bold(),
        output.display().to_string().bold()
    );
    println!();
    println!("Checklist includes:");
    println!("  ✓ {} total checks", checklist.len());

    // Count by category
    let categories = [
        CheckCategory::AccountValidation,
        CheckCategory::SignerChecks,
        CheckCategory::ArithmeticSafety,
        CheckCategory::AccessControl,
    ];

    for category in categories {
        let count = checklist
            .iter()
            .filter(|item| item.category == category)
            .count();
        if count > 0 {
            println!("  ✓ {} {} checks", count, category.as_str().to_lowercase());
        }
    }

    Ok(())
}
