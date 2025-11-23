// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Hover handler for LUMOS LSP

use tower_lsp::lsp_types::*;

/// Hover handler
#[derive(Debug)]
pub struct HoverHandler;

impl HoverHandler {
    /// Create new hover handler
    pub fn new() -> Self {
        Self
    }

    /// Get hover information at position
    pub fn get_hover(&self, text: &str, position: Position) -> Option<Hover> {
        // Get word at cursor position
        let word = Self::get_word_at_position(text, position)?;

        // Get hover content for word
        let content = Self::get_hover_content(&word)?;

        Some(Hover {
            contents: HoverContents::Markup(content),
            range: None,
        })
    }

    /// Extract word at cursor position
    fn get_word_at_position(text: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        let line = lines.get(position.line as usize)?;

        let char_pos = position.character as usize;
        if char_pos > line.len() {
            return None;
        }

        // Find word boundaries
        let start = line[..char_pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        let end = line[char_pos..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + char_pos)
            .unwrap_or(line.len());

        if start >= end {
            return None;
        }

        Some(line[start..end].to_string())
    }

    /// Get hover content for a word
    fn get_hover_content(word: &str) -> Option<MarkupContent> {
        let content = match word {
            // Solana types
            "PublicKey" => {
                "**PublicKey** - Solana public key\n\n\
                **Size**: 32 bytes  \n\
                **Use for**: Account addresses, program IDs, signers  \n\
                **Rust**: `anchor_lang::prelude::Pubkey`  \n\
                **TypeScript**: `PublicKey` from `@solana/web3.js`"
            }
            "Signature" => {
                "**Signature** - Solana transaction signature\n\n\
                **Size**: 64 bytes  \n\
                **Use for**: Transaction verification  \n\
                **Rust**: `solana_program::signature::Signature`  \n\
                **TypeScript**: `string` (base58-encoded)"
            }
            "Keypair" => {
                "**Keypair** - Solana keypair (public + private key)\n\n\
                **Use for**: Authority, signing operations  \n\
                **Rust**: `solana_program::signature::Keypair`  \n\
                **TypeScript**: `Keypair` from `@solana/web3.js`"
            }

            // Unsigned integers
            "u8" => "**u8** - 8-bit unsigned integer\n\n**Range**: 0 to 255",
            "u16" => "**u16** - 16-bit unsigned integer\n\n**Range**: 0 to 65,535",
            "u32" => "**u32** - 32-bit unsigned integer\n\n**Range**: 0 to 4,294,967,295",
            "u64" => {
                "**u64** - 64-bit unsigned integer\n\n\
                **Range**: 0 to 18,446,744,073,709,551,615  \n\
                **Common for**: Lamports, timestamps, counts  \n\
                ⚠️ **JavaScript precision limit**: 2^53-1 (9,007,199,254,740,991)"
            }
            "u128" => {
                "**u128** - 128-bit unsigned integer\n\n\
                **Range**: 0 to 340,282,366,920,938,463,463,374,607,431,768,211,455  \n\
                **TypeScript**: Maps to `bigint`"
            }

            // Signed integers
            "i8" => "**i8** - 8-bit signed integer\n\n**Range**: -128 to 127",
            "i16" => "**i16** - 16-bit signed integer\n\n**Range**: -32,768 to 32,767",
            "i32" => {
                "**i32** - 32-bit signed integer\n\n**Range**: -2,147,483,648 to 2,147,483,647"
            }
            "i64" => {
                "**i64** - 64-bit signed integer\n\n\
                **Range**: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807  \n\
                ⚠️ **JavaScript precision limit**: ±2^53-1"
            }
            "i128" => "**i128** - 128-bit signed integer\n\n**TypeScript**: Maps to `bigint`",

            // Floats
            "f32" => "**f32** - 32-bit floating point\n\n**Precision**: ~7 decimal digits",
            "f64" => "**f64** - 64-bit floating point\n\n**Precision**: ~15 decimal digits",

            // Boolean
            "bool" => "**bool** - Boolean type\n\n**Values**: `true` or `false`",

            // String
            "String" => {
                "**String** - UTF-8 string\n\n\
                **Variable length**: Stored with length prefix  \n\
                **Rust**: `String`  \n\
                **TypeScript**: `string`"
            }

            // Complex types
            "Vec" => {
                "**Vec\\<T>** - Dynamic array\n\n\
                **Example**: `items: Vec<PublicKey>`  \n\
                **Rust**: `Vec<T>`  \n\
                **TypeScript**: `T[]`  \n\
                **Note**: Use `#[max(n)]` to limit length"
            }
            "Option" => {
                "**Option\\<T>** - Optional value\n\n\
                **Example**: `email: Option<String>`  \n\
                **Rust**: `Option<T>` (Some/None)  \n\
                **TypeScript**: `T | undefined`"
            }

            // Keywords
            "struct" => {
                "**struct** - Define a structured data type\n\n\
                **Syntax**:\n```rust\n\
                #[solana]\n\
                struct Name {\n    \
                    field: Type,\n\
                }\n\
                ```"
            }
            "enum" => {
                "**enum** - Define an enumeration type\n\n\
                **Syntax**:\n```rust\n\
                #[solana]\n\
                enum State {\n    \
                    Variant1,\n    \
                    Variant2(Type),\n    \
                    Variant3 { field: Type },\n\
                }\n\
                ```"
            }

            // Attributes
            "solana" => {
                "**#[solana]** - Mark as Solana-compatible\n\n\
                Indicates this type is designed for Solana blockchain.  \n\
                **Required for**: Code generation  \n\
                **Applies to**: structs, enums"
            }
            "account" => {
                "**#[account]** - Mark as Anchor account\n\n\
                Indicates this struct is an Anchor account.  \n\
                **Requires**: `#[solana]` attribute  \n\
                **Applies to**: structs only  \n\
                **Generates**: Anchor Account trait implementation"
            }
            "key" => {
                "**#[key]** - Mark field as unique key\n\n\
                Indicates this field is a unique identifier.  \n\
                **Applies to**: struct fields"
            }
            "max" => {
                "**#[max(n)]** - Set maximum length\n\n\
                Sets the maximum length for arrays or strings.  \n\
                **Example**: `#[max(100)]`  \n\
                **Applies to**: `Vec<T>` and `String` fields"
            }
            "deprecated" => {
                "**#[deprecated]** - Mark field as deprecated\n\n\
                Marks a field as deprecated with optional migration message.  \n\
                **Example**: `#[deprecated(\"Use new_field instead\")]`  \n\
                **Applies to**: struct fields, enum variant fields"
            }

            _ => return None,
        };

        Some(MarkupContent {
            kind: MarkupKind::Markdown,
            value: content.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hover_on_publickey() {
        let handler = HoverHandler::new();
        let text = "owner: PublicKey";
        let hover = handler.get_hover(text, Position::new(0, 10));

        assert!(hover.is_some());
        if let Some(h) = hover {
            if let HoverContents::Markup(content) = h.contents {
                assert!(content.value.contains("PublicKey"));
                assert!(content.value.contains("32 bytes"));
            }
        }
    }

    #[test]
    fn test_hover_on_u64() {
        let handler = HoverHandler::new();
        let text = "balance: u64";
        let hover = handler.get_hover(text, Position::new(0, 10));

        assert!(hover.is_some());
        if let Some(h) = hover {
            if let HoverContents::Markup(content) = h.contents {
                assert!(content.value.contains("u64"));
                assert!(content.value.contains("64-bit"));
            }
        }
    }

    #[test]
    fn test_get_word_at_position() {
        let text = "owner: PublicKey";
        let word = HoverHandler::get_word_at_position(text, Position::new(0, 10));
        assert_eq!(word, Some("PublicKey".to_string()));

        let word = HoverHandler::get_word_at_position(text, Position::new(0, 2));
        assert_eq!(word, Some("owner".to_string()));
    }

    #[test]
    fn test_hover_on_unknown_word() {
        let handler = HoverHandler::new();
        let text = "foo: UnknownType";
        let hover = handler.get_hover(text, Position::new(0, 10));

        assert!(hover.is_none());
    }
}
