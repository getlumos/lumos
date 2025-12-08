// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Metaplex schema validator
//!
//! Validates LUMOS schemas against Metaplex Token Metadata standards.

use crate::ir::{FieldDefinition, StructDefinition, TypeDefinition, TypeInfo};

use super::types::constraints;

/// Validation error for Metaplex schemas
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Field or struct name
    pub location: String,

    /// Error message
    pub message: String,

    /// Severity level
    pub severity: Severity,

    /// Suggestion for fixing
    pub suggestion: Option<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{}: {}: {}", severity, self.location, self.message)?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, " (suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

/// Severity level for validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Must be fixed - schema is invalid
    Error,

    /// Should be reviewed - may cause issues
    Warning,
}

/// Validation result
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Errors found
    pub errors: Vec<ValidationError>,

    /// Warnings found
    pub warnings: Vec<ValidationError>,
}

impl ValidationResult {
    /// Create empty result
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Add an error
    pub fn add_error(&mut self, location: String, message: String, suggestion: Option<String>) {
        self.errors.push(ValidationError {
            location,
            message,
            severity: Severity::Error,
            suggestion,
        });
    }

    /// Add a warning
    pub fn add_warning(&mut self, location: String, message: String, suggestion: Option<String>) {
        self.warnings.push(ValidationError {
            location,
            message,
            severity: Severity::Warning,
            suggestion,
        });
    }

