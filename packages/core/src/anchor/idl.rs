// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Anchor IDL Generation
//!
//! Generates Anchor IDL JSON from LUMOS type definitions.
//! The IDL format follows the Anchor framework specification.

use crate::ir::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, StructDefinition, TypeDefinition,
    TypeInfo,
};
use serde::{Deserialize, Serialize};

/// Anchor IDL root structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idl {
    /// IDL version (semantic versioning)
    pub version: String,

    /// Program name (snake_case)
    pub name: String,

    /// Program instructions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instructions: Vec<IdlInstruction>,

    /// Account types (structs with #[account])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accounts: Vec<IdlTypeDef>,

    /// Custom types (non-account structs and enums)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<IdlTypeDef>,

    /// Program events
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<IdlEvent>,

    /// Program errors
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<IdlError>,

    /// Program metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<IdlMetadata>,
}

/// IDL instruction definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlInstruction {
    /// Instruction name (snake_case)
    pub name: String,

    /// Instruction accounts
    #[serde(default)]
    pub accounts: Vec<IdlAccountItem>,

    /// Instruction arguments
    #[serde(default)]
    pub args: Vec<IdlField>,
}

/// IDL account item (can be single account or nested accounts)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IdlAccountItem {
    /// Single account
    Single(IdlAccount),

    /// Composite (nested) accounts
    Composite(IdlAccountComposite),
}

/// Single account in an instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlAccount {
    /// Account name
    pub name: String,

    /// Whether account is mutable
    #[serde(rename = "isMut")]
    pub is_mut: bool,

    /// Whether account is a signer
    #[serde(rename = "isSigner")]
    pub is_signer: bool,

    /// Whether account is optional
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "isOptional"
    )]
    pub is_optional: Option<bool>,

    /// Account documentation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub docs: Vec<String>,

    /// PDA seeds (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pda: Option<IdlPda>,
}

/// Composite accounts (e.g., CPI accounts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlAccountComposite {
    /// Composite name
    pub name: String,

    /// Nested accounts
    pub accounts: Vec<IdlAccountItem>,
}

/// PDA (Program Derived Address) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlPda {
    /// Seeds for PDA derivation
    pub seeds: Vec<IdlSeed>,

    /// Program ID (if different from current program)
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "programId")]
    pub program_id: Option<String>,
}

/// PDA seed definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IdlSeed {
    /// Constant seed (literal bytes)
    #[serde(rename = "const")]
    Const { value: String },

    /// Account-derived seed
    #[serde(rename = "account")]
    Account { path: String },

    /// Argument-derived seed
    #[serde(rename = "arg")]
    Arg { path: String },
}

/// Type definition in IDL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlTypeDef {
    /// Type name (PascalCase)
    pub name: String,

    /// Type documentation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub docs: Vec<String>,

    /// Type definition (struct or enum)
    #[serde(rename = "type")]
    pub ty: IdlTypeDefTy,
}

/// Type definition body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IdlTypeDefTy {
    /// Struct type
    #[serde(rename = "struct")]
    Struct {
        /// Struct fields
        fields: Vec<IdlField>,
    },

    /// Enum type
    #[serde(rename = "enum")]
    Enum {
        /// Enum variants
        variants: Vec<IdlEnumVariant>,
    },
}

/// Field in a struct or instruction args
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlField {
    /// Field name (snake_case)
    pub name: String,

    /// Field type
    #[serde(rename = "type")]
    pub ty: IdlType,

    /// Field documentation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub docs: Vec<String>,
}

/// Enum variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlEnumVariant {
    /// Variant name (PascalCase)
    pub name: String,

    /// Variant fields (for struct/tuple variants)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<IdlEnumFields>,
}

/// Enum variant fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IdlEnumFields {
    /// Named fields (struct variant)
    Named(Vec<IdlField>),

    /// Unnamed fields (tuple variant)
    Tuple(Vec<IdlType>),
}

/// IDL type representation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IdlType {
    /// Primitive type (string representation)
    Primitive(String),

    /// Array type
    Array(IdlTypeArray),

    /// Option type
    Option(IdlTypeOption),

    /// Vec type
    Vec(IdlTypeVec),

    /// Defined type (reference to another type)
    Defined(IdlTypeDefined),
}

