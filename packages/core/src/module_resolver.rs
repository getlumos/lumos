// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Module resolver for handling hierarchical LUMOS module structures
//!
//! This module provides functionality to:
//! - Resolve `mod name;` declarations to files (name.lumos or name/mod.lumos)
//! - Build a module tree structure
//! - Resolve `use` statements with path resolution (crate::, super::, self::)
//! - Detect circular module dependencies
//! - Cache loaded modules to avoid duplicate parsing

use crate::ast::{Item as AstItem, LumosFile};
use crate::error::{LumosError, Result};
use crate::ir::TypeDefinition;
use crate::parser::parse_lumos_file;
use crate::transform::{
    transform_to_ir_with_resolver_no_validation, validate_user_defined_types, TypeAliasResolver,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a node in the module tree
#[derive(Debug, Clone)]
pub struct ModuleNode {
    /// Module name (empty string for root)
    pub name: String,

    /// Absolute file path
    pub file_path: PathBuf,

    /// Parsed AST
    pub ast: LumosFile,

    /// Child module names (name -> child path)
    pub children: HashMap<String, PathBuf>,

    /// Parent module path (None for root)
    pub parent: Option<PathBuf>,
}

/// Module resolver that handles hierarchical module loading
#[derive(Debug)]
pub struct ModuleResolver {
    /// Map of absolute path -> module node
    modules: HashMap<PathBuf, ModuleNode>,

    /// Root module path
    root_path: PathBuf,

    /// Stack of modules currently being loaded (for circular dependency detection)
    loading_stack: Vec<PathBuf>,
}

impl ModuleResolver {
    /// Create a new module resolver
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            root_path: PathBuf::new(),
            loading_stack: Vec::new(),
        }
    }

    /// Resolve all modules starting from an entry file and return combined type definitions
    ///
    /// This is the main entry point for the module resolver. It:
    /// 1. Loads the entry file as the root module
    /// 2. Recursively loads all child modules declared with `mod name;`
    /// 3. Resolves all type aliases across all modules
    /// 4. Transforms all modules to IR
    /// 5. Validates all user-defined types
    pub fn resolve_modules(&mut self, entry_file: &Path) -> Result<Vec<TypeDefinition>> {
        // Canonicalize the entry file path
        let entry_path = self.canonicalize_path(entry_file)?;
        self.root_path = entry_path.clone();

        // Load the root module and all its dependencies
        self.load_module_recursive(&entry_path, None)?;

        // Validate all use statements
        self.validate_use_statements()?;

        // First pass: Collect all type aliases from all modules
        let mut resolver = TypeAliasResolver::new();
        for module_node in self.modules.values() {
            for item in &module_node.ast.items {
                if let AstItem::TypeAlias(alias_def) = item {
                    resolver.add_alias(alias_def.name.clone(), alias_def.target.clone())?;
                }
            }
        }

        // Resolve all aliases recursively
        resolver.resolve_all_aliases()?;

        // Second pass: Transform all modules with shared resolver (skip per-file validation)
        let mut all_type_defs = Vec::new();
        for module_node in self.modules.values() {
            let type_defs =
                transform_to_ir_with_resolver_no_validation(module_node.ast.clone(), &resolver)?;
            all_type_defs.extend(type_defs);
        }

        // Third pass: Validate all user-defined types across all modules
        validate_user_defined_types(&all_type_defs)?;

        Ok(all_type_defs)
    }

    /// Recursively load a module and all its child modules
    fn load_module_recursive(&mut self, file_path: &Path, parent: Option<PathBuf>) -> Result<()> {
        // Check if already loaded (cache hit)
        if self.modules.contains_key(file_path) {
            return Ok(());
        }

        // Check for circular dependencies
        if self.loading_stack.contains(&file_path.to_path_buf()) {
            return Err(LumosError::SchemaParse(
                format!(
                    "Circular module dependency detected: {} -> {}",
                    self.format_loading_chain(),
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

        let ast = parse_lumos_file(&content)?;

        // Get the directory of the current file for resolving relative modules
        let current_dir = file_path.parent().ok_or_else(|| {
            LumosError::SchemaParse(format!("Invalid file path: {}", file_path.display()), None)
        })?;

        // Extract child module declarations
        let mut children = HashMap::new();
        for item in &ast.items {
            if let AstItem::Module(module) = item {
                let child_path = self.resolve_module_path(current_dir, &module.name)?;
                children.insert(module.name.clone(), child_path.clone());

                // Recursively load the child module
                self.load_module_recursive(&child_path, Some(file_path.to_path_buf()))?;
            }
        }

        // Create module node
        let module_name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let module_node = ModuleNode {
            name: if file_path == self.root_path {
                String::new() // Root module has empty name
            } else {
                module_name
            },
            file_path: file_path.to_path_buf(),
            ast,
            children,
            parent,
        };

        // Remove from loading stack
        self.loading_stack.pop();

        // Add to modules map
        self.modules.insert(file_path.to_path_buf(), module_node);

        Ok(())
    }

    /// Resolve a module declaration to a file path
    ///
    /// Tries two strategies:
    /// 1. Sibling file: `current_dir/name.lumos`
    /// 2. Directory module: `current_dir/name/mod.lumos`
    ///
    /// Prefers sibling file if both exist.
    fn resolve_module_path(&self, current_dir: &Path, module_name: &str) -> Result<PathBuf> {
        // Strategy 1: Sibling file (name.lumos)
        let sibling_path = current_dir.join(format!("{}.lumos", module_name));
        if sibling_path.exists() {
            return self.canonicalize_path(&sibling_path);
        }

        // Strategy 2: Directory module (name/mod.lumos)
        let dir_module_path = current_dir.join(module_name).join("mod.lumos");
        if dir_module_path.exists() {
            return self.canonicalize_path(&dir_module_path);
        }

        // Neither exists - error
        Err(LumosError::SchemaParse(
            format!(
                "Module '{}' not found. Tried:\n  - {}\n  - {}",
                module_name,
                sibling_path.display(),
                dir_module_path.display()
            ),
            None,
        ))
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

    /// Format the current loading chain for error messages
    fn format_loading_chain(&self) -> String {
        self.loading_stack
            .iter()
            .map(|p| p.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"))
            .collect::<Vec<_>>()
            .join(" -> ")
    }

    /// Get all loaded module paths
    pub fn loaded_modules(&self) -> Vec<&Path> {
        self.modules.keys().map(|p| p.as_path()).collect()
    }

    /// Get the module tree structure (for debugging/visualization)
    pub fn module_tree(&self) -> Option<&ModuleNode> {
        self.modules.get(&self.root_path)
    }

    /// Validate all use statements in all modules
    ///
    /// Checks that:
    /// 1. The path can be resolved to a module
    /// 2. The referenced type exists
    /// 3. The type is accessible (visibility check)
    pub fn validate_use_statements(&self) -> Result<()> {
        for (module_path, module_node) in &self.modules {
            for item in &module_node.ast.items {
                if let AstItem::Use(use_stmt) = item {
                    self.validate_use_statement(module_path, use_stmt)?;
                }
            }
        }
        Ok(())
    }

    /// Validate a single use statement
    fn validate_use_statement(
        &self,
        current_module_path: &Path,
        use_stmt: &crate::ast::UseStatement,
    ) -> Result<()> {
        use crate::ast::Visibility;

        // Get the final identifier (the type being imported)
        let type_name = use_stmt.path.final_ident().ok_or_else(|| {
            LumosError::SchemaParse(format!("Invalid use path: {}", use_stmt.path), None)
        })?;

        // Resolve the module path
        let target_module_path =
            self.resolve_use_path(current_module_path, &use_stmt.path.segments)?;

        // Find the type in the target module
        let target_module = self.modules.get(&target_module_path).ok_or_else(|| {
            LumosError::SchemaParse(
                format!("Module not found for path: {}", use_stmt.path),
                None,
            )
        })?;

        // Check if the type exists in the target module
        let mut type_found = false;
        let mut type_visibility = Visibility::Private;

        for item in &target_module.ast.items {
            let (name, visibility) = match item {
                AstItem::Struct(s) => (Some(&s.name), &s.visibility),
                AstItem::Enum(e) => (Some(&e.name), &e.visibility),
                AstItem::TypeAlias(t) => (Some(&t.name), &t.visibility),
                AstItem::Module(_) | AstItem::Use(_) => (None, &Visibility::Private),
            };

            if let Some(name) = name {
                if name == type_name {
                    type_found = true;
                    type_visibility = visibility.clone();
                    break;
                }
            }
        }

        if !type_found {
            return Err(LumosError::SchemaParse(
                format!(
                    "Type '{}' not found in module '{}'",
                    type_name,
                    target_module_path.display()
                ),
                None,
            ));
        }

        // Check visibility (type must be public if importing from different module)
        if target_module_path != current_module_path && type_visibility == Visibility::Private {
            return Err(LumosError::SchemaParse(
                format!(
                    "Type '{}' is private and cannot be imported from '{}'",
                    type_name,
                    target_module_path.display()
                ),
                None,
            ));
        }

        Ok(())
    }

    /// Resolve a use path to an absolute module path
    ///
    /// Handles:
    /// - `crate::models::User` → resolve from root
    /// - `super::User` → resolve from parent
    /// - `self::types::UserId` → resolve from current
    /// - `models::User` → resolve from current (implicit self)
    fn resolve_use_path(
        &self,
        current_module_path: &Path,
        segments: &[crate::ast::PathSegment],
    ) -> Result<PathBuf> {
        use crate::ast::PathSegment;

        if segments.is_empty() {
            return Ok(current_module_path.to_path_buf());
        }

        let mut target_path = match &segments[0] {
            PathSegment::Crate => {
                // Absolute path from root
                self.root_path.clone()
            }
            PathSegment::Super => {
                // Relative to parent module
                let current_module = self.modules.get(current_module_path).ok_or_else(|| {
                    LumosError::SchemaParse(
                        format!(
                            "Current module not found: {}",
                            current_module_path.display()
                        ),
                        None,
                    )
                })?;

                current_module.parent.clone().ok_or_else(|| {
                    LumosError::SchemaParse("Cannot use 'super' in root module".to_string(), None)
                })?
            }
            PathSegment::SelfPath => {
                // Relative to current module
                current_module_path.to_path_buf()
            }
            PathSegment::Ident(name) => {
                // Implicit self - relative to current module
                let current_module = self.modules.get(current_module_path).ok_or_else(|| {
                    LumosError::SchemaParse(
                        format!(
                            "Current module not found: {}",
                            current_module_path.display()
                        ),
                        None,
                    )
                })?;

                // Check if this is a child module
                if let Some(child_path) = current_module.children.get(name) {
                    return self.resolve_use_path(child_path, &segments[1..]);
                } else {
                    // If not a child module, the type should be in the current module
                    return Ok(current_module_path.to_path_buf());
                }
            }
        };

        // Traverse the remaining path segments
        for segment in &segments[1..] {
            match segment {
                PathSegment::Ident(name) => {
                    let current_module = self.modules.get(&target_path).ok_or_else(|| {
                        LumosError::SchemaParse(
                            format!("Module not found: {}", target_path.display()),
                            None,
                        )
                    })?;

                    // Check if this is a child module
                    if let Some(child_path) = current_module.children.get(name) {
                        target_path = child_path.clone();
                    } else {
                        // Not a child module, so this must be the final type name
                        // Return the current module path
                        return Ok(target_path);
                    }
                }
                PathSegment::Super => {
                    let current_module = self.modules.get(&target_path).ok_or_else(|| {
                        LumosError::SchemaParse(
                            format!("Module not found: {}", target_path.display()),
                            None,
                        )
                    })?;

                    target_path = current_module.parent.clone().ok_or_else(|| {
                        LumosError::SchemaParse(
                            format!(
                                "Cannot use 'super' - module '{}' has no parent",
                                target_path.display()
                            ),
                            None,
                        )
                    })?;
                }
                PathSegment::Crate | PathSegment::SelfPath => {
                    return Err(LumosError::SchemaParse(
                        format!(
                            "'{}' can only appear at the start of a path",
                            if matches!(segment, PathSegment::Crate) {
                                "crate"
                            } else {
                                "self"
                            }
                        ),
                        None,
                    ));
                }
            }
        }

        Ok(target_path)
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_single_module() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("main.lumos");

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

        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve_modules(&file_path);

        assert!(result.is_ok());
        let type_defs = result.unwrap();
        assert_eq!(type_defs.len(), 1);
    }

    #[test]
    fn test_resolve_sibling_module() {
        let temp_dir = TempDir::new().unwrap();

        // Create models.lumos (sibling file)
        let models_path = temp_dir.path().join("models.lumos");
        fs::write(
            &models_path,
            r#"
pub struct User {
    id: u64,
    name: String,
}
"#,
        )
        .unwrap();

        // Create main.lumos that declares models module
        let main_path = temp_dir.path().join("main.lumos");
        fs::write(
            &main_path,
            r#"
mod models;

struct App {
    version: u64,
}
"#,
        )
        .unwrap();

        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve_modules(&main_path);

        assert!(result.is_ok());
        assert_eq!(resolver.loaded_modules().len(), 2);
    }

    #[test]
    fn test_resolve_directory_module() {
        let temp_dir = TempDir::new().unwrap();

        // Create models/ directory with mod.lumos
        let models_dir = temp_dir.path().join("models");
        fs::create_dir(&models_dir).unwrap();
        let mod_path = models_dir.join("mod.lumos");
        fs::write(
            &mod_path,
            r#"
pub struct User {
    id: u64,
}
"#,
        )
        .unwrap();

        // Create main.lumos
        let main_path = temp_dir.path().join("main.lumos");
        fs::write(
            &main_path,
            r#"
mod models;
"#,
        )
        .unwrap();

        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve_modules(&main_path);

        assert!(result.is_ok());
        assert_eq!(resolver.loaded_modules().len(), 2);
    }

    #[test]
    fn test_detect_circular_module_dependency() {
        let temp_dir = TempDir::new().unwrap();

        // Create a.lumos that declares mod b
        let a_path = temp_dir.path().join("a.lumos");
        fs::write(&a_path, "mod b;").unwrap();

        // Create b.lumos that declares mod a (circular!)
        let b_path = temp_dir.path().join("b.lumos");
        fs::write(&b_path, "mod a;").unwrap();

        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve_modules(&a_path);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Circular module dependency"));
    }

    #[test]
    fn test_module_not_found() {
        let temp_dir = TempDir::new().unwrap();

        // Create main.lumos that declares non-existent module
        let main_path = temp_dir.path().join("main.lumos");
        fs::write(&main_path, "mod nonexistent;").unwrap();

        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve_modules(&main_path);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Module 'nonexistent' not found"));
    }

    #[test]
    fn test_nested_modules() {
        let temp_dir = TempDir::new().unwrap();

        // Create types.lumos
        let types_path = temp_dir.path().join("types.lumos");
        fs::write(&types_path, "pub struct UserId { value: u64 }").unwrap();

        // Create models/ directory
        let models_dir = temp_dir.path().join("models");
        fs::create_dir(&models_dir).unwrap();

        // Create models/mod.lumos that declares user module
        let models_mod_path = models_dir.join("mod.lumos");
        fs::write(&models_mod_path, "pub mod user;").unwrap();

        // Create models/user.lumos
        let user_path = models_dir.join("user.lumos");
        fs::write(
            &user_path,
            r#"
pub struct User {
    id: u64,
}
"#,
        )
        .unwrap();

        // Create main.lumos
        let main_path = temp_dir.path().join("main.lumos");
        fs::write(
            &main_path,
            r#"
mod types;
mod models;
"#,
        )
        .unwrap();

        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve_modules(&main_path);

        if let Err(e) = &result {
            eprintln!("Error: {}", e);
        }
        assert!(result.is_ok());
        assert_eq!(resolver.loaded_modules().len(), 4); // main, types, models/mod, models/user
    }

    #[test]
    fn test_use_statement_valid() {
        let temp_dir = TempDir::new().unwrap();

        // Create types.lumos
        let types_path = temp_dir.path().join("types.lumos");
        fs::write(
            &types_path,
            r#"
pub struct UserId { value: u64 }
pub type Timestamp = i64;
"#,
        )
        .unwrap();

        // Create main.lumos that uses types from types module
        let main_path = temp_dir.path().join("main.lumos");
        fs::write(
            &main_path,
            r#"
mod types;
use types::UserId;
use types::Timestamp;

pub struct User {
    id: UserId,
    created: Timestamp,
}
"#,
        )
        .unwrap();

        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve_modules(&main_path);

        assert!(result.is_ok());
    }

    #[test]
    fn test_use_statement_with_crate() {
        let temp_dir = TempDir::new().unwrap();

        // Create types.lumos
        let types_path = temp_dir.path().join("types.lumos");
        fs::write(&types_path, "pub struct UserId { value: u64 }").unwrap();

        // Create main.lumos
        let main_path = temp_dir.path().join("main.lumos");
        fs::write(
            &main_path,
            r#"
mod types;
use crate::types::UserId;

pub struct User { id: UserId }
"#,
        )
        .unwrap();

        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve_modules(&main_path);

        assert!(result.is_ok());
    }

    #[test]
    fn test_use_statement_type_not_found() {
        let temp_dir = TempDir::new().unwrap();

        // Create types.lumos without UserId
        let types_path = temp_dir.path().join("types.lumos");
        fs::write(&types_path, "pub type Timestamp = i64;").unwrap();

        // Create main.lumos that tries to use non-existent UserId
        let main_path = temp_dir.path().join("main.lumos");
        fs::write(
            &main_path,
            r#"
mod types;
use types::UserId;
"#,
        )
        .unwrap();

        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve_modules(&main_path);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Type 'UserId' not found"));
    }

    #[test]
    fn test_use_statement_private_type() {
        let temp_dir = TempDir::new().unwrap();

        // Create types.lumos with private UserId using pub(crate) for explicit private
        // Note: LUMOS defaults to public visibility, so we use pub(crate) to make it private
        let types_path = temp_dir.path().join("types.lumos");
        fs::write(&types_path, "pub(crate) struct UserId { value: u64 }").unwrap();

        // Create main.lumos that tries to use private UserId
        let main_path = temp_dir.path().join("main.lumos");
        fs::write(
            &main_path,
            r#"
mod types;
use types::UserId;
"#,
        )
        .unwrap();

        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve_modules(&main_path);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("is private and cannot be imported"));
    }

    #[test]
    fn test_use_statement_with_super() {
        let temp_dir = TempDir::new().unwrap();

        // Create main.lumos
        let main_path = temp_dir.path().join("main.lumos");
        fs::write(
            &main_path,
            r#"
pub struct Config { value: u64 }
pub mod sub;
"#,
        )
        .unwrap();

        // Create sub.lumos that uses super::Config
        let sub_path = temp_dir.path().join("sub.lumos");
        fs::write(
            &sub_path,
            r#"
use super::Config;

pub struct App { config: Config }
"#,
        )
        .unwrap();

        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve_modules(&main_path);

        assert!(result.is_ok());
    }

    #[test]
    fn test_use_statement_nested_modules() {
        let temp_dir = TempDir::new().unwrap();

        // Create types/mod.lumos
        let types_dir = temp_dir.path().join("types");
        fs::create_dir(&types_dir).unwrap();
        let types_mod_path = types_dir.join("mod.lumos");
        fs::write(
            &types_mod_path,
            r#"
pub mod common;
"#,
        )
        .unwrap();

        // Create types/common.lumos
        let common_path = types_dir.join("common.lumos");
        fs::write(
            &common_path,
            r#"
pub struct UserId { value: u64 }
"#,
        )
        .unwrap();

        // Create main.lumos
        let main_path = temp_dir.path().join("main.lumos");
        fs::write(
            &main_path,
            r#"
mod types;
use crate::types::common::UserId;

pub struct User { id: UserId }
"#,
        )
        .unwrap();

        let mut resolver = ModuleResolver::new();
        let result = resolver.resolve_modules(&main_path);

        assert!(result.is_ok());
    }
}
