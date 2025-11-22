# LUMOS Extractor - Implementation Logic

**Purpose:** Concrete implementation details for the extraction tool
**Status:** Implementation guide with working code examples

---

## Overview: How Extraction Works

```
Input: Rust source code (String)
   ↓
1. Parse with syn → Rust AST
   ↓
2. Walk AST, find extractable items
   ↓
3. Analyze each item (check derives, attributes)
   ↓
4. Convert Rust types → LUMOS types
   ↓
5. Build LUMOS AST
   ↓
6. Format LUMOS AST → String
   ↓
Output: LUMOS schema (String)
```

---

## Core Module Structure

```rust
// packages/core/src/extract/mod.rs

pub mod analyzer;    // Step 2: Find extractable types
pub mod converter;   // Step 4: Rust → LUMOS conversion
pub mod formatter;   // Step 6: LUMOS AST → String
pub mod types;       // Data structures for extraction

use syn::File as RustFile;
use crate::ast as lumos_ast;

pub struct Extractor {
    config: ExtractConfig,
}

pub struct ExtractConfig {
    pub filter: FilterType,
    pub preserve_comments: bool,
}

pub enum FilterType {
    Account,    // Only #[account]
    Borsh,      // Only Borsh derives
    All,        // All serializable
}

impl Extractor {
    pub fn extract(&self, rust_source: &str) -> Result<String> {
        // Step 1: Parse Rust
        let rust_ast = syn::parse_file(rust_source)?;

        // Step 2: Find extractable types
        let extractable = analyzer::analyze(&rust_ast, &self.config)?;

        // Step 3-5: Convert to LUMOS AST
        let lumos_items = converter::convert_all(&extractable)?;

        // Step 6: Format to string
        let output = formatter::format(&lumos_items, &self.config)?;

        Ok(output)
    }
}
```

---

## Step 1: Parse Rust with syn

**Input:** Rust source code as string

**Code:**
```rust
use syn::{parse_file, File as RustFile};

pub fn parse_rust(source: &str) -> syn::Result<RustFile> {
    syn::parse_file(source)
}
```

**Example:**

**Input:**
```rust
let rust_source = r#"
    use anchor_lang::prelude::*;

    #[account]
    pub struct Player {
        pub wallet: Pubkey,
        pub score: u64,
    }
"#;
```

**Output (syn AST):**
```rust
File {
    items: [
        Use(...),  // use anchor_lang::prelude::*;
        Struct {
            attrs: [Attribute { path: "account" }],
            ident: "Player",
            fields: Fields::Named(FieldsNamed {
                named: [
                    Field {
                        ident: "wallet",
                        ty: Type::Path("Pubkey"),
                    },
                    Field {
                        ident: "score",
                        ty: Type::Path("u64"),
                    }
                ]
            })
        }
    ]
}
```

---

## Step 2: Analyzer - Find Extractable Types

**Goal:** Walk AST and identify types we can extract

### 2.1 Main Analyzer Function

```rust
// packages/core/src/extract/analyzer.rs

use syn::{File, Item, ItemStruct, ItemEnum, Attribute};
use super::types::ExtractableItem;
use super::ExtractConfig;

pub fn analyze(
    rust_ast: &File,
    config: &ExtractConfig,
) -> Result<Vec<ExtractableItem>> {
    let mut extractable = Vec::new();

    // Walk all items in the file
    for item in &rust_ast.items {
        match item {
            Item::Struct(s) => {
                if should_extract_struct(s, config) {
                    extractable.push(extract_struct_info(s)?);
                }
            }
            Item::Enum(e) => {
                if should_extract_enum(e, config) {
                    extractable.push(extract_enum_info(e)?);
                }
            }
            _ => {
                // Skip other items (use, fn, impl, etc.)
            }
        }
    }

    Ok(extractable)
}
```

### 2.2 Check if Struct is Extractable

