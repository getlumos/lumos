// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Intermediate Representation (IR) for type definitions
//!
//! The IR is a language-agnostic representation of type definitions
//! that can be transformed into various target languages.

/// Intermediate representation of a type definition
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// Type name
    pub name: String,

    /// Fields in this type
    pub fields: Vec<FieldDefinition>,

    /// Metadata
    pub metadata: Metadata,
}

/// A field in a type definition
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// Field name
    pub name: String,

    /// Field type
    pub type_info: TypeInfo,

    /// Whether this field is optional
    pub optional: bool,
}

/// Type information
#[derive(Debug, Clone)]
pub enum TypeInfo {
    /// Primitive types (u64, string, etc.)
    Primitive(String),

    /// User-defined types
    UserDefined(String),

    /// Array types
    Array(Box<TypeInfo>),

    /// Option types
    Option(Box<TypeInfo>),
}

/// Metadata about a type
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    /// Whether this is Solana-specific
    pub solana: bool,

    /// Additional attributes
    pub attributes: Vec<String>,
}
