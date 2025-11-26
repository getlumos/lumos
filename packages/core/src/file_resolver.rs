// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! File resolver for handling multi-file LUMOS schemas with imports
//!
//! This module provides functionality to:
//! - Resolve import paths relative to the current file
//! - Load and parse .lumos files
//! - Detect circular import dependencies
//! - Cache loaded files to avoid duplicate parsing

use crate::ast::{Item as AstItem, LumosFile};
use crate::error::{LumosError, Result};
use crate::ir::TypeDefinition;
use crate::parser::parse_lumos_file;
use crate::transform::{
    transform_to_ir_with_resolver_no_validation, validate_user_defined_types, TypeAliasResolver,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// File resolver that handles multi-file schemas with imports
#[derive(Debug)]
pub struct FileResolver {
    /// Map of resolved absolute path -> parsed LumosFile
    loaded_files: HashMap<PathBuf, LumosFile>,

    /// Stack of files currently being loaded (for circular import detection)
    loading_stack: Vec<PathBuf>,
}

impl Default for FileResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl FileResolver {
    /// Create a new file resolver
    pub fn new() -> Self {
        Self {
            loaded_files: HashMap::new(),
            loading_stack: Vec::new(),
        }
    }

    /// Resolve all imports starting from an entry file and return combined type definitions
    pub fn resolve_imports(&mut self, entry_file: &Path) -> Result<Vec<TypeDefinition>> {
        // Canonicalize the entry file path
        let entry_path = self.canonicalize_path(entry_file)?;

        // Load the entry file and all its dependencies
        self.load_file_recursive(&entry_path)?;

        // First pass: Collect all type aliases from all files
        let mut resolver = TypeAliasResolver::new();
        for lumos_file in self.loaded_files.values() {
            for item in &lumos_file.items {
                if let AstItem::TypeAlias(alias_def) = item {
                    resolver.add_alias(alias_def.name.clone(), alias_def.target.clone())?;
                }
            }
        }

        // Resolve all aliases recursively
        resolver.resolve_all_aliases()?;

        // Second pass: Transform all files with shared resolver (skip per-file validation)
        let mut all_type_defs = Vec::new();
        for lumos_file in self.loaded_files.values() {
            let type_defs =
                transform_to_ir_with_resolver_no_validation(lumos_file.clone(), &resolver)?;
            all_type_defs.extend(type_defs);
        }

        // Third pass: Validate all user-defined types across all files
        validate_user_defined_types(&all_type_defs)?;

        Ok(all_type_defs)
    }

    /// Recursively load a file and all its imports
    fn load_file_recursive(&mut self, file_path: &Path) -> Result<()> {
        // Check if already loaded (cache hit)
        if self.loaded_files.contains_key(file_path) {
            return Ok(());
        }

        // Check for circular imports
        if self.loading_stack.contains(&file_path.to_path_buf()) {
            return Err(LumosError::SchemaParse(
                format!(
                    "Circular import detected: {} -> {}",
                    self.format_import_chain(),
                    file_path.display()
                ),
                None,
            ));
        }

        // Add to loading stack
        self.loading_stack.push(file_path.to_path_buf());

        // Read and parse the file
        let content = fs::read_to_string(file_path).map_err(|e| {
            LumosError::SchemaParse(
                format!("Failed to read file '{}': {}", file_path.display(), e),
                None,
            )
        })?;

        let lumos_file = parse_lumos_file(&content)?;

        // Get the directory of the current file for resolving relative imports
        let current_dir = file_path.parent().ok_or_else(|| {
            LumosError::SchemaParse(format!("Invalid file path: {}", file_path.display()), None)
        })?;

        // Recursively load all imported files
        for import in &lumos_file.imports {
            let import_path = self.resolve_import_path(current_dir, &import.path)?;
            self.load_file_recursive(&import_path)?;
        }

        // Remove from loading stack
        self.loading_stack.pop();

        // Add to cache
        self.loaded_files
            .insert(file_path.to_path_buf(), lumos_file);

        Ok(())
    }

    /// Resolve an import path relative to the current directory
    fn resolve_import_path(&self, current_dir: &Path, import_path: &str) -> Result<PathBuf> {
        // Handle relative paths like "./types.lumos" or "../common/types.lumos"
        let mut resolved_path = current_dir.join(import_path);

        // Add .lumos extension if not present
        if resolved_path.extension().is_none() {
            resolved_path.set_extension("lumos");
        }

        // Canonicalize to get absolute path
        self.canonicalize_path(&resolved_path)
    }

    /// Canonicalize a path to an absolute path
    fn canonicalize_path(&self, path: &Path) -> Result<PathBuf> {
        path.canonicalize().map_err(|e| {
            LumosError::SchemaParse(
                format!("Failed to resolve path '{}': {}", path.display(), e),
                None,
            )
        })
    }

    /// Format the current import chain for error messages
    fn format_import_chain(&self) -> String {
        self.loading_stack
            .iter()
            .map(|p| p.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"))
            .collect::<Vec<_>>()
            .join(" -> ")
    }

    /// Get all loaded file paths
    pub fn loaded_files(&self) -> Vec<&Path> {
        self.loaded_files.keys().map(|p| p.as_path()).collect()
    }

    /// Validate that all imported types exist
    pub fn validate_imports(&self) -> Result<()> {
        // Collect all defined type names
        let mut defined_types = HashSet::new();
        for lumos_file in self.loaded_files.values() {
            for item in &lumos_file.items {
                let name_opt = match item {
                    crate::ast::Item::Struct(s) => Some(&s.name),
                    crate::ast::Item::Enum(e) => Some(&e.name),
                    crate::ast::Item::TypeAlias(a) => Some(&a.name),
                    crate::ast::Item::Module(m) => Some(&m.name),
                    // Use statements don't define types, skip them
                    crate::ast::Item::Use(_) => None,
                };
                if let Some(name) = name_opt {
                    defined_types.insert(name.clone());
                }
            }
        }

        // Check that all imported items exist
        for (file_path, lumos_file) in &self.loaded_files {
            for import in &lumos_file.imports {
                for item in &import.items {
                    if !defined_types.contains(item) {
                        return Err(LumosError::SchemaParse(
                            format!(
                                "Import error in '{}': type '{}' not found in '{}'",
                                file_path.display(),
                                item,
                                import.path
                            ),
                            None,
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.lumos");

        fs::write(
            &file_path,
            r#"
#[solana]
#[account]
struct Account {
    balance: u64,
}
"#,
        )
        .unwrap();

        let mut resolver = FileResolver::new();
        let result = resolver.resolve_imports(&file_path);

        assert!(result.is_ok());
        let type_defs = result.unwrap();
        assert_eq!(type_defs.len(), 1);
    }

    #[test]
    fn test_detect_circular_import() {
        let temp_dir = TempDir::new().unwrap();

        // Create a.lumos that imports b.lumos
        let a_path = temp_dir.path().join("a.lumos");
        fs::write(
            &a_path,
            r#"
import { B } from "./b.lumos";
struct A { b: B }
"#,
        )
        .unwrap();

        // Create b.lumos that imports a.lumos (circular!)
        let b_path = temp_dir.path().join("b.lumos");
        fs::write(
            &b_path,
            r#"
import { A } from "./a.lumos";
struct B { a: A }
"#,
        )
        .unwrap();

        let mut resolver = FileResolver::new();
        let result = resolver.resolve_imports(&a_path);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Circular import"));
    }

    #[test]
    fn test_resolve_multiple_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create types.lumos
        let types_path = temp_dir.path().join("types.lumos");
        fs::write(
            &types_path,
            r#"
type UserId = PublicKey;
type Timestamp = i64;
"#,
        )
        .unwrap();

        // Create main.lumos that imports types.lumos
        let main_path = temp_dir.path().join("main.lumos");
        fs::write(
            &main_path,
            r#"
import { UserId, Timestamp } from "./types.lumos";

struct Account {
    owner: UserId,
    created_at: Timestamp,
    balance: u64,
}
"#,
        )
        .unwrap();

        let mut resolver = FileResolver::new();
        let result = resolver.resolve_imports(&main_path);

        assert!(result.is_ok());
        assert_eq!(resolver.loaded_files().len(), 2);
    }

    #[test]
    fn test_validate_imports_missing_type() {
        let temp_dir = TempDir::new().unwrap();

        // Create types.lumos without the imported type
        let types_path = temp_dir.path().join("types.lumos");
        fs::write(&types_path, "type Foo = u64;").unwrap();

        // Create main.lumos that imports non-existent type
        let main_path = temp_dir.path().join("main.lumos");
        fs::write(
            &main_path,
            r#"
import { NonExistent } from "./types.lumos";
struct Account { value: NonExistent }
"#,
        )
        .unwrap();

        let mut resolver = FileResolver::new();
        let _ = resolver.resolve_imports(&main_path);

        let result = resolver.validate_imports();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("NonExistent"));
    }
}
