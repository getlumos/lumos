// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Completion handler for LUMOS LSP

use tower_lsp::lsp_types::*;

/// Completion handler
#[derive(Debug)]
pub struct CompletionHandler;

impl CompletionHandler {
    /// Create new completion handler
    pub fn new() -> Self {
        Self
    }

    /// Get completion items at position
    pub fn get_completions(&self, _text: &str, _position: Position) -> Vec<CompletionItem> {
        // For now, return all available completions
        // TODO: Context-aware completions based on cursor position
        let mut items = Vec::new();

        // Solana types
        items.extend(Self::solana_type_completions());

        // Primitive types
        items.extend(Self::primitive_type_completions());

        // Attributes
        items.extend(Self::attribute_completions());

        // Keywords
        items.extend(Self::keyword_completions());

        items
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

    /// Attribute completions
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_includes_solana_types() {
        let handler = CompletionHandler::new();
        let items = handler.get_completions("", Position::new(0, 0));

        assert!(items.iter().any(|i| i.label == "PublicKey"));
        assert!(items.iter().any(|i| i.label == "Signature"));
        assert!(items.iter().any(|i| i.label == "Keypair"));
    }

    #[test]
    fn test_completion_includes_primitives() {
        let handler = CompletionHandler::new();
        let items = handler.get_completions("", Position::new(0, 0));

        assert!(items.iter().any(|i| i.label == "u64"));
        assert!(items.iter().any(|i| i.label == "String"));
        assert!(items.iter().any(|i| i.label == "bool"));
    }

    #[test]
    fn test_completion_includes_attributes() {
        let handler = CompletionHandler::new();
        let items = handler.get_completions("", Position::new(0, 0));

        assert!(items.iter().any(|i| i.label == "#[solana]"));
        assert!(items.iter().any(|i| i.label == "#[account]"));
        assert!(items.iter().any(|i| i.label == "#[key]"));
    }

    #[test]
    fn test_completion_includes_keywords() {
        let handler = CompletionHandler::new();
        let items = handler.get_completions("", Position::new(0, 0));

        assert!(items.iter().any(|i| i.label == "struct"));
        assert!(items.iter().any(|i| i.label == "enum"));
        assert!(items.iter().any(|i| i.label == "Vec"));
        assert!(items.iter().any(|i| i.label == "Option"));
    }
}