```rust
fn should_extract_struct(s: &ItemStruct, config: &ExtractConfig) -> bool {
    match config.filter {
        FilterType::Account => has_account_attr(&s.attrs),
        FilterType::Borsh => has_borsh_derive(&s.attrs),
        FilterType::All => {
            has_account_attr(&s.attrs) || has_borsh_derive(&s.attrs)
        }
    }
}

// Check for #[account] attribute
fn has_account_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("account")
    })
}

// Check for #[derive(BorshSerialize, BorshDeserialize)]
fn has_borsh_derive(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if let Ok(meta) = attr.parse_meta() {
            if let syn::Meta::List(list) = meta {
                if list.path.is_ident("derive") {
                    // Check if any derive is BorshSerialize
                    return list.nested.iter().any(|nested| {
                        if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = nested {
                            path.is_ident("BorshSerialize") ||
                            path.is_ident("AnchorSerialize")
                        } else {
                            false
                        }
                    });
                }
            }
        }
        false
    })
}
```

**Example usage:**

**Input:**
```rust
#[account]
pub struct Player {
    pub wallet: Pubkey,
}
```

**Result:**
```rust
has_account_attr(&player.attrs) == true
should_extract_struct(&player, &config) == true
```

### 2.3 Extract Struct Information

```rust
use super::types::{ExtractableItem, ExtractedField};

fn extract_struct_info(s: &ItemStruct) -> Result<ExtractableItem> {
    let is_account = has_account_attr(&s.attrs);

    // Extract fields
    let fields = extract_fields(&s.fields)?;

    // Extract doc comments
    let doc_comments = extract_doc_comments(&s.attrs);

    Ok(ExtractableItem::Struct {
        name: s.ident.to_string(),
        is_account,
        fields,
        doc_comments,
    })
}

fn extract_fields(fields: &syn::Fields) -> Result<Vec<ExtractedField>> {
    match fields {
        syn::Fields::Named(named) => {
            named.named.iter().map(|field| {
                Ok(ExtractedField {
                    name: field.ident.as_ref()
                        .ok_or("Field without name")?
                        .to_string(),
                    rust_type: type_to_string(&field.ty),
                    doc_comment: extract_doc_comments(&field.attrs),
                })
            }).collect()
        }
        _ => Err("Only named fields supported"),
    }
}

fn type_to_string(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(path) => {
            // Convert type path to string
            quote::quote!(#path).to_string()
        }
        _ => quote::quote!(#ty).to_string(),
    }
}

fn extract_doc_comments(attrs: &[Attribute]) -> Vec<String> {
    attrs.iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                // Parse doc comment
                if let Ok(meta) = attr.parse_meta() {
                    if let syn::Meta::NameValue(nv) = meta {
                        if let syn::Lit::Str(s) = nv.lit {
                            return Some(s.value().trim().to_string());
                        }
                    }
                }
            }
            None
        })
        .collect()
}
```

**Example:**

**Input:**
```rust
/// Player account
#[account]
pub struct Player {
    /// Wallet address
    pub wallet: Pubkey,
    pub score: u64,
}
```

**Output:**
```rust
ExtractableItem::Struct {
    name: "Player",
    is_account: true,
    fields: [
        ExtractedField {
            name: "wallet",
            rust_type: "Pubkey",
            doc_comment: ["Wallet address"],
        },
        ExtractedField {
            name: "score",
            rust_type: "u64",
            doc_comment: [],
        }
    ],
    doc_comments: ["Player account"],
}
```

### 2.4 Extract Enum Information

```rust
fn extract_enum_info(e: &ItemEnum) -> Result<ExtractableItem> {
    let variants = e.variants.iter().map(|variant| {
        ExtractedVariant {
            name: variant.ident.to_string(),
            kind: match &variant.fields {
                syn::Fields::Unit => VariantKind::Unit,
                syn::Fields::Unnamed(fields) => {
                    let types = fields.unnamed.iter()
                        .map(|f| type_to_string(&f.ty))
                        .collect();
                    VariantKind::Tuple(types)
                }
                syn::Fields::Named(fields) => {
                    let fields = extract_fields(&syn::Fields::Named(fields.clone()))?;
                    VariantKind::Struct(fields)
                }
            },
        }
    }).collect::<Result<Vec<_>>>()?;

    Ok(ExtractableItem::Enum {
        name: e.ident.to_string(),
        variants,
        doc_comments: extract_doc_comments(&e.attrs),
    })
}
```

