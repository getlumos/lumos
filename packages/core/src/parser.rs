// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! LUMOS Parser
//!
//! Parses `.lumos` files using `syn` and builds an Abstract Syntax Tree (AST).
//!
//! ## Overview
//!
//! The parser leverages the `syn` crate to parse Rust-style syntax and extract
//! struct and enum definitions with their attributes. It handles:
//!
//! - Struct definitions with `#[account]`, `#[solana]` attributes
//! - Enum definitions (unit, tuple, and struct variants)
//! - Field types (primitives, arrays, options, user-defined)
//! - Attribute parsing (`#[max(n)]`, `#[key]`, etc.)
//!
//! ## Example
//!
//! ```rust
//! use lumos_core::parser::parse_lumos_file;
//!
//! let source = r#"
//!     #[solana]
//!     struct Account {
//!         owner: PublicKey,
//!         balance: u64,
//!     }
//! "#;
//!
//! let ast = parse_lumos_file(source)?;
//! assert_eq!(ast.items.len(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::ast::{
    Attribute, AttributeValue, EnumDef, EnumVariant, FieldDef, Item as AstItem, LumosFile,
    StructDef, TypeSpec,
};
use crate::error::{LumosError, Result};
use syn::{Item, Meta, Type};

/// Parse a `.lumos` file into an Abstract Syntax Tree.
///
/// This is the main entry point for parsing LUMOS schemas. It accepts source code
/// as a string and returns a [`LumosFile`] containing all parsed type definitions.
///
/// # Arguments
///
/// * `input` - Source code of a `.lumos` file (Rust-style syntax)
///
/// # Returns
///
/// * `Ok(LumosFile)` - Successfully parsed AST with all structs and enums
/// * `Err(LumosError)` - Syntax error or no type definitions found
///
/// # Supported Syntax
///
/// - **Structs**: `struct Name { field: Type, ... }`
/// - **Enums**: `enum Name { Variant, Variant(Type), Variant { field: Type } }`
/// - **Attributes**: `#[solana]`, `#[account]`, `#[max(n)]`, `#[key]`
/// - **Types**: Primitives (`u64`, `String`), Solana types (`PublicKey`), arrays `[T]`, `Option<T>`
///
/// # Example
///
/// ```rust
/// use lumos_core::parser::parse_lumos_file;
///
/// let source = r#"
///     #[solana]
///     #[account]
///     struct UserAccount {
///         wallet: PublicKey,
///         balance: u64,
///         items: [PublicKey],
///     }
///
///     #[solana]
///     enum GameState {
///         Active,
///         Paused,
///         Finished,
///     }
/// "#;
///
/// let ast = parse_lumos_file(source)?;
/// assert_eq!(ast.items.len(), 2); // 1 struct + 1 enum
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns [`LumosError::SchemaParse`] if:
/// - Syntax is invalid (not valid Rust-style code)
/// - No struct or enum definitions found
/// - Unsupported type syntax encountered
pub fn parse_lumos_file(input: &str) -> Result<LumosFile> {
    let mut items = Vec::new();

    // Parse the file as Rust code using syn
    let file = syn::parse_file(input).map_err(|e| {
        LumosError::SchemaParse(format!("Failed to parse .lumos file: {}", e), None)
    })?;

    // Extract struct and enum definitions
    for item in file.items {
        match item {
            Item::Struct(item_struct) => {
                let struct_def = parse_struct(item_struct)?;
                items.push(AstItem::Struct(struct_def));
            }
            Item::Enum(item_enum) => {
                let enum_def = parse_enum(item_enum)?;
                items.push(AstItem::Enum(enum_def));
            }
            _ => {
                // Ignore other items (functions, impls, etc.)
            }
        }
    }

    if items.is_empty() {
        return Err(LumosError::SchemaParse(
            "No type definitions found in .lumos file".to_string(),
            None,
        ));
    }

    Ok(LumosFile { items })
}

