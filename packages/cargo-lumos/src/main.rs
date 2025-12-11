// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! cargo-lumos - Cargo subcommand for LUMOS schema generation
//!
//! This binary allows using LUMOS through cargo:
//!
//! ```bash
//! cargo lumos generate schema.lumos
//! cargo lumos validate schema.lumos
//! cargo lumos watch schema.lumos
//! ```
//!
//! It also supports configuration via Cargo.toml:
//!
//! ```toml
//! [package.metadata.lumos]
//! schema = "schemas/types.lumos"
//! output_rust = "src/generated.rs"
//! output_ts = "app/src/generated.ts"
//! ```

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

/// Cargo subcommand for LUMOS schema generation
#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
struct Cargo {
    #[command(subcommand)]
    command: CargoSubcommand,
}

#[derive(Subcommand)]
enum CargoSubcommand {
    /// LUMOS schema code generator
    Lumos(LumosArgs),
}

/// LUMOS CLI arguments
#[derive(Parser, Debug)]
#[command(name = "lumos")]
#[command(about = "Type-safe schema language for Solana development")]
#[command(version)]
struct LumosArgs {
    #[command(subcommand)]
    command: Option<LumosCommands>,

    /// Use configuration from Cargo.toml
    #[arg(long, global = true)]
    use_config: bool,
}

#[derive(Subcommand, Debug)]
enum LumosCommands {
    /// Generate Rust and TypeScript code from schema
    Generate {
        /// Path to .lumos schema file (optional if configured in Cargo.toml)
        schema: Option<PathBuf>,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Watch for changes
        #[arg(short, long)]
        watch: bool,

        /// Preview changes without writing files
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Create backup before overwriting
        #[arg(short = 'b', long)]
        backup: bool,
    },

    /// Validate schema syntax
    Validate {
        /// Path to .lumos schema file
        schema: Option<PathBuf>,
    },

    /// Initialize a new LUMOS project
    Init {
        /// Project name
        name: Option<String>,
    },

    /// Check if generated code is up-to-date
    Check {
        /// Path to .lumos schema file
        schema: Option<PathBuf>,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run any lumos command (passthrough)
    #[command(external_subcommand)]
    External(Vec<String>),
}

/// LUMOS configuration from Cargo.toml
#[derive(Debug, Deserialize, Default)]
struct LumosConfig {
    /// Path to schema file
    schema: Option<String>,
    /// Output path for Rust code
    ///
    /// Part of stable config API - users can configure this in Cargo.toml.
    /// Currently deserialized but not consumed (lumos CLI uses --output flag).
    /// Planned: Pass these values to lumos via --output-rust/--output-ts flags.
    /// Tracked in: https://github.com/getlumos/lumos/issues/TBD
    #[allow(dead_code)] // Reserved for future use - part of public config API
    output_rust: Option<String>,
    /// Output path for TypeScript code
    ///
    /// Part of stable config API - users can configure this in Cargo.toml.
    /// Currently deserialized but not consumed (lumos CLI uses --output flag).
    /// Planned: Pass these values to lumos via --output-rust/--output-ts flags.
    /// Tracked in: https://github.com/getlumos/lumos/issues/TBD
    #[allow(dead_code)] // Reserved for future use - part of public config API
    output_ts: Option<String>,
    /// Watch mode enabled by default
    watch: Option<bool>,
}

/// Cargo.toml structure for metadata extraction
#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Option<PackageSection>,
}

#[derive(Debug, Deserialize)]
struct PackageSection {
    metadata: Option<MetadataSection>,
}

#[derive(Debug, Deserialize)]
struct MetadataSection {
    lumos: Option<LumosConfig>,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode> {
    let Cargo {
        command: CargoSubcommand::Lumos(args),
    } = Cargo::parse();

    // Load config from Cargo.toml if present
    let config = load_cargo_config().unwrap_or_default();

    // Build lumos command arguments
    let lumos_args = build_lumos_args(&args, &config)?;

    // Find lumos binary
    let lumos_path = find_lumos_binary()?;

    // Execute lumos command
    let status = Command::new(&lumos_path)
        .args(&lumos_args)
        .status()
        .with_context(|| {
            format!(
                "Failed to execute: {} {}",
                lumos_path.display(),
                lumos_args.join(" ")
            )
        })?;

    Ok(if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(status.code().unwrap_or(1) as u8)
    })
}

/// Load LUMOS configuration from Cargo.toml
fn load_cargo_config() -> Option<LumosConfig> {
    let cargo_toml_path = find_cargo_toml()?;
    let content = std::fs::read_to_string(&cargo_toml_path).ok()?;
    let cargo_toml: CargoToml = toml::from_str(&content).ok()?;

    cargo_toml
        .package
        .and_then(|p| p.metadata)
        .and_then(|m| m.lumos)
}

/// Find Cargo.toml by walking up the directory tree
fn find_cargo_toml() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return Some(cargo_toml);
        }

        if !current.pop() {
            return None;
        }
    }
}

