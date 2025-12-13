// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Multi-Language Code Generators
//!
//! This module provides a unified interface for generating code in multiple languages
//! from LUMOS IR (Intermediate Representation). The architecture supports:
//!
//! - **Rust** - Anchor/Borsh compatible structs and enums
//! - **TypeScript** - Interfaces with Borsh schemas for web3.js
//! - **Python** - Dataclasses with borsh-python serialization
//! - **Go** - Structs with go-borsh serialization
//! - **Ruby** - Classes with borsh-rb serialization
//! - **Seahorse** - Seahorse-compatible Python for Solana programs
//!
//! ## Architecture
//!
//! ```text
//! IR (TypeDefinition[]) → CodeGenerator trait → Language-specific output
//!                              ↓
//!                    ┌─────────┴──────────┐
//!                    │   RustGenerator    │
//!                    │ TypeScriptGenerator│
//!                    │  PythonGenerator   │
//!                    │    GoGenerator     │
//!                    │   RubyGenerator    │
//!                    │ SeahorseGenerator  │
//!                    └────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use lumos_core::generators::{Language, get_generator};
//! use lumos_core::{parser, transform};
//!
//! let source = r#"
//!     #[solana]
//!     struct User { id: u64 }
//! "#;
//!
//! let ast = parser::parse_lumos_file(source)?;
//! let ir = transform::transform_to_ir(ast)?;
//!
//! // Generate for specific languages
//! let rust_gen = get_generator(Language::Rust);
//! let ts_gen = get_generator(Language::TypeScript);
//!
//! let rust_code = rust_gen.generate_module(&ir);
//! let ts_code = ts_gen.generate_module(&ir);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Adding New Languages
//!
//! 1. Create a new module (e.g., `python.rs`)
//! 2. Implement `CodeGenerator` trait
//! 3. Add variant to `Language` enum
//! 4. Update `get_generator()` factory function

use crate::ir::TypeDefinition;
use std::fmt;

/// Supported target languages for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// Rust with Anchor/Borsh support
    Rust,
    /// TypeScript with @coral-xyz/borsh
    TypeScript,
    /// Python with borsh-python
    Python,
    /// Go with go-borsh
    Go,
    /// Ruby with borsh-rb
    Ruby,
    /// Seahorse Python for Solana programs
    Seahorse,
}

impl Language {
    /// Get all currently supported languages
    pub fn supported() -> Vec<Language> {
        vec![
            Language::Rust,
            Language::TypeScript,
            Language::Python,
            Language::Go,
            Language::Ruby,
            Language::Seahorse,
        ]
    }

    /// Get all languages (including planned)
    pub fn all() -> Vec<Language> {
        vec![
            Language::Rust,
            Language::TypeScript,
            Language::Python,
            Language::Go,
            Language::Ruby,
            Language::Seahorse,
        ]
    }

    /// Check if this language is currently implemented
    pub fn is_implemented(&self) -> bool {
        matches!(
            self,
            Language::Rust
                | Language::TypeScript
                | Language::Python
                | Language::Go
                | Language::Ruby
                | Language::Seahorse
        )
    }

