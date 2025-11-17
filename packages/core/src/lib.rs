// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! # LUMOS Core
//!
//! Core schema parsing and code generation for LUMOS.
//!
//! This crate provides the fundamental building blocks for LUMOS:
//! - Schema parsing (TOML → Intermediate Representation)
//! - Code generation (IR → Rust/TypeScript)
//! - Type system and validation

/// Schema parsing and validation
pub mod schema;

/// Intermediate representation (IR) for type definitions
pub mod ir;

/// Rust code generator
pub mod generators {
    /// Generate Rust code from IR
    pub mod rust;

    /// Generate TypeScript code from IR
    pub mod typescript;
}

/// Error types for LUMOS core
pub mod error;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
