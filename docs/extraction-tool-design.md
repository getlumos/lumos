# LUMOS Extraction Tool Design

**Feature:** Automatic extraction of Rust types to LUMOS schema
**Status:** Design Phase
**Target:** Phase 3.3+
**Owner:** RECTOR

---

## Overview

The extraction tool automatically converts existing Rust code to LUMOS schema format, making migration from existing projects seamless.

**Goal:** One-command conversion from legacy Rust types to LUMOS.

```bash
# Convert existing Rust to LUMOS
lumos extract programs/game/src/state.rs
```

---

## Table of Contents

1. [CLI Interface](#cli-interface)
2. [Extraction Logic](#extraction-logic)
3. [Type Mapping](#type-mapping)
4. [Edge Cases](#edge-cases)
5. [Interactive Mode](#interactive-mode)
6. [Implementation Architecture](#implementation-architecture)
7. [Examples](#examples)

---

## CLI Interface

### Command: `lumos extract`

**Synopsis:**
```bash
lumos extract [OPTIONS] <INPUT>
```

### Arguments

**`<INPUT>`** (required)
- Path to Rust file or directory
- Examples: `src/state.rs`, `programs/game/src/`

### Options

**Output:**
- `--output <FILE>` - Output file path (default: stdout)
- `-o <FILE>` - Short alias

**Filtering:**
- `--filter <TYPE>` - Filter by type (account, borsh, all)
  - `account` - Only `#[account]` structs
  - `borsh` - Types with BorshSerialize
  - `all` - All serializable types (default)
- `--pattern <REGEX>` - Filter by name pattern
- `--exclude <PATTERN>` - Exclude matching types

**Behavior:**
- `--recursive` - Process directories recursively
- `-r` - Short alias
- `--dry-run` - Show what would be extracted without writing
- `--interactive` - Interactive mode with prompts
- `-i` - Short alias

**Formatting:**
- `--preserve-comments` - Include Rust doc comments
- `--add-metadata` - Add extraction metadata
- `--group-by <STRATEGY>` - Grouping strategy (file, module, type)

**Advanced:**
- `--merge <FILE>` - Merge with existing LUMOS file
- `--validate` - Validate extracted schema
- `--verbose` - Verbose output
- `-v` - Short alias

### Examples

**Basic extraction:**
```bash
lumos extract src/state.rs
```

**Extract to file:**
```bash
lumos extract src/state.rs --output schema.lumos
```

**Extract only #[account] structs:**
```bash
lumos extract src/state.rs --filter account -o accounts.lumos
```

**Recursive extraction:**
```bash
lumos extract programs/game/src/ --recursive --output game-schema.lumos
```

**Interactive mode:**
```bash
lumos extract src/state.rs --interactive
```

**Dry run (preview):**
```bash
lumos extract src/state.rs --dry-run
```

**Merge with existing schema:**
```bash
lumos extract src/new-types.rs --merge schema.lumos --output schema.lumos
```

---

## Extraction Logic

### What Gets Extracted

#### 1. Anchor Account Structs

**Detects:**
- `#[account]` attribute
- `#[account(zero_copy)]`
- Anchor imports

**Example:**
```rust
use anchor_lang::prelude::*;

#[account]
pub struct PlayerAccount {
    pub wallet: Pubkey,
    pub score: u64,
}
```

**Extracts to:**
```rust
#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    score: u64,
}
```

---

#### 2. Borsh-Serialized Structs

**Detects:**
- `#[derive(BorshSerialize, BorshDeserialize)]`
- `borsh::BorshSerialize` trait bounds

**Example:**
```rust
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct GameConfig {
    pub max_players: u32,
    pub entry_fee: u64,
}
```

**Extracts to:**
```rust
#[solana]
struct GameConfig {
    max_players: u32,
    entry_fee: u64,
}
```

---

#### 3. Enums

**Detects:**
- Enums with Borsh/Anchor derives
- All variant types (unit, tuple, struct)

**Example:**
```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum GameEvent {
    PlayerJoined(Pubkey),
    ScoreUpdated { player: Pubkey, score: u64 },
    GameEnded,
}
```

**Extracts to:**
```rust
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

#### 4. Nested Types

**Detects:**
- Types used as fields in extractable structs
- Recursively extracts dependencies

**Example:**
```rust
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Attribute {
    pub trait_type: String,
    pub value: String,
}

#[account]
pub struct NFTMetadata {
    pub mint: Pubkey,
    pub attributes: Vec<Attribute>,  // ← Dependency
}
```

**Extracts to:**
```rust
#[solana]
struct Attribute {
    trait_type: String,
    value: String,
}

#[solana]
#[account]
struct NFTMetadata {
    mint: PublicKey,
    attributes: [Attribute],
}
```

---

### What Gets Skipped

**Not extractable:**

1. **Internal types without derives:**
   ```rust
   struct InternalHelper {  // No derives
       temp_data: u64,
   }
   ```

2. **Non-serializable types:**
   ```rust
   struct WithLifetime<'a> {  // Lifetimes not supported
       data: &'a str,
   }
   ```

3. **Generic types (Phase 1):**
   ```rust
   struct Container<T> {  // Generics not supported yet
       value: T,
   }
   ```

4. **Types with custom serialization:**
   ```rust
   #[derive(Serialize)]  // Not Borsh
   struct CustomSerialized { ... }
   ```

---

## Type Mapping

### Rust → LUMOS Type Conversion

| Rust Type | LUMOS Type | Notes |
|-----------|------------|-------|
| `Pubkey` | `PublicKey` | Solana public key |
| `u8`, `u16`, `u32`, `u64`, `u128` | Same | Unsigned integers |
| `i8`, `i16`, `i32`, `i64`, `i128` | Same | Signed integers |
| `bool` | `bool` | Boolean |
| `String` | `String` | Dynamic string |
| `Vec<T>` | `[T]` | Dynamic array |
| `Option<T>` | `Option<T>` | Optional value |
| `[T; N]` | **Warning** | Fixed arrays pending |
| Custom struct | Same name | If extractable |

### Field Visibility

**Rust `pub` modifier is removed:**

```rust
pub struct Player {
    pub wallet: Pubkey,   // ← pub removed
    score: u64,           // ← pub removed
}
```

```rust
struct Player {
    wallet: PublicKey,
    score: u64,
}
```

**Reason:** LUMOS generates appropriate visibility automatically.

---

## Edge Cases

### Case 1: Name Conflicts

**Problem:** Multiple types with same name

**Input:**
```rust
// src/accounts.rs
pub struct Player { ... }

// src/entities.rs
pub struct Player { ... }  // ← Conflict!
```

**Solution: Namespace with module name**

**Output:**
```rust
// From src/accounts.rs
#[namespace("accounts")]
struct Player { ... }

// From src/entities.rs
#[namespace("entities")]
struct Player { ... }
```

**Or: Interactive prompt**
```
⚠️  Name conflict detected: Player
  Found in:
    1. src/accounts.rs
    2. src/entities.rs

  Choose action:
  [1] Rename to accounts::Player
  [2] Rename to entities::Player
  [3] Skip duplicate
  [4] Manual rename

  > 1
```

---

### Case 2: Unsupported Types

**Problem:** Type cannot be mapped to LUMOS

**Input:**
```rust
#[account]
pub struct Complex {
    pub data: HashMap<String, Vec<u64>>,  // ← Not supported
}
```

**Solution: Warning with manual intervention**

**Output:**
```
⚠️  Unsupported type in Complex::data
  Type: HashMap<String, Vec<u64>>
  Location: src/state.rs:42

  Suggestions:
  - Replace with Vec<(String, Vec<u64>)>
  - Use custom serialization
  - Skip this field

  Action: [Skip field / Manual edit / Abort]
```

**Generated (with placeholder):**
```rust
#[solana]
#[account]
struct Complex {
    // TODO: Unsupported type HashMap<String, Vec<u64>>
    // Original field: data: HashMap<String, Vec<u64>>
    // Suggestion: Replace with alternative structure
}
```

---

### Case 3: Doc Comments

**Input:**
```rust
/// Player account storing user game state
///
/// Updated when player joins or levels up
#[account]
pub struct PlayerAccount {
    /// Owner's wallet address
    pub wallet: Pubkey,

    /// Current player level
    pub level: u16,
}
```

**With `--preserve-comments`:**

```rust
// Player account storing user game state
//
// Updated when player joins or levels up
#[solana]
#[account]
struct PlayerAccount {
    // Owner's wallet address
    wallet: PublicKey,

    // Current player level
    level: u16,
}
```

**Conversion:** `///` → `//`, `/** ... */` → `/* ... */`

---

### Case 4: Derive Macros

**Input:**
```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct Player {
    pub wallet: Pubkey,
}
```

**Smart detection:**
- **Anchor derives** → `#[solana]` attribute
- **Borsh derives** → `#[solana]` attribute
- **Other derives** → Ignored (generated by LUMOS)

**Output:**
```rust
#[solana]
struct Player {
    wallet: PublicKey,
}
```

---

### Case 5: Circular Dependencies

**Problem:** Types reference each other

**Input:**
```rust
pub struct Player {
    pub guild: Pubkey,  // Reference to Guild account
}

pub struct Guild {
    pub members: Vec<Pubkey>,  // References to Players
}
```

**Solution: Both extracted (works fine)**

```rust
#[solana]
#[account]
struct Player {
    guild: PublicKey,
}

#[solana]
#[account]
struct Guild {
    members: [PublicKey],
}
```

**Note:** LUMOS uses Pubkey references, not direct struct embedding.

---

## Interactive Mode

**Activated with `--interactive` or `-i`**

### Flow

```
$ lumos extract src/state.rs --interactive

🔍 Analyzing src/state.rs...

Found 8 types:
  ✓ PlayerAccount (#[account])
  ✓ GameState (#[account])
  ✓ Guild (BorshSerialize)
  ✓ Metadata (BorshSerialize)
  ⚠ InternalCache (no derives) - skip?
  ⚠ Helper (non-serializable) - skip?
  ✗ WithLifetime (unsupported) - cannot extract
  ✓ GameEvent (enum)

Extract all compatible types? [Y/n] y

📝 Extraction options:
  Output file: schema.lumos
  Preserve comments? [Y/n] y
  Include metadata? [y/N] n
  Grouping: by type

✓ Extracting 6 types...

  [1/6] PlayerAccount ✓
  [2/6] GameState ✓
  [3/6] Guild ✓
  [4/6] Metadata ✓
  [5/6] GameEvent ✓
  [6/6] Dependencies (2 types) ✓

✓ Extracted to schema.lumos

Summary:
  Extracted: 6 types
  Skipped: 2 types (1 unsupported, 1 no derives)
  Warnings: 0
  Lines generated: 120

Next steps:
  1. Review schema.lumos
  2. Run: lumos validate schema.lumos
  3. Generate code: lumos generate schema.lumos
```

---

### Interactive Prompts

**1. Name conflicts:**
```
⚠️  Name conflict: Player

  Found in:
    1. src/accounts/mod.rs (pub struct Player)
    2. src/entities/mod.rs (pub struct Player)

  Resolve conflict:
    [1] Prefix with module name (accounts::Player, entities::Player)
    [2] Keep first, skip second
    [3] Rename manually
    [4] Skip both

  > 1

✓ Renamed to accounts::Player and entities::Player
```

**2. Unsupported types:**
```
⚠️  Unsupported field: Complex::data
  Type: HashMap<String, u64>

  Options:
    [1] Skip this field
    [2] Add TODO comment
    [3] Abort extraction
    [4] Custom replacement

  > 2

✓ Added TODO comment for manual resolution
```

**3. Ambiguous derives:**
```
⚠️  Struct Item has multiple serialization derives:
  - BorshSerialize
  - serde::Serialize

  Which one to use?
    [1] Borsh (recommended for Solana)
    [2] Serde (skip, not compatible)
    [3] Skip this type

  > 1

✓ Using Borsh serialization
```

---

## Implementation Architecture

### High-Level Flow

```
┌─────────────────────────────────────────────────────┐
│ Input: Rust source file(s)                          │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ Parser: syn crate (already used in LUMOS)           │
│ - Parse Rust to AST                                 │
│ - Extract ItemStruct, ItemEnum                      │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ Analyzer: Detect extractable types                  │
│ - Check for #[account]                              │
│ - Check for Borsh derives                           │
│ - Identify dependencies                             │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ Converter: Rust AST → LUMOS AST                     │
│ - Map Rust types to LUMOS types                     │
│ - Convert field definitions                         │
│ - Handle attributes                                 │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ Validator: Check LUMOS AST                          │
│ - Verify type compatibility                         │
│ - Detect unsupported features                       │
│ - Generate warnings                                 │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ Generator: LUMOS AST → LUMOS source                 │
│ - Format LUMOS syntax                               │
│ - Add comments and metadata                         │
│ - Write to file                                     │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│ Output: schema.lumos                                │
└─────────────────────────────────────────────────────┘
```

---

### Module Structure

```rust
// packages/core/src/extract/mod.rs

pub mod analyzer;    // Detect extractable types
pub mod converter;   // Rust AST → LUMOS AST
pub mod validator;   // Check compatibility
pub mod formatter;   // Format LUMOS output

use syn::File;

pub struct Extractor {
    config: ExtractorConfig,
}

pub struct ExtractorConfig {
    pub filter: FilterType,
    pub preserve_comments: bool,
    pub add_metadata: bool,
    pub interactive: bool,
}

pub enum FilterType {
    Account,      // Only #[account]
    Borsh,        // BorshSerialize types
    All,          // All serializable
}

impl Extractor {
    pub fn new(config: ExtractorConfig) -> Self {
        Self { config }
    }

    pub fn extract(&self, rust_source: &str) -> Result<String> {
        // 1. Parse Rust
        let ast = syn::parse_file(rust_source)?;

        // 2. Analyze and filter
        let extractable = analyzer::find_extractable(&ast, &self.config)?;

        // 3. Convert to LUMOS AST
        let lumos_ast = converter::convert(&extractable)?;

        // 4. Validate
        validator::validate(&lumos_ast)?;

        // 5. Format and generate
        let output = formatter::format(&lumos_ast, &self.config)?;

        Ok(output)
    }
}
```

---

### Analyzer Module

```rust
// packages/core/src/extract/analyzer.rs

use syn::{ItemStruct, ItemEnum, Attribute};

pub struct ExtractableType {
    pub name: String,
    pub kind: TypeKind,
    pub attributes: Vec<ExtractedAttribute>,
    pub fields: Vec<ExtractedField>,
}

pub enum TypeKind {
    Struct { is_account: bool },
    Enum,
}

pub fn find_extractable(
    ast: &syn::File,
    config: &ExtractorConfig,
) -> Result<Vec<ExtractableType>> {
    let mut types = Vec::new();

    for item in &ast.items {
        match item {
            syn::Item::Struct(s) => {
                if is_extractable_struct(s, config) {
                    types.push(extract_struct(s)?);
                }
            }
            syn::Item::Enum(e) => {
                if is_extractable_enum(e, config) {
                    types.push(extract_enum(e)?);
                }
            }
            _ => {}
        }
    }

    Ok(types)
}

fn is_extractable_struct(s: &ItemStruct, config: &ExtractorConfig) -> bool {
    match config.filter {
        FilterType::Account => has_account_attribute(&s.attrs),
        FilterType::Borsh => has_borsh_derive(&s.attrs),
        FilterType::All => {
            has_account_attribute(&s.attrs) || has_borsh_derive(&s.attrs)
        }
    }
}

fn has_account_attribute(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("account")
    })
}

fn has_borsh_derive(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("derive") {
            // Parse derive list and check for BorshSerialize
            // Implementation details...
            true
        } else {
            false
        }
    })
}
```

---

### Converter Module

```rust
// packages/core/src/extract/converter.rs

use crate::ast as lumos_ast;

pub fn convert(extractable: &[ExtractableType]) -> Result<Vec<lumos_ast::Item>> {
    extractable
        .iter()
        .map(convert_type)
        .collect()
}

fn convert_type(t: &ExtractableType) -> Result<lumos_ast::Item> {
    match &t.kind {
        TypeKind::Struct { is_account } => {
            convert_struct(t, *is_account)
        }
        TypeKind::Enum => {
            convert_enum(t)
        }
    }
}

fn convert_struct(t: &ExtractableType, is_account: bool) -> Result<lumos_ast::Item> {
    let mut attributes = vec![
        lumos_ast::Attribute {
            name: "solana".to_string(),
        },
    ];

    if is_account {
        attributes.push(lumos_ast::Attribute {
            name: "account".to_string(),
        });
    }

    let fields = t.fields
        .iter()
        .map(convert_field)
        .collect::<Result<Vec<_>>>()?;

    Ok(lumos_ast::Item::Struct(lumos_ast::StructDef {
        attributes,
        name: t.name.clone(),
        fields,
    }))
}

fn convert_field(f: &ExtractedField) -> Result<lumos_ast::FieldDef> {
    Ok(lumos_ast::FieldDef {
        name: f.name.clone(),
        ty: convert_type_path(&f.rust_type)?,
    })
}

fn convert_type_path(rust_ty: &str) -> Result<lumos_ast::Type> {
    match rust_ty {
        "Pubkey" => Ok(lumos_ast::Type::PublicKey),
        "String" => Ok(lumos_ast::Type::String),
        "u64" => Ok(lumos_ast::Type::U64),
        "bool" => Ok(lumos_ast::Type::Bool),
        // Vec<T> → [T]
        ty if ty.starts_with("Vec<") => {
            let inner = extract_generic_param(ty)?;
            Ok(lumos_ast::Type::Array(Box::new(convert_type_path(&inner)?)))
        }
        // Option<T> → Option<T>
        ty if ty.starts_with("Option<") => {
            let inner = extract_generic_param(ty)?;
            Ok(lumos_ast::Type::Option(Box::new(convert_type_path(&inner)?)))
        }
        // Custom type
        ty => Ok(lumos_ast::Type::Custom(ty.to_string())),
    }
}
```

---

## Examples

### Example 1: Simple Extraction

**Input:** `programs/game/src/state.rs`

```rust
use anchor_lang::prelude::*;

#[account]
pub struct PlayerAccount {
    pub wallet: Pubkey,
    pub score: u64,
    pub level: u16,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct GameConfig {
    pub max_players: u32,
}
```

**Command:**
```bash
lumos extract programs/game/src/state.rs --output schema.lumos
```

**Output:** `schema.lumos`

```rust
// Extracted from programs/game/src/state.rs
// Generated: 2025-01-18 10:30:00

#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    score: u64,
    level: u16,
}

#[solana]
struct GameConfig {
    max_players: u32,
}
```

---

### Example 2: Recursive Extraction

**Directory structure:**
```
programs/game/src/
├── lib.rs
├── state/
│   ├── mod.rs
│   ├── player.rs
│   └── game.rs
└── events.rs
```

**Command:**
```bash
lumos extract programs/game/src/ --recursive --output game-schema.lumos
```

**Output:** `game-schema.lumos`

```rust
// Extracted from programs/game/src/
// Files processed: 4
// Types extracted: 8
// Generated: 2025-01-18 10:30:00

// From programs/game/src/state/player.rs
#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    score: u64,
}

// From programs/game/src/state/game.rs
#[solana]
#[account]
struct GameState {
    authority: PublicKey,
    active: bool,
}

// From programs/game/src/events.rs
#[solana]
enum GameEvent {
    PlayerJoined(PublicKey),
    GameStarted,
}
```

---

### Example 3: Filtered Extraction

**Command:**
```bash
lumos extract src/state.rs --filter account --dry-run
```

**Output (preview):**
```
🔍 Analyzing src/state.rs...

Found types:
  ✓ PlayerAccount (#[account]) - will extract
  ✓ GameState (#[account]) - will extract
  ⊘ GameConfig (BorshSerialize only) - skipped (filter: account)
  ⊘ Helper (no derives) - skipped (not extractable)

Would extract 2 types:
  - PlayerAccount
  - GameState

Total lines: ~45

Run without --dry-run to perform extraction.
```

---

### Example 4: Interactive Extraction

**Command:**
```bash
lumos extract src/state.rs --interactive
```

**Session:**
```
🔍 Analyzing src/state.rs...

Found 5 types:
  1. [✓] PlayerAccount (#[account])
  2. [✓] GameState (#[account])
  3. [✓] GameConfig (BorshSerialize)
  4. [⚠] InternalCache (no derives)
  5. [✗] Helper<T> (generic - unsupported)

Select types to extract:
  [a] All compatible (1, 2, 3)
  [n] Choose individually
  [q] Quit

> n

Extract PlayerAccount? [Y/n] y
Extract GameState? [Y/n] y
Extract GameConfig? [Y/n] n

Selected: 2 types

Output file: [schema.lumos]
> game-accounts.lumos

Preserve Rust comments? [Y/n] y
Add generation metadata? [y/N] y

✓ Extracting...

  PlayerAccount ✓
  GameState ✓

✓ Written to game-accounts.lumos (78 lines)

Next: lumos validate game-accounts.lumos
```

---

### Example 5: Merge with Existing

**Existing:** `schema.lumos`
```rust
#[solana]
struct OldType {
    data: u64,
}
```

**Command:**
```bash
lumos extract src/new-types.rs --merge schema.lumos --output schema.lumos
```

**Result:** `schema.lumos`
```rust
// Existing types
#[solana]
struct OldType {
    data: u64,
}

// Extracted from src/new-types.rs (2025-01-18)
#[solana]
#[account]
struct NewType {
    value: String,
}
```

---

## Output Format

### Standard Format

```rust
// Extraction metadata (if --add-metadata)
// Source: programs/game/src/state.rs
// Extracted: 2025-01-18 10:30:00
// LUMOS version: 0.1.0

// Original: src/state.rs:10
#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    score: u64,
}
```

### Grouped by Module

**With `--group-by module`:**

```rust
// Module: accounts
#[solana]
#[account]
struct PlayerAccount { ... }

#[solana]
#[account]
struct GameAccount { ... }

// Module: events
#[solana]
enum GameEvent { ... }
```

### With Preserved Comments

**With `--preserve-comments`:**

```rust
// Player account storing game state
// Updated on every action
#[solana]
#[account]
struct PlayerAccount {
    // Player's wallet address
    wallet: PublicKey,

    // Current score (accumulated points)
    score: u64,
}
```

---

## Success Metrics

### User Experience Goals

- ⏱️ **Speed:** Extract 100 types in < 5 seconds
- 🎯 **Accuracy:** > 95% correct type conversion
- ⚠️ **Warnings:** Clear, actionable warnings for edge cases
- 🤝 **Interactivity:** Helpful prompts for ambiguous cases

### Technical Goals

- ✅ Support all Anchor account types
- ✅ Support all Borsh-serializable types
- ✅ Handle 90% of real-world Solana projects
- ✅ Graceful handling of unsupported types

---

## Future Enhancements

### Phase 1 (v0.2.0)
- Basic struct extraction
- Enum support
- Simple type mapping

### Phase 2 (v0.3.0)
- Interactive mode
- Comment preservation
- Merge with existing schemas

### Phase 3 (v0.4.0)
- TypeScript → LUMOS extraction
- Generic type support (limited)
- Custom derive detection

### Phase 4 (v0.5.0)
- AI-assisted conversion suggestions
- Automatic refactoring recommendations
- Migration verification tools

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_account() {
        let rust_code = r#"
            #[account]
            pub struct Player {
                pub wallet: Pubkey,
            }
        "#;

        let result = extract(rust_code, Default::default()).unwrap();

        assert!(result.contains("struct Player"));
        assert!(result.contains("wallet: PublicKey"));
    }

    #[test]
    fn test_vec_to_array_conversion() {
        let rust_code = r#"
            #[account]
            pub struct Container {
                pub items: Vec<u64>,
            }
        "#;

        let result = extract(rust_code, Default::default()).unwrap();

        assert!(result.contains("items: [u64]"));
    }
}
```

### Integration Tests

**Test with real Solana projects:**
- Anchor example programs
- Open-source Solana projects
- LUMOS example schemas

---

## Documentation

### CLI Help

```bash
$ lumos extract --help

LUMOS Extract - Convert Rust code to LUMOS schema

USAGE:
    lumos extract [OPTIONS] <INPUT>

ARGS:
    <INPUT>    Path to Rust file or directory

OPTIONS:
    -o, --output <FILE>       Output file path (default: stdout)
    --filter <TYPE>           Filter by type (account, borsh, all)
    -r, --recursive           Process directories recursively
    --dry-run                 Show what would be extracted
    -i, --interactive         Interactive mode
    --preserve-comments       Include Rust doc comments
    --add-metadata            Add extraction metadata
    --merge <FILE>            Merge with existing LUMOS file
    -v, --verbose             Verbose output
    -h, --help                Print help information

EXAMPLES:
    lumos extract src/state.rs
    lumos extract src/state.rs --output schema.lumos
    lumos extract programs/game/src/ --recursive
    lumos extract src/state.rs --interactive
    lumos extract src/state.rs --filter account --dry-run

For more information, see: https://github.com/RECTOR-LABS/lumos/docs
```

---

## Implementation Roadmap

### Milestone 1: Parser & Analyzer (Week 1)
- [ ] Implement Rust AST parser
- [ ] Detect #[account] structs
- [ ] Detect Borsh derives
- [ ] Basic type mapping

### Milestone 2: Converter (Week 2)
- [ ] Rust → LUMOS AST conversion
- [ ] Type path mapping
- [ ] Field conversion
- [ ] Attribute handling

### Milestone 3: CLI Integration (Week 3)
- [ ] Add `extract` subcommand
- [ ] Implement command-line options
- [ ] File I/O handling
- [ ] Error reporting

### Milestone 4: Advanced Features (Week 4)
- [ ] Interactive mode
- [ ] Comment preservation
- [ ] Merge functionality
- [ ] Comprehensive testing

### Milestone 5: Polish & Release (Week 5)
- [ ] Documentation
- [ ] Examples
- [ ] Integration with existing LUMOS commands
- [ ] Release v0.2.0 with extraction support

---

**Status:** Design Complete
**Next Step:** Implementation (Phase 3.3)
**Estimated Effort:** 4-5 weeks
**Priority:** High (critical for existing project adoption)
