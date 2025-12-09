// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Generate command - main code generation from LUMOS schema

use anyhow::{Context, Result};
use colored::Colorize;
use lumos_core::ast::Item;
use lumos_core::file_resolver::FileResolver;
use lumos_core::generators::{get_generators, Language};
use lumos_core::ir::TypeDefinition;
use lumos_core::module_resolver::ModuleResolver;
use lumos_core::parser::parse_lumos_file;
use lumos_core::transform::transform_to_ir;
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::{
    create_backup_if_exists, preview_file_changes, validate_output_path, write_with_diff_check,
};

/// Target framework for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetMode {
    /// Auto-detect based on #[account] attribute
    Auto,
    /// Force native Solana (pure Borsh, no Anchor)
    Native,
    /// Force Anchor framework
    Anchor,
}

/// Resolve schema with auto-detection of module system
pub fn resolve_schema(schema_path: &Path) -> Result<(Vec<TypeDefinition>, usize)> {
    // Read the file to detect which resolution strategy to use
    let content = fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema: {}", schema_path.display()))?;

    let ast = parse_lumos_file(&content)
        .with_context(|| format!("Failed to parse schema: {}", schema_path.display()))?;

    // Check if file has module declarations
    let has_mod_declarations = ast
        .items
        .iter()
        .any(|item| matches!(item, Item::Module(_)));

    // Check if file has JS-style imports
    let has_imports = !ast.imports.is_empty();

    if has_mod_declarations {
        // Use ModuleResolver for hierarchical module system
        let mut resolver = ModuleResolver::new();
        let ir = resolver.resolve_modules(schema_path).with_context(|| {
            format!("Failed to resolve modules from: {}", schema_path.display())
        })?;
        let file_count = resolver.loaded_modules().len();
        Ok((ir, file_count))
    } else if has_imports {
        // Use FileResolver for JS-style imports
        let mut resolver = FileResolver::new();
        let ir = resolver.resolve_imports(schema_path).with_context(|| {
            format!("Failed to resolve imports from: {}", schema_path.display())
        })?;
        let file_count = resolver.loaded_files().len();
        Ok((ir, file_count))
    } else {
        // Single file, no imports or modules
        let ir = transform_to_ir(ast)
            .with_context(|| format!("Failed to transform schema: {}", schema_path.display()))?;
        Ok((ir, 1))
    }
}

