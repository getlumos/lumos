// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Fuzz testing commands - generate fuzz targets, run fuzzing, and create corpus files

use anyhow::{Context, Result};
use colored::Colorize;
use lumos_core::corpus_generator::CorpusGenerator;
use lumos_core::fuzz_generator::FuzzGenerator;
use lumos_core::parser::parse_lumos_file;
use lumos_core::transform::transform_to_ir;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::utils::to_snake_case;

/// Generate fuzz targets from schema
pub fn run_generate(
    schema_path: &Path,
    output_dir: Option<&Path>,
    type_name: Option<&str>,
) -> Result<()> {
    let output_dir = output_dir.unwrap_or_else(|| Path::new("fuzz"));

    println!("{:>12} fuzz targets...", "Generating".cyan().bold());

    // Read and parse schema
    let source = fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;

    let ast = parse_lumos_file(&source)?;
    let ir = transform_to_ir(ast)?;

    let generator = FuzzGenerator::new(&ir);

    // Filter by type if specified
    let targets: Vec<_> = if let Some(name) = type_name {
        if !generator.type_exists(name) {
            anyhow::bail!("Type '{}' not found in schema", name);
        }
        generator
            .generate_all()
            .into_iter()
            .filter(|t| t.type_name == name)
            .collect()
    } else {
        generator.generate_all()
    };

    if targets.is_empty() {
        println!("{}", "⚠ No types found in schema".yellow());
        return Ok(());
    }

    // Create directory structure
    let fuzz_dir = output_dir;
    let fuzz_targets_dir = fuzz_dir.join("fuzz_targets");

    fs::create_dir_all(&fuzz_targets_dir)
        .with_context(|| format!("Failed to create directory: {}", fuzz_targets_dir.display()))?;

    // Generate Cargo.toml
    let cargo_toml_path = fuzz_dir.join("Cargo.toml");
    let cargo_toml = generator.generate_cargo_toml("generated");
    fs::write(&cargo_toml_path, cargo_toml)
        .with_context(|| format!("Failed to write {}", cargo_toml_path.display()))?;

    println!(
        "{:>12} {}",
        "Created".green().bold(),
        cargo_toml_path.display()
    );

    // Generate README
    let readme_path = fuzz_dir.join("README.md");
    let readme = generator.generate_readme();
    fs::write(&readme_path, readme)
        .with_context(|| format!("Failed to write {}", readme_path.display()))?;

    println!("{:>12} {}", "Created".green().bold(), readme_path.display());

    // Generate fuzz targets
    for target in &targets {
        let target_path = fuzz_targets_dir.join(format!("{}.rs", target.name));
        fs::write(&target_path, &target.code)
            .with_context(|| format!("Failed to write {}", target_path.display()))?;

        println!(
            "{:>12} {} (for {})",
            "Generated".green().bold(),
            target_path.display(),
            target.type_name
        );
    }

    println!(
        "\n{} Generated {} fuzz target{}",
        "✓".green().bold(),
        targets.len(),
        if targets.len() == 1 { "" } else { "s" }
    );

    println!("\n{}", "Next steps:".cyan().bold());
    println!(
        "  1. Install cargo-fuzz: {}",
        "cargo install cargo-fuzz".yellow()
    );
    println!(
        "  2. Run fuzzing: {}",
        format!(
            "cd {} && cargo fuzz run {}",
            fuzz_dir.display(),
            targets[0].name
        )
        .yellow()
    );

    Ok(())
}

/// Run fuzzing for a specific type
pub fn run_fuzz(
    schema_path: &Path,
    type_name: &str,
    jobs: usize,
    max_time: Option<u64>,
) -> Result<()> {
    println!(
        "{:>12} fuzzer for type '{}'",
        "Running".cyan().bold(),
        type_name
    );

    // Read and parse schema to verify type exists
    let source = fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;

    let ast = parse_lumos_file(&source)?;
    let ir = transform_to_ir(ast)?;

    let generator = FuzzGenerator::new(&ir);

    if !generator.type_exists(type_name) {
        anyhow::bail!("Type '{}' not found in schema", type_name);
    }

    // Convert type name to fuzz target name
    let target_name = format!("fuzz_{}", to_snake_case(type_name));

    // Build cargo-fuzz command
    let mut args = vec!["fuzz", "run", &target_name];

    // Add arguments
    let mut extra_args = vec![];

    if jobs > 1 {
        extra_args.push(format!("-jobs={}", jobs));
    }

    if let Some(time) = max_time {
        extra_args.push(format!("-max_total_time={}", time));
    }

    if !extra_args.is_empty() {
        args.push("--");
        for arg in &extra_args {
            args.push(arg);
        }
    }

    println!(
        "{:>12} {}",
        "Executing".cyan().bold(),
        format!("cargo {}", args.join(" ")).yellow()
    );

    // Execute cargo-fuzz
    let status = Command::new("cargo")
        .args(&args)
        .current_dir("fuzz")
        .status()
        .with_context(|| "Failed to run cargo-fuzz. Is it installed? (cargo install cargo-fuzz)")?;

    if !status.success() {
        anyhow::bail!("Fuzzing failed with exit code: {}", status);
    }

    println!("{}", "✓ Fuzzing completed".green().bold());

    Ok(())
}

/// Generate corpus files for fuzzing
pub fn run_corpus(
    schema_path: &Path,
    output_dir: Option<&Path>,
    type_name: Option<&str>,
) -> Result<()> {
    let output_dir = output_dir.unwrap_or_else(|| Path::new("fuzz/corpus"));

    println!("{:>12} corpus files...", "Generating".cyan().bold());

    // Read and parse schema
    let source = fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;

    let ast = parse_lumos_file(&source)?;
    let ir = transform_to_ir(ast)?;

    let generator = CorpusGenerator::new(&ir);

    // Filter by type if specified
    let corpus_files: Vec<_> = if let Some(name) = type_name {
        generator
            .generate_all()
            .into_iter()
            .filter(|c| c.type_name == name)
            .collect()
    } else {
        generator.generate_all()
    };

    if corpus_files.is_empty() {
        println!("{}", "⚠ No corpus files generated".yellow());
        return Ok(());
    }

    // Create corpus directory structure
    // Organize by type: fuzz/corpus/{target_name}/...
    for file in &corpus_files {
        let target_name = format!("fuzz_{}", to_snake_case(&file.type_name));
        let target_corpus_dir = output_dir.join(&target_name);

        fs::create_dir_all(&target_corpus_dir).with_context(|| {
            format!(
                "Failed to create directory: {}",
                target_corpus_dir.display()
            )
        })?;

        let file_path = target_corpus_dir.join(&file.name);
        fs::write(&file_path, &file.data)
            .with_context(|| format!("Failed to write {}", file_path.display()))?;

        println!(
            "{:>12} {} ({} bytes) - {}",
            "Created".green().bold(),
            file_path.display(),
            file.data.len(),
            file.description
        );
    }

    println!(
        "\n{} Generated {} corpus file{}",
        "✓".green().bold(),
        corpus_files.len(),
        if corpus_files.len() == 1 { "" } else { "s" }
    );

    Ok(())
}
