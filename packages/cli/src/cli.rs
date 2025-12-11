// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! CLI argument definitions and command structures

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "lumos")]
#[command(about = "Type-safe schema language for Solana development", long_about = None)]
#[command(version)]
#[command(author)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
pub enum SecurityCommands {
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
pub enum AuditCommands {
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
pub enum FuzzCommands {
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
pub enum AnchorCommands {
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
pub enum MetaplexCommands {
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
