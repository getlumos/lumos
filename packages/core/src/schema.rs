// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Schema parsing and validation

use serde::{Deserialize, Serialize};

/// A LUMOS schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Schema name
    pub name: String,

    /// Whether this is a Solana-specific type
    #[serde(default)]
    pub solana: bool,

    /// Fields in this schema
    pub fields: Vec<Field>,
}

/// A field in a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Field name
    pub name: String,

    /// Field type (e.g., "u64", "string", "PublicKey")
    #[serde(rename = "type")]
    pub type_name: String,

    /// Whether this field is optional
    #[serde(default)]
    pub optional: bool,
}

impl Schema {
    /// Parse a schema from TOML string
    pub fn from_toml(input: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_schema() {
        let toml = r#"
            name = "User"
            solana = true

            [[fields]]
            name = "id"
            type = "u64"
        "#;

        let result = Schema::from_toml(toml);
        assert!(result.is_ok());

        if let Ok(schema) = result {
            assert_eq!(schema.name, "User");
            assert_eq!(schema.solana, true);
            assert_eq!(schema.fields.len(), 1);
            assert_eq!(schema.fields[0].name, "id");
            assert_eq!(schema.fields[0].type_name, "u64");
        }
    }
}