/// Generate code from schema
pub fn run(
    schema_path: &Path,
    output_dir: Option<&Path>,
    lang: &str,
    target: &str,
    dry_run: bool,
    backup: bool,
    show_diff: bool,
) -> Result<()> {
    let output_dir = output_dir.unwrap_or_else(|| Path::new("."));

    // Validate target framework
    let target_mode = match target.to_lowercase().as_str() {
        "auto" => TargetMode::Auto,
        "native" => TargetMode::Native,
        "anchor" => TargetMode::Anchor,
        _ => anyhow::bail!(
            "Invalid target '{}'. Supported: auto, native, anchor",
            target
        ),
    };

    // Validate output directory for security
    validate_output_path(output_dir)?;

    // Parse target languages
    let requested_langs = Language::parse_list(lang);
    if requested_langs.is_empty() {
        anyhow::bail!(
            "No valid languages specified. Supported: rust, typescript. Planned: python, go, ruby"
        );
    }

    // Check for unimplemented languages
    let unimplemented: Vec<_> = requested_langs
        .iter()
        .filter(|l| !l.is_implemented())
        .collect();
    if !unimplemented.is_empty() {
        let names: Vec<_> = unimplemented.iter().map(|l| l.name()).collect();
        eprintln!(
            "{}: {} not yet implemented (see roadmap)",
            "warning".yellow().bold(),
            names.join(", ")
        );
    }

    // Get generators for implemented languages
    let generators = get_generators(&requested_langs);
    if generators.is_empty() {
        anyhow::bail!(
            "No implemented languages in selection. Currently supported: rust, typescript"
        );
    }

    // Dry-run mode header
    if dry_run {
        println!(
            "{}",
            "🔍 Dry-run mode (no files will be written)\n".cyan().bold()
        );
    }

    // Resolve schema (auto-detects module vs import resolution)
    if !dry_run {
        println!("{:>12} {}", "Reading".cyan().bold(), schema_path.display());
        println!("{:>12} schema and dependencies", "Resolving".cyan().bold());
    }

    let (mut ir, file_count) = resolve_schema(schema_path)?;

    // Report loaded files if multiple
    if file_count > 1 && !dry_run {
        println!("{:>12} {} files", "Loaded".green().bold(), file_count);
    }

    if ir.is_empty() {
        eprintln!(
            "{}: No type definitions found in schema",
            "warning".yellow().bold()
        );
        return Ok(());
    }

    // Handle target mode
    let has_account_attrs = ir.iter().any(|t| match t {
        TypeDefinition::Struct(s) => s.metadata.attributes.contains(&"account".to_string()),
        TypeDefinition::Enum(e) => e.metadata.attributes.contains(&"account".to_string()),
        TypeDefinition::TypeAlias(_) => false,
    });

    match target_mode {
        TargetMode::Native => {
            if has_account_attrs {
                eprintln!(
                    "{}: Schema contains #[account] attributes but target is 'native'",
                    "warning".yellow().bold()
                );
                eprintln!("         Remove #[account] or use --target auto for Anchor integration");
                // Strip account attributes for native mode
                for type_def in &mut ir {
                    match type_def {
                        TypeDefinition::Struct(s) => {
                            s.metadata.attributes.retain(|a| a != "account");
                        }
                        TypeDefinition::Enum(e) => {
                            e.metadata.attributes.retain(|a| a != "account");
                        }
                        TypeDefinition::TypeAlias(_) => {}
                    }
                }
            }
            if !dry_run {
                println!(
                    "{:>12} native Solana mode (pure Borsh)",
                    "Using".cyan().bold()
                );
            }
        }
        TargetMode::Anchor => {
            if !has_account_attrs {
                eprintln!(
                    "{}: Target is 'anchor' but no #[account] attributes found",
                    "warning".yellow().bold()
                );
                eprintln!("         Add #[account] to structs or use --target auto");
            }
        }
        TargetMode::Auto => {
            // Auto-detect: no action needed, generator handles it
        }
    }

    // Generate code for each language
    if !dry_run {
        let lang_names: Vec<_> = generators.iter().map(|g| g.language().name()).collect();
        println!(
            "{:>12} {} code",
            "Generating".green().bold(),
            lang_names.join(", ")
        );
    }

    // Collect generated code for each language
    let generated: Vec<(Language, String, PathBuf)> = generators
        .iter()
        .map(|gen| {
            let code = gen.generate_module(&ir);
            let output_file = output_dir.join(format!("generated.{}", gen.file_extension()));
            (gen.language(), code, output_file)
        })
        .collect();

    // Dry-run mode: preview only
    if dry_run {
        for (lang, code, output_path) in &generated {
            preview_file_changes(output_path, code, lang.name())?;
        }

        println!("\n{}", "No files written (dry-run mode).".yellow());
        println!("Run without --dry-run to apply changes.");
        return Ok(());
    }

    // Backup mode: create backups for all files
    if backup {
        println!("{:>12} files...", "Backing up".cyan().bold());
        for (_, _, output_path) in &generated {
            create_backup_if_exists(output_path)?;
        }
    }

    // Write files for each language
    let mut any_written = false;
    let mut backup_paths: Vec<PathBuf> = Vec::new();

    for (lang, code, output_path) in &generated {
        let written = write_with_diff_check(output_path, code, show_diff, lang.name())?;

        if written {
            println!(
                "{:>12} {}",
                "Wrote".green().bold(),
                output_path.display().to_string().bold()
            );
            any_written = true;

            // Track backup paths
            let backup_ext = format!("{}.backup", lang.file_extension());
            let backup_path = output_path.with_extension(&backup_ext);
            if backup && backup_path.exists() {
                backup_paths.push(backup_path);
            }
        } else if show_diff {
            println!(
                "{:>12} {}",
                "Skipped".yellow().bold(),
                output_path.display().to_string().dimmed()
            );
        }
    }

    // Success summary
    if any_written {
        println!(
            "\n{:>12} generated {} type definitions in {} languages",
            "Finished".green().bold(),
            ir.len(),
            generators.len()
        );
    }

    // Backup restoration hint
    if backup && !backup_paths.is_empty() {
        println!("\n{}", "Backups created. Restore with:".dimmed());
        for backup_path in backup_paths {
            let original = backup_path.with_extension("");
            println!(
                "  mv {} {}",
                backup_path.display().to_string().dimmed(),
                original.display().to_string().dimmed()
            );
        }
    }

    Ok(())
}
