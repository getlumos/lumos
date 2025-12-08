// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Security analyzer for detecting common Solana vulnerabilities
//!
//! Performs static analysis on LUMOS schemas to identify potential security
//! issues before code generation and deployment.

use crate::ir::{StructDefinition, TypeDefinition, TypeInfo};

/// Severity level of a security finding
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational finding - best practice recommendation
    Info,

    /// Warning - potential issue that should be reviewed
    Warning,

    /// Critical - security vulnerability that must be fixed
    Critical,
}

/// Type of vulnerability detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VulnerabilityType {
    /// Missing signer check for authority fields
    MissingSigner,

    /// Integer overflow/underflow risk
    IntegerOverflow,

    /// Missing owner validation
    MissingOwnerValidation,

    /// Account may be uninitialized
    UninitializedAccount,

    /// Re-initialization vulnerability
    ReInitialization,

    /// Unchecked account data
    UncheckedAccountData,

    /// No discriminator (not using Anchor account)
    NoDiscriminator,

    /// Arithmetic-prone fields without checked math
    UncheckedArithmetic,
}

/// A security finding from analysis
#[derive(Debug, Clone)]
pub struct SecurityFinding {
    /// Severity level
    pub severity: Severity,

    /// Type of vulnerability
    pub vulnerability: VulnerabilityType,

    /// Location in schema (type name, field name)
    pub location: Location,

    /// Human-readable message
    pub message: String,

    /// Suggested fix
    pub suggestion: String,
}

/// Location of a finding in the schema
#[derive(Debug, Clone)]
pub struct Location {
    /// Type name
    pub type_name: String,

    /// Field name (if applicable)
    pub field_name: Option<String>,
}

/// Security analyzer
pub struct SecurityAnalyzer<'a> {
    /// All type definitions
    type_defs: &'a [TypeDefinition],

    /// Analysis mode (strict or permissive)
    strict_mode: bool,
}

impl<'a> SecurityAnalyzer<'a> {
    /// Create a new security analyzer
    pub fn new(type_defs: &'a [TypeDefinition]) -> Self {
        Self {
            type_defs,
            strict_mode: false,
        }
    }

    /// Enable strict mode (more aggressive warnings)
    pub fn with_strict_mode(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Analyze all type definitions and return findings
    pub fn analyze(&self) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for type_def in self.type_defs {
            match type_def {
                TypeDefinition::Struct(s) => {
                    findings.extend(self.analyze_struct(s));
                }
                TypeDefinition::Enum(_) => {
                    // Enums have fewer security concerns
                    // Future: Could check for sensitive data in variants
                }
                TypeDefinition::TypeAlias(_) => {
                    // Type aliases don't introduce security concerns (they're just name mappings)
                }
            }
        }

        // Sort by severity (Critical first)
        findings.sort_by(|a, b| b.severity.cmp(&a.severity));

        findings
    }

    /// Analyze a struct for vulnerabilities
    fn analyze_struct(&self, struct_def: &StructDefinition) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        // Check if this is an Anchor account
        let is_account = struct_def
            .metadata
            .attributes
            .contains(&"account".to_string());

        // Check for missing discriminator
        if struct_def.metadata.solana && !is_account {
            findings.push(SecurityFinding {
                severity: Severity::Warning,
                vulnerability: VulnerabilityType::NoDiscriminator,
                location: Location {
                    type_name: struct_def.name.clone(),
                    field_name: None,
                },
                message: format!(
                    "Struct '{}' is marked #[solana] but not #[account] - missing discriminator protection",
                    struct_def.name
                ),
                suggestion: "Add #[account] attribute to enable Anchor discriminator protection against type confusion attacks".to_string(),
            });
        }

