// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Comprehensive error path testing for LUMOS core
//!
//! This test suite ensures that all error scenarios are handled gracefully
//! and produce clear, actionable error messages.

use lumos_core::parser::parse_lumos_file;
use lumos_core::transform::transform_to_ir;

/// Test parser error handling for invalid syntax
#[cfg(test)]
mod parser_errors {
    use super::*;

    #[test]
    fn test_unclosed_brace() {
        let input = r#"
            struct Player {
                name: String,
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_err(), "Should fail on unclosed brace");
    }

    #[test]
    fn test_invalid_identifier_starting_with_number() {
        let input = r#"
            struct 123Invalid {
                field: u64,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(
            result.is_err(),
            "Should fail on identifier starting with number"
        );
    }

    #[test]
    fn test_missing_type_annotation() {
        let input = r#"
            struct Player {
                name
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_err(), "Should fail on missing type annotation");
    }

    #[test]
    fn test_invalid_attribute_syntax() {
        let input = r#"
            #[solana
            struct Player {
                name: String,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_err(), "Should fail on malformed attribute");
    }

    #[test]
    fn test_missing_struct_keyword() {
        let input = r#"
            Player {
                name: String,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_err(), "Should fail on missing struct keyword");
    }

    #[test]
    fn test_empty_struct_body() {
        let input = r#"
            struct Player {}
        "#;

        // This should actually succeed - empty structs are valid
        let result = parse_lumos_file(input);
        assert!(result.is_ok(), "Empty structs should be valid");
    }

    #[test]
    fn test_invalid_field_separator() {
        let input = r#"
            struct Player {
                name: String;
                level: u16;
            }
        "#;

        let result = parse_lumos_file(input);
        // Semicolons instead of commas - should fail
        assert!(result.is_err(), "Should fail on invalid field separator");
    }

    #[test]
    fn test_unclosed_bracket_in_array() {
        let input = r#"
            struct Inventory {
                items: [u64,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_err(), "Should fail on unclosed bracket");
    }

    #[test]
    fn test_invalid_enum_variant_syntax() {
        let input = r#"
            enum GameState {
                Active,
                Paused
                Finished,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_err(), "Should fail on missing comma");
    }

    #[test]
    fn test_reserved_rust_keyword_as_field_name() {
        let input = r#"
            struct Player {
                type: u64,
            }
        "#;

        // This might actually parse but could cause issues in generation
        // Let's test that it at least parses
        let result = parse_lumos_file(input);
        // Note: Parser might allow this, but generator should handle it
        let _ = result; // We'll check this separately
    }

    #[test]
    fn test_nested_option_option() {
        let input = r#"
            struct Player {
                data: Option<Option<u64>>,
            }
        "#;

        let result = parse_lumos_file(input);
        // This should parse but might not be semantically valid
        assert!(result.is_ok(), "Parser should accept nested Option");
    }

    #[test]
    fn test_invalid_type_in_array() {
        let input = r#"
            struct Inventory {
                items: [InvalidType],
            }
        "#;

        let ast = parse_lumos_file(input);
        assert!(ast.is_ok(), "Parser accepts unknown types");

        // But transform should catch it
        let result = transform_to_ir(ast.unwrap());
        assert!(result.is_err(), "Transform should catch undefined type");
    }
}

/// Test transform error handling
#[cfg(test)]
mod transform_errors {
    use super::*;

    #[test]
    fn test_undefined_type_in_field() {
        let input = r#"
            struct Player {
                inventory: UndefinedInventory,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("UndefinedInventory"));
    }

    #[test]
    fn test_undefined_type_in_enum_tuple() {
        let input = r#"
            enum GameEvent {
                PlayerJoined(NonexistentPlayer),
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("NonexistentPlayer"));
    }

    #[test]
    fn test_undefined_type_in_enum_struct() {
        let input = r#"
            enum Instruction {
                Initialize {
                    config: MissingConfig,
                },
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("MissingConfig"));
    }

    #[test]
    fn test_undefined_type_in_nested_array() {
        let input = r#"
            struct Container {
                items: [UndefinedItem],
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("UndefinedItem"));
    }

    #[test]
    fn test_undefined_type_in_option() {
        let input = r#"
            struct Player {
                profile: Option<MissingProfile>,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("MissingProfile"));
    }

    #[test]
    fn test_deeply_nested_undefined_type() {
        let input = r#"
            struct Container {
                data: Option<[UndefinedType]>,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("UndefinedType"));
    }

    #[test]
    fn test_multiple_undefined_types() {
        let input = r#"
            struct Player {
                inventory: Inventory,
                stats: Stats,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        // Should fail on first undefined type
        assert!(result.is_err());
    }

    #[test]
    fn test_circular_reference_detection() {
        // Note: This is a future feature - circular references
        // For now, we just test that the types are recognized
        let input = r#"
            struct Node {
                next: Option<Node>,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        // Should fail because Node references itself but isn't defined yet
        // Actually, it might work because we collect all types first
        // This is edge case behavior that depends on implementation
        let _ = result; // Just ensure it doesn't panic
    }
}

/// Test generator error handling
#[cfg(test)]
mod generator_errors {
    use lumos_core::generators::{rust, typescript};
    use lumos_core::ir::{
        EnumDefinition, EnumVariantDefinition, FieldDefinition, Metadata, StructDefinition,
        TypeDefinition, TypeInfo,
    };

    #[test]
    fn test_empty_struct_generation() {
        let empty_struct = StructDefinition {
            name: "Empty".to_string(),
            fields: vec![],
            metadata: Metadata::default(),
        };

        let type_def = TypeDefinition::Struct(empty_struct);

        // Should handle empty structs gracefully
        let rust_code = rust::generate_module(std::slice::from_ref(&type_def));
        let ts_code = typescript::generate_module(std::slice::from_ref(&type_def));

        assert!(rust_code.contains("struct Empty"));
        assert!(ts_code.contains("interface Empty"));
    }

    #[test]
    fn test_enum_with_no_variants() {
        // This is an edge case - enums should have at least one variant
        let empty_enum = EnumDefinition {
            name: "EmptyEnum".to_string(),
            variants: vec![],
            metadata: Metadata::default(),
        };

        let type_def = TypeDefinition::Enum(empty_enum);

        // Should handle gracefully without panicking
        let rust_code = rust::generate_module(std::slice::from_ref(&type_def));
        let ts_code = typescript::generate_module(std::slice::from_ref(&type_def));

        assert!(rust_code.contains("enum EmptyEnum"));
        assert!(ts_code.contains("type EmptyEnum"));
    }

    #[test]
    fn test_deeply_nested_types_generation() {
        // Test generation of deeply nested Option<[Option<T>]>
        let nested_field = FieldDefinition {
            name: "nested".to_string(),
            type_info: TypeInfo::Option(Box::new(TypeInfo::Array(Box::new(TypeInfo::Option(
                Box::new(TypeInfo::Primitive("u64".to_string())),
            ))))),
            optional: true,
            deprecated: None,
        };

        let struct_def = StructDefinition {
            name: "Nested".to_string(),
            fields: vec![nested_field],
            metadata: Metadata::default(),
        };

        let type_def = TypeDefinition::Struct(struct_def);

        // Should handle complex nesting without stack overflow
        let rust_code = rust::generate_module(std::slice::from_ref(&type_def));
        let ts_code = typescript::generate_module(std::slice::from_ref(&type_def));

        assert!(!rust_code.is_empty());
        assert!(!ts_code.is_empty());
    }

    #[test]
    fn test_mixed_enum_variants() {
        // Test enum with all three variant types
        let mixed_enum = EnumDefinition {
            name: "MixedEnum".to_string(),
            variants: vec![
                EnumVariantDefinition::Unit {
                    name: "Empty".to_string(),
                },
                EnumVariantDefinition::Tuple {
                    name: "WithData".to_string(),
                    types: vec![TypeInfo::Primitive("u64".to_string())],
                },
                EnumVariantDefinition::Struct {
                    name: "WithFields".to_string(),
                    fields: vec![FieldDefinition {
                        name: "value".to_string(),
                        type_info: TypeInfo::Primitive("String".to_string()),
                        optional: false,
                        deprecated: None,
                    }],
                },
            ],
            metadata: Metadata::default(),
        };

        let type_def = TypeDefinition::Enum(mixed_enum);

        let rust_code = rust::generate_module(std::slice::from_ref(&type_def));
        let ts_code = typescript::generate_module(std::slice::from_ref(&type_def));

        // Should generate all variant types correctly
        assert!(rust_code.contains("Empty"));
        assert!(rust_code.contains("WithData"));
        assert!(rust_code.contains("WithFields"));

        assert!(ts_code.contains("kind: 'Empty'"));
        assert!(ts_code.contains("kind: 'WithData'"));
        assert!(ts_code.contains("kind: 'WithFields'"));
    }
}

/// Test edge cases and boundary conditions
#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_extremely_long_identifier() {
        let long_name = "A".repeat(1000);
        let input = format!(
            r#"
            struct {} {{
                field: u64,
            }}
        "#,
            long_name
        );

        // Should handle long identifiers
        let result = parse_lumos_file(&input);
        let _ = result; // Just ensure no panic
    }

    #[test]
    fn test_unicode_in_identifiers() {
        let input = r#"
            struct Player {
                naïve: u64,
            }
        "#;

        // Rust identifiers support some Unicode
        let result = parse_lumos_file(input);
        // This depends on syn's identifier rules
        let _ = result;
    }

    #[test]
    fn test_extremely_nested_types() {
        // Create deeply nested Option types
        let input = r#"
            struct Nested {
                data: Option<Option<Option<Option<u64>>>>,
            }
        "#;

        let ast = parse_lumos_file(input).unwrap();
        let result = transform_to_ir(ast);

        // Should handle deep nesting
        assert!(result.is_ok());
    }

    #[test]
    fn test_whitespace_variations() {
        let inputs = vec![
            "struct Player{name:String,}",                 // No spaces
            "struct    Player   {  name  :  String  ,  }", // Extra spaces
            "struct\nPlayer\n{\nname:\nString,\n}",        // Newlines
        ];

        for input in inputs {
            let result = parse_lumos_file(input);
            assert!(result.is_ok(), "Should handle whitespace variations");
        }
    }

    #[test]
    fn test_trailing_comma_in_struct() {
        let input = r#"
            struct Player {
                name: String,
                level: u16,
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok(), "Trailing commas should be allowed");
    }

    #[test]
    fn test_no_trailing_comma_in_struct() {
        let input = r#"
            struct Player {
                name: String,
                level: u16
            }
        "#;

        let result = parse_lumos_file(input);
        assert!(result.is_ok(), "No trailing comma should be allowed");
    }
}