/// Array type wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlTypeArray {
    /// Array element type and size
    pub array: (Box<IdlType>, usize),
}

/// Option type wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlTypeOption {
    /// Inner type
    pub option: Box<IdlType>,
}

/// Vec type wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlTypeVec {
    /// Element type
    pub vec: Box<IdlType>,
}

/// Defined type reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlTypeDefined {
    /// Type name
    pub defined: String,
}

/// Program event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlEvent {
    /// Event name
    pub name: String,

    /// Event fields
    pub fields: Vec<IdlEventField>,
}

/// Event field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlEventField {
    /// Field name
    pub name: String,

    /// Field type
    #[serde(rename = "type")]
    pub ty: IdlType,

    /// Whether field is indexed
    pub index: bool,
}

/// Program error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlError {
    /// Error code
    pub code: u32,

    /// Error name
    pub name: String,

    /// Error message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
}

/// IDL metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlMetadata {
    /// Program address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

/// Configuration for IDL generation
#[derive(Debug, Clone)]
pub struct IdlGeneratorConfig {
    /// Program name (will be converted to snake_case)
    pub program_name: String,

    /// Program version (semantic versioning)
    pub version: String,

    /// Optional program address
    pub address: Option<String>,
}

impl Default for IdlGeneratorConfig {
    fn default() -> Self {
        Self {
            program_name: "my_program".to_string(),
            version: "0.1.0".to_string(),
            address: None,
        }
    }
}

/// IDL generator from LUMOS type definitions
pub struct IdlGenerator {
    config: IdlGeneratorConfig,
}

impl IdlGenerator {
    /// Create a new IDL generator with the given configuration
    pub fn new(config: IdlGeneratorConfig) -> Self {
        Self { config }
    }

    /// Generate an Anchor IDL from type definitions
    pub fn generate(&self, type_defs: &[TypeDefinition]) -> Idl {
        let mut accounts = Vec::new();
        let mut types = Vec::new();

        for type_def in type_defs {
            match type_def {
                TypeDefinition::Struct(struct_def) => {
                    let idl_type = self.convert_struct(struct_def);
                    if self.is_account_type(struct_def) {
                        accounts.push(idl_type);
                    } else {
                        types.push(idl_type);
                    }
                }
                TypeDefinition::Enum(enum_def) => {
                    types.push(self.convert_enum(enum_def));
                }
                TypeDefinition::TypeAlias(_) => {
                    // Type aliases are resolved during transformation,
                    // they don't appear in the IDL
                }
            }
        }

        Idl {
            version: self.config.version.clone(),
            name: to_snake_case(&self.config.program_name),
            instructions: Vec::new(), // Instructions are parsed separately
            accounts,
            types,
            events: Vec::new(),
            errors: Vec::new(),
            metadata: self.config.address.as_ref().map(|addr| IdlMetadata {
                address: Some(addr.clone()),
            }),
        }
    }

    /// Convert a struct definition to IDL type definition
    fn convert_struct(&self, struct_def: &StructDefinition) -> IdlTypeDef {
        let fields = struct_def
            .fields
            .iter()
            .map(|f| self.convert_field(f))
            .collect();

        IdlTypeDef {
            name: struct_def.name.clone(),
            docs: Vec::new(),
            ty: IdlTypeDefTy::Struct { fields },
        }
    }

    /// Convert an enum definition to IDL type definition
    fn convert_enum(&self, enum_def: &EnumDefinition) -> IdlTypeDef {
        let variants = enum_def
            .variants
            .iter()
            .map(|v| self.convert_variant(v))
            .collect();

        IdlTypeDef {
            name: enum_def.name.clone(),
            docs: Vec::new(),
            ty: IdlTypeDefTy::Enum { variants },
        }
    }

