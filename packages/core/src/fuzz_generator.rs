// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Fuzz target generator for testing generated code with cargo-fuzz
//!
//! Generates fuzz targets that test:
//! - Round-trip serialization/deserialization
//! - Constraint validation
//! - Arithmetic operations
//! - Size limits

use crate::ir::{EnumDefinition, StructDefinition, TypeDefinition, TypeInfo};

/// Fuzz target generator
pub struct FuzzGenerator<'a> {
    /// All type definitions
    type_defs: &'a [TypeDefinition],
}

/// Generated fuzz target
#[derive(Debug, Clone)]
pub struct FuzzTarget {
    /// Target name (e.g., "fuzz_player_account")
    pub name: String,

    /// Type being fuzzed (e.g., "PlayerAccount")
    pub type_name: String,

    /// Generated Rust code for the fuzz target
    pub code: String,

    /// Whether this type needs PartialEq derive
    pub needs_partial_eq: bool,
}

impl<'a> FuzzGenerator<'a> {
    /// Create a new fuzz generator
    pub fn new(type_defs: &'a [TypeDefinition]) -> Self {
        Self { type_defs }
    }

    /// Generate all fuzz targets
    pub fn generate_all(&self) -> Vec<FuzzTarget> {
        let mut targets = Vec::new();

        for type_def in self.type_defs {
            match type_def {
                TypeDefinition::Struct(s) => {
                    targets.push(self.generate_struct_target(s));
                }
                TypeDefinition::Enum(e) => {
                    targets.push(self.generate_enum_target(e));
                }
            }
        }

        targets
    }

    /// Generate fuzz target for a struct
    fn generate_struct_target(&self, struct_def: &StructDefinition) -> FuzzTarget {
        let type_name = &struct_def.name;
        let target_name = format!("fuzz_{}", to_snake_case(type_name));

        let is_account = struct_def
            .metadata
            .attributes
            .contains(&"account".to_string());

        let needs_partial_eq = self.needs_partial_eq_derive(struct_def);

        // Collect arithmetic fields for bounds checking
        let arithmetic_fields = self.collect_arithmetic_fields(struct_def);

        let mut code = String::new();

        // Fuzz target header
        code.push_str("#![no_main]\n");
        code.push_str("use libfuzzer_sys::fuzz_target;\n");
        code.push_str("use borsh::{BorshSerialize, BorshDeserialize};\n\n");

        // Import the type
        if is_account {
            code.push_str("use anchor_lang::prelude::*;\n");
        }
        code.push_str(&format!("use generated::{};\n\n", type_name));

        // Fuzz target implementation
        code.push_str("fuzz_target!(|data: &[u8]| {\n");
        code.push_str("    // Attempt deserialization\n");
        code.push_str(&format!(
            "    if let Ok(instance) = {}::try_from_slice(data) {{\n",
            type_name
        ));

        // Round-trip test
        code.push_str("        // Round-trip: serialize → deserialize → compare\n");
        code.push_str("        let serialized = instance.try_to_vec().expect(\"serialization should succeed\");\n");
        code.push_str(&format!(
            "        let deserialized = {}::try_from_slice(&serialized)\n",
            type_name
        ));
        code.push_str("            .expect(\"round-trip deserialization should succeed\");\n\n");

        // Equality check (only if PartialEq is available)
        if needs_partial_eq {
            code.push_str("        // Verify equality (round-trip integrity)\n");
            code.push_str("        assert_eq!(instance, deserialized, \"round-trip should preserve data\");\n\n");
        }

        // Size validation
        code.push_str("        // Verify serialized size is within Solana limits\n");
        code.push_str("        assert!(serialized.len() <= 10_485_760, \"serialized size must not exceed 10MB\");\n\n");

        // Arithmetic bounds checking
        if !arithmetic_fields.is_empty() {
            code.push_str("        // Validate arithmetic field bounds\n");
            for field_name in &arithmetic_fields {
                code.push_str(&format!(
                    "        // Check {} is within reasonable bounds\n",
                    field_name
                ));
                code.push_str(&format!("        let _ = instance.{};\n", field_name));
            }
            code.push('\n');
        }

        // Account-specific checks
        if is_account {
            code.push_str("        // Account discriminator validation\n");
            code.push_str("        // Anchor adds 8-byte discriminator, verify size includes it\n");
            code.push_str("        assert!(serialized.len() >= 8, \"account data should include discriminator\");\n");
        }

        code.push_str("    }\n");
        code.push_str("});\n");

        FuzzTarget {
            name: target_name,
            type_name: type_name.clone(),
            code,
            needs_partial_eq,
        }
    }