---

## Step 3: Data Structures

```rust
// packages/core/src/extract/types.rs

pub enum ExtractableItem {
    Struct {
        name: String,
        is_account: bool,
        fields: Vec<ExtractedField>,
        doc_comments: Vec<String>,
    },
    Enum {
        name: String,
        variants: Vec<ExtractedVariant>,
        doc_comments: Vec<String>,
    },
}

pub struct ExtractedField {
    pub name: String,
    pub rust_type: String,  // e.g., "Pubkey", "Vec<u64>"
    pub doc_comment: Vec<String>,
}

pub struct ExtractedVariant {
    pub name: String,
    pub kind: VariantKind,
}

pub enum VariantKind {
    Unit,
    Tuple(Vec<String>),  // Type names
    Struct(Vec<ExtractedField>),
}
```

---

## Step 4: Converter - Rust Types to LUMOS Types

### 4.1 Main Conversion Function

```rust
// packages/core/src/extract/converter.rs

use crate::ast as lumos_ast;
use super::types::*;

pub fn convert_all(extractable: &[ExtractableItem]) -> Result<Vec<lumos_ast::Item>> {
    extractable.iter()
        .map(convert_item)
        .collect()
}

fn convert_item(item: &ExtractableItem) -> Result<lumos_ast::Item> {
    match item {
        ExtractableItem::Struct { name, is_account, fields, .. } => {
            convert_struct(name, *is_account, fields)
        }
        ExtractableItem::Enum { name, variants, .. } => {
            convert_enum(name, variants)
        }
    }
}
```

### 4.2 Convert Struct

```rust
fn convert_struct(
    name: &str,
    is_account: bool,
    fields: &[ExtractedField],
) -> Result<lumos_ast::Item> {
    // Build attributes
    let mut attributes = vec![
        lumos_ast::Attribute {
            name: "solana".to_string(),
            args: None,
        }
    ];

    if is_account {
        attributes.push(lumos_ast::Attribute {
            name: "account".to_string(),
            args: None,
        });
    }

    // Convert fields
    let lumos_fields = fields.iter()
        .map(convert_field)
        .collect::<Result<Vec<_>>>()?;

    Ok(lumos_ast::Item::Struct(lumos_ast::StructDef {
        attributes,
        visibility: None,  // LUMOS doesn't use pub
        name: name.to_string(),
        fields: lumos_fields,
    }))
}

fn convert_field(field: &ExtractedField) -> Result<lumos_ast::FieldDef> {
    Ok(lumos_ast::FieldDef {
        name: field.name.clone(),
        ty: convert_type(&field.rust_type)?,
    })
}
```

### 4.3 Type Conversion Logic (The Core!)