/// Parse a struct definition
fn parse_struct(item: syn::ItemStruct) -> Result<StructDef> {
    let name = item.ident.to_string();
    let span = Some(item.ident.span());

    // Extract attributes
    let attributes = parse_attributes(&item.attrs)?;

    // Extract version from attributes
    let version = extract_version_attribute(&attributes)?.map(|v| v.to_string());

    // Extract fields
    let fields = match item.fields {
        syn::Fields::Named(fields_named) => {
            let mut field_defs = Vec::new();
            for field in fields_named.named {
                let field_def = parse_field(field)?;
                field_defs.push(field_def);
            }
            field_defs
        }
        _ => {
            return Err(LumosError::SchemaParse(
                format!("Struct '{}' must have named fields", name),
                None,
            ))
        }
    };

    Ok(StructDef {
        name,
        attributes,
        fields,
        version,
        span,
    })
}

/// Parse an enum definition
fn parse_enum(item: syn::ItemEnum) -> Result<EnumDef> {
    let name = item.ident.to_string();
    let span = Some(item.ident.span());

    // Extract attributes
    let attributes = parse_attributes(&item.attrs)?;

    // Extract version from attributes
    let version = extract_version_attribute(&attributes)?.map(|v| v.to_string());

    // Extract variants
    let mut variants = Vec::new();
    for variant in item.variants {
        let variant_def = parse_enum_variant(variant)?;
        variants.push(variant_def);
    }

    if variants.is_empty() {
        return Err(LumosError::SchemaParse(
            format!("Enum '{}' must have at least one variant", name),
            None,
        ));
    }

    Ok(EnumDef {
        name,
        attributes,
        variants,
        version,
        span,
    })
}

/// Parse an enum variant
fn parse_enum_variant(variant: syn::Variant) -> Result<EnumVariant> {
    let name = variant.ident.to_string();
    let span = Some(variant.ident.span());

    match variant.fields {
        // Unit variant: `Active`
        syn::Fields::Unit => Ok(EnumVariant::Unit { name, span }),

        // Tuple variant: `PlayerJoined(PublicKey, u64)`
        syn::Fields::Unnamed(fields_unnamed) => {
            let mut types = Vec::new();
            for field in fields_unnamed.unnamed {
                let (type_spec, _optional) = parse_type(&field.ty)?;
                types.push(type_spec);
            }
            Ok(EnumVariant::Tuple { name, types, span })
        }

        // Struct variant: `Initialize { authority: PublicKey }`
        syn::Fields::Named(fields_named) => {
            let mut fields = Vec::new();
            for field in fields_named.named {
                let field_def = parse_field(field)?;
                fields.push(field_def);
            }
            Ok(EnumVariant::Struct { name, fields, span })
        }
    }
}

/// Parse a field definition
fn parse_field(field: syn::Field) -> Result<FieldDef> {
    let name = field
        .ident
        .as_ref()
        .ok_or_else(|| LumosError::SchemaParse("Field must have a name".to_string(), None))?
        .to_string();

    let span = field.ident.as_ref().map(|i| i.span());

    // Extract field attributes
    let attributes = parse_attributes(&field.attrs)?;

    // Parse field type
    let (type_spec, optional) = parse_type(&field.ty)?;

    Ok(FieldDef {
        name,
        type_spec,
        optional,
        attributes,
        span,
    })
}

/// Parse attributes (e.g., #[solana], #[account], #[key], #[max(100)])
fn parse_attributes(attrs: &[syn::Attribute]) -> Result<Vec<Attribute>> {
    let mut attributes = Vec::new();

    for attr in attrs {
        // Parse meta (attribute content)
        let meta = &attr.meta;

        match meta {
            // Simple path attribute: #[solana]
            Meta::Path(path) => {
                if let Some(ident) = path.get_ident() {
                    attributes.push(Attribute {
                        name: ident.to_string(),
                        value: None,
                        span: Some(ident.span()),
                    });
                }
            }

            // List attribute: #[max(100)]
            Meta::List(meta_list) => {
                let name = meta_list
                    .path
                    .get_ident()
                    .ok_or_else(|| LumosError::SchemaParse("Invalid attribute".to_string(), None))?
                    .to_string();

                // Parse the value inside parentheses
                let value = parse_attribute_value(&meta_list.tokens.to_string())?;

                attributes.push(Attribute {
                    name,
                    value: Some(value),
                    span: Some(meta_list.path.get_ident().unwrap().span()),
                });
            }

            // Name-value attribute: #[version = "1.0.0"]
            Meta::NameValue(meta_name_value) => {
                let name = meta_name_value
                    .path
                    .get_ident()
                    .ok_or_else(|| LumosError::SchemaParse("Invalid attribute".to_string(), None))?
                    .to_string();

                // Extract the value (e.g., "1.0.0" from #[version = "1.0.0"])
                let value_str = match &meta_name_value.value {
                    syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                        syn::Lit::Str(lit_str) => lit_str.value(),
                        _ => {
                            return Err(LumosError::SchemaParse(
                                format!("Attribute '{}' must have a string value", name),
                                None,
                            ))
                        }
                    },
                    _ => {
                        return Err(LumosError::SchemaParse(
                            format!("Attribute '{}' must have a literal value", name),
                            None,
                        ))
                    }
                };

                attributes.push(Attribute {
                    name,
                    value: Some(AttributeValue::String(value_str)),
                    span: Some(meta_name_value.path.get_ident().unwrap().span()),
                });
            }
        }
    }

    Ok(attributes)
}