    /// Get the file extension for generated code
    pub fn file_extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::TypeScript => "ts",
            Language::Python => "py",
            Language::Go => "go",
            Language::Ruby => "rb",
            Language::Seahorse => "py",
        }
    }

    /// Get the canonical name for the language
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
            Language::Python => "python",
            Language::Go => "go",
            Language::Ruby => "ruby",
            Language::Seahorse => "seahorse",
        }
    }

    /// Parse language from string (case-insensitive)
    pub fn from_name(s: &str) -> Option<Language> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Some(Language::Rust),
            "typescript" | "ts" => Some(Language::TypeScript),
            "python" | "py" => Some(Language::Python),
            "go" | "golang" => Some(Language::Go),
            "ruby" | "rb" => Some(Language::Ruby),
            "seahorse" => Some(Language::Seahorse),
            _ => None,
        }
    }

    /// Parse comma-separated language list
    ///
    /// # Example
    /// ```
    /// use lumos_core::generators::Language;
    ///
    /// let langs = Language::parse_list("rust,typescript,python");
    /// assert_eq!(langs.len(), 3);
    /// ```
    pub fn parse_list(s: &str) -> Vec<Language> {
        s.split(',')
            .filter_map(|part| Language::from_name(part.trim()))
            .collect()
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Trait for language-specific code generators
///
/// All code generators must implement this trait to support the unified
/// generation pipeline. This enables polymorphic code generation and
/// easy addition of new target languages.
///
/// # Implementation Guide
///
/// When implementing a new generator:
///
/// 1. Handle all `TypeDefinition` variants (Struct, Enum, TypeAlias)
/// 2. Generate appropriate imports/dependencies
/// 3. Map IR types to target language types
/// 4. Generate serialization code (Borsh-compatible)
/// 5. Include file header with generation notice
///
/// # Example Implementation
///
/// ```rust,ignore
/// pub struct MyLanguageGenerator;
///
/// impl CodeGenerator for MyLanguageGenerator {
///     fn language(&self) -> Language {
///         Language::MyLanguage
///     }
///
///     fn generate_module(&self, type_defs: &[TypeDefinition]) -> String {
///         // Generate code...
///     }
/// }
/// ```
pub trait CodeGenerator: Send + Sync {
    /// Get the target language for this generator
    fn language(&self) -> Language;

    /// Generate code for a complete module with multiple type definitions
    ///
    /// This is the primary generation method. It should:
    /// - Add file header (auto-generated notice)
    /// - Collect and optimize imports
    /// - Generate all type definitions
    /// - Add serialization schemas where appropriate
    fn generate_module(&self, type_defs: &[TypeDefinition]) -> String;

    /// Generate code for a single type definition
    ///
    /// Useful for incremental generation or testing. Default implementation
    /// delegates to `generate_module` with a single-element slice.
    fn generate(&self, type_def: &TypeDefinition) -> String {
        self.generate_module(std::slice::from_ref(type_def))
    }

    /// Get the file extension for generated files
    fn file_extension(&self) -> &'static str {
        self.language().file_extension()
    }

    /// Get the default output filename for a schema
    fn output_filename(&self, schema_name: &str) -> String {
        format!("{}.{}", schema_name, self.file_extension())
    }
}

// Re-export existing generators
pub mod go;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod seahorse;
pub mod typescript;

/// Rust code generator implementing `CodeGenerator` trait
pub struct RustGenerator;

impl CodeGenerator for RustGenerator {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn generate_module(&self, type_defs: &[TypeDefinition]) -> String {
        rust::generate_module(type_defs)
    }

    fn generate(&self, type_def: &TypeDefinition) -> String {
        rust::generate(type_def)
    }
}

/// TypeScript code generator implementing `CodeGenerator` trait
pub struct TypeScriptGenerator;

impl CodeGenerator for TypeScriptGenerator {
    fn language(&self) -> Language {
        Language::TypeScript
    }

    fn generate_module(&self, type_defs: &[TypeDefinition]) -> String {
        typescript::generate_module(type_defs)
    }

    fn generate(&self, type_def: &TypeDefinition) -> String {
        typescript::generate(type_def)
    }
}

/// Python code generator implementing `CodeGenerator` trait
pub struct PythonGenerator;

impl CodeGenerator for PythonGenerator {
    fn language(&self) -> Language {
        Language::Python
    }

    fn generate_module(&self, type_defs: &[TypeDefinition]) -> String {
        python::generate_module(type_defs)
    }

    fn generate(&self, type_def: &TypeDefinition) -> String {
        python::generate(type_def)
    }
}

/// Go code generator implementing `CodeGenerator` trait
pub struct GoGenerator;

impl CodeGenerator for GoGenerator {
    fn language(&self) -> Language {
        Language::Go
    }

    fn generate_module(&self, type_defs: &[TypeDefinition]) -> String {
        go::generate_module(type_defs)
    }

    fn generate(&self, type_def: &TypeDefinition) -> String {
        go::generate(type_def)
    }
}

/// Ruby code generator implementing `CodeGenerator` trait
pub struct RubyGenerator;

impl CodeGenerator for RubyGenerator {
    fn language(&self) -> Language {
        Language::Ruby
    }

    fn generate_module(&self, type_defs: &[TypeDefinition]) -> String {
        ruby::generate_module(type_defs)
    }

    fn generate(&self, type_def: &TypeDefinition) -> String {
        ruby::generate(type_def)
    }
}

/// Seahorse Python code generator implementing `CodeGenerator` trait
pub struct SeahorseGenerator;

impl CodeGenerator for SeahorseGenerator {
    fn language(&self) -> Language {
        Language::Seahorse
    }

    fn generate_module(&self, type_defs: &[TypeDefinition]) -> String {
        seahorse::generate_module(type_defs)
    }

