// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! AST to IR Transformation
//!
//! This module transforms the Abstract Syntax Tree (AST) produced by the parser
//! into the Intermediate Representation (IR) used by code generators.
//!
//! ## Overview
//!
//! The transformation layer serves as a bridge between parsing and code generation,
//! converting language-specific AST nodes into language-agnostic IR. This separation
//! enables:
//!
//! - **Language independence** - IR can target multiple output languages
//! - **Type normalization** - TypeScript aliases (`number`, `string`) map to Rust types
//! - **Metadata extraction** - Attributes like `#[solana]`, `#[account]` are preserved
//! - **Validation** - Type information is validated during transformation
//!
//! ## Transformation Pipeline
//!
//! ```text
//! AST (syn-based) → Transform → IR (language-agnostic)
//!     ├─ StructDef  → StructDefinition
//!     ├─ EnumDef    → EnumDefinition
//!     ├─ FieldDef   → FieldDefinition
//!     └─ TypeSpec   → TypeInfo
//! ```
//!
//! ## Type Mapping
//!
//! The transform layer normalizes type aliases:
//!
//! - `number` → `u64`
//! - `string` → `String`
//! - `boolean` → `bool`
//! - Solana types (`PublicKey`, `Signature`) are preserved for generator mapping
//!
//! ## Example
//!
//! ```rust
//! use lumos_core::{parser, transform};
//!
//! let source = r#"
//!     #[solana]
//!     #[account]
//!     struct UserAccount {
//!         wallet: PublicKey,
//!         balance: u64,
//!     }
//!
//!     #[solana]
//!     enum GameState {
//!         Active,
//!         Paused,
//!     }
//! "#;
//!
//! let ast = parser::parse_lumos_file(source)?;
//! let ir = transform::transform_to_ir(ast)?;
//!
//! assert_eq!(ir.len(), 2); // 1 struct + 1 enum
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::ast::{
    EnumDef as AstEnum, EnumVariant as AstEnumVariant, FieldDef as AstField, Item as AstItem,
    LumosFile, StructDef as AstStruct, TypeSpec as AstType,
};
use crate::error::Result;
use crate::ir::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, Metadata, StructDefinition,
    TypeDefinition, TypeInfo,
};

/// Transform a parsed LUMOS file (AST) into Intermediate Representation (IR).
///
/// This is the main entry point for AST → IR transformation. It processes all
/// type definitions (structs and enums) in the parsed file and converts them to
/// language-agnostic IR suitable for code generation.
///
/// # Arguments
///
/// * `file` - Parsed LUMOS file containing AST items (structs and enums)
///
/// # Returns
///
/// * `Ok(Vec<TypeDefinition>)` - Successfully transformed IR type definitions
/// * `Err(LumosError)` - Transformation error (e.g., invalid type)
///
/// # Type Normalization
///
/// The transformation performs type alias normalization:
/// - TypeScript-friendly aliases (`number`, `string`, `boolean`) are mapped to Rust types
/// - Solana types (`PublicKey`, `Signature`) are preserved for generator-specific mapping
/// - Optional types (`Option<T>`) are detected and wrapped in `TypeInfo::Option`
///
/// # Example
///
/// ```rust
/// use lumos_core::{parser, transform};
///
/// let source = r#"
///     #[solana]
///     struct Account {
///         owner: PublicKey,
///         balance: number,  // TypeScript alias
///         active: boolean,  // TypeScript alias
///     }
/// "#;
///
/// let ast = parser::parse_lumos_file(source)?;
/// let ir = transform::transform_to_ir(ast)?;
///
/// // Type aliases are normalized to Rust types in IR
/// // number → u64, boolean → bool
/// assert_eq!(ir.len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns [`crate::error::LumosError`] if transformation fails (rare, most validation happens in parser).
pub fn transform_to_ir(file: LumosFile) -> Result<Vec<TypeDefinition>> {
    let mut type_defs = Vec::new();

    for item in file.items {
        match item {
            AstItem::Struct(struct_def) => {
                let type_def = transform_struct(struct_def)?;
                type_defs.push(TypeDefinition::Struct(type_def));
            }
            AstItem::Enum(enum_def) => {
                let type_def = transform_enum(enum_def)?;
                type_defs.push(TypeDefinition::Enum(type_def));
            }
        }
    }

    Ok(type_defs)
}

