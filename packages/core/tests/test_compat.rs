// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Tests for schema compatibility checking

use lumos_core::compat::{CompatibilityChecker, IssueLevel};
use lumos_core::ir::{FieldDefinition, Metadata, StructDefinition, TypeDefinition, TypeInfo};

/// Helper to create a simple struct type definition
fn create_struct(
    name: &str,
    fields: Vec<(&str, TypeInfo, bool)>,
    version: Option<&str>,
) -> TypeDefinition {
    let field_defs: Vec<FieldDefinition> = fields
        .into_iter()
        .map(|(name, type_info, optional)| FieldDefinition {
            name: name.to_string(),
            type_info,
            optional,
            deprecated: None,
            anchor_attrs: vec![],
        })
        .collect();

    TypeDefinition::Struct(StructDefinition {
        name: name.to_string(),
        generic_params: vec![],
        fields: field_defs,
        metadata: Metadata {
            solana: true,
            attributes: vec!["account".to_string()],
            version: version.map(|s| s.to_string()),
            custom_derives: vec![],
            is_instruction: false,
            anchor_attrs: vec![],
        },
    })
}

#[test]
fn test_adding_optional_field_is_compatible() {
    let old = create_struct(
        "Player",
        vec![(
            "wallet",
            TypeInfo::Primitive("PublicKey".to_string()),
            false,
        )],
        Some("1.0.0"),
    );

    let new = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("level", TypeInfo::Primitive("u16".to_string()), true),
        ],
        Some("1.1.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(report.is_compatible);
    assert_eq!(report.count_by_level(IssueLevel::Breaking), 0);
    assert_eq!(report.count_by_level(IssueLevel::Info), 1);
}

#[test]
fn test_adding_required_field_is_breaking() {
    let old = create_struct(
        "Player",
        vec![(
            "wallet",
            TypeInfo::Primitive("PublicKey".to_string()),
            false,
        )],
        Some("1.0.0"),
    );

    let new = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("balance", TypeInfo::Primitive("u64".to_string()), false),
        ],
        Some("2.0.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(!report.is_compatible);
    assert_eq!(report.count_by_level(IssueLevel::Breaking), 1);

    let breaking_issues = report.breaking_issues();
    assert!(breaking_issues[0]
        .message
        .contains("Added required field: balance"));
}