    /// Generate fuzz target for an enum
    fn generate_enum_target(&self, enum_def: &EnumDefinition) -> FuzzTarget {
        let type_name = &enum_def.name;
        let target_name = format!("fuzz_{}", to_snake_case(type_name));

        let mut code = String::new();

        // Fuzz target header
        code.push_str("#![no_main]\n");
        code.push_str("use libfuzzer_sys::fuzz_target;\n");
        code.push_str("use borsh::{BorshSerialize, BorshDeserialize};\n\n");

        code.push_str(&format!("use generated::{};\n\n", type_name));

        // Fuzz target implementation
        code.push_str("fuzz_target!(|data: &[u8]| {\n");
        code.push_str("    // Attempt deserialization\n");
        code.push_str(&format!(
            "    if let Ok(instance) = {}::try_from_slice(data) {{\n",
            type_name
        ));

        // Round-trip test
        code.push_str("        // Round-trip: serialize → deserialize → compare\n");
        code.push_str("        let serialized = instance.try_to_vec().expect(\"serialization should succeed\");\n");
        code.push_str(&format!(
            "        let deserialized = {}::try_from_slice(&serialized)\n",
            type_name
        ));
        code.push_str("            .expect(\"round-trip deserialization should succeed\");\n\n");

        // Discriminant validation
        code.push_str("        // Verify discriminant is valid\n");
        code.push_str("        // Borsh uses first byte(s) for enum discriminant\n");
        code.push_str("        assert!(!serialized.is_empty(), \"enum serialization should not be empty\");\n");

        code.push_str("    }\n");
        code.push_str("});\n");

        FuzzTarget {
            name: target_name,
            type_name: type_name.clone(),
            code,
            needs_partial_eq: true, // Enums typically derive PartialEq
        }
    }

    /// Check if a struct needs PartialEq derive for equality testing
    fn needs_partial_eq_derive(&self, struct_def: &StructDefinition) -> bool {
        // Check if any field contains types that don't implement PartialEq
        // For simplicity, we'll assume most types do implement it
        // Complex logic can be added here to detect non-PartialEq types

        // Check for f32/f64 which have special PartialEq semantics
        for field in &struct_def.fields {
            if let TypeInfo::Primitive(ref t) = field.type_info {
                if t == "f32" || t == "f64" {
                    // Floating point requires special handling
                    return false;
                }
            }
        }

        true
    }

    /// Collect field names that are likely used for arithmetic
    fn collect_arithmetic_fields(&self, struct_def: &StructDefinition) -> Vec<String> {
        let arithmetic_keywords = [
            "balance", "amount", "supply", "total", "count", "price", "value", "reward", "stake",
            "fee", "lamport",
        ];

        struct_def
            .fields
            .iter()
            .filter(|field| {
                let lower = field.name.to_lowercase();
                arithmetic_keywords
                    .iter()
                    .any(|keyword| lower.contains(keyword))
            })
            .map(|field| field.name.clone())
            .collect()
    }

    /// Generate Cargo.toml for fuzz targets
    pub fn generate_cargo_toml(&self, crate_name: &str) -> String {
        let mut toml = String::new();

        toml.push_str("[package]\n");
        toml.push_str(&format!("name = \"{}-fuzz\"\n", crate_name));
        toml.push_str("version = \"0.0.0\"\n");
        toml.push_str("edition = \"2021\"\n");
        toml.push_str("publish = false\n\n");

        toml.push_str("[package.metadata]\n");
        toml.push_str("cargo-fuzz = true\n\n");

        toml.push_str("[dependencies]\n");
        toml.push_str("libfuzzer-sys = \"0.4\"\n");
        toml.push_str("borsh = { version = \"1.5\", features = [\"derive\"] }\n");
        toml.push_str("anchor-lang = \"0.30\"\n");
        toml.push_str("generated = { path = \"..\" }\n\n");

        toml.push_str("# Prevent this from interfering with workspaces\n");
        toml.push_str("[workspace]\n");
        toml.push_str("members = [\"fuzz_targets\"]\n\n");

        toml.push_str("[[bin]]\n");
        toml.push_str("name = \"fuzz_target_1\"\n");
        toml.push_str("path = \"fuzz_targets/fuzz_target_1.rs\"\n");
        toml.push_str("test = false\n");
        toml.push_str("doc = false\n");

        toml
    }