    fn generate(&self, type_def: &TypeDefinition) -> String {
        seahorse::generate(type_def)
    }
}

/// Get a code generator for the specified language
///
/// # Arguments
///
/// * `language` - Target language for code generation
///
/// # Returns
///
/// A boxed `CodeGenerator` implementation for the specified language.
///
/// # Panics
///
/// Panics if the language is not yet implemented (Python, Go, Ruby).
/// Use `Language::is_implemented()` to check before calling.
///
/// # Example
///
/// ```rust
/// use lumos_core::generators::{Language, get_generator};
///
/// let rust_gen = get_generator(Language::Rust);
/// assert_eq!(rust_gen.language(), Language::Rust);
/// assert_eq!(rust_gen.file_extension(), "rs");
/// ```
pub fn get_generator(language: Language) -> Box<dyn CodeGenerator> {
    match language {
        Language::Rust => Box::new(RustGenerator),
        Language::TypeScript => Box::new(TypeScriptGenerator),
        Language::Python => Box::new(PythonGenerator),
        Language::Go => Box::new(GoGenerator),
        Language::Ruby => Box::new(RubyGenerator),
        Language::Seahorse => Box::new(SeahorseGenerator),
    }
}

/// Try to get a code generator, returning None if not implemented
///
/// Safer alternative to `get_generator()` that doesn't panic.
///
/// # Example
///
/// ```rust
/// use lumos_core::generators::{Language, try_get_generator};
///
/// // Implemented languages
/// assert!(try_get_generator(Language::Rust).is_some());
/// assert!(try_get_generator(Language::Go).is_some());
/// assert!(try_get_generator(Language::Ruby).is_some());
/// ```
pub fn try_get_generator(language: Language) -> Option<Box<dyn CodeGenerator>> {
    match language {
        Language::Rust => Some(Box::new(RustGenerator)),
        Language::TypeScript => Some(Box::new(TypeScriptGenerator)),
        Language::Python => Some(Box::new(PythonGenerator)),
        Language::Go => Some(Box::new(GoGenerator)),
        Language::Ruby => Some(Box::new(RubyGenerator)),
        Language::Seahorse => Some(Box::new(SeahorseGenerator)),
    }
}

/// Get generators for multiple languages
///
/// Returns generators for all specified languages in the list.
///
/// # Example
///
/// ```rust
/// use lumos_core::generators::{Language, get_generators};
///
/// let langs = vec![Language::Rust, Language::TypeScript, Language::Python, Language::Go, Language::Ruby];
/// let generators = get_generators(&langs);
///
/// // All 5 languages are implemented
/// assert_eq!(generators.len(), 5);
/// ```
pub fn get_generators(languages: &[Language]) -> Vec<Box<dyn CodeGenerator>> {
    languages
        .iter()
        .filter_map(|lang| try_get_generator(*lang))
        .collect()
}