/// Find the lumos binary
fn find_lumos_binary() -> Result<PathBuf> {
    // First, check for local debug/release builds (useful during development)
    let local_paths = [
        PathBuf::from("./target/debug/lumos"),
        PathBuf::from("./target/release/lumos"),
    ];

    for path in &local_paths {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    // Check if lumos is in PATH
    if let Ok(path) = which::which("lumos") {
        return Ok(path);
    }

    // Check cargo bin directory
    if let Some(home) = dirs_next::home_dir() {
        let cargo_bin = home.join(".cargo").join("bin").join("lumos");
        if cargo_bin.exists() {
            return Ok(cargo_bin);
        }
    }

    anyhow::bail!(
        "Could not find 'lumos' binary. Install it with:\n\n    {}\n",
        "cargo install lumos-cli".cyan()
    )
}

/// Build command-line arguments for lumos
fn build_lumos_args(args: &LumosArgs, config: &LumosConfig) -> Result<Vec<String>> {
    let mut lumos_args = Vec::new();

    match &args.command {
        Some(LumosCommands::Generate {
            schema,
            output,
            watch,
            dry_run,
            backup,
        }) => {
            lumos_args.push("generate".to_string());

            // Schema path: CLI arg > config > error
            let schema_path = schema
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .or_else(|| config.schema.clone())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "No schema file specified. Use:\n\n    {} or configure in Cargo.toml:\n\n    {}",
                        "cargo lumos generate <schema.lumos>".cyan(),
                        "[package.metadata.lumos]\nschema = \"path/to/schema.lumos\"".dimmed()
                    )
                })?;

            lumos_args.push(schema_path);

            // Output directory
            if let Some(out) = output {
                lumos_args.push("--output".to_string());
                lumos_args.push(out.to_string_lossy().to_string());
            }

            // Watch mode
            if *watch || config.watch.unwrap_or(false) {
                lumos_args.push("--watch".to_string());
            }

            // Dry run
            if *dry_run {
                lumos_args.push("--dry-run".to_string());
            }

            // Backup
            if *backup {
                lumos_args.push("--backup".to_string());
            }
        }

        Some(LumosCommands::Validate { schema }) => {
            lumos_args.push("validate".to_string());

            let schema_path = schema
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .or_else(|| config.schema.clone())
                .ok_or_else(|| anyhow::anyhow!("No schema file specified"))?;

            lumos_args.push(schema_path);
        }

        Some(LumosCommands::Init { name }) => {
            lumos_args.push("init".to_string());
            if let Some(n) = name {
                lumos_args.push(n.clone());
            }
        }

        Some(LumosCommands::Check { schema, output }) => {
            lumos_args.push("check".to_string());

            let schema_path = schema
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .or_else(|| config.schema.clone())
                .ok_or_else(|| anyhow::anyhow!("No schema file specified"))?;

            lumos_args.push(schema_path);

            if let Some(out) = output {
                lumos_args.push("--output".to_string());
                lumos_args.push(out.to_string_lossy().to_string());
            }
        }

        Some(LumosCommands::External(external_args)) => {
            // Pass through all external arguments
            lumos_args.extend(external_args.clone());
        }

        None => {
            // No subcommand - show help
            lumos_args.push("--help".to_string());
        }
    }

    Ok(lumos_args)
}

/// Support for dirs_next if not available
mod dirs_next {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let toml_content = r#"
[package]
name = "test"
version = "0.1.0"

[package.metadata.lumos]
schema = "schemas/types.lumos"
output_rust = "src/generated.rs"
"#;
        let cargo_toml: CargoToml = toml::from_str(toml_content).unwrap();
        let config = cargo_toml.package.unwrap().metadata.unwrap().lumos.unwrap();

        assert_eq!(config.schema, Some("schemas/types.lumos".to_string()));
        assert_eq!(config.output_rust, Some("src/generated.rs".to_string()));
    }

    #[test]
    fn test_build_args_generate() {
        let args = LumosArgs {
            command: Some(LumosCommands::Generate {
                schema: Some(PathBuf::from("test.lumos")),
                output: None,
                watch: false,
                dry_run: false,
                backup: false,
            }),
            use_config: false,
        };

        let config = LumosConfig::default();
        let result = build_lumos_args(&args, &config).unwrap();

        assert_eq!(result, vec!["generate", "test.lumos"]);
    }

    #[test]
    fn test_build_args_with_config() {
        let args = LumosArgs {
            command: Some(LumosCommands::Generate {
                schema: None,
                output: None,
                watch: false,
                dry_run: false,
                backup: false,
            }),
            use_config: true,
        };

        let config = LumosConfig {
            schema: Some("configured.lumos".to_string()),
            output_rust: None,
            output_ts: None,
            watch: Some(true),
        };

        let result = build_lumos_args(&args, &config).unwrap();

        assert_eq!(result, vec!["generate", "configured.lumos", "--watch"]);
    }

    #[test]
    fn test_build_args_validate() {
        let args = LumosArgs {
            command: Some(LumosCommands::Validate {
                schema: Some(PathBuf::from("test.lumos")),
            }),
            use_config: false,
        };

        let config = LumosConfig::default();
        let result = build_lumos_args(&args, &config).unwrap();

        assert_eq!(result, vec!["validate", "test.lumos"]);
    }

    #[test]
    fn test_build_args_external() {
        let args = LumosArgs {
            command: Some(LumosCommands::External(vec![
                "diff".to_string(),
                "v1.lumos".to_string(),
                "v2.lumos".to_string(),
            ])),
            use_config: false,
        };

        let config = LumosConfig::default();
        let result = build_lumos_args(&args, &config).unwrap();

        assert_eq!(result, vec!["diff", "v1.lumos", "v2.lumos"]);
    }
}
