// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Schema migration code generation
//!
//! This module provides functionality to compare two schema versions
//! and generate migration code to transform data from one version to another.

use crate::ir::{EnumDefinition, FieldDefinition, StructDefinition, TypeDefinition, TypeInfo};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Represents a change between two schema versions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchemaChange {
    /// A field was added
    FieldAdded {
        name: String,
        type_info: TypeInfo,
        optional: bool,
    },

    /// A field was removed
    FieldRemoved { name: String, type_info: TypeInfo },

    /// A field's type was changed
    FieldTypeChanged {
        name: String,
        old_type: TypeInfo,
        new_type: TypeInfo,
    },

    /// A field was reordered
    FieldReordered {
        name: String,
        old_position: usize,
        new_position: usize,
    },

    /// An enum variant was added
    VariantAdded { name: String },

    /// An enum variant was removed
    VariantRemoved { name: String },
}

/// Represents the difference between two schema versions
#[derive(Debug, Clone)]
pub struct SchemaDiff {
    /// The name of the type being compared
    pub type_name: String,

    /// Version being migrated from
    pub from_version: Option<String>,

    /// Version being migrated to
    pub to_version: Option<String>,

    /// List of detected changes
    pub changes: Vec<SchemaChange>,

    /// Whether the migration is safe (backward compatible)
    pub is_safe: bool,
}

/// Safety classification for migrations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationSafety {
    /// Safe migration (backward compatible)
    Safe,

    /// Unsafe migration (potential data loss or incompatibility)
    Unsafe,
}

impl SchemaDiff {
    /// Create a new empty SchemaDiff
    pub fn new(
        type_name: String,
        from_version: Option<String>,
        to_version: Option<String>,
    ) -> Self {
        Self {
            type_name,
            from_version,
            to_version,
            changes: Vec::new(),
            is_safe: true,
        }
    }

    /// Compute the difference between two type definitions
    pub fn compute(old: &TypeDefinition, new: &TypeDefinition) -> Result<Self, String> {
        if old.name() != new.name() {
            return Err(format!(
                "Type names don't match: '{}' vs '{}'",
                old.name(),
                new.name()
            ));
        }

        match (old, new) {
            (TypeDefinition::Struct(old_struct), TypeDefinition::Struct(new_struct)) => {
                Self::compute_struct_diff(old_struct, new_struct)
            }
            (TypeDefinition::Enum(old_enum), TypeDefinition::Enum(new_enum)) => {
                Self::compute_enum_diff(old_enum, new_enum)
            }
            _ => Err(format!(
                "Cannot compare different type kinds for '{}'",
                old.name()
            )),
        }
    }

