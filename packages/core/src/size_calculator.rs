// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Account size calculator for Solana programs
//!
//! Calculates the exact byte size of account data structures based on
//! Borsh serialization format.

use crate::ir::{
    EnumDefinition, EnumVariantDefinition, StructDefinition, TypeDefinition, TypeInfo,
};
use std::collections::HashMap;

/// Result of size calculation for an account
#[derive(Debug, Clone)]
pub struct AccountSize {
    /// Account name
    pub name: String,

    /// Total size in bytes
    pub total_bytes: SizeInfo,

    /// Breakdown of size by field
    pub field_breakdown: Vec<FieldSize>,

    /// Whether this has #[account] attribute
    pub is_account: bool,

    /// Estimated rent in SOL (lamports / 1e9)
    pub rent_sol: f64,

    /// Warnings about size
    pub warnings: Vec<String>,
}

/// Size information that can be fixed or variable
#[derive(Debug, Clone)]
pub enum SizeInfo {
    /// Fixed size in bytes
    Fixed(usize),

    /// Variable size with minimum bytes
    Variable { min: usize, reason: String },
}

/// Field size breakdown
#[derive(Debug, Clone)]
pub struct FieldSize {
    /// Field name
    pub name: String,

    /// Field size
    pub size: SizeInfo,

    /// Description
    pub description: String,
}

/// Size calculator
pub struct SizeCalculator<'a> {
    /// All type definitions for resolving user-defined types
    type_defs: &'a [TypeDefinition],

    /// Cache of calculated sizes for user-defined types
    size_cache: HashMap<String, SizeInfo>,
}

impl<'a> SizeCalculator<'a> {
    /// Create a new size calculator
    pub fn new(type_defs: &'a [TypeDefinition]) -> Self {
        Self {
            type_defs,
            size_cache: HashMap::new(),
        }
    }

    /// Calculate sizes for all accounts
    pub fn calculate_all(&mut self) -> Vec<AccountSize> {
        self.type_defs
            .iter()
            .filter_map(|type_def| match type_def {
                TypeDefinition::Struct(s) => Some(self.calculate_struct_size(s)),
                TypeDefinition::Enum(e) => Some(self.calculate_enum_size(e)),
                TypeDefinition::TypeAlias(_) => None, // Type aliases don't have sizes (they're resolved)
            })
            .collect()
    }

    /// Calculate size for a struct
    fn calculate_struct_size(&mut self, struct_def: &StructDefinition) -> AccountSize {
        let mut field_breakdown = Vec::new();
        let mut total_size = 0;
        let mut is_variable = false;
        let mut variable_reason = String::new();
        let mut warnings = Vec::new();

        // Add discriminator for Anchor accounts
        let is_account = struct_def
            .metadata
            .attributes
            .contains(&"account".to_string());
        if is_account {
            field_breakdown.push(FieldSize {
                name: "discriminator".to_string(),
                size: SizeInfo::Fixed(8),
                description: "Anchor account discriminator".to_string(),
            });
            total_size += 8;
        }

        // Calculate size for each field
        for field in &struct_def.fields {
            let size = self.calculate_type_size(&field.type_info);
            let description = self.describe_type(&field.type_info);

            match &size {
                SizeInfo::Fixed(bytes) => {
                    total_size += bytes;
                }
                SizeInfo::Variable { min, reason } => {
                    total_size += min;
                    is_variable = true;
                    if !variable_reason.is_empty() {
                        variable_reason.push_str(", ");
                    }
                    variable_reason.push_str(&format!("{}: {}", field.name, reason));
                }
            }

            field_breakdown.push(FieldSize {
                name: field.name.clone(),
                size,
                description,
            });
        }

        // Calculate rent (using Solana rent formula: ~0.00000348 SOL per byte per year)
        // Minimum rent-exempt balance = (size + 128) * 6.96 lamports/byte
        let rent_lamports = (total_size + 128) as f64 * 6.96;
        let rent_sol = rent_lamports / 1_000_000_000.0;

        // Generate warnings
        const MAX_ACCOUNT_SIZE: usize = 10 * 1024 * 1024; // 10MB Solana limit
        const WARNING_THRESHOLD: usize = 1024 * 1024; // Warn at 1MB

        if total_size > MAX_ACCOUNT_SIZE {
            warnings.push(format!(
                "Account exceeds Solana's 10MB limit ({:.2} MB). Consider splitting into multiple accounts.",
                total_size as f64 / (1024.0 * 1024.0)
            ));
        } else if total_size > WARNING_THRESHOLD {
            warnings.push(format!(
                "Large account size ({:.2} KB). Consider optimization.",
                total_size as f64 / 1024.0
            ));
        }

        let total_bytes = if is_variable {
            SizeInfo::Variable {
                min: total_size,
                reason: variable_reason,
            }
        } else {
            SizeInfo::Fixed(total_size)
        };

        AccountSize {
            name: struct_def.name.clone(),
            total_bytes,
            field_breakdown,
            is_account,
            rent_sol,
            warnings,
        }
    }

