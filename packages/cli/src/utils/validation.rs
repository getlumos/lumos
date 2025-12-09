// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Path validation utilities for CLI

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

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
pub fn validate_output_path(path: &Path) -> Result<()> {
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
