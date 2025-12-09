// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Diff command - compare schema files and show differences

use anyhow::{Context, Result};
use colored::Colorize;
use lumos_core::ir::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, StructDefinition, TypeDefinition,
};
use lumos_core::parser::parse_lumos_file;
use lumos_core::transform::transform_to_ir;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::utils::format_type;

/// Compare two schema files and show differences
pub fn run(schema1_path: &Path, schema2_path: &Path, format: &str) -> Result<()> {
    println!("{:>12} schemas...", "Comparing".cyan().bold());
    println!("  Schema 1: {}", schema1_path.display());
    println!("  Schema 2: {}", schema2_path.display());
    println!();

    // Read and parse first schema
    let content1 = fs::read_to_string(schema1_path)
        .with_context(|| format!("Failed to read schema file: {}", schema1_path.display()))?;

    let ast1 = parse_lumos_file(&content1)
        .with_context(|| format!("Failed to parse schema: {}", schema1_path.display()))?;

    let ir1 = transform_to_ir(ast1)?;

    // Read and parse second schema
    let content2 = fs::read_to_string(schema2_path)
        .with_context(|| format!("Failed to read schema file: {}", schema2_path.display()))?;

    let ast2 = parse_lumos_file(&content2)
        .with_context(|| format!("Failed to parse schema: {}", schema2_path.display()))?;

    let ir2 = transform_to_ir(ast2)?;

    // Build maps for efficient lookup
    let map1: HashMap<&str, &TypeDefinition> = ir1.iter().map(|t| (t.name(), t)).collect();

    let map2: HashMap<&str, &TypeDefinition> = ir2.iter().map(|t| (t.name(), t)).collect();

    let names1: HashSet<&str> = map1.keys().copied().collect();
    let names2: HashSet<&str> = map2.keys().copied().collect();

    // Calculate differences
    let added: Vec<&str> = names2.difference(&names1).copied().collect();
    let removed: Vec<&str> = names1.difference(&names2).copied().collect();
    let common: Vec<&str> = names1.intersection(&names2).copied().collect();

    // Track modifications
    let mut modified = Vec::new();
    let mut modifications = Vec::new();

    for name in &common {
        let type1 = map1[name];
        let type2 = map2[name];

        let changes = compare_types(type1, type2);
        if !changes.is_empty() {
            modified.push(*name);
            modifications.push((*name, changes));
        }
    }

    // Output based on format
    // TODO: Implement output formatting
    if format == "json" {
        // output_diff_json(&added, &removed, &modifications)?;
        eprintln!("JSON output not implemented");
    } else {
        // output_diff_text(&added, &removed, &modifications)?;
        eprintln!("Text output not implemented");
    }

    // Summary
    let total_changes = added.len() + removed.len() + modified.len();
    if total_changes == 0 {
        println!("\n{} No differences found", "✓".green().bold());
    } else {
        println!(
            "\n{} {} change{} detected",
            "✓".green().bold(),
            total_changes,
            if total_changes == 1 { "" } else { "s" }
        );
    }

    Ok(())
}

/// Compare two type definitions and return list of changes
fn compare_types(type1: &TypeDefinition, type2: &TypeDefinition) -> Vec<String> {
    let mut changes = Vec::new();

    match (type1, type2) {
        (TypeDefinition::Struct(s1), TypeDefinition::Struct(s2)) => {
            compare_structs(s1, s2, &mut changes);
        }
        (TypeDefinition::Enum(e1), TypeDefinition::Enum(e2)) => {
            compare_enums(e1, e2, &mut changes);
        }
        (TypeDefinition::TypeAlias(_), TypeDefinition::TypeAlias(_)) => {
            changes.push("Type alias target may have changed".to_string());
        }
        (TypeDefinition::Struct(_), TypeDefinition::Enum(_)) => {
            changes.push("Type changed from struct to enum".to_string());
        }
        (TypeDefinition::Struct(_), TypeDefinition::TypeAlias(_)) => {
            changes.push("Type changed from struct to type alias".to_string());
        }
        (TypeDefinition::Enum(_), TypeDefinition::Struct(_)) => {
            changes.push("Type changed from enum to struct".to_string());
        }
        (TypeDefinition::Enum(_), TypeDefinition::TypeAlias(_)) => {
            changes.push("Type changed from enum to type alias".to_string());
        }
        (TypeDefinition::TypeAlias(_), TypeDefinition::Struct(_)) => {
            changes.push("Type changed from type alias to struct".to_string());
        }
        (TypeDefinition::TypeAlias(_), TypeDefinition::Enum(_)) => {
            changes.push("Type changed from type alias to enum".to_string());
        }
    }

    changes
}

