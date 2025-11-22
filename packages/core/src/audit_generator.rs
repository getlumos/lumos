// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Security audit checklist generator
//!
//! Generates comprehensive security audit checklists from LUMOS schemas
//! for manual code review and security audits.

use crate::ir::{StructDefinition, TypeDefinition, TypeInfo};

/// A single checklist item
#[derive(Debug, Clone)]
pub struct ChecklistItem {
    /// Category of the check
    pub category: CheckCategory,

    /// Priority level
    pub priority: Priority,

    /// The actual checklist item text
    pub item: String,

    /// Context (which account/field this applies to)
    pub context: String,

    /// Detailed explanation
    pub explanation: String,
}

/// Category of security check
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CheckCategory {
    AccountValidation,
    SignerChecks,
    ArithmeticSafety,
    AccessControl,
    StateTransition,
    DataValidation,
    RentExemption,
    Initialization,
}

/// Priority level for checklist items
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Audit checklist generator
pub struct AuditGenerator<'a> {
    /// All type definitions
    type_defs: &'a [TypeDefinition],
}

impl<'a> AuditGenerator<'a> {
    /// Create a new audit generator
    pub fn new(type_defs: &'a [TypeDefinition]) -> Self {
        Self { type_defs }
    }

    /// Generate complete audit checklist
    pub fn generate(&self) -> Vec<ChecklistItem> {
        let mut items = Vec::new();

        for type_def in self.type_defs {
            match type_def {
                TypeDefinition::Struct(s) => {
                    items.extend(self.generate_struct_checks(s));
                }
                TypeDefinition::Enum(_) => {
                    // Enums have fewer security concerns
                }
            }
        }

        // Sort by priority (Critical first)
        items.sort_by(|a, b| a.priority.cmp(&b.priority));

        items
    }