    /// Merge another result into this one
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

/// Metaplex schema validator
pub struct MetaplexValidator {
    /// Strict mode - treat warnings as errors
    pub strict: bool,
}

impl Default for MetaplexValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaplexValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self { strict: false }
    }

    /// Create validator in strict mode
    pub fn strict() -> Self {
        Self { strict: true }
    }

    /// Validate a LUMOS schema for Metaplex compatibility
    pub fn validate(&self, types: &[TypeDefinition]) -> ValidationResult {
        let mut result = ValidationResult::new();

        for type_def in types {
            if let TypeDefinition::Struct(struct_def) = type_def {
                // Check if this struct has metaplex attributes
                if self.is_metaplex_metadata(struct_def) {
                    result.merge(self.validate_metadata_struct(struct_def));
                } else if self.is_metaplex_creator(struct_def) {
                    result.merge(self.validate_creator_struct(struct_def));
                } else if self.is_metaplex_collection(struct_def) {
                    result.merge(self.validate_collection_struct(struct_def));
                }
            }
        }

        result
    }

    /// Check if struct is a Metaplex metadata type (by name or attributes)
    fn is_metaplex_metadata(&self, struct_def: &StructDefinition) -> bool {
        let name = &struct_def.name;
        name == "Metadata"
            || name == "TokenMetadata"
            || name.ends_with("Metadata")
            || struct_def
                .metadata
                .attributes
                .iter()
                .any(|a| a.contains("metaplex"))
    }

    /// Check if struct is a Creator type
    fn is_metaplex_creator(&self, struct_def: &StructDefinition) -> bool {
        struct_def.name == "Creator"
            || struct_def
                .metadata
                .attributes
                .iter()
                .any(|a| a.contains("metaplex") && a.contains("creator"))
    }

    /// Check if struct is a Collection type
    fn is_metaplex_collection(&self, struct_def: &StructDefinition) -> bool {
        struct_def.name == "Collection"
            || struct_def
                .metadata
                .attributes
                .iter()
                .any(|a| a.contains("metaplex") && a.contains("collection"))
    }

    /// Validate a metadata struct
    fn validate_metadata_struct(&self, struct_def: &StructDefinition) -> ValidationResult {
        let mut result = ValidationResult::new();
        let struct_name = &struct_def.name;

        // Check for required fields
        let required_fields = ["name", "symbol", "uri"];
        for field_name in required_fields {
            if !struct_def.fields.iter().any(|f| f.name == field_name) {
                result.add_warning(
                    struct_name.clone(),
                    format!("Missing recommended field '{}'", field_name),
                    Some(format!("Add '{}: String' field", field_name)),
                );
            }
        }

        // Validate individual fields
        for field in &struct_def.fields {
            result.merge(self.validate_metadata_field(struct_name, field));
        }

        // Check for seller_fee_basis_points
        if let Some(fee_field) = struct_def
            .fields
            .iter()
            .find(|f| f.name == "seller_fee_basis_points")
        {
            if !matches!(&fee_field.type_info, TypeInfo::Primitive(p) if p == "u16") {
                result.add_error(
                    format!("{}.seller_fee_basis_points", struct_name),
                    "seller_fee_basis_points must be u16".to_string(),
                    Some("Change type to u16".to_string()),
                );
            }
        }

        // Check creators field
        if let Some(creators_field) = struct_def.fields.iter().find(|f| f.name == "creators") {
            self.validate_creators_field(struct_name, creators_field, &mut result);
        }

        result
    }

    /// Validate a metadata field
    fn validate_metadata_field(
        &self,
        struct_name: &str,
        field: &FieldDefinition,
    ) -> ValidationResult {
        let mut result = ValidationResult::new();
        let field_name = &field.name;

        match field_name.as_str() {
            "name" => {
                if !matches!(&field.type_info, TypeInfo::Primitive(p) if p == "String") {
                    result.add_error(
                        format!("{}.name", struct_name),
                        "name field must be String type".to_string(),
                        None,
                    );
                }
                // Add warning about max length
                result.add_warning(
                    format!("{}.name", struct_name),
                    format!(
                        "Metaplex limits name to {} characters",
                        constraints::MAX_NAME_LENGTH
                    ),
                    Some("Ensure name length is validated at runtime".to_string()),
                );
            }
            "symbol" => {
                if !matches!(&field.type_info, TypeInfo::Primitive(p) if p == "String") {
                    result.add_error(
                        format!("{}.symbol", struct_name),
                        "symbol field must be String type".to_string(),
                        None,
                    );
                }
                result.add_warning(
                    format!("{}.symbol", struct_name),
                    format!(
                        "Metaplex limits symbol to {} characters",
                        constraints::MAX_SYMBOL_LENGTH
                    ),
                    Some("Ensure symbol length is validated at runtime".to_string()),
                );
            }
            "uri" => {
                if !matches!(&field.type_info, TypeInfo::Primitive(p) if p == "String") {
                    result.add_error(
                        format!("{}.uri", struct_name),
                        "uri field must be String type".to_string(),
                        None,
                    );
                }
                result.add_warning(
                    format!("{}.uri", struct_name),
                    format!(
                        "Metaplex limits URI to {} characters",
                        constraints::MAX_URI_LENGTH
                    ),
                    Some("Ensure URI length is validated at runtime".to_string()),
                );
            }
            _ => {}
        }

        result
    }

    /// Validate creators field
    fn validate_creators_field(
        &self,
        struct_name: &str,
        field: &FieldDefinition,
        result: &mut ValidationResult,
    ) {
        match &field.type_info {
            TypeInfo::Array(inner) | TypeInfo::Option(inner) => {
                if let TypeInfo::Array(elem) = inner.as_ref() {
                    // Vec<Creator> or Option<Vec<Creator>>
                    if !matches!(elem.as_ref(), TypeInfo::UserDefined(name) if name == "Creator") {
                        result.add_warning(
                            format!("{}.creators", struct_name),
                            "creators should be Vec<Creator> or Option<Vec<Creator>>".to_string(),
                            None,
                        );
                    }
                }
            }
            TypeInfo::UserDefined(name) if name.contains("Creator") => {
                // Single Creator - warn about Vec
                result.add_warning(
                    format!("{}.creators", struct_name),
                    "creators is typically a Vec<Creator>".to_string(),
                    Some("Consider using Vec<Creator> instead".to_string()),
                );
            }
            _ => {
                result.add_warning(
                    format!("{}.creators", struct_name),
                    "Unexpected type for creators field".to_string(),
                    Some("Use Vec<Creator> or Option<Vec<Creator>>".to_string()),
                );
            }
        }

        result.add_warning(
            format!("{}.creators", struct_name),
            format!("Metaplex limits to {} creators", constraints::MAX_CREATORS),
            Some("Validate creator count at runtime".to_string()),
        );
    }

    /// Validate a Creator struct
    fn validate_creator_struct(&self, struct_def: &StructDefinition) -> ValidationResult {
        let mut result = ValidationResult::new();
        let struct_name = &struct_def.name;

        // Required fields: address, verified, share
        let required = [
            ("address", "PublicKey"),
            ("verified", "bool"),
            ("share", "u8"),
        ];

        for (field_name, expected_type) in required {
            match struct_def.fields.iter().find(|f| f.name == field_name) {
                Some(field) => {
                    let type_matches = match (&field.type_info, expected_type) {
                        (TypeInfo::Primitive(p), "PublicKey") => p == "PublicKey",
                        (TypeInfo::Primitive(p), "bool") => p == "bool",
                        (TypeInfo::Primitive(p), "u8") => p == "u8",
                        _ => false,
                    };
                    if !type_matches {
                        result.add_error(
                            format!("{}.{}", struct_name, field_name),
                            format!("{} must be {} type", field_name, expected_type),
                            None,
                        );
                    }
                }
                None => {
                    result.add_error(
                        struct_name.clone(),
                        format!("Creator struct missing required field '{}'", field_name),
                        Some(format!("Add '{}: {}' field", field_name, expected_type)),
                    );
                }
            }
        }

        // Warn about share validation
        if struct_def.fields.iter().any(|f| f.name == "share") {
            result.add_warning(
                format!("{}.share", struct_name),
                "Creator shares must sum to 100 across all creators".to_string(),
                Some("Validate share totals at runtime".to_string()),
            );
        }

        result
    }

    /// Validate a Collection struct
    fn validate_collection_struct(&self, struct_def: &StructDefinition) -> ValidationResult {
        let mut result = ValidationResult::new();
        let struct_name = &struct_def.name;

        // Required fields: key, verified
        if !struct_def.fields.iter().any(|f| f.name == "key") {
            result.add_error(
                struct_name.clone(),
                "Collection struct missing required field 'key'".to_string(),
                Some("Add 'key: PublicKey' field".to_string()),
            );
        }

        if !struct_def.fields.iter().any(|f| f.name == "verified") {
            result.add_error(
                struct_name.clone(),
                "Collection struct missing required field 'verified'".to_string(),
                Some("Add 'verified: bool' field".to_string()),
            );
        }

        result
    }
}

