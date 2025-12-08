// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Formatting handler for LUMOS LSP

use lumos_core::ast::*;
use lumos_core::parser::parse_lumos_file;
use tower_lsp::lsp_types::*;

/// Formatting handler
#[derive(Debug)]
pub struct FormattingHandler {
    /// Indentation configuration (number of spaces)
    indent_size: usize,
}

impl FormattingHandler {
    /// Create new formatting handler with default settings
    pub fn new() -> Self {
        Self { indent_size: 4 }
    }

    /// Format LUMOS document and generate TextEdits
    pub fn format(&self, text: &str) -> Result<Vec<TextEdit>, String> {
        // Parse the document
        let ast = parse_lumos_file(text).map_err(|e| e.to_string())?;

        // Format the AST
        let formatted = self.format_file(&ast);

        // If text is already formatted, return empty edits
        if text == formatted {
            return Ok(vec![]);
        }

        // Generate a single TextEdit that replaces entire document
        // (More efficient than line-by-line diffs for formatting)
        let lines = text.lines().count() as u32;
        let last_line_len = text.lines().last().map(|l| l.len()).unwrap_or(0) as u32;

        Ok(vec![TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: lines.saturating_sub(1),
                    character: last_line_len,
                },
            },
            new_text: formatted,
        }])
    }

    /// Format a complete LUMOS file
    fn format_file(&self, file: &LumosFile) -> String {
        let mut output = String::new();

        // Format use statements
        for use_stmt in &file.imports {
            output.push_str(&self.format_import(use_stmt));
            output.push('\n');
        }

        // Add blank line after imports if any exist
        if !file.imports.is_empty() && !file.items.is_empty() {
            output.push('\n');
        }

        // Format items (structs, enums, modules, etc.)
        for (i, item) in file.items.iter().enumerate() {
            if i > 0 {
                output.push('\n'); // Blank line between items
            }
            output.push_str(&self.format_item(item, 0));

            // Add newline after each item except the last one
            if i < file.items.len() - 1 {
                output.push('\n');
            }
        }

        output
    }

    /// Format an import statement
    fn format_import(&self, import: &Import) -> String {
        if import.items.len() == 1 {
            format!("use {} from \"{}\";", import.items[0], import.path)
        } else {
            format!(
                "use {{ {} }} from \"{}\";",
                import.items.join(", "),
                import.path
            )
        }
    }

    /// Format a top-level item
    fn format_item(&self, item: &Item, indent_level: usize) -> String {
        match item {
            Item::Struct(s) => self.format_struct_def(s, indent_level),
            Item::Enum(e) => self.format_enum_def(e, indent_level),
            Item::TypeAlias(t) => self.format_type_alias(t, indent_level),
            Item::Module(m) => self.format_module(m, indent_level),
            Item::Use(u) => self.format_use(u, indent_level),
        }
    }

    /// Format a struct definition
    fn format_struct_def(&self, struct_def: &StructDef, indent_level: usize) -> String {
        let indent = self.indent(indent_level);
        let field_indent = self.indent(indent_level + 1);
        let mut output = String::new();

        // Format attributes
        for attr in &struct_def.attributes {
            output.push_str(&format!("{}#[{}]\n", indent, self.format_attribute(attr)));
        }

        // Format visibility
        let visibility = match struct_def.visibility {
            Visibility::Public => "pub ",
            Visibility::Private => "",
        };

        // Struct header
        output.push_str(&format!(
            "{}{}struct {} {{\n",
            indent, visibility, struct_def.name
        ));

        // Format fields
        for field in &struct_def.fields {
            // Field attributes
            for attr in &field.attributes {
                output.push_str(&format!(
                    "{}#[{}]\n",
                    field_indent,
                    self.format_attribute(attr)
                ));
            }

            // Field definition
            let field_type = self.format_type_spec(&field.type_spec);
            output.push_str(&format!(
                "{}{}: {},\n",
                field_indent, field.name, field_type
            ));
        }

        // Closing brace
        output.push_str(&format!("{}}}", indent));

        output
    }

    /// Format an enum definition
    fn format_enum_def(&self, enum_def: &EnumDef, indent_level: usize) -> String {
        let indent = self.indent(indent_level);
        let variant_indent = self.indent(indent_level + 1);
        let mut output = String::new();

        // Format attributes
        for attr in &enum_def.attributes {
            output.push_str(&format!("{}#[{}]\n", indent, self.format_attribute(attr)));
        }

        // Format visibility
        let visibility = match enum_def.visibility {
            Visibility::Public => "pub ",
            Visibility::Private => "",
        };

        // Enum header
        output.push_str(&format!(
            "{}{}enum {} {{\n",
            indent, visibility, enum_def.name
        ));

        // Format variants
        for variant in &enum_def.variants {
            match variant {
                EnumVariant::Unit { name, .. } => {
                    output.push_str(&format!("{}{},\n", variant_indent, name));
                }
                EnumVariant::Tuple { name, types, .. } => {
                    let formatted_types: Vec<String> =
                        types.iter().map(|t| self.format_type_spec(t)).collect();
                    output.push_str(&format!(
                        "{}{}({}),\n",
                        variant_indent,
                        name,
                        formatted_types.join(", ")
                    ));
                }
                EnumVariant::Struct { name, fields, .. } => {
                    output.push_str(&format!("{}{} {{\n", variant_indent, name));

                    let field_indent = self.indent(indent_level + 2);
                    for field in fields {
                        let field_type = self.format_type_spec(&field.type_spec);
                        output.push_str(&format!(
                            "{}{}: {},\n",
                            field_indent, field.name, field_type
                        ));
                    }

                    output.push_str(&format!("{}}},\n", variant_indent));
                }
            }
        }

        // Closing brace
        output.push_str(&format!("{}}}", indent));

        output
    }

    /// Format a type alias
    fn format_type_alias(&self, type_alias: &TypeAlias, indent_level: usize) -> String {
        let indent = self.indent(indent_level);

        // Format visibility
        let visibility = match type_alias.visibility {
            Visibility::Public => "pub ",
            Visibility::Private => "",
        };

        format!(
            "{}{}type {} = {};",
            indent,
            visibility,
            type_alias.name,
            self.format_type_spec(&type_alias.target)
        )
    }

    /// Format a module declaration
    fn format_module(&self, module: &Module, indent_level: usize) -> String {
        let indent = self.indent(indent_level);

        // Format visibility
        let visibility = match module.visibility {
            Visibility::Public => "pub ",
            Visibility::Private => "",
        };

        format!("{}{}mod {};", indent, visibility, module.name)
    }

    /// Format a use statement
    fn format_use(&self, use_stmt: &UseStatement, indent_level: usize) -> String {
        let indent = self.indent(indent_level);
        let path = self.format_module_path(&use_stmt.path);

        if let Some(alias) = &use_stmt.alias {
            format!("{}use {} as {};", indent, path, alias)
        } else {
            format!("{}use {};", indent, path)
        }
    }

    /// Format a module path
    fn format_module_path(&self, path: &ModulePath) -> String {
        path.segments
            .iter()
            .map(|seg| match seg {
                PathSegment::Crate => "crate".to_string(),
                PathSegment::Super => "super".to_string(),
                PathSegment::SelfPath => "self".to_string(),
                PathSegment::Ident(name) => name.clone(),
            })
            .collect::<Vec<_>>()
            .join("::")
    }

    /// Format a type specification
    #[allow(clippy::only_used_in_recursion)]
    fn format_type_spec(&self, type_spec: &TypeSpec) -> String {
        match type_spec {
            TypeSpec::Primitive(name) => name.clone(),
            TypeSpec::Generic(name) => name.clone(),
            TypeSpec::UserDefined(name) => name.clone(),
            TypeSpec::Array(inner) => format!("[{}]", self.format_type_spec(inner)),
            TypeSpec::FixedArray { element, size } => {
                format!("[{}; {}]", self.format_type_spec(element), size)
            }
        }
    }

    /// Format an attribute
    fn format_attribute(&self, attr: &Attribute) -> String {
        match &attr.value {
            Some(AttributeValue::String(s)) => format!("{} = \"{}\"", attr.name, s),
            Some(AttributeValue::Integer(n)) => format!("{} = {}", attr.name, n),
            Some(AttributeValue::Bool(b)) => format!("{} = {}", attr.name, b),
            Some(AttributeValue::List(items)) => format!("{}({})", attr.name, items.join(", ")),
            None => attr.name.clone(),
        }
    }

    /// Generate indentation string
    fn indent(&self, level: usize) -> String {
        " ".repeat(self.indent_size * level)
    }
}