    /// Calculate size for an enum
    fn calculate_enum_size(&mut self, enum_def: &EnumDefinition) -> AccountSize {
        let mut field_breakdown = Vec::new();
        let mut max_variant_size = 0;
        let mut warnings = Vec::new();

        // Borsh enum discriminant (1 byte for < 256 variants, 4 bytes for >= 256)
        let discriminant_size = if enum_def.variants.len() < 256 { 1 } else { 4 };

        field_breakdown.push(FieldSize {
            name: "discriminant".to_string(),
            size: SizeInfo::Fixed(discriminant_size),
            description: "Enum variant discriminant".to_string(),
        });

        // Calculate size for each variant
        for variant in &enum_def.variants {
            let variant_size = match variant {
                EnumVariantDefinition::Unit { name } => {
                    field_breakdown.push(FieldSize {
                        name: format!("  └─ {}", name),
                        size: SizeInfo::Fixed(0),
                        description: "Unit variant (no data)".to_string(),
                    });
                    0
                }
                EnumVariantDefinition::Tuple { name, types } => {
                    let mut tuple_size = 0;
                    for (i, type_info) in types.iter().enumerate() {
                        let size = self.calculate_type_size(type_info);
                        if let SizeInfo::Fixed(bytes) = size {
                            tuple_size += bytes;
                        }
                        field_breakdown.push(FieldSize {
                            name: format!("  └─ {}.{}", name, i),
                            size,
                            description: self.describe_type(type_info),
                        });
                    }
                    tuple_size
                }
                EnumVariantDefinition::Struct { name, fields } => {
                    let mut struct_size = 0;
                    for field in fields {
                        let size = self.calculate_type_size(&field.type_info);
                        if let SizeInfo::Fixed(bytes) = size {
                            struct_size += bytes;
                        }
                        field_breakdown.push(FieldSize {
                            name: format!("  └─ {}.{}", name, field.name),
                            size,
                            description: self.describe_type(&field.type_info),
                        });
                    }
                    struct_size
                }
            };

            max_variant_size = max_variant_size.max(variant_size);
        }

        let total_size = discriminant_size + max_variant_size;

        // Calculate rent
        let rent_lamports = (total_size + 128) as f64 * 6.96;
        let rent_sol = rent_lamports / 1_000_000_000.0;

        // Warnings
        if total_size > 10 * 1024 * 1024 {
            warnings.push(format!(
                "Enum exceeds Solana's 10MB limit ({:.2} MB)",
                total_size as f64 / (1024.0 * 1024.0)
            ));
        }

        AccountSize {
            name: enum_def.name.clone(),
            total_bytes: SizeInfo::Fixed(total_size),
            field_breakdown,
            is_account: false,
            rent_sol,
            warnings,
        }
    }