    /// Compute diff for struct types
    fn compute_struct_diff(old: &StructDefinition, new: &StructDefinition) -> Result<Self, String> {
        let mut diff = Self::new(
            old.name.clone(),
            old.metadata.version.clone(),
            new.metadata.version.clone(),
        );

        // Create maps for efficient lookup
        let old_fields: HashMap<&str, &FieldDefinition> =
            old.fields.iter().map(|f| (f.name.as_str(), f)).collect();
        let new_fields: HashMap<&str, &FieldDefinition> =
            new.fields.iter().map(|f| (f.name.as_str(), f)).collect();

        let old_field_names: HashSet<&str> = old_fields.keys().copied().collect();
        let new_field_names: HashSet<&str> = new_fields.keys().copied().collect();

        // Detect removed fields
        for removed_name in old_field_names.difference(&new_field_names) {
            let field = old_fields[removed_name];
            diff.changes.push(SchemaChange::FieldRemoved {
                name: removed_name.to_string(),
                type_info: field.type_info.clone(),
            });
            diff.is_safe = false; // Removing fields is unsafe
        }

        // Detect added fields
        for added_name in new_field_names.difference(&old_field_names) {
            let field = new_fields[added_name];
            diff.changes.push(SchemaChange::FieldAdded {
                name: added_name.to_string(),
                type_info: field.type_info.clone(),
                optional: field.optional,
            });

            // Adding required fields is unsafe
            if !field.optional {
                diff.is_safe = false;
            }
        }

        // Detect type changes in existing fields
        for common_name in old_field_names.intersection(&new_field_names) {
            let old_field = old_fields[common_name];
            let new_field = new_fields[common_name];

            if !type_info_equal(&old_field.type_info, &new_field.type_info) {
                diff.changes.push(SchemaChange::FieldTypeChanged {
                    name: common_name.to_string(),
                    old_type: old_field.type_info.clone(),
                    new_type: new_field.type_info.clone(),
                });
                diff.is_safe = false; // Type changes are generally unsafe
            }
        }

        // Detect reordering
        let old_positions: HashMap<&str, usize> = old
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.as_str(), i))
            .collect();
        let new_positions: HashMap<&str, usize> = new
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.as_str(), i))
            .collect();

        for name in old_field_names.intersection(&new_field_names) {
            let old_pos = old_positions[name];
            let new_pos = new_positions[name];
            if old_pos != new_pos {
                diff.changes.push(SchemaChange::FieldReordered {
                    name: name.to_string(),
                    old_position: old_pos,
                    new_position: new_pos,
                });
                // Reordering is safe with Borsh (uses field names, not positions)
            }
        }

        Ok(diff)
    }

    /// Compute diff for enum types
    fn compute_enum_diff(old: &EnumDefinition, new: &EnumDefinition) -> Result<Self, String> {
        let mut diff = Self::new(
            old.name.clone(),
            old.metadata.version.clone(),
            new.metadata.version.clone(),
        );

        let old_variants: HashSet<&str> = old.variants.iter().map(|v| v.name()).collect();
        let new_variants: HashSet<&str> = new.variants.iter().map(|v| v.name()).collect();

        // Detect removed variants
        for removed_name in old_variants.difference(&new_variants) {
            diff.changes.push(SchemaChange::VariantRemoved {
                name: removed_name.to_string(),
            });
            diff.is_safe = false; // Removing variants is unsafe
        }

        // Detect added variants
        for added_name in new_variants.difference(&old_variants) {
            diff.changes.push(SchemaChange::VariantAdded {
                name: added_name.to_string(),
            });
            // Adding variants is generally safe
        }

        Ok(diff)
    }

    /// Check if the migration is safe
    pub fn safety(&self) -> MigrationSafety {
        if self.is_safe {
            MigrationSafety::Safe
        } else {
            MigrationSafety::Unsafe
        }
    }

    /// Get a human-readable description of the changes
    pub fn describe(&self) -> String {
        let mut desc = Vec::new();

        desc.push(format!("Comparing schemas for '{}':", self.type_name));
        if let Some(ref from) = self.from_version {
            desc.push(format!("  From: v{}", from));
        }
        if let Some(ref to) = self.to_version {
            desc.push(format!("  To: v{}", to));
        }
        desc.push(String::new());

        if self.changes.is_empty() {
            desc.push("No changes detected.".to_string());
        } else {
            desc.push(format!("Changes detected ({}):", self.changes.len()));
            for change in &self.changes {
                desc.push(format!("  {}", describe_change(change)));
            }
        }

        desc.push(String::new());
        desc.push(format!(
            "Migration is {}",
            if self.is_safe {
                "SAFE ✓"
            } else {
                "UNSAFE ⚠"
            }
        ));

        desc.join("\n")
    }
}

/// Compare two TypeInfo for equality
fn type_info_equal(a: &TypeInfo, b: &TypeInfo) -> bool {
    match (a, b) {
        (TypeInfo::Primitive(a_name), TypeInfo::Primitive(b_name)) => a_name == b_name,
        (TypeInfo::UserDefined(a_name), TypeInfo::UserDefined(b_name)) => a_name == b_name,
        (TypeInfo::Array(a_inner), TypeInfo::Array(b_inner)) => type_info_equal(a_inner, b_inner),
        (TypeInfo::Option(a_inner), TypeInfo::Option(b_inner)) => type_info_equal(a_inner, b_inner),
        _ => false,
    }
}