impl Default for FormattingHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_struct() {
        let input = r#"
#[solana]
struct PlayerAccount{
wallet:PublicKey,
    level: u16,
experience:u64,
}
"#;

        let formatter = FormattingHandler::new();
        let result = formatter.format(input.trim());

        assert!(result.is_ok());
        let edits = result.unwrap();
        assert_eq!(edits.len(), 1);

        let formatted = &edits[0].new_text;
        assert!(formatted.contains("struct PlayerAccount {"));
        assert!(formatted.contains("    wallet: PublicKey,"));
        assert!(formatted.contains("    level: u16,"));
        assert!(formatted.contains("    experience: u64,"));
    }

    #[test]
    fn test_format_enum() {
        let input = r#"enum GameState{Active,Paused,Finished}"#;

        let formatter = FormattingHandler::new();
        let result = formatter.format(input);

        assert!(result.is_ok());
        let edits = result.unwrap();
        assert_eq!(edits.len(), 1);

        let formatted = &edits[0].new_text;
        assert!(formatted.contains("enum GameState {"));
        assert!(formatted.contains("    Active,"));
        assert!(formatted.contains("    Paused,"));
        assert!(formatted.contains("    Finished,"));
    }

    #[test]
    fn test_already_formatted() {
        // Canonical format: pub visibility (LUMOS default) and no trailing newline
        let input = r#"pub struct User {
    id: u64,
    name: String,
}"#;

        let formatter = FormattingHandler::new();
        let result = formatter.format(input);

        assert!(result.is_ok());
        let edits = result.unwrap();
        // Should return empty edits if already formatted
        assert_eq!(edits.len(), 0);
    }
}
