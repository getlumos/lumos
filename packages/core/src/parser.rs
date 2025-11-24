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
    Attribute, AttributeValue, EnumDef, EnumVariant, FieldDef, Import, Item as AstItem,
    LumosFile, StructDef, TypeAlias, TypeSpec,
};
use crate::error::{LumosError, Result};
use regex::Regex;
use syn::{Item, Meta, Type};

/// Extract JavaScript-style import statements from source code
///
/// Parses `import { Item1, Item2 } from "./path.lumos";` statements and removes
/// them from the source, returning both the parsed imports and the remaining code.
///
/// # Arguments
///
/// * `input` - Source code potentially containing import statements
///
/// # Returns
///
/// * `Ok((imports, remaining_code))` - Parsed imports and code without imports
/// * `Err(LumosError)` - Invalid import syntax
///
/// # Example
///
/// ```ignore
/// let source = r#"
/// import { UserId, Timestamp } from "./types.lumos";
///
/// struct Player {
///     id: UserId,
/// }
/// "#;
/// let (imports, remaining) = extract_imports(source)?;
/// assert_eq!(imports.len(), 1);
/// assert_eq!(imports[0].items, vec!["UserId", "Timestamp"]);
/// ```
fn extract_imports(input: &str) -> Result<(Vec<Import>, String)> {
    let mut imports = Vec::new();

    // Multi-line regex: match import { ... } from "..." across multiple lines
    // Note: Using (?s) flag is not supported in regex crate, so we handle newlines with [\s\S]
    let import_regex = Regex::new(
        r#"import\s*\{([^}]+)\}\s*from\s+["']([^"']+)["']\s*;?"#,
    )
    .map_err(|e| {
        LumosError::SchemaParse(format!("Failed to compile import regex: {}", e), None)
    })?;

    let mut remaining = input.to_string();

    // Find and extract all imports (including multi-line)
    for capture in import_regex.captures_iter(input) {
        // Extract imported items (e.g., "UserId, Timestamp" or "UserId,\n    Timestamp")
        let items_str = capture.get(1).unwrap().as_str();
        let items: Vec<String> = items_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if items.is_empty() {
            return Err(LumosError::SchemaParse(
                "Empty import list".to_string(),
                None,
            ));
        }

        // Extract path (e.g., "./types.lumos")
        let path = capture.get(2).unwrap().as_str().to_string();

        imports.push(Import {
            items,
            path,
            span: None, // No span info for regex-parsed imports
        });
    }

    // Remove all import statements from the input
    remaining = import_regex.replace_all(&remaining, "").to_string();

    Ok((imports, remaining))
}

