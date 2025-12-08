// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Completion handler for LUMOS LSP

use tower_lsp::lsp_types::*;

/// Completion context determined by cursor position
#[derive(Debug, PartialEq, Eq)]
enum CompletionContext {
    /// Top-level (before any struct/enum)
    TopLevel,
    /// After #[ (attribute context)
    Attribute,
    /// After field name and : (type context)
    TypePosition,
    /// Inside enum body (variant context)
    EnumBody,
    /// Generic/unknown context
    Generic,
}

/// Completion handler
#[derive(Debug)]
pub struct CompletionHandler;

impl CompletionHandler {
    /// Create new completion handler
    pub fn new() -> Self {
        Self
    }

    /// Get completion items at position
    pub fn get_completions(&self, text: &str, position: Position) -> Vec<CompletionItem> {
        // Detect context based on cursor position
        let context = self.detect_context(text, position);

        // Return context-appropriate completions
        match context {
            CompletionContext::TopLevel => {
                let mut items = Vec::new();
                items.extend(Self::keyword_completions());
                items.extend(Self::attribute_completions());
                items
            }
            CompletionContext::Attribute => Self::attribute_names_only(),
            CompletionContext::TypePosition => {
                let mut items = Vec::new();
                items.extend(Self::solana_type_completions());
                items.extend(Self::primitive_type_completions());
                items.extend(Self::container_type_completions());
                items
            }
            CompletionContext::EnumBody => {
                // For enum context, show variant patterns
                vec![]
            }
            CompletionContext::Generic => {
                // Fallback: return all completions
                let mut items = Vec::new();
                items.extend(Self::solana_type_completions());
                items.extend(Self::primitive_type_completions());
                items.extend(Self::attribute_completions());
                items.extend(Self::keyword_completions());
                items
            }
        }
    }

    /// Detect completion context based on cursor position
    fn detect_context(&self, text: &str, position: Position) -> CompletionContext {
        // Handle empty text special case
        if text.is_empty() {
            return CompletionContext::TopLevel;
        }

        // Get the line at cursor position
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return CompletionContext::Generic;
        }

        let current_line = lines[position.line as usize];
        let cursor_col = position.character as usize;

        // Get text before cursor on current line
        let before_cursor = if cursor_col <= current_line.len() {
            &current_line[..cursor_col]
        } else {
            current_line
        };

        // Check for attribute context: after #[
        if before_cursor.trim_end().ends_with("#[") {
            return CompletionContext::Attribute;
        }

        // Check for type position: after field_name:
        if before_cursor.contains(':') && !before_cursor.contains('{') {
            let after_colon = before_cursor.split(':').next_back().unwrap_or("").trim();
            // If we're right after colon or in middle of type name
            if after_colon.is_empty() || !after_colon.contains(' ') {
                return CompletionContext::TypePosition;
            }
        }

        // Check if we're inside enum by looking at previous lines
        let mut in_enum = false;
        for line in &lines[..=position.line as usize] {
            if line.trim().starts_with("enum ") {
                in_enum = true;
            }
            if line.trim() == "}" {
                in_enum = false;
            }
        }
        if in_enum {
            return CompletionContext::EnumBody;
        }

        // Check if we're at top level (no struct/enum before us)
        let mut found_declaration = false;
        for line in &lines[..position.line as usize] {
            let trimmed = line.trim();
            if trimmed.starts_with("struct ") || trimmed.starts_with("enum ") {
                found_declaration = true;
                break;
            }
        }

        if !found_declaration && before_cursor.trim().is_empty() {
            return CompletionContext::TopLevel;
        }