    /// Generate checklist items for a struct
    fn generate_struct_checks(&self, struct_def: &StructDefinition) -> Vec<ChecklistItem> {
        let mut items = Vec::new();

        let is_account = struct_def.metadata.attributes.contains(&"account".to_string());

        // Account validation checks
        if is_account {
            items.push(ChecklistItem {
                category: CheckCategory::AccountValidation,
                priority: Priority::Critical,
                item: "Verify account ownership (program owns the account)".to_string(),
                context: struct_def.name.clone(),
                explanation: "Ensure the account is owned by the program to prevent attacks where an attacker passes an account owned by a different program.".to_string(),
            });

            items.push(ChecklistItem {
                category: CheckCategory::AccountValidation,
                priority: Priority::Critical,
                item: "Validate account discriminator".to_string(),
                context: struct_def.name.clone(),
                explanation: "Anchor's 8-byte discriminator prevents type confusion attacks. Verify it's checked on deserialization.".to_string(),
            });

            items.push(ChecklistItem {
                category: CheckCategory::Initialization,
                priority: Priority::High,
                item: "Check account is initialized before use".to_string(),
                context: struct_def.name.clone(),
                explanation: "Verify the account has been properly initialized and is not in an uninitialized state.".to_string(),
            });

            items.push(ChecklistItem {
                category: CheckCategory::RentExemption,
                priority: Priority::Medium,
                item: "Verify account has sufficient lamports for rent exemption".to_string(),
                context: struct_def.name.clone(),
                explanation: "Ensure the account has enough lamports to remain rent-exempt and won't be garbage collected.".to_string(),
            });
        }

        // Field-specific checks
        for field in &struct_def.fields {
            // Signer checks for authority fields
            if self.is_authority_field(&field.name) {
                items.push(ChecklistItem {
                    category: CheckCategory::SignerChecks,
                    priority: Priority::Critical,
                    item: format!("Verify '{}' field requires signer", field.name),
                    context: format!("{}::{}", struct_def.name, field.name),
                    explanation: "Authority fields must validate that the transaction is signed by the corresponding private key.".to_string(),
                });

                items.push(ChecklistItem {
                    category: CheckCategory::AccessControl,
                    priority: Priority::Critical,
                    item: format!("Ensure only '{}' can perform privileged operations", field.name),
                    context: format!("{}::{}", struct_def.name, field.name),
                    explanation: "Implement proper access control checks to prevent unauthorized users from executing privileged functions.".to_string(),
                });
            }

            // Arithmetic safety for numeric fields
            if self.is_arithmetic_field(&field.name, &field.type_info) {
                items.push(ChecklistItem {
                    category: CheckCategory::ArithmeticSafety,
                    priority: Priority::High,
                    item: format!("Verify '{}' uses checked arithmetic operations", field.name),
                    context: format!("{}::{}", struct_def.name, field.name),
                    explanation: "Use checked_add, checked_sub, checked_mul to prevent integer overflow/underflow vulnerabilities that could lead to loss of funds.".to_string(),
                });

                items.push(ChecklistItem {
                    category: CheckCategory::DataValidation,
                    priority: Priority::Medium,
                    item: format!("Validate '{}' bounds and constraints", field.name),
                    context: format!("{}::{}", struct_def.name, field.name),
                    explanation: "Ensure the value is within acceptable ranges and meets business logic constraints.".to_string(),
                });
            }

            // Owner validation
            if field.name == "owner" {
                items.push(ChecklistItem {
                    category: CheckCategory::AccessControl,
                    priority: Priority::Critical,
                    item: "Validate owner matches transaction signer for mutations".to_string(),
                    context: format!("{}::{}", struct_def.name, field.name),
                    explanation: "Before modifying account state, verify that the signer is the owner or has proper authorization.".to_string(),
                });
            }

            // PublicKey validation
            if matches!(field.type_info, TypeInfo::Primitive(ref t) if t == "PublicKey" || t == "Pubkey") {
                items.push(ChecklistItem {
                    category: CheckCategory::DataValidation,
                    priority: Priority::Medium,
                    item: format!("Verify '{}' is not system program or default pubkey", field.name),
                    context: format!("{}::{}", struct_def.name, field.name),
                    explanation: "Ensure PublicKey fields are not set to default values (all zeros) or system program addresses unless intentional.".to_string(),
                });
            }

            // Vec/Array bounds checking
            if matches!(field.type_info, TypeInfo::Array(_)) {
                items.push(ChecklistItem {
                    category: CheckCategory::DataValidation,
                    priority: Priority::High,
                    item: format!("Validate '{}' length before iteration", field.name),
                    context: format!("{}::{}", struct_def.name, field.name),
                    explanation: "Check vector/array length to prevent excessive compute usage or out-of-bounds access.".to_string(),
                });

                items.push(ChecklistItem {
                    category: CheckCategory::ArithmeticSafety,
                    priority: Priority::Medium,
                    item: format!("Ensure '{}' max size doesn't exceed account limits", field.name),
                    context: format!("{}::{}", struct_def.name, field.name),
                    explanation: "Verify that the maximum possible size of this vector won't cause the account to exceed Solana's 10MB limit.".to_string(),
                });
            }

            // Option type handling
            if matches!(field.type_info, TypeInfo::Option(_)) {
                items.push(ChecklistItem {
                    category: CheckCategory::DataValidation,
                    priority: Priority::Medium,
                    item: format!("Handle None case for optional '{}' field", field.name),
                    context: format!("{}::{}", struct_def.name, field.name),
                    explanation: "Ensure program logic properly handles the case when this optional field is None.".to_string(),
                });
            }
        }

        // State transition checks
        if is_account {
            items.push(ChecklistItem {
                category: CheckCategory::StateTransition,
                priority: Priority::High,
                item: "Verify state transitions are valid and atomic".to_string(),
                context: struct_def.name.clone(),
                explanation: "Ensure state changes follow expected patterns and can't leave the account in an inconsistent state.".to_string(),
            });

            items.push(ChecklistItem {
                category: CheckCategory::StateTransition,
                priority: Priority::Medium,
                item: "Check for reentrancy vulnerabilities".to_string(),
                context: struct_def.name.clone(),
                explanation: "If the program makes cross-program invocations, ensure it can't be re-entered in an unsafe state.".to_string(),
            });
        }

        items
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

        // Check for exact matches or as complete words (prefix/suffix with underscore)
        authority_keywords.iter().any(|keyword| {
            // Exact match
            if lower == *keyword {
                return true;
            }

            // Match as prefix (e.g., "owner_id", "admin_key")
            if lower.starts_with(&format!("{}_", keyword)) {
                return true;
            }

            // Match as suffix (e.g., "pool_owner", "vault_authority")
            if lower.ends_with(&format!("_{}", keyword)) {
                return true;
            }

            // Match in middle (e.g., "multi_owner_account")
            if lower.contains(&format!("_{}_", keyword)) {
                return true;
            }

            false
        })
    }