```rust
fn convert_type(rust_type: &str) -> Result<lumos_ast::Type> {
    // Remove whitespace for easier matching
    let ty = rust_type.replace(" ", "");

    // Primitive types - direct mapping
    match ty.as_str() {
        "u8" => return Ok(lumos_ast::Type::U8),
        "u16" => return Ok(lumos_ast::Type::U16),
        "u32" => return Ok(lumos_ast::Type::U32),
        "u64" => return Ok(lumos_ast::Type::U64),
        "u128" => return Ok(lumos_ast::Type::U128),
        "i8" => return Ok(lumos_ast::Type::I8),
        "i16" => return Ok(lumos_ast::Type::I16),
        "i32" => return Ok(lumos_ast::Type::I32),
        "i64" => return Ok(lumos_ast::Type::I64),
        "i128" => return Ok(lumos_ast::Type::I128),
        "bool" => return Ok(lumos_ast::Type::Bool),
        "String" => return Ok(lumos_ast::Type::String),

        // Solana types - special conversion
        "Pubkey" => return Ok(lumos_ast::Type::PublicKey),
        "solana_program::pubkey::Pubkey" => return Ok(lumos_ast::Type::PublicKey),

        _ => {}
    }

    // Vec<T> → [T]
    if ty.starts_with("Vec<") && ty.ends_with(">") {
        let inner = extract_generic_arg(&ty, "Vec")?;
        let inner_type = convert_type(&inner)?;
        return Ok(lumos_ast::Type::Array(Box::new(inner_type)));
    }

    // Option<T> → Option<T>
    if ty.starts_with("Option<") && ty.ends_with(">") {
        let inner = extract_generic_arg(&ty, "Option")?;
        let inner_type = convert_type(&inner)?;
        return Ok(lumos_ast::Type::Option(Box::new(inner_type)));
    }

    // Fixed-size array [T; N] → [T] (with warning)
    if ty.starts_with("[") && ty.contains(";") && ty.ends_with("]") {
        let inner = ty[1..ty.len()-1]
            .split(";")
            .next()
            .ok_or("Invalid array syntax")?
            .trim();

        eprintln!("Warning: Converting fixed array [{}; N] to dynamic array", inner);

        let inner_type = convert_type(inner)?;
        return Ok(lumos_ast::Type::Array(Box::new(inner_type)));
    }

    // Custom type (assume it's another struct/enum)
    Ok(lumos_ast::Type::Custom(ty))
}

fn extract_generic_arg(ty: &str, container: &str) -> Result<String> {
    let start = container.len() + 1;  // Skip "Container<"
    let end = ty.len() - 1;           // Skip ">"

    if start >= end {
        return Err("Invalid generic type");
    }

    Ok(ty[start..end].to_string())
}
```

**Type Conversion Examples:**

```rust
convert_type("Pubkey")              → PublicKey
convert_type("u64")                 → U64
convert_type("Vec<u64>")            → Array(U64)
convert_type("Vec<Pubkey>")         → Array(PublicKey)
convert_type("Option<String>")      → Option(String)
convert_type("Vec<Vec<u64>>")       → Array(Array(U64))
convert_type("[u64; 10]")           → Array(U64) + warning
convert_type("CustomStruct")        → Custom("CustomStruct")
```

### 4.4 Convert Enum

```rust
fn convert_enum(
    name: &str,
    variants: &[ExtractedVariant],
) -> Result<lumos_ast::Item> {
    let lumos_variants = variants.iter()
        .map(convert_variant)
        .collect::<Result<Vec<_>>>()?;

    Ok(lumos_ast::Item::Enum(lumos_ast::EnumDef {
        attributes: vec![
            lumos_ast::Attribute {
                name: "solana".to_string(),
                args: None,
            }
        ],
        name: name.to_string(),
        variants: lumos_variants,
    }))
}

fn convert_variant(variant: &ExtractedVariant) -> Result<lumos_ast::EnumVariant> {
    let kind = match &variant.kind {
        VariantKind::Unit => {
            lumos_ast::EnumVariantDef::Unit {
                name: variant.name.clone(),
            }
        }
        VariantKind::Tuple(types) => {
            let lumos_types = types.iter()
                .map(|t| convert_type(t))
                .collect::<Result<Vec<_>>>()?;

            lumos_ast::EnumVariantDef::Tuple {
                name: variant.name.clone(),
                types: lumos_types,
            }
        }
        VariantKind::Struct(fields) => {
            let lumos_fields = fields.iter()
                .map(convert_field)
                .collect::<Result<Vec<_>>>()?;

            lumos_ast::EnumVariantDef::Struct {
                name: variant.name.clone(),
                fields: lumos_fields,
            }
        }
    };

    Ok(kind)
}
```

---

## Step 5: Formatter - LUMOS AST to String