/// Transform a single struct definition
fn transform_struct(struct_def: AstStruct) -> Result<StructDefinition> {
    // Extract metadata from attributes BEFORE consuming struct
    let metadata = extract_struct_metadata(&struct_def);

    let name = struct_def.name;

    // Transform fields
    let fields = struct_def
        .fields
        .into_iter()
        .map(transform_field)
        .collect::<Result<Vec<_>>>()?;

    Ok(StructDefinition {
        name,
        fields,
        metadata,
    })
}

/// Transform a single enum definition
fn transform_enum(enum_def: AstEnum) -> Result<EnumDefinition> {
    // Extract metadata from attributes BEFORE consuming enum
    let metadata = extract_enum_metadata(&enum_def);

    let name = enum_def.name;

    // Transform variants
    let variants = enum_def
        .variants
        .into_iter()
        .map(transform_enum_variant)
        .collect::<Result<Vec<_>>>()?;

    Ok(EnumDefinition {
        name,
        variants,
        metadata,
    })
}

/// Transform an enum variant
fn transform_enum_variant(variant: AstEnumVariant) -> Result<EnumVariantDefinition> {
    match variant {
        AstEnumVariant::Unit { name, .. } => Ok(EnumVariantDefinition::Unit { name }),

        AstEnumVariant::Tuple { name, types, .. } => {
            let transformed_types = types
                .into_iter()
                .map(|t| transform_type(t, false))
                .collect::<Result<Vec<_>>>()?;

            Ok(EnumVariantDefinition::Tuple {
                name,
                types: transformed_types,
            })
        }

        AstEnumVariant::Struct { name, fields, .. } => {
            let transformed_fields = fields
                .into_iter()
                .map(transform_field)
                .collect::<Result<Vec<_>>>()?;

            Ok(EnumVariantDefinition::Struct {
                name,
                fields: transformed_fields,
            })
        }
    }
}

/// Transform a field definition
fn transform_field(field: AstField) -> Result<FieldDefinition> {
    let name = field.name;
    let optional = field.optional;

    // Transform type
    let type_info = transform_type(field.type_spec, optional)?;

    Ok(FieldDefinition {
        name,
        type_info,
        optional,
    })
}

/// Transform type specification
fn transform_type(type_spec: AstType, optional: bool) -> Result<TypeInfo> {
    let base_type = match type_spec {
        AstType::Primitive(name) => {
            // Map TypeScript-friendly aliases to Rust types
            let rust_type = map_type_alias(&name);
            TypeInfo::Primitive(rust_type)
        }

        AstType::Array(inner) => {
            let inner_type = transform_type(*inner, false)?;
            TypeInfo::Array(Box::new(inner_type))
        }

        AstType::UserDefined(name) => TypeInfo::UserDefined(name),
    };

    // Wrap in Option if optional
    if optional {
        Ok(TypeInfo::Option(Box::new(base_type)))
    } else {
        Ok(base_type)
    }
}

/// Map TypeScript-friendly type aliases to Rust types
fn map_type_alias(name: &str) -> String {
    match name {
        // TypeScript aliases
        "number" => "u64".to_string(),
        "string" => "String".to_string(),
        "boolean" => "bool".to_string(),

        // Solana types (keep as-is, will be mapped in generators)
        "PublicKey" | "Signature" | "Keypair" => name.to_string(),

        // Rust types (keep as-is)
        _ => name.to_string(),
    }
}

/// Extract metadata from struct attributes
fn extract_struct_metadata(struct_def: &AstStruct) -> Metadata {
    Metadata {
        solana: struct_def.has_attribute("solana"),
        attributes: struct_def
            .attributes
            .iter()
            .map(|attr| attr.name.clone())
            .collect(),
    }
}

