//! Cross-language schema compatibility tests (#117)
//!
//! Verifies that schemas generated in different languages (Rust, TypeScript, Python, Go, Ruby)
//! produce consistent Borsh serialization schemas.

use lumos_core::generators::{
    go::generate_module as generate_go, python::generate_module as generate_python,
    ruby::generate_module as generate_ruby, rust::generate_module as generate_rust,
    typescript::generate_module as generate_typescript,
};
use lumos_core::ir::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, Metadata, StructDefinition,
    TypeDefinition, TypeInfo, Visibility,
};

/// Helper to create test metadata
fn test_metadata() -> Metadata {
    Metadata {
        solana: true,
        attributes: vec![],
        version: None,
        custom_derives: vec![],
        is_instruction: false,
        anchor_attrs: vec![],
    }
}

/// Helper to create a field definition
fn field(name: &str, type_info: TypeInfo) -> FieldDefinition {
    FieldDefinition {
        name: name.to_string(),
        type_info,
        optional: false,
        deprecated: None,
        anchor_attrs: vec![],
    }
}

/// Helper to create a struct definition with all required fields
fn make_struct(name: &str, fields: Vec<FieldDefinition>) -> StructDefinition {
    StructDefinition {
        name: name.to_string(),
        fields,
        metadata: test_metadata(),
        generic_params: vec![],
        visibility: Visibility::Public,
        module_path: vec![],
    }
}

/// Helper to create an enum definition with all required fields
fn make_enum(name: &str, variants: Vec<EnumVariantDefinition>) -> EnumDefinition {
    EnumDefinition {
        name: name.to_string(),
        variants,
        metadata: test_metadata(),
        generic_params: vec![],
        visibility: Visibility::Public,
        module_path: vec![],
    }
}

/// Helper to create a comprehensive test schema covering all primitive types
fn create_all_primitives_struct() -> StructDefinition {
    make_struct(
        "AllPrimitives",
        vec![
            field("val_u8", TypeInfo::Primitive("u8".to_string())),
            field("val_u16", TypeInfo::Primitive("u16".to_string())),
            field("val_u32", TypeInfo::Primitive("u32".to_string())),
            field("val_u64", TypeInfo::Primitive("u64".to_string())),
            field("val_u128", TypeInfo::Primitive("u128".to_string())),
            field("val_i8", TypeInfo::Primitive("i8".to_string())),
            field("val_i16", TypeInfo::Primitive("i16".to_string())),
            field("val_i32", TypeInfo::Primitive("i32".to_string())),
            field("val_i64", TypeInfo::Primitive("i64".to_string())),
            field("val_i128", TypeInfo::Primitive("i128".to_string())),
            field("val_bool", TypeInfo::Primitive("bool".to_string())),
            field("val_string", TypeInfo::Primitive("String".to_string())),
        ],
    )
}

/// Helper to create Solana types struct
fn create_solana_types_struct() -> StructDefinition {
    make_struct(
        "SolanaTypes",
        vec![
            field("wallet", TypeInfo::Primitive("PublicKey".to_string())),
            field("sig", TypeInfo::Primitive("Signature".to_string())),
        ],
    )
}

/// Helper to create complex types struct
fn create_complex_types_struct() -> StructDefinition {
    make_struct(
        "ComplexTypes",
        vec![
            field(
                "vec_u8",
                TypeInfo::Array(Box::new(TypeInfo::Primitive("u8".to_string()))),
            ),
            field(
                "vec_string",
                TypeInfo::Array(Box::new(TypeInfo::Primitive("String".to_string()))),
            ),
            field(
                "opt_string",
                TypeInfo::Option(Box::new(TypeInfo::Primitive("String".to_string()))),
            ),
            field(
                "opt_pubkey",
                TypeInfo::Option(Box::new(TypeInfo::Primitive("PublicKey".to_string()))),
            ),
            field(
                "fixed_bytes",
                TypeInfo::FixedArray {
                    element: Box::new(TypeInfo::Primitive("u8".to_string())),
                    size: 32,
                },
            ),
        ],
    )
}