/// Generate standard Metaplex type definitions as LUMOS schema
pub fn generate_standard_types() -> String {
    r#"// Standard Metaplex Token Metadata types for LUMOS
// Generated by LUMOS - compatible with mpl-token-metadata

#[solana]
#[metaplex(creator)]
struct Creator {
    address: PublicKey,
    verified: bool,
    share: u8,
}

#[solana]
#[metaplex(collection)]
struct Collection {
    verified: bool,
    key: PublicKey,
}

#[solana]
#[metaplex(uses)]
struct Uses {
    use_method: u8,  // 0 = Burn, 1 = Multiple, 2 = Single
    remaining: u64,
    total: u64,
}

#[solana]
#[metaplex(metadata)]
struct TokenMetadata {
    name: String,           // Max 32 chars
    symbol: String,         // Max 10 chars
    uri: String,            // Max 200 chars
    seller_fee_basis_points: u16,  // 0-10000 (100% = 10000)
    creators: Option<[Creator]>,   // Max 5 creators
}

// Token standards as enum
#[solana]
enum TokenStandard {
    NonFungible,
    FungibleAsset,
    Fungible,
    NonFungibleEdition,
    ProgrammableNonFungible,
    ProgrammableNonFungibleEdition,
}
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Metadata, Visibility};

    fn create_metadata_struct() -> StructDefinition {
        StructDefinition {
            name: "TokenMetadata".to_string(),
            generic_params: vec![],
            fields: vec![
                FieldDefinition {
                    name: "name".to_string(),
                    type_info: TypeInfo::Primitive("String".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "symbol".to_string(),
                    type_info: TypeInfo::Primitive("String".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "uri".to_string(),
                    type_info: TypeInfo::Primitive("String".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "seller_fee_basis_points".to_string(),
                    type_info: TypeInfo::Primitive("u16".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
            ],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: vec![],
        }
    }

    fn create_creator_struct() -> StructDefinition {
        StructDefinition {
            name: "Creator".to_string(),
            generic_params: vec![],
            fields: vec![
                FieldDefinition {
                    name: "address".to_string(),
                    type_info: TypeInfo::Primitive("PublicKey".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "verified".to_string(),
                    type_info: TypeInfo::Primitive("bool".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "share".to_string(),
                    type_info: TypeInfo::Primitive("u8".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
            ],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: vec![],
        }
    }

    #[test]
    fn test_validate_valid_metadata_struct() {
        let validator = MetaplexValidator::new();
        let struct_def = create_metadata_struct();
        let types = vec![TypeDefinition::Struct(struct_def)];

        let result = validator.validate(&types);

        // Should pass with only warnings (length constraints)
        assert!(result.is_valid());
        assert!(result.has_warnings());
    }

    #[test]
    fn test_validate_valid_creator_struct() {
        let validator = MetaplexValidator::new();
        let struct_def = create_creator_struct();
        let types = vec![TypeDefinition::Struct(struct_def)];

        let result = validator.validate(&types);

        // Should pass with warnings about share validation
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_invalid_creator_missing_fields() {
        let validator = MetaplexValidator::new();
        let struct_def = StructDefinition {
            name: "Creator".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "address".to_string(),
                type_info: TypeInfo::Primitive("PublicKey".to_string()),
                optional: false,
                deprecated: None,
                span: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: vec![],
        };
        let types = vec![TypeDefinition::Struct(struct_def)];

        let result = validator.validate(&types);

        // Should fail - missing verified and share
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_validate_wrong_seller_fee_type() {
        let validator = MetaplexValidator::new();
        let mut struct_def = create_metadata_struct();
        // Change seller_fee_basis_points to wrong type
        struct_def.fields[3].type_info = TypeInfo::Primitive("u64".to_string());

        let types = vec![TypeDefinition::Struct(struct_def)];
        let result = validator.validate(&types);

        // Should fail
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("seller_fee_basis_points must be u16")));
    }

    #[test]
    fn test_generate_standard_types() {
        let types = generate_standard_types();

        assert!(types.contains("struct Creator"));
        assert!(types.contains("struct Collection"));
        assert!(types.contains("struct TokenMetadata"));
        assert!(types.contains("enum TokenStandard"));
        assert!(types.contains("seller_fee_basis_points: u16"));
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::new();
        result1.add_error("loc1".to_string(), "error1".to_string(), None);

        let mut result2 = ValidationResult::new();
        result2.add_warning("loc2".to_string(), "warning1".to_string(), None);

        result1.merge(result2);

        assert_eq!(result1.errors.len(), 1);
        assert_eq!(result1.warnings.len(), 1);
    }
}
