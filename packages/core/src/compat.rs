// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Schema backward compatibility validation
//!
//! This module provides functionality to validate that schema changes
//! are backward compatible, preventing breaking changes that could cause
//! deployed Solana programs to fail reading existing account data.

use crate::ir::TypeDefinition;
use crate::migration::{SchemaChange, SchemaDiff};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Compatibility issue severity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueLevel {
    /// Breaking change - CANNOT read old data
    Breaking,
    /// Warning - may cause issues but not breaking
    Warning,
    /// Informational - safe change
    Info,
}

/// A compatibility issue found during schema comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityIssue {
    /// Issue severity level
    pub level: IssueLevel,

    /// Type name where issue was found
    pub type_name: String,

    /// Short description of the issue
    pub message: String,

    /// Detailed explanation of why this is an issue
    pub reason: String,

    /// Suggested fix or workaround
    pub suggestion: Option<String>,

    /// The specific change that caused this issue
    pub change: SchemaChange,
}

impl CompatibilityIssue {
    /// Create a new breaking change issue
    pub fn breaking(
        type_name: String,
        message: String,
        reason: String,
        suggestion: Option<String>,
        change: SchemaChange,
    ) -> Self {
        Self {
            level: IssueLevel::Breaking,
            type_name,
            message,
            reason,
            suggestion,
            change,
        }
    }

    /// Create a new warning issue
    pub fn warning(
        type_name: String,
        message: String,
        reason: String,
        suggestion: Option<String>,
        change: SchemaChange,
    ) -> Self {
        Self {
            level: IssueLevel::Warning,
            type_name,
            message,
            reason,
            suggestion,
            change,
        }
    }

    /// Create a new info issue
    pub fn info(
        type_name: String,
        message: String,
        reason: String,
        change: SchemaChange,
    ) -> Self {
        Self {
            level: IssueLevel::Info,
            type_name,
            message,
            reason,
            suggestion: None,
            change,
        }
    }
}

impl fmt::Display for CompatibilityIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self.level {
            IssueLevel::Breaking => "✗",
            IssueLevel::Warning => "⚠",
            IssueLevel::Info => "ℹ",
        };

        write!(f, "{} {}: {}", symbol, self.type_name, self.message)?;

        if let Some(ref suggestion) = self.suggestion {
            write!(f, "\n  Suggestion: {}", suggestion)?;
        }

        Ok(())
    }
}

/// Result of a compatibility check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    /// Old schema version
    pub from_version: Option<String>,

    /// New schema version
    pub to_version: Option<String>,

    /// All issues found
    pub issues: Vec<CompatibilityIssue>,

    /// Whether schemas are backward compatible
    pub is_compatible: bool,

    /// Whether version bump is appropriate for changes
    pub version_bump_valid: bool,
}

impl CompatibilityReport {
    /// Create a new compatibility report
    pub fn new(from_version: Option<String>, to_version: Option<String>) -> Self {
        Self {
            from_version,
            to_version,
            issues: Vec::new(),
            is_compatible: true,
            version_bump_valid: true,
        }
    }

    /// Add an issue to the report
    pub fn add_issue(&mut self, issue: CompatibilityIssue) {
        if issue.level == IssueLevel::Breaking {
            self.is_compatible = false;
        }
        self.issues.push(issue);
    }

    /// Get count of issues by level
    pub fn count_by_level(&self, level: IssueLevel) -> usize {
        self.issues.iter().filter(|i| i.level == level).count()
    }

    /// Get breaking issues
    pub fn breaking_issues(&self) -> Vec<&CompatibilityIssue> {
        self.issues
            .iter()
            .filter(|i| i.level == IssueLevel::Breaking)
            .collect()
    }

    /// Get warning issues
    pub fn warnings(&self) -> Vec<&CompatibilityIssue> {
        self.issues
            .iter()
            .filter(|i| i.level == IssueLevel::Warning)
            .collect()
    }
}

/// Compatibility checker for schema evolution
pub struct CompatibilityChecker {
    old_schema: TypeDefinition,
    new_schema: TypeDefinition,
}

impl CompatibilityChecker {
    /// Create a new compatibility checker
    pub fn new(old_schema: TypeDefinition, new_schema: TypeDefinition) -> Self {
        Self {
            old_schema,
            new_schema,
        }
    }

    /// Check backward compatibility
    pub fn check(&self) -> Result<CompatibilityReport, String> {
        // Compute schema diff
        let diff = SchemaDiff::compute(&self.old_schema, &self.new_schema)?;

        let mut report = CompatibilityReport::new(
            diff.from_version.clone(),
            diff.to_version.clone(),
        );

        // Analyze each change for compatibility
        for change in diff.changes.iter() {
            if let Some(issue) = self.analyze_change(&diff.type_name, change) {
                report.add_issue(issue);
            }
        }

        // Validate version bump if both versions are present
        if let (Some(ref from_ver), Some(ref to_ver)) = (&diff.from_version, &diff.to_version) {
            if let Err(msg) = validate_version_bump(from_ver, to_ver, &report) {
                report.version_bump_valid = false;
                // Add a breaking issue for invalid version bump
                report.add_issue(CompatibilityIssue::breaking(
                    diff.type_name.clone(),
                    "Invalid version bump".to_string(),
                    msg,
                    Some("Bump major version for breaking changes, minor for new features".to_string()),
                    SchemaChange::FieldAdded {
                        name: String::new(),
                        type_info: crate::ir::TypeInfo::Primitive("bool".to_string()),
                        optional: false,
                    },
                ));
            }
        }

        Ok(report)
    }