/// Get a human-readable description of a change
fn describe_change(change: &SchemaChange) -> String {
    match change {
        SchemaChange::FieldAdded {
            name,
            type_info,
            optional,
        } => {
            let opt_str = if *optional { " (optional)" } else { "" };
            format!(
                "✓ Added field: {} ({}){}",
                name,
                type_info_display(type_info),
                opt_str
            )
        }
        SchemaChange::FieldRemoved { name, type_info } => {
            format!(
                "⚠ Removed field: {} ({})",
                name,
                type_info_display(type_info)
            )
        }
        SchemaChange::FieldTypeChanged {
            name,
            old_type,
            new_type,
        } => {
            format!(
                "⚠ Changed field type: {} ({} -> {})",
                name,
                type_info_display(old_type),
                type_info_display(new_type)
            )
        }
        SchemaChange::FieldReordered {
            name,
            old_position,
            new_position,
        } => {
            format!(
                "↕ Reordered field: {} (position {} -> {})",
                name, old_position, new_position
            )
        }
        SchemaChange::VariantAdded { name } => {
            format!("✓ Added enum variant: {}", name)
        }
        SchemaChange::VariantRemoved { name } => {
            format!("⚠ Removed enum variant: {}", name)
        }
    }
}

/// Get a display string for TypeInfo
fn type_info_display(type_info: &TypeInfo) -> String {
    match type_info {
        TypeInfo::Primitive(name) => name.clone(),
        TypeInfo::Generic(param_name) => param_name.clone(),
        TypeInfo::UserDefined(name) => name.clone(),
        TypeInfo::Array(inner) => format!("[{}]", type_info_display(inner)),
        TypeInfo::FixedArray { element, size } => {
            format!("[{}; {}]", type_info_display(element), size)
        }
        TypeInfo::Option(inner) => format!("Option<{}>", type_info_display(inner)),
    }
}

/// Generate Rust migration code from a SchemaDiff
pub fn generate_rust_migration(diff: &SchemaDiff, old_def: &TypeDefinition) -> String {
    match old_def {
        TypeDefinition::Struct(old_struct) => generate_rust_struct_migration(diff, old_struct),
        TypeDefinition::Enum(_) => {
            // Enum migrations are not yet fully supported
            format!("// TODO: Enum migration for {}\n", diff.type_name)
        }
        TypeDefinition::TypeAlias(_) => {
            // Type aliases are resolved during transformation and don't need migrations
            format!("// Type alias '{}' - no migration needed (resolved at compile time)\n", diff.type_name)
        }
    }
}

/// Generate Rust migration code for structs
fn generate_rust_struct_migration(diff: &SchemaDiff, old_struct: &StructDefinition) -> String {
    let from_version = diff
        .from_version
        .as_deref()
        .unwrap_or("unknown")
        .replace('.', "_");
    let _to_version = diff
        .to_version
        .as_deref()
        .unwrap_or("current")
        .replace('.', "_");

    let old_struct_name = format!("{}V{}", diff.type_name, from_version);
    let new_struct_name = &diff.type_name;

    let mut code = Vec::new();

    // Add header comment
    code.push(format!(
        "// Auto-generated migration code by LUMOS\n\
         // Migration from v{} to v{}\n",
        diff.from_version.as_deref().unwrap_or("unknown"),
        diff.to_version.as_deref().unwrap_or("current")
    ));

    // Generate old struct definition
    code.push("\n// Old version struct definition\n".to_string());
    code.push("#[derive(BorshSerialize, BorshDeserialize)]\n".to_string());
    code.push(format!("pub struct {} {{\n", old_struct_name));
    for field in &old_struct.fields {
        let rust_type = map_type_to_rust(&field.type_info, field.optional);
        code.push(format!("    pub {}: {},\n", field.name, rust_type));
    }
    code.push("}\n".to_string());

    // Generate migration impl
    code.push(format!("\nimpl {} {{\n", new_struct_name));
    code.push(format!(
        "    /// Migrate from v{} to v{}\n",
        diff.from_version.as_deref().unwrap_or("unknown"),
        diff.to_version.as_deref().unwrap_or("current")
    ));

    // Add change documentation
    if !diff.changes.is_empty() {
        code.push("    ///\n".to_string());
        code.push("    /// Changes:\n".to_string());
        for change in &diff.changes {
            match change {
                SchemaChange::FieldAdded { name, .. } => {
                    code.push(format!("    /// - Added field: {}\n", name));
                }
                SchemaChange::FieldRemoved { name, .. } => {
                    code.push(format!("    /// - Removed field: {}\n", name));
                }
                SchemaChange::FieldTypeChanged { name, .. } => {
                    code.push(format!("    /// - Changed field type: {}\n", name));
                }
                _ => {}
            }
        }
    }

    code.push(format!(
        "    pub fn migrate_from_v{}(old: {}) -> Self {{\n",
        from_version, old_struct_name
    ));
    code.push("        Self {\n".to_string());

    // Build field mapping
    let old_fields: HashMap<&str, &FieldDefinition> = old_struct
        .fields
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();

    // Generate field assignments (this needs access to the new struct, we'll infer from changes)
    let mut new_field_names: HashSet<String> = old_fields.keys().map(|&s| s.to_string()).collect();
    for change in &diff.changes {
        match change {
            SchemaChange::FieldAdded { name, .. } => {
                new_field_names.insert(name.clone());
            }
            SchemaChange::FieldRemoved { name, .. } => {
                new_field_names.remove(name);
            }
            _ => {}
        }
    }

    for field_name in &new_field_names {
        if old_fields.contains_key(field_name.as_str()) {
            // Field exists in old version - copy it
            code.push(format!("            {}: old.{},\n", field_name, field_name));
        } else {
            // Field was added - use default value
            let default_value = get_default_value_for_added_field(field_name, &diff.changes);
            code.push(format!(
                "            {}: {}, // Default: Added in v{}\n",
                field_name,
                default_value,
                diff.to_version.as_deref().unwrap_or("current")
            ));
        }
    }

    code.push("        }\n".to_string());
    code.push("    }\n".to_string());
    code.push("}\n".to_string());

    code.join("")
}