/// Compare two struct definitions
fn compare_structs(s1: &StructDefinition, s2: &StructDefinition, changes: &mut Vec<String>) {
    let fields1: HashMap<&str, &FieldDefinition> =
        s1.fields.iter().map(|f| (f.name.as_str(), f)).collect();

    let fields2: HashMap<&str, &FieldDefinition> =
        s2.fields.iter().map(|f| (f.name.as_str(), f)).collect();

    let names1: HashSet<&str> = fields1.keys().copied().collect();
    let names2: HashSet<&str> = fields2.keys().copied().collect();

    // Added fields
    for field in names2.difference(&names1) {
        let f = fields2[field];
        changes.push(format!(
            "+ Added field: {} ({})",
            field,
            format_type(&f.type_info)
        ));
    }

    // Removed fields
    for field in names1.difference(&names2) {
        let f = fields1[field];
        changes.push(format!(
            "- Removed field: {} ({})",
            field,
            format_type(&f.type_info)
        ));
    }

    // Modified fields
    for field in names1.intersection(&names2) {
        let f1 = fields1[field];
        let f2 = fields2[field];

        if format_type(&f1.type_info) != format_type(&f2.type_info) {
            changes.push(format!(
                "~ Modified field: {} ({} → {})",
                field,
                format_type(&f1.type_info),
                format_type(&f2.type_info)
            ));
        }
    }

    // Check metadata changes
    if s1.metadata.solana != s2.metadata.solana {
        changes.push(format!(
            "~ Solana attribute changed: {} → {}",
            s1.metadata.solana, s2.metadata.solana
        ));
    }

    // Check other attributes
    if s1.metadata.attributes != s2.metadata.attributes {
        changes.push("~ Attributes changed".to_string());
    }
}

/// Compare two enum definitions
fn compare_enums(e1: &EnumDefinition, e2: &EnumDefinition, changes: &mut Vec<String>) {
    let variants1: HashMap<&str, &EnumVariantDefinition> =
        e1.variants.iter().map(|v| (v.name(), v)).collect();

    let variants2: HashMap<&str, &EnumVariantDefinition> =
        e2.variants.iter().map(|v| (v.name(), v)).collect();

    let names1: HashSet<&str> = variants1.keys().copied().collect();
    let names2: HashSet<&str> = variants2.keys().copied().collect();

    // Added variants
    for variant in names2.difference(&names1) {
        changes.push(format!("+ Added variant: {}", variant));
    }

    // Removed variants
    for variant in names1.difference(&names2) {
        changes.push(format!("- Removed variant: {}", variant));
    }

    // Modified variants
    for variant in names1.intersection(&names2) {
        let v1 = variants1[variant];
        let v2 = variants2[variant];

        if !variants_equal(v1, v2) {
            changes.push(format!("~ Modified variant: {}", variant));
        }
    }

    // Check metadata changes
    if e1.metadata.solana != e2.metadata.solana {
        changes.push(format!(
            "~ Solana attribute changed: {} → {}",
            e1.metadata.solana, e2.metadata.solana
        ));
    }
}

/// Check if two enum variants are equal
fn variants_equal(v1: &EnumVariantDefinition, v2: &EnumVariantDefinition) -> bool {
    match (v1, v2) {
        (EnumVariantDefinition::Unit { .. }, EnumVariantDefinition::Unit { .. }) => true,
        (
            EnumVariantDefinition::Tuple { types: t1, .. },
            EnumVariantDefinition::Tuple { types: t2, .. },
        ) => {
            t1.len() == t2.len()
                && t1
                    .iter()
                    .zip(t2.iter())
                    .all(|(a, b)| format_type(a) == format_type(b))
        }
        (
            EnumVariantDefinition::Struct { fields: f1, .. },
            EnumVariantDefinition::Struct { fields: f2, .. },
        ) => {
            f1.len() == f2.len()
                && f1.iter().zip(f2.iter()).all(|(a, b)| {
                    a.name == b.name && format_type(&a.type_info) == format_type(&b.type_info)
                })
        }
        _ => false,
    }
}