```rust
// packages/core/src/extract/formatter.rs

use crate::ast as lumos_ast;
use super::ExtractConfig;

pub fn format(items: &[lumos_ast::Item], config: &ExtractConfig) -> Result<String> {
    let mut output = String::new();

    // Add header comment
    if config.add_metadata {
        output.push_str(&format_header());
    }

    // Format each item
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            output.push_str("\n\n");
        }

        output.push_str(&format_item(item, config)?);
    }

    Ok(output)
}

fn format_header() -> String {
    format!(
        "// Extracted by LUMOS\n\
         // Generated: {}\n\
         \n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    )
}

fn format_item(item: &lumos_ast::Item, config: &ExtractConfig) -> Result<String> {
    match item {
        lumos_ast::Item::Struct(s) => format_struct(s, config),
        lumos_ast::Item::Enum(e) => format_enum(e, config),
    }
}

fn format_struct(s: &lumos_ast::StructDef, config: &ExtractConfig) -> Result<String> {
    let mut output = String::new();

    // Attributes
    for attr in &s.attributes {
        output.push_str(&format!("#[{}]\n", attr.name));
    }

    // Struct declaration
    output.push_str(&format!("struct {} {{\n", s.name));

    // Fields
    for field in &s.fields {
        output.push_str(&format!(
            "    {}: {},\n",
            field.name,
            format_type(&field.ty)
        ));
    }

    output.push_str("}");

    Ok(output)
}

fn format_type(ty: &lumos_ast::Type) -> String {
    match ty {
        lumos_ast::Type::U8 => "u8".to_string(),
        lumos_ast::Type::U16 => "u16".to_string(),
        lumos_ast::Type::U32 => "u32".to_string(),
        lumos_ast::Type::U64 => "u64".to_string(),
        lumos_ast::Type::U128 => "u128".to_string(),
        lumos_ast::Type::I8 => "i8".to_string(),
        lumos_ast::Type::I16 => "i16".to_string(),
        lumos_ast::Type::I32 => "i32".to_string(),
        lumos_ast::Type::I64 => "i64".to_string(),
        lumos_ast::Type::I128 => "i128".to_string(),
        lumos_ast::Type::Bool => "bool".to_string(),
        lumos_ast::Type::String => "String".to_string(),
        lumos_ast::Type::PublicKey => "PublicKey".to_string(),
        lumos_ast::Type::Array(inner) => {
            format!("[{}]", format_type(inner))
        }
        lumos_ast::Type::Option(inner) => {
            format!("Option<{}>", format_type(inner))
        }
        lumos_ast::Type::Custom(name) => name.clone(),
    }
}

fn format_enum(e: &lumos_ast::EnumDef, config: &ExtractConfig) -> Result<String> {
    let mut output = String::new();

    // Attributes
    for attr in &e.attributes {
        output.push_str(&format!("#[{}]\n", attr.name));
    }

    // Enum declaration
    output.push_str(&format!("enum {} {{\n", e.name));

    // Variants
    for variant in &e.variants {
        match variant {
            lumos_ast::EnumVariantDef::Unit { name } => {
                output.push_str(&format!("    {},\n", name));
            }
            lumos_ast::EnumVariantDef::Tuple { name, types } => {
                let types_str = types.iter()
                    .map(format_type)
                    .collect::<Vec<_>>()
                    .join(", ");
                output.push_str(&format!("    {}({}),\n", name, types_str));
            }
            lumos_ast::EnumVariantDef::Struct { name, fields } => {
                output.push_str(&format!("    {} {{\n", name));
                for field in fields {
                    output.push_str(&format!(
                        "        {}: {},\n",
                        field.name,
                        format_type(&field.ty)
                    ));
                }
                output.push_str("    },\n");
            }
        }
    }

    output.push_str("}");

    Ok(output)
}
```

---

## Complete Example: End-to-End

### Input: Rust Code

```rust
use anchor_lang::prelude::*;

/// Player account storing game state
#[account]
pub struct Player {
    /// Owner wallet
    pub wallet: Pubkey,

    /// Current score
    pub score: u64,

    /// Inventory items
    pub items: Vec<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum GameEvent {
    PlayerJoined(Pubkey),
    ScoreUpdated { player: Pubkey, score: u64 },
    GameEnded,
}
```

### Execution Flow

**Step 1: Parse**
```rust
let ast = syn::parse_file(rust_source)?;
// Creates syn AST with all items
```

