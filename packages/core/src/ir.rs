// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Intermediate Representation (IR) for type definitions
//!
//! The IR is a language-agnostic representation of type definitions
//! that can be transformed into various target languages.

/// Visibility of a type definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    /// Public visibility (accessible from other modules)
    #[default]
    Public,
    /// Private visibility (only accessible within the same module)
    Private,
}

impl Visibility {
    /// Check if this is public visibility
    pub fn is_public(&self) -> bool {
        matches!(self, Visibility::Public)
    }

    /// Check if this is private visibility
    pub fn is_private(&self) -> bool {
        matches!(self, Visibility::Private)
    }
}

/// Intermediate representation of a type definition (struct, enum, or type alias)
#[derive(Debug, Clone)]
pub enum TypeDefinition {
    /// Struct definition
    Struct(StructDefinition),

    /// Enum definition
    Enum(EnumDefinition),

    /// Type alias definition
    TypeAlias(TypeAliasDefinition),
}

/// Type alias definition
#[derive(Debug, Clone)]
pub struct TypeAliasDefinition {
    /// Alias name (e.g., "UserId")
    pub name: String,

    /// Target type (e.g., PublicKey)
    pub target: TypeInfo,

    /// Visibility (pub or private)
    pub visibility: Visibility,

    /// Module path (e.g., ["models", "user"] for crate::models::user)
    pub module_path: Vec<String>,
}

/// Struct type definition
#[derive(Debug, Clone)]
pub struct StructDefinition {
    /// Struct name
    pub name: String,

    /// Generic type parameters (e.g., ["T", "U"])
    pub generic_params: Vec<String>,

    /// Fields in this struct
    pub fields: Vec<FieldDefinition>,

    /// Metadata
    pub metadata: Metadata,

    /// Visibility (pub or private)
    pub visibility: Visibility,

    /// Module path (e.g., ["models", "user"] for crate::models::user)
    pub module_path: Vec<String>,
}

/// Enum type definition
#[derive(Debug, Clone)]
pub struct EnumDefinition {
    /// Enum name
    pub name: String,

    /// Generic type parameters (e.g., ["T", "E"])
    pub generic_params: Vec<String>,

    /// Variants in this enum
    pub variants: Vec<EnumVariantDefinition>,

    /// Metadata
    pub metadata: Metadata,

    /// Visibility (pub or private)
    pub visibility: Visibility,

    /// Module path (e.g., ["models", "state"] for crate::models::state)
    pub module_path: Vec<String>,
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

    /// Raw anchor attribute strings (e.g., ["init, payer = authority, space = 8 + 32"])
    /// These are parsed by the anchor module during code generation
    pub anchor_attrs: Vec<String>,
}

/// Type information
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TypeInfo {
    /// Primitive types (u64, string, etc.)
    Primitive(String),

    /// Generic type parameter (T, U, K, V, etc.)
    Generic(String),

    /// User-defined types
    UserDefined(String),

    /// Dynamic array types (Vec<T>)
    Array(Box<TypeInfo>),

    /// Fixed-size array types ([T; N])
    FixedArray { element: Box<TypeInfo>, size: usize },

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

    /// Custom derive macros specified by user (e.g., PartialEq, Eq, Hash)
    /// These are added on top of auto-generated derives
    pub custom_derives: Vec<String>,

    /// Whether this struct is an Anchor instruction context (has #[instruction])
    pub is_instruction: bool,

    /// Anchor-specific struct attributes (raw strings for later parsing)
    pub anchor_attrs: Vec<String>,
}

impl TypeDefinition {
    /// Get the name of this type definition
    pub fn name(&self) -> &str {
        match self {
            TypeDefinition::Struct(s) => &s.name,
            TypeDefinition::Enum(e) => &e.name,
            TypeDefinition::TypeAlias(a) => &a.name,
        }
    }

    /// Get the metadata for this type definition (not applicable to type aliases)
    pub fn metadata(&self) -> Option<&Metadata> {
        match self {
            TypeDefinition::Struct(s) => Some(&s.metadata),
            TypeDefinition::Enum(e) => Some(&e.metadata),
            TypeDefinition::TypeAlias(_) => None, // Type aliases don't have metadata
        }
    }

    /// Check if this is a Solana type (type aliases inherit from their target)
    pub fn is_solana(&self) -> bool {
        match self {
            TypeDefinition::Struct(s) => s.metadata.solana,
            TypeDefinition::Enum(e) => e.metadata.solana,
            TypeDefinition::TypeAlias(_) => false, // Will be resolved based on target type
        }
    }

    /// Check if this is a type alias
    pub fn is_type_alias(&self) -> bool {
        matches!(self, TypeDefinition::TypeAlias(_))
    }

    /// Get the visibility of this type definition
    pub fn visibility(&self) -> Visibility {
        match self {
            TypeDefinition::Struct(s) => s.visibility,
            TypeDefinition::Enum(e) => e.visibility,
            TypeDefinition::TypeAlias(a) => a.visibility,
        }
    }

    /// Get the module path of this type definition
    pub fn module_path(&self) -> &[String] {
        match self {
            TypeDefinition::Struct(s) => &s.module_path,
            TypeDefinition::Enum(e) => &e.module_path,
            TypeDefinition::TypeAlias(a) => &a.module_path,
        }
    }

    /// Check if this type is public
    pub fn is_public(&self) -> bool {
        self.visibility().is_public()
    }
}

impl TypeAliasDefinition {
    /// Get the alias name
    pub fn alias_name(&self) -> &str {
        &self.name
    }

    /// Get the target type
    pub fn target_type(&self) -> &TypeInfo {
        &self.target
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