#[test]
fn test_removing_field_is_breaking() {
    let old = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("level", TypeInfo::Primitive("u16".to_string()), false),
        ],
        Some("1.0.0"),
    );

    let new = create_struct(
        "Player",
        vec![(
            "wallet",
            TypeInfo::Primitive("PublicKey".to_string()),
            false,
        )],
        Some("2.0.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(!report.is_compatible);
    assert_eq!(report.count_by_level(IssueLevel::Breaking), 1);

    let breaking_issues = report.breaking_issues();
    assert!(breaking_issues[0].message.contains("Removed field: level"));
}

#[test]
fn test_changing_field_type_is_breaking() {
    let old = create_struct(
        "Stats",
        vec![("count", TypeInfo::Primitive("u32".to_string()), false)],
        Some("1.0.0"),
    );

    let new = create_struct(
        "Stats",
        vec![("count", TypeInfo::Primitive("u16".to_string()), false)],
        Some("2.0.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(!report.is_compatible);
    assert_eq!(report.count_by_level(IssueLevel::Breaking), 1);

    let breaking_issues = report.breaking_issues();
    assert!(breaking_issues[0]
        .message
        .contains("Changed type of 'count'"));
}

#[test]
fn test_reordering_fields_is_safe() {
    let old = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("level", TypeInfo::Primitive("u16".to_string()), false),
            ("score", TypeInfo::Primitive("u64".to_string()), false),
        ],
        Some("1.0.0"),
    );

    let new = create_struct(
        "Player",
        vec![
            ("level", TypeInfo::Primitive("u16".to_string()), false),
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("score", TypeInfo::Primitive("u64".to_string()), false),
        ],
        Some("1.0.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(report.is_compatible);
    assert_eq!(report.count_by_level(IssueLevel::Breaking), 0);
    // Reordering shows up as info
    assert!(report.count_by_level(IssueLevel::Info) > 0);
}

#[test]
fn test_version_bump_validation_breaking_changes() {
    let old = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("level", TypeInfo::Primitive("u16".to_string()), false),
        ],
        Some("1.0.0"),
    );

    // Removing field but only bumping minor version (should fail)
    let new = create_struct(
        "Player",
        vec![(
            "wallet",
            TypeInfo::Primitive("PublicKey".to_string()),
            false,
        )],
        Some("1.1.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(!report.is_compatible);
    assert!(!report.version_bump_valid); // Version bump is invalid
}

#[test]
fn test_version_bump_validation_valid_major_bump() {
    let old = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("level", TypeInfo::Primitive("u16".to_string()), false),
        ],
        Some("1.0.0"),
    );

    // Removing field with major version bump (valid)
    let new = create_struct(
        "Player",
        vec![(
            "wallet",
            TypeInfo::Primitive("PublicKey".to_string()),
            false,
        )],
        Some("2.0.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(!report.is_compatible); // Still not compatible
    assert!(report.version_bump_valid); // But version bump is appropriate
}

#[test]
fn test_version_bump_validation_minor_bump_for_compatible_changes() {
    let old = create_struct(
        "Player",
        vec![(
            "wallet",
            TypeInfo::Primitive("PublicKey".to_string()),
            false,
        )],
        Some("1.0.0"),
    );

    // Adding optional field with minor version bump (valid)
    let new = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("email", TypeInfo::Primitive("String".to_string()), true),
        ],
        Some("1.1.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(report.is_compatible);
    assert!(report.version_bump_valid);
}

#[test]
fn test_multiple_breaking_changes() {
    let old = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("level", TypeInfo::Primitive("u16".to_string()), false),
            ("score", TypeInfo::Primitive("u64".to_string()), false),
        ],
        Some("1.0.0"),
    );

    let new = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            // level removed (breaking)
            ("score", TypeInfo::Primitive("u32".to_string()), false), // type changed (breaking)
            ("balance", TypeInfo::Primitive("u64".to_string()), false), // required field added (breaking)
        ],
        Some("2.0.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(!report.is_compatible);
    assert_eq!(report.count_by_level(IssueLevel::Breaking), 3);
}

#[test]
fn test_mixed_changes_compatible_and_breaking() {
    let old = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("level", TypeInfo::Primitive("u16".to_string()), false),
        ],
        Some("1.0.0"),
    );

    let new = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("level", TypeInfo::Primitive("u16".to_string()), false),
            ("email", TypeInfo::Primitive("String".to_string()), true), // optional (compatible)
            ("balance", TypeInfo::Primitive("u64".to_string()), false), // required (breaking)
        ],
        Some("2.0.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(!report.is_compatible); // Overall not compatible due to breaking change
    assert_eq!(report.count_by_level(IssueLevel::Breaking), 1);
    assert_eq!(report.count_by_level(IssueLevel::Info), 1);
}

#[test]
fn test_no_changes_is_compatible() {
    let old = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("level", TypeInfo::Primitive("u16".to_string()), false),
        ],
        Some("1.0.0"),
    );

    let new = create_struct(
        "Player",
        vec![
            (
                "wallet",
                TypeInfo::Primitive("PublicKey".to_string()),
                false,
            ),
            ("level", TypeInfo::Primitive("u16".to_string()), false),
        ],
        Some("1.0.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(report.is_compatible);
    assert_eq!(report.count_by_level(IssueLevel::Breaking), 0);
    assert_eq!(report.issues.len(), 0);
}

#[test]
fn test_array_type_change() {
    let old = create_struct(
        "Inventory",
        vec![(
            "items",
            TypeInfo::Array(Box::new(TypeInfo::Primitive("u64".to_string()))),
            false,
        )],
        Some("1.0.0"),
    );

    let new = create_struct(
        "Inventory",
        vec![(
            "items",
            TypeInfo::Array(Box::new(TypeInfo::Primitive("u32".to_string()))),
            false,
        )],
        Some("2.0.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(!report.is_compatible);
    assert_eq!(report.count_by_level(IssueLevel::Breaking), 1);
}

#[test]
fn test_option_type_change() {
    let old = create_struct(
        "User",
        vec![(
            "email",
            TypeInfo::Option(Box::new(TypeInfo::Primitive("String".to_string()))),
            true,
        )],
        Some("1.0.0"),
    );

    let new = create_struct(
        "User",
        vec![("email", TypeInfo::Primitive("String".to_string()), false)],
        Some("2.0.0"),
    );

    let checker = CompatibilityChecker::new(old, new);
    let report = checker.check().unwrap();

    assert!(!report.is_compatible);
    assert_eq!(report.count_by_level(IssueLevel::Breaking), 1);
}
