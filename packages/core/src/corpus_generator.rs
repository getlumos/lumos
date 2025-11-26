// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Corpus generator for fuzz testing
//!
//! Generates initial corpus files with valid serialized instances
//! to seed the fuzzer with known-good inputs.

use crate::ir::{
    EnumDefinition, EnumVariantDefinition, StructDefinition, TypeDefinition, TypeInfo,
};

/// Corpus file entry
#[derive(Debug, Clone)]
pub struct CorpusFile {
    /// File name (e.g., "player_account_minimal")
    pub name: String,

    /// Type being tested
    pub type_name: String,

    /// Serialized bytes (Borsh-encoded)
    pub data: Vec<u8>,

    /// Human-readable description
    pub description: String,
}

/// Corpus generator
pub struct CorpusGenerator<'a> {
    /// All type definitions
    type_defs: &'a [TypeDefinition],
}

impl<'a> CorpusGenerator<'a> {
    /// Create a new corpus generator
    pub fn new(type_defs: &'a [TypeDefinition]) -> Self {
        Self { type_defs }
    }

    /// Generate corpus files for all types
    pub fn generate_all(&self) -> Vec<CorpusFile> {
        let mut files = Vec::new();

        for type_def in self.type_defs {
            match type_def {
                TypeDefinition::Struct(s) => {
                    files.extend(self.generate_struct_corpus(s));
                }
                TypeDefinition::Enum(e) => {
                    files.extend(self.generate_enum_corpus(e));
                }
                TypeDefinition::TypeAlias(_) => {
                    // Type aliases don't generate corpus files (they're resolved)
                }
            }
        }

        files
    }

    /// Generate corpus files for a struct
    fn generate_struct_corpus(&self, struct_def: &StructDefinition) -> Vec<CorpusFile> {
        let mut files = Vec::new();

        // Generate minimal valid instance (all zeros/defaults)
        files.push(self.generate_minimal_struct(struct_def));

        // Generate maximal instance (if applicable)
        if let Some(maximal) = self.generate_maximal_struct(struct_def) {
            files.push(maximal);
        }

        // Generate edge cases
        files.extend(self.generate_struct_edge_cases(struct_def));

        files
    }

    /// Generate minimal struct corpus (zero values)
    fn generate_minimal_struct(&self, struct_def: &StructDefinition) -> CorpusFile {
        let mut data = Vec::new();

        // Add Anchor discriminator if it's an account
        if struct_def
            .metadata
            .attributes
            .contains(&"account".to_string())
        {
            // 8-byte discriminator (zeros for corpus)
            data.extend_from_slice(&[0u8; 8]);
        }

        // Serialize each field with minimal values
        for field in &struct_def.fields {
            data.extend(self.serialize_minimal_value(&field.type_info, field.optional));
        }

        CorpusFile {
            name: format!("{}_minimal", to_snake_case(&struct_def.name)),
            type_name: struct_def.name.clone(),
            data,
            description: "Minimal valid instance with zero/default values".to_string(),
        }
    }

    /// Generate maximal struct corpus (max values where applicable)
    fn generate_maximal_struct(&self, struct_def: &StructDefinition) -> Option<CorpusFile> {
        let mut data = Vec::new();

        // Add Anchor discriminator if it's an account
        if struct_def
            .metadata
            .attributes
            .contains(&"account".to_string())
        {
            data.extend_from_slice(&[0u8; 8]);
        }

        // Serialize each field with maximal values
        for field in &struct_def.fields {
            data.extend(self.serialize_maximal_value(&field.type_info, field.optional));
        }

        Some(CorpusFile {
            name: format!("{}_maximal", to_snake_case(&struct_def.name)),
            type_name: struct_def.name.clone(),
            data,
            description: "Maximal valid instance with maximum values".to_string(),
        })
    }

    /// Generate edge case corpus files for a struct
    fn generate_struct_edge_cases(&self, struct_def: &StructDefinition) -> Vec<CorpusFile> {
        let mut files = Vec::new();

        // Check if struct has Option fields
        let has_optional = struct_def
            .fields
            .iter()
            .any(|f| matches!(f.type_info, TypeInfo::Option(_)));

        if has_optional {
            files.push(self.generate_optional_none_case(struct_def));
            files.push(self.generate_optional_some_case(struct_def));
        }

        // Check if struct has Vec fields
        let has_vec = struct_def
            .fields
            .iter()
            .any(|f| matches!(f.type_info, TypeInfo::Array(_)));

        if has_vec {
            files.push(self.generate_empty_vec_case(struct_def));
            files.push(self.generate_single_elem_vec_case(struct_def));
        }

        files
    }

