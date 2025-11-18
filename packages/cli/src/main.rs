// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! LUMOS CLI - Command-line interface for LUMOS schema code generator

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

use lumos_core::generators::{rust, typescript};
use lumos_core::parser::parse_lumos_file;
use lumos_core::transform::transform_to_ir;

#[derive(Parser)]
#[command(name = "lumos")]
#[command(about = "Type-safe schema language for Solana development", long_about = None)]
#[command(version)]
#[command(author)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Rust and TypeScript code from schema
    Generate {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Watch for changes and regenerate automatically
        #[arg(short, long)]
        watch: bool,

        /// Preview changes without writing files
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Create backup before overwriting existing files
        #[arg(short = 'b', long)]
        backup: bool,

        /// Show diff and ask for confirmation before writing
        #[arg(short = 'd', long)]
        show_diff: bool,
    },

    /// Validate schema syntax without generating code
    Validate {
        /// Path to .lumos schema file
        schema: PathBuf,
    },

    /// Initialize a new LUMOS project
    Init {
        /// Project name (optional, defaults to current directory)
        name: Option<String>,
    },

    /// Check if generated code is up-to-date
    Check {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            schema,
            output,
            watch,
            dry_run,
            backup,
            show_diff,
        } => {
            if watch {
                run_watch_mode(&schema, output.as_deref())
            } else {
                run_generate(&schema, output.as_deref(), dry_run, backup, show_diff)
            }
        }
        Commands::Validate { schema } => run_validate(&schema),
        Commands::Init { name } => run_init(name.as_deref()),
        Commands::Check { schema, output } => run_check(&schema, output.as_deref()),
    }
}

/// Generate Rust and TypeScript code from schema
fn run_generate(
    schema_path: &Path,
    output_dir: Option<&Path>,
    dry_run: bool,
    backup: bool,
    show_diff: bool,
) -> Result<()> {
    let output_dir = output_dir.unwrap_or_else(|| Path::new("."));

    // Dry-run mode header
    if dry_run {
        println!(
            "{}",
            "🔍 Dry-run mode (no files will be written)\n".cyan().bold()
        );
    }

    // Read schema file
    if !dry_run {
        println!("{:>12} {}", "Reading".cyan().bold(), schema_path.display());
    }

    let content = fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;

    // Parse schema
    if !dry_run {
        println!("{:>12} schema", "Parsing".cyan().bold());
    }

    let ast = parse_lumos_file(&content)
        .with_context(|| format!("Failed to parse schema: {}", schema_path.display()))?;

    // Transform to IR
    let ir = transform_to_ir(ast).with_context(|| "Failed to transform AST to IR")?;

    if ir.is_empty() {
        eprintln!(
            "{}: No type definitions found in schema",
            "warning".yellow().bold()
        );
        return Ok(());
    }

    // Generate code
    if !dry_run {
        println!("{:>12} code", "Generating".green().bold());
    }

    let rust_code = rust::generate_module(&ir);
    let ts_code = typescript::generate_module(&ir);

    let rust_output = output_dir.join("generated.rs");
    let ts_output = output_dir.join("generated.ts");

    // Dry-run mode: preview only
    if dry_run {
        preview_file_changes(&rust_output, &rust_code, "Rust")?;
        preview_file_changes(&ts_output, &ts_code, "TypeScript")?;

        println!("\n{}", "No files written (dry-run mode).".yellow());
        println!("Run without --dry-run to apply changes.");
        return Ok(());
    }

    // Backup mode: create backups
    if backup {
        println!("{:>12} files...", "Backing up".cyan().bold());
        create_backup_if_exists(&rust_output)?;
        create_backup_if_exists(&ts_output)?;
    }

    // Write Rust file
    let rust_written = write_with_diff_check(&rust_output, &rust_code, show_diff, "Rust")?;

    if rust_written {
        println!(
            "{:>12} {}",
            "Wrote".green().bold(),
            rust_output.display().to_string().bold()
        );
    } else if show_diff {
        println!(
            "{:>12} {}",
            "Skipped".yellow().bold(),
            rust_output.display().to_string().dimmed()
        );
    }

    // Write TypeScript file
    let ts_written = write_with_diff_check(&ts_output, &ts_code, show_diff, "TypeScript")?;

    if ts_written {
        println!(
            "{:>12} {}",
            "Wrote".green().bold(),
            ts_output.display().to_string().bold()
        );
    } else if show_diff {
        println!(
            "{:>12} {}",
            "Skipped".yellow().bold(),
            ts_output.display().to_string().dimmed()
        );
    }

    // Success summary
    if rust_written || ts_written {
        println!(
            "\n{:>12} generated {} type definitions",
            "Finished".green().bold(),
            ir.len()
        );
    }

    // Backup restoration hint
    if backup && (rust_written || ts_written) {
        println!("\n{}", "Backups created. Restore with:".dimmed());
        if rust_written && rust_output.with_extension("rs.backup").exists() {
            println!(
                "  mv {} {}",
                rust_output
                    .with_extension("rs.backup")
                    .display()
                    .to_string()
                    .dimmed(),
                rust_output.display().to_string().dimmed()
            );
        }
        if ts_written && ts_output.with_extension("ts.backup").exists() {
            println!(
                "  mv {} {}",
                ts_output
                    .with_extension("ts.backup")
                    .display()
                    .to_string()
                    .dimmed(),
                ts_output.display().to_string().dimmed()
            );
        }
    }

    Ok(())
}

