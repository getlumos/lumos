// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Diagnostics handler for LUMOS LSP

use lumos_core::parser::parse_lumos_file;
use lumos_core::transform::transform_to_ir;
use tower_lsp::lsp_types::*;

/// Diagnostics handler
#[derive(Debug)]
pub struct DiagnosticsHandler;

impl DiagnosticsHandler {
    /// Create new diagnostics handler
    pub fn new() -> Self {
        Self
    }

    /// Analyze document and return diagnostics
    pub fn analyze(&self, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Step 1: Parse LUMOS file
        match parse_lumos_file(text) {
            Ok(ast) => {
                // Step 2: Transform to IR (validates types)
                if let Err(e) = transform_to_ir(ast) {
                    // Type validation errors
                    diagnostics.push(Self::error_to_diagnostic(&e));
                }
            }
            Err(e) => {
                // Parse errors
                diagnostics.push(Self::error_to_diagnostic(&e));
            }
        }

        diagnostics
    }

    /// Convert LUMOS error to LSP diagnostic
    fn error_to_diagnostic(error: &lumos_core::error::LumosError) -> Diagnostic {
        // Extract error message
        let message = error.to_string();

        // Try to extract line/column from error message
        let (line, column) = Self::extract_location_from_error(&message);

        // Create LSP range (0-indexed for LSP)
        let range = Range {
            start: Position {
                line: line.saturating_sub(1) as u32,
                character: column.saturating_sub(1) as u32,
            },
            end: Position {
                line: line.saturating_sub(1) as u32,
                character: column.saturating_sub(1) as u32 + 10, // Approximate end
            },
        };

        // Determine severity based on error type
        let severity = match error {
            lumos_core::error::LumosError::SchemaParse(_, _) => DiagnosticSeverity::ERROR,
            lumos_core::error::LumosError::TypeValidation(_, _) => DiagnosticSeverity::ERROR,
            lumos_core::error::LumosError::CodeGen(_) => DiagnosticSeverity::WARNING,
            _ => DiagnosticSeverity::ERROR,
        };

        Diagnostic {
            range,
            severity: Some(severity),
            source: Some("lumos".to_string()),
            message: Self::clean_error_message(&message),
            ..Default::default()
        }
    }

    /// Extract line and column from error message
    ///
    /// Error messages from LUMOS have format: "message at line N, column M"
    fn extract_location_from_error(message: &str) -> (usize, usize) {
        // Default to line 1, column 1 if parsing fails
        let mut line = 1;
        let mut column = 1;

        // Try to extract "at line N, column M" pattern
        if let Some(at_pos) = message.find(" at line ") {
            let rest = &message[at_pos + 9..]; // Skip " at line "

            // Extract line number
            if let Some(comma_pos) = rest.find(',') {
                if let Ok(l) = rest[..comma_pos].trim().parse::<usize>() {
                    line = l;
                }

                // Extract column number
                let rest = &rest[comma_pos + 1..];
                if let Some(col_start) = rest.find("column ") {
                    let col_str = &rest[col_start + 7..];
                    // Find first non-digit
                    let col_end = col_str
                        .find(|c: char| !c.is_numeric())
                        .unwrap_or(col_str.len());
                    if let Ok(c) = col_str[..col_end].trim().parse::<usize>() {
                        column = c;
                    }
                }
            }
        }

        (line, column)
    }

    /// Clean error message by removing location info
    fn clean_error_message(message: &str) -> String {
        // Remove " at line N, column M" suffix
        if let Some(at_pos) = message.find(" at line ") {
            message[..at_pos].to_string()
        } else if let Some(at_pos) = message.find(" (at ") {
            message[..at_pos].to_string()
        } else {
            message.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_on_valid_schema() {
        let handler = DiagnosticsHandler::new();
        let text = r#"
            #[solana]
            struct Account {
                owner: PublicKey,
                balance: u64,
            }
        "#;

        let diagnostics = handler.analyze(text);
        assert!(diagnostics.is_empty(), "Valid schema should have no diagnostics");
    }

    #[test]
    fn test_diagnostics_on_syntax_error() {
        let handler = DiagnosticsHandler::new();
        let text = r#"
            struct Account {
                invalid syntax here
            }
        "#;

        let diagnostics = handler.analyze(text);
        assert!(!diagnostics.is_empty(), "Syntax error should produce diagnostics");
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn test_diagnostics_on_undefined_type() {
        let handler = DiagnosticsHandler::new();
        let text = r#"
            #[solana]
            struct Account {
                owner: UndefinedType,
            }
        "#;

        let diagnostics = handler.analyze(text);
        assert!(!diagnostics.is_empty(), "Undefined type should produce diagnostics");
    }

    #[test]
    fn test_extract_location_from_error() {
        let message = "Failed to parse at line 5, column 12";
        let (line, column) = DiagnosticsHandler::extract_location_from_error(message);
        assert_eq!(line, 5);
        assert_eq!(column, 12);
    }

    #[test]
    fn test_clean_error_message() {
        let message = "Undefined type 'Foo' at line 5, column 12";
        let cleaned = DiagnosticsHandler::clean_error_message(message);
        assert_eq!(cleaned, "Undefined type 'Foo'");
    }
}