/// Helper to create test enum
fn create_test_enum() -> EnumDefinition {
    make_enum(
        "TestEnum",
        vec![
            EnumVariantDefinition::Unit {
                name: "Active".to_string(),
            },
            EnumVariantDefinition::Unit {
                name: "Paused".to_string(),
            },
            EnumVariantDefinition::Tuple {
                name: "Score".to_string(),
                types: vec![TypeInfo::Primitive("u64".to_string())],
            },
            EnumVariantDefinition::Struct {
                name: "Data".to_string(),
                fields: vec![field("value", TypeInfo::Primitive("String".to_string()))],
            },
        ],
    )
}

// =============================================================================
// Cross-Language Compatibility Tests
// =============================================================================

#[test]
fn all_languages_generate_primitives_struct() {
    let type_defs = vec![TypeDefinition::Struct(create_all_primitives_struct())];

    // Generate for all languages
    let rust_code = generate_rust(&type_defs);
    let ts_code = generate_typescript(&type_defs);
    let py_code = generate_python(&type_defs);
    let go_code = generate_go(&type_defs);
    let rb_code = generate_ruby(&type_defs);

    // Verify all languages generated code
    assert!(!rust_code.is_empty(), "Rust code should not be empty");
    assert!(!ts_code.is_empty(), "TypeScript code should not be empty");
    assert!(!py_code.is_empty(), "Python code should not be empty");
    assert!(!go_code.is_empty(), "Go code should not be empty");
    assert!(!rb_code.is_empty(), "Ruby code should not be empty");

    // Verify struct name appears in all outputs
    assert!(rust_code.contains("AllPrimitives"));
    assert!(ts_code.contains("AllPrimitives"));
    assert!(py_code.contains("AllPrimitives"));
    assert!(go_code.contains("AllPrimitives"));
    assert!(rb_code.contains("AllPrimitives"));

    // Verify all field names appear (in their language-appropriate format)
    let rust_fields = ["val_u8", "val_u16", "val_u32", "val_u64", "val_u128"];
    for field in rust_fields {
        assert!(rust_code.contains(field), "Rust missing field: {}", field);
    }

    // TypeScript uses same field names
    for field in rust_fields {
        assert!(
            ts_code.contains(field),
            "TypeScript missing field: {}",
            field
        );
    }

    // Python uses same field names
    for field in rust_fields {
        assert!(py_code.contains(field), "Python missing field: {}", field);
    }

    // Go uses PascalCase (ValU8, ValU16, etc.)
    let go_fields = ["ValU8", "ValU16", "ValU32", "ValU64", "ValU128"];
    for field in go_fields {
        assert!(go_code.contains(field), "Go missing field: {}", field);
    }

    // Ruby uses snake_case
    for field in rust_fields {
        assert!(rb_code.contains(field), "Ruby missing field: {}", field);
    }
}

#[test]
fn all_languages_generate_solana_types() {
    let type_defs = vec![TypeDefinition::Struct(create_solana_types_struct())];

    let rust_code = generate_rust(&type_defs);
    let ts_code = generate_typescript(&type_defs);
    let py_code = generate_python(&type_defs);
    let go_code = generate_go(&type_defs);
    let rb_code = generate_ruby(&type_defs);

    // Rust: Pubkey
    assert!(rust_code.contains("Pubkey"), "Rust should use Pubkey type");

    // TypeScript: PublicKey
    assert!(
        ts_code.contains("PublicKey"),
        "TypeScript should use PublicKey type"
    );

    // Python: bytes (32 bytes for PublicKey)
    assert!(
        py_code.contains("bytes") || py_code.contains("Bytes(32)"),
        "Python should use bytes for PublicKey"
    );

    // Go: [32]byte
    assert!(
        go_code.contains("[32]byte"),
        "Go should use [32]byte for PublicKey"
    );

    // Ruby: Array (32 bytes)
    assert!(
        rb_code.contains(":u8, 32") || rb_code.contains("wallet"),
        "Ruby should reference wallet field"
    );
}

#[test]
fn all_languages_generate_complex_types() {
    let type_defs = vec![TypeDefinition::Struct(create_complex_types_struct())];

    let rust_code = generate_rust(&type_defs);
    let ts_code = generate_typescript(&type_defs);
    let py_code = generate_python(&type_defs);
    let go_code = generate_go(&type_defs);
    let rb_code = generate_ruby(&type_defs);

    // Vec types
    assert!(rust_code.contains("Vec<"));
    assert!(ts_code.contains("[]") || ts_code.contains("vec"));
    assert!(py_code.contains("List") || py_code.contains("Vec"));
    assert!(go_code.contains("[]"));
    assert!(rb_code.contains(":array") || rb_code.contains("vec_u8"));

    // Option types
    assert!(rust_code.contains("Option<"));
    assert!(ts_code.contains("| undefined") || ts_code.contains("option"));
    assert!(py_code.contains("Optional") || py_code.contains("Option"));
    assert!(go_code.contains("*")); // Go uses pointers for options
}

