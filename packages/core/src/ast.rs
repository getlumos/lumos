// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Abstract Syntax Tree (AST) for LUMOS
//!
//! This module defines the AST representation of `.lumos` files.
//! The AST is a direct representation of the parsed syntax,
//! before transformation into the IR.

use serde::{Deserialize, Serialize};

/// A complete LUMOS file (can contain multiple items)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LumosFile {
    /// Import statements at the top of the file
    pub imports: Vec<Import>,

    /// All items (structs, enums, type aliases) in this file
    pub items: Vec<Item>,
}

/// An import statement (JavaScript-style, legacy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    /// Items being imported (e.g., ["UserId", "Timestamp"])
    pub items: Vec<String>,

    /// Path to the file being imported (e.g., "./types.lumos")
    pub path: String,

    /// Span information for error reporting
    #[serde(skip)]
    pub span: Option<proc_macro2::Span>,
}

/// A module declaration (Rust-style: `mod name;`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// Module name (e.g., "models")
    pub name: String,

    /// Visibility (pub or private)
    pub visibility: Visibility,

    /// Span information for error reporting
    #[serde(skip)]
    pub span: Option<proc_macro2::Span>,
}

/// A use statement (Rust-style: `use path::Type;`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseStatement {
    /// Module path (e.g., crate::models::User)
    pub path: ModulePath,

    /// Optional alias (e.g., `use path::Type as Alias;`)
    pub alias: Option<String>,

    /// Span information for error reporting
    #[serde(skip)]
    pub span: Option<proc_macro2::Span>,
}

/// Module path (e.g., crate::models::User)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModulePath {
    /// Path segments (e.g., [Crate, Ident("models"), Ident("User")])
    pub segments: Vec<PathSegment>,
}

/// A segment in a module path
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PathSegment {
    /// `crate` keyword
    Crate,

    /// `super` keyword
    Super,

    /// `self` keyword
    SelfPath,

    /// Identifier (module or type name)
    Ident(String),
}

/// Visibility modifier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Visibility {
    /// Private (no `pub` keyword)
    #[default]
    Private,

    /// Public (`pub` keyword)
    Public,
}

/// A type alias definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeAlias {
    /// Alias name (e.g., "UserId")
    pub name: String,

    /// Visibility (pub or private)
    pub visibility: Visibility,

    /// Target type (e.g., PublicKey)
    pub target: TypeSpec,

    /// Span information for error reporting
    #[serde(skip)]
    pub span: Option<proc_macro2::Span>,
}

/// An item in a LUMOS file (struct, enum, type alias, module, or use statement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Item {
    /// Struct definition
    Struct(StructDef),

    /// Enum definition
    Enum(EnumDef),

    /// Type alias definition
    TypeAlias(TypeAlias),

    /// Module declaration (Rust-style)
    Module(Module),

    /// Use statement (Rust-style)
    Use(UseStatement),
}

/// A struct definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDef {
    /// Struct name (e.g., "UserAccount")
    pub name: String,

    /// Visibility (pub or private)
    pub visibility: Visibility,

    /// Attributes applied to the struct (e.g., @solana, @account)
    pub attributes: Vec<Attribute>,

    /// Fields in this struct
    pub fields: Vec<FieldDef>,

    /// Optional semantic version (e.g., "1.0.0" from #[version = "1.0.0"])
    pub version: Option<String>,

    /// Span information for error reporting
    #[serde(skip)]
    pub span: Option<proc_macro2::Span>,
}

/// An enum definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDef {
    /// Enum name (e.g., "GameState")
    pub name: String,

    /// Visibility (pub or private)
    pub visibility: Visibility,

    /// Attributes applied to the enum (e.g., @solana)
    pub attributes: Vec<Attribute>,

    /// Variants in this enum
    pub variants: Vec<EnumVariant>,

    /// Optional semantic version (e.g., "1.0.0" from #[version = "1.0.0"])
    pub version: Option<String>,

    /// Span information for error reporting
    #[serde(skip)]
    pub span: Option<proc_macro2::Span>,
}

/// An enum variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnumVariant {
    /// Unit variant (e.g., `Active`)
    Unit {
        name: String,
        #[serde(skip)]
        span: Option<proc_macro2::Span>,
    },

    /// Tuple variant (e.g., `PlayerJoined(PublicKey)`)
    Tuple {
        name: String,
        types: Vec<TypeSpec>,
        #[serde(skip)]
        span: Option<proc_macro2::Span>,
    },

    /// Struct variant (e.g., `Initialize { authority: PublicKey }`)
    Struct {
        name: String,
        fields: Vec<FieldDef>,
        #[serde(skip)]
        span: Option<proc_macro2::Span>,
    },
}

/// A field definition within a struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    /// Field name (e.g., "wallet")
    pub name: String,

    /// Field type
    pub type_spec: TypeSpec,

    /// Whether this field is optional (has `?` suffix)
    pub optional: bool,

    /// Attributes applied to this field (e.g., @key, @max(32))
    pub attributes: Vec<Attribute>,

    /// Span information for error reporting
    #[serde(skip)]
    pub span: Option<proc_macro2::Span>,
}

