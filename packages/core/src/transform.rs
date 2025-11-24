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
    Attribute, AttributeValue, EnumDef as AstEnum, EnumVariant as AstEnumVariant,
    FieldDef as AstField, Item as AstItem, LumosFile, StructDef as AstStruct,
    TypeAlias as AstTypeAlias, TypeSpec as AstType,
};
use crate::error::{LumosError, Result};
use crate::ir::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, Metadata, StructDefinition,
    TypeAliasDefinition, TypeDefinition, TypeInfo,
};
use std::collections::{HashMap, HashSet};

/// Type alias resolver
///
/// Resolves type aliases recursively and detects circular references.
/// Maintains a map of alias names to their resolved target types.
#[derive(Debug, Clone)]
pub struct TypeAliasResolver {
    /// Map of alias names to their AST type specs (unresolved)
    aliases: HashMap<String, AstType>,

    /// Map of alias names to their resolved TypeInfo (after resolution)
    resolved: HashMap<String, TypeInfo>,
}

impl TypeAliasResolver {
    /// Create a new empty resolver
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
            resolved: HashMap::new(),
        }
    }

    /// Add a type alias to the resolver
    pub fn add_alias(&mut self, name: String, target: AstType) -> Result<()> {
        if self.aliases.contains_key(&name) {
            return Err(LumosError::Transform(
                format!("Duplicate type alias definition: {}", name),
                None,
            ));
        }
        self.aliases.insert(name, target);
        Ok(())
    }

    /// Resolve all aliases and detect circular references
    pub fn resolve_all_aliases(&mut self) -> Result<()> {
        for alias_name in self.aliases.keys().cloned().collect::<Vec<_>>() {
            let mut visited = HashSet::new();
            self.resolve_alias(&alias_name, &mut visited)?;
        }
        Ok(())
    }

    /// Resolve a single alias recursively
    fn resolve_alias(&mut self, name: &str, visited: &mut HashSet<String>) -> Result<TypeInfo> {
        // Check if already resolved
        if let Some(resolved) = self.resolved.get(name) {
            return Ok(resolved.clone());
        }

        // Check for circular reference
        if visited.contains(name) {
            return Err(LumosError::Transform(
                format!("Circular type alias detected: {}", name),
                None,
            ));
        }

        visited.insert(name.to_string());

        // Get the target type
        let target = self
            .aliases
            .get(name)
            .ok_or_else(|| {
                LumosError::Transform(format!("Unknown type alias: {}", name), None)
            })?
            .clone();

        // Transform the target, resolving any nested aliases
        let resolved = self.transform_type_with_resolver(target, false, visited)?;

        // Cache the resolved type
        self.resolved.insert(name.to_string(), resolved.clone());

        visited.remove(name);

        Ok(resolved)
    }

    /// Transform a type spec, resolving any aliases it contains
    fn transform_type_with_resolver(
        &mut self,
        type_spec: AstType,
        optional: bool,
        visited: &mut HashSet<String>,
    ) -> Result<TypeInfo> {
        let base_type = match type_spec {
            AstType::Primitive(name) => {
                // Check if it's a known primitive type
                if is_valid_primitive_type(&name) {
                    // Map TypeScript-friendly aliases to Rust types
                    let rust_type = map_type_alias(&name);
                    TypeInfo::Primitive(rust_type)
                } else if self.aliases.contains_key(&name) {
                    // It's a type alias, resolve it
                    self.resolve_alias(&name, visited)?
                } else {
                    // Treat as user-defined type (enum or struct)
                    TypeInfo::UserDefined(name)
                }
            }

            AstType::Array(inner) => {
                let inner_type = self.transform_type_with_resolver(*inner, false, visited)?;
                TypeInfo::Array(Box::new(inner_type))
            }

            AstType::FixedArray { element, size } => {
                let element_type = self.transform_type_with_resolver(*element, false, visited)?;
                TypeInfo::FixedArray {
                    element: Box::new(element_type),
                    size,
                }
            }

            AstType::UserDefined(name) => {
                // Check if it's an alias
                if self.aliases.contains_key(&name) {
                    self.resolve_alias(&name, visited)?
                } else {
                    // User-defined type (struct/enum)
                    TypeInfo::UserDefined(name)
                }
            }
        };

        // Wrap in Option if optional
        if optional {
            Ok(TypeInfo::Option(Box::new(base_type)))
        } else {
            Ok(base_type)
        }
    }

    /// Get resolved type for an alias (after resolve_all_aliases)
    fn get_resolved(&self, name: &str) -> Option<&TypeInfo> {
        self.resolved.get(name)
    }
}

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

    // First pass: Build type alias resolver
    let mut alias_resolver = TypeAliasResolver::new();

    for item in &file.items {
        if let AstItem::TypeAlias(alias_def) = item {
            alias_resolver.add_alias(alias_def.name.clone(), alias_def.target.clone())?;
        }
    }

    // Resolve all aliases recursively and check for cycles
    alias_resolver.resolve_all_aliases()?;

    // Second pass: Transform all items (structs, enums, type aliases)
    for item in file.items {
        match item {
            AstItem::Struct(struct_def) => {
                let type_def = transform_struct(struct_def, &alias_resolver)?;
                type_defs.push(TypeDefinition::Struct(type_def));
            }
            AstItem::Enum(enum_def) => {
                let type_def = transform_enum(enum_def, &alias_resolver)?;
                type_defs.push(TypeDefinition::Enum(type_def));
            }
            AstItem::TypeAlias(alias_def) => {
                let type_def = transform_type_alias(alias_def, &alias_resolver)?;
                type_defs.push(TypeDefinition::TypeAlias(type_def));
            }
            // Module and Use statements are not yet transformed in this pass
            // They will be handled by ModuleResolver in #53b
            AstItem::Module(_) | AstItem::Use(_) => {
                // Skip for now - module resolution is not yet implemented
            }
        }
    }

    // Validate user-defined type references
    validate_user_defined_types(&type_defs)?;

    // Emit deprecation warnings
    emit_deprecation_warnings(&type_defs);

    Ok(type_defs)
}

