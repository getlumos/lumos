// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! LUMOS CLI - Command-line interface for LUMOS schema code generator

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

mod commands;
mod utils;
use crate::utils::*;

use lumos_core::anchor::{
    generate_accounts_context, parse_anchor_attrs, IdlGenerator,
    IdlGeneratorConfig, InstructionAccount, InstructionContext,
};
use lumos_core::audit_generator::AuditGenerator;
use lumos_core::corpus_generator::CorpusGenerator;
use lumos_core::fuzz_generator::FuzzGenerator;
use lumos_core::generators::typescript;
use lumos_core::ir::TypeDefinition;
use lumos_core::metaplex::{MetaplexGenerator, MetaplexValidator, Severity};
use lumos_core::parser::parse_lumos_file;
use lumos_core::security_analyzer::SecurityAnalyzer;
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
    /// Generate code from schema in multiple languages
    Generate {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Target languages (comma-separated: rust,typescript,python,go,ruby)
        ///
        /// Supported: rust (rs), typescript (ts)
        /// Planned: python (py), go, ruby (rb)
        ///
        /// Default: rust,typescript
        #[arg(short = 'l', long, default_value = "rust,typescript")]
        lang: String,

        /// Target framework for code generation
        ///
        /// - auto: Detect based on #[account] attribute (default)
        /// - native: Force pure Borsh, no Anchor dependencies
        /// - anchor: Use Anchor framework (requires #[account])
        #[arg(short = 't', long, default_value = "auto")]
        target: String,

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

    /// Compare two schema files and show differences
    Diff {
        /// Path to first .lumos schema file (v1)
        schema1: PathBuf,

        /// Path to second .lumos schema file (v2)
        schema2: PathBuf,

        /// Output format (text or json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Generate migration code from one schema version to another
    Migrate {
        /// Path to old .lumos schema file (v1)
        from_schema: PathBuf,

        /// Path to new .lumos schema file (v2)
        to_schema: PathBuf,

        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Generate migration code for specific language (rust, typescript, or both)
        #[arg(short = 'l', long, default_value = "both")]
        language: String,

        /// Dry run (show changes without generating code)
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Force generation even for unsafe migrations
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Check backward compatibility between two schema versions
    CheckCompat {
        /// Path to old .lumos schema file (v1)
        from_schema: PathBuf,

        /// Path to new .lumos schema file (v2)
        to_schema: PathBuf,

        /// Output format (text or json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Show verbose output with detailed explanations
        #[arg(short, long)]
        verbose: bool,

        /// Fail on warnings (treat warnings as errors)
        #[arg(short = 's', long)]
        strict: bool,
    },

    /// Anchor Framework integration commands
    Anchor {
        #[command(subcommand)]
        command: AnchorCommands,
    },

    /// Metaplex Token Metadata integration commands
    Metaplex {
        #[command(subcommand)]
        command: MetaplexCommands,
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

#[derive(Subcommand)]
enum AnchorCommands {
    /// Generate complete Anchor program from LUMOS schema
    ///
    /// Generates:
    /// - Rust program with #[derive(Accounts)] contexts
    /// - Account LEN constants
    /// - Anchor IDL JSON
    /// - TypeScript client (optional)
    Generate {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Program name (default: derived from schema filename)
        #[arg(short, long)]
        name: Option<String>,

        /// Program version (default: 0.1.0)
        #[arg(short = 'V', long, default_value = "0.1.0")]
        version: String,

        /// Program address (optional)
        #[arg(short, long)]
        address: Option<String>,

        /// Generate TypeScript client
        #[arg(long)]
        typescript: bool,

        /// Dry run (show what would be generated without writing files)
        #[arg(long)]
        dry_run: bool,
    },

    /// Generate Anchor IDL from LUMOS schema
    Idl {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output file path (default: target/idl/<program_name>.json)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Program name (default: derived from schema filename)
        #[arg(short, long)]
        name: Option<String>,

        /// Program version (default: 0.1.0)
        #[arg(short = 'V', long, default_value = "0.1.0")]
        version: String,

        /// Program address (optional)
        #[arg(short, long)]
        address: Option<String>,

        /// Pretty print JSON output
        #[arg(short, long)]
        pretty: bool,
    },

    /// Generate Rust code with Anchor account space constants
    Space {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output format (text or rust)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Specific account type to calculate (optional)
        #[arg(short, long)]
        account: Option<String>,
    },
}

#[derive(Subcommand)]
enum MetaplexCommands {
    /// Validate schema against Metaplex Token Metadata standards
    ///
    /// Validates:
    /// - Name length (max 32 characters)
    /// - Symbol length (max 10 characters)
    /// - URI length (max 200 characters)
    /// - Seller fee basis points (0-10000)
    /// - Creator constraints (max 5, shares sum to 100)
    Validate {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output format (text or json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Show verbose output with all validations
        #[arg(short, long)]
        verbose: bool,
    },

    /// Generate Metaplex-compatible code from schema
    ///
    /// Generates Rust and TypeScript code with:
    /// - Metaplex constraints validation
    /// - Token Metadata compatible types
    /// - Proper Borsh serialization
    Generate {
        /// Path to .lumos schema file
        schema: PathBuf,

        /// Output directory (default: current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Generate TypeScript code
        #[arg(long)]
        typescript: bool,

        /// Generate Rust code (default: true)
        #[arg(long, default_value = "true")]
        rust: bool,

        /// Dry run (show what would be generated without writing files)
        #[arg(long)]
        dry_run: bool,
    },

    /// Show standard Metaplex type definitions
    ///
    /// Outputs the standard Metaplex types (Metadata, Creator, Collection, etc.)
    /// in LUMOS schema format for reference or inclusion in your schemas.
    Types {
        /// Output format (lumos or json)
        #[arg(short, long, default_value = "lumos")]
        format: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            schema,
            output,
            lang,
            target,
            watch,
            dry_run,
            backup,
            show_diff,
        } => {
            if watch {
                commands::watch::run(&schema, output.as_deref(), &lang, &target)
            } else {
                commands::generate::run(
                    &schema,
                    output.as_deref(),
                    &lang,
                    &target,
                    dry_run,
                    backup,
                    show_diff,
                )
            }
        }
        Commands::Validate { schema } => commands::validate::run(&schema),
        Commands::Init { name } => commands::init::run(name.as_deref()),
        Commands::Check { schema, output } => commands::check::run(&schema, output.as_deref()),
        Commands::CheckSize { schema, format } => commands::check::run_size(&schema, &format),
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
        Commands::Diff {
            schema1,
            schema2,
            format,
        } => commands::diff::run(&schema1, &schema2, &format),
        Commands::Migrate {
            from_schema,
            to_schema,
            output,
            language,
            dry_run,
            force,
        } => commands::migrate::run(
            &from_schema,
            &to_schema,
            output.as_deref(),
            &language,
            dry_run,
            force,
        ),

        Commands::CheckCompat {
            from_schema,
            to_schema,
            format,
            verbose,
            strict,
        } => commands::check_compat::run(&from_schema, &to_schema, &format, verbose, strict),

        Commands::Anchor { command } => match command {
            AnchorCommands::Generate {
                schema,
                output,
                name,
                version,
                address,
                typescript,
                dry_run,
            } => run_anchor_generate(
                &schema,
                output.as_deref(),
                name.as_deref(),
                &version,
                address.as_deref(),
                typescript,
                dry_run,
            ),
            AnchorCommands::Idl {
                schema,
                output,
                name,
                version,
                address,
                pretty,
            } => run_anchor_idl(
                &schema,
                output.as_deref(),
                name.as_deref(),
                &version,
                address.as_deref(),
                pretty,
            ),
            AnchorCommands::Space {
                schema,
                format,
                account,
            } => run_anchor_space(&schema, &format, account.as_deref()),
        },

        Commands::Metaplex { command } => match command {
            MetaplexCommands::Validate {
                schema,
                format,
                verbose,
            } => run_metaplex_validate(&schema, &format, verbose),
            MetaplexCommands::Generate {
                schema,
                output,
                typescript,
                rust,
                dry_run,
            } => run_metaplex_generate(&schema, output.as_deref(), typescript, rust, dry_run),
            MetaplexCommands::Types { format } => run_metaplex_types(&format),
        },
    }
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



/// Generate fuzz targets from schema
fn run_fuzz_generate(
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
fn run_fuzz_run(
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

// ============================================================================
// Anchor Framework Commands
// ============================================================================

/// Generate Anchor IDL from LUMOS schema
fn run_anchor_idl(
    schema_path: &Path,
    output_path: Option<&Path>,
    program_name: Option<&str>,
    version: &str,
    address: Option<&str>,
    pretty: bool,
) -> Result<()> {
    // Parse and transform schema
    let (type_defs, _file_count) = commands::generate::resolve_schema(schema_path)?;

    // Derive program name from schema filename if not provided
    let name = program_name.map(String::from).unwrap_or_else(|| {
        schema_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("my_program")
            .to_string()
    });

    // Configure IDL generator
    let config = IdlGeneratorConfig {
        program_name: name.clone(),
        version: version.to_string(),
        address: address.map(String::from),
    };

    // Generate IDL
    let generator = IdlGenerator::new(config);
    let idl = generator.generate(&type_defs);

    // Serialize to JSON
    let json_output = if pretty {
        serde_json::to_string_pretty(&idl)?
    } else {
        serde_json::to_string(&idl)?
    };

    // Determine output path
    if let Some(out_path) = output_path {
        // Validate output path
        validate_output_path(out_path)?;
        fs::write(out_path, &json_output)?;
        println!("{:>12} {}", "Generated".green().bold(), out_path.display());
    } else {
        // Default: target/idl/<program_name>.json
        let default_path = PathBuf::from("target/idl").join(format!("{}.json", name));

        // Create directory if it doesn't exist
        if let Some(parent) = default_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&default_path, &json_output)?;
        println!(
            "{:>12} {}",
            "Generated".green().bold(),
            default_path.display()
        );
    }

    // Print summary
    let account_count = idl.accounts.len();
    let type_count = idl.types.len();

    println!();
    println!("{}", "IDL Summary:".bold());
    println!("  Program: {}", idl.name.cyan());
    println!("  Version: {}", idl.version);
    if let Some(ref meta) = idl.metadata {
        if let Some(ref addr) = meta.address {
            println!("  Address: {}", addr);
        }
    }
    println!("  Accounts: {}", account_count);
    println!("  Types: {}", type_count);

    Ok(())
}

/// Generate Anchor account space constants
fn run_anchor_space(schema_path: &Path, format: &str, account_name: Option<&str>) -> Result<()> {
    // Parse and transform schema
    let (type_defs, _file_count) = commands::generate::resolve_schema(schema_path)?;

    // Create generator for space calculation
    let generator = IdlGenerator::new(IdlGeneratorConfig::default());

    // Find account types
    let accounts: Vec<_> = type_defs
        .iter()
        .filter_map(|td| {
            if let TypeDefinition::Struct(struct_def) = td {
                if struct_def
                    .metadata
                    .attributes
                    .iter()
                    .any(|a| a == "account")
                {
                    if let Some(name) = account_name {
                        if struct_def.name != name {
                            return None;
                        }
                    }
                    return Some(struct_def);
                }
            }
            None
        })
        .collect();

    if accounts.is_empty() {
        if let Some(name) = account_name {
            println!(
                "{:>12} No account '{}' found in schema",
                "Warning".yellow().bold(),
                name
            );
        } else {
            println!(
                "{:>12} No account types found in schema",
                "Warning".yellow().bold()
            );
            println!("  Hint: Add #[account] attribute to struct definitions");
        }
        return Ok(());
    }

    println!("{:>12} account space...\n", "Calculating".cyan().bold());

    if format == "rust" {
        // Generate Rust code
        println!("// Auto-generated account space constants");
        println!("// Generated by: lumos anchor space\n");

        for account in &accounts {
            let space = generator.calculate_account_space(account);
            println!("impl {} {{", account.name);
            println!("    /// Account size including 8-byte discriminator");
            println!("    pub const LEN: usize = {};", space);
            println!("}}\n");
        }
    } else {
        // Text output
        println!(
            "{:<30} {:>10} {:>15}",
            "Account".bold(),
            "Size".bold(),
            "Breakdown".bold()
        );
        println!("{}", "-".repeat(60));

        for account in &accounts {
            let space = generator.calculate_account_space(account);
            let breakdown = format!("8 (disc) + {} (data)", space - 8);
            println!(
                "{:<30} {:>10} {:>15}",
                account.name,
                format!("{} bytes", space),
                breakdown
            );
        }

        println!();
        println!("{}", "Note:".bold());
        println!("  - Size includes 8-byte Anchor discriminator");
        println!("  - Variable-length fields (String, Vec) show prefix only");
        println!("  - Max Solana account size: 10,485,760 bytes (10 MiB)");
    }

    Ok(())
}

/// Generate complete Anchor program from LUMOS schema
///
/// This command generates:
/// 1. Rust program with #[derive(Accounts)] contexts for instruction structs
/// 2. Account LEN constants for all account types
/// 3. Anchor IDL JSON
/// 4. TypeScript client (optional)
fn run_anchor_generate(
    schema_path: &Path,
    output_dir: Option<&Path>,
    program_name: Option<&str>,
    version: &str,
    address: Option<&str>,
    generate_typescript: bool,
    dry_run: bool,
) -> Result<()> {
    let output_dir = output_dir.unwrap_or_else(|| Path::new("."));

    // Validate output directory
    if !dry_run {
        validate_output_path(output_dir)?;
    }

    // Parse and transform schema
    let (type_defs, _file_count) = commands::generate::resolve_schema(schema_path)?;

    // Derive program name from schema filename if not provided
    let name = program_name.map(String::from).unwrap_or_else(|| {
        schema_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("my_program")
            .to_string()
    });

    println!(
        "{:>12} Anchor program '{}'...\n",
        "Generating".cyan().bold(),
        name
    );

    // Configure IDL generator
    let idl_config = IdlGeneratorConfig {
        program_name: name.clone(),
        version: version.to_string(),
        address: address.map(String::from),
    };
    let idl_generator = IdlGenerator::new(idl_config);

    // Collect accounts, instructions, and types
    let mut accounts = Vec::new();
    let mut instructions = Vec::new();
    let mut other_types = Vec::new();

    for type_def in &type_defs {
        match type_def {
            TypeDefinition::Struct(s) => {
                let is_account = s.metadata.attributes.iter().any(|a| a == "account");
                let is_instruction = s.metadata.is_instruction;

                if is_instruction {
                    instructions.push(s);
                } else if is_account {
                    accounts.push(s);
                } else {
                    other_types.push(type_def);
                }
            }
            TypeDefinition::Enum(_) | TypeDefinition::TypeAlias(_) => {
                other_types.push(type_def);
            }
        }
    }

    // === Generate Rust Code ===
    let mut rust_output = String::new();

    // Header comment
    rust_output.push_str("// Auto-generated by LUMOS - Anchor Program\n");
    rust_output.push_str(&format!("// Source: {}\n", schema_path.display()));
    rust_output.push_str(&format!("// Program: {}\n", name));
    rust_output.push_str(&format!("// Version: {}\n\n", version));

    // Imports
    rust_output.push_str("use anchor_lang::prelude::*;\n\n");

    // Program ID declaration (placeholder if no address provided)
    if let Some(addr) = address {
        rust_output.push_str(&format!("declare_id!(\"{}\");\n\n", addr));
    } else {
        rust_output.push_str("// TODO: Replace with your program ID\n");
        rust_output.push_str("declare_id!(\"YourProgramIdHere11111111111111111111111111\");\n\n");
    }

    // Generate account structs with LEN constants
    if !accounts.is_empty() {
        rust_output.push_str(
            "// ============================================================================\n",
        );
        rust_output.push_str("// Account Types\n");
        rust_output.push_str(
            "// ============================================================================\n\n",
        );

        for account in &accounts {
            // Account struct
            rust_output.push_str("#[account]\n");
            rust_output.push_str(&format!("pub struct {} {{\n", account.name));
            for field in &account.fields {
                let rust_type = type_info_to_rust_type(&field.type_info);
                rust_output.push_str(&format!("    pub {}: {},\n", field.name, rust_type));
            }
            rust_output.push_str("}\n\n");

            // LEN constant
            let space = idl_generator.calculate_account_space(account);
            rust_output.push_str(&format!("impl {} {{\n", account.name));
            rust_output.push_str("    /// Account size including 8-byte discriminator\n");
            rust_output.push_str(&format!("    pub const LEN: usize = {};\n", space));
            rust_output.push_str("}\n\n");
        }
    }

    // Generate instruction contexts
    if !instructions.is_empty() {
        rust_output.push_str(
            "// ============================================================================\n",
        );
        rust_output.push_str("// Instruction Contexts\n");
        rust_output.push_str(
            "// ============================================================================\n\n",
        );

        for instruction in &instructions {
            // Build InstructionContext from the struct
            let mut ctx_accounts = Vec::new();

            for field in &instruction.fields {
                let mut attrs = Vec::new();
                for attr_str in &field.anchor_attrs {
                    attrs.extend(parse_anchor_attrs(attr_str));
                }

                let account_type = infer_anchor_account_type(&field.type_info);

                ctx_accounts.push(InstructionAccount {
                    name: field.name.clone(),
                    account_type,
                    attrs,
                    optional: field.optional,
                    docs: vec![],
                });
            }

            let ctx = InstructionContext {
                name: instruction.name.clone(),
                accounts: ctx_accounts,
                args: vec![],
            };

            // Generate the #[derive(Accounts)] context
            rust_output.push_str(&generate_accounts_context(&ctx));
            rust_output.push('\n');
        }
    }

    // Generate other types (enums, non-account structs)
    if !other_types.is_empty() {
        rust_output.push_str(
            "// ============================================================================\n",
        );
        rust_output.push_str("// Custom Types\n");
        rust_output.push_str(
            "// ============================================================================\n\n",
        );

        for type_def in &other_types {
            match type_def {
                TypeDefinition::Struct(s) => {
                    rust_output
                        .push_str("#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]\n");
                    rust_output.push_str(&format!("pub struct {} {{\n", s.name));
                    for field in &s.fields {
                        let rust_type = type_info_to_rust_type(&field.type_info);
                        rust_output.push_str(&format!("    pub {}: {},\n", field.name, rust_type));
                    }
                    rust_output.push_str("}\n\n");
                }
                TypeDefinition::Enum(e) => {
                    rust_output.push_str(
                        "#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]\n",
                    );
                    rust_output.push_str(&format!("pub enum {} {{\n", e.name));
                    for variant in &e.variants {
                        match variant {
                            lumos_core::ir::EnumVariantDefinition::Unit { name } => {
                                rust_output.push_str(&format!("    {},\n", name));
                            }
                            lumos_core::ir::EnumVariantDefinition::Tuple { name, types } => {
                                let type_strs: Vec<String> =
                                    types.iter().map(type_info_to_rust_type).collect();
                                rust_output.push_str(&format!(
                                    "    {}({}),\n",
                                    name,
                                    type_strs.join(", ")
                                ));
                            }
                            lumos_core::ir::EnumVariantDefinition::Struct { name, fields } => {
                                rust_output.push_str(&format!("    {} {{\n", name));
                                for field in fields {
                                    let rust_type = type_info_to_rust_type(&field.type_info);
                                    rust_output.push_str(&format!(
                                        "        {}: {},\n",
                                        field.name, rust_type
                                    ));
                                }
                                rust_output.push_str("    },\n");
                            }
                        }
                    }
                    rust_output.push_str("}\n\n");
                }
                TypeDefinition::TypeAlias(_) => {
                    // Type aliases are resolved, skip
                }
            }
        }
    }

    // === Generate IDL ===
    let idl = idl_generator.generate(&type_defs);
    let idl_json = serde_json::to_string_pretty(&idl)?;

    // === Generate TypeScript (optional) ===
    let ts_output = if generate_typescript {
        Some(typescript::generate_module(&type_defs))
    } else {
        None
    };

    // === Output Results ===
    if dry_run {
        println!("{}", "=== DRY RUN - No files written ===\n".yellow().bold());

        println!("{}", "Rust Program (lib.rs):".bold());
        println!("{}", "-".repeat(60));
        println!("{}", rust_output);

        println!("{}", "IDL (idl.json):".bold());
        println!("{}", "-".repeat(60));
        println!("{}", idl_json);

        if let Some(ref ts) = ts_output {
            println!("{}", "TypeScript Client (types.ts):".bold());
            println!("{}", "-".repeat(60));
            println!("{}", ts);
        }
    } else {
        // Create output directories
        let programs_dir = output_dir.join("programs").join(&name).join("src");
        let idl_dir = output_dir.join("target").join("idl");

        fs::create_dir_all(&programs_dir)?;
        fs::create_dir_all(&idl_dir)?;

        // Write Rust program
        let rust_path = programs_dir.join("lib.rs");
        fs::write(&rust_path, &rust_output)?;
        println!("{:>12} {}", "Generated".green().bold(), rust_path.display());

        // Write IDL
        let idl_path = idl_dir.join(format!("{}.json", name));
        fs::write(&idl_path, &idl_json)?;
        println!("{:>12} {}", "Generated".green().bold(), idl_path.display());

        // Write TypeScript
        if let Some(ref ts) = ts_output {
            let app_dir = output_dir.join("app").join("src");
            fs::create_dir_all(&app_dir)?;

            let ts_path = app_dir.join("types.ts");
            fs::write(&ts_path, ts)?;
            println!("{:>12} {}", "Generated".green().bold(), ts_path.display());
        }
    }

    // Print summary
    println!();
    println!("{}", "Summary:".bold());
    println!("  Program: {}", name.cyan());
    println!("  Version: {}", version);
    println!("  Accounts: {}", accounts.len());
    println!("  Instructions: {}", instructions.len());
    println!("  Other types: {}", other_types.len());

    if !dry_run {
        println!();
        println!("{}", "Next steps:".bold());
        println!("  1. Update declare_id!() with your program address");
        println!("  2. Implement instruction handlers");
        println!("  3. Run `anchor build` to compile");
        println!("  4. Run `anchor test` to verify");
    }

    Ok(())
}

// =============================================================================
// Metaplex Token Metadata Commands
// =============================================================================

/// Validate schema against Metaplex Token Metadata standards
fn run_metaplex_validate(schema_path: &Path, format: &str, verbose: bool) -> Result<()> {
    // Parse and transform schema
    let (type_defs, _file_count) = commands::generate::resolve_schema(schema_path)?;

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
fn run_metaplex_generate(
    schema_path: &Path,
    output_dir: Option<&Path>,
    typescript: bool,
    rust: bool,
    dry_run: bool,
) -> Result<()> {
    // Parse and transform schema
    let (type_defs, _file_count) = commands::generate::resolve_schema(schema_path)?;

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
fn run_metaplex_types(format: &str) -> Result<()> {
    use lumos_core::metaplex::generate_standard_types;

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