**Step 2: Analyze**
```rust
let extractable = analyzer::analyze(&ast, &config)?;

// Result:
vec![
    ExtractableItem::Struct {
        name: "Player",
        is_account: true,
        fields: [
            ExtractedField { name: "wallet", rust_type: "Pubkey", .. },
            ExtractedField { name: "score", rust_type: "u64", .. },
            ExtractedField { name: "items", rust_type: "Vec<Pubkey>", .. },
        ],
        doc_comments: ["Player account storing game state"],
    },
    ExtractableItem::Enum {
        name: "GameEvent",
        variants: [
            ExtractedVariant {
                name: "PlayerJoined",
                kind: VariantKind::Tuple(vec!["Pubkey"]),
            },
            ExtractedVariant {
                name: "ScoreUpdated",
                kind: VariantKind::Struct(vec![
                    ExtractedField { name: "player", rust_type: "Pubkey", .. },
                    ExtractedField { name: "score", rust_type: "u64", .. },
                ]),
            },
            ExtractedVariant {
                name: "GameEnded",
                kind: VariantKind::Unit,
            },
        ],
        doc_comments: [],
    }
]
```

**Step 3-4: Convert**
```rust
let lumos_items = converter::convert_all(&extractable)?;

// Result: LUMOS AST
vec![
    lumos_ast::Item::Struct(lumos_ast::StructDef {
        attributes: vec![
            Attribute { name: "solana" },
            Attribute { name: "account" },
        ],
        name: "Player",
        fields: vec![
            FieldDef { name: "wallet", ty: Type::PublicKey },
            FieldDef { name: "score", ty: Type::U64 },
            FieldDef { name: "items", ty: Type::Array(Box::new(Type::PublicKey)) },
        ],
    }),
    lumos_ast::Item::Enum(lumos_ast::EnumDef {
        attributes: vec![Attribute { name: "solana" }],
        name: "GameEvent",
        variants: vec![
            EnumVariantDef::Tuple {
                name: "PlayerJoined",
                types: vec![Type::PublicKey],
            },
            EnumVariantDef::Struct {
                name: "ScoreUpdated",
                fields: vec![
                    FieldDef { name: "player", ty: Type::PublicKey },
                    FieldDef { name: "score", ty: Type::U64 },
                ],
            },
            EnumVariantDef::Unit { name: "GameEnded" },
        ],
    }),
]
```

**Step 5: Format**
```rust
let output = formatter::format(&lumos_items, &config)?;
```

### Output: LUMOS Schema

```rust
// Player account storing game state
#[solana]
#[account]
struct Player {
    wallet: PublicKey,
    score: u64,
    items: [PublicKey],
}

#[solana]
enum GameEvent {
    PlayerJoined(PublicKey),
    ScoreUpdated {
        player: PublicKey,
        score: u64,
    },
    GameEnded,
}
```

---

## Testing the Extractor

### Unit Test Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_struct() {
        let rust_code = r#"
            #[account]
            pub struct Player {
                pub wallet: Pubkey,
                pub score: u64,
            }
        "#;

        let extractor = Extractor {
            config: ExtractConfig {
                filter: FilterType::All,
                preserve_comments: false,
            },
        };

        let result = extractor.extract(rust_code).unwrap();

        assert!(result.contains("#[solana]"));
        assert!(result.contains("#[account]"));
        assert!(result.contains("struct Player"));
        assert!(result.contains("wallet: PublicKey"));
        assert!(result.contains("score: u64"));
    }

    #[test]
    fn test_type_conversion() {
        assert_eq!(
            convert_type("Pubkey").unwrap(),
            lumos_ast::Type::PublicKey
        );

        assert_eq!(
            convert_type("Vec<u64>").unwrap(),
            lumos_ast::Type::Array(Box::new(lumos_ast::Type::U64))
        );

        assert_eq!(
            convert_type("Option<String>").unwrap(),
            lumos_ast::Type::Option(Box::new(lumos_ast::Type::String))
        );
    }

    #[test]
    fn test_nested_types() {
        let result = convert_type("Vec<Vec<u64>>").unwrap();

        match result {
            lumos_ast::Type::Array(inner) => {
                match *inner {
                    lumos_ast::Type::Array(inner2) => {
                        assert_eq!(*inner2, lumos_ast::Type::U64);
                    }
                    _ => panic!("Expected nested array"),
                }
            }
            _ => panic!("Expected array type"),
        }
    }

    #[test]
    fn test_extract_enum() {
        let rust_code = r#"
            #[derive(BorshSerialize, BorshDeserialize)]
            pub enum Status {
                Active,
                Paused,
            }
        "#;

        let result = extractor.extract(rust_code).unwrap();

        assert!(result.contains("enum Status"));
        assert!(result.contains("Active,"));
        assert!(result.contains("Paused,"));
    }
}
```

---

## Performance Considerations

### Optimization Strategies

**1. Parallel processing for multiple files:**
```rust
use rayon::prelude::*;