/// Parse attribute value from token stream
fn parse_attribute_value(tokens: &str) -> Result<AttributeValue> {
    let tokens_trimmed = tokens.trim();

    // Try parsing as integer
    if let Ok(n) = tokens_trimmed.parse::<u64>() {
        return Ok(AttributeValue::Integer(n));
    }

    // Try parsing as boolean
    if tokens_trimmed == "true" {
        return Ok(AttributeValue::Bool(true));
    }
    if tokens_trimmed == "false" {
        return Ok(AttributeValue::Bool(false));
    }

    // Try parsing as string (remove quotes)
    if tokens_trimmed.starts_with('"') && tokens_trimmed.ends_with('"') {
        let s = tokens_trimmed[1..tokens_trimmed.len() - 1].to_string();
        return Ok(AttributeValue::String(s));
    }

    // Default: treat as string
    Ok(AttributeValue::String(tokens_trimmed.to_string()))
}

/// Parse a type specification
fn parse_type(ty: &Type) -> Result<(TypeSpec, bool)> {
    match ty {
        // Simple type: u64, string, PublicKey
        Type::Path(type_path) => {
            let type_name = type_path
                .path
                .segments
                .last()
                .ok_or_else(|| LumosError::SchemaParse("Invalid type".to_string(), None))?
                .ident
                .to_string();

            // Check if it's an Option<T> (optional type)
            if type_name == "Option" {
                // Extract the inner type from Option<T>
                if let Some(segment) = type_path.path.segments.last() {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            let (inner_type_spec, _) = parse_type(inner_ty)?;
                            return Ok((inner_type_spec, true)); // optional = true
                        }
                    }
                }
            }

            // Regular type
            Ok((TypeSpec::Primitive(type_name), false))
        }

        // Array type: [T]
        Type::Array(type_array) => {
            let (inner_type_spec, _) = parse_type(&type_array.elem)?;
            Ok((TypeSpec::Array(Box::new(inner_type_spec)), false))
        }

        // Slice type: [T] (also treated as array)
        Type::Slice(type_slice) => {
            let (inner_type_spec, _) = parse_type(&type_slice.elem)?;
            Ok((TypeSpec::Array(Box::new(inner_type_spec)), false))
        }

        _ => Err(LumosError::SchemaParse(
            format!("Unsupported type: {:?}", ty),
            None,
        )),
    }
}