    /// Generate corpus with all Option fields as None
    fn generate_optional_none_case(&self, struct_def: &StructDefinition) -> CorpusFile {
        let mut data = Vec::new();

        if struct_def
            .metadata
            .attributes
            .contains(&"account".to_string())
        {
            data.extend_from_slice(&[0u8; 8]);
        }

        for field in &struct_def.fields {
            if matches!(field.type_info, TypeInfo::Option(_)) {
                // Borsh encodes Option::None as 0u8
                data.push(0);
            } else {
                data.extend(self.serialize_minimal_value(&field.type_info, field.optional));
            }
        }

        CorpusFile {
            name: format!("{}_optional_none", to_snake_case(&struct_def.name)),
            type_name: struct_def.name.clone(),
            data,
            description: "Instance with all Option fields set to None".to_string(),
        }
    }

    /// Generate corpus with all Option fields as Some
    fn generate_optional_some_case(&self, struct_def: &StructDefinition) -> CorpusFile {
        let mut data = Vec::new();

        if struct_def
            .metadata
            .attributes
            .contains(&"account".to_string())
        {
            data.extend_from_slice(&[0u8; 8]);
        }

        for field in &struct_def.fields {
            if let TypeInfo::Option(inner) = &field.type_info {
                // Borsh encodes Option::Some as 1u8 followed by the value
                data.push(1);
                data.extend(self.serialize_minimal_value(inner, false));
            } else {
                data.extend(self.serialize_minimal_value(&field.type_info, field.optional));
            }
        }

        CorpusFile {
            name: format!("{}_optional_some", to_snake_case(&struct_def.name)),
            type_name: struct_def.name.clone(),
            data,
            description: "Instance with all Option fields set to Some".to_string(),
        }
    }

    /// Generate corpus with empty vectors
    fn generate_empty_vec_case(&self, struct_def: &StructDefinition) -> CorpusFile {
        let mut data = Vec::new();

        if struct_def
            .metadata
            .attributes
            .contains(&"account".to_string())
        {
            data.extend_from_slice(&[0u8; 8]);
        }

        for field in &struct_def.fields {
            if matches!(field.type_info, TypeInfo::Array(_)) {
                // Borsh encodes Vec length as u32 (little-endian)
                data.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                data.extend(self.serialize_minimal_value(&field.type_info, field.optional));
            }
        }

        CorpusFile {
            name: format!("{}_empty_vec", to_snake_case(&struct_def.name)),
            type_name: struct_def.name.clone(),
            data,
            description: "Instance with all Vec fields empty".to_string(),
        }
    }

    /// Generate corpus with single-element vectors
    fn generate_single_elem_vec_case(&self, struct_def: &StructDefinition) -> CorpusFile {
        let mut data = Vec::new();

        if struct_def
            .metadata
            .attributes
            .contains(&"account".to_string())
        {
            data.extend_from_slice(&[0u8; 8]);
        }

        for field in &struct_def.fields {
            if let TypeInfo::Array(inner) = &field.type_info {
                // Length: 1 (u32 little-endian)
                data.extend_from_slice(&[1, 0, 0, 0]);
                // Single element
                data.extend(self.serialize_minimal_value(inner, false));
            } else {
                data.extend(self.serialize_minimal_value(&field.type_info, field.optional));
            }
        }

        CorpusFile {
            name: format!("{}_single_elem_vec", to_snake_case(&struct_def.name)),
            type_name: struct_def.name.clone(),
            data,
            description: "Instance with all Vec fields containing one element".to_string(),
        }
    }

    /// Generate corpus files for an enum
    fn generate_enum_corpus(&self, enum_def: &EnumDefinition) -> Vec<CorpusFile> {
        let mut files = Vec::new();

        // Generate one corpus file per variant
        for (index, variant) in enum_def.variants.iter().enumerate() {
            files.push(self.generate_enum_variant_corpus(enum_def, variant, index));
        }

        files
    }

    /// Generate corpus for a single enum variant
    fn generate_enum_variant_corpus(
        &self,
        enum_def: &EnumDefinition,
        variant: &EnumVariantDefinition,
        discriminant: usize,
    ) -> CorpusFile {
        let mut data = Vec::new();

        // Borsh encodes enum discriminant as u32 (little-endian)
        data.extend_from_slice(&(discriminant as u32).to_le_bytes());

        // Encode variant data
        match variant {
            EnumVariantDefinition::Unit { .. } => {
                // Unit variant has no data
            }
            EnumVariantDefinition::Tuple { types, .. } => {
                // Tuple variant: serialize each field
                for field in types {
                    data.extend(self.serialize_minimal_value(field, false));
                }
            }
            EnumVariantDefinition::Struct { fields, .. } => {
                // Struct variant: serialize each field
                for field in fields {
                    data.extend(self.serialize_minimal_value(&field.type_info, field.optional));
                }
            }
        }

        CorpusFile {
            name: format!(
                "{}_{}_variant",
                to_snake_case(&enum_def.name),
                to_snake_case(variant.name())
            ),
            type_name: enum_def.name.clone(),
            data,
            description: format!("Enum variant: {}", variant.name()),
        }
    }