/// Transform AST to IR with a pre-populated type alias resolver
///
/// This is useful when resolving imports across multiple files,
/// where type aliases from all files need to be available.
///
/// # Example
///
/// ```rust,ignore
/// use lumos_core::{transform::TypeAliasResolver, parser};
///
/// let mut resolver = TypeAliasResolver::new();
///
/// // Load type aliases from all files
/// for file in &all_files {
///     let ast = parser::parse_lumos_file(file)?;
///     for item in &ast.items {
///         if let Item::TypeAlias(alias) = item {
///             resolver.add_alias(alias.name.clone(), alias.target.clone())?;
///         }
///     }
/// }
///
/// // Resolve all aliases once
/// resolver.resolve_all_aliases()?;
///
/// // Transform all files with shared resolver
/// for file in all_files {
///     let ir = transform_to_ir_with_resolver(file, &resolver)?;
/// }
/// ```
pub fn transform_to_ir_with_resolver(
    file: LumosFile,
    resolver: &TypeAliasResolver,
) -> Result<Vec<TypeDefinition>> {
    transform_to_ir_with_resolver_impl(file, resolver, true)
}

/// Transform with optional validation skip (for multi-file scenarios)
pub fn transform_to_ir_with_resolver_no_validation(
    file: LumosFile,
    resolver: &TypeAliasResolver,
) -> Result<Vec<TypeDefinition>> {
    transform_to_ir_with_resolver_impl(file, resolver, false)
}