/// Map TypeInfo to Rust type string
fn map_type_to_rust(type_info: &TypeInfo, optional: bool) -> String {
    let base_type = match type_info {
        TypeInfo::Primitive(name) => match name.as_str() {
            "string" => "String",
            "PublicKey" => "Pubkey",
            other => other,
        }
        .to_string(),
        TypeInfo::Generic(param_name) => param_name.clone(),
        TypeInfo::UserDefined(name) => name.clone(),
        TypeInfo::Array(inner) => format!("Vec<{}>", map_type_to_rust(inner, false)),
        TypeInfo::FixedArray { element, size } => {
            format!("[{}; {}]", map_type_to_rust(element, false), size)
        }
        TypeInfo::Option(inner) => return format!("Option<{}>", map_type_to_rust(inner, false)),
    };

    if optional {
        format!("Option<{}>", base_type)
    } else {
        base_type
    }
}

/// Get default value for an added field
fn get_default_value_for_added_field(field_name: &str, changes: &[SchemaChange]) -> String {
    for change in changes {
        if let SchemaChange::FieldAdded {
            name,
            type_info,
            optional,
        } = change
        {
            if name == field_name {
                if *optional {
                    return "None".to_string();
                }
                return get_default_value_for_type(type_info);
            }
        }
    }
    "Default::default()".to_string()
}

/// Get default value for a TypeInfo
fn get_default_value_for_type(type_info: &TypeInfo) -> String {
    match type_info {
        TypeInfo::Primitive(name) => match name.as_str() {
            "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128" => "0",
            "f32" | "f64" => "0.0",
            "bool" => "false",
            "string" | "String" => "String::new()",
            "PublicKey" => "Pubkey::default()",
            _ => "Default::default()",
        }
        .to_string(),
        TypeInfo::Generic(_) => "Default::default()".to_string(),
        TypeInfo::UserDefined(_) => "Default::default()".to_string(),
        TypeInfo::Array(_) => "Vec::new()".to_string(),
        TypeInfo::FixedArray { element, size } => {
            // For fixed arrays, generate [default(); size]
            let elem_default = get_default_value_for_type(element);
            format!("[{}; {}]", elem_default, size)
        }
        TypeInfo::Option(_) => "None".to_string(),
    }
}

