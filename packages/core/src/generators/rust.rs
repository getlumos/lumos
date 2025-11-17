// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Rust code generator

use crate::ir::TypeDefinition;

/// Generate Rust code from a type definition
pub fn generate(_type_def: &TypeDefinition) -> String {
    // TODO: Implement Rust code generation
    "// Generated Rust code will appear here\n".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{TypeDefinition, Metadata};

    #[test]
    fn generates_rust_code() {
        let type_def = TypeDefinition {
            name: "User".to_string(),
            fields: vec![],
            metadata: Metadata::default(),
        };

        let code = generate(&type_def);
        assert!(!code.is_empty());
    }
}