    /// Serialize a minimal value for a given type
    fn serialize_minimal_value(&self, type_info: &TypeInfo, _optional: bool) -> Vec<u8> {
        match type_info {
            TypeInfo::Primitive(name) => self.serialize_minimal_primitive(name),
            TypeInfo::Generic(_) => {
                // Generic parameters can't be serialized without concrete type
                // Return empty bytes (fuzzer will discover valid structures)
                vec![]
            }
            TypeInfo::Array(_) => {
                // Empty vec (length = 0)
                vec![0, 0, 0, 0]
            }
            TypeInfo::FixedArray { element, size } => {
                // Fixed array: serialize size * element minimal values (no length prefix!)
                let mut data = Vec::new();
                for _ in 0..*size {
                    data.extend(self.serialize_minimal_value(element, false));
                }
                data
            }
            TypeInfo::Option(_) => {
                // None
                vec![0]
            }
            TypeInfo::UserDefined(_) => {
                // For user-defined types, we can't easily generate valid instances
                // Return empty bytes (fuzzer will discover valid structures)
                vec![]
            }
        }
    }

    /// Serialize a maximal value for a given type
    fn serialize_maximal_value(&self, type_info: &TypeInfo, _optional: bool) -> Vec<u8> {
        match type_info {
            TypeInfo::Primitive(name) => self.serialize_maximal_primitive(name),
            TypeInfo::Generic(_) => {
                // Generic parameters can't be serialized without concrete type
                vec![]
            }
            TypeInfo::Array(inner) => {
                // Vec with 10 elements
                let mut data = vec![10, 0, 0, 0]; // length = 10
                for _ in 0..10 {
                    data.extend(self.serialize_minimal_value(inner, false));
                }
                data
            }
            TypeInfo::FixedArray { element, size } => {
                // Fixed array: serialize size * element maximal values (no length prefix!)
                let mut data = Vec::new();
                for _ in 0..*size {
                    data.extend(self.serialize_maximal_value(element, false));
                }
                data
            }
            TypeInfo::Option(inner) => {
                // Some(max_value)
                let mut data = vec![1]; // Some
                data.extend(self.serialize_maximal_value(inner, false));
                data
            }
            TypeInfo::UserDefined(_) => vec![],
        }
    }

    /// Serialize minimal primitive value
    fn serialize_minimal_primitive(&self, type_name: &str) -> Vec<u8> {
        match type_name {
            "bool" => vec![0],
            "u8" | "i8" => vec![0],
            "u16" | "i16" => vec![0, 0],
            "u32" | "i32" | "f32" => vec![0, 0, 0, 0],
            "u64" | "i64" | "f64" => vec![0, 0, 0, 0, 0, 0, 0, 0],
            "u128" | "i128" => vec![0; 16],
            "Pubkey" | "PublicKey" => vec![0; 32],
            "Signature" => vec![0; 64],
            "String" => {
                // Empty string: length 0
                vec![0, 0, 0, 0]
            }
            _ => vec![],
        }
    }

    /// Serialize maximal primitive value
    fn serialize_maximal_primitive(&self, type_name: &str) -> Vec<u8> {
        match type_name {
            "bool" => vec![1],
            "u8" => vec![255],
            "i8" => vec![127],
            "u16" => vec![255, 255],
            "i16" => vec![255, 127],
            "u32" => vec![255, 255, 255, 255],
            "i32" => vec![255, 255, 255, 127],
            "u64" => vec![255, 255, 255, 255, 255, 255, 255, 255],
            "i64" => vec![255, 255, 255, 255, 255, 255, 255, 127],
            "u128" => vec![255; 16],
            "i128" => {
                let mut bytes = vec![255; 16];
                bytes[15] = 127; // Max positive i128
                bytes
            }
            "f32" => {
                // Max f32 value
                let max: f32 = 3.4028235e38;
                max.to_le_bytes().to_vec()
            }
            "f64" => {
                // Max f64 value
                let max: f64 = 1.7976931348623157e308;
                max.to_le_bytes().to_vec()
            }
            "Pubkey" | "PublicKey" => vec![255; 32],
            "Signature" => vec![255; 64],
            "String" => {
                // String with 100 'A' characters
                let s = "A".repeat(100);
                let mut data = (s.len() as u32).to_le_bytes().to_vec();
                data.extend_from_slice(s.as_bytes());
                data
            }
            _ => vec![],
        }
    }
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{FieldDefinition, Metadata};