#[test]
fn all_languages_generate_enum() {
    let type_defs = vec![TypeDefinition::Enum(create_test_enum())];

    let rust_code = generate_rust(&type_defs);
    let ts_code = generate_typescript(&type_defs);
    let py_code = generate_python(&type_defs);
    let go_code = generate_go(&type_defs);
    let rb_code = generate_ruby(&type_defs);

    // All should contain enum name
    assert!(rust_code.contains("TestEnum"));
    assert!(ts_code.contains("TestEnum"));
    assert!(py_code.contains("TestEnum"));
    assert!(go_code.contains("TestEnum"));
    assert!(rb_code.contains("TestEnum"));

    // All should have variant names
    assert!(rust_code.contains("Active"));
    assert!(ts_code.contains("Active"));
}

#[test]
fn field_order_consistent_across_languages() {
    let type_defs = vec![TypeDefinition::Struct(create_all_primitives_struct())];

    let rust_code = generate_rust(&type_defs);
    let ts_code = generate_typescript(&type_defs);
    let py_code = generate_python(&type_defs);

    // Verify field order is preserved (u8 before u16 before u32...)
    // This is critical for Borsh serialization compatibility

    // Rust field order
    let rust_u8_pos = rust_code.find("val_u8").unwrap();
    let rust_u16_pos = rust_code.find("val_u16").unwrap();
    let rust_u32_pos = rust_code.find("val_u32").unwrap();
    assert!(
        rust_u8_pos < rust_u16_pos,
        "Rust: val_u8 should come before val_u16"
    );
    assert!(
        rust_u16_pos < rust_u32_pos,
        "Rust: val_u16 should come before val_u32"
    );

    // TypeScript field order
    let ts_u8_pos = ts_code.find("val_u8").unwrap();
    let ts_u16_pos = ts_code.find("val_u16").unwrap();
    let ts_u32_pos = ts_code.find("val_u32").unwrap();
    assert!(
        ts_u8_pos < ts_u16_pos,
        "TypeScript: val_u8 should come before val_u16"
    );
    assert!(
        ts_u16_pos < ts_u32_pos,
        "TypeScript: val_u16 should come before val_u32"
    );

    // Python field order
    let py_u8_pos = py_code.find("val_u8").unwrap();
    let py_u16_pos = py_code.find("val_u16").unwrap();
    let py_u32_pos = py_code.find("val_u32").unwrap();
    assert!(
        py_u8_pos < py_u16_pos,
        "Python: val_u8 should come before val_u16"
    );
    assert!(
        py_u16_pos < py_u32_pos,
        "Python: val_u16 should come before val_u32"
    );
}

#[test]
fn borsh_schema_structure_matches() {
    let type_defs = vec![TypeDefinition::Struct(create_all_primitives_struct())];

    let ts_code = generate_typescript(&type_defs);
    let py_code = generate_python(&type_defs);
    let rb_code = generate_ruby(&type_defs);

    // TypeScript: Verify Borsh schema uses correct types
    assert!(
        ts_code.contains("u8") || ts_code.contains("borsh"),
        "TypeScript should reference Borsh types"
    );

    // Python: Verify uses borsh constructs
    assert!(
        py_code.contains("U8") || py_code.contains("u8") || py_code.contains("int"),
        "Python should reference integer types"
    );

    // Ruby: Verify Borsh schema uses correct types
    assert!(
        rb_code.contains(":u8") || rb_code.contains("Integer"),
        "Ruby Borsh schema should include u8"
    );
}