/// Parse a type alias definition
///
/// Converts `type UserId = PublicKey;` into a TypeAlias AST node.
///
/// # Arguments
///
/// * `item` - syn ItemType node representing a type alias
///
/// # Returns
///
/// * `Ok(TypeAlias)` - Successfully parsed type alias
/// * `Err(LumosError)` - Invalid type syntax
fn parse_type_alias(item: syn::ItemType) -> Result<TypeAlias> {
    let name = item.ident.to_string();
    let span = Some(item.ident.span());

    // Parse the target type
    let (target, _optional) = parse_type(&item.ty)?;

    Ok(TypeAlias {
        name,
        target,
        span,
    })
}

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
/// * `Ok(LumosFile)` - Successfully parsed AST with all structs, enums, and type aliases
/// * `Err(LumosError)` - Syntax error or no type definitions found
///
/// # Supported Syntax
///
/// - **Imports**: `import { Type1, Type2 } from "./path.lumos";`
/// - **Type Aliases**: `type UserId = PublicKey;`
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
///     import { Timestamp } from "./common.lumos";
///
///     type UserId = PublicKey;
///
///     #[solana]
///     #[account]
///     struct UserAccount {
///         id: UserId,
///         created_at: Timestamp,
///         balance: u64,
///     }
/// "#;
///
/// let ast = parse_lumos_file(source)?;
/// assert_eq!(ast.imports.len(), 1);
/// assert_eq!(ast.items.len(), 2); // 1 type alias + 1 struct
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns [`LumosError::SchemaParse`] if:
/// - Syntax is invalid (not valid Rust-style code)
/// - Invalid import syntax
/// - Unsupported type syntax encountered
pub fn parse_lumos_file(input: &str) -> Result<LumosFile> {
    let mut items = Vec::new();

    // Extract imports first (JavaScript-style imports are not valid Rust syntax)
    let (imports, remaining_input) = extract_imports(input)?;

    // Parse the remaining file as Rust code using syn
    let file = syn::parse_file(&remaining_input).map_err(|e| {
        LumosError::SchemaParse(format!("Failed to parse .lumos file: {}", e), None)
    })?;

    // Extract struct, enum, and type alias definitions
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
            Item::Type(item_type) => {
                let type_alias = parse_type_alias(item_type)?;
                items.push(AstItem::TypeAlias(type_alias));
            }
            _ => {
                // Ignore other items (functions, impls, etc.)
            }
        }
    }

    if items.is_empty() && imports.is_empty() {
        return Err(LumosError::SchemaParse(
            "No type definitions or imports found in .lumos file".to_string(),
            None,
        ));
    }

    Ok(LumosFile { imports, items })
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

            // List attribute: #[max(100)] or #[derive(Debug, Clone)]
            Meta::List(meta_list) => {
                let name = meta_list
                    .path
                    .get_ident()
                    .ok_or_else(|| LumosError::SchemaParse("Invalid attribute".to_string(), None))?
                    .to_string();

                // Special handling for #[derive(...)] - contains comma-separated list of macros
                let value = if name == "derive" {
                    parse_derive_list(&meta_list.tokens.to_string())?
                } else {
                    // Parse the value inside parentheses for other list attributes
                    parse_attribute_value(&meta_list.tokens.to_string())?
                };

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

/// Parse derive list from token stream
///
/// Parses comma-separated derive macro names from `#[derive(Debug, Clone, PartialEq)]`.
///
/// # Arguments
///
/// * `tokens` - Token stream containing derive macro names (e.g., "Debug, Clone, PartialEq")
///
/// # Returns
///
/// * `Ok(AttributeValue::List)` - List of derive macro names
/// * `Err(LumosError)` - Empty derive list or invalid syntax
///
/// # Example
///
/// ```ignore
/// // Input: "Debug, Clone, PartialEq"
/// // Output: AttributeValue::List(vec!["Debug", "Clone", "PartialEq"])
/// ```
fn parse_derive_list(tokens: &str) -> Result<AttributeValue> {
    let tokens_trimmed = tokens.trim();

    // Split by commas and trim whitespace
    let derives: Vec<String> = tokens_trimmed
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if derives.is_empty() {
        return Err(LumosError::SchemaParse(
            "derive attribute must have at least one macro".to_string(),
            None,
        ));
    }

    Ok(AttributeValue::List(derives))
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

        // Fixed-size array type: [T; N]
        Type::Array(type_array) => {
            let (inner_type_spec, _) = parse_type(&type_array.elem)?;

            // Extract array size from length expression
            let size = parse_array_size(&type_array.len)?;

            // Validate size constraints
            validate_array_size(size)?;

            Ok((
                TypeSpec::FixedArray {
                    element: Box::new(inner_type_spec),
                    size,
                },
                false,
            ))
        }

        // Slice type: [T] (dynamic array/Vec)
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

/// Parse array size from expression (must be literal integer)
///
/// # Arguments
///
/// * `expr` - Expression representing array size (e.g., `32` in `[u8; 32]`)
///
/// # Returns
///
/// * `Ok(usize)` - Successfully parsed array size
/// * `Err(LumosError)` - Non-literal or invalid size expression
///
/// # Examples
///
/// ```ignore
/// // Valid: [u8; 32]
/// let size = parse_array_size(&lit_expr)?;
/// assert_eq!(size, 32);
/// ```
fn parse_array_size(expr: &syn::Expr) -> Result<usize> {
    match expr {
        syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
            syn::Lit::Int(lit_int) => {
                lit_int.base10_parse::<usize>().map_err(|e| {
                    LumosError::SchemaParse(
                        format!("Invalid array size (must be valid usize): {}", e),
                        None,
                    )
                })
            }
            _ => Err(LumosError::SchemaParse(
                "Array size must be an integer literal".to_string(),
                None,
            )),
        },
        _ => Err(LumosError::SchemaParse(
            "Array size must be a literal integer (const generics not yet supported)".to_string(),
            None,
        )),
    }
}

/// Validate array size constraints
///
/// Ensures array size is within reasonable bounds for Solana programs.
///
/// # Constraints
///
/// * Size must be > 0 (zero-sized arrays are invalid)
/// * Size must be ≤ 1024 (practical limit for most use cases)
///
/// # Arguments
///
/// * `size` - Array size to validate
///
/// # Returns
///
/// * `Ok(())` - Size is valid
/// * `Err(LumosError)` - Size is out of bounds
fn validate_array_size(size: usize) -> Result<()> {
    if size == 0 {
        return Err(LumosError::SchemaParse(
            "Array size must be greater than 0".to_string(),
            None,
        ));
    }

    if size > 1024 {
        return Err(LumosError::SchemaParse(
            format!(
                "Array size {} exceeds maximum of 1024 elements (consider using Vec for dynamic arrays)",
                size
            ),
            None,
        ));
    }

    Ok(())
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

    #[test]
    fn test_parse_single_derive() {
        let input = r#"
            #[derive(Debug)]
            struct Account {
                balance: u64,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Struct(struct_def) => {
                let derive_attr = struct_def.get_attribute("derive");
                assert!(derive_attr.is_some());

                if let Some(AttributeValue::List(derives)) = &derive_attr.unwrap().value {
                    assert_eq!(derives.len(), 1);
                    assert_eq!(derives[0], "Debug");
                } else {
                    panic!("Expected List attribute value");
                }
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_multiple_derives() {
        let input = r#"
            #[derive(Debug, Clone, PartialEq)]
            struct Account {
                balance: u64,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Struct(struct_def) => {
                let derive_attr = struct_def.get_attribute("derive");
                assert!(derive_attr.is_some());

                if let Some(AttributeValue::List(derives)) = &derive_attr.unwrap().value {
                    assert_eq!(derives.len(), 3);
                    assert_eq!(derives[0], "Debug");
                    assert_eq!(derives[1], "Clone");
                    assert_eq!(derives[2], "PartialEq");
                } else {
                    panic!("Expected List attribute value");
                }
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_derive_with_whitespace() {
        let input = r#"
            #[derive(  Debug  ,  Clone  ,  PartialEq  )]
            struct Account {
                balance: u64,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Struct(struct_def) => {
                let derive_attr = struct_def.get_attribute("derive");
                assert!(derive_attr.is_some());

                if let Some(AttributeValue::List(derives)) = &derive_attr.unwrap().value {
                    assert_eq!(derives.len(), 3);
                    // Whitespace should be trimmed
                    assert_eq!(derives[0], "Debug");
                    assert_eq!(derives[1], "Clone");
                    assert_eq!(derives[2], "PartialEq");
                } else {
                    panic!("Expected List attribute value");
                }
            }
            _ => panic!("Expected struct item"),
        }
    }

    #[test]
    fn test_parse_derive_on_enum() {
        let input = r#"
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            enum GameState {
                Active,
                Paused,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Enum(enum_def) => {
                let derive_attr = enum_def.get_attribute("derive");
                assert!(derive_attr.is_some());

                if let Some(AttributeValue::List(derives)) = &derive_attr.unwrap().value {
                    assert_eq!(derives.len(), 5);
                    assert_eq!(derives[0], "Debug");
                    assert_eq!(derives[1], "Clone");
                    assert_eq!(derives[2], "PartialEq");
                    assert_eq!(derives[3], "Eq");
                    assert_eq!(derives[4], "Hash");
                } else {
                    panic!("Expected List attribute value");
                }
            }
            _ => panic!("Expected enum item"),
        }
    }

    #[test]
    fn test_parse_derive_with_solana_attributes() {
        let input = r#"
            #[solana]
            #[account]
            #[derive(PartialEq, Eq)]
            struct UserAccount {
                wallet: PublicKey,
                balance: u64,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok());

        let file = result.unwrap();
        match &file.items[0] {
            AstItem::Struct(struct_def) => {
                assert!(struct_def.has_attribute("solana"));
                assert!(struct_def.has_attribute("account"));
                assert!(struct_def.has_attribute("derive"));

                let derive_attr = struct_def.get_attribute("derive");
                if let Some(AttributeValue::List(derives)) = &derive_attr.unwrap().value {
                    assert_eq!(derives.len(), 2);
                    assert_eq!(derives[0], "PartialEq");
                    assert_eq!(derives[1], "Eq");
                } else {
                    panic!("Expected List attribute value");
                }
            }
            _ => panic!("Expected struct item"),
        }
    }
}
