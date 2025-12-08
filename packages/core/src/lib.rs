// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! # LUMOS Core
//!
//! Core schema parsing and code generation for LUMOS - a type-safe cross-language
//! code generator for Solana blockchain development.
//!
//! ## Overview
//!
//! LUMOS enables developers to define data structures once and generate synchronized
//! code for both Rust (Anchor programs) and TypeScript (frontend SDKs) with guaranteed
//! Borsh serialization compatibility.
//!
//! ## Pipeline
//!
//! ```text
//! .lumos file → Parser → AST → Transform → IR → Generators → Rust + TypeScript
//! ```
//!
//! ## Main Components
//!
//! - **[`parser`]** - Parse `.lumos` files into Abstract Syntax Tree (AST)
//! - **[`ast`]** - AST data structures and utilities
//! - **[`transform`]** - Transform AST into Intermediate Representation (IR)
//! - **[`ir`]** - Language-agnostic intermediate representation
//! - **[`generators::rust`]** - Generate Rust code with Anchor/Borsh integration
//! - **[`generators::typescript`]** - Generate TypeScript with Borsh schemas
//!
//! ## Example Usage
//!
//! ```rust
//! use lumos_core::{parser, transform, generators};
//!
//! // Parse a .lumos schema
//! let source = r#"
//!     #[solana]
//!     #[account]
//!     struct UserAccount {
//!         wallet: PublicKey,
//!         balance: u64,
//!     }
//! "#;
//!
//! let ast = parser::parse_lumos_file(source)?;
//! let ir = transform::transform_to_ir(ast)?;
//!
//! // Generate code
//! let rust_code = generators::rust::generate_module(&ir);
//! let typescript_code = generators::typescript::generate_module(&ir);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Features
//!
//! - ✅ **Context-aware generation** - Detects Anchor vs pure Borsh usage
//! - ✅ **Smart imports** - Automatically manages dependencies
//! - ✅ **Enum support** - Unit, tuple, and struct variants
//! - ✅ **Type safety** - Guaranteed synchronization between languages
//! - ✅ **Borsh compatibility** - Automatic schema generation

/// Abstract Syntax Tree (AST) for .lumos files
pub mod ast;

/// Parser for .lumos files (builds AST from source code)
pub mod parser;

/// Schema parsing and validation (TOML format - legacy)
pub mod schema;

/// Intermediate representation (IR) for type definitions
pub mod ir;

/// Transform AST into IR
pub mod transform;

/// Multi-language code generators
///
/// Provides a unified interface for generating code in multiple languages:
/// - Rust (Anchor/Borsh)
/// - TypeScript (@coral-xyz/borsh)
/// - Python (planned)
/// - Go (planned)
/// - Ruby (planned)
///
/// See [`generators::Language`] for supported languages and
/// [`generators::CodeGenerator`] trait for the generation interface.
pub mod generators;

/// Error types for LUMOS core
pub mod error;

/// Account size calculator for Solana programs
pub mod size_calculator;

/// Security analyzer for detecting common Solana vulnerabilities
pub mod security_analyzer;

/// Security audit checklist generator
pub mod audit_generator;

/// Fuzz target generator for cargo-fuzz integration
pub mod fuzz_generator;

/// Corpus generator for fuzz testing
pub mod corpus_generator;

/// Schema migration code generation
pub mod migration;

/// Schema backward compatibility validation
pub mod compat;

/// File resolver for multi-file schemas with imports
pub mod file_resolver;

/// Module resolver for hierarchical module structures
pub mod module_resolver;

/// Anchor Framework integration (IDL generation, constraints, instructions)
pub mod anchor;

/// Metaplex Token Metadata integration (NFT schemas, validation, types)
pub mod metaplex;

/// WASM bindings for browser playground
#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