    /// Analyze a single schema change for compatibility
    fn analyze_change(
        &self,
        type_name: &str,
        change: &SchemaChange,
    ) -> Option<CompatibilityIssue> {
        match change {
            SchemaChange::FieldAdded { name, type_info, optional } => {
                if *optional {
                    // Adding optional fields is safe
                    Some(CompatibilityIssue::info(
                        type_name.to_string(),
                        format!("Added optional field: {}", name),
                        "Old data will deserialize successfully with this field as None".to_string(),
                        change.clone(),
                    ))
                } else {
                    // Adding required fields is unsafe
                    Some(CompatibilityIssue::breaking(
                        type_name.to_string(),
                        format!("Added required field: {} ({:?})", name, type_info),
                        "Old account data lacks this field, deserialization will fail".to_string(),
                        Some(format!("Make field optional or provide migration code")),
                        change.clone(),
                    ))
                }
            }

            SchemaChange::FieldRemoved { name, type_info } => {
                // Removing fields is always breaking
                Some(CompatibilityIssue::breaking(
                    type_name.to_string(),
                    format!("Removed field: {} ({:?})", name, type_info),
                    "Old account data contains this field, causing extra bytes during deserialization".to_string(),
                    Some("Keep deprecated field or create migration".to_string()),
                    change.clone(),
                ))
            }

            SchemaChange::FieldTypeChanged { name, old_type, new_type } => {
                // Type changes are breaking
                Some(CompatibilityIssue::breaking(
                    type_name.to_string(),
                    format!("Changed type of '{}': {:?} → {:?}", name, old_type, new_type),
                    "Borsh deserialization will fail due to type mismatch".to_string(),
                    Some("Create a migration function or use a new field name".to_string()),
                    change.clone(),
                ))
            }

            SchemaChange::FieldReordered { name, old_position, new_position } => {
                // Field reordering is safe in Borsh (position-based serialization)
                Some(CompatibilityIssue::info(
                    type_name.to_string(),
                    format!("Reordered field '{}': position {} → {}", name, old_position, new_position),
                    "Borsh uses field order for serialization, so this is safe".to_string(),
                    change.clone(),
                ))
            }

            SchemaChange::VariantAdded { name } => {
                // Adding enum variants is generally safe
                Some(CompatibilityIssue::info(
                    type_name.to_string(),
                    format!("Added enum variant: {}", name),
                    "Old data won't use this variant, so deserialization remains compatible".to_string(),
                    change.clone(),
                ))
            }

            SchemaChange::VariantRemoved { name } => {
                // Removing enum variants is breaking
                Some(CompatibilityIssue::breaking(
                    type_name.to_string(),
                    format!("Removed enum variant: {}", name),
                    "Old data may contain this variant, causing deserialization failure".to_string(),
                    Some("Mark variant as deprecated instead of removing".to_string()),
                    change.clone(),
                ))
            }
        }
    }
}

/// Parse a semantic version string into (major, minor, patch)
fn parse_semver(version: &str) -> Result<(u32, u32, u32), String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("Invalid semver format: {}", version));
    }

    let major = parts[0]
        .parse::<u32>()
        .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
    let minor = parts[1]
        .parse::<u32>()
        .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
    let patch = parts[2]
        .parse::<u32>()
        .map_err(|_| format!("Invalid patch version: {}", parts[2]))?;

    Ok((major, minor, patch))
}

/// Validate that version bump is appropriate for the changes
fn validate_version_bump(
    from_version: &str,
    to_version: &str,
    report: &CompatibilityReport,
) -> Result<(), String> {
    let (old_major, old_minor, _old_patch) = parse_semver(from_version)?;
    let (new_major, new_minor, _new_patch) = parse_semver(to_version)?;

    let has_breaking = report.count_by_level(IssueLevel::Breaking) > 0;

    if has_breaking {
        // Breaking changes require major version bump
        if new_major == old_major {
            return Err(format!(
                "Breaking changes detected but major version not bumped ({} → {})",
                from_version, to_version
            ));
        }
    } else {
        // No breaking changes - minor or patch bump is OK
        // But major bump is also allowed (e.g., for marketing reasons)
        if new_major == old_major && new_minor < old_minor {
            return Err(format!(
                "Minor version decreased ({} → {})",
                from_version, to_version
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("1.2.3"), Ok((1, 2, 3)));
        assert_eq!(parse_semver("0.0.1"), Ok((0, 0, 1)));
        assert_eq!(parse_semver("10.20.30"), Ok((10, 20, 30)));
        assert!(parse_semver("1.2").is_err());
        assert!(parse_semver("a.b.c").is_err());
    }

    #[test]
    fn test_version_bump_validation() {
        // Test helper to create a report with breaking changes
        let mut report_with_breaking = CompatibilityReport::new(
            Some("1.0.0".to_string()),
            Some("2.0.0".to_string()),
        );
        report_with_breaking.add_issue(CompatibilityIssue::breaking(
            "Test".to_string(),
            "Test breaking".to_string(),
            "Test".to_string(),
            None,
            SchemaChange::FieldRemoved {
                name: "test".to_string(),
                type_info: crate::ir::TypeInfo::Primitive("bool".to_string()),
            },
        ));

        // Breaking changes require major bump
        assert!(validate_version_bump("1.0.0", "2.0.0", &report_with_breaking).is_ok());
        assert!(validate_version_bump("1.0.0", "1.1.0", &report_with_breaking).is_err());

        // Non-breaking changes can use minor or patch bump
        let report_no_breaking = CompatibilityReport::new(
            Some("1.0.0".to_string()),
            Some("1.1.0".to_string()),
        );
        assert!(validate_version_bump("1.0.0", "1.1.0", &report_no_breaking).is_ok());
        assert!(validate_version_bump("1.0.0", "1.0.1", &report_no_breaking).is_ok());
    }
}
