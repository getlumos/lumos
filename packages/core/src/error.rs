// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Error types for LUMOS core

use thiserror::Error;

/// Source location information for error reporting
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

impl SourceLocation {
    /// Create a new source location
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Format location as "line:column"
    pub fn format(&self) -> String {
        format!("{}:{}", self.line, self.column)
    }
}

/// Errors that can occur in LUMOS core
#[derive(Error, Debug)]
pub enum LumosError {
    /// Schema parsing error with optional source location
    #[error("{}{}", .0, .1.as_ref().map(|loc| format!(" at line {}, column {}", loc.line, loc.column)).unwrap_or_default())]
    SchemaParse(String, Option<SourceLocation>),

    /// Code generation error
    #[error("Code generation error: {0}")]
    CodeGen(String),

    /// Type validation error with optional source location
    #[error("{}{}", .0, .1.as_ref().map(|loc| format!(" (at {})", loc.format())).unwrap_or_default())]
    TypeValidation(String, Option<SourceLocation>),

    /// Transformation error (AST to IR) with optional source location
    #[error("{}{}", .0, .1.as_ref().map(|loc| format!(" (at {})", loc.format())).unwrap_or_default())]
    Transform(String, Option<SourceLocation>),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML deserialization error
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Result type for LUMOS operations
pub type Result<T> = std::result::Result<T, LumosError>;