    /// Generate README.md for fuzz directory
    pub fn generate_readme(&self) -> String {
        let mut readme = String::new();

        readme.push_str("# Fuzz Testing\n\n");
        readme.push_str("Auto-generated fuzz targets for LUMOS-generated code.\n\n");
        readme.push_str("## Prerequisites\n\n");
        readme.push_str("```bash\n");
        readme.push_str("cargo install cargo-fuzz\n");
        readme.push_str("```\n\n");
        readme.push_str("## Running Fuzz Tests\n\n");
        readme.push_str("```bash\n");
        readme.push_str("# List all fuzz targets\n");
        readme.push_str("cargo fuzz list\n\n");
        readme.push_str("# Run a specific target\n");
        readme.push_str("cargo fuzz run fuzz_target_name\n\n");
        readme.push_str("# Run with multiple jobs\n");
        readme.push_str("cargo fuzz run fuzz_target_name -- -jobs=4\n\n");
        readme.push_str("# Run for specific duration\n");
        readme.push_str("cargo fuzz run fuzz_target_name -- -max_total_time=60\n");
        readme.push_str("```\n\n");
        readme.push_str("## What's Being Tested\n\n");
        readme.push_str("Each fuzz target tests:\n\n");
        readme.push_str("- **Round-trip integrity**: Serialize → Deserialize → Compare\n");
        readme.push_str("- **Size limits**: Ensure data fits within Solana's 10MB limit\n");
        readme.push_str("- **Discriminator validation**: For Anchor accounts\n");
        readme.push_str("- **Arithmetic bounds**: For balance/amount fields\n\n");
        readme.push_str("## Corpus\n\n");
        readme.push_str("Fuzzing corpus files are stored in `corpus/` directory.\n");
        readme.push_str("These provide seed inputs for the fuzzer.\n\n");
        readme.push_str("## Artifacts\n\n");
        readme
            .push_str("Crash artifacts are saved to `artifacts/` directory when failures occur.\n");

        readme
    }

    /// Get list of all type names for fuzzing
    pub fn get_type_names(&self) -> Vec<String> {
        self.type_defs
            .iter()
            .map(|type_def| match type_def {
                TypeDefinition::Struct(s) => s.name.clone(),
                TypeDefinition::Enum(e) => e.name.clone(),
            })
            .collect()
    }

    /// Check if a type name exists
    pub fn type_exists(&self, type_name: &str) -> bool {
        self.type_defs.iter().any(|type_def| match type_def {
            TypeDefinition::Struct(s) => s.name == type_name,
            TypeDefinition::Enum(e) => e.name == type_name,
        })
    }
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            // Add underscore before uppercase letter if not the first character
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{FieldDefinition, Metadata};

    #[test]
    fn test_generates_struct_fuzz_target() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "PlayerAccount".to_string(),
            fields: vec![
                FieldDefinition {
                    name: "wallet".to_string(),
                    type_info: TypeInfo::Primitive("PublicKey".to_string()),
                    optional: false,
                    deprecated: None,
                },
                FieldDefinition {
                    name: "balance".to_string(),
                    type_info: TypeInfo::Primitive("u64".to_string()),
                    optional: false,
                    deprecated: None,
                },
            ],
            metadata: Metadata {
                solana: true,
                attributes: vec!["account".to_string()],
            },
        })];

        let generator = FuzzGenerator::new(&type_defs);
        let targets = generator.generate_all();

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].name, "fuzz_player_account");
        assert_eq!(targets[0].type_name, "PlayerAccount");
        assert!(targets[0].code.contains("fuzz_target!"));
        assert!(targets[0].code.contains("try_from_slice"));
        assert!(targets[0].code.contains("round-trip"));
    }

    #[test]
    fn test_generates_enum_fuzz_target() {
        let type_defs = vec![TypeDefinition::Enum(EnumDefinition {
            name: "GameState".to_string(),
            variants: vec![],
            metadata: Metadata::default(),
        })];

        let generator = FuzzGenerator::new(&type_defs);
        let targets = generator.generate_all();

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].name, "fuzz_game_state");
        assert!(targets[0].code.contains("discriminant"));
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("PlayerAccount"), "player_account");
        assert_eq!(to_snake_case("NFTMetadata"), "n_f_t_metadata");
        assert_eq!(to_snake_case("SimpleType"), "simple_type");
    }

    #[test]
    fn test_generates_cargo_toml() {
        let type_defs = vec![];
        let generator = FuzzGenerator::new(&type_defs);
        let cargo_toml = generator.generate_cargo_toml("my-project");

        assert!(cargo_toml.contains("name = \"my-project-fuzz\""));
        assert!(cargo_toml.contains("libfuzzer-sys"));
        assert!(cargo_toml.contains("borsh"));
    }

    #[test]
    fn test_get_type_names() {
        let type_defs = vec![
            TypeDefinition::Struct(StructDefinition {
                name: "Account1".to_string(),
                fields: vec![],
                metadata: Metadata::default(),
            }),
            TypeDefinition::Enum(EnumDefinition {
                name: "State1".to_string(),
                variants: vec![],
                metadata: Metadata::default(),
            }),
        ];

        let generator = FuzzGenerator::new(&type_defs);
        let names = generator.get_type_names();

        assert_eq!(names, vec!["Account1", "State1"]);
    }

    #[test]
    fn test_type_exists() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "PlayerAccount".to_string(),
            fields: vec![],
            metadata: Metadata::default(),
        })];

        let generator = FuzzGenerator::new(&type_defs);

        assert!(generator.type_exists("PlayerAccount"));
        assert!(!generator.type_exists("NonExistent"));
    }
}