/// Extract metadata from enum attributes
fn extract_enum_metadata(enum_def: &AstEnum) -> Metadata {
    Metadata {
        solana: enum_def.has_attribute("solana"),
        attributes: enum_def
            .attributes
            .iter()
            .map(|attr| attr.name.clone())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_lumos_file;

    #[test]
    fn test_transform_simple_struct() {
        let input = r#"
            struct User {
                id: u64,
                name: String,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        assert_eq!(ir.len(), 1);
        match &ir[0] {
            TypeDefinition::Struct(s) => {
                assert_eq!(s.name, "User");
                assert_eq!(s.fields.len(), 2);
            }
            _ => panic!("Expected struct type definition"),
        }
    }

    #[test]
    fn test_transform_with_type_aliases() {
        let input = r#"
            struct Product {
                price: number,
                name: string,
                available: boolean,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        match &ir[0] {
            TypeDefinition::Struct(s) => {
                let fields = &s.fields;
                assert!(matches!(fields[0].type_info, TypeInfo::Primitive(ref t) if t == "u64"));
                assert!(matches!(fields[1].type_info, TypeInfo::Primitive(ref t) if t == "String"));
                assert!(matches!(fields[2].type_info, TypeInfo::Primitive(ref t) if t == "bool"));
            }
            _ => panic!("Expected struct type definition"),
        }
    }

    #[test]
    fn test_transform_optional_field() {
        let input = r#"
            struct User {
                email: Option<String>,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        match &ir[0] {
            TypeDefinition::Struct(s) => {
                let field = &s.fields[0];
                assert!(field.optional);
                assert!(matches!(field.type_info, TypeInfo::Option(_)));
            }
            _ => panic!("Expected struct type definition"),
        }
    }

    #[test]
    fn test_transform_array_type() {
        let input = r#"
            struct Team {
                members: [u64],
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        match &ir[0] {
            TypeDefinition::Struct(s) => {
                let field = &s.fields[0];
                assert!(matches!(field.type_info, TypeInfo::Array(_)));
            }
            _ => panic!("Expected struct type definition"),
        }
    }

    #[test]
    fn test_transform_solana_metadata() {
        let input = r#"
            #[solana]
            #[account]
            struct UserAccount {
                wallet: PublicKey,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        match &ir[0] {
            TypeDefinition::Struct(s) => {
                assert!(s.metadata.solana);
                assert!(s.metadata.attributes.contains(&"account".to_string()));
            }
            _ => panic!("Expected struct type definition"),
        }
    }

    #[test]
    fn test_transform_unit_enum() {
        let input = r#"
            #[solana]
            enum GameState {
                Inactive,
                Active,
                Paused,
                Finished,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        assert_eq!(ir.len(), 1);
        match &ir[0] {
            TypeDefinition::Enum(e) => {
                assert_eq!(e.name, "GameState");
                assert_eq!(e.variants.len(), 4);
                assert!(e.metadata.solana);
                assert!(e.is_unit_only());

                // Check variant names
                assert_eq!(e.variants[0].name(), "Inactive");
                assert_eq!(e.variants[1].name(), "Active");
                assert_eq!(e.variants[2].name(), "Paused");
                assert_eq!(e.variants[3].name(), "Finished");
            }
            _ => panic!("Expected enum type definition"),
        }
    }

    #[test]
    fn test_transform_tuple_enum() {
        let input = r#"
            enum GameEvent {
                PlayerJoined(PublicKey),
                ScoreUpdated(PublicKey, u64),
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        assert_eq!(ir.len(), 1);
        match &ir[0] {
            TypeDefinition::Enum(e) => {
                assert_eq!(e.name, "GameEvent");
                assert_eq!(e.variants.len(), 2);
                assert!(e.has_tuple_variants());

                // Check tuple variant types
                match &e.variants[0] {
                    EnumVariantDefinition::Tuple { name, types } => {
                        assert_eq!(name, "PlayerJoined");
                        assert_eq!(types.len(), 1);
                    }
                    _ => panic!("Expected tuple variant"),
                }

                match &e.variants[1] {
                    EnumVariantDefinition::Tuple { name, types } => {
                        assert_eq!(name, "ScoreUpdated");
                        assert_eq!(types.len(), 2);
                    }
                    _ => panic!("Expected tuple variant"),
                }
            }
            _ => panic!("Expected enum type definition"),
        }
    }

    #[test]
    fn test_transform_struct_enum() {
        let input = r#"
            enum GameInstruction {
                Initialize {
                    authority: PublicKey,
                    max_players: u8,
                },
                Terminate,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        assert_eq!(ir.len(), 1);
        match &ir[0] {
            TypeDefinition::Enum(e) => {
                assert_eq!(e.name, "GameInstruction");
                assert_eq!(e.variants.len(), 2);
                assert!(e.has_struct_variants());

                // Check struct variant fields
                match &e.variants[0] {
                    EnumVariantDefinition::Struct { name, fields } => {
                        assert_eq!(name, "Initialize");
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].name, "authority");
                        assert_eq!(fields[1].name, "max_players");
                    }
                    _ => panic!("Expected struct variant"),
                }

                // Check unit variant
                match &e.variants[1] {
                    EnumVariantDefinition::Unit { name } => {
                        assert_eq!(name, "Terminate");
                    }
                    _ => panic!("Expected unit variant"),
                }
            }
            _ => panic!("Expected enum type definition"),
        }
    }
}
