// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Transform AST into IR
//!
//! This module transforms the Abstract Syntax Tree (AST) produced by the parser
//! into the Intermediate Representation (IR) used by code generators.

use crate::ast::{
    FieldDef as AstField, Item as AstItem, LumosFile, StructDef as AstStruct, TypeSpec as AstType,
};
use crate::error::Result;
use crate::ir::{FieldDefinition, Metadata, TypeDefinition, TypeInfo};

/// Transform a parsed LUMOS file (AST) into IR
pub fn transform_to_ir(file: LumosFile) -> Result<Vec<TypeDefinition>> {
    let mut type_defs = Vec::new();

    for item in file.items {
        match item {
            AstItem::Struct(struct_def) => {
                let type_def = transform_struct(struct_def)?;
                type_defs.push(type_def);
            }
            AstItem::Enum(_enum_def) => {
                // TODO: Implement enum transformation in next step
                // For now, skip enums to keep existing tests passing
            }
        }
    }

    Ok(type_defs)
}

/// Transform a single struct definition
fn transform_struct(struct_def: AstStruct) -> Result<TypeDefinition> {
    // Extract metadata from attributes BEFORE consuming struct
    let metadata = extract_metadata(&struct_def);

    let name = struct_def.name;

    // Transform fields
    let fields = struct_def
        .fields
        .into_iter()
        .map(transform_field)
        .collect::<Result<Vec<_>>>()?;

    Ok(TypeDefinition {
        name,
        fields,
        metadata,
    })
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
fn extract_metadata(struct_def: &AstStruct) -> Metadata {
    let mut metadata = Metadata::default();

    // Check for @solana attribute
    metadata.solana = struct_def.has_attribute("solana");

    // Collect all other attributes
    metadata.attributes = struct_def
        .attributes
        .iter()
        .map(|attr| attr.name.clone())
        .collect();

    metadata
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
        assert_eq!(ir[0].name, "User");
        assert_eq!(ir[0].fields.len(), 2);
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

        let fields = &ir[0].fields;
        assert!(matches!(fields[0].type_info, TypeInfo::Primitive(ref s) if s == "u64"));
        assert!(matches!(fields[1].type_info, TypeInfo::Primitive(ref s) if s == "String"));
        assert!(matches!(fields[2].type_info, TypeInfo::Primitive(ref s) if s == "bool"));
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

        let field = &ir[0].fields[0];
        assert!(field.optional);
        assert!(matches!(field.type_info, TypeInfo::Option(_)));
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

        let field = &ir[0].fields[0];
        assert!(matches!(field.type_info, TypeInfo::Array(_)));
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

        assert!(ir[0].metadata.solana);
        assert!(ir[0].metadata.attributes.contains(&"account".to_string()));
    }
}