fn transform_to_ir_with_resolver_impl(
    file: LumosFile,
    resolver: &TypeAliasResolver,
    validate: bool,
) -> Result<Vec<TypeDefinition>> {
    let mut type_defs = Vec::new();

    // Transform all items using the provided resolver
    for item in file.items {
        match item {
            AstItem::Struct(struct_def) => {
                let type_def = transform_struct(struct_def, resolver)?;
                type_defs.push(TypeDefinition::Struct(type_def));
            }
            AstItem::Enum(enum_def) => {
                let type_def = transform_enum(enum_def, resolver)?;
                type_defs.push(TypeDefinition::Enum(type_def));
            }
            AstItem::TypeAlias(alias_def) => {
                let type_def = transform_type_alias(alias_def, resolver)?;
                type_defs.push(TypeDefinition::TypeAlias(type_def));
            }
            // Module and Use statements are not yet transformed
            // They will be handled by ModuleResolver in #53b
            AstItem::Module(_) | AstItem::Use(_) => {
                // Skip for now - module resolution is not yet implemented
            }
        }
    }

    // Validate user-defined type references (skip for multi-file scenarios)
    if validate {
        validate_user_defined_types(&type_defs)?;
    }

    // Emit deprecation warnings
    emit_deprecation_warnings(&type_defs);

    Ok(type_defs)
}

/// Transform a type alias definition
fn transform_type_alias(
    alias_def: AstTypeAlias,
    resolver: &TypeAliasResolver,
) -> Result<TypeAliasDefinition> {
    let name = alias_def.name;

    // Get the resolved target type from the resolver
    let target = resolver
        .get_resolved(&name)
        .ok_or_else(|| {
            LumosError::Transform(
                format!("Type alias '{}' was not resolved", name),
                None,
            )
        })?
        .clone();

    Ok(TypeAliasDefinition { name, target })
}

/// Transform a single struct definition
fn transform_struct(struct_def: AstStruct, resolver: &TypeAliasResolver) -> Result<StructDefinition> {
    // Extract metadata from attributes BEFORE consuming struct
    let metadata = extract_struct_metadata(&struct_def);

    let name = struct_def.name;

    // Transform fields
    let fields = struct_def
        .fields
        .into_iter()
        .map(|f| transform_field(f, resolver))
        .collect::<Result<Vec<_>>>()?;

    Ok(StructDefinition {
        name,
        fields,
        metadata,
    })
}

/// Transform a single enum definition
fn transform_enum(enum_def: AstEnum, resolver: &TypeAliasResolver) -> Result<EnumDefinition> {
    // Extract metadata from attributes BEFORE consuming enum
    let metadata = extract_enum_metadata(&enum_def);

    let name = enum_def.name;

    // Transform variants
    let variants = enum_def
        .variants
        .into_iter()
        .map(|v| transform_enum_variant(v, resolver))
        .collect::<Result<Vec<_>>>()?;

    Ok(EnumDefinition {
        name,
        variants,
        metadata,
    })
}

/// Transform an enum variant
fn transform_enum_variant(variant: AstEnumVariant, resolver: &TypeAliasResolver) -> Result<EnumVariantDefinition> {
    match variant {
        AstEnumVariant::Unit { name, .. } => Ok(EnumVariantDefinition::Unit { name }),

        AstEnumVariant::Tuple { name, types, .. } => {
            let transformed_types = types
                .into_iter()
                .map(|t| transform_type(t, false, resolver))
                .collect::<Result<Vec<_>>>()?;

            Ok(EnumVariantDefinition::Tuple {
                name,
                types: transformed_types,
            })
        }

        AstEnumVariant::Struct { name, fields, .. } => {
            let transformed_fields = fields
                .into_iter()
                .map(|f| transform_field(f, resolver))
                .collect::<Result<Vec<_>>>()?;

            Ok(EnumVariantDefinition::Struct {
                name,
                fields: transformed_fields,
            })
        }
    }
}

/// Transform a field definition
fn transform_field(field: AstField, resolver: &TypeAliasResolver) -> Result<FieldDefinition> {
    let name = field.name;
    let optional = field.optional;

    // Extract deprecation info from attributes
    let deprecated = extract_deprecation(&field.attributes);

    // Transform type using the alias resolver
    let type_info = transform_type(field.type_spec, optional, resolver)?;

    Ok(FieldDefinition {
        name,
        type_info,
        optional,
        deprecated,
    })
}