    /// Calculate size for a type
    fn calculate_type_size(&mut self, type_info: &TypeInfo) -> SizeInfo {
        match type_info {
            TypeInfo::Primitive(type_name) => self.calculate_primitive_size(type_name),
            TypeInfo::Generic(param_name) => {
                // Generic parameters have unknown size until concrete type is provided
                SizeInfo::Variable {
                    min: 0,
                    reason: format!(
                        "Generic parameter '{}' (size depends on concrete type)",
                        param_name
                    ),
                }
            }
            TypeInfo::UserDefined(type_name) => {
                // Check cache first
                if let Some(cached) = self.size_cache.get(type_name) {
                    return cached.clone();
                }

                // Find type definition and calculate
                if let Some(type_def) = self.type_defs.iter().find(|t| t.name() == type_name) {
                    let size = match type_def {
                        TypeDefinition::Struct(s) => {
                            let account_size = self.calculate_struct_size(s);
                            account_size.total_bytes
                        }
                        TypeDefinition::Enum(e) => {
                            let account_size = self.calculate_enum_size(e);
                            account_size.total_bytes
                        }
                        TypeDefinition::TypeAlias(a) => {
                            // Type aliases are resolved - calculate the target type size
                            self.calculate_type_size(&a.target)
                        }
                    };
                    self.size_cache.insert(type_name.clone(), size.clone());
                    size
                } else {
                    // Unknown user-defined type, assume reasonable size
                    SizeInfo::Variable {
                        min: 0,
                        reason: format!("Unknown type '{}'", type_name),
                    }
                }
            }
            TypeInfo::Array(inner) => {
                // Vec<T> = 4 bytes (length) + variable data
                SizeInfo::Variable {
                    min: 4,
                    reason: format!(
                        "Vec length prefix + elements ({})",
                        self.describe_type(inner)
                    ),
                }
            }
            TypeInfo::FixedArray { element, size } => {
                // Fixed array [T; N] = element_size * N (no length prefix!)
                let element_size = self.calculate_type_size(element);
                match element_size {
                    SizeInfo::Fixed(bytes) => SizeInfo::Fixed(bytes * size),
                    SizeInfo::Variable { min, reason } => SizeInfo::Variable {
                        min: min * size,
                        reason: format!(
                            "Fixed array[{}] with variable-sized elements ({})",
                            size, reason
                        ),
                    },
                }
            }
            TypeInfo::Option(inner) => {
                // Option<T> = 1 byte (discriminant) + T
                let inner_size = self.calculate_type_size(inner);
                match inner_size {
                    SizeInfo::Fixed(bytes) => SizeInfo::Fixed(1 + bytes),
                    SizeInfo::Variable { min, reason } => SizeInfo::Variable {
                        min: 1 + min,
                        reason,
                    },
                }
            }
        }
    }

    /// Calculate size for a primitive type
    fn calculate_primitive_size(&self, type_name: &str) -> SizeInfo {
        match type_name {
            // Integer types
            "u8" | "i8" | "bool" => SizeInfo::Fixed(1),
            "u16" | "i16" => SizeInfo::Fixed(2),
            "u32" | "i32" | "f32" => SizeInfo::Fixed(4),
            "u64" | "i64" | "f64" => SizeInfo::Fixed(8),
            "u128" | "i128" => SizeInfo::Fixed(16),

            // Solana types
            "Pubkey" | "PublicKey" => SizeInfo::Fixed(32),
            "Signature" => SizeInfo::Fixed(64),

            // String is variable length
            "String" => SizeInfo::Variable {
                min: 4,
                reason: "String length prefix + UTF-8 bytes".to_string(),
            },

            // Unknown
            _ => SizeInfo::Variable {
                min: 0,
                reason: format!("Unknown primitive type '{}'", type_name),
            },
        }
    }

