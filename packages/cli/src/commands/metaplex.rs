// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Metaplex Token Metadata commands - validation, code generation, and standards

use anyhow::{Context, Result};
use colored::Colorize;
use lumos_core::metaplex::{
    generate_standard_types, MetaplexGenerator, MetaplexValidator, Severity,
};
use std::fs;
use std::path::Path;

use crate::commands::generate::resolve_schema;

/// Validate schema against Metaplex Token Metadata standards
pub fn run_validate(schema_path: &Path, format: &str, verbose: bool) -> Result<()> {
    // Parse and transform schema
    let (type_defs, _file_count) = resolve_schema(schema_path)?;

    println!(
        "{:>12} schema against Metaplex standards...\n",
        "Validating".cyan().bold()
    );

    // Create validator and run validation
    let validator = MetaplexValidator::new();
    let result = validator.validate(&type_defs);

    if format == "json" {
        // Output JSON format
        let json_output = serde_json::json!({
            "valid": result.is_valid(),
            "errors": result.errors.iter().filter(|e| e.severity == Severity::Error).map(|e| {
                serde_json::json!({
                    "location": e.location,
                    "message": e.message,
                    "suggestion": e.suggestion
                })
            }).collect::<Vec<_>>(),
            "warnings": result.errors.iter().filter(|e| e.severity == Severity::Warning).map(|e| {
                serde_json::json!({
                    "location": e.location,
                    "message": e.message,
                    "suggestion": e.suggestion
                })
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Text output
        let errors: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.severity == Severity::Error)
            .collect();
        let warnings: Vec<_> = result
            .errors
            .iter()
            .filter(|e| e.severity == Severity::Warning)
            .collect();

        for error in &errors {
            print!("{:>12} ", "Error".red().bold());
            print!("{}: ", error.location);
            println!("{}", error.message);
            if let Some(suggestion) = &error.suggestion {
                println!("  {}: {}", "Hint".yellow(), suggestion);
            }
        }

        for warning in &warnings {
            print!("{:>12} ", "Warning".yellow().bold());
            print!("{}: ", warning.location);
            println!("{}", warning.message);
            if verbose {
                if let Some(suggestion) = &warning.suggestion {
                    println!("  {}: {}", "Hint".yellow(), suggestion);
                }
            }
        }

        println!();

        if result.is_valid() {
            println!(
                "{:>12} Schema is Metaplex compliant! ✓",
                "Success".green().bold()
            );
            if !warnings.is_empty() {
                println!(
                    "  {} warning(s) - consider addressing for best practices",
                    warnings.len()
                );
            }
        } else {
            println!(
                "{:>12} Schema has {} error(s), {} warning(s)",
                "Failed".red().bold(),
                errors.len(),
                warnings.len()
            );
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Generate Metaplex-compatible code from schema
pub fn run_generate(
    schema_path: &Path,
    output_dir: Option<&Path>,
    typescript: bool,
    rust: bool,
    dry_run: bool,
) -> Result<()> {
    // Parse and transform schema
    let (type_defs, _file_count) = resolve_schema(schema_path)?;

    // First validate
    let validator = MetaplexValidator::new();
    let validation = validator.validate(&type_defs);

    if !validation.is_valid() {
        println!("{:>12} Schema validation failed", "Error".red().bold());
        println!(
            "  Run `lumos metaplex validate {}` for details",
            schema_path.display()
        );
        std::process::exit(1);
    }

    println!(
        "{:>12} Metaplex-compatible code...\n",
        "Generating".cyan().bold()
    );

    let generator = MetaplexGenerator::new();
    let output = output_dir.unwrap_or(Path::new("."));

    // Generate Rust code
    if rust {
        let rust_code = generator.generate_rust(&type_defs);
        let rust_path = output.join("metaplex_types.rs");

        if dry_run {
            println!("{}:", "Rust output (dry run)".cyan());
            println!("{}", rust_code);
        } else {
            fs::write(&rust_path, &rust_code)
                .with_context(|| format!("Failed to write {}", rust_path.display()))?;
            println!("{:>12} {}", "Generated".green().bold(), rust_path.display());
        }
    }

    // Generate TypeScript code
    if typescript {
        let ts_code = generator.generate_typescript(&type_defs);
        let ts_path = output.join("metaplex_types.ts");

        if dry_run {
            println!("\n{}:", "TypeScript output (dry run)".cyan());
            println!("{}", ts_code);
        } else {
            fs::write(&ts_path, &ts_code)
                .with_context(|| format!("Failed to write {}", ts_path.display()))?;
            println!("{:>12} {}", "Generated".green().bold(), ts_path.display());
        }
    }

    if !dry_run {
        println!(
            "\n{:>12} Metaplex-compatible code generated!",
            "Success".green().bold()
        );
    }

    Ok(())
}

/// Show standard Metaplex type definitions
pub fn run_types(format: &str) -> Result<()> {
    println!(
        "{:>12} Metaplex standard types...\n",
        "Showing".cyan().bold()
    );

    if format == "json" {
        // Output JSON format with type information
        let json_output = serde_json::json!({
            "types": [
                {
                    "name": "Metadata",
                    "description": "Token Metadata standard struct",
                    "fields": [
                        {"name": "name", "type": "String", "constraint": "max 32 chars"},
                        {"name": "symbol", "type": "String", "constraint": "max 10 chars"},
                        {"name": "uri", "type": "String", "constraint": "max 200 chars"},
                        {"name": "seller_fee_basis_points", "type": "u16", "constraint": "0-10000"},
                        {"name": "creators", "type": "Option<Vec<Creator>>", "constraint": "max 5"}
                    ]
                },
                {
                    "name": "Creator",
                    "description": "NFT creator with royalty share",
                    "fields": [
                        {"name": "address", "type": "PublicKey"},
                        {"name": "verified", "type": "bool"},
                        {"name": "share", "type": "u8", "constraint": "all shares sum to 100"}
                    ]
                },
                {
                    "name": "Collection",
                    "description": "Collection reference for NFT grouping",
                    "fields": [
                        {"name": "verified", "type": "bool"},
                        {"name": "key", "type": "PublicKey"}
                    ]
                },
                {
                    "name": "Uses",
                    "description": "NFT use tracking (e.g., for consumable NFTs)",
                    "fields": [
                        {"name": "use_method", "type": "UseMethod"},
                        {"name": "remaining", "type": "u64"},
                        {"name": "total", "type": "u64"}
                    ]
                }
            ],
            "constraints": {
                "MAX_NAME_LENGTH": 32,
                "MAX_SYMBOL_LENGTH": 10,
                "MAX_URI_LENGTH": 200,
                "MAX_CREATORS": 5,
                "MAX_SELLER_FEE_BASIS_POINTS": 10000,
                "CREATOR_SHARES_TOTAL": 100
            }
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Output LUMOS schema format
        let schema = generate_standard_types();
        println!("{}", schema);
    }

    Ok(())
}