    /// Convert a field definition to IDL field
    fn convert_field(&self, field: &FieldDefinition) -> IdlField {
        // Handle optional fields - avoid double-wrapping if type is already Option
        let ty = if field.optional && !matches!(field.type_info, TypeInfo::Option(_)) {
            IdlType::Option(IdlTypeOption {
                option: Box::new(self.convert_type(&field.type_info)),
            })
        } else {
            self.convert_type(&field.type_info)
        };

        IdlField {
            name: to_snake_case(&field.name),
            ty,
            docs: field
                .deprecated
                .as_ref()
                .map(|msg| vec![format!("@deprecated {}", msg)])
                .unwrap_or_default(),
        }
    }

    /// Convert an enum variant to IDL variant
    fn convert_variant(&self, variant: &EnumVariantDefinition) -> IdlEnumVariant {
        match variant {
            EnumVariantDefinition::Unit { name } => IdlEnumVariant {
                name: name.clone(),
                fields: None,
            },
            EnumVariantDefinition::Tuple { name, types } => IdlEnumVariant {
                name: name.clone(),
                fields: Some(IdlEnumFields::Tuple(
                    types.iter().map(|t| self.convert_type(t)).collect(),
                )),
            },
            EnumVariantDefinition::Struct { name, fields } => IdlEnumVariant {
                name: name.clone(),
                fields: Some(IdlEnumFields::Named(
                    fields.iter().map(|f| self.convert_field(f)).collect(),
                )),
            },
        }
    }

    /// Convert a LUMOS type to IDL type
    fn convert_type(&self, type_info: &TypeInfo) -> IdlType {
        convert_type_to_idl(type_info)
    }

    /// Check if a struct is an account type (has #[account] attribute)
    fn is_account_type(&self, struct_def: &StructDefinition) -> bool {
        struct_def
            .metadata
            .attributes
            .iter()
            .any(|attr| attr == "account")
    }

    /// Calculate account space (with 8-byte discriminator)
    pub fn calculate_account_space(&self, struct_def: &StructDefinition) -> usize {
        let mut size = 8; // Anchor discriminator

        for field in &struct_def.fields {
            size += calculate_type_size(&field.type_info, field.optional);
        }

        size
    }

    /// Generate LEN constant for a struct
    pub fn generate_len_constant(&self, struct_def: &StructDefinition) -> String {
        let space = self.calculate_account_space(struct_def);
        format!(
            "impl {} {{\n    pub const LEN: usize = {};\n}}",
            struct_def.name, space
        )
    }
}

/// Convert a LUMOS type to IDL type (standalone function for recursion)
fn convert_type_to_idl(type_info: &TypeInfo) -> IdlType {
    match type_info {
        TypeInfo::Primitive(name) => {
            let idl_type = match name.as_str() {
                "u8" => "u8",
                "u16" => "u16",
                "u32" => "u32",
                "u64" => "u64",
                "u128" => "u128",
                "i8" => "i8",
                "i16" => "i16",
                "i32" => "i32",
                "i64" => "i64",
                "i128" => "i128",
                "f32" => "f32",
                "f64" => "f64",
                "bool" => "bool",
                "String" | "string" => "string",
                "PublicKey" | "Pubkey" => "publicKey",
                "Signature" => "string", // Signatures are typically base58 strings in IDL
                _ => name.as_str(),
            };
            IdlType::Primitive(idl_type.to_string())
        }
        TypeInfo::Generic(name) => {
            // Generic types are treated as defined types in IDL
            IdlType::Defined(IdlTypeDefined {
                defined: name.clone(),
            })
        }
        TypeInfo::UserDefined(name) => IdlType::Defined(IdlTypeDefined {
            defined: name.clone(),
        }),
        TypeInfo::Array(inner) => IdlType::Vec(IdlTypeVec {
            vec: Box::new(convert_type_to_idl(inner)),
        }),
        TypeInfo::FixedArray { element, size } => IdlType::Array(IdlTypeArray {
            array: (Box::new(convert_type_to_idl(element)), *size),
        }),
        TypeInfo::Option(inner) => IdlType::Option(IdlTypeOption {
            option: Box::new(convert_type_to_idl(inner)),
        }),
    }
}