/// Generate code for all specified languages
///
/// Returns a vector of (Language, generated_code) tuples.
///
/// # Example
///
/// ```rust
/// use lumos_core::generators::{Language, generate_for_languages};
/// use lumos_core::ir::{TypeDefinition, StructDefinition, Metadata, Visibility};
///
/// let type_defs = vec![TypeDefinition::Struct(StructDefinition {
///     name: "User".to_string(),
///     generic_params: vec![],
///     fields: vec![],
///     metadata: Metadata::default(),
///     visibility: Visibility::Public,
///     module_path: Vec::new(),
/// })];
///
/// let results = generate_for_languages(&type_defs, &[Language::Rust, Language::TypeScript]);
/// assert_eq!(results.len(), 2);
/// ```
pub fn generate_for_languages(
    type_defs: &[TypeDefinition],
    languages: &[Language],
) -> Vec<(Language, String)> {
    get_generators(languages)
        .iter()
        .map(|gen| (gen.language(), gen.generate_module(type_defs)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{FieldDefinition, Metadata, StructDefinition, TypeInfo, Visibility};

    #[test]
    fn test_language_from_name() {
        assert_eq!(Language::from_name("rust"), Some(Language::Rust));
        assert_eq!(Language::from_name("RS"), Some(Language::Rust));
        assert_eq!(
            Language::from_name("typescript"),
            Some(Language::TypeScript)
        );
        assert_eq!(Language::from_name("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_name("python"), Some(Language::Python));
        assert_eq!(Language::from_name("py"), Some(Language::Python));
        assert_eq!(Language::from_name("go"), Some(Language::Go));
        assert_eq!(Language::from_name("golang"), Some(Language::Go));
        assert_eq!(Language::from_name("ruby"), Some(Language::Ruby));
        assert_eq!(Language::from_name("rb"), Some(Language::Ruby));
        assert_eq!(Language::from_name("invalid"), None);
    }

    #[test]
    fn test_language_parse_list() {
        let langs = Language::parse_list("rust,typescript,python");
        assert_eq!(langs.len(), 3);
        assert_eq!(langs[0], Language::Rust);
        assert_eq!(langs[1], Language::TypeScript);
        assert_eq!(langs[2], Language::Python);

        // With whitespace
        let langs = Language::parse_list("rust , ts , py");
        assert_eq!(langs.len(), 3);

        // Invalid entries filtered
        let langs = Language::parse_list("rust,invalid,typescript");
        assert_eq!(langs.len(), 2);
    }

    #[test]
    fn test_language_file_extension() {
        assert_eq!(Language::Rust.file_extension(), "rs");
        assert_eq!(Language::TypeScript.file_extension(), "ts");
        assert_eq!(Language::Python.file_extension(), "py");
        assert_eq!(Language::Go.file_extension(), "go");
        assert_eq!(Language::Ruby.file_extension(), "rb");
    }

    #[test]
    fn test_language_is_implemented() {
        assert!(Language::Rust.is_implemented());
        assert!(Language::TypeScript.is_implemented());
        assert!(Language::Python.is_implemented());
        assert!(Language::Go.is_implemented());
        assert!(Language::Ruby.is_implemented());
    }

    #[test]
    fn test_get_generator_rust() {
        let gen = get_generator(Language::Rust);
        assert_eq!(gen.language(), Language::Rust);
        assert_eq!(gen.file_extension(), "rs");
    }

    #[test]
    fn test_get_generator_typescript() {
        let gen = get_generator(Language::TypeScript);
        assert_eq!(gen.language(), Language::TypeScript);
        assert_eq!(gen.file_extension(), "ts");
    }

    #[test]
    fn test_try_get_generator() {
        assert!(try_get_generator(Language::Rust).is_some());
        assert!(try_get_generator(Language::TypeScript).is_some());
        assert!(try_get_generator(Language::Python).is_some());
        assert!(try_get_generator(Language::Go).is_some());
        assert!(try_get_generator(Language::Ruby).is_some());
        assert!(try_get_generator(Language::Seahorse).is_some());
    }

    #[test]
    fn test_get_generators_all_implemented() {
        let langs = vec![
            Language::Rust,
            Language::TypeScript,
            Language::Python,
            Language::Go,
            Language::Ruby,
            Language::Seahorse,
        ];
        let generators = get_generators(&langs);

        // All 6 languages are implemented
        assert_eq!(generators.len(), 6);
        assert_eq!(generators[0].language(), Language::Rust);
        assert_eq!(generators[1].language(), Language::TypeScript);
        assert_eq!(generators[2].language(), Language::Python);
        assert_eq!(generators[3].language(), Language::Go);
        assert_eq!(generators[4].language(), Language::Ruby);
        assert_eq!(generators[5].language(), Language::Seahorse);
    }

    #[test]
    fn test_generate_for_languages() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "TestStruct".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "value".to_string(),
                type_info: TypeInfo::Primitive("u64".to_string()),
                optional: false,
                deprecated: None,
                span: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let results = generate_for_languages(&type_defs, &[Language::Rust, Language::TypeScript]);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, Language::Rust);
        assert!(results[0].1.contains("pub struct TestStruct"));
        assert_eq!(results[1].0, Language::TypeScript);
        assert!(results[1].1.contains("export interface TestStruct"));
    }

    #[test]
    fn test_rust_generator_output() {
        let gen = RustGenerator;
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "User".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "id".to_string(),
                type_info: TypeInfo::Primitive("u64".to_string()),
                optional: false,
                deprecated: None,
                span: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let code = gen.generate_module(&type_defs);
        assert!(code.contains("// Auto-generated by LUMOS"));
        assert!(code.contains("pub struct User"));
        assert!(code.contains("pub id: u64"));
    }

    #[test]
    fn test_typescript_generator_output() {
        let gen = TypeScriptGenerator;
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "User".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "id".to_string(),
                type_info: TypeInfo::Primitive("u64".to_string()),
                optional: false,
                deprecated: None,
                span: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let code = gen.generate_module(&type_defs);
        assert!(code.contains("// Auto-generated by LUMOS"));
        assert!(code.contains("export interface User"));
        assert!(code.contains("id: number"));
    }

    #[test]
    fn test_python_generator_output() {
        let gen = PythonGenerator;
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "User".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "id".to_string(),
                type_info: TypeInfo::Primitive("u64".to_string()),
                optional: false,
                deprecated: None,
                span: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let code = gen.generate_module(&type_defs);
        assert!(code.contains("# Auto-generated by LUMOS"));
        assert!(code.contains("@dataclass"));
        assert!(code.contains("class User:"));
        assert!(code.contains("id: int"));
    }

    #[test]
    fn test_get_generator_python() {
        let gen = get_generator(Language::Python);
        assert_eq!(gen.language(), Language::Python);
        assert_eq!(gen.file_extension(), "py");
    }

    #[test]
    fn test_go_generator_output() {
        let gen = GoGenerator;
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "User".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "id".to_string(),
                type_info: TypeInfo::Primitive("u64".to_string()),
                optional: false,
                deprecated: None,
                span: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let code = gen.generate_module(&type_defs);
        assert!(code.contains("// Auto-generated by LUMOS"));
        assert!(code.contains("package generated"));
        assert!(code.contains("type User struct {"));
        assert!(code.contains("Id uint64"));
    }

    #[test]
    fn test_get_generator_go() {
        let gen = get_generator(Language::Go);
        assert_eq!(gen.language(), Language::Go);
        assert_eq!(gen.file_extension(), "go");
    }

    #[test]
    fn test_ruby_generator_output() {
        let gen = RubyGenerator;
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "User".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "id".to_string(),
                type_info: TypeInfo::Primitive("u64".to_string()),
                optional: false,
                deprecated: None,
                span: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let code = gen.generate_module(&type_defs);
        assert!(code.contains("# Auto-generated by LUMOS"));
        assert!(code.contains("class User"));
        assert!(code.contains("attr_accessor :id"));
    }

    #[test]
    fn test_get_generator_ruby() {
        let gen = get_generator(Language::Ruby);
        assert_eq!(gen.language(), Language::Ruby);
        assert_eq!(gen.file_extension(), "rb");
    }

    #[test]
    fn test_output_filename() {
        let rust_gen = RustGenerator;
        let ts_gen = TypeScriptGenerator;

        assert_eq!(rust_gen.output_filename("schema"), "schema.rs");
        assert_eq!(ts_gen.output_filename("schema"), "schema.ts");
    }

    #[test]
    fn test_language_supported() {
        let supported = Language::supported();
        assert_eq!(supported.len(), 6);
        assert!(supported.contains(&Language::Rust));
        assert!(supported.contains(&Language::TypeScript));
        assert!(supported.contains(&Language::Python));
        assert!(supported.contains(&Language::Go));
        assert!(supported.contains(&Language::Ruby));
        assert!(supported.contains(&Language::Seahorse));
    }

    #[test]
    fn test_language_all() {
        let all = Language::all();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_seahorse_generator() {
        let gen = get_generator(Language::Seahorse);
        assert_eq!(gen.language(), Language::Seahorse);
        assert_eq!(gen.file_extension(), "py");
    }

    #[test]
    fn test_seahorse_generator_output() {
        let gen = SeahorseGenerator;
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "PlayerAccount".to_string(),
            generic_params: vec![],
            fields: vec![
                FieldDefinition {
                    name: "wallet".to_string(),
                    type_info: TypeInfo::Primitive("PublicKey".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "level".to_string(),
                    type_info: TypeInfo::Primitive("u16".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
            ],
            metadata: Metadata {
                solana: true,
                attributes: vec!["account".to_string()],
                version: None,
                custom_derives: vec![],
                is_instruction: false,
                anchor_attrs: vec![],
            },
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let code = gen.generate_module(&type_defs);
        assert!(code.contains("from seahorse.prelude import *"));
        assert!(code.contains("@account"));
        assert!(code.contains("class PlayerAccount:"));
        assert!(code.contains("wallet: Pubkey"));
        assert!(code.contains("level: u16"));
    }

    #[test]
    fn test_language_display() {
        assert_eq!(format!("{}", Language::Rust), "rust");
        assert_eq!(format!("{}", Language::TypeScript), "typescript");
        assert_eq!(format!("{}", Language::Python), "python");
    }
}
