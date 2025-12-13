// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Check compatibility command - verify backward compatibility between schema versions

use anyhow::{Context, Result};
use colored::Colorize;
use lumos_core::compat::{CompatibilityChecker, IssueLevel};
use lumos_core::parser::parse_lumos_file;
use lumos_core::transform::transform_to_ir;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Check backward compatibility between two schema versions
pub fn run(
    from_schema_path: &Path,
    to_schema_path: &Path,
    format: &str,
    verbose: bool,
    strict: bool,
) -> Result<()> {
    // Read and parse old schema
    let from_content = fs::read_to_string(from_schema_path)
        .with_context(|| format!("Failed to read schema file: {}", from_schema_path.display()))?;

    let from_ast = parse_lumos_file(&from_content)
        .with_context(|| format!("Failed to parse schema: {}", from_schema_path.display()))?;

    let from_ir = transform_to_ir(from_ast)?;

    // Read and parse new schema
    let to_content = fs::read_to_string(to_schema_path)
        .with_context(|| format!("Failed to read schema file: {}", to_schema_path.display()))?;

    let to_ast = parse_lumos_file(&to_content)
        .with_context(|| format!("Failed to parse schema: {}", to_schema_path.display()))?;

    let to_ir = transform_to_ir(to_ast)?;

    // Build maps of types
    let from_map: HashMap<&str, &lumos_core::ir::TypeDefinition> =
        from_ir.iter().map(|t| (t.name(), t)).collect();
    let to_map: HashMap<&str, &lumos_core::ir::TypeDefinition> =
        to_ir.iter().map(|t| (t.name(), t)).collect();

    // Collect all compatibility reports
    let mut all_reports = Vec::new();
    let mut total_breaking = 0;
    let mut total_warnings = 0;
    let mut total_info = 0;

    // Check each type that exists in both schemas
    for (type_name, from_def) in &from_map {
        if let Some(to_def) = to_map.get(type_name) {
            let checker = CompatibilityChecker::new((*from_def).clone(), (*to_def).clone());
            match checker.check() {
                Ok(report) => {
                    total_breaking += report.count_by_level(IssueLevel::Breaking);
                    total_warnings += report.count_by_level(IssueLevel::Warning);
                    total_info += report.count_by_level(IssueLevel::Info);
                    all_reports.push(report);
                }
                Err(e) => {
                    eprintln!("{} Failed to check {}: {}", "✗".red().bold(), type_name, e);
                }
            }
        }
    }

    // Output results based on format
    if format == "json" {
        // JSON output
        let json_output = serde_json::json!({
            "from_version": all_reports.first().and_then(|r| r.from_version.clone()),
            "to_version": all_reports.first().and_then(|r| r.to_version.clone()),
            "compatible": total_breaking == 0,
            "version_bump_valid": all_reports.iter().all(|r| r.version_bump_valid),
            "breaking_changes": total_breaking,
            "warnings": total_warnings,
            "info": total_info,
            "reports": all_reports,
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Text output
        println!("{:>12} backward compatibility...", "Checking".cyan().bold());
        println!("  From: {}", from_schema_path.display());
        println!("  To:   {}", to_schema_path.display());
        println!();

        // Show version info if available
        if let Some(report) = all_reports.first() {
            if let (Some(ref from_ver), Some(ref to_ver)) =
                (&report.from_version, &report.to_version)
            {
                println!("Version: {} → {}", from_ver.bold(), to_ver.bold());
                println!();
            }
        }

        // Display issues for each type
        for report in &all_reports {
            if !report.issues.is_empty() {
                for issue in &report.issues {
                    println!("{}", issue);
                    if verbose {
                        println!("  Reason: {}", issue.reason);
                    }
                }
                println!();
            }
        }

        // Summary
        println!("{}", "Summary:".bold());

        if total_breaking == 0 {
            println!(
                "  {} All changes are backward compatible",
                "✓".green().bold()
            );
            println!(
                "  {} New schema can read data written by old schema",
                "✓".green().bold()
            );
        } else {
            println!(
                "  {} {} breaking change{}",
                "✗".red().bold(),
                total_breaking,
                if total_breaking == 1 { "" } else { "s" }
            );
            println!(
                "  {} New schema CANNOT read data written by old schema",
                "✗".red().bold()
            );
            println!();
            println!("  Recommendation: Create migration code or bump major version");
        }

        if total_warnings > 0 {
            println!(
                "  {} {} warning{}",
                "⚠".yellow().bold(),
                total_warnings,
                if total_warnings == 1 { "" } else { "s" }
            );
        }

        if verbose && total_info > 0 {
            println!(
                "  {} {} informational change{}",
                "ℹ".blue().bold(),
                total_info,
                if total_info == 1 { "" } else { "s" }
            );
        }

        // Version bump validation
        if !all_reports.iter().all(|r| r.version_bump_valid) {
            println!("\n{} Version bump validation failed", "✗".red().bold());
            println!("  Breaking changes require a major version bump");
        }
    }

    // Exit codes
    if total_breaking > 0 {
        std::process::exit(1); // Breaking changes
    } else if strict && total_warnings > 0 {
        std::process::exit(2); // Warnings in strict mode
    }

    Ok(())
}