/// Extract and validate version attribute from a list of attributes
///
/// Searches for `#[version = "X.Y.Z"]` attribute and validates it as semantic version.
///
/// # Arguments
///
/// * `attributes` - List of parsed attributes
///
/// # Returns
///
/// * `Ok(Some(Version))` - Valid semantic version found
/// * `Ok(None)` - No version attribute present
/// * `Err(LumosError)` - Invalid version format
///
/// # Examples
///
/// ```ignore
/// let attributes = vec![
///     Attribute { name: "solana".to_string(), value: None, span: None },
///     Attribute { name: "version".to_string(), value: Some(AttributeValue::String("1.0.0".to_string())), span: None },
/// ];
/// let version = extract_version_attribute(&attributes)?;
/// assert!(version.is_some());
/// assert_eq!(version.unwrap().to_string(), "1.0.0");
/// ```
pub fn extract_version_attribute(attributes: &[Attribute]) -> Result<Option<semver::Version>> {
    // Find version attribute
    let version_attr = attributes.iter().find(|attr| attr.name == "version");

    if let Some(attr) = version_attr {
        // Extract version string
        let version_str = match &attr.value {
            Some(AttributeValue::String(s)) => s,
            Some(_) => {
                return Err(LumosError::SchemaParse(
                    "Version attribute must be a string (e.g., #[version = \"1.0.0\"])".to_string(),
                    None,
                ))
            }
            None => {
                return Err(LumosError::SchemaParse(
                    "Version attribute must have a value (e.g., #[version = \"1.0.0\"])"
                        .to_string(),
                    None,
                ))
            }
        };

        // Parse and validate semantic version
        match semver::Version::parse(version_str) {
            Ok(version) => Ok(Some(version)),
            Err(e) => Err(LumosError::SchemaParse(
                format!(
                    "Invalid semantic version '{}': {}. Expected format: MAJOR.MINOR.PATCH (e.g., \"1.0.0\")",
                    version_str, e
                ),
                None,
            )),
        }
    } else {
        // No version attribute found
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_struct() {
        let input = r#"
            struct User {
                id: u64,
                name: String,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        assert_eq!(file.items.len(), 1);

        match &file.items[0] {
            AstItem::Struct(struct_def) => {
                assert_eq!(struct_def.name, "User");
                assert_eq!(struct_def.fields.len(), 2);
                assert_eq!(struct_def.fields[0].name, "id");
                assert_eq!(struct_def.fields[1].name, "name");
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_with_attributes() {
        let input = r#"
            #[solana]
            #[account]
            struct UserAccount {
                #[key]
                wallet: PublicKey,

                #[max(32)]
                username: String,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Struct(struct_def) => {
                assert!(struct_def.has_attribute("solana"));
                assert!(struct_def.has_attribute("account"));
                assert_eq!(struct_def.fields[0].name, "wallet");
                assert!(struct_def.fields[0].has_attribute("key"));
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_optional_type() {
        let input = r#"
            struct User {
                email: Option<String>,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Struct(struct_def) => {
                let field = &struct_def.fields[0];
                assert!(field.optional);
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_array_type() {
        let input = r#"
            struct Team {
                members: [u64],
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Struct(struct_def) => {
                let field = &struct_def.fields[0];
                assert!(field.type_spec.is_array());
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_struct_with_version() {
        let input = r#"
            #[solana]
            #[version = "1.0.0"]
            struct PlayerAccount {
                wallet: PublicKey,
                level: u16,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Struct(struct_def) => {
                assert_eq!(struct_def.version, Some("1.0.0".to_string()));
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_enum_with_version() {
        let input = r#"
            #[solana]
            #[version = "2.1.3"]
            enum GameState {
                Active,
                Paused,
                Finished,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Enum(enum_def) => {
                assert_eq!(enum_def.version, Some("2.1.3".to_string()));
            }
            _ => panic!("Expected enum item"),
        }
    }

    #[test]
    fn test_parse_struct_without_version() {
        let input = r#"
            #[solana]
            struct Account {
                owner: PublicKey,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Struct(struct_def) => {
                assert_eq!(struct_def.version, None);
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_invalid_version_format() {
        let input = r#"
            #[solana]
            #[version = "1.0"]
            struct Account {
                owner: PublicKey,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.to_string().contains("Invalid semantic version"));
        }
    }

    #[test]
    fn test_parse_version_with_prerelease() {
        let input = r#"
            #[solana]
            #[version = "1.0.0-beta.1"]
            struct Account {
                owner: PublicKey,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Struct(struct_def) => {
                assert_eq!(struct_def.version, Some("1.0.0-beta.1".to_string()));
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_version_with_build_metadata() {
        let input = r#"
            #[solana]
            #[version = "1.0.0+build.123"]
            struct Account {
                owner: PublicKey,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Struct(struct_def) => {
                assert_eq!(struct_def.version, Some("1.0.0+build.123".to_string()));
            }
            _ => panic!("Expected struct item"),
        }
    }
}