/// Calculate the size of a type in bytes (standalone function for recursion)
fn calculate_type_size(type_info: &TypeInfo, optional: bool) -> usize {
    let base_size = match type_info {
        TypeInfo::Primitive(name) => match name.as_str() {
            "u8" | "i8" | "bool" => 1,
            "u16" | "i16" => 2,
            "u32" | "i32" | "f32" => 4,
            "u64" | "i64" | "f64" => 8,
            "u128" | "i128" => 16,
            "PublicKey" | "Pubkey" => 32,
            "Signature" => 64,
            "String" | "string" => 4, // Only the length prefix, actual content is variable
            _ => 0,
        },
        TypeInfo::Generic(_) => 0,     // Generic types have unknown size
        TypeInfo::UserDefined(_) => 0, // User-defined types need separate calculation
        TypeInfo::Array(inner) => 4 + calculate_type_size(inner, false), // Vec prefix + content
        TypeInfo::FixedArray { element, size } => calculate_type_size(element, false) * size,
        TypeInfo::Option(inner) => 1 + calculate_type_size(inner, false),
    };

    if optional && !matches!(type_info, TypeInfo::Option(_)) {
        1 + base_size // Option tag
    } else {
        base_size
    }
}

/// Convert a string to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_lower = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if prev_lower || (i > 0 && !result.ends_with('_')) {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_lower = false;
        } else {
            result.push(c);
            prev_lower = c.is_lowercase();
        }
    }

    // Clean up leading underscore
    if result.starts_with('_') {
        result.remove(0);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Metadata, Visibility};

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("PlayerAccount"), "player_account");
        assert_eq!(to_snake_case("MyProgram"), "my_program");
        assert_eq!(to_snake_case("NFTMarketplace"), "n_f_t_marketplace");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn test_generate_simple_idl() {
        let struct_def = StructDefinition {
            name: "PlayerAccount".to_string(),
            generic_params: vec![],
            fields: vec![
                FieldDefinition {
                    name: "authority".to_string(),
                    type_info: TypeInfo::Primitive("PublicKey".to_string()),
                    optional: false,
                    deprecated: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "level".to_string(),
                    type_info: TypeInfo::Primitive("u16".to_string()),
                    optional: false,
                    deprecated: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "experience".to_string(),
                    type_info: TypeInfo::Primitive("u64".to_string()),
                    optional: false,
                    deprecated: None,
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
        };

        let config = IdlGeneratorConfig {
            program_name: "MyGame".to_string(),
            version: "0.1.0".to_string(),
            address: None,
        };

        let generator = IdlGenerator::new(config);
        let type_defs = vec![TypeDefinition::Struct(struct_def)];
        let idl = generator.generate(&type_defs);

        assert_eq!(idl.name, "my_game");
        assert_eq!(idl.version, "0.1.0");
        assert_eq!(idl.accounts.len(), 1);
        assert_eq!(idl.accounts[0].name, "PlayerAccount");
    }

    #[test]
    fn test_calculate_account_space() {
        let struct_def = StructDefinition {
            name: "PlayerAccount".to_string(),
            generic_params: vec![],
            fields: vec![
                FieldDefinition {
                    name: "authority".to_string(),
                    type_info: TypeInfo::Primitive("PublicKey".to_string()),
                    optional: false,
                    deprecated: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "level".to_string(),
                    type_info: TypeInfo::Primitive("u16".to_string()),
                    optional: false,
                    deprecated: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "experience".to_string(),
                    type_info: TypeInfo::Primitive("u64".to_string()),
                    optional: false,
                    deprecated: None,
                    anchor_attrs: vec![],
                },
            ],
            metadata: Metadata::default(),
            visibility: Visibility::Public,

            module_path: Vec::new(),
        };

        let generator = IdlGenerator::new(IdlGeneratorConfig::default());
        let space = generator.calculate_account_space(&struct_def);

        // 8 (discriminator) + 32 (PublicKey) + 2 (u16) + 8 (u64) = 50
        assert_eq!(space, 50);
    }

    #[test]
    fn test_convert_enum() {
        let enum_def = EnumDefinition {
            name: "GameState".to_string(),
            generic_params: vec![],
            variants: vec![
                EnumVariantDefinition::Unit {
                    name: "Active".to_string(),
                },
                EnumVariantDefinition::Unit {
                    name: "Paused".to_string(),
                },
                EnumVariantDefinition::Struct {
                    name: "Finished".to_string(),
                    fields: vec![FieldDefinition {
                        name: "winner".to_string(),
                        type_info: TypeInfo::Primitive("PublicKey".to_string()),
                        optional: false,
                        deprecated: None,
                        anchor_attrs: vec![],
                    }],
                },
            ],
            metadata: Metadata::default(),
            visibility: Visibility::Public,

            module_path: Vec::new(),
        };

        let config = IdlGeneratorConfig::default();
        let generator = IdlGenerator::new(config);
        let type_defs = vec![TypeDefinition::Enum(enum_def)];
        let idl = generator.generate(&type_defs);

        assert_eq!(idl.types.len(), 1);
        assert_eq!(idl.types[0].name, "GameState");

        if let IdlTypeDefTy::Enum { variants } = &idl.types[0].ty {
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].name, "Active");
            assert!(variants[0].fields.is_none());
            assert_eq!(variants[2].name, "Finished");
            assert!(variants[2].fields.is_some());
        } else {
            panic!("Expected enum type");
        }
    }

    #[test]
    fn test_idl_json_serialization() {
        let idl = Idl {
            version: "0.1.0".to_string(),
            name: "test_program".to_string(),
            instructions: vec![],
            accounts: vec![IdlTypeDef {
                name: "TestAccount".to_string(),
                docs: vec![],
                ty: IdlTypeDefTy::Struct {
                    fields: vec![IdlField {
                        name: "value".to_string(),
                        ty: IdlType::Primitive("u64".to_string()),
                        docs: vec![],
                    }],
                },
            }],
            types: vec![],
            events: vec![],
            errors: vec![],
            metadata: None,
        };

        let json = serde_json::to_string_pretty(&idl).unwrap();
        assert!(json.contains("\"version\": \"0.1.0\""));
        assert!(json.contains("\"name\": \"test_program\""));
        assert!(json.contains("\"TestAccount\""));
    }

    #[test]
    fn test_type_conversion() {
        let generator = IdlGenerator::new(IdlGeneratorConfig::default());

        // Test primitive types
        let pubkey = generator.convert_type(&TypeInfo::Primitive("PublicKey".to_string()));
        assert!(matches!(pubkey, IdlType::Primitive(s) if s == "publicKey"));

        // Test array type
        let array = generator.convert_type(&TypeInfo::FixedArray {
            element: Box::new(TypeInfo::Primitive("u8".to_string())),
            size: 32,
        });
        assert!(matches!(array, IdlType::Array(_)));

        // Test option type
        let option = generator.convert_type(&TypeInfo::Option(Box::new(TypeInfo::Primitive(
            "u64".to_string(),
        ))));
        assert!(matches!(option, IdlType::Option(_)));

        // Test vec type
        let vec = generator.convert_type(&TypeInfo::Array(Box::new(TypeInfo::Primitive(
            "u8".to_string(),
        ))));
        assert!(matches!(vec, IdlType::Vec(_)));
    }

    #[test]
    fn test_generate_len_constant() {
        let struct_def = StructDefinition {
            name: "MyAccount".to_string(),
            generic_params: vec![],
            fields: vec![
                FieldDefinition {
                    name: "owner".to_string(),
                    type_info: TypeInfo::Primitive("PublicKey".to_string()),
                    optional: false,
                    deprecated: None,
                    anchor_attrs: vec![],
                },
                FieldDefinition {
                    name: "amount".to_string(),
                    type_info: TypeInfo::Primitive("u64".to_string()),
                    optional: false,
                    deprecated: None,
                    anchor_attrs: vec![],
                },
            ],
            metadata: Metadata::default(),
            visibility: Visibility::Public,

            module_path: Vec::new(),
        };

        let generator = IdlGenerator::new(IdlGeneratorConfig::default());
        let len_const = generator.generate_len_constant(&struct_def);

        assert!(len_const.contains("impl MyAccount"));
        assert!(len_const.contains("pub const LEN: usize = 48")); // 8 + 32 + 8
    }
}
