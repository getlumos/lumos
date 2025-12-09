// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Backup utilities for CLI

use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::Path;

/// Create backup of file if it exists
pub fn create_backup_if_exists(path: &Path) -> Result<()> {
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
        "  {} -> {}",
        path.display().to_string().dimmed(),
        backup_path.display().to_string().cyan()
    );

    Ok(())
}