/// Preview file changes in dry-run mode
fn preview_file_changes(path: &Path, new_content: &str, label: &str) -> Result<()> {
    let new_lines = new_content.lines().count();
    let new_size = new_content.len();

    println!(
        "Would generate: {} ({})",
        path.display().to_string().bold(),
        label.cyan()
    );
    println!(
        "  Size: {} lines ({:.1} KB)",
        new_lines,
        new_size as f64 / 1024.0
    );

    if path.exists() {
        let old_content = fs::read_to_string(path)?;
        let old_lines = old_content.lines().count();

        if new_content == old_content {
            println!("  {}", "No changes (identical to existing)".dimmed());
        } else {
            let added = new_lines.saturating_sub(old_lines);
            let removed = old_lines.saturating_sub(new_lines);

            if added > 0 {
                println!("  {} {} lines", "+".green(), added);
            }
            if removed > 0 {
                println!("  {} {} lines", "-".red(), removed);
            }
            if added == 0 && removed == 0 {
                println!("  {} content modified", "~".yellow());
            }
        }
    } else {
        println!("  {}", "New file (doesn't exist yet)".green());
    }

    println!();
    Ok(())
}

/// Create backup of file if it exists
fn create_backup_if_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let backup_path = path.with_extension(format!(
        "{}.backup",
        path.extension().and_then(|s| s.to_str()).unwrap_or("")
    ));

    fs::copy(path, &backup_path)
        .with_context(|| format!("Failed to create backup: {}", backup_path.display()))?;

    println!(
        "  {} → {}",
        path.display().to_string().dimmed(),
        backup_path.display().to_string().cyan()
    );

    Ok(())
}

/// Write file with optional diff check and confirmation
fn write_with_diff_check(path: &Path, content: &str, show_diff: bool, label: &str) -> Result<bool> {
    // If show_diff and file exists, show diff and ask for confirmation
    if show_diff && path.exists() {
        let old_content = fs::read_to_string(path)?;

        // If identical, skip
        if content == old_content {
            println!(
                "{}: {} {}",
                "Unchanged".dimmed(),
                path.display().to_string().dimmed(),
                format!("({})", label).dimmed()
            );
            return Ok(false);
        }

        // Show diff
        show_diff_and_ask_confirmation(path, &old_content, content, label)?;

        // User declined
        return Ok(false);
    }

    // Write file
    fs::write(path, content)
        .with_context(|| format!("Failed to write {}: {}", label, path.display()))?;

    Ok(true)
}