/// Type specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeSpec {
    /// Primitive type (e.g., u64, string, bool)
    Primitive(String),

    /// Dynamic array type (e.g., `Vec<PublicKey>` in Rust, `[T]` in LUMOS)
    Array(Box<TypeSpec>),

    /// Fixed-size array type (e.g., `[u8; 32]` in Rust and LUMOS)
    FixedArray { element: Box<TypeSpec>, size: usize },

    /// User-defined type (e.g., Address, CustomStruct)
    UserDefined(String),
}

/// Attribute (e.g., @solana, @account, @key, @max(100))
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    /// Attribute name (e.g., "solana", "account", "key", "max")
    pub name: String,

    /// Optional attribute value (e.g., Some("100") for @max(100))
    pub value: Option<AttributeValue>,

    /// Span information for error reporting
    #[serde(skip)]
    pub span: Option<proc_macro2::Span>,
}

/// Attribute value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeValue {
    /// String value
    String(String),

    /// Integer value
    Integer(u64),

    /// Boolean value
    Bool(bool),

    /// List of values (e.g., derive macros: `#[derive(Debug, Clone)]`)
    List(Vec<String>),
}

impl StructDef {
    /// Check if struct has a specific attribute
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().any(|attr| attr.name == name)
    }

    /// Get attribute by name
    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|attr| attr.name == name)
    }
}

impl EnumDef {
    /// Check if enum has a specific attribute
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().any(|attr| attr.name == name)
    }

    /// Get attribute by name
    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|attr| attr.name == name)
    }

    /// Check if this enum has only unit variants
    pub fn is_unit_only(&self) -> bool {
        self.variants
            .iter()
            .all(|v| matches!(v, EnumVariant::Unit { .. }))
    }

    /// Check if this enum has struct variants
    pub fn has_struct_variants(&self) -> bool {
        self.variants
            .iter()
            .any(|v| matches!(v, EnumVariant::Struct { .. }))
    }
}

impl EnumVariant {
    /// Get the variant name
    pub fn name(&self) -> &str {
        match self {
            EnumVariant::Unit { name, .. } => name,
            EnumVariant::Tuple { name, .. } => name,
            EnumVariant::Struct { name, .. } => name,
        }
    }
}

impl FieldDef {
    /// Check if field has a specific attribute
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.iter().any(|attr| attr.name == name)
    }

    /// Get attribute by name
    pub fn get_attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|attr| attr.name == name)
    }

    /// Get the max length constraint if present
    pub fn max_length(&self) -> Option<u64> {
        self.get_attribute("max").and_then(|attr| {
            if let Some(AttributeValue::Integer(n)) = &attr.value {
                Some(*n)
            } else {
                None
            }
        })
    }
}

impl TypeSpec {
    /// Check if this is an array type (dynamic or fixed)
    pub fn is_array(&self) -> bool {
        matches!(self, TypeSpec::Array(_) | TypeSpec::FixedArray { .. })
    }

