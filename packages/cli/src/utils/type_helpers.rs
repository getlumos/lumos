// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Type conversion and formatting utilities for CLI

use lumos_core::anchor::AnchorAccountType;
use lumos_core::ir::TypeInfo;

/// Format a TypeInfo as a string representation
pub fn format_type(type_info: &TypeInfo) -> String {
    match type_info {
        TypeInfo::Primitive(p) => p.clone(),
        TypeInfo::Generic(param) => param.clone(),
        TypeInfo::UserDefined(u) => u.clone(),
        TypeInfo::Array(inner) => format!("Vec<{}>", format_type(inner)),
        TypeInfo::FixedArray { element, size } => {
            format!("[{}; {}]", format_type(element), size)
        }
        TypeInfo::Option(inner) => format!("Option<{}>", format_type(inner)),
    }
}

/// Convert PascalCase to snake_case
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else {
            result.push(ch);
            prev_is_upper = false;
        }
    }

    result
}

/// Convert TypeInfo to Rust type string
pub fn type_info_to_rust_type(ty: &TypeInfo) -> String {
    match ty {
        TypeInfo::Primitive(name) => match name.as_str() {
            "PublicKey" | "Pubkey" => "Pubkey".to_string(),
            _ => name.clone(),
        },
        TypeInfo::Generic(name) => name.clone(),
        TypeInfo::UserDefined(name) => name.clone(),
        TypeInfo::Array(inner) => {
            format!("Vec<{}>", type_info_to_rust_type(inner))
        }
        TypeInfo::FixedArray { element, size } => {
            format!("[{}; {}]", type_info_to_rust_type(element), size)
        }
        TypeInfo::Option(inner) => {
            format!("Option<{}>", type_info_to_rust_type(inner))
        }
    }
}

/// Infer Anchor account type from LUMOS type
pub fn infer_anchor_account_type(ty: &TypeInfo) -> AnchorAccountType {
    match ty {
        TypeInfo::Primitive(name) if name == "Signer" => AnchorAccountType::Signer,
        TypeInfo::UserDefined(name) => match name.as_str() {
            "Signer" => AnchorAccountType::Signer,
            "SystemAccount" => AnchorAccountType::SystemAccount,
            "UncheckedAccount" => AnchorAccountType::UncheckedAccount,
            "AccountInfo" => AnchorAccountType::AccountInfo,
            _ if name.starts_with("Program<") => {
                let inner = name
                    .strip_prefix("Program<")
                    .and_then(|s| s.strip_suffix('>'))
                    .unwrap_or("System");
                AnchorAccountType::Program(inner.to_string())
            }
            _ if name.starts_with("Sysvar<") => {
                let inner = name
                    .strip_prefix("Sysvar<")
                    .and_then(|s| s.strip_suffix('>'))
                    .unwrap_or("Rent");
                AnchorAccountType::Sysvar(inner.to_string())
            }
            _ if name == "SystemProgram" => AnchorAccountType::Program("System".to_string()),
            _ => AnchorAccountType::Account(name.clone()),
        },
        _ => AnchorAccountType::AccountInfo,
    }
}