/// Show diff and ask for user confirmation
fn show_diff_and_ask_confirmation(
    path: &Path,
    old_content: &str,
    new_content: &str,
    label: &str,
) -> Result<()> {
    use std::io::{self, Write};

    println!("\n{}", "─".repeat(60).dimmed());
    println!(
        "DIFF: {} ({})",
        path.display().to_string().bold(),
        label.cyan()
    );
    println!("{}", "─".repeat(60).dimmed());
    println!();

    // Simple line-by-line diff
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    let mut added = 0;
    let mut removed = 0;
    let max_lines = old_lines.len().max(new_lines.len());

    // Show first 20 lines of diff
    let preview_limit = 20;
    for i in 0..max_lines.min(preview_limit) {
        let old_line = old_lines.get(i);
        let new_line = new_lines.get(i);

        match (old_line, new_line) {
            (Some(old), Some(new)) if old != new => {
                println!("{} {}", "-".red(), old);
                println!("{} {}", "+".green(), new);
                added += 1;
                removed += 1;
            }
            (Some(old), None) => {
                println!("{} {}", "-".red(), old);
                removed += 1;
            }
            (None, Some(new)) => {
                println!("{} {}", "+".green(), new);
                added += 1;
            }
            (Some(line), Some(_)) => {
                println!("  {}", line.dimmed());
            }
            _ => {}
        }
    }

    if max_lines > preview_limit {
        println!(
            "\n{}",
            format!("... ({} more lines)", max_lines - preview_limit).dimmed()
        );
    }

    println!();
    println!("Summary:");
    if added > 0 {
        println!("  Lines added: {}", added.to_string().green());
    }
    if removed > 0 {
        println!("  Lines removed: {}", removed.to_string().red());
    }
    println!();

    // Ask for confirmation
    print!("Apply changes to {}? [y/N] ", path.display());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    if response == "y" || response == "yes" {
        fs::write(path, new_content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        println!("{:>12} {}", "Applied".green().bold(), path.display());
        Ok(())
    } else {
        println!("{:>12} {}", "Skipped".yellow().bold(), path.display());
        Ok(())
    }
}

/// Validate schema syntax without generating code
fn run_validate(schema_path: &Path) -> Result<()> {
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

/// Initialize a new LUMOS project
fn run_init(project_name: Option<&str>) -> Result<()> {
    let project_dir = if let Some(name) = project_name {
        println!("{:>12} project: {}", "Creating".cyan().bold(), name.bold());
        let dir = PathBuf::from(name);
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create project directory: {}", name))?;
        dir
    } else {
        println!("{:>12} current directory", "Initializing".cyan().bold());
        PathBuf::from(".")
    };

    // Create example schema
    let schema_content = r#"// Example LUMOS schema
// Edit this file and run: lumos generate schema.lumos

#[solana]
#[account]
struct UserAccount {
    owner: PublicKey,
    balance: u64,
    created_at: i64,
}
"#;

    let schema_path = project_dir.join("schema.lumos");
    fs::write(&schema_path, schema_content)
        .with_context(|| format!("Failed to write schema file: {}", schema_path.display()))?;

    println!(
        "{:>12} {}",
        "Created".green().bold(),
        schema_path.display().to_string().bold()
    );

    // Create lumos.toml config
    let config_content = r#"# LUMOS Configuration File

[output]
# Output directory for generated files (relative to this file)
directory = "."

# Rust output file name
rust = "generated.rs"

# TypeScript output file name
typescript = "generated.ts"
"#;

    let config_path = project_dir.join("lumos.toml");
    fs::write(&config_path, config_content)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

    println!(
        "{:>12} {}",
        "Created".green().bold(),
        config_path.display().to_string().bold()
    );

    // Create README
    let readme_content = r#"# LUMOS Project

## Quick Start

1. Edit `schema.lumos` to define your data structures
2. Generate code:
   ```bash
   lumos generate schema.lumos
   ```
3. Use the generated `generated.rs` and `generated.ts` in your project

## Commands

- `lumos generate schema.lumos` - Generate Rust + TypeScript code
- `lumos validate schema.lumos` - Validate schema syntax
- `lumos generate schema.lumos --watch` - Watch for changes
- `lumos check schema.lumos` - Verify generated code is up-to-date

## Documentation

https://github.com/RECTOR-LABS/lumos
"#;

    let readme_path = project_dir.join("README.md");
    fs::write(&readme_path, readme_content)
        .with_context(|| format!("Failed to write README: {}", readme_path.display()))?;

    println!(
        "{:>12} {}",
        "Created".green().bold(),
        readme_path.display().to_string().bold()
    );

    // Success message
    println!();
    println!("{:>12} project initialized", "Finished".green().bold());
    println!();
    println!("Next steps:");
    if project_name.is_some() {
        println!("  cd {}", project_name.unwrap());
    }
    println!("  lumos generate schema.lumos");

    Ok(())
}

/// Check if generated code is up-to-date
fn run_check(schema_path: &Path, output_dir: Option<&Path>) -> Result<()> {
    let output_dir = output_dir.unwrap_or_else(|| Path::new("."));

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

/// Watch mode: regenerate on file changes
fn run_watch_mode(schema_path: &Path, output_dir: Option<&Path>) -> Result<()> {
    use notify::{RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    let schema_path = schema_path.to_path_buf();
    let output_dir_buf = output_dir.map(|p| p.to_path_buf());

    println!(
        "{:>12} {} for changes...",
        "Watching".cyan().bold(),
        schema_path.display()
    );
    println!("Press Ctrl+C to stop");
    println!();

    // Initial generation (no safety flags in watch mode)
    if let Err(e) = run_generate(&schema_path, output_dir, false, false, false) {
        eprintln!("{}: {}", "error".red().bold(), e);
    }

    // Set up file watcher
    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;

    watcher.watch(&schema_path, RecursiveMode::NonRecursive)?;

    // Watch for changes
    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(_event) => {
                // Debounce: wait a bit for multiple rapid changes
                std::thread::sleep(Duration::from_millis(100));

                // Drain any pending events
                while rx.try_recv().is_ok() {}

                println!();
                println!("{:>12} change detected", "Detected".yellow().bold());

                if let Err(e) =
                    run_generate(&schema_path, output_dir_buf.as_deref(), false, false, false)
                {
                    eprintln!("{}: {}", "error".red().bold(), e);
                }

                println!();
                println!("{:>12} for changes...", "Watching".cyan().bold());
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Normal timeout, continue watching
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    Ok(())
}