    /// Get the inner type if this is an array
    pub fn array_inner(&self) -> Option<&TypeSpec> {
        match self {
            TypeSpec::Array(inner) => Some(inner),
            TypeSpec::FixedArray { element, .. } => Some(element),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn as_string(&self) -> String {
        match self {
            TypeSpec::Primitive(name) => name.clone(),
            TypeSpec::Array(inner) => format!("[{}]", inner.as_string()),
            TypeSpec::FixedArray { element, size } => {
                format!("[{}; {}]", element.as_string(), size)
            }
            TypeSpec::UserDefined(name) => name.clone(),
        }
    }
}

impl std::fmt::Display for TypeSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl TypeAlias {
    /// Get the alias name
    pub fn alias_name(&self) -> &str {
        &self.name
    }

    /// Get the target type
    pub fn target_type(&self) -> &TypeSpec {
        &self.target
    }
}

impl Import {
    /// Get the imported items
    pub fn imported_items(&self) -> &[String] {
        &self.items
    }

    /// Get the import path
    pub fn import_path(&self) -> &str {
        &self.path
    }
}

impl ModulePath {
    /// Create a new module path from segments
    pub fn new(segments: Vec<PathSegment>) -> Self {
        Self { segments }
    }

    /// Create a simple identifier path (e.g., "models" -> [Ident("models")])
    pub fn from_ident(name: String) -> Self {
        Self {
            segments: vec![PathSegment::Ident(name)],
        }
    }

    /// Check if this path starts with `crate::`
    pub fn is_absolute(&self) -> bool {
        matches!(self.segments.first(), Some(PathSegment::Crate))
    }

    /// Check if this path starts with `super::`
    pub fn starts_with_super(&self) -> bool {
        matches!(self.segments.first(), Some(PathSegment::Super))
    }

    /// Check if this path starts with `self::`
    pub fn starts_with_self(&self) -> bool {
        matches!(self.segments.first(), Some(PathSegment::SelfPath))
    }

    /// Get the final identifier in the path (e.g., "User" from "crate::models::User")
    pub fn final_ident(&self) -> Option<&str> {
        self.segments.iter().rev().find_map(|seg| {
            if let PathSegment::Ident(name) = seg {
                Some(name.as_str())
            } else {
                None
            }
        })
    }

    /// Convert to string representation (e.g., "crate::models::User")
    pub fn to_string(&self) -> String {
        self.segments
            .iter()
            .map(|seg| match seg {
                PathSegment::Crate => "crate".to_string(),
                PathSegment::Super => "super".to_string(),
                PathSegment::SelfPath => "self".to_string(),
                PathSegment::Ident(name) => name.clone(),
            })
            .collect::<Vec<_>>()
            .join("::")
    }
}

impl PathSegment {
    /// Check if this segment is an identifier
    pub fn is_ident(&self) -> bool {
        matches!(self, PathSegment::Ident(_))
    }

    /// Get the identifier name if this is an Ident segment
    pub fn as_ident(&self) -> Option<&str> {
        if let PathSegment::Ident(name) = self {
            Some(name)
        } else {
            None
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_has_attribute() {
        let struct_def = StructDef {
            name: "User".to_string(),
            visibility: Visibility::Private,
            attributes: vec![
                Attribute {
                    name: "solana".to_string(),
                    value: None,
                    span: None,
                },
                Attribute {
                    name: "account".to_string(),
                    value: None,
                    span: None,
                },
            ],
            fields: vec![],
            version: None,
            span: None,
        };

        assert!(struct_def.has_attribute("solana"));
        assert!(struct_def.has_attribute("account"));
        assert!(!struct_def.has_attribute("key"));
    }

    #[test]
    fn test_field_max_length() {
        let field = FieldDef {
            name: "username".to_string(),
            type_spec: TypeSpec::Primitive("string".to_string()),
            optional: false,
            attributes: vec![Attribute {
                name: "max".to_string(),
                value: Some(AttributeValue::Integer(32)),
                span: None,
            }],
            span: None,
        };

        assert_eq!(field.max_length(), Some(32));
    }

    #[test]
    fn test_type_spec_to_string() {
        let type_u64 = TypeSpec::Primitive("u64".to_string());
        assert_eq!(type_u64.to_string(), "u64");

        let type_array = TypeSpec::Array(Box::new(TypeSpec::Primitive("PublicKey".to_string())));
        assert_eq!(type_array.to_string(), "[PublicKey]");
    }

    #[test]
    fn test_enum_has_attribute() {
        let enum_def = EnumDef {
            name: "GameState".to_string(),
            visibility: Visibility::Private,
            attributes: vec![Attribute {
                name: "solana".to_string(),
                value: None,
                span: None,
            }],
            variants: vec![],
            version: None,
            span: None,
        };

        assert!(enum_def.has_attribute("solana"));
        assert!(!enum_def.has_attribute("account"));
    }

    #[test]
    fn test_enum_is_unit_only() {
        let unit_enum = EnumDef {
            name: "GameState".to_string(),
            visibility: Visibility::Private,
            attributes: vec![],
            variants: vec![
                EnumVariant::Unit {
                    name: "Active".to_string(),
                    span: None,
                },
                EnumVariant::Unit {
                    name: "Inactive".to_string(),
                    span: None,
                },
            ],
            version: None,
            span: None,
        };

        assert!(unit_enum.is_unit_only());

        let mixed_enum = EnumDef {
            name: "GameEvent".to_string(),
            visibility: Visibility::Private,
            attributes: vec![],
            variants: vec![
                EnumVariant::Unit {
                    name: "Start".to_string(),
                    span: None,
                },
                EnumVariant::Tuple {
                    name: "PlayerJoined".to_string(),
                    types: vec![TypeSpec::Primitive("PublicKey".to_string())],
                    span: None,
                },
            ],
            version: None,
            span: None,
        };

        assert!(!mixed_enum.is_unit_only());
    }

    #[test]
    fn test_enum_variant_name() {
        let unit = EnumVariant::Unit {
            name: "Active".to_string(),
            span: None,
        };
        assert_eq!(unit.name(), "Active");

        let tuple = EnumVariant::Tuple {
            name: "PlayerJoined".to_string(),
            types: vec![],
            span: None,
        };
        assert_eq!(tuple.name(), "PlayerJoined");

        let struct_variant = EnumVariant::Struct {
            name: "Initialize".to_string(),
            fields: vec![],
            span: None,
        };
        assert_eq!(struct_variant.name(), "Initialize");
    }

    #[test]
    fn test_item_enum() {
        let enum_def = EnumDef {
            name: "Status".to_string(),
            visibility: Visibility::Private,
            attributes: vec![],
            variants: vec![],
            span: None,
            version: None,
        };

        let item = Item::Enum(enum_def.clone());
        match item {
            Item::Enum(e) => assert_eq!(e.name, "Status"),
            _ => panic!("Expected enum item"),
        }
    }
}