        CompletionContext::Generic
    }

    /// Solana type completions
    fn solana_type_completions() -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "PublicKey".to_string(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some("Solana public key (32 bytes)".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Solana public key type.\n\n**Size**: 32 bytes\n**Use for**: Account addresses, signers".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "Signature".to_string(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some("Solana signature (64 bytes)".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Solana signature type.\n\n**Size**: 64 bytes\n**Use for**: Transaction signatures".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "Keypair".to_string(),
                kind: Some(CompletionItemKind::STRUCT),
                detail: Some("Solana keypair".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Solana keypair type.\n\n**Use for**: Authority, signing".to_string(),
                })),
                ..Default::default()
            },
        ]
    }

    /// Primitive type completions
    fn primitive_type_completions() -> Vec<CompletionItem> {
        vec![
            // Unsigned integers
            Self::create_type_item("u8", "8-bit unsigned integer", "Range: 0 to 255"),
            Self::create_type_item("u16", "16-bit unsigned integer", "Range: 0 to 65,535"),
            Self::create_type_item("u32", "32-bit unsigned integer", "Range: 0 to 4,294,967,295"),
            Self::create_type_item("u64", "64-bit unsigned integer", "Range: 0 to 18,446,744,073,709,551,615\n\n**Warning**: JavaScript numbers are safe up to 2^53-1"),
            Self::create_type_item("u128", "128-bit unsigned integer", "Range: 0 to 340,282,366,920,938,463,463,374,607,431,768,211,455"),
            // Signed integers
            Self::create_type_item("i8", "8-bit signed integer", "Range: -128 to 127"),
            Self::create_type_item("i16", "16-bit signed integer", "Range: -32,768 to 32,767"),
            Self::create_type_item("i32", "32-bit signed integer", "Range: -2,147,483,648 to 2,147,483,647"),
            Self::create_type_item("i64", "64-bit signed integer", "Range: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807"),
            Self::create_type_item("i128", "128-bit signed integer", "Very large signed integer"),
            // Floats
            Self::create_type_item("f32", "32-bit floating point", "Single precision float"),
            Self::create_type_item("f64", "64-bit floating point", "Double precision float"),
            // Boolean
            Self::create_type_item("bool", "Boolean type", "Values: true or false"),
            // String
            Self::create_type_item("String", "UTF-8 string", "Variable-length text"),
        ]
    }

    /// Attribute completions (full with #[...])
    fn attribute_completions() -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "#[solana]".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Mark as Solana-compatible type".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Marks a struct or enum as Solana-compatible.\n\n**Usage**: `#[solana]`\n**Applies to**: structs, enums".to_string(),
                })),
                insert_text: Some("#[solana]".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "#[account]".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Mark as Anchor account".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Marks a struct as an Anchor account.\n\n**Usage**: `#[account]`\n**Applies to**: structs\n**Requires**: `#[solana]` attribute".to_string(),
                })),
                insert_text: Some("#[account]".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "#[key]".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Mark field as unique key".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Marks a field as a unique key.\n\n**Usage**: `#[key]`\n**Applies to**: struct fields".to_string(),
                })),
                insert_text: Some("#[key]".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "#[max(n)]".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Set maximum array/string length".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Sets maximum length for arrays or strings.\n\n**Usage**: `#[max(100)]`\n**Applies to**: array and String fields".to_string(),
                })),
                insert_text: Some("#[max($1)]".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "#[deprecated]".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Mark field as deprecated".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Marks a field as deprecated.\n\n**Usage**: `#[deprecated]` or `#[deprecated(\"message\")]`\n**Applies to**: struct fields, enum variant fields".to_string(),
                })),
                insert_text: Some("#[deprecated]".to_string()),
                ..Default::default()
            },
        ]
    }

    /// Attribute name completions (without #[, for after #[ context)
    fn attribute_names_only() -> Vec<CompletionItem> {
        vec![
            Self::create_attribute_name_item("solana", "Mark as Solana-compatible type", "solana]"),
            Self::create_attribute_name_item("account", "Mark as Anchor account", "account]"),
            Self::create_attribute_name_item("key", "Mark field as unique key", "key]"),
            Self::create_attribute_name_item("max", "Set maximum array/string length", "max($1)]"),
            Self::create_attribute_name_item(
                "deprecated",
                "Mark field as deprecated",
                "deprecated]",
            ),
        ]
    }

    /// Container type completions (Vec, Option)
    fn container_type_completions() -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "Vec".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Dynamic array type".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Dynamic array type.\n\n**Usage**: `Vec<Type>`\n**Example**: `items: Vec<PublicKey>`".to_string(),
                })),
                insert_text: Some("Vec<$1>".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "Option".to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Optional type".to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "Optional type (can be None).\n\n**Usage**: `Option<Type>`\n**Example**: `email: Option<String>`".to_string(),
                })),
                insert_text: Some("Option<$1>".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    /// Keyword completions
    fn keyword_completions() -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "struct".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Define a struct".to_string()),
                documentation: Some(Documentation::String("Define a struct type".to_string())),
                insert_text: Some("struct $1 {\n    $2\n}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "enum".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Define an enum".to_string()),
                documentation: Some(Documentation::String("Define an enum type".to_string())),
                insert_text: Some("enum $1 {\n    $2\n}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    /// Helper to create type completion item
    fn create_type_item(label: &str, detail: &str, docs: &str) -> CompletionItem {
        CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some(detail.to_string()),
            documentation: Some(Documentation::String(docs.to_string())),
            ..Default::default()
        }
    }

    /// Helper to create attribute name completion item (without #[)
    fn create_attribute_name_item(label: &str, detail: &str, insert_text: &str) -> CompletionItem {
        CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            insert_text: Some(insert_text.to_string()),
            insert_text_format: if insert_text.contains('$') {
                Some(InsertTextFormat::SNIPPET)
            } else {
                None
            },
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_includes_solana_types() {
        let handler = CompletionHandler::new();
        // Generic context should include all types
        let items = handler.get_completions("struct Foo { x: ", Position::new(0, 16));

        assert!(items.iter().any(|i| i.label == "PublicKey"));
        assert!(items.iter().any(|i| i.label == "Signature"));
        assert!(items.iter().any(|i| i.label == "Keypair"));
    }

    #[test]
    fn test_completion_includes_primitives() {
        let handler = CompletionHandler::new();
        // Generic context should include all types
        let items = handler.get_completions("struct Foo { x: ", Position::new(0, 16));

        assert!(items.iter().any(|i| i.label == "u64"));
        assert!(items.iter().any(|i| i.label == "String"));
        assert!(items.iter().any(|i| i.label == "bool"));
    }

    #[test]
    fn test_completion_includes_attributes() {
        let handler = CompletionHandler::new();
        // Top-level context should include attributes
        let items = handler.get_completions("", Position::new(0, 0));

        assert!(items.iter().any(|i| i.label == "#[solana]"));
        assert!(items.iter().any(|i| i.label == "#[account]"));
        assert!(items.iter().any(|i| i.label == "#[key]"));
    }

    #[test]
    fn test_completion_includes_keywords() {
        let handler = CompletionHandler::new();
        // Top-level context should include keywords
        let items = handler.get_completions("", Position::new(0, 0));

        assert!(items.iter().any(|i| i.label == "struct"));
        assert!(items.iter().any(|i| i.label == "enum"));
    }

    #[test]
    fn test_context_top_level() {
        let handler = CompletionHandler::new();
        let items = handler.get_completions("", Position::new(0, 0));

        // Should include keywords and attributes, but not types
        assert!(items.iter().any(|i| i.label == "struct"));
        assert!(items.iter().any(|i| i.label == "enum"));
        assert!(items.iter().any(|i| i.label == "#[solana]"));
        assert!(!items.iter().any(|i| i.label == "u64"));
        assert!(!items.iter().any(|i| i.label == "PublicKey"));
    }

    #[test]
    fn test_context_after_attribute_bracket() {
        let handler = CompletionHandler::new();
        let text = "#[";
        let items = handler.get_completions(text, Position::new(0, 2));

        // Should only include attribute names (without #[)
        assert!(items.iter().any(|i| i.label == "solana"));
        assert!(items.iter().any(|i| i.label == "account"));
        assert!(items.iter().any(|i| i.label == "key"));
        assert!(!items.iter().any(|i| i.label == "struct"));
        assert!(!items.iter().any(|i| i.label == "u64"));
    }

    #[test]
    fn test_context_type_position() {
        let handler = CompletionHandler::new();
        let text = "struct User {\n    id: ";
        let items = handler.get_completions(text, Position::new(1, 8));

        // Should only include types, not keywords or attributes
        assert!(items.iter().any(|i| i.label == "u64"));
        assert!(items.iter().any(|i| i.label == "PublicKey"));
        assert!(items.iter().any(|i| i.label == "Vec"));
        assert!(items.iter().any(|i| i.label == "Option"));
        assert!(!items.iter().any(|i| i.label == "struct"));
        assert!(!items.iter().any(|i| i.label == "#[solana]"));
    }

    #[test]
    fn test_context_enum_body() {
        let handler = CompletionHandler::new();
        let text = "enum State {\n    ";
        let items = handler.get_completions(text, Position::new(1, 4));

        // Currently returns empty for enum body
        // In future, could suggest variant patterns
        assert!(items.is_empty());
    }
}
