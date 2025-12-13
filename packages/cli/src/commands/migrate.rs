// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Migrate command - generate migration code between schema versions

use anyhow::{Context, Result};
use colored::Colorize;
use lumos_core::migration::{generate_rust_migration, generate_typescript_migration, SchemaDiff};
use lumos_core::parser::parse_lumos_file;
use lumos_core::transform::transform_to_ir;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Generate migration code from one schema version to another
pub fn run(
    from_schema_path: &Path,
    to_schema_path: &Path,
    output_path: Option<&Path>,
    language: &str,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    println!(
        "{:>12} migration from schemas...",
        "Generating".cyan().bold()
    );
    println!("  From: {}", from_schema_path.display());
    println!("  To:   {}", to_schema_path.display());
    println!();

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

    // Find types with same name across both schemas
    let from_map: HashMap<&str, &lumos_core::ir::TypeDefinition> =
        from_ir.iter().map(|t| (t.name(), t)).collect();
    let to_map: HashMap<&str, &lumos_core::ir::TypeDefinition> =
        to_ir.iter().map(|t| (t.name(), t)).collect();

    // Find common types that need migration
    let mut migrations_generated = 0;
    let mut rust_code = String::new();
    let mut typescript_code = String::new();

    for (type_name, from_def) in &from_map {
        if let Some(to_def) = to_map.get(type_name) {
            // Compute diff
            match SchemaDiff::compute(from_def, to_def) {
                Ok(diff) => {
                    // Show diff description
                    println!("{}", diff.describe());
                    println!();

                    // Check if migration is safe
                    if !diff.is_safe && !force {
                        println!(
                            "{} Unsafe migration detected. Use --force to proceed anyway.",
                            "✗".red().bold()
                        );
                        anyhow::bail!("Unsafe migration requires --force flag");
                    }

                    // Skip if no changes
                    if diff.changes.is_empty() {
                        println!(
                            "{} No migration needed for {}",
                            "✓".green().bold(),
                            type_name
                        );
                        continue;
                    }

                    // Generate migration code
                    if language == "rust" || language == "both" {
                        let migration_code = generate_rust_migration(&diff, from_def);
                        rust_code.push_str(&migration_code);
                        rust_code.push('\n');
                    }

                    if language == "typescript" || language == "both" {
                        let migration_code = generate_typescript_migration(&diff, from_def);
                        typescript_code.push_str(&migration_code);
                        typescript_code.push('\n');
                    }

                    migrations_generated += 1;
                }
                Err(e) => {
                    println!(
                        "{} Failed to compute diff for {}: {}",
                        "✗".red().bold(),
                        type_name,
                        e
                    );
                }
            }
        }
    }

    if migrations_generated == 0 {
        println!("{} No migrations needed", "✓".green().bold());
        return Ok(());
    }

    // Output migration code
    if dry_run {
        println!(
            "\n{} Dry run mode - showing generated code:",
            "ℹ".cyan().bold()
        );
        println!();

        if language == "rust" || language == "both" {
            println!("{}:", "Rust Migration Code".bold());
            println!("{}", "─".repeat(80));
            println!("{}", rust_code);
        }

        if language == "typescript" || language == "both" {
            println!("{}:", "TypeScript Migration Code".bold());
            println!("{}", "─".repeat(80));
            println!("{}", typescript_code);
        }
    } else {
        // Write to files or stdout
        if let Some(output_path) = output_path {
            if language == "rust" || language == "both" {
                let rust_output = if language == "both" {
                    output_path.with_extension("rs")
                } else {
                    output_path.to_path_buf()
                };
                fs::write(&rust_output, &rust_code).with_context(|| {
                    format!(
                        "Failed to write Rust migration code to: {}",
                        rust_output.display()
                    )
                })?;
                println!(
                    "{} Generated: {}",
                    "✓".green().bold(),
                    rust_output.display()
                );
            }

            if language == "typescript" || language == "both" {
                let ts_output = if language == "both" {
                    output_path.with_extension("ts")
                } else {
                    output_path.to_path_buf()
                };
                fs::write(&ts_output, &typescript_code).with_context(|| {
                    format!(
                        "Failed to write TypeScript migration code to: {}",
                        ts_output.display()
                    )
                })?;
                println!("{} Generated: {}", "✓".green().bold(), ts_output.display());
            }
        } else {
            // Output to stdout
            if language == "rust" || language == "both" {
                print!("{}", rust_code);
            }
            if language == "typescript" || language == "both" {
                print!("{}", typescript_code);
            }
        }
    }

    println!(
        "\n{} Successfully generated {} migration{}",
        "✓".green().bold(),
        migrations_generated,
        if migrations_generated == 1 { "" } else { "s" }
    );

    Ok(())
}