    #[test]
    fn test_generates_minimal_struct_corpus() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "SimpleStruct".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "value".to_string(),
                type_info: TypeInfo::Primitive("u32".to_string()),
                optional: false,
                deprecated: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
        })];

        let generator = CorpusGenerator::new(&type_defs);
        let corpus = generator.generate_all();

        assert!(!corpus.is_empty());
        let minimal = corpus.iter().find(|c| c.name.contains("minimal")).unwrap();

        // u32 minimal value: 4 bytes of zeros
        assert_eq!(minimal.data, vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_generates_account_discriminator() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "AccountStruct".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "value".to_string(),
                type_info: TypeInfo::Primitive("u8".to_string()),
                optional: false,
                deprecated: None,
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
        })];

        let generator = CorpusGenerator::new(&type_defs);
        let corpus = generator.generate_all();

        let minimal = corpus.iter().find(|c| c.name.contains("minimal")).unwrap();

        // Should have 8-byte discriminator + 1 byte for u8 field
        assert_eq!(minimal.data.len(), 9);
        assert_eq!(&minimal.data[0..8], &[0u8; 8]); // discriminator
    }

    #[test]
    fn test_generates_optional_corpus() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "OptionalStruct".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "maybe_value".to_string(),
                type_info: TypeInfo::Option(Box::new(TypeInfo::Primitive("u32".to_string()))),
                optional: true,
                deprecated: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
        })];

        let generator = CorpusGenerator::new(&type_defs);
        let corpus = generator.generate_all();

        let none_case = corpus
            .iter()
            .find(|c| c.name.contains("optional_none"))
            .unwrap();
        let some_case = corpus
            .iter()
            .find(|c| c.name.contains("optional_some"))
            .unwrap();

        // None: just 0u8
        assert_eq!(none_case.data, vec![0]);

        // Some: 1u8 + 4 bytes for u32
        assert_eq!(some_case.data.len(), 5);
        assert_eq!(some_case.data[0], 1);
    }

    #[test]
    fn test_generates_vec_corpus() {
        let type_defs = vec![TypeDefinition::Struct(StructDefinition {
            name: "VecStruct".to_string(),
            generic_params: vec![],
            fields: vec![FieldDefinition {
                name: "items".to_string(),
                type_info: TypeInfo::Array(Box::new(TypeInfo::Primitive("u8".to_string()))),
                optional: false,
                deprecated: None,
                anchor_attrs: vec![],
            }],
            metadata: Metadata::default(),
        })];

        let generator = CorpusGenerator::new(&type_defs);
        let corpus = generator.generate_all();

        let empty_vec = corpus
            .iter()
            .find(|c| c.name.contains("empty_vec"))
            .unwrap();
        let single_elem = corpus
            .iter()
            .find(|c| c.name.contains("single_elem_vec"))
            .unwrap();

        // Empty vec: length 0 (u32)
        assert_eq!(empty_vec.data, vec![0, 0, 0, 0]);

        // Single elem: length 1 + 1 byte for u8
        assert_eq!(single_elem.data, vec![1, 0, 0, 0, 0]);
    }

    #[test]
    fn test_generates_enum_corpus() {
        let type_defs = vec![TypeDefinition::Enum(EnumDefinition {
            name: "SimpleEnum".to_string(),
            generic_params: vec![],
            variants: vec![
                EnumVariantDefinition::Unit {
                    name: "Variant1".to_string(),
                },
                EnumVariantDefinition::Tuple {
                    name: "Variant2".to_string(),
                    types: vec![TypeInfo::Primitive("u32".to_string())],
                },
            ],
            metadata: Metadata::default(),
        })];

        let generator = CorpusGenerator::new(&type_defs);
        let corpus = generator.generate_all();

        assert_eq!(corpus.len(), 2); // One per variant

        // Variant1: discriminant 0 (u32)
        assert_eq!(corpus[0].data, vec![0, 0, 0, 0]);

        // Variant2: discriminant 1 + u32 value
        assert_eq!(corpus[1].data.len(), 8);
        assert_eq!(&corpus[1].data[0..4], &[1, 0, 0, 0]); // discriminant
    }
}