    /// Describe a type for display
    #[allow(clippy::only_used_in_recursion)]
    fn describe_type(&self, type_info: &TypeInfo) -> String {
        match type_info {
            TypeInfo::Primitive(name) => match name.as_str() {
                "Pubkey" | "PublicKey" => "PublicKey (32 bytes)".to_string(),
                "Signature" => "Signature (64 bytes)".to_string(),
                "String" => "String (variable)".to_string(),
                _ => name.clone(),
            },
            TypeInfo::Generic(param_name) => param_name.clone(),
            TypeInfo::UserDefined(name) => name.clone(),
            TypeInfo::Array(inner) => format!("Vec<{}>", self.describe_type(inner)),
            TypeInfo::FixedArray { element, size } => {
                format!("[{}; {}]", self.describe_type(element), size)
            }
            TypeInfo::Option(inner) => format!("Option<{}>", self.describe_type(inner)),
        }
    }
}

impl SizeInfo {
    /// Get the minimum size in bytes
    pub fn min_bytes(&self) -> usize {
        match self {
            SizeInfo::Fixed(bytes) => *bytes,
            SizeInfo::Variable { min, .. } => *min,
        }
    }

    /// Check if this is a fixed size
    pub fn is_fixed(&self) -> bool {
        matches!(self, SizeInfo::Fixed(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{FieldDefinition, Metadata, StructDefinition, Visibility};

    #[test]
    fn test_primitive_sizes() {
        let type_defs = vec![];
        let calc = SizeCalculator::new(&type_defs);

        assert_eq!(calc.calculate_primitive_size("u8").min_bytes(), 1);
        assert_eq!(calc.calculate_primitive_size("u16").min_bytes(), 2);
        assert_eq!(calc.calculate_primitive_size("u32").min_bytes(), 4);
        assert_eq!(calc.calculate_primitive_size("u64").min_bytes(), 8);
        assert_eq!(calc.calculate_primitive_size("u128").min_bytes(), 16);
        assert_eq!(calc.calculate_primitive_size("PublicKey").min_bytes(), 32);
        assert_eq!(calc.calculate_primitive_size("Signature").min_bytes(), 64);
    }

    #[test]
    fn test_simple_struct_size() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "Player".to_string(),
            generic_params: vec![],
            fields: vec![
                FieldDefinition {
                    name: "wallet".to_string(),
                    type_info: TypeInfo::Primitive("PublicKey".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "score".to_string(),
                    type_info: TypeInfo::Primitive("u64".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
            ],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let mut calc = SizeCalculator::new(&type_defs);
        let sizes = calc.calculate_all();

        assert_eq!(sizes.len(), 1);
        assert_eq!(sizes[0].name, "Player");
        assert_eq!(sizes[0].total_bytes.min_bytes(), 32 + 8); // PublicKey + u64
    }

    #[test]
    fn test_account_with_discriminator() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "GameAccount".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "score".to_string(),
                type_info: TypeInfo::Primitive("u64".to_string()),
                optional: false,
                deprecated: None,
                span: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata {
                solana: true,
                attributes: vec!["account".to_string()],
                version: None,
                custom_derives: vec![],
                is_instruction: false,
                anchor_attrs: vec![],
            },
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let mut calc = SizeCalculator::new(&type_defs);
        let sizes = calc.calculate_all();

        assert_eq!(sizes.len(), 1);
        assert_eq!(sizes[0].total_bytes.min_bytes(), 8 + 8); // discriminator + u64
        assert!(sizes[0].is_account);
    }

    #[test]
    fn test_option_size() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "Optional".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "maybe_value".to_string(),
                type_info: TypeInfo::Option(Box::new(TypeInfo::Primitive("u64".to_string()))),
                optional: true,
                deprecated: None,
                span: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let mut calc = SizeCalculator::new(&type_defs);
        let sizes = calc.calculate_all();

        assert_eq!(sizes.len(), 1);
        assert_eq!(sizes[0].total_bytes.min_bytes(), 1 + 8); // discriminant + u64
    }
}
