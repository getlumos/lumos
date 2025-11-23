// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Intermediate Representation (IR) for type definitions
//!
//! The IR is a language-agnostic representation of type definitions
//! that can be transformed into various target languages.

/// Intermediate representation of a type definition (struct or enum)
#[derive(Debug, Clone)]
pub enum TypeDefinition {
    /// Struct definition
    Struct(StructDefinition),

    /// Enum definition
    Enum(EnumDefinition),
}

/// Struct type definition
#[derive(Debug, Clone)]
pub struct StructDefinition {
    /// Struct name
    pub name: String,

    /// Fields in this struct
    pub fields: Vec<FieldDefinition>,

    /// Metadata
    pub metadata: Metadata,
}

/// Enum type definition
#[derive(Debug, Clone)]
pub struct EnumDefinition {
    /// Enum name
    pub name: String,

    /// Variants in this enum
    pub variants: Vec<EnumVariantDefinition>,

    /// Metadata
    pub metadata: Metadata,
}

/// Enum variant definition
#[derive(Debug, Clone)]
pub enum EnumVariantDefinition {
    /// Unit variant (e.g., `Active`)
    Unit { name: String },

    /// Tuple variant (e.g., `PlayerJoined(PublicKey, u64)`)
    Tuple { name: String, types: Vec<TypeInfo> },

    /// Struct variant (e.g., `Initialize { authority: PublicKey }`)
    Struct {
        name: String,
        fields: Vec<FieldDefinition>,
    },
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

    /// Deprecation message (None if not deprecated)
    pub deprecated: Option<String>,
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

    /// Optional semantic version (e.g., "1.0.0")
    pub version: Option<String>,
}

impl TypeDefinition {
    /// Get the name of this type definition
    pub fn name(&self) -> &str {
        match self {
            TypeDefinition::Struct(s) => &s.name,
            TypeDefinition::Enum(e) => &e.name,
        }
    }

    /// Get the metadata for this type definition
    pub fn metadata(&self) -> &Metadata {
        match self {
            TypeDefinition::Struct(s) => &s.metadata,
            TypeDefinition::Enum(e) => &e.metadata,
        }
    }

    /// Check if this is a Solana type
    pub fn is_solana(&self) -> bool {
        self.metadata().solana
    }
}

impl EnumDefinition {
    /// Check if this enum has only unit variants
    pub fn is_unit_only(&self) -> bool {
        self.variants
            .iter()
            .all(|v| matches!(v, EnumVariantDefinition::Unit { .. }))
    }

    /// Check if this enum has struct variants
    pub fn has_struct_variants(&self) -> bool {
        self.variants
            .iter()
            .any(|v| matches!(v, EnumVariantDefinition::Struct { .. }))
    }

    /// Check if this enum has tuple variants
    pub fn has_tuple_variants(&self) -> bool {
        self.variants
            .iter()
            .any(|v| matches!(v, EnumVariantDefinition::Tuple { .. }))
    }
}

impl EnumVariantDefinition {
    /// Get the variant name
    pub fn name(&self) -> &str {
        match self {
            EnumVariantDefinition::Unit { name } => name,
            EnumVariantDefinition::Tuple { name, .. } => name,
            EnumVariantDefinition::Struct { name, .. } => name,
        }
    }
}