    /// Check if a field is used for arithmetic operations
    fn is_arithmetic_field(&self, field_name: &str, type_info: &TypeInfo) -> bool {
        let arithmetic_keywords = [
            "balance",
            "amount",
            "supply",
            "total",
            "count",
            "price",
            "value",
            "reward",
            "stake",
            "fee",
            "lamport",
        ];

        let lower = field_name.to_lowercase();
        let is_arithmetic_name = arithmetic_keywords.iter().any(|keyword| lower.contains(keyword));

        let is_numeric = matches!(type_info, TypeInfo::Primitive(ref t) if
            t == "u64" || t == "u128" || t == "i64" || t == "i128" ||
            t == "u32" || t == "i32" || t == "u16" || t == "i16"
        );

        is_arithmetic_name && is_numeric
    }
}

impl CheckCategory {
    /// Get string representation
    pub fn as_str(&self) -> &str {
        match self {
            CheckCategory::AccountValidation => "Account Validation",
            CheckCategory::SignerChecks => "Signer Checks",
            CheckCategory::ArithmeticSafety => "Arithmetic Safety",
            CheckCategory::AccessControl => "Access Control",
            CheckCategory::StateTransition => "State Transition",
            CheckCategory::DataValidation => "Data Validation",
            CheckCategory::RentExemption => "Rent Exemption",
            CheckCategory::Initialization => "Initialization",
        }
    }

    /// Get emoji representation
    pub fn emoji(&self) -> &str {
        match self {
            CheckCategory::AccountValidation => "🔐",
            CheckCategory::SignerChecks => "✍️",
            CheckCategory::ArithmeticSafety => "🔢",
            CheckCategory::AccessControl => "🚪",
            CheckCategory::StateTransition => "🔄",
            CheckCategory::DataValidation => "✅",
            CheckCategory::RentExemption => "💰",
            CheckCategory::Initialization => "🎬",
        }
    }
}

impl Priority {
    /// Get string representation
    pub fn as_str(&self) -> &str {
        match self {
            Priority::Critical => "CRITICAL",
            Priority::High => "HIGH",
            Priority::Medium => "MEDIUM",
            Priority::Low => "LOW",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{FieldDefinition, Metadata};

    #[test]
    fn test_generates_account_validation_checks() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "PlayerAccount".to_string(),
            fields: vec![],
            metadata: Metadata {
                solana: true,
                attributes: vec!["account".to_string()],
            },
        })];

        let generator = AuditGenerator::new(&type_defs);
        let checklist = generator.generate();

        assert!(checklist.iter().any(|item|
            matches!(item.category, CheckCategory::AccountValidation) &&
            item.item.contains("ownership")
        ));
    }

    #[test]
    fn test_generates_signer_checks_for_authority() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "Config".to_string(),
            fields: vec![FieldDefinition {
                name: "authority".to_string(),
                type_info: TypeInfo::Primitive("PublicKey".to_string()),
                optional: false,
            }],
            metadata: Metadata {
                solana: true,
                attributes: vec!["account".to_string()],
            },
        })];

        let generator = AuditGenerator::new(&type_defs);
        let checklist = generator.generate();

        assert!(checklist.iter().any(|item|
            matches!(item.category, CheckCategory::SignerChecks) &&
            matches!(item.priority, Priority::Critical)
        ));
    }

    #[test]
    fn test_generates_arithmetic_checks() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "Vault".to_string(),
            fields: vec![FieldDefinition {
                name: "balance".to_string(),
                type_info: TypeInfo::Primitive("u64".to_string()),
                optional: false,
            }],
            metadata: Metadata::default(),
        })];

        let generator = AuditGenerator::new(&type_defs);
        let checklist = generator.generate();

        assert!(checklist.iter().any(|item|
            matches!(item.category, CheckCategory::ArithmeticSafety) &&
            item.item.contains("checked arithmetic")
        ));
    }

    #[test]
    fn test_sorted_by_priority() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "TokenAccount".to_string(),
            fields: vec![
                FieldDefinition {
                    name: "authority".to_string(),
                    type_info: TypeInfo::Primitive("PublicKey".to_string()),
                    optional: false,
                },
                FieldDefinition {
                    name: "balance".to_string(),
                    type_info: TypeInfo::Primitive("u64".to_string()),
                    optional: false,
                },
            ],
            metadata: Metadata {
                solana: true,
                attributes: vec!["account".to_string()],
            },
        })];

        let generator = AuditGenerator::new(&type_defs);
        let checklist = generator.generate();

        // Verify sorted by priority
        for i in 1..checklist.len() {
            assert!(checklist[i - 1].priority <= checklist[i].priority);
        }
    }
}
