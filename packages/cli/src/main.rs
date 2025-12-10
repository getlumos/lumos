// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! LUMOS CLI - Command-line interface for LUMOS schema code generator

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

use lumos_core::audit_generator::AuditGenerator;
use lumos_core::corpus_generator::CorpusGenerator;
use lumos_core::fuzz_generator::FuzzGenerator;
use lumos_core::generators::{rust, typescript};
use lumos_core::parser::parse_lumos_file;
use lumos_core::security_analyzer::SecurityAnalyzer;
use lumos_core::size_calculator::SizeCalculator;
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
        ///
        /// Debounce duration can be configured via LUMOS_WATCH_DEBOUNCE env var
        /// (default: 100ms, max: 5000ms). Example: LUMOS_WATCH_DEBOUNCE=200
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
        /// Anchor program id to embed in generated Anchor code (required when schema uses Anchor)
        #[arg(long = "address")]
        address: Option<String>,
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

    /// Analyze account sizes and check for Solana limits
    CheckSize {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output format (text or json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Security analysis commands
    Security {
        #[command(subcommand)]
        command: SecurityCommands,
    },

    /// Audit checklist generation commands
    Audit {
        #[command(subcommand)]
        command: AuditCommands,
    },

    /// Fuzz testing commands
    Fuzz {
        #[command(subcommand)]
        command: FuzzCommands,
    },
}

#[derive(Subcommand)]
enum SecurityCommands {
    /// Analyze schema for common Solana vulnerabilities
    Analyze {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output format (text or json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Enable strict mode (more aggressive warnings)
        #[arg(short, long)]
        strict: bool,
    },
}

#[derive(Subcommand)]
enum AuditCommands {
    /// Generate security audit checklist from schema
    Generate {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output file path (default: SECURITY_AUDIT.md)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format (markdown or json)
        #[arg(short, long, default_value = "markdown")]
        format: String,
    },
}

#[derive(Subcommand)]
enum FuzzCommands {
    /// Generate fuzz targets for types
    Generate {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output directory for fuzz targets (default: fuzz/)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Specific type to generate fuzz target for (optional)
        #[arg(short, long)]
        type_name: Option<String>,
    },

    /// Run fuzzing for a specific type
    Run {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Type to fuzz
        #[arg(short, long)]
        type_name: String,

        /// Number of parallel jobs
        #[arg(short, long, default_value = "1")]
        jobs: usize,

        /// Maximum run time in seconds (optional)
        #[arg(short, long)]
        max_time: Option<u64>,
    },

    /// Generate corpus files for fuzzing
    Corpus {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output directory for corpus (default: fuzz/corpus/)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Specific type to generate corpus for (optional)
        #[arg(short, long)]
        type_name: Option<String>,
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
            address,
        } => {
            if watch {
                run_watch_mode(&schema, output.as_deref(), address.as_deref())
            } else {
                run_generate(
                    &schema,
                    output.as_deref(),
                    dry_run,
                    backup,
                    show_diff,
                    address.as_deref(),
                )
            }
        }
        Commands::Validate { schema } => run_validate(&schema),
        Commands::Init { name } => run_init(name.as_deref()),
        Commands::Check { schema, output } => run_check(&schema, output.as_deref()),
        Commands::CheckSize { schema, format } => run_check_size(&schema, &format),
        Commands::Security { command } => match command {
            SecurityCommands::Analyze {
                schema,
                format,
                strict,
            } => run_security_analyze(&schema, &format, strict),
        },
        Commands::Audit { command } => match command {
            AuditCommands::Generate {
                schema,
                output,
                format,
            } => run_audit_generate(&schema, output.as_deref(), &format),
        },
        Commands::Fuzz { command } => match command {
            FuzzCommands::Generate {
                schema,
                output,
                type_name,
            } => run_fuzz_generate(&schema, output.as_deref(), type_name.as_deref()),
            FuzzCommands::Run {
                schema,
                type_name,
                jobs,
                max_time,
            } => run_fuzz_run(&schema, &type_name, jobs, max_time),
            FuzzCommands::Corpus {
                schema,
                output,
                type_name,
            } => run_fuzz_corpus(&schema, output.as_deref(), type_name.as_deref()),
        },
    }
}

/// Generate Rust and TypeScript code from schema
fn run_generate(
    schema_path: &Path,
    output_dir: Option<&Path>,
    dry_run: bool,
    backup: bool,
    show_diff: bool,
    address: Option<&str>,
) -> Result<()> {
    let output_dir = output_dir.unwrap_or_else(|| Path::new("."));

    // Validate output directory for security
    validate_output_path(output_dir)?;

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

    // If generated Rust code uses Anchor, require `--address` to be provided.
    let uses_anchor = rust_code.contains("use anchor_lang::prelude::*");

    let mut rust_code = rust_code;
    if uses_anchor {
        let addr = if let Some(a) = address {
            a.to_string()
        } else {
            anyhow::bail!("--address is required for Anchor code generation. Run: lumos generate <schema> --address <PROGRAM_ID>");
        };

        // Insert declare_id! after the anchor prelude import if possible
        let prelude = "use anchor_lang::prelude::*;";
        if let Some(pos) = rust_code.find(prelude) {
            if let Some(line_end) = rust_code[pos..].find('\n') {
                let insert_at = pos + line_end + 1;
                let decl = format!("\ndeclare_id!(\"{}\");\n\n", addr);
                rust_code.insert_str(insert_at, &decl);
            } else {
                let decl = format!("\n\ndeclare_id!(\"{}\");\n\n", addr);
                rust_code.push_str(&decl);
            }
        } else {
            let decl = format!("declare_id!(\"{}\");\n\n", addr);
            rust_code = format!("{}{}", decl, rust_code);
        }
    }

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

/// Watch mode: regenerate on file changes
fn run_watch_mode(schema_path: &Path, output_dir: Option<&Path>, address: Option<&str>) -> Result<()> {
        use notify::{RecursiveMode, Watcher};
        use std::sync::mpsc::channel;
        use std::time::Duration;
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
        if let Err(e) = run_generate(&schema_path, output_dir, false, false, false, address) {
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

    // Get configurable debounce duration (default: 100ms)
    let debounce_ms = std::env::var("LUMOS_WATCH_DEBOUNCE")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .filter(|&ms| ms <= 5000) // Max 5 seconds
        .unwrap_or(100);

    // Watch for changes
    loop {
        match rx.recv_timeout(Duration::from_millis(debounce_ms)) {
            Ok(_event) => {
                // Debounce: wait a bit for multiple rapid changes
                std::thread::sleep(Duration::from_millis(debounce_ms));

                // Drain any pending events
                while rx.try_recv().is_ok() {}

                println!();
                println!("{:>12} change detected", "Detected".yellow().bold());

                if let Err(e) = run_generate(
                    &schema_path,
                    output_dir_buf.as_deref(),
                    false,
                    false,
                    false,
                    address,
                ) {
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

    inner_watch(schema_path, output_dir, address)
}

/// Check account sizes and detect overflow
fn run_check_size(schema_path: &Path, format: &str) -> Result<()> {
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

    if format == "json" {
        // JSON output for programmatic use
        output_json(&sizes)?;
    } else {
        // Human-readable text output
        output_text(&sizes)?;
    }

    // Exit with error if any account exceeds limits
    let has_errors = sizes.iter().any(|s| !s.warnings.is_empty());
    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}

/// Output sizes in human-readable format
fn output_text(sizes: &[lumos_core::size_calculator::AccountSize]) -> Result<()> {
    use lumos_core::size_calculator::SizeInfo;

    println!("{}", "Account Size Analysis:".bold());
    println!();

    for account in sizes {
        // Account header
        let status = if account.warnings.is_empty() {
            "✓".green()
        } else {
            "⚠".yellow()
        };

        let size_str = match &account.total_bytes {
            SizeInfo::Fixed(bytes) => format!("{} bytes", bytes),
            SizeInfo::Variable { min, .. } => format!("{}+ bytes (variable)", min),
        };

        println!(
            "{} {}: {}",
            status,
            account.name.bold(),
            size_str.cyan()
        );

        // Field breakdown
        for field in &account.field_breakdown {
            let field_size = match &field.size {
                SizeInfo::Fixed(bytes) => format!("{} bytes", bytes),
                SizeInfo::Variable { min, .. } => format!("{}+ bytes", min),
            };

            println!(
                "  {} {} ({}) - {}",
                "├─".dimmed(),
                field.name,
                field_size.dimmed(),
                field.description.dimmed()
            );
        }

        // Total and rent
        println!(
            "  {} Total: {}",
            "└─".dimmed(),
            size_str.bold()
        );
        println!("     Rent: {} SOL", format!("{:.8}", account.rent_sol).cyan());

        // Warnings
        for warning in &account.warnings {
            println!();
            println!("  {} {}", "⚠".yellow(), warning.yellow());
        }

        println!();
    }

    // Summary
    let total_accounts = sizes.len();
    let accounts_with_warnings = sizes.iter().filter(|s| !s.warnings.is_empty()).count();

    println!("{}", "Summary:".bold());
    println!("  Total accounts: {}", total_accounts);

    if accounts_with_warnings > 0 {
        println!(
            "  {} with warnings/errors",
            accounts_with_warnings.to_string().yellow()
        );
    } else {
        println!("  {}", "All accounts within limits ✓".green());
    }

    Ok(())
}

/// Output sizes in JSON format
fn output_json(sizes: &[lumos_core::size_calculator::AccountSize]) -> Result<()> {
    use lumos_core::size_calculator::SizeInfo;
    use serde_json::json;

    let json_data: Vec<_> = sizes
        .iter()
        .map(|account| {
            let (total_bytes, is_variable) = match &account.total_bytes {
                SizeInfo::Fixed(bytes) => (*bytes, false),
                SizeInfo::Variable { min, .. } => (*min, true),
            };

            json!({
                "name": account.name,
                "total_bytes": total_bytes,
                "is_variable": is_variable,
                "is_account": account.is_account,
                "rent_sol": account.rent_sol,
                "warnings": account.warnings,
                "fields": account.field_breakdown.iter().map(|field| {
                    let (bytes, var) = match &field.size {
                        SizeInfo::Fixed(b) => (*b, false),
                        SizeInfo::Variable { min, .. } => (*min, true),
                    };
                    json!({
                        "name": field.name,
                        "bytes": bytes,
                        "is_variable": var,
                        "description": field.description,
                    })
                }).collect::<Vec<_>>(),
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json_data)?);
    Ok(())
}

/// Run security analysis on schema
fn run_security_analyze(schema_path: &Path, format: &str, strict: bool) -> Result<()> {
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

    if format == "json" {
        output_security_json(&findings)?;
    } else {
        output_security_text(&findings, schema_path)?;
    }

    // Exit with error if any critical findings
    let has_critical = findings
        .iter()
        .any(|f| matches!(f.severity, lumos_core::security_analyzer::Severity::Critical));

    if has_critical {
        std::process::exit(1);
    }

    Ok(())
}

/// Output security findings in human-readable format
fn output_security_text(
    findings: &[lumos_core::security_analyzer::SecurityFinding],
    schema_path: &Path,
) -> Result<()> {
    use lumos_core::security_analyzer::Severity;

    println!("{}", "Security Analysis Report".bold());
    println!("Schema: {}", schema_path.display().to_string().cyan());
    println!();

    if findings.is_empty() {
        println!("{}", "✓ No security issues found!".green().bold());
        println!();
        println!("All checks passed. Your schema follows Solana security best practices.");
        return Ok(());
    }

    // Group by severity
    let critical: Vec<_> = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Critical))
        .collect();
    let warnings: Vec<_> = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Warning))
        .collect();
    let info: Vec<_> = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Info))
        .collect();

    // Summary
    println!("{}", "Summary:".bold());
    if !critical.is_empty() {
        println!(
            "  {} {} critical issues",
            "🚨".to_string(),
            critical.len().to_string().red().bold()
        );
    }
    if !warnings.is_empty() {
        println!("  ⚠️  {} warnings", warnings.len().to_string().yellow());
    }
    if !info.is_empty() {
        println!("  ℹ️  {} informational", info.len());
    }
    println!();

    // Critical findings
    if !critical.is_empty() {
        println!("{}", "CRITICAL ISSUES".red().bold());
        println!("{}", "═".repeat(60).red());
        println!();

        for (i, finding) in critical.iter().enumerate() {
            print_finding(finding, i + 1);
        }
    }

    // Warnings
    if !warnings.is_empty() {
        println!("{}", "WARNINGS".yellow().bold());
        println!("{}", "═".repeat(60).yellow());
        println!();

        for (i, finding) in warnings.iter().enumerate() {
            print_finding(finding, i + 1);
        }
    }

    // Info
    if !info.is_empty() {
        println!("{}", "INFORMATIONAL".dimmed().bold());
        println!("{}", "═".repeat(60).dimmed());
        println!();

        for (i, finding) in info.iter().enumerate() {
            print_finding(finding, i + 1);
        }
    }

    // Footer
    println!();
    println!("{}", "Recommendations:".bold());
    if !critical.is_empty() {
        println!(
            "  {} Fix all critical issues before deployment",
            "🚨".red()
        );
    }
    if !warnings.is_empty() {
        println!("  ⚠️  Review and address warnings");
    }
    println!("  📚 See: docs/security/static-analysis.md");

    Ok(())
}

/// Print a single finding
fn print_finding(finding: &lumos_core::security_analyzer::SecurityFinding, _index: usize) {
    use lumos_core::security_analyzer::Severity;

    let emoji = finding.severity.emoji();
    let severity_str = match finding.severity {
        Severity::Critical => finding.severity.as_str().red().bold(),
        Severity::Warning => finding.severity.as_str().yellow().bold(),
        Severity::Info => finding.severity.as_str().dimmed().bold(),
    };

    println!(
        "{} [{}] {}",
        emoji,
        severity_str,
        finding.vulnerability.as_str().bold()
    );

    // Location
    let location = if let Some(ref field) = finding.location.field_name {
        format!("{}::{}", finding.location.type_name, field)
    } else {
        finding.location.type_name.clone()
    };
    println!("   Location: {}", location.cyan());

    // Message
    println!("   {}", finding.message);

    // Suggestion
    println!("   💡 {}", finding.suggestion.dimmed());

    println!();
}

/// Output security findings in JSON format
fn output_security_json(findings: &[lumos_core::security_analyzer::SecurityFinding]) -> Result<()> {
    use serde_json::json;

    let json_data: Vec<_> = findings
        .iter()
        .map(|finding| {
            json!({
                "severity": finding.severity.as_str(),
                "vulnerability_type": finding.vulnerability.as_str(),
                "location": {
                    "type_name": finding.location.type_name,
                    "field_name": finding.location.field_name,
                },
                "message": finding.message,
                "suggestion": finding.suggestion,
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&json_data)?);
    Ok(())
}

/// Run audit checklist generation
fn run_audit_generate(schema_path: &Path, output_path: Option<&Path>, format: &str) -> Result<()> {
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
    if format == "json" {
        generate_audit_json(&checklist, output)?;
    } else {
        generate_audit_markdown(&checklist, schema_path, output)?;
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
    use lumos_core::audit_generator::CheckCategory;
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

/// Generate audit checklist in Markdown format
fn generate_audit_markdown(
    checklist: &[lumos_core::audit_generator::ChecklistItem],
    schema_path: &Path,
    output_path: &Path,
) -> Result<()> {
    use lumos_core::audit_generator::CheckCategory;
    use std::collections::HashMap;

    let mut content = String::new();

    // Header
    content.push_str("# Security Audit Checklist\n\n");
    content.push_str(&format!(
        "**Generated from:** `{}`\n",
        schema_path.display()
    ));
    content.push_str(&format!("**Date:** {}\n\n", chrono::Local::now().format("%Y-%m-%d")));
    content.push_str(&format!("**Total Checks:** {}\n\n", checklist.len()));

    content.push_str("---\n\n");
    content.push_str("## ⚠️ Important Disclaimer\n\n");
    content.push_str("**This automated checklist is a supplementary tool and does NOT replace professional security audits.**\n\n");
    content.push_str("- Generated checklists identify common vulnerability patterns based on schema structure\n");
    content.push_str("- They cannot detect logic bugs, business logic flaws, or complex attack vectors\n");
    content.push_str("- **Always conduct thorough manual code review and professional security audits** before deploying to production\n");
    content.push_str("- Consider engaging professional security auditors for mainnet deployments\n\n");
    content.push_str("---\n\n");
    content.push_str("## How to Use This Checklist\n\n");
    content.push_str("- [ ] = Not checked yet\n");
    content.push_str("- [x] = Verified and compliant\n");
    content.push_str("- Priority: 🔴 CRITICAL | 🟡 HIGH | 🟢 MEDIUM | ⚪ LOW\n\n");
    content.push_str("**Review each item during your security audit and check the box when verified.**\n\n");

    content.push_str("---\n\n");

    // Group by category
    let mut by_category: HashMap<CheckCategory, Vec<&lumos_core::audit_generator::ChecklistItem>> =
        HashMap::new();

    for item in checklist {
        by_category
            .entry(item.category.clone())
            .or_default()
            .push(item);
    }

    // Output each category
    let category_order = [
        CheckCategory::AccountValidation,
        CheckCategory::SignerChecks,
        CheckCategory::AccessControl,
        CheckCategory::ArithmeticSafety,
        CheckCategory::DataValidation,
        CheckCategory::StateTransition,
        CheckCategory::Initialization,
        CheckCategory::RentExemption,
    ];

    for category in category_order {
        if let Some(items) = by_category.get(&category) {
            content.push_str(&format!(
                "## {} {}\n\n",
                category.emoji(),
                category.as_str()
            ));

            for item in items {
                let priority_icon = match item.priority {
                    lumos_core::audit_generator::Priority::Critical => "🔴",
                    lumos_core::audit_generator::Priority::High => "🟡",
                    lumos_core::audit_generator::Priority::Medium => "🟢",
                    lumos_core::audit_generator::Priority::Low => "⚪",
                };

                content.push_str(&format!("- [ ] {} **{}**\n", priority_icon, item.item));
                content.push_str(&format!("  - Context: `{}`\n", item.context));
                content.push_str(&format!("  - {}\n\n", item.explanation));
            }
        }
    }

    // Footer
    content.push_str("---\n\n");
    content.push_str("## Additional Security Considerations\n\n");
    content.push_str("- [ ] **Program Logic:** Verify business logic correctness\n");
    content.push_str("- [ ] **Error Handling:** Ensure all error paths are covered\n");
    content.push_str("- [ ] **Testing:** Comprehensive test suite including edge cases\n");
    content.push_str("- [ ] **Documentation:** Code is well-documented\n");
    content.push_str("- [ ] **Dependencies:** All dependencies are audited and up-to-date\n\n");

    content.push_str("---\n\n");
    content.push_str("**Audit Status:**\n\n");
    content.push_str("- Auditor: _________________\n");
    content.push_str("- Date Started: _________________\n");
    content.push_str("- Date Completed: _________________\n");
    content.push_str("- Findings: _________________\n\n");

    fs::write(output_path, content)
        .with_context(|| format!("Failed to write checklist to {}", output_path.display()))?;

    Ok(())
}

/// Generate audit checklist in JSON format
fn generate_audit_json(
    checklist: &[lumos_core::audit_generator::ChecklistItem],
    output_path: &Path,
) -> Result<()> {
    use serde_json::json;

    let json_data: Vec<_> = checklist
        .iter()
        .map(|item| {
            json!({
                "category": item.category.as_str(),
                "priority": item.priority.as_str(),
                "item": item.item,
                "context": item.context,
                "explanation": item.explanation,
                "checked": false,
            })
        })
        .collect();

    let output = serde_json::to_string_pretty(&json_data)?;
    fs::write(output_path, output)
        .with_context(|| format!("Failed to write checklist to {}", output_path.display()))?;

    Ok(())
}

/// Generate fuzz targets from schema
fn run_fuzz_generate(
    schema_path: &Path,
    output_dir: Option<&Path>,
    type_name: Option<&str>,
) -> Result<()> {
    let output_dir = output_dir.unwrap_or_else(|| Path::new("fuzz"));

    println!(
        "{:>12} {}",
        "Generating".cyan().bold(),
        "fuzz targets..."
    );

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

    println!(
        "{:>12} {}",
        "Created".green().bold(),
        readme_path.display()
    );

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
    println!("  1. Install cargo-fuzz: {}", "cargo install cargo-fuzz".yellow());
    println!("  2. Run fuzzing: {}", format!("cd {} && cargo fuzz run {}", fuzz_dir.display(), targets[0].name).yellow());

    Ok(())
}

/// Run fuzzing for a specific type
fn run_fuzz_run(
    schema_path: &Path,
    type_name: &str,
    jobs: usize,
    max_time: Option<u64>,
) -> Result<()> {
    println!(
        "{:>12} {} for type '{}'",
        "Running".cyan().bold(),
        "fuzzer",
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
    use std::process::Command;

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
fn run_fuzz_corpus(
    schema_path: &Path,
    output_dir: Option<&Path>,
    type_name: Option<&str>,
) -> Result<()> {
    let output_dir = output_dir.unwrap_or_else(|| Path::new("fuzz/corpus"));

    println!(
        "{:>12} {}",
        "Generating".cyan().bold(),
        "corpus files..."
    );

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
            format!("Failed to create directory: {}", target_corpus_dir.display())
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

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else {
            result.push(ch);
            prev_is_upper = false;
        }
    }

    result
}

/// Validate output path for security and accessibility
///
/// This function prevents path traversal attacks and ensures the path
/// is writable before attempting file operations.
///
/// # Security Checks
///
/// 1. **Path Canonicalization** - Resolves `..`, `.`, and symlinks
/// 2. **Directory Existence** - Ensures parent directory exists
/// 3. **Write Permissions** - Verifies write access to the directory
///
/// # Arguments
///
/// * `path` - Output path to validate
///
/// # Returns
///
/// * `Ok(())` - Path is valid and writable
/// * `Err(anyhow::Error)` - Path is invalid or not writable
///
/// # Examples
///
/// ```rust,ignore
/// // Valid paths
/// validate_output_path(Path::new("./output"))?;
/// validate_output_path(Path::new("."))?;
///
/// // Invalid paths (would fail)
/// validate_output_path(Path::new("../../etc"))?;  // Path traversal
/// validate_output_path(Path::new("/root"))?;      // No write permission
/// ```
fn validate_output_path(path: &Path) -> Result<()> {
    // If path doesn't exist, check parent directory
    let check_path = if path.exists() {
        path
    } else if let Some(parent) = path.parent() {
        // If parent doesn't exist, we can't validate write permissions
        if !parent.exists() {
            anyhow::bail!(
                "Output directory parent does not exist: {}. Create it first.",
                parent.display()
            );
        }
        parent
    } else {
        // No parent means root directory or invalid path
        anyhow::bail!("Invalid output path: {}", path.display());
    };

    // Check if path is absolute or can be canonicalized
    let canonical = check_path
        .canonicalize()
        .with_context(|| format!("Cannot resolve output path: {}", path.display()))?;

    // Verify the canonical path is writable
    // Try to create a temporary file to test write permissions
    let test_file = canonical.join(".lumos_write_test");
    match fs::write(&test_file, "") {
        Ok(_) => {
            // Clean up test file
            let _ = fs::remove_file(&test_file);
            Ok(())
        }
        Err(e) => {
            anyhow::bail!(
                "Output directory is not writable: {}\nError: {}",
                canonical.display(),
                e
            );
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_schema(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("temp file");
        write!(file, "{}", content).expect("write schema");
        file
    }

    #[test]
    fn anchor_generate_noninteractive_requires_address() {
        // Schema that triggers Anchor usage
        let schema = r#"#[solana]
#[account]
struct Foo { id: u64 }
"#;

        let file = write_schema(schema);

        // Missing address should cause an error in strict mode
        let res = run_generate(
            file.path(),
            Some(Path::new(".")),
            true,  // dry_run
            false, // backup
            false, // show_diff
            None,  // address
        );

        assert!(res.is_err(), "Expected error when address is not provided in strict mode");
    }

    #[test]
    fn anchor_generate_with_address_succeeds_dry_run() {
        let schema = r#"#[solana]
#[account]
struct Foo { id: u64 }
"#;

        let file = write_schema(schema);
        let res = run_generate(
            file.path(),
            Some(Path::new(".")),
            true,                      // dry_run
            false,                     // backup
            false,                     // show_diff
            Some("5Hj3...xyz"), // address
        );

        assert!(res.is_ok(), "Expected success when address provided");
    }

    #[test]
    fn anchor_generate_writes_declare_id_with_address() {
        use tempfile::tempdir;

        // Create temp dir for output
        let dir = tempdir().expect("tempdir");
        let out = dir.path();

        // Schema that triggers Anchor usage
        let schema = r#"#[solana]
#[account]
struct Foo { id: u64 }
"#;

        let schema_file = write_schema(schema);

        // Run generation with explicit address and write files
        let res = run_generate(
            schema_file.path(),
            Some(out),
            false,                    // dry_run = false -> writes files
            false,                    // backup
            false,                    // show_diff
            Some("5Hj3SomeValidAddrXyz"),
        );

        assert!(res.is_ok(), "Generation should succeed when address provided");

        // Read generated file and verify declare_id! inserted
        let gen_path = out.join("generated.rs");
        let contents = std::fs::read_to_string(&gen_path).expect("read generated.rs");
        assert!(contents.contains("declare_id!(\"5Hj3SomeValidAddrXyz\")"), "declare_id not found");
    }

    #[test]
    fn anchor_generate_writes_sentinel_when_env_override() {
        use tempfile::tempdir;

        // Create temp dir for output
        let dir = tempdir().expect("tempdir");
        let out = dir.path();

        // Schema that triggers Anchor usage
        let schema = r#"#[solana]
#[account]
struct Foo { id: u64 }
"#;

        let schema_file = write_schema(schema);

        // Run generation using explicit sentinel address
        let res = run_generate(
            schema_file.path(),
            Some(out),
            false, // dry_run = false
            false,
            false,
            Some("REPLACE_WITH_YOUR_PROGRAM_ID"),
        );

        assert!(res.is_ok(), "Generation should succeed with explicit sentinel address");

        let gen_path = out.join("generated.rs");
        let contents = std::fs::read_to_string(&gen_path).expect("read generated.rs");
        assert!(contents.contains("declare_id!(\"REPLACE_WITH_YOUR_PROGRAM_ID\")"), "sentinel not found");
    }
}
