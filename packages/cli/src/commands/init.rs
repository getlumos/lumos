// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Init command - initialize new LUMOS project

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

/// Initialize a new LUMOS project
pub fn run(project_name: Option<&str>) -> Result<()> {
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
    if let Some(name) = project_name {
        println!("  cd {}", name);
    }
    println!("  lumos generate schema.lumos");

    Ok(())
}