pub fn extract_directory(dir: &Path) -> Result<String> {
    let rust_files: Vec<_> = find_rust_files(dir)?;

    let results: Vec<_> = rust_files.par_iter()
        .map(|file| {
            let content = std::fs::read_to_string(file)?;
            extractor.extract(&content)
        })
        .collect::<Result<Vec<_>>>()?;

    // Merge results
    Ok(results.join("\n\n"))
}
```

**2. Caching for repeated extractions:**
```rust
use std::collections::HashMap;

pub struct CachingExtractor {
    cache: HashMap<String, String>,  // source hash → result
    extractor: Extractor,
}

impl CachingExtractor {
    pub fn extract(&mut self, source: &str) -> Result<String> {
        let hash = hash_source(source);

        if let Some(cached) = self.cache.get(&hash) {
            return Ok(cached.clone());
        }

        let result = self.extractor.extract(source)?;
        self.cache.insert(hash, result.clone());

        Ok(result)
    }
}
```

---

## Error Handling

### Custom Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("Failed to parse Rust code: {0}")]
    ParseError(#[from] syn::Error),

    #[error("Unsupported type: {0}")]
    UnsupportedType(String),

    #[error("Invalid field: {0}")]
    InvalidField(String),

    #[error("No extractable types found")]
    NoExtractableTypes,
}

pub type Result<T> = std::result::Result<T, ExtractError>;
```

### Graceful Degradation

```rust
fn convert_type_safe(rust_type: &str) -> Result<lumos_ast::Type> {
    match convert_type(rust_type) {
        Ok(ty) => Ok(ty),
        Err(_) => {
            eprintln!("Warning: Could not convert type '{}', using as custom type", rust_type);
            Ok(lumos_ast::Type::Custom(rust_type.to_string()))
        }
    }
}
```

---

## Summary: Key Logic Components

### 1. **Parser** (syn crate)
- Converts Rust source → AST
- Already battle-tested library
- Handles all Rust syntax

### 2. **Analyzer** (Custom)
- Walks AST to find extractable items
- Checks for #[account] or Borsh derives
- Extracts metadata (names, fields, types)

### 3. **Type Converter** (Custom - The Heart!)
- Maps Rust types → LUMOS types
- Handles primitives, Solana types, containers
- Smart conversions (Vec → Array, Pubkey → PublicKey)

### 4. **Formatter** (Custom)
- LUMOS AST → String
- Pretty-prints with proper indentation
- Adds attributes and comments

### Critical Type Conversion Table

| Rust Input | Parse | Convert | LUMOS Output |
|------------|-------|---------|--------------|
| `Pubkey` | ✅ | `Type::PublicKey` | `PublicKey` |
| `Vec<u64>` | ✅ | `Type::Array(U64)` | `[u64]` |
| `Option<String>` | ✅ | `Type::Option(String)` | `Option<String>` |
| `[u64; 10]` | ✅ | `Type::Array(U64)` + ⚠️ | `[u64]` |
| `CustomStruct` | ✅ | `Type::Custom("CustomStruct")` | `CustomStruct` |

**The entire extraction process is deterministic and testable!**

---

**This is the complete implementation logic for the LUMOS extractor!** Ready to code when you reach Phase 3.3! 🚀