/// Transform type specification with alias resolution
fn transform_type(type_spec: AstType, optional: bool, resolver: &TypeAliasResolver) -> Result<TypeInfo> {
    let base_type = match type_spec {
        AstType::Primitive(name) => {
            // Check if it's a known primitive type
            if is_valid_primitive_type(&name) {
                // Map TypeScript-friendly aliases to Rust types
                let rust_type = map_type_alias(&name);
                TypeInfo::Primitive(rust_type)
            } else if let Some(resolved) = resolver.get_resolved(&name) {
                // It's a type alias, use the resolved type
                resolved.clone()
            } else {
                // Treat as user-defined type (enum or struct defined in schema)
                // Validation of whether the type actually exists happens in a later phase
                TypeInfo::UserDefined(name)
            }
        }

        AstType::Array(inner) => {
            let inner_type = transform_type(*inner, false, resolver)?;
            TypeInfo::Array(Box::new(inner_type))
        }

        AstType::FixedArray { element, size } => {
            let element_type = transform_type(*element, false, resolver)?;
            TypeInfo::FixedArray {
                element: Box::new(element_type),
                size,
            }
        }

        AstType::UserDefined(name) => {
            // Check if it's a type alias first
            if let Some(resolved) = resolver.get_resolved(&name) {
                resolved.clone()
            } else {
                // User-defined type (struct/enum)
                // Validated after full transformation via validate_user_defined_types()
                TypeInfo::UserDefined(name)
            }
        }
    };

    // Wrap in Option if optional
    if optional {
        Ok(TypeInfo::Option(Box::new(base_type)))
    } else {
        Ok(base_type)
    }
}

/// Check if a type name is a valid primitive type
fn is_valid_primitive_type(name: &str) -> bool {
    matches!(
        name,
        // Unsigned integers
        "u8" | "u16" | "u32" | "u64" | "u128" |
        // Signed integers
        "i8" | "i16" | "i32" | "i64" | "i128" |
        // Floating point
        "f32" | "f64" |
        // Boolean
        "bool" |
        // String
        "String" |
        // Solana types
        "PublicKey" | "Signature" | "Keypair" |
        // TypeScript aliases
        "number" | "string" | "boolean"
    )
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
        version: struct_def.version.clone(),
        custom_derives: extract_custom_derives(&struct_def.attributes),
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
        version: enum_def.version.clone(),
        custom_derives: extract_custom_derives(&enum_def.attributes),
    }
}

/// Extract deprecation information from field attributes
///
/// Returns `Some(message)` if the field has a `#[deprecated]` attribute,
/// otherwise returns `None`.
///
/// Supports two forms:
/// - `#[deprecated]` - uses default message
/// - `#[deprecated("reason")]` - uses custom message
fn extract_deprecation(attributes: &[Attribute]) -> Option<String> {
    attributes
        .iter()
        .find(|attr| attr.name == "deprecated")
        .map(|attr| {
            // Extract custom message if provided
            if let Some(AttributeValue::String(msg)) = &attr.value {
                msg.clone()
            } else {
                "This field is deprecated".to_string()
            }
        })
}