/// Generate TypeScript migration code from a SchemaDiff
pub fn generate_typescript_migration(diff: &SchemaDiff, old_def: &TypeDefinition) -> String {
    match old_def {
        TypeDefinition::Struct(old_struct) => {
            generate_typescript_struct_migration(diff, old_struct)
        }
        TypeDefinition::Enum(_) => {
            // Enum migrations are not yet fully supported
            format!("// TODO: Enum migration for {}\n", diff.type_name)
        }
        TypeDefinition::TypeAlias(_) => {
            // Type aliases are resolved during transformation and don't need migrations
            format!("// Type alias '{}' - no migration needed (resolved at compile time)\n", diff.type_name)
        }
    }
}

/// Generate TypeScript migration code for structs
fn generate_typescript_struct_migration(
    diff: &SchemaDiff,
    old_struct: &StructDefinition,
) -> String {
    let from_version = diff
        .from_version
        .as_deref()
        .unwrap_or("unknown")
        .replace('.', "_");
    let _to_version = diff
        .to_version
        .as_deref()
        .unwrap_or("current")
        .replace('.', "_");

    let old_type_name = format!("{}V{}", diff.type_name, from_version);
    let new_type_name = &diff.type_name;
    let fn_name = format!("migrate{}FromV{}", diff.type_name, from_version);

    let mut code = Vec::new();

    // Add header comment
    code.push(format!(
        "// Auto-generated migration code by LUMOS\n\
         // Migration from v{} to v{}\n\n",
        diff.from_version.as_deref().unwrap_or("unknown"),
        diff.to_version.as_deref().unwrap_or("current")
    ));

    // Generate old interface definition
    code.push("// Old version interface\n".to_string());
    code.push(format!("export interface {} {{\n", old_type_name));
    for field in &old_struct.fields {
        let ts_type = map_type_to_typescript(&field.type_info, field.optional);
        let optional_marker = if field.optional { "?" } else { "" };
        code.push(format!(
            "  {}{}: {};\n",
            field.name, optional_marker, ts_type
        ));
    }
    code.push("}\n\n".to_string());

    // Generate migration function
    code.push("/**\n".to_string());
    code.push(format!(
        " * Migrate {} from v{} to v{}\n",
        diff.type_name,
        diff.from_version.as_deref().unwrap_or("unknown"),
        diff.to_version.as_deref().unwrap_or("current")
    ));

    if !diff.changes.is_empty() {
        code.push(" *\n".to_string());
        code.push(" * Changes:\n".to_string());
        for change in &diff.changes {
            match change {
                SchemaChange::FieldAdded { name, .. } => {
                    code.push(format!(" * - Added field: {}\n", name));
                }
                SchemaChange::FieldRemoved { name, .. } => {
                    code.push(format!(" * - Removed field: {}\n", name));
                }
                SchemaChange::FieldTypeChanged { name, .. } => {
                    code.push(format!(" * - Changed field type: {}\n", name));
                }
                _ => {}
            }
        }
    }

    code.push(" */\n".to_string());
    code.push(format!(
        "export function {}(old: {}): {} {{\n",
        fn_name, old_type_name, new_type_name
    ));
    code.push("  return {\n".to_string());

    // Build field mapping
    let old_fields: HashMap<&str, &FieldDefinition> = old_struct
        .fields
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect();

    let mut new_field_names: HashSet<String> = old_fields.keys().map(|&s| s.to_string()).collect();
    for change in &diff.changes {
        match change {
            SchemaChange::FieldAdded { name, .. } => {
                new_field_names.insert(name.clone());
            }
            SchemaChange::FieldRemoved { name, .. } => {
                new_field_names.remove(name);
            }
            _ => {}
        }
    }

    for field_name in &new_field_names {
        if old_fields.contains_key(field_name.as_str()) {
            // Field exists in old version - copy it
            code.push(format!("    {}: old.{},\n", field_name, field_name));
        } else {
            // Field was added - use default value
            let default_value =
                get_typescript_default_value_for_added_field(field_name, &diff.changes);
            code.push(format!(
                "    {}: {}, // Default: Added in v{}\n",
                field_name,
                default_value,
                diff.to_version.as_deref().unwrap_or("current")
            ));
        }
    }

    code.push("  };\n".to_string());
    code.push("}\n".to_string());

    code.join("")
}

