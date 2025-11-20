// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! WASM bindings for the LUMOS playground
//!
//! This module provides WebAssembly bindings that allow the LUMOS code generator
//! to run in the browser for the interactive playground.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{parser, transform, generators};

/// Result of code generation containing both Rust and TypeScript outputs
#[derive(Serialize, Deserialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct GeneratedCode {
    /// Generated Rust code
    pub rust: String,
    /// Generated TypeScript code
    pub typescript: String,
}

/// Generate Rust and TypeScript code from a LUMOS schema
///
/// # Arguments
///
/// * `source` - The .lumos schema source code
///
/// # Returns
///
/// A `GeneratedCode` struct containing both Rust and TypeScript outputs,
/// or a JavaScript Error if parsing/generation fails
///
/// # Example (JavaScript)
///
/// ```js
/// import { generateCode } from 'lumos-wasm';
///
/// const schema = `
/// #[solana]
/// #[account]
/// struct PlayerAccount {
///     wallet: PublicKey,
///     level: u16,
/// }
/// `;
///
/// try {
///     const result = generateCode(schema);
///     console.log('Rust:', result.rust);
///     console.log('TypeScript:', result.typescript);
/// } catch (error) {
///     console.error('Generation failed:', error.message);
/// }
/// ```
#[wasm_bindgen(js_name = generateCode)]
pub fn generate_code(source: &str) -> Result<GeneratedCode, JsValue> {
    // Parse the .lumos file into AST
    let ast = parser::parse_lumos_file(source)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    // Transform AST into IR
    let ir = transform::transform_to_ir(ast)
        .map_err(|e| JsValue::from_str(&format!("Transform error: {}", e)))?;

    // Generate Rust code
    let rust_code = generators::rust::generate_module(&ir);

    // Generate TypeScript code
    let typescript_code = generators::typescript::generate_module(&ir);

    Ok(GeneratedCode {
        rust: rust_code,
        typescript: typescript_code,
    })
}

/// Validate a LUMOS schema without generating code
///
/// Useful for providing real-time feedback in the editor without
/// the overhead of full code generation.
///
/// # Arguments
///
/// * `source` - The .lumos schema source code
///
/// # Returns
///
/// `Ok(())` if the schema is valid, or a JavaScript Error with the validation message
#[wasm_bindgen(js_name = validateSchema)]
pub fn validate_schema(source: &str) -> Result<(), JsValue> {
    // Parse the .lumos file
    let ast = parser::parse_lumos_file(source)
        .map_err(|e| JsValue::from_str(&format!("Validation error: {}", e)))?;

    // Transform to IR to catch semantic errors
    let _ = transform::transform_to_ir(ast)
        .map_err(|e| JsValue::from_str(&format!("Validation error: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code_simple_struct() {
        let source = r#"
            #[solana]
            #[account]
            struct PlayerAccount {
                wallet: PublicKey,
                level: u16,
            }
        "#;

        let result = generate_code(source);
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.rust.contains("pub struct PlayerAccount"));
        assert!(code.rust.contains("#[account]"));
        assert!(code.typescript.contains("interface PlayerAccount"));
        assert!(code.typescript.contains("borsh.publicKey"));
    }

    #[test]
    fn test_generate_code_enum() {
        let source = r#"
            #[solana]
            enum GameState {
                Active,
                Paused,
                Finished,
            }
        "#;

        let result = generate_code(source);
        assert!(result.is_ok());

        let code = result.unwrap();
        assert!(code.rust.contains("pub enum GameState"));
        assert!(code.typescript.contains("type GameState"));
        assert!(code.typescript.contains("kind:"));
    }

    #[test]
    fn test_validate_schema_valid() {
        let source = r#"
            #[solana]
            struct Account {
                owner: PublicKey,
            }
        "#;

        let result = validate_schema(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_schema_invalid() {
        let source = r#"
            #[solana]
            struct Account {
                owner: InvalidType,
            }
        "#;

        let result = validate_schema(source);
        assert!(result.is_err());
    }
}