/// Extract custom derive macros from attributes
///
/// Returns a vector of derive macro names from `#[derive(...)]` attribute,
/// or an empty vector if no derive attribute is present.
///
/// # Arguments
///
/// * `attributes` - List of attributes to search
///
/// # Returns
///
/// Vector of derive macro names (e.g., `vec!["PartialEq", "Eq", "Hash"]`)
///
/// # Example
///
/// ```ignore
/// // Attribute: #[derive(PartialEq, Eq, Hash)]
/// // Returns: vec!["PartialEq", "Eq", "Hash"]
/// ```
fn extract_custom_derives(attributes: &[Attribute]) -> Vec<String> {
    attributes
        .iter()
        .find(|attr| attr.name == "derive")
        .and_then(|attr| {
            // Extract list of derive macros if present
            if let Some(AttributeValue::List(derives)) = &attr.value {
                Some(derives.clone())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// Emit deprecation warnings for all deprecated fields in the schema
///
/// This function scans all type definitions and emits warnings to stderr
/// for any fields marked with the `#[deprecated]` attribute.
fn emit_deprecation_warnings(type_defs: &[TypeDefinition]) {
    use colored::Colorize;

    for type_def in type_defs {
        match type_def {
            TypeDefinition::Struct(s) => {
                // Check struct fields
                for field in &s.fields {
                    if let Some(msg) = &field.deprecated {
                        eprintln!(
                            "{} {}.{}: {}",
                            "warning:".yellow().bold(),
                            s.name,
                            field.name,
                            msg
                        );
                    }
                }
            }
            TypeDefinition::Enum(e) => {
                // Check enum struct variant fields
                for variant in &e.variants {
                    if let EnumVariantDefinition::Struct { name, fields } = variant {
                        for field in fields {
                            if let Some(msg) = &field.deprecated {
                                eprintln!(
                                    "{} {}.{}::{}: {}",
                                    "warning:".yellow().bold(),
                                    e.name,
                                    name,
                                    field.name,
                                    msg
                                );
                            }
                        }
                    }
                }
            }
            TypeDefinition::TypeAlias(_) => {
                // Type aliases don't have fields that can be deprecated
            }
        }
    }
}

/// Validate that all user-defined type references are defined in the schema
///
/// This function ensures type safety by catching references to undefined types
/// during transformation rather than at Rust/TypeScript compile time.
///
/// # Arguments
///
/// * `type_defs` - All type definitions in the schema
///
/// # Returns
///
/// * `Ok(())` - All user-defined types are valid
/// * `Err(LumosError::TypeValidation)` - Found reference to undefined type
///
/// # Example
///
/// ```rust,ignore
/// // This would fail validation:
/// struct Player {
///     inventory: UndefinedType  // Error: UndefinedType not found
/// }
/// ```
pub fn validate_user_defined_types(type_defs: &[TypeDefinition]) -> Result<()> {
    use std::collections::HashSet;

    // Collect all defined type names
    let defined_types: HashSet<String> = type_defs.iter().map(|t| t.name().to_string()).collect();

    // Validate each type definition
    for type_def in type_defs {
        match type_def {
            TypeDefinition::Struct(s) => {
                // Validate struct fields
                for field in &s.fields {
                    validate_type_info(&field.type_info, &defined_types, &s.name, &field.name)?;
                }
            }
            TypeDefinition::Enum(e) => {
                // Validate enum variants
                for variant in &e.variants {
                    match variant {
                        EnumVariantDefinition::Unit { .. } => {
                            // Unit variants have no types to validate
                        }
                        EnumVariantDefinition::Tuple { name, types } => {
                            // Validate tuple variant types
                            for (idx, type_info) in types.iter().enumerate() {
                                let context = format!("{}.{}[{}]", e.name, name, idx);
                                validate_type_info(type_info, &defined_types, &context, "")?;
                            }
                        }
                        EnumVariantDefinition::Struct { name, fields } => {
                            // Validate struct variant fields
                            for field in fields {
                                let context = format!("{}.{}", e.name, name);
                                validate_type_info(
                                    &field.type_info,
                                    &defined_types,
                                    &context,
                                    &field.name,
                                )?;
                            }
                        }
                    }
                }
            }
            TypeDefinition::TypeAlias(a) => {
                // Type aliases are already resolved - validate their target type
                validate_type_info(&a.target, &defined_types, &a.name, "")?;
            }
        }
    }

    Ok(())
}

/// Recursively validate a TypeInfo against defined types
///
/// # Arguments
///
/// * `type_info` - The type to validate
/// * `defined_types` - Set of all defined type names
/// * `parent_context` - Parent type name for error messages (e.g., "Player")
/// * `field_name` - Field name for error messages (e.g., "inventory")
fn validate_type_info(
    type_info: &TypeInfo,
    defined_types: &std::collections::HashSet<String>,
    parent_context: &str,
    field_name: &str,
) -> Result<()> {
    use crate::error::LumosError;

    match type_info {
        TypeInfo::Primitive(_) => {
            // Primitive types are always valid
            Ok(())
        }
        TypeInfo::UserDefined(type_name) => {
            // Check if the user-defined type exists
            if !defined_types.contains(type_name) {
                let location = if field_name.is_empty() {
                    parent_context.to_string()
                } else {
                    format!("{}.{}", parent_context, field_name)
                };
                return Err(LumosError::TypeValidation(
                    format!(
                        "Undefined type '{}' referenced in '{}'",
                        type_name, location
                    ),
                    None, // TODO: Add actual source location from AST spans
                ));
            }
            Ok(())
        }
        TypeInfo::Array(inner) => {
            // Recursively validate array element type
            validate_type_info(inner, defined_types, parent_context, field_name)
        }
        TypeInfo::FixedArray { element, .. } => {
            // Recursively validate fixed array element type
            validate_type_info(element, defined_types, parent_context, field_name)
        }
        TypeInfo::Option(inner) => {
            // Recursively validate optional type
            validate_type_info(inner, defined_types, parent_context, field_name)
        }
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

    // Type validation tests
    #[test]
    fn test_validate_undefined_type_in_struct() {
        let input = r#"
            struct Player {
                inventory: UndefinedType,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            crate::error::LumosError::TypeValidation(_, _)
        ));
        assert!(err.to_string().contains("Undefined type 'UndefinedType'"));
        assert!(err.to_string().contains("Player.inventory"));
    }

    #[test]
    fn test_validate_undefined_type_in_array() {
        let input = r#"
            struct Inventory {
                items: [MissingItem],
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Undefined type 'MissingItem'"));
    }

    #[test]
    fn test_validate_undefined_type_in_option() {
        let input = r#"
            struct User {
                profile: Option<NonexistentProfile>,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("Undefined type 'NonexistentProfile'"));
    }

    #[test]
    fn test_validate_undefined_type_in_enum_tuple_variant() {
        let input = r#"
            enum GameEvent {
                PlayerJoined(UndefinedPlayer),
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Undefined type 'UndefinedPlayer'"));
    }

    #[test]
    fn test_validate_undefined_type_in_enum_struct_variant() {
        let input = r#"
            enum Instruction {
                Initialize {
                    config: MissingConfig,
                },
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Undefined type 'MissingConfig'"));
    }

    #[test]
    fn test_validate_valid_user_defined_types() {
        let input = r#"
            struct Item {
                id: u64,
                name: String,
            }

            struct Inventory {
                items: [Item],
                selected: Option<Item>,
            }

            enum GameState {
                Playing {
                    inventory: Inventory,
                },
                GameOver,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        // Should succeed - all user-defined types are valid
        assert!(result.is_ok());
        let ir = result.unwrap();
        assert_eq!(ir.len(), 3);
    }

    #[test]
    fn test_validate_nested_user_defined_types() {
        let input = r#"
            struct Inner {
                value: u64,
            }

            struct Middle {
                inner: Inner,
            }

            struct Outer {
                middle: [Middle],
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        // Should succeed - nested references are valid
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_primitive_types_always_valid() {
        let input = r#"
            struct AllPrimitives {
                a: u8,
                b: u16,
                c: u32,
                d: u64,
                e: u128,
                f: i8,
                g: i16,
                h: i32,
                i: i64,
                j: i128,
                k: bool,
                l: String,
                m: PublicKey,
                n: Signature,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        // Should succeed - all primitive types
        assert!(result.is_ok());
    }

    #[test]
    fn test_deprecated_field_without_message() {
        let input = r#"
            struct User {
                name: String,
                #[deprecated]
                old_field: u64,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        // Check that the deprecated field is marked correctly
        if let TypeDefinition::Struct(s) = &ir[0] {
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].deprecated, None);
            assert_eq!(
                s.fields[1].deprecated,
                Some("This field is deprecated".to_string())
            );
        } else {
            panic!("Expected struct definition");
        }
    }

    #[test]
    fn test_deprecated_field_with_message() {
        let input = r#"
            struct Account {
                balance: u64,
                #[deprecated("Use new_email field instead")]
                email: String,
                new_email: Option<String>,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        // Check that the deprecated field has custom message
        if let TypeDefinition::Struct(s) = &ir[0] {
            assert_eq!(s.fields.len(), 3);
            assert_eq!(s.fields[0].deprecated, None);
            assert_eq!(
                s.fields[1].deprecated,
                Some("Use new_email field instead".to_string())
            );
            assert_eq!(s.fields[2].deprecated, None);
        } else {
            panic!("Expected struct definition");
        }
    }

    #[test]
    fn test_deprecated_field_in_enum_variant() {
        let input = r#"
            #[solana]
            enum Instruction {
                Initialize {
                    authority: PublicKey,
                    #[deprecated]
                    old_param: u64,
                },
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        // Check that enum variant field is marked as deprecated
        if let TypeDefinition::Enum(e) = &ir[0] {
            if let EnumVariantDefinition::Struct { fields, .. } = &e.variants[0] {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].deprecated, None);
                assert_eq!(
                    fields[1].deprecated,
                    Some("This field is deprecated".to_string())
                );
            } else {
                panic!("Expected struct variant");
            }
        } else {
            panic!("Expected enum definition");
        }
    }

    #[test]
    fn test_custom_derives_on_struct() {
        let input = r#"
            #[derive(PartialEq, Eq, Hash)]
            struct Account {
                balance: u64,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        // Check that custom derives are extracted
        if let TypeDefinition::Struct(s) = &ir[0] {
            assert_eq!(s.metadata.custom_derives.len(), 3);
            assert_eq!(s.metadata.custom_derives[0], "PartialEq");
            assert_eq!(s.metadata.custom_derives[1], "Eq");
            assert_eq!(s.metadata.custom_derives[2], "Hash");
        } else {
            panic!("Expected struct definition");
        }
    }

    #[test]
    fn test_custom_derives_on_enum() {
        let input = r#"
            #[derive(PartialEq, Eq)]
            enum GameState {
                Active,
                Paused,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        // Check that custom derives are extracted
        if let TypeDefinition::Enum(e) = &ir[0] {
            assert_eq!(e.metadata.custom_derives.len(), 2);
            assert_eq!(e.metadata.custom_derives[0], "PartialEq");
            assert_eq!(e.metadata.custom_derives[1], "Eq");
        } else {
            panic!("Expected enum definition");
        }
    }

    #[test]
    fn test_custom_derives_with_solana_attrs() {
        let input = r#"
            #[solana]
            #[derive(PartialEq, Eq, Hash)]
            struct UserAccount {
                wallet: PublicKey,
                balance: u64,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        // Check that both solana attr and custom derives are present
        if let TypeDefinition::Struct(s) = &ir[0] {
            assert!(s.metadata.solana);
            assert_eq!(s.metadata.custom_derives.len(), 3);
            assert_eq!(s.metadata.custom_derives[0], "PartialEq");
            assert_eq!(s.metadata.custom_derives[1], "Eq");
            assert_eq!(s.metadata.custom_derives[2], "Hash");
        } else {
            panic!("Expected struct definition");
        }
    }

    #[test]
    fn test_no_custom_derives() {
        let input = r#"
            #[solana]
            struct Account {
                balance: u64,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let ir = transform_to_ir(ast).unwrap();

        // Check that custom_derives is empty when no derive attribute present
        if let TypeDefinition::Struct(s) = &ir[0] {
            assert_eq!(s.metadata.custom_derives.len(), 0);
        } else {
            panic!("Expected struct definition");
        }
    }
}