/// Map TypeInfo to TypeScript type string
fn map_type_to_typescript(type_info: &TypeInfo, optional: bool) -> String {
    let base_type = match type_info {
        TypeInfo::Primitive(name) => match name.as_str() {
            "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64" => "number",
            "u128" | "i128" => "bigint",
            "bool" => "boolean",
            "string" | "String" => "string",
            "PublicKey" => "PublicKey",
            other => other,
        }
        .to_string(),
        TypeInfo::Generic(param_name) => param_name.clone(),
        TypeInfo::UserDefined(name) => name.clone(),
        TypeInfo::Array(inner) => format!("{}[]", map_type_to_typescript(inner, false)),
        TypeInfo::FixedArray { element, .. } => {
            format!("{}[]", map_type_to_typescript(element, false))
        }
        TypeInfo::Option(inner) => {
            return format!("{} | undefined", map_type_to_typescript(inner, false))
        }
    };

    if optional {
        format!("{} | undefined", base_type)
    } else {
        base_type
    }
}

/// Get default value for an added field in TypeScript
fn get_typescript_default_value_for_added_field(
    field_name: &str,
    changes: &[SchemaChange],
) -> String {
    for change in changes {
        if let SchemaChange::FieldAdded {
            name,
            type_info,
            optional,
        } = change
        {
            if name == field_name {
                if *optional {
                    return "undefined".to_string();
                }
                return get_typescript_default_value_for_type(type_info);
            }
        }
    }
    "undefined".to_string()
}

