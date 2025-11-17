// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Error types for LUMOS core

use thiserror::Error;

/// Errors that can occur in LUMOS core
#[derive(Error, Debug)]
pub enum LumosError {
    /// Schema parsing error
    #[error("Schema parsing error: {0}")]
    SchemaParse(String),

    /// Code generation error
    #[error("Code generation error: {0}")]
    CodeGen(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML deserialization error
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Result type for LUMOS operations
pub type Result<T> = std::result::Result<T, LumosError>;