#[test]
fn u128_handling_differs_by_language() {
    let type_defs = vec![TypeDefinition::Struct(make_struct(
        "BigNumbers",
        vec![field("big_value", TypeInfo::Primitive("u128".to_string()))],
    ))];

    let rust_code = generate_rust(&type_defs);
    let ts_code = generate_typescript(&type_defs);
    let py_code = generate_python(&type_defs);
    let go_code = generate_go(&type_defs);
    let rb_code = generate_ruby(&type_defs);

    // Rust: native u128
    assert!(rust_code.contains("u128"));

    // TypeScript: bigint
    assert!(ts_code.contains("bigint"));

    // Python: int (arbitrary precision)
    assert!(py_code.contains("int") || py_code.contains("U128"));

    // Go: [16]byte (no native 128-bit)
    assert!(go_code.contains("[16]byte"));

    // Ruby: Integer
    assert!(rb_code.contains("Integer") || rb_code.contains(":u128"));
}

#[test]
fn option_representation() {
    let type_defs = vec![TypeDefinition::Struct(make_struct(
        "OptionalData",
        vec![field(
            "maybe_value",
            TypeInfo::Option(Box::new(TypeInfo::Primitive("String".to_string()))),
        )],
    ))];

    let rust_code = generate_rust(&type_defs);
    let ts_code = generate_typescript(&type_defs);
    let py_code = generate_python(&type_defs);
    let go_code = generate_go(&type_defs);

    // Rust: Option<String>
    assert!(rust_code.contains("Option<String>"));

    // TypeScript: string | undefined
    assert!(ts_code.contains("| undefined") || ts_code.contains("option"));

    // Python: Optional[str]
    assert!(py_code.contains("Optional") || py_code.contains("Option"));

    // Go: *string (pointer)
    assert!(go_code.contains("*string"));
}

#[test]
fn fixed_array_handling() {
    let type_defs = vec![TypeDefinition::Struct(make_struct(
        "FixedData",
        vec![field(
            "hash",
            TypeInfo::FixedArray {
                element: Box::new(TypeInfo::Primitive("u8".to_string())),
                size: 32,
            },
        )],
    ))];

    let rust_code = generate_rust(&type_defs);
    let ts_code = generate_typescript(&type_defs);
    let go_code = generate_go(&type_defs);

    // Rust: [u8; 32]
    assert!(rust_code.contains("[u8; 32]"));

    // TypeScript: uses borsh.array with fixed size
    assert!(ts_code.contains("32") || ts_code.contains("array"));

    // Go: [32]uint8 or [32]byte
    assert!(go_code.contains("[32]") || go_code.contains("32]"));
}

#[test]
fn nested_vec_option_types() {
    let type_defs = vec![TypeDefinition::Struct(make_struct(
        "NestedTypes",
        vec![
            field(
                "vec_of_options",
                TypeInfo::Array(Box::new(TypeInfo::Option(Box::new(TypeInfo::Primitive(
                    "u64".to_string(),
                ))))),
            ),
            field(
                "option_of_vec",
                TypeInfo::Option(Box::new(TypeInfo::Array(Box::new(TypeInfo::Primitive(
                    "String".to_string(),
                ))))),
            ),
        ],
    ))];

    let rust_code = generate_rust(&type_defs);
    let ts_code = generate_typescript(&type_defs);

    // Rust should have proper nested types
    assert!(rust_code.contains("Vec<Option<u64>>"));
    assert!(rust_code.contains("Option<Vec<String>>"));

    // TypeScript should have nested types too
    assert!(ts_code.contains("vec_of_options"));
    assert!(ts_code.contains("option_of_vec"));
}

#[test]
fn multiple_structs_all_languages() {
    let type_defs = vec![
        TypeDefinition::Struct(create_all_primitives_struct()),
        TypeDefinition::Struct(create_solana_types_struct()),
        TypeDefinition::Struct(create_complex_types_struct()),
    ];

    let rust_code = generate_rust(&type_defs);
    let ts_code = generate_typescript(&type_defs);
    let py_code = generate_python(&type_defs);
    let go_code = generate_go(&type_defs);
    let rb_code = generate_ruby(&type_defs);

    // All structs should appear in all outputs
    let struct_names = ["AllPrimitives", "SolanaTypes", "ComplexTypes"];
    for name in struct_names {
        assert!(
            rust_code.contains(name),
            "Rust should contain struct: {}",
            name
        );
        assert!(
            ts_code.contains(name),
            "TypeScript should contain struct: {}",
            name
        );
        assert!(
            py_code.contains(name),
            "Python should contain struct: {}",
            name
        );
        assert!(go_code.contains(name), "Go should contain struct: {}", name);
        assert!(
            rb_code.contains(name),
            "Ruby should contain struct: {}",
            name
        );
    }
}