/// Get default value for a TypeInfo in TypeScript
fn get_typescript_default_value_for_type(type_info: &TypeInfo) -> String {
    match type_info {
        TypeInfo::Primitive(name) => match name.as_str() {
            "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32" | "f64" => "0",
            "u128" | "i128" => "0n",
            "bool" => "false",
            "string" | "String" => "\"\"",
            "PublicKey" => "PublicKey.default",
            _ => "undefined",
        }
        .to_string(),
        TypeInfo::Generic(_) => "undefined".to_string(),
        TypeInfo::UserDefined(_) => "undefined".to_string(),
        TypeInfo::Array(_) => "[]".to_string(),
        TypeInfo::FixedArray { element, size } => {
            // For fixed arrays, generate an array of size filled with defaults
            let elem_default = get_typescript_default_value_for_type(element);
            format!("new Array({}).fill({})", size, elem_default)
        }
        TypeInfo::Option(_) => "undefined".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_struct(name: &str, fields: Vec<(&str, TypeInfo, bool)>) -> StructDefinition {
        StructDefinition {
            name: name.to_string(),
            generic_params: vec![],
            fields: fields
                .into_iter()
                .map(|(field_name, type_info, optional)| FieldDefinition {
                    name: field_name.to_string(),
                    type_info,
                    optional,
                    deprecated: None,
                })
                .collect(),
            metadata: Default::default(),
        }
    }

    #[test]
    fn test_no_changes() {
        let v1 = create_test_struct(
            "User",
            vec![
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("name", TypeInfo::Primitive("String".to_string()), false),
            ],
        );
        let v2 = v1.clone();

        let diff =
            SchemaDiff::compute(&TypeDefinition::Struct(v1), &TypeDefinition::Struct(v2)).unwrap();

        assert_eq!(diff.changes.len(), 0);
        assert_eq!(diff.safety(), MigrationSafety::Safe);
    }

    #[test]
    fn test_field_added_optional() {
        let v1 = create_test_struct(
            "User",
            vec![
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("name", TypeInfo::Primitive("String".to_string()), false),
            ],
        );
        let v2 = create_test_struct(
            "User",
            vec![
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("name", TypeInfo::Primitive("String".to_string()), false),
                ("email", TypeInfo::Primitive("String".to_string()), true),
            ],
        );

        let diff =
            SchemaDiff::compute(&TypeDefinition::Struct(v1), &TypeDefinition::Struct(v2)).unwrap();

        assert_eq!(diff.changes.len(), 1);
        assert!(matches!(
            diff.changes[0],
            SchemaChange::FieldAdded { optional: true, .. }
        ));
        assert_eq!(diff.safety(), MigrationSafety::Safe);
    }

    #[test]
    fn test_field_added_required() {
        let v1 = create_test_struct(
            "User",
            vec![
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("name", TypeInfo::Primitive("String".to_string()), false),
            ],
        );
        let v2 = create_test_struct(
            "User",
            vec![
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("name", TypeInfo::Primitive("String".to_string()), false),
                ("age", TypeInfo::Primitive("u16".to_string()), false),
            ],
        );

        let diff =
            SchemaDiff::compute(&TypeDefinition::Struct(v1), &TypeDefinition::Struct(v2)).unwrap();

        assert_eq!(diff.changes.len(), 1);
        assert!(matches!(
            diff.changes[0],
            SchemaChange::FieldAdded {
                optional: false,
                ..
            }
        ));
        assert_eq!(diff.safety(), MigrationSafety::Unsafe);
    }

    #[test]
    fn test_field_removed() {
        let v1 = create_test_struct(
            "User",
            vec![
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("name", TypeInfo::Primitive("String".to_string()), false),
                ("old_field", TypeInfo::Primitive("bool".to_string()), false),
            ],
        );
        let v2 = create_test_struct(
            "User",
            vec![
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("name", TypeInfo::Primitive("String".to_string()), false),
            ],
        );

        let diff =
            SchemaDiff::compute(&TypeDefinition::Struct(v1), &TypeDefinition::Struct(v2)).unwrap();

        assert_eq!(diff.changes.len(), 1);
        assert!(matches!(diff.changes[0], SchemaChange::FieldRemoved { .. }));
        assert_eq!(diff.safety(), MigrationSafety::Unsafe);
    }

    #[test]
    fn test_field_type_changed() {
        let v1 = create_test_struct(
            "User",
            vec![
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("age", TypeInfo::Primitive("u16".to_string()), false),
            ],
        );
        let v2 = create_test_struct(
            "User",
            vec![
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("age", TypeInfo::Primitive("u32".to_string()), false),
            ],
        );

        let diff =
            SchemaDiff::compute(&TypeDefinition::Struct(v1), &TypeDefinition::Struct(v2)).unwrap();

        assert_eq!(diff.changes.len(), 1);
        assert!(matches!(
            diff.changes[0],
            SchemaChange::FieldTypeChanged { .. }
        ));
        assert_eq!(diff.safety(), MigrationSafety::Unsafe);
    }

    #[test]
    fn test_field_reordered() {
        let v1 = create_test_struct(
            "User",
            vec![
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("name", TypeInfo::Primitive("String".to_string()), false),
                ("age", TypeInfo::Primitive("u16".to_string()), false),
            ],
        );
        let v2 = create_test_struct(
            "User",
            vec![
                ("name", TypeInfo::Primitive("String".to_string()), false),
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("age", TypeInfo::Primitive("u16".to_string()), false),
            ],
        );

        let diff =
            SchemaDiff::compute(&TypeDefinition::Struct(v1), &TypeDefinition::Struct(v2)).unwrap();

        // Should detect reordering
        assert!(diff
            .changes
            .iter()
            .any(|c| matches!(c, SchemaChange::FieldReordered { .. })));
        // Reordering is safe with Borsh
        assert_eq!(diff.safety(), MigrationSafety::Safe);
    }

    #[test]
    fn test_multiple_changes() {
        let v1 = create_test_struct(
            "User",
            vec![
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("old_field", TypeInfo::Primitive("bool".to_string()), false),
            ],
        );
        let v2 = create_test_struct(
            "User",
            vec![
                ("id", TypeInfo::Primitive("u64".to_string()), false),
                ("new_field", TypeInfo::Primitive("String".to_string()), true),
            ],
        );

        let diff =
            SchemaDiff::compute(&TypeDefinition::Struct(v1), &TypeDefinition::Struct(v2)).unwrap();

        assert_eq!(diff.changes.len(), 2);
        assert!(diff
            .changes
            .iter()
            .any(|c| matches!(c, SchemaChange::FieldAdded { .. })));
        assert!(diff
            .changes
            .iter()
            .any(|c| matches!(c, SchemaChange::FieldRemoved { .. })));
        assert_eq!(diff.safety(), MigrationSafety::Unsafe);
    }
}