        // Analyze each field
        for field in &struct_def.fields {
            // Check for authority/signer fields
            if self.is_authority_field(&field.name) {
                findings.push(SecurityFinding {
                    severity: Severity::Critical,
                    vulnerability: VulnerabilityType::MissingSigner,
                    location: Location {
                        type_name: struct_def.name.clone(),
                        field_name: Some(field.name.clone()),
                    },
                    message: format!(
                        "Field '{}' appears to be an authority but lacks explicit signer validation",
                        field.name
                    ),
                    suggestion: "Ensure this field requires signer validation in your Anchor program. In Anchor, use the Signer<'info> type or add a manual signer check.".to_string(),
                });
            }

            // Check for owner validation
            if field.name == "owner"
                && matches!(field.type_info, TypeInfo::Primitive(ref t) if t == "PublicKey" || t == "Pubkey")
                && self.strict_mode
            {
                findings.push(SecurityFinding {
                    severity: Severity::Warning,
                    vulnerability: VulnerabilityType::MissingOwnerValidation,
                    location: Location {
                        type_name: struct_def.name.clone(),
                        field_name: Some(field.name.clone()),
                    },
                    message: "Owner field requires validation to prevent unauthorized access".to_string(),
                    suggestion: "Validate that msg.sender or transaction signer matches the owner field before state mutations".to_string(),
                });
            }

            // Check for arithmetic-prone fields
            if self.is_arithmetic_field(&field.name, &field.type_info) {
                findings.push(SecurityFinding {
                    severity: Severity::Warning,
                    vulnerability: VulnerabilityType::UncheckedArithmetic,
                    location: Location {
                        type_name: struct_def.name.clone(),
                        field_name: Some(field.name.clone()),
                    },
                    message: format!(
                        "Field '{}' is arithmetic-prone and may overflow/underflow",
                        field.name
                    ),
                    suggestion: "Use checked arithmetic operations (checked_add, checked_sub, checked_mul) to prevent integer overflow/underflow vulnerabilities".to_string(),
                });
            }

            // Check for integer overflow in large numeric types
            if self.is_large_integer(&field.type_info) && self.strict_mode {
                findings.push(SecurityFinding {
                    severity: Severity::Info,
                    vulnerability: VulnerabilityType::IntegerOverflow,
                    location: Location {
                        type_name: struct_def.name.clone(),
                        field_name: Some(field.name.clone()),
                    },
                    message: format!(
                        "Large integer field '{}' - consider overflow protection",
                        field.name
                    ),
                    suggestion: "Ensure arithmetic operations on this field use checked math or saturating operations".to_string(),
                });
            }
        }

        // Check for re-initialization risks
        if is_account && !self.has_initialized_flag(struct_def) && self.strict_mode {
            findings.push(SecurityFinding {
                severity: Severity::Warning,
                vulnerability: VulnerabilityType::ReInitialization,
                location: Location {
                    type_name: struct_def.name.clone(),
                    field_name: None,
                },
                message: "Account lacks explicit initialization flag - vulnerable to re-initialization attacks".to_string(),
                suggestion: "Add an 'is_initialized' boolean field or use Anchor's init constraint to prevent re-initialization".to_string(),
            });
        }

        findings
    }

    /// Check if a field name suggests it's an authority/signer
    fn is_authority_field(&self, field_name: &str) -> bool {
        let authority_keywords = [
            "authority",
            "admin",
            "owner",
            "signer",
            "payer",
            "creator",
            "minter",
            "updater",
        ];

        let lower = field_name.to_lowercase();
        authority_keywords
            .iter()
            .any(|keyword| lower.contains(keyword))
    }

    /// Check if a field is used for arithmetic operations
    fn is_arithmetic_field(&self, field_name: &str, type_info: &TypeInfo) -> bool {
        // Common field names that involve arithmetic
        let arithmetic_keywords = [
            "balance", "amount", "supply", "total", "count", "price", "value", "reward", "stake",
            "fee", "lamport",
        ];

        let lower = field_name.to_lowercase();
        let is_arithmetic_name = arithmetic_keywords
            .iter()
            .any(|keyword| lower.contains(keyword));

        // Must be a numeric type
        let is_numeric = matches!(type_info, TypeInfo::Primitive(ref t) if
            t == "u64" || t == "u128" || t == "i64" || t == "i128" ||
            t == "u32" || t == "i32" || t == "u16" || t == "i16"
        );

        is_arithmetic_name && is_numeric
    }

    /// Check if a type is a large integer (u64, u128, i64, i128)
    fn is_large_integer(&self, type_info: &TypeInfo) -> bool {
        matches!(type_info, TypeInfo::Primitive(ref t) if
            t == "u64" || t == "u128" || t == "i64" || t == "i128"
        )
    }

    /// Check if struct has an initialization flag
    fn has_initialized_flag(&self, struct_def: &StructDefinition) -> bool {
        struct_def.fields.iter().any(|f| {
            let lower = f.name.to_lowercase();
            (lower.contains("initialized") || lower.contains("init"))
                && matches!(f.type_info, TypeInfo::Primitive(ref t) if t == "bool")
        })
    }
}

impl Severity {
    /// Get string representation
    pub fn as_str(&self) -> &str {
        match self {
            Severity::Info => "INFO",
            Severity::Warning => "WARNING",
            Severity::Critical => "CRITICAL",
        }
    }

    /// Get emoji representation
    pub fn emoji(&self) -> &str {
        match self {
            Severity::Info => "ℹ️",
            Severity::Warning => "⚠️",
            Severity::Critical => "🚨",
        }
    }
}

impl VulnerabilityType {
    /// Get string representation
    pub fn as_str(&self) -> &str {
        match self {
            VulnerabilityType::MissingSigner => "Missing Signer Check",
            VulnerabilityType::IntegerOverflow => "Integer Overflow Risk",
            VulnerabilityType::MissingOwnerValidation => "Missing Owner Validation",
            VulnerabilityType::UninitializedAccount => "Uninitialized Account",
            VulnerabilityType::ReInitialization => "Re-initialization Risk",
            VulnerabilityType::UncheckedAccountData => "Unchecked Account Data",
            VulnerabilityType::NoDiscriminator => "No Discriminator",
            VulnerabilityType::UncheckedArithmetic => "Unchecked Arithmetic",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{FieldDefinition, Metadata, Visibility};

    #[test]
    fn test_detects_missing_signer() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "UpdateInstruction".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "authority".to_string(),
                type_info: TypeInfo::Primitive("PublicKey".to_string()),
                optional: false,
                deprecated: None,
                span: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let analyzer = SecurityAnalyzer::new(&type_defs);
        let findings = analyzer.analyze();

        assert!(findings.iter().any(|f| matches!(
            f.vulnerability,
            VulnerabilityType::MissingSigner
        ) && matches!(f.severity, Severity::Critical)));
    }

    #[test]
    fn test_detects_unchecked_arithmetic() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "TokenAccount".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "balance".to_string(),
                type_info: TypeInfo::Primitive("u64".to_string()),
                optional: false,
                deprecated: None,
                span: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let analyzer = SecurityAnalyzer::new(&type_defs);
        let findings = analyzer.analyze();

        assert!(findings
            .iter()
            .any(|f| matches!(f.vulnerability, VulnerabilityType::UncheckedArithmetic)));
    }

    #[test]
    fn test_detects_no_discriminator() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "GameAccount".to_string(),
            generic_params: vec![],
            fields: vec![],
            metadata: Metadata {
                solana: true,
                attributes: vec![], // Missing #[account]
                version: None,
                custom_derives: vec![],
                is_instruction: false,
                anchor_attrs: vec![],
            },
            visibility: Visibility::Public,
            module_path: Vec::new(),
        })];

        let analyzer = SecurityAnalyzer::new(&type_defs);
        let findings = analyzer.analyze();

        assert!(findings
            .iter()
            .any(|f| matches!(f.vulnerability, VulnerabilityType::NoDiscriminator)));
    }

    #[test]
    fn test_strict_mode_more_warnings() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "Account".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "owner".to_string(),
                type_info: TypeInfo::Primitive("PublicKey".to_string()),
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

        // Normal mode
        let analyzer = SecurityAnalyzer::new(&type_defs);
        let normal_findings = analyzer.analyze();

        // Strict mode
        let analyzer_strict = SecurityAnalyzer::new(&type_defs).with_strict_mode();
        let strict_findings = analyzer_strict.analyze();

        assert!(strict_findings.len() >= normal_findings.len());
    }

    #[test]
    fn test_no_false_positives_on_safe_struct() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "SafeData".to_string(),
            generic_params: vec![],
            fields: vec![
                FieldDefinition {
                    name: "id".to_string(),
                    type_info: TypeInfo::Primitive("u32".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "name".to_string(),
                    type_info: TypeInfo::Primitive("String".to_string()),
                    optional: false,
                    deprecated: None,
                    span: None,
                    anchor_attrs: vec![],
                },
            ],
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

        let analyzer = SecurityAnalyzer::new(&type_defs);
        let findings = analyzer.analyze();

        // Should have no critical findings
        assert!(!findings
            .iter()
            .any(|f| matches!(f.severity, Severity::Critical)));
    }
}
